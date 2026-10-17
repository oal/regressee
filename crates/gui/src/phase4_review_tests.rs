// SPDX-License-Identifier: MPL-2.0
//! Phase 4 controller/layout safety, not OS input or accessibility certification.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::TestAppContext;

#[gpui::test]
fn repair_v4_2_native_shell_pointer_restores_rail_in_every_read_state(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for state in ["ready", "pending", "failed", "dismissed", "empty"] {
            shell.update(cx, |s, cx| {
                s.error = None;
                s.evidence_read_failed = false;
                s.pending = None;
                s.focus.busy = false;
                s.queue(Work::Evidence(project), cx);
                finish_jobs(s, cx);
                s.navigation_preference = Some(true);
                s.configure_review_geometry(width);
                if state != "ready" {
                    s.evidence = ProjectEvidence::default();
                    s.clear_viewer();
                    s.evidence_dirty = state != "empty";
                }
                if state == "pending" {
                    s.queue(Work::Evidence(project), cx);
                }
                if matches!(state, "failed" | "dismissed") {
                    s.evidence_read_failed = true;
                    if state == "failed" {
                        s.error = Some("Synthetic read failure".into());
                    }
                }
                assert!(s.control_enabled(Control::Guided(53), cx));
                cx.notify();
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            let requests = shell.read_with(cx, |s, _| s.next_request);
            for collapsed in [false, true] {
                let toggle = cx.debug_bounds("action-Guided(53)").unwrap();
                cx.simulate_click(toggle.center(), gpui::Modifiers::none());
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                shell.read_with(cx, |s, cx| {
                    assert_eq!(s.navigation_collapsed, collapsed, "{width}: {state}");
                    assert_eq!(
                        s.control_mounted(Control::Navigate(Screen::History)),
                        !collapsed
                    );
                    assert_eq!(s.next_request, requests);
                    assert!(s.control_enabled(Control::Guided(53), cx));
                });
            }
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn repair_v4_1_inner_note_bounds_caret_and_draft_are_usable_at_both_sizes(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    let draft = "Readable release-owner draft: intentional headline; delivery and account changes need confirmation. ".repeat(6);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.analysis_fields[9].update(cx, |f, cx| f.set_value(&draft, cx));
                s.activate_control(Control::Phase5(37), w, cx);
            })
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        cx.run_until_parked();
        let border = cx.debug_bounds("review-note-input-border").unwrap();
        assert_eq!(border.size.height, px(text_input::REVIEW_NOTE_HEIGHT));
        assert!(
            border.size.width >= px(300.),
            "{width}: actual painted border {border:?}"
        );
        shell.read_with(cx, |s, cx| {
            let input = s.analysis_fields[9].read(cx).rendered_bounds().unwrap();
            let viewport = s.review_details_scroll.bounds();
            assert!(
                input.size.width >= px(270.),
                "{width}: actual inner content {input:?}"
            );
            assert!(input.left() >= viewport.left() && input.right() <= viewport.right());
            assert!(
                input.top() >= viewport.top() && input.bottom() <= viewport.bottom(),
                "{width}: input {input:?}, viewport {viewport:?}"
            );
            assert!(!s.control_enabled(Control::Phase5(21), cx));
            assert!(s.control_enabled(Control::Phase5(32), cx));
            assert!(s.control_enabled(Control::Phase5(39), cx));
        });
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                w.focus(&s.handle(Control::AnalysisField(9), cx))
            });
            w.dispatch_action(Box::new(text_input::DocumentHome), cx);
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        for caret in [0, draft.encode_utf16().count()] {
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.analysis_fields[9].update(cx, |input, cx| {
                        let bounds = input.rendered_bounds().unwrap();
                        let caret_bounds = gpui::EntityInputHandler::bounds_for_range(
                            input,
                            caret..caret,
                            bounds,
                            w,
                            cx,
                        )
                        .unwrap();
                        assert!(
                            caret_bounds.left() >= bounds.left()
                                && caret_bounds.right() <= bounds.right()
                                && caret_bounds.top() >= bounds.top()
                                && caret_bounds.bottom() <= bounds.bottom(),
                            "caret {caret}: {caret_bounds:?} outside {bounds:?}"
                        );
                    });
                    w.dispatch_action(Box::new(text_input::DocumentEnd), cx);
                })
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
        }
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                assert!(s.handle(Control::AnalysisField(9), cx).is_focused(w));
                s.dismiss(&Dismiss, w, cx);
                assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), draft);
                assert!(s.handle(Control::Phase5(37), cx).is_focused(w));
            })
        });
    }
    owned.join_after_ui();
}

#[gpui::test]
fn repair_v4_2_navigation_real_shell_pending_failed_dismissed_ready_empty_and_guards(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let stored = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let project_before = serde_json::to_value(store.load_project(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.evidence_read_failed = false;
                s.error = None;
                s.queue(Work::Evidence(project), cx);
                finish_jobs(s, cx);
                s.configure_review_geometry(width);
                s.navigation_preference = Some(true);
                s.configure_review_geometry(width);
                let target = s.selected_key.clone();
                s.analysis_fields[9]
                    .update(cx, |f, cx| f.set_value("retained note during read", cx));
                let check_toggle = |s: &mut Shell, w: &mut Window, cx: &mut Context<Shell>| {
                    let requests = s.next_request;
                    let controls: Vec<_> = s
                        .visible_controls()
                        .into_iter()
                        .filter(|c| s.control_enabled(*c, cx))
                        .collect();
                    assert!(s.control_mounted(Control::Guided(53)));
                    assert!(controls.contains(&Control::Guided(53)));
                    w.focus(&s.handle(Control::ChooseProject, cx));
                    for _ in 0..controls.len() + 1 {
                        s.move_focus(false, w, cx);
                        if s.handle(Control::Guided(53), cx).is_focused(w) {
                            break;
                        }
                    }
                    assert!(s.handle(Control::Guided(53), cx).is_focused(w));
                    s.activate_control(Control::Guided(53), w, cx);
                    assert!(!s.navigation_collapsed);
                    assert!(s.control_mounted(Control::Navigate(Screen::History)));
                    s.activate_control(Control::Guided(53), w, cx);
                    assert!(s.navigation_collapsed);
                    assert_eq!(s.presentation_focus, Some(Control::Guided(53)));
                    s.configure_review_geometry(if width == 820. { 1280. } else { 820. });
                    assert!(s.navigation_collapsed);
                    s.configure_review_geometry(width);
                    assert_eq!(requests, s.next_request);
                    assert_eq!(s.selected_key, target);
                    assert_eq!(
                        s.analysis_fields[9].read(cx).content.as_ref(),
                        "retained note during read"
                    );
                };
                check_toggle(s, w, cx); // ready
                s.evidence = ProjectEvidence::default();
                s.evidence_dirty = true;
                s.clear_viewer();
                s.queue(Work::Evidence(project), cx);
                assert!(s.initial_evidence_loading());
                check_toggle(s, w, cx); // actual tracked pending read
                let id = s.pending.unwrap();
                s.apply_update(
                    Update {
                        id,
                        reply: Reply::ProductFailure("Synthetic failed read".into()),
                    },
                    cx,
                );
                assert!(s.initial_evidence_unavailable());
                check_toggle(s, w, cx);
                s.activate_control(Control::DismissFeedback, w, cx);
                assert!(s.evidence_read_failed && s.error.is_none());
                check_toggle(s, w, cx);
                s.evidence_dirty = false;
                s.evidence_read_failed = false;
                assert!(s.authoritative_empty_capture());
                check_toggle(s, w, cx);
                for boundary in 0..3 {
                    s.focus.dialog = boundary == 0;
                    s.focus.approval_dialog = boundary == 1;
                    s.native_prompt = boundary == 2;
                    let requests = s.next_request;
                    assert!(!s.control_enabled(Control::Guided(53), cx));
                    s.activate_control(Control::Guided(53), w, cx);
                    assert!(s.navigation_collapsed);
                    assert_eq!(requests, s.next_request);
                    s.focus.dialog = false;
                    s.focus.approval_dialog = false;
                    s.native_prompt = false;
                }
                s.begin_task(regressee_gui::task::Spec::for_work(&Work::SetupRuntime).unwrap());
                assert!(!s.control_enabled(Control::Guided(53), cx));
                s.activate_control(Control::Guided(53), w, cx);
                assert!(s.navigation_collapsed);
                s.task = None;
            })
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        assert!(cx.debug_bounds("action-Guided(53)").is_some());
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        stored,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    assert_eq!(
        project_before,
        serde_json::to_value(store.load_project(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn repair_v4_3_persistent_non_root_identity_historical_environments_all_modes(
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
        let run = s.evidence.latest.as_mut().unwrap();
        run.run.targets[0].requested = "https://check.example/account/orders/customer-service/release-notes/?sort=recent&locale=nb-NO#delivery".parse().unwrap();
        run.run.environment.name = "Historical check".into();
        s.viewer.as_mut().unwrap().before_environment = Some("Historical reference".into());
        s.project.as_mut().unwrap().project.environments[0].name = "Today's unrelated settings".into();
        s.review_queue_collapsed = true; s.navigation_preference = Some(true);
        s.viewer_fit = false; s.zoom = 0.75; s.pan_x = 11.; s.pan_y = 12;
        s.analysis_fields[9].update(cx, |f,cx| f.set_value("identity draft remains attached",cx));
        s.approval_selection = s.selected_key.iter().cloned().collect();
    });
    let selected = shell.read_with(cx, |s, _| s.selected_key.clone());
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for (mode, after) in [
            (0, false),
            (1, false),
            (2, false),
            (3, false),
            (4, false),
            (4, true),
        ] {
            shell.update(cx, |s, cx| {
                s.viewer_mode = mode; s.full_width_after = after; s.configure_review_geometry(width);
                let (path, context) = s.inspection_identity();
                assert_eq!(path, "/account/orders/customer-service/release-notes/?sort=recent&locale=nb-NO#delivery");
                assert!(context.contains("Historical reference") && context.contains("Historical check"));
                let profile = &s.evidence.latest.as_ref().unwrap().evidence(s.selected_key.as_ref().unwrap()).unwrap().metadata.profile;
                assert!(context.contains(&profile.name) && context.contains(&format!("{} × {} pixels",profile.width,profile.height)));
                assert!(!context.contains("Today's")); cx.notify();
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            for name in [
                "inspected-target-path",
                "inspected-target-profile-environments",
            ] {
                let b = cx.debug_bounds(name).unwrap();
                let owner = cx.debug_bounds("bounded-review").unwrap();
                assert!(b.left() >= owner.left() && b.right() <= owner.right());
                assert!(b.top() >= owner.top() && b.bottom() <= owner.bottom());
                assert!(b.size.height > px(0.));
            }
            shell.read_with(cx, |s, cx| {
                assert_eq!((s.zoom, s.pan_x, s.pan_y), (0.75, 11., 12));
                assert_eq!(s.selected_key, selected);
                assert_eq!(s.approval_selection, selected.iter().cloned().collect());
                assert_eq!(
                    s.analysis_fields[9].read(cx).content.as_ref(),
                    "identity draft remains attached"
                );
            });
        }
        shell.update(cx, |s, cx| {
            s.evidence.comparison = None;
            cx.notify();
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        shell.read_with(cx, |s, _| {
            assert!(
                s.inspection_identity()
                    .1
                    .contains("Capture: Check · not compared · Historical check")
            )
        });
        // Restore comparison from the actual store-backed inspection for next size.
        shell.update(cx, |s, cx| {
            s.queue(Work::Evidence(project), cx);
            finish_jobs(s, cx);
        });
        shell.update(cx, |s, _| { s.evidence.latest.as_mut().unwrap().run.targets[0].requested = "https://check.example/account/orders/customer-service/release-notes/?sort=recent&locale=nb-NO#delivery".parse().unwrap(); s.evidence.latest.as_mut().unwrap().run.environment.name = "Historical check".into(); s.viewer.as_mut().unwrap().before_environment = Some("Historical reference".into()); s.viewer_fit=false; s.zoom=0.75; s.pan_x=11.; s.pan_y=12; });
    }
    owned.join_after_ui();
}

#[test]
fn result_paths_keep_query_fragment_and_non_web_identity() {
    assert_eq!(
        controls::page_path("https://example.test/orders/42?sort=asc#items"),
        "/orders/42?sort=asc#items"
    );
    assert_eq!(controls::page_path("https://example.test/"), "/");
    assert_eq!(
        controls::page_path("unavailable source"),
        "unavailable source"
    );
}

#[gpui::test]
fn local_collapse_modes_source_anchor_drafts_and_search_are_presentation_only(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let before_project = serde_json::to_value(store.load_project(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.configure_review_geometry(820.);
            assert!(s.navigation_collapsed);
            assert_eq!(s.working_width, 772.);
            assert!(!s.review_queue_collapsed);
            assert_eq!(s.viewer_mode, 0);
            let key = s.selected_key.clone().unwrap();
            s.approval_selection.insert(key.clone());
            s.analysis_fields[9].update(cx, |f, cx| f.set_value("unsaved release-owner note", cx));
            s.viewer_fit = false;
            s.zoom = 0.75;
            s.pan_x = 11.;
            s.pan_y = 12;
            s.activate_control(Control::Guided(53), w, cx);
            assert!(!s.navigation_collapsed);
            s.configure_review_geometry(1280.);
            s.configure_review_geometry(820.);
            assert!(
                !s.navigation_collapsed,
                "resize must respect explicit preference"
            );
            s.activate_control(Control::Guided(50), w, cx);
            assert!(s.review_queue_collapsed);
            assert!(!s.control_mounted(Control::CrawlField(9)));
            s.find_local(&FindLocal, w, cx);
            assert!(!s.review_queue_collapsed);
            assert!(s.handle(Control::CrawlField(9), cx).is_focused(w));
            for mode in 6..=9 {
                s.activate_control(Control::Guided(54), w, cx);
                assert!(s.control_mounted(Control::Phase5(mode)));
                assert!(!s.control_mounted(Control::Phase5(21)));
                s.activate_control(Control::Phase5(mode), w, cx);
                assert_eq!(s.viewer_mode, mode - 6);
            }
            s.activate_control(Control::Guided(54), w, cx);
            s.activate_control(Control::Guided(55), w, cx);
            assert_eq!(s.viewer_pane_indices(), &[0]);
            s.activate_control(Control::Guided(56), w, cx);
            assert_eq!(s.viewer_pane_indices(), &[1]);
            assert_eq!((s.zoom, s.pan_x, s.pan_y), (0.75, 11., 12));
            assert!(!s.viewer_fit);
            assert_eq!(s.selected_key.as_ref(), Some(&key));
            assert!(s.approval_selection.contains(&key));
            assert!(s.review_note_dirty(cx));
            s.activate_control(Control::Phase5(37), w, cx);
            assert!(s.review_note_open);
            for action in [20, 21, 22, 23, 24, 33] {
                assert!(!s.control_enabled(Control::Phase5(action), cx));
            }
            s.dismiss(&Dismiss, w, cx);
            assert!(!s.review_note_open);
            assert_eq!(
                s.analysis_fields[9].read(cx).content.as_ref(),
                "unsaved release-owner note"
            );
            s.crawl_fields[9].update(cx, |f, cx| f.set_value("no-such-page", cx));
            assert_eq!(s.selected_key.as_ref(), Some(&key));
            assert!(s.approval_selection.contains(&key));
            finish_jobs(s, cx);
        })
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    assert_eq!(
        before_project,
        serde_json::to_value(store.load_project(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn capture_overview_is_capped_manual_enlargement_and_empty_owner_remain_available(
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
        s.evidence.comparison = None;
        s.configure_review_geometry(1280.);
        assert_eq!(s.viewer_pane_indices(), &[1]);
        assert_eq!(s.fitted_viewer_zoom(), 1.);
        s.choose_viewer_zoom(ZoomChoice::Percent(150), cx);
        assert_eq!(s.zoom, 1.5);
        assert!(!s.viewer_fit);
        s.configure_review_geometry(820.);
        assert_eq!(s.zoom, 1.5);
        s.evidence = ProjectEvidence::default();
        s.evidence_dirty = false;
        s.clear_viewer();
        assert!(s.authoritative_empty_capture());
        assert_eq!(s.review_queue_width(), 0.);
        assert!(!s.control_mounted(Control::Guided(50)));
    });
    owned.join_after_ui();
}

#[gpui::test]
fn consent_one_long_and_bounded_many_rows_keep_exact_data_and_cancel_without_writes(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, analysis, _) = approval_fixture::fixture(dir.path());
    let key = store
        .project_evidence(project)
        .unwrap()
        .latest
        .unwrap()
        .run
        .targets[0]
        .key
        .clone();
    let preview = store
        .approval_preview(analysis, std::slice::from_ref(&key))
        .unwrap();
    assert_eq!(
        preview.source_environments.len(),
        preview.baseline.references.len()
    );
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.approval_selection.insert(key.clone());
        s.approval = Some(preview.clone());
        s.focus.approval_dialog = true;
        let long = format!(
            "https://example.test/{}/?query=full-exact-data#tail",
            "long-page/".repeat(36)
        );
        s.evidence.latest.as_mut().unwrap().run.targets[0].requested = long.parse().unwrap();
        s.evidence.latest.as_mut().unwrap().run.inventory.pages[0].canonical =
            long.parse().unwrap();
        *s.local_index.borrow_mut() = Default::default();
        s.approval.as_mut().unwrap().provenance[0].1 = long;
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        let row = cx.debug_bounds("approval-reference-0").unwrap();
        let title = cx.debug_bounds("approval-reference-title-0").unwrap();
        let time = cx.debug_bounds("approval-reference-time-0").unwrap();
        assert!(title.top() >= row.top() && time.bottom() <= row.bottom());
        assert!(
            row.size.height > px(100.),
            "long rows grow rather than crossing a separator"
        );
        assert!(cx.debug_bounds("approval-exact-provenance").is_none());
    }
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.activate_control(Control::Guided(52), w, cx);
            assert!(s.approval_exact_open);
            // Presentation-only expansion tests bounded mounting; never saved as site evidence.
            let p = s.approval.as_mut().unwrap();
            let template = p.baseline.references[0].clone();
            p.baseline.references = vec![template; 97];
            for _ in 0..96 {
                s.activate_control(Control::Guided(49), w, cx);
            }
            assert_eq!(s.approval_reference_index, 96);
        })
    });
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
    assert!(cx.debug_bounds("approval-reference-96").is_some());
    // GPUI retains debug bounds for selectors painted in earlier frames; they
    // are not a mounted-control oracle. This new final-page row proves reach.
    assert!(cx.debug_bounds("approval-reference-97").is_none());
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            let requests = s.next_request;
            s.activate_control(Control::CheckChanges, w, cx);
            assert_eq!(requests, s.next_request);
            s.dismiss(&Dismiss, w, cx);
            assert!(!s.focus.approval_dialog);
            assert!(s.handle(Control::Phase5(24), cx).is_focused(w));
        })
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
