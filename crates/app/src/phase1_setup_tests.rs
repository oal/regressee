// SPDX-License-Identifier: MPL-2.0
//! Phase 1 presentation/navigation regressions. GPUI simulation, not OS input.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::TestAppContext;

pub(super) fn inside(b: Bounds<Pixels>, clip: Bounds<Pixels>) {
    assert!(
        b.size.width > px(0.)
            && b.size.height > px(0.)
            && b.left() >= clip.left() - px(1.)
            && b.right() <= clip.right() + px(1.)
            && b.top() >= clip.top() - px(1.)
            && b.bottom() <= clip.bottom() + px(1.),
        "{b:?} outside actual owner {clip:?}"
    );
}

#[gpui::test]
fn progress_location_back_and_future_gates_keep_invalid_drafts(cx: &mut TestAppContext) {
    use regressee_app::ui::setup_progress::StepState;
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                s.focus.screen = Screen::Setup;
                s.focus.setup_stage = SetupStage::Pages;
                s.crawl_fields[4].update(cx, |f, cx| f.set_value("9", cx));
                let request = s.next_request;
                assert!(s.control_enabled(Control::SetupStage(SetupStage::Pages), cx));
                assert_eq!(
                    StepState::at(2, s.focus.setup_stage.index()),
                    StepState::Current
                );
                s.activate_control(Control::Save, w, cx);
                assert_eq!(s.focus.setup_stage, SetupStage::Pages);
                assert_eq!(s.next_request, request);
                assert!(s.control_enabled(Control::SetupStage(SetupStage::Pages), cx));
                s.activate_control(Control::SetupBack, w, cx);
                assert_eq!(s.focus.setup_stage, SetupStage::Access);
                assert!(
                    s.handle(Control::SetupStage(SetupStage::Access), cx)
                        .is_focused(w)
                );
                assert_eq!(s.crawl_fields[4].read(cx).content.as_ref(), "9");
                assert!(!s.control_enabled(Control::SetupStage(SetupStage::Pages), cx));
                s.activate_control(Control::SetupStage(SetupStage::Pages), w, cx);
                assert_eq!(s.focus.setup_stage, SetupStage::Access);
                s.activate_control(Control::SetupBack, w, cx);
                assert_eq!(s.focus.setup_stage, SetupStage::Website);
                assert!(!s.control_mounted(Control::SetupBack));
                assert!(!s.control_enabled(Control::SetupStage(SetupStage::Access), cx));
                assert_eq!(s.next_request, request);
                s.crawl_fields[4].update(cx, |f, cx| f.set_value("", cx));
                assert!(s.control_enabled(Control::SetupStage(SetupStage::Access), cx));
                s.activate_control(Control::SetupStage(SetupStage::Access), w, cx);
                s.activate_control(Control::Save, w, cx);
                assert_eq!(s.focus.setup_stage, SetupStage::Pages);
                assert_eq!(s.next_request, request, "clean Continue must not rewrite");
                s.notice = None;
            })
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        for name in [
            "action-SetupStage(Website)",
            "action-SetupStage(Access)",
            "action-SetupStage(Pages)",
        ] {
            inside(
                cx.debug_bounds(name).unwrap(),
                Bounds::new(gpui::point(px(0.), px(0.)), size(px(width), px(height))),
            );
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn consent_is_distinct_origin_relevant_and_resets_without_visits(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            s.create(&CreateProject, w, cx);
            let request = s.next_request;
            for address in [
                "",
                "https://example.test/",
                "https://example.test/ https://example.test/docs",
            ] {
                s.address.update(cx, |f, cx| f.set_value(address, cx));
                s.refresh_setup_origins(cx);
                assert!(!s.control_mounted(Control::Guided(11)));
                assert!(!s.control_enabled(Control::Guided(11), cx));
            }
            s.name.update(cx, |f, cx| f.set_value("Origin consent", cx));
            s.address.update(cx, |f, cx| {
                f.set_value("https://example.test/ https://second.test/", cx)
            });
            s.refresh_setup_origins(cx);
            assert_eq!(s.setup_origin_count, 2);
            assert!(s.control_enabled(Control::Guided(11), cx));
            s.activate_control(Control::Save, w, cx);
            assert_eq!(s.next_request, request);
            assert!(s.pending.is_none());
            s.activate_control(Control::Guided(11), w, cx);
            assert!(s.origins_confirmed);
            s.address.update(cx, |f, cx| {
                f.set_value("https://example.test/ https://third.test/", cx)
            });
            s.refresh_setup_origins(cx);
            assert!(!s.origins_confirmed && s.approved_origins.is_empty());
            s.activate_control(Control::Save, w, cx);
            assert_eq!(s.next_request, request);
            s.activate_control(Control::Guided(11), w, cx);
            s.activate_control(Control::Save, w, cx);
            finish_jobs(s, cx);
            assert!(s.project.is_some());
            assert_eq!(s.focus.setup_stage, SetupStage::Access);
            assert!(s.task.is_none());
            assert!(s.evidence.latest.is_none());
            assert!(
                s.origins_confirmed,
                "saved hydration must not clear matching consent"
            );
            s.activate_control(Control::SetupBack, w, cx);
            s.name
                .update(cx, |f, cx| f.set_value("Submitted origin settings", cx));
            s.activate_control(Control::Save, w, cx);
            assert!(s.pending.is_some());
            s.address.update(cx, |f, cx| {
                f.set_value("https://example.test/ https://newer.test/", cx)
            });
            finish_jobs(s, cx);
            assert_eq!(
                s.address.read(cx).content.as_ref(),
                "https://example.test/ https://newer.test/"
            );
            assert_eq!(s.focus.setup_stage, SetupStage::Website);
            assert!(
                !s.origins_confirmed && s.approved_origins.is_empty(),
                "save reply cannot reapprove newer origins"
            );
            assert!(s.unsaved(cx) && s.task.is_none());
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn optional_configuration_mount_focus_reveal_and_drafts_are_independent(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                s.focus.screen = Screen::Setup;
                s.focus.setup_stage = SetupStage::Access;
                s.setup_environment_open = false;
                s.focus.advanced = false;
                for c in [
                    Control::Guided(6),
                    Control::Environment(0),
                    Control::Guided(27),
                    Control::Guided(35),
                    Control::Guided(36),
                    Control::CrawlField(12),
                    Control::CrawlField(13),
                ] {
                    assert!(!s.control_mounted(c), "hidden {c:?}");
                    let request = s.next_request;
                    s.activate_control(c, w, cx);
                    assert_eq!(s.next_request, request);
                }
                s.activate_control(Control::EnvironmentOptions, w, cx);
                assert!(s.setup_environment_open);
            })
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        // Reach the real mounted mapping field with shell Tab and its own form clip.
        for pass in 0..3 {
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    let c = Control::CrawlField(13);
                    if pass == 0 {
                        let controls: Vec<_> = s
                            .visible_controls()
                            .into_iter()
                            .filter(|c| s.control_enabled(*c, cx))
                            .collect();
                        let i = controls.iter().position(|control| *control == c).unwrap();
                        w.focus(&s.handle(controls[i - 1], cx));
                        s.move_focus(false, w, cx);
                    }
                    assert!(s.handle(c, cx).is_focused(w));
                    s.reveal_form_control(c, true, w, cx);
                    cx.notify();
                })
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
        }
        shell.read_with(cx, |s, _| {
            inside(
                s.setup_control_bounds(Control::CrawlField(13))
                    .unwrap()
                    .get()
                    .unwrap(),
                s.screen_scroll.bounds(),
            )
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.crawl_fields[13]
                    .update(cx, |f, cx| f.set_value("https://example.test/edited", cx));
                let signature = s.setup_signature(cx);
                let request = s.next_request;
                s.dismiss(&Dismiss, w, cx);
                assert!(!s.setup_environment_open);
                assert!(!s.control_mounted(Control::CrawlField(13)));
                assert!(s.handle(Control::EnvironmentOptions, cx).is_focused(w));
                assert_eq!(s.setup_signature(cx), signature);
                s.activate_control(Control::EnvironmentOptions, w, cx);
                assert_eq!(
                    s.crawl_fields[13].read(cx).content.as_ref(),
                    "https://example.test/edited"
                );
                s.activate_control(Control::EnvironmentOptions, w, cx);
                s.activate_control(Control::Choice(Choice::LoginRequired), w, cx);
                for c in [
                    Control::CrawlField(10),
                    Control::CrawlField(11),
                    Control::CrawlField(12),
                    Control::Choice(Choice::SaveLogin),
                    Control::Choice(Choice::EphemeralLogin),
                ] {
                    assert!(s.control_enabled(c, cx), "required private login {c:?}");
                }
                assert!(!s.control_enabled(Control::Guided(31), cx));
                s.activate_control(Control::SetupBack, w, cx);
                s.focus.setup_stage = SetupStage::Pages;
                assert!(!s.focus.advanced, "Access disclosure must not expand Pages");
                assert!(!s.control_mounted(Control::CrawlField(1)));
                s.activate_control(Control::Guided(6), w, cx);
                assert!(s.control_enabled(Control::CrawlField(1), cx));
                s.dismiss(&Dismiss, w, cx);
                assert!(!s.focus.advanced);
                assert!(s.handle(Control::Guided(6), cx).is_focused(w));
                assert_eq!(s.next_request, request);
                s.focus.setup_stage = SetupStage::Settings;
                s.focus.advanced = true;
                for c in [
                    Control::Environment(0),
                    Control::CrawlField(10),
                    Control::CrawlField(13),
                    Control::CrawlField(16),
                    Control::Guided(35),
                    Control::Guided(36),
                ] {
                    assert!(s.control_mounted(c), "full Settings retains {c:?}");
                }
            })
        });
    }
    owned.join_after_ui();
}

#[test]
fn runtime_copy_covers_all_states_and_development_scope() {
    use regressee_app::product::{RuntimeReadiness, RuntimeState::*};
    let (label, _, help) = runtime_ux::runtime_copy(None);
    assert!(label.contains("Checking") && help.contains("no download"));
    for (state, scope, expected) in [
        (NotInstalled, None, "first use"),
        (
            NotInstalled,
            Some(regressee_runtime::distribution::Scope::DevelopmentLoopback),
            "local fixtures only",
        ),
        (Ready, None, "verified"),
        (
            Ready,
            Some(regressee_runtime::distribution::Scope::DevelopmentLoopback),
            "local fixtures only",
        ),
        (RepairRequired, None, "explicit repair"),
        (UnavailableDistribution, None, "incomplete"),
        (UnsupportedPlatform, None, "unavailable"),
    ] {
        let s = RuntimeReadiness {
            status: state,
            setup_available: matches!(state, NotInstalled | Ready | RepairRequired),
            message: "Synthetic state",
            identity: None,
            scope,
        };
        let (label, _, help) = runtime_ux::runtime_copy(Some(&s));
        assert!(label.contains(expected), "{label}");
        assert!(!help.contains("installs") || state == NotInstalled);
        assert!(
            help.contains("Local saving")
                || help.contains("locally")
                || state == Ready
                || state == NotInstalled
        );
    }
}
