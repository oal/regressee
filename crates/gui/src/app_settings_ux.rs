// SPDX-License-Identifier: MPL-2.0
//! Task-first settings; optional tools disclose without granting new capabilities.
use super::*;
use regressee_app::product::RuntimeState;
use regressee_gui::ui::status_icon::{StatusIcon, status_icon};

pub(super) struct BrowserPresentation {
    pub(super) title: &'static str,
    tone: Tone,
    pub(super) explanation: &'static str,
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
                if matches!(tone, Tone::Warning) {
                    help
                } else {
                    ""
                },
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

    fn settings_browser(&self, cx: &mut Context<Self>) -> Div {
        let view = self.settings_browser_presentation();
        let attention = matches!(view.tone, Tone::Warning);
        let ink = if attention {
            palette().warning
        } else {
            palette().ink
        };
        let icon = if attention {
            StatusIcon::Warning
        } else {
            StatusIcon::Information
        };
        let enabled = self.control_enabled(Control::SetupRuntime, cx);
        let section = controls::section(
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
                .bg(rgb(if attention {
                    palette().warning_surface
                } else {
                    palette().surface
                }))
                .border_1()
                .border_color(rgb(if attention {
                    palette().warning
                } else {
                    palette().line
                }))
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
                            status_icon(
                                icon,
                                if attention {
                                    palette().warning
                                } else {
                                    palette().accent
                                },
                                20.,
                            )
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
                .when(!view.explanation.is_empty(), |status| {
                    status.child(
                        div()
                            .debug_selector(|| "settings-browser-explanation".into())
                            .text_size(px(14.))
                            .line_height(px(20.))
                            .text_color(rgb(if attention {
                                palette().warning
                            } else {
                                palette().muted
                            }))
                            .child(view.explanation),
                    )
                })
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
                                .text_color(rgb(if attention {
                                    palette().warning
                                } else {
                                    palette().muted
                                }))
                                .child(view.action_help),
                        ),
                ),
        );
        let mut details = div()
            .debug_selector(|| "settings-browser-details".into())
            .flex()
            .flex_col()
            .gap_2()
            .text_size(px(13.))
            .line_height(px(18.))
            .text_color(rgb(palette().muted));
        if let Some(status) = &self.runtime_status {
            details = details.child(status.clone());
        }
        section.child(self.inline_disclosure(Control::Details(Detail::Browser), "Browser details", "", self.details.contains(&Detail::Browser), details.child("Regressee uses its own verified browser components, not your system browser. Updating or repairing keeps saved screenshots and baselines. Checks still require the baseline’s matching browser components; capture a new baseline only if you choose."), cx))
    }

    fn settings_storage(&self, cx: &mut Context<Self>) -> Div {
        let mut section = controls::section(
            "On this computer",
            "Use History to review saved files and deletion impact.",
        );
        let Some(summary) = &self.summary else {
            return section.child(controls::activity("Reading local storage…"));
        };
        section = section
            .child(self.setting_line("Saved projects", &summary.projects.to_string()))
            .child(self.setting_line("Screenshots & data", &human_bytes(summary.artifact_bytes)));
        if summary.pending_cleanup > 0 {
            section=section.child(div().debug_selector(||"settings-pending-cleanup".into()).text_size(px(HELP_TEXT_SIZE)).line_height(px(HELP_LINE_HEIGHT)).text_color(rgb(palette().warning))
                .child(format!("{} {} awaiting removal after a confirmed deletion. Check folder permissions in your operating system; Regressee retries cleanup when it starts. Other saved projects remain available.",summary.pending_cleanup,if summary.pending_cleanup==1 {"file is"} else {"files are"})));
        }
        section.child(self.inline_disclosure(Control::Details(Detail::LocalStorage), "Storage locations", "", self.details.contains(&Detail::LocalStorage),
                div().debug_selector(|| "settings-storage-details".into()).flex().flex_col().gap_3()
                    .child(storage_path("Project data", summary.paths.data.display().to_string()))
                    .child(storage_path("App cache", summary.paths.cache.display().to_string()))
                    .child(settings_help(format!("{} stored files. Totals exclude the database and browser/login caches. Browser repair never deletes screenshot history.", summary.artifacts))),
            cx))
    }

    pub(super) fn network_loading_help(&self) -> Option<&'static str> {
        if self.network_status.is_some() {
            None
        } else if self.pending.is_some()
            && self
                .progress
                .as_ref()
                .is_some_and(|(label, _, _)| label == Work::NetworkStatus.activity_label())
        {
            Some("Reading saved network settings…")
        } else if self.pending.is_some() {
            Some(
                "Network settings have not loaded. Wait for the current operation to finish, then retry loading them.",
            )
        } else {
            Some("Network settings have not loaded. Choose Retry loading saved settings.")
        }
    }

    fn settings_network(&self, cx: &mut Context<Self>) -> Div {
        let mut fields = div().w_full().flex().flex_col().gap_3();
        if self.details.contains(&Detail::ManagedNetwork) {
            fields = fields
                .child(self.crawl_field(17, "Proxy server (optional)", "Enter a server address such as http://proxy.example:3128, without login details. Leave blank to use your computer's network settings."))
                .child(self.crawl_field(18, "Trusted certificate file (optional)", "Path to a PEM certificate file for downloads and website access. The browser must also trust this certificate through your computer's settings. Connection security checks stay enabled."))
                .child(div().flex().flex_wrap().gap_2()
                    .when(self.network_status.is_none() && self.pending.is_none(), |el| el.child(self.guided_button(42, "Retry loading saved settings", cx)))
                    .child(self.guided_button(41, "Save network settings", cx)))
                .when_some(self.network_loading_help(), |fields, message| fields.child(settings_help(message)));
            let limitations = div().flex().flex_col().gap_2()
                    .when_some(self.network_status.clone(), |el, status| el.child(settings_help(status)))
                    .child(div().debug_selector(|| "network-limitations-copy".into()).text_size(px(14.)).line_height(px(20.)).text_color(rgb(palette().warning)).child("The development browser cannot read a website's robots.txt rules through a proxy. Only override these rules if you have permission. VPN and company networks are not fully tested. Custom certificates must also be trusted in your computer's browser settings. Connection security checks are never disabled."));
            fields = fields.child(self.inline_disclosure(
                Control::Details(Detail::Network),
                "Network limitations",
                "",
                self.details.contains(&Detail::Network),
                limitations,
                cx,
            ));
        }
        optional_group()
            .debug_selector(|| "managed-network-section".into())
            .child(self.inline_disclosure(
                Control::Details(Detail::ManagedNetwork),
                "Network settings",
                "Optional · Only needed for a proxy server or custom trusted certificate.",
                self.details.contains(&Detail::ManagedNetwork),
                fields,
                cx,
            ))
            .when(
                self.details.contains(&Detail::ManagedNetwork) || !self.form_errors(cx).is_empty(),
                |section| {
                    section.child(
                        div()
                            .min_h(px(CONTROL_HEIGHT))
                            .child(self.form_error_summary(cx)),
                    )
                },
            )
    }

    pub(super) fn app_settings(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut support_body = div().w_full().flex().flex_col().gap_4();
        if self.details.contains(&Detail::SupportTools) {
            support_body = support_body
                .child(self.diagnostic_panel(cx))
                .child(div().flex().flex_col().gap_2()
                    .child(div().font_weight(FontWeight::MEDIUM).child("Check saved data"))
                    .child(settings_help("Looks for interrupted saves and damaged files. Review the results before choosing any cleanup; nothing is deleted by this check."))
                    .child(self.button(Control::Phase6(13), "Check saved data…", ActionStyle::Secondary, cx)));
        }
        let support = optional_group().debug_selector(|| "settings-support-tools".into())
            .child(self.inline_disclosure(Control::Details(Detail::SupportTools), "Troubleshooting tools", "Preview a support report or check saved data for interrupted saves and damaged files.", self.details.contains(&Detail::SupportTools), support_body, cx))
            // Requested results remain available even if the optional tools are
            // collapsed while the asynchronous saved-data check finishes.
            .when(self.repair_preview.is_some(), |group| group.child(div().pl(px(32.)).flex().flex_col().gap_3()
                .child(div().font_weight(FontWeight::MEDIUM).child("Saved-data check results"))
                .child(self.portability_confirmation(cx))));
        div()
            .debug_selector(|| "app-settings-body".into())
            .w(px(680.))
            .max_w_full()
            .flex()
            .flex_col()
            .gap_5()
            .child(
                self.heading("App settings", "")
                    .debug_selector(|| "app-settings-heading".into()),
            )
            .child(self.settings_browser(cx))
            .child(self.settings_appearance(cx))
            .child(self.settings_storage(cx))
            .child(self.settings_network(cx))
            .child(support)
            .child(
                div()
                    .border_t_1()
                    .border_color(rgb(palette().line))
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
        .border_color(rgb(palette().line))
        .pt_4()
        .flex()
        .flex_col()
        .gap_2()
}

fn settings_help(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(13.))
        .line_height(px(18.))
        .text_color(rgb(palette().muted))
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
