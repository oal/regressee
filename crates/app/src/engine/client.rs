// SPDX-License-Identifier: MPL-2.0
use super::{ipc, ownership};
use crate::protocol::*;
use anyhow::{Context, Result, ensure};
use regressee_storage::AppPaths;
use std::{
    path::PathBuf,
    process::{Command, Stdio},
    time::{Duration, Instant},
};
use uuid::Uuid;

#[derive(Clone)]
pub struct Client {
    pub paths: AppPaths,
    credential: std::sync::Arc<std::sync::Mutex<Option<crate::control::Credential>>>,
}
/// A response that proves admission was refused, unlike a lost transport reply.
#[derive(Debug)]
pub struct Rejected {
    pub code: String,
    pub message: String,
}
impl std::fmt::Display for Rejected {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.code, self.message)
    }
}
impl std::error::Error for Rejected {}
#[derive(Clone)]
pub struct Launcher {
    executable: PathBuf,
}
impl Launcher {
    /// A canonical executable discovered by the installation, never a PATH lookup.
    pub fn current() -> Result<Self> {
        Ok(Self {
            executable: std::env::current_exe()?,
        })
    }
    pub fn sibling() -> Result<Self> {
        let current = std::env::current_exe()?;
        let sibling = current
            .parent()
            .context("Executable directory unavailable")?
            .join(if cfg!(windows) {
                "regressee.exe"
            } else {
                "regressee"
            });
        // Both desktop and CLI can host engine serve, including GUI-only previews.
        Ok(Self {
            executable: if sibling.is_file() { sibling } else { current },
        })
    }
    #[cfg(feature = "test-support")]
    pub fn exact(executable: PathBuf) -> Self {
        Self { executable }
    }
    fn spawn(&self, paths: &AppPaths) -> Result<()> {
        let metadata = std::fs::symlink_metadata(&self.executable)?;
        ensure!(
            self.executable.is_absolute()
                && metadata.is_file()
                && regressee_core::platform::ordinary(&metadata),
            "Unsafe engine executable"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            ensure!(
                metadata.mode() & 0o022 == 0
                    && [0, unsafe { libc::geteuid() }].contains(&metadata.uid()),
                "Engine executable is writable by other users"
            );
            let parent = std::fs::metadata(
                self.executable
                    .parent()
                    .context("Executable parent unavailable")?,
            )?;
            ensure!(
                parent.mode() & 0o022 == 0
                    && [0, unsafe { libc::geteuid() }].contains(&parent.uid()),
                "Engine executable directory is writable by other users"
            );
        }
        let mut command = Command::new(&self.executable);
        command
            .args(["engine", "serve", "--data-dir"])
            .arg(&paths.data)
            .arg("--cache-dir")
            .arg(&paths.cache)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            // setsid is async-signal-safe and detaches the engine from the CLI's
            // controlling terminal/session as well as its foreground group.
            unsafe {
                command.pre_exec(|| {
                    if libc::setsid() == -1 {
                        Err(std::io::Error::last_os_error())
                    } else {
                        Ok(())
                    }
                });
            }
        }
        #[cfg(windows)]
        {
            use std::os::windows::process::CommandExt;
            // DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP |
            // CREATE_BREAKAWAY_FROM_JOB. If a host job forbids breakaway, fail
            // startup rather than claim a daemon that dies with that client.
            command.creation_flags(0x00000008 | 0x00000200 | 0x01000000);
        }
        let mut child = command
            .spawn()
            .context("Cannot start installed engine executable")?;
        std::thread::spawn(move || {
            let _ = child.wait();
        });
        Ok(())
    }
}
impl Client {
    pub fn existing(paths: AppPaths) -> Result<Self> {
        Ok(Self {
            paths: ownership::canonical_paths(&paths, false)?,
            credential: Default::default(),
        })
    }
    pub fn autostart(paths: AppPaths, launcher: Launcher) -> Result<Self> {
        Self::autostart_while(paths, launcher, || true)
    }
    /// Interrupt only this client's startup wait, never the detached engine.
    pub fn autostart_while(
        paths: AppPaths,
        launcher: Launcher,
        keep_waiting: impl Fn() -> bool,
    ) -> Result<Self> {
        let client = Self {
            paths: ownership::canonical_paths(&paths, true)?,
            credential: Default::default(),
        };
        let deadline = Instant::now() + Duration::from_secs(30);
        let mut launched = false;
        let mut replacing_previous = false;
        loop {
            ensure!(
                keep_waiting(),
                "Engine client detached during startup; any launched engine remains independent"
            );
            match client.request(Request::Status) {
                Ok(Response::Status(_)) => return Ok(client),
                Ok(response) => anyhow::bail!("Engine refused startup: {response:?}"),
                Err(error) => {
                    let text = error.to_string();
                    // A stopped previous-version owner leaves its descriptor.
                    // Prove the store is unowned before replacing it; never
                    // delete the lock/descriptor or stop a live owner implicitly.
                    if !launched
                        && text.contains("version_mismatch")
                        && client.previous_endpoint()?
                        && let Ok(owner) = ownership::Ownership::acquire(&client.paths)
                    {
                        drop(owner);
                        launcher.spawn(&client.paths)?;
                        launched = true;
                        replacing_previous = true;
                        continue;
                    }
                    if replacing_previous
                        && text.contains("version_mismatch")
                        && client.previous_endpoint()?
                    {
                        ensure!(
                            Instant::now() < deadline,
                            "Engine upgrade startup deadline exceeded; the previous descriptor was kept"
                        );
                        std::thread::sleep(Duration::from_millis(25));
                        continue;
                    }
                    if text.contains("version_mismatch")
                        || text.contains("Unsafe")
                        || text.contains("private")
                        || text.contains("identity_mismatch")
                        || text.contains("cache_directory_conflict")
                    {
                        return Err(error);
                    }
                    if !launched {
                        launcher.spawn(&client.paths)?;
                        launched = true;
                    }
                    ensure!(
                        Instant::now() < deadline,
                        "Engine startup deadline exceeded; no independent store owner was started"
                    );
                    std::thread::sleep(Duration::from_millis(25));
                }
            }
        }
    }
    fn previous_endpoint(&self) -> Result<bool> {
        let path = self.paths.data.join("engine-v1/endpoint.json");
        ownership::private_directory(path.parent().unwrap())?;
        ownership::private_file(&path, 4096)?;
        let descriptor: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        Ok(matches!(descriptor["version"].as_u64(), Some(6 | 7 | 8)))
    }
    pub fn request(&self, request: Request) -> Result<Response> {
        let credential = self.credential.lock().unwrap().clone();
        // Only explicit, uncredentialed lifecycle Stop has the unchanged old
        // envelope. No status, product operation or authority can cross versions.
        let lifecycle_stop = credential.is_none() && matches!(request, Request::Stop);
        let request = if let Some(credential) = credential {
            let epoch = match &request {
                Request::Submit(s) => s.epoch,
                _ => self.uncredentialed().snapshot()?.epoch,
            };
            Request::Authorized {
                credential,
                epoch,
                request: Box::new(request),
            }
        } else {
            request
        };
        ownership::private_directory(&self.paths.data.join("engine-v1"))?;
        let path = self.paths.data.join("engine-v1/endpoint.json");
        ownership::private_file(&path, 4096)?;
        let descriptor: serde_json::Value = serde_json::from_slice(&std::fs::read(&path)?)?;
        let wire_version = descriptor["version"].as_u64();
        ensure!(
            wire_version == Some(VERSION as u64)
                || (lifecycle_stop && matches!(wire_version, Some(6 | 7 | 8))),
            "version_mismatch: background engine protocol {} differs from this build ({VERSION}). Close older desktop windows and finish active work, then run `regressee engine stop` with the same data/cache directories and reopen the desktop. For unsupported engine versions, stop it using its matching executable. Project data must not be deleted to resolve this.",
            wire_version.map_or_else(|| "unknown".into(), |version| version.to_string())
        );
        let wire_version = wire_version.unwrap() as u32;
        let declared_cache = serde_json::from_value::<PathBuf>(descriptor["cache"].clone()).ok();
        let expected: Uuid = serde_json::from_value(descriptor["engine"].clone())?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        runtime.block_on(async {
            let mut stream = ipc::connect(&self.paths).await?;
             ipc::write(&mut stream, &Request::Hello { version: wire_version }).await?;
            match ipc::read(&mut stream).await? {
                Response::Hello { version, engine } => {
                    ensure!(
                         version == wire_version,
                        "version_mismatch: engine handshake version differs"
                    );
                    if engine == expected {
                        ensure!(declared_cache.as_ref()==Some(&self.paths.cache),"cache_directory_conflict: the active engine uses a different cache directory; attach using its configured cache or stop it before changing cache selection");
                    }
                    if engine != expected {
                        // A restart can publish its new record after this client
                        // opened the old one, but before connect. Re-read only the
                        // private record; never trust an unsolicited peer UUID.
                        ownership::private_file(&path, 4096)?;
                        let current: serde_json::Value =
                            serde_json::from_slice(&std::fs::read(&path)?)?;
                        ensure!(
                             current["version"].as_u64() == Some(wire_version as u64)
                                && current["engine"] == serde_json::to_value(engine)?
                                && serde_json::from_value::<PathBuf>(current["cache"].clone())
                                    .ok()
                                    .as_ref()
                                    == Some(&self.paths.cache),
                            "identity_mismatch: endpoint does not match its private owner record"
                        );
                    }
                }
                response => {
                    anyhow::bail!("version_mismatch: engine handshake rejected: {response:?}")
                }
            }
            ipc::write(&mut stream, &request).await?;
            ipc::read(&mut stream).await
        })
    }
    pub fn with_credential(&self, credential: crate::control::Credential) -> Self {
        Self {
            paths: self.paths.clone(),
            credential: std::sync::Arc::new(std::sync::Mutex::new(Some(credential))),
        }
    }
    pub fn uncredentialed(&self) -> Self {
        Self {
            paths: self.paths.clone(),
            credential: Default::default(),
        }
    }
    pub fn credential(&self) -> Option<crate::control::Credential> {
        self.credential.lock().unwrap().clone()
    }
    pub fn set_credential(&self, credential: Option<crate::control::Credential>) {
        *self.credential.lock().unwrap() = credential;
    }
    pub fn control(&self, command: crate::control::Command) -> Result<Response> {
        match self.request(Request::Control(command))? {
            Response::Error { code, message } => Err(Rejected { code, message }.into()),
            response => Ok(response),
        }
    }
    /// Deliberate GUI bootstrap using private filesystem capability access.
    /// Ordinary request/control/status and uncredentialed() never read this file
    /// or acquire a frontend role. This is coordination, not an OS sandbox.
    pub fn register_frontend(&self) -> Result<crate::control::Credential> {
        ensure!(
            self.credential().is_none(),
            "authority_conflict: frontend bootstrap requires a separate GUI connection"
        );
        let directory = self.paths.data.join("engine-v1");
        ownership::private_directory(&directory)?;
        let path = directory.join("frontend-capability.json");
        ownership::private_file(&path, 4096)?;
        let capability = serde_json::from_slice(&std::fs::read(&path)?)?;
        match self.request(Request::RegisterFrontend { capability })? {
            Response::Credential(c) => Ok(c),
            Response::Error { code, message } => Err(Rejected { code, message }.into()),
            _ => anyhow::bail!("Frontend bootstrap unavailable"),
        }
    }
    pub fn snapshot(&self) -> Result<Snapshot> {
        match self.request(Request::Snapshot)? {
            Response::Status(s) => Ok(s),
            r => anyhow::bail!("{r:?}"),
        }
    }
    pub fn submit(&self, submitted: Submit) -> Result<Job> {
        match self.request(Request::Submit(Box::new(submitted)))? {
            Response::Job(j) => Ok(*j),
            Response::Error { code, message } => Err(Rejected { code, message }.into()),
            r => anyhow::bail!("{r:?}"),
        }
    }
    pub fn job(&self, id: Uuid) -> Result<Job> {
        match self.request(Request::Job { id })? {
            Response::Job(j) => Ok(*j),
            r => anyhow::bail!("{r:?}"),
        }
    }
    /// Disconnect/Ctrl-C never sends cancellation. Cancellation is an explicit RPC.
    pub fn wait(&self, id: Uuid) -> Result<Job> {
        loop {
            let job = self.job(id)?;
            if job.state.terminal() {
                return Ok(job);
            }
            std::thread::sleep(Duration::from_millis(20));
        }
    }
    pub fn execute(&self, operation: Operation, confirmation: Option<Uuid>) -> Result<Job> {
        let snapshot = self.snapshot()?;
        let submitted = Submit {
            request: Uuid::new_v4(),
            engine: snapshot.engine,
            epoch: snapshot.epoch,
            operation,
            confirmation,
            expected_revision: None,
        };
        let job=match self.submit(submitted.clone()) {
            Ok(job)=>job,
            Err(error) if error.is::<Rejected>()=>return Err(error),
            Err(error)=>return Err(error.context(format!("Request {} may be admitted; retry/query that exact ID and input rather than issuing a new mutation",submitted.request))),
        };
        self.wait(job.id)
    }
    pub fn prepare(&self, operation: Operation) -> Result<Confirmation> {
        let job = self.execute(Operation::Prepare(Box::new(operation)), None)?;
        match job.outcome {
            Some(Outcome::Confirmation(c)) => Ok(c),
            other => anyhow::bail!("Cannot prepare confirmation: {other:?}"),
        }
    }
}

#[cfg(test)]
#[path = "client_upgrade_tests.rs"]
mod upgrade_tests;

/// Shared minimal lifecycle parser, also entered before desktop graphics startup.
pub fn lifecycle(args: &[String]) -> Result<Option<String>> {
    if args.first().map(String::as_str) != Some("engine") {
        return Ok(None);
    }
    let command = args
        .get(1)
        .context("Usage: engine serve|status|stop|start [--data-dir DIR] [--cache-dir DIR]")?;
    let mut data = None;
    let mut cache = None;
    let mut i = 2;
    while i < args.len() {
        let value = args
            .get(i + 1)
            .context("Engine option requires a directory")?;
        match args[i].as_str() {
            "--data-dir" => {
                ensure!(data.is_none(), "Duplicate --data-dir");
                data = Some(PathBuf::from(value));
            }
            "--cache-dir" => {
                ensure!(cache.is_none(), "Duplicate --cache-dir");
                cache = Some(PathBuf::from(value));
            }
            _ => anyhow::bail!("Unknown engine option"),
        };
        i += 2;
    }
    let paths = if let Some(data) = data {
        AppPaths {
            cache: cache.unwrap_or_else(|| data.join("cache")),
            data,
        }
    } else {
        let mut paths = AppPaths::discover()?;
        if let Some(cache) = cache {
            paths.cache = cache;
        }
        paths
    };
    match command.as_str() {
        "serve" => {
            super::Server::open(paths)?.run()?;
            Ok(Some("Engine stopped".into()))
        }
        "start" => {
            let client = Client::autostart(paths, Launcher::current()?)?;
            Ok(Some(serde_json::to_string(&client.snapshot()?)?))
        }
        "status" => {
            let client = Client::existing(paths)?;
            Ok(Some(serde_json::to_string(&client.snapshot()?)?))
        }
        "stop" => {
            let client = Client::existing(paths)?;
            match client.request(Request::Stop)? {
                Response::Ok => Ok(Some("Engine stopping".into())),
                r => anyhow::bail!("{r:?}"),
            }
        }
        _ => anyhow::bail!("Unknown engine lifecycle command"),
    }
}
