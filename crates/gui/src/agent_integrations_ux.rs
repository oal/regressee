// SPDX-License-Identifier: MPL-2.0
//! A reversible local setup task, not an agent permission or connection wizard.
use super::*;
use regressee_app::agent_integrations::{Host, Installer, State, Status};

#[derive(Default)]
pub(super) struct IntegrationUi {
    pub statuses: Vec<Status>,
    pending: Option<Receiver<IntegrationResult>>,
    action: Option<Control>,
    target: Option<usize>,
    focus_after: Option<Control>,
    reveal_feedback: Option<Control>,
    feedback_bounds: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
    pub feedback: Option<String>,
    pub error: Option<String>,
    #[cfg(test)]
    pub home: Option<std::path::PathBuf>,
}
struct IntegrationResult {
    statuses: Vec<Status>,
    feedback: Option<String>,
    error: Option<String>,
}

impl Shell {
    pub(super) fn integration_mounted(&self, control: Control) -> bool {
        if !self.details.contains(&Detail::AgentIntegrations) {
            return false;
        }
        match control {
            Control::AgentRefresh => true,
            Control::AgentInstall(index) => {
                self.integrations.statuses.get(index).is_some_and(|s| {
                    matches!(s.state, State::NotInstalled | State::UpdateAvailable)
                })
            }
            Control::AgentRemove(index) => self
                .integrations
                .statuses
                .get(index)
                .is_some_and(|s| s.removable),
            Control::AgentRetry(index) => self
                .integrations
                .statuses
                .get(index)
                .is_some_and(|s| matches!(s.state, State::Conflict | State::Unavailable)),
            _ => false,
        }
    }
    pub(super) fn integration_enabled(&self, control: Control) -> bool {
        self.integrations.pending.is_none()
            && self.summary.is_some()
            && self.integration_mounted(control)
    }
    pub(super) fn start_integrations(&mut self, control: Control, cx: &mut Context<Self>) {
        if self.integrations.pending.is_some() {
            return;
        }
        let Some(summary) = &self.summary else {
            self.integrations.error = Some("Local storage locations are still loading. Choose Check again when loading finishes.".into());
            cx.notify();
            return;
        };
        let paths = summary.paths.clone();
        #[cfg(test)]
        let home = self.integrations.home.clone();
        let (send, recv) = std::sync::mpsc::channel();
        self.integrations.pending = Some(recv);
        self.integrations.action = Some(control);
        self.integrations.target = match control {
            Control::AgentInstall(index)
            | Control::AgentRemove(index)
            | Control::AgentRetry(index) => Some(index),
            _ => None,
        };
        self.integrations.focus_after = None;
        self.integrations.feedback_bounds.set(None);
        self.integrations.feedback = None;
        self.integrations.error = None;
        std::thread::spawn(move || {
            let result = (|| -> anyhow::Result<IntegrationResult> {
                let exe = std::env::current_exe()?;
                let cli = exe.with_file_name(if cfg!(windows) { "regressee.exe" } else { "regressee" });
                #[cfg(test)]
                let installer = if let Some(home) = home { Installer::new(home, &paths, exe)? } else { Installer::for_user(&paths, cli)? };
                #[cfg(not(test))]
                let installer = Installer::for_user(&paths, cli)?;
                let (feedback, error) = match control {
                    Control::AgentInstall(index) => {
                        let host = *Host::ALL.get(index).ok_or_else(|| anyhow::anyhow!("Unknown agent host"))?;
                        match installer.install(host) {
                            Ok(status) => (Some(format!("Skill installed. {}", status.message)), None),
                            Err(e) => (None, Some(format!("Could not install the {} skill: {e:#}", host.name()))),
                        }
                    }
                    Control::AgentRemove(index) => {
                        let host = *Host::ALL.get(index).ok_or_else(|| anyhow::anyhow!("Unknown agent host"))?;
                        match installer.remove(host) {
                            Ok(status) => (Some(format!("{} skill removed. {} Projects, screenshots and host permissions were not changed. Start a new agent session to stop using instructions already loaded.", host.name(), if status.message.starts_with("Skill removed") { status.message.as_str() } else { "" })), None),
                            Err(e) => (None, Some(format!("Could not remove the {} skill: {e:#}", host.name()))),
                        }
                    }
                    _ => (None, None),
                };
                Ok(IntegrationResult { statuses: installer.statuses(), feedback, error })
            })().unwrap_or_else(|e| IntegrationResult { statuses: Vec::new(), feedback: None, error: Some(format!("Agent setup is unavailable: {e:#}")) });
            let _ = send.send(result);
        });
        cx.notify();
    }
    pub(super) fn poll_integrations(&mut self, cx: &mut Context<Self>) {
        let Some(recv) = &self.integrations.pending else {
            return;
        };
        let result = match recv.try_recv() {
            Ok(result) => result,
            Err(std::sync::mpsc::TryRecvError::Empty) => return,
            Err(_) => IntegrationResult { statuses: Vec::new(), feedback: None, error: Some("Setup stopped unexpectedly. Choose Check again to inspect the actual files before retrying.".into()) },
        };
        self.complete_integrations(result, cx);
    }
    fn complete_integrations(&mut self, result: IntegrationResult, cx: &mut Context<Self>) {
        self.integrations.pending = None;
        self.integrations.focus_after = self.integrations.action.take();
        self.integrations.feedback_bounds.set(None);
        self.integrations.statuses = result.statuses;
        self.integrations.feedback = result.feedback;
        self.integrations.error = result.error;
        cx.notify();
    }
    pub(super) fn restore_integration_focus(
        &mut self,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.reveal_pending_integration_feedback(window, cx);
        let Some(previous) = self.integrations.focus_after.take() else {
            return;
        };
        // Completion cannot take focus from another control, screen or dialog.
        if self.focus.screen != Screen::Settings
            || self.focus.dialog
            || self.native_prompt
            || !self.details.contains(&Detail::AgentIntegrations)
            || !self.handle(previous, cx).is_focused(window)
        {
            return;
        }
        let target = if self.integration_mounted(previous) {
            previous
        } else {
            let Some(index) = self.integrations.target else {
                return;
            };
            [
                Control::AgentInstall(index),
                Control::AgentRemove(index),
                Control::AgentRetry(index),
            ]
            .into_iter()
            .find(|&c| self.integration_mounted(c))
            .unwrap_or(Control::AgentRefresh)
        };
        if self.control_enabled(target, cx) {
            window.focus(&self.handle(target, cx));
            self.reveal_form_control(target, true, window, cx);
            self.reveal_integration_feedback(target, window, cx);
        }
    }
    pub(super) fn reveal_integration_feedback(
        &mut self,
        control: Control,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.integrations.reveal_feedback = Some(control);
        self.reveal_pending_integration_feedback(window, cx);
    }
    fn reveal_pending_integration_feedback(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.integrations.pending.is_none()
            && self.integrations.feedback.is_none()
            && self.integrations.error.is_none()
        {
            self.integrations.reveal_feedback = None;
            return;
        }
        if let Some(control) = self.integrations.reveal_feedback {
            if self.focus.screen != Screen::Settings
                || self.focus.dialog
                || self.native_prompt
                || !self.details.contains(&Detail::AgentIntegrations)
                || !self.handle(control, cx).is_focused(window)
            {
                self.integrations.reveal_feedback = None;
                return;
            }
            if let Some(bounds) = self.integrations.feedback_bounds.get() {
                let clip = self.screen_scroll.bounds();
                let delta = if bounds.bottom() > clip.bottom() - px(8.) {
                    clip.bottom() - px(8.) - bounds.bottom()
                } else if bounds.top() < clip.top() + px(8.) {
                    clip.top() + px(8.) - bounds.top()
                } else {
                    px(0.)
                };
                let offset = self.screen_scroll.offset();
                self.screen_scroll.set_offset(gpui::point(
                    offset.x,
                    (offset.y + delta).clamp(-self.screen_scroll.max_offset().height, px(0.)),
                ));
                self.integrations.reveal_feedback = None;
            }
            // A newly mounted message is measured by this frame; finish the
            // bounded reveal on the next render, not a timer or focus side effect.
            cx.notify();
        }
    }
    fn integration_feedback(&self) -> Div {
        let bounds = self.integrations.feedback_bounds.clone();
        let mut feedback = div()
            .debug_selector(|| "agent-setup-feedback".into())
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_2();
        if self.integrations.pending.is_some() {
            feedback = feedback.child(controls::activity(match self.integrations.action {
                Some(Control::AgentInstall(_)) => "Installing skill…",
                Some(Control::AgentRemove(_)) => "Removing skill…",
                _ => "Checking installed skills…",
            }));
        }
        if let Some(message) = &self.integrations.feedback {
            feedback = feedback.child(help(message.clone()));
        }
        if let Some(message) = &self.integrations.error {
            feedback = feedback.child(
                help(message.clone())
                    .text_color(rgb(palette().error))
                    .debug_selector(|| "agent-setup-error".into()),
            );
        }
        feedback.relative().child(
            canvas(
                |_, _, _| (),
                move |painted, _, _, _| bounds.set(Some(painted)),
            )
            .absolute()
            .inset_0()
            .size_full(),
        )
    }
    pub(super) fn settings_integrations(&self, cx: &mut Context<Self>) -> Div {
        let mut body = div().w_full().min_w_0().flex().flex_col().gap_3();
        if self.details.contains(&Detail::AgentIntegrations) {
            body = body.child(help("Install Regressee instructions for your user account, across local projects. This does not install the agent itself or change its permissions."));
            let feedback = self.integrations.pending.is_some()
                || self.integrations.feedback.is_some()
                || self.integrations.error.is_some();
            if feedback && self.integrations.target.is_none() {
                body = body.child(self.integration_feedback());
            }
            for (index, host) in Host::ALL.into_iter().enumerate() {
                let status = self
                    .integrations
                    .statuses
                    .get(index)
                    .filter(|s| s.host == host);
                let (label, ink) = match status.map(|s| s.state) {
                    Some(State::Installed) => ("Installed", palette().ink),
                    Some(State::UpdateAvailable) => ("Update available", palette().warning),
                    Some(State::Conflict) => ("Existing files need attention", palette().warning),
                    Some(State::Unavailable) => ("CLI unavailable", palette().warning),
                    Some(State::NotInstalled) => ("Not installed", palette().muted),
                    None => ("Not checked", palette().muted),
                };
                let mut row = div()
                    .debug_selector(move || format!("agent-integration-{index}"))
                    .w_full()
                    .min_w_0()
                    .flex_shrink_0()
                    .border_t_1()
                    .border_color(rgb(palette().line))
                    .pt_3()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .gap_2()
                            .child(div().font_weight(FontWeight::SEMIBOLD).child(host.name()))
                            .child(div().text_size(px(13.)).text_color(rgb(ink)).child(label)),
                    );
                if let Some(status) = status {
                    if matches!(status.state, State::Conflict | State::Unavailable)
                        && !(self.integrations.target == Some(index)
                            && self.integrations.error.is_some())
                    {
                        row = row
                            .child(help(status.message.clone()).text_color(rgb(palette().warning)));
                    }
                    row = row.child(help(status.path.display().to_string()));
                    let mut actions = div().flex().flex_wrap().gap_2();
                    if self.integration_mounted(Control::AgentInstall(index)) {
                        actions = actions.child(self.button(
                            Control::AgentInstall(index),
                            if status.state == State::UpdateAvailable {
                                "Update skill"
                            } else {
                                "Install skill"
                            },
                            ActionStyle::Secondary,
                            cx,
                        ));
                    }
                    if self.integration_mounted(Control::AgentRemove(index)) {
                        actions = actions.child(self.button(
                            Control::AgentRemove(index),
                            "Remove skill",
                            ActionStyle::Secondary,
                            cx,
                        ));
                    }
                    if self.integration_mounted(Control::AgentRetry(index)) {
                        actions = actions.child(self.button(
                            Control::AgentRetry(index),
                            "Check again",
                            ActionStyle::Secondary,
                            cx,
                        ));
                    }
                    row = row.child(actions);
                    if status.state == State::Installed {
                        row = row.child(help(host.next_step()));
                    }
                }
                if feedback && self.integrations.target == Some(index) {
                    row = row.child(self.integration_feedback());
                }
                body = body.child(row);
            }
            body = body.child(self.button(Control::AgentRefresh, "Check again", ActionStyle::Secondary, cx))
                .child(help("Installed means the skill files are verified, not that the agent has loaded them. Existing files and local edits are never overwritten."))
                .child(help("Codex uses the shared .agents skills folder, which other compatible agents may also read."))
                .child(div().border_t_1().border_color(rgb(palette().line)).pt_3().flex().flex_col().gap_2()
                    .child(div().font_weight(FontWeight::MEDIUM).child("Watch an agent work"))
                    .child(help("Local agents use this app’s project data. Saved changes and activity update here automatically; your selected results and unsaved edits stay in place."))
                    .child(help("Ask the agent to start a named session and share its selection. Choose Follow agent in the top banner to follow its view, or Take control at any time. Setup never turns following on."))
                    .child(help("Cloud agent sessions do not automatically connect to this desktop.")));
        }
        div()
            .debug_selector(|| "settings-agent-integrations".into())
            .w_full()
            .flex_shrink_0()
            .border_t_1()
            .border_color(rgb(palette().line))
            .pt_4()
            .child(self.inline_disclosure(
                Control::Details(Detail::AgentIntegrations),
                "Agent integrations",
                "Claude Code, Codex and GitHub Copilot · Local, user-account setup",
                self.details.contains(&Detail::AgentIntegrations),
                body,
                cx,
            ))
    }
}
fn help(text: impl Into<SharedString>) -> Div {
    div()
        .w_full()
        .min_w_0()
        .text_size(px(13.))
        .line_height(px(18.))
        .text_color(rgb(palette().muted))
        .child(text.into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::workflow_refinement_tests::finish_jobs;

    fn finish_setup(s: &mut Shell, cx: &mut Context<Shell>) {
        let result = s
            .integrations
            .pending
            .as_ref()
            .unwrap()
            .recv_timeout(Duration::from_secs(10))
            .unwrap();
        s.complete_integrations(result, cx);
    }

    #[gpui::test]
    fn personal_skill_setup_roundtrips_without_engine_work_or_automatic_follow(
        cx: &mut gpui::TestAppContext,
    ) {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(&root.path().join("saved data"));
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_jobs(s, cx);
                s.focus.screen = Screen::Settings;
                s.refresh_visible(cx);
                finish_jobs(s, cx);
                s.integrations.home = Some(root.path().into());
                let request = s.next_request;
                assert!(!s.control_mounted(Control::AgentInstall(0)));
                s.activate_control(Control::Details(Detail::AgentIntegrations), window, cx);
                assert!(s.integrations.pending.is_some());
                finish_setup(s, cx);
                assert_eq!(s.integrations.statuses.len(), 3);
                for index in 0..3 {
                    assert!(s.control_enabled(Control::AgentInstall(index), cx));
                    s.activate_control(Control::AgentInstall(index), window, cx);
                    assert!(!s.control_enabled(Control::AgentRefresh, cx));
                    assert!(!s.control_enabled(Control::AgentInstall(index), cx));
                    assert!(
                        s.handle(Control::AgentInstall(index), cx)
                            .is_focused(window)
                    );
                    finish_setup(s, cx);
                    assert_eq!(s.integrations.statuses[index].state, State::Installed);
                    assert!(!s.control_mounted(Control::AgentInstall(index)));
                    assert!(s.control_enabled(Control::AgentRemove(index), cx));
                    assert!(
                        s.integrations
                            .feedback
                            .as_ref()
                            .unwrap()
                            .contains("Skill installed")
                    );
                    s.activate_control(Control::AgentRemove(index), window, cx);
                    finish_setup(s, cx);
                    assert_eq!(s.integrations.statuses[index].state, State::NotInstalled);
                    assert!(!s.control_mounted(Control::AgentRemove(index)));
                }
                assert_eq!(
                    s.next_request, request,
                    "Local setup must not submit product work"
                );
                assert!(s.agent.control.is_none());
                assert!(!s.native_prompt && !s.focus.dialog);
                s.activate_control(Control::Details(Detail::AgentIntegrations), window, cx);
                assert!(!s.control_mounted(Control::AgentRefresh));
                assert!(!s.control_enabled(Control::AgentInstall(0), cx));
            })
        });
        owned.join_after_ui();
    }

    #[gpui::test]
    fn conflicts_disable_destructive_actions_and_completion_never_reopens_settings(
        cx: &mut gpui::TestAppContext,
    ) {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(&root.path().join("data"));
        let (jobs, events) = Jobs::start_without_runtime(paths);
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_jobs(s, cx);
                s.focus.screen = Screen::Settings;
                s.refresh_visible(cx);
                finish_jobs(s, cx);
                s.integrations.home = Some(root.path().into());
                let path = root.path().join(".claude/skills/regressee");
                std::fs::create_dir_all(&path).unwrap();
                std::fs::write(path.join("SKILL.md"), "My own instructions").unwrap();
                s.activate_control(Control::Details(Detail::AgentIntegrations), window, cx);
                finish_setup(s, cx);
                assert_eq!(s.integrations.statuses[0].state, State::Conflict);
                assert!(!s.control_enabled(Control::AgentInstall(0), cx));
                assert!(!s.control_enabled(Control::AgentRemove(0), cx));
                s.activate_control(Control::AgentRefresh, window, cx);
                s.focus.screen = Screen::Projects;
                s.details.remove(&Detail::AgentIntegrations);
                s.presentation_focus = None;
                finish_setup(s, cx);
                assert_eq!(s.focus.screen, Screen::Projects);
                assert!(s.presentation_focus.is_none());
                assert_eq!(
                    std::fs::read_to_string(path.join("SKILL.md")).unwrap(),
                    "My own instructions"
                );
            })
        });
        owned.join_after_ui();
    }

    #[gpui::test]
    fn lower_row_setup_keeps_feedback_and_focus_visible_through_real_transitions(
        cx: &mut gpui::TestAppContext,
    ) {
        use crate::phase4_hierarchy_tests::draw;
        for (width, height) in [(1280., 800.), (820., 600.)] {
            let root = tempfile::tempdir().unwrap();
            let paths = AppPaths::isolated(&root.path().join("data"));
            let (jobs, events) = Jobs::start_without_runtime(paths);
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    finish_jobs(s, cx);
                    s.focus.screen = Screen::Settings;
                    s.refresh_visible(cx);
                    finish_jobs(s, cx);
                    s.integrations.home = Some(root.path().into());
                    s.activate_control(Control::Details(Detail::AgentIntegrations), window, cx);
                    finish_setup(s, cx);
                })
            });
            draw(cx);
            for (action, next) in [
                (Control::AgentInstall(2), Control::AgentRemove(2)),
                (Control::AgentRemove(2), Control::AgentInstall(2)),
            ] {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        s.activate_control(action, window, cx);
                        assert!(s.handle(action, cx).is_focused(window));
                    })
                });
                draw(cx);
                crate::phase1_setup_tests::inside(
                    cx.debug_bounds("agent-setup-feedback").unwrap(),
                    cx.debug_bounds("screen-scroll").unwrap(),
                );
                cx.update(|_, cx| shell.update(cx, |s, cx| finish_setup(s, cx)));
                draw(cx);
                cx.update(|window, cx| {
                    shell.read_with(cx, |s, cx| assert_eq!(s.focused(window, cx), Some(next)))
                });
                crate::phase1_setup_tests::inside(
                    cx.debug_bounds(if matches!(next, Control::AgentRemove(_)) {
                        "action-AgentRemove(2)"
                    } else {
                        "action-AgentInstall(2)"
                    })
                    .unwrap(),
                    cx.debug_bounds("screen-scroll").unwrap(),
                );
                crate::phase1_setup_tests::inside(
                    cx.debug_bounds("agent-setup-feedback").unwrap(),
                    cx.debug_bounds("screen-scroll").unwrap(),
                );
            }
            // A foreign skill appears after the read, producing a real failed
            // install rather than a fabricated error presentation.
            let path = root.path().join(".copilot/skills/regressee");
            std::fs::create_dir_all(&path).unwrap();
            std::fs::write(path.join("SKILL.md"), "Keep my instructions").unwrap();
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.activate_control(Control::AgentInstall(2), window, cx);
                    finish_setup(s, cx);
                    assert!(s.integrations.error.is_some());
                })
            });
            draw(cx);
            cx.update(|window, cx| {
                shell.read_with(cx, |s, cx| {
                    assert_eq!(s.focused(window, cx), Some(Control::AgentRetry(2)))
                })
            });
            crate::phase1_setup_tests::inside(
                cx.debug_bounds("action-AgentRetry(2)").unwrap(),
                cx.debug_bounds("screen-scroll").unwrap(),
            );
            crate::phase1_setup_tests::inside(
                cx.debug_bounds("agent-setup-feedback").unwrap(),
                cx.debug_bounds("screen-scroll").unwrap(),
            );
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.activate_control(Control::AgentRetry(2), window, cx);
                    window.focus(&s.handle(Control::Details(Detail::LocalStorage), cx));
                    finish_setup(s, cx);
                })
            });
            draw(cx);
            cx.update(|window, cx| {
                shell.read_with(cx, |s, cx| {
                    assert_eq!(
                        s.focused(window, cx),
                        Some(Control::Details(Detail::LocalStorage))
                    )
                })
            });
            assert_eq!(
                std::fs::read_to_string(path.join("SKILL.md")).unwrap(),
                "Keep my instructions"
            );
            owned.join_after_ui();
        }
    }

    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    #[gpui::test]
    #[ignore = "explicit native two-size, two-theme integration setup inspection"]
    fn renderer_capture_agent_integrations(cx: &mut gpui::TestAppContext) {
        use crate::{
            phase4_hierarchy_tests::draw,
            ui_renderer_capture::{capture, output_root},
        };
        let out = output_root().unwrap();
        cx.enable_offscreen_renderer().unwrap();
        let root = tempfile::tempdir().unwrap();
        for (width, height) in [(1280., 800.), (820., 600.)] {
            for (theme, appearance) in [
                ("light", AppAppearance::Light),
                ("dark", AppAppearance::Dark),
            ] {
                for scene in [
                    "install",
                    "installed",
                    "attention",
                    "busy",
                    "success",
                    "error",
                ] {
                    let paths = AppPaths::isolated(root.path());
                    let (jobs, events) = Jobs::start_without_runtime(paths);
                    let owned = jobs.clone();
                    owned.appearance.wait_loaded();
                    owned.appearance.choose(appearance);
                    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                    cx.simulate_resize(size(px(width), px(height)));
                    let (sender, receiver) = std::sync::mpsc::channel();
                    cx.update(|window, cx| shell.update(cx, |s, cx| {
                        finish_jobs(s, cx);
                        s.focus.screen = Screen::Settings;
                        s.refresh_visible(cx);
                        finish_jobs(s, cx);
                        s.details.insert(Detail::AgentIntegrations);
                        s.integrations.statuses = Host::ALL.into_iter().enumerate().map(|(index, host)| {
                            let state = match scene { "installed" => State::Installed, "success" if index == 2 => State::Installed, "error" if index == 2 => State::Conflict, "attention" => [State::Conflict, State::UpdateAvailable, State::Unavailable][index], _ => State::NotInstalled };
                            Status { host, state, path: format!("/home/user/{}/skills/regressee", [".claude", ".agents", ".copilot"][index]).into(),
                                message: match state { State::Conflict => "This folder contains a skill not installed by Regressee. Keep or move it before installing; existing files were kept.", State::Unavailable => "The Regressee CLI is missing. Keep regressee beside regressee-desktop, then check again.", _ => "Skill files verified. Host access is not tested." }.into(), removable: matches!(state, State::Installed | State::UpdateAvailable) }
                        }).collect();
                        if scene == "busy" { s.integrations.pending = Some(receiver); s.integrations.action = Some(Control::AgentInstall(2)); s.integrations.target = Some(2); }
                        if scene == "success" { s.integrations.target = Some(2); s.integrations.feedback = Some("Skill installed. Skill files verified. Host access is not tested.".into()); }
                        if scene == "error" { s.integrations.target = Some(2); s.integrations.error = Some("Could not install the GitHub Copilot skill: this folder contains an unmanaged skill. Existing files were kept. Move or preserve it before retrying.".into()); }
                        s.sync_appearance(cx);
                        cx.set_global(controls::ReducedMotion(true));
                        window.focus(&s.handle(Control::Details(Detail::AgentIntegrations), cx));
                        cx.notify();
                    }));
                    draw(cx);
                    // Align the disclosure with the top of its real settings
                    // scroller; the remaining content scrolls at compact size.
                    let bounds = cx.debug_bounds("settings-agent-integrations").unwrap();
                    cx.update(|_, cx| {
                        shell.update(cx, |s, cx| {
                            let clip = s.screen_scroll.bounds();
                            let offset = s.screen_scroll.offset();
                            s.screen_scroll.set_offset(gpui::point(
                                offset.x,
                                offset.y + clip.top() - bounds.top(),
                            ));
                            cx.notify();
                        })
                    });
                    draw(cx);
                    if matches!(scene, "busy" | "success" | "error") {
                        cx.update(|window, cx| {
                            shell.update(cx, |s, cx| {
                                let target = match scene {
                                    "success" => Control::AgentRemove(2),
                                    "error" => Control::AgentRetry(2),
                                    _ => Control::AgentInstall(2),
                                };
                                window.focus(&s.handle(target, cx));
                                s.reveal_form_control(target, true, window, cx);
                                s.reveal_integration_feedback(target, window, cx);
                                cx.notify();
                            })
                        });
                        draw(cx);
                        crate::phase1_setup_tests::inside(
                            cx.debug_bounds("agent-setup-feedback").unwrap(),
                            cx.debug_bounds("screen-scroll").unwrap(),
                        );
                    }
                    capture(
                        cx,
                        &out,
                        &format!("agent-integrations-{width:.0}x{height:.0}-{theme}-{scene}"),
                    )
                    .unwrap();
                    cx.update(|window, cx| {
                        shell.update(cx, |s, cx| {
                            window.focus(&s.handle(Control::AgentRefresh, cx));
                            s.screen_scroll.set_offset(gpui::point(
                                px(0.),
                                -s.screen_scroll.max_offset().height,
                            ));
                            cx.notify();
                        })
                    });
                    draw(cx);
                    capture(
                        cx,
                        &out,
                        &format!("agent-integrations-{width:.0}x{height:.0}-{theme}-{scene}-tail"),
                    )
                    .unwrap();
                    drop(sender);
                    owned.join_after_ui();
                }
            }
        }
    }
}
