// SPDX-License-Identifier: MPL-2.0
//! Native presentation and its display-only background work.
#[cfg(feature = "ui")]
pub mod activity;
#[cfg(feature = "ui")]
pub mod dom_source;
#[cfg(feature = "ui")]
pub mod jobs;
pub mod launch;
#[cfg(all(feature = "ui", any(test, feature = "test-support")))]
mod local_jobs;
#[cfg(feature = "ui")]
pub mod recovery;
#[cfg(feature = "ui")]
pub mod task;
#[cfg(feature = "ui")]
pub mod ui;
#[cfg(all(feature = "ui", any(test, feature = "test-support")))]
mod viewer_cache;
#[cfg(feature = "ui")]
pub mod viewer_images;
