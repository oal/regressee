// SPDX-License-Identifier: MPL-2.0
//! Actual Jobs/Store approval with distinct persisted pages; NOT OS input.
use super::*;
use gpui::{TestAppContext, VisualTestContext};

fn workflow(cx: &mut TestAppContext, render: bool) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, run, _) = bulk_fixture::fixture(dir.path(), 100);
    let original = store.load_project(project).unwrap();
    let old = store
        .load_baseline(original.project.active_baseline.unwrap())
        .unwrap();
    let record = store.load_run(run).unwrap();
    let keys = [
        record.run.targets[0].key.clone(),
        record.run.targets[2].key.clone(),
    ];
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, vcx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        vcx.simulate_resize(size(px(width), px(height)));
        vcx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                workflow_refinement_tests::load_review(s, project, cx);
                s.configure_review_geometry(width);
                s.selected_key = Some(keys[0].clone());
                s.reload_viewer(cx);
                workflow_refinement_tests::finish_jobs(s, cx);
                assert!(s.approval_eligible(&keys[0]));
                assert!(!s.approval_eligible(&record.run.targets[99].key));
                s.activate_control(Control::Phase5(23), window, cx);
                s.activate_control(Control::Phase5(24), window, cx);
                workflow_refinement_tests::finish_jobs(s, cx);
                assert_eq!(s.approval.as_ref().unwrap().selected, vec![keys[0].clone()]);
                assert_eq!(s.approval.as_ref().unwrap().inherited, 99);
            })
        });
        snapshot(&mut *vcx, width, height, "one-long-consent", render);
        assert_initial_consent(vcx, &shell);
        vcx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.dismiss(&Dismiss, window, cx);
                assert!(!s.focus.approval_dialog);
                assert!(s.approval_selection.contains(&keys[0]));
                s.selected_key = Some(keys[1].clone());
                s.reload_viewer(cx);
                workflow_refinement_tests::finish_jobs(s, cx);
                s.activate_control(Control::Phase5(23), window, cx);
                s.crawl_fields[9].update(cx, |f, cx| f.set_value("no-matches-phase7", cx));
            })
        });
        vcx.run_until_parked();
        vcx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert_eq!(s.approval_selection.len(), 2);
                assert_eq!(s.selected_key, Some(keys[1].clone()));
                s.activate_control(Control::Phase5(24), window, cx);
                workflow_refinement_tests::finish_jobs(s, cx);
                let preview = s.approval.as_ref().unwrap();
                assert_eq!(preview.selected.len(), 2);
                assert_eq!(preview.inherited, 98);
                assert_eq!(preview.baseline.references.len(), 100);
            })
        });
        snapshot(&mut *vcx, width, height, "filtered-two-consent", render);
        assert_initial_consent(vcx, &shell);
        vcx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                for _ in 0..99 {
                    s.activate_control(Control::Guided(49), window, cx);
                }
                assert_eq!(s.approval_reference_index, 99);
            })
        });
        vcx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        shell.update(vcx, |s, cx| {
            s.approval_provenance_scroll.set_offset(gpui::point(
                px(0.),
                -s.approval_provenance_scroll.max_offset().height,
            ));
            cx.notify();
        });
        vcx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        let row = vcx.debug_bounds("approval-reference-99").unwrap();
        shell.read_with(vcx, |s, _| {
            let owner = s.approval_provenance_scroll.bounds();
            assert!(
                row.left() >= owner.left()
                    && row.right() <= owner.right()
                    && row.top() >= owner.top()
                    && row.bottom() <= owner.bottom(),
                "last real reference {row:?} outside actual {owner:?}"
            );
            #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
            if render {
                std::fs::write(ui_renderer_capture::output_root().unwrap().join(format!("bulk-{width:.0}x{height:.0}-geometry.json")), serde_json::to_vec_pretty(&serde_json::json!({"row":format!("{row:?}"),"actual_inner_scroll_owner":format!("{owner:?}"),"fully_visible":true,"reference_index":99,"references":100,"selected":2,"inherited":98,"distinct_persisted_synthetic_pages":100,"os_input_verified":false})).unwrap()).unwrap();
            }
        });
        snapshot(&mut *vcx, width, height, "hundred-reference-tail", render);
        vcx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                // Both sizes preview/cancel the same stored identities; only the final
                // pass commits through the real worker, not Store in the UI callback.
                if width == 820. {
                    s.activate_control(Control::Phase5(25), window, cx);
                    workflow_refinement_tests::finish_jobs(s, cx);
                } else {
                    s.dismiss(&Dismiss, window, cx);
                    // The cached identical preview must reset its real tail scroll.
                    s.activate_control(Control::Phase5(24), window, cx);
                    assert_eq!(s.approval_reference_index, 0);
                    assert_eq!(s.approval_provenance_scroll.offset().y, px(0.));
                    s.dismiss(&Dismiss, window, cx);
                    s.approval_selection.clear();
                    s.crawl_fields[9].update(cx, |f, cx| f.set_value("", cx));
                }
            })
        });
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(store.baseline_history(project).unwrap().len(), 2);
    assert_eq!(
        serde_json::to_value(store.load_baseline(old.id).unwrap()).unwrap(),
        serde_json::to_value(&old).unwrap()
    );
    let active = store
        .load_baseline(
            store
                .load_project(project)
                .unwrap()
                .project
                .active_baseline
                .unwrap(),
        )
        .unwrap();
    for reference in active.references {
        assert_eq!(reference.run == run, keys.contains(&reference.key));
    }
}

fn assert_initial_consent(cx: &mut VisualTestContext, shell: &Entity<Shell>) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
    let summary = cx.debug_bounds("approval-impact-summary").unwrap();
    shell.read_with(cx, |s, _| {
        let owner = s.approval_provenance_scroll.bounds();
        assert_eq!(s.approval_reference_index, 0);
        assert_eq!(s.approval_provenance_scroll.offset().y, px(0.));
        assert!(summary.top() >= owner.top() && summary.bottom() <= owner.bottom());
    });
}

fn snapshot(cx: &mut VisualTestContext, width: f32, height: f32, label: &str, render: bool) {
    #[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
    if render {
        let root = ui_renderer_capture::output_root().unwrap();
        ui_renderer_capture::capture(
            cx,
            &root,
            &format!("all-{width:.0}x{height:.0}-bulk-{label}"),
        )
        .unwrap();
    }
    let _ = (cx, width, height, label, render);
}

#[gpui::test]
fn phase7_real_filtered_selection_preview_cancel_and_commit_at_both_sizes(cx: &mut TestAppContext) {
    workflow(cx, false);
}

#[cfg(all(feature = "ui-renderer-capture", target_os = "linux"))]
#[gpui::test]
#[ignore = "explicit source-bound native offscreen batch, not OS input"]
fn phase7_renderer_capture_distinct_bulk_consent(cx: &mut TestAppContext) {
    cx.enable_offscreen_renderer().unwrap();
    workflow(cx, true);
}
