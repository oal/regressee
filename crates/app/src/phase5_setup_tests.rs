// SPDX-License-Identifier: MPL-2.0
//! Local controller and actual GPUI paint bounds, not OS input qualification.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::TestAppContext;

fn inside(b: Bounds<Pixels>, clip: Bounds<Pixels>) {
    assert!(
        b.left() >= clip.left()
            && b.right() <= clip.right()
            && b.top() >= clip.top()
            && b.bottom() <= clip.bottom(),
        "{b:?} outside {clip:?}"
    );
}

#[gpui::test]
fn guided_public_save_continues_only_exact_success_and_preserves_newer_drafts(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Access;
            let requests = s.next_request;
            assert!(s.access_can_continue(cx));
            s.activate_control(Control::Save, window, cx);
            assert_eq!(s.focus.setup_stage, SetupStage::Pages);
            assert_eq!(s.next_request, requests);
            s.focus.setup_stage = SetupStage::Access;
            s.name.update(cx, |f, cx| f.set_value("Submitted name", cx));
            s.activate_control(Control::Save, window, cx);
            assert_eq!(s.focus.setup_stage, SetupStage::Access);
            let request = s.pending.unwrap();
            s.apply_update(
                Update {
                    id: request + 100,
                    reply: Reply::ProductFailure("stale".into()),
                },
                cx,
            );
            assert_eq!(s.pending, Some(request));
            s.name.update(cx, |f, cx| f.set_value("Newer name", cx));
            finish_jobs(s, cx);
            assert_eq!(s.name.read(cx).content.as_ref(), "Newer name");
            assert_eq!(s.project.as_ref().unwrap().project.name, "Submitted name");
            assert_eq!(s.focus.setup_stage, SetupStage::Access);
            assert!(s.unsaved(cx));
            assert!(s.pending.is_none());
            s.activate_control(Control::Save, window, cx);
            finish_jobs(s, cx);
            assert_eq!(s.focus.setup_stage, SetupStage::Pages);
            assert!(!s.unsaved(cx));
            s.focus.setup_stage = SetupStage::Website;
            s.address
                .update(cx, |f, cx| f.set_value("invalid.test", cx));
            s.activate_control(Control::Save, window, cx);
            assert!(s.pending.is_none());
            assert_eq!(s.presentation_focus, Some(Control::Address));
            assert_eq!(s.address.read(cx).content.as_ref(), "invalid.test");
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn private_save_and_login_have_separate_boundaries_and_failed_continuations_do_not_visit(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Access;
            s.choose(Choice::LoginRequired, cx);
            assert!(s.unsaved(cx));
            assert!(!s.control_enabled(Control::Guided(31), cx));
            assert!(!s.control_enabled(Control::SetupStage(SetupStage::Pages), cx));
            let request = s.next_request;
            s.activate_control(Control::Guided(31), window, cx);
            assert_eq!(s.next_request, request);
            s.activate_control(Control::Save, window, cx);
            let save = s.pending.unwrap();
            assert!(!s.control_enabled(Control::Guided(31), cx));
            s.apply_update(
                Update {
                    id: save,
                    reply: Reply::ProductFailure("Synthetic failed save".into()),
                },
                cx,
            );
            assert!(s.unsaved(cx));
            assert!(s.setup_save.is_none());
            assert_eq!(s.focus.setup_stage, SetupStage::Access);
            // Drain actual worker success as stale, then reload the saved fixture.
            for _ in 0..4 {
                if let Ok(update) = s.events.recv_timeout(Duration::from_millis(100)) {
                    s.apply_update(update, cx);
                }
            }
            assert!(s.unsaved(cx));
            assert!(s.task.is_some()); // Failure outcome blocks background visits until dismissed.
            s.task = None;
            let env = s.environment_draft.unwrap();
            s.project.as_mut().unwrap().project.environments =
                s.draft_settings.environments.clone();
            s.environment.as_mut().unwrap().environment = env;
            assert!(!s.access_can_continue(cx));
            assert!(
                s.control_enabled(Control::Guided(31), cx),
                "busy={} pending={:?} task={} dirty={} environment={:?}",
                s.focus.busy,
                s.pending,
                s.task.is_some(),
                s.unsaved(cx),
                s.environment.as_ref().map(|e| e.environment)
            );
            s.environment.as_mut().unwrap().login_open = true;
            assert!(s.control_enabled(Control::Guided(32), cx));
            assert!(!s.access_can_continue(cx));
            let session = regressee_core::domain::SessionReference {
                id: regressee_core::domain::SessionId::new(),
                project,
                environment: env,
            };
            s.project.as_mut().unwrap().project.environments[0].session = Some(session.clone());
            s.draft_settings.environments[0].session = Some(session);
            assert!(s.access_can_continue(cx));
            s.activate_control(Control::Save, window, cx);
            assert_eq!(s.focus.setup_stage, SetupStage::Pages);
            // Ordinary existing-project save never authorizes a visit.
            s.focus.setup_stage = SetupStage::Settings;
            s.name
                .update(cx, |f, cx| f.set_value("Explicit settings", cx));
            s.activate_control(Control::Save, window, cx);
            assert!(!s.setup_save.as_ref().unwrap().discover);
            let request = s.pending.unwrap();
            s.apply_update(
                Update {
                    id: request,
                    reply: Reply::ProductFailure("Synthetic settings save failure".into()),
                },
                cx,
            );
            assert_eq!(s.focus.setup_stage, SetupStage::Settings);
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn setup_first_viewport_and_advanced_actual_fields_are_contained_at_both_sizes(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::Setup;
        s.notice = None;
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            s.focus.setup_stage = SetupStage::Access;
            s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
            cx.notify();
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        let clip = shell.read_with(cx, |s, _| s.screen_scroll.bounds());
        inside(
            cx.debug_bounds("choice-Choice(PublicAccess)").unwrap(),
            clip,
        );
        inside(
            cx.debug_bounds("choice-Choice(LoginRequired)").unwrap(),
            clip,
        );
        for stage in [SetupStage::Website, SetupStage::Access, SetupStage::Pages] {
            inside(
                cx.debug_bounds(match stage {
                    SetupStage::Website => "action-SetupStage(Website)",
                    SetupStage::Access => "action-SetupStage(Access)",
                    _ => "action-SetupStage(Pages)",
                })
                .unwrap(),
                Bounds::new(gpui::point(px(0.), px(0.)), size(px(width), px(height))),
            );
        }
        inside(
            cx.debug_bounds("action-Save").unwrap(),
            Bounds::new(gpui::point(px(0.), px(0.)), size(px(width), px(height))),
        );
        shell.update(cx, |s, cx| {
            s.focus.setup_stage = SetupStage::Pages;
            s.focus.advanced = true;
            s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
            cx.notify();
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        inside(
            cx.debug_bounds("setup-capture-plan").unwrap(),
            shell.read_with(cx, |s, _| s.screen_scroll.bounds()),
        );
        for index in [0, 1, 2, 3, 4, 5, 6, 7, 8] {
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.reveal_form_control(Control::CrawlField(index), true, w, cx);
                    cx.notify();
                })
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            cx.run_until_parked();
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            let b = shell.read_with(cx, |s, cx| {
                s.crawl_fields[index].read(cx).rendered_bounds().unwrap()
            });
            assert!(b.size.width >= px(400.), "actual field {index}: {b:?}");
            inside(b, shell.read_with(cx, |s, _| s.screen_scroll.bounds()));
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn explicit_save_and_find_rejects_newer_draft_and_dispatches_only_after_exact_save(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Pages;
            let baseline = s.project.as_ref().unwrap().project.active_baseline;
            s.name
                .update(cx, |f, cx| f.set_value("Discovery submitted", cx));
            s.activate_control(Control::Guided(13), window, cx);
            assert!(s.setup_save.as_ref().unwrap().discover);
            let after_save_request = s.next_request;
            s.name
                .update(cx, |f, cx| f.set_value("Discovery newer", cx));
            finish_jobs(s, cx);
            assert_eq!(s.name.read(cx).content.as_ref(), "Discovery newer");
            assert_eq!(s.next_request, after_save_request);
            assert!(s.task.is_none());
            s.activate_control(Control::Guided(13), window, cx);
            let request = s.pending.unwrap();
            let update = loop {
                let u = s.events.recv_timeout(Duration::from_secs(10)).unwrap();
                if u.id == request && matches!(u.reply, Reply::Project(_)) {
                    break u;
                }
                s.apply_update(u, cx);
            };
            s.apply_update(update, cx);
            assert!(
                s.task
                    .as_ref()
                    .is_some_and(|t| t.spec.kind == regressee_app::task::Kind::Discovery)
            );
            assert_eq!(
                s.project.as_ref().unwrap().project.active_baseline,
                baseline
            );
            finish_jobs(s, cx);
        })
    });
    owned.join_after_ui();
}
