// SPDX-License-Identifier: MPL-2.0
//! Local consent and presentation. The engine independently enforces authority.
use super::*;
use regressee_app::control::{Command, Location, Session, Status, View};

impl Shell {
    pub(super) fn remember_agent_session(&mut self) {
        if let Some(control) = &self.agent.control
            && let Some(session) = control
                .sessions
                .iter()
                .find(|s| Some(s.id) == control.controller)
        {
            self.agent_last_session = Some((control.engine, session.clone()));
            self.agent_status_dismissed = None;
        }
    }
    fn last_agent_status(&self) -> Option<((uuid::Uuid, uuid::Uuid, Status), String)> {
        if self.controlling_agent().is_some() {
            return None;
        }
        let (old_engine, last) = self.agent_last_session.as_ref()?;
        let (engine, status, reason) = match &self.agent.control {
            None => (
                *old_engine,
                Status::Expired,
                "Connection lost. This window is unlocked; background jobs were not cancelled.",
            ),
            Some(control) if control.engine != *old_engine => (
                control.engine,
                Status::Ended,
                "Engine restarted. Previous control ended; this window is unlocked. Inspect retained job outcomes before continuing.",
            ),
            Some(control) => {
                let status = control
                    .sessions
                    .iter()
                    .find(|s| s.id == last.id)
                    .map_or(Status::Ended, |s| s.status);
                let reason = match status {
                    Status::Expired => {
                        "Session expired. This window is unlocked; background jobs continue. Start a new agent session deliberately to resume."
                    }
                    Status::Revoked => {
                        "Agent control was taken back. This window is unlocked; background jobs continue."
                    }
                    Status::Ended => {
                        "Agent session ended. This window is unlocked; background jobs continue."
                    }
                    _ => {
                        "Frontend disconnected. This window is unlocked; background jobs were not cancelled."
                    }
                };
                (control.engine, status, reason)
            }
        };
        let key = (engine, last.id, status);
        (self.agent_status_dismissed != Some(key))
            .then(|| (key, format!("{reason} Agent: {}", last.name)))
    }
    pub(super) fn agent_status_text(&self) -> Option<String> {
        self.last_agent_status().map(|(_, text)| text)
    }
    pub(super) fn agent_banner_visible(&self) -> bool {
        self.controlling_agent().is_some()
            || self.agent_status_text().is_some()
            || self.agent_message.is_some()
            || (self.agent.error.is_some()
                && (!self.agent.connection_error || self.agent_last_session.is_none()))
            || !self.agent.external.is_empty()
    }
    pub(super) fn dismiss_agent_status(&mut self, cx: &mut Context<Self>) {
        if let Some((key, _)) = self.last_agent_status() {
            self.agent_status_dismissed = Some(key);
            self.presentation_focus = Some(Control::Navigate(self.focus.screen));
            cx.notify();
        }
    }
    pub(super) fn controlling_agent(&self) -> Option<&Session> {
        let c = self.agent.control.as_ref()?;
        c.sessions.iter().find(|s| Some(s.id) == c.controller)
    }
    pub(super) fn agent_following(&self) -> bool {
        self.controlling_agent()
            .is_some_and(|s| s.follow.is_some() && s.follow == self.agent.frontend)
    }
    pub(super) fn agent_locked(&self) -> bool {
        self.agent_transition
            || (self.agent_following()
                && self
                    .controlling_agent()
                    .is_some_and(|s| s.status == Status::Active))
    }
    pub(super) fn agent_drafts(&self, cx: &App) -> bool {
        self.exit_project_dirty(cx)
            || self.exit_builder_dirty(cx)
            || self.exit_network_dirty(cx)
            || self.review_unsaved(cx)
            || self.mask_dirty
            || self.mask_drag.is_some()
            || !self.approval_selection.is_empty()
            || self.mask_geometry_dirty(cx)
            || self.analysis_policy_draft(cx)
            || self
                .pending_review_notes(cx)
                .map_or(true, |notes| !notes.is_empty())
            || self.preview.is_some()
            || self.name.read(cx).composing()
            || self.address.read(cx).composing()
            || self.history_search.read(cx).composing()
            || self.timezone.read(cx).is_composing(cx)
            || self
                .crawl_fields
                .iter()
                .chain(&self.analysis_fields)
                .any(|i| i.read(cx).composing())
    }
    pub(super) fn agent_can_follow(&self, cx: &App) -> bool {
        !self.agent_drafts(cx)
            && self.pending.is_none()
            && self.task.is_none()
            && !self.native_prompt
            && !self.focus.dialog
            && !self.focus.approval_dialog
            && self.analysis_pending.is_none()
            && self.analysis_due.is_none()
            && !self.exit_guard.saving()
    }
    pub(super) fn change_agent_control(
        &mut self,
        command: Command,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.agent.pending && !matches!(command, Command::Takeover) {
            return;
        }
        if matches!(command, Command::Follow { .. } | Command::Return) && !self.agent_can_follow(cx)
        {
            self.agent_message=Some("Save or deliberately discard your drafts and finish the open task or dialog before following. Your edits are kept.".into());
            cx.notify();
            return;
        }
        self.agent_message = None;
        self.agent_transition = matches!(command, Command::Follow { .. } | Command::Return);
        if self.agent_transition {
            self.pan_drag = None;
            self.split_drag = None;
            self.mask_drag = None;
            self.viewer_drag_pane = None;
            self.disable_agent_inputs(cx);
        }
        self.agent.pending = true;
        self.jobs.agent_command(command);
        window.focus(&self.agent_handles[0]);
        cx.notify();
    }
    pub(super) fn take_agent_control(
        &mut self,
        _: &TakeControl,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if self.controlling_agent().is_none() && !self.agent_transition {
            return;
        }
        // No task/modal/draft guard may make immediate takeover inaccessible.
        self.change_agent_control(Command::Takeover, window, cx);
    }
    pub(super) fn follow_agent(
        &mut self,
        _: &FollowAgent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(session) = self.controlling_agent().map(|s| s.id) {
            self.change_agent_control(Command::Follow { session }, window, cx);
        }
    }
    pub(super) fn return_agent(
        &mut self,
        _: &ReturnToAgent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.agent_following()
            || !self
                .controlling_agent()
                .is_some_and(|s| s.status == Status::Human)
        {
            return;
        }
        self.change_agent_control(Command::Return, window, cx);
    }
    pub(super) fn poll_agent(&mut self, cx: &mut Context<Self>) {
        if !self.jobs.is_remote() {
            return;
        }
        let next = self.jobs.agent_state();
        if self.agent.same_revision(&next) {
            return;
        }
        if next.completion != self.agent.completion || next.control.is_none() {
            self.agent_transition = false;
        }
        self.remember_agent_session();
        self.agent = next;
        self.remember_agent_session();
        if !self.agent_following() {
            self.agent_exact = None;
            self.agent_waiting = false;
        }
        if self.agent_following()
            && self
                .controlling_agent()
                .is_some_and(|s| s.status == Status::Active)
        {
            let control = self.agent.control.as_ref().unwrap();
            let agent = self.controlling_agent().unwrap();
            let current = (
                control.engine,
                control.epoch,
                agent.id,
                agent.selection_revision,
                self.agent.data_revision,
            );
            if (self.agent_location != Some(current) || self.agent.selection_unavailable)
                && !self.agent_waiting
            {
                self.agent_waiting = true;
                self.pending = None;
                self.storage_pending = None;
                self.viewer_pending = None;
                self.focus.busy = false;
                self.focus.view_loading = false;
                self.clear_viewer();
                self.project = None;
                self.selected_key = None;
                self.selected_page = None;
                self.evidence = ProjectEvidence::default();
                self.focus.project_saved = false;
                self.agent_exact = None;
                self.name.update(cx, |i, cx| i.set_value("", cx));
                self.address.update(cx, |i, cx| i.set_value("", cx));
                self.draft_settings = Project::draft("New website".into(), "https://example.test")
                    .expect("fixed defaults");
                self.set_crawl_fields(cx);
                self.environment = None;
                self.environment_draft = None;
                self.login_hydration_origin = None;
                self.login_ephemeral = false;
                for input in &self.crawl_fields[10..13] {
                    input.update(cx, |i, cx| i.set_value("", cx));
                }
                self.review_drafts = Default::default();
                self.analysis_fields[9].update(cx, |i, cx| i.set_value("", cx));
                self.mask_rectangles.clear();
                self.mask_selected = None;
                self.mask_dirty = false;
                self.mask_draw = false;
            }
        } else {
            self.agent_waiting = false;
        }
        if self.agent_locked() {
            self.analysis_due = None;
            self.pan_drag = None;
            self.split_drag = None;
            self.mask_drag = None;
            self.viewer_mode_open = false;
            // Disable already focused editors at the same state boundary, not
            // merely when a later frame paints the mouse shield.
            self.disable_agent_inputs(cx);
        }
        self.input_gate.set_allowed(!self.agent_locked());
        if self.agent_following()
            && self
                .controlling_agent()
                .is_some_and(|s| s.status == Status::Active)
            && let Some((engine, epoch, id, revision, data, location)) = self.agent.location.clone()
            && data == self.agent.data_revision
            && self.agent.control.as_ref().is_some_and(|c| {
                c.engine == engine
                    && c.epoch == epoch
                    && c.controller == Some(id)
                    && c.sessions.iter().any(|s| {
                        s.id == id
                            && s.selection_revision == revision
                            && s.selection.as_ref() == Some(&location.selection)
                    })
            })
            && self.agent_location != Some((engine, epoch, id, revision, data))
        {
            self.agent_location = Some((engine, epoch, id, revision, data));
            self.apply_agent_location((*location).clone(), cx);
        }
        cx.notify();
    }
    fn disable_agent_inputs(&self, cx: &mut Context<Self>) {
        self.input_gate.set_allowed(false);
        self.settings_scrollbar.cancel_drag();
        for input in std::iter::once(&self.name)
            .chain(std::iter::once(&self.address))
            .chain(&self.crawl_fields)
            .chain(&self.analysis_fields)
            .chain(std::iter::once(&self.history_search))
        {
            input.update(cx, |i, _| i.set_enabled(false));
        }
        self.timezone.update(cx, |s, cx| s.suspend_input(cx));
        self.zoom_select.update(cx, |s, cx| s.suspend_input(cx));
    }
    pub(super) fn apply_agent_location(&mut self, location: Location, cx: &mut Context<Self>) {
        self.agent_waiting = false;
        // A delayed read cannot hydrate a different project or default/latest
        // run. Invalidate every local async identity before installing the exact
        // engine-validated selection as one unit.
        self.pending = None;
        self.storage_pending = None;
        self.viewer_pending = None;
        self.focus.busy = false;
        self.focus.view_loading = false;
        self.clear_viewer();
        self.invalidate_approval();
        self.approval_selection.clear();
        self.history = None;
        self.project_storage = None;
        self.environment = None;
        self.focus.choosing_project = false;
        self.focus.project_saved = true;
        self.draft_settings = location.project.project.clone();
        self.name
            .update(cx, |i, cx| i.set_value(&location.project.project.name, cx));
        self.address.update(cx, |i, cx| {
            i.set_value(
                &location
                    .project
                    .project
                    .sources
                    .iter()
                    .map(|u| u.as_str())
                    .collect::<Vec<_>>()
                    .join(" "),
                cx,
            )
        });
        self.project = Some(location.project);
        self.set_crawl_fields(cx);
        self.refresh_setup_origins(cx);
        self.environment_draft = None;
        self.login_ephemeral = false;
        self.environment_read_signature = None;
        for input in &self.crawl_fields[10..13] {
            input.update(cx, |i, cx| i.set_value("", cx));
        }
        self.login_hydration_origin = Some(self.environment_draft_signature(cx));
        self.evidence = location.evidence.unwrap_or_default();
        *self.local_index.borrow_mut() = Default::default();
        *self.history_filter_cache.borrow_mut() = Default::default();
        if let Some(policy) = self
            .evidence
            .comparison
            .as_ref()
            .and_then(|a| a.perceptual.as_ref())
            .map(|p| p.policy.clone())
        {
            self.analysis_policy = policy;
            self.set_analysis_fields(cx);
        }
        self.capture_details_open = false;
        self.review_note_open = false;
        self.review_tools = false;
        self.image_controls_open = false;
        self.result_filters_open = false;
        self.evidence_dirty = false;
        self.evidence_read_failed = false;
        self.viewer_read_failed = false;
        self.selected_key = location.selection.key.clone();
        if self.selected_key.is_none() {
            self.selected_key = self
                .evidence
                .latest
                .as_ref()
                .and_then(|r| r.run.targets.first())
                .map(|t| t.key.clone());
        }
        self.selected_page = self.selected_key.as_ref().map(|k| k.page);
        self.mask_rectangles.clear();
        self.mask_selected = None;
        self.mask_draw = false;
        self.mask_dirty = false;
        self.search.clear();
        self.crawl_fields[9].update(cx, |i, cx| i.set_value("", cx));
        self.result_filter = 0;
        self.focus.screen = if location.selection.run.is_some() {
            Screen::Results
        } else {
            Screen::Setup
        };
        self.focus.setup_stage = SetupStage::Settings;
        self.focus.review_surface = ReviewSurface::Changes;
        self.focus.preview = false;
        self.focus.viewer = self.evidence.latest.is_some();
        self.focus.baseline = self.evidence.baseline.is_some();
        self.focus.candidate = false;
        self.history_inspection = location
            .selection
            .run
            .map(|run| (location.selection.project, run));
        self.viewer_mode = match location.selection.view {
            View::Compare => 0,
            View::Before | View::After => 4,
            View::Difference => 1,
            View::Overlay => 2,
            View::Split => 5,
        };
        self.full_width_after = location.selection.view == View::After;
        self.pan_y = 0;
        self.pan_x = 0.;
        self.viewer_fit = true;
        self.agent_exact =
            location
                .selection
                .run
                .map(|run| regressee_app::protocol::ViewerSelection {
                    project: location.selection.project,
                    run: Some(run),
                    analysis: location.selection.analysis,
                    key: self.selected_key.clone(),
                    tile: 0,
                });
        self.reload_viewer(cx);
        cx.notify();
    }
    pub(super) fn agent_shared_feedback(&self) -> bool {
        self.focus.screen == Screen::Results
            && !self.focus.shows_project_picker()
            && self.task.is_none()
            && (self.error.is_some() || self.notice.is_some())
            && (self.controlling_agent().is_some() || self.agent_status_text().is_some())
    }
    pub(super) fn agent_banner(&self, window_width: f32, cx: &mut Context<Self>) -> AnyElement {
        let shared_feedback = self.agent_shared_feedback();
        let mut banner = div()
            .id("agent-control-banner")
            .debug_selector(|| "agent-control-banner".into())
            .flex_shrink_0()
            .px_6()
            .py_2()
            .bg(rgb(palette().surface))
            .border_b_1()
            .border_color(rgb(palette().line))
            .flex()
            .flex_col()
            .gap_2();
        let mut details = div().min_w_0().flex().flex_col();
        if shared_feedback {
            if let Some(error) = &self.error {
                details = details.child(
                    div()
                        .debug_selector(|| "agent-business-error".into())
                        .text_color(rgb(palette().error))
                        .child(format!("Error (scroll for details): {error}")),
                );
            }
            if let Some(notice) = &self.notice {
                details = details.child(
                    div()
                        .debug_selector(|| "agent-business-notice".into())
                        .child(format!("Feedback (scroll for details): {notice}")),
                );
            }
        }
        if let Some(message) = self.agent_message.as_ref().or(self.agent.error.as_ref()) {
            details = details.child(format!("Control status (scroll for details): {message}"));
        }
        if let Some(agent) = self.controlling_agent() {
            let following = self.agent_following();
            // Percentage-width clipped text can resolve to zero under GPUI's
            // auto-width flex owner. Use a finite native measure, reserving the
            // pinned 40px actions and gaps, instead of a percentage here.
            let second = !following || agent.status == Status::Human;
            let text_width = (window_width
                - 48.
                - if second { 284. } else { 132. }
                - if shared_feedback { 156. } else { 0. })
            .max(1.);
            let status = if self.agent_transition {
                "Changing control…"
            } else if agent.status == Status::Human {
                "Your turn — agent paused"
            } else if following {
                "Following — agent controls this window"
            } else {
                "Not following — your view stays here"
            };
            let mut row = div().flex().items_center().gap_3().min_w_0().child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .h(px(40.))
                    .child(
                        div()
                            .debug_selector(|| "agent-control-name".into())
                            .h(px(20.))
                            .flex_shrink_0()
                            .w(px(text_width))
                            .min_w_0()
                            .line_height(px(20.))
                            .truncate()
                            .child(agent.name.clone()),
                    )
                    .child(
                        div()
                            .debug_selector(|| "agent-control-status".into())
                            .h(px(20.))
                            .flex_shrink_0()
                            .w(px(text_width))
                            .min_w_0()
                            .text_size(px(HELP_TEXT_SIZE))
                            .line_height(px(20.))
                            .truncate()
                            .child(format!("{status} · lease {}s", agent.remaining_seconds)),
                    ),
            );
            row = row.child(
                controls::action(
                    "agent-take-control",
                    "Take control",
                    &self.agent_handles[0],
                    ActionStyle::Primary,
                    true,
                )
                .debug_selector(|| "agent-take-control".into())
                .on_click(cx.listener(|s, _, w, cx| s.take_agent_control(&TakeControl, w, cx))),
            );
            if !following || agent.status == Status::Human {
                let identity = (self.agent.control.as_ref().unwrap().engine, agent.id);
                let label = if following {
                    "Return to agent"
                } else {
                    "Follow agent"
                };
                row = row.child(
                    controls::action(
                        "agent-follow-return",
                        label,
                        &self.agent_handles[1],
                        ActionStyle::Secondary,
                        !self.agent.pending,
                    )
                    .debug_selector(|| "agent-follow-return".into())
                    .on_click(cx.listener(move |s, _, w, cx| {
                        if s.agent.control.as_ref().map(|c| c.engine) != Some(identity.0)
                            || s.controlling_agent().map(|a| a.id) != Some(identity.1)
                        {
                            return;
                        }
                        if following {
                            s.return_agent(&ReturnToAgent, w, cx)
                        } else {
                            s.follow_agent(&FollowAgent, w, cx)
                        }
                    })),
                );
            }
            if shared_feedback {
                row = row.child(self.button(
                    Control::DismissFeedback,
                    "Dismiss feedback",
                    ActionStyle::Secondary,
                    cx,
                ));
            }
            banner = banner.child(row);
            details = details.child(if let Some(message) = &agent.handoff {
                format!("Request (scroll for more): {message} · Agent: {} · Use the app normally. Save your edits before Return to agent.", agent.name)
            } else { "Take control: Ctrl/Cmd+Shift+T. Closing or quitting never cancels background jobs.".into() });
        } else if let Some(((_, _, status), text)) = self.last_agent_status() {
            let headline = if self.agent.control.is_none() {
                "Agent disconnected — window unlocked"
            } else if self.agent_last_session.as_ref().is_some_and(|(engine, _)| {
                self.agent
                    .control
                    .as_ref()
                    .is_some_and(|c| c.engine != *engine)
            }) {
                "Engine restarted — window unlocked"
            } else {
                match status {
                    Status::Expired => "Agent session expired — window unlocked",
                    Status::Revoked => "Control taken back — window unlocked",
                    _ => "Agent session ended — window unlocked",
                }
            };
            let mut row = div()
                .flex()
                .items_center()
                .gap_3()
                .min_w_0()
                .child(div().flex_1().min_w_0().truncate().child(headline))
                .child(
                    controls::action(
                        "agent-dismiss-status",
                        "Dismiss",
                        &self.agent_handles[0],
                        ActionStyle::Secondary,
                        true,
                    )
                    .debug_selector(|| "agent-dismiss-status".into())
                    .on_click(cx.listener(|s, _, _, cx| s.dismiss_agent_status(cx))),
                );
            if shared_feedback {
                row = row.child(self.button(
                    Control::DismissFeedback,
                    "Dismiss feedback",
                    ActionStyle::Secondary,
                    cx,
                ));
            }
            banner = banner.child(row);
            details = details.child(text);
        }
        for job in self.agent.external.iter().take(2) {
            let label = if let Some(Outcome::Progress {
                message,
                completed,
                total,
            }) = &job.progress
            {
                format!(
                    "{message} · {completed}{}",
                    total.map(|n| format!(" / {n}")).unwrap_or_default()
                )
            } else if let Some(capture) = &job.capture {
                format!("{} · run {}", job.label, capture.run)
            } else {
                job.label.clone()
            };
            details = details.child(
                div()
                    .debug_selector(|| "external-job-activity".into())
                    .text_size(px(HELP_TEXT_SIZE))
                    .child(format!("{label} · {:?}", job.state)),
            );
        }
        // All untrusted variable-length request/error/activity text shares one
        // bounded reading owner. Status/actions never scroll out of reach and
        // the evidence workspace keeps a useful viewport at 820×600.
        banner
            .child(
                div()
                    .id("agent-control-details")
                    .debug_selector(|| "agent-control-details".into())
                    .h(px(22.))
                    .min_h(px(22.))
                    .w_full()
                    .min_w_0()
                    .overflow_hidden()
                    .overflow_y_scroll()
                    .track_scroll(&self.agent_details_scroll)
                    .track_focus(&self.agent_handles[2])
                    .focus(|s| s.border_b_1().border_color(rgb(palette().ink)))
                    .text_size(px(HELP_TEXT_SIZE))
                    .line_height(px(HELP_LINE_HEIGHT))
                    .text_color(rgb(palette().muted))
                    .on_key_down(cx.listener(|s, event: &gpui::KeyDownEvent, _, cx| {
                        let delta = match event.keystroke.key.as_str() {
                            "up" => 22.,
                            "down" => -22.,
                            "pageup" => 44.,
                            "pagedown" => -44.,
                            "home" => f32::MAX,
                            "end" => -f32::MAX,
                            _ => return,
                        };
                        let offset = s.agent_details_scroll.offset();
                        s.agent_details_scroll.set_offset(gpui::point(
                            offset.x,
                            (offset.y + px(delta))
                                .clamp(-s.agent_details_scroll.max_offset().height, px(0.)),
                        ));
                        cx.stop_propagation();
                        cx.notify();
                    }))
                    .child(details),
            )
            .into_any_element()
    }
}
