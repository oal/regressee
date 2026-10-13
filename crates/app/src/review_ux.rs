// SPDX-License-Identifier: MPL-2.0
//! Ordinary review controls; geometry edits remain drafts until the storage action.
use super::*;
use regressee_app::analysis_engine::{Dimensions, Rectangle};
use regressee_core::domain::{AnalysisId, ReviewDecision, ReviewState, TargetKey};

#[derive(Default)]
pub(super) struct ReviewDrafts {
    active: Option<(AnalysisId, TargetKey)>,
    notes: std::collections::BTreeMap<(AnalysisId, TargetKey), String>,
}

#[cfg(all(test, feature = "ui-tests"))]
impl ReviewDrafts {
    pub(super) fn exit_fixture(
        active: Option<(AnalysisId, TargetKey)>,
        notes: Vec<((AnalysisId, TargetKey), String)>,
    ) -> Self {
        Self {
            active,
            notes: notes.into_iter().collect(),
        }
    }
}

fn current_decision<'a>(
    reviews: &'a [ReviewDecision],
    analysis: AnalysisId,
    key: &TargetKey,
) -> Option<&'a ReviewDecision> {
    // Storage returns newest inserted decision first. Timestamps can tie or
    // move backwards; they must not resurrect an older disposition.
    reviews
        .iter()
        .find(|d| d.analysis == analysis && &d.key == key)
}

pub(super) fn navigation_preserves_drafts(
    mask_dirty: bool,
    geometry_dirty: bool,
    note_dirty: bool,
) -> bool {
    !(mask_dirty || geometry_dirty || note_dirty)
}

fn geometry(
    values: [&str; 4],
    before: Dimensions,
    after: Dimensions,
) -> Result<Rectangle, &'static str> {
    let mut n = [0; 4];
    for (i, value) in values.iter().enumerate() {
        n[i] = value
            .trim()
            .parse::<u32>()
            .map_err(|_| "Use whole non-negative pixel values for X, Y, Width and Height.")?;
    }
    let r = Rectangle {
        x: n[0],
        y: n[1],
        width: n[2],
        height: n[3],
    };
    if !fits(r, before) && !fits(r, after) {
        return Err(
            "The area must have positive dimensions and fit within an original screenshot. Nothing was changed.",
        );
    }
    Ok(r)
}

fn fits(r: Rectangle, d: Dimensions) -> bool {
    r.width > 0
        && r.height > 0
        && r.x.checked_add(r.width).is_some_and(|x| x <= d.width)
        && r.y.checked_add(r.height).is_some_and(|y| y <= d.height)
}

pub(super) fn all_masked(rectangles: &[Rectangle], before: Dimensions, after: Dimensions) -> bool {
    fn covered(rectangles: &[Rectangle], d: Dimensions) -> bool {
        if d.width == 0 || d.height == 0 {
            return true;
        }
        let mut rows = vec![0, d.height];
        for r in rectangles {
            rows.extend([
                r.y.min(d.height),
                r.y.saturating_add(r.height).min(d.height),
            ]);
        }
        rows.sort_unstable();
        rows.dedup();
        for band in rows.windows(2) {
            let mut spans: Vec<_> = rectangles
                .iter()
                .filter(|r| r.y <= band[0] && r.y.saturating_add(r.height) >= band[1])
                .map(|r| (r.x.min(d.width), r.x.saturating_add(r.width).min(d.width)))
                .collect();
            spans.sort_unstable();
            let mut end = 0;
            for (left, right) in spans {
                if left > end {
                    break;
                }
                end = end.max(right);
            }
            if end < d.width {
                return false;
            }
        }
        true
    }
    covered(rectangles, before) && covered(rectangles, after)
}

fn seed(before: Dimensions, after: Dimensions) -> Option<Rectangle> {
    let d = if before.width > 0 && before.height > 0 {
        before
    } else {
        after
    };
    // Never seed an all-image mask, even on unusually small captures.
    if u64::from(d.width) * u64::from(d.height) < 2 {
        return None;
    }
    Some(Rectangle {
        x: 0,
        y: 0,
        width: (d.width / 4).clamp(1, 80),
        height: (d.height / 4).clamp(1, 80),
    })
}

impl Shell {
    /// Controls inside the bounded Tools scroller share explicit scroll anchors.
    pub(super) fn review_anchor(&self, control: Control) -> Option<gpui::ScrollAnchor> {
        if let Some(anchor) = self.options_anchor(control) {
            return Some(anchor);
        }
        let index = match control {
            Control::Phase5(index @ (0..=5 | 12..=19 | 26..=28 | 34..=35)) => index,
            Control::AnalysisField(index @ (0..=8 | 10..=13)) => 36 + index,
            Control::Details(Detail::Analysis) => 50,
            _ => return None,
        };
        self.review_anchors.get(index).cloned()
    }

    pub(super) fn reveal_review_control(
        &self,
        control: Control,
        window: &mut Window,
        cx: &mut App,
    ) {
        if self.focus.screen == Screen::Results
            && self.options_panel_open()
            && self.control_mounted(control)
            && let Some(bounds) = self.options_control_bounds(control).and_then(|b| b.get())
        {
            let viewport = self.options_scroll.bounds();
            let delta = if bounds.bottom() > viewport.bottom() - px(8.) {
                viewport.bottom() - px(8.) - bounds.bottom()
            } else if bounds.top() < viewport.top() + px(8.) {
                viewport.top() + px(8.) - bounds.top()
            } else {
                px(0.)
            };
            let offset = self.options_scroll.offset();
            self.options_scroll
                .set_offset(gpui::point(offset.x, offset.y + delta));
            return;
        }
        if self.focus.screen == Screen::Results
            && self.review_tools
            && self.control_mounted(control)
            && let Some(anchor) = self.review_anchor(control)
        {
            anchor.scroll_to(window, cx);
            if matches!(control, Control::AnalysisField(10..=13)) {
                self.review_focus_reveal.set(2);
            }
            // Painted input bounds also support headless frames, which have no
            // OS frame callbacks, and immediate keyboard geometry navigation.
            if let Control::AnalysisField(index) = control
                && let Some(bounds) = self.analysis_fields[index].read(cx).rendered_bounds()
            {
                let viewport = self.review_details_scroll.bounds();
                let offset = self.review_details_scroll.offset();
                self.review_details_scroll.set_offset(gpui::point(
                    offset.x,
                    offset.y + viewport.top() + px(28.) - bounds.top(),
                ));
            }
        }
    }

    /// One exit snapshot of all pending notes. The mounted text belongs to
    /// `active`, not to a newly selected target awaiting `sync_review_note`.
    pub(super) fn pending_review_notes(
        &self,
        cx: &App,
    ) -> Result<Vec<ReviewDecision>, &'static str> {
        let mut notes = self.review_drafts.notes.clone();
        if let Some(identity) = &self.review_drafts.active {
            notes.insert(
                identity.clone(),
                self.analysis_fields[9].read(cx).content.to_string(),
            );
        } else if !self.analysis_fields[9].read(cx).content.is_empty() {
            return Err(
                "The note has no active review binding. Reopen its original result before saving it.",
            );
        }
        let mut pending = Vec::new();
        for ((analysis, key), note) in notes {
            let saved = current_decision(&self.evidence.reviews, analysis, &key);
            if saved.map(|d| d.note.as_str()).unwrap_or("") == note.as_str() {
                continue;
            }
            if note.chars().count() > 8192 {
                return Err(
                    "Shorten the pending review note to 8,192 characters. No note was truncated or saved.",
                );
            }
            pending.push(ReviewDecision {
                analysis,
                key,
                note,
                state: saved.map(|d| d.state).unwrap_or(ReviewState::Unreviewed),
                reviewed_at: 0,
            });
        }
        Ok(pending)
    }
    /// Includes retained notes and unapplied numeric input, even if its control
    /// is no longer selected. Committed reply data is deliberately not input.
    pub(super) fn review_exit_signature(&self, cx: &App) -> String {
        format!(
            "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
            self.review_drafts.active,
            self.review_drafts.notes,
            self.analysis_fields[9..14]
                .iter()
                .map(|v| v.read(cx).content.to_string())
                .collect::<Vec<_>>(),
            self.mask_selected,
            self.mask_rectangles,
            self.mask_drag,
            self.viewer.as_ref().and_then(|v| v.request.as_ref())
        )
    }
    /// Only retire drafts whose exact identity and text match the authoritative
    /// reply. Never replace mounted text or erase newer retained edits.
    pub(super) fn reconcile_exit_review_reply(&mut self, reviews: &[ReviewDecision]) {
        self.review_drafts.notes.retain(|(analysis, key), note| {
            current_decision(reviews, *analysis, key)
                .map(|d| d.note.as_str())
                .unwrap_or("")
                != note.as_str()
        });
    }
    /// Called only after the explicit Discard-and-quit decision. Saved review
    /// disposition and storage evidence are untouched.
    pub(super) fn discard_exit_review_drafts(&mut self, cx: &mut Context<Self>) {
        if self.mask_dirty {
            self.mask_rectangles = self
                .viewer
                .as_ref()
                .and_then(|v| v.request.as_ref())
                .and_then(|r| r.mask.as_ref())
                .map(|m| m.rectangles.clone())
                .unwrap_or_default();
            self.mask_dirty = false;
        }
        self.mask_drag = None;
        self.mask_selected = self
            .mask_selected
            .filter(|i| *i < self.mask_rectangles.len());
        self.discard_review_drafts(cx);
        if self.mask_selected.is_none() {
            for (i, value) in ["0", "0", "1", "1"].into_iter().enumerate() {
                self.analysis_fields[10 + i].update(cx, |input, cx| input.set_value(value, cx));
            }
        }
    }
    pub(super) fn review_save_completed(&mut self, cx: &App) {
        if let Some(identity) = self.review_identity()
            && !self.review_note_dirty(cx)
        {
            self.review_drafts.notes.remove(&identity);
        }
    }
    /// Exit/navigation guards should include both the mounted note and notes
    /// retained when explicit queue navigation changed the inspected target.
    pub(super) fn review_unsaved(&self, cx: &App) -> bool {
        self.review_note_dirty(cx)
            || self.mask_geometry_dirty(cx)
            || self
                .review_drafts
                .notes
                .iter()
                .any(|((analysis, key), note)| {
                    current_decision(&self.evidence.reviews, *analysis, key)
                        .map(|d| d.note.as_str())
                        .unwrap_or("")
                        != note
                })
    }
    pub(super) fn discard_review_drafts(&mut self, cx: &mut Context<Self>) {
        self.review_drafts = Default::default();
        self.sync_review_note(cx);
        self.load_mask_fields(cx);
    }
    pub(super) fn mask_geometry_dirty(&self, cx: &App) -> bool {
        let Some(r) = self.mask_selected.and_then(|i| self.mask_rectangles.get(i)) else {
            return false;
        };
        [r.x, r.y, r.width, r.height]
            .into_iter()
            .enumerate()
            .any(|(i, n)| {
                self.analysis_fields[10 + i]
                    .read(cx)
                    .content
                    .trim()
                    .parse::<u32>()
                    != Ok(n)
            })
    }
    pub(super) fn capture_provenance_details(&self) -> AnyElement {
        let mut details = div().flex().flex_col().gap_2().text_size(px(13.));
        details = details.child(
            div()
                .text_color(rgb(MUTED))
                .child(self.comparison_context()),
        );
        if let Some(a) = &self.evidence.comparison {
            details = details.child(format!(
                "Baseline {} · run {} · analysis {}",
                a.revision.baseline, a.revision.run, a.revision.id
            ));
        }
        if let Some(key) = &self.selected_key {
            details = details.child(format!("Page {} · profile {}", key.page, key.profile));
            let baseline_id = self
                .evidence
                .comparison
                .as_ref()
                .map(|a| a.revision.baseline);
            let baseline = self
                .evidence
                .baseline
                .as_ref()
                .filter(|b| Some(b.id) == baseline_id)
                .or_else(|| {
                    self.evidence
                        .baselines
                        .iter()
                        .find(|b| Some(b.id) == baseline_id)
                });
            if let Some(reference) =
                baseline.and_then(|b| b.references.iter().find(|r| &r.key == key))
            {
                details = details.child(format!(
                    "Before: source run {} · captured {} UTC Unix seconds · original {}",
                    reference.run,
                    reference.captured_at,
                    reference.artifact.as_str()
                ));
                let capture = self
                    .history
                    .as_ref()
                    .and_then(|h| h.runs.iter().find(|r| r.run.id == reference.run))
                    .and_then(|r| r.evidence(key))
                    .or_else(|| {
                        self.evidence.before.as_ref().filter(|c| {
                            &c.metadata.key == key && c.image.original == reference.artifact
                        })
                    });
                details = details.child(capture.map(|c| format!("Before requested URL: {}\nBefore final URL: {}\nBefore capture metadata: {:?}", c.metadata.requested, c.metadata.final_url, c.metadata)).unwrap_or_else(|| "Before full URL/capture record is not loaded; no URL inferred from today's project settings.".into()));
            }
            let run_id = self.evidence.comparison.as_ref().map(|a| a.revision.run);
            let run = self
                .evidence
                .latest
                .as_ref()
                .filter(|r| run_id.is_none_or(|id| id == r.run.id))
                .or_else(|| {
                    self.history
                        .as_ref()
                        .and_then(|h| h.runs.iter().find(|r| Some(r.run.id) == run_id))
                });
            if let Some(run) = run {
                details = details.child(format!(
                    "After: run {} · started {:?} · finished {:?}",
                    run.run.id, run.run.started_at, run.run.finished_at
                ));
                if let Some(c) = run.evidence(key) {
                    details = details.child(format!("After original {}\nAfter requested URL: {}\nAfter final URL: {}\nAfter capture metadata: {:?}", c.image.original.as_str(), c.metadata.requested, c.metadata.final_url, c.metadata));
                }
            }
            if let Some(request) = self.viewer.as_ref().and_then(|v| v.request.as_ref()) {
                details = details.child(format!(
                    "Exact pair descriptors and ignored-area revision: {request:?}"
                ));
            }
        }
        details.into_any_element()
    }
    fn review_identity(&self) -> Option<(AnalysisId, TargetKey)> {
        Some((
            self.evidence.comparison.as_ref()?.revision.id,
            self.selected_key.clone()?,
        ))
    }
    pub(super) fn sync_review_note(&mut self, cx: &mut Context<Self>) {
        let identity = self.review_identity();
        if identity == self.review_drafts.active {
            return;
        }
        if let Some(old) = self.review_drafts.active.take() {
            self.review_drafts
                .notes
                .insert(old, self.analysis_fields[9].read(cx).content.to_string());
        }
        let note = identity
            .as_ref()
            .map(|(analysis, key)| {
                self.review_drafts
                    .notes
                    .get(&(*analysis, key.clone()))
                    .cloned()
                    .unwrap_or_else(|| {
                        current_decision(&self.evidence.reviews, *analysis, key)
                            .map(|d| d.note.clone())
                            .unwrap_or_default()
                    })
            })
            .unwrap_or_default();
        self.analysis_fields[9].update(cx, |input, cx| input.set_value(&note, cx));
        self.review_drafts.active = identity;
    }
    pub(super) fn review_note_dirty(&self, cx: &App) -> bool {
        let Some((analysis, key)) = self.review_identity() else {
            return false;
        };
        if self.review_drafts.active.as_ref() != Some(&(analysis, key.clone())) {
            return false;
        }
        self.analysis_fields[9].read(cx).content.as_ref()
            != current_decision(&self.evidence.reviews, analysis, &key)
                .map(|d| d.note.as_str())
                .unwrap_or("")
    }
    pub(super) fn review_result_valid(&self) -> bool {
        if self.focus.busy
            || self.focus.dialog
            || self.focus.approval_dialog
            || self.native_prompt
            || self.mask_dirty
            || self.analysis_due.is_some()
            || self.analysis_pending.is_some()
        {
            return false;
        }
        let Some(a) = &self.evidence.comparison else {
            return false;
        };
        self.evidence
            .latest
            .as_ref()
            .is_some_and(|run| run.run.id == a.revision.run)
            && a.revision.results.iter().any(|r| {
                Some(&r.key) == self.selected_key.as_ref()
                    && r.eligible_pixels > 0
                    && matches!(
                        r.classification,
                        Classification::Changed | Classification::NoSignificantChange
                    )
            })
    }
    pub(super) fn next_unreviewed_key(&self) -> Option<TargetKey> {
        let a = self.evidence.comparison.as_ref()?;
        let run = self.evidence.latest.as_ref()?;
        if run.run.id != a.revision.run {
            return None;
        }
        let keys: Vec<_> = self
            .result_indices()
            .into_iter()
            .map(|i| &run.run.targets[i].key)
            .collect();
        let start = keys
            .iter()
            .position(|k| Some(*k) == self.selected_key.as_ref())
            .map_or(0, |i| i + 1);
        (0..keys.len())
            .map(|offset| keys[(start + offset) % keys.len()])
            .find(|key| {
                Some(*key) != self.selected_key.as_ref()
                    && a.revision.results.iter().any(|r| {
                        &r.key == *key
                            && r.eligible_pixels > 0
                            && matches!(
                                r.classification,
                                Classification::Changed | Classification::NoSignificantChange
                            )
                    })
                    && current_decision(&self.evidence.reviews, a.revision.id, key)
                        .is_none_or(|d| d.state == ReviewState::Unreviewed)
            })
            .cloned()
    }
    pub(super) fn load_mask_fields(&mut self, cx: &mut Context<Self>) {
        if let Some(r) = self
            .mask_selected
            .and_then(|i| self.mask_rectangles.get(i))
            .copied()
        {
            for (i, value) in [r.x, r.y, r.width, r.height].into_iter().enumerate() {
                self.analysis_fields[10 + i]
                    .update(cx, |input, cx| input.set_value(&value.to_string(), cx));
            }
        }
    }
    pub(super) fn review_ux_action(&mut self, index: usize, cx: &mut Context<Self>) {
        self.sync_review_note(cx);
        if !self.control_enabled(Control::Phase5(index), cx) {
            return;
        }
        match index {
            32 => {
                let state = self
                    .review_identity()
                    .and_then(|(analysis, key)| {
                        current_decision(&self.evidence.reviews, analysis, &key).map(|d| d.state)
                    })
                    .unwrap_or(ReviewState::Unreviewed);
                self.phase5_action(
                    match state {
                        ReviewState::Unreviewed => 20,
                        ReviewState::Intentional => 21,
                        ReviewState::NeedsFix => 22,
                    },
                    cx,
                );
            }
            33 => {
                // Drafts must be saved explicitly before leaving their capture pair.
                if !navigation_preserves_drafts(
                    self.mask_dirty,
                    self.mask_geometry_dirty(cx),
                    self.review_note_dirty(cx),
                ) {
                    return;
                }
                if let Some(key) = self.next_unreviewed_key() {
                    self.selected_key = Some(key);
                    self.mask_selected = None;
                    self.pan_x = 0.;
                    self.pan_y = 0;
                    self.invalidate_approval();
                    self.clear_viewer();
                    self.sync_review_note(cx);
                    self.reload_viewer(cx);
                }
            }
            34 => {
                if let Some(request) = self.viewer.as_ref().and_then(|v| v.request.as_ref())
                    && let Some(r) = seed(request.before.dimensions, request.after.dimensions)
                {
                    self.mask_rectangles.push(r);
                    self.mask_selected = Some(self.mask_rectangles.len() - 1);
                    self.mask_dirty = true;
                    self.mask_draw = false;
                    self.load_mask_fields(cx);
                    self.presentation_focus = Some(Control::AnalysisField(10));
                }
            }
            35 => {
                if self.analysis_fields[10..14]
                    .iter()
                    .any(|f| f.read(cx).composing())
                {
                    return;
                }
                let values: Vec<_> = self.analysis_fields[10..14]
                    .iter()
                    .map(|f| f.read(cx).content.to_string())
                    .collect();
                if let (Some(request), Some(i)) = (
                    self.viewer.as_ref().and_then(|v| v.request.as_ref()),
                    self.mask_selected,
                ) {
                    match geometry(
                        [&values[0], &values[1], &values[2], &values[3]],
                        request.before.dimensions,
                        request.after.dimensions,
                    ) {
                        Ok(r) => {
                            self.mask_rectangles[i] = r;
                            self.mask_dirty = true;
                        }
                        Err(message) => self.error = Some(message.into()),
                    }
                }
            }
            36 => {
                self.capture_details_open = !self.capture_details_open;
                self.capture_diagnostics_open = false;
                if self.capture_details_open {
                    self.review_note_open = false;
                    self.review_tools = false;
                    self.mask_draw = false;
                }
            }
            38 => self.capture_diagnostics_open = !self.capture_diagnostics_open,
            39 => {
                self.presentation_focus = Some(if self.capture_details_open {
                    Control::Phase5(36)
                } else if self.review_note_open {
                    Control::Phase5(37)
                } else {
                    Control::Guided(44)
                });
                self.capture_details_open = false;
                self.review_note_open = false;
                self.review_tools = false;
                self.mask_draw = false;
                self.mask_drag = None;
            }
            37 => {
                self.review_note_open = !self.review_note_open;
                if self.review_note_open {
                    self.presentation_focus = Some(Control::AnalysisField(9));
                    self.review_tools = false;
                    self.capture_details_open = false;
                    self.mask_draw = false;
                    self.mask_drag = None;
                }
            }
            _ => (),
        }
        cx.notify();
    }
    pub(super) fn ordinary_review_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.viewer_inspector_open() || self.options_panel_open() {
            return div().text_size(px(13.)).text_color(rgb(MUTED)).child("Temporary editor · return to evidence before making a review or baseline decision.").into_any_element();
        }
        if self.history_inspection.is_some() {
            return div()
                .text_size(px(13.))
                .text_color(rgb(MUTED))
                .child(format!(
                    "Recorded decision: {} · Inspection only; baseline unchanged.",
                    self.ordinary_review_status()
                ))
                .into_any_element();
        }
        let state = self.review_identity().and_then(|(analysis, key)| {
            current_decision(&self.evidence.reviews, analysis, &key).map(|d| d.state)
        });
        let label = match state {
            Some(ReviewState::Intentional) => "Intentional",
            Some(ReviewState::NeedsFix) => "Needs fix",
            _ => "Unreviewed",
        };
        let mut actions = div().flex().flex_wrap().gap_2();
        for (index, text) in [
            (
                21,
                if self.narrow_workspace {
                    "Intentional"
                } else {
                    "Mark intentional"
                },
            ),
            (
                22,
                if self.narrow_workspace {
                    "Needs fix"
                } else {
                    "Mark needs fix"
                },
            ),
            (
                20,
                if self.narrow_workspace {
                    "Unreviewed"
                } else {
                    "Mark unreviewed"
                },
            ),
        ] {
            actions = actions.child(self.button(
                Control::Phase5(index),
                text,
                ActionStyle::Choice {
                    selected: matches!(
                        (index, state),
                        (21, Some(ReviewState::Intentional))
                            | (22, Some(ReviewState::NeedsFix))
                            | (20, None | Some(ReviewState::Unreviewed))
                    ),
                },
                cx,
            ));
        }
        div()
            .debug_selector(|| "ordinary-review-decisions".into())
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_1()
            .child(
                div()
                    .text_size(px(13.))
                    .line_height(px(18.))
                    .child(if self.narrow_workspace {
                        format!("{label} · {}", self.review_progress_summary())
                    } else {
                        format!("Current review: {label} · displayed analysis only")
                    }),
            )
            .when(!self.narrow_workspace, |bar| {
                bar.child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(MUTED))
                        .child(self.review_progress_summary()),
                )
            })
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .child(actions)
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.button(
                                Control::Phase5(37),
                                "Review note",
                                ActionStyle::Secondary,
                                cx,
                            ))
                            .child(self.button(
                                Control::Phase5(33),
                                "Next unreviewed",
                                ActionStyle::Primary,
                                cx,
                            )),
                    ),
            )
            .when_some(self.review_blocking_reason(cx), |bar, reason| {
                bar.child(
                    div()
                        .debug_selector(|| "review-blocking-reason".into())
                        .text_size(px(13.))
                        .line_height(px(18.))
                        .text_color(rgb(MUTED))
                        .child(reason),
                )
            })
            .into_any_element()
    }
    /// Explanatory only: the authoritative action/draft guards remain separate.
    pub(super) fn review_blocking_reason(&self, cx: &App) -> Option<&'static str> {
        if self.mask_geometry_dirty(cx) {
            Some("Apply your coordinates, then save ignored areas in Analysis.")
        } else if self.mask_dirty {
            Some("Save ignored areas in Analysis before reviewing or moving on.")
        } else if self.review_note_dirty(cx) {
            Some("Save your draft in Review note before Next unreviewed.")
        } else if self.analysis_pending.is_some() || self.analysis_due.is_some() {
            Some("Analysis is pending. Wait for results before review decisions.")
        } else if !self.review_result_valid() {
            Some("This evidence is not comparable. Open Details for the cause.")
        } else if self.next_unreviewed_key().is_none() {
            Some("No other matching unreviewed result. Change filters or choose a result.")
        } else {
            None
        }
    }
    pub(super) fn keyboard_mask_editor(&self, cx: &mut Context<Self>) -> AnyElement {
        let editor = div().flex().flex_col().gap_2().child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .child(self.button(
                    Control::Phase5(14),
                    "Draw ignored area",
                    ActionStyle::Secondary,
                    cx,
                ))
                .child(self.button(
                    Control::Phase5(34),
                    "Add by coordinates",
                    ActionStyle::Secondary,
                    cx,
                )),
        );
        if self.mask_selected.is_none() {
            return editor.child(div().text_size(px(13.)).line_height(px(18.)).text_color(rgb(MUTED))
                .child("Ignore a changing banner or timestamp. Draw an area on the screenshot, or enter its coordinates."))
                .when(self.mask_dirty, |editor| editor.child(self.button(Control::Phase5(18), "Save ignored areas", ActionStyle::Secondary, cx)))
                .into_any_element();
        }
        let mut fields = div().flex().flex_wrap().gap_2();
        for (i, label) in [
            "X (pixels)",
            "Y (pixels)",
            "Width (pixels)",
            "Height (pixels)",
        ]
        .into_iter()
        .enumerate()
        {
            fields = fields.child(
                div().w(px(128.)).max_w_full().child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .flex_shrink_0()
                        .child(div().text_size(px(13.)).line_height(px(18.)).child(label))
                        .child(self.analysis_fields[10 + i].clone())
                        .id(("ignored-area-field", i))
                        .debug_selector(move || format!("ignored-area-field-{i}"))
                        .anchor_scroll(self.review_anchor(Control::AnalysisField(10 + i))),
                ),
            );
        }
        editor.child(fields)
            .child(div().flex().flex_wrap().gap_2()
                .child(self.button(Control::Phase5(35), "Apply area edit", ActionStyle::Secondary, cx))
                .child(self.button(Control::Phase5(18), "Save ignored areas", ActionStyle::Secondary, cx)))
            .child(div().text_size(px(13.)).line_height(px(18.)).text_color(rgb(MUTED)).child("Apply edits the draft. Save validates coverage and binds areas to this capture pair."))
            .into_any_element()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn seed_is_bounded_and_not_all_image() {
        for (w, h) in [(1280, 800), (2, 1), (1, 2)] {
            let d = Dimensions {
                width: w,
                height: h,
            };
            let r = seed(d, d).unwrap();
            assert!(fits(r, d));
            assert!(u64::from(r.width) * u64::from(r.height) < u64::from(w) * u64::from(h));
        }
        assert!(
            seed(
                Dimensions {
                    width: 1,
                    height: 1
                },
                Dimensions {
                    width: 1,
                    height: 1
                }
            )
            .is_none()
        );
    }
    #[test]
    fn geometry_rejects_invalid_integer_zero_and_overflow_without_rounding() {
        let d = Dimensions {
            width: 100,
            height: 80,
        };
        for values in [
            ["-1", "0", "1", "1"],
            ["1.5", "0", "1", "1"],
            ["0", "0", "0", "1"],
            ["99", "0", "2", "1"],
            ["4294967295", "0", "2", "1"],
        ] {
            assert!(geometry(values, d, d).is_err());
        }
        assert_eq!(
            geometry(["7", "9", "12", "14"], d, d).unwrap(),
            Rectangle {
                x: 7,
                y: 9,
                width: 12,
                height: 14
            }
        );
    }
    #[test]
    fn disposition_is_bound_to_exact_target_and_analysis_not_historical_review() {
        use regressee_core::domain::{PageId, ProfileId};
        let key = TargetKey {
            page: PageId::new(),
            profile: ProfileId::new(),
        };
        let old = AnalysisId::new();
        let current = AnalysisId::new();
        let mut reviews = vec![ReviewDecision {
            analysis: old,
            key: key.clone(),
            state: ReviewState::NeedsFix,
            note: "Old decision".into(),
            reviewed_at: 1,
        }];
        assert!(current_decision(&reviews, current, &key).is_none());
        reviews.push(ReviewDecision {
            analysis: current,
            key: key.clone(),
            state: ReviewState::Intentional,
            note: "Current decision".into(),
            reviewed_at: 2,
        });
        assert_eq!(
            current_decision(&reviews, current, &key).unwrap().state,
            ReviewState::Intentional
        );
        let other = TargetKey {
            page: PageId::new(),
            profile: key.profile,
        };
        assert!(current_decision(&reviews, current, &other).is_none());
        reviews.insert(
            0,
            ReviewDecision {
                analysis: current,
                key: key.clone(),
                state: ReviewState::Unreviewed,
                note: "Reset".into(),
                reviewed_at: 2,
            },
        );
        assert_eq!(
            current_decision(&reviews, current, &key).unwrap().state,
            ReviewState::Unreviewed
        );
    }
    #[test]
    fn saving_checks_exact_union_coverage_including_overlapping_areas() {
        let d = Dimensions {
            width: 100,
            height: 80,
        };
        let left = Rectangle {
            x: 0,
            y: 0,
            width: 60,
            height: 80,
        };
        let right = Rectangle {
            x: 40,
            y: 0,
            width: 60,
            height: 80,
        };
        assert!(all_masked(&[left, right], d, d));
        assert!(!all_masked(&[left], d, d));
        assert!(!all_masked(
            &[
                left,
                Rectangle {
                    height: 79,
                    ..right
                }
            ],
            d,
            d
        ));
        let taller = Dimensions {
            width: 100,
            height: 81,
        };
        assert!(!all_masked(&[left, right], d, taller));
        assert!(!all_masked(&[], d, d));
    }
    #[test]
    fn next_refuses_unsaved_drafts_and_never_mutates_them() {
        let areas = vec![Rectangle {
            x: 7,
            y: 9,
            width: 12,
            height: 14,
        }];
        let note = "Investigate the header".to_owned();
        let snapshot = (areas.clone(), note.clone());
        for state in [
            (true, false, false),
            (false, true, false),
            (false, false, true),
            (true, true, true),
        ] {
            assert!(!navigation_preserves_drafts(state.0, state.1, state.2));
            assert_eq!((&areas, &note), (&snapshot.0, &snapshot.1));
        }
        assert!(navigation_preserves_drafts(false, false, false));
    }
}
