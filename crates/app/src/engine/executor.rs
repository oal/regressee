// SPDX-License-Identifier: MPL-2.0
use crate::{
    jobs::{self, Reply, StorageFailure, Update, Work},
    product::Product,
};
use regressee_storage::Store;
use std::sync::{Arc, atomic::AtomicBool, mpsc::SyncSender};

/// Shared product handlers; only the engine worker owns this Store/Product.
pub fn execute(
    store: &mut Store,
    product: &anyhow::Result<Product>,
    work: Work,
    cancel: &Arc<AtomicBool>,
    pause: &Arc<AtomicBool>,
    updates: &SyncSender<Update>,
) -> Reply {
    if cancel.load(std::sync::atomic::Ordering::Acquire) {
        return Reply::Cancelled;
    }
    let result = (|| -> anyhow::Result<Reply> {
        Ok(match work {
            Work::Dom(command) => {
                Reply::Dom(Box::new(super::dom::execute(store, command, cancel)?))
            }
            Work::List { page } => store
                .list_projects(page)
                .map(Reply::Projects)
                .unwrap_or(Reply::Failure(StorageFailure::MissingOrCorrupt)),
            Work::Load(id) => store
                .load_project(id)
                .map(|p| Reply::Project(Box::new(p)))
                .unwrap_or(Reply::Failure(StorageFailure::MissingOrCorrupt)),
            Work::Save {
                project,
                expected_revision,
            } => {
                if project.validate().is_err() {
                    Reply::Failure(StorageFailure::InvalidProject)
                } else {
                    store
                        .save_project(&project, expected_revision)
                        .map(|p| Reply::Project(Box::new(p)))
                        .unwrap_or_else(|e| {
                            jobs::storage_failure(e, StorageFailure::ConflictOrWrite)
                        })
                }
            }
            work @ (Work::SaveEnvironment { .. } | Work::SaveReplacementEnvironment { .. }) => {
                jobs::environment_work(store, work)
            }
            Work::InspectStorage => store
                .summary()
                .map(Reply::Storage)
                .unwrap_or(Reply::Failure(StorageFailure::Unavailable)),
            Work::InspectRun { project, run } => Reply::InspectedRun {
                project,
                run,
                evidence: Box::new(jobs::inspect_screenshots(store, project, run)?),
            },
            Work::DiagnosticPreview => Reply::DiagnosticPreview(Box::new(
                crate::diagnostics::collect(store, product.as_ref().ok(), cancel)?,
            )),
            Work::DiagnosticExport { report, path } => {
                crate::diagnostics::export(&report, &path, cancel)?;
                Reply::DiagnosticExported
            }
            work @ (Work::ExportPreview { .. }
            | Work::Export { .. }
            | Work::ImportPreview(_)
            | Work::Import { .. }
            | Work::History { .. }
            | Work::ProjectStorage(_)
            | Work::DeletePreview { .. }
            | Work::Delete(_)
            | Work::RepairPreview
            | Work::CleanupTemporaries(_)
            | Work::DeleteUnreferenced(_)) => {
                jobs::storage_work(store, work, cancel, product.as_ref().ok())
            }
            Work::Extension(never) => match never {},
            work => match product {
                Ok(product) => jobs::product_work(product, store, work, cancel, pause, updates, 0),
                Err(_) if matches!(work, Work::RuntimeStatus) => Reply::RuntimeReadiness(
                    crate::product::RuntimeReadiness::invalid_configuration(),
                ),
                Err(_) => Reply::ActionRequired(crate::recovery::Problem::Browser(
                    crate::product::RuntimeState::UnavailableDistribution,
                )),
            },
        })
    })();
    jobs::operation_reply(result)
}
