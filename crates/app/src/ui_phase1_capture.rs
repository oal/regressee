// SPDX-License-Identifier: MPL-2.0
//! Opt-in Phase 1 evidence. Names are backed by asserted state and painted clips.
use super::*;
use crate::{phase1_setup_tests::inside, workflow_refinement_tests::finish_jobs};
use regressee_core::domain::{
    CaptureProfile, EnvironmentId, Project, ProjectId, SessionId, SessionReference,
};

fn geometry(b: Bounds<Pixels>) -> serde_json::Value {
    serde_json::json!({"x":f32::from(b.left()),"y":f32::from(b.top()),"width":f32::from(b.size.width),"height":f32::from(b.size.height)})
}

fn private_saved(s: &mut Shell, project: ProjectId, cx: &mut Context<Shell>) {
    s.choose(Choice::LoginRequired, cx);
    s.guided_save(false, cx);
    finish_jobs(s, cx);
    assert_eq!(s.project.as_ref().unwrap().project.id, project);
    assert!(!s.unsaved(cx));
    assert!(!s.access_can_continue(cx));
    assert_eq!(s.focus.setup_stage, SetupStage::Access);
    assert!(s.task.is_none());
}

#[gpui::test]
#[ignore = "explicit Phase 1 native setup capture; offline synthetic projects only"]
fn renderer_capture_phase1(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    let reviewed: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("proof-reviewed.json")).unwrap()).unwrap();
    assert_eq!(reviewed["personally_reviewed"], true);
    assert_eq!(
        reviewed["png_sha256"],
        hex::encode(Sha256::digest(
            fs::read(root.join("renderer-proof.png")).unwrap()
        ))
    );
    cx.enable_offscreen_renderer().unwrap();
    cx.update(|cx| {
        text_input::bind(cx);
        timezone_select::bind(cx);
    });
    let scenes = [
        "website-blank",
        "website-single-origin",
        "website-multiple-unapproved",
        "website-multiple-approved",
        "website-origins-changed",
        "website-invalid",
        "website-back-dirty",
        "access-public",
        "access-private-unsaved",
        "access-private-saved",
        "access-private-status-loading",
        "access-browser-open",
        "access-ready",
        "access-options-open",
        "access-mapping-revealed",
        "access-options-closed-draft",
        "access-provider-revealed",
        "pages-essentials",
        "pages-invalid-summary",
        "pages-invalid-revealed",
        "pages-custom-tail",
        "pages-limitations",
        "runtime-checking",
        "runtime-not-installed",
        "runtime-ready",
        "runtime-development-only",
        "runtime-repair",
        "runtime-unavailable",
        "runtime-unsupported",
        "about-regressee",
    ];
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for scene in scenes {
            let dir = tempfile::Builder::new()
                .prefix("phase1-synthetic-")
                .tempdir_in(&root)
                .unwrap();
            let mut store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
            let mut draft =
                Project::draft("Phase 1 · website".into(), "https://example.test/").unwrap();
            draft.environments[0].name = "Production".into();
            draft.profiles.push(CaptureProfile::mobile());
            let project = store.save_project(&draft, None).unwrap().project.id;
            drop(store);
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            let mut reveal = None;
            let mut constructed_browser_state = false;
            cx.update(|w, cx| shell.update(cx, |s, cx| {
                finish_jobs(s, cx);
                s.focus.screen = Screen::Setup;
                s.queue(Work::Load(project), cx);
                finish_jobs(s, cx);
                s.focus.screen = Screen::Setup;
                s.focus.setup_stage = if scene.starts_with("website") { SetupStage::Website }
                    else if scene.starts_with("pages") { SetupStage::Pages } else { SetupStage::Access };
                s.setup_environment_open = false;
                s.focus.advanced = false;
                match scene {
                    "website-blank" => {
                        s.focus.screen = Screen::Projects;
                        s.activate_control(Control::NewProject, w, cx);
                        assert!(s.name.read(cx).content.is_empty() && s.address.read(cx).content.is_empty());
                        assert!(s.project.is_none());
                    },
                    "website-multiple-unapproved" | "website-multiple-approved" | "website-origins-changed" => {
                        s.address.update(cx, |f, cx| f.set_value("https://example.test/ https://second.example.test/docs", cx));
                        s.refresh_setup_origins(cx);
                        if scene != "website-multiple-unapproved" { s.activate_control(Control::Guided(11), w, cx); }
                        if scene == "website-origins-changed" {
                            s.address.update(cx, |f, cx| f.set_value("https://example.test/ https://third.example.test/docs", cx));
                            s.refresh_setup_origins(cx);
                        }
                        assert_eq!(s.setup_origin_count, 2);
                        assert_eq!(s.origins_confirmed, scene == "website-multiple-approved");
                        reveal = Some(Control::Guided(11));
                    },
                    "website-invalid" => {
                        s.address.update(cx, |f, cx| f.set_value("example.test", cx));
                        s.activate_control(Control::Save, w, cx);
                        assert_eq!(s.presentation_focus, Some(Control::Address));
                        assert!(s.pending.is_none());
                        reveal = Some(Control::Address);
                    },
                    "website-back-dirty" => {
                        s.focus.setup_stage = SetupStage::Access;
                        s.activate_control(Control::Choice(Choice::LoginRequired), w, cx);
                        s.activate_control(Control::SetupBack, w, cx);
                        assert!(s.unsaved(cx));
                        assert_eq!(s.focus.setup_stage, SetupStage::Website);
                        assert!(!s.control_enabled(Control::SetupStage(SetupStage::Access), cx));
                    },
                    "access-private-unsaved" | "access-provider-revealed" => {
                        s.activate_control(Control::Choice(Choice::LoginRequired), w, cx);
                        assert!(s.unsaved(cx));
                        assert!(!s.control_enabled(Control::Guided(31), cx));
                        if scene.ends_with("revealed") { reveal = Some(Control::CrawlField(12)); }
                    },
                    "access-private-saved" | "access-private-status-loading" | "access-browser-open" | "access-ready" => {
                        private_saved(s, project, cx);
                        if scene == "access-private-status-loading" {
                            s.environment = None;
                            assert!(!s.access_can_continue(cx));
                            assert!(!s.control_enabled(Control::Guided(31), cx));
                        } else if scene == "access-browser-open" {
                            constructed_browser_state = true;
                            s.environment.as_mut().unwrap().login_open = true;
                            assert!(s.control_enabled(Control::Guided(32), cx));
                        } else if scene == "access-ready" {
                            constructed_browser_state = true;
                            let env: EnvironmentId = s.environment_draft.unwrap();
                            let session = SessionReference { id: SessionId::new(), project, environment: env };
                            s.project.as_mut().unwrap().project.environments[0].session = Some(session.clone());
                            s.draft_settings.environments[0].session = Some(session);
                            assert!(s.access_can_continue(cx));
                        }
                    },
                    "access-options-open" | "access-mapping-revealed" | "access-options-closed-draft" => {
                        s.activate_control(Control::EnvironmentOptions, w, cx);
                        assert!(s.setup_environment_open);
                        if scene == "access-options-open" { reveal = Some(Control::Environment(0)); }
                        else {
                            s.crawl_fields[13].update(cx, |f, cx| f.set_value("https://example.test/edited", cx));
                            if scene == "access-mapping-revealed" { reveal = Some(Control::CrawlField(13)); }
                            else {
                                let signature = s.setup_signature(cx);
                                s.dismiss(&Dismiss, w, cx);
                                assert!(!s.setup_environment_open);
                                assert_eq!(s.setup_signature(cx), signature);
                                assert!(!s.control_enabled(Control::CrawlField(13), cx));
                            }
                        }
                    },
                    "pages-invalid-summary" | "pages-invalid-revealed" => {
                        s.crawl_fields[4].update(cx, |f, cx| f.set_value("9", cx));
                        s.activate_control(Control::Save, w, cx);
                        assert_eq!(s.focus.setup_stage, SetupStage::Pages);
                        assert!(s.control_enabled(Control::SetupStage(SetupStage::Pages), cx));
                        assert!(s.pending.is_none());
                        if scene.ends_with("revealed") { reveal = Some(Control::CrawlField(4)); }
                        else { s.presentation_focus = None; }
                    },
                    "pages-custom-tail" => {
                        s.activate_control(Control::Guided(6), w, cx);
                        assert!(s.focus.advanced);
                        reveal = Some(Control::Guided(12));
                    },
                    "pages-limitations" => {
                        s.activate_control(Control::Details(Detail::Capture), w, cx);
                        assert!(s.details.contains(&Detail::Capture));
                        reveal = Some(Control::Details(Detail::Capture));
                    },
                    sname if sname.starts_with("runtime-") => {
                        use regressee_app::product::{RuntimeReadiness, RuntimeState::*};
                        let (state, scope) = match sname {
                            "runtime-not-installed" => (NotInstalled, None), "runtime-ready" => (Ready, None),
                            "runtime-development-only" => (Ready, Some(regressee_runtime::distribution::Scope::DevelopmentLoopback)),
                            "runtime-repair" => (RepairRequired, None), "runtime-unavailable" => (UnavailableDistribution, None),
                            "runtime-unsupported" => (UnsupportedPlatform, None), _ => (UnavailableDistribution, None),
                        };
                        s.runtime_readiness = (sname != "runtime-checking").then_some(RuntimeReadiness {
                            status: state, scope, setup_available: matches!(state, NotInstalled | Ready | RepairRequired),
                            message: "Synthetic runtime presentation; no runtime is installed or repaired by this test.", identity: None,
                        });
                    },
                    "about-regressee" => {
                        s.focus.screen = Screen::Settings;
                        s.activate_control(Control::About, w, cx);
                        assert!(s.focus.dialog);
                    },
                    _ => (),
                }
                s.refresh_setup_origins(cx);
                s.notice = None;
                s.error = None;
                s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                cx.notify();
            }));
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            if let Some(control) = reveal {
                // First use the same Tab route as production, then allow layout
                // and the next-frame whole-field reveal to settle in its owner.
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        assert!(s.control_enabled(control, cx));
                        let stops: Vec<_> = s
                            .visible_controls()
                            .into_iter()
                            .filter(|c| s.control_enabled(*c, cx))
                            .collect();
                        let i = stops.iter().position(|c| *c == control).unwrap();
                        w.focus(&s.handle(stops[(i + stops.len() - 1) % stops.len()], cx));
                        s.move_focus(false, w, cx);
                        assert!(s.handle(control, cx).is_focused(w));
                    })
                });
                for _ in 0..3 {
                    cx.update(|w, cx| {
                        let _ = w.draw(cx);
                    });
                    cx.update(|w, cx| {
                        shell.update(cx, |s, cx| {
                            s.reveal_form_control(control, true, w, cx);
                            cx.notify();
                        })
                    });
                }
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
            }
            if scene == "pages-limitations" {
                shell.update(cx, |s, cx| {
                    s.screen_scroll
                        .set_offset(gpui::point(px(0.), -s.screen_scroll.max_offset().height));
                    cx.notify();
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
            }
            let viewport = Bounds::new(gpui::point(px(0.), px(0.)), size(px(width), px(height)));
            let painted: std::collections::BTreeMap<_, _> = [
                "action-SetupStage(Website)",
                "action-SetupStage(Access)",
                "action-SetupStage(Pages)",
                "action-Save",
                "choice-Choice(PublicAccess)",
                "choice-Choice(LoginRequired)",
                "setup-capture-plan",
                "setup-capture-limitations",
            ]
            .into_iter()
            .filter_map(|selector| cx.debug_bounds(selector).map(|b| (selector, b)))
            .collect();
            let mut assertions = Vec::new();
            shell.read_with(cx, |s, cx| {
                assert!(s.pending.is_none());
                if s.focus.screen == Screen::Setup {
                    for stage in [SetupStage::Website, SetupStage::Access, SetupStage::Pages] {
                        let selector = format!("action-SetupStage({stage:?})");
                        let b = *painted.get(selector.as_str()).unwrap();
                        inside(b, viewport);
                        assertions.push(serde_json::json!({"selector":selector,"bounds":geometry(b),"fully_visible":true,
                            "state":regressee_app::ui::setup_progress::StepState::at(stage.index(), s.focus.setup_stage.index()).label(),
                            "enabled":s.control_enabled(Control::SetupStage(stage), cx)}));
                    }
                    inside(painted["action-Save"], viewport);
                    if let Some(control) = reveal.filter(|_| scene != "pages-limitations") {
                        let b = s.setup_control_bounds(control).unwrap().get().unwrap();
                        inside(b, s.screen_scroll.bounds());
                        assertions.push(serde_json::json!({"control":format!("{control:?}"),"painted_bounds":geometry(b),"actual_scroll_clip":geometry(s.screen_scroll.bounds()),"tab_focused":true,"fully_visible":true,"mounted":s.control_mounted(control),"enabled":s.control_enabled(control,cx)}));
                    }
                    if matches!(scene, "access-public" | "access-private-unsaved" | "access-private-saved" | "access-private-status-loading" | "access-browser-open" | "access-ready") {
                        for selector in ["choice-Choice(PublicAccess)", "choice-Choice(LoginRequired)"] {
                            let b = painted[selector]; inside(b, s.screen_scroll.bounds());
                            assertions.push(serde_json::json!({"selector":selector,"bounds":geometry(b),"fully_visible":true}));
                        }
                    }
                    if scene == "pages-essentials" { inside(painted["setup-capture-plan"], s.screen_scroll.bounds()); }
                    if scene == "pages-limitations" {
                        let b = painted["setup-capture-limitations"];
                        inside(b, s.screen_scroll.bounds());
                        assertions.push(serde_json::json!({"selector":"setup-capture-limitations","painted_bounds":geometry(b),"actual_scroll_clip":geometry(s.screen_scroll.bounds()),"fully_visible":true,"trigger_opened_and_tab_reached":true,"text_revealed_by":"actual form scroll to tail"}));
                    }
                }
                let states: Vec<_> = [Control::Guided(11), Control::EnvironmentOptions, Control::Environment(0), Control::CrawlField(12), Control::CrawlField(13), Control::Guided(31), Control::Guided(32), Control::Guided(35), Control::Guided(36), Control::SetupBack]
                    .into_iter().map(|control| serde_json::json!({"control":format!("{control:?}"),"mounted":s.control_mounted(control),"enabled":s.control_enabled(control,cx)})).collect();
                manifest.push(serde_json::json!({"capture":format!("phase1-{width:.0}x{height:.0}-{scene}"),"state":scene,
                    "width":width,"height":height,"stage":format!("{:?}",s.focus.setup_stage),"dirty":s.unsaved(cx),
                    "environment_disclosure_open":s.setup_environment_open,"pages_disclosure_open":s.focus.advanced,
                    "origin_count":s.setup_origin_count,"origins_confirmed":s.origins_confirmed,
                    "mapping_draft":s.crawl_fields[13].read(cx).content,"private_login":s.draft_settings.environments[0].access == regressee_core::domain::AccessPolicy::InteractiveLogin,
                    "login_status_loading":s.environment.is_none(),"access_can_continue":s.access_can_continue(cx),
                    "constructed_browser_or_session_state":constructed_browser_state,"runtime":format!("{:?}",s.runtime_readiness),
                    "visibility_assertions":assertions,"controls":states,"synthetic":true,"website_visited":false,"os_input_verified":false}));
            });
            capture(cx, &root, &format!("phase1-{width:.0}x{height:.0}-{scene}")).unwrap();
            owned.join_after_ui();
        }
    }
    fs::write(
        root.join("phase1-matrix.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}

#[gpui::test]
#[ignore = "bounded Phase 1 FIX confirmation: actual fresh NewProject entry only"]
fn renderer_capture_phase1_fresh_entry(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    let reviewed: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("proof-reviewed.json")).unwrap()).unwrap();
    assert_eq!(reviewed["personally_reviewed"], true);
    assert_eq!(
        reviewed["png_sha256"],
        hex::encode(Sha256::digest(
            fs::read(root.join("renderer-proof.png")).unwrap()
        ))
    );
    cx.enable_offscreen_renderer().unwrap();
    cx.update(|cx| {
        text_input::bind(cx);
        timezone_select::bind(cx);
    });
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        // No saved/preselected fixture: finish the empty Projects read, then
        // activate the same mounted control as a real first-run user.
        let dir = tempfile::Builder::new()
            .prefix("fresh-entry-")
            .tempdir_in(&root)
            .unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|w, cx| {
            let s = Shell::new(cx, jobs, events);
            w.focus(&s.handle(Control::Navigate(Screen::Projects), cx));
            s
        });
        cx.simulate_resize(size(px(width), px(height)));
        let initial = shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            assert_eq!(s.focus.screen, Screen::Projects);
            assert!(s.projects.items.is_empty() && s.project.is_none());
            assert!(s.pending.is_none() && s.runtime_pending.is_none());
            assert!(s.runtime_status.is_none() && s.runtime_readiness.is_none());
            assert!(
                s.control_mounted(Control::NewProject)
                    && s.control_enabled(Control::NewProject, cx)
            );
            s.next_request
        });
        cx.update(|w, cx| {
            let _ = w.draw(cx);
            shell.update(cx, |s, cx| s.activate_control(Control::NewProject, w, cx));
        });
        for state in ["checking", "authoritative-unavailable"] {
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    if state != "checking" {
                        finish_jobs(s, cx);
                    }
                    assert_eq!(s.focus.screen, Screen::Setup);
                    assert_eq!(s.focus.setup_stage, SetupStage::Website);
                    assert_eq!(s.next_request, initial + 1);
                    if state == "checking" {
                        assert_eq!(s.runtime_pending, Some(initial + 1));
                        assert!(s.runtime_readiness.is_none());
                        assert_eq!(s.runtime_presentation().0, "Checking browser setup");
                    } else {
                        assert!(s.runtime_pending.is_none());
                        assert_eq!(
                            s.runtime_readiness.as_ref().unwrap().status,
                            regressee_app::product::RuntimeState::UnavailableDistribution
                        );
                    }
                    assert!(s.pending.is_none() && !s.focus.busy && s.task.is_none());
                    assert!(
                        s.project.is_none()
                            && s.environment.is_none()
                            && s.evidence.latest.is_none()
                    );
                    assert!(
                        s.name.read(cx).content.is_empty() && s.address.read(cx).content.is_empty()
                    );
                    assert!(s.handle(Control::Name, cx).is_focused(w));
                    assert!(s.control_enabled(Control::Save, cx));
                    assert!(!s.control_mounted(Control::Guided(11)));
                    cx.notify();
                });
                let _ = w.draw(cx);
                shell.update(cx, |s, cx| {
                    s.screen_scroll
                        .set_offset(gpui::point(px(0.), -s.screen_scroll.max_offset().height));
                    cx.notify();
                });
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            let note = cx.debug_bounds("setup-runtime-availability").unwrap();
            let viewport = Bounds::new(gpui::point(px(0.), px(0.)), size(px(width), px(height)));
            let save = cx.debug_bounds("action-Save").unwrap();
            inside(save, viewport);
            for selector in [
                "action-SetupStage(Website)",
                "action-SetupStage(Access)",
                "action-SetupStage(Pages)",
            ] {
                inside(cx.debug_bounds(selector).unwrap(), viewport);
            }
            let label = format!("phase1-fresh-entry-{width:.0}x{height:.0}-{state}");
            shell.read_with(cx, |s, cx| {
                inside(note, s.screen_scroll.bounds());
                manifest.push(serde_json::json!({
                    "capture":label,"width":width,"height":height,"state":state,
                    "entry":"mounted/enabled NewProject from fully settled empty Projects",
                    "preselected_project":false,"test_side_refresh":false,
                    "request_before":initial,"runtime_request":initial+1,"request_after":s.next_request,
                    "runtime_pending":s.runtime_pending,"runtime":format!("{:?}",s.runtime_readiness),
                    "runtime_note":s.runtime_presentation().0,"runtime_note_bounds":geometry(note),
                    "actual_scroll_clip":geometry(s.screen_scroll.bounds()),"runtime_note_fully_visible":true,
                    "revealed_by":"actual form scroll to tail after layout","save_bounds":geometry(save),
                    "save_enabled":s.control_enabled(Control::Save,cx),"name_focused":true,
                    "project":null,"environment":null,"ordinary_pending":null,"busy":false,"task":null,
                    "website_visited":false,"synthetic":true,"os_input_verified":false
                }));
            });
            capture(cx, &root, &label).unwrap();
        }
        owned.join_after_ui();
    }
    fs::write(
        root.join("fresh-entry-matrix.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
