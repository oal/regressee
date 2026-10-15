// SPDX-License-Identifier: MPL-2.0
//! One finite two-size dialog inspection; isolated, synthetic offline projects.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review, screenshot_switch_fixture};

#[gpui::test]
#[ignore = "explicit two-size native offscreen dialog inspection"]
fn renderer_capture_consistent_dialogs(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    cx.update(|cx| {
        regressee_app::ui::dialog::install(cx);
        text_input::bind(cx);
        cx.bind_keys([
            KeyBinding::new("escape", Dismiss, Some("Shell")),
            KeyBinding::new("tab", Tab, Some("Shell")),
            KeyBinding::new("shift-tab", TabPrev, Some("Shell")),
            KeyBinding::new("enter", Activate, Some("Shell")),
        ]);
    });
    let full = tempfile::tempdir_in(&root).unwrap();
    let full_project = screenshot_switch_fixture(full.path(), false);
    let partial = tempfile::tempdir_in(&root).unwrap();
    let (partial_store, partial_project, _) =
        crate::approval_fixture::partial_fixture(partial.path());
    drop(partial_store);
    let review = tempfile::tempdir_in(&root).unwrap();
    let (store, review_project, _, analysis, _) = crate::approval_fixture::fixture(review.path());
    let original = serde_json::to_value(store.project_evidence(review_project).unwrap()).unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for scene in [
            "baseline",
            "partial-baseline",
            "discard",
            "quit",
            "about",
            "help",
            "approval",
            "task",
        ] {
            let (path, project) = match scene {
                "baseline" => (full.path(), full_project),
                "partial-baseline" => (partial.path(), partial_project),
                _ => (review.path(), review_project),
            };
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(path));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.error = None;
                    s.notice = None;
                    s.configure_review_geometry(width);
                    match scene {
                        "baseline" | "partial-baseline" => {
                            window.focus(&s.handle(Control::SaveBaseline, cx));
                            assert!(s.control_enabled(Control::SaveBaseline, cx));
                            s.activate_control(Control::SaveBaseline, window, cx);
                            assert!(s.native_prompt);
                        }
                        "discard" => {
                            s.confirm_switch(Control::Navigate(Screen::Projects), window, cx)
                        }
                        "quit" => {
                            s.name.update(cx, |input, cx| {
                                input.set_value("Edited local project", cx)
                            });
                            s.request_exit(window, cx);
                            assert!(s.native_prompt);
                        }
                        "about" => {
                            s.focus.screen = Screen::Settings;
                            s.activate_control(Control::About, window, cx);
                        }
                        "help" => s.show_keyboard_help(window, cx),
                        "approval" => {
                            let key = s.selected_key.clone().unwrap();
                            s.approval_selection.insert(key.clone());
                            s.approval = Some(store.approval_preview(analysis, &[key]).unwrap());
                            s.begin_approval_dialog();
                        }
                        "task" => {
                            s.next_request += 1;
                            s.pending = Some(s.next_request);
                            s.focus.busy = true;
                            s.begin_task(
                                regressee_app::task::Spec::for_work(&Work::RepairPreview).unwrap(),
                            );
                        }
                        _ => unreachable!(),
                    }
                    cx.notify();
                })
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            cx.run_until_parked();
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let selector = match scene {
                "about" | "help" => "information-dialog",
                "approval" => "approval-decision",
                "task" => "task-dialog",
                _ => "confirmation-dialog",
            };
            let panel = cx.debug_bounds(selector).unwrap();
            assert!(panel.left() >= px(16.) && panel.right() <= px(width - 16.));
            assert!(panel.top() >= px(16.) && panel.bottom() <= px(height - 16.));
            capture(
                cx,
                &root,
                &format!("dialogs-{width:.0}x{height:.0}-{scene}"),
            )
            .unwrap();
            if matches!(scene, "baseline" | "partial-baseline" | "discard" | "quit") {
                // Escape is the real custom renderer path, not simulated platform consent.
                cx.simulate_keystrokes("escape");
                cx.run_until_parked();
                shell.read_with(cx, |s, _| assert!(!s.native_prompt));
            }
            shell.update(cx, |s, cx| {
                s.pending = None;
                s.focus.busy = false;
                finish_jobs(s, cx);
            });
            owned.join_after_ui();
        }
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(review_project).unwrap()).unwrap()
    );
}
