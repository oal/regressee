// SPDX-License-Identifier: MPL-2.0
//! Documentation screenshot: real GPUI and unmodified owned-loopback browser pixels.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use regressee_core::{domain::*, evidence::*};
use std::sync::atomic::AtomicBool;

#[gpui::test]
#[ignore = "explicit README screenshot from an isolated managed-browser consumer fixture"]
fn renderer_capture_readme_comparison(cx: &mut TestAppContext) {
    let output = output_root().unwrap();
    let source = PathBuf::from(
        std::env::var_os("REGRESSEE_README_DEMO_SOURCE")
            .expect("explicit copied loopback demo fixture required"),
    );
    assert!(source.is_absolute() && source.starts_with(output.parent().unwrap()));
    let ids: serde_json::Value =
        serde_json::from_slice(&fs::read(source.join("fixture.json")).unwrap()).unwrap();
    assert_eq!(ids["kind"], "actual-managed-Chromium-owned-loopback-demo");
    let source_store = regressee_storage::Store::open(AppPaths::isolated(&source)).unwrap();
    let project_id: ProjectId = serde_json::from_value(ids["project"].clone()).unwrap();
    let baseline_run = source_store
        .load_run(serde_json::from_value(ids["baseline_run"].clone()).unwrap())
        .unwrap();
    let changed_run = source_store
        .load_run(serde_json::from_value(ids["changed_run"].clone()).unwrap())
        .unwrap();
    let mut project = source_store.load_project(project_id).unwrap().project;
    assert!(
        project
            .sources
            .iter()
            .all(|url| url.host_str() == Some("127.0.0.1"))
    );
    assert!(
        project
            .environments
            .iter()
            .all(|environment| environment.access == AccessPolicy::Public)
    );
    for run in [&baseline_run, &changed_run] {
        assert_eq!(run.run.project, project_id);
        assert_eq!(run.run.state, JobState::Completed);
        assert_eq!(run.run.targets.len(), 3);
        assert!(run.captures.iter().all(|capture| {
            matches!(capture.metadata.runtime, RuntimeIdentity::Pinned { .. })
                && capture.metadata.capture_mode == "chromium-new-headless"
                && capture.metadata.width == 1440
        }));
    }

    // Rebuild only this documentation demo's baseline/check lineage. The browser
    // capture metadata, times and content-addressed originals remain untouched.
    let demo = tempfile::Builder::new()
        .prefix("readme-demo-")
        .tempdir_in(&output)
        .unwrap();
    let paths = AppPaths::isolated(demo.path());
    let mut store = regressee_storage::Store::open(paths.clone()).unwrap();
    for run in [&baseline_run, &changed_run] {
        for capture in &run.captures {
            let image = regressee_app::images::ingest(
                &mut store,
                &source
                    .join("data/artifacts")
                    .join(capture.image.original.as_str()),
                ArtifactKind::Screenshot,
                &AtomicBool::new(false),
            )
            .unwrap();
            assert_eq!(
                serde_json::to_value(image).unwrap(),
                serde_json::to_value(&capture.image).unwrap()
            );
        }
    }
    project.name = "Loopback website demo".into();
    project.active_baseline = None;
    store.save_project(&project, None).unwrap();
    store.freeze_inventory(&baseline_run.run.inventory).unwrap();
    let save = |store: &mut regressee_storage::Store, record: &RunRecord| {
        let mut record = record.clone();
        if let Some(queue) = &mut record.queue {
            queue.project_revision = store.load_project(project_id).unwrap().revision;
        }
        let mut pending = record.clone();
        pending.run.state = JobState::Running;
        pending.run.finished_at = None;
        pending.captures.clear();
        for target in &mut pending.run.targets {
            target.outcome = TargetOutcome::Pending;
        }
        store.begin_run(&pending).unwrap();
        store.finish_run(&record).unwrap();
    };
    save(&mut store, &baseline_run);
    let saved = store.activate_baseline(baseline_run.run.id, 1).unwrap();
    let mut check = changed_run.clone();
    check.run.intent = RunIntent::Check {
        baseline: saved.project.active_baseline.unwrap(),
    };
    save(&mut store, &check);
    let analysis = regressee_app::review::analyze_run(
        &mut store,
        check.run.id,
        regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
        &AtomicBool::new(false),
    )
    .unwrap();
    assert!(
        analysis
            .revision
            .results
            .iter()
            .any(|result| result.classification == Classification::Changed)
    );
    let frozen = serde_json::to_value(store.project_evidence(project_id).unwrap()).unwrap();
    let originals: Vec<_> = [&baseline_run, &changed_run]
        .into_iter()
        .flat_map(|run| {
            run.captures
                .iter()
                .map(|capture| capture.image.original.clone())
        })
        .collect();
    for original in &originals {
        store.verify_artifact(original).unwrap();
    }
    drop(source_store);
    drop(store);
    regressee_storage::AppPreferences {
        appearance: AppAppearance::Light,
        ..Default::default()
    }
    .save(&paths)
    .unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::agent_control_tests::bind(cx);
    let (jobs, events) = Jobs::start_without_runtime(paths.clone());
    let owner = jobs.clone();
    owner.appearance.wait_loaded();
    let (shell, view) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    view.simulate_resize(size(px(1440.), px(900.)));
    view.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load_review(s, project_id, cx);
            s.activate_control(Control::DismissFeedback, window, cx);
            s.choose(Choice::AllResults, cx);
            let key = s
                .evidence
                .latest
                .as_ref()
                .unwrap()
                .run
                .targets
                .iter()
                .find(|target| target.requested.path() == "/details")
                .unwrap()
                .key
                .clone();
            s.selected_key = Some(key);
            s.reload_viewer(cx);
            finish_jobs(s, cx);
            assert!(s.viewer_has_pixels() && !s.agent_banner_visible());
            assert!(s.error.is_none() && s.notice.is_none());
            assert_eq!(s.viewer_mode, 0);
            // The shipped side-by-side mode shows the two originals. Difference
            // pixels are available through the normal View selector, not a third
            // invented pane in the documentation image.
            assert!(!s.viewer.as_ref().unwrap().panes[2].is_empty());
            window.focus(&s.root_focus);
            cx.notify();
        })
    });
    view.update(|window, cx| {
        let _ = window.draw(cx);
    });
    view.run_until_parked();
    view.update(|window, cx| {
        let _ = window.draw(cx);
    });
    for selector in ["brand-lockup", "phase5-before", "phase5-after"] {
        let bounds = view.debug_bounds(selector).unwrap();
        crate::phase2_evidence_tests::inside(
            bounds,
            Bounds::new(gpui::point(px(0.), px(0.)), size(px(1440.), px(900.))),
        );
        assert!(bounds.size.height > px(40.));
    }
    capture(view, &output, "regressee-comparison").unwrap();
    fs::write(output.join("readme-source.json"), serde_json::to_vec_pretty(&serde_json::json!({
        "renderer": "real GPUI Blade offscreen", "website_pixels": "actual managed-Chromium loopback demo originals",
        "synthetic_metadata_lineage": "isolated documentation baseline/check rebuilt from genuine captured records",
        "width":1440,"height":900,"theme":"Unified neutrals light","originals": originals,
        "harness_sha256":hex::encode(Sha256::digest(include_bytes!("ui_readme_capture.rs"))),
        "os_window_qualification":false,"originals_modified":false
    })).unwrap()).unwrap();
    owner.join_after_ui();
    view.update(|window, _| window.remove_window());
    let store = regressee_storage::Store::open(paths).unwrap();
    assert_eq!(
        frozen,
        serde_json::to_value(store.project_evidence(project_id).unwrap()).unwrap()
    );
}
