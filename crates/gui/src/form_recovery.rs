// SPDX-License-Identifier: MPL-2.0
//! Local, read-only validation wayfinding and discoverable keyboard help.
use super::*;

pub(super) const REPOSITORY_URL: &str = "https://github.com/oal/regressee";

const ABOUT_TEXT: &str = concat!(
    "Regressee ",
    env!("CARGO_PKG_VERSION"),
    " · MPL-2.0\n\n",
    "Compare website screenshots with a saved baseline to spot changes.\n\n",
    "Your projects, screenshots and review history stay on your computer. ",
    "Shared baseline ZIPs exclude logins and cookies, but screenshots may contain private information.\n\n",
    "In development: accessibility and platform support are not fully tested. ",
    "Development browsers are limited to approved local websites. ",
    "Website capture is not yet supported on macOS."
);

/// Shared controls have a one-pixel border. Their explicitly top/left-anchored
/// absolute canvas measures the padding box, so include the border for reveal.
pub(super) fn bordered_control_bounds(painted: Bounds<Pixels>) -> Bounds<Pixels> {
    Bounds::new(
        gpui::point(painted.left() - px(1.), painted.top() - px(1.)),
        size(painted.size.width + px(2.), painted.size.height + px(2.)),
    )
}

pub(super) fn next_correction(errors: &[Control], current: Option<Control>) -> Option<Control> {
    if errors.is_empty() {
        return None;
    }
    let index = current
        .and_then(|current| errors.iter().position(|control| *control == current))
        .map_or(0, |index| (index + 1) % errors.len());
    Some(errors[index])
}

fn field_label(control: Control) -> &'static str {
    match control {
        Control::Name => "Project name",
        Control::Address => "Starting addresses",
        Control::CrawlField(index) => match index {
            0 => "Maximum pages",
            1 => "Website section",
            2 => "Skipped address text",
            3 => "Links away from starting addresses",
            4 => "Pages at once",
            5 => "Pages at once per website",
            6 => "Retries",
            7 => "Custom width",
            8 => "Custom height",
            10 => "Login / consent page",
            11 => "Page to check after login",
            12 => "Approved login websites",
            13 => "Original website or section",
            14 => "Matching website or section",
            15 => "Website language",
            17 => "Proxy server",
            18 => "Trusted certificate file",
            _ => "Setting",
        },
        _ => "Setting",
    }
}

impl Shell {
    pub(super) fn setup_control_bounds(
        &self,
        control: Control,
    ) -> Option<std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>> {
        let secondary_action = matches!(self.focus.screen, Screen::Projects | Screen::Settings)
            && matches!(
                control,
                Control::Phase6(1 | 3 | 6 | 7 | 13 | 14 | 15)
                    | Control::Phase7(_)
                    | Control::Details(_)
                    | Control::CrawlField(17 | 18)
                    | Control::Guided(41 | 42)
                    | Control::SetupRuntime
                    | Control::About
                    | Control::Appearance(_)
                    | Control::Shortcuts
            );
        if !secondary_action
            && (self.focus.screen != Screen::Setup
                || !matches!(control,Control::Name|Control::Address|Control::CrawlField(_)|Control::Choice(_)|Control::Environment(_)|Control::EnvironmentOptions|Control::Guided(1..=12|25|27|31..=38)|Control::Details(Detail::Capture)|Control::CloseMappingPreview)
                || (self.focus.setup_stage == SetupStage::Access
                    && matches!(control, Control::Guided(31 | 32))))
        {
            return None;
        }
        if let Some(bounds) = self.options_control_bounds(control) {
            return Some(bounds);
        }
        let mut paint = self.setup_paint.borrow_mut();
        if let Some((_, bounds)) = paint.iter().find(|(c, _)| *c == control) {
            return Some(bounds.clone());
        }
        let bounds = std::rc::Rc::new(std::cell::Cell::new(None));
        paint.push((control, bounds.clone()));
        Some(bounds)
    }
    pub(super) fn measured_form_field(&self, control: Control, field: Div) -> Div {
        field.when_some(self.setup_control_bounds(control), |el, bounds| {
            el.relative().child(
                canvas(
                    |_, _, _| (),
                    move |painted, _, _, _| bounds.set(Some(painted)),
                )
                .absolute()
                .size_full(),
            )
        })
    }
    pub(super) fn route_form_correction(&mut self, control: Control) {
        if self.focus.screen == Screen::Settings && matches!(control, Control::CrawlField(17 | 18))
        {
            self.details.insert(Detail::ManagedNetwork);
        }
        if self.focus.screen != Screen::Setup {
            return;
        }
        let guided = self.focus.setup_stage != SetupStage::Settings;
        match control {
            Control::Name | Control::Address if guided => {
                self.focus.setup_stage = SetupStage::Website
            }
            Control::CrawlField(0..=8) => {
                if guided {
                    self.focus.setup_stage = SetupStage::Pages;
                }
                if matches!(control, Control::CrawlField(3..=8)) {
                    self.focus.advanced = true;
                }
            }
            Control::CrawlField(10..=16) => {
                if guided {
                    self.focus.setup_stage = SetupStage::Access;
                }
                self.setup_environment_open = true;
                if matches!(control, Control::CrawlField(10..=12)) {
                    self.setup_login_correction = true;
                }
            }
            _ => (),
        }
    }
    fn form_input(&self, control: Control) -> Option<&Entity<TextInput>> {
        match control {
            Control::Name => Some(&self.name),
            Control::Address => Some(&self.address),
            Control::CrawlField(index) if index != 9 && index != 16 => self.crawl_fields.get(index),
            _ => None,
        }
    }

    pub(super) fn form_anchor(&self, control: Control) -> Option<gpui::ScrollAnchor> {
        let index = match control {
            Control::Name => 0,
            Control::Address => 1,
            Control::CrawlField(index) if index != 9 && index != 16 => index + 2,
            _ => return None,
        };
        self.form_anchors.get(index).cloned()
    }

    pub(super) fn form_errors(&self, cx: &App) -> Vec<(Control, &'static str, &'static str)> {
        let controls: Vec<_> = match self.focus.screen {
            Screen::Setup => [Control::Name, Control::Address]
                .into_iter()
                .chain(
                    (0..=15)
                        .filter(|index| *index != 9)
                        .map(Control::CrawlField),
                )
                .collect(),
            Screen::Settings => vec![Control::CrawlField(17), Control::CrawlField(18)],
            _ => return Vec::new(),
        };
        controls
            .into_iter()
            .filter_map(|control| {
                let input = self.form_input(control)?.read(cx);
                input.touched.then_some(())?;
                Some((control, field_label(control), input.validation_error()?))
            })
            .collect()
    }

    pub(super) fn form_error_summary(&self, cx: &mut Context<Self>) -> AnyElement {
        let errors = self.form_errors(cx);
        if errors.is_empty() {
            return div().into_any_element();
        }
        div()
            .debug_selector(|| "form-correction-summary".into())
            .w_full()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_1()
            .text_size(px(13.))
            .line_height(px(HELP_LINE_HEIGHT))
            .text_color(rgb(palette().error))
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Correct these fields before saving"),
            )
            .children(
                errors
                    .iter()
                    .take(3)
                    .map(|(_, label, message)| div().child(format!("{label}: {message}"))),
            )
            .when(errors.len() > 3, |body| {
                body.child(format!(
                    "{} other fields also need attention.",
                    errors.len() - 3
                ))
            })
            .child(self.button(
                Control::ValidationNext,
                "Go to next correction",
                ActionStyle::Secondary,
                cx,
            ))
            .into_any_element()
    }

    pub(super) fn go_to_next_correction(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let errors: Vec<_> = self.form_errors(cx).iter().map(|error| error.0).collect();
        let current = self
            .focused(window, cx)
            .filter(|control| errors.contains(control))
            .or(self.last_validation_field);
        let Some(control) = next_correction(&errors, current) else {
            return;
        };
        self.route_form_correction(control);
        self.last_validation_field = Some(control);
        self.presentation_focus = Some(control);
        self.reveal_form_control(control, true, window, cx);
        cx.notify();
    }

    pub(super) fn reveal_form_control(
        &self,
        control: Control,
        force: bool,
        window: &mut Window,
        cx: &mut App,
    ) {
        if !matches!(
            self.focus.screen,
            Screen::Setup | Screen::Settings | Screen::Projects
        ) {
            return;
        }
        if let Some(bounds) = self.setup_control_bounds(control) {
            let reveal = |b: Bounds<Pixels>, scroll: &gpui::ScrollHandle| {
                let clip = scroll.bounds();
                let offset = scroll.offset();
                let delta = if b.top() < clip.top() + px(8.) {
                    clip.top() + px(8.) - b.top()
                } else if b.bottom() > clip.bottom() - px(8.) {
                    clip.bottom() - px(8.) - b.bottom()
                } else {
                    px(0.)
                };
                scroll.set_offset(gpui::point(
                    offset.x,
                    (offset.y + delta).clamp(-scroll.max_offset().height, px(0.)),
                ));
            };
            if let Some(b) = bounds.get() {
                reveal(b, &self.screen_scroll);
            }
            let scroll = self.screen_scroll.clone();
            window.on_next_frame(move |_, _| {
                if let Some(b) = bounds.get() {
                    reveal(b, &scroll);
                }
            });
            return;
        }
        if matches!(
            control,
            Control::CloseMappingPreview | Control::Guided(35 | 36)
        ) && self.control_mounted(control)
            && let Some(bounds) = self
                .options_control_bounds(control)
                .and_then(|bounds| bounds.get())
        {
            let viewport = self.screen_scroll.bounds();
            let delta = if bounds.top() < viewport.top() + px(8.) {
                viewport.top() + px(8.) - bounds.top()
            } else if bounds.bottom() > viewport.bottom() - px(8.) {
                viewport.bottom() - px(8.) - bounds.bottom()
            } else {
                px(0.)
            };
            let offset = self.screen_scroll.offset();
            self.screen_scroll
                .set_offset(gpui::point(offset.x, offset.y + delta));
            return;
        }
        let Some(anchor) = self.form_anchor(control) else {
            return;
        };
        let viewport = self.screen_scroll.bounds();
        let outside = self
            .form_input(control)
            .and_then(|input| input.read(cx).rendered_bounds())
            .is_none_or(|bounds| {
                bounds.top() < viewport.top()
                    || bounds.bottom() + px(HELP_LINE_HEIGHT + 4.) > viewport.bottom()
            });
        if force || outside {
            if let Some(bounds) = self
                .form_input(control)
                .and_then(|input| input.read(cx).rendered_bounds())
            {
                // Reveal the painted input, with room for its persistent label.
                // This also works before a host animation-frame callback arrives.
                let offset = self.screen_scroll.offset();
                let delta = viewport.top() + px(64.) - bounds.top();
                self.screen_scroll.set_offset(gpui::point(
                    offset.x,
                    (offset.y + delta).clamp(-self.screen_scroll.max_offset().height, px(0.)),
                ));
            }
            // The next frame uses the newly mounted stage's anchor, not cached
            // geometry from the previous form. Keep focus-reveal correct on validation.
            anchor.scroll_to(window, cx);
        }
    }

    pub(super) fn show_keyboard_help(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.task.is_some()
            || self.native_prompt
            || self.focus.dialog
            || self.focus.approval_dialog
        {
            return;
        }
        self.restore_focus = self.focused(window, cx);
        self.help_dialog = true;
        self.focus.dialog = true;
        window.focus(&self.handle(Control::CloseDialog, cx));
        cx.notify();
    }

    pub(super) fn information_dialog(&self, cx: &mut Context<Self>) -> AnyElement {
        let (title, body) = if self.help_dialog {
            ("Keyboard shortcuts", keyboard_help())
        } else {
            ("About Regressee", ABOUT_TEXT)
        };
        let heading = if self.help_dialog {
            div().child(title).into_any_element()
        } else {
            brand::lockup(Background::workspace(), title).into_any_element()
        };
        controls::dialog(
            heading,
            body,
            div()
                .w_full()
                .flex()
                .flex_col()
                .gap_3()
                .when(!self.help_dialog, |footer| {
                    let control = Control::AboutRepository;
                    footer.child(
                        regressee_gui::ui::web_link::web_link(
                            "AboutRepository",
                            REPOSITORY_URL,
                            &self.handle(control, cx),
                            self.control_enabled(control, cx),
                        )
                        .on_click(cx.listener(
                            move |this, event, window, cx| {
                                this.click_control(control, event, window, cx);
                            },
                        )),
                    )
                })
                .child(div().w_full().flex().justify_end().child(self.button(
                    Control::CloseDialog,
                    "Close",
                    ActionStyle::Primary,
                    cx,
                )))
                .into_any_element(),
        )
        .into_any_element()
    }
}

fn keyboard_help() -> &'static str {
    #[cfg(target_os = "macos")]
    {
        "⌘ N — New project\n⌘ S — Save settings\n⌘ F — Search pages or History\n⌘ 1…5 — Switch sections\n⌘ Q — Quit (checks drafts)\n\nTab / Shift Tab — Move focus\nEnter / Space — Activate\nEsc — Close a panel or dialog\nF1 — Keyboard help\nImages: drag / scroll to move.\n⌘ or Ctrl + scroll — Zoom at pointer.\nImage focused: + / − zoom; arrows move.\n0 — Fit width · 1 — Actual size\nReview does not update baselines.\nPreview and confirm updates separately."
    }
    #[cfg(not(target_os = "macos"))]
    {
        "Ctrl N — New project\nCtrl S — Save settings\nCtrl F — Search pages or History\nCtrl 1…5 — Switch sections\nCtrl Q — Quit (checks drafts)\n\nTab / Shift Tab — Move focus\nEnter / Space — Activate\nEsc — Close a panel or dialog\nF1 — Keyboard help\nImages: drag / scroll to move.\nCtrl + scroll — Zoom at pointer.\nImage focused: + / − zoom; arrows move.\n0 — Fit width · 1 — Actual size\nReview does not update baselines.\nPreview and confirm updates separately."
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn about_copy_is_concise_without_losing_privacy_or_development_limits() {
        assert!(ABOUT_TEXT.split_whitespace().count() < 90);
        for required in [
            env!("CARGO_PKG_VERSION"),
            "MPL-2.0",
            "saved baseline",
            "stay on your computer",
            "ZIPs exclude logins and cookies",
            "screenshots may contain private information",
            "accessibility and platform support are not fully tested",
            "limited to approved local websites",
            "not yet supported on macOS",
        ] {
            assert!(ABOUT_TEXT.contains(required), "Missing: {required}");
        }
        assert_eq!(REPOSITORY_URL, "https://github.com/oal/regressee");
    }

    #[test]
    fn corrections_cycle_without_a_submission_or_hidden_target() {
        let errors = [Control::Name, Control::Address, Control::CrawlField(0)];
        assert_eq!(next_correction(&errors, None), Some(Control::Name));
        assert_eq!(
            next_correction(&errors, Some(Control::Name)),
            Some(Control::Address)
        );
        assert_eq!(
            next_correction(&errors, Some(Control::CrawlField(0))),
            Some(Control::Name)
        );
        assert_eq!(
            next_correction(&errors[1..], Some(Control::Name)),
            Some(Control::Address)
        );
        assert_eq!(next_correction(&[], Some(Control::Name)), None);
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod ui_tests {
    use super::*;
    use crate::phase1_setup_tests::inside as assert_bounds_inside;
    use crate::workflow_refinement_tests::finish_jobs;
    use gpui::TestAppContext;

    #[gpui::test]
    fn about_repository_supports_pointer_keyboard_and_dialog_guards(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, finish_jobs);
        assert!(cx.opened_url().is_none());
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.focus.screen = Screen::Settings;
                    assert!(!s.control_enabled(Control::AboutRepository, cx));
                    s.activate_control(Control::About, window, cx);
                    assert_eq!(s.focused(window, cx), Some(Control::CloseDialog));
                    assert!(!s.control_enabled(Control::SetupRuntime, cx));
                });
                let _ = window.draw(cx);
            });
            let dialog = cx.debug_bounds("information-dialog").unwrap();
            let body = cx.debug_bounds("information-dialog-body").unwrap();
            let copy = cx.debug_bounds("information-dialog-copy").unwrap();
            let link = cx.debug_bounds("link-AboutRepository").unwrap();
            let close = cx.debug_bounds("action-CloseDialog").unwrap();
            assert_bounds_inside(
                dialog,
                Bounds::new(gpui::point(px(0.), px(0.)), size(px(width), px(height))),
            );
            for bounds in [body, copy, link, close] {
                assert_bounds_inside(bounds, dialog);
            }
            assert_bounds_inside(copy, body);
            assert!(body.bottom() <= link.top() && link.bottom() <= close.top());
            cx.simulate_click(
                gpui::point(link.left() + px(80.), link.center().y),
                gpui::Modifiers::none(),
            );
            assert_eq!(cx.opened_url().as_deref(), Some(REPOSITORY_URL));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    let request = s.next_request;
                    assert_eq!(s.focused(window, cx), Some(Control::AboutRepository));
                    s.move_focus(false, window, cx);
                    assert_eq!(s.focused(window, cx), Some(Control::CloseDialog));
                    s.move_focus(false, window, cx);
                    assert_eq!(s.focused(window, cx), Some(Control::AboutRepository));
                    s.activate(&Activate, window, cx);
                    s.move_focus(true, window, cx);
                    assert_eq!(s.focused(window, cx), Some(Control::CloseDialog));
                    s.native_prompt = true;
                    assert!(!s.control_enabled(Control::AboutRepository, cx));
                    s.native_prompt = false;
                    s.dismiss(&Dismiss, window, cx);
                    assert_eq!(s.focused(window, cx), Some(Control::About));
                    assert!(!s.control_enabled(Control::AboutRepository, cx));
                    s.show_keyboard_help(window, cx);
                    assert!(!s.control_enabled(Control::AboutRepository, cx));
                    assert!(!s.control_mounted(Control::AboutRepository));
                    s.move_focus(false, window, cx);
                    assert_eq!(s.focused(window, cx), Some(Control::CloseDialog));
                    assert_eq!(s.next_request, request);
                });
                let _ = window.draw(cx);
            });
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| s.dismiss(&Dismiss, window, cx));
            });
        }
        owned.join_after_ui();
    }

    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    #[gpui::test]
    #[ignore = "explicit two-size native offscreen About dialog and link-hover inspection"]
    fn renderer_capture_about_dialog(cx: &mut TestAppContext) {
        let root = crate::ui_renderer_capture::output_root().unwrap();
        cx.enable_offscreen_renderer().unwrap();
        let dir = tempfile::tempdir_in(&root).unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, finish_jobs);
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.focus.screen = Screen::Settings;
                    s.activate_control(Control::About, window, cx);
                });
                let _ = window.draw(cx);
            });
            cx.run_until_parked();
            let link = cx.debug_bounds("link-AboutRepository").unwrap();
            crate::ui_renderer_capture::capture(
                cx,
                &root,
                &format!("about-{width:.0}x{height:.0}-repository-rest"),
            )
            .unwrap();
            let rest = cx.update(|window, _| window.capture_offscreen_frame().unwrap());
            cx.simulate_mouse_move(
                gpui::point(link.left() + px(80.), link.center().y),
                None,
                gpui::Modifiers::none(),
            );
            cx.run_until_parked();
            crate::ui_renderer_capture::capture(
                cx,
                &root,
                &format!("about-{width:.0}x{height:.0}-repository-hover"),
            )
            .unwrap();
            let hover = cx.update(|window, _| window.capture_offscreen_frame().unwrap());
            let mut changed_ink = 0;
            let mut dark_ink_delta = 0;
            for y in f32::from(link.top()) as u32..f32::from(link.bottom()) as u32 {
                for x in f32::from(link.left()) as u32..f32::from(link.right()) as u32 {
                    let offset = ((y * rest.width + x) * 4) as usize;
                    let before = &rest.rgba[offset..offset + 4];
                    let after = &hover.rgba[offset..offset + 4];
                    // Underline thickening may paint an extra glyph row, but
                    // the empty trailing part of the control must stay white.
                    if x > f32::from(link.right() - px(100.)) as u32 {
                        assert_eq!(before, [255, 255, 255, 255]);
                        assert_eq!(before, after, "link hover must not fill its row");
                    }
                    if before != after {
                        changed_ink += 1;
                    }
                    dark_ink_delta += i32::from(after == [0x1b, 0x30, 0x44, 255])
                        - i32::from(before == [0x1b, 0x30, 0x44, 255]);
                }
            }
            assert!(changed_ink > 100, "hover must visibly change the link ink");
            assert!(dark_ink_delta > 100, "hover ink must be visibly darker");
            assert_eq!(link, cx.debug_bounds("link-AboutRepository").unwrap());
            cx.executor()
                .advance_clock(std::time::Duration::from_millis(600));
            cx.run_until_parked();
            crate::ui_renderer_capture::capture(
                cx,
                &root,
                &format!("about-{width:.0}x{height:.0}-repository-dwell"),
            )
            .unwrap();
            assert!(cx.debug_bounds("web-link-hint").is_none());
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| s.move_focus(false, window, cx));
            });
            crate::ui_renderer_capture::capture(
                cx,
                &root,
                &format!("about-{width:.0}x{height:.0}-repository-focused"),
            )
            .unwrap();
            assert_eq!(link, cx.debug_bounds("link-AboutRepository").unwrap());
            cx.simulate_mouse_move(gpui::point(px(0.), px(0.)), None, gpui::Modifiers::none());
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| s.dismiss(&Dismiss, window, cx));
            });
        }
        assert!(cx.opened_url().is_none());
        owned.join_after_ui();
    }
}
