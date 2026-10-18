// SPDX-License-Identifier: MPL-2.0
//! Serial real signed/sandboxed-runtime fault regressions. No test rescue: a
//! surviving process is an assertion failure and wrapper intervention is FAIL.
#![cfg(target_os = "linux")]
use anyhow::{Context, Result, ensure};
use regressee_runtime::{
    distribution,
    protocol::{CapturedChannel, Command, EventKind, PROTOCOL_VERSION, Request},
    supervisor::{ProcessSupervisor, Worker},
};
use std::{
    fs,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    os::unix::fs::MetadataExt,
    path::{Path, PathBuf},
    sync::{Arc, atomic::AtomicBool},
    time::{Duration, Instant},
};
struct Site {
    address: String,
    stop: Arc<AtomicBool>,
    owner: Option<std::thread::JoinHandle<()>>,
}
impl Site {
    fn start() -> Result<Self> {
        Self::start_with_body(b"<!doctype html><body style='margin:0;background:white'><p>Unrelated owned browser recapture</p></body>", false)
    }
    fn dom_fault() -> Result<Self> {
        // Lazy loading does not delay common document readiness. Screenshot-only
        // eager loading/decode then waits on an owned deliberately pending image,
        // leaving a deterministic window after DOM publication for the fault.
        Self::start_with_body(b"<!doctype html><body><p>Retained independent DOM evidence</p><img loading='lazy' src='/pending-image' width='1' height='1' style='position:absolute;top:20000px'></body>", true)
    }
    fn start_with_body(body: &'static [u8], pending_images: bool) -> Result<Self> {
        use std::io::{Read, Write};
        let listener = std::net::TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let address = format!("http://{}/owned", listener.local_addr()?);
        let stop = Arc::new(AtomicBool::new(false));
        let closing = stop.clone();
        let owner = std::thread::spawn(move || {
            let mut pending = Vec::new();
            while !closing.load(std::sync::atomic::Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_read_timeout(Some(Duration::from_millis(100)));
                        let _ = stream.set_write_timeout(Some(Duration::from_millis(100)));
                        let mut bytes = [0; 4096];
                        if let Ok(n) = stream.read(&mut bytes)
                            && n > 0
                        {
                            if pending_images
                                && bytes[..n].starts_with(b"GET /pending-image ")
                                && pending.len() < 16
                            {
                                pending.push(stream);
                                continue;
                            }
                            let _ = write!(
                                stream,
                                "HTTP/1.1 200 OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                body.len()
                            );
                            let _ = stream.write_all(body);
                        }
                    }
                    Err(_) => std::thread::sleep(Duration::from_millis(5)),
                }
            }
        });
        Ok(Self {
            address,
            stop,
            owner: Some(owner),
        })
    }
}
impl Drop for Site {
    fn drop(&mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        if let Some(owner) = self.owner.take() {
            let _ = owner.join();
        }
    }
}
fn take(
    worker: &mut Worker,
    runtime: &distribution::Installed,
    site: &Site,
    id: u64,
) -> Result<Vec<u8>> {
    let mut profile = regressee_core::domain::CaptureProfile::desktop();
    profile.runtime = runtime.manifest.identity.clone();
    profile.width = 600;
    profile.height = 300;
    profile.extent = regressee_core::domain::CaptureExtent::VisibleScreen;
    let mut token = None;
    worker.request(
        Request {
            version: PROTOCOL_VERSION,
            request_id: id,
            job: regressee_core::domain::RunId::new(),
            command: Command::CaptureTarget {
                key: regressee_core::domain::TargetKey {
                    page: regressee_core::domain::PageId::new(),
                    profile: profile.id,
                },
                address: site.address.clone(),
                profile: Box::new(profile),
                session: None,
                plan: Default::default(),
            },
        },
        Instant::now() + Duration::from_secs(20),
        Arc::new(AtomicBool::new(false)),
        &mut |event| {
            if let EventKind::ChannelCaptured {
                capture: CapturedChannel::Screenshot { output_token, .. },
                ..
            } = event.event
            {
                token = Some(output_token);
            }
            Ok(())
        },
    )?;
    let token = token.context("unrelated recapture missing")?;
    Ok(fs::read(worker.output_path(token))?)
}

fn installed() -> Result<distribution::Installed> {
    let config = std::env::var("REGRESSEE_OWNERSHIP_DISTRIBUTION")?;
    let cache = std::env::var("REGRESSEE_OWNERSHIP_CACHE")?;
    let distribution = serde_json::from_slice(&fs::read(config)?)?;
    distribution::resolve(Path::new(&cache), &distribution, &AtomicBool::new(false))
}
fn born(pid: u32) -> Result<(u32, u64)> {
    let stat = fs::read_to_string(format!("/proc/{pid}/stat"))?;
    let fields: Vec<_> = stat[stat.rfind(')').context("stat delimiter")? + 2..]
        .split_whitespace()
        .collect();
    Ok((fields[1].parse()?, fields[19].parse()?))
}
fn handle(pid: u32) -> Result<OwnedFd> {
    let before = born(pid)?;
    let fd = unsafe { libc::syscall(libc::SYS_pidfd_open, pid, 0) } as i32;
    ensure!(fd >= 0, "pidfd acquisition");
    let fd = unsafe { OwnedFd::from_raw_fd(fd) };
    ensure!(before == born(pid)?, "fault target lifetime changed");
    Ok(fd)
}
fn signal(fd: &OwnedFd, signal: i32) -> Result<()> {
    ensure!(
        unsafe {
            libc::syscall(
                libc::SYS_pidfd_send_signal,
                fd.as_raw_fd(),
                signal,
                std::ptr::null::<libc::siginfo_t>(),
                0,
            )
        } == 0,
        "exact fault pidfd signal"
    );
    Ok(())
}
fn dead(fd: &OwnedFd) -> bool {
    let mut poll = libc::pollfd {
        fd: fd.as_raw_fd(),
        events: libc::POLLIN,
        revents: 0,
    };
    unsafe { libc::poll(&mut poll, 1, 0) > 0 }
}
fn settled(handles: &[OwnedFd]) {
    let until = Instant::now() + Duration::from_secs(7);
    while handles.iter().any(|fd| !dead(fd)) && Instant::now() < until {
        std::thread::sleep(Duration::from_millis(10));
    }
    assert!(
        handles.iter().all(dead),
        "APP cleanup failed; no test rescue is credited"
    );
}
fn browser_handles(
    worker: &Worker,
    runtime: &distribution::Installed,
) -> Result<Vec<(u32, OwnedFd)>> {
    let scratch = worker
        .output_path(uuid::Uuid::new_v4())
        .parent()
        .unwrap()
        .to_owned();
    let browser = runtime
        .root
        .canonicalize()?
        .join(&runtime.manifest.browser_root);
    let profile = format!("--user-data-dir={}/", scratch.display());
    let mut handles = vec![];
    for entry in fs::read_dir("/proc")?.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        if !fs::metadata(entry.path()).is_ok_and(|m| m.uid() == unsafe { libc::geteuid() }) {
            continue;
        }
        let Ok(exe) = fs::read_link(entry.path().join("exe")) else {
            continue;
        };
        if !exe.starts_with(&browser) {
            continue;
        }
        let Ok(args) = fs::read(entry.path().join("cmdline")) else {
            continue;
        };
        // Chromium rewrites argv storage too, sometimes flattening it into a
        // single space-separated process title. This is test observation, NEVER
        // production authority; the guardian's kernel receipt covers the tree.
        if !args
            .split(|b| *b == 0 || *b == b' ')
            .any(|arg| arg.starts_with(profile.as_bytes()))
        {
            continue;
        }
        handles.push((pid, handle(pid)?));
    }
    ensure!(!handles.is_empty(), "real browser tree not observed");
    Ok(handles)
}
fn browser_main(handles: &[(u32, OwnedFd)]) -> Result<&OwnedFd> {
    for (pid, fd) in handles {
        let args = fs::read(format!("/proc/{pid}/cmdline"))?;
        if !args
            .split(|b| *b == 0 || *b == b' ')
            .any(|arg| arg.starts_with(b"--type="))
        {
            return Ok(fd);
        }
    }
    anyhow::bail!("browser main absent")
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct DetachedCheckpoint {
    run: regressee_core::domain::RunId,
    key: regressee_core::domain::TargetKey,
    artifact: regressee_core::domain::Artifact,
    artifact_path: PathBuf,
    metadata_path: PathBuf,
}
fn private_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    use std::{io::Write, os::unix::fs::OpenOptionsExt};
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(&serde_json::to_vec(value)?)?;
    file.sync_all()?;
    Ok(())
}

#[test]
#[ignore = "private fresh-process verifier invoked only by signed_dom_publication_survives_supervisor_faults"]
fn owned_detached_dom_child() -> Result<()> {
    use regressee_core::{domain::ArtifactKind, evidence::DomCaptureMetadata};
    let record = PathBuf::from(std::env::var("REGRESSEE_OWNERSHIP_DOM_CHECKPOINT")?);
    let parent = record.parent().context("checkpoint parent")?;
    distribution::safe_directory(parent)?;
    ensure!(
        fs::metadata(&record)?.len() <= 16 * 1024,
        "checkpoint exceeds test budget"
    );
    let checkpoint: DetachedCheckpoint = serde_json::from_slice(&fs::read(&record)?)?;
    ensure!(
        checkpoint.artifact_path.parent() == Some(parent)
            && checkpoint.metadata_path.parent() == Some(parent),
        "test checkpoint path binding"
    );
    let filename = checkpoint
        .artifact_path
        .file_name()
        .and_then(|s| s.to_str())
        .context("DOM filename")?;
    let token = uuid::Uuid::parse_str(
        filename
            .strip_suffix(".dom.jsonl")
            .context("DOM kind suffix")?,
    )?;
    ensure!(
        !token.is_nil() && checkpoint.metadata_path == parent.join(format!("{token}.dom.json")),
        "typed detached filenames"
    );
    ensure!(
        checkpoint.artifact.kind == ArtifactKind::DomSnapshot
            && checkpoint.artifact.bytes <= regressee_core::dom_snapshot::MAX_DOM_BYTES as u64,
        "test artifact kind/budget"
    );
    ensure!(
        fs::metadata(&checkpoint.metadata_path)?.len() <= 1024 * 1024
            && fs::metadata(&checkpoint.artifact_path)?.len() == checkpoint.artifact.bytes,
        "test detached size binding"
    );
    let metadata: DomCaptureMetadata =
        serde_json::from_slice(&fs::read(&checkpoint.metadata_path)?)?;
    ensure!(
        metadata.run == checkpoint.run
            && metadata.key == checkpoint.key
            && metadata.sha256 == checkpoint.artifact.id
            && metadata.bytes == checkpoint.artifact.bytes,
        "fresh-process source binding"
    );
    metadata.verify_snapshot(&fs::read(&checkpoint.artifact_path)?)?;
    ensure!(metadata.stable, "fixture DOM did not settle");
    Ok(())
}

#[test]
#[ignore = "requires explicit current signed loopback runtime, private evidence root and serial bounded wrapper"]
fn signed_dom_publication_survives_supervisor_faults() -> Result<()> {
    use regressee_core::domain::{
        CaptureExtent, CaptureMode, CapturePlan, CaptureProfile, CheckKind, PageId, RunId,
        RuntimeIdentity, TargetKey,
    };
    use regressee_runtime::{
        protocol::{FailureCode, WORKER_VERSION},
        supervisor::{DetachedMetadata, WorkerFailure},
    };
    use std::os::unix::fs::PermissionsExt;
    let runtime = installed()?;
    ensure!(
        runtime.manifest.scope == distribution::Scope::DevelopmentLoopback,
        "DOM faults require explicit development-loopback runtime scope"
    );
    ensure!(
        matches!(&runtime.manifest.identity, RuntimeIdentity::Pinned { worker, .. } if worker == WORKER_VERSION),
        "DOM faults require the current verified worker generation"
    );
    let root = PathBuf::from(std::env::var("REGRESSEE_OWNERSHIP_DOM_EVIDENCE")?);
    distribution::safe_directory(&root)?;
    ensure!(
        root.is_absolute(),
        "private DOM fault evidence requires an absolute path"
    );
    fs::create_dir(&root)?;
    fs::set_permissions(&root, fs::Permissions::from_mode(0o700))?;
    let site = Site::dom_fault()?;
    let mut evidence = vec![];
    for fault in ["timeout", "cancel", "worker-crash"] {
        let staging = root.join(fault);
        fs::create_dir(&staging)?;
        fs::set_permissions(&staging, fs::Permissions::from_mode(0o700))?;
        let mut worker = Worker::spawn(&runtime)?;
        let job = RunId::new();
        worker.handshake(job, Arc::new(AtomicBool::new(false)))?;
        let scratch = worker
            .output_path(uuid::Uuid::new_v4())
            .parent()
            .unwrap()
            .to_owned();
        let tree = browser_handles(&worker, &runtime)?;
        let node = handle(worker.pid())?;
        let mut profile = CaptureProfile::desktop();
        profile.runtime = runtime.manifest.identity.clone();
        profile.user_agent = "Regressee owned supervisor DOM fault fixture".into();
        profile.width = 600;
        profile.height = 300;
        profile.extent = CaptureExtent::VisibleScreen;
        profile.stabilization.navigation_ms = 3000;
        profile.stabilization.readiness_ms = 10_000;
        let key = TargetKey {
            page: PageId::new(),
            profile: profile.id,
        };
        let cancel = Arc::new(AtomicBool::new(false));
        let mut detached = None;
        let mut screenshot_published = false;
        let began = Instant::now();
        let result = worker.request_detached(
            Request {
                version: PROTOCOL_VERSION,
                request_id: 50,
                job,
                command: Command::CaptureTarget {
                    key: key.clone(),
                    address: site.address.clone(),
                    profile: Box::new(profile),
                    session: None,
                    plan: CapturePlan::for_mode(CaptureMode::ScreenshotsAndDomSnapshots),
                },
            },
            Instant::now() + Duration::from_secs(2),
            cancel.clone(),
            &staging,
            &mut |event, capture| {
                if let EventKind::ChannelCaptured {
                    capture: CapturedChannel::Screenshot { .. },
                    ..
                } = &event.event
                {
                    screenshot_published = true;
                }
                if let Some(capture) = capture
                    && capture.capture.kind() == CheckKind::DomSnapshot
                {
                    ensure!(detached.is_none(), "duplicate detached DOM");
                    let DetachedMetadata::DomSnapshot(metadata) = &capture.metadata else {
                        anyhow::bail!("wrong detached metadata kind");
                    };
                    ensure!(
                        metadata.run == job && metadata.key == key,
                        "publication must bind the actual request run/target"
                    );
                    private_json(
                        &staging.join("checkpoint.json"),
                        &DetachedCheckpoint {
                            run: job,
                            key: key.clone(),
                            artifact: capture.capture.artifact().clone(),
                            artifact_path: capture.artifact_path.clone(),
                            metadata_path: capture.metadata_path.clone(),
                        },
                    )?;
                    // Exact pidfd authority acquired from this unreaped owned Node.
                    // No numeric PID/process-group reconstruction or test rescue.
                    if fault == "worker-crash" {
                        signal(&node, libc::SIGKILL)?;
                    } else {
                        signal(&node, libc::SIGSTOP)?;
                    }
                    if fault == "cancel" {
                        cancel.store(true, std::sync::atomic::Ordering::Release);
                    }
                    detached = Some(capture);
                }
                Ok(())
            },
        );
        let error = result.expect_err("sibling fault must not report terminal success");
        let expected = match fault {
            "timeout" => FailureCode::Timeout,
            "cancel" => FailureCode::Cancelled,
            _ => FailureCode::WorkerStopped,
        };
        assert_eq!(
            error.downcast_ref::<WorkerFailure>().map(|e| e.0),
            Some(expected),
            "{fault}: {error:#}"
        );
        ensure!(
            !screenshot_published,
            "fault fixture did not leave the sibling pending"
        );
        let capture =
            detached.context("successful DOM was not published before the sibling fault")?;
        drop(worker);
        settled(
            &tree
                .into_iter()
                .map(|(_, fd)| fd)
                .chain(std::iter::once(node))
                .collect::<Vec<_>>(),
        );
        ensure!(!scratch.exists(), "worker scratch survived proved cleanup");
        ensure!(
            capture.artifact_path.is_file() && capture.metadata_path.is_file(),
            "worker cleanup deleted detached evidence"
        );
        let mut child = std::process::Command::new(std::env::current_exe()?)
            .args([
                "--ignored",
                "--exact",
                "owned_detached_dom_child",
                "--nocapture",
            ])
            .env(
                "REGRESSEE_OWNERSHIP_DOM_CHECKPOINT",
                staging.join("checkpoint.json"),
            )
            .spawn()?;
        let child_deadline = Instant::now() + Duration::from_secs(10);
        let status = loop {
            if let Some(status) = child.try_wait()? {
                break status;
            }
            if Instant::now() >= child_deadline {
                child.kill()?;
                child.wait()?;
                anyhow::bail!("fresh-process detached verification timed out");
            }
            std::thread::sleep(Duration::from_millis(10));
        };
        ensure!(
            status.success(),
            "fresh-process detached verification failed"
        );
        evidence.push(serde_json::json!({"fault":fault,"run":job,"key":key,"failure":format!("{expected:?}"),"dom_artifact":capture.capture.artifact(),"screenshot_published":false,"worker_scratch_removed":true,"observed_processes_settled":true,"fresh_process_verified":true,"elapsed_ms":began.elapsed().as_millis(),"scope":"runtime detached staging; not app CAS/database resume evidence"}));
    }
    private_json(&root.join("runtime-dom-faults.json"), &evidence)?;
    Ok(())
}

#[test]
#[ignore = "requires explicit current signed loopback runtime, serial bounded wrapper"]
fn signed_stopped_main_reader_worker_crash_cancel_and_unrelated_browser() -> Result<()> {
    let runtime = installed()?;
    let mut other = Worker::spawn(&runtime)?;
    other.handshake(
        regressee_core::domain::RunId::new(),
        Arc::new(AtomicBool::new(false)),
    )?;
    let other_handles = browser_handles(&other, &runtime)?;
    let site = Site::start()?;
    let original = take(&mut other, &runtime, &site, 10)?;
    for fault in ["main", "reader", "worker-kill", "active-cancel", "drop"] {
        let mut worker = Worker::spawn(&runtime)?;
        worker.handshake(
            regressee_core::domain::RunId::new(),
            Arc::new(AtomicBool::new(false)),
        )?;
        let scratch = worker
            .output_path(uuid::Uuid::new_v4())
            .parent()
            .unwrap()
            .to_owned();
        let tree = browser_handles(&worker, &runtime)?;
        let node = handle(worker.pid())?;
        let mut unrelated = std::process::Command::new("/bin/sleep")
            .arg("30")
            .env("TMPDIR", &scratch)
            .spawn()?;
        match fault {
            "main" => signal(browser_main(&tree)?, libc::SIGSTOP)?,
            "reader" | "active-cancel" => signal(&node, libc::SIGSTOP)?,
            "worker-kill" => signal(&node, libc::SIGKILL)?,
            _ => (),
        }
        if fault == "active-cancel" {
            let cancel = Arc::new(AtomicBool::new(false));
            let trigger = cancel.clone();
            let thread = std::thread::spawn(move || {
                std::thread::sleep(Duration::from_millis(50));
                trigger.store(true, std::sync::atomic::Ordering::Release);
            });
            assert!(
                worker
                    .request(
                        Request {
                            version: PROTOCOL_VERSION,
                            request_id: 77,
                            job: regressee_core::domain::RunId::new(),
                            command: Command::Handshake
                        },
                        Instant::now() + Duration::from_secs(10),
                        cancel,
                        &mut |_| Ok(())
                    )
                    .is_err()
            );
            thread.join().unwrap();
        } else if fault != "drop" {
            let _ = worker.stop(Instant::now() + Duration::from_millis(200));
        }
        drop(worker);
        settled(
            &tree
                .into_iter()
                .map(|(_, fd)| fd)
                .chain(std::iter::once(node))
                .collect::<Vec<_>>(),
        );
        assert!(!scratch.exists(), "settled tree scratch must be removed");
        assert!(
            unrelated.try_wait()?.is_none(),
            "unrelated same-TMPDIR process touched"
        );
        assert!(
            other_handles.iter().all(|(_, fd)| !dead(fd)),
            "other project browser touched"
        );
        assert_eq!(
            take(&mut other, &runtime, &site, 11)?,
            original,
            "unrelated browser must recapture identical bytes after every fault cleanup"
        );
        unrelated.kill()?;
        unrelated.wait()?;
    }
    let _ = other.stop(Instant::now() + Duration::from_secs(2));
    settled(
        &other_handles
            .into_iter()
            .map(|(_, fd)| fd)
            .collect::<Vec<_>>(),
    );
    Ok(())
}

#[test]
#[ignore = "private helper invoked only by signed_app_sigkill_recovery_and_launch_race"]
fn owned_app_child() -> Result<()> {
    let output = PathBuf::from(std::env::var("REGRESSEE_OWNERSHIP_CHILD_RECORD")?);
    let runtime = installed()?;
    let mut worker = Worker::spawn(&runtime)?;
    let scratch = worker
        .output_path(uuid::Uuid::new_v4())
        .parent()
        .unwrap()
        .to_owned();
    let race = std::env::var_os("REGRESSEE_OWNERSHIP_LAUNCH_RACE").is_some();
    if race {
        worker.request(Request { version:PROTOCOL_VERSION,request_id:1,job:regressee_core::domain::RunId::new(),command:Command::Handshake },Instant::now()+Duration::from_secs(30),Arc::new(AtomicBool::new(false)),&mut |event| {
            if matches!(event.event,regressee_runtime::protocol::EventKind::Acknowledged) {
                let pending=output.with_extension("pending");
                fs::write(&pending,serde_json::to_vec(&serde_json::json!({"scratch":scratch,"tree":[],"launch":"signed worker acknowledged browser launch"}))?)?;
                fs::rename(pending,&output)?;
            }
            Ok(())
        })?;
    } else {
        worker.handshake(
            regressee_core::domain::RunId::new(),
            Arc::new(AtomicBool::new(false)),
        )?;
    }
    let tree: Vec<u32> = browser_handles(&worker, &runtime)?
        .into_iter()
        .map(|(pid, _)| pid)
        .collect();
    if !race {
        let pending = output.with_extension("pending");
        fs::write(
            &pending,
            serde_json::to_vec(&serde_json::json!({"scratch":scratch,"tree":tree}))?,
        )?;
        fs::rename(pending, output)?;
    }
    loop {
        std::thread::sleep(Duration::from_secs(1));
    }
}

#[test]
#[ignore = "requires explicit current signed loopback runtime, serial bounded wrapper"]
fn signed_app_sigkill_recovery_and_launch_race() -> Result<()> {
    let root = PathBuf::from(std::env::var("REGRESSEE_OWNERSHIP_EVIDENCE")?);
    fs::create_dir(&root)?;
    let runtime = installed()?;
    for race in [false, true] {
        let record = root.join(if race { "race.json" } else { "app-crash.json" });
        let mut command = std::process::Command::new(std::env::current_exe()?);
        command
            .args(["--ignored", "--exact", "owned_app_child", "--nocapture"])
            .env("REGRESSEE_OWNERSHIP_CHILD_RECORD", &record);
        if race {
            command.env("REGRESSEE_OWNERSHIP_LAUNCH_RACE", "1");
        }
        let mut app = command.spawn()?;
        let until = Instant::now() + Duration::from_secs(30);
        while !record.exists() && Instant::now() < until {
            std::thread::sleep(Duration::from_millis(5));
        }
        ensure!(record.exists(), "child app failed before launch checkpoint");
        let value: serde_json::Value = serde_json::from_slice(&fs::read(&record)?)?;
        let scratch = PathBuf::from(value["scratch"].as_str().unwrap());
        let handles = value["tree"]
            .as_array()
            .unwrap()
            .iter()
            .map(|pid| handle(pid.as_u64().unwrap() as u32))
            .collect::<Result<Vec<_>>>()?;
        // Kill only this exact unreaped app Child; guardian must do ALL cleanup.
        app.kill()?;
        app.wait()?;
        settled(&handles);
        let until = Instant::now() + Duration::from_secs(7);
        while fs::read(scratch.join("settled")).is_ok_and(|bytes| bytes.is_empty())
            && Instant::now() < until
        {
            std::thread::sleep(Duration::from_millis(10));
        }
        assert_eq!(
            fs::read(scratch.join("settled"))?.len(),
            48,
            "app crash lost settlement proof"
        );
        let mut recovery = Worker::spawn(&runtime)?;
        recovery.handshake(
            regressee_core::domain::RunId::new(),
            Arc::new(AtomicBool::new(false)),
        )?;
        let _ = recovery.stop(Instant::now() + Duration::from_secs(2));
        assert!(
            !scratch.exists(),
            "startup did not recover proven settled namespace"
        );
    }
    Ok(())
}
