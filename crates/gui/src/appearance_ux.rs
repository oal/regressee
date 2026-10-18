// SPDX-License-Identifier: MPL-2.0
//! Live desktop appearance and the explicit, machine-local settings override.
use super::*;

impl Shell {
    fn apply_appearance(&self, cx: &mut Context<Self>) {
        let reduced = self.forced_reduced_motion || self.appearance.reduced_motion;
        let motion_changed =
            cx.try_global::<controls::ReducedMotion>().map(|m| m.0) != Some(reduced);
        if motion_changed {
            cx.set_global(controls::ReducedMotion(reduced));
        }
        if set_dark(self.appearance.preference.is_dark(self.desktop_dark)) || motion_changed {
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
            "Reading app preferences…".into()
        } else if self.appearance.saving {
            "Saving app preferences…".into()
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

    pub(super) fn settings_motion(&self, cx: &mut Context<Self>) -> Div {
        let effective = self.forced_reduced_motion || self.appearance.reduced_motion;
        let mut choices = div().w_full().flex().gap_2();
        for (reduced, label) in [(false, "Normal"), (true, "Reduced")] {
            choices = choices.child(div().flex_1().min_w_0().child(self.choice_button(
                Control::ReducedMotion(reduced),
                label,
                "",
                effective == reduced,
                cx,
            )));
        }
        let status = if self.forced_reduced_motion {
            "Reduced motion is forced by REGRESSEE_REDUCED_MOTION=1. Remove that environment setting and restart to use your remembered choice."
        } else if !self.appearance.loaded {
            "Reading app preferences…"
        } else if self.appearance.saving {
            "Saving app preferences…"
        } else {
            self.appearance
                .error
                .unwrap_or("Applied immediately and remembered on this computer.")
        };
        controls::section("Reduce motion", "Reduce movement and keep activity indicators still. Progress and status updates remain visible.")
            .debug_selector(|| "settings-motion".into())
            .child(choices)
            .child(div()
                .debug_selector(|| "settings-motion-status".into())
                .text_size(px(13.)).line_height(px(18.))
                .text_color(rgb(if self.appearance.error.is_some() && !self.forced_reduced_motion { palette().warning } else { palette().muted }))
                .child(status))
    }
}
