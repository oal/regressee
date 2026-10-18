// SPDX-License-Identifier: MPL-2.0
//! Phase 2 persisted evidence, not worker-owned filesystem paths.
use crate::dom_snapshot::*;
use crate::domain::*;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureMetadata {
    pub version: u32,
    pub key: TargetKey,
    pub requested: String,
    pub final_url: String,
    pub runtime: RuntimeIdentity,
    pub profile: CaptureProfile,
    pub width: u32,
    pub height: u32,
    pub sha256: ArtifactId,
    pub bytes: u64,
    pub warnings: Vec<String>,
    pub stable: bool,
    pub duration_ms: u64,
    pub readiness: Readiness,
    pub capture_mode: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub replacement: Option<serde_json::Value>,
}

/// Website loading cautions describe the capture, not whether two PNGs can be
/// compared. Unknown/replacement warnings are not silently admitted to a baseline.
pub fn loading_warning(warning: &str) -> bool {
    matches!(
        warning,
        "ImagesIncomplete"
            | "FontsIncomplete"
            | "ResourceFailed"
            | "EmbeddedContentBlocked"
            | "ReadinessBudget"
    )
}

impl CaptureMetadata {
    pub fn baseline_ready(&self) -> bool {
        self.stable && self.warnings.iter().all(|w| loading_warning(w))
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Readiness {
    pub font_failures: u32,
    pub image_failures: i32,
    pub resource_failures: u32,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ImageEvidence {
    pub original: ArtifactId,
    pub thumbnail: ArtifactId,
    pub width: u32,
    pub height: u32,
    pub tiles: Vec<ArtifactId>,
}
impl ImageEvidence {
    pub fn artifacts(&self) -> Vec<ArtifactId> {
        let mut a = vec![self.original.clone(), self.thumbnail.clone()];
        a.extend(self.tiles.clone());
        a
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureEvidence {
    pub metadata: CaptureMetadata,
    pub image: ImageEvidence,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRecord {
    pub run: CaptureRun,
    pub capture: Option<CaptureEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub captures: Vec<CaptureEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dom_captures: Vec<DomCaptureEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub queue: Option<QueueSettings>,
}
impl RunRecord {
    /// Finished baseline captures may be saved with explicit consent to omit
    /// unsuccessful targets. Active, interrupted and preview runs never qualify.
    pub fn baseline_save_counts(&self) -> Option<(usize, usize)> {
        if self.run.intent != RunIntent::NewBaseline
            || !matches!(
                self.run.state,
                JobState::Completed | JobState::CompletedWithErrors
            )
            || self.run.finished_at.is_none()
        {
            return None;
        }
        let captured = self
            .all_captures()
            .map(|c| &c.metadata.key)
            .chain(self.dom_captures.iter().map(|c| &c.metadata.key))
            .collect::<std::collections::BTreeSet<_>>()
            .len();
        (captured > 0 && captured <= self.run.targets.len())
            .then_some((captured, self.run.targets.len()))
    }

    pub fn evidence(&self, key: &TargetKey) -> Option<&CaptureEvidence> {
        self.captures
            .iter()
            .chain(self.capture.iter())
            .find(|c| &c.metadata.key == key)
    }
    pub fn all_captures(&self) -> impl Iterator<Item = &CaptureEvidence> {
        self.captures.iter().chain(self.capture.iter())
    }
    pub fn dom_evidence(&self, key: &TargetKey) -> Option<&DomCaptureEvidence> {
        self.dom_captures.iter().find(|c| &c.metadata.key == key)
    }
    /// Metadata/outcome validation only. Storage must additionally call
    /// `verify_snapshot` on each complete artifact before publishing/using it.
    pub fn validate_dom_evidence(&self) -> Result<()> {
        self.run.validate()?;
        let mut keys = std::collections::BTreeSet::new();
        for capture in &self.dom_captures {
            ensure!(
                keys.insert(&capture.metadata.key),
                "duplicate DOM capture evidence"
            );
            capture.metadata.validate_for_run(&self.run)?;
        }
        for target in &self.run.targets {
            if matches!(target.dom_outcome, DomTargetOutcome::Captured { .. }) {
                ensure!(
                    keys.contains(&target.key),
                    "missing successful DOM capture evidence"
                );
            }
        }
        Ok(())
    }
    pub fn baseline_save_coverage(&self) -> Option<BaselineSaveCoverage> {
        let (captured_targets, intended_targets) = self.baseline_save_counts()?;
        Some(BaselineSaveCoverage {
            captured_targets,
            intended_targets,
            screenshot_captures: self
                .all_captures()
                .map(|c| &c.metadata.key)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            dom_captures: self
                .dom_captures
                .iter()
                .map(|c| &c.metadata.key)
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            screenshot_intended: if self.run.capture_plan.mode.screenshots() {
                intended_targets
            } else {
                0
            },
            dom_intended: if self.run.capture_plan.mode.dom_snapshots() {
                intended_targets
            } else {
                0
            },
        })
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BaselineSaveCoverage {
    pub captured_targets: usize,
    pub intended_targets: usize,
    pub screenshot_captures: usize,
    pub dom_captures: usize,
    pub screenshot_intended: usize,
    pub dom_intended: usize,
}

/// Independent evidence: no image, dimensions, tiles, thumbnail or pixel metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomCaptureEvidence {
    pub metadata: DomCaptureMetadata,
}
impl DomCaptureEvidence {
    pub fn artifacts(&self) -> Vec<ArtifactId> {
        vec![self.metadata.sha256.clone()]
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomCaptureMetadata {
    pub version: u32,
    pub run: RunId,
    pub key: TargetKey,
    pub requested: String,
    pub final_url: String,
    pub runtime: RuntimeIdentity,
    pub profile: CaptureProfile,
    pub policy: DomSnapshotPolicy,
    pub sha256: ArtifactId,
    pub bytes: u64,
    pub node_count: u32,
    /// Includes excluded nodes/comments and explicit shadow/template boundaries.
    pub visited_nodes: u32,
    pub max_depth: u16,
    pub coverage: DomCoverage,
    pub warnings: Vec<String>,
    pub stable: bool,
    pub duration_ms: u64,
    pub readiness: Readiness,
}
impl DomCaptureMetadata {
    pub fn validate(&self) -> Result<()> {
        self.policy.validate()?;
        self.profile.validate()?;
        ensure!(
            self.version == 1
                && !self.run.0.is_nil()
                && !self.key.page.0.is_nil()
                && self.key.profile == self.profile.id,
            "invalid DOM source identity"
        );
        ensure!(
            self.profile.ready_for_capture() && self.runtime == self.profile.runtime,
            "DOM runtime/profile mismatch"
        );
        for value in [&self.requested, &self.final_url] {
            let url =
                url::Url::parse(value).map_err(|_| anyhow::anyhow!("invalid DOM source URL"))?;
            validate_address(&url)?;
            ensure!(url.as_str() == value, "noncanonical DOM source URL");
        }
        ensure!(
            self.bytes > DOM_CANONICAL_HEADER.len() as u64
                && self.bytes <= MAX_DOM_BYTES as u64
                && self.node_count > 0
                && self.node_count <= self.visited_nodes
                && self.visited_nodes <= MAX_DOM_NODES as u32
                && self.max_depth <= MAX_DOM_DEPTH,
            "DOM evidence exceeds limits"
        );
        ensure!(
            self.coverage.iframe_hosts <= self.node_count
                && self.coverage.open_shadow_roots <= self.node_count
                && self.coverage.template_contents <= self.node_count,
            "invalid DOM coverage counts"
        );
        ensure!(
            self.warnings.len() <= 64
                && self.warnings.iter().all(|w| w.len() <= 1024)
                && self.duration_ms <= 600_000,
            "DOM diagnostics exceed limits"
        );
        Ok(())
    }
    pub fn baseline_ready(&self) -> bool {
        self.validate().is_ok()
            && self.stable
            && self.warnings.iter().all(|w| {
                loading_warning(w)
                    || matches!(
                        w.as_str(),
                        "IframeDocumentsExcluded"
                            | "ClosedShadowRootsExcluded"
                            | "NearbySamplesNotAtomic"
                    )
            })
    }
    pub fn verify_snapshot(&self, bytes: &[u8]) -> Result<DomSnapshot> {
        self.validate()?;
        ensure!(
            bytes.len() as u64 == self.bytes && dom_artifact_id(bytes) == self.sha256,
            "DOM artifact identity mismatch"
        );
        let snapshot = DomSnapshot::from_canonical(bytes)?;
        let stats = snapshot.validate()?;
        ensure!(
            stats.node_count == self.node_count
                && stats.max_depth == self.max_depth
                && stats.coverage == self.coverage,
            "DOM artifact diagnostics mismatch"
        );
        Ok(snapshot)
    }
    pub fn validate_for_run(&self, run: &CaptureRun) -> Result<()> {
        self.validate()?;
        ensure!(
            self.run == run.id
                && Some(self.policy) == run.capture_plan.dom_policy
                && run.profiles.iter().any(|p| p == &self.profile),
            "DOM evidence/frozen run mismatch"
        );
        let target = run
            .targets
            .iter()
            .find(|t| t.key == self.key)
            .ok_or_else(|| anyhow::anyhow!("foreign DOM target"))?;
        // The target owns logical inventory identity; metadata records the exact
        // approved URL sent to the browser, which can be a staging destination.
        let requested = crate::environment::map_url(&target.requested, &run.environment)?;
        ensure!(
            requested.as_str() == self.requested
                && target
                    .final_url
                    .as_ref()
                    .is_some_and(|u| u.as_str() == self.final_url)
                && matches!(&target.dom_outcome, DomTargetOutcome::Captured { artifact } if artifact == &self.sha256),
            "DOM evidence/outcome mismatch"
        );
        Ok(())
    }
}

pub fn dom_source(run: &CaptureRun, capture: &DomCaptureEvidence) -> Result<DomSourceIdentity> {
    capture.metadata.validate_for_run(run)?;
    Ok(DomSourceIdentity {
        run: run.id,
        key: capture.metadata.key.clone(),
        artifact: capture.metadata.sha256.clone(),
        policy: capture.metadata.policy,
        material_profile: identity_hash(&capture.metadata.profile)?.try_into()?,
        rendering_environment: identity_hash(&run.rendering)?.try_into()?,
        stable: capture.metadata.stable,
    })
}
pub fn identity_hash(value: &impl Serialize) -> anyhow::Result<String> {
    use sha2::{Digest, Sha256};
    Ok(hex::encode(Sha256::digest(serde_json::to_vec(value)?)))
}
pub fn descriptor(
    run: &RunRecord,
    capture: &CaptureEvidence,
) -> anyhow::Result<crate::analysis_engine::CaptureDescriptor> {
    use crate::analysis_engine::*;
    Ok(CaptureDescriptor {
        capture_id: format!("{}:{}", run.run.id, capture.image.original.as_str()),
        target: Target {
            page: capture.metadata.key.page.to_string(),
            profile: capture.metadata.key.profile.to_string(),
        },
        dimensions: Dimensions {
            width: capture.image.width,
            height: capture.image.height,
        },
        device_scale_factor: capture.metadata.profile.dpr as f64,
        material_profile_id: identity_hash(&capture.metadata.profile)?,
        rendering_environment_id: identity_hash(&run.run.rendering)?,
        completed: true,
    })
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct QueueSettings {
    pub discovery: DiscoverySettings,
    pub project_revision: u64,
    pub resumed_at: Vec<i64>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComparisonEvidence {
    pub revision: AnalysisRevision,
    pub heatmap: Option<ImageEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub heatmaps: Vec<(TargetKey, ImageEvidence)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub perceptual: Option<PerceptualEvidence>,
}

fn pixel_review_classification(
    result: &AnalysisResult,
    report: Option<&crate::analysis_engine::AnalysisReport>,
    failed: bool,
) -> Classification {
    use crate::analysis_engine as engine;
    if result.classification != Classification::NotComparable || failed {
        return result.classification;
    }
    let Some(report) = report else {
        return result.classification;
    };
    if report.eligible_pixels == 0
        || !report.gates.is_empty()
        || report
            .warnings
            .contains(&engine::Warning::RenderingEnvironmentMismatch)
        || report.eligible_pixels != result.eligible_pixels
        || report.changed_pixels != result.changed_pixels
        || report.request.before.target.page != result.key.page.to_string()
        || report.request.before.target.profile != result.key.profile.to_string()
    {
        return result.classification;
    }
    match report.classification {
        engine::Classification::Changed => Classification::Changed,
        engine::Classification::NoSignificantChange => Classification::NoSignificantChange,
        engine::Classification::NotComparable => Classification::NotComparable,
    }
}

impl ComparisonEvidence {
    /// Use the actual saved pixel verdict, even when an earlier app version
    /// vetoed its summary for capture cautions. This is read-only: original
    /// summaries/reports, revision IDs, notes and decisions are never rewritten.
    /// Storage verifies report/source bindings before supplying this evidence.
    pub fn review_classification(&self, key: &TargetKey) -> Option<Classification> {
        let result = self.revision.results.iter().find(|r| &r.key == key)?;
        let perceptual = self.perceptual.as_ref();
        let report =
            perceptual.and_then(|p| p.reports.iter().find(|(k, _)| k == key).map(|(_, r)| r));
        let failed = perceptual.is_some_and(|p| p.failures.iter().any(|(k, _)| k == key));
        Some(pixel_review_classification(result, report, failed))
    }

    /// Index once for coverage, queue filters and progress, rather than scanning
    /// every report again for each target in a large project.
    pub fn review_classifications(&self) -> std::collections::BTreeMap<TargetKey, Classification> {
        if !self
            .revision
            .results
            .iter()
            .any(|r| r.classification == Classification::NotComparable)
        {
            return self
                .revision
                .results
                .iter()
                .map(|r| (r.key.clone(), r.classification))
                .collect();
        }
        let reports: std::collections::BTreeMap<_, _> = self
            .perceptual
            .iter()
            .flat_map(|p| p.reports.iter())
            .map(|(key, report)| (key, report))
            .collect();
        let failed: std::collections::BTreeSet<_> = self
            .perceptual
            .iter()
            .flat_map(|p| p.failures.iter())
            .map(|(key, _)| key)
            .collect();
        self.revision
            .results
            .iter()
            .map(|result| {
                (
                    result.key.clone(),
                    pixel_review_classification(
                        result,
                        reports.get(&result.key).copied(),
                        failed.contains(&result.key),
                    ),
                )
            })
            .collect()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PerceptualEvidence {
    pub policy: crate::analysis_engine::Policy,
    pub cache_key: String,
    pub created_at: i64,
    pub reports: Vec<(TargetKey, crate::analysis_engine::AnalysisReport)>,
    /// Analysis failure is not a capture failure or an unchanged result.
    pub failures: Vec<(TargetKey, String)>,
}
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ProjectEvidence {
    pub latest: Option<RunRecord>,
    pub baseline: Option<BaselineVersion>,
    pub before: Option<CaptureEvidence>,
    pub comparison: Option<ComparisonEvidence>,
    #[serde(default)]
    pub analyses: Vec<AnalysisId>,
    #[serde(default)]
    pub reviews: Vec<ReviewDecision>,
    /// Recent run-wide annotation history is not authoritative current state.
    /// Terminal current decisions and paginated history use focused store reads.
    #[serde(default)]
    pub review_history_truncated: bool,
    #[serde(default)]
    pub baselines: Vec<BaselineVersion>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dom_before: Option<DomCaptureEvidence>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub dom_comparison: Option<DomComparisonEvidence>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dom_analyses: Vec<DomAnalysisId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub dom_reviews: Vec<DomReviewDecision>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DomComparisonEvidence {
    pub revision: DomAnalysisRevision,
    pub created_at: i64,
    pub options: DomDiffOptions,
}
impl DomComparisonEvidence {
    pub fn validate(&self) -> Result<()> {
        self.revision.validate()?;
        self.options.validate()
    }
}
