// SPDX-License-Identifier: MPL-2.0
//! Real actor/storage failure routes, not a cloned no-frame presentation.
use super::*;
use gpui::TestAppContext;
use std::fs;

#[gpui::test]
fn phase6_exact_details_and_baseline_paging_are_real_mounted_actions(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let baseline = store
        .load_project(project)
        .unwrap()
        .project
        .active_baseline
        .unwrap();
    let preview = store.export_preview(project, baseline).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    vcx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            workflow_refinement_tests::load_review(s, project, cx);
            s.focus.screen = Screen::History;
            s.history_section = 1;
            assert!(s.control_mounted(Control::Phase6(11)));
            assert!(s.control_mounted(Control::Phase6(12)));
            assert!(!s.control_enabled(Control::Phase6(11), cx));
            s.export_preview = Some(preview.clone());
            assert!(s.control_enabled(Control::Details(Detail::Portability), cx));
            let request = s.next_request;
            s.activate_control(Control::Details(Detail::Portability), window, cx);
            assert!(s.details.contains(&Detail::Portability));
            assert!(!s.details.contains(&Detail::History));
            assert_eq!(s.next_request, request);
            s.activate_control(Control::Phase6(7), window, cx);
            assert!(!s.details.contains(&Detail::Portability));
            assert!(!s.control_mounted(Control::Details(Detail::Portability)));
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn phase6_missing_and_corrupt_comparison_originals_fail_locally(cx: &mut TestAppContext) {
    for missing in [true, false] {
        let dir = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(dir.path());
        let (store, project, run, _, _) = approval_fixture::fixture(dir.path());
        let record = store.load_run(run).unwrap();
        // Retain an actual validated metadata snapshot, as an already-open
        // workspace does before an external disk failure. No frame is cloned.
        let evidence = store.project_evidence(project).unwrap();
        let key = record.captures.last().unwrap().metadata.key.clone();
        let path = paths
            .artifacts()
            .join(record.captures.last().unwrap().image.original.as_str());
        if missing {
            fs::remove_file(&path).unwrap();
        } else {
            fs::write(&path, b"not a valid PNG").unwrap();
        }
        let original_baseline = store.load_project(project).unwrap().project.active_baseline;
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(paths.clone());
        let owned = jobs.clone();
        let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        vcx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                workflow_refinement_tests::finish_jobs(s, cx);
                s.focus.screen = Screen::Results;
                s.queue(Work::Load(project), cx);
                workflow_refinement_tests::finish_jobs(s, cx);
                s.evidence = evidence.clone();
                s.evidence_dirty = false;
                s.evidence_read_failed = false;
                s.selected_key = Some(key.clone());
                s.reload_viewer(cx);
                workflow_refinement_tests::finish_jobs(s, cx);
                assert!(s.viewer.is_none());
                assert!(
                    s.error.is_some(),
                    "real artifact read must retain its failure cause"
                );
                assert_eq!(s.selected_key.as_ref(), Some(&key));
                let (path, profile) = s.inspection_identity();
                assert!(!path.contains("not loaded"));
                assert!(profile.contains("96"));
                assert!(
                    !s.control_mounted(Control::Guided(44)),
                    "no irrelevant Analysis"
                );
                assert!(!s.approval_eligible(&key));
                let requests = s.next_request;
                s.activate_control(Control::Guided(44), window, cx);
                assert_eq!(requests, s.next_request);
            })
        });
        owned.join_after_ui();
        let store = regressee_storage::Store::open(paths).unwrap();
        assert_eq!(
            store.load_project(project).unwrap().project.active_baseline,
            original_baseline
        );
    }
}
