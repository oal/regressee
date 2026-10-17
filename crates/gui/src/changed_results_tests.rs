// SPDX-License-Identifier: MPL-2.0
//! Changes-first queue defaults and reversible, presentation-only filtering.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{TestAppContext, VisualTestContext};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}

#[gpui::test]
fn changed_results_default_and_show_all_preserve_evidence_drafts_and_focus(
    cx: &mut TestAppContext,
) {
    crate::image_controls_tests::bind(cx);
    for legacy in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let (store, project) = unchanged_review_tests::fixture(dir.path(), true, legacy);
        let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
        for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                s.notice = None;
                assert_eq!(s.result_filter, 1);
                assert_eq!(s.result_indices(), vec![1]);
                assert!(s.review_result_changed());
                assert!(s.review_progress_summary().contains("1 not compared"));
                assert!(s.control_enabled(Control::ShowAllResults, cx));
                s.group_profiles = true;
                cx.notify();
            });
            draw(cx);
            let queue = cx.debug_bounds("review-queue").unwrap();
            let notice = cx.debug_bounds("changed-results-notice").unwrap();
            let action = cx.debug_bounds("action-ShowAllResults").unwrap();
            for bounds in [notice, action] {
                assert!(bounds.left() >= queue.left() && bounds.right() <= queue.right());
                assert!(bounds.top() >= queue.top() && bounds.bottom() <= queue.bottom());
            }
            assert!(action.size.height >= px(40.));
            assert!(cx.debug_bounds("page-row-target-0").is_none());
            assert!(cx.debug_bounds("page-row-target-1").is_some());
            assert!(cx.debug_bounds("page-row-target-3").is_none());
            let stage = cx.debug_bounds("review-viewer").unwrap();
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    let selected = s.selected_key.clone();
                    let requests = s.next_request;
                    s.analysis_fields[9].update(cx, |f, cx| f.set_value("Keep this draft", cx));
                    s.mask_dirty = true;
                    w.focus(&s.handle(Control::ShowAllResults, cx));
                    s.activate_control(Control::ShowAllResults, w, cx);
                    assert_eq!(s.result_filter, 0);
                    assert_eq!(s.result_indices().len(), 4);
                    assert_eq!(s.selected_key, selected);
                    assert_eq!(s.next_request, requests);
                    assert!(s.mask_dirty && s.group_profiles);
                    assert_eq!(s.analysis_fields[9].read(cx).content, "Keep this draft");
                    assert!(s.handle(Control::Guided(45), cx).is_focused(w));
                    assert!(!s.control_mounted(Control::ShowAllResults));
                    s.mask_dirty = false;
                })
            });
            draw(cx);
            // GPUI retains old debug selectors; mounted-control assertions above
            // establish removal without treating stale bounds as current paint.
            assert_eq!(cx.debug_bounds("review-viewer").unwrap(), stage);
            shell.update(cx, |s, cx| {
                // Refreshing this run must not undo the user's Show all choice.
                s.queue(Work::Evidence(project), cx);
                finish_jobs(s, cx);
                assert_eq!(s.result_filter, 0);
            });
            owned.join_after_ui();
        }
        assert_eq!(
            frozen,
            serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
        );
    }
}

#[gpui::test]
fn changed_results_empty_state_keeps_failures_accessible_and_filters_independent(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project) = unchanged_review_tests::fixture(dir.path(), false, false);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.notice = None;
            assert_eq!(s.result_filter, 1);
            assert!(s.result_indices().is_empty());
            assert!(s.review_progress_summary().contains("1 not compared"));
        });
        draw(cx);
        assert!(cx.debug_bounds("review-empty-message").is_some());
        assert!(cx.debug_bounds("action-ShowAllResults").is_some());
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::ShowAllResults, w, cx);
                assert_eq!(s.result_indices().len(), 4);
                s.result_filters_open = true;
                s.choose(Choice::MissingResults, cx);
                assert_eq!(s.result_indices(), vec![3]);
                s.choose(Choice::UnchangedResults, cx);
                assert_eq!(s.result_indices().len(), 3);
                s.choose(Choice::ChangedResults, cx);
                s.result_filters_open = false;
                s.search = "/failed/".into();
                s.activate_control(Control::ShowAllResults, w, cx);
                assert_eq!(s.search, "/failed/");
                assert_eq!(s.result_indices(), vec![3]);
                s.search.clear();
                s.review_queue_collapsed = true;
                s.result_filter = 1;
                assert!(!s.control_mounted(Control::ShowAllResults));
                s.review_queue_collapsed = false;
            })
        });
    }
    owned.join_after_ui();
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn changed_results_defaults_do_not_hide_capture_only_evidence(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let project = workflow_refinement_tests::screenshot_queue_fixture(dir.path(), false, 3);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        assert!(s.evidence.comparison.is_none());
        assert_eq!(s.result_filter, 0);
        assert_eq!(s.result_indices().len(), 3);
        assert!(!s.control_mounted(Control::ShowAllResults));
    });
    owned.join_after_ui();
}

#[gpui::test]
fn changed_results_first_analysis_and_recorded_checks_default_without_resetting_refresh(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project) = unchanged_review_tests::fixture(dir.path(), true, false);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        let evidence = s.evidence.clone();
        let analysis = evidence.comparison.clone().unwrap();
        // Simulate the first saved analysis arriving after its captures loaded.
        s.evidence.comparison = None;
        s.result_filter = 0;
        s.next_request += 1;
        let id = s.next_request;
        s.pending = Some(id);
        s.apply_update(
            regressee_gui::jobs::Update {
                id,
                reply: Reply::Analysis(Box::new(analysis.clone())),
            },
            cx,
        );
        finish_jobs(s, cx);
        assert_eq!(s.result_filter, 1);
        assert_eq!(s.result_indices(), vec![1]);
        s.choose(Choice::AllResults, cx);
        s.next_request += 1;
        let id = s.next_request;
        s.pending = Some(id);
        s.apply_update(
            regressee_gui::jobs::Update {
                id,
                reply: Reply::Analysis(Box::new(analysis)),
            },
            cx,
        );
        finish_jobs(s, cx);
        assert_eq!(s.result_filter, 0, "another analysis must keep Show all");
        s.queue(Work::Evidence(project), cx);
        finish_jobs(s, cx);
        assert_eq!(s.result_filter, 0, "refresh must keep Show all");
        s.focus.screen = Screen::History;
        s.queue(Work::History { project, page: 0 }, cx);
        finish_jobs(s, cx);
        let run = evidence.latest.as_ref().unwrap().run.id;
        s.run_history_index = s
            .history
            .as_ref()
            .unwrap()
            .runs
            .iter()
            .position(|r| r.run.id == run)
            .unwrap();
        s.receive_history_inspection(project, run, evidence, cx);
        finish_jobs(s, cx);
        assert_eq!(s.result_filter, 1);
        assert_eq!(s.result_indices(), vec![1]);
        assert_eq!(
            s.selected_key.as_ref(),
            Some(&s.evidence.latest.as_ref().unwrap().run.targets[1].key)
        );
        assert!(s.control_enabled(Control::ShowAllResults, cx));
        assert!(
            !s.control_mounted(Control::Phase5(21)),
            "History remains read-only"
        );
    });
    owned.join_after_ui();
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "bounded changes-first queue inspection with synthetic saved evidence"]
fn renderer_capture_changed_results(cx: &mut TestAppContext) {
    use crate::ui_renderer_capture::{capture, output_root};
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    for changed in [true, false] {
        let dir = tempfile::Builder::new()
            .prefix("changed-results-synthetic-")
            .tempdir_in(&root)
            .unwrap();
        let (store, project) = unchanged_review_tests::fixture(dir.path(), changed, true);
        let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
        for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                s.notice = None;
                s.error = None;
                assert_eq!(s.result_filter, 1);
                cx.notify();
            });
            capture(
                cx,
                &root,
                &format!("changed-results-{changed}-{width:.0}x{height:.0}"),
            )
            .unwrap();
            if changed {
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        s.activate_control(Control::ShowAllResults, w, cx);
                    })
                });
                capture(cx, &root, &format!("all-results-{width:.0}x{height:.0}")).unwrap();
            }
            owned.join_after_ui();
        }
        assert_eq!(
            frozen,
            serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
        );
    }
}
