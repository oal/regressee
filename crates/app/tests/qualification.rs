// SPDX-License-Identifier: MPL-2.0
//! Storage-scale safety qualification; generated PNGs are not browser captures.
#[path = "support/bulk_fixture.rs"]
mod bulk_fixture;
#[path = "support/private_paths.rs"]
mod private_paths;
use regressee_storage::{AppPaths, Store};
use std::{collections::BTreeSet, sync::atomic::AtomicBool};

#[test]
fn phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention() {
    for pages in [100, 1000] {
        let start = std::time::Instant::now();
        let dir = tempfile::tempdir().unwrap();
        let (mut store, project, run, analysis) = bulk_fixture::fixture(dir.path(), pages);
        let original = store.load_project(project).unwrap();
        let old_id = original.project.active_baseline.unwrap();
        let old = store.load_baseline(old_id).unwrap();
        let immutable = serde_json::to_value(&old).unwrap();
        let record = store.load_run(run).unwrap();
        assert_eq!(record.run.targets.len(), pages);
        assert_eq!(
            record
                .run
                .targets
                .iter()
                .map(|t| t.key.page)
                .collect::<BTreeSet<_>>()
                .len(),
            pages
        );
        assert_eq!(
            record
                .run
                .targets
                .iter()
                .map(|t| t.requested.as_str())
                .collect::<BTreeSet<_>>()
                .len(),
            pages
        );
        assert_eq!(
            old.references
                .iter()
                .map(|r| &r.artifact)
                .collect::<BTreeSet<_>>()
                .len(),
            pages
        );
        assert_eq!(
            record
                .captures
                .iter()
                .map(|c| &c.image.original)
                .collect::<BTreeSet<_>>()
                .len(),
            pages - 1
        );
        let failed = record.run.targets.last().unwrap().key.clone();
        assert!(store.approval_preview(analysis, &[failed]).is_err());
        let key = record.run.targets[0].key.clone();
        let one = store
            .approval_preview(analysis, std::slice::from_ref(&key))
            .unwrap();
        assert_eq!(one.inherited, pages - 1);
        // A worker-local verified snapshot must NEVER survive between preview
        // and commit: external corruption after consent still fails closed.
        let artifact = record.captures[0].image.original.clone();
        let artifact_path = store.artifact_path(&artifact).unwrap();
        let bytes = std::fs::read(&artifact_path).unwrap();
        std::fs::write(&artifact_path, b"corrupted after preview").unwrap();
        assert!(store.accept_selected(&one, original.revision).is_err());
        std::fs::write(&artifact_path, bytes).unwrap();
        let accepted = store.accept_selected(&one, original.revision).unwrap();
        assert!(
            store.accept_selected(&one, original.revision).is_err(),
            "stale baseline identity accepted twice"
        );
        // An old check cannot approve against a different active baseline, even
        // after reanalysis. Restore explicitly before this independent many case.
        assert!(
            store
                .approval_preview(analysis, std::slice::from_ref(&key))
                .is_err()
        );
        let restored = store.restore_baseline(old_id, accepted.revision).unwrap();
        let a = regressee_app::review::analyze_run(
            &mut store,
            run,
            regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
            &AtomicBool::new(false),
        )
        .unwrap();
        let keys: Vec<_> = record
            .run
            .targets
            .iter()
            .skip(2)
            .step_by(2)
            .take(20)
            .map(|t| t.key.clone())
            .collect();
        let many = store.approval_preview(a.revision.id, &keys).unwrap();
        assert_eq!(many.selected.len(), 20);
        let accepted = store.accept_selected(&many, restored.revision).unwrap();
        let active = store
            .load_baseline(accepted.project.active_baseline.unwrap())
            .unwrap();
        for r in &active.references {
            let replaced = keys.contains(&r.key);
            assert_eq!(r.run == run, replaced, "unselected reference changed");
        }
        assert_eq!(
            serde_json::to_value(store.load_baseline(old_id).unwrap()).unwrap(),
            immutable
        );
        assert!(
            store
                .deletion_preview(
                    project,
                    regressee_storage::DeleteItem::Run(old.references[0].run)
                )
                .is_err(),
            "retained shared source deletable"
        );
        let zip = dir.path().join("qualification.zip");
        let export = store.export_preview(project, active.id).unwrap();
        store
            .export_archive(&export, &zip, &AtomicBool::new(false))
            .unwrap();
        let recipient_root = dir.path().join("recipient");
        let mut recipient = Store::open(AppPaths::isolated(&recipient_root)).unwrap();
        let preview = recipient
            .inspect_archive(&zip, &AtomicBool::new(false))
            .unwrap()
            .preview;
        let imported = recipient
            .import_archive(&zip, &preview, &AtomicBool::new(false))
            .unwrap();
        for env in &imported.project.environments {
            assert!(
                recipient
                    .session_reference(imported.project.id, env.id)
                    .unwrap()
                    .is_none()
            );
        }
        let imported_baseline = recipient
            .load_baseline(imported.project.active_baseline.unwrap())
            .unwrap();
        for (r, source) in imported_baseline.references.iter().zip(&active.references) {
            assert_eq!(r.artifact, source.artifact);
            assert_eq!(
                std::fs::read(recipient.artifact_path(&r.artifact).unwrap()).unwrap(),
                std::fs::read(store.artifact_path(&source.artifact).unwrap()).unwrap()
            );
        }
        drop(store);
        let reopened = Store::open(AppPaths::isolated(dir.path())).unwrap();
        assert_eq!(reopened.load_run(run).unwrap().run.targets.len(), pages);
        assert_eq!(reopened.baseline_history(project).unwrap().len(), 3);
        eprintln!(
            "Phase7: {pages} distinct persisted pages, unique originals, selective 1+20, stale rejection, retained source protection, ZIP byte identity: {:.3}s (NOT browser/native/user throughput)",
            start.elapsed().as_secs_f64()
        );
        if let Some(parent) = std::env::var_os("REGRESSEE_PHASE7_STORAGE_EVIDENCE") {
            let parent = std::path::PathBuf::from(parent);
            private_paths::private_directory(&parent).unwrap();
            let destination = parent.join(format!("storage-{pages}"));
            assert!(
                !destination.exists(),
                "never overwrite prior qualification evidence"
            );
            drop(reopened);
            std::fs::rename(dir.keep(), &destination).unwrap();
            std::fs::write(destination.join("qualification.json"), serde_json::to_vec_pretty(&serde_json::json!({"pages":pages,"project":project,"run":run,"analysis":analysis,"original_baseline":old_id,"active_baseline":active.id,"retained_versions":3,"distinct_baseline_originals":pages,"distinct_check_originals":pages-1,"selected_single":1,"selected_many":20,"corruption_after_preview_rejected":true,"stale_preview_rejected":true,"sessions_exported":false,"synthetic":true,"browser_throughput_verified":false,"os_input_verified":false})).unwrap()).unwrap();
            eprintln!(
                "Retained checkable distinct Store: {}",
                destination.display()
            );
        }
    }
}
