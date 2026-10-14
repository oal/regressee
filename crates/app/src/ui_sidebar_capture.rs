// SPDX-License-Identifier: MPL-2.0
//! Two-size sidebar inspection; isolated synthetic evidence, no website visits.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

#[gpui::test]
#[ignore = "explicit two-size native offscreen sidebar inspection"]
fn renderer_capture_sidebar(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    let dir = tempfile::Builder::new()
        .prefix("sidebar-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let mut logo = None;
        for scene in [
            "review-picker",
            "review-open",
            "review-collapsed",
            "change-project",
        ] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    if scene == "review-picker" {
                        finish_jobs(s, cx);
                        s.focus.screen = Screen::Results;
                    } else {
                        load_review(s, project, cx);
                    }
                    s.notice = None;
                    s.error = None;
                    s.navigation_preference =
                        Some(matches!(scene, "review-collapsed" | "change-project"));
                    s.configure_review_geometry(width);
                    if scene == "change-project" {
                        s.activate_control(Control::ChooseProject, w, cx);
                        finish_jobs(s, cx);
                    }
                    cx.notify();
                });
                let _ = w.draw(cx);
            });
            cx.run_until_parked();
            cx.update(|w, cx| {
                if scene != "review-collapsed" {
                    assert!(
                        brand::logo(Background::Dark)
                            .get_render_image(w, cx)
                            .is_some()
                    );
                }
                let _ = w.draw(cx);
            });
            if scene != "review-collapsed" {
                let bounds = cx.debug_bounds("brand-lockup").unwrap();
                assert_eq!(*logo.get_or_insert(bounds), bounds);
                assert!(bounds.top() < px(40.));
            }
            let toggle = cx.debug_bounds("action-Guided(53)");
            assert_eq!(
                toggle.is_some(),
                matches!(scene, "review-open" | "review-collapsed")
            );
            if let Some(toggle) = toggle {
                crate::phase1_setup_tests::inside(
                    toggle,
                    cx.debug_bounds("project-header").unwrap(),
                );
            }
            capture(
                cx,
                &root,
                &format!("sidebar-{width:.0}x{height:.0}-{scene}"),
            )
            .unwrap();
            owned.join_after_ui();
        }
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
