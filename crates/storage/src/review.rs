// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::capture::{encode, pin};
use regressee_core::{analysis_engine as engine, evidence::*};
use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};

// Read-local only: callers supply the authoritative reader, never viewer state.
fn read_source_once<T>(
    sources: &mut BTreeMap<RunId, T>,
    id: RunId,
    load: impl FnOnce() -> Result<T>,
) -> Result<&T> {
    if let Entry::Vacant(entry) = sources.entry(id) {
        entry.insert(load()?);
    }
    Ok(&sources[&id])
}

// Preserve the existing `.find` semantics for duplicate keys. Duplicate report/
// failure detection remains separate and must not be weakened by indexing.
fn first_by_target<'a, T>(
    values: impl Iterator<Item = (&'a TargetKey, &'a T)>,
) -> BTreeMap<TargetKey, &'a T> {
    let mut index = BTreeMap::new();
    for (key, value) in values {
        index.entry(key.clone()).or_insert(value);
    }
    index
}

fn capture_descriptors(run: &RunRecord) -> Result<BTreeMap<TargetKey, engine::CaptureDescriptor>> {
    let captures = first_by_target(
        run.all_captures()
            .map(|capture| (&capture.metadata.key, capture)),
    );
    captures
        .into_iter()
        .map(|(key, capture)| Ok((key, descriptor(run, capture)?)))
        .collect()
}

#[cfg(test)]
mod authoritative_read_index_tests {
    use super::*;

    #[test]
    fn analysis_sources_are_read_once_per_distinct_run_and_failures_are_not_cached() -> Result<()> {
        let ids = [RunId::new(), RunId::new(), RunId::new()];
        let mut sources = BTreeMap::new();
        let mut loads = BTreeMap::<RunId, usize>::new();
        for position in 0..3000 {
            let id = ids[position % ids.len()];
            assert_eq!(
                *read_source_once(&mut sources, id, || {
                    *loads.entry(id).or_default() += 1;
                    Ok(id)
                })?,
                id
            );
        }
        assert_eq!(sources.len(), 3);
        assert!(ids.iter().all(|id| loads.get(id) == Some(&1)));
        let damaged = RunId::new();
        for _ in 0..2 {
            assert!(
                read_source_once(&mut sources, damaged, || {
                    *loads.entry(damaged).or_default() += 1;
                    anyhow::bail!("missing/corrupt authoritative evidence")
                })
                .is_err()
            );
        }
        assert_eq!(loads[&damaged], 2);
        assert!(!sources.contains_key(&damaged));
        Ok(())
    }

    #[test]
    fn analysis_indexes_preserve_first_match_without_hiding_later_keys() {
        let key = TargetKey {
            page: PageId::new(),
            profile: ProfileId::new(),
        };
        let later = TargetKey {
            page: PageId::new(),
            profile: ProfileId::new(),
        };
        let values = [(key.clone(), 1), (key.clone(), 2), (later.clone(), 3)];
        let index = first_by_target(values.iter().map(|(key, value)| (key, value)));
        assert_eq!(index[&key], &1);
        assert_eq!(index[&later], &3);
        assert!(!index.contains_key(&TargetKey {
            page: PageId::new(),
            profile: ProfileId::new()
        }));
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ApprovalContext {
    pub project: ProjectId,
    pub project_revision: u64,
    pub run: RunId,
    pub baseline: BaselineId,
    pub analysis: AnalysisId,
    pub display_revision: u64,
    pub selected: Vec<TargetKey>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApprovalPreview {
    pub context: ApprovalContext,
    pub baseline: BaselineVersion,
    pub analysis: AnalysisId,
    pub selected: Vec<TargetKey>,
    pub inherited: usize,
    pub provenance: Vec<(TargetKey, String, String)>,
    /// Read-only consent metadata, resolved on the storage worker; no database
    /// schema or baseline/capture identity changes.
    #[serde(default)]
    pub source_environments: Vec<(TargetKey, String)>,
}
impl Store {
    pub fn display_analysis(&mut self, id: AnalysisId) -> Result<()> {
        let a = self.load_analysis(id)?;
        self.connection.execute("INSERT INTO analysis_display(run_id,analysis_id) VALUES(?1,?2) ON CONFLICT(run_id) DO UPDATE SET analysis_id=excluded.analysis_id,revision=analysis_display.revision+1",params![a.revision.run.to_string(),id.to_string()])?;
        Ok(())
    }
    pub fn invalidate_approval(&mut self, run: RunId) -> Result<()> {
        self.connection.execute(
            "UPDATE analysis_display SET revision=revision+1 WHERE run_id=?1",
            [run.to_string()],
        )?;
        Ok(())
    }
    pub fn load_analysis(&self, id: AnalysisId) -> Result<ComparisonEvidence> {
        let s: String = self.connection.query_row(
            "SELECT record FROM analyses WHERE id=?1",
            [id.to_string()],
            |r| r.get(0),
        )?;
        ensure!(s.len() <= 64 * 1024 * 1024, "analysis exceeds budget");
        let a: ComparisonEvidence = serde_json::from_str(&s)?;
        ensure!(a.revision.id == id, "analysis identity mismatch");
        a.revision.policy.validate()?;
        for result in &a.revision.results {
            result.validate()?;
        }
        if let Some(p) = &a.perceptual {
            ensure!(
                a.revision.algorithm == engine::ALGORITHM_ID,
                "stored algorithm mismatch"
            );
            p.policy.validate()?;
            let run = self.load_run(a.revision.run)?;
            let baseline = self.load_baseline(a.revision.baseline)?;
            ensure!(
                run.run.intent
                    == RunIntent::Check {
                        baseline: baseline.id
                    },
                "stored run/baseline mismatch"
            );
            let references = first_by_target(
                baseline
                    .references
                    .iter()
                    .map(|reference| (&reference.key, reference)),
            );
            let summaries = first_by_target(
                a.revision
                    .results
                    .iter()
                    .map(|result| (&result.key, result)),
            );
            // Failure validation originally used `.any`, not `.find`; preserve
            // that distinction even if a stored summary contains duplicate keys.
            let failed_summaries: BTreeSet<_> = a
                .revision
                .results
                .iter()
                .filter(|result| {
                    result.classification == Classification::NotComparable
                        && result.eligible_pixels == 0
                })
                .map(|result| result.key.clone())
                .collect();
            let after = capture_descriptors(&run)?;
            let mut sources = BTreeMap::new();
            sources.insert(run.run.id, after);
            let mut keys = BTreeSet::new();
            for (key, report) in &p.reports {
                ensure!(
                    keys.insert(key)
                        && report.request.policy == p.policy
                        && report.schema_version == engine::SCHEMA_VERSION,
                    "stored report identity mismatch"
                );
                report.request.validate()?;
                let reference = references
                    .get(key)
                    .context("stored report target missing")?;
                let before = read_source_once(&mut sources, reference.run, || {
                    // Verify every original and derivative before indexing. A
                    // corrupt later capture must fail even if this report is earlier.
                    capture_descriptors(&self.load_run(reference.run)?)
                })?
                .get(key)
                .context("stored before missing")?;
                let before_matches = before == &report.request.before;
                let after = sources[&run.run.id]
                    .get(key)
                    .context("stored after missing")?;
                ensure!(
                    before_matches && after == &report.request.after,
                    "stored report capture binding mismatch"
                );
                let result = summaries.get(key).context("stored summary missing")?;
                ensure!(
                    result.eligible_pixels == report.eligible_pixels
                        && result.changed_pixels == report.changed_pixels
                        && report.changed_fraction.numerator == report.changed_pixels
                        && report.changed_fraction.denominator == report.eligible_pixels
                        && report.eligible_pixels + report.ignored_pixels
                            == report.coordinate_union_pixels,
                    "stored report coverage mismatch"
                );
                if result.classification == Classification::NoSignificantChange {
                    ensure!(
                        report.eligible_pixels > 0
                            && report.classification == engine::Classification::NoSignificantChange
                            && report.gates.is_empty()
                            && !report
                                .warnings
                                .contains(&engine::Warning::RenderingEnvironmentMismatch),
                        "stored noncomparable evidence cannot be clean"
                    );
                }
                if report.eligible_pixels == 0 {
                    ensure!(
                        result.classification == Classification::NotComparable
                            && report.gates.contains(&engine::GateReason::AllMasked),
                        "stored all-masked result cannot pass"
                    );
                }
            }
            for (key, _) in &p.failures {
                ensure!(
                    keys.insert(key) && failed_summaries.contains(key),
                    "stored failure cannot be unchanged"
                );
            }
        }
        for image in a.heatmap.iter().chain(a.heatmaps.iter().map(|(_, h)| h)) {
            for id in image.artifacts() {
                self.verify_artifact(&id)?;
            }
        }
        Ok(a)
    }
    pub fn cached_analysis(&self, run: RunId, key: &str) -> Result<Option<ComparisonEvidence>> {
        // Bounded history lookup; cache misses never change evidence semantics.
        let mut q = self
            .connection
            .prepare("SELECT id FROM analyses WHERE run_id=?1 ORDER BY rowid DESC LIMIT 128")?;
        for id in q.query_map([run.to_string()], |r| r.get::<_, String>(0))? {
            let id = serde_json::from_value(serde_json::json!(id?))?;
            let a = self.load_analysis(id)?;
            if a.perceptual.as_ref().is_some_and(|p| p.cache_key == key) {
                return Ok(Some(a));
            }
        }
        Ok(None)
    }
    pub fn save_mask(
        &mut self,
        project: ProjectId,
        request: &engine::AnalysisRequest,
    ) -> Result<()> {
        request.validate()?;
        let mask = request.mask.as_ref().context("mask revision missing")?;
        self.load_project(project)?;
        // Two exact retained runs, never an unbounded history scan or caller claim.
        for d in [&request.before, &request.after] {
            let id = d
                .capture_id
                .split_once(':')
                .context("capture identity missing run")?
                .0;
            let run = self.load_run(serde_json::from_value(serde_json::json!(id))?)?;
            ensure!(
                run.run.project == project
                    && run
                        .all_captures()
                        .any(|c| descriptor(&run, c).is_ok_and(|actual| &actual == d)),
                "mask capture pair does not belong to this project"
            );
        }
        self.connection.execute(
            "INSERT INTO ignore_masks VALUES(?1,?2,?3,?4,?5)",
            params![
                mask.revision,
                project.to_string(),
                mask.target.page,
                mask.target.profile,
                encode(mask)?
            ],
        )?;
        Ok(())
    }
    pub fn latest_mask(
        &self,
        project: ProjectId,
        key: &TargetKey,
    ) -> Result<Option<engine::MaskBinding>> {
        let s: Option<String> = self.connection.query_row("SELECT record FROM ignore_masks WHERE project_id=?1 AND page_id=?2 AND profile_id=?3 ORDER BY rowid DESC LIMIT 1",params![project.to_string(),key.page.to_string(),key.profile.to_string()], |r| r.get(0)).optional()?;
        s.map(|s| {
            ensure!(s.len() <= 256 * 1024, "mask too large");
            Ok(serde_json::from_str(&s)?)
        })
        .transpose()
    }
    pub fn save_review(&mut self, decision: &ReviewDecision) -> Result<()> {
        ensure!(
            decision.note.len() <= 8192 && decision.reviewed_at > 0,
            "invalid review note/date"
        );
        let a = self.load_analysis(decision.analysis)?;
        ensure!(
            a.revision.results.iter().any(|r| r.key == decision.key),
            "review target/revision missing"
        );
        self.connection.execute(
            "INSERT INTO review_decisions(analysis_id,record) VALUES(?1,?2)",
            params![decision.analysis.to_string(), encode(decision)?],
        )?;
        Ok(())
    }
    pub fn review_history(&self, run: RunId) -> Result<Vec<ReviewDecision>> {
        let mut q = self.connection.prepare("SELECT d.record FROM review_decisions d JOIN analyses a ON a.id=d.analysis_id WHERE a.run_id=?1 ORDER BY d.id DESC LIMIT 256")?;
        q.query_map([run.to_string()], |r| r.get::<_, String>(0))?
            .map(|s| Ok(serde_json::from_str(&s?)?))
            .collect()
    }
    pub fn baseline_history(&self, project: ProjectId) -> Result<Vec<BaselineVersion>> {
        let mut q = self.connection.prepare(
            "SELECT id FROM baselines WHERE project_id=?1 ORDER BY rowid DESC LIMIT 128",
        )?;
        q.query_map([project.to_string()], |r| r.get::<_, String>(0))?
            .map(|s| self.load_baseline(serde_json::from_value(serde_json::json!(s?))?))
            .collect()
    }
    pub fn approval_preview(
        &self,
        analysis: AnalysisId,
        selected: &[TargetKey],
    ) -> Result<ApprovalPreview> {
        let a = self.load_analysis(analysis)?;
        let run = self.load_run(a.revision.run)?;
        let parent = self.load_baseline(a.revision.baseline)?;
        let project = self.load_project(parent.project)?;
        ensure!(
            run.run.project == parent.project,
            "approval project/run mismatch"
        );
        let (displayed, display_revision): (String, u64) = self.connection.query_row(
            "SELECT analysis_id,revision FROM analysis_display WHERE run_id=?1",
            [run.run.id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            displayed == analysis.to_string(),
            "Displayed analysis changed; preview again"
        );
        ensure!(
            project.project.active_baseline == Some(parent.id),
            "Active baseline changed; check the new baseline before accepting"
        );
        ensure!(
            !selected.is_empty() && selected.len() <= parent.references.len(),
            "Select successful compatible targets"
        );
        let keys: std::collections::BTreeSet<_> = selected.iter().collect();
        ensure!(keys.len() == selected.len(), "duplicate selection");
        let mut references = parent.references.clone();
        // One verified source snapshot, local to this preview. Commit calls this
        // method again, under its own fresh transaction, so no stale disk/cache
        // authority can cross the consent boundary. Memory stays bounded to one
        // additional run rather than retaining an arbitrary composite history.
        let mut source: Option<regressee_core::evidence::RunRecord> = None;
        for key in selected {
            ensure!(
                a.revision.results.iter().any(|r| &r.key == key
                    && matches!(
                        r.classification,
                        Classification::Changed | Classification::NoSignificantChange
                    )),
                "Failed or noncomparable analysis targets cannot be approved"
            );
            let reference = references
                .iter_mut()
                .find(|r| &r.key == key)
                .context("selected target not in baseline")?;
            if source.as_ref().is_none_or(|s| s.run.id != reference.run) {
                source = Some(self.load_run(reference.run)?);
            }
            let old = source.as_ref().unwrap();
            let before = old.evidence(key).context("inherited evidence missing")?;
            let after = run
                .evidence(key)
                .context("Failed or missing captures cannot be approved")?;
            ensure!(
                before.metadata.profile == after.metadata.profile
                    && old.run.rendering == run.run.rendering,
                "Incompatible rendering/profile mixing requires a complete replacement baseline"
            );
            ensure!(
                after.metadata.stable && after.metadata.warnings.is_empty(),
                "Capture did not settle or has readiness/replacement warnings; capture again before approval"
            );
            let p = a
                .perceptual
                .as_ref()
                .context("Reanalyze this historical result before approval")?;
            let report = p
                .reports
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, r)| r)
                .context("Failed analysis cannot be approved")?;
            ensure!(
                report.gates.is_empty()
                    && report
                        .warnings
                        .iter()
                        .all(|w| *w != engine::Warning::RenderingEnvironmentMismatch),
                "Noncomparable captures cannot be approved"
            );
            *reference = BaselineReference {
                key: key.clone(),
                run: run.run.id,
                artifact: after.image.original.clone(),
                captured_at: run.run.finished_at.context("capture date missing")?,
            };
        }
        let baseline = BaselineVersion {
            id: BaselineId::new(),
            project: parent.project,
            parent: Some(parent.id),
            inventory: parent.inventory,
            created_at: crate::capture::now(),
            references,
            intended_count: parent.intended_count,
            explicitly_partial: parent.explicitly_partial,
        };
        baseline.validate()?;
        let mut provenance = Vec::new();
        let mut source_environments = Vec::new();
        for reference in &baseline.references {
            // The check run is already fully verified above; inherited sources
            // use the bounded worker-local snapshot, not a UI cache.
            let record = if reference.run == run.run.id {
                &run
            } else {
                if source.as_ref().is_none_or(|s| s.run.id != reference.run) {
                    source = Some(self.load_run(reference.run)?);
                }
                source.as_ref().unwrap()
            };
            source_environments.push((reference.key.clone(), record.run.environment.name.clone()));
            let capture = record
                .evidence(&reference.key)
                .context("source evidence missing")?;
            provenance.push((
                reference.key.clone(),
                capture.metadata.requested.clone(),
                capture.metadata.final_url.clone(),
            ));
        }
        Ok(ApprovalPreview {
            context: ApprovalContext {
                project: parent.project,
                project_revision: project.revision,
                run: run.run.id,
                baseline: parent.id,
                analysis,
                display_revision,
                selected: selected.to_vec(),
            },
            inherited: baseline.references.len() - selected.len(),
            baseline,
            analysis,
            selected: selected.to_vec(),
            provenance,
            source_environments,
        })
    }
    pub fn accept_selected(
        &mut self,
        preview: &ApprovalPreview,
        expected_revision: u64,
    ) -> Result<ProjectRecord> {
        let mut expected = preview.context.clone();
        expected.project_revision = expected_revision;
        self.accept_selected_in_context(preview, &expected)
    }
    pub fn accept_selected_in_context(
        &mut self,
        preview: &ApprovalPreview,
        expected: &ApprovalContext,
    ) -> Result<ProjectRecord> {
        ensure!(
            &preview.context == expected,
            "Approval context changed; preview again"
        );
        let expected_revision = expected.project_revision;
        let fresh = self.approval_preview(preview.analysis, &preview.selected)?;
        ensure!(
            fresh.context == *expected
                && fresh.baseline.references == preview.baseline.references
                && fresh.baseline.parent == preview.baseline.parent
                && fresh.baseline.project == preview.baseline.project
                && fresh.provenance == preview.provenance
                && fresh.inherited == preview.inherited,
            "approval preview changed"
        );
        let mut project = self.load_project(preview.baseline.project)?;
        ensure!(
            project.revision == expected_revision,
            "project changed before acceptance"
        );
        let baseline = &preview.baseline;
        baseline.validate()?;
        ensure!(
            baseline.inventory == fresh.baseline.inventory
                && baseline.intended_count == fresh.baseline.intended_count
                && baseline.explicitly_partial == fresh.baseline.explicitly_partial,
            "approval coverage changed"
        );
        let mut captures = Vec::new();
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let mut originals = std::collections::BTreeSet::new();
        for r in &baseline.references {
            let capture = self
                .load_run(r.run)?
                .evidence(&r.key)
                .context("source missing")?
                .clone();
            if originals.insert(&r.artifact) {
                let artifact = self.verify_artifact(&r.artifact)?;
                ensure!(
                    artifact.kind == ArtifactKind::Screenshot
                        && artifact.bytes == capture.metadata.bytes,
                    "Original screenshot metadata conflicts"
                );
                ensure!(
                    crate::images::validate_png(&self.artifact_path(&r.artifact)?, &cancel)?
                        == (capture.metadata.width, capture.metadata.height),
                    "Original screenshot PNG dimensions conflict"
                );
            }
            captures.push(capture);
        }
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        ensure!(
            tx.query_row(
                "SELECT revision FROM projects WHERE id=?1",
                [project.project.id.to_string()],
                |r| r.get::<_, u64>(0)
            )? == expected_revision,
            "project changed during acceptance"
        );
        let (config, displayed, display_revision): (String, String, u64) = tx.query_row(
            "SELECT p.config,d.analysis_id,d.revision FROM projects p JOIN analysis_display d ON d.run_id=?2 WHERE p.id=?1",
            params![expected.project.to_string(), expected.run.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)),
        )?;
        let current: Project = serde_json::from_str(&config)?;
        ensure!(
            current.active_baseline == Some(expected.baseline)
                && displayed == expected.analysis.to_string()
                && display_revision == expected.display_revision,
            "Baseline or displayed analysis changed during acceptance"
        );
        tx.execute(
            "INSERT INTO baselines VALUES(?1,?2,?3,?4)",
            params![
                baseline.id.to_string(),
                baseline.project.to_string(),
                baseline.references[0].run.to_string(),
                encode(baseline)?
            ],
        )?;
        for c in captures {
            for id in c.image.artifacts() {
                pin(&tx, &self.paths, &ArtifactOwner::Baseline(baseline.id), &id)?;
            }
        }
        project.project.active_baseline = Some(baseline.id);
        project.revision += 1;
        tx.execute("UPDATE projects SET config=?1,revision=?2,updated_at=unixepoch() WHERE id=?3 AND revision=?4",params![encode(&project.project)?,project.revision,project.project.id.to_string(),expected_revision])?;
        tx.commit()?;
        Ok(project)
    }
    pub fn restore_baseline(
        &mut self,
        id: BaselineId,
        expected_revision: u64,
    ) -> Result<ProjectRecord> {
        let baseline = self.load_baseline(id)?;
        let cancel = std::sync::atomic::AtomicBool::new(false);
        let mut originals = std::collections::BTreeSet::new();
        for r in &baseline.references {
            if originals.insert(&r.artifact) {
                let source = self.load_run(r.run)?;
                let capture = source.evidence(&r.key).context("source missing")?;
                let artifact = self.verify_artifact(&r.artifact)?;
                ensure!(
                    artifact.kind == ArtifactKind::Screenshot
                        && artifact.bytes == capture.metadata.bytes,
                    "Original screenshot metadata conflicts"
                );
                ensure!(
                    crate::images::validate_png(&self.artifact_path(&r.artifact)?, &cancel)?
                        == (capture.metadata.width, capture.metadata.height),
                    "Original screenshot PNG dimensions conflict"
                );
            }
        }
        let mut project = self.load_project(baseline.project)?;
        ensure!(
            project.revision == expected_revision,
            "project changed before restore"
        );
        let source = self.load_run(baseline.references[0].run)?;
        project.project.active_baseline = Some(id);
        project.project.profiles = source.run.profiles;
        project.project.validate()?;
        project.revision += 1;
        ensure!(self.connection.execute("UPDATE projects SET config=?1,revision=?2,updated_at=unixepoch() WHERE id=?3 AND revision=?4",params![encode(&project.project)?,project.revision,project.project.id.to_string(),expected_revision])? == 1,"project changed during restore");
        Ok(project)
    }
}
