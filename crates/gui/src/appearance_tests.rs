// SPDX-License-Identifier: MPL-2.0
//! Simulated GPUI input and desktop notifications, not OS preference qualification.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::TestAppContext;

#[gpui::test]
fn appearance_follows_live_desktop_only_in_system_mode_and_preserves_evidence(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    owned.appearance.wait_loaded();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::Settings;
        s.notice = None;
        s.error = None;
    });
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            let settings = serde_json::to_value(&s.draft_settings).unwrap();
            let selected = s.selected_key.clone();
            let viewer = s.viewer.as_ref().unwrap().panes[0][0].clone();
            for appearance in [
                gpui::WindowAppearance::Dark,
                gpui::WindowAppearance::VibrantLight,
            ] {
                s.desktop_appearance_changed(appearance, cx);
                assert_eq!(is_dark(), desktop_is_dark(appearance));
            }
            for preference in [AppAppearance::Dark, AppAppearance::Light] {
                s.activate_control(Control::Appearance(preference), window, cx);
                assert_eq!(is_dark(), preference == AppAppearance::Dark);
                for appearance in [
                    gpui::WindowAppearance::VibrantDark,
                    gpui::WindowAppearance::Light,
                ] {
                    s.desktop_appearance_changed(appearance, cx);
                    assert_eq!(is_dark(), preference == AppAppearance::Dark);
                }
            }
            s.desktop_appearance_changed(gpui::WindowAppearance::Dark, cx);
            s.activate_control(Control::Appearance(AppAppearance::System), window, cx);
            assert!(is_dark());
            assert_eq!(s.appearance.preference, AppAppearance::System);
            assert!(s.pending.is_none() && s.task.is_none());
            assert_eq!(s.selected_key, selected);
            assert!(Arc::ptr_eq(
                &viewer,
                &s.viewer.as_ref().unwrap().panes[0][0]
            ));
            assert_eq!(serde_json::to_value(&s.draft_settings).unwrap(), settings);
            s.activate_control(Control::Appearance(AppAppearance::Light), window, cx);
            assert!(!is_dark());
        })
    });
    owned.join_after_ui();
    assert_eq!(
        regressee_storage::AppPreferences::load(&AppPaths::isolated(dir.path()))
            .unwrap()
            .appearance,
        AppAppearance::Light
    );
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn appearance_choices_follow_painted_order_reveal_and_obey_modal_guards(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    owned.appearance.wait_loaded();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        finish_jobs(s, cx);
        s.focus.screen = Screen::Settings;
    });
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                window.focus(&s.handle(Control::Details(Detail::Browser), cx));
                s.move_focus(false, window, cx);
                assert_eq!(
                    s.focused(window, cx),
                    Some(Control::Appearance(AppAppearance::System))
                );
            })
        });
        for preference in AppAppearance::ALL {
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.reveal_form_control(Control::Appearance(preference), true, window, cx);
                    window.focus(&s.handle(Control::Appearance(preference), cx));
                    s.activate_control(Control::Appearance(preference), window, cx);
                    assert_eq!(s.appearance.preference, preference);
                })
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let selector = match preference {
                AppAppearance::System => "choice-Appearance(System)",
                AppAppearance::Light => "choice-Appearance(Light)",
                AppAppearance::Dark => "choice-Appearance(Dark)",
            };
            let bounds = cx.debug_bounds(selector).unwrap();
            assert!(bounds.size.height >= px(40.));
            crate::phase1_setup_tests::inside(bounds, cx.debug_bounds("screen-scroll").unwrap());
        }
    }
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            s.focus.dialog = true;
            for preference in AppAppearance::ALL {
                assert!(!s.control_enabled(Control::Appearance(preference), cx));
                s.activate_control(Control::Appearance(preference), window, cx);
            }
            assert_eq!(s.appearance.preference, AppAppearance::Dark);
            s.focus.dialog = false;
            s.activate_control(Control::Appearance(AppAppearance::Light), window, cx);
            s.focus.screen = Screen::Projects;
            for preference in AppAppearance::ALL {
                assert!(!s.control_mounted(Control::Appearance(preference)));
            }
        })
    });
    owned.join_after_ui();
}
