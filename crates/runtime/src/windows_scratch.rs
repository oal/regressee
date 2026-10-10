// SPDX-License-Identifier: MPL-2.0
//! Windows crash recovery uses an exclusive kernel file lease, never a PID.
use anyhow::{Result, ensure};
use std::{
    fs::{self, File},
    io::{Read, Write},
    os::windows::fs::OpenOptionsExt,
    path::Path,
};
const MARKER: &[u8] = b"regressee-windows-private-scratch-v1\n";
pub(crate) struct Scratch {
    // Release the lease BEFORE TempDir's removal (Windows denies deleting it).
    lease: Option<File>,
    dir: tempfile::TempDir,
}
impl Scratch {
    pub fn path(&self) -> &Path {
        self.dir.path()
    }
    pub fn create() -> Result<Self> {
        let parent = std::env::var_os("REGRESSEE_SCRATCH_PARENT")
            .map(std::path::PathBuf::from)
            .unwrap_or_else(|| std::env::temp_dir().join("regressee-private"));
        ensure!(
            parent.as_os_str().len() < 160,
            "Private scratch path is too long; configure a short owner-private folder"
        );
        crate::distribution::private_directory(&parent)?;
        for entry in fs::read_dir(&parent)?.take(4096) {
            let entry = entry?;
            let path = entry.path();
            let meta = fs::symlink_metadata(&path)?;
            if !entry.file_name().to_string_lossy().starts_with("w-")
                || !meta.is_dir()
                || !regressee_core::platform::ordinary(&meta)
            {
                continue;
            }
            let marker = path.join("owner");
            let Ok(meta) = fs::symlink_metadata(&marker) else {
                continue;
            };
            if !meta.is_file()
                || !regressee_core::platform::ordinary(&meta)
                || meta.len() != MARKER.len() as u64
            {
                continue;
            }
            let mut bytes = Vec::new();
            File::open(marker)?.take(128).read_to_end(&mut bytes)?;
            if bytes != MARKER {
                continue;
            }
            let lease = path.join("lease");
            let Ok(meta) = fs::symlink_metadata(&lease) else {
                continue;
            };
            if !meta.is_file() || !regressee_core::platform::ordinary(&meta) {
                continue;
            }
            // Sharing violation means a live owner. No stale numeric PID decisions.
            let Ok(lock) = File::options()
                .read(true)
                .write(true)
                .share_mode(0)
                .open(lease)
            else {
                continue;
            };
            drop(lock);
            // Only delete a marker-confirmed lease-free scratch tree. Never follow
            // links/junctions or touch any retained DB/CAS/runtime generation.
            if ordinary_tree(&path, 0).unwrap_or(false) {
                let _ = fs::remove_dir_all(path);
            }
        }
        let dir = tempfile::Builder::new().prefix("w-").tempdir_in(&parent)?;
        crate::distribution::private_directory(dir.path())?;
        let lease = File::options()
            .create_new(true)
            .read(true)
            .write(true)
            .share_mode(0)
            .open(dir.path().join("lease"))?;
        let mut marker = File::options()
            .create_new(true)
            .write(true)
            .open(dir.path().join("owner"))?;
        marker.write_all(MARKER)?;
        marker.sync_all()?;
        Ok(Self {
            lease: Some(lease),
            dir,
        })
    }
}
fn ordinary_tree(path: &Path, depth: u32) -> Result<bool> {
    if depth > 32 {
        return Ok(false);
    }
    for (i, entry) in fs::read_dir(path)?.enumerate() {
        if i >= 20000 {
            return Ok(false);
        }
        let path = entry?.path();
        let meta = fs::symlink_metadata(&path)?;
        if !regressee_core::platform::ordinary(&meta) {
            return Ok(false);
        }
        if meta.is_dir() {
            if !ordinary_tree(&path, depth + 1)? {
                return Ok(false);
            }
        } else if !meta.is_file() {
            return Ok(false);
        }
    }
    Ok(true)
}
impl Drop for Scratch {
    fn drop(&mut self) {
        self.lease.take();
    }
}
