// SPDX-License-Identifier: MPL-2.0
//! Explicit independently selected kinds share one revision-bound confirmation.
use super::*;
use crate::workflow::capture_time;
use regressee_app::engine::dom::{Command, KindKeys, KindSelection};
use regressee_core::{dom_snapshot::*, domain::CheckKind};

impl Shell {
    pub(super) fn dom_approval_candidate(&self, key: &TargetKey) -> Option<DomApprovalSelection> {
        if self.history_inspection.is_some()
            || self.kind_read_error(CheckKind::DomSnapshot).is_some()
            || self.mask_dirty
            || self.analysis_pending.is_some()
            || self.analysis_due.is_some()
        {
            return None;
        }
        let analysis = &self.evidence.dom_comparison.as_ref()?.revision;
        let run = self.evidence.latest.as_ref()?;
        let project = self.project.as_ref()?;
        let baseline = self
            .evidence
            .baseline
            .as_ref()
            .filter(|b| Some(b.id) == project.project.active_baseline)
            .or_else(|| {
                self.evidence
                    .baselines
                    .iter()
                    .find(|b| Some(b.id) == project.project.active_baseline)
            })?;
        if run.run.id != analysis.run
            || baseline.project != project.project.id
            || Some(baseline.id) != project.project.active_baseline
        {
            return None;
        }
        let result = analysis
            .results
            .iter()
            .find(|r| &r.key == key && r.outcome.comparable())?;
        let selected = DomApprovalSelection {
            analysis: analysis.id,
            key: key.clone(),
            before: result.before.clone()?,
            after: result.after.clone()?,
        };
        DomApprovalBinding {
            checked_baseline: analysis.baseline,
            current_parent: baseline.id,
            project_revision: project.revision,
            selections: vec![selected.clone()],
        }
        .validate_sources(analysis, baseline)
        .ok()?;
        let capture = run.dom_evidence(key)?;
        if !capture.metadata.baseline_ready()
            || regressee_core::evidence::dom_source(&run.run, capture)
                .ok()
                .as_ref()
                != Some(&selected.after)
        {
            return None;
        }
        Some(selected)
    }
    pub(super) fn kind_approval_selection_valid(&self) -> bool {
        if self.approval_selection.is_empty() && self.dom.selected.is_empty() {
            return false;
        }
        let screenshots = self.review_index();
        if !self
            .approval_selection
            .iter()
            .all(|key| screenshots.eligible.contains(key))
        {
            return false;
        }
        if self.dom.selected.is_empty() {
            return true;
        }
        if self.kind_read_error(CheckKind::DomSnapshot).is_some() {
            return false;
        }
        if self.history_inspection.is_some()
            || self.mask_dirty
            || self.analysis_pending.is_some()
            || self.analysis_due.is_some()
        {
            return false;
        }
        let (Some(comparison), Some(run), Some(project)) = (
            &self.evidence.dom_comparison,
            &self.evidence.latest,
            &self.project,
        ) else {
            return false;
        };
        let Some(baseline) = self
            .evidence
            .baseline
            .as_ref()
            .filter(|b| Some(b.id) == project.project.active_baseline)
            .or_else(|| {
                self.evidence
                    .baselines
                    .iter()
                    .find(|b| Some(b.id) == project.project.active_baseline)
            })
        else {
            return false;
        };
        if comparison.revision.run != run.run.id || baseline.project != project.project.id {
            return false;
        }
        // Index once rather than validating/scanning every target once per
        // selected kind. The engine repeats the complete core/CAS guard on save.
        let results: std::collections::BTreeMap<_, _> = comparison
            .revision
            .results
            .iter()
            .map(|r| (&r.key, r))
            .collect();
        let references: std::collections::BTreeMap<_, _> = baseline
            .dom_references
            .iter()
            .map(|r| (&r.key, r))
            .collect();
        let captures: std::collections::BTreeMap<_, _> = run
            .dom_captures
            .iter()
            .map(|c| (&c.metadata.key, c))
            .collect();
        self.dom.selected.iter().all(|(key, selected)| {
            let (Some(result), Some(reference), Some(capture)) =
                (results.get(key), references.get(key), captures.get(key))
            else {
                return false;
            };
            selected.analysis == comparison.revision.id
                && selected.key == *key
                && result.outcome.comparable()
                && result.before.as_ref() == Some(&selected.before)
                && result.after.as_ref() == Some(&selected.after)
                && reference.run == selected.before.run
                && reference.artifact == selected.before.artifact
                && reference.policy == selected.before.policy
                && baseline.capture_plan.dom_policy == Some(selected.after.policy)
                && capture.metadata.baseline_ready()
                && capture.metadata.run == selected.after.run
                && capture.metadata.key == selected.after.key
                && capture.metadata.sha256 == selected.after.artifact
                && capture.metadata.policy == selected.after.policy
                && capture.metadata.stable == selected.after.stable
        })
    }
    pub(super) fn kind_approval_matches(
        &self,
        preview: &regressee_storage::KindApprovalPreview,
    ) -> bool {
        if !self.kind_approval_selection_valid() {
            return false;
        }
        let Some(project) = &self.project else {
            return false;
        };
        let Some(run) = &self.evidence.latest else {
            return false;
        };
        if preview.baseline.project != project.project.id
            || preview.baseline.parent != project.project.active_baseline
        {
            return false;
        }
        let screenshot_matches = match &preview.screenshot {
            Some(p) => self.approval_matches(p),
            None => self.approval_selection.is_empty(),
        };
        let dom_matches = match &preview.dom {
            Some(p) => {
                let Some(analysis) = &self.evidence.dom_comparison else {
                    return false;
                };
                p.context.project == project.project.id
                    && p.context.run == run.run.id
                    && p.analysis == analysis.revision.id
                    && p.context.binding.project_revision == project.revision
                    && Some(p.context.binding.current_parent) == project.project.active_baseline
                    && p.context.binding.checked_baseline == analysis.revision.baseline
                    && p.selected == self.dom.selected.keys().cloned().collect::<Vec<_>>()
                    && p.context.binding.selections.len() == self.dom.selected.len()
                    && p.context
                        .binding
                        .selections
                        .iter()
                        .all(|s| self.dom.selected.get(&s.key) == Some(s))
            }
            None => self.dom.selected.is_empty(),
        };
        screenshot_matches && dom_matches
    }
    pub(super) fn approval_preview_current(&self) -> bool {
        if self.recorded_mode().dom_snapshots() {
            self.dom
                .approval
                .as_ref()
                .is_some_and(|p| self.kind_approval_matches(p))
        } else {
            self.approval
                .as_ref()
                .is_some_and(|p| self.approval_matches(p))
        }
    }
    pub(super) fn kind_approval_action(&mut self, index: usize, cx: &mut Context<Self>) {
        match index {
            23 => {
                let Some(key) = self.selected_key.clone() else {
                    return;
                };
                if self.dom_active() {
                    if self.dom.selected.remove(&key).is_none()
                        && self.dom_review_comparable()
                        && let Some(candidate) = self.dom_approval_candidate(&key)
                    {
                        self.dom.selected.insert(key, candidate);
                    }
                } else if !self.approval_selection.remove(&key) && self.approval_eligible(&key) {
                    self.approval_selection.insert(key);
                }
                self.invalidate_approval();
            }
            24 if self.kind_approval_selection_valid() => {
                if self.approval_preview_current() {
                    self.begin_approval_dialog();
                    return;
                }
                let screenshot = (!self.approval_selection.is_empty())
                    .then(|| {
                        self.evidence.comparison.as_ref().map(|a| KindKeys {
                            analysis: a.revision.id,
                            selected: self.approval_selection.iter().cloned().collect(),
                        })
                    })
                    .flatten();
                let dom = (!self.dom.selected.is_empty())
                    .then(|| {
                        self.evidence.dom_comparison.as_ref().map(|a| KindKeys {
                            analysis: a.revision.id,
                            selected: self.dom.selected.keys().cloned().collect(),
                        })
                    })
                    .flatten();
                self.queue(
                    Work::Dom(Command::Approval(KindSelection { screenshot, dom })),
                    cx,
                );
            }
            25 if self.focus.approval_dialog && self.approval_preview_current() => {
                if let Some(preview) = &self.dom.approval {
                    self.queue(Work::Dom(Command::Accept(preview.clone())), cx);
                }
            }
            _ => {}
        }
        cx.notify();
    }
    pub(super) fn kind_approval_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let selected = self.selected_key.as_ref().is_some_and(|key| {
            if self.dom_active() {
                self.dom.selected.contains_key(key)
            } else {
                self.approval_selection.contains(key)
            }
        });
        let label = if self.dom_active() {
            "Include After DOM snapshot"
        } else {
            "Include After screenshot"
        };
        div().debug_selector(|| "kind-approval-footer".into()).flex_shrink_0().px_6().py_2().bg(rgb(palette().surface)).border_t_1().border_color(rgb(palette().line))
            .flex().flex_col().gap_2()
            .child(div().flex().flex_wrap().items_baseline().gap_2().child(div().font_weight(FontWeight::SEMIBOLD).child("Update baseline"))
                .child(div().text_size(px(HELP_TEXT_SIZE)).line_height(px(HELP_LINE_HEIGHT)).text_color(rgb(palette().muted))
                     .child(format!("{} screenshots · {} DOM snapshots selected", self.approval_selection.len(), self.dom.selected.len()))))
            .child(div().debug_selector(|| "baseline-update-explanation".into()).text_size(px(HELP_TEXT_SIZE)).line_height(px(HELP_LINE_HEIGHT)).text_color(rgb(palette().muted))
                .child("Use After as the reference for future checks. Review before saving."))
            .child(div().flex().flex_wrap().gap_2().when(self.control_mounted(Control::Phase5(23)), |row| row.child(self.toggle_control(Control::Phase5(23), label, selected, cx)))
                .when(self.control_mounted(Control::Phase5(24)), |row| row.child(self.button(Control::Phase5(24), "Review update…", ActionStyle::Primary, cx))))
            .when(!self.approval_selection.is_empty() || !self.dom.selected.is_empty(), |bar| bar.child(div().text_size(px(HELP_TEXT_SIZE)).line_height(px(HELP_LINE_HEIGHT)).text_color(rgb(palette().muted)).child(
                if self.kind_approval_selection_valid() { "Only the explicitly selected kinds will change. Review their sources before saving." } else { "A selection is stale or cannot update the baseline. Deselect it or refresh; nothing has been saved." })))
            .into_any_element()
    }
    pub(super) fn kind_approval_rows(&self) -> usize {
        self.dom.approval.as_ref().map_or(0, |p| {
            p.baseline.references.len() + p.baseline.dom_references.len()
        })
    }
    pub(super) fn kind_approval_decision(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(preview) = self
            .dom
            .approval
            .as_ref()
            .filter(|p| self.kind_approval_matches(p))
        else {
            return div().into_any_element();
        };
        let index = self.review_index();
        let count = self.kind_approval_rows();
        let current = self.approval_reference_index.min(count.saturating_sub(1));
        let mut flow = div().flex().flex_col().gap_3()
            .child(format!("Screenshots: {} replace · {} keep. DOM snapshots: {} replace · {} keep.", self.approval_selection.len(), preview.baseline.references.len().saturating_sub(self.approval_selection.len()), self.dom.selected.len(), preview.baseline.dom_references.len().saturating_sub(self.dom.selected.len())))
            .child("Only the selected After references become the baseline. Every unselected kind stays unchanged. Earlier versions remain in History.")
            .child("Source capture times can differ. This is not one simultaneous site snapshot.");
        let start = current / 24 * 24;
        for position in start..(start + 24).min(count) {
            let (kind, key, run, artifact, captured_at) =
                if let Some(r) = preview.baseline.references.get(position) {
                    (
                        CheckKind::Screenshot,
                        &r.key,
                        r.run,
                        r.artifact.as_str(),
                        r.captured_at,
                    )
                } else {
                    let r = &preview.baseline.dom_references
                        [position - preview.baseline.references.len()];
                    (
                        CheckKind::DomSnapshot,
                        &r.key,
                        r.run,
                        r.artifact.as_str(),
                        r.captured_at,
                    )
                };
            let selected = if kind == CheckKind::Screenshot {
                self.approval_selection.contains(key)
            } else {
                self.dom.selected.contains_key(key)
            };
            let page = index
                .target_labels
                .get(key)
                .map(|url| controls::page_path(url))
                .unwrap_or_else(|| key.page.to_string());
            let profile = index
                .profile_labels
                .get(&key.profile)
                .cloned()
                .unwrap_or_else(|| key.profile.to_string());
            flow = flow.child(div().debug_selector(move || format!("kind-approval-source-{position}")).flex_shrink_0().py_2().px_2().border_b_1().border_color(rgb(palette().line)).flex().flex_col().gap_1()
                .when(position == current, |row| row.bg(rgb(palette().selected)))
                .child(div().font_weight(FontWeight::SEMIBOLD).child(format!("{} {} · {page}", if selected { "Replace" } else { "Keep" }, if kind == CheckKind::Screenshot { "screenshot" } else { "DOM snapshot" })))
                .child(format!("{profile} · {}", capture_time(captured_at)))
                .when(position == current && self.approval_exact_open, |row| row.child(div().text_size(px(HELP_TEXT_SIZE)).line_height(px(HELP_LINE_HEIGHT)).child(format!("Source run: {run}\nArtifact: {artifact}\nPage: {}\nProfile: {}", key.page, key.profile)))));
        }
        let panel =
            regressee_gui::ui::dialog::surface(if self.narrow_workspace { 740. } else { 760. })
                .debug_selector(|| "kind-approval-decision".into())
                .h(px(520.))
                .p_6()
                .gap_3()
                .child(
                    regressee_gui::ui::dialog::heading()
                        .child("Use the selected After references for future checks?"),
                )
                .child(scrollbar::vertical(
                    "kind-approval-scrollbar",
                    &self.approval_provenance_scroll,
                    div()
                        .id("kind-approval-sources")
                        .flex_1()
                        .min_h_0()
                        .overflow_y_scroll()
                        .track_scroll(&self.approval_provenance_scroll)
                        .pr(px(scrollbar::GUTTER))
                        .child(flow),
                ))
                .child(
                    div()
                        .text_size(px(HELP_TEXT_SIZE))
                        .line_height(px(HELP_LINE_HEIGHT))
                        .text_color(rgb(palette().muted))
                        .child(format!(
                            "Reference {} of {count}. Previous/Next reaches every retained source.",
                            current + 1
                        )),
                )
                .child(
                    regressee_gui::ui::dialog::footer()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .child(self.guided_button(48, "Previous", cx))
                        .child(self.guided_button(49, "Next", cx))
                        .child(self.guided_button(
                            52,
                            if self.approval_exact_open {
                                "Hide source details"
                            } else {
                                "Source details"
                            },
                            cx,
                        ))
                        .child(self.guided_button(47, "Keep reviewing", cx))
                        .child(self.button(
                            Control::Phase5(25),
                            "Save baseline update",
                            ActionStyle::Primary,
                            cx,
                        )),
                );
        regressee_gui::ui::dialog::present("kind-approval-dialog-entrance", panel, cx)
            .id("approval-modal-backdrop")
            .into_any_element()
    }
}
