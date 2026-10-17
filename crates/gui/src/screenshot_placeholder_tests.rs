// SPDX-License-Identifier: MPL-2.0
//! Synthetic missing-image states; native layout, not OS-input qualification.
use super::*;
use crate::workflow_refinement_tests::{load_review, screenshot_switch_fixture};
use gpui::{TestAppContext, VisualTestContext, point};
use regressee_core::domain::{CaptureError, ProjectId};

const SCENES: [&str; 8] = [
    "image-limit",
    "timeout",
    "skipped",
    "cancelled",
    "unreadable",
    "long-reason",
    "hidden-results",
    "filters",
];

fn prepare_scene(s: &mut Shell, scene: &str, project: ProjectId, cx: &mut Context<Shell>) {
    load_review(s, project, cx);
    s.clear_viewer();
    s.focus.review_surface = ReviewSurface::Inspect;
    s.evidence.comparison = None;
    s.evidence.baseline = None;
    s.focus.baseline = false;
    s.focus.candidate = false;
    s.notice = None;
    s.error = None;
    s.viewer_read_failed = scene == "unreadable";
    s.navigation_preference = Some(false);
    s.review_queue_collapsed = scene == "hidden-results";
    s.result_filters_open = scene == "filters";
    let key = s.selected_key.as_ref().unwrap();
    let run = s.evidence.latest.as_mut().unwrap();
    run.run.profiles[0].width = 1440;
    run.run.profiles[0].height = 900;
    let target = run.run.targets.iter_mut().find(|t| &t.key == key).unwrap();
    target.outcome = match scene {
        "timeout" => TargetOutcome::Failed { code: CaptureError::Timeout },
        "cancelled" => TargetOutcome::Cancelled,
        "skipped" => TargetOutcome::Skipped { reason: "The page was outside the approved website.".into() },
        "long-reason" => TargetOutcome::Skipped { reason: "This deliberately long recorded explanation checks that recovery guidance remains reachable without pushing the screenshot frame or result list outside the window. ".repeat(12) },
        "unreadable" => target.outcome.clone(),
        _ => TargetOutcome::Failed { code: CaptureError::ImageLimit },
    };
    if scene != "unreadable" {
        run.run.state = JobState::CompletedWithErrors;
        run.captures.retain(|c| &c.metadata.key != key);
        run.capture = None;
    }
    s.review_details_scroll.set_offset(point(px(0.), px(0.)));
    cx.notify();
}

fn draw(cx: &mut VisualTestContext) {
    crate::phase4_hierarchy_tests::draw(cx);
    cx.run_until_parked();
}

fn check_layout(shell: &Entity<Shell>, cx: &mut VisualTestContext, scene: &str) -> bool {
    let frame = cx.debug_bounds("unavailable-target-recovery").unwrap();
    let owner = cx.debug_bounds("unavailable-target-body").unwrap();
    assert_eq!(frame.left(), owner.left());
    assert_eq!(frame.right(), owner.right());
    assert_eq!(
        frame.bottom(),
        owner.bottom(),
        "{scene}: fill the evidence height"
    );
    assert!(frame.size.height > px(160.), "{scene}: {frame:?}");
    let viewport = cx.debug_bounds("screenshot-placeholder-scroll").unwrap();
    let icon = cx.debug_bounds("screenshot-placeholder-icon").unwrap();
    let message = cx.debug_bounds("screenshot-placeholder-message").unwrap();
    let title = cx.debug_bounds("screenshot-placeholder-title").unwrap();
    let reason = cx.debug_bounds("screenshot-placeholder-reason").unwrap();
    assert_eq!(icon.size, size(px(48.), px(48.)));
    assert!((f32::from(icon.center().x - frame.center().x)).abs() < 1.);
    assert!(message.size.width <= px(440.));
    assert!(message.left() >= frame.left() + px(24.));
    assert!(message.right() <= frame.right() - px(24.));
    assert!(title.top() >= icon.bottom() + px(16.));
    assert!(reason.top() >= title.bottom() + px(8.));
    crate::phase1_setup_tests::inside(icon, viewport);
    for selector in [
        "screenshot-placeholder-title",
        "screenshot-placeholder-reason",
        "screenshot-placeholder-help",
    ] {
        crate::phase1_setup_tests::inside(cx.debug_bounds(selector).unwrap(), message);
    }
    if message.size.height + px(48.) <= viewport.size.height {
        crate::phase1_setup_tests::inside(message, viewport);
        assert!(
            (f32::from(message.center().y - viewport.center().y)).abs() < 1.,
            "{scene}: message {message:?}, viewport {viewport:?}"
        );
        false
    } else {
        assert!(message.top() >= viewport.top() + px(24.));
        cx.simulate_event(gpui::ScrollWheelEvent {
            position: viewport.center(),
            delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-10000.))),
            ..Default::default()
        });
        draw(cx);
        let help = cx.debug_bounds("screenshot-placeholder-help").unwrap();
        shell.read_with(cx, |s, _| assert!(s.screenshot_scroll.offset().y < px(0.)));
        crate::phase1_setup_tests::inside(help, viewport);
        assert_eq!(
            cx.debug_bounds("unavailable-target-recovery").unwrap(),
            frame
        );
        true
    }
}

fn close_filters(shell: &Entity<Shell>, cx: &mut VisualTestContext, scene: &str) {
    if scene == "filters" {
        assert!(cx.debug_bounds("page-options-panel").is_some());
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::CloseOptions, window, cx)
            })
        });
        draw(cx);
    }
}

#[gpui::test]
fn screenshot_placeholder_fills_the_stage_and_keeps_recovery_reachable(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let project = screenshot_switch_fixture(dir.path(), false);
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for scene in SCENES {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| prepare_scene(s, scene, project, cx));
            draw(cx);
            close_filters(&shell, cx, scene);
            check_layout(&shell, cx, scene);
            shell.read_with(cx, |s, cx| {
                assert!(!s.control_mounted(Control::Viewer));
                assert!(!s.control_mounted(Control::Guided(46)));
                assert!(!s.control_enabled(Control::SaveBaseline, cx));
                assert!(s.control_mounted(Control::Phase5(36)));
            });
            owned.join_after_ui();
        }
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "explicit two-size native offscreen missing-screenshot inspection"]
fn renderer_capture_screenshot_placeholders(cx: &mut TestAppContext) {
    let root = crate::ui_renderer_capture::output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    let dir = tempfile::Builder::new()
        .prefix("placeholder-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let project = screenshot_switch_fixture(dir.path(), false);
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for scene in SCENES {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| prepare_scene(s, scene, project, cx));
            draw(cx);
            cx.run_until_parked();
            draw(cx);
            let label = format!("placeholder-{width:.0}x{height:.0}-{scene}");
            crate::ui_renderer_capture::capture(cx, &root, &label).unwrap();
            manifest.push(label.clone());
            close_filters(&shell, cx, scene);
            if check_layout(&shell, cx, scene) {
                let label = format!("{label}-recovery");
                crate::ui_renderer_capture::capture(cx, &root, &label).unwrap();
                manifest.push(label);
            }
            owned.join_after_ui();
        }
    }
    std::fs::write(
        root.join("placeholder-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
