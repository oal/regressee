// SPDX-License-Identifier: MPL-2.0
//! Private capture tracing is compiled in only for explicit developer builds.
use std::path::PathBuf;

pub fn trace_directory() -> Option<PathBuf> {
    #[cfg(feature = "developer-tools")]
    {
        std::env::var_os("REGRESSEE_CAPTURE_TRACE_EVIDENCE").map(PathBuf::from)
    }
    #[cfg(not(feature = "developer-tools"))]
    {
        None
    }
}
