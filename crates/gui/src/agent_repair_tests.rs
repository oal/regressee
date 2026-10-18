// SPDX-License-Identifier: MPL-2.0
//! Correct-expectation regressions for the independent Phase 4 findings.
use super::*;
use crate::{
    agent_control_tests::{bind, pump, state},
    workflow_refinement_tests::{finish_jobs, load_review},
};
use gpui::{EntityInputHandler, Modifiers, TestAppContext, VisualTestContext, point};
use regressee_app::{
    control::{Command, Selection, Status, View},
    engine::{Client, Server},
    protocol::{Operation, Request, Response},
};

fn draw(cx: &mut VisualTestContext) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}

pub(super) fn edge_state(s: &mut Shell, label: &str, cx: &mut Context<Shell>) {
    s.agent = state(
        if label == "long-handoff" {
            Status::Human
        } else {
            Status::Active
        },
        true,
    );
    let session = &mut s.agent.control.as_mut().unwrap().sessions[0];
    session.name = "W".repeat(80);
    if label == "long-handoff" {
        let mut message="Please inspect the saved screenshots and verify the website login before returning control. ".repeat(11);
        message.push_str(&" ".repeat(1024 - message.len()));
        regressee_app::control::validate(&Command::Handoff {
            message: message.clone(),
        })
        .unwrap();
        session.handoff = Some(message);
    }
    s.remember_agent_session();
    if label == "long-failure" {
        s.agent.error = Some(format!(
            "Control failed (scroll for details): {}",
            "Saved revision changed; inspect the named session before retrying. ".repeat(64)
        ));
    }
    if label == "disconnected" {
        s.agent.control = None;
        s.agent.connection_error = true;
        s.agent.error = Some("Engine connection lost. Background jobs were not cancelled.".into());
    }
    if label == "expired" || label == "taken-over" {
        let c = s.agent.control.as_mut().unwrap();
        c.controller = None;
        c.epoch += 1;
        c.sessions[0].status = if label == "expired" {
            Status::Expired
        } else {
            Status::Revoked
        };
        c.sessions[0].follow = None;
        c.sessions[0].remaining_seconds = 0;
    }
    s.error = None;
    s.notice = None;
    s.agent_message = None;
    cx.notify();
}

#[gpui::test]
fn agent_long_handoff_names_errors_and_terminal_states_keep_evidence_and_pinned_actions(
    cx: &mut TestAppContext,
) {
    bind(cx);
    let root = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(root.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(root.path()));
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for label in [
            "long-handoff",
            "long-failure",
            "expired",
            "disconnected",
            "taken-over",
        ] {
            shell.update(cx, |s, cx| edge_state(s, label, cx));
            draw(cx);
            let banner = cx.debug_bounds("agent-control-banner").unwrap();
            assert!(banner.size.height <= px(88.), "{label} {width}: {banner:?}");
            assert!(
                cx.debug_bounds("phase5-before").unwrap().size.height >= px(64.),
                "{label}: evidence must retain the ordinary compact-banner viewport"
            );
            if matches!(label, "long-handoff" | "long-failure") {
                let name = cx.debug_bounds("agent-control-name").unwrap();
                let status = cx.debug_bounds("agent-control-status").unwrap();
                let details = cx.debug_bounds("agent-control-details").unwrap();
                assert!(name.size.height <= px(20.) && status.size.height <= px(20.));
                assert!(
                    name.size.width >= px(200.) && status.size.width >= px(200.),
                    "clipped status text needs a finite nonzero measure, not a percentage under an auto-width flex owner"
                );
                assert!(
                    name.bottom() <= status.top() && status.bottom() <= details.top(),
                    "long name must not wrap into status or request details"
                );
            }
            let action = cx
                .debug_bounds(if matches!(label, "long-handoff" | "long-failure") {
                    "agent-take-control"
                } else {
                    "agent-dismiss-status"
                })
                .unwrap();
            crate::phase2_evidence_tests::inside(action, banner);
            if label == "long-handoff" {
                assert!(
                    shell.read_with(cx, |s, _| s.agent_details_scroll.max_offset().height)
                        > px(22.)
                );
                cx.update(|w, cx| shell.update(cx, |s, _| w.focus(&s.agent_handles[2])));
                cx.simulate_keystrokes("end");
                assert!(
                    shell.read_with(cx, |s, _| s.agent_details_scroll.offset().y) < px(0.),
                    "full request remains keyboard-readable independently of evidence"
                );
                cx.simulate_keystrokes("home");
                crate::phase2_evidence_tests::inside(
                    cx.debug_bounds("agent-follow-return").unwrap(),
                    banner,
                );
            }
            assert!(
                cx.debug_bounds("agent-control-details")
                    .unwrap()
                    .size
                    .height
                    <= px(22.)
            );
        }
    }
    owner.join_after_ui();
}

#[gpui::test]
fn agent_return_cancels_preexisting_scrollbar_callbacks_during_transition_and_active(
    cx: &mut TestAppContext,
) {
    bind(cx);
    let root = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(root.path());
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
    cx.simulate_resize(size(px(820.), px(600.)));
    pump(&shell, cx, |s| {
        s.pending.is_none() && s.agent.frontend.is_some()
    });
    let Response::Credential(c) = client
        .control(Command::Start {
            name: "Held input regression".into(),
            seconds: 90,
        })
        .unwrap()
    else {
        panic!()
    };
    let agent = client.with_credential(c);
    agent
        .execute(
            Operation::SelectAgent(Selection {
                kind: Default::default(),
                project,
                run: None,
                analysis: None,
                key: None,
                view: View::Compare,
            }),
            None,
        )
        .unwrap();
    pump(&shell, cx, |s| s.controlling_agent().is_some());
    cx.update(|w, cx| shell.update(cx, |s, cx| s.follow_agent(&FollowAgent, w, cx)));
    pump(&shell, cx, |s| {
        s.agent_following() && !s.agent_waiting && s.project.is_some()
    });
    agent
        .control(Command::Handoff {
            message: "Inspect project settings".into(),
        })
        .unwrap();
    pump(&shell, cx, |s| {
        s.controlling_agent()
            .is_some_and(|a| a.status == Status::Human)
    });
    shell.update(cx, finish_jobs);
    draw(cx);
    let handle = shell.read_with(cx, |s, _| s.screen_scroll.clone());
    let bounds = cx.debug_bounds("settings-scrollbar").unwrap();
    let overflow = f32::from(handle.max_offset().height);
    assert!(overflow > 100.);
    let geometry = scrollbar::ScrollbarGeometry::new(
        bounds,
        f32::from(bounds.size.height),
        overflow,
        f32::from(handle.offset().y),
    )
    .unwrap();
    // These capture-phase handlers were installed while full human control was
    // active, and already own a held pointer before Return is invoked.
    cx.simulate_mouse_down(
        geometry.thumb.center(),
        MouseButton::Left,
        Modifiers::none(),
    );
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            assert!(s.agent_can_follow(cx));
            s.return_agent(&ReturnToAgent, w, cx);
            assert!(s.agent_locked());
        })
    });
    let before = handle.offset();
    cx.simulate_mouse_move(
        point(bounds.center().x, bounds.bottom() - px(5.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    draw(cx);
    assert_eq!(
        handle.offset(),
        before,
        "retained callback must reject transition input"
    );
    pump(&shell, cx, |s| {
        s.agent_locked()
            && !s.agent.pending
            && !s.agent_waiting
            && s.controlling_agent()
                .is_some_and(|a| a.status == Status::Active)
    });
    let before = handle.offset();
    cx.simulate_mouse_move(
        point(bounds.center().x, bounds.top() + px(30.)),
        MouseButton::Left,
        Modifiers::none(),
    );
    draw(cx);
    assert_eq!(
        handle.offset(),
        before,
        "retained callback must reject acknowledged Active input"
    );
    cx.simulate_mouse_up(bounds.center(), MouseButton::Left, Modifiers::none());
    agent.control(Command::End).unwrap();
    owner.join_after_ui();
    for _ in 0..500 {
        if matches!(client.request(Request::Stop), Ok(Response::Ok)) {
            break;
        }
        std::thread::sleep(Duration::from_millis(10));
    }
    thread.join().unwrap().unwrap();
}

#[gpui::test]
fn agent_retained_viewer_callbacks_recheck_lock_instead_of_old_enabled_state(
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
            s.agent = state(Status::Human, true);
            let zoom_callback = Shell::choose_viewer_zoom;
            let pointer_callback = Shell::viewer_pointer;
            let scroll_callback = Shell::viewer_scroll;
            let key_callback = Shell::viewer_key;
            s.return_agent(&ReturnToAgent, w, cx);
            assert!(s.agent_locked());
            let before = (
                s.zoom,
                s.pan_x,
                s.pan_y,
                s.overlay_fraction,
                s.split_fraction,
                s.next_request,
                s.selected_key.clone(),
            );
            zoom_callback(s, ZoomChoice::Percent(75), cx);
            pointer_callback(s, 0, (40., 40.), (0., 0.), cx);
            pointer_callback(s, 1, (80., 80.), (0., 0.), cx);
            scroll_callback(
                s,
                &gpui::ScrollWheelEvent {
                    delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-100.))),
                    ..Default::default()
                },
                (0., 0.),
                cx,
            );
            key_callback(
                s,
                &gpui::KeyDownEvent {
                    keystroke: gpui::Keystroke::parse("down").unwrap(),
                    is_held: false,
                },
                cx,
            );
            s.guided(46, cx);
            assert_eq!(
                before,
                (
                    s.zoom,
                    s.pan_x,
                    s.pan_y,
                    s.overlay_fraction,
                    s.split_fraction,
                    s.next_request,
                    s.selected_key.clone()
                )
            );
        })
    });
    owner.join_after_ui();
}

#[gpui::test]
fn agent_passive_project_refresh_preserves_same_text_and_empty_preedit_on_all_affected_fields(
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
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Settings;
            let original = s.project.clone().unwrap();
            let mut external = original.clone();
            external.revision += 1;
            external.project.name = "Changed externally".into();
            let fields = std::iter::once(s.name.clone())
                .chain(std::iter::once(s.address.clone()))
                .chain(s.crawl_fields.iter().cloned())
                .chain(std::iter::once(s.timezone.read(cx).test_search_input()))
                .collect::<Vec<_>>();
            for input in fields {
                let text = input.read(cx).content.clone();
                for empty in [false, true] {
                    input.update(cx, |i, cx| {
                        i.set_enabled(true);
                        i.replace_and_mark_text_in_range(
                            if empty {
                                None
                            } else {
                                Some(0..text.encode_utf16().count())
                            },
                            if empty { "" } else { &text },
                            None,
                            w,
                            cx,
                        );
                    });
                    assert!(input.read(cx).composing());
                    assert!(
                        !s.exit_project_dirty(cx),
                        "same visible settings are not a text draft"
                    );
                    s.refresh_saved_project(external.clone(), cx);
                    assert!(
                        input.read(cx).composing(),
                        "passive refresh must not end preedit"
                    );
                    assert_eq!(input.read(cx).content, text);
                    assert_eq!(
                        s.project.as_ref().unwrap().revision,
                        original.revision,
                        "retain the original Save precondition while composing"
                    );
                    input.update(cx, |i, cx| i.set_value(&text, cx));
                }
            }
            // The metadata-only path advances a clean revision without resetting an
            // unchanged editor's selection, touched state or correction metadata.
            s.name.update(cx, |i, cx| {
                i.set_enabled(true);
                i.replace_and_mark_text_in_range(Some(0..0), "", None, w, cx);
                i.unmark_text(w, cx);
                i.touched = true;
                i.correction = Some("retained metadata".into());
            });
            let before = s.name.update(cx, |i, cx| {
                i.selected_text_range(false, w, cx)
                    .map(|s| (s.range, s.reversed))
            });
            let mut same = original;
            same.revision += 1;
            s.refresh_saved_project(same.clone(), cx);
            assert_eq!(s.project.as_ref().unwrap().revision, same.revision);
            assert_eq!(
                s.name.update(cx, |i, cx| i
                    .selected_text_range(false, w, cx)
                    .map(|s| (s.range, s.reversed))),
                before
            );
            assert!(s.name.read(cx).touched);
            assert_eq!(
                s.name.read(cx).correction.as_deref(),
                Some("retained metadata")
            );
        })
    });
    owner.join_after_ui();
}

#[gpui::test]
fn agent_expiry_disconnect_takeover_and_restart_status_persist_until_deliberate_dismissal(
    cx: &mut TestAppContext,
) {
    bind(cx);
    let root = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(root.path()));
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, finish_jobs);
    for terminal in [Status::Expired, Status::Revoked, Status::Ended] {
        shell.update(cx, |s, cx| {
            s.agent = state(Status::Active, true);
            s.remember_agent_session();
            let c = s.agent.control.as_mut().unwrap();
            c.controller = None;
            c.epoch += 1;
            c.sessions[0].status = terminal;
            c.sessions[0].follow = None;
            cx.notify();
        });
        draw(cx);
        assert!(cx.debug_bounds("agent-control-banner").is_some());
        assert!(cx.debug_bounds("agent-dismiss-status").is_some());
        assert!(cx.debug_bounds("agent-input-shield").is_none());
        shell.read_with(cx, |s, _| {
            assert!(!s.agent_locked());
            assert!(
                s.agent_status_text()
                    .unwrap()
                    .contains(if terminal == Status::Expired {
                        "expired"
                    } else if terminal == Status::Revoked {
                        "taken back"
                    } else {
                        "ended"
                    })
            );
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                w.focus(&s.agent_handles[0]);
                s.activate(&Activate, w, cx);
                assert!(
                    !s.agent_banner_visible(),
                    "dismissal must retire the status: {:?}",
                    s.agent_status_text()
                );
            })
        });
        draw(cx);
        shell.read_with(cx, |s, _| assert!(!s.agent_banner_visible()));
    }
    shell.update(cx, |s, cx| {
        s.agent = state(Status::Active, true);
        s.remember_agent_session();
        s.agent.control = None;
        s.agent.connection_error = true;
        s.agent.error = Some("Engine connection lost".into());
        cx.notify();
    });
    draw(cx);
    assert!(cx.debug_bounds("agent-dismiss-status").is_some());
    shell.update(cx, |s, cx| {
        assert!(s.agent_status_text().unwrap().contains("Connection lost"));
        s.dismiss_agent_status(cx);
    });
    draw(cx);
    shell.read_with(cx, |s, _| assert!(!s.agent_banner_visible()));
    shell.update(cx, |s, cx| {
        let mut next = state(Status::Active, false);
        let c = next.control.as_mut().unwrap();
        c.controller = None;
        c.sessions.clear();
        s.agent = next;
        cx.notify();
    });
    draw(cx);
    shell.read_with(cx, |s, _| {
        assert!(!s.agent_locked());
        assert!(s.agent_status_text().unwrap().contains("Engine restarted"));
    });
    owner.join_after_ui();
}
