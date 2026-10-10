// SPDX-License-Identifier: MPL-2.0
//! Saved corpus pixels with synthetic capture metadata; no browser/native claim.
use regressee_core::{domain::*, evidence::*};
use regressee_storage::{AppPaths, Store};
use std::{path::Path, sync::atomic::AtomicBool};

pub fn fixture(path: &Path) -> (Store, ProjectId, RunId, AnalysisId, AnalysisId) {
    let mut store = Store::open(AppPaths::isolated(path)).unwrap();
    let mut project =
        Project::draft("Approval UI fixture".into(), "http://127.0.0.1:43210/").unwrap();
    project.profiles[0].width = 96;
    project.profiles[0].height = 64;
    project.profiles[0].runtime = RuntimeIdentity::Pinned {
        node: "fixture".into(),
        worker: "fixture".into(),
        playwright: "fixture".into(),
        browser: "fixture".into(),
    };
    store.save_project(&project, None).unwrap();
    let inventory = InventoryRevision {
        id: InventoryId::new(),
        project: project.id,
        identity_version: 2,
        pages: vec![InventoryPage {
            id: PageId::new(),
            canonical: project.sources[0].clone(),
            included: true,
            reason: None,
        }],
        coverage: DiscoveryCoverage::Complete,
    };
    store.freeze_inventory(&inventory).unwrap();
    let cancel = AtomicBool::new(false);
    let old = {
        let mut save_run = |intent, filename: &str| {
            let key = TargetKey {
                page: inventory.pages[0].id,
                profile: project.profiles[0].id,
            };
            let mut run = RunRecord {
                run: CaptureRun {
                    id: RunId::new(),
                    project: project.id,
                    intent,
                    inventory: inventory.clone(),
                    profiles: project.profiles.clone(),
                    environment: project.environments[0].clone(),
                    rendering: RenderingEnvironment {
                        os: "fixture".into(),
                        architecture: "fixture".into(),
                        browser: "fixture".into(),
                        worker_version: "fixture".into(),
                        profile_version: 1,
                        fonts_fingerprint: Some("fixture".into()),
                    },
                    state: JobState::Running,
                    started_at: Some(100),
                    finished_at: None,
                    targets: vec![CaptureTarget {
                        key: key.clone(),
                        requested: project.sources[0].clone(),
                        final_url: None,
                        attempts: 1,
                        outcome: TargetOutcome::Pending,
                        warnings: vec![],
                    }],
                },
                capture: None,
                captures: vec![],
                queue: None,
            };
            store.begin_run(&run).unwrap();
            let image = regressee_app::images::ingest(
                &mut store,
                &Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/fixtures/analysis-proposal/images")
                    .join(filename),
                ArtifactKind::Screenshot,
                &cancel,
            )
            .unwrap();
            let bytes = store.verify_artifact(&image.original).unwrap().bytes;
            run.run.targets[0].outcome = TargetOutcome::Captured {
                artifact: image.original.clone(),
                width: image.width,
                height: image.height,
            };
            run.run.targets[0].final_url = Some(project.sources[0].clone());
            run.captures.push(CaptureEvidence {
                metadata: CaptureMetadata {
                    version: 1,
                    key,
                    requested: project.sources[0].to_string(),
                    final_url: project.sources[0].to_string(),
                    runtime: project.profiles[0].runtime.clone(),
                    profile: project.profiles[0].clone(),
                    width: image.width,
                    height: image.height,
                    sha256: image.original.clone(),
                    bytes,
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
            run.run.state = JobState::Completed;
            run.run.finished_at = Some(101);
            store.finish_run(&run).unwrap();
            run.run.id
        };
        save_run(RunIntent::NewBaseline, "unchanged-before.png")
    };
    let active = store.activate_baseline(old, 1).unwrap();
    let baseline = active.project.active_baseline.unwrap();
    let mut check = store.load_run(old).unwrap();
    check.run.id = RunId::new();
    check.run.intent = RunIntent::Check { baseline };
    check.run.state = JobState::Running;
    check.run.finished_at = None;
    check.run.targets[0].outcome = TargetOutcome::Pending;
    check.captures.clear();
    store.begin_run(&check).unwrap();
    let image = regressee_app::images::ingest(
        &mut store,
        &Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../tests/fixtures/analysis-proposal/images/ignored-rectangles-after.png"),
        ArtifactKind::Screenshot,
        &cancel,
    )
    .unwrap();
    let mut evidence = store.load_run(old).unwrap().captures.remove(0);
    evidence.metadata.width = image.width;
    evidence.metadata.height = image.height;
    evidence.metadata.sha256 = image.original.clone();
    evidence.metadata.bytes = store.verify_artifact(&image.original).unwrap().bytes;
    evidence.image = image;
    check.run.targets[0].outcome = TargetOutcome::Captured {
        artifact: evidence.image.original.clone(),
        width: evidence.image.width,
        height: evidence.image.height,
    };
    check.captures.push(evidence);
    check.run.state = JobState::Completed;
    check.run.finished_at = Some(102);
    store.finish_run(&check).unwrap();
    let a = regressee_app::review::analyze_run(
        &mut store,
        check.run.id,
        regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
        &cancel,
    )
    .unwrap();
    let mut request = a.perceptual.as_ref().unwrap().reports[0].1.request.clone();
    request.mask = Some(regressee_app::analysis_engine::MaskBinding {
        revision: MaskId::new().to_string(),
        target: request.before.target.clone(),
        before_capture_id: request.before.capture_id.clone(),
        after_capture_id: request.after.capture_id.clone(),
        before_dimensions: request.before.dimensions,
        after_dimensions: request.after.dimensions,
        rectangles: vec![regressee_app::analysis_engine::Rectangle {
            x: 0,
            y: 0,
            width: request.before.dimensions.width,
            height: request.before.dimensions.height,
        }],
    });
    store.save_mask(project.id, &request).unwrap();
    let masked = regressee_app::review::analyze_run(
        &mut store,
        check.run.id,
        regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
        &cancel,
    )
    .unwrap();
    assert_eq!(
        masked.revision.results[0].classification,
        Classification::NotComparable
    );
    store.display_analysis(a.revision.id).unwrap();
    (
        store,
        project.id,
        check.run.id,
        a.revision.id,
        masked.revision.id,
    )
}
