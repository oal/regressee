// SPDX-License-Identifier: MPL-2.0
//! Local, read-only validation wayfinding and discoverable keyboard help.
use super::*;

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
            3 => "Link depth",
            4 => "Parallel active pages",
            5 => "Parallel pages per website",
            6 => "Retries",
            7 => "Custom width",
            8 => "Custom height",
            10 => "Login / consent page",
            11 => "Representative page",
            12 => "Identity-provider origins",
            13 => "Source mapping base",
            14 => "Destination mapping base",
            15 => "Capture language",
            17 => "Proxy server",
            18 => "Trusted CA file",
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
        if self.focus.screen != Screen::Setup
            || !matches!(control,Control::Name|Control::Address|Control::CrawlField(_)|Control::Choice(_)|Control::Environment(_)|Control::EnvironmentOptions|Control::Guided(1..=12|25|27|31..=38)|Control::Details(Detail::Capture)|Control::CloseMappingPreview)
            || (self.focus.setup_stage == SetupStage::Access
                && matches!(control, Control::Guided(31 | 32)))
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
            .text_color(rgb(ERROR))
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
        if !matches!(self.focus.screen, Screen::Setup | Screen::Settings) {
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
            (
                "About Regressee",
                concat!(
                    "Regressee ",
                    env!("CARGO_PKG_VERSION"),
                    " · MPL-2.0\n\nRegressee discovers pages within approved websites, captures chosen screen sizes and checks only an explicitly saved baseline. Projects, screenshots and review history stay local. Login sessions are never included in baseline ZIPs; screenshots may contain private information.\n\nDevelopment browsers remain restricted to authorized local fixtures. Native accessibility, cross-platform and public-release qualification remain pending; macOS capture is blocked by a process-ownership code gap."
                ),
            )
        };
        controls::dialog(
            title,
            body,
            self.button(Control::CloseDialog, "Close", ActionStyle::Primary, cx),
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
