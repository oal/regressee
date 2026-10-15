// SPDX-License-Identifier: MPL-2.0
//! The native controls and local lifecycle tests share this serial actor.
use crate::{
    images::{self, Tiles},
    runtime,
    supervisor::Worker,
};
use anyhow::Result;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender},
    },
    time::Duration,
};

pub enum Work {
    Repair,
    Command(&'static str, Value),
    Image(PathBuf),
}
pub struct Update {
    pub status: String,
    pub tiles: Option<Tiles>,
    pub busy: bool,
}
fn report(events: &SyncSender<Update>, update: Update) {
    if std::env::var("REGRESSEE_UI_TEST").as_deref() == Ok("1") {
        let _ = std::fs::write("artifacts/ui-state.json", serde_json::to_vec(&json!({"busy":update.busy,"status":update.status,"height":update.tiles.as_ref().map(|t|t.height),"tile_count":update.tiles.as_ref().map(|t|t.files.len())})).unwrap());
    }
    let _ = events.try_send(update);
}
pub struct Controller {
    pub tx: SyncSender<Work>,
    pub cancel: Arc<AtomicBool>,
    stopped: Arc<AtomicBool>,
    thread: Mutex<Option<std::thread::JoinHandle<()>>>,
}
impl Controller {
    pub fn start(root: PathBuf, pack: PathBuf, tiles: PathBuf) -> (Arc<Self>, Receiver<Update>) {
        let (tx, commands) = mpsc::sync_channel(4);
        let (events, rx) = mpsc::sync_channel(16);
        let cancel = Arc::new(AtomicBool::new(false));
        let stopped = Arc::new(AtomicBool::new(false));
        let (c, s) = (cancel.clone(), stopped.clone());
        let thread = std::thread::spawn(move || {
            let mut worker: Option<Worker> = None;
            while !s.load(Ordering::Relaxed) {
                // Cancel is observable even while no request is in flight.
                if c.load(Ordering::Relaxed) {
                    worker = None; // supervised Drop closes login and detached trees
                    while commands.try_recv().is_ok() {} // do not replay cancelled queued work
                    report(
                        &events,
                        Update {
                            status:
                                "Cancelled. Managed browser closed; completed evidence preserved."
                                    .into(),
                            tiles: None,
                            busy: false,
                        },
                    );
                    // Only activation resets cancellation, never an actor receive.
                    while c.load(Ordering::Relaxed) && !s.load(Ordering::Relaxed) {
                        std::thread::sleep(Duration::from_millis(20));
                    }
                    continue;
                }
                let work = match commands.recv_timeout(Duration::from_millis(20)) {
                    Ok(w) => w,
                    Err(mpsc::RecvTimeoutError::Timeout) => continue,
                    Err(_) => break,
                };
                let result = (|| -> Result<(String, Option<Tiles>)> {
                    crate::cancel::check(&c, "operation")?;
                    match work {
                        Work::Repair => {
                            worker = None;
                            runtime::install_cancellable(&pack, &root, c.clone())?;
                            Ok((
                                "Private runtime ready. Sessions remain in memory only.".into(),
                                None,
                            ))
                        }
                        Work::Image(path) => Ok((
                            "Image opened as bounded 512-row tiles.".into(),
                            Some(images::tile_cancellable(&path, &tiles, &c)?),
                        )),
                        Work::Command(command, payload) => {
                            if worker.is_none() {
                                let mut w = Worker::spawn_cancellable(&root, c.clone())?;
                                w.request("handshake", json!({}))?;
                                worker = Some(w);
                            }
                            let value = worker.as_mut().unwrap().request(command, payload)?;
                            if command == "capture" {
                                let t = images::tile_cancellable(
                                    Path::new(value["output"].as_str().ok_or_else(|| {
                                        anyhow::anyhow!("invalid capture output")
                                    })?),
                                    &tiles,
                                    &c,
                                )?;
                                let warnings = value["warnings"]
                                    .as_array()
                                    .map(|v| {
                                        v.iter()
                                            .filter_map(Value::as_str)
                                            .collect::<Vec<_>>()
                                            .join(", ")
                                    })
                                    .unwrap_or_default();
                                Ok((
                                    format!(
                                        "Captured {} × {}. Session: ephemeral. {}",
                                        t.width,
                                        t.height,
                                        if warnings.is_empty() {
                                            "Stable fixture.".into()
                                        } else {
                                            format!("Readiness warnings: {warnings}")
                                        }
                                    ),
                                    Some(t),
                                ))
                            } else {
                                Ok((match command { "openLogin" => "Sign in in the managed browser, then choose I’m ready.", "finishLogin" => "Signed in. This session is kept only until the app closes.", _ => "Operation complete." }.into(), None))
                            }
                        }
                    }
                })();
                let update = match result {
                    Ok((status, tiles)) => Update {
                        status,
                        tiles,
                        busy: false,
                    },
                    Err(e) => {
                        worker = None;
                        Update {
                            status: format!("Could not finish: {e}. Completed evidence preserved."),
                            tiles: None,
                            busy: false,
                        }
                    }
                };
                report(&events, update);
            }
            drop(worker);
        });
        (
            Arc::new(Self {
                tx,
                cancel,
                stopped,
                thread: Mutex::new(Some(thread)),
            }),
            rx,
        )
    }
    pub fn activate(&self, work: Work) -> Result<()> {
        anyhow::ensure!(!self.stopped.load(Ordering::Relaxed), "actor stopped");
        self.cancel.store(false, Ordering::Relaxed);
        self.tx.try_send(work)?;
        Ok(())
    }
    pub fn stop(&self) {
        self.stopped.store(true, Ordering::Relaxed);
        self.cancel.store(true, Ordering::Relaxed);
        if let Some(thread) = self.thread.lock().unwrap().take() {
            let _ = thread.join();
        }
    }
}
impl Drop for Controller {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Fixture-only integration test of the exact actor used by Escape/Cancel/Quit.
/// Native event delivery is a separate qualification, not implied here.
#[cfg(target_os = "linux")]
pub fn lifecycle_tests(root: &Path) -> Result<()> {
    use sha2::{Digest, Sha256};
    use std::time::Instant;
    let ready = std::fs::read(root.join("READY"))?;
    let evidence = std::fs::read("artifacts/public-before.png")?;
    let hash = hex::encode(Sha256::digest(&evidence));
    let tiles = tempfile::Builder::new()
        .prefix("regressee-actor-tiles-")
        .tempdir()?;
    let make = || Controller::start(root.into(), ".runtime/pack.zip".into(), tiles.path().into());
    let owned_scratch = || -> Vec<PathBuf> {
        std::fs::read_dir(crate::supervisor::scratch_parent())
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| {
                let owner: Value =
                    serde_json::from_slice(&std::fs::read(e.path().join("owner.json")).ok()?)
                        .ok()?;
                (owner["parent"] == std::process::id()).then(|| e.path())
            })
            .collect()
    };
    let assert_clean = || -> Result<()> {
        anyhow::ensure!(owned_scratch().is_empty(), "actor owned scratch remains");
        Ok(())
    };
    let (actor, rx) = make();
    let activated = Instant::now();
    actor.activate(Work::Command("capture", json!({"target":"/public"})))?;
    std::thread::sleep(Duration::from_millis(200));
    actor.cancel.store(true, Ordering::Relaxed);
    let cancelled = Instant::now();
    let update = rx.recv_timeout(Duration::from_secs(5))?;
    anyhow::ensure!(
        update.status.contains("cancelled"),
        "wrong activation cancellation reason: {}",
        update.status
    );
    actor.stop();
    assert_clean()?;
    anyhow::ensure!(
        activated.elapsed() < Duration::from_secs(5),
        "activation-to-cleanup exceeded 5 seconds"
    );
    let mut results = vec![
        json!({"test":"activation through installed verification cancellation","activation_to_cleanup_ms":activated.elapsed().as_millis(),"cancel_to_cleanup_ms":cancelled.elapsed().as_millis(),"reason":update.status}),
    ];
    let (actor, rx) = make();
    actor.activate(Work::Command("openLogin", json!({})))?;
    let update = rx.recv_timeout(Duration::from_secs(60))?;
    anyhow::ensure!(
        update.status.starts_with("Sign in"),
        "login did not open: {}",
        update.status
    );
    anyhow::ensure!(!owned_scratch().is_empty(), "idle login test has no worker");
    let cancel = Instant::now();
    actor.cancel.store(true, Ordering::Relaxed);
    let update = rx.recv_timeout(Duration::from_secs(5))?;
    anyhow::ensure!(
        update
            .status
            .starts_with("Cancelled. Managed browser closed"),
        "idle cancellation cause: {}",
        update.status
    );
    actor.stop();
    assert_clean()?;
    results.push(json!({"test":"idle open-login Cancel","cancel_to_cleanup_ms":cancel.elapsed().as_millis(),"reason":update.status}));
    let (actor, rx) = make();
    actor.activate(Work::Command("openLogin", json!({})))?;
    let deadline = Instant::now() + Duration::from_secs(60);
    let browsers = root.join("browsers").canonicalize()?;
    let mut saw_launch = false;
    while Instant::now() < deadline {
        if let Ok(update) = rx.try_recv() {
            anyhow::bail!(
                "quit-during-launch missed launch boundary: {}",
                update.status
            );
        }
        let scratch = owned_scratch();
        for entry in std::fs::read_dir("/proc")?.flatten() {
            if std::fs::read_link(entry.path().join("exe")).is_ok_and(|p| p.starts_with(&browsers))
                && std::fs::read(entry.path().join("cmdline")).is_ok_and(|args| {
                    scratch.iter().any(|s| {
                        args.windows(s.as_os_str().as_encoded_bytes().len())
                            .any(|w| w == s.as_os_str().as_encoded_bytes())
                    })
                })
            {
                saw_launch = true;
                break;
            }
        }
        if saw_launch {
            break;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    anyhow::ensure!(saw_launch, "no owned browser launch observed");
    let quit = Instant::now();
    actor.stop();
    assert_clean()?;
    anyhow::ensure!(
        quit.elapsed() < Duration::from_secs(5),
        "quit-during-launch cleanup exceeded 5 seconds"
    );
    results.push(json!({"test":"quit while owned Chromium launch observed before login result","quit_to_cleanup_ms":quit.elapsed().as_millis()}));
    anyhow::ensure!(
        std::fs::read(root.join("READY"))? == ready,
        "ready runtime changed"
    );
    anyhow::ensure!(
        hex::encode(Sha256::digest(std::fs::read(
            "artifacts/public-before.png"
        )?)) == hash,
        "original evidence changed"
    );
    std::fs::write(
        "artifacts/corrections-actor.json",
        serde_json::to_vec_pretty(
            &json!({"results":results,"original_sha256":hash,"native_input_delivery":"NOT RUN by actor test"}),
        )?,
    )?;
    println!("{}", serde_json::to_string_pretty(&results)?);
    Ok(())
}
