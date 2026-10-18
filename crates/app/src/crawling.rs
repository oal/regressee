// SPDX-License-Identifier: MPL-2.0
//! Blocking Phase 3 services. One bounded scheduler owns every browser operation;
//! persisted target boundaries, not worker progress, are authoritative.
use crate::capture_evidence::PublishedChannel;
use crate::product::{Product, map_failure, worker_version};
use anyhow::{Context, Result, ensure};
use regressee_core::{crawl::*, domain::*, evidence::*};
use regressee_runtime::{
    distribution::{self, Installed, Scope},
    protocol::{Command, EventKind, FailureCode, Request},
    supervisor::{ProcessSupervisor, Worker, WorkerFailure},
};
use regressee_storage::Store;
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

fn scoped(installed: &Installed, url: &url::Url) -> Result<()> {
    ensure!(
        installed.manifest.scope != Scope::DevelopmentLoopback || distribution::loopback(url),
        "This test browser can only visit approved local test websites, not public websites. Use a browser build that supports public websites."
    );
    Ok(())
}

fn capture_targets(
    inventory: &InventoryRevision,
    profiles: &[CaptureProfile],
    plan: &CapturePlan,
    baseline: Option<&BaselineVersion>,
) -> Vec<CaptureTarget> {
    let reference_keys = baseline.map(BaselineVersion::target_keys);
    let saved_units = baseline.map(BaselineVersion::saved_units);
    let mut targets = Vec::new();
    for page in inventory.pages.iter().filter(|p| p.included) {
        for profile in profiles {
            let key = TargetKey {
                page: page.id,
                profile: profile.id,
            };
            if reference_keys
                .as_ref()
                .is_some_and(|keys| !keys.contains(&key))
            {
                continue;
            }
            let saved = |kind| {
                saved_units.as_ref().is_none_or(|units| {
                    units.contains(&ReviewUnitKey {
                        key: key.clone(),
                        kind,
                    })
                })
            };
            let screenshot = plan.mode.screenshots() && saved(CheckKind::Screenshot);
            let dom = plan.mode.dom_snapshots() && saved(CheckKind::DomSnapshot);
            targets.push(CaptureTarget {
                key,
                requested: page.canonical.clone(),
                final_url: None,
                attempts: 0,
                outcome: if screenshot {
                    TargetOutcome::Pending
                } else {
                    TargetOutcome::NotRequested
                },
                dom_outcome: if dom {
                    DomTargetOutcome::Pending
                } else {
                    DomTargetOutcome::NotRequested
                },
                warnings: vec![],
            });
        }
    }
    targets
}
fn target_pending(target: &CaptureTarget) -> bool {
    matches!(target.outcome, TargetOutcome::Pending)
        || matches!(target.dom_outcome, DomTargetOutcome::Pending)
}
fn pending_plan(target: &CaptureTarget, frozen: &CapturePlan) -> Result<CapturePlan> {
    frozen.validate()?;
    let screenshot = matches!(target.outcome, TargetOutcome::Pending);
    let dom = matches!(target.dom_outcome, DomTargetOutcome::Pending);
    ensure!(
        (!screenshot || frozen.mode.screenshots()) && (!dom || frozen.mode.dom_snapshots()),
        "capture requested a disabled kind"
    );
    let mode = match (screenshot, dom) {
        (true, true) => CaptureMode::ScreenshotsAndDomSnapshots,
        (true, false) => CaptureMode::ScreenshotsOnly,
        (false, true) => CaptureMode::DomSnapshotsOnly,
        (false, false) => anyhow::bail!("target has no pending requested slots"),
    };
    let mut plan = CapturePlan::for_mode(mode);
    if dom {
        plan.dom_policy = frozen.dom_policy;
    }
    Ok(plan)
}
fn cancel_pending(target: &mut CaptureTarget) {
    if matches!(target.outcome, TargetOutcome::Pending) {
        target.outcome = TargetOutcome::Cancelled;
    }
    if matches!(target.dom_outcome, DomTargetOutcome::Pending) {
        target.dom_outcome = DomTargetOutcome::Cancelled;
    }
}
fn skip_pending(target: &mut CaptureTarget, reason: &str) {
    if matches!(target.outcome, TargetOutcome::Pending) {
        target.outcome = TargetOutcome::Skipped {
            reason: reason.into(),
        };
    }
    if matches!(target.dom_outcome, DomTargetOutcome::Pending) {
        target.dom_outcome = DomTargetOutcome::Skipped {
            reason: reason.into(),
        };
    }
}
fn fail_pending(target: &mut CaptureTarget, code: CaptureError) {
    if matches!(target.outcome, TargetOutcome::Pending) {
        target.outcome = TargetOutcome::Failed { code };
    }
    if matches!(target.dom_outcome, DomTargetOutcome::Pending) {
        target.dom_outcome = DomTargetOutcome::Failed {
            code: DomCaptureError::Capture {
                code: if code == CaptureError::ImageLimit {
                    CaptureError::CaptureFailed
                } else {
                    code
                },
            },
        };
    }
}

fn checkpoint_target_metadata(
    store: &mut Store,
    checkpoint: &mut regressee_storage::RunAppender,
    record: &RunRecord,
    indices: &[usize],
) -> Result<()> {
    if indices.is_empty() {
        return Ok(());
    }
    let updates: Vec<_> = indices
        .iter()
        .map(|index| regressee_storage::KindTargetCheckpoint {
            index: *index,
            target: &record.run.targets[*index],
            capture: None,
            dom_capture: None,
        })
        .collect();
    store.append_run_kind_targets(checkpoint, &updates)
}
fn capture_message(plan: &CapturePlan) -> &'static str {
    match plan.mode {
        CaptureMode::ScreenshotsOnly => "Capturing screenshots…",
        CaptureMode::ScreenshotsAndDomSnapshots => "Capturing screenshots and DOM snapshots…",
        CaptureMode::DomSnapshotsOnly => "Capturing DOM snapshots…",
    }
}

#[allow(clippy::too_many_arguments)]
fn publish_channel(
    store: &mut Store,
    record: &mut RunRecord,
    checkpoint: &mut regressee_storage::RunAppender,
    index: usize,
    kind: CheckKind,
    publication: Result<PublishedChannel>,
    retries: u8,
    backoff: &mut u64,
    pause: &AtomicBool,
) -> Result<()> {
    let screenshots = record.captures.len();
    let snapshots = record.dom_captures.len();
    let publication = publication.and_then(|publication| {
        ensure!(publication.kind() == kind, "foreign capture channel");
        match publication {
            PublishedChannel::Screenshot {
                metadata,
                artifact,
                path,
            } => crate::capture_evidence::retain_screenshot(
                store, record, index, metadata, &artifact, &path,
            ),
            PublishedChannel::DomSnapshot {
                metadata,
                artifact,
                path,
            } => crate::capture_evidence::retain_dom(
                store, record, index, metadata, &artifact, &path,
            ),
            PublishedChannel::ScreenshotFailed { code, retryable } => {
                let target = &mut record.run.targets[index];
                ensure!(
                    matches!(target.outcome, TargetOutcome::Pending),
                    "duplicate screenshot result"
                );
                if code == CaptureError::NeedsLogin {
                    pause.store(true, Ordering::Release);
                }
                if retryable && target.attempts <= retries {
                    *backoff = (*backoff).max(1000u64 << target.attempts.min(5));
                } else if code == CaptureError::Cancelled {
                    target.outcome = TargetOutcome::Cancelled;
                } else {
                    target.outcome = TargetOutcome::Failed { code };
                }
                Ok(())
            }
            PublishedChannel::DomSnapshotFailed { code, retryable } => {
                let target = &mut record.run.targets[index];
                ensure!(
                    matches!(target.dom_outcome, DomTargetOutcome::Pending),
                    "duplicate DOM result"
                );
                code.validate()?;
                if matches!(
                    code,
                    DomCaptureError::Capture {
                        code: CaptureError::NeedsLogin
                    }
                ) {
                    pause.store(true, Ordering::Release);
                }
                if retryable && target.attempts <= retries {
                    *backoff = (*backoff).max(1000u64 << target.attempts.min(5));
                } else if matches!(
                    code,
                    DomCaptureError::Capture {
                        code: CaptureError::Cancelled
                    }
                ) {
                    target.dom_outcome = DomTargetOutcome::Cancelled;
                } else {
                    target.dom_outcome = DomTargetOutcome::Failed { code };
                }
                Ok(())
            }
        }
    });
    if let Err(error) = publication {
        let target = &mut record.run.targets[index];
        // A malformed channel cannot discard a sibling's completed evidence.
        match kind {
            CheckKind::Screenshot if matches!(target.outcome, TargetOutcome::Pending) => {
                target.outcome = TargetOutcome::Failed {
                    code: map_failure(&error),
                }
            }
            CheckKind::DomSnapshot if matches!(target.dom_outcome, DomTargetOutcome::Pending) => {
                target.dom_outcome = DomTargetOutcome::Failed {
                    code: if map_failure(&error) == CaptureError::LowSpace {
                        DomCaptureError::Capture {
                            code: CaptureError::LowSpace,
                        }
                    } else {
                        DomCaptureError::SerializationFailed
                    },
                }
            }
            _ => return Err(error),
        }
    }
    let target = &record.run.targets[index];
    store.append_run_kind_targets(
        checkpoint,
        &[regressee_storage::KindTargetCheckpoint {
            index,
            target,
            capture: (record.captures.len() > screenshots).then(|| record.captures.last().unwrap()),
            dom_capture: (record.dom_captures.len() > snapshots)
                .then(|| record.dom_captures.last().unwrap()),
        }],
    )
}
fn request(
    worker: &mut Worker,
    job: RunId,
    id: &mut u64,
    command: Command,
    cancel: Arc<AtomicBool>,
    seconds: u64,
    event: &mut dyn FnMut(EventKind) -> Result<()>,
) -> Result<()> {
    *id += 2;
    worker.request(
        Request {
            version: regressee_runtime::protocol::PROTOCOL_VERSION,
            request_id: *id,
            job,
            command,
        },
        Instant::now() + Duration::from_secs(seconds),
        cancel,
        &mut |e| event(e.event),
    )
}
fn wait(until: Instant, cancel: &AtomicBool, pause: &AtomicBool) -> bool {
    while Instant::now() < until {
        if cancel.load(Ordering::Acquire) || pause.load(Ordering::Acquire) {
            return false;
        }
        std::thread::sleep(
            Duration::from_millis(20).min(until.saturating_duration_since(Instant::now())),
        );
    }
    true
}
fn speed(s: &DiscoverySettings) -> (usize, u64) {
    let (n, delay) = match s.speed {
        Speed::Gentle => (1, 1000),
        Speed::Balanced => (2, 350),
        Speed::Fast => (4, 0),
    };
    (usize::from(s.advanced.concurrency.unwrap_or(n)), delay)
}
fn retryable(e: &anyhow::Error) -> bool {
    matches!(
        e.downcast_ref::<WorkerFailure>().map(|e| e.0),
        Some(
            FailureCode::Timeout
                | FailureCode::ServerError
                | FailureCode::RateLimited
                | FailureCode::WorkerStopped
        )
    )
}
fn robots_for(
    worker: &mut Worker,
    job: RunId,
    id: &mut u64,
    url: &url::Url,
    cancel: Arc<AtomicBool>,
) -> Result<Robots> {
    let mut result = None;
    request(
        worker,
        job,
        id,
        Command::FetchRobots {
            address: url.to_string(),
        },
        cancel,
        12,
        &mut |event| {
            if let EventKind::RobotsFound { text, status } = event {
                ensure!(
                    [200, 404, 410].contains(&status),
                    "The website's robots.txt rules could not be read (HTTP {status}), so Regressee stopped visiting it. Only override these rules in Project settings if you have permission."
                );
                result = Some(Robots::parse(&text));
            }
            Ok(())
        },
    )?;
    result.context("robots.txt response missing")
}
impl Product {
    pub fn discover(
        &self,
        store: &Store,
        project: ProjectId,
        cancel: Arc<AtomicBool>,
        progress: &mut dyn FnMut(&str, u32, Option<u32>),
    ) -> Result<DiscoveryPreview> {
        progress("Checking website boundaries and login settings…", 0, None);
        let saved = store.load_project(project)?;
        let p = saved.project;
        p.validate()?;
        let environment = store.selected_environment(&p)?;
        let auth = self.auth_state(store, project, &environment)?;
        ensure!(
            store
                .selected_environment(&p)?
                .mappings
                .iter()
                .all(|m| m.source == m.destination),
            "Find pages on the source website. Use alternate environment mapping for fixed-baseline checks."
        );
        progress("Verifying the private browser for page discovery…", 0, None);
        let installed = self.resolve_runtime(&cancel)?;
        ensure!(
            worker_version(&installed)? == regressee_runtime::protocol::WORKER_VERSION,
            "The installed browser components cannot find pages with this app. Check the browser status in App settings before repairing or updating it. Saved screenshots are kept."
        );
        for u in &p.sources {
            scoped(&installed, u)?;
        }
        let job = RunId::new();
        progress("Starting the private browser…", 0, None);
        let mut worker = self.spawn_worker(&installed)?;
        worker.handshake(job, cancel.clone())?;
        if let Some(a) = &auth {
            progress(
                "Preparing your saved login session in the browser…",
                0,
                None,
            );
            a.install(&mut worker, job, cancel.clone())?;
        }
        let mut id = 1;
        let mut policies = BTreeMap::new();
        let mut notes = vec!["Only rendered links sampled after DOM readiness are explored. Delayed links, hidden application states and nested/virtualized content may contain additional pages. Queue completion is not an exhaustiveness guarantee.".into()];
        for source in &p.sources {
            if let std::collections::btree_map::Entry::Vacant(entry) =
                policies.entry(origin(source))
            {
                let policy = if p.discovery.respect_robots {
                    progress(
                        &format!(
                            "Checking website crawl rules (robots.txt) for {}…",
                            origin(source)
                        ),
                        0,
                        None,
                    );
                    robots_for(&mut worker, job, &mut id, source, cancel.clone())
                } else {
                    Ok(Robots::parse(""))
                };
                entry.insert(policy.map_err(|e| e.to_string()));
            }
        }
        if !p.discovery.respect_robots {
            notes.push("Authorized robots.txt override. Robots rules are not authorization and navigation can have server-side effects.".into());
        }
        let mut queue: VecDeque<_> = p.sources.iter().cloned().map(|u| (u, 0u32)).collect();
        let mut seen = BTreeSet::new();
        let mut pages = Vec::new();
        let mut aliases = Vec::new();
        let mut visited = 0;
        let mut limited = false;
        let mut last = BTreeMap::new();
        if p.discovery.mode == DiscoveryMode::OnlyAddresses {
            progress(
                "Reviewing the supplied page addresses — no links will be explored…",
                0,
                None,
            );
        }
        while let Some((raw, depth)) = queue.pop_front() {
            if cancel.load(Ordering::Acquire) {
                limited = true;
                notes.push("Discovery cancelled; the preview is incomplete.".into());
                break;
            }
            let Ok(u) = canonical(&raw, &p.discovery) else {
                continue;
            };
            if !seen.insert(u.to_string()) {
                continue;
            }
            if pages.len() >= MAX_CANDIDATES {
                notes.push(
                    "The page search stopped after checking 10,000 possible addresses.".into(),
                );
                limited = true;
                break;
            }
            let mut reason = exclusion(&u, &p, depth);
            if reason.is_none() {
                match policies.get(&origin(&u)) {
                    Some(Ok(r)) if !r.allows(&u) => reason = Some("Excluded by robots.txt".into()),
                    Some(Err(e)) => reason = Some(e.clone()),
                    _ => (),
                }
            }
            if reason.is_none()
                && pages
                    .iter()
                    .filter(|page: &&InventoryPage| page.included)
                    .count()
                    >= p.discovery.page_limit as usize
            {
                reason = Some("Page limit reached".into());
                limited = true;
            }
            let mut page = InventoryPage {
                id: PageId::new(),
                canonical: u.clone(),
                included: reason.is_none(),
                reason,
            };
            if page.included && p.discovery.mode == DiscoveryMode::AddressesAndLinks {
                let delay = speed(&p.discovery).1.max(
                    policies
                        .get(&origin(&u))
                        .and_then(|r| r.as_ref().ok())
                        .map(|r| r.delay_ms)
                        .unwrap_or(0),
                );
                if let Some(start) = last.get(&origin(&u)) {
                    let until: Instant = *start + Duration::from_millis(delay);
                    if let Some(message) = discovery_wait_message(
                        until.saturating_duration_since(Instant::now()),
                        visited,
                    ) {
                        progress(&message, visited, None);
                    }
                    wait(until, &cancel, &AtomicBool::new(false));
                }
                last.insert(origin(&u), Instant::now());
                let mut links = None;
                let mut attempts = 0;
                let result = loop {
                    attempts += 1;
                    progress(&discovery_loading_message(visited, attempts), visited, None);
                    let result = request(
                        &mut worker,
                        job,
                        &mut id,
                        if let Some(a) = &auth {
                            Command::DiscoverAuthenticated {
                                address: u.to_string(),
                                section: p.discovery.section.clone(),
                                session: a.scope.clone(),
                            }
                        } else {
                            Command::DiscoverPage {
                                address: u.to_string(),
                                section: p.discovery.section.clone(),
                            }
                        },
                        cancel.clone(),
                        36,
                        &mut |event| {
                            match event {
                                EventKind::LinksFound {
                                    addresses,
                                    truncated,
                                    final_url,
                                } => links = Some((addresses, truncated, final_url, false)),
                                EventKind::RedirectFound { address } => {
                                    links = Some((vec![], false, address, true))
                                }
                                _ => (),
                            }
                            Ok(())
                        },
                    );
                    if let Err(e) = &result
                        && retryable(e)
                        && attempts <= p.discovery.advanced.retries
                        && !cancel.load(Ordering::Acquire)
                    {
                        notes.push(format!("Temporary discovery failure on {u}; retry {attempts} after bounded backoff"));
                        progress(
                            &format!(
                                "The page could not be read. Waiting before retry {attempts} of {}…",
                                p.discovery.advanced.retries
                            ),
                            visited,
                            None,
                        );
                        wait(
                            Instant::now() + Duration::from_millis(1000 << attempts),
                            &cancel,
                            &cancel,
                        );
                        if matches!(
                            e.downcast_ref::<WorkerFailure>().map(|e| e.0),
                            Some(FailureCode::Timeout | FailureCode::WorkerStopped)
                        ) {
                            progress(
                                "Restarting the private browser before retrying the page…",
                                visited,
                                None,
                            );
                            let _ = worker.stop(Instant::now() + Duration::from_secs(2));
                            worker = self.spawn_worker(&installed)?;
                            worker.handshake(job, cancel.clone())?;
                            if let Some(a) = &auth {
                                a.install(&mut worker, job, cancel.clone())?;
                            }
                        }
                        continue;
                    }
                    break result;
                };
                visited += 1;
                match result {
                    Ok(()) => {
                        let (addresses, truncated, final_url, redirected) =
                            links.context("missing rendered links")?;
                        let final_url = canonical(&url::Url::parse(&final_url)?, &p.discovery)?;
                        if redirected && final_url == u {
                            page.included = false;
                            page.reason =
                                Some(format!("Redirect loop to {final_url} — not followed"));
                        } else if final_url != u {
                            if exclusion(&final_url, &p, depth).is_some()
                                || policies
                                    .get(&origin(&final_url))
                                    .and_then(|r| r.as_ref().ok())
                                    .is_none_or(|r| !r.allows(&final_url))
                            {
                                page.included = false;
                                page.reason = Some(format!(
                                    "Redirect destination {final_url} is outside the approved policy — not visited"
                                ));
                            } else {
                                aliases.push((u.clone(), final_url.clone()));
                                page.included = false;
                                page.reason = Some(format!("Redirect alias of {final_url}"));
                                queue.push_back((final_url, depth));
                            }
                        }
                        if truncated {
                            limited = true;
                            notes.push(format!("Rendered links truncated at 2,000 on {u}"));
                        }
                        for address in addresses {
                            if queue.len() + seen.len() >= MAX_CANDIDATES * 2 {
                                limited = true;
                                break;
                            }
                            if let Ok(link) = url::Url::parse(&address) {
                                queue.push_back((link, depth + 1));
                            }
                        }
                    }
                    Err(e) => {
                        if map_failure(&e) == CaptureError::Cancelled {
                            notes.push("Discovery cancelled; the preview is incomplete.".into());
                        }
                        if map_failure(&e) == CaptureError::NeedsLogin {
                            return Err(e).context(
                                "Log in again. Discovery stopped without adding login pages.",
                            );
                        }
                        limited = true;
                        page.included = false;
                        page.reason = Some(format!(
                            "Could not inspect page: {}",
                            map_failure(&e).label()
                        ));
                    }
                }
            }
            pages.push(page);
            if p.discovery.mode == DiscoveryMode::AddressesAndLinks && visited > 0 {
                progress(&discovery_loading_message(visited, 1), visited, None);
            }
        }
        if pages.iter().any(|p| !p.included) {
            limited = true;
        }
        if p.discovery.mode == DiscoveryMode::OnlyAddresses {
            notes.push(
                "Only the starting addresses were checked; no links were followed. The website's robots.txt rules still apply."
                    .into(),
            );
        }
        if p.discovery.advanced.max_depth.is_some() {
            notes.push("Your link-depth limit may leave pages further from the starting addresses out of this list.".into());
            limited = true;
        }
        progress(
            "Finishing the page list and closing the discovery browser…",
            visited,
            None,
        );
        worker.stop(Instant::now() + Duration::from_secs(5))?;
        Ok(DiscoveryPreview {
            project_revision: saved.revision,
            settings: p,
            inventory: InventoryRevision {
                id: InventoryId::new(),
                project,
                identity_version: 2,
                pages,
                coverage: if limited {
                    DiscoveryCoverage::Limited { reason: "Some pages were excluded, could not be opened or were not checked before a limit or stop. Other website pages may be missing.".into() }
                } else {
                    DiscoveryCoverage::Complete
                },
            },
            aliases,
            notes,
            visited,
        })
    }

    // Explicit user intent and atomic controls are separate from frozen settings.
    #[allow(clippy::too_many_arguments)]
    pub fn capture_inventory(
        &self,
        store: &mut Store,
        project: ProjectId,
        preview: Option<DiscoveryPreview>,
        check: bool,
        resume: Option<RunId>,
        cancel: Arc<AtomicBool>,
        pause: Arc<AtomicBool>,
        progress: &mut dyn FnMut(&str, u32, Option<u32>),
    ) -> Result<RunRecord> {
        self.capture_inventory_inner(
            store,
            None,
            project,
            preview,
            check,
            resume,
            cancel,
            pause,
            progress,
            false,
            &mut |_| {},
        )
    }
    /// Native task presentation consumes real counts and frozen setup without
    /// changing the public progress API or reading evidence on the UI thread.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn capture_inventory_with_status(
        &self,
        store: &mut Store,
        verified_runtime: Option<Installed>,
        project: ProjectId,
        preview: Option<DiscoveryPreview>,
        check: bool,
        resume: Option<RunId>,
        cancel: Arc<AtomicBool>,
        pause: Arc<AtomicBool>,
        progress: &mut dyn FnMut(&str, u32, Option<u32>),
        status: &mut dyn FnMut(&CaptureRun),
    ) -> Result<RunRecord> {
        self.capture_inventory_inner(
            store,
            verified_runtime,
            project,
            preview,
            check,
            resume,
            cancel,
            pause,
            progress,
            false,
            status,
        )
    }
    pub fn preview_images(
        &self,
        store: &mut Store,
        project: ProjectId,
        preview: DiscoveryPreview,
        cancel: Arc<AtomicBool>,
    ) -> Result<RunRecord> {
        self.preview_images_with_progress(store, project, preview, cancel, &mut |_, _, _| {})
    }
    pub fn preview_images_with_progress(
        &self,
        store: &mut Store,
        project: ProjectId,
        mut preview: DiscoveryPreview,
        cancel: Arc<AtomicBool>,
        progress: &mut dyn FnMut(&str, u32, Option<u32>),
    ) -> Result<RunRecord> {
        let first = preview
            .inventory
            .pages
            .iter()
            .find(|p| p.included)
            .context("Include a representative page first")?
            .id;
        for page in &mut preview.inventory.pages {
            page.included = page.id == first;
        }
        preview.inventory.id = InventoryId::new();
        self.capture_inventory_inner(
            store,
            None,
            project,
            Some(preview),
            false,
            None,
            cancel,
            Arc::new(AtomicBool::new(false)),
            progress,
            true,
            &mut |_| {},
        )
    }
    #[allow(clippy::too_many_arguments)]
    fn capture_inventory_inner(
        &self,
        store: &mut Store,
        verified_runtime: Option<Installed>,
        project: ProjectId,
        preview: Option<DiscoveryPreview>,
        check: bool,
        resume: Option<RunId>,
        cancel: Arc<AtomicBool>,
        pause: Arc<AtomicBool>,
        progress: &mut dyn FnMut(&str, u32, Option<u32>),
        preview_only: bool,
        status: &mut dyn FnMut(&CaptureRun),
    ) -> Result<RunRecord> {
        let installed = self.runtime_for_capture(verified_runtime, &cancel)?;
        ensure!(
            worker_version(&installed)? == regressee_runtime::protocol::WORKER_VERSION,
            "The installed browser components cannot capture logged-in pages with this app. Check the browser status in App settings before repairing or updating it. Older baselines still require their original browser version or a new baseline capture."
        );
        let saved = store.load_project(project)?;
        let mut record = if let Some(run) = resume {
            let mut r = store.load_run(run)?;
            ensure!(
                r.run.project == project
                    && matches!(r.run.state, JobState::Paused | JobState::Interrupted),
                "Only paused or interrupted queues can resume"
            );
            let queue = r
                .queue
                .as_mut()
                .context("This older single-page run has no resumable queue")?;
            ensure!(
                queue.project_revision == saved.revision
                    && (matches!(r.run.intent, RunIntent::Check { .. })
                        || queue.discovery == saved.project.discovery),
                "Project settings changed. Saved screenshots are kept. Start a new capture."
            );
            ensure!(
                r.run
                    .profiles
                    .iter()
                    .all(|p| p.runtime == installed.manifest.identity),
                "To resume, use the same browser version as the original capture."
            );
            ensure!(
                r.run.rendering.os == std::env::consts::OS
                    && r.run.rendering.architecture == std::env::consts::ARCH,
                "Rendering computer differs; start a compatible fresh baseline"
            );
            ensure!(
                r.run.rendering == crate::product::rendering_environment(&installed)?,
                "The computer, browser settings or fonts changed. Saved screenshots are kept. Start a new capture."
            );
            for target in &r.run.targets {
                scoped(&installed, &target.requested)?;
            }
            preflight(
                store,
                r.run
                    .targets
                    .iter()
                    .filter(|t| target_pending(t))
                    .count()
                    .min(8),
                false,
                &r.run.capture_plan,
            )?;
            queue.resumed_at.push(distribution::now());
            self.auth_state(store, project, &r.run.environment)?;
            for target in &mut r.run.targets {
                if target.outcome
                    == (TargetOutcome::Failed {
                        code: CaptureError::NeedsLogin,
                    })
                    || target.dom_outcome
                        == (DomTargetOutcome::Failed {
                            code: DomCaptureError::Capture {
                                code: CaptureError::NeedsLogin,
                            },
                        })
                {
                    ensure!(
                        target.attempts < 8,
                        "Login still failed after retrying. Log in again and start a new capture. Saved screenshots are kept."
                    );
                    if matches!(
                        target.outcome,
                        TargetOutcome::Failed {
                            code: CaptureError::NeedsLogin
                        }
                    ) {
                        target.outcome = TargetOutcome::Pending;
                    }
                    if matches!(
                        target.dom_outcome,
                        DomTargetOutcome::Failed {
                            code: DomCaptureError::Capture {
                                code: CaptureError::NeedsLogin
                            }
                        }
                    ) {
                        target.dom_outcome = DomTargetOutcome::Pending;
                    }
                    target
                        .warnings
                        .push("ResumedAfterLogin — capture times may differ".into());
                }
            }
            r.run.state = JobState::Running;
            r.run.finished_at = None;
            store.resume_run(&r)?;
            r
        } else {
            let (inventory, mut profiles, capture_plan, intent, environment, rendering, baseline) =
                if check {
                    let (baseline, before) = store.frozen_baseline_source(
                        saved
                            .project
                            .active_baseline
                            .context("Save a baseline before checking")?,
                    )?;
                    (
                        before.run.inventory.clone(),
                        before.run.profiles.clone(),
                        baseline.capture_plan.clone(),
                        RunIntent::Check {
                            baseline: baseline.id,
                        },
                        store.selected_environment(&saved.project)?,
                        crate::product::rendering_environment(&installed)?,
                        Some(baseline),
                    )
                } else {
                    let preview = preview
                        .context("Choose Find pages and review the page list before capturing.")?;
                    ensure!(
                        preview.project_revision == saved.revision
                            && preview.settings == saved.project
                            && preview.inventory.project == project,
                        "Settings changed after discovery. Find pages again."
                    );
                    preview.inventory.validate()?;
                    ensure!(
                        preview.included() > 0
                            && preview.included() <= saved.project.discovery.page_limit as usize,
                        "Include at least one page within the page limit"
                    );
                    for page in preview.inventory.pages.iter().filter(|p| p.included) {
                        ensure!(
                            exclusion(&page.canonical, &saved.project, 0).is_none(),
                            "Selected page is outside the saved scope"
                        );
                    }
                    store.freeze_inventory(&preview.inventory)?;
                    (
                        preview.inventory,
                        saved.project.profiles.clone(),
                        if preview_only {
                            CapturePlan::default()
                        } else {
                            saved.project.capture_plan.clone()
                        },
                        if preview_only {
                            RunIntent::RepresentativePreview
                        } else {
                            RunIntent::NewBaseline
                        },
                        store.selected_environment(&saved.project)?,
                        crate::product::rendering_environment(&installed)?,
                        None,
                    )
                };
            if check {
                ensure!(
                    profiles
                        .iter()
                        .all(|p| p.runtime == installed.manifest.identity),
                    "This baseline needs the browser version used to capture it."
                );
            } else {
                for p in &mut profiles {
                    crate::product::configure_profile_for_plan(p, &installed, &capture_plan)?;
                }
            }
            self.auth_state(store, project, &environment)?;
            if check {
                self.require_probe(store, project, &environment)?;
            }
            let targets = capture_targets(&inventory, &profiles, &capture_plan, baseline.as_ref());
            let frozen_discovery = if let RunIntent::Check { baseline } = &intent {
                store
                    .frozen_baseline_source(*baseline)?
                    .1
                    .queue
                    .map(|q| q.discovery)
                    .unwrap_or_default()
            } else {
                saved.project.discovery.clone()
            };
            let r = RunRecord {
                run: CaptureRun {
                    id: RunId::new(),
                    project,
                    intent,
                    inventory,
                    profiles,
                    capture_plan,
                    environment,
                    rendering,
                    state: JobState::Running,
                    started_at: Some(distribution::now()),
                    finished_at: None,
                    targets,
                },
                capture: None,
                captures: vec![],
                dom_captures: vec![],
                queue: Some(QueueSettings {
                    discovery: frozen_discovery,
                    project_revision: saved.revision,
                    resumed_at: vec![],
                }),
            };
            r.run.validate()?;
            if let RunIntent::Check { baseline } = r.run.intent {
                let (b, _) = store.frozen_baseline_source(baseline)?;
                b.validate_check_slots(&r.run)?;
            }
            for target in &r.run.targets {
                scoped(
                    &installed,
                    &regressee_core::environment::guarded_map(
                        &target.requested,
                        &r.run.environment,
                        &r.queue.as_ref().unwrap().discovery,
                    )?,
                )?;
            }
            ensure!(
                r.run.rendering.os == std::env::consts::OS
                    && r.run.rendering.architecture == std::env::consts::ARCH,
                "Rendering computer differs; start a compatible fresh baseline"
            );
            preflight(store, r.run.targets.len(), true, &r.run.capture_plan)?;
            store.begin_run(&r)?;
            r
        };
        ensure!(
            record.run.rendering.os == std::env::consts::OS
                && record.run.rendering.architecture == std::env::consts::ARCH,
            "Rendering computer differs; start a compatible fresh baseline"
        );
        for t in &record.run.targets {
            scoped(
                &installed,
                &regressee_core::environment::guarded_map(
                    &t.requested,
                    &record.run.environment,
                    &record.queue.as_ref().unwrap().discovery,
                )?,
            )?;
        }
        let result = self.execute_queue(
            store,
            &installed,
            &mut record,
            cancel.clone(),
            pause.clone(),
            progress,
            status,
        );
        if let Err(e) = result {
            // Persist the stopped queue, never relabel missing evidence unchanged.
            record.run.state = if cancel.load(Ordering::Acquire) {
                JobState::Cancelled
            } else {
                JobState::Interrupted
            };
            if record.run.state == JobState::Cancelled {
                for t in &mut record.run.targets {
                    cancel_pending(t);
                }
            }
            record.run.finished_at = Some(distribution::now());
            store.finish_run(&record)?;
            return Err(e);
        }
        if record.run.state == JobState::Paused {
            return Ok(record);
        }
        if let RunIntent::Check { baseline } = record.run.intent
            && !cancel.load(Ordering::Acquire)
        {
            self.analyze_queue(store, &record, baseline, &cancel, progress)?;
        }
        Ok(record)
    }

    #[allow(clippy::too_many_arguments)]
    fn execute_queue(
        &self,
        store: &mut Store,
        installed: &Installed,
        record: &mut RunRecord,
        cancel: Arc<AtomicBool>,
        pause: Arc<AtomicBool>,
        progress: &mut dyn FnMut(&str, u32, Option<u32>),
        status: &mut dyn FnMut(&CaptureRun),
    ) -> Result<()> {
        let settings = record.queue.as_ref().unwrap().discovery.clone();
        let auth = self.auth_state(store, record.run.project, &record.run.environment)?;
        let (parallel, spacing) = speed(&settings);
        progress(
            "Preparing the browser for the selected pages…",
            crate::task::CaptureStatus::from_run(&record.run).processed(),
            Some(record.run.targets.len() as u32),
        );
        status(&record.run);
        let mut workers = Vec::new();
        for _ in 0..parallel.min(record.run.targets.len()) {
            if pause.load(Ordering::Acquire) || cancel.load(Ordering::Acquire) {
                break;
            }
            let mut w = self.spawn_worker(installed)?;
            let (version, browser) = w.handshake(record.run.id, cancel.clone())?;
            ensure!(
                version == record.run.rendering.worker_version
                    && browser == record.run.rendering.browser,
                "worker identity differs"
            );
            if let Some(a) = &auth {
                a.install(&mut w, record.run.id, cancel.clone())?;
            }
            workers.push(w);
        }
        if pause.load(Ordering::Acquire) || cancel.load(Ordering::Acquire) {
            for worker in &mut workers {
                let _ = worker.stop(Instant::now() + Duration::from_secs(3));
            }
            if cancel.load(Ordering::Acquire) {
                record.run.state = JobState::Cancelled;
                for target in &mut record.run.targets {
                    cancel_pending(target);
                }
                record.run.finished_at = Some(distribution::now());
                store.finish_run(record)?;
            } else {
                record.run.state = JobState::Paused;
                store.checkpoint_run(record)?;
            }
            return Ok(());
        }
        let mut id = 1;
        let mut robots = BTreeMap::new();
        // Policy is probed again before a fresh capture or resume. Checks do not
        // discover links; robots access is policy, not added inventory.
        for t in &record.run.targets {
            if pause.load(Ordering::Acquire) {
                break;
            }
            let mapped =
                regressee_core::environment::map_url(&t.requested, &record.run.environment)?;
            let o = origin(&mapped);
            if let std::collections::btree_map::Entry::Vacant(entry) = robots.entry(o) {
                entry.insert(if settings.respect_robots {
                    robots_for(
                        &mut workers[0],
                        record.run.id,
                        &mut id,
                        &mapped,
                        cancel.clone(),
                    )
                    .map_err(|e| e.to_string())
                } else {
                    Ok(Robots::parse(""))
                });
            }
        }
        let mut last: BTreeMap<String, Instant> = BTreeMap::new();
        let mut checkpoint = store.begin_run_append(record.run.id)?;
        loop {
            if cancel.load(Ordering::Acquire) {
                for t in &mut record.run.targets {
                    cancel_pending(t);
                }
                record.run.state = JobState::Cancelled;
                break;
            }
            if pause.load(Ordering::Acquire) {
                record.run.state = JobState::Paused;
                store.checkpoint_run(record)?;
                break;
            }
            let pending: Vec<_> = record
                .run
                .targets
                .iter()
                .enumerate()
                .filter(|(_, t)| target_pending(t))
                .map(|(i, _)| i)
                .collect();
            if pending.is_empty() {
                record.run.state = if record
                    .run
                    .targets
                    .iter()
                    .all(CaptureTarget::requested_kinds_captured)
                {
                    JobState::Completed
                } else {
                    JobState::CompletedWithErrors
                };
                break;
            }
            preflight(store, workers.len(), false, &record.run.capture_plan)?;
            let mut selected = Vec::new();
            let mut changed = Vec::new();
            let mut origin_count = BTreeMap::new();
            for i in pending {
                let t = &mut record.run.targets[i];
                let mapped =
                    regressee_core::environment::map_url(&t.requested, &record.run.environment)?;
                let o = origin(&mapped);
                match robots.get(&o) {
                    Some(Ok(r)) if !r.allows(&mapped) => {
                        skip_pending(t, "robots.txt now excludes this saved target");
                        changed.push(i);
                        continue;
                    }
                    Some(Err(e)) => {
                        skip_pending(t, e);
                        changed.push(i);
                        continue;
                    }
                    _ => (),
                }
                let count = origin_count.entry(o.clone()).or_insert(0u8);
                if *count >= settings.advanced.per_origin {
                    continue;
                }
                let delay = spacing.max(
                    robots
                        .get(&o)
                        .and_then(|r| r.as_ref().ok())
                        .map(|r| r.delay_ms)
                        .unwrap_or(0),
                );
                // Start spacing is authoritative, not merely a gap while
                // assembling a batch. Spaced origins use one slot per batch.
                if delay > 0 && *count > 0 {
                    continue;
                }
                if let Some(start) = last.get(&o)
                    && !wait(*start + Duration::from_millis(delay), &cancel, &pause)
                {
                    break;
                }
                last.insert(o, Instant::now());
                *count += 1;
                t.attempts += 1;
                changed.push(i);
                selected.push(i);
                if selected.len() == workers.len() {
                    break;
                }
            }
            checkpoint_target_metadata(store, &mut checkpoint, record, &changed)?;
            if selected.is_empty() {
                continue;
            }
            let accounted = record
                .run
                .targets
                .iter()
                .filter(|t| !target_pending(t))
                .count();
            progress(
                capture_message(&record.run.capture_plan),
                accounted as u32,
                Some(record.run.targets.len() as u32),
            );
            let tasks: Vec<_> = selected
                .iter()
                .map(|i| {
                    let t = record.run.targets[*i].clone();
                    let p = record
                        .run
                        .profiles
                        .iter()
                        .find(|p| p.id == t.key.profile)
                        .unwrap()
                        .clone();
                    let address =
                        regressee_core::environment::map_url(&t.requested, &record.run.environment);
                    (t, p, address)
                })
                .collect();
            id += 2;
            let request_id = id;
            let job = record.run.id;
            let stage = crate::capture_evidence::private_staging(&store.paths().cache)?;
            let stage_path = stage.path().canonicalize()?;
            let plan = record.run.capture_plan.clone();
            let mut backoff = 0;
            let results = std::thread::scope(|scope| -> Result<Vec<_>> {
                // A bounded stream, not join-then-ingest: a successful publication
                // is detached and durably checkpointed while requests are live.
                let (tx, rx) = std::sync::mpsc::sync_channel(workers.len().max(1));
                let mut handles = Vec::new();
                for (slot, (worker, (target, profile, address))) in
                    workers.iter_mut().zip(tasks).enumerate()
                {
                    let index = selected[slot];
                    let cancel = cancel.clone();
                    let session = auth.as_ref().map(|a| a.scope.clone());
                    let tx = tx.clone();
                    let plan = plan.clone();
                    let staging = stage_path.as_path();
                    handles.push((
                        slot,
                        index,
                        scope.spawn(move || {
                            capture_one(
                                worker,
                                job,
                                request_id,
                                &target,
                                &profile,
                                installed,
                                cancel,
                                &address?,
                                session,
                                &plan,
                                staging,
                                &mut |kind, channel| {
                                    let (acknowledge, received) = std::sync::mpsc::sync_channel(1);
                                    tx.send((index, kind, channel, acknowledge)).map_err(|_| {
                                        anyhow::anyhow!("capture checkpoint owner stopped")
                                    })?;
                                    // Do not let later sibling work destroy scratch
                                    // before storage has accepted this publication.
                                    ensure!(
                                        received.recv().unwrap_or(false),
                                        "capture checkpoint was not saved"
                                    );
                                    Ok(())
                                },
                            )
                        }),
                    ));
                }
                drop(tx);
                let mut failure = None;
                for (index, kind, channel, acknowledge) in rx {
                    let result = publish_channel(
                        store,
                        record,
                        &mut checkpoint,
                        index,
                        kind,
                        channel,
                        settings.advanced.retries,
                        &mut backoff,
                        &pause,
                    );
                    let _ = acknowledge.send(result.is_ok());
                    if let Err(error) = result {
                        failure.get_or_insert(error);
                    }
                    status(&record.run);
                }
                let results = handles
                    .into_iter()
                    .map(|(slot, index, h)| {
                        (
                            slot,
                            index,
                            h.join()
                                .unwrap_or_else(|_| Err(anyhow::anyhow!("browser task panicked"))),
                        )
                    })
                    .collect::<Vec<_>>();
                if let Some(error) = failure {
                    return Err(error);
                }
                Ok(results)
            })?;
            for (slot, index, result) in results {
                match result {
                    Ok(()) => (),
                    Err(e) => {
                        let t = &mut record.run.targets[index];
                        if cancel.load(Ordering::Acquire) {
                            cancel_pending(t);
                        } else if target_pending(t)
                            && retryable(&e)
                            && t.attempts <= settings.advanced.retries
                        {
                            backoff = backoff.max(1000u64 << t.attempts.min(5));
                            t.warnings.push(format!(
                                "Retry {} after {:?}",
                                t.attempts,
                                map_failure(&e)
                            ));
                        } else {
                            if map_failure(&e) == CaptureError::NeedsLogin {
                                pause.store(true, Ordering::Release);
                            }
                            fail_pending(t, map_failure(&e));
                        }
                        // A deadline or crash invalidates this worker; replace it
                        // only after its owned tree is fully dropped and reaped.
                        if matches!(
                            e.downcast_ref::<WorkerFailure>().map(|e| e.0),
                            Some(FailureCode::Timeout | FailureCode::WorkerStopped)
                        ) && !cancel.load(Ordering::Acquire)
                        {
                            let _ = workers[slot].stop(Instant::now() + Duration::from_secs(2));
                            let mut replacement = self.spawn_worker(installed)?;
                            replacement.handshake(job, cancel.clone())?;
                            if let Some(a) = &auth {
                                a.install(&mut replacement, job, cancel.clone())?;
                            }
                            workers[slot] = replacement;
                        }
                    }
                }
                let target = &record.run.targets[index];
                store.append_run_kind_targets(
                    &mut checkpoint,
                    &[regressee_storage::KindTargetCheckpoint {
                        index,
                        target,
                        capture: None,
                        dom_capture: None,
                    }],
                )?;
            }
            let counts = crate::task::target_counts(&record.run);
            progress(
                capture_message(&record.run.capture_plan),
                (record.run.targets.len() - counts.pending) as u32,
                Some(record.run.targets.len() as u32),
            );
            status(&record.run);
            if backoff > 0 {
                wait(
                    Instant::now() + Duration::from_millis(backoff),
                    &cancel,
                    &pause,
                );
            }
        }
        for w in &mut workers {
            let _ = w.stop(Instant::now() + Duration::from_secs(3));
        }
        if record.run.state != JobState::Paused {
            record.run.finished_at = Some(distribution::now());
            store.finish_run(record)?;
        }
        Ok(())
    }
    fn analyze_queue(
        &self,
        store: &mut Store,
        record: &RunRecord,
        baseline: BaselineId,
        cancel: &AtomicBool,
        progress: &mut dyn FnMut(&str, u32, Option<u32>),
    ) -> Result<()> {
        let _ = baseline;
        progress(
            if record.run.capture_plan.mode.screenshots() {
                "Analyzing saved originals with Balanced screenshot sensitivity and exact DOM comparison where requested"
            } else {
                "Comparing saved DOM snapshots exactly"
            },
            0,
            None,
        );
        crate::review::analyze_saved_run(
            store,
            record.run.id,
            crate::review::preset(crate::analysis_engine::Preset::Balanced),
            cancel,
        )?;
        Ok(())
    }
}
fn discovery_loading_message(visited: u32, attempt: u8) -> String {
    let page_word = if visited == 1 { "page" } else { "pages" };
    if attempt > 1 {
        format!("Retrying the page and looking for links · {visited} {page_word} visited…")
    } else if visited == 0 {
        "Loading the first page and looking for links…".into()
    } else {
        format!("Finding pages · {visited} {page_word} visited…")
    }
}
fn discovery_wait_message(remaining: Duration, visited: u32) -> Option<String> {
    // Routine sub-second pacing (including Gentle's one-second interval) is
    // part of finding pages, not a new user-visible phase on every visit. Keep
    // genuine longer crawl-rule waits visible without changing their timing.
    (remaining >= Duration::from_secs(2)).then(|| {
        let page_word = if visited == 1 { "page" } else { "pages" };
        format!(
            "Waiting between page visits to respect website crawl limits · {visited} {page_word} visited…"
        )
    })
}
fn preflight(store: &Store, targets: usize, initial: bool, plan: &CapturePlan) -> Result<()> {
    space_check_for_plan(
        distribution::available_bytes(&store.paths().cache)?,
        targets,
        initial,
        plan,
    )?;
    space_check_for_plan(
        distribution::available_bytes(&store.paths().data)?,
        targets,
        initial,
        plan,
    )
}
#[cfg(test)]
fn space_check(available: u64, targets: usize, initial: bool) -> Result<()> {
    space_check_for_plan(available, targets, initial, &CapturePlan::default())
}
fn space_check_for_plan(
    available: u64,
    targets: usize,
    initial: bool,
    plan: &CapturePlan,
) -> Result<()> {
    let required = 256 * 1024 * 1024
        + targets as u64
            * if !plan.mode.screenshots() {
                // Complete canonical originals plus app-owned detached copies;
                // no image-area/height or tile/thumbnail reservation in DOM-only.
                2 * regressee_core::dom_snapshot::MAX_DOM_BYTES as u64
            } else if initial {
                1024 * 1024
            } else {
                384 * 1024 * 1024
            };
    ensure!(
        available >= required,
        "Low disk space: free local space and resume this queue. Existing originals and baseline are preserved."
    );
    Ok(())
}
#[allow(clippy::too_many_arguments)]
pub(crate) fn capture_one(
    worker: &mut Worker,
    job: RunId,
    id: u64,
    target: &CaptureTarget,
    profile: &CaptureProfile,
    installed: &Installed,
    cancel: Arc<AtomicBool>,
    address: &url::Url,
    session: Option<SessionReference>,
    frozen_plan: &CapturePlan,
    staging: &std::path::Path,
    publish: &mut dyn FnMut(CheckKind, Result<PublishedChannel>) -> Result<()>,
) -> Result<()> {
    use regressee_runtime::supervisor::DetachedMetadata;
    ensure!(
        profile.runtime == installed.manifest.identity,
        "capture profile/runtime mismatch"
    );
    // Retry/resume only missing saved slots. A successful sibling is immutable
    // and must not be recaptured, including screenshot transformations in DOM-only.
    let plan = pending_plan(target, frozen_plan)?;
    let deadline =
        Instant::now() + regressee_runtime::protocol::capture_request_budget(profile, &plan)?;
    let mut screenshot_token = None;
    let result = worker.request_detached(
        Request {
            version: regressee_runtime::protocol::PROTOCOL_VERSION,
            request_id: id,
            job,
            command: Command::CaptureTarget {
                key: target.key.clone(),
                address: address.to_string(),
                profile: Box::new(profile.clone()),
                session,
                plan,
            },
        },
        deadline,
        cancel,
        staging,
        &mut |e, detached| {
            match e.event {
                EventKind::ChannelCaptured { key, capture } => {
                    ensure!(key == target.key, "foreign captured target");
                    let detached = detached.context("capture publication was not detached")?;
                    ensure!(
                        detached.key == key && detached.capture.kind() == capture.kind(),
                        "detached capture channel mismatch"
                    );
                    let artifact = capture.artifact().clone();
                    let path = detached.artifact_path;
                    let channel = match detached.metadata {
                        DetachedMetadata::Screenshot(metadata) => {
                            ensure!(
                                capture.kind() == CheckKind::Screenshot,
                                "foreign screenshot metadata"
                            );
                            screenshot_token = Some(capture.output_token());
                            PublishedChannel::Screenshot {
                                metadata,
                                artifact,
                                path: path.clone(),
                            }
                        }
                        DetachedMetadata::DomSnapshot(metadata) => {
                            ensure!(
                                capture.kind() == CheckKind::DomSnapshot,
                                "foreign DOM metadata"
                            );
                            PublishedChannel::DomSnapshot {
                                metadata,
                                artifact,
                                path: path.clone(),
                            }
                        }
                    };
                    // Runtime validates/detaches before this callback. The
                    // blocking owner acknowledges only after durable checkpoint.
                    publish(capture.kind(), Ok(channel))?;
                    let _ = std::fs::remove_file(path);
                    let _ = std::fs::remove_file(detached.metadata_path);
                }
                EventKind::ChannelFailed {
                    key,
                    kind,
                    code,
                    retryable,
                } => {
                    ensure!(key == target.key, "foreign failed target");
                    let failure = anyhow::Error::new(WorkerFailure(code));
                    let channel = match kind {
                        CheckKind::Screenshot => PublishedChannel::ScreenshotFailed {
                            code: map_failure(&failure),
                            retryable,
                        },
                        CheckKind::DomSnapshot => PublishedChannel::DomSnapshotFailed {
                            code: match code {
                                FailureCode::DomLimit => DomCaptureError::LimitExceeded,
                                FailureCode::DomSerialization => {
                                    DomCaptureError::SerializationFailed
                                }
                                _ => DomCaptureError::Capture {
                                    code: map_failure(&failure),
                                },
                            },
                            retryable,
                        },
                    };
                    publish(kind, Ok(channel))?;
                }
                // Typed per-kind warnings and independent stability are checked
                // against each channel's sidecar by runtime's detached reader.
                _ => (),
            }
            Ok(())
        },
    );
    if result.is_ok()
        && let Some(token) = screenshot_token
        && let Some(directory) = regressee_runtime::capture_diagnostics::trace_directory()
    {
        let source = worker.output_path(token).with_extension("trace.json");
        regressee_runtime::protocol::retain_capture_trace(&directory, &source, job, token)?;
        std::fs::remove_file(source)?;
    }
    result
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn dom_only_scheduler_attempt_checkpoint_and_resume_preserve_not_requested_screenshot()
    -> Result<()> {
        let root = tempfile::tempdir()?;
        let paths = regressee_storage::AppPaths::isolated(root.path());
        let mut store = Store::open(paths.clone())?;
        let mut project = Project::draft("DOM-only attempts".into(), "http://127.0.0.1:9/inert")?;
        project.capture_plan = CapturePlan::for_mode(CaptureMode::DomSnapshotsOnly);
        project.profiles[0].runtime = RuntimeIdentity::Pinned {
            node: "inert".into(),
            worker: "inert".into(),
            playwright: "inert".into(),
            browser: "inert".into(),
        };
        let saved = store.save_project(&project, None)?;
        let inventory = InventoryRevision {
            id: InventoryId::new(),
            project: project.id,
            identity_version: 2,
            pages: vec![InventoryPage {
                id: PageId::new(),
                canonical: project.sources[0].clone(),
                included: true,
                reason: None,
            }],
            coverage: DiscoveryCoverage::Complete,
        };
        store.freeze_inventory(&inventory)?;
        let targets = capture_targets(&inventory, &project.profiles, &project.capture_plan, None);
        let mut record = RunRecord {
            run: CaptureRun {
                id: RunId::new(),
                project: project.id,
                intent: RunIntent::NewBaseline,
                inventory,
                profiles: project.profiles.clone(),
                capture_plan: project.capture_plan.clone(),
                environment: project.environments[0].clone(),
                rendering: RenderingEnvironment {
                    os: "inert".into(),
                    architecture: "inert".into(),
                    browser: "inert".into(),
                    worker_version: "inert".into(),
                    profile_version: 1,
                    fonts_fingerprint: None,
                },
                state: JobState::Running,
                started_at: Some(1),
                finished_at: None,
                targets,
            },
            capture: None,
            captures: vec![],
            dom_captures: vec![],
            queue: Some(QueueSettings {
                discovery: project.discovery.clone(),
                project_revision: saved.revision,
                resumed_at: vec![],
            }),
        };
        store.begin_run(&record)?;
        let mut checkpoint = store.begin_run_append(record.run.id)?;
        record.run.targets[0].attempts = 1;
        // The same metadata boundary used by execute_queue before dispatching
        // any worker. The legacy screenshot-only append rejects this target.
        checkpoint_target_metadata(&mut store, &mut checkpoint, &record, &[0])?;
        let saved = store.load_run(record.run.id)?;
        assert_eq!(saved.run.targets[0].outcome, TargetOutcome::NotRequested);
        assert_eq!(saved.run.targets[0].dom_outcome, DomTargetOutcome::Pending);
        assert_eq!(saved.run.targets[0].attempts, 1);
        record.run.state = JobState::Interrupted;
        record.run.finished_at = Some(2);
        store.finish_run(&record)?;
        drop(store);
        let mut store = Store::open(paths)?;
        let mut record = store.load_run(record.run.id)?;
        record.run.state = JobState::Running;
        record.run.finished_at = None;
        record.queue.as_mut().unwrap().resumed_at.push(3);
        store.resume_run(&record)?;
        let mut checkpoint = store.begin_run_append(record.run.id)?;
        record.run.targets[0].attempts += 1;
        checkpoint_target_metadata(&mut store, &mut checkpoint, &record, &[0])?;
        skip_pending(&mut record.run.targets[0], "synthetic policy stop");
        checkpoint_target_metadata(&mut store, &mut checkpoint, &record, &[0])?;
        record.run.state = JobState::CompletedWithErrors;
        record.run.finished_at = Some(4);
        store.finish_run(&record)?;
        let saved = store.load_run(record.run.id)?;
        assert_eq!(saved.run.targets[0].outcome, TargetOutcome::NotRequested);
        assert!(matches!(
            saved.run.targets[0].dom_outcome,
            DomTargetOutcome::Skipped { .. }
        ));
        assert_eq!(saved.run.targets[0].attempts, 2);
        assert_eq!(saved.all_captures().count(), 0);
        assert!(saved.dom_captures.is_empty());
        Ok(())
    }
    #[test]
    fn resume_plan_requests_only_the_pending_saved_kind_without_rewriting_frozen_mode() -> Result<()>
    {
        let project = Project::draft("Pending-kind fixture".into(), "http://127.0.0.1:9/")?;
        let frozen = CapturePlan::for_mode(CaptureMode::ScreenshotsAndDomSnapshots);
        let mut target = CaptureTarget {
            key: TargetKey {
                page: PageId::new(),
                profile: project.profiles[0].id,
            },
            requested: project.sources[0].clone(),
            final_url: None,
            attempts: 1,
            outcome: TargetOutcome::Pending,
            dom_outcome: DomTargetOutcome::Pending,
            warnings: vec![],
        };
        assert_eq!(pending_plan(&target, &frozen)?, frozen);
        target.outcome = TargetOutcome::Captured {
            artifact: "a".repeat(64).try_into()?,
            width: 1,
            height: 1,
        };
        assert_eq!(
            pending_plan(&target, &frozen)?.mode,
            CaptureMode::DomSnapshotsOnly
        );
        assert_eq!(
            pending_plan(&target, &frozen)?.dom_policy,
            frozen.dom_policy
        );
        assert!(pending_plan(&target, &CapturePlan::default()).is_err());
        target.dom_outcome = DomTargetOutcome::Captured {
            artifact: "b".repeat(64).try_into()?,
        };
        assert!(pending_plan(&target, &frozen).is_err());
        target.outcome = TargetOutcome::Pending;
        assert_eq!(pending_plan(&target, &frozen)?, CapturePlan::default());
        assert_eq!(frozen.mode, CaptureMode::ScreenshotsAndDomSnapshots);
        Ok(())
    }
    #[test]
    fn published_dom_is_checkpointed_before_sibling_failure_and_survives_interrupted_resume()
    -> Result<()> {
        use regressee_core::dom_snapshot::*;
        let root = tempfile::tempdir()?;
        let paths = regressee_storage::AppPaths::isolated(root.path());
        let mut store = Store::open(paths.clone())?;
        let mut project = Project::draft(
            "Published-channel checkpoint".into(),
            "http://127.0.0.1:9/inert",
        )?;
        project.capture_plan = CapturePlan::for_mode(CaptureMode::ScreenshotsAndDomSnapshots);
        project.environments[0].mappings[0].destination = "http://127.0.0.1:10/".parse()?;
        project.profiles[0].runtime = RuntimeIdentity::Pinned {
            node: "inert".into(),
            worker: "inert".into(),
            playwright: "inert".into(),
            browser: "inert".into(),
        };
        store.save_project(&project, None)?;
        let inventory = InventoryRevision {
            id: InventoryId::new(),
            project: project.id,
            identity_version: 2,
            pages: vec![InventoryPage {
                id: PageId::new(),
                canonical: project.sources[0].clone(),
                included: true,
                reason: None,
            }],
            coverage: DiscoveryCoverage::Complete,
        };
        let targets = capture_targets(&inventory, &project.profiles, &project.capture_plan, None);
        let mut record = RunRecord {
            run: CaptureRun {
                id: RunId::new(),
                project: project.id,
                intent: RunIntent::NewBaseline,
                inventory,
                profiles: project.profiles.clone(),
                capture_plan: project.capture_plan.clone(),
                environment: project.environments[0].clone(),
                rendering: RenderingEnvironment {
                    os: "inert".into(),
                    architecture: "inert".into(),
                    browser: "inert".into(),
                    worker_version: "inert".into(),
                    profile_version: 1,
                    fonts_fingerprint: None,
                },
                state: JobState::Running,
                started_at: Some(1),
                finished_at: None,
                targets,
            },
            capture: None,
            captures: vec![],
            dom_captures: vec![],
            queue: None,
        };
        store.begin_run(&record)?;
        let mut checkpoint = store.begin_run_append(record.run.id)?;
        record.run.targets[0].attempts = 1;
        checkpoint_target_metadata(&mut store, &mut checkpoint, &record, &[0])?;
        let bytes = DomSnapshot {
            nodes: vec![
                DomNode::Document,
                DomNode::Element {
                    depth: 1,
                    namespace: HTML_NAMESPACE.into(),
                    name: "html".into(),
                    prefix: None,
                    attributes: vec![],
                },
            ],
        }
        .canonical_bytes()?;
        let stats = DomSnapshot::from_canonical(&bytes)?.validate()?;
        let artifact = Artifact {
            id: dom_artifact_id(&bytes),
            bytes: bytes.len() as u64,
            kind: ArtifactKind::DomSnapshot,
        };
        let stage = crate::capture_evidence::private_staging(&store.paths().cache)?;
        let path = stage.path().join("detached.dom");
        fs::write(&path, &bytes)?;
        let mapped = regressee_core::environment::map_url(
            &record.run.targets[0].requested,
            &record.run.environment,
        )?;
        let metadata = DomCaptureMetadata {
            version: 1,
            run: record.run.id,
            key: record.run.targets[0].key.clone(),
            requested: mapped.to_string(),
            final_url: mapped.to_string(),
            runtime: project.profiles[0].runtime.clone(),
            profile: project.profiles[0].clone(),
            policy: project.capture_plan.dom_policy.unwrap(),
            sha256: artifact.id.clone(),
            bytes: artifact.bytes,
            node_count: stats.node_count,
            visited_nodes: stats.node_count,
            max_depth: stats.max_depth,
            coverage: stats.coverage,
            warnings: vec![],
            stable: true,
            duration_ms: 1,
            readiness: Readiness {
                font_failures: 0,
                image_failures: 0,
                resource_failures: 0,
            },
        };
        publish_channel(
            &mut store,
            &mut record,
            &mut checkpoint,
            0,
            CheckKind::DomSnapshot,
            Ok(PublishedChannel::DomSnapshot {
                metadata,
                artifact: artifact.clone(),
                path,
            }),
            0,
            &mut 0,
            &AtomicBool::new(false),
        )?;
        drop(stage); // Model worker scratch/tree cleanup after publication.
        let persisted = store.load_run(record.run.id)?;
        assert_eq!(persisted.dom_captures.len(), 1);
        assert_eq!(
            persisted.dom_captures[0].metadata.requested,
            mapped.as_str()
        );
        assert_ne!(
            persisted.dom_captures[0].metadata.requested,
            persisted.run.targets[0].requested.as_str()
        );
        assert!(matches!(
            persisted.run.targets[0].outcome,
            TargetOutcome::Pending
        ));
        record.run.state = JobState::Interrupted;
        record.run.finished_at = Some(2);
        store.finish_run(&record)?;
        drop(store);
        let mut store = Store::open(paths)?;
        let mut resumed = store.load_run(record.run.id)?;
        resumed.run.state = JobState::Running;
        resumed.run.finished_at = None;
        store.resume_run(&resumed)?;
        let mut checkpoint = store.begin_run_append(record.run.id)?;
        resumed.run.targets[0].attempts += 1;
        checkpoint_target_metadata(&mut store, &mut checkpoint, &resumed, &[0])?;
        publish_channel(
            &mut store,
            &mut resumed,
            &mut checkpoint,
            0,
            CheckKind::Screenshot,
            Ok(PublishedChannel::ScreenshotFailed {
                code: CaptureError::Timeout,
                retryable: false,
            }),
            0,
            &mut 0,
            &AtomicBool::new(false),
        )?;
        resumed.run.state = JobState::CompletedWithErrors;
        resumed.run.finished_at = Some(3);
        store.finish_run(&resumed)?;
        let persisted = store.load_run(resumed.run.id)?;
        assert_eq!(persisted.dom_captures.len(), 1);
        assert_eq!(persisted.dom_captures[0].metadata.sha256, artifact.id);
        assert!(matches!(
            persisted.run.targets[0].outcome,
            TargetOutcome::Failed {
                code: CaptureError::Timeout
            }
        ));
        assert!(matches!(
            persisted.run.targets[0].dom_outcome,
            DomTargetOutcome::Captured { .. }
        ));
        assert!(!persisted.run.targets[0].requested_kinds_captured());
        Ok(())
    }
    #[test]
    fn partial_baseline_check_schedules_only_saved_page_and_size_pairs() {
        let mut project =
            Project::draft("Partial baseline".into(), "https://example.test/").unwrap();
        let mut mobile = project.profiles[0].clone();
        mobile.id = ProfileId::new();
        mobile.name = "Mobile".into();
        project.profiles.push(mobile);
        let inventory = InventoryRevision {
            id: InventoryId::new(),
            project: project.id,
            identity_version: 2,
            pages: ["/", "/second"]
                .into_iter()
                .map(|path| InventoryPage {
                    id: PageId::new(),
                    canonical: project.sources[0].join(path).unwrap(),
                    included: true,
                    reason: None,
                })
                .collect(),
            coverage: DiscoveryCoverage::Complete,
        };
        let all = capture_targets(&inventory, &project.profiles, &CapturePlan::default(), None);
        assert_eq!(all.len(), 4);
        let saved: BTreeSet<_> = all[..3].iter().map(|t| t.key.clone()).collect();
        let baseline = BaselineVersion {
            id: BaselineId::new(),
            project: project.id,
            parent: None,
            inventory: inventory.id,
            created_at: 1,
            references: saved
                .iter()
                .map(|key| BaselineReference {
                    key: key.clone(),
                    run: RunId::new(),
                    artifact: "a".repeat(64).try_into().unwrap(),
                    captured_at: 1,
                })
                .collect(),
            dom_references: vec![],
            capture_plan: CapturePlan::default(),
            source_run: None,
            intended_targets: vec![],
            intended_count: 4,
            explicitly_partial: true,
        };
        let check = capture_targets(
            &inventory,
            &project.profiles,
            &baseline.capture_plan,
            Some(&baseline),
        );
        assert_eq!(check.len(), 3);
        assert_eq!(
            check.iter().map(|t| t.key.clone()).collect::<BTreeSet<_>>(),
            saved
        );
        assert!(
            check
                .iter()
                .all(|t| t.outcome == TargetOutcome::Pending && t.attempts == 0)
        );
        assert_eq!(
            inventory.pages.len(),
            2,
            "original discovery remains intact"
        );
    }
    #[test]
    fn all_modes_schedule_independent_saved_slots_and_do_not_retry_omissions() {
        let project = Project::draft("Mode fixture".into(), "http://127.0.0.1:9/").unwrap();
        let inventory = InventoryRevision {
            id: InventoryId::new(),
            project: project.id,
            identity_version: 2,
            pages: ["/a", "/b", "/omitted"]
                .into_iter()
                .map(|path| InventoryPage {
                    id: PageId::new(),
                    canonical: project.sources[0].join(path).unwrap(),
                    included: true,
                    reason: None,
                })
                .collect(),
            coverage: DiscoveryCoverage::Complete,
        };
        for mode in [
            CaptureMode::ScreenshotsOnly,
            CaptureMode::ScreenshotsAndDomSnapshots,
            CaptureMode::DomSnapshotsOnly,
        ] {
            let plan = CapturePlan::for_mode(mode);
            let all = capture_targets(&inventory, &project.profiles, &plan, None);
            assert_eq!(all.len(), 3);
            for t in &all {
                assert_eq!(
                    matches!(t.outcome, TargetOutcome::Pending),
                    mode.screenshots()
                );
                assert_eq!(
                    matches!(t.dom_outcome, DomTargetOutcome::Pending),
                    mode.dom_snapshots()
                );
            }
            let run = RunId::new();
            let baseline = BaselineVersion {
                id: BaselineId::new(),
                project: project.id,
                parent: None,
                inventory: inventory.id,
                created_at: 2,
                references: if mode.screenshots() {
                    vec![BaselineReference {
                        key: all[0].key.clone(),
                        run,
                        artifact: "a".repeat(64).try_into().unwrap(),
                        captured_at: 1,
                    }]
                } else {
                    vec![]
                },
                dom_references: if mode.dom_snapshots() {
                    vec![DomBaselineReference {
                        key: all[1].key.clone(),
                        run,
                        artifact: "b".repeat(64).try_into().unwrap(),
                        captured_at: 1,
                        policy: plan.dom_policy.unwrap(),
                    }]
                } else {
                    vec![]
                },
                capture_plan: plan.clone(),
                source_run: Some(run),
                intended_count: 3,
                explicitly_partial: true,
                intended_targets: all.iter().map(|t| t.key.clone()).collect(),
            };
            baseline.validate().unwrap();
            let check = capture_targets(&inventory, &project.profiles, &plan, Some(&baseline));
            assert_eq!(
                check.len(),
                if mode == CaptureMode::ScreenshotsAndDomSnapshots {
                    2
                } else {
                    1
                }
            );
            assert!(check.iter().all(target_pending));
            assert!(check.iter().all(|t| t.key != all[2].key));
            for mut t in check {
                let screenshot_pending = matches!(t.outcome, TargetOutcome::Pending);
                let dom_pending = matches!(t.dom_outcome, DomTargetOutcome::Pending);
                cancel_pending(&mut t);
                assert_eq!(
                    matches!(t.outcome, TargetOutcome::Cancelled),
                    screenshot_pending
                );
                assert_eq!(
                    matches!(t.dom_outcome, DomTargetOutcome::Cancelled),
                    dom_pending
                );
            }
        }
    }
    #[test]
    fn discovery_reports_preparation_before_runtime_failure_without_visits() -> Result<()> {
        let dir = tempfile::tempdir()?;
        let paths = regressee_storage::AppPaths::isolated(dir.path());
        let mut store = Store::open(paths.clone())?;
        let project = Project::draft("Discovery startup fixture".into(), "https://example.test")?;
        let saved = store.save_project(&project, None)?;
        let product = Product {
            distribution: None,
            runtime_cache: paths.cache.join("absent-browser"),
            official_runtime: false,
            auth: Default::default(),
        };
        let mut phases = Vec::new();
        let result = product.discover(
            &store,
            project.id,
            Arc::new(AtomicBool::new(false)),
            &mut |message, completed, total| phases.push((message.to_owned(), completed, total)),
        );
        assert!(result.is_err());
        assert_eq!(
            phases,
            vec![
                (
                    "Checking website boundaries and login settings…".into(),
                    0,
                    None
                ),
                (
                    "Verifying the private browser for page discovery…".into(),
                    0,
                    None
                ),
            ]
        );
        assert_eq!(
            serde_json::to_value(store.load_project(project.id)?)?,
            serde_json::to_value(saved)?
        );
        assert!(!product.runtime_cache.exists());
        Ok(())
    }

    #[test]
    fn loading_and_retry_messages_preserve_actual_visit_counts() {
        assert_eq!(
            discovery_loading_message(0, 1),
            "Loading the first page and looking for links…"
        );
        assert_eq!(
            discovery_loading_message(1, 1),
            "Finding pages · 1 page visited…"
        );
        assert_eq!(
            discovery_loading_message(23, 1),
            "Finding pages · 23 pages visited…"
        );
        assert_eq!(
            discovery_loading_message(0, 2),
            "Retrying the page and looking for links · 0 pages visited…"
        );
    }

    #[test]
    fn discovery_status_stays_steady_through_routine_pacing() {
        let expected = "Finding pages · 13 pages visited…";
        for delay in [0, 350, 1000, 1999] {
            let loading = discovery_loading_message(13, 1);
            let during_wait = discovery_wait_message(Duration::from_millis(delay), 13)
                .unwrap_or_else(|| loading.clone());
            let next_page = discovery_loading_message(13, 1);
            assert_eq!(loading, expected);
            assert_eq!(during_wait, expected);
            assert_eq!(next_page, expected);
        }
        assert_eq!(
            discovery_loading_message(14, 1),
            "Finding pages · 14 pages visited…"
        );
    }

    #[test]
    fn longer_discovery_waits_and_retries_keep_their_reason_and_actual_count() {
        for seconds in [2, 10, 60] {
            assert_eq!(
                discovery_wait_message(Duration::from_secs(seconds), 13).as_deref(),
                Some(
                    "Waiting between page visits to respect website crawl limits · 13 pages visited…"
                )
            );
        }
        assert_eq!(
            discovery_wait_message(Duration::from_secs(2), 1).as_deref(),
            Some("Waiting between page visits to respect website crawl limits · 1 page visited…")
        );
        assert_eq!(
            discovery_loading_message(13, 2),
            "Retrying the page and looking for links · 13 pages visited…"
        );
    }

    #[test]
    fn presets_and_space_are_bounded() {
        let mut s = DiscoverySettings::default();
        assert_eq!(speed(&s), (2, 350));
        s.speed = Speed::Gentle;
        assert_eq!(speed(&s), (1, 1000));
        s.speed = Speed::Fast;
        assert_eq!(speed(&s), (4, 0));
        s.advanced.concurrency = Some(8);
        assert_eq!(speed(&s), (8, 0));
        s.advanced.concurrency = Some(9);
        assert!(s.validate().is_err());
        assert!(space_check(0, 1, true).is_err());
        assert!(space_check(2 * 1024 * 1024 * 1024, 3, false).is_ok());
    }
}
