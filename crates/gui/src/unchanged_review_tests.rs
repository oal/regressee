// SPDX-License-Identifier: MPL-2.0
//! Saved, genuinely identical/changed corpus pairs; no user data or native-input claim.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{TestAppContext, VisualTestContext};
use regressee_core::{domain::*, evidence::RunRecord};

fn save_run(store: &mut regressee_storage::Store, run: &RunRecord) {
    let mut pending = run.clone();
    pending.run.state = JobState::Running;
    pending.run.finished_at = None;
    pending.captures.clear();
    for target in &mut pending.run.targets {
        target.outcome = TargetOutcome::Pending;
        target.final_url = None;
    }
    store.begin_run(&pending).unwrap();
    store.finish_run(run).unwrap();
}

/// Matching warnings on both sides, two matches, an optional real change, and a
/// genuine capture failure. Legacy summaries retain their verified pixel reports.
pub(super) fn fixture(
    path: &std::path::Path,
    changed: bool,
    legacy: bool,
) -> (regressee_storage::Store, ProjectId) {
    let (mut store, project, check, _, _) = approval_fixture::fixture(path);
    let original = store.project_evidence(project).unwrap();
    let baseline = original.baseline.unwrap();
    let mut run = store.load_run(baseline.references[0].run).unwrap();
    let before = run.captures[0].clone();
    let after = store.load_run(check).unwrap().captures[0].clone();
    run.run.id = RunId::new();
    run.run.inventory.id = InventoryId::new();
    let page = run.run.inventory.pages[0].clone();
    run.run.inventory.pages = ["unchanged/", "changed/", "another-match/", "failed/"]
        .into_iter()
        .map(|path| {
            let mut page = page.clone();
            page.id = PageId::new();
            page.canonical = page.canonical.join(path).unwrap();
            page
        })
        .collect();
    let target = run.run.targets[0].clone();
    run.run.targets = run
        .run
        .inventory
        .pages
        .iter()
        .map(|page| {
            let mut target = target.clone();
            target.key.page = page.id;
            target.requested = page.canonical.clone();
            target
        })
        .collect();
    store.freeze_inventory(&run.run.inventory).unwrap();
    for sequence in 0..2 {
        let saved = store.load_project(project).unwrap();
        run.queue = Some(regressee_core::evidence::QueueSettings {
            discovery: saved.project.discovery,
            project_revision: saved.revision,
            resumed_at: vec![],
        });
        run.run.id = RunId::new();
        run.run.started_at = Some(200 + sequence * 2);
        run.run.finished_at = Some(201 + sequence * 2);
        run.run.state = if sequence == 0 {
            JobState::Completed
        } else {
            JobState::CompletedWithErrors
        };
        run.captures.clear();
        for (index, target) in run.run.targets.iter_mut().enumerate() {
            if sequence == 1 && index == 3 {
                target.outcome = TargetOutcome::Failed {
                    code: CaptureError::Timeout,
                };
                target.final_url = None;
                continue;
            }
            let mut capture = if changed && sequence == 1 && index == 1 {
                after.clone()
            } else {
                before.clone()
            };
            capture.metadata.key = target.key.clone();
            capture.metadata.requested = target.requested.to_string();
            capture.metadata.final_url = target.requested.to_string();
            capture.metadata.warnings = comparison_ux_tests::WARNINGS
                .iter()
                .map(|w| (*w).to_owned())
                .collect();
            target.final_url = Some(target.requested.clone());
            target.outcome = TargetOutcome::Captured {
                artifact: capture.image.original.clone(),
                width: capture.image.width,
                height: capture.image.height,
            };
            run.captures.push(capture);
        }
        save_run(&mut store, &run);
        if sequence == 0 {
            let revision = store.load_project(project).unwrap().revision;
            let saved = store.activate_baseline(run.run.id, revision).unwrap();
            run.run.intent = RunIntent::Check {
                baseline: saved.project.active_baseline.unwrap(),
            };
        }
    }
    let mut analysis = regressee_app::review::analyze_run(
        &mut store,
        run.run.id,
        regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    assert_eq!(analysis.revision.results[0].changed_pixels, 0);
    assert_eq!(
        analysis.revision.results[1].classification,
        if changed {
            Classification::Changed
        } else {
            Classification::NoSignificantChange
        }
    );
    if legacy {
        analysis.revision.id = AnalysisId::new();
        analysis.perceptual.as_mut().unwrap().cache_key =
            regressee_core::evidence::identity_hash(&("legacy-warning-veto", analysis.revision.id))
                .unwrap();
        for result in &mut analysis.revision.results {
            if result.eligible_pixels > 0 {
                result.classification = Classification::NotComparable;
            }
        }
        store.save_analysis(&analysis).unwrap();
        store.display_analysis(analysis.revision.id).unwrap();
    }
    (store, project)
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
    cx.run_until_parked();
}

fn load_unchanged(s: &mut Shell, project: ProjectId, cx: &mut Context<Shell>) {
    load_review(s, project, cx);
    // The normal queue prioritizes failures/changes. Inspect a matching page as
    // the user did, rather than assuming its inventory index is the first row.
    s.selected_key = Some(
        s.evidence.latest.as_ref().unwrap().run.targets[0]
            .key
            .clone(),
    );
    s.clear_viewer();
    s.reload_viewer(cx);
    finish_jobs(s, cx);
    s.sync_review_note(cx);
}

fn assert_no_decisions(shell: &Entity<Shell>, cx: &mut VisualTestContext) {
    draw(cx);
    assert!(cx.debug_bounds("review-no-change").is_some());
    for index in 20..=22 {
        assert!(
            cx.debug_bounds(format!("action-Phase5({index})").leak())
                .is_none()
        );
        shell.read_with(cx, |s, cx| {
            let control = Control::Phase5(index);
            assert!(!s.control_mounted(control));
            assert!(!s.control_enabled(control, cx));
            assert!(
                !s.visible_controls()
                    .into_iter()
                    .filter(|candidate| s.control_enabled(*candidate, cx))
                    .any(|candidate| candidate == control)
            );
        });
    }
    cx.update(|w, cx| shell.read_with(cx, |s, cx| w.focus(&s.handle(Control::Viewer, cx))));
    crate::keyboard_navigation_tests::press(cx, "tab");
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert_eq!(
                s.focused(w, cx),
                Some(Control::Phase5(37)),
                "Tab must skip unmounted decisions"
            );
        })
    });
}

#[gpui::test]
fn unchanged_review_is_complete_and_next_only_visits_real_changes(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    for legacy in [false, true] {
        for (width, height) in [(1280., 800.), (820., 600.)] {
            let dir = tempfile::tempdir().unwrap();
            let (store, project) = fixture(dir.path(), true, legacy);
            let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| {
                load_unchanged(s, project, cx);
                s.notice = None;
                s.error = None;
                assert!(s.review_result_unchanged());
                assert_eq!(s.ordinary_review_status(), "No significant change");
                assert_eq!(
                    s.review_progress_summary(),
                    "1 change to review · 1 not compared"
                );
                assert!(s.capture_notice().is_some());
                assert!(s.control_enabled(Control::Phase5(37), cx));
                assert!(s.control_enabled(Control::BaselineUpdate, cx));
                assert!(s.approval_eligible(s.selected_key.as_ref().unwrap()));
                assert!(!s.control_mounted(Control::Phase5(23)));
                assert_eq!(
                    s.next_unreviewed_key(),
                    Some(
                        s.evidence.latest.as_ref().unwrap().run.targets[1]
                            .key
                            .clone()
                    )
                );
            });
            assert_no_decisions(&shell, cx);
            shell.update(cx, |s, _| {
                s.result_filter = 3;
                assert!(
                    s.next_unreviewed_key().is_none(),
                    "Next must respect the no-change filter"
                );
                assert!(!s.control_mounted(Control::Phase5(33)));
                s.result_filter = 0;
                let run = s.evidence.latest.as_ref().unwrap();
                let first = run.run.targets[0].key.clone();
                let changed = run.run.targets[1].key.clone();
                s.selected_key = Some(run.run.targets[2].key.clone());
                assert_eq!(
                    s.next_unreviewed_key(),
                    Some(changed),
                    "wraparound must skip matches and failures"
                );
                s.selected_key = Some(first);
            });
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    let requests = s.next_request;
                    for index in 20..=22 {
                        s.activate_control(Control::Phase5(index), w, cx);
                    }
                    assert_eq!(
                        s.next_request, requests,
                        "hidden choices cannot save decisions"
                    );
                    s.activate_control(Control::Phase5(33), w, cx);
                    finish_jobs(s, cx);
                    assert!(s.review_result_changed());
                    assert!(
                        s.next_unreviewed_key().is_none(),
                        "Next must not wrap through matching pages"
                    );
                    assert!(s.control_enabled(Control::Phase5(21), cx));
                    assert!(!s.control_mounted(Control::Phase5(33)));
                    s.activate_control(Control::Phase5(22), w, cx);
                    finish_jobs(s, cx);
                    assert!(s.review_progress_summary().contains("No changes to review"));
                    assert!(s.review_progress_summary().contains("1 marked Needs fix"));
                    s.result_filter = 3;
                    let key = s.evidence.latest.as_ref().unwrap().run.targets[0]
                        .key
                        .clone();
                    s.selected_key = Some(key);
                    s.reload_viewer(cx);
                    finish_jobs(s, cx);
                    assert!(s.next_unreviewed_key().is_none());
                    assert!(s.review_blocking_reason(cx).is_none());
                })
            });
            assert_no_decisions(&shell, cx);
            owned.join_after_ui();
            let mut after = store.project_evidence(project).unwrap();
            assert!(!after.reviews.is_empty());
            after.reviews.clear();
            assert_eq!(
                frozen,
                serde_json::to_value(after).unwrap(),
                "only explicit changed-page decisions may be saved"
            );
        }
    }
}

#[gpui::test]
fn unchanged_review_keeps_notes_explicit_and_never_approves_a_baseline(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project) = fixture(dir.path(), false, true);
    let frozen = store.project_evidence(project).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            load_unchanged(s, project, cx);
            assert!(s.review_result_unchanged());
            assert_eq!(
                s.review_progress_summary(),
                "No changes to review · 1 not compared"
            );
            assert!(!s.control_mounted(Control::Phase5(33)));
            assert!(s.review_blocking_reason(cx).is_none());
            assert!(s.evidence.reviews.is_empty());
            s.activate_control(Control::Phase5(37), w, cx);
            s.analysis_fields[9].update(cx, |f, cx| {
                f.set_value("Optional note on a matching screenshot", cx)
            });
            s.dismiss(&Dismiss, w, cx);
            assert!(s.review_note_dirty(cx));
            assert!(!s.control_enabled(Control::Phase5(33), cx));
            s.activate_control(Control::Phase5(37), w, cx);
            assert!(s.control_enabled(Control::Phase5(32), cx));
            s.activate_control(Control::Phase5(32), w, cx);
            finish_jobs(s, cx);
            assert!(!s.review_note_dirty(cx));
            s.dismiss(&Dismiss, w, cx);
            assert!(s.review_result_unchanged());
            assert!(s.approval_selection.is_empty());
            assert!(!s.control_mounted(Control::Phase5(24)));
            let saved = s.evidence.reviews.first().unwrap();
            assert_eq!(
                saved.state,
                ReviewState::Unreviewed,
                "saving a note must not invent human approval"
            );
            assert_eq!(saved.note, "Optional note on a matching screenshot");
        })
    });
    owned.join_after_ui();
    let mut after = store.project_evidence(project).unwrap();
    assert_eq!(after.reviews.len(), 1);
    after.reviews.clear();
    assert_eq!(
        serde_json::to_value(frozen).unwrap(),
        serde_json::to_value(after).unwrap()
    );
}

#[gpui::test]
fn unchanged_review_does_not_hide_saved_decisions_or_claim_pending_failed_evidence_passed(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project) = fixture(dir.path(), false, false);
    let evidence = store.project_evidence(project).unwrap();
    let key = evidence.latest.as_ref().unwrap().run.targets[0].key.clone();
    store
        .save_review(&ReviewDecision {
            analysis: evidence.comparison.as_ref().unwrap().revision.id,
            key,
            state: ReviewState::NeedsFix,
            note: "Retain a previously saved observation".into(),
            reviewed_at: 300,
        })
        .unwrap();
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_unchanged(s, project, cx);
        assert!(s.review_result_unchanged());
        assert_eq!(s.ordinary_review_status(), "Reviewed · needs fix");
        assert_eq!(
            s.analysis_fields[9].read(cx).content.as_ref(),
            "Retain a previously saved observation"
        );
        s.analysis_due = Some(std::time::Instant::now() + Duration::from_secs(3600));
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("review-no-change").is_none());
    assert!(cx.debug_bounds("review-unavailable").is_some());
    shell.update(cx, |s, cx| {
        s.analysis_due = None;
        s.viewer_read_failed = true;
        assert!(!s.review_result_unchanged());
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("review-no-change").is_none());
    shell.update(cx, |s, cx| {
        s.viewer_read_failed = false;
        s.selected_key = Some(
            s.evidence.latest.as_ref().unwrap().run.targets[3]
                .key
                .clone(),
        );
        s.reload_viewer(cx);
        finish_jobs(s, cx);
        assert!(!s.review_result_unchanged());
        assert!(!s.review_result_comparable());
        assert!(s.next_unreviewed_key().is_none());
    });
    draw(cx);
    assert!(cx.debug_bounds("review-no-change").is_none());
    owned.join_after_ui();
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
