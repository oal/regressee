// SPDX-License-Identifier: MPL-2.0
use anyhow::{Context, Result, bail, ensure};
use serde_json::{Value, json};
use std::{
    io::{BufReader, Write},
    path::Path,
    process::{Child, ChildStdin, Command, Stdio},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    time::Duration,
};
pub const MAX_LINE: usize = 1024 * 1024;
#[cfg(test)]
pub(crate) static SPAWN_ATTEMPTS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
#[cfg(target_os = "linux")]
const LINUX_SCRATCH: &str = "/tmp/opencode/r0";
pub struct Worker {
    child: Child,
    input: ChildStdin,
    output: Receiver<Result<Value>>,
    next: u64,
    pub cancelled: Arc<AtomicBool>,
    scratch: std::path::PathBuf,
    browser_root: std::path::PathBuf,
    stopped: bool,
}

#[cfg(target_os = "linux")]
fn kill_private_browsers(scratch: &Path, browser_root: &Path) {
    let Ok(processes) = std::fs::read_dir("/proc") else {
        return;
    };
    let needle = scratch.as_os_str().as_encoded_bytes();
    for process in processes.flatten() {
        let Ok(pid) = process.file_name().to_string_lossy().parse::<i32>() else {
            continue;
        };
        let Ok(exe) = std::fs::read_link(process.path().join("exe")) else {
            continue;
        };
        if !exe.starts_with(browser_root) {
            continue;
        }
        let Ok(args) = std::fs::read(process.path().join("cmdline")) else {
            continue;
        };
        if !args
            .windows(needle.len() + 1)
            .any(|w| &w[..needle.len()] == needle && matches!(w[needle.len()], b'/' | 0))
        {
            continue;
        }
        if let Ok(stat) = std::fs::read_to_string(process.path().join("stat"))
            && let Some((_, fields)) = stat.rsplit_once(')')
            && let Some(group) = fields
                .split_whitespace()
                .nth(2)
                .and_then(|s| s.parse::<i32>().ok())
            && group > 1
            && group != unsafe { libc::getpgrp() }
        {
            unsafe {
                libc::kill(-group, libc::SIGKILL);
            }
        }
        unsafe {
            libc::kill(pid, libc::SIGKILL);
        }
    }
}

/// Last-resort native-window shutdown, including Playwright's detached leaders.
/// Only marked scratch owned by this application process is eligible.
#[cfg(target_os = "linux")]
pub fn emergency_stop(pid: u32) {
    let Ok(entries) = std::fs::read_dir(LINUX_SCRATCH) else {
        return;
    };
    for entry in entries.flatten() {
        let scratch = entry.path();
        let Ok(bytes) = std::fs::read(scratch.join("owner.json")) else {
            continue;
        };
        let Ok(owner) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        if owner["namespace"] != "regressee-phase0-private-scratch-v1"
            || owner["parent"] != std::process::id()
        {
            continue;
        }
        let Some(worker) = owner["worker"]
            .as_u64()
            .filter(|p| *p > 1 && *p <= i32::MAX as u64)
        else {
            continue;
        };
        if pid != 0 && worker != u64::from(pid) {
            continue;
        }
        let Some(browser_root) = owner["browserRoot"].as_str() else {
            continue;
        };
        let expected = format!("TMPDIR={}", scratch.display());
        let matches_worker = std::fs::read(format!("/proc/{worker}/environ"))
            .is_ok_and(|env| env.split(|b| *b == 0).any(|v| v == expected.as_bytes()));
        if matches_worker {
            unsafe {
                libc::kill(-(worker as i32), libc::SIGKILL);
            }
        }
        for _ in 0..10 {
            kill_private_browsers(&scratch, Path::new(browser_root));
            std::thread::sleep(Duration::from_millis(50));
        }
        let _ = std::fs::remove_dir_all(scratch);
    }
}

fn recover_scratch(parent: &Path) -> Result<()> {
    std::fs::create_dir_all(parent)?;
    #[cfg(target_os = "linux")]
    for entry in std::fs::read_dir(parent)? {
        let path = entry?.path();
        if !path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .starts_with("worker-")
            || !std::fs::symlink_metadata(&path)?.is_dir()
        {
            continue;
        }
        let Ok(bytes) = std::fs::read(path.join("owner.json")) else {
            continue;
        };
        let Ok(owner) = serde_json::from_slice::<Value>(&bytes) else {
            continue;
        };
        if owner["namespace"] != "regressee-phase0-private-scratch-v1" {
            continue;
        }
        let Some(pid) = owner["parent"]
            .as_u64()
            .filter(|p| *p > 0 && *p <= i32::MAX as u64)
        else {
            continue;
        };
        let parent_dead = unsafe { libc::kill(pid as i32, 0) } < 0
            && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH);
        let worker = owner["worker"].as_u64().unwrap_or(0);
        if parent_dead && let Some(browser_root) = owner["browserRoot"].as_str() {
            kill_private_browsers(&path, Path::new(browser_root));
        }
        let mut group_live = false;
        for process in std::fs::read_dir("/proc")? {
            let Ok(stat) = std::fs::read_to_string(process?.path().join("stat")) else {
                continue;
            };
            let Some((_, fields)) = stat.rsplit_once(')') else {
                continue;
            };
            let fields: Vec<_> = fields.split_whitespace().collect();
            if fields.len() > 2 && fields[0] != "Z" && fields[2].parse::<u64>().ok() == Some(worker)
            {
                group_live = true;
                break;
            }
        }
        let group_dead = worker > 0 && worker <= i32::MAX as u64 && !group_live;
        if parent_dead && group_dead {
            std::fs::remove_dir_all(path)?;
        }
    }
    Ok(())
}
impl Worker {
    pub fn spawn(root: &Path) -> Result<Self> {
        Self::spawn_cancellable(root, Arc::new(AtomicBool::new(false)))
    }
    pub fn spawn_cancellable(root: &Path, cancel: Arc<AtomicBool>) -> Result<Self> {
        crate::runtime::verify_cancellable(root, &cancel)?;
        crate::cancel::check(&cancel, "worker launch")?;
        let mut worker = Self::spawn_unverified(root)?;
        worker.cancelled = cancel;
        Ok(worker)
    }
    pub fn spawn_unverified(root: &Path) -> Result<Self> {
        #[cfg(test)]
        SPAWN_ATTEMPTS.fetch_add(1, Ordering::Relaxed);
        let root = root.canonicalize()?;
        // Chromium's Unix-domain socket names must fit sockaddr_un. Long project
        // paths plus Playwright profile names can otherwise abort Chromium.
        #[cfg(target_os = "linux")]
        let scratch_parent = std::path::PathBuf::from(LINUX_SCRATCH);
        #[cfg(not(target_os = "linux"))]
        let scratch_parent = std::env::temp_dir().join("regressee-p0");
        recover_scratch(&scratch_parent)?;
        let scratch = scratch_parent.join(format!(
            "worker-{:x}-{:x}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_nanos()
                & 0xffff_ffff_ffff
        ));
        std::fs::create_dir(&scratch)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&scratch, std::fs::Permissions::from_mode(0o700))?;
        }
        let node = root.join(if cfg!(windows) {
            "node/node.exe"
        } else {
            "node/bin/node"
        });
        let mut command = Command::new(node);
        command
            .arg(root.join("worker/src/index.mjs"))
            .env("PLAYWRIGHT_BROWSERS_PATH", root.join("browsers"))
            .env("TMPDIR", &scratch)
            .env("TMP", &scratch)
            .env("TEMP", &scratch)
            .env(
                "REGRESSEE_ARTIFACTS",
                std::env::current_dir()?.join("artifacts"),
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            unsafe {
                command.pre_exec(|| {
                    if libc::setsid() < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    #[cfg(target_os = "linux")]
                    if libc::prctl(libc::PR_SET_PDEATHSIG, libc::SIGTERM) < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
        }
        let mut child = match command.spawn().context("launch private worker") {
            Ok(child) => child,
            Err(e) => {
                let _ = std::fs::remove_dir_all(&scratch);
                return Err(e);
            }
        };
        std::fs::write(
            scratch.join("owner.json"),
            serde_json::to_vec(
                &json!({"namespace":"regressee-phase0-private-scratch-v1","parent":std::process::id(),"worker":child.id(),"browserRoot":root.join("browsers")}),
            )?,
        )?;
        let input = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let (tx, output) = mpsc::sync_channel(32);
        std::thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                let value = match regressee_runtime::protocol::read_json_frame(&mut reader) {
                    Ok(None) => {
                        let _ = tx.send(Err(anyhow::anyhow!("worker stopped unexpectedly")));
                        break;
                    }
                    Ok(Some(value)) => Ok(value),
                    Err(e) => Err(e),
                };
                let failed = value.is_err();
                if tx.send(value).is_err() || failed {
                    break;
                }
            }
        });
        Ok(Self {
            child,
            input,
            output,
            next: 0,
            cancelled: Arc::new(AtomicBool::new(false)),
            scratch,
            browser_root: root.join("browsers"),
            stopped: false,
        })
    }
    pub fn pid(&self) -> u32 {
        self.child.id()
    }
    fn send(&mut self, id: u64, command: &str, payload: Value) -> Result<()> {
        let frame = serde_json::to_vec(
            &json!({"v":1,"id":id,"job":"phase0","command":command,"payload":payload}),
        )?;
        ensure!(frame.len() < MAX_LINE, "request too large");
        self.input.write_all(&frame)?;
        self.input.write_all(b"\n")?;
        self.input.flush()?;
        Ok(())
    }
    pub fn request(&mut self, command: &str, payload: Value) -> Result<Value> {
        crate::cancel::check(&self.cancelled, "capture")?;
        self.next += 1;
        let id = self.next;
        self.send(id, command, payload)?;
        let deadline = std::time::Instant::now() + Duration::from_secs(120);
        let mut ack = false;
        loop {
            if self.cancelled.load(Ordering::Relaxed) {
                self.next += 1;
                let _ = self.send(self.next, "cancel", json!({}));
                let end = std::time::Instant::now() + Duration::from_secs(2);
                while std::time::Instant::now() < end {
                    if let Ok(Ok(event)) = self.output.recv_timeout(Duration::from_millis(100))
                        && event["type"] == "cancelled"
                    {
                        self.kill_tree();
                        bail!("capture cancelled; worker cleaned up");
                    }
                }
                self.kill_tree();
                bail!("cancel grace expired; worker tree stopped");
            }
            ensure!(std::time::Instant::now() < deadline, "worker timeout");
            let event = match self.output.recv_timeout(Duration::from_millis(100)) {
                Ok(event) => event?,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => bail!("worker output closed"),
            };
            ensure!(
                event["v"] == 1 && event["job"] == "phase0",
                "protocol version/job mismatch"
            );
            if event["id"] != id {
                continue;
            }
            match event["type"].as_str() {
                Some("ack") => ack = true,
                Some("result") => {
                    ensure!(ack, "result without acknowledgement");
                    return Ok(event["payload"].clone());
                }
                Some("failure") => {
                    if let Some(assertion) = event["payload"]["fixtureAssertion"].as_str() {
                        bail!("synthetic fixture assertion: {assertion}");
                    }
                    return Err(WorkerFailure {
                        code: event["code"].as_str().unwrap_or("Unknown").to_owned(),
                    }
                    .into());
                }
                Some("progress" | "warning" | "requires-login") => (),
                _ => bail!("unexpected worker event"),
            }
        }
    }
    pub fn shutdown(mut self) -> Result<()> {
        self.request("shutdown", json!({}))?;
        self.kill_tree();
        Ok(())
    }
    fn kill_tree(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        #[cfg(target_os = "linux")]
        kill_private_browsers(&self.scratch, &self.browser_root);
        #[cfg(unix)]
        unsafe {
            libc::kill(-(self.child.id() as i32), libc::SIGKILL);
        }
        #[cfg(windows)]
        {
            let _ = Command::new("taskkill")
                .args(["/PID", &self.child.id().to_string(), "/T", "/F"])
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .status();
        }
        let _ = self.child.kill();
        let _ = self.child.wait();
        // Playwright launches Unix browser leaders detached from the Node group.
        // Include a short post-exit scan for a child that was mid-spawn on cancel.
        #[cfg(target_os = "linux")]
        for _ in 0..10 {
            kill_private_browsers(&self.scratch, &self.browser_root);
            std::thread::sleep(Duration::from_millis(50));
        }
        if std::fs::remove_dir_all(&self.scratch).is_err() && self.scratch.is_dir() {
            let _ = std::fs::write(self.scratch.join("owner.json"), serde_json::to_vec(&json!({"namespace":"regressee-phase0-private-scratch-v1","parent":std::process::id(),"worker":self.child.id(),"browserRoot":self.browser_root})).unwrap());
        }
    }
}
#[derive(Debug)]
pub struct WorkerFailure {
    pub code: String,
}
impl std::fmt::Display for WorkerFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "worker failure: {}", self.code)
    }
}
impl std::error::Error for WorkerFailure {}
impl Drop for Worker {
    fn drop(&mut self) {
        self.kill_tree();
    }
}

pub fn fault_tests(root: &Path) -> Result<()> {
    let mut w = Worker::spawn(root)?;
    w.request("handshake", json!({}))?;
    let recovered = w.request("crashBrowser", json!({}))?;
    ensure!(
        recovered["recovered"] == true,
        "browser crash recovery failed"
    );
    w.shutdown()?;
    let mut w = Worker::spawn(root)?;
    w.request("handshake", json!({}))?;
    let cancel = w.cancelled.clone();
    std::thread::spawn(move || {
        std::thread::sleep(Duration::from_millis(750));
        cancel.store(true, Ordering::Relaxed);
    });
    let started = std::time::Instant::now();
    let error = w
        .request("hang", json!({}))
        .expect_err("hang must be cancelled")
        .to_string();
    ensure!(
        error.contains("cancel"),
        "hang failed for a reason other than cancellation: {error}"
    );
    ensure!(
        started.elapsed() < Duration::from_secs(5),
        "cancel exceeded bounded grace/cleanup"
    );
    drop(w);
    let mut w = Worker::spawn(root)?;
    w.request("handshake", json!({}))?;
    ensure!(
        w.request("crashWorker", json!({})).unwrap_err().to_string()
            == "worker stopped unexpectedly",
        "crash must report EOF interruption"
    );
    ensure!(
        w.child.wait()?.code() == Some(86),
        "deliberate worker crash must exit 86"
    );
    drop(w);
    let mut w = Worker::spawn(root)?;
    w.request("handshake", json!({}))?;
    let unknown = w
        .request("notAllowed", json!({}))
        .expect_err("unknown command must fail");
    ensure!(
        unknown
            .downcast_ref::<WorkerFailure>()
            .is_some_and(|e| e.code == "UnknownCommand"),
        "unknown command lost typed category: {unknown}"
    );
    drop(w);
    let mut w = Worker::spawn(root)?;
    w.request("handshake", json!({}))?;
    ensure!(
        w.request("oversized", json!({})).unwrap_err().to_string()
            == "worker frame exceeds limit or is incomplete",
        "oversized event must report framing failure"
    );
    drop(w);
    println!(
        "PASS: browser crash/relaunch, bounded cancel, worker crash/restart, command whitelist, frame budget"
    );
    Ok(())
}
