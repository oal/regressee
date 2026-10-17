// SPDX-License-Identifier: MPL-2.0
pub use regressee_app::launch::app_paths;

/// Native executable metadata, before graphics or storage startup.
pub fn build_info() -> serde_json::Value {
    serde_json::json!({
        "version": env!("CARGO_PKG_VERSION"),
        "features": {
            "ui": cfg!(feature = "ui"),
            "developer-tools": cfg!(feature = "developer-tools"),
            "test-support": cfg!(feature = "test-support"),
            "ui-inspection": cfg!(feature = "ui-inspection"),
            "ui-renderer-capture": cfg!(feature = "ui-renderer-capture"),
        }
    })
}
