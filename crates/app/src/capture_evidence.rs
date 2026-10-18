// SPDX-License-Identifier: MPL-2.0
//! App-owned published channel evidence. Ingestion never recaptures a sibling.
use anyhow::{Result, ensure};
use regressee_core::{domain::*, evidence::*};
use regressee_storage::Store;
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};

/// Exclusive app-owned staging, private at directory creation rather than only
/// after files are published. Do not rely on the caller's process-wide umask.
/// The returned owner must live through detachment and checkpoint consumption.
pub(crate) fn private_staging(parent: &Path) -> Result<tempfile::TempDir> {
    regressee_runtime::distribution::safe_directory(parent)?;
    let metadata = std::fs::symlink_metadata(parent)?;
    ensure!(
        metadata.is_dir() && regressee_core::platform::ordinary(&metadata),
        "capture staging parent is not a physical directory"
    );
    let mut builder = tempfile::Builder::new();
    builder.prefix(".capture-");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        builder.permissions(std::fs::Permissions::from_mode(0o700));
    }
    let stage = builder.tempdir_in(parent)?;
    #[cfg(windows)]
    regressee_core::platform::protect(stage.path())?;
    Ok(stage)
}

pub(crate) enum PublishedChannel {
    Screenshot {
        metadata: CaptureMetadata,
        artifact: Artifact,
        path: PathBuf,
    },
    DomSnapshot {
        metadata: DomCaptureMetadata,
        artifact: Artifact,
        path: PathBuf,
    },
    ScreenshotFailed {
        code: CaptureError,
        retryable: bool,
    },
    DomSnapshotFailed {
        code: DomCaptureError,
        retryable: bool,
    },
}
impl PublishedChannel {
    pub(crate) fn kind(&self) -> CheckKind {
        match self {
            Self::Screenshot { .. } | Self::ScreenshotFailed { .. } => CheckKind::Screenshot,
            Self::DomSnapshot { .. } | Self::DomSnapshotFailed { .. } => CheckKind::DomSnapshot,
        }
    }
}

pub(crate) fn retain_screenshot(
    store: &mut Store,
    record: &mut RunRecord,
    index: usize,
    metadata: CaptureMetadata,
    artifact: &Artifact,
    path: &Path,
) -> Result<()> {
    let target = &record.run.targets[index];
    let address = regressee_core::environment::map_url(&target.requested, &record.run.environment)?;
    ensure!(
        matches!(target.outcome, TargetOutcome::Pending)
            && artifact.kind == ArtifactKind::Screenshot
            && metadata.version == 1
            && metadata.key == target.key
            && record.run.profiles.contains(&metadata.profile)
            && metadata.runtime == metadata.profile.runtime
            && metadata.requested == address.as_str()
            && metadata.final_url == address.as_str()
            && metadata.sha256 == artifact.id
            && metadata.bytes == artifact.bytes
            && metadata.duration_ms <= 600000
            && metadata.capture_mode == "chromium-new-headless",
        "published screenshot source binding mismatch"
    );
    crate::images::budget(metadata.width, metadata.height)?;
    // A channel already published successfully is retained even if a sibling
    // subsequently triggers the operation's cancellation/deadline token.
    let image = crate::images::ingest(
        store,
        path,
        ArtifactKind::Screenshot,
        &AtomicBool::new(false),
    )?;
    ensure!(
        image.original == artifact.id
            && image.width == metadata.width
            && image.height == metadata.height,
        "decoded screenshot differs from published evidence"
    );
    let target = &mut record.run.targets[index];
    target.final_url = Some(address);
    target
        .warnings
        .retain(|w| w.starts_with("ResumedAfterLogin"));
    target.warnings.extend(metadata.warnings.iter().cloned());
    target.outcome = TargetOutcome::Captured {
        artifact: image.original.clone(),
        width: image.width,
        height: image.height,
    };
    record.captures.push(CaptureEvidence { metadata, image });
    Ok(())
}

pub(crate) fn retain_dom(
    store: &mut Store,
    record: &mut RunRecord,
    index: usize,
    metadata: DomCaptureMetadata,
    artifact: &Artifact,
    path: &Path,
) -> Result<()> {
    let target = &record.run.targets[index];
    let address = regressee_core::environment::map_url(&target.requested, &record.run.environment)?;
    metadata.validate()?;
    ensure!(
        matches!(target.dom_outcome, DomTargetOutcome::Pending)
            && artifact.kind == ArtifactKind::DomSnapshot
            && record.run.capture_plan.mode.dom_snapshots()
            && metadata.run == record.run.id
            && metadata.key == target.key
            && record.run.profiles.contains(&metadata.profile)
            && Some(metadata.policy) == record.run.capture_plan.dom_policy
            && metadata.requested == address.as_str()
            && metadata.final_url == address.as_str()
            && metadata.sha256 == artifact.id
            && metadata.bytes == artifact.bytes,
        "published DOM source binding mismatch"
    );
    // Verification includes privacy grammar, byte/node/coverage diagnostics and
    // exact hash before even the private pending CAS file can be created.
    let file = std::fs::symlink_metadata(path)?;
    ensure!(
        file.is_file() && regressee_core::platform::ordinary(&file) && file.len() == artifact.bytes,
        "unsafe staged DOM output"
    );
    let capture = store.ingest_dom_snapshot(std::fs::File::open(path)?, &metadata)?;
    let target = &mut record.run.targets[index];
    target.final_url = Some(address);
    target.dom_outcome = DomTargetOutcome::Captured {
        artifact: artifact.id.clone(),
    };
    record.dom_captures.push(capture);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(unix)]
    #[test]
    fn capture_staging_is_private_at_creation_unique_and_owned_until_drop() -> Result<()> {
        use std::os::unix::fs::{MetadataExt, PermissionsExt};
        let root = tempfile::tempdir()?;
        // No umask mutation: the helper must request private permissions even
        // when its parent is deliberately an ordinary 0755 directory.
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o755))?;
        let first = private_staging(root.path())?;
        let second = private_staging(root.path())?;
        let metadata = std::fs::symlink_metadata(first.path())?;
        assert_eq!(metadata.mode() & 0o7777, 0o700);
        assert_eq!(metadata.uid(), std::fs::metadata(root.path())?.uid());
        assert_ne!(first.path(), second.path());
        let path = first.path().to_owned();
        std::fs::write(
            path.join("owned-publication"),
            b"inert staging lifetime fixture",
        )?;
        assert!(path.join("owned-publication").is_file());
        drop(first);
        assert!(!path.exists());
        assert!(second.path().is_dir());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn capture_staging_rejects_symlinked_or_missing_parents_without_writing() -> Result<()> {
        use std::os::unix::fs::symlink;
        let root = tempfile::tempdir()?;
        let real = root.path().join("real");
        std::fs::create_dir(&real)?;
        let linked = root.path().join("linked");
        symlink(&real, &linked)?;
        assert!(private_staging(&linked).is_err());
        assert_eq!(std::fs::read_dir(&real)?.count(), 0);
        let missing = root.path().join("missing");
        assert!(private_staging(&missing).is_err());
        assert!(!missing.exists());
        Ok(())
    }
}
