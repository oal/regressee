// SPDX-License-Identifier: MPL-2.0
//! Versioned local protocol. Never serialize the actor's Work/Reply or native types.
use crate::{analysis_engine, auth, diagnostics, jobs, product, recovery, task};
use regressee_core::{appearance::AppAppearance, crawl::DiscoveryPreview, domain::*, evidence::*};
use regressee_runtime::{distribution::Scope, network::NetworkSettings};
use regressee_storage::*;
use serde::{Deserialize, Serialize};
use std::{
    path::PathBuf,
    sync::{Arc, atomic::AtomicBool},
};
use uuid::Uuid;

pub const VERSION: u32 = 6;
pub const MAX_FRAME: usize = 64 * 1024 * 1024;
pub const MAX_CONNECTIONS: usize = 16;
pub const MAX_JOBS: usize = 4096;
pub const MAX_EVENTS: usize = 512;
pub const MAX_QUEUE: usize = 64;

macro_rules! operations {
    ($($name:ident $(($binding:ident: $ty:ty))? $({$($field:ident: $ft:ty),* $(,)?})?),* $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "operation", content = "arguments", deny_unknown_fields)]
        pub enum Operation {
            $($name $(($ty))? $({$($field: $ft),*})?),*,
            Reanalyze {run: RunId, policy: analysis_engine::Policy},
            Viewer(ViewerSelection),
            Prepare(Box<Operation>),
            Terminal(crate::terminal::Command),
            SetAppearance(AppAppearance),
            SelectAgent(crate::control::Selection),
            AgentLocation { session: Uuid },
            SelectedEvidence(crate::control::Selection),
            #[cfg(feature = "test-support")]
            TestWait { millis: u64 },
        }
        impl Operation {
            pub fn from_work<W>(work: jobs::Command<W>) -> Result<Self, W> {
                Ok(match work {
                    $(jobs::Command::$name $(($binding))? $({$($field),*})? => Self::$name $(($binding))? $({$($field),*})?),*,
                    jobs::Command::Reanalyze {run, policy, ..} => Self::Reanalyze {run, policy},
                    jobs::Command::Extension(value) => return Err(value),
                })
            }
            pub(crate) fn into_work(self, cancel: Arc<AtomicBool>) -> anyhow::Result<jobs::Work> {
                Ok(match self {
                    $(Self::$name $(($binding))? $({$($field),*})? => jobs::Work::$name $(($binding))? $({$($field),*})?),*,
                    Self::Reanalyze {run, policy} => jobs::Work::Reanalyze {run, policy, cancel},
                    _ => anyhow::bail!("operation is handled by the engine"),
                })
            }
        }
    }
}
operations! {
    List {page: usize}, Load(value: ProjectId),
    Save {project: Box<Project>, expected_revision: Option<u64>},
    InspectStorage, DiagnosticPreview,
    DiagnosticExport {report: Box<diagnostics::Report>, path: PathBuf},
    ExportPreview {project: ProjectId, baseline: BaselineId},
    Export {preview: Box<ExportPreview>, path: PathBuf},
    ImportPreview(value: PathBuf), Import {preview: Box<ImportPreview>, path: PathBuf},
    History {project: ProjectId, page: usize}, ProjectStorage(value: ProjectId),
    InspectRun {project: ProjectId, run: RunId},
    DeletePreview {project: ProjectId, item: DeleteItem}, Delete(value: Box<DeletionPreview>),
    RepairPreview, CleanupTemporaries(value: Box<RepairPreview>), DeleteUnreferenced(value: Box<RepairPreview>),
    RuntimeStatus, NetworkStatus, SaveNetwork(value: NetworkSettings), SetupRuntime,
    Evidence(value: ProjectId), Capture {project: ProjectId, check: bool}, Discover(value: ProjectId),
    ExistingInventory(value: ProjectId), EnvironmentStatus(value: ProjectId),
    SaveEnvironment {project: Box<Project>, revision: u64, environment: EnvironmentId, settings: LocalLoginSettings},
    OpenLogin(value: ProjectId), ConfirmLogin(value: ProjectId), CloseLogin(value: ProjectId), ForgetLogin(value: ProjectId), ProbeEnvironment(value: ProjectId),
    CaptureInventory {project: ProjectId, preview: Option<Box<DiscoveryPreview>>, check: bool, resume: Option<RunId>},
    SaveReplacement {project: Box<Project>, expected_revision: u64},
    SaveReplacementEnvironment {project: Box<Project>, revision: u64, environment: EnvironmentId, settings: LocalLoginSettings},
    Activate {run: RunId, revision: u64, confirm_partial: bool},
    Mask {project: ProjectId, request: Box<analysis_engine::AnalysisRequest>}, Review(value: ReviewDecision), LoadAnalysis(value: AnalysisId),
    Approval {analysis: AnalysisId, selected: Vec<TargetKey>},
    Accept {preview: Box<ApprovalPreview>, context: ApprovalContext}, Restore {baseline: BaselineId, revision: u64},
    PreviewImages {project: ProjectId, preview: Box<DiscoveryPreview>},
}

impl Operation {
    pub(crate) fn capture_project(&self) -> Option<ProjectId> {
        match self {
            Self::Capture { project, .. }
            | Self::CaptureInventory { project, .. }
            | Self::PreviewImages { project, .. } => Some(*project),
            _ => None,
        }
    }
    pub fn freezes_project(&self) -> bool {
        matches!(
            self,
            Self::Discover(_)
                | Self::Capture { .. }
                | Self::CaptureInventory { .. }
                | Self::PreviewImages { .. }
                | Self::OpenLogin(_)
                | Self::ProbeEnvironment(_)
        )
    }
    pub fn needs_confirmation(&self) -> bool {
        if let Self::Terminal(command) = self {
            return command.writes();
        }
        matches!(
            self,
            Self::Activate { .. }
                | Self::Accept { .. }
                | Self::Restore { .. }
                | Self::Delete(_)
                | Self::CleanupTemporaries(_)
                | Self::DeleteUnreferenced(_)
                | Self::Import { .. }
                | Self::Export { .. }
                | Self::DiagnosticExport { .. }
                | Self::ForgetLogin(_)
                | Self::SaveReplacement { .. }
                | Self::SaveReplacementEnvironment { .. }
        )
    }
    pub fn mutates(&self) -> bool {
        if let Self::Terminal(command) = self {
            return command.writes();
        }
        !matches!(
            self,
            Self::List { .. }
                | Self::Load(_)
                | Self::InspectStorage
                | Self::DiagnosticPreview
                | Self::ExportPreview { .. }
                | Self::ImportPreview(_)
                | Self::History { .. }
                | Self::ProjectStorage(_)
                | Self::InspectRun { .. }
                | Self::DeletePreview { .. }
                | Self::RepairPreview
                | Self::RuntimeStatus
                | Self::NetworkStatus
                | Self::Evidence(_)
                | Self::ExistingInventory(_)
                | Self::EnvironmentStatus(_)
                | Self::Approval { .. }
                | Self::Viewer(_)
                | Self::Prepare(_)
                | Self::AgentLocation { .. }
                | Self::SelectedEvidence(_)
        )
    }
    pub fn requires_authority(&self) -> bool {
        self.mutates() || matches!(self, Self::Prepare(_) | Self::SelectAgent(_))
    }
    pub fn project(&self) -> Option<ProjectId> {
        match self {
            Self::Load(p)
            | Self::ProjectStorage(p)
            | Self::Evidence(p)
            | Self::Discover(p)
            | Self::ExistingInventory(p)
            | Self::EnvironmentStatus(p)
            | Self::OpenLogin(p)
            | Self::ConfirmLogin(p)
            | Self::CloseLogin(p)
            | Self::ForgetLogin(p)
            | Self::ProbeEnvironment(p) => Some(*p),
            Self::Save { project, .. }
            | Self::SaveEnvironment { project, .. }
            | Self::SaveReplacement { project, .. } => Some(project.id),
            Self::SaveReplacementEnvironment { project, .. } => Some(project.id),
            Self::Capture { project, .. }
            | Self::CaptureInventory { project, .. }
            | Self::PreviewImages { project, .. }
            | Self::History { project, .. }
            | Self::InspectRun { project, .. }
            | Self::DeletePreview { project, .. }
            | Self::ExportPreview { project, .. }
            | Self::Mask { project, .. } => Some(*project),
            Self::Viewer(v) => Some(v.project),
            Self::SelectedEvidence(v) => Some(v.project),
            Self::Terminal(c) => c.project(),
            Self::Accept { context, .. } => Some(context.project),
            Self::Delete(preview) => Some(preview.project),
            Self::Export { preview, .. } => Some(preview.project),
            _ => None,
        }
    }
    pub fn fingerprint(&self) -> anyhow::Result<String> {
        use sha2::{Digest, Sha256};
        Ok(hex::encode(Sha256::digest(serde_json::to_vec(self)?)))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ViewerSelection {
    pub project: ProjectId,
    pub run: Option<RunId>,
    pub tile: usize,
    pub key: Option<TargetKey>,
    pub analysis: Option<AnalysisId>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ViewerIdentity {
    pub project: ProjectId,
    pub run: RunId,
    pub key: TargetKey,
    pub analysis: Option<AnalysisId>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArtifactFile {
    pub artifact: Artifact,
    pub path: PathBuf,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViewerFrame {
    pub tile: usize,
    pub panes: Vec<Vec<ArtifactFile>>,
    pub width: u32,
    pub height: u32,
    pub request: Option<analysis_engine::AnalysisRequest>,
    pub analysis: Option<AnalysisId>,
    pub identity: ViewerIdentity,
    pub before_environment: Option<String>,
    pub before_capture: Option<CaptureEvidence>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Readiness {
    pub status: product::RuntimeState,
    pub setup_available: bool,
    pub message: String,
    pub identity: Option<RuntimeIdentity>,
    pub scope: Option<Scope>,
}

fn readiness_message(message: &str) -> anyhow::Result<&'static str> {
    let known = [
        product::unsupported_runtime_message(),
        "This build's browser trust configuration could not be validated. Browser setup is unavailable; existing evidence is unchanged.",
        "This build is incomplete: automatic browser setup has no trusted distribution configured. Existing evidence is unchanged.",
        "The private browser will be installed automatically on first use.",
        "Browser setup will start on first use. This configured development runtime is restricted to owned loopback fixtures, not public websites.",
        "Browser setup will start on first use using configured public trust. Release and platform qualification remain separate requirements.",
        "Private browser is ready.",
        "Private browser verified. Development runtime: owned loopback fixtures only; public-site qualification is pending.",
        "Private browser verified using configured public trust. External release and platform qualification remain required.",
        recovery::Problem::Browser(product::RuntimeState::UpdateRequired).message(),
        recovery::Problem::Browser(product::RuntimeState::RepairRequired).message(),
    ];
    known
        .into_iter()
        .find(|v| *v == message)
        .ok_or_else(|| anyhow::anyhow!("Unknown versioned browser status message"))
}

macro_rules! outcomes {
    ($($name:ident $(($binding:ident: $ty:ty))? $({$($field:ident: $ft:ty),* $(,)?})?),* $(,)?) => {
        #[derive(Debug, Clone, Serialize, Deserialize)]
        #[serde(tag = "result", content = "value", deny_unknown_fields)]
        pub enum Outcome { $($name $(($ty))? $({$($field: $ft),*})?),*, RuntimeReadiness(Readiness), Viewer(Box<ViewerFrame>), Confirmation(Confirmation), Terminal(Box<crate::terminal::Output>), AgentLocation(Box<crate::control::Location>), AgentSelected {selection:crate::control::Selection}, UnverifiedCapture {message:String, prior:Box<Outcome>} }
        impl Outcome {
            pub fn into_reply<R>(self, viewer: impl FnOnce(Box<ViewerFrame>) -> anyhow::Result<R>) -> anyhow::Result<jobs::JobReply<R>> {
                Ok(match self {
                    $(Self::$name $(($binding))? $({$($field),*})? => jobs::JobReply::$name $(($binding))? $({$($field),*})?),*,
                    Self::RuntimeReadiness(r) => jobs::JobReply::RuntimeReadiness(product::RuntimeReadiness {
                        status:r.status, setup_available:r.setup_available, message:readiness_message(&r.message)?, identity:r.identity, scope:r.scope,
                    }),
                    Self::Viewer(frame) => jobs::JobReply::Extension(viewer(frame)?),
                    Self::Confirmation(_) => anyhow::bail!("Unexpected confirmation response"),
                    Self::Terminal(_) => anyhow::bail!("Terminal inspection response is not a GUI actor reply"),
                    Self::AgentLocation(_) => anyhow::bail!("Agent location is handled separately from GUI jobs"),
                    Self::AgentSelected {..} => anyhow::bail!("Agent selection is handled separately from GUI jobs"),
                    Self::UnverifiedCapture {message,..}=>jobs::JobReply::ProductFailure(message),
                })
            }
            pub(crate) fn from_reply(reply: jobs::Reply) -> Self {
                match reply {
                    $(jobs::Reply::$name $(($binding))? $({$($field),*})? => Self::$name $(($binding))? $({$($field),*})?),*,
                    jobs::Reply::RuntimeReadiness(r) => Self::RuntimeReadiness(Readiness {status:r.status, setup_available:r.setup_available, message:r.message.into(), identity:r.identity, scope:r.scope}),
                    jobs::Reply::Extension(never) => match never {},
                }
            }
            pub fn failed(&self) -> bool { matches!(self, Self::Failure(_) | Self::ProductFailure(_) | Self::ActionRequired(_) | Self::UnverifiedCapture {..}) }
        }
    }
}
outcomes! {
    Projects(value: ProjectPage), Project(value: Box<ProjectRecord>), Storage(value: StorageSummary), DiagnosticPreview(value: Box<diagnostics::Report>), DiagnosticExported,
    ExportPreview(value: Box<ExportPreview>), Exported, ImportPreview(value: Box<ImportPreview>), Imported(value: Box<ProjectRecord>),
    History {history: Box<HistoryPage>}, ProjectStorage(value: ProjectStorage), DeletePreview(value: Box<DeletionPreview>), Deleted,
    RepairPreview(value: Box<RepairPreview>), CleanedTemporaries, Failure(value: jobs::StorageFailure), ProductFailure(value: String), ActionRequired(value: recovery::Problem),
    Cancelled, Runtime(value: String), Network {settings: NetworkSettings, message: String}, Evidence(value: Box<ProjectEvidence>), Captured(value: Box<ProjectEvidence>),
    InspectedRun {project: ProjectId, run: RunId, evidence: Box<ProjectEvidence>}, Analysis(value: Box<ComparisonEvidence>), Approval(value: Box<ApprovalPreview>),
    SavedReview(value: Vec<ReviewDecision>), SavedMask, Discovery(value: Box<DiscoveryPreview>), Environment(value: Box<auth::EnvironmentStatus>),
    Progress {message: String, completed: u32, total: Option<u32>}, CaptureStatus(value: task::CaptureStatus),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Confirmation {
    pub id: Uuid,
    pub engine: Uuid,
    pub epoch: u64,
    #[serde(default)]
    pub owner: Option<crate::control::Owner>,
    pub fingerprint: String,
    pub revision: (u64, u64),
    #[serde(default)]
    pub preview: Option<serde_json::Value>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Submit {
    pub request: Uuid,
    pub engine: Uuid,
    pub epoch: u64,
    pub operation: Operation,
    pub confirmation: Option<Uuid>,
    pub expected_revision: Option<u64>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Queued,
    Running,
    Succeeded,
    Failed,
    Paused,
    Cancelled,
    Interrupted,
}
impl JobState {
    pub fn terminal(self) -> bool {
        !matches!(self, Self::Queued | Self::Running)
    }
}
impl Job {
    /// Preserve structured service errors. Successful capture-shaped replies,
    /// however, must carry the executed run and agree with all known identities.
    pub(crate) fn capture_result_error(&self, outcome: &Outcome) -> Option<&'static str> {
        let project = self.submitted.operation.capture_project()?;
        if outcome.failed() || matches!(outcome, Outcome::Cancelled) {
            return None;
        }
        let Outcome::Captured(evidence) = outcome else {
            return Some("capture_result_missing: no executed-run receipt was returned");
        };
        let Some(record) = &evidence.latest else {
            return Some("capture_result_missing: executed run is absent");
        };
        if record.run.project != project
            || self
                .capture
                .as_ref()
                .is_some_and(|c| c.project != project || c.run != record.run.id)
            || matches!(&self.submitted.operation,Operation::CaptureInventory {resume:Some(id),..} if *id!=record.run.id)
        {
            return Some(
                "capture_result_mismatch: returned run does not match this capture request",
            );
        }
        let intent_matches = match &self.submitted.operation {
            Operation::PreviewImages { .. } => {
                matches!(record.run.intent, RunIntent::RepresentativePreview)
            }
            Operation::Capture { check, .. }
            | Operation::CaptureInventory {
                check,
                resume: None,
                ..
            } => {
                if *check {
                    matches!(record.run.intent, RunIntent::Check { .. })
                } else {
                    matches!(record.run.intent, RunIntent::NewBaseline)
                }
            }
            _ => true, // A resume retains the original run's intent, not a new flag.
        };
        if !intent_matches
            || evidence
                .comparison
                .as_ref()
                .is_some_and(|a| a.revision.run != record.run.id)
        {
            return Some(
                "capture_result_mismatch: capture intent or analysis belongs to another operation",
            );
        }
        None
    }
    /// The engine task can finish while a persisted capture is paused, failed or
    /// cancelled. Never turn a successfully delivered evidence DTO into a pass.
    pub(crate) fn terminal_state(&self, outcome: &Outcome) -> JobState {
        if matches!(outcome, Outcome::Cancelled) {
            return JobState::Cancelled;
        }
        if outcome.failed() {
            return JobState::Failed;
        }
        if self.submitted.operation.capture_project().is_some() {
            if self.capture_result_error(outcome).is_some() {
                return JobState::Failed;
            }
            if let Outcome::Captured(evidence) = outcome {
                return capture_terminal_state(evidence.latest.as_ref().map(|r| r.run.state));
            }
        }
        JobState::Succeeded
    }
}
fn capture_terminal_state(state: Option<regressee_core::domain::JobState>) -> JobState {
    match state {
        Some(regressee_core::domain::JobState::Completed) => JobState::Succeeded,
        Some(regressee_core::domain::JobState::Paused) => JobState::Paused,
        Some(regressee_core::domain::JobState::Cancelled) => JobState::Cancelled,
        Some(regressee_core::domain::JobState::Interrupted) => JobState::Interrupted,
        _ => JobState::Failed,
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Job {
    pub id: Uuid,
    pub submitted: Submit,
    #[serde(default)]
    pub owner: Option<crate::control::Owner>,
    pub state: JobState,
    pub frozen: Option<ProjectRecord>,
    #[serde(default)]
    pub frozen_revision: Option<u64>,
    pub progress: Option<Outcome>,
    #[serde(default)]
    pub capture: Option<task::CaptureStatus>,
    pub outcome: Option<Outcome>,
    pub sequence: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    pub sequence: u64,
    pub job: Uuid,
    pub state: JobState,
    pub progress: Option<Outcome>,
    pub changed: bool,
    pub project: Option<ProjectRecord>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: Uuid,
    pub state: JobState,
    pub sequence: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub engine: Uuid,
    pub epoch: u64,
    pub sequence: u64,
    pub jobs: Vec<JobInfo>,
    pub total_jobs: usize,
    pub project_revisions: Vec<(ProjectId, u64)>,
    pub draining: bool,
    pub login_open: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "command", content = "arguments", deny_unknown_fields)]
pub enum Request {
    RegisterFrontend {
        capability: crate::control::FrontendCapability,
    },
    Authorized {
        credential: crate::control::Credential,
        epoch: u64,
        request: Box<Request>,
    },
    Control(crate::control::Command),
    Hello {
        version: u32,
    },
    Status,
    Stop,
    Snapshot,
    Events {
        engine: Uuid,
        after: u64,
    },
    Submit(Box<Submit>),
    Job {
        id: Uuid,
    },
    Jobs {
        offset: usize,
        limit: usize,
    },
    Cancel {
        id: Uuid,
    },
    Pause {
        id: Uuid,
    },
    Acknowledge {
        id: Uuid,
    },
    AdvanceEpoch {
        expected: u64,
    },
    Preferences,
    SetAppearance(AppAppearance),
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "response", content = "value", deny_unknown_fields)]
pub enum Response {
    Control(crate::control::Snapshot),
    Credential(crate::control::Credential),
    Hello {
        version: u32,
        engine: Uuid,
    },
    Status(Snapshot),
    Job(Box<Job>),
    Jobs {
        items: Vec<JobInfo>,
        total: usize,
        next_offset: Option<usize>,
    },
    Events {
        sequence: u64,
        events: Vec<Event>,
        resync: bool,
    },
    Preferences(AppAppearance),
    Ok,
    Error {
        code: String,
        message: String,
    },
}
impl Response {
    pub fn error(code: &str, message: impl Into<String>) -> Self {
        Self::Error {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
#[path = "../tests/support/approval_fixture.rs"]
mod receipt_fixture;
#[cfg(test)]
mod tests {
    use super::receipt_fixture as fixture;
    use super::*;

    fn receipt(operation: Operation) -> Job {
        let id = Uuid::new_v4();
        Job {
            id,
            owner: None,
            submitted: Submit {
                request: id,
                engine: Uuid::new_v4(),
                epoch: 0,
                operation,
                confirmation: None,
                expected_revision: None,
            },
            state: JobState::Running,
            frozen: None,
            frozen_revision: None,
            progress: None,
            capture: None,
            outcome: None,
            sequence: 0,
        }
    }

    #[test]
    fn representative_preview_failure_is_not_a_successful_receipt() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let (store, project, _, _, _) = fixture::fixture(root.path());
        let mut evidence = store.project_evidence(project)?;
        evidence.latest.as_mut().unwrap().run.state =
            regressee_core::domain::JobState::CompletedWithErrors;
        let saved = store.load_project(project)?;
        let preview = DiscoveryPreview {
            project_revision: saved.revision,
            settings: saved.project,
            inventory: evidence.latest.as_ref().unwrap().run.inventory.clone(),
            aliases: vec![],
            notes: vec![],
            visited: 0,
        };
        assert_eq!(
            receipt(Operation::PreviewImages {
                project,
                preview: Box::new(preview)
            })
            .terminal_state(&Outcome::Evidence(Box::new(evidence))),
            JobState::Failed
        );
        Ok(())
    }

    #[test]
    fn resumed_run_cannot_use_a_different_runs_successful_evidence() -> anyhow::Result<()> {
        let root = tempfile::tempdir()?;
        let (store, project, _, _, _) = fixture::fixture(root.path());
        let evidence = store.project_evidence(project)?;
        let other = RunId::new();
        assert_ne!(evidence.latest.as_ref().unwrap().run.id, other);
        assert_eq!(
            receipt(Operation::CaptureInventory {
                project,
                preview: None,
                check: false,
                resume: Some(other)
            })
            .terminal_state(&Outcome::Evidence(Box::new(evidence))),
            JobState::Failed
        );
        Ok(())
    }

    fn save_run(
        store: &mut Store,
        template: &RunRecord,
        state: regressee_core::domain::JobState,
    ) -> anyhow::Result<RunRecord> {
        save_run_with_intent(store, template, state, RunIntent::NewBaseline)
    }
    fn save_run_with_intent(
        store: &mut Store,
        template: &RunRecord,
        state: regressee_core::domain::JobState,
        intent: RunIntent,
    ) -> anyhow::Result<RunRecord> {
        use regressee_core::domain::JobState as Capture;
        let saved = store.load_project(template.run.project)?;
        let mut record = template.clone();
        record.run.id = RunId::new();
        record.run.intent = intent;
        record.run.state = Capture::Running;
        record.run.finished_at = None;
        record.capture = None;
        record.captures.clear();
        record.queue = Some(QueueSettings {
            discovery: saved.project.discovery,
            project_revision: saved.revision,
            resumed_at: vec![],
        });
        for target in &mut record.run.targets {
            target.outcome = TargetOutcome::Pending;
            target.final_url = None;
        }
        store.begin_run(&record)?;
        record.run.state = state;
        if state == Capture::Completed {
            record.captures = template.captures.clone();
            record.capture = template.capture.clone();
            record.run.targets = template.run.targets.clone();
        }
        if state == Capture::CompletedWithErrors {
            for target in &mut record.run.targets {
                target.outcome = TargetOutcome::Failed {
                    code: CaptureError::Timeout,
                };
            }
        }
        if state == Capture::Cancelled {
            for target in &mut record.run.targets {
                target.outcome = TargetOutcome::Cancelled;
            }
        }
        if state == Capture::Paused {
            store.checkpoint_run(&record)?;
        } else {
            record.run.finished_at = Some(300);
            store.finish_run(&record)?;
        }
        store.load_run(record.run.id)
    }

    #[test]
    fn paused_check_receipt_keeps_its_reference_even_without_analysis() -> anyhow::Result<()> {
        use regressee_core::domain::JobState as Capture;
        let root = tempfile::tempdir()?;
        let (mut store, project, _, _, _) = fixture::fixture(root.path());
        let template = store.project_evidence(project)?.latest.unwrap();
        let original = match template.run.intent {
            RunIntent::Check { baseline } => baseline,
            _ => unreachable!(),
        };
        let paused = save_run_with_intent(
            &mut store,
            &template,
            Capture::Paused,
            RunIntent::Check { baseline: original },
        )?;
        let newer = save_run(&mut store, &template, Capture::Completed)?;
        let revision = store.load_project(project)?.revision;
        let changed = store.activate_baseline(newer.run.id, revision)?;
        assert_ne!(changed.project.active_baseline, Some(original));
        let outcome = Outcome::Captured(Box::new(
            store.project_evidence_for_run(project, paused.run.id)?,
        ));
        let Outcome::Captured(e) = &outcome else {
            unreachable!()
        };
        assert_eq!(e.latest.as_ref().unwrap().run.id, paused.run.id);
        assert_eq!(e.baseline.as_ref().unwrap().id, original);
        assert!(e.comparison.is_none());
        assert!(
            store.inspect_run(project, paused.run.id).is_err(),
            "Historical analysis inspection intentionally remains stricter than a capture receipt"
        );
        assert_eq!(
            receipt(Operation::CaptureInventory {
                project,
                preview: None,
                check: true,
                resume: Some(paused.run.id)
            })
            .terminal_state(&outcome),
            JobState::Paused
        );
        Ok(())
    }

    #[test]
    fn non_newest_capture_receipts_bind_saved_run_identity_and_terminal_state() -> anyhow::Result<()>
    {
        use regressee_core::domain::JobState as Capture;
        for (state, expected) in [
            (Capture::Paused, JobState::Paused),
            (Capture::CompletedWithErrors, JobState::Failed),
            (Capture::Cancelled, JobState::Cancelled),
            (Capture::Interrupted, JobState::Interrupted),
            (Capture::Completed, JobState::Succeeded),
        ] {
            for newer_state in [Capture::Completed, Capture::CompletedWithErrors] {
                let root = tempfile::tempdir()?;
                let (mut store, project, _, _, _) = fixture::fixture(root.path());
                let template = store.project_evidence(project)?.latest.unwrap();
                let executed = save_run(&mut store, &template, state)?;
                let newer = save_run(&mut store, &template, newer_state)?;
                assert_eq!(
                    store.project_evidence(project)?.latest.unwrap().run.id,
                    newer.run.id
                );
                let outcome = Outcome::Captured(Box::new(
                    store.project_evidence_for_run(project, executed.run.id)?,
                ));
                let job = receipt(Operation::CaptureInventory {
                    project,
                    preview: None,
                    check: false,
                    resume: Some(executed.run.id),
                });
                assert_eq!(
                    job.terminal_state(&outcome),
                    expected,
                    "Executed {state:?}, newer {newer_state:?}"
                );
                let Outcome::Captured(evidence) = &outcome else {
                    panic!("Capture reply must be distinct from workspace refresh")
                };
                assert_eq!(evidence.latest.as_ref().unwrap().run.id, executed.run.id);
                assert_eq!(evidence.latest.as_ref().unwrap().run.state, state);
                assert!(
                    evidence.comparison.is_none(),
                    "Do not borrow the newer run's analysis"
                );
                let wrong = Outcome::Captured(Box::new(store.project_evidence(project)?));
                assert_eq!(job.terminal_state(&wrong), JobState::Failed);
                assert!(
                    job.capture_result_error(&wrong)
                        .unwrap()
                        .contains("mismatch")
                );
            }
        }
        Ok(())
    }

    #[test]
    fn every_capture_operation_classifies_its_actual_run_and_preserves_errors() -> anyhow::Result<()>
    {
        use regressee_core::domain::JobState as Capture;
        let root = tempfile::tempdir()?;
        let (store, project, _, _, _) = fixture::fixture(root.path());
        let saved = store.load_project(project)?;
        let original = store.project_evidence(project)?;
        let preview = DiscoveryPreview {
            project_revision: saved.revision,
            settings: saved.project,
            inventory: original.latest.as_ref().unwrap().run.inventory.clone(),
            aliases: vec![],
            notes: vec![],
            visited: 0,
        };
        for operation in [
            Operation::Capture {
                project,
                check: true,
            },
            Operation::CaptureInventory {
                project,
                preview: None,
                check: true,
                resume: None,
            },
            Operation::PreviewImages {
                project,
                preview: Box::new(preview),
            },
        ] {
            let mut original = original.clone();
            if matches!(operation, Operation::PreviewImages { .. }) {
                original.latest.as_mut().unwrap().run.intent = RunIntent::RepresentativePreview;
                original.comparison = None;
            }
            let job = receipt(operation);
            for (state, expected) in [
                (Capture::Completed, JobState::Succeeded),
                (Capture::CompletedWithErrors, JobState::Failed),
                (Capture::Cancelled, JobState::Cancelled),
                (Capture::Paused, JobState::Paused),
                (Capture::Interrupted, JobState::Interrupted),
                (Capture::Running, JobState::Failed),
            ] {
                let mut evidence = original.clone();
                evidence.latest.as_mut().unwrap().run.state = state;
                assert_eq!(
                    job.terminal_state(&Outcome::Captured(Box::new(evidence))),
                    expected
                );
            }
            // A changed comparison in the real fixture is not an operational failure.
            assert_eq!(
                job.terminal_state(&Outcome::Captured(Box::new(original.clone()))),
                JobState::Succeeded
            );
            for outcome in [
                Outcome::ActionRequired(recovery::Problem::Worker(
                    regressee_runtime::protocol::FailureCode::NeedsLogin,
                )),
                Outcome::ProductFailure("Storage failure, not cancellation".into()),
            ] {
                assert_eq!(job.capture_result_error(&outcome), None);
                assert_eq!(job.terminal_state(&outcome), JobState::Failed);
                assert!(matches!(
                    outcome.into_reply::<std::convert::Infallible>(|_| unreachable!())?,
                    jobs::Reply::ActionRequired(_) | jobs::Reply::ProductFailure(_)
                ));
            }
            assert_eq!(job.terminal_state(&Outcome::Cancelled), JobState::Cancelled);
            assert_eq!(
                job.terminal_state(&Outcome::Captured(Box::default())),
                JobState::Failed
            );
            let mut foreign = original.clone();
            foreign.latest.as_mut().unwrap().run.project = ProjectId::new();
            assert_eq!(
                job.terminal_state(&Outcome::Captured(Box::new(foreign))),
                JobState::Failed
            );
            let mut wrong_intent = original.clone();
            wrong_intent.latest.as_mut().unwrap().run.intent = RunIntent::NewBaseline;
            assert_eq!(
                job.terminal_state(&Outcome::Captured(Box::new(wrong_intent))),
                JobState::Failed
            );
            if let Some(analysis) = &original.comparison {
                let mut wrong_analysis = original.clone();
                let mut analysis = analysis.clone();
                analysis.revision.run = RunId::new();
                wrong_analysis.comparison = Some(analysis);
                assert_eq!(
                    job.terminal_state(&Outcome::Captured(Box::new(wrong_analysis))),
                    JobState::Failed
                );
            }
            let mut mismatch = job.clone();
            mismatch.capture = Some(task::CaptureStatus::from_run(
                &original.latest.as_ref().unwrap().run,
            ));
            mismatch.capture.as_mut().unwrap().run = RunId::new();
            assert_eq!(
                mismatch.terminal_state(&Outcome::Captured(Box::new(original.clone()))),
                JobState::Failed
            );
        }
        Ok(())
    }
    #[test]
    fn delivering_evidence_does_not_relabel_incomplete_cancelled_or_paused_capture_as_success() {
        use regressee_core::domain::JobState as Capture;
        for (state, expected) in [
            (Capture::Completed, JobState::Succeeded),
            (Capture::CompletedWithErrors, JobState::Failed),
            (Capture::Paused, JobState::Paused),
            (Capture::Cancelled, JobState::Cancelled),
            (Capture::Interrupted, JobState::Interrupted),
            (Capture::Running, JobState::Failed),
        ] {
            assert_eq!(capture_terminal_state(Some(state)), expected);
        }
        assert_eq!(capture_terminal_state(None), JobState::Failed);
    }
}
