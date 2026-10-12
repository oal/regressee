// SPDX-License-Identifier: MPL-2.0
//! One small native stroke family. Callers always pair these marks with words.
use gpui::{prelude::*, *};

#[derive(Clone, Copy, Debug)]
pub enum StatusIcon {
    Information,
    Saved,
    Warning,
    Error,
    Skipped,
    Pending,
    Paused,
    Pages,
    Clock,
}

fn icon_paths(icon: StatusIcon, bounds: Bounds<Pixels>) -> Vec<Path<Pixels>> {
    let at = |x: f32, y: f32| bounds.origin + point(bounds.size.width * x, bounds.size.height * y);
    let mut paths = Vec::new();
    let mut line = |points: &[(f32, f32)]| {
        let mut path = PathBuilder::stroke(px(1.8));
        path.move_to(at(points[0].0, points[0].1));
        for &(x, y) in &points[1..] {
            path.line_to(at(x, y));
        }
        if let Ok(path) = path.build() {
            paths.push(path);
        }
    };
    match icon {
        StatusIcon::Warning => {
            line(&[(0.5, 0.12), (0.91, 0.85), (0.09, 0.85), (0.5, 0.12)]);
            line(&[(0.5, 0.35), (0.5, 0.57)]);
            line(&[(0.5, 0.67), (0.5, 0.72)]);
        }
        StatusIcon::Saved => line(&[(0.16, 0.5), (0.4, 0.74), (0.86, 0.25)]),
        StatusIcon::Skipped => {
            line(&[(0.15, 0.25), (0.6, 0.5), (0.15, 0.75)]);
            line(&[(0.81, 0.21), (0.81, 0.79)]);
        }
        StatusIcon::Paused => {
            line(&[(0.34, 0.2), (0.34, 0.8)]);
            line(&[(0.66, 0.2), (0.66, 0.8)]);
        }
        StatusIcon::Pages => {
            line(&[
                (0.24, 0.12),
                (0.62, 0.12),
                (0.82, 0.32),
                (0.82, 0.87),
                (0.24, 0.87),
                (0.24, 0.12),
            ]);
            line(&[(0.6, 0.12), (0.6, 0.34), (0.82, 0.34)]);
            line(&[(0.37, 0.52), (0.68, 0.52)]);
            line(&[(0.37, 0.68), (0.68, 0.68)]);
        }
        StatusIcon::Information | StatusIcon::Error | StatusIcon::Clock | StatusIcon::Pending => {
            // Native circular curves, not font glyphs or emoji.
            let mut circle = PathBuilder::stroke(px(1.8));
            circle.move_to(at(0.5, 0.1));
            circle.cubic_bezier_to(at(0.9, 0.5), at(0.72, 0.1), at(0.9, 0.28));
            circle.cubic_bezier_to(at(0.5, 0.9), at(0.9, 0.72), at(0.72, 0.9));
            circle.cubic_bezier_to(at(0.1, 0.5), at(0.28, 0.9), at(0.1, 0.72));
            circle.cubic_bezier_to(at(0.5, 0.1), at(0.1, 0.28), at(0.28, 0.1));
            // Add the inner strokes before closing the borrow held by `line`.
            match icon {
                StatusIcon::Information => {
                    line(&[(0.5, 0.3), (0.5, 0.35)]);
                    line(&[(0.5, 0.47), (0.5, 0.73)]);
                }
                StatusIcon::Error => {
                    line(&[(0.34, 0.34), (0.66, 0.66)]);
                    line(&[(0.66, 0.34), (0.34, 0.66)]);
                }
                StatusIcon::Pending => line(&[(0.3, 0.5), (0.7, 0.5)]),
                _ => line(&[(0.5, 0.27), (0.5, 0.52), (0.7, 0.62)]),
            }
            if let Ok(path) = circle.build() {
                paths.push(path);
            }
        }
    }
    paths
}

pub fn status_icon(icon: StatusIcon, color: u32, edge: f32) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            for path in icon_paths(icon, bounds) {
                window.paint_path(path, rgb(color));
            }
        },
    )
    .size(px(edge))
    .flex_shrink_0()
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    #[test]
    fn every_status_mark_has_native_geometry() {
        let bounds = Bounds::new(point(px(0.), px(0.)), size(px(18.), px(18.)));
        for icon in [
            StatusIcon::Information,
            StatusIcon::Saved,
            StatusIcon::Warning,
            StatusIcon::Error,
            StatusIcon::Skipped,
            StatusIcon::Pending,
            StatusIcon::Paused,
            StatusIcon::Pages,
            StatusIcon::Clock,
        ] {
            assert!(!icon_paths(icon, bounds).is_empty(), "{icon:?}");
        }
    }
}
