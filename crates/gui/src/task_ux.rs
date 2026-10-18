// SPDX-License-Identifier: MPL-2.0
//! Exclusive tasks own their presentation, focus and stop lifecycle.
use super::*;
use regressee_app::recovery::Problem;
use regressee_gui::recovery::Destination;
use regressee_gui::{
    task::{CaptureDetails, CaptureStatus, Kind, Spec, State, Surface, capture_recovery},
    ui::{
        motion::{self, Entrance},
        status_icon::{StatusIcon, status_icon},
        web_link,
    },
};

pub(super) struct Task {
    pub request: u64,
    pub spec: Spec,
    pub state: State,
    pub origin: Screen,
    pub restore: Option<Control>,
    pub project: String,
    pub project_id: Option<regressee_core::domain::ProjectId>,
    pub website: String,
    pub environment: String,
    pub details: bool,
    pub started: std::time::Instant,
    pub elapsed: u64,
    pub result: Option<Outcome>,
    pub settings_saved_before_task: bool,
    pub progress: Option<(String, u32, Option<u32>)>,
    pub capture: Option<CaptureStatus>,
    pub scroll: gpui::ScrollHandle,
    pub reveal_details: std::rc::Rc<std::cell::Cell<bool>>,
    pub details_bounds: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
    pub link_bounds: Vec<std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>>,
}

pub(super) struct Outcome {
    pub(super) title: String,
    pub(super) message: String,
    pub(super) action: &'static str,
    failed: bool,
    destination: Option<(Screen, ReviewSurface)>,
    pub(super) recovery: Option<Destination>,
    pub(super) update_browser: bool,
    diagnostic: Option<String>,
    tone: Tone,
    icon: StatusIcon,
    counts: Option<Counts>,
    pub(super) capture: Option<CaptureDetails>,
    notes: Vec<String>,
    note: Option<(&'static str, String)>,
}

enum Counts {
    Discovery {
        included: usize,
        excluded: usize,
        visited: usize,
    },
    Capture(crawl::TargetCounts),
}

impl Task {
    pub(super) fn can_update_browser(&self) -> bool {
        self.state == State::Finished
            && self
                .result
                .as_ref()
                .is_some_and(|result| result.update_browser)
    }

    fn controls(&self) -> Vec<Control> {
        let mut controls = Vec::new();
        for index in 0..regressee_gui::task::MAX_TASK_LINKS {
            if index == 1 && self.has_details() {
                controls.push(Control::TaskDetails);
            }
            if self.link_url(index).is_some() {
                controls.push(Control::TaskLink(index));
            }
        }
        if self.state == State::Finished {
            if self.can_update_browser() {
                controls.push(Control::SetupRuntime);
            }
            controls.push(Control::TaskContinue);
        } else if matches!(self.state, State::Running | State::Pausing) {
            if self.spec.stop_label.is_some() {
                controls.push(Control::CancelCapture);
            }
            if self.spec.pausable && self.state == State::Running {
                controls.push(Control::PauseCapture);
            }
        }
        controls
    }

    fn link_address(&self, index: usize) -> Option<&str> {
        if index == 0 {
            return Some(&self.website);
        }
        if !self.details {
            return None;
        }
        let gaps = &self.result.as_ref()?.capture.as_ref()?.gaps;
        let gap = gaps.get((index - 1) / 2)?;
        if (index - 1).is_multiple_of(2) {
            Some(&gap.page)
        } else {
            gap.final_url.as_deref()
        }
    }

    pub(super) fn link_url(&self, index: usize) -> Option<url::Url> {
        web_link::web_url(self.link_address(index)?)
    }

    pub(super) fn has_details(&self) -> bool {
        self.result
            .as_ref()
            .map_or(self.capture.is_some(), |result| {
                result.diagnostic.is_some()
                    || result.capture.is_some()
                    || self.capture.is_some()
                    || !result.notes.is_empty()
            })
    }

    fn outcome(&self, reply: &Reply) -> Outcome {
        let mut outcome = Outcome {
            title: "Task finished".into(),
            message: "The operation finished successfully.".into(),
            action: "Continue",
            failed: false,
            destination: None,
            recovery: None,
            update_browser: false,
            diagnostic: None,
            tone: Tone::Neutral,
            icon: StatusIcon::Saved,
            counts: None,
            capture: None,
            notes: Vec::new(),
            note: None,
        };
        match reply {
            Reply::Discovery(preview) => {
                let stopped = preview
                    .notes
                    .iter()
                    .any(|n| n == "Discovery cancelled; the preview is incomplete.");
                let limited = !matches!(
                    preview.inventory.coverage,
                    regressee_core::domain::DiscoveryCoverage::Complete
                );
                outcome.title = if stopped {
                    "Stopped finding pages"
                } else {
                    "Pages found"
                }
                .into();
                outcome.message = if limited {
                    "This list is incomplete or limited. Review the included pages before capturing screenshots."
                } else {
                    "Finished finding pages within your website boundaries. Review the list before capturing screenshots."
                }.into();
                outcome.counts = Some(Counts::Discovery {
                    included: preview.included(),
                    excluded: preview.inventory.pages.len() - preview.included(),
                    visited: preview.visited as usize,
                });
                if limited {
                    outcome.tone = Tone::Warning;
                    outcome.icon = StatusIcon::Warning;
                }
                if let regressee_core::domain::DiscoveryCoverage::Limited { reason } =
                    &preview.inventory.coverage
                {
                    outcome.notes.push(reason.clone());
                }
                for note in &preview.notes {
                    if note != "Discovery cancelled; the preview is incomplete."
                        && !outcome.notes.contains(note)
                    {
                        outcome.notes.push(note.clone());
                    }
                }
                outcome.note = Some((
                    "Your baseline is unchanged",
                    "Finding pages does not save or replace a baseline.".into(),
                ));
                outcome.action = "Review found pages";
                outcome.destination = Some((Screen::Results, ReviewSurface::Pages));
            }
            Reply::Evidence(evidence) | Reply::Captured(evidence) => {
                if let Some(run) = &evidence.latest
                    && self.project_id == Some(run.run.project)
                    && matches!(
                        run.run.state,
                        JobState::Completed
                            | JobState::CompletedWithErrors
                            | JobState::Paused
                            | JobState::Cancelled
                            | JobState::Interrupted
                    )
                    && self
                        .capture
                        .as_ref()
                        .is_none_or(|status| status.run == run.run.id)
                {
                    let counts = regressee_app::task::target_counts(&run.run);
                    outcome.title = match run.run.state {
                        JobState::Paused => "Capture paused",
                        JobState::Cancelled => "Capture stopped",
                        JobState::CompletedWithErrors | JobState::Interrupted => {
                            "Capture needs attention"
                        }
                        _ if self.spec.kind == Kind::Check => "Website check finished",
                        _ if self.spec.kind == Kind::Representative => "Preview captured",
                        _ => "Screenshot capture finished",
                    }
                    .into();
                    outcome.message = match run.run.state {
                        JobState::Paused => "Return to the workspace to resume this paused capture.",
                        JobState::Cancelled => "Start a new capture from the workspace to capture the remaining pages.",
                        JobState::Interrupted => "Review the saved work and resume the interrupted capture from the workspace.",
                        _ => "Review the screenshots and any pages that could not be captured.",
                    }.into();
                    if run.run.state == JobState::CompletedWithErrors && counts.captured == 0 {
                        outcome.title = "No screenshots captured".into();
                        outcome.message = "None of the selected screenshots could be captured. Review the failures before trying again.".into();
                        if let Some(TargetOutcome::Failed { code }) = run.run.targets.first().map(|t| &t.outcome)
                            && run.run.targets.iter().all(|t| matches!(t.outcome, TargetOutcome::Failed { code: other } if other == *code))
                        {
                            outcome.message = format!("{} failed: {}. {}", workflow::count_label(counts.failed, "screenshot attempt", "screenshot attempts"), code.label(), capture_recovery(*code));
                        }
                    }
                    if run
                        .baseline_save_counts()
                        .is_some_and(|(captured, intended)| captured < intended)
                    {
                        outcome.message = "You can save the successful screenshots as your baseline and skip those that could not be captured. Review the results, then choose Save baseline.".into();
                    }
                    if run.run.capture_plan.mode.dom_snapshots() {
                        outcome.title = match run.run.state {
                            JobState::Paused => "Capture paused",
                            JobState::Cancelled => "Capture stopped",
                            JobState::CompletedWithErrors | JobState::Interrupted => {
                                "Capture needs attention"
                            }
                            _ if self.spec.kind == Kind::Check => "Website check finished",
                            _ => "Page capture finished",
                        }
                        .into();
                        outcome.message = "Review each requested kind independently. A saved screenshot does not mean the DOM snapshot succeeded, or vice versa.".into();
                        if regressee_gui::task::baseline_consent_coverage(run).is_some_and(|c| {
                            c.dom_captures < c.dom_intended
                                || c.screenshot_captures < c.screenshot_intended
                        }) {
                            outcome.message.push_str(" Save baseline requires explicit consent to omit each missing kind; missing coverage needs a new baseline capture.");
                        }
                    }
                    outcome.tone = if run.run.state == JobState::Completed {
                        Tone::Neutral
                    } else {
                        Tone::Warning
                    };
                    outcome.icon = match run.run.state {
                        JobState::Paused => StatusIcon::Paused,
                        JobState::Cancelled => StatusIcon::Pending,
                        JobState::Completed => StatusIcon::Saved,
                        _ => StatusIcon::Warning,
                    };
                    outcome.counts = Some(Counts::Capture(counts));
                    outcome.capture = Some(CaptureDetails::from_run(&run.run));
                    outcome.note = Some((
                        "Your baseline is unchanged",
                        if run.run.capture_plan.mode.dom_snapshots() {
                            "Saved captures are kept."
                        } else {
                            "Saved screenshots are kept."
                        }
                        .into(),
                    ));
                    outcome.failed = matches!(
                        run.run.state,
                        JobState::CompletedWithErrors | JobState::Interrupted
                    );
                } else {
                    outcome.title = "Capture results unavailable".into();
                    outcome.message = "Results for this task could not be loaded. Return to the workspace and choose Refresh to load saved results.".into();
                    outcome.failed = true;
                    outcome.tone = Tone::Error;
                    outcome.icon = StatusIcon::Error;
                }
                if outcome.capture.is_some() {
                    outcome.action = if outcome
                        .capture
                        .as_ref()
                        .is_some_and(|c| c.setup.counts.captured == 0)
                    {
                        "Review capture failures"
                    } else {
                        "Review captured pages"
                    };
                    outcome.destination = Some((Screen::Results, ReviewSurface::Inspect));
                    if self.spec.kind == Kind::Check {
                        outcome.action = "Review check results";
                        outcome.destination = Some((Screen::Results, ReviewSurface::Changes));
                    }
                    if outcome.capture.as_ref().is_some_and(|_| {
                        evidence.latest.as_ref().is_some_and(|r| {
                            matches!(r.run.state, JobState::Paused | JobState::Interrupted)
                        })
                    }) {
                        outcome.action = "Return to capture recovery";
                    }
                } else {
                    outcome.action = "Return to workspace";
                }
            }
            Reply::Cancelled => {
                outcome.title = "Task stopped".into();
                outcome.icon = StatusIcon::Pending;
                outcome.message = match self.spec.kind {
                    Kind::Discovery => "Stopped before a page list was ready. Your saved baseline stays unchanged.",
                    Kind::Capture | Kind::Check | Kind::Representative => "Active work stopped. Saved screenshots are kept; the saved baseline stays unchanged. Comparison analysis may be incomplete.",
                    Kind::Setup => "Browser setup stopped. Any browser already installed successfully is kept; your projects stay unchanged.",
                    Kind::Import => "Import stopped before the project was saved. Files from this attempt may still be stored for recovery.",
                    Kind::Export => "Export stopped before the ZIP was saved. Your project stays unchanged.",
                    Kind::ArchivePreview => "ZIP inspection stopped. No project was imported.",
                    Kind::Diagnostics => "Support report preparation or saving stopped before the file was saved. Nothing was sent.",
                    Kind::Login => "The browser or session check stopped. Any previously saved session is kept.",
                    Kind::Access => "The website access check stopped. No screenshots or baseline were changed.",
                    _ => "The operation stopped.",
                }.into();
                outcome.action = "Return to workspace";
            }
            Reply::Failure(error) => {
                outcome.title = format!("{} could not finish", self.spec.title);
                outcome.message = error.message().into();
                outcome.action = "Return to workspace";
                outcome.failed = true;
                outcome.tone = Tone::Error;
                outcome.icon = StatusIcon::Error;
            }
            Reply::ProductFailure(error) => {
                outcome.title = format!("{} could not finish", self.spec.title);
                let write_error = error.strip_prefix("storage_write_failed: ");
                let explanation: String = write_error
                    .unwrap_or(error)
                    .lines()
                    .next()
                    .filter(|line| !line.trim().is_empty())
                    .unwrap_or("The cause was not reported.")
                    .chars()
                    .take(1024)
                    .map(|c| if c.is_control() { ' ' } else { c })
                    .collect();
                outcome.message = format!(
                    "{explanation}\n{}",
                    regressee_gui::task::failure_recovery(self.spec.kind)
                );
                if write_error.is_some() {
                    outcome.title = "Project settings could not be saved".into();
                }
                if error.starts_with(regressee_runtime::setup_diagnostics::FAILURE_PREFIX) {
                    outcome.title = "Browser setup could not finish".into();
                    outcome.message = format!(
                        "{} Keep the error details below if setup fails again. Your projects and screenshots are unchanged.",
                        error.lines().next().unwrap_or("Browser setup failed.")
                    );
                }
                outcome.action = "Return to workspace";
                outcome.failed = true;
                outcome.tone = Tone::Error;
                outcome.icon = StatusIcon::Error;
                outcome.diagnostic = Some(error.clone());
                if self.spec.kind == Kind::Setup
                    || error.starts_with(regressee_runtime::setup_diagnostics::FAILURE_PREFIX)
                {
                    outcome.recovery = Some(Destination::AppSettings);
                    outcome.action = "Open App settings";
                }
            }
            Reply::ActionRequired(problem) => {
                outcome.title = problem.title().into();
                outcome.message = problem.message().into();
                if self.spec.kind == Kind::Check && matches!(problem, Problem::Browser(_)) {
                    outcome.message.push_str(" Checks need the browser version used for the baseline. Updating the browser may not be enough to compare against an older baseline. You can choose to capture a new baseline instead.");
                }
                outcome.recovery = Some(problem.destination());
                outcome.action = problem.destination().label();
                if self.spec.kind != Kind::Setup
                    && matches!(
                        problem,
                        Problem::Browser(regressee_app::product::RuntimeState::UpdateRequired)
                    )
                {
                    outcome.update_browser = true;
                    outcome.recovery = Some(Destination::Workspace);
                    outcome.action = if self.origin == Screen::Setup {
                        "Back to setup"
                    } else {
                        "Return to workspace"
                    };
                    outcome.message = "The installed private-browser components do not match this app build. Choose Update browser to download and verify the matching components here, then retry your task. Your project settings, saved screenshots and baseline are kept. Updating does not visit your website.".into();
                    if self.spec.kind == Kind::Check {
                        outcome.message.push_str(" Checks still need the browser version used for the baseline; updating may not make an older baseline comparable.");
                    }
                }
                if self.project_id.is_none() && problem.destination() == Destination::History {
                    outcome.recovery = Some(Destination::AppSettings);
                    outcome.action = "Open App settings";
                    outcome.message = "There is not enough free disk space to finish. Free space before retrying. App settings lists the local data and cache folders; no cleanup will run automatically.".into();
                }
                if matches!(self.spec.kind, Kind::Export | Kind::Diagnostics)
                    && matches!(problem, Problem::PermissionDenied)
                {
                    outcome.recovery = Some(Destination::Workspace);
                    outcome.action = "Return to workspace";
                    outcome.message = "The selected destination is not writable. Return to the workspace and choose a different folder or fix its permissions before exporting again. Your local project is kept; nothing was uploaded.".into();
                }
                outcome.failed = true;
                let prerequisite = matches!(
                    problem,
                    Problem::Browser(_)
                        | Problem::Worker(regressee_runtime::protocol::FailureCode::NeedsLogin)
                );
                outcome.tone = if prerequisite {
                    Tone::Warning
                } else {
                    Tone::Error
                };
                outcome.icon = if prerequisite {
                    StatusIcon::Warning
                } else {
                    StatusIcon::Error
                };
            }
            Reply::RuntimeReadiness(readiness) => {
                outcome.title = "Browser setup finished".into();
                outcome.message = readiness.message.into();
                if readiness.status != regressee_app::product::RuntimeState::Ready {
                    outcome.title = "Browser setup needs attention".into();
                    outcome.tone = Tone::Warning;
                    outcome.icon = StatusIcon::Warning;
                } else if self.origin == Screen::Setup {
                    outcome.action = "Continue setup";
                    outcome.message.push_str(" Your project settings are kept. Return to setup to retry your task; no website was visited by this update.");
                }
            }
            Reply::Environment(status) => {
                outcome.title = "Access check finished".into();
                outcome.message = status.message.clone();
                if self.spec.kind == Kind::Access && !status.probed {
                    outcome.title = "Website access needs attention".into();
                    outcome.tone = Tone::Warning;
                    outcome.icon = StatusIcon::Warning;
                }
            }
            Reply::Imported(_) => {
                outcome.title = "Project imported".into();
                outcome.message = "The project and selected baseline are saved locally. Set up your own login session before visiting private pages.".into();
                outcome.action = "Open imported project";
                outcome.destination = Some((Screen::Results, ReviewSurface::Changes));
            }
            Reply::Exported => {
                outcome.title = "Baseline ZIP saved".into();
                outcome.message = "The selected baseline ZIP is saved locally.".into();
                outcome.note = Some(("Share with care", "Screenshots may contain private information. Share only with an authorized recipient.".into()));
            }
            Reply::DiagnosticExported => {
                outcome.title = "Support report saved".into();
                outcome.message = "The support report is saved locally. Nothing was sent.".into();
            }
            _ => (),
        }
        if outcome.failed && self.settings_saved_before_task {
            outcome.message = format!("Your project settings were saved. {}", outcome.message);
        }
        outcome
    }
}

fn tone_ink(tone: Tone) -> u32 {
    match tone {
        Tone::Warning => palette().warning,
        Tone::Error => palette().error,
        Tone::Neutral | Tone::Information => palette().ink,
    }
}

fn count_group() -> Div {
    div()
        .debug_selector(|| "task-counts".into())
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_wrap()
        .gap_3()
        .py_2()
        .border_t_1()
        .border_b_1()
        .border_color(rgb(palette().line))
}

fn kind_capture_counts(setup: &CaptureStatus) -> Div {
    let mut group = div()
        .debug_selector(|| "task-kind-coverage".into())
        .flex()
        .flex_col()
        .gap_2();
    for (enabled, label, counts) in [
        (
            setup.capture_plan.mode.screenshots(),
            "Screenshots",
            &setup.screenshots,
        ),
        (
            setup.capture_plan.mode.dom_snapshots(),
            "DOM snapshots",
            &setup.dom_snapshots,
        ),
    ] {
        if enabled {
            group = group.child(div().text_size(px(HELP_TEXT_SIZE)).line_height(px(HELP_LINE_HEIGHT)).child(format!("{label}: {} / {} saved · {} failed · {} skipped or stopped · {} pending · {} baseline omissions", counts.captured, counts.requested, counts.failed, counts.skipped + counts.cancelled, counts.pending, counts.baseline_omitted)));
        }
    }
    group
}

fn capture_counts(counts: &crawl::TargetCounts, running: bool) -> Div {
    count_group()
        .child(controls::labelled_count(
            counts.captured,
            if counts.captured == 1 {
                "screenshot saved"
            } else {
                "screenshots saved"
            },
            StatusIcon::Saved,
            if counts.captured > 0 {
                palette().ink
            } else {
                palette().muted
            },
        ))
        .child(controls::labelled_count(
            counts.failed,
            "failed",
            StatusIcon::Error,
            if counts.failed > 0 {
                palette().error
            } else {
                palette().muted
            },
        ))
        .child(controls::labelled_count(
            counts.skipped,
            "skipped",
            StatusIcon::Skipped,
            if counts.skipped > 0 {
                palette().warning
            } else {
                palette().muted
            },
        ))
        .child(controls::labelled_count(
            counts.pending + counts.cancelled,
            if running { "remaining" } else { "not captured" },
            StatusIcon::Pending,
            if !running && counts.pending + counts.cancelled > 0 {
                palette().warning
            } else {
                palette().muted
            },
        ))
}

fn running_note(task: &Task) -> (&'static str, String) {
    match task.state {
        State::Stopping => ("Stopping safely", format!(
            "Stop requested. Waiting for active work to finish safely. {}", task.spec.reassurance())),
        State::Pausing => ("Pausing safely", "No new batch will start. Active pages finish before the capture can be resumed. Saved screenshots are kept; your baseline is unchanged.".into()),
        _ => {
            let title = match task.spec.kind {
                Kind::Discovery | Kind::Capture | Kind::Check | Kind::Representative => "Your baseline is protected",
                Kind::Setup => "Your saved work is protected",
                Kind::Import => "Existing projects are protected",
                Kind::Export => "Your project is unchanged",
                Kind::Diagnostics => "Nothing is sent",
                Kind::ArchivePreview | Kind::Access | Kind::Storage => "Read-only check",
                Kind::Login => "Your saved session is kept",
                Kind::Baseline | Kind::Save | Kind::Delete => "Confirmed action",
            };
            (title, task.spec.reassurance().into())
        }
    }
}

fn task_context(shell: &Shell, cx: &mut Context<Shell>) -> Div {
    let task = shell.task.as_ref().unwrap();
    div()
        .flex_shrink_0()
        .min_w_0()
        .flex()
        .flex_col()
        .gap_1()
        .child(
            div()
                .font_weight(FontWeight::MEDIUM)
                .child(task.project.clone()),
        )
        .child(
            div()
                .min_w_0()
                .flex()
                .flex_wrap()
                .gap_3()
                .text_size(px(13.))
                .line_height(px(18.))
                .text_color(rgb(palette().muted))
                .when(!task.website.is_empty(), |el| {
                    el.child(
                        div()
                            .w(px(280.))
                            .max_w_full()
                            .flex_shrink_0()
                            .min_w_0()
                            .child(shell.task_link(0, true, cx)),
                    )
                })
                .when(!task.environment.is_empty(), |el| {
                    el.child(format!("Environment: {}", task.environment))
                }),
        )
        .when(
            task.result.as_ref().is_some_and(|result| result.failed),
            |el| {
                el.child(
                    div()
                        .text_size(px(13.))
                        .line_height(px(18.))
                        .text_color(rgb(palette().muted))
                        .child(format!("Task: {}", task.spec.title)),
                )
            },
        )
}

fn detail_heading(title: &'static str) -> Div {
    div()
        .font_weight(FontWeight::SEMIBOLD)
        .text_size(px(18.))
        .line_height(px(24.))
        .child(title)
}

fn detail_row(label: &'static str, value: impl Into<SharedString>) -> Div {
    detail_element(
        label,
        div()
            .min_w_0()
            .flex_1()
            .flex()
            .flex_col()
            .child(value.into()),
    )
}

fn detail_element(label: &'static str, value: impl IntoElement) -> Div {
    div()
        .flex()
        .items_start()
        .gap_3()
        .min_w_0()
        .child(
            div()
                .w(px(120.))
                .flex_shrink_0()
                .text_color(rgb(palette().muted))
                .child(label),
        )
        .child(value)
}

fn capture_setup(setup: &CaptureStatus) -> Div {
    let mut section = div()
        .flex()
        .flex_col()
        .gap_2()
        .child(detail_heading("Capture setup"))
        .child(detail_row("Pages", setup.pages.to_string()))
        .child(detail_row(
            "Excluded URLs",
            format!("{} · not scheduled for capture", setup.excluded_pages),
        ))
        .child(detail_row(
            "What to check",
            match setup.capture_plan.mode {
                CaptureMode::ScreenshotsOnly => "Screenshots only",
                CaptureMode::ScreenshotsAndDomSnapshots => "Screenshots + DOM snapshots",
                CaptureMode::DomSnapshotsOnly => "DOM snapshots only",
            },
        ))
        .child(detail_row(
            "Page and screen-size pairs",
            setup.total.to_string(),
        ))
        .child(detail_row("Environment", setup.environment.clone()));
    for profile in &setup.profiles {
        section = section.child(detail_row("Screen size", profile.clone()));
    }
    if let Some(start) = setup.started_at {
        section = section.child(detail_row(
            "Started",
            workspace_history::history_date(start),
        ));
    }
    if let Some(finish) = setup.finished_at {
        section = section.child(detail_row(
            "Finished",
            workspace_history::history_date(finish),
        ));
    }
    section
}

fn task_details(shell: &Shell, cx: &mut Context<Shell>) -> AnyElement {
    let task = shell.task.as_ref().unwrap();
    let mut details = div()
        .id("task-details")
        .debug_selector(|| "task-details".into())
        .relative()
        .flex_shrink_0()
        .w(px(560.))
        .max_w_full()
        .min_w_0()
        .text_size(px(14.))
        .line_height(px(20.))
        .flex()
        .flex_col()
        .gap_6();
    if let Some(result) = &task.result {
        if let Some(capture) = &result.capture {
            if capture.gap_count > 0 {
                let mut gaps = div()
                    .debug_selector(|| "task-gaps".into())
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(detail_heading("Screenshots not captured"))
                    .child(
                        div()
                            .text_color(rgb(palette().muted))
                            .child("Each row is one page at one screen size. Page links open in your default browser."),
                    );
                for (index, gap) in capture.gaps.iter().enumerate() {
                    let (icon, ink) = if gap.failure.is_some() {
                        (StatusIcon::Error, palette().error)
                    } else if gap.skipped {
                        (StatusIcon::Skipped, palette().warning)
                    } else {
                        (StatusIcon::Pending, palette().warning)
                    };
                    let mut row = div()
                        .min_w_0()
                        .py_3()
                        .border_b_1()
                        .border_color(rgb(palette().line))
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .flex()
                                .items_start()
                                .gap_2()
                                .text_color(rgb(ink))
                                .child(div().flex_shrink_0().child(status_icon(icon, ink, 18.)))
                                .child(
                                    div()
                                        .min_w_0()
                                        .flex_1()
                                        .flex()
                                        .flex_col()
                                        .font_weight(FontWeight::MEDIUM)
                                        .child(gap.reason.clone()),
                                ),
                        )
                        .child(detail_element(
                            "Page",
                            shell.task_link(1 + index * 2, false, cx),
                        ))
                        .child(detail_row("Screen size", gap.profile.clone()))
                        .child(detail_row(
                            "Check kind",
                            if gap.kind == regressee_core::domain::CheckKind::Screenshot {
                                "Screenshot"
                            } else {
                                "DOM snapshot"
                            },
                        ))
                        .child(detail_row("Attempts", gap.attempts.to_string()));
                    if let Some(code) = gap.failure {
                        row = row.child(detail_row("Next step", capture_recovery(code)));
                    }
                    if gap.final_url.is_some() {
                        row = row.child(detail_element(
                            "Redirected to",
                            shell.task_link(2 + index * 2, false, cx),
                        ));
                    }
                    for warning in gap.warnings.iter().take(3) {
                        row = row.child(detail_row(
                            "Warning",
                            capture_warning_label(warning).to_owned(),
                        ));
                    }
                    if gap.warnings.len() > 3 {
                        row = row.child(div().text_color(rgb(palette().muted)).child(
                            "More recorded warnings are available in the review workspace.",
                        ));
                    }
                    gaps = gaps.child(row);
                }
                if capture.gap_count > capture.gaps.len() {
                    gaps = gaps.child(div().text_color(rgb(palette().muted)).child(format!(
                        "Showing {} of {} missing checks. Review captured pages to see the complete list.", capture.gaps.len(), capture.gap_count)));
                }
                details = details.child(gaps);
            }
            details = details.child(capture_setup(&capture.setup));
        } else if let Some(setup) = &task.capture {
            // Frozen setup is useful even if an operation-level failure prevented
            // a terminal evidence reply. Never infer final counts from it.
            details = details.child(capture_setup(setup));
        }
        if !result.notes.is_empty() {
            details = details.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(detail_heading("Discovery details"))
                    .children(
                        result
                            .notes
                            .iter()
                            .map(|note| div().w_full().flex().flex_col().child(note.clone())),
                    ),
            );
        }
        if let Some(diagnostic) = &result.diagnostic {
            details = details.child(
                div()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(detail_heading("Error details"))
                    .child(
                        div()
                            .debug_selector(|| "task-error-details".into())
                            .w_full()
                            .flex()
                            .flex_col()
                            .child(diagnostic.clone()),
                    ),
            );
        }
    } else if let Some(setup) = &task.capture {
        details = details.child(capture_setup(setup));
    }
    details.into_any_element()
}

impl Shell {
    fn task_link(&self, index: usize, ellipsis: bool, cx: &mut Context<Self>) -> AnyElement {
        let task = self.task.as_ref().unwrap();
        let address = task.link_address(index).unwrap_or_default().to_owned();
        if task.link_url(index).is_none() {
            return div()
                .min_w_0()
                .flex_1()
                .flex()
                .flex_col()
                .child(address)
                .into_any_element();
        }
        let control = Control::TaskLink(index);
        let enabled = self.control_enabled(control, cx);
        let bounds = task.link_bounds[index].clone();
        web_link::web_link(
            SharedString::from(format!("TaskLink({index})")),
            address,
            &self.handle(control, cx),
            enabled,
        )
        .flex_1()
        .relative()
        .when(ellipsis, |el| el.text_ellipsis())
        .child(
            canvas(
                |_, _, _| (),
                move |painted, _, _, _| bounds.set(Some(painted)),
            )
            .absolute()
            .size_full(),
        )
        .when(enabled, |el| {
            el.on_click(cx.listener(move |this, event, window, cx| {
                this.click_control(control, event, window, cx);
            }))
        })
        .into_any_element()
    }

    pub(super) fn reveal_task_control(&self, control: Control) {
        let Some(task) = &self.task else { return };
        let bounds = match control {
            Control::TaskDetails => task.details_bounds.get(),
            Control::TaskLink(index) => task.link_bounds.get(index).and_then(|bounds| bounds.get()),
            _ => None,
        };
        let Some(bounds) = bounds else {
            return;
        };
        let viewport = task.scroll.bounds();
        let delta = if bounds.top() < viewport.top() + px(8.) {
            viewport.top() + px(8.) - bounds.top()
        } else if bounds.bottom() > viewport.bottom() - px(8.) {
            viewport.bottom() - px(8.) - bounds.bottom()
        } else {
            px(0.)
        };
        let offset = task.scroll.offset();
        task.scroll
            .set_offset(gpui::point(offset.x, offset.y + delta));
    }

    pub(super) fn visible_controls(&self) -> Vec<Control> {
        if let Some(task) = &self.task {
            return task.controls();
        }
        let mut controls = self.focus.controls();
        if self.focus.screen == Screen::Results && !self.focus.dialog && !self.focus.approval_dialog
        {
            // DOM view selection now belongs to the full-width toolbar, before
            // the queue. Complete-source commands in Details retain their old
            // evidence/inspector reading order instead of moving with it.
            if self.dom_active()
                && !self.viewer_inspector_open()
                && let Some(toolbar) = controls.iter().position(|c| *c == Control::Guided(50))
            {
                let views: Vec<_> = controls
                    .iter()
                    .copied()
                    .filter(|c| matches!(c, Control::DomSource(_)))
                    .collect();
                controls.retain(|c| !matches!(c, Control::DomSource(_)));
                controls.splice(toolbar + 1..toolbar + 1, views);
            }
            // Page previews and unavailable captures put their evidence controls
            // after the queue. Loaded inspection puts the toolbar above both.
            if !self.inspection_chrome_mounted()
                && let (Some(start), Some(end)) = (
                    controls.iter().position(|c| *c == Control::Guided(54)),
                    controls.iter().position(|c| *c == Control::Guided(44)),
                )
            {
                let toolbar: Vec<_> = controls.drain(start..=end).collect();
                let evidence = controls
                    .iter()
                    .position(|c| matches!(c, Control::Viewer | Control::Phase5(10)))
                    .unwrap();
                controls.splice(evidence..evidence, toolbar);
            }
            // Draw lives in Ignored areas; the same action becomes Done in
            // the toolbar during drawing. Move only that mounted destination.
            if self.mask_draw
                && let (Some(draw), Some(toolbar_end)) = (
                    controls.iter().position(|c| *c == Control::Phase5(14)),
                    controls.iter().position(|c| *c == Control::Guided(44)),
                )
            {
                let control = controls.remove(draw);
                controls.insert(toolbar_end + 1, control);
            }
        }
        controls
    }

    pub(super) fn begin_task(&mut self, mut spec: Spec) {
        if spec.kind == Kind::Capture
            && self
                .project
                .as_ref()
                .is_some_and(|p| p.project.capture_plan.mode.dom_snapshots())
        {
            spec.title = "Capturing pages";
        }
        if matches!(
            spec.kind,
            Kind::Discovery
                | Kind::Capture
                | Kind::Check
                | Kind::Representative
                | Kind::Setup
                | Kind::Login
                | Kind::Access
        ) {
            // The discovery instruction is no longer current once capture starts.
            self.notice = None;
        }
        let (project, website, environment) = self.project.as_ref().map_or_else(
            || ("Local workspace".into(), String::new(), String::new()),
            |p| {
                (
                    p.project.name.clone(),
                    p.project
                        .sources
                        .first()
                        .map_or_else(String::new, ToString::to_string),
                    self.environment
                        .as_ref()
                        .map_or_else(String::new, |e| e.name.clone()),
                )
            },
        );
        let scroll = gpui::ScrollHandle::new();
        let task = Task {
            request: self.next_request,
            spec,
            state: State::Running,
            origin: self.focus.screen,
            restore: self.last_action.take().filter(|control| {
                !matches!(
                    control,
                    Control::TaskContinue
                        | Control::TaskDetails
                        | Control::CancelCapture
                        | Control::PauseCapture
                        | Control::TaskLink(_)
                )
            }),
            project,
            project_id: self.project.as_ref().map(|p| p.project.id),
            website,
            environment,
            details: false,
            started: std::time::Instant::now(),
            elapsed: 0,
            result: None,
            settings_saved_before_task: false,
            progress: self.progress.clone(),
            capture: None,
            reveal_details: std::rc::Rc::new(std::cell::Cell::new(false)),
            details_bounds: Default::default(),
            link_bounds: (0..regressee_gui::task::MAX_TASK_LINKS)
                .map(|_| Default::default())
                .collect(),
            scroll,
        };
        self.presentation_focus = task
            .controls()
            .into_iter()
            .find(|control| !matches!(control, Control::TaskLink(_)));
        self.task = Some(task);
    }

    pub(super) fn finish_task(&mut self, request: u64, reply: &Reply) {
        if !self
            .task
            .as_ref()
            .is_some_and(|task| task.request == request)
        {
            return;
        }
        let task = self.task.as_mut().unwrap();
        let result = task.outcome(reply);
        if let Some(capture) = &result.capture {
            task.environment = capture.setup.environment.clone();
        }
        if task.spec.show_result || result.failed || matches!(reply, Reply::Cancelled) {
            task.state = State::Finished;
            task.result = Some(result);
            task.progress = None;
            task.reveal_details.set(false);
            if task
                .result
                .as_ref()
                .and_then(|result| result.diagnostic.as_ref())
                .is_some_and(|error| {
                    error.starts_with(regressee_runtime::setup_diagnostics::FAILURE_PREFIX)
                })
            {
                task.details = true;
            }
            if !task.has_details() {
                task.details = false;
            }
            task.scroll.set_offset(gpui::point(px(0.), px(0.)));
            self.presentation_focus = Some(if task.can_update_browser() {
                Control::SetupRuntime
            } else {
                Control::TaskContinue
            });
        } else {
            let restore = task.restore;
            let origin = task.origin;
            self.task = None;
            // Do not replace authoritative workflow focus (e.g. first save).
            self.presentation_focus = restore.or(Some(Control::Navigate(origin)));
        }
    }

    pub(super) fn request_task_stop(
        &mut self,
        pause: bool,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(task) = &mut self.task else { return };
        let accepted = if pause {
            task.state.request_pause(task.spec)
        } else {
            task.state.request_stop(task.spec)
        };
        if !accepted {
            return;
        }
        if pause {
            self.jobs.pause();
        } else {
            self.jobs.cancel();
        }
        self.presentation_focus = None;
        if self.task.as_ref().unwrap().has_details() {
            window.focus(&self.handle(Control::TaskDetails, cx));
        } else {
            window.focus(&self.root_focus);
        }
        cx.notify();
    }

    pub(super) fn continue_task(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self
            .task
            .as_ref()
            .is_some_and(|task| task.state == State::Finished)
        {
            return;
        }
        let task = self.task.take().unwrap();
        // Continue is an explicit human navigation intent, unlike an external
        // saved-data refresh of the currently inspected run.
        self.external_evidence_refresh = false;
        self.passive_evidence_pending = None;
        let mut restore = task.restore;
        let recovery = task.result.as_ref().and_then(|outcome| outcome.recovery);
        if let Some(destination) =
            recovery.filter(|destination| *destination != Destination::Workspace)
        {
            // The blocking task has already explained the problem. Do not let
            // its banner prevent the destination's read-only status/settings load.
            self.error = None;
            self.notice = None;
            self.focus.screen = match destination {
                Destination::AppSettings => Screen::Settings,
                Destination::ProjectSettings => {
                    self.focus.setup_stage = SetupStage::Settings;
                    Screen::Setup
                }
                Destination::History => {
                    self.history_section = 2;
                    Screen::History
                }
                Destination::Workspace => task.origin,
            };
            self.focus.choosing_project = false;
            self.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
            restore = None;
        } else if let Some((screen, surface)) = task.result.and_then(|outcome| outcome.destination)
        {
            self.focus.screen = screen;
            self.focus.review_surface = surface;
            restore = None;
        } else {
            self.focus.screen = task.origin;
        }
        self.presentation_focus = restore
            .filter(|control| self.control_enabled(*control, cx))
            .or(Some(Control::Navigate(self.focus.screen)));
        window.focus(&self.root_focus);
        self.refresh_visible(cx);
        cx.notify();
    }

    /// Reuse the normal installer only after explicit consent. Keep the blocked
    /// task mounted if submission fails, and carry its setup/focus origin forward.
    pub(super) fn setup_browser(&mut self, cx: &mut Context<Self>) {
        if self.pending.is_some()
            || self
                .task
                .as_ref()
                .is_some_and(|task| !task.can_update_browser())
        {
            return;
        }
        let previous = self.task.take();
        self.error = None;
        self.queue(Work::SetupRuntime, cx);
        if let Some(previous) = previous {
            if let Some(task) = &mut self.task {
                task.origin = previous.origin;
                task.restore = previous.restore;
                task.spec.title = "Updating browser";
            } else {
                self.task = Some(previous);
            }
        }
        self.focus.capturing = self.pending.is_some();
    }

    fn task_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let task = self.task.as_ref().unwrap();
        let title = match task.state {
            State::Running => task.spec.title.to_owned(),
            State::Stopping => format!("{} — stopping…", task.spec.title),
            State::Pausing => "Pausing screenshot capture…".into(),
            State::Finished => task.result.as_ref().unwrap().title.clone(),
        };
        let mut body = div()
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            .gap_6()
            .text_size(px(15.))
            .line_height(px(22.))
            .child(
                div()
                    .w_full()
                    .flex_shrink_0()
                    .flex()
                    .items_start()
                    .gap_3()
                    .debug_selector(|| "task-title".into())
                    .text_size(px(28.))
                    .line_height(px(34.))
                    .when(task.spec.surface == Surface::Dialog, |heading| {
                        heading.text_size(px(21.)).line_height(px(28.))
                    })
                    .font_weight(FontWeight::SEMIBOLD)
                    .when_some(task.result.as_ref(), |el, result| {
                        // The authoritative reply owns the outcome. Reveal its
                        // existing mark once per task; never animate the title,
                        // counts, evidence or the logical completion boundary.
                        el.child(motion::enter(
                            ("task-outcome-mark", task.request),
                            div().flex_shrink_0().pt(px(5.)).child(status_icon(
                                result.icon,
                                tone_ink(result.tone),
                                24.,
                            )),
                            Entrance::Completion,
                            cx,
                        ))
                    })
                    .child(
                        div()
                            .debug_selector(|| "task-title-copy".into())
                            .min_w_0()
                            .flex_1()
                            .flex()
                            .flex_col()
                            .child(title),
                    ),
            )
            .child(task_context(self, cx));
        if let Some(outcome) = &task.result {
            if let Some(capture) = &outcome.capture {
                body = body.child(
                    div()
                        .debug_selector(|| "task-capture-selection".into())
                        .text_color(rgb(palette().muted))
                        .child(capture.setup.selection_summary()),
                );
            }
            if let Some(counts) = &outcome.counts {
                body = body.child(match counts {
                    Counts::Capture(counts) => {
                        if let Some(details) = &outcome.capture
                            && details.setup.capture_plan.mode.dom_snapshots()
                        {
                            kind_capture_counts(&details.setup)
                        } else {
                            capture_counts(counts, false)
                        }
                    }
                    Counts::Discovery {
                        included,
                        excluded,
                        visited,
                    } => count_group()
                        .child(controls::labelled_count(
                            *included,
                            if *included == 1 {
                                "page included"
                            } else {
                                "pages included"
                            },
                            StatusIcon::Pages,
                            palette().ink,
                        ))
                        .child(controls::labelled_count(
                            *excluded,
                            "URLs excluded",
                            StatusIcon::Skipped,
                            palette().muted,
                        ))
                        .child(controls::labelled_count(
                            *visited,
                            if *visited == 1 {
                                "page visited"
                            } else {
                                "pages visited"
                            },
                            StatusIcon::Saved,
                            palette().muted,
                        )),
                });
            }
            body = body.child(
                div()
                    .flex_shrink_0()
                    .debug_selector(|| "task-outcome-message".into())
                    .w(px(560.))
                    .max_w_full()
                    .text_color(rgb(palette().ink))
                    .child(outcome.message.clone()),
            );
            if let Some((title, message)) = &outcome.note {
                body = body.child(controls::note(*title, message.clone(), Tone::Information));
            }
        } else {
            let (message, completed, total) =
                task.progress
                    .clone()
                    .unwrap_or((task.spec.title.into(), 0, None));
            // Counters may be bytes or other task-specific units. Readable
            // amounts belong in the label, not a duplicate unitless counter.
            let mut activity =
                div()
                    .debug_selector(|| "task-activity".into())
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap_3()
                    .child(controls::progress(message, completed, total))
                    .child(
                        div()
                            .debug_selector(|| "task-elapsed".into())
                            .flex()
                            .items_center()
                            .gap_2()
                            .text_size(px(13.))
                            .line_height(px(18.))
                            .text_color(rgb(palette().muted))
                            .child(status_icon(StatusIcon::Clock, palette().muted, 16.))
                            .child(div().debug_selector(|| "task-elapsed-copy".into()).child(
                                format!(
                                    "Elapsed: {}m {:02}s",
                                    task.elapsed / 60,
                                    task.elapsed % 60
                                ),
                            )),
                    );
            if let Some(capture) = &task.capture {
                activity = activity
                    .child(
                        div()
                            .text_size(px(13.))
                            .line_height(px(HELP_LINE_HEIGHT))
                            .text_color(rgb(palette().muted))
                            .child(format!(
                                "{} {} · {} {}",
                                capture.pages,
                                if capture.pages == 1 { "page" } else { "pages" },
                                capture.profiles.len(),
                                if capture.profiles.len() == 1 {
                                    "screen size"
                                } else {
                                    "screen sizes"
                                }
                            )),
                    )
                    .child(if capture.capture_plan.mode.dom_snapshots() {
                        kind_capture_counts(capture)
                    } else {
                        capture_counts(&capture.counts, true)
                    });
            }
            if task.spec.kind == Kind::Discovery
                && task.state == State::Running
                && task
                    .progress
                    .as_ref()
                    .is_some_and(|(_, completed, total)| *completed == 0 && total.is_none())
                && self
                    .project
                    .as_ref()
                    .is_some_and(|p| p.project.discovery.mode == DiscoveryMode::AddressesAndLinks)
            {
                activity = activity.child(div().debug_selector(|| "discovery-startup-help".into())
                    .w(px(560.)).max_w_full().text_size(px(13.)).line_height(px(18.)).text_color(rgb(palette().muted))
                    .child("Pages are counted after each page check finishes. Browser preparation does not add to the count."));
            }
            let (title, explanation) = running_note(task);
            body =
                body.child(activity)
                    .child(controls::note(title, explanation, Tone::Information));
        }
        if task.has_details() {
            let bounds = task.details_bounds.clone();
            let reveal = task.reveal_details.clone();
            let scroll = task.scroll.clone();
            let owner = cx.entity().downgrade();
            let trigger = self
                .disclosure_control(Control::TaskDetails, "Details", task.details, cx)
                .relative()
                .child(
                    canvas(
                        |_, _, _| (),
                        move |painted, _, _, _| {
                            bounds.set(Some(form_recovery::bordered_control_bounds(painted)))
                        },
                    )
                    .absolute()
                    .inset_0()
                    .size_full(),
                );
            body = body.child(
                controls::disclosure_group(
                    "TaskDetails",
                    trigger.into_any_element(),
                    "",
                    task.details.then(|| task_details(self, cx)),
                )
                .relative()
                .child(
                    canvas(
                        move |bounds, window, cx| {
                            // Reveal the owner heading, not just its newly mounted body.
                            if reveal.replace(false) {
                                let offset = scroll.offset();
                                scroll.set_offset(gpui::point(
                                    offset.x,
                                    offset.y + scroll.bounds().top() + px(24.) - bounds.top(),
                                ));
                                let owner = owner.clone();
                                window.defer(cx, move |_, cx| {
                                    if let Some(owner) = owner.upgrade() {
                                        owner.update(cx, |_, cx| cx.notify());
                                    }
                                });
                            }
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .top_0()
                    .left_0()
                    .w_full()
                    .h(px(0.)),
                ),
            );
        }
        let mut actions = div()
            .debug_selector(|| "task-actions".into())
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .min_h(px(40.));
        if let Some(result) = &task.result {
            if result.update_browser {
                actions = actions.child(self.button(
                    Control::SetupRuntime,
                    "Update browser…",
                    ActionStyle::Primary,
                    cx,
                ));
            }
            actions = actions.child(self.button(
                Control::TaskContinue,
                result.action,
                if result.update_browser {
                    ActionStyle::Secondary
                } else {
                    ActionStyle::Primary
                },
                cx,
            ));
        } else if matches!(task.state, State::Running | State::Pausing) {
            if let Some(label) = task.spec.stop_label {
                actions = actions.child(self.button(
                    Control::CancelCapture,
                    label,
                    ActionStyle::Secondary,
                    cx,
                ));
            }
            if task.spec.pausable && task.state == State::Running {
                actions = actions.child(self.button(
                    Control::PauseCapture,
                    "Pause capture",
                    ActionStyle::Secondary,
                    cx,
                ));
            }
        }
        let actions = if task.result.is_some() {
            // Actions and focus are available immediately. Only their paint
            // settles; this does not advance to another screen or save anything.
            motion::enter(
                ("task-outcome-actions", task.request),
                actions,
                Entrance::Completion,
                cx,
            )
        } else {
            actions.into_any_element()
        };
        div()
            .debug_selector(|| "task-panel".into())
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(rgb(palette().surface))
            .child(scrollbar::vertical(
                "task-scrollbar",
                &task.scroll,
                div()
                    .id("task-body")
                    .debug_selector(|| "task-body".into())
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_y_scroll()
                    .track_scroll(&task.scroll)
                    .p_6()
                    .child(body),
            ))
            .child(
                div()
                    .flex_shrink_0()
                    .p_6()
                    .border_t_1()
                    .border_color(rgb(palette().line))
                    .child(actions),
            )
            .into_any_element()
    }

    pub(super) fn task_workspace(&self, cx: &mut Context<Self>) -> AnyElement {
        let task = self.task.as_ref().unwrap();
        div()
            .debug_selector(|| "task-workspace".into())
            .size_full()
            .flex()
            .child(
                div()
                    .w(px(192.))
                    .h_full()
                    .flex_shrink_0()
                    .p_6()
                    .bg(rgb(palette().rail))
                    .text_color(rgb(palette().rail_ink))
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(brand::navigation_lockup())
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .child(task.project.clone()),
                    ),
            )
            .child(
                div()
                    .flex_1()
                    .min_w_0()
                    .h_full()
                    .flex()
                    .flex_col()
                    .p_6()
                    .child(
                        div()
                            .max_w(px(720.))
                            .w_full()
                            .h_full()
                            .min_h_0()
                            .child(self.task_panel(cx)),
                    ),
            )
            .into_any_element()
    }

    pub(super) fn task_dialog(&self, cx: &mut Context<Self>) -> AnyElement {
        let panel = regressee_gui::ui::dialog::surface(600.)
            .debug_selector(|| "task-dialog".into())
            .h(px(480.))
            .overflow_hidden()
            .child(self.task_panel(cx));
        regressee_gui::ui::dialog::present(
            ("task-dialog-entrance", self.task.as_ref().unwrap().request),
            panel,
            cx,
        )
        .debug_selector(|| "task-dialog-overlay".into())
        .into_any_element()
    }

    pub(super) fn task_owns_workspace(&self) -> bool {
        self.task
            .as_ref()
            .is_some_and(|task| task.spec.surface == Surface::Workspace)
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    #[gpui::test]
    fn outcome_motion_keeps_reply_authority_and_immediate_continue(cx: &mut TestAppContext) {
        for reduced in [false, true] {
            for (width, height) in [(1280., 800.), (820., 600.)] {
                for failed in [false, true] {
                    let dir = tempfile::tempdir().unwrap();
                    let paths = AppPaths::isolated(dir.path());
                    regressee_storage::AppPreferences {
                        reduced_motion: reduced,
                        ..Default::default()
                    }
                    .save(&paths)
                    .unwrap();
                    let (jobs, events) = Jobs::start_without_runtime(paths);
                    let owned = jobs.clone();
                    assert_eq!(owned.appearance.wait_loaded().reduced_motion, reduced);
                    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                    cx.simulate_resize(size(px(width), px(height)));
                    cx.update(|window, cx| {
                        shell.update(cx, |s, cx| {
                            drain_initial_read(s, cx);
                            s.next_request = 710;
                            s.pending = Some(710);
                            s.focus.busy = true;
                            s.begin_task(Spec {
                                kind: Kind::Export,
                                surface: Surface::Dialog,
                                title: "Saving baseline ZIP",
                                stop_label: Some("Cancel export"),
                                pausable: false,
                                show_result: true,
                            });
                            s.apply_update(
                                Update {
                                    id: 709,
                                    reply: Reply::Exported,
                                },
                                cx,
                            );
                            assert_eq!(s.task.as_ref().unwrap().state, State::Running);
                            assert!(s.task.as_ref().unwrap().result.is_none());
                            s.apply_update(
                                Update {
                                    id: 710,
                                    reply: if failed {
                                        Reply::ProductFailure("Synthetic export failure.".into())
                                    } else {
                                        Reply::Exported
                                    },
                                },
                                cx,
                            );
                            assert_eq!(s.task.as_ref().unwrap().state, State::Finished);
                            let outcome = s.task.as_ref().unwrap().result.as_ref().unwrap();
                            assert_eq!(outcome.failed, failed);
                            assert_eq!(matches!(outcome.icon, StatusIcon::Error), failed);
                            assert!(s.visible_controls().contains(&Control::TaskContinue));
                            cx.notify();
                        });
                        let _ = window.draw(cx);
                        assert_eq!(cx.global::<controls::ReducedMotion>().0, reduced);
                    });
                    let title = cx.debug_bounds("task-title-copy").unwrap();
                    let actions = cx.debug_bounds("task-actions").unwrap();
                    assert!(title.top() >= px(0.) && title.bottom() <= px(height));
                    assert!(actions.top() >= title.bottom() && actions.bottom() <= px(height));
                    // No timer or animation completion authorizes the next action.
                    cx.update(|window, cx| {
                        shell.update(cx, |s, cx| {
                            s.activate_control(Control::TaskContinue, window, cx);
                            assert!(s.task.is_none());
                            assert_ne!(s.pending, Some(710));
                        });
                        window.remove_window();
                    });
                    owned.join_after_ui();
                }
            }
        }
    }

    #[gpui::test]
    fn every_task_workspace_keeps_the_shared_navigation_brand(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        drop(store);
        for (width, height) in [(1280., 800.), (820., 600.)] {
            for work in [
                Work::Discover(project),
                Work::CaptureInventory {
                    project,
                    preview: None,
                    check: false,
                    resume: None,
                },
                Work::CaptureInventory {
                    project,
                    preview: None,
                    check: true,
                    resume: None,
                },
                Work::SetupRuntime,
            ] {
                let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
                let owned = jobs.clone();
                let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                cx.simulate_resize(size(px(width), px(height)));
                shell.update(cx, |s, cx| {
                    drain_initial_read(s, cx);
                    s.project = Some(record.clone());
                    s.next_request += 1;
                    s.pending = Some(s.next_request);
                    s.focus.busy = true;
                    s.begin_task(Spec::for_work(&work).unwrap());
                    assert!(s.task_owns_workspace());
                    cx.notify();
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let mark = cx.debug_bounds("brand-nav-logo").unwrap();
                let name = cx.debug_bounds("brand-nav-name").unwrap();
                assert!((f32::from(mark.size.width) - 96.).abs() < 0.1);
                assert!((f32::from(mark.size.height) - 96. * 180. / 294.).abs() < 1.);
                assert!((f32::from(mark.center().x) - 96.).abs() < 0.1);
                assert!((f32::from(name.center().x) - 96.).abs() < 0.1);
                assert!((f32::from(name.top() - mark.bottom()) - 8.).abs() < 0.1);
                assert!(mark.top() >= px(24.) && name.bottom() < px(height));
                assert!(name.left() >= px(24.) && name.right() <= px(168.));
                let stop = cx.debug_bounds("action-CancelCapture").unwrap();
                assert!(stop.left() >= px(192.) && stop.bottom() <= px(height));
                owned.join_after_ui();
            }
        }
    }

    #[gpui::test]
    fn browser_download_keeps_readable_amount_bar_and_elapsed_time(cx: &mut TestAppContext) {
        browser_download_progress(cx, false);
    }

    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    #[gpui::test]
    #[ignore = "bounded browser-download offscreen inspection in an explicit private directory"]
    fn renderer_capture_browser_download_progress(cx: &mut TestAppContext) {
        browser_download_progress(cx, true);
    }

    fn browser_download_progress(cx: &mut TestAppContext, render: bool) {
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        let root = render.then(|| {
            cx.enable_offscreen_renderer().unwrap();
            crate::ui_renderer_capture::output_root().unwrap()
        });
        #[cfg(not(all(feature = "ui-renderer-capture", target_os = "linux")))]
        assert!(!render);
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        drop(store);
        for (width, height) in [(1280., 800.), (820., 600.)] {
            for work in [
                Work::SetupRuntime,
                Work::Discover(project),
                Work::OpenLogin(project),
            ] {
                for total in [Some(258_433_266), None] {
                    let (jobs, events) =
                        Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
                    let owned = jobs.clone();
                    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                    cx.simulate_resize(size(px(width), px(height)));
                    shell.update(cx, drain_initial_read);
                    let message = match total {
                        Some(_) => "Downloading private browser · 55.4 MiB of 246.5 MiB received",
                        None => "Downloading private browser · 55.4 MiB received",
                    };
                    cx.update(|_, cx| {
                        shell.update(cx, |s, cx| {
                            s.project = Some(record.clone());
                            s.next_request = u64::MAX;
                            s.pending = Some(u64::MAX);
                            s.focus.busy = true;
                            s.begin_task(Spec::for_work(&work).unwrap());
                            s.apply_update(
                                Update {
                                    id: u64::MAX,
                                    reply: Reply::Progress {
                                        message: message.into(),
                                        completed: 58_080_587,
                                        total,
                                    },
                                },
                                cx,
                            );
                            let task = s.task.as_mut().unwrap();
                            task.elapsed = 4;
                            assert_eq!(task.progress.as_ref().unwrap().0, message);
                            assert_eq!(task.progress.as_ref().unwrap().1, 58_080_587);
                            assert_eq!(task.progress.as_ref().unwrap().2, total);
                            cx.notify();
                        })
                    });
                    cx.update(|window, cx| {
                        let _ = window.draw(cx);
                    });
                    let body = cx.debug_bounds("task-body").unwrap();
                    let elapsed = cx.debug_bounds("task-elapsed-copy").unwrap();
                    assert!(elapsed.top() >= body.top() && elapsed.bottom() <= body.bottom());
                    assert!(elapsed.size.width <= px(180.));
                    let track = cx.debug_bounds("progress-track").unwrap();
                    if let Some(total) = total {
                        let fill = cx.debug_bounds("progress-fill").unwrap();
                        let fraction = f32::from(fill.size.width) / f32::from(track.size.width);
                        assert!((fraction - 58_080_587_f32 / total as f32).abs() < 0.001);
                    } else {
                        assert!(cx.debug_bounds("progress-fill").is_none());
                        assert!(cx.debug_bounds("progress-indeterminate").is_some());
                    }
                    assert!(cx.debug_bounds("action-CancelCapture").is_some());
                    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
                    if let Some(root) = &root
                        && matches!(work, Work::SetupRuntime)
                    {
                        let total_label = if total.is_some() { "known" } else { "unknown" };
                        crate::ui_renderer_capture::capture(
                            cx,
                            root,
                            &format!("{width:.0}x{height:.0}-browser-download-{total_label}"),
                        )
                        .unwrap();
                    }
                    owned.join_after_ui();
                }
            }
        }
    }

    #[gpui::test]
    fn zero_capture_explains_failures_and_reconciles_the_full_url_list(cx: &mut TestAppContext) {
        zero_capture_recovery(cx, false);
    }

    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    #[gpui::test]
    #[ignore = "bounded zero-capture recovery offscreen inspection in an explicit private directory"]
    fn renderer_capture_zero_capture_recovery(cx: &mut TestAppContext) {
        zero_capture_recovery(cx, true);
    }

    fn zero_capture_recovery(cx: &mut TestAppContext, render: bool) {
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        let root = render.then(|| {
            cx.enable_offscreen_renderer().unwrap();
            crate::ui_renderer_capture::output_root().unwrap()
        });
        #[cfg(not(all(feature = "ui-renderer-capture", target_os = "linux")))]
        assert!(!render);
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let mut evidence = store.project_evidence(project).unwrap();
        let r = evidence.latest.as_mut().unwrap();
        r.run.intent = RunIntent::NewBaseline;
        r.run.state = JobState::CompletedWithErrors;
        r.run.profiles.truncate(1);
        let profile = r.run.profiles[0].id;
        r.run.inventory.pages = (0..579)
            .map(|index| regressee_core::domain::InventoryPage {
                id: regressee_core::domain::PageId::new(),
                canonical: format!("https://example.test/page-{index}")
                    .parse()
                    .unwrap(),
                included: index < 5,
                reason: (index >= 5).then(|| "Website not approved".into()),
            })
            .collect();
        r.run.targets = r.run.inventory.pages[..5]
            .iter()
            .map(|page| regressee_core::domain::CaptureTarget {
                dom_outcome: Default::default(),
                key: regressee_core::domain::TargetKey {
                    page: page.id,
                    profile,
                },
                requested: page.canonical.clone(),
                final_url: None,
                attempts: 1,
                outcome: TargetOutcome::Failed {
                    code: regressee_core::domain::CaptureError::RedirectOutsideScope,
                },
                warnings: vec![],
            })
            .collect();
        r.capture = None;
        r.captures.clear();
        evidence.comparison = None;
        evidence.before = None;
        evidence.baseline = None;
        drop(store);
        for (width, height) in [(1280., 800.), (820., 600.)] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    crate::workflow_refinement_tests::load_review(s, project, cx);
                    s.notice = Some("Review included pages and coverage before capturing.".into());
                    s.next_request += 1;
                    s.pending = Some(s.next_request);
                    s.focus.busy = true;
                    s.begin_task(
                        Spec::for_work(&Work::CaptureInventory {
                            project,
                            preview: None,
                            check: false,
                            resume: None,
                        })
                        .unwrap(),
                    );
                    assert!(
                        s.notice.is_none(),
                        "stale discovery instructions must not survive capture"
                    );
                    s.apply_update(
                        Update {
                            id: s.next_request,
                            reply: Reply::Evidence(Box::new(evidence.clone())),
                        },
                        cx,
                    );
                    let result = s.task.as_ref().unwrap().result.as_ref().unwrap();
                    assert_eq!(result.title, "No screenshots captured");
                    assert_eq!(result.action, "Review capture failures");
                    assert!(result.message.contains("Unexpected redirect"));
                    assert!(!result.message.contains("RedirectOutsideScope"));
                    assert!(
                        result
                            .capture
                            .as_ref()
                            .unwrap()
                            .setup
                            .selection_summary()
                            .contains("5 of 579")
                    );
                    assert_eq!(result.capture.as_ref().unwrap().setup.excluded_pages, 574);
                    cx.notify();
                    let _ = window;
                })
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let body = cx.debug_bounds("task-body").unwrap();
            let selection = cx.debug_bounds("task-capture-selection").unwrap();
            assert!(selection.top() >= body.top() && selection.bottom() <= body.bottom());
            #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
            if let Some(root) = &root {
                crate::ui_renderer_capture::capture(
                    cx,
                    root,
                    &format!("{width:.0}x{height:.0}-zero-capture-task"),
                )
                .unwrap();
            }
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.activate_control(Control::TaskContinue, window, cx);
                    assert!(s.task.is_none());
                    assert_eq!(s.focus.review_surface, ReviewSurface::Inspect);
                    assert!(!s.control_enabled(Control::SaveBaseline, cx));
                    assert!(s.selected_capture_failed());
                    assert!(s.notice.is_none());
                })
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            assert!(cx.debug_bounds("unavailable-target-recovery").is_some());
            #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
            if let Some(root) = &root {
                crate::ui_renderer_capture::capture(
                    cx,
                    root,
                    &format!("{width:.0}x{height:.0}-zero-capture-inspect"),
                )
                .unwrap();
            }
            owned.join_after_ui();
        }
    }

    #[gpui::test]
    fn phase6_task_recovery_labels_dispatch_to_implemented_workspace(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let evidence = store.project_evidence(project).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        vcx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                crate::workflow_refinement_tests::load_review(s, project, cx);
                for state in [JobState::Paused, JobState::Interrupted, JobState::Completed] {
                    let mut recorded = evidence.clone();
                    recorded.latest.as_mut().unwrap().run.state = state;
                    s.evidence = recorded.clone();
                    s.begin_task(
                        Spec::for_work(&Work::CaptureInventory {
                            project,
                            preview: None,
                            check: true,
                            resume: None,
                        })
                        .unwrap(),
                    );
                    let task = s.task.as_ref().unwrap();
                    let result = task.outcome(&Reply::Evidence(Box::new(recorded)));
                    assert_eq!(
                        result.action,
                        if state == JobState::Completed {
                            "Review check results"
                        } else {
                            "Return to capture recovery"
                        }
                    );
                    assert_eq!(
                        result.destination,
                        Some((Screen::Results, ReviewSurface::Changes))
                    );
                    s.task.as_mut().unwrap().result = Some(result);
                    s.task.as_mut().unwrap().state = State::Finished;
                    s.continue_task(window, cx);
                    assert_eq!(s.focus.screen, Screen::Results);
                    assert!(s.task.is_none());
                    if state != JobState::Completed {
                        assert!(s.control_mounted(Control::Guided(15)));
                    }
                    crate::workflow_refinement_tests::finish_jobs(s, cx);
                }
            })
        });
        owned.join_after_ui();
    }

    fn drain_initial_read(shell: &mut Shell, cx: &mut Context<Shell>) {
        while shell.pending.is_some() {
            let update = shell.events.recv_timeout(Duration::from_secs(3)).unwrap();
            shell.apply_update(update, cx);
        }
    }

    #[gpui::test]
    fn access_save_error_shows_the_actual_cause_and_keeps_details_and_the_draft(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let saved = store.load_project(project).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                crate::workflow_refinement_tests::load_review(s, project, cx);
                s.focus.screen = Screen::Setup;
                s.focus.setup_stage = SetupStage::Access;
                // Reproduce an invalid selection left by an older app build.
                s.environment_draft = Some(regressee_core::domain::EnvironmentId::new());
                s.name.update(cx, |f, cx| f.set_value("Retained draft", cx));
                s.activate_control(Control::Save, window, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                let task = s.task.as_ref().unwrap();
                let result = task.result.as_ref().unwrap();
                assert_eq!(result.title, "Project settings could not be saved");
                assert!(
                    result
                        .message
                        .contains("selected environment does not belong to this project")
                );
                assert!(result.message.contains("Your draft is kept"));
                assert!(!result.message.contains("storage_write_failed:"));
                assert!(!result.message.contains("disk space"));
                assert!(
                    result
                        .diagnostic
                        .as_ref()
                        .unwrap()
                        .starts_with("storage_write_failed:")
                );
                assert!(task.has_details());
                assert_eq!(s.name.read(cx).content.as_ref(), "Retained draft");
                assert_eq!(s.project.as_ref().unwrap().revision, saved.revision);
                assert_eq!(s.focus.setup_stage, SetupStage::Access);
                s.activate_control(Control::TaskDetails, window, cx);
            })
        });
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            assert!(cx.debug_bounds("task-error-details").is_some());
            let action = cx.debug_bounds("action-TaskContinue").unwrap();
            assert!(action.top() >= px(0.) && action.bottom() <= px(height));
        }
        owned.join_after_ui();
        let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
        assert_eq!(
            store.load_project(project).unwrap().revision,
            saved.revision
        );
    }

    #[gpui::test]
    fn browser_setup_failure_details_open_for_explicit_discovery_and_login_tasks(
        cx: &mut TestAppContext,
    ) {
        use regressee_core::domain::ProjectId;
        use regressee_runtime::{distribution::InstallProgress, setup_diagnostics::SetupAttempt};
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        let mut attempt = SetupAttempt::default();
        attempt.observe(InstallProgress::Downloading {
            bytes: 1024,
            total: Some(2048),
        });
        let message = attempt
            .failure(anyhow::anyhow!("Synthetic download fault"))
            .to_string();
        vcx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                drain_initial_read(s, cx);
                for work in [
                    Work::SetupRuntime,
                    Work::Discover(ProjectId::new()),
                    Work::OpenLogin(ProjectId::new()),
                ] {
                    s.next_request += 1;
                    s.begin_task(Spec::for_work(&work).unwrap());
                    let request = s.next_request;
                    s.finish_task(request, &Reply::ProductFailure(message.clone()));
                    let task = s.task.as_ref().unwrap();
                    assert_eq!(task.state, State::Finished);
                    assert!(task.details && task.has_details());
                    let result = task.result.as_ref().unwrap();
                    assert_eq!(result.title, "Browser setup could not finish");
                    assert!(result.message.contains("failed during download"));
                    assert_eq!(result.diagnostic.as_deref(), Some(message.as_str()));
                    assert!(s.control_enabled(Control::TaskDetails, cx));
                    s.activate_control(Control::TaskDetails, window, cx);
                    assert!(!s.task.as_ref().unwrap().details);
                    s.task = None;
                }
            })
        });
        owned.join_after_ui();
    }

    #[gpui::test]
    fn task_links_support_pointer_keyboard_and_guarded_destinations_without_changing_work(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
        let website = record.project.sources[0].to_string();
        let page = "https://example.test/failing?b=2&a=1#/capture";
        let redirected = "https://example.test/redirected?x=1#/status";
        let mut evidence = store.project_evidence(project).unwrap();
        let run = &mut evidence.latest.as_mut().unwrap().run;
        run.state = JobState::CompletedWithErrors;
        run.targets[0].requested = page.parse().unwrap();
        run.targets[0].final_url = Some(redirected.parse().unwrap());
        run.targets[0].outcome = TargetOutcome::Failed {
            code: regressee_core::domain::CaptureError::NeedsLogin,
        };
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(820.), px(600.)));
        shell.update(cx, |s, cx| {
            drain_initial_read(s, cx);
            s.project = Some(record);
            s.next_request = 800;
            s.pending = Some(800);
            s.focus.busy = true;
            s.begin_task(
                Spec::for_work(&Work::CaptureInventory {
                    project,
                    preview: None,
                    check: false,
                    resume: None,
                })
                .unwrap(),
            );
            assert_eq!(s.presentation_focus, Some(Control::CancelCapture));
            assert!(s.control_enabled(Control::TaskLink(0), cx));
            assert!(!s.control_enabled(Control::TaskLink(1), cx));
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert!(cx.opened_url().is_none());
        let link = cx.debug_bounds("link-TaskLink(0)").unwrap();
        assert!(link.size.height >= px(24.) && link.size.width >= px(200.));
        // Click within the text, not merely the root's left edge/empty padding.
        cx.simulate_click(
            gpui::point(link.left() + px(80.), link.center().y),
            gpui::Modifiers::none(),
        );
        assert_eq!(cx.opened_url().as_deref(), Some(website.as_str()));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert_eq!(s.pending, Some(800));
                assert_eq!(s.next_request, 800);
                assert_eq!(s.task.as_ref().unwrap().state, State::Running);
                assert!(!s.control_enabled(Control::SaveBaseline, cx));
                s.task.as_mut().unwrap().website = "file:///home/private".into();
                assert!(!s.control_enabled(Control::TaskLink(0), cx));
                s.activate_control(Control::TaskLink(0), window, cx);
                s.task.as_mut().unwrap().website = website.clone();
                s.finish_task(800, &Reply::Evidence(Box::new(evidence)));
                s.pending = None;
                s.focus.busy = false;
                assert!(!s.control_enabled(Control::TaskLink(1), cx));
                s.activate_control(Control::TaskLink(1), window, cx);
                s.activate_control(Control::TaskDetails, window, cx);
                assert!(s.control_enabled(Control::TaskLink(1), cx));
                assert!(s.control_enabled(Control::TaskLink(2), cx));
                assert!(
                    !s.control_enabled(Control::TaskLink(regressee_gui::task::MAX_TASK_LINKS), cx)
                );
            })
        });
        assert_eq!(cx.opened_url().as_deref(), Some(website.as_str()));
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                window.focus(&s.handle(Control::TaskLink(0), cx));
                s.move_focus(false, window, cx);
                assert!(s.handle(Control::TaskDetails, cx).is_focused(window));
                s.move_focus(false, window, cx);
                assert!(s.handle(Control::TaskLink(1), cx).is_focused(window));
                s.activate(&Activate, window, cx);
            })
        });
        assert_eq!(cx.opened_url().as_deref(), Some(page));
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let viewport = cx.debug_bounds("task-body").unwrap();
        let link = cx.debug_bounds("link-TaskLink(1)").unwrap();
        assert!(link.top() >= viewport.top() && link.bottom() <= viewport.bottom());
        let final_link = cx.debug_bounds("link-TaskLink(2)").unwrap();
        cx.simulate_click(final_link.center(), gpui::Modifiers::none());
        assert_eq!(cx.opened_url().as_deref(), Some(redirected));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.native_prompt = true;
                s.activate_control(Control::TaskLink(0), window, cx);
                s.native_prompt = false;
                s.activate_control(Control::TaskDetails, window, cx);
                assert!(!s.control_enabled(Control::TaskLink(1), cx));
                assert!(!s.control_enabled(Control::TaskLink(2), cx));
                s.activate_control(Control::TaskLink(1), window, cx);
                assert_eq!(s.next_request, 800);
                assert_eq!(s.task.as_ref().unwrap().state, State::Finished);
            })
        });
        assert_eq!(cx.opened_url().as_deref(), Some(redirected));
        owned.join_after_ui();
        let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
        assert_eq!(
            serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
            before
        );
    }

    #[gpui::test]
    fn details_use_frozen_capture_data_not_repeated_phase_text_or_later_evidence(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, run_id, _, _) = crate::approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let mut evidence = store.project_evidence(project).unwrap();
        let run = &mut evidence.latest.as_mut().unwrap().run;
        assert_eq!(run.id, run_id);
        run.state = JobState::CompletedWithErrors;
        run.targets[0].outcome = TargetOutcome::Failed {
            code: regressee_core::domain::CaptureError::Timeout,
        };
        run.targets[0].attempts = 2;
        let status = CaptureStatus::from_run(run);
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                drain_initial_read(s, cx);
                s.project = Some(record);
                s.next_request = 705;
                s.pending = Some(705);
                s.progress = Some(("Capturing screenshots…".into(), 1, Some(1)));
                s.begin_task(
                    Spec::for_work(&Work::CaptureInventory {
                        project,
                        preview: None,
                        check: false,
                        resume: None,
                    })
                    .unwrap(),
                );
                assert!(!s.visible_controls().contains(&Control::TaskDetails));
                s.activate_control(Control::TaskDetails, window, cx);
                assert!(!s.task.as_ref().unwrap().details);
                let mut wrong = status.clone();
                wrong.project = regressee_core::domain::ProjectId::new();
                s.apply_update(
                    Update {
                        id: 705,
                        reply: Reply::CaptureStatus(wrong),
                    },
                    cx,
                );
                assert!(s.task.as_ref().unwrap().capture.is_none());
                s.apply_update(
                    Update {
                        id: 704,
                        reply: Reply::CaptureStatus(status.clone()),
                    },
                    cx,
                );
                assert!(s.task.as_ref().unwrap().capture.is_none());
                s.apply_update(
                    Update {
                        id: 705,
                        reply: Reply::CaptureStatus(status.clone()),
                    },
                    cx,
                );
                assert!(s.task.as_ref().unwrap().has_details());
                let mut wrong = status.clone();
                wrong.run = regressee_core::domain::RunId::new();
                s.apply_update(
                    Update {
                        id: 705,
                        reply: Reply::CaptureStatus(wrong),
                    },
                    cx,
                );
                assert_eq!(
                    s.task.as_ref().unwrap().capture.as_ref().unwrap().run,
                    run_id
                );
                s.finish_task(705, &Reply::Evidence(Box::new(evidence.clone())));
                let result = s.task.as_ref().unwrap().result.as_ref().unwrap();
                assert!(matches!(result.tone, Tone::Warning));
                assert!(
                    matches!(result.counts, Some(Counts::Capture(ref counts)) if counts.failed == 1)
                );
                assert!(result.diagnostic.is_none());
                let details = result.capture.as_ref().unwrap();
                assert_eq!(details.gaps[0].reason, "Website took too long");
                assert_eq!(details.gaps[0].attempts, 2);
                let frozen = details.setup.profiles.clone();
                s.project.as_mut().unwrap().project.profiles[0].name = "Today's edited size".into();
                evidence.latest.as_mut().unwrap().run.id = regressee_core::domain::RunId::new();
                s.evidence = evidence;
                assert_eq!(
                    s.task
                        .as_ref()
                        .unwrap()
                        .result
                        .as_ref()
                        .unwrap()
                        .capture
                        .as_ref()
                        .unwrap()
                        .setup
                        .profiles,
                    frozen
                );
                assert!(s.task.as_ref().unwrap().progress.is_none());
            })
        });
        owned.join_after_ui();
    }

    #[gpui::test]
    fn task_focus_and_stop_do_not_depend_on_the_underlying_destination(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                drain_initial_read(s, cx);
                s.focus.screen = Screen::Setup;
                s.next_request = 700;
                s.pending = Some(700);
                s.focus.busy = true;
                s.progress = Some(("23 pages visited".into(), 23, None));
                s.begin_task(
                    Spec::for_work(&Work::Discover(regressee_core::domain::ProjectId::new()))
                        .unwrap(),
                );
                assert_eq!(s.visible_controls(), vec![Control::CancelCapture]);
                for destination in Screen::ALL {
                    s.focus.screen = destination;
                    assert!(s.control_enabled(Control::CancelCapture, cx));
                    assert!(!s.control_enabled(Control::Navigate(destination), cx));
                    assert!(!s.control_enabled(Control::Save, cx));
                }
                window.focus(&s.handle(Control::CancelCapture, cx));
                s.move_focus(false, window, cx);
                assert!(s.handle(Control::CancelCapture, cx).is_focused(window));
                s.move_focus(false, window, cx);
                assert!(s.handle(Control::CancelCapture, cx).is_focused(window));
                s.navigate(Screen::Results, window, cx);
                assert_eq!(s.focus.screen, Screen::Settings);
                s.show_keyboard_help(window, cx);
                assert!(!s.focus.dialog);
                s.activate_control(Control::CancelCapture, window, cx);
                assert_eq!(s.task.as_ref().unwrap().state, State::Stopping);
                assert_eq!(s.pending, Some(700));
                assert!(s.focus.busy);
                assert!(!s.control_enabled(Control::CancelCapture, cx));
                s.apply_update(
                    Update {
                        id: 699,
                        reply: Reply::Cancelled,
                    },
                    cx,
                );
                assert_eq!(s.pending, Some(700));
                s.apply_update(
                    Update {
                        id: 700,
                        reply: Reply::Progress {
                            message: "24 pages visited".into(),
                            completed: 24,
                            total: None,
                        },
                    },
                    cx,
                );
                assert_eq!(s.task.as_ref().unwrap().state, State::Stopping);
                s.apply_update(
                    Update {
                        id: 700,
                        reply: Reply::Cancelled,
                    },
                    cx,
                );
                assert_eq!(s.task.as_ref().unwrap().state, State::Finished);
                assert!(s.error.is_none());
                assert!(!s.focus.busy);
                assert_eq!(s.visible_controls(), vec![Control::TaskContinue]);
                s.continue_task(window, cx);
                assert!(s.task.is_none());
                assert_eq!(s.focus.screen, Screen::Setup);
            })
        });
        owned.join_after_ui();
    }

    #[gpui::test]
    fn late_cancel_does_not_relabel_a_committed_export_or_hide_a_real_failure(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            drain_initial_read(s, cx);
            s.next_request = 701;
            s.pending = Some(701);
            s.begin_task(Spec {
                kind: Kind::Export,
                surface: Surface::Dialog,
                title: "Saving baseline ZIP",
                stop_label: Some("Cancel export"),
                pausable: false,
                show_result: true,
            });
            s.task.as_mut().unwrap().state = State::Stopping;
            s.apply_update(
                Update {
                    id: 701,
                    reply: Reply::Exported,
                },
                cx,
            );
            let outcome = s.task.as_ref().unwrap().result.as_ref().unwrap();
            assert_eq!(outcome.title, "Baseline ZIP saved");
            assert!(!outcome.failed);
            s.task = None;
            s.next_request = 702;
            s.pending = Some(702);
            s.begin_task(Spec::for_work(&Work::SetupRuntime).unwrap());
            s.task.as_mut().unwrap().state = State::Stopping;
            s.apply_update(
                Update {
                    id: 702,
                    reply: Reply::ProductFailure("Disk is full".into()),
                },
                cx,
            );
            assert!(s.task.as_ref().unwrap().result.as_ref().unwrap().failed);
            assert_eq!(s.error.as_deref(), Some("Disk is full"));
        });
        owned.join_after_ui();
    }

    #[gpui::test]
    fn committed_actions_have_no_fake_cancel_and_escape_does_not_hide_them(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                drain_initial_read(s, cx);
                s.next_request = 703;
                s.pending = Some(703);
                s.focus.busy = true;
                s.begin_task(Spec::for_work(&Work::SaveNetwork(Default::default())).unwrap());
                assert!(s.visible_controls().is_empty());
                s.dismiss(&Dismiss, window, cx);
                assert_eq!(s.task.as_ref().unwrap().state, State::Running);
                assert_eq!(s.pending, Some(703));
                s.create(&CreateProject, window, cx);
                assert_eq!(s.focus.screen, Screen::Projects);
                let next = s.next_request;
                s.save(&SaveProject, window, cx);
                assert_eq!(s.next_request, next);
                s.apply_update(
                    Update {
                        id: 703,
                        reply: Reply::Network {
                            settings: Default::default(),
                            message: "Saved".into(),
                        },
                    },
                    cx,
                );
                assert!(s.task.is_none());
            })
        });
        owned.join_after_ui();
    }

    #[gpui::test]
    fn incoming_import_has_its_own_identity_and_removed_triggers_get_focus_fallback(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            drain_initial_read(s, cx);
            let project = Project::draft(
                "Previously selected site".into(),
                "https://previous.example.test",
            )
            .unwrap();
            s.project = Some(ProjectRecord {
                project: project.clone(),
                revision: 1,
            });
            let preview = regressee_storage::ImportPreview {
                archive_hash: "synthetic".into(),
                name: "Incoming project".into(),
                targets: 0,
                originals: 0,
                bytes: 0,
                masks: 0,
                warning: "Synthetic identity-only test".into(),
            };
            // The missing ZIP yields a real worker failure, never an import.
            s.queue(
                Work::Import {
                    preview: Box::new(preview),
                    path: dir.path().join("missing.zip"),
                },
                cx,
            );
            let task = s.task.as_ref().unwrap();
            assert_eq!(task.project, "Incoming project");
            assert!(task.website.is_empty());
            assert!(task.environment.is_empty());
            drain_initial_read(s, cx);
            assert!(s.task.as_ref().unwrap().result.as_ref().unwrap().failed);
            s.task = None;
            s.error = None;

            // Presentation-only terminal save: its original Save baseline
            // button is removed after the authoritative project reply.
            s.focus.screen = Screen::Results;
            s.focus.project_saved = true;
            s.focus.candidate = true;
            s.last_action = Some(Control::SaveBaseline);
            s.next_request = 704;
            s.pending = Some(704);
            s.begin_task(
                Spec::for_work(&Work::Activate {
                    run: regressee_core::domain::RunId::new(),
                    revision: 1,
                    confirm_partial: false,
                })
                .unwrap(),
            );
            let mut saved = project;
            saved.active_baseline = Some(regressee_core::domain::BaselineId::new());
            s.apply_update(
                Update {
                    id: 704,
                    reply: Reply::Project(Box::new(ProjectRecord {
                        project: saved,
                        revision: 2,
                    })),
                },
                cx,
            );
            assert!(s.task.is_none());
            assert!(!s.control_mounted(Control::SaveBaseline));
            assert_eq!(
                s.presentation_focus,
                Some(Control::Navigate(Screen::Results))
            );
        });
        owned.join_after_ui();
    }
}
