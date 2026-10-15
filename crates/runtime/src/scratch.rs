// SPDX-License-Identifier: MPL-2.0
//! Linux-only short, per-user product scratch. Never inherits TMPDIR and never
//! scans the caller's temporary directory. Not a cross-platform qualification.
use anyhow::{Context, Result, ensure};
use std::{
    os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Component, Path, PathBuf},
};

pub(crate) const NAMESPACE: &str = "regressee-product-scratch-v5";
// Linux sockaddr_un has 108 bytes including NUL. Leave 58 bytes for Chromium/
// Playwright's profile/socket suffix, and 9 for /w-XXXXXX. Fail, don't truncate.
pub(crate) const MAX_PARENT_BYTES: usize = 40;
pub(crate) const MAX_SCRATCH_BYTES: usize = 49;

thread_local! {
    // Parent-only reentrancy for error-path Scratch::drop during launch. Never
    // inspected/dropped in either raw-fork child. Separate processes use flock.
    static HELD: std::cell::RefCell<Vec<PathBuf>> = const { std::cell::RefCell::new(Vec::new()) };
}
pub(crate) struct Lifecycle {
    file: Option<std::fs::File>,
    parent: PathBuf,
}
pub(crate) fn lifecycle(parent: &Path) -> Result<Lifecycle> {
    use std::os::fd::AsRawFd;
    if HELD.with(|paths| paths.borrow().iter().any(|p| p == parent)) {
        return Ok(Lifecycle {
            file: None,
            parent: parent.into(),
        });
    }
    prepare_parent(parent)?;
    let file = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .mode(0o600)
        .custom_flags(libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(parent.join(".launch-lock"))?;
    let meta = file.metadata()?;
    ensure!(
        meta.is_file() && meta.uid() == uid() && meta.mode() & 0o7777 == 0o600 && meta.len() == 0,
        "Linux launch lifecycle lock ownership/mode/type invalid; browser budget reserved"
    );
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(15);
    loop {
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
            break;
        }
        let error = std::io::Error::last_os_error();
        ensure!(
            matches!(error.raw_os_error(), Some(libc::EWOULDBLOCK | libc::EINTR)),
            "Linux launch lifecycle lock failed: {error}"
        );
        ensure!(
            std::time::Instant::now() < deadline,
            "Linux launch lifecycle busy; browser budget reserved"
        );
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    HELD.with(|paths| paths.borrow_mut().push(parent.into()));
    Ok(Lifecycle {
        file: Some(file),
        parent: parent.into(),
    })
}
impl Drop for Lifecycle {
    fn drop(&mut self) {
        if self.file.is_some() {
            HELD.with(|paths| paths.borrow_mut().retain(|p| p != &self.parent));
            // Closing this exact open-file description releases the flock. Raw
            // guardian/Node close all unneeded inherited descriptors as well.
            self.file.take();
        }
    }
}

fn uid() -> u32 {
    unsafe { libc::geteuid() }
}

pub(crate) fn parent() -> Result<PathBuf> {
    let path = parent_path(std::env::var_os("REGRESSEE_WORKER_SCRATCH_PARENT").map(PathBuf::from));
    prepare_parent(&path)?;
    Ok(path)
}

fn parent_path(override_path: Option<PathBuf>) -> PathBuf {
    // Product defaults must not depend on another application's directories or
    // the development environment. Keep the path short for Linux socket limits.
    override_path.unwrap_or_else(|| Path::new("/tmp").join(format!("rg2-{}", uid())))
}

pub(crate) fn prepare_parent(path: &Path) -> Result<()> {
    ensure!(
        path.is_absolute(),
        "worker scratch parent must be absolute: {}",
        path.display()
    );
    ensure!(
        path.as_os_str().as_encoded_bytes().len() <= MAX_PARENT_BYTES,
        "worker scratch parent exceeds {MAX_PARENT_BYTES}-byte Linux socket budget: {}",
        path.display()
    );
    ensure!(
        path.components()
            .all(|c| matches!(c, Component::RootDir | Component::Normal(_)))
            && path.parent().is_some(),
        "worker scratch parent must be a normalized non-root path: {}",
        path.display()
    );
    // No recursive creation through a symlink. Ancestors must be trusted or a
    // sticky shared temporary directory; the namespace itself MUST be ours/0700.
    let mut current = PathBuf::new();
    for component in path.components() {
        current.push(component);
        if current == path && !current.try_exists()? {
            match std::fs::DirBuilder::new().mode(0o700).create(&current) {
                Ok(()) => (),
                Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                Err(e) => {
                    return Err(e).with_context(|| {
                        format!("create worker scratch parent {}", path.display())
                    });
                }
            }
        }
        let meta = std::fs::symlink_metadata(&current)
            .with_context(|| format!("inspect worker scratch path {}", current.display()))?;
        ensure!(
            !meta.file_type().is_symlink(),
            "worker scratch path is a symlink: {}",
            current.display()
        );
        ensure!(
            meta.is_dir(),
            "worker scratch path is not a directory: {}",
            current.display()
        );
        if current == path {
            ensure!(
                meta.uid() == uid(),
                "worker scratch parent has foreign ownership: {}",
                path.display()
            );
            ensure!(
                meta.mode() & 0o7777 == 0o700,
                "worker scratch parent must have mode 0700: {}",
                path.display()
            );
        } else {
            // A foreign owner's arbitrary sticky directory is not trusted: its
            // owner can remove our entries. /tmp is the conventional OS-managed
            // shared root (also in rootless UID-mapped Linux harnesses).
            let sticky_shared = meta.mode() & 0o1000 != 0
                && meta.mode() & 0o002 != 0
                && (meta.uid() == 0 || meta.uid() == uid() || current == Path::new("/tmp"));
            ensure!(
                sticky_shared
                    || ((meta.uid() == 0 || meta.uid() == uid()) && meta.mode() & 0o022 == 0),
                "worker scratch ancestor has unsafe ownership/permissions: {}",
                current.display()
            );
        }
    }
    ensure!(
        path.canonicalize()? == path,
        "worker scratch parent is not canonical: {}",
        path.display()
    );
    Ok(())
}

pub(crate) struct Scratch {
    dir: Option<tempfile::TempDir>,
    path: PathBuf,
    registered: bool,
}
impl Scratch {
    pub(crate) fn path(&self) -> &Path {
        &self.path
    }
    pub(crate) fn mark_owned(&mut self) {
        self.registered = true;
    }
    pub(crate) fn cleanup(&mut self) {
        let _lifecycle = self.path.parent().and_then(|p| lifecycle(p).ok());
        let owner = if self.registered {
            Owner::read(&self.path)
        } else {
            None
        };
        let valid = _lifecycle.is_some()
            && self
                .path
                .parent()
                .is_some_and(|p| prepare_parent(p).is_ok())
            && private_dir(&self.path)
            && (!self.registered
                || owner.as_ref().is_some_and(|o| {
                    o.parent == std::process::id() && o.settled() && o.guardian_exited()
                }));
        if let Some(dir) = self.dir.take() {
            if valid {
                if dir.close().is_ok()
                    && let Some(owner) = owner
                {
                    let _ = owner.retire_journal();
                }
            } else {
                let _ = dir.keep();
            }
        }
    }
}
impl Drop for Scratch {
    fn drop(&mut self) {
        self.cleanup();
    }
}

pub(crate) fn create(parent: &Path) -> Result<Scratch> {
    prepare_parent(parent)?;
    let dir = tempfile::Builder::new()
        .prefix("w-")
        .permissions(std::fs::Permissions::from_mode(0o700))
        .rand_bytes(6)
        .tempdir_in(parent)
        .context("create private Linux worker scratch")?;
    ensure!(
        dir.path().as_os_str().as_encoded_bytes().len() <= MAX_SCRATCH_BYTES,
        "worker scratch exceeds {MAX_SCRATCH_BYTES}-byte Linux socket budget: {}",
        dir.path().display()
    );
    ensure!(
        private_dir(dir.path()),
        "worker scratch must be owned mode 0700: {}",
        dir.path().display()
    );
    let path = dir.path().to_path_buf();
    Ok(Scratch {
        dir: Some(dir),
        path,
        registered: false,
    })
}

pub(crate) fn private_dir(path: &Path) -> bool {
    std::fs::symlink_metadata(path)
        .is_ok_and(|m| m.is_dir() && m.uid() == uid() && m.mode() & 0o7777 == 0o700)
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Owner {
    pub namespace: String,
    pub uid: u32,
    pub scratch: PathBuf,
    pub parent: u32,
    pub worker: u32,
    pub worker_exe: PathBuf,
    pub browser_root: PathBuf,
    pub parent_start: u64,
    pub guardian_start: u64,
    pub boot: String,
    pub receipt: String,
    pub generation: String,
    pub device: u64,
    pub inode: u64,
    // Never serialized, logged, exported, or retained in the journal/marker.
    // The guardian alone publishes this preimage after kernel ECHILD.
    #[serde(skip)]
    secret: zeroize::Zeroizing<String>,
}

impl Owner {
    pub(crate) fn new(scratch: &Path, worker_exe: PathBuf, browser_root: PathBuf) -> Self {
        use sha2::{Digest, Sha256};
        let secret = uuid::Uuid::new_v4().to_string();
        let metadata = std::fs::symlink_metadata(scratch).expect("new private scratch exists");
        Self {
            namespace: NAMESPACE.into(),
            uid: uid(),
            scratch: scratch.into(),
            parent: std::process::id(),
            worker: 0,
            worker_exe,
            browser_root,
            parent_start: birth(std::process::id()).unwrap_or(0),
            guardian_start: 0,
            boot: std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
                .unwrap_or_default()
                .trim()
                .into(),
            receipt: hex::encode(Sha256::digest(secret.as_bytes())),
            generation: uuid::Uuid::new_v4().to_string(),
            device: metadata.dev(),
            inode: metadata.ino(),
            secret: zeroize::Zeroizing::new(secret),
        }
    }
    pub(crate) fn secret(&self) -> &[u8] {
        self.secret.as_bytes()
    }
    fn journal(&self) -> PathBuf {
        self.scratch.parent().unwrap().join(".launches")
    }
    fn journal_file(&self, suffix: &str) -> PathBuf {
        self.journal()
            .join(format!("{}.{}", self.generation, suffix))
    }
    pub(crate) fn registration(&self) -> Result<std::fs::File> {
        Ok(std::fs::OpenOptions::new()
            .write(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(self.journal_file("birth"))?)
    }
    // Called ONLY after receipt + exited guardian proof and successful scratch
    // removal. A crash in this retirement window leaves a conservative orphan
    // journal refusal, never a reconstructed PID signaling authority.
    pub(crate) fn retire_journal(&self) -> Result<()> {
        ensure!(
            uuid::Uuid::parse_str(&self.generation).is_ok(),
            "invalid journal generation"
        );
        std::fs::remove_file(self.journal_file("json"))?;
        std::fs::remove_file(self.journal_file("birth"))?;
        std::fs::File::open(self.journal())?.sync_all()?;
        Ok(())
    }
    pub(crate) fn write(&self) -> Result<()> {
        use std::io::Write;
        if !self.scratch.join("owner.json").try_exists()? {
            ensure!(
                self.worker == 0 && self.guardian_start == 0,
                "initial launch already registered"
            );
            let journal = self.journal();
            if !journal.try_exists()? {
                match std::fs::DirBuilder::new().mode(0o700).create(&journal) {
                    Ok(()) => (),
                    Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => (),
                    Err(e) => return Err(e.into()),
                }
            }
            ensure!(
                private_dir(&journal),
                "launch journal ownership/mode invalid"
            );
            let mut file = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(self.journal_file("json"))?;
            file.write_all(&serde_json::to_vec(self)?)?;
            file.sync_all()?;
            std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(self.journal_file("birth"))?
                .sync_all()?;
            std::fs::File::open(journal)?.sync_all()?;
        }
        // Atomic marker replacement avoids partial crash records. Only our
        // unique scratch's marker is replaced; existing evidence is never here.
        let pending = self.scratch.join("owner.pending");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&pending)?;
        file.write_all(&serde_json::to_vec(self)?)?;
        file.sync_all()?;
        std::fs::rename(pending, self.scratch.join("owner.json"))?;
        std::fs::File::open(&self.scratch)?.sync_all()?;
        Ok(())
    }
    pub(crate) fn read(path: &Path) -> Option<Self> {
        Self::read_checked(path).ok()
    }
    pub(crate) fn read_checked(path: &Path) -> Result<Self> {
        let owner = Self::read_marker(path)?;
        owner.trusted()
    }
    fn read_marker(path: &Path) -> Result<Self> {
        let name = path
            .file_name()
            .and_then(|s| s.to_str())
            .context("invalid Linux scratch name")?;
        ensure!(
            private_dir(path),
            "Linux scratch ownership/mode/type invalid"
        );
        ensure!(
            name.len() == 8
                && name.starts_with("w-")
                && name[2..].bytes().all(|c| c.is_ascii_alphanumeric())
                && path.as_os_str().as_encoded_bytes().len() <= MAX_SCRATCH_BYTES,
            "invalid Linux scratch name/length"
        );
        prepare_parent(path.parent().context("Linux scratch parent missing")?)?;
        let marker = path.join("owner.json");
        let bytes = private_file(&marker, 4096)
            .context("Linux ownership marker missing/foreign/mode/type invalid")?;
        let owner: Self = serde_json::from_slice(&bytes)
            .context("Linux ownership marker schema invalid/unknown/legacy")?;
        ensure!(
            owner.namespace == NAMESPACE,
            "Linux ownership marker namespace unknown/legacy"
        );
        ensure!(
            owner.uid == uid(),
            "Linux ownership marker UID binding mismatch"
        );
        ensure!(
            owner.scratch == path,
            "Linux ownership marker scratch path binding mismatch"
        );
        ensure!(
            owner.parent > 1
                && owner.parent <= i32::MAX as u32
                && owner.worker <= i32::MAX as u32
                && owner.worker_exe.is_absolute()
                && owner.browser_root.is_absolute()
                && owner.browser_root.parent().is_some()
                && owner.parent_start > 0
                && uuid::Uuid::parse_str(&owner.boot).is_ok()
                && owner.receipt.len() == 64
                && owner.receipt.bytes().all(|b| b.is_ascii_hexdigit())
                && uuid::Uuid::parse_str(&owner.generation).is_ok(),
            "Linux ownership marker launch fields invalid"
        );
        Ok(owner)
    }
    fn trusted(mut self) -> Result<Self> {
        ensure!(
            std::fs::read_to_string("/proc/sys/kernel/random/boot_id")?.trim() == self.boot,
            "Linux launch boot binding mismatch; marker retained"
        );
        let meta = std::fs::symlink_metadata(&self.scratch)?;
        ensure!(
            meta.dev() == self.device && meta.ino() == self.inode,
            "Linux launch scratch filesystem binding mismatch; marker retained"
        );
        ensure!(
            private_dir(&self.journal()),
            "Linux launch registry missing/foreign; marker retained"
        );
        let bytes = private_file(&self.journal_file("json"), 4096)
            .context("Linux launch registry missing/invalid; marker retained")?;
        let registered: Self =
            serde_json::from_slice(&bytes).context("Linux launch registry schema invalid")?;
        // Compare every parent-owned binding, not just a writable UUID. The
        // separately opened birth file is written by the guardian BEFORE clone.
        let worker = self.worker;
        let birth = self.guardian_start;
        self.worker = 0;
        self.guardian_start = 0;
        ensure!(
            serde_json::to_vec(&self)? == serde_json::to_vec(&registered)?,
            "Linux launch registry/marker binding mismatch; marker retained"
        );
        let identity = private_file(&self.journal_file("birth"), 12)
            .context("Linux guardian registration missing/invalid; marker retained")?;
        ensure!(
            identity.len() == 12,
            "Linux guardian registration incomplete; marker retained"
        );
        let guardian = u32::from_ne_bytes(identity[..4].try_into()?);
        let guardian_birth = u64::from_ne_bytes(identity[4..].try_into()?);
        ensure!(
            guardian > 1
                && guardian <= i32::MAX as u32
                && guardian_birth > 0
                && ((worker == 0 && birth == 0) || (worker == guardian && birth == guardian_birth)),
            "Linux guardian birth/marker binding mismatch; marker retained"
        );
        self.worker = guardian;
        self.guardian_start = guardian_birth;
        Ok(self)
    }
    pub(crate) fn settled(&self) -> bool {
        use sha2::{Digest, Sha256};
        if Self::read_checked(&self.scratch).is_err() {
            return false;
        }
        let path = self.scratch.join("settled");
        private_file(&path, 48).is_ok_and(|bytes| {
            bytes.len() == 48
                && private_file(&self.journal_file("birth"), 12)
                    .is_ok_and(|birth| birth == bytes[..12])
                && hex::encode(Sha256::digest(&bytes[12..])) == self.receipt
        })
    }
    pub(crate) fn validate_receipt(&self) -> Result<()> {
        let bytes = private_file(&self.scratch.join("settled"), 48)
            .context("Linux settlement receipt missing/invalid")?;
        ensure!(
            bytes.is_empty() || self.settled(),
            "Linux settlement receipt commitment/birth mismatch; marker retained"
        );
        Ok(())
    }
    pub(crate) fn parent_alive(&self) -> bool {
        std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
            .is_ok_and(|boot| boot.trim() == self.boot)
            && birth(self.parent) == Some(self.parent_start)
    }
    pub(crate) fn guardian_exited(&self) -> bool {
        // This is deletion evidence ONLY, never signal authority. A receipt alone
        // must not delete a still-live guardian's namespace (e.g. blocked fsync).
        if self.worker <= 1
            || self.guardian_start == 0
            || !std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
                .is_ok_and(|b| b.trim() == self.boot)
        {
            return false;
        }
        match std::fs::read_to_string(format!("/proc/{}/stat", self.worker)) {
            Ok(stat) => {
                let Some(end) = stat.rfind(')') else {
                    return false;
                };
                let fields: Vec<_> = stat[end + 2..].split_whitespace().collect();
                fields
                    .get(19)
                    .and_then(|birth| birth.parse::<u64>().ok())
                    .is_some_and(|birth| birth != self.guardian_start)
                    || fields
                        .first()
                        .is_some_and(|state| matches!(*state, "Z" | "X"))
            }
            Err(error) => error.kind() == std::io::ErrorKind::NotFound,
        }
    }
    pub(crate) fn wait_guardian(&self) {
        use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
        // Recovery NEVER signals. A PID-reused guardian cannot even become our
        // wait target; boot + unchanged kernel birth bind the stable handle.
        if self.guardian_exited()
            || birth(self.worker) != Some(self.guardian_start)
            || !std::fs::read_to_string("/proc/sys/kernel/random/boot_id")
                .is_ok_and(|boot| boot.trim() == self.boot)
        {
            return;
        }
        let raw = unsafe { libc::syscall(libc::SYS_pidfd_open, self.worker, 0) } as i32;
        if raw < 0 {
            return;
        }
        let handle = unsafe { OwnedFd::from_raw_fd(raw) };
        if birth(self.worker) != Some(self.guardian_start) {
            return;
        }
        let mut fd = libc::pollfd {
            fd: handle.as_raw_fd(),
            events: libc::POLLIN,
            revents: 0,
        };
        unsafe {
            libc::poll(&mut fd, 1, 6000);
        }
    }
}

pub(crate) fn private_file(path: &Path, limit: u64) -> Result<Vec<u8>> {
    use std::io::Read;
    let mut file = std::fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)?;
    let meta = file.metadata()?;
    ensure!(
        meta.is_file()
            && meta.uid() == uid()
            && meta.mode() & 0o7777 == 0o600
            && meta.len() <= limit,
        "launch journal file ownership/mode/size invalid"
    );
    let mut bytes = Vec::new();
    file.by_ref().take(limit + 1).read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= limit,
        "launch journal file exceeds bound"
    );
    Ok(bytes)
}

pub(crate) fn birth(pid: u32) -> Option<u64> {
    let stat = std::fs::read_to_string(format!("/proc/{pid}/stat")).ok()?;
    stat[stat.rfind(')')? + 2..]
        .split_whitespace()
        .nth(19)?
        .parse()
        .ok()
}
#[cfg(test)]
pub(crate) fn legacy_unsettled(path: &Path) -> bool {
    let valid_name = path
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| {
            name.len() == 8
                && name.starts_with("w-")
                && name[2..].bytes().all(|byte| byte.is_ascii_alphanumeric())
        });
    if !valid_name
        || !private_dir(path)
        || path.as_os_str().as_encoded_bytes().len() > MAX_SCRATCH_BYTES
    {
        return false;
    }
    let marker = path.join("owner.json");
    if !std::fs::symlink_metadata(&marker).is_ok_and(|m| {
        m.is_file() && m.uid() == uid() && m.mode() & 0o7777 == 0o600 && m.len() <= 4096
    }) {
        return false;
    }
    std::fs::read(marker)
        .ok()
        .and_then(|bytes| serde_json::from_slice::<serde_json::Value>(&bytes).ok())
        .is_some_and(|owner| owner["namespace"] == "regressee-product-scratch-v3")
}

/// Historical diagnostic ONLY. Never used as signal/deletion authority: Chromium
/// rewrites environment storage. Retained for the exact independent regression.
#[allow(dead_code)]
pub(crate) fn owns_process(pid: i32, scratch: &Path, root: &Path, exact_exe: bool) -> bool {
    let proc = PathBuf::from(format!("/proc/{pid}"));
    if !std::fs::metadata(&proc).is_ok_and(|m| m.uid() == uid()) {
        return false;
    }
    let Ok(exe) = std::fs::read_link(proc.join("exe")) else {
        return false;
    };
    if if exact_exe {
        exe != root
    } else {
        !exe.starts_with(root)
    } {
        return false;
    }
    let expected = [
        b"TMPDIR=".as_slice(),
        scratch.as_os_str().as_encoded_bytes(),
    ]
    .concat();
    std::fs::read(proc.join("environ"))
        .is_ok_and(|env| env.split(|b| *b == 0).any(|v| v == expected))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    fn test_base() -> PathBuf {
        std::env::var_os("REGRESSEE_WORKER_SCRATCH_PARENT")
            .map(PathBuf::from)
            .and_then(|path| path.parent().map(Path::to_owned))
            .unwrap_or_else(|| PathBuf::from("/tmp"))
    }
    #[test]
    fn scratch_parent_defaults_to_own_namespace_and_preserves_explicit_overrides() {
        let default = parent_path(None);
        assert_eq!(default, Path::new("/tmp").join(format!("rg2-{}", uid())));
        assert!(default.as_os_str().as_encoded_bytes().len() <= MAX_PARENT_BYTES);
        // Selection neither normalizes nor silently replaces invalid overrides;
        // prepare_parent must still reject them before launch.
        for path in [
            PathBuf::from("/tmp/custom-private"),
            PathBuf::from("relative"),
            PathBuf::from("/tmp").join("x".repeat(50)),
        ] {
            assert_eq!(parent_path(Some(path.clone())), path);
        }
    }
    #[test]
    fn unsafe_override_is_rejected_without_permission_repair_or_fallback() -> Result<()> {
        let base = tempfile::Builder::new()
            .prefix("t")
            .rand_bytes(1)
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir_in(test_base())?;
        let parent = base.path().join("p");
        let sentinel = base.path().join("keep");
        std::fs::write(&sentinel, b"unrelated directory contents")?;
        for mode in [0o775, 0o777] {
            std::fs::set_permissions(base.path(), std::fs::Permissions::from_mode(mode))?;
            let selected = parent_path(Some(parent.clone()));
            let error = prepare_parent(&selected).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("ancestor has unsafe ownership/permissions")
            );
            assert!(!parent.try_exists()?);
            assert_eq!(std::fs::metadata(base.path())?.mode() & 0o7777, mode);
            assert_eq!(std::fs::read(&sentinel)?, b"unrelated directory contents");
        }
        std::fs::set_permissions(base.path(), std::fs::Permissions::from_mode(0o700))?;
        prepare_parent(&parent)?;
        assert!(private_dir(&parent));
        Ok(())
    }
    #[test]
    fn lifecycle_reentrancy_and_concurrent_cleanup_are_serialized() -> Result<()> {
        let parent = tempfile::Builder::new()
            .prefix("t")
            .rand_bytes(3)
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir_in(test_base())?;
        let held = lifecycle(parent.path())?;
        drop(lifecycle(parent.path())?); // Error-path nested cleanup must not deadlock.
        let path = parent.path().to_owned();
        let (started, ready) = std::sync::mpsc::channel();
        let (completed, done) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            started.send(()).unwrap();
            let acquired = lifecycle(&path).is_ok();
            completed.send(acquired).unwrap();
        });
        ready.recv_timeout(std::time::Duration::from_secs(2))?;
        assert!(matches!(
            done.recv_timeout(std::time::Duration::from_millis(20)),
            Err(std::sync::mpsc::RecvTimeoutError::Timeout)
        ));
        drop(held);
        assert!(done.recv_timeout(std::time::Duration::from_secs(2))?);
        thread.join().unwrap();
        Ok(())
    }
    #[test]
    fn scratch_parent_specific_diagnostics() -> Result<()> {
        let base = tempfile::Builder::new()
            .prefix("t")
            .rand_bytes(1)
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir_in(test_base())?;
        let parent = base.path().join("p");
        prepare_parent(&parent)?;
        let scratch = create(&parent)?;
        assert!(private_dir(scratch.path()));
        assert!(scratch.path().as_os_str().as_encoded_bytes().len() <= MAX_SCRATCH_BYTES);
        let cause = |path: &Path| prepare_parent(path).unwrap_err().to_string();
        assert!(cause(Path::new("relative")).contains("must be absolute"));
        assert!(
            cause(&base.path().join("x".repeat(50)))
                .contains("exceeds 40-byte Linux socket budget")
        );
        let link = base.path().join("l");
        symlink(&parent, &link)?;
        assert!(cause(&link).contains("is a symlink"));
        assert!(cause(&link.join("c")).contains("is a symlink"));
        assert!(cause(&base.path().join("../b")).contains("normalized non-root path"));
        let file = base.path().join("f");
        std::fs::write(&file, b"kept")?;
        assert!(cause(&file).contains("not a directory"));
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o755))?;
        assert!(cause(&parent).contains("must have mode 0700"));
        std::fs::set_permissions(&parent, std::fs::Permissions::from_mode(0o700))?;
        // An existing root-owned short directory is rejected before mode, no chown/sudo.
        if uid() != 0 && std::fs::metadata("/etc").is_ok_and(|m| m.uid() == 0) {
            assert!(cause(Path::new("/etc")).contains("foreign ownership"));
        }
        let dangling = base.path().join("d");
        symlink(base.path().join("absent"), &dangling)?;
        assert!(cause(&dangling).contains("is a symlink"));
        std::fs::set_permissions(base.path(), std::fs::Permissions::from_mode(0o777))?;
        assert!(cause(&parent).contains("ancestor has unsafe ownership/permissions"));
        std::fs::set_permissions(base.path(), std::fs::Permissions::from_mode(0o700))?;
        Ok(())
    }

    #[test]
    fn final_cleanup_preserves_invalid_registered_owner() -> Result<()> {
        let parent = tempfile::Builder::new()
            .prefix("t")
            .rand_bytes(3)
            .permissions(std::fs::Permissions::from_mode(0o700))
            .tempdir_in(test_base())?;
        let mut scratch = create(parent.path())?;
        let mut owner = Owner::new(
            scratch.path(),
            "/nonexistent/node".into(),
            "/nonexistent/browser".into(),
        );
        owner.write()?;
        scratch.mark_owned();
        let path = scratch.path().to_path_buf();
        owner.namespace = "unrelated".into();
        owner.write()?;
        drop(scratch);
        assert!(
            path.join("owner.json").exists(),
            "Drop bypassed the ownership policy"
        );
        // Only the unique test fixture's parent TempDir removes this inert entry.
        Ok(())
    }

    #[test]
    fn unproved_and_legacy_markers_cannot_authorize_cleanup() -> Result<()> {
        let parent = parent()?;
        let mut scratch = create(&parent)?;
        let owner = Owner::new(
            scratch.path(),
            "/nonexistent/node".into(),
            "/nonexistent/browser".into(),
        );
        owner.write()?;
        scratch.mark_owned();
        let path = scratch.path().to_owned();
        scratch.cleanup();
        assert!(
            path.join("owner.json").exists(),
            "unsettled ownership evidence lost"
        );
        assert!(!owner.settled());
        let mut legacy = serde_json::to_value(&owner)?;
        legacy["namespace"] = serde_json::json!("regressee-product-scratch-v3");
        std::fs::write(path.join("owner.json"), serde_json::to_vec(&legacy)?)?;
        assert!(
            Owner::read(&path).is_none(),
            "legacy matcher must not grant recovery authority"
        );
        assert!(
            legacy_unsettled(&path),
            "legacy namespace must block silent unsafe recapture"
        );
        // Only this inert exact test-owned entry is removed; no processes launched.
        std::fs::remove_dir_all(path)?;
        // This inert test never launches; remove its exact test-created journal
        // only after all refusal/preservation assertions, not as product cleanup.
        std::fs::remove_file(owner.journal_file("json"))?;
        std::fs::remove_file(owner.journal_file("birth"))?;
        Ok(())
    }
}
