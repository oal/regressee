// SPDX-License-Identifier: MPL-2.0
//! Saved-data refresh is not follow navigation or permission to overwrite drafts.
use super::*;
use regressee_core::domain::RunId;

#[gpui::test]
fn capture_completion_uses_its_run_before_a_separate_workspace_refresh(
    cx: &mut gpui::TestAppContext,
) {
    use crate::workflow_refinement_tests::{finish_jobs, load_review};
    let root = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(root.path());
    let latest = store.project_evidence(project).unwrap();
    drop(store);
    let mut executed = latest.clone();
    let run = &mut executed.latest.as_mut().unwrap().run;
    run.id = RunId::new();
    run.state = JobState::Paused;
    let executed_id = run.id;
    let latest_id = latest.latest.as_ref().unwrap().run.id;
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(root.path()));
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            finish_jobs(s, cx);
            load_review(s, project, cx);
            s.next_request += 1;
            s.begin_task(
                regressee_gui::task::Spec::for_work(&Work::CaptureInventory {
                    project,
                    preview: None,
                    check: false,
                    resume: Some(executed_id),
                })
                .unwrap(),
            );
            let request = s.next_request;
            s.pending = Some(request);
            s.focus.busy = true;
            s.focus.capturing = true;
            s.apply_update(
                Update {
                    id: request,
                    reply: Reply::Captured(Box::new(executed)),
                },
                cx,
            );
            let task = s.task.as_ref().unwrap();
            assert_eq!(
                task.result
                    .as_ref()
                    .unwrap()
                    .capture
                    .as_ref()
                    .unwrap()
                    .setup
                    .run,
                executed_id
            );
            assert_eq!(task.result.as_ref().unwrap().title, "Capture paused");
            assert_eq!(
                s.evidence.latest.as_ref().unwrap().run.id,
                latest_id,
                "Capture receipt must not hydrate the workspace"
            );
            assert!(s.evidence_dirty);
            s.continue_task(window, cx);
            finish_jobs(s, cx);
            assert_eq!(
                s.evidence.latest.as_ref().unwrap().run.id,
                latest_id,
                "Newest workspace data comes from its own Evidence command"
            );
        })
    });
    owner.join_after_ui();
}

#[gpui::test]
fn shared_engine_refresh_preserves_navigation_and_conflicting_draft_revision(
    cx: &mut gpui::TestAppContext,
) {
    let root = tempfile::tempdir().unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(root.path()));
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|_, cx| {
        shell.update(cx, |s, cx| {
            crate::workflow_refinement_tests::finish_jobs(s, cx);
            let project = Project::draft("Original".into(), "https://example.test/").unwrap();
            s.name
                .update(cx, |input, cx| input.set_value(&project.name, cx));
            s.address.update(cx, |input, cx| {
                input.set_value(project.sources[0].as_str(), cx)
            });
            s.draft_settings = project.clone();
            s.set_crawl_fields(cx);
            s.project = Some(ProjectRecord {
                project: project.clone(),
                revision: 1,
            });
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Website;
            let mut changed = project.clone();
            changed.name = "Saved elsewhere".into();
            s.refresh_saved_project(
                ProjectRecord {
                    project: changed,
                    revision: 2,
                },
                cx,
            );
            assert_eq!(s.project.as_ref().unwrap().revision, 2);
            assert_eq!(s.name.read(cx).content.as_ref(), "Saved elsewhere");
            assert_eq!(s.focus.screen, Screen::Setup);
            assert_eq!(s.focus.setup_stage, SetupStage::Website);
            s.name
                .update(cx, |input, cx| input.set_value("Unsent local draft", cx));
            let mut changed = project;
            changed.name = "Another saved revision".into();
            s.refresh_saved_project(
                ProjectRecord {
                    project: changed,
                    revision: 3,
                },
                cx,
            );
            assert_eq!(
                s.project.as_ref().unwrap().revision,
                2,
                "The draft must retain its conflict precondition"
            );
            assert_eq!(s.name.read(cx).content.as_ref(), "Unsent local draft");
            assert_eq!(s.focus.screen, Screen::Setup);
            assert_eq!(s.focus.setup_stage, SetupStage::Website);
        })
    });
    owner.join_after_ui();
}
