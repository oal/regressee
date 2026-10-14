// SPDX-License-Identifier: MPL-2.0
//! Phase 4 hierarchy, mounted traversal and ownership, not OS qualification.
use super::*;
use crate::phase2_evidence_tests::inside;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{TestAppContext, VisualTestContext};
use regressee_core::domain::{ProjectId, RunId};

pub(super) fn draw(cx: &mut VisualTestContext) {
    for _ in 0..3 {
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
    }
}

/// Traverse the real enabled/mounted focus order; never set an offscreen offset
/// or focus an unmounted field to manufacture reachability.
pub(super) fn tab_to(shell: &Entity<Shell>, cx: &mut VisualTestContext, control: Control) {
    draw(cx);
    for _ in 0..260 {
        if cx.update(|w, cx| {
            shell.read_with(cx, |s, cx| {
                s.control_enabled(control, cx) && s.handle(control, cx).is_focused(w)
            })
        }) {
            draw(cx);
            return;
        }
        cx.update(|w, cx| shell.update(cx, |s, cx| s.move_focus(false, w, cx)));
        draw(cx);
    }
    panic!("Tab did not reach {control:?}");
}

pub(super) fn activate(shell: &Entity<Shell>, cx: &mut VisualTestContext, control: Control) {
    tab_to(shell, cx, control);
    cx.update(|w, cx| shell.update(cx, |s, cx| s.activate(&Activate, w, cx)));
    draw(cx);
}

/// Real stored records, including a second 64-run history page. Baseline
/// activation here is isolated fixture construction, never app navigation.
pub(super) fn extend_history(
    store: &mut regressee_storage::Store,
    project: ProjectId,
    template: RunId,
    count: usize,
) {
    let original = store.load_run(template).unwrap();
    for n in 0..count {
        let mut run = original.clone();
        run.run.id = RunId::new();
        run.run.intent = RunIntent::NewBaseline;
        run.run.started_at = Some(1_780_000_000 + n as i64 * 3600);
        run.run.finished_at = None;
        run.run.state = JobState::Running;
        let captures = std::mem::take(&mut run.captures);
        let capture = run.capture.take();
        for target in &mut run.run.targets {
            target.outcome = TargetOutcome::Pending;
        }
        store.begin_run(&run).unwrap();
        run.captures = captures;
        run.capture = capture;
        run.run.targets = original.run.targets.clone();
        run.run.state = JobState::Completed;
        run.run.finished_at = run.run.started_at.map(|t| t + 180);
        store.finish_run(&run).unwrap();
        if n < 6 {
            store
                .activate_baseline(run.run.id, store.load_project(project).unwrap().revision)
                .unwrap();
        }
    }
    // Restore the incumbent reference after constructing retained versions.
    let reference = store
        .project_evidence(project)
        .unwrap()
        .baselines
        .iter()
        .find(|b| b.references.iter().any(|r| r.run == template))
        .map(|b| b.id);
    if let Some(reference) = reference {
        store
            .restore_baseline(reference, store.load_project(project).unwrap().revision)
            .unwrap();
    }
}

#[gpui::test]
fn analysis_groups_hide_mutations_preserve_drafts_and_reveal_every_field_and_tail(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let originals = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.review_tools = false;
                s.mask_dirty = false;
                s.mask_selected = None;
                s.details.remove(&Detail::IgnoredAreas);
                s.analysis_advanced = false;
                s.error = None;
                s.notice = None;
                s.activate_control(Control::Guided(44), w, cx);
                for hidden in [
                    Control::Phase5(14),
                    Control::Phase5(34),
                    Control::Phase5(27),
                    Control::AnalysisField(0),
                ] {
                    assert!(!s.control_mounted(hidden));
                    let request = s.next_request;
                    s.activate_control(hidden, w, cx);
                    assert_eq!(request, s.next_request);
                }
                assert!(!s.control_mounted(Control::Phase5(5)));
                let request = s.next_request;
                s.analysis_fields[0].update(cx, |f, cx| f.set_value("11", cx));
                s.poll_analysis(cx);
                assert_eq!(request, s.next_request);
                assert!(s.analysis_due.is_none());
                s.set_analysis_fields(cx);
            })
        });
        draw(cx);
        let summary = cx.debug_bounds("analysis-current-summary").unwrap();
        let sensitivity = cx.debug_bounds("analysis-sensitivity-heading").unwrap();
        let balanced = cx.debug_bounds("action-Phase5(1)").unwrap();
        let ignored = cx.debug_bounds("action-Details(IgnoredAreas)").unwrap();
        assert!(summary.bottom() <= sensitivity.top());
        assert!(balanced.bottom() < ignored.top());
        shell.read_with(cx, |s, _| {
            inside(balanced, s.review_details_scroll.bounds())
        });
        activate(&shell, cx, Control::Phase5(3));
        for i in 0..9 {
            tab_to(&shell, cx, Control::AnalysisField(i));
            let field = cx
                .debug_bounds(format!("analysis-parameter-field-{i}").leak())
                .unwrap();
            shell.read_with(cx, |s, cx| {
                inside(field, s.review_details_scroll.bounds());
                inside(
                    s.analysis_fields[i].read(cx).rendered_bounds().unwrap(),
                    field,
                );
            });
        }
        for c in [Control::Phase5(27), Control::Phase5(28)] {
            tab_to(&shell, cx, c);
            let field = cx.debug_bounds(format!("check-{c:?}").leak()).unwrap();
            shell.read_with(cx, |s, _| inside(field, s.review_details_scroll.bounds()));
        }
        tab_to(&shell, cx, Control::AnalysisField(0));
        shell.update(cx, |s, cx| {
            s.analysis_fields[0].update(cx, |f, cx| f.set_value("999", cx));
            s.poll_analysis(cx);
            assert!(s.analysis_due.is_none());
        });
        let requests = shell.read_with(cx, |s, _| s.next_request);
        activate(&shell, cx, Control::Phase5(4));
        shell.read_with(cx, |s, _| assert_eq!(s.next_request, requests));
        tab_to(&shell, cx, Control::AnalysisField(0));
        for selector in ["analysis-parameter-field-0", "field-correction"] {
            let b = cx.debug_bounds(selector).unwrap();
            shell.read_with(cx, |s, cx| {
                assert!(s.analysis_fields[0].read(cx).touched);
                assert!(s.analysis_fields[0].read(cx).validation_error().is_some());
                inside(b, s.review_details_scroll.bounds());
            });
        }
        shell.update(cx, |s, cx| s.set_analysis_fields(cx));
        draw(cx);
        activate(&shell, cx, Control::Details(Detail::IgnoredAreas));
        tab_to(&shell, cx, Control::Phase5(34));
        for selector in ["action-Phase5(14)", "action-Phase5(34)"] {
            let bounds = cx.debug_bounds(selector).unwrap();
            shell.read_with(cx, |s, _| inside(bounds, s.review_details_scroll.bounds()));
        }
        activate(&shell, cx, Control::Phase5(34));
        for i in 10..14 {
            tab_to(&shell, cx, Control::AnalysisField(i));
            let field = cx
                .debug_bounds(format!("ignored-area-field-{}", i - 10).leak())
                .unwrap();
            shell.read_with(cx, |s, _| inside(field, s.review_details_scroll.bounds()));
        }
        let pair = shell.read_with(cx, |s, _| s.viewer.as_ref().unwrap().request.clone());
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.analysis_fields[10].update(cx, |f, cx| f.set_value("7", cx));
                assert!(s.mask_geometry_dirty(cx));
                let count = s.mask_rectangles.len();
                assert_eq!(
                    s.ignored_area_status(cx),
                    format!(
                        "{count} area{} · unapplied coordinates · Apply, then Save",
                        if count == 1 { "" } else { "s" }
                    )
                );
                s.activate_control(Control::Phase5(18), w, cx);
                assert!(s.error.as_ref().unwrap().contains("Apply"));
                let rectangles = s.mask_rectangles.clone();
                s.error = None;
                s.activate_control(Control::Details(Detail::IgnoredAreas), w, cx);
                assert!(!s.control_mounted(Control::Phase5(35)));
                s.activate_control(Control::Phase5(35), w, cx);
                assert_eq!(s.mask_rectangles, rectangles);
                s.activate_control(Control::Phase5(39), w, cx);
                s.activate_control(Control::Guided(44), w, cx);
                s.activate_control(Control::Details(Detail::IgnoredAreas), w, cx);
                assert_eq!(s.analysis_fields[10].read(cx).content.as_ref(), "7");
                assert_eq!(s.viewer.as_ref().unwrap().request, pair);
                let dimensions = s
                    .viewer
                    .as_ref()
                    .unwrap()
                    .request
                    .as_ref()
                    .unwrap()
                    .before
                    .dimensions;
                for (i, value) in [
                    "0".into(),
                    "0".into(),
                    dimensions.width.to_string(),
                    dimensions.height.to_string(),
                ]
                .iter()
                .enumerate()
                {
                    s.analysis_fields[10 + i].update(cx, |f, cx| f.set_value(value, cx));
                }
                s.activate_control(Control::Phase5(35), w, cx);
                let requests = s.next_request;
                s.activate_control(Control::Phase5(18), w, cx);
                assert!(s.error.as_ref().unwrap().contains("all screenshot pixels"));
                assert_eq!(s.next_request, requests);
                s.error = None;
                s.dismiss(&Dismiss, w, cx);
                assert!(s.handle(Control::Guided(44), cx).is_focused(w));
            })
        });
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        originals,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn presets_reanalysis_cancel_previous_revision_and_hidden_polling_are_truthful(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let baseline = store.load_project(project).unwrap().project.active_baseline;
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.activate_control(Control::Guided(44), w, cx);
            s.activate_control(Control::Phase5(0), w, cx);
            assert_eq!(analysis_ux::policy_name(&s.analysis_policy), "Sensitive");
            assert!(s.analysis_due.is_some() && s.control_mounted(Control::Phase5(5)));
            s.activate_control(Control::Phase5(5), w, cx);
            assert!(s.analysis_due.is_none() && !s.control_mounted(Control::Phase5(5)));
            s.activate_control(Control::Phase5(4), w, cx);
            s.analysis_due = Some(std::time::Instant::now());
            s.poll_analysis(cx);
            assert!(s.analysis_pending.is_some());
            // Wait for the real isolated worker; then use the identity-guarded reply.
            let update = s.events.recv_timeout(Duration::from_secs(10)).unwrap();
            s.apply_update(update, cx);
            finish_jobs(s, cx);
            assert!(s.analysis_pending.is_none());
            assert_eq!(
                analysis_ux::policy_name(
                    &s.evidence
                        .comparison
                        .as_ref()
                        .unwrap()
                        .perceptual
                        .as_ref()
                        .unwrap()
                        .policy
                ),
                "Sensitive"
            );
            assert!(s.control_mounted(Control::Phase5(26)));
            let latest = s.evidence.comparison.as_ref().unwrap().revision.id;
            s.activate_control(Control::Phase5(26), w, cx);
            finish_jobs(s, cx);
            assert_ne!(s.evidence.comparison.as_ref().unwrap().revision.id, latest);
            assert!(s.evidence.analyses.contains(&latest));
            s.activate_control(Control::Phase5(3), w, cx);
            s.activate_control(Control::Phase5(39), w, cx);
            let policy = s.analysis_policy.clone();
            s.analysis_fields[0].update(cx, |f, cx| f.set_value("12", cx));
            s.poll_analysis(cx);
            assert_eq!(policy, s.analysis_policy);
            assert!(s.analysis_due.is_none());
        })
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        baseline,
        store.load_project(project).unwrap().project.active_baseline
    );
}

#[gpui::test]
fn history_real_counts_last_row_actions_page_filter_and_provenance_keep_scope(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let baseline = store
        .load_project(project)
        .unwrap()
        .project
        .active_baseline
        .unwrap();
    let run = store
        .project_evidence(project)
        .unwrap()
        .baseline
        .unwrap()
        .references[0]
        .run;
    extend_history(&mut store, project, run, 68);
    let preview = store.export_preview(project, baseline).unwrap();
    assert!(preview.targets > 0);
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::History;
        s.notice = None;
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|_, cx| {
            shell.update(cx, |s, cx| {
                s.history_section = 0;
                s.queue(Work::History { project, page: 0 }, cx);
                finish_jobs(s, cx);
            })
        });
        draw(cx);
        assert_eq!(
            cx.debug_bounds("history-run-list").unwrap().size.height,
            px(256.)
        );
        let last = shell.read_with(cx, |s, _| s.history.as_ref().unwrap().runs.len() - 1);
        activate(&shell, cx, Control::HistoryRun(last));
        let row = cx
            .debug_bounds(format!("history-row-HistoryRun({last})").leak())
            .unwrap();
        shell.read_with(cx, |s, _| {
            inside(row, s.history_run_scroll.bounds());
            inside(row, s.screen_scroll.bounds());
        });
        tab_to(&shell, cx, Control::Phase6(19));
        let action = cx.debug_bounds("action-Phase6(19)").unwrap();
        let selected = cx.debug_bounds("history-selected-run").unwrap();
        inside(action, selected);
        shell.read_with(cx, |s, _| inside(action, s.screen_scroll.bounds()));
        activate(&shell, cx, Control::Phase6(11));
        shell.update(cx, finish_jobs);
        draw(cx);
        shell.read_with(cx, |s, _| {
            assert_eq!(s.history.as_ref().unwrap().page, 1);
            assert!(!s.history.as_ref().unwrap().more);
        });
        shell.update(cx, |s, cx| {
            s.history_search
                .update(cx, |f, cx| f.set_value("nothing-matches", cx));
        });
        draw(cx);
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                assert!(s.history_run_indices().is_empty());
                assert!(!s.control_mounted(Control::Phase6(19)));
                let request = s.next_request;
                s.activate_control(Control::Phase6(19), w, cx);
                assert_eq!(request, s.next_request);
                s.activate_control(Control::HistoryClear, w, cx);
                if !s.details.contains(&Detail::History) {
                    s.activate_control(Control::Details(Detail::History), w, cx);
                }
                assert!(s.details.contains(&Detail::History));
                assert!(!s.details.contains(&Detail::Portability));
            })
        });
    }
    owned.join_after_ui();
}

#[test]
fn count_adaptive_history_and_classification_language_are_explicit() {
    for (count, height) in [
        (0, 0.),
        (1, 64.),
        (2, 128.),
        (3, 192.),
        (4, 256.),
        (64, 256.),
        (128, 256.),
    ] {
        assert_eq!(workspace_history::history_list_height(count), height);
    }
    assert_eq!(Screen::Settings.label(), "App settings");
}
