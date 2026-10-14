// SPDX-License-Identifier: MPL-2.0
//! An overflow-only native scrollbar attached to an existing scroll owner.
use super::theme::*;
use gpui::{prelude::*, *};
use std::{cell::Cell, rc::Rc};

pub const GUTTER: f32 = 16.;
const INSET: f32 = 4.;
const MIN_THUMB: f32 = 32.;

#[derive(Clone, Copy, Debug)]
pub struct ScrollbarGeometry {
    pub track: Bounds<Pixels>,
    pub thumb: Bounds<Pixels>,
    overflow: f32,
    travel: f32,
}
impl ScrollbarGeometry {
    pub fn new(bounds: Bounds<Pixels>, viewport: f32, overflow: f32, offset: f32) -> Option<Self> {
        let height = f32::from(bounds.size.height) - INSET * 2.;
        if overflow <= 1. || height <= MIN_THUMB || viewport <= 0. {
            return None;
        }
        let track = Bounds::new(
            bounds.origin + point(px(INSET), px(INSET)),
            size(bounds.size.width - px(INSET * 2.), px(height)),
        );
        let thumb_height = (height * viewport / (viewport + overflow)).clamp(MIN_THUMB, height);
        let travel = height - thumb_height;
        let thumb = Bounds::new(
            track.origin + point(px(0.), px((-offset / overflow).clamp(0., 1.) * travel)),
            size(track.size.width, px(thumb_height)),
        );
        Some(Self {
            track,
            thumb,
            overflow,
            travel,
        })
    }
    fn drag_offset(self, pointer: Pixels, grab: f32) -> Pixels {
        if self.travel <= 0. {
            return px(0.);
        }
        px(
            -((f32::from(pointer - self.track.top()) - grab) / self.travel).clamp(0., 1.)
                * self.overflow,
        )
    }
}

#[derive(Clone, Default)]
pub struct ScrollbarState {
    grab: Rc<Cell<Option<f32>>>,
}
impl ScrollbarState {
    pub fn cancel_drag(&self) {
        self.grab.set(None);
    }
    /// The gutter is reserved by the caller. Geometry is read after the scroll
    /// owner's layout, so first paint, resize and disclosures use actual overflow.
    pub fn element(&self, scroll: &ScrollHandle, owner: EntityId) -> impl IntoElement {
        let layout_scroll = scroll.clone();
        let scroll = scroll.clone();
        let grab = self.grab.clone();
        div()
            .id("settings-scrollbar")
            .debug_selector(|| "settings-scrollbar".into())
            .absolute()
            .top_0()
            .bottom_0()
            .right_0()
            .w(px(GUTTER))
            .child(
                canvas(
                    move |bounds, window, _| {
                        ScrollbarGeometry::new(
                            bounds,
                            f32::from(layout_scroll.bounds().size.height),
                            f32::from(layout_scroll.max_offset().height),
                            f32::from(layout_scroll.offset().y),
                        )
                        .map(|geometry| {
                            (
                                geometry,
                                window.insert_hitbox(bounds, HitboxBehavior::Normal),
                            )
                        })
                    },
                    move |bounds, geometry, window, _| {
                        let Some((geometry, hitbox)) = geometry else {
                            grab.set(None);
                            return;
                        };
                        window.paint_quad(fill(bounds, rgb(ACTION_SURFACE)));
                        window.paint_quad(fill(geometry.track, rgb(LINE)));
                        window.paint_quad(fill(
                            geometry.thumb,
                            rgb(if grab.get().is_some() || hitbox.is_hovered(window) {
                                INK
                            } else {
                                MUTED
                            }),
                        ));
                        window.set_cursor_style(CursorStyle::Arrow, &hitbox);
                        let scroll_down = scroll.clone();
                        let down_grab = grab.clone();
                        let down_hitbox = hitbox.clone();
                        window.on_mouse_event(move |e: &MouseDownEvent, phase, window, cx| {
                            if phase != DispatchPhase::Bubble
                                || e.button != MouseButton::Left
                                || !down_hitbox.is_hovered(window)
                            {
                                return;
                            }
                            if e.position.y >= geometry.thumb.top()
                                && e.position.y <= geometry.thumb.bottom()
                            {
                                down_grab.set(Some(f32::from(e.position.y - geometry.thumb.top())));
                            } else {
                                // Native-style track paging, without moving focus.
                                let direction = if e.position.y < geometry.thumb.top() {
                                    1.
                                } else {
                                    -1.
                                };
                                let offset = scroll_down.offset();
                                let page = scroll_down.bounds().size.height * 0.9;
                                scroll_down.set_offset(point(
                                    offset.x,
                                    (offset.y + page * direction)
                                        .clamp(-scroll_down.max_offset().height, px(0.)),
                                ));
                            }
                            cx.stop_propagation();
                            cx.notify(owner);
                        });
                        let move_grab = grab.clone();
                        let scroll_move = scroll.clone();
                        window.on_mouse_event(move |e: &MouseMoveEvent, phase, _, cx| {
                            if phase != DispatchPhase::Capture {
                                return;
                            }
                            let Some(distance) = move_grab.get() else {
                                return;
                            };
                            if e.pressed_button != Some(MouseButton::Left) {
                                move_grab.set(None);
                            } else if let Some(current) = ScrollbarGeometry::new(
                                bounds,
                                f32::from(scroll_move.bounds().size.height),
                                f32::from(scroll_move.max_offset().height),
                                f32::from(scroll_move.offset().y),
                            ) {
                                scroll_move.set_offset(point(
                                    scroll_move.offset().x,
                                    current.drag_offset(e.position.y, distance),
                                ));
                            }
                            cx.stop_propagation();
                            cx.notify(owner);
                        });
                        window.on_mouse_event(move |e: &MouseUpEvent, phase, _, cx| {
                            if phase == DispatchPhase::Capture
                                && e.button == MouseButton::Left
                                && grab.take().is_some()
                            {
                                cx.stop_propagation();
                                cx.notify(owner);
                            }
                        });
                        window.on_mouse_event(move |e: &ScrollWheelEvent, phase, window, cx| {
                            if phase == DispatchPhase::Bubble && hitbox.is_hovered(window) {
                                let offset = scroll.offset();
                                let next = (offset.y + e.delta.pixel_delta(px(20.)).y)
                                    .clamp(-scroll.max_offset().height, px(0.));
                                if next != offset.y {
                                    scroll.set_offset(point(offset.x, next));
                                    cx.stop_propagation();
                                    cx.notify(owner);
                                }
                            }
                        });
                    },
                )
                .size_full(),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;
    #[test]
    fn scrollbar_geometry_tracks_overflow_and_clamps_dragging() {
        let bounds = Bounds::new(point(px(0.), px(100.)), size(px(GUTTER), px(400.)));
        assert!(ScrollbarGeometry::new(bounds, 400., 0., 0.).is_none());
        let top = ScrollbarGeometry::new(bounds, 400., 1200., 0.).unwrap();
        let bottom = ScrollbarGeometry::new(bounds, 400., 1200., -1200.).unwrap();
        assert_eq!(top.thumb.top(), top.track.top());
        assert_eq!(bottom.thumb.bottom(), bottom.track.bottom());
        assert_eq!(top.drag_offset(px(-100.), 8.), px(0.));
        assert_eq!(top.drag_offset(px(1000.), 8.), px(-1200.));
        assert!(
            ScrollbarGeometry::new(bounds, 400., 100_000., 0.)
                .unwrap()
                .thumb
                .size
                .height
                >= px(MIN_THUMB)
        );
    }
}
