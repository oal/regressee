// SPDX-License-Identifier: MPL-2.0
//! Opt-in diagnostic tests using GPUI's real offscreen renderer, not OS capture.
use crate::*;
use anyhow::{Context as _, Result, ensure};
use gpui::{TestAppContext, VisualTestContext};
use sha2::{Digest, Sha256};
use std::{
    fs,
    os::unix::fs::{OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
};

#[test]
fn offscreen_extent_limits_without_gpu() {
    for (width, height) in [(640, 440), (1120, 800), (820, 600), (4096, 4096)] {
        gpui::OffscreenFrame::validate_dimensions(width, height).unwrap();
    }
    for (width, height) in [
        (0, 600),
        (800, 0),
        (4096, 4097),
        (1 << 31, 1 << 31),
        (u32::MAX, u32::MAX),
    ] {
        assert!(gpui::OffscreenFrame::validate_dimensions(width, height).is_err());
    }
}

fn output_root() -> Result<PathBuf> {
    let root = PathBuf::from(
        std::env::var_os("REGRESSEE_RENDERER_CAPTURE_DIR")
            .context("explicit private renderer capture directory required")?,
    );
    // Keep the private default, but let an explicitly authorized diagnostic run
    // supply its physical scratch root without baking a developer's home path
    // into source. This remains an opt-in test, never a production setting.
    let approved = PathBuf::from(
        std::env::var_os("REGRESSEE_RENDERER_APPROVED_ROOT")
            .unwrap_or_else(|| "/path/to/private-tests/opencode".into()),
    );
    ensure!(approved.is_absolute() && fs::canonicalize(&approved)? == approved);
    ensure!(root.is_absolute() && root.starts_with(&approved));
    ensure!(fs::canonicalize(&root)? == root);
    ensure!(fs::metadata(&root)?.permissions().mode() & 0o777 == 0o700);
    Ok(root)
}

fn write_frame(
    root: &Path,
    label: &str,
    width: u32,
    height: u32,
    rgba: Vec<u8>,
    cpu_draw_ms: f64,
    gpu_capture_ms: f64,
) -> Result<()> {
    ensure!(width > 0 && height > 0 && width as u64 * height as u64 <= 4_000_000);
    ensure!(rgba.len() == width as usize * height as usize * 4);
    // Preserve the actual bytes even if calibration fails; never tint or fix them.
    let opaque = rgba.as_chunks::<4>().0.iter().all(|pixel| pixel[3] == 255);
    let mut colors = std::collections::BTreeSet::new();
    colors.extend(
        rgba.as_chunks::<4>()
            .0
            .iter()
            .map(|pixel| [pixel[0], pixel[1], pixel[2]]),
    );
    let nonblank = colors.len() > 16;
    let root = root.to_owned();
    let label = label.to_owned();
    // Encoding/file I/O is outside the App/Window callback and render thread.
    std::thread::spawn(move || -> Result<()> {
        let path = root.join(format!("{label}.png"));
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(&path)?;
        let mut encoder = png::Encoder::new(file, width, height);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        let mut writer = encoder.write_header()?;
        writer.write_image_data(&rgba)?;
        writer.finish()?;
        let bytes = fs::read(&path)?;
        let metadata = serde_json::json!({
            "width": width, "height": height,
            "png_sha256": hex::encode(Sha256::digest(&bytes)),
            "distinct_rgb_colors": colors.len(), "opaque": opaque,
            "srgb": true, "premultiplied_alpha": true,
            "cpu_draw_ms": cpu_draw_ms, "gpu_draw_readback_wait_ms": gpu_capture_ms,
            "real_gpui_blade_offscreen": true, "real_linux_fonts": true,
            "os_presentation_verified": false, "os_input_verified": false,
            "synthetic_project_only": true
        });
        let file = fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(root.join(format!("{label}.json")))?;
        serde_json::to_writer_pretty(file, &metadata)?;
        Ok(())
    })
    .join()
    .map_err(|_| anyhow::anyhow!("frame encoding thread failed"))??;
    ensure!(
        opaque,
        "nonopaque readback; preserved without alpha conversion"
    );
    ensure!(
        nonblank,
        "blank/unreadable readback; preserved without fallback"
    );
    Ok(())
}

fn capture(cx: &mut VisualTestContext, root: &Path, label: &str) -> Result<()> {
    let (frame, viewport, cpu_draw_ms, gpu_capture_ms) = cx.update(|window, cx| {
        let started = std::time::Instant::now();
        let _ = window.draw(cx);
        let cpu_draw_ms = started.elapsed().as_secs_f64() * 1000.;
        let started = std::time::Instant::now();
        let frame = window.capture_offscreen_frame();
        let gpu_capture_ms = started.elapsed().as_secs_f64() * 1000.;
        (frame, window.viewport_size(), cpu_draw_ms, gpu_capture_ms)
    });
    let frame = frame?;
    ensure!(
        frame.width == f32::from(viewport.width) as u32
            && frame.height == f32::from(viewport.height) as u32,
        "readback dimensions do not match the owned viewport"
    );
    let calibration_counts: Vec<_> = [[255_u8, 0, 0], [0, 255, 0], [0, 0, 255], [64, 128, 192]]
        .into_iter()
        .map(|rgb| {
            frame
                .rgba
                .as_chunks::<4>()
                .0
                .iter()
                .filter(|pixel| {
                    pixel[..3]
                        .iter()
                        .zip(rgb)
                        .all(|(actual, expected)| actual.abs_diff(expected) <= 1)
                })
                .count()
        })
        .collect();
    write_frame(
        root,
        label,
        frame.width,
        frame.height,
        frame.rgba,
        cpu_draw_ms,
        gpu_capture_ms,
    )?;
    if label == "renderer-proof" {
        eprintln!("Calibration exact RGB counts: {calibration_counts:?}");
        for count in calibration_counts {
            ensure!(
                count >= 512,
                "calibration channel/readback mismatch; stop without fallback"
            );
        }
    }
    Ok(())
}

struct ProofView {
    input: Entity<TextInput>,
    action_focus: FocusHandle,
    choice_focus: FocusHandle,
    image: Arc<gpui::RenderImage>,
}

impl Render for ProofView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .flex()
            .flex_col()
            .p_6()
            .gap_4()
            .bg(rgb(GROUND))
            .text_color(rgb(INK))
            .font_family("DejaVu Sans")
            .text_size(px(16.))
            .child(div().text_size(px(24.)).child("Real GPUI renderer · æøå"))
            .child(controls::action(
                "proof-action",
                "Filled action",
                &self.action_focus,
                ActionStyle::Primary,
                true,
            ))
            .child(self.input.clone())
            .child(controls::choice(
                "proof-choice",
                "Selected radio option",
                "Real fonts, glyph atlas and GPUI layout.",
                &self.choice_focus,
                true,
                true,
            ))
            .child(
                div()
                    .flex()
                    .gap_4()
                    .child(div().size(px(32.)).bg(rgb(0xff0000)))
                    .child(div().size(px(32.)).bg(rgb(0x00ff00)))
                    .child(div().size(px(32.)).bg(rgb(0x0000ff)))
                    .child(div().size(px(32.)).bg(rgb(0x4080c0)))
                    .child(img(self.image.clone()).size(px(32.))),
            )
    }
}

#[gpui::test]
#[ignore = "one explicitly requested offscreen GPU/font/atlas calibration"]
fn renderer_capture_proof(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    let (_, cx) = cx.add_window_view(|_, cx| {
        let input = cx.new(TextInput::new);
        input.update(cx, |input, cx| {
            input.set_value("Editable white field · actual font", cx)
        });
        // RenderImage 0.2.2 consumes BGRA pixels, just like viewer_tile().
        let image = image::RgbaImage::from_raw(
            2,
            2,
            vec![
                0, 0, 255, 255, 0, 255, 0, 255, 255, 0, 0, 255, 255, 255, 255, 255,
            ],
        )
        .unwrap();
        ProofView {
            input,
            action_focus: cx.focus_handle(),
            choice_focus: cx.focus_handle(),
            image: Arc::new(gpui::RenderImage::new(vec![image::Frame::new(image)])),
        }
    });
    cx.simulate_resize(size(px(640.), px(440.)));
    capture(cx, &root, "renderer-proof").unwrap();
}

#[gpui::test]
#[ignore = "bounded direct-viewer UX capture; synthetic isolated evidence only"]
fn renderer_capture_viewer_ux(cx: &mut TestAppContext) {
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
    let dir = tempfile::Builder::new()
        .prefix("viewer-ux-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for label in [
            "review",
            "analysis",
            "area-editor",
            "details",
            "capture-details",
            "failed-capture",
            "failed-details",
            "help",
        ] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    finish_jobs(s, cx);
                    s.activate_control(Control::SelectProject(0), window, cx);
                    finish_jobs(s, cx);
                    s.navigate(Screen::Results, window, cx);
                    finish_jobs(s, cx);
                    s.configure_review_geometry(width);
                    if label.contains("capture") || label == "failed-details" {
                        s.navigate(Screen::History, window, cx);
                        finish_jobs(s, cx);
                        let source = s.evidence.baseline.as_ref().unwrap().references[0].run;
                        let index = s
                            .history
                            .as_ref()
                            .unwrap()
                            .runs
                            .iter()
                            .position(|r| r.run.id == source)
                            .unwrap();
                        s.activate_control(Control::HistoryRun(index), window, cx);
                        s.activate_control(Control::Phase6(19), window, cx);
                        finish_jobs(s, cx);
                        assert!(s.evidence.comparison.is_none());
                        if label.starts_with("failed") {
                            // Presentation-only failed capture; never saved or activated.
                            s.history_inspection = None;
                            s.evidence.baseline = None;
                            s.project.as_mut().unwrap().project.active_baseline = None;
                            s.focus.baseline = false;
                            let key = s.selected_key.as_ref().unwrap();
                            let run = s.evidence.latest.as_mut().unwrap();
                            run.run
                                .targets
                                .iter_mut()
                                .find(|t| &t.key == key)
                                .unwrap()
                                .outcome = TargetOutcome::Failed {
                                code: regressee_core::domain::CaptureError::ImageLimit,
                            };
                            run.run.state = JobState::CompletedWithErrors;
                            for pane in &mut s.viewer.as_mut().unwrap().panes {
                                pane.clear();
                            }
                        }
                    }
                    s.notice = None;
                    if matches!(label, "analysis" | "area-editor") {
                        s.activate_control(Control::Guided(44), window, cx);
                    }
                    if label.ends_with("details") {
                        s.activate_control(Control::Phase5(36), window, cx);
                    }
                    if label == "help" {
                        s.show_keyboard_help(window, cx);
                    }
                    cx.notify();
                })
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
                window.capture_offscreen_frame().unwrap();
            });
            if label == "area-editor" {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        s.activate_control(Control::Phase5(34), window, cx)
                    })
                });
                cx.run_until_parked();
            }
            capture(cx, &root, &format!("viewer-{width:.0}x{height:.0}-{label}")).unwrap();
            owned.join_after_ui();
        }
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
        original
    );
}

fn finish_jobs(shell: &mut Shell, cx: &mut Context<Shell>) {
    while shell.pending.is_some() || shell.focus.view_loading || shell.storage_pending.is_some() {
        let update = shell.events.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            !matches!(update.reply, Reply::Failure(_)),
            "{:?}",
            update.reply
        );
        shell.apply_update(update, cx);
    }
}

#[gpui::test]
#[ignore = "bounded zoom/loading renderer batch; isolated synthetic evidence only"]
fn renderer_capture_zoom_loading(cx: &mut TestAppContext) {
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
    let dir = tempfile::Builder::new()
        .prefix("zoom-loading-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for label in [
            "zoom-fit",
            "zoom-menu",
            "zoom-manual",
            "zoom-area-editor",
            "loading-project",
            "loading-screenshot",
            "loading-history",
            "empty-screenshots",
            "failed-load",
        ] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| shell.update(cx, |s, cx| {
                finish_jobs(s, cx);
                s.activate_control(Control::SelectProject(0), window, cx);
                finish_jobs(s, cx);
                s.configure_review_geometry(width);
                s.notice = None;
                match label {
                    "zoom-manual" => { s.choose_viewer_zoom(ZoomChoice::Percent(75), cx); finish_jobs(s, cx); }
                    "zoom-area-editor" => {
                        s.review_tools = true;
                        s.activate_control(Control::Phase5(34), window, cx);
                    }
                    "loading-project" | "empty-screenshots" | "failed-load" => {
                        s.evidence = ProjectEvidence::default();
                        s.clear_viewer();
                        s.focus.viewer = false;
                        s.focus.candidate = false;
                        s.focus.baseline = false;
                        s.evidence_dirty = label != "empty-screenshots";
                        if label == "loading-project" {
                            s.pending = Some(u64::MAX);
                            s.focus.busy = true;
                            s.progress = Some(("Loading saved screenshots and results…".into(), 0, None));
                        } else if label == "failed-load" {
                            s.error = Some("Synthetic disk-read failure. Check the project data folder, then refresh.".into());
                        }
                    }
                    "loading-screenshot" => {
                        let identity = s.viewer_identity().unwrap();
                        s.clear_viewer();
                        s.viewer_pending = Some((u64::MAX, identity, 0));
                        s.focus.view_loading = true;
                    }
                    "loading-history" => {
                        s.pending = Some(u64::MAX);
                        s.focus.screen = Screen::History;
                        s.history = None;
                        s.focus.busy = true;
                        s.progress = Some(("Loading capture and baseline history…".into(), 0, None));
                    }
                    _ => (),
                }
                cx.notify();
            }));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            if label == "zoom-menu" {
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        s.activate_control(Control::Guided(46), window, cx)
                    })
                });
            }
            capture(
                cx,
                &root,
                &format!("zoom-loading-{width:.0}x{height:.0}-{label}"),
            )
            .unwrap();
            owned.join_after_ui();
        }
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap(),
        original
    );
}

// Only the final workflow batch uses these helpers. Older diagnostic tests and
// calibration guards intentionally remain unchanged.
fn workflow_fixture_mixed_queue(s: &mut Shell) {
    use regressee_core::domain::{CaptureError, InventoryPage, PageId};
    // Presentation-only clones, NEVER saved/analyzed as real website results.
    // Keep the real stored target/key and its Jobs-loaded screenshot selected.
    // Extra rows have distinct keys and explicit fixture addresses; cloned CAS
    // evidence is only used to exercise layout, not to establish eligibility.
    let selected = s.selected_key.clone().unwrap();
    let run = s.evidence.latest.as_mut().unwrap();
    let template = run.run.targets[0].clone();
    assert_eq!(template.key, selected);
    let pixels = run.evidence(&selected).unwrap().clone();
    let comparison = s.evidence.comparison.as_mut().unwrap();
    let result = comparison
        .revision
        .results
        .iter()
        .find(|r| r.key == selected)
        .unwrap()
        .clone();
    for (name, classification) in [
        ("fixture-unchanged", Classification::NoSignificantChange),
        ("fixture-failed-timeout", Classification::CouldNotCapture),
        ("fixture-not-comparable", Classification::NotComparable),
        ("fixture-changed", Classification::Changed),
        ("fixture-failed-login", Classification::CouldNotCapture),
    ] {
        let page = InventoryPage {
            id: PageId::new(),
            canonical: template.requested.join(&format!("/{name}/")).unwrap(),
            included: true,
            reason: None,
        };
        let mut target = template.clone();
        target.key.page = page.id;
        target.requested = page.canonical.clone();
        target.final_url = Some(page.canonical.clone());
        target.warnings =
            vec!["Synthetic fixture-only presentation row; not browser evidence".into()];
        let mut classified = result.clone();
        classified.key = target.key.clone();
        classified.classification = classification;
        match classification {
            Classification::CouldNotCapture => {
                target.outcome = TargetOutcome::Failed {
                    code: if name.ends_with("login") {
                        CaptureError::NeedsLogin
                    } else {
                        CaptureError::Timeout
                    },
                };
                target.final_url = None;
                classified.eligible_pixels = 0;
                classified.changed_pixels = 0;
            }
            Classification::NotComparable => {
                classified.eligible_pixels = 0;
                classified.changed_pixels = 0;
            }
            Classification::NoSignificantChange => classified.changed_pixels = 0,
            Classification::Changed => assert!(classified.changed_pixels > 0),
        }
        if !matches!(target.outcome, TargetOutcome::Failed { .. }) {
            let mut capture = pixels.clone();
            capture.metadata.key = target.key.clone();
            capture.metadata.requested = page.canonical.to_string();
            capture.metadata.final_url = page.canonical.to_string();
            capture.metadata.warnings = target.warnings.clone();
            run.captures.push(capture);
        }
        classified.validate().unwrap();
        comparison.revision.results.push(classified);
        run.run.inventory.pages.push(page);
        run.run.targets.push(target);
    }
    run.run.state = JobState::CompletedWithErrors;
    run.run.validate().unwrap();
    assert_eq!(run.run.targets.len(), 6);
    assert_eq!(s.selected_key.as_ref(), Some(&selected));
    assert!(run.evidence(&selected).is_some());
    // No invented perceptual reports are added: cloned rows cannot be approved.
    for target in run.run.targets.iter().skip(1) {
        assert!(
            !comparison
                .perceptual
                .as_ref()
                .unwrap()
                .reports
                .iter()
                .any(|(key, _)| key == &target.key)
        );
    }
}

fn workflow_visible_bounds(
    cx: &mut VisualTestContext,
    selector: &str,
    width: f32,
    height: f32,
) -> gpui::Bounds<gpui::Pixels> {
    // GPUI's test selector API requires a static string. This opt-in finite
    // batch interns a bounded number of short diagnostic selectors only.
    let bounds = cx
        .debug_bounds(Box::leak(selector.to_owned().into_boxed_str()))
        .unwrap_or_else(|| panic!("required mounted selector: {selector}"));
    assert!(
        bounds.size.width > px(0.) && bounds.size.height > px(0.),
        "empty {selector}"
    );
    assert!(
        bounds.left() >= px(0.)
            && bounds.top() >= px(0.)
            && bounds.right() <= px(width)
            && bounds.bottom() <= px(height),
        "{selector} clipped outside {width}x{height}: {bounds:?}"
    );
    bounds
}

fn workflow_visible_action(cx: &mut VisualTestContext, control: Control, width: f32, height: f32) {
    let action = workflow_visible_bounds(cx, &format!("action-{control:?}"), width, height);
    let label = workflow_visible_bounds(cx, &format!("action-label-{control:?}"), width, height);
    assert!(
        action.size.height >= px(40.),
        "action target shrunk: {control:?}"
    );
    assert!(
        label.left() >= action.left()
            && label.right() <= action.right()
            && label.top() >= action.top()
            && label.bottom() <= action.bottom(),
        "action label clipped: {control:?}"
    );
}

fn workflow_review_geometry(cx: &mut VisualTestContext, first_row: usize, width: f32, height: f32) {
    let bar = workflow_visible_bounds(cx, "approval-next-action", width, height);
    let list = workflow_visible_bounds(cx, "page-list", width, height);
    let row = workflow_visible_bounds(cx, &format!("page-row-target-{first_row}"), width, height);
    assert!(row.size.height >= px(PAGE_ROW_HEIGHT));
    assert!(
        row.top() >= list.top() && row.bottom() <= list.bottom() && row.bottom() <= bar.top(),
        "first complete review row must be visible above approval bar"
    );
    // Use existing pane/tile selectors, not guessed layout coordinates. New
    // workspace selectors are useful but are not necessary to pass this guard.
    let after = workflow_visible_bounds(cx, "phase5-after", width, height);
    assert!(after.size.height >= px(64.) && after.size.width >= px(96.));
    assert!(
        after.bottom() <= bar.top(),
        "after pane hidden by approval bar"
    );
    let tile = cx
        .debug_bounds("phase5-tile-1-1-0")
        .expect("actual stored after image must be painted");
    assert!(
        (tile.right().min(after.right()) - tile.left().max(after.left())) >= px(32.)
            && (tile.bottom().min(after.bottom()) - tile.top().max(after.top())) >= px(32.),
        "stored screenshot pixels must be visible without scrolling"
    );
    if width >= 1280. {
        let before = workflow_visible_bounds(cx, "phase5-before", width, height);
        assert!(before.bottom() <= bar.top() && before.size.height >= px(64.));
        assert!(
            row.right() <= before.left(),
            "wide review must put queue beside image panes"
        );
    }
    workflow_visible_action(cx, Control::Phase5(23), width, height);
}

#[gpui::test]
#[ignore = "one final bounded 34-image workflow confirmation batch after proof-image review"]
fn renderer_capture_workflow_revamp(cx: &mut TestAppContext) {
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
        .prefix("workflow-synthetic-store-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let original_project = serde_json::to_value(store.load_project(project).unwrap()).unwrap();
    let original_evidence = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    // One fixed batch, no capture retries or QA loop. Every state gets a fresh
    // shell: navigation resets stages, and access edits/masks/approval must not
    // leak into another image. All actor commands below are read-only.
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for (label, screen, stage, surface) in [
            (
                "projects",
                Screen::Projects,
                SetupStage::Settings,
                ReviewSurface::Changes,
            ),
            (
                "website",
                Screen::Setup,
                SetupStage::Website,
                ReviewSurface::Changes,
            ),
            (
                "access",
                Screen::Setup,
                SetupStage::Access,
                ReviewSurface::Changes,
            ),
            (
                "capture-options",
                Screen::Setup,
                SetupStage::Pages,
                ReviewSurface::Changes,
            ),
            (
                "review",
                Screen::Results,
                SetupStage::Settings,
                ReviewSurface::Changes,
            ),
            (
                "pages-baseline",
                Screen::Results,
                SetupStage::Settings,
                ReviewSurface::Pages,
            ),
            (
                "history",
                Screen::History,
                SetupStage::Settings,
                ReviewSurface::Changes,
            ),
            (
                "app-settings",
                Screen::Settings,
                SetupStage::Settings,
                ReviewSurface::Changes,
            ),
            (
                "fixture-mixed-queue",
                Screen::Results,
                SetupStage::Settings,
                ReviewSurface::Changes,
            ),
            (
                "approval-selected",
                Screen::Results,
                SetupStage::Settings,
                ReviewSurface::Changes,
            ),
            (
                "approval-impact-confirm",
                Screen::Results,
                SetupStage::Settings,
                ReviewSurface::Changes,
            ),
            (
                "fixture-candidate-inspect-save",
                Screen::Results,
                SetupStage::Settings,
                ReviewSurface::Inspect,
            ),
            (
                "fixture-discovery-before-capture",
                Screen::Results,
                SetupStage::Settings,
                ReviewSurface::Pages,
            ),
            (
                "fixture-finite-capture-progress",
                Screen::Results,
                SetupStage::Settings,
                ReviewSurface::Capture,
            ),
            (
                "fixture-missing-evidence",
                Screen::Results,
                SetupStage::Settings,
                ReviewSurface::Changes,
            ),
            (
                "fixture-login-required",
                Screen::Setup,
                SetupStage::Access,
                ReviewSurface::Changes,
            ),
            (
                "fixture-dirty-access",
                Screen::Setup,
                SetupStage::Access,
                ReviewSurface::Changes,
            ),
        ] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    finish_jobs(s, cx);
                    s.activate_control(Control::SelectProject(0), window, cx);
                    finish_jobs(s, cx);
                });
            });
            // Simulated window draws can queue atlas uploads without submitting
            // a GPU frame. Flush the loaded state before a presentation fixture
            // retires its images (the missing-evidence state), matching native
            // frame submission. This is not another saved inspection image.
            cx.update(|window, cx| {
                let _ = window.draw(cx);
                window.capture_offscreen_frame().unwrap();
            });
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.navigate(screen, window, cx);
                    finish_jobs(s, cx);
                    s.focus.setup_stage = stage;
                    s.focus.review_surface = surface;
                    s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                    assert!(!s.mask_dirty && s.analysis_pending.is_none() && s.analysis_due.is_none());
                    assert!(s.approval_selection.is_empty() && s.approval.is_none());
                    match label {
                        "fixture-mixed-queue" => workflow_fixture_mixed_queue(s),
                        "approval-selected" | "approval-impact-confirm" => {
                            let key = s.selected_key.clone().unwrap();
                            assert!(s.approval_eligible(&key), "stored fixture must be genuinely eligible before selecting");
                            assert!(s.control_enabled(Control::Phase5(23), cx));
                            s.activate_control(Control::Phase5(23), window, cx);
                            assert!(s.approval_selection.contains(&key));
                            assert!(s.control_enabled(Control::Phase5(24), cx));
                            if label == "approval-impact-confirm" {
                                // Real Jobs -> Store preview, bound to the exact
                                // saved analysis/selection. NEVER dispatch Accept.
                                s.activate_control(Control::Phase5(24), window, cx);
                                assert!(s.pending.is_some(), "preview must really reach the actor");
                                finish_jobs(s, cx);
                                let preview = s.approval.as_ref().expect("actor approval preview");
                                assert!(s.approval_matches(preview));
                                assert_eq!(preview.selected, vec![key]);
                                assert_eq!(preview.provenance.len(), 1);
                                assert!(s.control_enabled(Control::Phase5(25), cx));
                            }
                        }
                        "fixture-candidate-inspect-save" => {
                            // Private first-baseline presentation ONLY. Retain
                            // actual stored pixels; do not create/save a run or
                            // pretend this clone is an activatable stored capture.
                            s.evidence.latest.as_mut().unwrap().run.intent = RunIntent::NewBaseline;
                            s.evidence.baseline = None;
                            s.evidence.before = None;
                            s.evidence.comparison = None;
                            s.project.as_mut().unwrap().project.active_baseline = None;
                            s.focus.baseline = false;
                            s.focus.candidate = true;
                            s.viewer_mode = 1; // Retained after pixels only; no invented first reference.
                            s.viewer.as_mut().unwrap().request = None;
                            s.notice = Some("Fixture-only candidate inspection using retained pixels; no new website capture. Save is explicit and is not executed in this batch.".into());
                            assert!(s.focus.can_activate(Control::SaveBaseline));
                            assert!(s.viewer.is_some());
                        }
                        "fixture-discovery-before-capture" => {
                            // Typed, in-memory discovery preview. This inventory
                            // is copied from saved synthetic evidence, NOT crawled.
                            let mut inventory = s.evidence.latest.as_ref().unwrap().run.inventory.clone();
                            let template = inventory.pages[0].clone();
                            for name in ["fixture-included", "fixture-excluded", "fixture-help"] {
                                let mut page = template.clone();
                                page.id = regressee_core::domain::PageId::new();
                                page.canonical = template.canonical.join(&format!("/{name}/")).unwrap();
                                page.included = name != "fixture-excluded";
                                page.reason = (!page.included).then(|| "Excluded in fixture preview".into());
                                inventory.pages.push(page);
                            }
                            inventory.validate().unwrap();
                            s.selected_page = Some(inventory.pages[0].id);
                            let record = s.project.as_ref().unwrap();
                            s.preview = Some(DiscoveryPreview {
                                project_revision: record.revision,
                                settings: record.project.clone(),
                                inventory,
                                aliases: vec![],
                                notes: vec!["Synthetic fixture inventory preview; no discovery browser was run.".into()],
                                visited: 4,
                            });
                            s.focus.preview = true;
                            s.focus.candidate = false;
                            assert!(s.preview.as_ref().unwrap().inventory.pages.iter().any(|p| !p.included));
                        }
                        "fixture-finite-capture-progress" => {
                            // Finite progress presentation, not a capture job.
                            s.focus.busy = true;
                            s.focus.capturing = true;
                            s.focus.pausable = false;
                            s.progress = Some(("Synthetic fixture progress · 3 of 6 screenshot targets processed".into(), 3, Some(6)));
                        }
                        "fixture-missing-evidence" => {
                            workflow_fixture_mixed_queue(s);
                            let failed = s.evidence.latest.as_ref().unwrap().run.targets.iter()
                                .find(|t| matches!(t.outcome, TargetOutcome::Failed { .. })).unwrap().key.clone();
                            assert!(!s.approval_eligible(&failed));
                            s.selected_key = Some(failed);
                            s.clear_viewer();
                            s.focus.viewer = false;
                            assert!(!s.control_enabled(Control::Phase5(23), cx));
                            assert!(!s.control_enabled(Control::Phase5(24), cx));
                        }
                        "fixture-login-required" | "fixture-dirty-access" => {
                            // Unsaved access choice is private UI state. Do not
                            // save login/settings or launch/probe any browser.
                            s.choose(Choice::LoginRequired, cx);
                            assert!(s.environment_unsaved(cx));
                            assert!(!s.control_enabled(Control::Guided(31), cx));
                            assert!(!s.control_enabled(Control::Guided(36), cx));
                            s.activate_control(Control::Guided(31), window, cx);
                            assert!(s.pending.is_none(), "dirty access must refuse browser dispatch");
                        }
                        _ => {}
                    }
                    s.configure_review_geometry(width);
                    cx.notify();
                });
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            if label == "fixture-dirty-access" {
                let login = cx
                    .debug_bounds("action-Guided(31)")
                    .expect("login action mounted");
                shell.update(cx, |s, cx| {
                    let top = (login.top() - s.screen_scroll.bounds().top() - px(24.))
                        .max(px(0.))
                        .min(s.screen_scroll.max_offset().height);
                    s.screen_scroll.set_offset(gpui::point(px(0.), -top));
                    cx.notify();
                });
            }
            let name = format!("workflow-{width:.0}x{height:.0}-{label}");
            capture(cx, &root, &name).unwrap();
            if matches!(
                label,
                "review" | "fixture-mixed-queue" | "approval-selected"
            ) {
                let first = shell.read_with(cx, |s, _| s.result_indices()[0]);
                workflow_review_geometry(cx, first, width, height);
                workflow_visible_action(cx, Control::Phase5(24), width, height);
            }
            if label == "approval-impact-confirm" {
                workflow_visible_action(cx, Control::Phase5(25), width, height);
                // The integrating source must expose the bounded exact impact
                // surface with this selector. A missing/offscreen preview fails
                // rather than silently accepting a correctly named wrong image.
                let impact = workflow_visible_bounds(cx, "approval-decision", width, height);
                assert!(
                    impact.size.height >= px(32.) && impact.size.width >= px(160.),
                    "exact impact must have a usable visible decision surface"
                );
            }
            if screen == Screen::Setup {
                workflow_visible_action(cx, Control::Save, width, height);
            }
            if label == "fixture-dirty-access" {
                workflow_visible_action(cx, Control::Guided(31), width, height);
                workflow_visible_bounds(cx, "environment-login-hint", width, height);
            }
            if label == "fixture-candidate-inspect-save" {
                workflow_visible_action(cx, Control::SaveBaseline, width, height);
            }
            if label == "fixture-finite-capture-progress" {
                let track = workflow_visible_bounds(cx, "progress-track", width, height);
                let fill = workflow_visible_bounds(cx, "progress-fill", width, height);
                let fraction = f32::from(fill.size.width) / f32::from(track.size.width);
                assert!(
                    (0.45..=0.55).contains(&fraction),
                    "finite 3/6 progress must be shown, not an indeterminate fallback"
                );
            }
            manifest.push(serde_json::json!({
                "capture": name, "width": width, "height": height,
                "synthetic_project_only": true,
                "in_memory_presentation_only": label.starts_with("fixture-"),
                "real_website_capture": false,
                "baseline_mutation_dispatched": false,
                "approval_preview_from_jobs": label == "approval-impact-confirm"
            }));
            owned.join_after_ui();
        }
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        original_project,
        serde_json::to_value(store.load_project(project).unwrap()).unwrap()
    );
    assert_eq!(
        original_evidence,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    assert_eq!(manifest.len(), 34);
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join("workflow-confirmation-matrix.json"))
        .unwrap();
    serde_json::to_writer_pretty(file, &manifest).unwrap();
}

#[gpui::test]
#[ignore = "bounded project-selection renderer batch, after personal proof-image review"]
fn renderer_capture_project_selection(cx: &mut TestAppContext) {
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
    let dir = tempfile::Builder::new()
        .prefix("selection-store-")
        .tempdir_in(&root)
        .unwrap();
    let paths = AppPaths::isolated(dir.path());
    let (mut store, project_id, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let other = Project::draft(
        "A long project name for a website with several environments ".repeat(2),
        "https://example.test/",
    )
    .unwrap();
    store.save_project(&other, None).unwrap();
    drop(store);
    for (width, height) in [(1120., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, finish_jobs);
        cx.simulate_resize(size(px(width), px(height)));
        for (label, screen) in [
            ("projects", Screen::Projects),
            ("results-choose", Screen::Results),
            ("history-choose", Screen::History),
        ] {
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.navigate(screen, window, cx);
                    finish_jobs(s, cx);
                })
            });
            capture(
                cx,
                &root,
                &format!("{width:.0}x{height:.0}-selection-{label}"),
            )
            .unwrap();
        }
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.navigate(Screen::Results, window, cx);
                let index = s
                    .projects
                    .items
                    .iter()
                    .position(|p| p.id == project_id)
                    .unwrap();
                s.activate_control(Control::SelectProject(index), window, cx);
                finish_jobs(s, cx);
            })
        });
        capture(
            cx,
            &root,
            &format!("{width:.0}x{height:.0}-selection-results-selected"),
        )
        .unwrap();
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::ChooseProject, window, cx);
                finish_jobs(s, cx);
            })
        });
        capture(
            cx,
            &root,
            &format!("{width:.0}x{height:.0}-selection-results-switch"),
        )
        .unwrap();
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.navigate(Screen::Projects, window, cx);
                finish_jobs(s, cx);
            })
        });
        capture(
            cx,
            &root,
            &format!("{width:.0}x{height:.0}-selection-projects-selected"),
        )
        .unwrap();
        owned.join_after_ui();
    }
}

#[gpui::test]
#[ignore = "bounded synthetic UI renderer batch, only after personal proof-image review"]
fn renderer_capture_app_batch(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    let reviewed: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("proof-reviewed.json")).unwrap()).unwrap();
    let proof = fs::read(root.join("renderer-proof.png")).unwrap();
    assert_eq!(reviewed["personally_reviewed"], true);
    assert_eq!(reviewed["png_sha256"], hex::encode(Sha256::digest(proof)));
    cx.enable_offscreen_renderer().unwrap();
    cx.update(|cx| {
        text_input::bind(cx);
        timezone_select::bind(cx);
    });
    let dir = tempfile::Builder::new()
        .prefix("synthetic-store-")
        .tempdir_in(&root)
        .unwrap();
    let (store, fixture_project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let before_project =
        serde_json::to_value(store.load_project(fixture_project).unwrap()).unwrap();
    let before_evidence =
        serde_json::to_value(store.project_evidence(fixture_project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|window, cx| {
        window
            .text_system()
            .retain_unwrapped_layouts(2048, 8 * 1024 * 1024);
        Shell::new(cx, jobs, events)
    });
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            s.activate_control(Control::OpenProject(0), window, cx);
            finish_jobs(s, cx);
        })
    });
    for (width, height) in [(1120., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.navigate(Screen::Projects, window, cx);
                finish_jobs(s, cx);
                s.activate_control(Control::OpenProject(0), window, cx);
                finish_jobs(s, cx);
            })
        });
        for (label, screen, advanced, selector) in [
            ("projects", Screen::Projects, false, None),
            ("setup", Screen::Setup, false, None),
            (
                "setup-screenshots",
                Screen::Setup,
                false,
                Some("section-Screenshots"),
            ),
            (
                "setup-speed",
                Screen::Setup,
                false,
                Some("section-Capture speed"),
            ),
            ("setup-images", Screen::Setup, false, Some("section-Images")),
            (
                "setup-advanced",
                Screen::Setup,
                true,
                Some("field-Links away from starting addresses"),
            ),
            (
                "setup-environment",
                Screen::Setup,
                true,
                Some("section-Environment & login"),
            ),
            (
                "setup-mapping",
                Screen::Setup,
                true,
                Some("field-Source origin or section base"),
            ),
            (
                "setup-timezone",
                Screen::Setup,
                true,
                Some("field-Capture timezone"),
            ),
            ("results", Screen::Results, false, None),
            (
                "results-viewer",
                Screen::Results,
                false,
                Some("phase5-before"),
            ),
            (
                "results-analysis",
                Screen::Results,
                false,
                Some("action-Phase5(0)"),
            ),
            ("history", Screen::History, false, None),
            (
                "history-lower",
                Screen::History,
                false,
                Some("action-Phase6(8)"),
            ),
            ("settings", Screen::Settings, false, None),
            (
                "settings-lower",
                Screen::Settings,
                false,
                Some("field-Proxy server (optional)"),
            ),
        ] {
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.focus.advanced = advanced;
                    s.navigate(screen, window, cx);
                    s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                    finish_jobs(s, cx);
                })
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            if let Some(selector) = selector {
                let bounds = cx
                    .debug_bounds(selector)
                    .expect("requested UI section must be laid out");
                shell.update(cx, |s, cx| {
                    let top = (bounds.top() - s.screen_scroll.bounds().top() - px(16.))
                        .max(px(0.))
                        .min(s.screen_scroll.max_offset().height);
                    s.screen_scroll.set_offset(gpui::point(px(0.), -top));
                    cx.notify();
                });
            }
            capture(cx, &root, &format!("{width:.0}x{height:.0}-{label}")).unwrap();
            if label == "setup-timezone" {
                let value = cx.debug_bounds("timezone-value").unwrap();
                let button = cx.debug_bounds("timezone-select-button").unwrap();
                let chevron = cx.debug_bounds("timezone-chevron").unwrap();
                assert!(value.size.width > px(20.) && value.size.height > px(10.));
                assert!(value.left() >= button.left() && value.right() <= chevron.left());
                assert!(chevron.right() >= button.right() - px(16.));
                let hint = cx.debug_bounds("environment-login-hint").unwrap();
                let next = cx.debug_bounds("action-Details(Capture)").unwrap();
                assert!(
                    hint.bottom() + px(8.) <= next.top(),
                    "login hint must not overlap the next action"
                );
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        window.focus(&s.handle(Control::CrawlField(16), cx));
                        window.dispatch_action(Box::new(timezone_select::Toggle), cx);
                    })
                });
                capture(cx, &root, &format!("{width:.0}x{height:.0}-timezone-popup")).unwrap();
                cx.update(|window, cx| {
                    window.dispatch_action(Box::new(timezone_select::Close), cx)
                });
            }
        }
        cx.dispatch_action(CreateProject);
        shell.read_with(cx, |s, _| {
            assert_eq!(s.screen_scroll.offset(), gpui::point(px(0.), px(0.)))
        });
        cx.dispatch_action(SaveProject);
        capture(
            cx,
            &root,
            &format!("{width:.0}x{height:.0}-new-project-feedback"),
        )
        .unwrap();
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        before_project,
        serde_json::to_value(store.load_project(fixture_project).unwrap()).unwrap()
    );
    assert_eq!(
        before_evidence,
        serde_json::to_value(store.project_evidence(fixture_project).unwrap()).unwrap()
    );
}

#[gpui::test]
#[ignore = "bounded status/progress renderer batch, only after personal proof-image review"]
fn renderer_capture_status_progress(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    let reviewed: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("proof-reviewed.json")).unwrap()).unwrap();
    let proof = fs::read(root.join("renderer-proof.png")).unwrap();
    assert_eq!(reviewed["personally_reviewed"], true);
    assert_eq!(reviewed["png_sha256"], hex::encode(Sha256::digest(proof)));
    cx.enable_offscreen_renderer().unwrap();
    let dir = tempfile::Builder::new()
        .prefix("synthetic-status-store-")
        .tempdir_in(&root)
        .unwrap();
    let (store, _, _, _, _) = crate::approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            s.activate_control(Control::OpenProject(0), window, cx);
            finish_jobs(s, cx);
        })
    });
    for (width, height) in [(1120., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for label in ["download", "discovery", "working", "notice"] {
            shell.update(cx, |s, cx| {
                s.focus.screen = if label == "download" { Screen::Setup } else { Screen::Results };
                s.focus.busy = label != "notice";
                s.focus.capturing = s.focus.busy;
                s.focus.pausable = false;
                s.progress = match label {
                    "download" => Some((format!("Downloading private browser · {} of {} received", human_bytes(82_350_444), human_bytes(258_433_266)), 82_350_444, Some(258_433_266))),
                    "discovery" => Some(("Finding rendered links · 23 pages visited — no clicking or forms".into(), 23, None)),
                    _ => None,
                };
                s.error = None;
                s.notice = Some("Review included pages and coverage before capturing. Your active baseline is unchanged.".into());
                s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                cx.notify();
            });
            capture(cx, &root, &format!("{width:.0}x{height:.0}-status-{label}")).unwrap();
        }
    }
    owned.join_after_ui();
}

#[gpui::test]
#[ignore = "one bounded local-first UX renderer batch; isolated synthetic evidence and read-only actor commands"]
fn renderer_capture_local_first_ux(cx: &mut TestAppContext) {
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
        .prefix("local-first-synthetic-store-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let original_project = serde_json::to_value(store.load_project(project).unwrap()).unwrap();
    let original_evidence = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for label in [
            "review",
            "review-focused-evidence",
            "review-note",
            "keyboard-ignored-area",
            "review-filtered-out-draft",
            "discovery",
            "discovery-live-search",
            "history",
            "history-baselines",
            "history-storage",
            "history-reopened-check",
            "history-live-filter",
            "saved-network-settings",
            "field-validation",
            "keyboard-help",
            "capture-gaps",
            "capture-saved-baseline",
            "capture-ready",
        ] {
            let name = format!("local-first-{width:.0}x{height:.0}-{label}");
            // Complete an interrupted final batch without recapturing any
            // existing UI pixels. Only the fixture harness may change.
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
                manifest.push(serde_json::json!({"capture": name, "width": width, "height": height, "retained_from_same_final_batch": true, "native_os_input_verified": false}));
                continue;
            }
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    finish_jobs(s, cx);
                    s.activate_control(Control::SelectProject(0), window, cx);
                    finish_jobs(s, cx);
                    let screen = match label {
                        "history"
                        | "history-baselines"
                        | "history-storage"
                        | "history-reopened-check"
                        | "history-live-filter"
                        | "capture-gaps"
                        | "capture-saved-baseline"
                        | "capture-ready" => Screen::History,
                        "saved-network-settings" => Screen::Settings,
                        "field-validation" => Screen::Setup,
                        _ => Screen::Results,
                    };
                    s.navigate(screen, window, cx);
                    finish_jobs(s, cx);
                    s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                    s.configure_review_geometry(width);
                    if matches!(
                        label,
                        "capture-gaps" | "capture-saved-baseline" | "capture-ready"
                    ) {
                        let baseline = s.evidence.baseline.as_ref().unwrap().clone();
                        let run = baseline.references[0].run;
                        let index = s
                            .history
                            .as_ref()
                            .unwrap()
                            .runs
                            .iter()
                            .position(|record| record.run.id == run)
                            .unwrap();
                        // Use the actual guarded History route, not an out-of-
                        // screen receiver invocation that silently keeps a check.
                        s.activate_control(Control::HistoryRun(index), window, cx);
                        s.activate_control(Control::Phase6(19), window, cx);
                        finish_jobs(s, cx);
                        assert_eq!(s.history_inspection, Some((project, run)));
                        assert_eq!(s.evidence.latest.as_ref().unwrap().run.id, run);
                        assert!(s.evidence.comparison.is_none());
                        assert_eq!(s.viewer_pane_indices(), &[1]);
                        assert!(s.viewer.as_ref().unwrap().request.is_none());
                        assert_eq!(
                            s.viewer.as_ref().unwrap().identity.as_ref().unwrap().run,
                            run
                        );
                        // Presentation-only capture states; the actor never saves
                        // these synthetic gaps or changes baseline activation.
                        s.history_inspection = None;
                        s.focus.baseline = label == "capture-saved-baseline";
                        s.focus.candidate = label == "capture-ready";
                        if label == "capture-saved-baseline" {
                            s.evidence.baseline = Some(baseline);
                        } else {
                            s.evidence.baseline = None;
                            s.project.as_mut().unwrap().project.active_baseline = None;
                        }
                        if label == "capture-gaps" {
                            let record = s.evidence.latest.as_mut().unwrap();
                            let mut missing = record.run.targets[0].clone();
                            missing.key.page = regressee_core::domain::PageId::new();
                            missing.outcome = TargetOutcome::Failed {
                                code: regressee_core::domain::CaptureError::Timeout,
                            };
                            missing.requested =
                                url::Url::parse("http://127.0.0.1:43210/synthetic-gap").unwrap();
                            record.run.targets.push(missing);
                            record.run.state = JobState::CompletedWithErrors;
                        }
                        s.notice = None;
                    }
                    if matches!(label, "discovery" | "discovery-live-search") {
                        let mut inventory =
                            s.evidence.latest.as_ref().unwrap().run.inventory.clone();
                        inventory.id = regressee_core::domain::InventoryId::new();
                        let template = inventory.pages[0].clone();
                        for name in ["fixture-included", "fixture-excluded", "fixture-help"] {
                            let mut page = template.clone();
                            page.id = regressee_core::domain::PageId::new();
                            page.canonical = template.canonical.join(&format!("/{name}/")).unwrap();
                            page.included = name != "fixture-excluded";
                            page.reason =
                                (!page.included).then(|| "Excluded in synthetic preview".into());
                            inventory.pages.push(page);
                        }
                        inventory.validate().unwrap();
                        let record = s.project.as_ref().unwrap();
                        s.selected_page = Some(inventory.pages[0].id);
                        s.preview = Some(DiscoveryPreview {
                            project_revision: record.revision,
                            settings: record.project.clone(),
                            inventory,
                            aliases: vec![],
                            notes: vec![
                                "Synthetic local search fixture; no website was visited.".into(),
                            ],
                            visited: 4,
                        });
                        s.focus.preview = true;
                        s.focus.candidate = false;
                        s.focus.review_surface = ReviewSurface::Pages;
                    }
                    if label == "review-focused-evidence" {
                        s.activate_control(Control::Guided(50), window, cx);
                    }
                    if label == "review-note" {
                        s.activate_control(Control::Phase5(37), window, cx);
                        s.analysis_fields[9].update(cx, |f, cx| {
                            f.set_value(
                                "The heading change is intentional; baseline approval is separate.",
                                cx,
                            )
                        });
                    }
                    if label == "keyboard-ignored-area" {
                        s.activate_control(Control::Guided(44), window, cx);
                    }
                    if label == "history-baselines" {
                        s.activate_control(Control::Phase6(17), window, cx);
                    }
                    if label == "history-storage" {
                        s.activate_control(Control::Phase6(18), window, cx);
                    }
                    if label == "history-reopened-check" {
                        s.activate_control(Control::Phase6(19), window, cx);
                        finish_jobs(s, cx);
                        assert!(s.history_inspection.is_some());
                    }
                    if label == "keyboard-help" {
                        s.show_keyboard_help(window, cx);
                    }
                    if label == "field-validation" {
                        s.focus.setup_stage = SetupStage::Website;
                        s.name.update(cx, |input, cx| input.set_value("", cx));
                        s.address.update(cx, |input, cx| {
                            input.set_value("missing-scheme.example", cx)
                        });
                        s.activate_control(Control::Save, window, cx);
                        assert!(s.pending.is_none(), "invalid draft must not reach storage");
                    }
                    cx.notify();
                });
            });
            // Flush actual uploads before any subsequent presentation fixture
            // retires images. This is not an additional inspection screenshot.
            cx.update(|window, cx| {
                let _ = window.draw(cx);
                window.capture_offscreen_frame().unwrap();
            });
            if label == "keyboard-ignored-area" {
                // As in real input, Tools must have a painted viewport before
                // creating the area and revealing its initial keyboard focus.
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        s.activate_control(Control::Phase5(34), window, cx)
                    });
                });
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                cx.run_until_parked();
            }
            if label == "saved-network-settings" {
                let network = cx.debug_bounds("managed-network-section").unwrap();
                shell.update(cx, |s, cx| {
                    assert!(s.network_status.is_some());
                    assert!(s.field_enabled(17) && s.field_enabled(18));
                    s.screen_scroll
                        .set_offset(gpui::point(px(0.), px(150.) - network.top()));
                    cx.notify();
                });
            }
            let inspected = shell.read_with(cx, |s, _| s.selected_key.clone());
            shell.update(cx, |s, cx| match label {
                "review-filtered-out-draft" => {
                    // Removing the last ignored rectangle is a valid empty but
                    // dirty draft; typing must not discard that draft or pair.
                    s.mask_dirty = true;
                    s.crawl_fields[9].update(cx, |input, cx| {
                        input.set_value("no-matching-synthetic-page", cx)
                    });
                }
                "discovery-live-search" => {
                    s.crawl_fields[9]
                        .update(cx, |input, cx| input.set_value("fixture-included", cx));
                }
                "history-live-filter" => {
                    s.history_search
                        .update(cx, |input, cx| input.set_value("comparison", cx));
                    s.history_kind = 1;
                    cx.notify();
                }
                _ => (),
            });
            cx.run_until_parked();
            shell.read_with(cx, |s, _| {
                if label == "review-filtered-out-draft" {
                    assert_eq!(s.search, "no-matching-synthetic-page");
                    assert!(s.result_indices().is_empty());
                    assert_eq!(s.selected_key, inspected);
                    assert!(s.mask_dirty && s.viewer.is_some());
                    assert!(!s.focus.view_loading, "typing must not request new pixels");
                }
                if label == "discovery-live-search" {
                    assert_eq!(s.search, "fixture-included");
                    assert_eq!(s.preview_page_ids().len(), 1);
                }
                if label == "history-live-filter" {
                    assert_eq!(s.history_query, "comparison");
                    assert!(!s.history_run_indices().is_empty());
                    assert!(s.history_run_indices().iter().all(|i| matches!(
                        s.history.as_ref().unwrap().runs[*i].run.intent,
                        RunIntent::Check { .. }
                    )));
                }
            });
            let name = format!("local-first-{width:.0}x{height:.0}-{label}");
            capture(cx, &root, &name).unwrap();
            if label == "keyboard-ignored-area" {
                cx.update(|window, cx| {
                    shell.read_with(cx, |s, cx| {
                        assert!(s.mask_dirty && s.mask_selected.is_some());
                        assert!(s.handle(Control::AnalysisField(10), cx).is_focused(window));
                        let field = s.analysis_fields[10].read(cx).rendered_bounds().unwrap();
                        let viewport = s.review_details_scroll.bounds();
                        assert!(
                            field.top() >= viewport.top() && field.bottom() <= viewport.bottom()
                        );
                    });
                });
                for pane in ["phase5-before", "phase5-after"] {
                    assert!(cx.debug_bounds(pane).unwrap().size.height >= px(96.));
                }
            }
            if matches!(
                label,
                "history" | "history-live-filter" | "history-baselines"
            ) {
                let (list, row) = shell.read_with(cx, |s, _| {
                    if label == "history-baselines" {
                        assert_eq!(s.history_index, 0);
                        ("history-baseline-list", "history-row-HistoryBaseline(0)")
                    } else {
                        assert_eq!(s.run_history_index, 0);
                        ("history-run-list", "history-row-HistoryRun(0)")
                    }
                });
                assert!(
                    f32::from(
                        cx.debug_bounds(list).unwrap().size.width
                            - cx.debug_bounds(row).unwrap().size.width
                    )
                    .abs()
                        <= 1.
                );
            }
            if label == "review-filtered-out-draft" {
                assert!(
                    cx.debug_bounds("review-empty-message").unwrap().bottom()
                        <= cx.debug_bounds("review-queue").unwrap().bottom()
                );
            }
            if label == "keyboard-help" {
                assert!(
                    cx.debug_bounds("information-dialog-copy").unwrap().bottom()
                        <= cx.debug_bounds("information-dialog-body").unwrap().bottom()
                );
            }
            if label.starts_with("capture-") {
                assert!(cx.debug_bounds("phase5-before").is_none());
                assert!(cx.debug_bounds("phase5-difference").is_none());
                assert!(cx.debug_bounds("ordinary-review-decisions").is_none());
                assert!(cx.debug_bounds("phase5-after").unwrap().size.height >= px(96.));
            }
            manifest.push(serde_json::json!({
                "capture": name, "width": width, "height": height,
                "synthetic_project_only": true, "website_visited": false,
                "mutation_dispatched": false, "input_model_verified": true,
                "layout_assertions_verified": true,
                "scrolled_to_inspected_section": label == "saved-network-settings",
                "native_os_input_verified": false
            }));
            owned.join_after_ui();
        }
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        original_project,
        serde_json::to_value(store.load_project(project).unwrap()).unwrap()
    );
    assert_eq!(
        original_evidence,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    assert_eq!(manifest.len(), 36);
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join("local-first-ux-matrix.json"))
        .unwrap();
    serde_json::to_writer_pretty(file, &manifest).unwrap();
}

#[gpui::test]
#[ignore = "bounded capture-only baseline-state renderer batch; synthetic local evidence, no website visits or activation"]
fn renderer_capture_baseline_states(cx: &mut TestAppContext) {
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
    let dir = tempfile::Builder::new()
        .prefix("synthetic-baseline-states-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let record = store.load_project(project).unwrap();
    let baseline = store.project_evidence(project).unwrap().baseline.unwrap();
    let run = store.load_run(baseline.references[0].run).unwrap();
    let image = &run.all_captures().next().unwrap().image;
    let tiles: Vec<_> = image
        .tiles
        .iter()
        .take(3)
        .map(|id| regressee_app::images::viewer_tile(&store, id).unwrap())
        .collect();
    let dimensions = (image.width, image.height);
    drop(store);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, finish_jobs);
        cx.simulate_resize(size(px(width), px(height)));
        for label in [
            "incomplete",
            "failed-target",
            "ready-to-save",
            "saved-baseline",
        ] {
            shell.update(cx, |s, cx| {
                // Presentation clones only. Source pixels are original baseline
                // tiles, never computed difference/heatmap pixels. No actor
                // activation, network operation or persisted evidence edit.
                s.project = Some(record.clone());
                s.project.as_mut().unwrap().project.active_baseline = None;
                s.evidence = ProjectEvidence {
                    latest: Some(run.clone()),
                    ..Default::default()
                };
                s.focus.screen = Screen::Results;
                s.focus.project_saved = true;
                s.focus.review_surface = ReviewSurface::Changes;
                s.focus.baseline = label == "saved-baseline";
                s.focus.candidate = label == "ready-to-save";
                s.focus.viewer = true;
                s.viewer_mode = 2; // Stale overlay mode must not leak into capture-only UI.
                s.viewer_fit = true;
                s.selected_key = Some(run.run.targets[0].key.clone());
                s.viewer = Some(regressee_app::jobs::ViewerFrame {
                    tile: 0,
                    panes: vec![vec![], tiles.clone(), vec![]],
                    width: dimensions.0,
                    height: dimensions.1,
                    request: None,
                    analysis: None,
                    identity: None,
                    before_environment: None,
                    before_capture: None,
                });
                if matches!(label, "incomplete" | "failed-target") {
                    let r = s.evidence.latest.as_mut().unwrap();
                    let mut failed = r.run.targets[0].clone();
                    failed.key.page = regressee_core::domain::PageId::new();
                    failed.requested =
                        url::Url::parse("http://127.0.0.1:43210/unavailable-page").unwrap();
                    failed.outcome = TargetOutcome::Failed {
                        code: regressee_core::domain::CaptureError::Timeout,
                    };
                    if label == "failed-target" {
                        s.selected_key = Some(failed.key.clone());
                        s.viewer.as_mut().unwrap().panes[1].clear();
                    }
                    r.run.targets.push(failed);
                    r.run.state = JobState::CompletedWithErrors;
                } else if label == "saved-baseline" {
                    s.evidence.baseline = Some(baseline.clone());
                    s.project.as_mut().unwrap().project.active_baseline = Some(baseline.id);
                }
                s.notice = None;
                s.error = None;
                s.configure_review_geometry(width);
                cx.notify();
            });
            capture(
                cx,
                &root,
                &format!("baseline-{width:.0}x{height:.0}-{label}"),
            )
            .unwrap();
            assert!(cx.debug_bounds("phase5-before").is_none());
            assert!(cx.debug_bounds("phase5-difference").is_none());
            assert!(cx.debug_bounds("phase5-after").unwrap().size.height >= px(64.));
            assert!(
                cx.debug_bounds("capture-next-step").unwrap().bottom()
                    <= cx.debug_bounds("review-queue").unwrap().top()
            );
            assert!(cx.debug_bounds("review-queue").unwrap().size.height >= px(PAGE_ROW_HEIGHT));
        }
        owned.join_after_ui();
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
#[ignore = "bounded results-list renderer batch using in-memory synthetic targets only"]
fn renderer_capture_results_rows(cx: &mut TestAppContext) {
    use regressee_core::domain::{InventoryPage, PageId, ProfileId};
    let root = output_root().unwrap();
    let reviewed: serde_json::Value =
        serde_json::from_slice(&fs::read(root.join("proof-reviewed.json")).unwrap()).unwrap();
    let proof = fs::read(root.join("renderer-proof.png")).unwrap();
    assert_eq!(reviewed["personally_reviewed"], true);
    assert_eq!(reviewed["png_sha256"], hex::encode(Sha256::digest(proof)));
    cx.enable_offscreen_renderer().unwrap();
    let dir = tempfile::Builder::new()
        .prefix("synthetic-rows-store-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        finish_jobs(s, cx);
        s.focus.screen = Screen::Results;
        s.queue(Work::Load(project), cx);
        finish_jobs(s, cx);
        s.queue(Work::Evidence(project), cx);
        finish_jobs(s, cx);
        s.clear_viewer();
        s.focus.viewer = false;
        s.evidence.comparison = None;
        let r = s.evidence.latest.as_mut().unwrap();
        let template = r.run.targets[0].clone();
        let mut desktop = r.run.profiles[0].clone();
        desktop.name = "Desktop".into();
        let mut mobile = desktop.clone();
        mobile.id = ProfileId::new();
        mobile.name = "Mobile".into();
        r.run.profiles = vec![desktop, mobile];
        r.run.inventory.pages = [
            "https://example.test/".to_owned(),
            "https://example.test/articles/quad-tree-canvas/".to_owned(),
            format!(
                "https://example.test/articles/{}?view=all",
                "a-long-page-name-".repeat(20)
            ),
        ]
        .into_iter()
        .map(|address| InventoryPage {
            id: PageId::new(),
            canonical: url::Url::parse(&address).unwrap(),
            included: true,
            reason: None,
        })
        .collect();
        r.run.targets = r
            .run
            .inventory
            .pages
            .iter()
            .flat_map(|page| {
                r.run.profiles.iter().map(|profile| {
                    let mut target = template.clone();
                    target.key = TargetKey {
                        page: page.id,
                        profile: profile.id,
                    };
                    target.requested = page.canonical.clone();
                    target
                })
            })
            .collect();
        s.selected_key = Some(r.run.targets[1].key.clone());
        s.error = None;
        s.notice = None;
        cx.notify();
    });
    let inventory = shell.read_with(cx, |s, _| {
        s.evidence.latest.as_ref().unwrap().run.inventory.clone()
    });
    for (width, height) in [(1120., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for preview in [false, true] {
            shell.update(cx, |s, cx| {
                s.preview = preview.then(|| DiscoveryPreview {
                    project_revision: s.project.as_ref().unwrap().revision,
                    settings: s.project.as_ref().unwrap().project.clone(),
                    inventory: inventory.clone(),
                    aliases: vec![],
                    notes: vec![],
                    visited: 3,
                });
                s.focus.preview = preview;
                s.selected_page = Some(inventory.pages[1].id);
                s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                cx.notify();
            });
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let list = cx.debug_bounds("page-list").unwrap();
            shell.update(cx, |s, cx| {
                let top = (list.top()
                    - s.screen_scroll.bounds().top()
                    - px(if preview { 190. } else { 160. }))
                .max(px(0.))
                .min(s.screen_scroll.max_offset().height);
                s.screen_scroll.set_offset(gpui::point(px(0.), -top));
                cx.notify();
            });
            let label = if preview {
                "discovery-rows"
            } else {
                "results-rows"
            };
            capture(cx, &root, &format!("{width:.0}x{height:.0}-{label}")).unwrap();
        }
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
