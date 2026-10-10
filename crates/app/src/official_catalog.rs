// SPDX-License-Identifier: MPL-2.0
//! Immutable application trust root for official component provisioning.
//! No environment, configuration file, remote catalog or signing-key substitute.
use anyhow::{Context, Result, ensure};
use regressee_core::domain::RuntimeIdentity;
use regressee_runtime::{
    distribution,
    official::{self, Catalog},
};
use std::sync::OnceLock;

pub(crate) fn bundled() -> &'static [u8] {
    include_bytes!("../../runtime/assets/official/bundled-worker.zip")
}

fn metadata() -> Result<&'static str> {
    match (std::env::consts::OS, std::env::consts::ARCH) {
        ("linux", "x86_64") => Ok(include_str!(
            "../../runtime/assets/official/linux-x86_64.json"
        )),
        ("windows", "x86_64") => Ok(include_str!(
            "../../runtime/assets/official/windows-x86_64.json"
        )),
        ("macos", "aarch64") => Ok(include_str!(
            "../../runtime/assets/official/macos-aarch64.json"
        )),
        ("macos", "x86_64") => Ok(include_str!(
            "../../runtime/assets/official/macos-x86_64.json"
        )),
        _ => anyhow::bail!("No app-pinned browser catalog for this platform"),
    }
}

pub(crate) fn current() -> Result<&'static Catalog> {
    static CATALOG: OnceLock<std::result::Result<Catalog, ()>> = OnceLock::new();
    let catalog = CATALOG
        .get_or_init(|| {
            (|| -> Result<Catalog> {
                let catalog: Catalog = serde_json::from_str(metadata()?)?;
                ensure!(
                    catalog.manifest.version == 3
                        && catalog.manifest.scope == distribution::Scope::OfficialPinned
                        && catalog.manifest.os == std::env::consts::OS
                        && catalog.manifest.arch == std::env::consts::ARCH,
                    "App-pinned browser catalog platform or scope mismatch"
                );
                ensure!(
                    catalog.manifest.identity
                        == RuntimeIdentity::Pinned {
                            node: "24.21.0".into(),
                            worker: regressee_runtime::protocol::WORKER_VERSION.into(),
                            playwright: "1.64.0".into(),
                            browser: "156.0.8078.4".into(),
                        },
                    "App-pinned browser catalog identity mismatch"
                );
                // Full embedded worker-bundle checksum and inventory validation once.
                official::validate(&catalog, bundled())?;
                Ok(catalog)
            })()
            .map_err(|_| ())
        })
        .as_ref()
        .map_err(|_| anyhow::anyhow!("App-pinned browser catalog could not be validated"))?;
    // Do not cache time-based validity. An expired release cannot install or
    // become ready even if this process loaded it before the support deadline.
    ensure!(
        catalog
            .manifest
            .support_until
            .context("App-pinned browser support deadline is missing")?
            > distribution::now(),
        "App-pinned browser support expired"
    );
    Ok(catalog)
}
