// SPDX-License-Identifier: MPL-2.0
//! Phase 2 persisted evidence, not worker-owned filesystem paths.
use crate::domain::*;
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
        let captured = self.all_captures().count();
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
    #[serde(default)]
    pub baselines: Vec<BaselineVersion>,
}
