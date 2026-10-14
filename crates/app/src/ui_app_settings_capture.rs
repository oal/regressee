// SPDX-License-Identifier: MPL-2.0
//! Bounded, offline App settings inspection in real GPUI/Blade pixels.
use super::*;
use crate::{app_settings_tests::readiness, workflow_refinement_tests::load_review};
use regressee_app::product::RuntimeState;

#[gpui::test]
#[ignore = "explicit two-size native offscreen App settings inspection"]
fn renderer_capture_app_settings(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    cx.update(|cx| {
        text_input::bind(cx);
        timezone_select::bind(cx);
    });
    let dir = tempfile::Builder::new()
        .prefix("settings-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for scene in [
            "checking",
            "ready",
            "update",
            "repair",
            "unavailable",
            "read-failed",
            "storage",
            "network",
            "support",
        ] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.project = None;
                    s.focus.project_saved = false;
                    s.focus.screen = Screen::Settings;
                    s.notice = None;
                    s.error = None;
                    s.details.clear();
                    let mut summary = store.summary().unwrap();
                    summary.paths.data = "/home/user/.local/share/regressee".into();
                    summary.paths.cache = "/home/user/.cache/regressee".into();
                    s.summary = Some(summary);
                    s.network_status = Some("Synthetic network settings loaded.".into());
                    s.runtime_readiness = Some(readiness(RuntimeState::Ready, true));
                    s.runtime_status = Some("Synthetic verified browser status.".into());
                    match scene {
                        "checking" => {
                            s.runtime_readiness = None;
                            s.runtime_status = Some("Checking installed browser integrity…".into());
                            s.focus.busy = true;
                            s.progress =
                                Some((Work::RuntimeStatus.activity_label().into(), 0, None));
                        }
                        "update" => {
                            s.runtime_readiness =
                                Some(readiness(RuntimeState::UpdateRequired, true))
                        }
                        "repair" => {
                            s.runtime_readiness =
                                Some(readiness(RuntimeState::RepairRequired, true))
                        }
                        "unavailable" => {
                            s.runtime_readiness =
                                Some(readiness(RuntimeState::UnavailableDistribution, false))
                        }
                        "read-failed" => {
                            s.runtime_readiness = None;
                            s.runtime_status = Some(
                                "Browser availability could not be read: synthetic read failure"
                                    .into(),
                            );
                        }
                        "storage" => {
                            s.details.insert(Detail::LocalStorage);
                        }
                        "network" => {
                            s.details.insert(Detail::ManagedNetwork);
                        }
                        "support" => {
                            s.details.insert(Detail::SupportTools);
                        }
                        _ => (),
                    }
                    if scene == "ready" {
                        window.focus(&s.handle(Control::SetupRuntime, cx));
                    }
                    cx.notify();
                })
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            cx.run_until_parked();
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            if matches!(scene, "storage" | "network" | "support") {
                let control = match scene {
                    "storage" => Control::ChooseFolder,
                    "network" => Control::CrawlField(17),
                    _ => Control::Phase6(13),
                };
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        window.focus(&s.handle(control, cx));
                        s.reveal_form_control(control, true, window, cx);
                        cx.notify();
                    })
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let selector = match scene {
                    "storage" => "action-ChooseFolder",
                    "network" => "field-Proxy server (optional)",
                    _ => "action-Phase6(13)",
                };
                crate::phase1_setup_tests::inside(
                    cx.debug_bounds(selector).unwrap(),
                    cx.debug_bounds("screen-scroll").unwrap(),
                );
            } else {
                for selector in [
                    "settings-browser-status",
                    "settings-browser-state",
                    "settings-browser-action-help",
                ] {
                    crate::phase1_setup_tests::inside(
                        cx.debug_bounds(selector).unwrap(),
                        cx.debug_bounds("screen-scroll").unwrap(),
                    );
                }
            }
            let label = format!("settings-{width:.0}x{height:.0}-{scene}");
            capture(cx, &root, &label).unwrap();
            manifest.push(label);
            owned.join_after_ui();
        }
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("settings-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
