// SPDX-License-Identifier: MPL-2.0
//! Verified historical inspection. No display pointer, settings or evidence writes.
use super::*;
use regressee_core::evidence::ProjectEvidence;
use std::collections::{BTreeMap, btree_map::Entry};

/// Per-read summaries of authoritative run loads, never a persistent integrity cache.
struct ReferenceSource {
    project: ProjectId,
    inventory: InventoryId,
    captured_at: Option<i64>,
    originals: BTreeMap<TargetKey, ArtifactId>,
}

fn verify_reference_sources(
    baseline: &BaselineVersion,
    project: ProjectId,
    mut load: impl FnMut(RunId) -> Result<ReferenceSource>,
) -> Result<usize> {
    let mut sources = BTreeMap::new();
    for reference in &baseline.references {
        if let Entry::Vacant(entry) = sources.entry(reference.run) {
            entry.insert(load(reference.run)?);
        }
        let source = &sources[&reference.run];
        ensure!(
            source.project == project,
            "Reference source belongs to another project"
        );
        ensure!(
            source.inventory == baseline.inventory
                && source.captured_at == Some(reference.captured_at),
            "Original reference provenance does not match"
        );
        let original = source
            .originals
            .get(&reference.key)
            .context("Original reference screenshot is missing")?;
        ensure!(
            *original == reference.artifact,
            "Original reference artifact does not match"
        );
    }
    Ok(sources.len())
}

impl Store {
    pub fn inspect_run(&self, project: ProjectId, run: RunId) -> Result<ProjectEvidence> {
        self.load_project(project)?;
        let record = self.load_run(run)?;
        ensure!(
            record.run.project == project,
            "Recorded run belongs to another project"
        );
        let baseline = match record.run.intent {
            RunIntent::Check { baseline } => {
                let baseline = self.load_baseline(baseline)
                    .context("The check's original baseline is missing or damaged; results cannot be reopened")?;
                ensure!(
                    baseline.project == project,
                    "Recorded baseline belongs to another project"
                );
                Some(baseline)
            }
            _ => None,
        };
        let comparison = if let Some(baseline) = &baseline {
            let id: Option<String> = self.connection.query_row(
                "SELECT COALESCE((SELECT analysis_id FROM analysis_display WHERE run_id=?1),(SELECT id FROM analyses WHERE run_id=?1 ORDER BY rowid DESC LIMIT 1))",
                [run.to_string()], |row| row.get(0),
            )?;
            let id = id
                .context("This check has no saved analysis. No unchanged verdict can be claimed")?;
            let analysis = self
                .load_analysis(serde_json::from_value(serde_json::json!(id))?)
                .context("Saved check analysis is missing or damaged")?;
            ensure!(
                analysis.revision.run == run && analysis.revision.baseline == baseline.id,
                "Saved analysis does not match the recorded check and original baseline"
            );
            Some(analysis)
        } else {
            None
        };
        // Each distinct source is loaded/verified only once in this read. Indexed
        // originals also avoid rescanning a large run for every reference.
        // Never replace a missing original with today's reference or viewer pixels.
        if let Some(baseline) = &baseline {
            verify_reference_sources(baseline, project, |run| {
                // load_run verifies every original, thumbnail and tile against CAS.
                // Summaries exist only after that authoritative validation succeeds.
                let source = self.load_run(run)?;
                Ok(ReferenceSource {
                    project: source.run.project,
                    inventory: source.run.inventory.id,
                    captured_at: source.run.finished_at,
                    originals: source
                        .all_captures()
                        .map(|capture| {
                            (capture.metadata.key.clone(), capture.image.original.clone())
                        })
                        .collect(),
                })
            })?;
        }
        Ok(ProjectEvidence {
            latest: Some(record),
            baseline,
            before: None,
            analyses: comparison.iter().map(|a| a.revision.id).collect(),
            reviews: self.review_history(run)?,
            comparison,
            baselines: vec![],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn repeated_references_load_only_distinct_sources_and_check_each_binding() -> Result<()> {
        let project = ProjectId::new();
        let inventory = InventoryId::new();
        let runs = [RunId::new(), RunId::new(), RunId::new()];
        let artifact: ArtifactId = "a".repeat(64).try_into()?;
        let references: Vec<_> = (0..3000)
            .map(|index| BaselineReference {
                key: TargetKey {
                    page: PageId::new(),
                    profile: ProfileId::new(),
                },
                run: runs[index % runs.len()],
                artifact: artifact.clone(),
                captured_at: 42,
            })
            .collect();
        let mut baseline = BaselineVersion {
            id: BaselineId::new(),
            project,
            parent: None,
            inventory,
            created_at: 43,
            intended_count: references.len() as u32,
            explicitly_partial: false,
            references,
        };
        let originals: BTreeMap<_, _> = baseline
            .references
            .iter()
            .map(|r| (r.key.clone(), r.artifact.clone()))
            .collect();
        let mut loads = BTreeMap::<RunId, usize>::new();
        let distinct = verify_reference_sources(&baseline, project, |run| {
            *loads.entry(run).or_default() += 1;
            Ok(ReferenceSource {
                project,
                inventory,
                captured_at: Some(42),
                originals: originals.clone(),
            })
        })?;
        assert_eq!(distinct, runs.len());
        assert!(runs.iter().all(|run| loads.get(run) == Some(&1)));
        // A late reference still receives artifact, target and provenance checks.
        let last = baseline.references.len() - 1;
        baseline.references[last].artifact = "b".repeat(64).try_into()?;
        assert!(
            verify_reference_sources(&baseline, project, |_| Ok(ReferenceSource {
                project,
                inventory,
                captured_at: Some(42),
                originals: originals.clone(),
            }))
            .is_err()
        );
        baseline.references[last].artifact = artifact;
        baseline.references[last].key.page = PageId::new();
        assert!(
            verify_reference_sources(&baseline, project, |_| Ok(ReferenceSource {
                project,
                inventory,
                captured_at: Some(42),
                originals: originals.clone(),
            }))
            .is_err()
        );
        baseline.references[last].key = originals.keys().next().unwrap().clone();
        baseline.references[last].captured_at = 41;
        assert!(
            verify_reference_sources(&baseline, project, |_| Ok(ReferenceSource {
                project,
                inventory,
                captured_at: Some(42),
                originals: originals.clone(),
            }))
            .is_err()
        );
        Ok(())
    }
}
