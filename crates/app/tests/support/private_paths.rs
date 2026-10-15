// SPDX-License-Identifier: MPL-2.0
//! Explicit caller-selected test directories, never a developer's fixed path.
use anyhow::{Result, ensure};
use std::path::Path;

pub fn private_directory(path: &Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)?;
    ensure!(
        path.is_absolute()
            && path.canonicalize()? == path
            && metadata.is_dir()
            && regressee_core::platform::ordinary(&metadata),
        "test directory must be an absolute physical directory"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure!(
            metadata.uid() == unsafe { libc::geteuid() } && metadata.mode() & 0o7777 == 0o700,
            "test directory must be current-user-owned mode 0700"
        );
    }
    Ok(())
}
