// SPDX-License-Identifier: MPL-2.0
#[cfg(test)]
extern crate self as regressee_app;
pub mod analysis_engine;
pub mod appearance;
#[cfg(test)]
#[path = "../tests/support/approval_fixture.rs"]
pub(crate) mod approval_fixture;
pub mod auth;
mod capture_evidence;
pub mod control;
pub mod crawling;
pub mod diagnostics;
pub mod dom_snapshot;
pub mod engine;
pub mod formatting;
pub mod images;
pub mod jobs;
pub mod launch;
pub(crate) mod official_catalog;
pub mod product;
pub mod protocol;
pub mod recovery;
pub mod review;
pub mod task;
pub mod terminal;
