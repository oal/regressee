// SPDX-License-Identifier: MPL-2.0
//! Bounded native loading feedback checks; not OS-input or accessibility qualification.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{TestAppContext, point};
use regressee_core::domain::ProjectId;

pub(super) const SCENES: [&str; 9] = [
    "opening-project",
    "projects-initial",
    "history-initial",
    "results-initial",
    "screenshot",
    "browser-check",
    "storage-read",
    "saving",
    "long-operation",
];

pub(super) fn prepare_scene(
    s: &mut Shell,
    scene: &str,
    project: ProjectId,
    cx: &mut Context<Shell>,
) {
    if matches!(scene, "opening-project" | "projects-initial") {
        finish_jobs(s, cx);
        s.focus.screen = Screen::Results;
        if scene == "opening-project" {
            s.queue(Work::Load(project), cx);
        } else {
            s.projects.items.clear();
            s.projects_dirty = true;
            s.queue(Work::List { page: 0 }, cx);
        }
    } else {
        load_review(s, project, cx);
        match scene {
            "history-initial" => {
                s.focus.screen = Screen::History;
                s.history = None;
                s.queue(Work::History { project, page: 0 }, cx);
            }
            "results-initial" => {
                s.evidence = ProjectEvidence::default();
                s.clear_viewer();
                s.evidence_dirty = true;
                s.queue(Work::Evidence(project), cx);
            }
            "screenshot" => {
                s.clear_viewer();
                s.reload_viewer(cx);
            }
            "browser-check" | "storage-read" => {
                s.queue(Work::InspectStorage, cx);
                finish_jobs(s, cx);
                s.runtime_readiness = Some(crate::app_settings_tests::readiness(
                    regressee_app::product::RuntimeState::Ready,
                    true,
                ));
                s.focus.screen = Screen::Settings;
                s.project = None;
                s.focus.project_saved = false;
                s.focus.busy = true;
                if scene == "browser-check" {
                    s.runtime_readiness = None;
                    s.runtime_status = None;
                    s.progress = Some((Work::RuntimeStatus.activity_label().into(), 0, None));
                } else {
                    s.summary = None;
                    s.progress = Some((Work::InspectStorage.activity_label().into(), 0, None));
                }
            }
            "saving" | "long-operation" => {
                // Presentation fixture only: never submit a mutation to storage.
                s.focus.screen = Screen::Projects;
                s.focus.busy = true;
                s.progress = Some((
                    if scene == "saving" {
                        "Saving project settings…".into()
                    } else {
                        "Opening a project with a longer name and reading its saved website, login and screen-size settings from local storage…".repeat(2)
                    },
                    0,
                    None,
                ));
            }
            _ => panic!("unknown scene: {scene}"),
        }
    }
    s.notice = None;
    s.error = None;
    s.navigation_preference = Some(false);
    cx.notify();
}

#[gpui::test]
fn loading_feedback_has_one_owner_and_keeps_its_icon_and_copy_visible(cx: &mut TestAppContext) {
    cx.update(|cx| cx.set_global(controls::ReducedMotion(true)));
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for scene in SCENES {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| prepare_scene(s, scene, project, cx));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            if scene == "storage-read" {
                shell.update(cx, |s, _| {
                    s.screen_scroll.set_offset(point(px(0.), px(-400.)))
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
            }
            let global = matches!(scene, "opening-project" | "saving" | "long-operation");
            assert_eq!(
                cx.debug_bounds("activity-surface").is_some(),
                global,
                "{scene}"
            );
            assert!(cx.debug_bounds("progress-track").is_none(), "{scene}");
            let icon = cx.debug_bounds("activity-indicator").unwrap();
            assert_eq!(icon.size, size(px(16.), px(16.)), "{scene}");
            phase1_setup_tests::inside(
                icon,
                Bounds::new(point(px(0.), px(0.)), size(px(width), px(height))),
            );
            if global {
                let surface = cx.debug_bounds("activity-surface").unwrap();
                let label = cx.debug_bounds("activity-label").unwrap();
                phase1_setup_tests::inside(icon, surface);
                phase1_setup_tests::inside(label, surface);
                assert!(surface.size.height >= px(40.));
                assert!(surface.size.height <= px(height / 3.));
            }
            shell.update(cx, |s, cx| {
                // An error must remain independently visible, and a save must
                // never disappear just because an initial-read placeholder exists.
                s.error = Some("Synthetic read failure".into());
                assert!(!s.activity_is_inline());
                s.error = None;
                s.progress = Some(("Saving project settings…".into(), 0, None));
                assert!(!s.activity_is_inline());
                finish_jobs(s, cx);
            });
            owned.join_after_ui();
        }
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
