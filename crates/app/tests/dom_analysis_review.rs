// SPDX-License-Identifier: MPL-2.0
//! Inert, synthetic local-store integration. No runtime resolution/site visits.
use anyhow::Result;
use regressee_app::{analysis_engine, dom_snapshot, images, review};
use regressee_core::{dom_snapshot::*, domain::*, evidence::*};
use regressee_storage::{AppPaths, Store};
use std::{io::Cursor, sync::atomic::AtomicBool};

fn canonical(text: &str) -> Result<Vec<u8>> {
    DomSnapshot {
        nodes: vec![
            DomNode::Document,
            DomNode::Element {
                depth: 1,
                namespace: HTML_NAMESPACE.into(),
                name: "html".into(),
                prefix: None,
                attributes: vec![],
            },
            DomNode::Text {
                depth: 2,
                text: text.into(),
            },
        ],
    }
    .canonical_bytes()
}

fn empty_run(project: &Project, inventory: InventoryRevision, intent: RunIntent) -> RunRecord {
    let profile = &project.profiles[0];
    let page = &inventory.pages[0];
    let target = CaptureTarget {
        key: TargetKey {
            page: page.id,
            profile: profile.id,
        },
        requested: page.canonical.clone(),
        final_url: None,
        attempts: 1,
        outcome: if project.capture_plan.mode.screenshots() {
            TargetOutcome::Pending
        } else {
            TargetOutcome::NotRequested
        },
        dom_outcome: if project.capture_plan.mode.dom_snapshots() {
            DomTargetOutcome::Pending
        } else {
            DomTargetOutcome::NotRequested
        },
        warnings: vec![],
    };
    RunRecord {
        run: CaptureRun {
            id: RunId::new(),
            project: project.id,
            intent,
            inventory,
            profiles: project.profiles.clone(),
            capture_plan: project.capture_plan.clone(),
            environment: project.environments[0].clone(),
            rendering: RenderingEnvironment {
                os: "synthetic".into(),
                architecture: "synthetic".into(),
                browser: "synthetic".into(),
                worker_version: "synthetic".into(),
                profile_version: 1,
                fonts_fingerprint: None,
            },
            state: JobState::Running,
            started_at: Some(1),
            finished_at: None,
            targets: vec![target],
        },
        capture: None,
        captures: vec![],
        dom_captures: vec![],
        queue: None,
    }
}

fn publish(
    store: &mut Store,
    record: &mut RunRecord,
    image_ok: bool,
    dom_ok: bool,
    stable: bool,
    after: bool,
) -> Result<()> {
    let target = &record.run.targets[0];
    let profile = record.run.profiles[0].clone();
    let address = regressee_core::environment::map_url(&target.requested, &record.run.environment)?
        .to_string();
    let key = target.key.clone();
    if !matches!(record.run.targets[0].outcome, TargetOutcome::NotRequested) {
        if image_ok {
            let stage = tempfile::tempdir_in(&store.paths().cache)?;
            let path = stage.path().join("synthetic.png");
            let mut encoder = png::Encoder::new(std::fs::File::create(&path)?, 32, 32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()?
                .write_image_data(&[if after { 220 } else { 20 }, 40, 80, 255].repeat(32 * 32))?;
            let image = images::ingest(
                store,
                &path,
                ArtifactKind::Screenshot,
                &AtomicBool::new(false),
            )?;
            let bytes = store.verify_artifact(&image.original)?.bytes;
            record.captures.push(CaptureEvidence {
                metadata: CaptureMetadata {
                    version: 1,
                    key: key.clone(),
                    requested: address.clone(),
                    final_url: address.clone(),
                    runtime: profile.runtime.clone(),
                    profile: profile.clone(),
                    width: 32,
                    height: 32,
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
                image: image.clone(),
            });
            record.run.targets[0].outcome = TargetOutcome::Captured {
                artifact: image.original,
                width: 32,
                height: 32,
            };
        } else {
            record.run.targets[0].outcome = TargetOutcome::Failed {
                code: CaptureError::Timeout,
            };
        }
    }
    if !record.run.targets[0].dom_outcome.is_not_requested() {
        if dom_ok {
            let bytes = canonical(if after {
                "hydrated after"
            } else {
                "hydrated before"
            })?;
            let stats = DomSnapshot::from_canonical(&bytes)?.validate()?;
            let metadata = DomCaptureMetadata {
                version: 1,
                run: record.run.id,
                key,
                requested: address.clone(),
                final_url: address.clone(),
                runtime: profile.runtime.clone(),
                profile,
                policy: record.run.capture_plan.dom_policy.unwrap(),
                sha256: dom_artifact_id(&bytes),
                bytes: bytes.len() as u64,
                node_count: stats.node_count,
                visited_nodes: stats.node_count,
                max_depth: stats.max_depth,
                coverage: stats.coverage,
                warnings: if stable {
                    vec![]
                } else {
                    vec!["Unstable".into()]
                },
                stable,
                duration_ms: 1,
                readiness: Readiness {
                    font_failures: 0,
                    image_failures: 0,
                    resource_failures: 0,
                },
            };
            record
                .dom_captures
                .push(store.ingest_dom_snapshot(Cursor::new(bytes), &metadata)?);
            record.run.targets[0].dom_outcome = DomTargetOutcome::Captured {
                artifact: metadata.sha256,
            };
        } else {
            record.run.targets[0].dom_outcome = DomTargetOutcome::Failed {
                code: DomCaptureError::LimitExceeded,
            };
        }
    }
    if record.run.targets[0].any_kind_captured() {
        record.run.targets[0].final_url = Some(address.parse()?);
    }
    record.run.state = if record
        .run
        .targets
        .iter()
        .all(CaptureTarget::requested_kinds_captured)
    {
        JobState::Completed
    } else {
        JobState::CompletedWithErrors
    };
    record.run.finished_at = Some(2);
    store.finish_run(record)
}

fn fixture(
    mode: CaptureMode,
) -> Result<(
    tempfile::TempDir,
    Store,
    Project,
    InventoryRevision,
    BaselineId,
)> {
    let root = tempfile::tempdir()?;
    let mut store = Store::open(AppPaths::isolated(root.path()))?;
    let mut project = Project::draft("Synthetic DOM analysis".into(), "http://127.0.0.1:9/inert")?;
    project.capture_plan = CapturePlan::for_mode(mode);
    project.profiles[0].width = 32;
    project.profiles[0].height = 32;
    project.profiles[0].runtime = RuntimeIdentity::Pinned {
        node: "synthetic".into(),
        worker: "synthetic".into(),
        playwright: "synthetic".into(),
        browser: "synthetic".into(),
    };
    let saved = store.save_project(&project, None)?;
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
    store.freeze_inventory(&inventory)?;
    let mut before = empty_run(&project, inventory.clone(), RunIntent::NewBaseline);
    store.begin_run(&before)?;
    publish(&mut store, &mut before, true, true, true, false)?;
    let saved = store.activate_baseline(before.run.id, saved.revision)?;
    let baseline = saved.project.active_baseline.unwrap();
    Ok((root, store, saved.project, inventory, baseline))
}

#[test]
fn three_mode_outcome_matrix_preserves_sibling_success_and_unstable_dom_is_not_comparable()
-> Result<()> {
    for mode in [
        CaptureMode::ScreenshotsOnly,
        CaptureMode::ScreenshotsAndDomSnapshots,
        CaptureMode::DomSnapshotsOnly,
    ] {
        for image_ok in [true, false] {
            for (dom_ok, stable) in [(true, true), (false, true), (true, false)] {
                if !mode.screenshots() && !image_ok {
                    continue;
                }
                if !mode.dom_snapshots() && (!dom_ok || !stable) {
                    continue;
                }
                let (_root, mut store, project, inventory, baseline) = fixture(mode)?;
                let mut after = empty_run(&project, inventory, RunIntent::Check { baseline });
                store.begin_run(&after)?;
                publish(&mut store, &mut after, image_ok, dom_ok, stable, true)?;
                let reopened = store.load_run(after.run.id)?;
                assert_eq!(
                    reopened.captures.len(),
                    usize::from(mode.screenshots() && image_ok)
                );
                assert_eq!(
                    reopened.dom_captures.len(),
                    usize::from(mode.dom_snapshots() && dom_ok)
                );
                if mode.screenshots() {
                    let image = review::analyze_run(
                        &mut store,
                        after.run.id,
                        review::preset(analysis_engine::Preset::Balanced),
                        &AtomicBool::new(false),
                    )?;
                    assert_eq!(
                        image.revision.results[0].classification,
                        if image_ok {
                            Classification::Changed
                        } else {
                            Classification::CouldNotCapture
                        }
                    );
                }
                if mode.dom_snapshots() {
                    let dom = dom_snapshot::analyze_run(
                        &mut store,
                        after.run.id,
                        &AtomicBool::new(false),
                    )?;
                    assert_eq!(
                        dom.revision.results[0].outcome,
                        if !dom_ok {
                            DomComparisonOutcome::CouldNotCapture {
                                code: DomCaptureError::LimitExceeded,
                            }
                        } else if !stable {
                            DomComparisonOutcome::NotComparable {
                                reason: DomNotComparableReason::Unstable,
                            }
                        } else {
                            DomComparisonOutcome::Changed
                        }
                    );
                }
            }
        }
    }
    Ok(())
}

#[test]
fn same_check_has_independent_reviews_and_screenshot_reanalysis_preserves_dom_review() -> Result<()>
{
    let (_root, mut store, project, inventory, baseline) =
        fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
    let mut after = empty_run(&project, inventory, RunIntent::Check { baseline });
    store.begin_run(&after)?;
    publish(&mut store, &mut after, true, true, true, true)?;
    let cancel = AtomicBool::new(false);
    let image = review::analyze_run(
        &mut store,
        after.run.id,
        review::preset(analysis_engine::Preset::Balanced),
        &cancel,
    )?;
    let dom = dom_snapshot::analyze_run(&mut store, after.run.id, &cancel)?;
    let key = after.run.targets[0].key.clone();
    store.save_review(&ReviewDecision {
        analysis: image.revision.id,
        key: key.clone(),
        state: ReviewState::Intentional,
        note: "screenshot only".into(),
        reviewed_at: 3,
    })?;
    assert!(
        store
            .current_dom_reviews(dom.revision.id, std::slice::from_ref(&key))?
            .is_empty()
    );
    let result = &dom.revision.results[0];
    store.save_dom_review(&DomReviewDecision {
        analysis: dom.revision.id,
        key: key.clone(),
        before: result.before.clone().unwrap(),
        after: result.after.clone().unwrap(),
        state: ReviewState::NeedsFix,
        note: "DOM only".into(),
        reviewed_at: 4,
    })?;
    let revised_image = review::analyze_run(
        &mut store,
        after.run.id,
        review::preset(analysis_engine::Preset::Sensitive),
        &cancel,
    )?;
    assert_ne!(revised_image.revision.id, image.revision.id);
    let dom_again = dom_snapshot::analyze_run(&mut store, after.run.id, &cancel)?;
    assert_eq!(dom_again.revision.id, dom.revision.id);
    let decisions = store.current_dom_reviews(dom.revision.id, std::slice::from_ref(&key))?;
    assert_eq!(decisions.len(), 1);
    assert_eq!(decisions[0].note, "DOM only");
    assert_eq!(decisions[0].state, ReviewState::NeedsFix);
    let original = store.load_baseline(baseline)?;
    let image_preview =
        store.approval_preview(revised_image.revision.id, std::slice::from_ref(&key))?;
    let image_saved =
        store.accept_selected(&image_preview, image_preview.context.project_revision)?;
    let image_baseline = store.load_baseline(image_saved.project.active_baseline.unwrap())?;
    assert_eq!(image_baseline.references[0].run, after.run.id);
    assert_eq!(image_baseline.dom_references, original.dom_references);
    // Fresh DOM consent safely rebases onto the screenshot-only descendant of
    // the baseline checked by this SAME immutable check/DOM analysis.
    let dom_preview = store.dom_approval_preview(dom.revision.id, &[key])?;
    assert_eq!(dom_preview.context.binding.checked_baseline, baseline);
    assert_eq!(
        dom_preview.context.binding.current_parent,
        image_baseline.id
    );
    let both_saved = store.accept_dom_selected(&dom_preview, image_saved.revision)?;
    let both = store.load_baseline(both_saved.project.active_baseline.unwrap())?;
    assert_eq!(both.references, image_baseline.references);
    assert_eq!(both.dom_references[0].run, after.run.id);
    Ok(())
}

#[test]
fn omitted_dom_slot_stays_an_explicit_baseline_omission_and_cannot_be_filled_after_only()
-> Result<()> {
    let (_root, mut store, project, inventory, _) =
        fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
    let mut partial = empty_run(&project, inventory.clone(), RunIntent::NewBaseline);
    store.begin_run(&partial)?;
    publish(&mut store, &mut partial, true, false, true, false)?;
    let revision = store.load_project(project.id)?.revision;
    assert!(store.activate_baseline(partial.run.id, revision).is_err());
    let saved =
        store.activate_baseline_with_partial_confirmation(partial.run.id, revision, true)?;
    let baseline = saved.project.active_baseline.unwrap();
    let mut after = empty_run(&saved.project, inventory, RunIntent::Check { baseline });
    after.run.targets[0].dom_outcome = DomTargetOutcome::NotRequested;
    store.begin_run(&after)?;
    publish(&mut store, &mut after, true, false, true, true)?;
    assert!(store.load_run(after.run.id)?.dom_captures.is_empty());
    let dom = dom_snapshot::analyze_run(&mut store, after.run.id, &AtomicBool::new(false))?;
    assert_eq!(
        dom.revision.results[0].outcome,
        DomComparisonOutcome::NotComparable {
            reason: DomNotComparableReason::BaselineOmission
        }
    );
    assert!(
        store
            .dom_approval_preview(dom.revision.id, &[after.run.targets[0].key.clone()])
            .is_err()
    );
    assert_eq!(
        store.load_project(project.id)?.project.active_baseline,
        Some(baseline)
    );
    Ok(())
}

#[test]
fn both_mode_with_only_saved_dom_slots_does_not_manufacture_an_image_analysis() -> Result<()> {
    let (_root, mut store, project, inventory, _) =
        fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
    let mut partial = empty_run(&project, inventory.clone(), RunIntent::NewBaseline);
    store.begin_run(&partial)?;
    publish(&mut store, &mut partial, false, true, true, false)?;
    let revision = store.load_project(project.id)?.revision;
    let saved =
        store.activate_baseline_with_partial_confirmation(partial.run.id, revision, true)?;
    let baseline = saved.project.active_baseline.unwrap();
    assert!(store.load_baseline(baseline)?.references.is_empty());
    let mut after = empty_run(&saved.project, inventory, RunIntent::Check { baseline });
    after.run.targets[0].outcome = TargetOutcome::NotRequested;
    store.begin_run(&after)?;
    publish(&mut store, &mut after, false, true, true, true)?;
    review::analyze_saved_run(
        &mut store,
        after.run.id,
        review::preset(analysis_engine::Preset::Balanced),
        &AtomicBool::new(false),
    )?;
    let dom = store.displayed_dom_analysis(after.run.id)?.unwrap();
    assert_eq!(
        dom.revision.results[0].outcome,
        DomComparisonOutcome::Changed
    );
    assert!(
        review::analyze_run(
            &mut store,
            after.run.id,
            review::preset(analysis_engine::Preset::Balanced),
            &AtomicBool::new(false)
        )
        .is_err()
    );
    assert!(store.load_run(after.run.id)?.captures.is_empty());
    Ok(())
}

#[test]
fn failed_unselected_sibling_does_not_block_an_eligible_kind_approval() -> Result<()> {
    for (image_ok, dom_ok) in [(true, false), (false, true)] {
        let (_root, mut store, project, inventory, baseline) =
            fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
        let original = store.load_baseline(baseline)?;
        let mut after = empty_run(&project, inventory, RunIntent::Check { baseline });
        store.begin_run(&after)?;
        publish(&mut store, &mut after, image_ok, dom_ok, true, true)?;
        let key = &after.run.targets[0].key;
        let saved = if image_ok {
            let image = review::analyze_run(
                &mut store,
                after.run.id,
                review::preset(analysis_engine::Preset::Balanced),
                &AtomicBool::new(false),
            )?;
            let preview = store.approval_preview(image.revision.id, std::slice::from_ref(key))?;
            store.accept_selected(&preview, preview.context.project_revision)?
        } else {
            let dom = dom_snapshot::analyze_run(&mut store, after.run.id, &AtomicBool::new(false))?;
            let preview = store.dom_approval_preview(dom.revision.id, std::slice::from_ref(key))?;
            store.accept_dom_selected(&preview, preview.context.binding.project_revision)?
        };
        let updated = store.load_baseline(saved.project.active_baseline.unwrap())?;
        if image_ok {
            assert_eq!(updated.references[0].run, after.run.id);
            assert_eq!(updated.dom_references, original.dom_references);
        } else {
            assert_eq!(updated.dom_references[0].run, after.run.id);
            assert_eq!(updated.references, original.references);
        }
        let retained = store.load_run(after.run.id)?;
        assert_eq!(retained.run.state, JobState::CompletedWithErrors);
        assert_eq!(retained.captures.len(), usize::from(image_ok));
        assert_eq!(retained.dom_captures.len(), usize::from(dom_ok));
    }
    Ok(())
}

#[test]
fn unrelated_corrupt_before_kind_does_not_block_frozen_configuration_healthy_analysis_or_approval()
-> Result<()> {
    for corrupt in [CheckKind::Screenshot, CheckKind::DomSnapshot] {
        let (_root, mut store, project, inventory, baseline) =
            fixture(CaptureMode::ScreenshotsAndDomSnapshots)?;
        let original = store.load_baseline(baseline)?;
        let mut after = empty_run(&project, inventory, RunIntent::Check { baseline });
        store.begin_run(&after)?;
        publish(&mut store, &mut after, true, true, true, true)?;
        let artifact = match corrupt {
            CheckKind::Screenshot => &original.references[0].artifact,
            CheckKind::DomSnapshot => &original.dom_references[0].artifact,
        };
        let path = store.artifact_path(artifact)?;
        std::fs::write(path, b"synthetic corruption: must never be compared")?;
        let (resolved, frozen) = store.frozen_baseline_source(baseline)?;
        assert_eq!(resolved.id, baseline);
        assert_eq!(resolved.source_run, Some(frozen.run.id));
        assert_eq!(
            resolved.intended_targets,
            frozen
                .run
                .targets
                .iter()
                .map(|t| t.key.clone())
                .collect::<Vec<_>>()
        );
        assert!(
            review::analyze_saved_run(
                &mut store,
                after.run.id,
                review::preset(analysis_engine::Preset::Balanced),
                &AtomicBool::new(false)
            )
            .is_err()
        );
        let key = &after.run.targets[0].key;
        let saved = match corrupt {
            CheckKind::DomSnapshot => {
                assert!(
                    store
                        .dom_evidence_for_run(project.id, after.run.id)
                        .is_err()
                );
                let evidence = store.screenshot_evidence_for_run(project.id, after.run.id)?;
                let analysis = evidence.comparison.unwrap();
                assert_eq!(
                    analysis.revision.results[0].classification,
                    Classification::Changed
                );
                let preview =
                    store.approval_preview(analysis.revision.id, std::slice::from_ref(key))?;
                store.accept_selected(&preview, preview.context.project_revision)?
            }
            CheckKind::Screenshot => {
                assert!(
                    store
                        .screenshot_evidence_for_run(project.id, after.run.id)
                        .is_err()
                );
                let evidence = store.dom_evidence_for_run(project.id, after.run.id)?;
                let analysis = evidence.dom_comparison.unwrap();
                assert_eq!(
                    analysis.revision.results[0].outcome,
                    DomComparisonOutcome::Changed
                );
                assert!(
                    store
                        .viewer_dom_evidence(project.id, Some(analysis.revision.id))?
                        .dom_comparison
                        .is_some()
                );
                let preview =
                    store.dom_approval_preview(analysis.revision.id, std::slice::from_ref(key))?;
                store.accept_dom_selected(&preview, preview.context.binding.project_revision)?
            }
        };
        let (updated, _) = store.frozen_baseline_source(saved.project.active_baseline.unwrap())?;
        match corrupt {
            CheckKind::DomSnapshot => {
                assert_eq!(updated.references[0].run, after.run.id);
                assert_eq!(updated.dom_references, original.dom_references);
            }
            CheckKind::Screenshot => {
                assert_eq!(updated.dom_references[0].run, after.run.id);
                assert_eq!(updated.references, original.references);
            }
        }
    }
    Ok(())
}

#[test]
fn unstable_dom_receipt_is_retained_but_excluded_from_baseline_eligible_consent() -> Result<()> {
    for mode in [
        CaptureMode::DomSnapshotsOnly,
        CaptureMode::ScreenshotsAndDomSnapshots,
    ] {
        let (_root, mut store, project, inventory, original) = fixture(mode)?;
        let mut candidate = empty_run(&project, inventory, RunIntent::NewBaseline);
        store.begin_run(&candidate)?;
        publish(&mut store, &mut candidate, true, true, false, true)?;
        assert_eq!(candidate.dom_captures.len(), 1);
        let coverage = store.baseline_eligible_coverage(candidate.run.id)?;
        assert_eq!(coverage.dom_captures, 0);
        assert_eq!(coverage.dom_intended, 1);
        assert_eq!(
            coverage.screenshot_captures,
            usize::from(mode.screenshots())
        );
        let revision = store.load_project(project.id)?.revision;
        assert!(store.activate_baseline(candidate.run.id, revision).is_err());
        if mode.screenshots() {
            let saved = store.activate_baseline_with_partial_confirmation(
                candidate.run.id,
                revision,
                true,
            )?;
            let baseline = store.load_baseline(saved.project.active_baseline.unwrap())?;
            assert!(baseline.explicitly_partial);
            assert_eq!(baseline.references.len(), 1);
            assert!(baseline.dom_references.is_empty());
        } else {
            assert_eq!(coverage.captured_targets, 0);
            assert!(
                store
                    .activate_baseline_with_partial_confirmation(candidate.run.id, revision, true)
                    .is_err()
            );
            assert_eq!(
                store.load_project(project.id)?.project.active_baseline,
                Some(original)
            );
        }
        let retained = store.load_dom_run(candidate.run.id)?;
        assert!(!retained.dom_captures[0].metadata.stable);
        assert!(
            dom_snapshot::original_source(
                &store,
                &retained.dom_captures[0].metadata.sha256,
                retained.dom_captures[0].metadata.policy
            )?
            .contains("hydrated after")
        );
    }
    Ok(())
}

#[test]
fn environment_status_reopens_dom_only_and_mixed_partial_baselines_using_saved_page_union()
-> Result<()> {
    use regressee_app::product::Product;
    for mode in [
        CaptureMode::DomSnapshotsOnly,
        CaptureMode::ScreenshotsAndDomSnapshots,
    ] {
        let (root, mut store, project, mut inventory, initial) = fixture(mode)?;
        let old_image = if mode.screenshots() {
            let (_, source) = store.frozen_baseline_source(initial)?;
            source.all_captures().next().cloned()
        } else {
            None
        };
        inventory.id = InventoryId::new();
        inventory.pages = [
            "/saved-dom?b=2&a=1#/route",
            "/saved-image",
            "/saved-omitted",
        ]
        .into_iter()
        .map(|path| InventoryPage {
            id: PageId::new(),
            canonical: project.sources[0].join(path).unwrap(),
            included: true,
            reason: None,
        })
        .collect();
        store.freeze_inventory(&inventory)?;
        let revision = store.load_project(project.id)?.revision;
        let mut candidate = empty_run(&project, inventory.clone(), RunIntent::NewBaseline);
        let template = candidate.run.targets[0].clone();
        for page in &inventory.pages[1..] {
            let mut target = template.clone();
            target.key.page = page.id;
            target.requested = page.canonical.clone();
            candidate.run.targets.push(target);
        }
        candidate.queue = Some(QueueSettings {
            discovery: project.discovery.clone(),
            project_revision: revision,
            resumed_at: vec![],
        });
        store.begin_run(&candidate)?;
        for target in &mut candidate.run.targets[1..] {
            if mode.screenshots() {
                target.outcome = TargetOutcome::Failed {
                    code: CaptureError::Timeout,
                };
            }
            target.dom_outcome = DomTargetOutcome::Failed {
                code: DomCaptureError::Capture {
                    code: CaptureError::Timeout,
                },
            };
        }
        if let Some(mut image) = old_image {
            let target = &mut candidate.run.targets[1];
            image.metadata.key = target.key.clone();
            image.metadata.requested = target.requested.to_string();
            image.metadata.final_url = target.requested.to_string();
            target.final_url = Some(target.requested.clone());
            target.outcome = TargetOutcome::Captured {
                artifact: image.image.original.clone(),
                width: image.image.width,
                height: image.image.height,
            };
            candidate.captures.push(image);
        }
        publish(&mut store, &mut candidate, false, true, true, false)?;
        let saved =
            store.activate_baseline_with_partial_confirmation(candidate.run.id, revision, true)?;
        let baseline_id = saved.project.active_baseline.unwrap();
        let baseline = store.load_baseline(baseline_id)?;
        assert_eq!(
            baseline.target_keys().len(),
            if mode.screenshots() { 2 } else { 1 }
        );
        // Corruption is intentional and confined to this synthetic store. Setup
        // previews must read frozen metadata, not either kind's physical blobs.
        for artifact in baseline
            .references
            .iter()
            .map(|r| &r.artifact)
            .chain(baseline.dom_references.iter().map(|r| &r.artifact))
        {
            std::fs::write(
                store.artifact_path(artifact)?,
                b"synthetic unrelated blob corruption",
            )?;
        }
        assert!(store.load_baseline(baseline_id).is_err());
        let mut draft = saved.project;
        draft.sources = vec!["http://127.0.0.1:9/current-draft".parse()?];
        draft.discovery.skipped_text = vec!["saved".into()];
        draft.capture_plan = if mode.dom_snapshots() && !mode.screenshots() {
            CapturePlan::default()
        } else {
            CapturePlan::for_mode(CaptureMode::DomSnapshotsOnly)
        };
        draft.environments[0].mappings[0].destination = "http://127.0.0.1:10/mapped/".parse()?;
        store.save_replacement_project(&draft, saved.revision)?;
        let product = Product {
            distribution: None,
            official_runtime: false,
            runtime_cache: root.path().join("never-launched-runtime"),
            auth: Default::default(),
        };
        let status = product.environment_status(&store, project.id)?;
        assert!(!status.login_open);
        assert!(!status.probed);
        let mut expected = vec![(
            "http://127.0.0.1:9/saved-dom?b=2&a=1#/route".into(),
            "http://127.0.0.1:10/mapped/saved-dom?b=2&a=1#/route".into(),
        )];
        if mode.screenshots() {
            expected.push((
                "http://127.0.0.1:9/saved-image".into(),
                "http://127.0.0.1:10/mapped/saved-image".into(),
            ));
        }
        assert_eq!(status.mapped, expected);
        assert!(!product.runtime_cache.exists());
    }
    Ok(())
}
