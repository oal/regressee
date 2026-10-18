// SPDX-License-Identifier: MPL-2.0
//! Retained business feedback and agent chrome share the minimum viewport budget.
use super::*;
use crate::{
    agent_control_tests::{bind, state},
    workflow_refinement_tests::load_review,
};
use gpui::{TestAppContext, VisualTestContext};
use regressee_app::control::Status;

pub(super) const SAVED_NOTICE: &str = "Settings saved on this computer.";
pub(super) fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}

#[derive(Clone, Copy, Debug)]
pub(super) enum FeedbackCase {
    Saved,
    Error,
    Both,
}
impl FeedbackCase {
    pub(super) fn label(self) -> &'static str {
        match self {
            Self::Saved => "saved",
            Self::Error => "error",
            Self::Both => "both",
        }
    }
}
pub(super) fn configure_feedback(
    s: &mut Shell,
    status: Status,
    case: FeedbackCase,
    cx: &mut Context<Shell>,
) {
    // This is the genuine successful-load notice from the verifier's fixture,
    // not a clean-banner substitute or a message manufactured by the renderer.
    assert_eq!(s.notice.as_deref(), Some(SAVED_NOTICE));
    assert!(s.agent_can_follow(cx));
    if matches!(case, FeedbackCase::Error | FeedbackCase::Both) {
        s.error=Some(format!("{}END_OF_ERROR", "Could not refresh saved evidence. Existing screenshots and drafts are kept; inspect the saved result before retrying. ".repeat(12)));
        if matches!(case, FeedbackCase::Error) {
            s.notice = None;
        }
    }
    s.agent = state(status, true);
    if matches!(case, FeedbackCase::Both) && status == Status::Human {
        let message = format!(
            "{}END_OF_REQUEST",
            "Please inspect the exact saved screenshots before returning control. ".repeat(14)
        );
        regressee_app::control::validate(&regressee_app::control::Command::Handoff {
            message: message.clone(),
        })
        .unwrap();
        s.agent.control.as_mut().unwrap().sessions[0].handoff = Some(message);
    }
    s.agent_details_scroll
        .set_offset(gpui::point(px(0.), px(0.)));
    cx.notify();
}

pub(super) fn assert_shared_feedback(
    shell: &Entity<Shell>,
    view: &mut VisualTestContext,
    status: Status,
    case: FeedbackCase,
) -> f32 {
    let banner = view.debug_bounds("agent-control-banner").unwrap();
    let before = view.debug_bounds("phase5-before").unwrap();
    let after = view.debug_bounds("phase5-after").unwrap();
    let reader = view.debug_bounds("agent-control-details").unwrap();
    assert!(banner.size.height <= px(88.));
    assert_eq!(reader.size.height, px(22.));
    assert!(
        before.size.height >= px(64.) && after.size.height >= px(64.),
        "{status:?} {case:?}: retained feedback must not collapse evidence: {before:?} / {after:?}"
    );
    for selector in ["agent-take-control", "action-DismissFeedback"] {
        crate::phase2_evidence_tests::inside(view.debug_bounds(selector).unwrap(), banner);
    }
    if status == Status::Human {
        crate::phase2_evidence_tests::inside(
            view.debug_bounds("agent-follow-return").unwrap(),
            banner,
        );
    }
    shell.read_with(view, |s, cx| {
        assert!(s.agent_shared_feedback());
        assert_eq!(s.agent_locked(), status == Status::Active);
        assert_eq!(
            s.control_enabled(Control::DismissFeedback, cx),
            status == Status::Human,
            "sharing feedback must not relax the input lock"
        );
        assert_eq!(
            s.notice.as_deref(),
            if matches!(case, FeedbackCase::Error) {
                None
            } else {
                Some(SAVED_NOTICE)
            }
        );
        assert_eq!(s.error.is_some(), !matches!(case, FeedbackCase::Saved));
    });
    f32::from(before.size.height)
}

#[gpui::test]
fn agent_retained_feedback_must_not_collapse_handoff_evidence(cx: &mut TestAppContext) {
    bind(cx);
    for appearance in [AppAppearance::Light, AppAppearance::Dark] {
        let root = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(root.path());
        drop(store);
        regressee_storage::AppPreferences { appearance, ..Default::default() }
            .save(&AppPaths::isolated(root.path()))
            .unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(root.path()));
        let owner = jobs.clone();
        owner.appearance.wait_loaded();
        let (shell, view) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        view.simulate_resize(size(px(820.), px(600.)));
        shell.update(view, |s, cx| {
            load_review(s, project, cx);
            assert_eq!(s.notice.as_deref(), Some(SAVED_NOTICE));
            assert!(s.agent_can_follow(cx));
            cx.notify();
        });
        draw(view);
        let without = view.debug_bounds("phase5-before").unwrap();
        assert!(without.size.height >= px(64.));
        shell.update(view, |s, cx| {
            s.agent = state(Status::Human, true);
            assert!(!s.agent_locked());
            assert_eq!(s.notice.as_deref(), Some(SAVED_NOTICE));
            cx.notify();
        });
        draw(view);
        let with = view.debug_bounds("phase5-before").unwrap();
        let banner = view.debug_bounds("agent-control-banner").unwrap();
        eprintln!(
            "retained notice {appearance:?}: no-agent={without:?}; handoff={with:?}; banner={banner:?}"
        );
        assert!(
            with.size.height >= px(64.),
            "retained success feedback must not collapse the human handoff's evidence viewport"
        );
        shell.read_with(view, |s, _| {
            assert_eq!(s.notice.as_deref(), Some(SAVED_NOTICE))
        });
        owner.join_after_ui();
        view.update(|w, _| w.remove_window());
    }
}

#[gpui::test]
fn agent_success_error_feedback_share_banner_budget_and_remain_readable(cx: &mut TestAppContext) {
    bind(cx);
    for appearance in [AppAppearance::Light, AppAppearance::Dark] {
        for status in [Status::Active, Status::Human] {
            for case in [FeedbackCase::Saved, FeedbackCase::Error, FeedbackCase::Both] {
                let root = tempfile::tempdir().unwrap();
                let (store, project, _, _, _) = crate::approval_fixture::fixture(root.path());
                drop(store);
                regressee_storage::AppPreferences { appearance, ..Default::default() }
                    .save(&AppPaths::isolated(root.path()))
                    .unwrap();
                let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(root.path()));
                let owner = jobs.clone();
                owner.appearance.wait_loaded();
                let (shell, view) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                shell.update(view, |s, cx| {
                    load_review(s, project, cx);
                    configure_feedback(s, status, case, cx);
                });
                let evidence = shell.read_with(view, |s, _| {
                    (
                        s.project.as_ref().unwrap().revision,
                        s.evidence.latest.as_ref().unwrap().run.id,
                        s.evidence.comparison.as_ref().unwrap().revision.id,
                        s.selected_key.clone(),
                    )
                });
                for (width, height) in [(820., 600.), (1280., 800.), (1440., 900.)] {
                    view.simulate_resize(size(px(width), px(height)));
                    draw(view);
                    let height = assert_shared_feedback(&shell, view, status, case);
                    eprintln!(
                        "shared feedback {appearance:?} {status:?} {} width={width}: evidence={height}px",
                        case.label()
                    );
                    view.update(|w, cx| shell.update(cx, |s, _| w.focus(&s.agent_handles[2])));
                    view.simulate_keystrokes("end");
                    shell.read_with(view, |s, _| {
                        assert_eq!(
                            s.agent_details_scroll.offset().y,
                            -s.agent_details_scroll.max_offset().height
                        )
                    });
                    view.simulate_keystrokes("home");
                    shell.read_with(view, |s, _| {
                        assert_eq!(s.agent_details_scroll.offset().y, px(0.))
                    });
                    shell.read_with(view, |s, _| {
                        assert_eq!(
                            evidence,
                            (
                                s.project.as_ref().unwrap().revision,
                                s.evidence.latest.as_ref().unwrap().run.id,
                                s.evidence.comparison.as_ref().unwrap().revision.id,
                                s.selected_key.clone()
                            )
                        )
                    });
                }
                if status == Status::Human {
                    let dismiss = view.debug_bounds("action-DismissFeedback").unwrap();
                    view.simulate_click(dismiss.center(), gpui::Modifiers::none());
                    draw(view);
                    shell.read_with(view, |s, _| {
                        assert!(s.error.is_none() && s.notice.is_none());
                        assert!(s.agent_following());
                        assert!(!s.agent_locked());
                    });
                } else {
                    view.update(|w, cx| {
                        shell.update(cx, |s, cx| {
                            s.activate_control(Control::DismissFeedback, w, cx)
                        })
                    });
                    assert_shared_feedback(&shell, view, status, case);
                }
                owner.join_after_ui();
                view.update(|w, _| w.remove_window());
            }
        }
    }
}
