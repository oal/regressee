// SPDX-License-Identifier: MPL-2.0
//! State-aware mounted controls and actual shell geometry, not OS-input evidence.
use super::*;
use crate::phase2_evidence_tests::inside;
use crate::workflow_refinement_tests::load_review;
use gpui::{TestAppContext, VisualTestContext};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
    cx.run_until_parked();
}

fn click(shell: &Entity<Shell>, cx: &mut VisualTestContext, control: Control) {
    draw(cx);
    let selector = format!("action-{control:?}");
    let bounds = cx.debug_bounds(selector.leak()).expect("painted action");
    shell.read_with(cx, |s, cx| {
        assert!(s.control_enabled(control, cx), "{control:?}")
    });
    cx.simulate_click(bounds.center(), gpui::Modifiers::none());
    draw(cx);
}

pub(super) fn exercise(
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
    let original = shell.read_with(cx, |s, _| s.evidence.clone());
    let current = shell.read_with(cx, |s, _| s.selected_key.clone().unwrap());
    let check = |cx: &mut VisualTestContext, selectors: &[&str]| {
        draw(cx);
        let window = cx.update(|w, _| Bounds::new(gpui::point(px(0.), px(0.)), w.viewport_size()));
        for selector in selectors {
            inside(
                cx.debug_bounds(selector.to_string().leak()).unwrap(),
                window,
            );
        }
        if let Some(footer) = cx.debug_bounds("approval-next-action") {
            inside(
                cx.debug_bounds("baseline-update-explanation").unwrap(),
                footer,
            );
            for selector in ["action-Phase5(23)", "action-Phase5(24)"] {
                if let Some(action) = cx.debug_bounds(selector) {
                    inside(action, footer);
                    assert!(action.size.height >= px(40.));
                }
            }
        }
    };
    check(
        cx,
        &[
            "ordinary-review-decisions",
            "approval-next-action",
            "action-Phase5(20)",
            "action-Phase5(37)",
            "action-Phase5(23)",
        ],
    );
    assert!(cx.debug_bounds("action-Phase5(24)").is_none());
    let ready_height = cx.debug_bounds("queue-beside-viewer").unwrap().size.height;
    inspect("ready", cx);

    // Notes remain explicit, and a draft keeps Next blocked after dismissal.
    click(shell, cx, Control::Phase5(37));
    shell.update(cx, |s, cx| {
        s.analysis_fields[9].update(cx, |f, cx| f.set_value("Retained local draft", cx));
    });
    cx.update(|w, cx| shell.update(cx, |s, cx| s.dismiss(&Dismiss, w, cx)));
    shell.read_with(cx, |s, cx| {
        assert!(s.review_note_dirty(cx));
        assert!(!s.control_enabled(Control::Phase5(33), cx));
    });
    shell.update(cx, |s, cx| {
        s.analysis_fields[9].update(cx, |f, cx| f.set_value("", cx))
    });

    click(shell, cx, Control::Phase5(23));
    check(
        cx,
        &[
            "approval-next-action",
            "action-Phase5(23)",
            "action-Phase5(24)",
        ],
    );
    shell.read_with(cx, |s, cx| {
        assert!(s.approval_selection.contains(&current));
        assert!(s.control_enabled(Control::Phase5(24), cx));
        let order = s.visible_controls();
        let note = order
            .iter()
            .position(|c| *c == Control::Phase5(37))
            .unwrap();
        if let Some(next) = order.iter().position(|c| *c == Control::Phase5(33)) {
            assert!(note < next);
        }
    });
    inspect("selected", cx);
    click(shell, cx, Control::Phase5(24));
    shell.update(cx, crate::workflow_refinement_tests::finish_jobs);
    check(cx, &["approval-decision", "action-Phase5(25)"]);
    shell.read_with(cx, |s, _| {
        assert!(s.focus.approval_dialog);
        assert_eq!(s.approval.as_ref().unwrap().selected, vec![current.clone()]);
        assert_eq!(
            serde_json::to_value(&s.evidence).unwrap(),
            serde_json::to_value(&original).unwrap(),
            "selecting and previewing must not save a baseline",
        );
    });
    inspect("confirmation", cx);
    click(shell, cx, Control::Guided(47));
    shell.read_with(cx, |s, _| {
        assert!(!s.focus.approval_dialog);
        assert!(s.approval_selection.contains(&current));
    });
    click(shell, cx, Control::Phase5(23));

    shell.update(cx, |s, cx| {
        let result = s
            .evidence
            .comparison
            .as_mut()
            .unwrap()
            .revision
            .results
            .iter_mut()
            .find(|r| r.key == current)
            .unwrap();
        result.classification = Classification::NotComparable;
        result.eligible_pixels = 0;
        cx.notify();
    });
    check(cx, &["review-unavailable", "action-ReviewDetails"]);
    shell.read_with(cx, |s, cx| {
        assert!(!s.review_result_comparable());
        assert!(!s.baseline_update_mounted());
        for control in [
            Control::Phase5(20),
            Control::Phase5(21),
            Control::Phase5(22),
            Control::Phase5(23),
            Control::Phase5(24),
            Control::Phase5(37),
        ] {
            assert!(!s.control_mounted(control), "{control:?}");
            assert!(!s.control_enabled(control, cx), "{control:?}");
        }
    });
    inspect("unavailable-mixed", cx);
    click(shell, cx, Control::ReviewDetails);
    shell.read_with(cx, |s, _| assert!(s.capture_details_open));
    cx.update(|w, cx| shell.update(cx, |s, cx| s.dismiss(&Dismiss, w, cx)));
    draw(cx);
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(s.handle(Control::ReviewDetails, cx).is_focused(w));
        })
    });

    shell.update(cx, |s, cx| {
        for result in &mut s.evidence.comparison.as_mut().unwrap().revision.results {
            result.classification = Classification::NotComparable;
            result.eligible_pixels = 0;
        }
        cx.notify();
    });
    check(cx, &["review-unavailable", "action-ReviewDetails"]);
    shell.read_with(cx, |s, _| assert!(!s.control_mounted(Control::Phase5(33))));
    assert!(cx.debug_bounds("queue-beside-viewer").unwrap().size.height > ready_height);
    inspect("unavailable-all", cx);

    shell.update(cx, |s, cx| {
        s.evidence = original.clone();
        s.analysis_due = Some(std::time::Instant::now() + Duration::from_secs(3600));
        cx.notify();
    });
    check(cx, &["ordinary-review-decisions", "review-blocking-reason"]);
    shell.read_with(cx, |s, cx| {
        assert!(!s.control_enabled(Control::Phase5(21), cx))
    });
    shell.read_with(cx, |s, _| assert!(!s.baseline_update_mounted()));
    inspect("pending", cx);

    shell.update(cx, |s, cx| {
        s.analysis_due = None;
        s.approval_selection.insert(current.clone());
        let result = s
            .evidence
            .comparison
            .as_mut()
            .unwrap()
            .revision
            .results
            .iter_mut()
            .find(|r| r.key == current)
            .unwrap();
        result.classification = Classification::NotComparable;
        result.eligible_pixels = 0;
        cx.notify();
    });
    check(
        cx,
        &[
            "approval-next-action",
            "action-Phase5(23)",
            "action-Phase5(24)",
        ],
    );
    shell.read_with(cx, |s, cx| {
        assert!(!s.control_enabled(Control::Phase5(24), cx));
        assert!(s.control_enabled(Control::Phase5(23), cx));
    });
    inspect("stale-selection", cx);
    click(shell, cx, Control::Phase5(23));
    shell.read_with(cx, |s, _| assert!(!s.baseline_update_mounted()));
    shell.update(cx, |s, cx| {
        s.evidence = original;
        cx.notify();
    });
}

#[gpui::test]
fn baseline_update_preview_keeps_multiple_screen_selections_without_saving(
    cx: &mut TestAppContext,
) {
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::tempdir().unwrap();
    let project = crate::workflow_refinement_tests::screenshot_queue_fixture(dir.path(), true, 3);
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.notice = None;
            s.approval_selection = s
                .evidence
                .comparison
                .as_ref()
                .unwrap()
                .revision
                .results
                .iter()
                .filter(|result| s.approval_eligible(&result.key))
                .map(|result| result.key.clone())
                .collect();
            assert_eq!(s.approval_selection.len(), 2);
            cx.notify();
        });
        click(&shell, cx, Control::Phase5(24));
        shell.update(cx, crate::workflow_refinement_tests::finish_jobs);
        draw(cx);
        shell.read_with(cx, |s, _| {
            assert_eq!(s.approval.as_ref().unwrap().selected.len(), 2);
            assert!(s.focus.approval_dialog);
        });
        inside(
            cx.debug_bounds("action-Phase5(25)").unwrap(),
            cx.debug_bounds("approval-decision").unwrap(),
        );
        click(&shell, cx, Control::Guided(47));
        shell.read_with(cx, |s, _| assert_eq!(s.approval_selection.len(), 2));
        owned.join_after_ui();
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
    );
}

#[gpui::test]
fn review_footer_states_preserve_drafts_selection_guards_and_space(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        for navigation_hidden in [false, true] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                s.navigation_preference = Some(navigation_hidden);
                s.configure_review_geometry(width);
            });
            exercise(&shell, cx, |_, _| {});
            owned.join_after_ui();
        }
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
