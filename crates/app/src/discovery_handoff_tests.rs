// SPDX-License-Identifier: MPL-2.0
//! Discovery/evidence handoff regressions. Browser replies are synthetic; saved
//! evidence reads use isolated real storage. No website is visited by these tests.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::TestAppContext;
use regressee_core::domain::{
    DiscoveryCoverage, InventoryId, InventoryPage, InventoryRevision, PageId,
};

fn found_pages(s: &Shell) -> DiscoveryPreview {
    let project = s.project.as_ref().unwrap();
    DiscoveryPreview {
        project_revision: project.revision,
        settings: project.project.clone(),
        inventory: InventoryRevision {
            id: InventoryId::new(),
            project: project.project.id,
            identity_version: 2,
            pages: ["/", "/contact"]
                .into_iter()
                .map(|path| InventoryPage {
                    id: PageId::new(),
                    canonical: project.project.sources[0].join(path).unwrap(),
                    included: true,
                    reason: None,
                })
                .collect(),
            coverage: DiscoveryCoverage::Complete,
        },
        aliases: vec![],
        notes: vec![],
        visited: 2,
    }
}

fn receive_discovery(s: &mut Shell, preview: DiscoveryPreview, cx: &mut Context<Shell>) {
    assert!(s.pending.is_none());
    // Exercise the real terminal-reply/task-continuation path without starting
    // a browser or substituting the discovery list directly into UI state.
    s.next_request += 1;
    s.pending = Some(s.next_request);
    s.focus.busy = true;
    s.focus.capturing = true;
    s.begin_task(
        regressee_app::task::Spec::for_work(&Work::Discover(preview.settings.id)).unwrap(),
    );
    s.apply_update(
        Update {
            id: s.next_request,
            reply: Reply::Discovery(Box::new(preview)),
        },
        cx,
    );
}

#[gpui::test]
fn discovery_handoff_keeps_new_project_pages_through_initial_read_and_refresh(
    cx: &mut TestAppContext,
) {
    new_project_handoff(cx, false);
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "bounded discovery handoff/empty-state offscreen capture in an explicit private output directory"]
fn renderer_capture_discovery_handoff(cx: &mut TestAppContext) {
    new_project_handoff(cx, true);
}

fn new_project_handoff(cx: &mut TestAppContext, render: bool) {
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    let capture_root = render.then(|| {
        let root = crate::ui_renderer_capture::output_root().unwrap();
        cx.enable_offscreen_renderer().unwrap();
        root
    });
    #[cfg(not(all(feature = "ui-renderer-capture", target_os = "linux")))]
    assert!(
        !render,
        "offscreen capture requires the Linux renderer feature"
    );
    let dir = tempfile::tempdir().unwrap();
    let paths = AppPaths::isolated(dir.path());
    let mut store = regressee_storage::Store::open(paths.clone()).unwrap();
    let project = Project::draft("New discovery project".into(), "https://example.test").unwrap();
    store.save_project(&project, None).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(paths.clone());
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Pages;
            s.queue(Work::Load(project.id), cx);
            finish_jobs(s, cx);
            assert!(s.evidence_dirty, "setup has not read saved evidence yet");
            let preview = found_pages(s);
            let inventory = preview.inventory.clone();
            receive_discovery(s, preview, cx);
            assert!(s.task.is_some());
            assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
            s.activate_control(Control::TaskContinue, window, cx);
            assert!(s.pending.is_some(), "continuation reads saved evidence");
            finish_jobs(s, cx);
            assert_eq!(
                s.preview.as_ref().map(|p| &p.inventory),
                Some(&inventory),
                "an empty saved-evidence read must not erase discovered pages"
            );
            assert!(s.focus.preview);
            assert!(!s.evidence_dirty);
            assert!(s.evidence.latest.is_none());
            assert_eq!(s.focus.review_surface, ReviewSurface::Pages);
            assert!(s.control_enabled(Control::CaptureBaseline, cx));
            assert!(!s.control_enabled(Control::SaveBaseline, cx));
            s.activate_control(Control::RefreshEvidence, window, cx);
            finish_jobs(s, cx);
            assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
            assert_eq!(s.preview_page_ids().len(), 2);
            assert_eq!(s.selected_page, Some(inventory.pages[0].id));
        });
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let list = cx.debug_bounds("inventory-preview").unwrap();
        assert!(list.size.width >= px(500.));
        assert!(list.size.height >= px(PAGE_ROW_HEIGHT * 2.));
        assert!(cx.debug_bounds("discovery-capture-plan").is_some());
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if let Some(root) = &capture_root {
            crate::ui_renderer_capture::capture(
                cx,
                root,
                &format!("{width:.0}x{height:.0}-discovery-handoff"),
            )
            .unwrap();
        }
    }
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            s.select_preview_page(s.selected_page.unwrap());
            s.activate_control(Control::DiscardPageList, window, cx);
            assert!(s.native_prompt);
            assert!(s.preview.is_some(), "requesting discard does not discard");
        });
    });
    cx.simulate_prompt_answer("Discard pages");
    cx.run_until_parked();
    shell.update(cx, |s, cx| {
        assert!(s.preview.is_none());
        assert!(!s.focus.preview);
        assert_eq!(s.focus.review_surface, ReviewSurface::Pages);
        assert!(s.control_enabled(Control::CaptureBaseline, cx));
        s.notice = None;
        cx.notify();
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let empty = cx.debug_bounds("pages-empty-state").unwrap();
        assert!(empty.size.width >= px(500.));
        assert!(empty.size.height >= px(80.));
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if let Some(root) = &capture_root {
            crate::ui_renderer_capture::capture(
                cx,
                root,
                &format!("{width:.0}x{height:.0}-pages-empty-state"),
            )
            .unwrap();
        }
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(paths).unwrap();
    assert!(store.project_evidence(project.id).unwrap().latest.is_none());
    assert!(
        store
            .load_project(project.id)
            .unwrap()
            .project
            .active_baseline
            .is_none()
    );
}

#[gpui::test]
fn discovery_handoff_keeps_selection_until_its_own_inventory_is_captured(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            let baseline = s.project.as_ref().unwrap().project.active_baseline;
            let preview = found_pages(s);
            let inventory = preview.inventory.id;
            receive_discovery(s, preview, cx);
            s.activate_control(Control::TaskContinue, window, cx);
            // User exclusions and selection must survive local reads too.
            s.select_preview_page(s.preview.as_ref().unwrap().inventory.pages[1].id);
            s.guided(22, cx);
            let selected = s.selected_page;
            let edited = s.preview.as_ref().unwrap().inventory.clone();
            s.activate_control(Control::RefreshEvidence, window, cx);
            finish_jobs(s, cx);
            assert_eq!(s.preview.as_ref().map(|p| &p.inventory), Some(&edited));
            assert_eq!(s.selected_page, selected);
            assert_eq!(s.focus.review_surface, ReviewSurface::Pages);
            // Loading an older unsaved capture must not redirect discovery to
            // screenshot inspection or steal focus for its baseline-save action.
            let mut evidence = s.evidence.clone();
            evidence.latest.as_mut().unwrap().run.intent = RunIntent::NewBaseline;
            s.selected_key = None;
            s.next_request += 1;
            s.pending = Some(s.next_request);
            s.apply_update(
                Update {
                    id: s.next_request,
                    reply: Reply::Evidence(Box::new(evidence)),
                },
                cx,
            );
            finish_jobs(s, cx);
            assert!(s.focus.candidate);
            assert_eq!(s.focus.review_surface, ReviewSurface::Pages);
            assert_ne!(s.presentation_focus, Some(Control::SaveBaseline));
            assert_eq!(s.preview.as_ref().map(|p| &p.inventory), Some(&edited));
            // A representative screenshot never consumes the capture list, even
            // if a reply happens to carry that inventory identity.
            let mut evidence = s.evidence.clone();
            evidence.latest.as_mut().unwrap().run.inventory.id = inventory;
            evidence.latest.as_mut().unwrap().run.intent = RunIntent::RepresentativePreview;
            s.next_request += 1;
            s.pending = Some(s.next_request);
            s.apply_update(
                Update {
                    id: s.next_request,
                    reply: Reply::Evidence(Box::new(evidence.clone())),
                },
                cx,
            );
            finish_jobs(s, cx);
            assert_eq!(s.preview.as_ref().map(|p| &p.inventory), Some(&edited));
            // A full capture from the reviewed inventory does consume it.
            evidence.latest.as_mut().unwrap().run.intent = RunIntent::NewBaseline;
            s.next_request += 1;
            s.pending = Some(s.next_request);
            s.apply_update(
                Update {
                    id: s.next_request,
                    reply: Reply::Evidence(Box::new(evidence)),
                },
                cx,
            );
            finish_jobs(s, cx);
            assert!(s.preview.is_none());
            assert!(!s.focus.preview);
            assert_eq!(
                s.project.as_ref().unwrap().project.active_baseline,
                baseline
            );
        });
    });
    owned.join_after_ui();
}
