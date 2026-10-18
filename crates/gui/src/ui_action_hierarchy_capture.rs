// SPDX-License-Identifier: MPL-2.0
//! Finite app-wide control-role inspection. Synthetic offline projects only.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

#[gpui::test]
#[ignore = "explicit two-size light/dark semantic-control inspection"]
fn renderer_capture_action_hierarchy(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("hierarchy-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), false);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for appearance in [AppAppearance::Light, AppAppearance::Dark] {
            regressee_storage::AppPreferences {
                appearance,
                reduced_motion: true,
            }
            .save(&AppPaths::isolated(dir.path()))
            .unwrap();
            for scene in [
                "projects",
                "project-settings",
                "app-settings",
                "history-runs",
                "history-baselines",
                "review",
                "single-image",
                "analysis",
                "page-options",
            ] {
                let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
                let owner = jobs.clone();
                owner.appearance.wait_loaded();
                let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                vcx.simulate_resize(size(px(width), px(height)));
                vcx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        load_review(s, project, cx);
                        s.notice = None;
                        s.error = None;
                        s.navigation_preference = Some(false);
                        s.configure_review_geometry(width);
                        match scene {
                            "projects" => s.focus.screen = Screen::Projects,
                            "project-settings" => {
                                s.focus.screen = Screen::Setup;
                                s.focus.setup_stage = SetupStage::Settings;
                            }
                            "app-settings" => {
                                s.focus.screen = Screen::Settings;
                                s.details.clear();
                                s.summary = Some(store.summary().unwrap());
                                s.runtime_readiness = Some(crate::app_settings_tests::readiness(
                                    regressee_app::product::RuntimeState::Ready,
                                    true,
                                ));
                            }
                            "history-runs" | "history-baselines" => {
                                s.focus.screen = Screen::History;
                                s.queue(Work::History { project, page: 0 }, cx);
                                finish_jobs(s, cx);
                                s.history_section = usize::from(scene == "history-baselines");
                            }
                            "single-image" => {
                                s.viewer_mode = 4;
                                s.reload_viewer(cx);
                                finish_jobs(s, cx);
                            }
                            "analysis" => s.activate_control(Control::Guided(44), window, cx),
                            "page-options" => {
                                let inventory =
                                    s.evidence.latest.as_ref().unwrap().run.inventory.clone();
                                let record = s.project.as_ref().unwrap();
                                s.selected_page = Some(inventory.pages[0].id);
                                s.preview = Some(DiscoveryPreview {
                                    project_revision: record.revision,
                                    settings: record.project.clone(),
                                    inventory,
                                    aliases: vec![],
                                    notes: vec![],
                                    visited: 2,
                                });
                                s.focus.preview = true;
                                s.focus.candidate = false;
                                s.focus.review_surface = ReviewSurface::Pages;
                                s.result_filters_open = true;
                            }
                            _ => (),
                        }
                        s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                        cx.notify();
                    })
                });
                vcx.run_until_parked();
                vcx.update(|window, cx| {
                    window.blur();
                    let _ = window.draw(cx);
                });
                let label = format!(
                    "hierarchy-{}-{width:.0}x{height:.0}-{scene}",
                    appearance.label().to_lowercase()
                );
                if matches!(scene, "history-runs" | "history-baselines") {
                    // Layout bounds alone missed the old opaque hover layer
                    // painting over every History label. Check actual ink in
                    // both the title and detail region, not just row geometry.
                    let control = if scene == "history-runs" {
                        "HistoryRun(0)"
                    } else {
                        "HistoryBaseline(0)"
                    };
                    let regions: Vec<_> = ["page-address", "page-details"]
                        .into_iter()
                        .map(|part| {
                            vcx.debug_bounds(format!("{part}-{control}").leak())
                                .unwrap()
                        })
                        .collect();
                    let frame = vcx.update(|window, _| window.capture_offscreen_frame().unwrap());
                    let fill = palette().selected;
                    let background = [(fill >> 16) as u8, (fill >> 8) as u8, fill as u8];
                    for bounds in regions {
                        let mut foreground = 0;
                        for y in (f32::from(bounds.top()).ceil() as u32)
                            ..(f32::from(bounds.bottom()).floor() as u32).min(frame.height)
                        {
                            for x in (f32::from(bounds.left()).ceil() as u32)
                                ..(f32::from(bounds.right()).floor() as u32).min(frame.width)
                            {
                                let pixel =
                                    &frame.rgba[((y * frame.width + x) * 4) as usize..][..3];
                                // Details deliberately use muted/selected ink;
                                // both differ strongly from either row fill.
                                if pixel
                                    .iter()
                                    .zip(background)
                                    .map(|(a, b)| a.abs_diff(b) as u16)
                                    .sum::<u16>()
                                    > 90
                                {
                                    foreground += 1;
                                }
                            }
                        }
                        assert!(
                            foreground > 8,
                            "{label}: History copy is obscured ({bounds:?})"
                        );
                    }
                }
                capture(vcx, &root, &label).unwrap();
                if matches!(scene, "review" | "single-image") {
                    let toolbar = vcx.debug_bounds("viewer-toolbar").unwrap();
                    for selector in [
                        "action-Guided(50)",
                        "action-Guided(54)",
                        "action-Phase5(36)",
                        "action-Guided(44)",
                    ] {
                        crate::phase2_evidence_tests::inside(
                            vcx.debug_bounds(selector).unwrap(),
                            toolbar,
                        );
                    }
                }
                manifest.push(label);
                shell.update(vcx, finish_jobs);
                owner.join_after_ui();
                vcx.update(|window, _| window.remove_window());
            }
        }
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );

    // Source modes and baseline inclusion exercise the second evidence kind.
    use crate::dom_workflow_tests::{Engine, finish_source, load, syntax_fixture};
    let fixture = syntax_fixture(CaptureMode::ScreenshotsAndDomSnapshots);
    let engine = Engine::start(&fixture);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for appearance in [AppAppearance::Light, AppAppearance::Dark] {
            regressee_storage::AppPreferences {
                appearance,
                reduced_motion: true,
            }
            .save(&AppPaths::isolated(fixture.root.path()))
            .unwrap();
            for scene in ["dom-diff", "dom-source", "dom-selected"] {
                let (jobs, events) = engine.jobs();
                let owner = jobs.clone();
                owner.appearance.wait_loaded();
                let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                vcx.simulate_resize(size(px(width), px(height)));
                vcx.update(|_, cx| {
                    shell.update(cx, |s, cx| {
                        load(s, &fixture, cx);
                        finish_source(s, cx);
                        s.configure_review_geometry(width);
                        if scene == "dom-source" {
                            s.dom.presentation = regressee_gui::jobs::DomPresentation::After;
                            s.reload_dom_viewer(cx);
                            finish_source(s, cx);
                        } else if scene == "dom-selected" {
                            s.kind_approval_action(23, cx);
                            assert!(
                                s.dom
                                    .selected
                                    .contains_key(s.selected_key.as_ref().unwrap())
                            );
                            assert!(!s.focus.approval_dialog, "selection is not saving");
                        }
                        let order = s.visible_controls();
                        let position = |c| order.iter().position(|item| *item == c).unwrap();
                        assert!(position(Control::Guided(50)) < position(Control::DomSource(0)));
                        assert!(position(Control::DomSource(2)) < position(Control::CrawlField(9)));
                        cx.notify();
                    })
                });
                vcx.run_until_parked();
                vcx.update(|window, cx| {
                    window.blur();
                    let _ = window.draw(cx);
                });
                let toolbar = vcx.debug_bounds("viewer-toolbar").unwrap();
                for selector in [
                    "action-DomSource(0)",
                    "action-DomSource(1)",
                    "action-DomSource(2)",
                ] {
                    crate::phase2_evidence_tests::inside(
                        vcx.debug_bounds(selector).unwrap(),
                        toolbar,
                    );
                }
                assert!(vcx.debug_bounds("dom-diff-source").unwrap().size.height >= px(48.));
                let label = format!(
                    "hierarchy-{}-{width:.0}x{height:.0}-{scene}",
                    appearance.label().to_lowercase()
                );
                capture(vcx, &root, &label).unwrap();
                manifest.push(label);
                owner.join_after_ui();
                vcx.update(|window, _| window.remove_window());
            }
        }
    }
    set_dark(false);
    fs::write(
        root.join("action-hierarchy-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
