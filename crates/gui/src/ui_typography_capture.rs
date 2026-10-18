// SPDX-License-Identifier: MPL-2.0
//! Bounded native typography inspection. All evidence is isolated and synthetic.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

#[gpui::test]
#[ignore = "explicit three-size light/dark typography inspection"]
fn renderer_capture_typography(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("typography-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), false);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = vec![];
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        for preference in [AppAppearance::Light, AppAppearance::Dark] {
            regressee_storage::AppPreferences {
                appearance: preference,
                ..Default::default()
            }
            .save(&AppPaths::isolated(dir.path()))
            .unwrap();
            for scene in [
                "review",
                "project-settings",
                "history",
                "app-settings",
                "note",
                "about",
                "empty",
            ] {
                let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
                let owned = jobs.clone();
                owned.appearance.wait_loaded();
                let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                cx.simulate_resize(size(px(width), px(height)));
                cx.update(|w, cx| shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.error = None; s.notice = None;
                    s.navigation_preference = Some(false);
                    s.configure_review_geometry(width);
                    // Reproduce the reported rail wrap without altering stored evidence.
                    let baseline = s.evidence.baseline.as_mut().unwrap();
                    baseline.references.resize(25, baseline.references[0].clone());
                    s.runtime_readiness = Some(crate::app_settings_tests::readiness(
                        regressee_app::product::RuntimeState::Ready, true));
                    s.summary = Some(store.summary().unwrap());
                    match scene {
                        "project-settings" => { s.focus.screen = Screen::Setup; s.focus.setup_stage = SetupStage::Settings; },
                        "history" => { s.focus.screen = Screen::History; s.queue(Work::History { project, page: 0 }, cx); finish_jobs(s, cx); },
                        "app-settings" | "about" => { s.focus.screen = Screen::Settings; s.details.clear();
                            if scene == "about" { s.activate_control(Control::About, w, cx); } },
                        "note" => { s.activate_control(Control::Phase5(37), w, cx);
                            s.analysis_fields[9].update(cx, |f, cx| f.set_value(
                                "Keep related lines together.\nA longer review note should remain comfortably readable as it wraps, with accented text: Æ Ø Å, gjpqy. Saving remains explicit.", cx)); },
                        "empty" => { s.focus.screen = Screen::Projects; s.projects.items.clear(); },
                        _ => (),
                    }
                    s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                    cx.notify();
                }));
                cx.run_until_parked();
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                let label = format!(
                    "typography-{}-{width:.0}x{height:.0}-{scene}",
                    preference.label().to_lowercase()
                );
                capture(cx, &root, &label).unwrap();
                let mut bounds = |selector: &'static str| {
                    cx.debug_bounds(selector).map(|b| {
                        [
                            f32::from(b.left()),
                            f32::from(b.top()),
                            f32::from(b.size.width),
                            f32::from(b.size.height),
                        ]
                    })
                };
                manifest.push(serde_json::json!({"label":label,"scene":scene,"width":width,"height":height,
                    "rail_status":bounds("rail-baseline-status"),"heading":bounds("screen-heading-title"),
                    "subtitle":bounds("screen-heading-subtitle")}));
                shell.update(cx, finish_jobs);
                owned.join_after_ui();
                cx.update(|w, _| w.remove_window());
            }
        }
    }
    set_dark(false);
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("typography-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
