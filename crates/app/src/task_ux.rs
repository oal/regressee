// SPDX-License-Identifier: MPL-2.0
//! Exclusive tasks own their presentation, focus and stop lifecycle.
use super::*;
use regressee_app::{
    task::{CaptureDetails, CaptureStatus, Kind, Spec, State, Surface, capture_recovery},
    ui::{
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
    pub progress: Option<(String, u32, Option<u32>)>,
    pub capture: Option<CaptureStatus>,
    pub scroll: gpui::ScrollHandle,
    pub reveal_details: std::rc::Rc<std::cell::Cell<bool>>,
    pub link_bounds: Vec<std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>>,
}

pub(super) struct Outcome {
    title: String,
    message: String,
    action: &'static str,
    failed: bool,
    destination: Option<(Screen, ReviewSurface)>,
    diagnostic: Option<String>,
    tone: Tone,
    icon: StatusIcon,
    counts: Option<Counts>,
    capture: Option<CaptureDetails>,
    notes: Vec<String>,
    note: Option<(&'static str, String)>,
}

enum Counts {
    Discovery { included: usize, visited: usize },
    Capture(crawl::TargetCounts),
}

impl Task {
    fn controls(&self) -> Vec<Control> {
        let mut controls = Vec::new();
        for index in 0..regressee_app::task::MAX_TASK_LINKS {
            if self.link_url(index).is_some() {
                controls.push(Control::TaskLink(index));
            }
        }
        if self.state == State::Finished {
            controls.push(Control::TaskContinue);
        } else if matches!(self.state, State::Running | State::Pausing) {
            if self.spec.stop_label.is_some() {
                controls.push(Control::CancelCapture);
            }
            if self.spec.pausable && self.state == State::Running {
                controls.push(Control::PauseCapture);
            }
        }
        if self.has_details() {
            controls.push(Control::TaskDetails);
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
        if (index - 1) % 2 == 0 {
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
                    "Discovery finished within your website boundaries. Review the list before capturing screenshots."
                }.into();
                outcome.counts = Some(Counts::Discovery {
                    included: preview.included(),
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
            Reply::Evidence(evidence) => {
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
                    let counts = crawl::TargetCounts::from_run(&run.run);
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
                        _ => "Review the captured pages and any gaps.",
                    }.into();
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
                        "Saved screenshots are kept.".into(),
                    ));
                    outcome.failed = matches!(
                        run.run.state,
                        JobState::CompletedWithErrors | JobState::Interrupted
                    );
                } else {
                    outcome.title = "Capture results unavailable".into();
                    outcome.message = "The completed run could not be matched to this task. Return to the workspace and refresh its saved evidence.".into();
                    outcome.failed = true;
                    outcome.tone = Tone::Error;
                    outcome.icon = StatusIcon::Error;
                }
                if outcome.capture.is_some() {
                    outcome.action = "Review captured pages";
                    outcome.destination = Some((Screen::Results, ReviewSurface::Inspect));
                    if self.spec.kind == Kind::Check {
                        outcome.action = "Review check results";
                        outcome.destination = Some((Screen::Results, ReviewSurface::Changes));
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
                    Kind::Import => "Import stopped before the project was saved. Complete unreferenced recovery files may remain in local storage.",
                    Kind::Export => "Export stopped before the ZIP was published. Your project stays unchanged.",
                    Kind::ArchivePreview => "ZIP inspection stopped. No project was imported.",
                    Kind::Diagnostics => "Diagnostic preparation or export stopped before publication. Nothing was sent.",
                    Kind::Login => "The browser or session check stopped. Any previously saved session is kept.",
                    Kind::Access => "The website access check stopped. No screenshots or baseline were changed.",
                    _ => "The operation acknowledged cancellation.",
                }.into();
                outcome.action = "Return to workspace";
            }
            Reply::Failure(error) => {
                outcome.title = "Could not finish".into();
                outcome.message = error.message().into();
                outcome.action = "Return to workspace";
                outcome.failed = true;
                outcome.tone = Tone::Error;
                outcome.icon = StatusIcon::Error;
            }
            Reply::ProductFailure(error) => {
                outcome.title = "Could not finish".into();
                outcome.message = "The operation could not finish. Your unsaved edits are kept. Review the details, then return to the workspace to try again. Earlier completed steps may already be saved.".into();
                outcome.action = "Return to workspace";
                outcome.failed = true;
                outcome.tone = Tone::Error;
                outcome.icon = StatusIcon::Error;
                outcome.diagnostic = Some(error.clone());
            }
            Reply::RuntimeReadiness(readiness) => {
                outcome.title = "Browser setup finished".into();
                outcome.message = readiness.message.into();
                if readiness.status != regressee_app::product::RuntimeState::Ready {
                    outcome.title = "Browser setup needs attention".into();
                    outcome.tone = Tone::Warning;
                    outcome.icon = StatusIcon::Warning;
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
                outcome.title = "Diagnostic file saved".into();
                outcome.message = "The diagnostic file is saved locally. Nothing was sent.".into();
            }
            _ => (),
        }
        outcome
    }
}

fn tone_ink(tone: Tone) -> u32 {
    match tone {
        Tone::Warning => WARNING,
        Tone::Error => ERROR,
        Tone::Neutral | Tone::Information => INK,
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
        .border_color(rgb(LINE))
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
            if counts.captured > 0 { INK } else { MUTED },
        ))
        .child(controls::labelled_count(
            counts.failed,
            "failed",
            StatusIcon::Error,
            if counts.failed > 0 { ERROR } else { MUTED },
        ))
        .child(controls::labelled_count(
            counts.skipped,
            "skipped",
            StatusIcon::Skipped,
            if counts.skipped > 0 { WARNING } else { MUTED },
        ))
        .child(controls::labelled_count(
            counts.pending + counts.cancelled,
            if running { "remaining" } else { "not captured" },
            StatusIcon::Pending,
            if !running && counts.pending + counts.cancelled > 0 {
                WARNING
            } else {
                MUTED
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
                Kind::Diagnostics => "Local diagnostics only",
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
                .text_color(rgb(MUTED))
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
                .text_color(rgb(MUTED))
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
            "Screenshots",
            format!("{} targets · one per page and screen size", setup.total),
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
    let reveal = task.reveal_details.clone();
    let scroll = task.scroll.clone();
    let owner = cx.entity().downgrade();
    let mut details = div()
        .id("task-details")
        .debug_selector(|| "task-details".into())
        .relative()
        .flex_shrink_0()
        .w(px(560.))
        .max_w_full()
        .min_w_0()
        .border_t_1()
        .border_color(rgb(LINE))
        .pt_4()
        .text_size(px(14.))
        .line_height(px(20.))
        .flex()
        .flex_col()
        .gap_6()
        // Reveal after the new subtree has layout, including on headless draws
        // without OS frame callbacks. The next frame applies the new offset;
        // ordinary progress updates never reset the user's scroll position.
        .child(
            canvas(
                move |bounds, window, cx| {
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
        );
    if let Some(result) = &task.result {
        if let Some(capture) = &result.capture {
            if capture.gap_count > 0 {
                let mut gaps = div()
                    .debug_selector(|| "task-gaps".into())
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(detail_heading("Capture gaps"))
                    .child(
                        div()
                            .text_color(rgb(MUTED))
                            .child("Each row is one page at one screen size. Page links open in your default browser."),
                    );
                for (index, gap) in capture.gaps.iter().enumerate() {
                    let (icon, ink) = if gap.failure.is_some() {
                        (StatusIcon::Error, ERROR)
                    } else if gap.skipped {
                        (StatusIcon::Skipped, WARNING)
                    } else {
                        (StatusIcon::Pending, WARNING)
                    };
                    let mut row = div()
                        .min_w_0()
                        .py_3()
                        .border_b_1()
                        .border_color(rgb(LINE))
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
                        row = row.child(div().text_color(rgb(MUTED)).child(
                            "More recorded warnings are available in the review workspace.",
                        ));
                    }
                    gaps = gaps.child(row);
                }
                if capture.gap_count > capture.gaps.len() {
                    gaps = gaps.child(div().text_color(rgb(MUTED)).child(format!(
                        "Showing {} of {} gaps. Review captured pages to inspect the complete list.", capture.gaps.len(), capture.gap_count)));
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
                    .child(div().w_full().flex().flex_col().child(diagnostic.clone())),
            );
        }
    } else if let Some(setup) = &task.capture {
        details = details.child(capture_setup(setup));
    }
    details.into_any_element()
}

fn task_scrollbar(scroll: &gpui::ScrollHandle) -> impl IntoElement {
    let scroll = scroll.clone();
    canvas(
        |_, _, _| (),
        move |_, _, window, _| {
            let viewport = scroll.bounds();
            let overflow = f32::from(scroll.max_offset().height);
            let height = f32::from(viewport.size.height) - 8.;
            if overflow <= 0. || height <= 0. {
                return;
            }
            let thumb = (height * height / (height + overflow)).max(24.).min(height);
            let progress = (-f32::from(scroll.offset().y) / overflow).clamp(0., 1.);
            window.paint_quad(gpui::fill(
                Bounds::new(
                    gpui::point(
                        viewport.right() - px(6.),
                        viewport.top() + px(4. + progress * (height - thumb)),
                    ),
                    size(px(4.), px(thumb)),
                ),
                rgb(CONTROL_LINE),
            ));
        },
    )
    .h(px(0.))
    .w_full()
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
            el.on_click(cx.listener(move |this, _, window, cx| {
                this.activate_control(control, window, cx);
            }))
        })
        .into_any_element()
    }

    pub(super) fn reveal_task_control(&self, control: Control) {
        let Control::TaskLink(index) = control else {
            return;
        };
        let Some(task) = &self.task else { return };
        let Some(bounds) = task.link_bounds.get(index).and_then(|bounds| bounds.get()) else {
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
        self.task
            .as_ref()
            .map_or_else(|| self.focus.controls(), Task::controls)
    }

    pub(super) fn begin_task(&mut self, spec: Spec) {
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
            progress: self.progress.clone(),
            capture: None,
            reveal_details: std::rc::Rc::new(std::cell::Cell::new(false)),
            link_bounds: (0..regressee_app::task::MAX_TASK_LINKS)
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
            if !task.has_details() {
                task.details = false;
            }
            task.scroll.set_offset(gpui::point(px(0.), px(0.)));
            self.presentation_focus = Some(Control::TaskContinue);
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
        let mut restore = task.restore;
        if let Some((screen, surface)) = task.result.and_then(|outcome| outcome.destination) {
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
                    .font_weight(FontWeight::SEMIBOLD)
                    .when_some(task.result.as_ref(), |el, result| {
                        el.child(div().flex_shrink_0().pt(px(5.)).child(status_icon(
                            result.icon,
                            tone_ink(result.tone),
                            24.,
                        )))
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
            if let Some(counts) = &outcome.counts {
                body = body.child(match counts {
                    Counts::Capture(counts) => capture_counts(counts, false),
                    Counts::Discovery { included, visited } => count_group()
                        .child(controls::labelled_count(
                            *included,
                            if *included == 1 {
                                "page included"
                            } else {
                                "pages included"
                            },
                            StatusIcon::Pages,
                            INK,
                        ))
                        .child(controls::labelled_count(
                            *visited,
                            if *visited == 1 {
                                "page visited"
                            } else {
                                "pages visited"
                            },
                            StatusIcon::Saved,
                            MUTED,
                        )),
                });
            }
            body = body.child(
                div()
                    .flex_shrink_0()
                    .w(px(560.))
                    .max_w_full()
                    .text_color(rgb(INK))
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
            let mut activity = div()
                .debug_selector(|| "task-activity".into())
                .flex_shrink_0()
                .flex()
                .flex_col()
                .gap_3()
                .child(controls::progress(message, completed, total))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap_3()
                        .text_size(px(13.))
                        .line_height(px(18.))
                        .text_color(rgb(MUTED))
                        .child(
                            div()
                                .flex()
                                .items_center()
                                .gap_2()
                                .child(status_icon(StatusIcon::Clock, MUTED, 16.))
                                .child(format!(
                                    "Elapsed: {}m {:02}s",
                                    task.elapsed / 60,
                                    task.elapsed % 60
                                )),
                        )
                        .when_some(total.filter(|n| *n > 0), |el, total| {
                            el.child(format!("Step progress: {} / {total}", completed.min(total)))
                        }),
                );
            if let Some(capture) = &task.capture {
                activity = activity
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(rgb(MUTED))
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
                    .child(capture_counts(&capture.counts, true));
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
                    .w(px(560.)).max_w_full().text_size(px(13.)).line_height(px(18.)).text_color(rgb(MUTED))
                    .child("Pages are counted after each page check finishes. Browser preparation does not add to the count."));
            }
            let (title, explanation) = running_note(task);
            body =
                body.child(activity)
                    .child(controls::note(title, explanation, Tone::Information));
        }
        if task.details && task.has_details() {
            body = body.child(task_details(self, cx));
        }
        let mut actions = div()
            .debug_selector(|| "task-actions".into())
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .min_h(px(40.));
        if let Some(result) = &task.result {
            actions = actions.child(self.button(
                Control::TaskContinue,
                result.action,
                ActionStyle::Primary,
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
        if task.has_details() {
            actions = actions.child(self.button(
                Control::TaskDetails,
                if task.details {
                    "Hide details"
                } else {
                    "Show details"
                },
                ActionStyle::Secondary,
                cx,
            ));
        }
        div()
            .debug_selector(|| "task-panel".into())
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .bg(rgb(SURFACE))
            .child(
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
            )
            .child(task_scrollbar(&task.scroll))
            .child(
                div()
                    .flex_shrink_0()
                    .p_6()
                    .border_t_1()
                    .border_color(rgb(LINE))
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
                    .bg(rgb(RAIL))
                    .text_color(rgb(RAIL_INK))
                    .flex()
                    .flex_col()
                    .gap_6()
                    .child(
                        div()
                            .text_size(px(22.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child("Regressee"),
                    )
                    .child(
                        div()
                            .font_weight(FontWeight::MEDIUM)
                            .child(task.project.clone()),
                    )
                    .child(
                        div()
                            .text_size(px(13.))
                            .text_color(rgb(RAIL_MUTED))
                            .child("Local task workspace"),
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
        div()
            .debug_selector(|| "task-dialog-overlay".into())
            .occlude()
            .absolute()
            .inset_0()
            .p_4()
            .flex()
            .items_center()
            .justify_center()
            .bg(rgba((INK << 8) | 0x66))
            .child(
                div()
                    .debug_selector(|| "task-dialog".into())
                    .w(px(600.))
                    .h(px(480.))
                    .max_w_full()
                    .max_h_full()
                    .min_h_0()
                    .border_1()
                    .border_color(rgb(CONTROL_LINE))
                    .rounded(px(4.))
                    .overflow_hidden()
                    .child(self.task_panel(cx)),
            )
            .into_any_element()
    }

    pub(super) fn task_owns_workspace(&self) -> bool {
        self.task
            .as_ref()
            .is_some_and(|task| task.spec.surface == Surface::Workspace)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use gpui::TestAppContext;

    fn drain_initial_read(shell: &mut Shell, cx: &mut Context<Shell>) {
        while shell.pending.is_some() {
            let update = shell.events.recv_timeout(Duration::from_secs(3)).unwrap();
            shell.apply_update(update, cx);
        }
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
                    !s.control_enabled(Control::TaskLink(regressee_app::task::MAX_TASK_LINKS), cx)
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
