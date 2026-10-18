// SPDX-License-Identifier: MPL-2.0
//! Whole-app critique evidence. Test-only; every database and pixel is synthetic.
use super::*;
use regressee_core::domain::*;
use regressee_core::evidence::*;
use std::sync::atomic::AtomicBool;

fn fixture(root: &Path, path: &Path) -> (regressee_storage::Store, ProjectId, RunId) {
    fixture_named(root, path, false)
}

pub(super) fn fixture_named(
    root: &Path,
    path: &Path,
    recorded_names: bool,
) -> (regressee_storage::Store, ProjectId, RunId) {
    fixture_with_capture_warnings(root, path, recorded_names, &[])
}

pub(super) fn fixture_with_capture_warnings(
    root: &Path,
    path: &Path,
    recorded_names: bool,
    warnings: &[&str],
) -> (regressee_storage::Store, ProjectId, RunId) {
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
            "/account/orders/customer-service/very-long-retained-page-address-with-release-notes-and-delivery-preferences/?campaign=synthetic-phase4-containment&locale=nb-NO&return=%2Faccount%2Forders%2F",
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
                capture_plan: Default::default(),
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
                            dom_outcome: Default::default(),
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
            dom_captures: vec![],
            captures: vec![],
            queue: None,
        };
        if recorded_names {
            run.run.environment.name = if sequence == 0 {
                "Recorded reference · production release before the environment was renamed"
            } else {
                "Recorded check · staging release with a long historical environment name"
            }
            .into();
        }
        store.begin_run(&run).unwrap();
        for target in &mut run.run.targets {
            if std::env::var_os("REGRESSEE_RENDERER_PHASE6").is_some()
                && sequence == 2
                && target.requested.path().starts_with("/account")
            {
                target.outcome = TargetOutcome::Failed {
                    code: CaptureError::NeedsLogin,
                };
                continue;
            }
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
                    warnings: warnings.iter().map(|w| (*w).to_owned()).collect(),
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
        run.run.state = if run
            .run
            .targets
            .iter()
            .any(|t| matches!(t.outcome, TargetOutcome::Failed { .. }))
        {
            JobState::CompletedWithErrors
        } else {
            JobState::Completed
        };
        run.run.finished_at = run.run.started_at.map(|time| time + 180);
        store.finish_run(&run).unwrap();
        if sequence == 0 {
            baseline = store
                .activate_baseline(run.run.id, 1)
                .unwrap()
                .project
                .active_baseline;
            if std::env::var_os("REGRESSEE_RENDERER_PHASE6").is_some() {
                for version in 1..8 {
                    let mut retained = run.clone();
                    retained.run.id = RunId::new();
                    retained.run.started_at = Some(1_790_000_000 + version * 3600);
                    retained.run.finished_at = None;
                    retained.run.state = JobState::Running;
                    let captures = std::mem::take(&mut retained.captures);
                    for target in &mut retained.run.targets {
                        target.outcome = TargetOutcome::Pending;
                    }
                    store.begin_run(&retained).unwrap();
                    retained.captures = captures;
                    for target in &mut retained.run.targets {
                        let image = &retained
                            .captures
                            .iter()
                            .find(|c| c.metadata.key == target.key)
                            .unwrap()
                            .image;
                        target.outcome = TargetOutcome::Captured {
                            artifact: image.original.clone(),
                            width: image.width,
                            height: image.height,
                        };
                    }
                    retained.run.state = JobState::Completed;
                    retained.run.finished_at = retained.run.started_at.map(|t| t + 180);
                    store.finish_run(&retained).unwrap();
                    baseline = store
                        .activate_baseline(
                            retained.run.id,
                            store.load_project(project.id).unwrap().revision,
                        )
                        .unwrap()
                        .project
                        .active_baseline;
                }
            }
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

pub(super) fn capture_only(s: &mut Shell, store: &regressee_storage::Store, state: &str) {
    let baseline = s.evidence.baseline.clone().unwrap();
    let record = store.load_run(baseline.references[0].run).unwrap();
    let mut tiles = Vec::new();
    let image = &record.captures[0].image;
    for id in image.tiles.iter().take(3) {
        tiles.push(regressee_gui::viewer_images::viewer_tile(store, id).unwrap());
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
    s.viewer = Some(regressee_gui::jobs::ViewerFrame {
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
        zoom_select::bind(cx);
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
        "access-saved-private",
        "access-browser-open",
        "access-ready",
        "access-status-loading",
        "access-invalid",
        "access-options",
        "mapping-preview",
        "timezone-popup",
        "pages-sizes",
        "pages-screen-sizes",
        "pages-images",
        "pages-speed",
        "pages-advanced",
        "pages-advanced-tail",
        "pages-invalid",
        "pages-mobile-revealed",
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
        "review-diagnostics-tail",
        "review-analysis",
        "review-analysis-advanced",
        "review-analysis-field-0",
        "review-analysis-field-1",
        "review-analysis-field-2",
        "review-analysis-field-3",
        "review-analysis-field-4",
        "review-analysis-field-5",
        "review-analysis-field-6",
        "review-analysis-field-7",
        "review-analysis-field-8",
        "review-analysis-switches-tail",
        "review-ignored-area",
        "review-note",
        "review-filters",
        "review-no-matches",
        "review-approval-selection",
        "review-approval-confirm",
        "review-zoom-menu",
        "review-full-width-before",
        "review-full-width-after",
        "review-mode-chooser",
        "review-navigation-open",
        "review-approval-many",
        "review-approval-last-exact",
        "review-approval-last-row",
        "review-approval-exact-tail",
        "capture-gaps",
        "capture-failed",
        "capture-ready",
        "capture-mobile",
        "capture-saved",
        "capture-empty",
        "capture-loading",
        "capture-read-failed",
        "capture-read-dismissed",
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
    let previous: Vec<serde_json::Value> = fs::read(root.join("whole-app-matrix.json"))
        .ok()
        .map(|bytes| serde_json::from_slice(&bytes).unwrap())
        .unwrap_or_default();
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for label in scenes {
            let clarity_pass = std::env::var_os("REGRESSEE_RENDERER_CLARITY").is_some();
            if clarity_pass
                && !matches!(
                    label,
                    "access-options"
                        | "pages-images"
                        | "pages-advanced-tail"
                        | "review-analysis-advanced"
                        | "review-analysis-switches-tail"
                        | "app-settings-network"
                        | "diagnostic-preview"
                        | "about"
                )
            {
                continue;
            }
            // Finite wording inspection at both desktop sizes. Keep earlier
            // qualification batches and their scene selection unchanged.
            let copy_pass = std::env::var_os("REGRESSEE_RENDERER_COPY").is_some();
            if copy_pass
                && !matches!(
                    label,
                    "access-login"
                        | "mapping-preview"
                        | "discovery-options"
                        | "review-analysis"
                        | "review-filters"
                        | "review-approval-confirm"
                        | "capture-saved"
                        | "capture-gaps"
                        | "history-baselines"
                        | "history-storage"
                        | "export-preview"
                        | "import-preview"
                        | "delete-preview"
                        | "integrity-preview"
                )
            {
                continue;
            }
            let phase7 = std::env::var_os("REGRESSEE_RENDERER_PHASE7").is_some();
            if phase7
                && !(label == "review-diagnostics-tail"
                    || label.starts_with("review-analysis-field-")
                    || label == "review-analysis-switches-tail")
            {
                continue;
            }
            // New supplemental scenes never silently expand earlier batches.
            if !phase7
                && !clarity_pass
                && (label == "review-diagnostics-tail"
                    || label.starts_with("review-analysis-field-")
                    || label == "review-analysis-switches-tail")
            {
                continue;
            }
            let phase6 = std::env::var_os("REGRESSEE_RENDERER_PHASE6").is_some();
            if phase6
                && !matches!(
                    label,
                    "history-runs"
                        | "history-bottom"
                        | "history-baselines"
                        | "history-storage"
                        | "history-no-matches"
                        | "history-provenance"
                        | "history-recorded-check"
                        | "history-recorded-capture"
                        | "export-preview"
                        | "import-preview"
                        | "delete-preview"
                        | "integrity-preview"
                        | "review-failed-target"
                        | "capture-empty"
                )
            {
                continue;
            }
            let phase5_repair = std::env::var_os("REGRESSEE_RENDERER_PHASE5_REPAIR").is_some();
            let phase5 = std::env::var_os("REGRESSEE_RENDERER_PHASE5").is_some() || phase5_repair;
            if phase5_repair
                && !matches!(
                    label,
                    "access-invalid" | "pages-invalid" | "pages-mobile-revealed"
                )
            {
                continue;
            }
            if phase5
                && !matches!(
                    label,
                    "new-website"
                        | "website-validation"
                        | "access-public"
                        | "access-login"
                        | "access-saved-private"
                        | "access-browser-open"
                        | "access-ready"
                        | "access-status-loading"
                        | "access-invalid"
                        | "mapping-preview"
                        | "pages-sizes"
                        | "pages-images"
                        | "pages-speed"
                        | "pages-advanced"
                        | "pages-advanced-tail"
                        | "pages-invalid"
                        | "pages-mobile-revealed"
                        | "project-settings"
                        | "project-settings-bottom"
                        | "discovery"
                        | "discovery-options"
                )
            {
                continue;
            }
            let repair = std::env::var_os("REGRESSEE_RENDERER_PHASE4_REPAIR").is_some();
            if repair
                && !matches!(
                    label,
                    "review-note"
                        | "review-queue-hidden"
                        | "review-difference"
                        | "review-overlay"
                        | "review-heatmap"
                        | "review-full-width-before"
                        | "review-full-width-after"
                        | "capture-mobile"
                        | "capture-loading"
                        | "capture-read-failed"
                        | "capture-read-dismissed"
                )
            {
                continue;
            }
            if std::env::var_os("REGRESSEE_RENDERER_PHASE4").is_some()
                && !matches!(
                    label,
                    "review"
                        | "review-queue-hidden"
                        | "review-difference"
                        | "review-overlay"
                        | "review-heatmap"
                        | "review-note"
                        | "review-details"
                        | "review-no-matches"
                        | "review-approval-confirm"
                        | "review-full-width-before"
                        | "review-full-width-after"
                        | "review-mode-chooser"
                        | "review-navigation-open"
                        | "review-approval-many"
                        | "review-approval-last-exact"
                        | "review-approval-last-row"
                        | "review-approval-exact-tail"
                        | "capture-mobile"
                        | "capture-empty"
                )
            {
                continue;
            }
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
                let mut entry = previous.iter().find(|entry| entry["capture"] == name)
                    .cloned().unwrap_or_else(|| serde_json::json!({"capture":name,"state":label,"width":width,"height":height,"synthetic":true}));
                entry["retained_from_same_batch"] = true.into();
                manifest.push(entry);
                continue;
            }
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            let screen = if label.starts_with("projects")
                || label == "new-website"
                || (copy_pass && label == "import-preview")
            {
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
                    s.focus.advanced = matches!(label, "pages-advanced" | "pages-advanced-tail" | "access-options" | "mapping-preview" | "timezone-popup");
                }
                if clarity_pass {
                    match label {
                        "access-options" => s.activate_control(Control::EnvironmentOptions, window, cx),
                        "app-settings-network" => {
                            s.activate_control(Control::Details(Detail::ManagedNetwork), window, cx);
                            s.activate_control(Control::Details(Detail::Network), window, cx);
                        },
                        "diagnostic-preview" => s.activate_control(Control::Details(Detail::SupportTools), window, cx),
                        _ => (),
                    }
                }
                match label {
                    "projects-empty" => { s.projects.items.clear(); s.project = None; s.projects_dirty = false; },
                    "projects-loading" | "projects-failed" => {
                        s.projects.items.clear(); s.project = None; s.projects_dirty = true;
                        if label == "projects-loading" { s.pending = Some(u64::MAX); s.focus.busy = true; }
                        else { s.error = Some("Saved projects could not be read. Check the data folder permissions.".into()); }
                    },
                    "project-chooser" => { s.activate_control(Control::ChooseProject, window, cx); finish_jobs(s, cx); },
                    "new-website" => {
                        assert!(s.control_mounted(Control::NewProject));
                        s.activate_control(Control::NewProject, window, cx);
                        assert_eq!(s.focus.screen, Screen::Setup);
                        assert_eq!(s.focus.setup_stage, SetupStage::Website);
                        assert!(s.project.is_none());
                        assert!(!s.focus.project_saved);
                        assert!(s.name.read(cx).content.is_empty());
                        assert!(s.address.read(cx).content.is_empty());
                        assert!(s.evidence.latest.is_none() && s.evidence.baseline.is_none());
                    },
                    "website-validation" => { s.address.update(cx, |f, cx| f.set_value("northline.example", cx)); s.activate_control(Control::Save, window, cx); },
                    "access-login" | "access-login-actions" => { s.choose(Choice::LoginRequired, cx); },
                    "access-saved-private" | "access-browser-open" | "access-ready" | "access-invalid" => {
                        s.choose(Choice::LoginRequired, cx);
                        s.project.as_mut().unwrap().project.environments = s.draft_settings.environments.clone();
                        if label == "access-browser-open" { s.environment.as_mut().unwrap().login_open = true; }
                        if label == "access-ready" {
                            let session=SessionReference {id:SessionId::new(),project,environment:s.environment_draft.unwrap()};
                            s.draft_settings.environments[0].session=Some(session.clone());
                            s.project.as_mut().unwrap().project.environments[0].session=Some(session);
                        }
                        if label == "access-invalid" { s.crawl_fields[10].update(cx,|f,cx|f.set_value("invalid-login",cx));s.activate_control(Control::Save,window,cx); }
                    },
                    "access-status-loading" => { s.environment=None; },
                    "pages-invalid"=>{s.crawl_fields[4].update(cx,|f,cx|f.set_value("9",cx));s.activate_control(Control::Save,window,cx);assert_eq!(s.focus.setup_stage,SetupStage::Pages);assert!(s.pending.is_none());},
                    "mapping-preview" => { if copy_pass { s.setup_environment_open = true; } s.show_mapping = true; s.environment.as_mut().unwrap().mapped = (0..40).map(|i| (format!("https://northline.example/products/item-{i}/"), format!("https://staging.northline.example/products/item-{i}/"))).collect(); },
                    "pages-baseline" => { s.focus.review_surface = ReviewSurface::Pages; },
                    "discovery" | "discovery-options" | "discovery-no-matches" => { discovery(s); s.result_filters_open = label != "discovery"; if label.ends_with("no-matches") { s.crawl_fields[9].update(cx, |f, cx| f.set_value("nothing-matches", cx)); } },
                    "review-mobile" => { let key = s.evidence.latest.as_ref().unwrap().run.targets[1].key.clone(); s.selected_key = Some(key); s.reload_viewer(cx); finish_jobs(s, cx); },
                    "review-failed-target" if phase6 => { s.selected_key = Some(s.evidence.latest.as_ref().unwrap().run.targets.iter().find(|t| matches!(t.outcome, TargetOutcome::Failed { .. })).unwrap().key.clone()); s.reload_viewer(cx); finish_jobs(s, cx); assert!(s.selected_capture_failed()); assert!(!s.control_mounted(Control::Guided(44))); },
                    "review-mixed-148" | "review-failed-target" => { mixed_queue(s); if label == "review-failed-target" { s.selected_key = Some(s.evidence.latest.as_ref().unwrap().run.targets.iter().find(|t| matches!(t.outcome, TargetOutcome::Failed { .. })).unwrap().key.clone()); s.clear_viewer(); s.focus.viewer = false; } },
                    "review-queue-hidden" => { s.activate_control(Control::Guided(50), window, cx); },
                    "review-navigation-open" => { if s.navigation_collapsed { s.activate_control(Control::Guided(53), window, cx); } },
                    "review-mode-chooser" => { s.activate_control(Control::Guided(54), window, cx); assert!(s.viewer_mode_open); },
                    "review-full-width-before" | "review-full-width-after" => {
                        s.activate_control(Control::Guided(50), window, cx);
                        s.activate_control(Control::Guided(54), window, cx);
                        s.activate_control(Control::Guided(55), window, cx);
                        if label.ends_with("after") { s.activate_control(Control::Guided(56), window, cx); }
                        assert_eq!(s.viewer_mode, 4);
                        assert_eq!(s.full_width_after, label.ends_with("after"));
                    },
                    "review-difference" | "review-overlay" | "review-heatmap" => {
                        let mode = match label { "review-difference" => 7, "review-overlay" => 8, _ => 9 };
                        s.activate_control(Control::Guided(54), window, cx);
                        assert!(s.control_mounted(Control::Phase5(mode)));
                        s.activate_control(Control::Phase5(mode), window, cx);
                        finish_jobs(s, cx);
                        assert_eq!(s.viewer_mode, mode - 6);
                        if s.review_tools { s.activate_control(Control::Phase5(39), window, cx); }
                    },
                    "review-details" | "review-diagnostics" | "review-diagnostics-tail" => {
                        s.activate_control(Control::Phase5(36), window, cx);
                        if label.starts_with("review-diagnostics") {
                            assert!(s.control_mounted(Control::Phase5(38)));
                            s.activate_control(Control::Phase5(38), window, cx);
                            assert!(s.capture_details_open && s.capture_diagnostics_open);
                        }
                    },
                    "review-analysis" | "review-analysis-advanced" | "review-ignored-area" | "review-analysis-field-0" | "review-analysis-field-1" | "review-analysis-field-2" | "review-analysis-field-3" | "review-analysis-field-4" | "review-analysis-field-5" | "review-analysis-field-6" | "review-analysis-field-7" | "review-analysis-field-8" | "review-analysis-switches-tail" => {
                        s.activate_control(Control::Guided(44), window, cx);
                        if label == "review-analysis-advanced" || label.starts_with("review-analysis-field-") || label == "review-analysis-switches-tail" {
                            assert!(s.control_mounted(Control::Phase5(3)));
                            s.activate_control(Control::Phase5(3), window, cx);
                            assert!(s.review_tools && s.analysis_advanced);
                        }
                    },
                    "review-note" => { s.activate_control(Control::Phase5(37), window, cx); s.analysis_fields[9].update(cx, |f, cx| f.set_value("Autumn headline and CTA are intentional. Product grid needs release-owner confirmation.", cx)); },
                    "review-filters" | "review-no-matches" => { s.result_filters_open = true; if label.ends_with("no-matches") { s.crawl_fields[9].update(cx, |f, cx| f.set_value("nothing-matches", cx)); } },
                    "review-approval-selection" | "review-approval-confirm" => { s.activate_control(Control::Phase5(23), window, cx); if label.ends_with("confirm") { s.activate_control(Control::Phase5(24), window, cx); finish_jobs(s, cx); assert!(s.approval.is_some()); } },
                    "review-approval-many" | "review-approval-last-exact" | "review-approval-last-row" | "review-approval-exact-tail" => {
                        if label.ends_with("many") { s.approval_selection = s.evidence.latest.as_ref().unwrap().run.targets.iter().map(|t| t.key.clone()).filter(|k| s.approval_eligible(k)).collect(); }
                        else { s.activate_control(Control::Phase5(23), window, cx); }
                        s.activate_control(Control::Phase5(24), window, cx); finish_jobs(s, cx);
                        assert!(s.approval.is_some());
                        if label.ends_with("exact") || label.ends_with("tail") {
                            for _ in 0..11 { s.activate_control(Control::Guided(49), window, cx); }
                            s.activate_control(Control::Guided(52), window, cx);
                            assert_eq!(s.approval_reference_index, 11);
                            assert!(s.approval_exact_open);
                        }
                    },
                    "capture-gaps" | "capture-failed" | "capture-ready" | "capture-saved" => {
                        if copy_pass && label == "capture-saved" {
                            crate::ux_copy_tests::baseline_saved_reply(s, cx);
                            finish_jobs(s, cx);
                        }
                        capture_only(s, &store, label.strip_prefix("capture-").unwrap());
                    },
                    "capture-mobile" => {
                        capture_only(s, &store, "ready");
                        let record = s.evidence.latest.as_ref().unwrap();
                        let image = &record.captures[1].image;
                        s.selected_key = Some(record.run.targets[1].key.clone());
                let tiles = image.tiles.iter().take(3).map(|id| regressee_gui::viewer_images::viewer_tile(&store, id).unwrap()).collect();
                        let frame = s.viewer.as_mut().unwrap(); frame.width = image.width; frame.height = image.height; frame.panes[1] = tiles;
                    },
                    "capture-empty" | "capture-loading" | "capture-read-failed" | "capture-read-dismissed" => {
                        s.evidence = ProjectEvidence::default(); s.clear_viewer(); s.focus.viewer = false; s.focus.candidate = false; s.focus.baseline = false;
                        s.evidence_dirty = label != "capture-empty";
                        if label == "capture-loading" { s.pending = Some(u64::MAX); s.focus.busy = true; }
                        else if label.starts_with("capture-read-") { s.evidence_read_failed = true; s.error = Some("The saved screenshots could not be read. Check the project data folder, then refresh.".into()); if label.ends_with("dismissed") { s.activate_control(Control::DismissFeedback, window, cx); } }
                    },
                    "history-baselines" | "export-preview" => { s.history_section = 1; if label == "export-preview" { s.export_preview = Some(export.clone()); } else if phase6 { let last = s.evidence.baselines.len()-1; s.activate_control(Control::HistoryBaseline(last), window, cx); } },
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
                if repair {
                    // Presentation-only source records; fixture/store remains unchanged.
                    if let Some(run) = s.evidence.latest.as_mut() {
                        for target in &mut run.run.targets {
                            target.requested = "https://northline.example/account/orders/release-notes/?sort=recent&locale=nb-NO#delivery".parse().unwrap();
                        }
                        run.run.environment.name = "Historical check".into();
                    }
                    if let Some(frame) = s.viewer.as_mut() { frame.before_environment = Some("Historical reference".into()); }
                    s.navigation_preference = Some(true);
                    s.configure_review_geometry(width);
                    if label != "review-note" { s.review_queue_collapsed = true; }
                    if matches!(label, "capture-loading" | "capture-read-failed" | "capture-read-dismissed") {
                        assert!(s.control_enabled(Control::Guided(53), cx));
                        let requests = s.next_request;
                        s.activate_control(Control::Guided(53), window, cx);
                        assert!(!s.navigation_collapsed);
                        assert!(s.control_mounted(Control::Navigate(Screen::History)));
                        s.activate_control(Control::Guided(53), window, cx);
                        assert!(s.navigation_collapsed);
                        assert_eq!(requests, s.next_request);
                    }
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
                "access-options" => Some("field-Original website or section"),
                "mapping-preview" => Some("mapping-preview"),
                "timezone-popup" => Some("field-Capture timezone"),
                "pages-screen-sizes" => Some("section-Screen sizes"),
                "pages-images" => Some("section-Images"),
                "pages-speed" => Some("section-Capture speed"),
                "pages-advanced" => Some("field-Links away from starting addresses"),
                "pages-advanced-tail" => Some("field-Retries for temporary failures"),
                "app-settings-network" => Some("managed-network-section"),
                "app-settings-diagnostics" | "diagnostic-preview" => Some("section-Support report"),
                "history-provenance" => Some("action-Details(History)"),
                "export-preview" => Some("action-Phase6(3)"),
                "import-preview" if copy_pass => Some("portability-preview"),
                "delete-preview" => Some("action-Phase6(6)"),
                "integrity-preview" => Some("action-Phase6(14)"),
                _ => None,
            };
            if let Some(anchor) = anchor {
                let bounds = cx
                    .debug_bounds(anchor)
                    .unwrap_or_else(|| panic!("{label}: {anchor} must be rendered"));
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
            if phase6 && matches!(label, "history-bottom" | "history-baselines") {
                let list_bounds = cx.debug_bounds(if label == "history-bottom" {
                    "history-run-list"
                } else {
                    "history-baseline-list"
                });
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        let (control, scroll) = if label == "history-bottom" {
                            (
                                Control::HistoryRun(s.history.as_ref().unwrap().runs.len() - 1),
                                s.history_run_scroll.clone(),
                            )
                        } else {
                            (
                                Control::HistoryBaseline(s.evidence.baselines.len() - 1),
                                s.history_baseline_scroll.clone(),
                            )
                        };
                        s.activate_control(control, window, cx);
                        scroll.set_offset(gpui::point(px(0.), -scroll.max_offset().height));
                        if let Some(bounds) = list_bounds {
                            let delta = s.screen_scroll.bounds().top() + px(8.) - bounds.top();
                            let old = s.screen_scroll.offset();
                            s.screen_scroll
                                .set_offset(gpui::point(old.x, (old.y + delta).min(px(0.))));
                        }
                        cx.notify();
                    })
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
            let field_selector = label.strip_prefix("review-analysis-field-").map(|n| {
                // Debug selectors accept static strings; this bounded opt-in test
                // allocates only nine selectors per window, never product data.
                &*format!("analysis-parameter-field-{}", n.parse::<usize>().unwrap()).leak()
            });
            let disclosure_anchor = match label {
                "review-diagnostics" => Some("capture-provenance-details"),
                "review-analysis-advanced" => Some("analysis-parameter-field-0"),
                "review-analysis-switches-tail" => Some("check-Phase5(28)"),
                _ if field_selector.is_some() => field_selector,
                _ => None,
            };
            if label == "review-approval-last-row" {
                shell.update(cx, |s, cx| {
                    s.approval_provenance_scroll.set_offset(gpui::point(
                        px(0.),
                        -s.approval_provenance_scroll.max_offset().height,
                    ));
                    cx.notify();
                });
            }
            if label == "review-approval-exact-tail" {
                let exact = cx.debug_bounds("approval-exact-provenance").unwrap();
                shell.update(cx, |s, cx| {
                    let delta =
                        s.approval_provenance_scroll.bounds().bottom() - px(8.) - exact.bottom();
                    s.approval_provenance_scroll
                        .set_offset(gpui::point(px(0.), delta.min(px(0.))));
                    cx.notify();
                });
            }
            if phase5
                && matches!(
                    label,
                    "website-validation"
                        | "access-invalid"
                        | "pages-invalid"
                        | "pages-mobile-revealed"
                )
            {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        let control = if label == "pages-invalid" {
                            Control::CrawlField(4)
                        } else if label == "pages-mobile-revealed" {
                            Control::Guided(3)
                        } else if label == "website-validation" {
                            Control::Address
                        } else {
                            Control::CrawlField(10)
                        };
                        s.reveal_form_control(control, true, window, cx);
                        cx.notify();
                    })
                });
            }
            if let Some(anchor) = disclosure_anchor {
                let bounds = cx
                    .debug_bounds(anchor)
                    .unwrap_or_else(|| panic!("{label}: {anchor} must be mounted"));
                shell.update(cx, |s, cx| {
                    let old = s.review_details_scroll.offset();
                    let delta = s.review_details_scroll.bounds().top() + px(8.) - bounds.top();
                    s.review_details_scroll
                        .set_offset(gpui::point(old.x, (old.y + delta).min(px(0.))));
                    cx.notify();
                });
            }
            if label == "review-diagnostics-tail" {
                shell.update(cx, |s, cx| {
                    s.review_details_scroll.set_offset(gpui::point(
                        px(0.),
                        -s.review_details_scroll.max_offset().height,
                    ));
                    cx.notify();
                });
            }
            capture(cx, &root, &name).unwrap();
            let mut assertions = Vec::new();
            if phase7 {
                let owner = shell.read_with(cx, |s, _| s.review_details_scroll.bounds());
                if label == "review-diagnostics-tail" {
                    let content = cx.debug_bounds("capture-provenance-details").unwrap();
                    assert!(content.left() >= owner.left() && content.right() <= owner.right());
                    assert!(
                        content.bottom() > owner.top()
                            && content.bottom() <= owner.bottom() + px(1.)
                    );
                    shell.read_with(cx, |s, _| {
                        assert!(s.viewer.as_ref().unwrap().request.is_some())
                    });
                    assertions.push(serde_json::json!({"selector":"capture-provenance-details","tail_bottom_reachable":true,"content":format!("{content:?}"),"actual_inner_owner":format!("{owner:?}"),"qualification":"bottom of exact pair descriptor body, not whole oversized element visibility"}));
                } else {
                    let selector = disclosure_anchor.unwrap();
                    let field = cx.debug_bounds(selector).unwrap();
                    assert!(
                        field.left() >= owner.left()
                            && field.right() <= owner.right()
                            && field.top() >= owner.top()
                            && field.bottom() <= owner.bottom(),
                        "{selector} {field:?} outside current actual {owner:?}"
                    );
                    if let Some(i) = label.strip_prefix("review-analysis-field-") {
                        let i = i.parse::<usize>().unwrap();
                        shell.read_with(cx, |s, cx| {
                            let inner = s.analysis_fields[i].read(cx).rendered_bounds().unwrap();
                            assert!(
                                inner.size.width >= px(270.)
                                    && inner.left() >= owner.left()
                                    && inner.right() <= owner.right()
                                    && inner.top() >= owner.top()
                                    && inner.bottom() <= owner.bottom()
                            );
                        });
                    }
                    if label == "review-analysis-switches-tail" {
                        let copy = cx.debug_bounds("check-copy-Phase5(28)").unwrap();
                        assert!(
                            copy.left() >= field.left()
                                && copy.right() <= field.right()
                                && copy.top() >= field.top()
                                && copy.bottom() <= field.bottom()
                        );
                        assert!(
                            copy.size.height >= px(36.),
                            "full long label must wrap, not clip: {copy:?}"
                        );
                        assertions.push(serde_json::json!({"selector":"check-copy-Phase5(28)","wrapped_inner_label":format!("{copy:?}"),"painted_checkbox":format!("{field:?}"),"actual_inner_owner":format!("{owner:?}"),"fully_visible":true}));
                    }
                    assertions.push(serde_json::json!({"selector":selector,"painted_field":format!("{field:?}"),"actual_inner_owner":format!("{owner:?}"),"fully_visible":true}));
                }
            }
            if phase6 && matches!(label, "history-bottom" | "history-baselines") {
                let (control, owner) = shell.read_with(cx, |s, _| {
                    if label == "history-bottom" {
                        (
                            Control::HistoryRun(s.run_history_index),
                            s.history_run_scroll.bounds(),
                        )
                    } else {
                        (
                            Control::HistoryBaseline(s.history_index),
                            s.history_baseline_scroll.bounds(),
                        )
                    }
                });
                let row = cx
                    .debug_bounds(format!("history-row-{control:?}").leak())
                    .unwrap();
                let screen_clip = shell.read_with(cx, |s, _| s.screen_scroll.bounds());
                assert!(
                    row.left() >= owner.left()
                        && row.right() <= owner.right()
                        && row.top() >= owner.top()
                        && row.bottom() <= owner.bottom(),
                    "last selected painted row {row:?} outside current inner owner {owner:?}"
                );
                assert!(
                    row.top() >= screen_clip.top() && row.bottom() <= screen_clip.bottom(),
                    "last selected painted row outside screen clip"
                );
                assert!((f32::from(row.size.width) - f32::from(owner.size.width)).abs() <= 1.);
                assertions.push(serde_json::json!({"selected_last_row":format!("{control:?}"),"row":format!("{row:?}"),"actual_inner_scroll_owner":format!("{owner:?}"),"current_screen_clip":format!("{screen_clip:?}"),"full_width":true,"fully_visible":true}));
            }
            if phase6
                && matches!(
                    label,
                    "import-preview" | "export-preview" | "integrity-preview" | "delete-preview"
                )
            {
                shell.read_with(cx, |s, cx| {
                    assert!(s.control_enabled(Control::Details(Detail::Portability), cx))
                });
            }
            let viewport =
                gpui::Bounds::new(gpui::point(px(0.), px(0.)), size(px(width), px(height)));
            let selectors: &[&str] = match label {
                "new-website" => &["field-Project name", "field-Starting addresses"],
                "review-diagnostics" => &["capture-provenance-details"],
                "review-analysis-advanced" => &["analysis-parameter-field-0"],
                "review-zoom-menu" => &["zoom-popup", "zoom-option-0", "zoom-option-1"],
                "review-overlay" => &["phase5-before"],
                "review-heatmap" => &["phase5-difference"],
                "review-approval-last-row" => &[
                    "approval-reference-11",
                    "approval-reference-time-11",
                    "action-Phase5(25)",
                ],
                "review-approval-exact-tail" => &["action-Phase5(25)"],
                "review-note" => &[
                    "review-note-input-border",
                    "action-Phase5(32)",
                    "action-Phase5(39)",
                ],
                "review-full-width-before" => &["phase5-before"],
                "review-full-width-after" => &["phase5-after"],
                _ => &[],
            };
            if phase5 {
                let mut controls = vec!["action-Save"];
                if phase5_repair {
                    controls.extend([
                        "action-SetupStage(Website)",
                        "action-SetupStage(Access)",
                        "action-SetupStage(Pages)",
                    ]);
                    if label == "pages-mobile-revealed" {
                        controls.push("check-Guided(3)");
                    }
                }
                if matches!(
                    label,
                    "access-public"
                        | "access-login"
                        | "access-saved-private"
                        | "access-browser-open"
                        | "access-ready"
                        | "access-status-loading"
                ) {
                    controls.extend([
                        "action-SetupStage(Website)",
                        "action-SetupStage(Access)",
                        "action-SetupStage(Pages)",
                        "choice-Choice(PublicAccess)",
                        "choice-Choice(LoginRequired)",
                    ]);
                    if !matches!(label, "access-public" | "access-status-loading") {
                        controls.extend(["action-Guided(31)", "action-Guided(32)"]);
                    }
                }
                if label == "pages-sizes" {
                    controls.extend([
                        "setup-capture-plan",
                        "check-Guided(1)",
                        "check-Guided(2)",
                        "check-Guided(3)",
                        "action-Guided(13)",
                    ]);
                }
                if label == "mapping-preview" {
                    controls = vec!["mapping-preview-row-0", "action-CloseMappingPreview"];
                }
                if label.starts_with("discovery") {
                    controls = vec!["discovery-capture-plan", "action-CaptureBaseline"];
                }
                for selector in controls {
                    let b = cx
                        .debug_bounds(selector)
                        .unwrap_or_else(|| panic!("{name}: {selector} missing"));
                    let clip = if selector == "mapping-preview-row-0" {
                        shell.read_with(cx, |s, _| s.mapping_preview_scroll.bounds())
                    } else if selector.starts_with("choice-")
                        || selector.starts_with("check-")
                        || selector == "setup-capture-plan"
                    {
                        shell.read_with(cx, |s, _| s.screen_scroll.bounds())
                    } else {
                        viewport
                    };
                    let visible = b.size.width > px(0.)
                        && b.size.height > px(0.)
                        && b.left() >= clip.left()
                        && b.right() <= clip.right()
                        && b.top() >= clip.top()
                        && b.bottom() <= clip.bottom();
                    if selector != "check-Guided(3)" || label == "pages-mobile-revealed" {
                        assert!(
                            visible,
                            "{name}: {selector} {b:?} outside actual owner {clip:?}"
                        );
                    }
                    assertions.push(serde_json::json!({"selector":selector,"x":f32::from(b.left()),"y":f32::from(b.top()),"width":f32::from(b.size.width),"height":f32::from(b.size.height),"fully_visible":visible,"requires_scroll":!visible,"clip_x":f32::from(clip.left()),"clip_y":f32::from(clip.top()),"clip_width":f32::from(clip.size.width),"clip_height":f32::from(clip.size.height)}));
                }
                let input = match label {
                    "new-website" => Some(Control::Name),
                    "website-validation" => Some(Control::Address),
                    "access-invalid" => Some(Control::CrawlField(10)),
                    "pages-invalid" => Some(Control::CrawlField(4)),
                    "pages-advanced" => Some(Control::CrawlField(3)),
                    "pages-advanced-tail" => Some(Control::CrawlField(6)),
                    _ => None,
                };
                if let Some(control) = input {
                    shell.read_with(cx,|s,cx|{
                        let b=match control{Control::Name=>s.name.read(cx).rendered_bounds(),Control::Address=>s.address.read(cx).rendered_bounds(),Control::CrawlField(i)=>s.crawl_fields[i].read(cx).rendered_bounds(),_=>None}.unwrap();
                        let clip=s.screen_scroll.bounds();
                        assert!(b.size.width>=px(400.) && b.left()>=clip.left() && b.right()<=clip.right() && b.top()>=clip.top() && b.bottom()<=clip.bottom(),"{name}: actual input {b:?} outside {clip:?}");
                        if phase5_repair {let field=s.setup_control_bounds(control).unwrap().get().unwrap();assert!(field.left()>=clip.left() && field.right()<=clip.right() && field.top()>=clip.top() && field.bottom()<=clip.bottom(),"{name}: border/error/help {field:?} outside current {clip:?}");assertions.push(serde_json::json!({"painted_field_with_error_help":format!("{control:?}"),"x":f32::from(field.left()),"y":f32::from(field.top()),"width":f32::from(field.size.width),"height":f32::from(field.size.height),"clip_x":f32::from(clip.left()),"clip_y":f32::from(clip.top()),"clip_width":f32::from(clip.size.width),"clip_height":f32::from(clip.size.height),"fully_visible":true}));}
                        assertions.push(serde_json::json!({"actual_input":format!("{control:?}"),"x":f32::from(b.left()),"y":f32::from(b.top()),"width":f32::from(b.size.width),"height":f32::from(b.size.height),"fully_visible":true}));
                    });
                }
            }
            for selector in selectors {
                let bounds = cx
                    .debug_bounds(selector)
                    .unwrap_or_else(|| panic!("{name}: {selector} not mounted"));
                let clip = if disclosure_anchor.is_some() {
                    shell.read_with(cx, |s, _| s.review_details_scroll.bounds())
                } else {
                    viewport
                };
                assert!(bounds.size.width > px(0.) && bounds.size.height > px(0.));
                assert!(
                    bounds.left() >= clip.left() && bounds.right() <= clip.right(),
                    "{name}: {selector} horizontal visibility"
                );
                assert!(
                    bounds.top() >= clip.top() && bounds.top() < clip.bottom(),
                    "{name}: {selector} starts visibly inside its owner"
                );
                if *selector != "capture-provenance-details" {
                    assert!(
                        bounds.bottom() <= clip.bottom(),
                        "{name}: {selector} fully visible"
                    );
                }
                assertions.push(serde_json::json!({"selector":selector,"x":f32::from(bounds.left()),"y":f32::from(bounds.top()),"width":f32::from(bounds.size.width),"height":f32::from(bounds.size.height),"clip_x":f32::from(clip.left()),"clip_y":f32::from(clip.top()),"clip_width":f32::from(clip.size.width),"clip_height":f32::from(clip.size.height),"fully_visible":bounds.bottom() <= clip.bottom()}));
            }
            if repair {
                let identity_bounds: Vec<_> = [
                    "inspected-target-path",
                    "inspected-target-profile-environments",
                ]
                .into_iter()
                .filter_map(|name| cx.debug_bounds(name).map(|b| (name, b)))
                .collect();
                shell.read_with(cx, |s, cx| {
                    if label == "review-note" {
                        let inner = s.analysis_fields[9].read(cx).rendered_bounds().unwrap();
                        let clip = s.review_details_scroll.bounds();
                        assert!(inner.size.width >= px(270.));
                        assert!(inner.left() >= clip.left() && inner.right() <= clip.right() && inner.top() >= clip.top() && inner.bottom() <= clip.bottom());
                        assertions.push(serde_json::json!({"selector":"review-note-inner-rendered-bounds","width":f32::from(inner.size.width),"height":f32::from(inner.size.height),"viewport_width":f32::from(clip.size.width),"viewport_height":f32::from(clip.size.height),"draft":s.analysis_fields[9].read(cx).content,"fully_visible":true}));
                    } else if s.evidence.latest.is_some() {
                        let (path, context) = s.inspection_identity();
                        assert_eq!(path, "/account/orders/release-notes/?sort=recent&locale=nb-NO#delivery");
                        assert!(context.contains("Historical check"));
                        if s.evidence.comparison.is_some() { assert!(context.contains("Historical reference")); }
                        assert_eq!(identity_bounds.len(), 2);
                        for (selector, b) in &identity_bounds {
                            assert!(b.left() >= viewport.left() && b.right() <= viewport.right() && b.top() >= viewport.top() && b.bottom() <= viewport.bottom());
                            assertions.push(serde_json::json!({"selector":selector,"text":if selector.ends_with("path") { &path } else { &context },"width":f32::from(b.size.width),"height":f32::from(b.size.height),"fully_visible":true}));
                        }
                    } else {
                        assert!(s.control_enabled(Control::Guided(53), cx));
                        assertions.push(serde_json::json!({"selector":"action-Guided(53)","mounted":s.control_mounted(Control::Guided(53)),"enabled":true,"loading":s.initial_evidence_loading(),"unavailable":s.initial_evidence_unavailable(),"error_dismissed":s.error.is_none(),"read_failed":s.evidence_read_failed}));
                    }
                });
            }
            let geometry: Vec<_> = ["review-queue", "viewer-toolbar", "phase5-before", "phase5-after", "ordinary-review-decisions", "approval-next-action", "page-list", "history-run-list", "history-baseline-list", "selected-project-context", "approval-reference-list", "approval-exact-provenance", "approval-reference-11", "approval-reference-time-11", "viewer-inspector"].into_iter().filter_map(|selector| cx.debug_bounds(selector).map(|b| serde_json::json!({"selector":selector,"x":f32::from(b.left()),"y":f32::from(b.top()),"width":f32::from(b.size.width),"height":f32::from(b.size.height)}))).collect();
            manifest.push(serde_json::json!({"capture":name,"state":label,"width":width,"height":height,"synthetic":true,"website_visited":false,"os_input_verified":false,"geometry":geometry,"visibility_assertions":assertions,"harness_sha256":hex::encode(Sha256::digest(include_bytes!("ui_review_matrix.rs")))}));
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
