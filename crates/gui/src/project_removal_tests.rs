// SPDX-License-Identifier: MPL-2.0
//! Isolated native-controller lifecycle tests; no user data or website visits.
use super::*;
use crate::workflow_refinement_tests::finish_jobs;
use gpui::{TestAppContext, VisualTestContext};
use regressee_storage::Store;

#[gpui::test]
fn project_removal_cancel_unselected_selected_last_and_recreation(cx: &mut TestAppContext) {
    use regressee_app::{
        engine::{Client, Server},
        protocol,
    };
    let root = tempfile::tempdir().unwrap();
    let paths = AppPaths::isolated(root.path());
    let mut store = Store::open(paths.clone()).unwrap();
    let a = Project::draft("A test project".into(), "https://a.example.test/").unwrap();
    let b = Project::draft("B retained project".into(), "https://b.example.test/").unwrap();
    store.save_project(&a, None).unwrap();
    store.save_project(&b, None).unwrap();
    drop(store);
    let server = Server::without_runtime(paths.clone()).unwrap();
    let server_thread = std::thread::spawn(move || server.run());
    let client = Client::existing(paths.clone()).unwrap();
    let (jobs, events) = Jobs::connect(client.clone());
    assert!(jobs.is_remote());
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            s.focus.screen = Screen::Setup;
            s.queue(Work::Load(b.id), cx);
            finish_jobs(s, cx);
            s.focus.screen = Screen::Projects;
            s.queue(Work::List { page: 0 }, cx);
            finish_jobs(s, cx);
            s.name
                .update(cx, |i, cx| i.set_value("Keep this unsaved draft", cx));
            s.activate_control(Control::DeleteProject(0), window, cx);
            finish_jobs(s, cx);
            assert_eq!(s.deletion_preview.as_ref().unwrap().project, a.id);
            s.activate_control(Control::Phase6(6), window, cx);
        })
    });
    let prompt = cx.pending_prompt().unwrap();
    assert!(prompt.0.contains("A test project"));
    assert!(prompt.1.contains("https://a.example.test/"));
    cx.simulate_prompt_answer("Cancel");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert!(s.pending.is_none());
            assert_eq!(s.project.as_ref().unwrap().project.id, b.id);
            s.activate_control(Control::DeleteProject(0), window, cx);
            finish_jobs(s, cx);
            s.activate_control(Control::Phase6(6), window, cx);
        })
    });
    shell.update(cx, |s, cx| {
        s.name
            .update(cx, |i, cx| i.set_value("Newer retained draft", cx))
    });
    cx.simulate_prompt_answer("Delete project");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert!(s.pending.is_none());
            assert!(s.error.as_ref().unwrap().contains("Nothing was deleted"));
            s.name
                .update(cx, |i, cx| i.set_value("Keep this unsaved draft", cx));
            s.activate_control(Control::DeleteProject(0), window, cx);
            finish_jobs(s, cx);
            s.activate_control(Control::Phase6(6), window, cx);
        })
    });
    cx.simulate_prompt_answer("Delete project");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            assert!(s.error.is_none(), "{:?}", s.error);
            assert_eq!(s.project.as_ref().unwrap().project.id, b.id);
            assert_eq!(s.name.read(cx).content.as_ref(), "Keep this unsaved draft");
            assert_eq!(s.projects.items.len(), 1);
            assert_eq!(s.projects.items[0].id, b.id);
            s.activate_control(Control::DeleteProject(0), window, cx);
            finish_jobs(s, cx);
            s.activate_control(Control::Phase6(6), window, cx);
        })
    });
    assert!(
        cx.pending_prompt()
            .unwrap()
            .0
            .contains("B retained project")
    );
    cx.simulate_prompt_answer("Delete project");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            assert!(s.project.is_none());
            assert!(s.projects.items.is_empty());
            assert_eq!(s.focus.screen, Screen::Projects);
            assert!(!s.unsaved(cx));
            assert!(s.environment_draft.is_none());
            s.create(&CreateProject, window, cx);
            s.name.update(cx, |i, cx| i.set_value("A test project", cx));
            s.address
                .update(cx, |i, cx| i.set_value("https://a.example.test/", cx));
            s.activate_control(Control::Save, window, cx);
            finish_jobs(s, cx);
            s.activate_control(Control::Save, window, cx);
            finish_jobs(s, cx);
            assert_eq!(s.focus.setup_stage, SetupStage::Pages);
            assert!(!s.unsaved(cx));
            assert!(s.error.is_none());
        })
    });
    owned.join_after_ui();
    assert!(matches!(
        client.request(protocol::Request::Stop).unwrap(),
        protocol::Response::Ok
    ));
    server_thread.join().unwrap().unwrap();
    let store = Store::open(paths).unwrap();
    assert!(store.load_project(a.id).is_err());
    assert!(store.load_project(b.id).is_err());
    assert_eq!(store.list_projects(0).unwrap().items.len(), 1);
}

#[gpui::test]
fn environment_and_mapping_removal_keep_saved_history_and_last_item_guards(
    cx: &mut TestAppContext,
) {
    let root = tempfile::tempdir().unwrap();
    let paths = AppPaths::isolated(root.path());
    let mut store = Store::open(paths.clone()).unwrap();
    let mut p = Project::draft("Environment test".into(), "https://example.test/").unwrap();
    let mut staging = p.environments[0].clone();
    staging.id = regressee_core::domain::EnvironmentId::new();
    staging.name = "Staging".into();
    let mut mapping = staging.mappings[0].clone();
    mapping.source = "https://example.test/docs/".parse().unwrap();
    mapping.destination = mapping.source.clone();
    staging.mappings.push(mapping);
    p.environments.push(staging.clone());
    store.save_project(&p, None).unwrap();
    store
        .select_environment(p.id, staging.id, &Default::default())
        .unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(paths.clone());
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window,cx|shell.update(cx,|s,cx|{
        finish_jobs(s,cx);s.focus.screen=Screen::Setup;s.focus.setup_stage=SetupStage::Settings;
        s.queue(Work::Load(p.id),cx);finish_jobs(s,cx);
        s.focus.mappings=2;
        assert!(s.control_enabled(Control::RemoveMapping(1),cx));
        s.activate_control(Control::RemoveMapping(1),window,cx);
        assert!(s.pending.is_none());assert!(s.unsaved(cx));
        assert_eq!(s.project.as_ref().unwrap().project.environments[1].mappings.len(),2);
        assert_eq!(s.draft_settings.environments[1].mappings.len(),1);
        assert!(!s.control_mounted(Control::RemoveMapping(0)));
        assert!(!s.control_enabled(Control::RemoveEnvironment,cx));
        s.activate_control(Control::Save,window,cx);finish_jobs(s,cx);
        assert!(!s.unsaved(cx));
        s.activate_control(Control::RemoveEnvironment,window,cx);finish_jobs(s,cx);
        assert!(matches!(s.deletion_preview.as_ref().unwrap().item,regressee_storage::DeleteItem::Environment(id) if id==staging.id));
        s.activate_control(Control::Phase6(6),window,cx);
    }));
    assert!(cx.pending_prompt().unwrap().0.contains("Staging"));
    cx.simulate_prompt_answer("Remove environment");
    cx.run_until_parked();
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            assert!(s.error.is_none(), "{:?}", s.error);
            assert_eq!(s.draft_settings.environments.len(), 1);
            assert_eq!(s.environment_draft, Some(p.environments[0].id));
            assert!(!s.control_mounted(Control::RemoveEnvironment));
            let revision = s.project.as_ref().unwrap().revision;
            s.activate_control(Control::Guided(27), window, cx);
            assert_eq!(s.draft_settings.environments.len(), 2);
            s.activate_control(Control::RemoveEnvironment, window, cx);
            assert!(s.pending.is_none());
            assert_eq!(s.draft_settings.environments.len(), 1);
            assert_eq!(s.project.as_ref().unwrap().revision, revision);
            assert!(!s.unsaved(cx));
        })
    });
    owned.join_after_ui();
    let store = Store::open(paths).unwrap();
    assert_eq!(
        store.load_project(p.id).unwrap().project.environments.len(),
        1
    );
}

fn draw(cx: &mut VisualTestContext) {
    cx.run_until_parked();
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}

fn removal_layout(cx: &mut TestAppContext, mut inspect: impl FnMut(&str, &mut VisualTestContext)) {
    cx.update(|cx| {
        regressee_gui::ui::dialog::install(cx);
    });
    for preference in [
        regressee_core::appearance::AppAppearance::Light,
        regressee_core::appearance::AppAppearance::Dark,
    ] {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(root.path());
        let (mut store, project, _, _, _) = approval_fixture::fixture(root.path());
        let mut saved = store.load_project(project).unwrap();
        let mut staging = saved.project.environments[0].clone();
        staging.id = regressee_core::domain::EnvironmentId::new();
        staging.name = "Staging preview".into();
        let mut mapping = staging.mappings[0].clone();
        mapping.source = "https://example.test/docs/".parse().unwrap();
        mapping.destination = "https://staging.example.test/documentation/"
            .parse()
            .unwrap();
        staging.mappings.push(mapping);
        saved.project.environments.push(staging.clone());
        store
            .save_replacement_project(&saved.project, saved.revision)
            .unwrap();
        store
            .select_environment(project, staging.id, &Default::default())
            .unwrap();
        drop(store);
        regressee_storage::AppPreferences {
            appearance: preference,
            ..Default::default()
        }
        .save(&paths)
        .unwrap();
        let (jobs, events) = Jobs::start_without_runtime(paths);
        let owned = jobs.clone();
        owned.appearance.wait_loaded();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            s.focus.screen = Screen::Setup;
            s.queue(Work::Load(project), cx);
            finish_jobs(s, cx);
        });
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            let prefix = format!(
                "removal-{}-{width:.0}x{height:.0}",
                preference.label().to_lowercase()
            );
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.navigate(Screen::Projects, w, cx);
                    finish_jobs(s, cx);
                })
            });
            draw(cx);
            let action = cx.debug_bounds("action-DeleteProject(0)").unwrap();
            assert!(action.right() <= px(width));
            assert!(action.size.height >= px(CONTROL_HEIGHT));
            inspect(&format!("{prefix}-projects"), cx);
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.activate_control(Control::DeleteProject(0), w, cx);
                    finish_jobs(s, cx);
                })
            });
            draw(cx);
            assert!(cx.debug_bounds("confirmation-dialog").is_some());
            assert!(cx.debug_bounds("confirmation-action-0").unwrap().bottom() <= px(height));
            assert!(cx.debug_bounds("confirmation-action-1").unwrap().bottom() <= px(height));
            inspect(&format!("{prefix}-preview"), cx);
            cx.simulate_keystrokes("escape");
            cx.run_until_parked();
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    assert!(!s.native_prompt);
                    s.navigate(Screen::Setup, w, cx);
                    finish_jobs(s, cx);
                })
            });
            draw(cx);
            for (control, scene) in [
                (Control::DeleteCurrentProject, "project-settings"),
                (Control::RemoveEnvironment, "environments"),
                (Control::RemoveMapping(1), "mappings"),
            ] {
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        s.reveal_form_control(control, true, w, cx);
                        w.focus(&s.handle(control, cx));
                        cx.notify();
                    })
                });
                draw(cx);
                let selector = format!("action-{control:?}");
                let b = cx.debug_bounds(selector.leak()).unwrap();
                assert!(
                    b.top() >= px(0.) && b.bottom() <= px(height),
                    "{scene}: {b:?}"
                );
                inspect(&format!("{prefix}-{scene}"), cx);
                if scene == "environments" {
                    cx.update(|w, cx| {
                        shell.update(cx, |s, cx| {
                            s.activate_control(Control::RemoveEnvironment, w, cx);
                            finish_jobs(s, cx);
                        })
                    });
                    draw(cx);
                    assert!(cx.debug_bounds("confirmation-dialog").is_some());
                    inspect(&format!("{prefix}-environment-preview"), cx);
                    cx.simulate_keystrokes("escape");
                    cx.run_until_parked();
                }
            }
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.navigate(Screen::Settings, w, cx);
                    finish_jobs(s, cx);
                    // Synthetic presentation of the independently tested pending-GC receipt.
                    s.summary.as_mut().unwrap().pending_cleanup = 2;
                    cx.notify();
                })
            });
            draw(cx);
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.reveal_form_control(Control::Details(Detail::LocalStorage), true, w, cx)
                })
            });
            draw(cx);
            assert!(cx.debug_bounds("settings-pending-cleanup").is_some());
            inspect(&format!("{prefix}-pending-cleanup"), cx);
        }
        owned.join_after_ui();
        cx.update(|w, _| w.remove_window());
    }
    set_dark(false);
}

#[gpui::test]
fn removal_actions_and_impact_fit_both_themes_and_supported_sizes(cx: &mut TestAppContext) {
    removal_layout(cx, |_, _| {});
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "explicit bounded project-removal visual inspection"]
fn renderer_capture_project_removal(cx: &mut TestAppContext) {
    let root = crate::ui_renderer_capture::output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    removal_layout(cx, |label, cx| {
        crate::ui_renderer_capture::capture(cx, &root, label).unwrap()
    });
}
