// SPDX-License-Identifier: MPL-2.0
//! Finite app-wide scrollbar inspection. Offline synthetic evidence only.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

#[gpui::test]
#[ignore = "explicit three-size app-wide scrollbar inspection"]
fn renderer_capture_scrollbars(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("scrollbars-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), false);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.error = None;
            s.notice = None;
            s.navigation_preference = Some(false);
            s.configure_review_geometry(width);
            cx.notify();
        });
        crate::phase4_hierarchy_tests::draw(cx);
        let key = shell.read_with(cx, |s, _| s.selected_key.clone());
        let queue = shell.read_with(cx, |s, _| {
            s.preview_list_scroll.0.borrow().base_handle.clone()
        });
        assert!(queue.max_offset().height > px(0.));
        capture(
            cx,
            &root,
            &format!("scrollbars-{width:.0}x{height:.0}-review"),
        )
        .unwrap();
        let b = queue.bounds();
        let bar = gpui::point(b.right() - px(8.), b.top() + px(40.));
        cx.simulate_mouse_move(bar, None, gpui::Modifiers::none());
        capture(
            cx,
            &root,
            &format!("scrollbars-{width:.0}x{height:.0}-hover"),
        )
        .unwrap();
        cx.simulate_click(
            gpui::point(bar.x, b.bottom() - px(24.)),
            gpui::Modifiers::none(),
        );
        crate::phase4_hierarchy_tests::draw(cx);
        shell.read_with(cx, |s, _| {
            assert!(queue.offset().y < px(0.));
            assert_eq!((s.pan_x, s.pan_y), (0., 0));
            assert_eq!(s.selected_key, key);
            assert!(s.pan_drag.is_none() && s.mask_drag.is_none());
        });
        capture(
            cx,
            &root,
            &format!("scrollbars-{width:.0}x{height:.0}-scrolled"),
        )
        .unwrap();
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Guided(44), w, cx);
                s.capture_diagnostics_open = true;
                cx.notify();
            })
        });
        capture(
            cx,
            &root,
            &format!("scrollbars-{width:.0}x{height:.0}-diagnostics"),
        )
        .unwrap();
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Phase5(39), w, cx);
                s.activate_control(Control::Phase5(37), w, cx);
                s.analysis_fields[9].update(cx, |f, cx| {
                    f.set_value(
                        &"Review this line without losing the retained draft.\n".repeat(30),
                        cx,
                    )
                });
                cx.notify();
            })
        });
        capture(
            cx,
            &root,
            &format!("scrollbars-{width:.0}x{height:.0}-note"),
        )
        .unwrap();
        shell.update(cx, |s, cx| {
            s.review_note_open = false;
            s.analysis_fields[9].update(cx, |f, cx| f.set_value("", cx));
            s.focus.screen = Screen::History;
            s.queue(Work::History { project, page: 0 }, cx);
            finish_jobs(s, cx);
            // Repeat loaded synthetic records only in this test view, to inspect
            // the long-list state without manufacturing saved production history.
            let h = s.history.as_mut().unwrap();
            let rows = h.runs.clone();
            h.runs.extend(rows.clone());
            h.runs.extend(rows);
            s.history_section = 0;
            cx.notify();
        });
        crate::phase4_hierarchy_tests::draw(cx);
        shell.update(cx, |s, cx| {
            let b = s.history_run_scroll.bounds();
            let clip = s.screen_scroll.bounds();
            let delta = (b.bottom() - clip.bottom()).max(px(0.));
            s.screen_scroll.set_offset(gpui::point(px(0.), -delta));
            cx.notify();
        });
        capture(
            cx,
            &root,
            &format!("scrollbars-{width:.0}x{height:.0}-history"),
        )
        .unwrap();
        shell.update(cx, |s, cx| {
            s.focus.screen = Screen::Settings;
            s.details.insert(Detail::ManagedNetwork);
            s.details.insert(Detail::SupportTools);
            s.runtime_readiness = Some(crate::app_settings_tests::readiness(
                regressee_app::product::RuntimeState::Ready,
                true,
            ));
            s.runtime_status = Some("Synthetic verified browser status.".into());
            s.summary = Some(store.summary().unwrap());
            s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
            cx.notify();
        });
        capture(
            cx,
            &root,
            &format!("scrollbars-{width:.0}x{height:.0}-settings"),
        )
        .unwrap();
        shell.update(cx, |s, cx| finish_jobs(s, cx));
        owned.join_after_ui();
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
