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
    ensure!(root.is_absolute() && root.starts_with("/path/to/private-tests/opencode"));
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

fn finish_jobs(shell: &mut Shell, cx: &mut Context<Shell>) {
    while shell.pending.is_some() {
        let update = shell.events.recv_timeout(Duration::from_secs(5)).unwrap();
        assert_eq!(shell.pending, Some(update.id));
        assert!(
            !matches!(update.reply, Reply::Failure(_)),
            "{:?}",
            update.reply
        );
        shell.apply_update(update, cx);
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
    let (jobs, events) = Jobs::start(AppPaths::isolated(dir.path()));
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
