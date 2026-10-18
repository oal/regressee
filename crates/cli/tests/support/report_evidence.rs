// SPDX-License-Identifier: MPL-2.0
//! Test-only retention of actual engine output. No shipping API or default writes.
use super::*;
use anyhow::ensure;
use sha2::{Digest, Sha256};
use std::{
    io::{Read, Write},
    path::{Component, Path},
};

fn hash_file(path: &Path) -> Result<(String, u64)> {
    let mut file = fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut length = 0;
    let mut buffer = [0u8; 65536];
    loop {
        let count = file.read(&mut buffer)?;
        if count == 0 {
            break;
        }
        hasher.update(&buffer[..count]);
        length += count as u64;
    }
    Ok((hex::encode(hasher.finalize()), length))
}

fn private_file(path: &Path, bytes: &[u8]) -> Result<()> {
    use std::os::unix::fs::OpenOptionsExt;
    let mut file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)?;
    file.write_all(bytes)?;
    file.sync_all()?;
    Ok(())
}

pub(super) fn retain(
    fixture: &Fixture,
    report: &Path,
    preview: &Value,
    exported: &Value,
) -> Result<()> {
    let Some(destination) = std::env::var_os("REGRESSEE_DOM_REPORT_EVIDENCE") else {
        return Ok(());
    };
    let approved_root = std::path::PathBuf::from(
        std::env::var_os("REGRESSEE_DOM_REPORT_APPROVED_ROOT")
            .context("Explicit approved test-evidence root is required when retaining a report")?,
    );
    ensure!(
        approved_root.is_absolute() && approved_root.canonicalize()? == approved_root,
        "Approved test-evidence root must exist without symlink aliases"
    );
    regressee_app::engine::ownership::private_directory(&approved_root)?;
    let destination = std::path::PathBuf::from(destination);
    ensure!(
        destination.is_absolute()
            && destination.starts_with(&approved_root)
            && destination != approved_root
            && destination
                .components()
                .all(|c| !matches!(c, Component::ParentDir | Component::CurDir)),
        "test evidence destination must be within the explicitly approved engine-report-evidence root"
    );
    let parent = destination
        .parent()
        .context("test evidence parent missing")?;
    ensure!(
        parent.canonicalize()? == parent,
        "test evidence parent must exist without symlink aliases"
    );
    ensure!(
        fs::symlink_metadata(&destination).is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound),
        "test evidence destination already exists; choose a NEW child path, never overwrite a retained bundle"
    );
    ensure!(
        exported["data"]["job"]["state"] == "Succeeded",
        "retain only an actual successful engine export"
    );
    ensure!(
        preview["content"]["artifacts"]
            .as_array()
            .is_some_and(Vec::is_empty)
            && preview["content"]["dom_artifacts"]
                .as_array()
                .is_some_and(|a| a.len() == 1),
        "this browser-security fixture intentionally selects one DOM-only target"
    );
    use std::os::unix::fs::DirBuilderExt;
    fs::DirBuilder::new().mode(0o700).create(&destination)?;

    let mut files = Vec::new();
    let mut entries = fs::read_dir(report)?.collect::<std::io::Result<Vec<_>>>()?;
    entries.sort_by_key(|entry| entry.file_name());
    for entry in entries {
        let metadata = fs::symlink_metadata(entry.path())?;
        ensure!(
            metadata.is_file() && regressee_core::platform::ordinary(&metadata),
            "unexpected non-ordinary engine report asset"
        );
        let name = entry.file_name();
        let bytes = fs::read(entry.path())?;
        for sentinel in EXCLUDED_SENTINELS {
            ensure!(
                !bytes
                    .windows(sentinel.len())
                    .any(|w| w == sentinel.as_bytes()),
                "excluded synthetic canary leaked into an engine-exported asset"
            );
        }
        private_file(&destination.join(&name), &bytes)?;
        files.push(json!({"file":name.to_str().context("report filename is not UTF-8")?,"bytes":bytes.len(),"sha256":hex::encode(Sha256::digest(&bytes))}));
    }
    let mut originals = Vec::new();
    for pane in ["before", "after"] {
        let original: regressee_app::engine::dom::Original =
            serde_json::from_value(preview["content"]["dom_artifacts"][0][pane].clone())?;
        let file = format!("0-dom-{pane}.canonical.txt");
        let bytes = fs::read(destination.join(&file))?;
        original.metadata.verify_snapshot(&bytes)?;
        ensure!(
            original.source.artifact == original.artifact.id
                && original.metadata.run == original.source.run
                && original.metadata.key == original.source.key
                && original.metadata.policy == original.source.policy,
            "retained original's artifact/provenance bindings changed"
        );
        originals.push(json!({"pane":pane,"file":file,"original":original}));
    }
    let original_literal = fs::read_to_string(destination.join("0-dom-after.canonical.txt"))?;
    ensure!(
        original_literal.contains("</pre></script>")
            && original_literal.contains("onerror=")
            && original_literal.contains("https://attacker.invalid/x")
            && original_literal.contains("javascript:"),
        "retain the actual adversarial literal source, not a hand-constructed safe substitute"
    );

    private_file(
        &destination.join("report-preview.json"),
        &serde_json::to_vec_pretty(preview)?,
    )?;
    private_file(
        &destination.join("report-export-receipt.json"),
        &serde_json::to_vec_pretty(exported)?,
    )?;
    private_file(
        &destination.join("exclusion-expectations.json"),
        &serde_json::to_vec_pretty(&json!({
            "scope":"Synthetic canary names for audit only. They must be absent from report_files and captured originals. The fixture runs in-memory canonical rejection probes; it does not qualify browser collection/privacy.",
            "expected_absent_from_engine_exports":EXCLUDED_SENTINELS
        }))?,
    )?;
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()?;
    let mut source_hashes = Vec::new();
    for relative in [
        "crates/core/src/dom_snapshot.rs",
        "crates/app/src/dom_snapshot.rs",
        "crates/app/src/engine/dom.rs",
        "crates/app/src/engine/text.rs",
        "crates/app/src/engine/mod.rs",
        "crates/app/src/terminal.rs",
        "crates/app/src/protocol.rs",
        "crates/storage/src/dom.rs",
        "crates/storage/src/capture.rs",
        "crates/cli/src/main.rs",
        "crates/cli/src/commands.rs",
        "crates/cli/tests/dom_workflows.rs",
    ] {
        let (sha256, bytes) = hash_file(&root.join(relative))?;
        source_hashes.push(json!({"file":relative,"sha256":sha256,"bytes":bytes}));
    }
    let binary = Path::new(env!("CARGO_BIN_EXE_regressee")).canonicalize()?;
    let (binary_sha256, binary_bytes) = hash_file(&binary)?;
    let build = fixture.call(&["--build-info"], 0)?;
    private_file(
        &destination.join("bindings.json"),
        &serde_json::to_vec_pretty(&json!({
            "schema_version":1,
            "evidence_kind":"Actual prepare/confirm engine-exported report via public CLI/private IPC, owned retained synthetic DOM fixture; not browser-capture or browser-execution qualification",
            "test":"retained_dom_only_public_reads_review_approval_and_inert_report",
            "project":fixture.project,"check_run":fixture.run,"dom_analysis":fixture.analysis,"key":fixture.key,
            "preview_fingerprint":preview["fingerprint"],"preview_revision":preview["revision"],"selection":preview["selection"],
            "original_bindings":originals,"report_files":files,
            "preview_file":"report-preview.json","export_receipt_file":"report-export-receipt.json","exclusion_expectations_file":"exclusion-expectations.json",
            "browser_assertions":{"execution_property":"globalThis.executed","expected_execution_property":"undefined","forbidden_resource_urls":["https://attacker.invalid/x"],"expected_no_active_captured_markup":true,"expected_no_captured_resource_attempts":true},
            "binary":{"path":binary,"sha256":binary_sha256,"bytes":binary_bytes,"build_info":build["data"]},"source_hashes":source_hashes,
            "scope_note":"Only report_files are actual engine exports; audit JSON records include synthetic expected canary names. No credentials, sessions, keys, real user data, external visits, or raw excluded-data backups are retained. Export success is not a whole-feature or opened-browser PASS."
        }))?,
    )?;
    fs::File::open(&destination)?.sync_all()?;
    eprintln!(
        "Actual engine DOM report evidence retained at {}",
        destination.display()
    );
    Ok(())
}
