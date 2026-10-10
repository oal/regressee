// SPDX-License-Identifier: MPL-2.0
//! Explicitly requested synthetic fixture, never a browser/native test result.
#![cfg(all(feature = "ui-inspection", target_os = "linux"))]

#[path = "support/approval_fixture.rs"]
mod approval_fixture;

#[test]
#[ignore = "prepares only an explicitly authorized private native-inspection fixture"]
fn prepare_owned_native_inspection_fixture() {
    use std::{
        fs,
        os::unix::fs::{OpenOptionsExt, PermissionsExt},
        path::PathBuf,
    };
    let root = PathBuf::from(std::env::var_os("REGRESSEE_NATIVE_FIXTURE_ROOT").unwrap());
    assert!(root.starts_with("/path/to/private-tests/opencode"));
    assert_eq!(fs::canonicalize(&root).unwrap(), root);
    assert_eq!(
        fs::metadata(&root).unwrap().permissions().mode() & 0o777,
        0o700
    );
    let store_path = root.join("store");
    assert_eq!(fs::canonicalize(&store_path).unwrap(), store_path);
    assert!(fs::read_dir(&store_path).unwrap().next().is_none());
    let (store, project, run, analysis, masked) = approval_fixture::fixture(&store_path);
    drop(store);
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(root.join("fixture.json"))
        .unwrap();
    serde_json::to_writer_pretty(
        &mut file,
        &serde_json::json!({
            "project": project.to_string(), "run": run.to_string(),
            "analysis": analysis.to_string(), "masked_analysis": masked.to_string(),
            "synthetic_pixels": true, "browser_capture": false
        }),
    )
    .unwrap();
    file.sync_all().unwrap();
}
