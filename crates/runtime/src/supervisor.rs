// SPDX-License-Identifier: MPL-2.0
//! Bounded product worker adapter. Linux ownership includes detached Chromium.
use crate::protocol::{Event, Request};
use anyhow::Result;
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
    time::Instant,
};

#[derive(Debug, Clone)]
pub struct SupervisorConfig {
    /// Only a locally verified runtime resolver may populate these paths.
    /// Neither project files nor portable manifests contain executable paths.
    pub private_node: PathBuf,
    pub private_worker: PathBuf,
    pub private_browser_root: PathBuf,
    pub private_scratch: PathBuf,
    pub output_staging: PathBuf,
}
/// Implementors must use direct arguments (no shell), bound stdout/stderr,
/// correlate acknowledged requests, honor deadline/cancel, and reap their owned
/// process tree. Phase 0's proven Linux scratch/launch-race fixes are the starting
/// point for the Phase 2 adapter; cross-platform process ownership stays open.
pub trait ProcessSupervisor: Send {
    fn request(
        &mut self,
        request: Request,
        deadline: Instant,
        cancelled: Arc<AtomicBool>,
        events: &mut dyn FnMut(Event) -> Result<()>,
    ) -> Result<()>;
    fn stop(&mut self, deadline: Instant) -> Result<()>;
}

#[cfg(target_os = "linux")]
use crate::linux_guardian::Child;
#[cfg(windows)]
use crate::windows_process::Child;
use crate::{
    distribution::{Installed, Scope, check_cancel},
    protocol::{self, Command as WireCommand, EventKind, RequestTracker},
};
use anyhow::{Context, ensure};
#[cfg(not(target_os = "linux"))]
use std::io::BufReader;
#[cfg(not(any(windows, target_os = "linux")))]
use std::process::{Child, ChildStdin};
use std::{
    io::{Read, Write},
    process::{Command, Stdio},
    sync::{
        atomic::Ordering,
        mpsc::{self, Receiver},
    },
    thread::JoinHandle,
    time::Duration,
};
#[cfg(windows)]
type ChildStdin = std::fs::File;
#[cfg(target_os = "linux")]
type ChildStdin = std::fs::File;

pub struct Worker {
    permit: Option<BrowserPermit>,
    child: Child,
    input: ChildStdin,
    output: Receiver<Result<Event>>,
    readers: Vec<JoinHandle<()>>,
    readers_stop: Arc<AtomicBool>,
    #[cfg(target_os = "linux")]
    scratch: crate::scratch::Scratch,
    #[cfg(windows)]
    scratch: crate::windows_scratch::Scratch,
    #[cfg(not(any(target_os = "linux", windows)))]
    scratch: tempfile::TempDir,
    #[cfg(target_os = "linux")]
    guardian: crate::linux_guardian::Control,
    #[cfg(target_os = "linux")]
    node_pid: u32,
    stopped: bool,
    #[cfg(target_os = "linux")]
    tree_settled: bool,
}
#[cfg(target_os = "linux")]
struct PendingLaunch {
    child: Option<Child>,
    guardian: Option<crate::linux_guardian::Control>,
}
#[cfg(target_os = "linux")]
impl Drop for PendingLaunch {
    fn drop(&mut self) {
        if let Some(child) = &mut self.child {
            if let Some(control) = &self.guardian {
                control.stop();
            }
            let _ = settle_guardian(child);
        }
    }
}
#[cfg(target_os = "linux")]
fn settle_guardian(child: &mut Child) -> bool {
    let deadline = Instant::now() + Duration::from_secs(7);
    loop {
        match child.try_wait() {
            Ok(Some(_)) => return true,
            Ok(None) if Instant::now() < deadline => std::thread::sleep(Duration::from_millis(10)),
            _ => break,
        }
    }
    // Only this exact unreaped Child can be signaled, never its PID group.
    // Unkillable kernel tasks must not turn a bounded owner into an endless wait.
    let _ = child.kill();
    let deadline = Instant::now() + Duration::from_secs(1);
    while Instant::now() < deadline {
        if child.try_wait().is_ok_and(|status| status.is_some()) {
            return true;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    false // No deletion/acceptance; preserve durable scratch ownership evidence.
}
static ACTIVE_BROWSERS: std::sync::atomic::AtomicUsize = std::sync::atomic::AtomicUsize::new(0);
struct BrowserPermit;
impl BrowserPermit {
    fn acquire() -> Result<Self> {
        ACTIVE_BROWSERS
            .fetch_update(Ordering::AcqRel, Ordering::Acquire, |n| {
                if n < 8 { Some(n + 1) } else { None }
            })
            .map_err(|_| {
                anyhow::anyhow!(
                    "The global eight-browser limit is in use; wait before starting more work"
                )
            })?;
        Ok(Self)
    }
}
impl Drop for BrowserPermit {
    fn drop(&mut self) {
        ACTIVE_BROWSERS.fetch_sub(1, Ordering::AcqRel);
    }
}
#[derive(Debug)]
pub struct WorkerFailure(pub protocol::FailureCode);
impl std::fmt::Display for WorkerFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "worker failure: {:?}", self.0)
    }
}
impl std::error::Error for WorkerFailure {}
impl Worker {
    /// `Installed` must come from the signed resolver/installer, not project data.
    pub fn spawn(installed: &Installed) -> Result<Self> {
        Self::spawn_network(installed, &crate::network::NetworkSettings::default())
    }
    pub fn spawn_network(
        installed: &Installed,
        network: &crate::network::NetworkSettings,
    ) -> Result<Self> {
        network.validate()?;
        #[cfg(target_os = "linux")]
        let parent = crate::scratch::parent()?;
        #[cfg(target_os = "linux")]
        let _lifecycle = crate::scratch::lifecycle(&parent)?;
        let permit = BrowserPermit::acquire()?;
        ensure!(
            cfg!(any(target_os = "linux", windows)),
            "macOS owned process-tree supervision is not implemented; capture remains blocked"
        );
        #[cfg(target_os = "linux")]
        let mut scratch = {
            recover_scratch(&parent)?;
            crate::scratch::create(&parent)?
        };
        // Unreachable off Linux: retained only for compilation, not qualification.
        #[cfg(windows)]
        let scratch = crate::windows_scratch::Scratch::create()?;
        #[cfg(not(any(target_os = "linux", windows)))]
        let scratch = tempfile::tempdir()?;
        // Resolver overrides/headless roots may be relative. /proc exe paths
        // are absolute: canonicalize before launch AND ownership comparisons.
        let root = installed.root.canonicalize()?;
        let browser_root = root.join(&installed.manifest.browser_root);
        #[cfg(target_os = "linux")]
        let mut owner = crate::scratch::Owner::new(
            scratch.path(),
            root.join(&installed.manifest.node),
            browser_root.clone(),
        );
        #[cfg(target_os = "linux")]
        owner.write()?;
        #[cfg(target_os = "linux")]
        scratch.mark_owned();
        let mut command = Command::new(root.join(&installed.manifest.node));
        // Explicit private user-state roots. Never inherit Node injection, loader
        // overrides or a real browser profile through HOME/APPDATA/CWD.
        command.env_clear().current_dir(scratch.path());
        for key in [
            "SystemRoot",
            "WINDIR",
            "DISPLAY",
            "WAYLAND_DISPLAY",
            "XAUTHORITY",
            "XDG_RUNTIME_DIR",
            "DBUS_SESSION_BUS_ADDRESS",
            "LANG",
        ] {
            if let Some(value) = std::env::var_os(key) {
                command.env(key, value);
            }
        }
        for key in [
            "HOME",
            "USERPROFILE",
            "APPDATA",
            "LOCALAPPDATA",
            "XDG_CONFIG_HOME",
            "XDG_CACHE_HOME",
            "XDG_DATA_HOME",
        ] {
            command.env(key, scratch.path());
        }
        if let Some(proxy) = &network.proxy {
            command.env("REGRESSEE_PROXY", proxy);
        } else {
            command.env_remove("REGRESSEE_PROXY");
        }
        if let Some(ca) = &network.trusted_ca {
            command.env("NODE_EXTRA_CA_CERTS", ca);
        } else {
            command.env_remove("NODE_EXTRA_CA_CERTS");
        }
        if std::env::var_os("REGRESSEE_CAPTURE_TRACE_EVIDENCE").is_some() {
            command.env("REGRESSEE_PRIVATE_CAPTURE_DIAGNOSTICS", "1");
        }
        command
            .arg(root.join(&installed.manifest.worker))
            .env_remove("NODE_OPTIONS")
            .env_remove("NODE_PATH")
            .env_remove("NODE_TLS_REJECT_UNAUTHORIZED")
            .env("PLAYWRIGHT_BROWSERS_PATH", &browser_root)
            .env("TMPDIR", scratch.path())
            .env("TMP", scratch.path())
            .env("TEMP", scratch.path())
            .env("REGRESSEE_OUTPUT", scratch.path())
            .env(
                "REGRESSEE_RUNTIME",
                serde_json::to_string(&installed.manifest.identity)?,
            )
            .env(
                "REGRESSEE_SCOPE",
                if installed.manifest.scope == Scope::DevelopmentLoopback {
                    "loopback"
                } else {
                    "public"
                },
            )
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        #[cfg(all(unix, not(target_os = "linux")))]
        {
            use std::os::unix::process::CommandExt;
            unsafe {
                command.pre_exec(move || {
                    if libc::setsid() < 0 {
                        return Err(std::io::Error::last_os_error());
                    }
                    Ok(())
                });
            }
        }
        #[cfg(target_os = "linux")]
        let receipt = {
            use std::os::unix::fs::OpenOptionsExt;
            let file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(scratch.path().join("settled"))?;
            file.sync_all()?;
            std::fs::File::open(scratch.path())?.sync_all()?;
            file
        };
        #[cfg(target_os = "linux")]
        let launch = crate::linux_guardian::Launch::prepare_registered(
            &mut command,
            &receipt,
            owner.secret(),
            &owner.registration()?,
        )?;
        #[cfg(not(any(windows, target_os = "linux")))]
        let mut child = command
            .spawn()
            .context("launch verified private product worker")?;
        #[cfg(target_os = "linux")]
        let mut child = launch
            .spawn()
            .context("launch verified private product worker")?;
        #[cfg(windows)]
        let mut child =
            Child::spawn(&command).context("launch private Job-owned product worker")?;
        #[cfg(target_os = "linux")]
        let (guardian, node_pid) = match launch.started() {
            Ok(started) => started,
            Err(error) => {
                // Closing the control pipe requests guardian cleanup. Do not kill
                // the ancestor and abandon its children if registration fails.
                let _ = settle_guardian(&mut child);
                return Err(error);
            }
        };
        #[cfg(target_os = "linux")]
        if let Err(e) = (|| -> Result<()> {
            owner.worker = child.id();
            owner.guardian_start =
                crate::scratch::birth(child.id()).context("guardian lifetime missing")?;
            owner.write()
        })() {
            guardian.stop();
            let _ = settle_guardian(&mut child);
            return Err(e);
        }
        #[cfg(target_os = "linux")]
        let mut pending = PendingLaunch {
            child: Some(child),
            guardian: Some(guardian),
        };
        #[cfg(target_os = "linux")]
        let child = pending.child.as_mut().unwrap();
        let input = child.stdin.take().unwrap();
        let stdout = child.stdout.take().unwrap();
        let mut stderr = child.stderr.take().unwrap();
        #[cfg(target_os = "linux")]
        for fd in [&stdout as &dyn std::os::fd::AsRawFd, &stderr] {
            let flags = unsafe { libc::fcntl(fd.as_raw_fd(), libc::F_GETFL) };
            ensure!(
                flags >= 0
                    && unsafe {
                        libc::fcntl(fd.as_raw_fd(), libc::F_SETFL, flags | libc::O_NONBLOCK)
                    } == 0,
                "bound guardian stdio reader"
            );
        }
        let readers_stop = Arc::new(AtomicBool::new(false));
        let parser_stop = readers_stop.clone();
        let diagnostics_stop = readers_stop.clone();
        let (tx, output) = mpsc::sync_channel(8);
        let parser = std::thread::spawn(move || {
            #[cfg(target_os = "linux")]
            let mut stdout = stdout;
            #[cfg(not(target_os = "linux"))]
            let mut reader = BufReader::new(stdout);
            #[cfg(target_os = "linux")]
            let mut pending = Vec::new();
            loop {
                if parser_stop.load(Ordering::Acquire) {
                    break;
                }
                #[cfg(not(target_os = "linux"))]
                let result = match protocol::read_json_frame(&mut reader) {
                    Ok(Some(v)) => protocol::decode_event(v),
                    Ok(None) => Err(WorkerFailure(protocol::FailureCode::WorkerStopped).into()),
                    Err(e) => Err(e),
                };
                #[cfg(target_os = "linux")]
                let result = read_event_bounded(&mut stdout, &mut pending, &parser_stop);
                let failed = result.is_err();
                let mut result = result;
                loop {
                    match tx.try_send(result) {
                        Ok(()) => break,
                        Err(mpsc::TrySendError::Full(value)) => {
                            if parser_stop.load(Ordering::Acquire) {
                                return;
                            }
                            result = value;
                            std::thread::sleep(Duration::from_millis(5));
                        }
                        Err(_) => return,
                    }
                }
                if failed {
                    break;
                }
            }
        });
        // Drain without retaining site/credential strings. Diagnostics are typed
        // protocol failures; raw browser stacks are deliberately not logged.
        let diagnostics = std::thread::spawn(move || {
            let mut b = [0; 4096];
            while !diagnostics_stop.load(Ordering::Acquire) {
                match stderr.read(&mut b) {
                    Ok(0) => break,
                    Ok(_) => (),
                    Err(e)
                        if matches!(
                            e.kind(),
                            std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                        ) =>
                    {
                        std::thread::sleep(Duration::from_millis(5))
                    }
                    Err(_) => break,
                }
            }
        });
        Ok(Self {
            permit: Some(permit),
            #[cfg(not(target_os = "linux"))]
            child,
            #[cfg(target_os = "linux")]
            child: pending.child.take().unwrap(),
            input,
            output,
            readers: vec![parser, diagnostics],
            readers_stop,
            scratch,
            #[cfg(target_os = "linux")]
            guardian: pending.guardian.take().unwrap(),
            #[cfg(target_os = "linux")]
            node_pid,
            stopped: false,
            #[cfg(target_os = "linux")]
            tree_settled: false,
        })
    }
    pub fn output_path(&self, token: uuid::Uuid) -> PathBuf {
        self.scratch.path().join(format!("{token}.png"))
    }
    pub fn metadata_path(&self, token: uuid::Uuid) -> PathBuf {
        self.scratch.path().join(format!("{token}.json"))
    }
    pub fn pid(&self) -> u32 {
        #[cfg(target_os = "linux")]
        return self.node_pid;
        #[cfg(not(target_os = "linux"))]
        self.child.id()
    }
    fn send(&mut self, request: &Request, deadline: Instant, cancelled: &AtomicBool) -> Result<()> {
        #[cfg(windows)]
        {
            // Anonymous Windows pipes are synchronous. One bounded writer owner
            // is interrupted by terminating the exact Job when deadline/cancel
            // occurs; no unbounded detached writer or leaked session buffer.
            let bytes = zeroize::Zeroizing::new(protocol::encode_request(request)?);
            let input = &mut self.input;
            let child = &mut self.child;
            return std::thread::scope(|scope| {
                let (tx, rx) = mpsc::sync_channel(1);
                scope.spawn(move || {
                    let _ = tx.send(input.write_all(&bytes));
                });
                loop {
                    if cancelled.load(Ordering::Acquire) || Instant::now() >= deadline {
                        let _ = child.kill();
                        let _ = child.wait();
                        return Err(WorkerFailure(if cancelled.load(Ordering::Acquire) {
                            protocol::FailureCode::Cancelled
                        } else {
                            protocol::FailureCode::Timeout
                        })
                        .into());
                    }
                    match rx.recv_timeout(Duration::from_millis(5)) {
                        Ok(result) => return Ok(result?),
                        Err(mpsc::RecvTimeoutError::Timeout) => (),
                        Err(_) => anyhow::bail!("Private pipe writer stopped"),
                    }
                }
            });
        }
        #[cfg(not(windows))]
        {
            // Private session frames can exceed the pipe capacity. A stopped reader
            // must not hold the blocking owner forever before request's event deadline.
            #[cfg(target_os = "linux")]
            {
                use std::os::fd::AsRawFd;
                let fd = self.input.as_raw_fd();
                let flags = unsafe { libc::fcntl(fd, libc::F_GETFL) };
                ensure!(
                    flags >= 0
                        && unsafe { libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) } >= 0,
                    "Could not bound the private worker pipe"
                );
            }
            let bytes = zeroize::Zeroizing::new(protocol::encode_request(request)?);
            write_bounded(&mut self.input, &bytes, deadline, cancelled)?;
            Ok(())
        }
    }
    fn kill_tree(&mut self) {
        if self.stopped {
            return;
        }
        self.stopped = true;
        #[cfg(target_os = "linux")]
        self.guardian.stop();
        // The Child handle owns this unreaped PID. Do not signal a numeric
        // process group: a detached/reused group is not proof of ownership.
        #[cfg(not(target_os = "linux"))]
        let _ = self.child.kill();
        #[cfg(target_os = "linux")]
        {
            self.tree_settled = settle_guardian(&mut self.child)
                && crate::scratch::Owner::read(self.scratch.path())
                    .is_some_and(|owner| owner.settled() && owner.guardian_exited());
        }
        #[cfg(not(target_os = "linux"))]
        let _ = self.child.wait();
        // Includes a browser that was mid-launch when cancellation killed Node.
        while self.output.try_recv().is_ok() {}
        self.readers_stop.store(true, Ordering::Release);
        // Drop cannot join a parser blocked on a full bounded queue. Killing the
        // writer then draining unblocks it; join while draining until completion.
        for handle in self.readers.drain(..) {
            while !handle.is_finished() {
                let _ = self.output.recv_timeout(Duration::from_millis(10));
            }
            let _ = handle.join();
        }
        // Cancellation/stop removes only this validated unique namespace entry.
        // Keep its marker if removal fails so the same recovery policy can retry.
        #[cfg(target_os = "linux")]
        self.scratch.cleanup();
        #[cfg(target_os = "linux")]
        if !self.tree_settled {
            // Unproved trees must not silently free a slot in the eight-browser
            // budget. Conservatively reserve it until application restart/recovery.
            if let Some(permit) = self.permit.take() {
                std::mem::forget(permit);
            }
        }
        self.permit.take();
    }
    pub fn handshake(
        &mut self,
        job: regressee_core::domain::RunId,
        cancel: Arc<AtomicBool>,
    ) -> Result<(String, String)> {
        let mut identity = None;
        self.request(
            Request {
                version: 2,
                request_id: 1,
                job,
                command: WireCommand::Handshake,
            },
            Instant::now() + Duration::from_secs(30),
            cancel,
            &mut |e| {
                if let EventKind::Ready {
                    worker_version,
                    browser_version,
                } = e.event
                {
                    identity = Some((worker_version, browser_version));
                }
                Ok(())
            },
        )?;
        identity.context("worker did not identify its runtime")
    }
}
#[cfg(target_os = "linux")]
fn read_event_bounded(
    reader: &mut impl Read,
    pending: &mut Vec<u8>,
    stop: &AtomicBool,
) -> Result<Event> {
    let mut bytes = [0; 4096];
    loop {
        if stop.load(Ordering::Acquire) {
            return Err(WorkerFailure(protocol::FailureCode::WorkerStopped).into());
        }
        if let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
            ensure!(end < protocol::MAX_FRAME, "worker frame exceeds limit");
            let frame = pending.drain(..=end).collect::<Vec<_>>();
            return protocol::decode_event(serde_json::from_slice(&frame)?);
        }
        ensure!(
            pending.len() < protocol::MAX_FRAME,
            "worker frame exceeds limit or is incomplete"
        );
        match reader.read(&mut bytes) {
            Ok(0) => return Err(WorkerFailure(protocol::FailureCode::WorkerStopped).into()),
            Ok(n) => pending.extend_from_slice(&bytes[..n]),
            Err(e)
                if matches!(
                    e.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::Interrupted
                ) =>
            {
                std::thread::sleep(Duration::from_millis(5))
            }
            Err(e) => return Err(e.into()),
        }
    }
}
impl ProcessSupervisor for Worker {
    fn request(
        &mut self,
        request: Request,
        deadline: Instant,
        cancelled: Arc<AtomicBool>,
        events: &mut dyn FnMut(Event) -> Result<()>,
    ) -> Result<()> {
        check_cancel(&cancelled)?;
        ensure!(!self.stopped, "worker stopped");
        if let Err(e) = self.send(&request, deadline, &cancelled) {
            self.kill_tree();
            return Err(e);
        }
        let mut tracker = RequestTracker::new(&request);
        loop {
            if cancelled.load(Ordering::Acquire) {
                let cancel = Request {
                    version: 2,
                    request_id: request.request_id + 1,
                    job: request.job,
                    command: WireCommand::Cancel,
                };
                let _ = self.send(
                    &cancel,
                    Instant::now() + Duration::from_secs(2),
                    &AtomicBool::new(false),
                );
                let end = Instant::now() + Duration::from_secs(2);
                let mut t = RequestTracker::new(&cancel);
                while Instant::now() < end {
                    if let Ok(Ok(e)) = self.output.recv_timeout(Duration::from_millis(50))
                        && e.request_id == cancel.request_id
                        && t.accept(&e).is_ok()
                        && t.terminal()
                    {
                        break;
                    }
                }
                self.kill_tree();
                return Err(WorkerFailure(protocol::FailureCode::Cancelled).into());
            }
            if Instant::now() >= deadline {
                self.kill_tree();
                return Err(WorkerFailure(protocol::FailureCode::Timeout).into());
            }
            let e = match self.output.recv_timeout(Duration::from_millis(50)) {
                Ok(e) => e?,
                Err(mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => return Err(WorkerFailure(protocol::FailureCode::WorkerStopped).into()),
            };
            tracker.accept(&e)?;
            if let EventKind::Failure { code, .. } = e.event {
                return Err(WorkerFailure(code).into());
            }
            events(e)?;
            if tracker.terminal() {
                return Ok(());
            }
        }
    }
    fn stop(&mut self, deadline: Instant) -> Result<()> {
        if self.stopped {
            #[cfg(target_os = "linux")]
            ensure!(
                self.tree_settled,
                "Owned tree settlement unproved; private scratch retained"
            );
            return Ok(());
        }
        let r = self.request(
            Request {
                version: 2,
                request_id: 9000,
                job: regressee_core::domain::RunId::new(),
                command: WireCommand::Shutdown,
            },
            deadline,
            Arc::new(AtomicBool::new(false)),
            &mut |_| Ok(()),
        );
        self.kill_tree();
        #[cfg(target_os = "linux")]
        ensure!(
            self.tree_settled,
            "Owned tree settlement unproved; private scratch retained"
        );
        r
    }
}
impl Drop for Worker {
    fn drop(&mut self) {
        self.kill_tree();
    }
}
#[cfg(any(not(windows), test))]
fn write_bounded(
    writer: &mut impl Write,
    bytes: &[u8],
    deadline: Instant,
    cancelled: &AtomicBool,
) -> Result<()> {
    let mut offset = 0;
    while offset < bytes.len() {
        if cancelled.load(Ordering::Acquire) {
            return Err(WorkerFailure(protocol::FailureCode::Cancelled).into());
        }
        if Instant::now() >= deadline {
            return Err(WorkerFailure(protocol::FailureCode::Timeout).into());
        }
        match writer.write(&bytes[offset..]) {
            Ok(0) => anyhow::bail!("Private worker pipe closed"),
            Ok(n) => offset += n,
            Err(e) if e.kind() == std::io::ErrorKind::Interrupted => (),
            Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                std::thread::sleep(Duration::from_millis(5))
            }
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}

#[cfg(test)]
mod pipe_tests {
    use super::*;
    struct Stalled;
    impl Write for Stalled {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::ErrorKind::WouldBlock.into())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    #[test]
    fn stalled_private_transfer_honors_deadline_and_cancellation() {
        let bytes = vec![0; 512 * 1024];
        let start = Instant::now();
        let error = write_bounded(
            &mut Stalled,
            &bytes,
            start + Duration::from_millis(15),
            &AtomicBool::new(false),
        )
        .unwrap_err();
        assert_eq!(
            error.downcast_ref::<WorkerFailure>().unwrap().0,
            protocol::FailureCode::Timeout
        );
        assert!(start.elapsed() < Duration::from_secs(1));
        let error = write_bounded(
            &mut Stalled,
            &bytes,
            start + Duration::from_secs(10),
            &AtomicBool::new(true),
        )
        .unwrap_err();
        assert_eq!(
            error.downcast_ref::<WorkerFailure>().unwrap().0,
            protocol::FailureCode::Cancelled
        );
        let mut output = Vec::new();
        write_bounded(
            &mut output,
            &bytes,
            Instant::now() + Duration::from_secs(1),
            &AtomicBool::new(false),
        )
        .unwrap();
        assert_eq!(output, bytes);
        #[cfg(target_os = "linux")]
        {
            use std::os::fd::FromRawFd;
            let mut fds = [0; 2];
            assert_eq!(
                unsafe { libc::pipe2(fds.as_mut_ptr(), libc::O_NONBLOCK | libc::O_CLOEXEC) },
                0
            );
            let _reader = unsafe { std::fs::File::from_raw_fd(fds[0]) };
            let mut writer = unsafe { std::fs::File::from_raw_fd(fds[1]) };
            let start = Instant::now();
            let error = write_bounded(
                &mut writer,
                &bytes,
                start + Duration::from_millis(15),
                &AtomicBool::new(false),
            )
            .unwrap_err();
            assert_eq!(
                error.downcast_ref::<WorkerFailure>().unwrap().0,
                protocol::FailureCode::Timeout
            );
            assert!(start.elapsed() < Duration::from_secs(1));
        }
    }
}
#[cfg(target_os = "linux")]
fn recover_scratch(parent: &std::path::Path) -> Result<()> {
    let _lifecycle = crate::scratch::lifecycle(parent)?;
    crate::scratch::prepare_parent(parent)?;
    for entry in std::fs::read_dir(parent)? {
        let e = entry?;
        let path = e.path();
        if e.file_name() == ".launch-lock" {
            crate::scratch::private_file(&path, 0)
                .context("Linux launch lifecycle lock invalid; browser budget reserved")?;
            continue;
        }
        if e.file_name() == ".launches" {
            ensure!(
                crate::scratch::private_dir(&path),
                "Linux launch journal ownership/mode invalid"
            );
            for record in std::fs::read_dir(&path)? {
                let record = record?;
                let file = record.path();
                let stem = file.file_stem().and_then(|s| s.to_str()).unwrap_or("");
                ensure!(
                    uuid::Uuid::parse_str(stem).is_ok()
                        && file
                            .extension()
                            .is_some_and(|s| s == "json" || s == "birth"),
                    "Unknown Linux launch journal record; browser budget reserved; original record retained"
                );
                ensure!(
                    file.with_extension("json").try_exists()?
                        && file.with_extension("birth").try_exists()?,
                    "Incomplete Linux launch journal; browser budget reserved; original record retained"
                );
                if record.path().extension().is_some_and(|s| s == "json") {
                    let binding: crate::scratch::Owner =
                        serde_json::from_slice(&crate::scratch::private_file(&file, 4096)?)
                            .context(
                                "Linux launch journal schema unknown; browser budget reserved",
                            )?;
                    ensure!(
                        binding.generation == stem
                            && binding.scratch.parent() == Some(parent)
                            && binding.scratch.try_exists()?,
                        "Orphan Linux launch journal; browser budget reserved; original record retained"
                    );
                }
            }
            continue;
        }
        // Unknown/malformed/foreign/legacy records reserve the entire namespace:
        // no new browser may spawn until ownership is proved. Never silently skip.
        let owner = crate::scratch::Owner::read_checked(&path).context(
            "Prior Linux ownership unverifiable; browser budget reserved; original marker retained",
        )?;
        owner.validate_receipt().context(
            "Prior Linux receipt unverifiable; browser budget reserved; original marker retained",
        )?;
        // Never reconstruct signaling authority from PID/env/exe/profile data.
        // A crashed parent's dedicated guardian uses the SAME child-pidfd policy
        // as stop/Drop. Only its durable kernel-settlement receipt permits removal.
        if owner.parent_alive() {
            continue;
        }
        owner.wait_guardian();
        ensure!(
            owner.settled() && owner.guardian_exited(),
            "Prior Linux owned tree is not proved settled; private marker retained"
        );
        // Only scratch with our marker AND a dead owner is disposable. It never
        // contains retained evidence; valid originals live in the CAS/database.
        std::fs::remove_dir_all(path)?;
        owner.retire_journal()?;
    }
    Ok(())
}

#[cfg(all(test, target_os = "linux"))]
mod scratch_tests {
    use super::*;
    use crate::scratch::{self, Owner};
    use std::os::unix::fs::{PermissionsExt, symlink};

    #[test]
    fn recovery_refuses_forged_and_unknown_markers_preserving_bytes() -> Result<()> {
        let base = std::env::var_os("REGRESSEE_WORKER_SCRATCH_PARENT")
            .map(PathBuf::from)
            .and_then(|path| path.parent().map(std::path::Path::to_owned))
            .unwrap_or_else(|| {
                if std::fs::symlink_metadata("/tmp/opencode").is_ok_and(|m| m.is_dir()) {
                    "/tmp/opencode".into()
                } else {
                    "/tmp".into()
                }
            });
        let parent = tempfile::Builder::new()
            .prefix("t")
            .rand_bytes(3)
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir_in(base)?;
        let dead = scratch::create(parent.path())?;
        let mut owner = Owner::new(
            dead.path(),
            "/nonexistent/node".into(),
            "/nonexistent/browser".into(),
        );
        owner.write()?;
        owner.parent = i32::MAX as u32;
        owner.worker = i32::MAX as u32;
        owner.guardian_start = 1;
        assert!(!std::path::Path::new(&format!("/proc/{}", owner.parent)).exists());
        owner.write()?;
        std::fs::write(dead.path().join("settled"), owner.receipt.as_bytes())?;
        std::fs::set_permissions(
            dead.path().join("settled"),
            std::fs::Permissions::from_mode(0o600),
        )?;
        let live = scratch::create(parent.path())?;
        Owner::new(
            live.path(),
            "/nonexistent/node".into(),
            "/nonexistent/browser".into(),
        )
        .write()?;
        let unmarked = scratch::create(parent.path())?;
        std::fs::write(unmarked.path().join("keep"), b"retained")?;
        let foreign = scratch::create(parent.path())?;
        std::fs::write(
            foreign.path().join("owner.json"),
            br#"{"namespace":"regressee-product-v2","parent":2147483647}"#,
        )?;
        let linked = scratch::create(parent.path())?;
        symlink(
            dead.path().join("owner.json"),
            linked.path().join("owner.json"),
        )?;
        let entry_link = parent.path().join("w-symlink");
        symlink(unmarked.path(), &entry_link)?;
        let original = std::fs::read(dead.path().join("owner.json"))?;
        let error = recover_scratch(parent.path()).unwrap_err();
        assert!(format!("{error:#}").contains("browser budget reserved"));
        assert_eq!(std::fs::read(dead.path().join("owner.json"))?, original);
        assert!(live.path().join("owner.json").exists());
        assert!(foreign.path().join("owner.json").exists());
        assert!(
            std::fs::symlink_metadata(linked.path().join("owner.json"))?
                .file_type()
                .is_symlink()
        );
        assert!(
            std::fs::symlink_metadata(entry_link)?
                .file_type()
                .is_symlink()
        );
        assert_eq!(std::fs::read(unmarked.path().join("keep"))?, b"retained");
        Ok(())
    }

    #[test]
    fn recovery_launch_journal_binding_negatives_and_genuine_settlement() -> Result<()> {
        use std::os::unix::fs::OpenOptionsExt;
        let base = scratch::parent()?.parent().unwrap().to_owned();
        for case in [
            "positive",
            "midlaunch-positive",
            "boot",
            "forge",
            "copied",
            "registry-missing",
            "registry-tamper",
            "filesystem",
            "uid",
            "schema",
            "path",
            "guardian-birth",
            "registered-birth-tamper",
            "midlaunch-birth-tamper",
            "generation",
            "legacy",
        ] {
            let parent = tempfile::Builder::new()
                .prefix("t")
                .rand_bytes(3)
                .permissions(std::fs::Permissions::from_mode(0o700))
                .tempdir_in(&base)?;
            let mut space = scratch::create(parent.path())?;
            let mut owner = Owner::new(
                space.path(),
                "/bin/sleep".into(),
                "/nonexistent/browser".into(),
            );
            owner.write()?;
            space.mark_owned();
            let receipt_path = space.path().join("settled");
            let receipt = fs_file(&receipt_path)?;
            let mut command = std::process::Command::new("/bin/sleep");
            command.arg("30");
            let launch = crate::linux_guardian::Launch::prepare_registered(
                &mut command,
                &receipt,
                owner.secret(),
                &owner.registration()?,
            )?;
            let mut child = launch.spawn()?;
            let (control, _) = launch.started()?;
            owner.worker = child.id();
            owner.guardian_start = scratch::birth(child.id()).context("test guardian birth")?;
            owner.write()?;
            control.stop();
            assert!(child.wait()?.success());
            assert!(owner.settled());
            assert!(owner.guardian_exited());
            let registry = parent
                .path()
                .join(".launches")
                .join(format!("{}.json", owner.generation));
            let marker = space.path().join("owner.json");
            let mut value: serde_json::Value = serde_json::from_slice(&std::fs::read(&marker)?)?;
            match case {
                "boot" => value["boot"] = serde_json::json!(uuid::Uuid::new_v4().to_string()),
                "uid" => value["uid"] = serde_json::json!(owner.uid + 1),
                "schema" => value["unexpected"] = serde_json::json!(1),
                "path" => value["scratch"] = serde_json::json!(parent.path().join("w-other0")),
                "guardian-birth" => {
                    value["guardian_start"] = serde_json::json!(owner.guardian_start + 1)
                }
                "generation" => {
                    value["generation"] = serde_json::json!(uuid::Uuid::new_v4().to_string())
                }
                "legacy" => value["namespace"] = serde_json::json!("regressee-product-scratch-v4"),
                "filesystem" => value["inode"] = serde_json::json!(owner.inode + 1),
                "forge" => {
                    let mut bytes = std::fs::read(&receipt_path)?;
                    bytes[12..].copy_from_slice(uuid::Uuid::new_v4().to_string().as_bytes());
                    std::fs::write(&receipt_path, bytes)?;
                }
                "copied" => {
                    let mut donor = scratch::create(parent.path())?;
                    let mut other = Owner::new(
                        donor.path(),
                        "/bin/sleep".into(),
                        "/nonexistent/browser".into(),
                    );
                    other.write()?;
                    donor.mark_owned();
                    let donor_path = donor.path().join("settled");
                    let donor_receipt = fs_file(&donor_path)?;
                    let mut command = std::process::Command::new("/bin/sleep");
                    command.arg("30");
                    let launch = crate::linux_guardian::Launch::prepare_registered(
                        &mut command,
                        &donor_receipt,
                        other.secret(),
                        &other.registration()?,
                    )?;
                    let mut child = launch.spawn()?;
                    let (control, _) = launch.started()?;
                    other.worker = child.id();
                    other.guardian_start =
                        scratch::birth(child.id()).context("donor guardian birth")?;
                    other.write()?;
                    control.stop();
                    assert!(child.wait()?.success());
                    assert!(other.settled());
                    // Genuinely kernel-settled, but a DIFFERENT registered launch.
                    std::fs::copy(&donor_path, &receipt_path)?;
                    drop(donor);
                }
                "midlaunch-positive" => {
                    value["worker"] = serde_json::json!(0);
                    value["guardian_start"] = serde_json::json!(0);
                }
                "registered-birth-tamper" | "midlaunch-birth-tamper" => {
                    if case == "midlaunch-birth-tamper" {
                        value["worker"] = serde_json::json!(0);
                        value["guardian_start"] = serde_json::json!(0);
                    }
                    let file = parent
                        .path()
                        .join(".launches")
                        .join(format!("{}.birth", owner.generation));
                    let mut bytes = std::fs::read(&file)?;
                    bytes[4..].copy_from_slice(&(owner.guardian_start + 1).to_ne_bytes());
                    std::fs::write(file, bytes)?;
                }
                "registry-missing" => std::fs::remove_file(&registry)?,
                "registry-tamper" => {
                    let mut binding: serde_json::Value =
                        serde_json::from_slice(&std::fs::read(&registry)?)?;
                    binding["browser_root"] = serde_json::json!("/changed/browser");
                    std::fs::write(&registry, serde_json::to_vec(&binding)?)?;
                }
                _ => (),
            }
            std::fs::write(&marker, serde_json::to_vec(&value)?)?;
            let original = std::fs::read(&marker)?;
            let original_receipt = std::fs::read(&receipt_path)?;
            let result = recover_scratch(parent.path());
            if case == "positive" || case == "midlaunch-positive" {
                result?; // Live exact parent is left alone; cleanup proves tree.
                space.cleanup();
                assert!(!space.path().exists());
            } else {
                let cause = format!("{:#}", result.unwrap_err());
                assert!(cause.contains("browser budget reserved"), "{case}: {cause}");
                let expected = match case {
                    "boot" => "boot binding mismatch",
                    "forge" | "copied" | "midlaunch-birth-tamper" => "commitment/birth mismatch",
                    "registry-missing" | "generation" => "registry",
                    "registry-tamper" => "registry/marker binding mismatch",
                    "filesystem" => "filesystem binding mismatch",
                    "uid" => "UID binding mismatch",
                    "schema" => "schema invalid/unknown/legacy",
                    "path" => "scratch path binding mismatch",
                    "guardian-birth" | "registered-birth-tamper" => "birth/marker binding mismatch",
                    _ => "namespace unknown/legacy",
                };
                assert!(
                    cause.contains(expected)
                        || (case == "registry-missing"
                            && cause.contains("Incomplete Linux launch journal")),
                    "{case}: {cause}"
                );
                println!(
                    "{}",
                    serde_json::json!({"case":case,"refused":true,"cause":cause,"marker_and_receipt_preserved":true})
                );
                assert_eq!(
                    std::fs::read(&marker)?,
                    original,
                    "{case} marker bytes changed"
                );
                assert_eq!(
                    std::fs::read(&receipt_path)?,
                    original_receipt,
                    "{case} receipt bytes changed"
                );
            }
        }
        fn fs_file(path: &std::path::Path) -> Result<std::fs::File> {
            Ok(std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)?)
        }
        Ok(())
    }
}
