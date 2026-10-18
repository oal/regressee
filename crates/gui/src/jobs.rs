// SPDX-License-Identifier: MPL-2.0
//! Native client facade. Production never opens Store or starts browser workers.
pub use regressee_app::jobs::{COMMAND_CAPACITY, StorageFailure, UPDATE_CAPACITY};
use regressee_app::{
    engine::{Client, Launcher},
    jobs,
    protocol::{self, Operation, Outcome, Response},
};
use regressee_core::domain::{ProjectId, RunId, TargetKey};
use regressee_storage::AppPaths;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
    },
    thread,
    time::Duration,
};
use uuid::Uuid;

pub type Work = jobs::Command<ViewerWork>;
pub type Reply = jobs::JobReply<Box<ViewerFrame>>;
pub type Request = jobs::JobRequest<ViewerWork>;
pub type Update = jobs::JobUpdate<Box<ViewerFrame>>;
#[path = "dom_jobs.rs"]
mod dom;
pub use dom::{DomPresentation, DomViewerFrame, DomViewerIdentity, DomViewerUpdate};
#[path = "workspace_jobs.rs"]
mod workspace;
pub use workspace::EvidenceReadFailures;
#[derive(Debug)]
pub enum ViewerWork {
    ExactViewer(protocol::ViewerSelection),
    Viewer {
        project: ProjectId,
        tile: usize,
        key: Option<TargetKey>,
        analysis: Option<regressee_core::domain::AnalysisId>,
    },
    HistoricalViewer {
        project: ProjectId,
        run: RunId,
        tile: usize,
        key: Option<TargetKey>,
    },
}
pub use protocol::ViewerIdentity;
#[derive(Debug, Clone)]
pub struct ViewerFrame {
    pub tile: usize,
    pub panes: Vec<Vec<Arc<gpui::RenderImage>>>,
    pub width: u32,
    pub height: u32,
    pub request: Option<regressee_app::analysis_engine::AnalysisRequest>,
    pub analysis: Option<regressee_core::domain::AnalysisId>,
    pub identity: Option<ViewerIdentity>,
    pub before_environment: Option<String>,
    pub before_capture: Option<regressee_core::evidence::CaptureEvidence>,
}
impl ViewerWork {
    fn selection(self) -> protocol::ViewerSelection {
        match self {
            Self::ExactViewer(selection) => selection,
            Self::Viewer {
                project,
                tile,
                key,
                analysis,
            } => protocol::ViewerSelection {
                project,
                run: None,
                tile,
                key,
                analysis,
            },
            Self::HistoricalViewer {
                project,
                run,
                tile,
                key,
            } => protocol::ViewerSelection {
                project,
                run: Some(run),
                tile,
                key,
                analysis: None,
            },
        }
    }
}
enum Backend {
    Remote(Arc<Remote>),
    #[cfg(any(test, feature = "test-support"))]
    Local(Arc<crate::local_jobs::Jobs>),
}
pub enum Appearance {
    Remote(regressee_app::appearance::AppearanceSettings),
    #[cfg(any(test, feature = "test-support"))]
    Local(Arc<crate::local_jobs::Jobs>),
}
impl Appearance {
    fn inner(&self) -> &regressee_app::appearance::AppearanceSettings {
        match self {
            Self::Remote(p) => p,
            #[cfg(any(test, feature = "test-support"))]
            Self::Local(p) => &p.appearance,
        }
    }
    pub fn snapshot(&self) -> regressee_app::appearance::Snapshot {
        self.inner().snapshot()
    }
    pub fn wait_loaded(&self) -> regressee_app::appearance::Snapshot {
        self.inner().wait_loaded()
    }
    pub fn choose(&self, value: regressee_core::appearance::AppAppearance) {
        self.inner().choose(value)
    }
}
pub struct Jobs {
    pub appearance: Appearance,
    backend: Backend,
    dom_events: Mutex<Receiver<DomViewerUpdate>>,
}
struct Remote {
    evidence_failures: Mutex<BTreeMap<u64, EvidenceReadFailures>>,
    dom: dom::State,
    client: Client,
    stopped: AtomicBool,
    active: Mutex<BTreeSet<Uuid>>,
    workers: Mutex<Vec<thread::JoinHandle<()>>>,
    launcher: Option<Launcher>,
    viewer: AtomicU64,
    changed: AtomicBool,
    slots: std::sync::atomic::AtomicUsize,
    updates: SyncSender<Update>,
    tiles: Mutex<
        VecDeque<(
            regressee_core::domain::ArtifactId,
            Arc<gpui::RenderImage>,
            usize,
        )>,
    >,
    intents: Mutex<ViewIntents>,
    cancel_epoch: AtomicU64,
    pause_epoch: AtomicU64,
    project_refresh: Mutex<Option<(ProjectId, u64)>>,
    saved_projects: Mutex<BTreeMap<ProjectId, regressee_storage::ProjectRecord>>,
    agent: Mutex<AgentState>,
    agent_command: Mutex<Option<AgentIntent>>,
    data_revision: AtomicU64,
}
type AgentIntent = (
    regressee_app::control::Command,
    Option<regressee_app::control::Credential>,
    Option<u64>,
);
#[derive(Clone, Default)]
pub struct AgentState {
    pub control: Option<regressee_app::control::Snapshot>,
    pub frontend: Option<Uuid>,
    pub pending: bool,
    pub completion: u64,
    pub error: Option<String>,
    pub connection_error: bool,
    pub selection_unavailable: bool,
    pub data_revision: u64,
    pub location: Option<(
        Uuid,
        u64,
        Uuid,
        u64,
        u64,
        Arc<regressee_app::control::Location>,
    )>,
    pub external: Vec<ExternalJob>,
}
#[derive(Clone)]
pub struct ExternalJob {
    pub id: Uuid,
    pub sequence: u64,
    pub state: protocol::JobState,
    pub label: String,
    pub progress: Option<Outcome>,
    pub capture: Option<regressee_app::task::CaptureStatus>,
}
impl AgentState {
    /// A cheap presentation revision check. Large immutable evidence is shared,
    /// not cloned and redrawn on every idle control-heartbeat tick.
    pub fn same_revision(&self, other: &Self) -> bool {
        self.control == other.control
            && self.frontend == other.frontend
            && self.pending == other.pending
            && self.completion == other.completion
            && self.error == other.error
            && self.connection_error == other.connection_error
            && self.selection_unavailable == other.selection_unavailable
            && self.data_revision == other.data_revision
            && self.location.as_ref().map(|v| (v.0, v.1, v.2, v.3, v.4))
                == other.location.as_ref().map(|v| (v.0, v.1, v.2, v.3, v.4))
            && self
                .external
                .iter()
                .map(|j| (j.id, j.sequence, j.state))
                .eq(other.external.iter().map(|j| (j.id, j.sequence, j.state)))
    }
}
#[derive(Default)]
struct ViewIntents {
    latest: Option<(u64, Request)>,
    prefetch: Option<Request>,
}
impl Jobs {
    pub fn start(paths: AppPaths) -> anyhow::Result<(Arc<Self>, Receiver<Update>)> {
        // A source tree or partial upgrade can contain an older CLI sibling.
        // Host with this known matching executable, before graphics startup.
        let launcher = Launcher::current()?;
        let client = Client::autostart(paths, launcher.clone())?;
        Ok(Self::attach(client, Some(launcher)))
    }
    /// Attach to an already negotiated engine. This never opens a Store.
    pub fn connect(client: Client) -> (Arc<Self>, Receiver<Update>) {
        Self::attach(client, None)
    }
    fn attach(client: Client, launcher: Option<Launcher>) -> (Arc<Self>, Receiver<Update>) {
        let client = client.uncredentialed();
        let appearance = Appearance::Remote(
            regressee_app::appearance::AppearanceSettings::start_client(client.clone()),
        );
        let (updates, events) = mpsc::sync_channel(UPDATE_CAPACITY);
        let (dom, dom_events) = dom::State::new();
        let remote = Arc::new(Remote {
            evidence_failures: Mutex::new(BTreeMap::new()),
            dom,
            client,
            launcher,
            stopped: AtomicBool::new(false),
            active: Mutex::new(BTreeSet::new()),
            workers: Mutex::new(vec![]),
            viewer: AtomicU64::new(0),
            changed: AtomicBool::new(false),
            slots: std::sync::atomic::AtomicUsize::new(0),
            updates,
            tiles: Mutex::new(VecDeque::new()),
            intents: Mutex::new(ViewIntents::default()),
            cancel_epoch: AtomicU64::new(0),
            pause_epoch: AtomicU64::new(0),
            project_refresh: Mutex::new(None),
            saved_projects: Mutex::new(BTreeMap::new()),
            agent: Mutex::new(AgentState::default()),
            agent_command: Mutex::new(None),
            data_revision: AtomicU64::new(0),
        });
        let control_watch = Arc::downgrade(&remote);
        let control_thread = thread::spawn(move || {
            while let Some(r) = control_watch.upgrade() {
                if r.stopped.load(Ordering::Acquire) {
                    let _ = r.client.control(regressee_app::control::Command::Detach);
                    break;
                }
                r.poll_agent();
                drop(r);
                thread::sleep(Duration::from_millis(100));
            }
        });
        remote.workers.lock().unwrap().push(control_thread);
        let command_watch = Arc::downgrade(&remote);
        let command_thread = thread::spawn(move || {
            while let Some(r) = command_watch.upgrade() {
                if r.stopped.load(Ordering::Acquire) {
                    break;
                }
                let command = r.agent_command.lock().unwrap().take();
                if let Some((command, credential, epoch)) = command {
                    let result = (|| -> anyhow::Result<Response> {
                        let credential = credential
                            .ok_or_else(|| anyhow::anyhow!("GUI frontend is not connected"))?;
                        let response =
                            if matches!(command, regressee_app::control::Command::Takeover) {
                                // Takeover is not queued behind saved-data/location
                                // reads or large job receipts. Never substitute a new
                                // incarnation's frontend credential for this intent.
                                r.client.with_credential(credential).control(command)?
                            } else {
                                r.client.uncredentialed().request(
                                    protocol::Request::Authorized {
                                        credential,
                                        epoch: epoch.ok_or_else(|| {
                                            anyhow::anyhow!("Control snapshot unavailable")
                                        })?,
                                        request: Box::new(protocol::Request::Control(command)),
                                    },
                                )?
                            };
                        match response {
                            Response::Error { code, message } => anyhow::bail!("{code}: {message}"),
                            response => Ok(response),
                        }
                    })();
                    let mut a = r.agent.lock().unwrap();
                    a.pending = false;
                    a.completion += 1;
                    a.connection_error = false;
                    match result {
                        Ok(Response::Control(control)) => {
                            a.control = Some(control);
                            a.error = None;
                        }
                        Ok(_) => {}
                        Err(e) => a.error = Some(e.to_string()),
                    }
                }
                drop(r);
                thread::sleep(Duration::from_millis(10));
            }
        });
        remote.workers.lock().unwrap().push(command_thread);
        let location_watch = Arc::downgrade(&remote);
        let location_thread = thread::spawn(move || {
            let mut loaded = None;
            while let Some(r) = location_watch.upgrade() {
                if r.stopped.load(Ordering::Acquire) {
                    break;
                }
                let wanted = {
                    let a = r.agent.lock().unwrap();
                    a.control.as_ref().and_then(|c| {
                        c.sessions
                            .iter()
                            .find(|s| {
                                Some(s.id) == c.controller
                                    && s.follow == a.frontend
                                    && s.status == regressee_app::control::Status::Active
                                    && s.selection.is_some()
                            })
                            .map(|s| {
                                (
                                    c.engine,
                                    c.epoch,
                                    s.id,
                                    s.selection_revision,
                                    r.data_revision.load(Ordering::Acquire),
                                )
                            })
                    })
                };
                if let Some(wanted) = wanted
                    && loaded != Some(wanted)
                {
                    loaded = Some(wanted);
                    if let Ok(job) = r.query(Operation::AgentLocation { session: wanted.2 }) {
                        let _ = r
                            .client
                            .request(protocol::Request::Acknowledge { id: job.id });
                        let mut a = r.agent.lock().unwrap();
                        let current = a.control.as_ref().is_some_and(|c| {
                            c.engine == wanted.0
                                && c.epoch == wanted.1
                                && c.controller == Some(wanted.2)
                                && c.sessions.iter().any(|s| {
                                    s.id == wanted.2
                                        && s.follow == a.frontend
                                        && s.selection_revision == wanted.3
                                        && s.status == regressee_app::control::Status::Active
                                })
                        });
                        if current {
                            match job.outcome {
                                Some(Outcome::AgentLocation(location)) => {
                                    a.location = Some((
                                        wanted.0,
                                        wanted.1,
                                        wanted.2,
                                        wanted.3,
                                        wanted.4,
                                        Arc::new(*location),
                                    ));
                                    a.selection_unavailable = false;
                                }
                                _ => {
                                    a.location = None;
                                    a.selection_unavailable = true;
                                }
                            }
                        }
                    }
                }
                drop(r);
                thread::sleep(Duration::from_millis(50));
            }
        });
        remote.workers.lock().unwrap().push(location_thread);
        let observer = Arc::downgrade(&remote);
        let watch = thread::spawn(move || {
            let mut cursor = None;
            let mut revisions = BTreeMap::new();
            let mut snapshot_needed = true;
            while let Some(remote) = observer.upgrade() {
                if remote.stopped.load(Ordering::Acquire) {
                    break;
                }
                if let Some((engine, after)) = cursor
                    && let Ok(Response::Events {
                        sequence,
                        events,
                        resync,
                    }) = remote
                        .client
                        .request(protocol::Request::Events { engine, after })
                {
                    if resync {
                        remote.data_revision.fetch_add(1, Ordering::AcqRel);
                        cursor = None;
                        snapshot_needed = true;
                        remote.changed.store(true, Ordering::Release);
                    } else {
                        cursor = Some((engine, sequence));
                        if events.iter().any(|e| e.changed) {
                            remote.data_revision.fetch_add(1, Ordering::AcqRel);
                            snapshot_needed = true;
                            remote.changed.store(true, Ordering::Release);
                        }
                    }
                }
                if (cursor.is_none() || snapshot_needed)
                    && let Ok(snapshot) = remote.client.snapshot()
                {
                    cursor = Some((snapshot.engine, snapshot.sequence));
                    revisions = snapshot.project_revisions.into_iter().collect();
                    snapshot_needed = false;
                    remote.changed.store(true, Ordering::Release);
                }
                let wanted = *remote.project_refresh.lock().unwrap();
                if let Some((id, revision)) = wanted
                    && revisions.get(&id).is_some_and(|r| *r > revision)
                    && !remote
                        .saved_projects
                        .lock()
                        .unwrap()
                        .get(&id)
                        .is_some_and(|r| r.revision >= revisions[&id])
                {
                    // Separate read job, never a live database open. The snapshot
                    // cursor predates the read so changes racing it remain visible.
                    if let Ok(job) = remote.query(Operation::Load(id)) {
                        if let Some(Outcome::Project(record)) = job.outcome {
                            let mut records = remote.saved_projects.lock().unwrap();
                            records.clear();
                            records.insert(id, *record);
                        }
                        let _ = remote
                            .client
                            .request(protocol::Request::Acknowledge { id: job.id });
                    }
                }
                drop(remote);
                thread::sleep(Duration::from_millis(100));
            }
        });
        remote.workers.lock().unwrap().push(watch);
        let loader = Arc::downgrade(&remote);
        let viewer_thread = thread::spawn(move || {
            while let Some(remote) = loader.upgrade() {
                if remote.stopped.load(Ordering::Acquire) {
                    break;
                }
                let request = {
                    let mut intents = remote.intents.lock().unwrap();
                    if remote.slots.load(Ordering::Acquire) != 0 {
                        None
                    } else if let Some((g, r)) = intents.latest.take() {
                        Some((r, false, Some(g)))
                    } else {
                        intents.prefetch.take().map(|r| (r, true, None))
                    }
                };
                if let Some((request, silent, generation)) = request {
                    remote.slots.fetch_add(1, Ordering::AcqRel);
                    let epochs = (
                        remote.cancel_epoch.load(Ordering::Acquire),
                        remote.pause_epoch.load(Ordering::Acquire),
                    );
                    remote.perform(request, silent, generation, epochs, None);
                } else {
                    drop(remote);
                    thread::sleep(Duration::from_millis(10));
                }
            }
        });
        remote.workers.lock().unwrap().push(viewer_thread);
        remote
            .workers
            .lock()
            .unwrap()
            .push(dom::start(Arc::downgrade(&remote)));
        (
            Arc::new(Self {
                appearance,
                backend: Backend::Remote(remote),
                dom_events: Mutex::new(dom_events),
            }),
            events,
        )
    }
    pub fn take_data_changed(&self) -> bool {
        match &self.backend {
            Backend::Remote(r) => r.changed.swap(false, Ordering::AcqRel),
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(_) => false,
        }
    }
    pub fn submit_dom_viewer(
        &self,
        id: u64,
        identity: DomViewerIdentity,
    ) -> Result<(), &'static str> {
        match &self.backend {
            Backend::Remote(remote) if !remote.stopped.load(Ordering::Acquire) => {
                remote.dom.submit(id, identity);
                Ok(())
            }
            _ => Err("DOM source reads require a connected shared engine."),
        }
    }
    pub fn cancel_dom_viewer(&self) {
        match &self.backend {
            Backend::Remote(remote) => remote.dom.cancel(),
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(_) => (),
        }
    }
    pub fn take_dom_viewer(&self) -> Option<DomViewerUpdate> {
        self.dom_events.lock().unwrap().try_recv().ok()
    }
    pub fn take_evidence_failures(&self, id: u64) -> Option<EvidenceReadFailures> {
        match &self.backend {
            Backend::Remote(remote) => remote.evidence_failures.lock().unwrap().remove(&id),
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(_) => None,
        }
    }
    /// Passive saved-data refresh has its own identity/reply lane. It must not
    /// borrow a GUI mutation's pending/busy/task state or select the newest run.
    pub fn refresh_evidence(
        &self,
        id: u64,
        selection: regressee_app::control::Selection,
    ) -> Result<(), &'static str> {
        self.passive_read(id, Operation::SelectedEvidence(selection))
    }
    pub fn refresh_history(
        &self,
        id: u64,
        project: ProjectId,
        page: usize,
    ) -> Result<(), &'static str> {
        self.passive_read(id, Operation::History { project, page })
    }
    fn passive_read(&self, id: u64, operation: Operation) -> Result<(), &'static str> {
        let remote = match &self.backend {
            Backend::Remote(remote) => remote,
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(_) => return Err("Saved data refresh requires the shared engine."),
        };
        if remote.stopped.load(Ordering::Acquire) {
            return Err("Engine client has disconnected.");
        }
        if remote.slots.fetch_add(1, Ordering::AcqRel) >= 8 {
            remote.slots.fetch_sub(1, Ordering::AcqRel);
            return Err("Engine client is busy; saved evidence will refresh later.");
        }
        let remote = remote.clone();
        let worker_remote = remote.clone();
        let worker = thread::spawn(move || {
            let result = (|| -> anyhow::Result<Reply> {
                let outcome = if let Operation::SelectedEvidence(selection) = operation {
                    worker_remote.recover_workspace(
                        id,
                        selection.project,
                        selection.run,
                        Some(selection),
                    )?
                } else {
                    let job = worker_remote.query(operation)?;
                    let _ = worker_remote
                        .client
                        .request(protocol::Request::Acknowledge { id: job.id });
                    job.outcome
                        .ok_or_else(|| anyhow::anyhow!("Saved data read has no outcome"))?
                };
                outcome.into_reply(|_| anyhow::bail!("Saved data is not a pixel frame"))
            })();
            let mut update = Update {
                id,
                reply: result.unwrap_or_else(|e| Reply::ProductFailure(e.to_string())),
            };
            while !worker_remote.stopped.load(Ordering::Acquire) {
                match worker_remote.updates.try_send(update) {
                    Ok(()) => break,
                    Err(TrySendError::Full(value)) => {
                        update = value;
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(TrySendError::Disconnected(_)) => break,
                }
            }
            worker_remote.slots.fetch_sub(1, Ordering::AcqRel);
        });
        let mut workers = remote.workers.lock().unwrap();
        workers.retain(|w| !w.is_finished());
        workers.push(worker);
        Ok(())
    }
    pub fn agent_state(&self) -> AgentState {
        match &self.backend {
            Backend::Remote(r) => r.agent.lock().unwrap().clone(),
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(_) => AgentState::default(),
        }
    }
    pub fn agent_command(&self, command: regressee_app::control::Command) {
        match &self.backend {
            Backend::Remote(r) => {
                let mut pending = r.agent_command.lock().unwrap();
                if pending.is_none() || matches!(command, regressee_app::control::Command::Takeover)
                {
                    let mut a = r.agent.lock().unwrap();
                    *pending = Some((
                        command,
                        r.client.credential(),
                        a.control.as_ref().map(|c| c.epoch),
                    ));
                    a.pending = true;
                }
            }
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(_) => (),
        }
    }
    pub fn saved_project(
        &self,
        project: ProjectId,
        revision: u64,
    ) -> Option<regressee_storage::ProjectRecord> {
        match &self.backend {
            Backend::Remote(r) => {
                *r.project_refresh.lock().unwrap() = Some((project, revision));
                r.saved_projects
                    .lock()
                    .unwrap()
                    .get(&project)
                    .filter(|p| p.revision > revision)
                    .cloned()
            }
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(_) => None,
        }
    }
    pub fn is_remote(&self) -> bool {
        matches!(&self.backend, Backend::Remote(_))
    }
    pub fn submit(&self, request: Request) -> Result<(), &'static str> {
        match &self.backend {
            Backend::Remote(r) => r.submit(request, false),
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(j) => j.submit(request),
        }
    }
    pub fn submit_viewer(&self, request: Request) -> Result<(), &'static str> {
        if !matches!(request.work, Work::Extension(_)) {
            return Err("Viewer navigation only accepts screenshot reads.");
        }
        self.submit(request)
    }
    pub fn prefetch_viewer(&self, work: Work) -> Result<(), &'static str> {
        if !matches!(work, Work::Extension(ViewerWork::Viewer { .. })) {
            return Err("Viewer prefetch only accepts screenshot reads.");
        }
        match &self.backend {
            Backend::Remote(r) => r.submit(Request { id: 0, work }, true),
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(j) => j.prefetch_viewer(work),
        }
    }
    fn control(&self, pause: bool) {
        match &self.backend {
            Backend::Remote(r) => {
                if pause {
                    r.pause_epoch.fetch_add(1, Ordering::AcqRel);
                } else {
                    r.cancel_epoch.fetch_add(1, Ordering::AcqRel);
                }
                let remote = r.clone();
                thread::spawn(move || {
                    let active = remote.active.lock().unwrap().clone();
                    for id in active {
                        let _ = remote.client.request(if pause {
                            protocol::Request::Pause { id }
                        } else {
                            protocol::Request::Cancel { id }
                        });
                    }
                });
            }
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(j) => {
                if pause {
                    j.pause()
                } else {
                    j.cancel()
                }
            }
        }
    }
    pub fn cancel(&self) {
        self.control(false)
    }
    pub fn pause(&self) {
        self.control(true)
    }
    pub fn stop(&self) {
        match &self.backend {
            Backend::Remote(r) => {
                r.stopped.store(true, Ordering::Release);
            }
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(j) => j.stop(),
        };
        self.appearance.inner().stop();
    }
    pub fn join_after_ui(&self) {
        self.stop();
        self.appearance.inner().join_after_ui();
        match &self.backend {
            Backend::Remote(r) => {
                let threads = std::mem::take(&mut *r.workers.lock().unwrap());
                for t in threads {
                    let _ = t.join();
                }
            }
            #[cfg(any(test, feature = "test-support"))]
            Backend::Local(j) => j.join_after_ui(),
        }
    }
    #[cfg(any(test, feature = "test-support"))]
    fn local(
        pair: (Arc<crate::local_jobs::Jobs>, Receiver<Update>),
    ) -> (Arc<Self>, Receiver<Update>) {
        let (jobs, events) = pair;
        let (_, dom_events) = dom::State::new();
        (
            Arc::new(Self {
                appearance: Appearance::Local(jobs.clone()),
                backend: Backend::Local(jobs),
                dom_events: Mutex::new(dom_events),
            }),
            events,
        )
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn start_test_configured(paths: AppPaths) -> (Arc<Self>, Receiver<Update>) {
        Self::local(crate::local_jobs::Jobs::start(paths))
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn start_without_runtime(paths: AppPaths) -> (Arc<Self>, Receiver<Update>) {
        Self::local(crate::local_jobs::Jobs::start_without_runtime(paths))
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn start_with_product(
        paths: AppPaths,
        product: regressee_app::product::Product,
    ) -> (Arc<Self>, Receiver<Update>) {
        Self::local(crate::local_jobs::Jobs::start_with_product(paths, product))
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn start_with_distribution(
        paths: AppPaths,
        distribution: regressee_runtime::distribution::Distribution,
    ) -> (Arc<Self>, Receiver<Update>) {
        Self::local(crate::local_jobs::Jobs::start_with_distribution(
            paths,
            distribution,
        ))
    }
}
impl Drop for Jobs {
    fn drop(&mut self) {
        self.stop();
    }
}
impl Remote {
    fn poll_agent(&self) {
        use regressee_app::control::{Command, Owner};
        let result = (|| -> anyhow::Result<_> {
            let status = self.client.uncredentialed().snapshot()?;
            if self
                .client
                .credential()
                .is_none_or(|c| c.engine != status.engine)
            {
                let credential = self.client.uncredentialed().register_frontend()?;
                self.client.set_credential(Some(credential));
            }
            self.client.control(Command::Heartbeat)?;
            let Response::Control(control) = self.client.control(Command::List)? else {
                anyhow::bail!("Control status unavailable")
            };
            let frontend = self.client.credential().and_then(|c| {
                if let Owner::Frontend(id) = c.owner {
                    Some(id)
                } else {
                    None
                }
            });
            let mut ids = status
                .jobs
                .iter()
                .filter(|j| !j.state.terminal())
                .map(|j| j.id)
                .take(8)
                .collect::<Vec<_>>();
            ids.extend(self.agent.lock().unwrap().external.iter().map(|j| j.id));
            ids.sort();
            ids.dedup();
            let own = self.active.lock().unwrap().clone();
            let mut external = ids
                .into_iter()
                .filter(|id| !own.contains(id))
                .filter_map(|id| self.client.uncredentialed().job(id).ok())
                .filter(|j| {
                    j.submitted.operation.mutates()
                        && !matches!(j.submitted.operation, Operation::SelectAgent(_))
                })
                .collect::<Vec<_>>();
            external.sort_by_key(|j| (j.state.terminal(), std::cmp::Reverse(j.sequence)));
            external.truncate(8);
            let external = external
                .into_iter()
                .map(|j| ExternalJob {
                    id: j.id,
                    sequence: j.sequence,
                    state: j.state,
                    label: match j.submitted.operation {
                        Operation::Discover(_) => "Finding pages",
                        Operation::Capture { check: true, .. }
                        | Operation::CaptureInventory { check: true, .. } => "Checking screenshots",
                        Operation::Capture { .. }
                        | Operation::CaptureInventory { .. }
                        | Operation::PreviewImages { .. } => "Capturing screenshots",
                        Operation::Reanalyze { .. } => "Analyzing saved screenshots",
                        Operation::SetupRuntime => "Setting up the browser",
                        _ => "External operation",
                    }
                    .into(),
                    progress: j.progress,
                    capture: j.capture,
                })
                .collect();
            Ok((control, frontend, external))
        })();
        let mut a = self.agent.lock().unwrap();
        match result {
            Ok((control, frontend, external)) => {
                if a.connection_error {
                    a.error = None;
                    a.connection_error = false;
                }
                if a.control
                    .as_ref()
                    .is_none_or(|old| old.engine != control.engine || old.epoch <= control.epoch)
                {
                    a.control = Some(control);
                }
                a.frontend = frontend;
                a.external = external;
                a.data_revision = self.data_revision.load(Ordering::Acquire);
            }
            Err(_) => {
                // Fail safe locally immediately; a detached frontend also loses
                // engine authority within its bounded heartbeat lease.
                a.control = None;
                a.location = None;
                a.pending = false;
                a.error=Some("Engine connection lost. GUI control is unlocked; background jobs were not cancelled.".into());
                a.connection_error = true;
                self.client.set_credential(None);
                drop(a);
                self.agent_command.lock().unwrap().take();
            }
        }
    }
    fn query(&self, operation: Operation) -> anyhow::Result<protocol::Job> {
        anyhow::ensure!(
            !operation.mutates(),
            "Client query requires a read-only operation"
        );
        let snapshot = self.client.snapshot()?;
        let mut job = self.client.submit(protocol::Submit {
            request: Uuid::new_v4(),
            engine: snapshot.engine,
            epoch: snapshot.epoch,
            operation,
            confirmation: None,
            expected_revision: None,
        })?;
        while !job.state.terminal() {
            anyhow::ensure!(
                !self.stopped.load(Ordering::Acquire),
                "Client detached; read job {} remains owned by the engine",
                job.id
            );
            thread::sleep(Duration::from_millis(20));
            job = self.client.job(job.id)?;
        }
        Ok(job)
    }
    fn prepare(&self, operation: Operation) -> anyhow::Result<protocol::Confirmation> {
        let job = self.query(Operation::Prepare(Box::new(operation)))?;
        match job.outcome {
            Some(Outcome::Confirmation(c)) => Ok(c),
            other => anyhow::bail!("Cannot prepare confirmation: {other:?}"),
        }
    }
    fn connected_snapshot(&self) -> anyhow::Result<protocol::Snapshot> {
        match self.client.snapshot() {
            Ok(s) => Ok(s),
            Err(error) => {
                if self.stopped.load(Ordering::Acquire) {
                    return Err(error);
                }
                let Some(launcher) = &self.launcher else {
                    return Err(error);
                };
                Client::autostart_while(self.client.paths.clone(), launcher.clone(), || {
                    !self.stopped.load(Ordering::Acquire)
                })?
                .snapshot()
            }
        }
    }
    fn submit(self: &Arc<Self>, request: Request, silent: bool) -> Result<(), &'static str> {
        if self.stopped.load(Ordering::Acquire) {
            return Err("Engine client has disconnected.");
        }
        if matches!(request.work, Work::Extension(_)) {
            let mut intents = self.intents.lock().unwrap();
            if self.stopped.load(Ordering::Acquire) {
                return Err("Engine client has disconnected.");
            }
            if silent {
                intents.prefetch = Some(request);
            } else {
                let generation = self.viewer.fetch_add(1, Ordering::AcqRel) + 1;
                intents.latest = Some((generation, request));
                intents.prefetch = None;
            }
            return Ok(());
        }
        if self.slots.fetch_add(1, Ordering::AcqRel) >= 8 {
            self.slots.fetch_sub(1, Ordering::AcqRel);
            return Err("Engine client is busy; retry after a pending request finishes.");
        }
        let epochs = (
            self.cancel_epoch.load(Ordering::Acquire),
            self.pause_epoch.load(Ordering::Acquire),
        );
        let remote = self.clone();
        let authority = self
            .agent
            .lock()
            .unwrap()
            .control
            .as_ref()
            .map(|c| (c.engine, c.epoch));
        let worker =
            thread::spawn(move || remote.perform(request, silent, None, epochs, authority));
        let mut workers = self.workers.lock().unwrap();
        workers.retain(|w| !w.is_finished());
        workers.push(worker);
        Ok(())
    }
    fn perform(
        self: Arc<Self>,
        request: Request,
        silent: bool,
        generation: Option<u64>,
        epochs: (u64, u64),
        authority: Option<(Uuid, u64)>,
    ) {
        let remote = self;
        let id = request.id;
        let workspace = match &request.work {
            Work::Evidence(project) => Some((*project, None)),
            Work::InspectRun { project, run } => Some((*project, Some(*run))),
            _ => None,
        };
        let cancel = if let Work::Reanalyze { cancel, .. } = &request.work {
            Some(cancel.clone())
        } else {
            None
        };
        let operation = match Operation::from_work(request.work) {
            Ok(op) => op,
            Err(viewer) => Operation::Viewer(viewer.selection()),
        };
        let result = (|| -> anyhow::Result<Outcome> {
            let snapshot = remote.connected_snapshot()?;
            anyhow::ensure!(
                !operation.requires_authority()
                    || authority.is_none_or(|a| a == (snapshot.engine, snapshot.epoch)),
                "stale_epoch: GUI intent predates the current control authority; act again deliberately"
            );
            if remote.cancel_epoch.load(Ordering::Acquire) != epochs.0
                || cancel.as_ref().is_some_and(|c| c.load(Ordering::Acquire))
            {
                return Ok(Outcome::Cancelled);
            }
            let confirmation = if operation.needs_confirmation() {
                Some(remote.prepare(operation.clone())?.id)
            } else {
                None
            };
            if remote.cancel_epoch.load(Ordering::Acquire) != epochs.0 {
                return Ok(Outcome::Cancelled);
            }
            let job_id = Uuid::new_v4();
            let submitted = protocol::Submit {
                request: job_id,
                engine: snapshot.engine,
                epoch: snapshot.epoch,
                operation,
                confirmation,
                expected_revision: None,
            };
            // Retried admissions retain the exact UUID/input. Never manufacture
            // another mutation ID after a lost acknowledgement.
            let mut admitted = None;
            for _ in 0..3 {
                match remote.client.submit(submitted.clone()) {
                    Ok(j) => {
                        admitted = Some(j);
                        break;
                    }
                    Err(e) => {
                        if e.is::<regressee_app::engine::Rejected>() {
                            return Err(e);
                        }
                        if remote.stopped.load(Ordering::Acquire) {
                            return Err(e);
                        }
                        thread::sleep(Duration::from_millis(30));
                    }
                }
            }
            let mut job=admitted.ok_or_else(||anyhow::anyhow!("Engine acknowledgement unavailable; request {job_id} may be admitted. Inspect that job before retrying."))?;
            remote.active.lock().unwrap().insert(job_id);
            let mut sequence = 0;
            let mut capture_sent = None;
            let mut superseded = false;
            if remote.pause_epoch.load(Ordering::Acquire) != epochs.1 {
                let _ = remote
                    .client
                    .request(protocol::Request::Pause { id: job_id });
            }
            loop {
                if remote.stopped.load(Ordering::Acquire) {
                    remote.active.lock().unwrap().remove(&job_id);
                    anyhow::bail!("Client detached; engine work continues as job {job_id}");
                }
                if remote.cancel_epoch.load(Ordering::Acquire) != epochs.0
                    || cancel.as_ref().is_some_and(|c| c.load(Ordering::Acquire))
                {
                    let _ = remote
                        .client
                        .request(protocol::Request::Cancel { id: job_id });
                }
                if !superseded
                    && generation.is_some_and(|g| remote.viewer.load(Ordering::Acquire) != g)
                {
                    let _ = remote
                        .client
                        .request(protocol::Request::Cancel { id: job_id });
                    superseded = true;
                }
                if job.sequence != sequence {
                    sequence = job.sequence;
                    if let Some(progress) = job.progress.clone()
                        && !silent
                    {
                        let _ = remote.updates.try_send(Update {
                            id,
                            reply: progress.into_reply(|f| remote.decode_viewer(f))?,
                        });
                    }
                }
                if !silent && let Some(capture) = &job.capture {
                    let encoded = serde_json::to_vec(capture)?;
                    if capture_sent.as_ref() != Some(&encoded)
                        && remote
                            .updates
                            .try_send(Update {
                                id,
                                reply: Reply::CaptureStatus(capture.clone()),
                            })
                            .is_ok()
                    {
                        capture_sent = Some(encoded);
                    }
                }
                if job.state.terminal() {
                    break;
                }
                thread::sleep(Duration::from_millis(20));
                job = match remote.client.job(job_id) {
                    Ok(job) => job,
                    Err(error) => {
                        if remote.stopped.load(Ordering::Acquire) {
                            return Err(error);
                        }
                        remote.connected_snapshot()?;
                        remote.client.job(job_id).map_err(|e|anyhow::anyhow!("Job {job_id} remains the retry identity; completion may be unknown: {e}"))?
                    }
                };
            }
            remote.active.lock().unwrap().remove(&job_id);
            if !job.submitted.operation.mutates() {
                let _ = remote
                    .client
                    .request(protocol::Request::Acknowledge { id: job_id });
            }
            job.outcome
                .ok_or_else(|| anyhow::anyhow!("Engine job has no terminal outcome"))
        })();
        let result = if !result.as_ref().is_ok_and(|outcome| !outcome.failed()) {
            if let Some((project, exact)) = workspace {
                remote
                    .recover_workspace(id, project, exact, None)
                    .map(|outcome| match (exact, outcome) {
                        (Some(run), Outcome::Evidence(evidence)) => Outcome::InspectedRun {
                            project,
                            run,
                            evidence,
                        },
                        (_, outcome) => outcome,
                    })
            } else {
                result
            }
        } else {
            result
        };
        let reply = result
            .and_then(|outcome| outcome.into_reply(|f| remote.decode_viewer(f)))
            .unwrap_or_else(|e| Reply::ProductFailure(e.to_string()));
        if !silent {
            let mut update = Update { id, reply };
            while !remote.stopped.load(Ordering::Acquire)
                && generation.is_none_or(|g| remote.viewer.load(Ordering::Acquire) == g)
            {
                let guard = remote.intents.lock().unwrap();
                if generation.is_some_and(|g| remote.viewer.load(Ordering::Acquire) != g) {
                    break;
                }
                let result = remote.updates.try_send(update);
                drop(guard);
                match result {
                    Ok(()) => break,
                    Err(TrySendError::Full(value)) => {
                        update = value;
                        thread::sleep(Duration::from_millis(10));
                    }
                    Err(TrySendError::Disconnected(_)) => break,
                }
            }
        }
        remote.slots.fetch_sub(1, Ordering::AcqRel);
    }
}
impl Remote {
    fn decode_viewer(&self, frame: Box<protocol::ViewerFrame>) -> anyhow::Result<Box<ViewerFrame>> {
        let mut panes = Vec::new();
        let mut cache = self.tiles.lock().unwrap();
        for pane in frame.panes {
            let mut images = Vec::new();
            for artifact in pane {
                let image = if let Some(index) = cache
                    .iter()
                    .position(|(id, _, _)| *id == artifact.artifact.id)
                {
                    let entry = cache.remove(index).unwrap();
                    let image = entry.1.clone();
                    cache.push_back(entry);
                    image
                } else {
                    let image = crate::viewer_images::from_artifact(&artifact)?;
                    let size = image.size(0);
                    let bytes = size.width.0 as usize * size.height.0 as usize * 4;
                    while !cache.is_empty()
                        && (cache.len() >= 256
                            || cache.iter().map(|e| e.2).sum::<usize>() + bytes > 64 * 1024 * 1024)
                    {
                        cache.pop_front();
                    }
                    if bytes <= 64 * 1024 * 1024 {
                        cache.push_back((artifact.artifact.id, image.clone(), bytes));
                    }
                    image
                };
                images.push(image);
            }
            panes.push(images);
        }
        Ok(Box::new(ViewerFrame {
            tile: frame.tile,
            panes,
            width: frame.width,
            height: frame.height,
            request: frame.request,
            analysis: frame.analysis,
            identity: Some(frame.identity),
            before_environment: frame.before_environment,
            before_capture: frame.before_capture,
        }))
    }
}
