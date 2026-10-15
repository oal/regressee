// SPDX-License-Identifier: MPL-2.0
//! Two-size branding inspection; isolated synthetic evidence, no website visits.
use super::*;
use crate::workflow_refinement_tests::load_review;

#[gpui::test]
#[ignore = "explicit two-size native offscreen logo/palette inspection"]
fn renderer_capture_branding(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    cx.update(|cx| {
        text_input::bind(cx);
        timezone_select::bind(cx);
    });
    let dir = tempfile::Builder::new()
        .prefix("branding-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = Vec::new();
    let mut navigation_failures = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for scene in [
            "projects",
            "website",
            "access",
            "about",
            "review",
            "review-read-only",
            "task-discovery",
            "task-capture",
            "task-check",
            "task-browser-setup",
        ] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.notice = None;
                    s.error = None;
                    s.navigation_preference = Some(false);
                    s.configure_review_geometry(width);
                    match scene {
                        "projects" => s.focus.screen = Screen::Projects,
                        "website" | "access" => {
                            s.focus.screen = Screen::Setup;
                            s.focus.setup_stage = if scene == "website" {
                                SetupStage::Website
                            } else {
                                SetupStage::Access
                            };
                            if scene == "website" {
                                window.focus(&s.handle(Control::Name, cx));
                            }
                        }
                        "about" => {
                            s.focus.screen = Screen::Settings;
                            s.activate_control(Control::About, window, cx);
                        }
                        "review-read-only" => {
                            s.history_inspection =
                                Some((project, s.evidence.latest.as_ref().unwrap().run.id));
                        }
                        "task-discovery" | "task-capture" | "task-check" | "task-browser-setup" => {
                            let work = match scene {
                                "task-discovery" => Work::Discover(project),
                                "task-browser-setup" => Work::SetupRuntime,
                                _ => Work::CaptureInventory {
                                    project,
                                    preview: None,
                                    check: scene == "task-check",
                                    resume: None,
                                },
                            };
                            s.next_request += 1;
                            s.pending = Some(s.next_request);
                            s.focus.busy = true;
                            s.begin_task(regressee_app::task::Spec::for_work(&work).unwrap());
                        }
                        _ => (),
                    }
                    cx.notify();
                })
            });
            // Wait for GPUI's off-thread embedded PNG decoder before capturing.
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            cx.run_until_parked();
            cx.update(|window, cx| {
                for background in std::iter::once(Background::Dark)
                    .chain((scene == "about").then_some(Background::Light))
                {
                    assert!(
                        brand::logo(background)
                            .get_render_image(window, cx)
                            .is_some()
                    );
                }
                let _ = window.draw(cx);
            });
            let bounds = cx.debug_bounds("brand-lockup").unwrap();
            crate::phase1_setup_tests::inside(
                bounds,
                Bounds::new(gpui::point(px(0.), px(0.)), size(px(width), px(height))),
            );
            if scene != "about" {
                assert!(
                    f32::from(bounds.right()) <= 192.,
                    "wordmark must fit the rail"
                );
                let mark = cx.debug_bounds("brand-nav-logo").unwrap();
                let name = cx.debug_bounds("brand-nav-name").unwrap();
                assert!((f32::from(mark.size.width) - 96.).abs() < 0.1);
                assert!((f32::from(mark.size.height) - 96. * 180. / 294.).abs() < 1.);
                assert!((f32::from(mark.center().x) - 96.).abs() < 0.1);
                assert!((f32::from(name.center().x) - 96.).abs() < 0.1);
                assert!((f32::from(name.top() - mark.bottom()) - 8.).abs() < 0.1);
            }
            let label = format!("brand-{width:.0}x{height:.0}-{scene}");
            capture(cx, &root, &label).unwrap();
            manifest.push(label.clone());
            if scene.starts_with("task-") {
                assert!(cx.debug_bounds("task-workspace").is_some());
                assert!(cx.debug_bounds("action-CancelCapture").is_some());
                owned.join_after_ui();
                continue;
            }
            let mut previous_bottom = px(0.);
            for selector in [
                "action-Navigate(Projects)",
                "action-Navigate(Results)",
                "action-PagesBaseline",
                "action-Navigate(History)",
                "action-Navigate(Setup)",
                "action-Navigate(Settings)",
            ] {
                let bounds = cx.debug_bounds(selector).unwrap();
                if bounds.top() < previous_bottom
                    || bounds.bottom() > px(height)
                    || bounds.left() < px(0.)
                    || bounds.right() > px(192.)
                    || bounds.size.height < px(40.)
                {
                    navigation_failures.push(format!("{label}: {selector} {bounds:?}"));
                }
                previous_bottom = bounds.bottom();
            }
            owned.join_after_ui();
        }
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("brand-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
    assert!(navigation_failures.is_empty(), "{navigation_failures:#?}");
}
