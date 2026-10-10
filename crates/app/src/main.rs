// SPDX-License-Identifier: MPL-2.0
use gpui::{
    AnyElement, App, Application, Bounds, Context, Div, Entity, FocusHandle, FontWeight,
    KeyBinding, Menu, MenuItem, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PathPromptOptions, Pixels, PromptLevel, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, actions, canvas, div, img, prelude::*, px, relative, rgb, rgba, size,
    uniform_list,
};
use regressee_app::{
    jobs::{Jobs, Reply, Request, Update, Work},
    ui::{
        controls::{self, ActionStyle, Tone},
        text_input::{self, TextInput},
        theme::*,
        timezone_select::{self, TimezoneSelect},
    },
};
use regressee_core::{
    crawl::{self, DiscoveryPreview, TargetCounts},
    domain::{
        CaptureExtent, CaptureProfile, Classification, DiscoveryMode, JobState, Project, RunIntent,
        Speed, TargetKey, TargetOutcome,
    },
    evidence::ProjectEvidence,
    shell::{Choice, Control, Detail, FocusState, Screen},
};
use regressee_storage::{AppPaths, ProjectPage, ProjectRecord, StorageSummary};
use std::{
    sync::{Arc, mpsc::Receiver},
    time::Duration,
};

#[cfg(all(feature = "ui-inspection", target_os = "linux"))]
mod ui_inspection;

#[cfg(all(test, feature = "ui-renderer-capture", target_os = "linux"))]
mod ui_renderer_capture;

fn mask_outline_width(width: f32, height: f32) -> f32 {
    1.0_f32.min(width / 2.).min(height / 2.).max(0.)
}
fn capture_warning_label(code: &str) -> &str {
    match code {
        "ReadinessBudget" => "Page readiness timed out; content may be incomplete",
        "FontsIncomplete" => "Some fonts did not finish loading",
        "ImagesIncomplete" => "Some images did not finish loading",
        "Unstable" => "Page kept changing between screenshot samples",
        "ResourceFailed" => "Some website resources failed to load",
        "ReplacementGeometryChanged" => {
            "Image replacement changed measured element rectangles; source-dependent behavior is unsupported"
        }
        "ReplacementGeometryNotMeasured" => {
            "Element budget exceeded; replacement geometry was not measured"
        }
        "ReplacementSourceDependent" => {
            "Source-attribute CSS or unreadable stylesheets detected; replacement support is not assured"
        }
        "ReplacementUnsupportedMedia" => {
            "Some media/backgrounds cannot be replaced; preview the retained content"
        }
        value if value.starts_with("ResumedAfterLogin") => {
            "Resumed after login; capture times may differ"
        }
        value => value,
    }
}
fn analysis_trigger_label(trigger: &regressee_app::analysis_engine::Trigger) -> &'static str {
    use regressee_app::analysis_engine::Trigger;
    match trigger {
        Trigger::GlobalFraction => "whole-page changed percentage",
        Trigger::LocalConcentration => "concentrated local change",
        Trigger::ConnectedRegion => "connected change region",
        Trigger::HeightChange => "page height changed",
    }
}
fn analysis_gate_label(gate: &regressee_app::analysis_engine::GateReason) -> &'static str {
    use regressee_app::analysis_engine::GateReason;
    match gate {
        GateReason::UnexpectedWidth => "width changed — check capture profile",
        GateReason::DeviceScaleMismatch => "pixel scale differs",
        GateReason::MaterialProfileMismatch => "capture settings/image mode do not match",
        GateReason::AllMasked => "no eligible pixels — all content ignored",
        GateReason::NonOpaquePixels => "transparent pixels unsupported for ordinary comparison",
    }
}
fn analysis_warning_label(warning: &regressee_app::analysis_engine::Warning) -> &'static str {
    use regressee_app::analysis_engine::Warning;
    match warning {
        Warning::LargeIgnoredCoverage => "large ignored area — not whole-page coverage",
        Warning::RenderingEnvironmentMismatch => {
            "computer/fonts/rendering differs — inspection only"
        }
    }
}

actions!(
    shell,
    [
        Quit,
        Tab,
        TabPrev,
        Activate,
        CreateProject,
        SaveProject,
        Dismiss,
        ProjectsScreen,
        SetupScreen,
        ResultsScreen,
        HistoryScreen,
        SettingsScreen
    ]
);
struct Shell {
    root_focus: FocusHandle,
    handles: Vec<FocusHandle>,
    name: Entity<TextInput>,
    address: Entity<TextInput>,
    focus: FocusState,
    jobs: Arc<Jobs>,
    events: Receiver<Update>,
    next_request: u64,
    pending: Option<u64>,
    projects: ProjectPage,
    projects_dirty: bool,
    project: Option<ProjectRecord>,
    summary: Option<StorageSummary>,
    error: Option<String>,
    notice: Option<String>,
    restore_focus: Option<Control>,
    native_prompt: bool,
    evidence: ProjectEvidence,
    evidence_dirty: bool,
    runtime_status: Option<String>,
    runtime_readiness: Option<regressee_app::product::RuntimeReadiness>,
    progress: Option<(String, u32, Option<u32>)>,
    viewer: Option<regressee_app::jobs::ViewerFrame>,
    retired_images: Vec<Arc<gpui::RenderImage>>,
    zoom: f32,
    pan_x: f32,
    pan_y: u32,
    crawl_fields: Vec<Entity<TextInput>>,
    timezone: Entity<TimezoneSelect>,
    draft_settings: Project,
    preview: Option<DiscoveryPreview>,
    search: String,
    result_filter: usize,
    group_profiles: bool,
    selected_page: Option<regressee_core::domain::PageId>,
    selected_key: Option<TargetKey>,
    origins_confirmed: bool,
    approved_origins: std::collections::BTreeSet<String>,
    environment: Option<regressee_app::auth::EnvironmentStatus>,
    environment_draft: Option<regressee_core::domain::EnvironmentId>,
    login_ephemeral: bool,
    show_mapping: bool,
    network_status: Option<String>,
    analysis_policy: regressee_app::analysis_engine::Policy,
    analysis_fields: Vec<Entity<TextInput>>,
    analysis_signature: Vec<String>,
    analysis_due: Option<std::time::Instant>,
    analysis_pending: Option<(u64, Arc<std::sync::atomic::AtomicBool>)>,
    analysis_advanced: bool,
    viewer_mode: usize,
    overlay_fraction: f32,
    region_index: usize,
    mask_rectangles: Vec<regressee_app::analysis_engine::Rectangle>,
    mask_selected: Option<usize>,
    mask_draw: bool,
    mask_drag: Option<((u32, u32), (u32, u32))>,
    mask_dirty: bool,
    pan_drag: Option<((f32, f32), (f32, u32))>,
    approval_selection: std::collections::BTreeSet<TargetKey>,
    approval: Option<regressee_storage::ApprovalPreview>,
    approval_epoch: u64,
    approval_request_epoch: Option<u64>,
    history_index: usize,
    analysis_history_index: usize,
    export_preview: Option<regressee_storage::ExportPreview>,
    import_preview: Option<regressee_storage::ImportPreview>,
    import_path: Option<std::path::PathBuf>,
    deletion_preview: Option<regressee_storage::DeletionPreview>,
    repair_preview: Option<regressee_storage::RepairPreview>,
    history: Option<regressee_storage::HistoryPage>,
    project_storage: Option<regressee_storage::ProjectStorage>,
    run_history_index: usize,
    diagnostic_preview: Option<regressee_app::diagnostics::Report>,
    details: std::collections::BTreeSet<Detail>,
    navigation_started: Option<(Screen, std::time::Instant)>,
    screen_scroll: gpui::ScrollHandle,
}
impl Shell {
    fn new(cx: &mut Context<Self>, jobs: Arc<Jobs>, events: Receiver<Update>) -> Self {
        let mut shell = Self {
            root_focus: cx.focus_handle(),
            handles: (0..248)
                .map(|i| cx.focus_handle().tab_index(i + 1).tab_stop(true))
                .collect(),
            name: cx.new(|cx| TextInput::with_value("", 6, cx)),
            address: cx.new(|cx| {
                let mut input = TextInput::with_value("", 7, cx);
                input.max_bytes = 128 * 1024;
                input
            }),
            focus: FocusState::default(),
            jobs,
            events,
            next_request: 0,
            pending: None,
            projects: ProjectPage {
                items: vec![],
                page: 0,
                more: false,
            },
            projects_dirty: true,
            project: None,
            summary: None,
            error: None,
            notice: None,
            restore_focus: None,
            native_prompt: false,
            evidence: ProjectEvidence::default(),
            evidence_dirty: true,
            runtime_status: None,
            runtime_readiness: None,
            progress: None,
            viewer: None,
            retired_images: Vec::new(),
            zoom: 0.5,
            pan_x: 0.,
            pan_y: 0,
            crawl_fields: [
                "500", "", "", "", "", "2", "2", "1440", "900", "", "", "", "", "", "", "en-US",
                "UTC", "", "",
            ]
            .into_iter()
            .enumerate()
            .map(|(i, s)| cx.new(|cx| TextInput::with_value(s, 100 + i as isize, cx)))
            .collect(),
            timezone: cx.new(|cx| TimezoneSelect::new("UTC", cx)),
            draft_settings: Project::draft("New website".into(), "https://example.test").unwrap(),
            preview: None,
            search: String::new(),
            result_filter: 0,
            group_profiles: false,
            selected_page: None,
            selected_key: None,
            origins_confirmed: false,
            approved_origins: std::collections::BTreeSet::new(),
            environment: None,
            environment_draft: None,
            login_ephemeral: false,
            show_mapping: false,
            network_status: None,
            analysis_policy: regressee_app::review::preset(
                regressee_app::analysis_engine::Preset::Balanced,
            ),
            analysis_fields: ["2", "0.2", "5", "8", "64", "32", "16", "16", "50", ""]
                .into_iter()
                .enumerate()
                .map(|(i, s)| cx.new(|cx| TextInput::with_value(s, 160 + i as isize, cx)))
                .collect(),
            analysis_signature: vec![],
            analysis_due: None,
            analysis_pending: None,
            analysis_advanced: false,
            viewer_mode: 0,
            overlay_fraction: 0.5,
            region_index: 0,
            mask_rectangles: vec![],
            mask_selected: None,
            mask_draw: false,
            mask_drag: None,
            mask_dirty: false,
            pan_drag: None,
            approval_selection: Default::default(),
            approval: None,
            approval_epoch: 0,
            approval_request_epoch: None,
            history_index: 0,
            analysis_history_index: 0,
            export_preview: None,
            import_preview: None,
            import_path: None,
            deletion_preview: None,
            repair_preview: None,
            history: None,
            project_storage: None,
            run_history_index: 0,
            diagnostic_preview: None,
            details: Default::default(),
            navigation_started: None,
            screen_scroll: gpui::ScrollHandle::new(),
        };
        cx.observe(&shell.timezone, |_, _, cx| cx.notify()).detach();
        shell.queue(Work::List { page: 0 }, cx);
        cx.spawn(async move |this, cx| {
            loop {
                cx.background_executor()
                    .timer(Duration::from_millis(50))
                    .await;
                if this
                    .update(cx, |this, cx| {
                        this.poll_storage(cx);
                    })
                    .is_err()
                {
                    break;
                }
            }
        })
        .detach();
        shell
    }
    fn poll_storage(&mut self, cx: &mut Context<Self>) {
        // Bounded batch: at most the update channel capacity per tick.
        for _ in 0..regressee_app::jobs::UPDATE_CAPACITY {
            let Ok(update) = self.events.try_recv() else {
                break;
            };
            self.apply_update(update, cx);
        }
        self.poll_analysis(cx);
    }
    fn apply_update(&mut self, update: Update, cx: &mut Context<Self>) {
        if self
            .analysis_pending
            .as_ref()
            .is_some_and(|(id, _)| *id == update.id)
        {
            let cancelled = self
                .analysis_pending
                .take()
                .unwrap()
                .1
                .load(std::sync::atomic::Ordering::Acquire);
            self.invalidate_approval();
            match update.reply {
                Reply::Analysis(a) => {
                    if !cancelled
                        && !self.mask_dirty
                        && self.analysis_due.is_none()
                        && self
                            .evidence
                            .latest
                            .as_ref()
                            .is_some_and(|r| r.run.id == a.revision.run)
                    {
                        if !self.evidence.analyses.contains(&a.revision.id) {
                            self.evidence.analyses.insert(0, a.revision.id);
                        }
                        self.analysis_history_index = 0;
                        self.evidence.comparison = Some(*a);
                        self.reload_viewer(cx);
                    }
                }
                Reply::ProductFailure(error)
                    if !cancelled && !self.mask_dirty && self.analysis_due.is_none() =>
                {
                    self.error = Some(error)
                }
                _ => (),
            }
            cx.notify();
            return;
        }
        if self.pending != Some(update.id) {
            return;
        }
        if let Reply::Progress {
            message,
            completed,
            total,
        } = update.reply
        {
            self.progress = Some((message, completed, total));
            cx.notify();
            return;
        }
        self.pending = None;
        self.focus.view_loading = false;
        let failed_operation = self.focus.capturing;
        self.focus.busy = false;
        self.focus.capturing = false;
        self.progress = None;
        let imported = matches!(&update.reply, Reply::Imported(_));
        match update.reply {
            Reply::Projects(page) => {
                self.focus.projects = page.items.len();
                self.focus.page = page.page;
                self.focus.more = page.more;
                self.projects = page;
                self.projects_dirty = false;
            }
            Reply::Project(record) | Reply::Imported(record) => {
                self.clear_portability_previews();
                self.history = None;
                self.project_storage = None;
                self.invalidate_approval();
                self.name
                    .update(cx, |input, cx| input.set_value(&record.project.name, cx));
                self.address.update(cx, |input, cx| {
                    input.set_value(
                        &record
                            .project
                            .sources
                            .iter()
                            .map(|u| u.as_str())
                            .collect::<Vec<_>>()
                            .join(" "),
                        cx,
                    )
                });
                self.notice = Some(
                    "Project settings are saved on this computer. Open Results to capture or review this project's evidence."
                        .into(),
                );
                self.draft_settings = record.project.clone();
                self.focus.environments = self.draft_settings.environments.len();
                self.set_crawl_fields(cx);
                self.origins_confirmed = true;
                self.approved_origins = record.project.sources.iter().map(crawl::origin).collect();
                self.preview = None;
                self.project = Some(*record);
                self.environment = None;
                self.show_mapping = false;
                self.focus.project_saved = true;
                self.focus.candidate = false;
                self.focus.baseline = false;
                self.focus.viewer = false;
                self.evidence_dirty = true;
                self.evidence = ProjectEvidence::default();
                self.clear_viewer();
                self.summary = None;
                self.projects_dirty = true;
                if imported {
                    self.notice = Some(regressee_storage::portability_warning().into());
                }
            }
            Reply::DiagnosticPreview(report) => self.diagnostic_preview = Some(*report),
            Reply::DiagnosticExported => {
                self.diagnostic_preview = None;
                self.notice = Some(
                    "Diagnostic file saved locally. Nothing was sent; share it only if you choose."
                        .into(),
                );
            }
            Reply::ExportPreview(preview) => self.export_preview = Some(*preview),
            Reply::ImportPreview(preview) => self.import_preview = Some(*preview),
            Reply::Exported => {
                self.export_preview = None;
                self.notice=Some("Selected baseline ZIP saved. Raw screenshots are not redacted; share it only with an authorized recipient.".into());
            }
            Reply::History { history, storage } => {
                self.evidence.baselines = history.baselines.clone();
                self.history_index = 0;
                self.history = Some(*history);
                self.project_storage = Some(storage);
                self.run_history_index = 0;
            }
            Reply::DeletePreview(preview) => self.deletion_preview = Some(*preview),
            Reply::Deleted => {
                self.clear_portability_previews();
                self.history = None;
                self.project_storage = None;
                self.evidence_dirty = true;
                self.evidence = ProjectEvidence::default();
                self.history_index = 0;
                if let Some(p) = &self.project {
                    self.queue(Work::Load(p.project.id), cx);
                }
                self.notice=Some("Confirmed history deletion completed. Shared and baseline-referenced originals were protected.".into());
            }
            Reply::RepairPreview(preview) => self.repair_preview = Some(*preview),
            Reply::CleanedTemporaries => {
                self.repair_preview = None;
                self.summary = None;
                self.notice=Some("Confirmed cleanup completed. Only the explicitly selected temporary or unreferenced-object category was removed; referenced evidence was protected.".into());
            }
            Reply::Storage(summary) => {
                self.summary = Some(summary);
                if self.runtime_status.is_none() {
                    self.runtime_status = Some("Checking installed browser integrity…".into());
                    self.queue(Work::RuntimeStatus, cx);
                }
            }
            Reply::Failure(error) => self.error = Some(error.message().into()),
            Reply::ProductFailure(error) => {
                self.evidence_dirty = true;
                if failed_operation
                    && self.focus.screen == Screen::Results
                    && let Some(project) = &self.project
                {
                    self.queue(Work::Evidence(project.project.id), cx);
                }
                self.error = Some(error);
            }
            Reply::Runtime(status) => self.runtime_status = Some(status),
            Reply::RuntimeReadiness(status) => {
                self.runtime_status = Some(status.message.into());
                self.runtime_readiness = Some(status);
            }
            Reply::Network { settings, message } => {
                for (i, value) in [
                    (17, settings.proxy.unwrap_or_default()),
                    (
                        18,
                        settings
                            .trusted_ca
                            .map(|p| p.to_string_lossy().into_owned())
                            .unwrap_or_default(),
                    ),
                ] {
                    self.crawl_fields[i].update(cx, |input, cx| input.set_value(&value, cx));
                }
                self.network_status = Some(message);
            }
            Reply::Environment(status) => {
                self.environment_draft = Some(status.environment);
                self.login_ephemeral = status.settings.ephemeral;
                for (i, value) in [
                    (
                        10,
                        status
                            .settings
                            .login_address
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                    ),
                    (
                        11,
                        status
                            .settings
                            .representative
                            .as_ref()
                            .map(ToString::to_string)
                            .unwrap_or_default(),
                    ),
                    (
                        12,
                        status
                            .settings
                            .identity_origins
                            .iter()
                            .map(ToString::to_string)
                            .collect::<Vec<_>>()
                            .join(" "),
                    ),
                ] {
                    self.crawl_fields[i].update(cx, |input, cx| input.set_value(&value, cx));
                }
                self.environment = Some(*status);
            }
            Reply::Evidence(e) => {
                if !e
                    .latest
                    .as_ref()
                    .is_some_and(|r| r.run.intent == RunIntent::RepresentativePreview)
                {
                    self.preview = None;
                    self.focus.preview = false;
                }
                self.mask_dirty = false;
                self.mask_drag = None;
                self.mask_selected = None;
                self.evidence = *e;
                self.invalidate_approval();
                self.approval_selection.clear();
                if let Some(p) = self
                    .evidence
                    .comparison
                    .as_ref()
                    .and_then(|a| a.perceptual.as_ref())
                {
                    self.analysis_policy = p.policy.clone();
                    self.set_analysis_fields(cx);
                }
                self.evidence_dirty = false;
                self.focus.resumable = self.evidence.latest.as_ref().is_some_and(|r| {
                    matches!(r.run.state, JobState::Paused | JobState::Interrupted)
                });
                self.selected_key = self
                    .evidence
                    .latest
                    .as_ref()
                    .and_then(|r| r.run.targets.first())
                    .map(|t| t.key.clone());
                if let Some(index) = self.result_indices().first().copied() {
                    self.selected_key = self
                        .evidence
                        .latest
                        .as_ref()
                        .map(|r| r.run.targets[index].key.clone());
                }
                self.clear_viewer();
                self.pan_y = 0;
                self.pan_x = 0.;
                self.focus.candidate = self.evidence.latest.as_ref().is_some_and(|r| {
                    r.run.state == JobState::Completed
                        && r.run.intent == RunIntent::NewBaseline
                        && !self
                            .evidence
                            .baseline
                            .as_ref()
                            .is_some_and(|b| b.references[0].run == r.run.id)
                });
                self.focus.baseline = self.evidence.baseline.is_some();
                self.focus.viewer = self.evidence.before.is_some()
                    || self
                        .evidence
                        .latest
                        .as_ref()
                        .is_some_and(|r| r.all_captures().next().is_some());
                if self.focus.viewer
                    && self.focus.screen == Screen::Results
                    && let Some(p) = &self.project
                {
                    self.queue(
                        Work::Viewer {
                            analysis: self.evidence.comparison.as_ref().map(|a| a.revision.id),
                            project: p.project.id,
                            tile: 0,
                            key: self.selected_key.clone(),
                        },
                        cx,
                    );
                }
            }
            Reply::Viewer(frame) => {
                let stale = frame.tile != self.pan_y as usize / 512
                    || frame.analysis != self.evidence.comparison.as_ref().map(|a| a.revision.id);
                if !self.mask_dirty {
                    self.mask_rectangles = frame
                        .request
                        .as_ref()
                        .and_then(|r| r.mask.as_ref())
                        .map(|m| m.rectangles.clone())
                        .unwrap_or_default();
                }
                self.clear_viewer();
                self.viewer = Some(*frame);
                if stale {
                    self.reload_viewer(cx);
                }
            }
            Reply::Analysis(a) => {
                self.invalidate_approval();
                if let Some(p) = &a.perceptual {
                    self.analysis_policy = p.policy.clone();
                    self.set_analysis_fields(cx);
                }
                self.evidence.comparison = Some(*a);
                self.reload_viewer(cx);
            }
            Reply::Approval(p) => {
                if self.approval_request_epoch.take() == Some(self.approval_epoch)
                    && self.approval_matches(&p)
                {
                    self.approval = Some(*p);
                }
            }
            Reply::SavedReview(history) => {
                self.evidence.reviews = history;
                self.notice = Some(
                    "Review saved for this analysis revision; older decisions are retained.".into(),
                );
            }
            Reply::SavedMask => {
                self.mask_dirty = false;
                self.schedule_analysis(cx);
                self.notice = Some("Ignored rectangles saved as a new pair-bound revision. Originals are unchanged.".into());
            }
            Reply::Discovery(p) => {
                self.focus.preview = true;
                self.preview = Some(*p);
                self.search.clear();
                self.selected_page = self.preview_page_ids().first().copied();
                self.focus.screen = Screen::Results;
                self.notice=Some("Review included pages and coverage before capturing. Your active baseline is unchanged.".into());
            }
            Reply::Progress { .. } => unreachable!(),
        }
        self.refresh_visible(cx);
        cx.notify();
    }
    fn refresh_visible(&mut self, cx: &mut Context<Self>) {
        if self.pending.is_some() || self.error.is_some() {
            return;
        }
        if self.environment.is_none()
            && matches!(self.focus.screen, Screen::Setup | Screen::Results)
            && let Some(p) = &self.project
        {
            self.queue(Work::EnvironmentStatus(p.project.id), cx);
            return;
        }
        // Navigation during work invalidates the list but never replaces that
        // work. Completion schedules the authoritative refresh only if visible.
        if self.focus.screen == Screen::Projects && self.projects_dirty {
            self.queue(
                Work::List {
                    page: self.projects.page,
                },
                cx,
            );
        }
        if self.focus.screen == Screen::Settings && self.summary.is_none() {
            self.queue(Work::InspectStorage, cx);
        }
        if matches!(self.focus.screen, Screen::Results | Screen::History)
            && self.evidence_dirty
            && let Some(p) = &self.project
        {
            self.queue(Work::Evidence(p.project.id), cx);
        }
        if self.pending.is_none()
            && self.focus.screen == Screen::History
            && self.history.is_none()
            && let Some(p) = &self.project
        {
            self.queue(
                Work::History {
                    project: p.project.id,
                    page: 0,
                },
                cx,
            );
        }
    }
    fn clear_viewer(&mut self) {
        if let Some(frame) = self.viewer.take() {
            self.retired_images
                .extend(frame.panes.into_iter().flatten());
        }
    }
    fn set_analysis_fields(&mut self, cx: &mut Context<Self>) {
        let p = &self.analysis_policy;
        let l = p.local.as_ref();
        let values = vec![
            p.pixel_tolerance_delta_e00.to_string(),
            p.global_changed_fraction
                .map(|v| (v * 100.).to_string())
                .unwrap_or_default(),
            l.map(|v| (v.changed_fraction * 100.).to_string())
                .unwrap_or_default(),
            l.map(|v| v.min_changed.to_string()).unwrap_or_default(),
            l.map(|v| v.min_eligible.to_string()).unwrap_or_default(),
            l.map(|v| v.size.to_string()).unwrap_or_default(),
            l.map(|v| v.stride.to_string()).unwrap_or_default(),
            p.region_min_pixels
                .map(|v| v.to_string())
                .unwrap_or_default(),
            (p.ignored_warning_fraction * 100.).to_string(),
        ];
        for (i, s) in values.iter().enumerate() {
            self.analysis_fields[i].update(cx, |f, cx| f.set_value(s, cx));
        }
        self.analysis_signature = values;
    }
    fn refuse_dirty_mask_analysis(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.mask_dirty {
            return false;
        }
        if let Some((_, cancel)) = &self.analysis_pending {
            cancel.store(true, std::sync::atomic::Ordering::Release);
        }
        self.analysis_due = None;
        self.invalidate_approval();
        let message = "Save ignored areas before reanalyzing. Earlier analysis is still displayed; unsaved rectangles are kept.";
        if self.error.as_deref() != Some(message) {
            self.error = Some(message.into());
            cx.notify();
        }
        true
    }
    fn schedule_analysis(&mut self, cx: &mut Context<Self>) -> bool {
        if self.refuse_dirty_mask_analysis(cx) {
            return false;
        }
        if let Some((_, cancel)) = &self.analysis_pending {
            cancel.store(true, std::sync::atomic::Ordering::Release);
        }
        self.analysis_due = Some(std::time::Instant::now() + Duration::from_millis(300));
        self.invalidate_approval();
        true
    }
    fn invalidate_approval(&mut self) {
        self.approval = None;
        self.approval_epoch += 1;
    }
    fn approval_matches(&self, preview: &regressee_storage::ApprovalPreview) -> bool {
        let c = &preview.context;
        self.analysis_pending.is_none()
            && self.analysis_due.is_none()
            && self.project.as_ref().is_some_and(|p| {
                p.project.id == c.project
                    && p.revision == c.project_revision
                    && p.project.active_baseline == Some(c.baseline)
            })
            && self
                .evidence
                .latest
                .as_ref()
                .is_some_and(|r| r.run.id == c.run)
            && self.evidence.comparison.as_ref().is_some_and(|a| {
                a.revision.id == c.analysis
                    && a.revision.run == c.run
                    && a.revision.baseline == c.baseline
            })
            && c.selected == self.approval_selection.iter().cloned().collect::<Vec<_>>()
            && preview.analysis == c.analysis
            && preview.selected == c.selected
    }
    fn poll_analysis(&mut self, cx: &mut Context<Self>) {
        if self.analysis_advanced {
            let values: Vec<_> = self
                .analysis_fields
                .iter()
                .take(9)
                .map(|f| f.read(cx).content.to_string())
                .collect();
            if values != self.analysis_signature {
                self.invalidate_approval();
                self.analysis_signature = values.clone();
                let parsed = (|| -> anyhow::Result<_> {
                    let mut p = self.analysis_policy.clone();
                    p.pixel_tolerance_delta_e00 = values[0].parse()?;
                    p.global_changed_fraction = if values[1].trim().is_empty() {
                        None
                    } else {
                        Some(values[1].parse::<f64>()? / 100.)
                    };
                    p.local = if values[2].trim().is_empty() {
                        None
                    } else {
                        Some(regressee_app::analysis_engine::LocalCriterion {
                            changed_fraction: values[2].parse::<f64>()? / 100.,
                            min_changed: values[3].parse()?,
                            min_eligible: values[4].parse()?,
                            size: values[5].parse()?,
                            stride: values[6].parse()?,
                        })
                    };
                    p.region_min_pixels = if values[7].trim().is_empty() {
                        None
                    } else {
                        Some(values[7].parse()?)
                    };
                    p.ignored_warning_fraction = values[8].parse::<f64>()? / 100.;
                    regressee_app::review::edited_policy(p)
                })();
                match parsed {
                    Ok(policy) => {
                        self.analysis_policy = policy;
                        if self.schedule_analysis(cx) {
                            self.error = None;
                        }
                    }
                    Err(error) => {
                        if let Some((_, c)) = &self.analysis_pending {
                            c.store(true, std::sync::atomic::Ordering::Release);
                        }
                        self.analysis_due = None;
                        self.error = Some(format!(
                            "Analysis settings need correction: {error}. Earlier analysis is still displayed."
                        ));
                    }
                }
                cx.notify();
            }
        }
        if (self.analysis_due.is_some() || self.analysis_pending.is_some())
            && self.refuse_dirty_mask_analysis(cx)
        {
            return;
        }
        if self
            .analysis_due
            .is_some_and(|t| std::time::Instant::now() >= t)
            && self.analysis_pending.is_none()
            && self.pending.is_none()
        {
            let Some(run) = self
                .evidence
                .latest
                .as_ref()
                .filter(|r| matches!(r.run.intent, RunIntent::Check { .. }))
                .map(|r| r.run.id)
            else {
                self.analysis_due = None;
                return;
            };
            self.next_request += 1;
            let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
            let id = self.next_request;
            if self
                .jobs
                .submit(Request {
                    id,
                    work: Work::Reanalyze {
                        run,
                        policy: self.analysis_policy.clone(),
                        cancel: cancel.clone(),
                    },
                })
                .is_ok()
            {
                self.analysis_pending = Some((id, cancel));
                self.analysis_due = None;
                cx.notify();
            }
        }
    }
    fn reload_viewer(&mut self, cx: &mut Context<Self>) {
        if self.pending.is_none()
            && let Some(p) = &self.project
        {
            self.queue(
                Work::Viewer {
                    project: p.project.id,
                    tile: self.pan_y as usize / 512,
                    key: self.selected_key.clone(),
                    analysis: self.evidence.comparison.as_ref().map(|a| a.revision.id),
                },
                cx,
            );
        }
    }
    fn phase5_action(&mut self, index: usize, cx: &mut Context<Self>) {
        use regressee_app::analysis_engine as e;
        match index {
            0..=2 => {
                self.analysis_policy = regressee_app::review::preset(
                    [
                        e::Preset::Sensitive,
                        e::Preset::Balanced,
                        e::Preset::Relaxed,
                    ][index],
                );
                self.set_analysis_fields(cx);
                self.schedule_analysis(cx);
            }
            3 => self.analysis_advanced = !self.analysis_advanced,
            4 => {
                self.schedule_analysis(cx);
            }
            5 => {
                if let Some((_, c)) = &self.analysis_pending {
                    c.store(true, std::sync::atomic::Ordering::Release);
                }
                self.analysis_due = None;
            }
            6..=9 => self.viewer_mode = index - 6,
            10 => self.overlay_fraction = (self.overlay_fraction - 0.1).max(0.),
            11 => self.overlay_fraction = (self.overlay_fraction + 0.1).min(1.),
            12 | 13 => {
                if let Some(report) = self
                    .evidence
                    .comparison
                    .as_ref()
                    .and_then(|a| a.perceptual.as_ref())
                    .and_then(|p| {
                        p.reports
                            .iter()
                            .find(|(k, _)| Some(k) == self.selected_key.as_ref())
                    })
                    .map(|(_, r)| r)
                    && !report.regions.retained.is_empty()
                {
                    let n = report.regions.retained.len();
                    self.region_index = if index == 12 {
                        (self.region_index + n - 1) % n
                    } else {
                        (self.region_index + 1) % n
                    };
                    let r = &report.regions.retained[self.region_index];
                    self.pan_x = r.bounds.x.saturating_sub(16) as f32;
                    self.pan_y = r.bounds.y.saturating_sub(16);
                    self.reload_viewer(cx);
                }
            }
            14 => {
                self.mask_draw = !self.mask_draw;
                self.mask_selected = None;
            }
            15 => {
                if !self.mask_rectangles.is_empty() {
                    self.mask_selected = Some(
                        (self.mask_selected.map(|i| i + 1).unwrap_or(0))
                            % self.mask_rectangles.len(),
                    );
                }
            }
            16 => {
                if let Some(i) = self
                    .mask_selected
                    .take()
                    .filter(|i| *i < self.mask_rectangles.len())
                {
                    self.mask_rectangles.remove(i);
                    self.mask_dirty = true;
                }
            }
            17 => {
                self.mask_draw = true;
            }
            18 | 19 => {
                if let (Some(project), Some(mut request)) = (
                    &self.project,
                    self.viewer.as_ref().and_then(|v| v.request.clone()),
                ) {
                    request.policy = self.analysis_policy.clone();
                    request.mask = Some(e::MaskBinding {
                        revision: regressee_core::domain::MaskId::new().to_string(),
                        target: request.before.target.clone(),
                        before_capture_id: request.before.capture_id.clone(),
                        after_capture_id: request.after.capture_id.clone(),
                        before_dimensions: request.before.dimensions,
                        after_dimensions: request.after.dimensions,
                        rectangles: self.mask_rectangles.clone(),
                    });
                    match request.validate() {
                        Ok(()) => self.queue(
                            Work::Mask {
                                project: project.project.id,
                                request: Box::new(request),
                            },
                            cx,
                        ),
                        Err(error) => {
                            self.error = Some(format!(
                                "Ignored areas need correction: {error}. Nothing was stretched or saved."
                            ))
                        }
                    }
                }
            }
            20..=22 => {
                if let (Some(a), Some(key)) = (&self.evidence.comparison, &self.selected_key) {
                    self.queue(
                        Work::Review(regressee_core::domain::ReviewDecision {
                            analysis: a.revision.id,
                            key: key.clone(),
                            state: [
                                regressee_core::domain::ReviewState::Unreviewed,
                                regressee_core::domain::ReviewState::Intentional,
                                regressee_core::domain::ReviewState::NeedsFix,
                            ][index - 20],
                            note: self.analysis_fields[9]
                                .read(cx)
                                .content
                                .chars()
                                .take(8192)
                                .collect(),
                            reviewed_at: regressee_runtime::distribution::now(),
                        }),
                        cx,
                    );
                }
            }
            23 => {
                if let Some(key) = &self.selected_key {
                    if !self.approval_selection.remove(key) {
                        self.approval_selection.insert(key.clone());
                    }
                    self.invalidate_approval();
                }
            }
            24 => {
                if self.analysis_pending.is_none()
                    && self.analysis_due.is_none()
                    && let Some(a) = &self.evidence.comparison
                {
                    self.queue(
                        Work::Approval {
                            analysis: a.revision.id,
                            selected: self.approval_selection.iter().cloned().collect(),
                        },
                        cx,
                    );
                }
            }
            25 => {
                if let Some(preview) = &self.approval
                    && self.approval_matches(preview)
                {
                    self.queue(
                        Work::Accept {
                            preview: Box::new(preview.clone()),
                            context: preview.context.clone(),
                        },
                        cx,
                    );
                }
            }
            26 => {
                if !self.evidence.analyses.is_empty() {
                    self.analysis_history_index =
                        (self.analysis_history_index + 1) % self.evidence.analyses.len();
                    self.queue(
                        Work::LoadAnalysis(self.evidence.analyses[self.analysis_history_index]),
                        cx,
                    );
                }
            }
            27 => {
                let mut p = self.analysis_policy.clone();
                p.height_change_triggers = !p.height_change_triggers;
                if let Ok(p) = regressee_app::review::edited_policy(p) {
                    self.analysis_policy = p;
                    self.schedule_analysis(cx);
                }
            }
            28 => {
                let mut p = self.analysis_policy.clone();
                p.antialias = if p.antialias == e::AntiAliasPolicy::Off {
                    e::AntiAliasPolicy::SharedHorizontalBlendV1 {
                        max_delta_e00: 10.,
                        channel_residual: 2,
                    }
                } else {
                    e::AntiAliasPolicy::Off
                };
                match regressee_app::review::edited_policy(p) {
                    Ok(p) => {
                        self.analysis_policy = p;
                        self.schedule_analysis(cx);
                    }
                    Err(error) => self.error = Some(error.to_string()),
                }
            }
            30 => {
                if let (Some(p), Some(preview)) = (&self.project, &self.preview) {
                    self.queue(
                        Work::PreviewImages {
                            project: p.project.id,
                            preview: Box::new(preview.clone()),
                        },
                        cx,
                    );
                    self.focus.capturing = self.pending.is_some();
                }
            }
            40 | 41 => {
                self.export_preview = None;
                self.deletion_preview = None;
                if !self.evidence.baselines.is_empty() {
                    let n = self.evidence.baselines.len();
                    self.history_index = if index == 40 {
                        (self.history_index + n - 1) % n
                    } else {
                        (self.history_index + 1) % n
                    };
                }
            }
            42 => {
                if let (Some(p), Some(b)) = (
                    &self.project,
                    self.evidence.baselines.get(self.history_index),
                ) {
                    self.queue(
                        Work::Restore {
                            baseline: b.id,
                            revision: p.revision,
                        },
                        cx,
                    );
                }
            }
            _ => (),
        }
        cx.notify();
    }
    fn clear_portability_previews(&mut self) {
        self.export_preview = None;
        self.import_preview = None;
        self.import_path = None;
        self.deletion_preview = None;
        self.repair_preview = None;
    }
    // Native picker completions and simulated GPUI tests share these exact dispatch paths.
    fn selected_import_path(&mut self, path: std::path::PathBuf, cx: &mut Context<Self>) {
        self.clear_portability_previews();
        self.import_path = Some(path.clone());
        self.queue(Work::ImportPreview(path), cx);
    }
    fn selected_export_path(
        &mut self,
        path: std::path::PathBuf,
        preview: regressee_storage::ExportPreview,
        cx: &mut Context<Self>,
    ) {
        if self.export_preview.as_ref() != Some(&preview) {
            self.error = Some("Export selection changed; preview again.".into());
            return;
        }
        self.queue(
            Work::Export {
                preview: Box::new(preview),
                path,
            },
            cx,
        );
    }
    fn selected_diagnostic_path(
        &mut self,
        path: std::path::PathBuf,
        report: regressee_app::diagnostics::Report,
        cx: &mut Context<Self>,
    ) {
        if self.diagnostic_preview.as_ref() != Some(&report) {
            self.error =
                Some("Diagnostic preview changed; review it again before exporting.".into());
            return;
        }
        self.queue(
            Work::DiagnosticExport {
                report: Box::new(report),
                path,
            },
            cx,
        );
    }
    fn phase7_action(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        match index {
            0 => {
                self.diagnostic_preview = None;
                self.queue(Work::DiagnosticPreview, cx);
            }
            1 => {
                if let Some(report) = self.diagnostic_preview.clone() {
                    self.native_prompt = true;
                    let directory = std::env::var_os("HOME")
                        .or_else(|| std::env::var_os("USERPROFILE"))
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| std::path::PathBuf::from("."));
                    let prompt =
                        cx.prompt_for_new_path(&directory, Some("regressee-diagnostics.json"));
                    cx.spawn_in(window, async move |this, cx| {
                        let result = prompt.await;
                        let _ =
                            this.update_in(cx, |this, window, cx| {
                                this.native_prompt = false;
                                match result {
                                    Ok(Ok(Some(path))) => {
                                        this.selected_diagnostic_path(path, report, cx)
                                    }
                                    Ok(Ok(None)) => {
                                        this.diagnostic_preview = None;
                                    }
                                    _ => this.error = Some(
                                        "Save dialog unavailable. No diagnostic file was exported."
                                            .into(),
                                    ),
                                }
                                window.focus(&this.handle(Control::Phase7(0), cx));
                                cx.notify();
                            });
                    })
                    .detach();
                }
            }
            2 => {
                self.diagnostic_preview = None;
                window.focus(&self.handle(Control::Phase7(0), cx));
            }
            _ => (),
        }
        cx.notify();
    }
    fn diagnostic_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut panel=controls::section("Private diagnostics", "Preview a small diagnostic report before saving it. No screenshots, website addresses, project names or login secrets are included. Nothing is uploaded.")
            .child(self.button(Control::Phase7(0),"Preview diagnostic contents",ActionStyle::Secondary,cx));
        if let Some(report) = &self.diagnostic_preview {
            panel =
                panel
                    .child(regressee_app::diagnostics::PREVIEW)
                    .child(div().text_size(px(13.)).child(
                        serde_json::to_string_pretty(report).unwrap_or_else(|_| {
                            "Preview unavailable; cancel and try again.".into()
                        }),
                    ))
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_3()
                            .child(self.button(
                                Control::Phase7(1),
                                "I reviewed this — export locally…",
                                ActionStyle::Primary,
                                cx,
                            ))
                            .child(self.button(
                                Control::Phase7(2),
                                "Cancel diagnostic export",
                                ActionStyle::Secondary,
                                cx,
                            )),
                    );
        }
        panel.into_any_element()
    }
    fn phase6_action(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        match index {
            0 => {
                if self.unsaved(cx) {
                    self.error = Some("Save or discard project edits before importing.".into());
                    cx.notify();
                    return;
                }
                self.native_prompt = true;
                let prompt = cx.prompt_for_paths(PathPromptOptions {
                    files: true,
                    directories: false,
                    multiple: false,
                    prompt: Some(
                        "Choose a selected-baseline ZIP to inspect (does not import yet)".into(),
                    ),
                });
                cx.spawn_in(window, async move |this, cx| {
                    let result = prompt.await;
                    let _ = this.update_in(cx, |this, window, cx| {
                        this.native_prompt = false;
                        match result {
                            Ok(Ok(Some(paths))) => {
                                if let Some(path) = paths.into_iter().next() {
                                    this.selected_import_path(path, cx);
                                }
                            }
                            Ok(Ok(None)) => (),
                            _ => this.error = Some(
                                "The native file dialog could not open. No archive was imported."
                                    .into(),
                            ),
                        }
                        window.focus(&this.handle(Control::Navigate(this.focus.screen), cx));
                        cx.notify();
                    });
                })
                .detach();
            }
            1 => {
                if let (Some(preview), Some(path)) = (&self.import_preview, &self.import_path) {
                    self.queue(
                        Work::Import {
                            preview: Box::new(preview.clone()),
                            path: path.clone(),
                        },
                        cx,
                    );
                }
            }
            2 => {
                if let (Some(p), Some(b)) = (
                    &self.project,
                    self.evidence.baselines.get(self.history_index),
                ) {
                    self.deletion_preview = None;
                    self.queue(
                        Work::ExportPreview {
                            project: p.project.id,
                            baseline: b.id,
                        },
                        cx,
                    );
                }
            }
            3 => {
                if let Some(preview) = self.export_preview.clone() {
                    self.native_prompt = true;
                    let directory = std::env::var_os("HOME")
                        .map(std::path::PathBuf::from)
                        .unwrap_or_else(|| std::path::PathBuf::from("."));
                    let prompt = cx.prompt_for_new_path(&directory, Some("baseline.zip"));
                    cx.spawn_in(window, async move |this, cx| {
                        let result = prompt.await;
                        let _ = this.update_in(cx, |this, window, cx| {
                            this.native_prompt = false;
                            match result {
                                Ok(Ok(Some(path))) => this.selected_export_path(path, preview, cx),
                                Ok(Ok(None)) => (),
                                _ => this.error = Some(
                                    "The native save dialog could not open. No ZIP was exported."
                                        .into(),
                                ),
                            }
                            window.focus(&this.handle(Control::Navigate(this.focus.screen), cx));
                            cx.notify();
                        });
                    })
                    .detach();
                }
            }
            4 => {
                if let (Some(p), Some(b)) = (
                    &self.project,
                    self.evidence.baselines.get(self.history_index),
                ) {
                    self.export_preview = None;
                    self.queue(
                        Work::DeletePreview {
                            project: p.project.id,
                            item: regressee_storage::DeleteItem::Baseline(b.id),
                        },
                        cx,
                    );
                }
            }
            5 => {
                if let (Some(p), Some(r)) = (
                    &self.project,
                    self.history
                        .as_ref()
                        .and_then(|h| h.runs.get(self.run_history_index)),
                ) {
                    self.export_preview = None;
                    self.queue(
                        Work::DeletePreview {
                            project: p.project.id,
                            item: regressee_storage::DeleteItem::Run(r.run.id),
                        },
                        cx,
                    );
                }
            }
            6 => {
                if let Some(preview) = self.deletion_preview.clone() {
                    self.queue(Work::Delete(Box::new(preview)), cx);
                }
            }
            7 => {
                self.clear_portability_previews();
            }
            8 => {
                if let Some(p) = &self.project {
                    self.queue(
                        Work::History {
                            project: p.project.id,
                            page: self.history.as_ref().map(|h| h.page).unwrap_or(0),
                        },
                        cx,
                    );
                }
            }
            9 | 10 => {
                self.deletion_preview = None;
                if let Some(h) = &self.history
                    && !h.runs.is_empty()
                {
                    self.run_history_index = if index == 9 {
                        (self.run_history_index + 1) % h.runs.len()
                    } else {
                        (self.run_history_index + h.runs.len() - 1) % h.runs.len()
                    };
                }
            }
            11 | 12 => {
                self.deletion_preview = None;
                if let (Some(p), Some(h)) = (&self.project, &self.history) {
                    let page = if index == 11 {
                        if !h.more {
                            return;
                        }
                        h.page + 1
                    } else {
                        h.page.saturating_sub(1)
                    };
                    self.queue(
                        Work::History {
                            project: p.project.id,
                            page,
                        },
                        cx,
                    );
                }
            }
            13 => self.queue(Work::RepairPreview, cx),
            14 => {
                if let Some(preview) = self.repair_preview.clone() {
                    self.queue(Work::CleanupTemporaries(Box::new(preview)), cx);
                }
            }
            15 => {
                if let Some(preview) = self.repair_preview.clone() {
                    self.queue(Work::DeleteUnreferenced(Box::new(preview)), cx);
                }
            }
            _ => (),
        }
        cx.notify();
    }
    fn portability_confirmation(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div().flex().flex_col().gap_3().max_w(px(680.));
        if let Some(p) = &self.export_preview {
            body=body.child(div().font_weight(FontWeight::SEMIBOLD).child("Export contents — review before sharing"))
                .child(format!("{} targets · {} original PNGs · {} bytes · {} source capture dates · {} ignored-region configurations. Selected/inherited originals only; no unrelated history or database.",p.targets,p.originals,p.bytes,p.sources,p.masks))
                .child(div().text_color(rgb(WARNING)).child(p.privacy.clone()))
                .child(self.button(Control::Phase6(3),"I understand — export ZIP…",ActionStyle::Primary,cx));
        }
        if let Some(p) = &self.import_preview {
            body=body.child(div().font_weight(FontWeight::SEMIBOLD).child(format!("Import “{}” as a NEW local project",p.name)))
                .child(format!("Validated {} targets · {} original PNGs · {} bytes · {} ignored-region configurations. Name collisions get a separate safe name. Masks require revalidation for your capture pair.",p.targets,p.originals,p.bytes,p.masks))
                .child(div().text_color(rgb(WARNING)).child(p.warning.clone()))
                .child(self.button(Control::Phase6(1),"Confirm import — no website visits",ActionStyle::Primary,cx));
        }
        if let Some(p) = &self.deletion_preview {
            body=body.child(div().font_weight(FontWeight::SEMIBOLD).child("Delete retained history?"))
                .child(format!("{:?}: {} analyses and {} unshared objects removed; {} bytes deletable. {} shared bytes remain. This cannot be undone; active and retained baseline references are protected.",p.item,p.analyses,p.objects,p.bytes,p.shared_bytes))
                .child(self.button(Control::Phase6(6),"Confirm permanent deletion",ActionStyle::Danger,cx));
        }
        if let Some(p) = &self.repair_preview {
            body=body.child(div().font_weight(FontWeight::SEMIBOLD).child("Interrupted writes and integrity"))
                .child(format!("{} temporary files and {} dead-owner interrupted imports ({} bytes) may be discarded. {} valid complete unreferenced objects are retained, NOT treated as failed captures. {} missing/corrupt objects require recovery; they cannot establish a valid active baseline.",p.temporaries.len(),p.interrupted_imports.len(),p.temporary_bytes,p.valid_unreferenced.len(),p.missing_or_corrupt.len()))
                .child(self.button(Control::Phase6(14),"Confirm temporary-only cleanup",ActionStyle::Secondary,cx))
                .child(div().text_color(rgb(WARNING)).child("Complete unreferenced objects can be valid recovery originals, not junk. Retain them unless you explicitly want to delete this recovery evidence. This separate action does NOT delete retained/shared/active-baseline evidence."))
                .child(self.button(Control::Phase6(15),"Confirm deletion of complete unreferenced recovery objects",ActionStyle::Secondary,cx));
        }
        if self.export_preview.is_some()
            || self.import_preview.is_some()
            || self.deletion_preview.is_some()
            || self.repair_preview.is_some()
        {
            body = body.child(self.button(
                Control::Phase6(7),
                "Cancel preview",
                ActionStyle::Secondary,
                cx,
            ));
        }
        body.into_any_element()
    }
    fn viewer_dimensions(&self) -> (u32, u32) {
        if let Some(viewer) = &self.viewer {
            return (viewer.width, viewer.height);
        }
        let mut dimensions = (0, 0);
        for image in [
            self.evidence.before.as_ref().map(|c| &c.image),
            self.evidence
                .latest
                .as_ref()
                .and_then(|r| r.capture.as_ref())
                .map(|c| &c.image),
            self.evidence
                .comparison
                .as_ref()
                .and_then(|a| a.heatmap.as_ref()),
        ]
        .into_iter()
        .flatten()
        {
            dimensions.0 = dimensions.0.max(image.width);
            dimensions.1 = dimensions.1.max(image.height);
        }
        dimensions
    }
    fn handle(&self, control: Control, cx: &App) -> FocusHandle {
        let index = match control {
            Control::Navigate(screen) => Screen::ALL.iter().position(|s| *s == screen).unwrap(),
            Control::NewProject => 5,
            Control::Name => return self.name.read(cx).focus.clone(),
            Control::Address => return self.address.read(cx).focus.clone(),
            Control::Save => 6,
            Control::ChooseFolder => 7,
            Control::About => 8,
            Control::CloseDialog => 9,
            Control::PreviousPage => 10,
            Control::NextPage => 11,
            Control::OpenProject(index) => 12 + index,
            Control::CaptureBaseline => 76,
            Control::SaveBaseline => 77,
            Control::CheckChanges => 78,
            Control::CancelCapture => 79,
            Control::SetupRuntime => 80,
            Control::RefreshEvidence => 81,
            Control::ZoomIn => 82,
            Control::ZoomOut => 83,
            Control::PanLeft => 84,
            Control::PanRight => 85,
            Control::PanUp => 86,
            Control::PanDown => 87,
            Control::PauseCapture => 88,
            Control::Guided(index) => 90 + index,
            Control::CrawlField(16) => return self.timezone.read(cx).focus_handle(cx),
            Control::CrawlField(index) => return self.crawl_fields[index].read(cx).focus.clone(),
            Control::Phase5(index) => 140 + index,
            Control::AnalysisField(index) => {
                return self.analysis_fields[index].read(cx).focus.clone();
            }
            Control::Phase6(index) => 184 + index,
            Control::Phase7(index) => 200 + index,
            Control::Choice(choice) => 204 + choice.index(),
            Control::Environment(index) => 224 + index,
            Control::Details(detail) => 244 + detail as usize,
            Control::DismissFeedback => 243,
        };
        self.handles[index].clone()
    }
    fn focused(&self, window: &Window, cx: &App) -> Option<Control> {
        self.focus
            .controls()
            .into_iter()
            .find(|c| self.control_enabled(*c, cx) && self.handle(*c, cx).is_focused(window))
    }
    fn move_focus(&mut self, reverse: bool, window: &mut Window, cx: &mut Context<Self>) {
        let controls: Vec<_> = self
            .focus
            .controls()
            .into_iter()
            .filter(|control| self.control_enabled(*control, cx))
            .collect();
        if controls.is_empty() {
            return;
        }
        let current = self.focused(window, cx).unwrap_or(if reverse {
            *controls.first().unwrap()
        } else {
            *controls.last().unwrap()
        });
        let index = controls
            .iter()
            .position(|control| *control == current)
            .unwrap_or(0);
        let next = if reverse {
            (index + controls.len() - 1) % controls.len()
        } else {
            (index + 1) % controls.len()
        };
        window.focus(&self.handle(controls[next], cx));
    }
    fn queue(&mut self, work: Work, cx: &mut Context<Self>) {
        if self.pending.is_some() {
            return;
        }
        if matches!(
            &work,
            Work::Load(_)
                | Work::Save { .. }
                | Work::SaveReplacement { .. }
                | Work::Evidence(_)
                | Work::CaptureInventory { .. }
                | Work::Capture { .. }
                | Work::Activate { .. }
                | Work::Accept { .. }
                | Work::Restore { .. }
                | Work::LoadAnalysis(_)
                | Work::Mask { .. }
                | Work::SaveEnvironment { .. }
        ) {
            self.invalidate_approval();
            self.clear_portability_previews();
            self.history = None;
            self.project_storage = None;
        }
        if matches!(&work, Work::LoadAnalysis(_)) {
            if let Some((_, cancel)) = self.analysis_pending.take() {
                cancel.store(true, std::sync::atomic::Ordering::Release);
            }
            self.analysis_due = None;
        }
        if matches!(&work, Work::Approval { .. }) {
            self.approval_request_epoch = Some(self.approval_epoch);
        }
        if matches!(
            &work,
            Work::Load(_)
                | Work::Save { .. }
                | Work::SaveReplacement { .. }
                | Work::CaptureInventory { .. }
                | Work::Capture { .. }
                | Work::Activate { .. }
                | Work::Accept { .. }
                | Work::Restore { .. }
                | Work::PreviewImages { .. }
        ) {
            if let Some((_, cancel)) = self.analysis_pending.take() {
                cancel.store(true, std::sync::atomic::Ordering::Release);
            }
            self.analysis_due = None;
        }
        self.next_request += 1;
        self.focus.view_loading = matches!(&work, Work::Viewer { .. });
        self.focus.pausable = matches!(&work, Work::CaptureInventory { .. });
        match self.jobs.submit(Request {
            id: self.next_request,
            work,
        }) {
            Ok(()) => {
                self.pending = Some(self.next_request);
                self.focus.busy = true;
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error.into());
                self.focus.busy = false;
            }
        }
        cx.notify();
    }
    fn navigate(&mut self, screen: Screen, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus.dialog || self.native_prompt {
            return;
        }
        if self.focus.screen != screen && regressee_app::ui::performance::enabled() {
            self.navigation_started = Some((screen, std::time::Instant::now()));
        }
        if self.focus.screen != screen {
            self.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
        }
        self.focus.screen = screen;
        self.error = None;
        window.focus(&self.handle(Control::Navigate(screen), cx));
        if screen == Screen::Projects {
            self.projects_dirty = true;
        }
        if screen == Screen::Settings {
            self.summary = None;
            self.runtime_status = None;
            self.runtime_readiness = None;
        }
        self.refresh_visible(cx);
        cx.notify();
    }
    fn create(&mut self, _: &CreateProject, window: &mut Window, cx: &mut Context<Self>) {
        if self.focus.busy || self.focus.dialog || self.native_prompt {
            return;
        }
        if self.project.is_some() && self.unsaved(cx) {
            self.confirm_switch(Control::NewProject, window, cx);
            return;
        }
        // Preserve an existing unsaved form when visiting it again. Opening a
        // saved project is explicit; no automatic draft replacement on navigation.
        if self.project.is_some() || self.name.read(cx).content.is_empty() {
            self.project = None;
            self.focus.project_saved = false;
            self.focus.baseline = false;
            self.focus.candidate = false;
            self.focus.viewer = false;
            self.evidence = ProjectEvidence::default();
            self.clear_viewer();
            self.evidence_dirty = true;
            self.name.update(cx, |input, cx| input.set_value("", cx));
            self.address.update(cx, |input, cx| input.set_value("", cx));
            self.draft_settings =
                Project::draft("New website".into(), "https://example.test").unwrap();
            self.set_crawl_fields(cx);
            self.preview = None;
            self.origins_confirmed = false;
            self.approved_origins.clear();
            self.focus.environments = 1;
        }
        self.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
        self.focus.screen = Screen::Setup;
        self.error = None;
        self.notice = None;
        window.focus(&self.handle(Control::Name, cx));
        cx.notify();
    }
    fn save(&mut self, _: &SaveProject, _: &mut Window, cx: &mut Context<Self>) {
        self.save_settings(cx);
    }
    fn local_login_settings(
        &self,
        cx: &App,
    ) -> anyhow::Result<regressee_storage::LocalLoginSettings> {
        let text = |i: usize| self.crawl_fields[i].read(cx).content.trim().to_owned();
        let url = |i| -> anyhow::Result<Option<url::Url>> {
            let value = text(i);
            if value.is_empty() {
                return Ok(None);
            }
            let value = url::Url::parse(&value)?;
            regressee_core::domain::validate_address(&value)?;
            Ok(Some(value))
        };
        let settings = regressee_storage::LocalLoginSettings {
            login_address: url(10)?,
            representative: url(11)?,
            identity_origins: if text(12).is_empty() {
                vec![]
            } else {
                crawl::sources(&text(12))?
            },
            ephemeral: self.login_ephemeral,
        };
        settings.validate()?;
        Ok(settings)
    }
    fn save_settings(&mut self, cx: &mut Context<Self>) {
        if !self.focus.can_activate(Control::Save) || self.native_prompt {
            return;
        }
        let name = self.name.read(cx).content.trim().to_owned();
        let address = self.address.read(cx).content.trim().to_owned();
        let result = (|| -> anyhow::Result<Project> {
            let sources = crawl::sources(&address)?;
            let origins = sources
                .iter()
                .map(crawl::origin)
                .collect::<std::collections::BTreeSet<_>>();
            anyhow::ensure!(
                origins.is_subset(&self.approved_origins) || origins.len() == 1,
                "Confirm every listed website before saving multiple origins"
            );
            let mut draft = Project::draft(name.clone(), sources[0].as_str())?;
            draft.sources = sources;
            draft.discovery = self.draft_settings.discovery.clone();
            draft.profiles = self.draft_settings.profiles.clone();
            let text = |i: usize| self.crawl_fields[i].read(cx).content.trim().to_owned();
            if self.project.is_some() {
                for profile in &mut draft.profiles {
                    profile.locale = text(15);
                    profile.timezone = self.timezone.read(cx).selected.to_string();
                }
                self.local_login_settings(cx)?;
            }
            draft.discovery.page_limit = text(0).parse()?;
            draft.discovery.section = if text(1).is_empty() {
                None
            } else {
                Some(text(1))
            };
            draft.discovery.skipped_text = text(2)
                .split(',')
                .map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_owned)
                .collect();
            draft.discovery.advanced.max_depth = if text(3).is_empty() {
                None
            } else {
                Some(text(3).parse()?)
            };
            draft.discovery.advanced.concurrency = if text(4).is_empty() {
                None
            } else {
                Some(text(4).parse()?)
            };
            draft.discovery.advanced.per_origin = text(5).parse()?;
            draft.discovery.advanced.retries = text(6).parse()?;
            let mut origins = std::collections::BTreeSet::new();
            draft.environments[0].mappings = draft
                .sources
                .iter()
                .filter(|u| origins.insert(crawl::origin(u)))
                .map(|u| {
                    let mut root = u.clone();
                    root.set_path("/");
                    root.set_query(None);
                    root.set_fragment(None);
                    regressee_core::domain::OriginMapping {
                        source: root.clone(),
                        destination: root,
                    }
                })
                .collect();
            draft.validate()?;
            Ok(draft)
        })();
        match result {
            Err(e) => {
                self.error = Some(format!(
                    "Check your website addresses and capture settings: {e}"
                ));
                cx.notify();
            }
            Ok(draft) => {
                let (project, expected_revision) = if let Some(record) = &self.project {
                    let mut project = record.project.clone();
                    project.name = name;
                    project.sources = draft.sources.clone();
                    project.discovery = draft.discovery.clone();
                    project.profiles = draft.profiles.clone();
                    project.environments = self.draft_settings.environments.clone();
                    // The initial default environment follows an edited source.
                    // Custom mappings and additional sources are never discarded.
                    let old_defaults = Project::draft(
                        record.project.name.clone(),
                        record.project.sources[0].as_str(),
                    )
                    .expect("validated stored source");
                    if project.environments.len() == 1
                        && project.environments[0].name == "Website"
                        && project.environments[0].mappings == old_defaults.environments[0].mappings
                        && project.environments[0].session.is_none()
                    {
                        project.environments[0].mappings = draft.environments[0].mappings.clone();
                    }
                    (project, Some(record.revision))
                } else {
                    (draft, None)
                };
                self.notice = None;
                let work = if let (Some(revision), Some(environment)) =
                    (expected_revision, self.environment_draft)
                {
                    match self.local_login_settings(cx) {
                        Ok(settings) => Work::SaveEnvironment {
                            project: Box::new(project),
                            revision,
                            environment,
                            settings,
                        },
                        Err(error) => {
                            self.error = Some(format!("Check your login settings: {error}"));
                            cx.notify();
                            return;
                        }
                    }
                } else if project.active_baseline.is_some() {
                    Work::SaveReplacement {
                        project: Box::new(project),
                        expected_revision: expected_revision.unwrap(),
                    }
                } else {
                    Work::Save {
                        project: Box::new(project),
                        expected_revision,
                    }
                };
                self.queue(work, cx);
            }
        }
    }
    fn set_crawl_fields(&mut self, cx: &mut Context<Self>) {
        let d = &self.draft_settings.discovery;
        let values = [
            d.page_limit.to_string(),
            d.section.clone().unwrap_or_default(),
            d.skipped_text.join(", "),
            d.advanced
                .max_depth
                .map(|n| n.to_string())
                .unwrap_or_default(),
            d.advanced
                .concurrency
                .map(|n| n.to_string())
                .unwrap_or_default(),
            d.advanced.per_origin.to_string(),
            d.advanced.retries.to_string(),
        ];
        for (i, v) in values.iter().enumerate() {
            self.crawl_fields[i].update(cx, |input, cx| input.set_value(v, cx));
        }
        if let Some(p) = self.draft_settings.profiles.first() {
            self.crawl_fields[15].update(cx, |input, cx| input.set_value(&p.locale, cx));
            self.timezone
                .update(cx, |select, cx| select.set_value(&p.timezone, cx));
        }
    }
    fn guided(&mut self, index: usize, cx: &mut Context<Self>) {
        let extent = self
            .draft_settings
            .profiles
            .first()
            .map(|p| p.extent)
            .unwrap_or(CaptureExtent::FullPage);
        let d = &mut self.draft_settings.discovery;
        match index {
            0 => {
                d.mode = if d.mode == DiscoveryMode::OnlyAddresses {
                    DiscoveryMode::AddressesAndLinks
                } else {
                    DiscoveryMode::OnlyAddresses
                }
            }
            1..=3 => {
                let name = ["Desktop", "Tablet", "Mobile"][index - 1];
                if self.draft_settings.profiles.iter().any(|p| p.name == name) {
                    self.draft_settings.profiles.retain(|p| p.name != name);
                } else {
                    let mut profile = match index {
                        1 => CaptureProfile::desktop(),
                        2 => CaptureProfile::tablet(),
                        _ => CaptureProfile::mobile(),
                    };
                    profile.extent = extent;
                    if let Some(p) = self.draft_settings.profiles.first() {
                        profile.image_mode = p.image_mode.clone();
                        profile.locale = p.locale.clone();
                        profile.timezone = p.timezone.clone();
                        profile.dark = p.dark;
                    }
                    self.draft_settings.profiles.push(profile);
                }
            }
            4 => {
                for p in &mut self.draft_settings.profiles {
                    p.extent = if p.extent == CaptureExtent::FullPage {
                        CaptureExtent::VisibleScreen
                    } else {
                        CaptureExtent::FullPage
                    };
                }
            }
            5 => {
                d.speed = match d.speed {
                    Speed::Gentle => Speed::Balanced,
                    Speed::Balanced => Speed::Fast,
                    Speed::Fast => Speed::Gentle,
                }
            }
            6 => self.focus.advanced = !self.focus.advanced,
            7 => {
                d.respect_robots = !d.respect_robots;
                d.advanced.robots_authorized = !d.respect_robots;
            }
            8 => d.advanced.allow_action_links = !d.advanced.allow_action_links,
            9 => d.advanced.strip_tracking = !d.advanced.strip_tracking,
            10 => d.advanced.preserve_fragments = !d.advanced.preserve_fragments,
            11 => match crawl::sources(&self.address.read(cx).content) {
                Ok(sources) => {
                    let origins: std::collections::BTreeSet<_> =
                        sources.iter().map(crawl::origin).collect();
                    if self.origins_confirmed && origins.is_subset(&self.approved_origins) {
                        self.approved_origins.clear();
                        self.origins_confirmed = false;
                        self.notice = Some("Approval cleared. Multiple websites must be approved again before saving.".into());
                        cx.notify();
                        return;
                    }
                    self.approved_origins = origins;
                    self.origins_confirmed = true;
                    self.notice = Some(format!(
                        "Approved websites: {}. Adding another website requires confirmation again. Discovered external websites are never approved automatically.",
                        self.approved_origins
                            .iter()
                            .cloned()
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                Err(e) => self.error = Some(format!("Check the address list before approval: {e}")),
            },
            12 => {
                let width = self.crawl_fields[7].read(cx).content.trim().parse::<u32>();
                let height = self.crawl_fields[8].read(cx).content.trim().parse::<u32>();
                match (width, height) {
                    (Ok(width), Ok(height))
                        if (1..=4096).contains(&width) && (1..=8192).contains(&height) =>
                    {
                        self.draft_settings.profiles.retain(|p| p.name != "Custom");
                        self.draft_settings.profiles.push(CaptureProfile {
                            name: "Custom".into(),
                            width,
                            height,
                            extent,
                            ..CaptureProfile::desktop()
                        });
                    }
                    _ => {
                        self.error =
                            Some("Custom width must be 1–4096 and height 1–8192 CSS pixels.".into())
                    }
                }
            }
            13 => {
                if self.unsaved(cx) {
                    self.error = Some("Save your settings before finding pages.".into());
                } else if let Some(p) = &self.project {
                    self.preview = None;
                    self.queue(Work::Discover(p.project.id), cx);
                    self.focus.capturing = self.pending.is_some();
                }
            }
            14 => {
                if let (Some(p), Some(preview)) = (&self.project, &self.preview) {
                    self.queue(
                        Work::CaptureInventory {
                            project: p.project.id,
                            preview: Some(Box::new(preview.clone())),
                            check: false,
                            resume: None,
                        },
                        cx,
                    );
                    self.focus.capturing = self.pending.is_some();
                } else {
                    self.error = Some("Find pages and review their coverage first.".into());
                }
            }
            15 => {
                if let (Some(p), Some(r)) = (&self.project, &self.evidence.latest) {
                    if matches!(r.run.state, JobState::Paused | JobState::Interrupted) {
                        self.queue(
                            Work::CaptureInventory {
                                project: p.project.id,
                                preview: None,
                                check: false,
                                resume: Some(r.run.id),
                            },
                            cx,
                        );
                        self.focus.capturing = self.pending.is_some();
                    } else {
                        self.error =
                            Some("Only paused or interrupted capture queues can resume.".into());
                    }
                }
            }
            16 => {
                if self.preview.is_some() {
                    self.preview = None;
                    self.focus.preview = false;
                    self.notice = Some("Preview dismissed; the baseline is unchanged.".into());
                } else if let Some(p) = &self.project {
                    self.queue(Work::ExistingInventory(p.project.id), cx);
                }
            }
            17 => {
                self.search = self.crawl_fields[9].read(cx).content.trim().to_lowercase();
                let visible = self.preview_page_ids();
                if !self.selected_page.is_some_and(|id| visible.contains(&id)) {
                    self.selected_page = visible.first().copied();
                }
            }
            18 => self.result_filter = (self.result_filter + 1) % 4,
            19 => self.group_profiles = !self.group_profiles,
            20..=22 => {
                let visible: std::collections::BTreeSet<_> =
                    self.preview_page_ids().into_iter().collect();
                if let Some(preview) = &mut self.preview {
                    for page in &mut preview.inventory.pages {
                        if visible.contains(&page.id)
                            && (index != 22 || Some(page.id) == self.selected_page)
                        {
                            if index == 20 || (index == 22 && page.included) {
                                page.included = false;
                                page.reason = Some("Excluded in preview".into());
                            } else if page.reason.as_deref() == Some("Excluded in preview") {
                                page.included = true;
                                page.reason = None;
                            }
                        }
                    }
                    preview.inventory.coverage=regressee_core::domain::DiscoveryCoverage::Limited{reason:"Selection is user-bounded. Excluded pages are not captured or verified.".into()};
                }
            }
            23 | 24 => {
                if self.preview.is_some() {
                    let visible = self.preview_page_ids();
                    let current = visible
                        .iter()
                        .position(|id| Some(*id) == self.selected_page)
                        .unwrap_or(0);
                    let next = if index == 23 {
                        current.saturating_sub(1)
                    } else {
                        (current + 1).min(visible.len().saturating_sub(1))
                    };
                    self.selected_page = visible.get(next).copied();
                } else if let Some(r) = &self.evidence.latest {
                    let indices = self.result_indices();
                    let current = indices
                        .iter()
                        .position(|i| Some(&r.run.targets[*i].key) == self.selected_key.as_ref())
                        .unwrap_or(0);
                    let next = if index == 23 {
                        current.saturating_sub(1)
                    } else {
                        (current + 1).min(indices.len().saturating_sub(1))
                    };
                    if let Some(i) = indices.get(next) {
                        if self.mask_dirty {
                            self.error=Some("Save ignored areas before changing targets; unsaved rectangles belong to this exact page/profile.".into());
                            cx.notify();
                            return;
                        }
                        self.selected_key = Some(r.run.targets[*i].key.clone());
                        self.invalidate_approval();
                        self.pan_x = 0.;
                        self.pan_y = 0;
                        self.clear_viewer();
                        if let Some(p) = &self.project {
                            self.queue(
                                Work::Viewer {
                                    analysis: self
                                        .evidence
                                        .comparison
                                        .as_ref()
                                        .map(|a| a.revision.id),
                                    project: p.project.id,
                                    tile: 0,
                                    key: self.selected_key.clone(),
                                },
                                cx,
                            );
                        }
                    }
                }
            }
            25 => self.draft_settings.profiles.retain(|p| p.name != "Custom"),
            26 => {
                let current = self.environment_draft;
                let i = self
                    .draft_settings
                    .environments
                    .iter()
                    .position(|e| Some(e.id) == current)
                    .unwrap_or(0);
                self.environment_draft = Some(
                    self.draft_settings.environments
                        [(i + 1) % self.draft_settings.environments.len()]
                    .id,
                );
                for i in [10, 11, 12] {
                    self.crawl_fields[i].update(cx, |input, cx| input.set_value("", cx));
                }
                self.notice=Some("Environment selection is a draft. Save environment settings before opening its browser or checking.".into());
            }
            27 => {
                let mut env = self.draft_settings.environments[0].clone();
                env.id = regressee_core::domain::EnvironmentId::new();
                env.name = format!("Staging {}", self.draft_settings.environments.len());
                env.session = None;
                if self.draft_settings.environments.len() < 16 {
                    self.environment_draft = Some(env.id);
                    self.draft_settings.environments.push(env);
                    self.focus.environments = self.draft_settings.environments.len();
                    for i in [10, 11, 12] {
                        self.crawl_fields[i].update(cx, |input, cx| input.set_value("", cx));
                    }
                }
            }
            28 => {
                if let Some(env) = self
                    .draft_settings
                    .environments
                    .iter_mut()
                    .find(|e| Some(e.id) == self.environment_draft)
                {
                    env.access = if env.access == regressee_core::domain::AccessPolicy::Public {
                        regressee_core::domain::AccessPolicy::InteractiveLogin
                    } else {
                        regressee_core::domain::AccessPolicy::Public
                    };
                }
            }
            29 => self.login_ephemeral = !self.login_ephemeral,
            30 => self.save_settings(cx),
            31..=36 => {
                if matches!(index, 31 | 32 | 36) && self.environment_unsaved(cx) {
                    self.error=Some("Save settings before login or probing. These actions use the saved environment, not draft fields.".into());
                    cx.notify();
                    return;
                }
                if let Some(p) = &self.project {
                    if index == 35 || index == 36 {
                        self.show_mapping = true;
                    }
                    let id = p.project.id;
                    let work = match index {
                        31 => Work::OpenLogin(id),
                        32 => Work::ConfirmLogin(id),
                        33 => Work::CloseLogin(id),
                        34 => Work::ForgetLogin(id),
                        35 => Work::EnvironmentStatus(id),
                        _ => Work::ProbeEnvironment(id),
                    };
                    self.queue(work, cx);
                    self.focus.capturing = self.pending.is_some() && matches!(index, 31 | 32 | 36);
                }
            }
            37 => {
                let result = (|| -> anyhow::Result<regressee_core::domain::OriginMapping> {
                    let source = url::Url::parse(self.crawl_fields[13].read(cx).content.trim())?;
                    let destination =
                        url::Url::parse(self.crawl_fields[14].read(cx).content.trim())?;
                    let m = regressee_core::domain::OriginMapping {
                        source,
                        destination,
                    };
                    regressee_core::environment::validate_mapping(&m)?;
                    Ok(m)
                })();
                match result {
                    Ok(m) => {
                        if let Some(env) = self
                            .draft_settings
                            .environments
                            .iter_mut()
                            .find(|e| Some(e.id) == self.environment_draft)
                        {
                            env.mappings.retain(|old| old.source != m.source);
                            env.mappings.push(m);
                            for i in [13, 14] {
                                self.crawl_fields[i]
                                    .update(cx, |input, cx| input.set_value("", cx));
                            }
                        }
                    }
                    Err(e) => self.error = Some(format!("Check mapping bases: {e}")),
                }
            }
            38 => {
                if let Some(env) = self
                    .draft_settings
                    .environments
                    .iter_mut()
                    .find(|e| Some(e.id) == self.environment_draft)
                {
                    for m in &mut env.mappings {
                        m.destination = m.source.clone();
                    }
                }
            }
            39 => {
                for p in &mut self.draft_settings.profiles {
                    p.dark = !p.dark;
                }
            }
            40 => {
                self.notice=Some("Read-only accounts are strongly recommended: GET page visits can change server data. Logout/delete/action links are excluded by default. Login may use explicitly approved identity providers, but crawl/capture scope never expands. Basic Auth, device-bound and rotating sessions are not universally supported. Full page does not expose virtualized rows or nested scroll contents. VPN uses your existing OS network; managed-network qualification is pending.".into());
            }
            41 => {
                let proxy = self.crawl_fields[17].read(cx).content.trim().to_owned();
                let ca = self.crawl_fields[18].read(cx).content.trim().to_owned();
                self.queue(
                    Work::SaveNetwork(regressee_runtime::network::NetworkSettings {
                        proxy: if proxy.is_empty() { None } else { Some(proxy) },
                        trusted_ca: if ca.is_empty() { None } else { Some(ca.into()) },
                    }),
                    cx,
                );
            }
            42 => self.queue(Work::NetworkStatus, cx),
            43 => {
                let replace = self
                    .draft_settings
                    .profiles
                    .first()
                    .is_none_or(|p| p.image_mode == regressee_core::domain::ImageMode::Normal);
                for p in &mut self.draft_settings.profiles {
                    p.image_mode = if replace {
                        regressee_core::domain::ImageMode::ReplaceRaster {
                            algorithm: "neutral-raster-v2".into(),
                        }
                    } else {
                        regressee_core::domain::ImageMode::Normal
                    };
                }
                self.notice=Some("Image mode applies only to future baseline capture. Existing checks always use their baseline's immutable mode. Save settings and preview a representative page first.".into());
            }
            _ => (),
        }
        cx.notify();
    }
    fn guided_button(
        &self,
        index: usize,
        label: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.button(Control::Guided(index), label, ActionStyle::Secondary, cx)
    }
    fn choose(&mut self, choice: Choice, cx: &mut Context<Self>) {
        use regressee_core::domain::{AccessPolicy, ImageMode};
        match choice {
            Choice::AddressesOnly => {
                self.draft_settings.discovery.mode = DiscoveryMode::OnlyAddresses
            }
            Choice::AddressesAndLinks => {
                self.draft_settings.discovery.mode = DiscoveryMode::AddressesAndLinks
            }
            Choice::Gentle => self.draft_settings.discovery.speed = Speed::Gentle,
            Choice::Balanced => self.draft_settings.discovery.speed = Speed::Balanced,
            Choice::Fast => self.draft_settings.discovery.speed = Speed::Fast,
            Choice::FullPage | Choice::VisibleScreen => {
                for profile in &mut self.draft_settings.profiles {
                    profile.extent = if choice == Choice::FullPage {
                        CaptureExtent::FullPage
                    } else {
                        CaptureExtent::VisibleScreen
                    };
                }
            }
            Choice::OriginalImages | Choice::ReplaceImages => {
                for profile in &mut self.draft_settings.profiles {
                    profile.image_mode = if choice == Choice::OriginalImages {
                        ImageMode::Normal
                    } else {
                        ImageMode::ReplaceRaster {
                            algorithm: "neutral-raster-v2".into(),
                        }
                    };
                }
            }
            Choice::PublicAccess | Choice::LoginRequired => {
                if let Some(environment) = self
                    .draft_settings
                    .environments
                    .iter_mut()
                    .find(|environment| Some(environment.id) == self.environment_draft)
                {
                    environment.access = if choice == Choice::PublicAccess {
                        AccessPolicy::Public
                    } else {
                        AccessPolicy::InteractiveLogin
                    };
                }
            }
            Choice::SaveLogin | Choice::EphemeralLogin => {
                self.login_ephemeral = choice == Choice::EphemeralLogin
            }
            Choice::LightTheme | Choice::DarkTheme => {
                for profile in &mut self.draft_settings.profiles {
                    profile.dark = choice == Choice::DarkTheme;
                }
            }
            Choice::AllResults => self.result_filter = 0,
            Choice::ChangedResults => self.result_filter = 1,
            Choice::MissingResults => self.result_filter = 2,
            Choice::UnchangedResults => self.result_filter = 3,
        }
        cx.notify();
    }
    fn choice_button(
        &self,
        control: Control,
        label: impl Into<SharedString>,
        description: impl Into<SharedString>,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let enabled = self.control_enabled(control, cx);
        let handle = self.handle(control, cx);
        controls::choice(
            SharedString::from(format!("{control:?}")),
            label,
            description,
            &handle,
            selected,
            enabled,
        )
        .when(enabled, |el| {
            el.on_click(
                cx.listener(move |this, _, window, cx| this.activate_control(control, window, cx)),
            )
        })
        .into_any_element()
    }
    fn check_button(
        &self,
        control: Control,
        label: impl Into<SharedString>,
        checked: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let enabled = self.control_enabled(control, cx);
        let handle = self.handle(control, cx);
        controls::check(
            SharedString::from(format!("{control:?}")),
            label,
            &handle,
            checked,
            enabled,
        )
        .when(enabled, |el| {
            el.on_click(
                cx.listener(move |this, _, window, cx| this.activate_control(control, window, cx)),
            )
        })
        .into_any_element()
    }
    fn crawl_field(&self, index: usize, label: &'static str, help: &'static str) -> Div {
        controls::field(
            label,
            help,
            self.crawl_fields[index].clone().into_any_element(),
        )
    }
    fn crawl_setup(&self, cx: &mut Context<Self>) -> AnyElement {
        let d = &self.draft_settings.discovery;
        let approved = self.origins_confirmed
            && crawl::sources(&self.address.read(cx).content).is_ok_and(|sources| {
                sources
                    .iter()
                    .map(crawl::origin)
                    .all(|origin| self.approved_origins.contains(&origin))
            });
        let mut body=div().w_full().max_w(px(680.)).flex_shrink_0().flex().flex_col().gap_6()
            .child(self.heading("Project setup","Tell us what to capture. Save your settings, then review the page list in Results."))
            .child(controls::section("Website", "Saving settings does not visit your website.")
                .child(controls::field("Project name","Choose a recognizable name.",self.name.clone().into_any_element()))
                .child(controls::field("Starting addresses","Use complete http(s) addresses separated by spaces. Visits stay within the listed websites; subdomains are not added.",self.address.clone().into_any_element()))
                .child(self.check_button(Control::Guided(11), "Approve all listed websites", approved, cx))
                .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("Approval is required when you list more than one website. Adding a different website requires approval again.")))
            .child(controls::section("Pages to capture", "Choose how Regressee builds your page list.")
                .child(self.choice_button(Control::Choice(Choice::AddressesOnly), "Only the starting addresses", "Capture exactly the addresses you entered.", d.mode == DiscoveryMode::OnlyAddresses, cx))
                .child(self.choice_button(Control::Choice(Choice::AddressesAndLinks), "Find linked pages", "Follow rendered links within these websites, up to your page limit.", d.mode == DiscoveryMode::AddressesAndLinks, cx))
                .child(self.crawl_field(0,"Maximum pages","1–10,000. A page captured at three screen sizes creates three screenshots."))
                .child(self.crawl_field(1,"Website section (optional)","Use a path such as /docs to stay within that section. Leave blank for the whole listed website."))
                .child(self.crawl_field(2,"Skip addresses containing (optional)","Separate text exclusions with commas. The preview explains excluded addresses.")));
        let mut profiles = div().flex().flex_wrap().gap_2();
        for (index, name, dimensions) in [
            (1, "Desktop", "1440 × 900"),
            (2, "Tablet", "768 × 1024"),
            (3, "Mobile", "390 × 844"),
        ] {
            profiles = profiles.child(self.check_button(
                Control::Guided(index),
                format!("{name} · {dimensions}"),
                self.draft_settings.profiles.iter().any(|p| p.name == name),
                cx,
            ));
        }
        let full_page = self
            .draft_settings
            .profiles
            .first()
            .is_none_or(|p| p.extent == CaptureExtent::FullPage);
        let original_images = self
            .draft_settings
            .profiles
            .first()
            .is_none_or(|p| p.image_mode == regressee_core::domain::ImageMode::Normal);
        body=body.child(controls::section("Screenshots", "Choose one or more screen sizes. Phone and tablet use Chromium emulation, not real devices.")
            .child(profiles)
            .child(self.choice_button(Control::Choice(Choice::FullPage), "Full page", "Include the page below the fold, within capture limits.", full_page, cx))
            .child(self.choice_button(Control::Choice(Choice::VisibleScreen), "Visible screen only", "Capture one viewport without scrolling.", !full_page, cx)))
            .child(controls::section("Images", "Choose before capturing a baseline. Later checks keep the baseline's image mode.")
                .child(self.choice_button(Control::Choice(Choice::OriginalImages), "Keep original images", "Compare the website as it appears.", original_images, cx))
                .child(self.choice_button(Control::Choice(Choice::ReplaceImages), "Replace raster images", "Use neutral replacements to reduce changing-photo noise. Raster logos may also be replaced.", !original_images, cx)))
            .child(controls::section("Capture speed", "Use a read-only account: even visiting pages can change website data.")
                .child(self.choice_button(Control::Choice(Choice::Gentle), "Gentle", "1 page at a time, with 1-second spacing.", d.speed == Speed::Gentle, cx))
                .child(self.choice_button(Control::Choice(Choice::Balanced), "Balanced", "2 pages at a time, with 350 ms spacing.", d.speed == Speed::Balanced, cx))
                .child(self.choice_button(Control::Choice(Choice::Fast), "Fast", "4 pages at a time, with no extra spacing.", d.speed == Speed::Fast, cx))
                .child(self.check_button(Control::Guided(7), "Override robots.txt — I have permission to do this", !d.respect_robots, cx))
                .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("robots.txt is respected unless you explicitly authorize an override. Robots rules are not permission. Rate limits can slow any preset.")))
            .child(self.guided_button(6,if self.focus.advanced{"Hide advanced controls"}else{"Advanced controls"},cx));
        if self.focus.advanced {
            body=body.child(div().max_w(px(680.)).flex().flex_col().gap_4()
            .child(self.crawl_field(3,"Links away from starting addresses","Optional depth 0–100; blank means bounded only by page/candidate caps."))
            .child(self.crawl_field(4,"Parallel active pages","Optional 1–8; blank uses the speed preset. Global browser operations remain bounded."))
            .child(self.crawl_field(5,"Parallel pages per website","1–8, never exceeding the global limit."))
            .child(self.crawl_field(6,"Retries for temporary failures","0–3. Timeouts, server errors and rate limits only; exponential backoff is bounded."))
            .child(self.check_button(Control::Guided(8), "Allow action-like links — I understand visits may change data", d.advanced.allow_action_links, cx))
            .child(self.check_button(Control::Guided(9), "Ignore known tracking parameters", d.advanced.strip_tracking, cx))
            .child(self.check_button(Control::Guided(10), "Keep all explicit fragments (not just hash routes)", d.advanced.preserve_fragments, cx))
            .child(self.crawl_field(7,"Custom width (CSS pixels)","1–4096; custom profiles use desktop Chromium emulation."))
            .child(self.crawl_field(8,"Custom height (CSS pixels)","1–8192."))
            .child(self.guided_button(12,"Add / replace Custom screen size",cx)).child(self.guided_button(25,"Remove Custom screen size",cx))
            .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("Image replacement supports decoded HTML raster/picture images and single recognized raster CSS backgrounds. Vector icons remain. Canvas, video, frames, shadow DOM, pseudo-elements, complex/extensionless backgrounds and source-selector/script behavior are unsupported. Readiness or geometry warnings prevent assurance. Hidden original photo content is not separately saved in replacement-only captures. Rendering dimensions, touch/mobile emulation, DPR 1 and runtime are frozen with the baseline.")));
        }
        if self.project.is_some() {
            body = body.child(self.environment_setup(cx));
        }
        body = body.child(self.button(
            Control::Details(Detail::Capture),
            if self.details.contains(&Detail::Capture) {
                "Hide capture details and limitations"
            } else {
                "Capture details and limitations"
            },
            ActionStyle::Secondary,
            cx,
        ));
        if self.details.contains(&Detail::Capture) {
            body = body.child(div().text_size(px(14.)).text_color(rgb(MUTED)).child("Reduced motion, disabled animations, hidden caret and pointer-away capture are fixed. Font/image readiness, lazy loading and stability are bounded; tall/infinite pages fail rather than silently crop. Full page is limited to 32,768 pixels tall, 64 million pixels and 64 scroll steps, or the profile's lower limit. Virtualized rows and nested scroll areas are not expanded. Login may use approved identity providers without expanding crawl scope. Basic Auth, device-bound/rotating sessions, VPN and enterprise networks are not universally qualified."));
        }
        body
            .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("Saving edits never changes an existing baseline. The private browser is installed automatically when needed.")).into_any_element()
    }
    fn environment_setup(&self, cx: &mut Context<Self>) -> AnyElement {
        let draft = self
            .draft_settings
            .environments
            .iter()
            .find(|e| Some(e.id) == self.environment_draft);
        let login_required = draft.is_some_and(|environment| {
            environment.access == regressee_core::domain::AccessPolicy::InteractiveLogin
        });
        let mut body=controls::section("Environment & login", "Production and staging keep separate logins. Save your settings before opening a login browser.")
            .children(self.draft_settings.environments.iter().enumerate().map(|(index, environment)| self.choice_button(Control::Environment(index), environment.name.clone(), "", Some(environment.id) == self.environment_draft, cx)))
            .child(self.guided_button(27,"Add staging environment",cx))
            .when(self.environment_unsaved(cx), |el| el.child(controls::status("Unsaved environment edits — Save settings below before choosing another environment.", Tone::Information)))
            .child(self.choice_button(Control::Choice(Choice::PublicAccess), "No login required", "Use this for pages anyone can view.", !login_required, cx))
            .child(self.choice_button(Control::Choice(Choice::LoginRequired), "Login required", "Log in yourself in the app's browser, then confirm you are ready.", login_required, cx))
            .child(self.crawl_field(10,"Login / consent page (optional)","Blank opens the representative page. Use a complete URL, without credentials."))
            .child(self.crawl_field(11,"Representative page after login (optional)","Choose a real application page, not a login form. Blank uses the first mapped source. Replay is checked in a fresh isolated context."))
            .child(self.crawl_field(12,"Approved identity-provider origins (optional)","Exact http(s) origins ending in /, separated by spaces. Approve only providers you trust. This does not expand crawl scope."))
            .child(self.choice_button(Control::Choice(Choice::SaveLogin), "Remember login securely", "Use encrypted storage and your operating system's secure store.", !self.login_ephemeral, cx))
            .child(self.choice_button(Control::Choice(Choice::EphemeralLogin), "Only for this app session", "Forget the login when Regressee closes.", self.login_ephemeral, cx))
            .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("If secure storage is locked or unavailable, unlock it or explicitly choose Ephemeral. There is no plaintext fallback. Screenshots may still contain private website content."))
            .child(div().font_weight(FontWeight::SEMIBOLD).mt_3().child("Production → staging mapping"))
            .child(self.crawl_field(13,"Source origin or section base","For example https://production.example/app. No query or fragment in the base."))
            .child(self.crawl_field(14,"Matching destination base","For example https://staging.example/preview. Relative paths, ordered query pairs and hash routes are preserved."))
            .child(div().flex().flex_wrap().gap_2().child(self.guided_button(37,"Approve / replace this base mapping",cx)).child(self.guided_button(38,"Use same website for every mapping",cx)));
        if let Some(env) = draft {
            body = body.children(env.mappings.iter().take(16).map(|m| {
                div()
                    .text_size(px(13.))
                    .child(format!("{} → {}", m.source, m.destination))
            }));
        }
        body=body.child(self.crawl_field(15,"Capture language / locale","For example en-US or nb-NO. Changing rendering settings requires a replacement baseline."))
            .child(controls::field("Capture timezone", "Choose a timezone or search by region or city. Changes apply to future baseline captures.", self.timezone.clone().into_any_element()))
            .child(self.choice_button(Control::Choice(Choice::LightTheme), "Light website theme", "Capture the website's light color scheme.", self.draft_settings.profiles.first().is_none_or(|profile| !profile.dark), cx))
            .child(self.choice_button(Control::Choice(Choice::DarkTheme), "Dark website theme", "Capture the website's dark color scheme.", self.draft_settings.profiles.first().is_some_and(|profile| profile.dark), cx))
            .child(self.environment_actions(cx));
        body.into_any_element()
    }
    fn environment_actions(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div().flex().flex_col().gap_3().flex_shrink_0();
        if let Some(e) = &self.environment {
            body = body
                .child(controls::status(
                    if e.login_open {
                        "Browser open — finish login and consent, then choose I'm ready"
                    } else {
                        "Saved environment"
                    },
                    Tone::Neutral,
                ))
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(MUTED))
                        .child(format!("{} · {}", e.name, e.message)),
                );
        }
        body = body
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(self.guided_button(31, "Open login browser", cx))
                    .child(self.guided_button(32, "I'm ready", cx))
                    .child(self.guided_button(33, "Close login browser", cx))
                    .child(self.button(
                        Control::Guided(34),
                        "Forget login",
                        ActionStyle::Danger,
                        cx,
                    )),
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(self.guided_button(35, "Preview mapped URLs", cx))
                    .child(self.guided_button(36, "Probe representative pages", cx)),
            );
        if self.show_mapping
            && let Some(e) = &self.environment
        {
            body=body.child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(format!("{} intended page mappings · {} · preview shows first 16; probe visits at most 3, never a full discovery.",e.mapped.len(),if e.probed{"Representative probe confirmed"}else{"Probe not confirmed for these saved settings"})))
                .children(e.mapped.iter().take(16).map(|(a,b)|div().text_size(px(13.)).child(format!("{a} → {b}"))));
        }
        body.child(
            div()
                .debug_selector(|| "environment-login-hint".into())
                .flex()
                .w_full()
                .min_h(px(20.))
                .flex_shrink_0()
                .text_size(px(13.))
                .line_height(px(20.))
                .text_color(rgb(MUTED))
                .child("Open the browser and complete login or consent. Then choose I'm ready."),
        )
        .into_any_element()
    }
    fn preview_page_ids(&self) -> Vec<regressee_core::domain::PageId> {
        self.preview
            .as_ref()
            .map(|p| {
                p.inventory
                    .pages
                    .iter()
                    .filter(|p| p.canonical.as_str().to_lowercase().contains(&self.search))
                    .map(|p| p.id)
                    .collect()
            })
            .unwrap_or_default()
    }
    fn select_preview_page(&mut self, id: regressee_core::domain::PageId) {
        if self.preview_page_ids().contains(&id) {
            self.selected_page = Some(id);
        }
    }
    fn result_indices(&self) -> Vec<usize> {
        let Some(r) = &self.evidence.latest else {
            return vec![];
        };
        let classifications: std::collections::BTreeMap<_, _> = self
            .evidence
            .comparison
            .as_ref()
            .map(|a| {
                a.revision
                    .results
                    .iter()
                    .map(|r| (&r.key, r.classification))
                    .collect()
            })
            .unwrap_or_default();
        let classification =
            |t: &regressee_core::domain::CaptureTarget| classifications.get(&t.key).copied();
        let mut rows: Vec<_> = r
            .run
            .targets
            .iter()
            .enumerate()
            .filter(|(_, t)| {
                t.requested.as_str().to_lowercase().contains(&self.search)
                    && match self.result_filter {
                        1 => classification(t) == Some(Classification::Changed),
                        2 => !matches!(t.outcome, TargetOutcome::Captured { .. }),
                        3 => classification(t) == Some(Classification::NoSignificantChange),
                        _ => true,
                    }
            })
            .map(|(i, _)| i)
            .collect();
        rows.sort_by_cached_key(|i| {
            let t = &r.run.targets[*i];
            (
                if self.group_profiles {
                    t.key.profile.to_string()
                } else {
                    String::new()
                },
                match classification(t) {
                    Some(Classification::Changed) => 0,
                    Some(Classification::CouldNotCapture | Classification::NotComparable) => 1,
                    Some(Classification::NoSignificantChange) => 3,
                    _ => 2,
                },
                t.requested.to_string(),
            )
        });
        rows
    }
    fn inventory_and_results(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.preview.is_none() && self.evidence.latest.is_none() {
            return div().into_any_element();
        }
        let mut body=div().flex().flex_col().gap_3()
            .child(self.crawl_field(9,"Search page addresses","Enter address text, then Apply search. Search changes the visible list, not coverage."))
            .child(div().flex().flex_wrap().gap_2().child(self.guided_button(17,"Apply search",cx)).child(self.guided_button(23,"Previous target",cx)).child(self.guided_button(24,"Next target",cx)));
        if let Some(p) = &self.preview {
            body = body
                .child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .child("Discovery preview — not yet a baseline"),
                )
                .child(p.estimate())
                .child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(MUTED))
                        .child(format!(
                            "{} rendered pages visited · {} candidates · {}",
                            p.visited,
                            p.inventory.pages.len(),
                            p.inventory.coverage.label()
                        )),
                )
                .children(p.notes.iter().take(16).map(|s| {
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(MUTED))
                        .child(s.clone())
                }))
                .child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .child(self.guided_button(20, "Exclude search matches", cx))
                        .child(self.guided_button(21, "Restore excluded search matches", cx))
                        .child(self.guided_button(22, "Include / exclude selected page", cx))
                        .child(self.guided_button(16, "Dismiss preview — keep baseline", cx)),
                );
            let rows = self.preview_page_ids();
            body = body.child(
                div()
                    .text_size(px(13.))
                    .text_color(rgb(MUTED))
                    .child(format!(
                        "{} of {} page addresses shown",
                        rows.len(),
                        p.inventory.pages.len()
                    )),
            );
            if rows.is_empty() {
                body = body.child(controls::empty("No matching pages", "Change or clear the search text, then apply the search again. Your capture list is unchanged."));
            }
            body = body.child(
                uniform_list(
                    "inventory-preview",
                    rows.len(),
                    cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                        range
                            .map(|visible| {
                                let id = rows[visible];
                                let Some(p) = this
                                    .preview
                                    .as_ref()
                                    .and_then(|p| p.inventory.pages.iter().find(|p| p.id == id))
                                else {
                                    return div().into_any_element();
                                };
                                div()
                                    .id(SharedString::from(format!("candidate-{id}")))
                                    .h(px(60.))
                                    .px_3()
                                    .py_2()
                                    .border_b_1()
                                    .border_color(rgb(LINE))
                                    .bg(rgb(if Some(id) == this.selected_page {
                                        SELECTED
                                    } else {
                                        SURFACE
                                    }))
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_size(px(14.))
                                            .text_ellipsis()
                                            .child(p.canonical.to_string()),
                                    )
                                    .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(
                                        if p.included {
                                            "Included".into()
                                        } else {
                                            p.reason.clone().unwrap_or_else(|| "Excluded".into())
                                        },
                                    ))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        this.select_preview_page(id);
                                        cx.notify();
                                    }))
                                    .into_any_element()
                            })
                            .collect::<Vec<_>>()
                    }),
                )
                .h(px(300.)),
            );
        } else if let Some(r) = &self.evidence.latest {
            let counts = TargetCounts::from_run(&r.run);
            let pages = r.run.inventory.pages.iter().filter(|p| p.included).count();
            body=body.child(div().font_weight(FontWeight::SEMIBOLD).child(format!("{pages} intended pages · {} screenshot targets",r.run.targets.len())))
                .child(format!("{} captured · {} failed · {} skipped · {} cancelled · {} pending",counts.captured,counts.failed,counts.skipped,counts.cancelled,counts.pending))
                .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(format!("Inventory coverage: {} Missing screenshots are not unchanged. Resume keeps frozen settings and requires you to confirm the website release has not changed.",r.run.inventory.coverage.label())))
                .child(div().flex().flex_wrap().gap_2().children(Choice::RESULTS.into_iter().enumerate().map(|(index, choice)| self.button(Control::Choice(choice), ["All", "Changed", "Missing coverage", "No significant change"][index], ActionStyle::Choice { selected: self.result_filter == index }, cx))))
                .child(self.check_button(Control::Guided(19), "Group by screen size", self.group_profiles, cx))
                .when(matches!(r.run.state,JobState::Paused|JobState::Interrupted), |el| el.child(self.guided_button(15,"Resume — same website release confirmed",cx)));
            let rows = self.result_indices();
            body = body.child(
                div()
                    .text_size(px(13.))
                    .text_color(rgb(MUTED))
                    .child(format!(
                        "{} of {} screenshot targets shown",
                        rows.len(),
                        r.run.targets.len()
                    )),
            );
            if rows.is_empty() {
                body = body.child(controls::empty("No matching targets", "Choose All and clear the search text to see every target. Hidden or missing screenshots are not unchanged pages."));
            }
            body = body.child(
                uniform_list(
                    "capture-targets",
                    rows.len(),
                    cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                        range
                            .map(|visible| {
                                let index = rows[visible];
                                let r = this.evidence.latest.as_ref().unwrap();
                                let t = &r.run.targets[index];
                                let key = t.key.clone();
                                let profile =
                                    r.run.profiles.iter().find(|p| p.id == key.profile).unwrap();
                                let classification = this
                                    .evidence
                                    .comparison
                                    .as_ref()
                                    .and_then(|a| a.revision.results.iter().find(|a| a.key == key))
                                    .map(|a| a.classification.label().to_owned())
                                    .unwrap_or_else(|| match t.outcome {
                                        TargetOutcome::Captured { .. } => {
                                            "Captured — analysis not available".into()
                                        }
                                        TargetOutcome::Pending => "Pending".into(),
                                        TargetOutcome::Failed { code } => {
                                            format!("Could not capture: {}", code.label())
                                        }
                                        TargetOutcome::Skipped { ref reason } => {
                                            format!("Skipped: {reason}")
                                        }
                                        TargetOutcome::Cancelled => "Cancelled".into(),
                                    });
                                div()
                                    .id(("target", index))
                                    .h(px(60.))
                                    .px_3()
                                    .py_2()
                                    .border_b_1()
                                    .border_color(rgb(LINE))
                                    .bg(rgb(if this.selected_key.as_ref() == Some(&key) {
                                        SELECTED
                                    } else {
                                        SURFACE
                                    }))
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .text_size(px(14.))
                                             .text_ellipsis()
                                            .child(t.requested.to_string()),
                                    )
                                    .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(
                                        format!(
                                            "{} · {classification} · {} attempts",
                                            profile.name, t.attempts
                                        ),
                                    ))
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if this.pending.is_none() {
                                            if this.mask_dirty{this.error=Some("Save ignored areas before changing targets; rectangles will not be moved to another page/profile.".into());cx.notify();return;}
                                            this.selected_key = Some(key.clone());
                                            this.invalidate_approval();
                                            this.pan_x = 0.;
                                            this.pan_y = 0;
                                            this.clear_viewer();
                                            if let Some(p) = &this.project {
                                                this.queue(
                                                    Work::Viewer {
                                                        analysis: this.evidence.comparison.as_ref().map(|a|a.revision.id),
                                                        project: p.project.id,
                                                        tile: 0,
                                                        key: Some(key.clone()),
                                                    },
                                                    cx,
                                                );
                                            }
                                        }
                                    }))
                            })
                            .collect::<Vec<_>>()
                    }),
                )
                .h(px(300.)),
            );
        }
        body.into_any_element()
    }
    fn activate_control(&mut self, control: Control, window: &mut Window, cx: &mut Context<Self>) {
        if !self.control_enabled(control, cx) {
            return;
        }
        if self.mask_dirty
            && matches!(
                control,
                Control::RefreshEvidence
                    | Control::CaptureBaseline
                    | Control::CheckChanges
                    | Control::Phase5(0..=2 | 4 | 26 | 30)
                    | Control::Guided(13 | 16)
            )
        {
            self.error = Some("Save ignored areas before reloading or replacing this evidence. Your unsaved rectangles are kept.".into());
            cx.notify();
            return;
        }
        match control {
            Control::Navigate(screen) => self.navigate(screen, window, cx),
            Control::NewProject => self.create(&CreateProject, window, cx),
            Control::Save => self.save(&SaveProject, window, cx),
            Control::OpenProject(index) => {
                if self.unsaved(cx) {
                    self.confirm_switch(control, window, cx);
                    return;
                }
                if let Some(project) = self.projects.items.get(index) {
                    let id = project.id;
                    self.focus.screen = Screen::Setup;
                    self.notice = None;
                    self.queue(Work::Load(id), cx);
                    window.focus(&self.handle(Control::Navigate(Screen::Setup), cx));
                }
            }
            Control::PreviousPage => self.queue(
                Work::List {
                    page: self.projects.page.saturating_sub(1),
                },
                cx,
            ),
            Control::NextPage => self.queue(
                Work::List {
                    page: self.projects.page + 1,
                },
                cx,
            ),
            Control::ChooseFolder => {
                let prompt = cx.prompt_for_paths(PathPromptOptions {
                    files: false,
                    directories: true,
                    multiple: false,
                    prompt: Some("Inspect a folder (does not move project data)".into()),
                });
                cx.spawn(async move |this, cx| {
                    if let Ok(Ok(Some(paths))) = prompt.await && let Some(path) = paths.first() { let text = format!("Selected {}. Project data has not moved; changing the data location is not available yet.", path.display()); let _ = this.update(cx, |this, cx| { this.notice = Some(text); cx.notify(); }); }
                }).detach();
            }
            Control::About => {
                self.restore_focus = Some(control);
                self.focus.dialog = true;
                window.focus(&self.handle(Control::CloseDialog, cx));
                cx.notify();
            }
            Control::CloseDialog => self.dismiss(&Dismiss, window, cx),
            Control::Name
            | Control::Address
            | Control::CrawlField(_)
            | Control::AnalysisField(_) => (),
            Control::Phase5(index) => self.phase5_action(index, cx),
            Control::Phase6(index) => self.phase6_action(index, window, cx),
            Control::Phase7(index) => self.phase7_action(index, window, cx),
            Control::Guided(index) => self.guided(index, cx),
            Control::Choice(choice) => self.choose(choice, cx),
            Control::Environment(index) => {
                if let Some(environment) = self.draft_settings.environments.get(index) {
                    if self.environment_draft == Some(environment.id) {
                        return;
                    }
                    self.environment_draft = Some(environment.id);
                    for i in [10, 11, 12] {
                        self.crawl_fields[i].update(cx, |input, cx| input.set_value("", cx));
                    }
                    self.notice = Some("Environment selection is not saved yet. Save environment settings before logging in or checking.".into());
                    cx.notify();
                }
            }
            Control::Details(detail) => {
                if !self.details.remove(&detail) {
                    self.details.insert(detail);
                }
                cx.notify();
            }
            Control::DismissFeedback => {
                self.error = None;
                self.notice = None;
                window.focus(&self.handle(Control::Navigate(self.focus.screen), cx));
                cx.notify();
            }
            Control::PauseCapture => {
                self.jobs.pause();
                self.notice=Some("Pause requested. No new batch will start. Active pages finish their bounded browser work (default budget 55 seconds), then publish evidence. Large image processing may add time.".into());
                cx.notify();
            }
            Control::CancelCapture => {
                self.jobs.cancel();
                self.notice=Some("Stopping active work. Completed originals will be kept; no baseline is activated.".into());
                cx.notify();
            }
            Control::SetupRuntime => {
                self.queue(Work::SetupRuntime, cx);
                self.focus.capturing = self.pending.is_some();
            }
            Control::CaptureBaseline | Control::CheckChanges => {
                if self.unsaved(cx) {
                    self.error = Some("Save your project edits before capturing.".into());
                    cx.notify();
                    return;
                }
                if let Some(p) = &self.project {
                    if control == Control::CaptureBaseline && self.preview.is_none() {
                        self.queue(Work::Discover(p.project.id), cx);
                        self.focus.capturing = self.pending.is_some();
                        return;
                    }
                    self.queue(
                        Work::CaptureInventory {
                            project: p.project.id,
                            check: control == Control::CheckChanges,
                            preview: self.preview.clone().map(Box::new),
                            resume: None,
                        },
                        cx,
                    );
                    self.focus.capturing = self.pending.is_some();
                }
            }
            Control::SaveBaseline => {
                if let (Some(p), Some(r)) = (&self.project, &self.evidence.latest) {
                    let run = r.run.id;
                    let revision = p.revision;
                    self.native_prompt = true;
                    let answer=window.prompt(PromptLevel::Warning,"Use this successful capture as the approved baseline?",Some("This explicitly activates a new immutable reference. Earlier baselines and runs are kept."),&["Keep reviewing","Save baseline"],cx);
                    cx.spawn_in(window, async move |this, cx| {
                        let confirm = answer.await == Ok(1);
                        let _ = this.update_in(cx, |this, _, cx| {
                            this.native_prompt = false;
                            if confirm {
                                this.queue(Work::Activate { run, revision }, cx);
                            }
                            cx.notify();
                        });
                    })
                    .detach();
                }
            }
            Control::RefreshEvidence => {
                if let Some(p) = &self.project {
                    self.queue(Work::Evidence(p.project.id), cx);
                }
            }
            Control::ZoomIn
            | Control::ZoomOut
            | Control::PanLeft
            | Control::PanRight
            | Control::PanUp
            | Control::PanDown => {
                match control {
                    Control::ZoomIn => self.zoom = (self.zoom * 2.).min(4.),
                    Control::ZoomOut => self.zoom = (self.zoom / 2.).max(0.5),
                    Control::PanLeft => self.pan_x = (self.pan_x - 128.).max(0.),
                    Control::PanRight => {
                        self.pan_x = (self.pan_x + 128.)
                            .min(self.viewer_dimensions().0.saturating_sub(128) as f32)
                    }
                    Control::PanUp => self.pan_y = self.pan_y.saturating_sub(128),
                    Control::PanDown => {
                        self.pan_y =
                            (self.pan_y + 128).min(self.viewer_dimensions().1.saturating_sub(240))
                    }
                    _ => (),
                }
                let tile = self.pan_y as usize / 512;
                if self.viewer.as_ref().is_none_or(|v| v.tile != tile)
                    && let Some(p) = &self.project
                {
                    self.queue(
                        Work::Viewer {
                            analysis: self.evidence.comparison.as_ref().map(|a| a.revision.id),
                            project: p.project.id,
                            tile,
                            key: self.selected_key.clone(),
                        },
                        cx,
                    );
                }
                cx.notify();
            }
        }
    }
    fn activate(&mut self, _: &Activate, window: &mut Window, cx: &mut Context<Self>) {
        if let Some(control) = self.focused(window, cx) {
            self.activate_control(control, window, cx);
        }
    }
    fn dismiss(&mut self, _: &Dismiss, window: &mut Window, cx: &mut Context<Self>) {
        if self.diagnostic_preview.take().is_some() {
            window.focus(&self.handle(Control::Phase7(0), cx));
            cx.notify();
        }
        if self.focus.dialog {
            self.focus.dialog = false;
            let target = self
                .restore_focus
                .take()
                .unwrap_or(Control::Navigate(self.focus.screen));
            window.focus(&self.handle(target, cx));
            cx.notify();
        }
    }
    fn button(
        &self,
        control: Control,
        label: impl Into<SharedString>,
        style: ActionStyle,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let enabled = self.control_enabled(control, cx);
        let handle = self.handle(control, cx);
        controls::action(
            SharedString::from(format!("{control:?}")),
            label,
            &handle,
            style,
            enabled,
        )
        .when(enabled, |el| {
            el.on_click(
                cx.listener(move |this, _, window, cx| this.activate_control(control, window, cx)),
            )
        })
        .into_any_element()
    }
    /// The same prerequisites govern pointer actions, keyboard dispatch, and
    /// Tab stops. Backend validation remains authoritative for every mutation.
    fn control_enabled(&self, control: Control, cx: &App) -> bool {
        if !self.focus.can_activate(control) || !self.control_mounted(control) || self.native_prompt
        {
            return false;
        }
        let comparison = self.evidence.comparison.as_ref();
        let regions = comparison
            .and_then(|a| a.perceptual.as_ref())
            .and_then(|p| {
                p.reports
                    .iter()
                    .find(|(key, _)| Some(key) == self.selected_key.as_ref())
            })
            .map_or(0, |(_, report)| report.regions.retained.len());
        match control {
            Control::DismissFeedback => self.error.is_some() || self.notice.is_some(),
            Control::SetupRuntime => self
                .runtime_readiness
                .as_ref()
                .is_some_and(|status| status.setup_available),
            Control::CrawlField(index) => self.field_enabled(index),
            Control::Guided(index @ 1..=3) => {
                self.draft_settings.profiles.len() > 1
                    || !self
                        .draft_settings
                        .profiles
                        .iter()
                        .any(|profile| profile.name == ["Desktop", "Tablet", "Mobile"][index - 1])
            }
            Control::AnalysisField(index) => {
                comparison.is_some() && (index == 9 || self.analysis_advanced)
            }
            Control::Phase7(1 | 2) => self.diagnostic_preview.is_some(),
            Control::Environment(index) => {
                self.draft_settings
                    .environments
                    .get(index)
                    .is_some_and(|environment| {
                        Some(environment.id) == self.environment_draft
                            || !self.environment_unsaved(cx)
                    })
            }
            Control::Choice(choice) if Choice::RESULTS.contains(&choice) => {
                self.preview.is_none() && self.evidence.latest.is_some()
            }
            Control::Guided(0 | 4 | 5 | 18 | 26 | 28 | 29 | 30 | 39 | 40 | 43) => false,
            Control::Guided(12 | 25) => {
                self.focus.advanced
                    && (control != Control::Guided(25)
                        || self
                            .draft_settings
                            .profiles
                            .iter()
                            .any(|profile| profile.name == "Custom"))
            }
            Control::Guided(13) => {
                self.focus.screen == Screen::Setup && self.project.is_some() && !self.unsaved(cx)
            }
            Control::Guided(14) => false, // One contextual capture action, not a duplicate ID.
            Control::Guided(16) => self.preview.is_some() || self.evidence.baseline.is_some(),
            Control::Guided(17) => self.preview.is_some() || self.evidence.latest.is_some(),
            Control::Guided(19) => self.preview.is_none() && self.evidence.latest.is_some(),
            Control::Guided(20..=22) => self.preview.is_some(),
            Control::Guided(23 | 24) => {
                let (index, count) = if self.preview.is_some() {
                    let rows = self.preview_page_ids();
                    (
                        rows.iter()
                            .position(|id| Some(*id) == self.selected_page)
                            .unwrap_or(0),
                        rows.len(),
                    )
                } else {
                    let rows = self.result_indices();
                    (
                        self.evidence
                            .latest
                            .as_ref()
                            .and_then(|run| {
                                rows.iter().position(|index| {
                                    Some(&run.run.targets[*index].key) == self.selected_key.as_ref()
                                })
                            })
                            .unwrap_or(0),
                        rows.len(),
                    )
                };
                count > 0
                    && if control == Control::Guided(23) {
                        index > 0
                    } else {
                        index + 1 < count
                    }
            }
            Control::Guided(27) => {
                self.draft_settings.environments.len() < 16 && !self.environment_unsaved(cx)
            }
            Control::Guided(32 | 33) => self
                .environment
                .as_ref()
                .is_some_and(|environment| environment.login_open),
            Control::Guided(34) => self.project.as_ref().is_some_and(|project| {
                project
                    .project
                    .environments
                    .iter()
                    .any(|environment| environment.session.is_some())
            }),
            Control::Phase5(5) => self.analysis_pending.is_some() || self.analysis_due.is_some(),
            Control::Phase5(10 | 11) => self.viewer_mode == 2,
            Control::Phase5(12 | 13) => regions > 0,
            Control::Phase5(15) => !self.mask_rectangles.is_empty(),
            Control::Phase5(16 | 17) => self
                .mask_selected
                .is_some_and(|index| index < self.mask_rectangles.len()),
            Control::Phase5(18) => {
                self.mask_dirty
                    && self
                        .viewer
                        .as_ref()
                        .is_some_and(|viewer| viewer.request.is_some())
            }
            Control::Phase5(19) => self
                .viewer
                .as_ref()
                .and_then(|viewer| viewer.request.as_ref())
                .is_some_and(|request| request.validate().is_err()),
            Control::Phase5(20..=23) => comparison.is_some() && self.selected_key.is_some(),
            Control::Phase5(24) => {
                !self.approval_selection.is_empty()
                    && self.analysis_due.is_none()
                    && self.analysis_pending.is_none()
            }
            Control::Phase5(25) => self
                .approval
                .as_ref()
                .is_some_and(|preview| self.approval_matches(preview)),
            Control::Phase5(26) => self.evidence.analyses.len() > 1,
            Control::Phase5(27 | 28) => self.analysis_advanced,
            Control::Phase5(29 | 31) => false,
            Control::Phase5(30) => self.preview.is_some(),
            Control::Phase5(40) => self.history_index > 0,
            Control::Phase5(41) => self.history_index + 1 < self.evidence.baselines.len(),
            Control::Phase5(42) => {
                self.evidence
                    .baselines
                    .get(self.history_index)
                    .is_some_and(|baseline| {
                        self.project.as_ref().is_some_and(|project| {
                            project.project.active_baseline != Some(baseline.id)
                        })
                    })
            }
            Control::Phase6(1) => self.import_preview.is_some(),
            Control::Phase6(2 | 4) => self.evidence.baselines.get(self.history_index).is_some(),
            Control::Phase6(3) => self.export_preview.is_some(),
            Control::Phase6(5) => self
                .history
                .as_ref()
                .is_some_and(|history| self.run_history_index < history.runs.len()),
            Control::Phase6(6) => self.deletion_preview.is_some(),
            Control::Phase6(7) => {
                self.export_preview.is_some()
                    || self.import_preview.is_some()
                    || self.deletion_preview.is_some()
                    || self.repair_preview.is_some()
            }
            Control::Phase6(9) => self
                .history
                .as_ref()
                .is_some_and(|history| self.run_history_index + 1 < history.runs.len()),
            Control::Phase6(10) => self.run_history_index > 0,
            Control::Phase6(11) => self.history.as_ref().is_some_and(|history| history.more),
            Control::Phase6(12) => self
                .history
                .as_ref()
                .is_some_and(|history| history.page > 0),
            Control::Phase6(14 | 15) => self.repair_preview.is_some(),
            Control::Details(Detail::Analysis) => comparison.is_some(),
            _ => true,
        }
    }
    /// Mount conditions are separate from mutation prerequisites. A retained
    /// state from another screen must never become an invisible keyboard target.
    /// Shared action rendering, activation, and Tab all use control_enabled.
    fn control_mounted(&self, control: Control) -> bool {
        use Control::*;
        let screen = self.focus.screen;
        let project = self.project.is_some();
        let results = screen == Screen::Results && project;
        let setup = screen == Screen::Setup;
        let history = screen == Screen::History && project;
        let settings = screen == Screen::Settings;
        let portability = screen == Screen::Projects || history || settings;
        let inventory = results && (self.preview.is_some() || self.evidence.latest.is_some());
        let analysis = results && self.evidence.comparison.is_some();
        let viewer = results && self.viewer.is_some();
        match control {
            Navigate(_) => true,
            CloseDialog => self.focus.dialog,
            DismissFeedback => self.error.is_some() || self.notice.is_some(),
            NewProject => screen == Screen::Projects,
            Name | Address | Save => setup,
            OpenProject(index) => screen == Screen::Projects && index < self.projects.items.len(),
            PreviousPage => screen == Screen::Projects && self.projects.page > 0,
            NextPage => screen == Screen::Projects && self.projects.more,
            ChooseFolder | About | SetupRuntime => settings,
            CaptureBaseline | RefreshEvidence => results,
            SaveBaseline => results && self.focus.candidate,
            CheckChanges => results && self.focus.baseline,
            CancelCapture => self.focus.busy && self.focus.capturing,
            PauseCapture => self.focus.busy && self.focus.capturing && self.focus.pausable,
            ZoomIn | ZoomOut | PanLeft | PanRight | PanUp | PanDown => viewer,
            Guided(1..=3 | 6 | 7 | 11) => setup,
            Guided(8..=10 | 12 | 25) => setup && self.focus.advanced,
            Guided(13) => setup && project,
            Guided(15) => {
                results
                    && self.preview.is_none()
                    && self.evidence.latest.as_ref().is_some_and(|run| {
                        matches!(run.run.state, JobState::Paused | JobState::Interrupted)
                    })
            }
            Guided(16) => results && (self.preview.is_some() || self.evidence.baseline.is_some()),
            Guided(17 | 23 | 24) => inventory,
            Guided(19) => results && self.preview.is_none() && self.evidence.latest.is_some(),
            Guided(20..=22) => results && self.preview.is_some(),
            Guided(27 | 31..=38) => setup && project,
            Guided(41 | 42) => settings,
            Guided(_) => false,
            CrawlField(0..=2) => setup,
            CrawlField(3..=8) => setup && self.focus.advanced,
            CrawlField(9) => inventory,
            CrawlField(10..=16) => setup && project,
            CrawlField(17 | 18) => settings,
            CrawlField(_) => false,
            Phase5(0..=5 | 14..=24 | 26) => analysis,
            Phase5(6..=9) => viewer && analysis,
            Phase5(10 | 11) => viewer && self.viewer_mode == 2,
            Phase5(12 | 13) => {
                analysis
                    && self
                        .evidence
                        .comparison
                        .as_ref()
                        .and_then(|comparison| comparison.perceptual.as_ref())
                        .is_some_and(|perceptual| {
                            perceptual
                                .reports
                                .iter()
                                .any(|(key, _)| Some(key) == self.selected_key.as_ref())
                        })
            }
            Phase5(25) => {
                analysis
                    && self
                        .approval
                        .as_ref()
                        .is_some_and(|preview| self.approval_matches(preview))
            }
            Phase5(27 | 28) => analysis && self.analysis_advanced,
            Phase5(30) => results && self.preview.is_some(),
            Phase5(40..=42) => history,
            Phase5(_) => false,
            AnalysisField(0..=8) => analysis && self.analysis_advanced,
            AnalysisField(9) => analysis,
            AnalysisField(_) => false,
            Phase6(0) => screen == Screen::Projects || history,
            Phase6(1) => portability && self.import_preview.is_some(),
            Phase6(2 | 4 | 5 | 8..=12) => history,
            Phase6(3) => portability && self.export_preview.is_some(),
            Phase6(6) => portability && self.deletion_preview.is_some(),
            Phase6(7) => {
                portability
                    && (self.import_preview.is_some()
                        || self.export_preview.is_some()
                        || self.deletion_preview.is_some()
                        || self.repair_preview.is_some())
            }
            Phase6(13) => settings,
            Phase6(14 | 15) => portability && self.repair_preview.is_some(),
            Phase6(_) => false,
            Phase7(0) => settings,
            Phase7(1 | 2) => settings && self.diagnostic_preview.is_some(),
            Phase7(_) => false,
            Choice(choice) => {
                if regressee_core::shell::Choice::SETUP.contains(&choice) {
                    setup
                } else if regressee_core::shell::Choice::ENVIRONMENT.contains(&choice) {
                    setup && project
                } else {
                    results && self.preview.is_none() && self.evidence.latest.is_some()
                }
            }
            Environment(index) => {
                setup && project && index < self.draft_settings.environments.len()
            }
            Details(Detail::Capture) => setup,
            Details(Detail::History) => history,
            Details(Detail::Analysis) => analysis,
            Details(Detail::Network) => settings,
        }
    }
    fn field_enabled(&self, index: usize) -> bool {
        if self.focus.dialog
            || self.native_prompt
            || !self.control_mounted(Control::CrawlField(index))
        {
            return false;
        }
        let enabled = !self.focus.busy;
        match index {
            0..=2 => enabled && self.focus.screen == Screen::Setup,
            3..=8 => enabled && self.focus.screen == Screen::Setup && self.focus.advanced,
            9 => self.focus.screen == Screen::Results && self.project.is_some(),
            10..=16 => enabled && self.focus.screen == Screen::Setup && self.project.is_some(),
            17..=18 => enabled && self.focus.screen == Screen::Settings,
            _ => false,
        }
    }
    fn content(&self, cx: &mut Context<Self>) -> AnyElement {
        match self.focus.screen {
            Screen::Projects => {
                let mut body = div().flex().flex_col().gap_5().child(
                    div()
                        .flex()
                        .items_center()
                        .flex_wrap()
                        .justify_between()
                        .gap_4()
                        .child(self.heading(
                            "Projects",
                            "Website settings and evidence stay on this computer.",
                        ))
                        .child(self.button(
                            Control::NewProject,
                            "Create project",
                            ActionStyle::Primary,
                            cx,
                        )),
                );
                if self.projects.items.is_empty() && !self.focus.busy {
                    body = body.child(controls::empty("Your first project starts here", "Create a project to choose a website and screen sizes. Save a set of screenshots as your baseline, then compare later captures against it."));
                }
                body = body
                    .child(self.button(
                        Control::Phase6(0),
                        "Import baseline ZIP…",
                        ActionStyle::Secondary,
                        cx,
                    ))
                    .child(self.portability_confirmation(cx));
                for (index, project) in self.projects.items.iter().enumerate() {
                    body = body.child(
                        controls::list_row(("project-row", index))
                            .px_3()
                            .bg(rgb(
                                if self
                                    .project
                                    .as_ref()
                                    .is_some_and(|selected| selected.project.id == project.id)
                                {
                                    SELECTED
                                } else {
                                    GROUND
                                },
                            ))
                            .child(
                                div()
                                    .flex_1()
                                    .min_w_0()
                                    .flex()
                                    .flex_col()
                                    .gap_1()
                                    .child(
                                        div()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_ellipsis()
                                            .child(project.name.clone()),
                                    )
                                    .child(
                                        div()
                                            .text_size(px(14.))
                                            .text_color(rgb(MUTED))
                                            .text_ellipsis()
                                            .child(project.address.clone()),
                                    ),
                            )
                            .child(self.button(
                                Control::OpenProject(index),
                                "Open settings",
                                ActionStyle::Secondary,
                                cx,
                            )),
                    );
                }
                body.child(
                    div()
                        .flex()
                        .items_center()
                        .gap_3()
                        .when(self.projects.page > 0, |el| {
                            el.child(self.button(
                                Control::PreviousPage,
                                "Previous",
                                ActionStyle::Secondary,
                                cx,
                            ))
                        })
                        .when(self.projects.more, |el| {
                            el.child(self.button(
                                Control::NextPage,
                                "Next",
                                ActionStyle::Secondary,
                                cx,
                            ))
                        }),
                )
                .into_any_element()
            }
            Screen::Setup => self.crawl_setup(cx),
            Screen::Results => self.results(cx),
            Screen::History => self.baseline_history(cx),
            Screen::Settings => {
                use regressee_app::product::RuntimeState;
                let (browser_label, browser_tone) = match self
                    .runtime_readiness
                    .as_ref()
                    .map(|readiness| readiness.status)
                {
                    None => ("Checking browser setup", Tone::Neutral),
                    Some(RuntimeState::UnavailableDistribution) => {
                        ("Browser setup incomplete in this build", Tone::Warning)
                    }
                    Some(RuntimeState::UnsupportedPlatform) => {
                        ("Browser unavailable on this platform", Tone::Warning)
                    }
                    Some(RuntimeState::NotInstalled) => {
                        ("Browser will install on first use", Tone::Information)
                    }
                    Some(RuntimeState::Ready)
                        if self.runtime_readiness.as_ref().is_some_and(|status| {
                            status.scope
                                == Some(regressee_runtime::distribution::Scope::DevelopmentLoopback)
                        }) =>
                    {
                        ("Development browser — local fixtures only", Tone::Warning)
                    }
                    Some(RuntimeState::Ready) => ("Private browser verified", Tone::Information),
                    Some(RuntimeState::RepairRequired) => {
                        ("Browser needs explicit repair", Tone::Warning)
                    }
                };
                let mut body = div().max_w(px(680.)).flex().flex_col().gap_6().child(self.heading("Settings", "Browser setup, storage and local support tools."))
                      .child(controls::section("Private browser", "Regressee uses its own verified browser components, not your system browser.")
                          .child(controls::status(browser_label, browser_tone))
                          .child(div().text_color(rgb(MUTED)).child(self.runtime_status.clone().unwrap_or_else(||"Checking whether this build can install its private browser. No download is started by opening Settings.".into())))
                          .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("A supported build installs its private Node, Playwright and browser on first use. You do not need to install developer tools. Internet is required for setup."))
                          .child(self.button(Control::SetupRuntime, match self.runtime_readiness.as_ref().map(|status| status.status) {
                              Some(RuntimeState::NotInstalled) => "Install browser now",
                              Some(RuntimeState::Ready | RuntimeState::RepairRequired) => "Repair browser",
                              _ => "Browser setup unavailable",
                          },ActionStyle::Primary,cx)));
                if let Some(summary) = &self.summary {
                    body = body.child(div().border_t_1().border_color(rgb(LINE)).pt_5().flex().flex_col().gap_3().child(div().text_size(px(18.)).font_weight(FontWeight::SEMIBOLD).child("On this computer"))
                        .child(self.setting_line("Saved projects", &summary.projects.to_string()))
                        .child(self.setting_line("Screenshots & data", &format!("{} · {} objects", human_bytes(summary.artifact_bytes), summary.artifacts)))
                        .child(div().text_color(rgb(MUTED)).text_size(px(13.)).child(format!("Project data: {}", summary.paths.data.display())))
                        .child(div().text_color(rgb(MUTED)).text_size(px(13.)).child(format!("Rebuildable cache: {}", summary.paths.cache.display())))
                          .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("Object totals exclude the database and browser/login caches. Use History to inspect storage and deletion impact. Browser repair never deletes screenshot history.")));
                }
                body=body.child(div().border_t_1().border_color(rgb(LINE)).pt_5().max_w(px(680.)).flex().flex_col().gap_3()
                    .child(div().text_size(px(18.)).font_weight(FontWeight::SEMIBOLD).child("Managed network"))
                    .child(self.crawl_field(17,"Proxy server (optional)","Complete http(s) origin. No proxy username/password. Blank uses existing OS networking."))
                    .child(self.crawl_field(18,"Trusted CA PEM file (optional)","Machine-local path for runtime downloads and Node. Chromium must also trust it through its OS/browser trust store; never disable TLS verification."))
                    .child(div().flex().flex_wrap().gap_2().child(self.guided_button(42,"Load saved network settings",cx)).child(self.guided_button(41,"Save network settings",cx)))
                    .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(self.network_status.clone().unwrap_or_else(||"Load saved network settings before editing. Proxy username/password is not supported.".into())))
                    .child(self.button(Control::Details(Detail::Network), if self.details.contains(&Detail::Network) { "Hide network limitations" } else { "Network limitations" }, ActionStyle::Secondary, cx))
                    .when(self.details.contains(&Detail::Network), |el| el.child(div().text_size(px(14.)).text_color(rgb(WARNING)).child("The development browser's Node 22 worker cannot fetch robots.txt through a configured proxy. Use an explicit authorized robots override only where appropriate. VPN, enterprise certificates and production networks are not qualified. Chromium must trust a custom CA in its OS/browser trust store as well; TLS verification is never disabled."))));
                body = body.child(self.diagnostic_panel(cx));
                body = body
                    .child(self.button(
                        Control::Phase6(13),
                        "Inspect interrupted writes / integrity",
                        ActionStyle::Secondary,
                        cx,
                    ))
                    .child(self.portability_confirmation(cx));
                body.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_3()
                        .child(self.button(
                            Control::ChooseFolder,
                            "Choose a folder to inspect…",
                            ActionStyle::Secondary,
                            cx,
                        ))
                        .child(self.button(
                            Control::About,
                            "About this preview",
                            ActionStyle::Secondary,
                            cx,
                        )),
                )
                .into_any_element()
            }
        }
    }
    fn baseline_history(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div()
            .max_w(px(820.))
            .flex()
            .flex_col()
            .gap_4()
            .child(self.heading(
                "History & storage",
                "Your saved baselines and capture runs. Nothing is deleted automatically.",
            ));
        if self.project.is_none() {
            return body.child(controls::empty("Choose a project", "Open a saved project from Projects to see its baselines, capture history and storage.")).into_any_element();
        }
        body = body.child(self.button(
            Control::Details(Detail::History),
            if self.details.contains(&Detail::History) {
                "Hide provenance details"
            } else {
                "Provenance details"
            },
            ActionStyle::Secondary,
            cx,
        ));
        body = body.child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .child(self.button(
                    Control::Phase5(40),
                    "Previous version",
                    ActionStyle::Secondary,
                    cx,
                ))
                .child(self.button(
                    Control::Phase5(41),
                    "Next version",
                    ActionStyle::Secondary,
                    cx,
                ))
                .child(self.button(
                    Control::Phase5(42),
                    "Use this baseline",
                    ActionStyle::Primary,
                    cx,
                ))
                .child(self.button(
                    Control::Phase6(2),
                    "Export baseline ZIP…",
                    ActionStyle::Secondary,
                    cx,
                ))
                .child(self.button(
                    Control::Phase6(4),
                    "Review baseline deletion…",
                    ActionStyle::Danger,
                    cx,
                ))
                .child(self.button(
                    Control::Phase6(0),
                    "Import ZIP…",
                    ActionStyle::Secondary,
                    cx,
                )),
        );
        if let Some(b) = self.evidence.baselines.get(self.history_index) {
            body = body.child(div().child(format!(
                    "Baseline {} of {} · saved {} · {} / {} intended screenshots · {}",
                    self.history_index + 1,
                    self.evidence.baselines.len(),
                    relative_time(b.created_at),
                    b.references.len(),
                    b.intended_count,
                    if self
                        .project
                        .as_ref()
                        .is_some_and(|p| p.project.active_baseline == Some(b.id))
                    {
                        "Active reference"
                    } else {
                        "Retained version"
                    }
                )));
            if b.parent.is_some() {
                body = body.child(controls::status("Composite baseline — references come from more than one capture time, not one simultaneous website snapshot.", Tone::Warning));
            }
            if self.details.contains(&Detail::History) {
                body = body.child(div().text_size(px(13.)).child(format!(
                    "Baseline {} · created {} · parent {:?}",
                    b.id, b.created_at, b.parent
                )));
                for r in b.references.iter().take(64) {
                    body = body.child(div().text_size(px(13.)).child(format!(
                        "Page {} / profile {} · source run {} · captured {} · image {}",
                        r.key.page,
                        r.key.profile,
                        r.run,
                        r.captured_at,
                        r.artifact.as_str()
                    )));
                }
                if b.references.len() > 64 {
                    body = body.child(div().child(format!(
                        "Showing first 64 of {} references; exact full map remains retained.",
                        b.references.len()
                    )));
                }
            }
        } else {
            body = body.child(controls::empty(
                "No saved baseline versions",
                "Capture reviewed pages and explicitly Save baseline first.",
            ));
        }
        body = body.child(self.portability_confirmation(cx));
        if let Some(s) = &self.project_storage {
            body=body.child(div().border_t_1().border_color(rgb(LINE)).pt_5().flex().flex_col().gap_2()
                .child(div().text_size(px(18.)).font_weight(FontWeight::SEMIBOLD).child("Project evidence on disk"))
                .child(format!("{} total · {} originals · {} rebuildable derivatives",human_bytes(s.bytes),human_bytes(s.original_bytes),human_bytes(s.derived_bytes)))
                .child(format!("{} exclusive · {} shared · free space {}",human_bytes(s.exclusive_bytes),human_bytes(s.shared_bytes),s.available_bytes.map(human_bytes).unwrap_or_else(||"unavailable — preflight cannot be claimed".into())))
                .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("Shared originals stay protected. Review a specific run or baseline's deletion impact before deleting; exclusive storage is not automatically deletable.")));
        }
        body = body.child(
            div()
                .flex()
                .flex_wrap()
                .gap_2()
                .child(self.button(
                    Control::Phase6(8),
                    "Refresh storage / history",
                    ActionStyle::Secondary,
                    cx,
                ))
                .child(self.button(
                    Control::Phase6(10),
                    "Previous run",
                    ActionStyle::Secondary,
                    cx,
                ))
                .child(self.button(Control::Phase6(9), "Next run", ActionStyle::Secondary, cx))
                .child(self.button(
                    Control::Phase6(12),
                    "Previous history page",
                    ActionStyle::Secondary,
                    cx,
                ))
                .child(self.button(
                    Control::Phase6(11),
                    "Next history page",
                    ActionStyle::Secondary,
                    cx,
                ))
                .child(self.button(
                    Control::Phase6(5),
                    "Review run deletion…",
                    ActionStyle::Danger,
                    cx,
                )),
        );
        if let Some(h) = &self.history
            && let Some(r) = h.runs.get(self.run_history_index)
        {
            body = body
                .child(div().font_weight(FontWeight::SEMIBOLD).child(format!(
                    "Run {} of {} · {} · {} · {} screenshots",
                    self.run_history_index + 1,
                    h.runs.len(),
                    match r.run.intent {
                        RunIntent::NewBaseline => "Baseline capture",
                        RunIntent::Check { .. } => "Comparison check",
                        RunIntent::RepresentativePreview => "Representative preview",
                    },
                    job_label(r.run.state),
                    r.all_captures().count()
                )))
                .child(if h.imported_runs.contains(&r.run.id) {
                    "Imported evidence — not independently verified on this computer"
                } else {
                    "Recorded locally"
                });
            if self.details.contains(&Detail::History) {
                body = body
                    .child(format!(
                        "Run {} · started {:?} · finished {:?}",
                        r.run.id, r.run.started_at, r.run.finished_at
                    ))
                    .child(format!(
                        "Rendering: {} / {} · browser {} · worker {} · fonts {:?}",
                        r.run.rendering.os,
                        r.run.rendering.architecture,
                        r.run.rendering.browser,
                        r.run.rendering.worker_version,
                        r.run.rendering.fonts_fingerprint
                    ));
                for c in r.all_captures().take(32) {
                    body = body.child(div().text_size(px(13.)).child(format!(
                        "{} → {} · {} {}×{} DPR {} · {:?} · original {}",
                        c.metadata.requested,
                        c.metadata.final_url,
                        c.metadata.profile.name,
                        c.metadata.width,
                        c.metadata.height,
                        c.metadata.profile.dpr,
                        c.metadata.profile.image_mode,
                        c.image.original.as_str()
                    )));
                }
            }
            body=body.child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(format!("History page {}. Complete evidence and the fixed inventory remain retained; provenance details show the first 32 captures.",h.page+1)));
        }
        body.child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("Retained baselines protect their source runs. Replacing the whole baseline requires a new capture and explicit save. Imported ignored areas need revalidation; browser/cache repair is separate.")).into_any_element()
    }
    fn results(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div().flex().flex_col().gap_5().child(self.heading(
            "Results",
            "Review the fixed page inventory and its screen-size coverage. A change is not an automatic defect verdict.",
        ));
        let Some(project) = &self.project else {
            return body.child(controls::empty("Choose a saved project","Open a project or create and save one first. No sample screenshot is presented as your evidence.")).into_any_element();
        };
        body = body
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(project.project.name.clone()),
            )
            .child(
                div()
                    .text_color(rgb(MUTED))
                    .child(project.project.sources[0].to_string()),
            );
        let mut actions = div().flex().flex_wrap().gap_3().child(self.button(
            Control::CaptureBaseline,
            if self.evidence.baseline.is_some() {
                if self.preview.is_some() {
                    "Capture reviewed replacement"
                } else {
                    "Find pages for new baseline"
                }
            } else {
                if self.preview.is_some() {
                    "Capture reviewed pages"
                } else {
                    "Find pages"
                }
            },
            if self.focus.baseline || self.focus.candidate {
                ActionStyle::Secondary
            } else {
                ActionStyle::Primary
            },
            cx,
        ));
        if self.focus.candidate {
            actions = actions.child(self.button(
                Control::SaveBaseline,
                "Save baseline…",
                ActionStyle::Primary,
                cx,
            ));
        }
        if self.focus.baseline {
            actions = actions.child(self.button(
                Control::CheckChanges,
                "Check for changes",
                if self.focus.candidate {
                    ActionStyle::Secondary
                } else {
                    ActionStyle::Primary
                },
                cx,
            ));
        }
        actions = actions.child(self.button(
            Control::RefreshEvidence,
            "Refresh results",
            ActionStyle::Secondary,
            cx,
        ));
        if self.evidence.baseline.is_some() && self.preview.is_none() {
            actions = actions.child(self.guided_button(16, "Review baseline page list", cx));
        }
        body = body.child(actions);
        body = body.child(self.viewer_section(cx));
        body = body.child(self.inventory_and_results(cx));
        if let Some(r) = &self.evidence.latest {
            let target = r
                .run
                .targets
                .iter()
                .find(|t| Some(&t.key) == self.selected_key.as_ref())
                .unwrap_or(&r.run.targets[0]);
            let (mut label, mut tone) = match r.run.state {
                JobState::Completed => {
                    if let Some(a) = &self.evidence.comparison {
                        match a.revision.results[0].classification {
                            Classification::Changed => {
                                ("Changed — review the evidence", Tone::Warning)
                            }
                            Classification::NoSignificantChange => {
                                ("No significant change", Tone::Neutral)
                            }
                            _ => ("Could not compare", Tone::Warning),
                        }
                    } else if matches!(r.run.intent, RunIntent::Check { .. }) {
                        (
                            "Capture kept — analysis pending; start a fresh check",
                            Tone::Warning,
                        )
                    } else if self.focus.candidate {
                        (
                            "Capture complete — explicitly save to activate",
                            Tone::Neutral,
                        )
                    } else {
                        ("Approved baseline capture", Tone::Neutral)
                    }
                }
                JobState::Cancelled => (
                    "Cancelled — evidence kept; baseline unchanged",
                    Tone::Warning,
                ),
                JobState::Interrupted => (
                    "Interrupted — start a fresh capture; evidence kept",
                    Tone::Warning,
                ),
                JobState::CompletedWithErrors => {
                    ("Could not capture — not an unchanged result", Tone::Warning)
                }
                _ => ("Capture not complete", Tone::Warning),
            };
            if r.run.targets.len() > 1 {
                (label, tone) = match r.run.state {
                    JobState::Completed if matches!(r.run.intent, RunIntent::Check { .. }) => (
                        "Check captured — review changed targets and coverage",
                        Tone::Neutral,
                    ),
                    JobState::Completed if self.focus.candidate => (
                        "Capture complete — explicitly save to activate",
                        Tone::Neutral,
                    ),
                    JobState::Completed => ("Saved baseline capture", Tone::Neutral),
                    JobState::Paused => ("Paused — completed evidence kept", Tone::Warning),
                    JobState::Interrupted => (
                        "Interrupted — validate and resume, or start fresh",
                        Tone::Warning,
                    ),
                    JobState::CompletedWithErrors => (
                        "Incomplete coverage — not an unchanged check",
                        Tone::Warning,
                    ),
                    JobState::Cancelled => ("Cancelled — baseline unchanged", Tone::Warning),
                    _ => ("Capture not complete", Tone::Warning),
                };
            }
            body = body.child(controls::status(label, tone));
            if r.run.targets.iter().any(|t| {
                t.outcome
                    == (TargetOutcome::Failed {
                        code: regressee_core::domain::CaptureError::NeedsLogin,
                    })
            }) {
                body = body.child(controls::status(
                    "Log in again — queue paused; login forms are not baseline evidence",
                    Tone::Warning,
                ));
            }
            if self.evidence.comparison.as_ref().is_some_and(|a| {
                a.revision
                    .results
                    .iter()
                    .any(|r| r.classification == Classification::NotComparable)
            }) {
                body=body.child(controls::status("Rendering caution — computer/profile differs or page did not settle; inspect pixels, not a clean release check",Tone::Warning));
            }
            if let TargetOutcome::Failed { code } = &target.outcome {
                body = body.child(controls::error(format!(
                    "{} for the selected target. Previous baseline evidence is unchanged.",
                    code.label()
                )));
            }
            if !target.warnings.is_empty() {
                body = body.child(div().text_color(rgb(MUTED)).child(format!(
                        "Capture warnings: {}",
                        target
                            .warnings
                            .iter()
                            .map(|w| capture_warning_label(w))
                            .collect::<Vec<_>>()
                            .join("; ")
                    )));
            }
            if let Some(c) = r.evidence(&target.key) {
                body = body.child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(MUTED))
                        .child(format!(
                            "Lossless original · {} × {} pixels · {} viewer tiles · {} ms",
                            c.image.width,
                            c.image.height,
                            c.image.tiles.len(),
                            c.metadata.duration_ms
                        )),
                );
                if let Some(replacement) = &c.metadata.replacement {
                    body=body.child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(format!("Replacement-only capture · neutral-raster-v2 · {} transformed images/backgrounds · geometry {} · {} canceled source reloads recorded (not completed loads). Original hidden photos were not separately saved.",replacement["count"].as_u64().unwrap_or(0),match replacement["geometryPreserved"].as_bool(){Some(true)=>"equal for measured light-DOM rectangles",Some(false)=>"changed — unsupported behavior",None=>"not measured"},replacement["cancelledSourceReloads"].as_u64().unwrap_or(0))));
                }
            }
            if let Some(a) = &self.evidence.comparison {
                let r = a
                    .revision
                    .results
                    .iter()
                    .find(|r| r.key == target.key)
                    .unwrap_or(&a.revision.results[0]);
                let percent = if r.eligible_pixels > 0 {
                    r.changed_pixels as f64 / r.eligible_pixels as f64 * 100.
                } else {
                    0.
                };
                body = body.child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(
                    if r.eligible_pixels > 0 {
                        format!(
                            "{} changed / {} compared pixels · {:.4}% · {}",
                            r.changed_pixels, r.eligible_pixels, percent, a.revision.algorithm
                        )
                    } else {
                        "No valid pixels were compared. Capture failure is not an unchanged page."
                            .into()
                    },
                ));
            }
        } else {
            if self.preview.is_none() {
                body=body.child(controls::empty("Ready for your first baseline", "Choose Find pages, review the included pages, then capture and save a baseline. Later checks revisit that same fixed list."));
            }
        }
        if let Some(b) = &self.evidence.baseline {
            body = body.child(
                div()
                    .text_size(px(13.))
                    .text_color(rgb(MUTED))
                    .child(format!(
                        "Active baseline · {} of {} intended screenshot targets",
                        b.references.len(),
                        b.intended_count
                    )),
            );
        }
        body = body.child(self.phase5_panel(cx));
        body.child(div().text_size(px(13.)).text_color(rgb(MUTED)).max_w(px(680.)).child("Sensitivity and ignored rectangles change only analysis, never captured originals. Numerical change is evidence to review, not a defect probability. Fixture-calibrated presets are not proven user accuracy.")).into_any_element()
    }
    fn viewer_section(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(view) = &self.viewer else {
            return div().into_any_element();
        };
        let mut section = div().flex().flex_col().gap_3();
        if let Some(target) = self.evidence.latest.as_ref().and_then(|run| {
            run.run
                .targets
                .iter()
                .find(|target| Some(&target.key) == self.selected_key.as_ref())
        }) {
            section = section.child(
                div()
                    .font_weight(FontWeight::MEDIUM)
                    .text_ellipsis()
                    .child(target.requested.to_string()),
            );
        }
        if self.evidence.comparison.is_some() {
            section = section.child(
                div().flex().flex_wrap().gap_2().children(
                    [
                        (6, "Before & after"),
                        (7, "Difference"),
                        (8, "Overlay"),
                        (9, "Heatmap"),
                    ]
                    .into_iter()
                    .map(|(index, label)| {
                        self.button(
                            Control::Phase5(index),
                            label,
                            ActionStyle::Choice {
                                selected: self.viewer_mode == index - 6,
                            },
                            cx,
                        )
                    }),
                ),
            );
        }
        section = section
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(self.button(Control::ZoomOut, "Zoom out", ActionStyle::Secondary, cx))
                    .child(
                        div()
                            .px_2()
                            .child(format!("{}%", (self.zoom * 100.) as u32)),
                    )
                    .child(self.button(Control::ZoomIn, "Zoom in", ActionStyle::Secondary, cx))
                    .child(self.button(Control::PanLeft, "Pan left", ActionStyle::Secondary, cx))
                    .child(self.button(Control::PanRight, "Pan right", ActionStyle::Secondary, cx))
                    .child(self.button(Control::PanUp, "Pan up", ActionStyle::Secondary, cx))
                    .child(self.button(Control::PanDown, "Pan down", ActionStyle::Secondary, cx)),
            )
            .child(self.viewer_panes(view, cx));
        if self.viewer_mode == 2 {
            section = section.child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(self.button(
                        Control::Phase5(10),
                        "Less after",
                        ActionStyle::Secondary,
                        cx,
                    ))
                    .child(format!("{:.0}% after", self.overlay_fraction * 100.))
                    .child(self.button(
                        Control::Phase5(11),
                        "More after",
                        ActionStyle::Secondary,
                        cx,
                    )),
            );
        }
        section.into_any_element()
    }
    fn phase5_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut panel = div()
            .flex()
            .flex_col()
            .gap_3()
            .border_t_1()
            .border_color(rgb(LINE))
            .pt_4();
        if self.preview.is_some() {
            panel = panel.child(self.button(
                Control::Phase5(30),
                "Preview representative page with saved image mode",
                ActionStyle::Secondary,
                cx,
            ));
        }
        if self
            .evidence
            .latest
            .as_ref()
            .is_some_and(|r| r.run.intent == RunIntent::RepresentativePreview)
        {
            panel=panel.child(controls::status("Representative preview only — cannot be approved as a baseline. Review the full inventory before capture.",Tone::Warning));
        }
        let Some(a) = &self.evidence.comparison else {
            return panel.into_any_element();
        };
        panel = panel.child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Analysis & review"),
        );
        let mut actions = div().flex().flex_wrap().gap_2();
        for (i, label) in [
            (0, "Sensitive"),
            (1, "Balanced"),
            (2, "Relaxed"),
            (3, "Advanced parameters"),
            (4, "Reanalyze originals"),
            (5, "Cancel analysis"),
            (26, "View previous analysis"),
        ] {
            let style = if i <= 2 {
                ActionStyle::Choice {
                    selected: self.analysis_policy
                        == regressee_app::review::preset(
                            [
                                regressee_app::analysis_engine::Preset::Sensitive,
                                regressee_app::analysis_engine::Preset::Balanced,
                                regressee_app::analysis_engine::Preset::Relaxed,
                            ][i],
                        ),
                }
            } else {
                ActionStyle::Secondary
            };
            actions = actions.child(self.button(Control::Phase5(i), label, style, cx));
        }
        panel=panel.child(actions).child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("Sensitivity changes reanalyze the saved originals. Earlier analyses and reviews stay in history."))
            .child(self.button(Control::Details(Detail::Analysis), if self.details.contains(&Detail::Analysis) { "Hide analysis details" } else { "Analysis details" }, ActionStyle::Secondary, cx));
        if self.details.contains(&Detail::Analysis) {
            panel = panel.child(
                div()
                    .text_size(px(13.))
                    .text_color(rgb(MUTED))
                    .child(format!(
                        "Analysis {} · {}",
                        a.revision.id,
                        a.perceptual
                            .as_ref()
                            .map(|p| p.policy.revision.as_str())
                            .unwrap_or("historical RGB policy")
                    )),
            );
        }
        if self.analysis_pending.is_some() || self.analysis_due.is_some() {
            panel=panel.child(controls::status("Reanalysis pending — displayed metrics are from the revision above. Viewer remains interactive.",Tone::Neutral));
        }
        if self.analysis_advanced {
            for (i, label, help) in [
                (
                    0,
                    "Color tolerance (ΔE00)",
                    "Significant pixels exceed this value; not an RGB percentage.",
                ),
                (
                    1,
                    "Whole-page changed percentage",
                    "Blank disables the global criterion.",
                ),
                (
                    2,
                    "Local changed percentage",
                    "Blank disables local windows. Overlapping windows do not add area.",
                ),
                (
                    3,
                    "Local minimum changed pixels",
                    "Both the local count and percentage must meet the thresholds.",
                ),
                (
                    4,
                    "Local minimum eligible pixels",
                    "Windows with less unignored coverage cannot trigger.",
                ),
                (
                    5,
                    "Local window size (pixels)",
                    "1–256, in original capture pixels.",
                ),
                (
                    6,
                    "Local window stride (pixels)",
                    "Must be within size and keep at most four window rows active.",
                ),
                (
                    7,
                    "Connected region minimum pixels",
                    "Blank disables. Four-neighbor actual members, not bounding box area.",
                ),
                (
                    8,
                    "Ignored-coverage warning percentage",
                    "Warnings are not a whole-page verification claim.",
                ),
            ] {
                panel = panel.child(controls::field(
                    label,
                    help,
                    self.analysis_fields[i].clone().into_any_element(),
                ));
            }
            panel=panel.child(self.check_button(Control::Phase5(27), "Review page-height changes", self.analysis_policy.height_change_triggers, cx))
                .child(self.check_button(Control::Phase5(28), "Suppress stationary horizontal RGB blend noise", self.analysis_policy.antialias!=regressee_app::analysis_engine::AntiAliasPolicy::Off, cx))
                .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("Optional suppression may hide a real blend-color edit. It is not a proven font-noise detector. Edits debounce for 300 ms and cancel superseded work."));
        }
        if let Some(p) = &a.perceptual {
            if let Some((_, r)) = p
                .reports
                .iter()
                .find(|(k, _)| Some(k) == self.selected_key.as_ref())
            {
                panel = panel.child(div().font_weight(FontWeight::MEDIUM).child(
                    if r.eligible_pixels > 0 {
                        format!(
                            "{:.2}% changed · {} change regions · {:.2}% ignored",
                            100. * r.changed_pixels as f64 / r.eligible_pixels as f64,
                            r.regions.count,
                            r.ignored_coverage.value().unwrap_or(0.) * 100.
                        )
                    } else {
                        "No comparable pixels".into()
                    },
                ));
                if !r.triggers.is_empty() || !r.gates.is_empty() || !r.warnings.is_empty() {
                    panel = panel.child(div().text_size(px(14.)).text_color(rgb(WARNING)).child(
                        format!(
                            "Review reasons: {}. Comparison limits: {}. Warnings: {}.",
                            if r.triggers.is_empty() {
                                "none".into()
                            } else {
                                r.triggers
                                    .iter()
                                    .map(analysis_trigger_label)
                                    .collect::<Vec<_>>()
                                    .join(", ")
                            },
                            if r.gates.is_empty() {
                                "none".into()
                            } else {
                                r.gates
                                    .iter()
                                    .map(analysis_gate_label)
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            },
                            if r.warnings.is_empty() {
                                "none".into()
                            } else {
                                r.warnings
                                    .iter()
                                    .map(analysis_warning_label)
                                    .collect::<Vec<_>>()
                                    .join("; ")
                            }
                        ),
                    ));
                }
                if self.details.contains(&Detail::Analysis) {
                    panel = panel.child(div().text_size(px(13.)).child(format!("{} significant / {} eligible pixels · {} raw changes · {} suppressed. Ignored {} / {} pixels.", r.changed_pixels, r.eligible_pixels, r.raw_changed_pixels, r.antialias_suppressed_pixels, r.ignored_pixels, r.coordinate_union_pixels)))
                        .child(div().text_size(px(13.)).child(format!("{} qualifying local windows / {}. Before {}×{}, after {}×{}; width {:+}, height {:+}. {} structural pixels ({} eligible).", r.local.qualifying, r.local.windows, r.dimensions.before.width,r.dimensions.before.height,r.dimensions.after.width,r.dimensions.after.height,r.dimensions.width_delta,r.dimensions.height_delta,r.structural_pixels,r.unmasked_structural_pixels)));
                }
                if r.eligible_pixels == 0 {
                    panel = panel.child(controls::status(
                        "No eligible pixels — all content ignored. This is NOT a clean result.",
                        Tone::Warning,
                    ));
                }
                if let Some(window) = &r.local.strongest
                    && self.details.contains(&Detail::Analysis)
                {
                    panel = panel.child(div().text_size(px(13.)).child(format!(
                        "Strongest local window: {} / {} changed at x {}, y {} ({}×{}).",
                        window.changed,
                        window.eligible,
                        window.bounds.x,
                        window.bounds.y,
                        window.bounds.width,
                        window.bounds.height
                    )));
                }
                panel=panel.child(div().flex().gap_2().child(self.button(Control::Phase5(12),"Previous change region",ActionStyle::Secondary,cx)).child(self.button(Control::Phase5(13),"Next change region",ActionStyle::Secondary,cx))).child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(format!("Navigation retains the largest {} regions; {} further regions are counted but not retained.",r.regions.retained.len(),r.regions.omitted)));
            }
            if let Some((_, reason)) = p
                .failures
                .iter()
                .find(|(k, _)| Some(k) == self.selected_key.as_ref())
            {
                panel = panel.child(controls::error(reason.clone()));
            }
        }
        let slider_bounds = std::rc::Rc::new(std::cell::Cell::new(None::<Bounds<Pixels>>));
        let b = slider_bounds.clone();
        let b2 = slider_bounds.clone();
        let b3 = slider_bounds.clone();
        let slider = div()
            .id("overlay-slider")
            .relative()
            .h(px(28.))
            .w_full()
            .bg(rgb(RAIL))
            .border_1()
            .border_color(rgb(LINE))
            .child(
                div()
                    .absolute()
                    .h_full()
                    .w(relative(self.overlay_fraction))
                    .bg(rgb(SELECTED)),
            )
            .child(
                canvas(
                    move |bounds, _, _| {
                        b.set(Some(bounds));
                    },
                    |_, _, _, _| {},
                )
                .absolute()
                .size_full(),
            )
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                    if let Some(b) = b2.get() {
                        this.overlay_fraction =
                            ((event.position.x - b.origin.x) / b.size.width).clamp(0., 1.);
                        cx.notify();
                    }
                }),
            )
            .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
                if event.pressed_button == Some(MouseButton::Left)
                    && let Some(b) = b3.get()
                {
                    this.overlay_fraction =
                        ((event.position.x - b.origin.x) / b.size.width).clamp(0., 1.);
                    cx.notify();
                }
            }));
        panel = panel.when(self.viewer_mode == 2, |panel| {
            panel
                .child(div().text_size(px(13.)).child(format!(
                    "Overlay: {:.0}% after — drag the bar or use Less/More after.",
                    self.overlay_fraction * 100.
                )))
                .child(slider)
        });
        let mut masks = div().flex().flex_wrap().gap_2();
        for (i, label) in [
            (14, "Draw ignored areas"),
            (15, "Select next rectangle"),
            (17, "Redraw selected rectangle"),
            (16, "Delete selected rectangle"),
            (18, "Save ignored areas"),
            (19, "Revalidate for this pair"),
        ] {
            masks = masks.child(self.button(
                Control::Phase5(i),
                label,
                if i == 16 {
                    ActionStyle::Danger
                } else if i == 14 {
                    ActionStyle::Choice {
                        selected: self.mask_draw,
                    }
                } else {
                    ActionStyle::Secondary
                },
                cx,
            ));
        }
        panel=panel.child(masks).child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(format!("{} geometric rectangles · selected {:?} · {}. Drag on either original image; rectangles appear on both. Otherwise drag to pan. A mask is not attached to moving page content.",self.mask_rectangles.len(),self.mask_selected.map(|i|i+1),if self.mask_dirty{"unsaved edits"}else{"saved revision"})));
        if let Some(request) = self.viewer.as_ref().and_then(|v| v.request.as_ref())
            && request.validate().is_err()
        {
            panel=panel.child(controls::status("Saved ignored areas belong to another capture pair or geometry. Inspect and explicitly Revalidate; masks are never stretched.",Tone::Warning));
        }
        panel = panel.child(controls::field(
            "Review note",
            "Saved with the displayed analysis revision, not a metric verdict.",
            self.analysis_fields[9].clone().into_any_element(),
        ));
        let mut review = div().flex().flex_wrap().gap_2();
        for (i, label) in [
            (20, "Mark unreviewed"),
            (21, "Mark intentional"),
            (22, "Mark needs fix"),
            (24, "Preview acceptance impact"),
        ] {
            review =
                review.child(self.button(Control::Phase5(i), label, ActionStyle::Secondary, cx));
        }
        panel = panel.child(review).child(
            self.check_button(
                Control::Phase5(23),
                "Include this target in the next baseline",
                self.selected_key
                    .as_ref()
                    .is_some_and(|key| self.approval_selection.contains(key)),
                cx,
            ),
        );
        for d in self
            .evidence
            .reviews
            .iter()
            .filter(|d| Some(&d.key) == self.selected_key.as_ref())
            .take(8)
        {
            panel = panel.child(div().text_size(px(13.)).child(format!(
                "Review {:?} · analysis {} · {} · {}",
                d.state, d.analysis, d.reviewed_at, d.note
            )));
        }
        panel=panel.child(div().text_size(px(13.)).child(format!("{} targets selected for approval. Failed/missing/incompatible targets will be refused.",self.approval_selection.len())));
        if let Some(p) = &self.approval
            && self.approval_matches(p)
        {
            panel=panel.child(controls::status(format!("Accept {} selected references, inherit {} unchanged references. This composite is NOT one simultaneous site snapshot.",p.selected.len(),p.inherited),Tone::Warning));
            for r in p.baseline.references.iter().take(64) {
                panel = panel.child(div().text_size(px(13.)).child(format!(
                    "{} · page {} / profile {} · source run {} · captured {}",
                    if p.selected.contains(&r.key) {
                        "Selected"
                    } else {
                        "Inherited"
                    },
                    r.key.page,
                    r.key.profile,
                    r.run,
                    r.captured_at
                )));
                if let Some((_, requested, final_url)) =
                    p.provenance.iter().find(|(key, _, _)| *key == r.key)
                {
                    panel = panel.child(
                        div()
                            .text_size(px(13.))
                            .text_color(rgb(MUTED))
                            .child(format!("Captured source {requested} · final {final_url}")),
                    );
                }
            }
            panel = panel.child(self.button(
                Control::Phase5(25),
                "Confirm acceptance into new baseline",
                ActionStyle::Primary,
                cx,
            ));
        }
        panel.into_any_element()
    }
    fn viewer_panes(
        &self,
        view: &regressee_app::jobs::ViewerFrame,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let indices: &[usize] = match self.viewer_mode {
            0 => &[0, 1],
            1 | 3 => &[2],
            _ => &[0],
        };
        let mut panes = div().flex().gap_3();
        let transform = self.viewer_transform();
        for &index in indices {
            let tile_layer = |source: usize, opacity: f32| {
                let mut layer = div().absolute().size_full().opacity(opacity);
                for (i, tile) in view.panes[source].iter().enumerate() {
                    let s = tile.size(0);
                    let (x, y) = transform.source_to_viewport((
                        0.,
                        (view.tile + i) as f32 * regressee_app::images::TILE_HEIGHT as f32,
                    ));
                    layer = layer.child(
                        div()
                            .debug_selector(move || format!("phase5-tile-{index}-{source}-{i}"))
                            .absolute()
                            .left(px(x))
                            .top(px(y))
                            .w(px(s.width.0 as f32 * self.zoom))
                            .h(px(s.height.0 as f32 * self.zoom))
                            .child(img(tile.clone()).size_full()),
                    );
                }
                layer
            };
            let mut viewport = div()
                .id(("comparison-pane", index))
                .debug_selector(move || {
                    match index {
                        0 => "phase5-before",
                        1 => "phase5-after",
                        _ => "phase5-difference",
                    }
                    .into()
                })
                .relative()
                .h(px(280.))
                .overflow_hidden()
                .border_1()
                .border_color(rgb(LINE))
                .bg(rgb(SURFACE))
                .child(tile_layer(
                    if self.viewer_mode == 1 { 1 } else { index },
                    1.,
                ));
            if self.viewer_mode == 1 {
                viewport = viewport.child(tile_layer(2, 0.55));
            }
            if self.viewer_mode == 2 {
                viewport = viewport.child(tile_layer(1, self.overlay_fraction));
            }
            if view.panes[index].is_empty() {
                viewport = viewport.child(div().p_4().child("No saved evidence for this pane"));
            }
            let mut rectangles = self.mask_rectangles.clone();
            if let Some((a, b)) = self.mask_drag
                && let Some(r) = regressee_app::review::rectangle(a, b)
            {
                rectangles.push(r);
            }
            for (i, r) in rectangles.iter().enumerate() {
                let (x, y) = transform.source_to_viewport((r.x as f32, r.y as f32));
                viewport = viewport.child(
                    div()
                        .debug_selector(move || format!("phase5-mask-{index}-{i}"))
                        .absolute()
                        .left(px(x))
                        .top(px(y))
                        .w(px(r.width as f32 * self.zoom))
                        .h(px(r.height as f32 * self.zoom))
                        .child({
                            let color = rgb(if Some(i) == self.mask_selected {
                                ERROR
                            } else {
                                ACCENT
                            });
                            // Paint within the exact scaled rectangle; layout borders
                            // otherwise impose a four-pixel minimum on tiny masks.
                            canvas(
                                |_, _, _| (),
                                move |bounds, _, window, _| {
                                    let width = mask_outline_width(
                                        f32::from(bounds.size.width),
                                        f32::from(bounds.size.height),
                                    );
                                    window.paint_quad(gpui::quad(
                                        bounds,
                                        px(0.),
                                        rgba(0x08786f22),
                                        px(width),
                                        color,
                                        gpui::BorderStyle::Solid,
                                    ));
                                },
                            )
                            .size_full()
                        }),
                );
            }
            let origin = std::rc::Rc::new(std::cell::Cell::new((0., 0.)));
            let o = origin.clone();
            let down = origin.clone();
            let up = origin.clone();
            let mv = origin.clone();
            viewport = viewport
                .child(
                    canvas(
                        move |bounds, _, _| {
                            o.set((f32::from(bounds.origin.x), f32::from(bounds.origin.y)));
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &MouseDownEvent, _, cx| {
                        this.viewer_pointer(
                            0,
                            (f32::from(event.position.x), f32::from(event.position.y)),
                            down.get(),
                            cx,
                        );
                    }),
                )
                .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
                    if event.pressed_button == Some(MouseButton::Left) {
                        this.viewer_pointer(
                            1,
                            (f32::from(event.position.x), f32::from(event.position.y)),
                            mv.get(),
                            cx,
                        );
                    }
                }))
                .on_mouse_up(
                    MouseButton::Left,
                    cx.listener(move |this, event: &MouseUpEvent, _, cx| {
                        this.viewer_pointer(
                            2,
                            (f32::from(event.position.x), f32::from(event.position.y)),
                            up.get(),
                            cx,
                        );
                    }),
                );
            panes = panes.child(
                div()
                    .flex_1()
                    .min_w_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .text_size(px(13.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(match index {
                                0 => {
                                    if self.viewer_mode == 2 {
                                        "Overlay · saved before + after"
                                    } else {
                                        "Before · reference for this check"
                                    }
                                }
                                1 => "After · saved check",
                                _ => {
                                    if self.viewer_mode == 3 {
                                        "Heatmap · red significant, blue ignored"
                                    } else {
                                        "Difference · after with significant changes highlighted"
                                    }
                                }
                            }),
                    )
                    .child(viewport),
            );
        }
        panes.into_any_element()
    }
    fn viewer_transform(&self) -> regressee_app::review::ViewportTransform {
        regressee_app::review::ViewportTransform {
            zoom: self.zoom,
            pan: (self.pan_x, self.pan_y),
        }
    }
    fn viewer_pointer(
        &mut self,
        phase: usize,
        position: (f32, f32),
        origin: (f32, f32),
        cx: &mut Context<Self>,
    ) {
        let (w, h) = self.viewer_dimensions();
        if self.mask_draw {
            let point = self.viewer_transform().capture_point(
                position,
                origin,
                regressee_app::analysis_engine::Dimensions {
                    width: w,
                    height: h,
                },
            );
            match (phase, point) {
                (0, Some(p)) => self.mask_drag = Some((p, p)),
                (1, Some(p)) => {
                    if let Some((_, b)) = &mut self.mask_drag {
                        *b = p;
                    }
                }
                (2, Some(p)) => {
                    if let Some((a, _)) = self.mask_drag.take()
                        && let Some(r) = regressee_app::review::rectangle(a, p)
                    {
                        if let Some(i) = self
                            .mask_selected
                            .filter(|i| *i < self.mask_rectangles.len())
                        {
                            self.mask_rectangles[i] = r;
                        } else if self.mask_rectangles.len() < 1000 {
                            self.mask_rectangles.push(r);
                        }
                        self.mask_dirty = true;
                    }
                }
                (2, None) => self.mask_drag = None,
                _ => (),
            }
        } else {
            match phase {
                0 => self.pan_drag = Some((position, (self.pan_x, self.pan_y))),
                1 => {
                    if let Some((start, pan)) = self.pan_drag {
                        self.pan_x = (pan.0 + (start.0 - position.0) / self.zoom)
                            .clamp(0., w.saturating_sub(1) as f32);
                        self.pan_y = (pan.1 as f32 + (start.1 - position.1) / self.zoom)
                            .clamp(0., h.saturating_sub(1) as f32)
                            as u32;
                    }
                }
                2 => {
                    self.pan_drag = None;
                    self.reload_viewer(cx);
                }
                _ => (),
            }
        }
        cx.notify();
    }
    fn environment_unsaved(&self, cx: &App) -> bool {
        let (Some(p), Some(e)) = (&self.project, &self.environment) else {
            return false;
        };
        if self.environment_draft != Some(e.environment)
            || self.login_ephemeral != e.settings.ephemeral
            || self.draft_settings.environments != p.project.environments
        {
            return true;
        }
        let expected = [
            e.settings
                .login_address
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            e.settings
                .representative
                .as_ref()
                .map(ToString::to_string)
                .unwrap_or_default(),
            e.settings
                .identity_origins
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" "),
        ];
        for (i, value) in expected.iter().enumerate() {
            if self.crawl_fields[10 + i].read(cx).content.trim() != value {
                return true;
            }
        }
        if let Some(profile) = p.project.profiles.first()
            && (self.crawl_fields[15].read(cx).content.trim() != profile.locale
                || self.timezone.read(cx).selected.as_ref() != profile.timezone
                || self
                    .draft_settings
                    .profiles
                    .first()
                    .is_some_and(|draft| draft.dark)
                    != profile.dark)
        {
            return true;
        }
        false
    }
    fn unsaved(&self, cx: &App) -> bool {
        if self.mask_dirty {
            return true;
        }
        if self.environment_unsaved(cx) {
            return true;
        }
        match &self.project {
            Some(record) => {
                self.name.read(cx).content.as_ref() != record.project.name
                    || self.address.read(cx).content.as_ref()
                        != record
                            .project
                            .sources
                            .iter()
                            .map(|u| u.as_str())
                            .collect::<Vec<_>>()
                            .join(" ")
                    || self.draft_settings.discovery != record.project.discovery
                    || self.draft_settings.profiles != record.project.profiles
                    || self
                        .crawl_fields
                        .iter()
                        .take(7)
                        .enumerate()
                        .any(|(i, input)| {
                            let d = &record.project.discovery;
                            let expected = match i {
                                0 => d.page_limit.to_string(),
                                1 => d.section.clone().unwrap_or_default(),
                                2 => d.skipped_text.join(", "),
                                3 => d
                                    .advanced
                                    .max_depth
                                    .map(|n| n.to_string())
                                    .unwrap_or_default(),
                                4 => d
                                    .advanced
                                    .concurrency
                                    .map(|n| n.to_string())
                                    .unwrap_or_default(),
                                5 => d.advanced.per_origin.to_string(),
                                _ => d.advanced.retries.to_string(),
                            };
                            input.read(cx).content.as_ref() != expected
                        })
            }
            None => {
                !self.name.read(cx).content.is_empty() || !self.address.read(cx).content.is_empty()
            }
        }
    }
    fn confirm_switch(&mut self, control: Control, window: &mut Window, cx: &mut Context<Self>) {
        self.native_prompt = true;
        let answer = window.prompt(
            PromptLevel::Warning,
            "Discard unsaved project edits?",
            Some("Saved settings are unchanged. Keep editing to save these changes first."),
            &["Keep editing", "Discard edits"],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            let discard = answer.await == Ok(1);
            let _ = this.update_in(cx, |this, window, cx| {
                this.native_prompt = false;
                if discard {
                    // Reset to the stored snapshot before replaying the explicit
                    // action. The replay cannot prompt recursively.
                    let name = this
                        .project
                        .as_ref()
                        .map(|p| p.project.name.clone())
                        .unwrap_or_default();
                    let address = this
                        .project
                        .as_ref()
                        .map(|p| {
                            p.project
                                .sources
                                .iter()
                                .map(|u| u.as_str())
                                .collect::<Vec<_>>()
                                .join(" ")
                        })
                        .unwrap_or_default();
                    this.name.update(cx, |input, cx| input.set_value(&name, cx));
                    this.address
                        .update(cx, |input, cx| input.set_value(&address, cx));
                    if let Some(p) = &this.project {
                        this.draft_settings = p.project.clone();
                        this.set_crawl_fields(cx);
                    }
                    if let Some(environment) = &this.environment {
                        this.environment_draft = Some(environment.environment);
                        this.login_ephemeral = environment.settings.ephemeral;
                        let values = [
                            environment
                                .settings
                                .login_address
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                            environment
                                .settings
                                .representative
                                .as_ref()
                                .map(ToString::to_string)
                                .unwrap_or_default(),
                            environment
                                .settings
                                .identity_origins
                                .iter()
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                                .join(" "),
                        ];
                        for (index, value) in values.into_iter().enumerate() {
                            this.crawl_fields[10 + index]
                                .update(cx, |input, cx| input.set_value(&value, cx));
                        }
                    }
                    this.mask_rectangles = this
                        .viewer
                        .as_ref()
                        .and_then(|viewer| viewer.request.as_ref())
                        .and_then(|request| request.mask.as_ref())
                        .map(|mask| mask.rectangles.clone())
                        .unwrap_or_default();
                    this.mask_dirty = false;
                    this.mask_drag = None;
                    this.mask_selected = None;
                    if control == Control::NewProject {
                        this.create(&CreateProject, window, cx);
                    } else {
                        this.activate_control(control, window, cx);
                    }
                }
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }
    fn heading(&self, title: &'static str, subtitle: &'static str) -> Div {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .text_size(px(28.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(title),
            )
            .child(
                div()
                    .text_size(px(15.))
                    .text_color(rgb(MUTED))
                    .max_w(px(680.))
                    .child(subtitle),
            )
    }
    fn setting_line(&self, label: &'static str, value: &str) -> Div {
        div()
            .flex()
            .gap_4()
            .child(div().w(px(136.)).text_color(rgb(MUTED)).child(label))
            .child(value.to_owned())
    }
}
impl Drop for Shell {
    fn drop(&mut self) {
        if let Some((_, cancel)) = &self.analysis_pending {
            cancel.store(true, std::sync::atomic::Ordering::Release);
        }
        self.jobs.stop();
    }
}
impl Render for Shell {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let render_started = std::time::Instant::now();
        let navigation = self.navigation_started.take();
        // RenderImage uses GPUI's sprite atlas: dropping only an Arc does not
        // remove the GPU entry. Evict retired tile IDs before painting new ones.
        for image in self.retired_images.drain(..) {
            let _ = window.drop_image(image);
        }
        let enabled = !self.focus.busy && !self.focus.dialog && !self.native_prompt;
        self.name.update(cx, |input, _| input.set_enabled(enabled));
        self.address
            .update(cx, |input, _| input.set_enabled(enabled));
        for (i, input) in self.crawl_fields.iter().enumerate() {
            input.update(cx, |input, _| input.set_enabled(self.field_enabled(i)));
        }
        let timezone_enabled = self.field_enabled(16);
        if self.timezone.read(cx).is_enabled() != timezone_enabled {
            self.timezone.update(cx, |select, cx| {
                select.set_enabled(timezone_enabled);
                cx.notify();
            });
        }
        for (i, input) in self.analysis_fields.iter().enumerate() {
            let enabled = self.control_enabled(Control::AnalysisField(i), cx);
            input.update(cx, |input, _| input.set_enabled(enabled));
        }
        let mut feedback = div()
            .px_8()
            .py_3()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_3();
        if self.focus.busy {
            let (message, completed, total) = self
                .progress
                .clone()
                .unwrap_or_else(|| ("Working locally…".into(), 0, None));
            let mut progress = div().flex().flex_wrap().items_center().gap_3().child(
                div()
                    .flex_1()
                    .min_w(px(160.))
                    .child(controls::progress(message, completed, total)),
            );
            if self.focus.capturing {
                if self.focus.pausable {
                    progress = progress.child(self.button(
                        Control::PauseCapture,
                        "Pause at a safe boundary",
                        ActionStyle::Secondary,
                        cx,
                    ));
                }
                progress = progress.child(self.button(
                    Control::CancelCapture,
                    "Cancel — keep completed evidence",
                    ActionStyle::Secondary,
                    cx,
                ));
            }
            feedback = feedback.child(progress);
        }
        if self.error.is_some() || self.notice.is_some() {
            feedback = feedback.child(controls::feedback(
                self.error.clone().map(Into::into),
                self.notice.clone().map(Into::into),
                self.button(
                    Control::DismissFeedback,
                    "Dismiss",
                    ActionStyle::Secondary,
                    cx,
                ),
            ));
        }
        let mut content = div()
            .id("working-area")
            .flex_1()
            .min_w_0()
            .h_full()
            .flex()
            .flex_col()
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .justify_between()
                    .gap_3()
                    .px_8()
                    .py_4()
                    .bg(rgb(SURFACE))
                    .border_b_1()
                    .border_color(rgb(LINE))
                    .child(
                        div()
                            .min_w_0()
                            .flex_1()
                            .font_weight(FontWeight::MEDIUM)
                            .text_ellipsis()
                            .child(
                                self.project
                                    .as_ref()
                                    .map(|project| project.project.name.clone())
                                    .unwrap_or_else(|| "Your local workspace".into()),
                            ),
                    )
                    .child(controls::status(
                        if self.focus.screen == Screen::Setup && self.unsaved(cx) {
                            "Unsaved settings"
                        } else {
                            "Private development preview"
                        },
                        Tone::Neutral,
                    )),
            )
            .when(
                self.focus.busy || self.error.is_some() || self.notice.is_some(),
                |el| el.child(feedback),
            )
            .child(
                div()
                    .id("screen-scroll")
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.screen_scroll)
                    .px_8()
                    .py_6()
                    .child(self.content(cx)),
            );
        if self.focus.screen == Screen::Setup {
            content = content.child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_3()
                    .px_8()
                    .py_4()
                    .bg(rgb(SURFACE))
                    .border_t_1()
                    .border_color(rgb(LINE))
                    .child(self.button(Control::Save, "Save settings", ActionStyle::Primary, cx))
                    .when(self.project.is_some(), |el| {
                        el.child(self.guided_button(13, "Find pages", cx))
                    })
                    .child(
                        div()
                            .min_w(px(180.))
                            .flex_1()
                            .text_size(px(13.))
                            .text_color(rgb(MUTED))
                            .child(if self.unsaved(cx) {
                                "Save before finding pages or logging in."
                            } else {
                                "Saved settings do not change an existing baseline."
                            }),
                    ),
            );
        }
        let shell = div().id("regressee-shell").key_context("Shell").track_focus(&self.root_focus).relative().size_full().flex().bg(rgb(GROUND)).text_color(rgb(INK)).text_size(px(15.)).font_family(FONT)
            .on_action(cx.listener(|this, _: &Tab, window, cx| this.move_focus(false, window, cx)))
            .on_action(cx.listener(|this, _: &TabPrev, window, cx| this.move_focus(true, window, cx)))
            .on_action(cx.listener(Self::activate)).on_action(cx.listener(Self::create)).on_action(cx.listener(Self::save)).on_action(cx.listener(Self::dismiss))
            .on_action(cx.listener(|this, _: &ProjectsScreen, w, cx| this.navigate(Screen::Projects, w, cx)))
            .on_action(cx.listener(|this, _: &SetupScreen, w, cx| this.navigate(Screen::Setup, w, cx)))
            .on_action(cx.listener(|this, _: &ResultsScreen, w, cx| this.navigate(Screen::Results, w, cx)))
            .on_action(cx.listener(|this, _: &HistoryScreen, w, cx| this.navigate(Screen::History, w, cx)))
            .on_action(cx.listener(|this, _: &SettingsScreen, w, cx| this.navigate(Screen::Settings, w, cx)))
            .child(div().w(px(204.)).flex_shrink_0().h_full().p_4().bg(rgb(RAIL)).border_r_1().border_color(rgb(LINE)).flex().flex_col().gap_2()
                 .child(div().pt_2().pb_6().px_3().flex().flex_col().gap_1().child(div().text_size(px(21.)).font_weight(FontWeight::SEMIBOLD).child("Regressee")).child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("Visual regression, locally")))
                .children(Screen::ALL.into_iter().map(|screen| self.button(Control::Navigate(screen), screen.label(), ActionStyle::Navigation { selected: self.focus.screen == screen }, cx)))
                .child(div().flex_1())
                 .child(div().px_3().text_size(px(13.)).text_color(rgb(MUTED)).child("Local multi-page preview"))
                 .child(div().px_3().text_size(px(13.)).text_color(rgb(MUTED)).child("Public runtime qualification pending")))
            .child(content)
              .when(self.focus.dialog, |el| el.child(controls::dialog("About this preview", "Regressee finds rendered page links within approved websites, captures chosen screen sizes, preserves originals, and checks only an explicitly saved baseline inventory. Login setup, analysis/review and selected-baseline sharing are implemented locally. Development browsers remain owned-loopback-only. Public runtime, native rendering, accessibility and human qualification are pending; macOS capture is blocked by an ownership code gap. MPL-2.0.", self.button(Control::CloseDialog, "Close", ActionStyle::Primary, cx))));
        shell.when_some(navigation, |shell, (screen, navigation)| {
            let timing = regressee_app::ui::performance::FrameTiming {
                screen: screen.label(),
                navigation,
                render: render_started,
                construction: render_started.elapsed(),
            };
            timing.report("shell-constructed");
            shell.child(
                canvas(
                    move |_, _, _| {
                        timing.report("layout-and-prepaint-complete");
                        timing
                    },
                    |_, timing, _, _| timing.report("cpu-paint-complete"),
                )
                .absolute()
                .inset_0(),
            )
        })
    }
}
fn human_bytes(bytes: u64) -> String {
    let units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB", "EiB"];
    let mut value = bytes as f64;
    let mut unit = 0;
    while value >= 1024. && unit + 1 < units.len() {
        value /= 1024.;
        unit += 1;
    }
    if unit == 0 {
        format!("{bytes} B")
    } else {
        format!("{value:.1} {}", units[unit])
    }
}
fn relative_time(timestamp: i64) -> String {
    let age = regressee_runtime::distribution::now().saturating_sub(timestamp);
    if age < 0 {
        return "at a future recorded date".into();
    }
    match age {
        0..=59 => "just now".into(),
        60..=3599 => format!("{} min ago", age / 60),
        3600..=86399 => format!("{} h ago", age / 3600),
        86400..=31535999 => format!("{} days ago", age / 86400),
        _ => format!("{} years ago", age / 31536000),
    }
}
fn job_label(state: JobState) -> &'static str {
    match state {
        JobState::Queued => "Queued",
        JobState::Running => "Running",
        JobState::Pausing => "Pausing",
        JobState::Paused => "Paused",
        JobState::Completed => "Complete",
        JobState::CompletedWithErrors => "Incomplete coverage",
        JobState::Cancelled => "Cancelled",
        JobState::Interrupted => "Interrupted",
    }
}
fn main() -> anyhow::Result<()> {
    let inspect_ui = std::env::args_os().any(|argument| argument == "--inspect-ui");
    if inspect_ui {
        anyhow::ensure!(
            cfg!(all(feature = "ui-inspection", target_os = "linux")),
            "native inspection requires the Linux development inspection feature"
        );
        anyhow::ensure!(
            std::env::var_os("REGRESSEE_DEV_DATA").is_some()
                && std::env::var_os("REGRESSEE_NATIVE_INSPECTION_DIR").is_some(),
            "native inspection requires separate explicit synthetic data and inspection roots"
        );
    }
    let check_renderer = std::env::args_os()
        .skip(1)
        .any(|argument| argument == "--check-renderer");
    let verbose_renderer = check_renderer
        || std::env::var_os("REGRESSEE_GPU_DIAGNOSTICS").is_some_and(|value| value == "1");
    regressee_app::ui::gpu_diagnostics::initialize(verbose_renderer);
    // Initialize the real GPUI backend BEFORE opening any project store or
    // background job owner. This also makes --check-renderer use the same
    // executable/backend as an ordinary launch, without touching project data.
    let application = Application::new();
    if check_renderer {
        println!(
            "GPUI backend initialized. No project data or browsers were opened. Window rendering/input is NOT verified by this check."
        );
        return Ok(());
    }
    // Path discovery is pure OS/environment resolution; all preparation, SQLite,
    // artifact and image I/O belongs to the background owner.
    let paths = match std::env::var_os("REGRESSEE_DEV_DATA") {
        Some(root) => AppPaths::isolated(std::path::Path::new(&root)),
        None => AppPaths::discover()?,
    };
    let (jobs, events) = Jobs::start(paths);
    let after_ui = jobs.clone();
    application.run(move |cx: &mut App| {
        text_input::bind(cx);
        timezone_select::bind(cx);
        cx.bind_keys([
            KeyBinding::new("tab", Tab, Some("Shell")),
            KeyBinding::new("shift-tab", TabPrev, Some("Shell")),
            KeyBinding::new("enter", Activate, Some("Shell")),
            KeyBinding::new("space", Activate, Some("Shell && !TextInput")),
            KeyBinding::new("escape", Dismiss, Some("Shell")),
            KeyBinding::new("ctrl-n", CreateProject, Some("Shell")),
            KeyBinding::new("cmd-n", CreateProject, Some("Shell")),
            KeyBinding::new("ctrl-s", SaveProject, Some("Shell")),
            KeyBinding::new("cmd-s", SaveProject, Some("Shell")),
            KeyBinding::new("ctrl-q", Quit, None),
            KeyBinding::new("cmd-q", Quit, None),
            KeyBinding::new("ctrl-1", ProjectsScreen, Some("Shell")),
            KeyBinding::new("ctrl-2", SetupScreen, Some("Shell")),
            KeyBinding::new("ctrl-3", ResultsScreen, Some("Shell")),
            KeyBinding::new("ctrl-4", HistoryScreen, Some("Shell")),
            KeyBinding::new("ctrl-5", SettingsScreen, Some("Shell")),
            KeyBinding::new("cmd-1", ProjectsScreen, Some("Shell")),
            KeyBinding::new("cmd-2", SetupScreen, Some("Shell")),
            KeyBinding::new("cmd-3", ResultsScreen, Some("Shell")),
            KeyBinding::new("cmd-4", HistoryScreen, Some("Shell")),
            KeyBinding::new("cmd-5", SettingsScreen, Some("Shell")),
        ]);
        let stop = jobs.clone();
        cx.on_action(move |_: &Quit, cx| {
            stop.stop();
            cx.quit();
        });
        cx.set_menus(vec![Menu {
            name: "Regressee".into(),
            items: vec![
                MenuItem::action("Create project", CreateProject),
                MenuItem::action("Save project", SaveProject),
                MenuItem::separator(),
                MenuItem::action("Quit", Quit),
            ],
        }]);
        let stop = jobs.clone();
        cx.on_window_closed(move |cx| {
            if cx.windows().is_empty() {
                stop.stop();
                cx.quit();
            }
        })
        .detach();
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                    None,
                    size(px(1120.), px(800.)),
                    cx,
                ))),
                window_min_size: Some(size(px(820.), px(600.))),
                titlebar: Some(TitlebarOptions {
                    title: Some("Regressee — Multi-page preview".into()),
                    ..Default::default()
                }),
                ..Default::default()
            },
            |window, cx| {
                // The app uses a fixed font database. Keep a bounded set of
                // shaped lines across screens instead of reshaping Setup on
                // every return; wrapped layouts still use GPUI's frame cache.
                window
                    .text_system()
                    .retain_unwrapped_layouts(2048, 8 * 1024 * 1024);
                let shell = cx.new(|cx| Shell::new(cx, jobs, events));
                window.focus(
                    &shell
                        .read(cx)
                        .handle(Control::Navigate(Screen::Projects), cx),
                );
                #[cfg(all(feature = "ui-inspection", target_os = "linux"))]
                if inspect_ui {
                    ui_inspection::start(shell.clone(), window, cx);
                }
                shell
            },
        )
        .expect("open the native Regressee window");
        cx.activate(true);
    });
    after_ui.join_after_ui();
    Ok(())
}

#[cfg(all(test, feature = "ui-tests"))]
#[path = "../tests/support/approval_fixture.rs"]
mod approval_fixture;

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    #[gpui::test]
    fn retained_navigation_text_is_exact_and_resettable(cx: &mut TestAppContext) {
        struct OtherScreen;
        impl Render for OtherScreen {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().child("An unrelated screen")
            }
        }
        let (_, cx) = cx.add_window_view(|_, _| OtherScreen);
        cx.update(|window, cx| {
            let text: SharedString = "Retained navigation geometry".into();
            let runs = [gpui::TextRun {
                len: text.len(),
                font: gpui::font(FONT),
                color: rgb(INK).into(),
                background_color: None,
                underline: None,
                strikethrough: None,
            }];
            let shape = |window: &Window, size, width| {
                window
                    .text_system()
                    .shape_line(text.clone(), size, &runs, width)
            };
            window
                .text_system()
                .retain_unwrapped_layouts(2048, 8 * 1024 * 1024);
            let first = shape(window, px(14.), None);
            let _ = window.draw(cx);
            let _ = window.draw(cx);
            let returned = shape(window, px(14.), None);
            assert!(
                Arc::ptr_eq(&*first, &*returned),
                "reuse across unrelated frames"
            );
            let resized = shape(window, px(15.), None);
            let forced = shape(window, px(14.), Some(px(8.)));
            assert!(!Arc::ptr_eq(&*first, &*resized));
            assert!(!Arc::ptr_eq(&*first, &*forced));
            // Reset releases side references, not live shapes or frame geometry.
            window
                .text_system()
                .retain_unwrapped_layouts(0, 8 * 1024 * 1024);
            let _ = window.draw(cx);
            let _ = window.draw(cx);
            let reset = shape(window, px(14.), None);
            assert!(!Arc::ptr_eq(&*first, &*reset));
            assert_eq!(first.width, returned.width);
            // A deliberately tiny byte budget cannot admit this shape.
            window.text_system().retain_unwrapped_layouts(2048, 1);
            let _ = window.draw(cx);
            let _ = window.draw(cx);
            let oversized = shape(window, px(14.), None);
            let _ = window.draw(cx);
            let _ = window.draw(cx);
            assert!(!Arc::ptr_eq(&*oversized, &*shape(window, px(14.), None)));
        });
    }
    /// Times the real Shell navigation/layout/CPU scene path on GPUI's test
    /// platform. No compositor, GPU presentation, or native font claim.
    #[gpui::test]
    fn setup_cpu_render_profile(cx: &mut TestAppContext) {
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if std::env::var_os("REGRESSEE_PROFILE_REAL_FONTS").is_some_and(|value| value == "1") {
            cx.enable_offscreen_renderer().unwrap();
        }
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let mut store = regressee_storage::Store::open(paths.clone()).unwrap();
        let project =
            Project::draft("Setup CPU timing fixture".into(), "http://127.0.0.1:43210/").unwrap();
        store.save_project(&project, None).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths);
        let owned = jobs.clone();
        let retain_text =
            std::env::var_os("REGRESSEE_PROFILE_RETAIN_TEXT").is_some_and(|value| value == "1");
        let (shell, cx) = cx.add_window_view(|window, cx| {
            if retain_text {
                window
                    .text_system()
                    .retain_unwrapped_layouts(2048, 8 * 1024 * 1024);
            }
            Shell::new(cx, jobs, events)
        });
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_request(s, cx);
                s.activate_control(Control::OpenProject(0), window, cx);
                while s.pending.is_some() {
                    finish_request(s, cx);
                }
            });
        });
        for (width, height) in [(1120., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            for advanced in [false, true] {
                shell.update(cx, |s, cx| {
                    s.focus.advanced = advanced;
                    cx.notify();
                });
                for cycle in 0..3 {
                    for screen in [Screen::Projects, Screen::Setup] {
                        cx.update(|window, cx| {
                            // Isolate the true cold shape cost from subsequent
                            // returns. Reset only the side cache, not frame state.
                            let cold = screen == Screen::Setup && cycle == 0;
                            if cold && retain_text {
                                window.text_system().retain_unwrapped_layouts(2048, 8 * 1024 * 1024);
                            }
                            let started = std::time::Instant::now();
                            shell.update(cx, |s, cx| s.navigate(screen, window, cx));
                            let handler = started.elapsed();
                            let draw_started = std::time::Instant::now();
                            let _ = window.draw(cx);
                            eprintln!(
                                "CPU profile: screen={} size={}x{} advanced={} cycle={} cold_after_cache_reset={} handler_ms={:.3} draw_ms={:.3}",
                                screen.label(), width, height, advanced, cycle, cold && retain_text,
                                handler.as_secs_f64() * 1000.,
                                draw_started.elapsed().as_secs_f64() * 1000.,
                            );
                            shell.update(cx, |s, cx| {
                                while s.pending.is_some() { finish_request(s, cx); }
                            });
                        });
                    }
                }
            }
        }
        owned.join_after_ui();
    }
    #[gpui::test]
    fn projects_import_label_is_contained_at_real_window_sizes(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let mut store = regressee_storage::Store::open(paths.clone()).unwrap();
        let project =
            Project::draft("Visual layout fixture".into(), "https://example.test/").unwrap();
        store.save_project(&project, None).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths);
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, finish_request);
        for (width, height) in [(1120., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let button = cx.debug_bounds("action-Phase6(0)").unwrap();
            let label = cx.debug_bounds("action-label-Phase6(0)").unwrap();
            println!("{width}x{height}: import button {button:?}; label {label:?}");
            assert!(
                label.size.width > px(100.),
                "label must not collapse to a character column"
            );
            assert_eq!(
                label.size.height,
                px(20.),
                "short import label must remain on one line"
            );
            assert!(label.origin.x >= button.origin.x && label.origin.y >= button.origin.y);
            assert!(label.origin.x + label.size.width <= button.origin.x + button.size.width);
            assert!(label.origin.y + label.size.height <= button.origin.y + button.size.height);
            assert!(
                button.size.width < px(240.),
                "import is a compact action, not a full-width field"
            );
        }
        owned.join_after_ui();
    }
    #[gpui::test]
    fn setup_help_and_timezone_are_contained_in_the_scrolling_form(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let mut store = regressee_storage::Store::open(paths.clone()).unwrap();
        let project = Project::draft("Layout regression".into(), "https://example.test/").unwrap();
        store.save_project(&project, None).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths);
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                while s.pending.is_some() {
                    finish_request(s, cx);
                }
                s.activate_control(Control::OpenProject(0), window, cx);
                while s.pending.is_some() {
                    finish_request(s, cx);
                }
                s.focus.advanced = true;
                s.navigate(Screen::Setup, window, cx);
                while s.pending.is_some() {
                    finish_request(s, cx);
                }
            })
        });
        for (width, height) in [(1120., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let button = cx.debug_bounds("timezone-select-button").unwrap();
            let arrow = cx.debug_bounds("timezone-chevron").unwrap();
            let value = cx.debug_bounds("timezone-value").unwrap();
            let hint = cx.debug_bounds("environment-login-hint").unwrap();
            let next = cx.debug_bounds("action-Details(Capture)").unwrap();
            assert!(
                arrow.right() >= button.right() - px(16.),
                "button={button:?}, value={value:?}, arrow={arrow:?}, hint={hint:?}, next={next:?}"
            );
            assert!(
                hint.size.height >= px(20.) && hint.bottom() + px(8.) <= next.top(),
                "hint={hint:?}, next={next:?}"
            );
        }
        owned.join_after_ui();
    }
    #[gpui::test]
    fn feedback_dismiss_is_attached_and_long_errors_leave_a_working_viewport(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, finish_request);
        for (width, height) in [(1120., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            for failed in [false, true] {
                shell.update(cx, |s, cx| {
                    s.error = failed
                        .then(|| "A long readable failure with recovery instructions. ".repeat(80));
                    s.notice = (!failed).then(|| "Settings saved locally.".into());
                    cx.notify();
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let surface = cx.debug_bounds("feedback-surface").unwrap();
                let copy = cx.debug_bounds("feedback-copy").unwrap();
                let dismiss = cx.debug_bounds("action-DismissFeedback").unwrap();
                assert!(dismiss.left() >= surface.left() + px(16.));
                assert!(dismiss.right() <= surface.right() - px(16.));
                assert!(dismiss.top() >= surface.top() + px(16.));
                assert!(dismiss.bottom() <= surface.bottom() - px(16.));
                assert!(copy.right() <= dismiss.left() - px(16.));
                assert!(copy.size.height <= px(160.));
                assert!(surface.size.height <= px(194.));
                assert!(
                    surface.bottom() < px(height - 250.),
                    "feedback must not consume the working screen"
                );
            }
        }
        owned.join_after_ui();
    }
    #[gpui::test]
    fn browser_setup_actions_use_backend_capability_not_environment_presence(
        cx: &mut TestAppContext,
    ) {
        use regressee_app::product::{RuntimeReadiness, RuntimeState};
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Settings;
            assert!(!s.control_enabled(Control::SetupRuntime, cx));
            for (status, setup_available) in [
                (RuntimeState::UnavailableDistribution, false),
                (RuntimeState::UnsupportedPlatform, false),
                (RuntimeState::NotInstalled, true),
                (RuntimeState::Ready, true),
                (RuntimeState::RepairRequired, true),
            ] {
                s.runtime_readiness = Some(RuntimeReadiness {
                    status,
                    setup_available,
                    message: "Synthetic capability fixture only.",
                    identity: None,
                    scope: None,
                });
                assert_eq!(
                    s.control_enabled(Control::SetupRuntime, cx),
                    setup_available
                );
            }
            s.focus.busy = true;
            assert!(!s.control_enabled(Control::SetupRuntime, cx));
        });
        owned.join_after_ui();
    }
    #[test]
    fn human_storage_units_are_bounded_and_unambiguous() {
        assert_eq!(human_bytes(0), "0 B");
        assert_eq!(human_bytes(1023), "1023 B");
        assert_eq!(human_bytes(1024), "1.0 KiB");
        assert_eq!(human_bytes(1024 * 1024), "1.0 MiB");
        assert_eq!(human_bytes(u64::MAX), "16.0 EiB");
    }
    #[test]
    #[ignore = "explicit private native visual fixture preparation; no browser capture evidence"]
    fn prepare_native_ui_sweep_fixture() {
        let root = std::env::var_os("REGRESSEE_NATIVE_UI_FIXTURE")
            .expect("supply a fresh physical private fixture root");
        let root = std::path::PathBuf::from(root);
        assert!(!root.exists(), "never overwrite prior evidence");
        std::fs::create_dir(&root).unwrap();
        let (store, _, _, _, _) = approval_fixture::fixture(&root);
        drop(store);
        println!(
            "Prepared synthetic UI pixels/metadata only: {}",
            root.display()
        );
    }
    #[gpui::test]
    fn ui_sweep_focus_handles_and_field_availability_are_independent(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Setup;
            s.queue(Work::Load(project), cx);
            for _ in 0..8 {
                if s.pending.is_some() {
                    finish_request(s, cx);
                }
            }
            let controls = [
                Control::Phase5(40),
                Control::Phase5(41),
                Control::Phase5(42),
                Control::Phase6(0),
                Control::Phase6(1),
                Control::Phase6(2),
            ];
            for (index, first) in controls.iter().enumerate() {
                for second in &controls[index + 1..] {
                    assert_ne!(s.handle(*first, cx), s.handle(*second, cx));
                }
            }
            assert!(!s.focus.advanced);
            for index in 10..=16 {
                assert!(s.field_enabled(index));
            }
            for index in 3..=8 {
                assert!(!s.field_enabled(index));
            }
            s.focus.screen = Screen::Settings;
            assert!(s.field_enabled(17) && s.field_enabled(18));
            assert!(!s.field_enabled(10));
            s.focus.busy = true;
            assert!(!s.field_enabled(17));
            s.focus.busy = false;
            s.focus.screen = Screen::Results;
            assert!(!s.control_enabled(Control::AnalysisField(0), cx));
            assert!(!s.control_enabled(Control::Phase5(5), cx));
            assert!(!s.control_enabled(Control::Guided(32), cx));
            assert!(!s.control_enabled(Control::Guided(18), cx));
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn ui_sweep_one_save_preserves_website_environment_and_baseline(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        let baseline = store.load_project(project).unwrap().project.active_baseline;
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Setup;
            s.queue(Work::Load(project), cx);
            for _ in 0..8 {
                if s.pending.is_some() {
                    finish_request(s, cx);
                }
            }
            s.name
                .update(cx, |input, cx| input.set_value("Renamed UI fixture", cx));
            s.crawl_fields[0].update(cx, |input, cx| input.set_value("250", cx));
            s.crawl_fields[15].update(cx, |input, cx| input.set_value("nb-NO", cx));
            s.choose(Choice::LoginRequired, cx);
            s.choose(Choice::EphemeralLogin, cx);
            s.choose(Choice::Fast, cx);
            s.choose(Choice::Fast, cx); // Explicit choices are idempotent, not cycles.
            s.save_settings(cx);
            for _ in 0..8 {
                if s.pending.is_some() {
                    finish_request(s, cx);
                }
            }
            assert!(s.error.is_none(), "{:?}", s.error);
            let saved = &s.project.as_ref().unwrap().project;
            assert_eq!(saved.name, "Renamed UI fixture");
            assert_eq!(saved.discovery.page_limit, 250);
            assert_eq!(saved.discovery.speed, Speed::Fast);
            assert_eq!(saved.profiles[0].locale, "nb-NO");
            assert_eq!(
                saved.environments[0].access,
                regressee_core::domain::AccessPolicy::InteractiveLogin
            );
            assert_eq!(saved.active_baseline, baseline);
            assert!(s.environment.as_ref().unwrap().settings.ephemeral);
            assert!(!s.unsaved(cx));
            s.focus.screen = Screen::Results;
            s.mask_dirty = true;
        });
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::RefreshEvidence, window, cx);
                assert!(s.mask_dirty);
                assert!(s.pending.is_none());
                assert!(
                    s.error
                        .as_ref()
                        .unwrap()
                        .contains("unsaved rectangles are kept")
                );
            })
        });
        owned.join_after_ui();
        let store = regressee_storage::Store::open(paths).unwrap();
        assert_eq!(
            store.load_project(project).unwrap().project.active_baseline,
            baseline
        );
    }
    #[gpui::test]
    fn ui_sweep_enabled_candidates_match_actual_mounted_tab_handles(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        let saved_evidence = shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Results;
            s.queue(Work::Load(project), cx);
            for _ in 0..8 {
                if s.pending.is_some() {
                    finish_request(s, cx);
                }
            }
            assert!(s.evidence.comparison.is_some());
            s.evidence.clone()
        });
        // Observe GPUI's real rendered-frame Tab map, not another copy of the
        // mount predicates. These are simulated UI tests, not native pixels.
        for case in [
            "empty-results",
            "baseline-viewer",
            "discovery-preview",
            "comparison",
            "advanced-comparison",
            "loading-comparison",
            "history",
            "settings",
            "projects",
            "setup",
            "advanced-setup",
        ] {
            shell.update(cx, |s, cx| {
                s.evidence = saved_evidence.clone();
                s.preview = None;
                s.focus.preview = false;
                s.focus.busy = false;
                s.focus.view_loading = false;
                s.focus.viewer = true;
                s.focus.candidate = false;
                s.focus.advanced = false;
                s.analysis_advanced = false;
                s.focus.screen = Screen::Results;
                s.viewer = Some(regressee_app::jobs::ViewerFrame {
                    tile: 0,
                    panes: vec![vec![]; 3],
                    width: 96,
                    height: 64,
                    request: None,
                    analysis: None,
                });
                match case {
                    "empty-results" => {
                        s.evidence = ProjectEvidence::default();
                        s.viewer = None;
                        s.focus.viewer = false;
                    }
                    "baseline-viewer" => s.evidence.comparison = None,
                    "discovery-preview" => {
                        s.preview = Some(DiscoveryPreview {
                            project_revision: s.project.as_ref().unwrap().revision,
                            settings: s.project.as_ref().unwrap().project.clone(),
                            inventory: s.evidence.latest.as_ref().unwrap().run.inventory.clone(),
                            aliases: vec![],
                            notes: vec![],
                            visited: 1,
                        });
                        s.focus.preview = true;
                    }
                    "advanced-comparison" => s.analysis_advanced = true,
                    "loading-comparison" => {
                        s.focus.busy = true;
                        s.focus.view_loading = true;
                        s.analysis_advanced = true;
                    }
                    "history" => s.focus.screen = Screen::History,
                    "settings" => s.focus.screen = Screen::Settings,
                    "projects" => s.focus.screen = Screen::Projects,
                    "setup" => s.focus.screen = Screen::Setup,
                    "advanced-setup" => {
                        s.focus.screen = Screen::Setup;
                        s.focus.advanced = true;
                    }
                    _ => (),
                }
                cx.notify();
            });
            // Large test viewport exposes the whole mounted form. Screen-size
            // readability is a separate native obligation, not this invariant.
            cx.simulate_resize(size(px(1120.), px(8000.)));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    let candidates: Vec<_> = s
                        .focus
                        .controls()
                        .into_iter()
                        .filter(|control| s.control_enabled(*control, cx))
                        .collect();
                    let handles: Vec<_> = candidates
                        .iter()
                        .map(|control| s.handle(*control, cx))
                        .collect();
                    let mut mounted = Vec::new();
                    window.blur();
                    for _ in 0..256 {
                        window.focus_next();
                        let Some(handle) = window.focused(cx) else {
                            break;
                        };
                        if mounted.contains(&handle) {
                            break;
                        }
                        mounted.push(handle);
                    }
                    assert!(
                        !mounted.is_empty() && mounted.len() < 256,
                        "{case}: Tab map did not settle"
                    );
                    for (control, handle) in candidates.iter().zip(&handles) {
                        assert!(
                            mounted.contains(handle),
                            "{case}: unmounted enabled candidate {control:?}"
                        );
                    }
                    for handle in &mounted {
                        assert!(
                            handles.contains(handle),
                            "{case}: mounted Tab stop absent from candidates: {handle:?}"
                        );
                    }
                    assert_eq!(
                        handles.len(),
                        mounted.len(),
                        "{case}: duplicate or missing focus handles"
                    );
                    for control in [
                        Control::Guided(31),
                        Control::Guided(35),
                        Control::Guided(36),
                        Control::Phase6(13),
                    ] {
                        if s.focus.screen == Screen::Results || s.focus.screen == Screen::History {
                            assert!(
                                !s.control_enabled(control, cx),
                                "{case}: wrong-screen action {control:?}"
                            );
                        }
                    }
                    if s.focus.screen != Screen::Settings {
                        let requests = s.next_request;
                        s.activate_control(Control::Phase6(13), window, cx);
                        assert_eq!(s.next_request, requests);
                    }
                })
            });
        }
        owned.join_after_ui();
    }
    #[gpui::test]
    fn ui_sweep_dirty_masks_block_all_analysis_scheduling_and_stale_replies(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        let before = store.project_evidence(project).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Results;
            s.queue(Work::Load(project), cx);
            for _ in 0..8 {
                if s.pending.is_some() {
                    finish_request(s, cx);
                }
            }
            s.analysis_advanced = true;
            s.mask_rectangles
                .push(regressee_app::analysis_engine::Rectangle {
                    x: 1,
                    y: 1,
                    width: 4,
                    height: 4,
                });
            s.mask_dirty = true;
        });
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                let rectangles = s.mask_rectangles.clone();
                let requests = s.next_request;
                let displayed = s.evidence.comparison.as_ref().unwrap().revision.id;
                s.analysis_fields[0].update(cx, |input, cx| input.set_value("0.5", cx));
                s.poll_analysis(cx);
                assert_eq!(s.analysis_policy.pixel_tolerance_delta_e00, 0.5);
                for control in [Control::Phase5(27), Control::Phase5(28)] {
                    assert!(s.control_enabled(control, cx));
                    s.activate_control(control, window, cx);
                    assert!(s.analysis_due.is_none() && s.analysis_pending.is_none());
                    assert!(
                        s.error
                            .as_ref()
                            .unwrap()
                            .contains("unsaved rectangles are kept")
                    );
                    assert_eq!(s.mask_rectangles, rectangles);
                }
                s.analysis_due = Some(std::time::Instant::now());
                s.poll_analysis(cx);
                assert!(s.analysis_due.is_none() && s.analysis_pending.is_none());
                assert_eq!(
                    s.next_request, requests,
                    "dirty inputs/toggles/due work must submit zero Jobs requests"
                );
                let cancel = Arc::new(std::sync::atomic::AtomicBool::new(false));
                let id = requests + 100;
                s.analysis_pending = Some((id, cancel.clone()));
                s.poll_analysis(cx);
                assert!(cancel.load(std::sync::atomic::Ordering::Acquire));
                let mut stale = s.evidence.comparison.clone().unwrap();
                stale.revision.id = regressee_core::domain::AnalysisId::new();
                s.apply_update(
                    Update {
                        id,
                        reply: Reply::Analysis(Box::new(stale)),
                    },
                    cx,
                );
                assert!(s.analysis_pending.is_none() && s.pending.is_none());
                assert_eq!(
                    s.evidence.comparison.as_ref().unwrap().revision.id,
                    displayed
                );
                assert_eq!(s.mask_rectangles, rectangles);
                assert!(s.mask_dirty);
                assert_eq!(s.next_request, requests);
                // Saving/clearing the dirty condition restores normal scheduling.
                s.mask_dirty = false;
                assert!(s.schedule_analysis(cx));
                assert!(s.analysis_due.is_some());
                s.analysis_due = None;
            })
        });
        owned.join_after_ui();
        let store = regressee_storage::Store::open(paths).unwrap();
        let after = store.project_evidence(project).unwrap();
        assert_eq!(before.analyses, after.analyses);
        assert_eq!(before.baseline.unwrap().id, after.baseline.unwrap().id);
    }
    #[test]
    fn phase7_tiny_mask_outline_never_expands_capture_geometry() {
        for zoom in [0.125, 0.5, 1., 2., 4.] {
            for (w, h) in [(1, 1), (1, 20), (3, 2), (40, 40)] {
                let (w, h) = (w as f32 * zoom, h as f32 * zoom);
                let border = mask_outline_width(w, h);
                assert!(border * 2. <= w && border * 2. <= h && border <= 1.);
            }
        }
    }
    #[gpui::test]
    fn phase7_diagnostic_opt_in_cancel_stale_consent_and_actual_export(cx: &mut TestAppContext) {
        fn settled(s: &mut Shell, cx: &mut Context<Shell>) {
            for _ in 0..16 {
                if s.pending.is_none() {
                    return;
                }
                finish_request(s, cx);
            }
            panic!("diagnostic jobs did not settle");
        }
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (store, _, _, _, _) = approval_fixture::fixture(dir.path());
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        let destination = dir.path().join("diagnostic.json");
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                settled(s, cx);
                s.focus.screen = Screen::Settings;
                s.activate_control(Control::Phase7(0), window, cx);
                settled(s, cx);
                let report = s.diagnostic_preview.clone().unwrap();
                assert!(report.counts.captured > 0);
                assert!(!destination.exists());
                s.activate_control(Control::Phase7(2), window, cx);
                assert!(s.diagnostic_preview.is_none());
                assert!(s.pending.is_none());
                s.selected_diagnostic_path(destination.clone(), report, cx);
                assert!(s.error.as_ref().unwrap().contains("preview changed"));
                assert!(!destination.exists());
                s.activate_control(Control::Phase7(0), window, cx);
                settled(s, cx);
                let report = s.diagnostic_preview.clone().unwrap();
                // Simulated native save-path completion uses the exact real Jobs exporter.
                s.selected_diagnostic_path(destination.clone(), report.clone(), cx);
                settled(s, cx);
                assert_eq!(
                    serde_json::from_slice::<regressee_app::diagnostics::Report>(
                        &std::fs::read(&destination).unwrap()
                    )
                    .unwrap(),
                    report
                );
                assert!(s.diagnostic_preview.is_none());
                assert!(s.notice.as_ref().unwrap().contains("Nothing was sent"));
            })
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn phase6_simulated_native_picker_confirmation_and_actual_jobs_dataflow(
        cx: &mut TestAppContext,
    ) {
        fn settled(s: &mut Shell, cx: &mut Context<Shell>) {
            for _ in 0..16 {
                if s.pending.is_none() {
                    return;
                }
                finish_request(s, cx);
            }
            panic!("Phase6 jobs did not settle");
        }
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            settled(s, cx);
            s.focus.screen = Screen::History;
            s.queue(Work::Load(project), cx);
            settled(s, cx);
            assert!(s.history.is_some());
            assert!(s.project_storage.as_ref().unwrap().bytes > 0);
        });
        let zip = dir.path().join("native-dialog-completion.zip");
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Phase6(2), w, cx);
                settled(s, cx);
                let preview = s.export_preview.clone().unwrap();
                assert!(preview.privacy.contains("DO NOT redact"));
                assert!(!zip.exists());
                // Simulate only the platform dialog's returned path; the actual service writes the ZIP.
                s.selected_export_path(zip.clone(), preview, cx);
                settled(s, cx);
                assert!(zip.exists());
                assert!(s.export_preview.is_none());
                s.focus.screen = Screen::Projects;
                s.selected_import_path(zip.clone(), cx);
                settled(s, cx);
                assert!(s.import_preview.is_some());
                let store = regressee_storage::Store::open(paths.clone()).unwrap();
                assert_eq!(store.summary().unwrap().projects, 1);
                drop(store);
                s.activate_control(Control::Phase6(7), w, cx);
                assert!(s.import_preview.is_none());
                assert!(s.pending.is_none());
                s.selected_import_path(zip.clone(), cx);
                settled(s, cx);
                s.activate_control(Control::Phase6(1), w, cx);
                settled(s, cx);
                assert_eq!(
                    s.focus.screen,
                    Screen::Projects,
                    "Import must not navigate the shell or start browser work"
                );
                assert!(
                    s.notice
                        .as_ref()
                        .unwrap()
                        .contains("not an independently verified")
                );
                assert_eq!(s.projects.items.len(), 2);
                let imported = s.project.as_ref().unwrap().project.id;
                assert_ne!(imported, project);
                s.focus.screen = Screen::History;
                s.refresh_visible(cx);
                settled(s, cx);
                assert!(!s.history.as_ref().unwrap().imported_runs.is_empty());
                s.activate_control(Control::Phase6(4), w, cx);
                settled(s, cx);
                assert!(s.deletion_preview.is_none());
                assert!(s.error.as_ref().unwrap().contains("Active baseline"));
                s.error = None;
                s.focus.screen = Screen::Settings;
                s.activate_control(Control::Phase6(13), w, cx);
                settled(s, cx);
                assert!(s.repair_preview.is_some());
                s.activate_control(Control::Phase6(14), w, cx);
                settled(s, cx);
                assert!(s.repair_preview.is_none());
            })
        });
        owned.join_after_ui();
        let store = regressee_storage::Store::open(paths).unwrap();
        assert_eq!(store.summary().unwrap().projects, 2);
        store.project_evidence(project).unwrap();
    }
    #[gpui::test]
    fn phase5_approval_actions_refuse_old_preview_after_analysis_switch(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (store, project, run, valid, masked) = approval_fixture::fixture(dir.path());
        let count = store.baseline_history(project).unwrap().len();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::History;
            s.queue(Work::Load(project), cx);
            finish_request(s, cx);
            // Isolate Results from unrelated auth/status refreshes; all review
            // replies still come from the actual blocking Jobs/Store owner.
            finish_request(s, cx);
            finish_request(s, cx); // Phase 6's authoritative history/storage page.
            s.queue(Work::EnvironmentStatus(project), cx);
            finish_request(s, cx);
            s.focus.screen = Screen::Results;
            s.evidence.analyses = vec![valid, masked];
            s.approval_selection.insert(s.selected_key.clone().unwrap());
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Phase5(24), w, cx);
                finish_request(s, cx);
                assert_eq!(s.approval.as_ref().unwrap().analysis, valid);
                s.activate_control(Control::Phase5(26), w, cx);
                assert!(s.approval.is_none()); // Invalid at dispatch, before async reply.
                finish_request(s, cx);
                finish_request(s, cx); // authoritative viewer reload
                assert_eq!(s.evidence.comparison.as_ref().unwrap().revision.id, masked);
                s.activate_control(Control::Phase5(25), w, cx);
                assert!(s.pending.is_none());
            })
        });
        let store = regressee_storage::Store::open(paths.clone()).unwrap();
        assert_eq!(store.baseline_history(project).unwrap().len(), count);
        drop(store);
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Phase5(26), w, cx);
                finish_request(s, cx);
                finish_request(s, cx);
                s.activate_control(Control::Phase5(24), w, cx);
                finish_request(s, cx);
                let preview = s.approval.clone().unwrap();
                // Selection edits invalidate an already returned and a delayed preview.
                s.activate_control(Control::Phase5(23), w, cx);
                s.activate_control(Control::Phase5(25), w, cx);
                assert!(s.pending.is_none());
                s.activate_control(Control::Phase5(23), w, cx);
                s.activate_control(Control::Phase5(24), w, cx);
                s.schedule_analysis(cx);
                finish_request(s, cx);
                assert!(s.approval.is_none());
                s.analysis_due = None;
                // Exercise the real async reanalysis slot and completion, not a
                // fabricated busy flag. The latest saved mask is all-masked.
                s.activate_control(Control::Phase5(4), w, cx);
                s.analysis_due = Some(std::time::Instant::now());
                s.poll_analysis(cx);
                assert!(s.analysis_pending.is_some());
                s.approval = Some(preview.clone()); // queued old confirmation
                s.activate_control(Control::Phase5(25), w, cx);
                assert!(s.pending.is_none());
                let update = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
                assert!(matches!(update.reply, Reply::Analysis(_)));
                s.apply_update(update, cx);
                finish_request(s, cx);
                assert_eq!(s.evidence.comparison.as_ref().unwrap().revision.id, masked);
                s.approval = Some(preview.clone());
                s.activate_control(Control::Phase5(25), w, cx);
                assert!(s.pending.is_none());
                s.evidence.analyses = vec![masked, valid];
                s.activate_control(Control::Phase5(26), w, cx);
                finish_request(s, cx);
                finish_request(s, cx);
                // A changed project context cannot confirm a saved preview even
                // when an old activation event reaches the action handler.
                s.approval = Some(preview);
                s.project.as_mut().unwrap().revision += 1;
                s.activate_control(Control::Phase5(25), w, cx);
                assert!(s.pending.is_none());
                s.project.as_mut().unwrap().revision -= 1;
                s.invalidate_approval();
                s.activate_control(Control::Phase5(24), w, cx);
                finish_request(s, cx);
                s.activate_control(Control::Phase5(25), w, cx);
                s.focus.screen = Screen::History;
                finish_request(s, cx);
                finish_request(s, cx);
            })
        });
        owned.join_after_ui();
        let store = regressee_storage::Store::open(paths).unwrap();
        assert_eq!(store.baseline_history(project).unwrap().len(), count + 1);
        let active = store
            .load_project(project)
            .unwrap()
            .project
            .active_baseline
            .unwrap();
        assert_eq!(store.load_baseline(active).unwrap().references[0].run, run);
    }
    #[gpui::test]
    fn phase5_saved_pixels_seams_actual_layout_pan_and_mask_actions(cx: &mut TestAppContext) {
        use regressee_app::analysis_engine::Rectangle;
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let source = dir.path().join("coordinate-coded.png");
        let (width, height) = (96u32, 1137u32); // short last tile: 113 rows
        let pixels: Vec<u8> = (0..height)
            .flat_map(|y| (0..width).flat_map(move |x| [x as u8, y as u8, (y >> 8) as u8, 255]))
            .collect();
        let mut encoder = png::Encoder::new(std::fs::File::create(&source).unwrap(), width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .unwrap()
            .write_image_data(&pixels)
            .unwrap();
        let original = std::fs::read(&source).unwrap();
        let mut store = regressee_storage::Store::open(paths.clone()).unwrap();
        let image = regressee_app::images::ingest(
            &mut store,
            &source,
            regressee_core::domain::ArtifactKind::Screenshot,
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
        let tiles: Vec<_> = image
            .tiles
            .iter()
            .map(|id| regressee_app::images::viewer_tile(&store, id).unwrap())
            .collect();
        assert_eq!(tiles[2].size(0).height.0, 113);
        // Independently verify the actual decoded upload buffers, not frame metadata.
        for (i, tile) in tiles.iter().enumerate() {
            let bytes = tile.as_bytes(0).unwrap();
            for (j, pixel) in bytes.as_chunks::<4>().0.iter().enumerate() {
                let x = j % width as usize;
                let y = i * 512 + j / width as usize;
                assert_eq!(*pixel, [(y >> 8) as u8, y as u8, x as u8, 255]); // GPUI BGRA
            }
        }
        let (jobs, events) = Jobs::start_without_runtime(paths);
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Results;
            s.focus.viewer = true;
        });
        fn draw(shell: &Entity<Shell>, cx: &mut gpui::VisualTestContext) {
            cx.draw(
                gpui::point(px(0.), px(0.)),
                size(
                    gpui::AvailableSpace::Definite(px(800.)),
                    gpui::AvailableSpace::Definite(px(320.)),
                ),
                |_, cx| {
                    shell.update(cx, |s, cx| {
                        s.viewer_panes(s.viewer.as_ref().unwrap(), cx)
                            .into_element()
                    })
                },
            );
        }
        let mut cases = 0;
        for zoom in [0.5, 1., 2., 4.] {
            for retained in [0, 1] {
                for seam in [512u32, 1024] {
                    for pan in [seam - 1, seam, seam + 1] {
                        for direction in [-1., 1.] {
                            shell.update(cx, |s, _| {
                                s.viewer = Some(regressee_app::jobs::ViewerFrame {
                                    tile: retained,
                                    panes: vec![tiles[retained..].to_vec(); 3],
                                    width,
                                    height,
                                    request: None,
                                    analysis: None,
                                });
                                s.zoom = zoom;
                                s.pan_x = 7.;
                                s.pan_y = (pan as f32 + direction * 20.) as u32;
                                s.mask_draw = false;
                                s.mask_rectangles.clear();
                                s.viewer_mode = 0;
                            });
                            draw(&shell, cx);
                            let bounds = cx.debug_bounds("phase5-before").unwrap();
                            let start = bounds.origin + gpui::point(px(61.), px(141.));
                            cx.simulate_mouse_down(
                                start,
                                MouseButton::Left,
                                gpui::Modifiers::none(),
                            );
                            draw(&shell, cx);
                            // Real pointer action crosses forwards/backwards while the old
                            // bounded frame is retained (no release/refetch yet).
                            let end =
                                start + gpui::point(px(-3. * zoom), px(direction * 20. * zoom));
                            cx.simulate_mouse_move(end, MouseButton::Left, gpui::Modifiers::none());
                            draw(&shell, cx);
                            shell.update(cx, |s, _| {
                                assert_eq!(s.pan_y, pan);
                                assert_eq!(s.pan_x, 10.);
                                s.pan_drag = None;
                                s.mask_draw = true;
                            });
                            for mode in [0, 1, 2, 3] {
                                shell.update(cx, |s, _| s.viewer_mode = mode);
                                draw(&shell, cx);
                                let entries: &[(&str, &str)] = match mode {
                                    0 => &[
                                        ("phase5-before", "phase5-tile-0-0-0"),
                                        ("phase5-after", "phase5-tile-1-1-0"),
                                    ],
                                    1 => &[
                                        ("phase5-difference", "phase5-tile-2-1-0"),
                                        ("phase5-difference", "phase5-tile-2-2-0"),
                                    ],
                                    2 => &[
                                        ("phase5-before", "phase5-tile-0-0-0"),
                                        ("phase5-before", "phase5-tile-0-1-0"),
                                    ],
                                    _ => &[("phase5-difference", "phase5-tile-2-2-0")],
                                };
                                for &(pane, tile_selector) in entries {
                                    let viewport = cx.debug_bounds(pane).unwrap();
                                    let tile = cx.debug_bounds(tile_selector).unwrap();
                                    assert_eq!(
                                        f32::from(tile.origin.y - viewport.origin.y),
                                        1. + (retained as f32 * 512. - pan as f32) * zoom
                                    );
                                    assert_eq!(
                                        f32::from(tile.origin.x - viewport.origin.x),
                                        1. - 10. * zoom
                                    );
                                    assert_eq!(f32::from(tile.size.width), width as f32 * zoom);
                                    assert_eq!(f32::from(tile.size.height), 512. * zoom);
                                    // Resolve an actual saved-buffer pixel through measured layout.
                                    let source_y = pan + 8;
                                    let local_tile = source_y as usize / 512 - retained;
                                    let row = source_y as usize % 512;
                                    let pixel = &tiles[retained + local_tile].as_bytes(0).unwrap()
                                        [(row * width as usize + 21) * 4..][..4];
                                    assert_eq!(
                                        pixel,
                                        [(source_y >> 8) as u8, source_y as u8, 21, 255]
                                    );
                                    assert_eq!(
                                        f32::from(tile.origin.y)
                                            + (source_y as f32 - retained as f32 * 512.) * zoom,
                                        f32::from(viewport.origin.y) + 1. + 8. * zoom
                                    );
                                }
                            }
                            shell.update(cx, |s, _| s.viewer_mode = 0);
                            for pane in ["phase5-before", "phase5-after"] {
                                draw(&shell, cx);
                                let viewport = cx.debug_bounds(pane).unwrap();
                                let a = viewport.origin
                                    + gpui::point(px(1. + 11. * zoom), px(1. + 8. * zoom));
                                let b = a + gpui::point(px(4. * zoom), px(4. * zoom));
                                cx.simulate_mouse_down(
                                    a,
                                    MouseButton::Left,
                                    gpui::Modifiers::none(),
                                );
                                draw(&shell, cx);
                                cx.simulate_mouse_move(
                                    b,
                                    MouseButton::Left,
                                    gpui::Modifiers::none(),
                                );
                                draw(&shell, cx);
                                cx.simulate_mouse_up(b, MouseButton::Left, gpui::Modifiers::none());
                                shell.update(cx, |s, _| {
                                    assert_eq!(
                                        s.mask_rectangles.last(),
                                        Some(&Rectangle {
                                            x: 21,
                                            y: pan + 8,
                                            width: 4,
                                            height: 4,
                                        })
                                    )
                                });
                            }
                            draw(&shell, cx);
                            for (pane, selector) in [
                                ("phase5-before", "phase5-mask-0-0"),
                                ("phase5-after", "phase5-mask-1-0"),
                            ] {
                                let viewport = cx.debug_bounds(pane).unwrap();
                                let mask = cx.debug_bounds(selector).unwrap();
                                assert_eq!(
                                    mask.origin - viewport.origin,
                                    gpui::point(px(1. + 11. * zoom), px(1. + 8. * zoom))
                                );
                            }
                            cases += 1;
                        }
                    }
                }
            }
        }
        // Settled frame at the short last tile must not stretch it to 512 rows.
        shell.update(cx, |s, _| {
            s.viewer.as_mut().unwrap().tile = 2;
            s.viewer.as_mut().unwrap().panes = vec![vec![tiles[2].clone()]; 3];
            s.pan_y = 1136;
        });
        draw(&shell, cx);
        assert_eq!(
            f32::from(cx.debug_bounds("phase5-tile-0-0-0").unwrap().size.height),
            113. * 4.
        );
        assert_eq!(
            std::fs::read(store.artifact_path(&image.original).unwrap()).unwrap(),
            original
        );
        println!(
            "{cases} saved-pixel/simulated-GPUI pan cases; all modes and both pointer panes; CAS unchanged"
        );
        owned.join_after_ui();
    }
    #[gpui::test]
    fn phase5_real_tile_pointer_mask_coordinates_and_synchronized_actions(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        // Actual saved immutable corpus image/derivative pixels, decoded outside
        // the UI handlers. The GPUI platform here is simulated, not native approval.
        let (mut store, project, _, _, _) = approval_fixture::fixture(dir.path());
        let source = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/analysis-proposal/images/tiny-icon-long-page-before.png");
        let image = regressee_app::images::ingest(
            &mut store,
            &source,
            regressee_core::domain::ArtifactKind::Screenshot,
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
        let tiles = image
            .tiles
            .iter()
            .skip(1)
            .take(3)
            .map(|id| regressee_app::images::viewer_tile(&store, id).unwrap())
            .collect::<Vec<_>>();
        let (jobs, events) = Jobs::start_without_runtime(paths);
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Results;
            // Action dispatch now requires the same real project/comparison
            // sections as rendering, not a viewer flag on an empty shell.
            s.queue(Work::Load(project), cx);
            for _ in 0..8 {
                if s.pending.is_some() {
                    finish_request(s, cx);
                }
            }
            s.mask_rectangles.clear();
            s.mask_dirty = false;
            s.focus.viewer = true;
            s.focus.busy = false;
            s.viewer = Some(regressee_app::jobs::ViewerFrame {
                tile: 1,
                panes: vec![tiles.clone(), tiles.clone(), tiles],
                width: image.width,
                height: image.height,
                request: None,
                analysis: None,
            });
            s.zoom = 2.;
            s.pan_x = 10.;
            s.pan_y = 1000;
            s.mask_draw = true;
        });
        fn draw(shell: &Entity<Shell>, cx: &mut gpui::VisualTestContext) {
            cx.draw(
                gpui::point(px(0.), px(0.)),
                size(
                    gpui::AvailableSpace::Definite(px(600.)),
                    gpui::AvailableSpace::Definite(px(320.)),
                ),
                |_, cx| {
                    shell.update(cx, |s, cx| {
                        s.viewer_panes(s.viewer.as_ref().unwrap(), cx)
                            .into_element()
                    })
                },
            );
        }
        draw(&shell, cx);
        let bounds = cx.debug_bounds("phase5-before").unwrap();
        cx.simulate_mouse_down(
            bounds.origin + gpui::point(px(41.), px(31.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        draw(&shell, cx);
        let bounds = cx.debug_bounds("phase5-before").unwrap();
        cx.simulate_mouse_move(
            bounds.origin + gpui::point(px(57.), px(47.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        draw(&shell, cx);
        let bounds = cx.debug_bounds("phase5-before").unwrap();
        cx.simulate_mouse_up(
            bounds.origin + gpui::point(px(57.), px(47.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        shell.update(cx, |s, _| {
            assert_eq!(
                s.mask_rectangles,
                [regressee_app::analysis_engine::Rectangle {
                    x: 30,
                    y: 1015,
                    width: 8,
                    height: 8
                }]
            );
            assert!(s.mask_dirty);
            assert_eq!((s.zoom, s.pan_x, s.pan_y), (2., 10., 1000));
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Phase5(15), w, cx);
                s.activate_control(Control::Phase5(17), w, cx);
            })
        });
        draw(&shell, cx);
        let bounds = cx.debug_bounds("phase5-after").unwrap();
        cx.simulate_mouse_down(
            bounds.origin + gpui::point(px(61.), px(61.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        draw(&shell, cx);
        let bounds = cx.debug_bounds("phase5-after").unwrap();
        cx.simulate_mouse_move(
            bounds.origin + gpui::point(px(81.), px(81.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        draw(&shell, cx);
        let bounds = cx.debug_bounds("phase5-after").unwrap();
        cx.simulate_mouse_up(
            bounds.origin + gpui::point(px(81.), px(81.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        shell.update(cx, |s, _| {
            assert_eq!(
                s.mask_rectangles,
                [regressee_app::analysis_engine::Rectangle {
                    x: 40,
                    y: 1030,
                    width: 10,
                    height: 10
                }]
            )
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Phase5(15), w, cx);
                assert_eq!(s.mask_selected, Some(0));
                s.activate_control(Control::Phase5(16), w, cx);
                assert!(s.mask_rectangles.is_empty());
                s.activate_control(Control::Phase5(8), w, cx);
                assert_eq!(s.viewer_mode, 2);
                s.activate_control(Control::Phase5(11), w, cx);
                assert_eq!(s.overlay_fraction, 0.6);
                s.activate_control(Control::ZoomIn, w, cx);
                assert_eq!(s.zoom, 4.);
                s.activate_control(Control::PanDown, w, cx);
                assert_eq!(s.pan_y, 1128);
                assert!(s.focus.view_loading);
                finish_request(s, cx);
                assert!(s.mask_dirty);
                s.activate_control(Control::Phase5(0), w, cx);
                assert_eq!(s.analysis_policy.pixel_tolerance_delta_e00, 2.);
                assert!(s.analysis_due.is_none());
                assert!(s.mask_dirty);
                assert!(
                    s.error
                        .as_ref()
                        .unwrap()
                        .contains("unsaved rectangles are kept")
                );
                assert!(!s.focus.busy);
            })
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn filtered_preview_actions_use_visible_ids_and_freeze_exact_selection(
        cx: &mut TestAppContext,
    ) {
        use regressee_core::domain::*;
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, finish_request);
        let settings =
            Project::draft("Preview regression".into(), "https://example.test/").unwrap();
        let pages: Vec<_> = ["/", "/match/one", "/hidden", "/match/two"]
            .into_iter()
            .map(|path| InventoryPage {
                id: PageId::new(),
                canonical: format!("https://example.test{path}").parse().unwrap(),
                included: true,
                reason: None,
            })
            .collect();
        let ids: Vec<_> = pages.iter().map(|p| p.id).collect();
        shell.update(cx, |s, cx| {
            s.focus.screen = Screen::Results;
            s.focus.preview = true;
            s.focus.project_saved = true;
            s.project = Some(regressee_storage::ProjectRecord {
                project: settings.clone(),
                revision: 1,
            });
            s.preview = Some(DiscoveryPreview {
                project_revision: 1,
                inventory: InventoryRevision {
                    id: InventoryId::new(),
                    project: settings.id,
                    identity_version: 2,
                    pages,
                    coverage: DiscoveryCoverage::Complete,
                },
                settings: settings.clone(),
                aliases: vec![],
                notes: vec![],
                visited: 0,
            });
            s.selected_page = Some(ids[0]);
            s.crawl_fields[9].update(cx, |i, cx| i.set_value("/match", cx));
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Guided(17), w, cx);
                assert_eq!(s.selected_page, Some(ids[1]));
                s.activate_control(Control::Guided(22), w, cx);
                assert!(!s.preview.as_ref().unwrap().inventory.pages[1].included);
                assert!(s.preview.as_ref().unwrap().inventory.pages[0].included);
                s.activate_control(Control::Guided(24), w, cx);
                assert_eq!(s.selected_page, Some(ids[3]));
                s.activate_control(Control::Guided(23), w, cx);
                assert_eq!(s.selected_page, Some(ids[1]));
                // A stale click for a row hidden by a new filter cannot steal selection.
                s.select_preview_page(ids[2]);
                assert_eq!(s.selected_page, Some(ids[1]));
                s.select_preview_page(ids[3]);
                s.activate_control(Control::Guided(22), w, cx);
                s.crawl_fields[9].update(cx, |i, cx| i.set_value("no-matches", cx));
                s.activate_control(Control::Guided(17), w, cx);
                assert_eq!(s.selected_page, None);
                let before: Vec<_> = s
                    .preview
                    .as_ref()
                    .unwrap()
                    .inventory
                    .pages
                    .iter()
                    .map(|p| p.included)
                    .collect();
                for action in [22, 23, 24] {
                    s.activate_control(Control::Guided(action), w, cx);
                }
                assert_eq!(
                    before,
                    s.preview
                        .as_ref()
                        .unwrap()
                        .inventory
                        .pages
                        .iter()
                        .map(|p| p.included)
                        .collect::<Vec<_>>()
                );
                s.crawl_fields[9].update(cx, |i, cx| i.set_value("/match/one", cx));
                s.activate_control(Control::Guided(17), w, cx);
                s.activate_control(Control::Guided(22), w, cx); // restore one, keep two excluded
                s.crawl_fields[9].update(cx, |i, cx| i.set_value("", cx));
                s.activate_control(Control::Guided(17), w, cx);
                assert_eq!(s.selected_page, Some(ids[1]));
                s.crawl_fields[9].update(cx, |i, cx| i.set_value("/match", cx));
                s.activate_control(Control::Guided(17), w, cx);
                s.activate_control(Control::Guided(20), w, cx);
                assert_eq!(
                    s.preview
                        .as_ref()
                        .unwrap()
                        .inventory
                        .pages
                        .iter()
                        .map(|p| p.included)
                        .collect::<Vec<_>>(),
                    vec![true, false, true, false]
                );
                s.activate_control(Control::Guided(21), w, cx);
                assert!(
                    s.preview
                        .as_ref()
                        .unwrap()
                        .inventory
                        .pages
                        .iter()
                        .all(|p| p.included)
                );
                s.select_preview_page(ids[3]);
                s.activate_control(Control::Guided(22), w, cx);
                s.crawl_fields[9].update(cx, |i, cx| i.set_value("", cx));
                s.activate_control(Control::Guided(17), w, cx);
                assert_eq!(s.selected_page, Some(ids[3]));
            })
        });
        let inventory =
            cx.update(|_, cx| shell.read(cx).preview.as_ref().unwrap().inventory.clone());
        owned.join_after_ui();
        let mut store = regressee_storage::Store::open(paths).unwrap();
        store.save_project(&settings, None).unwrap();
        store.freeze_inventory(&inventory).unwrap();
        let frozen = store.load_inventory(inventory.id).unwrap();
        let mut changed = inventory.clone();
        changed.pages[0].included = false;
        assert!(store.freeze_inventory(&changed).is_err());
        assert!(store.load_inventory(inventory.id).unwrap().pages[0].included);
        assert_eq!(
            frozen
                .pages
                .iter()
                .filter(|p| p.included)
                .map(|p| p.id)
                .collect::<Vec<_>>(),
            vec![ids[0], ids[1], ids[2]]
        );
    }
    #[gpui::test]
    fn guided_controls_persist_lists_profiles_scope_and_speed(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |shell, cx| {
            finish_request(shell, cx);
            shell.focus.screen = Screen::Setup;
            shell
                .name
                .update(cx, |input, cx| input.set_value("Guided website", cx));
            shell.address.update(cx, |input, cx| {
                input.set_value("https://example.test/docs https://other.test/docs", cx)
            });
            shell.crawl_fields[0].update(cx, |input, cx| input.set_value("100", cx));
            shell.crawl_fields[1].update(cx, |input, cx| input.set_value("/docs", cx));
            shell.crawl_fields[2].update(cx, |input, cx| input.set_value("logout, private", cx));
        });
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                for index in [11, 2, 3] {
                    shell.activate_control(Control::Guided(index), window, cx);
                }
                shell.activate_control(Control::Choice(Choice::Fast), window, cx);
                shell.activate_control(Control::Choice(Choice::AddressesOnly), window, cx);
                shell.save(&SaveProject, window, cx);
            })
        });
        shell.update(cx, |shell, cx| {
            finish_request(shell, cx);
            let p = &shell.project.as_ref().unwrap().project;
            assert_eq!(p.sources.len(), 2);
            assert_eq!(p.profiles.len(), 3);
            assert_eq!(p.discovery.page_limit, 100);
            assert_eq!(p.discovery.mode, DiscoveryMode::OnlyAddresses);
            assert_eq!(p.discovery.speed, Speed::Fast);
            assert_eq!(p.discovery.section.as_deref(), Some("/docs"));
            assert_eq!(p.discovery.skipped_text, ["logout", "private"]);
            assert!(!shell.unsaved(cx));
        });
        owned.join_after_ui();
        let saved = regressee_storage::Store::open(paths)
            .unwrap()
            .list_projects(0)
            .unwrap();
        assert_eq!(saved.items.len(), 1);
    }
    #[gpui::test]
    fn environment_login_mapping_and_network_actions_use_authoritative_actor_state(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Setup;
            s.name
                .update(cx, |i, cx| i.set_value("Environment fixture", cx));
            s.address.update(cx, |i, cx| {
                i.set_value("https://production.test/app/home?b=2&a=1&a=0#/orders", cx)
            });
        });
        cx.update(|w, cx| shell.update(cx, |s, cx| s.save(&SaveProject, w, cx)));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            if s.pending.is_some() {
                finish_request(s, cx);
            }
            assert!(s.environment.is_some());
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Guided(27), w, cx);
                s.activate_control(Control::Choice(Choice::LoginRequired), w, cx);
                s.activate_control(Control::Choice(Choice::EphemeralLogin), w, cx);
                s.crawl_fields[13]
                    .update(cx, |i, cx| i.set_value("https://production.test/app", cx));
                s.crawl_fields[14]
                    .update(cx, |i, cx| i.set_value("https://staging.test/preview", cx));
                s.crawl_fields[15].update(cx, |i, cx| i.set_value("nb-NO", cx));
                s.timezone
                    .update(cx, |select, cx| select.set_value("Europe/Oslo", cx));
                s.activate_control(Control::Guided(37), w, cx);
                assert!(s.environment_unsaved(cx));
                s.activate_control(Control::Guided(31), w, cx);
                assert!(s.pending.is_none());
                assert!(s.error.as_ref().unwrap().contains("Save settings"));
                s.error = None;
                s.activate_control(Control::Save, w, cx);
            })
        });
        let id = shell.update(cx, |s, cx| {
            finish_request(s, cx);
            if s.pending.is_some() {
                finish_request(s, cx);
            }
            let e = s.environment.as_ref().unwrap();
            assert!(e.settings.ephemeral);
            assert!(e.name.starts_with("Staging"));
            assert_eq!(
                e.mapped[0].1,
                "https://staging.test/preview/home?b=2&a=1&a=0#/orders"
            );
            assert!(!s.environment_unsaved(cx));
            s.project.as_ref().unwrap().project.id
        });
        cx.update(|w, cx| shell.update(cx, |s, cx| s.activate_control(Control::Guided(32), w, cx)));
        shell.update(cx, |s, cx| {
            assert!(s.pending.is_none());
            assert!(!s.control_enabled(Control::Guided(32), cx));
            assert!(s.evidence.latest.is_none());
            s.error = None;
            s.focus.screen = Screen::Settings;
            s.crawl_fields[17].update(cx, |i, cx| i.set_value("http://127.0.0.1:3128/", cx));
        });
        cx.update(|w, cx| shell.update(cx, |s, cx| s.activate_control(Control::Guided(41), w, cx)));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            assert!(
                s.network_status
                    .as_ref()
                    .unwrap()
                    .contains("TLS verification stays enabled")
            );
        });
        owned.join_after_ui();
        let store = regressee_storage::Store::open(paths.clone()).unwrap();
        let p = store.load_project(id).unwrap().project;
        assert_eq!(p.profiles[0].locale, "nb-NO");
        assert_eq!(p.profiles[0].timezone, "Europe/Oslo");
        assert_eq!(
            store.selected_environment(&p).unwrap().access,
            regressee_core::domain::AccessPolicy::InteractiveLogin
        );
        assert!(
            store
                .session_reference(id, store.selected_environment(&p).unwrap().id)
                .unwrap()
                .is_none()
        );
        assert!(!serde_json::to_string(&p).unwrap().contains("session"));
        assert_eq!(
            regressee_runtime::network::NetworkSettings::load(
                &paths.cache.join("network-settings.json")
            )
            .unwrap()
            .proxy
            .as_deref(),
            Some("http://127.0.0.1:3128/")
        );
    }
    // Test-only wait for the real actor, then invoke the exact completion path
    // used by poll_storage. No fabricated successful persistence replies.
    fn finish_request(shell: &mut Shell, cx: &mut Context<Shell>) {
        let update = shell.events.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(shell.pending, Some(update.id));
        assert!(
            !matches!(update.reply, Reply::Failure(_)),
            "{:?}",
            update.reply
        );
        shell.apply_update(update, cx);
    }
    #[gpui::test]
    fn save_navigation_completion_refreshes_real_list_without_losing_draft(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned_jobs = jobs.clone();
        let (shell, cx) = cx.add_window_view(|window, cx| {
            let shell = Shell::new(cx, jobs, events);
            window.focus(&shell.handle(Control::Navigate(Screen::Projects), cx));
            shell
        });
        shell.update(cx, |shell, cx| {
            finish_request(shell, cx);
            assert!(shell.projects.items.is_empty());
            assert!(!shell.projects_dirty);
            assert!(shell.pending.is_none());
        });
        cx.dispatch_action(CreateProject);
        shell.update(cx, |shell, cx| {
            shell
                .name
                .update(cx, |input, cx| input.set_value("Saved project", cx));
            shell.address.update(cx, |input, cx| {
                input.set_value("https://example.test/docs", cx)
            });
        });
        cx.dispatch_action(SaveProject);
        let save_id = cx.update(|_, cx| shell.read(cx).pending.expect("save not queued"));
        // Use the same enabled navigation-button action as a click. Simulated
        // keyboard dispatch cannot route through the now-disabled input focus.
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Projects), window, cx)
            })
        });
        shell.update(cx, |shell, cx| {
            assert_eq!(shell.focus.screen, Screen::Projects);
            assert_eq!(shell.pending, Some(save_id));
            assert!(shell.focus.busy);
            assert!(shell.projects.items.is_empty());
            assert_eq!(shell.name.read(cx).content, "Saved project");
            finish_request(shell, cx);
            assert_eq!(shell.focus.screen, Screen::Projects);
            assert!(shell.projects_dirty);
            assert!(
                shell.pending.is_some_and(|id| id > save_id),
                "completion did not schedule refresh"
            );
            assert!(shell.focus.busy);
            let saved = shell.project.as_ref().unwrap();
            assert_eq!(saved.revision, 1);
            assert_eq!(saved.project.name, "Saved project");
            finish_request(shell, cx);
            assert!(shell.pending.is_none());
            assert!(!shell.focus.busy);
            assert!(!shell.projects_dirty);
            assert_eq!(shell.focus.projects, 1);
            assert_eq!(
                shell.projects.items[0].id,
                shell.project.as_ref().unwrap().project.id
            );
            assert_eq!(shell.projects.items[0].name, "Saved project");
        });
        // Edit/save again, visit Projects while pending, then leave it before
        // completion. No forced navigation or unnecessary hidden-list request.
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Setup), window, cx)
            })
        });
        shell.update(cx, |shell, cx| {
            if shell.pending.is_some() {
                finish_request(shell, cx);
            }
            shell
                .name
                .update(cx, |input, cx| input.set_value("Renamed project", cx));
        });
        cx.dispatch_action(SaveProject);
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Projects), window, cx)
            })
        });
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Results), window, cx)
            })
        });
        shell.update(cx, |shell, cx| {
            finish_request(shell, cx);
            assert_eq!(shell.focus.screen, Screen::Results);
            // Phase 2 now refreshes actual evidence on the visible Results
            // surface. It must still not schedule a hidden Projects list or
            // replace the selected screen/settings snapshot.
            assert!(shell.pending.is_some());
            finish_request(shell, cx);
            if shell.pending.is_some() {
                finish_request(shell, cx);
            }
            assert_eq!(shell.focus.screen, Screen::Results);
            assert!(shell.evidence.latest.is_none());
            assert!(shell.pending.is_none());
            assert!(shell.projects_dirty);
            assert_eq!(shell.project.as_ref().unwrap().revision, 2);
        });
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Setup), window, cx)
            })
        });
        shell.update(cx, |shell, cx| {
            shell
                .name
                .update(cx, |input, cx| input.set_value("Unsaved next edit", cx));
            assert!(shell.unsaved(cx));
        });
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Projects), window, cx)
            })
        });
        shell.update(cx, |shell, cx| {
            finish_request(shell, cx);
            assert_eq!(shell.focus.screen, Screen::Projects);
            assert_eq!(shell.projects.items[0].name, "Renamed project");
            assert_eq!(shell.projects.items[0].revision, 2);
            assert_eq!(shell.name.read(cx).content, "Unsaved next edit");
            assert!(shell.unsaved(cx));
            assert!(shell.pending.is_none());
        });
        // A real competing writer causes a revision conflict. Failure retains
        // the pending edit and navigation, and must not trigger a retry loop.
        let mut competing = regressee_storage::Store::open(paths.clone()).unwrap();
        let mut current = cx.update(|_, cx| shell.read(cx).project.clone().unwrap());
        current.project.name = "Other window".into();
        competing.save_project(&current.project, Some(2)).unwrap();
        drop(competing);
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Setup), window, cx)
            })
        });
        cx.dispatch_action(SaveProject);
        let conflict_id =
            cx.update(|_, cx| shell.read(cx).pending.expect("conflicting save not queued"));
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Projects), window, cx)
            })
        });
        shell.update(cx, |shell, cx| {
            // An unrelated completion cannot steal pending work or clear busy.
            shell.apply_update(
                Update {
                    id: conflict_id + 100,
                    reply: Reply::Failure(regressee_app::jobs::StorageFailure::Unavailable),
                },
                cx,
            );
            assert_eq!(shell.pending, Some(conflict_id));
            assert!(shell.focus.busy);
            assert!(shell.error.is_none());
            let update = shell.events.recv_timeout(Duration::from_secs(5)).unwrap();
            assert_eq!(update.id, conflict_id);
            assert!(matches!(
                update.reply,
                Reply::Failure(regressee_app::jobs::StorageFailure::ConflictOrWrite)
            ));
            shell.apply_update(update, cx);
            assert_eq!(shell.focus.screen, Screen::Projects);
            assert_eq!(shell.name.read(cx).content, "Unsaved next edit");
            assert_eq!(shell.project.as_ref().unwrap().revision, 2);
            assert!(shell.unsaved(cx));
            assert!(shell.projects_dirty);
            assert!(shell.error.is_some());
            assert!(shell.pending.is_none());
            assert!(!shell.focus.busy);
            shell.poll_storage(cx);
            assert!(shell.pending.is_none());
        });
        owned_jobs.join_after_ui();
        let store = regressee_storage::Store::open(paths).unwrap();
        assert_eq!(
            store.list_projects(0).unwrap().items[0].name,
            "Other window"
        );
    }
    #[gpui::test]
    fn shell_focus_actions_busy_guard_and_dialog_restore(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned_jobs = jobs.clone();
        let (shell, cx) = cx.add_window_view(|window, cx| {
            let mut shell = Shell::new(cx, jobs, events);
            // Simulated UI state; storage actor behavior is separately integrated.
            shell.focus.busy = false;
            window.focus(&shell.handle(Control::Navigate(Screen::Projects), cx));
            shell
        });
        cx.dispatch_action(CreateProject);
        cx.update(|window, cx| {
            assert!(shell.read(cx).handle(Control::Name, cx).is_focused(window))
        });
        cx.dispatch_action(Tab);
        cx.update(|window, cx| {
            assert!(
                shell
                    .read(cx)
                    .handle(Control::Address, cx)
                    .is_focused(window)
            )
        });
        cx.dispatch_action(Tab);
        cx.update(|window, cx| {
            assert!(
                shell
                    .read(cx)
                    .handle(Control::Guided(11), cx)
                    .is_focused(window)
            )
        });
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.focus.busy = true;
                let before = shell.next_request;
                shell.save(&SaveProject, window, cx);
                assert_eq!(shell.next_request, before);
                shell.focus.busy = false;
                shell.focus.screen = Screen::Settings;
                shell.activate_control(Control::About, window, cx);
                assert!(shell.handle(Control::CloseDialog, cx).is_focused(window));
                shell.move_focus(false, window, cx);
                assert!(shell.handle(Control::CloseDialog, cx).is_focused(window));
                shell.navigate(Screen::Results, window, cx);
                assert_eq!(shell.focus.screen, Screen::Settings);
                shell.dismiss(&Dismiss, window, cx);
                assert!(shell.handle(Control::About, cx).is_focused(window));
            })
        });
        owned_jobs.join_after_ui();
    }
    #[gpui::test]
    fn product_actions_use_real_actor_and_unconfigured_runtime_is_not_unchanged(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|window, cx| {
            let shell = Shell::new(cx, jobs, events);
            window.focus(&shell.handle(Control::Navigate(Screen::Projects), cx));
            shell
        });
        shell.update(cx, finish_request);
        cx.dispatch_action(CreateProject);
        shell.update(cx, |s, cx| {
            s.name
                .update(cx, |i, cx| i.set_value("Product action test", cx));
            s.address
                .update(cx, |i, cx| i.set_value("https://example.test/", cx));
        });
        cx.dispatch_action(SaveProject);
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            assert!(s.focus.project_saved);
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Navigate(Screen::Results), w, cx)
            });
        });
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            if s.pending.is_some() {
                finish_request(s, cx);
            }
            assert!(s.evidence.latest.is_none());
            assert!(!s.focus.candidate);
            assert!(!s.focus.baseline);
            assert!(!s.focus.viewer);
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::CaptureBaseline, w, cx);
                assert!(s.focus.busy && s.focus.capturing);
                assert!(s.focus.can_activate(Control::CancelCapture));
                assert!(!s.focus.can_activate(Control::SaveBaseline));
            });
        });
        shell.update(cx, |s, cx| {
            let update = s.events.recv_timeout(Duration::from_secs(10)).unwrap();
            assert!(matches!(update.reply, Reply::ProductFailure(_)));
            s.apply_update(update, cx);
            assert!(s.error.as_ref().unwrap().contains("trusted distribution"));
            assert!(s.evidence.latest.is_none());
            assert!(!s.focus.candidate && !s.focus.baseline);
            assert!(
                s.focus.busy,
                "failed operations must refresh authoritative coverage"
            );
            finish_request(s, cx);
            assert!(!s.focus.busy);
            assert!(s.error.as_ref().unwrap().contains("trusted distribution"));
        });
        owned.join_after_ui();
        let store = regressee_storage::Store::open(paths).unwrap();
        assert_eq!(store.list_projects(0).unwrap().items.len(), 1);
    }
}
