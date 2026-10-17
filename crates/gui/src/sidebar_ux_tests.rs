// SPDX-License-Identifier: MPL-2.0
//! Sidebar identity/mount regressions; simulated GPUI, not OS-input qualification.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::TestAppContext;

#[gpui::test]
fn wrapped_baseline_count_uses_help_leading_without_shrinking_navigation(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.navigation_preference = Some(false);
        s.error = None;
        s.notice = None;
    });
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for count in [1, 25, 1000] {
            shell.update(cx, |s, cx| {
                // Presentation-only count stress; never write fabricated evidence.
                let baseline = s.evidence.baseline.as_mut().unwrap();
                let reference = baseline.references[0].clone();
                baseline.references.resize(count, reference);
                cx.notify();
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            let rail = cx.debug_bounds("project-rail").unwrap();
            let name = cx.debug_bounds("rail-project-name").unwrap();
            let status = cx.debug_bounds("rail-baseline-status").unwrap();
            assert_eq!(name.size.height, px(BODY_LINE_HEIGHT));
            assert_eq!(status.top() - name.bottom(), px(8.));
            assert!(
                status.size.height == px(18.) || status.size.height == px(36.),
                "{count}: {status:?}"
            );
            if count == 25 {
                assert_eq!(status.size.height, px(36.));
            }
            crate::phase1_setup_tests::inside(status, rail);
            for screen in [
                Screen::Projects,
                Screen::History,
                Screen::Setup,
                Screen::Settings,
            ] {
                let action = cx
                    .debug_bounds(format!("action-Navigate({screen:?})").leak())
                    .unwrap();
                // Labels may wrap with the simulated font; keep the existing
                // minimum hit target instead of assuming every label is one line.
                assert!(action.size.height >= px(CONTROL_HEIGHT));
                crate::phase1_setup_tests::inside(action, rail);
            }
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn project_header_has_status_only_for_unsaved_settings(cx: &mut TestAppContext) {
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    let capture_root = std::env::var_os("REGRESSEE_RENDERER_CAPTURE_DIR").map(|_| {
        cx.enable_offscreen_renderer().unwrap();
        crate::ui_renderer_capture::output_root().unwrap()
    });
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for (scene, screen, dirty) in [
            ("app-settings", Screen::Settings, false),
            ("project-settings", Screen::Setup, false),
            ("unsaved-settings", Screen::Setup, true),
            ("review", Screen::Results, false),
        ] {
            // Fresh windows avoid GPUI's retained bounds for unmounted selectors.
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                assert!(!s.unsaved(cx));
                s.focus.screen = screen;
                s.focus.setup_stage = SetupStage::Settings;
                s.navigation_preference = Some(false);
                s.notice = None;
                s.error = None;
                s.summary = Some(store.summary().unwrap());
                s.runtime_readiness = Some(crate::app_settings_tests::readiness(
                    regressee_app::product::RuntimeState::Ready,
                    true,
                ));
                if dirty {
                    s.name
                        .update(cx, |input, cx| input.set_value("Unsaved name", cx));
                }
                assert_eq!(s.unsaved(cx), dirty);
                cx.notify();
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let header = cx.debug_bounds("project-header").unwrap();
            assert!(header.size.height <= px(60.), "{width}: {scene}");
            for selector in ["selected-project-context", "action-ChooseProject"] {
                crate::phase1_setup_tests::inside(cx.debug_bounds(selector).unwrap(), header);
            }
            assert_eq!(cx.debug_bounds("project-header-status").is_some(), dirty);
            if dirty {
                crate::phase1_setup_tests::inside(
                    cx.debug_bounds("project-header-status").unwrap(),
                    header,
                );
            }
            if screen == Screen::Settings {
                assert!(cx.debug_bounds("settings-browser-explanation").is_none());
                assert!(cx.debug_bounds("settings-browser-state").is_some());
                assert!(cx.debug_bounds("settings-browser-action-help").is_some());
            }
            #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
            if let Some(root) = &capture_root {
                crate::ui_renderer_capture::capture(
                    cx,
                    root,
                    &format!("quiet-copy-{width:.0}x{height:.0}-{scene}"),
                )
                .unwrap();
            }
            owned.join_after_ui();
        }
    }
}

#[gpui::test]
fn projects_selected_status_stays_with_project_name(cx: &mut TestAppContext) {
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    let capture_root = std::env::var_os("REGRESSEE_RENDERER_CAPTURE_DIR").map(|_| {
        cx.enable_offscreen_renderer().unwrap();
        crate::ui_renderer_capture::output_root().unwrap()
    });
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let other = Project::draft("Other project".into(), "https://example.org/").unwrap();
    store.save_project(&other, None).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::Projects;
        s.notice = None;
        s.error = None;
    });
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for (scene, name) in [
            ("short", "inorge.net"),
            (
                "long",
                "A very long project name that must leave the selected status visible beside its truncated title",
            ),
        ] {
            let index = shell.update(cx, |s, cx| {
                let index = s
                    .projects
                    .items
                    .iter()
                    .position(|p| p.id == project)
                    .unwrap();
                s.projects.items[index].name = name.into();
                cx.notify();
                index
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            let title = cx
                .debug_bounds(format!("project-name-{index}").leak())
                .unwrap();
            let status = cx.debug_bounds("project-selected-status").unwrap();
            let row = cx
                .debug_bounds(format!("project-row-{index}").leak())
                .unwrap();
            assert!(
                title.size.width > px(50.),
                "{width}: {scene} title collapsed"
            );
            if scene == "short" {
                assert!(title.size.width < px(120.), "short names must not stretch");
            }
            assert_eq!(status.left() - title.right(), px(8.), "{width}: {scene}");
            assert!((f32::from(status.center().y - title.center().y)).abs() < 1.);
            crate::phase1_setup_tests::inside(title, row);
            crate::phase1_setup_tests::inside(status, row);
            #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
            if let Some(root) = &capture_root {
                crate::ui_renderer_capture::capture(
                    cx,
                    root,
                    &format!("project-status-{width:.0}x{height:.0}-{scene}"),
                )
                .unwrap();
            }
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn sidebar_logo_stays_top_and_chooser_has_no_collapse_control(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        finish_jobs(s, cx);
        s.navigation_preference = Some(true);
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        let mut logo = None;
        for screen in Screen::ALL {
            shell.update(cx, |s, cx| {
                s.focus.screen = screen;
                s.configure_review_geometry(width);
                assert!(!s.navigation_collapsed);
                assert!(!s.control_mounted(Control::Guided(53)));
                assert!(!s.control_enabled(Control::Guided(53), cx));
                cx.notify();
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            let bounds = cx.debug_bounds("brand-lockup").unwrap();
            assert_eq!(*logo.get_or_insert(bounds), bounds, "{width}: {screen:?}");
            assert!(bounds.top() < px(40.));
            assert!(cx.debug_bounds("action-Guided(53)").is_none());
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    let requests = s.next_request;
                    s.activate_control(Control::Guided(53), w, cx);
                    assert_eq!(s.next_request, requests);
                    assert_eq!(s.navigation_preference, Some(true));
                    assert!(!s.navigation_collapsed);
                });
            });
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn sidebar_logo_stays_top_with_selected_project_across_sections(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.navigation_preference = Some(false);
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        let mut logo = None;
        for screen in Screen::ALL {
            shell.update(cx, |s, cx| {
                s.focus.screen = screen;
                cx.notify();
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            let bounds = cx.debug_bounds("brand-lockup").unwrap();
            assert_eq!(*logo.get_or_insert(bounds), bounds, "{width}: {screen:?}");
            assert!(bounds.top() < px(40.));
            // The simulated renderer retains selector bounds after unmount;
            // use the shared mount contract for absence, fresh captures for pixels.
            shell.read_with(cx, |s, _| {
                assert_eq!(
                    s.control_mounted(Control::Guided(53)),
                    screen == Screen::Results
                );
            });
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn sidebar_toggle_stays_in_header_and_chooser_restores_navigation(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.navigation_preference = Some(false);
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        let logo = cx.debug_bounds("brand-lockup").unwrap();
        let header = cx.debug_bounds("project-header").unwrap();
        let requests = shell.read_with(cx, |s, _| s.next_request);
        for collapsed in [true, false, true] {
            let toggle = cx.debug_bounds("action-Guided(53)").unwrap();
            crate::phase1_setup_tests::inside(toggle, cx.debug_bounds("project-header").unwrap());
            assert!(toggle.left() >= px(if collapsed { 192. } else { 0. }));
            cx.simulate_click(toggle.center(), gpui::Modifiers::none());
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            assert_eq!(
                cx.debug_bounds("project-header").unwrap().size.height,
                header.size.height
            );
            if !collapsed {
                assert_eq!(cx.debug_bounds("brand-lockup").unwrap(), logo);
            }
            cx.update(|w, cx| {
                shell.read_with(cx, |s, cx| {
                    assert_eq!(s.navigation_collapsed, collapsed);
                    assert_eq!(
                        s.control_mounted(Control::Navigate(Screen::History)),
                        !collapsed
                    );
                    assert!(s.handle(Control::Guided(53), cx).is_focused(w));
                    assert_eq!(s.next_request, requests);
                });
            });
        }
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::ChooseProject, w, cx);
                finish_jobs(s, cx);
                assert!(s.focus.shows_project_picker());
            });
            let _ = w.draw(cx);
        });
        assert_eq!(cx.debug_bounds("brand-lockup").unwrap(), logo);
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                assert!(!s.control_mounted(Control::Guided(53)));
                assert!(!s.navigation_collapsed);
                assert!(s.control_enabled(Control::Navigate(Screen::History), cx));
                s.activate_control(Control::CancelProjectChoice, w, cx);
                finish_jobs(s, cx);
                s.configure_review_geometry(width);
                assert!(
                    s.navigation_collapsed,
                    "chooser must preserve the review preference"
                );
                s.activate_control(Control::Guided(53), w, cx);
            });
            let _ = w.draw(cx);
        });
    }
    owned.join_after_ui();
}
