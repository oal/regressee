// SPDX-License-Identifier: MPL-2.0
//! Viewer controls remain source-synchronized; selectors cannot expose mutations.
use super::*;
use crate::workflow_refinement_tests::load_review;
use gpui::TestAppContext;

#[gpui::test]
fn zoom_occlusion_and_image_keyboard_preserve_source_pan_across_views(cx: &mut TestAppContext) {
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
    shell.update(cx, |s, cx| load_review(s, project, cx));
    cx.simulate_resize(size(px(820.), px(600.)));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.navigation_preference = Some(false);
            s.configure_review_geometry(820.);
            s.approval_selection = s.selected_key.iter().cloned().collect();
            assert!(s.control_enabled(Control::Phase5(24), cx));
            s.activate_control(Control::Guided(46), w, cx);
            let requests = s.next_request;
            for control in [
                Control::Phase5(21),
                Control::Phase5(23),
                Control::Phase5(24),
                Control::Phase5(33),
            ] {
                assert!(!s.control_enabled(control, cx));
                s.activate_control(control, w, cx);
                assert_eq!(s.next_request, requests);
            }
        })
    });
    cx.simulate_keystrokes("down enter"); // Actual-size selection, not shell approval.
    shell.read_with(cx, |s, cx| {
        assert_eq!(s.zoom, 1.);
        assert!(!s.viewer_fit);
        assert!(s.control_enabled(Control::Phase5(24), cx));
    });
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.choose_viewer_zoom(ZoomChoice::Percent(400), cx);
            w.focus(&s.handle(Control::Viewer, cx));
        })
    });
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
    cx.simulate_keystrokes("right down");
    let pan = shell.read_with(cx, |s, _| {
        assert!(s.pan_x > 0. && s.pan_y > 0);
        (s.zoom, s.pan_x, s.pan_y)
    });
    for mode in [0, 1, 2, 3, 5, 4] {
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Guided(54), w, cx);
                s.activate_control(
                    match mode {
                        4 => Control::Guided(55),
                        5 => Control::SplitView,
                        mode => Control::Phase5(mode + 6),
                    },
                    w,
                    cx,
                );
                assert_eq!((s.zoom, s.pan_x, s.pan_y), pan);
                let transform = s.viewer_transform();
                assert_eq!(
                    transform.source_to_viewport((11., 12.)),
                    ((11. - pan.1) * pan.0, (12. - pan.2 as f32) * pan.0)
                );
            })
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
    }
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.activate_control(Control::Guided(56), w, cx);
            assert_eq!((s.zoom, s.pan_x, s.pan_y), pan);
            s.viewer = None;
            s.review_queue_collapsed = true;
            cx.notify();
        })
    });
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
    assert!(cx.debug_bounds("action-Guided(50)").is_some());
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.activate_control(Control::Guided(50), w, cx);
            assert!(!s.review_queue_collapsed);
        })
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
