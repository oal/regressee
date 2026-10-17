// SPDX-License-Identifier: MPL-2.0
//! Bounded copy inspection using synthetic saved screenshots; no website visits.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

#[gpui::test]
#[ignore = "explicit two-size UI wording inspection"]
fn renderer_capture_ui_copy(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    let dir = tempfile::Builder::new()
        .prefix("ui-copy-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let support_report = regressee_app::diagnostics::collect(
        &store,
        None,
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for scene in [
            "comparison-default",
            "comparison-environments",
            "capture-unsaved",
            "capture-saved",
            "capture-stopped",
            "new-project",
            "access-mapping",
            "network-unavailable",
            "support-report",
        ] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.notice = None;
                    s.error = None;
                    s.navigation_preference = Some(false);
                    s.configure_review_geometry(width);
                    match scene {
                        "comparison-environments" => {
                            s.evidence.latest.as_mut().unwrap().run.environment.name =
                                "Staging".into();
                            s.viewer.as_mut().unwrap().before_environment =
                                Some("Production".into());
                        }
                        "capture-unsaved" | "capture-saved" => {
                            let baseline = s.evidence.baseline.clone().unwrap();
                            s.evidence.latest =
                                Some(store.load_run(baseline.references[0].run).unwrap());
                            s.evidence.comparison = None;
                            s.focus.candidate = scene == "capture-unsaved";
                            if scene == "capture-unsaved" {
                                s.evidence.baseline = None;
                            }
                        }
                        "capture-stopped" => {
                            s.evidence.comparison = None;
                            s.evidence.latest.as_mut().unwrap().run.targets[0].outcome =
                                TargetOutcome::Cancelled;
                            s.clear_viewer();
                        }
                        "new-project" => {
                            s.focus.screen = Screen::Projects;
                            s.create(&CreateProject, w, cx);
                        }
                        "access-mapping" => {
                            s.focus.screen = Screen::Setup;
                            s.focus.setup_stage = SetupStage::Settings;
                            s.runtime_readiness = Some(crate::app_settings_tests::readiness(
                                regressee_app::product::RuntimeState::Ready,
                                true,
                            ));
                        }
                        "network-unavailable" | "support-report" => {
                            s.focus.screen = Screen::Settings;
                            s.details.clear();
                            s.runtime_readiness = Some(crate::app_settings_tests::readiness(
                                regressee_app::product::RuntimeState::Ready,
                                true,
                            ));
                            s.summary = Some(store.summary().unwrap());
                            s.network_status = None;
                            s.pending = None;
                            s.progress = None;
                            s.details.insert(if scene == "network-unavailable" {
                                Detail::ManagedNetwork
                            } else {
                                Detail::SupportTools
                            });
                            if scene == "support-report" {
                                s.diagnostic_preview = Some(support_report.clone());
                            }
                        }
                        _ => (),
                    }
                    cx.notify();
                })
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            if matches!(scene, "network-unavailable" | "support-report") {
                shell.update(cx, |s, cx| {
                    s.screen_scroll
                        .set_offset(gpui::point(px(0.), -s.screen_scroll.max_offset().height));
                    cx.notify();
                });
            }
            if scene == "access-mapping" {
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        s.activate_control(Control::SettingsSection(SettingsSection::Access), w, cx)
                    })
                });
            }
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            capture(
                cx,
                &root,
                &format!("ui-copy-{width:.0}x{height:.0}-{scene}"),
            )
            .unwrap();
            shell.update(cx, |s, cx| finish_jobs(s, cx));
            owned.join_after_ui();
        }
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
