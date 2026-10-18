// SPDX-License-Identifier: MPL-2.0
//! DOM-only integrity readers: no image decoding, derivative paths or PNG gates.
use super::*;
use regressee_core::{dom_snapshot::*, evidence::*};
use std::collections::{BTreeMap, BTreeSet};

fn read_dom_analysis(connection: &Connection, id: DomAnalysisId) -> Result<DomComparisonEvidence> {
    let (json, run, baseline): (String, String, String) = connection.query_row(
        "SELECT record,run_id,baseline_id FROM dom_analyses WHERE id=?1",
        [id.to_string()],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
    )?;
    ensure!(
        json.len() <= 64 * 1024 * 1024,
        "DOM analysis exceeds budget"
    );
    let evidence: DomComparisonEvidence =
        serde_json::from_str(&json).map_err(|_| anyhow::anyhow!("Invalid saved DOM analysis"))?;
    evidence.revision.validate()?;
    evidence.options.validate()?;
    ensure!(
        evidence.created_at > 0
            && evidence.revision.id == id
            && evidence.revision.run.to_string() == run
            && evidence.revision.baseline.to_string() == baseline,
        "DOM analysis identity mismatch"
    );
    Ok(evidence)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomReviewHistoryEntry {
    pub id: i64,
    pub decision: DomReviewDecision,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomReviewHistoryPage {
    pub analysis: DomAnalysisId,
    pub run: RunId,
    pub key: Option<TargetKey>,
    pub items: Vec<DomReviewHistoryEntry>,
    pub total: usize,
    pub next_before: Option<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DomApprovalContext {
    pub project: ProjectId,
    pub run: RunId,
    pub binding: DomApprovalBinding,
    pub display_revision: u64,
    pub sources_hash: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DomApprovalPreview {
    pub context: DomApprovalContext,
    pub baseline: BaselineVersion,
    pub analysis: DomAnalysisId,
    pub selected: Vec<TargetKey>,
    pub inherited: usize,
}
/// Explicit, single-transaction consent for either kind or both. Empty siblings
/// are not inferred from a run, and cannot be approved as an incidental side effect.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct KindApprovalPreview {
    pub screenshot: Option<ApprovalPreview>,
    pub dom: Option<DomApprovalPreview>,
    pub baseline: BaselineVersion,
}

impl Store {
    pub fn inspect_run_kind(
        &self,
        project: ProjectId,
        run: RunId,
        kind: CheckKind,
    ) -> Result<ProjectEvidence> {
        match kind {
            CheckKind::Screenshot => self.inspect_run(project, run),
            CheckKind::DomSnapshot => self.dom_evidence_for_run(project, run),
        }
    }
    pub fn dom_approval_preview(
        &self,
        analysis: DomAnalysisId,
        selected: &[TargetKey],
    ) -> Result<DomApprovalPreview> {
        let evidence = self.load_dom_analysis(analysis)?;
        let checked = self.load_dom_baseline(evidence.revision.baseline)?;
        let parent = self.approval_parent(&checked)?;
        let project = self.load_project(parent.project)?;
        let run = self.load_dom_run(evidence.revision.run)?;
        let (displayed, display_revision): (String, u64) = self.connection.query_row(
            "SELECT analysis_id,revision FROM dom_analysis_display WHERE run_id=?1",
            [run.run.id.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        ensure!(
            displayed == analysis.to_string(),
            "Displayed DOM analysis changed; preview again"
        );
        ensure!(
            !selected.is_empty() && selected.len() <= parent.dom_references.len(),
            "Select eligible saved DOM slots"
        );
        let mut selections = Vec::new();
        for key in selected {
            let result = evidence
                .revision
                .results
                .iter()
                .find(|r| &r.key == key)
                .context("DOM approval target missing")?;
            let after = run.dom_evidence(key).context("DOM After missing")?;
            ensure!(
                after.metadata.baseline_ready(),
                "Unstable or unsupported DOM evidence cannot be approved"
            );
            selections.push(DomApprovalSelection {
                analysis,
                key: key.clone(),
                before: result
                    .before
                    .clone()
                    .context("DOM Before missing; new baseline capture required")?,
                after: result.after.clone().context("DOM After missing")?,
            });
        }
        let binding = DomApprovalBinding {
            checked_baseline: checked.id,
            current_parent: parent.id,
            project_revision: project.revision,
            selections,
        };
        binding.validate_sources(&evidence.revision, &parent)?;
        for selection in &binding.selections {
            ensure!(
                checked
                    .dom_references
                    .iter()
                    .find(|r| r.key == selection.key)
                    == parent
                        .dom_references
                        .iter()
                        .find(|r| r.key == selection.key),
                "Selected DOM Before changed; compare again"
            );
        }
        let mut baseline = parent.clone();
        baseline.id = BaselineId::new();
        baseline.parent = Some(parent.id);
        baseline.created_at = crate::capture::now();
        for selection in &binding.selections {
            let reference = baseline
                .dom_references
                .iter_mut()
                .find(|r| r.key == selection.key)
                .context("Cannot fill baseline omission by selective approval")?;
            *reference = DomBaselineReference {
                key: selection.key.clone(),
                run: run.run.id,
                artifact: selection.after.artifact.clone(),
                captured_at: run.run.finished_at.context("DOM capture date missing")?,
                policy: selection.after.policy,
            };
        }
        baseline.validate()?;
        Ok(DomApprovalPreview {
            context: DomApprovalContext {
                project: parent.project,
                run: run.run.id,
                binding,
                display_revision,
                sources_hash: identity_hash(&(
                    &evidence,
                    &checked,
                    &parent,
                    &baseline.dom_references,
                ))?,
            },
            inherited: baseline.dom_references.len() - selected.len(),
            baseline,
            analysis,
            selected: selected.to_vec(),
        })
    }
    pub fn approval_preview_kinds(
        &self,
        screenshot_analysis: Option<AnalysisId>,
        screenshots: &[TargetKey],
        dom_analysis: Option<DomAnalysisId>,
        snapshots: &[TargetKey],
    ) -> Result<KindApprovalPreview> {
        ensure!(
            screenshot_analysis.is_some() != screenshots.is_empty()
                && dom_analysis.is_some() != snapshots.is_empty(),
            "Selected kinds require exact analysis identities"
        );
        let mut screenshot = screenshot_analysis
            .map(|id| self.approval_preview(id, screenshots))
            .transpose()?;
        let mut dom = dom_analysis
            .map(|id| self.dom_approval_preview(id, snapshots))
            .transpose()?;
        let mut baseline = screenshot
            .as_ref()
            .map(|p| p.baseline.clone())
            .or_else(|| dom.as_ref().map(|p| p.baseline.clone()))
            .context("Select at least one eligible reference slot")?;
        if let (Some(image), Some(snapshot)) = (&screenshot, &dom) {
            ensure!(
                image.context.project == snapshot.context.project
                    && image.context.run == snapshot.context.run
                    && image.context.project_revision == snapshot.context.binding.project_revision
                    && image.context.baseline == snapshot.context.binding.checked_baseline
                    && image.context.parent == snapshot.context.binding.current_parent,
                "Kinds belong to different checks or approval parents"
            );
            baseline.dom_references = snapshot.baseline.dom_references.clone();
        }
        baseline.validate()?;
        // The per-kind maps describe one proposed atomic version. Use its one
        // generated identity/date in every nested consent view, so those fields
        // cannot be changed independently while leaving the committed map intact.
        if let Some(image) = &mut screenshot {
            image.baseline.id = baseline.id;
            image.baseline.created_at = baseline.created_at;
        }
        if let Some(snapshot) = &mut dom {
            snapshot.baseline.id = baseline.id;
            snapshot.baseline.created_at = baseline.created_at;
        }
        Ok(KindApprovalPreview {
            screenshot,
            dom,
            baseline,
        })
    }
    pub fn accept_dom_selected(
        &mut self,
        preview: &DomApprovalPreview,
        expected_revision: u64,
    ) -> Result<ProjectRecord> {
        ensure!(
            preview.context.binding.project_revision == expected_revision,
            "Project changed; preview DOM approval again"
        );
        self.accept_selected_kinds(&KindApprovalPreview {
            screenshot: None,
            dom: Some(preview.clone()),
            baseline: preview.baseline.clone(),
        })
    }
    pub fn accept_selected_kinds(
        &mut self,
        preview: &KindApprovalPreview,
    ) -> Result<ProjectRecord> {
        let mut fresh = self.approval_preview_kinds(
            preview.screenshot.as_ref().map(|p| p.analysis),
            preview
                .screenshot
                .as_ref()
                .map_or(&[], |p| p.selected.as_slice()),
            preview.dom.as_ref().map(|p| p.analysis),
            preview.dom.as_ref().map_or(&[], |p| p.selected.as_slice()),
        )?;
        ensure!(
            fresh.screenshot.as_ref().map(|p| &p.context)
                == preview.screenshot.as_ref().map(|p| &p.context)
                && fresh.dom.as_ref().map(|p| &p.context)
                    == preview.dom.as_ref().map(|p| &p.context),
            "Approval context changed; preview again"
        );
        // Only the new version's generated identity/date are carried across a
        // stateless preview refresh. All nested fields must independently match
        // the freshly derived consent view, including counts and retained sources.
        fresh.baseline.id = preview.baseline.id;
        fresh.baseline.created_at = preview.baseline.created_at;
        if let Some(image) = &mut fresh.screenshot {
            image.baseline.id = preview.baseline.id;
            image.baseline.created_at = preview.baseline.created_at;
        }
        if let Some(snapshot) = &mut fresh.dom {
            snapshot.baseline.id = preview.baseline.id;
            snapshot.baseline.created_at = preview.baseline.created_at;
        }
        ensure!(
            fresh == *preview && !preview.baseline.id.0.is_nil() && preview.baseline.created_at > 0,
            "Approval consent preview changed; preview again"
        );
        let baseline = &preview.baseline;
        let mut project = self.load_project(baseline.project)?;
        let revision = project.revision;
        let parent = baseline.parent.context("Approval parent missing")?;
        let mut new_artifacts = BTreeSet::new();
        if let Some(image) = &preview.screenshot {
            let run = self.load_screenshot_run(image.context.run)?;
            for key in &image.selected {
                let capture = run.evidence(key).context("Selected screenshot missing")?;
                let artifact = self.verify_artifact(&capture.image.original)?;
                ensure!(
                    artifact.kind == ArtifactKind::Screenshot
                        && artifact.bytes == capture.metadata.bytes,
                    "Screenshot original binding mismatch"
                );
                ensure!(
                    images::validate_png(
                        &self.artifact_path(&artifact.id)?,
                        &std::sync::atomic::AtomicBool::new(false)
                    )? == (capture.metadata.width, capture.metadata.height),
                    "Screenshot original dimensions mismatch"
                );
                new_artifacts.extend(capture.image.artifacts());
            }
        }
        if let Some(dom) = &preview.dom {
            for selection in &dom.context.binding.selections {
                new_artifacts.insert(selection.after.artifact.clone());
            }
        }
        // Resolve all pins from exact selected reference metadata, but do not
        // require unrelated inherited-kind files to be readable or approvable.
        let mut inherited = BTreeSet::new();
        for reference in &baseline.references {
            let source = self.load_run_snapshot(reference.run)?;
            let capture = source
                .evidence(&reference.key)
                .context("Inherited image source missing")?;
            ensure!(
                capture.image.original == reference.artifact,
                "Inherited image source mismatch"
            );
            inherited.extend(capture.image.artifacts());
        }
        inherited.extend(baseline.dom_references.iter().map(|r| r.artifact.clone()));
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let (config, current_revision): (String, u64) = tx.query_row(
            "SELECT config,revision FROM projects WHERE id=?1",
            [baseline.project.to_string()],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )?;
        let current: Project = serde_json::from_str(&config)?;
        ensure!(
            current_revision == revision && current.active_baseline == Some(parent),
            "Project or approval parent changed"
        );
        if let Some(image) = &preview.screenshot {
            ensure!(
                image.context.project_revision == revision && image.context.parent == parent,
                "Screenshot approval revision changed"
            );
            let display: (String, u64) = tx.query_row(
                "SELECT analysis_id,revision FROM analysis_display WHERE run_id=?1",
                [image.context.run.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            ensure!(
                display == (image.analysis.to_string(), image.context.display_revision),
                "Displayed screenshot analysis changed"
            );
        }
        if let Some(dom) = &preview.dom {
            ensure!(
                dom.context.binding.project_revision == revision
                    && dom.context.binding.current_parent == parent,
                "DOM approval revision changed"
            );
            let display: (String, u64) = tx.query_row(
                "SELECT analysis_id,revision FROM dom_analysis_display WHERE run_id=?1",
                [dom.context.run.to_string()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            ensure!(
                display == (dom.analysis.to_string(), dom.context.display_revision),
                "Displayed DOM analysis changed"
            );
        }
        baseline.validate()?;
        tx.execute(
            "INSERT INTO baselines VALUES(?1,?2,?3,?4)",
            params![
                baseline.id.to_string(),
                baseline.project.to_string(),
                baseline
                    .frozen_source_run()
                    .context("Frozen source missing")?
                    .to_string(),
                crate::capture::encode(baseline)?
            ],
        )?;
        for id in &inherited {
            if new_artifacts.contains(id) {
                crate::capture::pin(&tx, &self.paths, &ArtifactOwner::Baseline(baseline.id), id)?;
            } else {
                ensure!(tx.execute("INSERT OR IGNORE INTO artifact_refs SELECT 'Baseline',?1,hash FROM artifact_refs WHERE owner_kind='Baseline' AND owner_id=?2 AND hash=?3",params![baseline.id.to_string(),parent.to_string(),id.as_str()])?==1,"Inherited baseline pin missing");
            }
        }
        project.project.active_baseline = Some(baseline.id);
        project.revision += 1;
        ensure!(tx.execute("UPDATE projects SET config=?1,revision=?2,updated_at=unixepoch() WHERE id=?3 AND revision=?4",params![crate::capture::encode(&project.project)?,project.revision,project.project.id.to_string(),revision])?==1,"Project changed during approval");
        tx.commit()?;
        Ok(project)
    }
    fn validate_dom_analysis(&self, evidence: &DomComparisonEvidence) -> Result<()> {
        evidence.revision.validate()?;
        evidence.options.validate()?;
        ensure!(evidence.created_at > 0, "Invalid DOM analysis date");
        let run = self.load_dom_run(evidence.revision.run)?;
        let baseline = self.load_dom_baseline(evidence.revision.baseline)?;
        baseline.validate_check_slots(&run.run)?;
        let frozen = self.load_run_snapshot(
            baseline
                .frozen_source_run()
                .context("Frozen DOM baseline source missing")?,
        )?;
        ensure!(
            run.run.inventory == frozen.run.inventory && run.run.profiles == frozen.run.profiles,
            "DOM analysis changed frozen page/profile configuration"
        );
        ensure!(
            run.run.project == baseline.project
                && run.run.intent
                    == (RunIntent::Check {
                        baseline: baseline.id
                    })
                && run.run.state.terminal()
                && run.run.capture_plan == baseline.capture_plan
                && run.run.capture_plan.mode.dom_snapshots(),
            "DOM analysis run/baseline mismatch"
        );
        let actual: BTreeSet<_> = evidence
            .revision
            .results
            .iter()
            .map(|r| r.key.clone())
            .collect();
        let required: BTreeSet<_> = run.run.targets.iter().map(|t| t.key.clone()).collect();
        let allowed: BTreeSet<_> = if baseline.intended_targets.is_empty() {
            required.clone()
        } else {
            baseline.intended_targets.iter().cloned().collect()
        };
        ensure!(
            required.is_subset(&actual) && actual.is_subset(&allowed),
            "DOM analysis coverage mismatch"
        );
        let mut sources = BTreeMap::new();
        for result in &evidence.revision.results {
            let reference = baseline.dom_references.iter().find(|r| r.key == result.key);
            let before = if let Some(reference) = reference {
                if let std::collections::btree_map::Entry::Vacant(entry) =
                    sources.entry(reference.run)
                {
                    entry.insert(self.load_dom_run(reference.run)?);
                }
                let source = &sources[&reference.run];
                Some(dom_source(
                    &source.run,
                    source
                        .dom_evidence(&result.key)
                        .context("DOM Before missing")?,
                )?)
            } else {
                None
            };
            let after = run
                .dom_evidence(&result.key)
                .map(|c| dom_source(&run.run, c))
                .transpose()?;
            ensure!(
                result.before == before && result.after == after,
                "DOM analysis exact source binding mismatch"
            );
            let expected = if let (Some(before), Some(after)) = (before, after) {
                let (_, before_bytes) =
                    self.read_verified_artifact(&before.artifact, MAX_DOM_BYTES as u64)?;
                let (_, after_bytes) =
                    self.read_verified_artifact(&after.artifact, MAX_DOM_BYTES as u64)?;
                compare_dom_snapshots(
                    before,
                    &before_bytes,
                    after,
                    &after_bytes,
                    &evidence.options,
                    || false,
                )?
            } else {
                let outcome = if reference.is_none() {
                    DomComparisonOutcome::NotComparable {
                        reason: DomNotComparableReason::BaselineOmission,
                    }
                } else {
                    match run
                        .run
                        .targets
                        .iter()
                        .find(|t| t.key == result.key)
                        .map(|t| &t.dom_outcome)
                    {
                        Some(DomTargetOutcome::Failed { code }) => {
                            DomComparisonOutcome::CouldNotCapture { code: *code }
                        }
                        Some(DomTargetOutcome::Cancelled) => DomComparisonOutcome::NotComparable {
                            reason: DomNotComparableReason::Cancelled,
                        },
                        Some(DomTargetOutcome::Skipped { .. }) => {
                            DomComparisonOutcome::NotComparable {
                                reason: DomNotComparableReason::Skipped,
                            }
                        }
                        _ => DomComparisonOutcome::NotComparable {
                            reason: DomNotComparableReason::MissingAfter,
                        },
                    }
                };
                DomAnalysisResult {
                    key: result.key.clone(),
                    before: result.before.clone(),
                    after: result.after.clone(),
                    outcome,
                    diff: None,
                }
            };
            ensure!(
                *result == expected,
                "DOM analysis verdict/diff contradicts verified evidence"
            );
        }
        Ok(())
    }
    pub fn save_dom_analysis(&mut self, evidence: &DomComparisonEvidence) -> Result<()> {
        self.validate_dom_analysis(evidence)?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        tx.execute(
            "INSERT INTO dom_analyses VALUES(?1,?2,?3,?4)",
            params![
                evidence.revision.id.to_string(),
                evidence.revision.run.to_string(),
                evidence.revision.baseline.to_string(),
                crate::capture::encode(evidence)?
            ],
        )?;
        for result in &evidence.revision.results {
            for source in result.before.iter().chain(result.after.iter()) {
                crate::capture::pin(
                    &tx,
                    &self.paths,
                    &ArtifactOwner::DomAnalysis(evidence.revision.id),
                    &source.artifact,
                )?;
            }
        }
        tx.execute("INSERT INTO dom_analysis_display(run_id,analysis_id) VALUES(?1,?2) ON CONFLICT(run_id) DO UPDATE SET analysis_id=excluded.analysis_id,revision=dom_analysis_display.revision+1",params![evidence.revision.run.to_string(),evidence.revision.id.to_string()])?;
        tx.commit()?;
        Ok(())
    }
    pub fn load_dom_analysis(&self, id: DomAnalysisId) -> Result<DomComparisonEvidence> {
        let evidence = read_dom_analysis(&self.connection, id)?;
        self.validate_dom_analysis(&evidence)?;
        Ok(evidence)
    }
    pub fn displayed_dom_analysis(&self, run: RunId) -> Result<Option<DomComparisonEvidence>> {
        let id: Option<String> = self.connection.query_row("SELECT COALESCE((SELECT analysis_id FROM dom_analysis_display WHERE run_id=?1),(SELECT id FROM dom_analyses WHERE run_id=?1 ORDER BY rowid DESC LIMIT 1))",[run.to_string()],|r|r.get(0))?;
        id.map(|id| self.load_dom_analysis(serde_json::from_value(serde_json::json!(id))?))
            .transpose()
    }
    pub fn display_dom_analysis(&mut self, id: DomAnalysisId) -> Result<()> {
        let evidence = self.load_dom_analysis(id)?;
        self.connection.execute("INSERT INTO dom_analysis_display(run_id,analysis_id) VALUES(?1,?2) ON CONFLICT(run_id) DO UPDATE SET analysis_id=excluded.analysis_id,revision=dom_analysis_display.revision+1",params![evidence.revision.run.to_string(),id.to_string()])?;
        Ok(())
    }
    pub fn invalidate_dom_approval(&mut self, run: RunId) -> Result<()> {
        self.connection.execute(
            "UPDATE dom_analysis_display SET revision=revision+1 WHERE run_id=?1",
            [run.to_string()],
        )?;
        Ok(())
    }
    pub fn save_dom_review(&mut self, decision: &DomReviewDecision) -> Result<RunId> {
        ensure!(decision.reviewed_at > 0, "Invalid DOM review date");
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let evidence = read_dom_analysis(&tx, decision.analysis)?;
        decision.validate_for(&evidence.revision)?;
        tx.execute(
            "INSERT INTO dom_review_decisions(analysis_id,record) VALUES(?1,?2)",
            params![
                decision.analysis.to_string(),
                crate::capture::encode(decision)?
            ],
        )?;
        tx.commit()?;
        Ok(evidence.revision.run)
    }
    pub fn current_dom_reviews(
        &self,
        analysis: DomAnalysisId,
        keys: &[TargetKey],
    ) -> Result<Vec<DomReviewDecision>> {
        ensure!(
            keys.len() <= 200,
            "DOM review selection exceeds 200 targets"
        );
        let evidence = read_dom_analysis(&self.connection, analysis)?;
        ensure!(
            keys.iter()
                .all(|k| evidence.revision.results.iter().any(|r| &r.key == k)),
            "DOM review target missing"
        );
        let mut q = self.connection.prepare("SELECT record FROM dom_review_decisions WHERE id IN (SELECT MAX(id) FROM dom_review_decisions WHERE analysis_id=?1 AND (json_extract(record,'$.key.page'),json_extract(record,'$.key.profile')) IN (SELECT json_extract(value,'$.page'),json_extract(value,'$.profile') FROM json_each(?2)) GROUP BY json_extract(record,'$.key.page'),json_extract(record,'$.key.profile')) ORDER BY id DESC")?;
        q.query_map(
            params![analysis.to_string(), serde_json::to_string(keys)?],
            |r| r.get::<_, String>(0),
        )?
        .map(|r| {
            let d: DomReviewDecision = serde_json::from_str(&r?)?;
            d.validate_for(&evidence.revision)?;
            Ok(d)
        })
        .collect()
    }
    pub fn dom_review_history(&self, run: RunId) -> Result<Vec<DomReviewDecision>> {
        let mut q=self.connection.prepare("SELECT d.record FROM dom_review_decisions d JOIN dom_analyses a ON a.id=d.analysis_id WHERE a.run_id=?1 ORDER BY d.id DESC LIMIT 256")?;
        q.query_map([run.to_string()], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_str(&r?)?))
            .collect()
    }
    pub fn dom_review_history_truncated(&self, run: RunId) -> Result<bool> {
        Ok(self.connection.query_row("SELECT EXISTS(SELECT 1 FROM dom_review_decisions d JOIN dom_analyses a ON a.id=d.analysis_id WHERE a.run_id=?1 LIMIT 1 OFFSET 256)",[run.to_string()],|r|r.get(0))?)
    }
    pub fn dom_analysis_history(&self, run: RunId) -> Result<Vec<DomAnalysisId>> {
        let mut q = self
            .connection
            .prepare("SELECT id FROM dom_analyses WHERE run_id=?1 ORDER BY rowid DESC LIMIT 128")?;
        q.query_map([run.to_string()], |r| r.get::<_, String>(0))?
            .map(|r| Ok(serde_json::from_value(serde_json::json!(r?))?))
            .collect()
    }
    pub fn dom_review_history_page(
        &self,
        analysis: DomAnalysisId,
        key: Option<TargetKey>,
        before: Option<i64>,
        limit: usize,
    ) -> Result<DomReviewHistoryPage> {
        ensure!(
            (1..=200).contains(&limit) && before.is_none_or(|b| b > 0),
            "Invalid DOM review history page"
        );
        let evidence = read_dom_analysis(&self.connection, analysis)?;
        ensure!(
            key.as_ref()
                .is_none_or(|k| evidence.revision.results.iter().any(|r| &r.key == k)),
            "DOM review target missing"
        );
        let page = key.as_ref().map(|k| k.page.to_string());
        let profile = key.as_ref().map(|k| k.profile.to_string());
        let total=self.connection.query_row("SELECT count(*) FROM dom_review_decisions WHERE analysis_id=?1 AND (?2 IS NULL OR (json_extract(record,'$.key.page')=?2 AND json_extract(record,'$.key.profile')=?3))",params![analysis.to_string(),page,profile],|r|r.get(0))?;
        let mut q=self.connection.prepare("SELECT id,record FROM dom_review_decisions WHERE analysis_id=?1 AND (?2 IS NULL OR (json_extract(record,'$.key.page')=?2 AND json_extract(record,'$.key.profile')=?3)) AND (?4 IS NULL OR id<?4) ORDER BY id DESC LIMIT ?5")?;
        let mut items = q
            .query_map(
                params![analysis.to_string(), page, profile, before, limit + 1],
                |r| Ok((r.get::<_, i64>(0)?, r.get::<_, String>(1)?)),
            )?
            .map(|r| {
                let (id, json) = r?;
                let decision: DomReviewDecision = serde_json::from_str(&json)?;
                decision.validate_for(&evidence.revision)?;
                Ok(DomReviewHistoryEntry { id, decision })
            })
            .collect::<Result<Vec<_>>>()?;
        let more = items.len() > limit;
        items.truncate(limit);
        let next_before = more.then(|| items.last().unwrap().id);
        Ok(DomReviewHistoryPage {
            analysis,
            run: evidence.revision.run,
            key,
            items,
            total,
            next_before,
        })
    }
    pub fn dom_evidence_for_run(&self, project: ProjectId, run: RunId) -> Result<ProjectEvidence> {
        self.load_project(project)?;
        let record = self.load_dom_run(run)?;
        ensure!(
            record.run.project == project,
            "DOM evidence belongs to another project"
        );
        let baseline = match record.run.intent {
            RunIntent::Check { baseline } => Some(self.load_dom_baseline(baseline)?),
            _ => None,
        };
        let comparison = self.displayed_dom_analysis(run)?;
        let dom_reviews = if let Some(a) = &comparison {
            self.dom_review_history_page(a.revision.id, None, None, 200)?
                .items
                .into_iter()
                .map(|e| e.decision)
                .collect()
        } else {
            vec![]
        };
        let dom_before = baseline
            .as_ref()
            .and_then(|b| b.dom_references.first())
            .map(|r| self.read_dom_snapshot(r.run, &r.key).map(|(c, _)| c))
            .transpose()?;
        Ok(ProjectEvidence {
            latest: Some(record),
            baseline,
            dom_before,
            dom_analyses: comparison.iter().map(|a| a.revision.id).collect(),
            dom_comparison: comparison,
            dom_reviews,
            ..Default::default()
        })
    }
    /// Focused, historical-capable DOM viewer boundary. Never reads image blobs
    /// or borrows the active baseline for a saved check analysis.
    pub fn viewer_dom_evidence(
        &self,
        project: ProjectId,
        analysis: Option<DomAnalysisId>,
    ) -> Result<ProjectEvidence> {
        let saved = self.load_project(project)?;
        let Some(id) = analysis else {
            return if let Some(run) = self.latest_run_id(project)? {
                self.dom_evidence_for_run(project, run)
            } else {
                Ok(ProjectEvidence {
                    baseline: saved
                        .project
                        .active_baseline
                        .map(|id| self.load_dom_baseline(id))
                        .transpose()?,
                    ..Default::default()
                })
            };
        };
        let comparison = self.load_dom_analysis(id)?;
        let record = self.load_dom_run(comparison.revision.run)?;
        ensure!(
            record.run.project == project,
            "DOM viewer analysis belongs to another project"
        );
        let baseline = self.load_dom_baseline(comparison.revision.baseline)?;
        let dom_before = baseline
            .dom_references
            .first()
            .map(|r| self.read_dom_snapshot(r.run, &r.key).map(|(c, _)| c))
            .transpose()?;
        let dom_reviews = self
            .dom_review_history_page(id, None, None, 200)?
            .items
            .into_iter()
            .map(|e| e.decision)
            .collect();
        Ok(ProjectEvidence {
            latest: Some(record),
            baseline: Some(baseline),
            dom_before,
            dom_comparison: Some(comparison),
            dom_analyses: vec![id],
            dom_reviews,
            ..Default::default()
        })
    }
    /// Validate policy, full canonical grammar, privacy exclusions, limits and
    /// claimed diagnostics BEFORE retained CAS publication. No raw backup.
    pub fn ingest_dom_snapshot(
        &mut self,
        input: impl Read,
        metadata: &DomCaptureMetadata,
    ) -> Result<DomCaptureEvidence> {
        let source = self.load_run_snapshot(metadata.run)?;
        let target = source
            .run
            .targets
            .iter()
            .find(|t| t.key == metadata.key)
            .context("DOM ingestion target missing")?;
        let requested =
            regressee_core::environment::map_url(&target.requested, &source.run.environment)?;
        ensure!(
            source.run.capture_plan.mode.dom_snapshots()
                && Some(metadata.policy) == source.run.capture_plan.dom_policy
                && source.run.profiles.contains(&metadata.profile)
                && metadata.requested == requested.as_str()
                && metadata.final_url == requested.as_str()
                && matches!(
                    &target.dom_outcome,
                    DomTargetOutcome::Pending | DomTargetOutcome::Captured { .. }
                ),
            "DOM ingestion frozen source binding mismatch"
        );
        if let Some(existing) = source.dom_evidence(&metadata.key) {
            ensure!(
                serde_json::to_vec(&existing.metadata)? == serde_json::to_vec(metadata)?,
                "Successful DOM ingestion slot is immutable"
            );
        }
        self.ingest_validated_dom_snapshot(input, metadata)
    }
    /// Import-only path after exact manifest provenance and fresh IDs have been
    /// validated. The isolated preparation store does not yet have run rows.
    pub(crate) fn ingest_validated_dom_snapshot(
        &mut self,
        input: impl Read,
        metadata: &DomCaptureMetadata,
    ) -> Result<DomCaptureEvidence> {
        metadata.validate()?;
        let mut bytes = Vec::new();
        input
            .take(MAX_DOM_BYTES as u64 + 1)
            .read_to_end(&mut bytes)?;
        metadata.verify_snapshot(&bytes)?;
        let artifact = self.put_artifact(bytes.as_slice(), ArtifactKind::DomSnapshot)?;
        ensure!(
            artifact.id == metadata.sha256 && artifact.bytes == metadata.bytes,
            "DOM ingestion identity mismatch"
        );
        Ok(DomCaptureEvidence {
            metadata: metadata.clone(),
        })
    }
    pub(crate) fn verify_dom_capture(&self, capture: &DomCaptureEvidence) -> Result<DomSnapshot> {
        let (artifact, bytes) =
            self.read_verified_artifact(&capture.metadata.sha256, MAX_DOM_BYTES as u64)?;
        ensure!(
            artifact.kind == ArtifactKind::DomSnapshot,
            "DOM artifact kind mismatch"
        );
        capture.metadata.verify_snapshot(&bytes)
    }
    pub fn read_dom_snapshot(
        &self,
        run: RunId,
        key: &TargetKey,
    ) -> Result<(DomCaptureEvidence, DomSnapshot)> {
        let record = self.load_run_snapshot(run)?;
        let capture = record
            .dom_evidence(key)
            .context("DOM evidence missing for this saved slot")?;
        capture.metadata.validate_for_run(&record.run)?;
        let snapshot = self.verify_dom_capture(capture)?;
        Ok((capture.clone(), snapshot))
    }
    pub fn load_dom_run(&self, run: RunId) -> Result<RunRecord> {
        let record = self.load_run_snapshot(run)?;
        for capture in &record.dom_captures {
            self.verify_dom_capture(capture)?;
        }
        Ok(record)
    }
    pub fn load_run_kind(&self, run: RunId, kind: CheckKind) -> Result<RunRecord> {
        match kind {
            CheckKind::Screenshot => self.load_screenshot_run(run),
            CheckKind::DomSnapshot => self.load_dom_run(run),
        }
    }
    pub fn load_dom_baseline(&self, id: BaselineId) -> Result<BaselineVersion> {
        let baseline = self.load_baseline_metadata(id)?;
        let frozen = self.load_run_snapshot(
            baseline
                .frozen_source_run()
                .context("Frozen baseline source missing")?,
        )?;
        for reference in &baseline.dom_references {
            let source = self.load_run_snapshot(reference.run)?;
            let capture = source
                .dom_evidence(&reference.key)
                .context("DOM baseline source missing")?;
            ensure!(
                source.run.project == baseline.project
                    && source.run.inventory.id == baseline.inventory
                    && source.run.inventory == frozen.run.inventory
                    && source.run.profiles == frozen.run.profiles
                    && source.run.rendering == frozen.run.rendering
                    && source.run.capture_plan == baseline.capture_plan
                    && source.run.finished_at == Some(reference.captured_at)
                    && capture.metadata.sha256 == reference.artifact
                    && capture.metadata.policy == reference.policy,
                "DOM baseline source binding mismatch"
            );
            ensure!(
                capture.metadata.baseline_ready(),
                "Saved DOM reference is not eligible stable evidence"
            );
            self.verify_dom_capture(capture)?;
        }
        Ok(baseline)
    }
    pub fn load_baseline_kind(&self, id: BaselineId, kind: CheckKind) -> Result<BaselineVersion> {
        match kind {
            CheckKind::Screenshot => self.load_screenshot_baseline(id),
            CheckKind::DomSnapshot => self.load_dom_baseline(id),
        }
    }
}
