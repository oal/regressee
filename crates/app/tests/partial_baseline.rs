// SPDX-License-Identifier: MPL-2.0
//! Offline storage and fixed-target regressions; synthetic corpus evidence.
#[path = "support/approval_fixture.rs"]
mod fixture;
use regressee_core::domain::*;
use regressee_storage::{AppPaths, Store};
use std::{collections::BTreeSet, sync::atomic::AtomicBool};

#[test]
fn partial_baseline_requires_consent_retains_failures_and_checks_exact_saved_pairs() {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project, run) = fixture::partial_fixture(dir.path());
    let original = store.load_run(run).unwrap();
    let saved = store.load_project(project).unwrap();
    let old = saved.project.active_baseline.unwrap();
    assert_eq!(original.baseline_save_counts(), Some((3, 4)));
    assert!(store.activate_baseline(run, saved.revision).is_err());
    assert_eq!(
        store.load_project(project).unwrap().project.active_baseline,
        Some(old)
    );
    assert!(
        store
            .activate_baseline_with_partial_confirmation(run, saved.revision + 1, true)
            .is_err()
    );
    let activated = store
        .activate_baseline_with_partial_confirmation(run, saved.revision, true)
        .unwrap();
    let baseline = store
        .load_baseline(activated.project.active_baseline.unwrap())
        .unwrap();
    assert!(baseline.explicitly_partial);
    assert_eq!((baseline.references.len(), baseline.intended_count), (3, 4));
    assert_eq!(baseline.parent, Some(old));
    assert_eq!(
        serde_json::to_value(store.load_run(run).unwrap()).unwrap(),
        serde_json::to_value(&original).unwrap()
    );
    assert!(store.load_baseline(old).is_ok());

    let mut check = original.clone();
    check.run.id = RunId::new();
    check.run.intent = RunIntent::Check {
        baseline: baseline.id,
    };
    check.run.state = JobState::Running;
    check.run.finished_at = None;
    check.captures.clear();
    for target in &mut check.run.targets {
        target.outcome = TargetOutcome::Pending;
        target.final_url = None;
        target.attempts = 0;
    }
    // Including the failed pair, or omitting any saved pair, is never allowed.
    assert!(store.begin_run(&check).is_err());
    check.run.targets.pop();
    check.run.validate().unwrap();
    let mut omitted = check.clone();
    omitted.run.id = RunId::new();
    omitted.run.targets.pop();
    assert!(store.begin_run(&omitted).is_err());
    store.begin_run(&check).unwrap();
    for target in &mut check.run.targets {
        let source = original
            .run
            .targets
            .iter()
            .find(|t| t.key == target.key)
            .unwrap();
        *target = source.clone();
    }
    check.captures = original.captures.clone();
    check.run.state = JobState::Completed;
    check.run.finished_at = Some(103);
    store.finish_run(&check).unwrap();
    regressee_app::review::analyze_run(
        &mut store,
        check.run.id,
        regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
        &AtomicBool::new(false),
    )
    .unwrap();
    let evidence = store.project_evidence(project).unwrap();
    let comparison = evidence.comparison.unwrap();
    let results = &comparison.revision.results;
    assert_eq!(results.len(), 3);
    assert_eq!(
        results.iter().map(|r| &r.key).collect::<BTreeSet<_>>(),
        baseline.references.iter().map(|r| &r.key).collect()
    );
    assert!(
        results
            .iter()
            .all(|r| r.classification == Classification::NoSignificantChange),
        "{comparison:#?}"
    );

    let export = store.export_preview(project, baseline.id).unwrap();
    let archive = dir.path().join("partial.zip");
    store
        .export_archive(&export, &archive, &AtomicBool::new(false))
        .unwrap();
    let imported_dir = tempfile::tempdir().unwrap();
    let mut recipient = Store::open(AppPaths::isolated(imported_dir.path())).unwrap();
    let preview = recipient
        .inspect_archive(&archive, &AtomicBool::new(false))
        .unwrap()
        .preview;
    let imported = recipient
        .import_archive(&archive, &preview, &AtomicBool::new(false))
        .unwrap();
    let imported_baseline = recipient
        .load_baseline(imported.project.active_baseline.unwrap())
        .unwrap();
    assert!(imported_baseline.explicitly_partial);
    assert_eq!(
        (
            imported_baseline.references.len(),
            imported_baseline.intended_count
        ),
        (3, 4)
    );
}

#[test]
fn partial_consent_never_allows_unfinished_preview_check_empty_or_corrupt_evidence() {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project, run) = fixture::partial_fixture(dir.path());
    let original = store.load_run(run).unwrap();
    for state in [
        JobState::Running,
        JobState::Paused,
        JobState::Interrupted,
        JobState::Cancelled,
    ] {
        let mut record = original.clone();
        record.run.state = state;
        assert!(record.baseline_save_counts().is_none());
    }
    for intent in [
        RunIntent::RepresentativePreview,
        RunIntent::Check {
            baseline: BaselineId::new(),
        },
    ] {
        let mut record = original.clone();
        record.run.intent = intent;
        assert!(record.baseline_save_counts().is_none());
    }
    let mut empty = original.clone();
    empty.captures.clear();
    assert!(empty.baseline_save_counts().is_none());
    let revision = store.load_project(project).unwrap().revision;
    let mut no_success = original.clone();
    no_success.run.id = RunId::new();
    no_success.run.state = JobState::Running;
    no_success.run.finished_at = None;
    no_success.captures.clear();
    for target in &mut no_success.run.targets {
        target.outcome = TargetOutcome::Pending;
        target.final_url = None;
    }
    store.begin_run(&no_success).unwrap();
    for target in &mut no_success.run.targets {
        target.outcome = TargetOutcome::Failed {
            code: CaptureError::Timeout,
        };
    }
    no_success.run.state = JobState::CompletedWithErrors;
    no_success.run.finished_at = Some(104);
    store.finish_run(&no_success).unwrap();
    assert!(
        store
            .activate_baseline_with_partial_confirmation(no_success.run.id, revision, true)
            .is_err()
    );
    let artifact = store
        .artifact_path(&original.captures[0].image.original)
        .unwrap();
    std::fs::write(artifact, b"damaged screenshot").unwrap();
    assert!(
        store
            .activate_baseline_with_partial_confirmation(run, revision, true)
            .is_err()
    );
    assert_eq!(store.load_project(project).unwrap().revision, revision);
}
