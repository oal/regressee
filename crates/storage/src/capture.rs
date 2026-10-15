// SPDX-License-Identifier: MPL-2.0
use super::*;
use regressee_core::evidence::*;
const RECORD_LIMIT: usize = 64 * 1024 * 1024;
/// Non-cloneable optimistic claim for ONE running queue. No cached CAS authority:
/// load/activation/restore/export still verify files, and each new pin hashes bytes.
pub struct RunAppender {
    run: CaptureRun,
    revision: u64,
    work: CheckpointWork,
    store: uuid::Uuid,
}
#[derive(Debug, Clone, Copy, Default)]
pub struct CheckpointWork {
    pub target_validations: u64,
    pub evidence_serializations: u64,
    pub artifact_pins: u64,
    pub written_bytes: u64,
}
impl RunAppender {
    pub fn work(&self) -> CheckpointWork {
        self.work
    }
}
pub struct TargetCheckpoint<'a> {
    pub index: usize,
    pub target: &'a CaptureTarget,
    pub capture: Option<&'a CaptureEvidence>,
}
pub(crate) fn encode(value: &impl Serialize) -> Result<String> {
    let s = serde_json::to_string(value)?;
    ensure!(s.len() <= RECORD_LIMIT, "evidence record exceeds budget");
    Ok(s)
}
fn state(s: JobState) -> Result<String> {
    Ok(serde_json::to_string(&s)?)
}
impl Store {
    fn checkpoint_revision(&self, id: RunId) -> Result<u64> {
        Ok(self
            .connection
            .query_row(
                "SELECT revision FROM run_checkpoint_heads WHERE run_id=?1",
                [id.to_string()],
                |row| row.get(0),
            )
            .optional()?
            .unwrap_or(0))
    }
    pub fn begin_run_append(&mut self, id: RunId) -> Result<RunAppender> {
        let revision = self.checkpoint_revision(id)?;
        let record = self.load_run(id)?; // Explicit verified session boundary.
        ensure!(
            record.run.state == JobState::Running,
            "target append requires a running queue"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let owner: u32 = tx.query_row(
            "SELECT owner_pid FROM runs WHERE id=?1 AND state='\"Running\"'",
            [id.to_string()],
            |row| row.get(0),
        )?;
        ensure!(
            owner == std::process::id(),
            "queue belongs to another owner; explicit resume required"
        );
        tx.execute(
            "INSERT OR IGNORE INTO run_checkpoint_heads VALUES(?1,0)",
            [id.to_string()],
        )?;
        let current: u64 = tx.query_row(
            "SELECT revision FROM run_checkpoint_heads WHERE run_id=?1",
            [id.to_string()],
            |row| row.get(0),
        )?;
        ensure!(current == revision, "queue changed during append claim");
        tx.commit()?;
        Ok(RunAppender {
            run: record.run,
            revision,
            work: CheckpointWork::default(),
            store: self.instance,
        })
    }
    /// Validate and atomically persist ONLY explicitly changed target rows. This
    /// API cannot mutate profiles/inventory/queue settings or historical evidence.
    pub fn append_run_targets(
        &mut self,
        claim: &mut RunAppender,
        updates: &[TargetCheckpoint<'_>],
    ) -> Result<()> {
        ensure!(
            claim.store == self.instance,
            "append claim belongs to another Store owner"
        );
        ensure!(
            !updates.is_empty() && updates.len() <= claim.run.targets.len(),
            "invalid target delta count"
        );
        let mut indices = std::collections::BTreeSet::new();
        let mut encoded = Vec::with_capacity(updates.len());
        let mut work = CheckpointWork::default();
        for update in updates {
            let old = claim
                .run
                .targets
                .get(update.index)
                .context("target delta index out of range")?;
            let target = update.target;
            ensure!(
                indices.insert(update.index)
                    && old.key == target.key
                    && old.requested == target.requested,
                "duplicate or foreign target delta"
            );
            ensure!(
                target.attempts >= old.attempts
                    && target.attempts <= 8
                    && target.attempts <= old.attempts + 1
                    && target.warnings.len() <= 64
                    && target.warnings.iter().all(|warning| warning.len() <= 1024),
                "nonmonotonic or oversized target delta"
            );
            ensure!(
                matches!(old.outcome, TargetOutcome::Pending),
                "settled target/evidence is immutable"
            );
            if let Some(final_url) = &target.final_url {
                validate_address(final_url)?;
            }
            let new_capture = matches!(target.outcome, TargetOutcome::Captured { .. })
                && matches!(old.outcome, TargetOutcome::Pending);
            ensure!(
                new_capture == update.capture.is_some(),
                "delta outcome and new evidence disagree"
            );
            if let Some(capture) = update.capture {
                validate_evidence(&claim.run, target, capture)?;
                ensure!(target.attempts > 0, "successful delta was never attempted");
            }
            let target_json = encode(target)?;
            let capture_json = update.capture.map(encode).transpose()?;
            work.target_validations += 1;
            work.evidence_serializations += u64::from(capture_json.is_some());
            work.written_bytes +=
                (target_json.len() + capture_json.as_ref().map_or(0, String::len)) as u64;
            ensure!(
                target_json.len() + capture_json.as_ref().map_or(0, String::len) <= 1024 * 1024,
                "target delta exceeds budget"
            );
            encoded.push((update, target_json, capture_json));
        }
        let next = claim
            .revision
            .checked_add(1)
            .filter(|next| *next <= i64::MAX as u64)
            .context("checkpoint revision overflow")?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure!(tx.execute("UPDATE run_checkpoint_heads SET revision=?1 WHERE run_id=?2 AND revision=?3 AND EXISTS(SELECT 1 FROM runs WHERE id=?2 AND state='\"Running\"' AND owner_pid=?4)", params![next,claim.run.id.to_string(),claim.revision,std::process::id()])? == 1, "stale queue append claim");
        for (position, (update, target, capture)) in encoded.into_iter().enumerate() {
            if let Some(evidence) = update.capture {
                for artifact in evidence.image.artifacts() {
                    pin(
                        &tx,
                        &self.paths,
                        &ArtifactOwner::Run(claim.run.id),
                        &artifact,
                    )?;
                    work.artifact_pins += 1;
                }
            }
            let sequence = next
                .checked_mul(MAX_TARGETS as u64)
                .and_then(|value| value.checked_add(position as u64))
                .filter(|value| *value <= i64::MAX as u64)
                .context("target sequence overflow")?;
            tx.execute("INSERT INTO run_target_deltas VALUES(?1,?2,?3,?4,?5) ON CONFLICT(run_id,target_index) DO UPDATE SET sequence=excluded.sequence,target=excluded.target,capture=COALESCE(run_target_deltas.capture,excluded.capture)", params![claim.run.id.to_string(),update.index,sequence,target,capture])?;
        }
        tx.commit()?;
        for update in updates {
            claim.run.targets[update.index] = update.target.clone();
        }
        claim.revision = next;
        claim.work.target_validations += work.target_validations;
        claim.work.evidence_serializations += work.evidence_serializations;
        claim.work.artifact_pins += work.artifact_pins;
        claim.work.written_bytes += work.written_bytes;
        Ok(())
    }
    pub fn freeze_inventory(&mut self, inventory: &InventoryRevision) -> Result<()> {
        inventory.validate()?;
        self.connection.execute(
            "INSERT INTO inventories VALUES(?1,?2,?3)",
            params![
                inventory.id.to_string(),
                inventory.project.to_string(),
                encode(inventory)?
            ],
        )?;
        Ok(())
    }
    pub fn load_inventory(&self, id: InventoryId) -> Result<InventoryRevision> {
        let json: String = self.connection.query_row(
            "SELECT record FROM inventories WHERE id=?1",
            [id.to_string()],
            |row| row.get(0),
        )?;
        ensure!(
            json.len() <= RECORD_LIMIT,
            "inventory record exceeds budget"
        );
        let inventory: InventoryRevision = serde_json::from_str(&json)?;
        inventory.validate()?;
        ensure!(inventory.id == id, "inventory identity mismatch");
        Ok(inventory)
    }
    pub fn artifact_path(&self, id: &ArtifactId) -> Result<PathBuf> {
        self.verify_artifact(id)?;
        Ok(self.paths.artifacts().join(id.as_str()))
    }
    pub fn paths(&self) -> &AppPaths {
        &self.paths
    }
    pub fn begin_run(&mut self, record: &RunRecord) -> Result<()> {
        record.run.validate()?;
        if matches!(record.run.intent, RunIntent::Check { .. }) {
            self.validate_check_coverage(&record.run)?;
        }
        ensure!(
            record.run.state == JobState::Running
                && record.capture.is_none()
                && record.run.started_at.is_some()
                && record.run.finished_at.is_none(),
            "new run must be running without evidence"
        );
        ensure!(
            record.captures.is_empty(),
            "new queue must not contain evidence"
        );
        self.connection.execute(
            "INSERT INTO runs VALUES(?1,?2,?3,?4,unixepoch(),?5)",
            params![
                record.run.id.to_string(),
                record.run.project.to_string(),
                state(record.run.state)?,
                encode(record)?,
                std::process::id()
            ],
        )?;
        Ok(())
    }
    fn validate_check_coverage(&self, run: &CaptureRun) -> Result<()> {
        if let RunIntent::Check { baseline } = run.intent {
            let baseline = self.load_baseline(baseline)?;
            ensure!(
                baseline.project == run.project
                    && baseline.inventory == run.inventory.id
                    && run
                        .targets
                        .iter()
                        .map(|t| &t.key)
                        .collect::<std::collections::BTreeSet<_>>()
                        == baseline.references.iter().map(|r| &r.key).collect(),
                "check must use exactly the saved baseline screenshots"
            );
        }
        Ok(())
    }
    pub fn load_run(&self, id: RunId) -> Result<RunRecord> {
        let record = self.load_run_snapshot(id)?;
        let mut verified = std::collections::BTreeSet::new();
        for c in record.all_captures() {
            for a in c.image.artifacts() {
                if verified.insert(a.clone()) {
                    self.verify_artifact(&a)?;
                }
            }
        }
        Ok(record)
    }
    pub(super) fn load_run_snapshot(&self, id: RunId) -> Result<RunRecord> {
        let revision = self.checkpoint_revision(id)?;
        let s: String = self.connection.query_row(
            "SELECT record FROM runs WHERE id=?1 AND length(CAST(record AS BLOB))<=?2",
            params![id.to_string(), RECORD_LIMIT],
            |r| r.get(0),
        )?;
        let mut record: RunRecord = serde_json::from_str(&s)?;
        let mut deltas = self.connection.prepare("SELECT target_index,target,capture FROM run_target_deltas WHERE run_id=?1 ORDER BY sequence,target_index")?;
        let rows = deltas.query_map([id.to_string()], |row| {
            Ok((
                row.get::<_, usize>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, Option<String>>(2)?,
            ))
        })?;
        for row in rows {
            let (index, target, capture) = row?;
            ensure!(
                target.len() + capture.as_ref().map_or(0, String::len) <= 1024 * 1024,
                "stored target delta exceeds budget"
            );
            let next: CaptureTarget = serde_json::from_str(&target)?;
            let old = record
                .run
                .targets
                .get_mut(index)
                .context("stored target delta out of range")?;
            ensure!(
                old.key == next.key && old.requested == next.requested,
                "stored target delta changed frozen identity"
            );
            *old = next;
            if let Some(capture) = capture {
                record.captures.push(serde_json::from_str(&capture)?);
            }
        }
        record.run.validate()?;
        ensure!(record.run.id == id, "stored run ID mismatch");
        validate_capture(&record)?;
        ensure!(
            self.checkpoint_revision(id)? == revision,
            "queue changed during snapshot read"
        );
        Ok(record)
    }
    pub fn finish_run(&mut self, record: &RunRecord) -> Result<()> {
        ensure!(record.run.state.terminal(), "run is not terminal");
        self.checkpoint_run(record)
    }
    /// Publish safe target boundaries, pinning evidence with the queue update.
    pub fn checkpoint_run(&mut self, record: &RunRecord) -> Result<()> {
        self.checkpoint_inner(record, false)
    }
    pub fn resume_run(&mut self, record: &RunRecord) -> Result<()> {
        self.checkpoint_inner(record, true)
    }
    fn checkpoint_inner(&mut self, record: &RunRecord, resuming: bool) -> Result<()> {
        let previous_revision = self.checkpoint_revision(record.run.id)?;
        let previous_json: String = self.connection.query_row(
            "SELECT record FROM runs WHERE id=?1",
            [record.run.id.to_string()],
            |row| row.get(0),
        )?;
        record.run.validate()?;
        validate_capture(record)?;
        // Explicit compaction boundary: rehash ALL persisted originals and
        // derivatives, including evidence already pinned by target appends.
        // Never cache this authority or move this scan into per-target appends.
        let previous = self.load_run(record.run.id)?;
        ensure!(
            !resuming || matches!(previous.run.state, JobState::Paused | JobState::Interrupted),
            "Queue was already resumed by another owner"
        );
        ensure!(
            resuming
                || !matches!(previous.run.state, JobState::Paused | JobState::Interrupted)
                || record.run.state != JobState::Running,
            "Explicit validated resume is required"
        );
        ensure!(
            (!previous.run.state.terminal() || previous.run.state == JobState::Interrupted)
                && previous.run.inventory == record.run.inventory
                && previous.run.profiles == record.run.profiles
                && previous.run.project == record.run.project
                && previous.run.intent == record.run.intent
                && previous.run.environment == record.run.environment
                && previous.run.rendering == record.run.rendering
                && previous.run.started_at == record.run.started_at
                && previous
                    .run
                    .targets
                    .iter()
                    .zip(&record.run.targets)
                    .all(|(a, b)| a.key == b.key && a.requested == b.requested)
                && previous.run.targets.len() == record.run.targets.len(),
            "immutable run snapshot changed"
        );
        if previous.run.state == JobState::Interrupted {
            ensure!(
                record.run.state == JobState::Running && record.run.finished_at.is_none(),
                "Interrupted queues require explicit validated resume"
            );
        }
        for (old, new) in previous.run.targets.iter().zip(&record.run.targets) {
            ensure!(
                new.attempts >= old.attempts
                    && (!matches!(old.outcome, TargetOutcome::Captured { .. }) || old == new),
                "successful targets and attempt accounting are immutable"
            );
        }
        let new_captures: std::collections::BTreeMap<_, _> = record
            .all_captures()
            .map(|capture| (&capture.metadata.key, capture))
            .collect();
        let previous_keys: std::collections::BTreeSet<_> = previous
            .all_captures()
            .map(|capture| &capture.metadata.key)
            .collect();
        for c in previous.all_captures() {
            ensure!(new_captures.get(&c.metadata.key).is_some_and(|next| serde_json::to_vec(next).ok() == serde_json::to_vec(c).ok()), "previous successful evidence cannot change on resume");
        }
        if let Some(old) = &previous.queue {
            let new = record
                .queue
                .as_ref()
                .context("frozen queue settings missing")?;
            ensure!(
                old.discovery == new.discovery
                    && old.project_revision == new.project_revision
                    && new.resumed_at.starts_with(&old.resumed_at)
                    && new.resumed_at.len() <= 100,
                "immutable queue settings changed"
            );
            if old.resumed_at.len() != new.resumed_at.len() {
                ensure!(
                    matches!(previous.run.state, JobState::Paused | JobState::Interrupted)
                        && record.run.state == JobState::Running
                        && new.resumed_at.len() == old.resumed_at.len() + 1,
                    "Resume must claim a paused or interrupted queue exactly once"
                );
            }
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT OR IGNORE INTO run_checkpoint_heads VALUES(?1,0)",
            [record.run.id.to_string()],
        )?;
        ensure!(tx.execute("UPDATE run_checkpoint_heads SET revision=revision+1 WHERE run_id=?1 AND revision=?2", params![record.run.id.to_string(),previous_revision])? == 1, "queue changed before snapshot compaction");
        for c in record.all_captures() {
            if previous_keys.contains(&c.metadata.key) {
                continue;
            }
            for a in c.image.artifacts() {
                pin(&tx, &self.paths, &ArtifactOwner::Run(record.run.id), &a)?;
            }
        }
        ensure!(
            tx.execute(
                "UPDATE runs SET state=?1,record=?2,owner_pid=?5 WHERE id=?3 AND state=?4 AND record=?6",
                params![
                    state(record.run.state)?,
                    encode(record)?,
                    record.run.id.to_string(),
                    state(previous.run.state)?
                    ,std::process::id(),previous_json
                ]
            )? == 1,
            "run already finished"
        );
        tx.execute(
            "DELETE FROM run_target_deltas WHERE run_id=?1",
            [record.run.id.to_string()],
        )?;
        tx.commit()?;
        Ok(())
    }
    pub fn activate_baseline(
        &mut self,
        run: RunId,
        expected_revision: u64,
    ) -> Result<ProjectRecord> {
        self.activate_baseline_with_partial_confirmation(run, expected_revision, false)
    }

    /// `confirm_partial` must come from explicit consent to skip unsuccessful
    /// targets. The original intended coverage and failures remain immutable.
    pub fn activate_baseline_with_partial_confirmation(
        &mut self,
        run: RunId,
        expected_revision: u64,
        confirm_partial: bool,
    ) -> Result<ProjectRecord> {
        let record = self.load_run(run)?;
        ensure!(
            !self.is_imported_run(run)?,
            "Imported origins are portable provenance, not a new independently captured baseline. Set the imported version active or make a fresh capture."
        );
        let (captured, intended) = record
            .baseline_save_counts()
            .context("only a finished baseline capture with successful screenshots can be saved")?;
        let partial = captured < intended;
        ensure!(
            !partial || confirm_partial,
            "saving partial coverage requires explicit confirmation"
        );
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let mut originals = std::collections::BTreeSet::new();
        for capture in record.all_captures() {
            if originals.insert(&capture.image.original) {
                let artifact = self.verify_artifact(&capture.image.original)?;
                ensure!(
                    artifact.kind == ArtifactKind::Screenshot
                        && artifact.bytes == capture.metadata.bytes,
                    "Original screenshot metadata conflicts"
                );
                ensure!(
                    crate::images::validate_png(&self.artifact_path(&artifact.id)?, &cancel)?
                        == (capture.metadata.width, capture.metadata.height),
                    "Original screenshot PNG dimensions conflict"
                );
            }
        }
        let mut project = self.load_project(record.run.project)?;
        ensure!(
            project.revision == expected_revision
                && record
                    .queue
                    .as_ref()
                    .is_none_or(|q| q.project_revision == expected_revision)
                && (record.queue.is_some()
                    || project.project.sources[0] == record.run.targets[0].requested),
            "project changed; capture again before activation"
        );
        let baseline = BaselineVersion {
            id: BaselineId::new(),
            project: record.run.project,
            parent: project.project.active_baseline,
            inventory: record.run.inventory.id,
            created_at: crate::capture::now(),
            references: record
                .all_captures()
                .map(|capture| BaselineReference {
                    key: capture.metadata.key.clone(),
                    run,
                    artifact: capture.image.original.clone(),
                    captured_at: record.run.finished_at.unwrap(),
                })
                .collect(),
            intended_count: record.run.targets.len() as u32,
            explicitly_partial: partial,
        };
        baseline.validate()?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure!(
            tx.query_row(
                "SELECT revision FROM projects WHERE id=?1",
                [project.project.id.to_string()],
                |r| r.get::<_, u64>(0)
            )? == expected_revision,
            "project changed before baseline activation"
        );
        tx.execute(
            "INSERT INTO baselines VALUES(?1,?2,?3,?4)",
            params![
                baseline.id.to_string(),
                baseline.project.to_string(),
                run.to_string(),
                encode(&baseline)?
            ],
        )?;
        for capture in record.all_captures() {
            for a in capture.image.artifacts() {
                pin(&tx, &self.paths, &ArtifactOwner::Baseline(baseline.id), &a)?;
            }
        }
        project.project.active_baseline = Some(baseline.id);
        project.project.profiles = record.run.profiles.clone();
        project.revision += 1;
        tx.execute(
            "UPDATE projects SET config=?1,revision=?2,updated_at=unixepoch() WHERE id=?3",
            params![
                serde_json::to_string(&project.project)?,
                project.revision,
                project.project.id.to_string()
            ],
        )?;
        tx.commit()?;
        Ok(project)
    }
    pub fn load_baseline(&self, id: BaselineId) -> Result<BaselineVersion> {
        let s: String = self.connection.query_row(
            "SELECT record FROM baselines WHERE id=?1",
            [id.to_string()],
            |r| r.get(0),
        )?;
        ensure!(s.len() <= RECORD_LIMIT, "baseline too large");
        let b: BaselineVersion = serde_json::from_str(&s)?;
        b.validate()?;
        ensure!(b.id == id, "baseline ID mismatch");
        let mut runs = std::collections::BTreeMap::new();
        for r in &b.references {
            if let std::collections::btree_map::Entry::Vacant(e) = runs.entry(r.run) {
                e.insert(self.load_run(r.run)?);
            }
            let run = &runs[&r.run];
            ensure!(
                run.run.project == b.project
                    && r.captured_at
                        == run
                            .run
                            .finished_at
                            .context("baseline source date missing")?
                    && b.inventory == run.run.inventory.id
                    && run
                        .evidence(&r.key)
                        .is_some_and(|c| c.image.original == r.artifact && c.metadata.key == r.key),
                "baseline source mismatch"
            );
        }
        Ok(b)
    }
    pub fn save_analysis(&mut self, a: &ComparisonEvidence) -> Result<()> {
        ensure!(
            (a.revision.algorithm == "rgba-max-channel-v1"
                || (a.revision.algorithm == regressee_core::analysis_engine::ALGORITHM_ID
                    && a.perceptual.is_some()))
                && !a.revision.results.is_empty()
                && a.revision.mask.is_none(),
            "unsupported initial analysis"
        );
        a.revision.policy.validate()?;
        for result in &a.revision.results {
            result.validate()?;
        }
        let run = self.load_run(a.revision.run)?;
        let baseline = self.load_baseline(a.revision.baseline)?;
        if let Some(p) = &a.perceptual {
            use regressee_core::analysis_engine as engine;
            ensure!(
                a.revision.algorithm == engine::ALGORITHM_ID,
                "perceptual algorithm mismatch"
            );
            p.policy.validate()?;
            ensure!(
                p.cache_key.len() == 64
                    && p.created_at > 0
                    && p.reports.len() + p.failures.len() <= run.run.targets.len(),
                "invalid perceptual revision"
            );
            let mut keys = std::collections::BTreeSet::new();
            for (key, report) in &p.reports {
                ensure!(
                    keys.insert(key)
                        && report.schema_version == engine::SCHEMA_VERSION
                        && report.request.policy == p.policy,
                    "duplicate/foreign report policy"
                );
                report.request.validate()?;
                let source = baseline
                    .references
                    .iter()
                    .find(|r| &r.key == key)
                    .context("report source missing")?;
                let before = self.load_run(source.run)?;
                ensure!(
                    report.request.before
                        == descriptor(&before, before.evidence(key).context("before missing")?)?
                        && report.request.after
                            == descriptor(&run, run.evidence(key).context("after missing")?)?,
                    "report capture binding mismatch"
                );
                ensure!(
                    report.changed_pixels <= report.eligible_pixels
                        && report.eligible_pixels + report.ignored_pixels
                            == report.coordinate_union_pixels
                        && report.changed_fraction.numerator == report.changed_pixels
                        && report.changed_fraction.denominator == report.eligible_pixels,
                    "report coverage mismatch"
                );
                if report.eligible_pixels == 0 {
                    ensure!(
                        report.classification == engine::Classification::NotComparable
                            && report.gates.contains(&engine::GateReason::AllMasked),
                        "all-masked cannot pass"
                    );
                }
            }
            for (key, reason) in &p.failures {
                ensure!(
                    keys.insert(key) && reason.len() <= 2048,
                    "duplicate/oversized failure"
                );
            }
            for result in &a.revision.results {
                if run.evidence(&result.key).is_some() {
                    let report = p
                        .reports
                        .iter()
                        .find(|(k, _)| *k == result.key)
                        .map(|(_, r)| r);
                    let failure = p.failures.iter().any(|(k, _)| *k == result.key);
                    ensure!(
                        report.is_some() != failure,
                        "successful capture needs exactly one report or analysis failure"
                    );
                    if let Some(report) = report {
                        ensure!(
                            result.changed_pixels == report.changed_pixels
                                && result.eligible_pixels == report.eligible_pixels,
                            "summary/report metrics differ"
                        );
                        if result.classification == Classification::NoSignificantChange
                            || result.classification == Classification::Changed
                        {
                            ensure!(
                                report.gates.is_empty()
                                    && !report
                                        .warnings
                                        .contains(&engine::Warning::RenderingEnvironmentMismatch),
                                "gated/caution evidence cannot be clean"
                            );
                            ensure!(
                                matches!(
                                    (result.classification, report.classification),
                                    (
                                        Classification::NoSignificantChange,
                                        engine::Classification::NoSignificantChange
                                    ) | (Classification::Changed, engine::Classification::Changed)
                                ),
                                "summary classification contradicts report"
                            );
                        }
                    } else {
                        ensure!(
                            result.classification == Classification::NotComparable
                                && result.eligible_pixels == 0
                                && result.changed_pixels == 0,
                            "analysis failure cannot be unchanged"
                        );
                    }
                } else {
                    ensure!(
                        !keys.contains(&result.key),
                        "missing capture cannot have analysis pixels"
                    );
                }
            }
        }
        for result in &a.revision.results {
            if run.evidence(&result.key).is_none() {
                ensure!(
                    result.classification == Classification::CouldNotCapture
                        && result.eligible_pixels == 0
                        && result.changed_pixels == 0
                        && !result.dimensions_changed,
                    "A failed or missing target cannot be unchanged or have compared pixels"
                );
            }
            ensure!(
                baseline.references.iter().any(|r| r.key == result.key),
                "Analysis contains a target not saved in the baseline"
            );
        }
        ensure!(
            run.run.intent
                == RunIntent::Check {
                    baseline: baseline.id
                }
                && run.run.state.terminal()
                && a.revision.results.len() == run.run.targets.len()
                && a.revision
                    .results
                    .iter()
                    .map(|r| &r.key)
                    .collect::<std::collections::BTreeSet<_>>()
                    == run.run.targets.iter().map(|r| &r.key).collect(),
            "analysis run/baseline mismatch"
        );
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        if let Some(h) = &a.heatmap {
            for id in h.artifacts() {
                pin(
                    &tx,
                    &self.paths,
                    &ArtifactOwner::Analysis(a.revision.id),
                    &id,
                )?;
            }
        }
        for (key, h) in &a.heatmaps {
            ensure!(
                a.revision.results.iter().any(|r| &r.key == key
                    && (r.eligible_pixels > 0
                        || (r.classification == Classification::NotComparable
                            && a.perceptual
                                .as_ref()
                                .is_some_and(|p| p.reports.iter().any(|(k, _)| k == key))))),
                "heatmap has no compared target"
            );
            for id in h.artifacts() {
                pin(
                    &tx,
                    &self.paths,
                    &ArtifactOwner::Analysis(a.revision.id),
                    &id,
                )?;
            }
        }
        tx.execute(
            "INSERT INTO analyses VALUES(?1,?2,?3,?4)",
            params![
                a.revision.id.to_string(),
                a.revision.run.to_string(),
                baseline.id.to_string(),
                encode(a)?
            ],
        )?;
        tx.execute("INSERT INTO analysis_display(run_id,analysis_id) VALUES(?1,?2) ON CONFLICT(run_id) DO UPDATE SET analysis_id=excluded.analysis_id,revision=analysis_display.revision+1",params![a.revision.run.to_string(),a.revision.id.to_string()])?;
        tx.commit()?;
        Ok(())
    }
    pub fn project_evidence(&self, id: ProjectId) -> Result<ProjectEvidence> {
        let project = self.load_project(id)?;
        let latest: Option<String> = self
            .connection
            .query_row(
                "SELECT id FROM runs WHERE project_id=?1 ORDER BY rowid DESC LIMIT 1",
                [id.to_string()],
                |r| r.get(0),
            )
            .optional()?;
        let latest = latest
            .map(|s| {
                serde_json::from_value(serde_json::json!(s))
                    .and_then(|id| self.load_run(id).map_err(serde::de::Error::custom))
            })
            .transpose()?;
        let baseline = project
            .project
            .active_baseline
            .map(|id| self.load_baseline(id))
            .transpose()?;
        let before = baseline
            .as_ref()
            .map(|b| {
                self.load_run(b.references[0].run)
                    .map(|r| r.evidence(&b.references[0].key).cloned())
            })
            .transpose()?
            .flatten();
        let comparison = if let Some(r) = &latest {
            let s: Option<String> = self
                .connection
                .query_row(
                    "SELECT record FROM analyses WHERE id=COALESCE((SELECT analysis_id FROM analysis_display WHERE run_id=?1),(SELECT id FROM analyses WHERE run_id=?1 ORDER BY rowid DESC LIMIT 1))",
                    [r.run.id.to_string()],
                    |r| r.get(0),
                )
                .optional()?;
            s.map(|s| {
                ensure!(s.len() <= RECORD_LIMIT, "analysis too large");
                let a: ComparisonEvidence = serde_json::from_str(&s)?;
                let a = self.load_analysis(a.revision.id)?;
                if let Some(h) = &a.heatmap {
                    for id in h.artifacts() {
                        self.verify_artifact(&id)?;
                    }
                }
                for (_, h) in &a.heatmaps {
                    for id in h.artifacts() {
                        self.verify_artifact(&id)?;
                    }
                }
                Ok::<_, anyhow::Error>(a)
            })
            .transpose()?
        } else {
            None
        };
        Ok(ProjectEvidence {
            analyses: if let Some(r) = &latest {
                let mut q = self.connection.prepare(
                    "SELECT id FROM analyses WHERE run_id=?1 ORDER BY rowid DESC LIMIT 128",
                )?;
                q.query_map([r.run.id.to_string()], |r| r.get::<_, String>(0))?
                    .map(|s| Ok(serde_json::from_value(serde_json::json!(s?))?))
                    .collect::<Result<_>>()?
            } else {
                vec![]
            },
            reviews: if let Some(r) = &latest {
                self.review_history(r.run.id)?
            } else {
                vec![]
            },
            baselines: self.baseline_history(id)?,
            latest,
            baseline,
            before,
            comparison,
        })
    }
    /// Verified, focused viewer session boundary. Unlike the full workspace
    /// snapshot this does not load every retained baseline or review decision.
    /// Warm tiles reuse this metadata; all authoritative APIs remain uncached.
    pub fn viewer_evidence(
        &self,
        project: ProjectId,
        analysis: Option<AnalysisId>,
    ) -> Result<ProjectEvidence> {
        let saved = self.load_project(project)?;
        let comparison = if let Some(id) = analysis {
            Some(self.load_analysis(id)?)
        } else {
            let id: Option<String> = self.connection.query_row(
                "SELECT COALESCE((SELECT analysis_id FROM analysis_display WHERE run_id=r.id),(SELECT id FROM analyses WHERE run_id=r.id ORDER BY rowid DESC LIMIT 1)) FROM runs r WHERE project_id=?1 ORDER BY rowid DESC LIMIT 1",
                [project.to_string()], |row| row.get::<_, Option<String>>(0),
            ).optional()?.flatten();
            id.map(|id| self.load_analysis(serde_json::from_value(serde_json::json!(id))?))
                .transpose()?
        };
        let run = if let Some(a) = &comparison {
            Some(a.revision.run)
        } else {
            let id: Option<String> = self
                .connection
                .query_row(
                    "SELECT id FROM runs WHERE project_id=?1 ORDER BY rowid DESC LIMIT 1",
                    [project.to_string()],
                    |row| row.get(0),
                )
                .optional()?;
            id.map(|id| serde_json::from_value(serde_json::json!(id)))
                .transpose()?
        };
        let latest = run.map(|id| self.load_run(id)).transpose()?;
        ensure!(
            latest.as_ref().is_none_or(|r| r.run.project == project),
            "viewer analysis belongs to another project"
        );
        let baseline = comparison
            .as_ref()
            .map(|a| a.revision.baseline)
            .or(saved.project.active_baseline)
            .map(|id| self.load_baseline(id))
            .transpose()?;
        ensure!(
            baseline.as_ref().is_none_or(|b| b.project == project),
            "viewer baseline belongs to another project"
        );
        Ok(ProjectEvidence {
            latest,
            baseline,
            before: None,
            comparison,
            analyses: vec![],
            reviews: vec![],
            baselines: vec![],
        })
    }
    /// Called by the exclusive application storage owner on startup; never GC.
    pub fn recover_runs(&mut self) -> Result<u32> {
        let ids: Vec<(String, u32)> = {
            let mut s=self.connection.prepare("SELECT id,owner_pid FROM runs WHERE state IN ('\"Running\"','\"Pausing\"','\"Queued\"')")?;
            s.query_map([], |r| Ok((r.get(0)?, r.get(1)?)))?
                .collect::<rusqlite::Result<_>>()?
        };
        let mut count = 0;
        for (id, pid) in ids {
            if owner_alive(pid) {
                continue;
            }
            let id = serde_json::from_value(serde_json::json!(id))?;
            let mut r = self.load_run(id)?;
            r.run.state = JobState::Interrupted;
            r.run.finished_at = Some(now());
            self.finish_run(&r)?;
            count += 1;
        }
        Ok(count)
    }
}
pub(crate) fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
pub(crate) fn owner_alive(pid: u32) -> bool {
    #[cfg(target_os = "linux")]
    {
        std::path::Path::new(&format!("/proc/{pid}")).exists()
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = pid;
        true
    } // fail closed until OS ownership is qualified
}
pub(crate) fn pin(
    tx: &rusqlite::Transaction<'_>,
    paths: &AppPaths,
    owner: &ArtifactOwner,
    id: &ArtifactId,
) -> Result<()> {
    let bytes = tx.query_row(
        "SELECT bytes FROM artifacts WHERE hash=?1",
        [id.as_str()],
        |r| r.get(0),
    )?;
    verify_file(&paths.artifacts().join(id.as_str()), id, bytes)?;
    let (kind, owner) = owner_parts(owner);
    tx.execute(
        "INSERT OR IGNORE INTO artifact_refs VALUES(?1,?2,?3)",
        params![kind, owner, id.as_str()],
    )?;
    Ok(())
}
pub(crate) fn validate_capture(r: &RunRecord) -> Result<()> {
    let targets: std::collections::BTreeSet<_> =
        r.run.targets.iter().map(|target| &target.key).collect();
    let mut captures = std::collections::BTreeMap::new();
    for c in r.all_captures() {
        ensure!(
            captures.insert(&c.metadata.key, c).is_none() && targets.contains(&c.metadata.key),
            "duplicate or foreign evidence"
        );
    }
    for target in &r.run.targets {
        match (&target.outcome, captures.get(&target.key)) {
            (
                TargetOutcome::Captured {
                    artifact,
                    width,
                    height,
                },
                Some(c),
            ) => {
                let _ = (artifact, width, height);
                validate_evidence(&r.run, target, c)?;
            }
            (TargetOutcome::Captured { .. }, None) | (_, Some(_)) => {
                bail!("capture outcome and evidence disagree")
            }
            _ => (),
        }
    }
    Ok(())
}
fn validate_evidence(run: &CaptureRun, target: &CaptureTarget, c: &CaptureEvidence) -> Result<()> {
    let TargetOutcome::Captured {
        artifact,
        width,
        height,
    } = &target.outcome
    else {
        bail!("evidence requires a captured target");
    };
    let m = &c.metadata;
    ensure!(
        m.version == 1
            && m.key == target.key
            && run.profiles.contains(&m.profile)
            && m.runtime == m.profile.runtime
            && m.requested
                == regressee_core::environment::map_url(&target.requested, &run.environment)?
                    .as_str()
            && Some(m.final_url.as_str()) == target.final_url.as_ref().map(|url| url.as_str())
            && m.sha256 == *artifact
            && c.image.original == *artifact
            && m.width == *width
            && m.height == *height
            && c.image.width == *width
            && c.image.height == *height
            && *width > 0
            && *width <= 32768
            && *height > 0
            && *height <= 100_000
            && c.image.tiles.len() == (*height as usize).div_ceil(512),
        "capture metadata or derivatives mismatch"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn checkpoint_fixture(count: usize) -> (tempfile::TempDir, Store, RunRecord, CaptureEvidence) {
        let root = tempfile::tempdir().unwrap();
        let mut store = Store::open(AppPaths::isolated(root.path())).unwrap();
        let project = Project::draft(
            "Bounded checkpoint fixture".into(),
            "http://127.0.0.1:1/fixture",
        )
        .unwrap();
        store.save_project(&project, None).unwrap();
        let mut profile = project.profiles[0].clone();
        profile.width = 1;
        profile.height = 1;
        profile.extent = CaptureExtent::VisibleScreen;
        profile.runtime = RuntimeIdentity::Pinned {
            node: "fixture".into(),
            worker: "fixture".into(),
            playwright: "fixture".into(),
            browser: "fixture".into(),
        };
        let pages: Vec<_> = (0..count)
            .map(|i| InventoryPage {
                id: PageId::new(),
                canonical: format!("http://127.0.0.1:1/fixture/{i}").parse().unwrap(),
                included: true,
                reason: None,
            })
            .collect();
        let targets: Vec<_> = pages
            .iter()
            .map(|p| CaptureTarget {
                key: TargetKey {
                    page: p.id,
                    profile: profile.id,
                },
                requested: p.canonical.clone(),
                final_url: None,
                attempts: 0,
                outcome: TargetOutcome::Pending,
                warnings: vec![],
            })
            .collect();
        let artifact = store
            .put_artifact(
                &b"synthetic CAS accounting fixture (not browser coverage)"[..],
                ArtifactKind::Screenshot,
            )
            .unwrap();
        let sample = CaptureEvidence {
            metadata: CaptureMetadata {
                version: 1,
                key: targets[0].key.clone(),
                requested: targets[0].requested.to_string(),
                final_url: targets[0].requested.to_string(),
                runtime: profile.runtime.clone(),
                profile: profile.clone(),
                width: 1,
                height: 1,
                sha256: artifact.id.clone(),
                bytes: artifact.bytes,
                warnings: vec![],
                stable: true,
                duration_ms: 1,
                readiness: Readiness {
                    font_failures: 0,
                    image_failures: 0,
                    resource_failures: 0,
                },
                capture_mode: "synthetic-checkpoint-fixture".into(),
                replacement: None,
            },
            image: ImageEvidence {
                original: artifact.id.clone(),
                thumbnail: artifact.id.clone(),
                width: 1,
                height: 1,
                tiles: vec![artifact.id],
            },
        };
        let record = RunRecord {
            run: CaptureRun {
                id: RunId::new(),
                project: project.id,
                intent: RunIntent::NewBaseline,
                inventory: InventoryRevision {
                    id: InventoryId::new(),
                    project: project.id,
                    identity_version: 2,
                    pages,
                    coverage: DiscoveryCoverage::Complete,
                },
                profiles: vec![profile],
                environment: project.environments[0].clone(),
                rendering: RenderingEnvironment {
                    os: "fixture".into(),
                    architecture: "fixture".into(),
                    browser: "fixture".into(),
                    worker_version: "fixture".into(),
                    profile_version: 1,
                    fonts_fingerprint: None,
                },
                state: JobState::Running,
                started_at: Some(1),
                finished_at: None,
                targets,
            },
            capture: None,
            captures: vec![],
            queue: Some(QueueSettings {
                discovery: project.discovery,
                project_revision: 1,
                resumed_at: vec![],
            }),
        };
        store.freeze_inventory(&record.run.inventory).unwrap();
        store.begin_run(&record).unwrap();
        (root, store, record, sample)
    }
    fn add_capture(record: &mut RunRecord, sample: &CaptureEvidence, index: usize) {
        let target = &mut record.run.targets[index];
        target.attempts = 1;
        target.final_url = Some(target.requested.clone());
        target.outcome = TargetOutcome::Captured {
            artifact: sample.image.original.clone(),
            width: 1,
            height: 1,
        };
        let mut capture = sample.clone();
        capture.metadata.key = target.key.clone();
        capture.metadata.requested = target.requested.to_string();
        capture.metadata.final_url = target.requested.to_string();
        record.captures.push(capture);
    }
    #[test]
    fn viewer_revision_tracks_target_checkpoints_and_compaction() {
        let (_root, mut store, mut record, sample) = checkpoint_fixture(1);
        let mut claim = store.begin_run_append(record.run.id).unwrap();
        let first = store.viewer_revision().unwrap();
        add_capture(&mut record, &sample, 0);
        store
            .append_run_targets(
                &mut claim,
                &[TargetCheckpoint {
                    index: 0,
                    target: &record.run.targets[0],
                    capture: record.captures.last(),
                }],
            )
            .unwrap();
        let append = store.viewer_revision().unwrap();
        assert_ne!(first, append);
        record.run.state = JobState::Completed;
        record.run.finished_at = Some(2);
        store.finish_run(&record).unwrap();
        assert_ne!(append, store.viewer_revision().unwrap());
    }
    #[test]
    #[ignore = "bounded serial checkpoint profile, synthetic CAS fixture not browser coverage"]
    fn bounded_checkpoint_profile() {
        for count in [100, 200, 400] {
            let (_root, mut store, mut record, sample) = checkpoint_fixture(count);
            let io = || std::fs::read_to_string("/proc/self/io").unwrap_or_default();
            let before = io();
            let start = std::time::Instant::now();
            for i in 0..count {
                add_capture(&mut record, &sample, i);
                store.checkpoint_run(&record).unwrap();
            }
            assert_eq!(store.load_run(record.run.id).unwrap().captures.len(), count);
            println!(
                "{}",
                serde_json::json!({"scope":"synthetic normal checkpoint evidence, no wall-time acceptance","targets":count,"seconds":start.elapsed().as_secs_f64(),"io_before":before,"io_after":io()})
            );
        }
    }
    #[test]
    #[ignore = "bounded serial delta checkpoint profile, not browser coverage"]
    fn bounded_delta_checkpoint_profile() {
        for count in [100, 200, 400] {
            let (_root, mut store, mut record, sample) = checkpoint_fixture(count);
            let mut claim = store.begin_run_append(record.run.id).unwrap();
            let io = || std::fs::read_to_string("/proc/self/io").unwrap_or_default();
            let before = io();
            let start = std::time::Instant::now();
            for i in 0..count {
                add_capture(&mut record, &sample, i);
                store
                    .append_run_targets(
                        &mut claim,
                        &[TargetCheckpoint {
                            index: i,
                            target: &record.run.targets[i],
                            capture: record.captures.last(),
                        }],
                    )
                    .unwrap();
            }
            assert_eq!(
                serde_json::to_vec(&store.load_run(record.run.id).unwrap()).unwrap(),
                serde_json::to_vec(&record).unwrap()
            );
            let (rows, bytes): (u64,u64) = store.connection.query_row("SELECT count(*),sum(length(CAST(target AS BLOB))+length(CAST(capture AS BLOB))) FROM run_target_deltas WHERE run_id=?1", [record.run.id.to_string()], |row| Ok((row.get(0)?,row.get(1)?))).unwrap();
            let work = claim.work();
            assert_eq!(
                (
                    rows,
                    work.target_validations,
                    work.evidence_serializations,
                    work.artifact_pins
                ),
                (count as u64, count as u64, count as u64, 3 * count as u64)
            );
            assert_eq!(bytes, work.written_bytes);
            assert!(bytes < count as u64 * 4096);
            println!(
                "{}",
                serde_json::json!({"scope":"synthetic target deltas; no wall-time acceptance","targets":count,"seconds":start.elapsed().as_secs_f64(),"io_before":before,"io_after":io(),"target_validations":work.target_validations,"evidence_serializations":work.evidence_serializations,"artifact_pins":work.artifact_pins,"persisted_delta_rows":rows,"serialized_bytes":bytes})
            );
        }
    }
    #[test]
    fn compaction_rehashes_old_original_and_derivatives_atomically() {
        for boundary in ["pause", "resume", "finish", "cancel"] {
            for role in ["original", "thumbnail", "tile"] {
                for damage in ["same-size", "missing", "replaced", "symlink"] {
                    let (root, mut store, mut record, mut sample) = checkpoint_fixture(1);
                    let thumbnail = store
                        .put_artifact(&b"independent thumbnail"[..], ArtifactKind::Thumbnail)
                        .unwrap();
                    let tile = store
                        .put_artifact(&b"independent tile"[..], ArtifactKind::Tile)
                        .unwrap();
                    sample.image.thumbnail = thumbnail.id;
                    sample.image.tiles[0] = tile.id;
                    let mut claim = store.begin_run_append(record.run.id).unwrap();
                    add_capture(&mut record, &sample, 0);
                    store
                        .append_run_targets(
                            &mut claim,
                            &[TargetCheckpoint {
                                index: 0,
                                target: &record.run.targets[0],
                                capture: record.captures.last(),
                            }],
                        )
                        .unwrap();
                    if boundary == "resume" {
                        record.run.state = JobState::Paused;
                        store.checkpoint_run(&record).unwrap();
                        record.run.state = JobState::Running;
                        record.queue.as_mut().unwrap().resumed_at.push(2);
                    } else {
                        record.run.state = match boundary {
                            "pause" => JobState::Paused,
                            "finish" => JobState::Completed,
                            _ => JobState::Cancelled,
                        };
                        if record.run.state.terminal() {
                            record.run.finished_at = Some(2);
                        }
                    }
                    let id = match role {
                        "original" => &sample.image.original,
                        "thumbnail" => &sample.image.thumbnail,
                        _ => &sample.image.tiles[0],
                    };
                    let path = store.artifact_path(id).unwrap();
                    let original = std::fs::read(&path).unwrap();
                    match damage {
                        "same-size" => std::fs::write(&path, vec![b'x'; original.len()]).unwrap(),
                        "missing" => std::fs::remove_file(&path).unwrap(),
                        "replaced" => {
                            std::fs::remove_file(&path).unwrap();
                            std::fs::write(&path, vec![b'R'; original.len()]).unwrap();
                        }
                        _ => {
                            std::fs::remove_file(&path).unwrap();
                            let outside = root.path().join("outside");
                            std::fs::write(&outside, &original).unwrap();
                            #[cfg(unix)]
                            std::os::unix::fs::symlink(outside, &path).unwrap();
                            #[cfg(not(unix))]
                            std::fs::write(&path, b"replacement").unwrap();
                        }
                    }
                    let snapshot = |store: &Store| {
                        [
                            "SELECT * FROM runs",
                            "SELECT * FROM run_checkpoint_heads",
                            "SELECT * FROM run_target_deltas",
                            "SELECT * FROM artifact_refs ORDER BY owner_kind,owner_id,hash",
                            "SELECT * FROM projects",
                        ]
                        .map(|sql| {
                            let mut statement = store.connection.prepare(sql).unwrap();
                            let columns = statement.column_count();
                            statement
                                .query_map([], |row| {
                                    (0..columns)
                                        .map(|i| row.get::<_, rusqlite::types::Value>(i))
                                        .collect::<rusqlite::Result<Vec<_>>>()
                                })
                                .unwrap()
                                .collect::<rusqlite::Result<Vec<_>>>()
                                .unwrap()
                        })
                    };
                    let before = snapshot(&store);
                    let result = if boundary == "resume" {
                        store.resume_run(&record)
                    } else if boundary == "pause" {
                        store.checkpoint_run(&record)
                    } else {
                        store.finish_run(&record)
                    };
                    let error = result.unwrap_err();
                    let cause = format!("{error:#}");
                    if damage == "missing" {
                        assert_eq!(
                            error.downcast_ref::<std::io::Error>().unwrap().kind(),
                            std::io::ErrorKind::NotFound
                        );
                    } else {
                        assert_eq!(
                            cause,
                            if damage == "same-size" || damage == "replaced" {
                                "artifact integrity check failed"
                            } else {
                                "artifact is missing, changed, or unsafe"
                            },
                            "{boundary}/{role}/{damage}"
                        );
                    }
                    assert_eq!(
                        before,
                        snapshot(&store),
                        "{boundary}/{role}/{damage} committed a partial compaction"
                    );
                    assert!(store.load_run(record.run.id).is_err());
                    assert!(store.activate_baseline(record.run.id, 1).is_err());
                }
            }
        }
    }
    #[test]
    fn deltas_equal_full_snapshots_and_refgraph_with_linear_committed_work() {
        let (_root, mut delta, mut record, sample) = checkpoint_fixture(32);
        let legacy_root = tempfile::tempdir().unwrap();
        let mut legacy = Store::open(AppPaths::isolated(legacy_root.path())).unwrap();
        let project = delta.load_project(record.run.project).unwrap();
        legacy.save_project(&project.project, None).unwrap();
        legacy
            .put_artifact(
                &b"synthetic CAS accounting fixture (not browser coverage)"[..],
                ArtifactKind::Screenshot,
            )
            .unwrap();
        legacy.freeze_inventory(&record.run.inventory).unwrap();
        legacy.begin_run(&record).unwrap();
        let mut claim = delta.begin_run_append(record.run.id).unwrap();
        for i in 0..32 {
            add_capture(&mut record, &sample, i);
            delta
                .append_run_targets(
                    &mut claim,
                    &[TargetCheckpoint {
                        index: i,
                        target: &record.run.targets[i],
                        capture: record.captures.last(),
                    }],
                )
                .unwrap();
            legacy.checkpoint_run(&record).unwrap();
            assert_eq!(
                serde_json::to_vec(&delta.load_run(record.run.id).unwrap()).unwrap(),
                serde_json::to_vec(&legacy.load_run(record.run.id).unwrap()).unwrap()
            );
        }
        let refs = |store: &Store| {
            store.connection.prepare("SELECT owner_kind,owner_id,hash FROM artifact_refs ORDER BY owner_kind,owner_id,hash").unwrap().query_map([], |row| Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?,row.get::<_,String>(2)?))).unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap()
        };
        assert_eq!(refs(&delta), refs(&legacy));
        assert_eq!(claim.work().evidence_serializations, 32);
        assert_eq!(claim.work().target_validations, 32);
        record.run.state = JobState::Completed;
        record.run.finished_at = Some(2);
        delta.finish_run(&record).unwrap();
        legacy.finish_run(&record).unwrap();
        assert_eq!(
            delta
                .connection
                .query_row("SELECT count(*) FROM run_target_deltas", [], |row| row
                    .get::<_, u64>(0))
                .unwrap(),
            0
        );
        let immutable = serde_json::to_vec(&delta.load_run(record.run.id).unwrap()).unwrap();
        assert_eq!(
            immutable,
            serde_json::to_vec(&legacy.load_run(record.run.id).unwrap()).unwrap()
        );
        assert!(
            delta.activate_baseline(record.run.id, 1).is_err(),
            "synthetic non-PNG CAS must NEVER pass publication"
        );
        record.captures[0].metadata.duration_ms += 1;
        assert!(delta.finish_run(&record).is_err());
        assert_eq!(
            immutable,
            serde_json::to_vec(&delta.load_run(record.run.id).unwrap()).unwrap()
        );
    }
    #[test]
    fn delta_claim_pause_reopen_resume_refusal_and_corruption_boundaries() {
        let (root, mut store, mut record, sample) = checkpoint_fixture(4);
        let paths = AppPaths::isolated(root.path());
        let mut other = Store::open(paths.clone()).unwrap();
        let mut stale = other.begin_run_append(record.run.id).unwrap();
        let mut claim = store.begin_run_append(record.run.id).unwrap();
        add_capture(&mut record, &sample, 0);
        let update = TargetCheckpoint {
            index: 0,
            target: &record.run.targets[0],
            capture: record.captures.last(),
        };
        store.append_run_targets(&mut claim, &[update]).unwrap();
        assert!(
            other
                .append_run_targets(
                    &mut stale,
                    &[TargetCheckpoint {
                        index: 0,
                        target: &record.run.targets[0],
                        capture: record.captures.last()
                    }]
                )
                .is_err()
        );
        assert!(
            store
                .append_run_targets(
                    &mut claim,
                    &[TargetCheckpoint {
                        index: 0,
                        target: &record.run.targets[0],
                        capture: None
                    }]
                )
                .is_err()
        );
        let original = serde_json::to_vec(&record.captures).unwrap();
        record.run.state = JobState::Paused;
        store.checkpoint_run(&record).unwrap();
        drop(store);
        let mut store = Store::open(paths).unwrap();
        assert_eq!(
            original,
            serde_json::to_vec(&store.load_run(record.run.id).unwrap().captures).unwrap()
        );
        record.run.state = JobState::Running;
        record.queue.as_mut().unwrap().resumed_at.push(2);
        store.resume_run(&record).unwrap();
        add_capture(&mut record, &sample, 1);
        assert!(
            store
                .append_run_targets(
                    &mut claim,
                    &[TargetCheckpoint {
                        index: 1,
                        target: &record.run.targets[1],
                        capture: record.captures.last()
                    }]
                )
                .is_err(),
            "compaction/resume must invalidate old claims"
        );
        let mut claim = store.begin_run_append(record.run.id).unwrap();
        let mut invalid = record.run.targets[1].clone();
        invalid.attempts = 0;
        assert!(
            store
                .append_run_targets(
                    &mut claim,
                    &[TargetCheckpoint {
                        index: 1,
                        target: &invalid,
                        capture: record.captures.last()
                    }]
                )
                .is_err()
        );
        store
            .append_run_targets(
                &mut claim,
                &[TargetCheckpoint {
                    index: 1,
                    target: &record.run.targets[1],
                    capture: record.captures.last(),
                }],
            )
            .unwrap();
        assert_eq!(
            serde_json::to_vec(&store.load_run(record.run.id).unwrap()).unwrap(),
            serde_json::to_vec(&record).unwrap()
        );
        // Same-size CAS tampering is not cached away. Load/publication boundaries
        // verify bytes even though delta updates do not rehash old evidence.
        let path = store.artifact_path(&sample.image.original).unwrap();
        std::fs::write(&path, vec![b'x'; sample.metadata.bytes as usize]).unwrap();
        record.run.targets[2].attempts = 1;
        store
            .append_run_targets(
                &mut claim,
                &[TargetCheckpoint {
                    index: 2,
                    target: &record.run.targets[2],
                    capture: None,
                }],
            )
            .unwrap();
        assert!(store.load_run(record.run.id).is_err());
        assert!(store.activate_baseline(record.run.id, 1).is_err());
    }
    #[test]
    fn interrupted_delta_queue_recovers_and_resumes_without_changing_evidence() {
        let (_root, mut store, mut record, sample) = checkpoint_fixture(3);
        let mut claim = store.begin_run_append(record.run.id).unwrap();
        add_capture(&mut record, &sample, 0);
        store
            .append_run_targets(
                &mut claim,
                &[TargetCheckpoint {
                    index: 0,
                    target: &record.run.targets[0],
                    capture: record.captures.last(),
                }],
            )
            .unwrap();
        store
            .connection
            .execute(
                "UPDATE runs SET owner_pid=?1 WHERE id=?2",
                params![i32::MAX, record.run.id.to_string()],
            )
            .unwrap();
        #[cfg(target_os = "linux")]
        {
            let original = serde_json::to_vec(&record.captures).unwrap();
            assert_eq!(store.recover_runs().unwrap(), 1);
            let mut recovered = store.load_run(record.run.id).unwrap();
            assert_eq!(recovered.run.state, JobState::Interrupted);
            assert_eq!(serde_json::to_vec(&recovered.captures).unwrap(), original);
            recovered.run.state = JobState::Running;
            recovered.run.finished_at = None;
            recovered.queue.as_mut().unwrap().resumed_at.push(now());
            store.resume_run(&recovered).unwrap();
            let mut next = store.begin_run_append(recovered.run.id).unwrap();
            add_capture(&mut recovered, &sample, 1);
            store
                .append_run_targets(
                    &mut next,
                    &[TargetCheckpoint {
                        index: 1,
                        target: &recovered.run.targets[1],
                        capture: recovered.captures.last(),
                    }],
                )
                .unwrap();
            assert_eq!(
                serde_json::to_vec(&store.load_run(record.run.id).unwrap()).unwrap(),
                serde_json::to_vec(&recovered).unwrap()
            );
        }
    }
    #[test]
    fn queue_claim_is_exclusive_and_terminal_snapshots_are_immutable() {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(root.path());
        let mut first = Store::open(paths.clone()).unwrap();
        let project = Project::draft(
            "Synthetic queue contract fixture".into(),
            "https://example.test/docs",
        )
        .unwrap();
        first.save_project(&project, None).unwrap();
        let mut profile = project.profiles[0].clone();
        profile.runtime = RuntimeIdentity::Pinned {
            node: "fixture".into(),
            worker: "fixture".into(),
            playwright: "fixture".into(),
            browser: "fixture".into(),
        };
        let page = InventoryPage {
            id: PageId::new(),
            canonical: project.sources[0].clone(),
            included: true,
            reason: None,
        };
        let mut record = RunRecord {
            run: CaptureRun {
                id: RunId::new(),
                project: project.id,
                intent: RunIntent::NewBaseline,
                inventory: InventoryRevision {
                    id: InventoryId::new(),
                    project: project.id,
                    identity_version: 2,
                    pages: vec![page.clone()],
                    coverage: DiscoveryCoverage::Complete,
                },
                profiles: vec![profile.clone()],
                environment: project.environments[0].clone(),
                rendering: RenderingEnvironment {
                    os: "fixture".into(),
                    architecture: "fixture".into(),
                    browser: "fixture".into(),
                    worker_version: "fixture".into(),
                    profile_version: 1,
                    fonts_fingerprint: None,
                },
                state: JobState::Running,
                started_at: Some(1),
                finished_at: None,
                targets: vec![CaptureTarget {
                    key: TargetKey {
                        page: page.id,
                        profile: profile.id,
                    },
                    requested: page.canonical,
                    final_url: None,
                    attempts: 0,
                    outcome: TargetOutcome::Pending,
                    warnings: vec![],
                }],
            },
            capture: None,
            captures: vec![],
            queue: Some(QueueSettings {
                discovery: project.discovery,
                project_revision: 1,
                resumed_at: vec![],
            }),
        };
        first.freeze_inventory(&record.run.inventory).unwrap();
        assert!(first.freeze_inventory(&record.run.inventory).is_err());
        first.begin_run(&record).unwrap();
        record.run.state = JobState::Paused;
        first.checkpoint_run(&record).unwrap();
        let mut second = Store::open(paths).unwrap();
        record.run.state = JobState::Running;
        record.queue.as_mut().unwrap().resumed_at.push(2);
        first.resume_run(&record).unwrap();
        assert!(second.resume_run(&record).is_err());
        let mut foreign = record.clone();
        foreign.run.targets[0].requested = "https://outside.test/".parse().unwrap();
        assert!(first.checkpoint_run(&foreign).is_err());
        record.run.targets[0].attempts = 1;
        record.run.targets[0].outcome = TargetOutcome::Failed {
            code: CaptureError::Timeout,
        };
        record.run.state = JobState::CompletedWithErrors;
        record.run.finished_at = Some(3);
        first.finish_run(&record).unwrap();
        assert!(first.activate_baseline(record.run.id, 1).is_err());
        assert!(first.resume_run(&record).is_err());
        record.run.targets[0].warnings.push("mutation".into());
        assert!(first.finish_run(&record).is_err());
        assert!(
            first.load_run(record.run.id).unwrap().run.targets[0]
                .warnings
                .is_empty()
        );
    }
}
