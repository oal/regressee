// SPDX-License-Identifier: MPL-2.0
//! Distinct persisted synthetic identities AND PNGs; not browser throughput.
use regressee_core::{domain::*, evidence::*};
use regressee_storage::{AppPaths, Store};
use std::{path::Path, sync::atomic::AtomicBool};

pub fn fixture(path: &Path, pages: usize) -> (Store, ProjectId, RunId, AnalysisId) {
    assert!((100..=1000).contains(&pages));
    let mut store = Store::open(AppPaths::isolated(path)).unwrap();
    let mut project = Project::draft(
        "Qualification · very long project identity · ÆØÅ 日本語 · retained independent pages"
            .into(),
        "http://127.0.0.1:43210/qualification/",
    )
    .unwrap();
    project.profiles[0].width = 96;
    project.profiles[0].height = 64;
    project.profiles[0].runtime = RuntimeIdentity::Pinned {
        node: "synthetic".into(),
        worker: "synthetic".into(),
        playwright: "synthetic".into(),
        browser: "synthetic".into(),
    };
    let base = project.sources[0].clone();
    let inventory = InventoryRevision {
        id: InventoryId::new(), project: project.id, identity_version: 2,
        pages: (0..pages).map(|i| InventoryPage {
            id: PageId::new(),
            canonical: base.join(&format!("page-{i:04}/very-long-independent-release-notes-and-delivery-preferences/?locale=nb-NO&distinct={i}&return=%2Faccount%2Forders%2F#details")).unwrap(),
            included: true, reason: None,
        }).collect(), coverage: DiscoveryCoverage::Complete,
    };
    project.sources[0] = inventory.pages[0].canonical.clone();
    store.save_project(&project, None).unwrap();
    store.freeze_inventory(&inventory).unwrap();
    let cancel = AtomicBool::new(false);
    let scratch = tempfile::tempdir_in(path).unwrap();
    let mut baseline = None;
    let mut latest = None;
    for sequence in 0..2 {
        let mut record = RunRecord {
            run: CaptureRun {
                id: RunId::new(),
                project: project.id,
                intent: baseline.map_or(RunIntent::NewBaseline, |baseline| RunIntent::Check {
                    baseline,
                }),
                inventory: inventory.clone(),
                profiles: project.profiles.clone(),
                environment: project.environments[0].clone(),
                rendering: RenderingEnvironment {
                    os: "synthetic-linux".into(),
                    architecture: "x86_64".into(),
                    browser: "synthetic".into(),
                    worker_version: "synthetic".into(),
                    profile_version: 1,
                    fonts_fingerprint: Some("synthetic".into()),
                },
                state: JobState::Running,
                started_at: Some(1_791_792_000 + sequence * 3600),
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
        store.begin_run(&record).unwrap();
        for (i, target) in record.run.targets.iter_mut().enumerate() {
            // A real persisted gap, not a cleared/cloned viewer.
            if sequence == 1 && i == pages - 1 {
                target.outcome = TargetOutcome::Failed {
                    code: CaptureError::NeedsLogin,
                };
                continue;
            }
            let mut pixels = vec![255u8; 96 * 64 * 3];
            // Encode the page number into source pixels so CAS deduplication cannot
            // turn this into the old repeated-image presentation fixture.
            for (bit, pixel) in pixels.chunks_exact_mut(3).take(16).enumerate() {
                pixel.copy_from_slice(if i & (1 << bit) == 0 {
                    &[20, 70, 100]
                } else {
                    &[100, 70, 20]
                });
            }
            for y in 16..48 {
                for x in 16..80 {
                    let n = (y * 96 + x) * 3;
                    pixels[n..n + 3].copy_from_slice(if sequence == 1 && i % 2 == 0 {
                        &[200, 30, 50]
                    } else {
                        &[10, 120, 110]
                    });
                }
            }
            let png = scratch.path().join("owned.png");
            {
                let mut encoder = png::Encoder::new(std::fs::File::create(&png).unwrap(), 96, 64);
                encoder.set_color(png::ColorType::Rgb);
                encoder.set_depth(png::BitDepth::Eight);
                encoder
                    .write_header()
                    .unwrap()
                    .write_image_data(&pixels)
                    .unwrap();
            }
            let image =
                regressee_app::images::ingest(&mut store, &png, ArtifactKind::Screenshot, &cancel)
                    .unwrap();
            let bytes = store.verify_artifact(&image.original).unwrap().bytes;
            target.outcome = TargetOutcome::Captured {
                artifact: image.original.clone(),
                width: image.width,
                height: image.height,
            };
            target.final_url = Some(target.requested.clone());
            record.captures.push(CaptureEvidence {
                metadata: CaptureMetadata {
                    version: 1,
                    key: target.key.clone(),
                    requested: target.requested.to_string(),
                    final_url: target.requested.to_string(),
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
                    // Required portable protocol value, NOT a browser claim:
                    // runtime/rendering identity and fixture docs say synthetic.
                    capture_mode: "chromium-new-headless".into(),
                    replacement: None,
                },
                image,
            });
        }
        record.run.state = if sequence == 0 {
            JobState::Completed
        } else {
            JobState::CompletedWithErrors
        };
        record.run.finished_at = record.run.started_at.map(|t| t + 1);
        store.finish_run(&record).unwrap();
        if sequence == 0 {
            baseline = store
                .activate_baseline(record.run.id, 1)
                .unwrap()
                .project
                .active_baseline;
        } else {
            latest = Some(record.run.id);
        }
    }
    let run = latest.unwrap();
    let analysis = regressee_app::review::analyze_run(
        &mut store,
        run,
        regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
        &cancel,
    )
    .unwrap();
    (store, project.id, run, analysis.revision.id)
}
