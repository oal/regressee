// SPDX-License-Identifier: MPL-2.0
//! One opt-in batched native matrix; isolated real IPC, never OS desktop capture.
use super::*;
use crate::dom_workflow_tests::{
    Engine, finish_source, fixture, load, type_dom_note, unavailable_screenshot_fixture,
};
use regressee_core::domain::{CaptureMode, CheckKind};

#[gpui::test]
#[ignore = "explicit native DOM workflow two-size light/dark inspection"]
fn renderer_capture_dom_workflow(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let both = fixture(CaptureMode::ScreenshotsAndDomSnapshots, true);
    let only = fixture(CaptureMode::DomSnapshotsOnly, true);
    let failed = fixture(CaptureMode::ScreenshotsAndDomSnapshots, false);
    let unavailable_shot = unavailable_screenshot_fixture();
    let engines = [
        Engine::start(&both),
        Engine::start(&only),
        Engine::start(&failed),
        Engine::start(&unavailable_shot),
    ];
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for preference in [AppAppearance::Light, AppAppearance::Dark] {
            for scene in [
                "setup",
                "both",
                "dom-only",
                "failure",
                "loading",
                "history-source",
                "approval",
                "long-source",
                "note-dom-only",
                "note-unavailable-shot",
            ] {
                let (fixture, engine) = match scene {
                    "dom-only" | "note-dom-only" => (&only, &engines[1]),
                    "failure" => (&failed, &engines[2]),
                    "note-unavailable-shot" => (&unavailable_shot, &engines[3]),
                    _ => (&both, &engines[0]),
                };
                regressee_storage::AppPreferences {
                    appearance: preference,
                }
                .save(&AppPaths::isolated(fixture.root.path()))
                .unwrap();
                let (jobs, events) = engine.jobs();
                let owner = jobs.clone();
                owner.appearance.wait_loaded();
                let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                cx.simulate_resize(size(px(width), px(height)));
                cx.update(|window, cx| {
                    shell.update(cx, |s, cx| {
                        load(s, fixture, cx);
                        finish_source(s, cx);
                        s.configure_review_geometry(width);
                        match scene {
                            "setup" => {
                                s.focus.screen = Screen::Setup;
                                s.focus.setup_stage = SetupStage::Pages;
                            }
                            "loading" => {
                                let identity = s.dom.identity.clone().unwrap();
                                let token = s.dom.source.update(cx, |source, cx| {
                                    source.begin_load("explicit synthetic pending IPC", cx)
                                });
                                s.dom.frame = None;
                                s.dom.pending = Some((u64::MAX, token, identity));
                            }
                            "history-source" => {
                                s.navigate(Screen::History, window, cx);
                                crate::workflow_refinement_tests::finish_jobs(s, cx);
                                s.run_history_index = s
                                    .history
                                    .as_ref()
                                    .unwrap()
                                    .runs
                                    .iter()
                                    .position(|record| {
                                        record.run.id
                                            == fixture.evidence.latest.as_ref().unwrap().run.id
                                    })
                                    .unwrap();
                                s.activate_control(Control::Phase6(19), window, cx);
                                crate::workflow_refinement_tests::finish_jobs(s, cx);
                                if !s.dom_active() {
                                    assert!(s.control_enabled(
                                        Control::EvidenceKind(CheckKind::DomSnapshot),
                                        cx
                                    ));
                                    s.activate_control(
                                        Control::EvidenceKind(CheckKind::DomSnapshot),
                                        window,
                                        cx,
                                    );
                                }
                                finish_source(s, cx);
                                assert!(s.history_inspection.is_some() && s.dom_active());
                            }
                            "approval" => {
                                s.kind_approval_action(23, cx);
                                assert!(
                                    s.dom
                                        .selected
                                        .contains_key(s.selected_key.as_ref().unwrap())
                                );
                                s.choose_evidence_kind(CheckKind::Screenshot, cx);
                                s.kind_approval_action(23, cx);
                                assert!(!s.approval_selection.is_empty());
                                s.choose_evidence_kind(CheckKind::DomSnapshot, cx);
                                finish_source(s, cx);
                                s.kind_approval_action(24, cx);
                                crate::workflow_refinement_tests::finish_jobs(s, cx);
                                assert!(s.focus.approval_dialog && s.approval_preview_current());
                            }
                            "long-source" => {
                                s.dom.presentation = regressee_gui::jobs::DomPresentation::After;
                                s.reload_dom_viewer(cx);
                                finish_source(s, cx);
                            }
                            _ => {}
                        }
                        cx.notify();
                    })
                });
                cx.run_until_parked();
                if scene.starts_with("note-") {
                    type_dom_note(
                        &shell,
                        cx,
                        "Typed DOM review note 🦀\nThe hydrated page structure changed.\nScreenshot notes remain independent.",
                    );
                }
                if scene == "long-source" {
                    cx.update(|window, cx| {
                        shell.update(cx, |s, cx| {
                            s.dom
                                .source
                                .update(cx, |source, cx| source.focus_search(window, cx))
                        })
                    });
                    cx.simulate_input("Changed hydrated text");
                    cx.run_until_parked();
                }
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                if matches!(
                    scene,
                    "both" | "dom-only" | "long-source" | "history-source"
                ) {
                    assert!(cx.debug_bounds("dom-diff-source").unwrap().size.height >= px(48.));
                }
                let label = format!(
                    "dom-{}-{width:.0}x{height:.0}-{scene}",
                    preference.label().to_lowercase()
                );
                capture(cx, &root, &label).unwrap();
                owner.join_after_ui();
            }
        }
    }
}
