// SPDX-License-Identifier: MPL-2.0
//! DOM decisions and note drafts never borrow screenshot review identities.
use super::*;
use regressee_app::engine::dom::{Command, Output};
use regressee_core::{
    dom_snapshot::*,
    domain::{DomAnalysisId, ReviewState},
};

fn same_note(a: &DomReviewDecision, b: &DomReviewDecision) -> bool {
    a.analysis == b.analysis && a.key == b.key && a.before == b.before && a.after == b.after
}
impl Shell {
    pub(super) fn saved_dom_decision(
        &self,
        analysis: DomAnalysisId,
        key: &TargetKey,
    ) -> Option<&DomReviewDecision> {
        self.evidence
            .dom_reviews
            .iter()
            .find(|d| d.analysis == analysis && &d.key == key)
    }
    pub(super) fn dom_note_template(&self) -> Option<DomReviewDecision> {
        let analysis = self.evidence.dom_comparison.as_ref()?.revision.id;
        let result = self.dom_result()?;
        if !result.outcome.needs_review() {
            return None;
        }
        let saved = self.saved_dom_decision(analysis, &result.key);
        Some(DomReviewDecision {
            analysis,
            key: result.key.clone(),
            before: result.before.clone()?,
            after: result.after.clone()?,
            state: saved.map_or(ReviewState::Unreviewed, |d| d.state),
            note: saved.map_or_else(String::new, |d| d.note.clone()),
            reviewed_at: 0,
        })
    }
    pub(super) fn retain_dom_note(&mut self, cx: &App) {
        if let Some(mut active) = self.dom.active_note.take() {
            active.note = self.analysis_fields[9].read(cx).content.to_string();
            self.dom
                .notes
                .insert((active.analysis, active.key.clone()), active);
        }
    }
    pub(super) fn sync_dom_note(&mut self, cx: &mut Context<Self>) {
        let template = self.dom_note_template();
        if template
            .as_ref()
            .zip(self.dom.active_note.as_ref())
            .is_some_and(|(a, b)| same_note(a, b))
        {
            return;
        }
        self.retain_dom_note(cx);
        let note = template
            .as_ref()
            .map(|template| {
                self.dom
                    .notes
                    .get(&(template.analysis, template.key.clone()))
                    .filter(|d| same_note(d, template))
                    .map(|d| d.note.clone())
                    .unwrap_or_else(|| template.note.clone())
            })
            .unwrap_or_default();
        self.analysis_fields[9].update(cx, |input, cx| input.set_value(&note, cx));
        self.dom.active_note = template;
    }
    pub(super) fn dom_note_for_save(&self, cx: &App) -> Result<String, &'static str> {
        let template = self
            .dom_note_template()
            .ok_or("Reopen this changed DOM result before saving its note. Your draft is kept.")?;
        if !self
            .dom
            .active_note
            .as_ref()
            .is_some_and(|active| same_note(active, &template))
        {
            return Err(
                "Reopen the original DOM result before saving its note. Your draft is kept.",
            );
        }
        let input = self.analysis_fields[9].read(cx);
        if input.composing() {
            return Err("Finish entering your text before saving the note.");
        }
        regressee_core::domain::validate_review_note(&input.content)?;
        Ok(input.content.to_string())
    }
    pub(super) fn dom_note_dirty(&self, cx: &App) -> bool {
        self.dom.active_note.as_ref().is_some_and(|active| {
            self.analysis_fields[9].read(cx).content.as_ref()
                != self
                    .saved_dom_decision(active.analysis, &active.key)
                    .filter(|d| same_note(d, active))
                    .map_or("", |d| d.note.as_str())
        })
    }
    pub(super) fn pending_dom_notes(
        &self,
        cx: &App,
    ) -> Result<Vec<DomReviewDecision>, &'static str> {
        let mut notes = self.dom.notes.clone();
        if let Some(active) = &self.dom.active_note {
            let mut active = active.clone();
            active.note = self.analysis_fields[9].read(cx).content.to_string();
            notes.insert((active.analysis, active.key.clone()), active);
        }
        let mut pending = Vec::new();
        for (_, mut draft) in notes {
            let saved = self
                .saved_dom_decision(draft.analysis, &draft.key)
                .filter(|d| same_note(d, &draft));
            if saved.map_or("", |d| d.note.as_str()) == draft.note {
                continue;
            }
            regressee_core::domain::validate_review_note(&draft.note)?;
            draft.state = saved.map_or(ReviewState::Unreviewed, |d| d.state);
            pending.push(draft);
        }
        Ok(pending)
    }
    pub(super) fn reconcile_dom_review(&mut self, decision: DomReviewDecision) {
        self.dom
            .notes
            .retain(|_, draft| !(same_note(draft, &decision) && draft.note == decision.note));
        self.evidence
            .dom_reviews
            .retain(|saved| !(saved.analysis == decision.analysis && saved.key == decision.key));
        self.evidence.dom_reviews.insert(0, decision);
    }
    pub(super) fn dom_review_comparable(&self) -> bool {
        if self
            .kind_read_error(regressee_core::domain::CheckKind::DomSnapshot)
            .is_some()
        {
            return false;
        }
        if self.dom.failed || self.dom.pending.is_some() {
            return false;
        }
        let Some(result) = self.dom_result() else {
            return false;
        };
        let Some(frame) = &self.dom.frame else {
            return false;
        };
        result.outcome.comparable()
            && self.dom.identity.as_ref() == Some(&frame.identity)
            && frame.artifacts.key == result.key
            && frame.artifacts.result.as_ref().is_some_and(|loaded| {
                loaded.before == result.before
                    && loaded.after == result.after
                    && loaded.outcome == result.outcome
            })
    }
    pub(super) fn dom_review_valid(&self) -> bool {
        self.dom_review_comparable()
            && self.history_inspection.is_none()
            && !self.focus.busy
            && !self.focus.dialog
            && !self.focus.approval_dialog
            && !self.native_prompt
            && !self.agent_locked()
    }
    pub(super) fn save_dom_review(&mut self, index: usize, cx: &mut Context<Self>) {
        self.sync_review_note(cx);
        if !self.dom_review_valid() {
            return;
        }
        let Some(mut decision) = self.dom_note_template() else {
            return;
        };
        let note = match self.dom_note_for_save(cx) {
            Ok(note) => note,
            Err(message) => {
                self.analysis_fields[9].update(cx, |input, cx| {
                    input.correction = Some(message.into());
                    input.touched = true;
                    cx.notify();
                });
                return;
            }
        };
        decision.note = note;
        decision.state = [
            ReviewState::Unreviewed,
            ReviewState::Intentional,
            ReviewState::NeedsFix,
        ][index - 20];
        decision.reviewed_at = regressee_runtime::distribution::now();
        self.queue(Work::Dom(Command::Review(Box::new(decision))), cx);
    }
    pub(super) fn apply_dom_reply(&mut self, output: Output, cx: &mut Context<Self>) {
        match output {
            Output::Review(decision) => {
                self.reconcile_dom_review(decision);
                if self.review_note_open {
                    self.notice = None;
                } else {
                    self.notice = Some("DOM review decision saved. Screenshot decisions and the baseline have not changed.".into());
                }
            }
            Output::Evidence(evidence) => {
                self.evidence = *evidence;
                self.evidence_dirty = false;
                self.dom.identity = None;
                self.sync_dom_workspace(cx);
            }
            Output::Analysis(analysis) => {
                self.evidence.dom_comparison = Some(*analysis);
                self.dom.identity = None;
                self.invalidate_approval();
                self.sync_dom_workspace(cx);
            }
            Output::Project(record) => {
                self.project = Some(*record);
                self.dom.selected.clear();
                self.approval_selection.clear();
                self.invalidate_approval();
                self.evidence_dirty = true;
                self.dom.identity = None;
                self.refresh_visible(cx);
            }
            Output::Approval(preview) => {
                if self.approval_request_epoch.take() == Some(self.approval_epoch)
                    && self.kind_approval_matches(&preview)
                {
                    self.dom.approval = Some(preview);
                    self.begin_approval_dialog();
                }
            }
            _ => {
                self.error = Some("The engine returned an unexpected DOM response. Refresh the saved result before continuing.".into());
            }
        }
        cx.notify();
    }
    pub(super) fn dom_review_progress(&self) -> String {
        let Some(comparison) = &self.evidence.dom_comparison else {
            return "No DOM comparison to review.".into();
        };
        let mut remaining = 0;
        let mut reviewed = 0;
        let mut gaps = 0;
        for result in &comparison.revision.results {
            if result.outcome.needs_review() {
                if self
                    .saved_dom_decision(comparison.revision.id, &result.key)
                    .is_none_or(|d| d.state == ReviewState::Unreviewed)
                {
                    remaining += 1;
                } else {
                    reviewed += 1;
                }
            } else if !matches!(
                result.outcome,
                DomComparisonOutcome::Unchanged | DomComparisonOutcome::NotRequested
            ) {
                gaps += 1;
            }
        }
        let mut label = format!("{reviewed} DOM results reviewed · {remaining} changes to review");
        if gaps > 0 {
            label.push_str(&format!(" · {gaps} not compared"));
        }
        label
    }
    pub(super) fn dom_review_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.viewer_inspector_open() || self.options_panel_open() {
            return div()
                .text_size(px(HELP_TEXT_SIZE))
                .line_height(px(HELP_LINE_HEIGHT))
                .text_color(rgb(palette().muted))
                .child("Close this panel to review the DOM snapshot.")
                .into_any_element();
        }
        let unchanged = self
            .dom_result()
            .is_some_and(|r| r.outcome == DomComparisonOutcome::Unchanged);
        let state = self
            .evidence
            .dom_comparison
            .as_ref()
            .and_then(|a| {
                self.selected_key
                    .as_ref()
                    .and_then(|key| self.saved_dom_decision(a.revision.id, key))
            })
            .map_or(ReviewState::Unreviewed, |d| d.state);
        if self.history_inspection.is_some() {
            return div()
                .text_size(px(HELP_TEXT_SIZE))
                .line_height(px(HELP_LINE_HEIGHT))
                .text_color(rgb(palette().muted))
                .child(format!(
                    "Saved DOM result: {} · Read-only",
                    self.dom_result()
                        .map_or("Not compared", |r| dom_ux::dom_outcome_label(&r.outcome))
                ))
                .into_any_element();
        }
        if !self.dom_review_comparable() {
            let message = if self.dom.pending.is_some() {
                "Loading the selected DOM source…"
            } else {
                self.dom_result()
                    .map_or("No DOM comparison result is available.", |r| {
                        dom_ux::dom_outcome_label(&r.outcome)
                    })
            };
            return div().debug_selector(|| "dom-review-unavailable".into()).flex_shrink_0().pt_2().border_t_1().border_color(rgb(palette().line)).flex().flex_wrap().items_center().gap_2()
                .child(div().flex_1().min_w(px(180.)).text_size(px(HELP_TEXT_SIZE)).line_height(px(HELP_LINE_HEIGHT)).child(format!("DOM comparison: {message} Missing coverage needs a new baseline capture; Refresh only reads saved files.")))
                .when(self.control_mounted(Control::Phase5(33)), |bar| bar.child(self.button(Control::Phase5(33), "Next unreviewed", ActionStyle::Secondary, cx))).into_any_element();
        }
        let mut actions = div().flex().flex_wrap().gap_2();
        if !unchanged {
            for (index, label, value) in [
                (20, "Unreviewed", ReviewState::Unreviewed),
                (21, "Intentional", ReviewState::Intentional),
                (22, "Needs fix", ReviewState::NeedsFix),
            ] {
                actions = actions.child(self.button(
                    Control::Phase5(index),
                    label,
                    ActionStyle::Choice {
                        selected: state == value,
                    },
                    cx,
                ));
            }
            actions = actions.child(self.button(
                Control::Phase5(37),
                "Review note",
                ActionStyle::Secondary,
                cx,
            ));
        }
        div()
            .debug_selector(|| "dom-review-decisions".into())
            .flex_shrink_0()
            .pt_2()
            .border_t_1()
            .border_color(rgb(palette().line))
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(if unchanged {
                        "DOM snapshots are identical"
                    } else {
                        "Review DOM changes"
                    }))
                    .child(
                        div()
                            .text_size(px(HELP_TEXT_SIZE))
                            .line_height(px(HELP_LINE_HEIGHT))
                            .text_color(rgb(palette().muted))
                            .child(self.dom_review_progress()),
                    ),
            )
            .child(
                actions.when(self.control_mounted(Control::Phase5(33)), |actions| {
                    actions.child(self.button(
                        Control::Phase5(33),
                        "Next unreviewed",
                        ActionStyle::Primary,
                        cx,
                    ))
                }),
            )
            .into_any_element()
    }
}
