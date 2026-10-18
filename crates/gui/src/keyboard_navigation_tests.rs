// SPDX-License-Identifier: MPL-2.0
//! Full key press/release regressions. GPUI's simulate_keystrokes sends only
//! KeyDown; a redraw before KeyUp reproduces the native duplicate-click path.
use super::*;
use crate::{phase4_hierarchy_tests::draw, workflow_refinement_tests::finish_jobs};
use gpui::{TestAppContext, VisualTestContext};

pub(super) fn bind(cx: &mut App) {
    text_input::bind(cx);
    cx.bind_keys([
        KeyBinding::new("tab", Tab, Some("Shell")),
        KeyBinding::new("shift-tab", TabPrev, Some("Shell")),
        KeyBinding::new("enter", Activate, Some("Shell")),
        KeyBinding::new("space", Activate, Some("Shell && !TextInput")),
        KeyBinding::new("escape", Dismiss, Some("Shell")),
    ]);
}

pub(super) fn press(cx: &mut VisualTestContext, key: &str) {
    cx.simulate_keystrokes(key);
    draw(cx);
    cx.simulate_event(gpui::KeyUpEvent {
        keystroke: gpui::Keystroke::parse(key).unwrap(),
    });
    draw(cx);
}

#[gpui::test]
fn settings_keyboard_press_release_toggles_once_and_preserves_focus(cx: &mut TestAppContext) {
    cx.update(bind);
    let dir = tempfile::tempdir().unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        owned.appearance.wait_loaded();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                finish_jobs(s, cx);
                s.focus.screen = Screen::Settings;
                s.refresh_visible(cx);
                finish_jobs(s, cx);
                s.runtime_readiness = Some(app_settings_tests::readiness(
                    regressee_app::product::RuntimeState::Ready,
                    true,
                ));
                s.details.clear();
                s.notice = None;
                s.error = None;
                window.focus(&s.handle(Control::SetupRuntime, cx));
                cx.notify();
            });
        });
        draw(cx);
        let requests = shell.read_with(cx, |s, _| s.next_request);
        // Painted order, including children mounted by expansion. Collapse each
        // section before advancing so hidden fields cannot become Tab stops.
        for detail in [
            Detail::Browser,
            Detail::LocalStorage,
            Detail::ManagedNetwork,
            Detail::SupportTools,
        ] {
            if detail == Detail::LocalStorage {
                for preference in AppAppearance::ALL {
                    press(cx, "tab");
                    cx.update(|window, cx| {
                        shell.read_with(cx, |s, cx| {
                            assert_eq!(
                                s.focused(window, cx),
                                Some(Control::Appearance(preference))
                            );
                        })
                    });
                }
                for reduced in [false, true] {
                    press(cx, "tab");
                    cx.update(|window, cx| {
                        shell.read_with(cx, |s, cx| {
                            assert_eq!(
                                s.focused(window, cx),
                                Some(Control::ReducedMotion(reduced))
                            );
                        })
                    });
                }
                // This optional section remains collapsed in this navigation
                // regression; the dedicated integration tests exercise setup.
                press(cx, "tab");
                cx.update(|window, cx| {
                    shell.read_with(cx, |s, cx| {
                        assert_eq!(s.focused(window, cx), Some(Control::Details(Detail::AgentIntegrations)));
                    })
                });
            }
            press(cx, "tab");
            cx.update(|window, cx| {
                shell.read_with(cx, |s, cx| {
                    assert_eq!(s.focused(window, cx), Some(Control::Details(detail)));
                });
            });
            for key in ["enter", "enter", "space", "space"] {
                let was_open = shell.read_with(cx, |s, _| s.details.contains(&detail));
                press(cx, key);
                cx.update(|window, cx| {
                    shell.read_with(cx, |s, cx| {
                        assert_eq!(
                            s.details.contains(&detail),
                            !was_open,
                            "{detail:?}: {key} must toggle exactly once"
                        );
                        assert_eq!(s.focused(window, cx), Some(Control::Details(detail)));
                        assert_eq!(s.next_request, requests, "navigation must not perform work");
                    });
                });
                let selector = match detail {
                    Detail::Browser => "action-Details(Browser)",
                    Detail::LocalStorage => "action-Details(LocalStorage)",
                    Detail::ManagedNetwork => "action-Details(ManagedNetwork)",
                    _ => "action-Details(SupportTools)",
                };
                crate::phase1_setup_tests::inside(
                    cx.debug_bounds(selector).unwrap(),
                    cx.debug_bounds("screen-scroll").unwrap(),
                );
            }
        }
        // Opening a modal on KeyDown must not click its newly focused Close
        // button on KeyUp. Closing must not reopen About on that same release.
        press(cx, "tab");
        press(cx, "enter");
        shell.read_with(cx, |s, _| assert!(s.focus.dialog));
        press(cx, "enter");
        cx.update(|window, cx| {
            shell.read_with(cx, |s, cx| {
                assert!(!s.focus.dialog);
                assert_eq!(s.focused(window, cx), Some(Control::About));
                assert_eq!(s.next_request, requests);
            });
        });
        press(cx, "shift-tab");
        cx.update(|window, cx| {
            shell.read_with(cx, |s, cx| {
                assert_eq!(
                    s.focused(window, cx),
                    Some(Control::Details(Detail::SupportTools))
                );
            });
        });
        owned.join_after_ui();
    }
}

#[gpui::test]
fn keyboard_choices_checks_and_text_keep_one_activation_owner(cx: &mut TestAppContext) {
    cx.update(bind);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            workflow_refinement_tests::load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Pages;
            s.draft_settings.profiles.retain(|p| p.name == "Desktop");
            window.focus(&s.handle(Control::Guided(2), cx));
            cx.notify();
        });
    });
    draw(cx);
    let requests = shell.read_with(cx, |s, _| s.next_request);
    for key in ["enter", "enter", "space", "space"] {
        let checked = shell.read_with(cx, |s, _| {
            s.draft_settings.profiles.iter().any(|p| p.name == "Tablet")
        });
        press(cx, key);
        shell.read_with(cx, |s, _| {
            assert_eq!(
                s.draft_settings.profiles.iter().any(|p| p.name == "Tablet"),
                !checked
            )
        });
    }
    let checkbox = cx.debug_bounds("check-Guided(2)").unwrap();
    cx.simulate_click(checkbox.center(), gpui::Modifiers::none());
    draw(cx);
    shell.read_with(cx, |s, _| {
        assert!(s.draft_settings.profiles.iter().any(|p| p.name == "Tablet"))
    });
    cx.update(|window, cx| {
        shell.read_with(cx, |s, cx| {
            window.focus(&s.handle(Control::Choice(Choice::VisibleScreen), cx))
        });
    });
    press(cx, "space");
    shell.read_with(cx, |s, _| {
        assert!(
            s.draft_settings
                .profiles
                .iter()
                .all(|p| p.extent == CaptureExtent::VisibleScreen)
        )
    });
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            s.focus.setup_stage = SetupStage::Website;
            s.name.update(cx, |input, cx| input.set_value("", cx));
            window.focus(&s.handle(Control::Name, cx));
            cx.notify();
        });
    });
    draw(cx);
    cx.simulate_input("Draft with spaces");
    press(cx, "enter");
    shell.read_with(cx, |s, cx| {
        assert_eq!(s.name.read(cx).content, "Draft with spaces");
        assert_eq!(
            s.next_request, requests,
            "editing and Enter must not save or capture"
        );
    });
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    owned.join_after_ui();
}
