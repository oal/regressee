// SPDX-License-Identifier: MPL-2.0
//! Simulated GPUI geometry/focus tests, not native OS input qualification.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::TestAppContext;

fn inside(inner: Bounds<Pixels>, outer: Bounds<Pixels>) {
    assert!(
        inner.left() >= outer.left() - px(1.)
            && inner.right() <= outer.right() + px(1.)
            && inner.top() >= outer.top() - px(1.)
            && inner.bottom() <= outer.bottom() + px(1.),
        "{inner:?} outside {outer:?}"
    );
}

#[gpui::test]
fn discovery_selection_reveals_actions_and_navigation_inspects_pages(cx: &mut TestAppContext) {
    discovery_selection(cx, false);
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "bounded discovery selection offscreen capture with synthetic isolated data"]
fn renderer_capture_discovery_selection(cx: &mut TestAppContext) {
    discovery_selection(cx, true);
}

fn discovery_selection(cx: &mut TestAppContext, render: bool) {
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
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let saved_project = store.load_project(project).unwrap();
    let saved_evidence = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        options_ux::fixture_discovery(s);
        let blocked = &mut s.preview.as_mut().unwrap().inventory.pages[1];
        blocked.canonical = format!(
            "https://other.test/login?return=https%3A%2F%2Fexample.test%2F{}",
            "long-path%2F".repeat(8)
        )
        .parse()
        .unwrap();
        blocked.included = false;
        blocked.reason =
            Some("Website not approved — add and confirm it as a starting address".into());
        cx.notify();
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        let (first, blocked, last) = shell.update(cx, |s, cx| {
            s.configure_review_geometry(width);
            s.result_filters_open = false;
            s.discovery_details_open = false;
            s.notice = None;
            s.error = None;
            s.search.clear();
            let pages = &s.preview.as_ref().unwrap().inventory.pages;
            let ids = (pages[0].id, pages[1].id, pages.last().unwrap().id);
            s.selected_page = Some(ids.0);
            s.preview_list_scroll
                .scroll_to_item(0, gpui::ScrollStrategy::Top);
            cx.notify();
            ids
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert!(cx.debug_bounds("discovery-selection-help").is_some());
        shell.read_with(cx, |s, _| assert!(!s.options_panel_open()));
        // Clicking the real mounted row reveals its actions without a second
        // trip through the Page options trigger.
        let row = cx
            .debug_bounds(format!("page-row-candidate-{first}").leak())
            .unwrap();
        cx.simulate_click(row.center(), gpui::Modifiers::none());
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        shell.read_with(cx, |s, cx| {
            assert_eq!(s.selected_page, Some(first));
            assert!(s.options_panel_open());
            assert!(s.control_enabled(Control::Guided(22), cx));
        });
        inside(
            cx.debug_bounds("action-Guided(22)").unwrap(),
            cx.debug_bounds("page-options-panel-scroll").unwrap(),
        );
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if let Some(root) = &capture_root {
            crate::ui_renderer_capture::capture(
                cx,
                root,
                &format!("{width:.0}x{height:.0}-selected-included"),
            )
            .unwrap();
        }
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Guided(22), window, cx);
                assert!(!s.preview.as_ref().unwrap().inventory.pages[0].included);
                s.activate_control(Control::Guided(22), window, cx);
                assert!(s.preview.as_ref().unwrap().inventory.pages[0].included);
                s.activate_control(Control::CloseOptions, window, cx);
                assert!(!s.options_panel_open());
                s.activate_control(Control::Guided(24), window, cx);
                assert_eq!(s.selected_page, Some(blocked));
                assert!(
                    s.options_panel_open(),
                    "Next page reveals details, not just selection paint"
                );
                assert!(!s.control_enabled(Control::Guided(22), cx));
                s.activate_control(Control::Guided(22), window, cx);
                assert!(!s.preview.as_ref().unwrap().inventory.pages[1].included);
            });
            let _ = window.draw(cx);
        });
        // Font metrics and very long addresses can require scrolling the
        // bounded details body; recovery must remain reachable in that body.
        let recovery = cx.debug_bounds("discovery-inclusion-blocked-help").unwrap();
        let viewport = cx.debug_bounds("page-options-panel-scroll").unwrap();
        if recovery.bottom() > viewport.bottom() {
            shell.update(cx, |s, cx| {
                let offset = s.options_scroll.offset();
                s.options_scroll.set_offset(gpui::point(
                    offset.x,
                    offset.y + viewport.bottom() - recovery.bottom(),
                ));
                cx.notify();
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
        }
        inside(
            cx.debug_bounds("discovery-inclusion-blocked-help").unwrap(),
            viewport,
        );
        let address = cx.debug_bounds("discovery-selected-address").unwrap();
        assert!(address.left() >= viewport.left() && address.right() <= viewport.right());
        #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
        if let Some(root) = &capture_root {
            crate::ui_renderer_capture::capture(
                cx,
                root,
                &format!("{width:.0}x{height:.0}-selected-blocked"),
            )
            .unwrap();
        }
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.options_scroll
                    .set_offset(gpui::point(px(0.), -s.options_scroll.max_offset().height));
                s.activate_control(Control::Guided(23), window, cx);
                assert_eq!(s.selected_page, Some(first));
                assert_eq!(s.options_scroll.offset(), gpui::point(px(0.), px(0.)));
                // Navigation must reveal selections beyond the initial viewport.
                s.selected_page = Some(last);
                s.activate_control(Control::Guided(23), window, cx);
            });
            let _ = window.draw(cx);
        });
        let selected = shell.read_with(cx, |s, _| s.selected_page.unwrap());
        inside(
            cx.debug_bounds(format!("page-row-candidate-{selected}").leak())
                .unwrap(),
            cx.debug_bounds("inventory-preview").unwrap(),
        );
        // A stale click hidden by search cannot reopen the panel or steal selection.
        shell.update(cx, |s, cx| {
            s.result_filters_open = false;
            s.search = "no-matches".into();
            s.select_preview_page(first);
            assert_eq!(s.selected_page, Some(selected));
            assert!(!s.options_panel_open());
            s.search.clear();
            cx.notify();
        });
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        store.load_project(project).unwrap().project,
        saved_project.project
    );
    assert_eq!(
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
        saved_evidence
    );
}

#[gpui::test]
fn discovery_options_keep_the_list_and_close_reachable_with_long_notes(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        options_ux::fixture_discovery(s);
        cx.notify();
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.result_filters_open = false;
                s.discovery_details_open = false;
                s.configure_review_geometry(width);
                assert!(!s.control_enabled(Control::DiscoveryDetails, cx));
                s.activate_control(Control::Guided(45), window, cx);
                assert!(s.result_filters_open);
                assert!(s.handle(Control::CloseOptions, cx).is_focused(window));
                s.activate_control(Control::DiscoveryDetails, window, cx);
                s.preview.as_mut().unwrap().notes.extend((0..24).map(|i| {
                    format!(
                        "Synthetic diagnostic note {i}. {}",
                        "Additional discovery detail. ".repeat(12)
                    )
                }));
                cx.notify();
            });
            let _ = window.draw(cx);
        });
        let workspace = cx.debug_bounds("queue-beside-viewer").unwrap();
        let panel = cx.debug_bounds("page-options-panel").unwrap();
        let list = cx.debug_bounds("inventory-preview").unwrap();
        let close = cx.debug_bounds("action-CloseOptions").unwrap();
        let dismiss = cx.debug_bounds("action-DiscardPageList").unwrap();
        inside(panel, workspace);
        inside(list, workspace);
        inside(close, panel);
        inside(dismiss, panel);
        assert!(list.size.height >= px(128.) && list.size.width >= px(198.));
        shell.update(cx, |s, cx| {
            assert!(s.options_scroll.max_offset().height > px(100.));
            s.options_scroll
                .set_offset(gpui::point(px(0.), -s.options_scroll.max_offset().height));
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert_eq!(cx.debug_bounds("action-CloseOptions").unwrap(), close);
        assert_eq!(cx.debug_bounds("action-DiscardPageList").unwrap(), dismiss);
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.reveal_review_control(Control::Guided(22), window, cx);
                window.focus(&s.handle(Control::Guided(22), cx));
                cx.notify();
            });
            let _ = window.draw(cx);
        });
        inside(
            cx.debug_bounds("action-Guided(22)").unwrap(),
            cx.debug_bounds("page-options-panel-scroll").unwrap(),
        );
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                let inventory = s.preview.as_ref().unwrap().inventory.clone();
                s.dismiss(&Dismiss, window, cx);
                assert!(
                    !s.result_filters_open && s.handle(Control::Guided(45), cx).is_focused(window)
                );
                assert_eq!(s.preview.as_ref().unwrap().inventory, inventory);
            });
        });
    }
    owned.join_after_ui();
}

#[gpui::test]
fn result_filter_panel_preserves_evidence_and_drafts_and_reveals_keyboard_choices(
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
        s.mask_dirty = true;
        s.analysis_fields[9].update(cx, |f, cx| f.set_value("Keep this note", cx));
        cx.notify();
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.configure_review_geometry(width);
                s.result_filters_open = false;
                s.activate_control(Control::Guided(45), window, cx);
                assert!(!s.control_enabled(Control::Phase5(36), cx));
                assert!(!s.control_enabled(Control::Viewer, cx));
            });
            let _ = window.draw(cx);
        });
        inside(
            cx.debug_bounds("action-CloseOptions").unwrap(),
            cx.debug_bounds("page-options-panel").unwrap(),
        );
        let (key, identity, requests) = shell.read_with(cx, |s, _| {
            (
                s.selected_key.clone(),
                s.viewer.as_ref().unwrap().identity.clone(),
                s.next_request,
            )
        });
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.reveal_review_control(Control::Guided(19), window, cx);
                window.focus(&s.handle(Control::Guided(19), cx));
                cx.notify();
            });
            let _ = window.draw(cx);
        });
        inside(
            cx.debug_bounds("check-Guided(19)").unwrap(),
            cx.debug_bounds("page-options-panel-scroll").unwrap(),
        );
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Choice(Choice::MissingResults), window, cx);
                assert_eq!(s.result_filter, 2);
                assert_eq!(s.selected_key, key);
                assert_eq!(s.viewer.as_ref().unwrap().identity, identity);
                assert_eq!(s.next_request, requests);
                assert!(s.mask_dirty);
                assert_eq!(
                    s.analysis_fields[9].read(cx).content.as_ref(),
                    "Keep this note"
                );
                s.activate_control(Control::CloseOptions, window, cx);
                assert!(s.handle(Control::Guided(45), cx).is_focused(window));
            });
        });
    }
    owned.join_after_ui();
}

#[gpui::test]
fn mapping_preview_has_bounded_rows_and_a_pinned_close_without_mutations(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        load_review(s, project, cx);
        s.focus.screen = Screen::Setup;
        s.focus.setup_stage = SetupStage::Access;
        s.setup_environment_open = true;
        s.show_mapping = true;
        s.environment.as_mut().unwrap().mapped = (0..40)
            .map(|i| {
                (
                    format!("https://example.test/pages/{i}/"),
                    format!("https://staging.example.test/pages/{i}/"),
                )
            })
            .collect();
        cx.notify();
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            s.show_mapping = true;
            s.mapping_preview_scroll
                .set_offset(gpui::point(px(0.), px(0.)));
            s.environment.as_mut().unwrap().mapped = (0..40)
                .map(|i| {
                    (
                        format!("https://example.test/pages/{i}/"),
                        format!("https://staging.example.test/pages/{i}/"),
                    )
                })
                .collect();
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let panel = cx.debug_bounds("mapping-preview").unwrap();
        let close = cx.debug_bounds("action-CloseMappingPreview").unwrap();
        let title = cx.debug_bounds("mapping-preview-title").unwrap();
        assert!(title.size.width >= px(240.) && title.size.height <= px(24.));
        inside(title, panel);
        let body = cx.debug_bounds("mapping-preview-scroll").unwrap();
        assert!(body.size.height >= px(150.));
        inside(cx.debug_bounds("mapping-preview-row-0").unwrap(), body);
        inside(close, panel);
        assert!(panel.size.height <= px(280.));
        assert!(cx.debug_bounds("mapping-preview-row-15").is_some());
        assert!(cx.debug_bounds("mapping-preview-row-16").is_none());
        shell.update(cx, |s, cx| {
            assert!(s.mapping_preview_scroll.max_offset().height > px(100.));
            s.mapping_preview_scroll.set_offset(gpui::point(
                px(0.),
                -s.mapping_preview_scroll.max_offset().height,
            ));
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert_eq!(
            cx.debug_bounds("action-CloseMappingPreview").unwrap(),
            close
        );
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                window.focus(&s.handle(Control::Guided(36), cx));
                s.move_focus(false, window, cx);
                assert!(
                    s.handle(Control::CloseMappingPreview, cx)
                        .is_focused(window)
                );
            });
            let _ = window.draw(cx);
        });
        let screen = shell.read_with(cx, |s, _| s.screen_scroll.bounds());
        inside(
            cx.debug_bounds("action-CloseMappingPreview").unwrap(),
            screen,
        );
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.move_focus(true, window, cx);
                assert!(s.handle(Control::Guided(36), cx).is_focused(window));
            });
            let _ = window.draw(cx);
        });
        inside(cx.debug_bounds("action-Guided(36)").unwrap(), screen);
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                let request = s.next_request;
                let saved = s.project.clone().unwrap();
                s.dismiss(&Dismiss, window, cx);
                assert!(!s.show_mapping && s.handle(Control::Guided(35), cx).is_focused(window));
                assert_eq!(s.next_request, request);
                assert_eq!(s.project.as_ref().unwrap().project, saved.project);
                finish_jobs(s, cx);
            });
            let _ = window.draw(cx);
        });
        inside(cx.debug_bounds("action-Guided(35)").unwrap(), screen);
        shell.update(cx, |s, cx| {
            s.show_mapping = true;
            s.mapping_preview_scroll
                .set_offset(gpui::point(px(0.), px(0.)));
            s.environment.as_mut().unwrap().mapped = vec![(
                format!("https://example.test/{}/", "long-source-path/".repeat(24)),
                format!(
                    "https://staging.example.test/{}/",
                    "long-destination-path/".repeat(24)
                ),
            )];
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let row = cx.debug_bounds("mapping-preview-row-0").unwrap();
        let body = cx.debug_bounds("mapping-preview-scroll").unwrap();
        assert!(row.left() >= body.left() && row.right() <= body.right());
        shell.update(cx, |s, cx| {
            s.environment.as_mut().unwrap().mapped.clear();
            s.mapping_preview_scroll
                .set_offset(gpui::point(px(0.), px(0.)));
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        inside(
            cx.debug_bounds("mapping-preview-empty").unwrap(),
            cx.debug_bounds("mapping-preview-scroll").unwrap(),
        );
    }
    owned.join_after_ui();
}

#[gpui::test]
fn mapped_url_status_refresh_preserves_dirty_and_newer_environment_drafts(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let saved = serde_json::to_value(store.load_project(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Access;
            s.activate_control(Control::EnvironmentOptions, window, cx);
            let mut staging = s.draft_settings.environments[0].clone();
            staging.id = regressee_core::domain::EnvironmentId::new();
            staging.name = "Unsaved staging".into();
            s.environment_draft = Some(staging.id);
            s.draft_settings.environments.push(staging);
            s.login_ephemeral = !s.login_ephemeral;
            for i in 10..=12 {
                s.crawl_fields[i].update(cx, |f, cx| {
                    f.set_value(&format!("https://draft.example.test/{i}/"), cx)
                });
            }
            assert!(s.environment_unsaved(cx));
            let draft = s.environment_draft_signature(cx);
            let settings = s.draft_settings.clone();
            assert!(s.control_enabled(Control::Guided(35), cx));
            s.activate_control(Control::Guided(35), window, cx);
            assert!(s.pending.is_some());
            finish_jobs(s, cx);
            assert!(s.environment_draft_signature(cx) == draft);
            assert_eq!(s.draft_settings, settings);
            assert!(s.show_mapping && s.environment_unsaved(cx));
            // Initial hydration must also refuse edits arriving after the read started.
            s.environment = None;
            s.queue(Work::EnvironmentStatus(project), cx);
            s.crawl_fields[10].update(cx, |f, cx| {
                f.set_value("https://newer.example.test/login/", cx)
            });
            let newer = s.environment_draft_signature(cx);
            finish_jobs(s, cx);
            assert!(s.environment_draft_signature(cx) == newer);
            assert!(s.environment.is_some());
        })
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        serde_json::to_value(store.load_project(project).unwrap()).unwrap(),
        saved
    );
}
