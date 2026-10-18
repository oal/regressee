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
fn browser_integrity_check_keeps_about_and_shortcuts_available(cx: &mut TestAppContext) {
    settings_information_during_browser_check(cx, false);
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "explicit two-size App settings help inspection during browser verification"]
fn renderer_capture_settings_information_during_browser_check(cx: &mut TestAppContext) {
    settings_information_during_browser_check(cx, true);
}

fn settings_information_during_browser_check(cx: &mut TestAppContext, render: bool) {
    use crate::phase4_hierarchy_tests::draw;
    use crate::workflow_refinement_tests::finish_jobs;
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    let capture_root = render.then(|| {
        cx.enable_offscreen_renderer().unwrap();
        crate::ui_renderer_capture::output_root().unwrap()
    });
    #[cfg(not(all(feature = "ui-renderer-capture", target_os = "linux")))]
    assert!(!render);
    let dir = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        finish_jobs(s, cx);
        s.focus.screen = Screen::Settings;
        s.refresh_visible(cx);
        finish_jobs(s, cx);
        assert!(s.project.is_none());
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        let request = cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                // Hold a read in flight so worker timing cannot hide the busy guard.
                s.next_request += 1;
                s.pending = Some(s.next_request);
                s.focus.busy = true;
                s.runtime_readiness = None;
                s.runtime_status = Some("Checking installed browser integrity…".into());
                s.progress = Some((Work::RuntimeStatus.activity_label().into(), 0, None));
                assert!(s.settings_browser_presentation().checking);
                assert!(s.control_enabled(Control::About, cx));
                assert!(s.control_enabled(Control::Shortcuts, cx));
                for control in [
                    Control::SetupRuntime,
                    Control::Guided(41),
                    Control::CaptureBaseline,
                ] {
                    assert!(!s.control_enabled(control, cx));
                }
                window.focus(&s.handle(Control::Navigate(Screen::Settings), cx));
                s.next_request
            })
        });
        // Paint the resized/checking layout before using its control bounds,
        // and allow each focus-reveal scroll to paint before the next Tab.
        draw(cx);
        for (reverse, expected) in [
            (false, Control::About),
            (false, Control::Shortcuts),
            (true, Control::About),
        ] {
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.move_focus(reverse, window, cx);
                    assert_eq!(s.focused(window, cx), Some(expected));
                })
            });
            draw(cx);
        }
        let about = cx.debug_bounds("action-About").unwrap();
        crate::phase1_setup_tests::inside(about, cx.debug_bounds("screen-scroll").unwrap());
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if let Some(root) = &capture_root {
            crate::ui_renderer_capture::capture(
                cx,
                root,
                &format!("settings-check-{width:.0}x{height:.0}-information-actions"),
            )
            .unwrap();
        }
        cx.simulate_click(about.center(), gpui::Modifiers::none());
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if let Some(root) = &capture_root {
            crate::ui_renderer_capture::capture(
                cx,
                root,
                &format!("settings-check-{width:.0}x{height:.0}-about-open"),
            )
            .unwrap();
        }
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert!(s.focus.dialog && !s.help_dialog);
                assert_eq!(s.focused(window, cx), Some(Control::CloseDialog));
                assert_eq!(s.pending, Some(request));
                assert_eq!(s.next_request, request);
                assert!(s.focus.busy);
                s.dismiss(&Dismiss, window, cx);
                assert_eq!(s.focused(window, cx), Some(Control::About));
                s.move_focus(false, window, cx);
                s.activate(&Activate, window, cx);
                assert!(s.focus.dialog && s.help_dialog);
                s.dismiss(&Dismiss, window, cx);
                assert_eq!(s.focused(window, cx), Some(Control::Shortcuts));
                s.native_prompt = true;
                assert!(!s.control_enabled(Control::About, cx));
                assert!(!s.control_enabled(Control::Shortcuts, cx));
                s.native_prompt = false;
                s.move_focus(true, window, cx);
                s.activate(&Activate, window, cx);
                assert!(s.focus.dialog && !s.help_dialog);
                assert_eq!(s.next_request, request);
                s.apply_update(
                    Update {
                        id: request,
                        reply: Reply::RuntimeReadiness(readiness(RuntimeState::Ready, true)),
                    },
                    cx,
                );
                assert!(!s.focus.busy && s.pending.is_none());
                assert!(s.focus.dialog && !s.help_dialog);
                assert_eq!(s.focused(window, cx), Some(Control::CloseDialog));
                s.dismiss(&Dismiss, window, cx);
                assert_eq!(s.focused(window, cx), Some(Control::About));
                assert!(s.control_enabled(Control::About, cx));
                assert!(s.control_enabled(Control::Shortcuts, cx));
            })
        });
    }
    owned.join_after_ui();
}

#[gpui::test]
fn entering_app_settings_loads_inline_without_a_modal_or_focus_jump(cx: &mut TestAppContext) {
    use crate::workflow_refinement_tests::finish_jobs;
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_jobs(s, cx);
                s.navigate(Screen::Settings, window, cx);
            });
        });
        let mut reads = Vec::new();
        for _ in 0..32 {
            let pending = cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    assert!(
                        s.task.is_none(),
                        "Automatic settings reads must not open a task dialog"
                    );
                    assert!(!s.focus.dialog && !s.focus.approval_dialog && !s.native_prompt);
                    assert!(s.presentation_focus.is_none());
                    assert_eq!(
                        s.focused(window, cx),
                        Some(Control::Navigate(Screen::Settings))
                    );
                    let Some(request) = s.pending else {
                        return false;
                    };
                    reads.push(s.progress.as_ref().unwrap().0.clone());
                    assert!(s.control_enabled(Control::About, cx));
                    assert!(s.control_enabled(Control::Shortcuts, cx));
                    let update = s.events.recv_timeout(Duration::from_secs(10)).unwrap();
                    assert_eq!(update.id, request);
                    assert!(!matches!(
                        update.reply,
                        Reply::Failure(_) | Reply::ProductFailure(_)
                    ));
                    s.apply_update(update, cx);
                    true
                })
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            if !pending {
                break;
            }
        }
        shell.read_with(cx, |s, _| {
            assert!(s.pending.is_none() && !s.focus.busy && s.task.is_none());
            assert!(s.summary.is_some() && s.runtime_readiness.is_some());
        });
        for work in [
            Work::NetworkStatus,
            Work::InspectStorage,
            Work::RuntimeStatus,
        ] {
            assert!(reads.iter().any(|label| label == work.activity_label()));
        }
        owned.join_after_ui();
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
                assert_eq!(view.explanation.is_empty(), state == RuntimeState::Ready);
                assert_eq!(s.control_enabled(Control::SetupRuntime, cx), available);
            }
            let mut restricted = readiness(RuntimeState::Ready, true);
            restricted.scope = Some(regressee_runtime::distribution::Scope::DevelopmentLoopback);
            s.runtime_readiness = Some(restricted);
            assert!(
                s.settings_browser_presentation()
                    .explanation
                    .contains("not public websites")
            );
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

#[gpui::test]
fn storage_locations_disclose_real_paths_without_extra_actions_or_side_effects(
    cx: &mut TestAppContext,
) {
    use crate::phase4_hierarchy_tests::draw;
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::Settings;
        s.summary = Some(store.summary().unwrap());
        s.details.clear();
        s.notice = None;
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        let (controls, request) =
            shell.read_with(cx, |s, _| (s.visible_controls(), s.next_request));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Details(Detail::LocalStorage), window, cx);
                assert!(s.details.contains(&Detail::LocalStorage));
                assert_eq!(
                    s.visible_controls(),
                    controls,
                    "paths are information, not actions"
                );
                assert_eq!(s.next_request, request);
                assert!(!s.native_prompt);
                assert!(s.notice.is_none());
            })
        });
        cx.run_until_parked();
        draw(cx);
        assert!(cx.debug_bounds("settings-storage-details").is_some());
        let expanded_network_top = cx.debug_bounds("managed-network-section").unwrap().top();
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Details(Detail::LocalStorage), window, cx);
                assert!(!s.details.contains(&Detail::LocalStorage));
                assert_eq!(s.visible_controls(), controls);
                assert_eq!(s.next_request, request);
            })
        });
        cx.run_until_parked();
        draw(cx);
        // GPUI retains historical debug selectors; verify that the following
        // section actually moves up rather than treating a cached bound as mounted.
        assert!(cx.debug_bounds("managed-network-section").unwrap().top() < expanded_network_top);
    }
    owned.join_after_ui();
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn saved_data_results_remain_attached_and_available_when_tools_collapse(cx: &mut TestAppContext) {
    use crate::phase4_hierarchy_tests::draw;
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::Settings;
        s.repair_preview = Some(store.repair_preview().unwrap());
        s.details.remove(&Detail::SupportTools);
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("portability-preview").is_some());
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert!(s.control_enabled(Control::Phase6(7), cx));
            assert!(s.control_enabled(Control::Details(Detail::Portability), cx));
            s.activate_control(Control::Phase6(7), window, cx);
            assert!(s.repair_preview.is_none());
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn settings_tab_and_reverse_tab_follow_rendered_groups_at_both_sizes(cx: &mut TestAppContext) {
    use crate::phase4_hierarchy_tests::draw;
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    // Match production's pre-window preference readiness, not worker scheduling.
    owned.appearance.wait_loaded();
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
                Control::Appearance(AppAppearance::System),
                Control::Appearance(AppAppearance::Light),
                Control::Appearance(AppAppearance::Dark),
                Control::ReducedMotion(false),
                Control::ReducedMotion(true),
                Control::Details(Detail::LocalStorage),
            ];
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
