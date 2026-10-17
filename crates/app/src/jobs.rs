// SPDX-License-Identifier: MPL-2.0
//! One blocking storage owner, bounded command/update channels. Terminal replies
//! are retried (not silently dropped); shutdown breaks backpressure waits.
use crate::{formatting::human_bytes, product::Product};
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
pub enum Command<W> {
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
    ProjectStorage(ProjectId),
    InspectRun {
        project: ProjectId,
        run: RunId,
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
        confirm_partial: bool,
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
    /// Private in-process extension, not a public command/protocol surface.
    Extension(W),
}
pub type Work = Command<std::convert::Infallible>;
#[derive(Debug)]
pub struct JobRequest<W> {
    pub id: u64,
    pub work: Command<W>,
}
pub type Request = JobRequest<std::convert::Infallible>;
#[derive(Debug)]
pub enum JobReply<R> {
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
    },
    ProjectStorage(regressee_storage::ProjectStorage),
    DeletePreview(Box<regressee_storage::DeletionPreview>),
    Deleted,
    RepairPreview(Box<regressee_storage::RepairPreview>),
    CleanedTemporaries,
    Failure(StorageFailure),
    ProductFailure(String),
    ActionRequired(crate::recovery::Problem),
    /// A worker checkpoint acknowledged cancellation. Late requests must never
    /// turn successful publication or an unrelated failure into cancellation.
    Cancelled,
    Runtime(String),
    RuntimeReadiness(crate::product::RuntimeReadiness),
    Network {
        settings: regressee_runtime::network::NetworkSettings,
        message: String,
    },
    Evidence(Box<ProjectEvidence>),
    InspectedRun {
        project: ProjectId,
        run: RunId,
        evidence: Box<ProjectEvidence>,
    },
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
    /// Real screenshot-target counts and frozen setup, separate from phase text.
    CaptureStatus(crate::task::CaptureStatus),
    Extension(R),
}
pub type Reply = JobReply<std::convert::Infallible>;
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
                "The project could not be saved. Your draft is kept. Check available space, folder permissions and whether another window changed this project before saving again."
            }
            Self::MissingOrCorrupt => {
                "The project could not be opened. Its stored settings may be missing or damaged. Keep the data folder for recovery."
            }
        }
    }
}
#[derive(Debug)]
pub struct JobUpdate<R> {
    pub id: u64,
    pub reply: JobReply<R>,
}
pub type Update = JobUpdate<std::convert::Infallible>;

/// An optional private actor adapter. Application services know nothing about
/// the extension's data or native widgets. All work still uses one store owner.
pub trait Extension: Default + Send + 'static {
    type Work: std::fmt::Debug + Send + 'static;
    type Reply: std::fmt::Debug + Send + 'static;
    type Shared: Default + Send + Sync + 'static;
    fn submit(
        shared: &Self::Shared,
        request: JobRequest<Self::Work>,
        stop: &AtomicBool,
    ) -> Result<(), &'static str>;
    fn take(shared: &Self::Shared) -> Option<(JobRequest<Self::Work>, bool, Option<u64>)>;
    fn before_work(&mut self, shared: &Self::Shared, work: &Command<Self::Work>);
    fn execute(
        &mut self,
        store: &mut Store,
        work: Self::Work,
        speculative: bool,
    ) -> anyhow::Result<Self::Reply>;
    fn publish(
        sender: &SyncSender<JobUpdate<Self::Reply>>,
        update: JobUpdate<Self::Reply>,
        stop: &AtomicBool,
        shared: &Self::Shared,
        generation: u64,
    ) -> bool;
    fn stop(shared: &Self::Shared);
}

#[derive(Default)]
pub struct NoExtension;
impl Extension for NoExtension {
    type Work = std::convert::Infallible;
    type Reply = std::convert::Infallible;
    type Shared = ();
    fn submit(_: &(), request: Request, _: &AtomicBool) -> Result<(), &'static str> {
        match request.work {
            Command::Extension(never) => match never {},
            _ => unreachable!(),
        }
    }
    fn take(_: &()) -> Option<(Request, bool, Option<u64>)> {
        None
    }
    fn before_work(&mut self, _: &(), _: &Work) {}
    fn execute(&mut self, _: &mut Store, work: Self::Work, _: bool) -> anyhow::Result<Self::Reply> {
        match work {}
    }
    fn publish(
        sender: &SyncSender<Update>,
        update: Update,
        stop: &AtomicBool,
        _: &(),
        _: u64,
    ) -> bool {
        publish(sender, update, stop)
    }
    fn stop(_: &()) {}
}

pub type Jobs = BackgroundJobs<NoExtension>;
pub struct BackgroundJobs<E: Extension> {
    commands: SyncSender<JobRequest<E::Work>>,
    stopped: Arc<AtomicBool>,
    cancelled: Arc<AtomicBool>,
    operation_active: Arc<AtomicBool>,
    paused: Arc<AtomicBool>,
    analysis_cancel: Mutex<Option<Arc<AtomicBool>>>,
    thread: Mutex<Option<JoinHandle<()>>>,
    extension: Arc<E::Shared>,
}
impl<E: Extension> BackgroundJobs<E> {
    pub fn start(paths: AppPaths) -> (Arc<Self>, Receiver<JobUpdate<E::Reply>>) {
        Self::start_inner(paths, |paths| Product::configured(&paths.cache))
    }
    pub fn start_with_distribution(
        paths: AppPaths,
        distribution: regressee_runtime::distribution::Distribution,
    ) -> (Arc<Self>, Receiver<JobUpdate<E::Reply>>) {
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
    #[cfg(any(test, feature = "test-support"))]
    pub fn start_with_product(
        paths: AppPaths,
        product: Product,
    ) -> (Arc<Self>, Receiver<JobUpdate<E::Reply>>) {
        Self::start_inner(paths, move |_| {
            if let Some(d) = &product.distribution {
                d.validate()?;
            }
            Ok(product)
        })
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn start_without_runtime(paths: AppPaths) -> (Arc<Self>, Receiver<JobUpdate<E::Reply>>) {
        let product = Product {
            distribution: None,
            official_runtime: false,
            runtime_cache: paths.cache.join("product-runtime"),
            auth: Default::default(),
        };
        Self::start_with_product(paths, product)
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn start_with_factory(
        paths: AppPaths,
        configure: impl FnOnce(&AppPaths) -> anyhow::Result<Product> + Send + 'static,
    ) -> (Arc<Self>, Receiver<JobUpdate<E::Reply>>) {
        Self::start_inner(paths, configure)
    }
    fn start_inner(
        paths: AppPaths,
        configure: impl FnOnce(&AppPaths) -> anyhow::Result<Product> + Send + 'static,
    ) -> (Arc<Self>, Receiver<JobUpdate<E::Reply>>) {
        use {Command as Work, JobReply as Reply, JobUpdate as Update};
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
        let extension = Arc::new(E::Shared::default());
        let shared = extension.clone();
        let thread = thread::spawn(move || {
            let product = configure(&paths);
            let mut store = Store::open(paths).ok();
            if let Some(s) = store.as_mut() {
                let _ = s.recover_runs();
                let _ = s.complete_confirmed_gc();
            }
            let mut extension = E::default();
            while !stop.load(Ordering::Acquire) {
                let (request, speculative, generation) = match rx
                    .try_recv()
                    .ok()
                    .map(|request| (request, false, None))
                    .or_else(|| E::take(&shared))
                {
                    Some(work) => work,
                    None => match rx.recv_timeout(Duration::from_millis(20)) {
                        Ok(request) => (request, false, None),
                        Err(mpsc::RecvTimeoutError::Timeout) => continue,
                        Err(_) => break,
                    },
                };
                let is_operation = is_operation(&request.work);
                extension.before_work(&shared, &request.work);
                let reply = match store.as_mut() {
                    None => JobReply::Failure(StorageFailure::Unavailable),
                    Some(store) => match request.work {
                        Command::Extension(work) => extension.execute(store, work, speculative)
                            .map(JobReply::Extension)
                            .unwrap_or_else(|e| JobReply::ProductFailure(e.to_string())),
                        Work::InspectRun { project, run } => store.inspect_run(project, run)
                            .map(|evidence| Reply::InspectedRun { project, run, evidence: Box::new(evidence) })
                            .unwrap_or_else(|e| Reply::ProductFailure(e.to_string())),
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
                                    .unwrap_or_else(|error| storage_failure(error, StorageFailure::ConflictOrWrite))
                            }
                        }
                        Work::SaveEnvironment { project, revision, environment, settings } => {
                            if project.validate().is_err() || settings.validate().is_err() {
                                Reply::Failure(StorageFailure::InvalidProject)
                            } else {
                                store.save_project_and_environment(&project, revision, environment, &settings)
                                    .map(|record| Reply::Project(Box::new(record)))
                                    .unwrap_or_else(|error| storage_failure(error, StorageFailure::ConflictOrWrite))
                            }
                        }
                        Work::DiagnosticPreview => crate::diagnostics::collect(store,product.as_ref().ok(),&cancel)
                            .map(|report|Reply::DiagnosticPreview(Box::new(report)))
                            .unwrap_or_else(|error| if cancellation_error(&error) { Reply::Cancelled } else { Reply::ProductFailure("Support report preview unavailable. No private information was exported; try again.".into()) }),
                        Work::DiagnosticExport { report,path } => crate::diagnostics::export(&report,&path,&cancel)
                            .map(|_|Reply::DiagnosticExported)
                            .unwrap_or_else(|error| if cancellation_error(&error) { Reply::Cancelled } else { Reply::ProductFailure("Support report could not be saved. Choose a new filename and check folder permissions. Nothing was sent.".into()) }),
                        Work::InspectStorage => store
                            .summary()
                            .map(Reply::Storage)
                            .unwrap_or(Reply::Failure(StorageFailure::Unavailable)),
                        work @ (Work::ExportPreview {..} | Work::Export {..} | Work::ImportPreview(_) | Work::Import {..} | Work::History {..} | Work::ProjectStorage(_) | Work::DeletePreview {..} | Work::Delete(_) | Work::RepairPreview | Work::CleanupTemporaries(_) | Work::DeleteUnreferenced(_)) => storage_work(store,work,&cancel),
                        work=>match &product {
                             Err(_) if matches!(&work, Work::RuntimeStatus) => Reply::RuntimeReadiness(crate::product::RuntimeReadiness::invalid_configuration()),
                              Err(_)=>Reply::ActionRequired(crate::recovery::Problem::Browser(crate::product::RuntimeState::UnavailableDistribution)),
                             Ok(product)=>product_work(product,store,work,&cancel,&pause,&updates,request.id),
                        },
                    },
                };
                if is_operation {
                    active.store(false, Ordering::Release);
                }
                if speculative {
                    continue;
                }
                if let Some(generation) = generation {
                    if !E::publish(
                        &updates,
                        Update {
                            id: request.id,
                            reply,
                        },
                        &stop,
                        &shared,
                        generation,
                    ) {
                        break;
                    }
                    continue;
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
                extension,
            }),
            results,
        )
    }
    /// UI-safe: no DB/file/process access, blocking send, or thread join.
    pub fn submit(&self, request: JobRequest<E::Work>) -> Result<(), &'static str> {
        use Command as Work;
        if matches!(&request.work, Command::Extension(_)) {
            if self.stopped.load(Ordering::Acquire) {
                return Err("Background storage has stopped. Restart Regressee.");
            }
            return E::submit(&self.extension, request, &self.stopped);
        }
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
        E::stop(&self.extension);
        if let Some(cancel) = self.analysis_cancel.lock().unwrap().as_ref() {
            cancel.store(true, Ordering::Release);
        }
    }
    /// Run a non-blocking extension mailbox update under the actor's stop guard.
    pub fn with_extension(
        &self,
        update: impl FnOnce(&E::Shared, &AtomicBool) -> Result<(), &'static str>,
    ) -> Result<(), &'static str> {
        if self.stopped.load(Ordering::Acquire) {
            return Err("Background storage has stopped. Restart Regressee.");
        }
        update(&self.extension, &self.stopped)
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
fn product_work<W, R>(
    product: &Product,
    store: &mut Store,
    work: Command<W>,
    cancel: &Arc<AtomicBool>,
    pause: &Arc<AtomicBool>,
    updates: &SyncSender<JobUpdate<R>>,
    id: u64,
) -> JobReply<R> {
    use {Command as Work, JobReply as Reply, JobUpdate as Update};
    // The submit boundary creates one operation slot; cancellation requested
    // while a command is queued is preserved through its actual start.
    let result = (|| -> anyhow::Result<JobReply<R>> {
        let discovering = matches!(&work, Work::Discover(_));
        let mut verified_runtime = None;
        if let Some(project) = first_use_project(&work) {
            if discovering {
                discovery_update(
                    updates,
                    id,
                    "Reading saved website settings…",
                    0,
                    None,
                    cancel,
                );
            }
            let saved = store.load_project(project)?;
            saved.project.validate()?;
            if automatic_setup_allowed(&work, saved.project.active_baseline.is_some()) {
                if discovering {
                    discovery_update(
                        updates,
                        id,
                        "Checking the installed browser files before visiting the website…",
                        0,
                        None,
                        cancel,
                    );
                }
                verified_runtime = Some(product.ensure_runtime(cancel.clone(), &mut |phase| {
                    setup_update(updates, id, phase);
                })?);
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
                let run = store.save_review(&decision)?;
                Ok(Reply::SavedReview(store.review_history(run)?))
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
                product.preview_images_with_progress(
                    store,
                    project,
                    *preview,
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
                    discovery_update(updates, id, message, completed, total, cancel);
                },
            )?))),
            Work::ExistingInventory(project) => {
                let saved = store.load_project(project)?;
                let latest = store
                    .project_evidence(project)?
                    .latest
                    .context("No recorded page list")?;
                let mut inventory = latest.run.inventory;
                inventory.id = regressee_core::domain::InventoryId::new();
                Ok(Reply::Discovery(Box::new(DiscoveryPreview{project_revision:saved.revision,settings:saved.project,inventory,aliases:vec![],notes:vec!["Recorded page list selected. No website was visited; review included pages before starting another capture.".into()],visited:0})))
            }
            Work::CaptureInventory {
                project,
                preview,
                check,
                resume,
            } => {
                product.capture_inventory_with_status(
                    store,
                    verified_runtime,
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
                    &mut |run| {
                        let _ = updates.try_send(Update {
                            id,
                            reply: Reply::CaptureStatus(crate::task::CaptureStatus::from_run(run)),
                        });
                    },
                )?;
                Ok(Reply::Evidence(Box::new(store.project_evidence(project)?)))
            }
            Work::Capture { project, check } => {
                product.capture_with_runtime(
                    store,
                    verified_runtime,
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
            Work::Activate {
                run,
                revision,
                confirm_partial,
            } => Ok(Reply::Project(Box::new(
                store.activate_baseline_with_partial_confirmation(
                    run,
                    revision,
                    confirm_partial,
                )?,
            ))),
            _ => anyhow::bail!("invalid background product action"),
        }
    })();
    operation_reply(result)
}
fn discovery_update<R>(
    updates: &SyncSender<JobUpdate<R>>,
    id: u64,
    message: &str,
    completed: u32,
    total: Option<u32>,
    cancel: &AtomicBool,
) {
    use {JobReply as Reply, JobUpdate as Update};
    let update = Update {
        id,
        reply: Reply::Progress {
            message: message.into(),
            completed,
            total,
        },
    };
    if completed == 0 && total.is_none() {
        // Do not drop the only explanation for a long browser/first-page wait
        // when fast preceding phases filled the small update queue. Stop/Quit
        // releases this wait through the same operation cancellation token.
        let _ = publish(updates, update, cancel);
    } else {
        let _ = updates.try_send(update);
    }
}
fn first_use_project<W>(work: &Command<W>) -> Option<ProjectId> {
    use Command as Work;
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
fn explicit_setup_reply<R>(
    setup: impl FnOnce() -> anyhow::Result<String>,
    readiness: impl FnOnce(&AtomicBool) -> anyhow::Result<crate::product::RuntimeReadiness>,
) -> anyhow::Result<JobReply<R>> {
    setup()?;
    // Explicit installation already committed. Verify/report its actual state
    // without converting a late Cancel into a false installation failure. This
    // token is only for terminal verification, never for website continuation.
    Ok(JobReply::RuntimeReadiness(readiness(&AtomicBool::new(
        false,
    ))?))
}
fn automatic_setup_allowed<W>(work: &Command<W>, has_baseline: bool) -> bool {
    use Command as Work;
    match work {
        // Login/probe can prepare a pinned comparison: never select a new
        // default for an existing baseline, even if its current pointer is gone.
        Work::OpenLogin(_) | Work::ProbeEnvironment(_) => !has_baseline,
        _ => first_use_project(work).is_some(),
    }
}
fn setup_update<R>(
    updates: &SyncSender<JobUpdate<R>>,
    id: u64,
    phase: regressee_runtime::distribution::InstallProgress,
) {
    use {JobReply as Reply, JobUpdate as Update};
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
            let message = match total.filter(|n| *n > 0) {
                Some(total) => {
                    format!(
                        "Downloading private browser · {} of {} received",
                        human_bytes(bytes),
                        human_bytes(total)
                    )
                }
                None => format!(
                    "Downloading private browser · {} received",
                    human_bytes(bytes)
                ),
            };
            (message, completed, total_units)
        }
        InstallProgress::Verifying => ("Verifying private browser files…".into(), 0, None),
        InstallProgress::Extracting => ("Installing private browser files…".into(), 0, None),
        InstallProgress::TestingBrowser => ("Testing the private browser…".into(), 0, None),
        InstallProgress::Publishing => {
            ("Saving the verified browser installation…".into(), 0, None)
        }
        InstallProgress::Ready => ("Private browser setup finished.".into(), 0, None),
    }
}
fn storage_work<W, R>(store: &mut Store, work: Command<W>, cancel: &AtomicBool) -> JobReply<R> {
    use {Command as Work, JobReply as Reply};
    let result = (|| -> anyhow::Result<JobReply<R>> {
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
            }),
            Work::ProjectStorage(project) => {
                Ok(Reply::ProjectStorage(store.project_storage(project)?))
            }
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
    operation_reply(result)
}
fn cancellation_error(error: &anyhow::Error) -> bool {
    error.chain().any(|cause| {
        if let Some(failure) = cause.downcast_ref::<regressee_runtime::supervisor::WorkerFailure>()
        {
            return failure.0 == regressee_runtime::protocol::FailureCode::Cancelled;
        }
        // Recognize existing checkpoints, never an arbitrary diagnostic that
        // happens to mention cancellation.
        matches!(
            cause.to_string().as_str(),
            "operation cancelled"
                | "Operation cancelled; retained evidence is unchanged"
                | "analysis cancelled"
                | "Diagnostic preview cancelled"
                | "Diagnostic export cancelled"
        )
    })
}
fn operation_reply<R>(result: anyhow::Result<JobReply<R>>) -> JobReply<R> {
    use JobReply as Reply;
    result.unwrap_or_else(|error| {
        if cancellation_error(&error) {
            Reply::Cancelled
        } else if let Some(setup) = error.chain().find_map(|cause| {
            cause.downcast_ref::<regressee_runtime::setup_diagnostics::SetupFailure>()
        }) {
            Reply::ProductFailure(format!(
                "{setup}\nRegressee version: {}.",
                env!("CARGO_PKG_VERSION")
            ))
        } else if let Some(problem) = io_recovery(&error) {
            Reply::ActionRequired(problem)
        } else if let Some(blocked) = error.downcast_ref::<crate::product::RuntimeBlocked>() {
            Reply::ActionRequired(crate::recovery::Problem::Browser(blocked.0))
        } else if let Some(worker) =
            error.downcast_ref::<regressee_runtime::supervisor::WorkerFailure>()
        {
            Reply::ActionRequired(crate::recovery::Problem::Worker(worker.0))
        } else {
            Reply::ProductFailure(error.to_string())
        }
    })
}

fn io_recovery(error: &anyhow::Error) -> Option<crate::recovery::Problem> {
    error.chain().find_map(
        |cause| match cause.downcast_ref::<std::io::Error>()?.kind() {
            std::io::ErrorKind::PermissionDenied => {
                Some(crate::recovery::Problem::PermissionDenied)
            }
            std::io::ErrorKind::StorageFull => Some(crate::recovery::Problem::LowSpace),
            _ => None,
        },
    )
}

fn storage_failure<R>(error: anyhow::Error, fallback: StorageFailure) -> JobReply<R> {
    use JobReply as Reply;
    io_recovery(&error).map_or(Reply::Failure(fallback), Reply::ActionRequired)
}

#[cfg(test)]
mod task_reply_tests {
    use super::*;

    #[test]
    fn acknowledged_cancellation_is_distinct_from_failures_and_committed_success() {
        let cancelled = anyhow::anyhow!("operation cancelled").context("Preparing browser");
        assert!(matches!(operation_reply(Err(cancelled)), Reply::Cancelled));
        let failure = regressee_runtime::supervisor::WorkerFailure(
            regressee_runtime::protocol::FailureCode::Cancelled,
        );
        assert!(matches!(
            operation_reply(Err(failure.into())),
            Reply::Cancelled
        ));
        assert!(matches!(
            operation_reply(Err(anyhow::anyhow!("Disk failure after cancel requested"))),
            Reply::ProductFailure(_)
        ));
        assert!(matches!(
            operation_reply(Ok(Reply::Exported)),
            Reply::Exported
        ));
    }

    #[test]
    fn known_causes_survive_context_without_exposing_private_stack_text() {
        use crate::{
            product::{RuntimeBlocked, RuntimeState},
            recovery::Problem,
        };
        use regressee_runtime::{protocol::FailureCode, supervisor::WorkerFailure};
        for state in [
            RuntimeState::NotInstalled,
            RuntimeState::UpdateRequired,
            RuntimeState::RepairRequired,
            RuntimeState::UnavailableDistribution,
            RuntimeState::UnsupportedPlatform,
        ] {
            let error = anyhow::Error::new(RuntimeBlocked(state))
                .context("private website/query/session context");
            assert!(
                matches!(operation_reply(Err(error)), Reply::ActionRequired(Problem::Browser(actual)) if actual == state)
            );
        }
        for code in [
            FailureCode::NeedsLogin,
            FailureCode::AccessDenied,
            FailureCode::Timeout,
            FailureCode::RateLimited,
            FailureCode::WorkerStopped,
        ] {
            let error = anyhow::Error::new(WorkerFailure(code)).context("private site error");
            assert!(
                matches!(operation_reply(Err(error)), Reply::ActionRequired(Problem::Worker(actual)) if actual == code)
            );
        }
        for (kind, expected) in [
            (
                std::io::ErrorKind::PermissionDenied,
                Problem::PermissionDenied,
            ),
            (std::io::ErrorKind::StorageFull, Problem::LowSpace),
        ] {
            let error = anyhow::Error::new(std::io::Error::new(kind, "private path"))
                .context("Writing private data");
            assert!(
                matches!(operation_reply(Err(error)), Reply::ActionRequired(problem) if problem == expected)
            );
            let error = anyhow::Error::new(std::io::Error::new(kind, "private path"))
                .context("Saving private project");
            assert!(
                matches!(storage_failure(error, StorageFailure::ConflictOrWrite), Reply::ActionRequired(problem) if problem == expected)
            );
        }
        assert!(matches!(
            storage_failure(
                anyhow::anyhow!("revision changed"),
                StorageFailure::ConflictOrWrite
            ),
            Reply::Failure(StorageFailure::ConflictOrWrite)
        ));
    }

    #[test]
    fn wrapped_setup_failures_keep_details_but_other_operations_do_not_expand_errors() {
        use regressee_runtime::{distribution::InstallProgress, setup_diagnostics::SetupAttempt};
        let mut attempt = SetupAttempt::default();
        attempt.observe(InstallProgress::Downloading {
            bytes: 1024,
            total: Some(2048),
        });
        let source = anyhow::anyhow!("Read timed out")
            .context("Downloading Chromium from storage.googleapis.com");
        let error = anyhow::Error::new(attempt.failure(source)).context("Preparing to find pages");
        let Reply::ProductFailure(message) = operation_reply(Err(error)) else {
            panic!("expected setup failure")
        };
        assert!(message.starts_with("Browser setup failed during download."));
        assert!(message.contains("1024 of 2048 bytes"));
        assert!(message.contains("Downloading Chromium from storage.googleapis.com"));
        assert!(message.contains("Read timed out"));
        assert!(message.contains("Regressee version:"));

        let source = anyhow::anyhow!("private site/session detail").context("Capture failed");
        let Reply::ProductFailure(message) = operation_reply(Err(source)) else {
            panic!("expected failure")
        };
        assert_eq!(message, "Capture failed");
    }

    #[test]
    fn setup_failure_wrapper_preserves_cancellation_classification() {
        let error = regressee_runtime::setup_diagnostics::SetupAttempt::default()
            .failure(anyhow::anyhow!("operation cancelled"));
        assert!(matches!(
            operation_reply(Err(error.into())),
            Reply::Cancelled
        ));
    }

    #[cfg(all(target_arch = "x86_64", any(target_os = "linux", windows)))]
    #[test]
    fn explicit_and_first_use_setup_preserve_real_preflight_io_errors() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let paths = AppPaths::isolated(root.path());
        let product = Product::configured(&paths.cache)?;
        let mut store = Store::open(paths.clone())?;
        let project = Project::draft("Setup diagnostics fixture".into(), "https://example.test/")?;
        store.save_project(&project, None)?;
        std::fs::write(
            paths.cache.join("network-settings.json"),
            serde_json::to_vec(&regressee_runtime::network::NetworkSettings {
                proxy: None,
                trusted_ca: Some(root.path().join("missing-ca.pem")),
            })?,
        )?;
        let cancel = Arc::new(AtomicBool::new(false));
        let (updates, _results) = mpsc::sync_channel(16);
        for work in [
            Work::SetupRuntime,
            Work::Discover(project.id),
            Work::OpenLogin(project.id),
        ] {
            let Reply::ProductFailure(message) =
                product_work(&product, &mut store, work, &cancel, &cancel, &updates, 1)
            else {
                panic!("expected setup failure")
            };
            assert!(
                message.starts_with("Browser setup failed during preparation."),
                "{message}"
            );
            assert!(message.contains("Browser network settings could not be validated"));
            assert!(message.contains("os error 2"), "{message}");
            assert!(!message.contains("https://example.test"));
            assert!(
                !product.runtime_cache.exists(),
                "preflight failure started installation"
            );
        }
        assert_eq!(store.load_project(project.id)?.revision, 1);
        Ok(())
    }
}
fn is_operation<W>(work: &Command<W>) -> bool {
    use Command as Work;
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
impl<E: Extension> Drop for BackgroundJobs<E> {
    fn drop(&mut self) {
        self.stop();
    }
}
fn publish<R>(
    sender: &SyncSender<JobUpdate<R>>,
    mut update: JobUpdate<R>,
    stop: &AtomicBool,
) -> bool {
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
    fn history_and_storage_census_are_independent_replies() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let mut store = Store::open(AppPaths::isolated(root.path()))?;
        let project = Project::draft("History fixture".into(), "https://example.test/")?;
        store.save_project(&project, None)?;
        let cancel = AtomicBool::new(false);
        assert!(
            matches!(storage_work(&mut store, Work::History { project: project.id, page: 0 }, &cancel), Reply::History { history } if history.runs.is_empty())
        );
        assert!(
            matches!(storage_work(&mut store, Work::ProjectStorage(project.id), &cancel), Reply::ProjectStorage(storage) if storage.project == project.id && storage.bytes == 0)
        );
        Ok(())
    }
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
            explicit_setup_reply::<std::convert::Infallible>(
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
        let (message, completed, total) = setup_progress(InstallProgress::Downloading {
            bytes: 1024,
            total: Some(4096),
        });
        assert_eq!(
            message,
            "Downloading private browser · 1.0 KiB of 4.0 KiB received"
        );
        assert_eq!((completed, total), (1024, Some(4096)));
        let (message, completed, total) = setup_progress(InstallProgress::Downloading {
            bytes: 1024,
            total: None,
        });
        assert_eq!(message, "Downloading private browser · 1.0 KiB received");
        assert_eq!((completed, total), (1024, None));
        let (message, completed, total) = setup_progress(InstallProgress::Downloading {
            bytes: 82_350_444,
            total: Some(258_433_266),
        });
        assert_eq!(
            message,
            "Downloading private browser · 78.5 MiB of 246.5 MiB received"
        );
        assert_eq!((completed, total), (82_350_444, Some(258_433_266)));
        let (message, completed, total) = setup_progress(InstallProgress::Downloading {
            bytes: 0,
            total: Some(0),
        });
        assert_eq!(message, "Downloading private browser · 0 B received");
        assert_eq!((completed, total), (0, None));
        let (_, completed, total) = setup_progress(InstallProgress::Downloading {
            bytes: u64::MAX / 2,
            total: Some(u64::MAX),
        });
        assert!(completed > 0 && completed < total.unwrap());
        for phase in [
            InstallProgress::Verifying,
            InstallProgress::Extracting,
            InstallProgress::TestingBrowser,
            InstallProgress::Publishing,
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
        assert!(matches!(
            reply,
            Reply::ActionRequired(crate::recovery::Problem::Browser(
                crate::product::RuntimeState::UnavailableDistribution
            ))
        ));
        let phases: Vec<_> = rx.try_iter().collect();
        assert_eq!(phases.len(), 2);
        for phase in &phases {
            assert_eq!(phase.id, 42, "preparation belongs to the original request");
            assert!(matches!(
                phase.reply,
                Reply::Progress {
                    completed: 0,
                    total: None,
                    ..
                }
            ));
        }
        assert!(
            matches!(&phases[1].reply, Reply::Progress { message, .. } if message.contains("Checking the installed browser files")),
            "verification must be visible even before the first visit"
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
        assert!(matches!(reply, Reply::Cancelled));
        assert!(
            rx.try_recv().is_err(),
            "a queued Stop cannot restart preparation"
        );
        Ok(())
    }

    #[test]
    fn discovery_startup_status_survives_backpressure_but_stops_on_cancel() {
        let (tx, rx) = mpsc::sync_channel(1);
        tx.send(Update {
            id: 1,
            reply: Reply::Progress {
                message: "Earlier phase".into(),
                completed: 0,
                total: None,
            },
        })
        .unwrap();
        let cancelled = Arc::new(AtomicBool::new(false));
        let sender = tx.clone();
        let token = cancelled.clone();
        let task = thread::spawn(move || {
            discovery_update(
                &sender,
                2,
                "Loading the first page and looking for links…",
                0,
                None,
                &token,
            )
        });
        assert_eq!(rx.recv_timeout(Duration::from_secs(3)).unwrap().id, 1);
        let update = rx.recv_timeout(Duration::from_secs(3)).unwrap();
        assert_eq!(update.id, 2);
        assert!(matches!(
            update.reply,
            Reply::Progress {
                completed: 0,
                total: None,
                ..
            }
        ));
        task.join().unwrap();

        // A full queue must never prevent Stop/Quit from joining the worker.
        tx.send(Update {
            id: 3,
            reply: Reply::Progress {
                message: "Earlier phase".into(),
                completed: 0,
                total: None,
            },
        })
        .unwrap();
        let stop = cancelled.clone();
        let task = thread::spawn(move || {
            discovery_update(&tx, 4, "Checking crawl rules…", 0, None, &cancelled)
        });
        stop.store(true, Ordering::Release);
        task.join().unwrap();
        assert_eq!(rx.recv().unwrap().id, 3);
        assert!(rx.try_recv().is_err());
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
