// SPDX-License-Identifier: MPL-2.0
//! Finite native motion evidence. No browser, user project, or worker mutation.
//!
//! Run once in `inspection/`, then at most once in `confirmation/` after batching
//! findings. This is sampled Linux GPUI/Blade evidence, not OS-input, accessibility,
//! frame-pacing, or cross-platform qualification. Settings choices use the actual
//! asynchronous preference owner and are checked again after a fresh shell starts.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use std::time::Instant;

const SCENES: [&str; 6] = [
    "control-hover",
    "feedback",
    "menu",
    "inspector",
    "dialog",
    "completion",
];

struct Sample {
    phase: &'static str,
    requested_ms: u64,
    draw_started_ms: f64,
    readback_finished_ms: f64,
    frame: gpui::OffscreenFrame,
    cpu_ms: f64,
    gpu_ms: f64,
    pane: Option<Bounds<Pixels>>,
}

// Shared motion uses the GPUI executor clock. Advance only the bounded requested
// interval; do not sleep or drain a possibly repeating activity animation.
fn sample(
    cx: &mut VisualTestContext,
    origin: Instant,
    phase: &'static str,
    requested_ms: u64,
) -> Sample {
    let target = Duration::from_millis(requested_ms);
    let now = cx.update(|_, cx| cx.background_executor().now());
    if let Some(advance) = target.checked_sub(now.saturating_duration_since(origin)) {
        cx.executor().advance_clock(advance);
    }
    let (frame, cpu_ms, gpu_ms, draw_started_ms, readback_finished_ms) = cx.update(|window, cx| {
        window.refresh();
        let started = Instant::now();
        let draw_started_ms = cx
            .background_executor()
            .now()
            .saturating_duration_since(origin)
            .as_secs_f64()
            * 1000.;
        let _ = window.draw(cx);
        let cpu_ms = started.elapsed().as_secs_f64() * 1000.;
        let started = Instant::now();
        let frame = window.capture_offscreen_frame().unwrap();
        let gpu_ms = started.elapsed().as_secs_f64() * 1000.;
        assert_eq!(frame.width, f32::from(window.viewport_size().width) as u32);
        assert_eq!(
            frame.height,
            f32::from(window.viewport_size().height) as u32
        );
        (
            frame,
            cpu_ms,
            gpu_ms,
            draw_started_ms,
            cx.background_executor()
                .now()
                .saturating_duration_since(origin)
                .as_secs_f64()
                * 1000.,
        )
    });
    Sample {
        phase,
        requested_ms,
        draw_started_ms,
        readback_finished_ms,
        frame,
        cpu_ms,
        gpu_ms,
        pane: cx.debug_bounds("phase5-before"),
    }
}

fn pane_hash(sample: &Sample) -> Option<String> {
    let pane = sample.pane?;
    // Exclude only the 1px frame boundary and its adjacent antialiasing pixel.
    // Hash every interior pixel, not a selected strip or recolored image.
    let left = (f32::from(pane.left()).ceil() as u32 + 2).min(sample.frame.width);
    let top = (f32::from(pane.top()).ceil() as u32 + 2).min(sample.frame.height);
    let right = (f32::from(pane.right()).floor() as u32)
        .saturating_sub(2)
        .min(sample.frame.width);
    let bottom = (f32::from(pane.bottom()).floor() as u32)
        .saturating_sub(2)
        .min(sample.frame.height);
    assert!(
        right > left && bottom > top,
        "original stage must be visible"
    );
    let mut hash = Sha256::new();
    for y in top..bottom {
        let start = ((y * sample.frame.width + left) * 4) as usize;
        let end = ((y * sample.frame.width + right) * 4) as usize;
        hash.update(&sample.frame.rgba[start..end]);
    }
    Some(hex::encode(hash.finalize()))
}

fn retained_state(shell: &Entity<Shell>, cx: &mut VisualTestContext) -> serde_json::Value {
    shell.read_with(cx, |s, cx| {
        serde_json::json!({
            "selected_key": s.selected_key,
            "zoom": format!("{:?}", s.zoom),
            "pan": [s.pan_x, s.pan_y],
            "viewer_fit": s.viewer_fit,
            "review_note": s.analysis_fields[9].read(cx).content.to_string(),
            "ignored_regions": s.mask_rectangles,
            "name_draft": s.name.read(cx).content.to_string(),
            "project": s.project,
        })
    })
}

fn changed_pixels(a: &Sample, b: &Sample) -> usize {
    assert_eq!(
        (a.frame.width, a.frame.height),
        (b.frame.width, b.frame.height)
    );
    a.frame
        .rgba
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.frame.rgba.as_chunks::<4>().0.iter())
        .filter(|(a, b)| a != b)
        .count()
}

fn create_receipt(root: &Path, name: &str, value: &serde_json::Value) {
    let file = fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(root.join(name))
        .expect("one inspection and at most one confirmation; never overwrite evidence");
    serde_json::to_writer_pretty(file, value).unwrap();
}

#[allow(clippy::too_many_arguments)]
fn capture_settings_restart(
    cx: &mut TestAppContext,
    root: &Path,
    path: &Path,
    store: &regressee_storage::Store,
    project: regressee_core::domain::ProjectId,
    preference: AppAppearance,
    width: f32,
    height: f32,
    manifest: &mut Vec<serde_json::Value>,
) {
    let paths = AppPaths::isolated(path);
    regressee_storage::AppPreferences {
        appearance: preference,
        reduced_motion: false,
    }
    .save(&paths)
    .unwrap();
    for restarted in [false, true] {
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        assert_eq!(owned.appearance.wait_loaded().reduced_motion, restarted);
        let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        vcx.simulate_resize(size(px(width), px(height)));
        shell.update(vcx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Settings;
            s.notice = None;
            s.error = None;
            s.summary = Some(store.summary().unwrap());
            s.runtime_readiness = Some(crate::app_settings_tests::readiness(
                regressee_app::product::RuntimeState::Ready,
                true,
            ));
            s.runtime_status = Some("Synthetic verified browser status.".into());
            s.network_status = Some("Synthetic saved network settings.".into());
            cx.notify();
        });
        vcx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.reveal_form_control(Control::ReducedMotion(true), true, window, cx);
                window.focus(&s.handle(Control::ReducedMotion(true), cx));
                if !restarted {
                    s.activate_control(Control::ReducedMotion(true), window, cx);
                }
                assert!(s.appearance.reduced_motion);
                assert!(cx.global::<controls::ReducedMotion>().0);
                assert_eq!(s.appearance.preference, preference);
                assert!(s.pending.is_none() && s.task.is_none());
                cx.notify();
            });
            let _ = window.draw(cx);
        });
        let choice = vcx.debug_bounds("choice-ReducedMotion(true)").unwrap();
        crate::phase1_setup_tests::inside(choice, vcx.debug_bounds("screen-scroll").unwrap());
        if restarted {
            let label = format!(
                "motion-{}-{width:.0}x{height:.0}-settings-restarted",
                preference.label().to_lowercase()
            );
            capture(vcx, root, &label).unwrap();
            manifest.push(serde_json::json!({"scene":"settings-restarted","label":label,"width":width,"height":height,"appearance":preference,"reduced_motion":true,"actual_ui_toggle_and_restart":true}));
        }
        shell.update(vcx, finish_jobs);
        vcx.update(|window, _| window.remove_window());
        owned.join_after_ui();
        let saved = regressee_storage::AppPreferences::load(&paths).unwrap();
        assert!(saved.reduced_motion);
        assert_eq!(saved.appearance, preference);
    }
}

#[gpui::test]
#[ignore = "one finite two-size light/dark native motion inspection; synthetic local data only"]
fn renderer_capture_native_motion(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    assert!(matches!(
        root.file_name().and_then(|s| s.to_str()),
        Some("inspection" | "confirmation")
    ));
    create_receipt(
        &root,
        "motion-round-started.json",
        &serde_json::json!({"scenes":SCENES,"maximum_frames":100,"settings_integration":"actual UI toggle, flush, restart"}),
    );
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    cx.update(regressee_gui::ui::dialog::install);
    let dir = tempfile::Builder::new()
        .prefix("motion-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let frozen_project = serde_json::to_value(store.load_project(project).unwrap()).unwrap();
    let evidence = store.project_evidence(project).unwrap();
    let frozen_evidence = serde_json::to_value(&evidence).unwrap();
    let originals: Vec<_> = evidence
        .latest
        .iter()
        .flat_map(|run| &run.captures)
        .chain(evidence.before.iter())
        .map(|capture| {
            let path = store.artifact_path(&capture.image.original).unwrap();
            let hash = hex::encode(Sha256::digest(fs::read(&path).unwrap()));
            (path, hash)
        })
        .collect();
    create_receipt(
        &root,
        "motion-fixture-frozen.json",
        &serde_json::json!({
            "project":frozen_project,"evidence":frozen_evidence,"original_files":originals,
            "synthetic_store":dir.path(),"source":"existing offline approval_fixture; no browser visits",
        }),
    );
    let mut manifest = Vec::new();
    let mut findings = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for preference in [AppAppearance::Light, AppAppearance::Dark] {
            for reduced in [false, true] {
                for scene in SCENES {
                    // Sample actual feedback wiring as well as modality and the
                    // authored outcome; other reduced types have primitive tests.
                    if reduced && !matches!(scene, "feedback" | "dialog" | "completion") {
                        continue;
                    }
                    regressee_storage::AppPreferences {
                        appearance: preference,
                        reduced_motion: reduced,
                    }
                    .save(&AppPaths::isolated(dir.path()))
                    .unwrap();
                    let (jobs, events) =
                        Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
                    let owned = jobs.clone();
                    owned.appearance.wait_loaded();
                    let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                    vcx.simulate_resize(size(px(width), px(height)));
                    shell.update(vcx, |s, cx| {
                        load_review(s, project, cx);
                        s.error = None;
                        s.notice = None;
                        s.navigation_preference = Some(false);
                        s.configure_review_geometry(width);
                        s.forced_reduced_motion = true;
                        if scene == "completion" {
                            // Presentation-only running task. Nothing is dispatched.
                            s.next_request += 1;
                            s.pending = Some(s.next_request);
                            s.focus.busy = true;
                            s.begin_task(
                                regressee_gui::task::Spec::for_work(&Work::CaptureInventory {
                                    project,
                                    preview: None,
                                    check: true,
                                    resume: None,
                                })
                                .unwrap(),
                            );
                        }
                        cx.notify();
                    });
                    // Decode embedded images and settle initial reads with activity
                    // static; do NOT run_until_parked inside the timed sequence.
                    vcx.update(|window, cx| {
                        cx.set_global(controls::ReducedMotion(true));
                        let _ = window.draw(cx);
                    });
                    vcx.run_until_parked();
                    let before_clock = vcx.update(|_, cx| cx.background_executor().now());
                    let mut samples = vec![sample(vcx, before_clock, "before", 0)];
                    let retained = retained_state(&shell, vcx);
                    shell.update(vcx, |s, cx| {
                        s.forced_reduced_motion = false;
                        assert_eq!(s.appearance.reduced_motion, reduced);
                        cx.set_global(controls::ReducedMotion(reduced));
                    });
                    let origin = vcx.update(|_, cx| cx.background_executor().now());
                    if scene == "control-hover" {
                        let button = vcx.debug_bounds("action-Guided(54)").unwrap();
                        vcx.simulate_mouse_move(button.center(), None, gpui::Modifiers::none());
                    } else {
                        vcx.update(|window, cx| shell.update(cx, |s, cx| {
                            match scene {
                                "feedback" => s.notice = Some("Synthetic review feedback. The saved originals and baseline are unchanged.".into()),
                                "menu" => s.activate_control(Control::Guided(54), window, cx),
                                "inspector" => s.activate_control(Control::Phase5(36), window, cx),
                                "dialog" => s.show_keyboard_help(window, cx),
                                "completion" => {
                                    let request = s.next_request;
                                    s.finish_task(request, &Reply::Evidence(Box::new(s.evidence.clone())));
                                    s.pending = None;
                                    s.focus.busy = false;
                                    s.focus.capturing = false;
                                    if !s.control_enabled(Control::TaskContinue, cx) {
                                        findings.push("completion: Continue was not immediately available".to_string());
                                    }
                                }
                                _ => unreachable!(),
                            }
                            cx.notify();
                        }));
                    }
                    samples.push(sample(vcx, origin, "initial", 0));
                    if !reduced {
                        samples.push(sample(vcx, origin, "intermediate", 40));
                    }
                    samples.push(sample(vcx, origin, "settled", 220));
                    let selector = match scene {
                        "control-hover" => "action-Guided(54)",
                        "feedback" => "feedback-surface",
                        "menu" => "viewer-mode-chooser",
                        "inspector" => "viewer-inspector",
                        "dialog" => "information-dialog",
                        "completion" => "task-actions",
                        _ => unreachable!(),
                    };
                    let surface = vcx.debug_bounds(selector);
                    if !surface.is_some_and(|b| {
                        b.left() >= px(0.)
                            && b.right() <= px(width)
                            && b.top() >= px(0.)
                            && b.bottom() <= px(height)
                    }) {
                        findings.push(format!("{scene} {width}x{height} {preference:?} reduced={reduced}: missing/out-of-window {selector}: {surface:?}"));
                    }
                    if retained_state(&shell, vcx) != retained {
                        findings.push(format!("{scene}: presentation changed evidence or drafts"));
                    }
                    if scene == "control-hover" {
                        if samples[0].pane.is_none() {
                            findings.push("hover: original screenshot stage missing".to_string());
                        }
                        for frame in &samples[1..] {
                            if frame.pane != samples[0].pane {
                                findings.push(format!(
                                    "hover {}: screenshot geometry changed",
                                    frame.phase
                                ));
                            }
                            if pane_hash(frame) != pane_hash(&samples[0]) {
                                findings.push(format!(
                                    "hover {}: original-stage pixels changed",
                                    frame.phase
                                ));
                            }
                        }
                    }
                    if scene == "feedback" {
                        // Mounting the existing banner applies its final layout
                        // immediately. Its paint must not subsequently reflow or
                        // fade the non-occluded screenshot stage underneath.
                        for frame in &samples[2..] {
                            if frame.pane != samples[1].pane
                                || pane_hash(frame) != pane_hash(&samples[1])
                            {
                                findings.push(format!(
                                    "feedback {}: screenshot stage changed during feedback paint",
                                    frame.phase
                                ));
                            }
                        }
                    }
                    let prefix = format!(
                        "motion-{}-{width:.0}x{height:.0}-{}-{scene}",
                        preference.label().to_lowercase(),
                        if reduced { "reduced" } else { "full" }
                    );
                    let settled = samples.last().unwrap();
                    let meaningful_intermediates = samples
                        .iter()
                        .filter(|frame| {
                            frame.phase.starts_with("intermediate")
                                && changed_pixels(frame, &samples[1]) > 0
                                && changed_pixels(frame, settled) > 0
                        })
                        .count();
                    if !reduced && meaningful_intermediates != 1 {
                        findings.push(format!("{prefix}: expected a distinct intermediate frame between initial and settled paint"));
                    }
                    if reduced && changed_pixels(&samples[1], settled) != 0 {
                        findings.push(format!("{prefix}: reduced motion changed pixels between initial and settled paint"));
                    }
                    // Before remains in memory for full pixel/geometry checks.
                    // Do not encode redundant before frames: their budget now
                    // covers the actual reduced-feedback integration in all four
                    // desktop contexts, keeping the finite batch at 100 PNGs.
                    let rows: Vec<_> = samples.iter().enumerate().filter(|(_, frame)| frame.phase != "before").map(|(index, frame)| {
                        serde_json::json!({
                            "label": format!("{prefix}-{}",frame.phase),
                            "phase":frame.phase,"requested_ms":frame.requested_ms,
                            "draw_started_ms":frame.draw_started_ms,"readback_finished_ms":frame.readback_finished_ms,
                            "rgba_sha256":hex::encode(Sha256::digest(&frame.frame.rgba)),
                            "changed_pixels_from_previous":index.checked_sub(1).map(|i| changed_pixels(&samples[i],frame)),
                            "changed_pixels_from_settled":changed_pixels(settled,frame),
                            "original_stage_interior_sha256":pane_hash(frame),
                            "pane":frame.pane.map(|b| [f32::from(b.left()),f32::from(b.top()),f32::from(b.size.width),f32::from(b.size.height)]),
                        })
                    }).collect();
                    // Flush only after the time-sensitive frames have been acquired.
                    for frame in samples {
                        if frame.phase == "before" {
                            continue;
                        }
                        if let Err(error) = write_frame(
                            &root,
                            &format!("{prefix}-{}", frame.phase),
                            frame.frame.width,
                            frame.frame.height,
                            frame.frame.rgba,
                            frame.cpu_ms,
                            frame.gpu_ms,
                        ) {
                            findings.push(format!(
                                "{prefix}-{}: capture encoding/calibration failed: {error:#}",
                                frame.phase
                            ));
                        }
                    }
                    let receipt = serde_json::json!({"scene":scene,"width":width,"height":height,"appearance":preference,"persisted_reduced_motion":reduced,"meaningful_intermediate_frames":meaningful_intermediates,"frames":rows});
                    create_receipt(&root, &format!("{prefix}-sequence.json"), &receipt);
                    manifest.push(receipt);
                    if scene == "completion" {
                        vcx.update(|window, cx| {
                            shell.update(cx, |s, cx| {
                                if s.task.is_none() {
                                    findings.push("completion automatically continued".to_string());
                                }
                                s.activate_control(Control::TaskContinue, window, cx);
                                if s.task.is_some() {
                                    findings.push(
                                        "completion: explicit Continue did not dismiss task"
                                            .to_string(),
                                    );
                                }
                                finish_jobs(s, cx);
                            })
                        });
                    }
                    shell.update(vcx, finish_jobs);
                    owned.join_after_ui();
                    vcx.update(|window, _| window.remove_window());
                }
            }
            capture_settings_restart(
                cx,
                &root,
                dir.path(),
                &store,
                project,
                preference,
                width,
                height,
                &mut manifest,
            );
        }
    }
    assert_eq!(
        frozen_project,
        serde_json::to_value(store.load_project(project).unwrap()).unwrap()
    );
    assert_eq!(
        frozen_evidence,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    for (path, hash) in &originals {
        assert_eq!(*hash, hex::encode(Sha256::digest(fs::read(path).unwrap())));
    }
    set_dark(false);
    cx.update(|cx| cx.set_global(controls::ReducedMotion(false)));
    create_receipt(
        &root,
        "motion-manifest.json",
        &serde_json::json!({
            "sequences":manifest,"original_files":originals,
            "synthetic_store":dir.path(),
            "project_and_evidence_unchanged":true,"findings":findings,
            "settings_integration":"actual UI toggle, async flush, new owner and shell; remembered preference asserted",
            "intermediate_sampling":"bounded GPUI executor-clock samples at 0/40/220ms; CPU/GPU times are wall-clock, not frame-pacing qualification",
            "qualification":"Linux offscreen only; no OS input, accessibility, desktop motion preference or cross-platform claim",
        }),
    );
    // Keep the inspected synthetic originals and frozen receipts, not only PNGs.
    let _ = dir.keep();
    assert!(
        findings.is_empty(),
        "batched native motion findings: {findings:#?}"
    );
}
