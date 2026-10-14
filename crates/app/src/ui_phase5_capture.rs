// SPDX-License-Identifier: MPL-2.0
//! Targeted final integration, not a rerun of the historical whole-app matrix.
//! Every toggle is activated through mounted controls; reading uses real wheel
//! dispatch. Constructed offline fixtures/pending presentations are labelled.
use super::*;
use crate::phase1_setup_tests::inside;
use crate::phase4_hierarchy_tests::{activate, draw, extend_history, tab_to};
use crate::phase5_state_tests::{assert_form, edit, gap_check_from, read_at};
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use regressee_core::domain::{ProjectId, RunId};
use std::sync::atomic::AtomicBool;

#[derive(Clone, Copy)]
enum Owner {
    Window,
    Form,
    Workspace,
    Inspector,
    Options,
    Runs,
}

fn acknowledge_feedback(shell: &Entity<Shell>, cx: &mut VisualTestContext) {
    if shell.read_with(cx, |s, cx| {
        s.control_mounted(Control::DismissFeedback)
            && s.control_enabled(Control::DismissFeedback, cx)
    }) {
        activate(shell, cx, Control::DismissFeedback);
    }
}

fn geometry(b: Bounds<Pixels>) -> serde_json::Value {
    serde_json::json!({"x":f32::from(b.left()),"y":f32::from(b.top()),"width":f32::from(b.size.width),"height":f32::from(b.size.height)})
}

fn record(
    shell: &Entity<Shell>,
    cx: &mut VisualTestContext,
    root: &Path,
    scene: &str,
    selectors: &[(&str, Owner)],
    ledger: &mut Vec<serde_json::Value>,
) {
    draw(cx);
    let viewport = cx.update(|w, _| Bounds::new(gpui::point(px(0.), px(0.)), w.viewport_size()));
    let label = format!(
        "phase5-{:.0}x{:.0}-{scene}",
        f32::from(viewport.size.width),
        f32::from(viewport.size.height)
    );
    eprintln!("Final integration state: {label}");
    let workspace = if selectors.iter().any(|(_, o)| matches!(o, Owner::Workspace)) {
        cx.debug_bounds("bounded-review")
    } else {
        None
    };
    let measured:Vec<_>=selectors.iter().map(|(selector,owner)| {
        let b=cx.debug_bounds(selector.to_string().leak()).unwrap_or_else(||panic!("{label}: absent {selector}"));
        let clip=shell.read_with(cx,|s,_| match owner {
            Owner::Window=>viewport, Owner::Form=>s.screen_scroll.bounds(),
            Owner::Workspace=>workspace.unwrap(),
            Owner::Inspector=>s.review_details_scroll.bounds(),
            Owner::Options=>s.options_scroll.bounds(), Owner::Runs=>s.history_run_scroll.bounds(),
        });
        inside(b,clip); inside(b,viewport);
        serde_json::json!({"selector":selector,"bounds":geometry(b),"owner":match owner {Owner::Window=>"window",Owner::Form=>"screen-scroll",Owner::Workspace=>"bounded-review",Owner::Inspector=>"inspector-scroll",Owner::Options=>"options-scroll",Owner::Runs=>"history-run-scroll"},"clip":geometry(clip),"fully_visible":true})
    }).collect();
    let state=cx.update(|w,cx|shell.read_with(cx,|s,cx|{
        let mut state = serde_json::json!({
            "screen":format!("{:?}",s.focus.screen),"stage":format!("{:?}",s.focus.setup_stage),
            "focused":s.focused(w,cx).map(|c|format!("{c:?}")),"last_action":s.last_action.map(|c|format!("{c:?}")),
            "mounted_enabled":s.visible_controls().into_iter().filter(|c|s.control_enabled(*c,cx)).map(|c|format!("{c:?}")).collect::<Vec<_>>(),
            "setup_dirty":s.unsaved(cx),"advanced":s.focus.advanced,"optional_access":s.setup_environment_open,
            "private_access":s.draft_settings.environments.iter().find(|e|Some(e.id)==s.environment_draft).map(|e|format!("{:?}",e.access)),
            "login_status_loading":s.environment.is_none(),"access_can_continue":s.access_can_continue(cx),
            "ordinary_pending":s.pending,"task_present":s.task.is_some(),"request_counter":s.next_request,
            "profiles":s.draft_settings.profiles.iter().map(|p|serde_json::json!({"name":p.name,"width":p.width,"height":p.height,"extent":p.extent,"images":p.image_mode})).collect::<Vec<_>>(),
            "speed":s.draft_settings.discovery.speed,
            "parallel_field":s.crawl_fields[4].read(cx).content,"parallel_error":s.crawl_fields[4].read(cx).validation_error(),
            "selected_page_included":s.preview.as_ref().and_then(|p|p.inventory.pages.iter().find(|p|Some(p.id)==s.selected_page)).map(|p|p.included),
            "discovered":s.preview.as_ref().map(|p|p.inventory.pages.len()),"included":s.preview.as_ref().map(|p|p.included()),"matching_pages":s.preview_page_ids().len(),
            "filter":s.result_filter,"group_profiles":s.group_profiles,"shown_results":s.result_indices(),
            "shown_profiles":s.evidence.latest.as_ref().map(|r|s.result_indices().into_iter().map(|i|r.run.profiles.iter().find(|p|p.id==r.run.targets[i].key.profile).unwrap().name.clone()).collect::<Vec<_>>()),
            "shown_classifications":s.evidence.latest.as_ref().map(|r|s.result_indices().into_iter().map(|i|s.evidence.comparison.as_ref().and_then(|a|a.revision.results.iter().find(|a|a.key==r.run.targets[i].key)).map(|r|r.classification.label())).collect::<Vec<_>>()),
        });
        let review = serde_json::json!({
            "view":s.viewer_mode,"after_side":s.full_width_after,"results_hidden":s.review_queue_collapsed,"navigation_hidden":s.navigation_collapsed,
            "view_menu":s.viewer_mode_open,"zoom_menu":s.zoom_select.read(cx).is_open(),"fit":s.viewer_fit,"scale":s.zoom,
            "selected_key":s.selected_key,"pair":s.viewer.as_ref().and_then(|v|v.request.as_ref()).map(|r|serde_json::json!({"before":r.before,"after":r.after,"mask":r.mask})),
            "note_open":s.review_note_open,"note_dirty":s.review_note_dirty(cx),"note_bytes":s.analysis_fields[9].read(cx).content.len(),
            "note_sha256":hex::encode(Sha256::digest(s.analysis_fields[9].read(cx).content.as_bytes())),"note_status":s.note_status(cx).0,
            "mask_dirty":s.mask_dirty,"coordinate_draft":s.mask_geometry_dirty(cx),"blocker":s.review_blocking_reason(cx),
            "selection_count":s.approval_selection.len(),"analysis_open":s.review_tools,"ignored_open":s.details.contains(&Detail::IgnoredAreas),"analysis_advanced":s.analysis_advanced,
            "policy":crate::analysis_ux::policy_name(&s.analysis_policy),"retained_analyses":s.evidence.analyses.len(),
            "history_page":s.history.as_ref().map(|h|h.page),"history_more":s.history.as_ref().map(|h|h.more),"loaded_runs":s.history.as_ref().map(|h|h.runs.len()),"selected_run_index":s.run_history_index,
            "provenance_open":s.details.contains(&Detail::History),"portability_exact":s.details.contains(&Detail::Portability),
            "export_preview":s.export_preview.is_some(),"import_preview":s.import_preview.is_some(),"delete_preview":s.deletion_preview.is_some(),
            "network_limits":s.details.contains(&Detail::Network),"baseline":s.project.as_ref().and_then(|p|p.project.active_baseline),
        });
        state.as_object_mut().unwrap().extend(review.as_object().unwrap().clone());
        state
    }));
    if std::env::var_os("REGRESSEE_PHASE5_PREFLIGHT").is_none() {
        capture(cx, root, &label).unwrap();
    }
    ledger.push(serde_json::json!({"label":label,"scene":scene,"size":[f32::from(viewport.size.width),f32::from(viewport.size.height)],"state":state,"geometry":measured,
        "qualification":"isolated synthetic fixture; supported simulated mounted activation/wheel, not OS input","website_visited":false}));
    fs::write(
        root.join("phase5-matrix-progress.json"),
        serde_json::to_vec_pretty(ledger).unwrap(),
    )
    .unwrap();
}

fn setup(
    cx: &mut TestAppContext,
    root: &Path,
    width: f32,
    height: f32,
    ledger: &mut Vec<serde_json::Value>,
) {
    let dir = tempfile::Builder::new()
        .prefix("phase5-setup-")
        .tempdir_in(root)
        .unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.simulate_resize(size(px(width), px(height)));
    shell.update(cx, finish_jobs);
    activate(&shell, cx, Control::NewProject);
    shell.update(cx, finish_jobs);
    edit(&shell, cx, Control::Name, "Final offline setup");
    edit(&shell, cx, Control::Address, "https://example.test/");
    record(
        &shell,
        cx,
        root,
        "wizard-website",
        &[
            ("setup-progress", Owner::Window),
            ("action-Save", Owner::Window),
        ],
        ledger,
    );
    activate(&shell, cx, Control::Save);
    shell.update(cx, finish_jobs);
    acknowledge_feedback(&shell, cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.focus.setup_stage, SetupStage::Access)
    });
    // Assert the actual Public reading position, not simultaneous visibility
    // of both choice rows in the smaller pinned-context/footer viewport.
    assert_form(&shell, cx, Control::Choice(Choice::PublicAccess));
    record(
        &shell,
        cx,
        root,
        "access-public",
        &[("choice-Choice(PublicAccess)", Owner::Form)],
        ledger,
    );
    activate(&shell, cx, Control::EnvironmentOptions);
    assert_form(&shell, cx, Control::CrawlField(13));
    record(
        &shell,
        cx,
        root,
        "access-optional-mapping",
        &[("field-Source origin or section base", Owner::Form)],
        ledger,
    );
    activate(&shell, cx, Control::EnvironmentOptions);
    activate(&shell, cx, Control::Choice(Choice::LoginRequired));
    activate(&shell, cx, Control::Save);
    shell.update(cx, finish_jobs);
    acknowledge_feedback(&shell, cx);
    // A real read-only status request is outstanding; paint before consuming its
    // authoritative reply. No browser/login/session success is fabricated.
    shell.update(cx, |s, cx| {
        s.environment = None;
        s.queue(
            Work::EnvironmentStatus(s.project.as_ref().unwrap().project.id),
            cx,
        );
        assert!(s.pending.is_some());
        assert!(!s.access_can_continue(cx));
        assert!(!s.control_enabled(Control::Guided(31), cx));
    });
    read_at(&shell, cx, "choice-Choice(LoginRequired)");
    record(
        &shell,
        cx,
        root,
        "private-login-loading",
        &[
            ("setup-footer-status", Owner::Window),
            ("choice-Choice(LoginRequired)", Owner::Form),
        ],
        ledger,
    );
    shell.update(cx, finish_jobs);
    activate(&shell, cx, Control::Choice(Choice::PublicAccess));
    activate(&shell, cx, Control::Save);
    shell.update(cx, finish_jobs);
    acknowledge_feedback(&shell, cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.focus.setup_stage, SetupStage::Pages)
    });
    activate(&shell, cx, Control::Guided(3));
    assert_form(&shell, cx, Control::Guided(3));
    shell.read_with(cx, |s, _| {
        assert!(s.draft_settings.profiles.iter().any(|p| p.name == "Mobile"))
    });
    record(
        &shell,
        cx,
        root,
        "mobile-revealed-selected",
        &[("check-Guided(3)", Owner::Form)],
        ledger,
    );
    activate(&shell, cx, Control::Guided(2));
    record(
        &shell,
        cx,
        root,
        "tablet-selected",
        &[("check-Guided(2)", Owner::Form)],
        ledger,
    );
    activate(&shell, cx, Control::Guided(6));
    edit(&shell, cx, Control::CrawlField(7), "1024");
    record(
        &shell,
        cx,
        root,
        "custom-width",
        &[("field-Custom width (CSS pixels)", Owner::Form)],
        ledger,
    );
    edit(&shell, cx, Control::CrawlField(8), "768");
    record(
        &shell,
        cx,
        root,
        "custom-height",
        &[("field-Custom height (CSS pixels)", Owner::Form)],
        ledger,
    );
    activate(&shell, cx, Control::Guided(12));
    assert_form(&shell, cx, Control::Guided(25));
    shell.read_with(cx, |s, _| {
        assert!(
            s.draft_settings
                .profiles
                .iter()
                .any(|p| p.name == "Custom" && p.width == 1024 && p.height == 768)
        )
    });
    record(
        &shell,
        cx,
        root,
        "custom-added-actions",
        &[
            ("action-Guided(12)", Owner::Form),
            ("action-Guided(25)", Owner::Form),
        ],
        ledger,
    );
    read_at(&shell, cx, "setup-image-limitations");
    record(
        &shell,
        cx,
        root,
        "image-limitations-tail",
        &[("setup-image-limitations", Owner::Form)],
        ledger,
    );
    activate(&shell, cx, Control::Details(Detail::Capture));
    read_at(&shell, cx, "setup-capture-limitations");
    record(
        &shell,
        cx,
        root,
        "capture-limitations-tail",
        &[("setup-capture-limitations", Owner::Form)],
        ledger,
    );
    for (choice, scene, selector) in [
        (
            Choice::VisibleScreen,
            "visible-screen-selected",
            "choice-Choice(VisibleScreen)",
        ),
        (
            Choice::ReplaceImages,
            "replacement-selected",
            "choice-Choice(ReplaceImages)",
        ),
        (Choice::Fast, "fast-selected", "choice-Choice(Fast)"),
    ] {
        activate(&shell, cx, Control::Choice(choice));
        record(&shell, cx, root, scene, &[(selector, Owner::Form)], ledger);
    }
    shell.read_with(cx, |s, _| {
        assert_eq!(s.draft_settings.discovery.speed, Speed::Fast);
        assert!(
            s.draft_settings
                .profiles
                .iter()
                .all(|p| p.extent == CaptureExtent::VisibleScreen
                    && p.image_mode != regressee_core::domain::ImageMode::Normal)
        );
    });
    edit(&shell, cx, Control::CrawlField(4), "9");
    let requests = shell.read_with(cx, |s, _| s.next_request);
    activate(&shell, cx, Control::Save);
    assert_form(&shell, cx, Control::CrawlField(4));
    shell.read_with(cx, |s, cx| {
        assert_eq!(requests, s.next_request);
        assert!(
            s.crawl_fields[4].read(cx).touched
                && s.crawl_fields[4].read(cx).validation_error().is_some()
        );
        assert!(s.focus.advanced && s.pending.is_none() && s.task.is_none());
    });
    record(
        &shell,
        cx,
        root,
        "invalid-advanced-revealed",
        &[
            ("field-Parallel active pages", Owner::Form),
            ("field-correction", Owner::Form),
            ("setup-progress", Owner::Window),
        ],
        ledger,
    );
    owned.join_after_ui();
}

fn masked_fixture(root: &Path, path: &Path) -> (regressee_storage::Store, ProjectId, RunId) {
    let (mut store, project, original_check) =
        super::ui_review_matrix::fixture_named(root, path, true);
    let source = store
        .project_evidence(project)
        .unwrap()
        .baseline
        .unwrap()
        .references[0]
        .run;
    extend_history(&mut store, project, source, 68);
    let check = gap_check_from(&mut store, original_check);
    let evidence = store.project_evidence(project).unwrap();
    let mut request = evidence
        .comparison
        .as_ref()
        .unwrap()
        .perceptual
        .as_ref()
        .unwrap()
        .reports
        .iter()
        .find(|(k, _)| {
            evidence
                .latest
                .as_ref()
                .unwrap()
                .run
                .targets
                .iter()
                .any(|t| t.key == *k && t.requested.path().starts_with("/account"))
        })
        .unwrap()
        .1
        .request
        .clone();
    request.mask = Some(regressee_app::analysis_engine::MaskBinding {
        revision: "final-synthetic-ignore".into(),
        target: request.before.target.clone(),
        before_capture_id: request.before.capture_id.clone(),
        after_capture_id: request.after.capture_id.clone(),
        before_dimensions: request.before.dimensions,
        after_dimensions: request.after.dimensions,
        rectangles: vec![regressee_app::analysis_engine::Rectangle {
            x: 700,
            y: 160,
            width: 400,
            height: 300,
        }],
    });
    store.save_mask(project, &request).unwrap();
    let analysis = regressee_app::review::analyze_run(
        &mut store,
        check,
        request.policy.clone(),
        &AtomicBool::new(false),
    )
    .unwrap();
    // Reports are authoritative production output; both legend colors have data.
    assert!(
        analysis
            .perceptual
            .as_ref()
            .unwrap()
            .reports
            .iter()
            .any(|(_, r)| r.ignored_pixels == 120_000 && r.changed_pixels > 0)
    );
    (store, project, check)
}

fn view(shell: &Entity<Shell>, cx: &mut VisualTestContext, mode: usize) {
    activate(shell, cx, Control::Guided(54));
    // Menu rows use arrows, not Tab (Tab deliberately dismisses the selector).
    cx.simulate_keystrokes("home");
    for _ in 0..mode {
        cx.simulate_keystrokes("down");
    }
    cx.simulate_keystrokes("enter");
    draw(cx);
    shell.update(cx, finish_jobs);
    shell.read_with(cx, |s, _| assert_eq!(s.viewer_mode, mode));
}

#[gpui::test]
#[ignore = "explicit one targeted final two-size batch; synthetic/offline only"]
fn renderer_capture_phase5(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    let proof: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("proof-reviewed.json")).unwrap()).unwrap();
    assert_eq!(proof["personally_reviewed"], true);
    assert_eq!(
        proof["png_sha256"],
        hex::encode(Sha256::digest(
            fs::read(root.join("renderer-proof.png")).unwrap()
        ))
    );
    let preflight = std::env::var_os("REGRESSEE_PHASE5_PREFLIGHT").is_some();
    let preflight_size = std::env::var("REGRESSEE_PHASE5_PREFLIGHT_SIZE").ok();
    assert!(
        preflight || preflight_size.is_none(),
        "Size filtering is non-rendering preflight only"
    );
    if let Some(size) = &preflight_size {
        assert!(matches!(size.as_str(), "1280x800" | "820x600"));
    }
    if !preflight {
        cx.enable_offscreen_renderer().unwrap();
    }
    cx.update(|cx| {
        text_input::bind(cx);
        zoom_select::bind(cx);
        cx.bind_keys([
            KeyBinding::new("escape", Dismiss, Some("Shell")),
            KeyBinding::new("enter", Activate, Some("Shell")),
            KeyBinding::new("tab", Tab, Some("Shell")),
            KeyBinding::new("shift-tab", TabPrev, Some("Shell")),
        ]);
    });
    let dir = tempfile::Builder::new()
        .prefix("phase5-evidence-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, check) = masked_fixture(&root, dir.path());
    let baseline = store
        .load_project(project)
        .unwrap()
        .project
        .active_baseline
        .unwrap();
    let export = store.export_preview(project, baseline).unwrap();
    let archive = dir.path().join("synthetic-baseline.zip");
    store
        .export_archive(&export, &archive, &AtomicBool::new(false))
        .unwrap();
    let originals: Vec<_> = store
        .project_evidence(project)
        .unwrap()
        .latest
        .unwrap()
        .captures
        .iter()
        .map(|c| {
            let id = c.image.original.clone();
            let hash = hex::encode(Sha256::digest(
                fs::read(store.artifact_path(&id).unwrap()).unwrap(),
            ));
            (id, hash)
        })
        .collect();
    drop(store);
    let mut ledger = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        if preflight_size
            .as_ref()
            .is_some_and(|s| *s != format!("{width:.0}x{height:.0}"))
        {
            continue;
        }
        setup(cx, &root, width, height, &mut ledger);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.notice = None;
            s.error = None;
            crate::options_ux::fixture_discovery(s);
        });
        activate(&shell, cx, Control::Guided(45));
        for (action, scene, count) in [
            (22, "discovery-excluded", 74),
            (22, "discovery-included", 75),
        ] {
            activate(&shell, cx, Control::Guided(action));
            shell.read_with(cx, |s, _| {
                assert_eq!(s.preview.as_ref().unwrap().included(), count)
            });
            record(
                &shell,
                cx,
                &root,
                scene,
                &[("action-Guided(22)", Owner::Options)],
                &mut ledger,
            );
        }
        edit(&shell, cx, Control::CrawlField(9), "/pages/1/");
        for (action, scene, count) in [
            (20, "discovery-matches-excluded", 74),
            (21, "discovery-matches-restored", 75),
        ] {
            activate(&shell, cx, Control::Guided(action));
            shell.read_with(cx, |s, _| {
                assert_eq!(s.preview.as_ref().unwrap().included(), count);
                assert_eq!(s.preview_page_ids().len(), 1);
            });
            record(
                &shell,
                cx,
                &root,
                scene,
                &[
                    ("action-Guided(20)", Owner::Options),
                    ("action-Guided(21)", Owner::Options),
                ],
                &mut ledger,
            );
        }
        activate(&shell, cx, Control::Guided(16));
        activate(&shell, cx, Control::Guided(17));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.notice = None;
            s.error = None;
        });
        // Select the actual long-path result through the production next-target
        // control; never assign a selected key to manufacture this inspection.
        for _ in 0..12 {
            if shell.read_with(cx, |s, _| s.inspection_identity().0.starts_with("/account")) {
                break;
            }
            activate(&shell, cx, Control::Guided(24));
            shell.update(cx, finish_jobs);
        }
        shell.read_with(cx, |s, _| {
            assert!(s.inspection_identity().0.starts_with("/account"))
        });
        view(&shell, cx, 0);
        record(
            &shell,
            cx,
            &root,
            "review-compare",
            &[
                ("phase5-before", Owner::Workspace),
                ("phase5-after", Owner::Workspace),
                ("viewer-toolbar", Owner::Workspace),
            ],
            &mut ledger,
        );
        let clean = cx.debug_bounds("phase5-before").unwrap().size.height;
        activate(&shell, cx, Control::Guided(50));
        assert!(cx.debug_bounds("phase5-before").unwrap().size.height >= clean);
        record(
            &shell,
            cx,
            &root,
            "results-hidden",
            &[
                ("phase5-before", Owner::Workspace),
                ("inspected-target-context", Owner::Workspace),
            ],
            &mut ledger,
        );
        activate(&shell, cx, Control::Guided(53));
        record(
            &shell,
            cx,
            &root,
            "navigation-toggled",
            &[
                ("viewer-toolbar", Owner::Workspace),
                ("inspected-target-context", Owner::Workspace),
            ],
            &mut ledger,
        );
        // Compare evidence heights with the same navigation state. The narrow
        // navigation-open frame above intentionally has a smaller working width.
        activate(&shell, cx, Control::Guided(53));
        view(&shell, cx, 4);
        assert!(cx.debug_bounds("phase5-before").unwrap().size.height >= clean);
        record(
            &shell,
            cx,
            &root,
            "full-before",
            &[("phase5-before", Owner::Workspace)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Guided(56));
        record(
            &shell,
            cx,
            &root,
            "full-after",
            &[("phase5-after", Owner::Workspace)],
            &mut ledger,
        );
        // Restore the normal-width queue via actual controls, retaining target.
        activate(&shell, cx, Control::Guided(50));
        for (mode, scene, pane) in [
            (1, "difference", "phase5-difference"),
            (2, "overlay", "phase5-before"),
            (3, "heatmap-with-ignored", "phase5-difference"),
        ] {
            view(&shell, cx, mode);
            let mut selectors = vec![(pane, Owner::Workspace)];
            if mode == 3 {
                selectors.push(("heatmap-legend", Owner::Workspace));
            }
            record(&shell, cx, &root, scene, &selectors, &mut ledger);
        }
        view(&shell, cx, 0);
        activate(&shell, cx, Control::Guided(54));
        let trigger = cx.debug_bounds("action-Guided(54)").unwrap();
        let menu = cx.debug_bounds("viewer-mode-chooser").unwrap();
        assert_eq!(menu.top(), trigger.bottom() + px(4.));
        assert_eq!(menu.left(), trigger.left());
        record(
            &shell,
            cx,
            &root,
            "view-menu",
            &[("viewer-mode-chooser", Owner::Window)],
            &mut ledger,
        );
        cx.simulate_keystrokes("escape");
        draw(cx);
        activate(&shell, cx, Control::Guided(46));
        record(
            &shell,
            cx,
            &root,
            "zoom-menu",
            &[
                ("zoom-popup", Owner::Window),
                ("zoom-select-button", Owner::Workspace),
            ],
            &mut ledger,
        );
        cx.simulate_keystrokes("escape");
        draw(cx);
        activate(&shell, cx, Control::Phase5(37));
        tab_to(&shell, cx, Control::AnalysisField(9));
        // The first size explicitly saves its note. Use another realistic owner
        // for the second size so its subsequent dirty-draft scene is a real edit,
        // not the already-saved identical text from this shared isolated fixture.
        let owner = if width > 1100. { "Ingrid" } else { "Erik" };
        let note = format!(
            "Headline and CTA are intentional.\n\nProduct grid needs release-owner confirmation.\nCheck delivery copy before release.\nOwner: {owner} · café / 日本語"
        );
        cx.update(|_, cx| cx.write_to_clipboard(gpui::ClipboardItem::new_string(note.clone())));
        cx.simulate_keystrokes("ctrl-a ctrl-v ctrl-home");
        draw(cx);
        shell.read_with(cx, |s, cx| {
            assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), note);
            assert!(!s.control_enabled(Control::Phase5(23), cx));
        });
        record(
            &shell,
            cx,
            &root,
            "multiline-note",
            &[
                ("review-note-input-border", Owner::Inspector),
                ("review-note-status", Owner::Window),
                ("action-Phase5(32)", Owner::Window),
            ],
            &mut ledger,
        );
        cx.simulate_keystrokes("escape");
        draw(cx);
        shell.read_with(cx, |s, cx| {
            assert!(s.review_note_dirty(cx));
            assert!(!s.control_enabled(Control::Phase5(33), cx));
        });
        record(
            &shell,
            cx,
            &root,
            "dirty-note-blocker",
            &[("review-blocking-reason", Owner::Workspace)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase5(37));
        activate(&shell, cx, Control::Phase5(32));
        shell.update(cx, finish_jobs);
        activate(&shell, cx, Control::Phase5(39));
        activate(&shell, cx, Control::Guided(44));
        record(
            &shell,
            cx,
            &root,
            "analysis-sensitivity",
            &[
                ("analysis-current-summary", Owner::Inspector),
                ("analysis-sensitivity-heading", Owner::Inspector),
                ("action-Phase5(1)", Owner::Inspector),
            ],
            &mut ledger,
        );
        activate(&shell, cx, Control::Details(Detail::IgnoredAreas));
        activate(&shell, cx, Control::Phase5(34));
        tab_to(&shell, cx, Control::AnalysisField(10));
        record(
            &shell,
            cx,
            &root,
            "analysis-ignored-coordinates",
            &[("ignored-area-field-0", Owner::Inspector)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase5(39));
        shell.read_with(cx, |s, cx| {
            assert!(s.mask_dirty && !s.control_enabled(Control::Phase5(33), cx))
        });
        record(
            &shell,
            cx,
            &root,
            "dirty-area-blocker",
            &[("review-blocking-reason", Owner::Workspace)],
            &mut ledger,
        );
        // Delete only the newly added draft area, then explicitly Save. Original
        // saved geometry remains pair-bound; no silent draft reset as proof.
        activate(&shell, cx, Control::Guided(44));
        activate(&shell, cx, Control::Phase5(16));
        activate(&shell, cx, Control::Phase5(18));
        shell.update(cx, finish_jobs);
        // Save schedules the real debounced reanalysis. Advance only that
        // already-authorized timer, then consume the real worker replies.
        shell.update(cx, |s, cx| {
            if s.analysis_due.is_some() {
                s.analysis_due = Some(std::time::Instant::now());
                s.poll_analysis(cx);
            }
            while s.analysis_pending.is_some() {
                let update = s.events.recv_timeout(Duration::from_secs(60)).unwrap();
                assert!(!matches!(update.reply, Reply::Failure(_)));
                s.apply_update(update, cx);
            }
            finish_jobs(s, cx);
        });
        activate(&shell, cx, Control::Phase5(3));
        tab_to(&shell, cx, Control::Phase5(28));
        record(
            &shell,
            cx,
            &root,
            "analysis-advanced-tail",
            &[("check-Phase5(28)", Owner::Inspector)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase5(39));
        activate(&shell, cx, Control::Guided(45));
        let selected = shell.read_with(cx, |s, _| s.selected_key.clone());
        for (choice, scene, index) in [
            (Choice::ChangedResults, "filter-changed", 1),
            (Choice::MissingResults, "filter-gaps", 2),
            (Choice::UnchangedResults, "filter-no-significant-change", 3),
            (Choice::AllResults, "filter-grouped", 0),
        ] {
            activate(&shell, cx, Control::Choice(choice));
            if index == 0 {
                activate(&shell, cx, Control::Guided(19));
            }
            shell.read_with(cx, |s, _| {
                assert_eq!(s.result_filter, index);
                assert_eq!(s.selected_key, selected);
                assert!(!s.result_indices().is_empty());
            });
            activate(&shell, cx, Control::CloseOptions);
            record(
                &shell,
                cx,
                &root,
                scene,
                &[
                    ("inspected-target-context", Owner::Workspace),
                    ("viewer-toolbar", Owner::Workspace),
                ],
                &mut ledger,
            );
            activate(&shell, cx, Control::Guided(45));
        }
        activate(&shell, cx, Control::CloseOptions);
        activate(&shell, cx, Control::Phase5(23));
        record(
            &shell,
            cx,
            &root,
            "baseline-selection",
            &[("baseline-selection-label", Owner::Window)],
            &mut ledger,
        );
        if shell.read_with(cx, |s, _| s.navigation_collapsed) {
            activate(&shell, cx, Control::Guided(53));
        }
        activate(&shell, cx, Control::Navigate(Screen::History));
        shell.update(cx, finish_jobs);
        let last = shell.read_with(cx, |s, _| s.history.as_ref().unwrap().runs.len() - 1);
        activate(&shell, cx, Control::HistoryRun(last));
        record(
            &shell,
            cx,
            &root,
            "history-last-run",
            &[(&format!("history-row-HistoryRun({last})"), Owner::Runs)],
            &mut ledger,
        );
        tab_to(&shell, cx, Control::Phase6(19));
        record(
            &shell,
            cx,
            &root,
            "history-selected-actions",
            &[
                ("action-Phase6(19)", Owner::Form),
                ("action-Phase6(5)", Owner::Form),
            ],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(11));
        shell.update(cx, finish_jobs);
        tab_to(&shell, cx, Control::Phase6(12));
        shell.read_with(cx, |s, _| {
            assert_eq!(s.history.as_ref().unwrap().page, 1);
            assert!(!s.history.as_ref().unwrap().more);
        });
        record(
            &shell,
            cx,
            &root,
            "history-page-two",
            &[("action-Phase6(12)", Owner::Form)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Details(Detail::History));
        read_at(&shell, cx, "history-provenance-run");
        record(
            &shell,
            cx,
            &root,
            "history-provenance",
            &[("history-provenance-run", Owner::Form)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(12));
        shell.update(cx, finish_jobs);
        let index = shell.read_with(cx, |s, _| {
            s.history
                .as_ref()
                .unwrap()
                .runs
                .iter()
                .position(|r| r.run.id == check)
                .unwrap()
        });
        activate(&shell, cx, Control::HistoryRun(index));
        activate(&shell, cx, Control::Phase6(5));
        shell.update(cx, finish_jobs);
        read_at(&shell, cx, "deletion-consequences");
        record(
            &shell,
            cx,
            &root,
            "delete-consequences",
            &[("deletion-consequences", Owner::Form)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Details(Detail::Portability));
        read_at(&shell, cx, "portability-exact-delete");
        record(
            &shell,
            cx,
            &root,
            "delete-exact",
            &[("portability-exact-delete", Owner::Form)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(7));
        activate(&shell, cx, Control::Phase6(17));
        activate(&shell, cx, Control::Phase6(2));
        shell.update(cx, finish_jobs);
        read_at(&shell, cx, "export-privacy-consequences");
        record(
            &shell,
            cx,
            &root,
            "export-consequences",
            &[("export-privacy-consequences", Owner::Form)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Details(Detail::Portability));
        read_at(&shell, cx, "portability-exact-export");
        record(
            &shell,
            cx,
            &root,
            "export-exact",
            &[("portability-exact-export", Owner::Form)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(7));
        activate(&shell, cx, Control::Navigate(Screen::Projects));
        shell.update(cx, finish_jobs);
        // Use the production post-picker path with the known isolated archive;
        // native OS file-picker delivery itself remains unqualified.
        shell.update(cx, |s, cx| {
            s.selected_import_path(archive.clone(), cx);
            finish_jobs(s, cx);
            assert!(s.import_preview.is_some());
        });
        tab_to(&shell, cx, Control::Phase6(1));
        record(
            &shell,
            cx,
            &root,
            "import-consequences",
            &[("action-Phase6(1)", Owner::Form)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Details(Detail::Portability));
        read_at(&shell, cx, "portability-exact-import");
        record(
            &shell,
            cx,
            &root,
            "import-exact",
            &[("portability-exact-import", Owner::Form)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(7));
        activate(&shell, cx, Control::Navigate(Screen::Settings));
        shell.update(cx, finish_jobs);
        tab_to(&shell, cx, Control::Details(Detail::Network));
        record(
            &shell,
            cx,
            &root,
            "network-limits-closed",
            &[("action-Details(Network)", Owner::Form)],
            &mut ledger,
        );
        activate(&shell, cx, Control::Details(Detail::Network));
        read_at(&shell, cx, "network-limitations-copy");
        record(
            &shell,
            cx,
            &root,
            "network-limits-open",
            &[("network-limitations-copy", Owner::Form)],
            &mut ledger,
        );
        owned.join_after_ui();
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        Some(baseline),
        store.load_project(project).unwrap().project.active_baseline
    );
    assert_eq!(
        store.list_projects(0).unwrap().items.len(),
        4,
        "preview must not import a project"
    );
    assert!(
        store.load_run(check).is_ok(),
        "preview must not delete the selected run"
    );
    for (id, hash) in &originals {
        assert_eq!(
            *hash,
            hex::encode(Sha256::digest(
                fs::read(store.artifact_path(id).unwrap()).unwrap()
            ))
        );
    }
    fs::write(root.join(if preflight{"phase5-preflight-matrix.json"}else{"phase5-matrix.json"}),serde_json::to_vec_pretty(&serde_json::json!({"frames":ledger,"originals_unchanged":originals,"baseline_unchanged":true,"no_import_or_deletion":true,"synthetic_archive_constructed_locally":true,"preflight_only":preflight})).unwrap()).unwrap();
}
