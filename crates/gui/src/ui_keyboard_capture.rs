// SPDX-License-Identifier: MPL-2.0
//! Bounded native offscreen focus proof; no OS-input or AT certification.
use super::*;
use crate::{
    keyboard_navigation_tests::{bind, press},
    phase4_hierarchy_tests::draw,
    workflow_refinement_tests::finish_jobs,
};

#[gpui::test]
#[ignore = "explicit two-size focus-ring rendering and full key-release inspection"]
fn renderer_capture_keyboard_focus(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    cx.update(bind);
    let dir = tempfile::tempdir().unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_jobs(s, cx);
                s.focus.screen = Screen::Settings;
                s.refresh_visible(cx);
                finish_jobs(s, cx);
                s.runtime_readiness = Some(app_settings_tests::readiness(
                    regressee_app::product::RuntimeState::Ready,
                    true,
                ));
                s.details.clear();
                s.notice = None;
                s.error = None;
                window.focus(&s.handle(Control::SetupRuntime, cx));
                cx.notify();
            });
        });
        draw(cx);
        capture(
            cx,
            &root,
            &format!("keyboard-{width:.0}x{height:.0}-action"),
        )
        .unwrap();
        assert_ring(
            cx,
            "action-SetupRuntime",
            palette().accent,
            palette().surface,
        );
        press(cx, "tab");
        press(cx, "enter");
        shell.read_with(cx, |s, _| assert!(s.details.contains(&Detail::Browser)));
        let header = cx.debug_bounds("action-Details(Browser)").unwrap();
        cx.simulate_mouse_move(header.center(), None, gpui::Modifiers::none());
        draw(cx);
        capture(
            cx,
            &root,
            &format!("keyboard-{width:.0}x{height:.0}-disclosure-hover"),
        )
        .unwrap();
        assert_ring(
            cx,
            "action-Details(Browser)",
            palette().accent,
            palette().surface,
        );
        cx.update(|window, cx| {
            shell.read_with(cx, |s, cx| {
                window.focus(&s.handle(Control::Navigate(Screen::Settings), cx))
            });
        });
        draw(cx);
        capture(
            cx,
            &root,
            &format!("keyboard-{width:.0}x{height:.0}-selected-navigation"),
        )
        .unwrap();
        assert_ring(
            cx,
            "action-Navigate(Settings)",
            palette().rail_ink,
            palette().rail,
        );
        owned.join_after_ui();
    }
}

#[gpui::test]
#[ignore = "explicit three-size viewer focus inspection"]
fn renderer_capture_viewer_focus(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("focus-synthetic-")
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
            crate::workflow_refinement_tests::load_review(s, project, cx);
            s.navigation_preference = Some(false);
            s.configure_review_geometry(width);
            s.notice = None;
            s.error = None;
        });
        draw(cx);
        for (scene, control, selector) in [
            ("compare", Control::Guided(54), "action-Guided(54)"),
            ("zoom", Control::Guided(46), "zoom-select-button"),
            ("selected-choice", Control::Phase5(20), "action-Phase5(20)"),
            ("primary", Control::Phase5(33), "action-Phase5(33)"),
        ] {
            let before = cx.debug_bounds(selector).unwrap();
            cx.update(|window, cx| {
                shell.read_with(cx, |s, cx| {
                    assert!(s.control_enabled(control, cx));
                    window.focus(&s.handle(control, cx));
                });
            });
            draw(cx);
            assert_eq!(cx.debug_bounds(selector).unwrap(), before);
            capture(cx, &root, &format!("focus-{width:.0}x{height:.0}-{scene}")).unwrap();
            assert_ring(cx, selector, palette().accent, palette().surface);
        }
        owned.join_after_ui();
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

fn assert_ring(cx: &mut VisualTestContext, selector: &'static str, outer: u32, gap: u32) {
    let bounds = cx.debug_bounds(selector).unwrap();
    let frame = cx.update(|window, cx| {
        let _ = window.draw(cx);
        window.capture_offscreen_frame().unwrap()
    });
    let x = f32::from(bounds.center().x) as usize;
    let top = f32::from(bounds.top()) as usize;
    for (y, color) in [(top - 3, outer), (top - 2, outer), (top - 1, gap)] {
        let offset = (y * frame.width as usize + x) * 4;
        let expected = [(color >> 16) as u8, (color >> 8) as u8, color as u8, 255];
        assert_eq!(
            &frame.rgba[offset..offset + 4],
            &expected,
            "{selector}: focus paint at ({x}, {y})"
        );
    }
    let offset = ((top - 4) * frame.width as usize + x) * 4;
    let old_ring = [(outer >> 16) as u8, (outer >> 8) as u8, outer as u8, 255];
    assert_ne!(
        &frame.rgba[offset..offset + 4],
        &old_ring,
        "{selector}: focus must not extend beyond 3px"
    );
}
