// SPDX-License-Identifier: MPL-2.0
//! Real local IPC regression probes over owned synthetic stored evidence.
#![cfg(target_os = "linux")]
use anyhow::{Result, ensure};
use regressee_app::{
    engine::{Client, Server, dom},
    protocol::{self, Operation, Outcome, Request, Response, Submit},
    terminal,
};
use regressee_core::{dom_snapshot::*, domain::*, evidence::*};
use regressee_storage::{AppPaths, Store};
use std::{
    fs,
    path::Path,
    time::{Duration, Instant},
};

struct Fixture {
    root: tempfile::TempDir,
    client: Client,
    thread: Option<std::thread::JoinHandle<Result<()>>>,
    project: ProjectId,
    run: RunId,
    keys: Vec<TargetKey>,
}

impl Fixture {
    fn start(count: usize, escaped_warnings: bool) -> Result<Self> {
        let root = tempfile::tempdir()?;
        let paths = AppPaths::isolated(root.path());
        let mut store = Store::open(paths.clone())?;
        let mut project = Project::draft(
            "Owned IPC counterexample".into(),
            "http://127.0.0.1:43210/owned",
        )?;
        project.capture_plan = CapturePlan::for_mode(CaptureMode::DomSnapshotsOnly);
        project.profiles[0].runtime = RuntimeIdentity::Pinned {
            node: "fixture".into(),
            worker: "fixture".into(),
            playwright: "fixture".into(),
            browser: "fixture".into(),
        };
        let saved = store.save_project(&project, None)?;
        let pages: Vec<_> = (0..count)
            .map(|index| {
                let mut canonical = project.sources[0].clone();
                canonical.set_path(&format!("/owned/{index}"));
                InventoryPage {
                    id: PageId::new(),
                    canonical,
                    included: true,
                    reason: None,
                }
            })
            .collect();
        let keys: Vec<_> = pages
            .iter()
            .map(|p| TargetKey {
                page: p.id,
                profile: project.profiles[0].id,
            })
            .collect();
        let mut record = RunRecord {
            run: CaptureRun {
                id: RunId::new(),
                project: project.id,
                intent: RunIntent::NewBaseline,
                inventory: InventoryRevision {
                    id: InventoryId::new(),
                    project: project.id,
                    identity_version: 2,
                    pages: pages.clone(),
                    coverage: DiscoveryCoverage::Complete,
                },
                profiles: project.profiles.clone(),
                capture_plan: project.capture_plan.clone(),
                environment: project.environments[0].clone(),
                rendering: RenderingEnvironment {
                    os: "fixture".into(),
                    architecture: "fixture".into(),
                    browser: "fixture".into(),
                    worker_version: "fixture".into(),
                    profile_version: 1,
                    fonts_fingerprint: None,
                },
                state: JobState::Running,
                started_at: Some(100),
                finished_at: None,
                targets: pages
                    .iter()
                    .zip(&keys)
                    .map(|(p, key)| CaptureTarget {
                        key: key.clone(),
                        requested: p.canonical.clone(),
                        final_url: None,
                        attempts: 0,
                        outcome: TargetOutcome::NotRequested,
                        dom_outcome: DomTargetOutcome::Pending,
                        warnings: vec![],
                    })
                    .collect(),
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
        store.freeze_inventory(&record.run.inventory)?;
        store.begin_run(&record)?;
        let mut nodes = vec![
            DomNode::Document,
            DomNode::Element {
                depth: 1,
                namespace: HTML_NAMESPACE.into(),
                name: "html".into(),
                attributes: vec![],
                prefix: None,
            },
        ];
        for _ in 0..4 {
            nodes.push(DomNode::Element {
                depth: 2,
                namespace: HTML_NAMESPACE.into(),
                name: "div".into(),
                attributes: vec![],
                prefix: None,
            });
            nodes.push(DomNode::Text {
                depth: 3,
                text: "ordinary synthetic text\n".repeat(2048),
            });
        }
        let snapshot = DomSnapshot { nodes };
        let bytes = snapshot.canonical_bytes()?;
        let stats = snapshot.validate()?;
        for target in &mut record.run.targets {
            target.attempts = 1;
            if escaped_warnings {
                target.warnings = vec!["\0".repeat(1024); 64];
                target.dom_outcome = DomTargetOutcome::Failed {
                    code: DomCaptureError::SerializationFailed,
                };
            } else {
                target.final_url = Some(target.requested.clone());
                let metadata = DomCaptureMetadata {
                    version: 1,
                    run: record.run.id,
                    key: target.key.clone(),
                    requested: target.requested.to_string(),
                    final_url: target.requested.to_string(),
                    runtime: project.profiles[0].runtime.clone(),
                    profile: project.profiles[0].clone(),
                    policy: DomSnapshotPolicy::default(),
                    sha256: dom_artifact_id(&bytes),
                    bytes: bytes.len() as u64,
                    node_count: stats.node_count,
                    visited_nodes: stats.node_count,
                    max_depth: stats.max_depth,
                    coverage: stats.coverage.clone(),
                    warnings: vec![],
                    stable: true,
                    duration_ms: 1,
                    readiness: Readiness {
                        font_failures: 0,
                        image_failures: 0,
                        resource_failures: 0,
                    },
                };
                record
                    .dom_captures
                    .push(store.ingest_dom_snapshot(bytes.as_slice(), &metadata)?);
                target.dom_outcome = DomTargetOutcome::Captured {
                    artifact: metadata.sha256,
                };
            }
        }
        record.run.state = if escaped_warnings {
            JobState::CompletedWithErrors
        } else {
            JobState::Completed
        };
        record.run.finished_at = Some(101);
        store.finish_run(&record)?;
        drop(store);
        let server = Server::without_runtime(paths.clone())?;
        let thread = std::thread::spawn(move || server.run());
        let client = Client::existing(paths)?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while client.snapshot().is_err() {
            ensure!(
                Instant::now() < deadline,
                "owned IPC engine startup timed out"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        Ok(Self {
            root,
            client,
            thread: Some(thread),
            project: project.id,
            run: record.run.id,
            keys,
        })
    }

    fn submit(&self, operation: Operation) -> Result<protocol::Job> {
        let snapshot = self.client.snapshot()?;
        self.client.submit(Submit {
            request: uuid::Uuid::new_v4(),
            engine: snapshot.engine,
            epoch: snapshot.epoch,
            operation,
            confirmation: None,
            expected_revision: None,
        })
    }

    fn job(&self, id: uuid::Uuid) -> Result<protocol::Job> {
        match self.client.request(Request::Job { id })? {
            Response::Job(job) => Ok(*job),
            _ => anyhow::bail!("unexpected owned IPC job response"),
        }
    }

    fn wait(&self, mut job: protocol::Job) -> Result<protocol::Job> {
        let deadline = Instant::now() + Duration::from_secs(30);
        while !job.state.terminal() {
            ensure!(Instant::now() < deadline, "owned IPC job timed out");
            std::thread::sleep(Duration::from_millis(2));
            job = self.job(job.id)?;
        }
        Ok(job)
    }

    fn report(&self, destination: &Path, keys: &[TargetKey]) -> Result<protocol::Job> {
        let preview = self.wait(self.submit(Operation::Terminal(
            terminal::Command::ReportPreview {
                selection: terminal::Selection {
                    project: self.project,
                    run: Some(self.run),
                    analysis: None,
                },
                dom_analysis: None,
                selected: keys.to_vec(),
                destination: destination.into(),
            },
        ))?)?;
        let Some(Outcome::Terminal(output)) = preview.outcome else {
            anyhow::bail!("missing actual report preview")
        };
        let terminal::Output::ReportPreview(preview) = *output else {
            anyhow::bail!("unexpected report preview reply")
        };
        let operation = Operation::Terminal(terminal::Command::Report(preview));
        let prepared = self.wait(self.submit(Operation::Prepare(Box::new(operation.clone())))?)?;
        let Some(Outcome::Confirmation(confirmation)) = prepared.outcome else {
            anyhow::bail!("missing actual report confirmation")
        };
        let snapshot = self.client.snapshot()?;
        self.client.submit(Submit {
            request: uuid::Uuid::new_v4(),
            engine: snapshot.engine,
            epoch: snapshot.epoch,
            operation,
            confirmation: Some(confirmation.id),
            expected_revision: None,
        })
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if matches!(self.client.request(Request::Stop), Ok(Response::Ok)) {
                break;
            }
            if let Ok(snapshot) = self.client.snapshot() {
                for job in snapshot.jobs {
                    if !job.state.terminal() {
                        let _ = self.client.request(Request::Cancel { id: job.id });
                    }
                }
            }
            assert!(
                Instant::now() < deadline,
                "owned IPC engine cleanup timed out"
            );
            std::thread::sleep(Duration::from_millis(5));
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[test]
fn actual_ipc_escape_heavy_results_page_and_oversized_read_leave_engine_healthy() -> Result<()> {
    let f = Fixture::start(169, true)?;
    let mut offset = 0;
    let mut seen = std::collections::BTreeSet::new();
    loop {
        let job = f.wait(f.submit(Operation::Dom(dom::Command::Results {
            selection: dom::Selection {
                project: f.project,
                run: Some(f.run),
                analysis: None,
            },
            offset,
            limit: 200,
            page: None,
            profile: None,
            outcome: None,
            capture_state: None,
        }))?)?;
        assert_eq!(
            job.state,
            protocol::JobState::Succeeded,
            "{:?}",
            job.outcome
        );
        let Some(Outcome::Dom(output)) = job.outcome else {
            anyhow::bail!("missing real IPC DOM results")
        };
        let dom::Output::Results(page) = *output else {
            anyhow::bail!("unexpected real IPC DOM reply")
        };
        assert!(
            serde_json::to_vec(&page)?.len() <= regressee_app::engine::text::MAX_RESULT_PAGE_BYTES
        );
        assert_eq!(page.total, 169);
        assert_eq!(page.offset, offset);
        assert!(!page.rows.is_empty() && page.rows.len() < 169);
        let count = page.rows.len();
        for row in page.rows {
            assert!(seen.insert(row.target.key));
        }
        let Some(next) = page.next_offset else { break };
        assert_eq!(next, offset + count);
        offset = next;
    }
    assert_eq!(seen.len(), 169);
    let oversized = f.wait(f.submit(Operation::Evidence(f.project))?)?;
    assert_eq!(oversized.state, protocol::JobState::Failed);
    assert!(
        matches!(&oversized.outcome, Some(Outcome::ProductFailure(message)) if message.starts_with("response_too_large:"))
    );
    assert!(serde_json::to_vec(&oversized)?.len() < 8192);
    assert!(!f.client.snapshot()?.draining);
    let healthy = f.wait(f.submit(Operation::List { page: 0 })?)?;
    assert_eq!(healthy.state, protocol::JobState::Succeeded);
    Ok(())
}

#[test]
fn actual_ipc_inflight_dom_report_is_cancelled_but_late_cancel_keeps_committed_success()
-> Result<()> {
    let f = Fixture::start(32, false)?;
    let destination = f.root.path().join("cancelled-report");
    let mut job = f.report(&destination, &f.keys)?;
    let deadline = Instant::now() + Duration::from_secs(30);
    let mut observed_inflight_stage = false;
    while !job.state.terminal() {
        for entry in fs::read_dir(f.root.path())? {
            let entry = entry?;
            if entry.file_name().to_string_lossy().starts_with(".tmp")
                && entry.path().join("0-dom-after.canonical.txt").is_file()
            {
                observed_inflight_stage = true;
                break;
            }
        }
        if observed_inflight_stage {
            break;
        }
        ensure!(
            Instant::now() < deadline,
            "owned report never reached its rendering stage"
        );
        std::thread::sleep(Duration::from_millis(1));
        job = f.job(job.id)?;
    }
    ensure!(
        observed_inflight_stage,
        "must cancel a genuinely rendering report, not a queued/pre-cancelled request"
    );
    f.client.request(Request::Cancel { id: job.id })?;
    let cancelled = f.wait(job)?;
    assert_eq!(
        cancelled.state,
        protocol::JobState::Cancelled,
        "{:?}",
        cancelled.outcome
    );
    assert!(matches!(cancelled.outcome, Some(Outcome::Cancelled)));
    assert!(!destination.exists());
    assert!(!f.client.snapshot()?.draining);
    let committed_destination = f.root.path().join("committed-report");
    let success = f.wait(f.report(&committed_destination, &f.keys[..1])?)?;
    assert_eq!(
        success.state,
        protocol::JobState::Succeeded,
        "{:?}",
        success.outcome
    );
    let html = fs::read(committed_destination.join("index.html"))?;
    let _ = f.client.request(Request::Cancel { id: success.id });
    let after_late_cancel = f.job(success.id)?;
    assert_eq!(after_late_cancel.state, protocol::JobState::Succeeded);
    assert!(
        matches!(after_late_cancel.outcome, Some(Outcome::Terminal(output)) if matches!(*output, terminal::Output::Reported { .. }))
    );
    assert_eq!(html, fs::read(committed_destination.join("index.html"))?);
    Ok(())
}
