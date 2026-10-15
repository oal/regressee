// SPDX-License-Identifier: MPL-2.0
//! Explicitly requested synthetic fixture, never a browser/native test result.
#![cfg(all(feature = "ui-inspection", target_os = "linux"))]

#[path = "support/approval_fixture.rs"]
mod approval_fixture;
#[path = "support/private_paths.rs"]
mod private_paths;

#[test]
#[ignore = "prepares only an explicitly authorized private native-inspection fixture"]
fn prepare_owned_native_inspection_fixture() {
    use std::{fs, os::unix::fs::OpenOptionsExt, path::PathBuf};
    let root = PathBuf::from(std::env::var_os("REGRESSEE_NATIVE_FIXTURE_ROOT").unwrap());
    private_paths::private_directory(&root).unwrap();
    let store_path = root.join("store");
    private_paths::private_directory(&store_path).unwrap();
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
