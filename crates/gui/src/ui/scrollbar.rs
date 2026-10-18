// SPDX-License-Identifier: MPL-2.0
//! An overflow-only native scrollbar attached to an existing scroll owner.
use super::theme::*;
use gpui::{prelude::*, *};
use std::{cell::Cell, rc::Rc};

pub const GUTTER: f32 = 16.;
const INSET: f32 = 4.;
const MIN_THUMB: f32 = 32.;

/// Decorate a scroll owner without changing its layout, focus order or offset.
/// Read geometry after prepaint: virtual lists, disclosures and resizing all use
/// the actual current extent, including their very first frame.
pub fn vertical(
    id: impl Into<ElementId>,
    scroll: &ScrollHandle,
    content: impl IntoElement + 'static,
) -> Scrollbars {
    let mut bars = scrollable_element(id, move |_| content.into_any_element());
    bars.handle = Some(scroll.clone());
    bars
}

/// For small, otherwise untracked reading areas. The handle lives in element
/// state, not a render-local variable, so repainting never resets the position.
pub fn scrollable(id: impl Into<ElementId>, content: Stateful<Div>) -> Scrollbars {
    scrollable_element(id, move |handle| {
        content.track_scroll(&handle).into_any_element()
    })
}

pub fn both(
    id: impl Into<ElementId>,
    content: impl IntoElement + 'static,
    metrics: impl Fn(Bounds<Pixels>, &App) -> ScrollMetrics + 'static,
    change: impl Fn(Point<Pixels>, &mut Window, &mut App) + 'static,
) -> Scrollbars {
    let mut bars = scrollable_element(id, move |_| content.into_any_element());
    bars.metrics = Some(Box::new(metrics));
    bars.change = Some(Rc::new(change));
    bars.horizontal = true;
    bars
}

type MetricsFn = Box<dyn Fn(Bounds<Pixels>, &App) -> ScrollMetrics>;
type ChangeFn = Rc<dyn Fn(Point<Pixels>, &mut Window, &mut App)>;
type DragState = Rc<Cell<Option<(Axis, f32, u64)>>>;

#[derive(Clone, Copy)]
pub struct ScrollMetrics {
    pub viewport: Bounds<Pixels>,
    pub overflow: Size<Pixels>,
    /// GPUI convention: zero at the start, negative toward the end.
    pub offset: Point<Pixels>,
}

#[derive(Default)]
struct ElementState {
    handle: ScrollHandle,
    drag: DragState,
    hovered: Rc<Cell<bool>>,
}

pub struct Scrollbars {
    id: ElementId,
    build: Option<Box<dyn FnOnce(ScrollHandle) -> AnyElement>>,
    handle: Option<ScrollHandle>,
    metrics: Option<MetricsFn>,
    change: Option<ChangeFn>,
    horizontal: bool,
    drag: Option<DragState>,
}

fn scrollable_element(
    id: impl Into<ElementId>,
    build: impl FnOnce(ScrollHandle) -> AnyElement + 'static,
) -> Scrollbars {
    Scrollbars {
        id: id.into(),
        build: Some(Box::new(build)),
        handle: None,
        metrics: None,
        change: None,
        horizontal: false,
        drag: None,
    }
}

#[derive(Clone, Copy, PartialEq)]
pub enum Axis {
    Vertical,
    Horizontal,
}
impl Axis {
    fn coordinate(self, point: Point<Pixels>) -> Pixels {
        match self {
            Self::Vertical => point.y,
            Self::Horizontal => point.x,
        }
    }
    fn transpose(self, bounds: Bounds<Pixels>) -> Bounds<Pixels> {
        match self {
            Self::Vertical => bounds,
            Self::Horizontal => Bounds::new(
                point(bounds.top(), bounds.left()),
                size(bounds.size.height, bounds.size.width),
            ),
        }
    }
}

pub struct BarLayout {
    metrics: ScrollMetrics,
    bars: Vec<(Axis, ScrollbarGeometry, Bounds<Pixels>, Hitbox)>,
}

pub struct ScrollbarLayoutState {
    child: AnyElement,
    drag: DragState,
    hovered: Rc<Cell<bool>>,
}

impl IntoElement for Scrollbars {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for Scrollbars {
    type RequestLayoutState = ScrollbarLayoutState;
    type PrepaintState = BarLayout;
    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        window.with_element_state(id.unwrap(), |state: Option<ElementState>, window| {
            let state = state.unwrap_or_default();
            let handle = self
                .handle
                .get_or_insert_with(|| state.handle.clone())
                .clone();
            if self.change.is_none() {
                let handle = handle.clone();
                self.change = Some(Rc::new(move |offset, window, _| {
                    handle.set_offset(offset);
                    window.refresh();
                }));
            }
            let mut child = self.build.take().unwrap()(handle);
            let layout = child.request_layout(window, cx);
            (
                (
                    layout,
                    ScrollbarLayoutState {
                        child,
                        drag: self.drag.clone().unwrap_or_else(|| state.drag.clone()),
                        hovered: state.hovered.clone(),
                    },
                ),
                state,
            )
        })
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        ScrollbarLayoutState { child, drag, .. }: &mut Self::RequestLayoutState,
        window: &mut Window,
        cx: &mut App,
    ) -> BarLayout {
        child.prepaint(window, cx);
        let metrics = if let Some(read) = &self.metrics {
            read(bounds, cx)
        } else {
            let handle = self.handle.as_ref().unwrap();
            ScrollMetrics {
                viewport: handle.bounds(),
                overflow: handle.max_offset(),
                offset: handle.offset(),
            }
        };
        let vertical = metrics.overflow.height > px(1.);
        let horizontal = self.horizontal && metrics.overflow.width > px(1.);
        let mut bars = Vec::new();
        for axis in [Axis::Vertical, Axis::Horizontal] {
            if !(if axis == Axis::Vertical {
                vertical
            } else {
                horizontal
            }) {
                continue;
            }
            let viewport = axis.transpose(metrics.viewport);
            let other = if axis == Axis::Vertical {
                horizontal
            } else {
                vertical
            };
            let gutter = Bounds::new(
                point(viewport.right() - px(GUTTER), viewport.top()),
                size(
                    px(GUTTER),
                    (viewport.size.height - px(if other { GUTTER } else { 0. })).max(px(0.)),
                ),
            );
            let overflow = match axis {
                Axis::Vertical => metrics.overflow.height,
                Axis::Horizontal => metrics.overflow.width,
            };
            if let Some(geometry) = ScrollbarGeometry::new(
                gutter,
                f32::from(viewport.size.height),
                f32::from(overflow),
                f32::from(axis.coordinate(metrics.offset)),
            ) {
                let hitbox = window.insert_hitbox(axis.transpose(gutter), HitboxBehavior::Normal);
                bars.push((axis, geometry, gutter, hitbox));
            }
        }
        if drag
            .get()
            .is_some_and(|(axis, _, _)| !bars.iter().any(|b| b.0 == axis))
        {
            drag.set(None);
        }
        BarLayout { metrics, bars }
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        _: Bounds<Pixels>,
        ScrollbarLayoutState {
            child,
            drag,
            hovered,
        }: &mut Self::RequestLayoutState,
        layout: &mut BarLayout,
        window: &mut Window,
        cx: &mut App,
    ) {
        child.paint(window, cx);
        for (axis, geometry, gutter, hitbox) in &layout.bars {
            let active = drag.get().is_some_and(|d| d.0 == *axis) || hitbox.is_hovered(window);
            // Overlay at rest, so screenshot edges and full-row selections keep
            // their pixels. A surface keyline keeps the thumb legible on dark
            // evidence. Hover reveals the track within the stable 16px hit area.
            if active {
                window.paint_quad(fill(axis.transpose(*gutter), rgb(palette().action_surface)));
                window.paint_quad(fill(axis.transpose(geometry.track), rgb(palette().line)));
            }
            window.paint_quad(quad(
                axis.transpose(geometry.thumb),
                px(4.),
                rgb(if active {
                    palette().ink
                } else {
                    palette().muted
                }),
                px(1.),
                rgb(palette().surface),
                BorderStyle::Solid,
            ));
            window.set_cursor_style(CursorStyle::Arrow, hitbox);
        }
        let drag_down = drag.clone();
        let change = self.change.as_ref().unwrap().clone();
        let metrics = layout.metrics;
        let bars = layout.bars.clone();
        window.on_mouse_event(move |e: &MouseDownEvent, phase, window, cx| {
            if !super::interaction::allowed(window, cx) {
                drag_down.set(None);
                if phase == DispatchPhase::Capture && metrics.viewport.contains(&e.position) {
                    cx.stop_propagation();
                }
                return;
            }
            if phase != DispatchPhase::Bubble || e.button != MouseButton::Left {
                return;
            }
            for (axis, geometry, _, hitbox) in &bars {
                if !hitbox.is_hovered(window) {
                    continue;
                }
                let position = axis.coordinate(e.position);
                if position >= geometry.thumb.top() && position <= geometry.thumb.bottom() {
                    drag_down.set(Some((
                        *axis,
                        f32::from(position - geometry.thumb.top()),
                        super::interaction::epoch(window, cx),
                    )));
                } else {
                    let direction = if position < geometry.thumb.top() {
                        1.
                    } else {
                        -1.
                    };
                    let viewport = axis.transpose(metrics.viewport);
                    let value = (axis.coordinate(metrics.offset)
                        + viewport.size.height * 0.9 * direction)
                        .clamp(px(-geometry.overflow), px(0.));
                    let mut next = metrics.offset;
                    match axis {
                        Axis::Vertical => next.y = value,
                        Axis::Horizontal => next.x = value,
                    }
                    change(next, window, cx);
                }
                cx.stop_propagation();
                window.refresh();
                break;
            }
        });
        let drag_move = drag.clone();
        let hovered = hovered.clone();
        let change = self.change.as_ref().unwrap().clone();
        let bars = layout.bars.clone();
        window.on_mouse_event(move |e: &MouseMoveEvent, phase, window, cx| {
            if phase != DispatchPhase::Capture {
                return;
            }
            if !super::interaction::allowed(window, cx) {
                drag_move.set(None);
                return;
            }
            let over = bars.iter().any(|b| b.3.is_hovered(window));
            if hovered.replace(over) != over {
                window.refresh();
            }
            let Some((axis, grab, epoch)) = drag_move.get() else {
                return;
            };
            if epoch != super::interaction::epoch(window, cx)
                || e.pressed_button != Some(MouseButton::Left)
            {
                drag_move.set(None);
            } else if let Some((_, geometry, _, _)) = bars.iter().find(|b| b.0 == axis) {
                let value = geometry.drag_offset(axis.coordinate(e.position), grab);
                let mut next = metrics.offset;
                match axis {
                    Axis::Vertical => next.y = value,
                    Axis::Horizontal => next.x = value,
                }
                change(next, window, cx);
            }
            cx.stop_propagation();
            window.refresh();
        });
        let drag_up = drag.clone();
        window.on_mouse_event(move |e: &MouseUpEvent, phase, window, cx| {
            if phase == DispatchPhase::Capture
                && e.button == MouseButton::Left
                && drag_up.take().is_some()
            {
                cx.stop_propagation();
                window.refresh();
            }
        });
        let change = self.change.as_ref().unwrap().clone();
        let bars = layout.bars.clone();
        window.on_mouse_event(move |e: &ScrollWheelEvent, phase, window, cx| {
            if !super::interaction::allowed(window, cx) {
                // Guard GPUI's child scroll owner too, not just our thumb's
                // offset callback. This listener predates a possible lock.
                if phase == DispatchPhase::Capture && metrics.viewport.contains(&e.position) {
                    cx.stop_propagation();
                }
                return;
            }
            if phase != DispatchPhase::Bubble || !bars.iter().any(|b| b.3.is_hovered(window)) {
                return;
            }
            let mut delta = e.delta.pixel_delta(px(20.));
            if e.modifiers.shift && delta.x == px(0.) {
                delta.x = delta.y;
                delta.y = px(0.);
            }
            let next = point(
                (metrics.offset.x + delta.x).clamp(-metrics.overflow.width, px(0.)),
                (metrics.offset.y + delta.y).clamp(-metrics.overflow.height, px(0.)),
            );
            if next != metrics.offset {
                change(next, window, cx);
                cx.stop_propagation();
            }
        });
    }
}

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
        if !overflow.is_finite()
            || !viewport.is_finite()
            || !offset.is_finite()
            || overflow <= 1.
            || height <= INSET * 2.
            || viewport <= 0.
        {
            return None;
        }
        let track = Bounds::new(
            bounds.origin + point(px(INSET), px(INSET)),
            size(bounds.size.width - px(INSET * 2.), px(height)),
        );
        let minimum = MIN_THUMB.min(height * 0.5);
        let thumb_height = (height * viewport / (viewport + overflow)).clamp(minimum, height);
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
    drag: DragState,
}
impl ScrollbarState {
    pub fn cancel_drag(&self) {
        self.drag.set(None);
    }
    /// The gutter is reserved by the caller. Geometry is read after the scroll
    /// owner's layout, so first paint, resize and disclosures use actual overflow.
    pub fn element(&self, scroll: &ScrollHandle, owner: EntityId) -> impl IntoElement {
        let layout_scroll = scroll.clone();
        let scroll = scroll.clone();
        let mut bars = both(
            "reserved-scrollbar",
            div()
                .id("settings-scrollbar")
                .debug_selector(|| "settings-scrollbar".into())
                .absolute()
                .top_0()
                .bottom_0()
                .right_0()
                .w(px(GUTTER)),
            move |bounds, _| ScrollMetrics {
                viewport: bounds,
                overflow: size(px(0.), layout_scroll.max_offset().height),
                offset: layout_scroll.offset(),
            },
            move |offset, window, cx| {
                scroll.set_offset(offset);
                cx.notify(owner);
                window.refresh();
            },
        );
        bars.drag = Some(self.drag.clone());
        bars
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

#[cfg(all(test, feature = "ui-tests"))]
#[path = "scrollbar_tests.rs"]
mod interaction_tests;
