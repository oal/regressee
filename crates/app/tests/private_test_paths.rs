// SPDX-License-Identifier: MPL-2.0
#[path = "support/private_paths.rs"]
mod private_paths;

#[test]
fn explicit_private_test_directory_has_no_host_specific_location() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(root.path(), std::fs::Permissions::from_mode(0o700))?;
    }
    let physical = root.path().canonicalize()?;
    private_paths::private_directory(&physical)?;
    assert!(private_paths::private_directory(std::path::Path::new(".")).is_err());
    Ok(())
}

#[cfg(unix)]
#[test]
fn links_and_shared_test_directories_are_refused_without_mutation() -> anyhow::Result<()> {
    use std::os::unix::fs::{PermissionsExt, symlink};
    let root = tempfile::tempdir()?;
    let physical = root.path().canonicalize()?;
    let target = physical.join("target");
    std::fs::create_dir(&target)?;
    std::fs::set_permissions(&target, std::fs::Permissions::from_mode(0o700))?;
    let link = physical.join("link");
    symlink(&target, &link)?;
    assert!(private_paths::private_directory(&link).is_err());
    for mode in [0o755, 0o775, 0o777] {
        std::fs::set_permissions(&target, std::fs::Permissions::from_mode(mode))?;
        assert!(private_paths::private_directory(&target).is_err());
        assert_eq!(
            std::fs::metadata(&target)?.permissions().mode() & 0o7777,
            mode
        );
    }
    Ok(())
}
