// SPDX-License-Identifier: MPL-2.0
//! Live desktop appearance and the explicit, machine-local settings override.
use super::*;

impl Shell {
    fn apply_appearance(&self, cx: &mut Context<Self>) {
        if set_dark(self.appearance.preference.is_dark(self.desktop_dark)) {
            // Text inputs, selectors and prompts are cached child entities. A
            // shell notification alone leaves some of their old colors painted.
            cx.refresh_windows();
        }
    }

    pub(super) fn sync_appearance(&mut self, cx: &mut Context<Self>) {
        let snapshot = self.jobs.appearance.snapshot();
        if self.appearance != snapshot {
            self.appearance = snapshot;
            self.apply_appearance(cx);
            cx.notify();
        }
    }

    pub(super) fn desktop_appearance_changed(
        &mut self,
        appearance: gpui::WindowAppearance,
        cx: &mut Context<Self>,
    ) {
        self.desktop_dark = desktop_is_dark(appearance);
        self.apply_appearance(cx);
        cx.notify();
    }

    pub(super) fn observe_appearance(&mut self, window: &Window, cx: &mut Context<Self>) {
        if self.appearance_observer.is_none() {
            self.desktop_dark = desktop_is_dark(window.appearance());
            let owner = cx.entity().downgrade();
            self.appearance_observer = Some(window.observe_window_appearance(move |window, cx| {
                let _ = owner.update(cx, |shell, cx| {
                    shell.desktop_appearance_changed(window.appearance(), cx);
                });
            }));
            self.apply_appearance(cx);
        }
    }

    pub(super) fn settings_appearance(&self, cx: &mut Context<Self>) -> Div {
        let mut choices = div().w_full().flex().gap_2();
        for preference in AppAppearance::ALL {
            choices = choices.child(div().flex_1().min_w_0().child(self.choice_button(
                Control::Appearance(preference),
                preference.label(),
                "",
                self.appearance.preference == preference,
                cx,
            )));
        }
        let status = if !self.appearance.loaded {
            "Reading appearance preference…".into()
        } else if self.appearance.saving {
            "Saving appearance…".into()
        } else if let Some(error) = self.appearance.error {
            error.into()
        } else if self.appearance.preference == AppAppearance::System {
            format!(
                "Follows your desktop · {}. Changes are applied automatically.",
                if self.desktop_dark { "Dark" } else { "Light" }
            )
        } else {
            "Applied immediately and remembered on this computer.".into()
        };
        controls::section(
            "Appearance",
            "Choose how Regressee looks. Website screenshots are unchanged.",
        )
        .debug_selector(|| "settings-appearance".into())
        .child(choices)
        .child(
            div()
                .debug_selector(|| "settings-appearance-status".into())
                .text_size(px(13.))
                .line_height(px(18.))
                .text_color(rgb(if self.appearance.error.is_some() {
                    palette().warning
                } else {
                    palette().muted
                }))
                .child(status),
        )
    }
}
