// SPDX-License-Identifier: MPL-2.0
//! Settings state/focus checks; simulated GPUI, not OS input qualification.
use super::*;
use crate::workflow_refinement_tests::load_review;
use gpui::TestAppContext;
use regressee_app::product::{RuntimeReadiness, RuntimeState};

pub(super) fn readiness(status: RuntimeState, setup_available: bool) -> RuntimeReadiness {
    RuntimeReadiness {
        status,
        setup_available,
        message: "Synthetic browser status for settings tests.",
        identity: None,
        scope: None,
    }
}

#[gpui::test]
fn browser_settings_explains_waiting_and_preserves_installer_capabilities(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (_, project, _, _, _) = approval_fixture::fixture(dir.path());
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Settings;
            s.runtime_readiness = None;
            s.runtime_status = Some("Checking installed browser integrity…".into());
            let view = s.settings_browser_presentation();
            assert_eq!(view.title, "Checking browser integrity…");
            assert_eq!(view.action, "Checking browser…");
            assert!(view.action_help.contains("until this check finishes"));
            assert!(!s.control_enabled(Control::SetupRuntime, cx));
            let request = s.next_request;
            s.activate_control(Control::SetupRuntime, window, cx);
            assert_eq!(s.next_request, request);
            for (state, available, action) in [
                (RuntimeState::NotInstalled, true, "Install browser now"),
                (RuntimeState::Ready, true, "Repair browser…"),
                (RuntimeState::RepairRequired, true, "Repair browser…"),
                (RuntimeState::UpdateRequired, true, "Update browser…"),
                (
                    RuntimeState::UnsupportedPlatform,
                    false,
                    "Browser setup unavailable",
                ),
                (
                    RuntimeState::UnavailableDistribution,
                    false,
                    "Browser setup unavailable",
                ),
                (
                    RuntimeState::NotInstalled,
                    false,
                    "Browser setup unavailable",
                ),
            ] {
                s.runtime_readiness = Some(readiness(state, available));
                let view = s.settings_browser_presentation();
                assert_eq!(view.action, action);
                assert!(!view.checking);
                assert_eq!(
                    view.primary,
                    matches!(
                        state,
                        RuntimeState::NotInstalled
                            | RuntimeState::UpdateRequired
                            | RuntimeState::RepairRequired
                    )
                );
                assert!(!view.action_help.is_empty());
                assert_eq!(s.control_enabled(Control::SetupRuntime, cx), available);
            }
            s.runtime_readiness = None;
            s.runtime_status =
                Some("Browser availability could not be read: synthetic failure".into());
            assert!(!s.settings_browser_presentation().checking);
            assert!(
                s.settings_browser_presentation()
                    .action_help
                    .contains("reported cause")
            );
            assert_eq!(s.next_request, request);
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn settings_disclosures_remove_hidden_controls_and_retain_network_drafts(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Settings;
            s.summary = Some(store.summary().unwrap());
            s.network_status = Some("Synthetic network settings loaded.".into());
            s.details.clear();
            let request = s.next_request;
            for (detail, children) in [
                (Detail::LocalStorage, vec![Control::ChooseFolder]),
                (
                    Detail::ManagedNetwork,
                    vec![
                        Control::CrawlField(17),
                        Control::CrawlField(18),
                        Control::Guided(41),
                        Control::Details(Detail::Network),
                    ],
                ),
                (
                    Detail::SupportTools,
                    vec![Control::Phase7(0), Control::Phase6(13)],
                ),
            ] {
                for control in &children {
                    assert!(!s.control_mounted(*control), "{control:?}");
                    assert!(!s.control_enabled(*control, cx));
                    s.activate_control(*control, window, cx);
                    assert_eq!(s.next_request, request, "hidden controls must not dispatch");
                }
                s.activate_control(Control::Details(detail), window, cx);
                for control in &children {
                    assert!(s.control_mounted(*control), "{control:?}");
                }
                s.activate_control(Control::Details(detail), window, cx);
                for control in &children {
                    assert!(!s.control_mounted(*control), "{control:?}");
                }
            }
            s.activate_control(Control::Details(Detail::ManagedNetwork), window, cx);
            s.crawl_fields[17].update(cx, |field, cx| {
                field.set_value("https://proxy.example:8443", cx)
            });
            s.activate_control(Control::Details(Detail::ManagedNetwork), window, cx);
            assert_eq!(
                s.crawl_fields[17].read(cx).content.as_ref(),
                "https://proxy.example:8443"
            );
            s.route_form_correction(Control::CrawlField(17));
            assert!(s.control_mounted(Control::CrawlField(17)));
            assert_eq!(s.next_request, request, "disclosures never save or install");
            let handles: Vec<_> = [
                Detail::Browser,
                Detail::LocalStorage,
                Detail::ManagedNetwork,
                Detail::SupportTools,
            ]
            .map(|detail| s.handle(Control::Details(detail), cx))
            .into();
            for (index, handle) in handles.iter().enumerate() {
                assert!(!handles[index + 1..].contains(handle));
            }
        })
    });
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    owned.join_after_ui();
}

#[test]
fn folder_preview_copy_describes_the_actual_no_op() {
    assert_eq!(
        app_settings_ux::FOLDER_PREVIEW_LABEL,
        "Preview a folder path…"
    );
    assert!(
        app_settings_ux::FOLDER_PREVIEW_HELP
            .contains("does not inspect files or move project data")
    );
    assert!(app_settings_ux::FOLDER_PREVIEW_HELP.contains("not available yet"));
}

#[gpui::test]
fn settings_tab_and_reverse_tab_follow_rendered_groups_at_both_sizes(cx: &mut TestAppContext) {
    use crate::phase4_hierarchy_tests::draw;
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::Settings;
        s.project = None;
        s.focus.project_saved = false;
        s.runtime_readiness = Some(readiness(RuntimeState::Ready, true));
        s.summary = Some(store.summary().unwrap());
        s.network_status = Some("Synthetic settings loaded.".into());
        s.notice = None;
        s.error = None;
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for expanded in [false, true] {
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.details.clear();
                    if expanded {
                        for detail in [
                            Detail::Browser,
                            Detail::LocalStorage,
                            Detail::ManagedNetwork,
                            Detail::SupportTools,
                        ] {
                            s.activate_control(Control::Details(detail), window, cx);
                        }
                    }
                    s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                    window.focus(&s.handle(Control::SetupRuntime, cx));
                    cx.notify();
                })
            });
            draw(cx);
            let mut expected = vec![
                Control::SetupRuntime,
                Control::Details(Detail::Browser),
                Control::Details(Detail::LocalStorage),
            ];
            if expanded {
                expected.push(Control::ChooseFolder);
            }
            expected.push(Control::Details(Detail::ManagedNetwork));
            if expanded {
                expected.extend([
                    Control::CrawlField(17),
                    Control::CrawlField(18),
                    Control::Guided(41),
                    Control::Details(Detail::Network),
                ]);
            }
            expected.push(Control::Details(Detail::SupportTools));
            if expanded {
                expected.extend([Control::Phase7(0), Control::Phase6(13)]);
            }
            expected.extend([Control::About, Control::Shortcuts]);
            let request = shell.read_with(cx, |s, cx| {
                let actual: Vec<_> = s
                    .visible_controls()
                    .into_iter()
                    .filter(|control| {
                        s.control_enabled(*control, cx) && !matches!(control, Control::Navigate(_))
                    })
                    .collect();
                assert_eq!(actual, expected);
                s.next_request
            });
            for target in &expected[1..] {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        s.move_focus(false, window, cx);
                        assert_eq!(s.focused(window, cx), Some(*target));
                    })
                });
                draw(cx);
                let bounds = shell.read_with(cx, |s, _| {
                    s.setup_control_bounds(*target).unwrap().get().unwrap()
                });
                crate::phase1_setup_tests::inside(
                    bounds,
                    cx.debug_bounds("screen-scroll").unwrap(),
                );
            }
            for target in expected[..expected.len() - 1].iter().rev() {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        s.move_focus(true, window, cx);
                        assert_eq!(s.focused(window, cx), Some(*target));
                    })
                });
                draw(cx);
                let bounds = shell.read_with(cx, |s, _| {
                    s.setup_control_bounds(*target).unwrap().get().unwrap()
                });
                crate::phase1_setup_tests::inside(
                    bounds,
                    cx.debug_bounds("screen-scroll").unwrap(),
                );
            }
            shell.read_with(cx, |s, _| assert_eq!(s.next_request, request));
        }
    }
    owned.join_after_ui();
}
