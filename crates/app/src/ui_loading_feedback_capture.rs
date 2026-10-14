// SPDX-License-Identifier: MPL-2.0
//! Actual GPUI/Blade loading states at both desktop sizes; synthetic, offline.
use super::*;
use crate::loading_feedback_tests::{SCENES, prepare_scene};
use gpui::point;

#[gpui::test]
#[ignore = "explicit two-size native offscreen loading feedback inspection"]
fn renderer_capture_loading_feedback(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    cx.update(|cx| cx.set_global(controls::ReducedMotion(true)));
    let dir = tempfile::Builder::new()
        .prefix("loading-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for scene in SCENES {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| prepare_scene(s, scene, project, cx));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            if scene == "storage-read" {
                shell.update(cx, |s, _| {
                    s.screen_scroll.set_offset(point(px(0.), px(-400.)))
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
            }
            cx.run_until_parked();
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let icon = cx.debug_bounds("activity-indicator").unwrap();
            crate::phase1_setup_tests::inside(
                icon,
                Bounds::new(point(px(0.), px(0.)), size(px(width), px(height))),
            );
            if matches!(scene, "opening-project" | "saving" | "long-operation") {
                let surface = cx.debug_bounds("activity-surface").unwrap();
                crate::phase1_setup_tests::inside(
                    cx.debug_bounds("activity-label").unwrap(),
                    surface,
                );
                crate::phase1_setup_tests::inside(icon, surface);
            }
            let label = format!("loading-{width:.0}x{height:.0}-{scene}");
            capture(cx, &root, &label).unwrap();
            manifest.push(label);
            owned.join_after_ui();
        }
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("loading-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
