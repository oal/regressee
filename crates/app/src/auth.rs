// SPDX-License-Identifier: MPL-2.0
//! Blocking auth owner used by Jobs. Secrets never reach UI replies/evidence.
use crate::product::Product;
use anyhow::{Context, Result, ensure};
use regressee_core::{domain::*, environment::map_url};
use regressee_runtime::{
    distribution,
    protocol::{Command, EventKind, FailureCode, Request, SecretPayload},
    secrets::{EncryptedSessions, SecretError, SessionMaterial, SessionSecrets},
    supervisor::{ProcessSupervisor, Worker, WorkerFailure},
};
use regressee_storage::{LocalLoginSettings, Store};
use std::{
    collections::BTreeMap,
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};

#[derive(Default)]
pub struct AuthOwner {
    vault: Option<EncryptedSessions>,
    ephemeral: BTreeMap<SessionId, EphemeralSession>,
    login: Option<Login>,
    pub(crate) probe: Option<String>,
}
struct EphemeralSession {
    // Trusted owner metadata supplies the identity binding that persistent state
    // gets from AEAD AAD. A copied local reference must not transplant memory.
    scope: SessionReference,
    material: SessionMaterial,
}
struct Login {
    worker: Worker,
    scope: SessionReference,
    ephemeral: bool,
    opened: Instant,
    settings_snapshot: String,
}
pub struct AuthState {
    pub scope: SessionReference,
    pub origins: Vec<String>,
    pub material: SessionMaterial,
}
impl AuthState {
    pub fn install(&self, worker: &mut Worker, job: RunId, cancel: Arc<AtomicBool>) -> Result<()> {
        worker.request(
            Request {
                version: 2,
                request_id: 7000,
                job,
                command: Command::InstallSession {
                    session: self.scope.clone(),
                    origins: self.origins.clone(),
                    material: SecretPayload(String::from_utf8(self.material.expose().to_vec())?),
                },
            },
            Instant::now() + Duration::from_secs(10),
            cancel,
            &mut |_| Ok(()),
        )
    }
}
#[derive(Debug, Clone)]
pub struct EnvironmentStatus {
    pub environment: EnvironmentId,
    pub name: String,
    pub settings: LocalLoginSettings,
    pub message: String,
    pub login_open: bool,
    pub mapped: Vec<(String, String)>,
    pub probed: bool,
}
fn origins(env: &Environment) -> Vec<String> {
    env.mappings
        .iter()
        .map(|m| format!("{}/", m.destination.origin().ascii_serialization()))
        .collect::<std::collections::BTreeSet<_>>()
        .into_iter()
        .collect()
}
// Private encrypted payload only: never stored in project/run/export metadata.
// v2 deliberately does not interpret or upgrade an unbound legacy payload.
#[derive(serde::Serialize, serde::Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct SessionBinding {
    origins: Vec<String>,
    access: AccessPolicy,
}
impl SessionBinding {
    fn intended(env: &Environment) -> Self {
        Self {
            origins: origins(env),
            access: env.access,
        }
    }
}
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct BoundSession {
    version: u8,
    binding: SessionBinding,
    material: String,
}
impl Drop for BoundSession {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.material.zeroize();
    }
}
fn needs_login(message: &'static str) -> anyhow::Error {
    anyhow::Error::new(WorkerFailure(FailureCode::NeedsLogin)).context(message)
}
fn worker_binding_matches(material: &[u8], approved: &[String]) -> bool {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Binding {
        version: u8,
        origins: Vec<String>,
    }
    #[derive(serde::Deserialize)]
    struct State {
        binding: Binding,
    }
    let Ok(state) = serde_json::from_slice::<State>(material) else {
        return false;
    };
    state.binding.version == 1 && state.binding.origins == approved
}
impl AuthOwner {
    /// Explicit development-test injection, never selected by project/import/env
    /// configuration. Tests must supply their own provider and owned fixtures.
    #[cfg(feature = "test-support")]
    pub fn with_test_vault(vault: EncryptedSessions) -> Self {
        Self {
            vault: Some(vault),
            ..Default::default()
        }
    }
    #[cfg(feature = "test-support")]
    pub fn save_fixture_session(
        &mut self,
        store: &mut Store,
        scope: &SessionReference,
        material: &SessionMaterial,
        ephemeral: bool,
    ) -> Result<()> {
        let project = store.load_project(scope.project)?.project;
        let env = project
            .environments
            .iter()
            .find(|e| e.id == scope.environment)
            .context("Unknown fixture environment")?;
        ensure!(
            env.mappings.iter().all(
                |m| distribution::loopback(&m.source) && distribution::loopback(&m.destination)
            ),
            "Test session injection permits owned literal-loopback fixtures only"
        );
        ensure!(
            material.expose().len() <= 512 * 1024,
            "Fixture state exceeds private transfer budget"
        );
        // Explicit fixture establishment is fresh consent, not legacy migration.
        let mut state: serde_json::Value = serde_json::from_slice(material.expose())?;
        state["binding"] = serde_json::json!({"version":1,"origins":origins(env)});
        self.replace_session(
            store,
            scope,
            SessionMaterial::new(serde_json::to_vec(&state)?)?,
            ephemeral,
        )
    }
    fn remove_material(
        &mut self,
        store: &Store,
        project: ProjectId,
        environment: EnvironmentId,
        id: SessionId,
        ephemeral: bool,
    ) -> Result<()> {
        // Only the session ID comes from a local reference. Ownership comes from
        // the actual operation's project/environment, never reference claims.
        // Do not use auth_state: expired/origin-changed owners may still Forget.
        let scope = SessionReference {
            id,
            project,
            environment,
        };
        if ephemeral {
            if let Some(saved) = self.ephemeral.get(&id) {
                if saved.scope != scope {
                    return Err(SecretError::ScopeMismatch.into());
                }
                self.ephemeral.remove(&scope.id);
            }
        } else {
            match self.vault(store)?.delete(&scope) {
                Ok(()) | Err(SecretError::Missing) => (),
                Err(e) => return Err(e.into()),
            }
        }
        Ok(())
    }
    fn replace_session(
        &mut self,
        store: &mut Store,
        scope: &SessionReference,
        material: SessionMaterial,
        ephemeral: bool,
    ) -> Result<()> {
        ensure!(
            material.expose().len() <= 512 * 1024,
            "Browser state exceeds the private transfer budget"
        );
        let project = store.load_project(scope.project)?.project;
        let env = project
            .environments
            .iter()
            .find(|e| e.id == scope.environment)
            .context("Unknown login environment")?;
        let binding = SessionBinding::intended(env);
        ensure!(
            worker_binding_matches(material.expose(), &binding.origins),
            "The login does not match the approved websites. Log in again."
        );
        let bound = BoundSession {
            version: 2,
            binding,
            material: String::from_utf8(material.expose().to_vec())?,
        };
        let material = SessionMaterial::new(serde_json::to_vec(&bound)?)?;
        ensure!(
            material.expose().len() <= 512 * 1024,
            "Bound browser state exceeds the private transfer budget"
        );
        let previous = store.session_reference(scope.project, scope.environment)?;
        ensure!(
            previous.as_ref().is_none_or(|(s, _)| s.id != scope.id),
            "Re-login requires a fresh session identity"
        );
        if ephemeral {
            let replacing = previous.as_ref().is_some_and(|(s, e)| {
                *e && self.ephemeral.get(&s.id).is_some_and(|v| &v.scope == s)
            });
            ensure!(
                self.ephemeral.len() < 16 || replacing,
                "The 16 ephemeral-login limit is in use. Forget an unused environment's login first."
            );
            self.ephemeral.insert(
                scope.id,
                EphemeralSession {
                    scope: scope.clone(),
                    material,
                },
            );
        } else {
            self.vault(store)?.save(scope, &material)?;
        }
        if let Err(e) = store.save_session_reference(scope, ephemeral) {
            self.remove_material(store, project.id, env.id, scope.id, ephemeral)?;
            return Err(e);
        }
        self.probe = None;
        if let Some((old, mode)) = previous {
            self.remove_material(store, project.id, env.id, old.id, mode)
                .context("New login saved, but superseded private state could not be removed")?;
        }
        Ok(())
    }
    fn vault(&mut self, store: &Store) -> Result<&mut EncryptedSessions> {
        if self.vault.is_none() {
            self.vault = Some(EncryptedSessions::open(
                &store.paths().data.join("private-sessions"),
            )?);
        }
        Ok(self.vault.as_mut().unwrap())
    }
}
impl Product {
    pub fn environment_status(
        &self,
        store: &Store,
        project: ProjectId,
    ) -> Result<EnvironmentStatus> {
        let p = store.load_project(project)?.project;
        let env = store.selected_environment(&p)?;
        let settings = store.local_login_settings(project)?;
        let saved = store.session_reference(project, env.id)?;
        let invalid = if saved.is_some() {
            self.auth_state(store, project, &env)
                .err()
                .map(|e| e.to_string())
        } else {
            None
        };
        let mut auth = self.auth.lock().unwrap();
        let token = probe_token(store, project, &env)?;
        let snapshot = format!("{}:{}", token, serde_json::to_string(&settings)?);
        if auth.login.as_ref().is_some_and(|l| {
            l.scope.project == project
                && (l.scope.environment != env.id || l.settings_snapshot != snapshot)
        }) && let Some(mut login) = auth.login.take()
        {
            let _ = login.worker.stop(Instant::now() + Duration::from_secs(3));
        }
        if invalid.is_some() {
            auth.probe = None;
        }
        let message=match saved {
            Some(_) if invalid.is_some()=>invalid.as_ref().unwrap().clone(),
            Some((s,true)) if auth.ephemeral.contains_key(&s.id)=>"Login is available only for this app session, until Regressee closes. Not saved to disk.".into(),
            Some((_,true))=>"Your temporary login session ended. Log in again.".into(),
            Some((_,false))=>"Login saved securely. Regressee checks that it still works when visiting pages.".into(),
            None if env.access==AccessPolicy::InteractiveLogin=>"Log in before finding or capturing pages.".into(),
            None=>"Public access selected. You can still set up consent in the browser.".into(),
        };
        let mapped = self.mapped_preview(store, project, &env)?;
        Ok(EnvironmentStatus {
            environment: env.id,
            name: env.name,
            settings,
            message,
            login_open: auth.login.as_ref().is_some_and(|l| {
                l.scope.project == project
                    && l.scope.environment == env.id
                    && l.opened.elapsed() < Duration::from_secs(15 * 60)
            }),
            mapped,
            probed: auth.probe.as_ref() == Some(&token),
        })
    }
    pub(crate) fn auth_state(
        &self,
        store: &Store,
        project: ProjectId,
        env: &Environment,
    ) -> Result<Option<AuthState>> {
        let mut owner = self.auth.lock().unwrap();
        // The caller may hold an immutable run snapshot. Its authority must also
        // agree with today's saved environment, including when no reference exists.
        let current = store.load_project(project)?.project;
        let current = current
            .environments
            .iter()
            .find(|e| e.id == env.id)
            .ok_or_else(|| needs_login("Login environment no longer exists. Log in again."))?;
        if SessionBinding::intended(current) != SessionBinding::intended(env) {
            owner.probe = None;
            return Err(needs_login(
                "Environment security settings changed. Log in again before visits.",
            ));
        }
        let reference = store.session_reference(project, env.id)?;
        // A bad supplied non-null reference is never discarded into anonymous work.
        if let Some(ref supplied) = env.session
            && !reference.as_ref().is_some_and(|(s, _)| s == supplied)
        {
            return Err(needs_login(
                "The saved login could not be verified. Log in again.",
            ));
        }
        let Some((scope, ephemeral)) = reference else {
            if env.access != AccessPolicy::Public {
                return Err(needs_login(
                    "Log in again before continuing this environment",
                ));
            }
            return Ok(None);
        };
        let material = if ephemeral {
            let saved = owner
                .ephemeral
                .get(&scope.id)
                .ok_or_else(|| needs_login("Your temporary login session ended. Log in again."))?;
            if saved.scope != scope {
                return Err(needs_login(
                    "This temporary login belongs to another environment. Log in again.",
                ));
            }
            SessionMaterial::new(saved.material.expose().to_vec())?
        } else {
            owner.vault(store)?.load(&scope).map_err(|e| {
                anyhow::Error::new(WorkerFailure(FailureCode::NeedsLogin)).context(e)
            })?
        };
        let bound = serde_json::from_slice::<BoundSession>(material.expose()).map_err(|_| {
            owner.probe = None;
            needs_login(
                "This saved login is no longer compatible. Log in again before opening pages.",
            )
        })?;
        if bound.version != 2
            || bound.binding != SessionBinding::intended(current)
            || !worker_binding_matches(bound.material.as_bytes(), &bound.binding.origins)
        {
            owner.probe = None;
            return Err(needs_login(
                "The saved login does not match this environment's websites or access settings. Log in again before opening pages.",
            ));
        }
        Ok(Some(AuthState {
            scope,
            origins: bound.binding.origins.clone(),
            material: SessionMaterial::new(bound.material.as_bytes().to_vec())?,
        }))
    }
    pub fn open_login(
        &self,
        store: &Store,
        project: ProjectId,
        cancel: Arc<AtomicBool>,
    ) -> Result<EnvironmentStatus> {
        let p = store.load_project(project)?.project;
        let env = store.selected_environment(&p)?;
        let settings = store.local_login_settings(project)?;
        settings.validate()?;
        let representative = settings
            .representative
            .clone()
            .unwrap_or(map_url(&p.sources[0], &env)?);
        let address = settings
            .login_address
            .clone()
            .unwrap_or(representative.clone());
        let approved = origins(&env);
        ensure!(
            approved.contains(&format!(
                "{}/",
                representative.origin().ascii_serialization()
            )),
            "Representative page must be on a mapped website, not the identity provider"
        );
        let installed = self.resolve_runtime(&cancel)?;
        for u in std::iter::once(&address)
            .chain(std::iter::once(&representative))
            .chain(settings.identity_origins.iter())
        {
            ensure!(
                installed.manifest.scope != distribution::Scope::DevelopmentLoopback
                    || distribution::loopback(u),
                "This development runtime permits owned literal-loopback fixtures only"
            );
        }
        let scope = SessionReference {
            id: SessionId::new(),
            project,
            environment: env.id,
        };
        let mut owner = self.auth.lock().unwrap();
        ensure!(
            owner.login.is_none(),
            "A login browser is already open. Finish or close it first."
        );
        if !settings.ephemeral {
            owner.vault(store)?.available()?;
        }
        let mut worker = self.spawn_worker(&installed)?;
        let job = RunId::new();
        worker.handshake(job, cancel.clone())?;
        worker.request(
            Request {
                version: 2,
                request_id: 3,
                job,
                command: Command::OpenLogin {
                    session: scope.clone(),
                    address: address.to_string(),
                    representative: representative.to_string(),
                    origins: approved,
                    identity_origins: settings
                        .identity_origins
                        .iter()
                        .map(ToString::to_string)
                        .collect(),
                },
            },
            Instant::now() + Duration::from_secs(40),
            cancel,
            &mut |_| Ok(()),
        )?;
        owner.probe = None;
        owner.login = Some(Login {
            worker,
            scope,
            ephemeral: settings.ephemeral,
            opened: Instant::now(),
            settings_snapshot: format!(
                "{}:{}",
                probe_token(store, project, &env)?,
                serde_json::to_string(&settings)?
            ),
        });
        drop(owner);
        self.environment_status(store, project)
    }
    pub fn confirm_login(
        &self,
        store: &mut Store,
        project: ProjectId,
        cancel: Arc<AtomicBool>,
    ) -> Result<EnvironmentStatus> {
        let mut owner = self.auth.lock().unwrap();
        let mut login = owner
            .login
            .take()
            .context("Open the browser before choosing Confirm login")?;
        ensure!(
            login.scope.project == project,
            "Login belongs to another project"
        );
        ensure!(
            login.opened.elapsed() < Duration::from_secs(15 * 60),
            "Login setup expired. Open browser again."
        );
        let saved = store.load_project(project)?.project;
        let selected = store.selected_environment(&saved)?;
        ensure!(
            selected.id == login.scope.environment
                && login.settings_snapshot
                    == format!(
                        "{}:{}",
                        probe_token(store, project, &selected)?,
                        serde_json::to_string(&store.local_login_settings(project)?)?
                    ),
            "Environment settings changed while logging in. Open a fresh browser for the saved environment."
        );
        let mut material = None;
        let scope = login.scope.clone();
        let result = login.worker.request(
            Request {
                version: 2,
                request_id: 5,
                job: RunId::new(),
                command: Command::ConfirmLogin {
                    session: scope.clone(),
                },
            },
            Instant::now() + Duration::from_secs(40),
            cancel,
            &mut |e| {
                if let EventKind::SessionReady { material: m } = e.event {
                    material = Some(SessionMaterial::new(m.0.as_bytes().to_vec())?);
                }
                Ok(())
            },
        );
        if let Err(e) = result {
            owner.login = Some(login);
            return Err(e).context("Your saved login could not open the test page in a separate browser session. Finish login or cookie consent, then choose Confirm login again. Some websites do not allow saved logins to be reused.");
        }
        let material = material.context("Browser did not return confirmed state")?;
        owner.replace_session(store, &scope, material, login.ephemeral)?;
        login.worker.stop(Instant::now() + Duration::from_secs(3))?;
        drop(owner);
        self.environment_status(store, project)
    }
    pub fn close_login(&self, store: &Store, project: ProjectId) -> Result<EnvironmentStatus> {
        let mut owner = self.auth.lock().unwrap();
        if let Some(mut l) = owner.login.take() {
            let _ = l.worker.stop(Instant::now() + Duration::from_secs(3));
        }
        drop(owner);
        self.environment_status(store, project)
    }
    pub fn forget_login(&self, store: &mut Store, project: ProjectId) -> Result<EnvironmentStatus> {
        let p = store.load_project(project)?.project;
        let env = store.selected_environment(&p)?;
        let mut owner = self.auth.lock().unwrap();
        if let Some((s, e)) = store.session_reference(project, env.id)? {
            owner.remove_material(store, p.id, env.id, s.id, e)?;
        }
        store.forget_session_reference(project, env.id)?;
        owner.probe = None;
        drop(owner);
        self.environment_status(store, project)
    }
    fn mapped_preview(
        &self,
        store: &Store,
        project: ProjectId,
        env: &Environment,
    ) -> Result<Vec<(String, String)>> {
        let p = store.load_project(project)?.project;
        let (addresses, discovery) = if let Some(id) = p.active_baseline {
            let baseline = store.load_baseline(id)?;
            let source = store.load_run(baseline.references[0].run)?;
            let discovery = source.queue.map(|q| q.discovery).unwrap_or_default();
            (
                source
                    .run
                    .inventory
                    .pages
                    .into_iter()
                    .filter(|p| p.included)
                    .map(|p| p.canonical)
                    .collect(),
                discovery,
            )
        } else {
            (p.sources, p.discovery)
        };
        addresses
            .iter()
            .map(|u| {
                Ok((
                    u.to_string(),
                    regressee_core::environment::guarded_map(u, env, &discovery)?.to_string(),
                ))
            })
            .collect()
    }
    pub fn probe_environment(
        &self,
        store: &Store,
        project: ProjectId,
        cancel: Arc<AtomicBool>,
    ) -> Result<EnvironmentStatus> {
        let p = store.load_project(project)?.project;
        let env = store.selected_environment(&p)?;
        let state = self.auth_state(store, project, &env)?;
        let installed = self.resolve_runtime(&cancel)?;
        let mut worker = self.spawn_worker(&installed)?;
        let job = RunId::new();
        worker.handshake(job, cancel.clone())?;
        if let Some(s) = &state {
            s.install(&mut worker, job, cancel.clone())?;
        }
        let addresses = self.mapped_preview(store, project, &env)?;
        for (i, (_, address)) in addresses.iter().take(3).enumerate() {
            let command = if let Some(s) = &state {
                Command::DiscoverAuthenticated {
                    address: address.clone(),
                    section: None,
                    session: s.scope.clone(),
                }
            } else {
                Command::DiscoverPage {
                    address: address.clone(),
                    section: None,
                }
            };
            let mut redirect = false;
            worker.request(
                Request {
                    version: 2,
                    request_id: 9 + i as u64 * 2,
                    job,
                    command,
                },
                Instant::now() + Duration::from_secs(36),
                cancel.clone(),
                &mut |e| {
                    if matches!(e.event, EventKind::RedirectFound { .. }) {
                        redirect = true;
                    }
                    Ok(())
                },
            )?;
            ensure!(
                !redirect,
                "Mapped page redirected. Confirm the mapping/login before checking."
            );
        }
        worker.stop(Instant::now() + Duration::from_secs(3))?;
        self.auth.lock().unwrap().probe = Some(probe_token(store, project, &env)?);
        self.environment_status(store, project)
    }
    pub(crate) fn require_probe(
        &self,
        store: &Store,
        project: ProjectId,
        env: &Environment,
    ) -> Result<()> {
        // Identity mappings still expose preview/probe; changed maps must be probed.
        if env.mappings.iter().any(|m| m.source != m.destination) {
            ensure!(
                self.auth.lock().unwrap().probe.as_ref()
                    == Some(&probe_token(store, project, env)?),
                "Preview mapped URLs and check access to sample pages before running the full check"
            );
        }
        Ok(())
    }
}
fn probe_token(store: &Store, project: ProjectId, env: &Environment) -> Result<String> {
    let p = store.load_project(project)?;
    Ok(format!(
        "{}:{}:{}:{}",
        p.revision,
        serde_json::to_string(env)?,
        p.project
            .active_baseline
            .map(|id| id.to_string())
            .unwrap_or_default(),
        store
            .session_reference(project, env.id)?
            .map(|(s, _)| s.id.to_string())
            .unwrap_or_default()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use regressee_runtime::secrets::KeyProvider;
    use regressee_storage::AppPaths;
    struct FixtureKey;
    impl KeyProvider for FixtureKey {
        fn key(&mut self) -> std::result::Result<zeroize::Zeroizing<[u8; 32]>, SecretError> {
            Ok(zeroize::Zeroizing::new([11; 32]))
        }
    }
    fn fixture_material(env: &Environment) -> Result<SessionMaterial> {
        Ok(SessionMaterial::new(serde_json::to_vec(
            &serde_json::json!({
                "binding":{"version":1,"origins":origins(env)},
                "storage":{"cookies":[],"origins":[]},"session_storage":{}
            }),
        )?)?)
    }
    fn assert_needs_login(result: Result<Option<AuthState>>) {
        let error = result.err().expect("must refuse invalid saved login");
        assert_eq!(
            error.downcast_ref::<WorkerFailure>().unwrap().0,
            FailureCode::NeedsLogin
        );
    }
    #[test]
    fn encrypted_binding_restart_legacy_wrongbinding_tamper_and_policy_are_closed() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut store = Store::open(AppPaths::isolated(root.path()))?;
        let mut p = Project::draft("Binding".into(), "http://127.0.0.1:8010/auth")?;
        p.environments[0].access = AccessPolicy::InteractiveLogin;
        store.save_project(&p, None)?;
        let vault_path = root.path().join("vault");
        let owner = || -> Result<AuthOwner> {
            Ok(AuthOwner {
                vault: Some(EncryptedSessions::with_provider(
                    &vault_path,
                    Box::new(FixtureKey),
                )?),
                ..Default::default()
            })
        };
        let product = Product {
            distribution: None,
            runtime_cache: root.path().join("cache"),
            official_runtime: false,
            auth: std::sync::Mutex::new(owner()?),
        };
        let scope = SessionReference {
            id: SessionId::new(),
            project: p.id,
            environment: p.environments[0].id,
        };
        product.auth.lock().unwrap().replace_session(
            &mut store,
            &scope,
            fixture_material(&p.environments[0])?,
            false,
        )?;
        *product.auth.lock().unwrap() = owner()?;
        assert!(
            product
                .auth_state(&store, p.id, &p.environments[0])?
                .is_some()
        );
        // Name and same-origin prefixes are not new origin consent. Probe receipts
        // still include the whole settings/revision and therefore become stale.
        let old = p.environments[0].clone();
        p.environments[0].name = "Renamed".into();
        p.environments[0].mappings[0]
            .destination
            .set_path("/preview");
        let revision = store.load_project(p.id)?.revision;
        store.save_replacement_project(&p, revision)?;
        assert!(
            product
                .auth_state(&store, p.id, &p.environments[0])?
                .is_some()
        );
        for changed in [
            "http://127.0.0.1:8011/preview",
            "https://127.0.0.1:8010/preview",
        ] {
            let mut drift = p.clone();
            drift.environments[0].mappings[0].destination = changed.parse()?;
            let revision = store.load_project(p.id)?.revision;
            store.save_replacement_project(&drift, revision)?;
            assert_needs_login(product.auth_state(&store, p.id, &drift.environments[0]));
            // A frozen caller cannot use yesterday's authority either.
            assert_needs_login(product.auth_state(&store, p.id, &old));
            let status = product.environment_status(&store, p.id)?;
            assert!(
                !status.probed && !status.login_open && status.message.contains("Log in again")
            );
        }
        let revision = store.load_project(p.id)?.revision;
        p.environments[0].access = AccessPolicy::Public;
        store.save_replacement_project(&p, revision)?;
        assert_needs_login(product.auth_state(&store, p.id, &p.environments[0]));
        p.environments[0].access = AccessPolicy::InteractiveLogin;
        let revision = store.load_project(p.id)?.revision;
        store.save_replacement_project(&p, revision)?;
        // Explicitly create encrypted *legacy* state without the bound owner.
        let legacy = SessionReference {
            id: SessionId::new(),
            ..scope.clone()
        };
        let mut vault = EncryptedSessions::with_provider(&vault_path, Box::new(FixtureKey))?;
        vault.save(
            &legacy,
            &SessionMaterial::new(
                br#"{"storage":{"cookies":[],"origins":[]},"session_storage":{}}"#.to_vec(),
            )?,
        )?;
        store.save_session_reference(&legacy, false)?;
        let legacy_file = vault_path.join(format!("{}.session", legacy.id));
        let original = std::fs::read(&legacy_file)?;
        assert_needs_login(product.auth_state(&store, p.id, &p.environments[0]));
        assert_eq!(std::fs::read(&legacy_file)?, original); // no migration/write
        let wrong = SessionReference {
            id: SessionId::new(),
            ..scope.clone()
        };
        let bound = BoundSession {
            version: 2,
            binding: SessionBinding {
                origins: vec!["http://127.0.0.1:9999/".into()],
                access: p.environments[0].access,
            },
            material: String::from_utf8(fixture_material(&p.environments[0])?.expose().to_vec())?,
        };
        vault.save(&wrong, &SessionMaterial::new(serde_json::to_vec(&bound)?)?)?;
        store.save_session_reference(&wrong, false)?;
        assert_needs_login(product.auth_state(&store, p.id, &p.environments[0]));
        let wrong_inner = SessionReference {
            id: SessionId::new(),
            ..scope.clone()
        };
        let mut changed = p.environments[0].clone();
        changed.mappings[0].destination = "http://127.0.0.1:9998/".parse()?;
        let bound = BoundSession {
            version: 2,
            binding: SessionBinding::intended(&p.environments[0]),
            material: String::from_utf8(fixture_material(&changed)?.expose().to_vec())?,
        };
        vault.save(
            &wrong_inner,
            &SessionMaterial::new(serde_json::to_vec(&bound)?)?,
        )?;
        store.save_session_reference(&wrong_inner, false)?;
        assert_needs_login(product.auth_state(&store, p.id, &p.environments[0]));
        store.save_session_reference(&scope, false)?;
        let file = vault_path.join(format!("{}.session", scope.id));
        let mut bytes = std::fs::read(&file)?;
        bytes[20] ^= 1;
        std::fs::write(&file, bytes)?;
        assert_needs_login(product.auth_state(&store, p.id, &p.environments[0]));
        Ok(())
    }
    #[test]
    fn ephemeral_binding_and_current_policy_cannot_fall_back_anonymously() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut store = Store::open(AppPaths::isolated(root.path()))?;
        let mut p = Project::draft("Ephemeral binding".into(), "http://127.0.0.1:8010/auth")?;
        p.environments[0].access = AccessPolicy::InteractiveLogin;
        store.save_project(&p, None)?;
        let product = Product {
            distribution: None,
            runtime_cache: root.path().join("cache"),
            official_runtime: false,
            auth: Default::default(),
        };
        let scope = SessionReference {
            id: SessionId::new(),
            project: p.id,
            environment: p.environments[0].id,
        };
        product.auth.lock().unwrap().replace_session(
            &mut store,
            &scope,
            fixture_material(&p.environments[0])?,
            true,
        )?;
        assert!(
            product
                .auth_state(&store, p.id, &p.environments[0])?
                .is_some()
        );
        let old = p.environments[0].clone();
        p.environments[0].mappings[0].destination = "http://127.0.0.1:8011/auth".parse()?;
        let revision = store.load_project(p.id)?.revision;
        store.save_replacement_project(&p, revision)?;
        assert_needs_login(product.auth_state(&store, p.id, &p.environments[0]));
        p.environments[0].access = AccessPolicy::Public;
        let revision = store.load_project(p.id)?.revision;
        store.save_replacement_project(&p, revision)?;
        assert_needs_login(product.auth_state(&store, p.id, &p.environments[0]));
        product.forget_login(&mut store, p.id)?;
        assert_needs_login(product.auth_state(&store, p.id, &old));
        assert!(
            product
                .auth_state(&store, p.id, &p.environments[0])?
                .is_none()
        );
        assert!(!store.paths().data.join("private-sessions").exists());
        Ok(())
    }
    #[test]
    fn binding_normalizes_default_ports_case_order_duplicates_but_not_scheme() -> Result<()> {
        let mut p = Project::draft("Canonical".into(), "https://EXAMPLE.test:443/auth")?;
        let initial = SessionBinding::intended(&p.environments[0]);
        p.environments[0].mappings[0].destination = "https://example.test/preview".parse()?;
        let duplicate = p.environments[0].mappings[0].clone();
        p.environments[0].mappings.push(duplicate);
        assert!(initial == SessionBinding::intended(&p.environments[0]));
        p.environments[0].mappings[0].destination = "http://example.test:80/preview".parse()?;
        assert!(initial != SessionBinding::intended(&p.environments[0]));
        p.environments[0].mappings.reverse();
        assert_eq!(
            origins(&p.environments[0]),
            vec!["http://example.test/", "https://example.test/"]
        );
        Ok(())
    }
    #[test]
    fn ephemeral_reference_cannot_be_transplanted_between_same_origin_environments() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut store = Store::open(AppPaths::isolated(root.path()))?;
        let mut p = Project::draft("Scope binding".into(), "http://127.0.0.1:8010/auth")?;
        p.environments[0].access = AccessPolicy::InteractiveLogin;
        let mut other = p.environments[0].clone();
        other.id = EnvironmentId::new();
        other.name = "Staging".into();
        p.environments.push(other.clone());
        store.save_project(&p, None)?;
        let product = Product {
            distribution: None,
            runtime_cache: root.path().join("cache"),
            official_runtime: false,
            auth: Default::default(),
        };
        let scope = SessionReference {
            id: SessionId::new(),
            project: p.id,
            environment: p.environments[0].id,
        };
        product.auth.lock().unwrap().replace_session(
            &mut store,
            &scope,
            fixture_material(&p.environments[0])?,
            true,
        )?;
        let transplanted = SessionReference {
            environment: other.id,
            ..scope.clone()
        };
        store.save_session_reference(&transplanted, true)?;
        assert_needs_login(product.auth_state(&store, p.id, &other));
        store.select_environment(p.id, other.id, &Default::default())?;
        let error = product.forget_login(&mut store, p.id).unwrap_err();
        assert_eq!(
            error.downcast_ref::<SecretError>(),
            Some(&SecretError::ScopeMismatch)
        );
        assert_eq!(
            store.session_reference(p.id, other.id)?,
            Some((transplanted, true))
        );
        assert!(
            product
                .auth_state(&store, p.id, &p.environments[0])?
                .is_some()
        );
        assert!(!store.paths().data.join("private-sessions").exists());
        Ok(())
    }
    #[test]
    fn forget_authenticates_actual_owner_not_copied_or_forged_reference_claims() -> Result<()> {
        for ephemeral in [false, true] {
            let root = tempfile::tempdir()?;
            let mut store = Store::open(AppPaths::isolated(root.path()))?;
            let mut p = Project::draft("Forget owner".into(), "http://127.0.0.1:8010/auth")?;
            p.environments[0].access = AccessPolicy::InteractiveLogin;
            let mut b = p.environments[0].clone();
            b.id = EnvironmentId::new();
            b.name = "Borrower".into();
            p.environments.push(b.clone());
            store.save_project(&p, None)?;
            let q = Project::draft("Foreign project".into(), "http://127.0.0.1:8010/auth")?;
            store.save_project(&q, None)?;
            let vault_path = root.path().join("vault");
            let product = Product {
                distribution: None,
                runtime_cache: root.path().join("cache"),
                official_runtime: false,
                auth: std::sync::Mutex::new(AuthOwner {
                    vault: Some(EncryptedSessions::with_provider(
                        &vault_path,
                        Box::new(FixtureKey),
                    )?),
                    ..Default::default()
                }),
            };
            let a = SessionReference {
                id: SessionId::new(),
                project: p.id,
                environment: p.environments[0].id,
            };
            let unrelated = SessionReference {
                id: SessionId::new(),
                project: q.id,
                environment: q.environments[0].id,
            };
            for (scope, env) in [(&a, &p.environments[0]), (&unrelated, &q.environments[0])] {
                product.auth.lock().unwrap().replace_session(
                    &mut store,
                    scope,
                    fixture_material(env)?,
                    ephemeral,
                )?;
            }
            let a_file = vault_path.join(format!("{}.session", a.id));
            let unrelated_file = vault_path.join(format!("{}.session", unrelated.id));
            let bytes = (!ephemeral).then(|| std::fs::read(&a_file)).transpose()?;
            let unrelated_bytes = (!ephemeral)
                .then(|| std::fs::read(&unrelated_file))
                .transpose()?;
            assert!(
                product
                    .auth_state(&store, p.id, &p.environments[0])?
                    .is_some()
            );
            for (project, env) in [(p.id, &b), (q.id, &q.environments[0])] {
                let copied = SessionReference {
                    id: a.id,
                    project,
                    environment: env.id,
                };
                store.save_session_reference(&copied, ephemeral)?;
                store.select_environment(project, env.id, &Default::default())?;
                assert_needs_login(product.auth_state(&store, project, env));
                let error = product.forget_login(&mut store, project).unwrap_err();
                let expected = if ephemeral {
                    SecretError::ScopeMismatch
                } else {
                    SecretError::Corrupt
                };
                assert_eq!(error.downcast_ref::<SecretError>(), Some(&expected));
                assert_eq!(
                    store.session_reference(project, env.id)?,
                    Some((copied, ephemeral))
                );
                // A forged reference can claim A's owner fields, but the remover
                // accepts only its ID and the operation's trusted B context.
                let forged = a.clone();
                let error = product
                    .auth
                    .lock()
                    .unwrap()
                    .remove_material(&store, project, env.id, forged.id, ephemeral)
                    .unwrap_err();
                assert_eq!(error.downcast_ref::<SecretError>(), Some(&expected));
                assert_eq!(
                    store.session_reference(p.id, a.environment)?,
                    Some((a.clone(), ephemeral))
                );
                if let Some(ref bytes) = bytes {
                    assert_eq!(&std::fs::read(&a_file)?, bytes);
                }
                assert!(
                    product
                        .auth_state(&store, p.id, &p.environments[0])?
                        .is_some()
                );
            }
            store.save_session_reference(&unrelated, ephemeral)?;
            // Replacement/pruning must not revoke a borrowed prior identity.
            // The fresh login is saved, but cleanup reports its explicit failure.
            let fresh_b = SessionReference {
                id: SessionId::new(),
                project: p.id,
                environment: b.id,
            };
            let error = product
                .auth
                .lock()
                .unwrap()
                .replace_session(&mut store, &fresh_b, fixture_material(&b)?, ephemeral)
                .unwrap_err();
            assert_eq!(
                error.to_string(),
                "New login saved, but superseded private state could not be removed"
            );
            assert!(product.auth_state(&store, p.id, &b)?.is_some());
            if let Some(ref bytes) = bytes {
                assert_eq!(&std::fs::read(&a_file)?, bytes);
            }
            assert!(
                product
                    .auth_state(&store, p.id, &p.environments[0])?
                    .is_some()
            );
            // Change origin/access: Forget checks identity, not access binding.
            p.environments[0].mappings[0].destination = "http://127.0.0.1:8011/auth".parse()?;
            p.environments[0].access = AccessPolicy::Public;
            let revision = store.load_project(p.id)?.revision;
            store.save_replacement_project(&p, revision)?;
            store.select_environment(p.id, a.environment, &Default::default())?;
            assert_needs_login(product.auth_state(&store, p.id, &p.environments[0]));
            product.forget_login(&mut store, p.id)?;
            assert!(store.session_reference(p.id, a.environment)?.is_none());
            assert!(!a_file.exists());
            assert!(!product.auth.lock().unwrap().ephemeral.contains_key(&a.id));
            assert_eq!(
                store.session_reference(q.id, unrelated.environment)?,
                Some((unrelated.clone(), ephemeral))
            );
            if let Some(ref bytes) = unrelated_bytes {
                assert_eq!(&std::fs::read(&unrelated_file)?, bytes);
            }
            assert!(
                product
                    .auth_state(&store, q.id, &q.environments[0])?
                    .is_some()
            );
        }
        Ok(())
    }
    #[test]
    fn locked_or_corrupt_forget_preserves_references_and_all_ciphertext() -> Result<()> {
        struct LockedKey;
        impl KeyProvider for LockedKey {
            fn key(&mut self) -> std::result::Result<zeroize::Zeroizing<[u8; 32]>, SecretError> {
                Err(SecretError::SecureStoreUnavailable)
            }
        }
        let root = tempfile::tempdir()?;
        let mut store = Store::open(AppPaths::isolated(root.path()))?;
        let p = Project::draft("Locked Forget".into(), "http://127.0.0.1:8010/auth")?;
        store.save_project(&p, None)?;
        let scope = SessionReference {
            id: SessionId::new(),
            project: p.id,
            environment: p.environments[0].id,
        };
        let vault_path = root.path().join("vault");
        let mut vault = EncryptedSessions::with_provider(&vault_path, Box::new(FixtureKey))?;
        vault.save(&scope, &fixture_material(&p.environments[0])?)?;
        store.save_session_reference(&scope, false)?;
        let file = vault_path.join(format!("{}.session", scope.id));
        let bytes = std::fs::read(&file)?;
        let product = Product {
            distribution: None,
            runtime_cache: root.path().join("cache"),
            official_runtime: false,
            auth: std::sync::Mutex::new(AuthOwner {
                vault: Some(EncryptedSessions::with_provider(
                    &vault_path,
                    Box::new(LockedKey),
                )?),
                ..Default::default()
            }),
        };
        let error = product.forget_login(&mut store, p.id).unwrap_err();
        assert_eq!(
            error.downcast_ref::<SecretError>(),
            Some(&SecretError::SecureStoreUnavailable)
        );
        assert_eq!(std::fs::read(&file)?, bytes);
        assert_eq!(
            store.session_reference(p.id, scope.environment)?,
            Some((scope.clone(), false))
        );
        product.auth.lock().unwrap().vault = Some(vault);
        let mut corrupt = bytes;
        corrupt[20] ^= 1;
        std::fs::write(&file, &corrupt)?;
        let error = product.forget_login(&mut store, p.id).unwrap_err();
        assert_eq!(
            error.downcast_ref::<SecretError>(),
            Some(&SecretError::Corrupt)
        );
        assert_eq!(std::fs::read(&file)?, corrupt);
        assert_eq!(
            store.session_reference(p.id, scope.environment)?,
            Some((scope.clone(), false))
        );
        // Missing ciphertext is the only unauthenticated unlink: there is no
        // material to revoke, and only this actual owner's local row is removed.
        std::fs::remove_file(&file)?;
        product.forget_login(&mut store, p.id)?;
        assert!(store.session_reference(p.id, scope.environment)?.is_none());
        Ok(())
    }
    #[test]
    fn replacing_sessions_prunes_secrets_and_ephemeral_restart_never_falls_back() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut store = Store::open(AppPaths::isolated(root.path()))?;
        let p = Project::draft("Login boundary".into(), "http://127.0.0.1/")?;
        store.save_project(&p, None)?;
        let scope = SessionReference {
            id: SessionId::new(),
            project: p.id,
            environment: p.environments[0].id,
        };
        let vault_root = root.path().join("vault");
        let mut owner = AuthOwner {
            vault: Some(EncryptedSessions::with_provider(
                &vault_root,
                Box::new(FixtureKey),
            )?),
            ..Default::default()
        };
        owner.replace_session(
            &mut store,
            &scope,
            fixture_material(&p.environments[0])?,
            false,
        )?;
        assert!(vault_root.join(format!("{}.session", scope.id)).exists());
        let mut latest = scope.clone();
        for _ in 0..20 {
            let next = SessionReference {
                id: SessionId::new(),
                ..scope.clone()
            };
            owner.replace_session(
                &mut store,
                &next,
                fixture_material(&p.environments[0])?,
                true,
            )?;
            assert_eq!(owner.ephemeral.len(), 1);
            latest = next;
        }
        assert!(!vault_root.join(format!("{}.session", scope.id)).exists());
        assert_eq!(
            store.session_reference(p.id, scope.environment)?,
            Some((latest, true))
        );
        let product = Product {
            distribution: None,
            runtime_cache: root.path().join("cache"),
            official_runtime: false,
            auth: Default::default(),
        };
        assert!(
            product
                .auth_state(&store, p.id, &p.environments[0])
                .is_err()
        );
        assert!(
            product
                .environment_status(&store, p.id)?
                .message
                .contains("ended")
        );
        product.forget_login(&mut store, p.id)?;
        assert!(
            product
                .auth_state(&store, p.id, &p.environments[0])?
                .is_none()
        );
        Ok(())
    }
}
