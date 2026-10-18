// SPDX-License-Identifier: MPL-2.0
//! Recovery journeys, not just error-screen snapshots. Real isolated storage;
//! synthetic worker failures and release markers; never visit a website.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::TestAppContext;
use regressee_app::{product::RuntimeState, recovery::Problem};
use regressee_gui::{recovery::Destination, task::Spec};
use regressee_runtime::protocol::FailureCode;

#[cfg(all(target_arch = "x86_64", any(target_os = "linux", windows)))]
#[gpui::test]
fn title_save_then_discovery_offers_inline_update_without_leaving_setup(cx: &mut TestAppContext) {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::isolated(temp.path());
    let mut store = regressee_storage::Store::open(paths.clone()).unwrap();
    let project = Project::draft("Titel typo".into(), "https://example.test/").unwrap();
    store.save_project(&project, None).unwrap();
    drop(store);
    let cache = paths.cache.join("product-runtime");
    let root = cache.join("runtime-previous");
    std::fs::create_dir_all(&root).unwrap();
    let previous = format!(
        "official-24.21.0-1.64.0-{}-{}",
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    std::fs::write(root.join("READY"), &previous).unwrap();
    std::fs::write(cache.join("current"), "runtime-previous").unwrap();
    let (jobs, events) = Jobs::start_test_configured(paths.clone());
    let owned = jobs.clone();
    let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    vcx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Settings;
            s.queue(Work::Load(project.id), cx);
            finish_jobs(s, cx);
            s.name
                .update(cx, |input, cx| input.set_value("Title corrected", cx));
            assert!(s.unsaved(cx));
            s.activate_control(Control::Guided(13), window, cx);
            finish_jobs(s, cx);
            assert_eq!(s.project.as_ref().unwrap().project.name, "Title corrected");
            assert!(!s.unsaved(cx));
            let task = s.task.as_ref().unwrap();
            let result = task.result.as_ref().unwrap();
            assert_eq!(result.title, "Browser update required");
            assert!(result.message.contains("do not match this app build"));
            assert!(
                result
                    .message
                    .starts_with("Your project settings were saved.")
            );
            assert!(result.message.contains("Choose Update browser"));
            assert!(result.update_browser);
            assert_eq!(result.action, "Back to setup");
            assert!(
                !task.details,
                "essential recovery must not be hidden in details"
            );
            assert!(s.control_enabled(Control::TaskContinue, cx));
            assert!(s.control_enabled(Control::SetupRuntime, cx));
            assert!(s.preview.is_none() && s.evidence.latest.is_none());
            let request = s.next_request;
            s.activate_control(Control::SetupRuntime, window, cx);
            assert_eq!(
                s.next_request, request,
                "confirmation must precede download"
            );
            assert!(s.native_prompt);
            assert_eq!(s.focus.screen, Screen::Setup);
        })
    });
    assert!(vcx.pending_prompt().unwrap().1.contains("Saved projects"));
    vcx.simulate_prompt_answer("Cancel");
    vcx.run_until_parked();
    vcx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert!(!s.native_prompt);
            assert!(s.task.as_ref().unwrap().can_update_browser());
            assert!(s.handle(Control::SetupRuntime, cx).is_focused(window));
            assert_eq!(
                s.runtime_readiness.as_ref().unwrap().status,
                RuntimeState::UpdateRequired
            );
            assert!(s.control_enabled(Control::SetupRuntime, cx));
            // Cancel keeps both the old install and the blocked task intact.
            assert_eq!(
                std::fs::read_to_string(root.join("READY")).unwrap(),
                previous
            );
            s.activate_control(Control::TaskContinue, window, cx);
            finish_jobs(s, cx);
            assert!(s.task.is_none());
            assert_eq!(s.focus.screen, Screen::Setup);
            assert_eq!(s.name.read(cx).content.as_ref(), "Title corrected");
        })
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(paths).unwrap();
    let saved = store.load_project(project.id).unwrap();
    assert_eq!(saved.project.name, "Title corrected");
    assert!(saved.project.active_baseline.is_none());
    assert!(store.project_evidence(project.id).unwrap().latest.is_none());
    assert_eq!(
        std::fs::read_to_string(cache.join("current")).unwrap(),
        "runtime-previous"
    );
}

#[gpui::test]
fn recovery_routes_keep_drafts_and_baseline_and_do_not_retry(cx: &mut TestAppContext) {
    recovery_matrix(cx, false);
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "bounded recovery journey matrix at both desktop sizes; synthetic errors, no visits"]
fn renderer_capture_recovery(cx: &mut TestAppContext) {
    recovery_matrix(cx, true);
}

fn recovery_matrix(cx: &mut TestAppContext, render: bool) {
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    let capture_root = render.then(|| {
        let root = crate::ui_renderer_capture::output_root().unwrap();
        let proof: serde_json::Value =
            serde_json::from_slice(&std::fs::read(root.join("proof-reviewed.json")).unwrap())
                .unwrap();
        assert_eq!(proof["personally_reviewed"], true);
        use sha2::Digest as _;
        assert_eq!(
            proof["png_sha256"],
            hex::encode(sha2::Sha256::digest(
                std::fs::read(root.join("renderer-proof.png")).unwrap()
            ))
        );
        cx.enable_offscreen_renderer().unwrap();
        root
    });
    #[cfg(not(all(feature = "ui-renderer-capture", target_os = "linux")))]
    assert!(!render);
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::isolated(temp.path());
    let (store, project, _, _, _) = approval_fixture::fixture(temp.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let baseline = store.load_project(project).unwrap().project.active_baseline;
    drop(store);
    let cases = [
        (
            "update",
            Some(Problem::Browser(RuntimeState::UpdateRequired)),
        ),
        (
            "repair",
            Some(Problem::Browser(RuntimeState::RepairRequired)),
        ),
        ("login", Some(Problem::Worker(FailureCode::NeedsLogin))),
        ("timeout", Some(Problem::Worker(FailureCode::Timeout))),
        ("space", Some(Problem::LowSpace)),
        ("unknown", None),
    ];
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for (label, problem) in cases {
            let (jobs, events) = Jobs::start_without_runtime(paths.clone());
            let owned = jobs.clone();
            let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            vcx.simulate_resize(size(px(width), px(height)));
            vcx.update(|_, cx| {
                shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.focus.screen = Screen::Setup;
                    s.focus.setup_stage = SetupStage::Settings;
                    s.name
                        .update(cx, |input, cx| input.set_value("Newer unsaved title", cx));
                    s.next_request += 1;
                    s.pending = Some(s.next_request);
                    s.focus.busy = true;
                    s.begin_task(Spec::for_work(&Work::Discover(project)).unwrap());
                    let reply = problem.map_or_else(
                        || Reply::ProductFailure("A required local file could not be read.".into()),
                        Reply::ActionRequired,
                    );
                    s.apply_update(
                        Update {
                            id: s.next_request,
                            reply,
                        },
                        cx,
                    );
                    let task = s.task.as_ref().unwrap();
                    let result = task.result.as_ref().unwrap();
                    if let Some(problem) = problem {
                        assert_eq!(result.title, problem.title());
                        if label == "update" {
                            assert!(result.update_browser);
                            assert!(result.message.contains("here, then retry"));
                            assert_eq!(result.action, "Back to setup");
                            assert!(s.control_enabled(Control::SetupRuntime, cx));
                            assert_eq!(s.presentation_focus, Some(Control::SetupRuntime));
                        } else {
                            assert_eq!(result.message, problem.message());
                            assert_eq!(result.action, problem.destination().label());
                        }
                        assert!(!task.details && !task.has_details());
                    } else {
                        assert!(result.message.contains("required local file"));
                        assert!(result.message.contains("Find pages again"));
                        assert!(!result.message.contains("The operation could not finish"));
                    }
                    assert!(s.control_enabled(Control::TaskContinue, cx));
                    cx.notify();
                })
            });
            vcx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let body = vcx.debug_bounds("task-body").unwrap();
            let message = vcx.debug_bounds("task-outcome-message").unwrap();
            let action = vcx.debug_bounds("action-TaskContinue").unwrap();
            assert!(
                message.top() >= body.top() && message.bottom() <= body.bottom(),
                "essential recovery must fit without opening details"
            );
            assert!(action.bottom() <= px(height) && action.top() >= body.bottom());
            if label == "update" {
                let update = vcx.debug_bounds("action-SetupRuntime").unwrap();
                assert!(update.top() >= body.bottom() && update.bottom() <= px(height));
                assert!(update.right() <= action.left());
            }
            #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
            if let Some(root) = &capture_root {
                crate::ui_renderer_capture::capture(
                    vcx,
                    root,
                    &format!("recovery-{width:.0}x{height:.0}-{label}"),
                )
                .unwrap();
            }
            vcx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.activate_control(Control::TaskContinue, window, cx);
                    assert!(s.task.is_none());
                    let expected = match problem.map(Problem::destination) {
                        _ if label == "update" => Screen::Setup,
                        Some(Destination::AppSettings) => Screen::Settings,
                        Some(Destination::History) => Screen::History,
                        _ => Screen::Setup,
                    };
                    assert_eq!(s.focus.screen, expected);
                    // Read-only continuation is allowed; install/login/capture is not.
                    finish_jobs(s, cx);
                    assert_eq!(s.name.read(cx).content.as_ref(), "Newer unsaved title");
                    assert!(s.unsaved(cx));
                    assert_eq!(
                        s.project.as_ref().unwrap().project.active_baseline,
                        baseline
                    );
                    if label == "repair" {
                        assert!(s.control_mounted(Control::SetupRuntime));
                        assert!(s.control_enabled(Control::SetupRuntime, cx));
                    }
                    if label == "login" {
                        assert_eq!(s.focus.setup_stage, SetupStage::Settings);
                        assert!(s.control_mounted(Control::Guided(31)));
                        assert!(
                            !s.control_enabled(Control::Guided(31), cx),
                            "a preserved dirty draft must still require Save first"
                        );
                    }
                    if label == "space" {
                        assert_eq!(s.history_section, 2, "Open History must land on Storage");
                    }
                })
            });
            #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
            if let Some(root) = &capture_root
                && matches!(label, "update" | "repair" | "login" | "space")
            {
                crate::ui_renderer_capture::capture(
                    vcx,
                    root,
                    &format!("recovery-{width:.0}x{height:.0}-{label}-destination"),
                )
                .unwrap();
                if label == "repair" {
                    assert!(vcx.debug_bounds("app-settings-body").unwrap().size.width <= px(680.));
                    assert!(
                        vcx.debug_bounds("action-SetupRuntime").unwrap().bottom() <= px(height),
                        "the correction must be visible on arrival"
                    );
                }
            }
            owned.join_after_ui();
        }
    }
    let store = regressee_storage::Store::open(paths).unwrap();
    assert_eq!(
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
        original
    );
}

#[gpui::test]
fn recovery_copy_respects_checks_exports_and_projectless_setup(cx: &mut TestAppContext) {
    let temp = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(temp.path()));
    let owned = jobs.clone();
    let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    vcx.update(|_, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            let mut spec = Spec::for_work(&Work::SetupRuntime).unwrap();
            // The first-use setup task has no project; do not send it to a
            // project-dependent History screen or promise a retry will fix a build.
            for (problem, phrase) in [
                (Problem::LowSpace, "App settings lists the local data"),
                (
                    Problem::Browser(RuntimeState::UnavailableDistribution),
                    "supported version",
                ),
                (
                    Problem::Browser(RuntimeState::UnsupportedPlatform),
                    "supports your operating system and processor",
                ),
            ] {
                s.next_request += 1;
                s.begin_task(spec);
                s.finish_task(s.next_request, &Reply::ActionRequired(problem));
                let result = s.task.as_ref().unwrap().result.as_ref().unwrap();
                assert_eq!(result.recovery, Some(Destination::AppSettings));
                assert!(result.message.contains(phrase));
                s.task = None;
            }
            spec.kind = regressee_gui::task::Kind::Check;
            s.next_request += 1;
            s.begin_task(spec);
            s.finish_task(
                s.next_request,
                &Reply::ActionRequired(Problem::Browser(RuntimeState::UpdateRequired)),
            );
            assert!(
                s.task
                    .as_ref()
                    .unwrap()
                    .result
                    .as_ref()
                    .unwrap()
                    .message
                    .contains("browser version used for the baseline")
            );
            s.task = None;
            for kind in [
                regressee_gui::task::Kind::Export,
                regressee_gui::task::Kind::Diagnostics,
            ] {
                spec.kind = kind;
                s.next_request += 1;
                s.begin_task(spec);
                s.finish_task(
                    s.next_request,
                    &Reply::ActionRequired(Problem::PermissionDenied),
                );
                let result = s.task.as_ref().unwrap().result.as_ref().unwrap();
                assert_eq!(result.recovery, Some(Destination::Workspace));
                assert!(result.message.contains("choose a different folder"));
                s.task = None;
            }
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn inline_update_confirmation_dispatches_only_setup_and_preserves_the_draft(
    cx: &mut TestAppContext,
) {
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::isolated(temp.path());
    let (store, project, _, _, _) = approval_fixture::fixture(temp.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    // This backend cannot download or visit websites, even after confirmation.
    let (jobs, events) = Jobs::start_without_runtime(paths.clone());
    let owned = jobs.clone();
    let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    let request = vcx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Settings;
            s.name
                .update(cx, |input, cx| input.set_value("Unsaved title", cx));
            s.last_action = Some(Control::Guided(13));
            s.next_request += 1;
            s.pending = Some(s.next_request);
            s.begin_task(Spec::for_work(&Work::Discover(project)).unwrap());
            s.apply_update(
                Update {
                    id: s.next_request,
                    reply: Reply::ActionRequired(Problem::Browser(RuntimeState::UpdateRequired)),
                },
                cx,
            );
            let request = s.next_request;
            // The task must honor current availability, not just its old warning.
            s.runtime_readiness.as_mut().unwrap().setup_available = false;
            assert!(!s.control_enabled(Control::SetupRuntime, cx));
            s.activate_control(Control::SetupRuntime, window, cx);
            assert!(!s.native_prompt);
            assert_eq!(s.next_request, request);
            s.runtime_readiness.as_mut().unwrap().setup_available = true;
            s.activate_control(Control::SetupRuntime, window, cx);
            s.activate_control(Control::SetupRuntime, window, cx);
            assert!(s.native_prompt);
            assert_eq!(s.next_request, request);
            request
        })
    });
    vcx.simulate_prompt_answer("Replace browser components");
    vcx.run_until_parked();
    vcx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert_eq!(s.next_request, request + 1);
            let task = s.task.as_ref().unwrap();
            assert_eq!(task.spec.kind, regressee_gui::task::Kind::Setup);
            assert_eq!(task.spec.title, "Updating browser");
            assert_eq!(task.origin, Screen::Setup);
            assert_eq!(task.restore, Some(Control::Guided(13)));
            assert_eq!(task.state, regressee_gui::task::State::Running);
            assert_eq!(s.focus.setup_stage, SetupStage::Settings);
            assert!(s.error.is_none());
            assert!(s.control_enabled(Control::CancelCapture, cx));
            assert!(!s.control_enabled(Control::SetupRuntime, cx));
            s.activate_control(Control::SetupRuntime, window, cx);
            assert_eq!(s.next_request, request + 1);
            finish_jobs(s, cx);
            let result = s.task.as_ref().unwrap().result.as_ref().unwrap();
            assert!(
                !result.update_browser,
                "installer failures must not offer an update loop"
            );
            assert_eq!(result.recovery, Some(Destination::AppSettings));
            assert_eq!(s.name.read(cx).content.as_ref(), "Unsaved title");
            assert!(s.unsaved(cx));
        })
    });
    owned.join_after_ui();
    let store = regressee_storage::Store::open(paths).unwrap();
    assert_eq!(
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
        original
    );
    assert_ne!(
        store.load_project(project).unwrap().project.name,
        "Unsaved title"
    );
}

#[gpui::test]
fn inline_update_terminal_states_return_to_setup_without_visiting(cx: &mut TestAppContext) {
    inline_update_terminal_matrix(cx, false);
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "bounded inline browser update inspection; synthetic states, no downloads or visits"]
fn renderer_capture_inline_browser_update(cx: &mut TestAppContext) {
    inline_update_terminal_matrix(cx, true);
}

fn inline_update_terminal_matrix(cx: &mut TestAppContext, render: bool) {
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    let root = render.then(|| {
        cx.enable_offscreen_renderer().unwrap();
        crate::ui_renderer_capture::output_root().unwrap()
    });
    #[cfg(not(all(feature = "ui-renderer-capture", target_os = "linux")))]
    assert!(!render);
    let temp = tempfile::tempdir().unwrap();
    let paths = AppPaths::isolated(temp.path());
    let (store, project, _, _, _) = approval_fixture::fixture(temp.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for (theme, appearance) in [
            ("light", AppAppearance::Light),
            ("dark", AppAppearance::Dark),
        ] {
            let (jobs, events) = Jobs::start_without_runtime(paths.clone());
            let owned = jobs.clone();
            owned.appearance.wait_loaded();
            owned.appearance.choose(appearance);
            let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            vcx.simulate_resize(size(px(width), px(height)));
            shell.update(vcx, |s, cx| {
                load_review(s, project, cx);
                s.focus.screen = Screen::Setup;
                s.focus.setup_stage = SetupStage::Settings;
                s.name
                    .update(cx, |input, cx| input.set_value("Retained setup draft", cx));
            });
            for label in ["required", "running", "ready", "cancelled"] {
                vcx.update(|_, cx| {
                    shell.update(cx, |s, cx| {
                        s.task = None;
                        s.last_action = Some(Control::Guided(13));
                        s.next_request += 1;
                        s.pending = Some(s.next_request);
                        s.focus.busy = true;
                        s.begin_task(
                            Spec::for_work(&if label == "required" {
                                Work::Discover(project)
                            } else {
                                Work::SetupRuntime
                            })
                            .unwrap(),
                        );
                        let reply = match label {
                            "required" => Reply::ActionRequired(Problem::Browser(
                                RuntimeState::UpdateRequired,
                            )),
                            "running" => {
                                s.task.as_mut().unwrap().spec.title = "Updating browser";
                                Reply::Progress {
                                    message: "Downloading private browser".into(),
                                    completed: 25,
                                    total: Some(100),
                                }
                            }
                            "ready" => {
                                Reply::RuntimeReadiness(regressee_app::product::RuntimeReadiness {
                                    status: RuntimeState::Ready,
                                    setup_available: true,
                                    message: "Browser ready.",
                                    identity: None,
                                    scope: None,
                                })
                            }
                            "cancelled" => Reply::Cancelled,
                            _ => unreachable!(),
                        };
                        s.apply_update(
                            Update {
                                id: s.next_request,
                                reply,
                            },
                            cx,
                        );
                        assert_eq!(s.focus.screen, Screen::Setup);
                        assert_eq!(s.name.read(cx).content.as_ref(), "Retained setup draft");
                        assert!(s.unsaved(cx));
                        cx.notify();
                    })
                });
                vcx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let body = vcx.debug_bounds("task-body").unwrap();
                let action = vcx
                    .debug_bounds(if label == "running" {
                        "action-CancelCapture"
                    } else {
                        "action-TaskContinue"
                    })
                    .unwrap();
                assert!(action.top() >= body.bottom() && action.bottom() <= px(height));
                #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
                if let Some(root) = &root {
                    crate::ui_renderer_capture::capture(
                        vcx,
                        root,
                        &format!("inline-update-{width:.0}x{height:.0}-{theme}-{label}"),
                    )
                    .unwrap();
                }
                if matches!(label, "ready" | "cancelled") {
                    vcx.update(|window, cx| {
                        shell.update(cx, |s, cx| {
                            if label == "ready" {
                                assert_eq!(
                                    s.task.as_ref().unwrap().result.as_ref().unwrap().action,
                                    "Continue setup"
                                );
                            }
                            let request = s.next_request;
                            s.activate_control(Control::TaskContinue, window, cx);
                            assert!(s.task.is_none());
                            assert_eq!(s.focus.screen, Screen::Setup);
                            assert_eq!(s.focus.setup_stage, SetupStage::Settings);
                            assert_eq!(s.next_request, request, "returning must not retry or save");
                            assert_eq!(s.name.read(cx).content.as_ref(), "Retained setup draft");
                            assert!(s.unsaved(cx));
                        })
                    });
                }
            }
            owned.join_after_ui();
        }
    }
    let store = regressee_storage::Store::open(paths).unwrap();
    assert_eq!(
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
        original
    );
}
