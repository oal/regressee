// SPDX-License-Identifier: MPL-2.0
use gpui::{
    AnyElement, App, Application, Bounds, Context, Div, Entity, FocusHandle, FontWeight,
    KeyBinding, Menu, MenuItem, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent,
    PathPromptOptions, Pixels, PromptLevel, SharedString, TitlebarOptions, Window, WindowBounds,
    WindowOptions, actions, canvas, div, img, prelude::*, px, relative, rgb, rgba, size,
    uniform_list,
};
use regressee_app::{
    formatting::human_bytes,
    jobs::{Jobs, Reply, Request, Update, Work},
    ui::{
        controls::{self, ActionStyle, Tone},
        text_input::{self, TextInput},
        theme::*,
        timezone_select::{self, TimezoneSelect},
        zoom_select::{self, ZoomChoice, ZoomSelect},
    },
};
use regressee_core::{
    crawl::{self, DiscoveryPreview},
    domain::{
        CaptureExtent, CaptureProfile, Classification, DiscoveryMode, JobState, Project, RunIntent,
        Speed, TargetKey, TargetOutcome,
    },
    evidence::ProjectEvidence,
    shell::{Choice, Control, Detail, FocusState, ReviewSurface, Screen, SetupStage},
};
use regressee_storage::{AppPaths, ProjectPage, ProjectRecord, StorageSummary};
use std::{
    sync::{Arc, mpsc::Receiver},
    time::Duration,
};

mod exit_guard;
mod form_recovery;
mod loading_ux;
mod local_ux;
mod options_ux;
mod review_ux;
mod task_ux;
#[cfg(all(feature = "ui-inspection", target_os = "linux"))]
mod ui_inspection;
mod viewer_ux;
mod workflow;
mod workspace_history;

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
        SettingsScreen,
        FindLocal,
        KeyboardHelp
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
    task: Option<task_ux::Task>,
    last_action: Option<Control>,
    viewer_pending: Option<(u64, regressee_app::jobs::ViewerIdentity, usize)>,
    storage_pending: Option<(u64, regressee_core::domain::ProjectId)>,
    projects: ProjectPage,
    projects_dirty: bool,
    project: Option<ProjectRecord>,
    summary: Option<StorageSummary>,
    error: Option<String>,
    notice: Option<String>,
    restore_focus: Option<Control>,
    help_dialog: bool,
    native_prompt: bool,
    exit_guard: exit_guard::ExitGuard,
    exit_network_saved: Option<(String, String)>,
    evidence: ProjectEvidence,
    evidence_dirty: bool,
    evidence_read_failed: bool,
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
    zoom_select: Entity<ZoomSelect>,
    draft_settings: Project,
    preview: Option<DiscoveryPreview>,
    search: String,
    local_index: std::cell::RefCell<local_ux::QueueIndex>,
    history_search: Entity<TextInput>,
    history_query: String,
    history_kind: usize,
    history_section: usize,
    history_selection_epoch: u64,
    history_open_epoch: Option<u64>,
    history_inspection: Option<(
        regressee_core::domain::ProjectId,
        regressee_core::domain::RunId,
    )>,
    history_filter_cache: std::cell::RefCell<local_ux::HistoryFilterCache>,
    result_filter: usize,
    group_profiles: bool,
    selected_page: Option<regressee_core::domain::PageId>,
    selected_key: Option<TargetKey>,
    origins_confirmed: bool,
    approved_origins: std::collections::BTreeSet<String>,
    environment: Option<regressee_app::auth::EnvironmentStatus>,
    environment_read_signature: Option<(u64, options_ux::EnvironmentDraftSignature)>,
    environment_draft: Option<regressee_core::domain::EnvironmentId>,
    login_ephemeral: bool,
    show_mapping: bool,
    network_status: Option<String>,
    network_load_started: bool,
    network_read_signature: Option<(String, String)>,
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
    review_drafts: review_ux::ReviewDrafts,
    capture_details_open: bool,
    capture_diagnostics_open: bool,
    review_note_open: bool,
    mask_selected: Option<usize>,
    mask_draw: bool,
    mask_drag: Option<((u32, u32), (u32, u32))>,
    mask_dirty: bool,
    pan_drag: Option<((f32, f32), (f32, u32))>,
    viewer_drag_pane: Option<usize>,
    viewer_bounds: std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>,
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
    form_anchors: Vec<gpui::ScrollAnchor>,
    review_anchors: Vec<gpui::ScrollAnchor>,
    last_validation_field: Option<Control>,
    history_baseline_scroll: gpui::ScrollHandle,
    history_run_scroll: gpui::ScrollHandle,
    review_details_scroll: gpui::ScrollHandle,
    options_scroll: gpui::ScrollHandle,
    options_anchors: Vec<gpui::ScrollAnchor>,
    options_bounds: Vec<std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>>,
    mapping_preview_scroll: gpui::ScrollHandle,
    discovery_details_open: bool,
    review_focus_reveal: std::rc::Rc<std::cell::Cell<u8>>,
    approval_provenance_scroll: gpui::ScrollHandle,
    approval_reference_index: usize,
    viewer_fit: bool,
    narrow_workspace: bool,
    review_tools: bool,
    review_queue_collapsed: bool,
    discovery_preview_visible: bool,
    result_filters_open: bool,
    working_width: f32,
    presentation_focus: Option<Control>,
}
impl Shell {
    fn new(cx: &mut Context<Self>, jobs: Arc<Jobs>, events: Receiver<Update>) -> Self {
        let mut shell = Self {
            root_focus: cx.focus_handle(),
            handles: (0..(621 + regressee_app::task::MAX_TASK_LINKS))
                .map(|i| cx.focus_handle().tab_index(i as isize + 1).tab_stop(true))
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
            task: None,
            last_action: None,
            viewer_pending: None,
            storage_pending: None,
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
            help_dialog: false,
            native_prompt: false,
            exit_guard: exit_guard::ExitGuard::default(),
            exit_network_saved: None,
            evidence: ProjectEvidence::default(),
            evidence_dirty: true,
            evidence_read_failed: false,
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
            zoom_select: cx.new(ZoomSelect::new),
            draft_settings: Project::draft("New website".into(), "https://example.test").unwrap(),
            preview: None,
            search: String::new(),
            local_index: Default::default(),
            history_search: cx.new(|cx| TextInput::with_value("", 317, cx)),
            history_query: String::new(),
            history_kind: 0,
            history_section: 0,
            history_selection_epoch: 0,
            history_open_epoch: None,
            history_inspection: None,
            history_filter_cache: Default::default(),
            result_filter: 0,
            group_profiles: false,
            selected_page: None,
            selected_key: None,
            origins_confirmed: false,
            approved_origins: std::collections::BTreeSet::new(),
            environment: None,
            environment_read_signature: None,
            environment_draft: None,
            login_ephemeral: false,
            show_mapping: false,
            network_status: None,
            network_load_started: false,
            network_read_signature: None,
            analysis_policy: regressee_app::review::preset(
                regressee_app::analysis_engine::Preset::Balanced,
            ),
            analysis_fields: [
                "2", "0.2", "5", "8", "64", "32", "16", "16", "50", "", "0", "0", "1", "1",
            ]
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
            review_drafts: Default::default(),
            capture_details_open: false,
            capture_diagnostics_open: false,
            review_note_open: false,
            mask_selected: None,
            mask_draw: false,
            mask_drag: None,
            mask_dirty: false,
            pan_drag: None,
            viewer_drag_pane: None,
            viewer_bounds: Default::default(),
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
            form_anchors: Vec::new(),
            review_anchors: Vec::new(),
            last_validation_field: None,
            history_baseline_scroll: gpui::ScrollHandle::new(),
            history_run_scroll: gpui::ScrollHandle::new(),
            review_details_scroll: gpui::ScrollHandle::new(),
            options_scroll: gpui::ScrollHandle::new(),
            options_anchors: Vec::new(),
            options_bounds: (0..12).map(|_| Default::default()).collect(),
            mapping_preview_scroll: gpui::ScrollHandle::new(),
            discovery_details_open: false,
            review_focus_reveal: Default::default(),
            approval_provenance_scroll: gpui::ScrollHandle::new(),
            approval_reference_index: 0,
            viewer_fit: true,
            narrow_workspace: false,
            review_tools: false,
            review_queue_collapsed: false,
            discovery_preview_visible: false,
            result_filters_open: false,
            working_width: 1040.,
            presentation_focus: None,
        };
        cx.observe(&shell.timezone, |_, _, cx| cx.notify()).detach();
        cx.subscribe(&shell.zoom_select, |this, _, choice: &ZoomChoice, cx| {
            if this.control_enabled(Control::Guided(46), cx) {
                this.choose_viewer_zoom(*choice, cx);
            }
        })
        .detach();
        shell.form_anchors = (0..21)
            .map(|_| gpui::ScrollAnchor::for_handle(shell.screen_scroll.clone()))
            .collect();
        shell.review_anchors = (0..64)
            .map(|_| gpui::ScrollAnchor::for_handle(shell.review_details_scroll.clone()))
            .collect();
        shell.options_anchors = (0..9)
            .map(|_| gpui::ScrollAnchor::for_handle(shell.options_scroll.clone()))
            .collect();
        shell
            .name
            .update(cx, |input, _| input.validator = Some(local_ux::name));
        shell
            .address
            .update(cx, |input, _| input.validator = Some(local_ux::addresses));
        for (i, input) in shell.crawl_fields.iter().enumerate() {
            input.update(cx, |input, _| {
                input.validator = local_ux::crawl_validator(i)
            });
        }
        for (i, input) in shell.analysis_fields.iter().enumerate() {
            input.update(cx, |input, _| {
                input.validator = local_ux::analysis_validator(i)
            });
        }
        for input in shell.analysis_fields.iter().skip(9) {
            cx.observe(input, |_, _, cx| cx.notify()).detach();
        }
        cx.observe(&shell.crawl_fields[9], |this, input, cx| {
            let query = input.read(cx).content.trim().to_lowercase();
            if this.search != query {
                this.search = query;
                cx.notify();
            }
        })
        .detach();
        cx.observe(&shell.history_search, |this, input, cx| {
            let query = input.read(cx).content.trim().to_lowercase();
            if this.history_query != query {
                this.history_query = query;
                this.clear_hidden_history_previews();
                cx.notify();
            }
        })
        .detach();
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
        self.sync_review_note(cx);
        // Bounded batch: at most the update channel capacity per tick.
        let mut updates = Vec::new();
        for _ in 0..regressee_app::jobs::UPDATE_CAPACITY {
            let Ok(update) = self.events.try_recv() else {
                break;
            };
            updates.push(update);
        }
        updates.sort_by_key(|update| {
            !self
                .viewer_pending
                .as_ref()
                .is_some_and(|(id, _, _)| *id == update.id)
                && !self.storage_pending.is_some_and(|(id, _)| id == update.id)
        });
        for update in updates {
            self.apply_update(update, cx);
        }
        self.poll_analysis(cx);
        if let Some(task) = &mut self.task
            && task.state != regressee_app::task::State::Finished
        {
            let elapsed = task.started.elapsed().as_secs();
            if elapsed != task.elapsed {
                task.elapsed = elapsed;
                cx.notify();
            }
        }
    }
    fn apply_update(&mut self, update: Update, cx: &mut Context<Self>) {
        self.observe_exit_reply(&update);
        let Some(update) = self.continue_exit_save(update, cx) else {
            return;
        };
        // Independent reads never lock mutation controls, and stale read replies
        // must be rejected before retiring or replacing any displayed pixels.
        if self
            .viewer_pending
            .as_ref()
            .is_some_and(|(id, _, _)| *id == update.id)
        {
            let (_, identity, tile) = self.viewer_pending.take().unwrap();
            self.focus.view_loading = false;
            match update.reply {
                Reply::Viewer(frame)
                    if frame.identity.as_ref() == Some(&identity)
                        && frame.analysis == identity.analysis
                        && frame.tile == tile
                        && self.viewer_identity().as_ref() == Some(&identity)
                        && tile == self.pan_y as usize / 512 =>
                {
                    if !self.mask_dirty {
                        self.mask_rectangles = frame
                            .request
                            .as_ref()
                            .and_then(|r| r.mask.as_ref())
                            .map(|m| m.rectangles.clone())
                            .unwrap_or_default();
                    }
                    self.clear_viewer();
                    self.retired_images.retain(|image| {
                        !frame
                            .panes
                            .iter()
                            .flatten()
                            .any(|current| Arc::ptr_eq(image, current))
                    });
                    self.viewer = Some(*frame);
                    self.evidence.before =
                        self.viewer.as_ref().and_then(|v| v.before_capture.clone());
                    self.prefetch_next_viewer();
                }
                Reply::ProductFailure(error) => self.error = Some(error),
                Reply::Failure(error) => self.error = Some(error.message().into()),
                _ => (),
            }
            cx.notify();
            return;
        }
        if self.storage_pending.is_some_and(|(id, _)| id == update.id) {
            let (_, project) = self.storage_pending.take().unwrap();
            if self
                .project
                .as_ref()
                .is_some_and(|p| p.project.id == project)
            {
                match update.reply {
                    Reply::ProjectStorage(storage) => self.project_storage = Some(storage),
                    Reply::ProductFailure(error) => self.error = Some(error),
                    Reply::Failure(error) => self.error = Some(error.message().into()),
                    _ => (),
                }
            }
            cx.notify();
            return;
        }
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
        if let Reply::CaptureStatus(status) = update.reply {
            if let Some(task) = &mut self.task
                && task.request == update.id
                && task.state != regressee_app::task::State::Finished
                && matches!(
                    task.spec.kind,
                    regressee_app::task::Kind::Capture
                        | regressee_app::task::Kind::Check
                        | regressee_app::task::Kind::Representative
                )
                && task.project_id == Some(status.project)
                && task
                    .capture
                    .as_ref()
                    .is_none_or(|previous| previous.run == status.run)
            {
                task.environment = status.environment.clone();
                task.capture = Some(status);
            }
            cx.notify();
            return;
        }
        if let Reply::Progress {
            message,
            completed,
            total,
        } = update.reply
        {
            if let Some(task) = &mut self.task
                && task.request == update.id
            {
                task.progress = Some((message.clone(), completed, total));
            }
            self.progress = Some((message, completed, total));
            cx.notify();
            return;
        }
        // Resolve the task from the authoritative terminal reply, not from a
        // stop request or a guessed elapsed-time threshold.
        let completing_task = self
            .task
            .as_ref()
            .is_some_and(|task| task.request == update.id);
        self.finish_task(update.id, &update.reply);
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
                self.history_inspection = None;
                let selecting_project = self.focus.choosing_project;
                let first_save = self.project.is_none() && !selecting_project && !imported;
                let saved_baseline = self.focus.candidate
                    && record.project.active_baseline
                        != self
                            .project
                            .as_ref()
                            .and_then(|p| p.project.active_baseline);
                self.focus.choosing_project = false;
                if self
                    .project
                    .as_ref()
                    .is_none_or(|p| p.project.id != record.project.id)
                {
                    self.search.clear();
                    self.result_filter = 0;
                    self.selected_page = None;
                    self.selected_key = None;
                    self.crawl_fields[9].update(cx, |input, cx| input.set_value("", cx));
                    self.mask_rectangles.clear();
                    self.mask_selected = None;
                    self.mask_draw = false;
                    self.mask_drag = None;
                    self.approval_selection.clear();
                    self.focus.review_surface = ReviewSurface::Changes;
                    self.review_tools = false;
                    self.review_queue_collapsed = false;
                    self.discovery_preview_visible = false;
                    self.result_filters_open = false;
                }
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
                self.notice = (!selecting_project).then(|| {
                    if saved_baseline { "Baseline saved explicitly. Future checks revisit this fixed inventory; earlier versions remain in History.".into() } else { "Settings saved locally. Review and History use the selected project.".into() }
                });
                if saved_baseline {
                    self.focus.review_surface = ReviewSurface::Changes;
                }
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
                self.focus.preview = false;
                self.focus.resumable = false;
                self.evidence_dirty = true;
                self.evidence = ProjectEvidence::default();
                self.evidence_read_failed = false;
                self.clear_viewer();
                if !selecting_project || imported {
                    self.summary = None;
                    self.projects_dirty = true;
                }
                if first_save && self.focus.setup_stage == SetupStage::Website {
                    self.focus.setup_stage = SetupStage::Access;
                    self.presentation_focus = Some(Control::SetupStage(SetupStage::Access));
                    self.notice = Some("Website saved locally. Choose whether login is needed; no website was visited.".into());
                }
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
            Reply::History { history } => {
                let baseline = self
                    .evidence
                    .baselines
                    .get(self.history_index)
                    .map(|b| b.id);
                let run = self
                    .history
                    .as_ref()
                    .and_then(|h| h.runs.get(self.run_history_index))
                    .map(|r| r.run.id);
                self.evidence.baselines = history.baselines.clone();
                self.history_index = baseline
                    .and_then(|id| history.baselines.iter().position(|b| b.id == id))
                    .unwrap_or(0);
                self.run_history_index = run
                    .and_then(|id| history.runs.iter().position(|r| r.run.id == id))
                    .unwrap_or(0);
                self.focus.history_baselines = history.baselines.len();
                self.focus.history_runs = history.runs.len();
                self.history = Some(*history);
                *self.history_filter_cache.borrow_mut() = Default::default();
                let next_baseline = self
                    .evidence
                    .baselines
                    .get(self.history_index)
                    .map(|b| b.id);
                let next_run = self
                    .history
                    .as_ref()
                    .and_then(|h| h.runs.get(self.run_history_index))
                    .map(|r| r.run.id);
                if baseline != next_baseline || run != next_run {
                    self.clear_portability_previews();
                }
                if self.project_storage.is_none() {
                    self.queue_project_storage(cx);
                }
            }
            Reply::ProjectStorage(_) => (), // handled by the independent read generation above
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
            Reply::Failure(error) => {
                self.evidence_read_failed =
                    self.project.is_some() && self.evidence_dirty && self.evidence.latest.is_none();
                self.error = Some(error.message().into());
            }
            Reply::ProductFailure(error) => {
                self.evidence_read_failed =
                    self.project.is_some() && self.evidence_dirty && self.evidence.latest.is_none();
                self.evidence_dirty = true;
                if failed_operation
                    && self.focus.screen == Screen::Results
                    && let Some(project) = &self.project
                {
                    self.queue(Work::Evidence(project.project.id), cx);
                }
                self.error = Some(error);
            }
            Reply::Cancelled => {
                self.notice = Some("The task stopped. Already saved work is kept.".into());
                if failed_operation && let Some(project) = &self.project {
                    self.evidence_dirty = true;
                    self.queue(Work::Evidence(project.project.id), cx);
                }
            }
            Reply::Runtime(status) => self.runtime_status = Some(status),
            Reply::RuntimeReadiness(status) => {
                self.runtime_status = Some(status.message.into());
                self.runtime_readiness = Some(status);
            }
            Reply::Network { settings, message } => {
                let signature = self.network_read_signature.take();
                let unchanged = signature.as_ref().is_none_or(|(proxy, ca)| {
                    self.crawl_fields[17].read(cx).content.as_ref() == proxy.as_str()
                        && self.crawl_fields[18].read(cx).content.as_ref() == ca.as_str()
                });
                if unchanged {
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
                }
                self.network_status = Some(message);
                if !unchanged {
                    self.notice = Some("Saved network settings were read. Your newer draft was kept; Save explicitly to apply it.".into());
                }
            }
            Reply::Environment(status) => {
                // A status/URL preview refresh observes saved settings; it never
                // rehydrates an already loaded draft. Initial hydration is bound
                // to the exact request and rejected if edits arrived meanwhile.
                let hydrate = self.environment.is_none()
                    && self
                        .environment_read_signature
                        .take()
                        .is_some_and(|(id, draft)| {
                            id == update.id && draft == self.environment_draft_signature(cx)
                        });
                if hydrate {
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
                }
                self.environment = Some(*status);
            }
            Reply::Evidence(e) => {
                self.evidence_read_failed = false;
                self.history_inspection = None;
                let history_baseline = self
                    .evidence
                    .baselines
                    .get(self.history_index)
                    .map(|b| b.id);
                let compatible =
                    self.evidence
                        .latest
                        .as_ref()
                        .zip(e.latest.as_ref())
                        .is_some_and(|(old, new)| {
                            old.run.id == new.run.id
                                && self.selected_key.as_ref().is_some_and(|key| {
                                    new.run.targets.iter().any(|t| &t.key == key)
                                })
                        });
                if self.mask_dirty {
                    self.invalidate_approval();
                    self.evidence_dirty = false;
                    self.notice = Some("New evidence is available. Save ignored areas before refreshing; your inspected evidence is kept.".into());
                    self.refresh_visible(cx);
                    cx.notify();
                    return;
                }
                if !e
                    .latest
                    .as_ref()
                    .is_some_and(|r| r.run.intent == RunIntent::RepresentativePreview)
                {
                    self.preview = None;
                    self.focus.preview = false;
                }
                if !compatible {
                    self.mask_dirty = false;
                    self.mask_drag = None;
                    self.mask_selected = None;
                }
                self.evidence = *e;
                self.history_index = history_baseline
                    .and_then(|id| self.evidence.baselines.iter().position(|b| b.id == id))
                    .unwrap_or(0);
                if history_baseline
                    != self
                        .evidence
                        .baselines
                        .get(self.history_index)
                        .map(|b| b.id)
                {
                    self.clear_portability_previews();
                }
                *self.local_index.borrow_mut() = Default::default();
                *self.history_filter_cache.borrow_mut() = Default::default();
                self.invalidate_approval();
                if !compatible {
                    self.approval_selection.clear();
                }
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
                if !compatible {
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
                }
                self.focus.candidate = self.evidence.latest.as_ref().is_some_and(|r| {
                    r.run.state == JobState::Completed
                        && r.run.intent == RunIntent::NewBaseline
                        && !self.capture_is_saved_baseline()
                });
                self.focus.baseline = self.evidence.baseline.is_some();
                if self.focus.candidate && !compatible {
                    self.focus.review_surface = ReviewSurface::Inspect;
                    self.presentation_focus = Some(Control::SaveBaseline);
                } else if self.focus.review_surface == ReviewSurface::Capture
                    && self
                        .evidence
                        .latest
                        .as_ref()
                        .is_some_and(|r| matches!(r.run.intent, RunIntent::Check { .. }))
                {
                    self.focus.review_surface = ReviewSurface::Changes;
                }
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
                    let _ = p;
                    if !self.mask_dirty {
                        self.reload_viewer(cx);
                    }
                }
            }
            Reply::InspectedRun {
                project,
                run,
                evidence,
            } => {
                self.receive_history_inspection(project, run, *evidence, cx);
            }
            Reply::Viewer(_) => (), // only the independent, identity-bound read path may replace pixels
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
                    self.focus.approval_dialog = true;
                    self.approval_reference_index = 0;
                    self.presentation_focus = Some(Control::Phase5(25));
                }
            }
            Reply::SavedReview(history) => {
                self.evidence.reviews = history;
                self.review_save_completed(cx);
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
                self.discovery_preview_visible = false;
                self.review_queue_collapsed = false;
                self.result_filters_open = false;
                self.focus.preview = true;
                self.preview = Some(*p);
                self.search.clear();
                self.crawl_fields[9].update(cx, |input, cx| input.set_value("", cx));
                self.local_index.borrow_mut().preview = None;
                self.selected_page = self.preview_page_ids().first().copied();
                self.focus.screen = Screen::Results;
                self.focus.review_surface = ReviewSurface::Pages;
                self.presentation_focus = Some(Control::CaptureBaseline);
                self.notice=Some("Review included pages and coverage before capturing. Your active baseline is unchanged.".into());
            }
            Reply::Progress { .. } | Reply::CaptureStatus(_) => unreachable!(),
        }
        if self
            .task
            .as_ref()
            .is_some_and(|task| task.state == regressee_app::task::State::Finished)
        {
            self.presentation_focus = Some(Control::TaskContinue);
        } else if completing_task
            && self.task.is_none()
            && self
                .presentation_focus
                .is_none_or(|control| !self.control_enabled(control, cx))
        {
            // Successful actions can remove their own trigger (for example,
            // saving a candidate baseline). Never focus its detached handle.
            self.presentation_focus = Some(Control::Navigate(self.focus.screen));
        }
        self.refresh_visible(cx);
        cx.notify();
    }
    fn refresh_visible(&mut self, cx: &mut Context<Self>) {
        if self.pending.is_some() || self.error.is_some() || self.task.is_some() {
            return;
        }
        if self.focus.shows_project_picker() {
            if self.projects_dirty {
                self.queue(
                    Work::List {
                        page: self.projects.page,
                    },
                    cx,
                );
            }
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
        if self.focus.screen == Screen::Settings && !self.network_load_started {
            self.network_load_started = true;
            self.network_read_signature = Some((
                self.crawl_fields[17].read(cx).content.to_string(),
                self.crawl_fields[18].read(cx).content.to_string(),
            ));
            self.queue(Work::NetworkStatus, cx);
            return;
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
            l.map(|v| v.min_changed.to_string())
                .unwrap_or_else(|| self.analysis_fields[3].read(cx).content.to_string()),
            l.map(|v| v.min_eligible.to_string())
                .unwrap_or_else(|| self.analysis_fields[4].read(cx).content.to_string()),
            l.map(|v| v.size.to_string())
                .unwrap_or_else(|| self.analysis_fields[5].read(cx).content.to_string()),
            l.map(|v| v.stride.to_string())
                .unwrap_or_else(|| self.analysis_fields[6].read(cx).content.to_string()),
            p.region_min_pixels
                .map(|v| v.to_string())
                .unwrap_or_default(),
            (p.ignored_warning_fraction * 100.).to_string(),
        ];
        for (i, s) in values.iter().enumerate() {
            self.analysis_fields[i].update(cx, |f, cx| f.set_value(s, cx));
        }
        self.analysis_signature = values;
        self.configure_analysis_inputs(cx);
    }
    fn analysis_field_active(&self, index: usize, cx: &App) -> bool {
        !(3..=6).contains(&index) || !self.analysis_fields[2].read(cx).content.trim().is_empty()
    }
    /// Disabling local windows keeps numeric drafts but removes their validation
    /// and stale touched corrections until the criterion is enabled again.
    fn configure_analysis_inputs(&self, cx: &mut Context<Self>) {
        for (i, input) in self.analysis_fields.iter().enumerate().take(9) {
            let active = self.analysis_field_active(i, cx);
            input.update(cx, |input, cx| {
                let changed =
                    input.validator.is_some() != active || (!active && input.correction.is_some());
                input.validator = if active {
                    local_ux::analysis_validator(i)
                } else {
                    None
                };
                if !active {
                    input.correction = None;
                }
                if changed {
                    cx.notify();
                }
            });
        }
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
        if self.history_inspection.is_some() {
            return false;
        }
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
        if self.focus.approval_dialog {
            self.presentation_focus = Some(Control::Phase5(24));
        }
        self.focus.approval_dialog = false;
        self.approval = None;
        self.approval_epoch += 1;
    }
    fn approval_matches(&self, preview: &regressee_storage::ApprovalPreview) -> bool {
        let c = &preview.context;
        self.analysis_pending.is_none()
            && !self.mask_dirty
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
        if self.history_inspection.is_some() {
            return;
        }
        if self.analysis_advanced {
            self.configure_analysis_inputs(cx);
            if self
                .analysis_fields
                .iter()
                .take(9)
                .enumerate()
                .any(|(i, f)| self.analysis_field_active(i, cx) && f.read(cx).composing())
            {
                return;
            }
            let values: Vec<_> = self
                .analysis_fields
                .iter()
                .take(9)
                .map(|f| f.read(cx).content.trim().to_owned())
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
                        for input in self.analysis_fields.iter().take(9) {
                            input.update(cx, |input, _| input.correction = None);
                        }
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
                        // Incomplete typing is field-local; the previous analysis
                        // remains visible until a complete valid draft exists.
                        for input in self.analysis_fields.iter().take(9) {
                            input.update(cx, |input, _| input.correction = None);
                        }
                        if self.analysis_field_active(3, cx)
                            && self
                                .analysis_fields
                                .iter()
                                .take(9)
                                .all(|input| input.read(cx).validation_error().is_none())
                        {
                            for i in [3, 4, 5, 6] {
                                self.analysis_fields[i].update(cx, |input, _| input.correction = Some(format!("Check related thresholds: {error}. Pixel counts must fit the local window; stride must cover it in at most four steps.")));
                            }
                        }
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
        let Some(identity) = self.viewer_identity() else {
            return;
        };
        if self
            .viewer
            .as_ref()
            .and_then(|v| v.identity.as_ref())
            .is_some_and(|current| current != &identity)
        {
            self.clear_viewer();
        }
        let tile = self.pan_y as usize / 512;
        if self
            .viewer_pending
            .as_ref()
            .is_some_and(|(_, current, current_tile)| current == &identity && *current_tile == tile)
        {
            return;
        }
        self.next_request += 1;
        let id = self.next_request;
        let work = if let Some((project, run)) = self.history_inspection {
            Work::HistoricalViewer {
                project,
                run,
                tile,
                key: Some(identity.key.clone()),
            }
        } else {
            Work::Viewer {
                project: identity.project,
                key: Some(identity.key.clone()),
                analysis: identity.analysis,
                tile,
            }
        };
        match self.jobs.submit_viewer(Request { id, work }) {
            Ok(()) => {
                self.viewer_pending = Some((id, identity, tile));
                self.focus.view_loading = true;
            }
            Err(error) => self.error = Some(error.into()),
        }
        cx.notify();
    }
    fn viewer_identity(&self) -> Option<regressee_app::jobs::ViewerIdentity> {
        Some(regressee_app::jobs::ViewerIdentity {
            project: self.project.as_ref()?.project.id,
            run: self.evidence.latest.as_ref()?.run.id,
            key: self.selected_key.clone()?,
            analysis: self.evidence.comparison.as_ref().map(|a| a.revision.id),
        })
    }
    fn viewer_current(&self) -> bool {
        self.viewer.as_ref().is_some_and(|frame| {
            frame
                .identity
                .as_ref()
                .is_none_or(|identity| self.viewer_identity().as_ref() == Some(identity))
                && frame.tile == self.pan_y as usize / 512
        })
    }
    fn prefetch_next_viewer(&self) {
        if self.history_inspection.is_some() {
            return;
        }
        if self.focus.screen != Screen::Results
            || self.focus.shows_project_picker()
            || self.preview.is_some()
            || self.pending.is_some()
            || self.mask_dirty
            || self.viewer_pending.is_some()
        {
            return;
        }
        let Some(identity) = self.viewer_identity() else {
            return;
        };
        let Some(run) = &self.evidence.latest else {
            return;
        };
        let rows = self.result_indices();
        if let Some(next) = rows
            .iter()
            .position(|i| run.run.targets[*i].key == identity.key)
            .and_then(|i| rows.get(i + 1))
        {
            let _ = self.jobs.prefetch_viewer(Work::Viewer {
                project: identity.project,
                key: Some(run.run.targets[*next].key.clone()),
                analysis: identity.analysis,
                tile: 0,
            });
        }
    }
    fn queue_project_storage(&mut self, cx: &mut Context<Self>) {
        let Some(project) = self.project.as_ref().map(|p| p.project.id) else {
            return;
        };
        if self.storage_pending.is_some_and(|(_, id)| id == project) {
            return;
        }
        self.next_request += 1;
        let id = self.next_request;
        match self.jobs.submit(Request {
            id,
            work: Work::ProjectStorage(project),
        }) {
            Ok(()) => self.storage_pending = Some((id, project)),
            Err(error) => self.error = Some(error.into()),
        }
        cx.notify();
    }
    fn phase5_action(&mut self, index: usize, cx: &mut Context<Self>) {
        if self.history_inspection.is_some() && !matches!(index, 6..=13 | 36 | 38..=39) {
            return;
        }
        if (32..=39).contains(&index) {
            self.review_ux_action(index, cx);
            return;
        }
        use regressee_app::analysis_engine as e;
        if (14..=19).contains(&index) && !self.viewer_current() {
            return;
        }
        if (40..=42).contains(&index)
            && !self
                .history_baseline_indices()
                .contains(&self.history_index)
        {
            return;
        }
        if index == 42 && self.mask_dirty {
            return;
        }
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
                if self.analysis_advanced {
                    self.configure_analysis_inputs(cx);
                    if self
                        .analysis_fields
                        .iter()
                        .take(9)
                        .enumerate()
                        .any(|(i, input)| {
                            self.analysis_field_active(i, cx) && input.read(cx).composing()
                        })
                    {
                        return;
                    }
                    for (i, input) in self.analysis_fields.iter().enumerate().take(9) {
                        if !self.analysis_field_active(i, cx) {
                            continue;
                        }
                        input.update(cx, |input, cx| {
                            input.touched = true;
                            cx.notify();
                        });
                    }
                    if let Some(i) =
                        self.analysis_fields
                            .iter()
                            .enumerate()
                            .take(9)
                            .find_map(|(i, input)| {
                                (self.analysis_field_active(i, cx)
                                    && (input.read(cx).validation_error().is_some()
                                        || input.read(cx).correction.is_some()))
                                .then_some(i)
                            })
                    {
                        self.presentation_focus = Some(Control::AnalysisField(i));
                        return;
                    }
                }
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
                if self.mask_geometry_dirty(cx) {
                    return;
                }
                self.mask_draw = !self.mask_draw;
                self.mask_selected = None;
            }
            15 => {
                if self.mask_geometry_dirty(cx) {
                    return;
                }
                if !self.mask_rectangles.is_empty() {
                    self.mask_selected = Some(
                        (self.mask_selected.map(|i| i + 1).unwrap_or(0))
                            % self.mask_rectangles.len(),
                    );
                    self.load_mask_fields(cx);
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
                if index == 18
                    && self
                        .viewer
                        .as_ref()
                        .and_then(|v| v.request.as_ref())
                        .is_some_and(|r| r.validate().is_err())
                {
                    return;
                }
                if self.mask_geometry_dirty(cx) {
                    self.error = Some("Apply the area edit before saving ignored areas; your numeric draft is preserved.".into());
                    return;
                }
                if self
                    .viewer
                    .as_ref()
                    .and_then(|v| v.request.as_ref())
                    .is_some_and(|r| {
                        review_ux::all_masked(
                            &self.mask_rectangles,
                            r.before.dimensions,
                            r.after.dimensions,
                        )
                    })
                {
                    self.error = Some("Ignored areas cover all screenshot pixels. Reduce or delete an area before saving; this cannot be a clean result.".into());
                    return;
                }
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
                self.sync_review_note(cx);
                if !self.review_result_valid() {
                    return;
                }
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
                    if !self.approval_selection.remove(key) && self.approval_eligible(key) {
                        self.approval_selection.insert(key.clone());
                    }
                    self.invalidate_approval();
                }
            }
            24 => {
                if self
                    .approval
                    .as_ref()
                    .is_some_and(|p| self.approval_matches(p))
                {
                    self.focus.approval_dialog = true;
                    self.presentation_focus = Some(Control::Phase5(25));
                    cx.notify();
                    return;
                }
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
                if self.focus.approval_dialog
                    && let Some(preview) = &self.approval
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
                    self.discovery_preview_visible = true;
                    self.queue(
                        Work::PreviewImages {
                            project: p.project.id,
                            preview: Box::new(preview.clone()),
                        },
                        cx,
                    );
                    self.focus.capturing = self.pending.is_some();
                    if self.focus.capturing {
                        self.focus.review_surface = ReviewSurface::Capture;
                    }
                }
            }
            40 | 41 => {
                let rows = self.history_baseline_indices();
                if let Some(next) =
                    local_ux::adjacent_visible(&rows, Some(&self.history_index), index == 40)
                    && self.history_index != rows[next]
                {
                    self.clear_portability_previews();
                    self.history_index = rows[next];
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
        if matches!(index, 2 | 4)
            && !self
                .history_baseline_indices()
                .contains(&self.history_index)
        {
            return;
        }
        if matches!(index, 5 | 9 | 10 | 19)
            && !self.history_run_indices().contains(&self.run_history_index)
        {
            return;
        }
        if matches!(index, 4..=6) && self.mask_dirty {
            return;
        }
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
                self.queue_project_storage(cx);
            }
            9 | 10 => {
                self.history_selection_epoch = self.history_selection_epoch.wrapping_add(1);
                let rows = self.history_run_indices();
                if let Some(next) =
                    local_ux::adjacent_visible(&rows, Some(&self.run_history_index), index == 10)
                    && self.run_history_index != rows[next]
                {
                    self.clear_portability_previews();
                    self.run_history_index = rows[next];
                }
            }
            11 | 12 => {
                self.history_selection_epoch = self.history_selection_epoch.wrapping_add(1);
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
            16..=18 => {
                self.history_selection_epoch = self.history_selection_epoch.wrapping_add(1);
                self.history_section = index - 16;
                self.clear_portability_previews();
            }
            19 => self.open_history_run(cx),
            _ => (),
        }
        cx.notify();
    }
    fn portability_confirmation(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div().flex().flex_col().gap_3().max_w(px(680.));
        if let Some(p) = &self.export_preview {
            body=body.child(div().font_weight(FontWeight::SEMIBOLD).child("Export contents — review before sharing"))
                .child(format!("{} targets · {} original PNGs · {} · {} source capture dates · {} ignored-region configurations. Selected/inherited originals only; no unrelated history or database.",p.targets,p.originals,human_bytes(p.bytes),p.sources,p.masks))
                .child(div().text_color(rgb(WARNING)).child(p.privacy.clone()))
                .child(self.button(Control::Phase6(3),"I understand — export ZIP…",ActionStyle::Primary,cx));
        }
        if let Some(p) = &self.import_preview {
            body=body.child(div().font_weight(FontWeight::SEMIBOLD).child(format!("Import “{}” as a NEW local project",p.name)))
                .child(format!("Validated {} targets · {} original PNGs · {} · {} ignored-region configurations. Name collisions get a separate safe name. Masks require revalidation for your capture pair.",p.targets,p.originals,human_bytes(p.bytes),p.masks))
                .child(div().text_color(rgb(WARNING)).child(p.warning.clone()))
                .child(self.button(Control::Phase6(1),"Confirm import — no website visits",ActionStyle::Primary,cx));
        }
        if let Some(p) = &self.deletion_preview {
            body=body.child(div().font_weight(FontWeight::SEMIBOLD).child("Delete retained history?"))
                .child(format!("{:?}: {} analyses and {} unshared objects removed; {} deletable. {} of shared data remain. This cannot be undone; active and retained baseline references are protected.",p.item,p.analyses,p.objects,human_bytes(p.bytes),human_bytes(p.shared_bytes)))
                .child(self.button(Control::Phase6(6),"Confirm permanent deletion",ActionStyle::Danger,cx));
        }
        if let Some(p) = &self.repair_preview {
            body=body.child(div().font_weight(FontWeight::SEMIBOLD).child("Interrupted writes and integrity"))
                .child(format!("{} temporary files and {} dead-owner interrupted imports ({}) may be discarded. {} valid complete unreferenced objects are retained, NOT treated as failed captures. {} missing/corrupt objects require recovery; they cannot establish a valid active baseline.",p.temporaries.len(),p.interrupted_imports.len(),human_bytes(p.temporary_bytes),p.valid_unreferenced.len(),p.missing_or_corrupt.len()))
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
            Control::SetupStage(stage) => 312 + stage.index(),
            Control::PagesBaseline => 316,
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
            Control::SelectProject(index) => 248 + index,
            Control::ChooseProject => 240,
            Control::CancelProjectChoice => 241,
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
            Control::Viewer => 615,
            Control::Guided(46) => return self.zoom_select.read(cx).focus_handle(),
            Control::PauseCapture => 88,
            Control::Guided(index @ 50..=52) => 612 + index - 50,
            Control::Guided(index) => 90 + index,
            Control::CrawlField(16) => return self.timezone.read(cx).focus_handle(cx),
            Control::CrawlField(index) => return self.crawl_fields[index].read(cx).focus.clone(),
            Control::Phase5(index @ 32..=37) => 606 + index - 32,
            Control::Phase5(index) => 140 + index,
            Control::AnalysisField(index) => {
                return self.analysis_fields[index].read(cx).focus.clone();
            }
            Control::Phase6(index @ 16..=19) => 600 + index - 16,
            Control::Phase6(index) => 184 + index,
            Control::Phase7(index) => 200 + index,
            Control::Choice(choice) => 204 + choice.index(),
            Control::Environment(index) => 224 + index,
            Control::Details(detail) => 244 + detail as usize,
            Control::DismissFeedback => 243,
            Control::HistorySearch => return self.history_search.read(cx).focus.clone(),
            Control::HistoryClear => 514,
            Control::HistoryRefresh => 515,
            Control::HistoryKind(index) => 318 + index,
            Control::HistoryBaseline(index) => 322 + index,
            Control::HistoryRun(index) => 450 + index,
            Control::Shortcuts => 604,
            Control::ValidationNext => 605,
            Control::TaskDetails => 616,
            Control::TaskContinue => 617,
            Control::CloseOptions => 618,
            Control::DiscoveryDetails => 619,
            Control::CloseMappingPreview => 620,
            Control::TaskLink(index) => 621 + index,
        };
        self.handles[index].clone()
    }
    fn focused(&self, window: &Window, cx: &App) -> Option<Control> {
        self.visible_controls()
            .into_iter()
            .find(|c| self.control_enabled(*c, cx) && self.handle(*c, cx).is_focused(window))
    }
    fn move_focus(&mut self, reverse: bool, window: &mut Window, cx: &mut Context<Self>) {
        let controls: Vec<_> = self
            .visible_controls()
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
        self.reveal_history_control(controls[next]);
        self.reveal_form_control(controls[next], false, window, cx);
        self.reveal_review_control(controls[next], window, cx);
        self.reveal_task_control(controls[next]);
        window.focus(&self.handle(controls[next], cx));
        cx.notify();
    }
    fn reveal_history_control(&self, control: Control) {
        let (scroll, position) = match control {
            Control::HistoryBaseline(index) => (
                &self.history_baseline_scroll,
                self.history_baseline_indices()
                    .iter()
                    .position(|i| *i == index),
            ),
            Control::HistoryRun(index) => (
                &self.history_run_scroll,
                self.history_run_indices().iter().position(|i| *i == index),
            ),
            _ => return,
        };
        let Some(position) = position else { return };
        let top = position as f32 * 64.;
        let viewport = f32::from(scroll.bounds().size.height).max(64.);
        let offset = scroll.offset();
        let current_top = -f32::from(offset.y);
        let next_top = if top < current_top {
            top
        } else if top + 64. > current_top + viewport {
            top + 64. - viewport
        } else {
            current_top
        };
        scroll.set_offset(gpui::point(offset.x, px(-next_top)));
        let list = scroll.bounds();
        let screen = self.screen_scroll.bounds();
        let correction = if list.top() < screen.top() {
            list.top() - screen.top()
        } else if list.bottom() > screen.bottom() {
            list.bottom() - screen.bottom()
        } else {
            px(0.)
        };
        let offset = self.screen_scroll.offset();
        self.screen_scroll
            .set_offset(gpui::point(offset.x, offset.y - correction));
    }
    fn clear_hidden_history_previews(&mut self) {
        if !self
            .history_baseline_indices()
            .contains(&self.history_index)
            || !self.history_run_indices().contains(&self.run_history_index)
        {
            self.clear_portability_previews();
        }
    }
    fn history_row_visible(&self, control: Control) -> bool {
        let identity = (
            self.evidence.baselines.as_ptr() as usize,
            self.evidence.baselines.len(),
            self.history
                .as_ref()
                .map_or(0, |h| h.runs.as_ptr() as usize),
            self.history.as_ref().map_or(0, |h| h.runs.len()),
        );
        let mut cache = self.history_filter_cache.borrow_mut();
        if cache.identity != identity
            || cache.query != self.history_query
            || cache.kind != self.history_kind
        {
            cache.query = self.history_query.clone();
            cache.kind = self.history_kind;
            cache.identity = identity;
            cache.baselines = self.history_baseline_indices();
            cache.runs = self.history_run_indices();
        }
        match control {
            Control::HistoryBaseline(index) => {
                self.history_section == 1 && cache.baselines.contains(&index)
            }
            Control::HistoryRun(index) => self.history_section == 0 && cache.runs.contains(&index),
            _ => false,
        }
    }
    fn find_local(&mut self, _: &FindLocal, window: &mut Window, cx: &mut Context<Self>) {
        if self.task.is_some()
            || self.focus.dialog
            || self.focus.approval_dialog
            || self.native_prompt
        {
            return;
        }
        if self
            .crawl_fields
            .iter()
            .chain(self.analysis_fields.iter())
            .chain([&self.name, &self.address, &self.history_search])
            .any(|input| input.read(cx).focus.is_focused(window) && input.read(cx).composing())
        {
            return;
        }
        let control = match self.focus.screen {
            Screen::Results => Control::CrawlField(9),
            Screen::History => Control::HistorySearch,
            _ => return,
        };
        if self.control_enabled(control, cx) {
            window.focus(&self.handle(control, cx));
        }
    }
    fn queue(&mut self, work: Work, cx: &mut Context<Self>) {
        if self
            .task
            .as_ref()
            .is_some_and(|task| task.state == regressee_app::task::State::Finished)
            && regressee_app::task::Spec::for_work(&work).is_some()
        {
            return;
        }
        if matches!(&work, Work::Viewer { .. } | Work::HistoricalViewer { .. }) {
            self.reload_viewer(cx);
            return;
        }
        if self.pending.is_some() {
            return;
        }
        if matches!(&work, Work::NetworkStatus | Work::SaveNetwork(_)) {
            self.network_load_started = true;
            self.network_read_signature = Some((
                self.crawl_fields[17].read(cx).content.to_string(),
                self.crawl_fields[18].read(cx).content.to_string(),
            ));
        }
        if matches!(&work, Work::Evidence(_)) {
            self.invalidate_approval();
        }
        if matches!(
            &work,
            Work::Save { .. }
                | Work::SaveReplacement { .. }
                | Work::SaveEnvironment { .. }
                | Work::Import { .. }
                | Work::CaptureInventory { .. }
                | Work::Capture { .. }
                | Work::Activate { .. }
                | Work::Accept { .. }
                | Work::Restore { .. }
                | Work::Delete(_)
                | Work::CleanupTemporaries(_)
                | Work::DeleteUnreferenced(_)
        ) {
            self.projects_dirty = true;
            self.summary = None;
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
                | Work::Mask { .. }
                | Work::SaveEnvironment { .. }
        ) {
            self.viewer_pending = None;
            self.focus.view_loading = false;
            self.invalidate_approval();
            self.clear_portability_previews();
            self.history = None;
            self.project_storage = None;
            self.storage_pending = None;
        }
        if matches!(&work, Work::LoadAnalysis(_)) {
            self.viewer_pending = None;
            self.focus.view_loading = false;
            self.invalidate_approval();
            self.clear_portability_previews();
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
        let activity_label = work.activity_label();
        self.environment_read_signature =
            if matches!(&work, Work::EnvironmentStatus(_)) && self.environment.is_none() {
                Some((self.next_request, self.environment_draft_signature(cx)))
            } else {
                None
            };
        let task_spec = regressee_app::task::Spec::for_work(&work);
        let incoming_project = match &work {
            Work::Import { preview, .. } => Some(preview.name.clone()),
            _ => None,
        };
        let evidence_read = matches!(
            &work,
            Work::Load(_) | Work::Evidence(_) | Work::EnvironmentStatus(_)
        );
        self.focus.view_loading = matches!(&work, Work::Viewer { .. });
        self.focus.pausable = matches!(&work, Work::CaptureInventory { .. });
        match self.jobs.submit(Request {
            id: self.next_request,
            work,
        }) {
            Ok(()) => {
                self.pending = Some(self.next_request);
                self.focus.busy = true;
                self.progress = Some((activity_label.into(), 0, None));
                if let Some(spec) = task_spec {
                    self.begin_task(spec);
                    if let Some(name) = incoming_project {
                        let task = self.task.as_mut().unwrap();
                        task.project = name;
                        task.project_id = None;
                        // ImportPreview does not contain the incoming origins
                        // or environment; don't attribute the selected site's.
                        task.website.clear();
                        task.environment.clear();
                    }
                }
                if evidence_read {
                    self.evidence_read_failed = false;
                }
                self.error = None;
            }
            Err(error) => {
                self.error = Some(error.into());
                self.focus.busy = false;
                self.progress = None;
                if evidence_read {
                    self.evidence_read_failed = true;
                }
            }
        }
        cx.notify();
    }
    fn navigate(&mut self, screen: Screen, window: &mut Window, cx: &mut Context<Self>) {
        if self.task.is_some()
            || self.focus.dialog
            || self.focus.approval_dialog
            || self.native_prompt
        {
            return;
        }
        if self.focus.screen != screen && regressee_app::ui::performance::enabled() {
            self.navigation_started = Some((screen, std::time::Instant::now()));
        }
        if self.focus.screen != screen {
            self.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
        }
        self.focus.screen = screen;
        self.presentation_focus = None;
        if screen == Screen::Setup {
            self.focus.setup_stage = if self.project.is_some() {
                SetupStage::Settings
            } else {
                SetupStage::Website
            };
        }
        if screen == Screen::Results {
            self.focus.review_surface = ReviewSurface::Changes;
        }
        self.focus.choosing_project = false;
        self.error = None;
        window.focus(&self.handle(Control::Navigate(screen), cx));
        self.refresh_visible(cx);
        cx.notify();
    }
    fn create(&mut self, _: &CreateProject, window: &mut Window, cx: &mut Context<Self>) {
        if self.task.is_some()
            || self.focus.busy
            || self.focus.dialog
            || self.focus.approval_dialog
            || self.native_prompt
        {
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
        self.focus.setup_stage = SetupStage::Website;
        self.presentation_focus = None;
        self.focus.choosing_project = false;
        self.error = None;
        self.notice = None;
        window.focus(&self.handle(Control::Name, cx));
        cx.notify();
    }
    fn save(&mut self, _: &SaveProject, _: &mut Window, cx: &mut Context<Self>) {
        self.last_action = Some(Control::Save);
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
        if ((self.task.is_some() || !self.focus.can_activate(Control::Save))
            && !self.exit_guard.saving())
            || self.native_prompt
        {
            return;
        }
        if self.mask_dirty {
            self.error = Some("Save ignored areas before saving project settings. Your rectangles belong to the inspected evidence and are kept.".into());
            cx.notify();
            return;
        }
        let mut fields = vec![
            (Control::Name, self.name.clone()),
            (Control::Address, self.address.clone()),
        ];
        fields.extend(
            [0, 3, 4, 5, 6]
                .into_iter()
                .map(|i| (Control::CrawlField(i), self.crawl_fields[i].clone())),
        );
        if self.project.is_some() {
            fields.extend(
                [10, 11, 12]
                    .into_iter()
                    .map(|i| (Control::CrawlField(i), self.crawl_fields[i].clone())),
            );
        }
        if fields.iter().any(|(_, input)| input.read(cx).composing()) {
            return;
        }
        let mut first_invalid = None;
        for (control, input) in fields {
            input.update(cx, |input, cx| {
                input.touched = true;
                cx.notify();
            });
            if first_invalid.is_none() && input.read(cx).validation_error().is_some() {
                first_invalid = Some(control);
            }
        }
        if let Some(control) = first_invalid {
            match control {
                Control::Name | Control::Address => {
                    if self.focus.setup_stage != SetupStage::Settings {
                        self.focus.setup_stage = SetupStage::Website;
                    }
                }
                Control::CrawlField(0..=6) => {
                    self.focus.setup_stage = SetupStage::Settings;
                    self.focus.advanced = true;
                }
                Control::CrawlField(10..=12) => self.focus.setup_stage = SetupStage::Settings,
                _ => (),
            }
            self.presentation_focus = Some(control);
            cx.notify();
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
        if matches!(index, 48 | 49) {
            let count = self
                .approval
                .as_ref()
                .map_or(0, |p| p.baseline.references.len());
            self.approval_reference_index = if index == 48 {
                self.approval_reference_index.saturating_sub(1)
            } else {
                (self.approval_reference_index + 1).min(count.saturating_sub(1))
            };
            self.approval_provenance_scroll
                .set_offset(gpui::point(px(0.), px(0.)));
            cx.notify();
            return;
        }
        if index == 47 {
            self.focus.approval_dialog = false;
            self.presentation_focus = Some(Control::Phase5(24));
            cx.notify();
            return;
        }
        if index == 46 {
            self.viewer_fit = true;
            self.zoom = self.fitted_viewer_zoom();
            self.pan_x = 0.;
            self.pan_y = 0;
            self.reload_viewer(cx);
            cx.notify();
            return;
        }
        if index == 45 {
            self.result_filters_open = !self.result_filters_open;
            self.options_scroll.set_offset(gpui::point(px(0.), px(0.)));
            if self.result_filters_open {
                self.capture_details_open = false;
                self.review_tools = false;
                self.mask_draw = false;
                self.mask_drag = None;
            }
            cx.notify();
            return;
        }
        if index == 44 {
            self.review_tools = !self.review_tools;
            if self.review_tools {
                self.capture_details_open = false;
            }
            self.mask_draw = false;
            cx.notify();
            return;
        }
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
                if [7, 8]
                    .into_iter()
                    .any(|i| self.crawl_fields[i].read(cx).composing())
                {
                    return;
                }
                for i in [7, 8] {
                    self.crawl_fields[i].update(cx, |input, cx| {
                        input.touched = true;
                        cx.notify();
                    });
                }
                if let Some(i) = [7, 8]
                    .into_iter()
                    .find(|i| self.crawl_fields[*i].read(cx).validation_error().is_some())
                {
                    self.presentation_focus = Some(Control::CrawlField(i));
                    return;
                }
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
                    self.result_filters_open = false;
                    self.discovery_details_open = false;
                    self.notice = Some("Preview dismissed; the baseline is unchanged.".into());
                } else if let Some(p) = &self.project {
                    self.queue(Work::ExistingInventory(p.project.id), cx);
                }
            }
            17 => {
                self.search.clear();
                self.crawl_fields[9].update(cx, |input, cx| input.set_value("", cx));
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
                    if let Some(next) = local_ux::adjacent_visible(
                        &visible,
                        self.selected_page.as_ref(),
                        index == 23,
                    ) {
                        self.selected_page = visible.get(next).copied();
                    }
                } else if let Some(r) = &self.evidence.latest {
                    let indices = self.result_indices();
                    let current = indices
                        .iter()
                        .position(|i| Some(&r.run.targets[*i].key) == self.selected_key.as_ref());
                    let next = if index == 23 {
                        current
                            .map(|i| i.saturating_sub(1))
                            .unwrap_or(indices.len().saturating_sub(1))
                    } else {
                        current
                            .map(|i| (i + 1).min(indices.len().saturating_sub(1)))
                            .unwrap_or(0)
                    };
                    if let Some(i) = indices.get(next) {
                        if self.mask_dirty || self.mask_geometry_dirty(cx) {
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
                        self.mapping_preview_scroll
                            .set_offset(gpui::point(px(0.), px(0.)));
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
                if [13, 14]
                    .into_iter()
                    .any(|i| self.crawl_fields[i].read(cx).composing())
                {
                    return;
                }
                for i in [13, 14] {
                    self.crawl_fields[i].update(cx, |input, cx| {
                        input.touched = true;
                        input.correction = input.content.trim().is_empty().then(|| "Enter a complete http(s) origin or section base to add this mapping.".into());
                        cx.notify();
                    });
                }
                if let Some(i) = [13, 14].into_iter().find(|i| {
                    self.crawl_fields[*i].read(cx).content.trim().is_empty()
                        || self.crawl_fields[*i].read(cx).validation_error().is_some()
                }) {
                    self.presentation_focus = Some(Control::CrawlField(i));
                    return;
                }
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
                if self.network_status.is_none() {
                    return;
                }
                if self.crawl_fields[17].read(cx).composing()
                    || self.crawl_fields[18].read(cx).composing()
                {
                    return;
                }
                self.crawl_fields[17].update(cx, |input, cx| {
                    input.touched = true;
                    cx.notify();
                });
                if self.crawl_fields[17].read(cx).validation_error().is_some() {
                    self.presentation_focus = Some(Control::CrawlField(17));
                    return;
                }
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
            42 => {
                if self.network_status.is_none() {
                    self.queue(Work::NetworkStatus, cx);
                }
            }
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
        .when_some(self.options_control_bounds(control), |el, bounds| {
            el.relative().child(
                canvas(
                    |_, _, _| (),
                    move |painted, _, _, _| bounds.set(Some(painted)),
                )
                .absolute()
                .size_full(),
            )
        })
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
        .anchor_scroll(self.review_anchor(control))
        .when_some(self.options_control_bounds(control), |el, bounds| {
            el.relative().child(
                canvas(
                    |_, _, _| (),
                    move |painted, _, _, _| bounds.set(Some(painted)),
                )
                .absolute()
                .size_full(),
            )
        })
        .when(enabled, |el| {
            el.on_click(
                cx.listener(move |this, _, window, cx| this.activate_control(control, window, cx)),
            )
        })
        .into_any_element()
    }
    fn crawl_field(
        &self,
        index: usize,
        label: &'static str,
        help: &'static str,
    ) -> gpui::Stateful<Div> {
        controls::field(
            label,
            help,
            self.crawl_fields[index].clone().into_any_element(),
        )
        .id(("form-crawl-field", index))
        .anchor_scroll(self.form_anchor(Control::CrawlField(index)))
    }
    fn crawl_setup(&self, cx: &mut Context<Self>) -> AnyElement {
        let stage = self.focus.setup_stage;
        let website = matches!(stage, SetupStage::Website | SetupStage::Settings);
        let pages = matches!(stage, SetupStage::Pages | SetupStage::Settings);
        let d = &self.draft_settings.discovery;
        let approved = self.origins_confirmed
            && crawl::sources(&self.address.read(cx).content).is_ok_and(|sources| {
                sources
                    .iter()
                    .map(crawl::origin)
                    .all(|origin| self.approved_origins.contains(&origin))
            });
        let mut body=div().w_full().max_w(px(680.)).flex_shrink_0().flex().flex_col().gap_6()
            .child(self.setup_heading(cx))
            .when(!self.form_errors(cx).is_empty(), |body| body.child(self.form_error_summary(cx)))
            .when(website, |body| body.child(controls::section("Website", "Saving settings does not visit your website.")
                .child(controls::field("Project name","Choose a recognizable name.",self.name.clone().into_any_element()).id("form-project-name").anchor_scroll(self.form_anchor(Control::Name)))
                .child(controls::field("Starting addresses","Use complete http(s) addresses separated by spaces. Visits stay within the listed websites; subdomains are not added.",self.address.clone().into_any_element()).id("form-starting-addresses").anchor_scroll(self.form_anchor(Control::Address)))
                .child(self.check_button(Control::Guided(11), "Approve all listed websites", approved, cx))
                .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("Approval is required when you list more than one website. Adding a different website requires approval again."))))
            .when(pages, |body| body.child(controls::section("Pages to capture", "Choose how Regressee builds your page list.")
                .child(self.choice_button(Control::Choice(Choice::AddressesOnly), "Only the starting addresses", "Capture exactly the addresses you entered.", d.mode == DiscoveryMode::OnlyAddresses, cx))
                .child(self.choice_button(Control::Choice(Choice::AddressesAndLinks), "Find linked pages", "Follow rendered links within these websites, up to your page limit.", d.mode == DiscoveryMode::AddressesAndLinks, cx))
                .child(self.crawl_field(0,"Maximum pages","1–10,000. A page captured at three screen sizes creates three screenshots."))
                .child(self.crawl_field(1,"Website section (optional)","Use a path such as /docs to stay within that section. Leave blank for the whole listed website."))
                .child(self.crawl_field(2,"Skip addresses containing (optional)","Separate text exclusions with commas. The preview explains excluded addresses."))));
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
        if pages {
            body=body.child(controls::section("Screen sizes", "Choose one or more screen sizes. Phone and tablet use Chromium emulation, not real devices.")
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
        }
        if self.project.is_some() && matches!(stage, SetupStage::Access | SetupStage::Settings) {
            body = body.child(self.environment_setup(cx));
        }
        if pages {
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
        let login_fields = login_required || self.focus.setup_stage == SetupStage::Settings;
        let advanced = self.focus.advanced || self.focus.setup_stage == SetupStage::Settings;
        let mut body=controls::section("Environment & login", "Production and staging keep separate logins. Save your settings before opening a login browser.")
            .children(self.draft_settings.environments.iter().enumerate().map(|(index, environment)| self.choice_button(Control::Environment(index), environment.name.clone(), "", Some(environment.id) == self.environment_draft, cx)))
            .child(self.guided_button(27,"Add staging environment",cx))
            .when(self.environment_unsaved(cx), |el| el.child(controls::status("Unsaved environment edits — Save settings below before choosing another environment.", Tone::Information)))
            .child(self.choice_button(Control::Choice(Choice::PublicAccess), "No login required", "Use this for pages anyone can view.", !login_required, cx))
            .child(self.choice_button(Control::Choice(Choice::LoginRequired), "Login required", "Log in yourself in the app's browser, then confirm you are ready.", login_required, cx))
            .when(login_fields, |body| body.child(self.crawl_field(10,"Login / consent page (optional)","Blank opens the representative page. Use a complete URL, without credentials."))
            .child(self.crawl_field(11,"Representative page after login (optional)","Choose a real application page, not a login form. Blank uses the first mapped source. Replay is checked in a fresh isolated context."))
            .child(self.crawl_field(12,"Approved identity-provider origins (optional)","Exact http(s) origins ending in /, separated by spaces. Approve only providers you trust. This does not expand crawl scope."))
            .child(self.choice_button(Control::Choice(Choice::SaveLogin), "Remember login securely", "Use encrypted storage and your operating system's secure store.", !self.login_ephemeral, cx))
            .child(self.choice_button(Control::Choice(Choice::EphemeralLogin), "Only for this app session", "Forget the login when Regressee closes.", self.login_ephemeral, cx))
            .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child("If secure storage is locked or unavailable, unlock it or explicitly choose Ephemeral. There is no plaintext fallback. Screenshots may still contain private website content.")))
            .when(self.focus.setup_stage == SetupStage::Access, |body| body.child(self.guided_button(6, if advanced { "Hide environment options" } else { "Mapping, language & website theme" }, cx)))
            .when(advanced, |body| body.child(div().font_weight(FontWeight::SEMIBOLD).mt_3().child("Production → staging mapping"))
            .child(self.crawl_field(13,"Source origin or section base","For example https://production.example/app. No query or fragment in the base."))
            .child(self.crawl_field(14,"Matching destination base","For example https://staging.example/preview. Relative paths, ordered query pairs and hash routes are preserved."))
            .child(div().flex().flex_wrap().gap_2().child(self.guided_button(37,"Approve / replace this base mapping",cx)).child(self.guided_button(38,"Use same website for every mapping",cx))));
        if let Some(env) = draft.filter(|_| advanced) {
            body = body.children(env.mappings.iter().take(16).map(|m| {
                div()
                    .text_size(px(13.))
                    .child(format!("{} → {}", m.source, m.destination))
            }));
        }
        body=body.when(advanced, |body| body.child(self.crawl_field(15,"Capture language / locale","For example en-US or nb-NO. Changing rendering settings requires a replacement baseline."))
            .child(controls::field("Capture timezone", "Choose a timezone or search by region or city. Changes apply to future baseline captures.", self.timezone.clone().into_any_element()))
            .child(self.choice_button(Control::Choice(Choice::LightTheme), "Light website theme", "Capture the website's light color scheme.", self.draft_settings.profiles.first().is_none_or(|profile| !profile.dark), cx))
            .child(self.choice_button(Control::Choice(Choice::DarkTheme), "Dark website theme", "Capture the website's dark color scheme.", self.draft_settings.profiles.first().is_some_and(|profile| profile.dark), cx)))
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
        let login_required = self
            .draft_settings
            .environments
            .iter()
            .find(|e| Some(e.id) == self.environment_draft)
            .is_some_and(|e| e.access == regressee_core::domain::AccessPolicy::InteractiveLogin);
        body = body
            .when(
                login_required || self.focus.setup_stage == SetupStage::Settings,
                |body| {
                    body.child(
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
                },
            )
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(self.guided_button(35, "Preview mapped URLs", cx))
                    .child(self.guided_button(36, "Probe representative pages", cx)),
            );
        if self.show_mapping && self.environment.is_some() {
            body = body.child(self.mapping_preview(cx));
        }
        body.when(
            login_required || self.focus.setup_stage == SetupStage::Settings,
            |body| {
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
            },
        )
        .into_any_element()
    }
    fn preview_page_ids(&self) -> Vec<regressee_core::domain::PageId> {
        let Some(preview) = &self.preview else {
            return vec![];
        };
        let mut cache = self.local_index.borrow_mut();
        if cache.preview.as_ref().is_none_or(|(id, len, _)| {
            *id != preview.inventory.id || *len != preview.inventory.pages.len()
        }) {
            cache.preview = Some((
                preview.inventory.id,
                preview.inventory.pages.len(),
                preview
                    .inventory
                    .pages
                    .iter()
                    .map(|p| (p.id, p.canonical.as_str().to_lowercase()))
                    .collect(),
            ));
        }
        cache
            .preview
            .as_ref()
            .unwrap()
            .2
            .iter()
            .filter(|(_, url)| url.contains(&self.search))
            .map(|(id, _)| *id)
            .collect()
    }
    fn select_preview_page(&mut self, id: regressee_core::domain::PageId) {
        if self.preview_page_ids().contains(&id) {
            self.selected_page = Some(id);
        }
    }
    fn result_indices(&self) -> Vec<usize> {
        self.local_index.borrow_mut().results(
            &self.evidence,
            &self.search,
            self.result_filter,
            self.group_profiles,
        )
    }
    fn result_filter_options(&self, cx: &mut Context<Self>) -> AnyElement {
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(
                div().flex().flex_col().gap_2().children(
                    Choice::RESULTS
                        .into_iter()
                        .enumerate()
                        .map(|(index, choice)| {
                            let control = Control::Choice(choice);
                            self.choice_button(
                                control,
                                ["All", "Changed", "Gaps", "No significant change"][index],
                                [
                                    "Every screenshot target.",
                                    "Visual differences to review, not confirmed defects.",
                                    "Missing captures or targets that could not be compared.",
                                    "Comparable captures below the current change thresholds.",
                                ][index],
                                self.result_filter == index,
                                cx,
                            )
                        }),
                ),
            )
            .child(self.check_button(
                Control::Guided(19),
                "Group by screen size",
                self.group_profiles,
                cx,
            ))
            .into_any_element()
    }
    fn inventory_and_results(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.preview.is_none() && self.evidence.latest.is_none() {
            return div().into_any_element();
        }
        let fixed_pages =
            self.preview.is_none() && self.focus.review_surface == ReviewSurface::Pages;
        let empty_results = self.preview.is_none()
            && !fixed_pages
            && self.evidence.latest.is_some()
            && self.result_indices().is_empty();
        let mut body = div()
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_2()
            .when(self.narrow_workspace, |body| body.gap_1())
            .when(!empty_results, |body| {
                body.child(
                    div()
                        .font_weight(FontWeight::SEMIBOLD)
                        .text_size(px(14.))
                        .child(if self.preview.is_some() {
                            "Pages to capture".to_owned()
                        } else if fixed_pages {
                            "Recorded page list".to_owned()
                        } else {
                            format!(
                                "{} · {}",
                                if self.evidence.comparison.is_some() {
                                    "Results"
                                } else {
                                    "Screenshots"
                                },
                                ["All", "Changed", "Gaps", "Unchanged"][self.result_filter]
                            )
                        }),
                )
            })
            .when(empty_results, |body| {
                body.child(
                    div()
                        .debug_selector(|| "review-empty-message".into())
                        .w_full()
                        .min_w_0()
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(
                            div()
                                .w_full()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_size(px(14.))
                                .line_height(px(20.))
                                .child("No matches"),
                        )
                        .child(
                            div()
                                .w_full()
                                .text_size(px(13.))
                                .line_height(px(18.))
                                .text_color(rgb(MUTED))
                                .child(if self.search.is_empty() {
                                    "Filter → All."
                                } else {
                                    "Clear search."
                                })
                                .when(self.selected_key.is_some(), |el| {
                                    el.child(
                                        div()
                                            .debug_selector(|| "empty-retained-evidence".into())
                                            .w_full()
                                            .text_size(px(13.))
                                            .line_height(px(18.))
                                            .text_color(rgb(MUTED))
                                            .child("Inspected screenshot and edits are kept."),
                                    )
                                }),
                        ),
                )
            })
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .items_center()
                    .gap_2()
                    .child(div().text_size(px(13.)).child("Search"))
                    .child(div().flex_1().min_w_0().child(self.crawl_fields[9].clone()))
                    .when(!self.search.is_empty(), |el| {
                        el.child(self.guided_button(17, "Clear", cx))
                    }),
            )
            .when(!fixed_pages, |body| {
                body.child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .child(self.button(
                            Control::Guided(45),
                            if self.preview.is_some() {
                                "Page options"
                            } else {
                                "Filter"
                            },
                            ActionStyle::Choice {
                                selected: self.result_filters_open,
                            },
                            cx,
                        ))
                        .when(!empty_results, |actions| {
                            actions.child(self.guided_button(
                                23,
                                if self.narrow_workspace {
                                    "Prev"
                                } else {
                                    "Previous"
                                },
                                cx,
                            ))
                        })
                        .when(!empty_results, |actions| {
                            actions.child(self.guided_button(24, "Next", cx))
                        }),
                )
            });
        if let Some(p) = &self.preview {
            body = body.child(
                div()
                    .flex_shrink_0()
                    .text_size(px(13.))
                    .line_height(px(18.))
                    .text_color(rgb(MUTED))
                    .child(self.discovery_coverage_hint()),
            );
            let rows = self.preview_page_ids();
            body = body.child(
                div()
                    .text_size(px(13.))
                    .text_color(rgb(MUTED))
                    .child(format!(
                        "{} of {} shown",
                        rows.len(),
                        workflow::count_label(
                            p.inventory.pages.len(),
                            "page address",
                            "page addresses"
                        )
                    )),
            );
            if self
                .selected_page
                .is_some_and(|selected| !rows.contains(&selected))
            {
                body = body.child(div().id("selected-page-outside-search").debug_selector(|| "selected-page-outside-search".into()).text_size(px(13.)).text_color(rgb(MUTED)).child("Selected page is outside search. Previous / Next selects a match; inclusion is unchanged."));
            }
            if rows.is_empty() {
                body = body.child(controls::empty(
                    "No matching pages",
                    "Change or clear the search text. Your capture list is unchanged.",
                ));
            } else {
                body = body.child(
                    controls::page_list(
                        uniform_list(
                            "inventory-preview",
                            rows.len(),
                            cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                                range
                                    .map(|visible| {
                                        let id = rows[visible];
                                        let Some(p) = this.preview.as_ref().and_then(|p| {
                                            p.inventory.pages.iter().find(|p| p.id == id)
                                        }) else {
                                            return div().into_any_element();
                                        };
                                        controls::page_row(
                                            SharedString::from(format!("candidate-{id}")),
                                            p.canonical.to_string(),
                                            if p.included {
                                                "Included".to_owned()
                                            } else {
                                                p.reason
                                                    .clone()
                                                    .unwrap_or_else(|| "Excluded".into())
                                            },
                                            Some(id) == this.selected_page,
                                            this.pending.is_none(),
                                        )
                                        .on_click(cx.listener(move |this, _, _, cx| {
                                            if this.pending.is_none() {
                                                this.select_preview_page(id);
                                                cx.notify();
                                            }
                                        }))
                                        .into_any_element()
                                    })
                                    .collect::<Vec<_>>()
                            }),
                        )
                        .debug_selector(|| "inventory-preview".into())
                        .w_full()
                        .h_full()
                        .flex_1()
                        .min_h_0(),
                    )
                    .flex_1()
                    .min_h(px(PAGE_ROW_HEIGHT * 2.))
                    .flex()
                    .flex_col(),
                );
            }
        } else if fixed_pages {
            let run = self.evidence.latest.as_ref().unwrap();
            let query = self.search.trim().to_lowercase();
            let rows: Vec<_> = run
                .run
                .inventory
                .pages
                .iter()
                .enumerate()
                .filter(|(_, page)| page.canonical.as_str().to_lowercase().contains(&query))
                .map(|(i, _)| i)
                .collect();
            body = body.child(
                div()
                    .text_size(px(13.))
                    .text_color(rgb(MUTED))
                    .child(format!(
                        "{} of {} shown",
                        rows.len(),
                        workflow::count_label(
                            run.run.inventory.pages.len(),
                            "page address",
                            "page addresses"
                        )
                    )),
            );
            if rows.is_empty() {
                body = body.child(controls::empty(
                    "No matching pages",
                    "Change or clear search. The fixed baseline page list is unchanged.",
                ));
            } else {
                body = body.child(
                    controls::page_list(
                        uniform_list(
                            "fixed-inventory",
                            rows.len(),
                            cx.processor(move |this, range: std::ops::Range<usize>, _, _| {
                                let Some(run) = &this.evidence.latest else {
                                    return Vec::new();
                                };
                                range
                                    .map(|visible| {
                                        let page = &run.run.inventory.pages[rows[visible]];
                                        controls::page_row(
                                            SharedString::from(format!("fixed-page-{}", page.id)),
                                            page.canonical.to_string(),
                                            if page.included {
                                                "Included in this capture list".into()
                                            } else {
                                                page.reason
                                                    .clone()
                                                    .unwrap_or_else(|| "Excluded".into())
                                            },
                                            false,
                                            false,
                                        )
                                        .into_any_element()
                                    })
                                    .collect::<Vec<_>>()
                            }),
                        )
                        .w_full()
                        .h_full()
                        .flex_1()
                        .min_h_0(),
                    )
                    .flex_1()
                    .min_h(px(PAGE_ROW_HEIGHT * 2.))
                    .flex()
                    .flex_col(),
                );
            }
        } else if let Some(r) = &self.evidence.latest {
            let rows = self.result_indices();
            body = body.when(
                matches!(r.run.state, JobState::Paused | JobState::Interrupted),
                |el| {
                    el.child(self.guided_button(15, "Resume — same website release confirmed", cx))
                },
            );
            if !rows.is_empty() {
                body = body.child(
                    div()
                        .text_size(px(13.))
                        .text_color(rgb(MUTED))
                        .child(format!(
                            "{} of {} {} shown",
                            rows.len(),
                            r.run.targets.len(),
                            if r.run.targets.len() == 1 {
                                "screenshot"
                            } else {
                                "screenshots"
                            }
                        )),
                );
            }
            if !rows.is_empty()
                && self
                    .selected_key
                    .as_ref()
                    .is_some_and(|key| !rows.iter().any(|i| &r.run.targets[*i].key == key))
            {
                body = body.child(div().id("inspected-outside-filter").debug_selector(|| "inspected-outside-filter".into()).text_size(px(13.)).text_color(rgb(MUTED)).child("Inspected screenshot is outside this filter. Evidence and ignored-area edits are kept."));
            }
            if !rows.is_empty() {
                body = body.child(controls::page_list(
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
                                    .filter(|a| a.revision.run == r.run.id && matches!(t.outcome, TargetOutcome::Captured { .. }))
                                    .and_then(|a| a.revision.results.iter().find(|a| a.key == key))
                                    .map(|a| a.classification.label().to_owned())
                                    .unwrap_or_else(|| match t.outcome {
                                        TargetOutcome::Captured { .. } => {
                                            this.captured_target_label(&key).into()
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
                                let attempt_word = if t.attempts == 1 { "attempt" } else { "attempts" };
                                controls::page_row(
                                    SharedString::from(format!("target-{index}")),
                                    t.requested.to_string(),
                                        format!(
                                            "{} · {classification} · {} {attempt_word}",
                                            profile.name, t.attempts
                                        ),
                                    this.selected_key.as_ref() == Some(&key),
                                    this.pending.is_none(),
                                )
                                    .on_click(cx.listener(move |this, _, _, cx| {
                                        if this.pending.is_none() {
                                             if this.mask_dirty || this.mask_geometry_dirty(cx) {
                                                this.error = Some("Save ignored areas before changing targets; rectangles will not be moved to another page/profile.".into());
                                                cx.notify();
                                                return;
                                            }
                                            this.selected_key = Some(key.clone());
                                            this.invalidate_approval();
                                            this.pan_x = 0.;
                                            this.pan_y = 0;
                                            this.clear_viewer();
                                            if let Some(p) = &this.project {
                                                this.queue(
                                                    Work::Viewer {
                                                        analysis: this.evidence.comparison.as_ref().map(|a| a.revision.id),
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
                .w_full()
                .h_full().flex_1().min_h_0(),
            ).flex_1().min_h(px(PAGE_ROW_HEIGHT)).flex().flex_col());
            }
        }
        body.into_any_element()
    }
    fn activate_control(&mut self, control: Control, window: &mut Window, cx: &mut Context<Self>) {
        if !self.control_enabled(control, cx) {
            return;
        }
        self.last_action = Some(control);
        if self.mask_dirty
            && matches!(
                control,
                Control::RefreshEvidence
                    | Control::CaptureBaseline
                    | Control::CheckChanges
                    | Control::Phase5(0..=2 | 4 | 26 | 30)
                    | Control::Guided(13 | 16)
                    | Control::Phase5(42)
                    | Control::Phase6(4..=6)
            )
        {
            self.error = Some("Save ignored areas before reloading or replacing this evidence. Your unsaved rectangles are kept.".into());
            cx.notify();
            return;
        }
        match control {
            Control::CloseOptions => {
                self.result_filters_open = false;
                window.focus(&self.handle(Control::Guided(45), cx));
                cx.notify();
            }
            Control::DiscoveryDetails => {
                self.discovery_details_open = !self.discovery_details_open;
                cx.notify();
            }
            Control::CloseMappingPreview => {
                self.show_mapping = false;
                self.reveal_form_control(Control::Guided(35), false, window, cx);
                window.focus(&self.handle(Control::Guided(35), cx));
                cx.notify();
            }
            Control::TaskContinue => self.continue_task(window, cx),
            Control::TaskLink(index) => {
                if let Some(url) = self.task.as_ref().and_then(|task| task.link_url(index)) {
                    cx.open_url(url.as_str());
                    window.focus(&self.handle(control, cx));
                }
            }
            Control::TaskDetails => {
                if let Some(task) = &mut self.task {
                    task.details = !task.details;
                    if task.details {
                        task.reveal_details.set(true);
                    } else {
                        task.reveal_details.set(false);
                        task.scroll.set_offset(gpui::point(px(0.), px(0.)));
                    }
                }
                window.focus(&self.handle(control, cx));
                cx.notify();
            }
            Control::Navigate(screen) => self.navigate(screen, window, cx),
            Control::SetupStage(stage) => {
                self.focus.setup_stage = stage;
                self.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                window.focus(&self.handle(control, cx));
                cx.notify();
            }
            Control::PagesBaseline => {
                self.navigate(Screen::Results, window, cx);
                self.focus.review_surface = ReviewSurface::Pages;
                window.focus(&self.handle(control, cx));
                cx.notify();
            }
            Control::NewProject => self.create(&CreateProject, window, cx),
            Control::Save => self.save(&SaveProject, window, cx),
            Control::ChooseProject => {
                self.focus.choosing_project = true;
                self.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                self.notice = None;
                self.error = None;
                self.refresh_visible(cx);
                window.focus(&self.handle(Control::Navigate(self.focus.screen), cx));
                cx.notify();
            }
            Control::CancelProjectChoice => {
                self.navigate(self.focus.screen, window, cx);
            }
            Control::OpenProject(index) | Control::SelectProject(index) => {
                let Some(project) = self.projects.items.get(index) else {
                    return;
                };
                let id = project.id;
                let screen = if matches!(control, Control::OpenProject(_)) {
                    Screen::Setup
                } else if matches!(self.focus.screen, Screen::Projects | Screen::Settings) {
                    Screen::Results
                } else {
                    self.focus.screen
                };
                // Reopening the current project is navigation, not a reload or
                // a reason to discard edits/ignored areas.
                if self.project.as_ref().is_some_and(|p| p.project.id == id) {
                    self.navigate(screen, window, cx);
                    return;
                }
                if self.unsaved(cx) {
                    self.confirm_switch(control, window, cx);
                    return;
                }
                self.focus.screen = screen;
                self.focus.choosing_project = true;
                self.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                self.notice = None;
                self.queue(Work::Load(id), cx);
                window.focus(&self.handle(Control::Navigate(screen), cx));
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
                self.help_dialog = false;
                self.focus.dialog = true;
                window.focus(&self.handle(Control::CloseDialog, cx));
                cx.notify();
            }
            Control::CloseDialog => self.dismiss(&Dismiss, window, cx),
            Control::Shortcuts => self.show_keyboard_help(window, cx),
            Control::ValidationNext => self.go_to_next_correction(window, cx),
            Control::Name
            | Control::Address
            | Control::CrawlField(_)
            | Control::AnalysisField(_) => (),
            Control::Phase5(index) => self.phase5_action(index, cx),
            Control::Phase6(index) => self.phase6_action(index, window, cx),
            Control::Phase7(index) => self.phase7_action(index, window, cx),
            Control::HistorySearch => window.focus(&self.handle(control, cx)),
            Control::HistoryRefresh => {
                if let Some(project) = self.project.as_ref().map(|p| p.project.id) {
                    let page = self.history.as_ref().map_or(0, |h| h.page);
                    self.clear_portability_previews();
                    self.invalidate_approval();
                    // History returns current baseline records too. Do not
                    // replace inspected run evidence or dirty mask drafts.
                    self.queue(Work::History { project, page }, cx);
                }
            }
            Control::HistoryClear => {
                self.history_query.clear();
                self.history_search
                    .update(cx, |input, cx| input.set_value("", cx));
                window.focus(&self.handle(Control::HistorySearch, cx));
                cx.notify();
            }
            Control::HistoryKind(index) => {
                self.history_kind = index;
                self.clear_hidden_history_previews();
                cx.notify();
            }
            Control::HistoryBaseline(index) => {
                if self.history_index != index {
                    self.clear_portability_previews();
                    self.history_index = index;
                }
                cx.notify();
            }
            Control::HistoryRun(index) => {
                self.history_selection_epoch = self.history_selection_epoch.wrapping_add(1);
                if self.run_history_index != index {
                    self.clear_portability_previews();
                    self.run_history_index = index;
                }
                cx.notify();
            }
            Control::Guided(46) => {
                self.zoom_select
                    .update(cx, |select, cx| select.toggle(window, cx));
            }
            Control::Guided(index) => {
                if !self.toggle_workspace_presentation(control) {
                    self.guided(index, cx);
                } else {
                    cx.notify();
                }
                if index == 17 {
                    window.focus(&self.handle(Control::CrawlField(9), cx));
                } else if index == 45 {
                    window.focus(&self.handle(
                        if self.result_filters_open {
                            Control::CloseOptions
                        } else {
                            control
                        },
                        cx,
                    ));
                }
            }
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
                if self.task.is_some() {
                    self.request_task_stop(true, window, cx);
                    return;
                }
                self.jobs.pause();
                self.notice=Some("Pause requested. No new batch will start. Active pages finish their bounded browser work (default budget 55 seconds), then publish evidence. Large image processing may add time.".into());
                cx.notify();
            }
            Control::CancelCapture => {
                if self.task.is_some() {
                    self.request_task_stop(false, window, cx);
                    return;
                }
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
                    if self.focus.capturing {
                        self.focus.review_surface = ReviewSurface::Capture;
                    }
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
            Control::Viewer => (),
            Control::ZoomIn
            | Control::ZoomOut
            | Control::PanLeft
            | Control::PanRight
            | Control::PanUp
            | Control::PanDown => {
                match control {
                    Control::ZoomIn => {
                        self.viewer_fit = false;
                        self.zoom = (self.zoom * 2.).min(4.);
                    }
                    Control::ZoomOut => {
                        self.viewer_fit = false;
                        self.zoom = (self.zoom / 2.).max(0.05);
                    }
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
                        if let Some((project, run)) = self.history_inspection {
                            Work::HistoricalViewer {
                                project,
                                run,
                                tile,
                                key: self.selected_key.clone(),
                            }
                        } else {
                            Work::Viewer {
                                analysis: self.evidence.comparison.as_ref().map(|a| a.revision.id),
                                project: p.project.id,
                                tile,
                                key: self.selected_key.clone(),
                            }
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
        if let Some(task) = &self.task {
            if task.state == regressee_app::task::State::Finished {
                self.continue_task(window, cx);
            } else {
                self.request_task_stop(false, window, cx);
            }
            return;
        }
        if self.focus.approval_dialog {
            self.guided(47, cx);
            window.focus(&self.handle(Control::Phase5(24), cx));
            return;
        }
        if self.diagnostic_preview.take().is_some() {
            window.focus(&self.handle(Control::Phase7(0), cx));
            cx.notify();
        }
        if self.focus.dialog {
            self.focus.dialog = false;
            self.help_dialog = false;
            let target = self
                .restore_focus
                .take()
                .unwrap_or(Control::Navigate(self.focus.screen));
            window.focus(&self.handle(target, cx));
            cx.notify();
        } else if self.result_filters_open && self.focus.screen == Screen::Results {
            self.result_filters_open = false;
            window.focus(&self.handle(Control::Guided(45), cx));
            cx.notify();
        } else if self.show_mapping && self.focus.screen == Screen::Setup {
            self.show_mapping = false;
            self.reveal_form_control(Control::Guided(35), false, window, cx);
            window.focus(&self.handle(Control::Guided(35), cx));
            cx.notify();
        } else if self.capture_details_open || self.review_tools {
            let control = if self.capture_details_open {
                Control::Phase5(36)
            } else {
                Control::Guided(44)
            };
            self.capture_details_open = false;
            self.review_tools = false;
            self.mask_draw = false;
            self.mask_drag = None;
            window.focus(&self.handle(control, cx));
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
        .when(
            self.narrow_workspace && self.focus.screen == Screen::Results,
            |el| el.px(px(8.)),
        )
        .anchor_scroll(self.review_anchor(control))
        .when_some(self.options_control_bounds(control), |el, bounds| {
            el.relative().child(
                canvas(
                    |_, _, _| (),
                    move |painted, _, _, _| bounds.set(Some(painted)),
                )
                .absolute()
                .size_full(),
            )
        })
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
        if self.task.is_some() {
            return !self.native_prompt && self.visible_controls().contains(&control);
        }
        if !self.focus.can_activate(control)
            || !self.control_mounted(control)
            || self.native_prompt
            || (self.focus.approval_dialog && self.focus.busy)
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
        if matches!(control, Control::Phase5(14..=19)) && !self.viewer_current() {
            return false;
        }
        match control {
            Control::ValidationNext => !self.form_errors(cx).is_empty(),
            Control::Phase6(19) => {
                self.history_run_indices().contains(&self.run_history_index)
                    && !self.unsaved(cx)
                    && !self.mask_geometry_dirty(cx)
                    && !self.review_unsaved(cx)
            }
            Control::SetupStage(stage) => {
                // Setup progression remains deliberate.
                !self.focus.busy
                    && (stage == SetupStage::Website || self.project.is_some())
                    && (stage != SetupStage::Pages || !self.unsaved(cx))
            }
            Control::PagesBaseline => self.project.is_some(),
            Control::HistorySearch => true,
            Control::HistoryRefresh => self.project.is_some() && self.pending.is_none(),
            Control::HistoryClear => !self.history_query.is_empty(),
            Control::HistoryKind(index) => index < 4,
            Control::HistoryBaseline(_) | Control::HistoryRun(_) => {
                self.history_row_visible(control)
            }
            Control::Phase6(8) => self.storage_pending.is_none(),
            Control::Guided(41) => self.network_status.is_some(),
            Control::Guided(42) => true,
            Control::Guided(48) => self.approval_reference_index > 0,
            Control::Guided(49) => self
                .approval
                .as_ref()
                .is_some_and(|p| self.approval_reference_index + 1 < p.baseline.references.len()),
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
                comparison.is_some()
                    && (index >= 9 || self.analysis_advanced)
                    && (index < 10
                        || (self.viewer_current()
                            && self
                                .mask_selected
                                .is_some_and(|i| i < self.mask_rectangles.len())))
                    && self.analysis_field_active(index, cx)
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
            Control::Guided(20 | 21) => {
                let visible: std::collections::BTreeSet<_> =
                    self.preview_page_ids().into_iter().collect();
                self.preview.as_ref().is_some_and(|p| {
                    p.inventory.pages.iter().any(|page| {
                        visible.contains(&page.id)
                            && if control == Control::Guided(20) {
                                page.included
                            } else {
                                page.reason.as_deref() == Some("Excluded in preview")
                            }
                    })
                })
            }
            Control::Guided(22) => self.selected_page.is_some_and(|id| {
                self.preview_page_ids().contains(&id)
                    && self.preview.as_ref().is_some_and(|p| {
                        p.inventory.pages.iter().any(|page| {
                            page.id == id
                                && (page.included
                                    || page.reason.as_deref() == Some("Excluded in preview"))
                        })
                    })
            }),
            Control::Guided(23 | 24) => {
                let (index, count) = if self.preview.is_some() {
                    let rows = self.preview_page_ids();
                    (
                        rows.iter()
                            .position(|id| Some(*id) == self.selected_page)
                            .unwrap_or(usize::MAX),
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
                            .unwrap_or(usize::MAX),
                        rows.len(),
                    )
                };
                count > 0
                    && if index == usize::MAX {
                        true
                    } else if control == Control::Guided(23) {
                        index > 0
                    } else {
                        index + 1 < count
                    }
            }
            Control::Guided(27) => {
                self.draft_settings.environments.len() < 16 && !self.environment_unsaved(cx)
            }
            Control::Guided(31 | 36) => !self.environment_unsaved(cx),
            Control::Guided(32) => {
                !self.environment_unsaved(cx)
                    && self
                        .environment
                        .as_ref()
                        .is_some_and(|environment| environment.login_open)
            }
            Control::Guided(33) => self
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
                    && self.viewer.as_ref().is_some_and(|viewer| {
                        viewer
                            .request
                            .as_ref()
                            .is_some_and(|r| r.validate().is_ok())
                    })
            }
            Control::Phase5(19) => self
                .viewer
                .as_ref()
                .and_then(|viewer| viewer.request.as_ref())
                .is_some_and(|request| request.validate().is_err()),
            Control::Phase5(20..=22 | 32) => self.review_result_valid(),
            Control::Phase5(33) => {
                review_ux::navigation_preserves_drafts(
                    self.mask_dirty,
                    self.mask_geometry_dirty(cx),
                    self.review_note_dirty(cx),
                ) && self.next_unreviewed_key().is_some()
            }
            Control::Phase5(34) => {
                !self.mask_geometry_dirty(cx)
                    && self.viewer_current()
                    && self
                        .viewer
                        .as_ref()
                        .and_then(|v| v.request.as_ref())
                        .is_some_and(|r| r.validate().is_ok())
                    && self.mask_rectangles.len() < 1000
            }
            Control::Phase5(35) => {
                self.viewer_current()
                    && self
                        .mask_selected
                        .is_some_and(|i| i < self.mask_rectangles.len())
            }
            Control::Phase5(23) => self.selected_key.as_ref().is_some_and(|key| {
                self.approval_selection.contains(key) || self.approval_eligible(key)
            }),
            Control::Phase5(24) => {
                !self.approval_selection.is_empty()
                    && !self.mask_dirty
                    && self.approval_selection_eligible()
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
            Control::Phase5(40) => self
                .history_baseline_indices()
                .iter()
                .position(|i| *i == self.history_index)
                .is_some_and(|i| i > 0),
            Control::Phase5(41) => {
                let rows = self.history_baseline_indices();
                rows.iter()
                    .position(|i| *i == self.history_index)
                    .is_some_and(|i| i + 1 < rows.len())
            }
            Control::Phase5(42) => {
                self.project.is_some()
                    && self
                        .evidence
                        .baselines
                        .get(self.history_index)
                        .is_some_and(|baseline| self.history_active_baseline() != Some(baseline.id))
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
            Control::Phase6(9) => {
                let rows = self.history_run_indices();
                rows.iter()
                    .position(|i| *i == self.run_history_index)
                    .is_some_and(|i| i + 1 < rows.len())
            }
            Control::Phase6(10) => self
                .history_run_indices()
                .iter()
                .position(|i| *i == self.run_history_index)
                .is_some_and(|i| i > 0),
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
        if self.task.is_some() {
            return self.visible_controls().contains(&control);
        }
        use Control::*;
        let screen = self.focus.screen;
        if self.focus.approval_dialog {
            return match control {
                Guided(47..=49) => true,
                Phase5(25) => self
                    .approval
                    .as_ref()
                    .is_some_and(|p| self.approval_matches(p)),
                _ => false,
            };
        }
        if !self.stage_control_mounted(control) || !self.workspace_control_mounted(control) {
            return false;
        }
        if self.history_inspection.is_some()
            && matches!(control,
            CaptureBaseline | SaveBaseline | CheckChanges | Guided(15)
                | Phase5(0..=5 | 14..=28 | 32..=35 | 37) | AnalysisField(_))
        {
            return false;
        }
        if screen == Screen::Setup
            && !matches!(
                control,
                Navigate(_)
                    | Shortcuts
                    | SetupStage(_)
                    | PagesBaseline
                    | ChooseProject
                    | CancelProjectChoice
                    | CloseDialog
                    | DismissFeedback
                    | CancelCapture
                    | PauseCapture
            )
            && !self.focus.setup_stage.mounts(control)
        {
            return false;
        }
        let picker = self.focus.shows_project_picker();
        if picker {
            return match control {
                Navigate(_) => true,
                Shortcuts => screen == Screen::Settings,
                SetupStage(_) | PagesBaseline => false,
                CloseDialog => self.focus.dialog,
                DismissFeedback => self.error.is_some() || self.notice.is_some(),
                CancelProjectChoice => self.focus.choosing_project && self.project.is_some(),
                NewProject => true,
                SelectProject(index) => index < self.projects.items.len(),
                OpenProject(index) => {
                    screen == Screen::Projects && index < self.projects.items.len()
                }
                PreviousPage => self.projects.page > 0,
                NextPage => self.projects.more,
                Phase6(0..=15) if screen == Screen::Projects => match control {
                    Phase6(0) => true,
                    Phase6(1) => self.import_preview.is_some(),
                    Phase6(3) => self.export_preview.is_some(),
                    Phase6(6) => self.deletion_preview.is_some(),
                    Phase6(7) => {
                        self.import_preview.is_some()
                            || self.export_preview.is_some()
                            || self.deletion_preview.is_some()
                            || self.repair_preview.is_some()
                    }
                    Phase6(14 | 15) => self.repair_preview.is_some(),
                    _ => false,
                },
                _ => false,
            };
        }
        let project = self.project.is_some();
        let results = screen == Screen::Results && project;
        let setup = screen == Screen::Setup;
        let history = screen == Screen::History && project;
        let settings = screen == Screen::Settings;
        let portability = screen == Screen::Projects || history || settings;
        let inventory = results && (self.preview.is_some() || self.evidence.latest.is_some());
        let analysis = results && self.evidence.comparison.is_some() && !self.page_list_workspace();
        let viewer = results
            && self.viewer.is_some()
            && (!self.page_list_workspace() || self.discovery_image_mounted());
        match control {
            TaskDetails | TaskContinue | TaskLink(_) => false,
            CloseOptions => results && self.result_filters_open,
            DiscoveryDetails => results && self.preview.is_some() && self.result_filters_open,
            CloseMappingPreview => setup && self.show_mapping && self.environment.is_some(),
            Navigate(_) => true,
            Shortcuts => settings,
            ValidationNext => setup || settings,
            HistorySearch | HistoryRefresh => history && self.history_section != 2,
            HistoryKind(_) => history && self.history_section == 0,
            HistoryClear => history && self.history_section != 2 && !self.history_query.is_empty(),
            HistoryBaseline(_) | HistoryRun(_) => history && self.history_row_visible(control),
            SetupStage(stage) => {
                setup
                    && self.focus.setup_stage != regressee_core::shell::SetupStage::Settings
                    && stage != regressee_core::shell::SetupStage::Settings
            }
            PagesBaseline => project,
            CloseDialog => self.focus.dialog,
            DismissFeedback => self.error.is_some() || self.notice.is_some(),
            NewProject => screen == Screen::Projects,
            Name | Address | Save => setup,
            OpenProject(index) => screen == Screen::Projects && index < self.projects.items.len(),
            SelectProject(_) | CancelProjectChoice => false,
            ChooseProject => screen != Screen::Projects && project,
            PreviousPage => screen == Screen::Projects && self.projects.page > 0,
            NextPage => screen == Screen::Projects && self.projects.more,
            ChooseFolder | About | SetupRuntime => settings,
            CaptureBaseline => {
                results
                    && (self.focus.review_surface == ReviewSurface::Pages
                        || self.preview.is_some()
                        || (!self.focus.baseline && !self.focus.candidate))
            }
            RefreshEvidence => results,
            SaveBaseline => results && self.focus.candidate,
            CheckChanges => {
                results
                    && self.focus.baseline
                    && self.focus.review_surface != ReviewSurface::Pages
                    && self.preview.is_none()
            }
            CancelCapture => self.focus.busy && self.focus.capturing,
            PauseCapture => self.focus.busy && self.focus.capturing && self.focus.pausable,
            ZoomIn | ZoomOut | PanLeft | PanRight | PanUp | PanDown => false,
            Viewer => viewer && self.viewer_has_pixels(),
            Guided(1..=3 | 6 | 7 | 11) => setup,
            Guided(44) => analysis && self.preview.is_none(),
            Guided(45) => inventory,
            Guided(46) => viewer && self.viewer_has_pixels(),
            Guided(52) => false,
            Guided(47..=49) => false,
            Guided(50 | 51) => self.workspace_control_mounted(control),
            Guided(8..=10 | 12 | 25) => setup && self.focus.advanced,
            Guided(13) => setup && project,
            Guided(15) => {
                results
                    && self.preview.is_none()
                    && self.evidence.latest.as_ref().is_some_and(|run| {
                        matches!(run.run.state, JobState::Paused | JobState::Interrupted)
                    })
            }
            Guided(16) => {
                results
                    && (self.preview.is_some()
                        || (self.focus.review_surface == ReviewSurface::Pages
                            && self.evidence.baseline.is_some()))
            }
            Guided(17) => inventory && !self.search.is_empty(),
            Guided(23 | 24) => inventory,
            Guided(19) => {
                results
                    && self.preview.is_none()
                    && self.evidence.latest.is_some()
                    && self.result_filters_open
            }
            Guided(20..=22) => results && self.preview.is_some() && self.result_filters_open,
            Guided(27 | 31..=38) => setup && project,
            Guided(41) => settings,
            Guided(42) => settings && self.network_status.is_none() && self.pending.is_none(),
            Guided(_) => false,
            CrawlField(0..=2) => setup,
            CrawlField(3..=8) => setup && self.focus.advanced,
            CrawlField(9) => inventory,
            CrawlField(10..=16) => setup && project,
            CrawlField(17 | 18) => settings,
            CrawlField(_) => false,
            Phase5(0..=5 | 14..=19 | 26 | 34..=35) => analysis && self.review_tools,
            Phase5(20..=22 | 33) => analysis,
            Phase5(32) => analysis && self.review_note_open,
            Phase5(36) => viewer,
            Phase5(38) => viewer && self.capture_details_open,
            Phase5(39) => results && self.viewer_inspector_open(),
            Phase5(37) => analysis,
            Phase5(23) => analysis,
            Phase5(24) => analysis,
            Phase5(6..=7) => viewer && analysis,
            Phase5(8..=9) => viewer && analysis && (!self.narrow_workspace || self.review_tools),
            Phase5(10 | 11) => viewer && self.viewer_mode == 2,
            Phase5(12 | 13) => {
                analysis
                    && self.review_tools
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
                    && self.focus.approval_dialog
                    && self
                        .approval
                        .as_ref()
                        .is_some_and(|preview| self.approval_matches(preview))
            }
            Phase5(27 | 28) => analysis && self.analysis_advanced && self.review_tools,
            Phase5(30) => results && self.preview.is_some(),
            Phase5(40 | 41) => {
                history
                    && self.history_section == 1
                    && self.details.contains(&Detail::History)
                    && self
                        .history_baseline_indices()
                        .contains(&self.history_index)
            }
            Phase5(42) => {
                history
                    && self.history_section == 1
                    && self
                        .evidence
                        .baselines
                        .get(self.history_index)
                        .is_some_and(|baseline| self.history_active_baseline() != Some(baseline.id))
                    && self
                        .history_baseline_indices()
                        .contains(&self.history_index)
            }
            Phase5(_) => false,
            AnalysisField(0..=8) => analysis && self.analysis_advanced && self.review_tools,
            AnalysisField(9) => analysis && self.review_note_open,
            AnalysisField(10..=13) => analysis && self.review_tools,
            AnalysisField(_) => false,
            Phase6(0) => screen == Screen::Projects || (history && self.history_section == 1),
            Phase6(1) => portability && self.import_preview.is_some(),
            Phase6(2 | 4) => {
                history
                    && self.history_section == 1
                    && self
                        .history_baseline_indices()
                        .contains(&self.history_index)
            }
            Phase6(5 | 19) => {
                history
                    && self.history_section == 0
                    && self.history_run_indices().contains(&self.run_history_index)
            }
            Phase6(9 | 10) => {
                history
                    && self.history_section == 0
                    && self.details.contains(&Detail::History)
                    && self.history_run_indices().contains(&self.run_history_index)
            }
            Phase6(8) => history && self.history_section == 2,
            Phase6(11 | 12) => history && self.history_section == 0,
            Phase6(16..=18) => history,
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
                    results
                        && self.preview.is_none()
                        && self.evidence.latest.is_some()
                        && self.result_filters_open
                }
            }
            Environment(index) => {
                setup && project && index < self.draft_settings.environments.len()
            }
            Details(Detail::Capture) => setup,
            Details(Detail::History) => history && self.history_section != 2,
            Details(Detail::Analysis) => analysis && self.review_tools,
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
            17..=18 => {
                enabled && self.focus.screen == Screen::Settings && self.network_status.is_some()
            }
            _ => false,
        }
    }
    fn project_picker(&self, cx: &mut Context<Self>) -> AnyElement {
        let projects_screen = self.focus.screen == Screen::Projects;
        let (title, subtitle) = if projects_screen {
            (
                "Projects",
                "View results or edit settings. Results and History use the selected project.",
            )
        } else {
            (
                self.focus.screen.label(),
                "Choose a project below. You don't need to open its settings first.",
            )
        };
        let mut body = div().w_full().min_w_0().flex().flex_col().gap_5().child(
            div()
                .flex()
                .items_center()
                .flex_wrap()
                .justify_between()
                .gap_4()
                .child(self.heading(title, subtitle))
                .child(self.button(
                    Control::NewProject,
                    "Create project",
                    ActionStyle::Primary,
                    cx,
                )),
        );
        if self.projects.items.is_empty()
            && self.projects_dirty
            && self.pending.is_some()
            && self.error.is_none()
        {
            body = body.child(controls::loading(
                "Loading projects…",
                "Reading your saved projects from local storage.",
            ));
        } else if self.projects.items.is_empty() && (self.error.is_some() || self.projects_dirty) {
            body = body.child(controls::empty(
                "Projects could not load",
                "Reopen Projects to try reading saved projects again. Your saved projects have not been removed.",
            ));
        } else if self.projects.items.is_empty() && !self.focus.busy && self.projects.page == 0 {
            body = body.child(controls::empty("Your first project starts here", "Create a project to choose a website and screen sizes. Save a set of screenshots as your baseline, then compare later captures against it."));
        }
        if projects_screen {
            body = body
                .child(self.button(
                    Control::Phase6(0),
                    "Import baseline ZIP…",
                    ActionStyle::Secondary,
                    cx,
                ))
                .child(self.portability_confirmation(cx));
        }
        for (index, project) in self.projects.items.iter().enumerate() {
            let selected = self
                .project
                .as_ref()
                .is_some_and(|p| p.project.id == project.id);
            body = body.child(
                controls::list_row(("project-row", index))
                    .debug_selector(move || format!("project-row-{index}"))
                    .px_3()
                    .flex_wrap()
                    .bg(rgb(if selected { SELECTED } else { GROUND }))
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(180.))
                            .flex()
                            .flex_col()
                            .gap_1()
                            .child(
                                div()
                                    .flex()
                                    .items_center()
                                    .gap_2()
                                    .child(
                                        div()
                                            .min_w_0()
                                            .flex_1()
                                            .font_weight(FontWeight::SEMIBOLD)
                                            .text_ellipsis()
                                            .child(project.name.clone()),
                                    )
                                    .when(selected, |el| {
                                        el.child(controls::status("Selected", Tone::Information))
                                    }),
                            )
                            .child(
                                div()
                                    .text_size(px(14.))
                                    .text_color(rgb(if selected { ACCENT_HOVER } else { MUTED }))
                                    .text_ellipsis()
                                    .child(project.address.clone()),
                            ),
                    )
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .gap_2()
                            .child(self.button(
                                Control::SelectProject(index),
                                match self.focus.screen {
                                    Screen::History => "View history",
                                    Screen::Setup => "Edit settings",
                                    _ => "View results",
                                },
                                ActionStyle::Secondary,
                                cx,
                            ))
                            .when(projects_screen, |el| {
                                el.child(self.button(
                                    Control::OpenProject(index),
                                    "Edit settings",
                                    ActionStyle::Secondary,
                                    cx,
                                ))
                            }),
                    ),
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
                    el.child(self.button(Control::NextPage, "Next", ActionStyle::Secondary, cx))
                }),
        )
        .into_any_element()
    }
    fn content(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.focus.shows_project_picker() {
            return self.project_picker(cx);
        }
        match self.focus.screen {
            Screen::Projects => self.project_picker(cx),
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
                body=body.child(div().debug_selector(|| "managed-network-section".into()).border_t_1().border_color(rgb(LINE)).pt_5().max_w(px(680.)).flex().flex_col().gap_3()
                    .child(div().text_size(px(18.)).font_weight(FontWeight::SEMIBOLD).child("Managed network"))
                    .when(!self.form_errors(cx).is_empty(), |body| body.child(self.form_error_summary(cx)))
                    .child(self.crawl_field(17,"Proxy server (optional)","Complete http(s) origin without credentials. Blank uses existing OS networking."))
                    .child(self.crawl_field(18,"Trusted CA PEM file (optional)","PEM file for downloads and Node. Chromium also needs OS/browser trust; TLS stays verified."))
                    .child(div().flex().flex_wrap().gap_2().when(self.network_status.is_none() && self.pending.is_none(), |el| el.child(self.guided_button(42,"Retry loading saved settings",cx))).child(self.guided_button(41,"Save network settings",cx)))
                    .child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(if self.network_status.is_some() { "Saved network settings loaded. Saving changes is explicit." } else { "Reading saved network settings…" }))
                    .child(self.button(Control::Details(Detail::Network), if self.details.contains(&Detail::Network) { "Hide network limitations" } else { "Network limitations" }, ActionStyle::Secondary, cx))
                    .when(self.details.contains(&Detail::Network), |el| el
                        .when_some(self.network_status.clone(), |el, status| el.child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(status)))
                        .child(div().text_size(px(14.)).text_color(rgb(WARNING)).child("The development browser's Node 22 worker cannot fetch robots.txt through a configured proxy. An authorized robots override is explicit. VPN and enterprise networks remain unqualified. Chromium needs custom CA trust in its OS/browser store; TLS verification is never disabled."))));
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
    }
    fn selected_capture_details(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div().flex().flex_col().gap_3().child(self.phase5_panel(cx));
        if self.viewer.is_some() {
            body = body.child(div().flex().flex_wrap().gap_2().when(
                self.narrow_workspace && self.evidence.comparison.is_some(),
                |tools| {
                    tools
                        .child(self.button(
                            Control::Phase5(8),
                            "Overlay",
                            ActionStyle::Secondary,
                            cx,
                        ))
                        .child(self.button(
                            Control::Phase5(9),
                            "Heatmap",
                            ActionStyle::Secondary,
                            cx,
                        ))
                },
            ));
        }
        if let Some(run) = &self.evidence.latest {
            body = body.child(
                div()
                    .text_size(px(14.))
                    .child(run.run.inventory.coverage.label()),
            );
        }
        if let Some(r) = &self.evidence.latest {
            let Some(target) = r
                .run
                .targets
                .iter()
                .find(|t| Some(&t.key) == self.selected_key.as_ref())
                .or_else(|| r.run.targets.first())
            else {
                return body.into_any_element();
            };
            let (mut label, mut tone) = match r.run.state {
                JobState::Completed => {
                    if let Some(a) = &self.evidence.comparison {
                        match a
                            .revision
                            .results
                            .iter()
                            .find(|result| result.key == target.key)
                            .map(|result| result.classification)
                        {
                            Some(Classification::Changed) => {
                                ("Changed — review the evidence", Tone::Warning)
                            }
                            Some(Classification::NoSignificantChange)
                                if matches!(target.outcome, TargetOutcome::Captured { .. }) =>
                            {
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
                        ("Capture complete — choose Save baseline", Tone::Neutral)
                    } else if self.capture_is_saved_baseline() {
                        ("Saved baseline capture", Tone::Neutral)
                    } else if r.run.intent == RunIntent::RepresentativePreview {
                        ("Preview only — not a baseline", Tone::Neutral)
                    } else {
                        ("Capture kept — not a saved baseline", Tone::Neutral)
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
                    JobState::Completed if self.focus.candidate => {
                        ("Capture complete — choose Save baseline", Tone::Neutral)
                    }
                    JobState::Completed if self.capture_is_saved_baseline() => {
                        ("Saved baseline capture", Tone::Neutral)
                    }
                    JobState::Completed => ("Capture kept — not a saved baseline", Tone::Neutral),
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
            if let Some(a) = &self.evidence.comparison
                && let Some(r) = a.revision.results.iter().find(|r| r.key == target.key)
            {
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
        body.child(div().text_size(px(13.)).text_color(rgb(MUTED)).max_w(px(680.)).child("Sensitivity and ignored rectangles change only analysis, never captured originals. Numerical change is evidence to review, not a defect probability. Fixture-calibrated presets are not proven user accuracy.")).into_any_element()
    }
    fn viewer_section(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(view) = &self.viewer else {
            return div()
                .flex_1()
                .min_h_0()
                .flex()
                .flex_col()
                .gap_2()
                .relative()
                .when(
                    self.evidence.comparison.is_some() && self.preview.is_none(),
                    |viewer| viewer.child(self.guided_button(44, "Analysis", cx)),
                )
                .when(self.viewer_inspector_open(), |viewer| {
                    viewer.child(self.viewer_inspector(cx))
                })
                .child(self.screenshot_placeholder())
                .into_any_element();
        };
        let mut section = div()
            .flex_1()
            .min_h_0()
            .relative()
            .flex()
            .flex_col()
            .gap_2()
            .when(self.narrow_workspace, |section| section.gap_1());
        let mut view_tools = div()
            .debug_selector(|| "viewer-toolbar".into())
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .gap_2()
            .when(self.narrow_workspace, |tools| tools.gap_1());
        if self.evidence.comparison.is_some() {
            view_tools = view_tools.children(
                [
                    (6, "Compare"),
                    (
                        7,
                        if self.narrow_workspace {
                            "Diff"
                        } else {
                            "Difference"
                        },
                    ),
                    (8, "Overlay"),
                    (9, "Heatmap"),
                ]
                .into_iter()
                .filter(|(index, _)| *index <= 7 || !self.narrow_workspace)
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
            );
        }
        view_tools = view_tools
            .when(self.viewer_has_pixels(), |tools| {
                tools.child(self.zoom_select.clone())
            })
            .child(self.button(
                Control::Phase5(36),
                "Details",
                ActionStyle::Choice {
                    selected: self.capture_details_open,
                },
                cx,
            ));
        if self.evidence.comparison.is_some() && self.preview.is_none() {
            view_tools = view_tools.child(self.button(
                Control::Guided(44),
                "Analysis",
                ActionStyle::Choice {
                    selected: self.review_tools,
                },
                cx,
            ));
        }
        if self.mask_draw {
            view_tools = view_tools.child(self.button(
                Control::Phase5(14),
                "Done drawing",
                ActionStyle::Secondary,
                cx,
            ));
        }
        section = section.child(view_tools);
        section = section.child(self.viewer_panes_layout(view, true, cx));
        if self.viewer_has_pixels() && !self.narrow_workspace {
            section = section.child(div().flex_shrink_0().text_size(px(13.)).line_height(px(18.)).text_color(rgb(MUTED))
                .child(if self.mask_draw { "Drag to mark an ignored area · Esc to finish" }
                else if self.narrow_workspace { "Drag / scroll to move · Ctrl + scroll to zoom" }
                 else { "Drag or scroll to move · Ctrl + scroll to zoom · Zoom menu for fit width or actual size" }));
        }
        if self.evidence.comparison.is_some() && self.viewer_mode == 2 {
            section = section.child(
                div()
                    .flex_shrink_0()
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
        section
            .when(self.viewer_inspector_open(), |section| {
                section.child(self.viewer_inspector(cx))
            })
            .into_any_element()
    }
    fn phase5_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut panel = div()
            .flex()
            .flex_col()
            .gap_3()
            .border_t_1()
            .border_color(rgb(LINE))
            .pt_4()
            .pt_0()
            .border_t_0()
            .gap_2();
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
        if self.preview.is_some() {
            return panel.into_any_element();
        }
        panel = panel.child(
            div()
                .font_weight(FontWeight::SEMIBOLD)
                .child("Analysis sensitivity & ignored areas"),
        );
        if self.review_tools {
            panel = panel.child(self.keyboard_mask_editor(cx));
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
                panel = panel.child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(
                    format!(
                        "Analysis {} · {}",
                        a.revision.id,
                        a.perceptual
                            .as_ref()
                            .map(|p| p.policy.revision.as_str())
                            .unwrap_or("historical RGB policy")
                    ),
                ));
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
                    panel = panel.child(
                        controls::field(
                            label,
                            help,
                            self.analysis_fields[i].clone().into_any_element(),
                        )
                        .id(("analysis-parameter-field", i))
                        .anchor_scroll(self.review_anchor(Control::AnalysisField(i))),
                    );
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
                        panel =
                            panel.child(div().text_size(px(14.)).text_color(rgb(WARNING)).child(
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
                    panel=panel.child(div().flex().flex_wrap().gap_2().child(self.button(Control::Phase5(12),"Previous change region",ActionStyle::Secondary,cx)).child(self.button(Control::Phase5(13),"Next change region",ActionStyle::Secondary,cx))).child(div().text_size(px(13.)).text_color(rgb(MUTED)).child(format!("Navigation retains the largest {} regions; {} further regions are counted but not retained.",r.regions.retained.len(),r.regions.omitted)));
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
                (15, "Select next rectangle"),
                (17, "Redraw selected rectangle"),
                (16, "Delete selected rectangle"),
                (19, "Revalidate for this pair"),
            ] {
                masks = masks.child(self.button(
                    Control::Phase5(i),
                    label,
                    if i == 16 {
                        ActionStyle::Danger
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
        }
        for d in self
            .evidence
            .reviews
            .iter()
            .filter(|d| Some(&d.key) == self.selected_key.as_ref())
            .filter(|d| {
                self.evidence
                    .comparison
                    .as_ref()
                    .is_none_or(|a| d.analysis != a.revision.id)
            })
            .take(if self.review_tools { 8 } else { 0 })
        {
            panel = panel.child(div().text_size(px(13.)).child(format!(
                "Historical review {:?} · analysis {} · {} · {}",
                d.state, d.analysis, d.reviewed_at, d.note
            )));
        }
        panel.into_any_element()
    }
    #[cfg(all(test, feature = "ui-tests"))]
    fn viewer_panes(
        &self,
        view: &regressee_app::jobs::ViewerFrame,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        self.viewer_panes_layout(view, false, cx)
    }
    fn viewer_panes_layout(
        &self,
        view: &regressee_app::jobs::ViewerFrame,
        bounded: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let indices = self.viewer_pane_indices();
        let mode = if self.evidence.comparison.is_some() {
            self.viewer_mode
        } else {
            0
        };
        let mut panes = div()
            .id("image-viewer")
            .when(
                bounded && self.control_enabled(Control::Viewer, cx),
                |panes| {
                    panes
                        .track_focus(&self.handle(Control::Viewer, cx))
                        .key_context("ImageViewer")
                        .on_key_down(cx.listener(|this, event: &gpui::KeyDownEvent, _, cx| {
                            this.viewer_key(event, cx)
                        }))
                        .focus(|mut style| {
                            style.box_shadow = Some(vec![gpui::BoxShadow {
                                color: rgb(ACCENT).into(),
                                offset: gpui::point(px(0.), px(0.)),
                                blur_radius: px(0.),
                                spread_radius: px(2.),
                            }]);
                            style
                        })
                },
            )
            .flex()
            .gap_3()
            .when(bounded, |panes| panes.flex_1().min_h_0())
            .when(!bounded, |panes| panes.flex_wrap());
        let transform = self.viewer_transform();
        for &index in indices {
            let origin = std::rc::Rc::new(std::cell::Cell::new((0., 0.)));
            let scroll_origin = origin.clone();
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
                .when(bounded, |pane| pane.flex_1().min_h_0())
                .when(!bounded, |pane| pane.h(px(280.)))
                .overflow_hidden()
                .border_1()
                .border_color(rgb(LINE))
                .bg(rgb(SURFACE))
                .when(bounded, |pane| {
                    pane.on_scroll_wheel(cx.listener(
                        move |this, event: &gpui::ScrollWheelEvent, _, cx| {
                            this.viewer_scroll(event, scroll_origin.get(), cx)
                        },
                    ))
                })
                .child(tile_layer(if mode == 1 { 1 } else { index }, 1.));
            if mode == 1 {
                viewport = viewport.child(tile_layer(2, 0.55));
            }
            if mode == 2 {
                viewport = viewport.child(tile_layer(1, self.overlay_fraction));
            }
            if view.panes[index].is_empty() {
                let message = if index == 1 {
                    self.evidence
                        .latest
                        .as_ref()
                        .and_then(|run| {
                            run.run
                                .targets
                                .iter()
                                .find(|target| Some(&target.key) == self.selected_key.as_ref())
                        })
                        .and_then(|target| match target.outcome {
                            TargetOutcome::Failed { code } => {
                                Some(format!("No screenshot captured: {}.", code.label()))
                            }
                            _ => None,
                        })
                        .unwrap_or_else(|| "No captured screenshot for this target".into())
                } else if index == 0 {
                    "No baseline screenshot for this target".into()
                } else {
                    "No difference image for this target".into()
                };
                viewport = viewport.child(div().p_4().child(message));
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
            let o = origin.clone();
            let down = origin.clone();
            let up = origin.clone();
            let mv = origin.clone();
            let viewport_bounds = self.viewer_bounds.clone();
            viewport = viewport
                .when(bounded, |viewport| {
                    viewport.cursor(if !self.viewer_has_pixels() {
                        gpui::CursorStyle::Arrow
                    } else if self.mask_draw {
                        gpui::CursorStyle::Crosshair
                    } else if self.pan_drag.is_some() {
                        gpui::CursorStyle::ClosedHand
                    } else {
                        gpui::CursorStyle::OpenHand
                    })
                })
                .child(
                    canvas(
                        move |bounds, _, _| {
                            o.set((f32::from(bounds.origin.x), f32::from(bounds.origin.y)));
                            if bounded {
                                viewport_bounds.set(Some(bounds));
                            }
                        },
                        |_, _, _, _| {},
                    )
                    .absolute()
                    .size_full(),
                )
                .on_mouse_down(
                    MouseButton::Left,
                    cx.listener(move |this, event: &MouseDownEvent, window, cx| {
                        if bounded && this.control_enabled(Control::Viewer, cx) {
                            window.focus(&this.handle(Control::Viewer, cx));
                        }
                        this.viewer_drag_pane = Some(index);
                        this.viewer_pointer(
                            0,
                            (f32::from(event.position.x), f32::from(event.position.y)),
                            down.get(),
                            cx,
                        );
                    }),
                )
                .on_mouse_move(cx.listener(move |this, event: &MouseMoveEvent, _, cx| {
                    if event.pressed_button == Some(MouseButton::Left)
                        && this.viewer_drag_pane == Some(index)
                    {
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
                        if this.viewer_drag_pane != Some(index) {
                            return;
                        }
                        this.viewer_drag_pane = None;
                        this.viewer_pointer(
                            2,
                            (f32::from(event.position.x), f32::from(event.position.y)),
                            up.get(),
                            cx,
                        );
                    }),
                )
                .when(bounded, |viewport| {
                    viewport.on_mouse_up_out(
                        MouseButton::Left,
                        cx.listener(move |this, _, _, cx| {
                            if this.viewer_drag_pane != Some(index) {
                                return;
                            }
                            this.viewer_drag_pane = None;
                            this.pan_drag = None;
                            this.mask_drag = None;
                            cx.notify();
                        }),
                    )
                });
            panes = panes.child(
                div()
                    .flex_1()
                    .min_w(px(if bounded { 0. } else { 220. }))
                    .min_h_0()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .when(bounded && self.narrow_workspace, |pane| pane.gap_1())
                    .child(if bounded {
                        let (mut title, origin, profile) = self.pane_caption(index);
                        if mode == 2 {
                            title = "Overlay · before + after".into();
                        }
                        div()
                            .debug_selector(move || format!("evidence-caption-{index}"))
                            .h(px(if indices.len() == 1 {
                                40.
                            } else if self.narrow_workspace {
                                36.
                            } else {
                                60.
                            }))
                            .flex_shrink_0()
                            .overflow_hidden()
                            .text_size(px(if self.narrow_workspace { 13. } else { 14. }))
                            .line_height(px(if self.narrow_workspace { 18. } else { 20. }))
                            .child(if indices.len() == 1 {
                                div()
                                    .flex()
                                    .items_center()
                                    .justify_between()
                                    .gap_2()
                                    .child(div().font_weight(FontWeight::SEMIBOLD).child(title))
                                    .child(
                                        div()
                                            .text_color(rgb(MUTED))
                                            .text_ellipsis()
                                            .child(profile.clone()),
                                    )
                                    .into_any_element()
                            } else {
                                div()
                                    .text_ellipsis()
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(title)
                                    .into_any_element()
                            })
                            .when(indices.len() == 1 || !self.narrow_workspace, |caption| {
                                caption.child(
                                    div()
                                        .text_ellipsis()
                                        .text_color(rgb(MUTED))
                                        .child(origin.replace('\n', " · ")),
                                )
                            })
                            .when(indices.len() > 1, |caption| {
                                caption.child(div().text_ellipsis().child(
                                    if self.narrow_workspace {
                                        profile
                                            .rsplit_once(" · ")
                                            .map(|(name, dimensions)| {
                                                format!("{dimensions} · {name}")
                                            })
                                            .unwrap_or(profile)
                                    } else {
                                        profile
                                    },
                                ))
                            })
                            .into_any_element()
                    } else {
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
                            })
                            .into_any_element()
                    })
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
        if self.focus.busy
            || self.focus.dialog
            || self.focus.approval_dialog
            || self.native_prompt
            || !self.viewer_has_pixels()
        {
            self.mask_drag = None;
            self.pan_drag = None;
            return;
        }
        if self.mask_draw && !self.viewer_current() {
            self.mask_drag = None;
            return;
        }
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
                        self.mask_draw = false;
                        self.load_mask_fields(cx);
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
                        if self.viewer_bounds.get().is_some() {
                            self.refresh_viewer_navigation(cx);
                        }
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
        if self.mask_dirty || self.mask_geometry_dirty(cx) || self.review_unsaved(cx) {
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
                    this.discard_review_drafts(cx);
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
        self.focus.history_baselines = self.evidence.baselines.len();
        self.focus.history_runs = self.history.as_ref().map_or(0, |h| h.runs.len());
        let render_started = std::time::Instant::now();
        self.configure_review_geometry(f32::from(window.viewport_size().width));
        let bounded_review =
            self.focus.screen == Screen::Results && !self.focus.shows_project_picker();
        if self.focus.approval_dialog
            && self
                .approval
                .as_ref()
                .is_none_or(|p| !self.approval_matches(p))
        {
            self.invalidate_approval();
        }
        let navigation = self.navigation_started.take();
        // RenderImage uses GPUI's sprite atlas: dropping only an Arc does not
        // remove the GPU entry. Evict retired tile IDs before painting new ones.
        // Hold retired atlas entries until the replacement read is accepted;
        // cache-hit Arcs can then be retained without evicting/re-uploading them.
        if self.viewer_pending.is_none() {
            for image in self.retired_images.drain(..) {
                let _ = window.drop_image(image);
            }
        }
        let enabled = !self.focus.busy && !self.focus.dialog && !self.native_prompt;
        self.name.update(cx, |input, _| {
            input.set_enabled(enabled && self.control_mounted(Control::Name))
        });
        self.address.update(cx, |input, _| {
            input.set_enabled(enabled && self.control_mounted(Control::Address))
        });
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
        self.configure_analysis_inputs(cx);
        let zoom_enabled = self.control_enabled(Control::Guided(46), cx);
        self.zoom_select.update(cx, |select, cx| {
            select.sync(
                self.viewer_fit,
                self.zoom,
                zoom_enabled,
                self.narrow_workspace,
                cx,
            );
        });
        for (i, input) in self.analysis_fields.iter().enumerate() {
            let enabled = self.control_enabled(Control::AnalysisField(i), cx);
            input.update(cx, |input, _| input.set_enabled(enabled));
        }
        let history_enabled = self.control_enabled(Control::HistorySearch, cx);
        self.history_search
            .update(cx, |input, _| input.set_enabled(history_enabled));
        if let Some(control) = self.presentation_focus
            && self.control_enabled(control, cx)
        {
            window.focus(&self.handle(control, cx));
            self.reveal_form_control(control, true, window, cx);
            self.reveal_review_control(control, window, cx);
            self.presentation_focus = None;
        }
        let mut feedback = div()
            .id("feedback")
            .when(bounded_review, |feedback| {
                feedback.max_h(px(96.)).overflow_y_scroll()
            })
            .px_6()
            .py_2()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_3();
        if self.focus.busy && self.task.is_none() {
            let (message, completed, total) = self
                .progress
                .clone()
                .unwrap_or_else(|| ("Loading project data…".into(), 0, None));
            let mut progress = div().flex().flex_wrap().items_center().gap_3().child(
                div().flex_1().min_w(px(160.)).child(
                    if self.focus.capturing || total.is_some_and(|n| n > 0) {
                        controls::progress(message, completed, total).into_any_element()
                    } else {
                        controls::activity(message).into_any_element()
                    },
                ),
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
            .min_h_0()
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
                    .px_6()
                    .py_2()
                    .bg(rgb(SURFACE))
                    .border_b_1()
                    .border_color(rgb(LINE))
                    .child(
                        div()
                            .min_w_0()
                            .min_w(px(240.))
                            .flex_1()
                            .debug_selector(|| "selected-project-context".into())
                            .font_weight(FontWeight::MEDIUM)
                            .text_ellipsis()
                            .child(
                                self.project
                                    .as_ref()
                                    .map(|project| {
                                        format!("Selected project: {}", project.project.name)
                                    })
                                    .unwrap_or_else(|| "No project selected".into()),
                            ),
                    )
                    .when(
                        self.project.is_some() && self.focus.screen != Screen::Projects,
                        |el| {
                            el.child(if self.focus.choosing_project {
                                self.button(
                                    Control::CancelProjectChoice,
                                    "Keep current project",
                                    ActionStyle::Secondary,
                                    cx,
                                )
                            } else {
                                self.button(
                                    Control::ChooseProject,
                                    "Change project",
                                    ActionStyle::Secondary,
                                    cx,
                                )
                            })
                        },
                    )
                    .when(!bounded_review, |header| {
                        header.child(controls::status(
                            if self.focus.screen == Screen::Setup && self.unsaved(cx) {
                                "Unsaved settings"
                            } else {
                                "Local project workspace"
                            },
                            Tone::Neutral,
                        ))
                    }),
            )
            .when(
                self.task.is_none()
                    && (self.focus.busy || self.error.is_some() || self.notice.is_some()),
                |el| el.child(feedback),
            )
            .child(
                div()
                    .id("screen-scroll")
                    .debug_selector(|| "screen-scroll".into())
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .when(!bounded_review, |body| {
                        body.overflow_y_scroll().track_scroll(&self.screen_scroll)
                    })
                    .when(bounded_review, |body| {
                        body.overflow_hidden().flex().flex_col()
                    })
                    .px_6()
                    .py(px(if bounded_review { 8. } else { 16. }))
                    .child(self.content(cx)),
            );
        if self.focus.screen == Screen::Setup && !self.focus.shows_project_picker() {
            content = content.child(self.setup_footer(cx));
        }
        if self.focus.screen == Screen::Results
            && !self.focus.shows_project_picker()
            && !self.page_list_workspace()
            && self.history_inspection.is_none()
            && self.evidence.comparison.is_some()
        {
            content = content.child(self.approval_bar(cx));
        }
        let shell = div()
            .id("regressee-shell")
            .key_context("Shell")
            .track_focus(&self.root_focus)
            .relative()
            .size_full()
            .flex()
            .bg(rgb(GROUND))
            .text_color(rgb(INK))
            .text_size(px(15.))
            .font_family(FONT)
            .on_action(cx.listener(|this, _: &Tab, window, cx| this.move_focus(false, window, cx)))
            .on_action(
                cx.listener(|this, _: &TabPrev, window, cx| this.move_focus(true, window, cx)),
            )
            .on_action(cx.listener(Self::activate))
            .on_action(cx.listener(Self::create))
            .on_action(cx.listener(Self::save))
            .on_action(cx.listener(Self::dismiss))
            .on_action(cx.listener(Self::find_local))
            .on_action(cx.listener(|this, _: &KeyboardHelp, w, cx| this.show_keyboard_help(w, cx)))
            .on_action(
                cx.listener(|this, _: &ProjectsScreen, w, cx| {
                    this.navigate(Screen::Projects, w, cx)
                }),
            )
            .on_action(
                cx.listener(|this, _: &SetupScreen, w, cx| this.navigate(Screen::Setup, w, cx)),
            )
            .on_action(
                cx.listener(|this, _: &ResultsScreen, w, cx| this.navigate(Screen::Results, w, cx)),
            )
            .on_action(
                cx.listener(|this, _: &HistoryScreen, w, cx| this.navigate(Screen::History, w, cx)),
            )
            .on_action(
                cx.listener(|this, _: &SettingsScreen, w, cx| {
                    this.navigate(Screen::Settings, w, cx)
                }),
            )
            .when(self.task_owns_workspace(), |shell| {
                shell.child(self.task_workspace(cx))
            })
            .when(!self.task_owns_workspace(), |shell| {
                shell.child(self.project_rail(cx)).child(content)
            })
            .when(self.task.is_none() && self.focus.approval_dialog, |shell| {
                shell.child(self.approval_decision(cx))
            })
            .when(self.task.is_none() && self.focus.dialog, |el| {
                el.child(self.information_dialog(cx))
            })
            .when(
                self.task.is_some() && !self.task_owns_workspace(),
                |shell| shell.child(self.task_dialog(cx)),
            );
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
        cx.set_global(controls::ReducedMotion(
            std::env::var_os("REGRESSEE_REDUCED_MOTION").is_some_and(|value| value == "1"),
        ));
        text_input::bind(cx);
        timezone_select::bind(cx);
        zoom_select::bind(cx);
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
            KeyBinding::new("ctrl-f", FindLocal, Some("Shell")),
            KeyBinding::new("cmd-f", FindLocal, Some("Shell")),
            KeyBinding::new("f1", KeyboardHelp, Some("Shell")),
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
        let termination_stop = jobs.clone();
        let main_window = cx
            .open_window(
                WindowOptions {
                    window_bounds: Some(WindowBounds::Windowed(Bounds::centered(
                        None,
                        size(px(1280.), px(800.)),
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
                    let close_shell = shell.downgrade();
                    window.on_window_should_close(cx, move |window, cx| {
                        // Always veto the platform destruction first. Confirmed exit
                        // uses the existing application shutdown path instead.
                        let _ = close_shell.update(cx, |shell, cx| shell.request_exit(window, cx));
                        false
                    });
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
        cx.on_action(move |_: &Quit, cx| {
            let _ = main_window.update(cx, |shell, window, cx| {
                shell.request_exit(window, cx);
            });
        });
        // GPUI's on_app_quit cannot veto termination: never prompt here.
        // This retains cleanup for platform termination outside our Quit action.
        let stop = termination_stop;
        cx.on_app_quit(move |_| {
            stop.stop();
            async {}
        })
        .detach();
        cx.activate(true);
    });
    after_ui.join_after_ui();
    Ok(())
}

#[cfg(all(test, feature = "ui-tests"))]
#[path = "../tests/support/approval_fixture.rs"]
mod approval_fixture;

#[cfg(all(test, feature = "ui-tests"))]
mod workflow_refinement_tests;

#[cfg(all(test, feature = "ui-tests"))]
mod options_ux_tests;

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    use gpui::TestAppContext;
    #[gpui::test]
    fn guided_first_save_advances_locally_and_hides_irrelevant_controls(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, finish_request);
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.create(&CreateProject, window, cx);
                assert_eq!(s.focus.setup_stage, SetupStage::Website);
                assert!(s.control_mounted(Control::Name));
                assert!(!s.control_mounted(Control::CrawlField(0)));
                s.name
                    .update(cx, |input, cx| input.set_value("Guided website", cx));
                s.address
                    .update(cx, |input, cx| input.set_value("https://example.test", cx));
                s.activate_control(Control::Save, window, cx);
            })
        });
        shell.update(cx, |s, cx| {
            settle_requests(s, cx);
            assert!(s.error.is_none(), "{:?}", s.error);
            assert_eq!(s.focus.setup_stage, SetupStage::Access);
            assert!(s.project.is_some());
            assert!(s.evidence.latest.is_none());
            assert!(s.preview.is_none());
            assert!(!s.control_mounted(Control::Name));
            assert!(!s.control_mounted(Control::CrawlField(10)));
            assert!(!s.control_mounted(Control::Guided(13)));
            s.choose(Choice::LoginRequired, cx);
            assert!(s.control_mounted(Control::CrawlField(10)));
            assert!(
                !s.control_enabled(Control::Guided(31), cx),
                "Unsaved access cannot open a browser"
            );
        });
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                let requests = s.next_request;
                s.activate_control(Control::Guided(31), window, cx);
                assert_eq!(
                    s.next_request, requests,
                    "Dirty access must submit no OpenLogin request"
                );
                assert!(s.pending.is_none());
                s.choose(Choice::PublicAccess, cx);
                assert!(!s.control_mounted(Control::CrawlField(10)));
                s.activate_control(Control::SetupStage(SetupStage::Pages), window, cx);
                assert_eq!(s.focus.setup_stage, SetupStage::Pages);
                assert!(s.control_mounted(Control::CrawlField(0)));
                assert!(!s.control_mounted(Control::Address));
                assert!(!s.control_mounted(Control::Guided(31)));
                assert!(s.evidence.latest.is_none());
            })
        });
        owned.join_after_ui();
    }

    #[gpui::test]
    fn returning_current_project_enters_workspace_without_reloading_dirty_evidence(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Results;
            s.queue(Work::Load(project), cx);
            settle_requests(s, cx);
            s.name
                .update(cx, |input, cx| input.set_value("Retained draft", cx));
            s.mask_dirty = true;
        });
        cx.update(|window, cx| shell.update(cx, |s, cx| s.navigate(Screen::Projects, window, cx)));
        shell.update(cx, |s, cx| {
            settle_requests(s, cx);
        });
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                let index = s
                    .projects
                    .items
                    .iter()
                    .position(|p| p.id == project)
                    .unwrap();
                let request = s.next_request;
                let key = s.selected_key.clone();
                s.activate_control(Control::SelectProject(index), window, cx);
                assert_eq!(s.focus.screen, Screen::Results);
                assert_eq!(s.focus.review_surface, ReviewSurface::Changes);
                assert_eq!(
                    s.next_request, request,
                    "Returning to the same project must not reload evidence"
                );
                assert_eq!(s.selected_key, key);
                assert!(s.mask_dirty);
                assert_eq!(s.name.read(cx).content.as_ref(), "Retained draft");
            })
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn results_project_selection_loads_evidence_without_editing_settings(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (store, project_id, run_id, _, _) = crate::approval_fixture::fixture(dir.path());
        let before = serde_json::to_value(store.load_project(project_id).unwrap()).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_request(s, cx);
                s.navigate(Screen::Results, window, cx);
                assert!(s.project.is_none());
                assert!(s.focus.shows_project_picker());
                assert!(s.control_enabled(Control::SelectProject(0), cx));
                assert!(!s.control_enabled(Control::OpenProject(0), cx));
                window.focus(&s.handle(Control::SelectProject(0), cx));
            })
        });
        // The picker is usable through the same keyboard action as other controls.
        cx.dispatch_action(Activate);
        shell.update(cx, |s, cx| {
            settle_requests(s, cx);
            assert_eq!(s.focus.screen, Screen::Results);
            assert!(!s.focus.shows_project_picker());
            assert_eq!(s.project.as_ref().unwrap().project.id, project_id);
            assert_eq!(s.evidence.latest.as_ref().unwrap().run.id, run_id);
            assert!(s.viewer.is_some());
            assert!(s.notice.is_none(), "Opening results is not a settings save");
            assert!(s.control_enabled(Control::ChooseProject, cx));
            assert!(!s.control_enabled(Control::SelectProject(0), cx));
            assert!(!s.unsaved(cx));
        });
        owned.join_after_ui();
        let store = regressee_storage::Store::open(paths).unwrap();
        assert_eq!(
            before,
            serde_json::to_value(store.load_project(project_id).unwrap()).unwrap()
        );
    }
    #[gpui::test]
    fn changing_projects_preserves_edits_until_confirmed_and_keeps_the_destination(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (mut store, first, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let second = Project::draft("Another website".into(), "https://example.test/").unwrap();
        store.save_project(&second, None).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths);
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_request(s, cx);
                let index = s.projects.items.iter().position(|p| p.id == first).unwrap();
                s.activate_control(Control::SelectProject(index), window, cx);
                settle_requests(s, cx);
                s.navigate(Screen::Setup, window, cx);
                s.name
                    .update(cx, |input, cx| input.set_value("Unsaved edit", cx));
                s.activate_control(Control::ChooseProject, window, cx);
                settle_requests(s, cx);
                assert!(s.focus.shows_project_picker());
                assert!(!s.control_enabled(Control::Save, cx));
                assert!(!s.control_enabled(Control::Name, cx));
                s.activate_control(Control::CancelProjectChoice, window, cx);
                assert_eq!(s.focus.screen, Screen::Setup);
                assert_eq!(s.name.read(cx).content, "Unsaved edit");
                // Reopening this same project must not reload it or discard its draft.
                s.navigate(Screen::Projects, window, cx);
                settle_requests(s, cx);
                let index = s.projects.items.iter().position(|p| p.id == first).unwrap();
                s.activate_control(Control::SelectProject(index), window, cx);
                assert_eq!(s.focus.screen, Screen::Results);
                assert_eq!(s.name.read(cx).content, "Unsaved edit");
                assert!(!s.native_prompt);
                s.activate_control(Control::ChooseProject, window, cx);
                settle_requests(s, cx);
                let index = s
                    .projects
                    .items
                    .iter()
                    .position(|p| p.id == second.id)
                    .unwrap();
                s.activate_control(Control::SelectProject(index), window, cx);
                assert!(s.native_prompt);
                assert_eq!(s.project.as_ref().unwrap().project.id, first);
            })
        });
        cx.simulate_prompt_answer("Keep editing");
        cx.run_until_parked();
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert!(!s.native_prompt);
                assert_eq!(s.name.read(cx).content, "Unsaved edit");
                let index = s
                    .projects
                    .items
                    .iter()
                    .position(|p| p.id == second.id)
                    .unwrap();
                s.activate_control(Control::SelectProject(index), window, cx);
            })
        });
        cx.simulate_prompt_answer("Discard edits");
        cx.run_until_parked();
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                settle_requests(s, cx);
                assert_eq!(s.focus.screen, Screen::Results);
                assert_eq!(s.project.as_ref().unwrap().project.id, second.id);
                assert!(s.evidence.latest.is_none());
                assert!(s.viewer.is_none());
                assert!(!s.focus.baseline);
                assert!(!s.focus.preview);
                assert!(s.selected_key.is_none());
                s.navigate(Screen::History, window, cx);
                settle_requests(s, cx);
                s.activate_control(Control::ChooseProject, window, cx);
                settle_requests(s, cx);
                let index = s.projects.items.iter().position(|p| p.id == first).unwrap();
                s.activate_control(Control::SelectProject(index), window, cx);
                settle_requests(s, cx);
                assert_eq!(s.focus.screen, Screen::History);
                assert_eq!(s.project.as_ref().unwrap().project.id, first);
                assert!(s.history.is_some());
                assert!(s.focus.baseline);
            })
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn project_selection_actions_and_long_names_fit_supported_window_sizes(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let mut store = regressee_storage::Store::open(paths.clone()).unwrap();
        let project = Project::draft(
            "A long project name for a website with several environments ".repeat(2),
            "https://example.test/",
        )
        .unwrap();
        store.save_project(&project, None).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths);
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, finish_request);
        for (width, height) in [(1120., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            for screen in [Screen::Projects, Screen::Results, Screen::History] {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        s.navigate(screen, window, cx);
                        settle_requests(s, cx);
                    })
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let row = cx.debug_bounds("project-row-0").unwrap();
                let working_area = cx.debug_bounds("screen-scroll").unwrap();
                assert_eq!(working_area.left(), px(192.));
                assert_eq!(row.left(), working_area.left() + px(24.));
                assert_eq!(row.right(), working_area.right() - px(24.));
                assert_eq!(row.size.width, working_area.size.width - px(48.));
                let select = cx.debug_bounds("action-SelectProject(0)").unwrap();
                assert!(select.left() >= row.left() && select.right() <= row.right());
                assert!(select.top() >= row.top() && select.bottom() <= row.bottom());
                assert!(select.size.height >= px(40.));
                if screen == Screen::Projects {
                    let edit = cx.debug_bounds("action-OpenProject(0)").unwrap();
                    assert!(edit.left() >= select.right() + px(8.));
                    assert!(edit.right() <= row.right());
                }
            }
        }
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::SelectProject(0), window, cx);
                settle_requests(s, cx);
            })
        });
        for (width, height) in [(1120., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let context = cx.debug_bounds("selected-project-context").unwrap();
            let change = cx.debug_bounds("action-ChooseProject").unwrap();
            let working_area = cx.debug_bounds("screen-scroll").unwrap();
            assert!(context.size.width >= px(140.));
            assert!(context.left() >= working_area.left() + px(24.));
            assert!(context.right() + px(12.) <= change.left());
            assert!(change.right() <= working_area.right() - px(24.));
        }
        owned.join_after_ui();
    }
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
                settle_requests(s, cx);
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
                                settle_requests(s, cx);
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
                settle_requests(s, cx);
                s.activate_control(Control::OpenProject(0), window, cx);
                settle_requests(s, cx);
                s.focus.advanced = true;
                s.navigate(Screen::Setup, window, cx);
                settle_requests(s, cx);
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
            for message in ["short", "wrapped", "error"] {
                shell.update(cx, |s, cx| {
                    s.error = (message == "error")
                        .then(|| "A long readable failure with recovery instructions. ".repeat(80));
                    s.notice = match message {
                        "short" => Some("Settings saved locally.".into()),
                        "wrapped" => Some("Review included pages and coverage before capturing. Your active baseline is unchanged.".into()),
                        _ => None,
                    };
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
                assert!(
                    (copy.center().y - dismiss.center().y).abs() <= px(1.),
                    "message copy and Dismiss should share a vertical center"
                );
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
            settle_requests(s, cx);
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
            assert!(!s.field_enabled(17) && !s.field_enabled(18));
            assert!(!s.control_enabled(Control::Guided(41), cx));
            s.queue(Work::NetworkStatus, cx);
            settle_requests(s, cx);
            assert!(s.network_status.is_some());
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
            settle_requests(s, cx);
            s.name
                .update(cx, |input, cx| input.set_value("Renamed UI fixture", cx));
            s.crawl_fields[0].update(cx, |input, cx| input.set_value("250", cx));
            s.crawl_fields[15].update(cx, |input, cx| input.set_value("nb-NO", cx));
            s.choose(Choice::LoginRequired, cx);
            s.choose(Choice::EphemeralLogin, cx);
            s.choose(Choice::Fast, cx);
            s.choose(Choice::Fast, cx); // Explicit choices are idempotent, not cycles.
            s.save_settings(cx);
            settle_requests(s, cx);
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
            settle_requests(s, cx);
            assert!(s.evidence.comparison.is_some());
            s.evidence.clone()
        });
        // Observe GPUI's real rendered-frame Tab map, not another copy of the
        // mount predicates. These are simulated UI tests, not native pixels.
        for case in [
            "empty-results",
            "baseline-viewer",
            "baseline-pages",
            "discovery-preview",
            "comparison",
            "review-note",
            "historical-inspection",
            "tools-comparison",
            "drawing-comparison",
            "expanded-filters",
            "advanced-comparison",
            "loading-comparison",
            "history",
            "history-baselines",
            "history-storage",
            "settings",
            "projects",
            "setup",
            "advanced-setup",
            "website-stage",
            "public-access-stage",
            "login-access-stage",
            "pages-stage",
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
                s.review_tools = false;
                s.mask_draw = false;
                s.review_note_open = false;
                s.history_inspection = None;
                s.history_section = 0;
                s.result_filters_open = false;
                s.focus.setup_stage = SetupStage::Settings;
                s.focus.review_surface = ReviewSurface::Changes;
                s.draft_settings = s.project.as_ref().unwrap().project.clone();
                s.focus.screen = Screen::Results;
                s.viewer = Some(regressee_app::jobs::ViewerFrame {
                    identity: None,
                    before_environment: None,
                    before_capture: None,
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
                    "baseline-pages" => s.focus.review_surface = ReviewSurface::Pages,
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
                    "tools-comparison" => s.review_tools = true,
                    "drawing-comparison" => {
                        s.review_tools = true;
                        s.mask_draw = true;
                    }
                    "review-note" => s.review_note_open = true,
                    "historical-inspection" => {
                        s.history_inspection =
                            s.evidence.latest.as_ref().map(|r| (project, r.run.id))
                    }
                    "expanded-filters" => s.result_filters_open = true,
                    "advanced-comparison" => {
                        s.analysis_advanced = true;
                        s.review_tools = true;
                    }
                    "loading-comparison" => {
                        s.focus.busy = true;
                        s.focus.view_loading = true;
                        s.analysis_advanced = true;
                        s.review_tools = true;
                    }
                    "history" => s.focus.screen = Screen::History,
                    "history-baselines" => {
                        s.focus.screen = Screen::History;
                        s.history_section = 1;
                    }
                    "history-storage" => {
                        s.focus.screen = Screen::History;
                        s.history_section = 2;
                    }
                    "settings" => s.focus.screen = Screen::Settings,
                    "projects" => s.focus.screen = Screen::Projects,
                    "setup" => s.focus.screen = Screen::Setup,
                    "advanced-setup" => {
                        s.focus.screen = Screen::Setup;
                        s.focus.advanced = true;
                    }
                    "website-stage" => {
                        s.focus.screen = Screen::Setup;
                        s.focus.setup_stage = SetupStage::Website;
                    }
                    "public-access-stage" => {
                        s.focus.screen = Screen::Setup;
                        s.focus.setup_stage = SetupStage::Access;
                        s.choose(Choice::PublicAccess, cx);
                    }
                    "login-access-stage" => {
                        s.focus.screen = Screen::Setup;
                        s.focus.setup_stage = SetupStage::Access;
                        s.choose(Choice::LoginRequired, cx);
                    }
                    "pages-stage" => {
                        s.focus.screen = Screen::Setup;
                        s.focus.setup_stage = SetupStage::Pages;
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
            settle_requests(s, cx);
            s.analysis_advanced = true;
            s.review_tools = true;
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
            settle_requests(s, cx);
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
            settle_requests(s, cx);
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
            s.history_section = 1;
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
                s.continue_task(w, cx);
                settled(s, cx);
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
                assert!(
                    s.task
                        .as_ref()
                        .is_some_and(|task| task.state == regressee_app::task::State::Finished)
                );
                let imported = s.project.as_ref().unwrap().project.id;
                assert_ne!(imported, project);
                s.continue_task(w, cx);
                settled(s, cx);
                s.navigate(Screen::Projects, w, cx);
                settled(s, cx);
                assert_eq!(s.projects.items.len(), 2);
                s.focus.screen = Screen::History;
                s.refresh_visible(cx);
                settled(s, cx);
                assert!(!s.history.as_ref().unwrap().imported_runs.is_empty());
                s.activate_control(Control::Phase6(4), w, cx);
                settled(s, cx);
                assert!(s.deletion_preview.is_none());
                assert!(s.error.as_ref().unwrap().contains("Active baseline"));
                s.continue_task(w, cx);
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
            settle_requests(s, cx); // The census is now an independent read.
            s.queue(Work::EnvironmentStatus(project), cx);
            finish_request(s, cx);
            s.focus.screen = Screen::Results;
            s.evidence.analyses = vec![valid, masked];
            s.review_tools = true;
            s.approval_selection.insert(s.selected_key.clone().unwrap());
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Phase5(24), w, cx);
                finish_request(s, cx);
                assert_eq!(s.approval.as_ref().unwrap().analysis, valid);
                assert!(s.focus.approval_dialog);
                s.activate_control(Control::Guided(47), w, cx);
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
                s.activate_control(Control::Guided(47), w, cx);
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
                let analysis_id = s.analysis_pending.as_ref().unwrap().0;
                let update = receive_update_for(s, analysis_id, cx);
                assert_eq!(update.id, analysis_id);
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
        let (mut store, project, _, _, _) = approval_fixture::fixture(dir.path());
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
        let comparison = store.project_evidence(project).unwrap().comparison;
        let (jobs, events) = Jobs::start_without_runtime(paths);
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Results;
            s.focus.viewer = true;
            // This is comparison geometry, not a first-capture viewer. Supply
            // the saved fixture comparison before swapping in coordinate tiles.
            s.evidence.comparison = comparison;
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
                                    identity: None,
                                    before_environment: None,
                                    before_capture: None,
                                    tile: retained,
                                    panes: vec![tiles[retained..].to_vec(); 3],
                                    width,
                                    height,
                                    request: None,
                                    analysis: None,
                                });
                                // These pointer/seam cases explicitly choose a
                                // manual scale; the workspace's initial fit must
                                // not replace it during event-triggered renders.
                                s.viewer_fit = false;
                                s.zoom = zoom;
                                s.pan_x = 7.;
                                s.pan_y = (pan as f32 + direction * 20.) as u32;
                                s.mask_draw = false;
                                s.mask_rectangles.clear();
                                s.mask_dirty = false;
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
                            if !shell.update(cx, |s, _| s.viewer_current()) {
                                // Retained older tiles still support the seam
                                // transform checks above, but are not authority
                                // for drawing/saving ignored areas after a pan.
                                draw(&shell, cx);
                                let viewport = cx.debug_bounds("phase5-before").unwrap();
                                let point = viewport.origin
                                    + gpui::point(px(1. + 11. * zoom), px(1. + 8. * zoom));
                                cx.simulate_mouse_down(
                                    point,
                                    MouseButton::Left,
                                    gpui::Modifiers::none(),
                                );
                                draw(&shell, cx);
                                cx.simulate_mouse_up(
                                    point,
                                    MouseButton::Left,
                                    gpui::Modifiers::none(),
                                );
                                shell.update(cx, |s, _| {
                                    assert!(s.mask_drag.is_none());
                                    assert!(s.mask_rectangles.is_empty());
                                    assert!(!s.mask_dirty);
                                });
                            }
                            shell.update(cx, |s, _| {
                                // This saved-pixel geometry fixture deliberately
                                // supplies decoded tiles directly. Install the
                                // settled range before exercising real pointer
                                // mask actions; real Jobs acceptance is covered
                                // by phase5_real_tile_pointer_mask_coordinates.
                                let tile = s.pan_y as usize / 512;
                                let frame = s.viewer.as_mut().unwrap();
                                frame.tile = tile;
                                frame.panes = vec![tiles[tile..].to_vec(); 3];
                                assert!(s.viewer_current());
                            });
                            for pane in ["phase5-before", "phase5-after"] {
                                shell.update(cx, |s, _| s.mask_draw = true);
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
            settle_requests(s, cx);
            s.mask_rectangles.clear();
            s.mask_dirty = false;
            s.review_tools = true;
            s.focus.viewer = true;
            s.focus.busy = false;
            s.viewer = Some(regressee_app::jobs::ViewerFrame {
                identity: None,
                before_environment: None,
                before_capture: None,
                tile: 1,
                panes: vec![tiles.clone(), tiles.clone(), tiles],
                width: image.width,
                height: image.height,
                request: None,
                analysis: None,
            });
            s.viewer_fit = false;
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
                s.zoom_viewer_at(4., (0., 0.), cx);
                assert_eq!(s.zoom, 4.);
                s.pan_y += 128;
                s.refresh_viewer_navigation(cx);
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
            s.result_filters_open = true;
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
                assert_eq!(s.preview_page_ids(), vec![ids[1], ids[3]]);
                assert_eq!(s.selected_page, Some(ids[0]));
                s.activate_control(Control::Guided(24), w, cx);
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
                s.search = "no-matches".into(); // synchronous fixture; observer tested separately
                assert_eq!(s.selected_page, Some(ids[3]));
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
                s.search = "/match/one".into();
                s.select_preview_page(ids[1]);
                s.activate_control(Control::Guided(22), w, cx); // restore one, keep two excluded
                s.crawl_fields[9].update(cx, |i, cx| i.set_value("", cx));
                s.activate_control(Control::Guided(17), w, cx);
                assert_eq!(s.selected_page, Some(ids[1]));
                s.crawl_fields[9].update(cx, |i, cx| i.set_value("/match", cx));
                s.search = "/match".into();
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
            settle_requests(s, cx);
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
                assert!(!s.control_enabled(Control::Guided(31), cx));
                let requests = s.next_request;
                s.activate_control(Control::Guided(31), w, cx);
                assert!(s.pending.is_none());
                assert_eq!(s.next_request, requests);
                // A stale/direct handler call must retain the dispatch guard
                // as well as the disabled pointer/keyboard presentation.
                s.guided(31, cx);
                assert!(s.pending.is_none());
                assert_eq!(s.next_request, requests);
                assert!(s.error.as_ref().unwrap().contains("Save settings"));
                s.error = None;
                s.activate_control(Control::Save, w, cx);
            })
        });
        let id = shell.update(cx, |s, cx| {
            finish_request(s, cx);
            settle_requests(s, cx);
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
            s.refresh_visible(cx);
            settle_requests(s, cx);
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
        let waiting = shell
            .pending
            .or_else(|| shell.viewer_pending.as_ref().map(|(id, _, _)| *id))
            .or_else(|| shell.storage_pending.map(|(id, _)| id))
            .expect("a read or mutation is pending");
        let update = receive_update_for(shell, waiting, cx);
        assert!(
            !matches!(update.reply, Reply::Failure(_)),
            "{:?}",
            update.reply
        );
        shell.apply_update(update, cx);
    }
    /// Other tracked read replies can arrive before the operation under test.
    /// Apply those normally rather than asserting the next channel item is ours.
    fn receive_update_for(shell: &mut Shell, id: u64, cx: &mut Context<Shell>) -> Update {
        let deadline = std::time::Instant::now() + Duration::from_secs(10);
        loop {
            let remaining = deadline.saturating_duration_since(std::time::Instant::now());
            let update = shell
                .events
                .recv_timeout(remaining)
                .unwrap_or_else(|error| panic!("reply for request {id} did not arrive: {error}"));
            if update.id == id {
                return update;
            }
            shell.apply_update(update, cx);
        }
    }
    /// Await all currently tracked mutations and independent reads through the
    /// real reply acceptance path. Prefetch has no reply and is not a UI wait.
    fn settle_requests(shell: &mut Shell, cx: &mut Context<Shell>) {
        for _ in 0..32 {
            if shell.pending.is_none()
                && shell.viewer_pending.is_none()
                && shell.storage_pending.is_none()
            {
                return;
            }
            finish_request(shell, cx);
        }
        panic!("mutations and independent viewer/storage reads did not settle");
    }
    #[gpui::test]
    fn local_search_3000_targets_keeps_inspected_evidence_masks_and_approval_selection(
        cx: &mut TestAppContext,
    ) {
        use regressee_core::domain::PageId;
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let mut evidence = store.project_evidence(project).unwrap();
        drop(store);
        let run = evidence.latest.as_mut().unwrap();
        let template = run.run.targets[0].clone();
        run.run.targets = (0..3000)
            .map(|i| {
                let mut target = template.clone();
                target.key.page = PageId::new();
                target.requested = format!("https://example.test/page/{i}").parse().unwrap();
                target
            })
            .collect();
        let selected = run.run.targets[0].key.clone();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        let requests = shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.project = Some(record);
            s.evidence = evidence;
            s.evidence_dirty = false;
            s.focus.screen = Screen::Results;
            s.focus.project_saved = true;
            s.selected_key = Some(selected.clone());
            s.approval_selection.insert(selected.clone());
            s.mask_dirty = true;
            s.mask_rectangles = vec![regressee_app::analysis_engine::Rectangle {
                x: 1,
                y: 2,
                width: 3,
                height: 4,
            }];
            s.mask_selected = Some(0);
            s.viewer = Some(regressee_app::jobs::ViewerFrame {
                identity: None,
                before_environment: None,
                before_capture: None,
                tile: 1,
                panes: vec![vec![]; 3],
                width: 50,
                height: 60,
                request: None,
                analysis: None,
            });
            s.viewer_fit = false;
            s.pan_x = 123.;
            s.pan_y = 777;
            s.zoom = 0.75;
            s.crawl_fields[9].update(cx, |input, cx| {
                input.set_value(" HTTPS://EXAMPLE.TEST/PAGE/29 ", cx)
            });
            s.next_request
        });
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert_eq!(s.result_indices().len(), 111);
                assert_eq!(s.selected_key.as_ref(), Some(&selected));
                assert!(s.mask_dirty);
                assert_eq!(s.mask_rectangles.len(), 1);
                assert_eq!(s.mask_selected, Some(0));
                assert_eq!(s.viewer.as_ref().unwrap().width, 50);
                assert!(s.approval_selection.contains(&selected));
                assert_eq!((s.pan_x, s.pan_y, s.zoom), (123., 777, 0.75));
                assert_eq!(s.next_request, requests);
                assert!(s.viewer_pending.is_none());
                s.choose(Choice::ChangedResults, cx);
                s.group_profiles = true;
                assert!(s.result_indices().is_empty());
                assert_eq!(s.selected_key.as_ref(), Some(&selected));
                assert!(s.mask_dirty && s.approval_selection.contains(&selected));
                assert_eq!(s.next_request, requests);
                s.find_local(&FindLocal, window, cx);
                assert!(s.crawl_fields[9].read(cx).focus.is_focused(window));
                s.focus.dialog = true;
                window.focus(&s.handle(Control::CloseDialog, cx));
                s.find_local(&FindLocal, window, cx);
                assert!(s.handle(Control::CloseDialog, cx).is_focused(window));
            })
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn local_discovery_search_3000_pages_keeps_selection_and_excludes_exact_matches(
        cx: &mut TestAppContext,
    ) {
        use regressee_core::domain::*;
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        let selected = shell.update(cx, |s, cx| {
            finish_request(s, cx);
            let settings =
                Project::draft("Discovery search".into(), "https://example.test").unwrap();
            let pages: Vec<_> = (0..3000)
                .map(|i| InventoryPage {
                    id: PageId::new(),
                    canonical: format!("https://example.test/page/{i}").parse().unwrap(),
                    included: true,
                    reason: None,
                })
                .collect();
            let selected = pages[0].id;
            s.focus.screen = Screen::Results;
            s.focus.project_saved = true;
            s.focus.preview = true;
            s.project = Some(ProjectRecord {
                project: settings.clone(),
                revision: 1,
            });
            s.preview = Some(DiscoveryPreview {
                project_revision: 1,
                settings: settings.clone(),
                inventory: InventoryRevision {
                    id: InventoryId::new(),
                    project: settings.id,
                    identity_version: 2,
                    pages,
                    coverage: DiscoveryCoverage::Complete,
                },
                aliases: vec![],
                notes: vec![],
                visited: 0,
            });
            s.selected_page = Some(selected);
            s.crawl_fields[9].update(cx, |input, cx| input.set_value("/page/29", cx));
            selected
        });
        shell.update(cx, |s, cx| {
            assert_eq!(s.preview_page_ids().len(), 111);
            assert_eq!(s.selected_page, Some(selected));
            let visible: std::collections::BTreeSet<_> = s.preview_page_ids().into_iter().collect();
            let requests = s.next_request;
            s.guided(20, cx);
            let reviewed = s.preview.as_ref().unwrap().clone();
            for page in &reviewed.inventory.pages {
                assert_eq!(page.included, !visible.contains(&page.id));
            }
            s.crawl_fields[9].update(cx, |input, cx| input.set_value("does-not-match", cx));
            assert_eq!(
                reviewed
                    .inventory
                    .pages
                    .iter()
                    .filter(|p| p.included)
                    .count(),
                2889
            );
            assert_eq!(s.next_request, requests);
        });
        shell.update(cx, |s, _| {
            assert!(s.preview_page_ids().is_empty());
            assert_eq!(s.selected_page, Some(selected));
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn local_viewer_replies_reject_wrong_identity_and_tile_before_replacing_pixels(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let evidence = store.project_evidence(project).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.project = Some(record);
            s.selected_key = Some(evidence.latest.as_ref().unwrap().run.targets[0].key.clone());
            s.evidence = evidence;
            s.pan_y = 0;
            let identity = s.viewer_identity().unwrap();
            let frame = regressee_app::jobs::ViewerFrame {
                identity: Some(identity.clone()),
                before_environment: None,
                before_capture: None,
                tile: 0,
                panes: vec![],
                width: 50,
                height: 60,
                request: None,
                analysis: identity.analysis,
            };
            s.viewer = Some(frame.clone());
            for wrong in 0..5 {
                s.viewer_pending = Some((100 + wrong, identity.clone(), 0));
                let mut reply = frame.clone();
                match wrong {
                    0 => {
                        reply.identity.as_mut().unwrap().project =
                            regressee_core::domain::ProjectId::new()
                    }
                    1 => {
                        reply.identity.as_mut().unwrap().run = regressee_core::domain::RunId::new()
                    }
                    2 => {
                        reply.identity.as_mut().unwrap().key.page =
                            regressee_core::domain::PageId::new()
                    }
                    3 => {
                        reply.identity.as_mut().unwrap().analysis =
                            Some(regressee_core::domain::AnalysisId::new())
                    }
                    _ => reply.tile = 1,
                }
                reply.width = 999;
                s.apply_update(
                    Update {
                        id: 100 + wrong,
                        reply: Reply::Viewer(Box::new(reply)),
                    },
                    cx,
                );
                assert_eq!(s.viewer.as_ref().unwrap().width, 50);
                assert_eq!(
                    s.viewer.as_ref().unwrap().identity.as_ref(),
                    Some(&identity)
                );
            }
            s.viewer_pending = Some((200, identity, 0));
            let mut reply = frame;
            reply.width = 999;
            s.apply_update(
                Update {
                    id: 199,
                    reply: Reply::Viewer(Box::new(reply)),
                },
                cx,
            );
            assert_eq!(s.viewer.as_ref().unwrap().width, 50);
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn local_network_read_is_automatic_once_and_keeps_newer_draft(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, finish_request);
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.navigate(Screen::Settings, window, cx);
                assert!(s.network_load_started);
                assert!(!s.field_enabled(17));
                assert!(!s.control_enabled(Control::Guided(41), cx));
                settle_requests(s, cx);
                assert!(s.network_status.is_some());
                let requests = s.next_request;
                s.navigate(Screen::Projects, window, cx);
                s.navigate(Screen::Settings, window, cx);
                assert_eq!(s.next_request, requests);
                s.network_read_signature = Some((String::new(), String::new()));
                s.pending = Some(1000);
                s.crawl_fields[17]
                    .update(cx, |input, cx| input.set_value("https://draft.test", cx));
                s.apply_update(
                    Update {
                        id: 1000,
                        reply: Reply::Network {
                            settings: regressee_runtime::network::NetworkSettings {
                                proxy: Some("https://saved.test".into()),
                                trusted_ca: None,
                            },
                            message: "Read saved configuration".into(),
                        },
                    },
                    cx,
                );
                assert_eq!(
                    s.crawl_fields[17].read(cx).content.as_ref(),
                    "https://draft.test"
                );
            })
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn local_history_filters_guard_item_actions_and_focus_ranges_do_not_collide(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let evidence = store.project_evidence(project).unwrap();
        let history = store.history(project, 0).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_request(s, cx);
                s.project = Some(record);
                s.evidence = evidence;
                s.history = Some(history);
                s.focus.screen = Screen::History;
                s.focus.project_saved = true;
                s.focus.history_baselines = s.evidence.baselines.len();
                s.focus.history_runs = s.history.as_ref().unwrap().runs.len();
                s.history_section = 1;
                let mut handles = Vec::new();
                for control in (0..128)
                    .map(Control::HistoryBaseline)
                    .chain((0..64).map(Control::HistoryRun))
                    .chain((0..4).map(Control::HistoryKind))
                    .chain([
                        Control::HistoryClear,
                        Control::HistoryRefresh,
                        Control::Phase6(8),
                        Control::Phase7(0),
                        Control::HistorySearch,
                    ])
                {
                    let handle = s.handle(control, cx);
                    assert!(!handles.contains(&handle), "focus collision: {control:?}");
                    handles.push(handle);
                }
                assert!(!s.control_mounted(Control::Phase5(40)));
                s.details.insert(Detail::History);
                assert!(s.control_mounted(Control::Phase5(40)));
                let requests = s.next_request;
                s.mask_dirty = true;
                s.history_query = "does-not-match-any-loaded-record".into();
                for control in [
                    Control::Phase5(42),
                    Control::Phase6(2),
                    Control::Phase6(4),
                    Control::Phase6(5),
                    Control::HistoryBaseline(0),
                    Control::HistoryRun(0),
                ] {
                    assert!(!s.control_enabled(control, cx));
                    s.activate_control(control, window, cx);
                }
                assert_eq!(s.next_request, requests);
                assert!(s.mask_dirty);
                assert!(!s.control_mounted(Control::Phase5(40)));
                assert!(!s.control_mounted(Control::Phase6(9)));
            })
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn local_compatible_evidence_and_history_refresh_preserve_ids_pan_zoom_and_dirty_masks(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let evidence = store.project_evidence(project).unwrap();
        let history = store.history(project, 0).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.project = Some(record);
            s.evidence = evidence.clone();
            s.evidence_dirty = false;
            s.history = Some(history.clone());
            s.focus.project_saved = true;
            s.focus.screen = Screen::History;
            s.history_index = s.evidence.baselines.len() - 1;
            s.run_history_index = history.runs.len() - 1;
            let baseline = s.evidence.baselines[s.history_index].id;
            let run = history.runs[s.run_history_index].run.id;
            s.selected_key = Some(
                s.evidence
                    .latest
                    .as_ref()
                    .unwrap()
                    .run
                    .targets
                    .last()
                    .unwrap()
                    .key
                    .clone(),
            );
            let key = s.selected_key.clone();
            s.pan_x = 77.;
            s.pan_y = 1024;
            s.zoom = 1.5;
            s.pending = Some(2000);
            s.apply_update(
                Update {
                    id: 2000,
                    reply: Reply::Evidence(Box::new(evidence.clone())),
                },
                cx,
            );
            assert_eq!(s.selected_key, key);
            assert_eq!((s.pan_x, s.pan_y, s.zoom), (77., 1024, 1.5));
            let mut refreshed = history;
            refreshed.baselines.reverse();
            refreshed.runs.reverse();
            s.pending = Some(2001);
            s.apply_update(
                Update {
                    id: 2001,
                    reply: Reply::History {
                        history: Box::new(refreshed),
                    },
                },
                cx,
            );
            assert_eq!(s.evidence.baselines[s.history_index].id, baseline);
            assert_eq!(
                s.history.as_ref().unwrap().runs[s.run_history_index].run.id,
                run
            );
            s.mask_dirty = true;
            s.mask_rectangles = vec![regressee_app::analysis_engine::Rectangle {
                x: 1,
                y: 2,
                width: 3,
                height: 4,
            }];
            s.pending = Some(2002);
            s.apply_update(
                Update {
                    id: 2002,
                    reply: Reply::Evidence(Box::new(evidence)),
                },
                cx,
            );
            assert!(s.mask_dirty);
            assert_eq!(s.mask_rectangles.len(), 1);
            assert_eq!(s.selected_key, key);
            assert_eq!((s.pan_x, s.pan_y, s.zoom), (77., 1024, 1.5));
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn local_disabled_windows_preserve_drafts_skip_validation_and_allow_reanalysis(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_request(s, cx);
                s.focus.screen = Screen::Results;
                s.queue(Work::Load(project), cx);
                settle_requests(s, cx);
                s.review_tools = true;
                s.activate_control(Control::Phase5(3), window, cx);
                assert!(s.analysis_advanced);
                for (i, value) in [(3, "unfinished"), (4, "4096"), (5, "128"), (6, "32")] {
                    s.analysis_fields[i].update(cx, |input, cx| input.set_value(value, cx));
                }
                s.analysis_fields[3].update(cx, |input, _| {
                    input.touched = true;
                    input.correction = Some("Old invalid local-window correction".into());
                });
                s.analysis_policy.local = None;
                s.analysis_policy.validate().unwrap();
                s.set_analysis_fields(cx);
                // Loading a policy without local windows must not erase dormant
                // numeric drafts or turn them into errors beside disabled fields.
                for (i, value) in [(3, "unfinished"), (4, "4096"), (5, "128"), (6, "32")] {
                    let input = s.analysis_fields[i].read(cx);
                    assert_eq!(input.content.as_ref(), value);
                    assert!(input.validation_error().is_none());
                    assert!(input.correction.is_none());
                    assert!(!s.control_enabled(Control::AnalysisField(i), cx));
                }
                assert!(s.control_enabled(Control::AnalysisField(2), cx));
                let focus_before = s.presentation_focus;
                s.activate_control(Control::Phase5(4), window, cx);
                assert!(
                    s.analysis_due.is_some(),
                    "a valid disabled-local policy must schedule"
                );
                assert!(s.error.is_none());
                assert_eq!(s.presentation_focus, focus_before);
                s.analysis_due = None;
                let displayed = s.evidence.comparison.as_ref().unwrap().revision.id;
                s.analysis_fields[2].update(cx, |input, cx| input.set_value("5", cx));
                s.poll_analysis(cx);
                assert!(s.control_enabled(Control::AnalysisField(3), cx));
                assert!(s.analysis_fields[3].read(cx).validation_error().is_some());
                assert_eq!(s.analysis_fields[3].read(cx).content.as_ref(), "unfinished");
                assert!(
                    s.error.is_none(),
                    "incomplete typing must remain field-local"
                );
                assert_eq!(
                    s.evidence.comparison.as_ref().unwrap().revision.id,
                    displayed
                );
                s.activate_control(Control::Phase5(4), window, cx);
                assert!(s.analysis_due.is_none());
                assert_eq!(s.presentation_focus, Some(Control::AnalysisField(3)));
                s.analysis_fields[3].update(cx, |input, cx| input.set_value("8", cx));
                s.poll_analysis(cx);
                assert!(s.analysis_policy.local.is_some());
                s.analysis_policy.validate().unwrap();
                assert!(s.analysis_due.is_some());
                s.analysis_due = None;
                s.analysis_fields[3].update(cx, |input, cx| input.set_value("unfinished", cx));
                s.poll_analysis(cx);
                s.activate_control(Control::Phase5(4), window, cx);
                assert!(s.analysis_fields[3].read(cx).touched);
                s.analysis_fields[3].update(cx, |input, _| {
                    input.correction = Some("Old related-threshold correction".into());
                });
                s.analysis_fields[2].update(cx, |input, cx| input.set_value("", cx));
                s.poll_analysis(cx);
                assert!(s.analysis_fields[3].read(cx).touched);
                assert!(s.analysis_fields[3].read(cx).validation_error().is_none());
                assert!(s.analysis_fields[3].read(cx).correction.is_none());
                assert!(!s.control_enabled(Control::AnalysisField(3), cx));
                assert!(s.analysis_policy.local.is_none());
                assert!(s.error.is_none());
                s.activate_control(Control::Phase5(4), window, cx);
                assert!(s.analysis_due.is_some());
                s.analysis_due = None;
            })
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn local_history_refresh_reloads_current_page_baselines_and_preserves_selection_without_census(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (store, project, _, valid, _) = approval_fixture::fixture(dir.path());
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_request(s, cx);
                s.focus.screen = Screen::History;
                s.queue(Work::Load(project), cx);
                settle_requests(s, cx);
                assert_eq!(s.history.as_ref().unwrap().page, 0);
                assert!(!s.history.as_ref().unwrap().more);
                assert!(s.control_enabled(Control::HistoryRefresh, cx));
                assert!(!s.control_enabled(Control::Phase6(11), cx));
                let baseline = s.evidence.baselines[s.history_index].id;
                let run = s.history.as_ref().unwrap().runs[s.run_history_index].run.id;
                let key = s.selected_key.clone();
                let baseline_count = s.evidence.baselines.len();
                let storage_bytes = s.project_storage.as_ref().unwrap().bytes;
                let mut other_window = regressee_storage::Store::open(paths.clone()).unwrap();
                s.export_preview = Some(other_window.export_preview(project, baseline).unwrap());
                let approval = other_window
                    .approval_preview(valid, &[key.clone().unwrap()])
                    .unwrap();
                let new_baseline = approval.baseline.id;
                other_window
                    .accept_selected_in_context(&approval, &approval.context)
                    .unwrap();
                drop(other_window);
                s.approval = Some(approval);
                s.mask_dirty = true;
                s.mask_rectangles = vec![regressee_app::analysis_engine::Rectangle {
                    x: 1,
                    y: 2,
                    width: 3,
                    height: 4,
                }];
                s.pan_x = 77.;
                s.pan_y = 1024;
                s.zoom = 1.5;
                let requests = s.next_request;
                s.activate_control(Control::HistoryRefresh, window, cx);
                assert!(s.export_preview.is_none() && s.approval.is_none());
                assert!(s.pending.is_some());
                settle_requests(s, cx);
                assert_eq!(
                    s.next_request,
                    requests + 1,
                    "refresh history is not a storage census or evidence replacement"
                );
                assert_eq!(s.history.as_ref().unwrap().page, 0);
                assert_eq!(s.evidence.baselines.len(), baseline_count + 1);
                assert!(s.evidence.baselines.iter().any(|b| b.id == new_baseline));
                assert_eq!(
                    s.history.as_ref().unwrap().active_baseline,
                    Some(new_baseline)
                );
                assert_eq!(s.history_active_baseline(), Some(new_baseline));
                assert_ne!(s.history_active_baseline(), Some(baseline));
                assert_eq!(s.evidence.baselines[s.history_index].id, baseline);
                assert_eq!(
                    s.history.as_ref().unwrap().runs[s.run_history_index].run.id,
                    run
                );
                assert_eq!(s.project_storage.as_ref().unwrap().bytes, storage_bytes);
                assert!(s.storage_pending.is_none());
                assert_eq!(s.selected_key, key);
                assert!(s.mask_dirty);
                assert_eq!(s.mask_rectangles.len(), 1);
                assert_eq!((s.pan_x, s.pan_y, s.zoom), (77., 1024, 1.5));
                assert!(s.control_enabled(Control::HistoryRefresh, cx));
                // Refresh must reload the current provenance page, not reset it to
                // page zero. This fixture deliberately has no rows on page one.
                s.queue(Work::History { project, page: 1 }, cx);
                settle_requests(s, cx);
                assert!(s.history.as_ref().unwrap().runs.is_empty());
                s.activate_control(Control::HistoryRefresh, window, cx);
                assert!(s.pending.is_some());
                assert!(!s.control_enabled(Control::HistoryRefresh, cx));
                settle_requests(s, cx);
                assert_eq!(s.history.as_ref().unwrap().page, 1);
                assert!(s.history.as_ref().unwrap().runs.is_empty());
                assert!(s.evidence.baselines.is_empty());
                assert_eq!(s.history_active_baseline(), Some(new_baseline));
                assert!(s.mask_dirty);
                s.focus.dialog = true;
                assert!(!s.control_enabled(Control::HistoryRefresh, cx));
            })
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn local_validation_save_focuses_first_invalid_without_global_keystroke_errors(
        cx: &mut TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_request(s, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Website;
            s.address
                .update(cx, |input, cx| input.set_value("https:/", cx));
            let requests = s.next_request;
            s.save_settings(cx);
            assert_eq!(s.presentation_focus, Some(Control::Name));
            assert!(s.name.read(cx).touched && s.address.read(cx).touched);
            assert!(s.error.is_none());
            assert_eq!(s.next_request, requests);
            s.name
                .update(cx, |input, cx| input.set_value("Website", cx));
            s.save_settings(cx);
            assert_eq!(s.presentation_focus, Some(Control::Address));
            assert!(s.pending.is_none());
        });
        owned.join_after_ui();
    }
    #[gpui::test]
    fn save_task_protects_completion_and_refreshes_real_list_without_losing_draft(
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
        // Confirmed saves own protected focus until their authoritative reply.
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Projects), window, cx)
            })
        });
        shell.update(cx, |shell, cx| {
            assert_eq!(shell.focus.screen, Screen::Setup);
            assert_eq!(shell.pending, Some(save_id));
            assert!(shell.focus.busy);
            assert!(shell.projects.items.is_empty());
            assert_eq!(shell.name.read(cx).content, "Saved project");
            finish_request(shell, cx);
            assert_eq!(shell.focus.screen, Screen::Setup);
            assert!(shell.projects_dirty);
            let saved = shell.project.as_ref().unwrap();
            assert_eq!(saved.revision, 1);
            assert_eq!(saved.project.name, "Saved project");
            settle_requests(shell, cx);
        });
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Projects), window, cx);
            })
        });
        shell.update(cx, |shell, cx| {
            settle_requests(shell, cx);
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
        // Edit/save again. Both navigation shortcuts remain guarded until the
        // save completes; navigating afterwards must not lose the saved draft.
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Setup), window, cx)
            })
        });
        shell.update(cx, |shell, cx| {
            settle_requests(shell, cx);
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
            assert_eq!(shell.focus.screen, Screen::Setup);
            settle_requests(shell, cx);
        });
        cx.update(|window, cx| {
            shell.update(cx, |shell, cx| {
                shell.activate_control(Control::Navigate(Screen::Results), window, cx);
            })
        });
        shell.update(cx, |shell, cx| {
            // Phase 2 now refreshes actual evidence on the visible Results
            // surface. It must still not schedule a hidden Projects list or
            // replace the selected screen/settings snapshot.
            assert!(shell.pending.is_some());
            finish_request(shell, cx);
            settle_requests(shell, cx);
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
            let update = receive_update_for(shell, conflict_id, cx);
            assert_eq!(update.id, conflict_id);
            assert!(matches!(
                update.reply,
                Reply::Failure(regressee_app::jobs::StorageFailure::ConflictOrWrite)
            ));
            shell.apply_update(update, cx);
            assert_eq!(shell.focus.screen, Screen::Setup);
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
            settle_requests(s, cx);
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
            let request = s.pending.expect("the explicit capture request is pending");
            let mut preparation_seen = false;
            let update = loop {
                let update = receive_update_for(s, request, cx);
                if let Reply::Progress {
                    completed, total, ..
                } = &update.reply
                {
                    assert_eq!((*completed, *total), (0, None));
                    s.apply_update(update, cx);
                    assert!(s.focus.busy && s.focus.capturing);
                    preparation_seen = true;
                } else {
                    break update;
                }
            };
            assert!(
                preparation_seen,
                "the initial browser check must be visible"
            );
            assert_eq!(update.id, request);
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
