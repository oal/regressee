// SPDX-License-Identifier: MPL-2.0
//! Bounded two-size image-help capture. Offline synthetic screenshots only.
use super::*;
use crate::{image_controls_tests, workflow_refinement_tests::load_review};

#[gpui::test]
#[ignore = "explicit two-size native offscreen image-controls inspection"]
fn renderer_capture_image_controls(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    let reviewed: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("proof-reviewed.json")).unwrap()).unwrap();
    assert_eq!(reviewed["personally_reviewed"], true);
    assert_eq!(
        reviewed["png_sha256"],
        hex::encode(Sha256::digest(
            fs::read(root.join("renderer-proof.png")).unwrap()
        ))
    );
    cx.enable_offscreen_renderer().unwrap();
    image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("image-help-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), true);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = vec![];
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for comparison in [false, true] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                if !comparison {
                    ui_review_matrix::capture_only(s, &store, "ready");
                }
                s.navigation_preference = Some(width < 1100.);
                s.configure_review_geometry(width);
            });
            image_controls_tests::exercise(&shell, cx, width, |state, cx| {
                let label = format!(
                    "image-controls-{}-{state}-{width:.0}x{height:.0}",
                    if comparison { "compare" } else { "capture" }
                );
                capture(cx, &root, &label).unwrap();
                manifest.push(label);
            });
            // Inspect the keyboard rows at the end of the independent scroller,
            // not just the initial viewport. The Close action stays attached.
            shell.update(cx, |s, cx| {
                if !comparison {
                    s.evidence = store.project_evidence(project).unwrap();
                    ui_review_matrix::capture_only(s, &store, "ready");
                }
                s.open_image_controls(cx);
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            shell.update(cx, |s, _| {
                s.review_details_scroll.set_offset(gpui::point(
                    px(0.),
                    -s.review_details_scroll.max_offset().height,
                ));
            });
            let label = format!(
                "image-controls-{}-help-tail-{width:.0}x{height:.0}",
                if comparison { "compare" } else { "capture" }
            );
            capture(cx, &root, &label).unwrap();
            manifest.push(label);
            owned.join_after_ui();
        }
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("image-controls-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
