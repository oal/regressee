// SPDX-License-Identifier: MPL-2.0
use super::*;
use regressee_core::dom_snapshot::*;
use regressee_core::evidence::*;
use std::sync::atomic::AtomicBool;

fn bytes(element: &str, attributes: &str, child: &str) -> Vec<u8> {
    format!("{DOM_CANONICAL_HEADER}[\"document\",0]\n[\"element\",1,\"{HTML_NAMESPACE}\",\"{element}\",{attributes}]\n{child}").into_bytes()
}

#[test]
fn dom_cas_semantically_validates_before_any_disk_publication() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut store = Store::open(AppPaths::isolated(root.path()))?;
    for payload in [
        bytes("script", "[]", "[\"text\",2,\"private-canary\"]\n"),
        bytes("input", "[[\"\",\"value\",\"private-canary\"]]", ""),
        bytes("textarea", "[]", "[\"text\",2,\"private-canary\"]\n"),
        bytes("iframe", "[[\"\",\"srcdoc\",\"private-canary\"]]", ""),
        bytes("button", "[[\"\",\"onclick\",\"private-canary\"]]", ""),
        bytes("a", "[[\"\",\"href\",\"javascript:private-canary\"]]", ""),
        b"\xffprivate-canary".to_vec(),
        vec![b'x'; MAX_DOM_BYTES + 1],
    ] {
        let error = store
            .put_artifact(payload.as_slice(), ArtifactKind::DomSnapshot)
            .unwrap_err();
        assert!(!format!("{error:#}").contains("private-canary"));
        assert_eq!(store.summary()?.artifacts, 0);
        assert_eq!(fs::read_dir(store.paths.artifacts())?.count(), 0);
    }
    let safe = bytes(
        "html",
        "[]",
        "[\"text\",2,\"<script>inert ordinary private text</script>\"]\n",
    );
    let artifact = store.put_artifact(safe.as_slice(), ArtifactKind::DomSnapshot)?;
    assert_eq!(
        store
            .read_verified_artifact(&artifact.id, MAX_DOM_BYTES as u64)?
            .1,
        safe
    );
    assert_eq!(
        store.verify_artifact(&artifact.id)?.kind,
        ArtifactKind::DomSnapshot
    );
    let golden = include_bytes!("../../core/tests/fixtures/dom-v1.ndjson");
    let artifact = store.put_artifact(golden.as_slice(), ArtifactKind::DomSnapshot)?;
    assert_eq!(
        store
            .read_verified_artifact(&artifact.id, MAX_DOM_BYTES as u64)?
            .1
            .as_slice(),
        golden.as_slice()
    );
    Ok(())
}

fn fixture(mode: CaptureMode) -> Result<(tempfile::TempDir, Store, RunRecord, ProjectRecord)> {
    fixture_count(mode, 1)
}
fn fixture_count(
    mode: CaptureMode,
    count: usize,
) -> Result<(tempfile::TempDir, Store, RunRecord, ProjectRecord)> {
    let root = tempfile::tempdir()?;
    let mut store = Store::open(AppPaths::isolated(root.path()))?;
    let mut project = Project::draft(
        "Independent DOM fixture".into(),
        "http://127.0.0.1:43210/fixture",
    )?;
    project.capture_plan = CapturePlan::for_mode(mode);
    if count > 1 {
        project.discovery.mode = DiscoveryMode::AddressesAndLinks;
    }
    project.profiles[0].width = 8;
    project.profiles[0].height = 8;
    project.profiles[0].extent = CaptureExtent::VisibleScreen;
    project.profiles[0].runtime = RuntimeIdentity::Pinned {
        node: "fixture".into(),
        worker: "fixture".into(),
        playwright: "fixture".into(),
        browser: "fixture".into(),
    };
    let saved = store.save_project(&project, None)?;
    let page = InventoryPage {
        id: PageId::new(),
        canonical: project.sources[0].clone(),
        included: true,
        reason: None,
    };
    let mut record = RunRecord {
        run: CaptureRun {
            id: RunId::new(),
            project: project.id,
            intent: RunIntent::NewBaseline,
            inventory: InventoryRevision {
                id: InventoryId::new(),
                project: project.id,
                identity_version: 2,
                pages: vec![page.clone()],
                coverage: DiscoveryCoverage::Complete,
            },
            profiles: project.profiles.clone(),
            capture_plan: project.capture_plan.clone(),
            environment: project.environments[0].clone(),
            rendering: RenderingEnvironment {
                os: "fixture".into(),
                architecture: "fixture".into(),
                browser: "fixture".into(),
                worker_version: "fixture".into(),
                profile_version: 1,
                fonts_fingerprint: None,
            },
            state: JobState::Running,
            started_at: Some(100),
            finished_at: None,
            targets: vec![CaptureTarget {
                key: TargetKey {
                    page: page.id,
                    profile: project.profiles[0].id,
                },
                requested: page.canonical,
                final_url: None,
                attempts: 0,
                outcome: if mode.screenshots() {
                    TargetOutcome::Pending
                } else {
                    TargetOutcome::NotRequested
                },
                dom_outcome: if mode.dom_snapshots() {
                    DomTargetOutcome::Pending
                } else {
                    DomTargetOutcome::NotRequested
                },
                warnings: vec![],
            }],
        },
        capture: None,
        captures: vec![],
        dom_captures: vec![],
        queue: Some(QueueSettings {
            discovery: project.discovery,
            project_revision: saved.revision,
            resumed_at: vec![],
        }),
    };
    for index in 1..count {
        let page = InventoryPage {
            id: PageId::new(),
            canonical: format!("http://127.0.0.1:43210/fixture/{index}").parse()?,
            included: true,
            reason: None,
        };
        let mut target = record.run.targets[0].clone();
        target.key.page = page.id;
        target.requested = page.canonical.clone();
        record.run.inventory.pages.push(page);
        record.run.targets.push(target);
    }
    store.freeze_inventory(&record.run.inventory)?;
    store.begin_run(&record)?;
    Ok((root, store, record, saved))
}
fn populate(store: &mut Store, record: &mut RunRecord, label: &str) -> Result<()> {
    record.run.targets[0].attempts = 1;
    let requested = regressee_core::environment::map_url(
        &record.run.targets[0].requested,
        &record.run.environment,
    )?;
    record.run.targets[0].final_url = Some(requested.clone());
    let key = record.run.targets[0].key.clone();
    let profile = record.run.profiles[0].clone();
    if !matches!(record.run.targets[0].outcome, TargetOutcome::NotRequested) {
        let path = store
            .paths
            .cache
            .join(format!("fixture-{}.png", record.run.id));
        let mut png = Vec::new();
        {
            let mut pixels = [0u8; 256];
            for pixel in pixels.as_chunks_mut::<4>().0 {
                *pixel = [
                    label.as_bytes()[0],
                    label.as_bytes()[0],
                    label.as_bytes()[0],
                    255,
                ];
            }
            let mut encoder = png::Encoder::new(&mut png, 8, 8);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.write_header()?.write_image_data(&pixels)?;
        }
        fs::write(&path, png)?;
        let image = images::ingest(
            store,
            &path,
            ArtifactKind::Screenshot,
            &AtomicBool::new(false),
        )?;
        let artifact = store.verify_artifact(&image.original)?;
        record.run.targets[0].outcome = TargetOutcome::Captured {
            artifact: image.original.clone(),
            width: 8,
            height: 8,
        };
        record.captures.push(CaptureEvidence {
            metadata: CaptureMetadata {
                version: 1,
                key: key.clone(),
                requested: requested.to_string(),
                final_url: requested.to_string(),
                runtime: profile.runtime.clone(),
                profile: profile.clone(),
                width: 8,
                height: 8,
                sha256: image.original.clone(),
                bytes: artifact.bytes,
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
    }
    if !record.run.targets[0].dom_outcome.is_not_requested() {
        let canonical = bytes(
            "html",
            "[]",
            &format!("[\"text\",2,{}]\n", serde_json::to_string(label)?),
        );
        let snapshot = DomSnapshot::from_canonical(&canonical)?;
        let stats = snapshot.validate()?;
        let metadata = DomCaptureMetadata {
            version: 1,
            run: record.run.id,
            key,
            requested: requested.to_string(),
            final_url: requested.to_string(),
            runtime: profile.runtime.clone(),
            profile,
            policy: DomSnapshotPolicy::default(),
            sha256: dom_artifact_id(&canonical),
            bytes: canonical.len() as u64,
            node_count: stats.node_count,
            visited_nodes: stats.node_count,
            max_depth: stats.max_depth,
            coverage: stats.coverage,
            warnings: vec![],
            stable: true,
            duration_ms: 1,
            readiness: Readiness {
                font_failures: 0,
                image_failures: 0,
                resource_failures: 0,
            },
        };
        let capture = store.ingest_dom_snapshot(canonical.as_slice(), &metadata)?;
        record.run.targets[0].dom_outcome = DomTargetOutcome::Captured {
            artifact: metadata.sha256,
        };
        record.dom_captures.push(capture);
    }
    Ok(())
}
fn complete(store: &mut Store, record: &mut RunRecord, label: &str) -> Result<()> {
    populate(store, record, label)?;
    record.run.state = JobState::Completed;
    record.run.finished_at = Some(record.run.started_at.unwrap() + 1);
    store.finish_run(record)
}
fn check(
    store: &mut Store,
    before: &RunRecord,
    project: &ProjectRecord,
    label: &str,
) -> Result<RunRecord> {
    let mut after = before.clone();
    after.run.id = RunId::new();
    after.run.intent = RunIntent::Check {
        baseline: project.project.active_baseline.unwrap(),
    };
    after.run.state = JobState::Running;
    after.run.started_at = Some(before.run.finished_at.unwrap() + 100);
    after.run.finished_at = None;
    after.capture = None;
    after.captures.clear();
    after.dom_captures.clear();
    after.queue.as_mut().unwrap().project_revision = project.revision;
    let baseline = store.load_baseline(project.project.active_baseline.unwrap())?;
    after
        .run
        .targets
        .retain(|target| baseline.target_keys().contains(&target.key));
    for target in &mut after.run.targets {
        target.attempts = 0;
        target.final_url = None;
        target.outcome = if baseline.references.iter().any(|r| r.key == target.key) {
            TargetOutcome::Pending
        } else {
            TargetOutcome::NotRequested
        };
        target.dom_outcome = if baseline.dom_references.iter().any(|r| r.key == target.key) {
            DomTargetOutcome::Pending
        } else {
            DomTargetOutcome::NotRequested
        };
    }
    store.begin_run(&after)?;
    complete(store, &mut after, label)?;
    Ok(after)
}
fn analyze_dom(
    store: &mut Store,
    before: &RunRecord,
    after: &RunRecord,
    baseline: BaselineId,
) -> Result<DomComparisonEvidence> {
    let key = &after.run.targets[0].key;
    let before_source = dom_source(&before.run, before.dom_evidence(key).unwrap())?;
    let after_source = dom_source(&after.run, after.dom_evidence(key).unwrap())?;
    let before_bytes = store
        .read_verified_artifact(&before_source.artifact, MAX_DOM_BYTES as u64)?
        .1;
    let after_bytes = store
        .read_verified_artifact(&after_source.artifact, MAX_DOM_BYTES as u64)?
        .1;
    let options = DomDiffOptions::default();
    let result = compare_dom_snapshots(
        before_source,
        &before_bytes,
        after_source,
        &after_bytes,
        &options,
        || false,
    )?;
    let evidence = DomComparisonEvidence {
        revision: DomAnalysisRevision {
            version: 1,
            id: DomAnalysisId::new(),
            baseline,
            run: after.run.id,
            algorithm: DOM_DIFF_ALGORITHM.into(),
            results: vec![result],
        },
        created_at: 202,
        options,
    };
    store.save_dom_analysis(&evidence)?;
    Ok(evidence)
}
fn analyze_image(
    store: &mut Store,
    before: &RunRecord,
    after: &RunRecord,
    baseline: BaselineId,
) -> Result<ComparisonEvidence> {
    use regressee_core::analysis_engine as engine;
    let key = &after.run.targets[0].key;
    let b = before.evidence(key).unwrap();
    let a = after.evidence(key).unwrap();
    let request = engine::AnalysisRequest {
        before: descriptor(before, b)?,
        after: descriptor(after, a)?,
        policy: engine::Policy::seed(engine::Preset::Balanced),
        mask: None,
    };
    let report = engine::analyze_pngs(
        fs::File::open(store.artifact_path(&b.image.original)?)?,
        fs::File::open(store.artifact_path(&a.image.original)?)?,
        &request,
        &AtomicBool::new(false),
    )?;
    let classification = match report.classification {
        engine::Classification::Changed => Classification::Changed,
        engine::Classification::NoSignificantChange => Classification::NoSignificantChange,
        engine::Classification::NotComparable => Classification::NotComparable,
    };
    let evidence = ComparisonEvidence {
        revision: AnalysisRevision {
            id: AnalysisId::new(),
            run: after.run.id,
            baseline,
            algorithm: engine::ALGORITHM_ID.into(),
            policy: AnalysisPolicy {
                color_tolerance: 0.,
                changed_percent: 0.,
                region_pixels: 0,
            },
            mask: None,
            results: vec![AnalysisResult {
                key: key.clone(),
                classification,
                eligible_pixels: report.eligible_pixels,
                changed_pixels: report.changed_pixels,
                dimensions_changed: false,
            }],
        },
        heatmap: None,
        heatmaps: vec![],
        perceptual: Some(PerceptualEvidence {
            policy: request.policy,
            cache_key: "a".repeat(64),
            created_at: 202,
            reports: vec![(key.clone(), report)],
            failures: vec![],
        }),
    };
    store.save_analysis(&evidence)?;
    Ok(evidence)
}

#[test]
fn dom_only_roundtrip_analysis_review_approval_and_retention_have_no_png_dependencies() -> Result<()>
{
    let (root, mut store, mut before, project) = fixture(CaptureMode::DomSnapshotsOnly)?;
    complete(&mut store, &mut before, "before")?;
    let project = store.activate_baseline(before.run.id, project.revision)?;
    let baseline = project.project.active_baseline.unwrap();
    let after = check(&mut store, &before, &project, "after")?;
    let analysis = analyze_dom(&mut store, &before, &after, baseline)?;
    let result = &analysis.revision.results[0];
    let key = result.key.clone();
    let mut forged = analysis.clone();
    forged.revision.id = DomAnalysisId::new();
    forged.revision.results[0].before.as_mut().unwrap().run = RunId::new();
    assert!(store.save_dom_analysis(&forged).is_err());
    let mut forged = analysis.clone();
    forged.revision.id = DomAnalysisId::new();
    forged.revision.results[0].diff.as_mut().unwrap().hunks[0].rows[0].text =
        "private-unsourced-diff-canary".into();
    let error = store.save_dom_analysis(&forged).unwrap_err();
    assert!(!format!("{error:#}").contains("private-unsourced-diff-canary"));
    assert_eq!(store.dom_analysis_history(after.run.id)?.len(), 1);
    let decision = DomReviewDecision {
        analysis: analysis.revision.id,
        key: key.clone(),
        before: result.before.clone().unwrap(),
        after: result.after.clone().unwrap(),
        state: ReviewState::Intentional,
        note: "DOM note only".into(),
        reviewed_at: 203,
    };
    store.save_dom_review(&decision)?;
    assert_eq!(
        store.current_dom_reviews(analysis.revision.id, std::slice::from_ref(&key))?,
        vec![decision]
    );
    assert!(store.review_history(after.run.id)?.is_empty());
    let preview = store.dom_approval_preview(analysis.revision.id, std::slice::from_ref(&key))?;
    let project = store.accept_dom_selected(&preview, project.revision)?;
    let updated = store.load_baseline(project.project.active_baseline.unwrap())?;
    assert!(updated.references.is_empty());
    assert_eq!(updated.dom_references[0].run, after.run.id);
    assert!(
        store
            .deletion_preview(project.project.id, DeleteItem::Run(before.run.id))
            .is_err()
    );
    let storage = store.project_storage(project.project.id)?;
    assert!(storage.original_bytes > 0);
    assert_eq!(storage.derived_bytes, 0);
    let export = store.export_preview(project.project.id, updated.id)?;
    let path = root.path().join("dom-only.zip");
    store.export_archive(&export, &path, &AtomicBool::new(false))?;
    let archive = store.inspect_archive(&path, &AtomicBool::new(false))?;
    assert_eq!(archive.manifest.version, 2);
    assert!(
        archive
            .manifest
            .artifacts
            .iter()
            .all(|a| a.kind == ArtifactKind::DomSnapshot)
    );
    assert!(
        archive
            .manifest
            .provenance
            .iter()
            .all(|p| p.captures.is_empty())
    );
    let imported = store.import_archive(&path, &archive.preview, &AtomicBool::new(false))?;
    assert_ne!(imported.project.id, project.project.id);
    let imported_baseline = store.load_baseline(imported.project.active_baseline.unwrap())?;
    assert!(imported_baseline.references.is_empty());
    assert_eq!(imported_baseline.dom_references.len(), 1);
    let paths = store.paths.clone();
    drop(store);
    let store = Store::open(paths)?;
    store.load_dom_analysis(analysis.revision.id)?;
    store.dom_evidence_for_run(project.project.id, after.run.id)?;
    Ok(())
}

#[test]
fn project_removal_cleans_both_kinds_and_keeps_shared_imported_originals_and_zip() -> Result<()> {
    let (root, mut store, mut before, saved) = fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
    complete(&mut store, &mut before, "before")?;
    let saved = store.activate_baseline(before.run.id, saved.revision)?;
    let baseline = saved.project.active_baseline.unwrap();
    let after = check(&mut store, &before, &saved, "after")?;
    let dom = analyze_dom(&mut store, &before, &after, baseline)?;
    let result = &dom.revision.results[0];
    store.save_dom_review(&DomReviewDecision {
        analysis: dom.revision.id,
        key: result.key.clone(),
        before: result.before.clone().unwrap(),
        after: result.after.clone().unwrap(),
        state: ReviewState::NeedsFix,
        note: "Retained only until deliberate project deletion".into(),
        reviewed_at: 203,
    })?;
    let path = root.path().join("exported-baseline.zip");
    let cancel = AtomicBool::new(false);
    let export = store.export_preview(saved.project.id, baseline)?;
    store.export_archive(&export, &path, &cancel)?;
    let import = store.inspect_archive(&path, &cancel)?;
    let imported = store.import_archive(&path, &import.preview, &cancel)?;
    let kept = serde_json::to_value(store.project_evidence(imported.project.id)?)?;
    let preview = store.deletion_preview(saved.project.id, DeleteItem::Project)?;
    assert!(preview.shared_bytes > 0);
    assert!(preview.bytes > 0);
    assert_eq!(preview.scope.as_ref().unwrap().runs, 2);
    assert_eq!(preview.scope.as_ref().unwrap().reviews, 1);
    assert_eq!(preview.analyses, 1);
    let original = before.dom_captures[0].metadata.sha256.clone();
    let removed = after.dom_captures[0].metadata.sha256.clone();
    store.delete_confirmed(&preview)?;
    assert!(store.load_project(saved.project.id).is_err());
    assert!(store.load_run(before.run.id).is_err());
    assert!(store.load_dom_analysis(dom.revision.id).is_err());
    assert!(store.verify_artifact(&removed).is_err());
    assert!(store.verify_artifact(&original).is_ok());
    assert_eq!(
        serde_json::to_value(store.project_evidence(imported.project.id)?)?,
        kept
    );
    assert!(
        path.is_file(),
        "exported ZIPs are outside the deletion scope"
    );
    let foreign_key_errors: u64 =
        store
            .connection
            .query_row("SELECT count(*) FROM pragma_foreign_key_check", [], |r| {
                r.get(0)
            })?;
    assert_eq!(foreign_key_errors, 0);
    Ok(())
}

#[test]
fn sibling_approvals_rebase_and_both_kinds_commit_atomically() -> Result<()> {
    for image_first in [true, false] {
        let (_root, mut store, mut before, project) =
            fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
        complete(&mut store, &mut before, "before")?;
        let project = store.activate_baseline(before.run.id, project.revision)?;
        let baseline = project.project.active_baseline.unwrap();
        let after = check(&mut store, &before, &project, "after")?;
        let image = analyze_image(&mut store, &before, &after, baseline)?;
        let dom = analyze_dom(&mut store, &before, &after, baseline)?;
        let keys = vec![after.run.targets[0].key.clone()];
        let stale_dom = store.dom_approval_preview(dom.revision.id, &keys)?;
        let first = if image_first {
            let p = store.approval_preview(image.revision.id, &keys)?;
            store.accept_selected(&p, project.revision)?
        } else {
            store.accept_dom_selected(&stale_dom, project.revision)?
        };
        let midway = store.load_baseline(first.project.active_baseline.unwrap())?;
        let portable = store.archive_manifest(project.project.id, midway.id)?;
        assert_eq!(
            portable
                .provenance
                .iter()
                .flat_map(|p| &p.dom_captures)
                .count(),
            1
        );
        assert_eq!(
            portable.provenance.iter().flat_map(|p| &p.captures).count(),
            1
        );
        if image_first {
            assert!(
                portable
                    .provenance
                    .iter()
                    .find(|p| p.run == after.run.id)
                    .unwrap()
                    .dom_captures
                    .is_empty()
            );
        }
        assert_eq!(
            midway.references[0].run,
            if image_first {
                after.run.id
            } else {
                before.run.id
            }
        );
        assert_eq!(
            midway.dom_references[0].run,
            if image_first {
                before.run.id
            } else {
                after.run.id
            }
        );
        if image_first {
            assert!(
                store
                    .accept_dom_selected(&stale_dom, first.revision)
                    .is_err()
            );
            let p = store.dom_approval_preview(dom.revision.id, &keys)?;
            store.accept_dom_selected(&p, first.revision)?;
        } else {
            let p = store.approval_preview(image.revision.id, &keys)?;
            store.accept_selected(&p, first.revision)?;
        }
        let current = store.load_project(project.project.id)?;
        let updated = store.load_baseline(current.project.active_baseline.unwrap())?;
        assert_eq!(updated.references[0].run, after.run.id);
        assert_eq!(updated.dom_references[0].run, after.run.id);
        assert!(store.approval_preview(image.revision.id, &keys).is_err());
        assert!(store.dom_approval_preview(dom.revision.id, &keys).is_err());
    }
    let (_root, mut store, mut before, project) = fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
    complete(&mut store, &mut before, "before")?;
    let project = store.activate_baseline(before.run.id, project.revision)?;
    let baseline = project.project.active_baseline.unwrap();
    let after = check(&mut store, &before, &project, "after")?;
    let image = analyze_image(&mut store, &before, &after, baseline)?;
    let dom = analyze_dom(&mut store, &before, &after, baseline)?;
    let keys = vec![after.run.targets[0].key.clone()];
    let preview = store.approval_preview_kinds(
        Some(image.revision.id),
        &keys,
        Some(dom.revision.id),
        &keys,
    )?;
    store.invalidate_dom_approval(after.run.id)?;
    assert!(store.accept_selected_kinds(&preview).is_err());
    assert_eq!(
        store
            .load_project(project.project.id)?
            .project
            .active_baseline,
        Some(baseline)
    );
    let preview = store.approval_preview_kinds(
        Some(image.revision.id),
        &keys,
        Some(dom.revision.id),
        &keys,
    )?;
    let count: u64 = store
        .connection
        .query_row("SELECT count(*) FROM baselines", [], |r| r.get(0))?;
    store.connection.execute_batch("CREATE TRIGGER fail_kind_approval BEFORE UPDATE ON projects BEGIN SELECT RAISE(ABORT,'owned atomic approval failure'); END;")?;
    assert!(store.accept_selected_kinds(&preview).is_err());
    assert_eq!(
        store
            .connection
            .query_row("SELECT count(*) FROM baselines", [], |r| r.get::<_, u64>(0))?,
        count
    );
    assert_eq!(
        store
            .load_project(project.project.id)?
            .project
            .active_baseline,
        Some(baseline)
    );
    store
        .connection
        .execute_batch("DROP TRIGGER fail_kind_approval;")?;
    let project = store.accept_selected_kinds(&preview)?;
    let baseline = store.load_baseline(project.project.active_baseline.unwrap())?;
    assert_eq!(baseline.references[0].run, after.run.id);
    assert_eq!(baseline.dom_references[0].run, after.run.id);
    Ok(())
}

#[test]
fn kind_scoped_readers_and_approvals_ignore_corrupt_unselected_sibling_files() -> Result<()> {
    for corrupt_image in [true, false] {
        let (_root, mut store, mut before, project) =
            fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
        complete(&mut store, &mut before, "before")?;
        let project = store.activate_baseline(before.run.id, project.revision)?;
        let baseline = project.project.active_baseline.unwrap();
        let after = check(&mut store, &before, &project, "after")?;
        let image = analyze_image(&mut store, &before, &after, baseline)?;
        let dom = analyze_dom(&mut store, &before, &after, baseline)?;
        let key = after.run.targets[0].key.clone();
        let corrupt = if corrupt_image {
            after.captures[0].image.tiles[0].clone()
        } else {
            after.dom_captures[0].metadata.sha256.clone()
        };
        fs::write(store.paths.artifacts().join(corrupt.as_str()), b"corrupted")?;
        if corrupt_image {
            store.read_dom_snapshot(after.run.id, &key)?;
            store.load_dom_analysis(dom.revision.id)?;
            let preview = store.dom_approval_preview(dom.revision.id, &[key])?;
            store.accept_dom_selected(&preview, project.revision)?;
        } else {
            store.load_analysis(image.revision.id)?;
            let preview = store.approval_preview(image.revision.id, &[key])?;
            store.accept_selected(&preview, project.revision)?;
        }
    }
    Ok(())
}

#[test]
fn successful_slots_checkpoint_in_either_order_survive_fresh_process_resume() -> Result<()> {
    for dom_first in [true, false] {
        let (_root, mut store, mut record, _project) =
            fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
        let mut claim = store.begin_run_append(record.run.id)?;
        populate(&mut store, &mut record, "both")?;
        let mut first = record.run.targets[0].clone();
        if dom_first {
            first.outcome = TargetOutcome::Pending;
        } else {
            first.dom_outcome = DomTargetOutcome::Pending;
        }
        store.append_run_kind_targets(
            &mut claim,
            &[KindTargetCheckpoint {
                index: 0,
                target: &first,
                capture: (!dom_first).then_some(&record.captures[0]),
                dom_capture: dom_first.then_some(&record.dom_captures[0]),
            }],
        )?;
        let partial = store.load_run(record.run.id)?;
        assert_eq!(partial.all_captures().count(), usize::from(!dom_first));
        assert_eq!(partial.dom_captures.len(), usize::from(dom_first));
        store.append_run_kind_targets(
            &mut claim,
            &[KindTargetCheckpoint {
                index: 0,
                target: &record.run.targets[0],
                capture: dom_first.then_some(&record.captures[0]),
                dom_capture: (!dom_first).then_some(&record.dom_captures[0]),
            }],
        )?;
        let saved = store.load_run(record.run.id)?;
        assert_eq!(saved.all_captures().count(), 1);
        assert_eq!(saved.dom_captures.len(), 1);
        let mut invalid = record.run.targets[0].clone();
        invalid.dom_outcome = DomTargetOutcome::Failed {
            code: DomCaptureError::SerializationFailed,
        };
        assert!(
            store
                .append_run_kind_targets(
                    &mut claim,
                    &[KindTargetCheckpoint {
                        index: 0,
                        target: &invalid,
                        capture: None,
                        dom_capture: None
                    }]
                )
                .is_err()
        );
        record.run.state = JobState::Paused;
        store.checkpoint_run(&record)?;
        let paths = store.paths.clone();
        drop(store);
        let mut store = Store::open(paths)?;
        record.run.state = JobState::Running;
        record.queue.as_mut().unwrap().resumed_at.push(300);
        store.resume_run(&record)?;
        let mut forged = record.clone();
        forged.dom_captures[0].metadata.stable = false;
        assert!(store.checkpoint_run(&forged).is_err());
        record.run.state = JobState::Completed;
        record.run.finished_at = Some(301);
        store.finish_run(&record)?;
        let saved = store.load_run(record.run.id)?;
        assert_eq!(saved.dom_captures.len(), 1);
        assert_eq!(saved.all_captures().count(), 1);
    }
    Ok(())
}

#[test]
fn explicit_omissions_stay_missing_and_cannot_be_filled_from_after_only_approval() -> Result<()> {
    let (_root, mut store, mut before, project) = fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
    populate(&mut store, &mut before, "before")?;
    before.dom_captures.clear();
    before.run.targets[0].dom_outcome = DomTargetOutcome::Failed {
        code: DomCaptureError::SerializationFailed,
    };
    before.run.state = JobState::CompletedWithErrors;
    before.run.finished_at = Some(101);
    store.finish_run(&before)?;
    assert!(
        store
            .activate_baseline(before.run.id, project.revision)
            .is_err()
    );
    let project =
        store.activate_baseline_with_partial_confirmation(before.run.id, project.revision, true)?;
    let baseline = store.load_baseline(project.project.active_baseline.unwrap())?;
    assert!(baseline.explicitly_partial);
    assert_eq!(baseline.intended_count, 1);
    assert_eq!(baseline.saved_units().len(), 1);
    assert_eq!(baseline.omitted_units(&before.run)?.len(), 1);
    let after = check(&mut store, &before, &project, "after")?;
    assert!(after.dom_captures.is_empty());
    assert!(after.run.targets[0].dom_outcome.is_not_requested());
    let analysis = DomComparisonEvidence {
        revision: DomAnalysisRevision {
            version: 1,
            id: DomAnalysisId::new(),
            baseline: baseline.id,
            run: after.run.id,
            algorithm: DOM_DIFF_ALGORITHM.into(),
            results: vec![DomAnalysisResult {
                key: after.run.targets[0].key.clone(),
                before: None,
                after: None,
                outcome: DomComparisonOutcome::NotComparable {
                    reason: DomNotComparableReason::BaselineOmission,
                },
                diff: None,
            }],
        },
        created_at: 202,
        options: DomDiffOptions::default(),
    };
    store.save_dom_analysis(&analysis)?;
    assert!(
        store
            .dom_approval_preview(analysis.revision.id, &[after.run.targets[0].key.clone()])
            .is_err()
    );
    let image = analyze_image(&mut store, &before, &after, baseline.id)?;
    let preview = store.approval_preview(image.revision.id, &[after.run.targets[0].key.clone()])?;
    store.accept_selected(&preview, project.revision)?;
    let current = store.load_project(project.project.id)?;
    let updated = store.load_baseline(current.project.active_baseline.unwrap())?;
    assert!(updated.dom_references.is_empty());
    assert!(updated.explicitly_partial);
    Ok(())
}

#[test]
fn correctly_hashed_malicious_zip_snapshot_is_rejected_before_retained_ingestion() -> Result<()> {
    use regressee_core::archive::artifact_entry;
    let (root, mut store, mut before, project) = fixture(CaptureMode::DomSnapshotsOnly)?;
    complete(&mut store, &mut before, "before")?;
    let project = store.activate_baseline(before.run.id, project.revision)?;
    let mut manifest =
        store.archive_manifest(project.project.id, project.project.active_baseline.unwrap())?;
    let payload = bytes("input", "[[\"\",\"value\",\"private-zip-canary\"]]", "");
    let id = dom_artifact_id(&payload);
    manifest.artifacts[0].id = id.clone();
    manifest.artifacts[0].bytes = payload.len() as u64;
    manifest.baseline.dom_references[0].artifact = id.clone();
    let metadata = &mut manifest.provenance[0].dom_captures[0];
    metadata.sha256 = id.clone();
    metadata.bytes = payload.len() as u64;
    metadata.node_count = 2;
    metadata.visited_nodes = 2;
    metadata.max_depth = 1;
    let path = root.path().join("malicious-dom.zip");
    let mut zip = zip::ZipWriter::new(fs::File::create(&path)?);
    let options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o600);
    zip.start_file("manifest.json", options)?;
    zip.write_all(&serde_json::to_vec(&manifest)?)?;
    zip.start_file(artifact_entry(&id), options)?;
    zip.write_all(&payload)?;
    zip.finish()?;
    let old = store.summary()?.artifacts;
    let error = store
        .inspect_archive(&path, &AtomicBool::new(false))
        .err()
        .expect("malicious semantic payload accepted");
    assert!(!format!("{error:#}").contains("private-zip-canary"));
    assert_eq!(store.summary()?.artifacts, old);
    assert!(!store.paths.artifacts().join(id.as_str()).exists());
    assert!(
        store
            .inspect_archive(&path, &AtomicBool::new(true))
            .is_err()
    );
    assert_eq!(store.summary()?.projects, 1);
    Ok(())
}

#[test]
fn dom_import_cancellation_and_late_sql_failure_never_publish_partial_metadata() -> Result<()> {
    let (root, mut source, mut before, project) = fixture(CaptureMode::DomSnapshotsOnly)?;
    complete(&mut source, &mut before, "portable")?;
    let project = source.activate_baseline(before.run.id, project.revision)?;
    let path = root.path().join("atomic-dom.zip");
    source.export_archive(
        &source.export_preview(project.project.id, project.project.active_baseline.unwrap())?,
        &path,
        &AtomicBool::new(false),
    )?;
    let mut recipient = Store::open(AppPaths::isolated(&root.path().join("recipient")))?;
    let preview = recipient
        .inspect_archive(&path, &AtomicBool::new(false))?
        .preview;
    assert!(
        recipient
            .import_archive(&path, &preview, &AtomicBool::new(true))
            .is_err()
    );
    assert_eq!(recipient.summary()?.projects, 0);
    assert_eq!(recipient.summary()?.artifacts, 0);
    recipient.connection.execute_batch("CREATE TRIGGER fail_dom_import BEFORE INSERT ON projects BEGIN SELECT RAISE(ABORT,'owned import failure'); END;")?;
    assert!(
        recipient
            .import_archive(&path, &preview, &AtomicBool::new(false))
            .is_err()
    );
    assert_eq!(recipient.summary()?.projects, 0);
    assert_eq!(recipient.summary()?.artifacts, 0);
    assert_eq!(
        recipient
            .connection
            .query_row("SELECT count(*) FROM runs", [], |r| r.get::<_, u64>(0))?,
        0
    );
    assert_eq!(
        recipient
            .connection
            .query_row("SELECT count(*) FROM baselines", [], |r| r.get::<_, u64>(0))?,
        0
    );
    assert_eq!(
        recipient.repair_preview()?.valid_unreferenced.len(),
        1,
        "complete orphan must remain for explicit repair, never automatic deletion"
    );
    recipient
        .connection
        .execute_batch("DROP TRIGGER fail_dom_import;")?;
    let imported = recipient.import_archive(&path, &preview, &AtomicBool::new(false))?;
    let baseline = recipient.load_dom_baseline(imported.project.active_baseline.unwrap())?;
    assert_eq!(baseline.dom_references.len(), 1);
    assert_eq!(recipient.summary()?.artifacts, 1);
    Ok(())
}

#[test]
fn explicit_run_deletion_counts_dom_analysis_and_preserves_shared_before_original() -> Result<()> {
    let (_root, mut store, mut before, project) = fixture(CaptureMode::DomSnapshotsOnly)?;
    complete(&mut store, &mut before, "before")?;
    let project = store.activate_baseline(before.run.id, project.revision)?;
    let after = check(&mut store, &before, &project, "after")?;
    let analysis = analyze_dom(
        &mut store,
        &before,
        &after,
        project.project.active_baseline.unwrap(),
    )?;
    let result = &analysis.revision.results[0];
    store.save_dom_review(&DomReviewDecision {
        analysis: analysis.revision.id,
        key: result.key.clone(),
        before: result.before.clone().unwrap(),
        after: result.after.clone().unwrap(),
        state: ReviewState::NeedsFix,
        note: "keep Before".into(),
        reviewed_at: 203,
    })?;
    let preview = store.deletion_preview(project.project.id, DeleteItem::Run(after.run.id))?;
    assert_eq!(preview.analyses, 1);
    assert!(preview.bytes > 0);
    assert!(preview.shared_bytes > 0);
    store.delete_confirmed(&preview)?;
    store.verify_artifact(&before.dom_captures[0].metadata.sha256)?;
    assert!(
        store
            .verify_artifact(&after.dom_captures[0].metadata.sha256)
            .is_err()
    );
    assert!(store.load_dom_analysis(analysis.revision.id).is_err());
    assert_eq!(store.summary()?.artifacts, 1);
    Ok(())
}

fn shipped_legacy_baseline(b: &BaselineVersion) -> Result<String> {
    #[derive(Serialize)]
    struct Legacy<'a> {
        id: BaselineId,
        project: ProjectId,
        parent: Option<BaselineId>,
        inventory: InventoryId,
        created_at: i64,
        references: &'a [BaselineReference],
        intended_count: u32,
        explicitly_partial: bool,
    }
    let json = serde_json::to_string(&Legacy {
        id: b.id,
        project: b.project,
        parent: b.parent,
        inventory: b.inventory,
        created_at: b.created_at,
        references: &b.references,
        intended_count: b.intended_count,
        explicitly_partial: b.explicitly_partial,
    })?;
    assert!(
        !json.contains("source_run")
            && !json.contains("intended_targets")
            && !json.contains("capture_plan")
            && !json.contains("dom_references")
    );
    Ok(json)
}

#[test]
fn shipped_legacy_fully_replaced_partial_baseline_recovers_only_verified_ancestor_source()
-> Result<()> {
    let (root, mut store, mut original, project) = fixture_count(CaptureMode::ScreenshotsOnly, 2)?;
    populate(&mut store, &mut original, "original")?;
    original.run.targets[1].outcome = TargetOutcome::Failed {
        code: CaptureError::Timeout,
    };
    original.run.targets[1].attempts = 1;
    original.run.state = JobState::CompletedWithErrors;
    original.run.finished_at = Some(101);
    store.finish_run(&original)?;
    let project = store.activate_baseline_with_partial_confirmation(
        original.run.id,
        project.revision,
        true,
    )?;
    let root_baseline = store.load_baseline(project.project.active_baseline.unwrap())?;
    let root_json = shipped_legacy_baseline(&root_baseline)?;
    store.connection.execute(
        "UPDATE baselines SET record=?1 WHERE id=?2",
        params![root_json, root_baseline.id.to_string()],
    )?;
    let first_check = check(&mut store, &original, &project, "first replacement")?;
    assert_eq!(first_check.run.targets.len(), 1);
    let first_analysis = analyze_image(&mut store, &original, &first_check, root_baseline.id)?;
    let keys = vec![first_check.run.targets[0].key.clone()];
    let preview = store.approval_preview(first_analysis.revision.id, &keys)?;
    let project = store.accept_selected(&preview, project.revision)?;
    let replaced = store.load_baseline(project.project.active_baseline.unwrap())?;
    assert_eq!(replaced.references[0].run, first_check.run.id);
    let legacy_json = shipped_legacy_baseline(&replaced)?;
    // This is the exact pre-feature SQL/JSON binding: source_run column follows
    // the first current reference, whose check has only one of two intended slots.
    store.connection.execute(
        "UPDATE baselines SET source_run=?1,record=?2 WHERE id=?3",
        params![
            first_check.run.id.to_string(),
            legacy_json,
            replaced.id.to_string()
        ],
    )?;
    let paths = store.paths.clone();
    drop(store);
    let mut store = Store::open(paths)?;
    let recovered = store.load_baseline(replaced.id)?;
    assert_eq!(recovered.source_run, Some(original.run.id));
    assert_eq!(recovered.intended_count, 2);
    assert_eq!(recovered.intended_targets.len(), 2);
    assert_eq!(recovered.references.len(), 1);
    assert_eq!(recovered.omitted_units(&original.run)?.len(), 1);
    let history = store.history(project.project.id, 0)?;
    assert_eq!(
        history
            .baselines
            .iter()
            .find(|b| b.id == replaced.id)
            .unwrap()
            .source_run,
        Some(original.run.id)
    );
    // A same-inventory original run exists, but without the recorded ancestor
    // link it is not recovery authority. Never search arbitrary matching runs.
    let mut disconnected: serde_json::Value = serde_json::from_str(&legacy_json)?;
    disconnected["parent"] = serde_json::Value::Null;
    store.connection.execute(
        "UPDATE baselines SET record=?1 WHERE id=?2",
        params![
            serde_json::to_string(&disconnected)?,
            replaced.id.to_string()
        ],
    )?;
    assert!(store.load_baseline(replaced.id).is_err());
    store.connection.execute(
        "UPDATE baselines SET record=?1 WHERE id=?2",
        params![legacy_json, replaced.id.to_string()],
    )?;
    let portable = store.archive_manifest(project.project.id, replaced.id)?;
    assert_eq!(portable.baseline.source_run, Some(original.run.id));
    assert_eq!(portable.baseline.intended_targets.len(), 2);
    assert!(
        portable
            .provenance
            .iter()
            .find(|p| p.run == original.run.id)
            .unwrap()
            .captures
            .is_empty(),
        "unselected original must not leak through configuration provenance"
    );
    let zip = root.path().join("shipped-legacy-partial.zip");
    store.export_archive(
        &store.export_preview(project.project.id, replaced.id)?,
        &zip,
        &AtomicBool::new(false),
    )?;
    let mut recipient = Store::open(AppPaths::isolated(&root.path().join("legacy-recipient")))?;
    let import = recipient
        .inspect_archive(&zip, &AtomicBool::new(false))?
        .preview;
    let imported = recipient.import_archive(&zip, &import, &AtomicBool::new(false))?;
    let imported_baseline = recipient.load_baseline(imported.project.active_baseline.unwrap())?;
    let imported_config = recipient.load_run(imported_baseline.frozen_source_run().unwrap())?;
    assert_eq!(
        imported_baseline.omitted_units(&imported_config.run)?.len(),
        1
    );
    let imported_source = recipient.load_run(imported_baseline.references[0].run)?;
    let imported_check = check(
        &mut recipient,
        &imported_source,
        &imported,
        "new imported check",
    )?;
    let imported_analysis = analyze_image(
        &mut recipient,
        &imported_source,
        &imported_check,
        imported_baseline.id,
    )?;
    let imported_preview = recipient.approval_preview(
        imported_analysis.revision.id,
        &[imported_check.run.targets[0].key.clone()],
    )?;
    let imported = recipient.accept_selected(&imported_preview, imported.revision)?;
    let imported_updated = recipient.load_baseline(imported.project.active_baseline.unwrap())?;
    assert_eq!(imported_updated.intended_count, 2);
    assert_eq!(imported_updated.references.len(), 1);
    assert_eq!(imported_updated.source_run, Some(imported_config.run.id));
    let second_check = check(&mut store, &first_check, &project, "second replacement")?;
    let second_analysis = analyze_image(&mut store, &first_check, &second_check, replaced.id)?;
    let preview = store.approval_preview(second_analysis.revision.id, &keys)?;
    let project = store.accept_selected(&preview, project.revision)?;
    let updated = store.load_baseline(project.project.active_baseline.unwrap())?;
    assert_eq!(updated.source_run, Some(original.run.id));
    assert_eq!(updated.intended_targets.len(), 2);
    assert_eq!(updated.references[0].run, second_check.run.id);
    assert!(updated.explicitly_partial);
    let newest_legacy = shipped_legacy_baseline(&updated)?;
    store.connection.execute(
        "UPDATE baselines SET source_run=?1,record=?2 WHERE id=?3",
        params![
            second_check.run.id.to_string(),
            newest_legacy,
            updated.id.to_string()
        ],
    )?;
    let mut terminal_cycle: serde_json::Value = serde_json::from_str(&root_json)?;
    terminal_cycle["parent"] = serde_json::to_value(updated.id)?;
    let terminal_cycle = serde_json::to_string(&terminal_cycle)?;
    store.connection.execute(
        "UPDATE baselines SET record=?1 WHERE id=?2",
        params![terminal_cycle, root_baseline.id.to_string()],
    )?;
    assert!(
        store.load_baseline(updated.id).is_err(),
        "current → intermediate → original → current cycle accepted"
    );
    assert_eq!(
        store.connection.query_row(
            "SELECT record FROM baselines WHERE id=?1",
            [root_baseline.id.to_string()],
            |r| r.get::<_, String>(0)
        )?,
        terminal_cycle,
        "failed recovery mutated terminal ancestor"
    );
    store.connection.execute(
        "UPDATE baselines SET record=?1 WHERE id=?2",
        params![root_json, root_baseline.id.to_string()],
    )?;
    assert_eq!(
        store.load_baseline(updated.id)?.source_run,
        Some(original.run.id)
    );
    assert_eq!(
        store.connection.query_row(
            "SELECT record FROM baselines WHERE id=?1",
            [root_baseline.id.to_string()],
            |r| r.get::<_, String>(0)
        )?,
        root_json
    );
    assert_eq!(
        store.connection.query_row(
            "SELECT record FROM baselines WHERE id=?1",
            [replaced.id.to_string()],
            |r| r.get::<_, String>(0)
        )?,
        legacy_json,
        "recovery must never rewrite shipped historical bytes"
    );
    Ok(())
}

#[test]
fn mapped_dom_sources_roundtrip_without_logical_url_impersonation() -> Result<()> {
    let (root, mut store, mut record, project) = fixture(CaptureMode::DomSnapshotsOnly)?;
    let mut project = project.project;
    project.environments[0].mappings = vec![OriginMapping {
        source: "http://127.0.0.1:43210/".parse()?,
        destination: "http://127.0.0.1:43211/preview/".parse()?,
    }];
    let project = store.save_project(&project, Some(1))?;
    record.run.id = RunId::new();
    record.run.environment = project.project.environments[0].clone();
    record.queue.as_mut().unwrap().project_revision = project.revision;
    store.begin_run(&record)?;
    complete(&mut store, &mut record, "mapped source")?;
    assert!(
        record.dom_captures[0]
            .metadata
            .requested
            .starts_with("http://127.0.0.1:43211/preview/")
    );
    let capture = &record.dom_captures[0];
    let bytes = store
        .read_verified_artifact(&capture.metadata.sha256, MAX_DOM_BYTES as u64)?
        .1;
    let mut forged = capture.metadata.clone();
    forged.requested = record.run.targets[0].requested.to_string();
    assert!(
        store
            .ingest_dom_snapshot(bytes.as_slice(), &forged)
            .is_err()
    );
    let project = store.activate_baseline(record.run.id, project.revision)?;
    let path = root.path().join("mapped-dom.zip");
    store.export_archive(
        &store.export_preview(project.project.id, project.project.active_baseline.unwrap())?,
        &path,
        &AtomicBool::new(false),
    )?;
    let import = store
        .inspect_archive(&path, &AtomicBool::new(false))?
        .preview;
    let imported = store.import_archive(&path, &import, &AtomicBool::new(false))?;
    let baseline = store.load_dom_baseline(imported.project.active_baseline.unwrap())?;
    let source = store.load_dom_run(baseline.dom_references[0].run)?;
    assert!(
        source.dom_captures[0]
            .metadata
            .requested
            .starts_with("http://127.0.0.1:43211/preview/")
    );
    assert_eq!(
        source.dom_captures[0].metadata.requested,
        record.dom_captures[0].metadata.requested
    );
    Ok(())
}

#[test]
fn unstable_dom_is_retained_but_partial_save_requires_explicit_eligible_slot_consent() -> Result<()>
{
    let (_root, mut store, mut record, project) = fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
    populate(&mut store, &mut record, "unstable")?;
    record.dom_captures[0].metadata.stable = false;
    record.run.state = JobState::Completed;
    record.run.finished_at = Some(101);
    store.finish_run(&record)?;
    assert_eq!(record.baseline_save_coverage().unwrap().dom_captures, 1);
    let eligible = store.baseline_eligible_coverage(record.run.id)?;
    assert_eq!(eligible.dom_captures, 0);
    assert_eq!(eligible.dom_intended, 1);
    assert_eq!(eligible.screenshot_captures, 1);
    assert_eq!(eligible.captured_targets, 1);
    assert!(
        store
            .activate_baseline(record.run.id, project.revision)
            .is_err()
    );
    let project =
        store.activate_baseline_with_partial_confirmation(record.run.id, project.revision, true)?;
    let baseline = store.load_baseline(project.project.active_baseline.unwrap())?;
    assert!(baseline.dom_references.is_empty());
    assert_eq!(baseline.omitted_units(&record.run)?.len(), 1);
    store.read_dom_snapshot(record.run.id, &record.run.targets[0].key)?;
    Ok(())
}

#[test]
fn screenshot_exact_run_receipt_needs_no_dom_files_or_analysis_but_verifies_png_sources()
-> Result<()> {
    let (_root, mut store, mut before, project) = fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
    complete(&mut store, &mut before, "before")?;
    let project = store.activate_baseline(before.run.id, project.revision)?;
    let after = check(&mut store, &before, &project, "after")?;
    for record in [&before, &after] {
        fs::write(
            store
                .paths
                .artifacts()
                .join(record.dom_captures[0].metadata.sha256.as_str()),
            b"invalid DOM sibling",
        )?;
    }
    let receipt = store.screenshot_evidence_for_run(project.project.id, after.run.id)?;
    assert_eq!(receipt.latest.unwrap().run.id, after.run.id);
    assert_eq!(
        receipt.baseline.unwrap().id,
        project.project.active_baseline.unwrap()
    );
    assert!(receipt.before.is_some());
    assert!(receipt.comparison.is_none());
    assert!(receipt.dom_comparison.is_none());
    store.project_evidence_for_run_kind(project.project.id, after.run.id, CheckKind::Screenshot)?;
    let other = Project::draft("Other exact-run scope".into(), "http://127.0.0.1:43212/")?;
    store.save_project(&other, None)?;
    assert!(
        store
            .screenshot_evidence_for_run(other.id, after.run.id)
            .is_err()
    );
    fs::write(
        store
            .paths
            .artifacts()
            .join(after.captures[0].image.tiles[0].as_str()),
        b"invalid PNG source",
    )?;
    assert!(
        store
            .screenshot_evidence_for_run(project.project.id, after.run.id)
            .is_err()
    );
    Ok(())
}

#[test]
fn approval_rejects_every_tampered_nested_consent_field_without_mutation() -> Result<()> {
    let (_root, mut store, mut before, project) = fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
    complete(&mut store, &mut before, "before")?;
    let project = store.activate_baseline(before.run.id, project.revision)?;
    let baseline = project.project.active_baseline.unwrap();
    let after = check(&mut store, &before, &project, "after")?;
    let image = analyze_image(&mut store, &before, &after, baseline)?;
    let dom = analyze_dom(&mut store, &before, &after, baseline)?;
    let keys = vec![after.run.targets[0].key.clone()];
    let preview = store.approval_preview_kinds(
        Some(image.revision.id),
        &keys,
        Some(dom.revision.id),
        &keys,
    )?;
    assert_eq!(
        preview.screenshot.as_ref().unwrap().baseline.id,
        preview.baseline.id
    );
    assert_eq!(
        preview.dom.as_ref().unwrap().baseline.id,
        preview.baseline.id
    );
    assert_eq!(
        preview.dom.as_ref().unwrap().baseline.created_at,
        preview.baseline.created_at
    );
    let revision = store.viewer_revision()?;
    type Tamper = (&'static str, fn(&mut KindApprovalPreview));
    let cases: &[Tamper] = &[
        ("DOM keep count", |p| {
            p.dom.as_mut().unwrap().inherited = 999_999
        }),
        ("screenshot keep count", |p| {
            p.screenshot.as_mut().unwrap().inherited = 999_999
        }),
        ("DOM nested retained screenshot source", |p| {
            p.dom.as_mut().unwrap().baseline.references[0].run = RunId::new()
        }),
        ("screenshot nested retained DOM source", |p| {
            p.screenshot.as_mut().unwrap().baseline.dom_references[0].run = RunId::new()
        }),
        ("DOM nested After artifact", |p| {
            p.dom.as_mut().unwrap().baseline.dom_references[0].artifact =
                "c".repeat(64).try_into().unwrap()
        }),
        ("screenshot nested After artifact", |p| {
            p.screenshot.as_mut().unwrap().baseline.references[0].artifact =
                "d".repeat(64).try_into().unwrap()
        }),
        ("DOM nested ID", |p| {
            p.dom.as_mut().unwrap().baseline.id = BaselineId::new()
        }),
        ("screenshot nested ID", |p| {
            p.screenshot.as_mut().unwrap().baseline.id = BaselineId::new()
        }),
        ("DOM nested version date", |p| {
            p.dom.as_mut().unwrap().baseline.created_at += 1
        }),
        ("screenshot nested version date", |p| {
            p.screenshot.as_mut().unwrap().baseline.created_at += 1
        }),
        ("DOM nested intended count", |p| {
            p.dom.as_mut().unwrap().baseline.intended_count += 1
        }),
        ("screenshot nested coverage flag", |p| {
            p.screenshot.as_mut().unwrap().baseline.explicitly_partial = true
        }),
        ("screenshot displayed requested URL", |p| {
            p.screenshot.as_mut().unwrap().provenance[0].1 = "tampered-consent-canary".into()
        }),
        ("screenshot displayed final URL", |p| {
            p.screenshot.as_mut().unwrap().provenance[0].2 = "tampered-consent-canary".into()
        }),
        ("screenshot displayed environment", |p| {
            p.screenshot.as_mut().unwrap().source_environments[0].1 =
                "tampered-consent-canary".into()
        }),
        ("DOM nested source date", |p| {
            p.dom.as_mut().unwrap().baseline.dom_references[0].captured_at += 1
        }),
        ("screenshot nested retained source date", |p| {
            p.screenshot.as_mut().unwrap().baseline.dom_references[0].captured_at += 1
        }),
        ("DOM nested parent", |p| {
            p.dom.as_mut().unwrap().baseline.parent = Some(BaselineId::new())
        }),
        ("screenshot nested configuration source", |p| {
            p.screenshot.as_mut().unwrap().baseline.source_run = Some(RunId::new())
        }),
    ];
    for (label, tamper) in cases {
        let mut supplied = preview.clone();
        tamper(&mut supplied);
        assert_eq!(
            supplied.baseline, preview.baseline,
            "{label} changed outer map"
        );
        assert_eq!(
            supplied.screenshot.as_ref().unwrap().context,
            preview.screenshot.as_ref().unwrap().context
        );
        assert_eq!(
            supplied.dom.as_ref().unwrap().context,
            preview.dom.as_ref().unwrap().context
        );
        let error = store.accept_selected_kinds(&supplied).expect_err(label);
        assert!(!format!("{error:#}").contains("tampered-consent-canary"));
        assert_eq!(
            store.viewer_revision()?,
            revision,
            "{label} mutated the store"
        );
        assert_eq!(
            store
                .load_project(project.project.id)?
                .project
                .active_baseline,
            Some(baseline)
        );
    }
    let mut dom_only = store.dom_approval_preview(dom.revision.id, &keys)?;
    dom_only.inherited = 999_999;
    assert!(
        store
            .accept_dom_selected(&dom_only, project.revision)
            .is_err()
    );
    let mut screenshot_only = store.approval_preview(image.revision.id, &keys)?;
    screenshot_only.source_environments[0].1 = "tampered-consent-canary".into();
    assert!(
        store
            .accept_selected(&screenshot_only, project.revision)
            .is_err()
    );
    assert_eq!(store.viewer_revision()?, revision);
    let accepted = store.accept_selected_kinds(&preview)?;
    let accepted = store.load_baseline(accepted.project.active_baseline.unwrap())?;
    assert_eq!(accepted.references[0].run, after.run.id);
    assert_eq!(accepted.dom_references[0].run, after.run.id);
    Ok(())
}

#[test]
fn shipped_single_page_without_inventory_sql_uses_only_verified_embedded_source() -> Result<()> {
    let (root, mut store, mut original, project) = fixture(CaptureMode::ScreenshotsOnly)?;
    // Exact shipped Product::capture shape: no freeze_inventory call, identity
    // version 1, no queue settings, and the singular successful capture slot.
    store.connection.execute(
        "DELETE FROM runs WHERE id=?1",
        [original.run.id.to_string()],
    )?;
    store.connection.execute(
        "DELETE FROM inventories WHERE id=?1",
        [original.run.inventory.id.to_string()],
    )?;
    original.run.inventory.identity_version = 1;
    original.queue = None;
    original.run.targets[0].attempts = 1;
    store.begin_run(&original)?;
    populate(&mut store, &mut original, "shipped-single-page")?;
    original.capture = original.captures.pop();
    original.run.state = JobState::Completed;
    original.run.finished_at = Some(101);
    store.finish_run(&original)?;
    let project = store.activate_baseline(original.run.id, project.revision)?;
    let id = project.project.active_baseline.unwrap();
    let legacy = shipped_legacy_baseline(&store.load_baseline(id)?)?;
    store.connection.execute(
        "UPDATE baselines SET record=?1 WHERE id=?2",
        params![legacy, id.to_string()],
    )?;
    let run_json: String = store.connection.query_row(
        "SELECT record FROM runs WHERE id=?1",
        [original.run.id.to_string()],
        |r| r.get(0),
    )?;
    assert!(
        !run_json.contains("dom_captures")
            && !run_json.contains("dom_outcome")
            && !run_json.contains("capture_plan")
    );
    let mut draft = project.project.clone();
    draft.sources[0] = "http://127.0.0.1:43210/unrelated-future-draft".parse()?;
    draft.profiles[0].width = 16;
    let project = store.save_replacement_project(&draft, project.revision)?;
    let paths = store.paths.clone();
    drop(store);
    let mut store = Store::open(paths)?;
    let read_revision = store.viewer_revision()?;
    store.load_run(original.run.id)?;
    let (baseline, frozen) = store.frozen_baseline_source(id)?;
    assert_eq!(frozen.run.inventory, original.run.inventory);
    assert_eq!(frozen.run.profiles, original.run.profiles);
    assert_eq!(baseline.source_run, Some(original.run.id));
    store.load_baseline(id)?;
    store.viewer_evidence(project.project.id, None)?;
    store.screenshot_evidence_for_run(project.project.id, original.run.id)?;
    store.history(project.project.id, 0)?;
    let manifest = store.archive_manifest(project.project.id, id)?;
    assert_eq!(manifest.inventory, original.run.inventory);
    assert_eq!(manifest.project.sources, draft.sources);
    assert_eq!(manifest.provenance[0].profiles, original.run.profiles);
    let path = root.path().join("shipped-no-inventory.zip");
    store.export_archive(
        &store.export_preview(project.project.id, id)?,
        &path,
        &AtomicBool::new(false),
    )?;
    assert_eq!(
        store.viewer_revision()?,
        read_revision,
        "read/export backfilled or rewrote SQL"
    );
    assert_eq!(
        store
            .connection
            .query_row("SELECT count(*) FROM inventories", [], |r| r
                .get::<_, u64>(0))?,
        0
    );
    // Absence is the only fallback condition. A corrupt, misbound, or merely
    // different existing row must reject rather than using the embedded copy.
    let mut different = original.run.inventory.clone();
    different.pages[0].canonical = "http://127.0.0.1:43210/not-the-captured-page".parse()?;
    let mut misbound = original.run.inventory.clone();
    misbound.project = ProjectId::new();
    for record in [
        "not JSON".to_owned(),
        serde_json::to_string(&different)?,
        serde_json::to_string(&misbound)?,
    ] {
        store.connection.execute(
            "INSERT INTO inventories VALUES(?1,?2,?3)",
            params![
                original.run.inventory.id.to_string(),
                project.project.id.to_string(),
                record
            ],
        )?;
        let revision = store.viewer_revision()?;
        assert!(store.frozen_baseline_source(id).is_err());
        assert!(store.archive_manifest(project.project.id, id).is_err());
        assert_eq!(
            store.viewer_revision()?,
            revision,
            "failed read repaired conflicting inventory row"
        );
        assert_eq!(
            store.connection.query_row(
                "SELECT record FROM inventories WHERE id=?1",
                [original.run.inventory.id.to_string()],
                |r| r.get::<_, String>(0)
            )?,
            record
        );
        store.connection.execute(
            "DELETE FROM inventories WHERE id=?1",
            [original.run.inventory.id.to_string()],
        )?;
    }
    let mut after = original.clone();
    after.run.id = RunId::new();
    after.run.intent = RunIntent::Check { baseline: id };
    after.run.state = JobState::Running;
    after.run.started_at = Some(200);
    after.run.finished_at = None;
    after.capture = None;
    after.captures.clear();
    after.run.targets[0].outcome = TargetOutcome::Pending;
    after.run.targets[0].final_url = None;
    store.begin_run(&after)?;
    complete(&mut store, &mut after, "new legacy check")?;
    let analysis = analyze_image(&mut store, &original, &after, id)?;
    let preview =
        store.approval_preview(analysis.revision.id, &[after.run.targets[0].key.clone()])?;
    let updated = store.accept_selected(&preview, project.revision)?;
    let replaced = updated.project.active_baseline.unwrap();
    assert_eq!(
        store.frozen_baseline_source(replaced)?.1.run.inventory,
        original.run.inventory
    );
    assert_eq!(
        store
            .archive_manifest(project.project.id, replaced)?
            .inventory,
        original.run.inventory
    );
    store.restore_baseline(id, updated.revision)?;
    store.history(project.project.id, 0)?;
    assert_eq!(
        store
            .connection
            .query_row("SELECT count(*) FROM inventories", [], |r| r
                .get::<_, u64>(0))?,
        0
    );
    assert_eq!(
        store.connection.query_row(
            "SELECT record FROM runs WHERE id=?1",
            [original.run.id.to_string()],
            |r| r.get::<_, String>(0)
        )?,
        run_json
    );
    assert_eq!(
        store.connection.query_row(
            "SELECT record FROM baselines WHERE id=?1",
            [id.to_string()],
            |r| r.get::<_, String>(0)
        )?,
        legacy
    );
    let mut recipient = Store::open(AppPaths::isolated(
        &root.path().join("no-inventory-recipient"),
    ))?;
    let import = recipient
        .inspect_archive(&path, &AtomicBool::new(false))?
        .preview;
    let imported = recipient.import_archive(&path, &import, &AtomicBool::new(false))?;
    let (_, source) =
        recipient.frozen_baseline_source(imported.project.active_baseline.unwrap())?;
    assert_eq!(source.run.inventory.identity_version, 1);
    assert_eq!(
        source.run.inventory.pages[0].canonical,
        original.run.inventory.pages[0].canonical
    );
    assert_eq!(source.run.profiles[0].width, original.run.profiles[0].width);
    assert_eq!(imported.project.sources, draft.sources);
    Ok(())
}

#[test]
fn archive_version_preserves_current_mode_and_independent_frozen_baseline_mode() -> Result<()> {
    for (saved, future, version) in [
        (
            CaptureMode::ScreenshotsOnly,
            CaptureMode::ScreenshotsOnly,
            1,
        ),
        (
            CaptureMode::ScreenshotsOnly,
            CaptureMode::DomSnapshotsOnly,
            2,
        ),
        (
            CaptureMode::ScreenshotsOnly,
            CaptureMode::ScreenshotsAndDomSnapshots,
            2,
        ),
        (
            CaptureMode::DomSnapshotsOnly,
            CaptureMode::ScreenshotsOnly,
            2,
        ),
        (
            CaptureMode::ScreenshotsAndDomSnapshots,
            CaptureMode::ScreenshotsOnly,
            2,
        ),
    ] {
        let (root, mut store, mut before, project) = fixture(saved)?;
        complete(&mut store, &mut before, "baseline")?;
        let project = store.activate_baseline(before.run.id, project.revision)?;
        let id = project.project.active_baseline.unwrap();
        let previous = store.archive_manifest(project.project.id, id)?;
        let mut draft = project.project.clone();
        draft.capture_plan = CapturePlan::for_mode(future);
        let project = store.save_replacement_project(&draft, project.revision)?;
        let revision = store.viewer_revision()?;
        let manifest = store.archive_manifest(project.project.id, id)?;
        assert_eq!(manifest.version, version);
        assert_eq!(manifest.project.capture_plan.mode, future);
        assert_eq!(manifest.baseline.capture_plan.mode, saved);
        assert_eq!(manifest.baseline.references, previous.baseline.references);
        assert_eq!(
            manifest.baseline.dom_references,
            previous.baseline.dom_references
        );
        assert_eq!(
            manifest.baseline.intended_count,
            previous.baseline.intended_count
        );
        assert_eq!(
            manifest.baseline.explicitly_partial,
            previous.baseline.explicitly_partial
        );
        assert_eq!(
            serde_json::to_value(&manifest.provenance)?,
            serde_json::to_value(&previous.provenance)?
        );
        assert_eq!(manifest.artifacts, previous.artifacts);
        let path = root.path().join("independent-future-mode.zip");
        store.export_archive(
            &store.export_preview(project.project.id, id)?,
            &path,
            &AtomicBool::new(false),
        )?;
        assert_eq!(
            store.viewer_revision()?,
            revision,
            "export rewrote current capture settings"
        );
        assert_eq!(
            store.load_project(project.project.id)?.project.capture_plan,
            draft.capture_plan
        );
        let mut recipient = Store::open(AppPaths::isolated(&root.path().join("mode-recipient")))?;
        let import = recipient
            .inspect_archive(&path, &AtomicBool::new(false))?
            .preview;
        let imported = recipient.import_archive(&path, &import, &AtomicBool::new(false))?;
        assert_eq!(imported.project.capture_plan.mode, future);
        let (baseline, source) =
            recipient.frozen_baseline_source(imported.project.active_baseline.unwrap())?;
        assert_eq!(baseline.capture_plan.mode, saved);
        assert_eq!(source.run.capture_plan.mode, saved);
        let checked = check(&mut recipient, &source, &imported, "after import")?;
        assert_eq!(checked.run.capture_plan.mode, saved);
        assert_eq!(!checked.captures.is_empty(), saved.screenshots());
        assert_eq!(!checked.dom_captures.is_empty(), saved.dom_snapshots());
    }
    Ok(())
}

mod historical_v1 {
    use super::*;
    use regressee_core::archive::{
        ArchiveManifest, PortableEnvironment, PortableMask, artifact_entry,
    };

    // Exact changed container field sets from HEAD 8381759, not the current
    // additive domain/archive types. Screenshot leaf schemas remain unchanged.
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ProjectV1 {
        name: String,
        sources: Vec<url::Url>,
        discovery: DiscoverySettings,
        environments: Vec<PortableEnvironment>,
        profiles: Vec<CaptureProfile>,
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct BaselineV1 {
        id: BaselineId,
        project: ProjectId,
        parent: Option<BaselineId>,
        inventory: InventoryId,
        created_at: i64,
        references: Vec<BaselineReference>,
        intended_count: u32,
        explicitly_partial: bool,
    }
    #[derive(Debug, Serialize, Deserialize)]
    enum KindV1 {
        Screenshot,
        Thumbnail,
        Tile,
        Difference,
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ArtifactV1 {
        id: ArtifactId,
        bytes: u64,
        kind: KindV1,
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct SourceV1 {
        run: RunId,
        rendering: RenderingEnvironment,
        profiles: Vec<CaptureProfile>,
        environment: PortableEnvironment,
        started_at: i64,
        finished_at: i64,
        captures: Vec<CaptureMetadata>,
        discovery: DiscoverySettings,
    }
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ManifestV1 {
        version: u32,
        project: ProjectV1,
        inventory: InventoryRevision,
        baseline: BaselineV1,
        artifacts: Vec<ArtifactV1>,
        provenance: Vec<SourceV1>,
        masks: Vec<PortableMask>,
    }

    fn write_untrusted(
        store: &Store,
        path: &Path,
        manifest: &serde_json::Value,
        artifacts: &[Artifact],
    ) -> Result<()> {
        let mut zip = zip::ZipWriter::new(fs::File::create(path)?);
        let options = zip::write::SimpleFileOptions::default()
            .compression_method(zip::CompressionMethod::Stored)
            .unix_permissions(0o600);
        zip.start_file("manifest.json", options)?;
        zip.write_all(&serde_json::to_vec(manifest)?)?;
        for artifact in artifacts {
            zip.start_file(artifact_entry(&artifact.id), options)?;
            zip.write_all(&fs::read(store.artifact_path(&artifact.id)?)?)?;
        }
        zip.finish()?;
        Ok(())
    }

    #[test]
    fn simple_screenshot_zip_is_genuinely_compatible_with_shipped_strict_v1_schema() -> Result<()> {
        let (root, mut store, mut before, project) = fixture(CaptureMode::ScreenshotsOnly)?;
        complete(&mut store, &mut before, "before")?;
        let project = store.activate_baseline(before.run.id, project.revision)?;
        let id = project.project.active_baseline.unwrap();
        let original_json: String = store.connection.query_row(
            "SELECT record FROM baselines WHERE id=?1",
            [id.to_string()],
            |r| r.get(0),
        )?;
        assert!(original_json.contains("source_run") && original_json.contains("intended_targets"));
        let revision = store.viewer_revision()?;
        let manifest = store.archive_manifest(project.project.id, id)?;
        assert_eq!(manifest.version, 1);
        assert!(
            manifest.baseline.source_run.is_none() && manifest.baseline.intended_targets.is_empty()
        );
        let path = root.path().join("strict-shipped-v1.zip");
        store.export_archive(
            &store.export_preview(project.project.id, id)?,
            &path,
            &AtomicBool::new(false),
        )?;
        let mut zip = zip::ZipArchive::new(fs::File::open(&path)?)?;
        let mut bytes = Vec::new();
        zip.by_name("manifest.json")?.read_to_end(&mut bytes)?;
        let historical: ManifestV1 = serde_json::from_slice(&bytes)?;
        assert_eq!(historical.version, 1);
        assert_eq!(historical.baseline.references[0].run, before.run.id);
        assert_eq!(historical.provenance.len(), 1);
        assert_eq!(historical.provenance[0].captures.len(), 1);
        let raw: serde_json::Value = serde_json::from_slice(&bytes)?;
        assert_eq!(serde_json::to_value(&historical)?, raw);
        // Nondefault V2 hints and even explicit newer default fields cannot be
        // smuggled into the old version. Local hints are rebuilt only on import.
        let mut cases = Vec::new();
        let mut source = raw.clone();
        source["baseline"]["source_run"] = serde_json::to_value(before.run.id)?;
        cases.push(("source", source));
        let mut intended = raw.clone();
        intended["baseline"]["intended_targets"] = serde_json::to_value(
            before
                .run
                .targets
                .iter()
                .map(|t| t.key.clone())
                .collect::<Vec<_>>(),
        )?;
        cases.push(("intended", intended));
        let mut default_source = raw.clone();
        default_source["baseline"]["source_run"] = serde_json::Value::Null;
        cases.push(("default-source", default_source));
        let mut default_intended = raw.clone();
        default_intended["baseline"]["intended_targets"] = serde_json::json!([]);
        cases.push(("default-intended", default_intended));
        let mut project_plan = raw.clone();
        project_plan["project"]["capture_plan"] = serde_json::to_value(CapturePlan::default())?;
        cases.push(("project-plan", project_plan));
        let mut baseline_dom = raw.clone();
        baseline_dom["baseline"]["dom_references"] = serde_json::json!([]);
        cases.push(("baseline-dom", baseline_dom));
        let mut source_plan = raw.clone();
        source_plan["provenance"][0]["capture_plan"] =
            serde_json::to_value(CapturePlan::default())?;
        cases.push(("source-plan", source_plan));
        let mut source_dom = raw.clone();
        source_dom["provenance"][0]["dom_captures"] = serde_json::json!([]);
        cases.push(("source-dom", source_dom));
        let preview = store
            .inspect_archive(&path, &AtomicBool::new(false))?
            .preview;
        for (label, payload) in cases {
            assert!(serde_json::from_value::<ManifestV1>(payload.clone()).is_err());
            let hostile = root.path().join(format!("mislabelled-v1-{label}.zip"));
            write_untrusted(&store, &hostile, &payload, &manifest.artifacts)?;
            assert!(
                store
                    .inspect_archive(&hostile, &AtomicBool::new(false))
                    .is_err(),
                "accepted {label}"
            );
            assert!(
                store
                    .import_archive(&hostile, &preview, &AtomicBool::new(false))
                    .is_err(),
                "import accepted {label}"
            );
            assert_eq!(
                store.viewer_revision()?,
                revision,
                "invalid {label} mutated SQL"
            );
        }
        assert_eq!(
            store.connection.query_row(
                "SELECT record FROM baselines WHERE id=?1",
                [id.to_string()],
                |r| r.get::<_, String>(0)
            )?,
            original_json
        );
        let local = store.load_baseline(id)?;
        assert_eq!(local.source_run, Some(before.run.id));
        assert_eq!(local.intended_targets.len(), 1);
        let mut recipient =
            Store::open(AppPaths::isolated(&root.path().join("strict-v1-recipient")))?;
        let preview = recipient
            .inspect_archive(&path, &AtomicBool::new(false))?
            .preview;
        let imported = recipient.import_archive(&path, &preview, &AtomicBool::new(false))?;
        let (baseline, source) =
            recipient.frozen_baseline_source(imported.project.active_baseline.unwrap())?;
        assert_eq!(baseline.source_run, Some(source.run.id));
        assert_eq!(baseline.intended_targets.len(), 1);
        assert_eq!(source.run.capture_plan.mode, CaptureMode::ScreenshotsOnly);
        Ok(())
    }

    #[test]
    fn screenshot_composite_with_nonredundant_frozen_source_is_explicit_v2_without_sibling_leak()
    -> Result<()> {
        let (_root, mut store, mut before, project) = fixture(CaptureMode::ScreenshotsOnly)?;
        complete(&mut store, &mut before, "before")?;
        let project = store.activate_baseline(before.run.id, project.revision)?;
        let old = project.project.active_baseline.unwrap();
        let after = check(&mut store, &before, &project, "after")?;
        let analysis = analyze_image(&mut store, &before, &after, old)?;
        let preview =
            store.approval_preview(analysis.revision.id, &[after.run.targets[0].key.clone()])?;
        let project = store.accept_selected(&preview, project.revision)?;
        let manifest: ArchiveManifest =
            store.archive_manifest(project.project.id, project.project.active_baseline.unwrap())?;
        assert_eq!(manifest.version, 2);
        assert_eq!(manifest.baseline.source_run, Some(before.run.id));
        assert_eq!(manifest.baseline.intended_targets.len(), 1);
        assert!(
            manifest
                .provenance
                .iter()
                .find(|p| p.run == before.run.id)
                .unwrap()
                .captures
                .is_empty()
        );
        assert_eq!(
            manifest.provenance.iter().flat_map(|p| &p.captures).count(),
            1
        );
        assert_eq!(manifest.artifacts.len(), 1);
        assert_eq!(manifest.artifacts[0].id, after.captures[0].image.original);
        assert!(
            manifest
                .artifacts
                .iter()
                .all(|a| a.id != before.captures[0].image.original)
        );
        Ok(())
    }
}
