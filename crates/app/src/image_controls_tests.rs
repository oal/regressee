// SPDX-License-Identifier: MPL-2.0
//! Explicit viewer help uses mounted native controls, never screenshot hover.
use super::*;
use crate::{phase2_evidence_tests::inside, workflow_refinement_tests::load_review};
use gpui::{TestAppContext, VisualTestContext};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}

pub(super) fn bind(cx: &mut TestAppContext) {
    cx.update(|cx| {
        zoom_select::bind(cx);
        cx.bind_keys([
            KeyBinding::new("enter", Activate, Some("Shell")),
            KeyBinding::new("escape", Dismiss, Some("Shell")),
            KeyBinding::new("tab", Tab, Some("Shell")),
            KeyBinding::new("shift-tab", TabPrev, Some("Shell")),
        ]);
    });
}

pub(super) fn exercise(
    shell: &Entity<Shell>,
    cx: &mut VisualTestContext,
    width: f32,
    mut inspect: impl FnMut(&str, &mut VisualTestContext),
) {
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            if s.control_enabled(Control::DismissFeedback, cx) {
                s.activate_control(Control::DismissFeedback, w, cx);
            }
            s.configure_review_geometry(width);
        });
    });
    draw(cx);
    let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
    assert_eq!(toolbar.size.height, px(CONTROL_HEIGHT));
    let pane = cx.debug_bounds("phase5-after").unwrap();
    let pane_height = pane.size.height;
    let before = shell.read_with(cx, |s, cx| {
        (
            s.selected_key.clone(),
            s.zoom,
            s.pan_x,
            s.pan_y,
            s.viewer_fit,
            s.analysis_fields[9].read(cx).content.to_string(),
            s.mask_rectangles.clone(),
        )
    });
    let requests = shell.read_with(cx, |s, _| s.next_request);
    let retained = |cx: &mut VisualTestContext| {
        shell.read_with(cx, |s, cx| {
            assert_eq!(
                (
                    s.selected_key.clone(),
                    s.zoom,
                    s.pan_x,
                    s.pan_y,
                    s.viewer_fit,
                    s.analysis_fields[9].read(cx).content.to_string(),
                    s.mask_rectangles.clone(),
                ),
                before,
                "Opening/closing help must not change evidence, zoom or drafts"
            );
        });
    };
    // Hovering the image must not open help or alter the evidence.
    cx.simulate_mouse_move(pane.center(), None, gpui::Modifiers::none());
    draw(cx);
    shell.read_with(cx, |s, _| assert!(!s.image_controls_open));
    inspect("evidence", cx);
    if width >= 1100. {
        let button = cx.debug_bounds("action-ImageControls").unwrap();
        inside(button, toolbar);
        cx.simulate_click(button.center(), gpui::Modifiers::none());
    } else {
        shell.read_with(cx, |s, cx| {
            assert!(!s.control_mounted(Control::ImageControls));
            assert!(!s.control_enabled(Control::ImageControls, cx));
        });
        cx.update(|w, cx| {
            shell.read_with(cx, |s, cx| w.focus(&s.handle(Control::Guided(46), cx)));
        });
        cx.simulate_keystrokes("enter end");
        draw(cx);
        let popup = cx.debug_bounds("zoom-popup").unwrap();
        inside(cx.debug_bounds("zoom-image-controls").unwrap(), popup);
        inspect("zoom-menu", cx);
        cx.simulate_keystrokes("enter");
    }
    draw(cx);
    shell.read_with(cx, |s, cx| {
        assert!(s.image_controls_open && s.viewer_inspector_open(),
            "help must open at width {width}: focus={:?}, pixels={}, trigger={}, last={:?}, comparison={}",
            s.focus, s.viewer_has_pixels(), s.control_enabled(s.image_controls_trigger(), cx), s.last_action, s.evidence.comparison.is_some());
        assert!(!s.zoom_select.read(cx).is_open());
        assert!(!s.capture_details_open && !s.review_tools && !s.review_note_open);
        assert_eq!(s.next_request, requests, "Opening help must not queue work");
        for control in [
            Control::Phase5(21),
            Control::Phase5(23),
            Control::Phase5(24),
        ] {
            assert!(!s.control_enabled(control, cx));
        }
    });
    let inspector = cx.debug_bounds("viewer-inspector").unwrap();
    inside(inspector, cx.debug_bounds("review-viewer").unwrap());
    inside(cx.debug_bounds("action-Phase5(39)").unwrap(), inspector);
    inside(
        cx.debug_bounds("image-controls-help").unwrap(),
        Bounds::new(inspector.origin, size(inspector.size.width, px(2000.))),
    );
    assert!(cx.debug_bounds("phase5-after").unwrap().size.height >= pane_height);
    assert_eq!(cx.debug_bounds("viewer-toolbar").unwrap(), toolbar);
    retained(cx);
    inspect("help", cx);
    cx.simulate_keystrokes("escape");
    draw(cx);
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(!s.image_controls_open);
            assert!(s.handle(s.image_controls_trigger(), cx).is_focused(w));
        });
    });
    assert_eq!(
        cx.debug_bounds("phase5-after").unwrap().size.height,
        pane_height
    );
    retained(cx);

    // Both the visible Close action and an outside image click work without
    // moving the pointer off the screenshot to dismiss a hover popup.
    cx.simulate_keystrokes(if width >= 1100. {
        "enter"
    } else {
        "enter end enter"
    });
    draw(cx);
    let close = cx.debug_bounds("action-Phase5(39)").unwrap();
    cx.simulate_click(close.center(), gpui::Modifiers::none());
    draw(cx);
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(!s.image_controls_open);
            assert!(s.handle(s.image_controls_trigger(), cx).is_focused(w));
        })
    });
    retained(cx);
    shell.read_with(cx, |s, _| assert_eq!(s.next_request, requests));
    cx.simulate_keystrokes(if width >= 1100. {
        "enter"
    } else {
        "enter end enter"
    });
    draw(cx);
    let first_pane = shell.read_with(cx, |s, _| s.viewer_pane_indices()[0]);
    let pane = cx
        .debug_bounds(if first_pane == 0 {
            "phase5-before"
        } else {
            "phase5-after"
        })
        .unwrap();
    cx.simulate_click(
        pane.origin + gpui::point(px(3.), px(30.)),
        gpui::Modifiers::none(),
    );
    draw(cx);
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(!s.image_controls_open);
            assert!(s.handle(Control::Viewer, cx).is_focused(w));
        })
    });
    // A real image click follows the incumbent viewer's tile-read path.
    shell.update(cx, crate::workflow_refinement_tests::finish_jobs);
    retained(cx);
}

#[gpui::test]
fn image_controls_are_explicit_responsive_and_preserve_evidence(cx: &mut TestAppContext) {
    bind(cx);
    for comparison in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let project =
            crate::workflow_refinement_tests::screenshot_switch_fixture(dir.path(), comparison);
        let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
        let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            s.mask_dirty = false;
            load_review(s, project, cx);
            assert_eq!(s.evidence.comparison.is_some(), comparison);
            s.analysis_fields[9].update(cx, |f, cx| {
                f.set_value(
                    if comparison {
                        "Preserve my review draft"
                    } else {
                        ""
                    },
                    cx,
                )
            });
            if comparison {
                s.mask_rectangles = vec![regressee_app::analysis_engine::Rectangle {
                    x: 0,
                    y: 0,
                    width: 1,
                    height: 1,
                }];
            }
            s.mask_dirty = comparison;
        });
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            exercise(&shell, cx, width, |_, _| {});
        }
        owned.join_after_ui();
        let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
        assert_eq!(
            original,
            serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
        );
    }
}

#[gpui::test]
fn image_controls_share_disclosure_lifecycle_and_restore_after_resize(cx: &mut TestAppContext) {
    bind(cx);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    cx.simulate_resize(size(px(1280.), px(800.)));
    draw(cx);
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.activate_control(Control::Phase5(36), w, cx);
            s.activate_control(Control::ImageControls, w, cx);
            assert!(s.image_controls_open && !s.capture_details_open);
            s.activate_control(Control::Guided(44), w, cx);
            assert!(s.review_tools && !s.image_controls_open);
            s.activate_control(Control::ImageControls, w, cx);
            assert!(s.image_controls_open && !s.review_tools);
        })
    });
    draw(cx);
    cx.simulate_resize(size(px(820.), px(600.)));
    draw(cx);
    cx.simulate_keystrokes("escape");
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(!s.image_controls_open);
            assert!(s.handle(Control::Guided(46), cx).is_focused(w));
        })
    });
    cx.simulate_keystrokes("enter end enter");
    draw(cx);
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.navigate(Screen::History, w, cx);
            assert!(!s.image_controls_open);
            s.focus.screen = Screen::Results;
            s.viewer = None;
            assert!(!s.control_mounted(Control::ImageControls));
            assert!(!s.control_enabled(Control::Guided(46), cx));
        })
    });
    owned.join_after_ui();
}
