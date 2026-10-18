// SPDX-License-Identifier: MPL-2.0
//! Isolated synthetic evidence only: no browser, sessions or website visits.
use anyhow::Result;
use regressee_app::dom_snapshot::{compare_originals, original_source};
use regressee_core::{dom_snapshot::*, domain::*};
use regressee_storage::{AppPaths, Store};
use std::sync::atomic::AtomicBool;

fn snapshot(text: &str) -> Result<Vec<u8>> {
    DomSnapshot {
        nodes: vec![
            DomNode::Document,
            DomNode::Element {
                depth: 1,
                namespace: HTML_NAMESPACE.into(),
                name: "html".into(),
                prefix: None,
                attributes: vec![],
            },
            DomNode::Element {
                depth: 2,
                namespace: HTML_NAMESPACE.into(),
                name: "body".into(),
                prefix: None,
                attributes: vec![],
            },
            DomNode::Text {
                depth: 3,
                text: text.into(),
            },
        ],
    }
    .canonical_bytes()
}

#[test]
fn exact_saved_dom_comparison_is_independent_of_display_budget_and_reopen() -> Result<()> {
    let root = tempfile::tempdir()?;
    let paths = AppPaths::isolated(root.path());
    let mut store = Store::open(paths.clone())?;
    let before = store.put_artifact(
        snapshot("hydrated before\n  significant whitespace")?.as_slice(),
        ArtifactKind::DomSnapshot,
    )?;
    let after = store.put_artifact(
        snapshot("hydrated after\n significant whitespace")?.as_slice(),
        ArtifactKind::DomSnapshot,
    )?;
    drop(store);
    let store = Store::open(paths)?;
    let policy = DomSnapshotPolicy::default();
    let cancel = AtomicBool::new(false);
    let equal = compare_originals(
        &store,
        &before.id,
        &before.id,
        policy,
        &DomDiffOptions::default(),
        &cancel,
    )?;
    assert!(!equal.changed);
    assert!(equal.hunks.is_empty());
    let changed = compare_originals(
        &store,
        &before.id,
        &after.id,
        policy,
        &DomDiffOptions::default(),
        &cancel,
    )?;
    assert!(changed.changed);
    assert!(changed.added_lines > 0 && changed.removed_lines > 0);
    let bounded = compare_originals(
        &store,
        &before.id,
        &after.id,
        policy,
        &DomDiffOptions {
            max_work: 0,
            max_output_lines: 0,
            ..Default::default()
        },
        &cancel,
    )?;
    assert!(bounded.changed);
    assert!(bounded.display_limited);
    assert!(bounded.hunks.is_empty());
    assert!(original_source(&store, &after.id, policy)?.contains("hydrated after"));
    assert_eq!(
        store.summary()?.artifacts,
        2,
        "DOM comparison must not manufacture PNG derivatives"
    );
    Ok(())
}

#[test]
fn saved_dom_reads_reject_wrong_kind_corruption_unknown_policy_and_cancellation() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut store = Store::open(AppPaths::isolated(root.path()))?;
    let bytes = snapshot("inert </script></pre><img src=private>&\" text")?;
    let dom = store.put_artifact(bytes.as_slice(), ArtifactKind::DomSnapshot)?;
    let wrong = store.put_artifact(b"not a snapshot".as_slice(), ArtifactKind::Screenshot)?;
    assert!(original_source(&store, &wrong.id, DomSnapshotPolicy::default()).is_err());
    assert!(original_source(&store, &dom.id, DomSnapshotPolicy { version: u32::MAX }).is_err());
    let cancelled = compare_originals(
        &store,
        &dom.id,
        &dom.id,
        DomSnapshotPolicy::default(),
        &DomDiffOptions::default(),
        &AtomicBool::new(true),
    )
    .unwrap_err();
    assert!(cancelled.is::<regressee_core::analysis_engine::Cancelled>());
    let cancelled =
        regressee_app::dom_snapshot::analyze_run(&mut store, RunId::new(), &AtomicBool::new(true))
            .unwrap_err();
    assert!(cancelled.is::<regressee_core::analysis_engine::Cancelled>());
    let source = original_source(&store, &dom.id, DomSnapshotPolicy::default())?;
    assert!(
        source.contains("</script></pre><img src=private>"),
        "ordinary markup-looking text remains inert source"
    );
    let path = store.artifact_path(&dom.id)?;
    std::fs::write(path, snapshot("corrupt but valid canonical")?)?;
    assert!(original_source(&store, &dom.id, DomSnapshotPolicy::default()).is_err());
    Ok(())
}

#[test]
fn excluded_payload_is_never_published_even_with_a_correct_content_hash() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut store = Store::open(AppPaths::isolated(root.path()))?;
    let payload = b"[\"regressee-dom\",1]\n[\"document\",0]\n[\"element\",1,\"http://www.w3.org/1999/xhtml\",\"script\",[]]\n[\"text\",2,\"private hydration secret\"]\n";
    let id = dom_artifact_id(payload);
    let error = store
        .put_artifact(payload.as_slice(), ArtifactKind::DomSnapshot)
        .unwrap_err();
    assert!(!format!("{error:#}").contains("private hydration secret"));
    assert!(!store.paths().artifacts().join(id.as_str()).exists());
    assert_eq!(store.summary()?.artifacts, 0);
    Ok(())
}
