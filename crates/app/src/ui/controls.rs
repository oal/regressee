// SPDX-License-Identifier: MPL-2.0
use super::status_icon::{StatusIcon, status_icon};
use super::theme::*;
use gpui::{prelude::*, *};
use std::time::{Duration, Instant};

/// GPUI does not expose the OS motion preference. The app supplies an explicit
/// reduced-motion override at startup; it never changes completion counters.
#[derive(Default)]
pub struct ReducedMotion(pub bool);
impl Global for ReducedMotion {}

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
            if selected { RAIL } else { RAIL_INK },
            if selected { SELECTED } else { RAIL },
            if selected { SELECTED } else { RAIL_HOVER },
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

fn focus_paint(style: StyleRefinement) -> StyleRefinement {
    focus_outline(style, INK, INK)
}

pub(super) fn focus_outline(
    mut style: StyleRefinement,
    border: u32,
    outline: u32,
) -> StyleRefinement {
    // GPUI 0.2.2 replaces Style.text wholesale, rather than recursively merging
    // it. Never set text_color (or any text property) in an interaction state.
    // This is a functional focus outline, not decorative elevation. Its paint
    // extends outside the existing 1px border without changing layout metrics.
    style = style.border_color(rgb(border));
    style.box_shadow = Some(vec![BoxShadow {
        color: rgb(outline).into(),
        offset: point(px(0.), px(0.)),
        blur_radius: px(0.),
        spread_radius: px(2.),
    }]);
    style
}

fn action_focus(style: StyleRefinement, variant: ActionStyle) -> StyleRefinement {
    match variant {
        // The white outer outline remains visible against the rail. A dark
        // inner border also marks the selected light surface independently.
        ActionStyle::Navigation { selected } => {
            focus_outline(style, if selected { RAIL } else { RAIL_INK }, RAIL_INK)
        }
        _ => focus_paint(style),
    }
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
        .when(enabled, |el| el.focus(move |s| action_focus(s, style)))
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
            .flex_col()
            .child(label.into()),
    )
}

/// Restrained form grouping: a rule and heading, not a nested card. Append the
/// section's controls with `.child` / `.children` at the call site.
pub fn section(title: &'static str, help: &'static str) -> Div {
    div()
        .debug_selector(move || format!("section-{title}"))
        .w_full()
        .flex_shrink_0()
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

/// A bounded disclosure: its title and Close never scroll with the content.
/// The caller supplies the height/placement and owns focus and dismissal.
pub fn disclosure_panel(
    id: &'static str,
    title: &'static str,
    close: AnyElement,
    body: AnyElement,
    scroll: &ScrollHandle,
) -> Stateful<Div> {
    let scrollbar = scroll.clone();
    div()
        .id(id)
        .debug_selector(move || id.into())
        .w_full()
        .h_full()
        .min_h_0()
        .min_w_0()
        .flex()
        .flex_col()
        .bg(rgb(SURFACE))
        .border_1()
        .border_color(rgb(CONTROL_LINE))
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
                .child(
                    div()
                        .debug_selector(move || format!("{id}-title"))
                        .flex_1()
                        .min_w_0()
                        .text_size(px(18.))
                        .line_height(px(24.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title),
                )
                .child(close),
        )
        .child(
            div()
                .id(SharedString::from(format!("{id}-scroll")))
                .debug_selector(move || format!("{id}-scroll"))
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .track_scroll(scroll)
                .p_4()
                .child(body),
        )
        .child(
            canvas(
                |_, _, _| (),
                move |_, _, window, _| {
                    let viewport = scrollbar.bounds();
                    let overflow = f32::from(scrollbar.max_offset().height);
                    let height = f32::from(viewport.size.height) - 8.;
                    if overflow <= 0. || height <= 0. {
                        return;
                    }
                    let thumb = (height * height / (height + overflow)).max(24.).min(height);
                    let progress = (-f32::from(scrollbar.offset().y) / overflow).clamp(0., 1.);
                    let bounds = Bounds::new(
                        point(
                            viewport.right() - px(6.),
                            viewport.top() + px(4. + progress * (height - thumb)),
                        ),
                        size(px(4.), px(thumb)),
                    );
                    window.paint_quad(fill(bounds, rgb(CONTROL_LINE)));
                },
            )
            .h(px(0.))
            .w_full(),
        )
}
pub fn status(label: impl Into<SharedString>, tone: Tone) -> Div {
    let (background, ink) = match tone {
        Tone::Neutral => (ACTION_SURFACE, MUTED),
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

/// A labelled explanation, not a decorative card. Information stays white;
/// warning/error surfaces are reserved for their actual consequences.
pub fn note(title: impl Into<SharedString>, message: impl Into<SharedString>, tone: Tone) -> Div {
    let (icon, ink, background, border) = match tone {
        Tone::Neutral | Tone::Information => (StatusIcon::Information, INK, SURFACE, LINE),
        Tone::Warning => (StatusIcon::Warning, WARNING, WARNING_SURFACE, LINE),
        Tone::Error => (StatusIcon::Error, ERROR, ERROR_SURFACE, ERROR),
    };
    div()
        .debug_selector(|| "task-note".into())
        .w(px(560.))
        .max_w_full()
        .flex_shrink_0()
        .p_4()
        .border_1()
        .border_color(rgb(border))
        .rounded(px(4.))
        .bg(rgb(background))
        .flex()
        .items_start()
        .gap_3()
        .child(
            div()
                .w(px(18.))
                .flex_shrink_0()
                .pt(px(1.))
                .child(status_icon(icon, ink, 18.)),
        )
        .child(
            div()
                .min_w_0()
                .flex_1()
                .flex()
                .flex_col()
                .gap_1()
                .text_size(px(14.))
                .line_height(px(20.))
                .text_color(rgb(ink))
                .child(
                    div()
                        .debug_selector(|| "task-note-title".into())
                        .w_full()
                        .flex()
                        .flex_col()
                        .font_weight(FontWeight::MEDIUM)
                        .child(title.into()),
                )
                .child(
                    div()
                        .debug_selector(|| "task-note-copy".into())
                        .w_full()
                        .flex()
                        .flex_col()
                        .child(message.into()),
                ),
        )
}

/// A small count with a semantic icon and complete textual meaning. Whole
/// items wrap together; counts never float away from their labels.
pub fn labelled_count(count: usize, label: &'static str, icon: StatusIcon, ink: u32) -> Div {
    div()
        .w(px(248.))
        .max_w_full()
        .flex_shrink_0()
        .flex()
        .items_start()
        .gap_2()
        .text_size(px(15.))
        .line_height(px(22.))
        .text_color(rgb(ink))
        .child(
            div()
                .flex_shrink_0()
                .pt(px(2.))
                .child(status_icon(icon, ink, 18.)),
        )
        .child(
            div()
                .debug_selector(move || format!("task-count-label-{label}"))
                .min_w_0()
                .flex_1()
                .flex()
                .flex_col()
                .child(format!("{count} {label}")),
        )
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
        .items_center()
        .gap(px(16.))
        .flex_shrink_0()
        .bg(rgb(if failed { ERROR_SURFACE } else { SURFACE }))
        .text_color(rgb(if failed { ERROR } else { INK }))
        .text_size(px(14.))
        .line_height(px(20.))
        .border_1()
        .border_color(rgb(if failed { ERROR } else { LINE }))
        .rounded(px(4.))
        .debug_selector(|| "feedback-surface".into())
        .child(copy)
        // Actions deliberately align themselves to the start of form groups.
        // Wrap this one so the feedback row can center it alongside its copy.
        .child(div().flex_shrink_0().flex().items_center().child(dismiss))
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
        .debug_selector(move || format!("field-{label}"))
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap_2()
        .child(div().font_weight(FontWeight::MEDIUM).child(label))
        .child(input)
        .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(hint))
}

/// A native activity mark, never a completion fraction. The open arc remains
/// recognizable when reduced motion is requested; a dot would read as a bullet.
pub fn activity_indicator() -> Div {
    div()
        .debug_selector(|| "activity-indicator".into())
        .size(px(16.))
        .flex_shrink_0()
        .child(
            canvas(
                |_, window, cx| activity_frame(window, cx, "compact-activity-clock"),
                |bounds, (phase, animate), window, _| {
                    if bounds.intersect(&window.content_mask().bounds).is_empty() {
                        return;
                    }
                    if let Ok(track) = activity_arc(bounds, 0., 1.) {
                        window.paint_path(track, rgb(CONTROL_LINE));
                    }
                    if let Ok(arc) = activity_arc(bounds, phase, 0.75) {
                        window.paint_path(arc, rgb(ACCENT));
                    }
                    if animate {
                        window.request_animation_frame();
                    }
                },
            )
            .size_full(),
        )
}

/// Compact unknown-duration activity, paired with readable task wording.
pub fn activity(label: impl Into<SharedString>) -> Div {
    activity_label(label, false)
}

fn activity_label(label: impl Into<SharedString>, prominent: bool) -> Div {
    let (text_size, line_height, weight) = if prominent {
        (18., 24., FontWeight::SEMIBOLD)
    } else {
        (CONTROL_TEXT_SIZE, CONTROL_LINE_HEIGHT, FontWeight::MEDIUM)
    };
    div()
        .debug_selector(|| "loading-activity".into())
        .w_full()
        .min_w_0()
        .flex()
        .items_start()
        .gap(px(8.))
        .text_size(px(text_size))
        .line_height(px(line_height))
        .font_weight(weight)
        .text_color(rgb(INK))
        .child(
            div()
                .flex_shrink_0()
                .pt(px((line_height - 16.) / 2.))
                .child(activity_indicator()),
        )
        .child(
            div()
                .debug_selector(|| "activity-label".into())
                .min_w_0()
                .flex_1()
                .flex()
                .flex_col()
                .child(label.into()),
        )
}

/// Global operations have a quiet, bounded status surface rather than an
/// unframed line that can be mistaken for an item in the screen's content.
pub fn activity_notice(label: impl Into<SharedString>) -> Div {
    div()
        .debug_selector(|| "activity-surface".into())
        .w_full()
        .min_w_0()
        .min_h(px(CONTROL_HEIGHT))
        .flex_shrink_0()
        .px(px(12.))
        .py(px(9.))
        .bg(rgb(SURFACE))
        .border_1()
        .border_color(rgb(LINE))
        .rounded(px(4.))
        .child(activity(label))
}

pub fn loading(title: &'static str, explanation: &'static str) -> Div {
    div()
        .debug_selector(|| "loading-state".into())
        .w_full()
        .max_w(px(660.))
        .py(px(24.))
        .flex()
        .flex_col()
        .gap(px(8.))
        .child(activity_label(title, true))
        .child(
            div()
                .ml(px(24.))
                .min_w_0()
                .text_size(px(14.))
                .line_height(px(20.))
                .text_color(rgb(MUTED))
                .child(explanation),
        )
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

/// One continuous list surface; the caller keeps its rows virtualized.
pub fn page_list(list: impl IntoElement) -> Div {
    div()
        .debug_selector(|| "page-list".into())
        .w_full()
        .border_1()
        .border_color(rgb(LINE))
        .rounded(px(4.))
        .overflow_hidden()
        .flex()
        .flex_col()
        .bg(rgb(SURFACE))
        .child(list)
}

/// Fixed-height, full-width rows for both discovery and captured targets.
/// Long addresses/details truncate instead of changing the virtual row height.
pub fn page_row(
    id: impl Into<ElementId>,
    address: impl Into<SharedString>,
    details: impl Into<SharedString>,
    selected: bool,
    enabled: bool,
) -> Stateful<Div> {
    target_row(id, address, details, selected, enabled, RowKind::Page)
}

/// Discovery opens page options, not a screenshot or an external website.
/// The complete address is available in the selected page's details.
pub fn discovery_row(
    id: impl Into<ElementId>,
    address: impl Into<SharedString>,
    details: impl Into<SharedString>,
    selected: bool,
    enabled: bool,
) -> Stateful<Div> {
    target_row(id, address, details, selected, enabled, RowKind::Discovery)
}

pub fn result_row(
    id: impl Into<ElementId>,
    address: impl Into<SharedString>,
    details: impl Into<SharedString>,
    selected: bool,
    enabled: bool,
) -> Stateful<Div> {
    target_row(id, address, details, selected, enabled, RowKind::Result)
}

#[derive(Clone, Copy)]
enum RowKind {
    Page,
    Discovery,
    Result,
}

fn target_row(
    id: impl Into<ElementId>,
    address: impl Into<SharedString>,
    details: impl Into<SharedString>,
    selected: bool,
    enabled: bool,
    kind: RowKind,
) -> Stateful<Div> {
    let address = address.into();
    let hint = match kind {
        RowKind::Discovery => Some(SharedString::from(
            "View page details and include/exclude options",
        )),
        RowKind::Page => Some(address.clone()),
        // The path is already visible; project origins belong in Details,
        // not in a hover overlay that obscures the screenshot being reviewed.
        RowKind::Result => None,
    };
    let result = matches!(kind, RowKind::Result);
    let path = page_path(&address);
    let details = details.into();
    let (profile, status) = details.split_once(" · ").unwrap_or((&details, ""));
    let profile = profile.to_owned();
    let status = status.to_owned();
    let id = id.into();
    let row_selector = id.clone();
    let address_selector = id.clone();
    let details_selector = id.clone();
    div()
        .id(id)
        .debug_selector(move || format!("page-row-{row_selector}"))
        .w_full()
        .min_w_0()
        .h(px(if result { 84. } else { PAGE_ROW_HEIGHT }))
        .px(px(16.))
        .py(px(8.))
        .border_b_1()
        .border_color(rgb(LINE))
        .bg(rgb(if selected { SELECTED } else { SURFACE }))
        .flex()
        .flex_col()
        .justify_center()
        .gap(px(4.))
        .when(enabled, |el| {
            el.cursor_pointer()
                .when_some(hint, |el, hint| {
                    el.tooltip(move |_, cx| cx.new(|_| PageAddressHint(hint.clone())).into())
                })
                .hover(move |style| style.bg(rgb(if selected { SELECTED } else { GROUND })))
        })
        .child(
            div()
                .debug_selector(move || format!("page-address-{address_selector}"))
                .w_full()
                .min_w_0()
                .text_size(px(14.))
                .line_height(px(20.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(INK))
                .text_ellipsis()
                .child(if result { path } else { address.to_string() }),
        )
        .child(
            div()
                .debug_selector(move || format!("page-details-{details_selector}"))
                .w_full()
                .min_w_0()
                .text_size(px(13.))
                .line_height(px(18.))
                .font_weight(FontWeight::NORMAL)
                .text_color(rgb(if selected { ACCENT_HOVER } else { MUTED }))
                .text_ellipsis()
                .child(if result { profile } else { details.to_string() }),
        )
        .when(result, |row| {
            row.child(
                div()
                    .w_full()
                    .min_w_0()
                    .text_size(px(13.))
                    .line_height(px(18.))
                    .text_color(rgb(MUTED))
                    .text_ellipsis()
                    .child(status),
            )
        })
}

/// Keep query and fragment identity; the full origin remains available in the
/// selected screenshot's keyboard-accessible Details.
pub fn page_path(address: &str) -> String {
    url::Url::parse(address)
        .ok()
        .filter(|u| matches!(u.scheme(), "http" | "https"))
        .map(|u| {
            format!(
                "{}{}{}",
                u.path(),
                u.query().map(|q| format!("?{q}")).unwrap_or_default(),
                u.fragment().map(|f| format!("#{f}")).unwrap_or_default()
            )
        })
        .unwrap_or_else(|| address.to_owned())
}

struct PageAddressHint(SharedString);
impl Render for PageAddressHint {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let mut text_style = window.text_style();
        text_style.font_weight = FontWeight::NORMAL;
        let text_width = self
            .0
            .lines()
            .map(|line| {
                window
                    .text_system()
                    .shape_line(
                        line.to_owned().into(),
                        px(14.),
                        &[text_style.to_run(line.len())],
                        None,
                    )
                    .width
            })
            .max()
            .unwrap_or(px(0.));
        // GPUI lays tooltips out with min-content space. Give wrapping a finite
        // content-derived width, not the same 480px reservation for every hint.
        let inset = px(12.);
        let width = (text_width.ceil() + (inset + px(1.)) * 2.)
            .min(px(480.))
            .min(window.viewport_size().width);
        div()
            .debug_selector(|| "page-row-hint".into())
            .w(width)
            .p(inset)
            .bg(rgb(SURFACE))
            .border_1()
            .border_color(rgb(CONTROL_LINE))
            .text_size(px(14.))
            .line_height(px(20.))
            .font_weight(FontWeight::NORMAL)
            .text_color(rgb(INK))
            .flex()
            .flex_col()
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .child(self.0.clone()),
            )
    }
}

/// History is a selection row, not a label-sized action. Its focus and selected
/// paint share the complete list width, including the otherwise empty trailing area.
pub fn history_row(
    id: impl Into<ElementId>,
    title: impl Into<SharedString>,
    details: impl Into<SharedString>,
    focus: &FocusHandle,
    selected: bool,
    enabled: bool,
) -> Stateful<Div> {
    interaction(
        // page_row owns pointer-only hover; interaction owns focus/hover for
        // keyboard rows. Registering both hover handlers panics in GPUI.
        page_row(id, title, details, selected, false).flex_shrink_0(),
        focus,
        enabled,
        SELECTED,
    )
}

fn progress_fraction(completed: u32, total: Option<u32>) -> Option<f32> {
    total
        .filter(|n| *n > 0)
        .map(|n| completed.min(n) as f32 / n as f32)
}

const ACTIVITY_PERIOD: Duration = Duration::from_millis(1200);

fn activity_phase(elapsed: Duration) -> f32 {
    (elapsed.as_secs_f64() / ACTIVITY_PERIOD.as_secs_f64()).fract() as f32
}

fn activity_frame(window: &mut Window, cx: &mut App, key: &'static str) -> (f32, bool) {
    let animate = !cx
        .try_global::<ReducedMotion>()
        .is_some_and(|motion| motion.0);
    let phase = if animate {
        // Native frames advance this element-local clock independently of worker
        // updates. Unmounting discards it; reduced motion never schedules frames.
        let clock = window.use_keyed_state(key, cx, |_, _| Instant::now());
        activity_phase(clock.read(cx).elapsed())
    } else {
        0.
    };
    (phase, animate)
}

fn activity_arc(bounds: Bounds<Pixels>, phase: f32, turns: f32) -> anyhow::Result<Path<Pixels>> {
    let radius = bounds.size.width.min(bounds.size.height) / 2. - px(1.);
    let center = bounds.center();
    let start = std::f32::consts::TAU * phase.rem_euclid(1.) - std::f32::consts::FRAC_PI_2;
    let at = |angle: f32| center + point(radius * angle.cos(), radius * angle.sin());
    let mut path = PathBuilder::stroke(px(1.8));
    path.move_to(at(start));
    // Two arcs also support a complete track: an SVG arc cannot end at its start.
    for step in [0.5, 1.] {
        path.arc_to(
            point(radius, radius),
            px(0.),
            false,
            true,
            at(start + std::f32::consts::TAU * turns * step),
        );
    }
    path.build()
}

fn activity_segments(bounds: Bounds<Pixels>, phase: f32) -> [Bounds<Pixels>; 5] {
    // Four repetitions, plus the one entering from the left. Moving one repeat
    // per cycle makes the wrap seamless; clipping keeps every painted quad in
    // the existing track without relaying out the label or surrounding controls.
    let stride = bounds.size.width / 4.;
    let segment_width = stride / 3.;
    let offset = stride * phase.rem_euclid(1.);
    std::array::from_fn(|index| {
        let start = stride * (index as f32 - 1.) + offset;
        let left = start.clamp(px(0.), bounds.size.width);
        let right = (start + segment_width).clamp(px(0.), bounds.size.width);
        Bounds::new(
            bounds.origin + point(left, px(0.)),
            size(right - left, bounds.size.height),
        )
    })
}

fn indeterminate_track() -> Div {
    div()
        .debug_selector(|| "progress-indeterminate".into())
        .size_full()
        .child(
            canvas(
                |_, window, cx| activity_frame(window, cx, "progress-activity-clock"),
                |bounds, (phase, animate), window, _| {
                    if bounds.intersect(&window.content_mask().bounds).is_empty() {
                        return;
                    }
                    for segment in activity_segments(bounds, phase) {
                        if !segment.is_empty() {
                            window.paint_quad(fill(segment, rgb(ACCENT)));
                        }
                    }
                    if animate {
                        // Native frame scheduling, not the worker's sparse
                        // progress reports or a polling timer. No frames are
                        // requested after unmounting, clipping, or reducing motion.
                        window.request_animation_frame();
                    }
                },
            )
            .size_full(),
        )
}

fn progress_track(completed: u32, total: Option<u32>) -> Div {
    let fraction = progress_fraction(completed, total);
    div()
        .debug_selector(|| "progress-track".into())
        .w_full()
        .h(px(4.))
        .flex_shrink_0()
        .bg(rgb(LINE))
        .when_some(fraction, |el, fraction| {
            el.child(
                div()
                    .debug_selector(|| "progress-fill".into())
                    .h_full()
                    .w(relative(fraction))
                    .bg(rgb(ACCENT)),
            )
        })
        .when(fraction.is_none(), |el| {
            // Activity is independent of actual work counts. Only known totals
            // above render a completion fraction.
            el.child(indeterminate_track())
        })
}

/// No false percentage for unknown-length operations. Determinate values clamp
/// to their total; zero-total uses the patterned activity track instead.
pub fn progress(label: impl Into<SharedString>, completed: u32, total: Option<u32>) -> Div {
    div()
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .flex_shrink_0()
                .text_color(rgb(INK))
                .font_weight(FontWeight::MEDIUM)
                .child(label.into()),
        )
        .child(progress_track(completed, total))
}
/// Protected information dialog only. Root shell traps/restores focus and Escape.
pub fn dialog(title: impl IntoElement, body: &'static str, close: AnyElement) -> Div {
    div()
        .debug_selector(|| "information-dialog-overlay".into())
        .occlude()
        .absolute()
        .inset_0()
        .flex()
        .items_center()
        .justify_center()
        .p_4()
        .bg(rgba((INK << 8) | 0x66))
        .child(
            div()
                .debug_selector(|| "information-dialog".into())
                .w(px(480.))
                .max_w_full()
                .max_h_full()
                .min_h_0()
                .p_6()
                .bg(rgb(SURFACE))
                .border_1()
                .border_color(rgb(CONTROL_LINE))
                .rounded(px(4.))
                .flex()
                .flex_col()
                .gap_4()
                .child(
                    div()
                        .debug_selector(|| "information-dialog-title".into())
                        .flex_shrink_0()
                        .text_size(px(21.))
                        .font_weight(FontWeight::SEMIBOLD)
                        .child(title),
                )
                .child(
                    div()
                        .id("information-dialog-body")
                        .debug_selector(|| "information-dialog-body".into())
                        .min_h_0()
                        .max_h(px(320.))
                        .overflow_y_scroll()
                        .text_color(rgb(MUTED))
                        .text_size(px(14.))
                        .line_height(px(20.))
                        .child(
                            div()
                                .debug_selector(|| "information-dialog-copy".into())
                                .child(body),
                        ),
                )
                .child(div().flex_shrink_0().child(close)),
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
                    merged.refine(&action_focus(StyleRefinement::default(), variant));
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
                    let border = match variant {
                        ActionStyle::Navigation { selected: true } => RAIL,
                        ActionStyle::Navigation { selected: false } => RAIL_INK,
                        _ => INK,
                    };
                    assert_eq!(merged.border_color, Some(rgb(border).into()));
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
        assert_eq!(secondary.border_color, Some(rgb(CONTROL_LINE).into()));
        assert!(contrast(INK, ACTION_SURFACE) >= 4.5);
        assert!(contrast(INK, ACTION_HOVER) >= 4.5);
        let selection = merged_base(selection_base(false));
        assert_eq!(selection.padding.top, px(CHOICE_INSET).into());
        assert_eq!(selection.padding.bottom, px(CHOICE_INSET).into());
        assert_eq!(selection.flex_shrink, 0.);
    }

    #[test]
    fn navigation_focus_is_visible_on_both_rail_and_selected_surface() {
        for selected in [false, true] {
            let variant = ActionStyle::Navigation { selected };
            let base = merged_base(action_base(variant));
            let (background, ink, _, hover) = action_colors(variant);
            assert!(contrast(ink, background) >= 4.5);
            assert!(contrast(ink, hover) >= 4.5);
            for hovered in [false, true] {
                let mut focused = base
                    .clone()
                    .refined(action_focus(StyleRefinement::default(), variant));
                if hovered {
                    focused.refine(&hover_paint(StyleRefinement::default(), hover));
                }
                assert_same_layout(&base, &focused);
                assert_eq!(inherited_text(&base), inherited_text(&focused));
                assert_eq!(focused.box_shadow[0].color, rgb(RAIL_INK).into());
                assert!(contrast(RAIL_INK, RAIL) >= 3.);
                if selected {
                    assert_eq!(focused.border_color, Some(rgb(RAIL).into()));
                    assert!(contrast(RAIL, SELECTED) >= 3.);
                } else {
                    assert_eq!(focused.border_color, Some(rgb(RAIL_INK).into()));
                    assert!(contrast(RAIL_INK, RAIL_HOVER) >= 3.);
                }
            }
        }
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

    #[test]
    fn progress_only_has_a_completion_fraction_for_known_nonzero_totals() {
        assert_eq!(progress_fraction(0, None), None);
        assert_eq!(progress_fraction(42, None), None);
        assert_eq!(progress_fraction(42, Some(0)), None);
        assert_eq!(progress_fraction(0, Some(100)), Some(0.));
        assert_eq!(progress_fraction(25, Some(100)), Some(0.25));
        assert_eq!(progress_fraction(100, Some(100)), Some(1.));
        assert_eq!(progress_fraction(u32::MAX, Some(100)), Some(1.));
    }

    #[test]
    fn activity_clock_advances_between_worker_updates_and_wraps_seamlessly() {
        assert_eq!(activity_phase(Duration::ZERO), 0.);
        assert_eq!(activity_phase(ACTIVITY_PERIOD), 0.);
        assert_eq!(activity_phase(Duration::from_millis(300)), 0.25);
        assert_eq!(activity_phase(Duration::from_millis(1500)), 0.25);

        let bounds = Bounds::new(point(px(10.), px(20.)), size(px(320.), px(4.)));
        let first = activity_segments(bounds, activity_phase(Duration::ZERO));
        let next = activity_segments(bounds, activity_phase(Duration::from_millis(16)));
        let displacement = next[2].left() - first[2].left();
        assert!(displacement > px(0.) && displacement < px(2.));
        assert_eq!(first[2].size, next[2].size);
        assert_eq!(first, activity_segments(bounds, 1.));
    }

    #[test]
    fn moving_activity_segments_stay_inside_narrow_and_wide_tracks() {
        for width in [0., 1., 160., 320., 680.] {
            let bounds = Bounds::new(point(px(10.), px(20.)), size(px(width), px(4.)));
            for frame in 0..=120 {
                for segment in activity_segments(bounds, frame as f32 / 120.) {
                    assert!(segment.left() >= bounds.left());
                    assert!(segment.right() <= bounds.right());
                    assert!(segment.size.width >= px(0.));
                    assert_eq!(segment.top(), bounds.top());
                    assert_eq!(segment.size.height, bounds.size.height);
                }
            }
        }
    }

    #[test]
    fn informational_feedback_is_distinct_from_selected_navigation() {
        let notice = merged_base(feedback(
            None,
            Some("Settings saved locally.".into()),
            div().into_any_element(),
        ));
        let navigation = merged_base(action_base(ActionStyle::Navigation { selected: true }));
        assert_eq!(
            notice.background.as_ref().unwrap().color(),
            Some(rgb(SURFACE).into())
        );
        assert_ne!(notice.background, navigation.background);
        assert_eq!(inherited_text(&notice).color, rgb(INK).into());
        assert_eq!(notice.align_items, Some(AlignItems::Center));

        let error = merged_base(feedback(
            Some("Could not save. Try again.".into()),
            None,
            div().into_any_element(),
        ));
        assert_eq!(
            error.background.as_ref().unwrap().color(),
            Some(rgb(ERROR_SURFACE).into())
        );
        assert_eq!(inherited_text(&error).color, rgb(ERROR).into());
    }

    #[test]
    fn compact_activity_is_readable_and_not_selection_or_completion_paint() {
        let row = merged_base(activity("Opening project…"));
        assert_eq!(inherited_text(&row).color, rgb(INK).into());
        assert_eq!(inherited_text(&row).font_weight, FontWeight::MEDIUM);
        let surface = merged_base(activity_notice("Opening project…"));
        assert_eq!(surface.min_size.height, px(CONTROL_HEIGHT).into());
        assert_eq!(
            surface.background.as_ref().unwrap().color(),
            Some(rgb(SURFACE).into())
        );
        assert_ne!(
            surface.background.as_ref().unwrap().color(),
            Some(rgb(SELECTED).into())
        );
        assert!(contrast(ACCENT, SURFACE) >= 3.);
        assert!(contrast(CONTROL_LINE, SURFACE) >= 3.);
        let bounds = Bounds::new(point(px(12.), px(20.)), size(px(16.), px(16.)));
        for frame in 0..120 {
            assert!(activity_arc(bounds, frame as f32 / 120., 0.75).is_ok());
        }
        assert!(activity_arc(bounds, 0., 1.).is_ok());
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod ui_tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    const SELECTORS: [&str; 5] = ["primary", "secondary", "danger", "radio", "checkbox"];

    struct NavigationView {
        focus: FocusHandle,
        selected: bool,
    }

    impl Render for NavigationView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(192.)).p(px(14.)).bg(rgb(RAIL)).child(action(
                "rail-navigation",
                "Review changes",
                &self.focus,
                ActionStyle::Navigation {
                    selected: self.selected,
                },
                true,
            ))
        }
    }

    #[gpui::test]
    fn mounted_navigation_geometry_survives_focus_and_light_selection(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, cx| NavigationView {
            focus: cx.focus_handle(),
            selected: false,
        });
        let initial = cx.debug_bounds("action-rail-navigation").unwrap();
        assert!(initial.size.height >= px(CONTROL_HEIGHT));
        for selected in [false, true, false] {
            view.update(cx, |view, cx| {
                view.selected = selected;
                cx.notify();
            });
            cx.update(|window, cx| view.read(cx).focus.focus(window));
            cx.run_until_parked();
            assert_eq!(initial, cx.debug_bounds("action-rail-navigation").unwrap());
        }
    }

    struct PageRowsView {
        width: f32,
        selected: bool,
        enabled: bool,
    }

    struct RowHintView {
        kind: RowKind,
        address: SharedString,
        enabled: bool,
    }

    impl Render for RowHintView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(272.)).child(target_row(
                "hint-test",
                self.address.clone(),
                "Desktop · Could not compare · 1 attempt",
                false,
                self.enabled,
                self.kind,
            ))
        }
    }

    #[gpui::test]
    fn result_rows_have_no_redundant_hint_and_other_hints_fit_content(cx: &mut TestAppContext) {
        for (width, height) in [(1280., 800.), (820., 600.)] {
            for (kind, enabled, expected_hint) in [
                (RowKind::Result, true, false),
                (RowKind::Result, false, false),
                (RowKind::Discovery, true, true),
                (RowKind::Discovery, false, false),
                (RowKind::Page, true, true),
                (RowKind::Page, false, false),
            ] {
                // Fresh windows avoid GPUI's retained debug bounds after unmount.
                let (_, cx) = cx.add_window_view(|_, _| RowHintView {
                    kind,
                    address: "https://example.test/side/aviser/".into(),
                    enabled,
                });
                cx.simulate_resize(size(px(width), px(height)));
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let row = cx.debug_bounds("page-row-hint-test").unwrap();
                cx.simulate_mouse_move(row.center(), None, Modifiers::none());
                cx.executor().advance_clock(Duration::from_millis(600));
                cx.run_until_parked();
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let hint = cx.debug_bounds("page-row-hint");
                assert_eq!(hint.is_some(), expected_hint);
                if let Some(hint) = hint {
                    assert!(hint.size.width > px(24.));
                    assert!(
                        hint.size.width < px(480.),
                        "short hints must not reserve 480px"
                    );
                    assert!(hint.right() <= px(width));
                    assert!(hint.bottom() <= px(height));
                }
                assert_eq!(row, cx.debug_bounds("page-row-hint-test").unwrap());
            }
        }
    }

    #[gpui::test]
    fn page_hints_wrap_long_addresses_within_the_window(cx: &mut TestAppContext) {
        for width in [320., 820.] {
            let (_, cx) = cx.add_window_view(|_, _| RowHintView {
                kind: RowKind::Page,
                address: format!("https://example.test/{}", "long-page-name/".repeat(30)).into(),
                enabled: true,
            });
            cx.simulate_resize(size(px(width), px(600.)));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let row = cx.debug_bounds("page-row-hint-test").unwrap();
            cx.simulate_mouse_move(row.center(), None, Modifiers::none());
            cx.executor().advance_clock(Duration::from_millis(600));
            cx.run_until_parked();
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let hint = cx.debug_bounds("page-row-hint").unwrap();
            assert!(hint.size.width <= px(480.).min(px(width)));
            assert!(hint.size.height > px(46.), "long hints must wrap");
            assert!(hint.left() >= px(0.) && hint.right() <= px(width));
            assert!(hint.top() >= px(0.) && hint.bottom() <= px(600.));
        }
    }

    impl Render for PageRowsView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            let selected = self.selected;
            let enabled = self.enabled;
            div().w(px(self.width)).child(page_list(
                uniform_list("test-page-rows", 5, move |range, _, _| {
                    range.map(|index| {
                        page_row(
                            SharedString::from(format!("row-{index}")),
                            if index == 0 { "https://example.test/".into() } else { format!("https://example.test/articles/{}?filter=long-address", "a-long-page-name-".repeat(20)) },
                            if index == 0 { "Desktop · Captured — analysis not available · 1 attempt" } else { "Mobile · Could not capture: a deliberately long failure description that must remain on one line · 2 attempts" },
                            selected && index == 0,
                            enabled,
                        )
                    }).collect::<Vec<_>>()
                }).w_full().h(px(PAGE_ROW_HEIGHT * 5.)),
            ))
        }
    }

    #[gpui::test]
    fn page_rows_fill_the_list_and_long_copy_never_changes_their_height(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| PageRowsView {
            width: 848.,
            selected: false,
            enabled: true,
        });
        for width in [320., 552., 848.] {
            for (selected, enabled) in [(false, true), (true, true), (true, false)] {
                view.update(cx, |view, cx| {
                    view.width = width;
                    view.selected = selected;
                    view.enabled = enabled;
                    cx.notify();
                });
                cx.run_until_parked();
                let list = cx.debug_bounds("page-list").unwrap();
                assert_eq!(list.size.width, px(width));
                let short = cx.debug_bounds("page-row-row-0").unwrap();
                let long = cx.debug_bounds("page-row-row-1").unwrap();
                assert_eq!(short.size, size(px(width - 2.), px(PAGE_ROW_HEIGHT)));
                assert_eq!(short.size, long.size);
                assert_eq!(short.left(), long.left());
                assert_eq!(short.bottom(), long.top());
                for (row, address, details) in [
                    (short, "page-address-row-0", "page-details-row-0"),
                    (long, "page-address-row-1", "page-details-row-1"),
                ] {
                    let address = cx.debug_bounds(address).unwrap();
                    let details = cx.debug_bounds(details).unwrap();
                    assert_eq!(address.left(), row.left() + px(16.));
                    assert_eq!(address.right(), row.right() - px(16.));
                    assert_eq!(details.left(), address.left());
                    assert_eq!(details.right(), address.right());
                    assert_eq!(address.size.height, px(20.));
                    assert_eq!(details.size.height, px(18.));
                    assert!(details.top() >= address.bottom() + px(4.));
                    assert!(details.bottom() <= row.bottom() - px(8.));
                }
            }
        }
    }

    struct ProgressView {
        completed: u32,
        total: Option<u32>,
        width: f32,
    }

    struct ActivityView {
        width: f32,
        prominent: bool,
    }

    impl Render for ActivityView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(self.width)).child(if self.prominent {
                loading(
                    "Loading saved screenshots…",
                    "Reading this project’s page list, captures and comparison results from disk.",
                )
            } else {
                activity_notice("Opening a project with a longer name and reading its saved settings from local storage…")
            })
        }
    }

    #[gpui::test]
    fn activity_icon_and_wrapped_labels_keep_their_bounds_with_reduced_motion(
        cx: &mut TestAppContext,
    ) {
        cx.update(|cx| cx.set_global(ReducedMotion(true)));
        let (view, cx) = cx.add_window_view(|_, _| ActivityView {
            width: 160.,
            prominent: false,
        });
        for width in [160., 320., 680.] {
            for prominent in [false, true] {
                view.update(cx, |view, cx| {
                    view.width = width;
                    view.prominent = prominent;
                    cx.notify();
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let icon = cx.debug_bounds("activity-indicator").unwrap();
                let label = cx.debug_bounds("activity-label").unwrap();
                let row = cx.debug_bounds("loading-activity").unwrap();
                assert_eq!(icon.size, size(px(16.), px(16.)));
                assert_eq!(label.left(), icon.right() + px(8.));
                assert!(label.right() <= row.right());
                assert!(label.bottom() <= row.bottom());
                assert_eq!(icon.top(), row.top() + px(if prominent { 4. } else { 2. }));
                cx.update(|window, cx| {
                    assert_eq!(activity_frame(window, cx, "reduced-test"), (0., false));
                    let _ = window.draw(cx);
                });
                assert_eq!(icon, cx.debug_bounds("activity-indicator").unwrap());
                assert_eq!(label, cx.debug_bounds("activity-label").unwrap());
            }
        }
    }

    impl Render for ProgressView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(self.width)).child(progress(
                "Finding rendered links · 1 page visited — no clicking or forms",
                self.completed,
                self.total,
            ))
        }
    }

    #[gpui::test]
    fn known_and_unknown_progress_keep_a_visible_bounded_track(cx: &mut TestAppContext) {
        let (view, cx) = cx.add_window_view(|_, _| ProgressView {
            completed: 1,
            total: None,
            width: 320.,
        });
        for width in [160., 320., 680.] {
            for (completed, total) in [
                (0, None),
                (1, None),
                (u32::MAX, Some(0)),
                (0, Some(100)),
                (25, Some(100)),
                (100, Some(100)),
                (u32::MAX, Some(100)),
            ] {
                view.update(cx, |view, cx| {
                    view.completed = completed;
                    view.total = total;
                    view.width = width;
                    cx.notify();
                });
                cx.run_until_parked();
                let track = cx.debug_bounds("progress-track").unwrap();
                assert_eq!(track.size, size(px(width), px(4.)));
                // Pinned GPUI retains old debug selectors between frames;
                // measure the active mode rather than asserting old ones vanish.
                if let Some(fraction) = progress_fraction(completed, total) {
                    let fill = cx.debug_bounds("progress-fill").unwrap();
                    assert_eq!(fill.size, size(px(width * fraction), px(4.)));
                } else {
                    assert_eq!(cx.debug_bounds("progress-indeterminate").unwrap(), track);
                }
            }
        }
    }

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
