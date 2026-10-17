// SPDX-License-Identifier: MPL-2.0
//! Normal builds use only the OS application directories.
use anyhow::Result;
use regressee_storage::AppPaths;

pub fn app_paths() -> Result<AppPaths> {
    #[cfg(feature = "developer-tools")]
    if let Some(root) = std::env::var_os("REGRESSEE_DEV_DATA") {
        return Ok(AppPaths::isolated(std::path::Path::new(&root)));
    }
    AppPaths::discover()
}

/// Data-free executable metadata, available before graphics or storage startup.
pub fn build_info() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "features": {
            "ui": false,
            "developer-tools": cfg!(feature = "developer-tools"),
            "test-support": cfg!(feature = "test-support"),
            "ui-inspection": false,
            "ui-renderer-capture": false,
        }
    })
}
