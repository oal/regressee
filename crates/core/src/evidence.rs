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
