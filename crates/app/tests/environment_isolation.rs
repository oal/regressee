// SPDX-License-Identifier: MPL-2.0
//! Exercise ambient developer variables in a child, never mutate the test runner.
use anyhow::Result;
use regressee_storage::AppPaths;

#[test]
fn developer_environment_is_ignored_unless_explicitly_compiled_in() -> Result<()> {
    let root = tempfile::tempdir()?;
    let status = std::process::Command::new(std::env::current_exe()?)
        .args(["--exact", "developer_environment_child", "--ignored"])
        .env("REGRESSEE_DEV_DATA", root.path())
        .env("REGRESSEE_CAPTURE_TRACE_EVIDENCE", root.path())
        .status()?;
    assert!(status.success());
    assert!(std::fs::read_dir(root.path())?.next().is_none());
    Ok(())
}

#[test]
#[ignore = "private child of developer_environment_is_ignored_unless_explicitly_compiled_in"]
fn developer_environment_child() -> Result<()> {
    let root = std::path::PathBuf::from(std::env::var_os("REGRESSEE_DEV_DATA").unwrap());
    let actual = regressee_app::launch::app_paths()?;
    let trace = regressee_runtime::capture_diagnostics::trace_directory();
    let info = regressee_app::launch::build_info();
    assert_eq!(
        info["features"]["developer-tools"],
        cfg!(feature = "developer-tools")
    );
    if cfg!(feature = "developer-tools") {
        let expected = AppPaths::isolated(&root);
        assert_eq!(actual.data, expected.data);
        assert_eq!(actual.cache, expected.cache);
        assert_eq!(trace, Some(root));
    } else {
        let expected = AppPaths::discover()?;
        assert_eq!(actual.data, expected.data);
        assert_eq!(actual.cache, expected.cache);
        assert!(!actual.data.starts_with(&root));
        assert!(!actual.cache.starts_with(&root));
        assert_eq!(trace, None);
    }
    Ok(())
}
