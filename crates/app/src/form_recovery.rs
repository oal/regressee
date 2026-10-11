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
        if self.focus.screen == Screen::Setup {
            match control {
                Control::Name | Control::Address => {
                    if self.focus.setup_stage != SetupStage::Settings {
                        self.focus.setup_stage = SetupStage::Website;
                    }
                }
                Control::CrawlField(0..=8) => {
                    if self.focus.setup_stage != SetupStage::Settings {
                        self.focus.setup_stage = SetupStage::Pages;
                    }
                    if matches!(control, Control::CrawlField(3..=8)) {
                        self.focus.advanced = true;
                    }
                }
                Control::CrawlField(10..=15) => {
                    self.focus.setup_stage = SetupStage::Settings;
                    self.focus.advanced = true;
                }
                _ => (),
            }
        }
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
            anchor.scroll_to(window, cx);
        }
    }

    pub(super) fn show_keyboard_help(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.native_prompt || self.focus.dialog || self.focus.approval_dialog {
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
                "About this preview",
                "Regressee discovers pages within approved websites, captures chosen screen sizes and checks only an explicitly saved baseline. Projects, screenshots and review history stay local. Login sessions are never included in baseline ZIPs; screenshots may contain private information.\n\nThis is a private preview. Development browsers remain restricted to authorized local fixtures. Native accessibility, cross-platform and public-release qualification remain pending; macOS capture is blocked by a process-ownership code gap. MPL-2.0.",
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
