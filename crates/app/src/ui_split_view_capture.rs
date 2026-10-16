// SPDX-License-Identifier: MPL-2.0
//! Two-size actual GPUI offscreen capture using local synthetic website originals.
use super::*;
use crate::workflow_refinement_tests::load_review;

#[gpui::test]
#[ignore = "explicit native offscreen split-slider inspection"]
fn renderer_capture_split_view(cx: &mut TestAppContext) {
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
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("split-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), true);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| load_review(s, project, cx));
        crate::split_view_tests::exercise(&shell, cx, |state, cx| {
            let label = format!("split-{state}-{width:.0}x{height:.0}");
            capture(cx, &root, &label).unwrap();
            manifest.push(label);
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Guided(50), w, cx);
                s.choose_viewer_zoom(ZoomChoice::Percent(100), cx);
                s.pan_x = 120.;
                s.pan_y = 160;
                s.refresh_viewer_navigation(cx);
            });
        });
        let label = format!("split-zoomed-hidden-results-{width:.0}x{height:.0}");
        capture(cx, &root, &label).unwrap();
        manifest.push(label);
        owned.join_after_ui();
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("split-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
