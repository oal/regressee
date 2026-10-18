// SPDX-License-Identifier: MPL-2.0
//! Viewer toolbar geometry across every mode and both navigation preferences.
use super::*;
use crate::{phase2_evidence_tests::inside, workflow_refinement_tests::load_review};
use gpui::TestAppContext;

#[gpui::test]
fn sidebar_tooltip_follows_mouse_and_keyboard_without_blurring(cx: &mut TestAppContext) {
    use crate::{keyboard_navigation_tests::press, phase4_hierarchy_tests::draw};
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    let capture_root = std::env::var_os("REGRESSEE_RENDERER_CAPTURE_DIR").map(|_| {
        cx.enable_offscreen_renderer().unwrap();
        crate::ui_renderer_capture::output_root().unwrap()
    });
    cx.update(crate::keyboard_navigation_tests::bind);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        // Fresh windows establish tooltip absence without GPUI's stale selectors.
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.navigation_preference = Some(false);
            s.notice = None;
            s.error = None;
            cx.notify();
        });
        draw(cx);
        let requests = shell.read_with(cx, |s, _| s.next_request);
        let button = cx.debug_bounds("action-Guided(53)").unwrap();
        cx.simulate_mouse_move(button.center(), None, gpui::Modifiers::none());
        cx.simulate_click(button.center(), gpui::Modifiers::none());
        draw(cx);
        assert!(cx.debug_bounds("toolbar-focus-hint").is_none());
        let away = gpui::point(button.right() + px(100.), button.bottom() + px(100.));
        cx.simulate_mouse_move(away, None, gpui::Modifiers::none());
        cx.executor().advance_clock(Duration::from_millis(600));
        cx.run_until_parked();
        draw(cx);
        assert!(cx.debug_bounds("toolbar-hover-hint").is_none());
        assert!(cx.debug_bounds("toolbar-focus-hint").is_none());
        cx.update(|w, cx| {
            shell.read_with(cx, |s, cx| {
                assert!(s.navigation_collapsed);
                assert!(s.handle(Control::Guided(53), cx).is_focused(w));
                assert!(!s.tool_hint_input.as_ref().unwrap().is_keyboard());
            });
        });
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if let Some(root) = &capture_root {
            crate::ui_renderer_capture::capture(
                cx,
                root,
                &format!("tooltip-{width:.0}x{height:.0}-mouse-away"),
            )
            .unwrap();
        }

        // Re-hovering still offers the ordinary delayed identifying tooltip.
        let button = cx.debug_bounds("action-Guided(53)").unwrap();
        cx.simulate_mouse_move(button.center(), None, gpui::Modifiers::none());
        cx.executor().advance_clock(Duration::from_millis(600));
        cx.run_until_parked();
        draw(cx);
        assert!(cx.debug_bounds("toolbar-hover-hint").is_some());
        cx.simulate_mouse_move(away, None, gpui::Modifiers::none());
        draw(cx);

        // Real bound Tab events must restore keyboard hints, not just raw keys.
        press(cx, "tab");
        press(cx, "shift-tab");
        assert!(cx.debug_bounds("toolbar-focus-hint").is_some());
        press(cx, "enter");
        cx.update(|w, cx| {
            shell.read_with(cx, |s, cx| {
                assert!(!s.navigation_collapsed, "Enter activates exactly once");
                assert!(s.handle(Control::Guided(53), cx).is_focused(w));
                assert!(s.tool_hint_input.as_ref().unwrap().is_keyboard());
                assert_eq!(s.next_request, requests);
            });
        });
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if let Some(root) = &capture_root {
            crate::ui_renderer_capture::capture(
                cx,
                root,
                &format!("tooltip-{width:.0}x{height:.0}-keyboard"),
            )
            .unwrap();
        }
        cx.simulate_mouse_move(away, None, gpui::Modifiers::none());
        draw(cx);
        cx.update(|w, cx| {
            shell.read_with(cx, |s, cx| {
                assert!(!s.tool_hint_input.as_ref().unwrap().is_keyboard());
                assert!(s.handle(Control::Guided(53), cx).is_focused(w));
            });
        });
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if let Some(root) = &capture_root {
            crate::ui_renderer_capture::capture(
                cx,
                root,
                &format!("tooltip-{width:.0}x{height:.0}-return-to-mouse"),
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

// This expectation is the painted layout, not another sort of the shell's
// candidate list. Hidden and disabled controls must disappear without changing
// the order of the remaining destinations.
fn review_reading_order() -> Vec<Control> {
    vec![
        Control::Navigate(Screen::Projects),
        Control::Navigate(Screen::Results),
        Control::PagesBaseline,
        Control::Navigate(Screen::History),
        Control::Navigate(Screen::Setup),
        Control::Navigate(Screen::Settings),
        Control::Guided(53),
        Control::ChooseProject,
        Control::CheckChanges,
        Control::BaselineUpdate,
        Control::RefreshEvidence,
        Control::Guided(50),
        Control::Guided(54),
        Control::Guided(57),
        Control::Guided(56),
        Control::Guided(46),
        Control::ImageControls,
        Control::Phase5(36),
        Control::Guided(44),
        Control::ShowAllResults,
        Control::CrawlField(9),
        Control::Guided(17),
        Control::Guided(45),
        Control::Guided(23),
        Control::Guided(24),
        Control::Viewer,
        Control::SplitDivider,
        Control::Phase5(10),
        Control::Phase5(11),
        Control::Phase5(20),
        Control::Phase5(21),
        Control::Phase5(22),
        Control::Phase5(37),
        Control::Phase5(33),
        Control::Phase5(23),
        Control::Phase5(24),
    ]
}

fn traverse(shell: &Entity<Shell>, cx: &mut gpui::VisualTestContext, expected: &[Control]) {
    let expected: Vec<_> = shell.read_with(cx, |s, cx| {
        let expected: Vec<_> = expected
            .iter()
            .copied()
            .filter(|c| s.control_enabled(*c, cx))
            .collect();
        let actual: Vec<_> = s
            .visible_controls()
            .into_iter()
            .filter(|c| s.control_enabled(*c, cx))
            .collect();
        assert_eq!(
            actual, expected,
            "enabled stops must match the painted layout"
        );
        expected
    });
    assert!(!expected.is_empty());
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| w.focus(&s.handle(expected[0], cx)));
    });
    for (key, sequence) in [
        (
            "tab",
            expected
                .iter()
                .cycle()
                .skip(1)
                .take(expected.len())
                .copied()
                .collect::<Vec<_>>(),
        ),
        ("shift-tab", expected.iter().rev().copied().collect()),
    ] {
        for target in sequence {
            crate::keyboard_navigation_tests::press(cx, key);
            cx.update(|w, cx| {
                shell.read_with(cx, |s, cx| {
                    assert_eq!(s.focused(w, cx), Some(target), "{key}");
                });
            });
        }
    }
}

#[gpui::test]
fn review_tab_sequence_matches_painted_groups_in_both_directions(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    let requests = shell.read_with(cx, |s, _| s.next_request);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for hidden_navigation in [false, true] {
            for (mode, collapsed_queue) in [(0, false), (2, false), (4, false), (5, true)] {
                shell.update(cx, |s, cx| {
                    s.navigation_preference = Some(hidden_navigation);
                    s.configure_review_geometry(width);
                    s.viewer_mode = mode;
                    s.review_queue_collapsed = collapsed_queue;
                    s.presentation_focus = None;
                    s.notice = None;
                    s.error = None;
                    cx.notify();
                });
                crate::phase4_hierarchy_tests::draw(cx);
                // Verify the expectation against real geometry, not just IDs.
                let mut right = cx.debug_bounds("viewer-toolbar").unwrap().left();
                for (control, name) in [
                    (Control::Guided(50), "action-Guided(50)"),
                    (Control::Guided(54), "action-Guided(54)"),
                    (Control::Guided(57), "action-Guided(57)"),
                    (Control::Guided(56), "action-Guided(56)"),
                    (Control::Guided(46), "zoom-select-button"),
                    (Control::ImageControls, "action-ImageControls"),
                    (Control::Phase5(36), "action-Phase5(36)"),
                    (Control::Guided(44), "action-Guided(44)"),
                ] {
                    // Debug selectors can retain the last bounds of unmounted
                    // Before/After controls after switching out of Single image.
                    if shell.read_with(cx, |s, _| s.control_mounted(control)) {
                        let b = cx.debug_bounds(name).unwrap();
                        assert!(b.left() >= right, "{name}: toolbar reading order");
                        right = b.right();
                    }
                }
                if !collapsed_queue {
                    assert!(
                        cx.debug_bounds("viewer-toolbar").unwrap().bottom()
                            <= cx.debug_bounds("queue-toolbar").unwrap().top()
                    );
                }
                traverse(&shell, cx, &review_reading_order());
            }
        }
    }
    shell.read_with(cx, |s, _| assert_eq!(s.next_request, requests));
    owned.join_after_ui();
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn review_menu_tab_exits_and_inspector_contents_keep_reading_order(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            s.navigation_preference = Some(true);
            s.configure_review_geometry(width);
            s.review_queue_collapsed = false;
            s.presentation_focus = None;
            s.notice = None;
            s.error = None;
            cx.notify();
        });
        crate::phase4_hierarchy_tests::draw(cx);
        // Menus are one Tab stop, with arrow-key choices. Dismiss and continue
        // from the trigger in either direction, without landing in popup rows.
        for (trigger, key, next) in [
            (Control::Guided(54), "tab", Control::Guided(46)),
            (Control::Guided(54), "shift-tab", Control::Guided(50)),
            (
                Control::Guided(46),
                "tab",
                if width >= 1100. {
                    Control::ImageControls
                } else {
                    Control::Phase5(36)
                },
            ),
            (Control::Guided(46), "shift-tab", Control::Guided(54)),
        ] {
            cx.update(|w, cx| shell.read_with(cx, |s, cx| w.focus(&s.handle(trigger, cx))));
            crate::keyboard_navigation_tests::press(cx, "enter");
            crate::keyboard_navigation_tests::press(cx, key);
            cx.update(|w, cx| {
                shell.read_with(cx, |s, cx| {
                    assert!(!s.viewer_mode_open && !s.zoom_select.read(cx).is_open());
                    assert_eq!(s.focused(w, cx), Some(next), "{trigger:?}: {key}");
                })
            });
        }
        for panel in ["filters", "details", "note", "analysis", "drawing"] {
            shell.update(cx, |s, cx| {
                s.result_filters_open = panel == "filters";
                s.capture_details_open = panel == "details";
                s.capture_diagnostics_open = panel == "details";
                s.review_note_open = panel == "note";
                s.review_tools = matches!(panel, "analysis" | "drawing");
                s.mask_draw = panel == "drawing";
                s.analysis_advanced = panel == "analysis";
                s.details.insert(Detail::IgnoredAreas);
                s.details.insert(Detail::Analysis);
                s.presentation_focus = None;
                s.analysis_fields[9].update(cx, |f, cx| f.set_value("Keep this unsaved note", cx));
                cx.notify();
            });
            crate::phase4_hierarchy_tests::draw(cx);
            let mut expected = review_reading_order();
            let evidence = expected.iter().position(|c| *c == Control::Viewer).unwrap();
            if panel == "filters" {
                expected.splice(
                    evidence..evidence,
                    [Control::CloseOptions]
                        .into_iter()
                        .chain(Choice::RESULTS.map(Control::Choice))
                        .chain([Control::Guided(19)]),
                );
            } else if panel == "drawing" {
                let search = expected
                    .iter()
                    .position(|c| *c == Control::ShowAllResults)
                    .unwrap();
                expected.insert(search, Control::Phase5(14));
            } else {
                let mut contents = vec![Control::Phase5(39)];
                match panel {
                    "details" => {
                        contents.extend([Control::Phase5(38), Control::CopyCaptureDiagnostics])
                    }
                    "note" => contents.extend([Control::AnalysisField(9), Control::Phase5(32)]),
                    _ => {
                        contents.extend([0, 1, 2, 4, 5, 26].map(Control::Phase5));
                        contents.push(Control::Details(Detail::IgnoredAreas));
                        contents.extend([14, 34, 18, 15, 17, 16, 19, 3].map(Control::Phase5));
                        contents.extend((0..9).map(Control::AnalysisField));
                        contents.extend([27, 28].map(Control::Phase5));
                        contents.push(Control::Details(Detail::Analysis));
                        contents.extend([12, 13].map(Control::Phase5));
                    }
                }
                expected.splice(evidence + 1..evidence + 1, contents);
            }
            traverse(&shell, cx, &expected);
            shell.read_with(cx, |s, cx| {
                assert_eq!(
                    s.analysis_fields[9].read(cx).content.as_ref(),
                    "Keep this unsaved note"
                );
            });
        }
        shell.update(cx, |s, cx| {
            s.review_tools = false;
            s.mask_draw = false;
            cx.notify();
        });
    }
    owned.join_after_ui();
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn compact_toolbars_fit_all_modes_without_changing_evidence_or_drafts(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        // Item navigation follows the vertical queue; pagination stays horizontal.
        for (control, icon) in [
            (Control::Guided(23), ToolIcon::Up),
            (Control::Guided(24), ToolIcon::Down),
            (Control::PreviousPage, ToolIcon::Previous),
            (Control::NextPage, ToolIcon::Next),
            (Control::Guided(48), ToolIcon::Previous),
            (Control::Guided(49), ToolIcon::Next),
        ] {
            assert_eq!(s.action_icon(control, cx), Some(icon));
        }
        s.analysis_fields[9].update(cx, |f, cx| f.set_value("Toolbar draft stays", cx));
    });
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for navigation_hidden in [false, true] {
            for mode in 0..6 {
                shell.update(cx, |s, cx| {
                    s.navigation_preference = Some(navigation_hidden);
                    s.configure_review_geometry(width);
                    s.viewer_mode = mode;
                    s.mask_draw = false;
                    s.review_tools = false;
                    s.capture_details_open = false;
                    s.viewer_mode_open = false;
                    cx.notify();
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
                assert_eq!(toolbar.size.height, px(40.));
                inside(toolbar, cx.debug_bounds("bounded-review").unwrap());
                let heading = cx.debug_bounds("inspected-target-context").unwrap();
                let identity = cx.debug_bounds("inspection-identity-group").unwrap();
                let sources = cx
                    .debug_bounds("inspected-target-profile-environments")
                    .unwrap();
                inside(identity, heading);
                inside(sources, cx.debug_bounds("bounded-review").unwrap());
                assert!(sources.top() >= heading.bottom());
                inside(
                    cx.debug_bounds("inspection-profile-dimensions").unwrap(),
                    identity,
                );
                assert_eq!(sources.left(), identity.left());
                assert!(
                    cx.debug_bounds("inspection-before-environment-value")
                        .unwrap()
                        .size
                        .width
                        > px(32.)
                );
                assert!(
                    cx.debug_bounds("inspection-profile-name")
                        .unwrap()
                        .size
                        .width
                        > px(16.)
                );
                let mut last = toolbar.left();
                let mut names = vec!["action-Guided(50)", "action-Guided(54)"];
                if mode == 4 {
                    names.extend(["action-Guided(57)", "action-Guided(56)"]);
                }
                names.extend([
                    "zoom-select-button",
                    "action-Phase5(36)",
                    "action-Guided(44)",
                ]);
                for name in names {
                    let b = cx.debug_bounds(name).unwrap();
                    inside(b, toolbar);
                    assert!(b.left() >= last, "{width}, mode={mode}: overlapping {name}");
                    last = b.right();
                }
                let queue = cx.debug_bounds("queue-toolbar").unwrap();
                for name in [
                    "action-Guided(45)",
                    "action-Guided(23)",
                    "action-Guided(24)",
                ] {
                    inside(cx.debug_bounds(name).unwrap(), queue);
                }
                assert_eq!(
                    cx.debug_bounds("tool-label-Guided(54)")
                        .unwrap()
                        .size
                        .height,
                    px(CONTROL_LINE_HEIGHT)
                );
                shell.read_with(cx, |s, cx| {
                    assert_eq!(
                        s.analysis_fields[9].read(cx).content.as_ref(),
                        "Toolbar draft stays"
                    )
                });
                let review = cx.debug_bounds("ordinary-review-decisions").unwrap();
                for name in ["Phase5(20)", "Phase5(21)", "Phase5(22)", "Phase5(37)"] {
                    let frame = cx.debug_bounds(format!("action-{name}").leak()).unwrap();
                    inside(frame, review);
                    inside(
                        cx.debug_bounds(format!("action-label-{name}").leak())
                            .unwrap(),
                        frame,
                    );
                    inside(
                        cx.debug_bounds(format!("action-icon-{name}").leak())
                            .unwrap(),
                        frame,
                    );
                }
            }
        }
    }
    // Focus labels are available without pointer hover or task activation.
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| w.focus(&s.handle(Control::Phase5(36), cx)));
        let _ = w.draw(cx);
    });
    assert!(cx.debug_bounds("toolbar-focus-hint").is_some());
    owned.join_after_ui();
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn single_image_sides_are_explicit_idempotent_and_keyboard_reachable(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    cx.update(|cx| cx.bind_keys([KeyBinding::new("space", Activate, Some("Shell"))]));
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.analysis_fields[9].update(cx, |f, cx| f.set_value("Keep this draft", cx));
    });
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            assert!(!s.control_mounted(Control::Guided(56)));
            assert!(!s.control_mounted(Control::Guided(57)));
            s.activate_control(Control::Guided(54), w, cx);
            s.activate_control(Control::Guided(55), w, cx);
            w.blur();
            cx.notify();
        });
        let _ = w.draw(cx);
    });
    let key = shell.read_with(cx, |s, _| s.selected_key.clone());
    // Pointer selects a destination, never a toggle. Repeated clicks retain it.
    for (control, after) in [(56, true), (56, true), (57, false), (57, false)] {
        let name = if after {
            "action-Guided(56)"
        } else {
            "action-Guided(57)"
        };
        let target = cx.debug_bounds(name).unwrap().center();
        cx.simulate_click(target, gpui::Modifiers::none());
        cx.update(|w, cx| {
            let _ = w.draw(cx);
            shell.read_with(cx, |s, cx| {
                assert_eq!(s.full_width_after, after, "Guided({control})");
                assert_eq!(
                    s.viewer_pane_indices(),
                    if after { &[1][..] } else { &[0][..] }
                );
                assert!(s.handle(Control::Guided(control), cx).is_focused(w));
            });
        });
    }
    cx.simulate_keystrokes("tab space"); // Before → After; Space selects After.
    shell.read_with(cx, |s, _| assert!(s.full_width_after));
    cx.simulate_keystrokes("shift-tab enter");
    shell.read_with(cx, |s, _| assert!(!s.full_width_after));

    // Side selection preserves both automatic and manual source transforms.
    for fit in [true, false] {
        let before = shell.update(cx, |s, cx| {
            s.viewer_fit = fit;
            if fit {
                s.zoom = s.fitted_viewer_zoom();
            } else {
                s.zoom = 0.75;
                s.pan_x = 11.;
                s.pan_y = 12;
            }
            cx.notify();
            (s.zoom, s.pan_x, s.pan_y, s.next_request)
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                for side in [56, 57] {
                    s.activate_control(Control::Guided(side), w, cx);
                    assert_eq!((s.zoom, s.pan_x, s.pan_y, s.next_request), before);
                    assert_eq!(s.selected_key, key);
                    assert_eq!(
                        s.analysis_fields[9].read(cx).content.as_ref(),
                        "Keep this draft"
                    );
                }
                s.activate_control(Control::Guided(54), w, cx);
                s.activate_control(Control::Phase5(6), w, cx);
                assert!(!s.control_mounted(Control::Guided(56)));
                assert!(!s.control_mounted(Control::Guided(57)));
                s.activate_control(Control::Guided(54), w, cx);
                s.activate_control(Control::Guided(55), w, cx);
            });
        });
    }
    owned.join_after_ui();
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
