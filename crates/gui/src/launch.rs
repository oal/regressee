// SPDX-License-Identifier: MPL-2.0
pub use regressee_app::launch::{app_paths, desktop_paths};

/// Native executable metadata, before graphics or storage startup.
pub fn build_info() -> serde_json::Value {
    let mut info = regressee_app::launch::build_info();
    info["executable"] = "regressee-desktop".into();
    info["version"] = env!("CARGO_PKG_VERSION").into();
    info["features"]["ui"] = cfg!(feature = "ui").into();
    // Report the application layer too: Cargo may unify its features through
    // another workspace target even when this crate's own feature is disabled.
    for (name, enabled) in [
        ("developer-tools", cfg!(feature = "developer-tools")),
        ("test-support", cfg!(feature = "test-support")),
        ("ui-inspection", cfg!(feature = "ui-inspection")),
        ("ui-renderer-capture", cfg!(feature = "ui-renderer-capture")),
    ] {
        info["features"][name] = (enabled || info["features"][name] == true).into();
    }
    info
}
