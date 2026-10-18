// SPDX-License-Identifier: MPL-2.0
//! Copy regressions through the real reply/presentation paths. No website visits.
use super::*;
use gpui::TestAppContext;

#[test]
fn routine_copy_does_not_expose_storage_units_or_internal_workflow_terms() {
    let sources = [
        ("note panel", include_str!("viewer_ux.rs")),
        ("review", include_str!("review_ux.rs")),
        ("note editor", include_str!("ui/text_input/multiline.rs")),
        ("workspace", include_str!("main.rs")),
        ("validation", include_str!("local_ux.rs")),
        ("form recovery", include_str!("form_recovery.rs")),
        ("loading", include_str!("loading_ux.rs")),
        ("history", include_str!("workspace_history.rs")),
        ("settings", include_str!("app_settings_ux.rs")),
        ("browser", include_str!("runtime_ux.rs")),
        (
            "failure recovery",
            include_str!("../../app/src/recovery.rs"),
        ),
        ("setup and capture", include_str!("workflow.rs")),
        ("login", include_str!("../../app/src/auth.rs")),
        ("task recovery", include_str!("task.rs")),
        ("recovery navigation", include_str!("recovery.rs")),
        ("task status", include_str!("task_ux.rs")),
        ("operation status", include_str!("jobs.rs")),
        ("operation labels", include_str!("activity.rs")),
        ("application replies", include_str!("../../app/src/jobs.rs")),
        (
            "shared note validation",
            include_str!("../../core/src/domain.rs"),
        ),
    ];
    for (surface, source) in sources {
        let source = source.to_lowercase();
        for phrase in [
            "return to screenshots",
            "utf-8 bytes maximum",
            "shorten to 8,192 utf-8 bytes",
            "8,192-byte limit",
            "unsaved · {} bytes",
            "saved · {} bytes",
            "parallel active pages",
            "identity-provider origins",
            "source origin or section base",
            "review the evidence",
            "evidence kept",
            "saved evidence size",
            "stored objects",
            "revalidate ignored areas explicitly",
            "save your website locally",
            "website saved locally",
            "save website & continue",
            "find pages & preview",
            "i'm ready",
            "production → staging mapping",
            "approve / replace this base mapping",
            "local task workspace",
            "local project workspace",
            "diagnostic file saved",
            "saving diagnostic file",
            "preparing diagnostic preview",
            "diagnostic preview unavailable",
            "diagnostic file could not be saved",
            "check its folder permissions in app settings",
        ] {
            assert!(!source.contains(phrase), "{surface} exposes: {phrase}");
        }
    }
    let message = regressee_core::domain::REVIEW_NOTE_LIMIT_MESSAGE;
    assert!(message.contains("Shorten it before saving"));
    assert!(message.contains("Your draft is kept") && message.contains("nothing was saved"));
    assert!(!message.contains("bytes") && !message.contains("UTF"));
}

#[gpui::test]
fn screenshot_context_names_roles_without_repeating_default_environment(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        workflow_refinement_tests::load_review(s, project, cx);
        assert_eq!(
            s.inspection_sources(),
            ("Saved baseline".into(), "This check".into())
        );
        assert!(!s.inspection_identity().1.contains("Website"));
        // Source names belong to the recorded captures, not today's settings.
        s.project.as_mut().unwrap().project.environments[0].name = "Today's unrelated name".into();
        s.evidence.latest.as_mut().unwrap().run.environment.name = "Staging".into();
        s.viewer.as_mut().unwrap().before_environment = Some("Production".into());
        assert_eq!(
            s.inspection_sources(),
            (
                "Saved baseline · Production".into(),
                "This check · Staging".into()
            )
        );
        s.evidence.latest.as_mut().unwrap().run.environment.name = "Production".into();
        assert_eq!(
            s.inspection_sources(),
            ("Saved baseline".into(), "This check · Production".into())
        );
        let summary = s.capture_summary();
        assert!(summary.iter().any(
            |(label, value)| label == "Before environment" && value.starts_with("Production\n")
        ));
        assert!(summary.iter().any(
            |(label, value)| label == "After environment" && value.starts_with("Production\n")
        ));
        let run = s.evidence.latest.as_ref().unwrap().run.id;
        s.history_inspection = Some((project, run));
        assert_eq!(s.inspection_sources().1, "Recorded check · Production");
        s.history_inspection = None;
        s.viewer.as_mut().unwrap().before_environment = None;
        s.history = None;
        assert_eq!(
            s.inspection_sources().0,
            "Saved baseline · Source not loaded"
        );
        // Captures and previews never imply that baseline consent happened.
        s.evidence.comparison = None;
        let reference = s.evidence.baseline.as_ref().unwrap().references[0].clone();
        s.evidence.latest = Some(store.load_run(reference.run).unwrap());
        assert_eq!(s.inspection_sources().1, "Saved baseline");
        s.evidence.baseline = None;
        s.focus.candidate = true;
        assert_eq!(s.review_title(), "Inspect capture");
        assert_eq!(s.inspection_sources().1, "Not yet a baseline");
        s.evidence.latest.as_mut().unwrap().run.intent = RunIntent::RepresentativePreview;
        assert_eq!(s.inspection_sources().1, "Screenshot preview");
        assert_eq!(
            workspace_history::history_run_type(&RunIntent::RepresentativePreview),
            "Screenshot preview"
        );
        // History intentionally returns no active baseline for capture runs.
        // That does not establish that these screenshots were never saved as one.
        s.evidence = store.inspect_run(project, reference.run).unwrap();
        assert!(s.evidence.baseline.is_none());
        s.history_inspection = Some((project, reference.run));
        s.focus.candidate = false;
        assert_eq!(s.inspection_sources().1, "Recorded capture");
    });
    owned.join_after_ui();
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn network_copy_only_claims_loading_during_its_actual_read(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        workflow_refinement_tests::finish_jobs(s, cx);
        s.network_status = None;
        s.pending = None;
        s.focus.screen = Screen::Settings;
        s.details.insert(Detail::ManagedNetwork);
        assert!(s.control_mounted(Control::Guided(42)));
        assert!(
            s.network_loading_help()
                .unwrap()
                .contains("Retry loading saved settings")
        );
        s.pending = Some(u64::MAX);
        assert!(!s.control_mounted(Control::Guided(42)));
        s.progress = Some((Work::NetworkStatus.activity_label().into(), 0, None));
        assert_eq!(
            s.network_loading_help(),
            Some("Reading saved network settings…")
        );
        s.progress = Some((Work::InspectStorage.activity_label().into(), 0, None));
        let waiting = s.network_loading_help().unwrap();
        assert!(waiting.contains("Wait for the current operation"));
        assert!(!waiting.contains("Choose Retry"));
        s.pending = None;
        s.progress = None;
        assert!(s.control_mounted(Control::Guided(42)));
        s.network_status = Some("Saved settings loaded.".into());
        assert_eq!(s.network_loading_help(), None);
    });
    owned.join_after_ui();
}

#[test]
fn skipped_and_stopped_captures_do_not_claim_an_attempt_failed() {
    use regressee_core::domain::{CaptureError, TargetOutcome};
    for outcome in [
        TargetOutcome::Cancelled,
        TargetOutcome::Skipped {
            reason: "Excluded by choice".into(),
        },
    ] {
        assert_eq!(
            loading_ux::unavailable_heading(&outcome),
            "Screenshot not captured"
        );
    }
    assert_eq!(
        loading_ux::unavailable_heading(&TargetOutcome::Failed {
            code: CaptureError::Timeout
        }),
        "This page could not be captured"
    );
    assert_eq!(
        loading_ux::unavailable_heading(&TargetOutcome::Pending),
        "Screenshot unavailable"
    );
}

#[gpui::test]
fn review_note_status_uses_plain_language_and_close_keeps_the_draft(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        workflow_refinement_tests::load_review(s, project, cx);
        assert_eq!(s.note_status(cx), ("No changes".into(), false));
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Phase5(37), window, cx);
                let draft = "Check the heading.\n日本語 · café · 🦀";
                s.analysis_fields[9].update(cx, |input, cx| input.set_value(draft, cx));
                assert_eq!(s.note_status(cx), ("Unsaved changes".into(), false));
                let requests = s.next_request;
                s.activate_control(Control::Phase5(39), window, cx);
                assert!(!s.review_note_open);
                assert_eq!(s.next_request, requests, "Close does not save");
                s.activate_control(Control::Phase5(37), window, cx);
                assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), draft);
                assert_eq!(s.note_status(cx), ("Unsaved changes".into(), false));
                s.activate_control(Control::Phase5(39), window, cx);
            });
        });
    }
    owned.join_after_ui();
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

/// Simulate only the receipt of a successful baseline save. The stored fixture
/// stays untouched; this exercises the same message path as the worker reply.
pub(super) fn baseline_saved_reply(s: &mut Shell, cx: &mut Context<Shell>) {
    let record = s.project.clone().unwrap();
    assert!(record.project.active_baseline.is_some());
    s.project.as_mut().unwrap().project.active_baseline = None;
    s.focus.candidate = true;
    s.pending = Some(u64::MAX);
    s.apply_update(
        Update {
            id: u64::MAX,
            reply: Reply::Project(Box::new(record)),
        },
        cx,
    );
}

#[gpui::test]
fn success_copy_names_the_outcome_and_baseline_consequences(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        workflow_refinement_tests::load_review(s, project, cx);
        baseline_saved_reply(s, cx);
        let message = s.notice.as_ref().unwrap();
        assert!(message.starts_with("Baseline saved."));
        assert!(message.contains("compare against these screenshots"));
        assert!(message.contains("same page list"));
        assert!(message.contains("Earlier versions are available in History"));
        assert!(!message.contains("explicitly") && !message.contains("inventory"));
        workflow_refinement_tests::finish_jobs(s, cx);

        s.pending = Some(u64::MAX);
        s.apply_update(Update { id: u64::MAX, reply: Reply::SavedMask }, cx);
        assert_eq!(s.notice.as_deref(), Some("Ignored areas saved for these before-and-after screenshots. Original images are unchanged."));
        s.analysis_due = None;
        workflow_refinement_tests::finish_jobs(s, cx);
    });
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    owned.join_after_ui();
}

#[test]
fn page_list_status_explains_limits_without_internal_queue_terms() {
    use regressee_core::domain::DiscoveryCoverage;
    let complete = DiscoveryCoverage::Complete.label();
    assert!(complete.contains("visible links") && complete.contains("pages may still be missing"));
    assert_eq!(
        DiscoveryCoverage::Interrupted.label(),
        "Page search interrupted. The page list is incomplete."
    );
    assert_eq!(
        DiscoveryCoverage::Limited {
            reason: "Some pages were excluded.".into()
        }
        .label(),
        "Limited page list: Some pages were excluded."
    );
    assert_eq!(
        DiscoveryCoverage::Limited {
            reason: " Page limit reached ".into()
        }
        .label(),
        "Limited page list: Page limit reached."
    );
    for message in [complete, DiscoveryCoverage::Interrupted.label()] {
        assert!(!message.contains("queue") && !message.contains("exhausted"));
    }
}

#[gpui::test]
fn deletion_copy_uses_singular_and_plural_counts(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let before = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        workflow_refinement_tests::load_review(s, project, cx);
        s.focus.screen = Screen::History;
        s.history_section = 2;
        s.task = None;
    });
    for (count, analysis, file) in [
        (0, "analyses", "files"),
        (1, "analysis", "file"),
        (2, "analyses", "files"),
    ] {
        let requests = shell.read_with(cx, |s, _| s.next_request);
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.deletion_preview = Some(regressee_storage::DeletionPreview {
                    project,
                    revision: s.project.as_ref().unwrap().revision,
                    item: regressee_storage::DeleteItem::Run(
                        s.evidence.latest.as_ref().unwrap().run.id,
                    ),
                    fingerprint: "copy-preview-only".into(),
                    bytes: 42,
                    shared_bytes: 84,
                    objects: count,
                    analyses: count,
                    scope: None,
                });
                s.activate_control(Control::Phase6(6), window, cx);
            });
        });
        let message = cx.pending_prompt().unwrap().1;
        assert!(message.contains(&format!(
            "{count} {analysis} and {count} {file} used only by this record"
        )));
        assert!(message.contains("This cannot be undone"));
        cx.simulate_prompt_answer("Cancel");
        cx.run_until_parked();
        shell.read_with(cx, |s, _| assert_eq!(s.next_request, requests));
    }
    owned.join_after_ui();
    assert_eq!(
        before,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
