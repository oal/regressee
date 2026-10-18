// SPDX-License-Identifier: MPL-2.0
//! Result navigation must reveal the selected row without moving focus or evidence.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review, screenshot_queue_fixture};
use gpui::{TestAppContext, VisualTestContext};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}

fn selected_row_visible(shell: &Entity<Shell>, cx: &mut VisualTestContext) {
    let index = shell.read_with(cx, |s, _| {
        s.evidence
            .latest
            .as_ref()
            .unwrap()
            .run
            .targets
            .iter()
            .position(|t| Some(&t.key) == s.selected_key.as_ref())
            .unwrap()
    });
    let row = cx
        .debug_bounds(format!("page-row-target-{index}").leak())
        .unwrap_or_else(|| panic!("selected target {index} was not rendered in the list"));
    let list = shell.read_with(cx, |s, _| {
        s.preview_list_scroll.0.borrow().base_handle.bounds()
    });
    // Very short recovery layouts can have less than one row of space. Reveal
    // its beginning there; ordinary viewports must contain the complete row.
    let visible_height = row.size.height.min(list.size.height);
    assert!(
        row.top() >= list.top() && row.top() + visible_height <= list.bottom(),
        "selected target {index} must be fully visible: {row:?} within {list:?}"
    );
}

#[gpui::test]
fn result_arrows_reveal_each_selection_in_both_directions_and_filtered_order(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let project = screenshot_queue_fixture(dir.path(), true, 25);
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for filtered in [false, true] {
            shell.update(cx, |s, cx| {
                finish_jobs(s, cx);
                // This traversal explicitly includes unchanged and failed rows.
                s.choose(Choice::AllResults, cx);
                s.search = if filtered {
                    "/queue/".into()
                } else {
                    String::new()
                };
                s.group_profiles = filtered;
                s.navigation_preference = None;
                s.configure_review_geometry(width);
                let first = s.result_indices()[0];
                s.selected_key = Some(
                    s.evidence.latest.as_ref().unwrap().run.targets[first]
                        .key
                        .clone(),
                );
                s.clear_viewer();
                s.reload_viewer(cx);
                finish_jobs(s, cx);
                s.preview_list_scroll
                    .scroll_to_item_strict(0, gpui::ScrollStrategy::Top);
                cx.notify();
            });
            draw(cx);
            selected_row_visible(&shell, cx);
            let count = shell.read_with(cx, |s, _| s.result_indices().len());
            for (control, positions) in [
                (Control::Guided(24), (1..count).collect::<Vec<_>>()),
                (Control::Guided(23), (0..count - 1).rev().collect()),
            ] {
                for position in positions {
                    cx.update(|w, cx| {
                        shell.update(cx, |s, cx| {
                            w.focus(&s.handle(control, cx));
                            s.activate_control(control, w, cx);
                            let target = s.result_indices()[position];
                            assert_eq!(
                                s.selected_key.as_ref(),
                                Some(&s.evidence.latest.as_ref().unwrap().run.targets[target].key)
                            );
                            assert!(
                                s.handle(control, cx).is_focused(w),
                                "revealing selection must not steal arrow focus"
                            );
                        })
                    });
                    draw(cx);
                    selected_row_visible(&shell, cx);
                    shell.update(cx, finish_jobs);
                    draw(cx);
                    selected_row_visible(&shell, cx);
                }
            }
            shell.update(cx, |s, cx| {
                let selected = s.selected_key.clone();
                let offset = s.preview_list_scroll.0.borrow().base_handle.offset();
                s.mask_dirty = true;
                s.guided(24, cx);
                assert_eq!(s.selected_key, selected);
                assert_eq!(
                    s.preview_list_scroll.0.borrow().base_handle.offset(),
                    offset
                );
                assert!(
                    s.preview_list_scroll
                        .0
                        .borrow()
                        .deferred_scroll_to_item
                        .is_none()
                );
                s.mask_dirty = false;
                s.error = None;
            });
        }
    }
    owned.join_after_ui();
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn next_unreviewed_reveals_distant_results_and_wraps_to_the_first(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let project = screenshot_queue_fixture(dir.path(), true, 25);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.choose(Choice::AllResults, cx);
        let run = s.evidence.latest.as_ref().unwrap();
        let keys = [
            run.run.targets[1].key.clone(),
            run.run.targets[24].key.clone(),
        ];
        let a = s.evidence.comparison.as_mut().unwrap();
        for key in &keys {
            // Synthetic review policy only; no evidence or review is written.
            a.revision
                .results
                .iter_mut()
                .find(|r| &r.key == key)
                .unwrap()
                .classification = Classification::Changed;
            a.perceptual
                .as_mut()
                .unwrap()
                .reports
                .iter_mut()
                .find(|(k, _)| k == key)
                .unwrap()
                .1
                .classification = regressee_app::analysis_engine::Classification::Changed;
        }
        s.selected_key = Some(keys[0].clone());
        s.clear_viewer();
        s.reload_viewer(cx);
        finish_jobs(s, cx);
        cx.notify();
    });
    draw(cx);
    for index in [24, 1] {
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                assert!(s.control_enabled(Control::Phase5(33), cx));
                s.activate_control(Control::Phase5(33), w, cx);
                assert_eq!(
                    s.selected_key.as_ref(),
                    Some(&s.evidence.latest.as_ref().unwrap().run.targets[index].key)
                );
            })
        });
        draw(cx);
        selected_row_visible(&shell, cx);
        shell.read_with(cx, |s, cx| {
            assert!(s.control_mounted(Control::Phase5(33)));
            assert!(!s.control_enabled(Control::Phase5(33), cx));
        });
        shell.update(cx, finish_jobs);
        draw(cx);
        selected_row_visible(&shell, cx);
    }
    owned.join_after_ui();
}

#[gpui::test]
fn rapid_result_navigation_tracks_the_latest_selection_before_images_arrive(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let project = screenshot_queue_fixture(dir.path(), true, 25);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.choose(Choice::AllResults, cx);
    });
    cx.simulate_resize(size(px(1280.), px(800.)));
    draw(cx);
    for control in std::iter::repeat_n(Control::Guided(24), 16)
        .chain(std::iter::repeat_n(Control::Guided(23), 8))
    {
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(control, w, cx);
                assert!(s.viewer.is_none() && s.viewer_pending.is_some());
                assert_eq!(
                    s.viewer_pending.as_ref().unwrap().1.key,
                    *s.selected_key.as_ref().unwrap()
                );
            })
        });
        draw(cx);
        selected_row_visible(&shell, cx);
    }
    shell.update(cx, |s, cx| {
        finish_jobs(s, cx);
        assert!(s.viewer_current());
        assert_eq!(
            s.viewer.as_ref().unwrap().identity.as_ref().unwrap().key,
            *s.selected_key.as_ref().unwrap()
        );
    });
    draw(cx);
    selected_row_visible(&shell, cx);
    owned.join_after_ui();
}

#[gpui::test]
fn result_reveal_preserves_visible_rows_and_manual_scroll(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let project = screenshot_queue_fixture(dir.path(), false, 25);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.simulate_resize(size(px(1440.), px(900.)));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    draw(cx);
    let before = shell.read_with(cx, |s, _| {
        s.preview_list_scroll.0.borrow().base_handle.offset()
    });
    shell.update(cx, |s, cx| s.guided(24, cx));
    draw(cx);
    selected_row_visible(&shell, cx);
    shell.update(cx, |s, cx| {
        assert_eq!(
            s.preview_list_scroll.0.borrow().base_handle.offset(),
            before,
            "an already visible row must not be recentered"
        );
        s.preview_list_scroll
            .0
            .borrow()
            .base_handle
            .set_offset(gpui::point(px(0.), px(-420.)));
        cx.notify();
    });
    draw(cx);
    shell.update(cx, finish_jobs);
    draw(cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(
            s.preview_list_scroll.0.borrow().base_handle.offset().y,
            px(-420.),
            "a completed read must not override subsequent manual scrolling"
        );
    });
    owned.join_after_ui();
}

fn toolbar_loading_matrix(cx: &mut TestAppContext, render: bool) {
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    let root = render.then(|| {
        cx.enable_offscreen_renderer().unwrap();
        crate::ui_renderer_capture::output_root().unwrap()
    });
    #[cfg(not(all(feature = "ui-renderer-capture", target_os = "linux")))]
    assert!(!render);
    let dir = tempfile::tempdir().unwrap();
    let project = screenshot_queue_fixture(dir.path(), true, 3);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.choose(Choice::AllResults, cx);
        s.guided(24, cx);
        finish_jobs(s, cx);
    });
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for mode in 0..=5 {
            shell.update(cx, |s, cx| {
                s.viewer_mode = mode;
                s.navigation_preference = Some(false);
                s.configure_review_geometry(width);
                cx.notify();
            });
            draw(cx);
            let mut controls = vec![
                (Control::Guided(50), "action-Guided(50)"),
                (Control::Guided(54), "action-Guided(54)"),
                (Control::Guided(46), "zoom-select-button"),
                (Control::Phase5(36), "action-Phase5(36)"),
                (Control::Guided(44), "action-Guided(44)"),
                (Control::Phase5(37), "action-Phase5(37)"),
                (Control::Phase5(23), "action-Phase5(23)"),
            ];
            if width >= 1100. {
                controls.push((Control::ImageControls, "action-ImageControls"));
            }
            if mode == 4 {
                controls.extend([
                    (Control::Guided(57), "action-Guided(57)"),
                    (Control::Guided(56), "action-Guided(56)"),
                ]);
            }
            let bounds: Vec<_> = controls
                .iter()
                .map(|(_, name)| cx.debug_bounds(name).unwrap())
                .collect();
            for action in [24, 23] {
                shell.update(cx, |s, cx| s.guided(action, cx));
                shell.read_with(cx, |s, cx| {
                    assert!(s.viewer_pending.is_some() && s.viewer.is_none());
                    for (control, _) in &controls {
                        assert!(s.control_mounted(*control), "{control:?}: mode {mode}");
                        if !matches!(control, Control::Guided(50) | Control::Phase5(36)) {
                            assert!(
                                !s.control_enabled(*control, cx),
                                "image-dependent action {control:?} during loading"
                            );
                        }
                    }
                    assert_eq!(s.viewer_mode, mode);
                });
                draw(cx);
                for ((_, name), expected) in controls.iter().zip(&bounds) {
                    assert_eq!(
                        cx.debug_bounds(name).unwrap(),
                        *expected,
                        "{width}: mode {mode}, {name}"
                    );
                }
                #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
                if let Some(root) = &root
                    && action == 24
                    && matches!(mode, 0 | 4 | 5)
                {
                    crate::ui_renderer_capture::capture(
                        cx,
                        root,
                        &format!("navigation-{width:.0}x{height:.0}-mode-{mode}-loading"),
                    )
                    .unwrap();
                }
                shell.update(cx, finish_jobs);
                draw(cx);
                for ((_, name), expected) in controls.iter().zip(&bounds) {
                    assert_eq!(
                        cx.debug_bounds(name).unwrap(),
                        *expected,
                        "{width}: mode {mode}, {name} after loading"
                    );
                }
                #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
                if let Some(root) = &root
                    && action == 24
                    && matches!(mode, 0 | 4 | 5)
                {
                    crate::ui_renderer_capture::capture(
                        cx,
                        root,
                        &format!("navigation-{width:.0}x{height:.0}-mode-{mode}-ready"),
                    )
                    .unwrap();
                }
            }
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn every_view_mode_keeps_its_toolbar_controls_in_place_while_loading(cx: &mut TestAppContext) {
    toolbar_loading_matrix(cx, false);
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "explicit three-size screenshot navigation inspection"]
fn renderer_capture_result_navigation(cx: &mut TestAppContext) {
    toolbar_loading_matrix(cx, true);
}
