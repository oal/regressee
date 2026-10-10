// SPDX-License-Identifier: MPL-2.0
//! Blocking native filesystem primitives shared by storage and runtime owners.
use anyhow::Result;
#[cfg(unix)]
use anyhow::ensure;
use std::path::Path;
#[cfg(windows)]
#[path = "platform/windows.rs"]
mod windows;

/// Quota-aware bytes available to the calling user, not total volume free space.
pub fn available_bytes(path: &Path) -> Result<u64> {
    #[cfg(unix)]
    {
        let path = std::ffi::CString::new(path.as_os_str().as_encoded_bytes())?;
        let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
        ensure!(
            unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) } == 0,
            "Cannot check available disk space"
        );
        let stat = unsafe { stat.assume_init() };
        Ok(
            (u128::from(stat.f_bavail) * u128::from(stat.f_frsize)).min(u128::from(u64::MAX))
                as u64,
        )
    }
    #[cfg(windows)]
    {
        windows::available_bytes(path)
    }
    #[cfg(not(any(unix, windows)))]
    {
        let _ = path;
        anyhow::bail!("Unsupported storage platform")
    }
}

/// Reject ALL Windows reparse points (junctions, mount points and links), not
/// just the subset represented by Rust's is_symlink. Never traverse one to write.
pub fn ordinary(metadata: &std::fs::Metadata) -> bool {
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        metadata.file_attributes() & 0x400 == 0
    }
    #[cfg(not(windows))]
    {
        !metadata.file_type().is_symlink()
    }
}

/// Existing owned paths only. Parent preflight belongs to the caller. On Windows
/// apply a protected DACL containing only the current token user. Fail closed on
/// unsupported ACL filesystems; do not silently inherit a public temporary ACL.
pub fn protect(path: &Path) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(windows)]
    {
        windows::protect(path)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn actual_space_and_missing_volume_error() {
        assert!(available_bytes(&std::env::temp_dir()).unwrap() > 0);
        assert!(available_bytes(Path::new("/regressee-nonexistent-volume-7e91")).is_err());
    }
}
