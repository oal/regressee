// SPDX-License-Identifier: MPL-2.0
//! Native-shell simulated input/layout coverage; never an OS-input qualification.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{TestAppContext, VisualTestContext};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}

pub(super) fn exercise(
    shell: &Entity<Shell>,
    cx: &mut VisualTestContext,
    mut inspect: impl FnMut(&str, &mut VisualTestContext),
) {
    draw(cx);
    let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
    let old_height = cx.debug_bounds("phase5-before").unwrap().size.height;
    let original = shell.read_with(cx, |s, cx| {
        (
            s.selected_key.clone(),
            s.next_request,
            s.analysis_fields[9].read(cx).content.to_string(),
            s.mask_rectangles.clone(),
        )
    });
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| w.focus(&s.handle(Control::Guided(54), cx)));
    });
    cx.simulate_keystrokes("enter end");
    draw(cx);
    inspect("menu", cx);
    cx.simulate_keystrokes("enter");
    draw(cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.viewer_mode, 5);
        assert_eq!(s.viewer_pane_indices(), &[0]);
        assert_eq!(s.split_fraction, 0.5);
    });
    assert_eq!(cx.debug_bounds("viewer-toolbar").unwrap(), toolbar);
    assert_eq!(
        cx.debug_bounds("phase5-before").unwrap().size.height,
        old_height
    );
    let bounds = cx.debug_bounds("split-divider").unwrap();
    assert_eq!(
        cx.debug_bounds("phase5-tile-0-0-0"),
        cx.debug_bounds("phase5-tile-0-1-0")
    );
    inspect("center", cx);
    let navigation = shell.read_with(cx, |s, _| (s.zoom, s.pan_x, s.pan_y, s.viewer_fit));
    let pointer = |fraction: f32| {
        gpui::point(
            bounds.left() + bounds.size.width * fraction,
            bounds.center().y,
        )
    };
    cx.simulate_mouse_down(bounds.center(), MouseButton::Left, gpui::Modifiers::none());
    shell.read_with(cx, |s, _| {
        assert!(s.split_drag.is_some());
        assert!(s.pan_drag.is_none());
    });
    for (fraction, label) in [(0.25, "left"), (0.75, "right")] {
        cx.simulate_mouse_move(
            pointer(fraction),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        draw(cx);
        shell.read_with(cx, |s, _| {
            assert!((s.split_fraction - fraction).abs() < 0.001);
            assert_eq!((s.zoom, s.pan_x, s.pan_y, s.viewer_fit), navigation);
        });
        let reveal = cx.debug_bounds("split-before-reveal").unwrap();
        assert!((reveal.size.width / bounds.size.width - fraction).abs() < 0.01);
        assert_eq!(
            cx.debug_bounds("phase5-tile-0-0-0"),
            cx.debug_bounds("phase5-tile-0-1-0")
        );
        inspect(label, cx);
    }
    // Capture-phase movement/release must work beyond the pane, not just over
    // the narrow moving target, and must never click an underlying action.
    cx.simulate_mouse_move(pointer(1.1), MouseButton::Left, gpui::Modifiers::none());
    draw(cx);
    shell.read_with(cx, |s, _| assert_eq!(s.split_fraction, 1.));
    cx.simulate_mouse_up(pointer(-0.1), MouseButton::Left, gpui::Modifiers::none());
    draw(cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.split_fraction, 0.);
        assert!(s.split_drag.is_none() && s.pan_drag.is_none());
    });
    // Both extreme handles remain inside the image and can be grabbed again.
    cx.simulate_mouse_down(
        pointer(0.) + gpui::point(px(20.), px(0.)),
        MouseButton::Left,
        gpui::Modifiers::none(),
    );
    shell.read_with(cx, |s, _| assert!(s.split_drag.is_some()));
    cx.simulate_mouse_up(
        pointer(0.) + gpui::point(px(20.), px(0.)),
        MouseButton::Left,
        gpui::Modifiers::none(),
    );
    draw(cx);
    cx.simulate_keystrokes("home right shift-right");
    shell.read_with(cx, |s, _| assert!((s.split_fraction - 0.11).abs() < 0.0001));
    cx.simulate_keystrokes("end right");
    shell.read_with(cx, |s, _| assert_eq!(s.split_fraction, 1.));
    cx.simulate_keystrokes("home left");
    shell.read_with(cx, |s, _| assert_eq!(s.split_fraction, 0.));
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| w.focus(&s.handle(Control::Viewer, cx)));
    });
    cx.simulate_keystrokes("tab");
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(s.handle(Control::SplitDivider, cx).is_focused(w))
        });
    });
    cx.simulate_keystrokes("home shift-right shift-right shift-right shift-right shift-right");
    draw(cx);
    inspect("keyboard", cx);
    shell.read_with(cx, |s, cx| {
        assert!((s.split_fraction - 0.5).abs() < 0.0001);
        assert_eq!((s.zoom, s.pan_x, s.pan_y, s.viewer_fit), navigation);
        assert_eq!(
            (
                s.selected_key.clone(),
                s.next_request,
                s.analysis_fields[9].read(cx).content.to_string(),
                s.mask_rectangles.clone()
            ),
            original
        );
    });
    // Away from the divider the incumbent image-drag path remains the owner.
    cx.simulate_mouse_down(pointer(0.2), MouseButton::Left, gpui::Modifiers::none());
    shell.read_with(cx, |s, _| {
        assert!(s.pan_drag.is_some());
        assert!(s.split_drag.is_none());
    });
    cx.simulate_mouse_up(pointer(0.2), MouseButton::Left, gpui::Modifiers::none());
    shell.update(cx, finish_jobs);
}

#[gpui::test]
fn split_view_drag_keyboard_alignment_and_drafts(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
        let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            assert_eq!(s.viewer_mode, 0, "Compare remains the default");
            s.analysis_fields[9].update(cx, |f, cx| f.set_value("Keep my unsaved note", cx));
            s.mask_dirty = true;
        });
        exercise(&shell, cx, |_, _| {});
        owned.join_after_ui();
        let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
        assert_eq!(
            frozen,
            serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
        );
    }
}

#[gpui::test]
fn split_view_missing_pairs_and_occlusion_fail_closed(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    draw(cx);
    shell.update(cx, |s, cx| {
        s.viewer_mode = 5;
        let frame = s.viewer.clone().unwrap();
        for source in 0..2 {
            s.viewer.as_mut().unwrap().panes[source].clear();
            assert!(!s.split_view_available());
            assert_eq!(s.viewer_pane_indices(), &[0, 1]);
            assert!(!s.control_enabled(Control::SplitDivider, cx));
            s.viewer_mode_open = true;
            assert!(!s.control_enabled(Control::SplitView, cx));
            s.viewer_mode_open = false;
            s.viewer = Some(frame.clone());
        }
        // A shorter original's blank tail must not disable the split or leak
        // the other version's pixels through its clipped, opaque reveal.
        let view = s.viewer.as_mut().unwrap();
        view.tile = 1;
        view.request.as_mut().unwrap().before.dimensions.height = 512;
        view.panes[0].clear();
        assert!(s.split_view_available());
        assert!(s.viewer_has_pixels());
        assert!(s.control_enabled(Control::SplitDivider, cx));
        s.viewer = Some(frame);
        for blocked in 0..6 {
            s.image_controls_open = blocked == 0;
            s.focus.busy = blocked == 1;
            s.focus.dialog = blocked == 2;
            s.native_prompt = blocked == 3;
            s.mask_draw = blocked == 4;
            s.viewer_mode_open = blocked == 5;
            assert!(!s.control_enabled(Control::SplitDivider, cx));
            assert!(!s.split_input_allowed());
        }
        s.viewer_mode_open = false;
        s.evidence.comparison = None;
        assert_eq!(s.viewer_pane_indices(), &[1]);
        assert!(!s.control_mounted(Control::SplitView));
        assert!(!s.control_mounted(Control::SplitDivider));
    });
    owned.join_after_ui();
}
