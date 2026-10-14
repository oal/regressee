// SPDX-License-Identifier: MPL-2.0
//! Synthetic arithmetic/history fixtures; not browser or user-accuracy evidence.
use anyhow::{Result, ensure};
use regressee_app::{analysis_engine as e, review};
use regressee_core::{domain::*, evidence::*};
use regressee_storage::{AppPaths, Store};
use std::{
    path::{Path, PathBuf},
    sync::atomic::AtomicBool,
};
fn corpus() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/analysis-proposal")
}
fn image(store: &mut Store, name: &str) -> Result<ImageEvidence> {
    regressee_app::images::ingest(
        store,
        &corpus().join("images").join(name),
        ArtifactKind::Screenshot,
        &AtomicBool::new(false),
    )
}
fn capture(
    store: &mut Store,
    project: &Project,
    inventory: &InventoryRevision,
    intent: RunIntent,
    names: &[Option<&str>],
    mapped: bool,
) -> Result<RunRecord> {
    capture_with_renderer(
        store,
        project,
        inventory,
        intent,
        names,
        mapped,
        RenderingEnvironment {
            os: "fixture".into(),
            architecture: "fixture".into(),
            browser: "fixture".into(),
            worker_version: "fixture".into(),
            profile_version: 1,
            fonts_fingerprint: Some("fixture-fonts".into()),
        },
    )
}
fn capture_with_renderer(
    store: &mut Store,
    project: &Project,
    inventory: &InventoryRevision,
    intent: RunIntent,
    names: &[Option<&str>],
    mapped: bool,
    rendering: RenderingEnvironment,
) -> Result<RunRecord> {
    let mut environment = project.environments[0].clone();
    if mapped {
        environment.mappings[0].destination = "http://127.0.0.1:43211/".parse()?;
    }
    let mut r = RunRecord {
        run: CaptureRun {
            id: RunId::new(),
            project: project.id,
            intent,
            inventory: inventory.clone(),
            profiles: project.profiles.clone(),
            environment,
            rendering,
            state: JobState::Running,
            started_at: Some(100),
            finished_at: None,
            targets: inventory
                .pages
                .iter()
                .map(|p| CaptureTarget {
                    key: TargetKey {
                        page: p.id,
                        profile: project.profiles[0].id,
                    },
                    requested: p.canonical.clone(),
                    final_url: None,
                    attempts: 1,
                    outcome: TargetOutcome::Pending,
                    warnings: vec![],
                })
                .collect(),
        },
        capture: None,
        captures: vec![],
        queue: None,
    };
    store.begin_run(&r)?;
    for (i, name) in names.iter().enumerate() {
        if let Some(name) = name {
            let image = image(store, name)?;
            let t = &mut r.run.targets[i];
            let address = regressee_core::environment::map_url(&t.requested, &r.run.environment)?;
            t.final_url = Some(address.clone());
            t.outcome = TargetOutcome::Captured {
                artifact: image.original.clone(),
                width: image.width,
                height: image.height,
            };
            r.captures.push(CaptureEvidence {
                metadata: CaptureMetadata {
                    version: 1,
                    key: t.key.clone(),
                    requested: address.to_string(),
                    final_url: address.to_string(),
                    runtime: project.profiles[0].runtime.clone(),
                    profile: project.profiles[0].clone(),
                    width: image.width,
                    height: image.height,
                    sha256: image.original.clone(),
                    bytes: store.verify_artifact(&image.original)?.bytes,
                    warnings: vec![],
                    stable: true,
                    duration_ms: 1,
                    readiness: Readiness {
                        font_failures: 0,
                        image_failures: 0,
                        resource_failures: 0,
                    },
                    capture_mode: "chromium-new-headless".into(),
                    replacement: None,
                },
                image,
            });
        } else {
            r.run.targets[i].outcome = TargetOutcome::Failed {
                code: CaptureError::Timeout,
            };
        }
    }
    r.run.finished_at = Some(101);
    r.run.state = if names.iter().all(Option::is_some) {
        JobState::Completed
    } else {
        JobState::CompletedWithErrors
    };
    store.finish_run(&r)?;
    Ok(r)
}
fn setup(
    path: &Path,
    n: usize,
) -> Result<(
    Store,
    Project,
    InventoryRevision,
    RunRecord,
    BaselineVersion,
    u64,
)> {
    let mut s = Store::open(AppPaths::isolated(path))?;
    let mut p = Project::draft(
        "Synthetic Phase 5 history fixture".into(),
        "http://127.0.0.1:43210/page",
    )?;
    p.profiles[0].runtime = RuntimeIdentity::Pinned {
        node: "fixture".into(),
        worker: "fixture".into(),
        playwright: "fixture".into(),
        browser: "fixture".into(),
    };
    p.profiles[0].width = 96;
    p.profiles[0].height = 64;
    s.save_project(&p, None)?;
    let inventory = InventoryRevision {
        id: InventoryId::new(),
        project: p.id,
        identity_version: 2,
        pages: (0..n)
            .map(|i| InventoryPage {
                id: PageId::new(),
                canonical: format!("http://127.0.0.1:43210/page{i}").parse().unwrap(),
                included: true,
                reason: None,
            })
            .collect(),
        coverage: DiscoveryCoverage::Complete,
    };
    s.freeze_inventory(&inventory)?;
    let old = capture(
        &mut s,
        &p,
        &inventory,
        RunIntent::NewBaseline,
        &vec![Some("unchanged-before.png"); n],
        false,
    )?;
    // Public API activation validates source address for legacy single-target mode.
    let mut project = s.load_project(p.id)?;
    project.project.sources[0] = inventory.pages[0].canonical.clone();
    s.save_project(&project.project, Some(project.revision))?;
    let activated = s.activate_baseline(old.run.id, 2)?;
    let baseline = s.load_baseline(activated.project.active_baseline.unwrap())?;
    Ok((
        s,
        activated.project,
        inventory,
        old,
        baseline,
        activated.revision,
    ))
}
#[test]
fn approval_context_refuses_switch_reanalysis_selection_and_stale_baseline() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let (mut s, p, inventory, old, b, revision) = setup(dir.path(), 2)?;
    let check = capture(
        &mut s,
        &p,
        &inventory,
        RunIntent::Check { baseline: b.id },
        &[
            Some("ignored-rectangles-after.png"),
            Some("unchanged-after.png"),
        ],
        true,
    )?;
    let cancel = AtomicBool::new(false);
    let a = review::analyze_run(
        &mut s,
        check.run.id,
        review::preset(e::Preset::Balanced),
        &cancel,
    )?;
    let selected = vec![check.run.targets[0].key.clone()];
    let preview = s.approval_preview(a.revision.id, &selected)?;
    let original = std::fs::read(s.artifact_path(&old.captures[0].image.original)?)?;
    let count = s.baseline_history(p.id)?.len();
    let different = review::analyze_run(
        &mut s,
        check.run.id,
        review::preset(e::Preset::Relaxed),
        &cancel,
    )?;
    assert!(s.accept_selected(&preview, revision).is_err());
    s.display_analysis(a.revision.id)?;
    // A -> B -> A cannot revive an old confirmation.
    assert!(s.accept_selected(&preview, revision).is_err());
    let preview = s.approval_preview(a.revision.id, &selected)?;
    let cached = review::analyze_run(
        &mut s,
        check.run.id,
        review::preset(e::Preset::Balanced),
        &cancel,
    )?;
    assert_eq!(cached.revision.id, a.revision.id);
    assert!(s.accept_selected(&preview, revision).is_err());
    let preview = s.approval_preview(a.revision.id, &selected)?;
    let mut expected = preview.context.clone();
    expected.selected = vec![check.run.targets[1].key.clone()];
    assert!(s.accept_selected_in_context(&preview, &expected).is_err());
    expected = preview.context.clone();
    expected.run = old.run.id;
    assert!(s.accept_selected_in_context(&preview, &expected).is_err());
    expected = preview.context.clone();
    expected.project = ProjectId::new();
    assert!(s.accept_selected_in_context(&preview, &expected).is_err());
    s.invalidate_approval(check.run.id)?; // Analysis start, even before a result exists.
    assert!(s.accept_selected(&preview, revision).is_err());
    assert_eq!(s.baseline_history(p.id)?.len(), count);
    s.display_analysis(different.revision.id)?;
    let fresh = s.approval_preview(different.revision.id, &selected)?;
    let next = s.accept_selected_in_context(&fresh, &fresh.context)?;
    let accepted = s.load_baseline(next.project.active_baseline.unwrap())?;
    assert_eq!(accepted.references[0].run, check.run.id);
    assert_eq!(accepted.references[1], b.references[1]);
    assert!(s.accept_selected(&fresh, next.revision).is_err());
    s.restore_baseline(b.id, next.revision)?;
    assert!(s.accept_selected(&fresh, revision).is_err());
    assert_eq!(s.baseline_history(p.id)?.len(), count + 1);
    assert_eq!(
        std::fs::read(s.artifact_path(&old.captures[0].image.original)?)?,
        original
    );
    Ok(())
}

#[test]
fn phase6_real_composite_roundtrip_historical_settings_privacy_and_shared_deletion() -> Result<()> {
    use regressee_storage::DeleteItem;
    let root = tempfile::tempdir()?;
    let (mut owner, p, inventory, old, b, revision) = setup(&root.path().join("owner"), 3)?;
    let check = capture(
        &mut owner,
        &p,
        &inventory,
        RunIntent::Check { baseline: b.id },
        &[
            Some("ignored-rectangles-after.png"),
            Some("unchanged-after.png"),
            None,
        ],
        true,
    )?;
    let cancel = AtomicBool::new(false);
    let a = review::analyze_run(
        &mut owner,
        check.run.id,
        review::preset(e::Preset::Balanced),
        &cancel,
    )?;
    let mut mask = a.perceptual.as_ref().unwrap().reports[0].1.request.clone();
    mask.mask = Some(e::MaskBinding {
        revision: MaskId::new().to_string(),
        target: mask.before.target.clone(),
        before_capture_id: mask.before.capture_id.clone(),
        after_capture_id: mask.after.capture_id.clone(),
        before_dimensions: mask.before.dimensions,
        after_dimensions: mask.after.dimensions,
        rectangles: vec![e::Rectangle {
            x: 0,
            y: 0,
            width: 2,
            height: 3,
        }],
    });
    owner.save_mask(p.id, &mask)?;
    let approval = owner.approval_preview(a.revision.id, &[check.run.targets[0].key.clone()])?;
    let mut saved = owner.accept_selected(&approval, revision)?;
    let composite = owner.load_baseline(saved.project.active_baseline.unwrap())?;
    let immutable = serde_json::to_vec(&owner.load_run(old.run.id)?)?;
    let session = SessionReference {
        id: SessionId::new(),
        project: p.id,
        environment: p.environments[0].id,
    };
    owner.save_session_reference(&session, false)?;
    owner.select_environment(
        p.id,
        p.environments[0].id,
        &regressee_storage::LocalLoginSettings {
            login_address: Some("http://127.0.0.1:43210/private-login?owner=SECRET-LOGIN".parse()?),
            representative: None,
            identity_origins: vec![],
            ephemeral: false,
        },
    )?;
    std::fs::write(
        owner.paths().cache.join("private-log"),
        "PRIVATE-LOG-MUST-NOT-SHARE",
    )?;
    saved.project.profiles[0].width = 180;
    saved.project.sources[0] =
        "http://127.0.0.1:43210/future?private-query=visible-warning".parse()?;
    saved = owner.save_replacement_project(&saved.project, saved.revision)?;
    let preview = owner.export_preview(p.id, composite.id)?;
    assert_eq!(preview.sources, 2);
    assert_eq!(preview.targets, 3);
    assert_eq!(preview.masks, 1);
    assert!(
        preview
            .privacy
            .contains("private information, including ignored areas")
    );
    let zip = root.path().join("selected.zip");
    owner.export_archive(&preview, &zip, &cancel)?;
    let destination = AppPaths::isolated(&root.path().join("recipient"));
    let mut recipient = Store::open(destination.clone())?;
    let validated = recipient.inspect_archive(&zip, &cancel)?;
    let manifest = &validated.manifest;
    assert_eq!(manifest.project.profiles[0].width, 180);
    assert_eq!(manifest.provenance[0].profiles[0].width, 96);
    assert_eq!(
        manifest
            .provenance
            .iter()
            .map(|p| p.captures.len())
            .sum::<usize>(),
        3
    );
    assert!(manifest.baseline.parent.is_none());
    assert_eq!(manifest.artifacts.len(), 2);
    let text = serde_json::to_string(manifest)?;
    for sentinel in [
        session.id.to_string(),
        "SECRET-LOGIN".into(),
        "PRIVATE-LOG-MUST-NOT-SHARE".into(),
        "session".into(),
        owner.paths().data.display().to_string(),
    ] {
        assert!(!text.contains(&sentinel), "exported {sentinel}");
    }
    assert!(text.contains("private-query=visible-warning"));
    let expected = validated.preview.clone();
    drop(validated);
    let imported = recipient.import_archive(&zip, &expected, &cancel)?;
    assert_ne!(imported.project.id, p.id);
    assert_eq!(imported.project.profiles[0].width, 180);
    let active = recipient.load_baseline(imported.project.active_baseline.unwrap())?;
    assert_ne!(active.id, composite.id);
    assert_ne!(active.inventory, inventory.id);
    assert!(active.parent.is_none());
    let imported_inventory = recipient.load_inventory(active.inventory)?;
    for (i, r) in active.references.iter().enumerate() {
        let origin = recipient.load_run(r.run)?;
        let c = origin.evidence(&r.key).unwrap();
        let old_ref = &composite.references[i];
        let source = owner.load_run(old_ref.run)?;
        let before = source.evidence(&old_ref.key).unwrap();
        assert!(recipient.is_imported_run(r.run)?);
        assert_ne!(r.run, old_ref.run);
        assert_ne!(r.key.page, old_ref.key.page);
        assert_ne!(r.key.profile, old_ref.key.profile);
        assert_eq!(r.artifact, old_ref.artifact);
        assert_eq!(r.captured_at, old_ref.captured_at);
        assert_eq!(c.metadata.requested, before.metadata.requested);
        assert_eq!(c.metadata.final_url, before.metadata.final_url);
        let mut profile = c.metadata.profile.clone();
        profile.id = before.metadata.profile.id;
        assert_eq!(profile, before.metadata.profile);
        assert_eq!(origin.run.rendering, source.run.rendering);
        assert!(
            imported
                .project
                .environments
                .iter()
                .any(|e| e.id == origin.run.environment.id)
        );
        assert_eq!(
            std::fs::read(recipient.artifact_path(&r.artifact)?)?,
            std::fs::read(owner.artifact_path(&r.artifact)?)?
        );
        assert!(
            recipient
                .activate_baseline(r.run, imported.revision)
                .is_err()
        );
    }
    for e in &imported.project.environments {
        assert!(
            recipient
                .session_reference(imported.project.id, e.id)?
                .is_none()
        );
    }
    assert_eq!(
        recipient
            .local_login_settings(imported.project.id)?
            .login_address,
        None
    );
    assert!(
        imported_inventory
            .pages
            .iter()
            .zip(&inventory.pages)
            .all(|(a, b)| a.canonical == b.canonical)
    );
    let rebound_mask = recipient
        .latest_mask(imported.project.id, &active.references[0].key)?
        .unwrap();
    assert_eq!(rebound_mask.rectangles.len(), 1);
    assert!(
        !rebound_mask
            .before_capture_id
            .contains(&old.run.id.to_string())
    );
    let same_name = recipient.import_archive(&zip, &expected, &cancel)?;
    assert_ne!(same_name.project.id, imported.project.id);
    assert_eq!(
        same_name.project.name,
        format!("{} (imported 1)", saved.project.name)
    );
    assert_eq!(serde_json::to_vec(&owner.load_run(old.run.id)?)?, immutable);
    // Protect original/composite source history with an explicit reason.
    assert!(
        owner
            .deletion_preview(p.id, DeleteItem::Run(old.run.id))
            .unwrap_err()
            .to_string()
            .contains("retained baseline")
    );
    assert!(
        owner
            .deletion_preview(p.id, DeleteItem::Baseline(composite.id))
            .unwrap_err()
            .to_string()
            .contains("Active baseline")
    );
    let spare = capture(
        &mut owner,
        &p,
        &inventory,
        RunIntent::Check {
            baseline: composite.id,
        },
        &[
            Some("large-coherent-region-after.png"),
            Some("unchanged-after.png"),
            Some("unchanged-after.png"),
        ],
        false,
    )?;
    let impact = owner.deletion_preview(p.id, DeleteItem::Run(spare.run.id))?;
    assert!(impact.bytes > 0);
    assert!(impact.shared_bytes > 0);
    // A foreign/current owner added after preview must invalidate deletion authority.
    let shared = spare.captures[0].image.original.clone();
    let foreign = ArtifactOwner::Run(RunId::new());
    owner.add_reference(&foreign, &shared)?;
    assert!(owner.delete_confirmed(&impact).is_err());
    owner.load_run(spare.run.id)?;
    let stale = owner.deletion_preview(p.id, DeleteItem::Run(spare.run.id))?;
    owner.restore_baseline(b.id, saved.revision)?;
    assert!(owner.delete_confirmed(&stale).is_err());
    let fresh = owner.deletion_preview(p.id, DeleteItem::Run(spare.run.id))?;
    owner.delete_confirmed(&fresh)?;
    assert!(owner.load_run(spare.run.id).is_err());
    owner.verify_artifact(&shared)?;
    owner.load_baseline(composite.id)?;
    for r in &composite.references {
        owner.verify_artifact(&r.artifact)?;
    }
    let storage = recipient.project_storage(imported.project.id)?;
    assert!(storage.shared_bytes > 0);
    assert_eq!(storage.exclusive_bytes, 0);
    assert_eq!(
        storage.bytes,
        storage.original_bytes + storage.derived_bytes
    );
    Ok(())
}

type ZipEntries = Vec<(String, Vec<u8>)>;
type MaliciousCase = (String, ZipEntries, u32, bool);
fn zip_entries(path: &Path) -> Result<ZipEntries> {
    use std::io::Read;
    let mut z = zip::ZipArchive::new(std::fs::File::open(path)?)?;
    let mut entries = Vec::new();
    for i in 0..z.len() {
        let mut e = z.by_index(i)?;
        let name = e.name().to_owned();
        let mut bytes = Vec::new();
        e.read_to_end(&mut bytes)?;
        entries.push((name, bytes));
    }
    Ok(entries)
}
fn write_zip(path: &Path, entries: &[(String, Vec<u8>)], mode: u32, deflate: bool) -> Result<()> {
    use std::io::Write;
    let mut z = zip::ZipWriter::new(std::fs::File::create(path)?);
    let options = zip::write::SimpleFileOptions::default()
        .unix_permissions(mode)
        .compression_method(if deflate {
            zip::CompressionMethod::Deflated
        } else {
            zip::CompressionMethod::Stored
        });
    for (name, bytes) in entries {
        z.start_file(name, options)?;
        z.write_all(bytes)?;
    }
    z.finish()?;
    Ok(())
}
#[test]
fn phase6_malicious_zip_schema_images_duplicates_and_control_paths_are_atomic() -> Result<()> {
    use sha2::{Digest, Sha256};
    use std::io::Write;
    let root = tempfile::tempdir()?;
    let (owner, p, _, _, baseline, _) = setup(&root.path().join("owner"), 1)?;
    let cancel = AtomicBool::new(false);
    let zip = root.path().join("good.zip");
    let preview = owner.export_preview(p.id, baseline.id)?;
    owner.export_archive(&preview, &zip, &cancel)?;
    let base = zip_entries(&zip)?;
    let recipient_paths = AppPaths::isolated(&root.path().join("recipient"));
    let mut recipient = Store::open(recipient_paths.clone())?;
    let existing = Project::draft(
        "Unchanged existing project".into(),
        "http://127.0.0.1:43213/",
    )?;
    recipient.save_project(&existing, None)?;
    let before = serde_json::to_vec(&recipient.load_project(existing.id)?.project)?;
    let good = recipient.inspect_archive(&zip, &cancel)?.preview;
    let imported = recipient.import_archive(&zip, &good, &cancel)?;
    let kept = recipient.load_baseline(imported.project.active_baseline.unwrap())?;
    let projects = recipient.summary()?.projects;
    let artifacts = recipient.summary()?.artifacts;
    let mut cases: Vec<MaliciousCase> = Vec::new();
    for name in [
        "../outside",
        "/tmp/escape",
        "artifacts\\evil",
        "artifacts/../evil",
        "C:/evil.exe",
        "run.sh",
        "manifest.JSON",
    ] {
        let mut entries = base.clone();
        entries.push((name.into(), b"untrusted".to_vec()));
        cases.push((format!("path-{name}"), entries, 0o600, false));
    }
    for (name, edit) in [
        ("newer-version", 0),
        ("unexpected-script", 1),
        ("session-reference", 2),
        ("foreign-target", 3),
        ("included-parent", 4),
        ("wrong-count", 5),
        ("profile-mismatch", 6),
        ("unknown-image-kind", 7),
        ("no-capture-date", 8),
        ("duplicate-artifact", 9),
        ("duplicate-capture", 10),
        ("oversized-image-dimensions", 11),
        ("machine-rendering-path", 12),
        ("private-log-as-warning", 13),
    ] {
        let mut entries = base.clone();
        let mut v: serde_json::Value = serde_json::from_slice(&entries[0].1)?;
        match edit {
            0 => v["version"] = 99.into(),
            1 => v["script"] = "execute-me.js".into(),
            2 => v["project"]["environments"][0]["session"] = SessionId::new().to_string().into(),
            3 => v["baseline"]["references"][0]["key"]["page"] = PageId::new().to_string().into(),
            4 => v["baseline"]["parent"] = BaselineId::new().to_string().into(),
            5 => v["baseline"]["intended_count"] = 2.into(),
            6 => v["provenance"][0]["captures"][0]["profile"]["width"] = 101.into(),
            7 => v["artifacts"][0]["kind"] = "Difference".into(),
            8 => v["provenance"][0]["finished_at"] = 0.into(),
            9 => {
                let a = v["artifacts"][0].clone();
                v["artifacts"].as_array_mut().unwrap().push(a);
            }
            10 => {
                let c = v["provenance"][0]["captures"][0].clone();
                v["provenance"][0]["captures"]
                    .as_array_mut()
                    .unwrap()
                    .push(c);
            }
            11 => v["provenance"][0]["captures"][0]["height"] = u32::MAX.into(),
            12 => v["provenance"][0]["rendering"]["os"] = "/private/machine/path".into(),
            13 => {
                v["provenance"][0]["captures"][0]["warnings"] =
                    serde_json::json!(["PRIVATE LOG secret=bad"])
            }
            _ => unreachable!(),
        }
        entries[0].1 = serde_json::to_vec(&v)?;
        cases.push((name.into(), entries, 0o600, false));
    }
    cases.push(("executable-mode".into(), base.clone(), 0o700, false));
    let mut bomb = base.clone();
    bomb.push((
        format!("artifacts/{}", "f".repeat(64)),
        vec![0; 4 * 1024 * 1024],
    ));
    cases.push(("zip-bomb".into(), bomb, 0o600, true));
    // Malformed PNGs have CONSISTENT manifest hashes/sizes; failures are image validation, not only SHA checks.
    for (name, pixels) in [
        ("not-png", b"#!/bin/sh\necho no".to_vec()),
        ("truncated-png", base[1].1[..base[1].1.len() - 12].to_vec()),
        (
            "concatenated-png",
            [base[1].1.as_slice(), base[1].1.as_slice()].concat(),
        ),
    ] {
        let mut entries = base.clone();
        let hash = hex::encode(Sha256::digest(&pixels));
        let old_hash = entries[1].0.strip_prefix("artifacts/").unwrap();
        let text = String::from_utf8(entries[0].1.clone())?.replace(old_hash, &hash);
        let mut v: serde_json::Value = serde_json::from_str(&text)?;
        v["artifacts"][0]["bytes"] = (pixels.len() as u64).into();
        v["provenance"][0]["captures"][0]["bytes"] = (pixels.len() as u64).into();
        entries[0].1 = serde_json::to_vec(&v)?;
        entries[1] = (format!("artifacts/{hash}"), pixels);
        cases.push((name.into(), entries, 0o600, false));
    }
    for (i, (name, entries, mode, deflate)) in cases.iter().enumerate() {
        let path = root.path().join(format!("bad-{i}.zip"));
        write_zip(&path, entries, *mode, *deflate)?;
        let error = recipient.inspect_archive(&path, &cancel).err().expect(name);
        println!("{name}: {error:#}");
        assert_eq!(recipient.summary()?.projects, projects);
        assert_eq!(recipient.summary()?.artifacts, artifacts);
        assert_eq!(
            serde_json::to_vec(&recipient.load_project(existing.id)?.project)?,
            before
        );
        recipient.load_baseline(kept.id)?;
    }
    let duplicate = root.path().join("duplicate.zip");
    let mut entries = base.clone();
    let extra = format!("artifacts/{}", "f".repeat(64));
    entries.push((extra.clone(), base[1].1.clone()));
    write_zip(&duplicate, &entries, 0o600, false)?;
    let mut bytes = std::fs::read(&duplicate)?;
    let target = base[1].0.as_bytes();
    let from = extra.as_bytes();
    for i in 0..=bytes.len() - from.len() {
        if bytes[i..i + from.len()] == *from {
            bytes[i..i + from.len()].copy_from_slice(target);
        }
    }
    std::fs::write(&duplicate, &bytes)?;
    assert_eq!(
        zip::ZipArchive::new(std::fs::File::open(&duplicate)?)?.len(),
        2,
        "zip 2.4 hides the duplicate raw entry"
    );
    assert!(
        recipient
            .inspect_archive(&duplicate, &cancel)
            .err()
            .unwrap()
            .to_string()
            .contains("duplicate")
    );
    let case = root.path().join("case-collision.zip");
    let mut entries = base.clone();
    entries.push((base[1].0.to_uppercase(), base[1].1.clone()));
    write_zip(&case, &entries, 0o600, false)?;
    assert!(recipient.inspect_archive(&case, &cancel).is_err());
    let symlink = root.path().join("symlink.zip");
    let mut z = zip::ZipWriter::new(std::fs::File::create(&symlink)?);
    z.start_file("manifest.json", zip::write::SimpleFileOptions::default())?;
    z.write_all(&base[0].1)?;
    z.add_symlink(
        &base[1].0,
        "../../outside",
        zip::write::SimpleFileOptions::default(),
    )?;
    z.finish()?;
    assert!(recipient.inspect_archive(&symlink, &cancel).is_err());
    let changed = root.path().join("changed-after-preview.zip");
    write_zip(&changed, &base, 0o600, false)?;
    let expected = recipient.inspect_archive(&changed, &cancel)?.preview;
    let mut reordered = base.clone();
    reordered.reverse();
    write_zip(&changed, &reordered, 0o600, false)?;
    assert!(
        recipient
            .import_archive(&changed, &expected, &cancel)
            .unwrap_err()
            .to_string()
            .contains("changed")
    );
    assert!(owner.export_archive(&preview, &zip, &cancel).is_err());
    let stopped = AtomicBool::new(true);
    assert!(recipient.import_archive(&zip, &good, &stopped).is_err());
    assert!(!root.path().join("outside").exists());
    assert_eq!(recipient.summary()?.projects, projects);
    assert!(std::fs::read_dir(&recipient_paths.data)?.all(|e| {
        !e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(".import-staging-")
    }));
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        let outside = root.path().join("outside-target");
        symlink(&outside, root.path().join("unsafe.zip"))?;
        assert!(
            owner
                .export_archive(&preview, &root.path().join("unsafe.zip"), &cancel)
                .is_err()
        );
        assert!(!outside.exists());
        let missing = root.path().join("outside-dir");
        symlink(&missing, root.path().join("unsafe-parent"))?;
        assert!(
            owner
                .export_archive(
                    &preview,
                    &root.path().join("unsafe-parent/archive.zip"),
                    &cancel
                )
                .is_err()
        );
        assert!(!missing.exists());
    }
    Ok(())
}
#[test]
fn immutable_composite_provenance_restore_failure_guards_and_recheck() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let (mut s, p, inventory, old, b, revision) = setup(dir.path(), 3)?;
    let old_bytes = serde_json::to_vec(&old)?;
    let baseline_bytes = serde_json::to_vec(&b)?;
    let check = capture(
        &mut s,
        &p,
        &inventory,
        RunIntent::Check { baseline: b.id },
        &[
            Some("ignored-rectangles-after.png"),
            Some("unchanged-after.png"),
            None,
        ],
        true,
    )?;
    let a = review::analyze_run(
        &mut s,
        check.run.id,
        review::preset(e::Preset::Balanced),
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        a.revision.results[2].classification,
        Classification::CouldNotCapture
    );
    assert!(
        s.approval_preview(a.revision.id, &[check.run.targets[2].key.clone()])
            .is_err()
    );
    let selected = check.run.targets[0].key.clone();
    let preview = s.approval_preview(a.revision.id, std::slice::from_ref(&selected))?;
    assert_eq!(preview.inherited, 2);
    assert!(preview.provenance[0].1.contains(":43211/"));
    let mut corrupt = preview.clone();
    corrupt.baseline.references[1].artifact = preview.baseline.references[0].artifact.clone();
    assert!(s.accept_selected(&corrupt, revision).is_err());
    let mut collision = preview.clone();
    collision.baseline.id = b.id;
    assert!(s.accept_selected(&collision, revision).is_err());
    let unrelated = Project::draft(
        "Unrelated synthetic approval owner".into(),
        "http://127.0.0.1:43212/page",
    )?;
    let unrelated = s.save_project(&unrelated, None)?;
    let mut foreign = preview.clone();
    foreign.baseline.project = unrelated.project.id;
    assert!(s.accept_selected(&foreign, unrelated.revision).is_err());
    assert_eq!(
        s.load_project(unrelated.project.id)?
            .project
            .active_baseline,
        None
    );
    assert_eq!(s.load_project(p.id)?.project.active_baseline, Some(b.id));
    assert!(s.accept_selected(&preview, revision + 1).is_err());
    assert_eq!(s.load_project(p.id)?.project.active_baseline, Some(b.id));
    let next = s.accept_selected(&preview, revision)?;
    let composite = s.load_baseline(next.project.active_baseline.unwrap())?;
    assert_eq!(composite.references[0].run, check.run.id);
    assert_eq!(composite.references[1], b.references[1]);
    assert_eq!(composite.references[2], b.references[2]);
    assert_eq!(composite.intended_count, 3);
    assert_eq!(serde_json::to_vec(&s.load_run(old.run.id)?)?, old_bytes);
    assert_eq!(serde_json::to_vec(&s.load_baseline(b.id)?)?, baseline_bytes);
    let recheck = capture(
        &mut s,
        &next.project,
        &inventory,
        RunIntent::Check {
            baseline: composite.id,
        },
        &[
            Some("ignored-rectangles-after.png"),
            Some("unchanged-after.png"),
            Some("unchanged-after.png"),
        ],
        false,
    )?;
    let again = review::analyze_run(
        &mut s,
        recheck.run.id,
        review::preset(e::Preset::Balanced),
        &AtomicBool::new(false),
    )?;
    assert!(
        again
            .revision
            .results
            .iter()
            .all(|r| r.classification == Classification::NoSignificantChange)
    );
    let reports = &again.perceptual.as_ref().unwrap().reports;
    assert!(
        reports[0]
            .1
            .request
            .before
            .capture_id
            .starts_with(&check.run.id.to_string())
    );
    assert!(
        reports[1]
            .1
            .request
            .before
            .capture_id
            .starts_with(&old.run.id.to_string())
    );
    let restored = s.restore_baseline(b.id, next.revision)?;
    assert_eq!(restored.project.active_baseline, Some(b.id));
    assert_eq!(
        s.load_analysis(again.revision.id)?.revision.baseline,
        composite.id
    );
    assert!(s.restore_baseline(composite.id, next.revision).is_err());
    Ok(())
}
#[test]
fn reanalysis_cache_masks_all_area_cancellation_and_review_history() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let (mut s, p, inventory, old, b, _) = setup(dir.path(), 1)?;
    let check = capture(
        &mut s,
        &p,
        &inventory,
        RunIntent::Check { baseline: b.id },
        &[Some("unchanged-after.png")],
        false,
    )?;
    let cancel = AtomicBool::new(false);
    let balanced = review::preset(e::Preset::Balanced);
    let first = review::analyze_run(&mut s, check.run.id, balanced.clone(), &cancel)?;
    let key = check.run.targets[0].key.clone();
    s.save_review(&ReviewDecision {
        analysis: first.revision.id,
        key: key.clone(),
        state: ReviewState::Intentional,
        note: "Synthetic note stays on this revision".into(),
        reviewed_at: 102,
    })?;
    let sensitive = review::analyze_run(
        &mut s,
        check.run.id,
        review::preset(e::Preset::Sensitive),
        &cancel,
    )?;
    assert_ne!(first.revision.id, sensitive.revision.id);
    assert_eq!(
        s.review_history(check.run.id)?[0].analysis,
        first.revision.id
    );
    let cached = review::analyze_run(&mut s, check.run.id, balanced.clone(), &cancel)?;
    assert_eq!(cached.revision.id, first.revision.id);
    assert_eq!(
        s.project_evidence(p.id)?.comparison.unwrap().revision.id,
        first.revision.id
    );
    let mut request = first.perceptual.as_ref().unwrap().reports[0]
        .1
        .request
        .clone();
    request.mask = Some(e::MaskBinding {
        revision: MaskId::new().to_string(),
        target: request.before.target.clone(),
        before_capture_id: request.before.capture_id.clone(),
        after_capture_id: request.after.capture_id.clone(),
        before_dimensions: request.before.dimensions,
        after_dimensions: request.after.dimensions,
        rectangles: vec![
            e::Rectangle {
                x: 0,
                y: 0,
                width: 96,
                height: 64,
            },
            e::Rectangle {
                x: 0,
                y: 0,
                width: 96,
                height: 64,
            },
        ],
    });
    s.save_mask(p.id, &request)?;
    let masked = review::analyze_run(&mut s, check.run.id, balanced.clone(), &cancel)?;
    let report = &masked.perceptual.as_ref().unwrap().reports[0].1;
    assert_eq!(report.ignored_pixels, 6144);
    assert_eq!(report.eligible_pixels, 0);
    assert_eq!(report.changed_fraction.value(), None);
    assert_eq!(
        masked.revision.results[0].classification,
        Classification::NotComparable
    );
    assert!(s.approval_preview(masked.revision.id, &[key]).is_err());
    let count = s.project_evidence(p.id)?.analyses.len();
    assert!(
        review::analyze_run(
            &mut s,
            check.run.id,
            balanced.clone(),
            &AtomicBool::new(true)
        )
        .is_err()
    );
    assert_eq!(s.project_evidence(p.id)?.analyses.len(), count);
    let mut invalid = request.clone();
    invalid.mask.as_mut().unwrap().revision = MaskId::new().to_string();
    invalid.mask.as_mut().unwrap().rectangles[0].width = u32::MAX;
    assert!(s.save_mask(p.id, &invalid).is_err());
    request.mask.as_mut().unwrap().revision = MaskId::new().to_string();
    request.mask.as_mut().unwrap().rectangles.clear();
    s.save_mask(p.id, &request)?;
    let cleared = review::analyze_run(&mut s, check.run.id, balanced.clone(), &cancel)?;
    assert_ne!(cleared.revision.id, first.revision.id);
    assert_eq!(
        cleared.revision.results[0].classification,
        Classification::NoSignificantChange
    );
    let taller = capture(
        &mut s,
        &p,
        &inventory,
        RunIntent::Check { baseline: b.id },
        &[Some("added-height-band-after.png")],
        false,
    )?;
    let stale = review::analyze_run(&mut s, taller.run.id, balanced, &cancel)?;
    assert_eq!(stale.perceptual.as_ref().unwrap().failures.len(), 1);
    assert_eq!(
        stale.revision.results[0].classification,
        Classification::NotComparable
    );
    assert_eq!(
        s.load_run(old.run.id)?.captures[0].image.original,
        old.captures[0].image.original
    );
    Ok(())
}
#[test]
fn product_presets_match_annotated_independent_corpus_without_claiming_accuracy() -> Result<()> {
    let manifest: serde_json::Value =
        serde_json::from_reader(std::fs::File::open(corpus().join("manifest.json"))?)?;
    for case in manifest["cases"].as_array().unwrap() {
        let desc = |side: &str| e::CaptureDescriptor {
            capture_id: side.into(),
            target: e::Target {
                page: "fixture-page".into(),
                profile: "fixture-profile".into(),
            },
            dimensions: e::Dimensions {
                width: case[side]["width"].as_u64().unwrap() as u32,
                height: case[side]["height"].as_u64().unwrap() as u32,
            },
            device_scale_factor: 1.,
            material_profile_id: "fixture-material".into(),
            rendering_environment_id: "fixture-renderer".into(),
            completed: true,
        };
        for preset in [
            e::Preset::Sensitive,
            e::Preset::Balanced,
            e::Preset::Relaxed,
        ] {
            let before = desc("before");
            let after = desc("after");
            let rects: Vec<e::Rectangle> = serde_json::from_value(case["masks"].clone())?;
            let mask = (!rects.is_empty()).then(|| e::MaskBinding {
                revision: "fixture-mask".into(),
                target: before.target.clone(),
                before_capture_id: before.capture_id.clone(),
                after_capture_id: after.capture_id.clone(),
                before_dimensions: before.dimensions,
                after_dimensions: after.dimensions,
                rectangles: rects,
            });
            let mut request = e::AnalysisRequest {
                before,
                after,
                policy: e::Policy::seed(preset),
                mask,
            };
            let analyze = |r: &e::AnalysisRequest| {
                e::analyze_pngs(
                    std::fs::File::open(corpus().join(case["before"]["path"].as_str().unwrap()))
                        .unwrap(),
                    std::fs::File::open(corpus().join(case["after"]["path"].as_str().unwrap()))
                        .unwrap(),
                    r,
                    &AtomicBool::new(false),
                )
            };
            let mut expected = analyze(&request)?;
            request.policy = review::preset(preset);
            let actual = analyze(&request)?;
            expected.request.policy = request.policy.clone();
            ensure!(
                actual == expected,
                "product policy diverged from corpus seed oracle"
            );
        }
    }
    Ok(())
}

#[test]
fn approval_refuses_material_rendering_missing_artifact_and_preview_mixing() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let (mut s, p, inventory, old, b, revision) = setup(dir.path(), 1)?;
    let mut replacement = p.clone();
    replacement.profiles[0].image_mode = ImageMode::ReplaceRaster {
        algorithm: "neutral-raster-v2".into(),
    };
    let changed = capture(
        &mut s,
        &replacement,
        &inventory,
        RunIntent::Check { baseline: b.id },
        &[Some("unchanged-after.png")],
        false,
    )?;
    let analysis = review::analyze_run(
        &mut s,
        changed.run.id,
        review::preset(e::Preset::Balanced),
        &AtomicBool::new(false),
    )?;
    let key = changed.run.targets[0].key.clone();
    assert!(
        analysis.perceptual.as_ref().unwrap().reports[0]
            .1
            .gates
            .contains(&e::GateReason::MaterialProfileMismatch)
    );
    assert!(
        s.approval_preview(analysis.revision.id, std::slice::from_ref(&key))
            .is_err()
    );
    let mut renderer = old.run.rendering.clone();
    renderer.fonts_fingerprint = Some("different-fonts".into());
    let changed = capture_with_renderer(
        &mut s,
        &p,
        &inventory,
        RunIntent::Check { baseline: b.id },
        &[Some("unchanged-after.png")],
        true,
        renderer,
    )?;
    let analysis = review::analyze_run(
        &mut s,
        changed.run.id,
        review::preset(e::Preset::Balanced),
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        analysis.revision.results[0].classification,
        Classification::NotComparable
    );
    assert!(
        s.approval_preview(analysis.revision.id, std::slice::from_ref(&key))
            .is_err()
    );
    let preview = capture(
        &mut s,
        &p,
        &inventory,
        RunIntent::RepresentativePreview,
        &[Some("unchanged-after.png")],
        false,
    )?;
    assert!(s.activate_baseline(preview.run.id, revision).is_err());
    assert!(
        review::analyze_run(
            &mut s,
            preview.run.id,
            review::preset(e::Preset::Balanced),
            &AtomicBool::new(false)
        )
        .is_err()
    );
    let changed = capture(
        &mut s,
        &p,
        &inventory,
        RunIntent::Check { baseline: b.id },
        &[Some("ignored-rectangles-after.png")],
        false,
    )?;
    let analysis = review::analyze_run(
        &mut s,
        changed.run.id,
        review::preset(e::Preset::Balanced),
        &AtomicBool::new(false),
    )?;
    let approval = s.approval_preview(analysis.revision.id, std::slice::from_ref(&key))?;
    // Fault injection is confined to this test's owned CAS, not the corpus.
    let path = s.artifact_path(&changed.captures[0].image.original)?;
    let bytes = std::fs::read(&path)?;
    std::fs::write(&path, b"owned corruption fixture")?;
    assert!(s.accept_selected(&approval, revision).is_err());
    assert_eq!(s.load_project(p.id)?.project.active_baseline, Some(b.id));
    std::fs::write(&path, bytes)?;
    assert_eq!(s.load_baseline(b.id)?.references, b.references);
    Ok(())
}
