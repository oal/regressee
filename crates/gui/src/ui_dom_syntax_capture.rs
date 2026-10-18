// SPDX-License-Identifier: MPL-2.0
//! Scoped syntax/readability matrix. Real local IPC, synthetic immutable evidence.
use super::*;
use crate::dom_workflow_tests::{Engine, finish_source, load, syntax_fixture};
use regressee_core::domain::{CaptureMode, CheckKind};
use regressee_gui::jobs::DomPresentation;

#[gpui::test]
#[ignore = "explicit two-size light/dark native snapshot syntax inspection"]
fn renderer_capture_dom_syntax(cx: &mut TestAppContext) {
    capture_matrix(cx, false);
}

#[gpui::test]
#[ignore = "recorded DOM History currently has a separately documented pre-existing loading failure"]
fn renderer_capture_dom_syntax_history(cx: &mut TestAppContext) {
    capture_matrix(cx, true);
}

fn capture_matrix(cx: &mut TestAppContext, history: bool) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let fixture = syntax_fixture(CaptureMode::ScreenshotsAndDomSnapshots);
    let engine = Engine::start(&fixture);
    let mut failures = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for preference in [AppAppearance::Light, AppAppearance::Dark] {
            let scenes: &[&str] = if history {
                &["history"]
            } else {
                &["diff", "before", "after", "search", "selection"]
            };
            for &scene in scenes {
                let label = format!(
                    "syntax-{}-{width:.0}x{height:.0}-{scene}",
                    preference.label().to_lowercase()
                );
                regressee_storage::AppPreferences {
                    appearance: preference,
                    reduced_motion: true,
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
                        load(s, &fixture, cx);
                        finish_source(s, cx);
                        s.configure_review_geometry(width);
                        if scene == "history" {
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
                                s.activate_control(
                                    Control::EvidenceKind(CheckKind::DomSnapshot),
                                    window,
                                    cx,
                                );
                            }
                            finish_source(s, cx);
                            assert!(s.history_inspection.is_some() && s.dom_active());
                        }
                        if scene != "diff" && scene != "history" {
                            s.dom.presentation = if scene == "before" {
                                DomPresentation::Before
                            } else {
                                DomPresentation::After
                            };
                            s.reload_dom_viewer(cx);
                            finish_source(s, cx);
                        }
                        cx.notify();
                    })
                });
                cx.run_until_parked();
                if scene == "search" || scene == "diff" {
                    cx.update(|window, cx| {
                        shell.update(cx, |s, cx| {
                            s.dom
                                .source
                                .update(cx, |source, cx| source.focus_search(window, cx));
                        })
                    });
                    // Reveal a real changed attribute in Diff, and exercise both
                    // current/passive occurrences across syntax in source mode.
                    cx.simulate_input(if scene == "diff" { "class" } else { "http" });
                    cx.run_until_parked();
                }
                cx.update(|window, cx| {
                    let _ = window.draw(cx);
                });
                let bounds = cx.debug_bounds("dom-diff-source");
                if bounds.is_none_or(|bounds| bounds.size.height < px(48.)) {
                    let stage = cx.debug_bounds("dom-evidence-stage");
                    let state = cx.debug_bounds("dom-diff-state");
                    let diagnostic = shell.read_with(cx, |s, _| format!("{label}: source={bounds:?}, stage={stage:?}, state={state:?}, screen={:?}, picker={}, active={}, frame={}, failed={}, pending={}, error={:?}", s.focus.screen, s.focus.shows_project_picker(), s.dom_active(), s.dom.frame.is_some(), s.dom.failed, s.dom.pending.is_some(), s.error));
                    failures.push(diagnostic);
                }
                if scene == "selection" {
                    let bounds = bounds.expect("selection requires source");
                    let point = gpui::point(bounds.right() - px(40.), bounds.top() + px(10.));
                    cx.simulate_mouse_down(point, MouseButton::Left, gpui::Modifiers::none());
                    cx.simulate_mouse_up(point, MouseButton::Left, gpui::Modifiers::none());
                    cx.simulate_keystrokes("ctrl-a ctrl-c");
                    cx.update(|_, cx| {
                        assert_eq!(
                            cx.read_from_clipboard().unwrap().text().unwrap(),
                            fixture.source.trim_end_matches('\n')
                        );
                    });
                    cx.run_until_parked();
                }
                capture(cx, &root, &label).unwrap();
                owner.join_after_ui();
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
