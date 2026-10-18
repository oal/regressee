// SPDX-License-Identifier: MPL-2.0
//! DOM-only retained-original services. Never starts a browser or an image path.
use anyhow::{Context, Result, ensure};
use regressee_core::{dom_snapshot::*, domain::*, evidence::*};
use regressee_storage::Store;
use std::sync::atomic::{AtomicBool, Ordering};

fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        return Err(regressee_core::analysis_engine::Cancelled.into());
    }
    Ok(())
}

/// Inert plaintext for a verified complete original. Call on the blocking owner,
/// not the UI thread; the return value is untrusted text, never HTML markup.
pub fn original_source(
    store: &Store,
    id: &ArtifactId,
    policy: DomSnapshotPolicy,
) -> Result<String> {
    policy.validate()?;
    let (_, snapshot) = verified_original(store, id)?;
    snapshot.format_source()
}
fn verified_original(store: &Store, id: &ArtifactId) -> Result<(Vec<u8>, DomSnapshot)> {
    let (artifact, bytes) = store.read_verified_artifact(id, MAX_DOM_BYTES as u64)?;
    ensure!(
        artifact.kind == ArtifactKind::DomSnapshot,
        "This artifact is not a DOM snapshot"
    );
    let snapshot = DomSnapshot::from_canonical(&bytes)?;
    Ok((bytes, snapshot))
}

/// Shared bounded diff over saved originals, with no navigation or pixel policy.
pub fn compare_originals(
    store: &Store,
    before: &ArtifactId,
    after: &ArtifactId,
    policy: DomSnapshotPolicy,
    options: &DomDiffOptions,
    cancel: &AtomicBool,
) -> Result<DomTextDiff> {
    check_cancel(cancel)?;
    policy.validate()?;
    let (before_bytes, before_snapshot) = verified_original(store, before)?;
    let (after_bytes, after_snapshot) = verified_original(store, after)?;
    let changed = before_bytes != after_bytes;
    let before_source = before_snapshot.format_source()?;
    let after_source = after_snapshot.format_source()?;
    let diff = diff_text(&before_source, &after_source, options, || {
        cancel.load(Ordering::Acquire)
    })?;
    // Both inputs have passed canonical validation; canonical identities, not
    // viewer output truncation or sensitivity, establish exact equality.
    ensure!(
        diff.changed == changed,
        "DOM source formatting changed exact equality"
    );
    check_cancel(cancel)?;
    Ok(diff)
}

/// One immutable DOM analysis revision, independent of screenshot analyses and
/// decisions. Reuses a source-identical displayed revision without losing notes.
pub fn analyze_run(
    store: &mut Store,
    id: RunId,
    cancel: &AtomicBool,
) -> Result<DomComparisonEvidence> {
    analyze_run_with_options(store, id, DomDiffOptions::default(), cancel)
}

pub fn analyze_run_with_options(
    store: &mut Store,
    id: RunId,
    mut options: DomDiffOptions,
    cancel: &AtomicBool,
) -> Result<DomComparisonEvidence> {
    options.validate()?;
    check_cancel(cancel)?;
    let run = store.load_dom_run(id)?;
    ensure!(
        run.run.capture_plan.mode.dom_snapshots(),
        "This run has no requested DOM snapshots"
    );
    ensure!(
        run.run.state.terminal(),
        "Capture must finish before DOM analysis"
    );
    let baseline_id = match run.run.intent {
        RunIntent::Check { baseline } => baseline,
        _ => anyhow::bail!("A check against a baseline is required"),
    };
    store.invalidate_dom_approval(id)?;
    let baseline = store.load_dom_baseline(baseline_id)?;
    let mut source: Option<RunRecord> = None;
    let mut results = Vec::new();
    // Include explicitly omitted intended slots, but never visit/fill them.
    let keys = if baseline.intended_targets.is_empty() {
        run.run.targets.iter().map(|t| t.key.clone()).collect()
    } else {
        baseline.intended_targets.clone()
    };
    // Bound the whole persisted analysis, not merely one target's pretty diff.
    // Exact byte verdicts still cover every complete original. The effective
    // options are recorded and validated by storage using the same shared diff.
    const TOTAL_DIFF_BYTES: usize = 8 * 1024 * 1024;
    options.max_output_bytes = options
        .max_output_bytes
        .min(TOTAL_DIFF_BYTES / keys.len().max(1));
    options.max_output_lines = options.max_output_lines.min(options.max_output_bytes / 64);
    for key in keys {
        check_cancel(cancel)?;
        let reference = baseline.dom_references.iter().find(|r| r.key == key);
        let before = if let Some(reference) = reference {
            if source.as_ref().is_none_or(|s| s.run.id != reference.run) {
                source = Some(store.load_dom_run(reference.run)?);
            }
            let source = source.as_ref().unwrap();
            let capture = source
                .dom_evidence(&key)
                .context("DOM Before evidence is missing")?;
            ensure!(
                capture.metadata.sha256 == reference.artifact
                    && capture.metadata.policy == reference.policy,
                "DOM Before source identity mismatch"
            );
            Some(dom_source(&source.run, capture)?)
        } else {
            None
        };
        let after = run
            .dom_evidence(&key)
            .map(|c| dom_source(&run.run, c))
            .transpose()?;
        let result = if let (Some(before), Some(after)) = (&before, &after) {
            let (_, before_bytes) =
                store.read_verified_artifact(&before.artifact, MAX_DOM_BYTES as u64)?;
            let (_, after_bytes) =
                store.read_verified_artifact(&after.artifact, MAX_DOM_BYTES as u64)?;
            compare_dom_snapshots(
                before.clone(),
                &before_bytes,
                after.clone(),
                &after_bytes,
                &options,
                || cancel.load(Ordering::Acquire),
            )?
        } else {
            let outcome = if reference.is_none() {
                DomComparisonOutcome::NotComparable {
                    reason: DomNotComparableReason::BaselineOmission,
                }
            } else {
                match run
                    .run
                    .targets
                    .iter()
                    .find(|t| t.key == key)
                    .map(|t| &t.dom_outcome)
                {
                    Some(DomTargetOutcome::Failed { code }) => {
                        DomComparisonOutcome::CouldNotCapture { code: *code }
                    }
                    Some(DomTargetOutcome::Cancelled) => DomComparisonOutcome::NotComparable {
                        reason: DomNotComparableReason::Cancelled,
                    },
                    Some(DomTargetOutcome::Skipped { .. }) => DomComparisonOutcome::NotComparable {
                        reason: DomNotComparableReason::Skipped,
                    },
                    _ => DomComparisonOutcome::NotComparable {
                        reason: DomNotComparableReason::MissingAfter,
                    },
                }
            };
            DomAnalysisResult {
                key,
                before,
                after,
                outcome,
                diff: None,
            }
        };
        result.validate()?;
        results.push(result);
    }
    check_cancel(cancel)?;
    // Storage revalidates the exact sources and shared diff; cache identity is
    // source provenance plus full options/results, not equal content alone.
    if let Some(cached) = store.displayed_dom_analysis(id)?
        && cached.revision.baseline == baseline_id
        && cached.options == options
        && cached.revision.results == results
    {
        check_cancel(cancel)?;
        store.display_dom_analysis(cached.revision.id)?;
        return Ok(cached);
    }
    let evidence = DomComparisonEvidence {
        revision: DomAnalysisRevision {
            version: 1,
            id: DomAnalysisId::new(),
            baseline: baseline_id,
            run: id,
            algorithm: DOM_DIFF_ALGORITHM.into(),
            results,
        },
        created_at: regressee_runtime::distribution::now(),
        options,
    };
    check_cancel(cancel)?;
    store.save_dom_analysis(&evidence)?;
    Ok(evidence)
}
