// SPDX-License-Identifier: MPL-2.0
//! Finite note-only native evidence. Constructed synthetic states, real native
//! controller transitions, storage replies and painted geometry; no OS claim.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

fn geometry(b: Bounds<Pixels>) -> serde_json::Value {
    serde_json::json!({"x":f32::from(b.left()),"y":f32::from(b.top()),"width":f32::from(b.size.width),"height":f32::from(b.size.height)})
}

#[gpui::test]
#[ignore = "explicit two-size Phase 3 note render batch; synthetic/offline only"]
fn renderer_capture_phase3(cx: &mut TestAppContext) {
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
        cx.bind_keys([
            KeyBinding::new("escape", Dismiss, Some("Shell")),
            KeyBinding::new("tab", Tab, Some("Shell")),
            KeyBinding::new("shift-tab", TabPrev, Some("Shell")),
        ]);
    });
    let dir = tempfile::Builder::new()
        .prefix("phase3-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = super::ui_review_matrix::fixture_named(&root, dir.path(), true);
    let evidence = store.project_evidence(project).unwrap();
    let key = evidence.latest.as_ref().unwrap().run.targets[0].key.clone();
    let baseline = evidence.baseline.as_ref().unwrap().id;
    drop(store);
    let short =
        "Autumn headline and CTA are intentional. Product grid needs release-owner confirmation.";
    let multiline = "Headline and CTA are intentional.\n\nProduct grid needs release-owner confirmation.\nCheck delivery copy before the release.\nOwner: Ingrid · café / 日本語 / 🦀";
    let long = (1..=35).map(|n| format!("Review {n}: headline is intentional; product grid needs release-owner confirmation.\n")).collect::<String>();
    let mut ledger = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        for scene in [
            "two-sentences",
            "wrapped-selection",
            "newline-paste",
            "long-top",
            "long-scrolled",
            "limit-feedback",
            "legacy-invalid",
            "saved",
            "closed-draft",
            "reopened-draft",
        ] {
            if std::env::var_os("REGRESSEE_RENDERER_CLARITY").is_some()
                && !matches!(
                    scene,
                    "two-sentences"
                        | "limit-feedback"
                        | "legacy-invalid"
                        | "saved"
                        | "closed-draft"
                        | "reopened-draft"
                )
            {
                continue;
            }
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| {
                    s.review_note_open = false;
                    s.review_drafts = Default::default();
                    s.analysis_fields[9].update(cx, |i, cx| i.set_value("", cx));
                    load_review(s, project, cx);
                    s.selected_key = Some(key.clone());
                    s.sync_review_note(cx);
                    s.error = None;
                    s.notice = None;
                    s.review_details_scroll
                        .set_offset(gpui::point(px(0.), px(0.)));
                    s.activate_control(Control::Phase5(37), w, cx);
                    s.analysis_fields[9].update(cx, |i, cx| {
                        i.set_value(
                            if scene.starts_with("long") {
                                &long
                            } else {
                                short
                            },
                            cx,
                        );
                        if scene == "newline-paste" {
                            i.set_value("", cx);
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                multiline.into(),
                            ));
                        }
                        if scene == "limit-feedback" {
                            let note = format!(
                                "{}{}",
                                short,
                                " Release-owner confirmation is pending.".repeat(300)
                            );
                            let padding = 8192 - short.len();
                            // This constructed boundary draft is exact UTF-8 bytes;
                            // rejection below is an actual native clipboard action.
                            let prefix = note.get(..8192).unwrap();
                            assert_eq!(prefix.len(), short.len() + padding);
                            i.set_value(prefix, cx);
                            cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                                "\nPlease also confirm delivery copy.".into(),
                            ));
                        }
                        if scene == "legacy-invalid" {
                            i.set_value(
                                &format!("{}{}", short, " Legacy release note.".repeat(500)),
                                cx,
                            );
                        }
                    });
                    if scene == "saved" {
                        s.activate_control(Control::Phase5(32), w, cx);
                        finish_jobs(s, cx);
                        assert!(!s.review_note_dirty(cx));
                    }
                    cx.notify();
                })
            });
            cx.run_until_parked();
            if matches!(scene, "newline-paste" | "limit-feedback") {
                cx.simulate_keystrokes("ctrl-v");
            }
            cx.simulate_keystrokes(if scene == "long-scrolled" {
                "ctrl-end"
            } else {
                "ctrl-home"
            });
            cx.run_until_parked();
            if scene == "wrapped-selection" {
                cx.simulate_keystrokes("shift-down shift-down");
                cx.run_until_parked();
            }
            if matches!(scene, "closed-draft" | "reopened-draft") {
                // Force a dirty draft even after the earlier saved scene.
                cx.simulate_keystrokes("ctrl-end enter");
                cx.simulate_input("Confirm delivery copy before release.");
                cx.simulate_keystrokes("escape");
                if scene == "reopened-draft" {
                    cx.update(|w, cx| {
                        shell.update(cx, |s, cx| s.activate_control(Control::Phase5(37), w, cx))
                    });
                }
                cx.run_until_parked();
            }
            let label = format!("phase3-{width:.0}x{height:.0}-{scene}");
            capture(cx, &root, &label).unwrap();
            let (
                input_bounds,
                status,
                text_bytes,
                text_hash,
                scroll,
                editor_overflow,
                body_overflow,
                dirty,
            ) = shell.read_with(cx, |s, cx| {
                let input = s.analysis_fields[9].read(cx);
                assert_eq!(s.selected_key.as_ref(), Some(&key));
                assert_eq!(s.evidence.baseline.as_ref().unwrap().id, baseline);
                if scene == "newline-paste" {
                    assert_eq!(input.content.as_ref(), multiline);
                }
                if scene == "limit-feedback" {
                    assert_eq!(input.content.len(), 8192);
                    assert!(input.correction.as_ref().unwrap().contains("paste less"));
                }
                if scene == "legacy-invalid" {
                    assert!(input.content.len() > 8192);
                    assert!(s.note_status(cx).1);
                }
                if scene == "long-scrolled" {
                    assert!(input.multiline_scroll_offset().unwrap() > px(0.));
                }
                if s.review_note_open {
                    for control in [
                        Control::Phase5(21),
                        Control::Phase5(23),
                        Control::Phase5(24),
                        Control::Phase5(33),
                    ] {
                        assert!(!s.control_enabled(control, cx));
                    }
                } else {
                    assert!(
                        s.review_blocking_reason(cx)
                            .unwrap()
                            .contains("Review note")
                    );
                }
                (
                    input.rendered_bounds().unwrap(),
                    s.note_status(cx),
                    input.content.len(),
                    hex::encode(Sha256::digest(input.content.as_bytes())),
                    f32::from(input.multiline_scroll_offset().unwrap()),
                    input.multiline_overflows(),
                    f32::from(s.review_details_scroll.max_offset().height),
                    s.review_note_dirty(cx),
                )
            });
            let panel = cx.debug_bounds("viewer-inspector");
            if scene != "closed-draft" {
                let border = cx.debug_bounds("review-note-input-border").unwrap();
                let save = cx.debug_bounds("action-Phase5(32)").unwrap();
                let status_bounds = cx.debug_bounds("review-note-status").unwrap();
                crate::phase2_evidence_tests::inside(save, panel.unwrap());
                crate::phase2_evidence_tests::inside(status_bounds, panel.unwrap());
                assert_eq!(border.size.height, px(text_input::REVIEW_NOTE_HEIGHT));
                shell.read_with(cx, |s, _| {
                    crate::phase2_evidence_tests::inside(
                        input_bounds,
                        s.review_details_scroll.bounds(),
                    )
                });
            }
            ledger.push(serde_json::json!({"label":label,"width":width,"height":height,"input_bounds":geometry(input_bounds),"panel_bounds":panel.map(geometry),"status":status.0,"invalid_or_rejected":status.1,"text_bytes":text_bytes,"text_sha256":text_hash,"scroll_y":scroll,"editor_overflow":editor_overflow,"body_overflow":body_overflow,"dirty":dirty,"selected_target_retained":true,"baseline_unchanged":true}));
        }
        owned.join_after_ui();
    }
    fs::write(
        root.join("phase3-matrix.json"),
        serde_json::to_vec_pretty(&ledger).unwrap(),
    )
    .unwrap();
}
