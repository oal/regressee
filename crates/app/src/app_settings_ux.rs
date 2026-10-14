// SPDX-License-Identifier: MPL-2.0
//! Task-first settings; optional tools disclose without granting new capabilities.
use super::*;
use regressee_app::{
    product::RuntimeState,
    ui::status_icon::{StatusIcon, status_icon},
};

pub(super) const FOLDER_PREVIEW_LABEL: &str = "Preview a folder path…";
pub(super) const FOLDER_PREVIEW_HELP: &str = "Opens a folder picker and shows the path you choose. It does not inspect files or move project data. Changing the data location is not available yet.";

pub(super) struct BrowserPresentation {
    pub(super) title: &'static str,
    tone: Tone,
    explanation: &'static str,
    pub(super) action: &'static str,
    pub(super) action_help: &'static str,
    pub(super) checking: bool,
    pub(super) primary: bool,
}

impl Shell {
    pub(super) fn settings_browser_presentation(&self) -> BrowserPresentation {
        let (label, tone, help) = self.runtime_presentation();
        let status = self.runtime_readiness.as_ref();
        let state = status.map(|status| status.status);
        let failed_read = state.is_none() && label == "Browser availability could not be read";
        let (title, explanation, action, action_help) = match state {
            None if failed_read => (
                label,
                "The browser status could not be verified. Local project saving and saved results remain available.",
                "Browser setup unavailable",
                "Setup stays disabled until the browser status is known. See Browser details for the reported cause.",
            ),
            None => (
                if self.runtime_status.as_deref() == Some("Checking installed browser integrity…")
                {
                    "Checking browser integrity…"
                } else {
                    "Checking browser setup…"
                },
                "Verifying the browser components before allowing setup, login or website capture. No download or website visit starts during this check.",
                "Checking browser…",
                "Setup is paused until this check finishes.",
            ),
            Some(RuntimeState::NotInstalled) => (
                label,
                help,
                if status.is_some_and(|status| status.setup_available) {
                    "Install browser now"
                } else {
                    "Browser setup unavailable"
                },
                if status.is_some_and(|status| status.setup_available) {
                    "Downloads and verifies Regressee’s private browser. Does not visit your website."
                } else {
                    "This build cannot install the browser. Local saving and saved results remain available."
                },
            ),
            Some(RuntimeState::Ready) => (
                label,
                help,
                "Repair browser…",
                "Only needed if the browser is not working. Saved screenshots and baselines are kept.",
            ),
            Some(RuntimeState::UpdateRequired) => (
                label,
                "The installed browser components do not match this app build. Update before logging in or capturing pages.",
                "Update browser…",
                "Saved screenshots and baselines are kept. Checks still need the baseline’s matching browser components.",
            ),
            Some(RuntimeState::RepairRequired) => (
                label,
                "The installed browser components could not be verified. Repair before logging in or capturing pages.",
                "Repair browser…",
                "Reinstalls the private browser. Saved screenshots and baselines are kept.",
            ),
            Some(RuntimeState::UnavailableDistribution | RuntimeState::UnsupportedPlatform) => (
                label,
                help,
                "Browser setup unavailable",
                "Use a supported Regressee build and platform for website operations.",
            ),
        };
        BrowserPresentation {
            title,
            tone,
            explanation,
            action,
            action_help,
            checking: state.is_none() && !failed_read,
            primary: matches!(
                state,
                Some(
                    RuntimeState::NotInstalled
                        | RuntimeState::UpdateRequired
                        | RuntimeState::RepairRequired
                )
            ),
        }
    }

    fn settings_disclosure(
        &self,
        detail: Detail,
        title: &'static str,
        cx: &mut Context<Self>,
    ) -> gpui::Stateful<Div> {
        let open = self.details.contains(&detail);
        self.action_control(Control::Details(detail), title, ActionStyle::Secondary, cx)
            .w_full()
            .justify_between()
            .gap_3()
            .child(
                canvas(
                    |_, _, _| (),
                    move |bounds, _, window, _| {
                        let at = |x: f32, y: f32| {
                            bounds.origin
                                + gpui::point(bounds.size.width * x, bounds.size.height * y)
                        };
                        let mut path = gpui::PathBuilder::stroke(px(1.8));
                        if open {
                            path.move_to(at(0.2, 0.65));
                            path.line_to(at(0.5, 0.35));
                            path.line_to(at(0.8, 0.65));
                        } else {
                            path.move_to(at(0.2, 0.35));
                            path.line_to(at(0.5, 0.65));
                            path.line_to(at(0.8, 0.35));
                        }
                        if let Ok(path) = path.build() {
                            window.paint_path(path, rgb(INK));
                        }
                    },
                )
                .size(px(16.))
                .flex_shrink_0(),
            )
    }

    fn settings_browser(&self, cx: &mut Context<Self>) -> Div {
        let view = self.settings_browser_presentation();
        let attention = matches!(view.tone, Tone::Warning);
        let ink = if attention { WARNING } else { INK };
        let icon = if attention {
            StatusIcon::Warning
        } else {
            StatusIcon::Information
        };
        let enabled = self.control_enabled(Control::SetupRuntime, cx);
        let mut section = controls::section(
            "Private browser",
            "Used for website login and screenshots, separate from your system browser.",
        )
        .child(
            div()
                .debug_selector(|| "settings-browser-status".into())
                .w_full()
                .flex_shrink_0()
                .p_4()
                .rounded(px(4.))
                .bg(rgb(if attention { WARNING_SURFACE } else { SURFACE }))
                .border_1()
                .border_color(rgb(if attention { WARNING } else { LINE }))
                .flex()
                .flex_col()
                .gap_3()
                .child(
                    div()
                        .flex()
                        .items_start()
                        .gap_2()
                        .child(div().flex_shrink_0().pt(px(3.)).child(if view.checking {
                            controls::activity_indicator().into_any_element()
                        } else {
                            status_icon(icon, if attention { WARNING } else { ACCENT }, 20.)
                                .into_any_element()
                        }))
                        .child(
                            div()
                                .debug_selector(|| "settings-browser-state".into())
                                .min_w_0()
                                .flex_1()
                                .text_size(px(18.))
                                .line_height(px(24.))
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_color(rgb(ink))
                                .child(view.title),
                        ),
                )
                .child(
                    div()
                        .debug_selector(|| "settings-browser-explanation".into())
                        .text_size(px(14.))
                        .line_height(px(20.))
                        .text_color(rgb(if attention { WARNING } else { MUTED }))
                        .child(view.explanation),
                )
                .child(
                    div()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(self.button(
                            Control::SetupRuntime,
                            view.action,
                            if enabled && view.primary {
                                ActionStyle::Primary
                            } else {
                                ActionStyle::Secondary
                            },
                            cx,
                        ))
                        .child(
                            div()
                                .debug_selector(|| "settings-browser-action-help".into())
                                .text_size(px(13.))
                                .line_height(px(18.))
                                .text_color(rgb(if attention { WARNING } else { MUTED }))
                                .child(view.action_help),
                        ),
                ),
        )
        .child(self.settings_disclosure(Detail::Browser, "Browser details", cx));
        if self.details.contains(&Detail::Browser) {
            let mut details = div()
                .debug_selector(|| "settings-browser-details".into())
                .flex()
                .flex_col()
                .gap_2()
                .text_size(px(13.))
                .line_height(px(18.))
                .text_color(rgb(MUTED));
            if let Some(status) = &self.runtime_status {
                details = details.child(status.clone());
            }
            section = section.child(details.child("Regressee uses its own verified browser components, not your system browser. Updating or repairing keeps saved screenshots and baselines. Checks still require the baseline’s matching browser components; capture a new baseline only if you choose."));
        }
        section
    }

    fn settings_storage(&self, cx: &mut Context<Self>) -> Div {
        let mut section = controls::section(
            "On this computer",
            "Projects and screenshots stay local. Use History to review saved files and deletion impact.",
        );
        let Some(summary) = &self.summary else {
            return section.child(controls::activity("Reading local storage…"));
        };
        section = section
            .child(self.setting_line("Saved projects", &summary.projects.to_string()))
            .child(self.setting_line("Screenshots & data", &human_bytes(summary.artifact_bytes)))
            .child(self.settings_disclosure(
                Detail::LocalStorage,
                "Storage locations & folder preview",
                cx,
            ));
        if self.details.contains(&Detail::LocalStorage) {
            section = section.child(
                div().debug_selector(|| "settings-storage-details".into()).flex().flex_col().gap_3()
                    .child(storage_path("Project data", summary.paths.data.display().to_string()))
                    .child(storage_path("App cache", summary.paths.cache.display().to_string()))
                    .child(settings_help(format!("{} stored files. Totals exclude the database and browser/login caches. Browser repair never deletes screenshot history.", summary.artifacts)))
                    .child(div().border_t_1().border_color(rgb(LINE)).pt_3().flex().flex_col().gap_2()
                        .child(div().font_weight(FontWeight::MEDIUM).child("Folder picker preview"))
                        .child(settings_help(FOLDER_PREVIEW_HELP))
                        .child(self.button(Control::ChooseFolder, FOLDER_PREVIEW_LABEL, ActionStyle::Secondary, cx))),
            );
        }
        section
    }

    fn settings_network(&self, cx: &mut Context<Self>) -> Div {
        let mut section = optional_group()
            .debug_selector(|| "managed-network-section".into())
            .child(self.settings_disclosure(Detail::ManagedNetwork, "Network settings", cx))
            .child(settings_help(
                "Optional · Only needed for a proxy server or custom trusted certificate.",
            ));
        if !self.form_errors(cx).is_empty() {
            section = section.child(self.form_error_summary(cx));
        }
        if self.details.contains(&Detail::ManagedNetwork) {
            section = section
                .child(self.crawl_field(17, "Proxy server (optional)", "Enter a server address such as http://proxy.example:3128, without login details. Leave blank to use your computer's network settings."))
                .child(self.crawl_field(18, "Trusted certificate file (optional)", "Path to a PEM certificate file for downloads and website access. The browser must also trust this certificate through your computer's settings. Connection security checks stay enabled."))
                .child(div().flex().flex_wrap().gap_2()
                    .when(self.network_status.is_none() && self.pending.is_none(), |el| el.child(self.guided_button(42, "Retry loading saved settings", cx)))
                    .child(self.guided_button(41, "Save network settings", cx)))
                .child(settings_help(if self.network_status.is_some() { "Network settings loaded. Choose Save to apply any changes." } else { "Reading saved network settings…" }))
                .child(self.button(Control::Details(Detail::Network), if self.details.contains(&Detail::Network) { "Hide network limitations" } else { "Network limitations" }, ActionStyle::Secondary, cx));
            if self.details.contains(&Detail::Network) {
                section = section
                    .when_some(self.network_status.clone(), |el, status| el.child(settings_help(status)))
                    .child(div().debug_selector(|| "network-limitations-copy".into()).text_size(px(14.)).line_height(px(20.)).text_color(rgb(WARNING)).child("The development browser cannot read a website's robots.txt rules through a proxy. Only override these rules if you have permission. VPN and company networks are not fully tested. Custom certificates must also be trusted in your computer's browser settings. Connection security checks are never disabled."));
            }
        }
        section
    }

    pub(super) fn app_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut support = optional_group()
            .debug_selector(|| "settings-support-tools".into())
            .child(self.settings_disclosure(Detail::SupportTools, "Troubleshooting tools", cx))
            .child(settings_help("Preview a support report or check saved data for interrupted saves and damaged files."));
        if self.details.contains(&Detail::SupportTools) {
            support = support
                .child(self.diagnostic_panel(cx))
                .child(div().flex().flex_col().gap_2()
                    .child(div().font_weight(FontWeight::MEDIUM).child("Check saved data"))
                    .child(settings_help("Looks for interrupted saves and damaged files. Review the results before choosing any cleanup; nothing is deleted by this check."))
                    .child(self.button(Control::Phase6(13), "Check saved data…", ActionStyle::Secondary, cx)));
        }
        div()
            .debug_selector(|| "app-settings-body".into())
            .w(px(680.))
            .max_w_full()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                self.heading(
                    "App settings",
                    "Browser status, local storage and optional tools.",
                )
                .debug_selector(|| "app-settings-heading".into()),
            )
            .child(self.settings_browser(cx))
            .child(self.settings_storage(cx))
            .child(self.settings_network(cx))
            .child(support)
            .child(self.portability_confirmation(cx))
            .child(
                div()
                    .border_t_1()
                    .border_color(rgb(LINE))
                    .pt_4()
                    .flex()
                    .flex_wrap()
                    .gap_3()
                    .child(self.button(
                        Control::About,
                        "About Regressee",
                        ActionStyle::Secondary,
                        cx,
                    ))
                    .child(self.button(
                        Control::Shortcuts,
                        "Keyboard shortcuts · F1",
                        ActionStyle::Secondary,
                        cx,
                    )),
            )
            .into_any_element()
    }
}

fn optional_group() -> Div {
    div()
        .w_full()
        .flex_shrink_0()
        .border_t_1()
        .border_color(rgb(LINE))
        .pt_4()
        .flex()
        .flex_col()
        .gap_2()
}

fn settings_help(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(13.))
        .line_height(px(18.))
        .text_color(rgb(MUTED))
        .child(text.into())
}

fn storage_path(label: &'static str, path: String) -> Div {
    div()
        .w_full()
        .min_w_0()
        .flex()
        .flex_col()
        .gap_1()
        .child(div().font_weight(FontWeight::MEDIUM).child(label))
        .child(settings_help(path))
}
