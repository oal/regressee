// SPDX-License-Identifier: MPL-2.0
//! Native workflow presentation; no storage or network work is performed here.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Built once per presentation operation, never once per selected target.
/// No persisted/cache identity is trusted in place of current evidence checks.
#[derive(Default)]
pub(super) struct ReviewIndex {
    classifications: BTreeMap<TargetKey, Classification>,
    captured: BTreeSet<TargetKey>,
    pub(super) eligible: BTreeSet<TargetKey>,
    pub(super) target_labels: BTreeMap<TargetKey, String>,
    pub(super) profile_labels: BTreeMap<regressee_core::domain::ProfileId, String>,
}
impl ReviewIndex {
    fn build(
        evidence: &ProjectEvidence,
        active: Option<regressee_core::domain::BaselineId>,
    ) -> Self {
        let Some(run) = &evidence.latest else {
            return Self::default();
        };
        let comparison = evidence
            .comparison
            .as_ref()
            .filter(|a| a.revision.run == run.run.id);
        let pages: BTreeMap<_, _> = run
            .run
            .inventory
            .pages
            .iter()
            .map(|p| (p.id, p.canonical.to_string()))
            .collect();
        let classifications = comparison
            .map(|a| a.review_classifications())
            .unwrap_or_default();
        let mut index = Self {
            classifications,
            captured: run.all_captures().map(|c| c.metadata.key.clone()).collect(),
            target_labels: run
                .run
                .targets
                .iter()
                .map(|t| {
                    (
                        t.key.clone(),
                        pages
                            .get(&t.key.page)
                            .cloned()
                            .unwrap_or_else(|| t.requested.to_string()),
                    )
                })
                .collect(),
            profile_labels: run
                .run
                .profiles
                .iter()
                .map(|p| (p.id, format!("{} · {} × {}", p.name, p.width, p.height)))
                .collect(),
            ..Self::default()
        };
        let Some(comparison) = comparison else {
            return index;
        };
        let Some(baseline) = evidence
            .baseline
            .as_ref()
            .filter(|b| Some(b.id) == active)
            .or_else(|| evidence.baselines.iter().find(|b| Some(b.id) == active))
        else {
            return index;
        };
        let checked = evidence
            .baselines
            .iter()
            .find(|b| b.id == comparison.revision.baseline)
            .or_else(|| {
                evidence
                    .baseline
                    .as_ref()
                    .filter(|b| b.id == comparison.revision.baseline)
            });
        let references: BTreeSet<_> = baseline
            .references
            .iter()
            .filter(|reference| {
                baseline.id == comparison.revision.baseline
                    || checked.is_some_and(|checked| {
                        checked.capture_plan == baseline.capture_plan
                            && checked.inventory == baseline.inventory
                            && checked.references.iter().any(|before| before == *reference)
                    })
            })
            .map(|r| &r.key)
            .collect();
        let captures: BTreeSet<_> = run
            .all_captures()
            .filter(|c| c.metadata.baseline_ready())
            .map(|c| &c.metadata.key)
            .collect();
        let results: BTreeSet<_> = comparison
            .revision
            .results
            .iter()
            .filter(|r| {
                r.eligible_pixels > 0
                    && matches!(
                        index
                            .classifications
                            .get(&r.key)
                            .copied()
                            .unwrap_or(r.classification),
                        Classification::Changed | Classification::NoSignificantChange
                    )
            })
            .map(|r| &r.key)
            .collect();
        let reports: BTreeSet<_> = comparison.perceptual.as_ref().map(|p| p.reports.iter().filter(|(_, r)| r.gates.is_empty() && !r.warnings.contains(&regressee_app::analysis_engine::Warning::RenderingEnvironmentMismatch)).map(|(key, _)| key).collect()).unwrap_or_default();
        index.eligible = run
            .run
            .targets
            .iter()
            .filter(|t| {
                matches!(t.outcome, TargetOutcome::Captured { .. })
                    && references.contains(&t.key)
                    && captures.contains(&t.key)
                    && results.contains(&t.key)
                    && reports.contains(&t.key)
            })
            .map(|t| t.key.clone())
            .collect();
        index
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Coverage {
    total: usize,
    changed: usize,
    unchanged: usize,
    missing: usize,
    unclassified: usize,
}
impl Coverage {
    fn from_run(run: &regressee_core::evidence::RunRecord, index: &ReviewIndex) -> Self {
        let mut coverage = Self::default();
        for target in &run.run.targets {
            coverage.count(
                matches!(target.outcome, TargetOutcome::Captured { .. })
                    && index.captured.contains(&target.key),
                index.classifications.get(&target.key).copied(),
            );
        }
        coverage
    }
    fn count(&mut self, captured: bool, classification: Option<Classification>) {
        self.total += 1;
        if !captured {
            self.missing += 1;
        } else {
            match classification {
                Some(Classification::Changed) => self.changed += 1,
                Some(Classification::NoSignificantChange) => self.unchanged += 1,
                Some(Classification::CouldNotCapture | Classification::NotComparable) => {
                    self.missing += 1
                }
                None => self.unclassified += 1,
            }
        }
    }
}

impl Shell {
    pub(super) fn capture_is_saved_baseline(&self) -> bool {
        self.evidence.latest.as_ref().is_some_and(|run| {
            self.evidence.baseline.as_ref().is_some_and(|baseline| {
                (!baseline.references.is_empty() || !baseline.dom_references.is_empty())
                    && baseline.references.len() == run.all_captures().count()
                    && baseline.dom_references.len() == run.dom_captures.len()
                    && baseline.intended_count as usize == run.run.targets.len()
                    && baseline
                        .references
                        .iter()
                        .all(|reference| reference.run == run.run.id)
                    && baseline
                        .dom_references
                        .iter()
                        .all(|reference| reference.run == run.run.id)
            })
        })
    }

    pub(super) fn review_title(&self) -> &'static str {
        if self.history_inspection.is_some() {
            return if self.has_kind_comparison() {
                "Saved check · read-only"
            } else {
                "Saved capture · read-only"
            };
        }
        if self.focus.review_surface == ReviewSurface::Pages || self.preview.is_some() {
            "Pages & baseline"
        } else if self.focus.capturing {
            "Capture coverage"
        } else if self.focus.candidate {
            "Inspect capture"
        } else if !self.has_kind_comparison() && self.evidence.latest.is_some() {
            if self.capture_is_saved_baseline() {
                if self.recorded_mode() == CaptureMode::DomSnapshotsOnly {
                    "Baseline DOM snapshots"
                } else if self.recorded_mode() == CaptureMode::ScreenshotsAndDomSnapshots {
                    "Baseline captures"
                } else {
                    "Baseline screenshots"
                }
            } else {
                "Inspect capture"
            }
        } else {
            "Review changes"
        }
    }

    pub(super) fn partial_baseline_capture(&self) -> bool {
        self.history_inspection.is_none()
            && self.preview.is_none()
            && self
                .evidence
                .latest
                .as_ref()
                .and_then(regressee_gui::task::baseline_consent_coverage)
                .is_some_and(|c| {
                    c.screenshot_captures < c.screenshot_intended || c.dom_captures < c.dom_intended
                })
    }

    pub(super) fn baseline_save_confirmation(
        &self,
        captured: usize,
        intended: usize,
    ) -> (String, String, &'static str) {
        let retained = if self.evidence.baseline.is_some() {
            "This replaces your current baseline. Earlier baseline versions and all capture results are kept in History."
        } else {
            "All capture results, including failures, are kept in History."
        };
        if self.recorded_mode().dom_snapshots()
            && let Some(coverage) = self
                .evidence
                .latest
                .as_ref()
                .and_then(regressee_gui::task::baseline_consent_coverage)
        {
            let mut included = Vec::new();
            if coverage.screenshot_intended > 0 {
                included.push(format!(
                    "{} of {} screenshots",
                    coverage.screenshot_captures, coverage.screenshot_intended
                ));
            }
            if coverage.dom_intended > 0 {
                included.push(format!(
                    "{} of {} DOM snapshots",
                    coverage.dom_captures, coverage.dom_intended
                ));
            }
            let partial = coverage.screenshot_captures < coverage.screenshot_intended
                || coverage.dom_captures < coverage.dom_intended;
            return (
                if partial {
                    "Save baseline and skip missing checks?"
                } else {
                    "Save these captures as the baseline?"
                }
                .into(),
                format!(
                    "{} will become the references for future checks. {captured} of {intended} page and screen-size pairs have at least one eligible capture. Screenshots and DOM snapshots keep separate references. Unstable DOM snapshots are retained but excluded from the baseline. {} {retained}",
                    included.join(" and "),
                    if partial {
                        "Missing kinds will be omitted, not retried automatically. Capture and save a replacement baseline to add missing coverage."
                    } else {
                        "Future checks use this saved page list and these check kinds."
                    }
                ),
                if partial {
                    "Save & skip missing"
                } else {
                    "Save baseline"
                },
            );
        }
        if captured < intended {
            let missing = count_label(intended - captured, "screenshot", "screenshots");
            (
                format!("Save baseline and skip {missing}?"),
                format!(
                    "{captured} of {intended} screenshots will be saved. {missing} could not be captured and will be skipped. Future checks will compare only the saved screenshots; skipped page and screen-size pairs will not be checked. To include them later, capture and save a replacement baseline. {retained}"
                ),
                "Save & skip missing",
            )
        } else {
            (
                "Save these screenshots as the baseline?".into(),
                format!(
                    "Future checks will compare against these screenshots using the same page list. {retained}"
                ),
                "Save baseline",
            )
        }
    }

    fn capture_next_step(&self) -> Option<String> {
        if self.history_inspection.is_some() {
            return None;
        }
        if self.has_kind_comparison() || self.preview.is_some() || self.focus.capturing {
            return None;
        }
        let run = self.evidence.latest.as_ref()?;
        if run.run.capture_plan.mode.dom_snapshots() {
            if self.capture_is_saved_baseline() {
                return Some("Baseline saved · Run check to compare its saved kinds.".into());
            }
            if let Some(c) = regressee_gui::task::baseline_consent_coverage(run) {
                return Some(format!(
                    "Captured {} screenshots and {} DOM snapshots. Save baseline explicitly to use these references for future checks{}.",
                    c.screenshot_captures,
                    c.dom_captures,
                    if c.screenshot_captures < c.screenshot_intended
                        || c.dom_captures < c.dom_intended
                    {
                        "; missing kinds will be skipped"
                    } else {
                        ""
                    }
                ));
            }
            return Some(
                "Capture incomplete. Inspect each requested kind; no baseline has been saved."
                    .into(),
            );
        }
        if self.capture_is_saved_baseline() {
            return Some("Baseline saved · Run check to compare.".into());
        }
        if let Some((captured, intended)) = run.baseline_save_counts()
            && captured < intended
        {
            return Some(format!(
                "{captured} of {intended} screenshots captured. Save baseline to keep these and skip {}, or review failures.",
                count_label(
                    intended - captured,
                    "missing screenshot",
                    "missing screenshots"
                )
            ));
        }
        Some(match run.run.intent {
            RunIntent::NewBaseline => match run.run.state {
                JobState::CompletedWithErrors if self.page_list_workspace() => {
                    "Capture incomplete · Inspect capture for failures, or Use recorded pages to prepare another capture."
                }
                JobState::Completed if self.evidence.baseline.is_some() => {
                    "Choose Save baseline to use these screenshots for future comparisons. The current baseline will be replaced."
                }
                JobState::Completed => {
                    "Choose Save baseline to use these screenshots for future comparisons."
                }
                JobState::CompletedWithErrors if self.evidence.baseline.is_some() => {
                    "Baseline unchanged. Filter by Missing or not compared to review failures, then Find pages to try again."
                }
                JobState::CompletedWithErrors => {
                    "Baseline not saved. Filter by Missing or not compared to review failures, then Find pages to try again."
                }
                JobState::Paused | JobState::Interrupted => {
                    "Capture unfinished · Resume or Find pages to start again."
                }
                JobState::Cancelled => "Capture stopped · Find pages to start again.",
                _ => "Capture unfinished · No baseline saved.",
            },
            RunIntent::RepresentativePreview => {
                "Preview only · Capture the full page list before saving a baseline."
            }
            RunIntent::Check { .. } => {
                "No comparison available · Run check to retry; baseline unchanged."
            }
        }.into())
    }

    fn baseline_rail_status(&self) -> String {
        if self.history_inspection.is_some() {
            return "Saved results · read-only\nCurrent baseline unchanged".into();
        }
        if self.initial_evidence_loading() {
            return "Loading saved results…".into();
        }
        if self.initial_evidence_unavailable() {
            return "Could not load saved results".into();
        }
        if let Some(baseline) = &self.evidence.baseline {
            if baseline.capture_plan.mode.dom_snapshots() {
                return if baseline.capture_plan.mode.screenshots() {
                    format!(
                        "{} screenshots\n{} DOM snapshots",
                        baseline.references.len(),
                        baseline.dom_references.len()
                    )
                } else {
                    count_label(
                        baseline.dom_references.len(),
                        "baseline DOM snapshot",
                        "baseline DOM snapshots",
                    )
                };
            }
            count_label(
                baseline.references.len(),
                "baseline screenshot",
                "baseline screenshots",
            )
        } else if self.evidence_dirty
            && self
                .project
                .as_ref()
                .is_some_and(|p| p.project.active_baseline.is_some())
        {
            "Saved baseline".into()
        } else if self.focus.candidate && self.partial_baseline_capture() {
            if self.recorded_mode().dom_snapshots() {
                if self
                    .evidence
                    .latest
                    .as_ref()
                    .and_then(regressee_gui::task::baseline_consent_coverage)
                    .is_some_and(|c| c.captured_targets == 0)
                {
                    "No baseline-ready captures\nInspect missing or unstable kinds".into()
                } else {
                    "Ready to save baseline\nMissing or unstable kinds will be skipped".into()
                }
            } else {
                "Ready to save baseline\nSome screenshots will be skipped".into()
            }
        } else if self.focus.candidate {
            "Ready to save baseline".into()
        } else if self.evidence.latest.as_ref().is_some_and(|run| {
            run.run.intent == RunIntent::NewBaseline
                && run.run.state == JobState::CompletedWithErrors
        }) {
            "Incomplete capture\nNo baseline saved".into()
        } else if self.evidence.latest.is_some() {
            "Capture only\nNo baseline saved".into()
        } else {
            "No saved baseline yet".into()
        }
    }

    pub(super) fn captured_target_label(&self, key: &TargetKey) -> &'static str {
        if self.evidence.latest.as_ref().is_some_and(|run| {
            self.evidence.baseline.as_ref().is_some_and(|baseline| {
                baseline
                    .references
                    .iter()
                    .any(|reference| reference.run == run.run.id && reference.key == *key)
            })
        }) {
            "Saved baseline screenshot"
        } else if self
            .evidence
            .latest
            .as_ref()
            .is_some_and(|run| run.run.intent == RunIntent::NewBaseline)
        {
            "Captured · not yet a baseline"
        } else {
            "Captured · not compared"
        }
    }

    pub(super) fn viewer_pane_indices(&self) -> &'static [usize] {
        if self.preview.is_some() || self.evidence.comparison.is_none() {
            // A first capture has only an original, not a before/after pair.
            return &[1];
        }
        match self.effective_viewer_mode() {
            0 => &[0, 1],
            1 | 3 => &[2],
            4 if self.full_width_after => &[1],
            _ => &[0],
        }
    }

    pub(super) fn fitted_viewer_zoom(&self) -> f32 {
        let pane = self.current_viewer_pane_width();
        (pane / self.viewer_dimensions().0.max(1) as f32).clamp(
            0.05,
            if self.evidence.comparison.is_none() {
                1.
            } else {
                4.
            },
        )
    }

    pub(super) fn current_viewer_pane_width(&self) -> f32 {
        viewer_pane_width(
            self.working_width,
            self.review_queue_width(),
            self.viewer_pane_indices().len(),
        )
    }

    pub(super) fn page_list_workspace(&self) -> bool {
        self.preview.is_some() || self.focus.review_surface == ReviewSurface::Pages
    }

    pub(super) fn discovery_image_mounted(&self) -> bool {
        self.preview.is_some()
            && self.discovery_preview_visible
            && self.evidence.comparison.is_none()
            && self.evidence.latest.as_ref().is_some_and(|r| {
                r.run.intent == RunIntent::RepresentativePreview
                    && self
                        .viewer
                        .as_ref()
                        .and_then(|v| v.identity.as_ref())
                        .is_some_and(|identity| {
                            identity.run == r.run.id
                                && self.selected_key.as_ref() == Some(&identity.key)
                                && self
                                    .project
                                    .as_ref()
                                    .is_some_and(|p| p.project.id == identity.project)
                        })
            })
    }

    pub(super) fn review_queue_width(&self) -> f32 {
        if self.page_list_workspace() && self.options_panel_open() {
            self.working_width - self.options_panel_width() - 12.
        } else if self.page_list_workspace() && !self.discovery_image_mounted() {
            self.working_width
        } else if (self.review_queue_collapsed || self.authoritative_empty_capture())
            && !self.page_list_workspace()
        {
            0.
        } else if self.narrow_workspace {
            200.
        } else {
            272.
        }
    }

    pub(super) fn authoritative_empty_capture(&self) -> bool {
        self.preview.is_none()
            && self.evidence.comparison.is_none()
            && !self.initial_evidence_loading()
            && !self.initial_evidence_unavailable()
            && self
                .evidence
                .latest
                .as_ref()
                .is_none_or(|r| r.run.targets.is_empty())
    }

    /// Parent routes Guided(50/51) here and refreshes mounted focus afterwards.
    pub(super) fn toggle_workspace_presentation(&mut self, control: Control) -> bool {
        if self.agent_locked() {
            return false;
        }
        match control {
            Control::Guided(50) if !self.page_list_workspace() => {
                self.review_queue_collapsed = !self.review_queue_collapsed;
                self.presentation_focus = Some(Control::Guided(50));
                if self.viewer_fit {
                    self.zoom = self.fitted_viewer_zoom();
                }
                true
            }
            Control::Guided(51) if self.preview.is_some() => {
                self.discovery_preview_visible = false;
                self.presentation_focus = Some(Control::Phase5(30));
                true
            }
            _ => false,
        }
    }

    pub(super) fn workspace_control_mounted(&self, control: Control) -> bool {
        if self.focus.screen == Screen::History
            && self.project.is_some()
            && self.history.is_none()
            && self.history_section != 2
        {
            return matches!(
                control,
                Control::Navigate(_)
                    | Control::PagesBaseline
                    | Control::ChooseProject
                    | Control::DismissFeedback
                    | Control::HistoryRefresh
                    | Control::Phase6(16..=18)
            );
        }
        let results = self.focus.screen == Screen::Results && self.project.is_some();
        if results && self.initial_evidence_loading() {
            return matches!(
                control,
                Control::Navigate(_)
                    | Control::Guided(53)
                    | Control::PagesBaseline
                    | Control::ChooseProject
                    | Control::DismissFeedback
            );
        }
        if results && self.initial_evidence_unavailable() {
            return matches!(
                control,
                Control::Navigate(_)
                    | Control::Guided(53)
                    | Control::PagesBaseline
                    | Control::ChooseProject
                    | Control::DismissFeedback
                    | Control::RefreshEvidence
            );
        }
        match control {
            Control::CloseOptions | Control::DiscoveryDetails if results => {
                self.options_panel_open()
            }
            Control::DiscardPageList if results => {
                self.preview.is_some() && self.options_panel_open()
            }
            Control::CopyCaptureDiagnostics
            | Control::Phase5(0..=19 | 26..=28 | 34..=39)
            | Control::AnalysisField(0..=8 | 10..=13)
            | Control::Viewer
            | Control::SplitView
            | Control::SplitDivider
            | Control::Guided(44 | 46)
            | Control::Details(Detail::Analysis)
                if results && self.options_panel_open() =>
            {
                false
            }
            Control::Phase5(23 | 24) | Control::AnalysisField(_)
                if results && self.page_list_workspace() =>
            {
                false
            }
            Control::Phase5(0..=5 | 12..=13 | 15..=19 | 26..=28 | 34..=35)
            | Control::AnalysisField(0..=8 | 10..=13)
            | Control::Details(Detail::Analysis)
                if results && self.mask_draw =>
            {
                false
            }
            Control::Phase5(8..=9) if results && self.mask_draw && self.narrow_workspace => false,
            Control::Phase5(0..=22 | 26..=29 | 32..=37)
            | Control::ZoomIn
            | Control::ZoomOut
            | Control::PanLeft
            | Control::PanRight
            | Control::PanUp
            | Control::PanDown
            | Control::Viewer
            | Control::SplitView
            | Control::SplitDivider
            | Control::Guided(44 | 46 | 52)
            | Control::Details(Detail::Analysis)
                if results && self.page_list_workspace() && !self.discovery_image_mounted() =>
            {
                false
            }
            Control::Guided(19)
                if results && self.page_list_workspace() && self.preview.is_none() =>
            {
                false
            }
            Control::Choice(choice)
                if results
                    && self.page_list_workspace()
                    && self.preview.is_none()
                    && Choice::RESULTS.contains(&choice) =>
            {
                false
            }
            Control::Choice(choice)
                if results
                    && !self.page_list_workspace()
                    && self.review_queue_collapsed
                    && Choice::RESULTS.contains(&choice) =>
            {
                false
            }
            Control::Guided(50) => {
                results && !self.page_list_workspace() && self.evidence.latest.is_some()
            }
            Control::Guided(51) => results && self.discovery_image_mounted(),
            Control::Guided(17 | 19..=24 | 45) | Control::CrawlField(9)
                if results && !self.page_list_workspace() && self.review_queue_collapsed =>
            {
                false
            }
            _ => true,
        }
    }

    pub(super) fn review_navigation_available(&self) -> bool {
        self.focus.screen == Screen::Results
            && self.project.is_some()
            && !self.focus.shows_project_picker()
    }

    /// Offscreen fixtures must call this with their capture width too; the
    /// native render path calls it with the actual desktop viewport width.
    pub(super) fn configure_review_geometry(&mut self, width: f32) {
        self.narrow_workspace = width < 1100.;
        self.navigation_collapsed = self.review_navigation_available()
            && self.navigation_preference.unwrap_or(self.narrow_workspace);
        self.working_width = (width - if self.navigation_collapsed { 48. } else { 240. }).max(1.);
        if self.viewer_fit && self.viewer.is_some() {
            self.zoom = self.fitted_viewer_zoom();
        }
    }
    pub(super) fn results(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.project.is_none() {
            return self.project_picker(cx);
        }
        if self.initial_evidence_loading() || self.initial_evidence_unavailable() {
            return div()
                .debug_selector(|| "results-loading".into())
                .size_full()
                .min_h_0()
                .flex()
                .flex_col()
                .gap_2()
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .justify_between()
                        .gap_2()
                        .child(
                            div()
                                .text_size(px(if self.narrow_workspace {
                                    TITLE_TEXT_SIZE
                                } else {
                                    REVIEW_HEADING_TEXT_SIZE
                                }))
                                .line_height(px(if self.narrow_workspace {
                                    TITLE_LINE_HEIGHT
                                } else {
                                    REVIEW_HEADING_LINE_HEIGHT
                                }))
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(self.review_title()),
                        )
                        .when(self.initial_evidence_unavailable(), |header| {
                            header.child(self.button(
                                Control::RefreshEvidence,
                                "Refresh",
                                ActionStyle::Primary,
                                cx,
                            ))
                        }),
                )
                .child(self.screenshot_placeholder())
                .into_any_element();
        }
        let pages = self.focus.review_surface == ReviewSurface::Pages || self.preview.is_some();
        let mut actions = div().flex().flex_wrap().items_center().gap_2();
        if self.focus.candidate {
            actions = actions.child(self.button(
                Control::SaveBaseline,
                "Save baseline…",
                ActionStyle::Primary,
                cx,
            ));
        }
        if self.partial_baseline_capture() && !pages {
            actions = actions.child(self.button(
                Control::ReviewCaptureFailures,
                "Review failures",
                ActionStyle::Secondary,
                cx,
            ));
        }
        if self.focus.baseline && !pages {
            actions = actions.child(self.button(
                Control::CheckChanges,
                "Run check",
                if self.focus.candidate {
                    ActionStyle::Secondary
                } else {
                    ActionStyle::Primary
                },
                cx,
            ));
        }
        if pages || (!self.focus.baseline && !self.focus.candidate) {
            actions = actions.child(self.button(
                Control::CaptureBaseline,
                if self.preview.is_some() {
                    "Capture pages"
                } else {
                    "Find pages"
                },
                ActionStyle::Primary,
                cx,
            ));
        }
        if pages
            && self.evidence.latest.is_some()
            && self.preview.is_none()
            && self.history_inspection.is_none()
        {
            actions = actions.child(self.guided_button(16, "Use recorded pages", cx));
        }
        if self
            .preview
            .as_ref()
            .is_some_and(|p| p.settings.capture_plan.mode.screenshots())
        {
            actions = actions.child(self.button(
                Control::Phase5(30),
                "Preview first included page…",
                ActionStyle::Secondary,
                cx,
            ));
        }
        actions = actions.child(
            self.tool_control(
                Control::RefreshEvidence,
                ToolIcon::Refresh,
                self.history_inspection
                    .is_some()
                    .then_some("Latest results"),
                if self.history_inspection.is_some() {
                    "Return to latest results"
                } else {
                    "Refresh saved results — does not recapture"
                },
                false,
                cx,
            ),
        );
        if self.discovery_image_mounted() {
            actions = actions.child(self.guided_button(51, "Hide preview", cx));
        }
        div()
            .debug_selector(|| "bounded-review".into())
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_2()
            .when(self.narrow_workspace, |workspace| workspace.gap_1())
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child(if self.inspection_chrome_mounted() {
                                div().flex_1().min_w_0().child(self.inspection_heading()).into_any_element()
                            } else {
                                div().text_size(px(if self.narrow_workspace { TITLE_TEXT_SIZE } else { REVIEW_HEADING_TEXT_SIZE }))
                                    .line_height(px(if self.narrow_workspace { TITLE_LINE_HEIGHT } else { REVIEW_HEADING_LINE_HEIGHT }))
                                    .font_weight(FontWeight::SEMIBOLD).child(self.review_title()).into_any_element()
                            })
                            .child(actions),
                    )
                    .when_some(self.preview.as_ref(), |header, preview| {
                        header.child(div().debug_selector(|| "discovery-capture-plan".into()).w_full().text_size(px(13.)).line_height(px(18.)).text_color(rgb(palette().muted)).child(format!("{} included × {} = {}. {} {}", count_label(preview.included(), "page", "pages"), count_label(preview.settings.profiles.len(), "screen size", "screen sizes"), planned_capture_count(preview), preview.inventory.coverage.label(), if preview.included() == 0 { "Include at least one eligible page before capturing." } else if !self.discovery_plan_current() { "Settings changed. In Project settings, choose Find pages again before capturing." } else { "Adjust settings and find pages again to expand this list." })))
                    })
                    .when_some(self.capture_next_step(), |header, next| {
                        header.child(
                            div()
                                .debug_selector(|| "capture-next-step".into())
                                .text_size(px(14.))
                                .line_height(px(20.))
                                .when(
                                    !self.capture_is_saved_baseline() && self.evidence.latest.as_ref().is_some_and(|run| {
                                        run.run.state == JobState::CompletedWithErrors
                                    }),
                                    |notice| {
                                        notice
                                            .bg(rgb(palette().warning_surface))
                                            .px_2()
                                            .py_1()
                                            .rounded(px(4.))
                                    },
                                )
                                .text_color(rgb(
                                    if !self.capture_is_saved_baseline() && self.evidence.latest.as_ref().is_some_and(|run| {
                                        run.run.state == JobState::CompletedWithErrors
                                    }) {
                                        palette().warning
                                    } else {
                                        palette().ink
                                    },
                                ))
                                .child(next),
                        )
                    }),
            )
            .when(!pages, |body| body.child(self.coverage_summary()))
            .when(!pages && self.recorded_mode() == CaptureMode::ScreenshotsAndDomSnapshots, |body| body.child(self.evidence_kind_choices(cx)))
            .when(self.inspection_chrome_mounted(), |body| {
                body.child(self.evidence_toolbar(cx))
            })
            .child(
                div()
                    .debug_selector(|| "queue-beside-viewer".into())
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .relative()
                    .flex()
                    .gap_3()
                    .overflow_hidden()
                    // Keep overflow bounded, but leave room for edge controls'
                    // focus paint. Equal negative margins preserve the queue,
                    // viewer and evidence geometry instead of insetting them.
                    .m(px(-controls::FOCUS_OUTSET))
                    .p(px(controls::FOCUS_OUTSET))
                    .when(
                        pages
                            || (!self.review_queue_collapsed
                                && !self.authoritative_empty_capture()),
                        |workspace| {
                            workspace.child(
                                div()
                                    .debug_selector(|| "review-queue".into())
                                    .w(px(self.review_queue_width()))
                                    .h_full()
                                    .min_h_0()
                                    .flex_shrink_0()
                                    .child(self.inventory_and_results(cx)),
                            )
                        },
                    )
                    .when(
                        (!pages || self.discovery_image_mounted())
                            && !(pages && self.options_panel_open()),
                        |workspace| {
                            workspace.child(
                                div()
                                    .debug_selector(|| "review-viewer".into())
                                    .flex_1()
                                    .min_w_0()
                                    .h_full()
                                    .min_h_0()
                                    .flex()
                                    .flex_col()
                                    .gap_2()
                                    .when(self.review_queue_collapsed
                                        && !self.inspection_chrome_mounted()
                                        && !self.authoritative_empty_capture(), |viewer| {
                                        viewer.child(self.tool_control(Control::Guided(50), ToolIcon::Results, None, "Show results", false, cx))
                                    })
                                    .child(self.viewer_section(cx)),
                            )
                        },
                    )
                    .when(self.options_panel_open(), |workspace| {
                        workspace.child(self.options_panel(cx))
                    }),
            )
            .when(!pages && self.has_kind_comparison(), |workspace| {
                workspace.child(self.ordinary_review_bar(cx))
            })
            .into_any_element()
    }

    pub(super) fn displayed_run(&self) -> Option<&regressee_core::evidence::RunRecord> {
        let id = self.evidence.comparison.as_ref().map(|a| a.revision.run);
        self.evidence
            .latest
            .as_ref()
            .filter(|r| id.is_none_or(|id| id == r.run.id))
            .or_else(|| {
                self.history
                    .as_ref()
                    .and_then(|h| h.runs.iter().find(|r| Some(r.run.id) == id))
            })
    }

    fn compared_baseline(&self) -> Option<&regressee_core::domain::BaselineVersion> {
        let id = self.evidence.comparison.as_ref()?.revision.baseline;
        self.evidence
            .baseline
            .as_ref()
            .filter(|b| b.id == id)
            .or_else(|| self.evidence.baselines.iter().find(|b| b.id == id))
    }

    pub(super) fn comparison_context(&self) -> String {
        if let Some(preview) = &self.preview {
            return discovery_summary(preview);
        }
        if self.focus.review_surface == ReviewSurface::Pages {
            return if self
                .evidence
                .latest
                .as_ref()
                .zip(self.evidence.baseline.as_ref())
                .is_some_and(|(r, b)| r.run.inventory.id == b.inventory)
            {
                if self
                    .evidence
                    .baseline
                    .as_ref()
                    .is_some_and(|b| b.explicitly_partial)
                {
                    "Saved baseline page list · Checks use only saved screenshots; missing page/size pairs are skipped. Find pages to prepare a replacement.".into()
                } else {
                    "Saved baseline page list · Checks revisit included pages. Find pages to prepare a replacement.".into()
                }
            } else {
                "Latest capture page list · Find pages to prepare a new list; saved baseline unchanged.".into()
            };
        }
        if self.dom_active()
            && let Some(comparison) = &self.evidence.dom_comparison
        {
            let historical = self
                .project
                .as_ref()
                .is_some_and(|p| p.project.active_baseline != Some(comparison.revision.baseline));
            return format!(
                "{}baseline · DOM check {} · {}",
                if historical {
                    "Historical "
                } else {
                    "Compared "
                },
                short_id(comparison.revision.run.to_string()),
                self.displayed_run().map_or("environment not loaded", |r| r
                    .run
                    .environment
                    .name
                    .as_str())
            );
        }
        let reference = self
            .evidence
            .comparison
            .as_ref()
            .map(|a| {
                let historical = self
                    .project
                    .as_ref()
                    .is_some_and(|p| p.project.active_baseline != Some(a.revision.baseline));
                format!(
                    "{}baseline",
                    if historical {
                        "Historical "
                    } else {
                        "Compared "
                    }
                )
            })
            .unwrap_or_else(|| {
                if self.capture_is_saved_baseline() {
                    format!(
                        "Saved baseline {}",
                        short_id(self.evidence.baseline.as_ref().unwrap().id.to_string())
                    )
                } else {
                    "Capture only — no comparison performed".into()
                }
            });
        let check = self
            .displayed_run()
            .map(|r| {
                format!(
                    "{} · {}",
                    if matches!(r.run.intent, RunIntent::Check { .. }) {
                        "Check"
                    } else {
                        "Capture"
                    },
                    r.run.environment.name
                )
            })
            .unwrap_or_else(|| {
                self.evidence
                    .comparison
                    .as_ref()
                    .map(|a| {
                        format!(
                            "Check {} · record not loaded",
                            short_id(a.revision.run.to_string())
                        )
                    })
                    .unwrap_or_else(|| "No saved capture yet".into())
            });
        format!("{reference} · {check}")
    }

    pub(super) fn pane_caption(&self, pane: usize) -> (String, String, String) {
        if self.dom_active() {
            return self.dom_pane_caption(pane);
        }
        let key = self.selected_key.as_ref();
        let run = self.displayed_run();
        let profile = run
            .and_then(|r| {
                r.run
                    .profiles
                    .iter()
                    .find(|p| Some(p.id) == key.map(|k| k.profile))
            })
            .map(|p| format!("{} · {} × {} pixels", p.name, p.width, p.height))
            .unwrap_or_else(|| "Profile record not loaded".into());
        if pane == 0 && self.evidence.comparison.is_some() {
            let reference = self
                .compared_baseline()
                .and_then(|b| b.references.iter().find(|r| Some(&r.key) == key));
            let source = reference.and_then(|r| {
                self.history
                    .as_ref()
                    .and_then(|h| h.runs.iter().find(|run| run.run.id == r.run))
            });
            let capture = source
                .and_then(|r| key.and_then(|key| r.evidence(key)))
                .or_else(|| {
                    self.evidence.before.as_ref().filter(|c| {
                        Some(&c.metadata.key) == key
                            && reference.is_some_and(|r| r.artifact == c.image.original)
                    })
                });
            let origin = capture
                .map(|c| capture_origin(&c.metadata.final_url))
                .unwrap_or_else(|| "Origin record not loaded".into());
            let environment = self
                .viewer_reference_environment()
                .or_else(|| source.map(|r| r.run.environment.name.as_str()))
                .unwrap_or("environment not loaded");
            let profile = capture
                .map(|c| compact_profile(&c.metadata.profile))
                .unwrap_or_else(|| "Screen size not loaded".into());
            (
                "Before · baseline".into(),
                format!("{environment}\n{origin}"),
                profile,
            )
        } else {
            let capture = run.and_then(|r| key.and_then(|key| r.evidence(key)));
            let label = if self.evidence.comparison.is_none() && self.capture_is_saved_baseline() {
                "Baseline capture"
            } else if pane == 2 {
                if self.viewer_mode == 3 {
                    "Heatmap"
                } else {
                    "Difference"
                }
            } else {
                if self.evidence.comparison.is_some() {
                    "After · check"
                } else {
                    "Capture"
                }
            };
            let title = if run.is_some() {
                label.to_owned()
            } else {
                format!("{label} record not loaded")
            };
            let origin = capture
                .map(|c| capture_origin(&c.metadata.final_url))
                .unwrap_or_else(|| "no saved image".into());
            (
                title,
                format!(
                    "{}\n{origin}",
                    run.map(|r| r.run.environment.name.as_str())
                        .unwrap_or("environment not loaded")
                ),
                capture
                    .map(|c| compact_profile(&c.metadata.profile))
                    .unwrap_or(profile),
            )
        }
    }

    /// Persistent evidence identity comes from the inspected run/reference,
    /// never from today's project/environment settings or a filtered queue.
    pub(super) fn inspection_identity(&self) -> (String, String) {
        let run = self.displayed_run();
        let key = self.selected_key.as_ref();
        let address = run
            .and_then(|r| r.run.targets.iter().find(|t| Some(&t.key) == key))
            .map(|t| t.requested.to_string())
            .unwrap_or_else(|| "Target record not loaded".into());
        let (_, _, profile) = self.pane_caption(1);
        let (before, after) = self.inspection_sources();
        let context = if self.has_kind_comparison() {
            format!("{profile} · Before: {before} · After: {after}")
        } else {
            format!("{profile} · Capture: {after}")
        };
        (controls::page_path(&address), context)
    }

    /// Reference context travels with the inspected pair, not with whichever
    /// History page happened to be opened. Never borrow a previous target's
    /// environment while a newer image request is in flight.
    fn viewer_reference_environment(&self) -> Option<&str> {
        let frame = self.viewer.as_ref()?;
        let identity = frame.identity.as_ref()?;
        let run = self
            .evidence
            .comparison
            .as_ref()
            .map(|a| a.revision.run)
            .or_else(|| self.evidence.latest.as_ref().map(|r| r.run.id))?;
        if self.project.as_ref()?.project.id != identity.project
            || run != identity.run
            || self.selected_key.as_ref() != Some(&identity.key)
            || self.evidence.comparison.as_ref().map(|a| a.revision.id) != identity.analysis
        {
            return None;
        }
        frame.before_environment.as_deref()
    }

    pub(super) fn approval_decision(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.recorded_mode().dom_snapshots() {
            return self.kind_approval_decision(cx);
        }
        let Some(preview) = self.approval.as_ref().filter(|p| self.approval_matches(p)) else {
            return div().into_any_element();
        };
        let index = self.review_index();
        let selected: BTreeSet<_> = preview.selected.iter().cloned().collect();
        let provenance: BTreeMap<_, _> = preview
            .provenance
            .iter()
            .map(|(key, requested, final_url)| {
                (
                    key.clone(),
                    format!("Requested URL: {requested}\nFinal URL: {final_url}"),
                )
            })
            .collect();
        let rows = preview.baseline.references.len();
        if rows == 0 {
            return div().into_any_element();
        }
        let mut order: Vec<_> = (0..rows).collect();
        order.sort_by_key(|i| !selected.contains(&preview.baseline.references[*i].key));
        let inspected = &preview.baseline.references
            [order[self.approval_reference_index.min(rows.saturating_sub(1))]];
        let exact = format!(
            "Page: {}\nProfile: {}\nSource run: {}\nCaptured: {}\nOriginal: {}\n{}\nPage ID: {}\nProfile ID: {}",
            index
                .target_labels
                .get(&inspected.key)
                .cloned()
                .unwrap_or_else(|| inspected.key.page.to_string()),
            index
                .profile_labels
                .get(&inspected.key.profile)
                .cloned()
                .unwrap_or_else(|| inspected.key.profile.to_string()),
            inspected.run,
            capture_time(inspected.captured_at),
            inspected.artifact.as_str(),
            provenance
                .get(&inspected.key)
                .cloned()
                .unwrap_or_else(|| "Inherited provenance retained in History".into()),
            inspected.key.page,
            inspected.key.profile
        );
        let role = if selected.contains(&inspected.key) {
            "Replace"
        } else {
            "Keep"
        };
        let environments: BTreeMap<_, _> = preview.source_environments.iter().cloned().collect();
        let cautioned = self.evidence.latest.as_ref().map_or(0, |run| {
            run.all_captures()
                .filter(|c| selected.contains(&c.metadata.key) && !c.metadata.warnings.is_empty())
                .count()
        });
        let environment = environments
            .get(&inspected.key)
            .cloned()
            .unwrap_or_else(|| "Source environment recorded in History".into());
        let mut flow = div().flex().flex_col().gap_3()
            .child(div().debug_selector(|| "approval-impact-summary".into()).child(format!("{} replaced · {} unchanged", count_label(preview.selected.len(), "screenshot", "screenshots"), preview.inherited)))
            .child("Future checks will compare against the selected After screenshots. Other baseline screenshots stay unchanged. Earlier versions remain in History.")
            .when(cautioned > 0, |flow| flow.child(div().debug_selector(|| "approval-capture-caution".into())
                .text_size(px(13.)).line_height(px(HELP_LINE_HEIGHT)).text_color(rgb(palette().warning))
                .child("Some selected screenshots have loading warnings. Their captured appearance will become the baseline.")))
            .child("Capture times may differ; this is not one simultaneous site snapshot.")
            .child(div().font_weight(FontWeight::SEMIBOLD).child(format!("Inspecting {role} · screenshot {} of {rows}", self.approval_reference_index + 1)))
            .child(format!("{} · {} · {environment} · {}", index.target_labels.get(&inspected.key).map(|url| controls::page_path(url)).unwrap_or_else(|| inspected.key.page.to_string()), index.profile_labels.get(&inspected.key.profile).cloned().unwrap_or_else(|| inspected.key.profile.to_string()), capture_time(inspected.captured_at)))
            .child(self.inline_disclosure(Control::Guided(52), "Screenshot source details", "", self.approval_exact_open,
                div().id("approval-exact-provenance").debug_selector(|| "approval-exact-provenance".into()).text_size(px(14.)).line_height(px(20.)).child(exact), cx));
        // Bound mounted rows, without dropping any unchanged reference. Previous/
        // Next visits every reference and switches the visible 32-row page.
        let start = self.approval_reference_index / 32 * 32;
        flow = flow.child(format!(
            "Screenshots {}–{} of {rows} · Previous/Next inspects every screenshot.",
            start + 1,
            (start + 32).min(rows)
        ));
        for (i, &position) in order.iter().enumerate().skip(start).take(32) {
            let r = &preview.baseline.references[position];
            let page = index
                .target_labels
                .get(&r.key)
                .map(|url| controls::page_path(url))
                .unwrap_or_else(|| format!("Page {}", r.key.page));
            let profile = index
                .profile_labels
                .get(&r.key.profile)
                .cloned()
                .unwrap_or_else(|| format!("Profile {}", r.key.profile));
            flow = flow.child(
                div()
                    .id(("approval-reference", i))
                    .debug_selector(move || format!("approval-reference-{i}"))
                    .flex_shrink_0()
                    .py_3()
                    .px_2()
                    .border_b_1()
                    .border_color(rgb(palette().line))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .line_height(px(20.))
                    .when(i == self.approval_reference_index, |row| {
                        row.bg(rgb(palette().selected))
                    })
                    .child(
                        div()
                            .debug_selector(move || format!("approval-reference-title-{i}"))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!(
                                "{} · {page}",
                                if selected.contains(&r.key) {
                                    "Replace"
                                } else {
                                    "Keep"
                                }
                            )),
                    )
                    .child(
                        div()
                            .text_size(px(14.))
                            .line_height(px(CONTROL_LINE_HEIGHT))
                            .child(profile),
                    )
                    .child(
                        div()
                            .debug_selector(move || format!("approval-reference-time-{i}"))
                            .text_size(px(14.))
                            .line_height(px(CONTROL_LINE_HEIGHT))
                            .child(format!(
                                "{} · {}",
                                environments
                                    .get(&r.key)
                                    .map(String::as_str)
                                    .unwrap_or("Source environment recorded in History"),
                                capture_time(r.captured_at)
                            )),
                    )
                    .on_click(cx.listener(move |this, _, _, cx| {
                        if this.agent_locked() {
                            return;
                        }
                        this.approval_reference_index = i;
                        cx.notify();
                    })),
            );
        }
        let panel =
            regressee_gui::ui::dialog::surface(if self.narrow_workspace { 740. } else { 760. })
                .debug_selector(|| "approval-decision".into())
                .h(px(520.))
                .p_6()
                .gap_4()
                .child(
                    regressee_gui::ui::dialog::heading().child(if preview.selected.len() == 1 {
                        "Use this After screenshot for future checks?"
                    } else {
                        "Use these After screenshots for future checks?"
                    }),
                )
                .child(scrollbar::vertical(
                    "approval-scrollbar",
                    &self.approval_provenance_scroll,
                    div()
                        .id("approval-reference-list")
                        .debug_selector(|| "approval-reference-list".into())
                        .flex_1()
                        .min_h_0()
                        .w_full()
                        .overflow_y_scroll()
                        .track_scroll(&self.approval_provenance_scroll)
                        .pr(px(scrollbar::GUTTER))
                        .child(flow),
                ))
                .child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(HELP_LINE_HEIGHT))
                        .text_color(rgb(palette().muted))
                        .child(
                            "Scroll to see more screenshots. Source details are available above.",
                        ),
                )
                .child(
                    regressee_gui::ui::dialog::footer()
                        .flex()
                        .flex_wrap()
                        .justify_between()
                        .gap_2()
                        .child(self.guided_button(48, "Previous", cx))
                        .child(self.guided_button(49, "Next", cx))
                        .child(self.guided_button(47, "Keep reviewing", cx))
                        .child(self.button(
                            Control::Phase5(25),
                            "Save baseline update",
                            ActionStyle::Primary,
                            cx,
                        )),
                );
        regressee_gui::ui::dialog::present("approval-dialog-entrance", panel, cx)
            .id("approval-modal-backdrop")
            .into_any_element()
    }

    /// Conservative presentation eligibility. Storage repeats the complete
    /// provenance/profile checks when previewing and accepting a selection.
    pub(super) fn approval_eligible(&self, key: &TargetKey) -> bool {
        self.review_index().eligible.contains(key)
    }

    pub(super) fn review_index(&self) -> ReviewIndex {
        let active = if self.history_inspection.is_some()
            || self
                .kind_read_error(regressee_core::domain::CheckKind::Screenshot)
                .is_some()
            || self.viewer_read_failed
            || self.mask_dirty
            || self.analysis_due.is_some()
            || self.analysis_pending.is_some()
        {
            None
        } else {
            self.project
                .as_ref()
                .and_then(|p| p.project.active_baseline)
        };
        ReviewIndex::build(&self.evidence, active)
    }

    pub(super) fn approval_selection_eligible(&self) -> bool {
        if self.recorded_mode().dom_snapshots() {
            return self.kind_approval_selection_valid();
        }
        let index = self.review_index();
        !self.approval_selection.is_empty()
            && self
                .approval_selection
                .iter()
                .all(|key| index.eligible.contains(key))
    }

    pub(super) fn approval_unavailable_reason(
        &self,
        key: Option<&TargetKey>,
    ) -> Option<&'static str> {
        if self.mask_dirty {
            return Some("Save ignored areas first.");
        }
        if self.analysis_due.is_some() || self.analysis_pending.is_some() {
            return Some("Wait for the updated analysis.");
        }
        let Some(comparison) = &self.evidence.comparison else {
            return Some("A completed comparison is required.");
        };
        if self
            .project
            .as_ref()
            .and_then(|p| p.project.active_baseline)
            != Some(comparison.revision.baseline)
        {
            return Some(
                "This check used an earlier baseline. Run a check against the current baseline before updating it.",
            );
        }
        let Some(key) = key else {
            return Some("Inspect a page and screen size first.");
        };
        if self.approval_eligible(key) {
            return None;
        }
        let Some(result) = comparison.revision.results.iter().find(|r| &r.key == key) else {
            return Some("Analysis is not available for this screenshot.");
        };
        match comparison
            .review_classification(key)
            .unwrap_or(result.classification)
        {
            Classification::CouldNotCapture => Some(
                "This screenshot was not captured, so it cannot replace a baseline screenshot.",
            ),
            Classification::NotComparable => {
                Some("These screenshots could not be compared, so they cannot update the baseline.")
            }
            _ => Some(
                "This screenshot cannot update the baseline because of capture warnings, missing originals or comparison problems. Open Details to learn more.",
            ),
        }
    }

    pub(super) fn ordinary_review_status(&self) -> &'static str {
        if self.dom_active() {
            if self.dom_result().is_some_and(|r| {
                r.outcome == regressee_core::dom_snapshot::DomComparisonOutcome::Unchanged
            }) {
                return "DOM snapshots are identical";
            }
            return match self.dom_note_template().map(|d| d.state) {
                Some(regressee_core::domain::ReviewState::Intentional) => "Reviewed · intentional",
                Some(regressee_core::domain::ReviewState::NeedsFix) => "Reviewed · needs fix",
                Some(regressee_core::domain::ReviewState::Unreviewed) => "Unreviewed DOM changes",
                None => "Not compared",
            };
        }
        let Some(comparison) = &self.evidence.comparison else {
            return "Not compared";
        };
        let Some(key) = &self.selected_key else {
            return "Choose a screenshot";
        };
        match current_review_state(&self.evidence, comparison.revision.id, key) {
            regressee_core::domain::ReviewState::Intentional => "Reviewed · intentional",
            regressee_core::domain::ReviewState::NeedsFix => "Reviewed · needs fix",
            regressee_core::domain::ReviewState::Unreviewed => {
                if comparison.review_classification(key)
                    == Some(Classification::NoSignificantChange)
                {
                    "No significant change"
                } else {
                    "Not reviewed for this analysis"
                }
            }
        }
    }

    pub(super) fn review_progress_summary(&self) -> String {
        if self.dom_active() {
            return self.dom_review_progress();
        }
        let Some(comparison) = &self.evidence.comparison else {
            return "No comparison to review.".into();
        };
        let progress = ReviewProgress::from_evidence(&self.evidence);
        let baseline_saved = self.evidence.baseline.as_ref().is_some_and(|b| {
            b.references
                .iter()
                .any(|r| r.run == comparison.revision.run)
        });
        let mut parts = Vec::new();
        if progress.reviewed > 0 {
            parts.push(format!("{} reviewed", progress.reviewed));
        }
        parts.push(match progress.remaining {
            0 => "No changes to review".into(),
            1 => "1 change to review".into(),
            n => format!("{n} changes to review"),
        });
        if progress.needs_fix > 0 {
            parts.push(format!("{} marked Needs fix", progress.needs_fix));
        }
        if progress.gaps > 0 {
            parts.push(format!("{} not compared", progress.gaps));
        }
        if progress.awaiting > 0 {
            parts.push(format!("{} awaiting analysis", progress.awaiting));
        }
        if baseline_saved {
            parts.push("Baseline updates saved".into());
        }
        parts.join(" · ")
    }

    pub(super) fn baseline_update_mounted(&self) -> bool {
        if self.recorded_mode().dom_snapshots() {
            return !self.approval_selection.is_empty()
                || !self.dom.selected.is_empty()
                || self.selected_key.as_ref().is_some_and(|key| {
                    if self.dom_active() {
                        self.dom_approval_candidate(key).is_some()
                    } else {
                        self.approval_eligible(key)
                    }
                });
        }
        !self.approval_selection.is_empty()
            || self
                .selected_key
                .as_ref()
                .is_some_and(|key| self.approval_eligible(key))
    }

    pub(super) fn approval_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.recorded_mode().dom_snapshots() {
            return self.kind_approval_bar(cx);
        }
        let index = self.review_index();
        let eligible = self
            .approval_selection
            .iter()
            .filter(|key| index.eligible.contains(*key))
            .count();
        let selected = self
            .selected_key
            .as_ref()
            .is_some_and(|key| self.approval_selection.contains(key));
        let stacked = self.working_width < 900.;
        div()
            .debug_selector(|| "approval-next-action".into())
            .shadow(regressee_gui::ui::depth::Elevation::Footer.shadows())
            .flex_shrink_0()
            .flex()
            .items_center()
            .when(stacked, |bar| bar.flex_col().items_start())
            .when(!stacked, |bar| bar.flex_wrap())
            .gap_3()
            .px_6()
            .py_2()
            .bg(rgb(palette().surface))
            .border_t_1()
            .border_color(rgb(palette().line))
            .child(
                div()
                    .when(stacked, |copy| copy.w_full().flex_shrink_0())
                    .when(!stacked, |copy| copy.flex_1())
                    .min_w(px(200.))
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div().flex().flex_wrap().items_baseline().gap_2()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child("Update baseline"))
                            .when(!self.approval_selection.is_empty(), |heading| heading.child(
                                div().debug_selector(|| "baseline-selection-label".into())
                                    .text_size(px(13.)).line_height(px(18.)).text_color(rgb(palette().muted))
                                    .child(crate::evidence_workspace::baseline_selection_label(self.approval_selection.len(), eligible)),
                            )),
                    )
                    .child(
                        div().debug_selector(|| "baseline-update-explanation".into())
                            .text_size(px(13.)).line_height(px(18.)).text_color(rgb(palette().muted))
                            .child("Use After as the reference for future checks. Review before saving."),
                    ),
            )
            .child(
                div().flex().flex_shrink_0().flex_wrap().gap_2()
                    .when(self.control_mounted(Control::Phase5(23)), |actions| actions.child(self.button(
                          Control::Phase5(23),
                          if selected {
                              "Deselect After screenshot"
                          } else {
                              "Select After screenshot"
                        },
                         ActionStyle::Choice { selected },
                         cx,
                    )))
                    .when(self.control_mounted(Control::Phase5(24)), |actions| actions.child(self.button(
                         Control::Phase5(24),
                          "Review update…",
                         ActionStyle::Primary,
                         cx,
                    ))),
            )
            .when(!self.approval_selection.is_empty() && !self.approval_selection_eligible(), |bar| {
                let reason = selected.then(|| self.approval_unavailable_reason(self.selected_key.as_ref())).flatten();
                bar.child(div().w_full().text_size(px(13.)).line_height(px(18.)).text_color(rgb(palette().warning)).child(reason.unwrap_or("A selected screenshot can no longer update the baseline. Remove it from the selection or refresh results before confirming.")))
            })
            .into_any_element()
    }

    pub(super) fn setup_heading(&self, cx: &mut Context<Self>) -> AnyElement {
        let (title, next) = match self.focus.setup_stage {
            SetupStage::Website => (
                "Set up your website",
                "Save your project settings on this computer. Next, choose whether login is needed.",
            ),
            SetupStage::Access => (
                "Website access",
                "Choose public pages, or save access settings and log in.",
            ),
            SetupStage::Pages => (
                "Pages & screen sizes",
                "Save your choices, then find pages and review the list before capturing.",
            ),
            SetupStage::Settings => (
                "Project settings",
                "Edit future capture settings. Saving never changes your active baseline.",
            ),
        };
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.heading(title, next))
            .when(self.focus.setup_stage != SetupStage::Settings, |body| {
                body.child(self.setup_progress(cx))
            })
            .into_any_element()
    }

    pub(super) fn stage_control_mounted(&self, control: Control) -> bool {
        if self.focus.screen == Screen::Setup
            && matches!(
                self.focus.setup_stage,
                SetupStage::Pages | SetupStage::Settings
            )
        {
            return match control {
                Control::CrawlField(1..=8)
                | Control::Guided(7..=10 | 12 | 25)
                | Control::Choice(Choice::Gentle | Choice::Balanced | Choice::Fast) => {
                    self.focus.advanced
                }
                _ => true,
            };
        }
        if self.focus.screen != Screen::Setup || self.focus.setup_stage != SetupStage::Access {
            return true;
        }
        let login = self
            .draft_settings
            .environments
            .iter()
            .find(|e| Some(e.id) == self.environment_draft)
            .is_some_and(|e| e.access == regressee_core::domain::AccessPolicy::InteractiveLogin);
        match control {
            Control::Guided(6) => false,
            Control::CrawlField(10..=12)
            | Control::Choice(Choice::SaveLogin | Choice::EphemeralLogin) => {
                self.setup_login_fields_visible()
            }
            Control::Guided(31..=34) => login,
            Control::Environment(_)
            | Control::CrawlField(13..=16)
            | Control::Guided(27 | 35..=38)
            | Control::CloseMappingPreview
            | Control::Choice(Choice::LightTheme | Choice::DarkTheme) => {
                self.setup_environment_open
            }
            _ => true,
        }
    }

    pub(super) fn setup_footer(&self, cx: &mut Context<Self>) -> AnyElement {
        let stage = self.focus.setup_stage;
        div()
            .debug_selector(|| "setup-footer".into())
            .shadow(regressee_gui::ui::depth::Elevation::Footer.shadows())
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .px_6()
            .py_3()
            .bg(rgb(palette().surface))
            .border_t_1()
            .border_color(rgb(palette().line))
            .when(
                matches!(stage, SetupStage::Access | SetupStage::Pages),
                |body| {
                    body.child(self.button(Control::SetupBack, "Back", ActionStyle::Secondary, cx))
                },
            )
            .child(self.button(
                Control::Save,
                if stage == SetupStage::Website {
                    if self.project.is_some() && !self.unsaved(cx) {
                        "Continue to access"
                    } else {
                        "Save project & continue"
                    }
                } else if stage == SetupStage::Access {
                    if self.access_can_continue(cx) {
                        "Continue to pages & sizes"
                    } else if self.unsaved(cx)
                        && self
                            .draft_settings
                            .environments
                            .iter()
                            .find(|e| Some(e.id) == self.environment_draft)
                            .is_some_and(|e| {
                                e.access == regressee_core::domain::AccessPolicy::InteractiveLogin
                            })
                    {
                        "Save settings first"
                    } else if self.unsaved(cx) {
                        "Save & continue"
                    } else {
                        "Save settings"
                    }
                } else {
                    "Save settings"
                },
                if stage == SetupStage::Pages
                    || (stage == SetupStage::Access
                        && !self.unsaved(cx)
                        && !self.access_can_continue(cx))
                {
                    ActionStyle::Secondary
                } else {
                    ActionStyle::Primary
                },
                cx,
            ))
            .when(
                stage == SetupStage::Access
                    && self
                        .draft_settings
                        .environments
                        .iter()
                        .find(|e| Some(e.id) == self.environment_draft)
                        .is_some_and(|e| {
                            e.access == regressee_core::domain::AccessPolicy::InteractiveLogin
                        }),
                |body| {
                    body.child(self.button(
                        Control::Guided(31),
                        "Open login browser",
                        if !self.unsaved(cx)
                            && !self.access_can_continue(cx)
                            && self.environment.as_ref().is_some_and(|e| !e.login_open)
                        {
                            ActionStyle::Primary
                        } else {
                            ActionStyle::Secondary
                        },
                        cx,
                    ))
                    .child(self.button(
                        Control::Guided(32),
                        "Confirm login",
                        if !self.unsaved(cx)
                            && self.environment.as_ref().is_some_and(|e| e.login_open)
                        {
                            ActionStyle::Primary
                        } else {
                            ActionStyle::Secondary
                        },
                        cx,
                    ))
                },
            )
            .when(
                self.project.is_some() && matches!(stage, SetupStage::Pages | SetupStage::Settings),
                |body| {
                    body.child(self.button(
                        Control::Guided(13),
                        if self.unsaved(cx) {
                            "Save & find pages"
                        } else {
                            "Find pages"
                        },
                        if stage == SetupStage::Pages {
                            ActionStyle::Primary
                        } else {
                            ActionStyle::Secondary
                        },
                        cx,
                    ))
                },
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(180.))
                    .min_h(px(CONTROL_HEIGHT))
                    .grid()
                    .grid_cols(1)
                    .text_size(px(13.))
                    .line_height(px(HELP_LINE_HEIGHT))
                    .text_color(rgb(palette().muted))
                    .debug_selector(|| "setup-footer-status".into())
                    .child(div().col_start(1).row_start(1).when(!self.form_errors(cx).is_empty(), |copy| copy.invisible()).child(if stage == SetupStage::Access {
                        if self.unsaved(cx) {
                            "Draft access: save first. Login opens separately."
                        } else if self.environment.as_ref().is_some_and(|e| e.login_open) {
                            "Saved access · browser open: finish login, then choose Confirm login."
                        } else if self.access_can_continue(cx) {
                            "Saved access: continue. Login status is checked separately."
                        } else if self.environment.is_none() {
                            "Loading saved browser status. Access choices are kept."
                        } else {
                            "Saved access: open login browser, then choose Confirm login."
                        }
                    } else if self.project.is_none() {
                        "Save the project settings first. No website is visited."
                    } else if self.unsaved(cx) {
                        "Save before visiting the website. Existing baseline unchanged."
                    } else if self.draft_settings.capture_plan.mode.dom_snapshots() {
                        "Settings saved. Capture the selected kinds, then choose Save baseline to use them for comparisons."
                    } else {
                        "Settings saved. Capture screenshots, then choose Save baseline to use them for comparisons."
                    }))
                    .child(div().col_start(1).row_start(1).child(self.form_error_summary(cx))),
            )
            .into_any_element()
    }

    pub(super) fn project_rail(&self, cx: &mut Context<Self>) -> AnyElement {
        let project = self.project.is_some();
        div()
            .debug_selector(|| "project-rail".into())
            .shadow(regressee_gui::ui::depth::Elevation::Rail.shadows())
            .w(px(192.))
            .flex_shrink_0()
            .h_full()
            .p_3()
            .bg(rgb(palette().rail))
            .text_color(rgb(palette().rail_ink))
            .flex()
            .flex_col()
            .gap_2()
            .child(div().px_2().pt_3().pb_3().child(brand::navigation_lockup()))
            .child(self.button(
                Control::Navigate(Screen::Projects),
                "Projects",
                ActionStyle::Navigation {
                    selected: self.focus.screen == Screen::Projects,
                },
                cx,
            ))
            .when(project, |rail| {
                rail.child(
                    div()
                        .px_3()
                        .py_4()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .debug_selector(|| "rail-project-name".into())
                                .text_size(px(BODY_TEXT_SIZE))
                                .line_height(px(BODY_LINE_HEIGHT))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_ellipsis()
                                .child(self.project.as_ref().unwrap().project.name.clone()),
                        )
                        .child(
                            div()
                                .debug_selector(|| "rail-baseline-status".into())
                                .text_size(px(13.))
                                .line_height(px(HELP_LINE_HEIGHT))
                                .font_weight(FontWeight::NORMAL)
                                .text_color(rgb(palette().rail_muted))
                                .child(self.baseline_rail_status()),
                        ),
                )
            })
            .child(self.button(
                Control::Navigate(Screen::Results),
                if project && !self.focus.baseline && self.evidence.latest.is_some() {
                    "Inspect capture"
                } else {
                    "Review changes"
                },
                ActionStyle::Navigation {
                    selected: self.focus.screen == Screen::Results
                        && self.focus.review_surface != ReviewSurface::Pages
                        && self.preview.is_none(),
                },
                cx,
            ))
            .when(project, |rail| {
                rail.child(self.button(
                    Control::PagesBaseline,
                    "Pages & baseline",
                    ActionStyle::Navigation {
                        selected: self.focus.screen == Screen::Results
                            && self.focus.review_surface == ReviewSurface::Pages,
                    },
                    cx,
                ))
            })
            .child(self.button(
                Control::Navigate(Screen::History),
                "History",
                ActionStyle::Navigation {
                    selected: self.focus.screen == Screen::History,
                },
                cx,
            ))
            .child(self.button(
                Control::Navigate(Screen::Setup),
                if project {
                    "Project settings"
                } else {
                    "Website setup"
                },
                ActionStyle::Navigation {
                    selected: self.focus.screen == Screen::Setup,
                },
                cx,
            ))
            .child(div().flex_1())
            .child(
                div()
                    .border_t_1()
                    .border_color(rgb(palette().rail_hover))
                    .pt_3()
                    .child(self.button(
                        Control::Navigate(Screen::Settings),
                        "App settings",
                        ActionStyle::Navigation {
                            selected: self.focus.screen == Screen::Settings,
                        },
                        cx,
                    )),
            )
            .into_any_element()
    }

    pub(super) fn coverage_summary(&self) -> AnyElement {
        if self.recorded_mode().dom_snapshots() {
            return self.kind_coverage_summary();
        }
        let Some(run) = &self.evidence.latest else {
            return div().into_any_element();
        };
        let coverage = Coverage::from_run(run, &self.review_index());
        let metrics = if self.evidence.comparison.is_none() {
            let pending = run
                .run
                .targets
                .iter()
                .filter(|target| matches!(target.outcome, TargetOutcome::Pending))
                .count();
            let mut metrics = vec![
                (
                    format!("{} / {}", coverage.unclassified, coverage.total),
                    "captured",
                    false,
                ),
                (
                    coverage.missing.saturating_sub(pending).to_string(),
                    "not captured",
                    coverage.missing > pending,
                ),
            ];
            if pending > 0 {
                metrics.push((pending.to_string(), "pending", false));
            }
            metrics
        } else {
            let mut metrics = vec![
                (coverage.changed.to_string(), "changed", false),
                (
                    coverage.unchanged.to_string(),
                    "no significant change",
                    false,
                ),
                (
                    coverage.missing.to_string(),
                    "not compared",
                    coverage.missing > 0,
                ),
            ];
            if coverage.unclassified > 0 {
                metrics.push((
                    coverage.unclassified.to_string(),
                    "awaiting analysis",
                    false,
                ));
            }
            metrics
        };
        let scope = match run.run.inventory.coverage {
            regressee_core::domain::DiscoveryCoverage::Complete => None,
            regressee_core::domain::DiscoveryCoverage::Limited { .. } => Some("Limited page list"),
            regressee_core::domain::DiscoveryCoverage::Interrupted => {
                Some("Page search interrupted")
            }
        };
        let kind = if self.evidence.comparison.is_some() {
            "Comparison"
        } else if run.run.intent == RunIntent::RepresentativePreview {
            "Preview only"
        } else if self.capture_is_saved_baseline() {
            "Saved baseline"
        } else {
            "Not compared"
        };
        div()
            .debug_selector(|| "real-coverage-summary".into())
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .py_1()
            .border_b_1()
            .border_color(rgb(palette().line))
            .text_size(px(13.))
            .line_height(px(20.))
            .child(
                div()
                    .px_2()
                    .bg(rgb(palette().surface))
                    .border_1()
                    .border_color(rgb(palette().line))
                    .rounded(px(3.))
                    .text_color(rgb(palette().muted))
                    .child(kind),
            )
            .children(metrics.into_iter().map(|(value, label, warning)| {
                div()
                    .flex()
                    .items_center()
                    .gap_1()
                    .text_color(rgb(if warning {
                        palette().warning
                    } else {
                        palette().ink
                    }))
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .text_size(px(15.))
                            .line_height(px(BODY_LINE_HEIGHT))
                            .child(value),
                    )
                    .child(label)
            }))
            .when(
                !self.narrow_workspace || run.run.inventory.pages.iter().any(|p| !p.included),
                |summary| {
                    let included = run
                        .run
                        .inventory
                        .pages
                        .iter()
                        .filter(|p| p.included)
                        .count();
                    let excluded = run.run.inventory.pages.len() - included;
                    let selection = if excluded > 0 {
                        format!(
                            "{included} / {} URLs included · {excluded} excluded before capture",
                            run.run.inventory.pages.len()
                        )
                    } else {
                        format!(
                            "{} · {}",
                            count_label(included, "page", "pages"),
                            count_label(run.run.profiles.len(), "screen size", "screen sizes")
                        )
                    };
                    summary.child(div().text_color(rgb(palette().muted)).child(selection))
                },
            )
            .when_some(scope, |summary, scope| {
                summary.child(div().text_color(rgb(palette().warning)).child(scope))
            })
            .when_some(
                self.evidence
                    .baseline
                    .as_ref()
                    .filter(|b| b.explicitly_partial && !self.capture_is_saved_baseline()),
                |summary, baseline| {
                    summary.child(div().text_color(rgb(palette().warning)).child(format!(
                        "Baseline: {} / {} screenshots · {} skipped",
                        baseline.references.len(),
                        baseline.intended_count,
                        baseline.intended_count as usize - baseline.references.len()
                    )))
                },
            )
            .into_any_element()
    }
}

pub(super) fn count_label(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

pub(super) fn compact_profile(profile: &regressee_core::domain::CaptureProfile) -> String {
    format!(
        "{} · {} × {} pixels",
        profile.name, profile.width, profile.height
    )
}

fn discovery_summary(preview: &DiscoveryPreview) -> String {
    format!(
        "{} included · {} excluded · {} · {} Review the list before capturing; baseline unchanged.",
        count_label(preview.included(), "page", "pages"),
        preview
            .inventory
            .pages
            .len()
            .saturating_sub(preview.included()),
        count_label(
            preview.settings.profiles.len(),
            "screen size",
            "screen sizes"
        ),
        preview.inventory.coverage.label()
    )
}

fn planned_capture_count(preview: &DiscoveryPreview) -> String {
    let count = preview.included() * preview.settings.profiles.len();
    match preview.settings.capture_plan.mode {
        CaptureMode::ScreenshotsOnly => count_label(count, "screenshot", "screenshots"),
        CaptureMode::DomSnapshotsOnly => count_label(count, "DOM snapshot", "DOM snapshots"),
        CaptureMode::ScreenshotsAndDomSnapshots => {
            format!("{} screenshots + {count} DOM snapshots", count)
        }
    }
}

fn viewer_pane_width(working: f32, queue: f32, panes: usize) -> f32 {
    let available = working - queue - if queue > 0. { 12. } else { 0. };
    ((available - if panes == 2 { 12. } else { 0. }) / panes.max(1) as f32).max(1.)
}

fn current_review_state(
    evidence: &ProjectEvidence,
    analysis: regressee_core::domain::AnalysisId,
    key: &TargetKey,
) -> regressee_core::domain::ReviewState {
    evidence
        .reviews
        .iter()
        .filter(|r| r.analysis == analysis && &r.key == key)
        .max_by_key(|r| r.reviewed_at)
        .map(|r| r.state)
        .unwrap_or(regressee_core::domain::ReviewState::Unreviewed)
}

#[derive(Default, Debug, PartialEq, Eq)]
struct ReviewProgress {
    reviewed: usize,
    remaining: usize,
    needs_fix: usize,
    gaps: usize,
    awaiting: usize,
}
impl ReviewProgress {
    fn from_evidence(evidence: &ProjectEvidence) -> Self {
        use regressee_core::domain::ReviewState;
        let Some(comparison) = &evidence.comparison else {
            return Self::default();
        };
        let mut progress = Self::default();
        let mut decisions: BTreeMap<&TargetKey, &regressee_core::domain::ReviewDecision> =
            BTreeMap::new();
        for decision in evidence
            .reviews
            .iter()
            .filter(|r| r.analysis == comparison.revision.id)
        {
            if decisions
                .get(&decision.key)
                .is_none_or(|old| old.reviewed_at <= decision.reviewed_at)
            {
                decisions.insert(&decision.key, decision);
            }
        }
        let analyzed: BTreeSet<_> = comparison.revision.results.iter().map(|r| &r.key).collect();
        let classifications = comparison.review_classifications();
        for result in &comparison.revision.results {
            let state = decisions
                .get(&result.key)
                .map(|r| r.state)
                .unwrap_or(ReviewState::Unreviewed);
            if state != ReviewState::Unreviewed {
                progress.reviewed += 1;
            }
            if state == ReviewState::NeedsFix {
                progress.needs_fix += 1;
            }
            match classifications
                .get(&result.key)
                .copied()
                .unwrap_or(result.classification)
            {
                Classification::Changed if state == ReviewState::Unreviewed => {
                    progress.remaining += 1
                }
                Classification::CouldNotCapture | Classification::NotComparable => {
                    progress.gaps += 1
                }
                _ => {}
            }
        }
        if let Some(run) = evidence
            .latest
            .as_ref()
            .filter(|r| r.run.id == comparison.revision.run)
        {
            for target in &run.run.targets {
                if !analyzed.contains(&target.key) {
                    if matches!(target.outcome, TargetOutcome::Captured { .. })
                        && run.evidence(&target.key).is_some()
                    {
                        progress.awaiting += 1;
                    } else {
                        progress.gaps += 1;
                    }
                }
            }
        }
        progress
    }
}

fn short_id(id: String) -> String {
    id.chars().take(8).collect()
}
fn capture_origin(address: &str) -> String {
    url::Url::parse(address)
        .map(|u| u.origin().ascii_serialization())
        .unwrap_or_else(|_| "Origin record invalid".into())
}

/// Exact UTC civil time, never a claim about the user's local timezone.
pub(super) fn capture_time(timestamp: i64) -> String {
    let days = timestamp.div_euclid(86_400);
    let seconds = timestamp.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn history_last_rows_reveal_and_select_at_supported_sizes(cx: &mut gpui::TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let mut evidence = store.project_evidence(project).unwrap();
        let mut history = store.history(project, 0).unwrap();
        let baseline = evidence.baselines[0].clone();
        evidence.baselines = (0..128)
            .map(|i| {
                let mut version = baseline.clone();
                if i != 0 {
                    version.id = regressee_core::domain::BaselineId::new();
                }
                version
            })
            .collect();
        let run = history.runs[0].clone();
        history.runs = (0..64)
            .map(|i| {
                let mut record = run.clone();
                if i != 0 {
                    record.run.id = regressee_core::domain::RunId::new();
                }
                record
            })
            .collect();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            let startup = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
            s.apply_update(startup, cx);
            s.project = Some(record);
            s.evidence = evidence;
            s.evidence_dirty = false;
            s.history = Some(history);
            s.focus.screen = Screen::History;
            s.focus.project_saved = true;
            s.focus.history_baselines = s.evidence.baselines.len();
            s.focus.history_runs = s.history.as_ref().unwrap().runs.len();
        });
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            for (control, list, row_selector) in [
                (
                    Control::HistoryBaseline(127),
                    "history-baseline-list",
                    "history-row-HistoryBaseline(127)",
                ),
                (
                    Control::HistoryRun(63),
                    "history-run-list",
                    "history-row-HistoryRun(63)",
                ),
            ] {
                shell.update(cx, |s, cx| {
                    s.history_section = if matches!(control, Control::HistoryBaseline(_)) {
                        1
                    } else {
                        0
                    };
                    s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                    cx.notify();
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        assert!(s.control_enabled(control, cx));
                        s.reveal_history_control(control);
                        window.focus(&s.handle(control, cx));
                        s.activate_control(control, window, cx);
                    })
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let row = cx.debug_bounds(row_selector).unwrap();
                let list = cx.debug_bounds(list).unwrap();
                let viewport = shell.read_with(cx, |s, _| s.screen_scroll.bounds());
                assert_eq!(row.size.height, px(64.));
                assert!(
                    row.top() >= list.top() && row.bottom() <= list.bottom(),
                    "{width}: {control:?} must be revealed in its list"
                );
                assert!(
                    list.top() >= viewport.top() && list.bottom() <= viewport.bottom(),
                    "{width}: {control:?} list must be revealed in the screen"
                );
                shell.read_with(cx, |s, _| match control {
                    Control::HistoryBaseline(i) => assert_eq!(s.history_index, i),
                    Control::HistoryRun(i) => assert_eq!(s.run_history_index, i),
                    _ => unreachable!(),
                });
            }
        }
        owned.join_after_ui();
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn reference_environment_loads_with_pair_without_history(cx: &mut gpui::TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let mut evidence = store.project_evidence(project).unwrap();
        let reference = &evidence.baseline.as_ref().unwrap().references[0];
        let before_environment = store.load_run(reference.run).unwrap().run.environment.name;
        let key = reference.key.clone();
        let analysis = evidence.comparison.as_ref().map(|a| a.revision.id);
        // Today's environment must never supply the historical reference label.
        evidence.latest.as_mut().unwrap().run.environment.name = "Today's staging".into();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            let startup = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
            s.apply_update(startup, cx);
            s.project = Some(record);
            s.evidence = evidence;
            s.selected_key = Some(key.clone());
            s.focus.screen = Screen::Results;
            s.focus.project_saved = true;
            s.history = None;
            s.jobs
                .submit_viewer(Request {
                    id: 900,
                    work: Work::Extension(ViewerWork::Viewer {
                        project,
                        tile: 0,
                        key: Some(key.clone()),
                        analysis,
                    }),
                })
                .unwrap();
            let reply = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
            assert_eq!(reply.id, 900);
            let Reply::Extension(frame) = reply.reply else {
                panic!("selected pair must load with recorded reference context");
            };
            assert_eq!(
                frame.before_environment.as_deref(),
                Some(before_environment.as_str())
            );
            s.viewer = Some(*frame);
            assert!(s.history.is_none());
            assert_eq!(
                s.viewer_reference_environment(),
                Some(before_environment.as_str())
            );
            assert!(s.pane_caption(0).1.starts_with(&before_environment));
            assert!(!s.pane_caption(0).1.contains("Today's staging"));

            // Historical analyses also name their source without a History
            // visit, even when the latest capture is a different run.
            s.evidence.latest.as_mut().unwrap().run.id = regressee_core::domain::RunId::new();
            assert!(s.displayed_run().is_none());
            assert_eq!(
                s.viewer_reference_environment(),
                Some(before_environment.as_str())
            );

            // A reply for another target/project cannot lend its provenance.
            s.selected_key.as_mut().unwrap().page = regressee_core::domain::PageId::new();
            assert!(s.viewer_reference_environment().is_none());
            s.selected_key = Some(key);
            s.viewer
                .as_mut()
                .unwrap()
                .identity
                .as_mut()
                .unwrap()
                .project = regressee_core::domain::ProjectId::new();
            assert!(s.viewer_reference_environment().is_none());
            assert!(s.pane_caption(0).1.contains("environment not loaded"));
        });
        owned.join_after_ui();
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn capture_only_distinguishes_incomplete_ready_and_saved_baselines(
        cx: &mut gpui::TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let mut record = store.load_project(project).unwrap();
        let baseline = store.project_evidence(project).unwrap().baseline.unwrap();
        let mut evidence = ProjectEvidence {
            latest: Some(store.load_run(baseline.references[0].run).unwrap()),
            ..Default::default()
        };
        record.project.active_baseline = None;
        let key = evidence.latest.as_ref().unwrap().run.targets[0].key.clone();
        let mut failed = evidence.latest.as_ref().unwrap().run.targets[0].clone();
        failed.key.page = regressee_core::domain::PageId::new();
        failed.requested = url::Url::parse("http://127.0.0.1:43210/failed").unwrap();
        failed.outcome = TargetOutcome::Failed {
            code: regressee_core::domain::CaptureError::Timeout,
        };
        let run = evidence.latest.as_mut().unwrap();
        run.run.targets.push(failed);
        run.run.state = JobState::CompletedWithErrors;
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            let update = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
            s.apply_update(update, cx);
            s.project = Some(record);
            s.focus.project_saved = true;
            s.pending = Some(1000);
            s.apply_update(
                Update {
                    id: 1000,
                    reply: Reply::Evidence(Box::new(evidence.clone())),
                },
                cx,
            );
            s.focus.screen = Screen::Results;
            assert!(s.focus.candidate && !s.focus.baseline);
            assert_eq!(s.review_title(), "Inspect capture");
            assert_eq!(
                s.baseline_rail_status(),
                "Ready to save baseline\nSome screenshots will be skipped"
            );
            assert!(
                s.capture_next_step()
                    .unwrap()
                    .contains("skip 1 missing screenshot")
            );
            assert!(s.comparison_context().contains("no comparison performed"));
            assert_eq!(
                s.captured_target_label(&key),
                "Captured · not yet a baseline"
            );
            assert!(s.control_enabled(Control::SaveBaseline, cx));
            assert!(s.control_enabled(Control::ReviewCaptureFailures, cx));
            assert!(!s.control_enabled(Control::CheckChanges, cx));
            assert!(!s.control_enabled(Control::CaptureBaseline, cx));
            let coverage =
                Coverage::from_run(s.evidence.latest.as_ref().unwrap(), &s.review_index());
            assert_eq!(
                (coverage.total, coverage.unclassified, coverage.missing),
                (2, 1, 1)
            );
            assert_eq!((coverage.changed, coverage.unchanged), (0, 0));
            // Every previously selected comparison mode still shows the original
            // capture only, never a blank before pane or a retained heatmap.
            for mode in 0..4 {
                s.viewer_mode = mode;
                assert_eq!(s.viewer_pane_indices(), &[1]);
            }
            s.working_width = 1040.;
            s.narrow_workspace = false;
            s.viewer = Some(regressee_gui::jobs::ViewerFrame {
                tile: 0,
                panes: vec![vec![], vec![], vec![]],
                width: 1440,
                height: 900,
                request: None,
                analysis: None,
                identity: None,
                before_environment: None,
                before_capture: None,
            });
            assert!((s.fitted_viewer_zoom() - 756. / 1440.).abs() < 0.0001);
            s.clear_viewer();

            // A complete capture offers an explicit save, not automatic activation.
            evidence.latest.as_mut().unwrap().run.targets.pop();
            evidence.latest.as_mut().unwrap().run.state = JobState::Completed;
            s.focus.screen = Screen::Projects;
            s.pending = Some(1001);
            s.apply_update(
                Update {
                    id: 1001,
                    reply: Reply::Evidence(Box::new(evidence.clone())),
                },
                cx,
            );
            s.focus.screen = Screen::Results;
            assert!(s.focus.candidate && !s.focus.baseline);
            assert_eq!(s.review_title(), "Inspect capture");
            assert_eq!(s.baseline_rail_status(), "Ready to save baseline");
            assert!(
                s.capture_next_step()
                    .unwrap()
                    .contains("Choose Save baseline")
            );
            assert!(s.control_enabled(Control::SaveBaseline, cx));
            assert!(!s.control_enabled(Control::CheckChanges, cx));
            assert!(
                s.project
                    .as_ref()
                    .unwrap()
                    .project
                    .active_baseline
                    .is_none()
            );

            evidence.baseline = Some(baseline.clone());
            s.project.as_mut().unwrap().project.active_baseline = Some(baseline.id);
            s.focus.screen = Screen::Projects;
            s.pending = Some(1002);
            s.apply_update(
                Update {
                    id: 1002,
                    reply: Reply::Evidence(Box::new(evidence)),
                },
                cx,
            );
            s.focus.screen = Screen::Results;
            assert!(!s.focus.candidate && s.focus.baseline);
            assert_eq!(s.review_title(), "Baseline screenshots");
            assert_eq!(s.captured_target_label(&key), "Saved baseline screenshot");
            assert!(s.capture_next_step().unwrap().contains("Run check"));
            assert!(s.comparison_context().contains("Saved baseline"));
            assert!(!s.control_enabled(Control::SaveBaseline, cx));
            assert!(s.control_enabled(Control::CheckChanges, cx));
        });
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let next = cx.debug_bounds("capture-next-step").unwrap();
            let queue = cx.debug_bounds("review-queue").unwrap();
            assert!(next.bottom() <= queue.top());
            assert!(queue.size.height >= px(PAGE_ROW_HEIGHT));
        }
        owned.join_after_ui();
    }

    #[cfg(feature = "ui-tests")]
    fn large_evidence() -> (ProjectRecord, ProjectEvidence) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let mut e = store.project_evidence(project).unwrap();
        let target = e.latest.as_ref().unwrap().run.targets[0].clone();
        let capture = e
            .latest
            .as_ref()
            .unwrap()
            .all_captures()
            .next()
            .unwrap()
            .clone();
        let result = e.comparison.as_ref().unwrap().revision.results[0].clone();
        let report = e
            .comparison
            .as_ref()
            .unwrap()
            .perceptual
            .as_ref()
            .unwrap()
            .reports[0]
            .1
            .clone();
        let reference = e.baseline.as_ref().unwrap().references[0].clone();
        let run = e.latest.as_mut().unwrap();
        run.run.targets.clear();
        run.capture = None;
        run.captures.clear();
        let comparison = e.comparison.as_mut().unwrap();
        comparison.revision.results.clear();
        comparison.perceptual.as_mut().unwrap().reports.clear();
        e.baseline.as_mut().unwrap().references.clear();
        for i in 0..3000 {
            let mut t = target.clone();
            t.key.page = regressee_core::domain::PageId::new();
            t.requested = url::Url::parse(&format!("https://example.test/page-{i}")).unwrap();
            if i % 5 == 2 {
                t.outcome = TargetOutcome::Failed {
                    code: regressee_core::domain::CaptureError::NeedsLogin,
                };
            }
            let mut c = capture.clone();
            c.metadata.key = t.key.clone();
            run.captures.push(c);
            let mut r = result.clone();
            r.key = t.key.clone();
            r.classification = match i % 5 {
                0 => Classification::Changed,
                1 => Classification::NoSignificantChange,
                _ => Classification::NotComparable,
            };
            if i % 5 != 4 {
                comparison.revision.results.push(r);
            }
            let mut p = report.clone();
            if matches!(i % 5, 2 | 3) {
                // These cases model genuinely incompatible comparisons, not
                // the old blanket veto of an otherwise valid pixel report.
                p.classification = regressee_app::analysis_engine::Classification::NotComparable;
                p.gates
                    .push(regressee_app::analysis_engine::GateReason::MaterialProfileMismatch);
            }
            p.request.before.target.page = t.key.page.to_string();
            p.request.after.target.page = t.key.page.to_string();
            comparison
                .perceptual
                .as_mut()
                .unwrap()
                .reports
                .push((t.key.clone(), p));
            let mut reference = reference.clone();
            reference.key = t.key.clone();
            e.baseline.as_mut().unwrap().references.push(reference);
            run.run.targets.push(t);
        }
        (record, e)
    }

    #[cfg(feature = "ui-tests")]
    #[test]
    fn indexed_3000_target_coverage_is_exact_and_rejects_stale_revisions() {
        let (record, mut e) = large_evidence();
        let index = ReviewIndex::build(&e, record.project.active_baseline);
        assert_eq!(index.classifications.len(), 2400);
        assert_eq!(index.eligible.len(), 1200);
        assert_eq!(
            Coverage::from_run(e.latest.as_ref().unwrap(), &index),
            Coverage {
                total: 3000,
                changed: 600,
                unchanged: 600,
                missing: 1200,
                unclassified: 600
            }
        );
        let mut missing = e.clone();
        let key = missing.latest.as_ref().unwrap().run.targets[1].key.clone();
        missing
            .latest
            .as_mut()
            .unwrap()
            .captures
            .retain(|c| c.metadata.key != key);
        let missing_index = ReviewIndex::build(&missing, record.project.active_baseline);
        assert_eq!(
            Coverage::from_run(missing.latest.as_ref().unwrap(), &missing_index),
            Coverage {
                total: 3000,
                changed: 600,
                unchanged: 599,
                missing: 1201,
                unclassified: 600
            }
        );
        assert!(!missing_index.eligible.contains(&key));
        e.comparison.as_mut().unwrap().revision.run = regressee_core::domain::RunId::new();
        let stale = ReviewIndex::build(&e, record.project.active_baseline);
        assert!(stale.classifications.is_empty() && stale.eligible.is_empty());
        assert_eq!(
            Coverage::from_run(e.latest.as_ref().unwrap(), &stale),
            Coverage {
                total: 3000,
                changed: 0,
                unchanged: 0,
                missing: 600,
                unclassified: 2400
            }
        );
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn review_3000_target_cpu_element_construction_benchmark(cx: &mut gpui::TestAppContext) {
        let (record, evidence) = large_evidence();
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        let timings = shell.update(cx, |s, cx| {
            let update = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
            s.apply_update(update, cx);
            s.project = Some(record);
            s.evidence = evidence;
            s.evidence_dirty = false;
            s.focus.screen = Screen::Results;
            s.focus.project_saved = true;
            s.focus.baseline = true;
            s.selected_key = s
                .evidence
                .latest
                .as_ref()
                .unwrap()
                .run
                .targets
                .first()
                .map(|t| t.key.clone());
            s.approval_selection = s.review_index().eligible;
            assert!(s.approval_selection_eligible());
            s.configure_review_geometry(1280.);
            let mut timings = Vec::new();
            for _ in 0..12 {
                let start = std::time::Instant::now();
                let _workspace = s.results(cx);
                let _decision_bar = s.approval_bar(cx);
                timings.push(start.elapsed().as_secs_f64() * 1000.);
            }
            timings.sort_by(f64::total_cmp);
            timings
        });
        let report = format!(
            "3000 synthetic targets; 1200 eligible selected; 12 samples\nCPU Rust element construction only (indexes, coverage, sorting, controls). No layout, paint or GPU readback included.\nmin_ms={:.3}\nmedian_ms={:.3}\nmax_ms={:.3}\n",
            timings[0], timings[6], timings[11]
        );
        std::fs::write(
            std::env::temp_dir().join("regressee-review-cpu-construction.txt"),
            report,
        )
        .unwrap();
        owned.join_after_ui();
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn review_first_viewport_contains_complete_rows_and_bounded_images(
        cx: &mut gpui::TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, valid, _) = crate::approval_fixture::fixture(dir.path());
        let key = store
            .project_evidence(project)
            .unwrap()
            .latest
            .unwrap()
            .run
            .targets[0]
            .key
            .clone();
        let preview = store
            .approval_preview(valid, std::slice::from_ref(&key))
            .unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            let update = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
            s.apply_update(update, cx);
            s.focus.screen = Screen::Results;
            s.queue(Work::Load(project), cx);
            for _ in 0..12 {
                if s.pending.is_some() || s.viewer_pending.is_some() || s.storage_pending.is_some()
                {
                    let update = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
                    s.apply_update(update, cx);
                }
            }
            assert!(s.viewer.is_some());
            let run = s.evidence.latest.as_mut().unwrap();
            let target = run.run.targets[0].clone();
            for i in 1..8 {
                let mut t = target.clone();
                t.key.page = regressee_core::domain::PageId::new();
                t.requested = url::Url::parse(&format!("http://127.0.0.1:43210/page-{i}")).unwrap();
                run.run.targets.push(t);
            }
            s.notice = None;
        });
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let queue = cx.debug_bounds("review-queue").unwrap();
            let viewer = cx.debug_bounds("review-viewer").unwrap();
            let row = cx.debug_bounds("page-row-target-0").unwrap();
            let image = cx.debug_bounds("phase5-after").unwrap();
            let bar = cx.debug_bounds("approval-next-action").unwrap();
            assert!(
                queue.right() < viewer.left(),
                "{width}: queue must stay beside evidence"
            );
            assert_eq!(row.size.height, px(84.));
            assert!(
                row.top() >= queue.top() && row.bottom() <= queue.bottom(),
                "{width}: one complete result must open in view: row {row:?}, queue {queue:?}"
            );
            assert!(
                image.size.height >= px(64.),
                "{width}: image region must remain usable: {:?}",
                image.size
            );
            assert!(image.bottom() <= viewer.bottom() && viewer.bottom() <= bar.top());
            assert!(bar.size.height <= px(96.));
        }
        shell.update(cx, |s, cx| {
            s.approval_selection.insert(key);
            s.approval = Some(preview);
            s.focus.approval_dialog = true;
            s.presentation_focus = Some(Control::Phase5(25));
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let decision = cx.debug_bounds("approval-decision").unwrap();
        let impact = cx.debug_bounds("approval-impact-summary").unwrap();
        let map = cx.debug_bounds("approval-reference-list").unwrap();
        assert!(cx.debug_bounds("approval-exact-provenance").is_none());
        assert!(decision.top() <= impact.top() && map.bottom() <= decision.bottom());
        assert!(map.size.height > px(0.));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert!(s.handle(Control::Phase5(25), cx).is_focused(window));
                let requests = s.next_request;
                s.activate_control(Control::CheckChanges, window, cx);
                s.activate_control(Control::Phase5(23), window, cx);
                assert_eq!(requests, s.next_request);
                let handles = [
                    s.handle(Control::Guided(47), cx),
                    s.handle(Control::Guided(48), cx),
                    s.handle(Control::Guided(49), cx),
                    s.handle(Control::Guided(52), cx),
                    s.handle(Control::Phase5(25), cx),
                ];
                for _ in 0..4 {
                    window.focus_next();
                    assert!(handles.contains(&window.focused(cx).unwrap()));
                }
                s.guided(47, cx);
                assert!(!s.focus.approval_dialog);
                assert!(s.approval.is_some());
                s.mask_dirty = true;
                assert!(!s.approval_matches(s.approval.as_ref().unwrap()));
                s.invalidate_approval();
                assert!(s.approval.is_none());
            })
        });
        owned.join_after_ui();
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn compared_reference_context_never_substitutes_the_current_active_baseline(
        cx: &mut gpui::TestAppContext,
    ) {
        let (mut record, evidence) = large_evidence();
        let compared = evidence.comparison.as_ref().unwrap().revision.baseline;
        let environment = evidence
            .latest
            .as_ref()
            .unwrap()
            .run
            .environment
            .name
            .clone();
        let active = regressee_core::domain::BaselineId::new();
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        assert_ne!(compared, active);
        assert!(
            ReviewIndex::build(&evidence, Some(active))
                .eligible
                .is_empty()
        );
        assert_eq!(record.project.active_baseline, Some(compared));
        record.project.active_baseline = Some(active);
        shell.update(cx, |s, _| {
            s.project = Some(record);
            s.evidence = evidence;
            s.selected_key = Some(
                s.evidence.latest.as_ref().unwrap().run.targets[0]
                    .key
                    .clone(),
            );
            let context = s.comparison_context();
            assert!(context.contains("Historical baseline"));
            assert!(!context.contains(&short_id(compared.to_string())));
            assert!(!context.contains(&short_id(active.to_string())));
            assert!(context.contains(&environment));
            assert_eq!(s.pane_caption(0).0, "Before · baseline");
            assert!(
                s.approval_unavailable_reason(s.selected_key.as_ref())
                    .unwrap()
                    .contains("earlier baseline")
            );
        });
        owned.join_after_ui();
    }
    #[test]
    fn coverage_never_calls_missing_or_unclassified_captures_clean() {
        let mut c = Coverage::default();
        c.count(false, Some(Classification::NoSignificantChange));
        c.count(true, None);
        c.count(true, Some(Classification::NotComparable));
        c.count(true, Some(Classification::CouldNotCapture));
        c.count(true, Some(Classification::Changed));
        c.count(true, Some(Classification::NoSignificantChange));
        assert_eq!(
            c,
            Coverage {
                total: 6,
                changed: 1,
                unchanged: 1,
                missing: 3,
                unclassified: 1
            }
        );
    }

    #[test]
    fn responsive_width_uses_the_current_queue_and_pane_count() {
        assert_eq!(viewer_pane_width(580., 200., 2), 178.);
        assert_eq!(viewer_pane_width(580., 0., 2), 284.);
        assert_eq!(viewer_pane_width(1040., 272., 1), 756.);
        assert_eq!(viewer_pane_width(580., 0., 1), 580.);
        assert_eq!(viewer_pane_width(1., 272., 2), 1.);
        assert_eq!(count_label(1, "page", "pages"), "1 page");
        assert_eq!(
            count_label(2, "screen size", "screen sizes"),
            "2 screen sizes"
        );
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn discovery_owns_its_scope_and_review_preserves_provenance(cx: &mut gpui::TestAppContext) {
        use regressee_core::domain::{ReviewDecision, ReviewState};
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let evidence = store.project_evidence(project).unwrap();
        let history = store.history(project, 0).unwrap();
        let key = evidence.latest.as_ref().unwrap().run.targets[0].key.clone();
        let reference = evidence
            .baseline
            .as_ref()
            .unwrap()
            .references
            .iter()
            .find(|r| r.key == key)
            .unwrap();
        let original = store.load_run(reference.run).unwrap();
        let original_profile = original.evidence(&key).unwrap().metadata.profile.clone();
        let original_environment = original.run.environment.name.clone();
        let mut inventory = evidence.latest.as_ref().unwrap().run.inventory.clone();
        let mut second = inventory.pages[0].clone();
        second.id = regressee_core::domain::PageId::new();
        second.canonical = url::Url::parse("http://127.0.0.1:43210/new-page").unwrap();
        second.included = true;
        inventory.pages.push(second);
        let preview = DiscoveryPreview {
            project_revision: record.revision,
            settings: record.project.clone(),
            inventory,
            aliases: vec![],
            notes: vec![],
            visited: 2,
        };
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            let startup = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
            s.apply_update(startup, cx);
            s.project = Some(record);
            s.evidence = evidence;
            s.evidence_dirty = false;
            s.history = Some(history);
            s.focus.screen = Screen::Results;
            s.focus.project_saved = true;
            s.selected_key = Some(key.clone());
            s.preview = Some(preview);
            s.focus.preview = true;
            s.focus.review_surface = ReviewSurface::Pages;
            s.configure_review_geometry(820.);
            assert!(s.comparison_context().contains("2 pages included"));
            assert!(!s.comparison_context().contains("Compared baseline"));
            assert!(!s.discovery_image_mounted());
            assert_eq!(s.review_queue_width(), 772.);
            assert_eq!(s.viewer_pane_indices(), &[1]);
            assert!(!s.control_mounted(Control::Guided(16)));
        });
        cx.simulate_resize(size(px(820.), px(600.)));
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let queue = cx.debug_bounds("review-queue").unwrap();
        assert!(queue.size.width >= px(580.));
        assert!(cx.debug_bounds("action-Guided(16)").is_none());
        // The inventory viewport, not merely its containing queue, reserves two rows.
        let list = cx.debug_bounds("inventory-preview").unwrap();
        assert!(list.size.height >= px(PAGE_ROW_HEIGHT * 2.));
        shell.update(cx, |s, _| {
            s.preview = None;
            s.focus.preview = false;
            s.focus.review_surface = ReviewSurface::Changes;
            s.evidence.latest.as_mut().unwrap().run.profiles[0].width = 999;
            s.evidence.latest.as_mut().unwrap().run.environment.name = "Today's staging".into();
            assert_eq!(s.pane_caption(0).0, "Before · baseline");
            assert!(s.pane_caption(0).1.starts_with(&original_environment));
            assert_eq!(s.pane_caption(0).2, compact_profile(&original_profile));
            s.mask_dirty = true;
            assert_eq!(
                s.approval_unavailable_reason(Some(&key)),
                Some("Save ignored areas first.")
            );
            s.mask_dirty = false;
            let classification =
                s.evidence.comparison.as_ref().unwrap().revision.results[0].classification;
            s.evidence.comparison.as_mut().unwrap().revision.results[0].classification =
                Classification::CouldNotCapture;
            assert_eq!(
                s.approval_unavailable_reason(Some(&key)),
                Some(
                    "This screenshot was not captured, so it cannot replace a baseline screenshot."
                )
            );
            assert_eq!(ReviewProgress::from_evidence(&s.evidence).gaps, 1);
            s.evidence.comparison.as_mut().unwrap().revision.results[0].classification =
                Classification::NotComparable;
            s.evidence
                .comparison
                .as_mut()
                .unwrap()
                .perceptual
                .as_mut()
                .unwrap()
                .reports[0]
                .1
                .gates
                .push(regressee_app::analysis_engine::GateReason::MaterialProfileMismatch);
            assert_eq!(
                s.approval_unavailable_reason(Some(&key)),
                Some(
                    "These screenshots could not be compared, so they cannot update the baseline."
                )
            );
            s.evidence.comparison.as_mut().unwrap().revision.results[0].classification =
                classification;
            s.evidence
                .comparison
                .as_mut()
                .unwrap()
                .perceptual
                .as_mut()
                .unwrap()
                .reports[0]
                .1
                .gates
                .clear();
            let analysis = s.evidence.comparison.as_ref().unwrap().revision.id;
            s.evidence.reviews.push(ReviewDecision {
                analysis: regressee_core::domain::AnalysisId::new(),
                key: key.clone(),
                state: ReviewState::Intentional,
                note: String::new(),
                reviewed_at: 9,
            });
            assert_eq!(s.ordinary_review_status(), "Not reviewed for this analysis");
            assert_eq!(ReviewProgress::from_evidence(&s.evidence).remaining, 1);
            s.evidence.reviews.push(ReviewDecision {
                analysis,
                key: key.clone(),
                state: ReviewState::Intentional,
                note: String::new(),
                reviewed_at: 10,
            });
            assert_eq!(ReviewProgress::from_evidence(&s.evidence).reviewed, 1);
            assert_eq!(ReviewProgress::from_evidence(&s.evidence).remaining, 0);
            s.evidence.reviews.push(ReviewDecision {
                analysis,
                key,
                state: ReviewState::NeedsFix,
                note: String::new(),
                reviewed_at: 11,
            });
            assert_eq!(ReviewProgress::from_evidence(&s.evidence).needs_fix, 1);
            assert_eq!(
                s.review_progress_summary(),
                "1 reviewed · No changes to review · 1 marked Needs fix"
            );
            assert!(s.toggle_workspace_presentation(Control::Guided(50)));
            assert_eq!(s.review_queue_width(), 0.);
            assert!(!s.workspace_control_mounted(Control::CrawlField(9)));
            assert!(s.toggle_workspace_presentation(Control::Guided(50)));
            assert_eq!(s.review_queue_width(), 200.);
        });
        owned.join_after_ui();
    }
}
