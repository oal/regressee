// SPDX-License-Identifier: MPL-2.0
//! Versioned selected-baseline interchange. No local session or execution fields.
use crate::domain::*;
use crate::evidence::CaptureMetadata;
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use url::Url;

pub const ARCHIVE_VERSION: u32 = 1;
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableProject {
    pub name: String,
    pub sources: Vec<Url>,
    pub discovery: DiscoverySettings,
    pub environments: Vec<PortableEnvironment>,
    pub profiles: Vec<CaptureProfile>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableEnvironment {
    pub name: String,
    pub mappings: Vec<OriginMapping>,
    pub access: AccessPolicy,
}
impl From<&Project> for PortableProject {
    fn from(project: &Project) -> Self {
        Self {
            name: project.name.clone(),
            sources: project.sources.clone(),
            discovery: project.discovery.clone(),
            environments: project
                .environments
                .iter()
                .map(|env| PortableEnvironment {
                    name: env.name.clone(),
                    mappings: env.mappings.clone(),
                    access: env.access,
                })
                .collect(),
            profiles: project.profiles.clone(),
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ArchiveManifest {
    pub version: u32,
    pub project: PortableProject,
    pub inventory: InventoryRevision,
    pub baseline: BaselineVersion,
    pub artifacts: Vec<Artifact>,
    pub provenance: Vec<CaptureProvenance>,
    pub masks: Vec<PortableMask>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureProvenance {
    pub run: RunId,
    pub rendering: RenderingEnvironment,
    pub profiles: Vec<CaptureProfile>,
    pub environment: PortableEnvironment,
    pub started_at: i64,
    pub finished_at: i64,
    pub captures: Vec<CaptureMetadata>,
    pub discovery: DiscoverySettings,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PortableMask {
    pub key: TargetKey,
    pub width: u32,
    pub height: u32,
    pub rectangles: Vec<crate::analysis_engine::Rectangle>,
}
/// The only permitted artifact entry path; never a machine-local path.
pub fn artifact_entry(id: &ArtifactId) -> String {
    format!("artifacts/{}", id.as_str())
}
pub fn validate_entry(path: &str) -> Result<ArtifactId> {
    let hash = path
        .strip_prefix("artifacts/")
        .ok_or_else(|| anyhow::anyhow!("unexpected archive entry"))?;
    let id: ArtifactId = hash.to_owned().try_into()?;
    ensure!(artifact_entry(&id) == path, "noncanonical archive path");
    Ok(id)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn portable_contract_cannot_carry_login_or_local_paths() {
        let mut p = Project::draft("Site".into(), "https://example.test").unwrap();
        p.environments[0].session = Some(SessionReference {
            id: SessionId::new(),
            project: p.id,
            environment: p.environments[0].id,
        });
        let json = serde_json::to_string(&PortableProject::from(&p)).unwrap();
        assert!(!json.contains("session"));
        assert!(!json.contains(&p.environments[0].session.as_ref().unwrap().id.to_string()));
        let value = serde_json::from_str::<PortableEnvironment>(
            r#"{"name":"Site","mappings":[],"access":"Public","session":"secret"}"#,
        );
        assert!(value.is_err());
    }
    #[test]
    fn archive_paths_have_no_traversal_or_platform_aliases() {
        for path in [
            "artifacts/../cookies",
            "/artifacts/file",
            "artifacts\\file",
            "artifacts/AA",
            "artifacts/x/y",
        ] {
            assert!(validate_entry(path).is_err());
        }
        let id = "a".repeat(64).try_into().unwrap();
        assert_eq!(validate_entry(&artifact_entry(&id)).unwrap(), id);
    }
}
