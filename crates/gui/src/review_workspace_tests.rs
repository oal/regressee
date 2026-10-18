// SPDX-License-Identifier: MPL-2.0
//! Disclosure is presentation only; existing previews still own baseline consent.
use super::*;
use crate::dom_workflow_tests::{Engine, finish_source, load, syntax_fixture};
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{TestAppContext, VisualTestContext};

fn activate(shell: &Entity<Shell>, cx: &mut VisualTestContext, control: Control) {
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert!(
                s.control_enabled(control, cx),
                "{control:?} must be visible and enabled"
            );
            s.activate_control(control, window, cx);
        })
    });
    crate::phase4_hierarchy_tests::draw(cx);
}

fn inspect(cx: &mut VisualTestContext, label: &str) {
    crate::phase4_hierarchy_tests::draw(cx);
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    if std::env::var_os("REGRESSEE_RENDERER_CAPTURE_DIR").is_some() {
        let root = crate::ui_renderer_capture::output_root().unwrap();
        crate::ui_renderer_capture::capture(cx, &root, label).unwrap();
    }
    let _ = label;
}

fn single_line_profile(cx: &mut VisualTestContext) {
    let profile = cx.debug_bounds("inspection-profile").unwrap();
    for selector in ["inspection-profile-name", "inspection-profile-dimensions"] {
        let text = cx.debug_bounds(selector).unwrap();
        crate::phase2_evidence_tests::inside(text, profile);
        assert!(
            text.size.height <= px(20.),
            "{selector}: glyph layout must remain one line: {text:?}"
        );
    }
    let name = cx.debug_bounds("inspection-profile-name").unwrap();
    let dimensions = cx.debug_bounds("inspection-profile-dimensions").unwrap();
    assert!(dimensions.left() - name.right() <= px(8.));
}

fn set_profile_name(shell: &Entity<Shell>, cx: &mut VisualTestContext, name: &str) {
    shell.update(cx, |s, cx| {
        let run = s.evidence.latest.as_mut().unwrap();
        for profile in &mut run.run.profiles {
            profile.name = name.into();
        }
        for capture in &mut run.captures {
            capture.metadata.profile.name = name.into();
        }
        for capture in &mut run.dom_captures {
            capture.metadata.profile.name = name.into();
        }
        cx.notify();
    });
    crate::phase4_hierarchy_tests::draw(cx);
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
fn profile_ink_is_bounded(shell: &Entity<Shell>, cx: &mut VisualTestContext, name: &str) {
    let bounds = cx.debug_bounds("inspection-profile").unwrap();
    let text = cx.debug_bounds("inspection-profile-name").unwrap();
    let with_name = cx.update(|window, _| window.capture_offscreen_frame().unwrap());
    // Same capped allocation, no name ink. Everything past the name allocation
    // (including the actual dimensions glyphs) must paint identically.
    set_profile_name(shell, cx, &" ".repeat(200));
    let reference_text = cx.debug_bounds("inspection-profile-name").unwrap();
    assert_eq!(text, reference_text);
    let without_name = cx.update(|window, _| window.capture_offscreen_frame().unwrap());
    for y in f32::from(bounds.top()).ceil() as u32..f32::from(bounds.bottom()).floor() as u32 {
        for x in
            (f32::from(text.right()).ceil() as u32 + 2)..f32::from(bounds.right()).floor() as u32
        {
            let index = ((y * with_name.width + x) * 4) as usize;
            assert_eq!(
                &with_name.rgba[index..index + 4],
                &without_name.rgba[index..index + 4],
                "profile name ink escaped its allocation at {x},{y}"
            );
        }
    }
    set_profile_name(shell, cx, name);
}

fn exercise(shell: &Entity<Shell>, cx: &mut VisualTestContext, label: &str) {
    crate::phase4_hierarchy_tests::draw(cx);
    let requests = shell.read_with(cx, |s, cx| {
        assert!(!s.baseline_update_open);
        assert!(s.control_enabled(Control::BaselineUpdate, cx));
        assert!(!s.control_mounted(Control::Phase5(23)));
        assert!(!s.control_mounted(Control::Phase5(24)));
        assert!(!s.control_enabled(Control::Phase5(25), cx));
        assert_eq!(s.baseline_update_label(), "Update baseline…");
        let order = s.visible_controls();
        assert!(
            order
                .iter()
                .position(|c| *c == Control::BaselineUpdate)
                .unwrap()
                < order
                    .iter()
                    .position(|c| *c == Control::RefreshEvidence)
                    .unwrap()
        );
        s.next_request
    });
    let workspace = cx.debug_bounds("bounded-review").unwrap();
    let queue = cx.debug_bounds("queue-beside-viewer").unwrap();
    let heading = cx.debug_bounds("inspected-target-context").unwrap();
    let sources = cx
        .debug_bounds("inspected-target-profile-environments")
        .unwrap();
    let coverage = cx.debug_bounds("real-coverage-summary").unwrap();
    let index = shell.read_with(cx, |s, _| {
        s.evidence
            .latest
            .as_ref()
            .unwrap()
            .run
            .targets
            .iter()
            .position(|target| Some(&target.key) == s.selected_key.as_ref())
            .unwrap()
    });
    let selected_row = cx
        .debug_bounds(format!("page-row-target-{index}").leak())
        .unwrap();
    let marker = cx
        .debug_bounds(format!("result-selection-target-{index}").leak())
        .unwrap();
    assert_eq!(marker.size.width, px(1.));
    assert_eq!(marker.left(), selected_row.left());
    crate::phase2_evidence_tests::inside(marker, selected_row);
    if shell.read_with(cx, |s, _| {
        s.recorded_mode() == CaptureMode::ScreenshotsAndDomSnapshots
    }) {
        let tabs = cx.debug_bounds("evidence-kind-tabs").unwrap();
        for selector in [
            "inspected-outcome-Screenshot",
            "inspected-outcome-DomSnapshot",
        ] {
            crate::phase2_evidence_tests::inside(cx.debug_bounds(selector).unwrap(), tabs);
        }
    }
    single_line_profile(cx);
    assert!(sources.top() >= heading.bottom());
    assert!(coverage.top() >= sources.bottom());
    let name = cx.debug_bounds("inspection-profile-name").unwrap();
    let dimensions = cx.debug_bounds("inspection-profile-dimensions").unwrap();
    assert!(
        dimensions.left() - name.right() <= px(8.),
        "screen size stays beside its profile"
    );
    assert!(
        queue.size.height > px(100.),
        "{label}: comparison retains useful space: {queue:?}"
    );
    crate::phase2_evidence_tests::inside(
        cx.debug_bounds("action-BaselineUpdate").unwrap(),
        workspace,
    );
    inspect(cx, &format!("{label}-closed"));

    activate(shell, cx, Control::BaselineUpdate);
    shell.read_with(cx, |s, cx| {
        assert!(s.baseline_update_open && s.viewer_inspector_open());
        assert!(s.control_enabled(Control::Phase5(23), cx));
        assert!(!s.control_mounted(Control::Phase5(24)));
        assert!(!s.control_enabled(Control::Phase5(21), cx));
        assert!(!s.control_enabled(Control::CheckChanges, cx));
        assert!(!s.control_enabled(Control::RefreshEvidence, cx));
        assert_eq!(requests, s.next_request, "opening is not an engine command");
    });
    cx.update(|window, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(s.handle(Control::Phase5(39), cx).is_focused(window));
        })
    });
    let panel = cx.debug_bounds("viewer-inspector").unwrap();
    for selector in [
        "action-Phase5(39)",
        "action-Phase5(23)",
        "baseline-panel-actions",
    ] {
        crate::phase2_evidence_tests::inside(cx.debug_bounds(selector).unwrap(), panel);
    }
    assert_eq!(
        queue,
        cx.debug_bounds("queue-beside-viewer").unwrap(),
        "overlay never resizes evidence"
    );
    inspect(cx, &format!("{label}-open"));
    activate(shell, cx, Control::Phase5(23));
    shell.read_with(cx, |s, cx| {
        assert_eq!(s.approval_selection.len() + s.dom.selected.len(), 1);
        assert_eq!(s.baseline_update_label(), "Update baseline… (1)");
        assert!(!s.focus.approval_dialog);
        assert!(s.control_enabled(Control::Phase5(24), cx));
        assert_eq!(requests, s.next_request, "inclusion is not saving");
    });
    crate::phase2_evidence_tests::inside(cx.debug_bounds("action-Phase5(24)").unwrap(), panel);
    inspect(cx, &format!("{label}-selected"));
    cx.update(|window, cx| {
        shell.read_with(cx, |s, cx| window.focus(&s.handle(Control::Phase5(39), cx)))
    });
    crate::keyboard_navigation_tests::press(cx, "tab");
    cx.update(|window, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(s.handle(Control::Phase5(23), cx).is_focused(window));
        })
    });
    crate::keyboard_navigation_tests::press(cx, "tab");
    cx.update(|window, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(s.handle(Control::Phase5(24), cx).is_focused(window));
        })
    });

    // Escape restores the disclosure trigger, and hidden selection controls are
    // no longer keyboard targets. Reopening retains the explicit selection.
    crate::keyboard_navigation_tests::press(cx, "escape");
    cx.update(|window, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(!s.baseline_update_open);
            assert!(!s.control_mounted(Control::Phase5(23)));
            assert!(s.handle(Control::BaselineUpdate, cx).is_focused(window));
            assert_eq!(s.approval_selection.len() + s.dom.selected.len(), 1);
        })
    });
    activate(shell, cx, Control::BaselineUpdate);
    activate(shell, cx, Control::Phase5(24));
    shell.update(cx, finish_jobs);
    crate::phase4_hierarchy_tests::draw(cx);
    shell.read_with(cx, |s, cx| {
        assert!(s.focus.approval_dialog);
        assert!(s.approval_preview_current());
        assert!(s.control_enabled(Control::Phase5(25), cx));
        assert!(!s.control_mounted(Control::BaselineUpdate));
    });
    inspect(cx, &format!("{label}-confirmation"));
    crate::keyboard_navigation_tests::press(cx, "escape");
    cx.update(|window, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(s.baseline_update_open && !s.focus.approval_dialog);
            assert!(s.handle(Control::Phase5(24), cx).is_focused(window));
        })
    });
    activate(shell, cx, Control::Phase5(39));
    cx.update(|window, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(!s.baseline_update_open);
            assert!(s.handle(Control::BaselineUpdate, cx).is_focused(window));
            assert!(!s.approval_selection.is_empty() || !s.dom.selected.is_empty());
        })
    });
}

#[gpui::test]
fn baseline_disclosure_screenshot_and_dom_keep_explicit_consent(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    if std::env::var_os("REGRESSEE_RENDERER_CAPTURE_DIR").is_some() {
        cx.enable_offscreen_renderer().unwrap();
    }
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let fixture = syntax_fixture(CaptureMode::ScreenshotsAndDomSnapshots);
    let engine = Engine::start(&fixture);
    for kind in ["screenshot", "dom"] {
        for (width, height, hidden) in [
            (1280., 800., false),
            (820., 600., true),
            (820., 600., false),
        ] {
            for appearance in [AppAppearance::Light, AppAppearance::Dark] {
                let path = if kind == "dom" {
                    fixture.root.path()
                } else {
                    dir.path()
                };
                regressee_storage::AppPreferences {
                    appearance,
                    reduced_motion: true,
                }
                .save(&AppPaths::isolated(path))
                .unwrap();
                let (jobs, events) = if kind == "dom" {
                    engine.jobs()
                } else {
                    Jobs::start_without_runtime(AppPaths::isolated(path))
                };
                let owner = jobs.clone();
                owner.appearance.wait_loaded();
                let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                vcx.simulate_resize(size(px(width), px(height)));
                shell.update(vcx, |s, cx| {
                    if kind == "dom" {
                        load(s, &fixture, cx);
                        finish_source(s, cx);
                    } else {
                        load_review(s, project, cx);
                    }
                    s.navigation_preference = Some(hidden);
                    s.configure_review_geometry(width);
                    s.notice = None;
                    s.error = None;
                    cx.notify();
                });
                exercise(
                    &shell,
                    vcx,
                    &format!(
                        "workspace-{kind}-{}-{width:.0}x{height:.0}-nav{}",
                        appearance.label().to_lowercase(),
                        !hidden
                    ),
                );
                // Presentation-only long-profile fixture: prove bounded ellipsis
                // and native line-box containment without modifying stored data.
                let name = "Desktop — an intentionally long screen-size profile name for regression inspection";
                set_profile_name(&shell, vcx, name);
                single_line_profile(vcx);
                #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
                if std::env::var_os("REGRESSEE_RENDERER_CAPTURE_DIR").is_some() {
                    profile_ink_is_bounded(&shell, vcx, name);
                }
                inspect(
                    vcx,
                    &format!(
                        "workspace-{kind}-{}-{width:.0}x{height:.0}-nav{}-long-profile",
                        appearance.label().to_lowercase(),
                        !hidden
                    ),
                );
                shell.update(vcx, finish_jobs);
                owner.join_after_ui();
                vcx.update(|window, _| window.remove_window());
            }
        }
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    let dom_store =
        regressee_storage::Store::open(AppPaths::isolated(fixture.root.path())).unwrap();
    assert_eq!(
        serde_json::to_value(&fixture.evidence).unwrap(),
        serde_json::to_value(
            dom_store
                .project_evidence(fixture.project.project.id)
                .unwrap()
        )
        .unwrap()
    );
}
