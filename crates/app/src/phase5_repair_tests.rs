// SPDX-License-Identifier: MPL-2.0
//! Finite V5 functional repair. Real local workers/storage, simulated GPUI input.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{EntityInputHandler, TestAppContext};

fn inside(b: Bounds<Pixels>, c: Bounds<Pixels>) {
    assert!(
        b.size.width > px(0.)
            && b.size.height > px(0.)
            && b.left() >= c.left() - px(1.)
            && b.right() <= c.right() + px(1.)
            && b.top() >= c.top() - px(1.)
            && b.bottom() <= c.bottom() + px(1.),
        "{b:?} outside CURRENT clip {c:?}"
    );
}

#[gpui::test]
fn repair_v5_invalid_and_composing_before_continue_or_find_never_queue(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            for discover in [false, true] {
                let control = if discover {
                    Control::Guided(13)
                } else {
                    Control::Save
                };
                s.focus.setup_stage = if discover {
                    SetupStage::Pages
                } else {
                    SetupStage::Access
                };
                let requests = s.next_request;
                s.crawl_fields[10].update(cx, |f, cx| f.set_value("invalid-login", cx));
                s.activate_control(control, w, cx);
                assert_eq!(s.focus.setup_stage, SetupStage::Access);
                assert_eq!(s.next_request, requests);
                assert!(s.pending.is_none());
                assert!(s.setup_save.is_none());
                s.crawl_fields[10].update(cx, |f, cx| {
                    f.set_value("", cx);
                    f.enabled = true;
                    f.replace_and_mark_text_in_range(None, "", None, w, cx);
                    assert!(f.composing());
                });
                s.focus.setup_stage = if discover {
                    SetupStage::Pages
                } else {
                    SetupStage::Access
                };
                let stage = s.focus.setup_stage;
                s.activate_control(control, w, cx);
                assert_eq!(s.focus.setup_stage, stage);
                assert_eq!(s.next_request, requests);
                assert!(s.pending.is_none());
                s.crawl_fields[10].update(cx, |f, cx| {
                    f.unmark_text(w, cx);
                    f.set_value("", cx);
                });
            }
        })
    });
    owned.join_after_ui();
}
fn terminal(s: &mut Shell, id: u64, cx: &mut Context<Shell>) -> Update {
    loop {
        let u = s.events.recv_timeout(Duration::from_secs(10)).unwrap();
        if u.id == id && !matches!(u.reply, Reply::Progress { .. }) {
            return u;
        }
        s.apply_update(u, cx);
    }
}

#[gpui::test]
fn required_newer_login_draft_survives_save_then_absent_status_hydration(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Access;
            s.environment = None;
            // Includes the independent worker probe, then expands to representative,
            // provider and retention edits with no name edit required for dirty state.
            s.name
                .update(cx, |f, cx| f.set_value("Submitted with absent status", cx));
            s.crawl_fields[10].update(cx, |f, cx| {
                f.set_value("https://example.test/submitted-login", cx)
            });
            assert!(s.environment_unsaved(cx));
            s.activate_control(Control::Save, w, cx);
            let request = s.pending.unwrap();
            for (i, value) in [
                (10, "https://example.test/newer-login"),
                (11, "https://example.test/newer-page"),
                (12, "https://identity.example.test/"),
            ] {
                s.crawl_fields[i].update(cx, |f, cx| f.set_value(value, cx));
            }
            s.login_ephemeral = true;
            let submitted = terminal(s, request, cx);
            assert!(matches!(submitted.reply, Reply::Project(_)));
            s.apply_update(submitted, cx);
            assert!(s.environment.is_none());
            assert_eq!(s.focus.setup_stage, SetupStage::Access);
            let status = s.pending.unwrap();
            s.crawl_fields[11].update(cx, |f, cx| {
                f.set_value("https://example.test/edited-during-read", cx)
            });
            let observed = terminal(s, status, cx);
            s.apply_update(observed, cx);
            finish_jobs(s, cx);
            assert_eq!(
                s.crawl_fields[10].read(cx).content.as_ref(),
                "https://example.test/newer-login"
            );
            assert_eq!(
                s.crawl_fields[11].read(cx).content.as_ref(),
                "https://example.test/edited-during-read"
            );
            assert_eq!(
                s.crawl_fields[12].read(cx).content.as_ref(),
                "https://identity.example.test/"
            );
            assert!(s.login_ephemeral);
            assert!(s.environment.is_some());
            assert!(s.environment_unsaved(cx));
            assert!(s.unsaved(cx));
            assert!(s.task.is_none());
            assert!(!s.control_enabled(Control::Guided(31), cx));
            // Save only login fields/retention with absent status and no other edits.
            s.environment = None;
            s.name
                .update(cx, |f, cx| f.set_value("Submitted with absent status", cx));
            s.activate_control(Control::Save, w, cx);
            finish_jobs(s, cx);
            assert!(!s.unsaved(cx));
            assert_eq!(s.focus.setup_stage, SetupStage::Pages);
            assert_eq!(
                s.crawl_fields[10].read(cx).content.as_ref(),
                "https://example.test/newer-login"
            );
            assert!(s.login_ephemeral);
            // Ordinary untouched initial hydration remains positive: reopen saved project.
            for i in [10, 11, 12] {
                s.crawl_fields[i].update(cx, |f, cx| f.set_value("", cx));
            }
            s.login_ephemeral = false;
            s.queue(Work::Load(project), cx);
            finish_jobs(s, cx);
            assert_eq!(
                s.crawl_fields[10].read(cx).content.as_ref(),
                "https://example.test/newer-login"
            );
            assert!(s.login_ephemeral);
            assert!(!s.unsaved(cx));
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn required_guided_validation_preserves_stage_and_reveals_border_error_help(
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
        for (full, control, stage, bad) in [
            (
                false,
                Control::Address,
                SetupStage::Website,
                "invalid-address",
            ),
            (
                false,
                Control::CrawlField(10),
                SetupStage::Access,
                "invalid-login",
            ),
            (
                false,
                Control::CrawlField(11),
                SetupStage::Access,
                "invalid-page",
            ),
            (
                false,
                Control::CrawlField(12),
                SetupStage::Access,
                "invalid-provider",
            ),
            (false, Control::CrawlField(0), SetupStage::Pages, "0"),
            (false, Control::CrawlField(3), SetupStage::Pages, "101"),
            (false, Control::CrawlField(4), SetupStage::Pages, "9"),
            (false, Control::CrawlField(5), SetupStage::Pages, "9"),
            (false, Control::CrawlField(6), SetupStage::Pages, "4"),
            (
                true,
                Control::CrawlField(10),
                SetupStage::Settings,
                "invalid-login",
            ),
        ] {
            let old = shell.read_with(cx, |s, cx| match control {
                Control::Address => s.address.read(cx).content.to_string(),
                Control::CrawlField(i) => s.crawl_fields[i].read(cx).content.to_string(),
                _ => unreachable!(),
            });
            let request = shell.read_with(cx, |s, _| s.next_request);
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.focus.setup_stage = if full {
                        SetupStage::Settings
                    } else {
                        SetupStage::Website
                    };
                    match control {
                        Control::Address => s.address.update(cx, |f, cx| f.set_value(bad, cx)),
                        Control::CrawlField(i) => {
                            s.crawl_fields[i].update(cx, |f, cx| f.set_value(bad, cx))
                        }
                        _ => (),
                    };
                    s.activate_control(Control::Save, w, cx);
                    assert_eq!(s.focus.setup_stage, stage);
                    assert_eq!(s.presentation_focus, Some(control));
                    assert!(s.pending.is_none());
                    assert_eq!(s.next_request, request);
                })
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.reveal_form_control(control, true, w, cx);
                    cx.notify();
                })
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            shell.read_with(cx, |s, cx| {
                let clip = s.screen_scroll.bounds();
                let field = s.setup_control_bounds(control).unwrap().get().unwrap();
                inside(field, clip);
                let inner = match control {
                    Control::Address => s.address.read(cx).rendered_bounds(),
                    Control::CrawlField(i) => s.crawl_fields[i].read(cx).rendered_bounds(),
                    _ => None,
                }
                .unwrap();
                inside(inner, field);
                assert!(inner.size.width >= px(400.));
                assert!(s.control_enabled(control, cx));
                for st in [SetupStage::Website, SetupStage::Access, SetupStage::Pages] {
                    assert_eq!(s.control_mounted(Control::SetupStage(st)), !full);
                }
            });
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    assert!(s.handle(control, cx).is_focused(w));
                    s.last_validation_field = None;
                    s.activate_control(Control::ValidationNext, w, cx);
                    assert_eq!(s.focus.setup_stage, stage);
                    assert_eq!(s.next_request, request);
                    match control {
                        Control::Address => s.address.update(cx, |f, cx| f.set_value(&old, cx)),
                        Control::CrawlField(i) => {
                            s.crawl_fields[i].update(cx, |f, cx| f.set_value(&old, cx))
                        }
                        _ => (),
                    };
                    assert!(s.form_errors(cx).is_empty());
                })
            });
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn repair_v5_save_continuation_races_keep_selected_draft_and_never_visit(cx: &mut TestAppContext) {
    for discover in [false, true] {
        for race in [
            "newer",
            "invalid",
            "composing",
            "stage",
            "screen",
            "environment",
            "project",
            "failure",
            "stale",
        ] {
            let dir = tempfile::tempdir().unwrap();
            let (mut store, project, _, _, _) = approval_fixture::fixture(dir.path());
            let other = regressee_core::domain::Project::draft(
                "Other selected project".into(),
                "https://other.example.test/",
            )
            .unwrap();
            let other = store.save_project(&other, None).unwrap();
            let baseline = store.load_project(project).unwrap().project.active_baseline;
            let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
            drop(store);
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            vcx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.focus.screen = Screen::Setup;
                    s.focus.setup_stage = if discover {
                        SetupStage::Pages
                    } else {
                        SetupStage::Access
                    };
                    s.environment = None;
                    s.crawl_fields[10].update(cx, |f, cx| {
                        f.set_value("https://example.test/submitted-login", cx)
                    });
                    assert!(s.unsaved(cx));
                    if race == "failure" {
                        s.project.as_mut().unwrap().revision += 1;
                    }
                    let action = if discover {
                        Control::Guided(13)
                    } else {
                        Control::Save
                    };
                    s.activate_control(action, w, cx);
                    let request = s.pending.unwrap();
                    if race == "stale" {
                        s.apply_update(
                            Update {
                                id: request + 100,
                                reply: Reply::ProductFailure("stale request".into()),
                            },
                            cx,
                        );
                        assert_eq!(s.pending, Some(request));
                    }
                    match race {
                        "newer" | "stale" => s.crawl_fields[10].update(cx, |f, cx| {
                            f.set_value("https://example.test/newer-login", cx)
                        }),
                        "invalid" => {
                            s.crawl_fields[10].update(cx, |f, cx| f.set_value("invalid-newer", cx))
                        }
                        "composing" => s.crawl_fields[10].update(cx, |f, cx| {
                            // Simulate a late IME event from the pre-save enabled frame.
                            f.enabled = true;
                            f.replace_and_mark_text_in_range(None, "x", None, w, cx);
                            assert!(f.composing());
                        }),
                        "stage" => s.focus.setup_stage = SetupStage::Website,
                        "screen" => s.focus.screen = Screen::Projects,
                        "environment" => {
                            let mut e = s.draft_settings.environments[0].clone();
                            e.id = regressee_core::domain::EnvironmentId::new();
                            e.name = "Newer environment draft".into();
                            s.environment_draft = Some(e.id);
                            s.draft_settings.environments.push(e);
                        }
                        "project" => {
                            s.project = Some(other.clone());
                            s.draft_settings = other.project.clone();
                            s.name
                                .update(cx, |f, cx| f.set_value("Other newer draft", cx));
                            s.address
                                .update(cx, |f, cx| f.set_value("https://other.example.test/", cx));
                            s.environment_draft = Some(other.project.environments[0].id);
                        }
                        _ => (),
                    }
                    let text = s.crawl_fields[10].read(cx).content.to_string();
                    let env = s.environment_draft;
                    let stage = s.focus.setup_stage;
                    let screen = s.focus.screen;
                    let reply = terminal(s, request, cx);
                    if race == "failure" {
                        assert!(matches!(reply.reply, Reply::Failure(_)));
                    } else {
                        assert!(matches!(reply.reply, Reply::Project(_)));
                    }
                    s.apply_update(reply, cx);
                    if race != "failure" {
                        finish_jobs(s, cx);
                    }
                    assert_eq!(
                        s.crawl_fields[10].read(cx).content.as_ref(),
                        text,
                        "{discover}/{race}"
                    );
                    assert_eq!(s.environment_draft, env);
                    assert_eq!(s.focus.setup_stage, stage, "{discover}/{race}");
                    assert_eq!(s.focus.screen, screen);
                    assert!(
                        !s.task
                            .as_ref()
                            .is_some_and(|t| t.spec.kind == regressee_app::task::Kind::Discovery)
                    );
                    if race == "project" {
                        assert_eq!(s.project.as_ref().unwrap().project.id, other.project.id);
                        assert_eq!(s.name.read(cx).content.as_ref(), "Other newer draft");
                    }
                    if matches!(race, "invalid" | "composing" | "newer" | "stale") {
                        assert!(s.unsaved(cx));
                        assert!(!s.control_enabled(Control::Guided(31), cx));
                    }
                })
            });
            owned.join_after_ui();
            let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
            assert_eq!(
                store.load_project(project).unwrap().project.active_baseline,
                baseline
            );
            assert_eq!(
                serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
                before
            );
        }
    }
}

#[gpui::test]
fn repair_v5_all_setup_capabilities_tab_reveal_and_dispatch_with_current_clips(
    cx: &mut TestAppContext,
) {
    cx.update(|cx| {
        text_input::bind(cx);
        timezone_select::bind(cx);
    });
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::Setup;
        s.focus.setup_stage = SetupStage::Settings;
        s.focus.advanced = true;
        s.notice = None;
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                if s.draft_settings.profiles.len() == 1 {
                    s.activate_control(Control::Guided(3), w, cx);
                }
                assert!(s.draft_settings.profiles.len() > 1);
            })
        });
        macro_rules! reach {
            ($control:expr) => {{
                let control = $control;
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        assert!(s.control_mounted(control), "not mounted {control:?}");
                        assert!(s.control_enabled(control, cx), "not enabled {control:?}");
                        let controls = s
                            .visible_controls()
                            .into_iter()
                            .filter(|c| s.control_enabled(*c, cx))
                            .collect::<Vec<_>>();
                        let index = controls
                            .iter()
                            .position(|c| *c == control)
                            .expect("missing core focus candidate");
                        let previous = controls[(index + controls.len() - 1) % controls.len()];
                        w.focus(&s.handle(previous, cx));
                        s.move_focus(false, w, cx);
                        assert!(s.handle(control, cx).is_focused(w), "Tab {control:?}");
                        s.reveal_form_control(control, true, w, cx);
                        cx.notify();
                    })
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        s.reveal_form_control(control, true, w, cx);
                        cx.notify();
                    })
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                shell.read_with(cx, |s, _| {
                    let b = s.setup_control_bounds(control).unwrap().get().unwrap();
                    let clip = s.screen_scroll.bounds();
                    assert!(
                        b.top() >= clip.top() - px(1.) && b.bottom() <= clip.bottom() + px(1.),
                        "reach {control:?}: {b:?} vs {clip:?}"
                    );
                    inside(
                        s.setup_control_bounds(control).unwrap().get().unwrap(),
                        s.screen_scroll.bounds(),
                    )
                });
            }};
        }
        // All website/source/bounds and advanced input fields, including timezone.
        for c in [Control::Name, Control::Address].into_iter().chain(
            [0, 1, 2, 3, 4, 5, 6, 7, 8, 10, 11, 12, 13, 14, 15, 16]
                .into_iter()
                .map(Control::CrawlField),
        ) {
            reach!(c);
        }
        let requests = shell.read_with(cx, |s, _| s.next_request);
        for c in [1, 2, 3, 7, 8, 9, 10]
            .map(Control::Guided)
            .into_iter()
            .chain(
                [
                    Choice::AddressesOnly,
                    Choice::AddressesAndLinks,
                    Choice::FullPage,
                    Choice::VisibleScreen,
                    Choice::OriginalImages,
                    Choice::ReplaceImages,
                    Choice::Gentle,
                    Choice::Balanced,
                    Choice::Fast,
                    Choice::PublicAccess,
                    Choice::LoginRequired,
                    Choice::SaveLogin,
                    Choice::EphemeralLogin,
                    Choice::LightTheme,
                    Choice::DarkTheme,
                ]
                .map(Control::Choice),
            )
        {
            reach!(c);
            let before_mobile = shell.read_with(cx, |s, _| {
                s.draft_settings.profiles.iter().any(|p| p.name == "Mobile")
            });
            let plan_height = cx.debug_bounds("setup-capture-plan").unwrap().size.height;
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.activate_control(c, w, cx);
                    assert_eq!(s.next_request, requests);
                })
            });
            if c == Control::Guided(3) {
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                shell.read_with(cx, |s, _| {
                    assert_ne!(
                        s.draft_settings.profiles.iter().any(|p| p.name == "Mobile"),
                        before_mobile
                    )
                });
                assert_ne!(
                    cx.debug_bounds("setup-capture-plan").unwrap().size.height,
                    plan_height,
                    "Mobile dispatch must update the painted selected-size plan"
                );
            }
        }
        reach!(Control::Guided(12));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.crawl_fields[7].update(cx, |f, cx| f.set_value("1200", cx));
                s.crawl_fields[8].update(cx, |f, cx| f.set_value("700", cx));
                s.activate_control(Control::Guided(12), w, cx);
                assert!(
                    s.draft_settings
                        .profiles
                        .iter()
                        .any(|p| p.name == "Custom" && p.width == 1200)
                );
            })
        });
        reach!(Control::Guided(12));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.crawl_fields[7].update(cx, |f, cx| f.set_value("1300", cx));
                s.activate_control(Control::Guided(12), w, cx);
                assert_eq!(
                    s.draft_settings
                        .profiles
                        .iter()
                        .filter(|p| p.name == "Custom")
                        .count(),
                    1
                );
                assert!(
                    s.draft_settings
                        .profiles
                        .iter()
                        .any(|p| { p.name == "Custom" && p.width == 1300 && p.height == 700 })
                );
            })
        });
        reach!(Control::Guided(25));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Guided(25), w, cx);
                assert!(!s.draft_settings.profiles.iter().any(|p| p.name == "Custom"));
            })
        });
        reach!(Control::Guided(37));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.crawl_fields[13].update(cx, |f, cx| f.set_value("https://example.test/", cx));
                s.crawl_fields[14]
                    .update(cx, |f, cx| f.set_value("https://staging.example.test/", cx));
                s.activate_control(Control::Guided(37), w, cx);
                assert!(
                    s.draft_settings.environments[0]
                        .mappings
                        .iter()
                        .any(|m| m.destination.host_str() == Some("staging.example.test"))
                );
            })
        });
        reach!(Control::Guided(38));
        cx.update(|w, cx| shell.update(cx, |s, cx| s.activate_control(Control::Guided(38), w, cx)));
        reach!(Control::CrawlField(15));
        shell.update(cx, |s, cx| {
            s.crawl_fields[15].update(cx, |f, cx| f.set_value("nb-NO", cx))
        });
        reach!(Control::CrawlField(16));
        let old_timezone = shell.read_with(cx, |s, cx| s.timezone.read(cx).selected.clone());
        cx.simulate_keystrokes("enter home down enter");
        shell.read_with(cx, |s, cx| {
            assert_ne!(s.timezone.read(cx).selected, old_timezone);
            assert_eq!(s.next_request, requests);
            assert!(s.unsaved(cx));
        });
        reach!(Control::Details(Detail::Capture));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Details(Detail::Capture), w, cx)
            })
        });
        // Explicit reload resets local test drafts, then real local read-only preview.
        shell.update(cx, |s, cx| {
            s.error = None;
            s.notice = None;
            s.queue(Work::Load(project), cx);
            finish_jobs(s, cx);
            s.focus.setup_stage = SetupStage::Settings;
            s.focus.advanced = true;
        });
        reach!(Control::Guided(35));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Guided(35), w, cx);
                finish_jobs(s, cx);
            })
        });
        reach!(Control::CloseMappingPreview);
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::CloseMappingPreview, w, cx)
            })
        });
        reach!(Control::Guided(27));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Guided(27), w, cx);
                assert_eq!(s.draft_settings.environments.len(), 2);
            })
        });
        shell.update(cx, |s, cx| {
            s.queue(Work::Load(project), cx);
            finish_jobs(s, cx);
            s.focus.setup_stage = SetupStage::Settings;
            s.focus.advanced = true;
        });
        reach!(Control::Environment(0));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Environment(0), w, cx)
            })
        });
        // Saved-session maintenance dispatch is explicitly exercised without runtime/network.
        for c in [31, 32, 33, 34, 36].map(Control::Guided) {
            shell.update(cx, |s, cx| {
                s.task = None;
                s.error = None;
                s.notice = None;
                s.queue(Work::Load(project), cx);
                finish_jobs(s, cx);
                s.focus.setup_stage = SetupStage::Settings;
                s.focus.advanced = true;
                if matches!(c, Control::Guided(32 | 33)) {
                    s.environment.as_mut().unwrap().login_open = true;
                }
                if c == Control::Guided(34) {
                    let e = s.environment_draft.unwrap();
                    let session = regressee_core::domain::SessionReference {
                        id: regressee_core::domain::SessionId::new(),
                        project,
                        environment: e,
                    };
                    s.project.as_mut().unwrap().project.environments[0].session =
                        Some(session.clone());
                    s.draft_settings.environments[0].session = Some(session);
                }
            });
            reach!(c);
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    let id = s.next_request;
                    s.activate_control(c, w, cx);
                    assert!(s.next_request > id);
                    let request = s.pending.unwrap();
                    let reply = terminal(s, request, cx);
                    s.apply_update(reply, cx);
                    if s.pending.is_some() {
                        finish_jobs(s, cx);
                    }
                })
            });
        }
        shell.update(cx, |s, cx| {
            s.task = None;
            s.error = None;
            s.queue(Work::Load(project), cx);
            finish_jobs(s, cx);
            s.focus.setup_stage = SetupStage::Settings;
            s.focus.advanced = true;
        });
    }
    owned.join_after_ui();
}
