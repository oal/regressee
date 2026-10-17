// SPDX-License-Identifier: MPL-2.0
//! A viewport-space reveal over the same source-space originals, never a blend.
use super::*;

impl Shell {
    pub(super) fn split_view_available(&self) -> bool {
        self.preview.is_none()
            && self.evidence.comparison.is_some()
            && !self.viewer_read_failed
            && self.viewer.as_ref().is_some_and(|view| {
                (0..2).all(|source| {
                    !view.panes[source].is_empty()
                        // A shorter page ending is evidence, not a missing image.
                        || view.request.as_ref().is_some_and(|request| {
                            let height = if source == 0 {
                                request.before.dimensions.height
                            } else {
                                request.after.dimensions.height
                            };
                            view.tile as u64 * regressee_app::images::TILE_HEIGHT as u64
                                >= height as u64
                        })
                })
            })
    }

    pub(super) fn effective_viewer_mode(&self) -> usize {
        if self.viewer_mode == 5 && !self.split_view_available() {
            // Retain the preference, but do not conceal the available original
            // behind a reveal whose other screenshot could not be loaded.
            0
        } else {
            self.viewer_mode
        }
    }

    pub(super) fn split_input_allowed(&self) -> bool {
        self.viewer_mode == 5
            && self.split_view_available()
            && self.viewer_input_allowed()
            && self.task.is_none()
            && !self.viewer_mode_open
            && !self.options_panel_open()
            && !self.mask_draw
    }

    fn move_split(&mut self, pointer: Pixels, bounds: Bounds<Pixels>, grab: f32) {
        let width = f32::from(bounds.size.width);
        if width > 0. {
            self.split_fraction =
                ((f32::from(pointer - bounds.left()) - grab) / width).clamp(0., 1.);
        }
    }

    fn split_key(&mut self, event: &gpui::KeyDownEvent, cx: &mut Context<Self>) {
        if !self.split_input_allowed() {
            return;
        }
        let modifiers = event.keystroke.modifiers;
        if modifiers.control || modifiers.platform || modifiers.alt {
            return;
        }
        let step = if modifiers.shift { 0.1 } else { 0.01 };
        self.split_fraction = match event.keystroke.key.as_str() {
            "left" => (self.split_fraction - step).max(0.),
            "right" => (self.split_fraction + step).min(1.),
            "home" => 0.,
            "end" => 1.,
            _ => return,
        };
        cx.stop_propagation();
        cx.notify();
    }

    pub(super) fn split_caption(&self) -> AnyElement {
        div()
            .debug_selector(|| "split-caption".into())
            .w_full()
            .flex()
            .items_center()
            .justify_between()
            .font_weight(FontWeight::SEMIBOLD)
            .child("Before")
            .when(self.current_viewer_pane_width() >= 460., |caption| {
                caption.child(
                    div()
                        .font_weight(FontWeight::NORMAL)
                        .text_color(rgb(palette().muted))
                        .child("Drag divider to compare"),
                )
            })
            .child("After")
            .into_any_element()
    }

    pub(super) fn split_divider(&self, cx: &mut Context<Self>) -> AnyElement {
        let owner = cx.entity();
        let fraction = self.split_fraction;
        let enabled = self.control_enabled(Control::SplitDivider, cx);
        let focus = self.handle(Control::SplitDivider, cx);
        let paint_focus = focus.clone();
        div()
            .id("split-divider")
            .debug_selector(|| "split-divider".into())
            .absolute()
            .inset_0()
            .when(enabled, |divider| {
                divider.track_focus(&focus).on_key_down(
                    cx.listener(|s, event: &gpui::KeyDownEvent, _, cx| s.split_key(event, cx)),
                )
            })
            .child(
                canvas(
                    move |bounds, window, _| {
                        let x = bounds.left() + bounds.size.width * fraction;
                        // Keep the complete target grabbable at both endpoints.
                        let width = px(40.).min(bounds.size.width);
                        let left = (x - width / 2.).clamp(bounds.left(), bounds.right() - width);
                        let target = Bounds::new(
                            gpui::point(left, bounds.top()),
                            size(width, bounds.size.height),
                        );
                        let hitbox = enabled
                            .then(|| window.insert_hitbox(target, gpui::HitboxBehavior::Normal));
                        (x, target, hitbox)
                    },
                    move |bounds, (x, target, hitbox), window, _| {
                        let stroke = |x: Pixels, width: f32| {
                            Bounds::new(
                                gpui::point(x - px(width / 2.), bounds.top()),
                                size(px(width), bounds.size.height),
                            )
                        };
                        window.paint_quad(gpui::fill(stroke(x, 3.), rgb(palette().surface)));
                        window.paint_quad(gpui::fill(stroke(x, 1.), rgb(palette().ink)));
                        let Some(hitbox) = hitbox else { return };
                        window.set_cursor_style(gpui::CursorStyle::ResizeLeftRight, &hitbox);
                        let handle = Bounds::new(
                            gpui::point(target.left(), bounds.center().y - px(20.)),
                            size(target.size.width, px(40.)),
                        );
                        if paint_focus.is_focused(window) {
                            window.paint_quad(gpui::quad(
                                handle.dilate(px(5.)),
                                px(9.),
                                rgb(palette().surface),
                                px(3.),
                                rgb(palette().ink),
                                gpui::BorderStyle::Solid,
                            ));
                        }
                        window.paint_quad(gpui::quad(
                            handle,
                            px(4.),
                            rgb(if hitbox.is_hovered(window) {
                                palette().action_hover
                            } else {
                                palette().surface
                            }),
                            px(1.),
                            rgb(palette().ink),
                            gpui::BorderStyle::Solid,
                        ));
                        // Native-drawn paired arrows: a small, legible drag affordance.
                        let mut arrows = gpui::PathBuilder::stroke(px(1.5));
                        let center = handle.center();
                        for direction in [-1., 1.] {
                            arrows.move_to(center + gpui::point(px(4. * direction), px(-5.)));
                            arrows.line_to(center + gpui::point(px(9. * direction), px(0.)));
                            arrows.line_to(center + gpui::point(px(4. * direction), px(5.)));
                        }
                        if let Ok(path) = arrows.build() {
                            window.paint_path(path, rgb(palette().ink));
                        }

                        let down_owner = owner.clone();
                        window.on_mouse_event(move |e: &MouseDownEvent, phase, window, cx| {
                            if phase != gpui::DispatchPhase::Bubble
                                || e.button != MouseButton::Left
                                || !hitbox.is_hovered(window)
                            {
                                return;
                            }
                            down_owner.update(cx, |s, cx| {
                                if !s.split_input_allowed() {
                                    return;
                                }
                                s.pan_drag = None;
                                s.viewer_drag_pane = None;
                                s.split_drag = Some(f32::from(e.position.x - x));
                                window.focus(&s.handle(Control::SplitDivider, cx));
                                cx.stop_propagation();
                                cx.notify();
                            });
                        });
                        let move_owner = owner.clone();
                        window.on_mouse_event(move |e: &MouseMoveEvent, phase, _, cx| {
                            if phase != gpui::DispatchPhase::Capture {
                                return;
                            }
                            move_owner.update(cx, |s, cx| {
                                let Some(grab) = s.split_drag else { return };
                                if !s.split_input_allowed()
                                    || e.pressed_button != Some(MouseButton::Left)
                                {
                                    s.split_drag = None;
                                } else {
                                    s.move_split(e.position.x, bounds, grab);
                                }
                                cx.stop_propagation();
                                cx.notify();
                            });
                        });
                        let up_owner = owner.clone();
                        window.on_mouse_event(move |e: &MouseUpEvent, phase, _, cx| {
                            if phase != gpui::DispatchPhase::Capture
                                || e.button != MouseButton::Left
                            {
                                return;
                            }
                            up_owner.update(cx, |s, cx| {
                                let Some(grab) = s.split_drag.take() else {
                                    return;
                                };
                                if s.split_input_allowed() {
                                    s.move_split(e.position.x, bounds, grab);
                                }
                                cx.stop_propagation();
                                cx.notify();
                            });
                        });
                    },
                )
                .size_full(),
            )
            .into_any_element()
    }
}
