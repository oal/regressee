// SPDX-License-Identifier: MPL-2.0
//! Bounded app-wide action/icon and context-grouping inspection, synthetic only.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

#[gpui::test]
#[ignore = "explicit three-size app-wide icon and spacing inspection"]
fn renderer_capture_icon_spacing(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("icon-spacing-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), false);
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = vec![];
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        for scene in [
            "review",
            "navigation-hidden",
            "long-context",
            "selected-baseline",
            "note-draft",
            "loading",
            "view-menu",
            "capture-only",
            "projects",
            "project-settings",
            "history",
            "app-settings",
            "network",
            "settings-tail",
        ] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|w,cx| shell.update(cx,|s,cx| {
                load_review(s,project,cx);
                s.error=None;s.notice=None;
                s.navigation_preference=Some(scene=="navigation-hidden");
                s.configure_review_geometry(width);
                s.screen_scroll.set_offset(gpui::point(px(0.),px(0.)));
                match scene {
                    "long-context" => {
                        s.evidence.latest.as_mut().unwrap().run.targets[0].requested="https://northline.example/account/orders/customer-service/release-notes/?sort=recent&locale=nb-NO#delivery".parse().unwrap();
                        s.evidence.latest.as_mut().unwrap().run.environment.name="Historical check — staging before the environment was renamed".into();
                        s.viewer.as_mut().unwrap().before_environment=Some("Historical reference — production before the environment was renamed".into());
                    },
                    "selected-baseline" => s.activate_control(Control::Phase5(23),w,cx),
                    "note-draft" => {
                        s.activate_control(Control::Phase5(37),w,cx);
                        s.analysis_fields[9].update(cx,|f,cx|f.set_value("A local review note; saving remains explicit.",cx));
                    },
                    "loading" => s.activate_control(Control::Guided(24),w,cx),
                    "view-menu" => s.activate_control(Control::Guided(54),w,cx),
                    "capture-only" => ui_review_matrix::capture_only(s,&store,"ready"),
                    "projects" => s.focus.screen=Screen::Projects,
                    "project-settings" => {
                        s.focus.screen=Screen::Setup;s.focus.setup_stage=SetupStage::Settings;
                        s.runtime_readiness=Some(crate::app_settings_tests::readiness(regressee_app::product::RuntimeState::Ready,true));
                    },
                    "history" => {s.focus.screen=Screen::History;s.queue(Work::History {project,page:0},cx);finish_jobs(s,cx);},
                    "app-settings" | "network" | "settings-tail" => {
                        s.focus.screen=Screen::Settings;s.details.clear();
                        s.runtime_readiness=Some(crate::app_settings_tests::readiness(regressee_app::product::RuntimeState::Ready,true));
                        s.runtime_status=Some("Synthetic verified browser status.".into());
                        s.network_status=Some("Synthetic saved network settings.".into());
                        s.summary=Some(store.summary().unwrap());
                        if scene=="network" {s.details.insert(Detail::ManagedNetwork);}
                        if scene=="settings-tail" {s.details.insert(Detail::SupportTools);}
                    },
                    _=>(),
                }
                cx.notify();
            }));
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            if matches!(scene, "network" | "settings-tail") {
                shell.update(cx, |s, cx| {
                    s.screen_scroll
                        .set_offset(gpui::point(px(0.), -s.screen_scroll.max_offset().height));
                    cx.notify();
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
            }
            shell.read_with(cx, |s, cx| match scene {
                "loading" => assert!(s.viewer_pending.is_some() && s.inspection_chrome_mounted()),
                "selected-baseline" => assert!(
                    !s.approval_selection.is_empty() && s.control_enabled(Control::Phase5(24), cx)
                ),
                "view-menu" => assert!(s.viewer_mode_open),
                "note-draft" => assert!(s.review_note_open && s.review_note_dirty(cx)),
                _ => (),
            });
            let label = format!("icon-spacing-{width:.0}x{height:.0}-{scene}");
            capture(cx, &root, &label).unwrap();
            manifest.push(label);
            shell.update(cx, |s, cx| finish_jobs(s, cx));
            owned.join_after_ui();
        }
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("icon-spacing-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
