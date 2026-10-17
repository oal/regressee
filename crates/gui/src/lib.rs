// SPDX-License-Identifier: MPL-2.0
//! Native presentation and its display-only background work.
#[cfg(feature = "ui")]
pub mod activity;
#[cfg(feature = "ui")]
pub mod jobs;
pub mod launch;
#[cfg(feature = "ui")]
pub mod recovery;
#[cfg(feature = "ui")]
pub mod task;
#[cfg(feature = "ui")]
pub mod ui;
#[cfg(feature = "ui")]
mod viewer_cache;
#[cfg(feature = "ui")]
pub mod viewer_images;
