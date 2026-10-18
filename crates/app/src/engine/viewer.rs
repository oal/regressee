// SPDX-License-Identifier: MPL-2.0
//! Immutable verified artifact descriptions, not native images or database handles.
use crate::{analysis_engine, protocol::*};
use anyhow::{Context, Result, ensure};
use regressee_core::{domain::*, evidence::*};
use regressee_storage::Store;
use std::{collections::VecDeque, sync::Arc};

struct Session {
    selection: (ProjectId, Option<RunId>, Option<AnalysisId>),
    revision: (u64, u64),
    evidence: Arc<ProjectEvidence>,
}
#[derive(Default)]
pub(super) struct ViewerCache {
    session: Option<Session>,
    sources: VecDeque<(RunId, Arc<RunRecord>, usize)>,
    plans: VecDeque<(TargetKey, Arc<Plan>, usize)>,
}
struct Plan {
    images: [Option<ImageEvidence>; 3],
    request: Option<analysis_engine::AnalysisRequest>,
    identity: ViewerIdentity,
    before_environment: Option<String>,
    before_capture: Option<CaptureEvidence>,
}
impl ViewerCache {
    fn evidence(
        &mut self,
        store: &Store,
        selection: &ViewerSelection,
    ) -> Result<Arc<ProjectEvidence>> {
        let revision = store.viewer_revision()?;
        let identity = (selection.project, selection.run, selection.analysis);
        if let Some(session) = &self.session
            && session.selection == identity
            && session.revision == revision
        {
            return Ok(session.evidence.clone());
        }
        *self = Self::default();
        let evidence = if let Some(analysis) = selection.analysis {
            let evidence = store.viewer_evidence(selection.project, Some(analysis))?;
            ensure!(
                selection
                    .run
                    .is_none_or(|run| evidence.latest.as_ref().is_some_and(|r| r.run.id == run)),
                "invalid_request: exact viewer analysis/run mismatch"
            );
            evidence
        } else if let Some(run) = selection.run {
            store.screenshot_evidence_for_run(selection.project, run)?
        } else {
            store.viewer_evidence(selection.project, selection.analysis)?
        };
        ensure!(
            serde_json::to_vec(&evidence)?.len() <= 128 * 1024 * 1024,
            "Viewer metadata exceeds its bounded session budget"
        );
        ensure!(
            store.viewer_revision()? == revision,
            "Evidence changed during viewer read"
        );
        let evidence = Arc::new(evidence);
        self.session = Some(Session {
            selection: identity,
            revision,
            evidence: evidence.clone(),
        });
        Ok(evidence)
    }
    fn source(&mut self, store: &Store, id: RunId) -> Result<Arc<RunRecord>> {
        if let Some(index) = self.sources.iter().position(|(run, _, _)| *run == id) {
            let entry = self.sources.remove(index).unwrap();
            let record = entry.1.clone();
            self.sources.push_back(entry);
            return Ok(record);
        }
        let record = Arc::new(store.load_screenshot_run(id)?);
        let bytes = serde_json::to_vec(&*record)?.len();
        if bytes <= 32 * 1024 * 1024 {
            while !self.sources.is_empty()
                && (self.sources.len() >= 8
                    || self.sources.iter().map(|e| e.2).sum::<usize>() + bytes > 32 * 1024 * 1024)
            {
                self.sources.pop_front();
            }
            self.sources.push_back((id, record.clone(), bytes));
        }
        Ok(record)
    }

    pub fn frame(&mut self, store: &Store, selection: ViewerSelection) -> Result<ViewerFrame> {
        ensure!(
            selection.tile <= 64,
            "Tile offset exceeds screenshot bounds"
        );
        let checkpoint = store.viewer_revision()?;
        let evidence = self.evidence(store, &selection)?;
        let run = evidence.latest.as_ref().context("No captured run")?;
        ensure!(
            run.run.capture_plan.mode.screenshots(),
            "invalid_request: this run requested DOM snapshots only; use the focused DOM source/diff viewer, not image panes"
        );
        let key = selection
            .key
            .or_else(|| run.run.targets.first().map(|t| t.key.clone()))
            .context("No target selected")?;
        let plan = if let Some(index) = self.plans.iter().position(|(k, _, _)| *k == key) {
            let entry = self.plans.remove(index).unwrap();
            let plan = entry.1.clone();
            self.plans.push_back(entry);
            plan
        } else {
            ensure!(
                run.run.targets.iter().any(|t| t.key == key),
                "Target not in selected run"
            );
            let reference = evidence
                .baseline
                .as_ref()
                .and_then(|b| b.references.iter().find(|r| r.key == key));
            let before_run = reference.map(|r| self.source(store, r.run)).transpose()?;
            let before = before_run
                .as_ref()
                .and_then(|r| r.all_captures().find(|c| c.metadata.key == key));
            let after = run.all_captures().find(|c| c.metadata.key == key);
            let diff = evidence.comparison.as_ref().and_then(|a| {
                a.heatmaps
                    .iter()
                    .find(|(k, _)| *k == key)
                    .map(|(_, h)| h)
                    .or(a.heatmap.as_ref())
            });
            let request =
                if let (Some(old), Some(before), Some(after)) = (&before_run, before, after) {
                    let recorded = evidence
                        .comparison
                        .as_ref()
                        .and_then(|a| a.perceptual.as_ref())
                        .and_then(|p| p.reports.iter().find(|(k, _)| *k == key));
                    if let Some((_, r)) = recorded {
                        Some(r.request.clone())
                    } else if selection.run.is_none() {
                        Some(analysis_engine::AnalysisRequest {
                            before: descriptor(old, before)?,
                            after: descriptor(run, after)?,
                            policy: evidence
                                .comparison
                                .as_ref()
                                .and_then(|a| a.perceptual.as_ref())
                                .map(|p| p.policy.clone())
                                .unwrap_or_else(|| {
                                    crate::review::preset(analysis_engine::Preset::Balanced)
                                }),
                            mask: store.latest_mask(selection.project, &key)?,
                        })
                    } else {
                        None
                    }
                } else {
                    None
                };
            let analysis = evidence.comparison.as_ref().map(|a| a.revision.id);
            let plan = Arc::new(Plan {
                images: [
                    before.map(|c| c.image.clone()),
                    after.map(|c| c.image.clone()),
                    diff.cloned(),
                ],
                request,
                identity: ViewerIdentity {
                    project: selection.project,
                    run: run.run.id,
                    key: key.clone(),
                    analysis,
                },
                before_environment: before_run.as_ref().map(|r| r.run.environment.name.clone()),
                before_capture: before.cloned(),
            });
            let bytes =
                serde_json::to_vec(&(&plan.images, &plan.request, &plan.before_capture))?.len();
            if bytes <= 16 * 1024 * 1024 {
                while !self.plans.is_empty()
                    && (self.plans.len() >= 32
                        || self.plans.iter().map(|e| e.2).sum::<usize>() + bytes > 16 * 1024 * 1024)
                {
                    self.plans.pop_front();
                }
                self.plans.push_back((key, plan.clone(), bytes));
            }
            plan
        };
        let mut panes = Vec::new();
        let mut width = 0;
        let mut height = 0;
        for image in &plan.images {
            let mut tiles = Vec::new();
            if let Some(image) = image {
                width = width.max(image.width);
                height = height.max(image.height);
                for id in image.tiles.iter().skip(selection.tile).take(3) {
                    let (artifact, _) = store.read_verified_artifact(id, 16 * 1024 * 1024)?;
                    ensure!(
                        artifact.kind == ArtifactKind::Tile,
                        "Viewer requires tile derivatives"
                    );
                    tiles.push(ArtifactFile {
                        path: store.paths().artifacts().join(id.as_str()),
                        artifact,
                    });
                }
            }
            panes.push(tiles);
        }
        ensure!(
            store.viewer_revision()? == checkpoint,
            "Evidence changed during viewer read"
        );
        Ok(ViewerFrame {
            tile: selection.tile,
            panes,
            width,
            height,
            request: plan.request.clone(),
            analysis: plan.identity.analysis,
            identity: plan.identity.clone(),
            before_environment: plan.before_environment.clone(),
            before_capture: plan.before_capture.clone(),
        })
    }
}
