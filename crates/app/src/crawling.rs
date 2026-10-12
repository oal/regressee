// SPDX-License-Identifier: MPL-2.0
//! Blocking Phase 3 services. One bounded scheduler owns every browser operation;
//! persisted target boundaries, not worker progress, are authoritative.
use crate::product::{Product, configure_profile, map_failure, worker_version};
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
    fs::{self, File},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

fn scoped(installed: &Installed, url: &url::Url) -> Result<()> {
    ensure!(
        installed.manifest.scope != Scope::DevelopmentLoopback || distribution::loopback(url),
        "This development browser may visit owned literal-loopback fixtures only. Qualified public runtime is pending."
    );
    Ok(())
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
            version: 2,
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
                    "robots.txt is unavailable (HTTP {status}); respect policy stops this website. An authorized override must be explicit."
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
            "Repair to a trusted Phase 3 worker before discovery. Historical evidence remains intact."
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
                notes.push("10,000 candidate safety budget reached".into());
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
                    if *start + Duration::from_millis(delay) > Instant::now() {
                        progress(
                            "Waiting between page visits to respect website crawl limits…",
                            visited,
                            None,
                        );
                    }
                    wait(
                        *start + Duration::from_millis(delay),
                        &cancel,
                        &AtomicBool::new(false),
                    );
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
                            "Could not inspect rendered page: {:?}",
                            map_failure(&e)
                        ));
                    }
                }
            }
            pages.push(page);
            if p.discovery.mode == DiscoveryMode::AddressesAndLinks && visited > 0 {
                let page_word = if visited == 1 { "page" } else { "pages" };
                progress(
                    &format!(
                        "Finding rendered links · {visited} {page_word} visited — no clicking or forms"
                    ),
                    visited,
                    None,
                );
            }
        }
        if pages.iter().any(|p| !p.included) {
            limited = true;
        }
        if p.discovery.mode == DiscoveryMode::OnlyAddresses {
            notes.push(
                "List-only discovery: no links were explored. robots.txt policy still applies."
                    .into(),
            );
        }
        if p.discovery.advanced.max_depth.is_some() {
            notes.push("Link depth is bounded; deeper pages may exist.".into());
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
                    DiscoveryCoverage::Limited { reason: "Some candidates were excluded, unavailable, cancelled or bounded. This is not proof of all website pages.".into() }
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
            store, project, preview, check, resume, cancel, pause, progress, false, status,
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
        let installed = self.resolve_runtime(&cancel)?;
        ensure!(
            worker_version(&installed)? == regressee_runtime::protocol::WORKER_VERSION,
            "Repair to a trusted Phase 4 worker for authenticated multi-page jobs. Older baselines require their matching runtime or an explicit replacement."
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
                "Project settings changed. Keep this evidence and start a fresh run."
            );
            ensure!(
                r.run
                    .profiles
                    .iter()
                    .all(|p| p.runtime == installed.manifest.identity),
                "Matching browser runtime is required to resume"
            );
            ensure!(
                r.run.rendering.os == std::env::consts::OS
                    && r.run.rendering.architecture == std::env::consts::ARCH,
                "Rendering computer differs; start a compatible fresh baseline"
            );
            ensure!(
                r.run.rendering == crate::product::rendering_environment(&installed)?,
                "Rendering environment or fonts changed. Keep this evidence and start a fresh run."
            );
            for target in &r.run.targets {
                scoped(&installed, &target.requested)?;
            }
            preflight(
                store,
                r.run
                    .targets
                    .iter()
                    .filter(|t| matches!(t.outcome, TargetOutcome::Pending))
                    .count()
                    .min(8),
                false,
            )?;
            queue.resumed_at.push(distribution::now());
            self.auth_state(store, project, &r.run.environment)?;
            for target in &mut r.run.targets {
                if target.outcome
                    == (TargetOutcome::Failed {
                        code: CaptureError::NeedsLogin,
                    })
                {
                    ensure!(
                        target.attempts < 8,
                        "Login retries exhausted. Start a fresh run; prior evidence is retained."
                    );
                    target.outcome = TargetOutcome::Pending;
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
            let (inventory, mut profiles, intent, environment, rendering) = if check {
                let baseline = store.load_baseline(
                    saved
                        .project
                        .active_baseline
                        .context("Save a complete baseline before checking")?,
                )?;
                let before = store.load_run(baseline.references[0].run)?;
                ensure!(
                    !baseline.explicitly_partial,
                    "Reduced coverage requires a fresh complete baseline in this preview"
                );
                (
                    before.run.inventory.clone(),
                    before.run.profiles.clone(),
                    RunIntent::Check {
                        baseline: baseline.id,
                    },
                    store.selected_environment(&saved.project)?,
                    crate::product::rendering_environment(&installed)?,
                )
            } else {
                let preview = preview
                    .context("Find pages and review the exact inventory before capturing")?;
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
                        RunIntent::RepresentativePreview
                    } else {
                        RunIntent::NewBaseline
                    },
                    store.selected_environment(&saved.project)?,
                    crate::product::rendering_environment(&installed)?,
                )
            };
            if check {
                ensure!(
                    profiles
                        .iter()
                        .all(|p| p.runtime == installed.manifest.identity),
                    "The baseline needs its matching browser runtime"
                );
            } else {
                for p in &mut profiles {
                    configure_profile(p, &installed)?;
                }
            }
            self.auth_state(store, project, &environment)?;
            if check {
                self.require_probe(store, project, &environment)?;
            }
            let targets = inventory
                .pages
                .iter()
                .filter(|p| p.included)
                .flat_map(|page| {
                    profiles.iter().map(move |profile| CaptureTarget {
                        key: TargetKey {
                            page: page.id,
                            profile: profile.id,
                        },
                        requested: page.canonical.clone(),
                        final_url: None,
                        attempts: 0,
                        outcome: TargetOutcome::Pending,
                        warnings: vec![],
                    })
                })
                .collect();
            let frozen_discovery = if let RunIntent::Check { baseline } = &intent {
                let b = store.load_baseline(*baseline)?;
                store
                    .load_run(b.references[0].run)?
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
                    environment,
                    rendering,
                    state: JobState::Running,
                    started_at: Some(distribution::now()),
                    finished_at: None,
                    targets,
                },
                capture: None,
                captures: vec![],
                queue: Some(QueueSettings {
                    discovery: frozen_discovery,
                    project_revision: saved.revision,
                    resumed_at: vec![],
                }),
            };
            r.run.validate()?;
            if let RunIntent::Check { baseline } = r.run.intent {
                let b = store.load_baseline(baseline)?;
                ensure!(
                    r.run
                        .targets
                        .iter()
                        .map(|t| &t.key)
                        .collect::<std::collections::BTreeSet<_>>()
                        == b.references.iter().map(|r| &r.key).collect(),
                    "Baseline fixed coverage mismatch; never visit extra or omit inherited targets"
                );
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
            preflight(store, r.run.targets.len(), true)?;
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
                    if matches!(t.outcome, TargetOutcome::Pending) {
                        t.outcome = TargetOutcome::Cancelled;
                    }
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
                    if matches!(target.outcome, TargetOutcome::Pending) {
                        target.outcome = TargetOutcome::Cancelled;
                    }
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
                    if matches!(t.outcome, TargetOutcome::Pending) {
                        t.outcome = TargetOutcome::Cancelled;
                    }
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
                .filter(|(_, t)| matches!(t.outcome, TargetOutcome::Pending))
                .map(|(i, _)| i)
                .collect();
            if pending.is_empty() {
                record.run.state = if record
                    .run
                    .targets
                    .iter()
                    .all(|t| matches!(t.outcome, TargetOutcome::Captured { .. }))
                {
                    JobState::Completed
                } else {
                    JobState::CompletedWithErrors
                };
                break;
            }
            preflight(store, workers.len(), false)?;
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
                        t.outcome = TargetOutcome::Skipped {
                            reason: "robots.txt now excludes this saved target".into(),
                        };
                        changed.push(i);
                        continue;
                    }
                    Some(Err(e)) => {
                        t.outcome = TargetOutcome::Skipped { reason: e.clone() };
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
            if !changed.is_empty() {
                let updates: Vec<_> = changed
                    .iter()
                    .map(|index| regressee_storage::TargetCheckpoint {
                        index: *index,
                        target: &record.run.targets[*index],
                        capture: None,
                    })
                    .collect();
                store.append_run_targets(&mut checkpoint, &updates)?;
            }
            if selected.is_empty() {
                continue;
            }
            let accounted = record
                .run
                .targets
                .iter()
                .filter(|t| !matches!(t.outcome, TargetOutcome::Pending))
                .count();
            progress(
                "Capturing screenshots…",
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
            let results = std::thread::scope(|scope| {
                let mut handles = Vec::new();
                for (worker, (target, profile, address)) in workers.iter_mut().zip(tasks) {
                    let cancel = cancel.clone();
                    let session = auth.as_ref().map(|a| a.scope.clone());
                    handles.push(scope.spawn(move || {
                        capture_one(
                            worker, job, request_id, &target, &profile, installed, cancel,
                            &address?, session,
                        )
                    }));
                }
                handles
                    .into_iter()
                    .map(|h| {
                        h.join()
                            .unwrap_or_else(|_| Err(anyhow::anyhow!("browser task panicked")))
                    })
                    .collect::<Vec<_>>()
            });
            let mut backoff = 0;
            for (slot, (index, result)) in selected.into_iter().zip(results).enumerate() {
                match result {
                    Ok((metadata, path, warnings)) => {
                        let image =
                            crate::images::ingest(store, &path, ArtifactKind::Screenshot, &cancel)?;
                        ensure!(
                            image.original == metadata.sha256
                                && image.width == metadata.width
                                && image.height == metadata.height,
                            "decoded evidence differs"
                        );
                        let t = &mut record.run.targets[index];
                        t.final_url = Some(url::Url::parse(&metadata.final_url)?);
                        t.warnings.retain(|w| w.starts_with("ResumedAfterLogin"));
                        t.warnings.extend(warnings);
                        t.outcome = TargetOutcome::Captured {
                            artifact: image.original.clone(),
                            width: image.width,
                            height: image.height,
                        };
                        record.captures.push(CaptureEvidence { metadata, image });
                        fs::remove_file(&path)?;
                        fs::remove_file(path.with_extension("json"))?;
                    }
                    Err(e) => {
                        let t = &mut record.run.targets[index];
                        if cancel.load(Ordering::Acquire) {
                            t.outcome = TargetOutcome::Cancelled;
                        } else if retryable(&e) && t.attempts <= settings.advanced.retries {
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
                            t.outcome = TargetOutcome::Failed {
                                code: map_failure(&e),
                            };
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
                let capture = record
                    .captures
                    .last()
                    .filter(|capture| capture.metadata.key == target.key);
                store.append_run_targets(
                    &mut checkpoint,
                    &[regressee_storage::TargetCheckpoint {
                        index,
                        target,
                        capture,
                    }],
                )?;
            }
            let counts = TargetCounts::from_run(&record.run);
            progress(
                "Capturing screenshots…",
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
            "Analyzing saved originals with Balanced sensitivity",
            0,
            None,
        );
        crate::review::analyze_run(
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
        format!("Loading the next page and looking for links · {visited} {page_word} visited…")
    }
}
fn preflight(store: &Store, targets: usize, initial: bool) -> Result<()> {
    space_check(
        distribution::available_bytes(&store.paths().cache)?,
        targets,
        initial,
    )?;
    space_check(
        distribution::available_bytes(&store.paths().data)?,
        targets,
        initial,
    )
}
fn space_check(available: u64, targets: usize, initial: bool) -> Result<()> {
    let required = 256 * 1024 * 1024
        + targets as u64
            * if initial {
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
fn capture_one(
    worker: &mut Worker,
    job: RunId,
    id: u64,
    target: &CaptureTarget,
    profile: &CaptureProfile,
    installed: &Installed,
    cancel: Arc<AtomicBool>,
    address: &url::Url,
    session: Option<SessionReference>,
) -> Result<(CaptureMetadata, std::path::PathBuf, Vec<String>)> {
    let mut captured = None;
    let mut warnings = target.warnings.clone();
    let mut wire_warnings = Vec::new();
    worker.request(
        Request {
            version: 2,
            request_id: id,
            job,
            command: Command::CaptureTarget {
                key: target.key.clone(),
                address: address.to_string(),
                profile: Box::new(profile.clone()),
                session,
            },
        },
        Instant::now()
            + Duration::from_millis(
                u64::from(profile.stabilization.navigation_ms)
                    + u64::from(profile.stabilization.readiness_ms)
                    + 10000,
            ),
        cancel.clone(),
        &mut |e| {
            match e.event {
                EventKind::TargetCaptured {
                    key,
                    output_token,
                    artifact,
                    width,
                    height,
                } => {
                    ensure!(
                        captured.is_none() && key == target.key,
                        "foreign or duplicate captured evidence"
                    );
                    captured = Some((output_token, artifact, width, height));
                }
                EventKind::Warning { code } => wire_warnings.push(format!("{code:?}")),
                _ => (),
            }
            Ok(())
        },
    )?;
    let (token, artifact, width, height) = captured.context("worker omitted captured evidence")?;
    crate::images::budget(width, height)?;
    let mp = worker.metadata_path(token);
    let m = fs::symlink_metadata(&mp)?;
    ensure!(
        m.is_file() && regressee_core::platform::ordinary(&m) && m.len() <= 256 * 1024,
        "unsafe metadata"
    );
    let metadata: CaptureMetadata = serde_json::from_reader(File::open(mp)?)?;
    ensure!(
        metadata.version == 1
            && metadata.key == target.key
            && metadata.profile == *profile
            && metadata.runtime == installed.manifest.identity
            && metadata.requested == address.as_str()
            && metadata.final_url == address.as_str()
            && metadata.sha256 == artifact.id
            && metadata.bytes == artifact.bytes
            && metadata.width == width
            && metadata.height == height
            && metadata.warnings == wire_warnings
            && metadata.duration_ms <= 300000
            && metadata.capture_mode == "chromium-new-headless",
        "capture metadata mismatch"
    );
    let path = worker.output_path(token);
    let m = fs::symlink_metadata(&path)?;
    ensure!(
        m.is_file()
            && regressee_core::platform::ordinary(&m)
            && m.len() == artifact.bytes
            && distribution::hash(File::open(&path)?, &cancel)? == artifact.id.as_str(),
        "unsafe screenshot output"
    );
    warnings.extend(wire_warnings);
    if let Some(directory) = std::env::var_os("REGRESSEE_CAPTURE_TRACE_EVIDENCE") {
        let directory = std::path::PathBuf::from(directory);
        let source = worker.output_path(token).with_extension("trace.json");
        regressee_runtime::protocol::retain_capture_trace(&directory, &source, job, token)?;
        fs::remove_file(source)?;
    }
    Ok((metadata, path, warnings))
}
#[cfg(test)]
mod tests {
    use super::*;
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
            "Loading the next page and looking for links · 1 page visited…"
        );
        assert_eq!(
            discovery_loading_message(23, 1),
            "Loading the next page and looking for links · 23 pages visited…"
        );
        assert_eq!(
            discovery_loading_message(0, 2),
            "Retrying the page and looking for links · 0 pages visited…"
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
