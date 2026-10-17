// SPDX-License-Identifier: MPL-2.0
#![cfg(feature = "test-support")]
use anyhow::{Result, ensure};
use regressee_app::{
    control::*,
    engine::{Client, Server},
    protocol::{self, JobState, Operation, Outcome, Request, Response, Submit},
};
use regressee_core::domain::{Project, TargetKey};
use regressee_storage::AppPaths;
use std::{
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;
#[path = "support/approval_fixture.rs"]
mod fixture;

struct Host {
    root: tempfile::TempDir,
    client: Client,
    thread: Option<thread::JoinHandle<Result<()>>>,
}
impl Host {
    fn new(root: tempfile::TempDir) -> Result<Self> {
        let server = Server::without_runtime(AppPaths::isolated(root.path()))?;
        let client = Client::existing(AppPaths::isolated(root.path()))?;
        let thread = Some(thread::spawn(move || server.run()));
        let deadline = Instant::now() + Duration::from_secs(5);
        while client.snapshot().is_err() {
            ensure!(Instant::now() < deadline);
            thread::sleep(Duration::from_millis(10));
        }
        Ok(Self {
            root,
            client,
            thread,
        })
    }
    fn agent(&self, name: &str) -> Result<Client> {
        let Response::Credential(c) = self.client.control(Command::Start {
            name: name.into(),
            seconds: 90,
        })?
        else {
            panic!()
        };
        Ok(self.client.with_credential(c))
    }
    fn frontend(&self) -> Result<Client> {
        let c = self.client.register_frontend()?;
        Ok(self.client.with_credential(c))
    }
    fn stop(&mut self) {
        if self.thread.is_none() {
            return;
        }
        let f = self.frontend().unwrap();
        let _ = f.control(Command::Takeover);
        for j in self.client.snapshot().unwrap().jobs {
            if !j.state.terminal() {
                let _ = self.client.request(Request::Cancel { id: j.id });
            }
        }
        for _ in 0..500 {
            if matches!(self.client.request(Request::Stop), Ok(Response::Ok)) {
                break;
            }
            thread::sleep(Duration::from_millis(10));
        }
        self.thread.take().unwrap().join().unwrap().unwrap();
    }
}
impl Drop for Host {
    fn drop(&mut self) {
        self.stop();
    }
}
fn id(client: &Client) -> Uuid {
    match client.credential().unwrap().owner {
        Owner::Agent(id) | Owner::Frontend(id) => id,
    }
}
fn submit(client: &Client, op: Operation) -> Result<protocol::Job> {
    let s = client.snapshot()?;
    client.submit(Submit {
        request: Uuid::new_v4(),
        engine: s.engine,
        epoch: s.epoch,
        operation: op,
        confirmation: None,
        expected_revision: None,
    })
}
fn denied(result: Result<Response>) {
    assert!(result.is_err() || matches!(result, Ok(Response::Error { .. })));
}

#[test]
fn anonymous_ipc_cannot_mint_gui_authority_and_bootstrap_is_private_incarnation_bound() -> Result<()>
{
    let mut h = Host::new(tempfile::tempdir()?)?;
    let agent = h.agent("No GUI impostor")?;
    let epoch = h.client.snapshot()?.epoch;
    let engine = h.client.snapshot()?.engine;
    for command in [
        Command::RegisterFrontend,
        Command::Follow {
            session: id(&agent),
        },
        Command::Takeover,
        Command::Return,
        Command::Detach,
        Command::Heartbeat,
        Command::Renew { seconds: 300 },
    ] {
        denied(h.client.uncredentialed().request(Request::Control(command)));
    }
    let forged = FrontendCapability {
        engine,
        secret: Uuid::new_v4(),
    };
    denied(
        h.client
            .request(Request::RegisterFrontend { capability: forged }),
    );
    let path = h
        .client
        .paths
        .data
        .join("engine-v1/frontend-capability.json");
    regressee_app::engine::ownership::private_file(&path, 4096)?;
    let capability: FrontendCapability = serde_json::from_slice(&std::fs::read(&path)?)?;
    assert!(!format!("{capability:?}").contains(&capability.secret.to_string()));
    denied(agent.request(Request::RegisterFrontend {
        capability: capability.clone(),
    }));
    let frontend = h.frontend()?; // deliberate private-file bootstrap, not raw IPC
    assert_eq!(
        h.client.snapshot()?.epoch,
        epoch,
        "registration is not a takeover"
    );
    let unrelated = h.frontend()?;
    unrelated.control(Command::Detach)?;
    denied(unrelated.control(Command::Heartbeat));
    assert_eq!(
        h.client.snapshot()?.epoch,
        epoch,
        "closing an unrelated GUI must not revoke a controller"
    );
    for _ in 0..3 {
        h.client.request(Request::Status)?;
        h.client.control(Command::List)?;
    }
    let Response::Control(status) = h.client.control(Command::List)? else {
        panic!()
    };
    assert_eq!(status.controller, Some(id(&agent)));
    assert!(status.sessions[0].follow.is_none());
    assert!(
        submit(
            &h.client,
            Operation::Save {
                project: Box::new(Project::draft(
                    "No anonymous write".into(),
                    "https://example.test"
                )?),
                expected_revision: None
            }
        )
        .is_err()
    );
    // Lease-only operations prove their original owner but do not acquire new
    // navigation/mutation authority, alter epochs, or change saved data.
    frontend.control(Command::Follow {
        session: id(&agent),
    })?;
    let current = h.client.snapshot()?;
    for (client, command) in [
        (&agent, Command::Renew { seconds: 120 }),
        (&frontend, Command::Heartbeat),
    ] {
        assert!(matches!(
            h.client.request(Request::Authorized {
                credential: client.credential().unwrap(),
                epoch,
                request: Box::new(Request::Control(command))
            })?,
            Response::Control(_)
        ));
    }
    assert_eq!(h.client.snapshot()?.epoch, current.epoch);
    assert_eq!(h.client.snapshot()?.sequence, current.sequence);
    frontend.control(Command::Takeover)?; // legitimate human remains immediate
    denied(agent.control(Command::Renew { seconds: 90 }));
    let snapshot = serde_json::to_string(&h.client.control(Command::List)?)?;
    assert!(!snapshot.contains(&capability.secret.to_string()));
    h.stop();
    assert!(!path.exists(), "normal shutdown removes the bootstrap file");
    let server = Server::without_runtime(h.client.paths.clone())?;
    h.thread = Some(thread::spawn(move || server.run()));
    while h.client.snapshot().is_err() {
        thread::sleep(Duration::from_millis(5));
    }
    denied(h.client.request(Request::RegisterFrontend { capability }));
    assert!(h.client.snapshot()?.engine != engine);
    Ok(())
}

#[test]
fn lifecycle_exclusivity_handoff_takeover_and_private_receipts() -> Result<()> {
    let h = Host::new(tempfile::tempdir()?)?;
    let a = h.agent("Release review")?;
    let frontend = h.frontend()?;
    let other = h.frontend()?;
    assert!(h.agent("Intruder").is_err());
    let project = Project::draft("Owned".into(), "https://example.test/")?;
    let save = Operation::Save {
        project: Box::new(project.clone()),
        expected_revision: None,
    };
    assert!(submit(&h.client, save.clone()).is_err());
    assert!(submit(&frontend, save.clone()).is_err());
    denied(h.client.control(Command::Follow { session: id(&a) }));
    denied(a.control(Command::Follow { session: id(&a) }));
    denied(a.control(Command::RegisterFrontend));
    denied(h.client.control(Command::Takeover));
    assert_eq!(a.execute(save, None)?.state, JobState::Succeeded);
    assert_eq!(
        h.client.execute(Operation::Load(project.id), None)?.state,
        JobState::Succeeded
    );
    frontend.control(Command::Follow { session: id(&a) })?;
    a.control(Command::Handoff {
        message: "Please review the project settings".into(),
    })?;
    let mut changed = project;
    changed.name = "Human edit".into();
    let save = Operation::Save {
        project: Box::new(changed.clone()),
        expected_revision: Some(1),
    };
    assert!(submit(&a, save.clone()).is_err());
    assert!(submit(&other, save.clone()).is_err());
    denied(other.control(Command::Return));
    assert_eq!(frontend.execute(save, None)?.state, JobState::Succeeded);
    frontend.control(Command::Return)?;
    assert!(
        submit(
            &frontend,
            Operation::SetAppearance(regressee_core::appearance::AppAppearance::Dark)
        )
        .is_err()
    );
    denied(frontend.request(Request::SetAppearance(
        regressee_core::appearance::AppAppearance::Dark,
    )));
    frontend.control(Command::Takeover)?;
    assert!(
        submit(
            &a,
            Operation::SetAppearance(regressee_core::appearance::AppAppearance::Dark)
        )
        .is_err()
    );
    denied(a.control(Command::Renew { seconds: 90 }));
    assert_eq!(
        h.client.execute(Operation::Load(changed.id), None)?.state,
        JobState::Succeeded
    );
    // The session secret never enters any persisted job or ordinary status DTO.
    let secret = a.credential().unwrap().secret.to_string();
    let snapshot = serde_json::to_string(&h.client.control(Command::List)?)?;
    assert!(!snapshot.contains(&secret));
    for e in std::fs::read_dir(h.client.paths.data.join("engine-v1"))? {
        let e = e?;
        if e.file_name().to_string_lossy().starts_with("job-") {
            assert!(!std::fs::read_to_string(e.path())?.contains(&secret));
        }
    }
    Ok(())
}

#[test]
fn takeover_rejects_queued_writes_and_stale_raw_controls_without_cancelling_inflight() -> Result<()>
{
    let h = Host::new(tempfile::tempdir()?)?;
    let a = h.agent("Queue test")?;
    let f = h.frontend()?;
    let running = submit(&a, Operation::TestWait { millis: 500 })?;
    while a.job(running.id)?.state == JobState::Queued {
        thread::sleep(Duration::from_millis(5));
    }
    let project = Project::draft("Must not save".into(), "https://example.test/")?;
    let queued = submit(
        &a,
        Operation::Save {
            project: Box::new(project.clone()),
            expected_revision: None,
        },
    )?;
    let old = a.snapshot()?;
    f.control(Command::Takeover)?;
    let raw = Request::Authorized {
        credential: a.credential().unwrap(),
        epoch: old.epoch,
        request: Box::new(Request::Cancel { id: running.id }),
    };
    denied(h.client.request(raw));
    denied(a.request(Request::Pause { id: running.id }));
    assert_eq!(h.client.wait(running.id)?.state, JobState::Succeeded);
    let rejected = h.client.wait(queued.id)?;
    assert_eq!(rejected.state, JobState::Failed);
    assert!(
        matches!(rejected.outcome,Some(Outcome::ProductFailure(m)) if m.starts_with("stale_epoch:"))
    );
    assert!(matches!(
        h.client.execute(Operation::Load(project.id), None)?.outcome,
        Some(Outcome::Failure(_))
    ));
    // Identical old requests only return their actual receipt; they cannot replay.
    assert_eq!(
        a.submit(rejected.submitted.clone())?.state,
        JobState::Failed
    );
    assert_eq!(
        a.submit(h.client.job(running.id)?.submitted)?.state,
        JobState::Succeeded
    );
    Ok(())
}

#[test]
fn semantic_selection_is_exact_validated_and_headless_and_confirmation_is_frontend_bound()
-> Result<()> {
    let root = tempfile::tempdir()?;
    let (store, project, run, analysis, _) = fixture::fixture(root.path());
    let saved = store.load_project(project)?;
    let evidence = store.project_evidence_for_run(project, run)?;
    let key = evidence.latest.as_ref().unwrap().run.targets[0].key.clone();
    drop(store);
    let h = Host::new(root)?;
    let a = h.agent("Headless inspection")?;
    let selection = Selection {
        project,
        run: Some(run),
        analysis: Some(analysis),
        key: Some(key),
        view: View::Split,
    };
    let j = a.execute(Operation::SelectAgent(selection.clone()), None)?;
    assert_eq!(j.state, JobState::Succeeded);
    assert!(
        matches!(&j.outcome, Some(Outcome::AgentSelected {selection: selected}) if selected == &selection)
    );
    assert!(
        serde_json::to_vec(&j)?.len() < 16 * 1024,
        "selection receipts retain IDs, not whole run evidence"
    );
    let Response::Control(control) = h.client.control(Command::List)? else {
        panic!()
    };
    assert_eq!(control.sessions[0].selection, Some(selection.clone()));
    assert!(control.sessions[0].follow.is_none());
    let mut wrong = selection.clone();
    wrong.key = Some(TargetKey {
        page: regressee_core::domain::PageId::new(),
        profile: saved.project.profiles[0].id,
    });
    assert_eq!(
        a.execute(Operation::SelectAgent(wrong), None)?.state,
        JobState::Failed
    );
    let mut wrong = selection;
    wrong.project = regressee_core::domain::ProjectId::new();
    assert_eq!(
        a.execute(Operation::SelectAgent(wrong), None)?.state,
        JobState::Failed
    );
    a.control(Command::End)?;
    let f = h.frontend()?;
    let other = h.frontend()?;
    let baseline = saved.project.active_baseline.unwrap();
    let restore = Operation::Restore {
        baseline,
        revision: saved.revision,
    };
    let token = f.prepare(restore.clone())?;
    let j = other.execute(restore.clone(), Some(token.id))?;
    assert_eq!(j.state, JobState::Failed);
    assert_eq!(
        f.execute(restore, Some(token.id))?.state,
        JobState::Succeeded
    );
    Ok(())
}

#[test]
fn restart_revokes_credentials_but_keeps_committed_receipts() -> Result<()> {
    let mut h = Host::new(tempfile::tempdir()?)?;
    let a = h.agent("Restart")?;
    let project = Project::draft("Saved".into(), "https://example.test/")?;
    let job = a.execute(
        Operation::Save {
            project: Box::new(project.clone()),
            expected_revision: None,
        },
        None,
    )?;
    let read = a.execute(Operation::Load(project.id), None)?;
    denied(h.client.request(Request::Acknowledge { id: read.id }));
    a.control(Command::End)?;
    h.stop();
    let server = Server::without_runtime(AppPaths::isolated(h.root.path()))?;
    h.thread = Some(thread::spawn(move || server.run()));
    while h.client.snapshot().is_err() {
        thread::sleep(Duration::from_millis(10));
    }
    denied(a.control(Command::Renew { seconds: 90 }));
    // Read recovery is intentionally available without former write authority.
    assert_eq!(h.client.job(job.id)?.state, JobState::Succeeded);
    assert_eq!(h.client.submit(job.submitted)?.state, JobState::Succeeded);
    assert!(matches!(
        h.client.request(Request::Acknowledge { id: read.id })?,
        Response::Ok
    ));
    denied(h.client.request(Request::Acknowledge { id: job.id }));
    assert_eq!(
        h.client.execute(Operation::Load(project.id), None)?.state,
        JobState::Succeeded
    );
    Ok(())
}

#[test]
fn raw_nonowner_control_confirmation_and_forged_credential_paths_fail_closed() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (store, project, _, _, _) = fixture::fixture(root.path());
    let saved = store.load_project(project)?;
    drop(store);
    let h = Host::new(root)?;
    let a = h.agent("Authority matrix")?;
    let f = h.frontend()?;
    let restore = Operation::Restore {
        baseline: saved.project.active_baseline.unwrap(),
        revision: saved.revision,
    };
    assert!(h.client.prepare(restore.clone()).is_err());
    assert!(f.prepare(restore.clone()).is_err());
    let confirmation = a.prepare(restore.clone())?;
    let snapshot = a.snapshot()?;
    for r in [
        Request::Stop,
        Request::AdvanceEpoch {
            expected: snapshot.epoch,
        },
        Request::SetAppearance(regressee_core::appearance::AppAppearance::Dark),
        Request::Cancel { id: Uuid::new_v4() },
        Request::Pause { id: Uuid::new_v4() },
    ] {
        denied(h.client.request(r.clone()));
        denied(f.request(r));
    }
    let mut forged = a.credential().unwrap();
    forged.owner = Owner::Agent(Uuid::new_v4());
    denied(h.client.request(Request::Authorized {
        credential: forged,
        epoch: snapshot.epoch,
        request: Box::new(Request::Control(Command::End)),
    }));
    let blocker = submit(&a, Operation::TestWait { millis: 300 })?;
    while a.job(blocker.id)?.state == JobState::Queued {
        thread::sleep(Duration::from_millis(5));
    }
    let queued = a.submit(Submit {
        request: Uuid::new_v4(),
        engine: snapshot.engine,
        epoch: snapshot.epoch,
        operation: restore,
        confirmation: Some(confirmation.id),
        expected_revision: None,
    })?;
    f.control(Command::Takeover)?;
    assert_eq!(h.client.wait(blocker.id)?.state, JobState::Succeeded);
    assert_eq!(h.client.wait(queued.id)?.state, JobState::Failed);
    assert_eq!(
        h.client.execute(Operation::Load(project), None)?.state,
        JobState::Succeeded
    );
    Ok(())
}
