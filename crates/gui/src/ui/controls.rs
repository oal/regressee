// SPDX-License-Identifier: MPL-2.0
use super::depth::Elevation;
use super::status_icon::{StatusIcon, status_icon};
use super::theme::*;
use gpui::{prelude::*, *};
use std::time::{Duration, Instant};

/// GPUI does not expose the OS motion preference. The app supplies an explicit
/// reduced-motion override at startup and on runtime preference changes; it never
/// changes completion counters. Keep this public path for existing callers.
#[derive(Default)]
pub struct ReducedMotion(pub bool);
impl Global for ReducedMotion {}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ActionStyle {
    Primary,
    Secondary,
    Quiet,
    Danger,
    Navigation {
        selected: bool,
    },
    Tab {
        selected: bool,
    },
    Segment {
        selected: bool,
    },
    Anchor {
        selected: bool,
    },
    /// Full-width descriptive form choices, not compact view selectors.
    Choice {
        selected: bool,
    },
}

fn action_colors(style: ActionStyle) -> (u32, u32, u32, u32) {
    match style {
        ActionStyle::Primary => (
            palette().accent,
            palette().on_accent,
            palette().accent,
            palette().accent_hover,
        ),
        ActionStyle::Secondary => (
            palette().action_surface,
            palette().ink,
            palette().control_line,
            palette().action_hover,
        ),
        ActionStyle::Quiet => (
            palette().ground,
            palette().ink,
            palette().ground,
            palette().action_hover,
        ),
        ActionStyle::Tab { selected } | ActionStyle::Anchor { selected } => (
            if selected {
                palette().surface
            } else {
                palette().ground
            },
            if selected {
                palette().accent_hover
            } else {
                palette().ink
            },
            if selected {
                palette().accent
            } else {
                palette().line
            },
            palette().action_hover,
        ),
        ActionStyle::Segment { selected } => (
            if selected {
                palette().surface
            } else {
                palette().action_hover
            },
            if selected {
                palette().accent_hover
            } else {
                palette().ink
            },
            if selected {
                palette().accent
            } else {
                palette().action_hover
            },
            palette().selected,
        ),
        ActionStyle::Danger => (
            palette().error_surface,
            palette().error,
            palette().error,
            palette().error_surface,
        ),
        ActionStyle::Navigation { selected } => (
            if selected {
                palette().selected
            } else {
                palette().rail
            },
            if selected {
                palette().nav_selected_ink
            } else {
                palette().rail_ink
            },
            if selected {
                palette().selected
            } else {
                palette().rail
            },
            if selected {
                palette().selected
            } else {
                palette().rail_hover
            },
        ),
        ActionStyle::Choice { selected } => (
            if selected {
                palette().selected
            } else {
                palette().surface
            },
            if selected {
                palette().accent_hover
            } else {
                palette().ink
            },
            if selected {
                palette().accent
            } else {
                palette().control_line
            },
            palette().selected,
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
        .when(
            matches!(
                style,
                ActionStyle::Primary
                    | ActionStyle::Secondary
                    | ActionStyle::Danger
                    | ActionStyle::Navigation { selected: true }
                    | ActionStyle::Choice { .. }
            ),
            |el| el.shadow(Elevation::Control.shadows()),
        )
        .when(matches!(style, ActionStyle::Quiet), |el| {
            el.border_color(transparent_black())
        })
        .when(
            matches!(style, ActionStyle::Tab { .. } | ActionStyle::Anchor { .. }),
            |el| {
                el.border_0()
                    .border_b_2()
                    .rounded(px(0.))
                    .border_color(match style {
                        ActionStyle::Anchor { selected: false } => transparent_black(),
                        _ => rgb(border).into(),
                    })
            },
        )
        .when(matches!(style, ActionStyle::Segment { .. }), |el| {
            el.px(px(10.))
                .py(px(0.))
                .h(px(CONTROL_HEIGHT))
                .border_color(
                    if matches!(style, ActionStyle::Segment { selected: true }) {
                        rgb(palette().accent).into()
                    } else {
                        transparent_black()
                    },
                )
        })
        .text_color(rgb(text))
        .text_size(px(CONTROL_TEXT_SIZE))
        .line_height(px(CONTROL_LINE_HEIGHT))
        .font_weight(FontWeight::MEDIUM)
}

fn focus_paint(style: StyleRefinement) -> StyleRefinement {
    focus_outline(style, palette().accent)
}

/// Paint outside a focused control: 2px ring plus the 1px separating gap.
pub const FOCUS_OUTSET: f32 = 3.;

pub(super) fn focus_outline(mut style: StyleRefinement, outline: u32) -> StyleRefinement {
    // GPUI 0.2.2 replaces Style.text wholesale, rather than recursively merging
    // it. Never set text_color (or any text property) in an interaction state.
    // Functional focus paint, not elevation: a crisp 2px ring with a 1px gap.
    // Keep the normal control border instead of drawing a second focus stroke.
    // The contrasting gap separates focus from filled/selected controls without
    // changing their geometry, semantic colors or inherited text metrics.
    let gap = if outline == palette().rail_ink {
        palette().rail
    } else {
        palette().surface
    };
    style.box_shadow = Some(vec![
        BoxShadow {
            color: rgb(outline).into(),
            offset: point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(FOCUS_OUTSET),
        },
        BoxShadow {
            color: rgb(gap).into(),
            offset: point(px(0.), px(0.)),
            blur_radius: px(0.),
            spread_radius: px(1.),
        },
    ]);
    style
}

fn action_focus(style: StyleRefinement, variant: ActionStyle) -> StyleRefinement {
    match variant {
        // The navy gap separates the white ring from a selected mint surface.
        ActionStyle::Navigation { .. } => focus_outline(style, palette().rail_ink),
        _ => focus_paint(style),
    }
}

#[cfg(test)]
fn hover_paint(style: StyleRefinement, background: u32) -> StyleRefinement {
    // Hover is merged after focus in GPUI. Do not overwrite the focus outline.
    style.bg(rgb(background))
}

fn interaction(
    control: Stateful<Div>,
    focus: &FocusHandle,
    enabled: bool,
    hover_background: u32,
    background: u32,
) -> Stateful<Div> {
    interaction_colors(
        control,
        focus,
        enabled,
        rgb(background).into(),
        rgb(hover_background).into(),
    )
}

fn interaction_colors(
    control: Stateful<Div>,
    focus: &FocusHandle,
    enabled: bool,
    background: Hsla,
    hover_background: Hsla,
) -> Stateful<Div> {
    control
        .relative()
        // Only the paint layer owns fill. In particular a transparent toolbar
        // target must not expose its next selected fill beneath the transition.
        .bg(gpui::transparent_black())
        .child(super::motion::color_layer(
            "control-color-motion",
            background,
            hover_background,
            enabled,
            true,
            3.,
        ))
        .when(enabled, |el| {
            el.track_focus(focus)
                .cursor_pointer()
                .focus(focus_paint)
                .hover(|s| s.bg(gpui::transparent_black()))
                .active(|s| s.bg(gpui::transparent_black()))
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
    let id = id.into();
    let label_selector = id.clone();
    action_frame(id, focus, style, enabled)
        // A flex text wrapper retains intrinsic measurement while allowing long
        // labels to wrap. Toolbar frames supply icons/labels themselves.
        .child(
            div()
                .flex()
                .min_w(px(0.))
                .debug_selector(move || format!("action-label-{label_selector}"))
                .child(label.into()),
        )
}

/// An ordinary labelled action, not a quiet toolbar tool. The glyph inherits
/// the action's ink (including white primary and red danger); labels still wrap.
pub fn action_with_icon(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    glyph: super::toolbar::ToolIcon,
    focus: &FocusHandle,
    style: ActionStyle,
    enabled: bool,
) -> Stateful<Div> {
    let id = id.into();
    let label_selector = id.clone();
    let icon_selector = id.clone();
    let (_, ink, _, _) = action_colors(style);
    action_frame(id, focus, style, enabled)
        .gap(px(8.))
        .child(
            div()
                .debug_selector(move || format!("action-icon-{icon_selector}"))
                .flex_shrink_0()
                .flex()
                .items_center()
                .child(super::toolbar::icon(glyph, ink, 18.)),
        )
        .child(ActionLabel {
            id: label_selector,
            text: label.into(),
        })
}

#[derive(IntoElement)]
struct ActionLabel {
    id: ElementId,
    text: SharedString,
}

impl RenderOnce for ActionLabel {
    fn render(self, window: &mut Window, _: &mut App) -> impl IntoElement {
        // A bare flex column has zero intrinsic preferred width in some GPUI
        // containers. Retain natural width, but let the parent subtract its
        // glyph/gap/insets and shrink this column to a finite wrapping width.
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let preferred = self
            .text
            .split('\n')
            .map(|line| {
                window
                    .text_system()
                    .shape_line(
                        SharedString::from(line.to_owned()),
                        font_size,
                        &[style.to_run(line.len())],
                        None,
                    )
                    .width
                    .ceil()
            })
            .fold(px(1.), |width, line| width.max(line));
        let id = self.id;
        div()
            .debug_selector(move || format!("action-label-{id}"))
            .w(preferred)
            .max_w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .child(self.text)
    }
}

pub(super) fn action_frame(
    id: impl Into<ElementId>,
    focus: &FocusHandle,
    style: ActionStyle,
    enabled: bool,
) -> Stateful<Div> {
    let (background, _, _, hover_background) = action_colors(style);
    action_frame_colors(
        id,
        focus,
        style,
        enabled,
        rgb(background).into(),
        rgb(hover_background).into(),
    )
}

pub(super) fn action_frame_colors(
    id: impl Into<ElementId>,
    focus: &FocusHandle,
    style: ActionStyle,
    enabled: bool,
    background: Hsla,
    hover_background: Hsla,
) -> Stateful<Div> {
    let id = id.into();
    let action_selector = id.clone();
    let background = if matches!(style, ActionStyle::Quiet | ActionStyle::Anchor { .. }) {
        transparent_black()
    } else {
        background
    };
    interaction_colors(
        action_base(style).bg(background).id(id),
        focus,
        enabled,
        background,
        hover_background,
    )
    .when(!enabled, |el| el.shadow(Vec::new()))
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
}

/// One connected tray for exclusive values. Each item retains a 40px target;
/// callers own the values, focus order and guarded activation, not this paint.
pub fn segmented_group(id: &'static str) -> Div {
    let mut group = div()
        .debug_selector(move || id.into())
        .flex()
        .flex_wrap()
        .items_center()
        .gap_0()
        .min_w_0()
        .max_w_full()
        .flex_shrink_0()
        .border_0()
        .rounded(px(5.))
        .bg(rgb(palette().action_hover));
    group.style().align_self = Some(AlignSelf::FlexStart);
    group
}

/// Compact independent membership/view toggle, never a command or a tab.
/// The drawn check remains immediate; focus and activation use ordinary guards.
pub fn toggle(
    id: impl Into<ElementId>,
    label: impl Into<SharedString>,
    focus: &FocusHandle,
    checked: bool,
    enabled: bool,
) -> Stateful<Div> {
    let id = id.into();
    let label_id = id.clone();
    action_frame(id, focus, ActionStyle::Quiet, enabled)
        .gap(px(8.))
        .justify_start()
        .px(px(8.))
        .child(indicator_base(checked, false).when(checked, |el| {
            el.child(
                canvas(
                    |_, _, _| (),
                    |bounds, (), window, _| {
                        if let Ok(path) = check_path(bounds) {
                            window.paint_path(path, rgb(palette().on_accent));
                        }
                    },
                )
                .size(px(12.)),
            )
        }))
        .child(ActionLabel {
            id: label_id,
            text: label.into(),
        })
}

fn selection_base(selected: bool) -> Div {
    action_base(ActionStyle::Choice { selected })
        // Form choices are a continuous reading surface, not raised buttons.
        .shadow(Vec::new())
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
        .border_color(rgb(if selected {
            palette().accent
        } else {
            palette().control_line
        }))
        // Keep the authoritative mark immediate: fading a white tick over an
        // unchecked white box would briefly hide the actual checked value.
        .bg(rgb(if selected && !radio {
            palette().accent
        } else {
            palette().surface
        }))
}

fn radio_dot() -> Div {
    div().size(px(8.)).rounded(px(4.)).bg(rgb(palette().accent))
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
        .text_color(rgb(palette().muted))
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
    interaction(
        selection_base(selected).id(id),
        focus,
        enabled,
        palette().selected,
        if selected {
            palette().selected
        } else {
            palette().surface
        },
    )
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
                        .text_color(rgb(if selected {
                            palette().accent_hover
                        } else {
                            palette().muted
                        }))
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
        palette().selected,
        if checked {
            palette().selected
        } else {
            palette().surface
        },
    )
    .debug_selector(move || format!("check-{row_selector}"))
    .child(indicator_base(checked, false).when(checked, |el| {
        el.child(
            canvas(
                |_, _, _| (),
                |bounds, (), window, _| {
                    if let Ok(path) = check_path(bounds) {
                        window.paint_path(path, rgb(palette().on_accent));
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
        .border_color(rgb(palette().line))
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
                        .text_color(rgb(palette().ink))
                        .child(title),
                )
                .when(!help.is_empty(), |el| {
                    el.child(help_copy().max_w(px(660.)).child(help))
                }),
        )
}

/// Inline expansion is not a selected value: a leading right/down marker and
/// explicit Show/Hide replace the field-shaped trigger used by real selectors.
pub fn disclosure_toggle(
    id: impl Into<ElementId>,
    title: impl Into<SharedString>,
    focus: &FocusHandle,
    open: bool,
    enabled: bool,
    section: bool,
) -> Stateful<Div> {
    let id = id.into();
    let action_selector = id.clone();
    let title_selector = id.clone();
    let state_selector = id.clone();
    interaction_colors(
        div()
            .id(id)
            .w_full()
            .min_w_0()
            .min_h(px(CONTROL_HEIGHT))
            .flex_shrink_0()
            .px(px(8.))
            .py(px(8.))
            .border_1()
            .border_color(gpui::transparent_black())
            .rounded(px(4.))
            .flex()
            .items_start()
            .gap(px(8.))
            .text_color(rgb(palette().ink))
            .text_size(px(if section { 18. } else { CONTROL_TEXT_SIZE }))
            .line_height(px(if section { 24. } else { CONTROL_LINE_HEIGHT }))
            .font_weight(FontWeight::SEMIBOLD),
        focus,
        enabled,
        transparent_black(),
        rgb(palette().action_hover).into(),
    )
    // Preserve action selectors so native focus/reveal probes measure the same
    // complete interactive bounds for both ordinary actions and disclosures.
    .debug_selector(move || format!("action-{action_selector}"))
    .child(
        div()
            .flex_shrink_0()
            .pt(px(if section { 4. } else { 2. }))
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        let at = |x: f32, y: f32| {
                            bounds.origin + point(bounds.size.width * x, bounds.size.height * y)
                        };
                        let mut path = PathBuilder::stroke(px(1.8));
                        if open {
                            path.move_to(at(0.2, 0.35));
                            path.line_to(at(0.5, 0.65));
                            path.line_to(at(0.8, 0.35));
                        } else {
                            path.move_to(at(0.35, 0.2));
                            path.line_to(at(0.65, 0.5));
                            path.line_to(at(0.35, 0.8));
                        }
                        if let Ok(path) = path.build() {
                            window.paint_path(path, rgb(palette().ink));
                        }
                    },
                )
                .size(px(16.)),
            ),
    )
    .child(
        div()
            .debug_selector(move || format!("disclosure-title-{title_selector}"))
            .min_w_0()
            .flex_1()
            .flex()
            .flex_col()
            .child(title.into()),
    )
    .child(
        help_copy()
            .debug_selector(move || format!("disclosure-state-{state_selector}"))
            .flex_shrink_0()
            .pt(px(if section { 3. } else { 1. }))
            .child(if open { "Hide" } else { "Show" }),
    )
}

/// One subtree owns header, persistent help and revealed content. There is no
/// separator or new scroll owner between the trigger and its indented body.
pub fn disclosure_group(
    id: impl Into<SharedString>,
    trigger: AnyElement,
    help: impl Into<SharedString>,
    body: Option<AnyElement>,
) -> Div {
    let id = id.into();
    let body_id = id.clone();
    let help = help.into();
    div()
        .debug_selector(move || format!("disclosure-group-{id}"))
        .w_full()
        .min_w_0()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap(px(4.))
        .child(trigger)
        .when(!help.is_empty(), |group| {
            group.child(help_copy().pl(px(32.)).child(help))
        })
        .when_some(body, |group, body| {
            group.child(
                div()
                    .debug_selector(move || format!("disclosure-body-{body_id}"))
                    .w_full()
                    .min_w_0()
                    .flex_shrink_0()
                    .pl(px(32.))
                    .pt(px(8.))
                    .pb(px(8.))
                    .child(body),
            )
        })
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
    div()
        .id(id)
        .debug_selector(move || id.into())
        .w_full()
        .h_full()
        .min_h_0()
        .min_w_0()
        .flex()
        .flex_col()
        .bg(rgb(palette().surface))
        .border_1()
        .border_color(rgb(palette().control_line))
        .shadow(Elevation::Floating.shadows())
        .child(
            div()
                .flex_shrink_0()
                .flex()
                .items_center()
                .justify_between()
                .gap_2()
                .p_3()
                .bg(super::depth::chrome())
                .border_b_1()
                .border_color(rgb(palette().line))
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
        .child(super::scrollbar::vertical(
            SharedString::from(format!("{id}-scrollbar")),
            scroll,
            div()
                .id(SharedString::from(format!("{id}-scroll")))
                .debug_selector(move || format!("{id}-scroll"))
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .track_scroll(scroll)
                .p_4()
                .child(body),
        ))
}
pub fn status(label: impl Into<SharedString>, tone: Tone) -> Div {
    let (background, ink) = match tone {
        Tone::Neutral => (palette().action_surface, palette().muted),
        Tone::Information => (palette().selected, palette().accent_hover),
        Tone::Warning => (palette().warning_surface, palette().warning),
        Tone::Error => (palette().error_surface, palette().error),
    };
    div()
        .px_2()
        .py_1()
        .rounded(px(3.))
        .bg(rgb(background))
        .text_color(rgb(ink))
        .text_size(px(13.))
        .line_height(px(HELP_LINE_HEIGHT))
        .child(label.into())
}

/// A labelled explanation, not a decorative card. Information stays white;
/// warning/error surfaces are reserved for their actual consequences.
pub fn note(title: impl Into<SharedString>, message: impl Into<SharedString>, tone: Tone) -> Div {
    let (icon, ink, background, border) = match tone {
        Tone::Neutral | Tone::Information => (
            StatusIcon::Information,
            palette().ink,
            palette().surface,
            palette().line,
        ),
        Tone::Warning => (
            StatusIcon::Warning,
            palette().warning,
            palette().warning_surface,
            palette().line,
        ),
        Tone::Error => (
            StatusIcon::Error,
            palette().error,
            palette().error_surface,
            palette().error,
        ),
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
        .bg(rgb(palette().error_surface))
        .text_color(rgb(palette().error))
        .border_1()
        .border_color(rgb(palette().error))
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
        .pr(px(super::scrollbar::GUTTER))
        .debug_selector(|| "feedback-copy".into())
        .child(messages);
    div()
        .p(px(16.))
        .flex()
        .items_center()
        .gap(px(16.))
        .flex_shrink_0()
        .bg(rgb(if failed {
            palette().error_surface
        } else {
            palette().surface
        }))
        .text_color(rgb(if failed {
            palette().error
        } else {
            palette().ink
        }))
        .text_size(px(14.))
        .line_height(px(20.))
        .border_1()
        .border_color(rgb(if failed {
            palette().error
        } else {
            palette().line
        }))
        .rounded(px(4.))
        .debug_selector(|| "feedback-surface".into())
        .child(super::scrollbar::scrollable(
            "feedback-copy-scrollbar",
            copy,
        ))
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
                .text_size(px(TITLE_TEXT_SIZE))
                .line_height(px(TITLE_LINE_HEIGHT))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
        .child(
            div()
                .text_color(rgb(palette().muted))
                .text_size(px(BODY_TEXT_SIZE))
                .line_height(px(BODY_LINE_HEIGHT))
                .child(explanation),
        )
}

/// Reserve the screenshot stage when its image is missing. The message centers
/// in spare space, but grows from the top and scrolls when it exceeds the stage.
pub fn image_placeholder(
    title: &'static str,
    explanation: impl Into<SharedString>,
    help: &'static str,
    tone: Tone,
    scroll: &ScrollHandle,
    stage_width: f32,
) -> Div {
    image_placeholder_frame(
        scroll,
        image_placeholder_message(stage_width)
            .child(
                div()
                    .debug_selector(|| "screenshot-placeholder-icon".into())
                    .size(px(48.))
                    .flex_shrink_0()
                    .child(status_icon(
                        StatusIcon::MissingImage,
                        if matches!(tone, Tone::Error) {
                            palette().error
                        } else {
                            palette().muted
                        },
                        48.,
                    )),
            )
            .child(
                div()
                    .w_full()
                    .min_w_0()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        image_placeholder_title(title)
                            .debug_selector(|| "screenshot-placeholder-title".into()),
                    )
                    .child(
                        div()
                            .debug_selector(|| "screenshot-placeholder-reason".into())
                            .text_size(px(14.))
                            .line_height(px(20.))
                            .text_color(rgb(palette().ink))
                            .child(explanation.into()),
                    ),
            )
            .child(
                help_copy()
                    .debug_selector(|| "screenshot-placeholder-help".into())
                    .w_full()
                    .flex_shrink_0()
                    .pt(px(16.))
                    .border_t_1()
                    .border_color(rgb(palette().line))
                    .child(help),
            ),
    )
}

/// Loading belongs to the same reserved stage as missing screenshot evidence.
/// The spinner expresses activity only, never a completion fraction or image.
pub fn image_loading(
    title: &'static str,
    explanation: &'static str,
    scroll: &ScrollHandle,
    stage_width: f32,
) -> Div {
    image_placeholder_frame(
        scroll,
        image_placeholder_message(stage_width)
            .child(activity_indicator_at_size(32.))
            .child(
                div()
                    .debug_selector(|| "loading-state".into())
                    .w_full()
                    .min_w_0()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap(px(8.))
                    .child(
                        image_placeholder_title(title).debug_selector(|| "loading-activity".into()),
                    )
                    .child(
                        div()
                            .debug_selector(|| "screenshot-loading-explanation".into())
                            .text_size(px(14.))
                            .line_height(px(20.))
                            .text_color(rgb(palette().muted))
                            .child(explanation),
                    ),
            ),
    )
    .debug_selector(|| "screenshot-loading".into())
}

fn image_placeholder_title(title: &'static str) -> Div {
    div()
        .text_size(px(18.))
        .line_height(px(24.))
        .font_weight(FontWeight::SEMIBOLD)
        .text_color(rgb(palette().ink))
        .child(title)
}

fn image_placeholder_message(stage_width: f32) -> Div {
    // Measure wrapped text at its final width. Percentage/max-width sizing can
    // otherwise underreport the message height and scroll extent in GPUI.
    let message_width = (stage_width - 50.).clamp(1., 440.);
    div()
        .debug_selector(|| "screenshot-placeholder-message".into())
        .w(px(message_width))
        .my_auto()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .items_center()
        .gap(px(16.))
        .text_center()
}

fn image_placeholder_frame(scroll: &ScrollHandle, message: Div) -> Div {
    div()
        .debug_selector(|| "screenshot-placeholder".into())
        .shadow(Elevation::Surface.shadows())
        .w_full()
        .min_w_0()
        .flex_1()
        .min_h_0()
        .flex()
        .flex_col()
        .overflow_hidden()
        .border_1()
        .border_color(rgb(palette().line))
        .bg(rgb(palette().surface))
        .child(super::scrollbar::vertical(
            "placeholder-scrollbar",
            scroll,
            div()
                .id("screenshot-placeholder-scroll")
                .debug_selector(|| "screenshot-placeholder-scroll".into())
                .track_scroll(scroll)
                .w_full()
                .min_w_0()
                .flex_1()
                .min_h_0()
                .overflow_y_scroll()
                .p(px(24.))
                .flex()
                .flex_col()
                .items_center()
                .child(message),
        ))
}

/// Compact list search. Keep the existing clear action and its shell-owned
/// keyboard order beside the field rather than introducing a second input.
pub fn search_field(input: AnyElement, clear: Option<AnyElement>) -> Div {
    div()
        .debug_selector(|| "search-field-row".into())
        .w_full()
        .flex_shrink_0()
        .flex()
        .items_center()
        .gap_2()
        .child(div().flex_1().min_w_0().child(input))
        .when_some(clear, |row, clear| row.child(clear))
}

pub fn field(label: &'static str, hint: &'static str, input: AnyElement) -> Div {
    div()
        .debug_selector(move || format!("field-{label}"))
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .debug_selector(move || format!("field-label-{label}"))
                .text_size(px(BODY_TEXT_SIZE))
                .line_height(px(BODY_LINE_HEIGHT))
                .font_weight(FontWeight::MEDIUM)
                .child(label),
        )
        .child(input)
        .when(!hint.is_empty(), |field| {
            field.child(
                help_copy()
                    .debug_selector(move || format!("field-hint-{label}"))
                    .child(hint),
            )
        })
}

/// One persistent label and one stable help/correction region per form input.
pub fn validated_field(
    label: &'static str,
    hint: &'static str,
    input: Entity<super::text_input::TextInput>,
) -> Div {
    field(
        label,
        "",
        super::text_input::SupportedInput { input, hint }.into_any_element(),
    )
}

/// A native activity mark, never a completion fraction. The open arc remains
/// recognizable when reduced motion is requested; a dot would read as a bullet.
pub fn activity_indicator() -> Div {
    activity_indicator_at_size(16.)
}

fn activity_indicator_at_size(edge: f32) -> Div {
    div()
        .debug_selector(|| "activity-indicator".into())
        .size(px(edge))
        .flex_shrink_0()
        .child(
            canvas(
                |_, window, cx| activity_frame(window, cx, "compact-activity-clock"),
                |bounds, (phase, animate), window, _| {
                    if bounds.intersect(&window.content_mask().bounds).is_empty() {
                        return;
                    }
                    if let Ok(track) = activity_arc(bounds, 0., 1.) {
                        window.paint_path(track, rgb(palette().control_line));
                    }
                    if let Ok(arc) = activity_arc(bounds, phase, 0.75) {
                        window.paint_path(arc, rgb(palette().accent));
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
        .text_color(rgb(palette().ink))
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
        .bg(rgb(palette().surface))
        .border_1()
        .border_color(rgb(palette().line))
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
                .text_color(rgb(palette().muted))
                .child(explanation),
        )
}
pub fn list_row(id: impl Into<ElementId>) -> Stateful<Div> {
    div()
        .id(id)
        .w_full()
        .py_4()
        .border_b_1()
        .border_color(rgb(palette().line))
        .flex()
        .items_center()
        .gap_4()
}

/// One continuous list surface; the caller keeps its rows virtualized.
pub fn page_list(list: impl IntoElement) -> Div {
    div()
        .debug_selector(|| "page-list".into())
        .shadow(Elevation::Surface.shadows())
        .w_full()
        .border_1()
        .border_color(rgb(palette().line))
        .rounded(px(4.))
        .overflow_hidden()
        .flex()
        .flex_col()
        .bg(rgb(palette().surface))
        .child(list)
}

/// Fixed-height, full-width rows for both discovery and captured targets.
/// Long addresses/details truncate instead of changing the virtual row height.
/// Full context belongs in the selected item's explicit details, not a tooltip.
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
        .border_color(rgb(palette().line))
        .bg(rgb(if selected {
            palette().selected
        } else {
            palette().surface
        }))
        .flex()
        .flex_col()
        .justify_center()
        .gap(px(4.))
        .when(enabled, |el| {
            el.cursor_pointer().hover(move |style| {
                style.bg(rgb(if selected {
                    palette().selected
                } else {
                    palette().ground
                }))
            })
        })
        .child(
            div()
                .debug_selector(move || format!("page-address-{address_selector}"))
                .w_full()
                .min_w_0()
                .text_size(px(14.))
                .line_height(px(20.))
                .font_weight(FontWeight::MEDIUM)
                .text_color(rgb(palette().ink))
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
                .text_color(rgb(if selected {
                    palette().accent_hover
                } else {
                    palette().muted
                }))
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
                    .text_color(rgb(palette().muted))
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
    let id = id.into();
    interaction(
        // Mount the animated background BEFORE the row's copy. Applying
        // interaction to an already populated page_row paints over its text.
        div()
            .id(SharedString::from(format!("history-frame-{id}")))
            .w_full()
            .min_w_0()
            .flex_shrink_0(),
        focus,
        enabled,
        palette().selected,
        if selected {
            palette().selected
        } else {
            palette().surface
        },
    )
    // Only the outer frame owns input/hover; the inner row retains its labels,
    // ellipsis, dimensions and separator over a transparent background.
    .child(page_row(id, title, details, selected, false).bg(transparent_black()))
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
                            window.paint_quad(fill(segment, rgb(palette().accent)));
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
        .relative()
        .w_full()
        .h(px(4.))
        .flex_shrink_0()
        .bg(rgb(palette().line))
        .when_some(fraction, |el, fraction| {
            el.child(
                div()
                    .debug_selector(|| "progress-fill".into())
                    // Preserve the reported target's diagnostic bounds. Only
                    // the sibling canvas paints; it never drives layout.
                    .h_full()
                    .w(relative(fraction)),
            )
            .child(
                canvas(
                    move |_, window, cx| {
                        let now = cx.background_executor().now();
                        window.use_keyed_state("determinate-progress-motion", cx, |_, _| {
                            super::motion::ProgressState::new(completed, total.unwrap(), now)
                        })
                    },
                    move |bounds, state, window, cx| {
                        let visible = !bounds.intersect(&window.content_mask().bounds).is_empty();
                        let immediate = super::motion::reduced(cx) || !visible;
                        let now = cx.background_executor().now();
                        let (fraction, active) = state.update(cx, |state, _| {
                            state.report(completed, total.unwrap(), now, immediate)
                        });
                        #[cfg(all(test, feature = "ui-tests"))]
                        super::motion::record_progress(cx, fraction, active);
                        if visible {
                            window.paint_quad(fill(
                                Bounds::new(
                                    bounds.origin,
                                    size(bounds.size.width * fraction, bounds.size.height),
                                ),
                                rgb(palette().accent),
                            ));
                            if active {
                                window.request_animation_frame();
                            }
                        }
                    },
                )
                .absolute()
                .inset_0(),
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
                .text_color(rgb(palette().ink))
                .font_weight(FontWeight::MEDIUM)
                .child(label.into()),
        )
        .child(progress_track(completed, total))
}
/// Protected information dialog only. Root shell traps/restores focus and Escape.
pub fn dialog(
    title: impl IntoElement + 'static,
    body: &'static str,
    close: AnyElement,
    cx: &App,
) -> Div {
    let panel = super::dialog::surface(520.)
        .debug_selector(|| "information-dialog".into())
        .p_6()
        .gap_4()
        .child(
            super::dialog::heading()
                .debug_selector(|| "information-dialog-title".into())
                .child(title),
        )
        .child(super::scrollbar::scrollable(
            "information-scrollbar",
            div()
                .id("information-dialog-body")
                .debug_selector(|| "information-dialog-body".into())
                .min_h_0()
                .max_h(px(360.))
                .overflow_y_scroll()
                .pr(px(super::scrollbar::GUTTER))
                .text_color(rgb(palette().muted))
                .text_size(px(15.))
                .line_height(px(22.))
                .child(
                    div()
                        .debug_selector(|| "information-dialog-copy".into())
                        .child(body),
                ),
        ))
        .child(super::dialog::footer().child(close));
    super::dialog::present("information-dialog-entrance", panel, cx)
        .debug_selector(|| "information-dialog-overlay".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[cfg(feature = "ui-tests")]
    struct TypographyView;

    #[cfg(feature = "ui-tests")]
    impl Render for TypographyView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(220.))
                .font_family(FONT)
                .text_size(px(HEADLINE_TEXT_SIZE))
                .line_height(px(HEADLINE_LINE_HEIGHT))
                .font_weight(FontWeight::SEMIBOLD)
                .flex()
                .flex_col()
                .child(field(
                    "Stable label",
                    "First line\nSecond line gjpqy ÆØÅ",
                    div().h(px(CONTROL_HEIGHT)).into_any_element(),
                ))
                .child(
                    status("First line\nSecond line", Tone::Neutral)
                        .debug_selector(|| "typography-status".into()),
                )
        }
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn field_and_status_leading_does_not_inherit_ancestor_headings(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| TypographyView);
        cx.run_until_parked();
        assert_eq!(
            cx.debug_bounds("field-label-Stable label")
                .unwrap()
                .size
                .height,
            px(BODY_LINE_HEIGHT)
        );
        assert_eq!(
            cx.debug_bounds("field-hint-Stable label")
                .unwrap()
                .size
                .height,
            px(2. * HELP_LINE_HEIGHT)
        );
        assert_eq!(
            cx.debug_bounds("typography-status").unwrap().size.height,
            px(2. * HELP_LINE_HEIGHT + 8.)
        );
    }

    const ACTION_STYLES: [ActionStyle; 14] = [
        ActionStyle::Primary,
        ActionStyle::Secondary,
        ActionStyle::Quiet,
        ActionStyle::Danger,
        ActionStyle::Navigation { selected: false },
        ActionStyle::Navigation { selected: true },
        ActionStyle::Choice { selected: false },
        ActionStyle::Choice { selected: true },
        ActionStyle::Tab { selected: false },
        ActionStyle::Tab { selected: true },
        ActionStyle::Segment { selected: false },
        ActionStyle::Segment { selected: true },
        ActionStyle::Anchor { selected: false },
        ActionStyle::Anchor { selected: true },
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
            .text_color(rgb(palette().surface));
        let resting = Style::default().refined(old.clone());
        let old_focus = StyleRefinement::default()
            .border_color(rgb(palette().ink))
            .bg(rgb(palette().selected))
            .text_color(rgb(palette().ink));
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
                    assert_eq!(merged.border_color, resting.border_color);
                    assert_eq!(merged.box_shadow.len(), 2);
                    assert_eq!(merged.box_shadow[0].spread_radius, px(3.));
                    assert_eq!(merged.box_shadow[1].spread_radius, px(1.));
                    let gap = match variant {
                        ActionStyle::Navigation { .. } => palette().rail,
                        _ => palette().surface,
                    };
                    assert_eq!(merged.box_shadow[1].color, rgb(gap).into());
                    let ring = match variant {
                        ActionStyle::Navigation { .. } => palette().rail_ink,
                        _ => palette().accent,
                    };
                    assert_eq!(merged.box_shadow[0].color, rgb(ring).into());
                    for shadow in &merged.box_shadow {
                        assert_eq!(shadow.offset, point(px(0.), px(0.)));
                        assert_eq!(shadow.blur_radius, px(0.));
                    }
                    assert!(contrast(ring, gap) >= 3.);
                }
                if matches!(variant, ActionStyle::Primary) {
                    assert_eq!(
                        inherited_text(&merged).color,
                        rgb(palette().on_accent).into()
                    );
                    let expected = if states & 6 != 0 {
                        palette().accent_hover
                    } else {
                        palette().accent
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
            Some(rgb(palette().action_surface).into())
        );
        assert_ne!(palette().action_surface, palette().surface);
        assert_eq!(
            secondary.border_color,
            Some(rgb(palette().control_line).into())
        );
        assert!(contrast(palette().ink, palette().action_surface) >= 4.5);
        assert!(contrast(palette().ink, palette().action_hover) >= 4.5);
        let selection = merged_base(selection_base(false));
        assert_eq!(selection.padding.top, px(CHOICE_INSET).into());
        assert_eq!(selection.padding.bottom, px(CHOICE_INSET).into());
        assert_eq!(selection.flex_shrink, 0.);
    }

    #[test]
    fn raised_actions_leave_form_choices_and_unselected_navigation_grounded() {
        for variant in [
            ActionStyle::Primary,
            ActionStyle::Secondary,
            ActionStyle::Danger,
        ] {
            let action = merged_base(action_base(variant));
            assert_eq!(action.box_shadow.len(), 1);
            assert!(action.box_shadow[0].blur_radius > px(0.));
        }
        for selected in [false, true] {
            assert!(merged_base(selection_base(selected)).box_shadow.is_empty());
            for variant in [
                ActionStyle::Tab { selected },
                ActionStyle::Segment { selected },
                ActionStyle::Anchor { selected },
                ActionStyle::Quiet,
            ] {
                assert!(
                    merged_base(action_base(variant)).box_shadow.is_empty(),
                    "{variant:?}"
                );
            }
        }
        assert!(
            merged_base(action_base(ActionStyle::Navigation { selected: false }))
                .box_shadow
                .is_empty()
        );
    }

    #[test]
    fn semantic_selection_ink_is_readable_in_both_themes() {
        for dark in [false, true] {
            set_dark(dark);
            for variant in ACTION_STYLES {
                let (background, ink, _, hover) = action_colors(variant);
                assert!(contrast(ink, background) >= 4.5, "{variant:?}, dark={dark}");
                assert!(
                    contrast(ink, hover) >= 4.5,
                    "hover {variant:?}, dark={dark}"
                );
            }
        }
        set_dark(false);
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
                assert_eq!(focused.box_shadow[0].color, rgb(palette().rail_ink).into());
                assert_eq!(focused.border_color, base.border_color);
                assert!(contrast(palette().rail_ink, palette().rail) >= 3.);
                assert!(contrast(palette().rail_ink, palette().rail_hover) >= 3.);
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
            Some(rgb(palette().surface).into())
        );
        assert_ne!(notice.background, navigation.background);
        assert_eq!(inherited_text(&notice).color, rgb(palette().ink).into());
        assert_eq!(notice.align_items, Some(AlignItems::Center));

        let error = merged_base(feedback(
            Some("Could not save. Try again.".into()),
            None,
            div().into_any_element(),
        ));
        assert_eq!(
            error.background.as_ref().unwrap().color(),
            Some(rgb(palette().error_surface).into())
        );
        assert_eq!(inherited_text(&error).color, rgb(palette().error).into());
    }

    #[test]
    fn compact_activity_is_readable_and_not_selection_or_completion_paint() {
        let row = merged_base(activity("Opening project…"));
        assert_eq!(inherited_text(&row).color, rgb(palette().ink).into());
        assert_eq!(inherited_text(&row).font_weight, FontWeight::MEDIUM);
        let surface = merged_base(activity_notice("Opening project…"));
        assert_eq!(surface.min_size.height, px(CONTROL_HEIGHT).into());
        assert_eq!(
            surface.background.as_ref().unwrap().color(),
            Some(rgb(palette().surface).into())
        );
        assert_ne!(
            surface.background.as_ref().unwrap().color(),
            Some(rgb(palette().selected).into())
        );
        assert!(contrast(palette().accent, palette().surface) >= 3.);
        assert!(contrast(palette().control_line, palette().surface) >= 3.);
        for edge in [16., 32.] {
            let bounds = Bounds::new(point(px(12.), px(20.)), size(px(edge), px(edge)));
            for frame in 0..120 {
                assert!(activity_arc(bounds, frame as f32 / 120., 0.75).is_ok());
            }
            assert!(activity_arc(bounds, 0., 1.).is_ok());
        }
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod ui_tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;

    const SELECTORS: [&str; 5] = ["primary", "secondary", "danger", "radio", "checkbox"];

    struct MissingImageView(ScrollHandle);

    impl Render for MissingImageView {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                .font_family(FONT)
                .child(image_placeholder(
                "Screenshot unavailable",
                "A deliberately long diagnostic must scroll without losing the recovery message. "
                    .repeat(20),
                "Use Refresh to read the saved screenshot again. Your baseline is unchanged.",
                Tone::Error,
                &self.0,
                f32::from(window.viewport_size().width),
            ))
        }
    }

    #[gpui::test]
    fn missing_image_message_scrolls_within_a_fixed_frame(cx: &mut TestAppContext) {
        let (_, cx) = cx.add_window_view(|_, _| MissingImageView(ScrollHandle::new()));
        cx.simulate_resize(size(px(360.), px(240.)));
        cx.run_until_parked();
        let frame = cx.debug_bounds("screenshot-placeholder").unwrap();
        assert_eq!(frame.size, size(px(360.), px(240.)));
        let viewport = cx.debug_bounds("screenshot-placeholder-scroll").unwrap();
        let before = cx.debug_bounds("screenshot-placeholder-help").unwrap();
        assert!(before.bottom() > viewport.bottom());
        cx.simulate_event(ScrollWheelEvent {
            position: viewport.center(),
            delta: ScrollDelta::Pixels(point(px(0.), px(-10000.))),
            ..Default::default()
        });
        cx.run_until_parked();
        let help = cx.debug_bounds("screenshot-placeholder-help").unwrap();
        assert!(
            help.top() >= viewport.top() && help.bottom() <= viewport.bottom(),
            "{help:?} outside {viewport:?}"
        );
        assert_eq!(cx.debug_bounds("screenshot-placeholder").unwrap(), frame);
    }

    struct LoadingImageView(ScrollHandle);

    impl Render for LoadingImageView {
        fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .flex()
                .flex_col()
                .font_family(FONT)
                .child(image_loading(
                    "Loading screenshot…",
                    "Reading the selected screenshot from disk. The page list stays available.",
                    &self.0,
                    f32::from(window.viewport_size().width),
                ))
        }
    }

    #[gpui::test]
    fn screenshot_loading_centers_or_scrolls_without_changing_its_frame(cx: &mut TestAppContext) {
        cx.update(|cx| cx.set_global(ReducedMotion(true)));
        for (width, height) in [(600., 400.), (360., 240.), (240., 120.)] {
            let (_, cx) = cx.add_window_view(|_, _| LoadingImageView(ScrollHandle::new()));
            cx.simulate_resize(size(px(width), px(height)));
            cx.run_until_parked();
            let frame = cx.debug_bounds("screenshot-loading").unwrap();
            assert_eq!(frame.size, size(px(width), px(height)));
            let viewport = cx.debug_bounds("screenshot-placeholder-scroll").unwrap();
            let message = cx.debug_bounds("screenshot-placeholder-message").unwrap();
            let icon = cx.debug_bounds("activity-indicator").unwrap();
            assert_eq!(icon.size, size(px(32.), px(32.)));
            assert!(message.size.width <= px(440.));
            if message.size.height + px(48.) <= viewport.size.height {
                assert!((f32::from(message.center().y - viewport.center().y)).abs() < 1.);
                assert!(message.top() >= viewport.top());
                assert!(message.bottom() <= viewport.bottom());
            } else {
                assert!(message.top() >= viewport.top() + px(24.));
                cx.simulate_event(ScrollWheelEvent {
                    position: viewport.center(),
                    delta: ScrollDelta::Pixels(point(px(0.), px(-10000.))),
                    ..Default::default()
                });
                cx.run_until_parked();
                let help = cx.debug_bounds("screenshot-loading-explanation").unwrap();
                assert!(help.top() >= viewport.top() && help.bottom() <= viewport.bottom());
            }
            cx.update(|window, cx| {
                assert_eq!(
                    activity_frame(window, cx, "loading-image-test"),
                    (0., false)
                );
            });
            assert_eq!(cx.debug_bounds("screenshot-loading").unwrap(), frame);
            assert!(cx.debug_bounds("progress-track").is_none());
        }
    }

    struct NavigationView {
        focus: FocusHandle,
        selected: bool,
    }

    impl Render for NavigationView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .w(px(192.))
                .p(px(14.))
                .bg(rgb(palette().rail))
                .child(action(
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
    fn page_rows_never_open_help_bubbles_on_hover(cx: &mut TestAppContext) {
        for (width, height) in [(1280., 800.), (820., 600.)] {
            for (kind, enabled) in [
                (RowKind::Result, true),
                (RowKind::Result, false),
                (RowKind::Discovery, true),
                (RowKind::Discovery, false),
                (RowKind::Page, true),
                (RowKind::Page, false),
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
                assert!(cx.debug_bounds("page-row-hint").is_none());
                assert_eq!(row, cx.debug_bounds("page-row-hint-test").unwrap());
            }
        }
    }

    #[gpui::test]
    fn long_page_addresses_remain_quiet_and_keep_row_geometry(cx: &mut TestAppContext) {
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
            assert!(cx.debug_bounds("page-row-hint").is_none());
            assert_eq!(row, cx.debug_bounds("page-row-hint-test").unwrap());
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

    struct DisclosureView {
        focus: FocusHandle,
        open: bool,
        enabled: bool,
        width: f32,
    }

    impl Render for DisclosureView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(self.width)).child(disclosure_group("probe",
                disclosure_toggle("probe", "An optional section with a deliberately long wrapping title", &self.focus, self.open, self.enabled, false).into_any_element(),
                "This explanation belongs to the section, even when its contents are hidden.",
                self.open.then(|| div().debug_selector(|| "disclosure-probe-content".into()).w_full().min_w_0().child("Related content stays under its heading and wraps inside the available width.").into_any_element()),
            ))
        }
    }

    #[gpui::test]
    fn disclosure_long_titles_keep_state_text_and_indented_content_in_bounds(
        cx: &mut TestAppContext,
    ) {
        let (view, cx) = cx.add_window_view(|_, cx| DisclosureView {
            focus: cx.focus_handle(),
            open: true,
            enabled: true,
            width: 240.,
        });
        for width in [240., 294., 552., 680.] {
            view.update(cx, |v, cx| {
                v.width = width;
                cx.notify();
            });
            cx.run_until_parked();
            let header = cx.debug_bounds("action-probe").unwrap();
            let title = cx.debug_bounds("disclosure-title-probe").unwrap();
            let state = cx.debug_bounds("disclosure-state-probe").unwrap();
            let content = cx.debug_bounds("disclosure-probe-content").unwrap();
            assert_eq!(header.size.width, px(width));
            assert!(header.size.height >= px(CONTROL_HEIGHT));
            assert!(title.right() < state.left());
            assert!(state.right() <= header.right());
            assert!((content.left() - title.left()).abs() <= px(1.));
            assert!(content.right() <= header.right());
            for enabled in [false, true] {
                view.update(cx, |v, cx| {
                    v.enabled = enabled;
                    cx.notify();
                });
                cx.update(|window, cx| view.read(cx).focus.focus(window));
                cx.run_until_parked();
                assert_eq!(header, cx.debug_bounds("action-probe").unwrap());
            }
        }
    }

    struct ControlsView {
        focus: Vec<FocusHandle>,
        selected: bool,
        enabled: bool,
        width: f32,
    }

    #[gpui::test]
    fn icon_action_labels_keep_intrinsic_width_in_wrapping_rows(cx: &mut TestAppContext) {
        struct Host {
            focus: FocusHandle,
            selected: bool,
            enabled: bool,
        }
        impl Render for Host {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().w(px(680.)).flex().flex_col().items_start().gap_3().children([
                    ActionStyle::Primary,ActionStyle::Secondary,ActionStyle::Danger,
                    ActionStyle::Navigation {selected:self.selected},ActionStyle::Choice {selected:self.selected}
                ].into_iter().enumerate().map(|(i,style)|div().flex().flex_wrap().items_start().gap_2()
                    .child(action_with_icon(SharedString::from(format!("natural-{i}")),"Repair browser…",super::super::toolbar::ToolIcon::Download,&self.focus,style,self.enabled))
                    .child(action_with_icon(SharedString::from(format!("wrapped-{i}")),
                        "Review this deliberately long action label with the complete explanation preserved",super::super::toolbar::ToolIcon::Details,&self.focus,style,self.enabled).max_w(px(260.)))))
            }
        }
        let (host, cx) = cx.add_window_view(|_, cx| Host {
            focus: cx.focus_handle(),
            selected: false,
            enabled: true,
        });
        let mut initial = vec![];
        for i in 0..5 {
            for kind in ["natural", "wrapped"] {
                let id = format!("{kind}-{i}");
                let frame = cx.debug_bounds(format!("action-{id}").leak()).unwrap();
                let label = cx
                    .debug_bounds(format!("action-label-{id}").leak())
                    .unwrap();
                assert!(label.size.width > px(40.), "{id}: {label:?}");
                assert!(
                    label.left() > frame.left() + px(30.)
                        && label.right() <= frame.right() - px(8.)
                );
                assert!(
                    label.top() >= frame.top() + px(8.)
                        && label.bottom() <= frame.bottom() - px(8.)
                );
                if kind == "natural" {
                    assert_eq!(label.size.height, px(CONTROL_LINE_HEIGHT));
                    assert_eq!(frame.size.height, px(CONTROL_HEIGHT));
                } else {
                    assert!(label.size.height > px(CONTROL_LINE_HEIGHT));
                    assert!(frame.size.height > px(CONTROL_HEIGHT));
                }
                initial.push((id, frame));
            }
        }
        cx.update(|w, cx| host.read(cx).focus.focus(w));
        cx.run_until_parked();
        for (id, before) in &initial {
            assert_eq!(
                cx.debug_bounds(format!("action-{id}").leak()).unwrap(),
                *before
            );
        }
        for (selected, enabled) in [(true, true), (true, false), (false, false)] {
            host.update(cx, |s, cx| {
                s.selected = selected;
                s.enabled = enabled;
                cx.notify();
            });
            cx.run_until_parked();
            for (id, before) in &initial {
                assert_eq!(
                    cx.debug_bounds(format!("action-{id}").leak()).unwrap(),
                    *before
                );
            }
        }
    }

    #[gpui::test]
    fn labelled_icon_actions_wrap_and_keep_geometry_in_every_action_style(cx: &mut TestAppContext) {
        struct Host {
            focus: Vec<FocusHandle>,
            selected: bool,
            enabled: bool,
        }
        const IDS: [&str; 5] = [
            "icon-primary",
            "icon-secondary",
            "icon-danger",
            "icon-navigation",
            "icon-choice",
        ];
        impl Render for Host {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().w(px(220.)).flex().flex_col().gap_3().children(
                    [
                        ActionStyle::Primary,
                        ActionStyle::Secondary,
                        ActionStyle::Danger,
                        ActionStyle::Navigation {
                            selected: self.selected,
                        },
                        ActionStyle::Choice {
                            selected: self.selected,
                        },
                    ]
                    .into_iter()
                    .enumerate()
                    .map(|(i, style)| {
                        action_with_icon(
                            IDS[i],
                            "Review this deliberately long action label with a complete explanation that must wrap",
                            super::super::toolbar::ToolIcon::Note,
                            &self.focus[i],
                            style,
                            self.enabled,
                        )
                    }),
                )
            }
        }
        let (host, cx) = cx.add_window_view(|_, cx| Host {
            focus: (0..5).map(|_| cx.focus_handle()).collect(),
            selected: false,
            enabled: true,
        });
        let mut initial = vec![];
        for id in IDS {
            let frame = cx.debug_bounds(format!("action-{id}").leak()).unwrap();
            let label = cx
                .debug_bounds(format!("action-label-{id}").leak())
                .unwrap();
            let glyph = cx.debug_bounds(format!("action-icon-{id}").leak()).unwrap();
            assert!(frame.size.height >= px(40.) && frame.size.width <= px(220.));
            assert!(
                label.size.height > px(CONTROL_LINE_HEIGHT),
                "{id}: frame={frame:?}, label={label:?}, glyph={glyph:?}"
            );
            assert!(
                glyph.left() >= frame.left() + px(8.) && glyph.right() + px(8.) <= label.left()
            );
            assert!(
                label.right() <= frame.right() - px(8.)
                    && label.bottom() <= frame.bottom() - px(8.)
            );
            initial.push(frame);
        }
        for (selected, enabled) in [(true, true), (true, false), (false, false)] {
            host.update(cx, |s, cx| {
                s.selected = selected;
                s.enabled = enabled;
                cx.notify();
            });
            cx.run_until_parked();
            for (id, before) in IDS.into_iter().zip(&initial) {
                assert_eq!(
                    cx.debug_bounds(format!("action-{id}").leak()).unwrap(),
                    *before
                );
            }
        }
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
