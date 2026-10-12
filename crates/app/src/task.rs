// SPDX-License-Identifier: MPL-2.0
//! Task presentation is independent of the destination and screenshot state.
use crate::jobs::Work;
use regressee_core::{crawl::TargetCounts, domain::*};

/// Small, UI-safe snapshot of a run's frozen setup and actual target counts.
/// No screenshots, session material, inferred progress or activity log.
#[derive(Debug, Clone)]
pub struct CaptureStatus {
    pub run: RunId,
    pub project: ProjectId,
    pub pages: usize,
    pub profiles: Vec<String>,
    pub environment: String,
    pub total: u32,
    pub counts: TargetCounts,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

impl CaptureStatus {
    pub fn from_run(run: &CaptureRun) -> Self {
        Self {
            run: run.id,
            project: run.project,
            pages: run.inventory.pages.iter().filter(|p| p.included).count(),
            profiles: run
                .profiles
                .iter()
                .map(|p| {
                    format!(
                        "{} · {} × {} · {}",
                        p.name,
                        p.width,
                        p.height,
                        match p.extent {
                            CaptureExtent::FullPage => "Full page",
                            CaptureExtent::VisibleScreen => "Visible screen",
                        }
                    )
                })
                .collect(),
            environment: run.environment.name.clone(),
            total: run.targets.len() as u32,
            counts: TargetCounts::from_run(run),
            started_at: run.started_at,
            finished_at: run.finished_at,
        }
    }

    pub fn processed(&self) -> u32 {
        self.total.saturating_sub(self.counts.pending as u32)
    }
}

pub const MAX_TASK_GAPS: usize = 20;
/// Context address, then requested/final addresses for each bounded gap row.
pub const MAX_TASK_LINKS: usize = 1 + 2 * MAX_TASK_GAPS;

#[derive(Debug, Clone)]
pub struct CaptureGap {
    pub page: String,
    pub final_url: Option<String>,
    pub profile: String,
    pub reason: String,
    pub failure: Option<CaptureError>,
    pub skipped: bool,
    pub attempts: u8,
    pub warnings: Vec<String>,
}

/// Retained from the authoritative task reply, never today's edited settings or
/// another run's `latest` pointer. Large gap lists stay in the review workspace.
#[derive(Debug, Clone)]
pub struct CaptureDetails {
    pub setup: CaptureStatus,
    pub gaps: Vec<CaptureGap>,
    pub gap_count: usize,
}

impl CaptureDetails {
    pub fn from_run(run: &CaptureRun) -> Self {
        let setup = CaptureStatus::from_run(run);
        let gap_count = setup.counts.failed
            + setup.counts.skipped
            + setup.counts.pending
            + setup.counts.cancelled;
        // Failures lead, so pending targets in a paused run cannot hide them.
        let mut gaps = Vec::new();
        for failed in [true, false] {
            for target in &run.targets {
                if gaps.len() == MAX_TASK_GAPS {
                    break;
                }
                let (reason, failure, skipped) = match &target.outcome {
                    TargetOutcome::Failed { code } if failed => {
                        (code.label().to_owned(), Some(*code), false)
                    }
                    TargetOutcome::Skipped { reason } if !failed => (reason.clone(), None, true),
                    TargetOutcome::Pending if !failed => ("Not captured yet".into(), None, false),
                    TargetOutcome::Cancelled if !failed => {
                        ("Not captured before stopping".into(), None, false)
                    }
                    _ => continue,
                };
                let profile = run
                    .profiles
                    .iter()
                    .find(|p| p.id == target.key.profile)
                    .map_or_else(
                        || "Screen size not recorded".into(),
                        |p| format!("{} · {} × {}", p.name, p.width, p.height),
                    );
                gaps.push(CaptureGap {
                    page: target.requested.to_string(),
                    final_url: target
                        .final_url
                        .as_ref()
                        .filter(|url| **url != target.requested)
                        .map(ToString::to_string),
                    profile,
                    reason,
                    failure,
                    skipped,
                    attempts: target.attempts,
                    warnings: target.warnings.clone(),
                });
            }
        }
        Self {
            setup,
            gaps,
            gap_count,
        }
    }
}

pub fn capture_recovery(code: CaptureError) -> &'static str {
    match code {
        CaptureError::NeedsLogin => "Log in again in Project settings before retrying.",
        CaptureError::AccessDenied => "Check that your login can open this page before retrying.",
        CaptureError::NotFound => "Check the page address and whether the page still exists.",
        CaptureError::RedirectOutsideScope => {
            "Check the website boundaries and environment mappings."
        }
        CaptureError::Timeout | CaptureError::Unstable => {
            "Check that this page loads normally, then retry the capture."
        }
        CaptureError::ImageLimit => {
            "Choose visible-screen-only capture in Project settings, then find pages and capture again."
        }
        CaptureError::LowSpace => {
            "Free up disk space before retrying. Inspect saved evidence in History before deleting anything."
        }
        CaptureError::RateLimited => "Choose a slower capture speed before retrying.",
        CaptureError::ServerError => "Check that the website is available, then retry.",
        CaptureError::WorkerStopped | CaptureError::CaptureFailed | CaptureError::Cancelled => {
            "Return to the workspace to review the saved work and retry the missing capture."
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Surface {
    Workspace,
    Dialog,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Discovery,
    Capture,
    Check,
    Setup,
    Login,
    Access,
    Representative,
    ArchivePreview,
    Import,
    Export,
    Diagnostics,
    Storage,
    Baseline,
    Save,
    Delete,
}

#[derive(Clone, Copy, Debug)]
pub struct Spec {
    pub kind: Kind,
    pub surface: Surface,
    pub title: &'static str,
    pub stop_label: Option<&'static str>,
    pub pausable: bool,
    pub show_result: bool,
}

impl Spec {
    /// Ordinary reads and debounced analysis keep their lightweight, local UX.
    /// Every exclusive operation has an explicit policy; no UI flag grants
    /// cancellation to a backend that cannot honor it.
    pub fn for_work(work: &Work) -> Option<Self> {
        use Kind::*;
        use Surface::*;
        let (kind, surface, title, stop_label, pausable, show_result) = match work {
            Work::Discover(_) => (
                Discovery,
                Workspace,
                "Finding pages",
                Some("Stop finding pages"),
                false,
                true,
            ),
            Work::Capture { check: true, .. } | Work::CaptureInventory { check: true, .. } => (
                Check,
                Workspace,
                "Checking for changes",
                Some("Stop check"),
                matches!(work, Work::CaptureInventory { .. }),
                true,
            ),
            Work::Capture { .. } | Work::CaptureInventory { .. } => (
                Capture,
                Workspace,
                "Capturing screenshots",
                Some("Stop capture"),
                matches!(work, Work::CaptureInventory { .. }),
                true,
            ),
            Work::SetupRuntime => (
                Setup,
                Workspace,
                "Setting up browser",
                Some("Cancel setup"),
                false,
                true,
            ),
            Work::OpenLogin(_) => (
                Login,
                Dialog,
                "Opening login browser",
                Some("Cancel browser launch"),
                false,
                false,
            ),
            Work::ConfirmLogin(_) => (
                Login,
                Dialog,
                "Saving login session",
                Some("Cancel session check"),
                false,
                false,
            ),
            Work::ProbeEnvironment(_) => (
                Access,
                Dialog,
                "Checking website access",
                Some("Cancel access check"),
                false,
                true,
            ),
            Work::PreviewImages { .. } => (
                Representative,
                Dialog,
                "Capturing a preview",
                Some("Stop preview"),
                false,
                true,
            ),
            Work::ImportPreview(_) => (
                ArchivePreview,
                Dialog,
                "Reading baseline ZIP",
                Some("Cancel ZIP inspection"),
                false,
                false,
            ),
            Work::Import { .. } => (
                Import,
                Dialog,
                "Importing project and baseline",
                Some("Cancel import"),
                false,
                true,
            ),
            Work::Export { .. } => (
                Export,
                Dialog,
                "Saving baseline ZIP",
                Some("Cancel export"),
                false,
                true,
            ),
            Work::DiagnosticPreview => (
                Diagnostics,
                Dialog,
                "Preparing diagnostic preview",
                Some("Cancel preview"),
                false,
                false,
            ),
            Work::DiagnosticExport { .. } => (
                Diagnostics,
                Dialog,
                "Saving diagnostic file",
                Some("Cancel export"),
                false,
                true,
            ),
            Work::InspectStorage | Work::ProjectStorage(_) | Work::RepairPreview => (
                Storage,
                Dialog,
                "Checking local storage",
                None,
                false,
                false,
            ),
            Work::ExportPreview { .. } => (
                ArchivePreview,
                Dialog,
                "Preparing baseline ZIP preview",
                None,
                false,
                false,
            ),
            Work::DeletePreview { .. } => (
                Storage,
                Dialog,
                "Preparing deletion preview",
                None,
                false,
                false,
            ),
            Work::Activate { .. } | Work::Accept { .. } | Work::Restore { .. } => (
                Baseline,
                Dialog,
                work.activity_label().trim_end_matches('…'),
                None,
                false,
                false,
            ),
            Work::Delete(_) | Work::CleanupTemporaries(_) | Work::DeleteUnreferenced(_) => (
                Delete,
                Dialog,
                work.activity_label().trim_end_matches('…'),
                None,
                false,
                false,
            ),
            Work::Save { .. }
            | Work::SaveReplacement { .. }
            | Work::SaveEnvironment { .. }
            | Work::SaveNetwork(_)
            | Work::Mask { .. }
            | Work::Review(_)
            | Work::CloseLogin(_)
            | Work::ForgetLogin(_)
            | Work::Approval { .. } => (
                Save,
                Dialog,
                work.activity_label().trim_end_matches('…'),
                None,
                false,
                false,
            ),
            Work::List { .. }
            | Work::Load(_)
            | Work::History { .. }
            | Work::InspectRun { .. }
            | Work::RuntimeStatus
            | Work::NetworkStatus
            | Work::Evidence(_)
            | Work::ExistingInventory(_)
            | Work::EnvironmentStatus(_)
            | Work::Reanalyze { .. }
            | Work::LoadAnalysis(_) => return None,
            #[cfg(feature = "ui")]
            Work::Viewer { .. } | Work::HistoricalViewer { .. } => return None,
        };
        Some(Self {
            kind,
            surface,
            title,
            stop_label,
            pausable,
            show_result,
        })
    }

    pub fn reassurance(self) -> &'static str {
        match self.kind {
            Kind::Discovery => {
                "Your saved baseline stays unchanged. If a page list is ready when you stop, it is kept as an incomplete list."
            }
            Kind::Capture | Kind::Check | Kind::Representative => {
                "Completed screenshots are kept. Your saved baseline stays unchanged."
            }
            Kind::Setup => {
                "Your projects and screenshots stay unchanged. A browser already installed successfully will be kept."
            }
            Kind::Import => {
                "An existing project is never replaced. If you cancel before saving, recovery files may remain in local storage."
            }
            Kind::Export => {
                "Your project stays unchanged. Cancellation stops the ZIP before publication; an already saved ZIP is kept."
            }
            Kind::ArchivePreview => {
                "This preview does not import a project or change your baseline."
            }
            Kind::Diagnostics => {
                "Nothing is sent. Cancellation cannot remove a diagnostic file already saved successfully."
            }
            Kind::Login => {
                "An already saved login session is kept. Once the browser opens, log in there and return to the app."
            }
            Kind::Access => "Checking access does not capture screenshots or update your baseline.",
            Kind::Storage => "This inspection does not delete your saved evidence.",
            Kind::Baseline => {
                "Earlier baseline versions are kept. This confirmed update must finish safely; cancellation cannot undo it."
            }
            Kind::Save => {
                "This action must finish safely. Already saved changes cannot be undone by closing this view."
            }
            Kind::Delete => {
                "This confirmed cleanup must finish safely. Files already removed cannot be restored by cancelling."
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum State {
    Running,
    Stopping,
    Pausing,
    Finished,
}

impl State {
    pub fn request_stop(&mut self, spec: Spec) -> bool {
        if !matches!(*self, Self::Running | Self::Pausing) || spec.stop_label.is_none() {
            return false;
        }
        *self = Self::Stopping;
        true
    }
    pub fn request_pause(&mut self, spec: Spec) -> bool {
        if *self != Self::Running || !spec.pausable {
            return false;
        }
        *self = Self::Pausing;
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn presentation_run(pages: usize) -> CaptureRun {
        let project = Project::draft("Synthetic task data".into(), "https://example.test").unwrap();
        let profiles = vec![CaptureProfile::desktop(), CaptureProfile::mobile()];
        let inventory = InventoryRevision {
            id: InventoryId::new(),
            project: project.id,
            identity_version: 1,
            pages: (0..pages)
                .map(|index| InventoryPage {
                    id: PageId::new(),
                    canonical: project.sources[0].join(&format!("/{index}")).unwrap(),
                    included: true,
                    reason: None,
                })
                .collect(),
            coverage: DiscoveryCoverage::Complete,
        };
        let targets = inventory
            .pages
            .iter()
            .flat_map(|page| {
                profiles.iter().map(move |profile| CaptureTarget {
                    key: TargetKey {
                        page: page.id,
                        profile: profile.id,
                    },
                    requested: page.canonical.clone(),
                    final_url: None,
                    attempts: 0,
                    outcome: TargetOutcome::Pending,
                    warnings: vec![],
                })
            })
            .collect();
        CaptureRun {
            id: RunId::new(),
            project: project.id,
            intent: RunIntent::NewBaseline,
            inventory,
            profiles,
            environment: project.environments[0].clone(),
            rendering: RenderingEnvironment {
                os: "synthetic".into(),
                architecture: "synthetic".into(),
                browser: "synthetic".into(),
                worker_version: "synthetic".into(),
                profile_version: 1,
                fonts_fingerprint: None,
            },
            state: JobState::Running,
            started_at: Some(0),
            finished_at: None,
            targets,
        }
    }

    #[test]
    fn processed_targets_do_not_mean_successful_screenshots_or_distinct_failed_pages() {
        let mut run = presentation_run(2);
        for target in &mut run.targets[..2] {
            target.outcome = TargetOutcome::Captured {
                artifact: "a".repeat(64).try_into().unwrap(),
                width: 1,
                height: 1,
            };
        }
        for target in &mut run.targets[2..] {
            target.outcome = TargetOutcome::Failed {
                code: CaptureError::Timeout,
            };
            target.attempts = 2;
        }
        let details = CaptureDetails::from_run(&run);
        assert_eq!(details.setup.pages, 2);
        assert_eq!(details.setup.total, 4);
        assert_eq!(details.setup.processed(), 4);
        assert_eq!(details.setup.counts.captured, 2);
        assert_eq!(details.setup.counts.failed, 2);
        assert_eq!(details.gaps[0].page, details.gaps[1].page);
        assert_ne!(details.gaps[0].profile, details.gaps[1].profile);
        assert_eq!(details.gaps[0].attempts, 2);
        let frozen = details.setup.profiles.clone();
        run.profiles[0].name = "Edited after this run".into();
        assert_eq!(details.setup.profiles, frozen);
    }

    #[test]
    fn bounded_gap_details_prioritize_failures_over_a_large_pending_queue() {
        let mut run = presentation_run(100);
        run.targets.last_mut().unwrap().outcome = TargetOutcome::Failed {
            code: CaptureError::LowSpace,
        };
        let details = CaptureDetails::from_run(&run);
        assert_eq!(details.gap_count, 200);
        assert_eq!(details.gaps.len(), MAX_TASK_GAPS);
        assert_eq!(details.gaps[0].failure, Some(CaptureError::LowSpace));
        assert!(capture_recovery(CaptureError::LowSpace).contains("disk space"));
        assert!(capture_recovery(CaptureError::RateLimited).contains("slower"));
    }
    use regressee_core::domain::ProjectId;

    #[test]
    fn cancellation_and_pause_follow_backend_capabilities_not_capture_wording() {
        let project = ProjectId::new();
        for work in [
            Work::Discover(project),
            Work::SetupRuntime,
            Work::ImportPreview("fixture.zip".into()),
            Work::ProbeEnvironment(project),
        ] {
            let spec = Spec::for_work(&work).unwrap();
            assert!(spec.stop_label.is_some());
            assert!(!spec.pausable);
            let mut state = State::Running;
            assert!(state.request_stop(spec));
            assert!(!state.request_stop(spec));
            assert!(!state.request_pause(spec));
            assert_eq!(state, State::Stopping);
        }
        let spec = Spec::for_work(&Work::CaptureInventory {
            project,
            preview: None,
            check: false,
            resume: None,
        })
        .unwrap();
        let mut state = State::Running;
        assert!(state.request_pause(spec));
        assert_eq!(state, State::Pausing);
        assert!(state.request_stop(spec));
        assert_eq!(state, State::Stopping);
        let save = Spec::for_work(&Work::SaveNetwork(Default::default())).unwrap();
        let mut state = State::Running;
        assert!(!state.request_stop(save));
        assert!(!state.request_pause(save));
        for work in [
            Work::List { page: 0 },
            Work::Evidence(project),
            Work::History { project, page: 0 },
            Work::RuntimeStatus,
        ] {
            assert!(Spec::for_work(&work).is_none());
        }
    }
}
