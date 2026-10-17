// SPDX-License-Identifier: MPL-2.0
#![cfg(all(feature = "ui", feature = "test-support"))]
#[path = "../../app/tests/support/approval_fixture.rs"]
mod fixture;
use regressee_app::{
    engine::{Client, Server},
    protocol::{self, Operation, Response},
};
use regressee_gui::jobs::{Jobs, Reply, Request, ViewerWork, Work};
use regressee_storage::AppPaths;
use std::{
    sync::Arc,
    thread,
    time::{Duration, Instant},
};

#[test]
fn production_facade_reads_engine_artifacts_coalesces_and_detaches() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let (store, project, _, analysis, _) = fixture::fixture(root.path());
    drop(store);
    let paths = AppPaths::isolated(root.path());
    let server = Server::without_runtime(paths.clone())?;
    let server_thread = thread::spawn(move || server.run());
    let client = Client::existing(paths)?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while client.snapshot().is_err() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    let (gui, events) = Jobs::connect(client.clone());
    assert!(gui.is_remote());
    gui.appearance.wait_loaded();
    client.request(protocol::Request::SetAppearance(
        regressee_core::appearance::AppAppearance::Dark,
    ))?;
    let deadline = Instant::now() + Duration::from_secs(3);
    while gui.appearance.snapshot().preference != regressee_core::appearance::AppAppearance::Dark {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(20));
    }
    gui.submit(Request {
        id: 1,
        work: Work::Load(project),
    })
    .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(5))?.reply,
        Reply::Project(_)
    ));
    let blocker = client.execute(Operation::List { page: 0 }, None)?;
    assert!(blocker.state.terminal());
    let snapshot = client.snapshot()?;
    let blocked = client.submit(protocol::Submit {
        request: uuid::Uuid::new_v4(),
        engine: snapshot.engine,
        epoch: snapshot.epoch,
        operation: Operation::TestWait { millis: 300 },
        confirmation: None,
        expected_revision: None,
    })?;
    for id in 100..=199 {
        gui.submit_viewer(Request {
            id,
            work: Work::Extension(ViewerWork::Viewer {
                project,
                tile: 0,
                key: None,
                analysis: Some(analysis),
            }),
        })
        .unwrap();
    }
    let frame = loop {
        let update = events.recv_timeout(Duration::from_secs(5))?;
        if update.id == 199 {
            match update.reply {
                Reply::Extension(frame) => break frame,
                r => panic!("{r:?}"),
            }
        }
    };
    assert!(frame.identity.is_some());
    assert_eq!(frame.panes.len(), 3);
    assert!(frame.panes.iter().flatten().count() > 0);
    gui.submit_viewer(Request {
        id: 200,
        work: Work::Extension(ViewerWork::Viewer {
            project,
            tile: 0,
            key: None,
            analysis: Some(analysis),
        }),
    })
    .unwrap();
    let update = events.recv_timeout(Duration::from_secs(5))?;
    assert_eq!(update.id, 200);
    match update.reply {
        Reply::Extension(next) => assert!(Arc::ptr_eq(
            frame.panes.iter().flatten().next().unwrap(),
            next.panes.iter().flatten().next().unwrap()
        )),
        r => panic!("{r:?}"),
    }
    assert!(client.wait(blocked.id)?.state.terminal());
    let saved = match client.execute(Operation::Load(project), None)?.outcome {
        Some(protocol::Outcome::Project(p)) => p,
        r => panic!("{r:?}"),
    };
    gui.take_data_changed();
    let mut changed = saved.project.clone();
    changed.name = "Changed by independent client".into();
    client.execute(
        Operation::Save {
            project: Box::new(changed),
            expected_revision: Some(saved.revision),
        },
        None,
    )?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while !gui.take_data_changed() {
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(20));
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let refreshed = loop {
        if let Some(p) = gui.saved_project(project, saved.revision) {
            break p;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(refreshed.project.name, "Changed by independent client");
    gui.submit(Request {
        id: 201,
        work: Work::Load(project),
    })
    .unwrap();
    match events.recv_timeout(Duration::from_secs(5))?.reply {
        Reply::Project(p) => assert_eq!(p.project.name, "Changed by independent client"),
        r => panic!("{r:?}"),
    }
    let saved = match client.execute(Operation::Load(project), None)?.outcome {
        Some(protocol::Outcome::Project(p)) => p,
        r => panic!("{r:?}"),
    };
    let mut replacement = saved.project.clone();
    replacement.profiles[0].width += 16;
    let environment = replacement.environments[0].id;
    gui.submit(Request {
        id: 301,
        work: Work::SaveEnvironment {
            project: Box::new(replacement.clone()),
            revision: saved.revision,
            environment,
            settings: Default::default(),
        },
    })
    .unwrap();
    assert!(matches!(
        events.recv_timeout(Duration::from_secs(5))?.reply,
        Reply::ProductFailure(message) if message.starts_with("replacement_required:")
    ));
    gui.submit(Request {
        id: 302,
        work: Work::SaveReplacementEnvironment {
            project: Box::new(replacement),
            revision: saved.revision,
            environment,
            settings: Default::default(),
        },
    })
    .unwrap();
    match events.recv_timeout(Duration::from_secs(5))?.reply {
        Reply::Project(p) => assert_eq!(p.revision, saved.revision + 1),
        r => panic!("{r:?}"),
    }
    let snapshot = client.snapshot()?;
    let live = client.submit(protocol::Submit {
        request: uuid::Uuid::new_v4(),
        engine: snapshot.engine,
        epoch: snapshot.epoch,
        operation: Operation::TestWait { millis: 2000 },
        confirmation: None,
        expected_revision: None,
    })?;
    gui.submit(Request {
        id: 202,
        work: Work::Evidence(project),
    })
    .unwrap();
    // Let the GUI's read be admitted behind the blocking job; detaching must
    // not wait for that read or cancel either the read or the blocking work.
    thread::sleep(Duration::from_millis(100));
    let detached_at = Instant::now();
    gui.join_after_ui();
    drop(gui);
    assert!(detached_at.elapsed() < Duration::from_secs(1));
    assert!(!client.job(live.id)?.state.terminal());
    assert_eq!(client.wait(live.id)?.state, protocol::JobState::Succeeded);
    for job in client.snapshot()?.jobs {
        if !job.state.terminal() {
            client.wait(job.id)?;
        }
    }
    assert!(matches!(
        client.request(protocol::Request::Stop)?,
        Response::Ok
    ));
    server_thread.join().unwrap()?;
    Ok(())
}
