// SPDX-License-Identifier: MPL-2.0
//! One frozen-evidence report for both the inspector and the clipboard.
use super::*;

pub(super) const SHARING_NOTE: &str =
    "Includes full page URLs and capture settings. Review before sharing.";

const RECORD_FONT: &str = if cfg!(target_os = "windows") {
    "Consolas"
} else if cfg!(target_os = "macos") {
    "Menlo"
} else {
    "DejaVu Sans Mono"
};

struct Field {
    label: &'static str,
    value: String,
    json: bool,
}

impl Field {
    fn text(label: &'static str, value: impl ToString) -> Self {
        Self {
            label,
            value: value.to_string(),
            json: false,
        }
    }

    fn record(label: &'static str, value: &impl serde::Serialize) -> Self {
        Self {
            label,
            value: serde_json::to_string_pretty(value)
                .unwrap_or_else(|error| format!("Could not format this record: {error}")),
            json: true,
        }
    }
}

struct Section {
    title: &'static str,
    fields: Vec<Field>,
}

pub(super) struct Report(Vec<Section>);

impl Report {
    pub(super) fn text(&self) -> String {
        use std::fmt::Write;
        let mut text = format!("Regressee capture diagnostics\n{SHARING_NOTE}\n");
        for section in &self.0 {
            let _ = write!(text, "\n{}\n", section.title);
            for field in &section.fields {
                if field.json {
                    let _ = writeln!(text, "{}:\n{}", field.label, field.value);
                } else {
                    let _ = writeln!(text, "{}: {}", field.label, field.value);
                }
            }
        }
        text
    }

    pub(super) fn element(&self) -> AnyElement {
        let mut body = div().w_full().min_w_0().flex().flex_col().gap_4();
        for (index, section) in self.0.iter().enumerate() {
            let mut group = div()
                .w_full()
                .min_w_0()
                .flex()
                .flex_col()
                .gap_3()
                .when(index > 0, |el| {
                    el.border_t_1().border_color(rgb(palette().line)).pt_4()
                })
                .child(div().font_weight(FontWeight::SEMIBOLD).child(section.title));
            for field in &section.fields {
                group = group.child(
                    div()
                        .w_full()
                        .min_w_0()
                        .flex()
                        .flex_col()
                        .gap_1()
                        .child(div().text_color(rgb(palette().muted)).child(field.label))
                        .child(
                            div()
                                .w_full()
                                .min_w_0()
                                .when(field.json, |el| {
                                    el.debug_selector(move || {
                                        format!("capture-diagnostics-json-{index}")
                                    })
                                    .font_family(RECORD_FONT)
                                    .bg(rgb(palette().ground))
                                    .p_2()
                                })
                                .child(field.value.clone()),
                        ),
                );
            }
            body = body.child(group);
        }
        body.text_size(px(HELP_TEXT_SIZE))
            .line_height(px(HELP_LINE_HEIGHT))
            .into_any_element()
    }
}

fn timestamp(value: Option<i64>) -> String {
    value.map_or_else(
        || "Not recorded".into(),
        |seconds| {
            format!(
                "{} · Unix seconds: {seconds}",
                workspace_history::history_date(seconds)
            )
        },
    )
}

impl Shell {
    pub(super) fn capture_diagnostics(&self) -> Report {
        let mut sections = vec![Section {
            title: "Capture context",
            fields: vec![Field::text("Context", self.comparison_context())],
        }];
        if let Some(comparison) = &self.evidence.comparison {
            sections[0].fields.extend([
                Field::text("Baseline ID", comparison.revision.baseline),
                Field::text("Comparison run ID", comparison.revision.run),
                Field::text("Analysis ID", comparison.revision.id),
            ]);
        }
        let Some(key) = &self.selected_key else {
            sections[0]
                .fields
                .push(Field::text("Screenshot", "No screenshot selected"));
            return Report(sections);
        };
        sections[0].fields.extend([
            Field::text("Page ID", key.page),
            Field::text("Profile ID", key.profile),
        ]);
        let baseline_id = self
            .evidence
            .comparison
            .as_ref()
            .map(|a| a.revision.baseline);
        let baseline = self
            .evidence
            .baseline
            .as_ref()
            .filter(|b| Some(b.id) == baseline_id)
            .or_else(|| {
                self.evidence
                    .baselines
                    .iter()
                    .find(|b| Some(b.id) == baseline_id)
            });
        if let Some(reference) = baseline.and_then(|b| b.references.iter().find(|r| &r.key == key))
        {
            let mut fields = vec![
                Field::text("Source run ID", reference.run),
                Field::text("Captured", timestamp(Some(reference.captured_at))),
                Field::text("Original SHA-256", reference.artifact.as_str()),
            ];
            let capture = self
                .history
                .as_ref()
                .and_then(|h| h.runs.iter().find(|r| r.run.id == reference.run))
                .and_then(|r| r.evidence(key))
                .filter(|c| c.image.original == reference.artifact)
                .or_else(|| {
                    self.evidence.before.as_ref().filter(|c| {
                        &c.metadata.key == key && c.image.original == reference.artifact
                    })
                });
            if let Some(capture) = capture {
                fields.extend([
                    Field::text("Requested URL", &capture.metadata.requested),
                    Field::text("Final URL", &capture.metadata.final_url),
                    Field::record("Capture metadata (JSON)", &capture.metadata),
                ]);
            } else {
                fields.push(Field::text(
                    "Capture record",
                    "Not loaded. No URL inferred from today's project settings.",
                ));
            }
            sections.push(Section {
                title: "Before screenshot",
                fields,
            });
        }
        let run_id = self.evidence.comparison.as_ref().map(|a| a.revision.run);
        let run = self
            .evidence
            .latest
            .as_ref()
            .filter(|r| run_id.is_none_or(|id| id == r.run.id))
            .or_else(|| {
                self.history
                    .as_ref()
                    .and_then(|h| h.runs.iter().find(|r| Some(r.run.id) == run_id))
            });
        let mut fields = vec![];
        if let Some(run) = run {
            fields.extend([
                Field::text("Run ID", run.run.id),
                Field::text("Started", timestamp(run.run.started_at)),
                Field::text("Finished", timestamp(run.run.finished_at)),
            ]);
            if let Some(capture) = run.evidence(key) {
                fields.extend([
                    Field::text("Original SHA-256", capture.image.original.as_str()),
                    Field::text("Requested URL", &capture.metadata.requested),
                    Field::text("Final URL", &capture.metadata.final_url),
                    Field::record("Capture metadata (JSON)", &capture.metadata),
                ]);
            } else if let Some(target) = run.run.targets.iter().find(|t| &t.key == key) {
                fields.push(Field::record("Target outcome (JSON)", target));
            } else {
                fields.push(Field::text("Capture record", "Not loaded for this target."));
            }
        } else {
            fields.push(Field::text("Capture record", "Source run is not loaded."));
        }
        sections.push(Section {
            title: if self.evidence.comparison.is_some() {
                "After capture"
            } else {
                "Capture run"
            },
            fields,
        });
        if let Some(request) = self.viewer.as_ref().and_then(|v| v.request.as_ref()) {
            sections.push(Section {
                title: "Comparison inputs",
                fields: vec![Field::record(
                    "Pair descriptors, policy and ignored-area revision (JSON)",
                    request,
                )],
            });
        }
        if let Some(comparison) = &self.evidence.comparison {
            let mut fields = Vec::new();
            if let Some(classification) = comparison.review_classification(key) {
                fields.push(Field::text(
                    "Pixel comparison used for review",
                    classification.label(),
                ));
            }
            if let Some(notice) = self.capture_notice() {
                fields.push(Field::text("Capture cautions", notice.detail));
            }
            if let Some(issue) = self.comparison_issue() {
                fields.push(Field::text("Comparison status", issue.heading));
                fields.push(Field::text("Why comparison is unavailable", issue.reason));
                fields.push(Field::text("What to do", issue.recovery));
            }
            if let Some(result) = comparison.revision.results.iter().find(|r| &r.key == key) {
                fields.push(Field::record("Saved comparison result (JSON)", result));
            }
            if let Some(perceptual) = &comparison.perceptual {
                if let Some((_, report)) = perceptual.reports.iter().find(|(k, _)| k == key) {
                    fields.push(Field::record("Saved analysis report (JSON)", report));
                }
                if let Some((_, failure)) = perceptual.failures.iter().find(|(k, _)| k == key) {
                    fields.push(Field::text("Recorded analysis failure", failure));
                }
            }
            sections.push(Section {
                title: "Comparison result",
                fields,
            });
        }
        Report(sections)
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    use crate::{phase2_evidence_tests::inside, workflow_refinement_tests::load_review};
    use gpui::{TestAppContext, VisualTestContext};

    fn draw(cx: &mut VisualTestContext) {
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
    }

    #[gpui::test]
    fn capture_diagnostics_preserve_exact_records_and_missing_states(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (_, project, _, _, _) = approval_fixture::fixture(dir.path());
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            let key = s.selected_key.clone().unwrap();
            let report = s.capture_diagnostics();
            assert_eq!(
                report.0.iter().map(|s| s.title).collect::<Vec<_>>(),
                [
                    "Capture context",
                    "Before screenshot",
                    "After capture",
                    "Comparison inputs",
                    "Comparison result"
                ]
            );
            let after = &s.evidence.latest.as_ref().unwrap().captures[0];
            let before = s.evidence.before.as_ref().unwrap();
            for (section, record) in [(&report.0[1], before), (&report.0[2], after)] {
                let field = section.fields.iter().find(|f| f.json).unwrap();
                assert_eq!(
                    serde_json::from_str::<serde_json::Value>(&field.value).unwrap(),
                    serde_json::to_value(&record.metadata).unwrap()
                );
                assert!(field.value.contains("\n  \"version\": 1,"));
                assert!(report.text().contains(record.image.original.as_str()));
            }
            assert_eq!(
                serde_json::from_str::<serde_json::Value>(&report.0[3].fields[0].value).unwrap(),
                serde_json::to_value(s.viewer.as_ref().unwrap().request.as_ref().unwrap()).unwrap()
            );
            let text = report.text();
            assert!(text.contains(&key.page.to_string()));
            assert!(text.contains(&key.profile.to_string()));
            assert!(text.contains("1970-01-01 00:01:40 UTC · Unix seconds: 100"));
            assert!(!text.contains("Some("));
            assert!(!text.contains("CaptureMetadata {"));

            // A missing historical capture must never be reconstructed from current settings.
            s.history = None;
            s.evidence.before = None;
            s.project.as_mut().unwrap().project.sources[0] =
                url::Url::parse("https://unrelated.example/").unwrap();
            let missing = s.capture_diagnostics().text();
            assert!(missing.contains("No URL inferred from today's project settings."));
            assert!(!missing.contains("unrelated.example"));

            s.evidence.comparison = None;
            s.viewer = None;
            let run = s.evidence.latest.as_mut().unwrap();
            run.run.started_at = None;
            run.run.finished_at = None;
            run.captures.clear();
            run.capture = None;
            run.run.targets[0].outcome = TargetOutcome::Failed {
                code: regressee_core::domain::CaptureError::NeedsLogin,
            };
            let failed = s.capture_diagnostics();
            assert_eq!(failed.0[1].title, "Capture run");
            assert!(
                failed
                    .text()
                    .contains("Started: Not recorded\nFinished: Not recorded")
            );
            assert!(failed.text().contains("NeedsLogin"));
            assert!(!failed.text().contains("Original SHA-256"));
            s.selected_key = None;
            assert!(
                s.capture_diagnostics()
                    .text()
                    .contains("No screenshot selected")
            );
        });
        owned.join_after_ui();
    }

    #[gpui::test]
    fn capture_diagnostics_copy_is_pinned_keyboard_reachable_and_read_only(
        cx: &mut TestAppContext,
    ) {
        crate::image_controls_tests::bind(cx);
        for comparison in [true, false] {
            let dir = tempfile::tempdir().unwrap();
            let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
            let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                if !comparison {
                    s.evidence.comparison = None;
                    s.viewer.as_mut().unwrap().request = None;
                }
                let metadata = &mut s.evidence.latest.as_mut().unwrap().captures[0].metadata;
                metadata.requested =
                    "https://northline.example/orders/?locale=nb-NO#størrelse".into();
                metadata.replacement =
                    Some(serde_json::json!({ "fixture": "måling ".repeat(1_200) }));
            });
            for (width, height) in [(1280., 800.), (820., 600.)] {
                cx.simulate_resize(size(px(width), px(height)));
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        s.configure_review_geometry(width);
                        assert!(!s.control_enabled(Control::CopyCaptureDiagnostics, cx));
                        s.activate_control(Control::Phase5(36), w, cx);
                        assert!(!s.control_mounted(Control::CopyCaptureDiagnostics));
                    })
                });
                draw(cx);
                // Tab reveals the complete disclosure in the actual inner scroller.
                cx.update(|w, cx| {
                    shell.read_with(cx, |s, cx| w.focus(&s.handle(Control::Phase5(39), cx)))
                });
                cx.simulate_keystrokes("tab");
                draw(cx);
                cx.update(|w, cx| {
                    shell.read_with(cx, |s, cx| {
                        assert!(s.handle(Control::Phase5(38), cx).is_focused(w))
                    })
                });
                inside(
                    cx.debug_bounds("action-Phase5(38)").unwrap(),
                    cx.debug_bounds("review-details-scroll").unwrap(),
                );
                cx.simulate_keystrokes("enter");
                draw(cx);
                let inspector = cx.debug_bounds("viewer-inspector").unwrap();
                let button = cx.debug_bounds("action-CopyCaptureDiagnostics").unwrap();
                inside(button, inspector);
                let footer = cx.debug_bounds("capture-diagnostics-footer").unwrap();
                let reading = cx.debug_bounds("review-details-scroll").unwrap();
                assert!(reading.bottom() <= footer.top());
                let disclosure = cx.debug_bounds("action-Phase5(38)").unwrap();
                assert!((disclosure.top() - reading.top() - px(8.)).abs() <= px(1.));
                let content = cx.debug_bounds("capture-provenance-details").unwrap();
                assert!(
                    content.top() + px(34.) <= reading.bottom(),
                    "The newly opened heading must be visible"
                );
                let (text, requests, drafts) = shell.read_with(cx, |s, cx| {
                    (
                        s.capture_diagnostics().text(),
                        s.next_request,
                        s.review_exit_signature(cx),
                    )
                });
                assert!(
                    text.len() > 8_192,
                    "Diagnostics must not inherit the review-note limit"
                );
                cx.simulate_keystrokes("tab enter");
                draw(cx);
                cx.update(|w, cx| {
                    shell.read_with(cx, |s, cx| {
                        assert!(s.handle(Control::CopyCaptureDiagnostics, cx).is_focused(w));
                        assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), text);
                        assert_eq!(s.next_request, requests);
                        assert_eq!(s.review_exit_signature(cx), drafts);
                    })
                });
                assert!(cx.debug_bounds("capture-diagnostics-copied").is_some());
                shell.update(cx, |s, _| s.native_prompt = true);
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                            "Guarded clipboard".into(),
                        ));
                        s.activate_control(Control::CopyCaptureDiagnostics, w, cx);
                        assert_eq!(
                            cx.read_from_clipboard().unwrap().text().unwrap(),
                            "Guarded clipboard"
                        );
                        s.native_prompt = false;
                    })
                });
                shell.update(cx, |s, _| {
                    s.review_details_scroll.set_offset(gpui::point(
                        px(0.),
                        -s.review_details_scroll.max_offset().height,
                    ))
                });
                draw(cx);
                assert_eq!(
                    cx.debug_bounds("action-CopyCaptureDiagnostics").unwrap(),
                    button
                );
                let content = cx.debug_bounds("capture-provenance-details").unwrap();
                assert!(
                    content.bottom()
                        <= cx.debug_bounds("review-details-scroll").unwrap().bottom() + px(1.)
                );
                // Pointer activation at the tail also copies the entire offscreen report.
                cx.simulate_click(button.center(), gpui::Modifiers::none());
                cx.update(|_, cx| {
                    assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), text)
                });
                cx.simulate_keystrokes("escape");
                draw(cx);
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        assert!(s.handle(Control::Phase5(36), cx).is_focused(w));
                        assert!(!s.control_enabled(Control::CopyCaptureDiagnostics, cx));
                        cx.write_to_clipboard(gpui::ClipboardItem::new_string(
                            "Keep clipboard".into(),
                        ));
                        s.activate_control(Control::CopyCaptureDiagnostics, w, cx);
                        assert_eq!(
                            cx.read_from_clipboard().unwrap().text().unwrap(),
                            "Keep clipboard"
                        );
                    })
                });
            }
            assert_eq!(
                original,
                serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
            );
            owned.join_after_ui();
        }
    }
}

#[cfg(all(test, feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "explicit two-size native offscreen capture diagnostics inspection"]
fn renderer_capture_diagnostics(cx: &mut gpui::TestAppContext) {
    use crate::ui_renderer_capture::{capture, output_root};
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    let dir = tempfile::Builder::new()
        .prefix("diagnostics-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for comparison in [false, true] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|w, cx| shell.update(cx, |s, cx| {
                crate::workflow_refinement_tests::load_review(s, project, cx);
                if !comparison { s.evidence.comparison = None; s.viewer.as_mut().unwrap().request = None; }
                // Long URLs, query strings and Unicode are synthetic local evidence only.
                let metadata = &mut s.evidence.latest.as_mut().unwrap().captures[0].metadata;
                metadata.requested = "https://northline.example/account/orders/very-long-retained-address/?locale=nb-NO&return=%2Forders%2F#størrelse".into();
                metadata.final_url = metadata.requested.clone();
                s.configure_review_geometry(width);
                s.activate_control(Control::Phase5(36), w, cx);
            }));
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            cx.update(|w, cx| {
                shell.update(cx, |s, cx| s.activate_control(Control::Phase5(38), w, cx))
            });
            cx.update(|w, cx| {
                let _ = w.draw(cx);
            });
            cx.run_until_parked();
            for state in ["start", "json", "copied-tail"] {
                if state == "json" {
                    let record = cx.debug_bounds("capture-diagnostics-json-1").unwrap();
                    shell.update(cx, |s, _| {
                        let viewport = s.review_details_scroll.bounds();
                        let offset = s.review_details_scroll.offset();
                        s.review_details_scroll.set_offset(gpui::point(
                            px(0.),
                            offset.y + viewport.top() - record.top(),
                        ));
                    });
                } else if state == "copied-tail" {
                    cx.update(|w, cx| {
                        shell.update(cx, |s, cx| {
                            s.activate_control(Control::CopyCaptureDiagnostics, w, cx);
                            s.review_details_scroll.set_offset(gpui::point(
                                px(0.),
                                -s.review_details_scroll.max_offset().height,
                            ));
                            assert_eq!(
                                cx.read_from_clipboard().unwrap().text().unwrap(),
                                s.capture_diagnostics().text()
                            );
                        })
                    });
                }
                let label = format!(
                    "diagnostics-{}-{width:.0}x{height:.0}-{state}",
                    if comparison { "compare" } else { "capture" }
                );
                capture(cx, &root, &label).unwrap();
                manifest.push(label);
            }
            owned.join_after_ui();
        }
    }
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    std::fs::write(
        root.join("diagnostics-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
