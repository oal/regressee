// SPDX-License-Identifier: MPL-2.0
//! One blocking storage owner, bounded command/update channels. Terminal replies
//! are retried (not silently dropped); shutdown breaks backpressure waits.
use crate::product::Product;
use anyhow::Context;
use regressee_core::domain::{Project, ProjectId};
use regressee_core::{crawl::DiscoveryPreview, domain::TargetKey};
use regressee_core::{domain::RunId, evidence::ProjectEvidence};
use regressee_storage::{AppPaths, ProjectPage, ProjectRecord, StorageSummary, Store};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread::{self, JoinHandle},
    time::Duration,
};

pub const COMMAND_CAPACITY: usize = 4;
pub const UPDATE_CAPACITY: usize = 4;
#[derive(Debug)]
pub enum Work {
    List {
        page: usize,
    },
    Load(ProjectId),
    Save {
        project: Box<Project>,
        expected_revision: Option<u64>,
    },
    InspectStorage,
    DiagnosticPreview,
    DiagnosticExport {
        report: Box<crate::diagnostics::Report>,
        path: std::path::PathBuf,
    },
    ExportPreview {
        project: ProjectId,
        baseline: regressee_core::domain::BaselineId,
    },
    Export {
        preview: Box<regressee_storage::ExportPreview>,
        path: std::path::PathBuf,
    },
    ImportPreview(std::path::PathBuf),
    Import {
        preview: Box<regressee_storage::ImportPreview>,
        path: std::path::PathBuf,
    },
    History {
        project: ProjectId,
        page: usize,
    },
    DeletePreview {
        project: ProjectId,
        item: regressee_storage::DeleteItem,
    },
    Delete(Box<regressee_storage::DeletionPreview>),
    RepairPreview,
    CleanupTemporaries(Box<regressee_storage::RepairPreview>),
    DeleteUnreferenced(Box<regressee_storage::RepairPreview>),
    RuntimeStatus,
    NetworkStatus,
    SaveNetwork(regressee_runtime::network::NetworkSettings),
    SetupRuntime,
    Evidence(ProjectId),
    Capture {
        project: ProjectId,
        check: bool,
    },
    Discover(ProjectId),
    ExistingInventory(ProjectId),
    EnvironmentStatus(ProjectId),
    SaveEnvironment {
        project: Box<Project>,
        revision: u64,
        environment: regressee_core::domain::EnvironmentId,
        settings: regressee_storage::LocalLoginSettings,
    },
    OpenLogin(ProjectId),
    ConfirmLogin(ProjectId),
    CloseLogin(ProjectId),
    ForgetLogin(ProjectId),
    ProbeEnvironment(ProjectId),
    CaptureInventory {
        project: ProjectId,
        preview: Option<Box<DiscoveryPreview>>,
        check: bool,
        resume: Option<RunId>,
    },
    SaveReplacement {
        project: Box<Project>,
        expected_revision: u64,
    },
    Activate {
        run: RunId,
        revision: u64,
    },
    Reanalyze {
        run: RunId,
        policy: crate::analysis_engine::Policy,
        cancel: Arc<AtomicBool>,
    },
    Mask {
        project: ProjectId,
        request: Box<crate::analysis_engine::AnalysisRequest>,
    },
    Review(regressee_core::domain::ReviewDecision),
    LoadAnalysis(regressee_core::domain::AnalysisId),
    Approval {
        analysis: regressee_core::domain::AnalysisId,
        selected: Vec<TargetKey>,
    },
    Accept {
        preview: Box<regressee_storage::ApprovalPreview>,
        context: regressee_storage::ApprovalContext,
    },
    Restore {
        baseline: regressee_core::domain::BaselineId,
        revision: u64,
    },
    PreviewImages {
        project: ProjectId,
        preview: Box<DiscoveryPreview>,
    },
    #[cfg(feature = "ui")]
    Viewer {
        project: ProjectId,
        tile: usize,
        key: Option<TargetKey>,
        analysis: Option<regressee_core::domain::AnalysisId>,
    },
}
#[derive(Debug)]
pub struct Request {
    pub id: u64,
    pub work: Work,
}
#[derive(Debug)]
pub enum Reply {
    Projects(ProjectPage),
    Project(Box<ProjectRecord>),
    Storage(StorageSummary),
    DiagnosticPreview(Box<crate::diagnostics::Report>),
    DiagnosticExported,
    ExportPreview(Box<regressee_storage::ExportPreview>),
    Exported,
    ImportPreview(Box<regressee_storage::ImportPreview>),
    Imported(Box<ProjectRecord>),
    History {
        history: Box<regressee_storage::HistoryPage>,
        storage: regressee_storage::ProjectStorage,
    },
    DeletePreview(Box<regressee_storage::DeletionPreview>),
    Deleted,
    RepairPreview(Box<regressee_storage::RepairPreview>),
    CleanedTemporaries,
    Failure(StorageFailure),
    ProductFailure(String),
    Runtime(String),
    RuntimeReadiness(crate::product::RuntimeReadiness),
    Network {
        settings: regressee_runtime::network::NetworkSettings,
        message: String,
    },
    Evidence(Box<ProjectEvidence>),
    Analysis(Box<regressee_core::evidence::ComparisonEvidence>),
    Approval(Box<regressee_storage::ApprovalPreview>),
    SavedReview(Vec<regressee_core::domain::ReviewDecision>),
    SavedMask,
    Discovery(Box<DiscoveryPreview>),
    Environment(Box<crate::auth::EnvironmentStatus>),
    Progress {
        message: String,
        completed: u32,
        total: Option<u32>,
    },
    #[cfg(feature = "ui")]
    Viewer(Box<ViewerFrame>),
}
#[cfg(feature = "ui")]
#[derive(Debug)]
pub struct ViewerFrame {
    pub tile: usize,
    pub panes: Vec<Vec<Arc<gpui::RenderImage>>>,
    pub width: u32,
    pub height: u32,
    pub request: Option<crate::analysis_engine::AnalysisRequest>,
    pub analysis: Option<regressee_core::domain::AnalysisId>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StorageFailure {
    Unavailable,
    InvalidProject,
    ConflictOrWrite,
    MissingOrCorrupt,
}
impl StorageFailure {
    pub fn message(self) -> &'static str {
        match self {
            Self::Unavailable => {
                "Local storage could not open. Check folder permissions and available space, then restart Regressee. No project was changed."
            }
            Self::InvalidProject => {
                "These project settings could not be saved. Check the name and complete website address."
            }
            Self::ConflictOrWrite => {
                "The project could not be saved. It may have changed in another window, or storage is unavailable. Reopen it and check available space."
            }
            Self::MissingOrCorrupt => {
                "The project could not be opened. Its stored settings may be missing or damaged. Keep the data folder for recovery."
            }
        }
    }
}
#[derive(Debug)]
pub struct Update {
    pub id: u64,
    pub reply: Reply,
}
pub struct Jobs {
    commands: SyncSender<Request>,
    stopped: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    operation_active: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    analysis_cancel: Mutex<Option<Arc<AtomicBool>>>,
    thread: Mutex<Option<JoinHandle<()>>>,
}
impl Jobs {
    pub fn start(paths: AppPaths) -> (Arc<Self>, Receiver<Update>) {
        Self::start_inner(paths, |paths| Product::configured(&paths.cache))
    }
    pub fn start_with_distribution(
        paths: AppPaths,
        distribution: regressee_runtime::distribution::Distribution,
    ) -> (Arc<Self>, Receiver<Update>) {
        Self::start_inner(paths, move |paths| {
            distribution.validate()?;
            Ok(Product {
                distribution: Some(distribution),
                official_runtime: false,
                runtime_cache: paths.cache.join("product-runtime"),
                auth: Default::default(),
            })
        })
    }
    /// Test-only injection; production `start` always uses configured app trust.
    #[cfg(any(test, feature = "ui-tests"))]
    pub fn start_with_product(paths: AppPaths, product: Product) -> (Arc<Self>, Receiver<Update>) {
        Self::start_inner(paths, move |_| {
            if let Some(d) = &product.distribution {
                d.validate()?;
            }
            Ok(product)
        })
    }
    #[cfg(any(test, feature = "ui-tests"))]
    pub fn start_without_runtime(paths: AppPaths) -> (Arc<Self>, Receiver<Update>) {
        let product = Product {
            distribution: None,
            official_runtime: false,
            runtime_cache: paths.cache.join("product-runtime"),
            auth: Default::default(),
        };
        Self::start_with_product(paths, product)
    }
    fn start_inner(
        paths: AppPaths,
        configure: impl FnOnce(&AppPaths) -> anyhow::Result<Product> + Send + 'static,
    ) -> (Arc<Self>, Receiver<Update>) {
        let (commands, rx) = mpsc::sync_channel(COMMAND_CAPACITY);
        let (updates, results) = mpsc::sync_channel(UPDATE_CAPACITY);
        let stopped = Arc::new(AtomicBool::new(false));
        let stop = stopped.clone();
        let cancelled = Arc::new(AtomicBool::new(false));
        let cancel = cancelled.clone();
        let operation_active = Arc::new(AtomicBool::new(false));
        let active = operation_active.clone();
        let paused = Arc::new(AtomicBool::new(false));
        let pause = paused.clone();
        let thread = thread::spawn(move || {
            let product = configure(&paths);
            let mut store = Store::open(paths).ok();
            if let Some(s) = store.as_mut() {
                let _ = s.recover_runs();
                let _ = s.complete_confirmed_gc();
            }
            while !stop.load(Ordering::Acquire) {
                let request: Request = match rx.recv_timeout(Duration::from_millis(20)) {
                    Ok(r) => r,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(_) => break,
                };
                let is_operation = is_operation(&request.work);
                let reply = match store.as_mut() {
                    None => Reply::Failure(StorageFailure::Unavailable),
                    Some(store) => match request.work {
                        Work::List { page } => store
                            .list_projects(page)
                            .map(Reply::Projects)
                            .unwrap_or(Reply::Failure(StorageFailure::MissingOrCorrupt)),
                        Work::Load(id) => store
                            .load_project(id)
                            .map(|p| Reply::Project(Box::new(p)))
                            .unwrap_or(Reply::Failure(StorageFailure::MissingOrCorrupt)),
                        Work::Save {
                            project,
                            expected_revision,
                        } => {
                            if project.validate().is_err() {
                                Reply::Failure(StorageFailure::InvalidProject)
                            } else {
                                store
                                    .save_project(&project, expected_revision)
                                    .map(|p| Reply::Project(Box::new(p)))
                                    .unwrap_or(Reply::Failure(StorageFailure::ConflictOrWrite))
                            }
                        }
                        Work::SaveEnvironment { project, revision, environment, settings } => {
                            if project.validate().is_err() || settings.validate().is_err() {
                                Reply::Failure(StorageFailure::InvalidProject)
                            } else {
                                store.save_project_and_environment(&project, revision, environment, &settings)
                                    .map(|record| Reply::Project(Box::new(record)))
                                    .unwrap_or(Reply::Failure(StorageFailure::ConflictOrWrite))
                            }
                        }
                        Work::DiagnosticPreview => crate::diagnostics::collect(store,product.as_ref().ok(),&cancel)
                            .map(|report|Reply::DiagnosticPreview(Box::new(report)))
                            .unwrap_or_else(|_|Reply::ProductFailure("Diagnostic preview unavailable. No private information was exported; cancel or try again.".into())),
                        Work::DiagnosticExport { report,path } => crate::diagnostics::export(&report,&path,&cancel)
                            .map(|_|Reply::DiagnosticExported)
                            .unwrap_or_else(|_|Reply::ProductFailure("Diagnostic file could not be saved. Choose a new filename and check folder permissions. Nothing was sent.".into())),
                        Work::InspectStorage => store
                            .summary()
                            .map(Reply::Storage)
                            .unwrap_or(Reply::Failure(StorageFailure::Unavailable)),
                        work @ (Work::ExportPreview {..} | Work::Export {..} | Work::ImportPreview(_) | Work::Import {..} | Work::History {..} | Work::DeletePreview {..} | Work::Delete(_) | Work::RepairPreview | Work::CleanupTemporaries(_) | Work::DeleteUnreferenced(_)) => storage_work(store,work,&cancel),
                        work=>match &product {
                             Err(_) if matches!(&work, Work::RuntimeStatus) => Reply::RuntimeReadiness(crate::product::RuntimeReadiness::invalid_configuration()),
                             Err(_)=>Reply::ProductFailure("Browser distribution configuration is invalid. Keep existing evidence and check its trusted configuration before retrying.".into()),
                             Ok(product)=>product_work(product,store,work,&cancel,&pause,&updates,request.id),
                        },
                    },
                };
                if is_operation {
                    active.store(false, Ordering::Release);
                }
                if !publish(
                    &updates,
                    Update {
                        id: request.id,
                        reply,
                    },
                    &stop,
                ) {
                    break;
                }
            }
        });
        (
            Arc::new(Self {
                commands,
                stopped,
                cancelled,
                operation_active,
                paused,
                analysis_cancel: Mutex::new(None),
                thread: Mutex::new(Some(thread)),
            }),
            results,
        )
    }
    /// UI-safe: no DB/file/process access, blocking send, or thread join.
    pub fn submit(&self, request: Request) -> Result<(), &'static str> {
        if self.stopped.load(Ordering::Acquire) {
            return Err("Background storage has stopped. Restart Regressee.");
        }
        if let Work::Reanalyze { cancel, .. } = &request.work
            && let Some(old) = self.analysis_cancel.lock().unwrap().replace(cancel.clone())
        {
            old.store(true, Ordering::Release);
        }
        let operation = is_operation(&request.work);
        if operation {
            if self
                .operation_active
                .compare_exchange(false, true, Ordering::AcqRel, Ordering::Acquire)
                .is_err()
            {
                return Err("A capture or setup is already active.");
            }
            self.cancelled.store(false, Ordering::Release);
            self.paused.store(false, Ordering::Release);
        }
        if self.commands.try_send(request).is_err() {
            if operation {
                self.operation_active.store(false, Ordering::Release);
            }
            return Err("Storage is busy. Wait for the current action to finish.");
        }
        Ok(())
    }
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Release);
        self.cancelled.store(true, Ordering::Release);
        if let Some(cancel) = self.analysis_cancel.lock().unwrap().as_ref() {
            cancel.store(true, Ordering::Release);
        }
    }
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }
    pub fn pause(&self) {
        self.paused.store(true, Ordering::Release);
    }
    /// Only after the native event loop exits, or in a headless test/tool.
    pub fn join_after_ui(&self) {
        self.stop();
        if let Some(thread) = self.thread.lock().unwrap().take() {
            let _ = thread.join();
        }
    }
}
fn product_work(
    product: &Product,
    store: &mut Store,
    work: Work,
    cancel: &Arc<AtomicBool>,
    pause: &Arc<AtomicBool>,
    updates: &SyncSender<Update>,
    id: u64,
) -> Reply {
    // The submit boundary creates one operation slot; cancellation requested
    // while a command is queued is preserved through its actual start.
    let result = (|| -> anyhow::Result<Reply> {
        if let Some(project) = first_use_project(&work) {
            let saved = store.load_project(project)?;
            saved.project.validate()?;
            if automatic_setup_allowed(&work, saved.project.active_baseline.is_some()) {
                product.ensure_runtime(cancel.clone(), &mut |phase| {
                    setup_update(updates, id, phase);
                })?;
            }
            // Same operation/cancellation slot, immediately before continuation.
            regressee_runtime::distribution::check_cancel(cancel)?;
        }
        match work {
            Work::Reanalyze {
                run,
                policy,
                cancel,
            } => Ok(Reply::Analysis(Box::new(crate::review::analyze_run(
                store, run, policy, &cancel,
            )?))),
            Work::Mask { project, request } => {
                store.save_mask(project, &request)?;
                Ok(Reply::SavedMask)
            }
            Work::Review(decision) => {
                store.save_review(&decision)?;
                Ok(Reply::SavedReview(store.review_history(
                    store.load_analysis(decision.analysis)?.revision.run,
                )?))
            }
            Work::LoadAnalysis(id) => {
                store.display_analysis(id)?;
                Ok(Reply::Analysis(Box::new(store.load_analysis(id)?)))
            }
            Work::Approval { analysis, selected } => Ok(Reply::Approval(Box::new(
                store.approval_preview(analysis, &selected)?,
            ))),
            Work::Accept { preview, context } => Ok(Reply::Project(Box::new(
                store.accept_selected_in_context(&preview, &context)?,
            ))),
            Work::Restore { baseline, revision } => Ok(Reply::Project(Box::new(
                store.restore_baseline(baseline, revision)?,
            ))),
            Work::PreviewImages { project, preview } => {
                product.preview_images(store, project, *preview, cancel.clone())?;
                Ok(Reply::Evidence(Box::new(store.project_evidence(project)?)))
            }
            Work::NetworkStatus => {
                let settings = product.network()?;
                let message = settings.diagnostics();
                Ok(Reply::Network { settings, message })
            }
            Work::SaveNetwork(settings) => {
                let message = product.save_network(&settings)?;
                Ok(Reply::Network { settings, message })
            }
            Work::EnvironmentStatus(project) => Ok(Reply::Environment(Box::new(
                product.environment_status(store, project)?,
            ))),
            Work::SaveEnvironment {
                project,
                revision,
                environment,
                settings,
            } => Ok(Reply::Project(Box::new(
                store.save_project_and_environment(&project, revision, environment, &settings)?,
            ))),
            Work::OpenLogin(project) => Ok(Reply::Environment(Box::new(product.open_login(
                store,
                project,
                cancel.clone(),
            )?))),
            Work::ConfirmLogin(project) => Ok(Reply::Environment(Box::new(
                product.confirm_login(store, project, cancel.clone())?,
            ))),
            Work::CloseLogin(project) => Ok(Reply::Environment(Box::new(
                product.close_login(store, project)?,
            ))),
            Work::ForgetLogin(project) => Ok(Reply::Environment(Box::new(
                product.forget_login(store, project)?,
            ))),
            Work::ProbeEnvironment(project) => Ok(Reply::Environment(Box::new(
                product.probe_environment(store, project, cancel.clone())?,
            ))),
            Work::RuntimeStatus => Ok(Reply::RuntimeReadiness(product.readiness(cancel)?)),
            Work::SetupRuntime => explicit_setup_reply(
                || {
                    product.setup_with_progress(cancel.clone(), &mut |phase| {
                        setup_update(updates, id, phase);
                    })
                },
                |committed| product.readiness(committed),
            ),
            Work::Evidence(project) => {
                Ok(Reply::Evidence(Box::new(store.project_evidence(project)?)))
            }
            Work::SaveReplacement {
                project,
                expected_revision,
            } => Ok(Reply::Project(Box::new(
                store.save_replacement_project(&project, expected_revision)?,
            ))),
            Work::Discover(project) => Ok(Reply::Discovery(Box::new(product.discover(
                store,
                project,
                cancel.clone(),
                &mut |message, completed, total| {
                    let _ = updates.try_send(Update {
                        id,
                        reply: Reply::Progress {
                            message: message.into(),
                            completed,
                            total,
                        },
                    });
                },
            )?))),
            Work::ExistingInventory(project) => {
                let saved = store.load_project(project)?;
                let baseline = store.load_baseline(
                    saved
                        .project
                        .active_baseline
                        .context("No saved baseline inventory")?,
                )?;
                let before = store.load_run(baseline.references[0].run)?;
                let mut inventory = before.run.inventory;
                inventory.id = regressee_core::domain::InventoryId::new();
                Ok(Reply::Discovery(Box::new(DiscoveryPreview{project_revision:saved.revision,settings:saved.project,inventory,aliases:vec![],notes:vec!["Existing inventory selected. No links were discovered; review the same pages for this replacement baseline.".into()],visited:0})))
            }
            Work::CaptureInventory {
                project,
                preview,
                check,
                resume,
            } => {
                product.capture_inventory(
                    store,
                    project,
                    preview.map(|p| *p),
                    check,
                    resume,
                    cancel.clone(),
                    pause.clone(),
                    &mut |message, completed, total| {
                        let _ = updates.try_send(Update {
                            id,
                            reply: Reply::Progress {
                                message: message.into(),
                                completed,
                                total,
                            },
                        });
                    },
                )?;
                Ok(Reply::Evidence(Box::new(store.project_evidence(project)?)))
            }
            Work::Capture { project, check } => {
                product.capture(
                    store,
                    project,
                    check,
                    cancel.clone(),
                    &mut |message, completed, total| {
                        let _ = updates.try_send(Update {
                            id,
                            reply: Reply::Progress {
                                message: message.into(),
                                completed,
                                total,
                            },
                        });
                    },
                )?;
                Ok(Reply::Evidence(Box::new(store.project_evidence(project)?)))
            }
            Work::Activate { run, revision } => Ok(Reply::Project(Box::new(
                product.activate(store, run, revision)?,
            ))),
            #[cfg(feature = "ui")]
            Work::Viewer {
                project,
                tile,
                key,
                analysis,
            } => {
                let mut e = store.project_evidence(project)?;
                if let Some(id) = analysis {
                    let a = store.load_analysis(id)?;
                    let run = store.load_run(a.revision.run)?;
                    anyhow::ensure!(
                        run.run.project == project,
                        "viewer analysis belongs to another project"
                    );
                    e.latest = Some(run);
                    e.comparison = Some(a);
                }
                let key = key
                    .or_else(|| e.latest.as_ref().map(|r| r.run.targets[0].key.clone()))
                    .context("No captured target selected")?;
                let baseline = e
                    .comparison
                    .as_ref()
                    .map(|a| store.load_baseline(a.revision.baseline))
                    .transpose()?
                    .or(e.baseline.clone());
                let reference = baseline
                    .as_ref()
                    .and_then(|b| b.references.iter().find(|r| r.key == key));
                let before_run = reference.map(|r| store.load_run(r.run)).transpose()?;
                let before = before_run
                    .as_ref()
                    .and_then(|r| r.evidence(&key))
                    .map(|c| &c.image);
                let after = e
                    .latest
                    .as_ref()
                    .and_then(|r| r.evidence(&key))
                    .map(|c| &c.image);
                let diff = e.comparison.as_ref().and_then(|a| {
                    a.heatmaps
                        .iter()
                        .find(|(k, _)| *k == key)
                        .map(|(_, h)| h)
                        .or(a.heatmap.as_ref())
                });
                let request = if let (Some(old), Some(run), Some(b), Some(a)) = (
                    &before_run,
                    &e.latest,
                    before_run.as_ref().and_then(|r| r.evidence(&key)),
                    e.latest.as_ref().and_then(|r| r.evidence(&key)),
                ) {
                    Some(
                        e.comparison
                            .as_ref()
                            .and_then(|a| a.perceptual.as_ref())
                            .and_then(|p| p.reports.iter().find(|(k, _)| *k == key))
                            .map(|(_, r)| r.request.clone())
                            .unwrap_or(crate::analysis_engine::AnalysisRequest {
                                before: regressee_core::evidence::descriptor(old, b)?,
                                after: regressee_core::evidence::descriptor(run, a)?,
                                policy: e
                                    .comparison
                                    .as_ref()
                                    .and_then(|a| a.perceptual.as_ref())
                                    .map(|p| p.policy.clone())
                                    .unwrap_or_else(|| {
                                        crate::review::preset(
                                            crate::analysis_engine::Preset::Balanced,
                                        )
                                    }),
                                mask: store.latest_mask(project, &key)?,
                            }),
                    )
                } else {
                    None
                };
                let mut panes = Vec::new();
                let mut width = 0;
                let mut height = 0;
                for image in [before, after, diff] {
                    let mut images = Vec::new();
                    if let Some(image) = image {
                        width = width.max(image.width);
                        height = height.max(image.height);
                        for a in image.tiles.iter().skip(tile).take(3) {
                            images.push(crate::images::viewer_tile(store, a)?);
                        }
                    }
                    panes.push(images);
                }
                Ok(Reply::Viewer(Box::new(ViewerFrame {
                    tile,
                    panes,
                    width,
                    height,
                    request,
                    analysis: e.comparison.as_ref().map(|a| a.revision.id),
                })))
            }
            _ => anyhow::bail!("invalid background product action"),
        }
    })();
    result.unwrap_or_else(|e| Reply::ProductFailure(format!("{e}")))
}
fn first_use_project(work: &Work) -> Option<ProjectId> {
    match work {
        Work::Discover(project) | Work::OpenLogin(project) | Work::ProbeEnvironment(project) => {
            Some(*project)
        }
        Work::Capture {
            project,
            check: false,
        }
        | Work::CaptureInventory {
            project,
            check: false,
            resume: None,
            ..
        } => Some(*project),
        _ => None,
    }
}
fn explicit_setup_reply(
    setup: impl FnOnce() -> anyhow::Result<String>,
    readiness: impl FnOnce(&AtomicBool) -> anyhow::Result<crate::product::RuntimeReadiness>,
) -> anyhow::Result<Reply> {
    setup()?;
    // Explicit installation already committed. Verify/report its actual state
    // without converting a late Cancel into a false installation failure. This
    // token is only for terminal verification, never for website continuation.
    Ok(Reply::RuntimeReadiness(readiness(&AtomicBool::new(false))?))
}
fn automatic_setup_allowed(work: &Work, has_baseline: bool) -> bool {
    match work {
        // Login/probe can prepare a pinned comparison: never select a new
        // default for an existing baseline, even if its current pointer is gone.
        Work::OpenLogin(_) | Work::ProbeEnvironment(_) => !has_baseline,
        _ => first_use_project(work).is_some(),
    }
}
fn setup_update(
    updates: &SyncSender<Update>,
    id: u64,
    phase: regressee_runtime::distribution::InstallProgress,
) {
    let (message, completed, total) = setup_progress(phase);
    let _ = updates.try_send(Update {
        id,
        reply: Reply::Progress {
            message,
            completed,
            total,
        },
    });
}
fn setup_progress(
    phase: regressee_runtime::distribution::InstallProgress,
) -> (String, u32, Option<u32>) {
    use regressee_runtime::distribution::InstallProgress;
    match phase {
        InstallProgress::Downloading { bytes, total } => {
            // Exact bytes within the installer's 2 GiB cap. A common unit scale
            // keeps larger future counters bounded without independently
            // saturating numerator and denominator into a false percentage.
            let scale = bytes
                .max(total.unwrap_or(0))
                .div_ceil(u64::from(u32::MAX))
                .max(1);
            let completed = (bytes / scale) as u32;
            let total_units = total.filter(|n| *n > 0).map(|n| n.div_ceil(scale) as u32);
            let message = match total {
                Some(total) => {
                    format!("Downloading private browser · {bytes} of {total} bytes received")
                }
                None => format!("Downloading private browser · {bytes} bytes received"),
            };
            (message, completed, total_units)
        }
        InstallProgress::Verifying => ("Verifying private browser files…".into(), 0, None),
        InstallProgress::Extracting => ("Installing private browser files…".into(), 0, None),
        InstallProgress::TestingBrowser => ("Testing the private browser…".into(), 0, None),
        InstallProgress::Ready => ("Private browser setup finished.".into(), 0, None),
    }
}
fn storage_work(store: &mut Store, work: Work, cancel: &AtomicBool) -> Reply {
    let result = (|| -> anyhow::Result<Reply> {
        match work {
            Work::ExportPreview { project, baseline } => Ok(Reply::ExportPreview(Box::new(
                store.export_preview(project, baseline)?,
            ))),
            Work::Export { preview, path } => {
                store.export_archive(&preview, &path, cancel)?;
                Ok(Reply::Exported)
            }
            Work::ImportPreview(path) => Ok(Reply::ImportPreview(Box::new(
                store.inspect_archive(&path, cancel)?.preview,
            ))),
            Work::Import { preview, path } => Ok(Reply::Imported(Box::new(
                store.import_archive(&path, &preview, cancel)?,
            ))),
            Work::History { project, page } => Ok(Reply::History {
                history: Box::new(store.history(project, page)?),
                storage: store.project_storage(project)?,
            }),
            Work::DeletePreview { project, item } => Ok(Reply::DeletePreview(Box::new(
                store.deletion_preview(project, item)?,
            ))),
            Work::Delete(preview) => {
                store.delete_confirmed(&preview)?;
                Ok(Reply::Deleted)
            }
            Work::RepairPreview => Ok(Reply::RepairPreview(Box::new(store.repair_preview()?))),
            Work::CleanupTemporaries(preview) => {
                store.cleanup_temporaries(&preview)?;
                Ok(Reply::CleanedTemporaries)
            }
            Work::DeleteUnreferenced(preview) => {
                store.delete_unreferenced_confirmed(&preview)?;
                Ok(Reply::CleanedTemporaries)
            }
            _ => anyhow::bail!("Invalid storage action"),
        }
    })();
    result.unwrap_or_else(|e| Reply::ProductFailure(e.to_string()))
}
fn is_operation(work: &Work) -> bool {
    matches!(
        work,
        Work::SetupRuntime
            | Work::DiagnosticPreview
            | Work::DiagnosticExport { .. }
            | Work::Export { .. }
            | Work::Import { .. }
            | Work::ImportPreview(_)
            | Work::OpenLogin(_)
            | Work::ConfirmLogin(_)
            | Work::ProbeEnvironment(_)
            | Work::Capture { .. }
            | Work::Discover(_)
            | Work::CaptureInventory { .. }
            | Work::PreviewImages { .. }
    )
}
impl Drop for Jobs {
    fn drop(&mut self) {
        self.stop();
    }
}
fn publish(sender: &SyncSender<Update>, mut update: Update, stop: &AtomicBool) -> bool {
    while !stop.load(Ordering::Acquire) {
        match sender.try_send(update) {
            Ok(()) => return true,
            Err(TrySendError::Full(value)) => {
                update = value;
                thread::sleep(Duration::from_millis(10));
            }
            Err(TrySendError::Disconnected(_)) => return false,
        }
    }
    false
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_setup_reports_committed_state_despite_late_cancel() -> anyhow::Result<()> {
        let cancel = AtomicBool::new(false);
        let reply = explicit_setup_reply(
            || {
                // Model Cancel arriving in the installer's post-commit Ready callback.
                cancel.store(true, Ordering::Release);
                Ok("Committed".into())
            },
            |verification| {
                assert!(cancel.load(Ordering::Acquire));
                assert!(!verification.load(Ordering::Acquire));
                Ok(crate::product::RuntimeReadiness {
                    status: crate::product::RuntimeState::Ready,
                    setup_available: true,
                    message: "Private browser is ready.",
                    identity: None,
                    scope: Some(regressee_runtime::distribution::Scope::OfficialPinned),
                })
            },
        )?;
        assert!(matches!(reply, Reply::RuntimeReadiness(state)
            if state.status == crate::product::RuntimeState::Ready));
        assert!(
            explicit_setup_reply(
                || anyhow::bail!("cancelled before commit"),
                |_| panic!("failed setup must not report committed readiness"),
            )
            .is_err()
        );
        Ok(())
    }
    #[test]
    fn first_use_policy_never_installs_for_comparison_or_resume() {
        let project = ProjectId::new();
        let resume = RunId::new();
        for work in [
            Work::Capture {
                project,
                check: true,
            },
            Work::CaptureInventory {
                project,
                preview: None,
                check: true,
                resume: None,
            },
            Work::CaptureInventory {
                project,
                preview: None,
                check: false,
                resume: Some(resume),
            },
            Work::CaptureInventory {
                project,
                preview: None,
                check: true,
                resume: Some(resume),
            },
            Work::RuntimeStatus,
            Work::ExistingInventory(project),
        ] {
            assert_eq!(first_use_project(&work), None);
            assert!(!automatic_setup_allowed(&work, false));
            assert!(!automatic_setup_allowed(&work, true));
        }
        for work in [
            Work::Discover(project),
            Work::Capture {
                project,
                check: false,
            },
            Work::CaptureInventory {
                project,
                preview: None,
                check: false,
                resume: None,
            },
        ] {
            assert_eq!(first_use_project(&work), Some(project));
            assert!(automatic_setup_allowed(&work, false));
            // Explicit new discovery/replacement is not a pinned comparison.
            assert!(automatic_setup_allowed(&work, true));
        }
        for work in [Work::OpenLogin(project), Work::ProbeEnvironment(project)] {
            assert!(automatic_setup_allowed(&work, false));
            assert!(!automatic_setup_allowed(&work, true));
        }
        assert!(!is_operation(&Work::RuntimeStatus));
    }

    #[test]
    fn setup_progress_reports_real_bytes_and_only_actual_phases() {
        use regressee_runtime::distribution::InstallProgress;
        let (_, completed, total) = setup_progress(InstallProgress::Downloading {
            bytes: 1024,
            total: Some(4096),
        });
        assert_eq!((completed, total), (1024, Some(4096)));
        let (_, completed, total) = setup_progress(InstallProgress::Downloading {
            bytes: 1024,
            total: None,
        });
        assert_eq!((completed, total), (1024, None));
        let (_, completed, total) = setup_progress(InstallProgress::Downloading {
            bytes: u64::MAX / 2,
            total: Some(u64::MAX),
        });
        assert!(completed > 0 && completed < total.unwrap());
        for phase in [
            InstallProgress::Verifying,
            InstallProgress::Extracting,
            InstallProgress::TestingBrowser,
            InstallProgress::Ready,
        ] {
            let (message, _, total) = setup_progress(phase);
            assert!(!message.contains("Downloading"));
            assert_eq!(total, None);
        }
    }

    #[test]
    fn readiness_and_missing_trust_first_use_use_the_same_owner_request() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let paths = AppPaths::isolated(root.path());
        let mut store = Store::open(paths.clone())?;
        let project = Project::draft("First use".into(), "https://example.test")?;
        store.save_project(&project, None)?;
        let product = Product {
            distribution: None,
            runtime_cache: paths.cache.join("product-runtime"),
            official_runtime: false,
            auth: Default::default(),
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let pause = Arc::new(AtomicBool::new(false));
        let (tx, rx) = mpsc::sync_channel(UPDATE_CAPACITY);
        let reply = product_work(
            &product,
            &mut store,
            Work::RuntimeStatus,
            &cancel,
            &pause,
            &tx,
            41,
        );
        assert!(matches!(
            reply,
            Reply::RuntimeReadiness(crate::product::RuntimeReadiness {
                status: crate::product::RuntimeState::UnavailableDistribution,
                setup_available: false,
                ..
            })
        ));
        let reply = product_work(
            &product,
            &mut store,
            Work::Discover(project.id),
            &cancel,
            &pause,
            &tx,
            42,
        );
        assert!(matches!(reply, Reply::ProductFailure(ref message)
            if message.contains("build is incomplete")));
        assert!(
            rx.try_recv().is_err(),
            "no second setup request or fake progress"
        );
        assert!(!product.runtime_cache.exists());
        cancel.store(true, Ordering::Release);
        let reply = product_work(
            &product,
            &mut store,
            Work::Discover(project.id),
            &cancel,
            &pause,
            &tx,
            43,
        );
        assert!(matches!(reply, Reply::ProductFailure(ref message)
            if message.contains("cancelled")));
        Ok(())
    }

    #[test]
    fn same_actions_as_ui_save_and_reopen_without_fixture_runtime() {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let project = Project::draft("Actor website".into(), "https://example.test").unwrap();
        let id = project.id;
        jobs.submit(Request {
            id: 1,
            work: Work::Save {
                project: Box::new(project),
                expected_revision: None,
            },
        })
        .unwrap();
        assert!(matches!(
            events.recv_timeout(Duration::from_secs(5)).unwrap().reply,
            Reply::Project(_)
        ));
        jobs.join_after_ui();
        let (jobs, events) = Jobs::start_without_runtime(paths);
        jobs.submit(Request {
            id: 2,
            work: Work::Load(id),
        })
        .unwrap();
        let result = events.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(result.id, 2);
        match result.reply {
            Reply::Project(p) => assert_eq!(p.project.name, "Actor website"),
            _ => panic!("project not reopened"),
        }
        jobs.join_after_ui();
    }
    #[test]
    fn terminal_updates_survive_backpressure_and_shutdown_is_bounded() {
        let (tx, rx) = mpsc::sync_channel(1);
        let stop = Arc::new(AtomicBool::new(false));
        tx.send(Update {
            id: 1,
            reply: Reply::Failure(StorageFailure::Unavailable),
        })
        .unwrap();
        let s = stop.clone();
        let thread = thread::spawn(move || {
            publish(
                &tx,
                Update {
                    id: 2,
                    reply: Reply::Failure(StorageFailure::Unavailable),
                },
                &s,
            )
        });
        thread::sleep(Duration::from_millis(30));
        assert_eq!(rx.recv().unwrap().id, 1);
        assert!(thread.join().unwrap());
        assert_eq!(rx.recv().unwrap().id, 2);
        let dir = tempfile::tempdir().unwrap();
        let (jobs, _) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        jobs.join_after_ui();
        assert!(
            jobs.submit(Request {
                id: 9,
                work: Work::InspectStorage
            })
            .is_err()
        );
    }
}
