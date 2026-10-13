// SPDX-License-Identifier: MPL-2.0
//! One bounded Phase 2 matrix. Real production panes/menus, offline synthetic data.
use super::*;
use crate::{
    phase2_evidence_tests::inside,
    workflow_refinement_tests::{finish_jobs, load_review},
};
use std::sync::atomic::AtomicBool;

fn geometry(b: Bounds<Pixels>) -> serde_json::Value {
    serde_json::json!({"x":f32::from(b.left()),"y":f32::from(b.top()),"width":f32::from(b.size.width),"height":f32::from(b.size.height)})
}

fn choose(s: &mut Shell, mode: usize, w: &mut Window, cx: &mut Context<Shell>) {
    assert!(s.control_enabled(Control::Guided(54), cx));
    s.activate_control(Control::Guided(54), w, cx);
    assert!(s.viewer_mode_open);
    let option = if mode == 4 {
        Control::Guided(55)
    } else {
        Control::Phase5(mode + 6)
    };
    assert!(s.control_mounted(option) && s.control_enabled(option, cx));
    s.activate_control(option, w, cx);
    assert!(!s.viewer_mode_open && s.viewer_mode == mode);
}

#[gpui::test]
#[ignore = "explicit two-size Phase 2 native render batch; synthetic/offline only"]
fn renderer_capture_phase2(cx: &mut TestAppContext) {
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
    cx.update(|cx| {
        zoom_select::bind(cx);
        cx.bind_keys([
            KeyBinding::new("enter", Activate, Some("Shell")),
            KeyBinding::new("escape", Dismiss, Some("Shell")),
            KeyBinding::new("tab", Tab, Some("Shell")),
            KeyBinding::new("shift-tab", TabPrev, Some("Shell")),
        ]);
    });
    let dir = tempfile::Builder::new()
        .prefix("phase2-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (mut store, project, check) =
        super::ui_review_matrix::fixture_named(&root, dir.path(), true);
    let evidence = store.project_evidence(project).unwrap();
    let key = evidence
        .latest
        .as_ref()
        .unwrap()
        .run
        .targets
        .iter()
        .find(|t| {
            t.requested.path().starts_with("/account")
                && t.key.profile == evidence.latest.as_ref().unwrap().run.profiles[0].id
        })
        .unwrap()
        .key
        .clone();
    let original_ids: Vec<_> = evidence
        .latest
        .as_ref()
        .unwrap()
        .captures
        .iter()
        .map(|c| c.image.original.clone())
        .collect();
    let original_hashes: Vec<_> = original_ids
        .iter()
        .map(|id| {
            hex::encode(Sha256::digest(
                fs::read(store.artifact_path(id).unwrap()).unwrap(),
            ))
        })
        .collect();
    let mut request = evidence
        .comparison
        .as_ref()
        .unwrap()
        .perceptual
        .as_ref()
        .unwrap()
        .reports
        .iter()
        .find(|(k, _)| k == &key)
        .unwrap()
        .1
        .request
        .clone();
    request.mask = Some(regressee_app::analysis_engine::MaskBinding {
        revision: "phase2-saved-ignore".into(),
        target: request.before.target.clone(),
        before_capture_id: request.before.capture_id.clone(),
        after_capture_id: request.after.capture_id.clone(),
        before_dimensions: request.before.dimensions,
        after_dimensions: request.after.dimensions,
        rectangles: vec![regressee_app::analysis_engine::Rectangle {
            x: 700,
            y: 160,
            width: 400,
            height: 300,
        }],
    });
    store.save_mask(project, &request).unwrap();
    let analysis = regressee_app::review::analyze_run(
        &mut store,
        check,
        request.policy.clone(),
        &AtomicBool::new(false),
    )
    .unwrap();
    let report = &analysis
        .perceptual
        .as_ref()
        .unwrap()
        .reports
        .iter()
        .find(|(k, _)| k == &key)
        .unwrap()
        .1;
    assert!(report.changed_pixels > 0 && report.ignored_pixels == 120_000);
    let heat = analysis
        .heatmaps
        .iter()
        .find(|(k, _)| k == &key)
        .unwrap()
        .1
        .clone();
    let heat_image =
        image::load_from_memory(&fs::read(store.artifact_path(&heat.original).unwrap()).unwrap())
            .unwrap()
            .into_rgba8();
    let significant = heat_image
        .pixels()
        .filter(|p| p.0 == [220, 50, 70, 255])
        .count();
    let ignored = heat_image
        .pixels()
        .filter(|p| p.0 == [95, 120, 130, 255])
        .count();
    assert!(significant > 0 && ignored == 120_000);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        let scenes = [
            "compare",
            "queue-hidden",
            "full-before",
            "full-after",
            "navigation-compare",
            "navigation-hidden",
            "navigation-before",
            "navigation-after",
            "difference",
            "overlay",
            "heatmap",
            "view-menu",
            "zoom-menu",
            "dirty-note",
            "dirty-geometry",
            "dirty-area",
            "pending",
            "noncomparable",
            "exhausted",
            "selected",
        ];
        let mut defaults = std::collections::BTreeMap::new();
        for scene in scenes {
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.analysis_due = None;
                    s.analysis_pending = None;
                    s.mask_dirty = false;
                    s.mask_selected = None;
                    s.review_drafts = Default::default();
                    s.search.clear();
                    s.crawl_fields[9].update(cx, |f, cx| f.set_value("", cx));
                    s.approval_selection.clear();
                    s.review_tools = false;
                    s.capture_details_open = false;
                    s.review_note_open = false;
                    s.viewer_mode_open = false;
                    load_review(s, project, cx);
                    s.selected_key = Some(key.clone());
                    s.clear_viewer();
                    s.reload_viewer(cx);
                    finish_jobs(s, cx);
                    s.sync_review_note(cx);
                    // Loading this synthetic saved project leaves a setup notice.
                    // Clean inspection must actually dismiss it before measuring.
                    if s.control_enabled(Control::DismissFeedback, cx) {
                        s.activate_control(Control::DismissFeedback, w, cx);
                    }
                    s.review_queue_collapsed = false;
                    s.viewer_mode = 0;
                    s.full_width_after = false;
                    s.viewer_fit = true;
                    s.navigation_preference = Some(true);
                    s.configure_review_geometry(width);
                    if scene.starts_with("navigation") {
                        s.activate_control(Control::Guided(53), w, cx);
                        assert!(!s.navigation_collapsed);
                    }
                    if matches!(
                        scene,
                        "queue-hidden"
                            | "full-before"
                            | "full-after"
                            | "navigation-hidden"
                            | "navigation-before"
                            | "navigation-after"
                    ) {
                        s.activate_control(Control::Guided(50), w, cx);
                        assert!(s.review_queue_collapsed);
                        assert!(!s.control_mounted(Control::CrawlField(9)));
                    }
                    match scene {
                        "full-before" | "full-after" | "navigation-before" | "navigation-after" => {
                            choose(s, 4, w, cx);
                            if scene.ends_with("after") {
                                s.activate_control(Control::Guided(56), w, cx);
                                assert!(s.full_width_after);
                            }
                        }
                        "difference" => choose(s, 1, w, cx),
                        "overlay" => choose(s, 2, w, cx),
                        "heatmap" => choose(s, 3, w, cx),
                        "view-menu" => {
                            s.activate_control(Control::Guided(54), w, cx);
                            assert!(s.viewer_mode_open);
                            assert!(!s.control_mounted(Control::Phase5(21)));
                        }
                        "zoom-menu" => s.activate_control(Control::Guided(46), w, cx),
                        "dirty-note" => {
                            s.activate_control(Control::Phase5(37), w, cx);
                            s.analysis_fields[9].update(cx, |f, cx| {
                                f.set_value(
                                    "Autumn headline is intentional; confirm delivery changes.",
                                    cx,
                                )
                            });
                            s.dismiss(&Dismiss, w, cx);
                            assert!(s.review_note_dirty(cx));
                        }
                        "dirty-geometry" | "dirty-area" => {
                            s.activate_control(Control::Guided(44), w, cx);
                            s.activate_control(Control::Phase5(34), w, cx);
                            assert!(s.mask_dirty && s.mask_selected.is_some());
                            if scene == "dirty-geometry" {
                                s.analysis_fields[10].update(cx, |f, cx| f.set_value("18", cx));
                                assert!(s.mask_geometry_dirty(cx));
                            }
                            s.dismiss(&Dismiss, w, cx);
                        }
                        "pending" => {
                            s.analysis_due =
                                Some(std::time::Instant::now() + Duration::from_secs(3600))
                        }
                        "noncomparable" => {
                            let r = s
                                .evidence
                                .comparison
                                .as_mut()
                                .unwrap()
                                .revision
                                .results
                                .iter_mut()
                                .find(|r| r.key == key)
                                .unwrap();
                            r.classification = Classification::NotComparable;
                            r.eligible_pixels = 0;
                        }
                        "exhausted" => {
                            s.search = "no-matching-screenshot".into();
                            s.crawl_fields[9]
                                .update(cx, |f, cx| f.set_value("no-matching-screenshot", cx));
                        }
                        "selected" => {
                            assert!(s.control_enabled(Control::Phase5(23), cx));
                            s.activate_control(Control::Phase5(23), w, cx);
                            assert!(s.approval_selection.contains(&key));
                        }
                        _ => (),
                    }
                    if matches!(
                        scene,
                        "dirty-note"
                            | "dirty-geometry"
                            | "dirty-area"
                            | "pending"
                            | "noncomparable"
                            | "exhausted"
                    ) {
                        assert!(s.review_blocking_reason(cx).is_some());
                        if scene.starts_with("dirty") || scene == "exhausted" {
                            assert!(!s.control_enabled(Control::Phase5(33), cx));
                        }
                    }
                    let (path, context) = s.inspection_identity();
                    assert!(path.starts_with("/account"));
                    assert!(
                        context.contains("Recorded reference")
                            && context.contains("Recorded check")
                    );
                    assert_eq!(s.selected_key.as_ref(), Some(&key));
                    cx.notify();
                })
            });
            // Settle production layout/focus without consuming the constructed pending state.
            for _ in 0..3 {
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
            }
            let label = format!("phase2-{width:.0}x{height:.0}-{scene}");
            if scene == "zoom-menu" {
                shell.read_with(cx, |s, cx| {
                    assert!(s.zoom_select.read(cx).is_open());
                    for control in [
                        Control::Phase5(21),
                        Control::Phase5(23),
                        Control::Phase5(24),
                        Control::Phase5(33),
                    ] {
                        assert!(!s.control_enabled(control, cx));
                    }
                });
            }
            capture(cx, &root, &label).unwrap();
            let pane = cx
                .debug_bounds(if scene.ends_with("after") {
                    "phase5-after"
                } else if matches!(scene, "difference" | "heatmap") {
                    "phase5-difference"
                } else {
                    "phase5-before"
                })
                .unwrap();
            let workspace = cx.debug_bounds("bounded-review").unwrap();
            let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
            inside(pane, workspace);
            inside(toolbar, workspace);
            assert_eq!(toolbar.size.height, px(40.));
            for name in [
                "inspected-target-context",
                "inspected-target-path",
                "inspection-profile",
                "inspection-before-environment",
                "inspection-after-environment",
            ] {
                inside(cx.debug_bounds(name).unwrap(), workspace);
            }
            let nav = scene.starts_with("navigation");
            if matches!(scene, "compare" | "navigation-compare") {
                defaults.insert(nav, pane);
            }
            if matches!(
                scene,
                "queue-hidden"
                    | "full-before"
                    | "full-after"
                    | "navigation-hidden"
                    | "navigation-before"
                    | "navigation-after"
            ) {
                assert!(
                    pane.size.height >= defaults[&nav].size.height,
                    "{label}: {pane:?} vs {:?}",
                    defaults[&nav]
                );
                assert!(
                    pane.size.height >= px(180.),
                    "{label}: usable native document height {pane:?}"
                );
            }
            for name in [
                "action-Guided(50)",
                "action-Guided(54)",
                "zoom-select-button",
                "action-Phase5(36)",
                "action-Guided(44)",
            ] {
                inside(cx.debug_bounds(name).unwrap(), toolbar);
            }
            if scene == "view-menu" {
                let trigger = cx.debug_bounds("action-Guided(54)").unwrap();
                let menu = cx.debug_bounds("viewer-mode-chooser").unwrap();
                assert_eq!(menu.top(), trigger.bottom() + px(4.));
                assert_eq!(menu.left(), trigger.left());
                inside(
                    menu,
                    Bounds::new(
                        gpui::point(px(8.), px(8.)),
                        size(px(width - 16.), px(height - 16.)),
                    ),
                );
            }
            if scene == "zoom-menu" {
                let trigger = cx.debug_bounds("zoom-select-button").unwrap();
                let menu = cx.debug_bounds("zoom-popup").unwrap();
                assert!(
                    (menu.top() - trigger.bottom() - px(4.)).abs() < px(1.)
                        || (trigger.top() - menu.bottom() - px(4.)).abs() < px(1.)
                );
            }
            let blocker =
                shell.read_with(cx, |s, cx| s.review_blocking_reason(cx).map(str::to_owned));
            if blocker.is_some() && scene != "view-menu" {
                inside(
                    cx.debug_bounds("review-blocking-reason").unwrap(),
                    workspace,
                );
            }
            if scene == "heatmap" {
                inside(
                    cx.debug_bounds("heatmap-legend").unwrap(),
                    cx.debug_bounds("evidence-caption-2").unwrap(),
                );
            }
            manifest.push(serde_json::json!({"label":label,"size":[width,height],"scene":scene,"pane":geometry(pane),"toolbar":geometry(toolbar),"heading":geometry(cx.debug_bounds("inspected-target-context").unwrap()),"blocker":blocker,"selected_target":key,"legend_active":scene=="heatmap","menu_trigger":if scene=="view-menu" {Some(geometry(cx.debug_bounds("action-Guided(54)").unwrap()))} else {None},"menu":if scene=="view-menu" {Some(geometry(cx.debug_bounds("viewer-mode-chooser").unwrap()))} else {None},"same_target_height_asserted":true}));
        }
        shell.update(cx, |s, _| s.analysis_due = None);
        owned.join_after_ui();
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    for (id, hash) in original_ids.iter().zip(original_hashes.iter()) {
        assert_eq!(
            *hash,
            hex::encode(Sha256::digest(
                fs::read(store.artifact_path(id).unwrap()).unwrap()
            ))
        );
    }
    fs::write(root.join("phase2-matrix.json"),serde_json::to_vec_pretty(&serde_json::json!({"frames":manifest,"heatmap":{"significant_pixels":significant,"ignored_pixels":ignored,"significant_rgb":[220,50,70],"ignored_rgb":[95,120,130]},"original_hashes_unchanged":original_hashes,"storage_unchanged_after_presentation":true})).unwrap()).unwrap();
}

#[gpui::test]
#[ignore = "targeted V2-1 drawing-toolbar confirmation; synthetic/offline only"]
fn renderer_capture_phase2_drawing_fix(cx: &mut TestAppContext) {
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
    cx.update(|cx| {
        zoom_select::bind(cx);
        cx.bind_keys([
            KeyBinding::new("enter", Activate, Some("Shell")),
            KeyBinding::new("escape", Dismiss, Some("Shell")),
            KeyBinding::new("tab", Tab, Some("Shell")),
        ]);
    });
    let dir = tempfile::Builder::new()
        .prefix("phase2-drawing-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = super::ui_review_matrix::fixture_named(&root, dir.path(), true);
    let evidence = store.project_evidence(project).unwrap();
    let run = evidence.latest.as_ref().unwrap();
    let key = run
        .run
        .targets
        .iter()
        .find(|t| {
            t.requested.path().starts_with("/account") && t.key.profile == run.run.profiles[0].id
        })
        .unwrap()
        .key
        .clone();
    let originals: Vec<_> = run
        .captures
        .iter()
        .map(|c| {
            let id = c.image.original.clone();
            let hash = hex::encode(Sha256::digest(
                fs::read(store.artifact_path(&id).unwrap()).unwrap(),
            ));
            (id, hash)
        })
        .collect();
    let frozen = serde_json::to_value(&evidence).unwrap();
    drop(store);
    let mut frames = Vec::new();
    for (width, height) in [(820., 600.), (1280., 800.)] {
        for after in [false, true] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.selected_key = Some(key.clone());
                    s.clear_viewer();
                    s.reload_viewer(cx);
                    finish_jobs(s, cx);
                    s.sync_review_note(cx);
                    if s.control_enabled(Control::DismissFeedback, cx) {
                        s.activate_control(Control::DismissFeedback, w, cx);
                    }
                })
            });
            let label = format!(
                "phase2-fix-{width:.0}x{height:.0}-navigation-drawing-{}",
                if after { "after" } else { "before" }
            );
            crate::phase2_evidence_tests::exercise_full_width_drawing_toolbar(
                &shell,
                cx,
                width,
                true,
                after,
                |cx| {
                    for _ in 0..3 {
                        cx.update(|w, cx| {
                            let _ = w.draw(cx);
                        });
                    }
                    capture(cx, &root, &label).unwrap();
                    let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
                    let actions: Vec<_> = ["action-Guided(50)", "action-Guided(54)", "action-Guided(56)", "zoom-select-button", "action-Phase5(14)"].into_iter().map(|name| {
                    let b = cx.debug_bounds(name).unwrap();
                    inside(b, toolbar);
                    serde_json::json!({"selector":name,"bounds":geometry(b),"mounted_enabled":true})
                }).collect();
                    frames.push(serde_json::json!({"label":label,"size":[width,height],"navigation_open":true,"results_hidden":true,"side":if after {"after"} else {"before"},"drawing_active":true,"pane":geometry(cx.debug_bounds(if after {"phase5-after"} else {"phase5-before"}).unwrap()),"toolbar":geometry(toolbar),"actions":actions,"selected_target":key,"all_actions_inside_owner_and_window":true}));
                },
            );
            // The shared input recipe completed pointer, Tab/Enter, exact
            // pair-bound drawing and Escape/reopened-Analysis assertions.
            owned.join_after_ui();
        }
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    for (id, hash) in &originals {
        assert_eq!(
            *hash,
            hex::encode(Sha256::digest(
                fs::read(store.artifact_path(id).unwrap()).unwrap()
            ))
        );
    }
    fs::write(root.join("phase2-drawing-fix.json"), serde_json::to_vec_pretty(&serde_json::json!({"frames":frames,"pointer_done_asserted":true,"tab_enter_done_asserted":true,"escape_and_analysis_reopen_asserted":true,"drawn_rectangle":{"x":8,"y":8,"width":16,"height":8},"pair_identity_and_unsaved_drafts_retained":true,"original_cas_hashes":originals,"storage_unchanged":true,"offscreen_only":true})).unwrap()).unwrap();
}
