// SPDX-License-Identifier: MPL-2.0
//! Opt-in public-API verifier. Signed inventories and PNG evidence are synthetic;
//! smoke callbacks are inert, never worker/browser or runtime qualification.
#![cfg(all(target_os = "linux", target_arch = "x86_64"))]

use anyhow::{Context, Result, ensure};
use ed25519_dalek::{Signer, SigningKey};
use regressee_app::{
    jobs::{Jobs, Reply, Request, Update, Work},
    product::{Product, RuntimeState},
};
use regressee_core::{domain::*, evidence::*};
use regressee_runtime::distribution::{
    self, Distribution, Entry, InstallProgress, Manifest, Scope,
};
use regressee_storage::{AppPaths, Store};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    net::TcpListener,
    os::unix::fs::PermissionsExt,
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
    },
    time::{Duration, Instant},
};
use zip::{ZipWriter, write::SimpleFileOptions};

fn private_root() -> Result<tempfile::TempDir> {
    let base = PathBuf::from(
        std::env::var_os("REGRESSEE_INDEPENDENT_ROOT")
            .context("explicit private verifier root required")?,
    );
    distribution::safe_directory(&base)?;
    ensure!(
        fs::metadata(&base)?.permissions().mode() & 0o777 == 0o700,
        "root must be private 0700"
    );
    let dir = tempfile::Builder::new()
        .prefix("public-api-")
        .tempdir_in(base)?;
    fs::set_permissions(dir.path(), fs::Permissions::from_mode(0o700))?;
    Ok(dir)
}

fn sentinel() -> Result<(TcpListener, Distribution, SigningKey)> {
    let listener = TcpListener::bind("127.0.0.1:0")?;
    listener.set_nonblocking(true)?;
    let key = SigningKey::from_bytes(&[137; 32]); // private test key, never app default trust
    let distribution = Distribution {
        version: 1,
        url: format!("http://{}/must-not-download", listener.local_addr()?).parse()?,
        public_key: hex::encode(key.verifying_key().to_bytes()),
        scope: Scope::DevelopmentLoopback,
    };
    Ok((listener, distribution, key))
}

fn zero_connections(listener: &TcpListener) -> Result<()> {
    match listener.accept() {
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => Ok(()),
        _ => anyhow::bail!("unexpected runtime download connection"),
    }
}

fn product(cache: PathBuf, distribution: Option<Distribution>) -> Product {
    Product {
        distribution,
        runtime_cache: cache,
        auth: Default::default(),
    }
}

fn inert_pack(root: &Path, distribution: &Distribution, key: &SigningKey) -> Result<PathBuf> {
    let payload = b"INERT independent inventory: no executable runtime";
    let entry = Entry {
        sha256: distribution::hash(&payload[..], &AtomicBool::new(false))?,
        bytes: payload.len() as u64,
        executable: false,
    };
    let manifest = Manifest {
        version: 2,
        id: "independent-inert".into(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        scope: distribution.scope,
        support_until: (distribution.scope == Scope::QualifiedPublic)
            .then(|| distribution::now() + 3600),
        identity: RuntimeIdentity::Pinned {
            node: "inert-independent".into(),
            worker: "inert-independent".into(),
            playwright: "inert-independent".into(),
            browser: "inert-independent".into(),
        },
        node: "node/inert".into(),
        worker: "worker/inert".into(),
        browser_root: "browsers".into(),
        files: BTreeMap::from([
            ("node/inert".into(), entry.clone()),
            ("worker/inert".into(), entry.clone()),
            ("browsers/inert".into(), entry),
        ]),
        links: BTreeMap::new(),
        modes: BTreeMap::new(),
    };
    let bytes = serde_json::to_vec(&manifest)?;
    let path = root.join("inert.zip");
    let mut zip = ZipWriter::new(File::create(&path)?);
    let options = SimpleFileOptions::default();
    zip.start_file("manifest.json", options)?;
    zip.write_all(&bytes)?;
    zip.start_file("manifest.sig", options)?;
    zip.write_all(&key.sign(&bytes).to_bytes())?;
    for name in manifest.files.keys() {
        zip.start_file(name, options)?;
        zip.write_all(payload)?;
    }
    zip.finish()?;
    Ok(path)
}

#[test]
#[ignore = "private inert verifier; requires REGRESSEE_INDEPENDENT_ROOT; no worker/browser"]
fn readiness_and_refusals_are_read_only_without_network() -> Result<()> {
    let root = private_root()?;
    let (listener, trusted, _) = sentinel()?;
    let cache = root.path().join("runtime");
    let cancel = Arc::new(AtomicBool::new(false));
    for configured in [
        None,
        Some(Distribution {
            public_key: "invalid".into(),
            ..trusted.clone()
        }),
    ] {
        let owner = product(cache.clone(), configured);
        let state = owner.readiness(&cancel)?;
        assert_eq!(state.status, RuntimeState::UnavailableDistribution);
        assert!(!state.setup_available);
        assert!(state.identity.is_none());
        assert!(!state.message.contains("must-not-download"));
        let mut phases = vec![];
        assert!(
            owner
                .ensure_runtime(cancel.clone(), &mut |phase| phases.push(phase))
                .is_err()
        );
        assert!(phases.is_empty());
        assert!(!cache.exists());
    }
    let owner = product(cache.clone(), Some(trusted));
    let state = owner.readiness(&cancel)?;
    assert_eq!(state.status, RuntimeState::NotInstalled);
    assert!(state.setup_available);
    assert_eq!(state.scope, Some(Scope::DevelopmentLoopback));
    assert!(!cache.exists());
    assert!(owner.status(&cancel).is_err());
    cancel.store(true, Ordering::Release);
    let mut phases = vec![];
    assert!(
        owner
            .ensure_runtime(cancel.clone(), &mut |phase| phases.push(phase))
            .is_err()
    );
    assert!(owner.readiness(&cancel).is_err());
    assert!(phases.is_empty());
    assert!(!cache.exists());
    let paths = AppPaths::isolated(&root.path().join("invalid-config-actor"));
    let (jobs, replies) = Jobs::start_with_distribution(
        paths.clone(),
        Distribution {
            public_key: "invalid".into(),
            ..owner.distribution.clone().unwrap()
        },
    );
    let actor = Actor { jobs, replies };
    assert!(
        matches!(actor.terminal(1, Work::RuntimeStatus)?, Reply::RuntimeReadiness(state)
        if state.status == RuntimeState::UnavailableDistribution && !state.setup_available
        && !state.message.contains("must-not-download"))
    );
    assert!(!paths.cache.join("product-runtime").exists());
    drop(actor);
    zero_connections(&listener)
}

#[test]
#[ignore = "private inert verifier; requires REGRESSEE_INDEPENDENT_ROOT; no worker/browser"]
fn public_ensure_reuses_ready_and_refuses_damage_links_and_expiry() -> Result<()> {
    let root = private_root()?;
    let (listener, trusted, key) = sentinel()?;
    let cancel = Arc::new(AtomicBool::new(false));
    let owner = product(root.path().join("runtime"), Some(trusted.clone()));
    fs::create_dir(&owner.runtime_cache)?;
    let current = owner.runtime_cache.join("current");
    for pointer in ["../escape", "runtime-missing"] {
        fs::write(&current, pointer)?;
        let mut phases = vec![];
        assert_eq!(
            owner.readiness(&cancel)?.status,
            RuntimeState::RepairRequired
        );
        assert!(
            owner
                .ensure_runtime(cancel.clone(), &mut |phase| phases.push(phase))
                .is_err()
        );
        assert!(phases.is_empty());
        assert_eq!(fs::read_to_string(&current)?, pointer);
    }
    fs::remove_file(&current)?;
    std::os::unix::fs::symlink("nonexistent", &current)?;
    assert!(!distribution::current_absent(&owner.runtime_cache)?);
    assert!(
        owner
            .ensure_runtime(cancel.clone(), &mut |_| panic!(
                "dangling pointer downloaded"
            ))
            .is_err()
    );
    assert!(fs::symlink_metadata(&current)?.file_type().is_symlink());
    fs::remove_file(&current)?;
    let pack = inert_pack(root.path(), &trusted, &key)?;
    let installed =
        distribution::install(&pack, &owner.runtime_cache, &trusted, &cancel, &mut |_| {
            Ok(())
        })?;
    let pointer = fs::read(&current)?;
    let ready = owner.readiness(&cancel)?;
    assert_eq!(ready.status, RuntimeState::Ready);
    assert_eq!(ready.identity, Some(installed.manifest.identity.clone()));
    let resolved = owner.ensure_runtime(cancel.clone(), &mut |_| panic!("ready redownloaded"))?;
    assert_eq!(resolved.root, installed.root);
    fs::write(installed.root.join("READY"), b"wrong-ready-id")?;
    assert_eq!(
        owner.readiness(&cancel)?.status,
        RuntimeState::RepairRequired
    );
    assert!(
        owner
            .ensure_runtime(cancel.clone(), &mut |_| panic!("READY repaired"))
            .is_err()
    );
    assert_eq!(fs::read(installed.root.join("READY"))?, b"wrong-ready-id");
    fs::write(
        installed.root.join("READY"),
        installed.manifest.id.as_bytes(),
    )?;
    fs::write(installed.root.join("node/inert"), b"tampered")?;
    assert_eq!(
        owner.readiness(&cancel)?.status,
        RuntimeState::RepairRequired
    );
    assert!(
        owner
            .ensure_runtime(cancel.clone(), &mut |_| panic!("tamper repaired"))
            .is_err()
    );
    assert_eq!(fs::read(&current)?, pointer);
    assert_eq!(fs::read(installed.root.join("node/inert"))?, b"tampered");
    let linked = root.path().join("linked-runtime");
    std::os::unix::fs::symlink(&owner.runtime_cache, &linked)?;
    let unsafe_owner = product(linked, Some(trusted.clone()));
    assert_eq!(
        unsafe_owner.readiness(&cancel)?.status,
        RuntimeState::RepairRequired
    );
    assert!(
        unsafe_owner
            .ensure_runtime(cancel.clone(), &mut |_| panic!("linked cache installed"))
            .is_err()
    );
    let public = Distribution {
        scope: Scope::QualifiedPublic,
        url: "https://invalid.example/inert-only".parse()?,
        ..trusted
    };
    let public_root = root.path().join("signed-public-fixture");
    fs::create_dir(&public_root)?;
    let pack = inert_pack(&public_root, &public, &key)?;
    let expired_owner = product(public_root.join("runtime"), Some(public.clone()));
    let installed = distribution::install(
        &pack,
        &expired_owner.runtime_cache,
        &public,
        &cancel,
        &mut |_| Ok(()),
    )?;
    let mut manifest = installed.manifest;
    manifest.support_until = Some(distribution::now() - 1);
    let bytes = serde_json::to_vec(&manifest)?;
    fs::write(installed.root.join("manifest.json"), &bytes)?;
    fs::write(
        installed.root.join("manifest.sig"),
        key.sign(&bytes).to_bytes(),
    )?;
    assert_eq!(
        expired_owner.readiness(&cancel)?.status,
        RuntimeState::RepairRequired
    );
    let error = expired_owner
        .ensure_runtime(cancel.clone(), &mut |_| panic!("expired replaced"))
        .unwrap_err();
    assert!(format!("{error:#}").contains("support expired"));
    zero_connections(&listener)
}

fn baseline(root: &Path) -> Result<(Store, ProjectId, RunId)> {
    let mut store = Store::open(AppPaths::isolated(root))?;
    let mut project = Project::draft(
        "Independent synthetic baseline".into(),
        "http://127.0.0.1:43210/never-visit",
    )?;
    project.profiles[0].width = 1;
    project.profiles[0].height = 1;
    project.profiles[0].runtime = RuntimeIdentity::Pinned {
        node: "synthetic".into(),
        worker: "synthetic".into(),
        playwright: "synthetic".into(),
        browser: "synthetic".into(),
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
    store.freeze_inventory(&inventory)?;
    let target = TargetKey {
        page: inventory.pages[0].id,
        profile: project.profiles[0].id,
    };
    let mut record = RunRecord {
        run: CaptureRun {
            id: RunId::new(),
            project: project.id,
            intent: RunIntent::NewBaseline,
            inventory,
            profiles: project.profiles.clone(),
            environment: project.environments[0].clone(),
            rendering: RenderingEnvironment {
                os: "synthetic".into(),
                architecture: "synthetic".into(),
                browser: "synthetic".into(),
                worker_version: "synthetic".into(),
                profile_version: 1,
                fonts_fingerprint: None,
            },
            state: JobState::Running,
            started_at: Some(1),
            finished_at: None,
            targets: vec![CaptureTarget {
                key: target.clone(),
                requested: project.sources[0].clone(),
                final_url: None,
                attempts: 1,
                outcome: TargetOutcome::Pending,
                warnings: vec![],
            }],
        },
        capture: None,
        captures: vec![],
        queue: None,
    };
    store.begin_run(&record)?;
    let png_path = root.join("synthetic.png");
    let mut encoder = png::Encoder::new(File::create(&png_path)?, 1, 1);
    encoder.set_color(png::ColorType::Rgb);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.write_header()?.write_image_data(&[8, 120, 111])?;
    let cancel = AtomicBool::new(false);
    let image =
        regressee_app::images::ingest(&mut store, &png_path, ArtifactKind::Screenshot, &cancel)?;
    let artifact = store.verify_artifact(&image.original)?;
    record.run.targets[0].outcome = TargetOutcome::Captured {
        artifact: image.original.clone(),
        width: 1,
        height: 1,
    };
    record.run.targets[0].final_url = Some(project.sources[0].clone());
    record.captures.push(CaptureEvidence {
        metadata: CaptureMetadata {
            version: 1,
            key: target,
            requested: project.sources[0].to_string(),
            final_url: project.sources[0].to_string(),
            runtime: project.profiles[0].runtime.clone(),
            profile: project.profiles[0].clone(),
            width: 1,
            height: 1,
            sha256: image.original.clone(),
            bytes: artifact.bytes,
            warnings: vec![],
            stable: true,
            duration_ms: 1,
            readiness: Readiness {
                font_failures: 0,
                image_failures: 0,
                resource_failures: 0,
            },
            capture_mode: "synthetic-no-browser".into(),
            replacement: None,
        },
        image,
    });
    record.run.state = JobState::Completed;
    record.run.finished_at = Some(2);
    store.finish_run(&record)?;
    store.activate_baseline(record.run.id, 1)?;
    Ok((store, project.id, record.run.id))
}

fn evidence_snapshot(store: &Store, project: ProjectId, run: RunId) -> Result<Vec<u8>> {
    let saved = store.load_project(project)?;
    let baseline =
        store.load_baseline(saved.project.active_baseline.context("baseline missing")?)?;
    let record = store.load_run(run)?;
    let mut hashes = BTreeMap::new();
    for capture in record.all_captures() {
        for artifact in capture.image.artifacts() {
            let verified = store.verify_artifact(&artifact)?;
            hashes.insert(
                artifact.as_str().to_owned(),
                (
                    verified.bytes,
                    distribution::hash(
                        File::open(store.artifact_path(&artifact)?)?,
                        &AtomicBool::new(false),
                    )?,
                ),
            );
        }
    }
    Ok(serde_json::to_vec(&(
        saved.revision,
        saved.project,
        baseline,
        record,
        hashes,
    ))?)
}

struct Actor {
    jobs: Arc<Jobs>,
    replies: Receiver<Update>,
}
impl Drop for Actor {
    fn drop(&mut self) {
        self.jobs.join_after_ui();
    }
}
impl Actor {
    fn terminal(&self, id: u64, work: Work) -> Result<Reply> {
        self.jobs
            .submit(Request { id, work })
            .map_err(anyhow::Error::msg)?;
        let update = self.replies.recv_timeout(Duration::from_secs(3))?;
        ensure!(update.id == id, "reply changed request identity");
        ensure!(
            !matches!(update.reply, Reply::Progress { .. }),
            "guarded request emitted setup progress"
        );
        Ok(update.reply)
    }
}

#[test]
#[ignore = "private inert verifier; requires REGRESSEE_INDEPENDENT_ROOT; no worker/browser"]
fn public_jobs_routes_pinned_work_and_queued_cancel_preserving_cas() -> Result<()> {
    let root = private_root()?;
    let (mut store, project, run) = baseline(root.path())?;
    let fresh = Project::draft(
        "Independent first use".into(),
        "http://127.0.0.1:43210/never-visit",
    )?;
    store.save_project(&fresh, None)?;
    let before = evidence_snapshot(&store, project, run)?;
    let paths = store.paths().clone();
    // Deliberately invalid private network settings: if setup is requested for
    // NEW work, it refuses before creating a downloader, connection, or worker.
    fs::write(paths.cache.join("network-settings.json"), b"invalid-json")?;
    drop(store);
    let (listener, trusted, _) = sentinel()?;
    let (jobs, replies) = Jobs::start_with_distribution(paths.clone(), trusted);
    let actor = Actor { jobs, replies };
    assert!(
        matches!(actor.terminal(1, Work::RuntimeStatus)?, Reply::RuntimeReadiness(state) if state.status == RuntimeState::NotInstalled && state.setup_available)
    );
    let works = [
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
            resume: Some(run),
        },
        Work::CaptureInventory {
            project,
            preview: None,
            check: true,
            resume: Some(run),
        },
        Work::OpenLogin(project),
        Work::ProbeEnvironment(project),
    ];
    for (index, work) in works.into_iter().enumerate() {
        let reply = actor.terminal(index as u64 + 2, work)?;
        assert!(
            matches!(reply, Reply::ProductFailure(ref message) if !message.contains("network settings")),
            "pinned request entered setup: {reply:?}"
        );
    }
    assert!(matches!(
        actor.terminal(8, Work::ExistingInventory(project))?,
        Reply::Discovery(_)
    ));
    for (id, work) in [
        (9, Work::Discover(project)),
        (
            10,
            Work::Capture {
                project,
                check: false,
            },
        ),
        (
            11,
            Work::CaptureInventory {
                project,
                preview: None,
                check: false,
                resume: None,
            },
        ),
    ] {
        assert!(
            matches!(actor.terminal(id, work)?, Reply::ProductFailure(message) if message.contains("network settings")),
            "intentional new/replacement work did not use prerequisite"
        );
    }
    assert!(
        matches!(actor.terminal(12, Work::Discover(ProjectId::new()))?, Reply::ProductFailure(message) if !message.contains("network settings"))
    );
    for (id, work) in [
        (13, Work::OpenLogin(fresh.id)),
        (14, Work::ProbeEnvironment(fresh.id)),
    ] {
        assert!(
            matches!(actor.terminal(id, work)?, Reply::ProductFailure(message)
            if message.contains("network settings")),
            "no-baseline auth did not request prerequisite"
        );
    }
    // Five terminal status replies fill the four-slot output queue and block
    // publication of the fifth. No replies are drained until cancel is set, so
    // the operation is deterministically queued, not a timing race.
    for id in 20..=25 {
        let until = Instant::now() + Duration::from_secs(3);
        loop {
            let work = if id == 25 {
                Work::Discover(project)
            } else {
                Work::RuntimeStatus
            };
            if actor.jobs.submit(Request { id, work }).is_ok() {
                break;
            }
            ensure!(Instant::now() < until, "bounded actor enqueue failed");
            std::thread::sleep(Duration::from_millis(2));
        }
    }
    actor.jobs.cancel();
    for id in 20..=25 {
        let update = actor.replies.recv_timeout(Duration::from_secs(3))?;
        assert_eq!(update.id, id);
        if id == 25 {
            assert!(
                matches!(update.reply, Reply::ProductFailure(message) if message.contains("cancelled"))
            );
        } else {
            assert!(!matches!(update.reply, Reply::Progress { .. }));
        }
    }
    assert!(!paths.cache.join("product-runtime").exists());
    drop(actor);
    let store = Store::open(paths)?;
    assert_eq!(evidence_snapshot(&store, project, run)?, before);
    zero_connections(&listener)
}

#[test]
#[ignore = "private inert verifier; requires REGRESSEE_INDEPENDENT_ROOT; no worker/browser"]
fn public_installer_cancellation_preserves_old_evidence_and_pointer() -> Result<()> {
    for phase in [
        InstallProgress::Verifying,
        InstallProgress::Extracting,
        InstallProgress::TestingBrowser,
        InstallProgress::Ready,
    ] {
        let root = private_root()?;
        let (store, project, run) = baseline(root.path())?;
        let evidence = evidence_snapshot(&store, project, run)?;
        let (listener, trusted, key) = sentinel()?;
        let pack = inert_pack(root.path(), &trusted, &key)?;
        let cache = root.path().join("runtime");
        let cancel = AtomicBool::new(false);
        let old = distribution::install(&pack, &cache, &trusted, &cancel, &mut |_| Ok(()))?;
        let old_pointer = fs::read(cache.join("current"))?;
        let mut smoked = false;
        let result = distribution::install_with_progress(
            &pack,
            &cache,
            &trusted,
            &cancel,
            &mut |event| {
                if event == phase {
                    cancel.store(true, Ordering::Release);
                }
            },
            &mut |_| {
                smoked = true;
                Ok(())
            },
        );
        assert!(result.is_err());
        assert_eq!(smoked, phase == InstallProgress::Ready);
        assert_eq!(
            fs::read(cache.join("current"))? == old_pointer,
            phase != InstallProgress::Ready
        );
        assert!(old.root.exists());
        assert!(distribution::verify(&old.root, &trusted, &AtomicBool::new(false)).is_ok());
        assert_eq!(evidence_snapshot(&store, project, run)?, evidence);
        zero_connections(&listener)?;
    }
    Ok(())
}

#[test]
#[ignore = "private inert verifier; requires REGRESSEE_INDEPENDENT_ROOT; no worker/browser"]
fn public_download_progress_uses_actual_fixture_bytes_and_cancels_before_install() -> Result<()> {
    let root = private_root()?;
    let (store, project, run) = baseline(root.path())?;
    let before = evidence_snapshot(&store, project, run)?;
    let (listener, trusted, _) = sentinel()?;
    let server = std::thread::spawn(move || -> Result<()> {
        let until = Instant::now() + Duration::from_secs(2);
        let mut stream = loop {
            match listener.accept() {
                Ok((stream, _)) => break stream,
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    ensure!(
                        Instant::now() < until,
                        "fixture connection was not received"
                    );
                    std::thread::sleep(Duration::from_millis(2));
                }
                Err(error) => return Err(error.into()),
            }
        };
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;
        let mut request = [0; 4096];
        let count = stream.read(&mut request)?;
        ensure!(
            String::from_utf8_lossy(&request[..count]).starts_with("GET /must-not-download "),
            "unexpected fixture request"
        );
        // INERT bytes, deliberately not an archive. Cancel on received bytes,
        // before signature/extraction/smoke; no runtime can be executed.
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1536\r\nConnection: close\r\n\r\n")?;
        stream.write_all(&[73; 1536])?;
        Ok(())
    });
    let cancel = AtomicBool::new(false);
    let cache = store.paths().cache.join("product-runtime");
    let mut phases = vec![];
    let result = distribution::setup_network_with_progress(
        &cache,
        &trusted,
        &cancel,
        &mut |phase| {
            phases.push(phase);
            if matches!(phase, InstallProgress::Downloading { bytes, .. } if bytes > 0) {
                cancel.store(true, Ordering::Release);
            }
        },
        &mut |_| panic!("inert byte fixture reached worker smoke"),
        &Default::default(),
    );
    server
        .join()
        .map_err(|_| anyhow::anyhow!("fixture server panicked"))??;
    assert!(format!("{:#}", result.unwrap_err()).contains("cancelled"));
    assert_eq!(
        phases.first(),
        Some(&InstallProgress::Downloading {
            bytes: 0,
            total: None
        })
    );
    assert!(
        phases.iter().any(|phase| matches!(phase,
        InstallProgress::Downloading { bytes, total: Some(1536) } if *bytes > 0 && *bytes <= 1536))
    );
    assert!(
        phases
            .iter()
            .all(|phase| matches!(phase, InstallProgress::Downloading { .. }))
    );
    assert!(!cache.join("current").exists());
    assert_eq!(
        fs::read_dir(&cache)?.count(),
        0,
        "partial download was retained"
    );
    assert_eq!(evidence_snapshot(&store, project, run)?, before);
    Ok(())
}
