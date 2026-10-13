// SPDX-License-Identifier: MPL-2.0
//! One runtime explanation for setup and App settings. This grants no capability.
use super::*;
use regressee_app::product::{RuntimeReadiness, RuntimeState};

pub(super) fn runtime_copy(
    status: Option<&RuntimeReadiness>,
) -> (&'static str, Tone, &'static str) {
    let local = "You can still save projects locally and inspect retained evidence.";
    match status.map(|s| s.status) {
        None => (
            "Checking browser setup",
            Tone::Neutral,
            "Checking availability; no download or website visit starts here. Local saving is available.",
        ),
        Some(RuntimeState::UnavailableDistribution) => (
            "Browser setup incomplete in this build",
            Tone::Warning,
            local,
        ),
        Some(RuntimeState::UnsupportedPlatform) => {
            ("Browser unavailable on this platform", Tone::Warning, local)
        }
        Some(RuntimeState::NotInstalled)
            if status.is_some_and(|s| {
                s.setup_available
                    && s.scope == Some(regressee_runtime::distribution::Scope::DevelopmentLoopback)
            }) =>
        {
            (
                "Development browser — local fixtures only",
                Tone::Warning,
                "This development build installs a browser for authorized local fixtures only. Internet is required; local saving is available.",
            )
        }
        Some(RuntimeState::NotInstalled) if status.is_some_and(|s| s.setup_available) => (
            "Browser will install on first use",
            Tone::Information,
            "This supported build installs its private browser when needed. Internet is required; no developer tools are needed.",
        ),
        Some(RuntimeState::NotInstalled) => {
            ("Browser installation unavailable", Tone::Warning, local)
        }
        Some(RuntimeState::Ready)
            if status.is_some_and(|s| {
                s.scope == Some(regressee_runtime::distribution::Scope::DevelopmentLoopback)
            }) =>
        {
            (
                "Development browser — local fixtures only",
                Tone::Warning,
                "Website capture and login are restricted to authorized local fixtures. Local saving is available.",
            )
        }
        Some(RuntimeState::Ready) => (
            "Private browser verified",
            Tone::Information,
            "Login and website visits start only with your explicit action.",
        ),
        Some(RuntimeState::RepairRequired) => (
            "Browser needs explicit repair",
            Tone::Warning,
            "Use App settings to repair the browser before login or capture. Local saving and retained evidence remain available.",
        ),
    }
}

impl Shell {
    pub(super) fn runtime_presentation(&self) -> (&'static str, Tone, &'static str) {
        if self.runtime_readiness.is_none()
            && self
                .runtime_status
                .as_ref()
                .is_some_and(|s| s.starts_with("Browser availability could not be read"))
        {
            (
                "Browser availability could not be read",
                Tone::Warning,
                "See App settings for details. Local saving remains available; no installation is promised.",
            )
        } else {
            runtime_copy(self.runtime_readiness.as_ref())
        }
    }
    pub(super) fn observe_setup_runtime(&mut self, cx: &mut Context<Self>) {
        if self.runtime_status.is_some() || self.runtime_pending.is_some() {
            return;
        }
        self.next_request += 1;
        let id = self.next_request;
        match self.jobs.submit(Request {
            id,
            work: Work::RuntimeStatus,
        }) {
            Ok(()) => {
                self.runtime_pending = Some(id);
                self.runtime_status = Some("Checking browser availability…".into());
            }
            Err(error) => {
                self.runtime_status =
                    Some(format!("Browser availability could not be read: {error}"))
            }
        }
        cx.notify();
    }
    pub(super) fn apply_runtime_observation(
        &mut self,
        update: &Update,
        cx: &mut Context<Self>,
    ) -> bool {
        if self.runtime_pending != Some(update.id) {
            return false;
        }
        if matches!(update.reply, Reply::Progress { .. }) {
            return true;
        }
        self.runtime_pending = None;
        match &update.reply {
            Reply::RuntimeReadiness(status) => {
                self.runtime_readiness = Some(status.clone());
                self.runtime_status = Some(status.message.into());
            },
            Reply::Runtime(status) => self.runtime_status = Some(status.clone()),
            Reply::ProductFailure(message) => self.runtime_status = Some(format!("Browser availability could not be read: {message}")),
            _ => self.runtime_status = Some("Browser availability could not be read. See App settings; local saving remains available.".into()),
        }
        // No save continuation, environment hydration, selection or focus changes.
        cx.notify();
        true
    }
    pub(super) fn setup_runtime_note(&self) -> AnyElement {
        let (label, tone, help) = self.runtime_presentation();
        div()
            .debug_selector(|| "setup-runtime-availability".into())
            .flex_shrink_0()
            .min_h(px(60.))
            .flex()
            .flex_col()
            .gap_1()
            .text_size(px(13.))
            .text_color(rgb(if matches!(tone, Tone::Warning) {
                WARNING
            } else {
                MUTED
            }))
            .child(div().font_weight(FontWeight::MEDIUM).child(label))
            .child(help)
            .into_any_element()
    }
}
