// SPDX-License-Identifier: MPL-2.0
//! Actual sandboxed product worker, signed HTTP install, immutable storage, diff.
//! Explicitly ignored by default: needs the retained verified development pack.
//! Build with `--features test-support`; no GPUI dependencies are needed.
use anyhow::{Context, Result, ensure};
use ed25519_dalek::{Signature, Signer, SigningKey};
use regressee_app::product::Product;
use regressee_core::domain::*;
#[cfg(target_os = "linux")]
use regressee_runtime::protocol::EventKind;
use regressee_runtime::{
    distribution::{self, Distribution, Entry, Manifest, Scope},
    protocol::{Command, FailureCode, Request},
    supervisor::{ProcessSupervisor, Worker, WorkerFailure},
};
use regressee_storage::{AppPaths, Store};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, AtomicU32, Ordering},
    },
    time::{Duration, Instant},
};
use zip::{ZipWriter, write::SimpleFileOptions};
struct Site {
    address: String,
    state: Arc<AtomicU32>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
    hits: Arc<std::sync::Mutex<BTreeMap<String, u32>>>,
}
impl Site {
    fn start(pack: PathBuf) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let address = format!("http://{}", listener.local_addr()?);
        let state = Arc::new(AtomicU32::new(0));
        let value = state.clone();
        let stop = Arc::new(AtomicBool::new(false));
        let shutdown = stop.clone();
        let hits = Arc::new(std::sync::Mutex::new(BTreeMap::new()));
        let visited = hits.clone();
        let thread = std::thread::spawn(move || {
            let mut handlers: Vec<std::thread::JoinHandle<()>> = Vec::new();
            while !shutdown.load(Ordering::Acquire) {
                let mut i = 0;
                while i < handlers.len() {
                    if handlers[i].is_finished() {
                        let _ = handlers.swap_remove(i).join();
                    } else {
                        i += 1;
                    }
                }
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        // Bounded concurrent handlers: speculative idle sockets
                        // must not hold up Chromium's document/resource requests.
                        if handlers.len() >= 64 {
                            continue;
                        }
                        let (visited, value, shutdown, pack) = (
                            visited.clone(),
                            value.clone(),
                            shutdown.clone(),
                            pack.clone(),
                        );
                        handlers.push(std::thread::spawn(move || {
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                        let _ = stream.set_write_timeout(Some(Duration::from_secs(3)));
                        let mut bytes = vec![0; 8192];
                        let n = stream.read(&mut bytes).unwrap_or(0);
                        if n == 0 { return; }
                        let req = String::from_utf8_lossy(&bytes[..n]);
                        let path = req
                            .lines()
                            .next()
                            .and_then(|s| s.split_whitespace().nth(1))
                            .unwrap_or("");
                        *visited.lock().unwrap().entry(path.to_owned()).or_insert(0) += 1;
                        if path == "/fixture-login" {
                            // Self-establishing synthetic account, not a human
                            // login. Exercise the real headed Open/I'm ready flow.
                            let body = b"<!doctype html><title>Fixture login ready</title><p>Synthetic fixture account ready</p>";
                            let _ = write!(stream, "HTTP/1.1 200 OK\r\nSet-Cookie: fixture-session=ready; Path=/; HttpOnly; SameSite=Lax\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n", body.len());
                            let _ = stream.write_all(body);
                        } else if path == "/runtime.zip" {
                            if let Ok(mut file) = File::open(&pack) {
                                let size = file.metadata().unwrap().len();
                                let _ = write!(
                                    stream,
                                    "HTTP/1.1 200 OK\r\nContent-Length: {size}\r\nConnection: close\r\n\r\n"
                                );
                                let _ = std::io::copy(&mut file, &mut stream);
                            }
                        } else if path == "/corrupt.zip" {
                            let _ = respond(&mut stream, 200, b"corrupt download");
                        } else if path == "/redirect.zip" {
                            let _=stream.write_all(b"HTTP/1.1 302 Found\r\nLocation: http://example.com/untrusted.zip\r\nContent-Length: 0\r\n\r\n");
                        } else if path.starts_with("/auth/page")
                            || path.starts_with("/preview/page")
                        {
                            if value.load(Ordering::Acquire) == 3 {
                                let _=stream.write_all(b"HTTP/1.1 302 Found\r\nLocation: /login\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                            } else if !req
                                .to_ascii_lowercase()
                                .contains("cookie: fixture-session=ready")
                            {
                                let _ = respond(&mut stream, 401, b"denied");
                            } else {
                                if path.contains("portable=") {
                                    let color = if path.contains("portable=change")
                                        && value.load(Ordering::Acquire) == 1
                                    {
                                        "#cc3344"
                                    } else {
                                        "#08786f"
                                    };
                                    let body = format!(
                                        "<!doctype html><body style='margin:0;font:20px sans-serif'><h1>Portable authenticated fixture</h1><div style='width:80px;height:80px;background:{color}'></div></body>"
                                    );
                                    let _ = respond(&mut stream, 200, body.as_bytes());
                                } else {
                                    let _=respond(&mut stream,200,b"<!doctype html><html><body style='margin:0;font:20px sans-serif'><h1>Read-only authenticated fixture</h1></body></html>");
                                }
                            }
                        } else if path.starts_with("/soak/") {
                            let body=format!("<!doctype html><meta charset=utf-8><meta name=viewport content='width=device-width, initial-scale=1'><style>body{{margin:0;background:white;font:18px sans-serif}}.mark{{width:50px;height:50px;background:#08786f}}</style><h1>Owned soak fixture</h1><p>{path}</p><div class=mark></div>");
                            let _=respond(&mut stream,200,body.as_bytes());
                        } else if path == "/image.png" {
                            let mut bytes = Vec::new();
                            let mut encoder = png::Encoder::new(&mut bytes, 80, 40);
                            encoder.set_color(png::ColorType::Rgb);
                            encoder.set_depth(png::BitDepth::Eight);
                            if let Ok(mut writer) = encoder.write_header() {
                                let color = if value.load(Ordering::Acquire) == 0 {
                                    [220, 70, 20]
                                } else {
                                    [20, 70, 220]
                                };
                                let pixels = std::iter::repeat_n(color, 80 * 40)
                                    .flatten()
                                    .collect::<Vec<_>>();
                                let _ = writer.write_image_data(&pixels);
                            }
                            let _ = write!(
                                stream,
                                "HTTP/1.1 200 OK\r\nContent-Type: image/png\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                                bytes.len()
                            );
                            let _ = stream.write_all(&bytes);
                        } else if path == "/photos" {
                            let _=respond(&mut stream,200,b"<!doctype html><style>body{margin:0;background:white}img{width:120px;height:80px;object-fit:cover;object-position:20% 60%;border:2px solid black}.bg{width:150px;height:70px;background-image:url('/image.png');background-size:cover}</style><img src='/image.png'><picture><source srcset='/image.png'><img src='/image.png'></picture><div class='bg'></div><svg width='12' height='12'><rect width='12' height='12' fill='red'/></svg>");
                        } else if path == "/page" {
                            if value.load(Ordering::Acquire) == 2 {
                                let _ = respond(&mut stream, 503, b"deliberate unavailable");
                            } else {
                                let content = format!(
                                    "<!doctype html><html><head><meta charset='utf-8'><style>body{{margin:0;background:#fff;font:20px sans-serif}}h1{{padding:32px}}.box{{width:80px;height:80px;background:{}}}</style></head><body><h1>Release fixture</h1><div class='box'></div></body></html>",
                                    if value.load(Ordering::Acquire) == 0 {
                                        "#08786f"
                                    } else {
                                        "#cc3344"
                                    }
                                );
                                let _ = respond(&mut stream, 200, content.as_bytes());
                            }
                        } else if path == "/tall" {
                            let _=respond(&mut stream,200,b"<html><style>body{margin:0;height:20000px;background:linear-gradient(#eee,#ddd)}.mark{position:absolute;top:19000px;width:50px;height:50px;background:red}</style><div class=mark></div></html>");
                        } else if path == "/hang" {
                            // no response until shutdown; bounded polling
                            for _ in 0..150 {
                                if shutdown.load(Ordering::Acquire) {
                                    break;
                                }
                                std::thread::sleep(Duration::from_millis(20));
                            }
                        } else {
                            let _ = respond(&mut stream, 404, b"not found");
                        }
                        }));
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10))
                    }
                    Err(_) => break,
                }
            }
            for handler in handlers {
                let _ = handler.join();
            }
        });
        Ok(Self {
            address,
            state,
            stop,
            thread: Some(thread),
            hits,
        })
    }
}
fn respond(s: &mut TcpStream, status: u32, body: &[u8]) -> std::io::Result<()> {
    write!(
        s,
        "HTTP/1.1 {status} Fixture\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body.len()
    )?;
    s.write_all(body)
}
fn kill_worker(pid: u32) -> Result<()> {
    // Kill only the owned worker, not its tree: supervision must clean up after
    // this deliberately abrupt crash on both Unix and Windows.
    #[cfg(unix)]
    ensure!(
        unsafe { libc::kill(pid as i32, libc::SIGKILL) } == 0,
        "crash owned worker: {}",
        std::io::Error::last_os_error()
    );
    #[cfg(windows)]
    ensure!(
        std::process::Command::new("taskkill")
            .args(["/F", "/PID", &pid.to_string()])
            .status()?
            .success(),
        "crash owned worker"
    );
    Ok(())
}
#[test]
fn phase6_fixture_speculative_connections_do_not_block_ready_requests() -> Result<()> {
    let site = Site::start(PathBuf::from("unused-pack"))?;
    let address = site.address.strip_prefix("http://").unwrap();
    // Chromium can open a socket without immediately sending a request. An
    // accept/read serial server lets these idle sockets starve real navigation.
    let idle = (0..4)
        .map(|_| TcpStream::connect(address))
        .collect::<std::io::Result<Vec<_>>>()?;
    std::thread::sleep(Duration::from_millis(50));
    let mut ready = TcpStream::connect(address)?;
    ready.set_read_timeout(Some(Duration::from_millis(800)))?;
    ready.write_all(b"GET /page HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")?;
    let mut reply = String::new();
    let result = ready.read_to_string(&mut reply);
    drop(idle);
    result.context("ready request starved behind speculative idle connections")?;
    ensure!(
        reply.starts_with("HTTP/1.1 200"),
        "fixture response missing"
    );
    Ok(())
}
impl Drop for Site {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
/// Source authenticity is checked before packaging. This intentionally imports
/// only the old fixture runtime's verified bytes, not its fixture worker behavior.
fn development_pack(source: &Path, pack: &Path) -> Result<SigningKey> {
    let old_bytes = fs::read(source.join("manifest.json"))?;
    SigningKey::from_bytes(&[17; 32])
        .verifying_key()
        .verify_strict(
            &old_bytes,
            &Signature::from_slice(&fs::read(source.join("manifest.sig"))?)?,
        )?;
    let old: serde_json::Value = serde_json::from_slice(&old_bytes)?;
    let old = old["files"]
        .as_object()
        .context("old signed file inventory")?;
    let mut inputs: BTreeMap<String, (PathBuf, Entry)> = BTreeMap::new();
    let cancel = AtomicBool::new(false);
    for (name, e) in old {
        let path = source.join(name);
        let m = fs::symlink_metadata(&path)?;
        anyhow::ensure!(
            m.is_file() && m.len() == e["bytes"].as_u64().unwrap(),
            "historical source changed"
        );
        let sha = distribution::hash(File::open(&path)?, &cancel)?;
        anyhow::ensure!(
            sha == e["sha256"].as_str().unwrap(),
            "historical source checksum changed"
        );
        if name.starts_with("node/")
            || name.starts_with("browsers/")
            || name.starts_with("worker/node_modules/")
        {
            inputs.insert(
                name.clone(),
                (
                    path,
                    Entry {
                        sha256: sha,
                        bytes: m.len(),
                        executable: e["executable"].as_bool().unwrap(),
                    },
                ),
            );
        }
    }
    let project = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for name in [
        "src/index.mjs",
        "src/capture.mjs",
        "src/discover.mjs",
        "src/protocol.mjs",
        "src/sessions.mjs",
        "test/protocol.test.mjs",
        "test/capture.test.mjs",
        "test/resources.browser.test.mjs",
        "test/phase4.browser.test.mjs",
        "test/phase5.browser.test.mjs",
        "test/fixtures/session-requests.ndjson",
    ] {
        let path = project.join("worker/product").join(name);
        inputs.insert(
            format!("worker/{name}"),
            (
                path.clone(),
                Entry {
                    sha256: distribution::hash(File::open(&path)?, &cancel)?,
                    bytes: fs::metadata(path)?.len(),
                    executable: false,
                },
            ),
        );
    }
    // Fresh ephemeral local-development signing key. No production key/host.
    let mut entropy = [0; 32];
    entropy[..16].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    entropy[16..].copy_from_slice(uuid::Uuid::new_v4().as_bytes());
    let key = SigningKey::from_bytes(&entropy);
    let m = Manifest {
        links: BTreeMap::new(),
        modes: BTreeMap::new(),
        version: 2,
        id: "development-node22.20.0-pw1.56.1-worker4".into(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        scope: Scope::DevelopmentLoopback,
        support_until: None,
        identity: RuntimeIdentity::Pinned {
            node: "22.20.0".into(),
            worker: regressee_runtime::protocol::WORKER_VERSION.into(),
            playwright: "1.56.1".into(),
            browser: "141.0.7390.37".into(),
        },
        node: "node/bin/node".into(),
        worker: "worker/src/index.mjs".into(),
        browser_root: "browsers".into(),
        files: inputs
            .iter()
            .map(|(n, (_, e))| (n.clone(), e.clone()))
            .collect(),
    };
    let bytes = serde_json::to_vec(&m)?;
    let mut z = ZipWriter::new(File::options().create_new(true).write(true).open(pack)?);
    let opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(1));
    z.start_file("manifest.json", opts)?;
    z.write_all(&bytes)?;
    z.start_file("manifest.sig", opts)?;
    z.write_all(&key.sign(&bytes).to_bytes())?;
    for (name, (path, _)) in inputs {
        z.start_file(name, opts)?;
        std::io::copy(&mut File::open(path)?, &mut z)?;
    }
    z.finish()?.sync_all()?;
    Ok(key)
}
#[test]
#[ignore = "serial Phase 5 check; requires fresh signed Phase 4 cache/distribution and a unique output"]
fn actual_phase5_product_analysis_composite_and_replacement() -> Result<()> {
    use regressee_app::{analysis_engine as e, review};
    let output = PathBuf::from(std::env::var("REGRESSEE_PHASE5_EVIDENCE")?);
    fs::create_dir(&output)?;
    let d: Distribution =
        serde_json::from_reader(File::open(std::env::var("REGRESSEE_PHASE5_DISTRIBUTION")?)?)?;
    d.validate()?;
    ensure!(
        d.scope == Scope::DevelopmentLoopback,
        "only the owned development pack is authorized"
    );
    let paths = AppPaths::isolated(&output.join("app"));
    let mut store = Store::open(paths.clone())?;
    let product = Product {
        distribution: Some(d.clone()),
        runtime_cache: PathBuf::from(std::env::var("REGRESSEE_PHASE5_RUNTIME_CACHE")?),
        official_runtime: false,
        auth: Default::default(),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    let installed = distribution::resolve(&product.runtime_cache, &d, &cancel)?;
    for name in ["src/capture.mjs", "test/phase5.browser.test.mjs"] {
        ensure!(
            distribution::hash(
                File::open(installed.root.join("worker").join(name))?,
                &cancel
            )? == distribution::hash(
                File::open(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../worker/product")
                        .join(name)
                )?,
                &cancel
            )?,
            "signed Phase 5 source differs from maintained source"
        );
    }
    // This unmanaged Node-suite fixture is NOT a registered Product launch.
    // Never place an unmarked browser directory in Product's reserved namespace:
    // fail-closed recovery must not skip unknown ownership to accommodate tests.
    let worker_parent = PathBuf::from(
        std::env::var_os("REGRESSEE_WORKER_SCRATCH_PARENT")
            .context("explicit physical short worker parent required")?,
    );
    let scratch = tempfile::Builder::new()
        .prefix("n-")
        .rand_bytes(6)
        .tempdir_in(
            worker_parent
                .parent()
                .context("physical Node-suite parent required")?,
        )?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o700))?;
    }
    let suite = std::process::Command::new(installed.root.join("node/bin/node"))
        .args(["--test", "--test-concurrency=1"])
        .arg(installed.root.join("worker/test/phase5.browser.test.mjs"))
        .env("PLAYWRIGHT_BROWSERS_PATH", installed.root.join("browsers"))
        .env(
            "REGRESSEE_PHASE5_WORKER_OUTPUT",
            output.join("replacement-worker"),
        )
        .env("TMPDIR", scratch.path())
        .env_remove("NODE_OPTIONS")
        .env_remove("NODE_TLS_REJECT_UNAUTHORIZED")
        .output()?;
    fs::write(
        output.join("signed-phase5-worker.log"),
        [suite.stdout, suite.stderr].concat(),
    )?;
    ensure!(suite.status.success(), "See signed-phase5-worker.log");
    let site = Site::start(output.join("unused-runtime.zip"))?;
    let mut project = Project::draft(
        "Actual owned Phase 5 browser fixture".into(),
        &format!("{}/page", site.address),
    )?;
    project
        .sources
        .push(format!("{}/tall", site.address).parse()?);
    project.discovery.mode = DiscoveryMode::OnlyAddresses;
    project.discovery.speed = Speed::Fast;
    project.discovery.respect_robots = false;
    project.discovery.advanced.robots_authorized = true;
    project.profiles[0].width = 600;
    project.profiles[0].height = 300;
    project.profiles[0].extent = CaptureExtent::VisibleScreen;
    store.save_project(&project, None)?;
    let preview = product.discover(&store, project.id, cancel.clone(), &mut |_, _, _| {})?;
    let before = product.capture_inventory(
        &mut store,
        project.id,
        Some(preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    ensure!(
        before.run.state == JobState::Completed && before.run.targets.len() == 2,
        "actual baseline incomplete"
    );
    let activated = product.activate(&mut store, before.run.id, 1)?;
    let baseline = store.load_baseline(activated.project.active_baseline.unwrap())?;
    let original = serde_json::to_vec(&before)?;
    let old_baseline = serde_json::to_vec(&baseline)?;
    site.state.store(1, Ordering::Release);
    let check = product.capture_inventory(
        &mut store,
        project.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    let evidence = store.project_evidence(project.id)?;
    let analysis = evidence.comparison.context("real analysis missing")?;
    ensure!(
        analysis
            .perceptual
            .as_ref()
            .is_some_and(|p| p.reports.len() == 2 && p.failures.is_empty()),
        "actual Chromium PNG integration rejected"
    );
    let key = check
        .run
        .targets
        .iter()
        .find(|t| t.requested.path() == "/page")
        .unwrap()
        .key
        .clone();
    ensure!(
        analysis
            .revision
            .results
            .iter()
            .find(|r| r.key == key)
            .unwrap()
            .classification
            == Classification::Changed,
        "actual changed page was not flagged"
    );
    store.save_review(&regressee_core::domain::ReviewDecision {
        analysis: analysis.revision.id,
        key: key.clone(),
        state: ReviewState::Intentional,
        note: "Synthetic fixture color change".into(),
        reviewed_at: distribution::now(),
    })?;
    let old_approval = store.approval_preview(analysis.revision.id, std::slice::from_ref(&key))?;
    let baseline_count = store.baseline_history(project.id)?.len();
    let relaxed = review::analyze_run(
        &mut store,
        check.run.id,
        review::preset(e::Preset::Relaxed),
        &cancel,
    )?;
    ensure!(
        store.review_history(check.run.id)?[0].analysis == analysis.revision.id,
        "sensitivity erased review association"
    );
    ensure!(
        store
            .accept_selected(&old_approval, activated.revision)
            .is_err()
            && store.baseline_history(project.id)?.len() == baseline_count
            && store.load_project(project.id)?.project.active_baseline == Some(baseline.id),
        "analysis switch accepted a stale Chromium-source preview"
    );
    let approval = store.approval_preview(relaxed.revision.id, std::slice::from_ref(&key))?;
    ensure!(approval.inherited == 1, "inheritance count wrong");
    let approved = store.accept_selected(&approval, activated.revision)?;
    let composite = store.load_baseline(approved.project.active_baseline.unwrap())?;
    ensure!(
        composite
            .references
            .iter()
            .find(|r| r.key == key)
            .unwrap()
            .run
            == check.run.id,
        "selected capture missing"
    );
    ensure!(
        composite
            .references
            .iter()
            .find(|r| r.key != key)
            .unwrap()
            .run
            == before.run.id,
        "inherited capture changed"
    );
    let again = product.capture_inventory(
        &mut store,
        project.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    let recheck = store
        .project_evidence(project.id)?
        .comparison
        .context("new baseline check missing")?;
    ensure!(
        recheck.revision.baseline == composite.id
            && again.run.targets.len() == 2
            && recheck
                .revision
                .results
                .iter()
                .all(|r| r.classification == Classification::NoSignificantChange),
        "composite recheck did not use exact inherited/selected references"
    );
    ensure!(
        serde_json::to_vec(&store.load_run(before.run.id)?)? == original
            && serde_json::to_vec(&store.load_baseline(baseline.id)?)? == old_baseline,
        "past evidence changed"
    );
    let restored = store.restore_baseline(baseline.id, approved.revision)?;
    ensure!(
        restored.project.active_baseline == Some(baseline.id),
        "historical restore failed"
    );
    site.state.store(0, Ordering::Release);
    let mut photos = Project::draft(
        "Actual replacement-only fixture".into(),
        &format!("{}/photos", site.address),
    )?;
    photos.discovery.mode = DiscoveryMode::OnlyAddresses;
    photos.discovery.speed = Speed::Fast;
    photos.discovery.respect_robots = false;
    photos.discovery.advanced.robots_authorized = true;
    photos.profiles[0].width = 600;
    photos.profiles[0].height = 300;
    photos.profiles[0].extent = CaptureExtent::VisibleScreen;
    photos.profiles[0].image_mode = ImageMode::ReplaceRaster {
        algorithm: "neutral-raster-v2".into(),
    };
    store.save_project(&photos, None)?;
    let inventory = product.discover(&store, photos.id, cancel.clone(), &mut |_, _, _| {})?;
    let representative =
        product.preview_images(&mut store, photos.id, inventory.clone(), cancel.clone())?;
    ensure!(
        representative.run.intent == RunIntent::RepresentativePreview
            && representative.captures.first().is_some_and(|c| c
                .metadata
                .replacement
                .as_ref()
                .is_some_and(|r| r["count"] == 3)),
        "actual representative replacement preview missing"
    );
    ensure!(
        store.activate_baseline(representative.run.id, 1).is_err(),
        "representative preview masqueraded as whole baseline"
    );
    let photo_before = product.capture_inventory(
        &mut store,
        photos.id,
        Some(inventory),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    let photo_baseline = product.activate(&mut store, photo_before.run.id, 1)?;
    site.state.store(1, Ordering::Release);
    let photo_after = product.capture_inventory(
        &mut store,
        photos.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    let photo_analysis = store
        .project_evidence(photos.id)?
        .comparison
        .context("replacement analysis missing")?;
    ensure!(
        photo_analysis.revision.results[0].classification == Classification::NoSignificantChange
            && photo_before.captures[0].image.original == photo_after.captures[0].image.original,
        "repeated photo replacement mismatched immutable material profile"
    );
    ensure!(
        photo_before.captures[0].metadata.profile.image_mode
            == photo_after.captures[0].metadata.profile.image_mode
            && photo_after.captures[0]
                .metadata
                .replacement
                .as_ref()
                .is_some_and(|r| r["geometryPreserved"] == true),
        "replacement geometry/mode metadata missing"
    );
    fs::write(
        output.join("replacement-product.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"representative":representative,"baseline":photo_baseline,"before":photo_before,"after":photo_after,"analysis":photo_analysis}),
        )?,
    )?;
    fs::write(
        output.join("phase5-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"qualification":"actual owned signed-worker product checks; not native/human/public release approval","stale_approval_after_analysis_switch":"refused; zero baseline records/activation","before":before,"check":check,"analysis":analysis,"approval":approval,"composite":composite,"recheck":recheck,"restore":restored,"hits":*site.hits.lock().unwrap()}),
        )?,
    )?;
    Ok(())
}
fn fresh_project(store: &mut Store, site: &Site, path: &str) -> Result<Project> {
    let p = Project::draft(
        format!("Fixture {path}"),
        &format!("{}{path}", site.address),
    )?;
    store.save_project(&p, None)?;
    Ok(p)
}
#[test]
#[ignore = "serial current signed worker replacement diagnosis/guard, fresh private output"]
fn actual_phase7_replacement_preview_baseline_check() -> Result<()> {
    let output = PathBuf::from(std::env::var("REGRESSEE_PHASE7_REPLACEMENT_EVIDENCE")?);
    fs::create_dir(&output)?;
    let d: Distribution =
        serde_json::from_reader(File::open(std::env::var("REGRESSEE_PHASE5_DISTRIBUTION")?)?)?;
    ensure!(
        d.scope == Scope::DevelopmentLoopback,
        "owned loopback fixture only"
    );
    let product = Product {
        distribution: Some(d),
        runtime_cache: PathBuf::from(std::env::var("REGRESSEE_PHASE5_RUNTIME_CACHE")?),
        official_runtime: false,
        auth: Default::default(),
    };
    let mut store = Store::open(AppPaths::isolated(&output.join("app")))?;
    let site = Site::start(output.join("unused-runtime.zip"))?;
    let mut project = Project::draft(
        "Replacement lifecycle guard".into(),
        &format!("{}/photos", site.address),
    )?;
    project.discovery.mode = DiscoveryMode::OnlyAddresses;
    project.discovery.speed = Speed::Fast;
    project.discovery.respect_robots = false;
    project.discovery.advanced.robots_authorized = true;
    project.profiles[0].width = 600;
    project.profiles[0].height = 300;
    project.profiles[0].extent = CaptureExtent::VisibleScreen;
    project.profiles[0].image_mode = ImageMode::ReplaceRaster {
        algorithm: "neutral-raster-v2".into(),
    };
    store.save_project(&project, None)?;
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    let inventory = product.discover(&store, project.id, cancel.clone(), &mut |_, _, _| {})?;
    let preview =
        product.preview_images(&mut store, project.id, inventory.clone(), cancel.clone())?;
    let before = product.capture_inventory(
        &mut store,
        project.id,
        Some(inventory),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    let activated = product.activate(&mut store, before.run.id, 1)?;
    let immutable = serde_json::to_vec(&store.load_run(before.run.id)?)?;
    site.state.store(1, Ordering::Release);
    let after = product.capture_inventory(
        &mut store,
        project.id,
        None,
        true,
        None,
        cancel,
        pause,
        &mut |_, _, _| {},
    )?;
    let analysis = store
        .project_evidence(project.id)?
        .comparison
        .context("replacement analysis missing")?;
    fs::write(
        output.join("replacement.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"preview":preview,"before":before,"after":after,"analysis":analysis,"activated":activated,"hits":*site.hits.lock().unwrap()}),
        )?,
    )?;
    ensure!(
        serde_json::to_vec(&store.load_run(before.run.id)?)? == immutable,
        "replacement changed immutable baseline capture"
    );
    let a = before
        .captures
        .first()
        .context("baseline capture missing")?;
    let b = after.captures.first().context("check capture missing")?;
    ensure!(
        a.metadata.profile == b.metadata.profile
            && a.metadata.profile.image_mode
                == (ImageMode::ReplaceRaster {
                    algorithm: "neutral-raster-v2".into()
                }),
        "replacement mode/profile mismatch"
    );
    ensure!(
        preview
            .captures
            .first()
            .is_some_and(|p| p.image.original == a.image.original)
            && a.image.original == b.image.original,
        "preview/baseline/check originals differ"
    );
    ensure!(
        a.metadata.warnings.is_empty()
            && b.metadata.warnings.is_empty()
            && analysis.revision.results[0].classification == Classification::NoSignificantChange,
        "warning-bearing replacement cannot be clean"
    );
    Ok(())
}
/// Persistent non-secret stage markers plus a cancellation deadline, not longer
/// worker timeouts. The outer process owner still bounds noncancellable setup I/O.
fn phase6_stage<T>(
    output: &Path,
    name: &str,
    seconds: u64,
    cancel: &Arc<AtomicBool>,
    run: impl FnOnce() -> Result<T>,
) -> Result<T> {
    let mut log = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(output.join("stages.jsonl"))?;
    writeln!(
        log,
        "{}",
        serde_json::json!({"stage":name,"state":"started","deadline_seconds":seconds})
    )?;
    log.sync_all()?;
    eprintln!("P6 stage started: {name} (bounded {seconds}s)");
    let start = Instant::now();
    let (done, wait) = std::sync::mpsc::channel();
    let flag = cancel.clone();
    let timed_out = Arc::new(AtomicBool::new(false));
    let expired = timed_out.clone();
    let watch = std::thread::spawn(move || {
        if matches!(
            wait.recv_timeout(Duration::from_secs(seconds)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ) {
            expired.store(true, Ordering::Release);
            flag.store(true, Ordering::Release);
        }
    });
    let result = run();
    let _ = done.send(());
    watch
        .join()
        .map_err(|_| anyhow::anyhow!("stage watchdog stopped"))?;
    let expired = timed_out.load(Ordering::Acquire);
    writeln!(
        log,
        "{}",
        serde_json::json!({"stage":name,"state":"settled","success":result.is_ok() && !expired,"deadline_reached":expired,"wall_seconds":start.elapsed().as_secs_f64()})
    )?;
    log.sync_all()?;
    eprintln!(
        "P6 stage settled: {name}; success={}; {:.3}s",
        result.is_ok() && !expired,
        start.elapsed().as_secs_f64()
    );
    ensure!(
        !expired,
        "Phase 6 stage {name} reached its bounded deadline"
    );
    result.with_context(|| format!("Phase 6 stage {name}"))
}
#[test]
#[ignore = "serial fresh signed runtime and retained Phase 6 output; no public/native/OS-key qualification"]
fn actual_phase6_composite_zip_fresh_recipient_login_and_fixed_mapped_check() -> Result<()> {
    use regressee_runtime::secrets::{
        EncryptedSessions, KeyProvider, SecretError, SessionMaterial,
    };
    struct FixtureKey(u8);
    impl KeyProvider for FixtureKey {
        fn key(&mut self) -> std::result::Result<zeroize::Zeroizing<[u8; 32]>, SecretError> {
            Ok(zeroize::Zeroizing::new([self.0; 32]))
        }
    }
    let output = PathBuf::from(std::env::var("REGRESSEE_PHASE6_EVIDENCE")?);
    fs::create_dir(&output)?;
    let config = PathBuf::from(std::env::var("REGRESSEE_PHASE6_DISTRIBUTION")?);
    let d: Distribution = serde_json::from_reader(File::open(&config)?)?;
    ensure!(
        d.scope == Scope::DevelopmentLoopback,
        "Only freshly signed owned-loopback runtime authorized"
    );
    let pack = config.parent().unwrap().join("development-worker4.zip");
    let production = Site::start(pack.clone())?;
    let staging = Site::start(pack)?;
    let paths = AppPaths::isolated(&output.join("owner"));
    let mut owner = Store::open(paths.clone())?;
    let product = Product {
        distribution: Some(d.clone()),
        runtime_cache: PathBuf::from(std::env::var("REGRESSEE_PHASE6_RUNTIME_CACHE")?),
        official_runtime: false,
        auth: Default::default(),
    };
    *product.auth.lock().unwrap() =
        regressee_app::auth::AuthOwner::with_test_vault(EncryptedSessions::with_provider(
            &paths.data.join("private-sessions"),
            Box::new(FixtureKey(42)),
        )?);
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    let mut p = Project::draft(
        "Actual portable authenticated fixture".into(),
        &format!(
            "{}/auth/page?portable=change&private-query=shared-warning&a=2&a=1#/orders",
            production.address
        ),
    )?;
    p.sources
        .push(format!("{}/auth/page?portable=static", production.address).parse()?);
    p.environments[0].access = AccessPolicy::InteractiveLogin;
    p.discovery.mode = DiscoveryMode::OnlyAddresses;
    p.discovery.speed = Speed::Fast;
    p.discovery.respect_robots = false;
    p.discovery.advanced.robots_authorized = true;
    p.profiles[0].width = 600;
    p.profiles[0].height = 300;
    p.profiles[0].extent = CaptureExtent::VisibleScreen;
    owner.save_project(&p, None)?;
    let material = SessionMaterial::new(serde_json::to_vec(
        &serde_json::json!({"storage":{"cookies":[{"name":"fixture-session","value":"ready","domain":"127.0.0.1","path":"/","expires":-1,"httpOnly":true,"secure":false,"sameSite":"Lax"}],"origins":[]},"session_storage":{}}),
    )?)?;
    let owner_session = SessionReference {
        id: SessionId::new(),
        project: p.id,
        environment: p.environments[0].id,
    };
    product.auth.lock().unwrap().save_fixture_session(
        &mut owner,
        &owner_session,
        &material,
        false,
    )?;
    let inventory = phase6_stage(&output, "owner-discover", 45, &cancel, || {
        product.discover(&owner, p.id, cancel.clone(), &mut |_, _, _| {})
    })?;
    let before = phase6_stage(&output, "owner-baseline", 60, &cancel, || {
        product.capture_inventory(
            &mut owner,
            p.id,
            Some(inventory),
            false,
            None,
            cancel.clone(),
            pause.clone(),
            &mut |_, _, _| {},
        )
    })?;
    ensure!(
        before.run.state == JobState::Completed && before.run.targets.len() == 2,
        "Actual owner baseline incomplete"
    );
    let activated = owner.activate_baseline(before.run.id, 1)?;
    production.state.store(1, Ordering::Release);
    staging.state.store(1, Ordering::Release);
    let check = phase6_stage(&output, "owner-check", 60, &cancel, || {
        product.capture_inventory(
            &mut owner,
            p.id,
            None,
            true,
            None,
            cancel.clone(),
            pause.clone(),
            &mut |_, _, _| {},
        )
    })?;
    let analysis = owner
        .project_evidence(p.id)?
        .comparison
        .context("Owner analysis missing")?;
    let selected = analysis
        .revision
        .results
        .iter()
        .find(|r| r.classification == Classification::Changed)
        .context("No actual changed target")?
        .key
        .clone();
    let approval = owner.approval_preview(analysis.revision.id, std::slice::from_ref(&selected))?;
    let saved = owner.accept_selected(&approval, activated.revision)?;
    let composite = owner.load_baseline(saved.project.active_baseline.unwrap())?;
    ensure!(
        composite.references.iter().any(|r| r.run == before.run.id)
            && composite.references.iter().any(|r| r.run == check.run.id),
        "Not a selected/inherited composite"
    );
    let old_run = serde_json::to_vec(&before)?;
    // Today's FUTURE settings must not invalidate the selected historical capture or fixed targets.
    let mut future = saved.project;
    future.profiles[0].width = 640;
    future.discovery.section = Some("/future".into());
    let future = owner.save_replacement_project(&future, saved.revision)?;
    let zip = output.join("selected-baseline.zip");
    let export = owner.export_preview(p.id, composite.id)?;
    phase6_stage(&output, "export", 20, &cancel, || {
        owner.export_archive(&export, &zip, &cancel)
    })?;
    let recipient_paths = AppPaths::isolated(&output.join("recipient"));
    let mut recipient = Store::open(recipient_paths.clone())?;
    let hits_before = staging.hits.lock().unwrap().clone();
    let expected = recipient.inspect_archive(&zip, &cancel)?.preview;
    let mut imported = phase6_stage(&output, "new-store-import", 20, &cancel, || {
        recipient.import_archive(&zip, &expected, &cancel)
    })?;
    ensure!(
        *staging.hits.lock().unwrap() == hits_before,
        "Import navigated automatically"
    );
    let baseline = recipient.load_baseline(imported.project.active_baseline.unwrap())?;
    let fixed = recipient.load_inventory(baseline.inventory)?;
    let mut inspected = Vec::new();
    for (i, r) in baseline.references.iter().enumerate() {
        let source = recipient.load_run(r.run)?;
        let c = source.evidence(&r.key).unwrap();
        let old = &composite.references[i];
        let original = owner.load_run(old.run)?;
        let m = original.evidence(&old.key).unwrap();
        ensure!(
            r.artifact == old.artifact
                && r.captured_at == old.captured_at
                && c.metadata.requested == m.metadata.requested
                && c.metadata.final_url == m.metadata.final_url
                && source.run.rendering == original.run.rendering
                && recipient.is_imported_run(r.run)?,
            "Portable provenance mismatch"
        );
        let mut profile = c.metadata.profile.clone();
        profile.id = m.metadata.profile.id;
        ensure!(
            profile == m.metadata.profile
                && c.metadata.profile.width == 600
                && imported.project.profiles[0].width == 640,
            "Frozen profile was replaced by future settings"
        );
        ensure!(
            fs::read(recipient.artifact_path(&r.artifact)?)?
                == fs::read(owner.artifact_path(&r.artifact)?)?,
            "Original PNG changed"
        );
        inspected.push(serde_json::json!({"key":r.key,"source":r.run,"original":r.artifact,"captured_at":r.captured_at,"metadata":c.metadata,"rendering":source.run.rendering,"imported":true}));
    }
    let manifest = recipient.archive_manifest(imported.project.id, baseline.id)?;
    let text = serde_json::to_string(&manifest)?;
    ensure!(
        !text.contains(&owner_session.id.to_string())
            && !text.contains("fixture-session")
            && !text.contains("private-sessions")
            && !text.contains(&paths.data.display().to_string()),
        "Owner login/private paths leaked into archive"
    );
    for env in &imported.project.environments {
        ensure!(
            recipient
                .session_reference(imported.project.id, env.id)?
                .is_none(),
            "Imported old session"
        );
    }
    // Explicit NEW recipient environment mapping, separate secret owner and fresh private runtime setup.
    imported.project.environments[0].mappings[0].destination =
        format!("{}/", staging.address).parse()?;
    imported = recipient.save_replacement_project(&imported.project, imported.revision)?;
    let recipient_env = imported.project.environments[0].id;
    recipient.select_environment(
        imported.project.id,
        recipient_env,
        &regressee_storage::LocalLoginSettings {
            representative: Some(format!("{}/auth/page?portable=change", staging.address).parse()?),
            login_address: Some(format!("{}/fixture-login", staging.address).parse()?),
            ..Default::default()
        },
    )?;
    let mut recipient_trust = d;
    recipient_trust.url = format!("{}/runtime.zip", staging.address).parse()?;
    let fresh = Product {
        distribution: Some(recipient_trust),
        runtime_cache: recipient_paths.cache.join("product-runtime"),
        official_runtime: false,
        auth: Default::default(),
    };
    ensure!(
        fresh.status(&cancel).is_err(),
        "Archive supplied runtime binaries"
    );
    phase6_stage(&output, "recipient-runtime-setup", 100, &cancel, || {
        fresh.setup(cancel.clone(), &mut |_, _| {})
    })?;
    let no_login_hits = staging.hits.lock().unwrap().clone();
    ensure!(
        fresh
            .capture_inventory(
                &mut recipient,
                imported.project.id,
                None,
                true,
                None,
                cancel.clone(),
                pause.clone(),
                &mut |_, _, _| {}
            )
            .is_err(),
        "Recipient silently reused owner login"
    );
    ensure!(
        *staging.hits.lock().unwrap() == no_login_hits,
        "Missing login caused anonymous navigation"
    );
    *fresh.auth.lock().unwrap() =
        regressee_app::auth::AuthOwner::with_test_vault(EncryptedSessions::with_provider(
            &recipient_paths.data.join("private-sessions"),
            Box::new(FixtureKey(43)),
        )?);
    phase6_stage(&output, "recipient-headed-open-login", 45, &cancel, || {
        fresh.open_login(&recipient, imported.project.id, cancel.clone())
    })?;
    let ready = phase6_stage(&output, "recipient-explicit-ready", 45, &cancel, || {
        fresh.confirm_login(&mut recipient, imported.project.id, cancel.clone())
    })?;
    ensure!(!ready.login_open, "Recipient headed login did not close");
    let (recipient_session, ephemeral) = recipient
        .session_reference(imported.project.id, recipient_env)?
        .context("Explicit ready did not establish a recipient session")?;
    ensure!(
        !ephemeral,
        "Expected separate injected-key recipient persistence"
    );
    ensure!(
        recipient_session.id != owner_session.id,
        "Session namespace collided"
    );
    let mapped = phase6_stage(&output, "recipient-mapped-probe", 40, &cancel, || {
        fresh.probe_environment(&recipient, imported.project.id, cancel.clone())
    })?;
    ensure!(
        mapped.probed && mapped.mapped.len() == 2,
        "Fresh mapped probe failed"
    );
    let before_check_hits = staging.hits.lock().unwrap().clone();
    let recheck = phase6_stage(&output, "recipient-fixed-check", 60, &cancel, || {
        fresh.capture_inventory(
            &mut recipient,
            imported.project.id,
            None,
            true,
            None,
            cancel.clone(),
            pause.clone(),
            &mut |_, _, _| {},
        )
    })?;
    let reanalysis = recipient
        .project_evidence(imported.project.id)?
        .comparison
        .context("Recipient analysis missing")?;
    ensure!(
        recheck.run.state == JobState::Completed
            && recheck.run.targets.len() == 2
            && recheck.all_captures().count() == 2
            && recheck.run.profiles.iter().all(|p| p.width == 600)
            && reanalysis.revision.results.len() == 2
            && reanalysis
                .revision
                .results
                .iter()
                .all(|r| r.classification == Classification::NoSignificantChange),
        "Recipient check did not use exact immutable composite"
    );
    ensure!(
        recheck
            .run
            .targets
            .iter()
            .map(|t| &t.requested)
            .collect::<Vec<_>>()
            == fixed
                .pages
                .iter()
                .filter(|p| p.included)
                .map(|p| &p.canonical)
                .collect::<Vec<_>>(),
        "Recipient fixed target list differs"
    );
    for (url, n) in staging.hits.lock().unwrap().iter() {
        if !matches!(url.as_str(), "/favicon.ico") && n > before_check_hits.get(url).unwrap_or(&0) {
            ensure!(
                url.starts_with("/auth/page?portable="),
                "Unexpected check visit {url}"
            );
        }
    }
    ensure!(
        serde_json::to_vec(&owner.load_run(before.run.id)?)? == old_run
            && owner.load_project(p.id)?.revision == future.revision,
        "Owner history changed"
    );
    fs::write(
        output.join("phase6-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"qualification":"actual signed sandboxed Product on two owned loopback origins and two fresh stores; owner injected session, separate recipient headed self-establishing synthetic account and explicit ready; injected persistence keys, NOT OS-key/native GPUI/human/cross-OS qualification","export_preview":export,"import_preview":expected,"inspected_originals":inspected,"imported_baseline":baseline,"mapped_probe":mapped.mapped,"probe_confirmed":mapped.probed,"recipient_check":recheck,"recipient_analysis":reanalysis,"owner_history_preserved":true,"import_no_navigation":true,"missing_login_zero_navigation":true,"recipient_runtime_freshly_downloaded_and_verified":true,"recipient_headed_open_and_explicit_ready":true,"archive_excludes_owner_state":true,"fixed_targets_only":true}),
        )?,
    )?;
    Ok(())
}
#[test]
#[ignore = "serial fresh signed Phase 4 fixtures; set REGRESSEE_PHASE4_EVIDENCE to a new output"]
fn actual_phase4_auth_environment_and_capture() -> Result<()> {
    use regressee_runtime::secrets::{
        EncryptedSessions, KeyProvider, SecretError, SessionMaterial,
    };
    struct FixtureKey;
    impl KeyProvider for FixtureKey {
        fn key(&mut self) -> std::result::Result<zeroize::Zeroizing<[u8; 32]>, SecretError> {
            Ok(zeroize::Zeroizing::new([43; 32]))
        }
    }
    let output = PathBuf::from(std::env::var("REGRESSEE_PHASE4_EVIDENCE")?);
    fs::create_dir(&output)?;
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.runtime/installed");
    let pack = output.join("development-worker4.zip");
    let signing = development_pack(&source, &pack)?;
    let production = Site::start(pack.clone())?;
    let staging = Site::start(pack)?;
    let paths = AppPaths::isolated(&output.join("app"));
    let mut store = Store::open(paths.clone())?;
    let distribution = Distribution {
        version: 1,
        url: format!("{}/runtime.zip", production.address).parse()?,
        public_key: hex::encode(signing.verifying_key().to_bytes()),
        scope: Scope::DevelopmentLoopback,
    };
    fs::write(
        output.join("distribution.json"),
        serde_json::to_vec_pretty(&distribution)?,
    )?;
    let product = Product {
        distribution: Some(distribution.clone()),
        runtime_cache: paths.cache.join("product-runtime"),
        official_runtime: false,
        auth: Default::default(),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    product.setup(cancel.clone(), &mut |_, _| {})?;
    let installed = distribution::resolve(&product.runtime_cache, &distribution, &cancel)?;
    for name in [
        "src/index.mjs",
        "src/capture.mjs",
        "src/discover.mjs",
        "src/protocol.mjs",
        "src/sessions.mjs",
        "test/phase4.browser.test.mjs",
    ] {
        ensure!(
            distribution::hash(
                File::open(installed.root.join("worker").join(name))?,
                &cancel
            )? == distribution::hash(
                File::open(
                    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                        .join("../../worker/product")
                        .join(name)
                )?,
                &cancel
            )?,
            "signed bytes differ from maintained source"
        );
    }
    let scratch = tempfile::Builder::new()
        .prefix("p4-suite-")
        .tempdir_in("/tmp/opencode")?;
    let suite = std::process::Command::new(installed.root.join("node/bin/node"))
        .args(["--test", "--test-concurrency=1"])
        .arg(installed.root.join("worker/test/protocol.test.mjs"))
        .arg(installed.root.join("worker/test/capture.test.mjs"))
        .arg(
            installed
                .root
                .join("worker/test/resources.browser.test.mjs"),
        )
        .arg(installed.root.join("worker/test/phase4.browser.test.mjs"))
        .env("PLAYWRIGHT_BROWSERS_PATH", installed.root.join("browsers"))
        .env("TMPDIR", scratch.path())
        .env("REGRESSEE_RESOURCE_EVIDENCE", output.join("resources"))
        .env(
            "REGRESSEE_RESOURCE_RUNTIME",
            serde_json::to_string(&installed.manifest.identity)?,
        )
        .env_remove("NODE_OPTIONS")
        .env_remove("NODE_TLS_REJECT_UNAUTHORIZED")
        .env("REGRESSEE_PHASE4_WORKER_OUTPUT", output.join("auth-worker"))
        .output()?;
    fs::write(
        output.join("signed-worker-suite.log"),
        [suite.stdout, suite.stderr].concat(),
    )?;
    ensure!(suite.status.success(), "See signed-worker-suite.log");
    let mut p = Project::draft(
        "Read-only authenticated fixture".into(),
        &format!("{}/auth/page?b=2&a=1&a=0#/orders", production.address),
    )?;
    p.discovery.mode = DiscoveryMode::OnlyAddresses;
    p.discovery.speed = Speed::Fast;
    p.environments[0].access = AccessPolicy::InteractiveLogin;
    p.profiles[0].width = 600;
    p.profiles[0].height = 300;
    p.profiles[0].extent = CaptureExtent::VisibleScreen;
    store.save_project(&p, None)?;
    let state = SessionMaterial::new(serde_json::to_vec(
        &serde_json::json!({"storage":{"cookies":[{"name":"fixture-session","value":"ready","domain":"127.0.0.1","path":"/","expires":-1,"httpOnly":true,"secure":false,"sameSite":"Lax"}],"origins":[]},"session_storage":{}}),
    )?)?;
    let vault_path = paths.data.join("private-sessions");
    *product.auth.lock().unwrap() = regressee_app::auth::AuthOwner::with_test_vault(
        EncryptedSessions::with_provider(&vault_path, Box::new(FixtureKey))?,
    );
    let prod_session = SessionReference {
        id: SessionId::new(),
        project: p.id,
        environment: p.environments[0].id,
    };
    product
        .auth
        .lock()
        .unwrap()
        .save_fixture_session(&mut store, &prod_session, &state, false)?;
    let preview = product.discover(&store, p.id, cancel.clone(), &mut |_, _, _| {})?;
    let baseline = product.capture_inventory(
        &mut store,
        p.id,
        Some(preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    ensure!(
        baseline.run.state == JobState::Completed,
        "authenticated baseline did not complete"
    );
    let saved = product.activate(&mut store, baseline.run.id, 1)?;
    let active = saved.project.active_baseline;
    let before_hash = baseline.captures[0].image.original.clone();
    // Direct signed-worker callers cannot reinstall old cookie-only material with
    // new origin authority, or visit the new origin after the failed installation.
    let mut worker = Worker::spawn(&installed)?;
    let job = RunId::new();
    worker.handshake(job, cancel.clone())?;
    let mut direct: serde_json::Value = serde_json::from_slice(state.expose())?;
    direct["binding"] =
        serde_json::json!({"version":1,"origins":[format!("{}/", production.address)]});
    let stage_visits = staging.hits.lock().unwrap().clone();
    let error = worker
        .request(
            Request {
                version: 2,
                request_id: 2,
                job,
                command: Command::InstallSession {
                    session: prod_session.clone(),
                    origins: vec![format!("{}/", staging.address)],
                    material: regressee_runtime::protocol::SecretPayload(serde_json::to_string(
                        &direct,
                    )?),
                },
            },
            Instant::now() + Duration::from_secs(10),
            cancel.clone(),
            &mut |_| Ok(()),
        )
        .unwrap_err();
    assert_eq!(
        error.downcast_ref::<WorkerFailure>().unwrap().0,
        FailureCode::NeedsLogin
    );
    let error = worker
        .request(
            Request {
                version: 2,
                request_id: 3,
                job,
                command: Command::DiscoverAuthenticated {
                    session: prod_session.clone(),
                    address: format!("{}/preview/page", staging.address),
                    section: None,
                },
            },
            Instant::now() + Duration::from_secs(10),
            cancel.clone(),
            &mut |_| Ok(()),
        )
        .unwrap_err();
    assert_eq!(
        error.downcast_ref::<WorkerFailure>().unwrap().0,
        FailureCode::InvalidRequest
    );
    assert_eq!(*staging.hits.lock().unwrap(), stage_visits);
    worker.stop(Instant::now() + Duration::from_secs(3))?;
    let mut edited = saved.project.clone();
    let mut stage = edited.environments[0].clone();
    stage.id = EnvironmentId::new();
    stage.name = "Staging".into();
    stage.mappings = vec![OriginMapping {
        source: format!("{}/auth", production.address).parse()?,
        destination: format!("{}/preview", staging.address).parse()?,
    }];
    edited.environments.push(stage.clone());
    store.save_replacement_project(&edited, saved.revision)?;
    store.select_environment(p.id, stage.id, &Default::default())?;
    let stage_scope = SessionReference {
        id: SessionId::new(),
        project: p.id,
        environment: stage.id,
    };
    product
        .auth
        .lock()
        .unwrap()
        .save_fixture_session(&mut store, &stage_scope, &state, false)?;
    let status = product.environment_status(&store, p.id)?;
    ensure!(
        status.mapped[0].1 == format!("{}/preview/page?b=2&a=1&a=0#/orders", staging.address),
        "mapping preview lost ordered query/hash"
    );
    let visits = staging.hits.lock().unwrap().clone();
    assert!(
        product
            .capture_inventory(
                &mut store,
                p.id,
                None,
                true,
                None,
                cancel.clone(),
                pause.clone(),
                &mut |_, _, _| {}
            )
            .is_err()
    );
    assert_eq!(*staging.hits.lock().unwrap(), visits);
    assert!(
        product
            .probe_environment(&store, p.id, cancel.clone())?
            .probed
    );
    let check = product.capture_inventory(
        &mut store,
        p.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(check.run.inventory, baseline.run.inventory);
    assert_eq!(
        check.run.targets[0].requested,
        baseline.run.targets[0].requested
    );
    assert!(
        check.captures[0]
            .metadata
            .requested
            .starts_with(&staging.address)
    );
    assert_eq!(check.run.state, JobState::Completed);
    assert_eq!(
        store
            .project_evidence(p.id)?
            .comparison
            .unwrap()
            .revision
            .results[0]
            .classification,
        Classification::NoSignificantChange
    );
    // Restart the explicit test provider, not the OS keychain: decrypt saved state
    // through a new owner and successfully replay another isolated worker context.
    *product.auth.lock().unwrap() = regressee_app::auth::AuthOwner::with_test_vault(
        EncryptedSessions::with_provider(&vault_path, Box::new(FixtureKey))?,
    );
    product.probe_environment(&store, p.id, cancel.clone())?;
    let restarted = product.capture_inventory(
        &mut store,
        p.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(restarted.run.state, JobState::Completed);
    staging.state.store(3, Ordering::Release);
    let expired = product.capture_inventory(
        &mut store,
        p.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(expired.run.state, JobState::Paused);
    assert_eq!(
        expired.run.targets[0].outcome,
        TargetOutcome::Failed {
            code: CaptureError::NeedsLogin
        }
    );
    assert!(expired.captures.is_empty());
    assert_eq!(staging.hits.lock().unwrap().get("/login"), None);
    assert_eq!(store.load_project(p.id)?.project.active_baseline, active);
    assert_eq!(
        store.load_run(baseline.run.id)?.captures[0].image.original,
        before_hash
    );
    staging.state.store(0, Ordering::Release);
    pause.store(false, Ordering::Release);
    let recovered = product.capture_inventory(
        &mut store,
        p.id,
        None,
        false,
        Some(expired.run.id),
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(recovered.run.state, JobState::Completed);
    assert!(
        recovered.run.targets[0]
            .warnings
            .iter()
            .any(|w| w.contains("ResumedAfterLogin"))
    );
    // P1 exact cause: cookie-only state on two same-host, different-port owned
    // origins. Destination approval is not approval to transplant the old login.
    let saved = store.load_project(p.id)?;
    let mut drift = saved.project.clone();
    let production_env = drift.environments[0].id;
    drift.environments[0].mappings = stage.mappings.clone();
    store.save_replacement_project(&drift, saved.revision)?;
    store.select_environment(p.id, production_env, &Default::default())?;
    let prod_visits = production.hits.lock().unwrap().clone();
    let stage_visits = staging.hits.lock().unwrap().clone();
    let require_login = |error: anyhow::Error| {
        assert_eq!(
            error.downcast_ref::<WorkerFailure>().unwrap().0,
            FailureCode::NeedsLogin
        );
    };
    require_login(
        product
            .probe_environment(&store, p.id, cancel.clone())
            .unwrap_err(),
    );
    require_login(
        product
            .discover(&store, p.id, cancel.clone(), &mut |_, _, _| {})
            .unwrap_err(),
    );
    require_login(
        product
            .capture_inventory(
                &mut store,
                p.id,
                None,
                true,
                None,
                cancel.clone(),
                pause.clone(),
                &mut |_, _, _| {},
            )
            .unwrap_err(),
    );
    // The legacy public-only API remains a refusal, not an anonymous bypass.
    assert!(
        product
            .capture(&mut store, p.id, true, cancel.clone(), &mut |_, _, _| {})
            .is_err()
    );
    assert_eq!(*production.hits.lock().unwrap(), prod_visits);
    assert_eq!(*staging.hits.lock().unwrap(), stage_visits); // no cookie OR anonymous visit
    let status = product.environment_status(&store, p.id)?;
    assert!(!status.probed && !status.login_open && status.message.contains("Log in again"));
    *product.auth.lock().unwrap() = regressee_app::auth::AuthOwner::with_test_vault(
        EncryptedSessions::with_provider(&vault_path, Box::new(FixtureKey))?,
    );
    require_login(
        product
            .probe_environment(&store, p.id, cancel.clone())
            .unwrap_err(),
    );
    assert_eq!(*staging.hits.lock().unwrap(), stage_visits);
    // Independently established staging remains usable after production drift.
    store.select_environment(p.id, stage.id, &Default::default())?;
    assert!(
        product
            .probe_environment(&store, p.id, cancel.clone())?
            .probed
    );
    let other = product.capture_inventory(
        &mut store,
        p.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(other.run.state, JobState::Completed);
    assert_eq!(
        store.session_reference(p.id, stage.id)?,
        Some((stage_scope.clone(), false))
    );
    // Explicit NEW consent/session identity at the changed production destination.
    store.select_environment(p.id, production_env, &Default::default())?;
    let new_session = SessionReference {
        id: SessionId::new(),
        ..prod_session.clone()
    };
    product
        .auth
        .lock()
        .unwrap()
        .save_fixture_session(&mut store, &new_session, &state, false)?;
    assert!(
        !vault_path
            .join(format!("{}.session", prod_session.id))
            .exists()
    );
    assert!(
        product
            .probe_environment(&store, p.id, cancel.clone())?
            .probed
    );
    let rebound = product.capture_inventory(
        &mut store,
        p.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(rebound.run.state, JobState::Completed);
    assert!(
        rebound.captures[0]
            .metadata
            .requested
            .starts_with(&staging.address)
    );
    assert_eq!(rebound.run.inventory, baseline.run.inventory);
    assert_eq!(store.load_project(p.id)?.project.active_baseline, active);
    assert_eq!(
        store.load_run(baseline.run.id)?.captures[0].image.original,
        before_hash
    );
    // Name-only edits preserve binding but reset the whole-settings probe receipt.
    let saved = store.load_project(p.id)?;
    let mut renamed = saved.project.clone();
    renamed.environments[0].name = "Production renamed".into();
    store.save_replacement_project(&renamed, saved.revision)?;
    let status = product.environment_status(&store, p.id)?;
    assert!(!status.probed && !status.message.contains("Log in again"));
    assert!(
        product
            .probe_environment(&store, p.id, cancel.clone())?
            .probed
    );
    // P2: real Product Forget with copied references, both storage modes and
    // wrong environment/project. No new runtime or public/native qualification.
    let preserved_run = serde_json::to_vec(&store.load_run(baseline.run.id)?.run)?;
    let preserved_baseline = serde_json::to_vec(&store.load_baseline(active.unwrap())?)?;
    let unrelated_files = [new_session.clone(), stage_scope.clone()]
        .into_iter()
        .map(|s| -> Result<_> {
            let file = vault_path.join(format!("{}.session", s.id));
            Ok((s, file.clone(), fs::read(file)?))
        })
        .collect::<Result<Vec<_>>>()?;
    let mut forget_records = Vec::new();
    for ephemeral in [false, true] {
        let mut a = Project::draft(
            "Forget fixture owner".into(),
            &format!("{}/preview/page", staging.address),
        )?;
        a.environments[0].access = AccessPolicy::InteractiveLogin;
        let mut b = a.environments[0].clone();
        b.id = EnvironmentId::new();
        b.name = "Copied reference".into();
        a.environments.push(b.clone());
        store.save_project(&a, None)?;
        let mut q = Project::draft(
            "Forget fixture foreign project".into(),
            &format!("{}/preview/page", staging.address),
        )?;
        q.environments[0].access = AccessPolicy::InteractiveLogin;
        store.save_project(&q, None)?;
        let a_scope = SessionReference {
            id: SessionId::new(),
            project: a.id,
            environment: a.environments[0].id,
        };
        product
            .auth
            .lock()
            .unwrap()
            .save_fixture_session(&mut store, &a_scope, &state, ephemeral)?;
        assert!(
            product
                .probe_environment(&store, a.id, cancel.clone())?
                .probed
        );
        let a_file = vault_path.join(format!("{}.session", a_scope.id));
        let original = (!ephemeral).then(|| fs::read(&a_file)).transpose()?;
        for (project, environment, variant) in [
            (a.id, b.id, "wrong-environment"),
            (q.id, q.environments[0].id, "wrong-project"),
        ] {
            let copied = SessionReference {
                id: a_scope.id,
                project,
                environment,
            };
            store.save_session_reference(&copied, ephemeral)?;
            store.select_environment(project, environment, &Default::default())?;
            let before_a = production.hits.lock().unwrap().clone();
            let before_b = staging.hits.lock().unwrap().clone();
            let replay_error = product
                .probe_environment(&store, project, cancel.clone())
                .unwrap_err();
            let replay_reason = replay_error.to_string();
            require_login(replay_error);
            let error = product.forget_login(&mut store, project).unwrap_err();
            let expected = if ephemeral {
                SecretError::ScopeMismatch
            } else {
                SecretError::Corrupt
            };
            assert_eq!(error.downcast_ref::<SecretError>(), Some(&expected));
            assert_eq!(error.to_string(), expected.to_string());
            assert_eq!(
                store.session_reference(project, environment)?,
                Some((copied, ephemeral))
            );
            assert_eq!(
                store.session_reference(a.id, a_scope.environment)?,
                Some((a_scope.clone(), ephemeral))
            );
            if let Some(ref original) = original {
                assert_eq!(&fs::read(&a_file)?, original);
            }
            assert_eq!(*production.hits.lock().unwrap(), before_a);
            assert_eq!(*staging.hits.lock().unwrap(), before_b);
            forget_records.push(serde_json::json!({"mode":if ephemeral {"ephemeral"} else {"encrypted-test-provider"},"variant":variant,"replay_code":"NeedsLogin","replay_reason":replay_reason,"forget_reason":error.to_string(),"owner_ciphertext_reference_and_use_preserved":true,"borrower_reference_retained":true,"production_hits_before":before_a,"production_hits_after":*production.hits.lock().unwrap(),"staging_hits_before":before_b,"staging_hits_after":*staging.hits.lock().unwrap()}));
        }
        store.select_environment(a.id, a_scope.environment, &Default::default())?;
        assert!(
            product
                .probe_environment(&store, a.id, cancel.clone())?
                .probed
        );
        // Actual server-side expired access is not a loss of immutable ownership.
        staging.state.store(3, Ordering::Release);
        let expired_error = product
            .probe_environment(&store, a.id, cancel.clone())
            .unwrap_err();
        staging.state.store(0, Ordering::Release);
        let expired_reason = expired_error.to_string();
        require_login(expired_error);
        let before_a = production.hits.lock().unwrap().clone();
        let before_b = staging.hits.lock().unwrap().clone();
        product.forget_login(&mut store, a.id)?;
        assert!(
            store
                .session_reference(a.id, a_scope.environment)?
                .is_none()
        );
        assert!(!a_file.exists());
        assert_eq!(*production.hits.lock().unwrap(), before_a);
        assert_eq!(*staging.hits.lock().unwrap(), before_b);
        // Fresh session, then origin/access drift: owner Forget still succeeds.
        let changed_scope = SessionReference {
            id: SessionId::new(),
            ..a_scope.clone()
        };
        product.auth.lock().unwrap().save_fixture_session(
            &mut store,
            &changed_scope,
            &state,
            ephemeral,
        )?;
        a.environments[0].mappings[0].destination =
            format!("{}/auth", production.address).parse()?;
        a.environments[0].access = AccessPolicy::Public;
        let revision = store.load_project(a.id)?.revision;
        store.save_replacement_project(&a, revision)?;
        let error = product
            .probe_environment(&store, a.id, cancel.clone())
            .unwrap_err();
        assert_eq!(
            error.to_string(),
            "Saved login does not match this environment's origins or access policy. Log in again before visits."
        );
        require_login(error);
        product.forget_login(&mut store, a.id)?;
        assert!(
            store
                .session_reference(a.id, a_scope.environment)?
                .is_none()
        );
        assert!(
            !vault_path
                .join(format!("{}.session", changed_scope.id))
                .exists()
        );
        assert_eq!(*production.hits.lock().unwrap(), before_a);
        assert_eq!(*staging.hits.lock().unwrap(), before_b);
        forget_records.push(serde_json::json!({"mode":if ephemeral {"ephemeral"} else {"encrypted-test-provider"},"owner_expired_forget":"success; no added visits","expired_reason":expired_reason,"owner_changed_origin_and_access_forget":"success; no added visits","production_hits_before":before_a,"production_hits_after":*production.hits.lock().unwrap(),"staging_hits_before":before_b,"staging_hits_after":*staging.hits.lock().unwrap()}));
    }
    assert_eq!(
        serde_json::to_vec(&store.load_run(baseline.run.id)?.run)?,
        preserved_run
    );
    assert_eq!(
        serde_json::to_vec(&store.load_baseline(active.unwrap())?)?,
        preserved_baseline
    );
    assert_eq!(store.load_project(p.id)?.project.active_baseline, active);
    assert_eq!(
        store.load_run(baseline.run.id)?.captures[0].image.original,
        before_hash
    );
    for (scope, file, bytes) in unrelated_files {
        assert_eq!(fs::read(file)?, bytes);
        assert_eq!(
            store.session_reference(scope.project, scope.environment)?,
            Some((scope, false))
        );
    }
    fs::write(
        output.join("forget-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"cases":forget_records,"unrelated_sessions_ciphertext_and_references":"identical","original_baseline_run_and_cas_reference":"identical","qualification":"implementer owned-loopback/test-provider; not OS/native/human/independent acceptance"}),
        )?,
    )?;
    store.select_environment(p.id, stage.id, &Default::default())?;
    // A valid-but-missing non-null local reference must fail before any HTTP visit.
    let missing = SessionReference {
        id: SessionId::new(),
        ..stage_scope.clone()
    };
    store.save_session_reference(&missing, false)?;
    let visits = staging.hits.lock().unwrap().clone();
    assert!(
        product
            .capture_inventory(
                &mut store,
                p.id,
                None,
                true,
                None,
                cancel.clone(),
                pause.clone(),
                &mut |_, _, _| {}
            )
            .is_err()
    );
    assert_eq!(*staging.hits.lock().unwrap(), visits);
    let project_json = serde_json::to_string(&store.load_project(p.id)?.project)?;
    assert!(!project_json.contains("session"));
    let evidence_json = serde_json::to_string(&store.project_evidence(p.id)?)?;
    assert!(
        !evidence_json.contains("fixture-session")
            && !evidence_json.contains(&prod_session.id.to_string())
            && !evidence_json.contains(&stage_scope.id.to_string())
    );
    fs::write(
        output.join("phase4-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"signed_worker":"4.0.0","scope":"DevelopmentLoopback","encrypted_test_provider":"explicit FixtureKey, not OS-store qualification","authenticated_baseline":baseline.run.id,"mapped_check":check.run.id,"restart_check":restarted.run.id,"expired_outcome":"NeedsLogin / Paused; zero login visits; no screenshot","resumed":recovered.run.id,"baseline_preserved":active,"origin_drift":"typed NeedsLogin before probe/discovery/check; zero destination visits/cookie leakage/anonymous fallback; restart still refuses","new_session_at_changed_origin":rebound.run.id,"other_environment_unchanged":other.run.id,"name_only_edit":"login preserved; probe reset","bad_reference":"missing encrypted state; zero visits; no anonymous fallback","logical_identity_preserved":true,"sessions_in_project_or_evidence":false,"native_human_and_enterprise_qualification":"pending"}),
        )?,
    )?;
    Ok(())
}
#[test]
#[ignore = "serial actual retained signed runtime smoke; no new pack or public-site capture"]
fn retained_runtime_relative_supervision_smoke() -> Result<()> {
    let data = PathBuf::from(std::env::var("REGRESSEE_PHASE2_RETAINED_FLOW")?);
    let d: Distribution = serde_json::from_reader(File::open(data.join("distribution.json"))?)?;
    assert_eq!(d.scope, Scope::DevelopmentLoopback);
    let cancel = Arc::new(AtomicBool::new(false));
    let mut installed =
        distribution::resolve(&data.join("app/cache/product-runtime"), &d, &cancel)?;
    // Cargo runs integration tests from the crate directory, not the shell's
    // repository root. Construct a relative path from the actual test cwd.
    let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let current = std::env::current_dir()?.canonicalize()?;
    let mut relative = PathBuf::new();
    for _ in current.strip_prefix(&repository)?.components() {
        relative.push("..");
    }
    relative.push(installed.root.canonicalize()?.strip_prefix(&repository)?);
    installed.root = relative;
    assert!(installed.root.is_relative());
    let mut worker = Worker::spawn(&installed)?;
    let (_, browser) = worker.handshake(RunId::new(), cancel)?;
    assert_eq!(browser, "141.0.7390.37");
    worker.stop(Instant::now() + Duration::from_secs(5))?;
    println!(
        "Current supervisor: actual signed retained product handshake/sandboxed screenshot/shutdown with a relative runtime root passed"
    );
    Ok(())
}

#[cfg(target_os = "linux")]
fn scratch_processes(scratch: &Path) -> Result<Vec<u32>> {
    let expected = format!("TMPDIR={}", scratch.display());
    let mut pids = Vec::new();
    for entry in fs::read_dir("/proc")?.flatten() {
        let Ok(pid) = entry.file_name().to_string_lossy().parse::<u32>() else {
            continue;
        };
        // Zombies have no executable/environment and are not live resources.
        if fs::read_link(entry.path().join("exe")).is_ok()
            && fs::read(entry.path().join("environ"))
                .is_ok_and(|env| env.split(|b| *b == 0).any(|v| v == expected.as_bytes()))
        {
            pids.push(pid);
        }
    }
    Ok(pids)
}

#[cfg(target_os = "linux")]
fn assert_scratch_gone(scratch: &Path) -> Result<()> {
    for _ in 0..100 {
        if scratch_processes(scratch)?.is_empty() && !scratch.exists() {
            return Ok(());
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    anyhow::bail!(
        "scratch or live owned processes remain: {} {:?}",
        scratch.display(),
        scratch_processes(scratch)?
    );
}

#[cfg(target_os = "linux")]
struct OwnedTestChild(std::process::Child);
#[cfg(target_os = "linux")]
impl std::ops::Deref for OwnedTestChild {
    type Target = std::process::Child;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[cfg(target_os = "linux")]
impl std::ops::DerefMut for OwnedTestChild {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
#[cfg(target_os = "linux")]
impl Drop for OwnedTestChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Each environment is inherited by a separate test process. No global env
/// mutations, no copied runtime, no HTTP pack download, no browser substitution.
#[cfg(target_os = "linux")]
#[test]
#[ignore = "serial sandboxed retained runtime; fresh REGRESSEE_PHASE2_CORRECTION_EVIDENCE required"]
fn retained_worker_long_and_short_tmpdir_regression() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let evidence = PathBuf::from(std::env::var("REGRESSEE_PHASE2_CORRECTION_EVIDENCE")?);
    fs::create_dir(&evidence)?;
    let base = if Path::new("/tmp/opencode").is_dir() {
        "/tmp/opencode"
    } else {
        "/tmp"
    };
    for case in ["long", "short"] {
        let output = evidence.join(case);
        fs::create_dir(&output)?;
        let namespace = tempfile::Builder::new()
            .prefix("t2-")
            .permissions(fs::Permissions::from_mode(0o700))
            .tempdir_in(base)?;
        let short = tempfile::Builder::new().prefix("ti-").tempdir_in(base)?;
        let inherited = if case == "long" {
            let p = output.join("inherited-".to_owned() + &"x".repeat(120));
            fs::create_dir(&p)?;
            assert!(p.as_os_str().as_encoded_bytes().len() > 108);
            p
        } else {
            short.path().to_path_buf()
        };
        // A legacy/unmarked entry in inherited TMPDIR must not be scanned/deleted.
        let untouched = inherited.join("rg2-unrelated");
        fs::create_dir(&untouched)?;
        fs::write(untouched.join("keep"), b"unrelated inherited evidence")?;
        let status = std::process::Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "worker_scratch_browser_child",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("TMPDIR", &inherited)
            .env("TMP", &inherited)
            .env("TEMP", &inherited)
            .env("REGRESSEE_WORKER_SCRATCH_PARENT", namespace.path())
            .env("REGRESSEE_SCRATCH_CASE_OUTPUT", &output)
            .env_remove("REGRESSEE_SCRATCH_CRASH_CHILD")
            .stdout(File::create(output.join("child.stdout"))?)
            .stderr(File::create(output.join("child.stderr"))?)
            .status()?;
        assert!(
            status.success(),
            "{case} child failed: inspect {}",
            output.display()
        );
        assert_eq!(
            fs::read(untouched.join("keep"))?,
            b"unrelated inherited evidence"
        );
        assert_eq!(
            fs::read_dir(namespace.path())?.count(),
            0,
            "owned namespace not empty"
        );
    }
    Ok(())
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "subprocess helper for retained_worker_long_and_short_tmpdir_regression"]
fn worker_scratch_browser_child() -> Result<()> {
    let data = PathBuf::from(std::env::var("REGRESSEE_PHASE2_RETAINED_FLOW")?);
    let d: Distribution = serde_json::from_reader(File::open(data.join("distribution.json"))?)?;
    assert_eq!(d.scope, Scope::DevelopmentLoopback);
    let cancel = Arc::new(AtomicBool::new(false));
    let installed = distribution::resolve(&data.join("app/cache/product-runtime"), &d, &cancel)?;
    let output = PathBuf::from(std::env::var("REGRESSEE_SCRATCH_CASE_OUTPUT")?);
    let parent = PathBuf::from(std::env::var("REGRESSEE_WORKER_SCRATCH_PARENT")?);
    let mut guard = Worker::spawn(&installed)?;
    let guard_scratch = guard
        .output_path(uuid::Uuid::new_v4())
        .parent()
        .unwrap()
        .to_path_buf();
    assert_eq!(guard_scratch.parent(), Some(parent.as_path()));
    assert!(guard_scratch.as_os_str().as_encoded_bytes().len() <= 49);
    assert_ne!(
        guard_scratch.parent(),
        Some(Path::new(&std::env::var("TMPDIR")?))
    );
    let (_, browser) = guard.handshake(RunId::new(), cancel.clone())?;
    assert_eq!(browser, "141.0.7390.37");
    assert!(
        scratch_processes(&guard_scratch)?.len() >= 2,
        "actual browser not observed"
    );
    if std::env::var_os("REGRESSEE_SCRATCH_CRASH_CHILD").is_some() {
        fs::write(
            output.join("crash-ready.json"),
            serde_json::to_vec(
                &serde_json::json!({"scratch":guard_scratch,"worker":guard.pid(),"live_pids":scratch_processes(&guard_scratch)?}),
            )?,
        )?;
        // Intentionally retain a live real browser until this app parent is killed.
        loop {
            std::thread::sleep(Duration::from_millis(50));
        }
    }
    let site = Site::start(output.join("unused-pack"))?;
    let mut profile = CaptureProfile::desktop();
    profile.runtime = installed.manifest.identity.clone();
    profile.user_agent = "Regressee owned scratch regression".into();
    let request = |address: String| Request {
        version: 2,
        request_id: 2,
        job: RunId::new(),
        command: Command::CaptureTarget {
            key: TargetKey {
                page: PageId::new(),
                profile: profile.id,
            },
            address,
            profile: Box::new(profile.clone()),
            session: None,
        },
    };
    let mut captured = None;
    guard.request(
        request(format!("{}/page", site.address)),
        Instant::now() + Duration::from_secs(30),
        cancel.clone(),
        &mut |event| {
            if let EventKind::TargetCaptured {
                output_token,
                width,
                height,
                ..
            } = event.event
            {
                assert_eq!((width, height), (1440, 900));
                captured = Some(output_token);
            }
            Ok(())
        },
    )?;
    let token = captured.context("real screenshot result required")?;
    let png = fs::read(guard.output_path(token))?;
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    fs::write(output.join("capture.png"), &png)?;
    fs::copy(guard.metadata_path(token), output.join("capture.json"))?;
    // Real in-flight capture cancellation. A second live owner/browser MUST stay.
    let mut worker = Worker::spawn(&installed)?;
    worker.handshake(RunId::new(), cancel.clone())?;
    let scratch = worker
        .output_path(uuid::Uuid::new_v4())
        .parent()
        .unwrap()
        .to_path_buf();
    let trigger = cancel.clone();
    let mut killer = None;
    let err = worker
        .request(
            request(format!("{}/hang", site.address)),
            Instant::now() + Duration::from_secs(15),
            cancel.clone(),
            &mut |event| {
                if matches!(event.event, EventKind::Progress { .. }) && killer.is_none() {
                    let trigger = trigger.clone();
                    killer = Some(std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(200));
                        trigger.store(true, Ordering::Release);
                    }));
                }
                Ok(())
            },
        )
        .unwrap_err();
    killer
        .context("capture progress before cancellation")?
        .join()
        .unwrap();
    assert_eq!(
        err.downcast_ref::<WorkerFailure>().unwrap().0,
        FailureCode::Cancelled
    );
    assert_scratch_gone(&scratch)?;
    drop(worker);
    cancel.store(false, Ordering::Release);
    assert!(
        scratch_processes(&guard_scratch)?.len() >= 2,
        "live owner was killed by cancellation/recovery"
    );
    // Also exercise abrupt worker death without re-running the HTTP pack suite.
    let mut worker = Worker::spawn(&installed)?;
    worker.handshake(RunId::new(), cancel.clone())?;
    let scratch = worker
        .output_path(uuid::Uuid::new_v4())
        .parent()
        .unwrap()
        .to_path_buf();
    let pid = worker.pid();
    let mut killer = None;
    let err = worker
        .request(
            request(format!("{}/hang", site.address)),
            Instant::now() + Duration::from_secs(15),
            cancel.clone(),
            &mut |event| {
                if matches!(event.event, EventKind::Progress { .. }) && killer.is_none() {
                    killer = Some(std::thread::spawn(move || {
                        std::thread::sleep(Duration::from_millis(200));
                        // This still-unreaped child PID belongs to this test's Worker.
                        unsafe {
                            libc::kill(pid as i32, libc::SIGKILL);
                        }
                    }));
                }
                Ok(())
            },
        )
        .unwrap_err();
    killer
        .context("capture progress before worker crash")?
        .join()
        .unwrap();
    assert_eq!(
        err.downcast_ref::<WorkerFailure>().unwrap().0,
        FailureCode::WorkerStopped
    );
    drop(worker);
    assert_scratch_gone(&scratch)?;
    // A process carrying the scratch env but NOT a runtime executable is unowned.
    let mut unrelated = OwnedTestChild(
        std::process::Command::new("/bin/sleep")
            .arg("120")
            .env("TMPDIR", &guard_scratch)
            .spawn()?,
    );
    // Kill an actual separate app parent AFTER its sandboxed browser is live.
    let mut app = OwnedTestChild(
        std::process::Command::new(std::env::current_exe()?)
            .args([
                "--exact",
                "worker_scratch_browser_child",
                "--ignored",
                "--nocapture",
                "--test-threads=1",
            ])
            .env("REGRESSEE_SCRATCH_CRASH_CHILD", "1")
            .stdout(File::create(output.join("crash.stdout"))?)
            .stderr(File::create(output.join("crash.stderr"))?)
            .spawn()?,
    );
    let ready = output.join("crash-ready.json");
    let deadline = Instant::now() + Duration::from_secs(90);
    while !ready.exists() {
        if Instant::now() >= deadline || app.try_wait()?.is_some() {
            let _ = app.kill();
            let _ = app.wait();
            let _ = unrelated.kill();
            let _ = unrelated.wait();
            anyhow::bail!("crash child did not establish a real browser");
        }
        std::thread::sleep(Duration::from_millis(20));
    }
    let dead: serde_json::Value = serde_json::from_slice(&fs::read(&ready)?)?;
    let dead_scratch = PathBuf::from(dead["scratch"].as_str().unwrap());
    assert!(dead["live_pids"].as_array().unwrap().len() >= 2);
    assert!(dead_scratch.exists());
    app.kill()?;
    app.wait()?;
    // Worker::spawn executes the same parent/marker policy used by cancellation.
    let mut recovered = Worker::spawn(&installed)?;
    assert_scratch_gone(&dead_scratch)?;
    assert!(
        scratch_processes(&guard_scratch)?.len() >= 3,
        "live owner or unrelated process was touched"
    );
    assert!(unrelated.try_wait()?.is_none());
    recovered.handshake(RunId::new(), cancel)?;
    let recovered_scratch = recovered
        .output_path(uuid::Uuid::new_v4())
        .parent()
        .unwrap()
        .to_path_buf();
    recovered.stop(Instant::now() + Duration::from_secs(5))?;
    assert_scratch_gone(&recovered_scratch)?;
    drop(recovered);
    // Prove the OTHER live browser still captures, not merely that a Node or
    // crashpad PID survived the cancellation and both crash-cleanup paths.
    let mut again = request(format!("{}/page", site.address));
    again.request_id = 3;
    let mut repeat = None;
    guard.request(
        again,
        Instant::now() + Duration::from_secs(30),
        Arc::new(AtomicBool::new(false)),
        &mut |event| {
            if let EventKind::TargetCaptured {
                output_token,
                width,
                height,
                ..
            } = event.event
            {
                assert_eq!((width, height), (1440, 900));
                repeat = Some(output_token);
            }
            Ok(())
        },
    )?;
    let repeated_png = fs::read(
        guard.output_path(repeat.context("live owner must still capture after cleanup")?),
    )?;
    assert_eq!(repeated_png, png);
    fs::write(output.join("live-owner-after-cleanup.png"), repeated_png)?;
    // Stop must not kill a non-browser even though it has the same scratch env.
    guard.stop(Instant::now() + Duration::from_secs(5))?;
    assert!(unrelated.try_wait()?.is_none());
    unrelated.kill()?;
    unrelated.wait()?;
    assert_scratch_gone(&guard_scratch)?;
    assert_eq!(fs::read_dir(&parent)?.count(), 0);
    fs::write(
        output.join("result.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"inherited_tmpdir":std::env::var("TMPDIR")?,"scratch_parent":parent,"max_scratch_bytes":49,"sandboxed_browser":browser,"capture_bytes":png.len(),"cancel":"Cancelled; scratch/processes gone","worker_sigkill":"WorkerStopped; drop reaped and removed scratch","app_sigkill":"live-browser parent killed/reaped; same-namespace recovery","unrelated_and_live_owner":"preserved; other browser recaptured identical PNG after cleanup","final_namespace_entries":0}),
        )?,
    )?;
    Ok(())
}

#[test]
#[ignore = "serial private runtime/browser integration; set REGRESSEE_PHASE2_EVIDENCE to a fresh retained directory"]
fn actual_signed_http_product_flow() -> Result<()> {
    let output = PathBuf::from(
        std::env::var("REGRESSEE_PHASE2_EVIDENCE").context("fresh evidence directory required")?,
    );
    fs::create_dir(&output)?;
    let source = std::env::var_os("REGRESSEE_TEST_SOURCE_RUNTIME")
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.runtime/installed")
        });
    let pack = output.join("development-product-runtime.zip");
    let key = development_pack(&source, &pack)?;
    let site = Site::start(pack)?;
    let paths = AppPaths::isolated(&output.join("app"));
    let mut store = Store::open(paths.clone())?;
    let d = Distribution {
        version: 1,
        url: format!("{}/runtime.zip", site.address).parse()?,
        public_key: hex::encode(key.verifying_key().to_bytes()),
        scope: Scope::DevelopmentLoopback,
    };
    // Retain the public trust configuration only. Ephemeral signing key is not saved.
    fs::write(
        output.join("distribution.json"),
        serde_json::to_vec_pretty(&d)?,
    )?;
    let mut product = Product {
        distribution: Some(d.clone()),
        runtime_cache: paths.cache.join("product-runtime"),
        official_runtime: false,
        auth: Default::default(),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let start = Instant::now();
    let mut downloaded = 0;
    assert!(
        product
            .setup(cancel.clone(), &mut |bytes, _| downloaded = bytes)?
            .contains("Development")
    );
    assert!(downloaded > 0);
    let installed = distribution::resolve(&product.runtime_cache, &d, &cancel)?;
    let suite = std::process::Command::new(installed.root.join("node/bin/node"))
        .args(["--test"])
        .arg(installed.root.join("worker/test/protocol.test.mjs"))
        .arg(installed.root.join("worker/test/capture.test.mjs"))
        .output()?;
    fs::write(
        output.join("private-worker-tests.log"),
        [suite.stdout, suite.stderr].concat(),
    )?;
    assert!(suite.status.success());
    let p = fresh_project(&mut store, &site, "/page")?;
    // Exact bounded actor actions used by GPUI, not fabricated completion data.
    use regressee_app::jobs::{Jobs, Reply, Request as JobRequest, Work};
    let (jobs, updates) = Jobs::start_with_distribution(paths.clone(), d.clone());
    jobs.submit(JobRequest {
        id: 1,
        work: Work::Capture {
            project: p.id,
            check: false,
        },
    })
    .unwrap();
    let baseline = loop {
        let u = updates.recv_timeout(Duration::from_secs(60))?;
        assert_eq!(u.id, 1);
        match u.reply {
            Reply::Progress { .. } => (),
            Reply::Evidence(e) => break e.latest.unwrap(),
            r => panic!("actor capture: {r:?}"),
        }
    };
    assert_eq!(baseline.run.state, JobState::Completed);
    assert!(baseline.capture.is_some());
    assert!(store.load_project(p.id)?.project.active_baseline.is_none());
    jobs.submit(JobRequest {
        id: 2,
        work: Work::Activate {
            run: baseline.run.id,
            revision: 1,
        },
    })
    .unwrap();
    let saved = match updates.recv_timeout(Duration::from_secs(10))?.reply {
        Reply::Project(p) => *p,
        r => panic!("actor activation: {r:?}"),
    };
    jobs.join_after_ui();
    let active = saved.project.active_baseline.unwrap();
    let before = baseline.capture.as_ref().unwrap().image.original.clone();
    assert!(!store.collect_unreferenced(&before)?);
    let unchanged = product.capture(&mut store, p.id, true, cancel.clone(), &mut |_, _, _| {})?;
    assert_ne!(unchanged.run.id, baseline.run.id);
    let evidence = store.project_evidence(p.id)?;
    let unchanged_metrics = evidence.comparison.unwrap().revision.results[0].clone();
    assert_eq!(
        unchanged_metrics.classification,
        Classification::NoSignificantChange
    );
    assert_eq!(unchanged_metrics.changed_pixels, 0);
    site.state.store(1, Ordering::Release);
    let changed = product.capture(&mut store, p.id, true, cancel.clone(), &mut |_, _, _| {})?;
    assert_eq!(changed.run.state, JobState::Completed);
    let evidence = store.project_evidence(p.id)?;
    let changed_metrics = evidence.comparison.as_ref().unwrap().revision.results[0].clone();
    assert_eq!(changed_metrics.classification, Classification::Changed);
    assert!(changed_metrics.changed_pixels >= 6400);
    assert!(evidence.comparison.unwrap().heatmap.is_some());
    assert_eq!(
        store.load_project(p.id)?.project.active_baseline,
        Some(active)
    );
    assert!(
        product
            .activate(&mut store, changed.run.id, saved.revision)
            .is_err(),
        "comparison must not be activated as new baseline"
    );
    // Failed replacement cannot accidentally change the active baseline; reject
    // a failure from a separate project too, preserving the approved reference.
    let f = fresh_project(&mut store, &site, "/failure")?;
    let failed = product.capture(&mut store, f.id, false, cancel.clone(), &mut |_, _, _| {})?;
    assert_eq!(failed.run.state, JobState::CompletedWithErrors);
    assert_eq!(
        failed.run.targets[0].outcome,
        TargetOutcome::Failed {
            code: CaptureError::NotFound
        }
    );
    assert!(product.activate(&mut store, failed.run.id, 1).is_err());
    assert!(store.load_project(f.id)?.project.active_baseline.is_none());
    // Fail a check of the same fixed /page URL, rather than swapping its address.
    let fail_site = site.state.clone();
    fail_site.store(2, Ordering::Release);
    // State 2 is handled below by the fixture's HTTP response.
    let failed_check =
        product.capture(&mut store, p.id, true, cancel.clone(), &mut |_, _, _| {})?;
    assert_eq!(failed_check.run.state, JobState::CompletedWithErrors);
    assert_eq!(
        store
            .project_evidence(p.id)?
            .comparison
            .unwrap()
            .revision
            .results[0]
            .classification,
        Classification::CouldNotCapture
    );
    assert_eq!(
        store.load_project(p.id)?.project.active_baseline,
        Some(active)
    );
    site.state.store(1, Ordering::Release);
    let tall = fresh_project(&mut store, &site, "/tall")?;
    let t = product.capture(
        &mut store,
        tall.id,
        false,
        cancel.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(t.run.state, JobState::Completed);
    let ti = &t.capture.as_ref().unwrap().image;
    assert_eq!(ti.height, 20000);
    assert_eq!(ti.tiles.len(), 40);
    #[cfg(feature = "ui")]
    {
        let texture = regressee_app::images::viewer_tile(&store, &ti.tiles[39])?;
        assert!(texture.size(0).height.0 <= 512);
    }
    let h = fresh_project(&mut store, &site, "/hang")?;
    let trigger = cancel.clone();
    let mut cancel_thread = None;
    let c = product.capture(
        &mut store,
        h.id,
        false,
        cancel.clone(),
        &mut |message, _, _| {
            if message == "Capturing the fixed page…" && cancel_thread.is_none() {
                let trigger = trigger.clone();
                cancel_thread = Some(std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(200));
                    trigger.store(true, Ordering::Release);
                }));
            }
        },
    )?;
    cancel_thread.unwrap().join().unwrap();
    assert_eq!(c.run.state, JobState::Cancelled);
    assert!(product.activate(&mut store, c.run.id, 1).is_err());
    cancel.store(false, Ordering::Release);
    // Kill the actual headless product application after its persisted Running
    // row exists. Recovery marks only a dead owner's run interrupted, keeping all
    // previously completed captures and the active reference untouched.
    let binary =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/regressee-flow");
    let mut app = std::process::Command::new(binary)
        .arg(output.join("app"))
        .arg("capture")
        .arg(h.id.to_string())
        .env("REGRESSEE_DISTRIBUTION", output.join("distribution.json"))
        .stdout(File::create(output.join("crashed-app.stdout"))?)
        .stderr(File::create(output.join("crashed-app.stderr"))?)
        .spawn()?;
    let pending = loop {
        if let Some(r) = store.project_evidence(h.id)?.latest
            && r.run.id != c.run.id
            && r.run.state == JobState::Running
        {
            break r;
        }
        anyhow::ensure!(app.try_wait()?.is_none(), "app exited before crash fixture");
        std::thread::sleep(Duration::from_millis(20));
    };
    assert_eq!(
        store.recover_runs()?,
        0,
        "live run owner must not be interrupted"
    );
    app.kill()?;
    app.wait()?;
    assert_eq!(store.recover_runs()?, 1);
    assert_eq!(
        store.load_run(pending.run.id)?.run.state,
        JobState::Interrupted
    );
    assert!(product.activate(&mut store, pending.run.id, 1).is_err());
    store.verify_artifact(&before)?;
    // Crash the actual product process, not an added fixture-only worker command.
    let mut worker = Worker::spawn(&installed)?;
    let job = RunId::new();
    worker.handshake(job, cancel.clone())?;
    let pid = worker.pid();
    let killer = std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(500));
        kill_worker(pid)
    });
    let profile = baseline.run.profiles[0].clone();
    let crash = worker
        .request(
            Request {
                version: 2,
                request_id: 2,
                job,
                command: Command::CaptureTarget {
                    key: baseline.run.targets[0].key.clone(),
                    address: format!("{}/hang", site.address),
                    profile: Box::new(profile),
                    session: None,
                },
            },
            Instant::now() + Duration::from_secs(10),
            cancel.clone(),
            &mut |_| Ok(()),
        )
        .unwrap_err();
    killer.join().unwrap()?;
    assert_eq!(
        crash.downcast_ref::<WorkerFailure>().unwrap().0,
        FailureCode::WorkerStopped
    );
    drop(worker);
    // A repaired signed HTTP generation replaces only the cache pointer.
    let pointer = fs::read(product.runtime_cache.join("current"))?;
    product.distribution.as_mut().unwrap().url = format!("{}/corrupt.zip", site.address).parse()?;
    assert!(product.setup(cancel.clone(), &mut |_, _| {}).is_err());
    assert_eq!(fs::read(product.runtime_cache.join("current"))?, pointer);
    product.distribution.as_mut().unwrap().url =
        format!("{}/redirect.zip", site.address).parse()?;
    assert!(
        product
            .setup(cancel.clone(), &mut |_, _| {})
            .unwrap_err()
            .to_string()
            .contains("redirects")
    );
    assert_eq!(fs::read(product.runtime_cache.join("current"))?, pointer);
    product.distribution = Some(d.clone());
    fs::write(
        installed.root.join("worker/src/protocol.mjs"),
        b"damaged cache",
    )?;
    assert!(product.status(&cancel).is_err());
    product.setup(cancel.clone(), &mut |_, _| {})?;
    assert!(installed.root.exists());
    let public = Project::draft("Public site refused".into(), "https://example.com/")?;
    store.save_project(&public, None)?;
    assert!(
        product
            .capture(
                &mut store,
                public.id,
                false,
                cancel.clone(),
                &mut |_, _, _| {}
            )
            .unwrap_err()
            .to_string()
            .contains("loopback")
    );
    drop(store);
    let reopened = Store::open(paths)?;
    assert_eq!(
        reopened.load_project(p.id)?.project.active_baseline,
        Some(active)
    );
    reopened.verify_artifact(&before)?;
    assert_eq!(
        reopened.load_run(changed.run.id)?.run.state,
        JobState::Completed
    );
    fs::write(
        output.join("flow-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"scope":"Actual product actor/services and sandboxed private worker; development owned-loopback only","project":p.id,"signed_http_download_bytes":downloaded,"duration_seconds":start.elapsed().as_secs_f64(),"unchanged":unchanged_metrics,"changed":changed_metrics,"tall_tiles":40,"failure_check":"CouldNotCapture","cancel":"preserved/no activation","crash":"WorkerStopped/reaped","app_crash_restart":"dead-owner run interrupted; prior successful evidence retained","corrupt_and_redirect_download":"old pointer preserved","repair":"new generation; old retained","restart":"baseline and originals reopen","external_public_runtime_qualification":"PENDING"}),
        )?,
    )?;
    Ok(())
}

struct CrawlSite {
    address: String,
    state: Arc<AtomicU32>,
    hits: Arc<std::sync::Mutex<BTreeMap<String, u32>>>,
    stop: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
#[test]
#[ignore = "Real 1000 distinct owned pages x 3 profiles; explicit budget/runtime/evidence required"]
fn phase7_actual_1000_page_soak_pause_resume_cancel() -> Result<()> {
    use regressee_core::crawl::TargetCounts;
    let output = PathBuf::from(std::env::var("REGRESSEE_PHASE7_EVIDENCE")?);
    fs::create_dir(&output)?;
    let distribution: Distribution =
        serde_json::from_reader(File::open(std::env::var("REGRESSEE_PHASE7_DISTRIBUTION")?)?)?;
    ensure!(
        distribution.scope == Scope::DevelopmentLoopback,
        "Soak must remain owned loopback only"
    );
    let product = Product {
        distribution: Some(distribution),
        runtime_cache: PathBuf::from(std::env::var("REGRESSEE_PHASE7_RUNTIME_CACHE")?),
        official_runtime: false,
        auth: Default::default(),
    };
    let site = Site::start(PathBuf::from("unused"))?;
    let mut store = Store::open(AppPaths::isolated(&output.join("app")))?;
    let mut project = Project::draft(
        "Owned 1000-page Phase 7 soak".into(),
        &format!("{}/soak/0", site.address),
    )?;
    project.sources = (0..1000)
        .map(|i| format!("{}/soak/{i}", site.address).parse().unwrap())
        .collect();
    project.profiles = vec![
        CaptureProfile::desktop(),
        CaptureProfile::tablet(),
        CaptureProfile::mobile(),
    ];
    project.discovery.mode = DiscoveryMode::OnlyAddresses;
    project.discovery.speed = Speed::Fast;
    project.discovery.advanced.per_origin = 4;
    project.discovery.page_limit = 1000;
    store.save_project(&project, None)?;
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    let preview = product.discover(&store, project.id, cancel.clone(), &mut |_, _, _| {})?;
    assert_eq!(preview.included(), 1000);
    let start = Instant::now();
    let mut intervals = Vec::new();
    let mut last = Instant::now();
    let before = store.summary()?.artifact_bytes;
    let mut checkpoint = File::options()
        .create_new(true)
        .write(true)
        .open(output.join("progress.jsonl"))?;
    let mut budget = |message: &str, done: u32, total: Option<u32>| {
        intervals.push(last.elapsed().as_secs_f64());
        last = Instant::now();
        if done.is_multiple_of(100) {
            let _ = writeln!(
                checkpoint,
                "{}",
                serde_json::json!({"elapsed_seconds":start.elapsed().as_secs_f64(),"completed":done,"total":total,"phase":message})
            );
            let _ = checkpoint.flush();
        }
        if start.elapsed() > Duration::from_secs(1200) {
            cancel.store(true, Ordering::Release);
        }
        if done >= 1000 {
            pause.store(true, Ordering::Release);
        }
    };
    let paused = product.capture_inventory(
        &mut store,
        project.id,
        Some(preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut budget,
    )?;
    fs::write(
        output.join("paused.json"),
        serde_json::to_vec_pretty(&paused)?,
    )?;
    ensure!(
        paused.run.state == JobState::Paused,
        "Soak budget stopped before intended pause; retained partial evidence"
    );
    let retained: Vec<_> = paused
        .captures
        .iter()
        .map(|c| (c.image.original.clone(), c.metadata.clone()))
        .collect();
    let paths = store.paths().clone();
    drop(store);
    let mut store = Store::open(paths)?;
    store.recover_runs()?;
    pause.store(false, Ordering::Release);
    let mut last = Instant::now();
    let completed=product.capture_inventory(&mut store,project.id,None,false,Some(paused.run.id),cancel.clone(),pause.clone(),&mut |message,done,total| {
        intervals.push(last.elapsed().as_secs_f64());last=Instant::now();
        if done.is_multiple_of(100) {let _=writeln!(checkpoint,"{}",serde_json::json!({"elapsed_seconds":start.elapsed().as_secs_f64(),"completed":done,"total":total,"phase":message}));}
        if start.elapsed()>Duration::from_secs(1200) {cancel.store(true,Ordering::Release);}
    })?;
    fs::write(
        output.join("completed.json"),
        serde_json::to_vec_pretty(&completed)?,
    )?;
    let count = TargetCounts::from_run(&completed.run);
    let visits: u32 = site
        .hits
        .lock()
        .unwrap()
        .iter()
        .filter(|(p, _)| p.starts_with("/soak/"))
        .map(|(_, n)| n)
        .sum();
    fs::write(
        output.join("coverage.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"intended":3000,"captured":completed.captures.len(),"visits":visits,"elapsed_seconds":start.elapsed().as_secs_f64(),"counts":count}),
        )?,
    )?;
    ensure!(
        completed.run.state == JobState::Completed
            && completed.captures.len() == 3000
            && visits == 3000,
        "Soak partial/failed coverage; no activation"
    );
    for (id, metadata) in &retained {
        let saved = completed
            .captures
            .iter()
            .find(|c| c.image.original == *id)
            .context("Resumed original missing")?;
        assert_eq!(
            serde_json::to_vec(&saved.metadata)?,
            serde_json::to_vec(metadata)?
        );
    }
    for c in &completed.captures {
        store.verify_artifact(&c.image.original)?;
    }
    assert!(completed.run.targets.iter().all(|t| t.attempts == 1));
    assert_eq!(
        site.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|(p, _)| p.starts_with("/soak/"))
            .count(),
        1000
    );
    assert!(
        site.hits
            .lock()
            .unwrap()
            .iter()
            .filter(|(p, _)| p.starts_with("/soak/"))
            .all(|(_, n)| *n == 3)
    );
    let saved = product.activate(&mut store, completed.run.id, 1)?;
    let baseline = store.load_baseline(saved.project.active_baseline.unwrap())?;
    let baseline_bytes = serde_json::to_vec(&baseline)?;
    let capture_wall = start.elapsed().as_secs_f64();
    // Cancel another large fixed inventory after >=1000 real persisted captures;
    // do not count queue generation or erase the completed immutable baseline.
    let mut inventory = completed.run.inventory.clone();
    inventory.id = InventoryId::new();
    let preview = regressee_core::crawl::DiscoveryPreview {
        project_revision: saved.revision,
        settings: saved.project,
        inventory,
        aliases: vec![],
        notes: vec![],
        visited: 0,
    };
    let mut cancel_requested = None;
    let cancelled = product.capture_inventory(
        &mut store,
        project.id,
        Some(preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, done, _| {
            if done >= 1000 || start.elapsed() > Duration::from_secs(1500) {
                cancel_requested.get_or_insert_with(Instant::now);
                cancel.store(true, Ordering::Release);
            }
        },
    )?;
    let cancel_latency = cancel_requested
        .context("large cancellation was never requested")?
        .elapsed()
        .as_secs_f64();
    assert_eq!(cancelled.run.state, JobState::Cancelled);
    assert!(cancelled.captures.len() >= 1000);
    assert!(product.activate(&mut store, cancelled.run.id, 2).is_err());
    assert_eq!(
        serde_json::to_vec(&store.load_baseline(baseline.id)?)?,
        baseline_bytes
    );
    assert_eq!(
        store.load_project(project.id)?.project.active_baseline,
        Some(baseline.id)
    );
    intervals.sort_by(f64::total_cmp);
    let percentile = |p: f64| intervals[((intervals.len() - 1) as f64 * p) as usize];
    fs::write(
        output.join("soak-report.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
        "scope":"actual owned numeric-loopback fixture; fresh fixture-signed worker5 on authenticated historical Node/browser, NOT public security qualification",
        "pages":1000,"profiles":3,"successful_persisted_targets":3000,"successful_visits":visits,
        "scheduler":"Fast, four per origin; Gentle/Balanced bounded contracts are unit-tested, not separate maximum-scale runs",
        "pause_retained_targets":retained.len(),"resume":"same frozen run, original hashes/metadata preserved after reopened Store",
        "cancelled_large_run_successful_targets":cancelled.captures.len(),"cancel_counts":TargetCounts::from_run(&cancelled.run),
        "cancel_request_to_settled_return_seconds":cancel_latency,
        "capture_wall_seconds":capture_wall,"total_wall_seconds":start.elapsed().as_secs_f64(),"artifact_growth_bytes":store.summary()?.artifact_bytes-before,
        "retry_count":0,"capture_boundary_interval_p50_seconds":percentile(0.5),"capture_boundary_interval_p95_seconds":percentile(0.95),
        "native_input_latency":"NOT MEASURED","originals_integrity":"all successful CAS hashes verified; no baseline overwritten"}))?,
    )?;
    Ok(())
}
impl CrawlSite {
    fn start() -> Result<Self> {
        Self::with_cross(None)
    }
    fn with_cross(cross: Option<String>) -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let address = format!("http://{}", listener.local_addr()?);
        let state = Arc::new(AtomicU32::new(0));
        let stop = Arc::new(AtomicBool::new(false));
        let hits = Arc::new(std::sync::Mutex::new(BTreeMap::<String, u32>::new()));
        let shutdown = stop.clone();
        let phase = state.clone();
        let visits = hits.clone();
        let thread = std::thread::spawn(move || {
            while !shutdown.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut stream, _)) => {
                        let _ = stream.set_read_timeout(Some(Duration::from_secs(2)));
                        let _ = stream.set_write_timeout(Some(Duration::from_secs(2)));
                        let mut bytes = [0; 8192];
                        let n = stream.read(&mut bytes).unwrap_or(0);
                        if n == 0 {
                            continue;
                        } // A closed/preconnected socket is not an HTTP request.
                        let request = String::from_utf8_lossy(&bytes[..n]);
                        let url = request
                            .lines()
                            .next()
                            .and_then(|s| s.split_whitespace().nth(1))
                            .unwrap_or("")
                            .to_owned();
                        let count = {
                            let mut hits = visits.lock().unwrap();
                            let count = hits.entry(url.clone()).or_default();
                            *count += 1;
                            *count
                        };
                        if url == "/slow" {
                            for _ in 0..250 {
                                if shutdown.load(Ordering::Acquire) {
                                    break;
                                }
                                std::thread::sleep(Duration::from_millis(20));
                            }
                        }
                        if url == "/robots.txt" {
                            let _=respond(&mut stream,200,b"User-agent: *\nDisallow: /docs/private\nAllow: /docs/private/ok\nDisallow: /docs/robot\nDisallow: /scope/private\nDisallow: /scope/wild/*$\n");
                            continue;
                        }
                        if url.starts_with("/scope") {
                            let destination = match url.as_str() {
                                "/scope/redirect-section" => Some("/outside".to_owned()),
                                "/scope/redirect-robots" => Some("/scope/private".to_owned()),
                                "/scope/redirect-wild" => Some("/scope/wild/secret".to_owned()),
                                "/scope/to-danger" => Some("/scope/delete".to_owned()),
                                "/scope/to-omit" => Some("/scope/skipped".to_owned()),
                                "/scope/redirect-download" => Some("/scope/file.pdf".to_owned()),
                                "/scope/redirect-cross" => {
                                    cross.as_ref().map(|u| format!("{u}/identity-provider"))
                                }
                                "/scope/approved-cross" => {
                                    cross.as_ref().map(|u| format!("{u}/scope/cross-allowed"))
                                }
                                "/scope/chain-one" => Some("/scope/chain-two".to_owned()),
                                "/scope/chain-two" => Some("/scope/allowed?b=2&a=1&a=0".to_owned()),
                                "/scope/loop-one" => Some("/scope/loop-two".to_owned()),
                                "/scope/loop-two" => Some("/scope/loop-one".to_owned()),
                                "/scope/loop-self" => Some("/scope/loop-self".to_owned()),
                                "/scope/stable" if phase.load(Ordering::Acquire) == 1 => {
                                    Some("/scope/private".to_owned())
                                }
                                _ => None,
                            };
                            if let Some(to) = destination {
                                let _ = write!(
                                    stream,
                                    "HTTP/1.1 302 Found\r\nLocation: {to}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                                );
                                continue;
                            }
                            let html = match url.as_str() {
                                 "/scope" => ["redirect-section", "redirect-robots", "redirect-wild", "to-danger", "to-omit", "redirect-download", "redirect-cross", "chain-one", "loop-one", "loop-self", "wild/secret", "stable", "client", "meta", "form", "frame"].iter().map(|p| format!("<a href='/scope/{p}'>{p}</a>")).collect::<String>(),
                                 "/scope/client" => "<script>location.replace('/scope/private')</script><a href='/scope/poison'>poison</a>".into(),
                                 "/scope/meta" => "<meta http-equiv=refresh content='0;url=/outside'><a href='/scope/poison'>poison</a>".into(),
                                 "/scope/form" => "<form method=post action='/scope/form'><input name=x value=y></form><script>document.forms[0].submit()</script><a href='/scope/poison'>poison</a>".into(),
                                 "/scope/frame" => format!("<iframe src='{}/identity-provider'></iframe><a href='/scope/poison'>poison</a>", cross.as_deref().unwrap_or("")),
                                 _ => "<h1>Allowed owned fixture</h1>".into(),
                             };
                            let _ = respond(
                                &mut stream,
                                200,
                                format!("<!doctype html><meta charset=utf-8>{html}").as_bytes(),
                            );
                            continue;
                        }
                        if url == "/docs/alias" {
                            let _=stream.write_all(b"HTTP/1.1 302 Found\r\nLocation: /docs/a\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                            continue;
                        }
                        if url == "/docs/loop-alias" {
                            let _=stream.write_all(b"HTTP/1.1 302 Found\r\nLocation: /docs/loop-alias\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
                            continue;
                        }
                        if url == "/docs/rate" && count == 1 {
                            let _ = respond(&mut stream, 429, b"slow down");
                            continue;
                        }
                        if url == "/docs/missing"
                            || ((url == "/docs/a" || url == "/custom")
                                && phase.load(Ordering::Acquire) == 2)
                        {
                            let _ = respond(&mut stream, 404, b"missing page");
                            continue;
                        }
                        let links = if url == "/docs" {
                            format!(
                                "<a href='/docs/a?b=2&a=1&a=0&utm_source=x#heading'>query</a><a href='/docs/a/?b=2'>slash</a><a href='/docs/hash#/one'>route one</a><a href='/docs/hash#!/two'>route two</a><a href='/docs/alias'>alias</a><a href='/docs/private'>robot excluded</a><a href='/docs/private/ok'>robot allowed</a><a href='/docs/logout'>action</a><a href='/outside'>section</a><a href='http://127.0.0.2:9/external'>unapproved origin</a><a href='/docs/loop?page=1'>pagination</a><a href='/docs/rate'>rate limit</a><a href='/docs/missing'>missing</a><a href='/docs/loop-alias'>redirect loop</a><script>document.addEventListener('DOMContentLoaded',()=>{{let a=document.createElement('a');a.href='/docs/rendered';a.textContent='Rendered link';document.body.append(a)}})</script>{}",
                                if phase.load(Ordering::Acquire) > 0 {
                                    "<a href='/docs/new'>Newly added page</a>"
                                } else {
                                    ""
                                }
                            )
                        } else if url.starts_with("/docs/loop?") {
                            format!(
                                "<a href='/docs/loop?page={}'>next</a>",
                                if url.ends_with('1') { 2 } else { 1 }
                            )
                        } else {
                            String::new()
                        };
                        let body = format!(
                            "<!doctype html><meta charset=utf-8><meta name=viewport content='width=device-width, initial-scale=1'><style>body{{margin:0;background:white;font:18px sans-serif}}h1{{padding:20px}}.box{{height:50px;width:50px;background:{}}}</style><h1>Owned Phase 3 fixture</h1><div class=box></div><p>{url}</p>{links}",
                            if phase.load(Ordering::Acquire) == 0 {
                                "#08786f"
                            } else {
                                "#cc3344"
                            }
                        );
                        let _ = respond(&mut stream, 200, body.as_bytes());
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            address,
            state,
            hits,
            stop,
            thread: Some(thread),
        })
    }
    fn count(&self, path: &str) -> u32 {
        *self.hits.lock().unwrap().get(path).unwrap_or(&0)
    }
}
impl Drop for CrawlSite {
    fn drop(&mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}

#[test]
#[ignore = "Focused signed-worker resource interception regressions; fresh evidence root required"]
fn actual_phase3_resource_corrections() -> Result<()> {
    let output = PathBuf::from(std::env::var("REGRESSEE_PHASE3_EVIDENCE")?);
    fs::create_dir(&output)?;
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.runtime/installed");
    let pack = output.join("development-worker3.zip");
    let key = development_pack(&source, &pack)?;
    let delivery = Site::start(pack)?;
    let trust = Distribution {
        version: 1,
        url: format!("{}/runtime.zip", delivery.address).parse()?,
        public_key: hex::encode(key.verifying_key().to_bytes()),
        scope: Scope::DevelopmentLoopback,
    };
    fs::write(
        output.join("distribution.json"),
        serde_json::to_vec_pretty(&trust)?,
    )?;
    let product = Product {
        distribution: Some(trust.clone()),
        runtime_cache: output.join("app/cache/product-runtime"),
        official_runtime: false,
        auth: Default::default(),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    product.setup(cancel.clone(), &mut |_, _| {})?;
    let installed = distribution::resolve(&product.runtime_cache, &trust, &cancel)?;
    let suite = std::process::Command::new(installed.root.join("node/bin/node"))
        .args(["--test", "--test-concurrency=1"])
        .arg(installed.root.join("worker/test/protocol.test.mjs"))
        .arg(installed.root.join("worker/test/capture.test.mjs"))
        .arg(
            installed
                .root
                .join("worker/test/resources.browser.test.mjs"),
        )
        .env("PLAYWRIGHT_BROWSERS_PATH", installed.root.join("browsers"))
        .env("REGRESSEE_RESOURCE_EVIDENCE", output.join("resources"))
        .env(
            "REGRESSEE_RESOURCE_RUNTIME",
            serde_json::to_string(&installed.manifest.identity)?,
        )
        .output()?;
    fs::write(
        output.join("worker-suite.log"),
        [suite.stdout, suite.stderr].concat(),
    )?;
    assert!(suite.status.success(), "see worker-suite.log");
    Ok(())
}

#[test]
#[ignore = "Focused real signed-worker navigation/robots corrections; fresh evidence root required"]
fn actual_phase3_policy_corrections() -> Result<()> {
    let output = PathBuf::from(std::env::var("REGRESSEE_PHASE3_EVIDENCE")?);
    fs::create_dir(&output)?;
    let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.runtime/installed");
    let pack = output.join("development-worker3.zip");
    let key = development_pack(&source, &pack)?;
    let delivery = Site::start(pack)?;
    let trust = Distribution {
        version: 1,
        url: format!("{}/runtime.zip", delivery.address).parse()?,
        public_key: hex::encode(key.verifying_key().to_bytes()),
        scope: Scope::DevelopmentLoopback,
    };
    fs::write(
        output.join("distribution.json"),
        serde_json::to_vec_pretty(&trust)?,
    )?;
    let product = Product {
        distribution: Some(trust.clone()),
        runtime_cache: output.join("app/cache/product-runtime"),
        official_runtime: false,
        auth: Default::default(),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    product.setup(cancel.clone(), &mut |_, _| {})?;
    let installed = distribution::resolve(&product.runtime_cache, &trust, &cancel)?;
    let suite = std::process::Command::new(installed.root.join("node/bin/node"))
        .args(["--test", "--test-concurrency=1"])
        .arg(installed.root.join("worker/test/protocol.test.mjs"))
        .arg(installed.root.join("worker/test/capture.test.mjs"))
        .output()?;
    fs::write(
        output.join("worker-suite.log"),
        [suite.stdout, suite.stderr].concat(),
    )?;
    assert!(suite.status.success());
    let second = CrawlSite::start()?;
    let fixture = CrawlSite::with_cross(Some(second.address.clone()))?;
    let mut store = Store::open(AppPaths::isolated(&output.join("app")))?;
    let mut project = Project::draft(
        "Policy corrections".into(),
        &format!("{}/scope", fixture.address),
    )?;
    project.discovery.section = Some("/scope".into());
    project.discovery.skipped_text = vec!["skipped".into()];
    project.discovery.speed = Speed::Fast;
    project.discovery.advanced.retries = 0;
    store.save_project(&project, None)?;
    let preview = product.discover(&store, project.id, cancel.clone(), &mut |_, _, _| {})?;
    fs::write(
        output.join("preview.json"),
        serde_json::to_vec_pretty(&preview)?,
    )?;
    for path in [
        "redirect-section",
        "redirect-robots",
        "redirect-wild",
        "to-danger",
        "to-omit",
        "redirect-download",
        "redirect-cross",
        "wild/secret",
        "client",
        "meta",
        "form",
        "frame",
        "chain-one",
        "chain-two",
        "loop-one",
        "loop-two",
        "loop-self",
    ] {
        let page = preview
            .inventory
            .pages
            .iter()
            .find(|p| p.canonical.path() == format!("/scope/{path}"))
            .with_context(|| format!("missing preview {path}"))?;
        assert!(!page.included, "excluded candidate {path} became evidence");
    }
    let wild = preview
        .inventory
        .pages
        .iter()
        .find(|p| p.canonical.path() == "/scope/wild/secret")
        .unwrap();
    assert_eq!(wild.reason.as_deref(), Some("Excluded by robots.txt"));
    assert!(
        preview
            .inventory
            .pages
            .iter()
            .any(|p| p.canonical.path() == "/scope/allowed"
                && p.canonical.query() == Some("b=2&a=1&a=0")
                && p.included)
    );
    for (from, to) in [
        ("/scope/chain-one", "/scope/chain-two"),
        ("/scope/chain-two", "/scope/allowed"),
    ] {
        assert!(
            preview
                .aliases
                .iter()
                .any(|(a, b)| a.path() == from && b.path() == to)
        );
    }
    assert_eq!(fixture.count("/scope/chain-one"), 1);
    assert_eq!(fixture.count("/scope/to-danger"), 1);
    assert_eq!(fixture.count("/scope/to-omit"), 1);
    assert_eq!(fixture.count("/scope/chain-two"), 1);
    assert_eq!(fixture.count("/scope/allowed?b=2&a=1&a=0"), 1);
    assert_eq!(fixture.count("/scope/loop-one"), 1);
    assert_eq!(fixture.count("/scope/loop-two"), 1);
    assert_eq!(fixture.count("/scope/loop-self"), 1);
    assert_eq!(
        fixture.count("/scope/form"),
        1,
        "automatic POST must not visit even the same URL"
    );
    assert!(
        !preview
            .inventory
            .pages
            .iter()
            .any(|p| p.canonical.path() == "/scope/poison")
    );
    let zero = || {
        for path in [
            "/outside",
            "/scope/private",
            "/scope/wild/secret",
            "/scope/delete",
            "/scope/skipped",
            "/scope/file.pdf",
            "/scope/poison",
        ] {
            assert_eq!(fixture.count(path), 0, "forbidden actual request: {path}");
        }
        assert_eq!(
            second.count("/identity-provider"),
            0,
            "unapproved origin actually visited"
        );
    };
    zero();
    // Initial list-only inventory may contain an approved alias, but capture must
    // reject any move outside its exact frozen target, before the destination.
    let mut alias_project = Project::draft(
        "Fixed redirected target".into(),
        &format!("{}/scope/redirect-section", fixture.address),
    )?;
    alias_project.discovery.mode = DiscoveryMode::OnlyAddresses;
    alias_project.discovery.speed = Speed::Fast;
    alias_project.discovery.advanced.retries = 0;
    store.save_project(&alias_project, None)?;
    let alias_preview =
        product.discover(&store, alias_project.id, cancel.clone(), &mut |_, _, _| {})?;
    let failed = product.capture_inventory(
        &mut store,
        alias_project.id,
        Some(alias_preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert!(matches!(
        failed.run.targets[0].outcome,
        TargetOutcome::Failed {
            code: CaptureError::RedirectOutsideScope
        }
    ));
    assert_eq!(failed.run.targets[0].attempts, 1);
    assert!(failed.captures.is_empty());
    let mut active_project = Project::draft(
        "Automatic navigations".into(),
        &format!("{}/scope/redirect-cross", fixture.address),
    )?;
    active_project.sources = [
        "redirect-cross",
        "to-danger",
        "client",
        "meta",
        "form",
        "frame",
    ]
    .iter()
    .map(|p| format!("{}/scope/{p}", fixture.address).parse().unwrap())
    .collect();
    active_project.discovery.mode = DiscoveryMode::OnlyAddresses;
    active_project.discovery.speed = Speed::Fast;
    active_project.discovery.advanced.retries = 0;
    store.save_project(&active_project, None)?;
    let active_preview =
        product.discover(&store, active_project.id, cancel.clone(), &mut |_, _, _| {})?;
    let active = product.capture_inventory(
        &mut store,
        active_project.id,
        Some(active_preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert!(active.captures.is_empty());
    assert!(
        active
            .run
            .targets
            .iter()
            .all(|t| matches!(t.outcome, TargetOutcome::Failed { .. }))
    );
    assert_eq!(
        fixture.count("/scope/form"),
        2,
        "neither discovery nor capture may submit the form"
    );
    zero();
    // An approved cross-origin redirect remains a policy-checked candidate.
    let mut approved = Project::draft(
        "Approved redirect".into(),
        &format!("{}/scope/approved-cross", fixture.address),
    )?;
    approved
        .sources
        .push(format!("{}/scope/cross-allowed", second.address).parse()?);
    approved.discovery.section = Some("/scope".into());
    approved.discovery.speed = Speed::Fast;
    store.save_project(&approved, None)?;
    let approved_preview =
        product.discover(&store, approved.id, cancel.clone(), &mut |_, _, _| {})?;
    assert_eq!(approved_preview.aliases.len(), 1);
    assert_eq!(
        approved_preview.aliases[0].0.path(),
        "/scope/approved-cross"
    );
    assert_eq!(
        approved_preview.aliases[0].1.origin().ascii_serialization(),
        second.address
    );
    assert_eq!(second.count("/scope/cross-allowed"), 1);
    assert!(
        approved_preview
            .inventory
            .pages
            .iter()
            .any(|p| p.included && p.canonical.path() == "/scope/cross-allowed")
    );
    // Baseline/check provenance stays frozen: the changed server now redirects
    // its saved target to a robots-forbidden page. No screenshot/unchanged result.
    let mut stable_preview = preview.clone();
    for page in &mut stable_preview.inventory.pages {
        page.included = page.canonical.path() == "/scope/stable";
    }
    let baseline_run = product.capture_inventory(
        &mut store,
        project.id,
        Some(stable_preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(baseline_run.run.state, JobState::Completed);
    assert_eq!(
        baseline_run.captures[0].metadata.requested,
        baseline_run.captures[0].metadata.final_url
    );
    let baseline = product.activate(&mut store, baseline_run.run.id, 1)?;
    fixture.state.store(1, Ordering::Release);
    let check = product.capture_inventory(
        &mut store,
        project.id,
        None,
        true,
        None,
        cancel.clone(),
        pause,
        &mut |_, _, _| {},
    )?;
    assert!(matches!(
        check.run.targets[0].outcome,
        TargetOutcome::Failed {
            code: CaptureError::RedirectOutsideScope
        }
    ));
    assert!(check.captures.is_empty());
    assert_eq!(
        store.load_project(project.id)?.project.active_baseline,
        baseline.project.active_baseline
    );
    assert_eq!(
        store
            .project_evidence(project.id)?
            .comparison
            .unwrap()
            .revision
            .results[0]
            .classification,
        Classification::CouldNotCapture
    );
    zero();
    fs::write(
        output.join("policy-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"main_hits":*fixture.hits.lock().unwrap(), "second_origin_hits":*second.hits.lock().unwrap(), "aliases":preview.aliases, "approved_cross_origin_preview":approved_preview, "automatic_navigation_capture":active.run, "failed_capture":failed.run, "redirected_check":check.run, "forbidden_destination_requests":0, "external_native_platform_soak":"PENDING"}),
        )?,
    )?;
    Ok(())
}

#[test]
#[ignore = "Real Phase 3 signed development pack, multi-page/profile capture, crash/resume and 100-page scale. Use a fresh evidence directory."]
fn actual_phase3_crawl_capture_resume_and_scale() -> Result<()> {
    use regressee_core::{crawl::TargetCounts, evidence::RunRecord};
    let output = PathBuf::from(std::env::var("REGRESSEE_PHASE3_EVIDENCE")?);
    fs::create_dir(&output)?;
    let retained = std::env::var_os("REGRESSEE_PHASE3_RETAINED_FLOW").map(PathBuf::from);
    let (_delivery, distribution, runtime_cache) = if let Some(retained) = &retained {
        let d: Distribution =
            serde_json::from_reader(File::open(retained.join("distribution.json"))?)?;
        assert_eq!(d.scope, Scope::DevelopmentLoopback);
        (None, d, retained.join("app/cache/product-runtime"))
    } else {
        let source = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../.runtime/installed");
        let pack = output.join("development-worker3.zip");
        let key = development_pack(&source, &pack)?;
        let delivery = Site::start(pack)?;
        let d = Distribution {
            version: 1,
            url: format!("{}/runtime.zip", delivery.address).parse()?,
            public_key: hex::encode(key.verifying_key().to_bytes()),
            scope: Scope::DevelopmentLoopback,
        };
        (Some(delivery), d, output.join("app/cache/product-runtime"))
    };
    let fixture = CrawlSite::start()?;
    let paths = AppPaths::isolated(&output.join("app"));
    let mut store = Store::open(paths.clone())?;
    fs::write(
        output.join("distribution.json"),
        serde_json::to_vec_pretty(&distribution)?,
    )?;
    let product = Product {
        distribution: Some(distribution.clone()),
        runtime_cache,
        official_runtime: false,
        auth: Default::default(),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    if retained.is_none() {
        product.setup(cancel.clone(), &mut |_, _| {})?;
    }
    let installed = distribution::resolve(&product.runtime_cache, &distribution, &cancel)?;
    for file in [
        "src/index.mjs",
        "src/capture.mjs",
        "src/discover.mjs",
        "src/protocol.mjs",
    ] {
        assert_eq!(
            fs::read(installed.root.join("worker").join(file))?,
            fs::read(
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../worker/product")
                    .join(file)
            )?,
            "retained signed worker must match maintained source"
        );
    }
    let suite = std::process::Command::new(installed.root.join("node/bin/node"))
        .arg("--test")
        .arg(installed.root.join("worker/test/protocol.test.mjs"))
        .arg(installed.root.join("worker/test/capture.test.mjs"))
        .output()?;
    fs::write(
        output.join("worker-suite.log"),
        [suite.stdout, suite.stderr].concat(),
    )?;
    assert!(suite.status.success());
    let mut project = Project::draft(
        "Guided multi-page fixture".into(),
        &format!("{}/docs", fixture.address),
    )?;
    project.discovery.section = Some("/docs".into());
    project.discovery.page_limit = 40;
    project.discovery.speed = Speed::Fast;
    project.profiles = vec![
        CaptureProfile::desktop(),
        CaptureProfile::tablet(),
        CaptureProfile::mobile(),
    ];
    store.save_project(&project, None)?;
    let preview = product.discover(&store, project.id, cancel.clone(), &mut |_, _, _| {})?;
    assert!(
        preview
            .inventory
            .pages
            .iter()
            .any(|p| p.canonical.as_str().ends_with("?b=2&a=1&a=0") && p.included)
    );
    assert!(
        preview
            .inventory
            .pages
            .iter()
            .any(|p| p.canonical.fragment() == Some("/one") && p.included)
    );
    assert!(
        preview
            .inventory
            .pages
            .iter()
            .any(|p| p.canonical.fragment() == Some("!/two") && p.included)
    );
    assert!(
        preview
            .inventory
            .pages
            .iter()
            .any(|p| p.canonical.path() == "/docs/rendered" && p.included)
    );
    assert_eq!(preview.aliases.len(), 1);
    assert!(
        preview
            .inventory
            .pages
            .iter()
            .any(|p| p.canonical.path() == "/docs/loop-alias" && !p.included)
    );
    assert!(
        preview
            .inventory
            .pages
            .iter()
            .any(|p| p.reason.as_deref() == Some("Excluded by robots.txt"))
    );
    assert_eq!(fixture.count("/docs/private"), 0);
    assert_eq!(fixture.count("/docs/logout"), 0);
    assert_eq!(fixture.count("/outside"), 0);
    assert_eq!(fixture.count("/docs/loop?page=1"), 1);
    assert_eq!(fixture.count("/docs/loop?page=2"), 1);
    assert_eq!(fixture.count("/docs/rate"), 2);
    fs::write(
        output.join("discovery-preview.json"),
        serde_json::to_vec_pretty(&preview)?,
    )?;
    // Pause is requested after a real persisted batch, not invented progress.
    let paused = product.capture_inventory(
        &mut store,
        project.id,
        Some(preview.clone()),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, completed, _| {
            if completed > 0 {
                pause.store(true, Ordering::Release);
            }
        },
    )?;
    assert_eq!(paused.run.state, JobState::Paused);
    assert!(!paused.captures.is_empty() && paused.captures.len() < paused.run.targets.len());
    assert!(product.activate(&mut store, paused.run.id, 1).is_err());
    let retained = paused.captures[0].image.original.clone();
    drop(store);
    let mut store = Store::open(paths.clone())?;
    assert_eq!(store.recover_runs()?, 0);
    pause.store(false, Ordering::Release);
    let baseline = product.capture_inventory(
        &mut store,
        project.id,
        None,
        false,
        Some(paused.run.id),
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(baseline.run.state, JobState::Completed);
    assert_eq!(baseline.captures.len(), preview.included() * 3);
    assert_eq!(baseline.queue.as_ref().unwrap().resumed_at.len(), 1);
    assert_eq!(baseline.captures[0].image.original, retained);
    assert!(baseline.run.profiles[2].mobile && baseline.run.profiles[2].touch);
    assert_eq!(baseline.run.profiles[1].width, 768);
    let saved = product.activate(&mut store, baseline.run.id, 1)?;
    let active = saved.project.active_baseline;
    let unchanged = product.capture_inventory(
        &mut store,
        project.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(unchanged.run.state, JobState::Completed);
    assert!(
        store
            .project_evidence(project.id)?
            .comparison
            .unwrap()
            .revision
            .results
            .iter()
            .all(|r| r.classification == Classification::NoSignificantChange)
    );
    fixture.state.store(1, Ordering::Release);
    let new_before = fixture.count("/docs/new");
    let changed = product.capture_inventory(
        &mut store,
        project.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(changed.run.inventory, baseline.run.inventory);
    assert_eq!(fixture.count("/docs/new"), new_before);
    assert!(
        store
            .project_evidence(project.id)?
            .comparison
            .unwrap()
            .revision
            .results
            .iter()
            .all(|r| r.classification == Classification::Changed)
    );
    fixture.state.store(2, Ordering::Release);
    let failed = product.capture_inventory(
        &mut store,
        project.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(failed.run.state, JobState::CompletedWithErrors);
    assert_eq!(TargetCounts::from_run(&failed.run).failed, 3);
    assert_eq!(
        store
            .project_evidence(project.id)?
            .comparison
            .unwrap()
            .revision
            .results
            .iter()
            .filter(|r| r.classification == Classification::CouldNotCapture)
            .count(),
        3
    );
    assert_eq!(fixture.count("/docs/new"), new_before);
    assert_eq!(
        store.load_project(project.id)?.project.active_baseline,
        active
    );
    fixture.state.store(0, Ordering::Release);
    // Explicit replacement settings do not mutate the completed baseline.
    let mut replacement = saved.project.clone();
    replacement.discovery.page_limit = 1;
    store.save_replacement_project(&replacement, saved.revision)?;
    let limited = product.discover(&store, project.id, cancel.clone(), &mut |_, _, _| {})?;
    assert_eq!(limited.included(), 1);
    assert!(matches!(
        limited.inventory.coverage,
        DiscoveryCoverage::Limited { .. }
    ));
    assert_eq!(
        store.load_run(baseline.run.id)?.run.inventory,
        baseline.run.inventory
    );
    // Crash a real application while resuming a list-only queued run, preserve
    // each already-published original, then restart/resume its pending targets.
    let mut recovery = Project::draft(
        "Restart queue".into(),
        &format!("{}/scale/0", fixture.address),
    )?;
    recovery.sources = (0..12)
        .map(|i| format!("{}/scale/{i}", fixture.address).parse().unwrap())
        .collect();
    recovery.discovery.mode = DiscoveryMode::OnlyAddresses;
    recovery.discovery.speed = Speed::Gentle;
    store.save_project(&recovery, None)?;
    let queue_preview = product.discover(&store, recovery.id, cancel.clone(), &mut |_, _, _| {})?;
    let paused_queue = product.capture_inventory(
        &mut store,
        recovery.id,
        Some(queue_preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, done, _| {
            if done > 0 {
                pause.store(true, Ordering::Release)
            }
        },
    )?;
    pause.store(false, Ordering::Release);
    let mut app = std::process::Command::new(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../target/debug/regressee-flow"),
    )
    .arg(output.join("app"))
    .arg("resume")
    .arg(paused_queue.run.id.to_string())
    .env("REGRESSEE_DISTRIBUTION", output.join("distribution.json"))
    .env("REGRESSEE_TEST_RUNTIME_CACHE", &product.runtime_cache)
    .stdout(File::create(output.join("restart-app.stdout"))?)
    .stderr(File::create(output.join("restart-app.stderr"))?)
    .spawn()?;
    let deadline = Instant::now() + Duration::from_secs(40);
    loop {
        let r = store.load_run(paused_queue.run.id)?;
        if r.run.state == JobState::Running && r.captures.len() > paused_queue.captures.len() {
            break;
        }
        ensure!(
            Instant::now() < deadline && app.try_wait()?.is_none(),
            "owned app did not reach a persisted capture boundary"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    assert_eq!(store.recover_runs()?, 0);
    app.kill()?;
    app.wait()?;
    assert_eq!(store.recover_runs()?, 1);
    let interrupted = store.load_run(paused_queue.run.id)?;
    assert_eq!(interrupted.run.state, JobState::Interrupted);
    let resumed = product.capture_inventory(
        &mut store,
        recovery.id,
        None,
        false,
        Some(interrupted.run.id),
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(resumed.run.state, JobState::Completed);
    for c in &interrupted.captures {
        assert_eq!(
            resumed.evidence(&c.metadata.key).unwrap().image.original,
            c.image.original
        );
    }
    // True cancel stops scheduling, preserves safe evidence and never activates.
    let cancel_preview =
        product.discover(&store, recovery.id, cancel.clone(), &mut |_, _, _| {})?;
    let cancelled = product.capture_inventory(
        &mut store,
        recovery.id,
        Some(cancel_preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, done, _| {
            if done > 0 {
                cancel.store(true, Ordering::Release)
            }
        },
    )?;
    assert_eq!(cancelled.run.state, JobState::Cancelled);
    assert!(TargetCounts::from_run(&cancelled.run).cancelled > 0);
    assert!(product.activate(&mut store, cancelled.run.id, 1).is_err());
    cancel.store(false, Ordering::Release);
    // A real 100-page x three-profile run. No synthesized navigation counts.
    let scale_count = std::env::var("REGRESSEE_PHASE3_SCALE_PAGES")
        .ok()
        .map(|s| s.parse::<usize>())
        .transpose()?
        .unwrap_or(100);
    ensure!((1..=1000).contains(&scale_count), "scale range 1–1000");
    let mut scale = Project::draft(
        "Measured scale fixture".into(),
        &format!("{}/scale/0", fixture.address),
    )?;
    scale.sources = (0..scale_count)
        .map(|i| format!("{}/scale/{i}", fixture.address).parse().unwrap())
        .collect();
    scale.profiles = vec![
        CaptureProfile::desktop(),
        CaptureProfile::tablet(),
        CaptureProfile::mobile(),
    ];
    scale.discovery.mode = DiscoveryMode::OnlyAddresses;
    scale.discovery.speed = Speed::Fast;
    scale.discovery.advanced.per_origin = 4;
    scale.discovery.page_limit = scale_count as u32;
    store.save_project(&scale, None)?;
    let scale_preview = product.discover(&store, scale.id, cancel.clone(), &mut |_, _, _| {})?;
    let start = Instant::now();
    let before_bytes = store.summary()?.artifact_bytes;
    let before_hits: u32 = fixture
        .hits
        .lock()
        .unwrap()
        .iter()
        .filter(|(p, _)| p.starts_with("/scale/"))
        .map(|(_, n)| n)
        .sum();
    let mut boundary_latencies = Vec::new();
    let mut last = Instant::now();
    let measured: RunRecord = product.capture_inventory(
        &mut store,
        scale.id,
        Some(scale_preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {
            boundary_latencies.push(last.elapsed().as_secs_f64());
            last = Instant::now();
        },
    )?;
    let duration = start.elapsed().as_secs_f64();
    assert_eq!(measured.run.state, JobState::Completed);
    assert_eq!(measured.captures.len(), scale_count * 3);
    assert!(measured.run.targets.iter().all(|t| t.attempts == 1));
    let after_hits: u32 = fixture
        .hits
        .lock()
        .unwrap()
        .iter()
        .filter(|(p, _)| p.starts_with("/scale/"))
        .map(|(_, n)| n)
        .sum();
    assert_eq!(after_hits - before_hits, (scale_count * 3) as u32);
    fs::write(
        output.join("phase3-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"scope":"Actual product services, sandboxed worker3, owned literal-loopback only","preview_pages":preview.included(),"profiles":3,"paused_saved_targets":paused.captures.len(),"restart_retained_targets":interrupted.captures.len(),"fixed_comparison_new_page_visits":fixture.count("/docs/new")-new_before,"failed_profile_targets":3,"scale_pages":scale_count,"scale_targets":measured.captures.len(),"actual_scale_page_visits":after_hits-before_hits,"scale_duration_seconds":duration,"artifact_disk_growth_bytes":store.summary()?.artifact_bytes-before_bytes,"safe_boundary_intervals_seconds":boundary_latencies,"native_input_latency":"NOT MEASURED","public_runtime":"PENDING","soak_1000_pages":if scale_count==1000{"RUN"}else{"NOT RUN: separate 3000-target soak exceeds this bounded implementation verification budget"}}),
        )?,
    )?;
    Ok(())
}

#[test]
#[ignore = "Actual retained worker3, in-navigation cancellation and exact custom viewport capture"]
fn actual_phase3_active_cancel_and_custom_viewport() -> Result<()> {
    let retained = PathBuf::from(std::env::var("REGRESSEE_PHASE3_RETAINED_FLOW")?);
    let output = PathBuf::from(std::env::var("REGRESSEE_PHASE3_EVIDENCE")?);
    fs::create_dir(&output)?;
    let distribution: Distribution =
        serde_json::from_reader(File::open(retained.join("distribution.json"))?)?;
    assert_eq!(distribution.scope, Scope::DevelopmentLoopback);
    let product = Product {
        distribution: Some(distribution),
        runtime_cache: retained.join("app/cache/product-runtime"),
        official_runtime: false,
        auth: Default::default(),
    };
    let fixture = CrawlSite::start()?;
    let mut store = Store::open(AppPaths::isolated(&output.join("app")))?;
    let cancel = Arc::new(AtomicBool::new(false));
    let pause = Arc::new(AtomicBool::new(false));
    let mut custom = Project::draft(
        "Exact custom viewport".into(),
        &format!("{}/custom", fixture.address),
    )?;
    custom.discovery.mode = DiscoveryMode::OnlyAddresses;
    custom.profiles = vec![CaptureProfile {
        name: "Custom".into(),
        width: 600,
        height: 300,
        extent: CaptureExtent::VisibleScreen,
        ..CaptureProfile::desktop()
    }];
    store.save_project(&custom, None)?;
    let preview = product.discover(&store, custom.id, cancel.clone(), &mut |_, _, _| {})?;
    pause.store(true, Ordering::Release);
    let paused_custom = product.capture_inventory(
        &mut store,
        custom.id,
        Some(preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(paused_custom.run.state, JobState::Paused);
    assert!(paused_custom.captures.is_empty());
    assert!(
        product
            .activate(&mut store, paused_custom.run.id, 1)
            .is_err()
    );
    pause.store(false, Ordering::Release);
    let captured = product.capture_inventory(
        &mut store,
        custom.id,
        None,
        false,
        Some(paused_custom.run.id),
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(captured.run.state, JobState::Completed);
    assert_eq!(
        (
            captured.captures[0].image.width,
            captured.captures[0].image.height
        ),
        (600, 300)
    );
    assert_eq!(
        captured.captures[0].metadata.profile.extent,
        CaptureExtent::VisibleScreen
    );
    let saved = product.activate(&mut store, captured.run.id, 1)?;
    fixture.state.store(2, Ordering::Release);
    let failed_check = product.capture_inventory(
        &mut store,
        custom.id,
        None,
        true,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |_, _, _| {},
    )?;
    assert_eq!(failed_check.run.state, JobState::CompletedWithErrors);
    let mut fabricated = store
        .project_evidence(custom.id)?
        .comparison
        .context("failed check must have coverage analysis")?;
    assert_eq!(
        fabricated.revision.results[0].classification,
        Classification::CouldNotCapture
    );
    fabricated.revision.id = AnalysisId::new();
    fabricated.revision.results[0].classification = Classification::NoSignificantChange;
    fabricated.revision.results[0].eligible_pixels = 1;
    assert!(
        store
            .save_analysis(&fabricated)
            .unwrap_err()
            .to_string()
            .contains("failed or missing target")
    );
    fixture.state.store(0, Ordering::Release);
    let mut slow = Project::draft(
        "Cancel a real active navigation".into(),
        &format!("{}/slow", fixture.address),
    )?;
    slow.discovery.mode = DiscoveryMode::OnlyAddresses;
    slow.discovery.speed = Speed::Gentle;
    store.save_project(&slow, None)?;
    let preview = product.discover(&store, slow.id, cancel.clone(), &mut |_, _, _| {})?;
    let hits = fixture.hits.clone();
    let flag = cancel.clone();
    let observed = Arc::new(std::sync::Mutex::new(None));
    let recorded = observed.clone();
    let mut trigger = None;
    let cancelled = product.capture_inventory(
        &mut store,
        slow.id,
        Some(preview),
        false,
        None,
        cancel.clone(),
        pause.clone(),
        &mut |message, _, _| {
            if message == "Capturing the next bounded batch" && trigger.is_none() {
                let hits = hits.clone();
                let flag = flag.clone();
                let recorded = recorded.clone();
                trigger = Some(std::thread::spawn(move || {
                    let deadline = Instant::now() + Duration::from_secs(10);
                    while Instant::now() < deadline {
                        if *hits.lock().unwrap().get("/slow").unwrap_or(&0) > 0 {
                            *recorded.lock().unwrap() = Some(Instant::now());
                            flag.store(true, Ordering::Release);
                            return;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    panic!("Actual slow navigation was not observed");
                }));
            }
        },
    )?;
    trigger
        .context("cancel trigger not started")?
        .join()
        .unwrap();
    let cancel_latency = observed
        .lock()
        .unwrap()
        .context("cancel not observed")?
        .elapsed()
        .as_secs_f64();
    assert!(cancel_latency < 10.);
    assert_eq!(cancelled.run.state, JobState::Cancelled);
    assert_eq!(cancelled.run.targets[0].outcome, TargetOutcome::Cancelled);
    assert!(product.activate(&mut store, cancelled.run.id, 1).is_err());
    assert_eq!(
        store.load_project(custom.id)?.project.active_baseline,
        saved.project.active_baseline
    );
    fs::write(
        output.join("active-cancel-report.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"actual_slow_page_visits":fixture.count("/slow"),"observed_navigation_to_cancel_return_seconds":cancel_latency,"custom_viewport_pixels":[600,300],"baseline_preserved":true,"native_ui_latency":"NOT MEASURED"}),
        )?,
    )?;
    Ok(())
}
