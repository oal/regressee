// SPDX-License-Identifier: MPL-2.0
//! Direct image navigation and task-focused, non-reflowing disclosures.
use super::*;

impl Shell {
    pub(super) fn choose_viewer_zoom(&mut self, choice: ZoomChoice, cx: &mut Context<Self>) {
        if !self.viewer_input_allowed() {
            return;
        }
        match choice {
            ZoomChoice::FitWidth => self.guided(46, cx),
            ZoomChoice::Percent(percent) => {
                let viewport = self
                    .viewer_bounds
                    .get()
                    .map(|b| (f32::from(b.size.width), f32::from(b.size.height)))
                    .unwrap_or((self.current_viewer_pane_width(), 240.));
                self.zoom_viewer_at(
                    percent as f32 / 100.,
                    (viewport.0 / 2., viewport.1 / 2.),
                    cx,
                );
            }
        }
    }

    pub(super) fn viewer_has_pixels(&self) -> bool {
        self.viewer.as_ref().is_some_and(|v| {
            self.viewer_pane_indices()
                .iter()
                .any(|&i| !v.panes[i].is_empty())
        })
    }

    pub(super) fn viewer_inspector_open(&self) -> bool {
        self.review_note_open || self.capture_details_open || (self.review_tools && !self.mask_draw)
    }

    fn viewer_input_allowed(&self) -> bool {
        self.focus.screen == Screen::Results
            && !self.focus.busy
            && !self.focus.dialog
            && !self.focus.approval_dialog
            && !self.native_prompt
            && !self.viewer_inspector_open()
            && self.viewer_has_pixels()
    }

    pub(super) fn clamp_viewer_pan(&mut self) {
        let (width, height) = self.viewer_dimensions();
        if self.viewer_bounds.get().is_none() {
            self.pan_x = self.pan_x.clamp(0., width.saturating_sub(1) as f32);
            self.pan_y = self.pan_y.min(height.saturating_sub(1));
            return;
        }
        let viewport = self
            .viewer_bounds
            .get()
            .map(|b| (f32::from(b.size.width), f32::from(b.size.height)))
            .unwrap_or((self.current_viewer_pane_width(), 240.));
        self.pan_x = self
            .pan_x
            .clamp(0., (width as f32 - viewport.0 / self.zoom).max(0.));
        self.pan_y =
            (self.pan_y as f32).clamp(0., (height as f32 - viewport.1 / self.zoom).max(0.)) as u32;
    }

    pub(super) fn refresh_viewer_navigation(&mut self, cx: &mut Context<Self>) {
        self.clamp_viewer_pan();
        let tile = self.pan_y as usize / regressee_app::images::TILE_HEIGHT as usize;
        if self.viewer.as_ref().is_some_and(|v| v.tile != tile) {
            self.reload_viewer(cx);
        }
        cx.notify();
    }

    /// Keep the image coordinate under the pointer stationary while zooming.
    pub(super) fn zoom_viewer_at(&mut self, zoom: f32, anchor: (f32, f32), cx: &mut Context<Self>) {
        if !self.viewer_input_allowed() || !zoom.is_finite() {
            return;
        }
        let zoom = zoom.clamp(0.05, 4.);
        let source = (
            self.pan_x + anchor.0 / self.zoom,
            self.pan_y as f32 + anchor.1 / self.zoom,
        );
        self.pan_x = (source.0 - anchor.0 / zoom).max(0.);
        self.pan_y = (source.1 - anchor.1 / zoom).max(0.) as u32;
        self.zoom = zoom;
        self.viewer_fit = false;
        self.refresh_viewer_navigation(cx);
    }

    pub(super) fn viewer_scroll(
        &mut self,
        event: &gpui::ScrollWheelEvent,
        origin: (f32, f32),
        cx: &mut Context<Self>,
    ) {
        if !self.viewer_input_allowed() || self.mask_drag.is_some() {
            return;
        }
        let delta = event.delta.pixel_delta(px(20.));
        if event.modifiers.control || event.modifiers.platform {
            let factor = (f32::from(delta.y) * 0.01).clamp(-0.5, 0.5).exp();
            self.zoom_viewer_at(
                self.zoom * factor,
                (
                    f32::from(event.position.x) - origin.0,
                    f32::from(event.position.y) - origin.1,
                ),
                cx,
            );
        } else {
            let (mut x, mut y) = (f32::from(delta.x), f32::from(delta.y));
            if event.modifiers.shift && x == 0. {
                x = y;
                y = 0.;
            }
            self.pan_x = (self.pan_x - x / self.zoom).max(0.);
            self.pan_y = (self.pan_y as f32 - y / self.zoom).max(0.) as u32;
            self.refresh_viewer_navigation(cx);
        }
        cx.stop_propagation();
    }

    pub(super) fn viewer_key(&mut self, event: &gpui::KeyDownEvent, cx: &mut Context<Self>) {
        if !self.viewer_input_allowed() || self.mask_drag.is_some() {
            return;
        }
        let modifiers = event.keystroke.modifiers;
        if modifiers.control || modifiers.platform || modifiers.alt {
            return;
        }
        let viewport = self
            .viewer_bounds
            .get()
            .map(|b| (f32::from(b.size.width), f32::from(b.size.height)))
            .unwrap_or((240., 240.));
        let step = 48. / self.zoom;
        match event.keystroke.key.as_str() {
            "+" | "=" => {
                self.zoom_viewer_at(self.zoom * 1.25, (viewport.0 / 2., viewport.1 / 2.), cx)
            }
            "-" => self.zoom_viewer_at(self.zoom / 1.25, (viewport.0 / 2., viewport.1 / 2.), cx),
            "0" => self.guided(46, cx),
            "1" => self.zoom_viewer_at(1., (viewport.0 / 2., viewport.1 / 2.), cx),
            "left" => self.pan_x = (self.pan_x - step).max(0.),
            "right" => self.pan_x += step,
            "up" => self.pan_y = (self.pan_y as f32 - step).max(0.) as u32,
            "down" => self.pan_y = self.pan_y.saturating_add(step as u32),
            "pageup" => {
                self.pan_y = self
                    .pan_y
                    .saturating_sub((viewport.1 / self.zoom * 0.9) as u32)
            }
            "pagedown" => {
                self.pan_y = self
                    .pan_y
                    .saturating_add((viewport.1 / self.zoom * 0.9) as u32)
            }
            _ => return,
        }
        self.refresh_viewer_navigation(cx);
        cx.stop_propagation();
    }

    /// A flyout over the viewer, never a replacement for the queue or a bottom strip.
    pub(super) fn viewer_inspector(&self, cx: &mut Context<Self>) -> AnyElement {
        let available = (self.working_width - self.review_queue_width() - 12.).max(1.);
        let details = self.capture_details_open;
        let fields = self.analysis_fields[9..14].to_vec();
        let scroll = self.review_details_scroll.clone();
        let overflow_scroll = scroll.clone();
        let owner = cx.entity().downgrade();
        let reveal = self.review_focus_reveal.clone();
        // A percentage-width Entity beneath an auto-width scrolling block can
        // resolve to its padding alone. Give the native field a finite content
        // measure, then stretch every intermediate flex owner to that measure.
        let mut body = div()
            .w(px((available.min(360.) - 34.).max(1.)))
            .min_w_0()
            .flex()
            .flex_col()
            .gap_4();
        if self.review_note_open {
            body = body.child(
                div()
                    .id("review-note-field")
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .anchor_scroll(self.review_anchor(Control::AnalysisField(9)))
                    .child(controls::field(
                        "Note",
                        "8,192 UTF-8 bytes maximum. Closing keeps unsaved text.",
                        self.analysis_fields[9].clone().into_any_element(),
                    )),
            );
        } else if details {
            for (label, value) in self.capture_summary() {
                body = body.child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(label))
                        .child(div().child(value)),
                );
            }
            body = body.child(self.button(
                Control::Phase5(38),
                if self.capture_diagnostics_open {
                    "Hide technical diagnostics"
                } else {
                    "Technical diagnostics"
                },
                ActionStyle::Secondary,
                cx,
            ));
            if self.capture_diagnostics_open {
                body = body.child(
                    div()
                        .debug_selector(|| "capture-provenance-details".into())
                        .border_t_1()
                        .border_color(rgb(LINE))
                        .pt_4()
                        .child(self.capture_provenance_details()),
                );
            }
        } else {
            body = body.child(self.selected_capture_details(cx));
        }
        div()
            .id("viewer-inspector")
            .debug_selector(|| "viewer-inspector".into())
            .absolute()
            // The inspection toolbar now belongs to the workspace above us.
            // Do not reserve its old height again inside the evidence flyout.
            .top_0()
            .bottom_0()
            .right_0()
            .w(px(available.min(360.)))
            .min_h_0()
            .flex()
            .flex_col()
            .bg(rgb(SURFACE))
            .border_1()
            .border_color(rgb(CONTROL_LINE))
            .occlude()
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .justify_between()
                    .gap_2()
                    .p_3()
                    .border_b_1()
                    .border_color(rgb(LINE))
                    .child(div().font_weight(FontWeight::SEMIBOLD).child(
                        if self.review_note_open {
                            "Review note"
                        } else if details {
                            "Capture details"
                        } else {
                            "Adjust comparison"
                        },
                    ))
                    .child(self.button(
                        Control::Phase5(39),
                        "Return to evidence",
                        ActionStyle::Secondary,
                        cx,
                    )),
            )
            .child(
                div()
                    .id("review-details-scroll")
                    .debug_selector(|| "review-details-scroll".into())
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .w_full()
                    .flex()
                    .flex_col()
                    .overflow_y_scroll()
                    .track_scroll(&self.review_details_scroll)
                    .p_4()
                    .child(body),
            )
            // Coordinate fields can mount only after Add/Draw. Reveal focus
            // using this frame's painted bounds, not absent/stale old anchors.
            .child(
                canvas(
                    |_, _, _| (),
                    move |_, _, window, cx| {
                        if reveal.get() == 0 {
                            return;
                        }
                        reveal.set(reveal.get() - 1);
                        for field in &fields {
                            let input = field.read(cx);
                            if !input.focus.is_focused(window) {
                                continue;
                            }
                            let Some(bounds) = input.rendered_bounds() else {
                                continue;
                            };
                            let viewport = scroll.bounds();
                            let top_inset = px(28.).min(
                                (viewport.size.height - bounds.size.height - px(8.)).max(px(0.)),
                            );
                            let delta = if bounds.bottom() > viewport.bottom() - px(8.) {
                                viewport.bottom() - px(8.) - bounds.bottom()
                            } else if bounds.top() < viewport.top() + top_inset {
                                viewport.top() + top_inset - bounds.top()
                            } else {
                                px(0.)
                            };
                            if delta.abs() > px(1.) {
                                let offset = scroll.offset();
                                scroll.set_offset(gpui::point(offset.x, offset.y + delta));
                                let owner = owner.clone();
                                let fields = fields.clone();
                                window.defer(cx, move |_, cx| {
                                    for field in &fields {
                                        cx.notify(field.entity_id());
                                    }
                                    if let Some(owner) = owner.upgrade() {
                                        owner.update(cx, |_, cx| cx.notify());
                                    }
                                });
                            }
                        }
                    },
                )
                .h(px(0.))
                .w_full(),
            )
            // Disclosure overflow is indicated only from actual painted geometry,
            // not inferred from the panel type or the window width.
            .child(
                canvas(
                    |_, _, _| (),
                    move |_, _, window, _| {
                        let b = overflow_scroll.bounds();
                        let max = overflow_scroll.max_offset().height;
                        if max <= px(0.) || b.size.height <= px(0.) {
                            return;
                        }
                        let h =
                            (b.size.height * (b.size.height / (b.size.height + max))).max(px(18.));
                        let y =
                            (b.size.height - h) * (-overflow_scroll.offset().y / max).clamp(0., 1.);
                        window.paint_quad(gpui::fill(
                            gpui::Bounds::new(
                                gpui::point(b.right() - px(5.), b.top() + y),
                                gpui::size(px(3.), h),
                            ),
                            rgb(CONTROL_LINE),
                        ));
                    },
                )
                .h(px(0.))
                .w_full(),
            )
            .when(self.review_note_open, |panel| {
                let (status, invalid) = self.note_status(cx);
                panel.child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .items_center()
                        .gap_3()
                        .p_3()
                        .border_t_1()
                        .border_color(rgb(LINE))
                        .child(self.button(
                            Control::Phase5(32),
                            "Save note",
                            ActionStyle::Primary,
                            cx,
                        ))
                        .child(
                            div()
                                .debug_selector(|| "review-note-status".into())
                                .flex_1()
                                .min_w_0()
                                .text_size(px(HELP_TEXT_SIZE))
                                .line_height(px(HELP_LINE_HEIGHT))
                                .text_color(rgb(if invalid { ERROR } else { MUTED }))
                                .child(status),
                        ),
                )
            })
            .into_any_element()
    }

    /// Useful facts only. Exact IDs, descriptors and debug structs are opt-in.
    pub(super) fn capture_summary(&self) -> Vec<(String, String)> {
        use regressee_core::domain::CaptureError;
        let mut rows = Vec::new();
        let Some(key) = &self.selected_key else {
            return rows;
        };
        let Some(run) = &self.evidence.latest else {
            return rows;
        };
        let Some(target) = run.run.targets.iter().find(|t| &t.key == key) else {
            return rows;
        };
        rows.push(("Page".into(), target.requested.to_string()));
        if let Some(profile) = run.run.profiles.iter().find(|p| p.id == key.profile) {
            rows.push((
                "Screen size".into(),
                format!("{} · {} × {}", profile.name, profile.width, profile.height),
            ));
        }
        let status = match target.outcome {
            TargetOutcome::Failed { code } => code.label(),
            TargetOutcome::Captured { .. } if self.evidence.comparison.is_some() => {
                "Captured for comparison with a saved baseline"
            }
            TargetOutcome::Captured { .. }
                if self
                    .evidence
                    .baseline
                    .iter()
                    .chain(self.evidence.baselines.iter())
                    .any(|b| {
                        b.references.iter().any(|r| {
                            &r.key == key
                                && r.run == run.run.id
                                && run
                                    .evidence(key)
                                    .is_some_and(|c| c.image.original == r.artifact)
                        })
                    }) =>
            {
                "Saved baseline screenshot"
            }
            TargetOutcome::Captured { .. } => "Captured · not saved as a baseline",
            _ => "No screenshot available yet",
        };
        rows.push(("Capture status".into(), status.into()));
        if let TargetOutcome::Failed { code } = target.outcome {
            let help = match code {
                CaptureError::ImageLimit => {
                    "This page is too large for a full-page screenshot. In Project settings, choose visible-screen-only capture, then find pages and capture again. Your saved baseline is unchanged."
                }
                CaptureError::NeedsLogin => {
                    "Log in again in Project settings before retrying. Your saved baseline is unchanged."
                }
                CaptureError::LowSpace => {
                    "Free up disk space before retrying. History lets you inspect stored captures before deleting anything."
                }
                _ => {
                    "Check that the page opens in your browser, then find pages and retry the capture. Your saved baseline is unchanged."
                }
            };
            rows.push(("What to do".into(), help.into()));
        }
        if matches!(target.outcome, TargetOutcome::Captured { .. })
            && let Some(capture) = run.evidence(key)
        {
            rows.push((
                "Screenshot".into(),
                format!("{} × {} pixels", capture.image.width, capture.image.height),
            ));
            if capture.metadata.final_url != capture.metadata.requested {
                rows.push(("Redirected to".into(), capture.metadata.final_url.clone()));
            }
            if !capture.metadata.warnings.is_empty() {
                rows.push((
                    "Capture cautions".into(),
                    capture
                        .metadata
                        .warnings
                        .iter()
                        .map(|w| capture_warning_label(w))
                        .collect::<Vec<_>>()
                        .join("; "),
                ));
            }
        }
        if let Some(finished) = run.run.finished_at {
            rows.push((
                "Capture run finished".into(),
                workspace_history::history_date(finished),
            ));
        }
        if let Some(comparison) = &self.evidence.comparison {
            let baseline = self
                .evidence
                .baseline
                .as_ref()
                .filter(|b| b.id == comparison.revision.baseline)
                .or_else(|| {
                    self.evidence
                        .baselines
                        .iter()
                        .find(|b| b.id == comparison.revision.baseline)
                });
            if let Some(reference) =
                baseline.and_then(|b| b.references.iter().find(|r| &r.key == key))
            {
                rows.push((
                    "Baseline screenshot captured".into(),
                    workspace_history::history_date(reference.captured_at),
                ));
            }
            rows.push(("Comparison".into(), "A visual difference needs review; it is not automatically a defect. Review decisions do not replace the saved baseline.".into()));
        } else {
            rows.push((
                "Comparison".into(),
                "This capture has not been compared with an earlier baseline.".into(),
            ));
        }
        if matches!(target.outcome, TargetOutcome::Failed { .. }) {
            rows.sort_by_key(|(label, _)| match label.as_str() {
                "Capture status" => 0,
                "What to do" => 1,
                "Page" => 2,
                "Screen size" => 3,
                _ => 4,
            });
        }
        rows
    }
}
