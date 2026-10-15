// SPDX-License-Identifier: MPL-2.0
//! Synthetic layout/controller checks, not OS-input or AT qualification.
use super::*;
use crate::{
    phase4_hierarchy_tests::draw,
    workflow_refinement_tests::{finish_jobs, load_review},
};
use gpui::{TestAppContext, VisualTestContext};

pub(super) fn matrix(
    cx: &mut TestAppContext,
    mut inspect: impl FnMut(&str, &mut VisualTestContext),
) {
    cx.update(crate::keyboard_navigation_tests::bind);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, analysis, _) = approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let baseline = store
        .load_project(project)
        .unwrap()
        .project
        .active_baseline
        .unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for (scene, control) in [
            ("browser", Control::Details(Detail::Browser)),
            ("storage", Control::Details(Detail::LocalStorage)),
            ("network", Control::Details(Detail::ManagedNetwork)),
            ("network-limits", Control::Details(Detail::Network)),
            ("support", Control::Details(Detail::SupportTools)),
            ("capture-limits", Control::Details(Detail::Capture)),
            ("scope", Control::Guided(6)),
            ("environments", Control::EnvironmentOptions),
            ("ignored-areas", Control::Details(Detail::IgnoredAreas)),
            ("parameters", Control::Phase5(3)),
            ("metrics", Control::Details(Detail::Analysis)),
            ("diagnostics", Control::Phase5(38)),
            ("history", Control::Details(Detail::History)),
            ("exact", Control::Details(Detail::Portability)),
            ("source", Control::Guided(52)),
            ("search", Control::DiscoveryDetails),
            ("task", Control::TaskDetails),
        ] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                s.notice = None;
                s.error = None;
                s.details.clear();
                s.runtime_readiness = Some(app_settings_tests::readiness(
                    regressee_app::product::RuntimeState::Ready,
                    true,
                ));
                s.runtime_status = Some("Synthetic verified browser components.".into());
                s.summary = Some(store.summary().unwrap());
                s.network_status = Some("Synthetic saved network settings.".into());
                match scene {
                    "browser" | "storage" | "network" | "network-limits" | "support" => {
                        s.focus.screen = Screen::Settings;
                        if scene == "network-limits" {
                            s.details.insert(Detail::ManagedNetwork);
                        }
                    }
                    "scope" | "capture-limits" | "environments" => {
                        s.focus.screen = Screen::Setup;
                        s.focus.setup_stage = if scene == "environments" {
                            SetupStage::Access
                        } else {
                            SetupStage::Pages
                        };
                        s.focus.advanced = false;
                        s.setup_environment_open = false;
                    }
                    "ignored-areas" | "parameters" | "metrics" => {
                        s.review_tools = true;
                        s.analysis_advanced = false;
                    }
                    "diagnostics" => {
                        s.capture_details_open = true;
                        s.capture_diagnostics_open = false;
                    }
                    "history" | "exact" => {
                        s.focus.screen = Screen::History;
                        s.queue(Work::History { project, page: 0 }, cx);
                        finish_jobs(s, cx);
                        if scene == "exact" {
                            s.export_preview =
                                Some(store.export_preview(project, baseline).unwrap());
                        }
                    }
                    "source" => {
                        let key = s.selected_key.clone().unwrap();
                        s.approval_selection.insert(key.clone());
                        s.approval = Some(store.approval_preview(analysis, &[key]).unwrap());
                        s.focus.approval_dialog = true;
                    }
                    "search" => {
                        s.preview = Some(DiscoveryPreview {
                            project_revision: s.project.as_ref().unwrap().revision,
                            settings: s.project.as_ref().unwrap().project.clone(),
                            inventory: s.evidence.latest.as_ref().unwrap().run.inventory.clone(),
                            aliases: vec![],
                            notes: vec!["Synthetic page-search note.".into()],
                            visited: 1,
                        });
                        s.focus.review_surface = ReviewSurface::Pages;
                        s.result_filters_open = true;
                    }
                    "task" => {
                        s.begin_task(
                            regressee_app::task::Spec::for_work(&Work::SetupRuntime).unwrap(),
                        );
                        s.finish_task(
                            s.next_request,
                            &Reply::ProductFailure(
                                "Synthetic browser setup diagnostic for disclosure inspection."
                                    .into(),
                            ),
                        );
                    }
                    _ => unreachable!(),
                }
                cx.notify();
            });
            draw(cx);
            let request = shell.read_with(cx, |s, _| s.next_request);
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    assert!(
                        s.control_enabled(control, cx),
                        "{scene}: enabled disclosure"
                    );
                    window.focus(&s.handle(control, cx));
                    assert_eq!(s.next_request, request, "disclosure must not perform work");
                    assert!(s.handle(control, cx).is_focused(window));
                })
            });
            crate::keyboard_navigation_tests::press(cx, "enter");
            draw(cx);
            draw(cx);
            // GPUI's diagnostic selector API requires static strings. The
            // finite matrix owns only four selector allocations per scene.
            let selector = |prefix: &str| -> &'static str {
                Box::leak(format!("{prefix}-{control:?}").into_boxed_str())
            };
            let group_selector = selector("disclosure-group");
            let header = cx.debug_bounds(selector("action")).unwrap();
            let group = cx.debug_bounds(group_selector).unwrap();
            let body = cx.debug_bounds(selector("disclosure-body")).unwrap();
            let state = cx.debug_bounds(selector("disclosure-state")).unwrap();
            assert!(header.size.height >= px(40.), "{scene}: target size");
            assert!(
                header.size.width >= px(240.),
                "{scene}: finite readable header width {header:?}"
            );
            assert!(
                header.top() >= px(0.) && header.bottom() <= px(height),
                "{scene}: heading remains visible {header:?}"
            );
            assert!(
                body.top() >= header.bottom() && body.top() - header.bottom() <= px(80.),
                "{scene}: attached body {header:?} {body:?}"
            );
            assert!(
                body.bottom() <= group.bottom() + px(1.),
                "{scene}: one owner"
            );
            assert!(state.right() <= header.right() && state.left() > header.left());
            inspect(
                &format!("disclosure-{width:.0}x{height:.0}-{scene}-open"),
                cx,
            );
            let expanded = group.size.height;
            // Cover the other activation key with a real release and redraw;
            // direct controller calls miss GPUI's synthesized KeyUp click.
            crate::keyboard_navigation_tests::press(cx, "space");
            assert!(cx.debug_bounds(group_selector).unwrap().size.height < expanded);
            crate::keyboard_navigation_tests::press(cx, "space");
            assert_eq!(
                cx.debug_bounds(group_selector).unwrap().size.height,
                expanded
            );
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    if scene == "network" {
                        window.focus(&s.handle(Control::CrawlField(17), cx));
                    }
                    s.activate_control(control, window, cx);
                    assert_eq!(s.next_request, request);
                    assert!(
                        s.handle(control, cx).is_focused(window),
                        "{scene}: restore owner on collapse"
                    );
                    if scene == "network" {
                        assert!(!s.control_enabled(Control::CrawlField(17), cx));
                    }
                })
            });
            draw(cx);
            draw(cx);
            let collapsed = cx.debug_bounds(group_selector).unwrap();
            assert!(collapsed.size.height < expanded, "{scene}: body unmounts");
            if scene == "support" {
                inspect(
                    &format!("disclosure-{width:.0}x{height:.0}-{scene}-closed"),
                    cx,
                );
            }
            owned.join_after_ui();
        }
    }
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn all_inline_disclosures_attach_content_restore_focus_and_do_not_mutate(cx: &mut TestAppContext) {
    matrix(cx, |_, _| {});
}
