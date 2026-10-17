// SPDX-License-Identifier: MPL-2.0
//! Shared in-app modality. File pickers remain platform-owned.
use super::{
    controls::{self, ActionStyle},
    scrollbar::ScrollbarState,
    theme::*,
};
use gpui::{prelude::*, *};

/// All dialogs use the same inset, scrim and occluding pointer boundary.
pub fn overlay() -> Div {
    div()
        .occlude()
        .absolute()
        .inset_0()
        .size_full()
        .p_4()
        .flex()
        .items_center()
        .justify_center()
        .bg(rgba((palette().shadow << 8) | 0x66))
}

/// Set type explicitly: GPUI prompts render outside the shell's inherited style.
pub fn surface(width: f32) -> Div {
    div()
        .w(px(width))
        .max_w_full()
        .max_h_full()
        .min_w_0()
        .min_h_0()
        .flex()
        .flex_col()
        .bg(rgb(palette().surface))
        .shadow(super::depth::Elevation::Dialog.shadows())
        .border_1()
        .border_color(rgb(palette().control_line))
        .rounded(px(4.))
        .font_family(FONT)
        .text_size(px(BODY_TEXT_SIZE))
        .line_height(px(BODY_LINE_HEIGHT))
        .text_color(rgb(palette().ink))
}

pub fn heading() -> Div {
    div()
        .flex_shrink_0()
        .text_size(px(TITLE_TEXT_SIZE))
        .line_height(px(TITLE_LINE_HEIGHT))
        .font_weight(FontWeight::SEMIBOLD)
}

/// Inset by its owner, with a rule separating the decision from its explanation.
pub fn footer() -> Div {
    div()
        .flex_shrink_0()
        .border_t_1()
        .border_color(rgb(palette().line))
        .pt_4()
}

/// `Cancel` is safe/dismissive, `Ok` is primary, and warning/critical `Other`
/// actions are destructive. Call sites declare roles, never infer them from copy.
fn action_style(level: PromptLevel, button: &PromptButton) -> ActionStyle {
    match button {
        PromptButton::Ok(_) => ActionStyle::Primary,
        PromptButton::Other(_) if level != PromptLevel::Info => ActionStyle::Danger,
        _ => ActionStyle::Secondary,
    }
}

/// Use the app's controls for every confirmation, including on platforms where
/// GPUI would otherwise show its unstyled fallback. Native file pickers are not
/// affected. PromptHandle retains the existing async answer and focus lifecycle.
pub fn install(cx: &mut App) {
    cx.set_prompt_builder(|level, title, detail, buttons, handle, window, cx| {
        let view = cx.new(|cx| Confirmation::new(level, title, detail, buttons, cx));
        handle.with_view(view, window, cx)
    });
}

struct Confirmation {
    level: PromptLevel,
    title: SharedString,
    detail: Option<SharedString>,
    buttons: Vec<PromptButton>,
    order: Vec<usize>,
    focus: Vec<FocusHandle>,
    cancel: usize,
    answered: bool,
    scroll: ScrollHandle,
    scrollbar: ScrollbarState,
}

impl Confirmation {
    fn new(
        level: PromptLevel,
        title: &str,
        detail: Option<&str>,
        buttons: &[PromptButton],
        cx: &mut Context<Self>,
    ) -> Self {
        assert!(!buttons.is_empty(), "a confirmation needs a safe action");
        let cancel = buttons
            .iter()
            .position(|b| matches!(b, PromptButton::Cancel(_)))
            .unwrap_or(0);
        let mut order: Vec<_> = (0..buttons.len()).collect();
        // Three-way quit: Discard, Cancel, Save. Preserve answer indices while
        // keeping the primary action at the trailing edge and Tab in paint order.
        if buttons.len() > 2 && buttons.iter().any(|b| matches!(b, PromptButton::Ok(_))) {
            order.sort_by_key(|&i| match buttons[i] {
                PromptButton::Other(_) => 0,
                PromptButton::Cancel(_) => 1,
                PromptButton::Ok(_) => 2,
            });
        }
        Self {
            level,
            title: title.to_owned().into(),
            detail: detail
                .filter(|s| !s.is_empty())
                .map(|s| s.to_owned().into()),
            buttons: buttons.to_vec(),
            order,
            focus: buttons
                .iter()
                .map(|_| cx.focus_handle().tab_stop(false))
                .collect(),
            cancel,
            answered: false,
            scroll: ScrollHandle::new(),
            scrollbar: ScrollbarState::default(),
        }
    }

    fn answer(&mut self, index: usize, cx: &mut Context<Self>) {
        if !self.answered {
            self.answered = true;
            cx.emit(PromptResponse(index));
        }
    }

    fn key_down(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        // No underlying form shortcuts, implicit saves or background activation.
        cx.stop_propagation();
        let key = &event.keystroke;
        if key.modifiers.control || key.modifiers.platform || key.modifiers.alt {
            return;
        }
        let focused = self
            .focus
            .iter()
            .position(|f| f.is_focused(window))
            .unwrap_or(self.cancel);
        match key.key.as_str() {
            "escape" => self.answer(self.cancel, cx),
            "enter" | "space" if !event.is_held => self.answer(focused, cx),
            "tab" | "left" | "right" => {
                let current = self.order.iter().position(|&i| i == focused).unwrap_or(0);
                let backward = key.modifiers.shift || key.key == "left";
                let next =
                    (current + if backward { self.order.len() - 1 } else { 1 }) % self.order.len();
                window.focus(&self.focus[self.order[next]]);
                cx.notify();
            }
            "up" | "down" | "pageup" | "pagedown" | "home" | "end" => {
                let page = self.scroll.bounds().size.height * 0.9;
                let offset = self.scroll.offset();
                let y = match key.key.as_str() {
                    "home" => px(0.),
                    "end" => -self.scroll.max_offset().height,
                    "up" => offset.y + px(22.),
                    "down" => offset.y - px(22.),
                    "pageup" => offset.y + page,
                    _ => offset.y - page,
                }
                .clamp(-self.scroll.max_offset().height, px(0.));
                self.scroll.set_offset(point(offset.x, y));
                cx.notify();
            }
            _ => (),
        }
    }
}

impl EventEmitter<PromptResponse> for Confirmation {}
impl Focusable for Confirmation {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus[self.cancel].clone()
    }
}

impl Render for Confirmation {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        overlay()
            .debug_selector(|| "confirmation-overlay".into())
            .key_context("Confirmation")
            .on_key_down(cx.listener(Self::key_down))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_scroll_wheel(|_, _, cx| cx.stop_propagation())
            .child(
                surface(560.)
                    .debug_selector(|| "confirmation-dialog".into())
                    .p_6()
                    .gap_4()
                    .child(
                        heading()
                            .debug_selector(|| "confirmation-title".into())
                            .child(self.title.clone()),
                    )
                    .when_some(self.detail.clone(), |panel, detail| {
                        panel.child(
                            div()
                                .relative()
                                .min_h_0()
                                .flex()
                                .flex_col()
                                .pr(px(super::scrollbar::GUTTER))
                                .child(
                                    div()
                                        .id("confirmation-body")
                                        .debug_selector(|| "confirmation-body".into())
                                        .min_h_0()
                                        .max_h(px(320.))
                                        .overflow_y_scroll()
                                        .track_scroll(&self.scroll)
                                        .text_color(rgb(palette().muted))
                                        .child(
                                            div()
                                                .debug_selector(|| "confirmation-copy".into())
                                                .child(detail),
                                        ),
                                )
                                .child(self.scrollbar.element(&self.scroll, cx.entity_id())),
                        )
                    })
                    .child(
                        footer()
                            .debug_selector(|| "confirmation-footer".into())
                            .flex()
                            .flex_wrap()
                            .justify_end()
                            .gap_3()
                            .children(self.order.iter().map(|&i| {
                                controls::action(
                                    ("confirmation", i),
                                    self.buttons[i].label().clone(),
                                    &self.focus[i],
                                    action_style(self.level, &self.buttons[i]),
                                    true,
                                )
                                .debug_selector(move || format!("confirmation-action-{i}"))
                                .on_click(cx.listener(move |this, _, _, cx| this.answer(i, cx)))
                            })),
                    ),
            )
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    struct Harness {
        trigger: FocusHandle,
        answers: Vec<usize>,
        background_clicks: usize,
    }
    impl Harness {
        fn show(
            &mut self,
            detail: &str,
            three_way: bool,
            window: &mut Window,
            cx: &mut Context<Self>,
        ) {
            window.focus(&self.trigger);
            let buttons = if three_way {
                vec![
                    PromptButton::cancel("Cancel"),
                    PromptButton::ok("Save and quit"),
                    PromptButton::new("Discard and quit"),
                ]
            } else {
                vec![
                    PromptButton::cancel("Keep reviewing"),
                    PromptButton::ok("Save baseline"),
                ]
            };
            let answer = window.prompt(
                PromptLevel::Warning,
                "Save these screenshots as the baseline?",
                Some(detail),
                &buttons,
                cx,
            );
            cx.spawn_in(window, async move |this, cx| {
                let answer = answer.await.unwrap();
                this.update(cx, |this, cx| {
                    this.answers.push(answer);
                    cx.notify();
                })
                .unwrap();
            })
            .detach();
        }
    }
    impl Render for Harness {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div().size_full().bg(rgb(palette().ground)).child(
                controls::action(
                    "background",
                    "Background action",
                    &self.trigger,
                    ActionStyle::Secondary,
                    true,
                )
                .on_click(cx.listener(|this, _, _, _| this.background_clicks += 1)),
            )
        }
    }

    fn inside(inner: Bounds<Pixels>, outer: Bounds<Pixels>) {
        assert!(
            inner.left() >= outer.left() && inner.right() <= outer.right(),
            "{inner:?} outside {outer:?}"
        );
        assert!(
            inner.top() >= outer.top() && inner.bottom() <= outer.bottom(),
            "{inner:?} outside {outer:?}"
        );
    }

    #[test]
    fn confirmation_roles_do_not_depend_on_button_wording() {
        assert!(matches!(
            action_style(
                PromptLevel::Warning,
                &PromptButton::ok("Save & skip missing")
            ),
            ActionStyle::Primary
        ));
        assert!(matches!(
            action_style(PromptLevel::Warning, &PromptButton::new("Delete files")),
            ActionStyle::Danger
        ));
        assert!(matches!(
            action_style(
                PromptLevel::Critical,
                &PromptButton::cancel("Keep everything")
            ),
            ActionStyle::Secondary
        ));
    }

    #[gpui::test]
    fn confirmations_trap_keys_restore_focus_and_preserve_answer_indices(cx: &mut TestAppContext) {
        cx.update(install);
        let (view, cx) = cx.add_window_view(|_, cx| Harness {
            trigger: cx.focus_handle(),
            answers: Vec::new(),
            background_clicks: 0,
        });
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            for (keys, three_way, expected) in [
                ("enter", false, 0),
                ("escape", false, 0),
                ("tab tab shift-tab space", false, 1),
                ("tab tab tab shift-tab enter", true, 2),
            ] {
                cx.update(|window, cx| view.update(cx, |this, cx| this.show("Future checks will compare against these screenshots. The page list stays the same.", three_way, window, cx)));
                cx.run_until_parked();
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let panel = cx.debug_bounds("confirmation-dialog").unwrap();
                let title = cx.debug_bounds("confirmation-title").unwrap();
                let body = cx.debug_bounds("confirmation-body").unwrap();
                let footer = cx.debug_bounds("confirmation-footer").unwrap();
                inside(
                    panel,
                    Bounds::new(
                        point(px(16.), px(16.)),
                        size(px(width - 32.), px(height - 32.)),
                    ),
                );
                assert_eq!(title.left(), panel.left() + px(25.));
                assert!(body.top() >= title.bottom() + px(16.));
                assert!(footer.top() >= body.bottom() + px(16.));
                for selector in [
                    "confirmation-action-0",
                    "confirmation-action-1",
                    "confirmation-action-2",
                ]
                .iter()
                .take(if three_way { 3 } else { 2 })
                {
                    let action = cx.debug_bounds(selector).unwrap();
                    inside(action, footer);
                    assert!(action.size.height >= px(CONTROL_HEIGHT));
                }
                let background = cx.debug_bounds("action-background").unwrap();
                cx.simulate_click(background.center(), Modifiers::none());
                assert_eq!(view.read_with(cx, |this, _| this.background_clicks), 0);
                cx.simulate_keystrokes("ctrl-enter");
                let before = view.read_with(cx, |this, _| this.answers.len());
                for key in keys.split(' ') {
                    cx.simulate_keystrokes(key);
                }
                cx.run_until_parked();
                view.read_with(cx, |this, _| {
                    assert_eq!(this.answers.len(), before + 1);
                    assert_eq!(this.answers.last(), Some(&expected));
                });
                cx.update(|window, cx| {
                    view.read_with(cx, |this, _| assert!(this.trigger.is_focused(window)))
                });
            }
        }
    }

    #[gpui::test]
    fn long_confirmation_scrolls_without_moving_its_actions(cx: &mut TestAppContext) {
        cx.update(install);
        let (view, cx) = cx.add_window_view(|_, cx| Harness {
            trigger: cx.focus_handle(),
            answers: Vec::new(),
            background_clicks: 0,
        });
        let detail =
            "Long consequence and source details remain readable without hiding the decision.\n"
                .repeat(60);
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                view.update(cx, |this, cx| this.show(&detail, false, window, cx))
            });
            cx.run_until_parked();
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let body = cx.debug_bounds("confirmation-body").unwrap();
            let footer = cx.debug_bounds("confirmation-footer").unwrap();
            let copy = cx.debug_bounds("confirmation-copy").unwrap();
            assert!(copy.size.height > body.size.height && body.size.height <= px(320.));
            cx.simulate_event(ScrollWheelEvent {
                position: body.center(),
                delta: ScrollDelta::Pixels(point(px(0.), px(-100.))),
                ..Default::default()
            });
            cx.run_until_parked();
            assert!(cx.debug_bounds("confirmation-copy").unwrap().top() < copy.top());
            cx.simulate_keystrokes("end");
            cx.run_until_parked();
            let last = cx.debug_bounds("confirmation-copy").unwrap();
            assert!(last.top() < copy.top());
            assert!((last.bottom() - body.bottom()).abs() <= px(1.));
            assert_eq!(cx.debug_bounds("confirmation-footer").unwrap(), footer);
            cx.simulate_keystrokes("home");
            cx.run_until_parked();
            assert_eq!(
                cx.debug_bounds("confirmation-copy").unwrap().top(),
                copy.top()
            );
            cx.simulate_keystrokes("escape");
            cx.run_until_parked();
        }
    }

    #[gpui::test]
    fn pointer_confirmation_returns_the_original_index_and_restores_its_trigger(
        cx: &mut TestAppContext,
    ) {
        cx.update(install);
        let (view, cx) = cx.add_window_view(|_, cx| Harness {
            trigger: cx.focus_handle(),
            answers: Vec::new(),
            background_clicks: 0,
        });
        cx.update(|window, cx| {
            view.update(cx, |this, cx| {
                this.show("Save only after reviewing.", true, window, cx)
            })
        });
        cx.run_until_parked();
        let discard = cx.debug_bounds("confirmation-action-2").unwrap();
        let cancel = cx.debug_bounds("confirmation-action-0").unwrap();
        let save = cx.debug_bounds("confirmation-action-1").unwrap();
        assert!(discard.right() < cancel.left() && cancel.right() < save.left());
        cx.simulate_click(save.center(), Modifiers::none());
        cx.run_until_parked();
        view.read_with(cx, |this, _| assert_eq!(this.answers, [1]));
        cx.update(|window, cx| {
            view.read_with(cx, |this, _| assert!(this.trigger.is_focused(window)))
        });
    }
}
