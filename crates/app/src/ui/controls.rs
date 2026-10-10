// SPDX-License-Identifier: MPL-2.0
use super::theme::*;
use gpui::{prelude::*, *};

#[derive(Clone, Copy)]
pub enum ActionStyle {
    Primary,
    Secondary,
    Danger,
    Navigation { selected: bool },
    Choice { selected: bool },
}

fn action_colors(style: ActionStyle) -> (u32, u32, u32, u32) {
    match style {
        ActionStyle::Primary => (ACCENT, SURFACE, ACCENT, ACCENT_HOVER),
        ActionStyle::Secondary => (ACTION_SURFACE, INK, CONTROL_LINE, ACTION_HOVER),
        ActionStyle::Danger => (ERROR_SURFACE, ERROR, ERROR, ERROR_SURFACE),
        ActionStyle::Navigation { selected } => (
            if selected { SELECTED } else { RAIL },
            if selected { ACCENT_HOVER } else { INK },
            if selected { ACCENT } else { RAIL },
            SELECTED,
        ),
        ActionStyle::Choice { selected } => (
            if selected { SELECTED } else { SURFACE },
            if selected { ACCENT_HOVER } else { INK },
            if selected { ACCENT } else { CONTROL_LINE },
            SELECTED,
        ),
    }
}

fn action_base(style: ActionStyle) -> Div {
    let (background, text, border, _) = action_colors(style);
    div()
        .px(px(12.))
        .py(px(8.))
        .min_h(px(CONTROL_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .when(!matches!(style, ActionStyle::Navigation { .. }), |el| {
            el.justify_center()
        })
        .border_1()
        .rounded(px(4.))
        .border_color(rgb(border))
        .bg(rgb(background))
        .text_color(rgb(text))
        .text_size(px(CONTROL_TEXT_SIZE))
        .line_height(px(CONTROL_LINE_HEIGHT))
        .font_weight(FontWeight::MEDIUM)
}

fn focus_paint(mut style: StyleRefinement) -> StyleRefinement {
    // GPUI 0.2.2 replaces Style.text wholesale, rather than recursively merging
    // it. Never set text_color (or any text property) in an interaction state.
    // This is a functional focus outline, not decorative elevation. Its paint
    // extends outside the existing 1px border without changing layout metrics.
    style = style.border_color(rgb(INK));
    style.box_shadow = Some(vec![BoxShadow {
        color: rgb(INK).into(),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(2.),
    }]);
    style
}

fn hover_paint(style: StyleRefinement, background: u32) -> StyleRefinement {
    // Hover is merged after focus in GPUI. Do not overwrite the focus outline.
    style.bg(rgb(background))
}

fn interaction(
    control: Stateful<Div>,
    focus: &FocusHandle,
    enabled: bool,
    hover_background: u32,
) -> Stateful<Div> {
    control
        .when(enabled, |el| {
            el.track_focus(focus)
                .cursor_pointer()
                .focus(focus_paint)
                .hover(move |s| hover_paint(s, hover_background))
                .active(move |s| hover_paint(s, hover_background))
        })
        .when(!enabled, |el| el.opacity(0.5))
}

/// Attach click/action handling at the call site; disabled controls have no
/// focus stop or hover affordance. Callers must also guard click/action activation.
pub fn action(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    focus: &FocusHandle,
    style: ActionStyle,
    enabled: bool,
) -> Stateful<Div> {
    let (_, _, _, hover_background) = action_colors(style);
    let id = id.into();
    let action_selector = id.clone();
    let label_selector = id.clone();
    interaction(action_base(style).id(id), focus, enabled, hover_background)
        // Actions fit their label; only navigation fills its column. Editable
        // fields and selection rows deliberately retain full-width geometry.
        .when(
            !matches!(style, ActionStyle::Navigation { .. }),
            |mut el| {
                el.style().align_self = Some(AlignSelf::FlexStart);
                el.max_w_full()
            },
        )
        .debug_selector(move || format!("action-{action_selector}"))
        // A block flex child with min-width zero collapses its intrinsic width
        // in stretched column layouts. A flex text wrapper measures its child
        // intrinsically and still permits long labels to wrap when constrained.
        .child(
            div()
                .flex()
                .min_w(px(0.))
                .debug_selector(move || format!("action-label-{label_selector}"))
                .child(label.into()),
        )
}

fn selection_base(selected: bool) -> Div {
    action_base(ActionStyle::Choice { selected })
        .p(px(CHOICE_INSET))
        .w_full()
        .justify_start()
        .items_start()
        .gap(px(12.))
}

fn indicator_base(selected: bool, radio: bool) -> Div {
    div()
        .size(px(18.))
        .flex_shrink_0()
        .flex()
        .items_center()
        .justify_center()
        .border_1()
        .rounded(px(if radio { 9. } else { 3. }))
        .border_color(rgb(if selected { ACCENT } else { CONTROL_LINE }))
        .bg(rgb(if selected && !radio { ACCENT } else { SURFACE }))
}

fn radio_dot() -> Div {
    div().size(px(8.)).rounded(px(4.)).bg(rgb(ACCENT))
}

fn check_points(bounds: Bounds<Pixels>) -> [Point<Pixels>; 3] {
    let at = |x: f32, y: f32| bounds.origin + point(bounds.size.width * x, bounds.size.height * y);
    [at(0.15, 0.5), at(0.4, 0.75), at(0.85, 0.25)]
}

fn check_path(bounds: Bounds<Pixels>) -> Result<Path<Pixels>, anyhow::Error> {
    let [start, corner, end] = check_points(bounds);
    let mut path = PathBuilder::stroke(px(2.));
    path.move_to(start);
    path.line_to(corner);
    path.line_to(end);
    path.build()
}

fn help_copy() -> Div {
    div()
        .text_size(px(HELP_TEXT_SIZE))
        .line_height(px(HELP_LINE_HEIGHT))
        .font_weight(FontWeight::NORMAL)
        .text_color(rgb(MUTED))
}

/// A single-choice row. Selection is the drawn radio dot; focus is an outline.
/// Callers own group exclusivity, focus order and guarded click/action handling.
pub fn choice(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    description: impl Into<SharedString>,
    focus: &FocusHandle,
    selected: bool,
    enabled: bool,
) -> Stateful<Div> {
    let description = description.into();
    let id = id.into();
    let row_selector = id.clone();
    let text_selector = id.clone();
    interaction(selection_base(selected).id(id), focus, enabled, SELECTED)
        .debug_selector(move || format!("choice-{row_selector}"))
        .child(
            indicator_base(selected, true)
                .mt(px(1.))
                .when(selected, |el| el.child(radio_dot())),
        )
        .child(
            div()
                .min_w(px(0.))
                .flex_1()
                .flex()
                .flex_col()
                .gap(px(4.))
                .debug_selector(move || format!("choice-copy-{text_selector}"))
                .child(div().child(label.into()))
                .when(!description.is_empty(), |el| {
                    el.child(
                        help_copy()
                            .text_color(rgb(if selected { ACCENT_HOVER } else { MUTED }))
                            .child(description),
                    )
                }),
        )
}

/// A checkbox row with an invariant label. Checked is a drawn tick, not text.
/// Callers own toggling and guarded click/action handling.
pub fn check(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    focus: &FocusHandle,
    checked: bool,
    enabled: bool,
) -> Stateful<Div> {
    let id = id.into();
    let row_selector = id.clone();
    let text_selector = id.clone();
    interaction(
        selection_base(checked).items_center().id(id),
        focus,
        enabled,
        SELECTED,
    )
    .debug_selector(move || format!("check-{row_selector}"))
    .child(indicator_base(checked, false).when(checked, |el| {
        el.child(
            canvas(
                |_, _, _| (),
                |bounds, (), window, _| {
                    if let Ok(path) = check_path(bounds) {
                        window.paint_path(path, rgb(SURFACE));
                    }
                },
            )
            .size(px(12.)),
        )
    }))
    .child(
        div()
            .flex()
            .min_w(px(0.))
            .flex_1()
            .debug_selector(move || format!("check-copy-{text_selector}"))
            .child(label.into()),
    )
}

/// Restrained form grouping: a rule and heading, not a nested card. Append the
/// section's controls with `.child` / `.children` at the call site.
pub fn section(title: &'static str, help: &'static str) -> Div {
    div()
        .flex()
        .flex_col()
        .gap(px(12.))
        .pt(px(24.))
        .border_t_1()
        .border_color(rgb(LINE))
        .child(
            div()
                .flex()
                .flex_col()
                .gap(px(4.))
                .child(
                    div()
                        .text_size(px(18.))
                        .line_height(px(24.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_color(rgb(INK))
                        .child(title),
                )
                .when(!help.is_empty(), |el| {
                    el.child(help_copy().max_w(px(660.)).child(help))
                }),
        )
}
#[derive(Clone, Copy)]
pub enum Tone {
    Neutral,
    Information,
    Warning,
    Error,
}
pub fn status(label: impl Into<SharedString>, tone: Tone) -> Div {
    let (background, ink) = match tone {
        Tone::Neutral => (RAIL, MUTED),
        Tone::Information => (SELECTED, ACCENT_HOVER),
        Tone::Warning => (WARNING_SURFACE, WARNING),
        Tone::Error => (ERROR_SURFACE, ERROR),
    };
    div()
        .px_2()
        .py_1()
        .rounded(px(3.))
        .bg(rgb(background))
        .text_color(rgb(ink))
        .text_size(px(13.))
        .child(label.into())
}
pub fn error(message: impl Into<SharedString>) -> Div {
    div()
        .p_4()
        .bg(rgb(ERROR_SURFACE))
        .text_color(rgb(ERROR))
        .border_1()
        .border_color(rgb(ERROR))
        .rounded(px(4.))
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Could not finish"),
        )
        .child(message.into())
}
/// The dismiss action belongs to the feedback surface, not a disconnected row.
/// Long diagnostic text scrolls inside a bounded message area so the active
/// screen and its recovery controls remain available at the minimum window size.
pub fn feedback(
    error: Option<SharedString>,
    notice: Option<SharedString>,
    dismiss: AnyElement,
) -> Div {
    let failed = error.is_some();
    let mut messages = div().flex().flex_col().gap(px(8.));
    if let Some(error) = error {
        messages = messages
            .child(
                div()
                    .flex_shrink_0()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Could not finish"),
            )
            .child(div().flex_shrink_0().child(error));
    }
    if let Some(notice) = notice {
        messages = messages.child(div().flex_shrink_0().child(notice));
    }
    let copy = div()
        .id("feedback-copy")
        .flex_1()
        .min_w_0()
        .max_h(px(160.))
        .overflow_y_scroll()
        .debug_selector(|| "feedback-copy".into())
        .child(messages);
    div()
        .p(px(16.))
        .flex()
        .items_start()
        .gap(px(16.))
        .flex_shrink_0()
        .bg(rgb(if failed { ERROR_SURFACE } else { SELECTED }))
        .text_color(rgb(if failed { ERROR } else { ACCENT_HOVER }))
        .text_size(px(14.))
        .line_height(px(20.))
        .border_1()
        .border_color(rgb(if failed { ERROR } else { LINE }))
        .rounded(px(4.))
        .debug_selector(|| "feedback-surface".into())
        .child(copy)
        .child(dismiss)
}
pub fn empty(title: &'static str, explanation: &'static str) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_3()
        .py_8()
        .max_w(px(660.))
        .child(
            div()
                .text_size(px(21.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(
            div()
                .text_color(rgb(MUTED))
                .text_size(px(15.))
                .child(explanation),
        )
}
pub fn field(label: &'static str, hint: &'static str, input: AnyElement) -> Div {
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().font_weight(FontWeight::MEDIUM).child(label))
        .child(input)
        .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(hint))
}
pub fn list_row(id: impl Into<ElementId>) -> Stateful<Div> {
    div()
        .id(id)
        .w_full()
        .py_4()
        .border_b_1()
        .border_color(rgb(LINE))
        .flex()
        .items_center()
        .gap_4()
}
/// No false percentage for unknown-length operations. Determinate values clamp
/// to their total; zero-total is unknown, never "100% completed".
pub fn progress(label: impl Into<SharedString>, completed: u32, total: Option<u32>) -> Div {
    let fraction = total
        .filter(|n| *n > 0)
        .map(|n| completed.min(n) as f32 / n as f32);
    div()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().text_color(rgb(MUTED)).child(label.into()))
        .when_some(fraction, |el, fraction| {
            el.child(
                div()
                    .h(px(4.))
                    .bg(rgb(LINE))
                    .child(div().h_full().w(relative(fraction)).bg(rgb(ACCENT))),
            )
        })
}
/// Protected information dialog only. Root shell traps/restores focus and Escape.
pub fn dialog(title: &'static str, body: &'static str, close: AnyElement) -> Div {
    div()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba(0x18313866))
        .child(
            div()
                .w(px(480.))
                .max_w_full()
                .p_6()
                .bg(rgb(SURFACE))
                .border_1()
                .border_color(rgb(CONTROL_LINE))
                .rounded(px(6.))
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .text_size(px(21.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title),
                )
                .child(div().text_color(rgb(MUTED)).child(body))
                .child(close),
        )
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    const ACTION_STYLES: [ActionStyle; 7] = [
        ActionStyle::Primary,
        ActionStyle::Secondary,
        ActionStyle::Danger,
        ActionStyle::Navigation { selected: false },
        ActionStyle::Navigation { selected: true },
        ActionStyle::Choice { selected: false },
        ActionStyle::Choice { selected: true },
    ];

    fn merged_base(mut element: Div) -> Style {
        Style::default().refined(element.style().clone())
    }

    fn inherited_text(style: &Style) -> TextStyle {
        TextStyle {
            font_size: px(15.).into(),
            font_weight: FontWeight::NORMAL,
            ..TextStyle::default()
        }
        .refined(style.text.clone())
    }

    fn assert_same_layout(before: &Style, after: &Style) {
        assert_eq!(before.display, after.display);
        assert_eq!(before.position, after.position);
        assert_eq!(before.inset, after.inset);
        assert_eq!(before.size, after.size);
        assert_eq!(before.min_size, after.min_size);
        assert_eq!(before.max_size, after.max_size);
        assert_eq!(before.margin, after.margin);
        assert_eq!(before.padding, after.padding);
        assert_eq!(before.border_widths, after.border_widths);
        assert_eq!(before.corner_radii, after.corner_radii);
        assert_eq!(before.gap, after.gap);
        assert_eq!(before.flex_direction, after.flex_direction);
        assert_eq!(before.flex_wrap, after.flex_wrap);
        assert_eq!(before.flex_basis, after.flex_basis);
        assert_eq!(before.flex_grow, after.flex_grow);
        assert_eq!(before.flex_shrink, after.flex_shrink);
        assert_eq!(before.align_items, after.align_items);
        assert_eq!(before.justify_content, after.justify_content);
    }

    #[test]
    fn pinned_gpui_old_color_only_focus_replaces_the_entire_text_refinement() {
        // Reproduce the previous implementation with GPUI's actual Refineable
        // impls, including inherited 15px/400 shell text, not a mock merge.
        let mut old = StyleRefinement::default()
            .text_size(px(14.))
            .font_weight(FontWeight::MEDIUM)
            .text_color(rgb(SURFACE));
        let resting = Style::default().refined(old.clone());
        let old_focus = StyleRefinement::default()
            .border_color(rgb(INK))
            .bg(rgb(SELECTED))
            .text_color(rgb(INK));
        old.refine(&old_focus);
        assert_eq!(old.text.as_ref().unwrap().font_size, None);
        assert_eq!(old.text.as_ref().unwrap().font_weight, None);

        let focused = resting.clone().refined(old_focus);
        let resting_text = inherited_text(&resting);
        let focused_text = inherited_text(&focused);
        assert_eq!(resting_text.font_size, px(14.).into());
        assert_eq!(resting_text.font_weight, FontWeight::MEDIUM);
        assert_eq!(focused_text.font_size, px(15.).into());
        assert_eq!(focused_text.font_weight, FontWeight::NORMAL);
        assert!(
            focused_text.line_height_in_pixels(px(16.))
                > resting_text.line_height_in_pixels(px(16.))
        );
    }

    #[test]
    fn all_action_state_combinations_preserve_merged_text_and_layout() {
        for variant in ACTION_STYLES {
            let resting = merged_base(action_base(variant));
            let (_, _, _, hover_background) = action_colors(variant);
            for states in 0..8 {
                let mut merged = resting.clone();
                // Match GPUI's actual order: base -> focus -> hover -> active.
                if states & 1 != 0 {
                    merged.refine(&focus_paint(StyleRefinement::default()));
                }
                if states & 2 != 0 {
                    merged.refine(&hover_paint(StyleRefinement::default(), hover_background));
                }
                if states & 4 != 0 {
                    merged.refine(&hover_paint(StyleRefinement::default(), hover_background));
                }
                assert_same_layout(&resting, &merged);
                assert_eq!(inherited_text(&resting), inherited_text(&merged));
                assert_eq!(
                    inherited_text(&merged).line_height_in_pixels(px(16.)),
                    px(20.)
                );
                if states & 1 != 0 {
                    assert_eq!(merged.border_color, Some(rgb(INK).into()));
                    assert_eq!(merged.box_shadow.len(), 1);
                    assert_eq!(merged.box_shadow[0].spread_radius, px(2.));
                }
                if matches!(variant, ActionStyle::Primary) {
                    assert_eq!(inherited_text(&merged).color, rgb(SURFACE).into());
                    let expected = if states & 6 != 0 {
                        ACCENT_HOVER
                    } else {
                        ACCENT
                    };
                    assert_eq!(
                        merged.background.as_ref().unwrap().color(),
                        Some(rgb(expected).into())
                    );
                }
            }
            let disabled = resting
                .clone()
                .refined(StyleRefinement::default().opacity(0.5));
            assert_same_layout(&resting, &disabled);
            assert_eq!(inherited_text(&resting), inherited_text(&disabled));
        }
    }

    #[test]
    fn selection_and_description_metrics_do_not_depend_on_selection_or_focus() {
        let unselected = merged_base(selection_base(false));
        let selected = merged_base(selection_base(true));
        assert_same_layout(&unselected, &selected);
        let description = merged_base(help_copy());
        for row in [unselected, selected] {
            let focused = row.clone().refined(focus_paint(StyleRefinement::default()));
            assert_same_layout(&row, &focused);
            let before = inherited_text(&row).refined(description.text.clone());
            let after = inherited_text(&focused).refined(description.text.clone());
            assert_eq!(before, after);
            assert_eq!(after.line_height_in_pixels(px(16.)), px(18.));
        }
        for radio in [true, false] {
            assert_same_layout(
                &merged_base(indicator_base(false, radio)),
                &merged_base(indicator_base(true, radio)),
            );
        }
    }

    #[test]
    fn actions_and_editable_fields_do_not_share_a_white_surface() {
        let secondary = merged_base(action_base(ActionStyle::Secondary));
        assert_eq!(
            secondary.background.as_ref().unwrap().color(),
            Some(rgb(ACTION_SURFACE).into())
        );
        assert_ne!(ACTION_SURFACE, SURFACE);
        let selection = merged_base(selection_base(false));
        assert_eq!(selection.padding.top, px(CHOICE_INSET).into());
        assert_eq!(selection.padding.bottom, px(CHOICE_INSET).into());
        assert_eq!(selection.flex_shrink, 0.);
    }

    #[test]
    fn drawn_check_geometry_is_bounded_and_translates_with_the_canvas() {
        let bounds = Bounds::new(point(px(10.), px(20.)), size(px(12.), px(12.)));
        let translated = Bounds::new(point(px(50.), px(70.)), bounds.size);
        let points = check_points(bounds);
        let moved = check_points(translated);
        for (point, moved) in points.into_iter().zip(moved) {
            // Include the 1px half-stroke: the drawn tick fits its 12px canvas.
            assert!(point.x >= bounds.origin.x + px(1.));
            assert!(point.y >= bounds.origin.y + px(1.));
            assert!(point.x <= bounds.origin.x + bounds.size.width - px(1.));
            assert!(point.y <= bounds.origin.y + bounds.size.height - px(1.));
            assert_eq!(moved - point, translated.origin - bounds.origin);
        }
        assert!(points[1].y > points[0].y && points[2].y < points[0].y);
        assert!(check_path(bounds).is_ok());
        assert!(check_path(translated).is_ok());
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod ui_tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    const SELECTORS: [&str; 5] = ["primary", "secondary", "danger", "radio", "checkbox"];

    struct ControlsView {
        focus: Vec<FocusHandle>,
        selected: bool,
        enabled: bool,
        width: f32,
    }

    impl Render for ControlsView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(self.width))
                .flex()
                .flex_col()
                .gap(px(12.))
                .text_size(px(15.))
                .font_weight(FontWeight::NORMAL)
                .children(
                    [ActionStyle::Primary, ActionStyle::Secondary, ActionStyle::Danger]
                        .into_iter()
                        .enumerate()
                        .map(|(index, style)| {
                            action(SELECTORS[index], "Stable action", &self.focus[index], style, self.enabled)
                                .debug_selector(move || SELECTORS[index].into())
                        }),
                )
                .child(
                    choice(
                        "radio", "A stable choice", "This deliberately long description wraps onto several lines without changing when focus moves or selection changes.",
                        &self.focus[3], self.selected, self.enabled,
                    )
                    .debug_selector(|| "radio".into()),
                )
                .child(
                    check("checkbox", "A stable checkbox label that wraps", &self.focus[4], self.selected, self.enabled)
                        .debug_selector(|| "checkbox".into()),
                )
        }
    }

    #[gpui::test]
    fn actual_control_bounds_survive_focus_selection_and_disabled_states(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, cx| ControlsView {
            focus: (0..5).map(|_| cx.focus_handle()).collect(),
            selected: false,
            enabled: true,
            width: 240.,
        });
        let initial: Vec<_> = SELECTORS
            .iter()
            .map(|selector| cx.debug_bounds(selector).unwrap())
            .collect();
        assert!(initial.iter().all(|bounds| bounds.size.height >= px(40.)));
        assert!(initial[3].size.height > px(60.));
        for index in 0..5 {
            cx.update(|window, cx| view.read(cx).focus[index].focus(window));
            cx.run_until_parked();
            for (selector, before) in SELECTORS.iter().zip(&initial) {
                assert_eq!(*before, cx.debug_bounds(selector).unwrap());
            }
        }
        for (selected, enabled) in [(true, true), (true, false), (false, false)] {
            cx.update(|_, cx| {
                view.update(cx, |view, cx| {
                    view.selected = selected;
                    view.enabled = enabled;
                    cx.notify();
                })
            });
            cx.run_until_parked();
            for (selector, before) in SELECTORS.iter().zip(&initial) {
                assert_eq!(*before, cx.debug_bounds(selector).unwrap());
            }
        }
    }

    #[gpui::test]
    fn wrapped_selection_copy_keeps_balanced_insets_and_actions_fit_labels(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = cx.add_window_view(|_, cx| ControlsView {
            focus: (0..5).map(|_| cx.focus_handle()).collect(),
            selected: false,
            enabled: true,
            width: 240.,
        });
        for width in [240., 552., 680.] {
            view.update(cx, |view, cx| {
                view.width = width;
                cx.notify();
            });
            cx.run_until_parked();
            for selector in ["primary", "secondary", "danger"] {
                let action = cx.debug_bounds(selector).unwrap();
                assert!(
                    action.size.width < px(width),
                    "{selector} must not stretch like an input"
                );
                assert_eq!(action.size.height, px(CONTROL_HEIGHT));
            }
            for (row, copy) in [
                ("radio", "choice-copy-radio"),
                ("checkbox", "check-copy-checkbox"),
            ] {
                let row = cx.debug_bounds(row).unwrap();
                let copy = cx.debug_bounds(copy).unwrap();
                assert_eq!(row.size.width, px(width));
                assert!(copy.top() - row.top() >= px(CHOICE_INSET));
                assert!(row.bottom() - copy.bottom() >= px(CHOICE_INSET));
                assert!(row.right() - copy.right() >= px(CHOICE_INSET));
            }
        }
    }
}
