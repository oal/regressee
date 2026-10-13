// SPDX-License-Identifier: MPL-2.0
//! Connected native progress. Gating affects interaction, never location paint.
use super::{controls::focus_outline, theme::*};
use gpui::{prelude::*, *};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StepState {
    Completed,
    Current,
    Upcoming,
}
impl StepState {
    pub fn at(index: usize, current: usize) -> Self {
        match index.cmp(&current) {
            std::cmp::Ordering::Less => Self::Completed,
            std::cmp::Ordering::Equal => Self::Current,
            std::cmp::Ordering::Greater => Self::Upcoming,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Completed => "Completed",
            Self::Current => "Current",
            Self::Upcoming => "Upcoming",
        }
    }
}

/// Callers supply guarded activation and focus order. Noninteractive progress
/// remains legible, with no disabled-button opacity or hover/focus affordance.
pub fn step(
    id: SharedString,
    number: usize,
    label: &'static str,
    state: StepState,
    focus: &FocusHandle,
    interactive: bool,
) -> Stateful<Div> {
    let current = state == StepState::Current;
    div()
        .id(id.clone())
        .debug_selector(move || format!("action-{id}"))
        .flex_1()
        .min_w_0()
        .min_h(px(52.))
        .p_2()
        .flex()
        .items_center()
        .gap_2()
        .rounded(px(4.))
        .bg(rgb(if current { SELECTED } else { GROUND }))
        .text_color(rgb(if current { ACCENT_HOVER } else { INK }))
        .when(interactive, |el| {
            el.track_focus(focus)
                .cursor_pointer()
                .focus(|s| focus_outline(s, INK, INK))
                .hover(|s| s.bg(rgb(ACTION_HOVER)))
        })
        .child(
            div()
                .size(px(24.))
                .flex_shrink_0()
                .rounded(px(12.))
                .border_1()
                .border_color(rgb(if current { ACCENT } else { CONTROL_LINE }))
                .bg(rgb(if current { ACCENT } else { SURFACE }))
                .text_color(rgb(if current { SURFACE } else { INK }))
                .text_size(px(14.))
                .flex()
                .items_center()
                .justify_center()
                .child(number.to_string()),
        )
        .child(
            div()
                .flex_1()
                .min_w_0()
                .flex()
                .flex_col()
                .text_size(px(14.))
                .line_height(px(18.))
                .child(div().font_weight(FontWeight::MEDIUM).child(label))
                .child(div().text_size(px(13.)).child(state.label())),
        )
}

pub fn connector() -> Div {
    div()
        .w(px(16.))
        .h(px(1.))
        .flex_shrink_0()
        .bg(rgb(CONTROL_LINE))
}
