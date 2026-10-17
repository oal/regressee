// SPDX-License-Identifier: MPL-2.0
//! Bounded toolbar inspection with actual native fonts and synthetic evidence.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

#[gpui::test]
#[ignore = "explicit three-size single-image choice inspection"]
fn renderer_capture_single_image_sides(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("single-image-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), false);
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.notice = None;
            s.error = None;
            s.navigation_preference = Some(false);
            s.configure_review_geometry(width);
        });
        for scene in ["menu", "before", "after", "before-focus"] {
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    match scene {
                        "menu" => s.activate_control(Control::Guided(54), w, cx),
                        "before" => s.activate_control(Control::Guided(55), w, cx),
                        "after" => s.activate_control(Control::Guided(56), w, cx),
                        _ => s.activate_control(Control::Guided(57), w, cx),
                    }
                    if scene == "before-focus" {
                        w.focus(&s.handle(Control::Guided(57), cx));
                    } else {
                        w.blur();
                    }
                    cx.notify();
                });
                let _ = w.draw(cx);
            });
            if scene != "menu" {
                let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
                let before = cx.debug_bounds("action-Guided(57)").unwrap();
                let after = cx.debug_bounds("action-Guided(56)").unwrap();
                let zoom = cx.debug_bounds("zoom-select-button").unwrap();
                for b in [before, after, zoom] {
                    crate::phase2_evidence_tests::inside(b, toolbar);
                }
                assert!(before.right() <= after.left() && after.right() <= zoom.left());
                shell.read_with(cx, |s, _| assert_eq!(s.full_width_after, scene == "after"));
            }
            capture(
                cx,
                &root,
                &format!("single-image-{width:.0}x{height:.0}-{scene}"),
            )
            .unwrap();
        }
        owned.join_after_ui();
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
#[ignore = "explicit three-size tooltip-cleanup inspection"]
fn renderer_capture_quiet_tooltips(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("quiet-tooltips-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), true);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        for scene in ["zoom", "zoom-menu", "view-menu", "metadata", "details"] {
            // A fresh window also makes absence assertions immune to stale bounds.
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                s.notice = None;
                s.error = None;
                s.navigation_preference = Some(false);
                s.configure_review_geometry(width);
                cx.notify();
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            let target = cx
                .debug_bounds(match scene {
                    "view-menu" => "action-Guided(54)",
                    "metadata" => "inspection-before-environment",
                    "details" => "action-Phase5(36)",
                    _ => "zoom-select-button",
                })
                .unwrap();
            cx.simulate_mouse_move(target.center(), None, gpui::Modifiers::none());
            if matches!(scene, "zoom-menu" | "view-menu" | "details") {
                cx.simulate_click(target.center(), gpui::Modifiers::none());
            }
            if scene == "details" {
                // The Details icon is allowed to name itself. Read inside the
                // opened panel rather than leaving the pointer on that icon.
                cx.run_until_parked();
                let body = cx.debug_bounds("review-details-scroll").unwrap();
                cx.simulate_mouse_move(body.center(), None, gpui::Modifiers::none());
            }
            cx.executor().advance_clock(Duration::from_secs(2));
            cx.run_until_parked();
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            assert!(
                cx.debug_bounds("toolbar-hover-hint").is_none(),
                "{width}: {scene}"
            );
            if scene != "details" {
                // Pointer activation may retain focus on the icon. Its short
                // identifying name is permitted, unlike labelled-control help.
                assert!(
                    cx.debug_bounds("toolbar-focus-hint").is_none(),
                    "{width}: {scene}"
                );
            }
            shell.read_with(cx, |s, cx| match scene {
                "zoom-menu" => assert!(s.zoom_select.read(cx).is_open()),
                "view-menu" => assert!(s.viewer_mode_open),
                "details" => {
                    assert!(s.capture_details_open);
                    assert!(s.capture_summary().iter().any(|(name, value)| {
                        name == "Before environment" && value.contains("Recorded reference")
                    }));
                }
                _ => (),
            });
            capture(
                cx,
                &root,
                &format!("quiet-tooltips-{width:.0}x{height:.0}-{scene}"),
            )
            .unwrap();
            owned.join_after_ui();
        }
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
#[ignore = "explicit three-size native toolbar inspection"]
fn renderer_capture_toolbars(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("toolbar-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), false);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = vec![];
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.notice = None;
            s.error = None;
        });
        for scene in [
            "compare",
            "view-menu",
            "analysis",
            "focus-help",
            "full-width",
            "drawing",
            "pages",
            "history-runs",
            "history-baselines",
            "history-pagination",
        ] {
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.focus.screen = Screen::Results;
                    s.focus.review_surface = ReviewSurface::Changes;
                    s.review_tools = false;
                    s.capture_details_open = false;
                    s.viewer_mode_open = false;
                    s.mask_draw = false;
                    s.viewer_mode = 0;
                    s.navigation_preference = Some(false);
                    s.review_queue_collapsed = false;
                    s.configure_review_geometry(width);
                    w.blur();
                    match scene {
                        "view-menu" => {
                            s.activate_control(Control::Guided(54), w, cx);
                        }
                        "analysis" => {
                            s.activate_control(Control::Guided(44), w, cx);
                        }
                        "focus-help" => w.focus(&s.handle(Control::Phase5(36), cx)),
                        "full-width" => {
                            s.viewer_mode = 4;
                            s.review_queue_collapsed = true;
                        }
                        "drawing" => {
                            s.viewer_mode = 4;
                            s.review_queue_collapsed = true;
                            s.activate_control(Control::Guided(44), w, cx);
                            if !s.details.contains(&Detail::IgnoredAreas) {
                                s.activate_control(Control::Details(Detail::IgnoredAreas), w, cx);
                            }
                            s.activate_control(Control::Phase5(14), w, cx);
                        }
                        "pages" => s.focus.review_surface = ReviewSurface::Pages,
                        "history-runs" | "history-baselines" | "history-pagination" => {
                            s.focus.screen = Screen::History;
                            s.history_section = usize::from(scene == "history-baselines");
                            s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                            s.queue(Work::History { project, page: 0 }, cx);
                            finish_jobs(s, cx);
                        }
                        _ => (),
                    }
                    cx.notify();
                });
                let _ = w.draw(cx);
            });
            if scene == "history-pagination" {
                shell.update(cx, |s, cx| {
                    s.screen_scroll
                        .set_offset(gpui::point(px(0.), -s.screen_scroll.max_offset().height));
                    cx.notify();
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
            }
            shell.read_with(cx, |s, cx| match scene {
                "view-menu" => assert!(s.viewer_mode_open),
                "drawing" => assert!(s.mask_draw && s.control_enabled(Control::Phase5(14), cx)),
                "analysis" => assert!(s.review_tools && s.viewer_inspector_open()),
                _ => (),
            });
            let label = format!("toolbar-{width:.0}x{height:.0}-{scene}");
            capture(cx, &root, &label).unwrap();
            manifest.push(label);
        }
        owned.join_after_ui();
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("toolbar-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
