// SPDX-License-Identifier: MPL-2.0
//! Engine-owned terminal inspection and local evidence export. No client SQLite.
use crate::engine::dom;
use anyhow::{Context, Result, ensure};
#[cfg(test)]
use regressee_core::dom_snapshot::diff_text;
use regressee_core::dom_snapshot::{
    DOM_COVERAGE_NOTICE, DOM_PRIVACY_NOTICE, DomDiffOptions, DomDiffRowKind, DomReviewDecision,
};
use regressee_core::{domain::*, evidence::*};
use regressee_storage::{Store, images};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub project: ProjectId,
    pub run: Option<RunId>,
    pub analysis: Option<AnalysisId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "arguments", deny_unknown_fields)]
pub enum Command {
    Analysis {
        analysis: AnalysisId,
    },
    ReviewHistory {
        analysis: AnalysisId,
        key: Option<TargetKey>,
        before: Option<i64>,
        limit: usize,
    },
    InventoryEdit {
        preview: Box<regressee_core::crawl::DiscoveryPreview>,
        pages: Vec<PageId>,
        included: bool,
    },
    Results {
        selection: Selection,
        offset: usize,
        limit: usize,
        page: Option<PageId>,
        profile: Option<ProfileId>,
        classification: Option<Classification>,
        capture_state: Option<CaptureFilter>,
    },
    Artifacts {
        selection: Selection,
        key: TargetKey,
    },
    CropPreview {
        selection: Selection,
        key: TargetKey,
        pane: Pane,
        rectangle: Rectangle,
        destination: PathBuf,
    },
    Crop(Box<CropPreview>),
    ReportPreview {
        selection: Selection,
        #[serde(default)]
        dom_analysis: Option<DomAnalysisId>,
        selected: Vec<TargetKey>,
        destination: PathBuf,
    },
    Report(Box<ReportPreview>),
}
impl Command {
    pub fn project(&self) -> Option<ProjectId> {
        match self {
            Self::Analysis { .. } | Self::ReviewHistory { .. } => None,
            Self::InventoryEdit { preview, .. } => Some(preview.settings.id),
            Self::Results { selection, .. }
            | Self::Artifacts { selection, .. }
            | Self::CropPreview { selection, .. }
            | Self::ReportPreview { selection, .. } => Some(selection.project),
            Self::Crop(p) => Some(p.selection.project),
            Self::Report(p) => Some(p.selection.project),
        }
    }
    pub fn writes(&self) -> bool {
        matches!(self, Self::Crop(_) | Self::Report(_))
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Pane {
    Before,
    After,
    Difference,
}
impl Pane {
    fn caption(self) -> &'static str {
        match self {
            Self::Before => "Before — original screenshot",
            Self::After => "After — original screenshot",
            Self::Difference => "Difference — derived analysis",
        }
    }
    fn source_label(self) -> &'static str {
        match self {
            Self::Before => "original before screenshot",
            Self::After => "original after screenshot",
            Self::Difference => "derived difference analysis",
        }
    }
    fn link_label(self) -> &'static str {
        match self {
            Self::Before => "Original before screenshot PNG",
            Self::After => "Original after screenshot PNG",
            Self::Difference => "Derived difference PNG",
        }
    }
}
const EVIDENCE_LABELS: &str = "Before/After are original screenshots; differences are derived analysis. Crops are derived evidence exported separately, not included in this report. Image bytes are copied unaltered.";
const SHARING_WARNING: &str = "Ignored areas are not redaction: original screenshot pixels remain visible, including private content. Before sharing, review every image, full URL/query string, note and recorded local path; share only with authorized recipients. Nothing is uploaded automatically.";
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum CaptureFilter {
    Pending,
    Captured,
    Failed,
    Skipped,
    Cancelled,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Image {
    pub artifact: Artifact,
    pub original_path: PathBuf,
    pub width: u32,
    pub height: u32,
    pub screenshot_pixels_per_css_pixel: Option<f32>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Artifacts {
    pub selection: Selection,
    pub key: TargetKey,
    pub baseline: Option<BaselineId>,
    pub before_run: Option<RunId>,
    pub before: Option<Image>,
    pub after: Option<Image>,
    pub difference: Option<Image>,
    pub target: CaptureTarget,
    pub before_capture: Option<CaptureMetadata>,
    pub after_capture: Option<CaptureMetadata>,
    pub analysis_report: Option<regressee_core::analysis_engine::AnalysisReport>,
    pub warnings: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Results {
    pub selection: Selection,
    pub baseline: Option<BaselineId>,
    pub inventory: InventoryId,
    pub coverage: DiscoveryCoverage,
    pub state: JobState,
    pub total: usize,
    pub offset: usize,
    pub next_offset: Option<usize>,
    pub rows: Vec<ResultRow>,
    #[serde(default)]
    pub reviews_semantics: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResultRow {
    pub target: CaptureTarget,
    pub classification: Option<Classification>,
    pub analysis_result: Option<AnalysisResult>,
    pub reviews: Vec<ReviewDecision>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CropPreview {
    pub selection: Selection,
    pub key: TargetKey,
    pub pane: Pane,
    /// Always physical screenshot pixels. No implicit CSS conversion.
    pub rectangle: Rectangle,
    pub coordinate_space: String,
    pub source: Image,
    pub destination: PathBuf,
    pub privacy: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportPreview {
    pub selection: Selection,
    pub selected: Vec<TargetKey>,
    pub destination: PathBuf,
    pub revision: (u64, u64),
    pub fingerprint: String,
    pub bytes: u64,
    #[serde(default)]
    pub maximum_output_bytes: u64,
    pub privacy: String,
    pub content: ReportContent,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportContent {
    pub run: CaptureRun,
    #[serde(default)]
    pub queue: Option<QueueSettings>,
    #[serde(default)]
    pub reference_settings: Vec<ReferenceSettings>,
    pub baseline: Option<BaselineVersion>,
    pub analysis: Option<ComparisonEvidence>,
    pub rows: Vec<ResultRow>,
    pub artifacts: Vec<Artifacts>,
    #[serde(default)]
    pub dom_artifacts: Vec<dom::Artifacts>,
    #[serde(default)]
    pub dom_analysis: Option<DomComparisonEvidence>,
    #[serde(default)]
    pub dom_reviews: Vec<DomReviewDecision>,
    #[serde(default)]
    pub dom_diff_options: DomDiffOptions,
    #[serde(default)]
    pub dom_diff_algorithm: String,
    pub limitations: Vec<String>,
    #[serde(default)]
    pub reviews_semantics: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReferenceSettings {
    pub run: RunId,
    #[serde(default)]
    pub capture_plan: CapturePlan,
    pub queue: Option<QueueSettings>,
    pub profiles: Vec<CaptureProfile>,
    pub environment: Environment,
    pub rendering: RenderingEnvironment,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value")]
pub enum Output {
    Analysis(Box<ComparisonEvidence>),
    ReviewHistory(regressee_storage::ReviewHistoryPage),
    Inventory(Box<regressee_core::crawl::DiscoveryPreview>),
    Results(Results),
    Artifacts(Box<Artifacts>),
    CropPreview(CropPreview),
    Cropped {
        path: PathBuf,
        source: Image,
        rectangle: Rectangle,
        coordinate_space: String,
        derived: bool,
    },
    ReportPreview(Box<ReportPreview>),
    Reported {
        html: PathBuf,
        markdown: PathBuf,
    },
}

fn evidence(
    store: &Store,
    selection: &Selection,
    include_dom: bool,
) -> Result<(Selection, ProjectEvidence)> {
    // Resolve one immutable identity first, including an explicitly chosen older
    // analysis. Never borrow a workspace's current active baseline for a check.
    let chosen = selection
        .analysis
        .map(|id| store.load_analysis(id))
        .transpose()?;
    let run_id = selection
        .run
        .or(chosen.as_ref().map(|a| a.revision.run))
        .or(store.latest_run_id(selection.project)?)
        .context("not_found: no recorded run")?;
    let mut e = if include_dom {
        store.project_evidence_for_run(selection.project, run_id)?
    } else if selection.analysis.is_some() {
        store.viewer_evidence(selection.project, selection.analysis)?
    } else {
        store.screenshot_evidence_for_run(selection.project, run_id)?
    };
    let run = e.latest.as_ref().context("not_found: no recorded run")?;
    ensure!(
        run.run.id == run_id,
        "invalid_request: analysis/run mismatch"
    );
    if !matches!(run.run.intent, RunIntent::Check { .. }) {
        e.baseline = None;
        e.before = None;
    }
    if let Some(a) = chosen {
        ensure!(
            a.revision.run == run.run.id
                && e.baseline
                    .as_ref()
                    .is_some_and(|b| b.id == a.revision.baseline),
            "invalid_request: analysis does not belong to the selected run/baseline"
        );
        e.comparison = Some(a);
    }
    if let RunIntent::Check { baseline } = run.run.intent {
        ensure!(
            e.baseline
                .as_ref()
                .is_some_and(|b| b.id == baseline && b.project == selection.project),
            "invalid_artifact: original check reference is missing/mismatched"
        );
    }
    ensure!(
        e.comparison
            .as_ref()
            .is_none_or(|a| a.revision.run == run_id
                && e.baseline
                    .as_ref()
                    .is_some_and(|b| b.id == a.revision.baseline)),
        "invalid_artifact: saved analysis/run/reference mismatch"
    );
    let selection = Selection {
        project: selection.project,
        run: Some(run.run.id),
        analysis: e.comparison.as_ref().map(|a| a.revision.id),
    };
    Ok((selection, e))
}
fn rows(e: &ProjectEvidence) -> Vec<ResultRow> {
    let classifications = e
        .comparison
        .as_ref()
        .map(|a| a.review_classifications())
        .unwrap_or_default();
    e.latest
        .as_ref()
        .into_iter()
        .flat_map(|r| r.run.targets.iter())
        .map(|t| ResultRow {
            target: t.clone(),
            classification: classifications.get(&t.key).copied(),
            analysis_result: e
                .comparison
                .as_ref()
                .and_then(|a| a.revision.results.iter().find(|r| r.key == t.key))
                .cloned(),
            reviews: vec![],
        })
        .collect()
}
const REVIEWS_SEMANTICS: &str = "Authoritative current decision per selected analysis/target (zero or one per row); not a history window. Use review history for bounded keyset pagination of all recorded decisions.";
fn attach_reviews(store: &Store, e: &ProjectEvidence, rows: &mut [ResultRow]) -> Result<()> {
    if let Some(a) = &e.comparison {
        let keys = rows
            .iter()
            .map(|r| r.target.key.clone())
            .collect::<Vec<_>>();
        for decision in store.current_reviews(a.revision.id, &keys)? {
            if let Some(row) = rows.iter_mut().find(|r| r.target.key == decision.key) {
                row.reviews.push(decision);
            }
        }
    }
    Ok(())
}
fn image(store: &Store, image: &ImageEvidence, scale: Option<f32>) -> Result<Image> {
    let artifact = store.verify_artifact(&image.original)?;
    ensure!(
        matches!(
            artifact.kind,
            ArtifactKind::Screenshot | ArtifactKind::Difference
        ),
        "invalid_artifact: expected original PNG evidence"
    );
    Ok(Image {
        artifact,
        original_path: store.artifact_path(&image.original)?,
        width: image.width,
        height: image.height,
        screenshot_pixels_per_css_pixel: scale,
    })
}
fn artifacts(
    store: &Store,
    selection: &Selection,
    e: &ProjectEvidence,
    key: &TargetKey,
) -> Result<Artifacts> {
    let run = e.latest.as_ref().context("not_found: no run")?;
    ensure!(
        run.run.capture_plan.mode.screenshots(),
        "invalid_request: this run requested DOM snapshots only; use --kind dom or snapshot source/diff. No screenshot was captured"
    );
    let target = run
        .run
        .targets
        .iter()
        .find(|t| &t.key == key)
        .context("not_found: target not in run")?
        .clone();
    let before = if let Some(reference) = e
        .baseline
        .as_ref()
        .and_then(|b| b.references.iter().find(|r| &r.key == key))
    {
        let source = store.load_screenshot_run(reference.run)?;
        ensure!(
            source.run.project == selection.project,
            "invalid_artifact: foreign baseline source"
        );
        let c = source
            .evidence(key)
            .context("invalid_artifact: baseline source missing")?
            .clone();
        ensure!(
            c.image.original == reference.artifact,
            "invalid_artifact: baseline reference mismatch"
        );
        Some(c)
    } else {
        None
    };
    let after = run.evidence(key);
    let diff = e.comparison.as_ref().and_then(|a| {
        a.heatmaps
            .iter()
            .find(|(k, _)| k == key)
            .map(|(_, i)| i)
            .or({
                if run.run.targets.len() == 1 {
                    a.heatmap.as_ref()
                } else {
                    None
                }
            })
    });
    let report = e
        .comparison
        .as_ref()
        .and_then(|a| a.perceptual.as_ref())
        .and_then(|p| p.reports.iter().find(|(k, _)| k == key))
        .map(|(_, r)| r.clone());
    let mut warnings=vec!["Coordinates are screenshot pixels; CSS coordinates require the recorded device scale. Regions are bounded heuristics, not exhaustive coverage.".into()];
    if before.is_none() {
        warnings.push("Before image unavailable; no substitute image was created.".into());
    }
    if after.is_none() {
        warnings.push("Capture missing/failed; this is not an empty-image success.".into());
    }
    if diff.is_none() {
        warnings.push(
            "Difference image unavailable; inspect capture and comparability outcomes.".into(),
        );
    }
    Ok(Artifacts {
        selection: selection.clone(),
        key: key.clone(),
        baseline: e.baseline.as_ref().map(|b| b.id),
        before_run: e
            .baseline
            .as_ref()
            .and_then(|b| b.references.iter().find(|r| &r.key == key))
            .map(|r| r.run),
        before: before
            .as_ref()
            .map(|c| image(store, &c.image, Some(c.metadata.profile.dpr)))
            .transpose()?,
        after: after
            .map(|c| image(store, &c.image, Some(c.metadata.profile.dpr)))
            .transpose()?,
        difference: diff.map(|i| image(store, i, None)).transpose()?,
        target,
        before_capture: before.map(|c| c.metadata),
        after_capture: after.map(|c| c.metadata.clone()),
        analysis_report: report,
        warnings,
    })
}
/// Resolve an exact new local destination, reject traversal/symlink parents and all
/// engine-owned roots. Never overwrite an original or an existing export.
fn destination(store: &Store, path: &Path) -> Result<PathBuf> {
    ensure!(
        path.is_absolute()
            && path.components().all(|c| !matches!(
                c,
                std::path::Component::ParentDir | std::path::Component::CurDir
            )),
        "invalid_path: use an absolute destination without dot components"
    );
    ensure!(
        fs::symlink_metadata(path).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound),
        "invalid_path: destination must not exist"
    );
    let parent = path
        .parent()
        .context("invalid_path: destination parent missing")?;
    let canonical = fs::canonicalize(parent)?;
    ensure!(
        canonical == parent,
        "invalid_path: destination parent must be canonical (no symlink aliases)"
    );
    ensure!(
        !path.starts_with(&store.paths().data) && !path.starts_with(&store.paths().cache),
        "invalid_path: output must be outside engine data/cache"
    );
    ensure!(
        fs::metadata(parent)?.is_dir(),
        "invalid_path: parent is not a directory"
    );
    Ok(path.to_path_buf())
}
fn crop_preview(
    store: &Store,
    selection: &Selection,
    key: &TargetKey,
    pane: Pane,
    rectangle: Rectangle,
    path: &Path,
) -> Result<CropPreview> {
    let (selection, e) = evidence(store, selection, false)?;
    let a = artifacts(store, &selection, &e, key)?;
    let source = match pane {
        Pane::Before => a.before,
        Pane::After => a.after,
        Pane::Difference => a.difference,
    }
    .context("incomplete_capture: selected image is unavailable")?;
    ensure!(
        rectangle.width > 0
            && rectangle.height > 0
            && rectangle
                .x
                .checked_add(rectangle.width)
                .is_some_and(|n| n <= source.width)
            && rectangle
                .y
                .checked_add(rectangle.height)
                .is_some_and(|n| n <= source.height)
            && u64::from(rectangle.width) * u64::from(rectangle.height) <= 4_000_000,
        "invalid_request: crop must fit source and be at most 4M pixels"
    );
    Ok(CropPreview {
        selection,
        key: key.clone(),
        pane,
        rectangle,
        coordinate_space: "screenshot_pixels".into(),
        source,
        destination: destination(store, path)?,
        privacy: format!(
            "Derived crop of {}; copies selected pixels to the exact local destination. Ignored areas are not redaction. May contain private page content. No credentials/session files are included; nothing is uploaded.",
            pane.source_label()
        ),
    })
}
fn report_preview(
    store: &Store,
    selection: &Selection,
    dom_analysis: Option<DomAnalysisId>,
    selected: &[TargetKey],
    path: &Path,
) -> Result<ReportPreview> {
    ensure!(
        !selected.is_empty()
            && selected.len() <= 100
            && selected
                .iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len()
                == selected.len(),
        "invalid_request: select 1–100 unique targets"
    );
    let chosen_dom = dom_analysis
        .map(|id| store.load_dom_analysis(id))
        .transpose()?;
    let mut selected_run = selection.clone();
    if selected_run.run.is_none() && selected_run.analysis.is_none() {
        selected_run.run = chosen_dom.as_ref().map(|a| a.revision.run);
    }
    let (selection, mut e) = evidence(store, &selected_run, true)?;
    if let Some(a) = chosen_dom {
        ensure!(
            selection.run == Some(a.revision.run)
                && e.baseline
                    .as_ref()
                    .is_some_and(|b| b.id == a.revision.baseline),
            "invalid_request: report DOM analysis/run/reference mismatch"
        );
        e.dom_comparison = Some(a);
    }
    let mut a = Vec::new();
    let mut dom_artifacts = Vec::new();
    let mut bytes = 0u64;
    let mut dom_bytes = 0u64;
    let record = e.latest.as_ref().unwrap();
    for key in selected {
        if record.run.capture_plan.mode.screenshots() {
            let entry = artifacts(store, &selection, &e, key)?;
            bytes += entry
                .before
                .iter()
                .chain(entry.after.iter())
                .chain(entry.difference.iter())
                .map(|i| i.artifact.bytes)
                .sum::<u64>();
            ensure!(
                bytes <= 256 * 1024 * 1024,
                "invalid_request: report images exceed 256 MiB"
            );
            a.push(entry);
        }
        if record.run.capture_plan.mode.dom_snapshots() {
            let selection = dom::Selection {
                project: selection.project,
                run: selection.run,
                analysis: e.dom_comparison.as_ref().map(|a| a.revision.id),
            };
            let entry = dom::artifacts(store, &selection, &e, key)?;
            for original in entry.before.iter().chain(entry.after.iter()) {
                let (canonical, _) = dom::read_original(store, original)?;
                dom_bytes += canonical.len() as u64
                    + crate::dom_snapshot::original_source(
                        store,
                        &original.source.artifact,
                        original.source.policy,
                    )?
                    .len() as u64;
            }
            ensure!(
                dom_bytes <= 64 * 1024 * 1024,
                "invalid_request: report DOM originals exceed 64 MiB; select fewer targets"
            );
            dom_artifacts.push(entry);
        }
    }
    bytes += dom_bytes;
    ensure!(
        bytes <= 256 * 1024 * 1024,
        "invalid_request: report original copies exceed 256 MiB"
    );
    let mut analysis = e.comparison.clone();
    if let Some(a) = &mut analysis {
        a.revision.results.retain(|r| selected.contains(&r.key));
        a.heatmaps.retain(|(k, _)| selected.contains(k));
        a.heatmap = None;
        if let Some(p) = &mut a.perceptual {
            p.reports.retain(|(k, _)| selected.contains(k));
            p.failures.retain(|(k, _)| selected.contains(k));
        }
    }
    let mut dom_analysis = e.dom_comparison.clone();
    if let Some(a) = &mut dom_analysis {
        a.revision.results.retain(|r| selected.contains(&r.key));
        for result in &mut a.revision.results {
            result.diff = None;
        }
    }
    // Overall coverage/settings and excluded page scope are intentional previewed
    // content; only selected target screenshots, reports and notes are copied.
    let mut report_rows = rows(&e)
        .into_iter()
        .filter(|r| selected.contains(&r.target.key))
        .collect::<Vec<_>>();
    attach_reviews(store, &e, &mut report_rows)?;
    let mut reference_settings = Vec::new();
    if let Some(baseline) = &e.baseline {
        let sources = baseline
            .references
            .iter()
            .filter(|r| selected.contains(&r.key))
            .map(|r| r.run)
            .chain(
                baseline
                    .dom_references
                    .iter()
                    .filter(|r| selected.contains(&r.key))
                    .map(|r| r.run),
            )
            .collect::<std::collections::BTreeSet<_>>();
        for id in sources {
            let source = store.load_run(id)?;
            reference_settings.push(ReferenceSettings {
                run: id,
                capture_plan: source.run.capture_plan,
                queue: source.queue,
                profiles: source.run.profiles,
                environment: source.run.environment,
                rendering: source.run.rendering,
            });
        }
    }
    let mut limitations=vec!["Visual differences are observations, not confirmed bugs. Discovery does not establish whole-site coverage.".into(),"Failed, pending and noncomparable captures are not passes. Review recorded warnings and rendering/profile settings.".into(),"Ignored rectangles and region heuristics are in screenshot pixels. Original PNG bytes are copied unaltered; no remote resources or executable page markup are included.".into()];
    if record.queue.is_none() {
        limitations.push("Unknown legacy capture queue/discovery settings, project revision and resume timestamps; no current project settings were substituted.".into());
    }
    for r in &reference_settings {
        if r.queue.is_none() {
            limitations.push(format!("Unknown legacy reference queue/discovery settings for run {}; no current project settings were substituted.",r.run));
        }
    }
    let content = ReportContent {
        run: record.run.clone(),
        queue: record.queue.clone(),
        reference_settings,
        baseline: e.baseline.clone(),
        analysis,
        rows: report_rows,
        artifacts: a,
        dom_artifacts,
        dom_analysis,
        dom_reviews: e
            .dom_comparison
            .as_ref()
            .map(|a| store.current_dom_reviews(a.revision.id, selected))
            .transpose()?
            .unwrap_or_default(),
        dom_diff_options: DomDiffOptions {
            max_output_bytes: 16 * 1024,
            max_output_lines: 500,
            ..Default::default()
        },
        dom_diff_algorithm: regressee_core::dom_snapshot::DOM_DIFF_ALGORITHM.into(),
        limitations,
        reviews_semantics: REVIEWS_SEMANTICS.into(),
    };
    ensure!(
        serde_json::to_vec(&content)?.len() <= 8 * 1024 * 1024,
        "invalid_request: report metadata exceeds 8 MiB"
    );
    Ok(ReportPreview {
        selection,
        selected: selected.to_vec(),
        destination: destination(store, path)?,
        revision: store.viewer_revision()?,
        fingerprint: identity_hash(&content)?,
        bytes,
        maximum_output_bytes: bytes + 72 * 1024 * 1024,
        privacy: format!(
            "LOCAL ONLY. Includes URLs (including query strings), selected current notes/screenshots/normalized DOM originals and bounded derived DOM diffs, original local image artifact paths (which may reveal usernames), recorded capture and reference settings, discovery/scope exclusions, robots authorization, pacing/retries, project revisions/resume timestamps, inventory including excluded URLs, independent per-kind baseline provenance/policies and ignored regions. These can expose private page content. No cookies, credentials, sessions, network configuration or logs. Review this entire preview before confirming the exact destination. Nothing is uploaded. {SHARING_WARNING} {DOM_PRIVACY_NOTICE}"
        ),
        content,
    })
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&#39;")
}
fn render_dom_diff(diff: &regressee_core::dom_snapshot::DomTextDiff) -> String {
    let mut html = format!(
        "<p>Exact normalized text {}. Added lines: {}; removed lines: {}. Counts describe lines, not semantic or node changes.</p>",
        if diff.changed { "differs" } else { "is equal" },
        diff.added_lines,
        diff.removed_lines
    );
    if diff.display_limited || diff.fallback.is_some() {
        html.push_str(&format!("<p>Bounded diff display: {:?}; display limited: {}. Complete captured originals are available below; a display limit never means unchanged.</p>", diff.fallback, diff.display_limited));
    }
    for hunk in &diff.hunks {
        html.push_str(&format!(
            "<pre>@@ -{},{} +{},{} @@\n",
            hunk.old_start, hunk.old_lines, hunk.new_start, hunk.new_lines
        ));
        for row in &hunk.rows {
            let (prefix, label) = match row.kind {
                DomDiffRowKind::Added => ('+', "added"),
                DomDiffRowKind::Removed => ('-', "removed"),
                DomDiffRowKind::Context => (' ', "context"),
            };
            html.push_str(&format!(
                "{} {} {} {} {}\n",
                prefix,
                label,
                row.old_line
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".into()),
                row.new_line
                    .map(|v| v.to_string())
                    .unwrap_or_else(|| "-".into()),
                escape(&row.text)
            ));
            if !row.has_newline {
                html.push_str("\\ No newline at end of source\n");
            }
        }
        html.push_str("</pre>");
    }
    html
}
fn evidence_labels(mode: CaptureMode) -> String {
    let mut labels = String::new();
    if mode.screenshots() {
        labels.push_str(EVIDENCE_LABELS);
    }
    if mode.dom_snapshots() {
        labels.push_str(" Before/After DOM files are original normalized canonical records and derived readable plaintext, never executable page HTML. Unified DOM diffs are derived bounded text views. Script/event/form exclusions are not complete redaction.");
    }
    labels
}
fn new_file(path: &Path) -> Result<fs::File> {
    let mut o = fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    let f = o.open(path)?;
    #[cfg(windows)]
    regressee_core::platform::protect(path)?;
    Ok(f)
}
fn checkpoint(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return Err(crate::analysis_engine::Cancelled.into());
    }
    Ok(())
}
pub(crate) fn execute(store: &Store, command: Command, cancel: &AtomicBool) -> Result<Output> {
    checkpoint(cancel)?;
    Ok(match command {
        Command::Analysis { analysis } => {
            Output::Analysis(Box::new(store.load_analysis(analysis)?))
        }
        Command::ReviewHistory {
            analysis,
            key,
            before,
            limit,
        } => Output::ReviewHistory(store.review_history_page(analysis, key, before, limit)?),
        Command::InventoryEdit {
            mut preview,
            pages,
            included,
        } => {
            let saved = store.load_project(preview.settings.id)?;
            ensure!(
                saved.revision == preview.project_revision && saved.project == preview.settings,
                "revision_conflict: inventory settings are stale"
            );
            preview.inventory.validate()?;
            ensure!(
                preview.inventory.project == preview.settings.id,
                "invalid_request: inventory/project mismatch"
            );
            ensure!(
                !pages.is_empty()
                    && pages.len() <= 10000
                    && pages
                        .iter()
                        .all(|id| preview.inventory.pages.iter().any(|p| p.id == *id)),
                "invalid_request: select existing inventory page IDs"
            );
            for p in &mut preview.inventory.pages {
                if pages.contains(&p.id) {
                    p.included = included;
                    p.reason = (!included).then(|| "Excluded by terminal selection".into());
                }
            }
            Output::Inventory(preview)
        }
        Command::Results {
            selection,
            offset,
            limit,
            page,
            profile,
            classification,
            capture_state,
        } => {
            ensure!(
                (1..=200).contains(&limit) && offset <= MAX_TARGETS,
                "invalid_request: limit 1–200; offset at most 30000"
            );
            let (selection, e) = evidence(store, &selection, false)?;
            let run = &e.latest.as_ref().unwrap().run;
            ensure!(
                run.capture_plan.mode.screenshots(),
                "invalid_request: this run requested DOM snapshots only; use results list --kind dom"
            );
            let rows = rows(&e)
                .into_iter()
                .filter(|r| {
                    page.is_none_or(|id| r.target.key.page == id)
                        && profile.is_none_or(|id| r.target.key.profile == id)
                        && classification.is_none_or(|c| r.classification == Some(c))
                        && capture_state.is_none_or(|c| {
                            matches!(
                                (c, &r.target.outcome),
                                (CaptureFilter::Pending, TargetOutcome::Pending)
                                    | (CaptureFilter::Captured, TargetOutcome::Captured { .. })
                                    | (CaptureFilter::Failed, TargetOutcome::Failed { .. })
                                    | (CaptureFilter::Skipped, TargetOutcome::Skipped { .. })
                                    | (CaptureFilter::Cancelled, TargetOutcome::Cancelled)
                            )
                        })
                })
                .collect::<Vec<_>>();
            let total = rows.len();
            let mut rows = rows
                .into_iter()
                .skip(offset)
                .take(limit)
                .collect::<Vec<_>>();
            attach_reviews(store, &e, &mut rows)?;
            Output::Results(Results {
                selection,
                baseline: e.baseline.as_ref().map(|b| b.id),
                inventory: run.inventory.id,
                coverage: run.inventory.coverage.clone(),
                state: run.state,
                total,
                offset,
                next_offset: (offset.saturating_add(limit) < total).then_some(offset + limit),
                rows,
                reviews_semantics: REVIEWS_SEMANTICS.into(),
            })
        }
        Command::Artifacts { selection, key } => {
            let (s, e) = evidence(store, &selection, false)?;
            Output::Artifacts(Box::new(artifacts(store, &s, &e, &key)?))
        }
        Command::CropPreview {
            selection,
            key,
            pane,
            rectangle,
            destination,
        } => Output::CropPreview(crop_preview(
            store,
            &selection,
            &key,
            pane,
            rectangle,
            &destination,
        )?),
        Command::Crop(plan) => {
            let actual = crop_preview(
                store,
                &plan.selection,
                &plan.key,
                plan.pane,
                plan.rectangle.clone(),
                &plan.destination,
            )?;
            ensure!(
                identity_hash(&actual)? == identity_hash(&plan)?,
                "confirmation_conflict: crop source/selection changed"
            );
            regressee_storage::require_space(&store.paths().cache, actual.source.artifact.bytes)?;
            regressee_storage::require_space(
                actual.destination.parent().unwrap(),
                u64::from(actual.rectangle.width) * u64::from(actual.rectangle.height) * 4
                    + 1024 * 1024,
            )?;
            // Freeze the exact bounded bytes whose hash was checked. Decoding
            // never reopens a live CAS pathname after verification.
            let (_, source_bytes) =
                store.read_verified_artifact(&actual.source.artifact.id, 128 * 1024 * 1024)?;
            let mut source = tempfile::NamedTempFile::new_in(&store.paths().cache)?;
            source.write_all(&source_bytes)?;
            drop(source_bytes);
            let mut rows = images::Rows::open(source.path(), cancel)?;
            ensure!(
                (rows.w, rows.h) == (actual.source.width, actual.source.height),
                "invalid_artifact: source dimensions mismatch"
            );
            let r = &actual.rectangle;
            let mut pixels = Vec::with_capacity(r.width as usize * r.height as usize * 4);
            for y in 0..rows.h {
                let row = rows.row(cancel)?;
                if y >= r.y && y < r.y + r.height {
                    pixels.extend_from_slice(&row[r.x as usize * 4..(r.x + r.width) as usize * 4]);
                }
            }
            rows.finish()?;
            let mut temp = tempfile::NamedTempFile::new_in(actual.destination.parent().unwrap())?;
            {
                let mut enc = png::Encoder::new(&mut temp, r.width, r.height);
                enc.set_color(png::ColorType::Rgba);
                enc.set_depth(png::BitDepth::Eight);
                enc.add_text_chunk(
                    "Regressee derivative".into(),
                    format!(
                        "derived crop of {}; screenshot_pixels; source bytes unaltered",
                        actual.pane.source_label()
                    ),
                )?;
                enc.write_header()?.write_image_data(&pixels)?;
            }
            checkpoint(cancel)?;
            temp.as_file().sync_all()?;
            temp.persist_noclobber(&actual.destination)
                .map_err(|e| e.error)?;
            Output::Cropped {
                path: actual.destination,
                source: actual.source,
                rectangle: actual.rectangle,
                coordinate_space: actual.coordinate_space,
                derived: true,
            }
        }
        Command::ReportPreview {
            selection,
            dom_analysis,
            selected,
            destination,
        } => Output::ReportPreview(Box::new(report_preview(
            store,
            &selection,
            dom_analysis,
            &selected,
            &destination,
        )?)),
        Command::Report(plan) => {
            let actual = report_preview(
                store,
                &plan.selection,
                plan.content.dom_analysis.as_ref().map(|a| a.revision.id),
                &plan.selected,
                &plan.destination,
            )?;
            ensure!(
                identity_hash(&actual)? == identity_hash(&plan)?,
                "confirmation_conflict: report content/revision changed"
            );
            regressee_storage::require_space(
                actual.destination.parent().unwrap(),
                actual.maximum_output_bytes,
            )?;
            let stage = tempfile::tempdir_in(actual.destination.parent().unwrap())?;
            let mut html = String::from(
                "<!doctype html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><meta http-equiv=\"Content-Security-Policy\" content=\"default-src 'none'; img-src 'self'; base-uri 'none'; form-action 'none'\"><title>Regressee local evidence</title></head><body><h1>Regressee local evidence</h1><p>Local evidence, not an automatic bug verdict.</p>",
            );
            let labels = evidence_labels(actual.content.run.capture_plan.mode);
            html.push_str(&format!("<section aria-labelledby=\"sharing-warning\"><h2 id=\"sharing-warning\">Before sharing</h2><p><strong>{}</strong></p><p>{}</p></section><p>{}</p>",escape(SHARING_WARNING),escape(&actual.privacy),escape(&labels)));
            let json = serde_json::to_string_pretty(&actual.content)?;
            let baseline_label = actual
                .content
                .baseline
                .as_ref()
                .map(|b| b.id.to_string())
                .unwrap_or_else(|| "No baseline reference".into());
            let screenshot_analysis_label = actual
                .selection
                .analysis
                .map(|id| id.to_string())
                .unwrap_or_else(|| "No saved analysis".into());
            let dom_analysis_label = actual
                .content
                .dom_analysis
                .as_ref()
                .map(|a| a.revision.id.to_string())
                .unwrap_or_else(|| "No saved DOM analysis".into());
            let analysis_label = match actual.content.run.capture_plan.mode {
                CaptureMode::ScreenshotsOnly => screenshot_analysis_label,
                CaptureMode::ScreenshotsAndDomSnapshots => {
                    format!("Screenshot {screenshot_analysis_label}; DOM {dom_analysis_label}")
                }
                CaptureMode::DomSnapshotsOnly => format!("DOM {dom_analysis_label}"),
            };
            let captured = actual
                .content
                .run
                .targets
                .iter()
                .filter(|t| t.any_kind_captured())
                .count();
            html.push_str(&format!("<p>Run: {}<br>Baseline: {}<br>Analysis: {}<br>Captured: {} of {} intended; state: {:?}.<br>Discovery coverage: {}</p>",actual.selection.run.unwrap(),baseline_label,analysis_label,captured,actual.content.run.targets.len(),actual.content.run.state,escape(&actual.content.run.inventory.coverage.label())));
            if actual.content.run.capture_plan.mode.dom_snapshots() {
                let screenshot_captured = actual
                    .content
                    .run
                    .targets
                    .iter()
                    .filter(|t| matches!(t.outcome, TargetOutcome::Captured { .. }))
                    .count();
                let screenshot_requested = actual
                    .content
                    .run
                    .targets
                    .iter()
                    .filter(|t| !matches!(t.outcome, TargetOutcome::NotRequested))
                    .count();
                let dom_captured = actual
                    .content
                    .run
                    .targets
                    .iter()
                    .filter(|t| matches!(t.dom_outcome, DomTargetOutcome::Captured { .. }))
                    .count();
                let dom_requested = actual
                    .content
                    .run
                    .targets
                    .iter()
                    .filter(|t| !t.dom_outcome.is_not_requested())
                    .count();
                html.push_str(&format!("<p>Checks: {:?}. Requested DOM captures: {dom_captured} of {dom_requested}. {} Baseline omissions remain separately recorded, never unchanged.</p>", actual.content.run.capture_plan.mode, if actual.content.run.capture_plan.mode.screenshots() { format!("Requested screenshot captures: {screenshot_captured} of {screenshot_requested}.") } else { "Screenshots not requested.".into() }));
            }
            let mut markdown = format!(
                "# Regressee local evidence\n\nRun: {}\nBaseline: {}\nAnalysis: {}\nSelected targets: {} of {} intended\nCapture state: {:?}\nCoverage: {}\n\nLocal evidence, not an automatic bug verdict. Full recorded settings, excluded inventory, ignored rectangles and comparability gates are in [index.html](index.html). No remote content is loaded.\n\n## Before sharing\n\n{}\n\n{}\n\n{}\n\n",
                actual.selection.run.unwrap(),
                baseline_label,
                analysis_label,
                actual.selected.len(),
                actual.content.run.targets.len(),
                actual.content.run.state,
                escape(&actual.content.run.inventory.coverage.label()),
                escape(SHARING_WARNING),
                escape(&actual.privacy),
                escape(&labels)
            );
            markdown.push_str(&format!("Recorded capture project revision: {}. Resume timestamps: {:?}. Reference-source settings are recorded separately in the HTML; legacy absence is explicitly unknown.\n\n{}\n\n",actual.content.queue.as_ref().map(|q|q.project_revision.to_string()).unwrap_or_else(||"unknown".into()),actual.content.queue.as_ref().map(|q|&q.resumed_at),actual.content.reviews_semantics));
            html.push_str("<ul>");
            for limitation in &actual.content.limitations {
                markdown.push_str(&format!("- {}\n", escape(limitation)));
                html.push_str(&format!("<li>{}</li>", escape(limitation)));
            }
            html.push_str("</ul>");
            for (index, a) in actual.content.artifacts.iter().enumerate() {
                html.push_str(&format!(
                    "<h2>Target {} / {}</h2>",
                    a.key.page, a.key.profile
                ));
                let row = actual
                    .content
                    .rows
                    .iter()
                    .find(|r| r.target.key == a.key)
                    .unwrap();
                let ignored = a
                    .analysis_report
                    .as_ref()
                    .and_then(|r| r.request.mask.as_ref())
                    .map_or(0, |m| m.rectangles.len());
                markdown.push_str(&format!("## Target {index}: {} / {}\n\nClassification: {:?}; capture: {:?}; ignored rectangles: {ignored}.\n\n<pre>URL: {}\nWarnings: {}\n",a.key.page,a.key.profile,row.classification,a.target.outcome,escape(a.target.requested.as_str()),escape(&format!("{:?}; {:?}",a.target.warnings,a.analysis_report.as_ref().map(|r|(&r.gates,&r.warnings))))));
                for review in &row.reviews {
                    markdown.push_str(&format!(
                        "Review {:?}: {}\n",
                        review.state,
                        escape(&review.note).replace('`', "&#96;")
                    ));
                }
                markdown.push_str("</pre>\n\n");
                for (name, pane, image) in [
                    ("before", Pane::Before, &a.before),
                    ("after", Pane::After, &a.after),
                    ("difference", Pane::Difference, &a.difference),
                ] {
                    if let Some(image) = image {
                        checkpoint(cancel)?;
                        let (_, bytes) =
                            store.read_verified_artifact(&image.artifact.id, 128 * 1024 * 1024)?;
                        let file = format!("{index}-{name}.png");
                        let mut out = new_file(&stage.path().join(&file))?;
                        out.write_all(&bytes)?;
                        out.sync_all()?;
                        html.push_str(&format!("<h3>{}: {} × {} screenshot pixels</h3><p><a href=\"{file}\">{}</a></p><img src=\"{file}\" alt=\"{}\" width=\"{}\">",pane.caption(),image.width,image.height,pane.link_label(),pane.caption(),image.width.min(1000)));
                        markdown.push_str(&format!(
                            "- Target {index} {}: [{}]({file}), {} × {} screenshot pixels\n",
                            pane.caption(),
                            pane.link_label(),
                            image.width,
                            image.height
                        ));
                    } else {
                        html.push_str(&format!(
                            "<p>{}: unavailable; not verified.</p>",
                            pane.caption()
                        ));
                        markdown.push_str(&format!(
                            "- Target {index} {}: unavailable; not verified.\n",
                            pane.caption()
                        ));
                    }
                }
            }
            for (index, a) in actual.content.dom_artifacts.iter().enumerate() {
                checkpoint(cancel)?;
                html.push_str(&format!(
                    "<p>DOM comparison: {}</p>",
                    escape(
                        &a.result
                            .as_ref()
                            .map(|r| format!("{:?}", r.outcome))
                            .unwrap_or_else(|| "No saved DOM comparison; not verified".into())
                    )
                ));
                for review in actual.content.dom_reviews.iter().filter(|r| r.key == a.key) {
                    html.push_str(&format!(
                        "<p>DOM review {:?}: {}</p>",
                        review.state,
                        escape(&review.note)
                    ));
                    markdown.push_str(&format!(
                        "\n<pre>DOM review {:?}: {}</pre>\n",
                        review.state,
                        escape(&review.note)
                    ));
                }
                html.push_str(&format!(
                    "<h2>DOM snapshot: {} / {}</h2><p>Capture outcome: {}</p><p>{}</p><p>{}</p>",
                    a.key.page,
                    a.key.profile,
                    escape(&format!("{:?}", a.target.dom_outcome)),
                    escape(DOM_PRIVACY_NOTICE),
                    escape(DOM_COVERAGE_NOTICE)
                ));
                markdown.push_str(&format!(
                    "\n## DOM snapshot: {} / {}\n\n<pre>Capture outcome: {}\n{}\n{}</pre>\n\n",
                    a.key.page,
                    a.key.profile,
                    escape(&format!("{:?}", a.target.dom_outcome)),
                    escape(DOM_PRIVACY_NOTICE),
                    escape(DOM_COVERAGE_NOTICE)
                ));
                for warning in &a.warnings {
                    html.push_str(&format!("<p>{}</p>", escape(warning)));
                }
                for (name, original) in [("before", &a.before), ("after", &a.after)] {
                    if let Some(original) = original {
                        let (canonical, _) = dom::read_original(store, original)?;
                        let plaintext = crate::dom_snapshot::original_source(
                            store,
                            &original.source.artifact,
                            original.source.policy,
                        )?;
                        // Filenames are generated locally, never taken from captured URLs.
                        let canonical_file = format!("{index}-dom-{name}.canonical.txt");
                        let text_file = format!("{index}-dom-{name}.txt");
                        for (file, bytes) in [
                            (&canonical_file, canonical.as_slice()),
                            (&text_file, plaintext.as_bytes()),
                        ] {
                            let mut out = new_file(&stage.path().join(file))?;
                            out.write_all(bytes)?;
                            out.sync_all()?;
                        }
                        html.push_str(&format!("<p>{name} DOM source: <a download href=\"{canonical_file}\">original normalized canonical JSON records (plaintext)</a>; <a download href=\"{text_file}\">readable plaintext</a>. Source run: {}; artifact: {}; policy: {}.</p>", original.source.run, original.source.artifact.as_str(), original.source.policy.version));
                        markdown.push_str(&format!("- {name} DOM source: [canonical JSON records (plaintext)]({canonical_file}); [readable plaintext]({text_file}). Source run: {}; artifact: {}; policy: {}.\n", original.source.run, original.source.artifact.as_str(), original.source.policy.version));
                    } else {
                        html.push_str(&format!(
                            "<p>{name} DOM original unavailable; not verified.</p>"
                        ));
                        markdown.push_str(&format!(
                            "- {name} DOM original unavailable; not verified.\n"
                        ));
                    }
                }
                if let (Some(before), Some(after)) = (&a.before, &a.after) {
                    let diff = crate::dom_snapshot::compare_originals(
                        store,
                        &before.source.artifact,
                        &after.source.artifact,
                        before.source.policy,
                        &actual.content.dom_diff_options,
                        cancel,
                    )?;
                    html.push_str("<h3>Derived unified DOM text diff</h3><p>Exact text equality is separate from capture stability, source policy/runtime compatibility and the recorded comparison outcome below.</p>");
                    html.push_str(&render_dom_diff(&diff));
                    markdown.push_str(&format!("\nDerived bounded diff: added lines {}; removed lines {}; display limited: {}; fallback: {:?}. See index.html.\n", diff.added_lines, diff.removed_lines, diff.display_limited, diff.fallback));
                }
            }
            html.push_str(
                "<h2>Identity, settings, coverage, warnings, reviews and ignored regions</h2><pre>",
            );
            html.push_str(&escape(&json));
            html.push_str("</pre></body></html>");
            ensure!(
                html.len() <= 64 * 1024 * 1024 && markdown.len() <= 8 * 1024 * 1024,
                "invalid_request: report display exceeds bounded output budget; select fewer targets"
            );
            for (name, bytes) in [
                ("index.html", html.as_bytes()),
                ("summary.md", markdown.as_bytes()),
            ] {
                let mut f = new_file(&stage.path().join(name))?;
                f.write_all(bytes)?;
                f.sync_all()?;
            }
            checkpoint(cancel)?;
            // create_dir is exclusive; a concurrent destination can never be replaced.
            fs::create_dir(&actual.destination)?;
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                fs::set_permissions(&actual.destination, fs::Permissions::from_mode(0o700))?;
            }
            #[cfg(windows)]
            regressee_core::platform::protect(&actual.destination)?;
            for entry in fs::read_dir(stage.path())? {
                let entry = entry?;
                fs::hard_link(entry.path(), actual.destination.join(entry.file_name()))?;
            }
            Output::Reported {
                html: actual.destination.join("index.html"),
                markdown: actual.destination.join("summary.md"),
            }
        }
    })
}

#[cfg(test)]
use crate::approval_fixture as legacy_fixture;
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn report_checkpoints_return_the_established_typed_cancellation() {
        let error = checkpoint(&AtomicBool::new(true)).unwrap_err();
        assert!(error.is::<crate::analysis_engine::Cancelled>());
        assert!(checkpoint(&AtomicBool::new(false)).is_ok());
    }
    #[test]
    fn dom_report_diff_encodes_untrusted_text_only_in_inert_text_contexts() -> Result<()> {
        let attack = "</pre></script><script>globalThis.executed=true</script><img src=\"https://attacker.invalid/x\" onerror=\"evil()\"><a href=\"javascript:evil()\">&'\"</a>";
        let diff = diff_text(
            "safe\n",
            &format!("{attack}\n"),
            &DomDiffOptions::default(),
            || false,
        )?;
        let html = render_dom_diff(&diff);
        for active in [
            "<script",
            "<img",
            "<a ",
            "</script>",
            " onerror=\"",
            "href=\"javascript:",
        ] {
            assert!(!html.contains(active), "active syntax leaked: {active}");
        }
        assert!(html.contains("&lt;/pre&gt;&lt;/script&gt;"));
        assert!(html.contains("&amp;&#39;&quot;"));
        assert!(html.contains("+ added"));
        assert!(html.contains("- removed"));
        assert_eq!(
            html.matches("<pre>").count(),
            html.matches("</pre>").count()
        );
        let labels = evidence_labels(CaptureMode::DomSnapshotsOnly);
        assert!(!labels.contains("original screenshots"));
        assert!(labels.contains("never executable page HTML"));
        Ok(())
    }
    #[test]
    fn legacy_reports_expose_unknown_queue_without_borrowing_current_settings() -> Result<()> {
        let root = tempfile::tempdir()?;
        let (mut store, project, run, _, _) = super::legacy_fixture::fixture(root.path());
        let mut saved = store.load_project(project)?;
        saved.project.discovery.skipped_text = vec!["TODAY_ONLY_NOT_RECORDED".into()];
        store.save_project(&saved.project, Some(saved.revision))?;
        let record = store.load_run(run)?;
        assert!(record.queue.is_none());
        let Output::ReportPreview(plan) = execute(
            &store,
            Command::ReportPreview {
                selection: Selection {
                    project,
                    run: Some(run),
                    analysis: None,
                },
                dom_analysis: None,
                selected: vec![record.run.targets[0].key.clone()],
                destination: root.path().join("legacy-report"),
            },
            &AtomicBool::new(false),
        )?
        else {
            unreachable!()
        };
        assert!(plan.content.queue.is_none());
        assert!(
            plan.content
                .reference_settings
                .iter()
                .all(|r| r.queue.is_none())
        );
        assert!(
            plan.content
                .limitations
                .iter()
                .any(|l| l.contains("Unknown legacy capture queue"))
        );
        assert!(
            plan.content
                .limitations
                .iter()
                .any(|l| l.contains("Unknown legacy reference queue"))
        );
        assert!(!serde_json::to_string(&plan.content)?.contains("TODAY_ONLY_NOT_RECORDED"));
        assert_eq!(plan.fingerprint, identity_hash(&plan.content)?);
        Ok(())
    }
}
