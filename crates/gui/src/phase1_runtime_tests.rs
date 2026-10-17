// SPDX-License-Identifier: MPL-2.0
//! Setup observes runtime availability without starting an operation or editing drafts.
use super::*;
use crate::workflow_refinement_tests::finish_jobs;
use gpui::TestAppContext;

#[gpui::test]
fn setup_runtime_read_is_once_and_preserves_newer_drafts_without_visits(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            assert_eq!(s.focus.screen, Screen::Projects);
            assert!(s.runtime_status.is_none() && s.runtime_readiness.is_none());
            assert!(s.control_enabled(Control::NewProject, cx));
            s.activate_control(Control::NewProject, w, cx);
            assert!(s.runtime_pending.is_some() && s.pending.is_none());
            assert!(!s.focus.busy);
            let read = s.runtime_pending.unwrap();
            s.apply_update(
                Update {
                    id: read + 1000,
                    reply: Reply::RuntimeReadiness(regressee_app::product::RuntimeReadiness {
                        status: regressee_app::product::RuntimeState::Ready,
                        setup_available: true,
                        message: "Stale synthetic runtime",
                        identity: None,
                        scope: None,
                    }),
                },
                cx,
            );
            assert_eq!(s.runtime_pending, Some(read));
            assert!(s.runtime_readiness.is_none());
            assert!(
                s.task.is_none(),
                "runtime observation is not an installer operation"
            );
            s.name
                .update(cx, |f, cx| f.set_value("Newer runtime-read draft", cx));
            s.address
                .update(cx, |f, cx| f.set_value("https://example.test/", cx));
            finish_jobs(s, cx);
            assert!(s.runtime_readiness.is_some());
            assert_eq!(
                s.runtime_readiness.as_ref().unwrap().status,
                regressee_app::product::RuntimeState::UnavailableDistribution
            );
            assert_eq!(s.name.read(cx).content.as_ref(), "Newer runtime-read draft");
            assert_eq!(s.address.read(cx).content.as_ref(), "https://example.test/");
            assert!(s.project.is_none() && s.evidence.latest.is_none() && s.task.is_none());
            let request = s.next_request;
            s.refresh_visible(cx);
            assert_eq!(s.next_request, request);
            s.activate_control(Control::Save, w, cx);
            finish_jobs(s, cx);
            assert!(s.project.is_some() && s.evidence.latest.is_none());
            assert!(
                s.task.is_none(),
                "local saving does not start login/discovery"
            );
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn settled_projects_control_and_shortcut_start_once_without_refresh(cx: &mut TestAppContext) {
    cx.update(|cx| {
        cx.bind_keys([KeyBinding::new("ctrl-n", CreateProject, Some("Shell"))]);
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for shortcut in [false, true] {
            let dir = tempfile::tempdir().unwrap();
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|w, cx| {
                let s = Shell::new(cx, jobs, events);
                w.focus(&s.handle(Control::Navigate(Screen::Projects), cx));
                s
            });
            cx.simulate_resize(size(px(width), px(height)));
            let initial = shell.update(cx, |s, cx| {
                finish_jobs(s, cx);
                assert_eq!(s.focus.screen, Screen::Projects);
                assert!(s.projects.items.is_empty() && s.project.is_none());
                assert!(s.pending.is_none() && s.runtime_pending.is_none());
                assert!(s.runtime_status.is_none() && s.runtime_readiness.is_none());
                assert!(s.control_mounted(Control::NewProject));
                assert!(s.control_enabled(Control::NewProject, cx));
                s.next_request
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            if shortcut {
                cx.simulate_keystrokes("ctrl-n");
            } else {
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| s.activate_control(Control::NewProject, w, cx))
                });
            }
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    assert_eq!(s.focus.screen, Screen::Setup);
                    assert_eq!(s.focus.setup_stage, SetupStage::Website);
                    assert_eq!(s.next_request, initial + 1);
                    assert_eq!(s.runtime_pending, Some(initial + 1));
                    assert!(s.runtime_readiness.is_none());
                    assert_eq!(s.runtime_presentation().0, "Checking browser setup");
                    assert!(s.pending.is_none() && !s.focus.busy && s.task.is_none());
                    assert!(s.control_enabled(Control::Save, cx));
                    assert!(s.handle(Control::Name, cx).is_focused(w));
                    s.name
                        .update(cx, |f, cx| f.set_value("Fresh entry draft", cx));
                    s.address
                        .update(cx, |f, cx| f.set_value("https://example.test/", cx));
                })
            });
            for _ in 0..5 {
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                shell.update(cx, |s, cx| s.poll_storage(cx));
            }
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    finish_jobs(s, cx);
                    assert_eq!(
                        s.runtime_presentation().0,
                        "Browser setup incomplete in this build"
                    );
                    assert!(s.runtime_pending.is_none());
                    assert_eq!(s.next_request, initial + 1);
                    assert!(s.project.is_none() && s.environment.is_none());
                    assert!(s.evidence.latest.is_none() && s.task.is_none());
                    assert!(s.handle(Control::Name, cx).is_focused(w));
                    s.navigate(Screen::Projects, w, cx);
                    assert!(s.control_enabled(Control::NewProject, cx));
                    s.activate_control(Control::NewProject, w, cx);
                    assert_eq!(
                        s.next_request,
                        initial + 1,
                        "re-entry cannot schedule another read"
                    );
                    assert_eq!(s.name.read(cx).content.as_ref(), "Fresh entry draft");
                    assert_eq!(s.address.read(cx).content.as_ref(), "https://example.test/");
                    s.activate_control(Control::Save, w, cx);
                    finish_jobs(s, cx);
                    assert!(s.project.is_some() && s.evidence.latest.is_none() && s.task.is_none());
                    assert_eq!(s.focus.setup_stage, SetupStage::Access);
                })
            });
            owned.join_after_ui();
        }
    }
}

#[gpui::test]
fn fresh_entry_runtime_reply_does_not_own_a_concurrent_project_save(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            assert!(s.control_enabled(Control::NewProject, cx));
            s.activate_control(Control::NewProject, w, cx);
            let read = s
                .runtime_pending
                .expect("actual entry must schedule the read");
            s.name
                .update(cx, |f, cx| f.set_value("Submitted project", cx));
            s.address
                .update(cx, |f, cx| f.set_value("https://example.test/", cx));
            assert!(s.control_enabled(Control::Save, cx));
            s.activate_control(Control::Save, w, cx);
            let save = s.pending.unwrap();
            assert_ne!(read, save);
            let submitted_project = s.setup_save.as_ref().unwrap().submitted_project;
            s.name
                .update(cx, |f, cx| f.set_value("Newer project draft", cx));
            let signature = s.setup_signature(cx);
            let focus = w.focused(cx);
            // Receive the real read reply while leaving the real save in flight.
            loop {
                let update = s.events.recv_timeout(Duration::from_secs(10)).unwrap();
                let done = update.id == read && !matches!(update.reply, Reply::Progress { .. });
                assert_ne!(
                    update.id, save,
                    "worker must finish its earlier read before save"
                );
                s.apply_update(update, cx);
                if done {
                    break;
                }
            }
            assert!(s.runtime_readiness.is_some() && s.runtime_pending.is_none());
            assert_eq!(s.pending, Some(save));
            assert!(s.focus.busy && s.project.is_none());
            assert_eq!(
                s.setup_save.as_ref().unwrap().submitted_project,
                submitted_project
            );
            assert_eq!(s.setup_signature(cx), signature);
            assert_eq!(w.focused(cx), focus);
            finish_jobs(s, cx);
            assert_eq!(s.project.as_ref().unwrap().project.id, submitted_project);
            assert_eq!(
                s.project.as_ref().unwrap().project.name,
                "Submitted project"
            );
            assert_eq!(s.name.read(cx).content.as_ref(), "Newer project draft");
            assert_eq!(s.focus.setup_stage, SetupStage::Website);
            assert!(s.unsaved(cx) && s.task.is_none() && s.evidence.latest.is_none());
        })
    });
    owned.join_after_ui();
}
