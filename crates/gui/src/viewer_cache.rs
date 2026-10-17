// SPDX-License-Identifier: MPL-2.0
//! Display-only, revision-bound metadata and immutable verified decoded tiles.
//! Never used by approval, activation, export, deletion or other integrity gates.
use crate::jobs::{ViewerFrame, ViewerIdentity};
use anyhow::{Context, Result, ensure};
use regressee_core::{domain::*, evidence::*};
use regressee_storage::Store;
use std::{
    collections::{BTreeMap, VecDeque},
    sync::Arc,
};

const TILE_BYTES: usize = 64 * 1024 * 1024;
const SESSION_BYTES: usize = 128 * 1024 * 1024;
const SOURCE_BYTES: usize = 32 * 1024 * 1024;
const PLAN_BYTES: usize = 16 * 1024 * 1024;

/// The budget accounts for owned payloads, not externally held frame Arcs.
/// Count bounds also cap entry/key overhead. Oversized items are not retained.
struct ByteLru<K, V> {
    entries: VecDeque<(K, V, usize)>,
    bytes: usize,
    budget: usize,
    capacity: usize,
}
impl<K: PartialEq, V: Clone> ByteLru<K, V> {
    fn new(budget: usize, capacity: usize) -> Self {
        Self {
            entries: VecDeque::new(),
            bytes: 0,
            budget,
            capacity,
        }
    }
    fn get(&mut self, key: &K) -> Option<V> {
        let index = self.entries.iter().position(|(k, _, _)| k == key)?;
        let entry = self.entries.remove(index)?;
        let value = entry.1.clone();
        self.entries.push_back(entry);
        Some(value)
    }
    fn insert(&mut self, key: K, value: V, bytes: usize) {
        if let Some(index) = self.entries.iter().position(|(k, _, _)| k == &key) {
            self.bytes -= self.entries.remove(index).unwrap().2;
        }
        if bytes > self.budget || self.capacity == 0 {
            return;
        }
        while self.bytes + bytes > self.budget || self.entries.len() >= self.capacity {
            self.bytes -= self.entries.pop_front().unwrap().2;
        }
        self.bytes += bytes;
        self.entries.push_back((key, value, bytes));
    }
}

struct Session {
    project: ProjectId,
    analysis: Option<AnalysisId>,
    revision: (u64, u64),
    evidence: Arc<ProjectEvidence>,
    index: Arc<EvidenceIndex>,
}
fn key_index(keys: impl Iterator<Item = TargetKey>) -> BTreeMap<TargetKey, usize> {
    let mut index = BTreeMap::new();
    for (position, key) in keys.enumerate() {
        index.entry(key).or_insert(position);
    }
    index
}
fn index_bytes(count: usize) -> usize {
    // Conservative per-entry allowance for owned UUID keys and B-tree nodes.
    count * (std::mem::size_of::<TargetKey>() + std::mem::size_of::<usize>() + 96)
}
fn indexed_capture<'a>(
    run: &'a RunRecord,
    index: &BTreeMap<TargetKey, usize>,
    key: &TargetKey,
) -> Option<&'a CaptureEvidence> {
    let position = *index.get(key)?;
    run.captures.get(position).or_else(|| {
        (position == run.captures.len())
            .then_some(run.capture.as_ref())
            .flatten()
    })
}
#[derive(Default)]
struct EvidenceIndex {
    targets: BTreeMap<TargetKey, usize>,
    captures: BTreeMap<TargetKey, usize>,
    references: BTreeMap<TargetKey, usize>,
    heatmaps: BTreeMap<TargetKey, usize>,
    reports: BTreeMap<TargetKey, usize>,
}
impl EvidenceIndex {
    fn new(e: &ProjectEvidence) -> Self {
        Self {
            targets: key_index(
                e.latest
                    .iter()
                    .flat_map(|r| r.run.targets.iter().map(|t| t.key.clone())),
            ),
            captures: key_index(
                e.latest
                    .iter()
                    .flat_map(|r| r.all_captures().map(|c| c.metadata.key.clone())),
            ),
            references: key_index(
                e.baseline
                    .iter()
                    .flat_map(|b| b.references.iter().map(|r| r.key.clone())),
            ),
            heatmaps: key_index(
                e.comparison
                    .iter()
                    .flat_map(|a| a.heatmaps.iter().map(|(k, _)| k.clone())),
            ),
            reports: key_index(e.comparison.iter().flat_map(|a| {
                a.perceptual
                    .iter()
                    .flat_map(|p| p.reports.iter().map(|(k, _)| k.clone()))
            })),
        }
    }
    fn bytes(&self) -> usize {
        index_bytes(
            self.targets.len()
                + self.captures.len()
                + self.references.len()
                + self.heatmaps.len()
                + self.reports.len(),
        )
    }
}
struct Source {
    record: RunRecord,
    captures: BTreeMap<TargetKey, usize>,
}
#[derive(Clone)]
struct Plan {
    identity: ViewerIdentity,
    images: [Option<ImageEvidence>; 3],
    request: Option<regressee_app::analysis_engine::AnalysisRequest>,
    before_environment: Option<String>,
    before_capture: Option<CaptureEvidence>,
}
#[derive(Default, Debug, Clone, Copy)]
pub(crate) struct WorkCounts {
    pub snapshots: u64,
    pub sources: u64,
    pub plans: u64,
    pub decodes: u64,
    pub tile_hits: u64,
}
pub(crate) struct ViewerCache {
    session: Option<Session>,
    sources: ByteLru<RunId, Arc<Source>>,
    plans: ByteLru<TargetKey, Arc<Plan>>,
    tiles: ByteLru<ArtifactId, Arc<gpui::RenderImage>>,
    counts: WorkCounts,
    foreground: Option<ViewerIdentity>,
}
impl Default for ViewerCache {
    fn default() -> Self {
        Self {
            session: None,
            sources: ByteLru::new(SOURCE_BYTES, 8),
            plans: ByteLru::new(PLAN_BYTES, 32),
            tiles: ByteLru::new(TILE_BYTES, 256),
            counts: WorkCounts::default(),
            foreground: None,
        }
    }
}
impl ViewerCache {
    pub(crate) fn invalidate(&mut self) {
        let counts = self.counts;
        *self = Self::default();
        self.counts = counts;
    }
    fn evidence(
        &mut self,
        store: &Store,
        project: ProjectId,
        analysis: Option<AnalysisId>,
    ) -> Result<(Arc<ProjectEvidence>, Arc<EvidenceIndex>)> {
        let revision = store.viewer_revision()?;
        if let Some(session) = &self.session
            && session.project == project
            && session.analysis == analysis
            && session.revision == revision
        {
            return Ok((session.evidence.clone(), session.index.clone()));
        }
        self.invalidate();
        let evidence = store.viewer_evidence(project, analysis)?;
        self.counts.snapshots += 1;
        ensure!(
            store.viewer_revision()? == revision,
            "Evidence changed while opening viewer; try again"
        );
        let index = Arc::new(EvidenceIndex::new(&evidence));
        // Serialize only at the cold session boundary, never on warm tiles.
        ensure!(
            serde_json::to_vec(&evidence)?.len() + index.bytes() <= SESSION_BYTES,
            "viewer metadata exceeds budget"
        );
        let evidence = Arc::new(evidence);
        self.session = Some(Session {
            project,
            analysis,
            revision,
            evidence: evidence.clone(),
            index: index.clone(),
        });
        Ok((evidence, index))
    }
    fn source(&mut self, store: &Store, run: RunId) -> Result<Arc<Source>> {
        if let Some(source) = self.sources.get(&run) {
            return Ok(source);
        }
        let record = store.load_run(run)?;
        let captures = key_index(record.all_captures().map(|c| c.metadata.key.clone()));
        let bytes = serde_json::to_vec(&record)?.len() + index_bytes(captures.len());
        let source = Arc::new(Source { record, captures });
        self.counts.sources += 1;
        self.sources.insert(run, source.clone(), bytes);
        Ok(source)
    }
    fn plan(
        &mut self,
        store: &Store,
        project: ProjectId,
        key: TargetKey,
        e: &ProjectEvidence,
        index: &EvidenceIndex,
        inspection: bool,
    ) -> Result<Arc<Plan>> {
        if let Some(plan) = self.plans.get(&key) {
            return Ok(plan);
        }
        let run = e.latest.as_ref().context("No captured run selected")?;
        ensure!(
            index.targets.contains_key(&key),
            "viewer target does not belong to selected run"
        );
        let reference = e.baseline.as_ref().and_then(|b| {
            index
                .references
                .get(&key)
                .and_then(|i| b.references.get(*i))
        });
        let before_run = reference.map(|r| self.source(store, r.run)).transpose()?;
        let before = before_run
            .as_ref()
            .and_then(|r| indexed_capture(&r.record, &r.captures, &key));
        let after = indexed_capture(run, &index.captures, &key);
        let diff = e.comparison.as_ref().and_then(|a| {
            index
                .heatmaps
                .get(&key)
                .and_then(|i| a.heatmaps.get(*i))
                .map(|(_, h)| h)
                .or(a.heatmap.as_ref())
        });
        let request = if let (Some(old), Some(before), Some(after)) = (&before_run, before, after) {
            let report = e
                .comparison
                .as_ref()
                .and_then(|a| a.perceptual.as_ref())
                .and_then(|p| index.reports.get(&key).and_then(|i| p.reports.get(*i)));
            if let Some((_, report)) = report {
                Some(report.request.clone())
            } else if !inspection {
                Some(regressee_app::analysis_engine::AnalysisRequest {
                    before: descriptor(&old.record, before)?,
                    after: descriptor(run, after)?,
                    policy: e
                        .comparison
                        .as_ref()
                        .and_then(|a| a.perceptual.as_ref())
                        .map(|p| p.policy.clone())
                        .unwrap_or_else(|| {
                            regressee_app::review::preset(
                                regressee_app::analysis_engine::Preset::Balanced,
                            )
                        }),
                    mask: store.latest_mask(project, &key)?,
                })
            } else {
                // Legacy analysis without a saved request must not borrow today's
                // ignored areas or sensitivity. Screenshots remain inspectable.
                None
            }
        } else {
            None
        };
        let analysis = e.comparison.as_ref().map(|a| a.revision.id);
        let plan = Arc::new(Plan {
            identity: ViewerIdentity {
                project,
                run: run.run.id,
                key: key.clone(),
                analysis,
            },
            images: [
                before.map(|c| c.image.clone()),
                after.map(|c| c.image.clone()),
                diff.cloned(),
            ],
            request,
            before_environment: before_run
                .as_ref()
                .map(|r| r.record.run.environment.name.clone()),
            before_capture: before.cloned(),
        });
        let bytes = serde_json::to_vec(&plan.images)?.len()
            + serde_json::to_vec(&plan.request)?.len()
            + plan.before_environment.as_ref().map_or(0, String::len);
        let bytes = bytes + serde_json::to_vec(&plan.before_capture)?.len();
        self.plans.insert(key, plan.clone(), bytes);
        self.counts.plans += 1;
        Ok(plan)
    }
    pub(crate) fn frame(
        &mut self,
        store: &Store,
        project: ProjectId,
        tile: usize,
        key: Option<TargetKey>,
        analysis: Option<AnalysisId>,
    ) -> Result<ViewerFrame> {
        let (evidence, index) = self.evidence(store, project, analysis)?;
        self.frame_evidence(store, tile, key, &evidence, &index, false)
    }
    pub(crate) fn historical_frame(
        &mut self,
        store: &Store,
        project: ProjectId,
        run: RunId,
        tile: usize,
        key: Option<TargetKey>,
    ) -> Result<ViewerFrame> {
        self.invalidate();
        let revision = store.viewer_revision()?;
        let evidence = Arc::new(store.inspect_run(project, run)?);
        let index = Arc::new(EvidenceIndex::new(&evidence));
        self.session = Some(Session {
            project,
            analysis: evidence.comparison.as_ref().map(|a| a.revision.id),
            revision,
            evidence: evidence.clone(),
            index: index.clone(),
        });
        self.frame_evidence(store, tile, key, &evidence, &index, true)
    }
    fn frame_evidence(
        &mut self,
        store: &Store,
        tile: usize,
        key: Option<TargetKey>,
        evidence: &ProjectEvidence,
        index: &EvidenceIndex,
        inspection: bool,
    ) -> Result<ViewerFrame> {
        let project = evidence
            .latest
            .as_ref()
            .context("No captured run")?
            .run
            .project;
        let key = key
            .or_else(|| {
                evidence
                    .latest
                    .as_ref()
                    .and_then(|r| r.run.targets.first())
                    .map(|t| t.key.clone())
            })
            .context("No captured target selected")?;
        let plan = self.plan(store, project, key, evidence, index, inspection)?;
        let mut panes = Vec::with_capacity(3);
        let mut width = 0;
        let mut height = 0;
        for image in &plan.images {
            let mut images = Vec::new();
            if let Some(image) = image {
                width = width.max(image.width);
                height = height.max(image.height);
                for id in image.tiles.iter().skip(tile).take(3) {
                    let decoded = if let Some(decoded) = self.tiles.get(id) {
                        self.counts.tile_hits += 1;
                        decoded
                    } else {
                        let (decoded, bytes) =
                            crate::viewer_images::viewer_tile_with_size(store, id)?;
                        self.counts.decodes += 1;
                        self.tiles.insert(id.clone(), decoded.clone(), bytes);
                        decoded
                    };
                    images.push(decoded);
                }
            }
            panes.push(images);
        }
        ensure!(
            self.session
                .as_ref()
                .is_some_and(|s| store.viewer_revision().is_ok_and(|r| r == s.revision)),
            "Evidence changed while reading viewer; try again"
        );
        self.foreground = Some(plan.identity.clone());
        Ok(ViewerFrame {
            tile,
            panes,
            width,
            height,
            request: plan.request.clone(),
            analysis: plan.identity.analysis,
            identity: Some(plan.identity.clone()),
            before_environment: plan.before_environment.clone(),
            before_capture: plan.before_capture.clone(),
        })
    }
    pub(crate) fn prefetch(
        &mut self,
        store: &Store,
        project: ProjectId,
        tile: usize,
        key: Option<TargetKey>,
        analysis: Option<AnalysisId>,
    ) -> Result<ViewerFrame> {
        let foreground = self
            .foreground
            .clone()
            .context("No viewer selection to prefetch")?;
        let session = self
            .session
            .as_ref()
            .context("No viewer session to prefetch")?;
        ensure!(
            session.project == project
                && session.analysis == analysis
                && session.revision == store.viewer_revision()?
                && session
                    .evidence
                    .latest
                    .as_ref()
                    .is_some_and(|r| r.run.id == foreground.run)
                && session.evidence.comparison.as_ref().map(|a| a.revision.id)
                    == foreground.analysis,
            "Stale viewer prefetch"
        );
        // Adjacency belongs to the UI's sorted/filtered visible queue, not the
        // frozen capture order. The mailbox retains only one speculative read;
        // its target must remain in this exact foreground session/revision.
        let key = key.context("Prefetch needs the next visible target")?;
        let current = session
            .index
            .targets
            .get(&foreground.key)
            .context("No current target")?;
        let next = session
            .index
            .targets
            .get(&key)
            .context("No next visible target in this run")?;
        ensure!(
            current != next,
            "Prefetch must select another visible target"
        );
        let result = self.frame(store, project, tile, Some(key), analysis);
        self.foreground = self
            .session
            .as_ref()
            .filter(|s| {
                s.evidence
                    .latest
                    .as_ref()
                    .is_some_and(|r| r.run.id == foreground.run)
                    && s.evidence.comparison.as_ref().map(|a| a.revision.id) == foreground.analysis
            })
            .map(|_| foreground);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use regressee_storage::AppPaths;
    use std::sync::atomic::AtomicBool;

    struct Fixture {
        _root: tempfile::TempDir,
        paths: AppPaths,
        store: Store,
        project: ProjectId,
        before: RunRecord,
        after: RunRecord,
        analysis: AnalysisId,
    }
    fn fixture() -> Result<Fixture> {
        let root = tempfile::tempdir()?;
        let paths = AppPaths::isolated(root.path());
        let mut store = Store::open(paths.clone())?;
        let project = Project::draft("Synthetic viewer fixture".into(), "https://example.test/")?;
        store.save_project(&project, None)?;
        let mut profile = project.profiles[0].clone();
        profile.width = 8;
        profile.height = 512;
        profile.runtime = RuntimeIdentity::Pinned {
            node: "fixture".into(),
            worker: "fixture".into(),
            playwright: "fixture".into(),
            browser: "fixture".into(),
        };
        let pages: Vec<_> = (0..3)
            .map(|i| InventoryPage {
                id: PageId::new(),
                canonical: format!("https://example.test/{i}").parse().unwrap(),
                included: true,
                reason: None,
            })
            .collect();
        let inventory = InventoryRevision {
            id: InventoryId::new(),
            project: project.id,
            identity_version: 2,
            pages,
            coverage: DiscoveryCoverage::Complete,
        };
        store.freeze_inventory(&inventory)?;
        let mut environment = project.environments[0].clone();
        environment.name = "Production evidence".into();
        let template = CaptureRun {
            id: RunId::new(),
            project: project.id,
            intent: RunIntent::NewBaseline,
            inventory,
            profiles: vec![profile],
            environment,
            rendering: RenderingEnvironment {
                os: "fixture".into(),
                architecture: "fixture".into(),
                browser: "fixture".into(),
                worker_version: "fixture".into(),
                profile_version: 1,
                fonts_fingerprint: None,
            },
            state: JobState::Running,
            started_at: Some(1),
            finished_at: None,
            targets: vec![],
        };
        let before = complete_run(&mut store, root.path(), template, 0)?;
        let saved = store.activate_baseline(before.run.id, 1)?;
        let baseline = saved.project.active_baseline.context("fixture baseline")?;
        let mut template = before.run.clone();
        template.id = RunId::new();
        template.intent = RunIntent::Check { baseline };
        template.state = JobState::Running;
        template.started_at = Some(3);
        template.finished_at = None;
        template.environment.name = "Staging evidence".into();
        let after = complete_run(&mut store, root.path(), template, 1)?;
        let cancel = AtomicBool::new(false);
        let (_, heatmap) = regressee_app::images::compare(
            &mut store,
            &before.captures[0].image,
            &after.captures[0].image,
            after.run.targets[0].key.clone(),
            &cancel,
        )?;
        let analysis = ComparisonEvidence {
            revision: AnalysisRevision {
                id: AnalysisId::new(),
                baseline,
                run: after.run.id,
                algorithm: "rgba-max-channel-v1".into(),
                policy: AnalysisPolicy {
                    color_tolerance: 0.,
                    changed_percent: 0.,
                    region_pixels: 0,
                },
                mask: None,
                results: after
                    .run
                    .targets
                    .iter()
                    .map(|t| AnalysisResult {
                        key: t.key.clone(),
                        classification: Classification::Changed,
                        eligible_pixels: 8 * 2049,
                        changed_pixels: 8 * 2049,
                        dimensions_changed: false,
                    })
                    .collect(),
            },
            heatmap: None,
            heatmaps: after
                .run
                .targets
                .iter()
                .map(|t| (t.key.clone(), heatmap.clone()))
                .collect(),
            perceptual: None,
        };
        store.save_analysis(&analysis)?;
        Ok(Fixture {
            _root: root,
            paths,
            store,
            project: project.id,
            before,
            after,
            analysis: analysis.revision.id,
        })
    }
    #[test]
    fn historical_reopening_is_read_only_and_uses_original_context() -> Result<()> {
        let mut f = fixture()?;
        let old_baseline = match f.after.run.intent {
            RunIntent::Check { baseline } => baseline,
            _ => unreachable!(),
        };
        // Today's reference and settings are deliberately unrelated to this check.
        let mut replacement = f.before.run.clone();
        replacement.id = RunId::new();
        replacement.state = JobState::Running;
        replacement.finished_at = None;
        replacement.environment.name = "Today's reference".into();
        let replacement = complete_run(&mut f.store, f._root.path(), replacement, 0)?;
        let revision = f.store.load_project(f.project)?.revision;
        f.store.activate_baseline(replacement.run.id, revision)?;
        let saved = serde_json::to_vec(&f.store.load_project(f.project)?)?;
        let version = f.store.viewer_revision()?;
        let evidence = f.store.inspect_run(f.project, f.after.run.id)?;
        assert_eq!(evidence.baseline.as_ref().unwrap().id, old_baseline);
        assert_eq!(
            evidence.comparison.as_ref().unwrap().revision.id,
            f.analysis
        );
        let run = evidence.latest.as_ref().unwrap();
        assert_eq!(run.run.environment.name, "Staging evidence");
        assert_eq!(
            serde_json::to_vec(&run.run.profiles)?,
            serde_json::to_vec(&f.after.run.profiles)?
        );
        assert_eq!(
            run.captures[0].metadata.requested,
            f.after.captures[0].metadata.requested
        );
        let mut cache = ViewerCache::default();
        let frame = cache.historical_frame(&f.store, f.project, f.after.run.id, 0, None)?;
        assert_eq!(
            frame.before_environment.as_deref(),
            Some("Production evidence")
        );
        assert_eq!(frame.identity.as_ref().unwrap().run, f.after.run.id);
        assert!(
            frame.request.is_none(),
            "legacy saved results must not borrow today's policy/masks"
        );
        let capture = cache.historical_frame(&f.store, f.project, f.before.run.id, 0, None)?;
        assert!(capture.panes[0].is_empty() && !capture.panes[1].is_empty());
        assert!(capture.analysis.is_none() && capture.request.is_none());
        assert!(
            f.store
                .inspect_run(ProjectId::new(), f.after.run.id)
                .is_err()
        );
        assert!(f.store.inspect_run(f.project, RunId::new()).is_err());
        assert_eq!(
            f.store.viewer_revision()?,
            version,
            "inspection must not write saved state"
        );
        assert_eq!(
            serde_json::to_vec(&f.store.load_project(f.project)?)?,
            saved
        );
        let original = &f.before.captures[0].image.original;
        let path = f.paths.artifacts().join(original.as_str());
        let mut bytes = std::fs::read(&path)?;
        bytes[20] ^= 1;
        std::fs::write(path, bytes)?;
        assert!(
            f.store.inspect_run(f.project, f.after.run.id).is_err(),
            "damaged original reference must not become a clean/empty result"
        );
        assert!(
            cache
                .historical_frame(&f.store, f.project, f.after.run.id, 0, None)
                .is_err(),
            "historical reads reverify originals rather than trusting prior pixels"
        );
        assert_eq!(f.store.viewer_revision()?, version);
        Ok(())
    }

    #[test]
    fn historical_viewer_uses_saved_mask_and_policy_not_newer_drafts() -> Result<()> {
        let mut f = fixture()?;
        let cancel = AtomicBool::new(false);
        let analysis = regressee_app::review::analyze_run(
            &mut f.store,
            f.after.run.id,
            regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
            &cancel,
        )?;
        let mut request = analysis
            .perceptual
            .as_ref()
            .context("perceptual fixture")?
            .reports
            .first()
            .context("report fixture")?
            .1
            .request
            .clone();
        request.mask = Some(regressee_app::analysis_engine::MaskBinding {
            revision: "recorded-mask".into(),
            target: request.before.target.clone(),
            before_capture_id: request.before.capture_id.clone(),
            after_capture_id: request.after.capture_id.clone(),
            before_dimensions: request.before.dimensions,
            after_dimensions: request.after.dimensions,
            rectangles: vec![regressee_app::analysis_engine::Rectangle {
                x: 0,
                y: 0,
                width: 1,
                height: 1,
            }],
        });
        f.store.save_mask(f.project, &request)?;
        let recorded = regressee_app::review::analyze_run(
            &mut f.store,
            f.after.run.id,
            regressee_app::review::preset(regressee_app::analysis_engine::Preset::Sensitive),
            &cancel,
        )?;
        let saved_request = recorded
            .perceptual
            .as_ref()
            .unwrap()
            .reports
            .first()
            .unwrap()
            .1
            .request
            .clone();
        request.mask.as_mut().unwrap().revision = "newer-mask".into();
        request.mask.as_mut().unwrap().rectangles[0].width = 2;
        f.store.save_mask(f.project, &request)?;
        let version = f.store.viewer_revision()?;
        let key = f.after.run.targets[0].key.clone();
        let frame = ViewerCache::default().historical_frame(
            &f.store,
            f.project,
            f.after.run.id,
            0,
            Some(key),
        )?;
        assert_eq!(frame.request.as_ref(), Some(&saved_request));
        assert_eq!(
            frame.request.unwrap().mask.unwrap().revision,
            "recorded-mask"
        );
        assert_eq!(f.store.viewer_revision()?, version);
        Ok(())
    }

    #[test]
    fn historical_inspection_rejects_missing_and_corrupt_after_or_capture_originals() -> Result<()>
    {
        for check in [false, true] {
            for missing in [false, true] {
                let f = fixture()?;
                let run = if check { &f.after } else { &f.before };
                // Alter the captured original itself, not metadata or a derivative.
                let original = &run.captures.last().unwrap().image.original;
                let path = f.paths.artifacts().join(original.as_str());
                let version = f.store.viewer_revision()?;
                if missing {
                    std::fs::remove_file(&path)?;
                } else {
                    let mut bytes = std::fs::read(&path)?;
                    bytes[20] ^= 1;
                    std::fs::write(&path, bytes)?;
                }
                assert!(
                    f.store.inspect_run(f.project, run.run.id).is_err(),
                    "missing/corrupt captured originals must fail before returning evidence"
                );
                assert!(
                    ViewerCache::default()
                        .historical_frame(&f.store, f.project, run.run.id, 0, None)
                        .is_err(),
                    "metadata bindings alone cannot establish readable original evidence"
                );
                assert_eq!(
                    f.store.viewer_revision()?,
                    version,
                    "failed inspection must not write repair artifacts or saved database state"
                );
            }
        }
        Ok(())
    }

    fn complete_run(
        store: &mut Store,
        root: &std::path::Path,
        mut run: CaptureRun,
        variant: u8,
    ) -> Result<RunRecord> {
        let project_revision = store.load_project(run.project)?.revision;
        run.targets = run
            .inventory
            .pages
            .iter()
            .map(|p| CaptureTarget {
                key: TargetKey {
                    page: p.id,
                    profile: run.profiles[0].id,
                },
                requested: p.canonical.clone(),
                final_url: None,
                attempts: 0,
                outcome: TargetOutcome::Pending,
                warnings: vec![],
            })
            .collect();
        let mut record = RunRecord {
            run,
            capture: None,
            captures: vec![],
            queue: Some(QueueSettings {
                discovery: DiscoverySettings::default(),
                project_revision,
                resumed_at: vec![],
            }),
        };
        store.begin_run(&record)?;
        let path = root.join(format!("viewer-{variant}.png"));
        let mut pixels = vec![255; 8 * 2049 * 4];
        for y in 0..2049 {
            for x in 0..8 {
                pixels[(y * 8 + x) * 4] = (y / 512) as u8 * 30 + variant * 120;
            }
        }
        let mut encoder = png::Encoder::new(std::fs::File::create(&path)?, 8, 2049);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header()?.write_image_data(&pixels)?;
        let image = regressee_app::images::ingest(
            store,
            &path,
            ArtifactKind::Screenshot,
            &AtomicBool::new(false),
        )?;
        let artifact = store.verify_artifact(&image.original)?;
        for target in &mut record.run.targets {
            target.attempts = 1;
            target.final_url = Some(target.requested.clone());
            target.outcome = TargetOutcome::Captured {
                artifact: image.original.clone(),
                width: 8,
                height: 2049,
            };
            record.captures.push(CaptureEvidence {
                metadata: CaptureMetadata {
                    version: 1,
                    key: target.key.clone(),
                    requested: target.requested.to_string(),
                    final_url: target.requested.to_string(),
                    runtime: record.run.profiles[0].runtime.clone(),
                    profile: record.run.profiles[0].clone(),
                    width: 8,
                    height: 2049,
                    sha256: image.original.clone(),
                    bytes: artifact.bytes,
                    warnings: vec![],
                    stable: true,
                    duration_ms: 1,
                    readiness: Readiness {
                        font_failures: 0,
                        image_failures: 0,
                        resource_failures: 0,
                    },
                    capture_mode: "synthetic-viewer-fixture".into(),
                    replacement: None,
                },
                image: image.clone(),
            });
        }
        record.run.state = JobState::Completed;
        record.run.finished_at = Some(if variant == 0 { 2 } else { 4 });
        store.finish_run(&record)?;
        Ok(record)
    }
    #[test]
    fn byte_lru_evicts_least_recent_and_bounds_entries() {
        let mut cache = ByteLru::new(8, 2);
        cache.insert(1, Arc::new(1), 4);
        cache.insert(2, Arc::new(2), 4);
        assert_eq!(*cache.get(&1).unwrap(), 1);
        cache.insert(3, Arc::new(3), 4);
        assert!(cache.get(&2).is_none());
        assert_eq!(cache.bytes, 8);
        cache.insert(4, Arc::new(4), 9);
        assert!(cache.get(&4).is_none());
        cache.insert(5, Arc::new(5), 0);
        assert_eq!(cache.entries.len(), 2);
    }
    #[test]
    fn warm_viewer_reuses_metadata_and_decoded_tiles_with_exact_identity() -> Result<()> {
        let mut f = fixture()?;
        let mut cache = ViewerCache::default();
        let key = f.after.run.targets[0].key.clone();
        let cold = cache.frame(&f.store, f.project, 0, Some(key.clone()), Some(f.analysis))?;
        let counts = cache.counts;
        let warm = cache.frame(&f.store, f.project, 0, Some(key.clone()), Some(f.analysis))?;
        assert_eq!(
            (
                cache.counts.snapshots,
                cache.counts.sources,
                cache.counts.plans,
                cache.counts.decodes
            ),
            (
                counts.snapshots,
                counts.sources,
                counts.plans,
                counts.decodes
            )
        );
        assert!(cache.counts.tile_hits > counts.tile_hits);
        assert!(Arc::ptr_eq(&cold.panes[0][0], &warm.panes[0][0]));
        assert_eq!(
            warm.identity,
            Some(ViewerIdentity {
                project: f.project,
                run: f.after.run.id,
                key,
                analysis: Some(f.analysis)
            })
        );
        assert_eq!(
            warm.before_environment.as_deref(),
            Some("Production evidence")
        );
        assert_ne!(
            warm.before_environment.as_deref(),
            Some(f.after.run.environment.name.as_str())
        );
        let next = f.after.run.targets[1].key.clone();
        cache.prefetch(&f.store, f.project, 0, Some(next.clone()), Some(f.analysis))?;
        assert!(
            cache
                .prefetch(
                    &f.store,
                    f.project,
                    0,
                    Some(f.after.run.targets[0].key.clone()),
                    Some(f.analysis)
                )
                .is_err()
        );
        let next = cache.frame(&f.store, f.project, 0, Some(next.clone()), Some(f.analysis))?;
        assert_eq!(next.identity.unwrap().key, f.after.run.targets[1].key);
        assert!(
            cache
                .frame(&f.store, ProjectId::new(), 0, None, Some(f.analysis))
                .is_err()
        );
        let other_project = Project::draft("Other viewer project".into(), "https://other.test/")?;
        f.store.save_project(&other_project, None)?;
        let error = cache
            .frame(&f.store, other_project.id, 0, None, Some(f.analysis))
            .unwrap_err();
        assert!(error.to_string().contains("another project"));
        assert!(
            cache
                .frame(
                    &f.store,
                    f.project,
                    0,
                    Some(TargetKey {
                        page: PageId::new(),
                        profile: ProfileId::new()
                    }),
                    Some(f.analysis)
                )
                .is_err()
        );
        Ok(())
    }
    #[test]
    fn revision_changes_invalidate_metadata_for_local_and_other_windows() -> Result<()> {
        let mut f = fixture()?;
        let mut cache = ViewerCache::default();
        cache.frame(&f.store, f.project, 0, None, None)?;
        let mut analysis = f.store.load_analysis(f.analysis)?;
        analysis.revision.id = AnalysisId::new();
        f.store.save_analysis(&analysis)?;
        let updated = cache.frame(&f.store, f.project, 0, None, None)?;
        assert_eq!(
            updated.identity.unwrap().analysis,
            Some(analysis.revision.id)
        );
        assert_eq!(cache.counts.snapshots, 2);
        let mut other = Store::open(f.paths.clone())?;
        other.display_analysis(f.analysis)?;
        let updated = cache.frame(&f.store, f.project, 0, None, None)?;
        assert_eq!(updated.analysis, Some(f.analysis));
        assert_eq!(cache.counts.snapshots, 3);
        other.invalidate_approval(f.after.run.id)?;
        cache.frame(&f.store, f.project, 0, None, None)?;
        assert_eq!(cache.counts.snapshots, 4);
        cache.invalidate();
        cache.frame(&f.store, f.project, 0, None, None)?;
        assert_eq!(cache.counts.snapshots, 5);
        // An explicitly inspected retained analysis is not silently replaced by
        // the run's currently displayed revision.
        let retained = cache.frame(&f.store, f.project, 0, None, Some(analysis.revision.id))?;
        assert_eq!(
            retained.identity.unwrap().analysis,
            Some(analysis.revision.id)
        );
        assert_eq!(retained.analysis, Some(analysis.revision.id));
        Ok(())
    }
    #[test]
    fn successful_adjacent_prefetch_publishes_no_update() -> Result<()> {
        use crate::jobs::{Jobs, Reply, Request, ViewerWork, Work};
        let f = fixture()?;
        let (jobs, events) = Jobs::start_without_runtime(f.paths.clone());
        jobs.submit_viewer(Request {
            id: 1,
            work: Work::Extension(ViewerWork::Viewer {
                project: f.project,
                tile: 0,
                key: Some(f.after.run.targets[0].key.clone()),
                analysis: Some(f.analysis),
            }),
        })
        .unwrap();
        assert!(matches!(
            events
                .recv_timeout(std::time::Duration::from_secs(5))?
                .reply,
            Reply::Extension(_)
        ));
        jobs.prefetch_viewer(Work::Extension(ViewerWork::Viewer {
            project: f.project,
            tile: 0,
            key: Some(f.after.run.targets[1].key.clone()),
            analysis: Some(f.analysis),
        }))
        .unwrap();
        assert!(
            events
                .recv_timeout(std::time::Duration::from_millis(100))
                .is_err()
        );
        jobs.submit_viewer(Request {
            id: 2,
            work: Work::Extension(ViewerWork::Viewer {
                project: f.project,
                tile: 0,
                key: Some(f.after.run.targets[1].key.clone()),
                analysis: Some(f.analysis),
            }),
        })
        .unwrap();
        let update = events.recv_timeout(std::time::Duration::from_secs(5))?;
        assert_eq!(update.id, 2);
        match update.reply {
            Reply::Extension(frame) => {
                assert_eq!(frame.identity.unwrap().key, f.after.run.targets[1].key)
            }
            _ => panic!("adjacent target did not load"),
        }
        jobs.join_after_ui();
        Ok(())
    }
    #[test]
    fn uncached_reads_fail_closed_but_cached_pixels_are_immutable() -> Result<()> {
        let f = fixture()?;
        let mut cache = ViewerCache::default();
        let cold = cache.frame(&f.store, f.project, 0, None, None)?;
        let cached = &f.before.captures[0].image.tiles[0];
        let unread = &f.before.captures[0].image.tiles[4];
        for id in [cached, unread] {
            let path = f.paths.artifacts().join(id.as_str());
            let mut bytes = std::fs::read(&path)?;
            bytes[20] ^= 1;
            std::fs::write(path, bytes)?;
        }
        let warm = cache.frame(&f.store, f.project, 0, None, None)?;
        assert!(Arc::ptr_eq(&cold.panes[0][0], &warm.panes[0][0]));
        assert!(cache.frame(&f.store, f.project, 3, None, None).is_err());
        assert!(
            f.store.load_run(f.before.run.id).is_err(),
            "authoritative load is never cached"
        );
        cache.invalidate();
        assert!(
            cache.frame(&f.store, f.project, 0, None, None).is_err(),
            "corrupt first read must not populate cache"
        );
        assert!(cache.tiles.entries.is_empty());
        Ok(())
    }
    #[test]
    fn filtered_next_target_prefetch_warms_plan_without_changing_foreground() -> Result<()> {
        let mut f = fixture()?;
        let mut analysis = f.store.load_analysis(f.analysis)?;
        analysis.revision.id = AnalysisId::new();
        analysis.revision.results[1].classification = Classification::NoSignificantChange;
        analysis.revision.results[1].changed_pixels = 0;
        f.store.save_analysis(&analysis)?;
        let visible: Vec<_> = analysis
            .revision
            .results
            .iter()
            .enumerate()
            .filter(|(_, result)| result.classification == Classification::Changed)
            .map(|(index, _)| index)
            .collect();
        assert_eq!(visible, [0, 2]);
        let mut cache = ViewerCache::default();
        let selected = f.after.run.targets[visible[0]].key.clone();
        let next = f.after.run.targets[visible[1]].key.clone();
        cache.frame(
            &f.store,
            f.project,
            0,
            Some(selected.clone()),
            Some(analysis.revision.id),
        )?;
        let foreground = cache.foreground.clone();
        let prefetched = cache.prefetch(
            &f.store,
            f.project,
            0,
            Some(next.clone()),
            Some(analysis.revision.id),
        )?;
        assert_eq!(prefetched.identity.as_ref().unwrap().key, next);
        assert_eq!(cache.foreground, foreground);
        assert_eq!(cache.foreground.as_ref().unwrap().key, selected);
        let warm = cache.counts;
        cache.frame(
            &f.store,
            f.project,
            0,
            Some(next),
            Some(analysis.revision.id),
        )?;
        assert_eq!(cache.counts.plans, warm.plans);
        assert_eq!(cache.counts.sources, warm.sources);
        assert_eq!(cache.counts.decodes, warm.decodes);
        Ok(())
    }
    #[test]
    #[ignore = "serial synthetic viewer microprofile; no native latency acceptance"]
    fn viewer_cold_warm_microprofile() -> Result<()> {
        let f = fixture()?;
        let start = std::time::Instant::now();
        for _ in 0..20 {
            // Reproduce the old per-tile metadata path using the unchanged
            // authoritative APIs; decoder is the current exact-bytes decoder.
            let e = f.store.project_evidence(f.project)?;
            let analysis = e.comparison.as_ref().context("fixture analysis")?;
            let baseline = f.store.load_baseline(analysis.revision.baseline)?;
            let before = f.store.load_run(baseline.references[0].run)?;
            let key = &f.after.run.targets[0].key;
            let images = [
                before.evidence(key).map(|c| &c.image),
                e.latest
                    .as_ref()
                    .and_then(|r| r.evidence(key))
                    .map(|c| &c.image),
                analysis
                    .heatmaps
                    .iter()
                    .find(|(k, _)| k == key)
                    .map(|(_, image)| image),
            ];
            for image in images.into_iter().flatten() {
                for id in image.tiles.iter().take(3) {
                    crate::viewer_images::viewer_tile(&f.store, id)?;
                }
            }
        }
        let cold = start.elapsed();
        let mut cache = ViewerCache::default();
        cache.frame(&f.store, f.project, 0, None, None)?;
        let start = std::time::Instant::now();
        for _ in 0..20 {
            cache.frame(&f.store, f.project, 0, None, None)?;
        }
        println!(
            "synthetic old per-tile metadata path vs warm, 20 frames: before={cold:?}, after={:?}, after source work={:?}; decoder is current, not native latency",
            start.elapsed(),
            cache.counts
        );
        assert_eq!(
            (
                cache.counts.snapshots,
                cache.counts.sources,
                cache.counts.plans
            ),
            (1, 1, 1)
        );
        Ok(())
    }
}
