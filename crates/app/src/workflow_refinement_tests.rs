// SPDX-License-Identifier: MPL-2.0
//! Controller/focus regressions using real local storage and incumbent corpus
//! pixels, including headless painted-layout bounds. These are not native input,
//! accessibility or native rendering qualification.
use super::*;
use gpui::TestAppContext;
use regressee_core::domain::{ProjectId, ReviewDecision, ReviewState};

#[gpui::test]
fn partial_baseline_save_is_discoverable_cancel_safe_and_explicit(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, run) = approval_fixture::partial_fixture(dir.path());
    let previous = store.load_project(project).unwrap().project.active_baseline;
    let source = serde_json::to_value(store.load_run(run).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        assert!(s.focus.candidate);
        assert!(s.control_enabled(Control::SaveBaseline, cx));
        assert!(s.control_enabled(Control::ReviewCaptureFailures, cx));
        let actions: Vec<_> = tab_controls(s, cx)
            .into_iter()
            .filter(|c| {
                matches!(
                    c,
                    Control::SaveBaseline
                        | Control::ReviewCaptureFailures
                        | Control::CheckChanges
                        | Control::RefreshEvidence
                )
            })
            .collect();
        assert_eq!(
            actions,
            [
                Control::SaveBaseline,
                Control::ReviewCaptureFailures,
                Control::CheckChanges,
                Control::RefreshEvidence
            ]
        );
        let (title, message, action) = s.baseline_save_confirmation(3, 4);
        assert_eq!(title, "Save baseline and skip 1 screenshot?");
        assert_eq!(action, "Save & skip missing");
        for text in [
            "3 of 4",
            "will not be checked",
            "replaces your current baseline",
            "History",
        ] {
            assert!(message.contains(text), "{message}");
        }
        assert!(
            s.baseline_save_confirmation(2, 4)
                .0
                .contains("2 screenshots")
        );
        assert_eq!(
            s.baseline_save_confirmation(149, 150).0,
            "Save baseline and skip 1 screenshot?"
        );
        assert_eq!(s.baseline_save_confirmation(4, 4).2, "Save baseline");
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        for selector in ["action-SaveBaseline", "action-ReviewCaptureFailures"] {
            let bounds = cx.debug_bounds(selector).unwrap();
            assert!(bounds.size.height >= px(40.));
            assert!(bounds.left() >= px(0.) && bounds.right() <= px(width));
        }
        assert!(cx.debug_bounds("review-viewer").unwrap().size.height >= px(200.));
    }
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            let key = s.selected_key.clone();
            s.activate_control(Control::ReviewCaptureFailures, window, cx);
            assert_eq!(s.result_indices(), vec![3]);
            assert_eq!(s.selected_key, key, "filtering must preserve inspection");
            assert!(s.control_enabled(Control::SaveBaseline, cx));
            s.activate_control(Control::SaveBaseline, window, cx);
            assert!(s.native_prompt);
            assert!(!s.control_enabled(Control::SaveBaseline, cx));
        })
    });
    assert_eq!(
        cx.pending_prompt().unwrap().0,
        "Save baseline and skip 1 screenshot?"
    );
    cx.simulate_prompt_answer("Keep reviewing");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert!(!s.native_prompt);
            assert_eq!(
                s.project.as_ref().unwrap().project.active_baseline,
                previous
            );
            assert!(s.focus.candidate);
            s.activate_control(Control::SaveBaseline, window, cx);
        })
    });
    cx.simulate_prompt_answer("Save & skip missing");
    cx.run_until_parked();
    shell.update(cx, |s, cx| {
        finish_jobs(s, cx);
        assert!(s.error.is_none(), "{:?}", s.error);
        // Evidence is read explicitly after the mutation, as in the render loop.
        s.queue(Work::Evidence(project), cx);
        finish_jobs(s, cx);
        assert!(s.capture_is_saved_baseline());
        assert!(!s.focus.candidate);
        assert!(s.control_enabled(Control::CheckChanges, cx));
        assert!(!s.control_mounted(Control::SaveBaseline));
        assert!(s.evidence.baseline.as_ref().unwrap().explicitly_partial);
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        serde_json::to_value(store.load_run(run).unwrap()).unwrap(),
        source
    );
}

pub(super) fn screenshot_switch_fixture(path: &std::path::Path, comparison: bool) -> ProjectId {
    use regressee_core::domain::{InventoryId, PageId, RunId};
    let (mut store, project, _, _, _) = approval_fixture::fixture(path);
    let baseline = store.project_evidence(project).unwrap().baseline.unwrap();
    let mut run = store.load_run(baseline.references[0].run).unwrap();
    run.run.id = RunId::new();
    run.run.inventory.id = InventoryId::new();
    run.run.environment.name = "Recorded website".into();
    let mut page = run.run.inventory.pages[0].clone();
    page.id = PageId::new();
    page.canonical = url::Url::parse("http://127.0.0.1:43210/next?view=full#section").unwrap();
    let mut target = run.run.targets[0].clone();
    target.key.page = page.id;
    target.requested = page.canonical.clone();
    target.final_url = Some(page.canonical.clone());
    let mut capture = run.captures[0].clone();
    capture.metadata.key = target.key.clone();
    capture.metadata.requested = page.canonical.to_string();
    capture.metadata.final_url = page.canonical.to_string();
    run.run.inventory.pages.push(page);
    run.run.targets.push(target);
    run.captures.push(capture);
    store.freeze_inventory(&run.run.inventory).unwrap();
    let save_run = |store: &mut regressee_storage::Store,
                    run: &regressee_core::evidence::RunRecord| {
        let mut pending = run.clone();
        pending.run.state = JobState::Running;
        pending.run.finished_at = None;
        pending.captures.clear();
        for target in &mut pending.run.targets {
            target.outcome = TargetOutcome::Pending;
        }
        store.begin_run(&pending).unwrap();
        store.finish_run(run).unwrap();
    };
    save_run(&mut store, &run);
    if comparison {
        let revision = store.load_project(project).unwrap().revision;
        let saved = store.activate_baseline(run.run.id, revision).unwrap();
        run.run.id = RunId::new();
        run.run.intent = RunIntent::Check {
            baseline: saved.project.active_baseline.unwrap(),
        };
        save_run(&mut store, &run);
        regressee_app::review::analyze_run(
            &mut store,
            run.run.id,
            regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
            &std::sync::atomic::AtomicBool::new(false),
        )
        .unwrap();
    }
    project
}

#[gpui::test]
fn screenshot_switch_keeps_inspection_header_and_layout_during_image_reads(
    cx: &mut TestAppContext,
) {
    for comparison in [false, true] {
        let dir = tempfile::tempdir().unwrap();
        let project = screenshot_switch_fixture(dir.path(), comparison);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            assert_eq!(s.evidence.comparison.is_some(), comparison);
            assert_eq!(s.focus.candidate, !comparison);
        });
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            for collapsed in [false, true] {
                shell.update(cx, |s, cx| {
                    s.review_queue_collapsed = collapsed;
                    cx.notify();
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                let heading = cx.debug_bounds("inspected-target-context").unwrap();
                let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
                let viewer = cx.debug_bounds("review-viewer").unwrap();
                for (control, expected_path) in [
                    (Control::Guided(24), "/next?view=full#section"),
                    (Control::Guided(23), "/"),
                ] {
                    if !comparison && !collapsed && width >= 1100. && control == Control::Guided(24)
                    {
                        let row = cx.debug_bounds("page-row-target-1").unwrap();
                        cx.simulate_click(row.center(), gpui::Modifiers::none());
                    } else {
                        cx.update(|w, cx| {
                            shell.update(cx, |s, cx| {
                                // Hidden queues use the same navigation route
                                // without a mounted Previous/Next control.
                                if collapsed {
                                    s.guided(
                                        if control == Control::Guided(24) {
                                            24
                                        } else {
                                            23
                                        },
                                        cx,
                                    );
                                } else {
                                    s.activate_control(control, w, cx);
                                }
                            });
                        });
                    }
                    shell.read_with(cx, |s, _| {
                        assert!(s.viewer.is_none(), "{width}: comparison={comparison}, hidden queue={collapsed}, {control:?}");
                        assert!(s.viewer_pending.is_some());
                        assert!(s.inspection_chrome_mounted());
                        let (path, context) = s.inspection_identity();
                        assert_eq!(path, expected_path);
                        assert!(context.contains("Recorded website"));
                    });
                    cx.update(|w, cx| {
                        let _ = w.draw(cx);
                    });
                    assert_eq!(
                        cx.debug_bounds("inspected-target-context").unwrap(),
                        heading
                    );
                    assert_eq!(cx.debug_bounds("viewer-toolbar").unwrap(), toolbar);
                    assert_eq!(
                        cx.debug_bounds("review-viewer").unwrap().origin,
                        viewer.origin,
                        "{width}x{height}, comparison={comparison}, hidden queue={collapsed}: loading must not replace the inspection header or collapse its toolbar"
                    );
                    shell.update(cx, |s, cx| {
                        assert!(s.viewer_pending.is_some());
                        assert!(!s.viewer_has_pixels());
                        assert!(!s.control_enabled(Control::Guided(46), cx));
                        finish_jobs(s, cx);
                        assert!(s.viewer_current() && s.viewer_has_pixels());
                        assert_eq!(s.inspection_identity().0, expected_path);
                    });
                    cx.update(|w, cx| {
                        let _ = w.draw(cx);
                    });
                    assert_eq!(
                        cx.debug_bounds("inspected-target-context").unwrap(),
                        heading
                    );
                    assert_eq!(cx.debug_bounds("viewer-toolbar").unwrap(), toolbar);
                    assert_eq!(
                        cx.debug_bounds("review-viewer").unwrap().origin,
                        viewer.origin
                    );
                }
            }
        }
        owned.join_after_ui();
    }
}

#[gpui::test]
fn screenshot_loading_header_requires_current_selection_and_preserves_failure_recovery(
    cx: &mut TestAppContext,
) {
    use crate::loading_ux::ScreenshotState;
    let dir = tempfile::tempdir().unwrap();
    let project = screenshot_switch_fixture(dir.path(), false);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        let previous_key = s.selected_key.clone();
        s.guided(24, cx);
        let selected_key = s.selected_key.clone();
        assert!(s.inspection_chrome_mounted());
        s.selected_key = previous_key;
        assert!(
            !s.inspection_chrome_mounted(),
            "stale reads cannot hold inspection chrome open"
        );
        s.selected_key = selected_key;
        s.focus.review_surface = ReviewSurface::Pages;
        assert!(!s.inspection_chrome_mounted());
        s.focus.review_surface = ReviewSurface::Inspect;
        let id = s.viewer_pending.as_ref().unwrap().0;
        s.apply_update(
            Update {
                id,
                reply: Reply::ProductFailure("Synthetic image read failure".into()),
            },
            cx,
        );
        assert!(!s.inspection_chrome_mounted());
        assert_eq!(s.screenshot_state(), ScreenshotState::Unavailable);
        assert!(!s.viewer_has_pixels());
        assert!(s.control_mounted(Control::Phase5(36)));
        assert!(!s.control_enabled(Control::Guided(46), cx));
        s.evidence.latest.as_mut().unwrap().run.targets[1].outcome = TargetOutcome::Failed {
            code: regressee_core::domain::CaptureError::Timeout,
        };
        s.reload_viewer(cx);
        assert!(s.selected_capture_failed());
        assert!(
            !s.inspection_chrome_mounted(),
            "failed targets keep their recovery header"
        );
    });
    owned.join_after_ui();
}

#[gpui::test]
fn dismissing_a_read_error_keeps_recovery_instead_of_an_idle_loading_state(
    cx: &mut TestAppContext,
) {
    use crate::loading_ux::ScreenshotState;
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.evidence = ProjectEvidence::default();
            s.evidence_dirty = true;
            s.clear_viewer();
            s.queue(Work::Evidence(project), cx);
            let id = s.pending.unwrap();
            s.apply_update(
                Update {
                    id,
                    reply: Reply::ProductFailure("Synthetic disk-read failure".into()),
                },
                cx,
            );
            assert!(s.evidence_read_failed);
            s.activate_control(Control::DismissFeedback, window, cx);
            assert!(s.error.is_none());
            assert_eq!(s.screenshot_state(), ScreenshotState::Unavailable);
            assert!(!s.initial_evidence_loading());
            assert!(s.control_enabled(Control::RefreshEvidence, cx));
            s.activate_control(Control::RefreshEvidence, window, cx);
            assert!(!s.evidence_read_failed);
            assert!(s.initial_evidence_loading());
            finish_jobs(s, cx);
            assert!(s.viewer_has_pixels());
        })
    });
    owned.join_after_ui();
}

pub(super) fn finish_jobs(s: &mut Shell, cx: &mut Context<Shell>) {
    let deadline = std::time::Instant::now() + Duration::from_secs(10);
    for _ in 0..64 {
        if s.pending.is_none()
            && s.viewer_pending.is_none()
            && s.storage_pending.is_none()
            && s.runtime_pending.is_none()
        {
            return;
        }
        let remaining = deadline.saturating_duration_since(std::time::Instant::now());
        let update = s
            .events
            .recv_timeout(remaining)
            .expect("tracked controller work must finish");
        assert!(
            !matches!(update.reply, Reply::Failure(_)),
            "{:?}",
            update.reply
        );
        s.apply_update(update, cx);
    }
    panic!("controller work did not settle");
}

pub(super) fn load_review(s: &mut Shell, project: ProjectId, cx: &mut Context<Shell>) {
    finish_jobs(s, cx);
    s.focus.screen = Screen::History;
    s.queue(Work::Load(project), cx);
    finish_jobs(s, cx);
    s.queue(Work::EnvironmentStatus(project), cx);
    finish_jobs(s, cx);
    s.focus.screen = Screen::Results;
    s.focus.review_surface = ReviewSurface::Changes;
    s.reload_viewer(cx);
    finish_jobs(s, cx);
    s.sync_review_note(cx);
    assert!(!s.focus.busy);
    assert!(s.viewer_current());
}

fn tab_controls(s: &Shell, cx: &App) -> Vec<Control> {
    s.focus
        .controls()
        .into_iter()
        .filter(|control| s.control_enabled(*control, cx))
        .collect()
}

fn discovery(s: &Shell) -> DiscoveryPreview {
    DiscoveryPreview {
        project_revision: s.project.as_ref().unwrap().revision,
        settings: s.project.as_ref().unwrap().project.clone(),
        inventory: s.evidence.latest.as_ref().unwrap().run.inventory.clone(),
        aliases: vec![],
        notes: vec![],
        visited: 1,
    }
}

fn assert_bounds_inside(inner: gpui::Bounds<gpui::Pixels>, outer: gpui::Bounds<gpui::Pixels>) {
    assert!(inner.size.width > px(0.) && inner.size.height > px(0.));
    assert!(
        inner.left() >= outer.left() - px(1.)
            && inner.right() <= outer.right() + px(1.)
            && inner.top() >= outer.top() - px(1.)
            && inner.bottom() <= outer.bottom() + px(1.),
        "painted bounds {inner:?} must fit inside {outer:?}"
    );
}

#[gpui::test]
fn everyday_review_is_revision_bound_and_discovery_excludes_old_comparison(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, valid, historical) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.review_tools = false;
        let key = s.selected_key.clone().unwrap();
        s.evidence.reviews = vec![ReviewDecision {
            analysis: historical,
            key: key.clone(),
            state: ReviewState::NeedsFix,
            note: "Historical decision must not be current".into(),
            reviewed_at: 3,
        }];
        s.discard_review_drafts(cx);
        assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), "");
        assert_eq!(s.evidence.comparison.as_ref().unwrap().revision.id, valid);
        for control in [
            Control::Phase5(20),
            Control::Phase5(21),
            Control::Phase5(22),
            Control::Phase5(37),
        ] {
            assert!(
                s.focus.controls().contains(&control),
                "missing core candidate {control:?}"
            );
            assert!(
                s.control_mounted(control),
                "ordinary review cannot require Tools: {control:?}"
            );
            assert!(
                tab_controls(s, cx).contains(&control),
                "missing enabled Tab stop {control:?}"
            );
        }
        s.review_note_open = true;
        assert!(tab_controls(s, cx).contains(&Control::AnalysisField(9)));
        assert!(tab_controls(s, cx).contains(&Control::Phase5(32)));
        s.evidence.reviews.insert(
            0,
            ReviewDecision {
                analysis: valid,
                key,
                state: ReviewState::Intentional,
                note: "Current revision".into(),
                reviewed_at: 3,
            },
        );
        s.discard_review_drafts(cx);
        assert_eq!(
            s.analysis_fields[9].read(cx).content.as_ref(),
            "Current revision"
        );
        let baseline = s.project.as_ref().unwrap().project.active_baseline;
        let approval_selection = s.approval_selection.clone();
        s.phase5_action(22, cx);
        finish_jobs(s, cx);
        let current = s
            .evidence
            .reviews
            .iter()
            .find(|d| d.analysis == valid && Some(&d.key) == s.selected_key.as_ref())
            .unwrap();
        assert_eq!(current.state, ReviewState::NeedsFix);
        assert_eq!(current.note, "Current revision");
        assert_eq!(
            s.project.as_ref().unwrap().project.active_baseline,
            baseline,
            "classification is not baseline approval"
        );
        assert_eq!(s.approval_selection, approval_selection);
        let comparison = s.evidence.comparison.as_ref().unwrap().revision.id;
        s.preview = Some(discovery(s));
        s.focus.preview = true;
        s.focus.review_surface = ReviewSurface::Pages;
        s.discovery_preview_visible = true;
        assert!(
            !s.discovery_image_mounted(),
            "an old check is not a representative preview"
        );
        assert_eq!(s.viewer_pane_indices(), &[1]);
        for control in [
            Control::Phase5(6),
            Control::Phase5(20),
            Control::Phase5(21),
            Control::Phase5(22),
            Control::Phase5(23),
            Control::Phase5(24),
            Control::AnalysisField(9),
            Control::Guided(51),
        ] {
            assert!(
                !s.control_mounted(control),
                "page-list leaked comparison control {control:?}"
            );
            assert!(!tab_controls(s, cx).contains(&control));
        }
        assert_eq!(
            s.evidence.comparison.as_ref().unwrap().revision.id,
            comparison,
            "hiding old comparison must not destroy it"
        );
        s.preview = None;
        s.focus.preview = false;
        for control in [
            Control::Phase5(20),
            Control::Phase5(21),
            Control::Phase5(22),
        ] {
            assert!(
                !s.control_mounted(control),
                "Pages without a discovery preview still must not show decisions"
            );
        }
    });
    owned.join_after_ui();
}

#[gpui::test]
fn collapsed_queue_removes_hidden_tab_targets_and_refits_without_touching_drafts(
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
            s.configure_review_geometry(820.);
            s.result_filters_open = true;
            let key = s.selected_key.clone();
            s.mask_dirty = true;
            let rectangles = s.mask_rectangles.clone();
            s.analysis_fields[9].update(cx, |f, cx| f.set_value("Preserved draft", cx));
            let width = s.current_viewer_pane_width();
            let requests = s.next_request;
            s.activate_control(Control::Guided(50), window, cx);
            assert!(s.review_queue_collapsed);
            assert_eq!(s.review_queue_width(), 0.);
            assert!(s.current_viewer_pane_width() > width);
            assert_eq!(s.zoom, s.fitted_viewer_zoom());
            assert!(tab_controls(s, cx).contains(&Control::Guided(50)));
            for control in [
                Control::CrawlField(9),
                Control::Guided(17),
                Control::Guided(19),
                Control::Guided(23),
                Control::Guided(24),
                Control::Guided(45),
                Control::Choice(Choice::ChangedResults),
            ] {
                assert!(
                    !s.control_mounted(control),
                    "collapsed queue leaked {control:?}"
                );
                assert!(!tab_controls(s, cx).contains(&control));
            }
            assert_eq!(s.selected_key, key);
            assert_eq!(s.mask_rectangles, rectangles);
            assert!(s.mask_dirty);
            assert_eq!(
                s.analysis_fields[9].read(cx).content.as_ref(),
                "Preserved draft"
            );
            assert_eq!(s.next_request, requests);
            s.activate_control(Control::Guided(50), window, cx);
            assert!(!s.review_queue_collapsed);
            assert_eq!(s.current_viewer_pane_width(), width);
            assert!(tab_controls(s, cx).contains(&Control::CrawlField(9)));
            assert!(!s.control_mounted(Control::Guided(51)));
            s.preview = Some(discovery(s));
            s.focus.preview = true;
            s.focus.review_surface = ReviewSurface::Pages;
            s.discovery_preview_visible = true;
            assert!(
                !s.control_mounted(Control::Guided(51)),
                "old comparison is not explicit representative evidence"
            );
            s.evidence.comparison = None;
            s.evidence.latest.as_mut().unwrap().run.intent = RunIntent::RepresentativePreview;
            assert!(s.discovery_image_mounted());
            assert!(tab_controls(s, cx).contains(&Control::Guided(51)));
            s.activate_control(Control::Guided(51), window, cx);
            assert!(!s.discovery_image_mounted());
            assert!(!s.control_mounted(Control::Guided(51)));
            assert_eq!(s.presentation_focus, Some(Control::Phase5(30)));
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn history_sections_exclude_off_section_controls_from_tab_order(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::History;
            s.history_section = 0;
            assert!(
                tab_controls(s, cx).contains(&Control::Phase6(19)),
                "Runs exposes Open saved evidence"
            );
            assert!(tab_controls(s, cx).contains(&Control::HistoryRun(s.run_history_index)));
            assert!(!s.control_mounted(Control::HistoryBaseline(s.history_index)));
            assert!(!s.control_mounted(Control::Phase6(8)));
            s.activate_control(Control::Phase6(17), window, cx);
            assert_eq!(s.history_section, 1);
            assert!(tab_controls(s, cx).contains(&Control::HistoryBaseline(s.history_index)));
            for control in [
                Control::HistoryRun(s.run_history_index),
                Control::Phase6(19),
                Control::Phase6(5),
                Control::HistoryKind(0),
                Control::Phase6(8),
            ] {
                assert!(!s.control_mounted(control), "Baselines leaked {control:?}");
            }
            s.activate_control(Control::Phase6(18), window, cx);
            assert_eq!(s.history_section, 2);
            assert!(tab_controls(s, cx).contains(&Control::Phase6(8)));
            for control in [
                Control::HistorySearch,
                Control::HistoryRefresh,
                Control::HistoryRun(s.run_history_index),
                Control::HistoryBaseline(s.history_index),
                Control::Phase6(19),
                Control::Phase6(2),
                Control::Phase6(4),
                Control::Phase5(42),
            ] {
                assert!(!s.control_mounted(control), "Storage leaked {control:?}");
                assert!(!tab_controls(s, cx).contains(&control));
            }
            s.activate_control(Control::Phase6(16), window, cx);
            assert_eq!(s.history_section, 0);
            assert!(tab_controls(s, cx).contains(&Control::Phase6(19)));
            let run = s.history.as_ref().unwrap().runs[s.run_history_index].run.id;
            s.activate_control(Control::Phase6(19), window, cx);
            assert!(
                s.pending.is_some(),
                "Open run must dispatch the explicit local inspection read"
            );
            finish_jobs(s, cx);
            assert_eq!(s.history_inspection, Some((project, run)));
            assert_eq!(s.focus.screen, Screen::Results);
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn stale_history_reply_is_rejected_and_recorded_check_is_readonly_with_saved_policy(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, run, _, _) = approval_fixture::fixture(dir.path());
    let inspected = store.inspect_run(project, run).unwrap();
    let saved_policy = inspected
        .comparison
        .as_ref()
        .unwrap()
        .perceptual
        .as_ref()
        .unwrap()
        .policy
        .clone();
    let baseline_count = store.baseline_history(project).unwrap().len();
    drop(store);
    let paths = AppPaths::isolated(dir.path());
    let (jobs, events) = Jobs::start_without_runtime(paths.clone());
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::History;
            s.history_section = 0;
            s.run_history_index = s
                .history
                .as_ref()
                .unwrap()
                .runs
                .iter()
                .position(|r| r.run.id == run)
                .unwrap();
            s.history_selection_epoch = 10;
            s.history_open_epoch = Some(9);
            let key = s.selected_key.clone();
            let requests = s.next_request;
            s.receive_history_inspection(project, run, inspected.clone(), cx);
            assert_eq!(s.focus.screen, Screen::History);
            assert!(s.history_inspection.is_none());
            assert_eq!(s.selected_key, key);
            assert_eq!(s.next_request, requests);
            s.analysis_policy =
                regressee_app::review::preset(regressee_app::analysis_engine::Preset::Sensitive);
            s.history_open_epoch = Some(s.history_selection_epoch);
            s.receive_history_inspection(project, run, inspected.clone(), cx);
            finish_jobs(s, cx);
            assert_eq!(s.history_inspection, Some((project, run)));
            assert_eq!(s.focus.screen, Screen::Results);
            assert!(!s.focus.candidate && !s.focus.resumable);
            s.review_tools = true;
            let requests = s.next_request;
            let analysis = s.evidence.comparison.as_ref().unwrap().revision.id;
            let selection = s.approval_selection.clone();
            let rectangles = s.mask_rectangles.clone();
            for control in [
                Control::SaveBaseline,
                Control::CheckChanges,
                Control::CaptureBaseline,
                Control::Guided(15),
                Control::Phase5(0),
                Control::Phase5(4),
                Control::Phase5(14),
                Control::Phase5(18),
                Control::Phase5(19),
                Control::Phase5(20),
                Control::Phase5(21),
                Control::Phase5(22),
                Control::Phase5(23),
                Control::Phase5(24),
                Control::Phase5(34),
                Control::Phase5(35),
                Control::AnalysisField(9),
            ] {
                assert!(
                    !s.control_mounted(control),
                    "recorded inspection mounted mutation {control:?}"
                );
                assert!(!s.control_enabled(control, cx));
                s.activate_control(control, window, cx);
            }
            for control in [
                Control::Guided(54),
                Control::Guided(46),
                Control::Viewer,
                Control::Phase5(36),
            ] {
                assert!(
                    tab_controls(s, cx).contains(&control),
                    "inspection must allow evidence navigation {control:?}"
                );
            }
            // Late controller calls must obey the same readonly contract as focus.
            for action in [0, 4, 14, 18, 20, 21, 22, 23, 24, 34, 35] {
                s.phase5_action(action, cx);
            }
            assert_eq!(s.next_request, requests);
            assert!(s.analysis_due.is_none() && s.analysis_pending.is_none());
            assert_eq!(s.approval_selection, selection);
            assert_eq!(s.mask_rectangles, rectangles);
            assert_eq!(
                s.evidence.comparison.as_ref().unwrap().revision.id,
                analysis
            );
            assert_eq!(
                s.analysis_policy, saved_policy,
                "historical sensitivity must come from the saved analysis, not today's draft"
            );
        })
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(paths).unwrap();
    assert_eq!(
        store.baseline_history(project).unwrap().len(),
        baseline_count
    );
}

#[gpui::test]
fn historical_baseline_capture_shows_only_its_saved_original(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let baseline = store.project_evidence(project).unwrap().baseline.unwrap();
    let source = baseline.references[0].run;
    let original = baseline.references[0].artifact.clone();
    let inspected = store.inspect_run(project, source).unwrap();
    assert!(inspected.comparison.is_none());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::History;
        s.run_history_index = s
            .history
            .as_ref()
            .unwrap()
            .runs
            .iter()
            .position(|r| r.run.id == source)
            .unwrap();
        s.history_open_epoch = Some(s.history_selection_epoch);
        s.receive_history_inspection(project, source, inspected, cx);
        finish_jobs(s, cx);
        assert_eq!(s.history_inspection, Some((project, source)));
        assert_eq!(s.viewer_pane_indices(), &[1]);
        assert!(s.evidence.comparison.is_none());
        assert!(!s.control_mounted(Control::Phase5(6)));
        assert!(!s.control_mounted(Control::SaveBaseline));
        assert_eq!(
            s.evidence
                .latest
                .as_ref()
                .unwrap()
                .evidence(s.selected_key.as_ref().unwrap())
                .unwrap()
                .image
                .original,
            original
        );
        assert!(s.viewer.as_ref().unwrap().request.is_none());
        assert!(!s.viewer.as_ref().unwrap().panes[1].is_empty());
    });
    owned.join_after_ui();
}

#[gpui::test]
fn keyboard_area_focus_reveals_fields_inside_bounded_tools(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.review_tools = true;
        cx.notify();
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                if !s.details.contains(&Detail::IgnoredAreas) {
                    s.activate_control(Control::Details(Detail::IgnoredAreas), window, cx);
                }
                s.activate_control(Control::Phase5(34), window, cx)
            });
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        cx.run_until_parked();
        for index in 10..=13 {
            if index > 10 {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| s.move_focus(false, window, cx));
                });
                cx.run_until_parked();
            }
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            cx.update(|window, cx| {
                shell.read_with(cx, |s, cx| {
                    assert!(
                        s.handle(Control::AnalysisField(index), cx)
                            .is_focused(window)
                    );
                    let field = s.analysis_fields[index].read(cx).rendered_bounds().unwrap();
                    let viewport = s.review_details_scroll.bounds();
                    assert_bounds_inside(field, viewport);
                });
            });
            for pane in ["phase5-before", "phase5-after"] {
                let viewport = cx.debug_bounds(pane).unwrap();
                assert!(
                    viewport.size.height >= px(96.),
                    "{width}x{height}: {pane} viewport must retain 96px: {viewport:?}; toolbar={:?}; compare={:?}; diff={:?}; zoom={:?}; details={:?}; analysis={:?}",
                    cx.debug_bounds("viewer-toolbar"),
                    cx.debug_bounds("action-Phase5(6)"),
                    cx.debug_bounds("action-Phase5(7)"),
                    cx.debug_bounds("zoom-select-button"),
                    cx.debug_bounds("action-Phase5(36)"),
                    cx.debug_bounds("action-Guided(44)"),
                );
                assert_bounds_inside(viewport, cx.debug_bounds("review-viewer").unwrap());
            }
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn viewer_disclosures_preserve_queue_image_geometry_and_drafts(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            s.review_tools = false;
            s.capture_details_open = false;
            s.analysis_fields[9].update(cx, |f, cx| f.set_value("Keep my draft", cx));
            cx.notify();
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        let queue = cx.debug_bounds("review-queue").unwrap();
        let before = cx.debug_bounds("phase5-before").unwrap();
        let after = cx.debug_bounds("phase5-after").unwrap();
        let identity = shell.read_with(cx, |s, _| {
            (s.selected_key.clone(), s.zoom, s.pan_x, s.pan_y)
        });
        for toggle in [Control::Guided(44), Control::Phase5(36)] {
            cx.update(|w, cx| shell.update(cx, |s, cx| s.activate_control(toggle, w, cx)));
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            assert!(cx.debug_bounds("review-queue").unwrap().size.height >= queue.size.height);
            assert!(cx.debug_bounds("phase5-before").unwrap().size.height >= before.size.height);
            assert!(cx.debug_bounds("phase5-after").unwrap().size.height >= after.size.height);
            assert_bounds_inside(
                cx.debug_bounds("viewer-inspector").unwrap(),
                cx.debug_bounds("review-viewer").unwrap(),
            );
            shell.read_with(cx, |s, cx| {
                assert_eq!((s.selected_key.clone(), s.zoom, s.pan_x, s.pan_y), identity);
                assert_eq!(
                    s.analysis_fields[9].read(cx).content.as_ref(),
                    "Keep my draft"
                );
                for control in [Control::CrawlField(9), Control::Guided(50)] {
                    assert!(s.control_mounted(control));
                }
                assert!(!s.control_mounted(Control::Phase5(21)));
                assert!(!s.control_mounted(Control::Phase5(24)));
                for control in [Control::ZoomIn, Control::PanLeft, Control::PanDown] {
                    assert!(!tab_controls(s, cx).contains(&control));
                }
            });
            cx.update(|w, cx| shell.update(cx, |s, cx| s.dismiss(&Dismiss, w, cx)));
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            shell.read_with(cx, |s, _| {
                assert!(!s.viewer_inspector_open() && !s.control_mounted(Control::Phase5(39)))
            });
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn viewer_scroll_zooms_at_pointer_and_keyboard_navigation_needs_no_tools(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.review_tools = false;
        s.zoom = 2.;
        s.viewer.as_mut().unwrap().width = 1200;
        s.viewer.as_mut().unwrap().height = 12000;
        s.viewer_fit = false;
        s.pan_x = 10.;
        s.pan_y = 100;
        s.viewer_bounds.set(Some(Bounds::new(
            gpui::point(px(30.), px(40.)),
            size(px(100.), px(100.)),
        )));
        assert!(tab_controls(s, cx).contains(&Control::Viewer));
        let before = s
            .viewer_transform()
            .capture_point(
                (70., 80.),
                (30., 40.),
                regressee_app::analysis_engine::Dimensions {
                    width: 120,
                    height: 1200,
                },
            )
            .unwrap();
        s.viewer_scroll(
            &gpui::ScrollWheelEvent {
                position: gpui::point(px(70.), px(80.)),
                delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(20.))),
                modifiers: gpui::Modifiers {
                    control: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            (30., 40.),
            cx,
        );
        let after = s
            .viewer_transform()
            .capture_point(
                (70., 80.),
                (30., 40.),
                regressee_app::analysis_engine::Dimensions {
                    width: 120,
                    height: 1200,
                },
            )
            .unwrap();
        assert!((before.0 as i32 - after.0 as i32).abs() <= 1);
        assert!((before.1 as i32 - after.1 as i32).abs() <= 1);
        assert!(s.zoom > 2.);
        assert!(!s.review_tools);
        let zoom = s.zoom;
        let pan = s.pan_y;
        s.viewer_scroll(
            &gpui::ScrollWheelEvent {
                delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(-24.))),
                ..Default::default()
            },
            (30., 40.),
            cx,
        );
        assert_eq!(s.zoom, zoom);
        assert!(s.pan_y > pan);
        let key = |key: &str| gpui::KeyDownEvent {
            keystroke: gpui::Keystroke::parse(key).unwrap(),
            is_held: false,
        };
        s.viewer_key(&key("1"), cx);
        assert_eq!(s.zoom, 1.);
        s.viewer_key(&key("0"), cx);
        assert!(s.viewer_fit);
        assert_eq!((s.pan_x, s.pan_y), (0., 0));
        let zoom = s.zoom;
        s.focus.dialog = true;
        s.viewer_key(&key("="), cx);
        assert_eq!(s.zoom, zoom);
        finish_jobs(s, cx);
    });
    owned.join_after_ui();
}

#[gpui::test]
fn unified_zoom_menu_preserves_fit_on_resize_and_manual_zoom_is_fixed(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        for width in [1280., 820.] {
            s.configure_review_geometry(width);
            assert!(tab_controls(s, cx).contains(&Control::Guided(46)));
            assert!(!tab_controls(s, cx).contains(&Control::Guided(52)));
            s.choose_viewer_zoom(ZoomChoice::Percent(75), cx);
            assert_eq!(s.zoom, 0.75);
            assert!(!s.viewer_fit);
            s.configure_review_geometry(width + 100.);
            assert_eq!(s.zoom, 0.75);
            s.choose_viewer_zoom(ZoomChoice::FitWidth, cx);
            assert!(s.viewer_fit);
            assert_eq!(s.zoom, s.fitted_viewer_zoom());
            assert_eq!((s.pan_x, s.pan_y), (0., 0));
        }
        s.focus.dialog = true;
        let zoom = s.zoom;
        s.choose_viewer_zoom(ZoomChoice::Percent(200), cx);
        assert_eq!(s.zoom, zoom);
        finish_jobs(s, cx);
    });
    owned.join_after_ui();
}

#[gpui::test]
fn project_loading_never_claims_missing_screenshots_before_evidence_arrives(
    cx: &mut TestAppContext,
) {
    use crate::loading_ux::ScreenshotState;
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        finish_jobs(s, cx);
        s.focus.screen = Screen::Results;
        s.queue(Work::Load(project), cx);
        assert_eq!(s.progress.as_ref().unwrap().0, "Opening project…");
        let update = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
        s.apply_update(update, cx);
        assert!(s.initial_evidence_loading());
        assert_eq!(s.screenshot_state(), ScreenshotState::LoadingEvidence);
        assert_eq!(
            s.progress.as_ref().unwrap().0,
            "Loading login and environment settings…"
        );
        assert!(!s.control_mounted(Control::CaptureBaseline));
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert!(cx.debug_bounds("results-loading").is_some());
        assert!(cx.debug_bounds("loading-activity").is_some());
    }
    shell.update(cx, |s, cx| {
        finish_jobs(s, cx);
        assert!(!s.initial_evidence_loading());
        assert!(s.viewer_has_pixels());
        s.clear_viewer();
        s.reload_viewer(cx);
        assert_eq!(s.screenshot_state(), ScreenshotState::LoadingImage);
        finish_jobs(s, cx);
        s.evidence = ProjectEvidence::default();
        s.clear_viewer();
        s.evidence_dirty = false;
        assert_eq!(s.screenshot_state(), ScreenshotState::Empty);
        s.evidence_dirty = true;
        s.error = Some("Synthetic disk-read failure".into());
        assert_eq!(s.screenshot_state(), ScreenshotState::Unavailable);
        assert!(!s.initial_evidence_loading());
        assert!(s.control_mounted(Control::RefreshEvidence));
        assert!(!s.control_mounted(Control::CaptureBaseline));
        s.error = None;
        s.focus.capturing = true;
        assert_eq!(s.screenshot_state(), ScreenshotState::Capturing);
    });
    owned.join_after_ui();
}

#[gpui::test]
fn capture_details_are_readable_and_missing_images_have_no_navigation_tools(
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
        let summary = format!("{:?}", s.capture_summary());
        let key = s.selected_key.as_ref().unwrap();
        assert!(!summary.contains(&key.page.to_string()));
        assert!(!summary.contains(&key.profile.to_string()));
        assert!(!summary.contains("Some("));
        assert!(summary.contains("Screen size") && summary.contains("UTC"));
        let key = key.clone();
        // A historical capture has no comparison baseline pointer, but may
        // still be a source in a saved baseline. Do not call it unsaved.
        let baseline = s.evidence.baseline.take().unwrap();
        let source = baseline
            .references
            .iter()
            .find(|r| r.key == key)
            .unwrap()
            .run;
        let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
        s.evidence.latest = Some(store.load_run(source).unwrap());
        s.evidence.baselines = vec![baseline];
        s.evidence.comparison = None;
        let summary = format!("{:?}", s.capture_summary());
        assert!(summary.contains("Saved baseline screenshot"));
        assert!(!summary.contains("not saved as a baseline"));
        let target = s
            .evidence
            .latest
            .as_mut()
            .unwrap()
            .run
            .targets
            .iter_mut()
            .find(|t| t.key == key)
            .unwrap();
        target.outcome = TargetOutcome::Failed {
            code: regressee_core::domain::CaptureError::ImageLimit,
        };
        for pane in &mut s.viewer.as_mut().unwrap().panes {
            pane.clear();
        }
        let summary = format!("{:?}", s.capture_summary());
        assert!(summary.contains("visible-screen-only capture"));
        assert!(s.control_mounted(Control::Phase5(36)));
        assert!(!s.control_mounted(Control::Guided(44)));
        assert!(!s.control_mounted(Control::Guided(46)));
        assert!(!s.control_mounted(Control::Viewer));
    });
    owned.join_after_ui();
}

#[gpui::test]
fn selected_history_rows_fill_the_painted_list_width(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::History;
        cx.notify();
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for (section, control, list, row) in [
            (
                0,
                Control::HistoryRun(0),
                "history-run-list",
                "history-row-HistoryRun(0)",
            ),
            (
                1,
                Control::HistoryBaseline(0),
                "history-baseline-list",
                "history-row-HistoryBaseline(0)",
            ),
        ] {
            shell.update(cx, |s, cx| {
                s.history_section = section;
                cx.notify();
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    assert!(s.control_enabled(control, cx));
                    s.reveal_history_control(control);
                    window.focus(&s.handle(control, cx));
                    s.activate_control(control, window, cx);
                });
            });
            cx.run_until_parked();
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let row = cx.debug_bounds(row).unwrap();
            let list = cx.debug_bounds(list).unwrap();
            assert!(
                (f32::from(row.size.width) - f32::from(list.size.width)).abs() <= 1.,
                "{width}x{height}: selected {control:?} must fill list width: {row:?} in {list:?}"
            );
            assert_bounds_inside(row, list);
            shell.read_with(cx, |s, _| match control {
                Control::HistoryRun(index) => assert_eq!(s.run_history_index, index),
                Control::HistoryBaseline(index) => assert_eq!(s.history_index, index),
                _ => unreachable!(),
            });
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
fn filtered_empty_message_fits_queue_without_rebinding_evidence_or_drafts(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.review_tools = false;
        s.result_filters_open = true;
        s.mask_dirty = true;
        s.analysis_fields[9].update(cx, |field, cx| {
            field.set_value("Preserved review draft", cx)
        });
        s.analysis_fields[10].update(cx, |field, cx| field.set_value("unfinished geometry", cx));
        cx.notify();
    });
    cx.run_until_parked();
    let (key, identity, rectangles, requests) = shell.read_with(cx, |s, _| {
        (
            s.selected_key.clone(),
            s.viewer.as_ref().unwrap().identity.clone(),
            s.mask_rectangles.clone(),
            s.next_request,
        )
    });
    shell.update(cx, |s, cx| {
        s.crawl_fields[9].update(cx, |field, cx| {
            field.set_value("no-matching-synthetic-page", cx)
        });
    });
    cx.run_until_parked();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert_bounds_inside(
            cx.debug_bounds("review-empty-message").unwrap(),
            cx.debug_bounds("review-queue").unwrap(),
        );
        shell.read_with(cx, |s, cx| {
            assert!(s.result_indices().is_empty());
            assert_eq!(s.selected_key, key);
            assert_eq!(s.viewer.as_ref().unwrap().identity, identity);
            assert_eq!(s.mask_rectangles, rectangles);
            assert!(s.mask_dirty && !s.focus.view_loading);
            assert_eq!(s.next_request, requests);
            assert_eq!(
                s.analysis_fields[9].read(cx).content.as_ref(),
                "Preserved review draft"
            );
            assert_eq!(
                s.analysis_fields[10].read(cx).content.as_ref(),
                "unfinished geometry"
            );
        });
    }
    owned.join_after_ui();
}

#[gpui::test]
fn keyboard_help_copy_fits_default_body_with_protected_title_and_close(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, finish_jobs);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| s.show_keyboard_help(window, cx));
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let dialog = cx.debug_bounds("information-dialog").unwrap();
        let body = cx.debug_bounds("information-dialog-body").unwrap();
        let copy = cx.debug_bounds("information-dialog-copy").unwrap();
        let title = cx.debug_bounds("information-dialog-title").unwrap();
        let close = cx.debug_bounds("action-CloseDialog").unwrap();
        assert_bounds_inside(
            dialog,
            gpui::Bounds::new(gpui::point(px(0.), px(0.)), size(px(width), px(height))),
        );
        assert_bounds_inside(body, dialog);
        assert_bounds_inside(copy, body);
        assert_bounds_inside(title, dialog);
        assert_bounds_inside(close, dialog);
        assert!(title.bottom() <= body.top());
        assert!(body.bottom() <= close.top());
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert_eq!(s.focused(window, cx), Some(Control::CloseDialog));
                s.activate_control(Control::CloseDialog, window, cx);
            });
        });
    }
    owned.join_after_ui();
}

#[gpui::test]
fn keyboard_areas_validate_original_pixels_and_save_only_on_explicit_action(
    cx: &mut TestAppContext,
) {
    use regressee_app::analysis_engine::Rectangle;
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let paths = AppPaths::isolated(dir.path());
    let (jobs, events) = Jobs::start_without_runtime(paths.clone());
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    let mut saved_key = None;
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.review_tools = true;
            s.mask_rectangles.clear();
            s.activate_control(Control::Details(Detail::IgnoredAreas), window, cx);
            s.mask_selected = None;
            s.mask_dirty = false;
            let original_pair = s.viewer.as_ref().unwrap().request.as_ref().unwrap().clone();
            saved_key = s.selected_key.clone();
            let requests = s.next_request;
            for restriction in 0..3 {
                s.focus.busy = restriction == 0;
                s.focus.approval_dialog = restriction == 1;
                s.native_prompt = restriction == 2;
                assert!(!s.control_enabled(Control::Phase5(34), cx));
                s.activate_control(Control::Phase5(34), window, cx);
                assert!(s.mask_rectangles.is_empty());
                assert_eq!(s.next_request, requests);
            }
            s.focus.busy = false;
            s.focus.approval_dialog = false;
            s.native_prompt = false;
            let identity = s.viewer.as_ref().unwrap().identity.clone();
            s.viewer.as_mut().unwrap().identity.as_mut().unwrap().run =
                regressee_core::domain::RunId::new();
            assert!(
                !s.control_enabled(Control::Phase5(34), cx),
                "stale pair identity cannot create ignored geometry"
            );
            s.activate_control(Control::Phase5(34), window, cx);
            assert!(s.mask_rectangles.is_empty());
            s.viewer.as_mut().unwrap().identity = identity;
            s.zoom = 0.3;
            s.pan_x = 17.;
            s.pan_y = 13;
            assert!(tab_controls(s, cx).contains(&Control::Phase5(34)));
            s.activate_control(Control::Phase5(34), window, cx);
            assert_eq!(s.mask_rectangles.len(), 1);
            assert_eq!(s.mask_selected, Some(0));
            assert!(s.mask_dirty);
            assert_eq!(s.presentation_focus, Some(Control::AnalysisField(10)));
            for index in 10..=13 {
                assert!(tab_controls(s, cx).contains(&Control::AnalysisField(index)));
            }
            for (i, value) in ["7", "9", "12", "14"].into_iter().enumerate() {
                s.analysis_fields[10 + i].update(cx, |f, cx| f.set_value(value, cx));
            }
            s.activate_control(Control::Phase5(35), window, cx);
            let exact = Rectangle {
                x: 7,
                y: 9,
                width: 12,
                height: 14,
            };
            assert_eq!(s.mask_rectangles, vec![exact]);
            assert_eq!(
                s.next_request, requests,
                "creation and Apply must not write storage"
            );
            let key = s.selected_key.clone();
            s.activate_control(Control::Phase5(33), window, cx);
            assert_eq!(
                s.selected_key, key,
                "Next must not rebind unsaved ignored geometry"
            );
            for invalid in ["1.5", "-1", "4294967295"] {
                s.analysis_fields[10].update(cx, |f, cx| f.set_value(invalid, cx));
                s.activate_control(Control::Phase5(35), window, cx);
                assert_eq!(s.mask_rectangles, vec![exact]);
                assert_eq!(s.next_request, requests);
            }
            s.analysis_fields[10].update(cx, |f, cx| f.set_value("7", cx));
            s.analysis_fields[12].update(cx, |f, cx| f.set_value("0", cx));
            s.activate_control(Control::Phase5(35), window, cx);
            assert_eq!(s.mask_rectangles, vec![exact]);
            s.analysis_fields[12].update(cx, |f, cx| f.set_value("12", cx));
            s.error = None;
            s.activate_control(Control::Phase5(15), window, cx);
            assert_eq!(s.analysis_fields[10].read(cx).content.as_ref(), "7");
            s.activate_control(Control::Phase5(18), window, cx);
            finish_jobs(s, cx);
            assert!(!s.mask_dirty);
            s.analysis_due = None;
            let current_pair = s.viewer.as_ref().unwrap().request.as_ref().unwrap();
            assert_eq!(current_pair.before, original_pair.before);
            assert_eq!(current_pair.after, original_pair.after);
            assert_eq!(s.mask_rectangles, vec![exact]);
        })
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(paths).unwrap();
    let mask = store
        .latest_mask(project, &saved_key.unwrap())
        .unwrap()
        .unwrap();
    assert_eq!(
        mask.rectangles,
        vec![Rectangle {
            x: 7,
            y: 9,
            width: 12,
            height: 14
        }]
    );
}

#[gpui::test]
fn history_inspection_reply_cannot_overwrite_new_review_or_numeric_drafts(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, run, _, _) = approval_fixture::fixture(dir.path());
    let inspected = store.inspect_run(project, run).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::History;
        s.run_history_index = s
            .history
            .as_ref()
            .unwrap()
            .runs
            .iter()
            .position(|r| r.run.id == run)
            .unwrap();
        let key = s.selected_key.clone();
        let analysis = s.evidence.comparison.as_ref().unwrap().revision.id;
        let viewer_identity = s.viewer.as_ref().unwrap().identity.clone();
        s.analysis_fields[9].update(cx, |f, cx| {
            f.set_value("New edit while inspection was loading", cx)
        });
        assert!(s.review_unsaved(cx));
        s.history_open_epoch = Some(s.history_selection_epoch);
        s.receive_history_inspection(project, run, inspected.clone(), cx);
        assert!(s.history_inspection.is_none());
        assert_eq!(s.focus.screen, Screen::History);
        assert_eq!(s.selected_key, key);
        assert_eq!(
            s.evidence.comparison.as_ref().unwrap().revision.id,
            analysis
        );
        assert_eq!(s.viewer.as_ref().unwrap().identity, viewer_identity);
        assert_eq!(
            s.analysis_fields[9].read(cx).content.as_ref(),
            "New edit while inspection was loading"
        );
        s.discard_review_drafts(cx);
        s.mask_rectangles = vec![regressee_app::analysis_engine::Rectangle {
            x: 1,
            y: 2,
            width: 3,
            height: 4,
        }];
        s.mask_selected = Some(0);
        s.load_mask_fields(cx);
        s.mask_dirty = false;
        s.analysis_fields[10].update(cx, |f, cx| f.set_value("unfinished", cx));
        assert!(s.mask_geometry_dirty(cx));
        s.history_open_epoch = Some(s.history_selection_epoch);
        s.receive_history_inspection(project, run, inspected, cx);
        assert!(s.history_inspection.is_none());
        assert_eq!(s.selected_key, key);
        assert_eq!(s.viewer.as_ref().unwrap().identity, viewer_identity);
        assert_eq!(
            s.mask_rectangles,
            vec![regressee_app::analysis_engine::Rectangle {
                x: 1,
                y: 2,
                width: 3,
                height: 4
            }]
        );
        assert_eq!(
            s.analysis_fields[10].read(cx).content.as_ref(),
            "unfinished"
        );
    });
    owned.join_after_ui();
}

#[gpui::test]
fn validation_correction_reveals_the_right_stage_without_submission_and_help_restores_focus(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let baseline_count = store.baseline_history(project).unwrap().len();
    drop(store);
    let paths = AppPaths::isolated(dir.path());
    let (jobs, events) = Jobs::start_without_runtime(paths.clone());
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Website;
            s.focus.advanced = false;
            s.last_validation_field = None;
            s.crawl_fields[0].update(cx, |f, cx| {
                f.set_value("bad", cx);
                f.touched = true;
            });
            assert!(
                s.form_errors(cx)
                    .iter()
                    .any(|e| e.0 == Control::CrawlField(0))
            );
            assert!(tab_controls(s, cx).contains(&Control::ValidationNext));
            let requests = s.next_request;
            s.activate_control(Control::ValidationNext, window, cx);
            assert_eq!(s.focus.setup_stage, SetupStage::Pages);
            assert_eq!(s.presentation_focus, Some(Control::CrawlField(0)));
            assert!(s.control_mounted(Control::CrawlField(0)));
            assert!(s.control_enabled(Control::CrawlField(0), cx));
            assert_eq!(s.next_request, requests);
            assert!(s.pending.is_none());
            s.crawl_fields[0].update(cx, |f, cx| f.set_value("500", cx));
            s.crawl_fields[3].update(cx, |f, cx| {
                f.set_value("bad", cx);
                f.touched = true;
            });
            s.last_validation_field = None;
            s.activate_control(Control::ValidationNext, window, cx);
            assert!(
                s.focus.advanced,
                "advanced correction must reveal its field"
            );
            assert_eq!(s.presentation_focus, Some(Control::CrawlField(3)));
            assert!(s.control_mounted(Control::CrawlField(3)));
            assert_eq!(s.next_request, requests);
            let original = Control::CrawlField(3);
            window.focus(&s.handle(original, cx));
            let baseline = s.project.as_ref().unwrap().project.active_baseline;
            s.show_keyboard_help(window, cx);
            assert!(s.help_dialog && s.focus.dialog);
            assert_eq!(
                s.focus.controls(),
                [Control::AboutRepository, Control::CloseDialog]
            );
            assert_eq!(tab_controls(s, cx), vec![Control::CloseDialog]);
            assert_eq!(s.focused(window, cx), Some(Control::CloseDialog));
            s.activate_control(Control::SaveBaseline, window, cx);
            s.activate_control(Control::Phase5(21), window, cx);
            assert_eq!(s.next_request, requests);
            s.activate_control(Control::CloseDialog, window, cx);
            assert!(!s.help_dialog && !s.focus.dialog);
            assert_eq!(s.focused(window, cx), Some(original));
            assert_eq!(
                s.project.as_ref().unwrap().project.active_baseline,
                baseline
            );
            assert_eq!(s.next_request, requests);
        })
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(paths).unwrap();
    assert_eq!(
        store.baseline_history(project).unwrap().len(),
        baseline_count
    );
}
