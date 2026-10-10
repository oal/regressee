// SPDX-License-Identifier: MPL-2.0
//! Real blocking services used by the GPUI actor and headless integration flow.
use anyhow::{Context, Result, ensure};
use regressee_core::{domain::*, evidence::*};
use regressee_runtime::{
    distribution::{self, Distribution, InstallProgress, Installed, Scope},
    protocol::{Command, EventKind, FailureCode, Request},
    supervisor::{ProcessSupervisor, Worker, WorkerFailure},
};
use regressee_storage::{ProjectRecord, Store};
use std::{
    fs::{self, File},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};
pub struct Product {
    pub distribution: Option<Distribution>,
    pub runtime_cache: PathBuf,
    pub auth: std::sync::Mutex<crate::auth::AuthOwner>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeState {
    UnavailableDistribution,
    UnsupportedPlatform,
    NotInstalled,
    Ready,
    RepairRequired,
}
/// UI-safe explanation. Availability means validated configured trust plus a
/// supported adapter, NOT public release/platform qualification.
#[derive(Debug, Clone, PartialEq)]
pub struct RuntimeReadiness {
    pub status: RuntimeState,
    pub setup_available: bool,
    pub message: &'static str,
    pub identity: Option<RuntimeIdentity>,
    pub scope: Option<Scope>,
}
impl RuntimeReadiness {
    pub(crate) fn invalid_configuration() -> Self {
        Self {
            status: RuntimeState::UnavailableDistribution,
            setup_available: false,
            message: "This build's browser trust configuration could not be validated. Browser setup is unavailable; existing evidence is unchanged.",
            identity: None,
            scope: None,
        }
    }
}
fn supported_runtime_platform(os: &str, arch: &str) -> bool {
    // Adapter eligibility only; retain the macOS owned-tree refusal and do not
    // expand the unqualified architecture matrix just because Rust compiles.
    matches!(os, "linux" | "windows") && arch == "x86_64"
}
fn unsupported_runtime_message() -> &'static str {
    if cfg!(target_os = "macos") {
        "Browser setup is unavailable on macOS in this build: owned browser process-tree supervision is incomplete. No browser will be downloaded; platform qualification remains open."
    } else {
        "This build has no supported browser adapter for this operating system or architecture. No browser will be downloaded; platform qualification remains open."
    }
}
impl Product {
    pub fn configured(cache: &Path) -> Result<Self> {
        let distribution = if let Some(p) = std::env::var_os("REGRESSEE_DISTRIBUTION") {
            let p = PathBuf::from(p);
            let m = fs::symlink_metadata(&p)?;
            ensure!(
                m.is_file() && regressee_core::platform::ordinary(&m) && m.len() <= 16384,
                "unsafe distribution configuration"
            );
            let d: Distribution = serde_json::from_reader(File::open(p)?)?;
            d.validate()?;
            Some(d)
        } else {
            None
        };
        Ok(Self {
            distribution,
            runtime_cache: cache.join("product-runtime"),
            auth: Default::default(),
        })
    }
    pub(crate) fn trust(&self) -> Result<&Distribution> {
        self.distribution.as_ref().context("This build is incomplete: browser setup has no trusted distribution configured. Automatic browser setup is not available in this build; no unverified browser will be downloaded. Existing evidence is unchanged.")
    }
    fn setup_trust(&self) -> Result<&Distribution> {
        let d = self.trust()?;
        d.validate().context("Browser trust could not be validated. Setup is unavailable; existing evidence is unchanged.")?;
        ensure!(
            supported_runtime_platform(std::env::consts::OS, std::env::consts::ARCH),
            "{}",
            unsupported_runtime_message()
        );
        Ok(d)
    }
    /// Blocking, read-only status for the background owner. Never downloads.
    pub fn readiness(&self, cancel: &AtomicBool) -> Result<RuntimeReadiness> {
        distribution::check_cancel(cancel)?;
        let Some(d) = &self.distribution else {
            return Ok(RuntimeReadiness {
                status: RuntimeState::UnavailableDistribution,
                setup_available: false,
                message: "This build is incomplete: automatic browser setup has no trusted distribution configured. Existing evidence is unchanged.",
                identity: None,
                scope: None,
            });
        };
        if d.validate().is_err() {
            return Ok(RuntimeReadiness::invalid_configuration());
        }
        if !supported_runtime_platform(std::env::consts::OS, std::env::consts::ARCH) {
            return Ok(RuntimeReadiness {
                status: RuntimeState::UnsupportedPlatform,
                setup_available: false,
                message: unsupported_runtime_message(),
                identity: None,
                scope: Some(d.scope),
            });
        }
        let resolved = match distribution::current_absent(&self.runtime_cache) {
            Ok(true) => {
                distribution::check_cancel(cancel)?;
                return Ok(RuntimeReadiness {
                    status: RuntimeState::NotInstalled,
                    setup_available: true,
                    message: if d.scope == Scope::DevelopmentLoopback {
                        "Browser setup will start on first use. This configured development runtime is restricted to owned loopback fixtures, not public websites."
                    } else {
                        "Browser setup will start on first use using configured public trust. Release and platform qualification remain separate requirements."
                    },
                    identity: None,
                    scope: Some(d.scope),
                });
            }
            Ok(false) => distribution::resolve(&self.runtime_cache, d, cancel),
            Err(e) => Err(e),
        };
        distribution::check_cancel(cancel)?;
        Ok(match resolved {
            Ok(installed) => RuntimeReadiness {
                status: RuntimeState::Ready,
                setup_available: true,
                message: if d.scope == Scope::DevelopmentLoopback {
                    "Private browser verified. Development runtime: owned loopback fixtures only; public-site qualification is pending."
                } else {
                    "Private browser verified using configured public trust. External release and platform qualification remain required."
                },
                identity: Some(installed.manifest.identity),
                scope: Some(d.scope),
            },
            Err(_) => RuntimeReadiness {
                status: RuntimeState::RepairRequired,
                setup_available: true,
                message: "The private browser could not be verified. Automatic replacement is blocked. Review browser diagnostics and explicitly repair; existing evidence is unchanged.",
                identity: None,
                scope: Some(d.scope),
            },
        })
    }
    /// First use only: never turn damaged/expired/unsafe resolver errors into a
    /// download. Callers checking/resuming pinned evidence must not invoke this.
    pub fn ensure_runtime(
        &self,
        cancel: Arc<AtomicBool>,
        progress: &mut dyn FnMut(InstallProgress),
    ) -> Result<Installed> {
        self.ensure_runtime_with(&cancel, || {
            self.setup_with_progress(cancel.clone(), progress)
                .map(|_| ())
        })
    }
    fn ensure_runtime_with(
        &self,
        cancel: &AtomicBool,
        setup: impl FnOnce() -> Result<()>,
    ) -> Result<Installed> {
        distribution::check_cancel(cancel)?;
        let d = self.setup_trust()?;
        let absent = distribution::current_absent(&self.runtime_cache)
            .context("Browser installation paths could not be verified. Automatic setup is blocked; existing evidence is unchanged.")?;
        distribution::check_cancel(cancel)?;
        if absent {
            setup()?;
            distribution::check_cancel(cancel)?;
        }
        let installed = distribution::resolve(&self.runtime_cache, d, cancel)
            .context("The private browser could not be verified. Automatic replacement is blocked; explicit repair is required. Existing evidence is unchanged.")?;
        distribution::check_cancel(cancel)?;
        Ok(installed)
    }
    fn network_path(&self) -> PathBuf {
        self.runtime_cache
            .parent()
            .unwrap_or(&self.runtime_cache)
            .join("network-settings.json")
    }
    pub fn network(&self) -> Result<regressee_runtime::network::NetworkSettings> {
        regressee_runtime::network::NetworkSettings::load(&self.network_path())
    }
    pub fn save_network(
        &self,
        settings: &regressee_runtime::network::NetworkSettings,
    ) -> Result<String> {
        settings.save(&self.network_path())?;
        Ok(settings.diagnostics())
    }
    pub(crate) fn spawn_worker(&self, installed: &Installed) -> Result<Worker> {
        Worker::spawn_network(installed, &self.network()?)
    }
    pub fn status(&self, cancel: &AtomicBool) -> Result<String> {
        let d = self.trust()?;
        let runtime = distribution::resolve(&self.runtime_cache, d, cancel)?;
        Ok(format!(
            "{} · {}",
            runtime.manifest.id,
            if d.scope == Scope::DevelopmentLoopback {
                "Development: owned loopback fixtures only. Public-site runtime qualification pending."
            } else {
                "Configured public trust; external runtime qualification required"
            }
        ))
    }
    pub fn setup(
        &self,
        cancel: Arc<AtomicBool>,
        progress: &mut dyn FnMut(u64, Option<u64>),
    ) -> Result<String> {
        self.setup_with_progress(cancel, &mut |phase| {
            if let InstallProgress::Downloading { bytes, total } = phase {
                progress(bytes, total);
            }
        })
    }
    pub fn setup_with_progress(
        &self,
        cancel: Arc<AtomicBool>,
        progress: &mut dyn FnMut(InstallProgress),
    ) -> Result<String> {
        distribution::check_cancel(&cancel)?;
        let d = self.setup_trust()?;
        let network = self.network().context("Browser network settings could not be validated. Setup is blocked; existing evidence is unchanged.")?;
        distribution::setup_network_with_progress(
            &self.runtime_cache,
            d,
            &cancel,
            progress,
            &mut |runtime| {
                let mut worker = self.spawn_worker(runtime)?;
                worker.handshake(RunId::new(), cancel.clone())?;
                worker.stop(Instant::now() + Duration::from_secs(5))
            },
            &network,
        ).context("Browser setup could not finish. Check the network, free space and browser diagnostics, then retry. Existing evidence is unchanged.")?;
        distribution::check_cancel(&cancel)?;
        self.status(&cancel)
    }
    pub fn capture(
        &self,
        store: &mut Store,
        id: ProjectId,
        check: bool,
        cancel: Arc<AtomicBool>,
        progress: &mut dyn FnMut(&str, u32, Option<u32>),
    ) -> Result<RunRecord> {
        let installed = distribution::resolve(&self.runtime_cache, self.trust()?, &cancel)?;
        regressee_storage::require_space(&store.paths().data, 384 * 1024 * 1024)?;
        regressee_storage::require_space(&store.paths().cache, 384 * 1024 * 1024)?;
        let project = store.load_project(id)?.project;
        ensure!(
            project.sources.len() == 1
                && project.profiles.len() == 1
                && project.environments.len() == 1
                && project.environments[0].session.is_none()
                && project.environments[0].access == AccessPolicy::Public,
            "This legacy single-page API supports public access only. Use guided inventory capture for login or environment mapping."
        );
        ensure!(
            store
                .session_reference(id, project.environments[0].id)?
                .is_none()
                && project.environments[0]
                    .mappings
                    .iter()
                    .all(|m| m.source == m.destination),
            "A saved login or changed mapping requires guided inventory capture; anonymous fallback is not allowed"
        );
        let (inventory, profile, intent, rendering) = if check {
            let baseline = store.load_baseline(
                project
                    .active_baseline
                    .context("Save a successful baseline before checking for changes")?,
            )?;
            let before = store.load_run(baseline.references[0].run)?;
            ensure!(
                before.run.profiles[0].runtime == installed.manifest.identity,
                "The installed browser differs from this baseline. Repair the matching runtime or capture a new baseline; existing evidence is preserved."
            );
            (
                before.run.inventory,
                before.run.profiles[0].clone(),
                RunIntent::Check {
                    baseline: baseline.id,
                },
                rendering_environment(&installed)?,
            )
        } else {
            let mut profile = project.profiles[0].clone();
            configure_profile(&mut profile, &installed)?;
            let inventory = InventoryRevision {
                id: InventoryId::new(),
                project: id,
                identity_version: 1,
                pages: vec![InventoryPage {
                    id: PageId::new(),
                    canonical: project.sources[0].clone(),
                    included: true,
                    reason: None,
                }],
                coverage: DiscoveryCoverage::Complete,
            };
            let rendering = rendering_environment(&installed)?;
            (inventory, profile, RunIntent::NewBaseline, rendering)
        };
        ensure!(
            rendering.os == std::env::consts::OS
                && rendering.architecture == std::env::consts::ARCH,
            "Cross-computer rendering is not comparable in this preview"
        );
        let address = inventory.pages[0].canonical.clone();
        ensure!(
            installed.manifest.scope != Scope::DevelopmentLoopback
                || distribution::loopback(&address),
            "This development runtime is restricted to owned literal-loopback fixtures. Public-site browser security qualification is pending."
        );
        let key = TargetKey {
            page: inventory.pages[0].id,
            profile: profile.id,
        };
        let mut record = RunRecord {
            run: CaptureRun {
                id: RunId::new(),
                project: id,
                intent,
                inventory,
                profiles: vec![profile.clone()],
                environment: project.environments[0].clone(),
                rendering,
                state: JobState::Running,
                started_at: Some(distribution::now()),
                finished_at: None,
                targets: vec![CaptureTarget {
                    key: key.clone(),
                    requested: address.clone(),
                    final_url: None,
                    attempts: 1,
                    outcome: TargetOutcome::Pending,
                    warnings: vec![],
                }],
            },
            capture: None,
            captures: vec![],
            queue: None,
        };
        store.begin_run(&record)?;
        progress("Starting the private browser…", 0, Some(1));
        let captured = (|| -> Result<CaptureEvidence> {
            let mut worker = self.spawn_worker(&installed)?;
            let (version, browser) = worker.handshake(record.run.id, cancel.clone())?;
            ensure!(
                version == record.run.rendering.worker_version
                    && browser == record.run.rendering.browser,
                "worker runtime identity mismatch"
            );
            let mut result = None;
            worker.request(
                Request {
                    version: 2,
                    request_id: 2,
                    job: record.run.id,
                    command: Command::CaptureTarget {
                        key: key.clone(),
                        address: address.to_string(),
                        profile: Box::new(profile.clone()),
                        session: None,
                    },
                },
                Instant::now()
                    + Duration::from_millis(
                        u64::from(profile.stabilization.navigation_ms)
                            + u64::from(profile.stabilization.readiness_ms)
                            + 10000,
                    ),
                cancel.clone(),
                &mut |e| {
                    match e.event {
                        EventKind::Progress { completed, total } => {
                            progress("Capturing the fixed page…", completed, total)
                        }
                        EventKind::TargetCaptured {
                            key,
                            output_token,
                            artifact,
                            width,
                            height,
                        } => {
                            ensure!(result.is_none(), "duplicate captured artifact");
                            result = Some((key, output_token, artifact, width, height));
                        }
                        EventKind::Warning { code } => {
                            record.run.targets[0].warnings.push(format!("{code:?}"))
                        }
                        _ => (),
                    }
                    Ok(())
                },
            )?;
            let (received, token, artifact, width, height) =
                result.context("worker completed without screenshot evidence")?;
            ensure!(
                received == key && artifact.kind == ArtifactKind::Screenshot,
                "foreign/unsafe worker artifact"
            );
            crate::images::budget(width, height)?;
            let path = worker.output_path(token);
            let mp = worker.metadata_path(token);
            let meta = fs::symlink_metadata(&mp)?;
            ensure!(
                meta.is_file()
                    && regressee_core::platform::ordinary(&meta)
                    && meta.len() <= 256 * 1024,
                "unsafe worker metadata"
            );
            let metadata: CaptureMetadata = serde_json::from_reader(File::open(mp)?)?;
            ensure!(
                metadata.version == 1
                    && metadata.key == key
                    && metadata.profile == profile
                    && metadata.runtime == installed.manifest.identity
                    && metadata.requested == address.as_str()
                    && metadata.final_url == address.as_str()
                    && metadata.sha256 == artifact.id
                    && metadata.bytes == artifact.bytes
                    && metadata.width == width
                    && metadata.height == height
                    && metadata.capture_mode == "chromium-new-headless"
                    && metadata.warnings == record.run.targets[0].warnings
                    && metadata.duration_ms <= 300000,
                "worker metadata mismatch"
            );
            let image_meta = fs::symlink_metadata(&path)?;
            ensure!(
                image_meta.is_file()
                    && regressee_core::platform::ordinary(&image_meta)
                    && image_meta.len() == artifact.bytes,
                "unsafe worker output"
            );
            ensure!(
                distribution::hash(File::open(&path)?, &cancel)? == artifact.id.as_str(),
                "worker screenshot hash mismatch"
            );
            progress("Saving originals and viewer tiles…", 1, Some(1));
            let image = crate::images::ingest(store, &path, ArtifactKind::Screenshot, &cancel)?;
            ensure!(
                image.original == artifact.id && image.width == width && image.height == height,
                "decoded screenshot dimensions/hash disagree"
            );
            // Shutdown failure is diagnostic; valid completed evidence is retained.
            if worker
                .stop(Instant::now() + Duration::from_secs(5))
                .is_err()
            {
                record.run.targets[0]
                    .warnings
                    .push("WorkerShutdownRequiredCleanup".into());
            }
            Ok(CaptureEvidence { metadata, image })
        })();
        match captured {
            Ok(c) => {
                record.run.targets[0].final_url = Some(url::Url::parse(&c.metadata.final_url)?);
                record.run.targets[0].outcome = TargetOutcome::Captured {
                    artifact: c.image.original.clone(),
                    width: c.image.width,
                    height: c.image.height,
                };
                record.capture = Some(c);
                record.run.state = if cancel.load(Ordering::Acquire) {
                    JobState::Cancelled
                } else {
                    JobState::Completed
                };
            }
            Err(e) => {
                let code = map_failure(&e);
                if cancel.load(Ordering::Acquire) || code == CaptureError::Cancelled {
                    record.run.state = JobState::Cancelled;
                    record.run.targets[0].outcome = TargetOutcome::Cancelled;
                } else {
                    record.run.state = JobState::CompletedWithErrors;
                    record.run.targets[0].outcome = TargetOutcome::Failed { code };
                }
            }
        }
        record.run.finished_at = Some(distribution::now());
        store.finish_run(&record)?;
        if let RunIntent::Check { .. } = record.run.intent {
            progress("Analyzing captured pixels…", 0, None);
            if !cancel.load(Ordering::Acquire) {
                crate::review::analyze_run(
                    store,
                    record.run.id,
                    crate::review::preset(crate::analysis_engine::Preset::Balanced),
                    &cancel,
                )?;
            }
        }
        Ok(record)
    }
    pub fn activate(&self, store: &mut Store, run: RunId, revision: u64) -> Result<ProjectRecord> {
        store.activate_baseline(run, revision)
    }
}
pub(crate) fn browser_version(i: &Installed) -> Result<String> {
    if let RuntimeIdentity::Pinned { browser, .. } = &i.manifest.identity {
        Ok(browser.clone())
    } else {
        anyhow::bail!("unconfigured runtime")
    }
}
pub(crate) fn worker_version(i: &Installed) -> Result<String> {
    if let RuntimeIdentity::Pinned { worker, .. } = &i.manifest.identity {
        Ok(worker.clone())
    } else {
        anyhow::bail!("unconfigured runtime")
    }
}
pub(crate) fn configure_profile(p: &mut CaptureProfile, i: &Installed) -> Result<()> {
    ensure!(
        p.width <= 4096
            && p.dpr == 1.
            && (p.image_mode == ImageMode::Normal
                || p.image_mode
                    == ImageMode::ReplaceRaster {
                        algorithm: "neutral-raster-v2".into()
                    }),
        "Unsupported capture profile: use DPR 1, normal images or neutral-raster-v2 and width up to 4096"
    );
    p.runtime = i.manifest.identity.clone();
    p.user_agent = format!(
        "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Safari/537.36",
        browser_version(i)?
    );
    if p.mobile {
        p.user_agent = format!(
            "Mozilla/5.0 (Linux; Android 14; Regressee Mobile) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/{} Mobile Safari/537.36",
            browser_version(i)?
        );
    }
    p.stabilization.max_height = p.stabilization.max_height.min(32768);
    p.validate()
}
pub(crate) fn rendering_environment(i: &Installed) -> Result<RenderingEnvironment> {
    use sha2::{Digest, Sha256};
    use std::io::Read;
    let mut paths = vec![
        PathBuf::from("/usr/share/fonts"),
        PathBuf::from("/usr/local/share/fonts"),
    ];
    if let Some(home) = std::env::var_os("HOME") {
        paths.push(PathBuf::from(home).join(".local/share/fonts"));
    }
    let mut files = Vec::new();
    let mut directories = 0;
    while let Some(path) = paths.pop() {
        directories += 1;
        if directories > 256 {
            break;
        }
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries.flatten().take(1024) {
                let m = entry.metadata()?;
                if entry.file_type()?.is_symlink() {
                    continue;
                }
                if m.is_dir() {
                    paths.push(entry.path());
                } else if m.is_file()
                    && entry
                        .path()
                        .extension()
                        .is_some_and(|s| matches!(s.to_str(), Some("ttf" | "otf" | "ttc")))
                {
                    files.push(entry.path());
                }
            }
        }
    }
    files.sort();
    let mut hash = Sha256::new();
    let mut total = 0;
    let mut count = 0;
    for p in files.into_iter().take(256) {
        let size = fs::metadata(&p)?.len();
        if size > 16 * 1024 * 1024 || total + size > 64 * 1024 * 1024 {
            break;
        }
        total += size;
        count += 1;
        hash.update(p.to_string_lossy().as_bytes());
        let mut f = File::open(p)?;
        let mut buf = [0; 65536];
        loop {
            let n = f.read(&mut buf)?;
            if n == 0 {
                break;
            }
            hash.update(&buf[..n]);
        }
    }
    Ok(RenderingEnvironment {
        os: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        browser: browser_version(i)?,
        worker_version: worker_version(i)?,
        profile_version: 1,
        fonts_fingerprint: if count > 0 {
            Some(format!(
                "bounded-system-fonts-v1:{}",
                hex::encode(hash.finalize())
            ))
        } else {
            None
        },
    })
}
pub(crate) fn map_failure(e: &anyhow::Error) -> CaptureError {
    if e.chain().any(|cause| {
        cause
            .downcast_ref::<std::io::Error>()
            .is_some_and(|io| io.kind() == std::io::ErrorKind::StorageFull)
    }) {
        return CaptureError::LowSpace;
    }
    match e.downcast_ref::<WorkerFailure>().map(|e| e.0) {
        Some(FailureCode::NeedsLogin) => CaptureError::NeedsLogin,
        Some(FailureCode::Timeout) => CaptureError::Timeout,
        Some(FailureCode::WorkerStopped) => CaptureError::WorkerStopped,
        Some(FailureCode::Cancelled) => CaptureError::Cancelled,
        Some(FailureCode::AccessDenied) => CaptureError::AccessDenied,
        Some(FailureCode::NotFound) => CaptureError::NotFound,
        Some(FailureCode::RedirectOutsideScope) => CaptureError::RedirectOutsideScope,
        Some(FailureCode::ImageLimit) => CaptureError::ImageLimit,
        Some(FailureCode::RateLimited) => CaptureError::RateLimited,
        Some(FailureCode::ServerError) => CaptureError::ServerError,
        _ => CaptureError::CaptureFailed,
    }
}

#[cfg(test)]
mod automatic_runtime_tests {
    use super::*;

    #[test]
    fn platform_eligibility_does_not_enable_macos_or_extra_architectures() {
        assert!(supported_runtime_platform("linux", "x86_64"));
        assert!(supported_runtime_platform("windows", "x86_64"));
        for (os, arch) in [
            ("macos", "aarch64"),
            ("macos", "x86_64"),
            ("linux", "aarch64"),
            ("windows", "aarch64"),
            ("unknown", "x86_64"),
        ] {
            assert!(!supported_runtime_platform(os, arch));
        }
    }

    #[test]
    fn missing_trust_is_build_incompleteness_and_status_never_installs() -> Result<()> {
        let root = tempfile::tempdir()?;
        let product = Product {
            distribution: None,
            runtime_cache: root.path().join("cache"),
            auth: Default::default(),
        };
        let cancel = AtomicBool::new(false);
        let status = product.readiness(&cancel)?;
        assert_eq!(status.status, RuntimeState::UnavailableDistribution);
        assert!(!status.setup_available);
        assert!(status.message.contains("build is incomplete"));
        assert!(
            product
                .ensure_runtime_with(&cancel, || panic!("no trust"))
                .unwrap_err()
                .to_string()
                .contains("build is incomplete")
        );
        assert!(!product.runtime_cache.exists());
        Ok(())
    }

    #[test]
    fn invalid_trust_is_unavailable_without_exposing_its_url() -> Result<()> {
        let root = tempfile::tempdir()?;
        let product = Product {
            distribution: Some(Distribution {
                version: 1,
                url: "https://example.invalid/private-endpoint".parse()?,
                public_key: "invalid".into(),
                scope: Scope::QualifiedPublic,
            }),
            runtime_cache: root.path().join("cache"),
            auth: Default::default(),
        };
        let cancel = AtomicBool::new(false);
        let status = product.readiness(&cancel)?;
        assert_eq!(status.status, RuntimeState::UnavailableDistribution);
        assert!(!status.setup_available);
        assert!(!status.message.contains("private-endpoint"));
        assert!(
            product
                .ensure_runtime_with(&cancel, || panic!("invalid trust"))
                .is_err()
        );
        assert!(!product.runtime_cache.exists());
        Ok(())
    }

    #[cfg(all(target_arch = "x86_64", any(target_os = "linux", windows)))]
    mod supported {
        use super::*;
        use distribution::{Entry, Manifest};
        use ed25519_dalek::{Signer, SigningKey};
        use std::{collections::BTreeMap, io::Write};
        use zip::{ZipWriter, write::SimpleFileOptions};

        // Signed INERT inventory: the callback models smoke acceptance, never
        // launches Node/browser and does not qualify a real runtime or website.
        fn fixture(root: &Path, scope: Scope) -> Result<(Product, PathBuf, SigningKey)> {
            let key = SigningKey::from_bytes(&[83; 32]); // Test-only, not app trust.
            let d = Distribution {
                version: 1,
                url: if scope == Scope::DevelopmentLoopback {
                    "http://127.0.0.1:9/inert.zip"
                } else {
                    "https://example.invalid/inert.zip"
                }
                .parse()?,
                public_key: hex::encode(key.verifying_key().to_bytes()),
                scope,
            };
            let payload = b"inert: never execute";
            let entry = Entry {
                sha256: distribution::hash(&payload[..], &AtomicBool::new(false))?,
                bytes: payload.len() as u64,
                executable: false,
            };
            let manifest = Manifest {
                version: 2,
                id: "inert-first-use-test".into(),
                os: std::env::consts::OS.into(),
                arch: std::env::consts::ARCH.into(),
                scope,
                support_until: if scope == Scope::QualifiedPublic {
                    Some(distribution::now() + 3600)
                } else {
                    None
                },
                identity: RuntimeIdentity::Pinned {
                    node: "inert".into(),
                    worker: "inert".into(),
                    playwright: "inert".into(),
                    browser: "inert".into(),
                },
                node: "node/file".into(),
                worker: "worker/file".into(),
                browser_root: "browsers".into(),
                files: BTreeMap::from([
                    ("node/file".into(), entry.clone()),
                    ("worker/file".into(), entry.clone()),
                    ("browsers/file".into(), entry),
                ]),
                links: BTreeMap::new(),
                modes: BTreeMap::new(),
            };
            let bytes = serde_json::to_vec(&manifest)?;
            let pack = root.join("inert.zip");
            let mut zip = ZipWriter::new(File::create(&pack)?);
            let options = SimpleFileOptions::default();
            zip.start_file("manifest.json", options)?;
            zip.write_all(&bytes)?;
            zip.start_file("manifest.sig", options)?;
            zip.write_all(&key.sign(&bytes).to_bytes())?;
            for name in manifest.files.keys() {
                zip.start_file(name, options)?;
                zip.write_all(payload)?;
            }
            zip.finish()?;
            Ok((
                Product {
                    distribution: Some(d),
                    runtime_cache: root.join("cache"),
                    auth: Default::default(),
                },
                pack,
                key,
            ))
        }

        fn install_inert(product: &Product, pack: &Path, cancel: &AtomicBool) -> Result<()> {
            distribution::install(
                pack,
                &product.runtime_cache,
                product.trust()?,
                cancel,
                &mut |_| Ok(()),
            )?;
            Ok(())
        }

        #[test]
        fn first_use_installs_once_existing_generation_is_verified_not_redownloaded() -> Result<()>
        {
            let root = tempfile::tempdir()?;
            let (product, pack, _) = fixture(root.path(), Scope::DevelopmentLoopback)?;
            let cancel = AtomicBool::new(false);
            let state = product.readiness(&cancel)?;
            assert_eq!(state.status, RuntimeState::NotInstalled);
            assert!(state.setup_available);
            assert!(!product.runtime_cache.exists(), "status wrote cache");
            let mut installs = 0;
            let first = product.ensure_runtime_with(&cancel, || {
                installs += 1;
                install_inert(&product, &pack, &cancel)
            })?;
            assert_eq!(installs, 1);
            let pointer = fs::read(product.runtime_cache.join("current"))?;
            let second = product.ensure_runtime_with(&cancel, || panic!("redownload"))?;
            assert_eq!(first.root, second.root);
            assert_eq!(fs::read(product.runtime_cache.join("current"))?, pointer);
            assert_eq!(product.readiness(&cancel)?.status, RuntimeState::Ready);
            fs::write(first.root.join("worker/file"), b"tampered")?;
            let state = product.readiness(&cancel)?;
            assert_eq!(state.status, RuntimeState::RepairRequired);
            assert!(state.message.contains("Automatic replacement is blocked"));
            let error = product
                .ensure_runtime_with(&cancel, || panic!("auto repair"))
                .unwrap_err();
            assert!(format!("{error:#}").contains("runtime damaged"));
            assert_eq!(fs::read(product.runtime_cache.join("current"))?, pointer);
            Ok(())
        }

        #[test]
        fn present_invalid_pointer_and_expired_manifest_never_install() -> Result<()> {
            let root = tempfile::tempdir()?;
            let (product, pack, key) = fixture(root.path(), Scope::QualifiedPublic)?;
            let cancel = AtomicBool::new(false);
            fs::create_dir(&product.runtime_cache)?;
            fs::write(product.runtime_cache.join("current"), b"../unsafe")?;
            assert!(
                product
                    .ensure_runtime_with(&cancel, || panic!("unsafe pointer"))
                    .is_err()
            );
            fs::write(product.runtime_cache.join("current"), b"runtime-missing")?;
            assert!(
                product
                    .ensure_runtime_with(&cancel, || panic!("missing generation"))
                    .is_err()
            );
            // Explicit install is allowed to repair a regular damaged pointer.
            install_inert(&product, &pack, &cancel)?;
            let installed =
                distribution::resolve(&product.runtime_cache, product.trust()?, &cancel)?;
            let mut manifest = installed.manifest;
            manifest.support_until = Some(distribution::now() - 1);
            let bytes = serde_json::to_vec(&manifest)?;
            fs::write(installed.root.join("manifest.json"), &bytes)?;
            fs::write(
                installed.root.join("manifest.sig"),
                key.sign(&bytes).to_bytes(),
            )?;
            let error = product
                .ensure_runtime_with(&cancel, || panic!("expired auto repair"))
                .unwrap_err();
            assert!(format!("{error:#}").contains("support expired"));
            assert_eq!(
                product.readiness(&cancel)?.status,
                RuntimeState::RepairRequired
            );
            Ok(())
        }

        #[test]
        fn cancel_before_during_and_after_setup_prevents_continuation() -> Result<()> {
            for phase in [
                None,
                Some(InstallProgress::TestingBrowser),
                Some(InstallProgress::Ready),
            ] {
                let root = tempfile::tempdir()?;
                let (product, pack, _) = fixture(root.path(), Scope::DevelopmentLoopback)?;
                let cancel = AtomicBool::new(phase.is_none());
                let mut continued = false;
                let mut smoked = false;
                let result = product
                    .ensure_runtime_with(&cancel, || {
                        distribution::install_with_progress(
                            &pack,
                            &product.runtime_cache,
                            product.trust()?,
                            &cancel,
                            &mut |event| {
                                if Some(event) == phase {
                                    cancel.store(true, Ordering::Release);
                                }
                            },
                            &mut |_| {
                                smoked = true;
                                Ok(())
                            },
                        )?;
                        Ok(())
                    })
                    .map(|_| continued = true);
                assert!(result.is_err());
                assert!(!continued, "cancelled setup continued website action");
                assert_eq!(smoked, phase == Some(InstallProgress::Ready));
                assert_eq!(
                    product.runtime_cache.join("current").exists(),
                    phase == Some(InstallProgress::Ready)
                );
            }
            // An installer may return success as cancellation arrives; the
            // orchestration itself must check, independent of installer checks.
            let root = tempfile::tempdir()?;
            let (product, pack, _) = fixture(root.path(), Scope::DevelopmentLoopback)?;
            let cancel = AtomicBool::new(false);
            assert!(
                product
                    .ensure_runtime_with(&cancel, || {
                        install_inert(&product, &pack, &cancel)?;
                        cancel.store(true, Ordering::Release);
                        Ok(())
                    })
                    .is_err()
            );
            Ok(())
        }

        #[cfg(unix)]
        #[test]
        fn dangling_pointer_and_linked_cache_are_not_first_use() -> Result<()> {
            let root = tempfile::tempdir()?;
            let (product, _, _) = fixture(root.path(), Scope::DevelopmentLoopback)?;
            let cancel = AtomicBool::new(false);
            fs::create_dir(&product.runtime_cache)?;
            std::os::unix::fs::symlink("missing", product.runtime_cache.join("current"))?;
            assert!(
                product
                    .ensure_runtime_with(&cancel, || panic!("dangling pointer"))
                    .is_err()
            );
            let linked = Product {
                runtime_cache: root.path().join("linked"),
                ..product
            };
            std::os::unix::fs::symlink(
                linked.runtime_cache.with_file_name("cache"),
                &linked.runtime_cache,
            )?;
            assert!(
                linked
                    .ensure_runtime_with(&cancel, || panic!("linked cache"))
                    .is_err()
            );
            assert_eq!(
                linked.readiness(&cancel)?.status,
                RuntimeState::RepairRequired
            );
            Ok(())
        }
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn macos_status_and_first_use_refuse_without_downloading() -> Result<()> {
        let root = tempfile::tempdir()?;
        // Public KEY only; this inert test configuration supplies no runtime.
        let key = ed25519_dalek::SigningKey::from_bytes(&[84; 32]);
        let product = Product {
            distribution: Some(Distribution {
                version: 1,
                url: "http://127.0.0.1:9/inert.zip".parse()?,
                public_key: hex::encode(key.verifying_key().to_bytes()),
                scope: Scope::DevelopmentLoopback,
            }),
            runtime_cache: root.path().join("cache"),
            auth: Default::default(),
        };
        let cancel = AtomicBool::new(false);
        let status = product.readiness(&cancel)?;
        assert_eq!(status.status, RuntimeState::UnsupportedPlatform);
        assert!(!status.setup_available);
        assert!(
            product
                .ensure_runtime_with(&cancel, || panic!("Mac install"))
                .is_err()
        );
        assert!(!product.runtime_cache.exists());
        Ok(())
    }
}
