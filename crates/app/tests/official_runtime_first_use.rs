// SPDX-License-Identifier: MPL-2.0
//! Explicit real-network first-use check. Never runs with the ordinary test suite.
#[path = "support/private_paths.rs"]
mod private_paths;
use anyhow::{Context, Result, ensure};
use regressee_app::{
    jobs::{Jobs, Reply, Request, Update, Work},
    product::{Product, RuntimeState},
};
use regressee_core::domain::{JobState, Project, Speed};
use regressee_runtime::distribution::Scope;
use regressee_storage::{AppPaths, Store};
use std::{
    fs,
    io::{Read, Write},
    net::TcpListener,
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::Receiver,
    },
    time::{Duration, Instant},
};

struct Site {
    address: String,
    stopped: Arc<AtomicBool>,
    thread: Option<std::thread::JoinHandle<()>>,
}
impl Site {
    fn start() -> Result<Self> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        let address = format!("http://{}/", listener.local_addr()?);
        let stopped = Arc::new(AtomicBool::new(false));
        let shutdown = stopped.clone();
        let thread = std::thread::spawn(move || {
            while !shutdown.load(Ordering::Acquire) {
                match listener.accept() {
                    Ok((mut socket, _)) => {
                        let _ = socket.set_read_timeout(Some(Duration::from_secs(2)));
                        let _ = socket.set_write_timeout(Some(Duration::from_secs(2)));
                        let mut request = [0; 8192];
                        let n = socket.read(&mut request).unwrap_or(0);
                        if n == 0 {
                            continue;
                        }
                        let request = String::from_utf8_lossy(&request[..n]);
                        let route = request
                            .lines()
                            .next()
                            .and_then(|s| s.split_whitespace().nth(1))
                            .unwrap_or("");
                        let (status, body) = match route {
                            "/robots.txt" => (200, "User-agent: *\nAllow: /\n"),
                            "/" => (
                                200,
                                "<!doctype html><title>Owned official-runtime fixture</title><style>body{margin:0;background:#fff;font:20px sans-serif}h1{padding:32px;color:#08786f}</style><h1>Fresh private browser</h1><a href='/second'>Second page</a>",
                            ),
                            "/second" => (
                                200,
                                "<!doctype html><title>Second owned page</title><h1>Second page</h1>",
                            ),
                            _ => (404, "Not found"),
                        };
                        let _ = write!(
                            socket,
                            "HTTP/1.1 {status} OK\r\nContent-Type: text/html\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                            body.len()
                        );
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            address,
            stopped,
            thread: Some(thread),
        })
    }
}
impl Drop for Site {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Release);
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}
struct OwnedJobs(Arc<Jobs>);
impl Drop for OwnedJobs {
    fn drop(&mut self) {
        self.0.join_after_ui();
    }
}
fn terminal(
    events: &Receiver<Update>,
    id: u64,
    seconds: u64,
    phases: &mut Vec<String>,
) -> Result<Reply> {
    let until = Instant::now() + Duration::from_secs(seconds);
    loop {
        let update = events.recv_timeout(until.saturating_duration_since(Instant::now()))?;
        ensure!(update.id == id, "unexpected request reply");
        match update.reply {
            Reply::Progress { message, .. } => {
                // Record phase changes only, not hundreds of transfer chunks.
                let phase = message
                    .split('·')
                    .next()
                    .unwrap_or(&message)
                    .trim()
                    .to_owned();
                if phases.last() != Some(&phase) {
                    eprintln!("First-use phase: {phase}");
                    phases.push(phase);
                }
            }
            Reply::CaptureStatus(_) => (),
            Reply::ProductFailure(message) => anyhow::bail!("{message}"),
            Reply::ActionRequired(problem) => {
                anyhow::bail!("{}: {}", problem.title(), problem.message())
            }
            Reply::Failure(failure) => anyhow::bail!("{}", failure.message()),
            reply => return Ok(reply),
        }
    }
}

#[test]
#[ignore = "explicit official HTTPS download and sandboxed owned-fixture first-use check"]
fn fresh_default_discovery_installs_and_reopens_verified_browser() -> Result<()> {
    ensure!(
        std::env::var_os("REGRESSEE_DISTRIBUTION").is_none(),
        "this check requires the actual app default, not a configured override"
    );
    let root = PathBuf::from(
        std::env::var_os("REGRESSEE_OFFICIAL_CHECK_DIR")
            .context("explicit private verification directory required")?,
    );
    private_paths::private_directory(&root)?;
    let paths = AppPaths::isolated(&root.join("first-use-store"));
    let mut store = Store::open(paths.clone())?;
    let site = Site::start()?;
    let mut project = Project::draft("Official browser first-use fixture".into(), &site.address)?;
    project.discovery.speed = Speed::Fast;
    project.discovery.page_limit = 2;
    store.save_project(&project, None)?;
    let before = serde_json::to_value(store.load_project(project.id)?)?;
    let product = Product::configured(&paths.cache)?;
    let cancel = AtomicBool::new(false);
    let readiness = product.readiness(&cancel)?;
    ensure!(
        readiness.status == RuntimeState::NotInstalled
            && readiness.setup_available
            && readiness.scope == Some(Scope::OfficialPinned),
        "default setup is not available"
    );
    ensure!(
        !product.runtime_cache.join("current").exists(),
        "must be a fresh cache"
    );
    drop(store);
    let (jobs, events) = Jobs::start(paths.clone());
    let owned = OwnedJobs(jobs.clone());
    jobs.submit(Request {
        id: 1,
        work: Work::Discover(project.id),
    })
    .map_err(anyhow::Error::msg)?;
    let mut phases = Vec::new();
    let Reply::Discovery(preview) = terminal(&events, 1, 1200, &mut phases)? else {
        anyhow::bail!("discovery reply missing");
    };
    ensure!(
        preview.inventory.pages.len() == 2,
        "owned page inventory incomplete"
    );
    ensure!(
        phases.iter().any(|p| p.contains("Downloading"))
            && phases.iter().any(|p| p.contains("Testing")),
        "real first-use phases missing"
    );
    let pointer = fs::read(product.runtime_cache.join("current"))?;
    let installed = product.resolve_runtime(&cancel)?;
    ensure!(
        installed.manifest.scope == Scope::OfficialPinned,
        "wrong runtime trust path"
    );
    let mut installed_again = false;
    product.ensure_runtime(Arc::new(AtomicBool::new(false)), &mut |_| {
        installed_again = true
    })?;
    ensure!(
        !installed_again && fs::read(product.runtime_cache.join("current"))? == pointer,
        "repeat use unexpectedly installed a runtime"
    );
    jobs.submit(Request {
        id: 2,
        work: Work::CaptureInventory {
            project: project.id,
            preview: Some(preview),
            check: false,
            resume: None,
        },
    })
    .map_err(anyhow::Error::msg)?;
    let Reply::Evidence(evidence) = terminal(&events, 2, 180, &mut phases)? else {
        anyhow::bail!("capture evidence reply missing");
    };
    let captured = evidence.latest.context("capture run missing")?;
    ensure!(
        captured.run.state == JobState::Completed && captured.captures.len() == 2,
        "actual owned screenshot captures failed"
    );
    drop(owned);
    let store = Store::open(paths.clone())?;
    ensure!(
        serde_json::to_value(store.load_project(project.id)?)? == before,
        "first-use changed project settings"
    );
    let reopened = Product::configured(&paths.cache)?;
    ensure!(
        reopened.readiness(&cancel)?.status == RuntimeState::Ready,
        "reopened installed runtime is not ready"
    );
    ensure!(
        reopened.resolve_runtime(&cancel)?.manifest.identity == installed.manifest.identity,
        "runtime identity changed on reopen"
    );
    fs::write(
        root.join("official-first-use-result.json"),
        serde_json::to_vec_pretty(
            &serde_json::json!({"scope": "OfficialPinned", "pages": 2, "captures": 2,
            "runtime": installed.manifest.identity, "reopened_ready": true,
            "repeat_download": false, "phases": phases,
            "native_ui_verified": false, "public_release_qualified": false}),
        )?,
    )?;
    Ok(())
}
