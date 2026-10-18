// SPDX-License-Identifier: MPL-2.0
//! Local engine: a live broker plus one blocking product/storage owner.
mod authority;
mod client;
pub mod dom;
mod executor;
mod ipc;
pub mod ownership;
pub mod text;
mod viewer;
use crate::{product::Product, protocol::*};
use anyhow::{Context, Result, ensure};
pub use client::{Client, Launcher, Rejected, lifecycle};
use regressee_storage::{AppPaths, AppPreferences, Store};
use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread,
    time::Duration,
};
use uuid::Uuid;

struct Controls {
    cancel: Arc<AtomicBool>,
    pause: Arc<AtomicBool>,
}
struct State {
    engine: Uuid,
    epoch: u64,
    sequence: u64,
    jobs: BTreeMap<Uuid, Job>,
    events: VecDeque<Event>,
    controls: BTreeMap<Uuid, Controls>,
    confirmations: BTreeMap<Uuid, (Confirmation, Option<Uuid>)>,
    draining: bool,
    login_open: bool,
    failed: bool,
    revisions: BTreeMap<regressee_core::domain::ProjectId, u64>,
    event_bytes: usize,
    authority: authority::Authority,
}
struct Shared {
    state: Mutex<State>,
    directory: std::path::PathBuf,
    paths: AppPaths,
}
impl Shared {
    fn save(&self, job: &Job) -> Result<()> {
        let bytes = serde_json::to_vec(job)?.len() as u64;
        ensure!(
            bytes < (MAX_FRAME - 1024 * 1024) as u64,
            "Job result exceeds bounded response budget"
        );
        let path = self.directory.join(format!("job-{}.json", job.id));
        let old = std::fs::metadata(&path).map_or(0, |m| m.len());
        let total =
            std::fs::read_dir(&self.directory)?.try_fold(0u64, |sum, e| -> Result<u64> {
                let e = e?;
                Ok(sum
                    + if e.file_name().to_string_lossy().starts_with("job-") {
                        e.metadata()?.len()
                    } else {
                        0
                    })
            })?;
        ensure!(
            total.saturating_sub(old) + bytes <= 512 * 1024 * 1024,
            "Engine journal quota exceeded; receipts are not silently evicted"
        );
        ownership::write_json(&path, job)
    }
    fn event(state: &mut State, id: Uuid, changed: bool) {
        state.sequence += 1;
        let job = state.jobs.get_mut(&id).unwrap();
        job.sequence = state.sequence;
        let project = if changed {
            match &job.outcome {
                Some(Outcome::Project(p)) | Some(Outcome::Imported(p))
                    if serde_json::to_vec(p).is_ok_and(|b| b.len() <= 65536) =>
                {
                    Some((**p).clone())
                }
                _ => None,
            }
        } else {
            None
        };
        let event = Event {
            sequence: state.sequence,
            job: id,
            state: job.state,
            progress: job.progress.clone(),
            changed,
            project,
        };
        state.event_bytes += serde_json::to_vec(&event).map_or(0, |b| b.len());
        state.events.push_back(event);
        while state.events.len() > MAX_EVENTS || state.event_bytes > 8 * 1024 * 1024 {
            if let Some(event) = state.events.pop_front() {
                state.event_bytes = state
                    .event_bytes
                    .saturating_sub(serde_json::to_vec(&event).map_or(0, |b| b.len()));
            } else {
                break;
            }
        }
    }
    fn snapshot(state: &State) -> Snapshot {
        let mut jobs = state
            .jobs
            .values()
            .map(|j| JobInfo {
                id: j.id,
                state: j.state,
                sequence: j.sequence,
            })
            .collect::<Vec<_>>();
        jobs.sort_by_key(|j| (j.state.terminal(), std::cmp::Reverse(j.sequence)));
        jobs.truncate(MAX_EVENTS);
        Snapshot {
            engine: state.engine,
            epoch: state.epoch,
            sequence: state.sequence,
            jobs,
            total_jobs: state.jobs.len(),
            project_revisions: state.revisions.iter().map(|(p, r)| (*p, *r)).collect(),
            draining: state.draining,
            login_open: state.login_open,
        }
    }
    fn finish(&self, id: Uuid, mut outcome: Outcome, interrupted: bool) {
        outcome.without_dom_bodies();
        let mut state = self.state.lock().unwrap();
        let pure_read = !state.jobs[&id].submitted.operation.mutates()
            && !matches!(state.jobs[&id].submitted.operation, Operation::Prepare(_));
        if pure_read && text::json_size_within(&outcome, MAX_READ_RESULT_BYTES).is_none() {
            outcome = Outcome::ProductFailure("response_too_large: read response exceeds 16 MiB of encoded JSON; request bounded results/source/diff pages or focused target artifacts".into());
        }
        let changed = state.jobs[&id].submitted.operation.mutates()
            && !matches!(
                state.jobs[&id].submitted.operation,
                Operation::SelectAgent(_)
            );
        let job = state.jobs.get_mut(&id).unwrap();
        if let Some(message) = job.capture_result_error(&outcome) {
            outcome = Outcome::ProductFailure(message.into());
        }
        if let Outcome::Captured(evidence) = &outcome
            && let Some(run) = &evidence.latest
        {
            job.capture = Some(crate::task::CaptureStatus::from_run(&run.run));
        }
        job.state = if interrupted {
            JobState::Interrupted
        } else {
            job.terminal_state(&outcome)
        };
        job.outcome = Some(outcome);
        if self.save(job).is_err() {
            job.state = JobState::Interrupted;
            job.outcome=Some(Outcome::ProductFailure("Result durability failed; the operation may have committed. Inspect saved data before issuing a new request. This request will not be executed again.".into()));
            state.failed = true;
            state.draining = true;
        }
        state.controls.remove(&id);
        Self::event(&mut state, id, changed);
    }
    fn handle(
        &self,
        request: Request,
        queue: &mpsc::SyncSender<Uuid>,
        shutdown: &AtomicBool,
    ) -> Result<Response> {
        let mut s = self.state.lock().unwrap();
        expire_authority(&mut s);
        let (request, owner) = match request {
            Request::Authorized {
                credential,
                epoch,
                request,
            } => {
                let owner = s.authority.authenticate(&credential, s.engine)?;
                if !matches!(
                    &*request,
                    Request::Submit(_)
                        | Request::Status
                        | Request::Snapshot
                        | Request::Job { .. }
                        | Request::Jobs { .. }
                        | Request::Events { .. }
                        | Request::Preferences
                        | Request::Control(
                            crate::control::Command::List
                                | crate::control::Command::Show { .. }
                                | crate::control::Command::Renew { .. }
                                | crate::control::Command::Heartbeat
                        )
                ) {
                    ensure!(
                        epoch == s.epoch,
                        "stale_epoch: control request authority revoked"
                    );
                }
                (*request, Some(owner))
            }
            request => (request, None),
        };
        if matches!(
            request,
            Request::Stop
                | Request::AdvanceEpoch { .. }
                | Request::SetAppearance(_)
                | Request::Cancel { .. }
                | Request::Pause { .. }
        ) {
            s.authority.authorize(owner)?;
        }
        Ok(match request {
            Request::RegisterFrontend { capability } => {
                ensure!(
                    owner.is_none(),
                    "authority_conflict: frontend bootstrap cannot switch an authenticated role"
                );
                ensure!(
                    !s.draining && !s.failed,
                    "engine_draining: frontend registration refused"
                );
                let engine = s.engine;
                s.authority.register_frontend(&capability, engine)?
            }
            Request::Authorized { .. } => {
                Response::error("invalid_request", "Nested authority envelope refused")
            }
            Request::Control(command) => {
                ensure!(
                    !s.draining
                        || !matches!(
                            command,
                            crate::control::Command::Start { .. }
                                | crate::control::Command::Follow { .. }
                                | crate::control::Command::Return
                        ),
                    "engine_draining: new control sessions/following are refused"
                );
                let engine = s.engine;
                let State {
                    authority, epoch, ..
                } = &mut *s;
                authority.command(command, owner, engine, epoch)?
            }
            Request::Hello { .. } => Response::error("handshake", "Handshake already completed"),
            Request::Status | Request::Snapshot => Response::Status(Self::snapshot(&s)),
            Request::Events { engine, after } => {
                let resync = engine != s.engine
                    || after > s.sequence
                    || s.events
                        .front()
                        .is_some_and(|e| after.saturating_add(1) < e.sequence);
                Response::Events {
                    sequence: s.sequence,
                    events: if resync {
                        vec![]
                    } else {
                        s.events
                            .iter()
                            .filter(|e| e.sequence > after)
                            .cloned()
                            .collect()
                    },
                    resync,
                }
            }
            Request::Job { id } => match s.jobs.get(&id) {
                Some(j) => Response::Job(Box::new(j.clone())),
                None => Response::error("not_found", "Unknown job"),
            },
            Request::Jobs { offset, limit } => {
                ensure!(
                    (1..=200).contains(&limit) && offset <= MAX_JOBS,
                    "invalid_request: jobs limit 1–200; offset at most 4096"
                );
                let mut jobs = s
                    .jobs
                    .values()
                    .map(|j| JobInfo {
                        id: j.id,
                        state: j.state,
                        sequence: j.sequence,
                    })
                    .collect::<Vec<_>>();
                jobs.sort_by_key(|j| (j.state.terminal(), std::cmp::Reverse(j.sequence), j.id));
                let total = jobs.len();
                Response::Jobs {
                    items: jobs.into_iter().skip(offset).take(limit).collect(),
                    total,
                    next_offset: (offset + limit < total).then_some(offset + limit),
                }
            }
            Request::Submit(submit) => {
                let submit = *submit;
                if let Some(job) = s.jobs.get(&submit.request) {
                    ensure!(
                        serde_json::to_vec(&job.submitted)? == serde_json::to_vec(&submit)?,
                        "request_conflict: request ID was already used for different input"
                    );
                    return Ok(Response::Job(Box::new(job.clone())));
                }
                ensure!(
                    !s.failed
                        && (!s.draining
                            || !submit.operation.mutates()
                            || matches!(submit.operation, Operation::CloseLogin(_))),
                    "engine_draining: new mutations are refused"
                );
                ensure!(
                    submit.engine == s.engine && submit.epoch == s.epoch,
                    "stale_epoch: authority was revoked or the engine restarted"
                );
                if submit.operation.requires_authority() {
                    s.authority.authorize(owner)?;
                }
                ensure!(
                    s.jobs.len() < MAX_JOBS,
                    "job_quota: acknowledge completed read jobs; mutation receipts are never silently evicted"
                );
                ensure!(
                    s.jobs.values().filter(|j| !j.state.terminal()).count() < MAX_QUEUE,
                    "busy: job admission queue is full"
                );
                ensure!(
                    serde_json::to_vec(&submit)?.len() < 8 * 1024 * 1024,
                    "request_too_large: operation input exceeds 8 MiB"
                );
                if submit.operation.needs_confirmation() {
                    ensure!(
                        submit.confirmation.is_some(),
                        "confirmation_required: prepare this exact operation first"
                    );
                }
                ensure!(
                    submit.confirmation.is_none() || submit.operation.needs_confirmation(),
                    "confirmation_conflict: confirmation is not applicable to this action"
                );
                let frozen_revision = submit.expected_revision.or_else(|| {
                    if submit.operation.freezes_project() {
                        submit
                            .operation
                            .project()
                            .and_then(|p| s.revisions.get(&p).copied())
                    } else {
                        None
                    }
                });
                ensure!(
                    !submit.operation.freezes_project() || frozen_revision.is_some(),
                    "Project revision unavailable; load/create the saved project before submitting this operation"
                );
                let id = submit.request;
                let job = Job {
                    id,
                    owner,
                    submitted: submit,
                    state: JobState::Queued,
                    frozen: None,
                    frozen_revision,
                    progress: None,
                    capture: None,
                    outcome: None,
                    sequence: 0,
                };
                self.save(&job)?;
                s.jobs.insert(id, job);
                s.controls.insert(
                    id,
                    Controls {
                        cancel: Arc::new(AtomicBool::new(false)),
                        pause: Arc::new(AtomicBool::new(false)),
                    },
                );
                Self::event(&mut s, id, false);
                if queue.try_send(id).is_err() {
                    s.failed = true;
                    s.draining = true;
                    let job = s.jobs.get_mut(&id).unwrap();
                    job.state = JobState::Failed;
                    job.outcome=Some(Outcome::ProductFailure("Worker admission failed before product work started. The admitted request ID is retained; restart the engine before issuing more work.".into()));
                    // Admission is already durable: return the actual job, not
                    // a rejection inviting a new mutation ID. A failed terminal
                    // write becomes Interrupted on restart, never replayed.
                    let _ = self.save(job);
                    s.controls.remove(&id);
                    Self::event(&mut s, id, false);
                }
                Response::Job(Box::new(s.jobs[&id].clone()))
            }
            Request::Cancel { id } | Request::Pause { id } => {
                let pause = matches!(request, Request::Pause { .. });
                if pause {
                    let job = s.jobs.get(&id).context("not_found: unknown job")?;
                    ensure!(
                        matches!(job.submitted.operation, Operation::CaptureInventory { .. }),
                        "unsupported: pause applies only to inventory capture/check/resume jobs"
                    );
                }
                if let Some(control) = s.controls.get(&id) {
                    if pause {
                        control.pause.store(true, Ordering::Release);
                    } else {
                        control.cancel.store(true, Ordering::Release);
                    }
                    Response::Ok
                } else if s.jobs.contains_key(&id) {
                    Response::Ok
                } else {
                    Response::error("not_found", "Unknown job")
                }
            }
            Request::Acknowledge { id } => {
                if let Some(job) = s.jobs.get(&id) {
                    ensure!(
                        job.owner == owner || job.submitted.engine != s.engine,
                        "authority_conflict: only the read's frontend/session may reclaim its receipt"
                    );
                    ensure!(
                        job.state.terminal()
                            && !job.submitted.operation.mutates()
                            && !matches!(job.submitted.operation, Operation::Prepare(_)),
                        "Mutation/confirmation receipts are retained for deduplication"
                    );
                    let path = self.directory.join(format!("job-{id}.json"));
                    ownership::private_file(&path, MAX_FRAME as u64)?;
                    std::fs::remove_file(path)?;
                    s.jobs.remove(&id);
                }
                Response::Ok
            }
            Request::AdvanceEpoch { expected } => {
                ensure!(expected == s.epoch, "stale_epoch");
                s.epoch = s.epoch.checked_add(1).context("epoch exhausted")?;
                Response::Status(Self::snapshot(&s))
            }
            Request::Stop => {
                ensure!(
                    !s.login_open && s.jobs.values().all(|j| j.state.terminal()),
                    "busy: jobs or an open login still need the engine; cancel/close them explicitly first"
                );
                ensure!(
                    s.authority.controller.is_none(),
                    "busy: end the agent session before stopping the engine"
                );
                s.draining = true;
                shutdown.store(true, Ordering::Release);
                Response::Ok
            }
            Request::Preferences => {
                Response::Preferences(AppPreferences::load(&self.paths)?.appearance)
            }
            Request::SetAppearance(appearance) => {
                AppPreferences { appearance }.save(&self.paths)?;
                Response::Preferences(appearance)
            }
        })
    }
}

pub struct Server {
    owner: Arc<ownership::Ownership>,
    shared: Arc<Shared>,
    queue: mpsc::SyncSender<Uuid>,
    worker: Option<thread::JoinHandle<()>>,
    shutdown: Arc<AtomicBool>,
}
impl Server {
    pub fn open(paths: AppPaths) -> Result<Self> {
        Self::configured(paths, |paths| Product::configured(&paths.cache))
    }
    #[cfg(feature = "test-support")]
    pub fn without_runtime(paths: AppPaths) -> Result<Self> {
        Self::configured(paths, |p| {
            Ok(Product {
                distribution: None,
                official_runtime: false,
                runtime_cache: p.cache.join("product-runtime"),
                auth: Default::default(),
            })
        })
    }
    /// Owned synthetic/local test runtime injection; never a production fallback.
    #[cfg(feature = "test-support")]
    pub fn with_product(paths: AppPaths, product: Product) -> Result<Self> {
        Self::configured(paths, |_| Ok(product))
    }
    fn configured(
        paths: AppPaths,
        configure: impl FnOnce(&AppPaths) -> Result<Product> + Send + 'static,
    ) -> Result<Self> {
        // Ownership precedes all migrations, journal recovery and interrupted-run recovery.
        let owner = Arc::new(ownership::Ownership::acquire(&paths)?);
        let paths = owner.paths.clone();
        let engine = Uuid::new_v4();
        let frontend_capability = owner.mint_frontend_capability(engine)?;
        let mut jobs = BTreeMap::new();
        let mut journal_bytes = 0u64;
        for entry in std::fs::read_dir(&owner.directory)? {
            let entry = entry?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else { continue };
            if !name.starts_with("job-") || !name.ends_with(".json") {
                continue;
            }
            ensure!(jobs.len() < MAX_JOBS, "Engine job journal exceeds quota");
            ownership::private_file(&entry.path(), MAX_FRAME as u64)?;
            journal_bytes += entry.metadata()?.len();
            ensure!(
                journal_bytes <= 512 * 1024 * 1024,
                "Engine journal exceeds byte quota"
            );
            let mut job: Job = serde_json::from_slice(&std::fs::read(entry.path())?)?;
            ensure!(
                name == format!("job-{}.json", job.id) && job.id == job.submitted.request,
                "Invalid durable job identity"
            );
            if !job.state.terminal() {
                job.state = JobState::Interrupted;
                job.outcome=Some(Outcome::ProductFailure("Engine stopped before a durable terminal receipt. Work may have committed; inspect saved data. Automatic mutation replay is refused.".into()));
                ownership::write_json(&entry.path(), &job)?;
            }
            // Pre-repair capture receipts used a workspace read, whose "latest"
            // may be another run. Preserve that payload for inspection, but do
            // not replay or retroactively certify its old Succeeded status.
            if job.submitted.operation.capture_project().is_some()
                && let Some(outcome) = &job.outcome
                && (job.capture_result_error(outcome).is_some()
                    || (job.state != JobState::Interrupted
                        && job.state != job.terminal_state(outcome)))
            {
                let prior = job.outcome.take().unwrap();
                job.state = JobState::Interrupted;
                job.outcome=Some(Outcome::UnverifiedCapture {message:"Capture receipt predates or violates executed-run binding. Its prior payload is retained but unverified; inspect the requested run. The operation was not replayed.".into(),prior:Box::new(prior)});
                ownership::write_json(&entry.path(), &job)?;
            }
            job.sequence = 0;
            jobs.insert(job.id, job);
        }
        let shared = Arc::new(Shared {
            directory: owner.directory.clone(),
            paths: paths.clone(),
            state: Mutex::new(State {
                engine,
                epoch: 0,
                sequence: 0,
                jobs,
                events: VecDeque::new(),
                controls: BTreeMap::new(),
                confirmations: BTreeMap::new(),
                draining: false,
                login_open: false,
                failed: false,
                revisions: BTreeMap::new(),
                event_bytes: 0,
                authority: authority::Authority::with_bootstrap(frontend_capability),
            }),
        });
        let (queue, rx) = mpsc::sync_channel(MAX_QUEUE);
        let (ready, wait) = mpsc::channel();
        let worker_shared = shared.clone();
        let worker_owner = owner.clone();
        let worker = thread::spawn(move || {
            let _ownership = worker_owner;
            let startup = (|| -> Result<_> {
                let product = configure(&paths);
                let mut store = Store::open(paths)?;
                store.recover_runs()?;
                // Confirmed metadata deletion must not make unrelated projects
                // unavailable when physical file cleanup needs repair. Retain
                // its journal and expose the pending count in storage status.
                let _ = store.complete_confirmed_gc();
                worker_shared.state.lock().unwrap().revisions = read_revisions(&store)?;
                Ok((store, product))
            })();
            let mut context = match startup {
                Ok(c) => {
                    let _ = ready.send(Ok(()));
                    c
                }
                Err(e) => {
                    let _ = ready.send(Err(e.to_string()));
                    return;
                }
            };
            let mut viewer = viewer::ViewerCache::default();
            while let Ok(id) = rx.recv() {
                run_job(&worker_shared, &mut context.0, &context.1, &mut viewer, id);
            }
        });
        wait.recv()
            .context("Engine worker failed before readiness")?
            .map_err(anyhow::Error::msg)?;
        ownership::write_json(
            &owner.directory.join("endpoint.json"),
            &serde_json::json!({"version":VERSION,"engine":engine,"cache":owner.paths.cache}),
        )?;
        Ok(Self {
            owner,
            shared,
            queue,
            worker: Some(worker),
            shutdown: Arc::new(AtomicBool::new(false)),
        })
    }
    pub fn run(self) -> Result<()> {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(2)
            .enable_all()
            .build()?;
        runtime.block_on(self.serve())
    }
    async fn serve(mut self) -> Result<()> {
        let mut listener = ipc::Listener::bind(&self.owner.paths)?;
        let semaphore = Arc::new(tokio::sync::Semaphore::new(MAX_CONNECTIONS));
        let mut tasks = tokio::task::JoinSet::new();
        let mut signal = Box::pin(tokio::signal::ctrl_c());
        let mut foreground_stop = false;
        while !self.shutdown.load(Ordering::Acquire) {
            expire_authority(&mut self.shared.state.lock().unwrap());
            if foreground_stop {
                let state = self.shared.state.lock().unwrap();
                if !state.login_open && state.jobs.values().all(|j| j.state.terminal()) {
                    self.shutdown.store(true, Ordering::Release);
                    break;
                }
            }
            while tasks.try_join_next().is_some() {}
            let accepted = tokio::select! {
                result=tokio::time::timeout(Duration::from_millis(50),listener.accept())=>result,
                _=&mut signal=>{
                    self.shared.state.lock().unwrap().draining=true;foreground_stop=true;
                    signal=Box::pin(tokio::signal::ctrl_c());continue;
                }
            };
            match accepted {
                Err(_) => continue,
                Ok(Err(_)) => {
                    tokio::time::sleep(Duration::from_millis(25)).await;
                    continue;
                }
                Ok(Ok(mut stream)) => {
                    let Ok(permit) = semaphore.clone().try_acquire_owned() else {
                        continue;
                    };
                    let shared = self.shared.clone();
                    let queue = self.queue.clone();
                    let shutdown = self.shutdown.clone();
                    tasks.spawn(async move {
                        let _permit = permit;
                        let result = async {
                            match ipc::read::<Request>(&mut stream).await? {
                                Request::Hello { version } if version == VERSION => {}
                                _ => {
                                    ipc::write(
                                        &mut stream,
                                        &Response::error(
                                            "version_mismatch",
                                            "Unsupported engine protocol version",
                                        ),
                                    )
                                    .await?;
                                    return Ok::<(), anyhow::Error>(());
                                }
                            }
                            let engine = shared.state.lock().unwrap().engine;
                            ipc::write(
                                &mut stream,
                                &Response::Hello {
                                    version: VERSION,
                                    engine,
                                },
                            )
                            .await?;
                            // One bounded request per authenticated connection. Slow/disconnected
                            // readers hold no broker/store lock and cannot block job publication.
                            let request = ipc::read::<Request>(&mut stream).await?;
                            // Journal fsync and preference IO must not pin the
                            // reactor that enforces connection/read deadlines.
                            let response = tokio::task::spawn_blocking(move || {
                                shared.handle(request, &queue, &shutdown)
                            })
                            .await
                            .map_err(|e| anyhow::anyhow!("Broker request failed: {e}"))?
                            .unwrap_or_else(|e| Response::error("request_rejected", e.to_string()));
                            if let Err(error) = ipc::write(&mut stream, &response).await {
                                let _ = error;
                            }
                            Ok(())
                        }
                        .await;
                        let _ = result;
                    });
                }
            }
        }
        while tasks.join_next().await.is_some() {}
        drop(listener);
        drop(self.queue);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
        // Lock is released only after Product/AuthOwner/Store have been dropped.
        Ok(())
    }
}

fn run_job(
    shared: &Arc<Shared>,
    store: &mut Store,
    product: &Result<Product>,
    viewer: &mut viewer::ViewerCache,
    id: Uuid,
) {
    let (submitted, cancel, pause) = {
        let mut s = shared.state.lock().unwrap();
        expire_authority(&mut s);
        let job = &s.jobs[&id];
        let submitted = job.submitted.clone();
        let controls = &s.controls[&id];
        let cancel = controls.cancel.clone();
        let pause = controls.pause.clone();
        if (submitted.operation.requires_authority()
            && (submitted.engine != s.engine
                || submitted.epoch != s.epoch
                || s.authority.authorize(job.owner).is_err()))
            || s.failed
        {
            drop(s);
            shared.finish(
                id,
                Outcome::ProductFailure("stale_epoch: queued command authority revoked".into()),
                false,
            );
            return;
        }
        if cancel.load(Ordering::Acquire) {
            drop(s);
            shared.finish(id, Outcome::Cancelled, false);
            return;
        }
        let frozen = submitted
            .operation
            .project()
            .and_then(|p| store.load_project(p).ok());
        if frozen
            .as_ref()
            .is_some_and(|p| serde_json::to_vec(p).map_or(true, |b| b.len() > 8 * 1024 * 1024))
        {
            drop(s);
            shared.finish(
                id,
                Outcome::ProductFailure("Frozen project exceeds engine input budget".into()),
                false,
            );
            return;
        }
        if let Some(expected) = s.jobs[&id].frozen_revision
            && frozen.as_ref().map(|r| r.revision) != Some(expected)
        {
            drop(s);
            shared.finish(
                id,
                Outcome::ProductFailure(
                    "revision_conflict: frozen project changed before execution".into(),
                ),
                false,
            );
            return;
        }
        let job = s.jobs.get_mut(&id).unwrap();
        job.frozen = frozen;
        job.state = JobState::Running;
        if shared.save(job).is_err() {
            drop(s);
            shared.finish(
                id,
                Outcome::ProductFailure(
                    "Cannot durably start job; no product work was started".into(),
                ),
                true,
            );
            return;
        }
        Shared::event(&mut s, id, false);
        (submitted, cancel, pause)
    };
    let mutating =
        submitted.operation.mutates() && !matches!(submitted.operation, Operation::SelectAgent(_));
    let (updates, rx) = mpsc::sync_channel(8);
    let observation = shared.clone();
    let observer = thread::spawn(move || {
        let mut durable_at = std::time::Instant::now();
        while let Ok(update) = rx.recv() {
            let update: crate::jobs::Update = update;
            let mut progress = Outcome::from_reply(update.reply);
            if serde_json::to_vec(&progress).map_or(true, |b| b.len() > 65536) {
                progress = Outcome::Progress {
                    message: "Operation is working; detailed progress exceeds the event budget"
                        .into(),
                    completed: 0,
                    total: None,
                };
            }
            let mut state = observation.state.lock().unwrap();
            if let Some(job) = state.jobs.get_mut(&id) {
                if let Outcome::CaptureStatus(status) = &progress {
                    job.capture = Some(status.clone());
                }
                job.progress = Some(progress);
                Shared::event(&mut state, id, false);
            }
            if durable_at.elapsed() >= Duration::from_secs(1) {
                if observation.save(&state.jobs[&id]).is_err() {
                    state.failed = true;
                    state.draining = true;
                }
                durable_at = std::time::Instant::now();
            }
        }
    });
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| -> Result<Outcome> {
        if let Operation::Prepare(operation) = submitted.operation.clone() {
            ensure!(
                operation.needs_confirmation(),
                "This operation has no consequential confirmation"
            );
            // Consent binds the exact action and this owner's current storage
            // revision. Underlying services also revalidate their specific previews.
            let mut state = shared.state.lock().unwrap();
            expire_authority(&mut state);
            ensure!(
                state.epoch == submitted.epoch,
                "stale_epoch: confirmation authority revoked"
            );
            state.authority.authorize(state.jobs[&id].owner)?;
            let confirmation = Confirmation {
                id: Uuid::new_v4(),
                engine: state.engine,
                epoch: state.epoch,
                owner: state.jobs[&id].owner,
                fingerprint: operation.fingerprint()?,
                revision: store.viewer_revision()?,
                preview: confirmation_preview(store, &operation)?,
            };
            state
                .confirmations
                .insert(confirmation.id, (confirmation.clone(), None));
            return Ok(Outcome::Confirmation(confirmation));
        }
        if submitted.operation.needs_confirmation() {
            let mut s = shared.state.lock().unwrap();
            expire_authority(&mut s);
            ensure!(
                s.epoch == submitted.epoch,
                "stale_epoch: authority revoked before confirmation dispatch"
            );
            let owner = s.jobs[&id].owner;
            s.authority.authorize(owner)?;
            let engine = s.engine;
            let (confirmation, used) = s
                .confirmations
                .get_mut(&submitted.confirmation.context("confirmation_required")?)
                .context("confirmation_conflict: unknown/expired confirmation; prepare again")?;
            ensure!(
                confirmation.engine == engine
                    && confirmation.owner == owner
                    && confirmation.epoch == submitted.epoch
                    && confirmation.fingerprint == submitted.operation.fingerprint()?
                    && confirmation.revision == store.viewer_revision()?,
                "confirmation_conflict: action, authority or saved revision changed"
            );
            ensure!(
                used.is_none(),
                "confirmation_replayed: confirmation already consumed"
            );
            *used = Some(id);
        }
        if let Operation::Viewer(selection) = submitted.operation.clone() {
            return Ok(Outcome::Viewer(Box::new(viewer.frame(store, selection)?)));
        }
        if let Operation::SelectedEvidence(selection) = submitted.operation.clone() {
            ensure!(
                selection.run.is_some(),
                "invalid_request: saved evidence refresh requires an exact run"
            );
            let location = authority::location(store, selection)?;
            return Ok(Outcome::Evidence(Box::new(
                location
                    .evidence
                    .context("not_found: selected run unavailable")?,
            )));
        }
        if let Operation::SelectAgent(selection) = submitted.operation.clone() {
            let location = authority::location(store, selection)?;
            let mut s = shared.state.lock().unwrap();
            expire_authority(&mut s);
            ensure!(
                s.epoch == submitted.epoch,
                "stale_epoch: selection authority revoked"
            );
            let owner = s.jobs[&id].owner;
            s.authority.authorize(owner)?;
            let Some(crate::control::Owner::Agent(agent)) = owner else {
                anyhow::bail!("authority_conflict: selection belongs to a named agent session")
            };
            let a = s
                .authority
                .agents
                .get_mut(&agent)
                .context("authority_conflict: session absent")?;
            a.info.selection = Some(location.selection.clone());
            a.info.selection_revision += 1;
            // Retain only semantic IDs in this authority-bearing receipt. Full
            // run evidence belongs to a reclaimable read, not every navigation
            // command in the bounded durable mutation journal.
            return Ok(Outcome::AgentSelected {
                selection: location.selection,
            });
        }
        if let Operation::AgentLocation { session } = submitted.operation {
            let selection = shared
                .state
                .lock()
                .unwrap()
                .authority
                .agents
                .get(&session)
                .and_then(|a| a.info.selection.clone())
                .context("not_found: no agent selection")?;
            return Ok(Outcome::AgentLocation(Box::new(authority::location(
                store, selection,
            )?)));
        }
        #[cfg(feature = "test-support")]
        if let Operation::TestWait { millis } = submitted.operation {
            ensure!(millis <= 30000, "test duration exceeds bounds");
            for n in 0..millis.div_ceil(10) {
                if cancel.load(Ordering::Acquire) {
                    return Ok(Outcome::Cancelled);
                }
                let _ = updates.try_send(crate::jobs::Update {
                    id: 0,
                    reply: crate::jobs::Reply::Progress {
                        message: "Owned synthetic checkpoint".into(),
                        completed: n as u32,
                        total: Some(millis.div_ceil(10) as u32),
                    },
                });
                thread::sleep(Duration::from_millis(10));
            }
            return Ok(Outcome::Runtime("Synthetic operation complete".into()));
        }
        if submitted.operation.requires_authority() {
            let mut state = shared.state.lock().unwrap();
            expire_authority(&mut state);
            ensure!(
                state.epoch == submitted.epoch,
                "stale_epoch: authority revoked before dispatch"
            );
            state.authority.authorize(state.jobs[&id].owner)?;
        }
        if let Operation::Terminal(command) = submitted.operation.clone() {
            return Ok(Outcome::Terminal(Box::new(crate::terminal::execute(
                store, command, &cancel,
            )?)));
        }
        if let Operation::SetAppearance(appearance) = submitted.operation {
            AppPreferences { appearance }.save(&shared.paths)?;
            return Ok(Outcome::Runtime("Appearance saved".into()));
        }
        validate_intent(store, &submitted.operation)?;
        let work = submitted.operation.into_work(cancel.clone())?;
        Ok(Outcome::from_reply(executor::execute(
            store, product, work, &cancel, &pause, &updates,
        )))
    }));
    drop(updates);
    let _ = observer.join();
    {
        let mut state = shared.state.lock().unwrap();
        state.login_open = product
            .as_ref()
            .is_ok_and(|p| p.auth.lock().map(|a| a.has_open_login()).unwrap_or(true));
        if mutating {
            match read_revisions(store) {
                Ok(r) => state.revisions = r,
                Err(_) => {
                    state.failed = true;
                    state.draining = true;
                }
            }
        }
    }
    match result {
        Ok(result) => shared.finish(id, normalized_outcome(result), false),
        Err(_) => {
            shared.state.lock().unwrap().failed = true;
            shared.finish(
                id,
                Outcome::ProductFailure(
                    "Worker panicked; completion is uncertain. Automatic retries are refused."
                        .into(),
                ),
                true,
            );
        }
    }
}

fn normalized_outcome(result: Result<Outcome>) -> Outcome {
    match result {
        Ok(outcome) => outcome,
        Err(error) if crate::jobs::typed_cancellation(&error) => Outcome::Cancelled,
        Err(error) => Outcome::ProductFailure(error.to_string()),
    }
}
fn expire_authority(state: &mut State) {
    state
        .authority
        .expire(&mut state.epoch, std::time::Instant::now());
    if (state.failed || state.draining)
        && let Some(id) = state.authority.controller.take()
    {
        if let Some(agent) = state.authority.agents.get_mut(&id) {
            agent.info.status = crate::control::Status::Revoked;
            agent.info.follow = None;
        }
        state.epoch += 1;
    }
}

/// Stable public validation failures before the existing transaction-time checks.
/// The latter are still authoritative; this never replaces them or opens a store
/// in a client. In particular write failures are not mislabeled as stale drafts.
fn validate_intent(store: &Store, operation: &Operation) -> Result<()> {
    let save = match operation {
        Operation::Save {
            project,
            expected_revision,
        } => Some((&**project, *expected_revision, false)),
        Operation::SaveReplacement {
            project,
            expected_revision,
        } => Some((&**project, Some(*expected_revision), true)),
        Operation::SaveEnvironment {
            project,
            revision,
            settings,
            ..
        } => {
            settings
                .validate()
                .context("invalid_request: invalid local login settings")?;
            Some((&**project, Some(*revision), false))
        }
        Operation::SaveReplacementEnvironment {
            project,
            revision,
            settings,
            ..
        } => {
            settings
                .validate()
                .context("invalid_request: invalid local login settings")?;
            Some((&**project, Some(*revision), true))
        }
        _ => None,
    };
    if let Some((project, revision, replacement)) = save {
        project
            .validate()
            .context("invalid_request: project settings failed validation")?;
        if let Some(revision) = revision {
            let saved = store
                .load_project(project.id)
                .context("not_found: project unavailable")?;
            ensure!(
                saved.revision == revision,
                "revision_conflict: project settings changed"
            );
            ensure!(
                saved.project.active_baseline == project.active_baseline,
                "invalid_request: baseline selection requires its separate confirmed action"
            );
            if saved.project.active_baseline.is_some() && !replacement {
                ensure!(
                    saved.project.sources == project.sources
                        && saved.project.profiles == project.profiles
                        && saved.project.environments == project.environments
                        && saved.project.capture_plan == project.capture_plan,
                    "replacement_required: active-baseline-bound sources, profiles, environments or check modes require explicit replacement preparation"
                );
            }
        } else {
            ensure!(
                store.load_project(project.id).is_err(),
                "revision_conflict: project already exists; use update with expected revision"
            );
        }
    }
    match operation {
        Operation::Dom(command) => {
            command.validate()?;
            if let dom::Command::Accept(preview) = command {
                dom::validate_accept(store, preview)?;
            }
        }
        Operation::Accept { preview, context } => {
            let fresh = store
                .approval_preview(preview.analysis, &preview.selected)
                .context("confirmation_conflict: approval preview is no longer current")?;
            ensure!(
                fresh.context == *context
                    && preview.context == *context
                    && fresh.baseline.references == preview.baseline.references,
                "confirmation_conflict: approval context changed"
            );
        }
        Operation::Restore { baseline, revision } => {
            let b = store
                .load_baseline(*baseline)
                .context("not_found: baseline unavailable")?;
            ensure!(
                store.load_project(b.project)?.revision == *revision,
                "revision_conflict: project changed before restoration"
            );
        }
        Operation::Delete(preview) => {
            ensure!(
                store.deletion_preview(preview.project, preview.item)? == **preview,
                "confirmation_conflict: deletion preview changed"
            );
        }
        Operation::Mask { request, .. } => request
            .validate()
            .context("invalid_request: invalid mask/analysis binding")?,
        Operation::List { page } | Operation::History { page, .. } => {
            ensure!(*page <= 15_625, "invalid_request: page exceeds 15625")
        }
        Operation::Activate {
            run,
            revision,
            confirm_partial,
        } => {
            let record = store
                .load_run(*run)
                .context("not_found: capture run unavailable")?;
            let saved = store.load_project(record.run.project)?;
            ensure!(
                saved.revision == *revision
                    && record
                        .queue
                        .as_ref()
                        .is_none_or(|q| q.project_revision == *revision),
                "revision_conflict: project changed since this baseline capture"
            );
            record
                .baseline_save_counts()
                .context("invalid_request: this run is not a finished baseline candidate")?;
            let coverage = store.baseline_eligible_coverage(*run)?;
            ensure!(
                coverage.screenshot_captures + coverage.dom_captures > 0,
                "invalid_request: this candidate has no eligible reference slots; unstable DOM evidence cannot become a baseline"
            );
            ensure!(
                (coverage.captured_targets == coverage.intended_targets
                    && coverage.screenshot_captures == coverage.screenshot_intended
                    && coverage.dom_captures == coverage.dom_intended)
                    || *confirm_partial,
                "confirmation_required: partial baseline omits requested screenshot/DOM captures; inspect per-kind coverage and explicitly include --partial when preparing"
            );
        }
        Operation::CaptureInventory {
            project,
            preview: Some(preview),
            ..
        }
        | Operation::PreviewImages { project, preview } => {
            let saved = store.load_project(*project)?;
            ensure!(
                saved.project == preview.settings
                    && saved.revision == preview.project_revision
                    && preview.inventory.project == *project,
                "revision_conflict: inventory preview settings changed"
            );
        }
        _ => {}
    }
    Ok(())
}

fn confirmation_preview(store: &Store, operation: &Operation) -> Result<Option<serde_json::Value>> {
    Ok(match operation {
        Operation::Dom(dom::Command::Accept(preview)) => {
            let canonical = dom::canonical_accept(store, preview)?;
            Some(
                serde_json::json!({"action":"accept_explicit_kinds","preview":canonical,"warning":"Only explicitly selected screenshot/DOM reference slots change. Sibling kinds and retained history are inherited without implicit approval."}),
            )
        }
        Operation::Activate {
            run,
            revision,
            confirm_partial,
        } => {
            use regressee_core::domain::{CheckKind, ReviewUnitKey};
            let r = store.load_run(*run)?;
            let coverage = store.baseline_eligible_coverage(*run)?;
            let mut omitted = Vec::new();
            for target in &r.run.targets {
                for kind in r.run.capture_plan.kinds() {
                    let eligible = match kind {
                        CheckKind::Screenshot => r.evidence(&target.key).is_some(),
                        CheckKind::DomSnapshot => r
                            .dom_evidence(&target.key)
                            .is_some_and(|c| c.metadata.baseline_ready()),
                    };
                    if !eligible {
                        omitted.push(ReviewUnitKey {
                            key: target.key.clone(),
                            kind,
                        });
                    }
                }
            }
            Some(
                serde_json::json!({"action":"save_baseline","run":r.run,"settings_revision":revision,"captured_and_intended":(coverage.captured_targets,coverage.intended_targets),"per_kind_coverage":coverage,"captured_coverage":r.baseline_save_coverage(),"omitted_slots":omitted,"partial_confirmation":confirm_partial,"warning":"Saving changes future checks; failed, missing or unstable DOM slots are ineligible, never unchanged evidence. Explicitly omitted reference slots remain omissions. Original baseline history is retained."}),
            )
        }
        Operation::Restore { baseline, revision } => Some(
            serde_json::json!({"action":"restore_baseline","baseline":store.load_baseline(*baseline)?,"expected_revision":revision}),
        ),
        Operation::SaveReplacement { project, .. }
        | Operation::SaveReplacementEnvironment { project, .. } => Some(
            serde_json::json!({"action":"replace_baseline_bound_settings","before":store.load_project(project.id)?,"after":project,"warning":"Existing baseline evidence/history is retained; capture and separately save a replacement baseline before fixed-list checks use the new settings."}),
        ),
        _ => None,
    })
}

fn read_revisions(store: &Store) -> Result<BTreeMap<regressee_core::domain::ProjectId, u64>> {
    let mut revisions = BTreeMap::new();
    let mut page = 0;
    loop {
        let records = store.list_projects(page)?;
        for record in records.items {
            revisions.insert(record.id, record.revision);
        }
        ensure!(
            revisions.len() <= 100_000,
            "Engine project index exceeds its bounded capacity"
        );
        if !records.more {
            break;
        }
        page += 1;
    }
    Ok(revisions)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn typed_report_cancellation_normalizes_without_rewriting_committed_success() {
        let cancelled = anyhow::Error::new(crate::analysis_engine::Cancelled)
            .context("Rendering saved DOM report");
        assert!(matches!(
            normalized_outcome(Err(cancelled)),
            Outcome::Cancelled
        ));
        let worker = anyhow::Error::new(regressee_runtime::supervisor::WorkerFailure(
            regressee_runtime::protocol::FailureCode::Cancelled,
        ))
        .context("Capture checkpoint");
        assert!(matches!(
            normalized_outcome(Err(worker)),
            Outcome::Cancelled
        ));
        assert!(matches!(
            normalized_outcome(Err(anyhow::anyhow!("DOM analysis cancelled"))),
            Outcome::ProductFailure(_)
        ));
        let late_cancel = AtomicBool::new(true);
        let committed = Outcome::Terminal(Box::new(crate::terminal::Output::Reported {
            html: "owned/index.html".into(),
            markdown: "owned/summary.md".into(),
        }));
        assert!(late_cancel.load(Ordering::Acquire));
        assert!(matches!(
            normalized_outcome(Ok(committed)),
            Outcome::Terminal(_)
        ));
    }

    #[test]
    fn oversized_read_fails_bounded_without_draining_or_poisoning_the_engine() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut server = Server::configured(AppPaths::isolated(root.path()), |_| {
            Err(anyhow::anyhow!("No runtime needed"))
        })?;
        let (queue, _receiver) = mpsc::sync_channel(MAX_QUEUE);
        let snapshot = Shared::snapshot(&server.shared.state.lock().unwrap());
        let results = dom::maximal_warning_results(169);
        let submitted = Submit {
            request: Uuid::new_v4(),
            engine: snapshot.engine,
            epoch: snapshot.epoch,
            operation: Operation::Dom(dom::Command::Results {
                selection: results.selection.clone(),
                offset: 0,
                limit: 200,
                page: None,
                profile: None,
                outcome: None,
                capture_state: None,
            }),
            confirmation: None,
            expected_revision: None,
        };
        server.shared.handle(
            Request::Submit(Box::new(submitted.clone())),
            &queue,
            &server.shutdown,
        )?;
        server.shared.finish(
            submitted.request,
            Outcome::Dom(Box::new(dom::Output::Results(results))),
            false,
        );
        {
            let state = server.shared.state.lock().unwrap();
            let job = &state.jobs[&submitted.request];
            assert_eq!(job.state, JobState::Failed);
            assert!(
                matches!(&job.outcome, Some(Outcome::ProductFailure(message)) if message.starts_with("response_too_large:"))
            );
            assert!(!state.failed && !state.draining);
        }
        let next = Submit {
            request: Uuid::new_v4(),
            operation: Operation::List { page: 0 },
            ..submitted
        };
        assert!(matches!(
            server.shared.handle(
                Request::Submit(Box::new(next.clone())),
                &queue,
                &server.shutdown
            )?,
            Response::Job(_)
        ));
        server.shared.finish(
            next.request,
            Outcome::Runtime("Healthy owned read".into()),
            false,
        );
        assert!(
            matches!(server.shared.handle(Request::Status, &queue, &server.shutdown)?, Response::Status(status) if !status.draining)
        );
        assert_eq!(
            server.shared.state.lock().unwrap().jobs[&next.request].state,
            JobState::Succeeded
        );
        drop(server.queue);
        server.worker.take().unwrap().join().unwrap();
        Ok(())
    }
    #[test]
    fn unavailable_worker_returns_the_durable_terminal_job_not_a_false_rejection() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut server = Server::configured(AppPaths::isolated(root.path()), |_| {
            Err(anyhow::anyhow!("No runtime needed"))
        })?;
        let (queue, receiver) = mpsc::sync_channel(MAX_QUEUE);
        drop(receiver);
        let snapshot = Shared::snapshot(&server.shared.state.lock().unwrap());
        let submitted = Submit {
            request: Uuid::new_v4(),
            engine: snapshot.engine,
            epoch: snapshot.epoch,
            operation: Operation::List { page: 0 },
            confirmation: None,
            expected_revision: None,
        };
        let result = server.shared.handle(
            Request::Submit(Box::new(submitted.clone())),
            &queue,
            &server.shutdown,
        )?;
        assert!(
            matches!(result,Response::Job(job) if job.id==submitted.request && job.state==JobState::Failed)
        );
        let disk: Job = serde_json::from_slice(&std::fs::read(
            server
                .owner
                .directory
                .join(format!("job-{}.json", submitted.request)),
        )?)?;
        assert_eq!(disk.state, JobState::Failed);
        assert!(matches!(
            server
                .shared
                .handle(Request::Status, &queue, &server.shutdown)?,
            Response::Status(_)
        ));
        drop(server.queue);
        server.worker.take().unwrap().join().unwrap();
        Ok(())
    }
}
