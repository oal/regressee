// SPDX-License-Identifier: MPL-2.0
//! Bounded desktop/theme inspection, real GPUI offscreen, not OS input.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

#[gpui::test]
#[ignore = "scoped retained-feedback viewport confirmation requested by independent verifier"]
fn renderer_capture_agent_retained_feedback(cx: &mut TestAppContext) {
    use crate::agent_feedback_tests::{FeedbackCase, assert_shared_feedback, configure_feedback};
    use regressee_app::control::Status;
    use sha2::{Digest, Sha256};
    let output = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::agent_control_tests::bind(cx);
    let mut frames = Vec::new();
    for appearance in [AppAppearance::Light, AppAppearance::Dark] {
        for status in [Status::Active, Status::Human] {
            for case in [FeedbackCase::Saved, FeedbackCase::Error, FeedbackCase::Both] {
                let dir = tempfile::tempdir_in(&output).unwrap();
                let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
                let frozen =
                    serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
                drop(store);
                regressee_storage::AppPreferences {
                    appearance,
                    ..Default::default()
                }
                .save(&AppPaths::isolated(dir.path()))
                .unwrap();
                let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
                let owner = jobs.clone();
                owner.appearance.wait_loaded();
                let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                cx.simulate_resize(size(px(820.), px(600.)));
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        load_review(s, project, cx);
                        configure_feedback(s, status, case, cx);
                        w.focus(&s.agent_handles[0]);
                    })
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                cx.run_until_parked();
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                let height = assert_shared_feedback(&shell, cx, status, case);
                let banner_height =
                    f32::from(cx.debug_bounds("agent-control-banner").unwrap().size.height);
                let name = format!(
                    "feedback-{}-820x600-{status:?}-{}",
                    appearance.label().to_lowercase(),
                    case.label()
                );
                capture(cx, &output, &name).unwrap();
                frames.push(shell.read_with(cx,|s,_|serde_json::json!({"name":name,"evidence_height":height,"banner_height":banner_height,"notice":s.notice,"error":s.error,"scroll_y":f32::from(s.agent_details_scroll.offset().y),"scroll_extent":f32::from(s.agent_details_scroll.max_offset().height)})));
                if matches!(case, FeedbackCase::Both) {
                    cx.update(|w, cx| shell.update(cx, |s, _| w.focus(&s.agent_handles[2])));
                    cx.simulate_keystrokes("end");
                    shell.read_with(cx, |s, _| {
                        assert_eq!(
                            s.agent_details_scroll.offset().y,
                            -s.agent_details_scroll.max_offset().height
                        )
                    });
                    assert_shared_feedback(&shell, cx, status, case);
                    capture(cx, &output, &format!("{name}-end")).unwrap();
                    frames.push(shell.read_with(cx,|s,_|serde_json::json!({"name":format!("{name}-end"),"evidence_height":height,"notice":s.notice,"error":s.error,"scroll_y":f32::from(s.agent_details_scroll.offset().y),"scroll_extent":f32::from(s.agent_details_scroll.max_offset().height)})));
                }
                shell.update(cx, finish_jobs);
                owner.join_after_ui();
                cx.update(|w, _| w.remove_window());
                let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
                assert_eq!(
                    frozen,
                    serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
                );
            }
        }
    }
    let proof = serde_json::json!({
        "main_source_sha256":hex::encode(Sha256::digest(include_bytes!("main.rs"))),
        "agent_ux_source_sha256":hex::encode(Sha256::digest(include_bytes!("agent_ux.rs"))),
        "regression_source_sha256":hex::encode(Sha256::digest(include_bytes!("agent_feedback_tests.rs"))),
        "capture_source_sha256":hex::encode(Sha256::digest(include_bytes!("ui_agent_capture.rs"))),
        "original_evidence_unchanged":true,"os_input_verified":false,"frames":frames
    });
    std::fs::write(
        output.join("feedback-metrics.json"),
        serde_json::to_vec_pretty(&proof).unwrap(),
    )
    .unwrap();
    set_dark(false);
}

#[gpui::test]
#[ignore = "explicit bounded independent-finding repair capture batch"]
fn renderer_capture_agent_control_repairs(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::agent_control_tests::bind(cx);
    let dir = tempfile::tempdir_in(&root).unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        for appearance in [AppAppearance::Light, AppAppearance::Dark] {
            for label in [
                "long-handoff",
                "long-failure",
                "expired",
                "disconnected",
                "taken-over",
            ] {
                regressee_storage::AppPreferences {
                    appearance,
                    ..Default::default()
                }
                .save(&AppPaths::isolated(dir.path()))
                .unwrap();
                let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
                let owner = jobs.clone();
                owner.appearance.wait_loaded();
                let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                cx.simulate_resize(size(px(width), px(height)));
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        load_review(s, project, cx);
                        crate::agent_repair_tests::edge_state(s, label, cx);
                        w.focus(&s.agent_handles[0]);
                    })
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                cx.run_until_parked();
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                let banner = cx.debug_bounds("agent-control-banner").unwrap();
                assert!(banner.size.height <= px(88.));
                assert!(cx.debug_bounds("phase5-before").unwrap().size.height >= px(64.));
                if matches!(label, "long-handoff" | "long-failure") {
                    let name = cx.debug_bounds("agent-control-name").unwrap();
                    let status = cx.debug_bounds("agent-control-status").unwrap();
                    let details = cx.debug_bounds("agent-control-details").unwrap();
                    assert!(name.size.height <= px(20.) && status.size.height <= px(20.));
                    assert!(name.size.width >= px(200.) && status.size.width >= px(200.));
                    assert!(name.bottom() <= status.top() && status.bottom() <= details.top());
                }
                let action = cx
                    .debug_bounds(if matches!(label, "long-handoff" | "long-failure") {
                        "agent-take-control"
                    } else {
                        "agent-dismiss-status"
                    })
                    .unwrap();
                crate::phase2_evidence_tests::inside(action, banner);
                if label == "long-handoff" {
                    crate::phase2_evidence_tests::inside(
                        cx.debug_bounds("agent-follow-return").unwrap(),
                        banner,
                    );
                }
                capture(
                    cx,
                    &root,
                    &format!(
                        "repair-{}-{width:.0}x{height:.0}-{label}",
                        appearance.label().to_lowercase()
                    ),
                )
                .unwrap();
                shell.update(cx, finish_jobs);
                owner.join_after_ui();
                cx.update(|w, _| w.remove_window());
            }
        }
    }
    set_dark(false);
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
#[ignore = "explicit bounded exclusive-follow banner inspection"]
fn renderer_capture_agent_control(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::agent_control_tests::bind(cx);
    let dir = tempfile::tempdir_in(&root).unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        for appearance in [AppAppearance::Light, AppAppearance::Dark] {
            for status in [
                regressee_app::control::Status::Active,
                regressee_app::control::Status::Human,
            ] {
                regressee_storage::AppPreferences {
                    appearance,
                    ..Default::default()
                }
                .save(&AppPaths::isolated(dir.path()))
                .unwrap();
                let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
                let owner = jobs.clone();
                owner.appearance.wait_loaded();
                let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                cx.simulate_resize(size(px(width), px(height)));
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        load_review(s, project, cx);
                        s.error = None;
                        s.notice = None;
                        s.agent = crate::agent_control_tests::state(status, true);
                        w.focus(&s.agent_handles[0]);
                        cx.notify();
                    })
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                cx.run_until_parked();
                cx.update(|w, cx| {
                    assert!(
                        brand::logo(Background::workspace())
                            .get_render_image(w, cx)
                            .is_some()
                    );
                    let _ = w.draw(cx);
                });
                let banner = cx.debug_bounds("agent-control-banner").unwrap();
                let take = cx.debug_bounds("agent-take-control").unwrap();
                crate::phase2_evidence_tests::inside(take, banner);
                assert!(banner.size.height <= px(160.));
                assert!(banner.right() <= px(width));
                assert_eq!(
                    cx.debug_bounds("agent-input-shield").is_some(),
                    status == regressee_app::control::Status::Active
                );
                if status == regressee_app::control::Status::Human {
                    crate::phase2_evidence_tests::inside(
                        cx.debug_bounds("agent-follow-return").unwrap(),
                        banner,
                    );
                }
                capture(
                    cx,
                    &root,
                    &format!(
                        "agent-{}-{width:.0}x{height:.0}-{status:?}",
                        appearance.label().to_lowercase()
                    ),
                )
                .unwrap();
                shell.update(cx, finish_jobs);
                owner.join_after_ui();
                cx.update(|w, _| w.remove_window());
            }
        }
    }
    set_dark(false);
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
