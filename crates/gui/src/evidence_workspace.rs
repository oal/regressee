// SPDX-License-Identifier: MPL-2.0
//! Inspection chrome has one height regardless of queue visibility or view mode.
use super::*;
use regressee_gui::ui::{motion, selector, timezone_select::chevron_sized};

const MODES: [(Control, &str); 6] = [
    (Control::Phase5(6), "Compare"),
    (Control::Phase5(7), "Difference"),
    (Control::Phase5(8), "Overlay"),
    (Control::Phase5(9), "Heatmap"),
    (Control::Guided(55), "Single image (full width)"),
    (Control::SplitView, "Split slider"),
];

impl Shell {
    fn toolbar_viewer_mode(&self) -> usize {
        if self.viewer.is_none() && self.inspection_chrome_mounted() {
            // No new pixels yet is not evidence that Split slider is unavailable.
            self.viewer_mode
        } else {
            self.effective_viewer_mode()
        }
    }

    fn view_icon(&self) -> ToolIcon {
        match self.toolbar_viewer_mode() {
            1 => ToolIcon::Difference,
            2 => ToolIcon::Overlay,
            3 => ToolIcon::Heatmap,
            4 => ToolIcon::Image,
            5 => ToolIcon::Split,
            _ => ToolIcon::Compare,
        }
    }

    pub(super) fn inspection_chrome_mounted(&self) -> bool {
        if self.dom_active() {
            return self.selected_key.is_some() && self.evidence.latest.is_some();
        }
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
        let (path, _) = self.inspection_identity();
        let displayed_path = format!(
            "{} · {path}",
            if self.history_inspection.is_some() {
                self.review_title()
            } else {
                "Inspect"
            }
        );
        let (_, _, profile) = self.pane_caption(1);
        let (before, after) = self.inspection_sources();
        let profile_icon = if self
            .displayed_run()
            .and_then(|r| {
                r.run
                    .profiles
                    .iter()
                    .find(|p| Some(p.id) == self.selected_key.as_ref().map(|k| k.profile))
            })
            .is_some_and(|p| p.mobile)
        {
            ToolIcon::Phone
        } else {
            ToolIcon::Desktop
        };
        InspectionHeading {
            displayed_path,
            profile,
            before,
            after,
            profile_icon,
            comparison: self.has_kind_comparison(),
            compact: self.working_width < 720.,
        }
        .into_any_element()
    }

    /// Name what the images represent, not the generic environment category.
    /// Full recorded names and origins remain in Capture details. Never borrow
    /// source names from today's project settings to fill missing history.
    pub(super) fn inspection_sources(&self) -> (String, String) {
        let before = self.pane_caption(0).1;
        let after = self.pane_caption(1).1;
        let before_environment = before.lines().next().unwrap_or("environment not loaded");
        let after_environment = after.lines().next().unwrap_or("environment not loaded");
        let comparison = self.has_kind_comparison();
        let describe = |role: &str, environment: &str, show_environment: bool| {
            if role == "Source not loaded" {
                role.to_owned()
            } else if environment == "environment not loaded" {
                format!("{role} · Source not loaded")
            } else if show_environment && environment != "Website" {
                format!("{role} · {environment}")
            } else {
                role.to_owned()
            }
        };
        let after_role = if comparison {
            if self.history_inspection.is_some() {
                "Recorded check"
            } else {
                "This check"
            }
        } else if self.capture_is_saved_baseline() {
            "Saved baseline"
        } else {
            match self.displayed_run().map(|r| &r.run.intent) {
                Some(RunIntent::RepresentativePreview) => "Screenshot preview",
                Some(RunIntent::Check { .. }) => "Check · not compared",
                Some(RunIntent::NewBaseline)
                    if self.history_inspection.is_none() && self.focus.candidate =>
                {
                    "Not yet a baseline"
                }
                Some(RunIntent::NewBaseline) => "Recorded capture",
                None => "Source not loaded",
            }
        };
        (
            describe(
                "Saved baseline",
                before_environment,
                before_environment != after_environment,
            ),
            // A meaningful shared name still identifies the source; show it
            // once rather than repeating it or dropping it altogether.
            describe(after_role, after_environment, true),
        )
    }
}

#[derive(IntoElement)]
struct InspectionHeading {
    displayed_path: String,
    profile: String,
    before: String,
    after: String,
    profile_icon: ToolIcon,
    comparison: bool,
    compact: bool,
}

impl RenderOnce for InspectionHeading {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        let Self {
            displayed_path,
            profile,
            before,
            after,
            profile_icon,
            comparison,
            compact,
        } = self;
        // Ellipsis-bearing flex children have no useful intrinsic width in
        // GPUI. Measure native text before capping the two related groups.
        let measure = |value: &str, size: f32, weight: FontWeight| {
            let style = gpui::TextStyle {
                font_family: FONT.into(),
                font_weight: weight,
                ..Default::default()
            };
            f32::from(
                window
                    .text_system()
                    .shape_line(
                        SharedString::from(value.to_owned()),
                        px(size),
                        &[style.to_run(value.len())],
                        None,
                    )
                    .width
                    .ceil(),
            )
        };
        let source = |selector: &'static str, role: &'static str, value: String| {
            div()
                .id(selector)
                .debug_selector(move || selector.into())
                .min_w_0()
                .h(px(20.))
                .flex()
                .items_center()
                .gap_2()
                .child(
                    div()
                        .w(px(48.))
                        .flex_shrink_0()
                        .text_color(rgb(palette().muted))
                        .child(role),
                )
                .child(
                    div()
                        .debug_selector(move || format!("{selector}-value"))
                        .min_w_0()
                        .flex_1()
                        .text_ellipsis()
                        .text_color(rgb(palette().ink))
                        .child(value),
                )
        };
        let (name, dimensions) = profile
            .rsplit_once(" · ")
            .map_or((profile.as_str(), None), |(name, size)| (name, Some(size)));
        let source_width = (56.
            + measure(&after, 13., FontWeight::NORMAL).max(if comparison {
                measure(&before, 13., FontWeight::NORMAL)
            } else {
                0.
            }))
        .min(if compact { 180. } else { 320. });
        let identity_width = measure(&displayed_path, 17., FontWeight::SEMIBOLD)
            .max(measure(&profile, 13., FontWeight::NORMAL) + 28.)
            .min(440.);
        let identity_min =
            dimensions.map_or(80., |size| measure(size, 13., FontWeight::NORMAL) + 40.);
        div()
            .debug_selector(|| "inspected-target-context".into())
            .w_full()
            .min_w_0()
            .h(px(40.))
            .flex_shrink_0()
            .flex()
            .items_center()
            .gap_4()
            .text_size(px(13.))
            .line_height(px(20.))
            .child(
                div()
                    .debug_selector(|| "inspection-identity-group".into())
                    .w(px(identity_width))
                    .min_w(px(identity_min))
                    .flex()
                    .flex_col()
                    .child(
                        div()
                            .id("inspection-path")
                            .debug_selector(|| "inspected-target-path".into())
                            .text_ellipsis()
                            .text_size(px(17.))
                            .line_height(px(CONTROL_LINE_HEIGHT))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(displayed_path),
                    )
                    .child(
                        div()
                            .id("inspection-profile")
                            .debug_selector(|| "inspection-profile".into())
                            .min_w_0()
                            .h(px(20.))
                            .flex()
                            .items_center()
                            .gap_1()
                            .text_color(rgb(palette().muted))
                            .child(toolbar::icon(profile_icon, palette().muted, 16.))
                            .child(
                                div()
                                    .debug_selector(|| "inspection-profile-name".into())
                                    .min_w_0()
                                    .flex_1()
                                    .text_ellipsis()
                                    .child(name.to_owned()),
                            )
                            .when_some(dimensions, |row, size| {
                                row.child(
                                    div()
                                        .debug_selector(|| "inspection-profile-dimensions".into())
                                        .flex_shrink_0()
                                        .whitespace_nowrap()
                                        .child(format!("· {size}")),
                                )
                            }),
                    ),
            )
            .child(
                div()
                    .id("inspection-context")
                    .debug_selector(|| "inspected-target-profile-environments".into())
                    .min_w_0()
                    .w(px(source_width))
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .when(comparison, |row| {
                        row.child(source("inspection-before-environment", "Before", before))
                    })
                    .child(source(
                        "inspection-after-environment",
                        if comparison { "After" } else { "Capture" },
                        after,
                    )),
            )
            .into_any_element()
    }
}

impl Shell {
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
            tools = tools.child(self.tool_control(
                Control::Guided(50),
                ToolIcon::Results,
                self.inspection_tool_label("Results"),
                if self.review_queue_collapsed {
                    "Show results"
                } else {
                    "Hide results"
                },
                !self.review_queue_collapsed,
                cx,
            ));
        }
        tools = tools.child(toolbar::separator());
        if self.dom_active() && self.control_mounted(Control::DomSource(0)) {
            tools = tools.child(self.dom_view_choices(cx));
        }
        if self.control_mounted(Control::Guided(54)) {
            let label = match self.toolbar_viewer_mode() {
                1 => "Difference",
                2 => "Overlay",
                3 => "Heatmap",
                5 => "Split slider",
                4 if self.working_width < 720. => "Single",
                4 => "Single image",
                _ => "Compare",
            };
            let entity = cx.entity();
            tools = tools.child(
                div()
                    .relative()
                    .flex_shrink_0()
                    .child(
                        self.tool_control(
                            Control::Guided(54),
                            self.view_icon(),
                            Some(label),
                            format!("View mode: {label}"),
                            self.viewer_mode_open,
                            cx,
                        )
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
                let mut sides = controls::segmented_group("single-image-sides");
                for (control, label, selected) in [
                    (Control::Guided(57), "Before", !self.full_width_after),
                    (Control::Guided(56), "After", self.full_width_after),
                ] {
                    // Both destinations stay visible. Selecting the current side
                    // is deliberately idempotent, unlike the old swap action.
                    let choice =
                        self.action_control(control, label, ActionStyle::Segment { selected }, cx);
                    sides = sides.child(choice);
                }
                tools = tools.child(sides);
            }
        }
        if !self.dom_active() && (self.viewer_has_pixels() || self.viewer_pending.is_some()) {
            // Retain its place during loading; the control stays disabled until
            // the selected image has pixels to navigate.
            tools = tools.child(self.zoom_select.clone());
        }
        tools = tools
            .child(div().flex_1().min_w_0())
            .child(toolbar::separator());
        if self.control_mounted(Control::ImageControls) {
            tools = tools.child(self.tool_control(
                Control::ImageControls,
                ToolIcon::Hand,
                self.inspection_tool_label("Image controls"),
                "Image controls",
                self.image_controls_open,
                cx,
            ));
        }
        // Drawing has one explicit exit in place of secondary disclosures, not
        // an appended action that overflows the navigation-open narrow row.
        if !self.mask_draw {
            tools = tools.child(self.tool_control(
                Control::Phase5(36),
                ToolIcon::Details,
                self.inspection_tool_label("Details"),
                "Details",
                self.capture_details_open,
                cx,
            ));
            if !self.dom_active() && self.evidence.comparison.is_some() && self.preview.is_none() {
                tools = tools.child(self.tool_control(
                    Control::Guided(44),
                    ToolIcon::Adjustments,
                    self.inspection_tool_label("Analysis"),
                    "Analysis",
                    self.review_tools,
                    cx,
                ));
            }
        }
        if self.mask_draw {
            tools = tools.child(self.tool_control(
                Control::Phase5(14),
                ToolIcon::Check,
                Some("Done"),
                "Done drawing",
                true,
                cx,
            ));
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
            .shadow(regressee_gui::ui::depth::Elevation::Floating.shadows())
            .debug_selector(|| "viewer-mode-chooser".into())
            .w(bounds.size.width)
            .h(bounds.size.height)
            .overflow_hidden()
            .occlude()
            .border_1()
            .border_color(rgb(palette().control_line))
            .rounded(px(4.))
            .bg(rgb(palette().surface))
            .flex()
            .flex_col()
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(cx.listener(|s, event: &MouseDownEvent, _, cx| {
                if s.agent_locked() {
                    return;
                }
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
                if s.agent_locked() {
                    return;
                }
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
            let selected = self.toolbar_viewer_mode() == i;
            let label = if control == Control::SplitView && !self.split_view_available() {
                "Split slider (needs both images)"
            } else {
                label
            };
            let icon = [
                ToolIcon::Compare,
                ToolIcon::Difference,
                ToolIcon::Overlay,
                ToolIcon::Heatmap,
                ToolIcon::Image,
                ToolIcon::Split,
            ][i];
            menu = menu.child(
                self.tool_control(control, icon, Some(label), label, selected, cx)
                    .w_full()
                    .h(px(CONTROL_HEIGHT))
                    .rounded(px(0.))
                    .border_0()
                    .child(div().flex_1())
                    .when(selected, |row| {
                        row.child(toolbar::icon(ToolIcon::Check, palette().accent_hover, 16.))
                    }),
            );
        }
        // Mount identity is independent of selected mode and keyboard highlight.
        // Dismissal removes the menu immediately; no exiting controls are kept.
        let menu = motion::enter("view-menu-entrance", menu, motion::Entrance::Menu, cx);
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
        .text_color(rgb(palette().muted))
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
                            .text_color(rgb(palette().muted))
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
                .border_color(rgb(palette().line))
                .pt_4()
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("With the image focused"),
                )
                .child(
                    div()
                        .text_color(rgb(palette().muted))
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
