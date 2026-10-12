// SPDX-License-Identifier: MPL-2.0
//! Whole-app critique evidence. Test-only; every database and pixel is synthetic.
use super::*;
use regressee_core::domain::*;
use regressee_core::evidence::*;
use std::sync::atomic::AtomicBool;

fn fixture(root: &Path, path: &Path) -> (regressee_storage::Store, ProjectId, RunId) {
    let mut store = regressee_storage::Store::open(AppPaths::isolated(path)).unwrap();
    let mut project = Project::draft(
        "Northline · website release".into(),
        "https://northline.example/",
    )
    .unwrap();
    project.discovery.mode = DiscoveryMode::AddressesAndLinks;
    project.profiles.push(CaptureProfile::mobile());
    for profile in &mut project.profiles {
        profile.runtime = RuntimeIdentity::Pinned {
            node: "synthetic".into(),
            worker: "synthetic".into(),
            playwright: "synthetic".into(),
            browser: "synthetic".into(),
        };
    }
    project.environments[0].name = "Production".into();
    store.save_project(&project, None).unwrap();
    let inventory = InventoryRevision {
        id: InventoryId::new(),
        project: project.id,
        identity_version: 2,
        pages: [
            "/",
            "/collections/autumn/",
            "/products/field-jacket/",
            "/journal/materials-and-care/",
            "/stores/bergen/",
            "/account/orders/",
        ]
        .into_iter()
        .map(|path| InventoryPage {
            id: PageId::new(),
            canonical: project.sources[0].join(path).unwrap(),
            included: true,
            reason: None,
        })
        .collect(),
        coverage: DiscoveryCoverage::Complete,
    };
    store.freeze_inventory(&inventory).unwrap();
    let cancel = AtomicBool::new(false);
    let mut images = Vec::new();
    for name in [
        "desktop-before",
        "mobile-before",
        "desktop-after",
        "mobile-after",
    ] {
        images.push(
            regressee_app::images::ingest(
                &mut store,
                &root.join(format!("website-fixtures/{name}.png")),
                ArtifactKind::Screenshot,
                &cancel,
            )
            .unwrap(),
        );
    }
    let mut baseline = None;
    let mut latest = None;
    for sequence in 0..3 {
        let mut run = RunRecord {
            run: CaptureRun {
                id: RunId::new(),
                project: project.id,
                intent: baseline.map_or(RunIntent::NewBaseline, |baseline| RunIntent::Check {
                    baseline,
                }),
                inventory: inventory.clone(),
                profiles: project.profiles.clone(),
                environment: project.environments[0].clone(),
                rendering: RenderingEnvironment {
                    os: "synthetic-linux".into(),
                    architecture: "x86_64".into(),
                    browser: "synthetic".into(),
                    worker_version: "synthetic".into(),
                    profile_version: 1,
                    fonts_fingerprint: Some("synthetic".into()),
                },
                state: JobState::Running,
                started_at: Some(1_791_792_000 + sequence * 86_400),
                finished_at: None,
                targets: inventory
                    .pages
                    .iter()
                    .flat_map(|page| {
                        project.profiles.iter().map(move |profile| CaptureTarget {
                            key: TargetKey {
                                page: page.id,
                                profile: profile.id,
                            },
                            requested: page.canonical.clone(),
                            final_url: None,
                            attempts: 1,
                            outcome: TargetOutcome::Pending,
                            warnings: vec![],
                        })
                    })
                    .collect(),
            },
            capture: None,
            captures: vec![],
            queue: None,
        };
        store.begin_run(&run).unwrap();
        for target in &mut run.run.targets {
            let profile = project
                .profiles
                .iter()
                .find(|p| p.id == target.key.profile)
                .unwrap();
            // Representative imagery is intentionally reused, never presented as live site data.
            let changed = sequence == 2 && !target.requested.path().starts_with("/stores");
            let image =
                images[usize::from(profile.name == "Mobile") + if changed { 2 } else { 0 }].clone();
            target.outcome = TargetOutcome::Captured {
                artifact: image.original.clone(),
                width: image.width,
                height: image.height,
            };
            target.final_url = Some(target.requested.clone());
            run.captures.push(CaptureEvidence {
                metadata: CaptureMetadata {
                    version: 1,
                    key: target.key.clone(),
                    requested: target.requested.to_string(),
                    final_url: target.requested.to_string(),
                    runtime: profile.runtime.clone(),
                    profile: profile.clone(),
                    width: image.width,
                    height: image.height,
                    sha256: image.original.clone(),
                    bytes: store.verify_artifact(&image.original).unwrap().bytes,
                    warnings: vec![],
                    stable: true,
                    duration_ms: 1460,
                    readiness: Readiness {
                        font_failures: 0,
                        image_failures: 0,
                        resource_failures: 0,
                    },
                    capture_mode: "chromium-new-headless".into(),
                    replacement: None,
                },
                image,
            });
        }
        run.run.state = JobState::Completed;
        run.run.finished_at = run.run.started_at.map(|time| time + 180);
        store.finish_run(&run).unwrap();
        if sequence == 0 {
            baseline = store
                .activate_baseline(run.run.id, 1)
                .unwrap()
                .project
                .active_baseline;
        } else {
            regressee_app::review::analyze_run(
                &mut store,
                run.run.id,
                regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
                &cancel,
            )
            .unwrap();
        }
        latest = Some(run.run.id);
    }
    for (name, address) in [
        ("Harbor · customer portal", "https://portal.harbor.example/"),
        (
            "City services · accessibility update",
            "https://services.city.example/",
        ),
        (
            "Northline · campaign microsite",
            "https://campaign.northline.example/",
        ),
    ] {
        store
            .save_project(&Project::draft(name.into(), address).unwrap(), None)
            .unwrap();
    }
    (store, project.id, latest.unwrap())
}

fn capture_only(s: &mut Shell, store: &regressee_storage::Store, state: &str) {
    let baseline = s.evidence.baseline.clone().unwrap();
    let record = store.load_run(baseline.references[0].run).unwrap();
    let mut tiles = Vec::new();
    let image = &record.captures[0].image;
    for id in image.tiles.iter().take(3) {
        tiles.push(regressee_app::images::viewer_tile(store, id).unwrap());
    }
    s.evidence = ProjectEvidence {
        latest: Some(record),
        ..Default::default()
    };
    s.focus.baseline = state == "saved";
    s.focus.candidate = state == "ready";
    s.focus.viewer = true;
    s.selected_key = Some(
        s.evidence.latest.as_ref().unwrap().run.targets[0]
            .key
            .clone(),
    );
    let image = &s.evidence.latest.as_ref().unwrap().captures[0].image;
    s.viewer = Some(regressee_app::jobs::ViewerFrame {
        tile: 0,
        panes: vec![vec![], tiles, vec![]],
        width: image.width,
        height: image.height,
        request: None,
        analysis: None,
        identity: None,
        before_environment: None,
        before_capture: None,
    });
    if state == "saved" {
        s.evidence.baseline = Some(baseline);
    } else {
        s.project.as_mut().unwrap().project.active_baseline = None;
    }
    if state == "gaps" || state == "failed" {
        let run = &mut s.evidence.latest.as_mut().unwrap().run;
        run.state = JobState::CompletedWithErrors;
        run.targets.last_mut().unwrap().outcome = TargetOutcome::Failed {
            code: CaptureError::Timeout,
        };
        if state == "failed" {
            s.selected_key = Some(run.targets.last().unwrap().key.clone());
            s.viewer.as_mut().unwrap().panes[1].clear();
        }
    }
}

fn discovery(s: &mut Shell) {
    let mut inventory = s.evidence.latest.as_ref().unwrap().run.inventory.clone();
    for i in 6..74 {
        inventory.pages.push(InventoryPage {
            id: PageId::new(),
            canonical: format!("https://northline.example/journal/seasonal-field-notes-{i}/")
                .parse()
                .unwrap(),
            included: i % 11 != 0,
            reason: (i % 11 == 0).then(|| "Skipped by address rule".into()),
        });
    }
    inventory.coverage = DiscoveryCoverage::Limited {
        reason: "Page limit reached".into(),
    };
    s.selected_page = Some(inventory.pages[0].id);
    s.preview = Some(DiscoveryPreview {
        project_revision: s.project.as_ref().unwrap().revision,
        settings: s.project.as_ref().unwrap().project.clone(),
        inventory,
        aliases: vec![],
        notes: vec!["Page limit reached. Increase Maximum pages to find more pages.".into()],
        visited: 74,
    });
    s.focus.preview = true;
    s.focus.review_surface = ReviewSurface::Pages;
}

fn mixed_queue(s: &mut Shell) {
    // Scale-only presentation clones, not eligible approvals or saved site evidence.
    let run = s.evidence.latest.as_mut().unwrap();
    let result = s.evidence.comparison.as_ref().unwrap().revision.results[0].clone();
    let templates = run.run.targets[..2].to_vec();
    let captures = run.captures[..2].to_vec();
    for page_index in 6..74 {
        let page = InventoryPage {
            id: PageId::new(),
            canonical: format!("https://northline.example/journal/field-notes-{page_index}/")
                .parse()
                .unwrap(),
            included: true,
            reason: None,
        };
        for (profile_index, template) in templates.iter().enumerate() {
            let mut target = template.clone();
            target.key.page = page.id;
            target.requested = page.canonical.clone();
            target.final_url = Some(page.canonical.clone());
            let mut classified = result.clone();
            classified.key = target.key.clone();
            if page_index == 6 {
                target.outcome = TargetOutcome::Failed {
                    code: if profile_index == 0 {
                        CaptureError::Timeout
                    } else {
                        CaptureError::NeedsLogin
                    },
                };
                target.final_url = None;
                classified.classification = Classification::CouldNotCapture;
                classified.eligible_pixels = 0;
                classified.changed_pixels = 0;
            } else {
                let mut capture = captures[profile_index].clone();
                capture.metadata.key = target.key.clone();
                capture.metadata.requested = target.requested.to_string();
                capture.metadata.final_url = target.requested.to_string();
                run.captures.push(capture);
                if page_index > 10 {
                    classified.classification = Classification::NoSignificantChange;
                    classified.changed_pixels = 0;
                }
            }
            s.evidence
                .comparison
                .as_mut()
                .unwrap()
                .revision
                .results
                .push(classified);
            run.run.targets.push(target);
        }
        run.run.inventory.pages.push(page);
    }
    run.run.state = JobState::CompletedWithErrors;
    run.run.inventory.coverage = DiscoveryCoverage::Limited {
        reason: "Page limit reached".into(),
    };
    *s.local_index.borrow_mut() = Default::default();
}

#[gpui::test]
#[ignore = "explicit whole-app screenshot critique; isolated synthetic data only"]
fn renderer_capture_whole_app(cx: &mut TestAppContext) {
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
    let dir = tempfile::Builder::new()
        .prefix("whole-app-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, check) = fixture(&root, dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let before_project = serde_json::to_value(store.load_project(project).unwrap()).unwrap();
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
    let import = store
        .inspect_archive(&archive, &AtomicBool::new(false))
        .unwrap()
        .preview;
    let delete = store
        .deletion_preview(project, regressee_storage::DeleteItem::Run(check))
        .unwrap();
    let scenes = [
        "projects",
        "projects-empty",
        "projects-loading",
        "projects-failed",
        "project-chooser",
        "new-website",
        "website",
        "website-validation",
        "access-public",
        "access-login",
        "access-login-actions",
        "access-options",
        "mapping-preview",
        "timezone-popup",
        "pages-sizes",
        "pages-screen-sizes",
        "pages-images",
        "pages-speed",
        "pages-advanced",
        "project-settings",
        "project-settings-bottom",
        "pages-baseline",
        "discovery",
        "discovery-options",
        "discovery-no-matches",
        "review",
        "review-mobile",
        "review-mixed-148",
        "review-failed-target",
        "review-queue-hidden",
        "review-difference",
        "review-overlay",
        "review-heatmap",
        "review-details",
        "review-diagnostics",
        "review-analysis",
        "review-analysis-advanced",
        "review-ignored-area",
        "review-note",
        "review-filters",
        "review-no-matches",
        "review-approval-selection",
        "review-approval-confirm",
        "review-zoom-menu",
        "capture-gaps",
        "capture-failed",
        "capture-ready",
        "capture-mobile",
        "capture-saved",
        "capture-empty",
        "capture-loading",
        "capture-read-failed",
        "history-runs",
        "history-bottom",
        "history-baselines",
        "history-storage",
        "history-empty",
        "history-no-matches",
        "history-recorded-check",
        "history-recorded-capture",
        "history-provenance",
        "export-preview",
        "import-preview",
        "delete-preview",
        "app-settings",
        "app-settings-network",
        "app-settings-diagnostics",
        "diagnostic-preview",
        "integrity-preview",
        "keyboard-help",
        "about",
    ];
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for label in scenes {
            let name = format!("all-{width:.0}x{height:.0}-{label}");
            if std::env::var_os("REGRESSEE_RENDERER_COMPLETE_BATCH").as_deref()
                == Some(std::ffi::OsStr::new("1"))
                && root.join(format!("{name}.png")).is_file()
            {
                let metadata: serde_json::Value =
                    serde_json::from_slice(&fs::read(root.join(format!("{name}.json"))).unwrap())
                        .unwrap();
                assert_eq!(
                    metadata["png_sha256"],
                    hex::encode(Sha256::digest(
                        fs::read(root.join(format!("{name}.png"))).unwrap()
                    ))
                );
                manifest.push(serde_json::json!({"capture":name,"state":label,"width":width,"height":height,"synthetic":true,"retained_from_same_batch":true}));
                continue;
            }
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            let screen = if label.starts_with("projects") {
                Screen::Projects
            } else if label.starts_with("history")
                || matches!(label, "export-preview" | "delete-preview")
            {
                Screen::History
            } else if label.starts_with("app-settings")
                || matches!(label, "diagnostic-preview" | "integrity-preview" | "about")
            {
                Screen::Settings
            } else if label.starts_with("access")
                || label.starts_with("website")
                || label.starts_with("pages-") && label != "pages-baseline"
                || label.starts_with("project-settings")
                || matches!(label, "new-website" | "mapping-preview" | "timezone-popup")
            {
                Screen::Setup
            } else if label == "import-preview" {
                Screen::Projects
            } else {
                Screen::Results
            };
            cx.update(|window, cx| shell.update(cx, |s, cx| {
                finish_jobs(s, cx);
                let index = s.projects.items.iter().position(|p| p.id == project).unwrap();
                s.activate_control(Control::SelectProject(index), window, cx);
                finish_jobs(s, cx);
                s.navigate(screen, window, cx);
                finish_jobs(s, cx);
                s.notice = None; s.error = None;
                s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                s.configure_review_geometry(width);
                if screen == Screen::Setup {
                    s.focus.setup_stage = if label.starts_with("access") || matches!(label, "mapping-preview" | "timezone-popup") { SetupStage::Access }
                        else if label.starts_with("website") || label == "new-website" { SetupStage::Website }
                        else if label.starts_with("pages-") { SetupStage::Pages } else { SetupStage::Settings };
                    s.focus.advanced = matches!(label, "pages-advanced" | "access-options" | "mapping-preview" | "timezone-popup");
                }
                match label {
                    "projects-empty" => { s.projects.items.clear(); s.project = None; s.projects_dirty = false; },
                    "projects-loading" | "projects-failed" => {
                        s.projects.items.clear(); s.project = None; s.projects_dirty = true;
                        if label == "projects-loading" { s.pending = Some(u64::MAX); s.focus.busy = true; }
                        else { s.error = Some("Saved projects could not be read. Check the data folder permissions.".into()); }
                    },
                    "project-chooser" => { s.activate_control(Control::ChooseProject, window, cx); finish_jobs(s, cx); },
                    "new-website" => { s.activate_control(Control::NewProject, window, cx); },
                    "website-validation" => { s.address.update(cx, |f, cx| f.set_value("northline.example", cx)); s.activate_control(Control::Save, window, cx); },
                    "access-login" | "access-login-actions" => { s.choose(Choice::LoginRequired, cx); },
                    "mapping-preview" => { s.show_mapping = true; s.environment.as_mut().unwrap().mapped = (0..40).map(|i| (format!("https://northline.example/products/item-{i}/"), format!("https://staging.northline.example/products/item-{i}/"))).collect(); },
                    "pages-baseline" => { s.focus.review_surface = ReviewSurface::Pages; },
                    "discovery" | "discovery-options" | "discovery-no-matches" => { discovery(s); s.result_filters_open = label != "discovery"; if label.ends_with("no-matches") { s.crawl_fields[9].update(cx, |f, cx| f.set_value("nothing-matches", cx)); } },
                    "review-mobile" => { let key = s.evidence.latest.as_ref().unwrap().run.targets[1].key.clone(); s.selected_key = Some(key); s.reload_viewer(cx); finish_jobs(s, cx); },
                    "review-mixed-148" | "review-failed-target" => { mixed_queue(s); if label == "review-failed-target" { s.selected_key = Some(s.evidence.latest.as_ref().unwrap().run.targets.iter().find(|t| matches!(t.outcome, TargetOutcome::Failed { .. })).unwrap().key.clone()); s.clear_viewer(); s.focus.viewer = false; } },
                    "review-queue-hidden" => { s.activate_control(Control::Guided(50), window, cx); },
                    "review-difference" | "review-overlay" | "review-heatmap" => { s.activate_control(Control::Phase5(match label { "review-difference" => 7, "review-overlay" => 8, _ => 9 }), window, cx); finish_jobs(s, cx); },
                    "review-details" | "review-diagnostics" => { s.activate_control(Control::Phase5(36), window, cx); s.capture_diagnostics_open = label == "review-diagnostics"; },
                    "review-analysis" | "review-analysis-advanced" | "review-ignored-area" => { s.activate_control(Control::Guided(44), window, cx); s.analysis_advanced = label == "review-analysis-advanced"; },
                    "review-note" => { s.activate_control(Control::Phase5(37), window, cx); s.analysis_fields[9].update(cx, |f, cx| f.set_value("Autumn headline and CTA are intentional. Product grid needs release-owner confirmation.", cx)); },
                    "review-filters" | "review-no-matches" => { s.result_filters_open = true; if label.ends_with("no-matches") { s.crawl_fields[9].update(cx, |f, cx| f.set_value("nothing-matches", cx)); } },
                    "review-approval-selection" | "review-approval-confirm" => { s.activate_control(Control::Phase5(23), window, cx); if label.ends_with("confirm") { s.activate_control(Control::Phase5(24), window, cx); finish_jobs(s, cx); assert!(s.approval.is_some()); } },
                    "capture-gaps" | "capture-failed" | "capture-ready" | "capture-saved" => { capture_only(s, &store, label.strip_prefix("capture-").unwrap()); },
                    "capture-mobile" => {
                        capture_only(s, &store, "ready");
                        let record = s.evidence.latest.as_ref().unwrap();
                        let image = &record.captures[1].image;
                        s.selected_key = Some(record.run.targets[1].key.clone());
                        let tiles = image.tiles.iter().take(3).map(|id| regressee_app::images::viewer_tile(&store, id).unwrap()).collect();
                        let frame = s.viewer.as_mut().unwrap(); frame.width = image.width; frame.height = image.height; frame.panes[1] = tiles;
                    },
                    "capture-empty" | "capture-loading" | "capture-read-failed" => {
                        s.evidence = ProjectEvidence::default(); s.clear_viewer(); s.focus.viewer = false; s.focus.candidate = false; s.focus.baseline = false;
                        s.evidence_dirty = label != "capture-empty";
                        if label == "capture-loading" { s.pending = Some(u64::MAX); s.focus.busy = true; }
                        else if label == "capture-read-failed" { s.evidence_read_failed = true; s.error = Some("The saved screenshots could not be read. Check the project data folder, then refresh.".into()); }
                    },
                    "history-baselines" | "export-preview" => { s.history_section = 1; if label == "export-preview" { s.export_preview = Some(export.clone()); } },
                    "history-storage" => { s.history_section = 2; },
                    "history-empty" => { s.history.as_mut().unwrap().runs.clear(); s.evidence.baselines.clear(); },
                    "history-no-matches" => { s.history_search.update(cx, |f, cx| f.set_value("nothing-matches", cx)); },
                    "history-provenance" => { s.details.insert(Detail::History); },
                    "history-recorded-check" | "history-recorded-capture" => { let index = s.history.as_ref().unwrap().runs.iter().position(|r| if label.ends_with("check") { matches!(r.run.intent, RunIntent::Check { .. }) } else { r.run.intent == RunIntent::NewBaseline }).unwrap(); s.activate_control(Control::HistoryRun(index), window, cx); s.activate_control(Control::Phase6(19), window, cx); finish_jobs(s, cx); },
                    "delete-preview" => { s.deletion_preview = Some(delete.clone()); },
                    "import-preview" => { s.import_preview = Some(import.clone()); s.import_path = Some(archive.clone()); },
                    "diagnostic-preview" => { s.activate_control(Control::Phase7(0), window, cx); finish_jobs(s, cx); },
                    "integrity-preview" => { s.activate_control(Control::Phase6(13), window, cx); finish_jobs(s, cx); },
                    "keyboard-help" => { s.show_keyboard_help(window, cx); },
                    "about" => { s.activate_control(Control::About, window, cx); },
                    _ => (),
                }
                s.configure_review_geometry(width);
                cx.notify();
            }));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
                window.capture_offscreen_frame().unwrap();
            });
            if label == "review-ignored-area" {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        s.activate_control(Control::Phase5(34), window, cx)
                    })
                });
                cx.run_until_parked();
            }
            let anchor = match label {
                "access-login-actions" => Some("action-Guided(31)"),
                "access-options" => Some("field-Source origin or section base"),
                "mapping-preview" => Some("mapping-preview"),
                "timezone-popup" => Some("field-Capture timezone"),
                "pages-screen-sizes" => Some("section-Screen sizes"),
                "pages-images" => Some("section-Images"),
                "pages-speed" => Some("section-Capture speed"),
                "pages-advanced" => Some("field-Links away from starting addresses"),
                "app-settings-network" => Some("managed-network-section"),
                "app-settings-diagnostics" | "diagnostic-preview" => {
                    Some("section-Private diagnostics")
                }
                "history-provenance" => Some("action-Details(History)"),
                "export-preview" => Some("action-Phase6(3)"),
                "delete-preview" => Some("action-Phase6(6)"),
                "integrity-preview" => Some("action-Phase6(14)"),
                _ => None,
            };
            if let Some(anchor) = anchor {
                let bounds = cx
                    .debug_bounds(anchor)
                    .expect("review section must be rendered");
                shell.update(cx, |s, cx| {
                    let context = if matches!(
                        label,
                        "export-preview" | "delete-preview" | "integrity-preview"
                    ) {
                        180.
                    } else {
                        12.
                    };
                    let delta = s.screen_scroll.bounds().top() + px(context) - bounds.top();
                    let old = s.screen_scroll.offset();
                    s.screen_scroll
                        .set_offset(gpui::point(old.x, (old.y + delta).min(px(0.))));
                    cx.notify();
                });
            }
            if matches!(label, "project-settings-bottom" | "history-bottom") {
                shell.update(cx, |s, cx| {
                    s.screen_scroll
                        .set_offset(gpui::point(px(0.), -s.screen_scroll.max_offset().height));
                    cx.notify();
                });
            }
            if label == "timezone-popup" {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        window.focus(&s.handle(Control::CrawlField(16), cx));
                        window.dispatch_action(Box::new(timezone_select::Toggle), cx);
                    })
                });
            }
            if label == "review-zoom-menu" {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        s.activate_control(Control::Guided(46), window, cx)
                    })
                });
            }
            capture(cx, &root, &name).unwrap();
            let geometry: Vec<_> = ["review-queue", "viewer-toolbar", "phase5-before", "phase5-after", "ordinary-review-decisions", "approval-next-action", "page-list", "history-run-list", "history-baseline-list", "selected-project-context"].into_iter().filter_map(|selector| cx.debug_bounds(selector).map(|b| serde_json::json!({"selector":selector,"x":f32::from(b.left()),"y":f32::from(b.top()),"width":f32::from(b.size.width),"height":f32::from(b.size.height)}))).collect();
            manifest.push(serde_json::json!({"capture":name,"state":label,"width":width,"height":height,"synthetic":true,"website_visited":false,"os_input_verified":false,"geometry":geometry}));
            owned.join_after_ui();
        }
    }
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    assert_eq!(
        before_project,
        serde_json::to_value(store.load_project(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("whole-app-matrix.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
