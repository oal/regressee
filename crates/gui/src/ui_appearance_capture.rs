// SPDX-License-Identifier: MPL-2.0
//! One bounded light/dark inspection round using native offscreen pixels.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

#[gpui::test]
#[ignore = "explicit three-size native light/dark appearance inspection"]
fn renderer_capture_appearance(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("appearance-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), false);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = vec![];
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        for preference in [AppAppearance::Light, AppAppearance::Dark] {
            for scene in [
                "review",
                "settings",
                "project-settings",
                "history",
                "menu",
                "about",
                "warning",
                "missing",
            ] {
                regressee_storage::AppPreferences {
                    appearance: preference,
                }
                .save(&AppPaths::isolated(dir.path()))
                .unwrap();
                let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
                let owned = jobs.clone();
                owned.appearance.wait_loaded();
                let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                cx.simulate_resize(size(px(width), px(height)));
                cx.update(|window, cx| shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.error = None; s.notice = None;
                    s.navigation_preference = Some(false);
                    s.configure_review_geometry(width);
                    s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                    match scene {
                        "settings" | "warning" | "about" => {
                            s.focus.screen = Screen::Settings;
                            s.details.clear();
                            s.runtime_readiness = Some(crate::app_settings_tests::readiness(
                                if scene == "warning" { regressee_app::product::RuntimeState::RepairRequired }
                                else { regressee_app::product::RuntimeState::Ready }, true));
                            s.runtime_status = Some("Synthetic verified browser status.".into());
                            s.network_status = Some("Synthetic saved network settings.".into());
                            s.summary = Some(store.summary().unwrap());
                            if scene == "about" { s.activate_control(Control::About, window, cx); }
                        },
                        "project-settings" => { s.focus.screen = Screen::Setup; s.focus.setup_stage = SetupStage::Settings; },
                        "history" => { s.focus.screen = Screen::History; s.queue(Work::History { project, page: 0 }, cx); finish_jobs(s, cx); },
                        "menu" => s.activate_control(Control::Guided(54), window, cx),
                        "missing" => { s.viewer = None; s.viewer_read_failed = true; s.error = Some("Synthetic screenshot read failure. The saved original was not changed.".into()); },
                        _ => (),
                    }
                    cx.notify();
                }));
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                // Embedded PNGs decode asynchronously. Match the brand capture
                // harness: settle first use and assert the correct variant is
                // ready, rather than accepting an empty logo slot as evidence.
                cx.run_until_parked();
                cx.update(|window, cx| {
                    assert!(
                        brand::logo(Background::workspace())
                            .get_render_image(window, cx)
                            .is_some()
                    );
                    if scene == "about" {
                        assert!(
                            brand::logo(Background::workspace())
                                .get_render_image(window, cx)
                                .is_some()
                        );
                    }
                    let _ = window.draw(cx);
                });
                if scene == "settings" {
                    cx.update(|window, cx| {
                        shell.update(cx, |s, cx| {
                            let control = Control::Appearance(preference);
                            s.reveal_form_control(control, true, window, cx);
                            window.focus(&s.handle(control, cx));
                            cx.notify();
                        })
                    });
                }
                let label = format!(
                    "appearance-{}-{width:.0}x{height:.0}-{scene}",
                    preference.label().to_lowercase()
                );
                capture(cx, &root, &label).unwrap();
                let pane = cx.debug_bounds("phase5-before");
                manifest.push(serde_json::json!({"label":label,"width":width,"height":height,"scene":scene,"appearance":preference,"pane":pane.map(|b| [f32::from(b.left()),f32::from(b.top()),f32::from(b.size.width),f32::from(b.size.height)])}));
                shell.update(cx, finish_jobs);
                owned.join_after_ui();
                cx.update(|window, _| window.remove_window());
            }
        }
    }
    set_dark(false);
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("appearance-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
