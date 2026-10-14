// SPDX-License-Identifier: MPL-2.0
//! Sidebar identity/mount regressions; simulated GPUI, not OS-input qualification.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::TestAppContext;

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
