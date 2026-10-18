// SPDX-License-Identifier: MPL-2.0
//! Isolated-storage and simulated GPUI consent/recovery tests. No website visits.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::TestAppContext;
use regressee_core::domain::*;

fn incomplete_capture(path: &std::path::Path) -> (regressee_storage::Store, ProjectId) {
    let mut store = regressee_storage::Store::open(AppPaths::isolated(path)).unwrap();
    let mut project =
        Project::draft("Recorded pages fixture".into(), "https://example.test/").unwrap();
    project.profiles[0].runtime = RuntimeIdentity::Pinned {
        node: "fixture".into(),
        worker: "fixture".into(),
        playwright: "fixture".into(),
        browser: "fixture".into(),
    };
    store.save_project(&project, None).unwrap();
    let inventory = InventoryRevision {
        id: InventoryId::new(),
        project: project.id,
        identity_version: 2,
        pages: (0..579)
            .map(|i| InventoryPage {
                id: PageId::new(),
                canonical: if i == 1 {
                    "https://other.test/login".parse().unwrap()
                } else {
                    project.sources[0].join(&format!("pages/{i}")).unwrap()
                },
                included: i == 0,
                reason: (i > 0).then(|| {
                    if i == 2 {
                        "Excluded in preview"
                    } else {
                        "Website not approved — add and confirm it as a starting address"
                    }
                    .into()
                }),
            })
            .collect(),
        coverage: DiscoveryCoverage::Limited {
            reason: "Synthetic limited discovery".into(),
        },
    };
    store.freeze_inventory(&inventory).unwrap();
    let mut run = regressee_core::evidence::RunRecord {
        run: CaptureRun {
            capture_plan: Default::default(),
            id: RunId::new(),
            project: project.id,
            intent: RunIntent::NewBaseline,
            profiles: project.profiles.clone(),
            environment: project.environments[0].clone(),
            rendering: RenderingEnvironment {
                os: "fixture".into(),
                architecture: "fixture".into(),
                browser: "fixture".into(),
                worker_version: "fixture".into(),
                profile_version: 1,
                fonts_fingerprint: Some("fixture".into()),
            },
            state: JobState::Running,
            started_at: Some(100),
            finished_at: None,
            targets: vec![CaptureTarget {
                dom_outcome: Default::default(),
                key: TargetKey {
                    page: inventory.pages[0].id,
                    profile: project.profiles[0].id,
                },
                requested: inventory.pages[0].canonical.clone(),
                final_url: None,
                attempts: 1,
                outcome: TargetOutcome::Pending,
                warnings: vec![],
            }],
            inventory,
        },
        capture: None,
        dom_captures: vec![],
        captures: vec![],
        queue: None,
    };
    store.begin_run(&run).unwrap();
    run.run.state = JobState::CompletedWithErrors;
    run.run.finished_at = Some(101);
    run.run.targets[0].outcome = TargetOutcome::Failed {
        code: CaptureError::Timeout,
    };
    store.finish_run(&run).unwrap();
    (store, project.id)
}

#[gpui::test]
fn recorded_pages_remain_inspectable_and_reusable_without_a_saved_baseline(
    cx: &mut TestAppContext,
) {
    recorded_pages(cx, false);
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "bounded recorded-page recovery capture with isolated synthetic storage"]
fn renderer_capture_recorded_page_recovery(cx: &mut TestAppContext) {
    recorded_pages(cx, true);
}

fn recorded_pages(cx: &mut TestAppContext, render: bool) {
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    let root = render.then(|| {
        let root = crate::ui_renderer_capture::output_root().unwrap();
        cx.enable_offscreen_renderer().unwrap();
        root
    });
    #[cfg(not(all(feature = "ui-renderer-capture", target_os = "linux")))]
    assert!(!render);
    let dir = tempfile::tempdir().unwrap();
    let (store, project) = incomplete_capture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        finish_jobs(s, cx);
        s.focus.screen = Screen::Results;
        s.queue(Work::Load(project), cx);
        finish_jobs(s, cx);
        s.focus.review_surface = ReviewSurface::Pages;
        s.notice = None;
        assert!(s.evidence.baseline.is_none());
        assert!(s.preview.is_none());
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        let (first, second, last) = shell.update(cx, |s, cx| {
            s.configure_review_geometry(width);
            s.result_filters_open = false;
            s.search.clear();
            s.selected_page = None;
            s.preview_list_scroll
                .scroll_to_item(0, gpui::ScrollStrategy::Top);
            let pages = &s.evidence.latest.as_ref().unwrap().run.inventory.pages;
            let ids = (pages[0].id, pages[1].id, pages.last().unwrap().id);
            assert!(s.control_enabled(Control::Guided(16), cx));
            assert!(s.control_enabled(Control::Guided(45), cx));
            assert!(s.control_enabled(Control::Guided(24), cx));
            assert!(!s.control_enabled(Control::DiscardPageList, cx));
            cx.notify();
            ids
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let row = cx
            .debug_bounds(format!("page-row-fixed-page-{first}").leak())
            .unwrap();
        cx.simulate_click(row.center(), gpui::Modifiers::none());
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert_eq!(s.selected_page, Some(first));
                assert!(s.options_panel_open());
                s.activate_control(Control::Guided(24), window, cx);
                assert_eq!(s.selected_page, Some(second));
                assert!(!s.control_enabled(Control::Guided(22), cx));
            });
            let _ = window.draw(cx);
        });
        assert!(cx.debug_bounds("recorded-selected-address").is_some());
        let panel = cx.debug_bounds("page-options-panel").unwrap();
        let list = cx.debug_bounds("fixed-inventory").unwrap();
        assert!(list.size.width >= px(198.) && list.size.height >= px(128.));
        assert!(list.right() <= panel.left());
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if let Some(root) = &root {
            crate::ui_renderer_capture::capture(
                cx,
                root,
                &format!("{width:.0}x{height:.0}-recorded-page-details"),
            )
            .unwrap();
        }
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Guided(23), window, cx);
                assert_eq!(s.selected_page, Some(first));
                s.select_page_list_page(last);
                s.activate_control(Control::Guided(23), window, cx);
            });
            let _ = window.draw(cx);
        });
        let selected = shell.read_with(cx, |s, _| s.selected_page.unwrap());
        let row = cx
            .debug_bounds(format!("page-row-fixed-page-{selected}").leak())
            .unwrap();
        assert!(row.top() >= list.top() && row.bottom() <= list.bottom());
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::CloseOptions, window, cx);
                s.search = "no-matching-pages".into();
                s.select_page_list_page(first);
                assert_eq!(s.selected_page, Some(selected));
                assert!(!s.options_panel_open());
                assert!(!s.control_enabled(Control::Guided(24), cx));
                s.search.clear();
                s.activate_control(Control::RefreshEvidence, window, cx);
                finish_jobs(s, cx);
                assert_eq!(s.page_list_page_ids().len(), 579);
                assert!(s.preview.is_none());
            });
        });
    }
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            let inventory = s.evidence.latest.as_ref().unwrap().run.inventory.clone();
            s.activate_control(Control::Guided(16), window, cx);
            finish_jobs(s, cx);
            let p = s.preview.as_ref().unwrap();
            assert_ne!(
                p.inventory.id, inventory.id,
                "a new capture cannot consume or mutate the recorded inventory"
            );
            assert_eq!(p.inventory.pages, inventory.pages);
            assert_eq!(p.inventory.coverage, inventory.coverage);
            assert_eq!(p.visited, 0);
            assert!(s.control_enabled(Control::CaptureBaseline, cx));
            s.select_preview_page(inventory.pages[1].id);
            assert!(
                !s.control_enabled(Control::Guided(22), cx),
                "reuse must not bypass blocked inclusion"
            );
            s.select_preview_page(inventory.pages[2].id);
            s.activate_control(Control::Guided(22), window, cx);
            assert!(s.preview.as_ref().unwrap().inventory.pages[2].included);
            assert!(!s.evidence.latest.as_ref().unwrap().run.inventory.pages[2].included);
        });
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
        before
    );
    assert!(
        store
            .load_project(project)
            .unwrap()
            .project
            .active_baseline
            .is_none()
    );
}

#[gpui::test]
fn page_list_discard_requires_consent_and_cancel_preserves_every_choice(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    let (inventory, requests) = cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            options_ux::fixture_discovery(s);
            s.preview.as_mut().unwrap().inventory.id = InventoryId::new();
            s.select_preview_page(s.selected_page.unwrap());
            s.activate_control(Control::Guided(22), window, cx);
            let inventory = s.preview.as_ref().unwrap().inventory.clone();
            let requests = s.next_request;
            s.activate_control(Control::DiscardPageList, window, cx);
            assert!(s.native_prompt);
            assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
            assert_eq!(s.next_request, requests);
            assert!(!s.control_enabled(Control::CaptureBaseline, cx));
            (inventory, requests)
        })
    });
    let (title, message) = cx.pending_prompt().unwrap();
    assert!(title.contains("Discard discovered pages"));
    assert!(message.contains("128 page addresses") && message.contains("include/exclude"));
    assert!(message.contains("does not delete recorded screenshots"));
    cx.simulate_prompt_answer("Keep page list");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert!(!s.native_prompt);
            assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
            assert_eq!(s.next_request, requests);
            assert!(s.handle(Control::DiscardPageList, cx).is_focused(window));
            s.activate_control(Control::CloseOptions, window, cx);
            assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
            s.select_preview_page(s.selected_page.unwrap());
            s.dismiss(&Dismiss, window, cx);
            assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
            s.select_preview_page(s.selected_page.unwrap());
            s.activate_control(Control::DiscardPageList, window, cx);
        });
    });
    cx.simulate_prompt_answer("Discard pages");
    cx.run_until_parked();
    shell.read_with(cx, |s, cx| {
        assert!(s.preview.is_none());
        assert!(!s.focus.preview && !s.result_filters_open);
        assert_eq!(s.next_request, requests);
        assert!(s.control_enabled(Control::Guided(16), cx));
        assert!(s.control_enabled(Control::Guided(45), cx));
        assert!(s.control_enabled(Control::Guided(24), cx));
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
        before
    );
}

#[gpui::test]
fn stale_discard_confirmation_cannot_erase_a_new_page_list(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            options_ux::fixture_discovery(s);
            s.select_preview_page(s.selected_page.unwrap());
            s.activate_control(Control::DiscardPageList, window, cx);
            s.preview.as_mut().unwrap().inventory.id = InventoryId::new();
        });
    });
    let replacement = shell.read_with(cx, |s, _| s.preview.as_ref().unwrap().inventory.clone());
    cx.simulate_prompt_answer("Discard pages");
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        assert_eq!(s.preview.as_ref().unwrap().inventory, replacement)
    });
    owned.join_after_ui();
}

#[gpui::test]
fn destructive_storage_actions_require_popup_consent_and_reject_stale_previews(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::History;
        s.history_section = 2;
        s.queue(Work::RepairPreview, cx);
        finish_jobs(s, cx);
        s.task = None;
        s.repair_preview
            .as_mut()
            .unwrap()
            .temporaries
            .push("synthetic-temporary".into());
        s.repair_preview
            .as_mut()
            .unwrap()
            .valid_unreferenced
            .push("synthetic-original".into());
        s.deletion_preview = Some(regressee_storage::DeletionPreview {
            project,
            revision: s.project.as_ref().unwrap().revision,
            item: regressee_storage::DeleteItem::Run(s.evidence.latest.as_ref().unwrap().run.id),
            fingerprint: "synthetic-preview".into(),
            bytes: 42,
            shared_bytes: 84,
            objects: 1,
            analyses: 2,
            scope: None,
        });
    });
    for (control, confirm, consequence) in [
        (Control::Phase6(6), "Delete permanently", "cannot be undone"),
        (
            Control::Phase6(14),
            "Discard temporary files",
            "Files still being saved",
        ),
        (
            Control::Phase6(15),
            "Delete recovery originals",
            "screenshots you could otherwise recover",
        ),
    ] {
        let requests = shell.read_with(cx, |s, _| s.next_request);
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert!(s.control_enabled(control, cx));
                s.activate_control(control, window, cx);
                assert!(s.native_prompt);
                assert_eq!(s.next_request, requests);
            });
        });
        assert!(cx.pending_prompt().unwrap().1.contains(consequence));
        cx.simulate_prompt_answer("Cancel");
        cx.run_until_parked();
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert_eq!(s.next_request, requests);
                s.activate_control(control, window, cx);
                if control == Control::Phase6(6) {
                    s.deletion_preview
                        .as_mut()
                        .unwrap()
                        .fingerprint
                        .push_str("-changed");
                } else {
                    s.repair_preview
                        .as_mut()
                        .unwrap()
                        .fingerprint
                        .push_str("-changed");
                }
            });
        });
        cx.simulate_prompt_answer(confirm);
        cx.run_until_parked();
        shell.read_with(cx, |s, _| {
            assert!(!s.native_prompt);
            assert_eq!(
                s.next_request, requests,
                "stale consent must not submit a mutation"
            );
        });
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
        before
    );
}

#[gpui::test]
fn confirmed_history_deletion_removes_only_the_reviewed_record(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project, original, _, _) = approval_fixture::fixture(dir.path());
    // The original fixture check is protected by retained mask history. Add a
    // distinct check without a mask; never weaken that protection for this test.
    let mut run = store.load_run(original).unwrap();
    let target = run.run.targets[0].clone();
    let captures = std::mem::take(&mut run.captures);
    run.run.id = RunId::new();
    let check = run.run.id;
    run.run.state = JobState::Running;
    run.run.finished_at = None;
    run.run.targets[0].outcome = TargetOutcome::Pending;
    run.run.targets[0].final_url = None;
    store.begin_run(&run).unwrap();
    run.run.targets[0] = target;
    run.captures = captures;
    run.run.state = JobState::Completed;
    run.run.finished_at = Some(103);
    store.finish_run(&run).unwrap();
    let baseline = store.load_project(project).unwrap().project.active_baseline;
    let preview = store
        .deletion_preview(project, regressee_storage::DeleteItem::Run(check))
        .unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    let requests = cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::History;
            s.deletion_preview = Some(preview.clone());
            let requests = s.next_request;
            s.activate_control(Control::Phase6(6), window, cx);
            assert!(s.native_prompt);
            assert_eq!(s.next_request, requests);
            requests
        })
    });
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert_eq!(s.next_request, requests);
            assert_eq!(s.deletion_preview.as_ref(), Some(&preview));
            s.activate_control(Control::Phase6(6), window, cx);
        });
    });
    cx.simulate_prompt_answer("Delete permanently");
    cx.run_until_parked();
    shell.update(cx, |s, cx| {
        assert!(s.next_request > requests);
        finish_jobs(s, cx);
        assert!(s.error.is_none(), "{:?}", s.error);
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert!(store.load_run(check).is_err());
    assert_eq!(
        store.load_project(project).unwrap().project.active_baseline,
        baseline
    );
    let saved = store.load_baseline(baseline.unwrap()).unwrap();
    for reference in saved.references {
        assert!(store.load_run(reference.run).is_ok());
    }
}

#[gpui::test]
fn retained_baseline_restoration_requires_consent_and_preserves_both_versions(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let record = store.load_project(project).unwrap();
    let old = record.project.active_baseline.unwrap();
    let run = store.load_baseline(old).unwrap().references[0].run;
    let new = store
        .activate_baseline(run, record.revision)
        .unwrap()
        .project
        .active_baseline
        .unwrap();
    assert_ne!(old, new);
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    let requests = cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::History;
            s.history_section = 1;
            s.history_index = s
                .evidence
                .baselines
                .iter()
                .position(|b| b.id == old)
                .unwrap();
            let requests = s.next_request;
            s.activate_control(Control::Phase5(42), window, cx);
            assert!(s.native_prompt);
            assert_eq!(s.next_request, requests);
            requests
        })
    });
    assert!(
        cx.pending_prompt()
            .unwrap()
            .1
            .contains("Future checks will compare against the baseline saved")
    );
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert_eq!(
                s.project.as_ref().unwrap().project.active_baseline,
                Some(new)
            );
            assert_eq!(s.next_request, requests);
            s.activate_control(Control::Phase5(42), window, cx);
        });
    });
    cx.simulate_prompt_answer("Use baseline");
    cx.run_until_parked();
    shell.update(cx, |s, cx| {
        assert!(s.next_request > requests);
        finish_jobs(s, cx);
        assert!(s.error.is_none(), "{:?}", s.error);
        assert_eq!(
            s.project.as_ref().unwrap().project.active_baseline,
            Some(old)
        );
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert!(store.load_baseline(old).is_ok());
    assert!(store.load_baseline(new).is_ok());
}

#[gpui::test]
fn forget_login_and_browser_replacement_do_not_start_before_consent(cx: &mut TestAppContext) {
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
        let session = SessionReference {
            id: SessionId::new(),
            project,
            environment: s.environment_draft.unwrap(),
        };
        s.project.as_mut().unwrap().project.environments[0].session = Some(session.clone());
        s.draft_settings.environments[0].session = Some(session);
    });
    let requests = shell.read_with(cx, |s, _| s.next_request);
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert!(s.control_enabled(Control::Guided(34), cx));
            s.activate_control(Control::Guided(34), window, cx);
            assert_eq!(s.next_request, requests);
        });
    });
    assert!(cx.pending_prompt().unwrap().1.contains("log in again"));
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert_eq!(s.next_request, requests);
            assert!(
                s.project.as_ref().unwrap().project.environments[0]
                    .session
                    .is_some()
            );
            s.activate_control(Control::Guided(34), window, cx);
        });
    });
    cx.simulate_prompt_answer("Forget login");
    cx.run_until_parked();
    shell.update(cx, |s, cx| {
        assert!(s.next_request > requests);
        finish_jobs(s, cx);
        s.task = None;
        s.focus.screen = Screen::Settings;
        s.runtime_readiness = Some(regressee_app::product::RuntimeReadiness {
            status: regressee_app::product::RuntimeState::RepairRequired,
            setup_available: true,
            message: "Synthetic repair required",
            identity: None,
            scope: None,
        });
    });
    let requests = shell.read_with(cx, |s, _| s.next_request);
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            s.activate_control(Control::SetupRuntime, window, cx);
            assert!(s.native_prompt);
            assert_eq!(s.next_request, requests);
        });
    });
    assert!(
        cx.pending_prompt()
            .unwrap()
            .1
            .contains("browser version used for the baseline")
    );
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    shell.read_with(cx, |s, _| assert_eq!(s.next_request, requests));
    owned.join_after_ui();
}

#[gpui::test]
fn replacing_discovery_and_quitting_warn_about_the_unsaved_page_list(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    let (inventory, requests) = cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            options_ux::fixture_discovery(s);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Settings;
            let inventory = s.preview.as_ref().unwrap().inventory.clone();
            let requests = s.next_request;
            s.activate_control(Control::Guided(13), window, cx);
            assert!(s.native_prompt);
            assert_eq!(s.next_request, requests);
            (inventory, requests)
        })
    });
    assert!(
        cx.pending_prompt()
            .unwrap()
            .1
            .contains("include/exclude choices")
    );
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
            assert_eq!(s.next_request, requests);
            s.request_exit(window, cx);
            assert!(s.native_prompt);
        });
    });
    assert!(
        cx.pending_prompt()
            .unwrap()
            .1
            .contains("discarded when you quit")
    );
    cx.simulate_prompt_answer("Keep page list");
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        assert!(!s.native_prompt);
        assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
        assert_eq!(s.next_request, requests);
    });
    owned.join_after_ui();
}

#[gpui::test]
fn same_project_settings_save_keeps_discovery_and_new_project_requires_discard_consent(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    let inventory = cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            options_ux::fixture_discovery(s);
            s.preview.as_mut().unwrap().inventory.id = InventoryId::new();
            s.select_preview_page(s.selected_page.unwrap());
            s.activate_control(Control::Guided(22), window, cx);
            let inventory = s.preview.as_ref().unwrap().inventory.clone();
            s.navigate(Screen::Setup, window, cx);
            s.name
                .update(cx, |f, cx| f.set_value("Updated fixture name", cx));
            s.activate_control(Control::Save, window, cx);
            finish_jobs(s, cx);
            assert_eq!(
                s.project.as_ref().unwrap().project.name,
                "Updated fixture name"
            );
            assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
            s.activate_control(Control::TaskContinue, window, cx);
            finish_jobs(s, cx);
            s.navigate(Screen::Results, window, cx);
            finish_jobs(s, cx);
            assert!(!s.discovery_plan_current());
            assert!(!s.control_enabled(Control::CaptureBaseline, cx));
            assert!(!s.control_enabled(Control::Phase5(30), cx));
            s.create(&CreateProject, window, cx);
            assert!(
                s.native_prompt,
                "create blocked: task={:?}, busy={}, dialog={}, approval={}, preview={}",
                s.task.as_ref().map(|t| t.state),
                s.focus.busy,
                s.focus.dialog,
                s.focus.approval_dialog,
                s.preview.is_some()
            );
            inventory
        })
    });
    assert!(
        cx.pending_prompt()
            .unwrap()
            .1
            .contains("discovered page list")
    );
    cx.simulate_prompt_answer("Keep editing");
    cx.run_until_parked();
    shell.read_with(cx, |s, _| {
        assert_eq!(s.project.as_ref().unwrap().project.id, project);
        assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
    });
    owned.join_after_ui();
}
