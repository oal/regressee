// SPDX-License-Identifier: MPL-2.0
use anyhow::{Context, Result, ensure};
use regressee_storage::AppPaths;
use std::{
    fs::{self, File, OpenOptions},
    path::{Path, PathBuf},
};

/// Held through store shutdown. Never unlink a lock file (including a stale one).
pub struct Ownership {
    _lock: File,
    pub paths: AppPaths,
    pub directory: PathBuf,
    frontend_engine: std::sync::Mutex<Option<uuid::Uuid>>,
}

pub fn canonical_paths(paths: &AppPaths, create: bool) -> Result<AppPaths> {
    fn directory(path: &Path, create: bool) -> Result<PathBuf> {
        if !path.exists() && create {
            let mut builder = fs::DirBuilder::new();
            builder.recursive(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            builder.create(path)?;
            #[cfg(windows)]
            regressee_core::platform::protect(path)?;
        }
        let path = fs::canonicalize(path).context("Data/cache directory unavailable")?;
        private_directory(&path)?;
        Ok(path)
    }
    Ok(AppPaths {
        data: directory(&paths.data, create)?,
        cache: directory(&paths.cache, create)?,
    })
}
pub fn private_directory(path: &Path) -> Result<()> {
    let m = fs::symlink_metadata(path)?;
    ensure!(
        m.is_dir() && regressee_core::platform::ordinary(&m),
        "Unsafe engine directory"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure!(
            m.uid() == unsafe { libc::geteuid() } && m.mode() & 0o077 == 0,
            "Engine directory must be owner-private"
        );
    }
    // Validate/protect current-user Windows ACLs, never follow reparse points.
    #[cfg(windows)]
    regressee_core::platform::protect(path)?;
    Ok(())
}
pub fn private_file(path: &Path, maximum: u64) -> Result<()> {
    let m = fs::symlink_metadata(path)?;
    ensure!(
        m.is_file() && regressee_core::platform::ordinary(&m) && m.len() <= maximum,
        "Unsafe engine file"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure!(
            m.uid() == unsafe { libc::geteuid() } && m.mode() & 0o077 == 0 && m.nlink() == 1,
            "Engine file must be owner-private with a single link"
        );
    }
    #[cfg(windows)]
    regressee_core::platform::protect(path)?;
    Ok(())
}
impl Ownership {
    pub fn acquire(paths: &AppPaths) -> Result<Self> {
        let paths = canonical_paths(paths, true)?;
        let directory = paths.data.join("engine-v1");
        if !directory.exists() {
            let mut b = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                b.mode(0o700);
            }
            b.create(&directory).or_else(|e| {
                if e.kind() == std::io::ErrorKind::AlreadyExists {
                    Ok(())
                } else {
                    Err(e)
                }
            })?;
            #[cfg(windows)]
            regressee_core::platform::protect(&directory)?;
        }
        private_directory(&directory)?;
        // Deliberately independent of protocol/journal version directories.
        let path = paths.data.join("engine-owner.lock");
        if path.exists() {
            private_file(&path, 4096)?;
        }
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options
                .mode(0o600)
                .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC);
        }
        let lock = options.open(&path)?;
        private_file(&path, 4096)?;
        lock.try_lock().map_err(|_| {
            anyhow::anyhow!("engine_owned: another owner is active for this data directory")
        })?;
        Ok(Self {
            _lock: lock,
            paths,
            directory,
            frontend_engine: Default::default(),
        })
    }
    pub(super) fn mint_frontend_capability(
        &self,
        engine: uuid::Uuid,
    ) -> Result<crate::control::FrontendCapability> {
        let capability = crate::control::FrontendCapability {
            engine,
            secret: uuid::Uuid::new_v4(),
        };
        write_json(
            &self.directory.join("frontend-capability.json"),
            &capability,
        )?;
        *self.frontend_engine.lock().unwrap() = Some(engine);
        Ok(capability)
    }
}

impl Drop for Ownership {
    fn drop(&mut self) {
        // Still holding the owner lock, after the store/worker has released its
        // ownership reference. A crash may leave a file; the next owner rotates
        // it, and the old incarnation's proof is never accepted by the engine.
        if let Some(engine) = *self.frontend_engine.lock().unwrap() {
            let path = self.directory.join("frontend-capability.json");
            if private_file(&path, 4096).is_ok()
                && std::fs::read(&path)
                    .ok()
                    .and_then(|b| {
                        serde_json::from_slice::<crate::control::FrontendCapability>(&b).ok()
                    })
                    .is_some_and(|c| c.engine == engine)
            {
                let _ = std::fs::remove_file(path);
            }
        }
    }
}

pub fn write_json(path: &Path, value: &impl serde::Serialize) -> Result<()> {
    use std::io::Write;
    if path.exists() {
        private_file(path, crate::protocol::MAX_FRAME as u64)?;
    }
    let bytes = serde_json::to_vec(value)?;
    ensure!(
        bytes.len() <= crate::protocol::MAX_FRAME,
        "Engine journal/result exceeds budget"
    );
    let mut temporary = tempfile::NamedTempFile::new_in(path.parent().context("journal parent")?)?;
    #[cfg(windows)]
    regressee_core::platform::protect(temporary.path())?;
    temporary.write_all(&bytes)?;
    temporary.as_file().sync_all()?;
    temporary.persist(path).map_err(|e| e.error)?;
    #[cfg(unix)]
    File::open(path.parent().unwrap())?.sync_all()?;
    Ok(())
}
