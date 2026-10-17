// SPDX-License-Identifier: MPL-2.0
use crate::jobs::Work;
pub trait ActivityLabel {
    fn activity_label(&self) -> &'static str;
}
impl ActivityLabel for Work {
    /// Immediate, honest task feedback before the worker's first progress update.
    fn activity_label(&self) -> &'static str {
        match self {
            Self::List { .. } => "Loading projects…",
            Self::Load(_) => "Opening project…",
            Self::Save { .. } | Self::SaveReplacement { .. } => "Saving project settings…",
            Self::InspectStorage | Self::ProjectStorage(_) => "Measuring local storage…",
            Self::DiagnosticPreview => "Preparing support report preview…",
            Self::DiagnosticExport { .. } => "Saving support report…",
            Self::ExportPreview { .. } => "Preparing baseline ZIP preview…",
            Self::Export { .. } => "Saving baseline ZIP…",
            Self::ImportPreview(_) => "Reading baseline ZIP…",
            Self::Import { .. } => "Importing project and baseline…",
            Self::History { .. } => "Loading capture and baseline history…",
            Self::InspectRun { .. } => "Loading saved screenshots and results…",
            Self::Extension(_) => "Loading screenshot…",
            Self::DeletePreview { .. } => "Preparing deletion preview…",
            Self::Delete(_) => "Deleting selected history…",
            Self::RepairPreview => "Checking local storage…",
            Self::CleanupTemporaries(_) => "Removing confirmed temporary files…",
            Self::DeleteUnreferenced(_) => "Deleting recovery files…",
            Self::RuntimeStatus => "Checking installed browser…",
            Self::NetworkStatus => "Loading network settings…",
            Self::SaveNetwork(_) => "Saving network settings…",
            Self::SetupRuntime => "Preparing browser setup…",
            Self::Evidence(_) => "Loading saved screenshots and results…",
            Self::Capture { check: true, .. } | Self::CaptureInventory { check: true, .. } => {
                "Starting website check…"
            }
            Self::Capture { .. } | Self::CaptureInventory { .. } => "Starting screenshot capture…",
            Self::Discover(_) => "Preparing to find pages…",
            Self::ExistingInventory(_) => "Loading saved page list…",
            Self::EnvironmentStatus(_) => "Loading login and environment settings…",
            Self::SaveEnvironment { .. } | Self::SaveReplacementEnvironment { .. } => {
                "Saving login and environment settings…"
            }
            Self::OpenLogin(_) => "Opening login browser…",
            Self::ConfirmLogin(_) => "Checking and saving login…",
            Self::CloseLogin(_) => "Closing login browser…",
            Self::ForgetLogin(_) => "Removing saved login session…",
            Self::ProbeEnvironment(_) => "Checking website access…",
            Self::Activate { .. } | Self::Accept { .. } => "Saving baseline…",
            Self::Restore { .. } => "Restoring selected baseline…",
            Self::Reanalyze { .. } => "Updating comparison analysis…",
            Self::Mask { .. } => "Saving ignored areas…",
            Self::Review(_) => "Saving review decision…",
            Self::LoadAnalysis(_) => "Loading saved comparison…",
            Self::Approval { .. } => "Preparing baseline update preview…",
            Self::PreviewImages { .. } => "Capturing a preview screenshot…",
        }
    }
}
