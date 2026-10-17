// SPDX-License-Identifier: MPL-2.0
use super::*;
use regressee_core::domain::Project;
use std::sync::mpsc;
#[cfg(feature = "ui")]
fn viewer_work(project: ProjectId, tile: usize) -> Work {
    Work::Extension(ViewerWork::Viewer {
        project,
        tile,
        key: None,
        analysis: None,
    })
}
#[cfg(feature = "ui")]
#[test]
fn viewer_mailbox_coalesces_and_prioritizes_commands_then_latest_then_prefetch() {
    let project = ProjectId::new();
    let mailbox = Mutex::new(ViewerMailbox::default());
    let (tx, rx) = mpsc::sync_channel(1);
    assert!(
        mailbox
            .lock()
            .unwrap()
            .submit(Request {
                id: 0,
                work: Work::List { page: 0 }
            })
            .is_err()
    );
    for id in 1..=100 {
        mailbox
            .lock()
            .unwrap()
            .submit(Request {
                id,
                work: viewer_work(project, id as usize),
            })
            .unwrap();
    }
    mailbox.lock().unwrap().prefetch = Some(viewer_work(project, 101));
    tx.send(Request {
        id: 200,
        work: Work::InspectStorage,
    })
    .unwrap();
    let (request, speculative, generation) = next_viewer_work(&rx, &mailbox).unwrap();
    assert_eq!(request.id, 200);
    assert!(!speculative && generation.is_none());
    let (request, speculative, generation) = next_viewer_work(&rx, &mailbox).unwrap();
    assert_eq!(
        (request.id, speculative, generation),
        (100, false, Some(100))
    );
    let (_, speculative, generation) = next_viewer_work(&rx, &mailbox).unwrap();
    assert!(speculative && generation.is_none());
    assert!(next_viewer_work(&rx, &mailbox).is_none());
    mailbox.lock().unwrap().prefetch = Some(viewer_work(project, 102));
    mailbox
        .lock()
        .unwrap()
        .submit(Request {
            id: 103,
            work: viewer_work(project, 103),
        })
        .unwrap();
    assert!(mailbox.lock().unwrap().prefetch.is_none());
}
#[cfg(feature = "ui")]
#[test]
fn viewer_invalidation_covers_refresh_display_and_mutations_not_browsing() {
    let project = ProjectId::new();
    for work in [
        Work::Evidence(project),
        Work::LoadAnalysis(regressee_core::domain::AnalysisId::new()),
        Work::Activate {
            run: RunId::new(),
            revision: 1,
            confirm_partial: false,
        },
        Work::Capture {
            project,
            check: true,
        },
    ] {
        assert!(invalidates_viewer(&work));
    }
    for work in [
        Work::History { project, page: 0 },
        Work::ProjectStorage(project),
        Work::Load(project),
        Work::List { page: 0 },
        viewer_work(project, 0),
    ] {
        assert!(!invalidates_viewer(&work));
    }
}
#[cfg(feature = "ui")]
#[test]
fn viewer_publication_drops_superseded_backpressure_and_stops() {
    let (tx, rx) = mpsc::sync_channel(1);
    tx.send(Update {
        id: 0,
        reply: Reply::Failure(StorageFailure::Unavailable),
    })
    .unwrap();
    let mailbox = Arc::new(Mutex::new(ViewerMailbox::default()));
    mailbox
        .lock()
        .unwrap()
        .submit(Request {
            id: 1,
            work: viewer_work(ProjectId::new(), 0),
        })
        .unwrap();
    let stop = Arc::new(AtomicBool::new(false));
    let m = mailbox.clone();
    let s = stop.clone();
    let t = tx.clone();
    let worker = thread::spawn(move || {
        publish_viewer(
            &t,
            Update {
                id: 1,
                reply: Reply::Failure(StorageFailure::Unavailable),
            },
            &s,
            &m,
            1,
        )
    });
    mailbox
        .lock()
        .unwrap()
        .submit(Request {
            id: 2,
            work: viewer_work(ProjectId::new(), 0),
        })
        .unwrap();
    assert!(worker.join().unwrap());
    assert_eq!(rx.recv().unwrap().id, 0);
    assert!(rx.try_recv().is_err());
    assert!(publish_viewer(
        &tx,
        Update {
            id: 2,
            reply: Reply::Failure(StorageFailure::Unavailable)
        },
        &stop,
        &mailbox,
        2
    ));
    stop.store(true, Ordering::Release);
    assert!(!publish_viewer(
        &tx,
        Update {
            id: 3,
            reply: Reply::Failure(StorageFailure::Unavailable)
        },
        &stop,
        &mailbox,
        2
    ));
    assert_eq!(rx.recv().unwrap().id, 2);
}
#[cfg(feature = "ui")]
#[test]
fn viewer_jobs_normal_submit_is_latest_wins_prefetch_is_silent_and_shutdown_rejects()
-> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let paths = AppPaths::isolated(root.path());
    let (ready, wait) = mpsc::channel();
    let (release, gate) = mpsc::channel();
    let (jobs, events) = Jobs::start_inner(paths, move |paths| {
        ready.send(()).unwrap();
        gate.recv().unwrap();
        Ok(Product {
            distribution: None,
            official_runtime: false,
            runtime_cache: paths.cache.join("product-runtime"),
            auth: Default::default(),
        })
    });
    wait.recv_timeout(Duration::from_secs(5))?;
    let project = Project::draft("Latest navigation".into(), "https://example.test/")?;
    jobs.submit(Request {
        id: 10,
        work: Work::Save {
            project: Box::new(project.clone()),
            expected_revision: None,
        },
    })
    .unwrap();
    for id in 1..=100 {
        jobs.submit(Request {
            id,
            work: viewer_work(project.id, 0),
        })
        .unwrap();
    }
    assert!(
        jobs.submit_viewer(Request {
            id: 0,
            work: Work::InspectStorage
        })
        .is_err()
    );
    assert!(jobs.prefetch_viewer(Work::InspectStorage).is_err());
    jobs.prefetch_viewer(viewer_work(project.id, 0)).unwrap();
    release.send(())?;
    assert_eq!(events.recv_timeout(Duration::from_secs(5))?.id, 10);
    // Save invalidates speculative work, but never the latest read intent.
    assert_eq!(events.recv_timeout(Duration::from_secs(5))?.id, 100);
    jobs.prefetch_viewer(viewer_work(project.id, 0)).unwrap();
    assert!(
        events.recv_timeout(Duration::from_millis(80)).is_err(),
        "prefetch publishes neither success nor failure"
    );
    jobs.join_after_ui();
    assert!(
        jobs.submit_viewer(Request {
            id: 101,
            work: viewer_work(project.id, 0)
        })
        .is_err()
    );
    assert!(jobs.prefetch_viewer(viewer_work(project.id, 0)).is_err());
    Ok(())
}
