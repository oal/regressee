// SPDX-License-Identifier: MPL-2.0
//! One runtime explanation for setup and App settings. This grants no capability.
use super::*;
use regressee_app::product::{RuntimeReadiness, RuntimeState};

pub(super) fn runtime_copy(
    status: Option<&RuntimeReadiness>,
) -> (&'static str, Tone, &'static str) {
    let local =
        "You can still save projects on this computer and view saved screenshots and results.";
    match status.map(|s| s.status) {
        None => (
            "Checking browser setup",
            Tone::Neutral,
            "Checking availability; no download or website visit starts here. Local saving is available.",
        ),
        Some(RuntimeState::UnavailableDistribution) => (
            "Browser setup incomplete in this build",
            Tone::Warning,
            regressee_app::recovery::Problem::Browser(RuntimeState::UnavailableDistribution)
                .message(),
        ),
        Some(RuntimeState::UnsupportedPlatform) => (
            "Browser unavailable on this platform",
            Tone::Warning,
            regressee_app::recovery::Problem::Browser(RuntimeState::UnsupportedPlatform).message(),
        ),
        Some(RuntimeState::NotInstalled)
            if status.is_some_and(|s| {
                s.setup_available
                    && s.scope == Some(regressee_runtime::distribution::Scope::DevelopmentLoopback)
            }) =>
        {
            (
                "Test browser — local websites only",
                Tone::Warning,
                "This development build can only visit approved local test websites, not public websites. Internet is required to install its browser. You can still save projects on this computer.",
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
                "Test browser — local websites only",
                Tone::Warning,
                "This browser can only capture and log in to approved local test websites, not public websites. You can still save projects on this computer.",
            )
        }
        Some(RuntimeState::Ready) => (
            "Browser ready",
            Tone::Information,
            "Ready for login, finding pages and capturing screenshots. Saving settings does not open a website.",
        ),
        Some(RuntimeState::RepairRequired) => (
            "Browser needs repair",
            Tone::Warning,
            "Open App settings and choose Repair browser before logging in or capturing pages. You can still save projects and view saved results.",
        ),
        Some(RuntimeState::UpdateRequired) => (
            "Browser update required",
            Tone::Warning,
            "Open App settings and choose Update browser before login or capture. Local saving remains available; saved screenshots and baselines are kept.",
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
                "See App settings for details. You can still save projects; browser setup is not available until its status is known.",
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
            Reply::ActionRequired(problem) => self.runtime_status = Some(problem.message().into()),
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
