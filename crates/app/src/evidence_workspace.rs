// SPDX-License-Identifier: MPL-2.0
//! Inspection chrome has one height regardless of queue visibility or view mode.
use super::*;
use regressee_app::ui::{selector, timezone_select::chevron_sized};

const MODES: [(Control, &str); 6] = [
    (Control::Phase5(6), "Compare"),
    (Control::Phase5(7), "Difference"),
    (Control::Phase5(8), "Overlay"),
    (Control::Phase5(9), "Heatmap"),
    (Control::Guided(55), "Before/After (full width)"),
    (Control::SplitView, "Split slider"),
];

pub(super) struct InspectionHint(pub(super) SharedString);
impl Render for InspectionHint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .max_w(px(620.))
            .p_2()
            .bg(rgb(SURFACE))
            .border_1()
            .border_color(rgb(CONTROL_LINE))
            .text_size(px(13.))
            .text_color(rgb(INK))
            .child(self.0.clone())
    }
}

impl Shell {
    pub(super) fn inspection_chrome_mounted(&self) -> bool {
        // Selection metadata is already available while its pixels are read.
        // Keep the same heading and toolbar instead of briefly showing the
        // workspace title. Only the current request may hold this chrome open.
        !self.page_list_workspace()
            && !self.selected_capture_failed()
            && (self.viewer.is_some()
                || self
                    .viewer_pending
                    .as_ref()
                    .is_some_and(|(_, identity, _)| {
                        self.viewer_identity().as_ref() == Some(identity)
                    }))
    }

    pub(super) fn inspection_heading(&self) -> AnyElement {
        let (path, context) = self.inspection_identity();
        let displayed_path = format!(
            "{} · {path}",
            if self.history_inspection.is_some() {
                self.review_title()
            } else {
                "Inspect"
            }
        );
        let (_, after, profile) = self.pane_caption(1);
        let (_, before, _) = self.pane_caption(0);
        let environment = |label: &str, origin: &str| {
            format!(
                "{label}: {}",
                origin.lines().next().unwrap_or("environment not loaded")
            )
        };
        let part = |selector: &'static str, value: String| {
            let full = value.clone();
            div()
                .id(selector)
                .debug_selector(move || selector.into())
                .flex_1()
                .min_w_0()
                .text_ellipsis()
                .tooltip(move |_, cx| cx.new(|_| InspectionHint(full.clone().into())).into())
                .child(value)
        };
        div()
            .debug_selector(|| "inspected-target-context".into())
            .w_full()
            .min_w_0()
            .h(px(40.))
            .flex_shrink_0()
            .flex()
            .flex_col()
            .text_size(px(13.))
            .line_height(px(20.))
            .child(
                div()
                    .id("inspection-path")
                    .debug_selector(|| "inspected-target-path".into())
                    .text_ellipsis()
                    .text_size(px(17.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .tooltip({
                        let full = path.clone();
                        move |_, cx| cx.new(|_| InspectionHint(full.clone().into())).into()
                    })
                    .child(displayed_path),
            )
            .child(
                div()
                    .id("inspection-context")
                    .debug_selector(|| "inspected-target-profile-environments".into())
                    .flex()
                    .gap_2()
                    .text_color(rgb(MUTED))
                    .tooltip({
                        let full = context.clone();
                        move |_, cx| cx.new(|_| InspectionHint(full.clone().into())).into()
                    })
                    .child(part("inspection-profile", profile))
                    .when(self.evidence.comparison.is_some(), |row| {
                        row.child(part(
                            "inspection-before-environment",
                            environment("Before", &before),
                        ))
                    })
                    .child(part(
                        "inspection-after-environment",
                        environment(
                            if self.evidence.comparison.is_some() {
                                "After"
                            } else {
                                "Capture"
                            },
                            &after,
                        ),
                    )),
            )
            .into_any_element()
    }

    pub(super) fn evidence_toolbar(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut tools = div()
            .debug_selector(|| "viewer-toolbar".into())
            .w_full()
            .h(px(CONTROL_HEIGHT))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap_1();
        if self.control_mounted(Control::Guided(50)) {
            tools = tools.child(
                self.action_control(
                    Control::Guided(50),
                    if self.review_queue_collapsed {
                        "Show results"
                    } else {
                        "Hide results"
                    },
                    ActionStyle::Secondary,
                    cx,
                )
                .px(px(8.)),
            );
        }
        if self.control_mounted(Control::Guided(54)) {
            let label = match self.effective_viewer_mode() {
                1 => "Difference",
                2 => "Overlay",
                3 => "Heatmap",
                5 => "Split slider",
                4 => {
                    if self.full_width_after {
                        "After"
                    } else {
                        "Before"
                    }
                }
                _ => "Compare",
            };
            let entity = cx.entity();
            tools = tools.child(
                div()
                    .relative()
                    .flex_shrink_0()
                    .child(
                        self.action_control(
                            Control::Guided(54),
                            if self.narrow_workspace {
                                String::new()
                            } else {
                                format!("View: {label}")
                            },
                            ActionStyle::Secondary,
                            cx,
                        )
                        .px(px(8.))
                        .gap_1()
                        .when(self.narrow_workspace, |button| {
                            button.w(px(96.)).h(px(40.)).px(px(4.)).child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .text_size(px(13.))
                                    .line_height(px(16.))
                                    .child("View")
                                    .child(label),
                            )
                        })
                        .child(chevron_sized(self.viewer_mode_open, 10.)),
                    )
                    .child(
                        canvas(
                            move |bounds, window, cx| {
                                let popup = entity.update(cx, |s, cx| {
                                    s.viewer_mode_trigger.set(Some(bounds));
                                    s.viewer_mode_open.then(|| s.view_menu(bounds, window, cx))
                                });
                                popup.map(|mut popup| {
                                    popup.layout_as_root(
                                        window.viewport_size().map(gpui::AvailableSpace::Definite),
                                        window,
                                        cx,
                                    );
                                    popup.prepaint_at(gpui::point(px(0.), px(0.)), window, cx);
                                    popup
                                })
                            },
                            |_, popup, window, cx| {
                                if let Some(mut popup) = popup {
                                    popup.paint(window, cx);
                                }
                            },
                        )
                        .absolute()
                        .top_0()
                        .left_0()
                        .size_full(),
                    ),
            );
            if self.viewer_mode == 4 {
                tools = tools.child(
                    self.action_control(
                        Control::Guided(56),
                        if self.narrow_workspace {
                            ""
                        } else if self.full_width_after {
                            "Show Before"
                        } else {
                            "Show After"
                        },
                        ActionStyle::Secondary,
                        cx,
                    )
                    .px(px(8.))
                    .when(self.narrow_workspace, |button| {
                        button.w(px(64.)).h(px(40.)).px(px(4.)).child(
                            div()
                                .flex()
                                .flex_col()
                                .text_size(px(13.))
                                .line_height(px(16.))
                                .child("Show")
                                .child(if self.full_width_after {
                                    "Before"
                                } else {
                                    "After"
                                }),
                        )
                    }),
                );
            }
        }
        if self.viewer_has_pixels() || self.viewer_pending.is_some() {
            // Retain its place during loading; the control stays disabled until
            // the selected image has pixels to navigate.
            tools = tools.child(self.zoom_select.clone());
        }
        if self.control_mounted(Control::ImageControls) {
            tools = tools.child(
                self.action_control(
                    Control::ImageControls,
                    "Image controls",
                    ActionStyle::Choice {
                        selected: self.image_controls_open,
                    },
                    cx,
                )
                .px(px(8.)),
            );
        }
        // Drawing has one explicit exit in place of secondary disclosures, not
        // an appended action that overflows the navigation-open narrow row.
        if !self.mask_draw {
            tools = tools.child(
                self.action_control(
                    Control::Phase5(36),
                    "Details",
                    ActionStyle::Choice {
                        selected: self.capture_details_open,
                    },
                    cx,
                )
                .px(px(8.)),
            );
            if self.evidence.comparison.is_some() && self.preview.is_none() {
                tools = tools.child(
                    self.action_control(
                        Control::Guided(44),
                        "Analysis",
                        ActionStyle::Choice {
                            selected: self.review_tools,
                        },
                        cx,
                    )
                    .px(px(8.)),
                );
            }
        }
        if self.mask_draw {
            tools = tools.child(
                self.action_control(
                    Control::Phase5(14),
                    "Done drawing",
                    ActionStyle::Secondary,
                    cx,
                )
                .px(px(8.)),
            );
        }
        tools.into_any_element()
    }

    fn view_menu(
        &self,
        trigger: Bounds<Pixels>,
        window: &Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let bounds = selector::popup_bounds(trigger, window.viewport_size(), MODES.len(), 304.);
        let mut menu = div()
            .id("viewer-mode-chooser")
            .debug_selector(|| "viewer-mode-chooser".into())
            .w(bounds.size.width)
            .h(bounds.size.height)
            .overflow_hidden()
            .occlude()
            .border_1()
            .border_color(rgb(CONTROL_LINE))
            .rounded(px(4.))
            .bg(rgb(SURFACE))
            .flex()
            .flex_col()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(cx.listener(|s, event: &MouseDownEvent, _, cx| {
                if !s
                    .viewer_mode_trigger
                    .get()
                    .is_some_and(|b| b.contains(&event.position))
                {
                    s.viewer_mode_open = false;
                    cx.notify();
                }
            }))
            .on_key_down(cx.listener(|s, event: &gpui::KeyDownEvent, window, cx| {
                let current = MODES
                    .iter()
                    .position(|(c, _)| s.handle(*c, cx).is_focused(window))
                    .unwrap_or(s.viewer_mode);
                let next = match event.keystroke.key.as_str() {
                    "down" | "right" => (current + 1) % MODES.len(),
                    "up" | "left" => (current + MODES.len() - 1) % MODES.len(),
                    "home" => 0,
                    "end" => MODES.len() - 1,
                    _ => return,
                };
                let reverse = matches!(event.keystroke.key.as_str(), "up" | "left" | "end");
                let next = (0..MODES.len())
                    .map(|offset| {
                        if reverse {
                            (next + MODES.len() - offset) % MODES.len()
                        } else {
                            (next + offset) % MODES.len()
                        }
                    })
                    .find(|&i| s.control_enabled(MODES[i].0, cx));
                if let Some(next) = next {
                    window.focus(&s.handle(MODES[next].0, cx));
                }
                cx.stop_propagation();
            }));
        for (i, (control, label)) in MODES.into_iter().enumerate() {
            let selected = self.effective_viewer_mode() == i;
            let label = if control == Control::SplitView && !self.split_view_available() {
                "Split slider (needs both images)"
            } else {
                label
            };
            menu = menu.child(
                self.action_control(control, label, ActionStyle::Choice { selected }, cx)
                    .w_full()
                    .h(px(CONTROL_HEIGHT))
                    .rounded(px(0.))
                    .border_0()
                    .justify_between()
                    .child(if selected { "Current" } else { "" }),
            );
        }
        gpui::deferred(gpui::anchored().position(bounds.origin).child(menu))
            .with_priority(2)
            .into_any_element()
    }
}

pub(super) fn baseline_selection_label(selected: usize, ready: usize) -> String {
    if selected == 0 {
        "0 selected".into()
    } else {
        format!("{selected} selected · {ready} ready to update")
    }
}

pub(super) fn image_zoom_gesture() -> &'static str {
    if cfg!(target_os = "macos") {
        "⌘ + scroll"
    } else {
        "Ctrl + scroll"
    }
}

pub(super) fn image_navigation_hint() -> AnyElement {
    div()
        .debug_selector(|| "image-navigation-hint".into())
        .flex_shrink_0()
        .whitespace_nowrap()
        .text_size(px(HELP_TEXT_SIZE))
        .line_height(px(HELP_LINE_HEIGHT))
        .text_color(rgb(MUTED))
        .child(format!("Drag to pan · {} to zoom", image_zoom_gesture()))
        .into_any_element()
}

pub(super) fn image_controls_help(split: bool) -> AnyElement {
    let rows = |items: Vec<(&'static str, &'static str)>| {
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .children(items.into_iter().map(|(action, gesture)| {
                div()
                    .debug_selector(move || format!("image-help-{action}-{gesture}"))
                    .w_full()
                    .flex()
                    .gap_3()
                    .child(
                        div()
                            .w(px(124.))
                            .flex_shrink_0()
                            .text_color(rgb(MUTED))
                            .child(action),
                    )
                    .child(div().flex_1().min_w_0().child(gesture))
            }))
    };
    div()
        .debug_selector(|| "image-controls-help".into())
        .w_full()
        .flex()
        .flex_col()
        .gap_4()
        .text_size(px(HELP_TEXT_SIZE))
        .line_height(px(HELP_LINE_HEIGHT))
        .child(rows(vec![
            ("Move around", "Drag or scroll"),
            ("Move horizontally", "Shift + scroll"),
            ("Zoom at pointer", image_zoom_gesture()),
        ]))
        .when(split, |help| help.child(rows(vec![
            ("Compare versions", "Drag the divider left or right"),
            ("Divider keyboard", "Tab to the divider; Left / Right adjust, Shift moves faster, Home / End show one version"),
        ])))
        .child(
            div()
                .flex()
                .flex_col()
                .gap_2()
                .border_t_1()
                .border_color(rgb(LINE))
                .pt_4()
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("With the image focused"),
                )
                .child(
                    div()
                        .text_color(rgb(MUTED))
                        .child("Close help, then click the image or Tab to it first."),
                )
                .child(rows(vec![
                    ("Move around", "Arrow keys"),
                    ("Move a screenful", "Page Up / Page Down"),
                    ("Zoom in / out", "+ / −"),
                    ("Fit width", "0"),
                    ("Actual size", "1"),
                ])),
        )
        .into_any_element()
}

pub(super) fn heatmap_legend() -> AnyElement {
    div()
        .debug_selector(|| "heatmap-legend".into())
        .flex()
        .flex_col()
        .text_size(px(13.))
        .line_height(px(18.))
        .child("Heatmap · derived, not defects")
        .child(
            div()
                .flex()
                .items_center()
                .gap_2()
                .child(div().size(px(10.)).bg(rgb(0xdc3246)))
                .child("Significant differences")
                .child(div().size(px(10.)).bg(rgb(0x5f7882)))
                .child("Ignored pixels"),
        )
        .into_any_element()
}
