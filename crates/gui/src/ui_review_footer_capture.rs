// SPDX-License-Identifier: MPL-2.0
//! Finite native offscreen inspection of the review footer. All data is synthetic.
use super::*;
use crate::workflow_refinement_tests::load_review;

#[gpui::test]
#[ignore = "explicit three-size state-aware review footer inspection"]
fn renderer_capture_review_footer(cx: &mut TestAppContext) {
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
        .prefix("review-footer-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), false);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = Vec::new();
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| load_review(s, project, cx));
        crate::review_footer_tests::exercise(&shell, cx, |state, cx| {
            let label = format!("review-footer-{state}-{width:.0}x{height:.0}");
            capture(cx, &root, &label).unwrap();
            manifest.push(label);
        });
        owned.join_after_ui();
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("review-footer-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

#[gpui::test]
#[ignore = "explicit reproduction of capture-warning reasons and clicked-viewer focus"]
fn renderer_capture_comparison_recovery(cx: &mut TestAppContext) {
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
        .prefix("warning-recovery-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (mut store, project, _) = ui_review_matrix::fixture_with_capture_warnings(
        &root,
        dir.path(),
        false,
        &crate::comparison_ux_tests::WARNINGS,
    );
    // The user's already-saved check was vetoed by an older app, but retained
    // complete valid pixel reports. Persist that exact legacy summary shape.
    let mut legacy = store.project_evidence(project).unwrap().comparison.unwrap();
    legacy.revision.id = regressee_core::domain::AnalysisId::new();
    legacy.perceptual.as_mut().unwrap().cache_key =
        regressee_core::evidence::identity_hash(&("legacy-warning-veto", legacy.revision.id))
            .unwrap();
    for result in &mut legacy.revision.results {
        result.classification = Classification::NotComparable;
    }
    store.save_analysis(&legacy).unwrap();
    store.display_analysis(legacy.revision.id).unwrap();
    let mut initial = store.project_evidence(project).unwrap();
    initial.reviews.clear();
    let frozen = serde_json::to_value(initial).unwrap();
    let mut manifest = Vec::new();
    let mut probes = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| load_review(s, project, cx));
        crate::comparison_ux_tests::exercise_warning_recovery(&shell, cx, |state, cx| {
            let label = format!("comparison-recovery-{state}-{width:.0}x{height:.0}");
            capture(cx, &root, &label).unwrap();
            if matches!(state, "inline" | "pointer-focus" | "keyboard-focus") {
                let mut bounds = |name: &'static str| {
                    let b = cx.debug_bounds(name).unwrap();
                    serde_json::json!([
                        f32::from(b.left()),
                        f32::from(b.top()),
                        f32::from(b.size.width),
                        f32::from(b.size.height)
                    ])
                };
                probes.push(serde_json::json!({"label":label,"before_caption":bounds("evidence-caption-0"),"after_caption":bounds("evidence-caption-1"),"before_stage":bounds("phase5-before"),"after_stage":bounds("phase5-after")}));
            }
            manifest.push(label);
        });
        owned.join_after_ui();
    }
    let mut after = store.project_evidence(project).unwrap();
    assert!(!after.reviews.is_empty());
    after.reviews.clear();
    assert_eq!(frozen, serde_json::to_value(after).unwrap());
    fs::write(
        root.join("comparison-recovery-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    fs::write(
        root.join("comparison-recovery-focus-probes.json"),
        serde_json::to_vec_pretty(&probes).unwrap(),
    )
    .unwrap();
}

#[gpui::test]
#[ignore = "bounded two-size unchanged-success and changed-only navigation inspection"]
fn renderer_capture_unchanged_review(cx: &mut TestAppContext) {
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
        .prefix("unchanged-review-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (mut store, project, _) = ui_review_matrix::fixture_with_capture_warnings(
        &root,
        dir.path(),
        false,
        &crate::comparison_ux_tests::WARNINGS,
    );
    let mut legacy = store.project_evidence(project).unwrap().comparison.unwrap();
    legacy.revision.id = regressee_core::domain::AnalysisId::new();
    legacy.perceptual.as_mut().unwrap().cache_key =
        regressee_core::evidence::identity_hash(&("legacy-warning-veto", legacy.revision.id))
            .unwrap();
    for result in &mut legacy.revision.results {
        result.classification = Classification::NotComparable;
    }
    store.save_analysis(&legacy).unwrap();
    store.display_analysis(legacy.revision.id).unwrap();
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = vec![];
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        for state in [
            "no-change",
            "next-change",
            "no-change-filter",
            "pending",
            "unreadable",
        ] {
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    if state == "no-change" {
                        load_review(s, project, cx);
                        s.notice = None;
                        s.error = None;
                    }
                    if state == "next-change" {
                        assert!(s.next_unreviewed_key().is_some());
                        s.activate_control(Control::Phase5(33), w, cx);
                        crate::workflow_refinement_tests::finish_jobs(s, cx);
                        assert!(s.review_result_changed());
                    } else {
                        let a = s.evidence.comparison.as_ref().unwrap();
                        let key = a
                            .revision
                            .results
                            .iter()
                            .find(|r| {
                                a.review_classification(&r.key)
                                    == Some(Classification::NoSignificantChange)
                            })
                            .unwrap()
                            .key
                            .clone();
                        s.selected_key = Some(key);
                        s.result_filter = if state == "no-change-filter" { 3 } else { 0 };
                        s.reload_viewer(cx);
                        crate::workflow_refinement_tests::finish_jobs(s, cx);
                        s.analysis_due = if state == "pending" {
                            Some(std::time::Instant::now() + Duration::from_secs(3600))
                        } else {
                            None
                        };
                        s.viewer_read_failed = state == "unreadable";
                        if !matches!(state, "pending" | "unreadable") {
                            assert!(s.review_result_unchanged());
                            for index in 20..=22 {
                                assert!(!s.control_mounted(Control::Phase5(index)));
                            }
                        }
                        if state == "no-change-filter" {
                            assert!(s.next_unreviewed_key().is_none());
                        }
                    }
                    cx.notify();
                })
            });
            let label = format!("unchanged-review-{state}-{width:.0}x{height:.0}");
            capture(cx, &root, &label).unwrap();
            manifest.push(label);
        }
        owned.join_after_ui();
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("unchanged-review-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
