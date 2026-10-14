// SPDX-License-Identifier: MPL-2.0
//! A single, selection-only zoom control. The viewer owns the actual transform.
use super::{controls::focus_outline, theme::*, timezone_select::chevron_sized};
use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Context, EventEmitter, FocusHandle, FontWeight,
    KeyBinding, MouseButton, MouseDownEvent, Pixels, ScrollStrategy, Size, Subscription,
    UniformListScrollHandle, Window, actions, anchored, canvas, deferred, div, point, prelude::*,
    px, rgb, uniform_list,
};

actions!(
    zoom_select,
    [Toggle, Next, Previous, First, Last, Choose, Close]
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ZoomChoice {
    FitWidth,
    Percent(u16),
}

/// A help request is not a zoom selection and must not change the transform.
pub struct ImageControlsRequested;

const OPTIONS: [ZoomChoice; 7] = [
    ZoomChoice::FitWidth,
    ZoomChoice::Percent(100),
    ZoomChoice::Percent(25),
    ZoomChoice::Percent(50),
    ZoomChoice::Percent(75),
    ZoomChoice::Percent(150),
    ZoomChoice::Percent(200),
];

fn label(fit: bool, zoom: f32) -> String {
    if fit {
        format!("Zoom: Fit width · {:.0}%", zoom * 100.)
    } else {
        format!("Zoom: {:.0}%", zoom * 100.)
    }
}

fn current_index(fit: bool, zoom: f32) -> Option<usize> {
    OPTIONS.iter().position(|choice| match choice {
        ZoomChoice::FitWidth => fit,
        ZoomChoice::Percent(percent) => !fit && (zoom * 100. - *percent as f32).abs() < 0.01,
    })
}

fn popup_bounds(trigger: Bounds<Pixels>, viewport: Size<Pixels>, rows: usize) -> Bounds<Pixels> {
    super::selector::popup_bounds(trigger, viewport, rows, 256.)
}

pub struct ZoomSelect {
    fit: bool,
    zoom: f32,
    enabled: bool,
    compact: bool,
    image_controls_available: bool,
    open: bool,
    focus: FocusHandle,
    highlighted: usize,
    scroll: UniformListScrollHandle,
    trigger: Option<Bounds<Pixels>>,
    blur: Option<Subscription>,
}

impl EventEmitter<ZoomChoice> for ZoomSelect {}
impl EventEmitter<ImageControlsRequested> for ZoomSelect {}

impl ZoomSelect {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self {
            fit: true,
            zoom: 1.,
            enabled: false,
            compact: false,
            image_controls_available: false,
            open: false,
            focus: cx.focus_handle().tab_stop(false),
            highlighted: 0,
            scroll: UniformListScrollHandle::new(),
            trigger: None,
            blur: None,
        }
    }

    pub fn focus_handle(&self) -> FocusHandle {
        self.focus.clone()
    }
    pub fn is_open(&self) -> bool {
        self.open
    }

    pub fn set_image_controls_available(&mut self, available: bool, cx: &mut Context<Self>) {
        if self.image_controls_available != available {
            self.image_controls_available = available;
            self.highlighted = self.highlighted.min(self.option_count() - 1);
            cx.notify();
        }
    }

    fn option_count(&self) -> usize {
        OPTIONS.len() + usize::from(self.compact && self.image_controls_available)
    }

    pub fn sync(
        &mut self,
        fit: bool,
        zoom: f32,
        enabled: bool,
        compact: bool,
        cx: &mut Context<Self>,
    ) {
        if (self.fit, self.zoom, self.enabled, self.compact) != (fit, zoom, enabled, compact) {
            self.fit = fit;
            self.zoom = zoom;
            self.enabled = enabled;
            self.compact = compact;
            self.highlighted = self.highlighted.min(self.option_count() - 1);
            self.focus = self.focus.clone().tab_stop(enabled);
            if !enabled {
                self.open = false;
            }
            cx.notify();
        }
    }

    pub fn toggle(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled {
            return;
        }
        if self.open {
            self.dismiss(true, window, cx);
        } else {
            self.open = true;
            self.highlighted = current_index(self.fit, self.zoom).unwrap_or(0);
            self.scroll
                .scroll_to_item(self.highlighted, ScrollStrategy::Center);
            window.focus(&self.focus);
            cx.notify();
        }
    }

    fn dismiss(&mut self, restore: bool, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        self.open = false;
        if restore && self.enabled {
            window.focus(&self.focus);
        }
        cx.notify();
    }

    fn navigate(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled {
            return;
        }
        if !self.open {
            self.toggle(window, cx);
            return;
        }
        self.highlighted = index.min(self.option_count() - 1);
        self.scroll
            .scroll_to_item(self.highlighted, ScrollStrategy::Center);
        cx.notify();
    }

    fn select(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled || !self.open {
            return;
        }
        if let Some(choice) = OPTIONS.get(index) {
            let choice = *choice;
            self.dismiss(true, window, cx);
            cx.emit(choice);
        } else if index == OPTIONS.len() && index < self.option_count() {
            self.dismiss(true, window, cx);
            cx.emit(ImageControlsRequested);
        }
    }

    fn popup(&self, trigger: Bounds<Pixels>, window: &Window, cx: &Context<Self>) -> AnyElement {
        let bounds = popup_bounds(trigger, window.viewport_size(), self.option_count());
        let current = current_index(self.fit, self.zoom);
        let highlighted = self.highlighted;
        let entity = cx.entity();
        let list = uniform_list("zoom-options", self.option_count(), move |range, _, _| {
            range
                .map(|index| {
                    let target = entity.clone();
                    let selected = current == Some(index);
                    let text = match OPTIONS.get(index) {
                        Some(ZoomChoice::FitWidth) => "Fit width".to_owned(),
                        Some(ZoomChoice::Percent(100)) => "Actual size (100%)".to_owned(),
                        Some(ZoomChoice::Percent(percent)) => format!("{percent}%"),
                        None => "Image controls…".to_owned(),
                    };
                    div()
                        .id(("zoom-option", index))
                        .debug_selector(move || {
                            if index == OPTIONS.len() {
                                "zoom-image-controls".to_owned()
                            } else {
                                format!("zoom-option-{index}")
                            }
                        })
                        .w_full()
                        .h(px(CONTROL_HEIGHT))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .border_1()
                        .border_color(rgb(if highlighted == index { INK } else { SURFACE }))
                        .bg(rgb(if selected { SELECTED } else { SURFACE }))
                        .when(index == OPTIONS.len(), |row| {
                            row.border_t_1().border_color(rgb(if highlighted == index {
                                INK
                            } else {
                                LINE
                            }))
                        })
                        .text_size(px(CONTROL_TEXT_SIZE))
                        .line_height(px(CONTROL_LINE_HEIGHT))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(if selected { ACCENT_HOVER } else { INK }))
                        .cursor_pointer()
                        .hover(move |s| s.bg(rgb(if selected { SELECTED } else { ACTION_HOVER })))
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_click(move |_, window, cx| {
                            target.update(cx, |this, cx| this.select(index, window, cx));
                            cx.stop_propagation();
                        })
                        .child(div().flex_1().child(text))
                        .when(selected, |row| {
                            row.child(div().text_size(px(HELP_TEXT_SIZE)).child("Current"))
                        })
                })
                .collect::<Vec<_>>()
        })
        .w_full()
        .h((bounds.size.height - px(2.)).max(px(0.)))
        .track_scroll(self.scroll.clone());
        let popup = div()
            .id("zoom-popup")
            .debug_selector(|| "zoom-popup".into())
            .occlude()
            .w(bounds.size.width)
            .h(bounds.size.height)
            .overflow_hidden()
            .border_1()
            .border_color(rgb(CONTROL_LINE))
            .rounded(px(4.))
            .bg(rgb(SURFACE))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                if !this
                    .trigger
                    .is_some_and(|bounds| bounds.contains(&event.position))
                {
                    this.dismiss(false, window, cx);
                }
            }))
            .child(list);
        deferred(anchored().position(bounds.origin).child(popup))
            .with_priority(1)
            .into_any_element()
    }
}

pub fn bind(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("space", Toggle, Some("ZoomSelect")),
        KeyBinding::new("enter", Choose, Some("ZoomSelect")),
        KeyBinding::new("down", Next, Some("ZoomSelect")),
        KeyBinding::new("up", Previous, Some("ZoomSelect")),
        KeyBinding::new("home", First, Some("ZoomSelect")),
        KeyBinding::new("end", Last, Some("ZoomSelect")),
        KeyBinding::new("escape", Close, Some("ZoomSelect")),
    ]);
}

impl Render for ZoomSelect {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.open && !self.focus.is_focused(window) {
            self.dismiss(false, window, cx);
        }
        if !self.enabled && self.focus.is_focused(window) {
            window.blur();
        }
        if self.blur.is_none() {
            self.blur = Some(cx.on_blur(&self.focus, window, |this, window, cx| {
                this.dismiss(false, window, cx)
            }));
        }
        let entity = cx.entity();
        div()
            .key_context("ZoomSelect")
            .relative()
            .w(px(if self.compact { 112. } else { 232. }))
            .max_w_full()
            .h(px(CONTROL_HEIGHT))
            .flex_shrink_0()
            .on_action(cx.listener(|this, _: &Toggle, window, cx| this.toggle(window, cx)))
            .on_action(cx.listener(|this, _: &Next, window, cx| {
                this.navigate(this.highlighted + 1, window, cx)
            }))
            .on_action(cx.listener(|this, _: &Previous, window, cx| {
                this.navigate(this.highlighted.saturating_sub(1), window, cx)
            }))
            .on_action(cx.listener(|this, _: &First, window, cx| {
                if !this.open {
                    this.toggle(window, cx);
                }
                this.navigate(0, window, cx);
            }))
            .on_action(cx.listener(|this, _: &Last, window, cx| {
                if !this.open {
                    this.toggle(window, cx);
                }
                this.navigate(this.option_count() - 1, window, cx);
            }))
            .on_action(cx.listener(|this, _: &Choose, window, cx| {
                if this.open {
                    this.select(this.highlighted, window, cx);
                } else {
                    this.toggle(window, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &Close, window, cx| {
                if this.open {
                    this.dismiss(true, window, cx);
                } else {
                    cx.propagate();
                }
            }))
            .child(
                div()
                    .id("zoom-select-button")
                    .debug_selector(|| "zoom-select-button".into())
                    .size_full()
                    .px(px(if self.compact { 2. } else { 12. }))
                    .flex()
                    .items_center()
                    .gap(px(if self.compact { 2. } else { 8. }))
                    .border_1()
                    .border_color(rgb(CONTROL_LINE))
                    .rounded(px(4.))
                    .bg(rgb(ACTION_SURFACE))
                    .text_color(rgb(INK))
                    .font_family(FONT)
                    .text_size(px(CONTROL_TEXT_SIZE))
                    .line_height(px(CONTROL_LINE_HEIGHT))
                    .font_weight(FontWeight::MEDIUM)
                    .when(self.enabled, |el| {
                        el.track_focus(&self.focus)
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(ACTION_HOVER)))
                            .focus(|s| focus_outline(s, INK, INK))
                    })
                    .when(!self.enabled, |el| el.opacity(0.5))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            this.toggle(window, cx);
                            cx.stop_propagation();
                        }),
                    )
                    .child(if self.compact {
                        div()
                            .debug_selector(|| "zoom-value".into())
                            .flex_1()
                            .min_w_0()
                            .flex()
                            .flex_col()
                            .text_size(px(13.))
                            .line_height(px(16.))
                            .child(
                                div()
                                    .debug_selector(|| "zoom-caption".into())
                                    .whitespace_nowrap()
                                    .child("Zoom"),
                            )
                            .child(
                                div()
                                    .debug_selector(|| "zoom-scale".into())
                                    .whitespace_nowrap()
                                    .child(format!(
                                        "{} · {:.0}%",
                                        if self.fit { "Fit" } else { "Manual" },
                                        self.zoom * 100.
                                    )),
                            )
                    } else {
                        div()
                            .debug_selector(|| "zoom-value".into())
                            .flex_1()
                            .min_w_0()
                            .text_ellipsis()
                            .child(label(self.fit, self.zoom))
                    })
                    .child(chevron_sized(
                        self.open,
                        if self.compact { 10. } else { 16. },
                    )),
            )
            // Use this frame's bounds. Deferred painting escapes the clipped
            // viewer without growing the toolbar or hiding the page queue.
            .child(
                canvas(
                    move |bounds, window, cx| {
                        let popup = entity.update(cx, |this, cx| {
                            this.trigger = Some(bounds);
                            (this.open && this.enabled).then(|| this.popup(bounds, window, cx))
                        });
                        popup.map(|mut popup| {
                            popup.layout_as_root(
                                window.viewport_size().map(AvailableSpace::Definite),
                                window,
                                cx,
                            );
                            popup.prepaint_at(point(px(0.), px(0.)), window, cx);
                            popup
                        })
                    },
                    |_, popup, window, cx| {
                        if let Some(mut popup) = popup {
                            popup.paint(window, cx);
                        }
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    use gpui::{Entity, Modifiers, TestAppContext, VisualTestContext, size};

    actions!(zoom_select_tests, [ShellEnter, ShellTab]);

    struct Host {
        select: Entity<ZoomSelect>,
        after: FocusHandle,
        chosen: Vec<ZoomChoice>,
        help_requests: usize,
        shell_enter: usize,
    }

    impl Render for Host {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .key_context("Shell")
                .flex()
                .flex_col()
                .gap(px(12.))
                .on_action(cx.listener(|this, _: &ShellEnter, _, _| this.shell_enter += 1))
                .on_action(cx.listener(|this, _: &ShellTab, window, _| window.focus(&this.after)))
                .child(
                    div()
                        .h(px(CONTROL_HEIGHT))
                        .overflow_hidden()
                        .child(self.select.clone()),
                )
                .child(
                    div()
                        .id("after-zoom")
                        .track_focus(&self.after)
                        .h(px(40.))
                        .child("Next control"),
                )
        }
    }

    fn setup(cx: &mut TestAppContext) -> (Entity<Host>, &mut VisualTestContext) {
        cx.update(|cx| {
            bind(cx);
            cx.bind_keys([
                KeyBinding::new("enter", ShellEnter, Some("Shell")),
                KeyBinding::new("tab", ShellTab, Some("Shell")),
            ]);
        });
        cx.add_window_view(|window, cx| {
            let select = cx.new(ZoomSelect::new);
            select.update(cx, |this, cx| this.sync(true, 0.52, true, false, cx));
            window.focus(&select.read(cx).focus_handle());
            cx.subscribe(&select, |this: &mut Host, _, choice: &ZoomChoice, cx| {
                this.chosen.push(*choice);
                cx.notify();
            })
            .detach();
            cx.subscribe(
                &select,
                |this: &mut Host, _, _: &ImageControlsRequested, cx| {
                    this.help_requests += 1;
                    cx.notify();
                },
            )
            .detach();
            Host {
                select,
                after: cx.focus_handle(),
                chosen: vec![],
                help_requests: 0,
                shell_enter: 0,
            }
        })
    }

    #[core::prelude::v1::test]
    fn label_and_selection_distinguish_automatic_fit_from_the_same_scale() {
        assert_eq!(label(true, 0.52), "Zoom: Fit width · 52%");
        assert_eq!(label(false, 0.75), "Zoom: 75%");
        assert_eq!(current_index(true, 0.5), Some(0));
        assert_eq!(current_index(false, 0.5), Some(3));
        assert_eq!(current_index(false, 1.), Some(1));
        assert_eq!(current_index(false, 0.523), None);
    }

    #[core::prelude::v1::test]
    fn menu_geometry_stays_inside_small_windows_and_flips_above() {
        let viewport = size(px(820.), px(600.));
        let trigger = Bounds::new(point(px(700.), px(500.)), size(px(232.), px(40.)));
        let popup = popup_bounds(trigger, viewport, OPTIONS.len());
        assert!(popup.right() <= viewport.width - px(8.));
        assert_eq!(popup.bottom() + px(4.), trigger.top());
        let popup = popup_bounds(trigger, size(px(280.), px(220.)), OPTIONS.len());
        assert!(popup.right() <= px(272.));
        assert!(popup.bottom() <= px(212.));
    }

    #[gpui::test]
    fn keyboard_selects_fit_actual_size_and_presets_without_shell_activation(
        cx: &mut TestAppContext,
    ) {
        let (host, cx) = setup(cx);
        cx.simulate_keystrokes("enter down enter");
        cx.update(|window, cx| {
            let host = host.read(cx);
            assert_eq!(host.chosen, [ZoomChoice::Percent(100)]);
            assert_eq!(host.shell_enter, 0);
            assert!(!host.select.read(cx).open);
            assert!(host.select.read(cx).focus.is_focused(window));
        });
        cx.simulate_keystrokes("space end enter enter home enter");
        cx.update(|_, cx| {
            assert_eq!(
                host.read(cx).chosen,
                [
                    ZoomChoice::Percent(100),
                    ZoomChoice::Percent(200),
                    ZoomChoice::FitWidth
                ]
            )
        });
        cx.simulate_keystrokes("enter down escape");
        cx.update(|_, cx| assert_eq!(host.read(cx).chosen.len(), 3));
    }

    #[gpui::test]
    fn pointer_menu_escapes_clip_and_tab_dismisses_without_focus_trap(cx: &mut TestAppContext) {
        let (host, cx) = setup(cx);
        let trigger = cx.debug_bounds("zoom-select-button").unwrap();
        cx.simulate_click(trigger.center(), Modifiers::none());
        cx.run_until_parked();
        let popup = cx.debug_bounds("zoom-popup").unwrap();
        assert!(popup.top() > trigger.bottom());
        assert_eq!(cx.debug_bounds("zoom-select-button").unwrap(), trigger);
        let actual = cx.debug_bounds("zoom-option-1").unwrap();
        cx.simulate_click(actual.center(), Modifiers::none());
        cx.update(|_, cx| assert_eq!(host.read(cx).chosen, [ZoomChoice::Percent(100)]));
        cx.simulate_keystrokes("enter tab");
        cx.run_until_parked();
        cx.update(|window, cx| {
            let host = host.read(cx);
            assert!(!host.select.read(cx).open);
            assert!(host.after.is_focused(window));
        });
    }

    #[gpui::test]
    fn outside_click_and_disabling_close_without_selecting(cx: &mut TestAppContext) {
        let (host, cx) = setup(cx);
        let select = cx.update(|_, cx| host.read(cx).select.clone());
        cx.simulate_keystrokes("enter");
        cx.simulate_click(point(px(500.), px(10.)), Modifiers::none());
        cx.update(|_, cx| {
            assert!(!select.read(cx).open);
            assert!(host.read(cx).chosen.is_empty());
        });
        select.update(cx, |this, cx| this.sync(false, 0.75, false, false, cx));
        cx.run_until_parked();
        let button = cx.debug_bounds("zoom-select-button").unwrap();
        cx.simulate_click(button.center(), Modifiers::none());
        cx.update(|window, cx| {
            select.update(cx, |this, cx| {
                this.toggle(window, cx);
                this.select(1, window, cx);
                assert!(!this.open);
                assert!(!this.focus.is_focused(window));
            });
            assert!(host.read(cx).chosen.is_empty());
        });
    }

    #[gpui::test]
    fn compact_zoom_keeps_both_lines_inside_one_control(cx: &mut TestAppContext) {
        let (host, cx) = setup(cx);
        let select = cx.update(|_, cx| host.read(cx).select.clone());
        select.update(cx, |this, cx| this.sync(true, 4., true, true, cx));
        cx.run_until_parked();
        let button = cx.debug_bounds("zoom-select-button").unwrap();
        assert_eq!(button.size, size(px(112.), px(CONTROL_HEIGHT)));
        for selector in ["zoom-caption", "zoom-scale"] {
            let line = cx.debug_bounds(selector).unwrap();
            assert_eq!(line.size.height, px(16.));
            assert!(line.top() >= button.top() && line.bottom() <= button.bottom());
        }
    }

    #[gpui::test]
    fn compact_help_is_a_separate_keyboard_and_pointer_action(cx: &mut TestAppContext) {
        let (host, cx) = setup(cx);
        let select = cx.update(|_, cx| host.read(cx).select.clone());
        select.update(cx, |this, cx| {
            this.sync(true, 0.52, true, true, cx);
            this.set_image_controls_available(true, cx);
        });
        cx.simulate_keystrokes("enter end enter");
        cx.update(|window, cx| {
            let host = host.read(cx);
            assert_eq!(host.help_requests, 1);
            assert!(host.chosen.is_empty());
            assert!(host.select.read(cx).focus.is_focused(window));
            assert!(!host.select.read(cx).open);
        });
        cx.simulate_keystrokes("enter end");
        let help = cx.debug_bounds("zoom-image-controls").unwrap();
        cx.simulate_click(help.center(), Modifiers::none());
        cx.update(|_, cx| {
            assert_eq!(host.read(cx).help_requests, 2);
            assert!(host.read(cx).chosen.is_empty());
        });
        select.update(cx, |this, cx| this.set_image_controls_available(false, cx));
        cx.simulate_keystrokes("enter end enter");
        cx.update(|_, cx| {
            assert_eq!(host.read(cx).help_requests, 2);
            assert_eq!(host.read(cx).chosen, [ZoomChoice::Percent(200)]);
        });
    }
}
