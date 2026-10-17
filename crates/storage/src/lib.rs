// SPDX-License-Identifier: MPL-2.0
//! Blocking storage API. Call only from the storage actor or a headless tool.
use anyhow::{Context, Result, bail, ensure};
use directories::ProjectDirs;
use regressee_core::domain::*;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
    time::Duration,
};
mod capture;
mod history_inspection;
pub use capture::{CheckpointWork, RunAppender, TargetCheckpoint};
mod diagnostics;
pub use diagnostics::DiagnosticCounts;
pub mod images;
#[cfg(test)]
mod phase6_corrections_tests;
mod portability;
mod preferences;
pub use preferences::AppPreferences;
mod retention;
mod review;
pub use portability::require_space;
pub use portability::{ExportPreview, ImportPreview, ValidatedArchive};
pub use retention::{DeleteItem, DeletionPreview, HistoryPage, ProjectStorage, RepairPreview};
pub fn portability_warning() -> &'static str {
    portability::COMPATIBILITY
}
pub use review::{ApprovalContext, ApprovalPreview};

pub const DB_VERSION: i64 = 9;
pub const MAX_ARTIFACT_BYTES: u64 = 128 * 1024 * 1024;
pub const PROJECT_PAGE_SIZE: usize = 64;
#[cfg(test)]
mod phase5_migration_tests {
    use super::*;
    #[test]
    fn version_eight_target_delta_migration_preserves_historical_bytes() {
        let mut connection = Connection::open_in_memory().unwrap();
        for migration in MIGRATIONS.iter().take(8) {
            connection.execute_batch(migration).unwrap();
        }
        connection.pragma_update(None, "user_version", 8).unwrap();
        connection
            .execute("INSERT INTO projects VALUES('p','fixture','{}',1,1)", [])
            .unwrap();
        let original = "{ \"historical\": \"exact snapshot bytes\" }";
        connection
            .execute(
                "INSERT INTO runs VALUES('r','p','\"Completed\"',?1,1,1)",
                [original],
            )
            .unwrap();
        migrate(&mut connection).unwrap();
        migrate(&mut connection).unwrap();
        assert_eq!(
            connection
                .query_row("SELECT record FROM runs WHERE id='r'", [], |row| row
                    .get::<_, String>(0))
                .unwrap(),
            original
        );
        assert_eq!(
            connection
                .query_row("SELECT count(*) FROM run_target_deltas", [], |row| row
                    .get::<_, u64>(0))
                .unwrap(),
            0
        );
    }
    #[test]
    fn version_six_display_context_migration_preserves_records() {
        let mut c = Connection::open_in_memory().unwrap();
        for sql in MIGRATIONS.iter().take(6) {
            c.execute_batch(sql).unwrap();
        }
        c.pragma_update(None, "user_version", 6).unwrap();
        c.execute("INSERT INTO projects(id,name,config,revision,updated_at) VALUES('p','fixture','{}',1,1)", []).unwrap();
        c.execute(
            "INSERT INTO runs VALUES('r','p','\"Completed\"','{}',1,1)",
            [],
        )
        .unwrap();
        c.execute("INSERT INTO baselines VALUES('b','p','r','{}')", [])
            .unwrap();
        let record = "{ \"immutable\": \"analysis bytes\" }";
        c.execute("INSERT INTO analyses VALUES('a','r','b',?1)", [record])
            .unwrap();
        c.execute("INSERT INTO analysis_display VALUES('r','a')", [])
            .unwrap();
        migrate(&mut c).unwrap();
        migrate(&mut c).unwrap();
        assert_eq!(
            c.query_row("SELECT record FROM analyses WHERE id='a'", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            record
        );
        assert_eq!(
            c.query_row(
                "SELECT analysis_id,revision FROM analysis_display WHERE run_id='r'",
                [],
                |r| Ok((r.get::<_, String>(0)?, r.get::<_, u64>(1)?))
            )
            .unwrap(),
            ("a".into(), 1)
        );
    }
    #[test]
    fn version_five_evidence_bytes_survive_multiple_analysis_revisions() {
        let mut c = Connection::open_in_memory().unwrap();
        for sql in MIGRATIONS.iter().take(5) {
            c.execute_batch(sql).unwrap();
        }
        c.pragma_update(None, "user_version", 5).unwrap();
        c.execute("INSERT INTO projects(id,name,config,revision,updated_at) VALUES('p','fixture','{}',1,1)",[]).unwrap();
        c.execute(
            "INSERT INTO runs VALUES('r','p','\"Completed\"','{}',1,1)",
            [],
        )
        .unwrap();
        c.execute("INSERT INTO baselines VALUES('b','p','r','{}')", [])
            .unwrap();
        let old = "{ \"historical\": \"rgba-max-channel-v1\", \"preserve_exact_bytes\": true }";
        c.execute("INSERT INTO analyses VALUES('a','r','b',?1)", [old])
            .unwrap();
        migrate(&mut c).unwrap();
        migrate(&mut c).unwrap();
        assert_eq!(
            c.query_row("SELECT record FROM analyses WHERE id='a'", [], |r| r
                .get::<_, String>(0))
                .unwrap(),
            old
        );
        c.execute("INSERT INTO analyses VALUES('a2','r','b','{}')", [])
            .unwrap();
        assert_eq!(
            c.query_row("SELECT count(*) FROM analyses WHERE run_id='r'", [], |r| {
                r.get::<_, u64>(0)
            })
            .unwrap(),
            2
        );
        assert_eq!(
            c.pragma_query_value(None, "user_version", |r| r.get::<_, u64>(0))
                .unwrap(),
            DB_VERSION as u64
        );
    }
}
const MIGRATIONS: [&str; 9] = [
    include_str!("../migrations/001-projects.sql"),
    include_str!("../migrations/002-artifacts.sql"),
    include_str!("../migrations/003-capture.sql"),
    include_str!("../migrations/004-crawling.sql"),
    include_str!("../migrations/005-local-auth.sql"),
    include_str!("../migrations/006-review.sql"),
    include_str!("../migrations/007-approval-context.sql"),
    include_str!("../migrations/008-portability.sql"),
    include_str!("../migrations/009-target-checkpoints.sql"),
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppPaths {
    pub data: PathBuf,
    pub cache: PathBuf,
}
impl AppPaths {
    pub fn discover() -> Result<Self> {
        let dirs = ProjectDirs::from("org", "Regressee", "Regressee")
            .context("OS app-data directory is unavailable")?;
        Ok(Self {
            data: dirs.data_local_dir().to_owned(),
            cache: dirs.cache_dir().to_owned(),
        })
    }
    /// Explicit isolated developer/test location; never inferred from cwd.
    pub fn isolated(root: &Path) -> Self {
        Self {
            data: root.join("data"),
            cache: root.join("cache"),
        }
    }
    pub fn database(&self) -> PathBuf {
        self.data.join("regressee.sqlite3")
    }
    pub fn artifacts(&self) -> PathBuf {
        self.data.join("artifacts")
    }
    pub fn prepare(&self) -> Result<()> {
        // Preflight every owned directory before create_dir_all can follow an
        // existing ancestor. This guards static unsafe layouts, not same-user races.
        for path in [&self.data, &self.cache, &self.artifacts()] {
            validate_directory_path(path)?;
        }
        for path in [&self.data, &self.cache, &self.artifacts()] {
            private_dir(path)?;
        }
        Ok(())
    }
}
fn private_dir(path: &Path) -> Result<()> {
    validate_directory_path(path)?;
    fs::create_dir_all(path)?;
    ensure!(
        fs::symlink_metadata(path)?.is_dir(),
        "storage directory is not a real directory"
    );
    regressee_core::platform::protect(path)?;
    Ok(())
}
pub(crate) fn create_private(path: &Path) -> Result<fs::File> {
    let mut options = fs::File::options();
    options.create_new(true).write(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    Ok(options.open(path)?)
}
fn entry_metadata(path: &Path) -> Result<Option<fs::Metadata>> {
    match fs::symlink_metadata(path) {
        Ok(metadata) => Ok(Some(metadata)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => {
            Err(error).with_context(|| format!("inspect storage path {}", path.display()))
        }
    }
}
fn validate_directory_path(path: &Path) -> Result<()> {
    // Explicit developer overrides may be relative (the OS-discovered defaults
    // never are). Anchor inspection without canonicalizing away unsafe entries.
    let absolute;
    let path = if path.is_absolute() {
        path
    } else {
        absolute = std::env::current_dir()?.join(path);
        &absolute
    };
    ensure!(
        !path
            .components()
            .any(|part| matches!(part, std::path::Component::ParentDir)),
        "storage path must not contain parent traversal"
    );
    // Inspect top-down: never look through a known symlink (including dangling
    // ones) to inspect or create a child. Only genuine NotFound means absent.
    for ancestor in path.ancestors().collect::<Vec<_>>().into_iter().rev() {
        if let Some(metadata) = entry_metadata(ancestor)? {
            ensure!(
                metadata.is_dir() && regressee_core::platform::ordinary(&metadata),
                "storage directory is not a real directory: {}",
                ancestor.display()
            );
        }
    }
    Ok(())
}
fn regular_file_or_absent(path: &Path) -> Result<bool> {
    validate_directory_path(path.parent().context("storage file has no parent")?)?;
    match entry_metadata(path)? {
        Some(metadata) => {
            ensure!(
                metadata.is_file() && regressee_core::platform::ordinary(&metadata),
                "storage file is not a regular file: {}",
                path.display()
            );
            Ok(true)
        }
        None => Ok(false),
    }
}
pub fn migrate(connection: &mut Connection) -> Result<()> {
    // A writer lock spans version inspection and all changes; concurrent first
    // launches cannot each apply the same migration. DDL + user_version roll back.
    let transaction = connection.transaction_with_behavior(TransactionBehavior::Immediate)?;
    let version: i64 = transaction.pragma_query_value(None, "user_version", |row| row.get(0))?;
    ensure!(
        (0..=DB_VERSION).contains(&version),
        "database was created by a newer app; do not modify it"
    );
    for next in version + 1..=DB_VERSION {
        transaction.execute_batch(MIGRATIONS[(next - 1) as usize])?;
        transaction.pragma_update(None, "user_version", next)?;
    }
    transaction.commit()?;
    Ok(())
}
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalLoginSettings {
    pub login_address: Option<url::Url>,
    pub representative: Option<url::Url>,
    pub identity_origins: Vec<url::Url>,
    pub ephemeral: bool,
}
impl LocalLoginSettings {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.identity_origins.len() <= 16,
            "Approve at most 16 identity-provider origins"
        );
        for u in self
            .login_address
            .iter()
            .chain(self.representative.iter())
            .chain(self.identity_origins.iter())
        {
            validate_address(u)?;
        }
        for u in &self.identity_origins {
            ensure!(
                u.path() == "/" && u.query().is_none() && u.fragment().is_none(),
                "Identity providers must be approved as exact origins"
            );
        }
        Ok(())
    }
}
pub struct Store {
    connection: Connection,
    paths: AppPaths,
    instance: uuid::Uuid,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectRecord {
    pub project: Project,
    pub revision: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectSummary {
    pub id: ProjectId,
    pub name: String,
    pub address: String,
    pub revision: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectPage {
    pub items: Vec<ProjectSummary>,
    pub page: usize,
    pub more: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSummary {
    pub artifact_bytes: u64,
    pub artifacts: u64,
    pub projects: u64,
    pub paths: AppPaths,
}
impl Store {
    pub fn selected_environment(&self, project: &Project) -> Result<Environment> {
        let id: Option<String> = self
            .connection
            .query_row(
                "SELECT environment FROM local_environment_settings WHERE project=?1",
                [project.id.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        let env = if let Some(id) = id {
            project
                .environments
                .iter()
                .find(|e| e.id.to_string() == id)
                .context("Selected environment was removed; select an environment again")?
        } else {
            &project.environments[0]
        };
        Ok(env.clone())
    }
    pub fn local_login_settings(&self, project: ProjectId) -> Result<LocalLoginSettings> {
        let json: Option<String> = self
            .connection
            .query_row(
                "SELECT settings FROM local_environment_settings WHERE project=?1",
                [project.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        Ok(json
            .map(|j| serde_json::from_str(&j))
            .transpose()?
            .unwrap_or_default())
    }
    pub fn select_environment(
        &mut self,
        project: ProjectId,
        environment: EnvironmentId,
        settings: &LocalLoginSettings,
    ) -> Result<()> {
        let p = self.load_project(project)?.project;
        ensure!(
            p.environments.iter().any(|e| e.id == environment),
            "Unknown environment"
        );
        settings.validate()?;
        self.connection.execute("INSERT INTO local_environment_settings VALUES(?1,?2,?3) ON CONFLICT(project) DO UPDATE SET environment=excluded.environment,settings=excluded.settings",params![project.to_string(),environment.to_string(),serde_json::to_string(settings)?])?;
        Ok(())
    }
    pub fn session_reference(
        &self,
        project: ProjectId,
        environment: EnvironmentId,
    ) -> Result<Option<(SessionReference, bool)>> {
        let row: Option<(String, bool)> = self
            .connection
            .query_row(
                "SELECT session,ephemeral FROM local_sessions WHERE project=?1 AND environment=?2",
                params![project.to_string(), environment.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        row.map(|(id, ephemeral)| {
            let id = SessionId(uuid::Uuid::parse_str(&id)?);
            ensure!(!id.0.is_nil(), "Invalid saved login reference");
            Ok((
                SessionReference {
                    id,
                    project,
                    environment,
                },
                ephemeral,
            ))
        })
        .transpose()
    }
    pub fn save_session_reference(
        &mut self,
        scope: &SessionReference,
        ephemeral: bool,
    ) -> Result<()> {
        let p = self.load_project(scope.project)?.project;
        ensure!(
            !scope.id.0.is_nil() && p.environments.iter().any(|e| e.id == scope.environment),
            "Invalid login scope"
        );
        self.connection.execute("INSERT INTO local_sessions VALUES(?1,?2,?3,?4) ON CONFLICT(project,environment) DO UPDATE SET session=excluded.session,ephemeral=excluded.ephemeral",params![scope.project.to_string(),scope.environment.to_string(),scope.id.to_string(),ephemeral])?;
        Ok(())
    }
    pub fn forget_session_reference(
        &mut self,
        project: ProjectId,
        env: EnvironmentId,
    ) -> Result<()> {
        self.connection.execute(
            "DELETE FROM local_sessions WHERE project=?1 AND environment=?2",
            params![project.to_string(), env.to_string()],
        )?;
        Ok(())
    }
    pub fn open(paths: AppPaths) -> Result<Self> {
        let database = paths.database();
        regular_file_or_absent(&database)?;
        // SQLite opens these control files itself during recovery/WAL setup.
        // Reject unsafe pre-existing entries before any database modification.
        for suffix in ["-journal", "-wal", "-shm"] {
            regular_file_or_absent(&paths.data.join(format!("regressee.sqlite3{suffix}")))?;
        }
        paths.prepare()?;
        let mut connection = Connection::open(&database)?;
        connection.busy_timeout(Duration::from_secs(3))?;
        connection.pragma_update(None, "foreign_keys", true)?;
        migrate(&mut connection)?;
        // WAL and FULL durability apply after version rejection, not before it.
        connection.pragma_update(None, "journal_mode", "WAL")?;
        connection.pragma_update(None, "synchronous", "FULL")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&database, fs::Permissions::from_mode(0o600))?;
        }
        Ok(Self {
            connection,
            paths,
            instance: uuid::Uuid::new_v4(),
        })
    }
    pub fn save_project(
        &mut self,
        project: &Project,
        expected_revision: Option<u64>,
    ) -> Result<ProjectRecord> {
        self.save_project_inner(project, expected_revision, false, None)
    }
    /// Explicit replacement setup only. The active reference still points to its
    /// frozen run; editing these settings never changes comparison targets.
    pub fn save_replacement_project(
        &mut self,
        project: &Project,
        expected_revision: u64,
    ) -> Result<ProjectRecord> {
        self.save_project_inner(project, Some(expected_revision), true, None)
    }
    /// One deliberate UI save: project settings and the selected local login
    /// configuration commit together. Authentication remains outside project JSON.
    pub fn save_project_and_environment(
        &mut self,
        project: &Project,
        expected_revision: u64,
        environment: EnvironmentId,
        settings: &LocalLoginSettings,
    ) -> Result<ProjectRecord> {
        settings.validate()?;
        ensure!(
            project
                .environments
                .iter()
                .any(|candidate| candidate.id == environment),
            "Unknown environment"
        );
        self.save_project_inner(
            project,
            Some(expected_revision),
            true,
            Some((environment, settings)),
        )
    }
    fn save_project_inner(
        &mut self,
        project: &Project,
        expected_revision: Option<u64>,
        replacement: bool,
        local_login: Option<(EnvironmentId, &LocalLoginSettings)>,
    ) -> Result<ProjectRecord> {
        project.validate()?;
        ensure!(
            project.environments.iter().all(|e| e.session.is_none()),
            "Session references must use private local storage, never project JSON"
        );
        let config = serde_json::to_string(project)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let existing: Option<u64> = tx
            .query_row(
                "SELECT revision FROM projects WHERE id=?1",
                [project.id.to_string()],
                |row| row.get(0),
            )
            .optional()?;
        ensure!(
            existing == expected_revision,
            "this project changed in another window; reopen it before saving"
        );
        let previous: Option<String> = tx
            .query_row(
                "SELECT config FROM projects WHERE id=?1",
                [project.id.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        let old: Option<Project> = previous.map(|s| serde_json::from_str(&s)).transpose()?;
        ensure!(
            project.active_baseline == old.as_ref().and_then(|p| p.active_baseline),
            "baseline activation requires explicit successful capture confirmation"
        );
        if let Some(old) = old.filter(|p| p.active_baseline.is_some() && !replacement) {
            ensure!(
                project.sources == old.sources
                    && project.profiles == old.profiles
                    && project.environments == old.environments,
                "the active single-page baseline fixes its address and capture settings; capture a new baseline before changing them"
            );
        }
        let revision = existing
            .unwrap_or(0)
            .checked_add(1)
            .context("project revision overflow")?;
        tx.execute("INSERT INTO projects(id,name,config,revision,updated_at) VALUES(?1,?2,?3,?4,unixepoch()) ON CONFLICT(id) DO UPDATE SET name=excluded.name,config=excluded.config,revision=excluded.revision,updated_at=excluded.updated_at", params![project.id.to_string(), project.name, config, revision])?;
        if let Some((environment, settings)) = local_login {
            tx.execute("INSERT INTO local_environment_settings VALUES(?1,?2,?3) ON CONFLICT(project) DO UPDATE SET environment=excluded.environment,settings=excluded.settings", params![project.id.to_string(), environment.to_string(), serde_json::to_string(settings)?])?;
        }
        tx.commit()?;
        Ok(ProjectRecord {
            project: project.clone(),
            revision,
        })
    }
    pub fn load_project(&self, id: ProjectId) -> Result<ProjectRecord> {
        let (config, revision): (String, u64) = self.connection.query_row(
            "SELECT config,revision FROM projects WHERE id=?1 AND length(CAST(config AS BLOB))<=?2",
            params![id.to_string(), MAX_PROJECT_BYTES],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        let project: Project = serde_json::from_str(&config)?;
        project.validate()?;
        ensure!(project.id == id, "stored project ID mismatch");
        Ok(ProjectRecord { project, revision })
    }
    pub fn list_projects(&self, page: usize) -> Result<ProjectPage> {
        let offset = page
            .checked_mul(PROJECT_PAGE_SIZE)
            .context("page offset overflow")?;
        ensure!(offset <= 1_000_000, "page offset exceeds limit");
        let mut statement = self.connection.prepare("SELECT id,name,json_extract(config,'$.sources[0]'),revision FROM projects ORDER BY name COLLATE NOCASE,id LIMIT ?1 OFFSET ?2")?;
        let mut items = Vec::new();
        for row in statement.query_map(params![PROJECT_PAGE_SIZE + 1, offset], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, u64>(3)?,
            ))
        })? {
            let (id, name, address, revision) = row?;
            ensure!(
                name.len() <= 480 && address.len() <= 4096,
                "invalid stored project summary"
            );
            let id = serde_json::from_value(serde_json::Value::String(id))?;
            items.push(ProjectSummary {
                id,
                name,
                address,
                revision,
            });
        }
        let more = items.len() > PROJECT_PAGE_SIZE;
        items.truncate(PROJECT_PAGE_SIZE);
        Ok(ProjectPage { items, page, more })
    }
    pub fn summary(&self) -> Result<StorageSummary> {
        let (artifact_bytes, artifacts) = self.connection.query_row(
            "SELECT coalesce(sum(bytes),0),count(*) FROM artifacts",
            [],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let projects = self
            .connection
            .query_row("SELECT count(*) FROM projects", [], |r| r.get(0))?;
        Ok(StorageSummary {
            artifact_bytes,
            artifacts,
            projects,
            paths: self.paths.clone(),
        })
    }
    /// Sync an owned temporary file, then atomically publish without replacement.
    /// The SQLite writer lock serializes publication/reference/collection across
    /// processes. A crash can leave an unreferenced file, never a partial object.
    pub fn put_artifact(&mut self, mut input: impl Read, kind: ArtifactKind) -> Result<Artifact> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let root = self.paths.artifacts();
        portability::require_space(&self.paths.data, 0)?;
        let mut temporary = tempfile::Builder::new()
            .prefix(".pending-")
            .tempfile_in(&root)?;
        let mut hash = Sha256::new();
        let mut bytes = 0u64;
        let mut buffer = [0u8; 64 * 1024];
        loop {
            let count = input.read(&mut buffer)?;
            if count == 0 {
                break;
            }
            bytes += count as u64;
            ensure!(
                bytes <= MAX_ARTIFACT_BYTES,
                "artifact exceeds 128 MiB limit"
            );
            hash.update(&buffer[..count]);
            portability::require_space(&self.paths.data, count as u64)?;
            temporary.write_all(&buffer[..count])?;
        }
        ensure!(bytes > 0, "artifact is empty");
        let id: ArtifactId = hex::encode(hash.finalize()).try_into()?;
        temporary.as_file().sync_all()?;
        let destination = root.join(id.as_str());
        if regular_file_or_absent(&destination)? {
            verify_file(&destination, &id, bytes)?;
        } else {
            match temporary.persist_noclobber(&destination) {
                Ok(file) => {
                    file.sync_all()?;
                    sync_directory(&root)?;
                }
                Err(error) if error.error.kind() == std::io::ErrorKind::AlreadyExists => {
                    verify_file(&destination, &id, bytes)?;
                }
                Err(error) => return Err(error.error.into()),
            }
        }
        let kind_name = kind_name(kind);
        tx.execute(
            "INSERT OR IGNORE INTO artifacts(hash,bytes,kind) VALUES(?1,?2,?3)",
            params![id.as_str(), bytes, kind_name],
        )?;
        let (stored_bytes, stored_kind): (u64, String) = tx.query_row(
            "SELECT bytes,kind FROM artifacts WHERE hash=?1",
            [id.as_str()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            stored_bytes == bytes && stored_kind == kind_name,
            "artifact metadata conflicts with immutable object"
        );
        tx.commit()?;
        Ok(Artifact { id, bytes, kind })
    }
    pub fn verify_artifact(&self, id: &ArtifactId) -> Result<Artifact> {
        let (bytes, kind): (u64, String) = self.connection.query_row(
            "SELECT bytes,kind FROM artifacts WHERE hash=?1",
            [id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        verify_file(&self.paths.artifacts().join(id.as_str()), id, bytes)?;
        let kind = match kind.as_str() {
            "Screenshot" => ArtifactKind::Screenshot,
            "Thumbnail" => ArtifactKind::Thumbnail,
            "Tile" => ArtifactKind::Tile,
            "Difference" => ArtifactKind::Difference,
            _ => bail!("unknown artifact kind"),
        };
        Ok(Artifact {
            id: id.clone(),
            bytes,
            kind,
        })
    }
    /// Read-only display input: hash the exact bounded bytes returned to the
    /// decoder. This is not a trusted-path token or mutation/approval authority.
    pub fn read_verified_artifact(
        &self,
        id: &ArtifactId,
        maximum: u64,
    ) -> Result<(Artifact, Vec<u8>)> {
        let (expected, kind): (u64, String) = self.connection.query_row(
            "SELECT bytes,kind FROM artifacts WHERE hash=?1",
            [id.as_str()],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;
        ensure!(
            expected > 0 && expected <= maximum.min(MAX_ARTIFACT_BYTES),
            "artifact exceeds display byte budget"
        );
        let path = self.paths.artifacts().join(id.as_str());
        let metadata = fs::symlink_metadata(&path)?;
        ensure!(
            metadata.is_file()
                && regressee_core::platform::ordinary(&metadata)
                && metadata.len() == expected,
            "artifact is missing, changed, or unsafe"
        );
        let mut bytes = Vec::with_capacity(expected as usize);
        fs::File::open(path)?
            .take(expected + 1)
            .read_to_end(&mut bytes)?;
        ensure!(
            bytes.len() as u64 == expected && hex::encode(Sha256::digest(&bytes)) == id.as_str(),
            "artifact integrity check failed"
        );
        let kind = match kind.as_str() {
            "Screenshot" => ArtifactKind::Screenshot,
            "Thumbnail" => ArtifactKind::Thumbnail,
            "Tile" => ArtifactKind::Tile,
            "Difference" => ArtifactKind::Difference,
            _ => bail!("unknown artifact kind"),
        };
        Ok((
            Artifact {
                id: id.clone(),
                bytes: expected,
                kind,
            },
            bytes,
        ))
    }
    /// Cheap display-cache checkpoint. SQLite data_version detects commits from
    /// other connections; total_changes detects this owner's writes, including
    /// checkpoint, mask and analysis-display changes without scanning evidence.
    /// Deliberately coarse: unrelated writes may invalidate a display session.
    pub fn viewer_revision(&self) -> Result<(u64, u64)> {
        Ok((
            self.connection
                .pragma_query_value(None, "data_version", |row| row.get(0))?,
            self.connection
                .query_row("SELECT total_changes()", [], |row| row.get(0))?,
        ))
    }
    pub fn add_reference(&mut self, owner: &ArtifactOwner, id: &ArtifactId) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let bytes = tx.query_row(
            "SELECT bytes FROM artifacts WHERE hash=?1",
            [id.as_str()],
            |r| r.get(0),
        )?;
        verify_file(&self.paths.artifacts().join(id.as_str()), id, bytes)?;
        let (kind, owner_id) = owner_parts(owner);
        tx.execute(
            "INSERT OR IGNORE INTO artifact_refs(owner_kind,owner_id,hash) VALUES(?1,?2,?3)",
            params![kind, owner_id, id.as_str()],
        )?;
        tx.commit()?;
        Ok(())
    }
    /// Low-level explicit deletion primitive, not an automatic retention policy.
    pub fn remove_reference(&mut self, owner: &ArtifactOwner, id: &ArtifactId) -> Result<()> {
        let (kind, owner_id) = owner_parts(owner);
        self.connection.execute(
            "DELETE FROM artifact_refs WHERE owner_kind=?1 AND owner_id=?2 AND hash=?3",
            params![kind, owner_id, id.as_str()],
        )?;
        Ok(())
    }
    pub fn collect_unreferenced(&mut self, id: &ArtifactId) -> Result<bool> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let references: u64 = tx.query_row(
            "SELECT count(*) FROM artifact_refs WHERE hash=?1",
            [id.as_str()],
            |r| r.get(0),
        )?;
        if references != 0 {
            return Ok(false);
        }
        let path = self.paths.artifacts().join(id.as_str());
        if regular_file_or_absent(&path)? {
            fs::remove_file(path)?;
            sync_directory(&self.paths.artifacts())?;
        }
        tx.execute("DELETE FROM artifacts WHERE hash=?1", [id.as_str()])?;
        tx.commit()?;
        Ok(true)
    }
}
fn owner_parts(owner: &ArtifactOwner) -> (&'static str, String) {
    match owner {
        ArtifactOwner::Run(id) => ("Run", id.to_string()),
        ArtifactOwner::Baseline(id) => ("Baseline", id.to_string()),
        ArtifactOwner::Analysis(id) => ("Analysis", id.to_string()),
    }
}
fn kind_name(kind: ArtifactKind) -> &'static str {
    match kind {
        ArtifactKind::Screenshot => "Screenshot",
        ArtifactKind::Thumbnail => "Thumbnail",
        ArtifactKind::Tile => "Tile",
        ArtifactKind::Difference => "Difference",
    }
}
fn verify_file(path: &Path, id: &ArtifactId, expected: u64) -> Result<()> {
    let metadata = fs::symlink_metadata(path)?;
    ensure!(
        metadata.is_file() && metadata.len() == expected && expected <= MAX_ARTIFACT_BYTES,
        "artifact is missing, changed, or unsafe"
    );
    let file = fs::File::open(path)?;
    let mut reader = file.take(MAX_ARTIFACT_BYTES + 1);
    let mut hash = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    let mut bytes = 0;
    loop {
        let n = reader.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        bytes += n as u64;
        hash.update(&buffer[..n]);
    }
    ensure!(
        bytes == expected && hex::encode(hash.finalize()) == id.as_str(),
        "artifact integrity check failed"
    );
    Ok(())
}
fn sync_directory(path: &Path) -> Result<()> {
    #[cfg(unix)]
    fs::File::open(path)?.sync_all()?;
    #[cfg(not(unix))]
    let _ = path; // Windows file sync + no-clobber publication; qualification pending.
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn isolated() -> (tempfile::TempDir, AppPaths) {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(root.path());
        (root, paths)
    }
    #[test]
    fn display_bytes_are_exact_bounded_and_revision_is_connection_sensitive() {
        let (_root, paths) = isolated();
        let mut store = Store::open(paths.clone()).unwrap();
        let first = store.viewer_revision().unwrap();
        let artifact = store
            .put_artifact(&b"verified display bytes"[..], ArtifactKind::Tile)
            .unwrap();
        let next = store.viewer_revision().unwrap();
        assert_ne!(first, next);
        assert_eq!(
            store.read_verified_artifact(&artifact.id, 128).unwrap().1,
            b"verified display bytes"
        );
        assert!(store.read_verified_artifact(&artifact.id, 2).is_err());
        let mut other = Store::open(paths.clone()).unwrap();
        let project = Project::draft("Other connection".into(), "https://example.test/").unwrap();
        other.save_project(&project, None).unwrap();
        assert_ne!(store.viewer_revision().unwrap().0, next.0);
        assert_eq!(store.viewer_revision().unwrap().1, next.1);
        fs::write(
            paths.artifacts().join(artifact.id.as_str()),
            b"corrupt! display bytes",
        )
        .unwrap();
        assert!(store.read_verified_artifact(&artifact.id, 128).is_err());
        fs::remove_file(paths.artifacts().join(artifact.id.as_str())).unwrap();
        assert!(store.read_verified_artifact(&artifact.id, 128).is_err());
    }
    #[test]
    fn coordinated_ui_save_is_atomic_private_and_revision_bound() {
        let (_root, paths) = isolated();
        let mut store = Store::open(paths.clone()).unwrap();
        let mut project =
            Project::draft("UI save fixture".into(), "https://example.test/docs").unwrap();
        store.save_project(&project, None).unwrap();
        let old_settings = LocalLoginSettings::default();
        store
            .select_environment(project.id, project.environments[0].id, &old_settings)
            .unwrap();
        let new_settings = LocalLoginSettings {
            login_address: Some(url::Url::parse("https://example.test/login").unwrap()),
            ephemeral: true,
            ..Default::default()
        };
        project.name = "Edited website and login".into();
        let environment = project.environments[0].id;
        assert!(
            store
                .save_project_and_environment(&project, 0, environment, &new_settings)
                .is_err()
        );
        assert!(
            store
                .save_project_and_environment(&project, 1, EnvironmentId::new(), &new_settings)
                .is_err()
        );
        assert_eq!(store.load_project(project.id).unwrap().revision, 1);
        assert_eq!(
            store.local_login_settings(project.id).unwrap(),
            old_settings
        );
        store.connection.execute_batch("CREATE TRIGGER fail_ui_local_save BEFORE INSERT ON local_environment_settings BEGIN SELECT RAISE(ABORT, 'owned atomic-save negative probe'); END;").unwrap();
        assert!(
            store
                .save_project_and_environment(&project, 1, environment, &new_settings)
                .is_err()
        );
        assert_eq!(
            store.load_project(project.id).unwrap().project.name,
            "UI save fixture"
        );
        assert_eq!(store.load_project(project.id).unwrap().revision, 1);
        assert_eq!(
            store.local_login_settings(project.id).unwrap(),
            old_settings
        );
        store
            .connection
            .execute_batch("DROP TRIGGER fail_ui_local_save;")
            .unwrap();
        assert_eq!(
            store
                .save_project_and_environment(&project, 1, environment, &new_settings)
                .unwrap()
                .revision,
            2
        );
        assert!(
            !serde_json::to_string(&store.load_project(project.id).unwrap().project)
                .unwrap()
                .contains("/login")
        );
        drop(store);
        let store = Store::open(paths).unwrap();
        assert_eq!(
            store.load_project(project.id).unwrap().project.name,
            project.name
        );
        assert_eq!(
            store.local_login_settings(project.id).unwrap(),
            new_settings
        );
    }
    #[test]
    fn saves_reopens_and_checks_revision_conflicts() {
        let (_root, paths) = isolated();
        let p = Project::draft("Documentation".into(), "https://example.test/docs").unwrap();
        let mut store = Store::open(paths.clone()).unwrap();
        store.save_project(&p, None).unwrap();
        drop(store);
        let mut store = Store::open(paths).unwrap();
        let mut record = store.load_project(p.id).unwrap();
        assert_eq!(record.project, p);
        record.project.name = "Release docs".into();
        assert_eq!(
            store
                .save_project(&record.project, Some(1))
                .unwrap()
                .revision,
            2
        );
        assert!(store.save_project(&p, Some(1)).is_err());
        assert_eq!(
            store.list_projects(0).unwrap().items[0].name,
            "Release docs"
        );
    }
    #[test]
    fn migrates_v1_fixture_without_losing_project_and_is_idempotent() {
        let (_root, paths) = isolated();
        paths.prepare().unwrap();
        let connection = Connection::open(paths.database()).unwrap();
        connection.execute_batch(MIGRATIONS[0]).unwrap();
        connection.pragma_update(None, "user_version", 1).unwrap();
        let p = Project::draft("Old project".into(), "https://example.test").unwrap();
        connection
            .execute(
                "INSERT INTO projects VALUES(?1,?2,?3,1,0)",
                params![p.id.to_string(), p.name, serde_json::to_string(&p).unwrap()],
            )
            .unwrap();
        drop(connection);
        let mut store = Store::open(paths.clone()).unwrap();
        assert_eq!(
            store
                .connection
                .pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            DB_VERSION
        );
        migrate(&mut store.connection).unwrap();
        drop(store);
        assert_eq!(
            Store::open(paths)
                .unwrap()
                .load_project(p.id)
                .unwrap()
                .project,
            p
        );
    }
    #[test]
    fn future_and_failed_migrations_do_not_mutate_schema() {
        let mut c = Connection::open_in_memory().unwrap();
        c.pragma_update(None, "user_version", 999).unwrap();
        assert!(migrate(&mut c).is_err());
        assert_eq!(
            c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            999
        );
        c.pragma_update(None, "user_version", 0).unwrap();
        c.execute_batch("CREATE TABLE artifacts(x)").unwrap();
        assert!(migrate(&mut c).is_err());
        assert_eq!(
            c.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
                .unwrap(),
            0
        );
        let projects: i64 = c
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE name='projects'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(projects, 0);
    }
    #[test]
    fn content_addressed_dedup_integrity_and_reference_safety() {
        let (_root, paths) = isolated();
        let mut store = Store::open(paths.clone()).unwrap();
        let a = store
            .put_artifact(&b"original evidence"[..], ArtifactKind::Screenshot)
            .unwrap();
        assert_eq!(
            store
                .put_artifact(&b"original evidence"[..], ArtifactKind::Screenshot)
                .unwrap(),
            a
        );
        let owner = ArtifactOwner::Baseline(BaselineId::new());
        store.add_reference(&owner, &a.id).unwrap();
        store.add_reference(&owner, &a.id).unwrap();
        assert!(!store.collect_unreferenced(&a.id).unwrap());
        store.verify_artifact(&a.id).unwrap();
        fs::write(paths.artifacts().join(a.id.as_str()), b"tampered evidence").unwrap();
        assert!(store.verify_artifact(&a.id).is_err());
        assert!(
            store
                .add_reference(&ArtifactOwner::Run(RunId::new()), &a.id)
                .is_err()
        );
        assert!(!store.collect_unreferenced(&a.id).unwrap());
        store.remove_reference(&owner, &a.id).unwrap();
        assert!(store.collect_unreferenced(&a.id).unwrap());
        assert!(!paths.artifacts().join(a.id.as_str()).exists());
    }
    #[test]
    fn interrupted_stream_does_not_publish_or_leave_temporary() {
        struct Failing;
        impl Read for Failing {
            fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
                Err(std::io::Error::other("disk/read interruption"))
            }
        }
        let (_root, paths) = isolated();
        let mut store = Store::open(paths.clone()).unwrap();
        assert!(
            store
                .put_artifact(Failing, ArtifactKind::Screenshot)
                .is_err()
        );
        assert!(
            store
                .put_artifact(&b""[..], ArtifactKind::Screenshot)
                .is_err()
        );
        assert_eq!(fs::read_dir(paths.artifacts()).unwrap().count(), 0);
        assert_eq!(store.summary().unwrap().artifacts, 0);
    }
    #[test]
    fn oversized_stream_is_bounded_and_not_published() {
        let (_root, paths) = isolated();
        let mut store = Store::open(paths.clone()).unwrap();
        assert!(
            store
                .put_artifact(
                    std::io::repeat(0).take(MAX_ARTIFACT_BYTES + 1),
                    ArtifactKind::Screenshot
                )
                .is_err()
        );
        assert_eq!(fs::read_dir(paths.artifacts()).unwrap().count(), 0);
        assert_eq!(store.summary().unwrap().artifacts, 0);
    }
    #[test]
    fn concurrent_publication_is_deduplicated() {
        let (_root, paths) = isolated();
        let mut first = Store::open(paths.clone()).unwrap();
        let thread = std::thread::spawn(move || {
            Store::open(paths)
                .unwrap()
                .put_artifact(&b"same evidence"[..], ArtifactKind::Screenshot)
                .unwrap()
        });
        let a = first
            .put_artifact(&b"same evidence"[..], ArtifactKind::Screenshot)
            .unwrap();
        assert_eq!(thread.join().unwrap(), a);
        assert_eq!(first.summary().unwrap().artifacts, 1);
    }
    #[test]
    fn pages_are_bounded_and_no_cwd_storage() {
        let (_root, paths) = isolated();
        let mut store = Store::open(paths).unwrap();
        for i in 0..65 {
            store
                .save_project(
                    &Project::draft(format!("Project {i:03}"), "https://example.test").unwrap(),
                    None,
                )
                .unwrap();
        }
        let first = store.list_projects(0).unwrap();
        assert_eq!(first.items.len(), 64);
        assert!(first.more);
        assert_eq!(store.list_projects(1).unwrap().items.len(), 1);
        let os = AppPaths::discover().unwrap();
        assert!(os.data.is_absolute() && os.cache.is_absolute());
        assert_ne!(os.data, std::env::current_dir().unwrap());
    }
    #[cfg(unix)]
    #[test]
    fn database_and_sqlite_control_symlinks_fail_before_outside_writes() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        for suffix in ["", "-journal", "-wal", "-shm"] {
            for existing in [false, true] {
                let (root, paths) = isolated();
                paths.prepare().unwrap();
                let outside = root.path().join("outside.sqlite3");
                if existing {
                    fs::write(&outside, b"outside must remain unchanged").unwrap();
                    fs::set_permissions(&outside, fs::Permissions::from_mode(0o640)).unwrap();
                }
                let unsafe_path = paths.data.join(format!("regressee.sqlite3{suffix}"));
                symlink(&outside, &unsafe_path).unwrap();
                let error = Store::open(paths.clone())
                    .err()
                    .expect("unsafe entry was accepted");
                assert!(
                    error.to_string().contains("not a regular file"),
                    "{error:#}"
                );
                assert!(
                    fs::symlink_metadata(&unsafe_path)
                        .unwrap()
                        .file_type()
                        .is_symlink()
                );
                if existing {
                    assert_eq!(
                        fs::read(&outside).unwrap(),
                        b"outside must remain unchanged"
                    );
                    assert_eq!(
                        fs::metadata(&outside).unwrap().permissions().mode() & 0o777,
                        0o640
                    );
                } else {
                    assert!(
                        fs::symlink_metadata(&outside)
                            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                    );
                }
                if !suffix.is_empty() {
                    assert!(
                        fs::symlink_metadata(paths.database())
                            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                    );
                }
                assert_eq!(
                    fs::read_dir(root.path()).unwrap().count(),
                    if existing { 3 } else { 2 }
                );
            }
        }
    }
    #[cfg(unix)]
    #[test]
    fn storage_rejects_symlink_directories_and_ancestors_before_creation() {
        use std::os::unix::fs::{PermissionsExt, symlink};
        for location in ["data", "cache", "artifacts", "ancestor"] {
            for existing in [false, true] {
                let (root, mut paths) = isolated();
                let outside = root.path().join("outside");
                if existing {
                    fs::create_dir(&outside).unwrap();
                    fs::write(outside.join("sentinel"), b"unchanged").unwrap();
                    fs::set_permissions(&outside, fs::Permissions::from_mode(0o750)).unwrap();
                }
                let link = match location {
                    "data" => paths.data.clone(),
                    "cache" => paths.cache.clone(),
                    "artifacts" => {
                        fs::create_dir(&paths.data).unwrap();
                        paths.artifacts()
                    }
                    "ancestor" => {
                        let link = root.path().join("ancestor");
                        paths = AppPaths::isolated(&link.join("not-created"));
                        link
                    }
                    _ => unreachable!(),
                };
                symlink(&outside, &link).unwrap();
                let error = Store::open(paths.clone())
                    .err()
                    .expect("unsafe directory was accepted");
                assert!(
                    error.to_string().contains("not a real directory"),
                    "{error:#}"
                );
                if existing {
                    assert_eq!(fs::read(outside.join("sentinel")).unwrap(), b"unchanged");
                    assert_eq!(fs::read_dir(&outside).unwrap().count(), 1);
                    assert_eq!(
                        fs::metadata(&outside).unwrap().permissions().mode() & 0o777,
                        0o750
                    );
                } else {
                    assert!(
                        fs::symlink_metadata(&outside)
                            .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
                    );
                }
                assert!(fs::symlink_metadata(paths.database()).is_err());
            }
        }
    }
    #[cfg(unix)]
    #[test]
    fn dangling_artifact_entry_is_not_published_or_collected() {
        use std::os::unix::fs::symlink;
        let (root, paths) = isolated();
        let mut store = Store::open(paths.clone()).unwrap();
        let a = store
            .put_artifact(&b"evidence"[..], ArtifactKind::Screenshot)
            .unwrap();
        let file = paths.artifacts().join(a.id.as_str());
        fs::remove_file(&file).unwrap();
        let outside = root.path().join("absent-object");
        symlink(&outside, &file).unwrap();
        for error in [
            store
                .put_artifact(&b"evidence"[..], ArtifactKind::Screenshot)
                .unwrap_err(),
            store.collect_unreferenced(&a.id).unwrap_err(),
        ] {
            assert!(
                error.to_string().contains("not a regular file"),
                "{error:#}"
            );
        }
        assert!(
            fs::symlink_metadata(&outside).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound)
        );
        assert!(
            fs::symlink_metadata(&file)
                .unwrap()
                .file_type()
                .is_symlink()
        );
        assert_eq!(store.summary().unwrap().artifacts, 1);
        assert_eq!(fs::read_dir(paths.artifacts()).unwrap().count(), 1);
    }
    #[test]
    fn nonregular_database_and_invalid_directory_layout_fail_closed() {
        validate_directory_path(Path::new(".")).unwrap();
        let (root, paths) = isolated();
        paths.prepare().unwrap();
        fs::create_dir(paths.database()).unwrap();
        assert!(
            Store::open(paths)
                .err()
                .unwrap()
                .to_string()
                .contains("not a regular file")
        );
        let paths = AppPaths::isolated(&root.path().join("file-parent"));
        fs::write(root.path().join("file-parent"), b"unchanged").unwrap();
        assert!(
            Store::open(paths)
                .err()
                .unwrap()
                .to_string()
                .contains("not a real directory")
        );
        let paths = AppPaths::isolated(&root.path().join("missing/../traversal"));
        assert!(
            Store::open(paths)
                .err()
                .unwrap()
                .to_string()
                .contains("parent traversal")
        );
        assert!(!root.path().join("missing").exists());
    }
    #[cfg(unix)]
    #[test]
    fn symlink_objects_are_never_referenced() {
        use std::os::unix::fs::symlink;
        let (root, paths) = isolated();
        let mut store = Store::open(paths.clone()).unwrap();
        let a = store
            .put_artifact(&b"evidence"[..], ArtifactKind::Screenshot)
            .unwrap();
        let file = paths.artifacts().join(a.id.as_str());
        fs::remove_file(&file).unwrap();
        let outside = root.path().join("outside");
        fs::write(&outside, b"evidence").unwrap();
        symlink(&outside, &file).unwrap();
        assert!(store.verify_artifact(&a.id).is_err());
        assert!(store.collect_unreferenced(&a.id).is_err());
        assert_eq!(fs::read(outside).unwrap(), b"evidence");
    }
}
#[test]
fn phase_three_migration_preserves_old_project_bytes_and_safe_defaults() {
    let root = tempfile::tempdir().unwrap();
    let paths = AppPaths::isolated(root.path());
    paths.prepare().unwrap();
    let mut connection = Connection::open(paths.database()).unwrap();
    for migration in MIGRATIONS.iter().take(3) {
        connection.execute_batch(migration).unwrap();
    }
    connection.pragma_update(None, "user_version", 3).unwrap();
    let project = Project::draft(
        "Old saved settings".into(),
        "https://example.test/docs?a=2&a=1",
    )
    .unwrap();
    let mut value = serde_json::to_value(&project).unwrap();
    value["discovery"]
        .as_object_mut()
        .unwrap()
        .remove("advanced");
    let original = serde_json::to_string(&value).unwrap();
    connection
        .execute(
            "INSERT INTO projects VALUES(?1,?2,?3,1,0)",
            params![project.id.to_string(), project.name, original],
        )
        .unwrap();
    migrate(&mut connection).unwrap();
    migrate(&mut connection).unwrap();
    let retained: String = connection
        .query_row("SELECT config FROM projects", [], |r| r.get(0))
        .unwrap();
    assert_eq!(retained, original);
    drop(connection);
    let reopened = Store::open(paths)
        .unwrap()
        .load_project(project.id)
        .unwrap();
    assert_eq!(reopened.project, project);
    assert_eq!(reopened.revision, 1);
}
