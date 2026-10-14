// SPDX-License-Identifier: MPL-2.0
//! Real mounted shell/paint assertions; not OS-input qualification.
use super::*;
use crate::workflow_refinement_tests::load_review;
use gpui::TestAppContext;

pub(super) fn inside(b: Bounds<Pixels>, owner: Bounds<Pixels>) {
    assert!(
        b.left() >= owner.left() && b.right() <= owner.right(),
        "{b:?} outside {owner:?}"
    );
    assert!(
        b.top() >= owner.top() && b.bottom() <= owner.bottom(),
        "{b:?} outside {owner:?}"
    );
}

/// Shared by the simulated regression and the targeted real-font render recipe.
/// The caller supplies a freshly loaded pair; all presentation changes use the
/// mounted controller routes and completion uses actual pointer/key dispatch.
pub(super) fn exercise_full_width_drawing_toolbar(
    shell: &Entity<Shell>,
    cx: &mut gpui::VisualTestContext,
    width: f32,
    navigation_open: bool,
    after: bool,
    inspect_drawing: impl FnOnce(&mut gpui::VisualTestContext),
) {
    use regressee_app::analysis_engine::Rectangle;
    let draw = |cx: &mut gpui::VisualTestContext| {
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
    };
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            if s.control_enabled(Control::DismissFeedback, cx) {
                s.activate_control(Control::DismissFeedback, w, cx);
            }
            s.navigation_preference = Some(true);
            s.configure_review_geometry(width);
            if navigation_open {
                s.activate_control(Control::Guided(53), w, cx);
            }
            assert_eq!(s.navigation_collapsed, !navigation_open);
            s.activate_control(Control::Guided(50), w, cx);
            assert!(s.review_queue_collapsed);
            s.activate_control(Control::Guided(54), w, cx);
            s.activate_control(Control::Guided(55), w, cx);
            if after {
                s.activate_control(Control::Guided(56), w, cx);
            }
            assert_eq!((s.viewer_mode, s.full_width_after), (4, after));
            s.analysis_fields[9].update(cx, |f, cx| {
                f.set_value("Pair-bound unsaved note survives drawing exits", cx)
            });
            s.mask_rectangles = vec![Rectangle {
                x: 1,
                y: 1,
                width: 2,
                height: 2,
            }];
            s.mask_dirty = true;
            s.mask_selected = None;
            assert!(s.control_enabled(Control::Guided(44), cx));
            s.activate_control(Control::Guided(44), w, cx);
            assert!(s.review_tools && s.viewer_inspector_open());
            if !s.details.contains(&Detail::IgnoredAreas) {
                s.activate_control(Control::Details(Detail::IgnoredAreas), w, cx);
            }
            assert!(s.control_mounted(Control::Phase5(14)));
            assert!(s.control_enabled(Control::Phase5(14), cx));
            s.activate_control(Control::Phase5(14), w, cx);
            assert!(s.mask_draw && !s.viewer_inspector_open());
        })
    });
    draw(cx);
    let (key, request, identity, requests, note) = shell.read_with(cx, |s, cx| {
        (
            s.selected_key.clone(),
            s.viewer.as_ref().unwrap().request.clone(),
            s.inspection_identity(),
            s.next_request,
            s.analysis_fields[9].read(cx).content.to_string(),
        )
    });
    assert!(request.is_some());
    let assert_retained = |cx: &mut gpui::VisualTestContext| {
        shell.read_with(cx, |s, cx| {
            assert_eq!(s.selected_key, key);
            assert_eq!(s.viewer.as_ref().unwrap().request, request);
            assert_eq!(s.inspection_identity(), identity);
            assert_eq!(s.next_request, requests);
            assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), note);
            assert!(s.mask_dirty && s.review_note_dirty(cx));
            assert_eq!((s.viewer_mode, s.full_width_after), (4, after));
            assert_eq!(s.navigation_collapsed, !navigation_open);
            assert!(s.review_queue_collapsed);
        });
    };
    let assert_drawing = |cx: &mut gpui::VisualTestContext| {
        draw(cx);
        let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
        let workspace = cx.debug_bounds("bounded-review").unwrap();
        let window = cx.update(|w, _| Bounds::new(gpui::point(px(0.), px(0.)), w.viewport_size()));
        assert_eq!(toolbar.size.height, px(40.));
        inside(toolbar, workspace);
        inside(toolbar, window);
        for (control, name) in [
            (Control::Guided(50), "action-Guided(50)"),
            (Control::Guided(54), "action-Guided(54)"),
            (Control::Guided(56), "action-Guided(56)"),
            (Control::Guided(46), "zoom-select-button"),
            (Control::Phase5(14), "action-Phase5(14)"),
        ] {
            shell.read_with(cx, |s, cx| {
                assert!(s.control_mounted(control) && s.control_enabled(control, cx));
                assert!(s.visible_controls().contains(&control));
            });
            let bounds = cx.debug_bounds(name).unwrap();
            assert!(bounds.size.width > px(0.) && bounds.size.height == px(40.));
            inside(bounds, toolbar);
            inside(bounds, window);
        }
        // Debug bounds can be retained from earlier frames; use mounting and
        // authoritative activation guards, not absence of an old selector.
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                let last = s.last_action;
                for control in [Control::Phase5(36), Control::Guided(44)] {
                    assert!(!s.control_mounted(control));
                    assert!(!s.control_enabled(control, cx));
                    s.activate_control(control, w, cx);
                }
                assert_eq!(s.last_action, last);
                assert!(s.mask_draw && s.review_tools);
            });
        });
        for name in [
            "inspected-target-path",
            "inspection-profile",
            "inspection-before-environment",
            "inspection-after-environment",
        ] {
            inside(cx.debug_bounds(name).unwrap(), workspace);
        }
        assert_retained(cx);
    };
    let assert_completed = |cx: &mut gpui::VisualTestContext| {
        draw(cx);
        shell.read_with(cx, |s, cx| {
            assert!(!s.mask_draw && s.mask_drag.is_none());
            assert!(s.review_tools && s.viewer_inspector_open());
            assert!(s.control_enabled(Control::Guided(44), cx));
            assert!(s.control_enabled(Control::Phase5(14), cx));
        });
        let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
        for name in ["action-Guided(44)", "action-Phase5(36)"] {
            inside(cx.debug_bounds(name).unwrap(), toolbar);
        }
        assert_retained(cx);
    };
    assert_drawing(cx);
    inspect_drawing(cx);
    let rectangles = shell.read_with(cx, |s, _| s.mask_rectangles.clone());
    // Actual painted pointer exit, not a direct phase5_action call.
    let done = cx.debug_bounds("action-Phase5(14)").unwrap();
    cx.simulate_click(done.center(), gpui::Modifiers::none());
    assert_completed(cx);
    shell.read_with(cx, |s, _| assert_eq!(s.mask_rectangles, rectangles));

    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.activate_control(Control::Phase5(14), w, cx);
            w.focus(&s.handle(Control::Guided(50), cx));
        })
    });
    assert_drawing(cx);
    for _ in 0..128 {
        if cx.update(|w, cx| {
            shell.read_with(cx, |s, cx| s.handle(Control::Phase5(14), cx).is_focused(w))
        }) {
            break;
        }
        cx.simulate_keystrokes("tab");
        cx.update(|w, cx| {
            shell.read_with(cx, |s, cx| {
                let focused = s
                    .focused(w, cx)
                    .expect("Tab reaches a mounted enabled control");
                assert!(s.control_mounted(focused) && s.control_enabled(focused, cx));
            })
        });
    }
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(s.handle(Control::Phase5(14), cx).is_focused(w))
        })
    });
    cx.simulate_keystrokes("enter");
    assert_completed(cx);
    shell.read_with(cx, |s, _| assert_eq!(s.mask_rectangles, rectangles));

    // Draw on the actually displayed Before/After pane. Half-source-pixel
    // points make the exact integer rectangle stable at fit/manual scales.
    cx.update(|w, cx| shell.update(cx, |s, cx| s.activate_control(Control::Phase5(14), w, cx)));
    assert_drawing(cx);
    let pane = cx
        .debug_bounds(if after {
            "phase5-after"
        } else {
            "phase5-before"
        })
        .unwrap();
    let zoom = shell.read_with(cx, |s, _| s.zoom);
    let a = pane.origin + gpui::point(px(1. + 8.5 * zoom), px(1. + 8.5 * zoom));
    let b = pane.origin + gpui::point(px(1. + 24.5 * zoom), px(1. + 16.5 * zoom));
    inside(Bounds::new(a, size(b.x - a.x, b.y - a.y)), pane);
    cx.simulate_mouse_down(a, MouseButton::Left, gpui::Modifiers::none());
    draw(cx);
    cx.simulate_mouse_move(b, MouseButton::Left, gpui::Modifiers::none());
    draw(cx);
    cx.simulate_mouse_up(b, MouseButton::Left, gpui::Modifiers::none());
    assert_completed(cx);
    let drawn = shell.read_with(cx, |s, _| {
        assert_eq!(&s.mask_rectangles[..rectangles.len()], rectangles);
        assert_eq!(
            s.mask_rectangles.last(),
            Some(&Rectangle {
                x: 8,
                y: 8,
                width: 16,
                height: 8
            })
        );
        s.mask_rectangles.clone()
    });
    cx.update(|w, cx| shell.update(cx, |s, cx| s.activate_control(Control::Phase5(14), w, cx)));
    assert_drawing(cx);
    cx.simulate_keystrokes("escape");
    draw(cx);
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(!s.mask_draw && !s.review_tools && s.mask_drag.is_none());
            assert_eq!(s.mask_rectangles, drawn);
            assert!(s.control_enabled(Control::Guided(44), cx));
            assert!(s.handle(Control::Guided(44), cx).is_focused(w));
        })
    });
    assert_retained(cx);
    // The restored visible Analysis trigger really reopens the editor.
    let analysis = cx.debug_bounds("action-Guided(44)").unwrap();
    cx.simulate_click(analysis.center(), gpui::Modifiers::none());
    assert_completed(cx);
}

#[gpui::test]
fn drawing_toolbar_full_width_exits_fit_mounted_bounds_and_preserve_pair_drafts(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| {
        zoom_select::bind(cx);
        cx.bind_keys([
            KeyBinding::new("enter", Activate, Some("Shell")),
            KeyBinding::new("escape", Dismiss, Some("Shell")),
            KeyBinding::new("tab", Tab, Some("Shell")),
        ]);
    });
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    for (width, height) in [(820., 600.), (1280., 800.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for navigation_open in [true, false] {
            for after in [false, true] {
                shell.update(cx, |s, cx| {
                    s.mask_dirty = false;
                    s.review_tools = false;
                    s.mask_draw = false;
                    s.viewer_mode = 0;
                    s.full_width_after = false;
                    s.review_queue_collapsed = false;
                    load_review(s, project, cx);
                });
                exercise_full_width_drawing_toolbar(
                    &shell,
                    cx,
                    width,
                    navigation_open,
                    after,
                    |_| {},
                );
            }
        }
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn same_target_painted_height_and_toolbar_survive_queue_modes_navigation_and_long_identity(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.evidence.latest.as_mut().unwrap().run.targets[0].requested = "https://example.test/account/orders/customer-service/release-notes/long-retained-path/?sort=recent&locale=nb-NO#delivery".parse().unwrap();
        s.evidence.latest.as_mut().unwrap().run.environment.name = "Historical check · long recorded environment".into();
        s.viewer.as_mut().unwrap().before_environment = Some("Historical reference · not today's settings".into());
        s.project.as_mut().unwrap().project.environments[0].name = "Today's unrelated configuration".into();
        s.viewer_fit = false; s.zoom = 0.75; s.pan_x = 11.; s.pan_y = 12;
        s.analysis_fields[9].update(cx, |f,cx| f.set_value("retained draft",cx));
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for navigation in [true, false] {
            shell.update(cx, |s, cx| {
                s.navigation_preference = Some(navigation);
                s.review_queue_collapsed = false;
                s.viewer_mode = 0;
                cx.notify();
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            let default = cx.debug_bounds("phase5-before").unwrap();
            for state in 0..4 {
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| match state {
                        0 => s.activate_control(Control::Guided(50), w, cx),
                        1 => {
                            s.activate_control(Control::Guided(54), w, cx);
                            s.activate_control(Control::Guided(55), w, cx);
                        }
                        2 => s.activate_control(Control::Guided(56), w, cx),
                        _ => {
                            s.activate_control(Control::Guided(54), w, cx);
                            s.activate_control(Control::Phase5(6), w, cx);
                        }
                    })
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                let pane = cx
                    .debug_bounds(if state == 2 {
                        "phase5-after"
                    } else {
                        "phase5-before"
                    })
                    .unwrap();
                assert!(
                    pane.size.height >= default.size.height,
                    "{width}, navigation={navigation}, state={state}: {pane:?} vs {default:?}"
                );
                assert!(pane.size.height > px(0.)); // Absolute native-font floor is asserted by the renderer recipe.
                let workspace = cx.debug_bounds("bounded-review").unwrap();
                let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
                assert_eq!(toolbar.size.height, px(40.));
                inside(toolbar, workspace);
                for control in [
                    "action-Guided(50)",
                    "action-Guided(54)",
                    "zoom-select-button",
                    "action-Phase5(36)",
                    "action-Guided(44)",
                ] {
                    inside(cx.debug_bounds(control).unwrap(), toolbar);
                }
                for name in [
                    "inspected-target-path",
                    "inspection-profile",
                    "inspection-before-environment",
                    "inspection-after-environment",
                ] {
                    inside(cx.debug_bounds(name).unwrap(), workspace);
                }
                shell.read_with(cx, |s, cx| {
                    assert_eq!((s.zoom, s.pan_x, s.pan_y), (0.75, 11., 12));
                    assert_eq!(
                        s.analysis_fields[9].read(cx).content.as_ref(),
                        "retained draft"
                    );
                    assert!(s.inspection_identity().1.contains("Historical check"));
                    assert!(!s.inspection_identity().1.contains("Today's unrelated"));
                });
            }
        }
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn view_menu_is_trigger_attached_full_row_and_native_keyboard_dismissable(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.bind_keys([
            KeyBinding::new("enter", Activate, Some("Shell")),
            KeyBinding::new("escape", Dismiss, Some("Shell")),
            KeyBinding::new("tab", Tab, Some("Shell")),
            KeyBinding::new("shift-tab", TabPrev, Some("Shell")),
        ])
    });
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.viewer_mode = 0;
                w.focus(&s.handle(Control::Guided(54), cx));
            })
        });
        cx.simulate_keystrokes("enter");
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        let trigger = cx.debug_bounds("action-Guided(54)").unwrap();
        let menu = cx.debug_bounds("viewer-mode-chooser").unwrap();
        assert_eq!(menu.top(), trigger.bottom() + px(4.));
        assert_eq!(menu.left(), trigger.left());
        inside(
            menu,
            Bounds::new(
                gpui::point(px(8.), px(8.)),
                size(px(width - 16.), px(height - 16.)),
            ),
        );
        for name in [
            "action-Phase5(6)",
            "action-Phase5(7)",
            "action-Phase5(8)",
            "action-Phase5(9)",
            "action-Guided(55)",
        ] {
            let row = cx.debug_bounds(name).unwrap();
            assert_eq!(row.size.width, menu.size.width - px(2.));
            inside(row, menu);
        }
        cx.simulate_keystrokes("end enter");
        shell.read_with(cx, |s, _| assert_eq!(s.viewer_mode, 4));
        cx.simulate_keystrokes("enter home down down enter");
        shell.read_with(cx, |s, _| assert_eq!(s.viewer_mode, 2));
        cx.simulate_keystrokes("enter up escape");
        cx.update(|w, cx| {
            shell.read_with(cx, |s, cx| {
                assert!(!s.viewer_mode_open);
                assert_eq!(s.viewer_mode, 2);
                assert!(s.handle(Control::Guided(54), cx).is_focused(w));
            })
        });
        cx.simulate_keystrokes("enter tab");
        cx.update(|w, cx| {
            shell.read_with(cx, |s, cx| {
                assert!(!s.viewer_mode_open);
                assert!(!s.handle(Control::Guided(54), cx).is_focused(w));
            })
        });
        cx.update(|w, cx| shell.update(cx, |s, cx| s.activate_control(Control::Guided(54), w, cx)));
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        cx.simulate_click(
            gpui::point(px(width - 12.), px(65.)),
            gpui::Modifiers::none(),
        );
        shell.read_with(cx, |s, _| assert!(!s.viewer_mode_open));
    }
    owned.join_after_ui();
}

#[gpui::test]
fn concise_blockers_are_painted_at_both_widths_without_relaxing_draft_guards(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for kind in [
            "note",
            "geometry",
            "area",
            "pending",
            "noncomparable",
            "exhausted",
        ] {
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.analysis_due = None;
                    s.mask_dirty = false;
                    s.mask_selected = None;
                    s.review_drafts = Default::default();
                    s.analysis_fields[9].update(cx, |f, cx| f.set_value("", cx));
                    load_review(s, project, cx);
                    match kind {
                        "note" => {
                            s.activate_control(Control::Phase5(37), w, cx);
                            s.analysis_fields[9]
                                .update(cx, |f, cx| f.set_value("unsaved draft", cx));
                            s.dismiss(&Dismiss, w, cx);
                        }
                        "geometry" | "area" => {
                            s.mask_rectangles = vec![regressee_app::analysis_engine::Rectangle {
                                x: 1,
                                y: 1,
                                width: 1,
                                height: 1,
                            }];
                            s.mask_selected = Some(0);
                            s.load_mask_fields(cx);
                            if kind == "geometry" {
                                s.analysis_fields[10].update(cx, |f, cx| f.set_value("2", cx));
                            } else {
                                s.mask_dirty = true;
                            }
                        }
                        "pending" => {
                            s.analysis_due =
                                Some(std::time::Instant::now() + Duration::from_secs(3600))
                        }
                        "noncomparable" => {
                            let key = s.selected_key.clone();
                            let r = s
                                .evidence
                                .comparison
                                .as_mut()
                                .unwrap()
                                .revision
                                .results
                                .iter_mut()
                                .find(|r| Some(&r.key) == key.as_ref())
                                .unwrap();
                            r.classification = Classification::NotComparable;
                            r.eligible_pixels = 0;
                        }
                        _ => {
                            s.crawl_fields[9]
                                .update(cx, |f, cx| f.set_value("nothing-matches", cx));
                            s.search = "nothing-matches".into();
                        }
                    }
                    assert!(s.review_blocking_reason(cx).is_some());
                    if matches!(kind, "note" | "geometry" | "area") {
                        let key = s.selected_key.clone();
                        assert!(!s.control_enabled(Control::Phase5(33), cx));
                        s.activate_control(Control::Phase5(33), w, cx);
                        assert_eq!(s.selected_key, key);
                    }
                    if matches!(kind, "area" | "pending" | "noncomparable") {
                        assert!(!s.control_enabled(Control::Phase5(21), cx));
                    }
                    cx.notify();
                })
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            inside(
                cx.debug_bounds("review-blocking-reason").unwrap(),
                cx.debug_bounds("ordinary-review-decisions").unwrap(),
            );
        }
    }
    shell.update(cx, |s, _| {
        s.analysis_due = None;
    });
    owned.join_after_ui();
}

#[test]
fn selection_copy_names_ready_members_of_the_selection() {
    assert_eq!(
        evidence_workspace::baseline_selection_label(0, 0),
        "0 selected"
    );
    assert_eq!(
        evidence_workspace::baseline_selection_label(3, 2),
        "3 selected · 2 ready to update"
    );
}
