// SPDX-License-Identifier: MPL-2.0
//! Focused, bounded DOM evidence. Aggregate evidence contains identities only.
use super::text::{self, DiffPage, DiffSummary, SourcePage};
use anyhow::{Context, Result, ensure};
use regressee_core::{dom_snapshot::*, domain::*, evidence::*};
use regressee_storage::Store;
use serde::{Deserialize, Serialize};
use std::sync::atomic::AtomicBool;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub project: ProjectId,
    pub run: Option<RunId>,
    pub analysis: Option<DomAnalysisId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindKeys<I> {
    pub analysis: I,
    pub selected: Vec<TargetKey>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct KindSelection {
    pub screenshot: Option<KindKeys<AnalysisId>>,
    pub dom: Option<KindKeys<DomAnalysisId>>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum OutcomeFilter {
    Changed,
    Unchanged,
    Failed,
    NotComparable,
    NotRequested,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "arguments", deny_unknown_fields)]
pub enum Command {
    Inspect {
        project: ProjectId,
        run: RunId,
    },
    Reanalyze {
        run: RunId,
        options: DomDiffOptions,
    },
    Results {
        selection: Selection,
        offset: usize,
        limit: usize,
        page: Option<PageId>,
        profile: Option<ProfileId>,
        outcome: Option<OutcomeFilter>,
        capture_state: Option<crate::terminal::CaptureFilter>,
    },
    Artifacts {
        selection: Selection,
        key: TargetKey,
    },
    Source {
        selection: Selection,
        key: TargetKey,
        pane: SourcePane,
        format: SourceFormat,
        offset: usize,
        limit: usize,
    },
    Diff {
        selection: Selection,
        key: TargetKey,
        options: DomDiffOptions,
        offset: usize,
        limit: usize,
    },
    Analysis {
        analysis: DomAnalysisId,
    },
    SelectAnalysis {
        analysis: DomAnalysisId,
    },
    Review(Box<DomReviewDecision>),
    SetReview {
        analysis: DomAnalysisId,
        key: TargetKey,
        state: ReviewState,
        note: String,
        reviewed_at: i64,
    },
    ReviewHistory {
        analysis: DomAnalysisId,
        key: Option<TargetKey>,
        before: Option<i64>,
        limit: usize,
    },
    Approval(KindSelection),
    Accept(
        #[serde(deserialize_with = "deserialize_kind_preview")]
        Box<regressee_storage::KindApprovalPreview>,
    ),
}
fn deserialize_kind_preview<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Box<regressee_storage::KindApprovalPreview>, D::Error> {
    let value = serde_json::Value::deserialize(d)?;
    let preview =
        serde_json::from_value::<Box<regressee_storage::KindApprovalPreview>>(value.clone())
            .map_err(|_| <D::Error as serde::de::Error>::custom("invalid kind approval preview"))?;
    let canonical =
        serde_json::to_value(&preview).map_err(<D::Error as serde::de::Error>::custom)?;
    if canonical != value {
        return Err(<D::Error as serde::de::Error>::custom(
            "kind approval preview has unknown/missing/changed versioned fields; regenerate it",
        ));
    }
    Ok(preview)
}
impl Command {
    pub fn writes(&self) -> bool {
        matches!(
            self,
            Self::SelectAnalysis { .. }
                | Self::Reanalyze { .. }
                | Self::Review(_)
                | Self::SetReview { .. }
                | Self::Accept(_)
        )
    }
    pub fn needs_confirmation(&self) -> bool {
        matches!(self, Self::Accept(_))
    }
    pub fn project(&self) -> Option<ProjectId> {
        match self {
            Self::Inspect { project, .. } => Some(*project),
            Self::Results { selection, .. }
            | Self::Artifacts { selection, .. }
            | Self::Source { selection, .. }
            | Self::Diff { selection, .. } => Some(selection.project),
            Self::Accept(p) => Some(p.baseline.project),
            _ => None,
        }
    }
    pub fn validate(&self) -> Result<()> {
        match self {
            Self::Reanalyze { options, .. } => options.validate()?,
            Self::Results { offset, limit, .. } => ensure!(
                *offset <= MAX_TARGETS && (1..=200).contains(limit),
                "invalid_request: DOM results limit 1–200; offset at most 30000"
            ),
            Self::Source { limit, offset, .. } => ensure!(
                (4..=text::MAX_SOURCE_PAGE_BYTES).contains(limit)
                    && *offset <= MAX_DOM_SOURCE_BYTES,
                "invalid_request: source page limit 4–262144 bytes"
            ),
            Self::Diff {
                options,
                limit,
                offset,
                ..
            } => {
                options.validate()?;
                ensure!(
                    (1..=text::MAX_DIFF_PAGE_ROWS).contains(limit)
                        && *offset <= MAX_DOM_DIFF_OUTPUT_LINES,
                    "invalid_request: diff page limit 1–1000 rows; offset at most 20000"
                );
            }
            Self::Review(d) => {
                validate_review_note(&d.note).map_err(anyhow::Error::msg)?;
            }
            Self::SetReview {
                note, reviewed_at, ..
            } => {
                validate_review_note(note).map_err(anyhow::Error::msg)?;
                ensure!(*reviewed_at > 0, "invalid_request: invalid review date");
            }
            Self::ReviewHistory { limit, before, .. } => ensure!(
                (1..=200).contains(limit) && before.is_none_or(|n| n > 0),
                "invalid_request: review history limit 1–200; before must be positive"
            ),
            Self::Approval(selection) => validate_kind_selection(selection)?,
            Self::Accept(preview) => validate_kind_selection(&selection_from_preview(preview))?,
            _ => {}
        }
        Ok(())
    }
}

fn validate_kind_selection(selection: &KindSelection) -> Result<()> {
    ensure!(
        selection.screenshot.is_some() || selection.dom.is_some(),
        "invalid_request: explicitly select at least one kind"
    );
    for keys in selection
        .screenshot
        .iter()
        .map(|k| &k.selected)
        .chain(selection.dom.iter().map(|k| &k.selected))
    {
        ensure!(
            !keys.is_empty()
                && keys.len() <= MAX_TARGETS
                && keys
                    .iter()
                    .all(|k| !k.page.0.is_nil() && !k.profile.0.is_nil())
                && keys.iter().collect::<std::collections::BTreeSet<_>>().len() == keys.len(),
            "invalid_request: selected kinds require unique nonempty target keys"
        );
    }
    ensure!(
        selection
            .screenshot
            .as_ref()
            .is_none_or(|s| !s.analysis.0.is_nil())
            && selection
                .dom
                .as_ref()
                .is_none_or(|s| !s.analysis.0.is_nil()),
        "invalid_request: selected analysis IDs must not be nil"
    );
    Ok(())
}
fn selection_from_preview(preview: &regressee_storage::KindApprovalPreview) -> KindSelection {
    KindSelection {
        screenshot: preview.screenshot.as_ref().map(|p| KindKeys {
            analysis: p.analysis,
            selected: p.selected.clone(),
        }),
        dom: preview.dom.as_ref().map(|p| KindKeys {
            analysis: p.analysis,
            selected: p.selected.clone(),
        }),
    }
}
pub(crate) fn approval(
    store: &Store,
    selection: &KindSelection,
) -> Result<regressee_storage::KindApprovalPreview> {
    validate_kind_selection(selection)?;
    store.approval_preview_kinds(
        selection.screenshot.as_ref().map(|s| s.analysis),
        selection
            .screenshot
            .as_ref()
            .map_or(&[], |s| s.selected.as_slice()),
        selection.dom.as_ref().map(|s| s.analysis),
        selection
            .dom
            .as_ref()
            .map_or(&[], |s| s.selected.as_slice()),
    )
}
pub(crate) fn validate_accept(
    store: &Store,
    preview: &regressee_storage::KindApprovalPreview,
) -> Result<()> {
    canonical_accept(store, preview).map(|_| ())
}
pub(crate) fn canonical_accept(
    store: &Store,
    preview: &regressee_storage::KindApprovalPreview,
) -> Result<regressee_storage::KindApprovalPreview> {
    let mut fresh = approval(store, &selection_from_preview(preview))
        .context("confirmation_conflict: kind approval is no longer eligible")?;
    // Stateless refresh carries only the new version's generated identity/date.
    // Every nested consent view must use that same identity and otherwise match
    // all freshly derived fields, not merely its storage eligibility context.
    fresh.baseline.id = preview.baseline.id;
    fresh.baseline.created_at = preview.baseline.created_at;
    if let Some(image) = &mut fresh.screenshot {
        image.baseline.id = preview.baseline.id;
        image.baseline.created_at = preview.baseline.created_at;
    }
    if let Some(dom) = &mut fresh.dom {
        dom.baseline.id = preview.baseline.id;
        dom.baseline.created_at = preview.baseline.created_at;
    }
    ensure!(
        serde_json::to_value(&fresh)? == serde_json::to_value(preview)?,
        "confirmation_conflict: approval consent fields, counts, nested baselines, identities or sources changed"
    );
    Ok(fresh)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultRow {
    pub target: CaptureTarget,
    pub result: Option<DomAnalysisResult>,
    pub diff_summary: Option<DiffSummary>,
    pub reviews: Vec<DomReviewDecision>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Results {
    pub selection: Selection,
    pub baseline: Option<BaselineId>,
    pub state: JobState,
    pub total: usize,
    pub offset: usize,
    pub next_offset: Option<usize>,
    pub rows: Vec<ResultRow>,
    pub coverage: DiscoveryCoverage,
    pub omitted: Vec<ReviewUnitKey>,
    pub reviews_semantics: String,
}
fn byte_bounded_results(mut page: Results) -> Result<Results> {
    let rows = std::mem::take(&mut page.rows);
    let mut bytes = text::json_size_within(&page, text::MAX_RESULT_PAGE_BYTES)
        .context("response_too_large: DOM coverage metadata exceeds 4 MiB; use focused artifacts/source reads")?
        .saturating_add(1024); // Cursor digits, commas and the outer enum envelope.
    for row in rows {
        let row_bytes = text::json_size_within(&row, text::MAX_RESULT_PAGE_BYTES)
            .context("response_too_large: one DOM result exceeds 4 MiB; inspect the exact target's artifacts/source")?;
        if bytes.saturating_add(row_bytes + 1) > text::MAX_RESULT_PAGE_BYTES {
            break;
        }
        bytes += row_bytes + 1;
        page.rows.push(row);
    }
    ensure!(
        !page.rows.is_empty() || page.offset >= page.total,
        "response_too_large: DOM result cannot fit the bounded page; inspect focused artifacts/source"
    );
    let end = page.offset.saturating_add(page.rows.len());
    page.next_offset = (end < page.total).then_some(end);
    ensure!(
        text::json_size_within(&page, text::MAX_RESULT_PAGE_BYTES).is_some(),
        "response_too_large: DOM results exceed their encoded byte budget"
    );
    Ok(page)
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", deny_unknown_fields)]
pub enum Output {
    Evidence(Box<ProjectEvidence>),
    Results(Results),
    Artifacts(Box<Artifacts>),
    Source(Source),
    Diff(Diff),
    Analysis(Box<DomComparisonEvidence>),
    Review(DomReviewDecision),
    ReviewHistory(regressee_storage::DomReviewHistoryPage),
    Approval(Box<regressee_storage::KindApprovalPreview>),
    Project(Box<regressee_storage::ProjectRecord>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SourcePane {
    Before,
    After,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SourceFormat {
    Canonical,
    Plaintext,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Original {
    pub source: DomSourceIdentity,
    pub artifact: Artifact,
    pub metadata: DomCaptureMetadata,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifacts {
    pub selection: Selection,
    pub key: TargetKey,
    pub baseline: Option<BaselineId>,
    pub target: CaptureTarget,
    pub before: Option<Original>,
    pub after: Option<Original>,
    pub result: Option<DomAnalysisResult>,
    pub diff_summary: Option<DiffSummary>,
    pub privacy: String,
    pub coverage: String,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Source {
    pub selection: Selection,
    pub key: TargetKey,
    pub pane: SourcePane,
    pub format: SourceFormat,
    pub source: DomSourceIdentity,
    pub page: SourcePage,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Diff {
    pub selection: Selection,
    pub key: TargetKey,
    pub before: DomSourceIdentity,
    pub after: DomSourceIdentity,
    pub outcome: DomComparisonOutcome,
    pub options: DomDiffOptions,
    pub page: DiffPage,
}

fn original(store: &Store, run: &RunRecord, capture: &DomCaptureEvidence) -> Result<Original> {
    let source = dom_source(&run.run, capture)?;
    let (artifact, bytes) = store.read_verified_artifact(&source.artifact, MAX_DOM_BYTES as u64)?;
    ensure!(
        artifact.kind == ArtifactKind::DomSnapshot,
        "invalid_artifact: expected DOM snapshot"
    );
    capture.metadata.verify_snapshot(&bytes)?;
    Ok(Original {
        source,
        artifact,
        metadata: capture.metadata.clone(),
    })
}

pub(crate) fn artifacts(
    store: &Store,
    selection: &Selection,
    e: &ProjectEvidence,
    key: &TargetKey,
) -> Result<Artifacts> {
    let run = e.latest.as_ref().context("not_found: no recorded run")?;
    ensure!(
        run.run.project == selection.project && selection.run == Some(run.run.id),
        "invalid_request: DOM run/project mismatch"
    );
    ensure!(
        run.run.capture_plan.mode.dom_snapshots(),
        "invalid_request: DOM snapshots were not requested for this run; capture a new DOM-enabled baseline first"
    );
    let target = run
        .run
        .targets
        .iter()
        .find(|t| &t.key == key)
        .context("not_found: target not in selected run")?
        .clone();
    let before = if let Some(reference) = e
        .baseline
        .as_ref()
        .and_then(|b| b.dom_references.iter().find(|r| &r.key == key))
    {
        let source = store.load_dom_run(reference.run)?;
        ensure!(
            source.run.project == selection.project,
            "invalid_artifact: foreign DOM reference source"
        );
        let capture = source
            .dom_evidence(key)
            .context("invalid_artifact: DOM reference evidence missing")?;
        ensure!(
            capture.metadata.sha256 == reference.artifact
                && capture.metadata.policy == reference.policy,
            "invalid_artifact: DOM baseline source mismatch"
        );
        Some(original(store, &source, capture)?)
    } else {
        None
    };
    let after = run
        .dom_evidence(key)
        .map(|c| original(store, run, c))
        .transpose()?;
    let mut warnings = vec![];
    if before.is_none() {
        warnings.push("Before DOM reference is absent or explicitly omitted; this is not comparable and cannot fill coverage through selective approval.".into());
    }
    if after.is_none() {
        warnings.push("After DOM capture is unavailable; inspect the actual per-kind outcome. No empty snapshot was substituted.".into());
    }
    let mut result = e
        .dom_comparison
        .as_ref()
        .and_then(|a| a.revision.results.iter().find(|r| &r.key == key))
        .cloned();
    let diff_summary = result
        .as_ref()
        .and_then(|r| r.diff.as_ref())
        .map(DiffSummary::from);
    if let Some(result) = &mut result {
        ensure!(
            result.before.as_ref() == before.as_ref().map(|o| &o.source)
                && result.after.as_ref() == after.as_ref().map(|o| &o.source),
            "invalid_artifact: DOM analysis/source identities differ"
        );
        result.diff = None;
    }
    Ok(Artifacts {
        selection: selection.clone(),
        key: key.clone(),
        baseline: e.baseline.as_ref().map(|b| b.id),
        target,
        before,
        after,
        result,
        diff_summary,
        privacy: DOM_PRIVACY_NOTICE.into(),
        coverage: DOM_COVERAGE_NOTICE.into(),
        warnings,
    })
}

pub(crate) fn read_original(store: &Store, original: &Original) -> Result<(Vec<u8>, DomSnapshot)> {
    let (artifact, bytes) =
        store.read_verified_artifact(&original.source.artifact, MAX_DOM_BYTES as u64)?;
    ensure!(
        artifact == original.artifact && artifact.kind == ArtifactKind::DomSnapshot,
        "invalid_artifact: DOM original changed"
    );
    let snapshot = original.metadata.verify_snapshot(&bytes)?;
    Ok((bytes, snapshot))
}

pub(crate) fn source(
    store: &Store,
    a: &Artifacts,
    pane: SourcePane,
    format: SourceFormat,
    offset: usize,
    limit: usize,
) -> Result<Source> {
    let original = match pane {
        SourcePane::Before => a.before.as_ref(),
        SourcePane::After => a.after.as_ref(),
    }
    .context("incomplete_capture: selected DOM original is unavailable")?;
    let (bytes, _) = read_original(store, original)?;
    let text = match format {
        SourceFormat::Canonical => {
            String::from_utf8(bytes).context("invalid_artifact: DOM UTF-8")?
        }
        SourceFormat::Plaintext => crate::dom_snapshot::original_source(
            store,
            &original.source.artifact,
            original.source.policy,
        )?,
    };
    Ok(Source {
        selection: a.selection.clone(),
        key: a.key.clone(),
        pane,
        format,
        source: original.source.clone(),
        page: text::source_page(&text, offset, limit)?,
    })
}

pub(crate) fn diff(
    store: &Store,
    a: &Artifacts,
    options: &DomDiffOptions,
    offset: usize,
    limit: usize,
    cancel: &AtomicBool,
) -> Result<Diff> {
    let before = a
        .before
        .as_ref()
        .context("incomplete_capture: Before DOM reference missing/omitted")?;
    let after = a
        .after
        .as_ref()
        .context("incomplete_capture: After DOM capture missing/failed")?;
    read_original(store, before)?;
    read_original(store, after)?;
    let result = DomAnalysisResult {
        key: a.key.clone(),
        before: Some(before.source.clone()),
        after: Some(after.source.clone()),
        outcome: if before.source.artifact == after.source.artifact {
            DomComparisonOutcome::Unchanged
        } else {
            DomComparisonOutcome::Changed
        },
        diff: None,
    };
    result.validate().context("incomplete_capture: DOM sources are unstable or have incompatible policy/runtime/material identities; inspect artifacts and originals")?;
    let diff = crate::dom_snapshot::compare_originals(
        store,
        &before.source.artifact,
        &after.source.artifact,
        before.source.policy,
        options,
        cancel,
    )?;
    Ok(Diff {
        selection: a.selection.clone(),
        key: a.key.clone(),
        before: before.source.clone(),
        after: after.source.clone(),
        outcome: result.outcome,
        options: options.clone(),
        page: text::diff_page(diff, offset, limit)?,
    })
}

pub(crate) fn evidence(
    store: &Store,
    selection: &Selection,
) -> Result<(Selection, ProjectEvidence)> {
    let evidence = if let Some(analysis) = selection.analysis {
        store.viewer_dom_evidence(selection.project, Some(analysis))?
    } else {
        let run = selection
            .run
            .or(store.latest_run_id(selection.project)?)
            .context("not_found: no recorded run")?;
        store.dom_evidence_for_run(selection.project, run)?
    };
    let run = evidence
        .latest
        .as_ref()
        .context("not_found: no recorded DOM run")?
        .run
        .id;
    ensure!(
        selection.run.is_none_or(|id| id == run),
        "invalid_request: DOM analysis/run mismatch"
    );
    ensure!(
        evidence
            .latest
            .as_ref()
            .is_some_and(|r| r.run.capture_plan.mode.dom_snapshots()),
        "invalid_request: DOM snapshots were not requested for this run; capture a new DOM-enabled baseline"
    );
    if let Some(analysis) = &evidence.dom_comparison {
        ensure!(
            analysis.revision.run == run
                && evidence
                    .baseline
                    .as_ref()
                    .is_some_and(|b| b.id == analysis.revision.baseline),
            "invalid_request: DOM analysis/run/original-baseline mismatch"
        );
    }
    let selection = Selection {
        project: selection.project,
        run: Some(run),
        analysis: evidence.dom_comparison.as_ref().map(|a| a.revision.id),
    };
    Ok((selection, evidence))
}

pub(crate) fn execute(store: &mut Store, command: Command, cancel: &AtomicBool) -> Result<Output> {
    command.validate()?;
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return Err(crate::analysis_engine::Cancelled.into());
    }
    let select_analysis = matches!(&command, Command::SelectAnalysis { .. });
    Ok(match command {
        Command::Inspect { project, run } => {
            let (_, mut e) = evidence(
                store,
                &Selection {
                    project,
                    run: Some(run),
                    analysis: None,
                },
            )?;
            crate::protocol::strip_dom_diff_bodies(&mut e);
            Output::Evidence(Box::new(e))
        }
        Command::Reanalyze { run, options } => {
            let mut e = crate::dom_snapshot::analyze_run_with_options(store, run, options, cancel)?;
            for result in &mut e.revision.results {
                result.diff = None;
            }
            Output::Analysis(Box::new(e))
        }
        Command::Artifacts { selection, key } => {
            let (selection, e) = evidence(store, &selection)?;
            Output::Artifacts(Box::new(artifacts(store, &selection, &e, &key)?))
        }
        Command::Source {
            selection,
            key,
            pane,
            format,
            offset,
            limit,
        } => {
            let (selection, e) = evidence(store, &selection)?;
            Output::Source(source(
                store,
                &artifacts(store, &selection, &e, &key)?,
                pane,
                format,
                offset,
                limit,
            )?)
        }
        Command::Diff {
            selection,
            key,
            options,
            offset,
            limit,
        } => {
            let (selection, e) = evidence(store, &selection)?;
            Output::Diff(diff(
                store,
                &artifacts(store, &selection, &e, &key)?,
                &options,
                offset,
                limit,
                cancel,
            )?)
        }
        Command::Analysis { analysis } | Command::SelectAnalysis { analysis } => {
            if select_analysis {
                store.display_dom_analysis(analysis)?;
            }
            let mut e = store.load_dom_analysis(analysis)?;
            for result in &mut e.revision.results {
                result.diff = None;
            }
            Output::Analysis(Box::new(e))
        }
        Command::Review(decision) => {
            store.save_dom_review(&decision)?;
            Output::Review(*decision)
        }
        Command::SetReview {
            analysis,
            key,
            state,
            note,
            reviewed_at,
        } => {
            let evidence = store.load_dom_analysis(analysis)?;
            let result = evidence
                .revision
                .results
                .iter()
                .find(|r| r.key == key)
                .context("not_found: DOM review target absent")?;
            let decision = DomReviewDecision {
                analysis,
                key,
                before: result
                    .before
                    .clone()
                    .context("incomplete_capture: DOM Before missing")?,
                after: result
                    .after
                    .clone()
                    .context("incomplete_capture: DOM After missing")?,
                state,
                note,
                reviewed_at,
            };
            store.save_dom_review(&decision)?;
            Output::Review(decision)
        }
        Command::ReviewHistory {
            analysis,
            key,
            before,
            limit,
        } => Output::ReviewHistory(store.dom_review_history_page(analysis, key, before, limit)?),
        Command::Approval(selection) => Output::Approval(Box::new(approval(store, &selection)?)),
        Command::Accept(preview) => {
            validate_accept(store, &preview)?;
            Output::Project(Box::new(store.accept_selected_kinds(&preview)?))
        }
        Command::Results {
            selection,
            offset,
            limit,
            page,
            profile,
            outcome,
            capture_state,
        } => {
            let (selection, e) = evidence(store, &selection)?;
            let run = &e.latest.as_ref().unwrap().run;
            let indexed = e
                .dom_comparison
                .as_ref()
                .map(|a| {
                    a.revision
                        .results
                        .iter()
                        .map(|r| (&r.key, r))
                        .collect::<std::collections::BTreeMap<_, _>>()
                })
                .unwrap_or_default();
            let all = run
                .targets
                .iter()
                .filter_map(|target| {
                    let result = indexed.get(&target.key).copied();
                    let matches_outcome = outcome.is_none_or(|filter| {
                        matches!(
                            (filter, result.map(|r| &r.outcome)),
                            (OutcomeFilter::Changed, Some(DomComparisonOutcome::Changed))
                                | (
                                    OutcomeFilter::Unchanged,
                                    Some(DomComparisonOutcome::Unchanged)
                                )
                                | (
                                    OutcomeFilter::Failed,
                                    Some(DomComparisonOutcome::CouldNotCapture { .. })
                                )
                                | (
                                    OutcomeFilter::NotComparable,
                                    Some(DomComparisonOutcome::NotComparable { .. })
                                )
                                | (
                                    OutcomeFilter::NotRequested,
                                    Some(DomComparisonOutcome::NotRequested)
                                )
                        )
                    });
                    let matches_capture = capture_state.is_none_or(|filter| {
                        matches!(
                            (filter, &target.dom_outcome),
                            (
                                crate::terminal::CaptureFilter::Pending,
                                DomTargetOutcome::Pending
                            ) | (
                                crate::terminal::CaptureFilter::Captured,
                                DomTargetOutcome::Captured { .. }
                            ) | (
                                crate::terminal::CaptureFilter::Failed,
                                DomTargetOutcome::Failed { .. }
                            ) | (
                                crate::terminal::CaptureFilter::Skipped,
                                DomTargetOutcome::Skipped { .. }
                            ) | (
                                crate::terminal::CaptureFilter::Cancelled,
                                DomTargetOutcome::Cancelled
                            )
                        )
                    });
                    (page.is_none_or(|p| p == target.key.page)
                        && profile.is_none_or(|p| p == target.key.profile)
                        && matches_outcome
                        && matches_capture)
                        .then(|| ResultRow {
                            target: target.clone(),
                            diff_summary: result
                                .and_then(|r| r.diff.as_ref())
                                .map(DiffSummary::from),
                            result: result.cloned().map(|mut r| {
                                r.diff = None;
                                r
                            }),
                            reviews: vec![],
                        })
                })
                .collect::<Vec<_>>();
            let total = all.len();
            let mut rows = all.into_iter().skip(offset).take(limit).collect::<Vec<_>>();
            if let Some(analysis) = selection.analysis {
                let keys = rows
                    .iter()
                    .map(|r| r.target.key.clone())
                    .collect::<Vec<_>>();
                for decision in store.current_dom_reviews(analysis, &keys)? {
                    if let Some(row) = rows.iter_mut().find(|r| r.target.key == decision.key) {
                        row.reviews.push(decision);
                    }
                }
            }
            let omitted = if let Some(baseline) = &e.baseline {
                let (baseline, frozen) = store.frozen_baseline_source(baseline.id)?;
                baseline.omitted_units(&frozen.run)?
            } else {
                vec![]
            };
            Output::Results(byte_bounded_results(Results { selection, baseline: e.baseline.as_ref().map(|b| b.id), state: run.state, total, offset, next_offset: (offset.saturating_add(limit) < total).then_some(offset + limit), rows, coverage: run.inventory.coverage.clone(), omitted, reviews_semantics: "Zero or one authoritative current decision for this DOM analysis/target. History is separately paged; screenshot decisions never review DOM evidence.".into() })?)
        }
    })
}

#[cfg(test)]
pub(crate) fn maximal_warning_results(count: usize) -> Results {
    let project = Project::draft(
        "Owned response budget fixture".into(),
        "http://127.0.0.1:43210/fixture",
    )
    .unwrap();
    let rows = (0..count)
        .map(|index| {
            let mut requested = project.sources[0].clone();
            requested.set_query(Some(&format!("page={index}")));
            ResultRow {
                target: CaptureTarget {
                    key: TargetKey {
                        page: PageId::new(),
                        profile: project.profiles[0].id,
                    },
                    requested,
                    final_url: None,
                    attempts: 0,
                    outcome: TargetOutcome::NotRequested,
                    dom_outcome: DomTargetOutcome::Pending,
                    warnings: vec!["\0".repeat(1024); 64],
                },
                result: None,
                diff_summary: None,
                reviews: vec![],
            }
        })
        .collect();
    Results {
        selection: Selection {
            project: project.id,
            run: Some(RunId::new()),
            analysis: None,
        },
        baseline: None,
        state: JobState::Running,
        total: count,
        offset: 0,
        next_offset: None,
        rows,
        coverage: DiscoveryCoverage::Complete,
        omitted: vec![],
        reviews_semantics: "Authoritative current decisions; separately paged history".into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn escape_heavy_valid_warning_rows_are_byte_paged_with_a_forward_cursor() -> Result<()> {
        let input = maximal_warning_results(169);
        assert!(text::json_size_within(&input, crate::protocol::MAX_READ_RESULT_BYTES).is_none());
        let page = byte_bounded_results(input)?;
        assert!(!page.rows.is_empty() && page.rows.len() < 169);
        assert_eq!(page.total, 169);
        assert_eq!(page.next_offset, Some(page.rows.len()));
        assert!(text::json_size_within(&page, text::MAX_RESULT_PAGE_BYTES).is_some());
        let mut next = maximal_warning_results(169 - page.rows.len());
        next.offset = page.rows.len();
        next.total = 169;
        let next = byte_bounded_results(next)?;
        assert!(
            next.next_offset
                .is_some_and(|cursor| cursor > page.next_offset.unwrap())
        );
        Ok(())
    }

    #[test]
    fn focused_requests_and_kind_selections_reject_unknown_or_implicit_scope() -> Result<()> {
        let command = Command::Source {
            selection: Selection {
                project: ProjectId::new(),
                run: Some(RunId::new()),
                analysis: None,
            },
            key: TargetKey {
                page: PageId::new(),
                profile: ProfileId::new(),
            },
            pane: SourcePane::After,
            format: SourceFormat::Canonical,
            offset: 0,
            limit: 4096,
        };
        command.validate()?;
        let mut value = serde_json::to_value(command)?;
        value["arguments"]["script"] = "untrusted active request".into();
        assert!(serde_json::from_value::<Command>(value).is_err());
        assert!(
            validate_kind_selection(&KindSelection {
                screenshot: None,
                dom: None
            })
            .is_err()
        );
        assert!(
            validate_kind_selection(&KindSelection {
                screenshot: None,
                dom: Some(KindKeys {
                    analysis: DomAnalysisId::new(),
                    selected: vec![]
                })
            })
            .is_err()
        );
        assert!(
            serde_json::from_value::<KindSelection>(
                serde_json::json!({"screenshot":null,"dom":null,"approve_all":true})
            )
            .is_err()
        );
        Ok(())
    }
}
