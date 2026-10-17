// SPDX-License-Identifier: MPL-2.0
//! Shared trigger-relative native selector geometry (View, Zoom).
use super::theme::CONTROL_HEIGHT;
use gpui::{Bounds, Pixels, Size, point, px, size};

pub fn popup_bounds(
    trigger: Bounds<Pixels>,
    viewport: Size<Pixels>,
    rows: usize,
    desired_width: f32,
) -> Bounds<Pixels> {
    let margin = px(8.);
    let gap = px(4.);
    let below = (viewport.height - margin - trigger.bottom() - gap).max(px(0.));
    let above = (trigger.top() - margin - gap).max(px(0.));
    let desired = px(rows as f32 * CONTROL_HEIGHT + 2.);
    let open_above = below < desired && above > below;
    let height = desired
        .min(if open_above { above } else { below })
        .min((viewport.height - margin * 2.).max(px(0.)));
    let width = px(desired_width).min((viewport.width - margin * 2.).max(px(0.)));
    let x = trigger
        .left()
        .min(viewport.width - margin - width)
        .max(margin);
    let y = if open_above {
        trigger.top() - gap - height
    } else {
        trigger.bottom() + gap
    };
    Bounds::new(
        point(x, y.min(viewport.height - margin - height).max(margin)),
        size(width, height),
    )
}
