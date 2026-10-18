// SPDX-License-Identifier: MPL-2.0
//! Recover pre-DOM selective baselines without rewriting their historical JSON.
//! A matching inventory is not authority: only verified parent links can recover
//! the original capture after every current reference came from a partial check.
use super::*;
use regressee_core::evidence::RunRecord;
use std::collections::{BTreeMap, BTreeSet};

impl Store {
    pub(crate) fn recover_legacy_baseline(
        &self,
        baseline: &BaselineVersion,
    ) -> Result<BaselineVersion> {
        ensure!(
            baseline.capture_plan.is_default() && baseline.dom_references.is_empty(),
            "DOM baseline is missing its frozen configuration"
        );
        if let Some(source) = baseline.source_run {
            let source = self.load_run_snapshot(source)?;
            let mut recovered = baseline.clone();
            recovered.freeze_source(&source.run)?;
            self.verified_baseline_inventory(&recovered, &source.run)?;
            return Ok(recovered);
        }
        let mut cursor = baseline.clone();
        let mut visited = BTreeSet::new();
        let mut ancestors = Vec::new();
        loop {
            ensure!(
                visited.insert(cursor.id) && visited.len() <= MAX_TARGETS,
                "Invalid legacy baseline ancestry"
            );
            ensure!(
                cursor.project == baseline.project
                    && cursor.inventory == baseline.inventory
                    && cursor.intended_count == baseline.intended_count
                    && cursor.capture_plan == baseline.capture_plan
                    && cursor.saved_units() == baseline.saved_units()
                    && cursor.explicitly_partial == baseline.explicitly_partial,
                "Legacy baseline ancestor changes intended or saved coverage"
            );
            let mut sources = BTreeMap::<RunId, RunRecord>::new();
            for reference in &cursor.references {
                if let std::collections::btree_map::Entry::Vacant(entry) =
                    sources.entry(reference.run)
                {
                    entry.insert(self.load_run_snapshot(reference.run)?);
                }
                verify_reference(&cursor, reference, &sources[&reference.run])?;
            }
            let owners = cursor
                .references
                .iter()
                .map(|r| r.run)
                .collect::<BTreeSet<_>>();
            let candidate = cursor.source_run.or_else(|| {
                if owners.len() == 1 {
                    owners.iter().next().copied()
                } else {
                    None
                }
            });
            if let Some(candidate) = candidate {
                if let std::collections::btree_map::Entry::Vacant(entry) = sources.entry(candidate)
                {
                    entry.insert(self.load_run_snapshot(candidate)?);
                }
                let source = &sources[&candidate];
                let original = source.run.intent == RunIntent::NewBaseline
                    && matches!(
                        source.run.state,
                        JobState::Completed | JobState::CompletedWithErrors
                    )
                    && source.run.project == baseline.project
                    && source.run.inventory.id == baseline.inventory
                    && source.run.capture_plan == baseline.capture_plan
                    && source.run.targets.len() == baseline.intended_count as usize;
                let complete_reference_origin =
                    cursor.references.iter().all(|r| r.run == candidate)
                        && source
                            .all_captures()
                            .map(|c| c.metadata.key.clone())
                            .collect::<BTreeSet<_>>()
                            == cursor.target_keys();
                if original && complete_reference_origin {
                    ensure!(
                        cursor
                            .parent
                            .is_none_or(|parent| !visited.contains(&parent)),
                        "Cyclic legacy baseline terminal parent"
                    );
                    let mut recovered = baseline.clone();
                    // The ancestor chain establishes source authority; the shared
                    // helper freezes the exact original intended target identities.
                    if ancestors.is_empty() {
                        recovered.freeze_source(&source.run)?;
                    } else {
                        recovered.recover_legacy_source(&ancestors, &source.run, |id| {
                            Ok(self.load_run_snapshot(id)?.run)
                        })?;
                    }
                    self.verified_baseline_inventory(&recovered, &source.run)?;
                    return Ok(recovered);
                }
            }
            let parent = self.read_baseline_metadata(cursor.parent.context(
                "Legacy original capture cannot be recovered from retained ancestors",
            )?)?;
            ensure!(
                parent.project == cursor.project
                    && parent.inventory == cursor.inventory
                    && parent.capture_plan == cursor.capture_plan
                    && parent.intended_count == cursor.intended_count
                    && parent.saved_units() == cursor.saved_units(),
                "Invalid legacy baseline parent coverage"
            );
            ancestors.push(parent.clone());
            cursor = parent;
        }
    }
}
fn verify_reference(
    baseline: &BaselineVersion,
    reference: &BaselineReference,
    source: &RunRecord,
) -> Result<()> {
    ensure!(
        source.run.project == baseline.project
            && source.run.inventory.id == baseline.inventory
            && source.run.capture_plan == baseline.capture_plan
            && source.run.finished_at == Some(reference.captured_at)
            && source
                .evidence(&reference.key)
                .is_some_and(|capture| capture.image.original == reference.artifact),
        "Legacy reference provenance mismatch"
    );
    Ok(())
}
