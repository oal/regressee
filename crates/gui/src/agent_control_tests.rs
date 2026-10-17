// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{EntityInputHandler, TestAppContext, VisualTestContext};
use regressee_app::{
    control::*,
    engine::{Client, Server},
    protocol::{Operation, Response},
};

pub(super) fn state(status: Status, following: bool) -> regressee_gui::jobs::AgentState {
    let agent = uuid::Uuid::new_v4();
    let frontend = uuid::Uuid::new_v4();
    regressee_gui::jobs::AgentState {
        frontend: Some(frontend),
        control: Some(regressee_app::control::Snapshot {
            engine: uuid::Uuid::new_v4(),
            epoch: 1,
            controller: Some(agent),
            sessions: vec![Session {
                id: agent,
                name: "Release review".into(),
                status,
                remaining_seconds: 87,
                follow: following.then_some(frontend),
                selection: None,
                selection_revision: 0,
                handoff: (status == Status::Human)
                    .then(|| "Please confirm the website login.".into()),
            }],
        }),
        ..Default::default()
    }
}
pub(super) fn bind(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    cx.update(|cx| {
        cx.bind_keys([
            KeyBinding::new("ctrl-n", CreateProject, Some("Shell")),
            KeyBinding::new("ctrl-s", SaveProject, Some("Shell")),
            KeyBinding::new("ctrl-5", SettingsScreen, Some("Shell")),
            KeyBinding::new("ctrl-shift-t", TakeControl, Some("Shell")),
        ])
    });
}
#[track_caller]
pub(super) fn pump(
    shell: &Entity<Shell>,
    cx: &mut VisualTestContext,
    predicate: impl Fn(&Shell) -> bool,
) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    loop {
        cx.update(|_, cx| shell.update(cx, |s, cx| s.poll_storage(cx)));
        if shell.read_with(cx, |s, _| predicate(s)) {
            break;
        }
        assert!(
            std::time::Instant::now() < deadline,
            "GUI/IPC state did not settle: {:?}",
            shell.read_with(cx, |s, _| (
                s.agent.control.clone(),
                s.agent.error.clone(),
                s.agent.pending,
                s.agent_waiting,
                s.agent_location,
                s.viewer.as_ref().map(|v| v.analysis)
            ))
        );
        std::thread::sleep(Duration::from_millis(10));
    }
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}

#[gpui::test]
fn follow_input_shield_blocks_pointer_shortcuts_focused_ime_and_retains_drafts(
    cx: &mut TestAppContext,
) {
    bind(cx);
    let root = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(root.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(root.path()));
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.agent = state(Status::Active, false);
            assert!(!s.agent_locked());
            s.name
                .update(cx, |i, cx| i.set_value("Retain project draft", cx));
            s.follow_agent(&FollowAgent, w, cx);
            assert!(!s.agent_locked());
            assert_eq!(s.name.read(cx).content.as_ref(), "Retain project draft");
            let name = s.project.as_ref().unwrap().project.name.clone();
            s.name.update(cx, |i, cx| i.set_value(&name, cx));
            s.analysis_fields[9].update(cx, |i, cx| i.set_value("Retain note", cx));
            s.follow_agent(&FollowAgent, w, cx);
            assert!(!s.agent_locked());
            assert_eq!(
                s.analysis_fields[9].read(cx).content.as_ref(),
                "Retain note"
            );
            s.analysis_fields[9].update(cx, |i, cx| i.set_value("", cx));
            s.sync_review_note(cx);
            s.mask_dirty = true;
            s.follow_agent(&FollowAgent, w, cx);
            assert!(!s.agent_locked());
            assert!(s.mask_dirty);
            s.mask_dirty = false;
            s.mask_drag = Some(((1, 1), (4, 4)));
            s.follow_agent(&FollowAgent, w, cx);
            assert!(!s.agent_locked());
            assert!(s.mask_drag.is_some());
            s.mask_drag = None;
            s.analysis_fields[0].update(cx, |i, cx| i.set_value("unfinished policy", cx));
            s.follow_agent(&FollowAgent, w, cx);
            assert!(!s.agent_locked());
            assert_eq!(
                s.analysis_fields[0].read(cx).content.as_ref(),
                "unfinished policy"
            );
            s.set_analysis_fields(cx);
            s.focus.screen = Screen::History;
            s.history_search.update(cx, |i, cx| {
                i.set_enabled(true);
                i.replace_and_mark_text_in_range(None, "日本", None, w, cx);
            });
            assert!(s.history_search.read(cx).composing());
            s.follow_agent(&FollowAgent, w, cx);
            assert!(!s.agent_locked());
            assert!(s.history_search.read(cx).composing());
            s.history_search.update(cx, |i, cx| i.set_value("", cx));
            s.approval_selection.insert(s.selected_key.clone().unwrap());
            s.follow_agent(&FollowAgent, w, cx);
            assert!(!s.agent_locked());
            assert!(!s.approval_selection.is_empty());
            s.approval_selection.clear();
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Settings;
            w.focus(&s.name.read(cx).focus);
            s.agent = state(Status::Active, true);
            cx.notify();
        })
    });
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
    let before = shell.read_with(cx, |s, cx| {
        (
            s.focus.screen,
            s.name.read(cx).content.to_string(),
            s.next_request,
        )
    });
    let sidebar = cx
        .debug_bounds("action-Navigate(Settings)")
        .expect("real mounted navigation control");
    cx.simulate_click(sidebar.center(), gpui::Modifiers::none());
    cx.simulate_keystrokes("ctrl-n ctrl-s ctrl-5 f1 escape");
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.name.update(cx, |i, cx| {
                i.replace_text_in_range(None, "blocked", w, cx);
                i.replace_and_mark_text_in_range(None, "日本", None, w, cx);
            });
            s.navigate(Screen::Projects, w, cx);
            s.show_keyboard_help(w, cx);
            assert_eq!(
                (
                    s.focus.screen,
                    s.name.read(cx).content.to_string(),
                    s.next_request
                ),
                before
            );
            assert!(!s.focus.dialog);
            assert!(!s.control_enabled(Control::Name, cx));
            s.move_focus(false, w, cx);
            assert!(s.agent_handles[0].is_focused(w));
        })
    });
    assert!(cx.debug_bounds("agent-take-control").is_some());
    assert!(cx.debug_bounds("agent-input-shield").is_some());
    let take = cx.debug_bounds("agent-take-control").unwrap();
    cx.simulate_click(take.center(), gpui::Modifiers::none());
    shell.read_with(cx, |s, _| {
        assert!(
            s.agent.pending,
            "banner pointer action remains live above the shield"
        )
    });
    // Full temporary human control, not a second restricted workflow.
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.agent = state(Status::Human, true);
            s.agent_transition = false;
            s.navigate(Screen::Settings, w, cx);
            assert_eq!(s.focus.screen, Screen::Settings);
            s.name.update(cx, |i, cx| i.set_value("Handoff draft", cx));
            s.return_agent(&ReturnToAgent, w, cx);
            assert!(!s.agent_locked());
            assert_eq!(s.name.read(cx).content.as_ref(), "Handoff draft");
        })
    });
    owner.join_after_ui();
}

#[gpui::test]
fn real_engine_gui_opt_in_exact_selection_handoff_return_takeover_and_external_jobs(
    cx: &mut TestAppContext,
) {
    bind(cx);
    let root = tempfile::tempdir().unwrap();
    let (store, project, run, original, masked) = crate::approval_fixture::fixture(root.path());
    let evidence = store.project_evidence_for_run(project, run).unwrap();
    let key = evidence.latest.as_ref().unwrap().run.targets[0].key.clone();
    drop(store);
    let paths = AppPaths::isolated(root.path());
    let server = Server::without_runtime(paths.clone()).unwrap();
    let thread = std::thread::spawn(move || server.run());
    let client = Client::existing(paths).unwrap();
    while client.snapshot().is_err() {
        std::thread::sleep(Duration::from_millis(10));
    }
    let (jobs, events) = Jobs::connect(client.clone());
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    pump(&shell, cx, |s| {
        s.pending.is_none() && s.agent.frontend.is_some()
    });
    let Response::Credential(credential) = client
        .control(Command::Start {
            name: "Exact review".into(),
            seconds: 90,
        })
        .unwrap()
    else {
        panic!()
    };
    let agent = client.with_credential(credential);
    let selected = agent
        .execute(
            Operation::SelectAgent(Selection {
                project,
                run: Some(run),
                analysis: Some(masked),
                key: Some(key.clone()),
                view: View::Split,
            }),
            None,
        )
        .unwrap();
    assert_eq!(selected.state, regressee_app::protocol::JobState::Succeeded);
    pump(&shell, cx, |s| s.controlling_agent().is_some());
    shell.read_with(cx, |s, _| {
        assert_eq!(s.focus.screen, Screen::Projects);
        assert!(s.project.is_none());
        assert!(!s.agent_locked());
    });
    cx.update(|_, cx| {
        shell.update(cx, |s, cx| {
            s.name
                .update(cx, |i, cx| i.set_value("Unsent local project", cx))
        })
    });
    let other_project = Project::draft(
        "External saved project".into(),
        "https://example.test/other",
    )
    .unwrap();
    assert_eq!(
        agent
            .execute(
                Operation::Save {
                    project: Box::new(other_project.clone()),
                    expected_revision: None
                },
                None
            )
            .unwrap()
            .state,
        regressee_app::protocol::JobState::Succeeded
    );
    pump(&shell, cx, |s| {
        s.pending.is_none() && s.projects.items.len() == 2
    });
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            assert_eq!(s.focus.screen, Screen::Projects);
            assert_eq!(s.name.read(cx).content.as_ref(), "Unsent local project");
            s.follow_agent(&FollowAgent, w, cx);
            assert!(!s.agent_locked());
            s.name.update(cx, |i, cx| i.set_value("", cx));
        })
    });
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            s.follow_agent(&FollowAgent, w, cx);
        })
    });
    pump(&shell, cx, |s| {
        s.agent_following()
            && s.viewer_current()
            && s.viewer
                .as_ref()
                .is_some_and(|v| v.analysis == Some(masked))
    });
    shell.read_with(cx, |s, _| {
        assert!(s.agent_locked());
        assert_eq!(s.selected_key, Some(key.clone()));
        assert_eq!(s.project.as_ref().unwrap().project.id, project);
        assert_eq!(s.evidence.latest.as_ref().unwrap().run.id, run);
        assert_eq!(s.viewer_mode, 5);
    });
    for selection in [
        Selection {
            project: other_project.id,
            run: None,
            analysis: None,
            key: None,
            view: View::Compare,
        },
        Selection {
            project,
            run: Some(run),
            analysis: Some(original),
            key: Some(key.clone()),
            view: View::Compare,
        },
        Selection {
            project,
            run: Some(run),
            analysis: Some(masked),
            key: Some(key.clone()),
            view: View::Split,
        },
    ] {
        assert_eq!(
            agent
                .execute(Operation::SelectAgent(selection), None)
                .unwrap()
                .state,
            regressee_app::protocol::JobState::Succeeded
        );
    }
    pump(&shell, cx, |s| {
        s.controlling_agent()
            .is_some_and(|a| a.selection_revision == 4)
            && s.viewer_current()
            && s.viewer
                .as_ref()
                .is_some_and(|v| v.analysis == Some(masked))
    });
    shell.read_with(cx, |s, _| {
        assert_eq!(s.project.as_ref().unwrap().project.id, project);
        assert_eq!(s.selected_key, Some(key.clone()));
        assert_eq!(s.viewer_mode, 5);
    });
    let cached = owner.agent_state().location.unwrap().5;
    std::thread::sleep(Duration::from_millis(150));
    let idle = owner.agent_state().location.unwrap().5;
    assert!(
        Arc::ptr_eq(&cached, &idle),
        "idle heartbeats reuse immutable evidence"
    );
    let snapshot = agent.snapshot().unwrap();
    let running = agent
        .submit(regressee_app::protocol::Submit {
            request: uuid::Uuid::new_v4(),
            engine: snapshot.engine,
            epoch: snapshot.epoch,
            // Give the periodic external-job observer a real observation window,
            // including on a loaded renderer host. This tests visible activity,
            // not an undocumented sub-500-ms UI scheduling guarantee.
            operation: Operation::TestWait { millis: 2000 },
            confirmation: None,
            expected_revision: None,
        })
        .unwrap();
    pump(&shell, cx, |s| {
        s.agent.external.iter().any(|j| j.id == running.id)
    });
    shell.read_with(cx, |s, _| {
        assert!(s.pending.is_none());
        assert!(s.task.is_none());
        assert!(!s.focus.busy);
    });
    agent
        .control(Command::Handoff {
            message: "Check the settings".into(),
        })
        .unwrap();
    pump(&shell, cx, |s| {
        s.controlling_agent()
            .is_some_and(|a| a.status == Status::Human)
    });
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            assert!(!s.agent_locked());
            s.navigate(Screen::Setup, w, cx);
            assert_eq!(s.focus.screen, Screen::Setup);
            finish_jobs(s, cx);
        })
    });
    cx.update(|w,cx|shell.update(cx,|s,cx|{assert!(s.agent_can_follow(cx),"handoff draft flags: project={} builder={} network={} note={} mask={} geometry={} pending={:?}",s.exit_project_dirty(cx),s.exit_builder_dirty(cx),s.exit_network_dirty(cx),s.review_unsaved(cx),s.mask_dirty,s.mask_geometry_dirty(cx),s.pending);s.return_agent(&ReturnToAgent,w,cx);}));
    pump(&shell, cx, |s| {
        !s.agent.pending
            && s.controlling_agent()
                .is_some_and(|a| a.status == Status::Active)
    });
    cx.simulate_keystrokes("ctrl-shift-t");
    pump(&shell, cx, |s| s.controlling_agent().is_none());
    assert_eq!(
        client.wait(running.id).unwrap().state,
        regressee_app::protocol::JobState::Succeeded
    );
    let old_engine = client.snapshot().unwrap().engine;
    assert!(matches!(
        client.request(regressee_app::protocol::Request::Stop),
        Ok(Response::Ok)
    ));
    thread.join().unwrap().unwrap();
    pump(&shell, cx, |s| {
        s.agent.control.is_none() && !s.agent_locked()
    });
    let server = Server::without_runtime(client.paths.clone()).unwrap();
    let thread = std::thread::spawn(move || server.run());
    pump(&shell, cx, |s| {
        s.agent
            .control
            .as_ref()
            .is_some_and(|c| c.engine != old_engine)
            && s.agent.frontend.is_some()
    });
    let Response::Credential(c) = client
        .control(Command::Start {
            name: "New engine session".into(),
            seconds: 90,
        })
        .unwrap()
    else {
        panic!()
    };
    let new_agent = client.with_credential(c);
    pump(&shell, cx, |s| s.controlling_agent().is_some());
    shell.read_with(cx, |s, _| {
        assert!(!s.agent_following());
        assert!(!s.agent_locked());
    });
    new_agent.control(Command::End).unwrap();
    owner.join_after_ui();
    assert!(matches!(
        client.request(regressee_app::protocol::Request::Stop),
        Ok(Response::Ok)
    ));
    thread.join().unwrap().unwrap();
}

#[gpui::test]
fn follow_and_return_decline_incompatible_modal_and_pending_tasks(cx: &mut TestAppContext) {
    let root = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(root.path()));
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            s.agent = state(Status::Active, false);
            s.focus.dialog = true;
            s.follow_agent(&FollowAgent, w, cx);
            assert!(!s.agent_locked());
            assert!(s.focus.dialog);
            s.focus.dialog = false;
            s.pending = Some(999);
            s.follow_agent(&FollowAgent, w, cx);
            assert!(!s.agent_locked());
            assert_eq!(s.pending, Some(999));
            s.pending = None;
            s.agent = state(Status::Human, true);
            s.focus.approval_dialog = true;
            s.return_agent(&ReturnToAgent, w, cx);
            assert!(!s.agent_locked());
            assert!(s.focus.approval_dialog);
            // Immediate takeover is never gated by the incompatible dialog.
            s.take_agent_control(&TakeControl, w, cx);
            assert!(s.agent.pending);
            assert!(s.focus.approval_dialog);
        })
    });
    owner.join_after_ui();
}

#[gpui::test]
fn agent_default_refresh_keeps_recorded_run_result_and_project_draft(cx: &mut TestAppContext) {
    bind(cx);
    let root = tempfile::tempdir().unwrap();
    let (store, project, newest, _, _) = crate::approval_fixture::fixture(root.path());
    let saved = store.load_project(project).unwrap();
    let older = store
        .project_evidence(project)
        .unwrap()
        .baseline
        .unwrap()
        .references[0]
        .run;
    assert_ne!(older, newest);
    let evidence = store.project_evidence_for_run(project, older).unwrap();
    let key = evidence.latest.as_ref().unwrap().run.targets[0].key.clone();
    drop(store);
    let paths = AppPaths::isolated(root.path());
    let server = Server::without_runtime(paths.clone()).unwrap();
    let thread = std::thread::spawn(move || server.run());
    let client = Client::existing(paths).unwrap();
    while client.snapshot().is_err() {
        std::thread::sleep(Duration::from_millis(10));
    }
    let (jobs, events) = Jobs::connect(client.clone());
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    pump(&shell, cx, |s| {
        s.pending.is_none() && s.agent.frontend.is_some()
    });
    cx.update(|_, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            // Synthetic equivalent of an explicitly opened recorded capture. A newer
            // check exists already; a passive update must not substitute it.
            s.evidence = evidence;
            s.history_inspection = Some((project, older));
            s.selected_key = Some(key.clone());
            s.review_drafts = Default::default();
            s.analysis_fields[9].update(cx, |i, cx| i.set_value("", cx));
            s.clear_viewer();
            s.reload_viewer(cx);
            finish_jobs(s, cx);
        })
    });
    let Response::Credential(c) = client
        .control(Command::Start {
            name: "Background saved changes".into(),
            seconds: 90,
        })
        .unwrap()
    else {
        panic!()
    };
    let agent = client.with_credential(c);
    let mut changed = saved.project.clone();
    changed.name = "Saved elsewhere".into();
    let updated = agent
        .execute(
            Operation::Save {
                project: Box::new(changed.clone()),
                expected_revision: Some(saved.revision),
            },
            None,
        )
        .unwrap();
    let Some(Outcome::Project(updated)) = updated.outcome else {
        panic!()
    };
    pump(&shell, cx, |s| {
        s.project
            .as_ref()
            .is_some_and(|p| p.revision == updated.revision)
            && !s.evidence_dirty
            && s.passive_evidence_pending.is_none()
    });
    shell.read_with(cx, |s, _| {
        assert!(!s.agent_following());
        assert_eq!(s.history_inspection, Some((project, older)));
        assert_eq!(s.evidence.latest.as_ref().unwrap().run.id, older);
        assert_eq!(s.selected_key, Some(key.clone()));
        assert_eq!(
            s.viewer.as_ref().unwrap().identity.as_ref().unwrap().run,
            older
        );
    });
    cx.update(|_, cx| {
        shell.update(cx, |s, cx| {
            s.analysis_fields[0].update(cx, |i, cx| i.set_value("unfinished policy", cx));
            s.external_evidence_refresh = true;
            s.evidence_dirty = true;
            assert!(s.refresh_selected_evidence(cx));
            assert!(
                s.passive_evidence_pending.is_none(),
                "raw policy edits defer replacement"
            );
            s.name
                .update(cx, |i, cx| i.set_value("Unsent project draft", cx))
        })
    });
    changed.name = "Another saved name".into();
    let result = agent
        .execute(
            Operation::Save {
                project: Box::new(changed),
                expected_revision: Some(updated.revision),
            },
            None,
        )
        .unwrap();
    let Some(Outcome::Project(latest)) = result.outcome else {
        panic!()
    };
    pump(&shell, cx, |s| {
        owner
            .saved_project(project, updated.revision)
            .is_some_and(|p| p.revision == latest.revision)
            && s.notice.is_some()
    });
    shell.read_with(cx, |s, cx| {
        assert_eq!(
            s.analysis_fields[0].read(cx).content.as_ref(),
            "unfinished policy"
        );
        assert_eq!(s.name.read(cx).content.as_ref(), "Unsent project draft");
        assert_eq!(s.project.as_ref().unwrap().revision, updated.revision);
        assert_eq!(s.history_inspection, Some((project, older)));
        assert_eq!(s.evidence.latest.as_ref().unwrap().run.id, older);
        assert_eq!(s.selected_key, Some(key));
    });
    agent.control(Command::End).unwrap();
    owner.join_after_ui();
    for _ in 0..500 {
        if matches!(
            client.request(regressee_app::protocol::Request::Stop),
            Ok(Response::Ok)
        ) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    thread.join().unwrap().unwrap();
}

#[gpui::test]
fn agent_passive_evidence_refreshes_clean_notes_but_keeps_drafts_and_ime(cx: &mut TestAppContext) {
    bind(cx);
    let root = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(root.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(root.path()));
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            let selection = Selection {
                project,
                run: s.evidence.latest.as_ref().map(|r| r.run.id),
                analysis: s.evidence.comparison.as_ref().map(|a| a.revision.id),
                key: None,
                view: View::Compare,
            };
            let mut evidence = s.evidence.clone();
            evidence
                .reviews
                .push(regressee_core::domain::ReviewDecision {
                    analysis: selection.analysis.unwrap(),
                    key: s.selected_key.clone().unwrap(),
                    state: regressee_core::domain::ReviewState::NeedsFix,
                    note: "Saved in another client".into(),
                    reviewed_at: 1,
                });
            s.next_request += 1;
            let id = s.next_request;
            s.passive_evidence_pending = Some((id, selection.clone(), s.data_refresh_generation));
            s.apply_update(
                Update {
                    id,
                    reply: Reply::Evidence(Box::new(evidence.clone())),
                },
                cx,
            );
            assert_eq!(
                s.analysis_fields[9].read(cx).content.as_ref(),
                "Saved in another client"
            );
            assert!(s.pending_review_notes(cx).unwrap().is_empty());
            assert_eq!(s.evidence.latest.as_ref().map(|r| r.run.id), selection.run);
            evidence.reviews[0].note = "A newer saved note".into();
            s.analysis_fields[9].update(cx, |i, cx| i.set_value("Unsent human note", cx));
            s.next_request += 1;
            let id = s.next_request;
            s.passive_evidence_pending = Some((id, selection.clone(), s.data_refresh_generation));
            s.apply_update(
                Update {
                    id,
                    reply: Reply::Evidence(Box::new(evidence.clone())),
                },
                cx,
            );
            assert_eq!(
                s.analysis_fields[9].read(cx).content.as_ref(),
                "Unsent human note"
            );
            assert_eq!(s.evidence.reviews[0].note, "Saved in another client");
            s.analysis_fields[9].update(cx, |i, cx| {
                i.set_value("Saved in another client", cx);
                i.set_enabled(true);
                // A zero-length IME composition still owns the field even though its
                // visible text is exactly the saved note.
                i.replace_and_mark_text_in_range(None, "", None, w, cx);
            });
            assert!(s.analysis_fields[9].read(cx).composing());
            s.next_request += 1;
            let id = s.next_request;
            s.passive_evidence_pending = Some((id, selection, s.data_refresh_generation));
            s.apply_update(
                Update {
                    id,
                    reply: Reply::Evidence(Box::new(evidence)),
                },
                cx,
            );
            assert!(s.analysis_fields[9].read(cx).composing());
            assert_eq!(s.evidence.reviews[0].note, "Saved in another client");
        })
    });
    owner.join_after_ui();
}

#[gpui::test]
fn agent_passive_history_keeps_page_ids_and_never_substitutes_a_missing_row(
    cx: &mut TestAppContext,
) {
    bind(cx);
    let root = tempfile::tempdir().unwrap();
    let (mut store, project, run, _, _) = crate::approval_fixture::fixture(root.path());
    let saved = store.load_project(project).unwrap();
    let template = store.load_run(run).unwrap();
    // Small metadata-only corpus crossing the real 64-row History boundary.
    // All fixture seeding ends before the engine owns the store.
    for _ in 0..65 {
        let mut record = template.clone();
        record.run.id = regressee_core::domain::RunId::new();
        record.run.state = JobState::Running;
        record.run.finished_at = None;
        record.capture = None;
        record.captures.clear();
        for target in &mut record.run.targets {
            target.outcome = TargetOutcome::Pending;
            target.final_url = None;
        }
        store.begin_run(&record).unwrap();
        record.run.state = JobState::Completed;
        record.run.finished_at = Some(102);
        record.run.targets = template.run.targets.clone();
        record.captures = template.captures.clone();
        store.finish_run(&record).unwrap();
    }
    let history = store.history(project, 1).unwrap();
    assert_eq!(history.runs.len(), 3);
    assert!(history.baselines.is_empty());
    let selected = history.runs[1].run.id;
    drop(store);
    let paths = AppPaths::isolated(root.path());
    let server = Server::without_runtime(paths.clone()).unwrap();
    let thread = std::thread::spawn(move || server.run());
    let client = Client::existing(paths).unwrap();
    while client.snapshot().is_err() {
        std::thread::sleep(Duration::from_millis(10));
    }
    let (jobs, events) = Jobs::connect(client.clone());
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    pump(&shell, cx, |s| {
        s.pending.is_none() && s.agent.frontend.is_some()
    });
    cx.update(|_, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::History;
            s.queue(Work::History { project, page: 1 }, cx);
            finish_jobs(s, cx);
            s.run_history_index = 1;
            s.history_index = usize::MAX;
        })
    });
    let selected_result = shell.read_with(cx, |s, _| s.evidence.latest.as_ref().unwrap().run.id);
    let Response::Credential(credential) = client
        .control(Command::Start {
            name: "History refresh".into(),
            seconds: 90,
        })
        .unwrap()
    else {
        panic!()
    };
    let agent = client.with_credential(credential);
    let mut changed = saved.project;
    changed.name = "History settings changed elsewhere".into();
    let result = agent
        .execute(
            Operation::Save {
                project: Box::new(changed),
                expected_revision: Some(saved.revision),
            },
            None,
        )
        .unwrap();
    let Some(Outcome::Project(updated)) = result.outcome else {
        panic!()
    };
    pump(&shell, cx, |s| {
        s.project
            .as_ref()
            .is_some_and(|p| p.revision == updated.revision)
            && !s.evidence_dirty
            && !s.history_dirty
            && s.passive_history_pending.is_none()
            && s.passive_evidence_pending.is_none()
    });
    cx.update(|_, cx| {
        shell.update(cx, |s, cx| {
            assert_eq!(s.focus.screen, Screen::History);
            assert!(!s.agent_following());
            assert_eq!(s.history.as_ref().unwrap().page, 1);
            assert_eq!(
                s.history.as_ref().unwrap().runs[s.run_history_index].run.id,
                selected
            );
            assert!(
                s.evidence.baselines.is_empty(),
                "an exact evidence read must not replace History's separate page with page 0"
            );
            assert_eq!(s.evidence.latest.as_ref().unwrap().run.id, selected_result);
            // Late independent reads never retire another request or its busy/task state.
            let mut removed = s.history.as_ref().unwrap().clone();
            removed.runs.retain(|r| r.run.id != selected);
            s.pending = Some(9000);
            s.focus.busy = true;
            s.passive_history_pending = Some((9001, project, 1, s.data_refresh_generation));
            s.apply_update(
                Update {
                    id: 9001,
                    reply: Reply::History {
                        history: Box::new(removed.clone()),
                    },
                },
                cx,
            );
            assert_eq!(s.pending, Some(9000));
            assert!(s.focus.busy);
            assert_eq!(
                s.history.as_ref().unwrap().runs[s.run_history_index].run.id,
                selected
            );
            s.pending = None;
            s.focus.busy = false;
            s.passive_history_pending = Some((
                9002,
                project,
                1,
                s.data_refresh_generation.saturating_sub(1),
            ));
            s.apply_update(
                Update {
                    id: 9002,
                    reply: Reply::History {
                        history: Box::new(removed.clone()),
                    },
                },
                cx,
            );
            assert_eq!(
                s.history.as_ref().unwrap().runs[s.run_history_index].run.id,
                selected
            );
            s.passive_history_pending = Some((9003, project, 1, s.data_refresh_generation));
            s.apply_update(
                Update {
                    id: 9003,
                    reply: Reply::History {
                        history: Box::new(removed.clone()),
                    },
                },
                cx,
            );
            assert!(
                s.history
                    .as_ref()
                    .unwrap()
                    .runs
                    .get(s.run_history_index)
                    .is_none()
            );
            s.receive_history(removed.clone(), true);
            assert!(
                s.history
                    .as_ref()
                    .unwrap()
                    .runs
                    .get(s.run_history_index)
                    .is_none()
            );
            s.receive_history(removed, false);
            assert_eq!(
                s.run_history_index, 0,
                "explicit navigation keeps the ordinary first-row default"
            );
        })
    });
    agent.control(Command::End).unwrap();
    owner.join_after_ui();
    for _ in 0..500 {
        if matches!(
            client.request(regressee_app::protocol::Request::Stop),
            Ok(Response::Ok)
        ) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    thread.join().unwrap().unwrap();
}
