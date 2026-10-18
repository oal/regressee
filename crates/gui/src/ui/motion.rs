// SPDX-License-Identifier: MPL-2.0
//! Bounded native paint continuity. No timers, layout animation or evidence effects.
//! State belongs to consecutively painted GPUI elements, never a global registry.
use super::{controls::ReducedMotion, theme::palette};
use gpui::{prelude::*, *};
use std::{
    cell::Cell,
    rc::Rc,
    time::{Duration, Instant},
};

pub const CONTROL: Duration = Duration::from_millis(120);
pub const PROGRESS: Duration = Duration::from_millis(160);

pub fn reduced(cx: &App) -> bool {
    cx.try_global::<ReducedMotion>()
        .is_some_and(|value| value.0)
}

/// Fast, monotone exponential arrival with exact endpoints (no overshoot).
pub fn ease_out(t: f32) -> f32 {
    let t = t.clamp(0., 1.);
    if t == 1. {
        1.
    } else {
        (1. - 2_f32.powf(-10. * t)) / (1. - 2_f32.powf(-10.))
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Transition<const N: usize> {
    from: [f32; N],
    target: [f32; N],
    start: Instant,
    duration: Duration,
}

impl<const N: usize> Transition<N> {
    pub(crate) fn settled(value: [f32; N], now: Instant, duration: Duration) -> Self {
        Self {
            from: value,
            target: value,
            start: now,
            duration,
        }
    }

    pub(crate) fn value(&self, now: Instant) -> [f32; N] {
        let t = if self.duration.is_zero() {
            1.
        } else {
            ease_out(
                now.saturating_duration_since(self.start).as_secs_f32()
                    / self.duration.as_secs_f32(),
            )
        };
        std::array::from_fn(|i| self.from[i] + (self.target[i] - self.from[i]) * t)
    }

    pub(crate) fn retarget(
        &mut self,
        target: [f32; N],
        now: Instant,
        immediate: bool,
    ) -> ([f32; N], bool) {
        if immediate {
            self.from = target;
            self.target = target;
        } else if self.target != target {
            // Interrupt from the currently painted value, not a stale endpoint.
            self.from = self.value(now);
            self.target = target;
            self.start = now;
        }
        let active =
            self.from != self.target && now.saturating_duration_since(self.start) < self.duration;
        if !active {
            self.from = self.target;
        }
        (self.value(now), active)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum Entrance {
    Menu,
    Panel,
    Dialog,
    Feedback,
    Completion,
}

impl Entrance {
    fn duration(self) -> Duration {
        Duration::from_millis(match self {
            Self::Menu => 120,
            Self::Panel | Self::Dialog => 160,
            Self::Feedback => 140,
            Self::Completion => 180,
        })
    }
}

/// Wrap only newly mounted overlay/feedback chrome, NEVER screenshot/DOM evidence.
/// This forwards the child's exact layout node and hitboxes. GPUI's public API
/// cannot offset paint independently of hitboxes, so arrivals are opacity-only,
/// starting at 94% visibility; focus, input and dismissal are never delayed.
/// Keep `id` stable while mounted; unmount before reusing it for a new arrival.
pub fn enter(
    id: impl Into<ElementId>,
    element: impl IntoElement + 'static,
    kind: Entrance,
    _cx: &App,
) -> AnyElement {
    Arrival {
        id: id.into(),
        child: element.into_any_element(),
        kind,
        opacity: Interactivity::default(),
    }
    .into_any_element()
}

struct Arrival {
    id: ElementId,
    child: AnyElement,
    kind: Entrance,
    opacity: Interactivity,
}

impl IntoElement for Arrival {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}

impl Element for Arrival {
    type RequestLayoutState = ();
    type PrepaintState = bool;
    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }
    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        (self.child.request_layout(window, cx), ())
    }
    fn prepaint(
        &mut self,
        id: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        window: &mut Window,
        cx: &mut App,
    ) -> bool {
        let now = cx.background_executor().now();
        let visible = !bounds.intersect(&window.content_mask().bounds).is_empty();
        let (value, active) =
            window.with_element_state(id.unwrap(), |state: Option<Transition<1>>, _| {
                let mut state = state.unwrap_or_else(|| {
                    let mut state = Transition::settled([0.94], now, self.kind.duration());
                    state.retarget([1.], now, false);
                    state
                });
                let result = state.retarget([1.], now, reduced(cx) || !visible);
                (result, state)
            });
        self.opacity.base_style.opacity = Some(value[0]);
        #[cfg(all(test, feature = "ui-tests"))]
        if cx.has_global::<MotionProbe>() {
            cx.global_mut::<MotionProbe>().arrival = Some((value[0], active));
        }
        self.child.prepaint(window, cx);
        active && visible
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        active: &mut bool,
        window: &mut Window,
        cx: &mut App,
    ) {
        // Interactivity provides GPUI's native opacity scope without a layout div.
        self.opacity
            .paint(None, None, bounds, None, window, cx, |_, window, cx| {
                self.child.paint(window, cx)
            });
        if *active && !reduced(cx) {
            window.request_animation_frame();
        }
    }
}

fn channels(color: Hsla) -> [f32; 4] {
    let color: Rgba = color.into();
    // Premultiplied interpolation avoids a dark flash when quiet toolbar tools
    // transition between a transparent idle surface and their hover/selection.
    [
        color.r * color.a,
        color.g * color.a,
        color.b * color.a,
        color.a,
    ]
}

fn color_from_channels(value: [f32; 4]) -> Rgba {
    let alpha = value[3];
    let component = |index| if alpha > 0. { value[index] / alpha } else { 0. };
    Rgba {
        r: component(0),
        g: component(1),
        b: component(2),
        a: alpha,
    }
}

struct ColorState {
    transition: Transition<4>,
    pressed: Rc<Cell<bool>>,
    theme: u32,
    fresh: bool,
}

/// First child of a positioned control; paint only, no layout footprint or ring.
/// Parent keeps native focus/disabled styles and activation handlers; press snaps
/// this paint layer immediately. Borders and logical marks never interpolate.
pub(super) fn color_layer(
    key: &'static str,
    background: Hsla,
    hover: Hsla,
    enabled: bool,
    interactive: bool,
    radius: f32,
) -> impl IntoElement {
    canvas(
        move |bounds, window, cx| {
            let now = cx.background_executor().now();
            let state = window.use_keyed_state(key, cx, |_, _| ColorState {
                transition: Transition::settled(channels(background), now, CONTROL),
                pressed: Rc::new(Cell::new(false)),
                theme: palette().surface,
                fresh: true,
            });
            let hitbox = (enabled && interactive)
                .then(|| window.insert_hitbox(bounds, HitboxBehavior::Normal));
            (state, hitbox)
        },
        move |bounds, (state, hitbox), window, cx| {
            let visible = !bounds.intersect(&window.content_mask().bounds).is_empty();
            let hovered = hitbox
                .as_ref()
                .is_some_and(|hitbox| hitbox.is_hovered(window))
                && !cx.has_active_drag();
            let target = channels(if hovered { hover } else { background });
            let now = cx.background_executor().now();
            let is_reduced = reduced(cx);
            let (value, active, pressed) = state.update(cx, |state, _| {
                let immediate = is_reduced
                    || state.fresh
                    || !visible
                    || !enabled
                    || state.pressed.get()
                    || state.theme != palette().surface;
                if !enabled || !visible {
                    state.pressed.set(false);
                }
                state.theme = palette().surface;
                state.fresh = false;
                let (value, active) = state.transition.retarget(target, now, immediate);
                (value, active, state.pressed.clone())
            });
            #[cfg(all(test, feature = "ui-tests"))]
            if cx.has_global::<MotionProbe>() {
                cx.global_mut::<MotionProbe>().control = Some((value, active));
            }
            if visible {
                let color = color_from_channels(value);
                window.paint_quad(quad(
                    bounds,
                    px(radius),
                    color,
                    px(0.),
                    transparent_black(),
                    BorderStyle::default(),
                ));
                if active {
                    window.request_animation_frame();
                }
            }
            if let Some(hitbox) = hitbox {
                let down = pressed.clone();
                window.on_mouse_event(move |event: &MouseDownEvent, phase, window, _| {
                    if phase == DispatchPhase::Capture
                        && event.button == MouseButton::Left
                        && hitbox.is_hovered(window)
                    {
                        down.set(true);
                        window.refresh();
                    }
                });
                window.on_mouse_event(move |_: &MouseUpEvent, phase, window, _| {
                    if phase == DispatchPhase::Capture && pressed.replace(false) {
                        window.refresh();
                    }
                });
            }
        },
    )
    .absolute()
    .inset_0()
}

/// A fraction never leads the last report. New totals and regressions indicate a
/// new work domain/reset and snap rather than interpolating unrelated operations.
pub(crate) struct ProgressState {
    transition: Transition<1>,
    total: u32,
    completed: u32,
}

impl ProgressState {
    pub(crate) fn new(completed: u32, total: u32, now: Instant) -> Self {
        Self {
            transition: Transition::settled([fraction(completed, total)], now, PROGRESS),
            total,
            completed,
        }
    }
    pub(crate) fn report(
        &mut self,
        completed: u32,
        total: u32,
        now: Instant,
        immediate: bool,
    ) -> (f32, bool) {
        let reset = total != self.total || completed < self.completed || total == 0;
        self.total = total;
        self.completed = completed;
        let target = fraction(completed, total);
        let (value, active) = self.transition.retarget([target], now, immediate || reset);
        (value[0].clamp(0., target), active)
    }
}

fn fraction(completed: u32, total: u32) -> f32 {
    if total == 0 {
        0.
    } else {
        completed.min(total) as f32 / total as f32
    }
}

#[cfg(all(test, feature = "ui-tests"))]
#[derive(Default)]
struct MotionProbe {
    control: Option<([f32; 4], bool)>,
    arrival: Option<(f32, bool)>,
    progress: Option<(f32, bool)>,
}
#[cfg(all(test, feature = "ui-tests"))]
impl Global for MotionProbe {}
#[cfg(all(test, feature = "ui-tests"))]
pub(super) fn record_progress(cx: &mut App, value: f32, active: bool) {
    if cx.has_global::<MotionProbe>() {
        cx.global_mut::<MotionProbe>().progress = Some((value, active));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn timing_is_bounded_monotone_and_exact() {
        assert_eq!(ease_out(0.), 0.);
        assert_eq!(ease_out(1.), 1.);
        let mut last = 0.;
        for frame in 0..=100 {
            let value = ease_out(frame as f32 / 100.);
            assert!((last..=1.).contains(&value));
            last = value;
        }
        let now = Instant::now();
        let mut state = Transition::settled([0.], now, CONTROL);
        assert_eq!(state.retarget([0.], now, false), ([0.], false));
        assert_eq!(state.retarget([1.], now, false), ([0.], true));
        assert_eq!(state.retarget([1.], now + CONTROL, false), ([1.], false));
        let mut instant = Transition::settled([0.], now, Duration::ZERO);
        assert_eq!(instant.retarget([1.], now, false), ([1.], false));
    }

    #[test]
    fn transparent_toolbar_color_does_not_pass_through_black() {
        let now = Instant::now();
        let target: Hsla = rgb(0xe8e8e8).into();
        let mut state = Transition::settled(channels(transparent_black()), now, CONTROL);
        state.retarget(channels(target), now, false);
        let value = color_from_channels(state.value(now + CONTROL / 3));
        let target: Rgba = target.into();
        assert!((value.r - target.r).abs() < 0.00001);
        assert!((value.g - target.g).abs() < 0.00001);
        assert!((value.b - target.b).abs() < 0.00001);
        assert!((0. ..1.).contains(&value.a));
    }

    #[test]
    fn interruption_preserves_painted_value_and_reduction_finishes_without_replay() {
        let now = Instant::now();
        let mut state = Transition::settled([0.], now, CONTROL);
        state.retarget([1.], now, false);
        let midway = now + CONTROL / 3;
        let painted = state.value(midway);
        assert_eq!(state.retarget([0.], midway, false), (painted, true));
        assert_eq!(state.retarget([0.], midway, true), ([0.], false));
        assert_eq!(state.retarget([0.], midway, false), ([0.], false));
    }

    #[test]
    fn progress_never_invents_work_and_resets_unrelated_domains() {
        let now = Instant::now();
        let mut state = ProgressState::new(20, 100, now);
        assert_eq!(state.report(40, 100, now, false), (0.2, true));
        let (value, active) = state.report(50, 100, now + PROGRESS / 3, false);
        assert!(active && (0.2..=0.4).contains(&value));
        assert_eq!(state.report(1, 100, now, false), (0.01, false));
        assert_eq!(state.report(50, 200, now, false), (0.25, false));
        assert_eq!(state.report(u32::MAX, 200, now, true), (1., false));
        assert_eq!(state.report(10, 0, now, false), (0., false));
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod ui_tests {
    use super::super::controls::{self, ActionStyle};
    use super::*;
    use core::prelude::v1::test;

    struct Host {
        focus: FocusHandle,
        selected: bool,
        enabled: bool,
        completed: u32,
        total: Option<u32>,
        mounted: bool,
        offscreen: bool,
        activations: usize,
    }
    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().p_2().when(self.mounted, |root| {
                root.child(
                    div()
                        .w(px(300.))
                        .flex()
                        .flex_col()
                        .gap_2()
                        .when(self.offscreen, |el| el.mt(px(2000.)))
                        .child(
                            controls::action(
                                "motion-test-action",
                                "Choose",
                                &self.focus,
                                ActionStyle::Choice {
                                    selected: self.selected,
                                },
                                self.enabled,
                            )
                            .on_click(cx.listener(|host, _, _, cx| {
                                if host.enabled {
                                    host.activations += 1;
                                    cx.notify();
                                }
                            })),
                        )
                        .child(controls::progress(
                            format!("{} of 100", self.completed),
                            self.completed,
                            self.total,
                        ))
                        .child(enter(
                            "motion-test-arrival",
                            div()
                                .debug_selector(|| "motion-test-overlay".into())
                                .w(px(100.))
                                .h(px(40.))
                                .child("Ready"),
                            Entrance::Panel,
                            cx,
                        )),
                )
            })
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<Host>, &mut VisualTestContext) {
        cx.update(|cx| {
            cx.set_global(ReducedMotion(false));
            cx.set_global(MotionProbe::default());
        });
        cx.add_window_view(|_, cx| Host {
            focus: cx.focus_handle(),
            selected: false,
            enabled: true,
            completed: 20,
            total: Some(100),
            mounted: true,
            offscreen: false,
            activations: 0,
        })
    }

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|window, cx| {
            window.refresh();
            let _ = window.draw(cx);
        });
    }

    fn color(cx: &mut VisualTestContext) -> ([f32; 4], bool) {
        cx.update(|_, cx| cx.global::<MotionProbe>().control.unwrap())
    }

    #[gpui::test]
    fn native_hover_interrupts_without_target_motion_and_press_is_immediate(
        cx: &mut TestAppContext,
    ) {
        let (host, cx) = setup(cx);
        draw(cx);
        let bounds = cx.debug_bounds("action-motion-test-action").unwrap();
        let base = color(cx).0;
        cx.simulate_mouse_move(bounds.center(), None, Modifiers::none());
        draw(cx);
        assert!(color(cx).1);
        cx.executor().advance_clock(CONTROL / 3);
        draw(cx);
        let midway = color(cx).0;
        assert_ne!(midway, base);
        cx.simulate_mouse_move(point(px(700.), px(500.)), None, Modifiers::none());
        draw(cx);
        assert_eq!(color(cx).0, midway);
        cx.executor().advance_clock(CONTROL);
        draw(cx);
        assert_eq!(color(cx), (base, false));
        cx.simulate_mouse_move(bounds.center(), None, Modifiers::none());
        draw(cx);
        cx.simulate_mouse_down(bounds.center(), MouseButton::Left, Modifiers::none());
        draw(cx);
        assert_eq!(color(cx), (channels(rgb(palette().selected).into()), false));
        cx.update(|window, cx| window.focus(&host.read(cx).focus));
        draw(cx);
        assert_eq!(
            cx.debug_bounds("action-motion-test-action").unwrap(),
            bounds
        );
        cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::none());
        assert_eq!(host.read_with(cx, |host, _| host.activations), 1);
    }

    #[gpui::test]
    fn native_runtime_reduction_disabled_and_unmounted_states_do_not_replay(
        cx: &mut TestAppContext,
    ) {
        let (host, cx) = setup(cx);
        draw(cx);
        let bounds = cx.debug_bounds("action-motion-test-action").unwrap();
        host.update(cx, |host, cx| {
            host.selected = true;
            cx.notify();
        });
        draw(cx);
        assert!(color(cx).1);
        cx.update(|_, cx| cx.set_global(ReducedMotion(true)));
        draw(cx);
        assert_eq!(color(cx), (channels(rgb(palette().selected).into()), false));
        assert_eq!(
            cx.update(|_, cx| cx.global::<MotionProbe>().arrival.unwrap()),
            (1., false)
        );
        cx.update(|_, cx| cx.set_global(ReducedMotion(false)));
        draw(cx);
        assert!(!color(cx).1);
        host.update(cx, |host, cx| {
            host.selected = false;
            host.enabled = false;
            cx.notify();
        });
        draw(cx);
        assert_eq!(color(cx), (channels(rgb(palette().surface).into()), false));
        assert_eq!(
            cx.debug_bounds("action-motion-test-action").unwrap(),
            bounds
        );
        cx.simulate_click(bounds.center(), Modifiers::none());
        assert_eq!(host.read_with(cx, |host, _| host.activations), 0);
        host.update(cx, |host, cx| {
            host.mounted = false;
            cx.notify();
        });
        draw(cx);
        cx.update(|_, cx| cx.set_global(MotionProbe::default()));
        cx.executor().advance_clock(Duration::from_secs(1));
        draw(cx);
        assert!(cx.update(|_, cx| cx.global::<MotionProbe>().control.is_none()));
        host.update(cx, |host, cx| {
            host.mounted = true;
            host.enabled = true;
            cx.notify();
        });
        draw(cx);
        assert!(!color(cx).1); // remounted controls start settled, not an entrance
        assert!(cx.update(|_, cx| cx.global::<MotionProbe>().arrival.unwrap().0 >= 0.94));
    }

    #[gpui::test]
    fn native_progress_retargets_reports_and_clipped_motion_settles(cx: &mut TestAppContext) {
        let (host, cx) = setup(cx);
        draw(cx);
        let track = cx.debug_bounds("progress-track").unwrap();
        let overlay = cx.debug_bounds("motion-test-overlay").unwrap();
        assert_eq!(
            cx.update(|_, cx| cx.global::<MotionProbe>().progress.unwrap()),
            (0.2, false)
        );
        host.update(cx, |host, cx| {
            host.completed = 60;
            cx.notify();
        });
        draw(cx);
        assert_eq!(
            cx.update(|_, cx| cx.global::<MotionProbe>().progress.unwrap()),
            (0.2, true)
        );
        assert_eq!(
            cx.debug_bounds("progress-fill").unwrap().size.width,
            track.size.width * 0.6
        );
        cx.executor().advance_clock(PROGRESS / 3);
        draw(cx);
        let painted = cx.update(|_, cx| cx.global::<MotionProbe>().progress.unwrap().0);
        assert!((0.2..0.6).contains(&painted));
        host.update(cx, |host, cx| {
            host.completed = 80;
            cx.notify();
        });
        draw(cx);
        assert_eq!(
            cx.update(|_, cx| cx.global::<MotionProbe>().progress.unwrap().0),
            painted
        );
        cx.update(|_, cx| cx.set_global(ReducedMotion(true)));
        draw(cx);
        assert_eq!(
            cx.update(|_, cx| cx.global::<MotionProbe>().progress.unwrap()),
            (0.8, false)
        );
        assert_eq!(cx.debug_bounds("progress-track").unwrap(), track);
        assert_eq!(cx.debug_bounds("motion-test-overlay").unwrap(), overlay);
        cx.update(|_, cx| cx.set_global(ReducedMotion(false)));
        host.update(cx, |host, cx| {
            host.completed = 1;
            host.total = Some(200);
            host.selected = true;
            host.offscreen = true;
            cx.notify();
        });
        draw(cx);
        assert!(!color(cx).1);
        assert_eq!(
            cx.update(|_, cx| cx.global::<MotionProbe>().progress.unwrap()),
            (0.005, false)
        );
        assert_eq!(
            cx.update(|_, cx| cx.global::<MotionProbe>().arrival.unwrap()),
            (1., false)
        );
    }
}
