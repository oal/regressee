// SPDX-License-Identifier: MPL-2.0
//! Consent-only local diagnostic document. Schema has no free-form private fields.
use anyhow::{Context, Result, ensure};
use regressee_core::domain::RuntimeIdentity;
use serde::{Deserialize, Serialize};
use std::{
    io::Write,
    path::Path,
    sync::atomic::{AtomicBool, Ordering},
};
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum RuntimeState {
    Unconfigured,
    Unavailable,
    VerifiedLoopback,
    VerifiedPublic,
    VerifiedOfficialPinned,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Versions {
    pub node: Vec<u32>,
    pub worker: Vec<u32>,
    pub playwright: Vec<u32>,
    pub browser: Vec<u32>,
}
fn version(value: &str) -> Result<Vec<u32>> {
    ensure!(
        value.len() <= 48 && !value.is_empty(),
        "Invalid diagnostic version"
    );
    let parts = value
        .strip_prefix('v')
        .unwrap_or(value)
        .split('.')
        .map(|p| {
            ensure!(
                !p.is_empty() && p.bytes().all(|c| c.is_ascii_digit()),
                "Invalid diagnostic version"
            );
            Ok(p.parse::<u32>()?)
        })
        .collect::<Result<Vec<_>>>()?;
    ensure!(parts.len() <= 4, "Invalid diagnostic version");
    Ok(parts)
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Platform {
    LinuxX64,
    WindowsX64,
    MacArm64,
    MacX64,
    Other,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Report {
    pub schema: u32,
    pub app_version: [u32; 3],
    pub database_schema: u32,
    pub platform: Platform,
    pub runtime: RuntimeState,
    pub versions: Option<Versions>,
    pub counts: regressee_storage::DiagnosticCounts,
}
pub const PREVIEW: &str = "Included: app/database schema, OS family/CPU, verified runtime state and numeric versions, outcome counts (latest 100 runs, at most 10,000 targets; table counts capped at 100,001). Excluded: screenshots, names, site addresses and queries, sessions and references, secrets, paths, logs, device identifiers and free-form errors. This saves a local JSON file only; nothing is sent. Review the contents, then choose Export or Cancel.";
pub fn collect(
    store: &regressee_storage::Store,
    product: Option<&crate::product::Product>,
    cancel: &AtomicBool,
) -> Result<Report> {
    let mut report = Report {
        schema: 1,
        app_version: [0, 1, 0],
        database_schema: regressee_storage::DB_VERSION as u32,
        platform: match (std::env::consts::OS, std::env::consts::ARCH) {
            ("linux", "x86_64") => Platform::LinuxX64,
            ("windows", "x86_64") => Platform::WindowsX64,
            ("macos", "aarch64") => Platform::MacArm64,
            ("macos", "x86_64") => Platform::MacX64,
            _ => Platform::Other,
        },
        runtime: RuntimeState::Unconfigured,
        versions: None,
        counts: store.diagnostic_counts()?,
    };
    if let Some(product) = product
        && (product.distribution.is_some() || product.official_runtime)
    {
        report.runtime = RuntimeState::Unavailable;
        if let Ok(installed) = product.resolve_runtime(cancel) {
            report.runtime = match installed.manifest.scope {
                regressee_runtime::distribution::Scope::DevelopmentLoopback => {
                    RuntimeState::VerifiedLoopback
                }
                regressee_runtime::distribution::Scope::QualifiedPublic => {
                    RuntimeState::VerifiedPublic
                }
                regressee_runtime::distribution::Scope::OfficialPinned => {
                    RuntimeState::VerifiedOfficialPinned
                }
            };
            if let RuntimeIdentity::Pinned {
                node,
                worker,
                playwright,
                browser,
            } = &installed.manifest.identity
            {
                // Even trusted strings are not copied into diagnostics.
                report.versions = (|| -> Result<Versions> {
                    Ok(Versions {
                        node: version(node)?,
                        worker: version(worker)?,
                        playwright: version(playwright)?,
                        browser: version(browser)?,
                    })
                })()
                .ok();
            }
        }
    }
    ensure!(
        !cancel.load(Ordering::Acquire),
        "Diagnostic preview cancelled"
    );
    Ok(report)
}
pub fn export(report: &Report, destination: &Path, cancel: &AtomicBool) -> Result<()> {
    ensure!(report.schema == 1, "Unsupported diagnostic schema");
    let bytes = serde_json::to_vec_pretty(report)?;
    ensure!(bytes.len() <= 16384, "Diagnostic budget exceeded");
    let parent = destination
        .parent()
        .context("Diagnostic destination has no parent")?;
    regressee_runtime::distribution::safe_directory(parent)?;
    let mut file = tempfile::NamedTempFile::new_in(parent)?;
    file.write_all(&bytes)?;
    file.as_file().sync_all()?;
    ensure!(
        !cancel.load(Ordering::Acquire),
        "Diagnostic export cancelled"
    );
    file.persist_noclobber(destination).map_err(|e| e.error)?;
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn structural_privacy_versions_cancel_and_no_overwrite() -> Result<()> {
        let root = tempfile::tempdir()?;
        let mut store =
            regressee_storage::Store::open(regressee_storage::AppPaths::isolated(root.path()))?;
        let project = regressee_core::domain::Project::draft(
            "PRIVATE_NAME_SENTINEL".into(),
            "https://example.test/?PRIVATE_QUERY_SENTINEL=secret",
        )?;
        store.save_project(&project, None)?;
        let report = collect(&store, None, &AtomicBool::new(false))?;
        let bytes = serde_json::to_string(&report)?;
        for private in [
            "PRIVATE_NAME_SENTINEL",
            "PRIVATE_QUERY_SENTINEL",
            "example.test",
            root.path().to_str().unwrap(),
        ] {
            assert!(!bytes.contains(private));
        }
        assert_eq!(report.counts.projects, 1);
        assert!(version("1.2.3/session-secret").is_err());
        assert!(version("99999999999999999999999").is_err());
        assert_eq!(version("v22.20.0")?, vec![22, 20, 0]);
        let path = root.path().join("diagnostics.json");
        assert!(export(&report, &path, &AtomicBool::new(true)).is_err());
        assert!(!path.exists());
        export(&report, &path, &AtomicBool::new(false))?;
        let original = std::fs::read(&path)?;
        assert!(export(&report, &path, &AtomicBool::new(false)).is_err());
        assert_eq!(std::fs::read(path)?, original);
        Ok(())
    }

    #[test]
    fn app_pinned_diagnostics_are_read_only_and_never_copy_catalog_sources() -> Result<()> {
        let root = tempfile::tempdir()?;
        let paths = regressee_storage::AppPaths::isolated(root.path());
        let store = regressee_storage::Store::open(paths.clone())?;
        let product = crate::product::Product {
            distribution: None,
            official_runtime: true,
            runtime_cache: paths.cache.join("product-runtime"),
            auth: Default::default(),
        };
        let report = collect(&store, Some(&product), &AtomicBool::new(false))?;
        assert_eq!(report.runtime, RuntimeState::Unavailable);
        assert!(report.versions.is_none());
        let json = serde_json::to_string(&report)?;
        for private in [
            "https://",
            "nodejs.org",
            "googleapis.com",
            "sha256",
            "public_key",
            root.path().to_str().unwrap(),
        ] {
            assert!(!json.contains(private));
        }
        assert!(!product.runtime_cache.exists());
        Ok(())
    }
}
