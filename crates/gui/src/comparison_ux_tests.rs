// SPDX-License-Identifier: MPL-2.0
//! Regression coverage for the user's missing-cause and filled-focus reports.
use super::*;
use crate::workflow_refinement_tests::load_review;
use gpui::{TestAppContext, VisualTestContext};
use regressee_app::analysis_engine::{GateReason, Warning};

pub(super) const WARNINGS: [&str; 3] = [
    "ImagesIncomplete",
    "ResourceFailed",
    "EmbeddedContentBlocked",
];

fn set_unavailable(s: &mut Shell) {
    let key = s.selected_key.as_ref().unwrap();
    s.evidence
        .comparison
        .as_mut()
        .unwrap()
        .revision
        .results
        .iter_mut()
        .find(|r| &r.key == key)
        .unwrap()
        .classification = Classification::NotComparable;
}

#[gpui::test]
fn comparison_causes_are_specific_side_bound_and_shared_with_details(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        assert!(s.comparison_issue().is_none());
        let original = s.evidence.clone();
        set_unavailable(s);
        let warnings: Vec<_> = WARNINGS.iter().map(|w| (*w).to_owned()).collect();
        s.evidence.before.as_mut().unwrap().metadata.warnings = warnings.clone();
        s.evidence.latest.as_mut().unwrap().captures[0]
            .metadata
            .warnings = warnings;
        assert!(s.comparison_issue().is_none());
        let notice = s.capture_notice().unwrap();
        assert!(notice.detail.starts_with("Both screenshots:"));
        assert!(notice.summary.contains("Some content didn’t load"));
        for warning in WARNINGS {
            assert!(notice.detail.contains(capture_warning_label(warning)));
        }
        let summary = s.capture_summary();
        assert_eq!(summary[1].1, notice.detail);
        let diagnostics = s.capture_diagnostics().text();
        assert!(diagnostics.contains(&notice.detail));
        assert!(diagnostics.contains("Saved comparison result (JSON)"));
        assert!(diagnostics.contains("Saved analysis report (JSON)"));
        assert!(s.control_enabled(Control::Phase5(21), cx));
        assert!(s.control_enabled(Control::Phase5(23), cx));

        s.evidence
            .before
            .as_mut()
            .unwrap()
            .metadata
            .warnings
            .clear();
        let after = s.capture_notice().unwrap();
        assert!(after.detail.starts_with("After:"));
        assert!(!after.detail.contains("Before:"));
        s.evidence.before.as_mut().unwrap().metadata.warnings = vec!["Unstable".into()];
        // A stale source must never be presented as this result's Before.
        s.evidence.before.as_mut().unwrap().metadata.key.page =
            regressee_core::domain::PageId::new();
        let notice = s.capture_notice().unwrap();
        assert!(!notice.detail.contains(capture_warning_label("Unstable")));

        for (gate, title) in [
            (GateReason::UnexpectedWidth, "Screenshot widths differ"),
            (GateReason::DeviceScaleMismatch, "Screenshot scales differ"),
            (
                GateReason::MaterialProfileMismatch,
                "Capture settings do not match",
            ),
            (GateReason::AllMasked, "All screenshot content is ignored"),
            (
                GateReason::NonOpaquePixels,
                "Screenshot transparency is unsupported",
            ),
        ] {
            s.evidence = original.clone();
            set_unavailable(s);
            s.evidence
                .comparison
                .as_mut()
                .unwrap()
                .perceptual
                .as_mut()
                .unwrap()
                .reports[0]
                .1
                .gates = vec![gate];
            let issue = s.comparison_issue().unwrap();
            assert_eq!(issue.heading, title);
            assert!(!issue.reason.is_empty() && !issue.recovery.is_empty());
            assert!(!issue.reason.contains("Open Details"));
        }
        s.evidence = original.clone();
        set_unavailable(s);
        s.evidence
            .comparison
            .as_mut()
            .unwrap()
            .perceptual
            .as_mut()
            .unwrap()
            .reports[0]
            .1
            .warnings = vec![Warning::RenderingEnvironmentMismatch];
        assert_eq!(
            s.comparison_issue().unwrap().heading,
            "Capture environments do not match"
        );

        s.evidence = original.clone();
        set_unavailable(s);
        let key = s.selected_key.clone().unwrap();
        s.evidence
            .comparison
            .as_mut()
            .unwrap()
            .perceptual
            .as_mut()
            .unwrap()
            .failures
            .push((
                key,
                "Analysis failed: after PNG: invalid chunk checksum".into(),
            ));
        assert!(
            s.comparison_issue()
                .unwrap()
                .reason
                .contains("invalid chunk checksum")
        );

        s.evidence = original.clone();
        set_unavailable(s);
        s.evidence
            .comparison
            .as_mut()
            .unwrap()
            .perceptual
            .as_mut()
            .unwrap()
            .reports
            .clear();
        assert_eq!(
            s.comparison_issue().unwrap().reason,
            "No reason was recorded for this result."
        );
        s.evidence.latest.as_mut().unwrap().run.targets[0].outcome = TargetOutcome::Failed {
            code: regressee_core::domain::CaptureError::NeedsLogin,
        };
        let issue = s.comparison_issue().unwrap();
        assert_eq!(issue.heading, "Screenshot not captured");
        assert!(issue.recovery.contains("Log in again"));
    });
    owned.join_after_ui();
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
    cx.run_until_parked();
}

/// Runs against a saved fixture analyzed by the actual service with the same
/// warning combination the user's run recorded, not an invented error string.
pub(super) fn exercise_warning_recovery(
    shell: &Entity<Shell>,
    cx: &mut VisualTestContext,
    mut inspect: impl FnMut(&str, &mut VisualTestContext),
) {
    shell.update(cx, |s, cx| {
        s.notice = None;
        s.error = None;
        cx.notify();
    });
    draw(cx);
    shell.read_with(cx, |s, cx| {
        let a = s.evidence.comparison.as_ref().unwrap();
        let p = a.perceptual.as_ref().unwrap();
        assert!(p.failures.is_empty());
        assert!(
            p.reports
                .iter()
                .all(|(_, r)| r.gates.is_empty() && r.warnings.is_empty())
        );
        assert!(s.comparison_issue().is_none());
        let notice = s.capture_notice().unwrap();
        assert!(notice.summary.chars().count() < 80);
        assert!(notice.detail.starts_with("Both screenshots:"));
        for warning in WARNINGS {
            assert!(notice.detail.contains(capture_warning_label(warning)));
        }
        assert_eq!(s.capture_summary()[1].1, notice.detail);
        assert!(s.baseline_update_mounted());
        assert!(s.control_enabled(Control::Phase5(21), cx));
        assert!(s.control_enabled(Control::Phase5(23), cx));
    });
    let owner = cx.debug_bounds("ordinary-review-decisions").unwrap();
    for selector in [
        "review-capture-caution",
        "action-Phase5(21)",
        "action-Phase5(37)",
    ] {
        crate::phase2_evidence_tests::inside(cx.debug_bounds(selector).unwrap(), owner);
    }
    inspect("inline", cx);
    let before = cx.debug_bounds("image-viewer").unwrap();
    let caption = cx.debug_bounds("evidence-caption-0").unwrap();
    let image = cx.debug_bounds("phase5-before").unwrap();
    cx.simulate_click(image.center(), gpui::Modifiers::none());
    draw(cx);
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(s.handle(Control::Viewer, cx).is_focused(w));
        })
    });
    assert_eq!(cx.debug_bounds("image-viewer").unwrap(), before);
    assert_eq!(cx.debug_bounds("evidence-caption-0").unwrap(), caption);
    inspect("pointer-focus", cx);
    cx.update(|w, cx| shell.read_with(cx, |s, cx| w.focus(&s.handle(Control::Guided(54), cx))));
    crate::phase4_hierarchy_tests::tab_to(shell, cx, Control::Viewer);
    inspect("keyboard-focus", cx);
    cx.update(|w, cx| shell.update(cx, |s, cx| s.activate_control(Control::Phase5(36), w, cx)));
    draw(cx);
    shell.read_with(cx, |s, _| assert!(s.capture_details_open));
    inspect("details", cx);
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.dismiss(&Dismiss, w, cx);
            assert!(s.handle(Control::Phase5(36), cx).is_focused(w));
            s.activate_control(Control::Guided(44), w, cx);
        })
    });
    draw(cx);
    inspect("analysis", cx);
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.dismiss(&Dismiss, w, cx);
            s.activate_control(Control::Phase5(21), w, cx);
            crate::workflow_refinement_tests::finish_jobs(s, cx);
            s.notice = None;
            s.activate_control(Control::Phase5(37), w, cx);
            s.analysis_fields[9].update(cx, |f, cx| {
                f.set_value("Expected unavailable external content", cx)
            });
            s.activate_control(Control::Phase5(32), w, cx);
            crate::workflow_refinement_tests::finish_jobs(s, cx);
            assert!(!s.review_note_dirty(cx));
        })
    });
    draw(cx);
    inspect("note-saved", cx);
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.dismiss(&Dismiss, w, cx);
            s.activate_control(Control::Phase5(23), w, cx);
            s.activate_control(Control::Phase5(24), w, cx);
            crate::workflow_refinement_tests::finish_jobs(s, cx);
            assert!(s.focus.approval_dialog);
        })
    });
    draw(cx);
    inspect("baseline-preview", cx);
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.dismiss(&Dismiss, w, cx);
            s.activate_control(Control::Phase5(23), w, cx);
        })
    });
    let saved_comparison = shell.read_with(cx, |s, _| s.evidence.comparison.clone());
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            let _ = w;
            let key = s.selected_key.clone().unwrap();
            set_unavailable(s);
            let result = s
                .evidence
                .comparison
                .as_mut()
                .unwrap()
                .revision
                .results
                .iter_mut()
                .find(|r| r.key == key)
                .unwrap();
            result.eligible_pixels = 0;
            result.changed_pixels = 0;
            s.evidence
                .comparison
                .as_mut()
                .unwrap()
                .perceptual
                .as_mut()
                .unwrap()
                .reports
                .retain(|(k, _)| k != &key);
            s.evidence
                .comparison
                .as_mut()
                .unwrap()
                .perceptual
                .as_mut()
                .unwrap()
                .failures
                .push((
                    key,
                    format!(
                        "Analysis failed: {}End of recorded failure.",
                        "Saved screenshot decoding failed. ".repeat(55)
                    ),
                ));
            cx.notify();
        })
    });
    draw(cx);
    let viewport = cx.debug_bounds("review-issue-scroll").unwrap();
    assert!(viewport.size.height <= px(108.));
    assert!(cx.debug_bounds("queue-beside-viewer").unwrap().size.height > px(100.));
    shell.read_with(cx, |s, _| {
        assert!(
            s.comparison_issue()
                .unwrap()
                .reason
                .contains("End of recorded failure.")
        );
        assert!(s.review_issue_scroll.max_offset().height > px(0.));
    });
    cx.simulate_event(gpui::ScrollWheelEvent {
        position: viewport.center(),
        delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(-10000.))),
        ..Default::default()
    });
    draw(cx);
    shell.read_with(cx, |s, _| {
        assert!(s.review_issue_scroll.offset().y < px(0.))
    });
    inspect("long-failure", cx);
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.evidence.comparison = saved_comparison;
            if s.next_unreviewed_key().is_some() {
                let previous = s.selected_key.clone();
                s.activate_control(Control::Phase5(33), w, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert_ne!(s.selected_key, previous);
            }
            cx.notify();
        })
    });
    draw(cx);
    inspect("next-result", cx);
}

#[gpui::test]
fn comparison_warning_recovery_keeps_copy_and_pointer_keyboard_focus_usable(
    cx: &mut TestAppContext,
) {
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let mut original = store.project_evidence(project).unwrap();
    original.reviews.clear();
    let frozen = serde_json::to_value(original).unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            set_unavailable(s);
            for capture in s
                .evidence
                .latest
                .as_mut()
                .unwrap()
                .captures
                .iter_mut()
                .chain(s.evidence.before.iter_mut())
            {
                capture.metadata.warnings = WARNINGS.iter().map(|w| (*w).to_owned()).collect();
            }
        });
        exercise_warning_recovery(&shell, cx, |_, _| {});
        owned.join_after_ui();
    }
    let mut after = store.project_evidence(project).unwrap();
    assert!(!after.reviews.is_empty());
    after.reviews.clear();
    assert_eq!(frozen, serde_json::to_value(after).unwrap());
}
