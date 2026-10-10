// SPDX-License-Identifier: MPL-2.0
//! Explicit reference-aware deletion; preview authority includes all owners.
use super::*;
use regressee_core::evidence::{RunRecord, identity_hash};
use std::collections::{BTreeMap, BTreeSet};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeleteItem {
    Run(RunId),
    Baseline(BaselineId),
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeletionPreview {
    pub project: ProjectId,
    pub revision: u64,
    pub item: DeleteItem,
    pub fingerprint: String,
    pub bytes: u64,
    pub shared_bytes: u64,
    pub objects: usize,
    pub analyses: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStorage {
    pub project: ProjectId,
    pub bytes: u64,
    pub original_bytes: u64,
    pub derived_bytes: u64,
    pub exclusive_bytes: u64,
    pub shared_bytes: u64,
    pub available_bytes: Option<u64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryPage {
    pub runs: Vec<RunRecord>,
    pub baselines: Vec<BaselineVersion>,
    pub imported_runs: Vec<RunId>,
    pub page: usize,
    pub more: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RepairPreview {
    pub fingerprint: String,
    pub temporaries: Vec<String>,
    pub interrupted_imports: Vec<String>,
    pub valid_unreferenced: Vec<String>,
    pub missing_or_corrupt: Vec<String>,
    pub temporary_bytes: u64,
}
fn project_owners(c: &Connection, project: ProjectId) -> Result<BTreeSet<(String, String)>> {
    let mut q=c.prepare("SELECT 'Run',id FROM runs WHERE project_id=?1 UNION SELECT 'Baseline',id FROM baselines WHERE project_id=?1 UNION SELECT 'Analysis',a.id FROM analyses a JOIN runs r ON r.id=a.run_id WHERE r.project_id=?1")?;
    Ok(
        q.query_map([project.to_string()], |r| Ok((r.get(0)?, r.get(1)?)))?
            .collect::<rusqlite::Result<_>>()?,
    )
}
fn refs(c: &Connection) -> Result<BTreeMap<String, BTreeSet<(String, String)>>> {
    let mut q = c.prepare(
        "SELECT hash,owner_kind,owner_id FROM artifact_refs ORDER BY hash,owner_kind,owner_id",
    )?;
    let mut map = BTreeMap::<_, BTreeSet<_>>::new();
    for row in q.query_map([], |r| {
        Ok((
            r.get::<_, String>(0)?,
            r.get::<_, String>(1)?,
            r.get::<_, String>(2)?,
        ))
    })? {
        let (hash, kind, id) = row?;
        map.entry(hash).or_default().insert((kind, id));
    }
    Ok(map)
}
type DeletionAuthority = (DeletionPreview, BTreeSet<(String, String)>, Vec<ArtifactId>);
fn deletion(
    c: &Connection,
    paths: &AppPaths,
    project: ProjectId,
    item: DeleteItem,
) -> Result<DeletionAuthority> {
    let (s, revision): (String, u64) = c.query_row(
        "SELECT config,revision FROM projects WHERE id=?1",
        [project.to_string()],
        |r| Ok((r.get(0)?, r.get(1)?)),
    )?;
    let p: Project = serde_json::from_str(&s)?;
    let mut owners = BTreeSet::new();
    let mut analyses = 0;
    let mut q = c.prepare("SELECT record FROM baselines WHERE project_id=?1")?;
    let baselines = q
        .query_map([project.to_string()], |r| r.get::<_, String>(0))?
        .map(|r| Ok(serde_json::from_str::<BaselineVersion>(&r?)?))
        .collect::<Result<Vec<_>>>()?;
    match item {
        DeleteItem::Run(id) => {
            let (owner, state): (String, String) = c.query_row(
                "SELECT project_id,state FROM runs WHERE id=?1",
                [id.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            ensure!(
                owner == project.to_string(),
                "Run belongs to another project"
            );
            let state: JobState = serde_json::from_str(&state)?;
            ensure!(
                !matches!(
                    state,
                    JobState::Running | JobState::Pausing | JobState::Queued
                ),
                "Current active capture cannot be deleted; stop it first"
            );
            ensure!(
                !baselines
                    .iter()
                    .any(|b| b.references.iter().any(|r| r.run == id)),
                "Run contains originals referenced by a retained baseline (possibly a composite). Keep it, or delete those inactive baselines first; shared evidence will not be deleted."
            );
            owners.insert(("Run".into(), id.to_string()));
            let mut q = c.prepare("SELECT id FROM analyses WHERE run_id=?1")?;
            for id in q.query_map([id.to_string()], |r| r.get::<_, String>(0))? {
                owners.insert(("Analysis".into(), id?));
                analyses += 1;
            }
        }
        DeleteItem::Baseline(id) => {
            let b = baselines
                .iter()
                .find(|b| b.id == id)
                .context("Baseline not in this project")?;
            ensure!(
                p.active_baseline != Some(id),
                "Active baseline is protected; explicitly set another retained version active first"
            );
            ensure!(
                !baselines.iter().any(|b| b.parent == Some(id)),
                "A retained baseline inherits this version. Delete inactive child versions first; history links are protected."
            );
            let dependencies: u64 = c.query_row(
                "SELECT count(*) FROM analyses WHERE baseline_id=?1",
                [id.to_string()],
                |r| r.get(0),
            )?;
            ensure!(
                dependencies == 0,
                "Retained check analyses use this baseline; explicitly delete their runs first"
            );
            let mut q = c.prepare("SELECT record FROM runs WHERE project_id=?1")?;
            for s in q.query_map([project.to_string()], |r| r.get::<_, String>(0))? {
                let r: RunRecord = serde_json::from_str(&s?)?;
                ensure!(
                    r.run.intent != RunIntent::Check { baseline: id },
                    "A retained check run uses this baseline; delete that run first"
                );
            }
            b.validate()?;
            owners.insert(("Baseline".into(), id.to_string()));
        }
    }
    let references = refs(c)?;
    let mut bytes = 0;
    let mut shared_bytes = 0;
    let mut candidates = Vec::new();
    let mut files = Vec::new();
    for (hash, existing) in &references {
        if !existing.is_disjoint(&owners) {
            let meta = entry_metadata(&paths.artifacts().join(hash))?;
            ensure!(
                meta.as_ref().is_none_or(|m| m.is_file()),
                "Unsafe evidence object; retain it for recovery"
            );
            let size = meta.as_ref().map(|m| m.len()).unwrap_or(0);
            files.push((hash.clone(), size, meta.and_then(|m| m.modified().ok())));
            if existing.is_subset(&owners) {
                bytes += size;
                candidates.push(hash.clone().try_into()?);
            } else {
                shared_bytes += size;
            }
        }
    }
    // Whole reference graph, immutable records and project revision prevent stale impact authority.
    let graph = {
        let mut q=c.prepare("SELECT 'Run',id,record FROM runs UNION SELECT 'Baseline',id,record FROM baselines UNION SELECT 'Analysis',id,record FROM analyses ORDER BY 1,2")?;
        q.query_map([], |r| {
            Ok((
                r.get::<_, String>(0)?,
                r.get::<_, String>(1)?,
                r.get::<_, String>(2)?,
            ))
        })?
        .collect::<rusqlite::Result<Vec<_>>>()?
    };
    let fingerprint = identity_hash(&(project, revision, item, &references, graph, files))?;
    Ok((
        DeletionPreview {
            project,
            revision,
            item,
            fingerprint,
            bytes,
            shared_bytes,
            objects: candidates.len(),
            analyses,
        },
        owners,
        candidates,
    ))
}
impl Store {
    pub fn history(&self, project: ProjectId, page: usize) -> Result<HistoryPage> {
        self.load_project(project)?;
        ensure!(page <= 15625, "History page exceeds limit");
        let mut q = self.connection.prepare(
            "SELECT id FROM runs WHERE project_id=?1 ORDER BY rowid DESC LIMIT 65 OFFSET ?2",
        )?;
        let ids = q
            .query_map(params![project.to_string(), page * 64], |r| {
                r.get::<_, String>(0)
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        let more = ids.len() > 64;
        let mut runs = Vec::new();
        let mut imported_runs = Vec::new();
        for id in ids.into_iter().take(64) {
            let id: RunId = serde_json::from_value(serde_json::json!(id))?;
            // History remains inspectable for repair even if a blob is corrupt; activation separately verifies.
            let s: String = self.connection.query_row(
                "SELECT record FROM runs WHERE id=?1",
                [id.to_string()],
                |r| r.get(0),
            )?;
            runs.push(serde_json::from_str(&s)?);
            if self.is_imported_run(id)? {
                imported_runs.push(id);
            }
        }
        let mut q=self.connection.prepare("SELECT record FROM baselines WHERE project_id=?1 ORDER BY rowid DESC LIMIT 65 OFFSET ?2")?;
        let mut baselines = q
            .query_map(params![project.to_string(), page * 64], |r| {
                r.get::<_, String>(0)
            })?
            .map(|s| Ok(serde_json::from_str::<BaselineVersion>(&s?)?))
            .collect::<Result<Vec<_>>>()?;
        let more = more || baselines.len() > 64;
        baselines.truncate(64);
        Ok(HistoryPage {
            runs,
            baselines,
            imported_runs,
            page,
            more,
        })
    }
    pub fn is_imported_run(&self, run: RunId) -> Result<bool> {
        Ok(self.connection.query_row(
            "SELECT count(*) FROM imported_origins WHERE run_id=?1",
            [run.to_string()],
            |r| r.get::<_, u64>(0),
        )? != 0)
    }
    pub fn project_storage(&self, project: ProjectId) -> Result<ProjectStorage> {
        self.load_project(project)?;
        let owned = project_owners(&self.connection, project)?;
        let references = refs(&self.connection)?;
        let mut summary = ProjectStorage {
            project,
            bytes: 0,
            original_bytes: 0,
            derived_bytes: 0,
            exclusive_bytes: 0,
            shared_bytes: 0,
            available_bytes: available_bytes(&self.paths.data).ok(),
        };
        for (hash, owners) in references {
            if owners.is_disjoint(&owned) {
                continue;
            }
            let kind: String = self.connection.query_row(
                "SELECT kind FROM artifacts WHERE hash=?1",
                [&hash],
                |r| r.get(0),
            )?;
            let path = self.paths.artifacts().join(&hash);
            let bytes = match entry_metadata(&path)? {
                Some(m) if m.is_file() => m.len(),
                _ => 0,
            };
            summary.bytes += bytes;
            if kind == "Screenshot" {
                summary.original_bytes += bytes;
            } else {
                summary.derived_bytes += bytes;
            }
            if owners.is_subset(&owned) {
                summary.exclusive_bytes += bytes;
            } else {
                summary.shared_bytes += bytes;
            }
        }
        Ok(summary)
    }
    pub fn deletion_preview(
        &self,
        project: ProjectId,
        item: DeleteItem,
    ) -> Result<DeletionPreview> {
        Ok(deletion(&self.connection, &self.paths, project, item)?.0)
    }
    pub fn delete_confirmed(&mut self, preview: &DeletionPreview) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (fresh, owners, candidates) =
            deletion(&tx, &self.paths, preview.project, preview.item)?;
        ensure!(
            &fresh == preview,
            "Deletion impact/active baseline changed; inspect impact again"
        );
        for (kind, id) in &owners {
            tx.execute(
                "DELETE FROM artifact_refs WHERE owner_kind=?1 AND owner_id=?2",
                params![kind, id],
            )?;
            if kind == "Analysis" {
                tx.execute("DELETE FROM review_decisions WHERE analysis_id=?1", [id])?;
                tx.execute("DELETE FROM analysis_display WHERE analysis_id=?1", [id])?;
                tx.execute("DELETE FROM analyses WHERE id=?1", [id])?;
            }
        }
        match preview.item {
            DeleteItem::Run(id) => {
                tx.execute(
                    "DELETE FROM imported_origins WHERE run_id=?1",
                    [id.to_string()],
                )?;
                tx.execute(
                    "DELETE FROM analysis_display WHERE run_id=?1",
                    [id.to_string()],
                )?;
                // Pair-bound masks are history too: refuse orphaning them, rather than secretly deleting settings.
                let mut q = tx.prepare("SELECT record FROM ignore_masks WHERE project_id=?1")?;
                for s in q.query_map([preview.project.to_string()], |r| r.get::<_, String>(0))? {
                    let mask: regressee_core::analysis_engine::MaskBinding =
                        serde_json::from_str(&s?)?;
                    ensure!(
                        !mask.before_capture_id.starts_with(&format!("{id}:"))
                            && !mask.after_capture_id.starts_with(&format!("{id}:")),
                        "Retained ignored-region revision binds this run; keep it to preserve mask history"
                    );
                }
                drop(q);
                tx.execute("DELETE FROM runs WHERE id=?1", [id.to_string()])?;
            }
            DeleteItem::Baseline(id) => {
                tx.execute("DELETE FROM baselines WHERE id=?1", [id.to_string()])?;
            }
        }
        for id in candidates {
            tx.execute("INSERT OR IGNORE INTO pending_gc VALUES(?1)", [id.as_str()])?;
        }
        tx.execute(
            "UPDATE projects SET revision=revision+1,updated_at=unixepoch() WHERE id=?1",
            [preview.project.to_string()],
        )?;
        tx.commit()?;
        self.complete_confirmed_gc()?;
        Ok(())
    }
    /// Only journals resulting from an explicitly confirmed deletion; never scan-and-GC.
    pub fn complete_confirmed_gc(&mut self) -> Result<()> {
        let ids = {
            let mut q = self.connection.prepare("SELECT hash FROM pending_gc")?;
            q.query_map([], |r| r.get::<_, String>(0))?
                .collect::<rusqlite::Result<Vec<_>>>()?
        };
        for id in ids {
            let id: ArtifactId = id.try_into()?;
            self.collect_unreferenced(&id)?;
            self.connection
                .execute("DELETE FROM pending_gc WHERE hash=?1", [id.as_str()])?;
        }
        Ok(())
    }
    pub fn repair_preview(&self) -> Result<RepairPreview> {
        Ok(repair_snapshot(&self.connection, &self.paths)?.0)
    }
    pub fn cleanup_temporaries(&mut self, preview: &RepairPreview) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        // Reader runs under writer lock; no cooperating artifact writer can be active.
        let (fresh, stages) = repair_snapshot(&tx, &self.paths)?;
        ensure!(&fresh == preview, "Repair impact changed; inspect again");
        for name in &preview.temporaries {
            ensure!(
                name.starts_with(".pending-") && !name.contains(['/', '\\']),
                "Unsafe temporary cleanup path"
            );
            let p = self.paths.artifacts().join(name);
            ensure!(regular_file_or_absent(&p)?, "Temporary file missing");
            fs::remove_file(p)?;
        }
        for stage in stages {
            let root = self.paths.data.join(stage.name);
            // All entries have just been bounded and checked without following symlinks.
            for (relative, _, _, directory) in stage.entries.into_iter().rev() {
                let p = root.join(relative);
                if directory {
                    fs::remove_dir(p)?;
                } else {
                    fs::remove_file(p)?;
                }
            }
            fs::remove_dir(root)?;
        }
        sync_directory(&self.paths.artifacts())?;
        tx.commit()?;
        Ok(())
    }
    pub fn delete_unreferenced_confirmed(&mut self, preview: &RepairPreview) -> Result<()> {
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure!(
            &repair_snapshot(&tx, &self.paths)?.0 == preview,
            "Recovery-object impact changed; inspect again"
        );
        for hash in &preview.valid_unreferenced {
            let id: ArtifactId = hash.clone().try_into()?;
            tx.execute("INSERT OR IGNORE INTO pending_gc VALUES(?1)", [id.as_str()])?;
        }
        tx.commit()?;
        self.complete_confirmed_gc()?;
        Ok(())
    }
}
fn repair_snapshot(
    c: &Connection,
    paths: &AppPaths,
) -> Result<(RepairPreview, Vec<InterruptedStage>)> {
    let references = refs(c)?;
    let mut preview = RepairPreview {
        fingerprint: String::new(),
        temporaries: vec![],
        interrupted_imports: vec![],
        valid_unreferenced: vec![],
        missing_or_corrupt: vec![],
        temporary_bytes: 0,
    };
    let mut facts = Vec::new();
    for e in fs::read_dir(paths.artifacts())? {
        let e = e?;
        let name = e.file_name().to_string_lossy().into_owned();
        let meta = fs::symlink_metadata(e.path())?;
        facts.push((name.clone(), meta.len(), meta.modified().ok()));
        if name.starts_with(".pending-") && meta.is_file() {
            preview.temporary_bytes += meta.len();
            preview.temporaries.push(name);
            continue;
        }
        if let Ok(id) = ArtifactId::try_from(name.clone())
            && references.get(&name).is_none_or(|r| r.is_empty())
            && meta.is_file()
            && verify_file(&e.path(), &id, meta.len()).is_ok()
        {
            preview.valid_unreferenced.push(name);
        }
    }
    let mut q = c.prepare("SELECT hash,bytes FROM artifacts")?;
    for row in q.query_map([], |r| Ok((r.get::<_, String>(0)?, r.get::<_, u64>(1)?)))? {
        let (id, bytes) = row?;
        if verify_file(&paths.artifacts().join(&id), &id.clone().try_into()?, bytes).is_err() {
            preview.missing_or_corrupt.push(id);
        }
    }
    facts.sort_by(|a, b| a.0.cmp(&b.0));
    preview.temporaries.sort();
    preview.valid_unreferenced.sort();
    preview.missing_or_corrupt.sort();
    let stages = interrupted_stages(paths)?;
    preview.interrupted_imports = stages.iter().map(|s| s.name.clone()).collect();
    preview.temporary_bytes += stages.iter().map(|s| s.bytes).sum::<u64>();
    preview.fingerprint = identity_hash(&(facts, references, &stages, &preview))?;
    Ok((preview, stages))
}
#[derive(Serialize)]
struct InterruptedStage {
    name: String,
    bytes: u64,
    entries: Vec<(String, u64, Option<std::time::SystemTime>, bool)>,
}
fn interrupted_stages(paths: &AppPaths) -> Result<Vec<InterruptedStage>> {
    let mut stages = Vec::new();
    for e in fs::read_dir(&paths.data)? {
        let e = e?;
        let name = e.file_name().to_string_lossy().into_owned();
        if !name.starts_with(".import-staging-") {
            continue;
        }
        ensure!(
            fs::symlink_metadata(e.path())?.is_dir(),
            "Unsafe interrupted import staging path; no cleanup performed"
        );
        let marker = e.path().join(".owner-pid");
        ensure!(
            regular_file_or_absent(&marker)? && fs::metadata(&marker)?.len() <= 16,
            "Unowned import staging path; retained for manual recovery"
        );
        let pid: u32 = fs::read_to_string(marker)?.parse()?;
        if pid == 0 || crate::capture::owner_alive(pid) {
            continue;
        }
        let mut stage = InterruptedStage {
            name,
            bytes: 0,
            entries: vec![],
        };
        let mut pending = vec![(e.path(), 0)];
        while let Some((directory, depth)) = pending.pop() {
            ensure!(depth < 8, "Interrupted staging depth exceeds limit");
            let mut entries = fs::read_dir(directory)?.collect::<std::io::Result<Vec<_>>>()?;
            entries.sort_by_key(|e| e.file_name());
            for e in entries {
                let m = fs::symlink_metadata(e.path())?;
                ensure!(
                    m.is_file() || m.is_dir(),
                    "Unsafe symlink/special entry in interrupted import; retained"
                );
                let relative = e
                    .path()
                    .strip_prefix(paths.data.join(&stage.name))?
                    .to_str()
                    .context("Non-UTF8 staging path")?
                    .to_owned();
                stage
                    .entries
                    .push((relative, m.len(), m.modified().ok(), m.is_dir()));
                ensure!(
                    stage.entries.len() <= 40000,
                    "Interrupted import entry count exceeds cleanup limit"
                );
                if m.is_dir() {
                    pending.push((e.path(), depth + 1));
                } else {
                    stage.bytes = stage
                        .bytes
                        .checked_add(m.len())
                        .context("Staging size overflow")?;
                    ensure!(
                        stage.bytes <= 4 * crate::portability::ARCHIVE_BYTES,
                        "Interrupted import exceeds cleanup size limit; retained"
                    );
                }
            }
        }
        stages.push(stage);
    }
    stages.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(stages)
}
pub fn available_bytes(path: &Path) -> Result<u64> {
    #[cfg(unix)]
    {
        let path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes())?;
        let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        ensure!(
            unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) } == 0,
            "Cannot inspect free storage space"
        );
        let stat = unsafe { stat.assume_init() };
        Ok(
            (u128::from(stat.f_bavail) * u128::from(stat.f_frsize)).min(u128::from(u64::MAX))
                as u64,
        )
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        bail!("Free space inspection is unqualified on this OS")
    }
}
