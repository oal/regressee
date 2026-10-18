// SPDX-License-Identifier: MPL-2.0
//! Shared trigger-relative native selector geometry (View, Zoom).
use super::theme::CONTROL_HEIGHT;
use gpui::{Bounds, ElementId, Pixels, Size, point, px, size};

/// Advance only on a logical closed-to-open transition, never on a repaint,
/// highlight or filter change. A generation also restarts a rapid reopen before
/// the closed state has had a chance to paint.
#[derive(Default)]
pub struct OpenCycle(u64);

impl OpenCycle {
    pub fn opened(&mut self) {
        self.0 = self.0.wrapping_add(1);
    }

    pub fn id(&self, name: &'static str) -> ElementId {
        (name, self.0).into()
    }
}

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entrance_identity_changes_only_on_open_including_unpainted_reopens() {
        let mut cycle = OpenCycle::default();
        cycle.opened();
        let first = cycle.id("menu");
        assert_eq!(cycle.id("menu"), first);
        // Dismissal retains no UI. Reopening, even without an intervening frame,
        // gets its own entrance; subsequent selection/highlight paints do not.
        cycle.opened();
        assert_ne!(cycle.id("menu"), first);
        assert_eq!(cycle.id("menu"), cycle.id("menu"));
        assert_ne!(cycle.id("menu"), cycle.id("other-menu"));
    }
}
