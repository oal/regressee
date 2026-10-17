// SPDX-License-Identifier: MPL-2.0
//! Simulated GPUI input, not OS/accessibility qualification.
use super::*;
use core::prelude::v1::test;

struct ReadingView {
    scroll: ScrollHandle,
    list: UniformListScrollHandle,
    focus: FocusHandle,
    rows: usize,
    virtualized: bool,
    clicks: usize,
}
impl Render for ReadingView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let rows = self.rows;
        let row = |i| div().h(px(40.)).flex_shrink_0().child(format!("Row {i}"));
        let content = if self.virtualized {
            vertical(
                "list-bar",
                &self.list.0.borrow().base_handle,
                uniform_list("rows", rows, move |range, _, _| range.map(row).collect())
                    .track_scroll(self.list.clone())
                    .w(px(280.))
                    .h(px(180.))
                    .debug_selector(|| "reading-clip".into()),
            )
            .into_any_element()
        } else {
            vertical(
                "reading-bar",
                &self.scroll,
                div()
                    .id("reading")
                    .debug_selector(|| "reading-clip".into())
                    .w(px(280.))
                    .h(px(180.))
                    .overflow_y_scroll()
                    .track_scroll(&self.scroll)
                    .flex()
                    .flex_col()
                    .children((0..rows).map(row))
                    .on_mouse_down(MouseButton::Left, cx.listener(|s, _, _, _| s.clicks += 1)),
            )
            .into_any_element()
        };
        div().size_full().track_focus(&self.focus).child(content)
    }
}
fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}
fn geometry(handle: &ScrollHandle) -> ScrollbarGeometry {
    let b = handle.bounds();
    ScrollbarGeometry::new(
        Bounds::new(
            point(b.right() - px(GUTTER), b.top()),
            size(px(GUTTER), b.size.height),
        ),
        f32::from(b.size.height),
        f32::from(handle.max_offset().height),
        f32::from(handle.offset().y),
    )
    .unwrap()
}

#[gpui::test]
fn agent_gate_revokes_retained_shared_scrollbar_drag_even_after_unlock(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, cx| ReadingView {
        scroll: ScrollHandle::new(),
        list: UniformListScrollHandle::new(),
        focus: cx.focus_handle(),
        rows: 80,
        virtualized: false,
        clicks: 0,
    });
    cx.simulate_resize(size(px(320.), px(240.)));
    let gate = crate::ui::interaction::InputGate::default();
    cx.update(|w, cx| gate.bind(w, cx));
    draw(cx);
    let handle = view.read_with(cx, |v, _| v.scroll.clone());
    let g = geometry(&handle);
    cx.simulate_mouse_down(g.thumb.center(), MouseButton::Left, Modifiers::none());
    gate.set_allowed(false);
    let before = handle.offset();
    cx.simulate_mouse_move(
        point(g.track.center().x, g.track.bottom()),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.simulate_event(ScrollWheelEvent {
        position: g.track.center(),
        delta: ScrollDelta::Pixels(point(px(0.), px(-200.))),
        ..Default::default()
    });
    assert_eq!(handle.offset(), before);
    gate.set_allowed(true);
    cx.simulate_mouse_move(
        point(g.track.center().x, g.track.bottom()),
        MouseButton::Left,
        Modifiers::none(),
    );
    assert_eq!(
        handle.offset(),
        before,
        "unlock must not revive an old held gesture"
    );
    cx.simulate_mouse_up(g.thumb.center(), MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_down(g.thumb.center(), MouseButton::Left, Modifiers::none());
    gate.set_allowed(false);
    gate.set_allowed(true);
    cx.simulate_mouse_move(
        point(g.track.center().x, g.track.bottom()),
        MouseButton::Left,
        Modifiers::none(),
    );
    assert_eq!(
        handle.offset(),
        before,
        "even without an event during the lock, the gesture epoch revokes retained ownership"
    );
    cx.simulate_mouse_up(g.thumb.center(), MouseButton::Left, Modifiers::none());
    // A new human gesture is still available; the gate is not a permanent trap.
    cx.simulate_click(
        point(g.track.center().x, g.track.bottom() - px(1.)),
        Modifiers::none(),
    );
    assert!(handle.offset().y < before.y);
}

#[gpui::test]
fn scrollbars_page_drag_release_wheel_and_preserve_focus(cx: &mut TestAppContext) {
    for virtualized in [false, true] {
        let (view, cx) = cx.add_window_view(|w, cx| {
            let focus = cx.focus_handle();
            w.focus(&focus);
            ReadingView {
                scroll: ScrollHandle::new(),
                list: UniformListScrollHandle::new(),
                focus,
                rows: 80,
                virtualized,
                clicks: 0,
            }
        });
        cx.simulate_resize(size(px(320.), px(240.)));
        draw(cx);
        let handle = view.read_with(cx, |v, _| {
            if virtualized {
                v.list.0.borrow().base_handle.clone()
            } else {
                v.scroll.clone()
            }
        });
        let top = geometry(&handle);
        cx.simulate_click(
            point(top.track.center().x, top.track.bottom() - px(1.)),
            Modifiers::none(),
        );
        draw(cx);
        assert!(handle.offset().y < px(0.));
        let current = geometry(&handle);
        cx.simulate_mouse_down(current.thumb.center(), MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(
            point(px(310.), px(230.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        draw(cx);
        assert_eq!(handle.offset().y, -handle.max_offset().height);
        cx.simulate_mouse_up(
            point(px(310.), px(230.)),
            MouseButton::Left,
            Modifiers::none(),
        );
        let offset = handle.offset();
        cx.simulate_mouse_move(top.thumb.center(), None, Modifiers::none());
        draw(cx);
        assert_eq!(handle.offset(), offset, "release outside must end the drag");
        cx.simulate_event(ScrollWheelEvent {
            position: top.track.center(),
            delta: ScrollDelta::Pixels(point(px(0.), px(120.))),
            ..Default::default()
        });
        draw(cx);
        assert!(handle.offset().y > offset.y);
        cx.update(|w, cx| {
            view.read_with(cx, |v, _| {
                assert!(v.focus.is_focused(w), "scrolling must not take focus");
                assert_eq!(
                    v.clicks, 0,
                    "scrollbar must not activate content beneath it"
                );
            })
        });
        view.update(cx, |_, cx| cx.notify());
        let retained = handle.offset();
        draw(cx);
        assert_eq!(
            handle.offset(),
            retained,
            "ordinary rerender must retain the offset"
        );
        view.update(cx, |v, cx| {
            v.rows = 2;
            cx.notify();
        });
        draw(cx);
        assert_eq!(handle.max_offset().height, px(0.));
        assert_eq!(handle.offset().y, px(0.));
    }
}

struct TwoAxisView {
    offset: Point<Pixels>,
}
impl Render for TwoAxisView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let offset = self.offset;
        let owner = cx.entity();
        both(
            "two-axis",
            div().w(px(280.)).h(px(180.)),
            move |b, _| ScrollMetrics {
                viewport: b,
                overflow: size(px(720.), px(820.)),
                offset,
            },
            move |offset, _, cx| {
                owner.update(cx, |v, cx| {
                    v.offset = offset;
                    cx.notify();
                })
            },
        )
    }
}

#[gpui::test]
fn two_axis_scrollbars_keep_corner_separate_and_drag_each_axis(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| TwoAxisView {
        offset: point(px(0.), px(0.)),
    });
    cx.simulate_resize(size(px(320.), px(240.)));
    draw(cx);
    // Horizontal gutter ends before the vertical gutter. Track paging cannot
    // trigger a vertical move, and vice versa.
    cx.simulate_click(point(px(240.), px(172.)), Modifiers::none());
    draw(cx);
    view.read_with(cx, |v, _| {
        assert!(v.offset.x < px(0.));
        assert_eq!(v.offset.y, px(0.));
    });
    cx.simulate_click(point(px(272.), px(150.)), Modifiers::none());
    draw(cx);
    view.read_with(cx, |v, _| assert!(v.offset.y < px(0.)));
}

struct UntrackedView;
impl Render for UntrackedView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        scrollable(
            "owned",
            div()
                .id("owned-reading")
                .debug_selector(|| "owned-reading".into())
                .w(px(280.))
                .h(px(180.))
                .overflow_y_scroll()
                .child(
                    div()
                        .h(px(1000.))
                        .child(div().debug_selector(|| "owned-top".into()).child("Top")),
                ),
        )
    }
}
#[gpui::test]
fn owned_scroll_handle_survives_repaint(cx: &mut TestAppContext) {
    let (view, cx) = cx.add_window_view(|_, _| UntrackedView);
    cx.simulate_resize(size(px(320.), px(240.)));
    draw(cx);
    cx.simulate_click(point(px(272.), px(160.)), Modifiers::none());
    draw(cx);
    let top = cx.debug_bounds("owned-top").unwrap().top();
    assert!(top < px(0.));
    view.update(cx, |_, cx| cx.notify());
    draw(cx);
    assert_eq!(cx.debug_bounds("owned-top").unwrap().top(), top);
}
