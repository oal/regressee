// SPDX-License-Identifier: MPL-2.0
//! Headless developer verification, not a non-developer product workflow.
use anyhow::{Context, Result, ensure};
use regressee_core::domain::{ArtifactKind, ArtifactOwner, BaselineId, Project};
use regressee_storage::{AppPaths, Store};
use std::path::PathBuf;
fn main() -> Result<()> {
    let root = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .context("usage: regressee-foundation <new-isolated-evidence-directory>")?;
    ensure!(
        !root.exists(),
        "use a fresh directory; never overwrite earlier evidence"
    );
    std::fs::create_dir_all(&root)?;
    let paths = AppPaths::isolated(&root);
    let mut store = Store::open(paths.clone())?;
    let project = Project::draft(
        "Foundation verification website".into(),
        "https://example.test/docs",
    )?;
    store.save_project(&project, None)?;
    let artifact = store.put_artifact(
        &b"synthetic storage integrity fixture, not screenshot/run evidence"[..],
        ArtifactKind::Screenshot,
    )?;
    let owner = ArtifactOwner::Baseline(BaselineId::new());
    store.add_reference(&owner, &artifact.id)?;
    ensure!(
        !store.collect_unreferenced(&artifact.id)?,
        "referenced evidence was deleted"
    );
    drop(store);
    let store = Store::open(paths)?;
    ensure!(
        store.load_project(project.id)?.project == project,
        "reopen lost settings"
    );
    store.verify_artifact(&artifact.id)?;
    let report = serde_json::json!({ "project_save_reopen": "PASS", "artifact_integrity_and_reference_guard": "PASS", "db_schema": regressee_storage::DB_VERSION, "summary": store.summary()?, "native_rendering_input": "NOT RUN by headless verification", "real_capture": "NOT EXERCISED by this synthetic foundation storage fixture; use regressee-flow / product_flow" });
    std::fs::write(
        root.join("foundation-report.json"),
        serde_json::to_vec_pretty(&report)?,
    )?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}
