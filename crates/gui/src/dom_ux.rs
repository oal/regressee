// SPDX-License-Identifier: MPL-2.0
//! Native kind-aware evidence state. Source bytes are requested only via Jobs IPC.
use super::*;
use regressee_core::{
    dom_snapshot::*,
    domain::{CheckKind, DomTargetOutcome},
};
use regressee_gui::{
    jobs::{DomPresentation, DomViewerFrame, DomViewerIdentity},
    ui::dom_diff::{DomDiff, DomDiffLoadToken},
};

pub(super) struct DomWorkspace {
    pub kind: CheckKind,
    pub source: Entity<DomDiff>,
    pub presentation: DomPresentation,
    pub scope: Option<(
        regressee_core::domain::ProjectId,
        regressee_core::domain::RunId,
    )>,
    pub identity: Option<DomViewerIdentity>,
    pub pending: Option<(u64, DomDiffLoadToken, DomViewerIdentity)>,
    pub frame: Option<DomViewerFrame>,
    pub failed: bool,
    pub read_failures: Option<regressee_gui::jobs::EvidenceReadFailures>,
    pub active_note: Option<DomReviewDecision>,
    pub selected: std::collections::BTreeMap<TargetKey, DomApprovalSelection>,
    pub approval: Option<Box<regressee_storage::KindApprovalPreview>>,
    pub notes: std::collections::BTreeMap<
        (regressee_core::domain::DomAnalysisId, TargetKey),
        DomReviewDecision,
    >,
}
impl DomWorkspace {
    pub fn new(cx: &mut Context<Shell>) -> Self {
        Self {
            kind: CheckKind::Screenshot,
            source: cx.new(DomDiff::new),
            presentation: DomPresentation::Diff,
            scope: None,
            identity: None,
            pending: None,
            frame: None,
            failed: false,
            read_failures: None,
            active_note: None,
            selected: Default::default(),
            approval: None,
            notes: Default::default(),
        }
    }
}
pub(super) fn dom_outcome_label(outcome: &DomComparisonOutcome) -> &'static str {
    match outcome {
        DomComparisonOutcome::NotRequested => "Not requested",
        DomComparisonOutcome::Changed => "Changed",
        DomComparisonOutcome::Unchanged => "Identical",
        DomComparisonOutcome::CouldNotCapture { code } => code.label(),
        DomComparisonOutcome::NotComparable { reason } => match reason {
            DomNotComparableReason::BaselineOmission => "Omitted from the saved baseline",
            DomNotComparableReason::MissingBefore => "Before DOM snapshot is missing",
            DomNotComparableReason::MissingAfter => "After DOM snapshot is missing",
            DomNotComparableReason::PolicyMismatch => "Snapshot policies differ",
            DomNotComparableReason::MaterialProfileMismatch => "Capture settings differ",
            DomNotComparableReason::RenderingEnvironmentMismatch => {
                "Browser or rendering environments differ"
            }
            DomNotComparableReason::Unstable => "DOM kept changing during capture",
            DomNotComparableReason::InvalidEvidence => "DOM evidence could not be verified",
            DomNotComparableReason::Skipped => "DOM capture skipped",
            DomNotComparableReason::Cancelled => "DOM capture stopped",
        },
    }
}
impl Shell {
    pub(super) fn kind_read_error(&self, kind: CheckKind) -> Option<&str> {
        let failures = self.dom.read_failures.as_ref()?;
        if self.project.as_ref().map(|p| p.project.id) != Some(failures.project)
            || self.evidence.latest.as_ref().map(|r| r.run.id) != Some(failures.run)
        {
            return None;
        }
        failures.by_kind.get(&kind).map(|error| error.as_ref())
    }
    pub(super) fn recorded_mode(&self) -> CaptureMode {
        self.preview
            .as_ref()
            .map(|p| p.settings.capture_plan.mode)
            .or_else(|| {
                self.evidence
                    .latest
                    .as_ref()
                    .map(|r| r.run.capture_plan.mode)
            })
            .or_else(|| self.evidence.baseline.as_ref().map(|b| b.capture_plan.mode))
            .unwrap_or_default()
    }
    pub(super) fn dom_active(&self) -> bool {
        let mode = self.recorded_mode();
        mode.dom_snapshots()
            && (mode == CaptureMode::DomSnapshotsOnly || self.dom.kind == CheckKind::DomSnapshot)
            && !self.page_list_workspace()
    }
    pub(super) fn dom_input_allowed(&self) -> bool {
        self.dom_active()
            && self.focus.screen == Screen::Results
            && !self.agent_locked()
            && !self.focus.busy
            && !self.focus.dialog
            && !self.focus.approval_dialog
            && !self.native_prompt
            && !self.viewer_inspector_open()
            && !self.options_panel_open()
            && self.task.is_none()
    }
    pub(super) fn evidence_kind_choices(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(key) = &self.selected_key else {
            return div().into_any_element();
        };
        if self.recorded_mode() != CaptureMode::ScreenshotsAndDomSnapshots
            || self.page_list_workspace()
        {
            return div().into_any_element();
        }
        div()
            .debug_selector(|| "evidence-kind-tabs".into())
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(self.button(
                Control::EvidenceKind(CheckKind::Screenshot),
                "Screenshots",
                ActionStyle::Choice {
                    selected: !self.dom_active(),
                },
                cx,
            ))
            .child(self.button(
                Control::EvidenceKind(CheckKind::DomSnapshot),
                "DOM snapshot",
                ActionStyle::Choice {
                    selected: self.dom_active(),
                },
                cx,
            ))
            .child(
                div()
                    .flex_1()
                    .min_w(px(160.))
                    .text_size(px(HELP_TEXT_SIZE))
                    .line_height(px(HELP_LINE_HEIGHT))
                    .text_color(rgb(palette().muted))
                    .child(self.result_kind_label(key)),
            )
            .into_any_element()
    }
    pub(super) fn dom_pane_caption(&self, pane: usize) -> (String, String, String) {
        let run = self.evidence.latest.as_ref();
        let key = self.selected_key.as_ref();
        let capture = if pane == 0 {
            self.dom
                .frame
                .as_ref()
                .and_then(|f| f.artifacts.before.as_ref())
                .map(|o| &o.metadata)
                .or_else(|| {
                    self.evidence
                        .dom_before
                        .as_ref()
                        .filter(|c| Some(&c.metadata.key) == key)
                        .map(|c| &c.metadata)
                })
        } else {
            run.and_then(|r| key.and_then(|key| r.dom_evidence(key)))
                .map(|c| &c.metadata)
        };
        let environment = if pane == 0 {
            capture
                .and_then(|capture| {
                    self.history
                        .as_ref()
                        .and_then(|history| history.runs.iter().find(|r| r.run.id == capture.run))
                })
                .map(|r| r.run.environment.name.as_str())
        } else {
            run.map(|r| r.run.environment.name.as_str())
        }
        .unwrap_or("environment not loaded");
        let profile = capture
            .map(|c| crate::workflow::compact_profile(&c.profile))
            .or_else(|| {
                run.and_then(|r| {
                    r.run
                        .profiles
                        .iter()
                        .find(|p| Some(p.id) == key.map(|key| key.profile))
                })
                .map(|p| format!("{} · {} × {} pixels", p.name, p.width, p.height))
            })
            .unwrap_or_else(|| "Screen size not loaded".into());
        (
            if pane == 0 {
                "Before · saved baseline"
            } else if self.has_kind_comparison() {
                "After · check"
            } else {
                "Captured DOM snapshot"
            }
            .into(),
            format!(
                "{environment}\n{}",
                capture
                    .map(|c| c.final_url.as_str())
                    .unwrap_or("Source URL not loaded")
            ),
            profile,
        )
    }
    pub(super) fn kind_coverage_summary(&self) -> AnyElement {
        let Some(run) = &self.evidence.latest else {
            return div().into_any_element();
        };
        let mut parts = Vec::new();
        for kind in run.run.capture_plan.kinds() {
            if self.kind_read_error(kind).is_some() {
                parts.push(div().text_color(rgb(palette().warning)).child(format!(
                    "{}: evidence could not be read · {} not verified",
                    if kind == CheckKind::Screenshot {
                        "Screenshots"
                    } else {
                        "DOM snapshots"
                    },
                    run.run.targets.len()
                )));
                continue;
            }
            let mut counts = [0usize; 5]; // changed, unchanged, missing, pending, captured
            match kind {
                CheckKind::Screenshot => {
                    let classes = self
                        .evidence
                        .comparison
                        .as_ref()
                        .filter(|a| a.revision.run == run.run.id)
                        .map(|a| a.review_classifications())
                        .unwrap_or_default();
                    for target in &run.run.targets {
                        counts[match classes.get(&target.key) {
                            Some(Classification::Changed) => 0,
                            Some(Classification::NoSignificantChange) => 1,
                            Some(
                                Classification::CouldNotCapture | Classification::NotComparable,
                            ) => 2,
                            None => match target.outcome {
                                TargetOutcome::Captured { .. } => 4,
                                TargetOutcome::Pending => 3,
                                _ => 2,
                            },
                        }] += 1;
                    }
                }
                CheckKind::DomSnapshot => {
                    let outcomes: std::collections::BTreeMap<_, _> = self
                        .evidence
                        .dom_comparison
                        .iter()
                        .filter(|a| a.revision.run == run.run.id)
                        .flat_map(|a| a.revision.results.iter())
                        .map(|r| (&r.key, &r.outcome))
                        .collect();
                    for target in &run.run.targets {
                        counts[match outcomes.get(&target.key) {
                            Some(DomComparisonOutcome::Changed) => 0,
                            Some(DomComparisonOutcome::Unchanged) => 1,
                            Some(_) => 2,
                            None => match target.dom_outcome {
                                DomTargetOutcome::Captured { .. } => 4,
                                DomTargetOutcome::Pending => 3,
                                _ => 2,
                            },
                        }] += 1;
                    }
                }
            }
            let name = if kind == CheckKind::Screenshot {
                "Screenshots"
            } else {
                "DOM snapshots"
            };
            let details = if self.has_kind_comparison() {
                format!(
                    "{} changed · {} unchanged · {} not compared",
                    counts[0], counts[1], counts[2]
                )
            } else {
                format!(
                    "{} / {} captured · {} not captured",
                    counts[4],
                    run.run.targets.len(),
                    counts[2]
                )
            };
            let tail = if counts[3] > 0 {
                format!(" · {} pending", counts[3])
            } else if self.has_kind_comparison() && counts[4] > 0 {
                format!(" · {} awaiting analysis", counts[4])
            } else {
                String::new()
            };
            parts.push(
                div()
                    .text_color(rgb(if counts[2] > 0 {
                        palette().warning
                    } else {
                        palette().ink
                    }))
                    .child(format!("{name}: {details}{tail}")),
            );
        }
        div()
            .debug_selector(|| "real-coverage-summary".into())
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .gap_3()
            .py_1()
            .border_b_1()
            .border_color(rgb(palette().line))
            .text_size(px(HELP_TEXT_SIZE))
            .line_height(px(HELP_LINE_HEIGHT))
            .children(parts)
            .into_any_element()
    }
    pub(super) fn has_kind_comparison(&self) -> bool {
        self.evidence.comparison.is_some() || self.evidence.dom_comparison.is_some()
    }
    pub(super) fn dom_result(&self) -> Option<&DomAnalysisResult> {
        let comparison = self.evidence.dom_comparison.as_ref()?;
        let run = self.evidence.latest.as_ref()?;
        (comparison.revision.run == run.run.id).then_some(())?;
        comparison
            .revision
            .results
            .iter()
            .find(|r| Some(&r.key) == self.selected_key.as_ref())
    }
    pub(super) fn kind_status(&self, kind: CheckKind, key: &TargetKey) -> String {
        if let Some(error) = self.kind_read_error(kind) {
            return format!(
                "Unreadable evidence: {}",
                error.chars().take(160).collect::<String>()
            );
        }
        let Some(run) = &self.evidence.latest else {
            return "Not loaded".into();
        };
        let Some(target) = run.run.targets.iter().find(|t| &t.key == key) else {
            return "Target not loaded".into();
        };
        match kind {
            CheckKind::Screenshot => {
                if !run.run.capture_plan.mode.screenshots() {
                    return "Not requested".into();
                }
                if let Some(classification) = self
                    .evidence
                    .comparison
                    .as_ref()
                    .filter(|a| a.revision.run == run.run.id)
                    .and_then(|a| a.review_classification(key))
                {
                    return classification.label().into();
                }
                match &target.outcome {
                    TargetOutcome::NotRequested => "Omitted from the saved baseline".into(),
                    TargetOutcome::Pending => "Pending".into(),
                    TargetOutcome::Captured { .. } => "Captured".into(),
                    TargetOutcome::Failed { code } => code.label().into(),
                    TargetOutcome::Skipped { reason } => format!("Skipped: {reason}"),
                    TargetOutcome::Cancelled => "Stopped".into(),
                }
            }
            CheckKind::DomSnapshot => {
                if !run.run.capture_plan.mode.dom_snapshots() {
                    return "Not requested".into();
                }
                if let Some(result) = self
                    .evidence
                    .dom_comparison
                    .as_ref()
                    .filter(|a| a.revision.run == run.run.id)
                    .and_then(|a| a.revision.results.iter().find(|r| &r.key == key))
                {
                    return dom_outcome_label(&result.outcome).into();
                }
                match &target.dom_outcome {
                    DomTargetOutcome::NotRequested => "Omitted from the saved baseline".into(),
                    DomTargetOutcome::Pending => "Pending".into(),
                    DomTargetOutcome::Captured { .. } => {
                        if run.dom_evidence(key).is_some_and(|c| !c.metadata.stable) {
                            "Captured · unstable · not comparable"
                        } else {
                            "Captured"
                        }
                        .into()
                    }
                    DomTargetOutcome::Failed { code } => code.label().into(),
                    DomTargetOutcome::Skipped { reason } => format!("Skipped: {reason}"),
                    DomTargetOutcome::Cancelled => "Stopped".into(),
                }
            }
        }
    }
    pub(super) fn result_kind_label(&self, key: &TargetKey) -> String {
        let mode = self.recorded_mode();
        if mode == CaptureMode::ScreenshotsOnly {
            return self.kind_status(CheckKind::Screenshot, key);
        }
        if mode == CaptureMode::DomSnapshotsOnly {
            return format!("DOM: {}", self.kind_status(CheckKind::DomSnapshot, key));
        }
        format!(
            "Screenshot: {} · DOM: {}",
            self.kind_status(CheckKind::Screenshot, key),
            self.kind_status(CheckKind::DomSnapshot, key)
        )
    }
    pub(super) fn sync_dom_workspace(&mut self, cx: &mut Context<Self>) {
        let scope = self
            .evidence
            .latest
            .as_ref()
            .map(|r| (r.run.project, r.run.id));
        if self.dom.scope != scope {
            self.retain_dom_note(cx);
            self.jobs.cancel_dom_viewer();
            self.dom.scope = scope;
            self.dom.identity = None;
            self.dom.pending = None;
            self.dom.frame = None;
            self.dom.failed = false;
            self.dom.selected.clear();
            self.dom.approval = None;
            self.dom.presentation = DomPresentation::Diff;
            self.dom.kind = if self.recorded_mode() == CaptureMode::DomSnapshotsOnly {
                CheckKind::DomSnapshot
            } else {
                CheckKind::Screenshot
            };
        }
        if self.recorded_mode() == CaptureMode::DomSnapshotsOnly {
            self.dom.kind = CheckKind::DomSnapshot;
        }
        if self.recorded_mode() == CaptureMode::ScreenshotsOnly {
            self.dom.kind = CheckKind::Screenshot;
        }
        if self.dom_active()
            && self.focus.screen == Screen::Results
            && self.project.is_some()
            && !self.focus.shows_project_picker()
        {
            self.reload_dom_viewer(cx);
        }
    }
    fn dom_identity(&self) -> Option<DomViewerIdentity> {
        Some(DomViewerIdentity {
            project: self.project.as_ref()?.project.id,
            run: self.evidence.latest.as_ref()?.run.id,
            analysis: self.evidence.dom_comparison.as_ref().map(|a| a.revision.id),
            key: self.selected_key.clone()?,
            presentation: self.dom.presentation,
        })
    }
    pub(super) fn reload_dom_viewer(&mut self, cx: &mut Context<Self>) {
        let Some(identity) = self.dom_identity() else {
            if self.dom.identity.take().is_some() {
                self.jobs.cancel_dom_viewer();
                self.dom.pending = None;
                self.dom.frame = None;
                self.dom.source.update(cx, |source, cx| {
                    let token = source.begin_load("no selected DOM target", cx);
                    source.finish_load(
                        &token,
                        Err(regressee_gui::ui::dom_diff::DomDiffError::new(
                            "No DOM snapshot selected.",
                            "Choose a page and screen size from Results.",
                        )),
                        cx,
                    );
                });
            }
            return;
        };
        if self.dom.identity.as_ref() == Some(&identity)
            && (self.dom.pending.is_some() || self.dom.frame.is_some() || self.dom.failed)
        {
            return;
        }
        self.next_request += 1;
        let id = self.next_request;
        let token = self.dom.source.update(cx, |source, cx| {
            source.begin_load(format!("{identity:?}"), cx)
        });
        self.dom.identity = Some(identity.clone());
        self.dom.frame = None;
        self.dom.failed = false;
        if let Some(error) = self
            .kind_read_error(CheckKind::DomSnapshot)
            .map(str::to_owned)
        {
            self.dom.pending = None;
            self.dom.failed = true;
            self.dom.source.update(cx, |source, cx| { source.finish_load(&token, Err(regressee_gui::ui::dom_diff::DomDiffError::new("DOM evidence could not be read.", format!("{error} Refresh after repairing the saved files. Other check kinds remain independent; no website was visited."))), cx); });
            return;
        }
        match self.jobs.submit_dom_viewer(id, identity.clone()) {
            Ok(()) => self.dom.pending = Some((id, token, identity)),
            Err(error) => {
                self.dom.failed = true;
                self.dom.source.update(cx, |source, cx| {
                    source.finish_load(
                        &token,
                        Err(regressee_gui::ui::dom_diff::DomDiffError::new(
                            error,
                            "Reconnect to the shared engine and refresh this result.",
                        )),
                        cx,
                    );
                });
            }
        }
    }
    pub(super) fn poll_dom_viewer(&mut self, cx: &mut Context<Self>) {
        for _ in 0..4 {
            let Some(update) = self.jobs.take_dom_viewer() else {
                break;
            };
            let Some((id, _, identity)) = &self.dom.pending else {
                continue;
            };
            if *id != update.id
                || identity != &update.identity
                || self.dom_identity().as_ref() != Some(identity)
            {
                continue;
            }
            let (_, token, _) = self.dom.pending.take().unwrap();
            match update.result {
                Ok(frame) => {
                    let document = frame.document.clone();
                    self.dom.source.update(cx, |source, cx| {
                        source.finish_load(&token, Ok(document), cx);
                    });
                    self.dom.frame = Some(frame);
                    self.dom.failed = false;
                }
                Err(error) => {
                    self.dom.source.update(cx, |source, cx| {
                        source.finish_load(&token, Err(error), cx);
                    });
                    self.dom.failed = true;
                }
            }
            cx.notify();
        }
    }
    pub(super) fn choose_evidence_kind(&mut self, kind: CheckKind, cx: &mut Context<Self>) {
        if self.dom.kind == kind {
            return;
        }
        self.dom.kind = kind;
        self.viewer_mode_open = false;
        self.image_controls_open = false;
        self.review_tools = false;
        self.review_note_open = false;
        self.capture_details_open = false;
        self.mask_draw = false;
        self.mask_drag = None;
        self.viewer_pending = None;
        self.focus.view_loading = false;
        self.sync_review_note(cx);
        if self.dom_active() {
            self.reload_dom_viewer(cx);
        } else {
            self.jobs.cancel_dom_viewer();
            self.dom.pending = None;
            self.reload_viewer(cx);
        }
        cx.notify();
    }
    pub(super) fn dom_source_available(&self, pane: usize) -> bool {
        match pane {
            0 => self.dom_result().is_some_and(|r| r.outcome.comparable()),
            1 => {
                self.dom_result().is_some_and(|r| r.before.is_some())
                    || self.evidence.baseline.as_ref().is_some_and(|b| {
                        b.dom_references
                            .iter()
                            .any(|r| Some(&r.key) == self.selected_key.as_ref())
                    })
            }
            2 => self.evidence.latest.as_ref().is_some_and(|r| {
                self.selected_key
                    .as_ref()
                    .is_some_and(|key| r.dom_evidence(key).is_some())
            }),
            _ => false,
        }
    }
    pub(super) fn dom_capture_summary(&self) -> Vec<(String, String)> {
        let mut rows = vec![("Check kind".into(), "DOM snapshot".into())];
        if let Some(error) = self.kind_read_error(CheckKind::DomSnapshot) {
            rows.push(("Evidence read failed".into(), error.into()));
        }
        if let Some(key) = &self.selected_key {
            rows.push((
                "Result".into(),
                self.kind_status(CheckKind::DomSnapshot, key),
            ));
        }
        let (path, profile) = self.inspection_identity();
        rows.push(("Page".into(), path));
        rows.push(("Recorded context".into(), profile));
        if let Some(frame) = &self.dom.frame {
            for (label, original) in [
                ("Before", frame.artifacts.before.as_ref()),
                ("After", frame.artifacts.after.as_ref()),
            ] {
                if let Some(original) = original {
                    rows.push((
                        format!("{label} page address"),
                        original.metadata.final_url.clone(),
                    ));
                    rows.push((
                        format!("{label} capture"),
                        format!(
                            "{} nodes · {} bytes · {}",
                            original.metadata.node_count,
                            original.metadata.bytes,
                            if original.metadata.stable {
                                "Stable"
                            } else {
                                "Unstable"
                            }
                        ),
                    ));
                    if !original.metadata.warnings.is_empty() {
                        rows.push((
                            format!("{label} warnings"),
                            original.metadata.warnings.join("; "),
                        ));
                    }
                } else {
                    rows.push((format!("{label} source"), "Not available".into()));
                }
            }
        }
        rows.push(("Coverage".into(), DOM_COVERAGE_NOTICE.into()));
        rows.push(("Privacy".into(), DOM_PRIVACY_NOTICE.into()));
        if let Some(analysis) = &self.evidence.dom_comparison
            && let Some(decision) = self
                .selected_key
                .as_ref()
                .and_then(|key| self.saved_dom_decision(analysis.revision.id, key))
        {
            rows.push((
                "Saved DOM decision".into(),
                match decision.state {
                    regressee_core::domain::ReviewState::Unreviewed => "Unreviewed",
                    regressee_core::domain::ReviewState::Intentional => "Intentional",
                    regressee_core::domain::ReviewState::NeedsFix => "Needs fix",
                }
                .into(),
            ));
            if !decision.note.is_empty() {
                rows.push(("Saved DOM note".into(), decision.note.clone()));
            }
        }
        rows
    }
    pub(super) fn dom_stage(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut stage = div()
            .debug_selector(|| "dom-evidence-stage".into())
            .flex_1()
            .min_h_0()
            .min_w_0()
            .relative()
            .flex()
            .flex_col()
            .gap_1();
        if !self.viewer_inspector_open()
            && self.dom_source_available(1)
            && self.dom_source_available(2)
        {
            stage = stage.child(
                div().flex_shrink_0().flex().flex_wrap().gap_1().children(
                    [
                        (DomPresentation::Diff, "Diff"),
                        (DomPresentation::Before, "Before source"),
                        (DomPresentation::After, "After source"),
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(index, (presentation, label))| {
                        self.button(
                            Control::DomSource(index),
                            label,
                            ActionStyle::Choice {
                                selected: self.dom.presentation == presentation,
                            },
                            cx,
                        )
                    }),
                ),
            );
        }
        stage
            .child(div().flex_1().min_h_0().child(self.dom.source.clone()))
            .when(self.viewer_inspector_open(), |stage| {
                stage.child(self.viewer_inspector(cx))
            })
            .into_any_element()
    }
}
