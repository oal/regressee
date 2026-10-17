// SPDX-License-Identifier: MPL-2.0
//! Search presentation must not change filtering, evidence or keyboard ownership.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review, screenshot_queue_fixture};
use gpui::{TestAppContext, point};

#[gpui::test]
fn queue_and_history_search_keep_filtering_clear_and_focus(cx: &mut TestAppContext) {
    cx.update(crate::keyboard_navigation_tests::bind);
    let dir = tempfile::tempdir().unwrap();
    let project = screenshot_queue_fixture(dir.path(), true, 8);
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.choose(Choice::AllResults, cx);
        s.navigation_preference = Some(false);
        s.configure_review_geometry(1280.);
        s.analysis_fields[9].update(cx, |field, cx| field.set_value("Keep this note draft", cx));
    });
    cx.simulate_resize(size(px(1280.), px(800.)));
    for screen in [Screen::Results, Screen::History] {
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.focus.screen = screen;
                if screen == Screen::History {
                    s.queue(Work::History { project, page: 0 }, cx);
                    finish_jobs(s, cx);
                }
                let input = if screen == Screen::History {
                    &s.history_search
                } else {
                    &s.crawl_fields[9]
                };
                assert!(input.read(cx).search_placeholder.is_some());
                w.focus(&input.read(cx).focus);
                cx.notify();
            })
        });
        cx.run_until_parked();
        let key = shell.read_with(cx, |s, _| s.selected_key.clone());
        cx.simulate_input("no matching page or run");
        cx.run_until_parked();
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                assert_eq!(s.selected_key, key);
                assert_eq!(
                    s.analysis_fields[9].read(cx).content.as_ref(),
                    "Keep this note draft"
                );
                let clear = if screen == Screen::History {
                    assert_eq!(s.history_query, "no matching page or run");
                    assert!(s.history_run_indices().is_empty());
                    Control::HistoryClear
                } else {
                    assert_eq!(s.search, "no matching page or run");
                    assert!(s.result_indices().is_empty());
                    Control::Guided(17)
                };
                assert!(s.control_enabled(clear, cx));
                s.activate_control(clear, w, cx);
                assert!(s.search.is_empty());
                assert!(s.history_query.is_empty());
                assert_eq!(s.selected_key, key);
            })
        });
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    owned.join_after_ui();
}

#[gpui::test]
fn search_rows_fit_narrow_queues_and_history_with_clear_actions(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let project = screenshot_queue_fixture(dir.path(), true, 8);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for screen in [Screen::Results, Screen::History] {
            shell.update(cx, |s, cx| {
                s.focus.screen = screen;
                s.navigation_preference = Some(false);
                s.configure_review_geometry(width);
                if screen == Screen::History {
                    s.queue(Work::History { project, page: 0 }, cx);
                    finish_jobs(s, cx);
                }
                let input = if screen == Screen::History {
                    &s.history_search
                } else {
                    &s.crawl_fields[9]
                };
                input.update(cx, |field, cx| field.set_value("no matches", cx));
                cx.notify();
            });
            cx.run_until_parked();
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            let row = cx.debug_bounds("search-field-row").unwrap();
            let clear = cx
                .debug_bounds(if screen == Screen::History {
                    "action-HistoryClear"
                } else {
                    "action-Guided(17)"
                })
                .unwrap();
            let icon = cx.debug_bounds("search-field-icon").unwrap();
            assert!(row.left() >= px(0.) && row.right() <= px(width));
            assert_eq!(row.size.height, px(CONTROL_HEIGHT));
            assert!(icon.left() >= row.left() && icon.right() < clear.left());
            assert!(clear.right() <= row.right() && clear.size.height >= px(40.));
            if screen == Screen::Results {
                let queue = cx.debug_bounds("review-queue").unwrap();
                let workspace = cx.debug_bounds("queue-beside-viewer").unwrap();
                let filter = cx.debug_bounds("action-Guided(45)").unwrap();
                for bounds in [row, clear, filter] {
                    crate::phase2_evidence_tests::inside(
                        Bounds::from_corners(
                            bounds.origin
                                - point(px(controls::FOCUS_OUTSET), px(controls::FOCUS_OUTSET)),
                            bounds.bottom_right()
                                + point(px(controls::FOCUS_OUTSET), px(controls::FOCUS_OUTSET)),
                        ),
                        workspace,
                    );
                }
                assert_eq!(row.left(), queue.left());
                assert_eq!(row.right(), queue.right());
                assert_eq!(filter.left(), queue.left());
                crate::phase2_evidence_tests::inside(row, queue);
                crate::phase2_evidence_tests::inside(clear, queue);
                crate::phase2_evidence_tests::inside(
                    cx.debug_bounds("queue-toolbar").unwrap(),
                    queue,
                );
            }
        }
    }
    owned.join_after_ui();
}
