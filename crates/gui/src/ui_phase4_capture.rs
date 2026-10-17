// SPDX-License-Identifier: MPL-2.0
//! Finite Phase 4 native recipe, with actual mounted Tab reveal and real local
//! history page reads. Synthetic/offline states, not OS-input qualification.
use super::*;
use crate::phase2_evidence_tests::inside;
use crate::phase4_hierarchy_tests::{activate, draw, extend_history, tab_to};
use crate::workflow_refinement_tests::{finish_jobs, load_review};

fn geometry(b: Bounds<Pixels>) -> serde_json::Value {
    serde_json::json!({"x":f32::from(b.left()),"y":f32::from(b.top()),"width":f32::from(b.size.width),"height":f32::from(b.size.height)})
}

fn record(
    shell: &Entity<Shell>,
    cx: &mut VisualTestContext,
    root: &Path,
    label: String,
    selectors: &[&str],
    ledger: &mut Vec<serde_json::Value>,
) {
    eprintln!("Phase 4 evidence state: {label}");
    draw(cx);
    let measurements: Vec<_> = selectors.iter().map(|selector| {
        let bounds = cx.debug_bounds(selector.to_string().leak()).unwrap_or_else(|| panic!("{label}: absent {selector}"));
        let clip = shell.read_with(cx, |s, _| if s.viewer_inspector_open() { s.review_details_scroll.bounds() } else { s.screen_scroll.bounds() });
        inside(bounds, clip);
        serde_json::json!({"selector":selector,"painted":geometry(bounds),"actual_clip":geometry(clip),"fully_visible":true})
    }).collect();
    let state = cx.update(|w, cx| shell.read_with(cx, |s, cx| {
        serde_json::json!({
            "screen":format!("{:?}",s.focus.screen),"focused":s.focused(w,cx).map(|c|format!("{c:?}")),
            "mounted_enabled":s.visible_controls().into_iter().filter(|c|s.control_enabled(*c,cx)).map(|c|format!("{c:?}")).collect::<Vec<_>>(),
            "analysis_open":s.review_tools,"ignored_open":s.details.contains(&Detail::IgnoredAreas),"advanced_open":s.analysis_advanced,
            "metrics_open":s.details.contains(&Detail::Analysis),"current_policy":crate::analysis_ux::policy_name(&s.analysis_policy),
            "numeric_fields":s.analysis_fields.iter().take(9).enumerate().map(|(index, f)| {let f=f.read(cx);serde_json::json!({"index":index,"value":f.content.to_string(),"touched":f.touched,"composing":f.composing(),"error":f.validation_error()})}).collect::<Vec<_>>(),
            "displayed_analysis":s.evidence.comparison.as_ref().map(|a|a.revision.id),
            "displayed_policy":s.evidence.comparison.as_ref().and_then(|a|a.perceptual.as_ref()).map(|p|crate::analysis_ux::policy_name(&p.policy)),
            "retained_analyses":s.evidence.analyses,"analysis_due":s.analysis_due.is_some(),"analysis_pending":s.analysis_pending.is_some(),
            "selected_key":s.selected_key,"pair":s.viewer.as_ref().and_then(|v|v.request.as_ref()).map(|r|serde_json::json!({"before":r.before,"after":r.after,"mask":r.mask})),
            "mask_dirty":s.mask_dirty,"coordinate_draft":s.mask_geometry_dirty(cx),"rectangles":s.mask_rectangles,
            "ignored_area_status":s.ignored_area_status(cx),
            "history_section":s.history_section,"history_page":s.history.as_ref().map(|h|h.page),"more_pages":s.history.as_ref().map(|h|h.more),
            "loaded_runs":s.history.as_ref().map(|h|h.runs.len()),"shown_runs":s.history_run_indices().len(),"loaded_baselines":s.evidence.baselines.len(),"shown_baselines":s.history_baseline_indices().len(),
            "selected_run":s.history.as_ref().and_then(|h|h.runs.get(s.run_history_index)).map(|r|r.run.id),"history_query":s.history_query,
            "provenance_open":s.details.contains(&Detail::History),"portability_exact_open":s.details.contains(&Detail::Portability),
            "export_preview":s.export_preview,"deletion_preview":s.deletion_preview,
            "inner_scroll":f32::from(s.review_details_scroll.offset().y),"screen_scroll":f32::from(s.screen_scroll.offset().y),
            "active_baseline":s.project.as_ref().and_then(|p|p.project.active_baseline)
        })
    }));
    if std::env::var_os("REGRESSEE_PHASE4_CAPTURE_PREFLIGHT").is_none() {
        capture(cx, root, &label).unwrap();
    }
    let qualification = if std::env::var_os("REGRESSEE_PHASE4_CAPTURE_PREFLIGHT").is_some() {
        "simulated geometry preflight only; no app PNG rendered"
    } else {
        "native offscreen + simulated mounted traversal; constructed offline project"
    };
    ledger.push(serde_json::json!({"label":label,"state":state,"geometry":measurements,"qualification":qualification}));
}

/// Supported wheel dispatch in the actual inspector or outer reading owner.
/// Interactive reachability is proven separately by mounted Tab traversal.
fn read_at(shell: &Entity<Shell>, cx: &mut VisualTestContext, selector: &'static str) {
    draw(cx);
    for _ in 0..8 {
        let b = cx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("Static reading copy {selector} is absent"));
        let clip = shell.read_with(cx, |s, _| {
            if s.viewer_inspector_open() {
                s.review_details_scroll.bounds()
            } else {
                s.screen_scroll.bounds()
            }
        });
        if b.top() >= clip.top() && b.bottom() <= clip.bottom() {
            return;
        }
        let delta = clip.top() + px(12.) - b.top();
        cx.simulate_event(gpui::ScrollWheelEvent {
            position: gpui::point(clip.right() - px(4.), clip.top() + px(12.)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), delta)),
            ..Default::default()
        });
        draw(cx);
    }
    panic!("Static reading copy {selector} not revealed by its wheel owner");
}

fn complete_analysis(s: &mut Shell, cx: &mut Context<Shell>) {
    s.analysis_due = Some(std::time::Instant::now());
    s.poll_analysis(cx);
    let deadline = std::time::Instant::now() + Duration::from_secs(60);
    while s.analysis_pending.is_some() {
        let update = s
            .events
            .recv_timeout(deadline.saturating_duration_since(std::time::Instant::now()))
            .unwrap();
        assert!(!matches!(update.reply, Reply::Failure(_)));
        s.apply_update(update, cx);
    }
    finish_jobs(s, cx);
}

#[gpui::test]
#[ignore = "explicit finite Phase 4 two-size native evidence; isolated offline projects"]
fn renderer_capture_phase4(cx: &mut TestAppContext) {
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
    let preflight = std::env::var_os("REGRESSEE_PHASE4_CAPTURE_PREFLIGHT").is_some();
    if !preflight {
        cx.enable_offscreen_renderer().unwrap();
    }
    cx.update(|cx| {
        text_input::bind(cx);
        cx.bind_keys([
            KeyBinding::new("escape", Dismiss, Some("Shell")),
            KeyBinding::new("tab", Tab, Some("Shell")),
            KeyBinding::new("shift-tab", TabPrev, Some("Shell")),
            KeyBinding::new("enter", Activate, Some("Shell")),
        ]);
    });
    let few = tempfile::Builder::new()
        .prefix("phase4-few-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = super::ui_review_matrix::fixture_named(&root, few.path(), false);
    let few_baseline = store.load_project(project).unwrap().project.active_baseline;
    let empty = regressee_core::domain::Project::draft(
        "Empty local history".into(),
        "https://empty.example/",
    )
    .unwrap();
    drop(store);
    let mut store = regressee_storage::Store::open(AppPaths::isolated(few.path())).unwrap();
    store.save_project(&empty, None).unwrap();
    drop(store);
    let many = tempfile::Builder::new()
        .prefix("phase4-many-")
        .tempdir_in(&root)
        .unwrap();
    let (mut store, many_project, _) =
        super::ui_review_matrix::fixture_named(&root, many.path(), false);
    let before = store
        .project_evidence(many_project)
        .unwrap()
        .baseline
        .unwrap()
        .references[0]
        .run;
    extend_history(&mut store, many_project, before, 68);
    drop(store);
    let mut ledger = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(few.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                s.error = None;
                s.notice = None;
                s.activate_control(Control::Guided(44), w, cx);
            })
        });
        let name = |scene: &str| format!("phase4-{width:.0}x{height:.0}-{scene}");
        record(
            &shell,
            cx,
            &root,
            name("analysis-basic"),
            &[
                "analysis-current-summary",
                "analysis-sensitivity-heading",
                "action-Phase5(1)",
            ],
            &mut ledger,
        );
        let original = shell.read_with(cx, |s, _| s.selected_key.clone());
        shell.update(cx, |s, cx| {
            s.selected_key = s
                .evidence
                .comparison
                .as_ref()
                .unwrap()
                .revision
                .results
                .iter()
                .find(|r| r.classification == Classification::NoSignificantChange)
                .map(|r| r.key.clone());
            assert!(s.selected_key.is_some());
            s.reload_viewer(cx);
            finish_jobs(s, cx);
            cx.notify();
        });
        record(
            &shell,
            cx,
            &root,
            name("analysis-no-significant-change"),
            &["analysis-current-summary", "action-Phase5(1)"],
            &mut ledger,
        );
        shell.update(cx, |s, cx| {
            s.selected_key = original;
            s.reload_viewer(cx);
            finish_jobs(s, cx);
            cx.notify();
        });
        for (index, preset) in [(0, "sensitive"), (2, "relaxed"), (1, "balanced")] {
            activate(&shell, cx, Control::Phase5(index));
            shell.update(cx, complete_analysis);
            draw(cx);
            shell.read_with(cx, |s, _| {
                assert_eq!(
                    crate::analysis_ux::policy_name(&s.analysis_policy).to_lowercase(),
                    preset
                )
            });
            tab_to(&shell, cx, Control::Phase5(index));
            record(
                &shell,
                cx,
                &root,
                name(&format!("preset-{preset}")),
                &[&format!("action-Phase5({index})")],
                &mut ledger,
            );
        }
        // A deliberately constructed pending-worker state permits deterministic
        // active/cancel paint; real worker/revision transitions are tested above.
        shell.update(cx, |s, cx| {
            s.analysis_pending = Some((
                u64::MAX,
                Arc::new(std::sync::atomic::AtomicBool::new(false)),
            ));
            cx.notify();
        });
        tab_to(&shell, cx, Control::Phase5(5));
        shell.read_with(cx, |s, cx| {
            assert!(!s.control_enabled(Control::Phase5(21), cx))
        });
        record(
            &shell,
            cx,
            &root,
            name("active-analysis"),
            &["action-Phase5(5)"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase5(5));
        shell.update(cx, |s, cx| {
            assert!(
                s.analysis_pending
                    .as_ref()
                    .unwrap()
                    .1
                    .load(std::sync::atomic::Ordering::Acquire)
            );
            s.analysis_pending = None;
            cx.notify();
        });
        let latest = shell.read_with(cx, |s, _| {
            s.evidence.comparison.as_ref().unwrap().revision.id
        });
        activate(&shell, cx, Control::Phase5(26));
        shell.update(cx, finish_jobs);
        draw(cx);
        shell.read_with(cx, |s, _| {
            assert_ne!(s.evidence.comparison.as_ref().unwrap().revision.id, latest)
        });
        record(
            &shell,
            cx,
            &root,
            name("previous-analysis"),
            &["action-Phase5(26)"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Details(Detail::IgnoredAreas));
        tab_to(&shell, cx, Control::Phase5(34));
        record(
            &shell,
            cx,
            &root,
            name("ignored-disclosure"),
            &["action-Phase5(14)", "action-Phase5(34)"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase5(34));
        for i in 0..4 {
            tab_to(&shell, cx, Control::AnalysisField(10 + i));
            record(
                &shell,
                cx,
                &root,
                name(&format!("ignored-coordinate-{i}")),
                &[&format!("ignored-area-field-{i}")],
                &mut ledger,
            );
        }
        // Only presentation/draft construction is reset between unrelated scenes.
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.mask_dirty = false;
                s.mask_selected = None;
                s.review_tools = false;
                load_review(s, project, cx);
                s.error = None;
                s.notice = None;
                s.details.remove(&Detail::IgnoredAreas);
                s.activate_control(Control::Guided(44), w, cx);
                s.activate_control(Control::Phase5(3), w, cx);
            })
        });
        for i in 0..9 {
            tab_to(&shell, cx, Control::AnalysisField(i));
            record(
                &shell,
                cx,
                &root,
                name(&format!("advanced-field-{i}")),
                &[&format!("analysis-parameter-field-{i}")],
                &mut ledger,
            );
        }
        tab_to(&shell, cx, Control::AnalysisField(0));
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("999");
        shell.update(cx, |s, cx| {
            s.poll_analysis(cx);
            assert!(s.analysis_fields[0].read(cx).validation_error().is_some());
            assert!(s.analysis_due.is_none());
        });
        // Use the real validation action: shared fields do not show correction
        // during the first incomplete edit or IME composition.
        let requests = shell.read_with(cx, |s, _| s.next_request);
        activate(&shell, cx, Control::Phase5(4));
        shell.read_with(cx, |s, _| assert_eq!(s.next_request, requests));
        tab_to(&shell, cx, Control::AnalysisField(0));
        shell.read_with(cx, |s, cx| assert!(s.analysis_fields[0].read(cx).touched));
        record(
            &shell,
            cx,
            &root,
            name("advanced-invalid-revealed"),
            &["analysis-parameter-field-0", "field-correction"],
            &mut ledger,
        );
        shell.update(cx, |s, cx| s.set_analysis_fields(cx));
        draw(cx);
        for (i, label) in [(27, "advanced-height-switch"), (28, "advanced-switch-tail")] {
            tab_to(&shell, cx, Control::Phase5(i));
            record(
                &shell,
                cx,
                &root,
                name(label),
                &[&format!("check-Phase5({i})")],
                &mut ledger,
            );
        }
        activate(&shell, cx, Control::Details(Detail::Analysis));
        read_at(&shell, cx, "analysis-metrics-summary");
        record(
            &shell,
            cx,
            &root,
            name("analysis-metrics"),
            &["analysis-metrics-summary"],
            &mut ledger,
        );
        read_at(&shell, cx, "analysis-metrics-dimensions");
        record(
            &shell,
            cx,
            &root,
            name("analysis-metrics-tail"),
            &["analysis-metrics-dimensions"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase5(39));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                assert!(s.handle(Control::Guided(44), cx).is_focused(w));
                s.focus.screen = Screen::History;
                s.review_tools = false;
                s.history_section = 0;
                s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                cx.notify();
            })
        });
        read_at(&shell, cx, "history-run-list");
        record(
            &shell,
            cx,
            &root,
            name("history-few-runs"),
            &["history-run-list"],
            &mut ledger,
        );
        shell.read_with(cx, |s, _| {
            assert_eq!(s.history.as_ref().unwrap().runs.len(), 3)
        });
        assert_eq!(
            cx.debug_bounds("history-run-list").unwrap().size.height,
            px(192.)
        );
        tab_to(&shell, cx, Control::Phase6(19));
        record(
            &shell,
            cx,
            &root,
            name("history-selected-run"),
            &["action-Phase6(19)", "action-Phase6(5)"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(17));
        record(
            &shell,
            cx,
            &root,
            name("history-few-baselines"),
            &["history-baseline-list"],
            &mut ledger,
        );
        assert_eq!(
            cx.debug_bounds("history-baseline-list")
                .unwrap()
                .size
                .height,
            px(64.)
        );
        tab_to(&shell, cx, Control::Phase6(2));
        record(
            &shell,
            cx,
            &root,
            name("history-selected-baseline"),
            &["action-Phase6(2)", "action-Phase6(4)"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(2));
        shell.update(cx, finish_jobs);
        draw(cx);
        shell.read_with(cx, |s, _| {
            assert!(s.export_preview.is_some(), "{:?}", s.error)
        });
        read_at(&shell, cx, "export-privacy-consequences");
        record(
            &shell,
            cx,
            &root,
            name("export-privacy"),
            &["export-privacy-consequences"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Details(Detail::Portability));
        record(
            &shell,
            cx,
            &root,
            name("export-exact"),
            &["action-Details(Portability)"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(7));
        activate(&shell, cx, Control::Phase6(16));
        activate(&shell, cx, Control::Phase6(5));
        shell.update(cx, finish_jobs);
        draw(cx);
        shell.read_with(cx, |s, _| {
            assert!(s.deletion_preview.is_some(), "{:?}", s.error)
        });
        read_at(&shell, cx, "deletion-consequences");
        record(
            &shell,
            cx,
            &root,
            name("delete-protections"),
            &["deletion-consequences"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Details(Detail::Portability));
        record(
            &shell,
            cx,
            &root,
            name("delete-exact"),
            &["action-Details(Portability)"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(7));
        shell.update(cx, |s, cx| {
            s.queue(Work::Load(empty.id), cx);
            finish_jobs(s, cx);
            s.focus.screen = Screen::History;
            s.notice = None;
            s.history_section = 0;
            s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
            cx.notify();
        });
        record(
            &shell,
            cx,
            &root,
            name("history-zero-runs"),
            &["history-section-navigation", "history-search-controls"],
            &mut ledger,
        );
        shell.read_with(cx, |s, _| {
            assert!(s.history.as_ref().unwrap().runs.is_empty());
            assert!(s.history_run_indices().is_empty());
            assert!(!s.control_mounted(Control::HistoryRun(0)));
        });
        activate(&shell, cx, Control::Phase6(17));
        record(
            &shell,
            cx,
            &root,
            name("history-zero-baselines"),
            &["history-section-navigation", "history-search-controls"],
            &mut ledger,
        );
        shell.read_with(cx, |s, _| {
            assert!(s.evidence.baselines.is_empty());
            assert!(s.history_baseline_indices().is_empty());
            assert!(!s.control_mounted(Control::HistoryBaseline(0)));
        });
        shell.update(cx, |s, cx| {
            s.focus.screen = Screen::Settings;
            s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
            cx.notify();
        });
        record(
            &shell,
            cx,
            &root,
            name("app-settings"),
            &["app-settings-heading"],
            &mut ledger,
        );
        owned.join_after_ui();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(many.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, many_project, cx);
            s.notice = None;
            s.focus.screen = Screen::History;
            s.history_section = 0;
            cx.notify();
        });
        read_at(&shell, cx, "history-run-list");
        record(
            &shell,
            cx,
            &root,
            name("history-many"),
            &["history-run-list"],
            &mut ledger,
        );
        assert_eq!(
            cx.debug_bounds("history-run-list").unwrap().size.height,
            px(256.)
        );
        let last = shell.read_with(cx, |s, _| {
            assert!(s.history.as_ref().unwrap().more);
            s.history.as_ref().unwrap().runs.len() - 1
        });
        activate(&shell, cx, Control::HistoryRun(last));
        let row = format!("history-row-HistoryRun({last})");
        let row_bounds = cx.debug_bounds(row.clone().leak()).unwrap();
        shell.read_with(cx, |s, _| inside(row_bounds, s.history_run_scroll.bounds()));
        record(
            &shell,
            cx,
            &root,
            name("history-last-run"),
            &[&row],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(11));
        shell.update(cx, finish_jobs);
        draw(cx);
        shell.read_with(cx, |s, _| {
            assert_eq!(s.history.as_ref().unwrap().page, 1);
            assert!(!s.history.as_ref().unwrap().more);
        });
        tab_to(&shell, cx, Control::Phase6(12));
        record(
            &shell,
            cx,
            &root,
            name("history-page-two"),
            &["action-Phase6(12)"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(12));
        shell.update(cx, finish_jobs);
        draw(cx);
        shell.update(cx, |s, cx| {
            assert_eq!(s.history.as_ref().unwrap().page, 0);
            let query = s.history.as_ref().unwrap().runs[0].run.id.to_string();
            s.history_search.update(cx, |f, cx| f.set_value(&query, cx))
        });
        draw(cx);
        shell.read_with(cx, |s, _| {
            assert_eq!(s.history_run_indices(), [0]);
            assert_eq!(s.history.as_ref().unwrap().page, 0);
        });
        tab_to(&shell, cx, Control::HistorySearch);
        read_at(&shell, cx, "history-search-controls");
        record(
            &shell,
            cx,
            &root,
            name("history-filter-one-run"),
            &["history-search-controls"],
            &mut ledger,
        );
        shell.update(cx, |s, cx| {
            s.history_search
                .update(cx, |f, cx| f.set_value("nothing-matches", cx))
        });
        draw(cx);
        shell.read_with(cx, |s, _| assert!(s.history_run_indices().is_empty()));
        record(
            &shell,
            cx,
            &root,
            name("history-no-matches"),
            &["history-search-controls"],
            &mut ledger,
        );
        activate(&shell, cx, Control::HistoryClear);
        activate(&shell, cx, Control::HistoryRun(0));
        activate(&shell, cx, Control::Details(Detail::History));
        read_at(&shell, cx, "history-provenance-run");
        record(
            &shell,
            cx,
            &root,
            name("history-provenance"),
            &["history-provenance-run"],
            &mut ledger,
        );
        activate(&shell, cx, Control::Phase6(17));
        let last = shell.read_with(cx, |s, _| s.evidence.baselines.len() - 1);
        activate(&shell, cx, Control::HistoryBaseline(last));
        let row = format!("history-row-HistoryBaseline({last})");
        let row_bounds = cx.debug_bounds(row.clone().leak()).unwrap();
        shell.read_with(cx, |s, _| {
            inside(row_bounds, s.history_baseline_scroll.bounds())
        });
        record(
            &shell,
            cx,
            &root,
            name("history-last-baseline"),
            &[&row],
            &mut ledger,
        );
        owned.join_after_ui();
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(few.path())).unwrap();
    assert_eq!(
        few_baseline,
        store.load_project(project).unwrap().project.active_baseline
    );
    fs::write(
        root.join(if preflight {
            "phase4-preflight-matrix.json"
        } else {
            "phase4-matrix.json"
        }),
        serde_json::to_vec_pretty(&ledger).unwrap(),
    )
    .unwrap();
}
