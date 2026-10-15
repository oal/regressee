// SPDX-License-Identifier: MPL-2.0
//! Exercise the actual app-owned confirmation renderer, not the platform stub.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{TestAppContext, VisualTestContext};

fn press(cx: &mut VisualTestContext, key: &str) {
    cx.simulate_keystrokes(key);
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
    cx.simulate_event(gpui::KeyUpEvent {
        keystroke: gpui::Keystroke::parse(key).unwrap(),
    });
    cx.update(|window, cx| {
        let _ = window.draw(cx);
    });
    cx.run_until_parked();
}

#[gpui::test]
fn styled_baseline_confirmation_keeps_safe_focus_and_requires_explicit_consent(
    cx: &mut TestAppContext,
) {
    cx.update(regressee_app::ui::dialog::install);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _) = approval_fixture::partial_fixture(dir.path());
    let previous = store.load_project(project).unwrap().project.active_baseline;
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    for key in ["enter", "escape"] {
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                window.focus(&s.handle(Control::SaveBaseline, cx));
                s.activate_control(Control::SaveBaseline, window, cx);
                assert!(s.native_prompt);
                let requests = s.next_request;
                s.activate_control(Control::SaveBaseline, window, cx);
                s.activate_control(Control::Navigate(Screen::Projects), window, cx);
                s.request_exit(window, cx);
                assert_eq!(s.next_request, requests);
                assert_eq!(s.focus.screen, Screen::Results);
            })
        });
        cx.run_until_parked();
        press(cx, key);
        cx.update(|window, cx| {
            shell.read_with(cx, |s, cx| {
                assert!(!s.native_prompt && s.focus.candidate);
                assert_eq!(
                    s.project.as_ref().unwrap().project.active_baseline,
                    previous
                );
                assert!(s.handle(Control::SaveBaseline, cx).is_focused(window));
            })
        });
    }
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            s.activate_control(Control::SaveBaseline, window, cx)
        })
    });
    cx.run_until_parked();
    press(cx, "tab");
    press(cx, "enter");
    shell.update(cx, |s, cx| {
        finish_jobs(s, cx);
        assert!(!s.native_prompt);
        assert!(s.error.is_none(), "{:?}", s.error);
        s.queue(Work::Evidence(project), cx);
        finish_jobs(s, cx);
        assert!(s.capture_is_saved_baseline());
        assert!(s.evidence.baseline.as_ref().unwrap().explicitly_partial);
    });
    owned.join_after_ui();
}
