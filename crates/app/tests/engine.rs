// SPDX-License-Identifier: MPL-2.0
#![cfg(feature = "test-support")]
use anyhow::{Context, Result, ensure};
use regressee_app::{
    engine::{Client, Server, ownership::Ownership},
    protocol::*,
};
use regressee_core::domain::Project;
use regressee_storage::AppPaths;
use std::{
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;
#[path = "support/approval_fixture.rs"]
mod fixture;

struct Harness {
    root: tempfile::TempDir,
    paths: AppPaths,
    thread: Option<thread::JoinHandle<Result<()>>>,
    client: Client,
}

#[test]
fn machine_local_motion_preferences_preserve_appearance_through_engine_and_worker() -> Result<()> {
    use regressee_core::appearance::AppAppearance;
    use regressee_storage::{AppPreferences, PreferencesUpdate};
    let h = Harness::new()?;
    let preferences = regressee_app::appearance::AppearanceSettings::start_client(h.client.clone());
    assert!(!preferences.wait_loaded().reduced_motion);
    preferences.choose(AppAppearance::Dark);
    preferences.choose_reduced_motion(true);
    preferences.join_after_ui();
    assert!(preferences.snapshot().error.is_none());
    assert_eq!(
        AppPreferences::load(&h.paths)?,
        AppPreferences {
            appearance: AppAppearance::Dark,
            reduced_motion: true
        }
    );
    // Both legacy appearance entry points must preserve remembered motion.
    assert!(matches!(
        h.client
            .request(Request::SetAppearance(AppAppearance::Light))?,
        Response::Preferences(AppAppearance::Light)
    ));
    let saved = h
        .client
        .execute(Operation::SetAppearance(AppAppearance::System), None)?;
    assert_eq!(saved.state, JobState::Succeeded);
    assert!(matches!(
        h.client.request(Request::AppSettings)?,
        Response::AppSettings(AppPreferences {
            appearance: AppAppearance::System,
            reduced_motion: true
        })
    ));
    h.client
        .request(Request::SetPreferences(PreferencesUpdate {
            reduced_motion: Some(false),
            ..Default::default()
        }))?;
    assert_eq!(AppPreferences::load(&h.paths)?, AppPreferences::default());
    let restarted = regressee_app::appearance::AppearanceSettings::start_client(h.client.clone());
    assert!(!restarted.wait_loaded().reduced_motion);
    h.client
        .request(Request::SetPreferences(PreferencesUpdate {
            reduced_motion: Some(true),
            ..Default::default()
        }))?;
    let deadline = Instant::now() + Duration::from_secs(3);
    while !restarted.snapshot().reduced_motion {
        ensure!(
            Instant::now() < deadline,
            "remote motion preference did not refresh"
        );
        thread::sleep(Duration::from_millis(10));
    }
    restarted.join_after_ui();
    Ok(())
}
impl Harness {
    fn new() -> Result<Self> {
        Self::from_root(tempfile::tempdir()?)
    }
    fn from_root(root: tempfile::TempDir) -> Result<Self> {
        let paths = AppPaths::isolated(root.path());
        let server = Server::without_runtime(paths.clone())?;
        let thread = thread::spawn(move || server.run());
        let client = Client::existing(paths.clone())?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while client.snapshot().is_err() {
            ensure!(Instant::now() < deadline, "server did not start");
            thread::sleep(Duration::from_millis(10));
        }
        Ok(Self {
            root,
            paths,
            thread: Some(thread),
            client,
        })
    }
    fn submit(&self, operation: Operation) -> Result<Job> {
        let s = self.client.snapshot()?;
        self.client.submit(Submit {
            request: Uuid::new_v4(),
            engine: s.engine,
            epoch: s.epoch,
            operation,
            confirmation: None,
            expected_revision: None,
        })
    }
}
impl Drop for Harness {
    fn drop(&mut self) {
        if let Ok(s) = self.client.snapshot() {
            for j in s.jobs {
                if !j.state.terminal() {
                    let _ = self.client.request(Request::Cancel { id: j.id });
                }
            }
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        while Instant::now() < deadline {
            if matches!(self.client.request(Request::Stop), Ok(Response::Ok)) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[test]
fn replacement_settings_cannot_bypass_confirmation_through_save_environment() -> Result<()> {
    use regressee_storage::LocalLoginSettings;
    let root = tempfile::tempdir()?;
    let (store, project, _, _, _) = fixture::fixture(root.path());
    let saved = store.load_project(project)?;
    drop(store);
    let h = Harness::from_root(root)?;
    let mut changed = saved.project.clone();
    changed.profiles[0].width += 16;
    assert!(
        h.submit(Operation::SaveReplacement {
            project: Box::new(changed.clone()),
            expected_revision: saved.revision
        })
        .is_err()
    );
    assert_eq!(
        h.client
            .execute(
                Operation::Save {
                    project: Box::new(changed.clone()),
                    expected_revision: Some(saved.revision)
                },
                None
            )?
            .state,
        JobState::Failed
    );
    let job = h.client.execute(
        Operation::SaveEnvironment {
            environment: changed.environments[0].id,
            settings: LocalLoginSettings::default(),
            project: Box::new(changed),
            revision: saved.revision,
        },
        None,
    )?;
    assert_eq!(
        job.state,
        JobState::Failed,
        "SaveEnvironment must not authorize replacement capture settings"
    );
    match h.client.execute(Operation::Load(project), None)?.outcome {
        Some(Outcome::Project(current)) => assert_eq!(current.project, saved.project),
        other => panic!("{other:?}"),
    }
    Ok(())
}

#[test]
fn project_removal_over_ipc_requires_exact_consent_and_preserves_other_projects() -> Result<()> {
    use regressee_storage::DeleteItem;
    let root = tempfile::tempdir()?;
    let (store, project, _, _, _) = fixture::fixture(root.path());
    drop(store);
    let (store, other, _, _, _) = fixture::fixture(root.path());
    let kept = serde_json::to_value(store.project_evidence(other)?)?;
    drop(store);
    let h = Harness::from_root(root)?;
    let preview = match h
        .client
        .execute(
            Operation::DeletePreview {
                project,
                item: DeleteItem::Project,
            },
            None,
        )?
        .outcome
    {
        Some(Outcome::DeletePreview(p)) => p,
        other => panic!("{other:?}"),
    };
    assert!(preview.shared_bytes > 0);
    assert!(h.submit(Operation::Delete(preview.clone())).is_err());
    let ticket = h.client.prepare(Operation::Delete(preview.clone()))?;
    let mut saved = match h.client.execute(Operation::Load(project), None)?.outcome {
        Some(Outcome::Project(p)) => p,
        other => panic!("{other:?}"),
    };
    saved.project.name = "Changed after preview".into();
    assert_eq!(
        h.client
            .execute(
                Operation::Save {
                    project: Box::new(saved.project),
                    expected_revision: Some(saved.revision)
                },
                None
            )?
            .state,
        JobState::Succeeded
    );
    let stale = h
        .client
        .execute(Operation::Delete(preview), Some(ticket.id));
    assert!(stale.is_err() || stale.is_ok_and(|j| j.state == JobState::Failed));
    let preview = match h
        .client
        .execute(
            Operation::DeletePreview {
                project,
                item: DeleteItem::Project,
            },
            None,
        )?
        .outcome
    {
        Some(Outcome::DeletePreview(p)) => p,
        other => panic!("{other:?}"),
    };
    let action = Operation::Delete(preview);
    let ticket = h.client.prepare(action.clone())?;
    let deleted = h.client.execute(action.clone(), Some(ticket.id))?;
    assert_eq!(deleted.state, JobState::Succeeded);
    assert!(
        matches!(deleted.outcome,Some(Outcome::ScopeDeleted {project:id,item:DeleteItem::Project,cleanup_pending:false}) if id==project)
    );
    let replay = h.client.execute(action, Some(ticket.id));
    assert!(replay.is_err() || replay.is_ok_and(|job| job.state == JobState::Failed));
    assert!(matches!(
        h.client.execute(Operation::Load(project), None)?.outcome,
        Some(Outcome::Failure(_))
    ));
    match h.client.execute(Operation::Evidence(other), None)?.outcome {
        Some(Outcome::Evidence(e)) => assert_eq!(serde_json::to_value(e)?, kept),
        other => panic!("{other:?}"),
    };
    Ok(())
}

#[test]
fn environment_removal_preserves_frozen_baselines_and_runs() -> Result<()> {
    use regressee_storage::DeleteItem;
    let root = tempfile::tempdir()?;
    let (mut store, project, run, _, _) = fixture::fixture(root.path());
    let mut saved = store.load_project(project)?;
    let baseline = saved.project.active_baseline.unwrap();
    let frozen = serde_json::to_value(store.load_baseline(baseline)?)?;
    let recorded = serde_json::to_value(store.load_run(run)?)?;
    let removed = saved.project.environments[0].id;
    let mut environment = saved.project.environments[0].clone();
    environment.id = regressee_core::domain::EnvironmentId::new();
    environment.name = "Retained staging".into();
    saved.project.environments.push(environment.clone());
    store.save_replacement_project(&saved.project, saved.revision)?;
    let preview = store.deletion_preview(project, DeleteItem::Environment(removed))?;
    store.delete_confirmed(&preview)?;
    let saved = store.load_project(project)?;
    assert_eq!(saved.project.active_baseline, Some(baseline));
    assert_eq!(
        store.selected_environment(&saved.project)?.id,
        environment.id
    );
    assert_eq!(
        serde_json::to_value(store.load_baseline(baseline)?)?,
        frozen
    );
    assert_eq!(serde_json::to_value(store.load_run(run)?)?, recorded);
    Ok(())
}

#[test]
fn pending_project_cleanup_does_not_block_engine_restart_or_project_browsing() -> Result<()> {
    use regressee_storage::DeleteItem;
    let root = tempfile::tempdir()?;
    let (mut store, project, run, _, _) = fixture::fixture(root.path());
    let other = regressee_core::domain::Project::draft(
        "Retained project".into(),
        "https://retained.example.test/",
    )?;
    store.save_project(&other, None)?;
    let record = store.load_run(run)?;
    let artifact = record.captures[0].image.original.clone();
    let path = store.paths().artifacts().join(artifact.as_str());
    let stash = root.path().join("owned-artifact-stash");
    let preview = store.deletion_preview(project, DeleteItem::Project)?;
    assert!(store.delete_confirmed_with_cleanup(&preview, |_| {
        std::fs::rename(&path, &stash)?;
        std::fs::create_dir(&path)?;
        Ok(())
    })?);
    assert!(store.summary()?.pending_cleanup > 0);
    drop(store);
    let h = Harness::from_root(root)?;
    assert!(
        matches!(h.client.execute(Operation::List {page:0},None)?.outcome,Some(Outcome::Projects(page)) if page.items.len()==1 && page.items[0].id==other.id)
    );
    assert!(
        matches!(h.client.execute(Operation::InspectStorage,None)?.outcome,Some(Outcome::Storage(summary)) if summary.pending_cleanup>0)
    );
    std::fs::remove_dir(path.clone())?;
    std::fs::rename(stash, path)?;
    Ok(())
}

#[test]
fn safe_environment_saves_and_confirmed_replacements_keep_distinct_atomic_intent() -> Result<()> {
    use regressee_storage::LocalLoginSettings;
    let root = tempfile::tempdir()?;
    let (store, project, _, _, _) = fixture::fixture(root.path());
    let original = store.load_project(project)?;
    drop(store);
    let h = Harness::from_root(root)?;
    let environment = original.project.environments[0].id;
    for (index, mut changed) in [
        original.project.clone(),
        original.project.clone(),
        original.project.clone(),
    ]
    .into_iter()
    .enumerate()
    {
        match index {
            0 => changed.sources[0] = changed.sources[0].join("other-path")?,
            1 => changed.profiles[0].width += 16,
            _ => changed.environments[0].name = "Changed environment".into(),
        };
        let rejected = h.client.execute(
            Operation::SaveEnvironment {
                project: Box::new(changed),
                revision: original.revision,
                environment,
                settings: LocalLoginSettings {
                    ephemeral: true,
                    ..Default::default()
                },
            },
            None,
        )?;
        assert_eq!(rejected.state, JobState::Failed);
        assert!(matches!(
            rejected.outcome,
            Some(Outcome::ProductFailure(message)) if message.starts_with("replacement_required:")
        ));
    }
    // Local login configuration and unprotected settings remain ordinary saves,
    // with no replacement token and no keychain/display/browser initialization.
    let mut safe = original.project.clone();
    safe.name = "Safe rename".into();
    safe.discovery.page_limit = 17;
    let local = LocalLoginSettings {
        ephemeral: true,
        login_address: Some(safe.sources[0].join("login")?),
        ..Default::default()
    };
    let safe = h.client.execute(
        Operation::SaveEnvironment {
            project: Box::new(safe),
            revision: original.revision,
            environment,
            settings: local.clone(),
        },
        None,
    )?;
    assert_eq!(safe.state, JobState::Succeeded);
    let Some(Outcome::Project(saved)) = safe.outcome else {
        panic!("Safe save lost its structured result")
    };
    assert_eq!(saved.revision, original.revision + 1);
    assert_eq!(
        saved.project.active_baseline,
        original.project.active_baseline
    );
    let Some(Outcome::Environment(status)) = h
        .client
        .execute(Operation::EnvironmentStatus(project), None)?
        .outcome
    else {
        panic!("Environment status missing")
    };
    assert_eq!(status.settings, local);
    let mut replacement = saved.project.clone();
    replacement.profiles[0].width += 16;
    let action = Operation::SaveReplacementEnvironment {
        project: Box::new(replacement.clone()),
        revision: saved.revision,
        environment,
        settings: local.clone(),
    };
    assert!(h.submit(action.clone()).is_err());
    let wrong_action = h.client.prepare(Operation::SaveReplacement {
        project: Box::new(replacement),
        expected_revision: saved.revision,
    })?;
    assert_eq!(
        h.client
            .execute(action.clone(), Some(wrong_action.id))?
            .state,
        JobState::Failed
    );
    let token = h.client.prepare(action.clone())?;
    let mut changed_action = action.clone();
    if let Operation::SaveReplacementEnvironment { settings, .. } = &mut changed_action {
        settings.ephemeral = false;
    }
    assert_eq!(
        h.client.execute(changed_action, Some(token.id))?.state,
        JobState::Failed
    );
    let replaced = h.client.execute(action.clone(), Some(token.id))?;
    assert_eq!(replaced.state, JobState::Succeeded);
    assert_eq!(
        h.client.submit(replaced.submitted.clone())?.state,
        JobState::Succeeded,
        "Retry retains the exact original job"
    );
    assert_eq!(
        h.client.execute(action, Some(token.id))?.state,
        JobState::Failed,
        "Consumed consent cannot authorize another ID"
    );
    let Some(Outcome::Project(current)) = h.client.execute(Operation::Load(project), None)?.outcome
    else {
        panic!("Project missing")
    };
    assert_eq!(current.revision, saved.revision + 1);
    // Even a confirmed replacement cannot activate/clear a baseline pointer.
    let mut invalid = current.project.clone();
    invalid.active_baseline = None;
    let invalid = Operation::SaveReplacementEnvironment {
        project: Box::new(invalid),
        revision: current.revision,
        environment,
        settings: local,
    };
    let consent = h.client.prepare(invalid.clone())?;
    assert_eq!(
        h.client.execute(invalid, Some(consent.id))?.state,
        JobState::Failed
    );
    assert!(!h.paths.data.join("private-sessions").exists());
    assert!(!h.paths.cache.join("product-runtime").exists());
    Ok(())
}

#[test]
fn capture_transport_preserves_structured_preflight_failures_instead_of_success_or_identity_errors()
-> Result<()> {
    use regressee_core::crawl::DiscoveryPreview;
    let root = tempfile::tempdir()?;
    let (store, project, run, _, _) = fixture::fixture(root.path());
    let saved = store.load_project(project)?;
    let inventory = store.load_run(run)?.run.inventory;
    drop(store);
    let preview = DiscoveryPreview {
        project_revision: saved.revision,
        settings: saved.project,
        inventory,
        aliases: vec![],
        notes: vec![],
        visited: 0,
    };
    let h = Harness::from_root(root)?;
    for operation in [
        Operation::Capture {
            project,
            check: true,
        },
        Operation::CaptureInventory {
            project,
            preview: None,
            check: false,
            resume: Some(run),
        },
        Operation::PreviewImages {
            project,
            preview: Box::new(preview),
        },
    ] {
        let job = h.client.execute(operation, None)?;
        assert_eq!(job.state, JobState::Failed);
        assert!(
            matches!(
                job.outcome,
                Some(Outcome::ActionRequired(
                    regressee_app::recovery::Problem::Browser(_)
                ))
            ),
            "{job:?}"
        );
        assert_eq!(
            h.client.job(job.id)?.state,
            JobState::Failed,
            "Failure is durable, not a successful evidence refresh"
        );
    }
    Ok(())
}

#[test]
fn legacy_workspace_capture_receipt_is_preserved_but_not_certified_or_replayed() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (store, project, _, _, _) = fixture::fixture(root.path());
    let workspace = store.project_evidence(project)?;
    let requested = workspace.baseline.as_ref().unwrap().references[0].run;
    let returned = workspace.latest.as_ref().unwrap().run.id;
    assert_ne!(requested, returned);
    drop(store);
    let id = Uuid::new_v4();
    let old = Job {
        owner: None,
        id,
        submitted: Submit {
            request: id,
            engine: Uuid::new_v4(),
            epoch: 0,
            operation: Operation::CaptureInventory {
                project,
                preview: None,
                check: false,
                resume: Some(requested),
            },
            confirmation: None,
            expected_revision: None,
        },
        state: JobState::Succeeded,
        frozen: None,
        frozen_revision: None,
        progress: None,
        capture: None,
        outcome: Some(Outcome::Evidence(Box::new(workspace))),
        sequence: 0,
    };
    let paths = AppPaths::isolated(root.path());
    let owner = Ownership::acquire(&paths)?;
    regressee_app::engine::ownership::write_json(
        &owner.directory.join(format!("job-{id}.json")),
        &old,
    )?;
    drop(owner);
    let mut h = Harness::from_root(root)?;
    for restart in [false, true] {
        if restart {
            h.client.request(Request::Stop)?;
            h.thread.take().unwrap().join().unwrap()?;
            let server = Server::without_runtime(h.paths.clone())?;
            h.thread = Some(thread::spawn(move || server.run()));
            let deadline = Instant::now() + Duration::from_secs(3);
            while h.client.snapshot().is_err() {
                ensure!(Instant::now() < deadline);
                thread::sleep(Duration::from_millis(10));
            }
        }
        let job = h.client.submit(old.submitted.clone())?;
        assert_eq!(job.state, JobState::Interrupted);
        match job.outcome {
            Some(Outcome::UnverifiedCapture { prior, .. }) => match *prior {
                Outcome::Evidence(e) => assert_eq!(e.latest.as_ref().unwrap().run.id, returned),
                other => panic!("Prior payload changed: {other:?}"),
            },
            other => panic!("Legacy receipt was certified: {other:?}"),
        }
        assert!(
            matches!(job.submitted.operation,Operation::CaptureInventory {resume:Some(run),..} if run==requested)
        );
    }
    Ok(())
}

#[test]
fn two_clients_one_owner_aliases_and_no_recovery_on_attach() -> Result<()> {
    let h = Harness::new()?;
    let other = Client::existing(h.paths.clone())?;
    let project = Project::draft("Shared fixture".into(), "https://example.test/")?;
    let saved = h.client.execute(
        Operation::Save {
            project: Box::new(project.clone()),
            expected_revision: None,
        },
        None,
    )?;
    assert_eq!(saved.state, JobState::Succeeded);
    match other.execute(Operation::Load(project.id), None)?.outcome {
        Some(Outcome::Project(p)) => assert_eq!(p.revision, 1),
        other => panic!("{other:?}"),
    }
    assert!(Ownership::acquire(&h.paths).is_err());
    #[cfg(unix)]
    {
        let alias = h.root.path().join("alias");
        std::os::unix::fs::symlink(&h.paths.data, &alias)?;
        let aliases = AppPaths {
            data: alias,
            cache: h.paths.cache.clone(),
        };
        assert!(Ownership::acquire(&aliases).is_err());
        assert_eq!(
            Client::existing(aliases)?.snapshot()?.engine,
            h.client.snapshot()?.engine
        );
    }
    let mut owners = vec![];
    for _ in 0..8 {
        let paths = h.paths.clone();
        owners.push(thread::spawn(move || {
            Server::without_runtime(paths).is_err()
        }));
    }
    assert!(owners.into_iter().all(|t| t.join().unwrap()));
    for _ in 0..20 {
        assert_eq!(other.snapshot()?.engine, h.client.snapshot()?.engine);
    }
    Ok(())
}

#[test]
fn attaching_clients_and_legacy_actor_cannot_recover_an_owned_running_row() -> Result<()> {
    use regressee_core::domain::{JobState as Capture, RunId, RunIntent, TargetOutcome};
    let root = tempfile::tempdir()?;
    let (mut store, project, run, _, _) = fixture::fixture(root.path());
    let mut running = store.load_run(run)?;
    running.run.id = RunId::new();
    running.run.intent = RunIntent::NewBaseline;
    running.run.state = Capture::Running;
    running.run.started_at = Some(200);
    running.run.finished_at = None;
    running.captures.clear();
    running.capture = None;
    for target in &mut running.run.targets {
        target.outcome = TargetOutcome::Pending;
        target.final_url = None;
        target.attempts = 0;
    }
    store.begin_run(&running)?;
    drop(store);
    let h = Harness::from_root(root)?;
    let (legacy, events) = regressee_app::jobs::Jobs::start_without_runtime(h.paths.clone());
    legacy
        .submit(regressee_app::jobs::Request {
            id: 1,
            work: regressee_app::jobs::Work::InspectStorage,
        })
        .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(2))?.reply,
        regressee_app::jobs::Reply::Failure(regressee_app::jobs::StorageFailure::Unavailable)
    ));
    legacy.join_after_ui();
    for _ in 0..5 {
        let client = Client::existing(h.paths.clone())?;
        client.snapshot()?;
        match client
            .execute(
                Operation::InspectRun {
                    project,
                    run: running.run.id,
                },
                None,
            )?
            .outcome
        {
            Some(Outcome::InspectedRun { evidence, .. }) => {
                assert_eq!(evidence.latest.unwrap().run.state, Capture::Running)
            }
            r => panic!("{r:?}"),
        }
    }
    Ok(())
}

#[test]
fn detached_jobs_progress_cancel_epoch_and_queued_revision_guards() -> Result<()> {
    let h = Harness::new()?;
    let job = h.submit(Operation::TestWait { millis: 1000 })?;
    let id = job.id;
    let detached = Client::existing(h.paths.clone())?;
    drop(detached);
    let deadline = Instant::now() + Duration::from_secs(2);
    while h.client.job(id)?.progress.is_none() {
        ensure!(Instant::now() < deadline, "no progress");
        thread::sleep(Duration::from_millis(10));
    }
    let start = Instant::now();
    let snapshot = h.client.snapshot()?;
    assert!(start.elapsed() < Duration::from_millis(500));
    assert!(
        h.client
            .request(Request::Stop)
            .is_ok_and(|r| matches!(r, Response::Error { .. }))
    );
    let project = Project::draft("Revoked queued draft".into(), "https://example.test/")?;
    let queued = h.submit(Operation::Save {
        project: Box::new(project.clone()),
        expected_revision: None,
    })?;
    h.client.request(Request::AdvanceEpoch {
        expected: snapshot.epoch,
    })?;
    h.client.request(Request::Cancel { id })?;
    assert_eq!(h.client.wait(id)?.state, JobState::Cancelled);
    assert_eq!(h.client.wait(queued.id)?.state, JobState::Failed);
    match h.client.execute(Operation::List { page: 0 }, None)?.outcome {
        Some(Outcome::Projects(p)) => assert!(p.items.is_empty()),
        r => panic!("{r:?}"),
    }
    let finished = h.submit(Operation::TestWait { millis: 100 })?;
    assert_eq!(
        h.client.wait(finished.id)?.state,
        JobState::Succeeded,
        "client exit is not cancellation"
    );
    h.client.request(Request::Cancel { id: finished.id })?;
    assert_eq!(
        h.client.job(finished.id)?.state,
        JobState::Succeeded,
        "late cancel cannot rewrite success"
    );
    Ok(())
}

#[test]
fn mutation_deduplication_survives_restart_and_refuses_changed_input() -> Result<()> {
    let mut h = Harness::new()?;
    let project = Project::draft("Exactly one create".into(), "https://example.test/")?;
    let admitted = h.submit(Operation::Save {
        project: Box::new(project.clone()),
        expected_revision: None,
    })?;
    let first = h.client.wait(admitted.id)?;
    assert_eq!(first.state, JobState::Succeeded);
    let again = h.client.submit(admitted.submitted.clone())?;
    assert_eq!(again.id, first.id);
    assert_eq!(again.state, JobState::Succeeded);
    let mut different = admitted.submitted.clone();
    different.operation = Operation::List { page: 0 };
    assert!(h.client.submit(different).is_err());
    assert!(matches!(h.client.request(Request::Stop)?, Response::Ok));
    h.thread.take().unwrap().join().unwrap()?;
    let server = Server::without_runtime(h.paths.clone())?;
    h.thread = Some(thread::spawn(move || server.run()));
    let deadline = Instant::now() + Duration::from_secs(3);
    while h.client.snapshot().is_err() {
        ensure!(Instant::now() < deadline, "restart failed");
        thread::sleep(Duration::from_millis(10));
    }
    assert_ne!(h.client.snapshot()?.engine, admitted.submitted.engine);
    assert_eq!(
        h.client.submit(admitted.submitted)?.state,
        JobState::Succeeded
    );
    match h.client.execute(Operation::Load(project.id), None)?.outcome {
        Some(Outcome::Project(p)) => assert_eq!(p.revision, 1),
        r => panic!("{r:?}"),
    }
    Ok(())
}

#[test]
fn exact_confirmation_rejects_missing_stale_wrong_action_and_replay() -> Result<()> {
    let h = Harness::new()?;
    let report = match h
        .client
        .execute(Operation::DiagnosticPreview, None)?
        .outcome
    {
        Some(Outcome::DiagnosticPreview(r)) => r,
        r => panic!("{r:?}"),
    };
    let action = Operation::DiagnosticExport {
        report: report.clone(),
        path: h.root.path().join("report.json"),
    };
    assert!(h.submit(action.clone()).is_err());
    let ticket = h.client.prepare(action.clone())?;
    assert!(
        h.client
            .execute(Operation::List { page: 0 }, Some(ticket.id))
            .is_err(),
        "A confirmation cannot silently switch to a token-free action"
    );
    assert!(
        h.client
            .execute(
                Operation::SetAppearance(regressee_core::appearance::AppAppearance::Dark),
                Some(ticket.id)
            )
            .is_err()
    );
    let wrong_action = h.client.execute(
        Operation::ForgetLogin(regressee_core::domain::ProjectId::new()),
        Some(ticket.id),
    )?;
    assert_eq!(wrong_action.state, JobState::Failed, "{wrong_action:?}");
    let wrong = Operation::DiagnosticExport {
        report: report.clone(),
        path: h.root.path().join("wrong.json"),
    };
    assert_eq!(
        h.client.execute(wrong, Some(ticket.id))?.state,
        JobState::Failed
    );
    assert_eq!(
        h.client.execute(action.clone(), Some(ticket.id))?.state,
        JobState::Succeeded
    );
    assert_eq!(
        h.client.execute(action.clone(), Some(ticket.id))?.state,
        JobState::Failed
    );
    let action = Operation::DiagnosticExport {
        report,
        path: h.root.path().join("stale.json"),
    };
    let stale = h.client.prepare(action.clone())?;
    let p = Project::draft("Revision changes consent".into(), "https://example.test/")?;
    h.client.execute(
        Operation::Save {
            project: Box::new(p),
            expected_revision: None,
        },
        None,
    )?;
    assert_eq!(
        h.client.execute(action, Some(stale.id))?.state,
        JobState::Failed
    );
    assert!(!h.root.path().join("wrong.json").exists());
    assert!(!h.root.path().join("stale.json").exists());
    let action = Operation::ForgetLogin(regressee_core::domain::ProjectId::new());
    let consent = h.client.prepare(action.clone())?;
    let epoch = h.client.snapshot()?.epoch;
    h.client
        .request(Request::AdvanceEpoch { expected: epoch })?;
    let rejected = h.client.execute(action, Some(consent.id))?;
    assert_eq!(rejected.state, JobState::Failed);
    assert!(
        matches!(rejected.outcome,Some(Outcome::ProductFailure(message)) if message.contains("confirmation_conflict"))
    );
    Ok(())
}

#[test]
fn snapshot_events_reconnect_and_terminal_receipts_are_not_dropped() -> Result<()> {
    let h = Harness::new()?;
    let before = h.client.snapshot()?;
    let job = h.submit(Operation::TestWait { millis: 100 })?;
    let final_job = h.client.wait(job.id)?;
    assert_eq!(final_job.state, JobState::Succeeded);
    let other = Client::existing(h.paths.clone())?;
    match other.request(Request::Events {
        engine: before.engine,
        after: before.sequence,
    })? {
        Response::Events {
            events,
            resync: false,
            ..
        } => assert!(
            events
                .iter()
                .any(|e| e.job == job.id && e.state == JobState::Succeeded)
        ),
        r => panic!("{r:?}"),
    }
    assert!(matches!(
        other.request(Request::Events {
            engine: Uuid::new_v4(),
            after: 0
        })?,
        Response::Events { resync: true, .. }
    ));
    assert!(matches!(
        other.request(Request::Events {
            engine: before.engine,
            after: u64::MAX
        })?,
        Response::Events { resync: true, .. }
    ));
    assert_eq!(other.job(job.id)?.state, JobState::Succeeded);
    Ok(())
}

#[test]
fn queued_expected_revision_is_checked_at_execution() -> Result<()> {
    let h = Harness::new()?;
    let p = Project::draft("Frozen revision".into(), "https://example.test/")?;
    h.client.execute(
        Operation::Save {
            project: Box::new(p.clone()),
            expected_revision: None,
        },
        None,
    )?;
    let wait = h.submit(Operation::TestWait { millis: 200 })?;
    let mut changed = p.clone();
    changed.name = "Changed first".into();
    let first = h.submit(Operation::Save {
        project: Box::new(changed),
        expected_revision: Some(1),
    })?;
    let mut stale = h.submit(Operation::Load(p.id))?.submitted;
    // A new queued mutation retains its original precondition, not a refreshed value.
    stale.request = Uuid::new_v4();
    stale.operation = Operation::Save {
        project: Box::new(p.clone()),
        expected_revision: Some(1),
    };
    stale.expected_revision = Some(1);
    let second = h.client.submit(stale)?;
    h.client.wait(wait.id)?;
    assert_eq!(h.client.wait(first.id)?.state, JobState::Succeeded);
    assert_eq!(h.client.wait(second.id)?.state, JobState::Failed);
    match h.client.execute(Operation::Load(p.id), None)?.outcome {
        Some(Outcome::Project(p)) => {
            assert_eq!(p.revision, 2);
            assert_eq!(p.project.name, "Changed first");
        }
        r => panic!("{r:?}"),
    }
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
fn malformed_frames_version_handshake_and_slow_clients_are_bounded() -> Result<()> {
    use sha2::{Digest, Sha256};
    use std::{
        io::{Read, Write},
        os::linux::net::SocketAddrExt,
        os::unix::net::{SocketAddr, UnixStream},
    };
    let h = Harness::new()?;
    let paths = std::fs::canonicalize(&h.paths.data)?;
    let key = hex::encode(Sha256::digest(paths.as_os_str().as_encoded_bytes()));
    let address = SocketAddr::from_abstract_name(format!("regressee-engine-{}-{key}", unsafe {
        libc::geteuid()
    }))?;
    let connect = || -> Result<UnixStream> {
        let s = UnixStream::connect_addr(&address)?;
        s.set_read_timeout(Some(Duration::from_secs(2)))?;
        Ok(s)
    };
    let mut old = connect()?;
    let hello = serde_json::to_vec(&Request::Hello {
        version: VERSION + 1,
    })?;
    old.write_all(&(hello.len() as u32).to_be_bytes())?;
    old.write_all(&hello)?;
    let mut header = [0; 4];
    old.read_exact(&mut header)?;
    let mut reply = vec![0; u32::from_be_bytes(header) as usize];
    old.read_exact(&mut reply)?;
    assert!(
        matches!(serde_json::from_slice::<Response>(&reply)?,Response::Error {code,..} if code=="version_mismatch")
    );
    let count = h.client.snapshot()?.total_jobs;
    let mut huge = connect()?;
    huge.write_all(&((MAX_FRAME + 1) as u32).to_be_bytes())?;
    let start = Instant::now();
    let rejected = huge.read(&mut [0]);
    assert!(
        matches!(rejected, Ok(0))
            || rejected.is_err_and(|e| matches!(
                e.kind(),
                std::io::ErrorKind::ConnectionReset | std::io::ErrorKind::ConnectionAborted
            ))
    );
    assert!(start.elapsed() < Duration::from_millis(500));
    let mut partial = connect()?;
    partial.write_all(&[0, 0])?;
    let start = Instant::now();
    assert_eq!(h.client.snapshot()?.total_jobs, count);
    assert!(start.elapsed() < Duration::from_millis(500));
    drop(partial);
    let job = h.submit(Operation::TestWait { millis: 200 })?;
    let mut slow_clients = Vec::new();
    for _ in 0..MAX_CONNECTIONS {
        let mut stream = connect()?;
        stream.write_all(&[0, 0])?;
        slow_clients.push(stream);
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while h.client.snapshot().is_ok() {
        ensure!(Instant::now() < deadline, "Connection cap was not enforced");
        thread::sleep(Duration::from_millis(10));
    }
    thread::sleep(Duration::from_millis(250));
    drop(slow_clients);
    let deadline = Instant::now() + Duration::from_secs(2);
    while h.client.snapshot().is_err() {
        ensure!(
            Instant::now() < deadline,
            "Disconnected readers did not release capacity"
        );
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(
        h.client.job(job.id)?.state,
        JobState::Succeeded,
        "Terminal receipts survive stalled readers and connection overload"
    );
    assert!(!h.paths.data.join("private-sessions").exists());
    Ok(())
}

#[test]
fn interrupted_receipt_is_durable_and_never_automatically_reexecuted() -> Result<()> {
    let mut h = Harness::new()?;
    let p = Project::draft("Uncertain receipt".into(), "https://example.test/")?;
    let job = h.client.execute(
        Operation::Save {
            project: Box::new(p.clone()),
            expected_revision: None,
        },
        None,
    )?;
    h.client.request(Request::Stop)?;
    h.thread.take().unwrap().join().unwrap()?;
    // Owned fault injection models a crash after product commit but before the
    // terminal receipt. The product data has committed exactly once.
    let mut interrupted = job.clone();
    interrupted.state = JobState::Running;
    interrupted.outcome = None;
    std::fs::write(
        h.paths.data.join(format!("engine-v1/job-{}.json", job.id)),
        serde_json::to_vec(&interrupted)?,
    )?;
    let server = Server::without_runtime(h.paths.clone())?;
    h.thread = Some(thread::spawn(move || server.run()));
    let deadline = Instant::now() + Duration::from_secs(3);
    while h.client.snapshot().is_err() {
        ensure!(Instant::now() < deadline, "restart failed");
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(h.client.submit(job.submitted)?.state, JobState::Interrupted);
    match h.client.execute(Operation::Load(p.id), None)?.outcome {
        Some(Outcome::Project(p)) => assert_eq!(p.revision, 1),
        r => panic!("{r:?}"),
    }
    Ok(())
}

#[test]
fn event_overflow_requires_snapshot_but_retains_terminal_outcome() -> Result<()> {
    let h = Harness::new()?;
    let before = h.client.snapshot()?;
    let job = h.submit(Operation::TestWait { millis: 6000 })?;
    assert_eq!(h.client.wait(job.id)?.state, JobState::Succeeded);
    assert!(matches!(
        h.client.request(Request::Events {
            engine: before.engine,
            after: before.sequence
        })?,
        Response::Events { resync: true, .. }
    ));
    assert_eq!(h.client.job(job.id)?.state, JobState::Succeeded);
    Ok(())
}

#[cfg(unix)]
#[test]
fn unsafe_lock_files_and_directory_permissions_are_refused_without_repair() -> Result<()> {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = tempfile::tempdir()?;
    let paths = AppPaths::isolated(root.path());
    {
        let _guard = Ownership::acquire(&paths)?;
    }
    std::fs::remove_file(paths.data.join("engine-owner.lock"))?;
    let untouched = root.path().join("untouched");
    std::fs::write(&untouched, b"untouched")?;
    symlink(&untouched, paths.data.join("engine-owner.lock"))?;
    assert!(Server::without_runtime(paths.clone()).is_err());
    assert_eq!(std::fs::read(&untouched)?, b"untouched");
    assert!(!paths.database().exists());
    std::fs::remove_file(paths.data.join("engine-owner.lock"))?;
    std::fs::set_permissions(&paths.data, std::fs::Permissions::from_mode(0o755))?;
    assert!(Ownership::acquire(&paths).is_err());
    assert_eq!(
        std::fs::metadata(&paths.data)?.permissions().mode() & 0o777,
        0o755
    );
    Ok(())
}

#[test]
fn saved_evidence_approval_and_archive_services_share_the_engine_handlers() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (store, project, _, analysis, _) = fixture::fixture(root.path());
    drop(store);
    let h = Harness::from_root(root)?;
    let evidence = match h
        .client
        .execute(Operation::Evidence(project), None)?
        .outcome
    {
        Some(Outcome::Evidence(e)) => e,
        r => panic!("{r:?}"),
    };
    let key = evidence.latest.as_ref().unwrap().run.targets[0].key.clone();
    let baseline = evidence.baseline.as_ref().unwrap().id;
    let export = match h
        .client
        .execute(Operation::ExportPreview { project, baseline }, None)?
        .outcome
    {
        Some(Outcome::ExportPreview(p)) => p,
        r => panic!("{r:?}"),
    };
    let path = h.root.path().join("baseline.zip");
    let action = Operation::Export {
        preview: export,
        path: path.clone(),
    };
    let consent = h.client.prepare(action.clone())?;
    assert_eq!(
        h.client.execute(action, Some(consent.id))?.state,
        JobState::Succeeded
    );
    let import = match h
        .client
        .execute(Operation::ImportPreview(path.clone()), None)?
        .outcome
    {
        Some(Outcome::ImportPreview(p)) => p,
        r => panic!("{r:?}"),
    };
    let action = Operation::Import {
        preview: import,
        path,
    };
    let consent = h.client.prepare(action.clone())?;
    assert_eq!(
        h.client.execute(action, Some(consent.id))?.state,
        JobState::Succeeded
    );
    assert_eq!(
        h.client
            .execute(Operation::LoadAnalysis(analysis), None)?
            .state,
        JobState::Succeeded
    );
    let approval = match h
        .client
        .execute(
            Operation::Approval {
                analysis,
                selected: vec![key],
            },
            None,
        )?
        .outcome
    {
        Some(Outcome::Approval(p)) => p,
        r => panic!("{r:?}"),
    };
    let action = Operation::Accept {
        context: approval.context.clone(),
        preview: approval,
    };
    let consent = h.client.prepare(action.clone())?;
    let accepted = h.client.execute(action, Some(consent.id))?;
    assert_eq!(accepted.state, JobState::Succeeded);
    let saved = match accepted.outcome {
        Some(Outcome::Project(p)) => p,
        r => panic!("{r:?}"),
    };
    assert!(saved.project.active_baseline.is_some());
    assert_ne!(saved.project.active_baseline, Some(baseline));
    let latest = match h
        .client
        .execute(
            Operation::Viewer(ViewerSelection {
                project,
                run: None,
                tile: 0,
                key: None,
                analysis: None,
            }),
            None,
        )?
        .outcome
    {
        Some(Outcome::Viewer(p)) => p,
        r => panic!("{r:?}"),
    };
    assert!(latest.panes.iter().flatten().all(|file| file.path.exists()));
    assert_eq!(
        h.client.execute(Operation::RuntimeStatus, None)?.state,
        JobState::Succeeded
    );
    assert!(!h.paths.data.join("private-sessions").exists());
    Ok(())
}

#[test]
fn queued_capture_freezes_admitted_revision_without_opening_a_browser() -> Result<()> {
    let h = Harness::new()?;
    let p = Project::draft("Frozen admission".into(), "https://example.test/")?;
    h.client.execute(
        Operation::Save {
            project: Box::new(p.clone()),
            expected_revision: None,
        },
        None,
    )?;
    let wait = h.submit(Operation::TestWait { millis: 200 })?;
    let mut changed = p.clone();
    changed.name = "New configuration".into();
    let saved = h.submit(Operation::Save {
        project: Box::new(changed),
        expected_revision: Some(1),
    })?;
    let capture = h.submit(Operation::Capture {
        project: p.id,
        check: false,
    })?;
    assert_eq!(capture.frozen_revision, Some(1));
    h.client.wait(wait.id)?;
    assert_eq!(h.client.wait(saved.id)?.state, JobState::Succeeded);
    let failed = h.client.wait(capture.id)?;
    assert_eq!(failed.state, JobState::Failed);
    assert!(
        matches!(failed.outcome,Some(Outcome::ProductFailure(message)) if message.contains("frozen project changed"))
    );
    assert!(!h.paths.cache.join("product-runtime").exists());
    Ok(())
}

#[test]
#[ignore = "Subprocess-only owned engine fixture; invoked explicitly by its parent test"]
fn engine_child_process() -> Result<()> {
    let root = std::env::var_os("REGRESSEE_ENGINE_TEST_CHILD_ROOT")
        .context("owned child fixture root required")?;
    let root = std::path::PathBuf::from(root);
    ensure!(
        std::fs::read(root.join("owned-engine-test"))? == b"synthetic child fixture",
        "Not an owned child fixture"
    );
    Server::without_runtime(AppPaths::isolated(&root))?.run()
}

#[cfg(unix)]
#[test]
fn foreground_ctrl_c_drains_without_cancelling_work_and_crash_reconnect_is_truthful() -> Result<()>
{
    use std::process::{Command, Stdio};
    struct Child(std::process::Child);
    impl std::ops::Deref for Child {
        type Target = std::process::Child;
        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }
    impl std::ops::DerefMut for Child {
        fn deref_mut(&mut self) -> &mut Self::Target {
            &mut self.0
        }
    }
    impl Drop for Child {
        fn drop(&mut self) {
            let _ = self.0.kill();
            let _ = self.0.wait();
        }
    }
    let root = tempfile::tempdir()?;
    std::fs::write(
        root.path().join("owned-engine-test"),
        b"synthetic child fixture",
    )?;
    let paths = AppPaths::isolated(root.path());
    drop(Ownership::acquire(&paths)?);
    let spawn = || {
        Command::new(std::env::current_exe().unwrap())
            .args([
                "--ignored",
                "--exact",
                "engine_child_process",
                "--nocapture",
            ])
            .env("REGRESSEE_ENGINE_TEST_CHILD_ROOT", root.path())
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
    };
    let mut child = Child(spawn()?);
    let client = Client::existing(paths.clone())?;
    let wait_ready = || -> Result<()> {
        let deadline = Instant::now() + Duration::from_secs(5);
        while client.snapshot().is_err() {
            ensure!(Instant::now() < deadline, "child failed readiness");
            thread::sleep(Duration::from_millis(10));
        }
        Ok(())
    };
    wait_ready()?;
    let live = client.execute(Operation::List { page: 0 }, None)?;
    assert_eq!(live.state, JobState::Succeeded);
    let snapshot = client.snapshot()?;
    let job = client.submit(Submit {
        request: Uuid::new_v4(),
        engine: snapshot.engine,
        epoch: 0,
        operation: Operation::TestWait { millis: 1500 },
        confirmation: None,
        expected_revision: None,
    })?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while client.job(job.id)?.progress.is_none() {
        ensure!(Instant::now() < deadline, "child job failed to start");
        thread::sleep(Duration::from_millis(10));
    }
    unsafe {
        ensure!(
            libc::kill(child.id() as i32, libc::SIGINT) == 0,
            "signal failed"
        );
    }
    let deadline = Instant::now() + Duration::from_secs(2);
    while !client.snapshot()?.draining {
        ensure!(Instant::now() < deadline, "child did not drain");
        thread::sleep(Duration::from_millis(10));
    }
    assert!(
        !client.job(job.id)?.state.terminal(),
        "Ctrl-C is not implicit job cancellation"
    );
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Some(status) = child.try_wait()? {
            assert!(status.success());
            break;
        }
        ensure!(Instant::now() < deadline, "child did not finish drain");
        thread::sleep(Duration::from_millis(10));
    }
    let path = paths.data.join(format!("engine-v1/job-{}.json", job.id));
    let completed: Job = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(completed.state, JobState::Succeeded);
    child = Child(spawn()?);
    wait_ready()?;
    let snapshot = client.snapshot()?;
    let unfinished = client.submit(Submit {
        request: Uuid::new_v4(),
        engine: snapshot.engine,
        epoch: 0,
        operation: Operation::TestWait { millis: 10000 },
        confirmation: None,
        expected_revision: None,
    })?;
    let deadline = Instant::now() + Duration::from_secs(2);
    while client.job(unfinished.id)?.state != JobState::Running {
        ensure!(Instant::now() < deadline, "child job not running");
        thread::sleep(Duration::from_millis(10));
    }
    child.kill()?;
    child.wait()?;
    child = Child(spawn()?);
    wait_ready()?;
    assert_eq!(
        client.submit(unfinished.submitted)?.state,
        JobState::Interrupted
    );
    assert!(matches!(client.request(Request::Stop)?, Response::Ok));
    assert!(child.wait()?.success());
    Ok(())
}
