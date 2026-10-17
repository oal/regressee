// SPDX-License-Identifier: MPL-2.0
//! Public control DTOs. Credentials travel only in a private request envelope,
//! never in jobs, events, reports or diagnostic output.
use regressee_core::{domain::*, evidence::ProjectEvidence};
use regressee_storage::ProjectRecord;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const DEFAULT_LEASE: u64 = 90;
pub const MIN_LEASE: u64 = 10;
pub const MAX_LEASE: u64 = 300;
pub const FRONTEND_LEASE: u64 = 15;

/// Private per-incarnation GUI bootstrap. Distributed through an owner-private
/// file, never an anonymous IPC response, public status, job or report.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrontendCapability {
    pub engine: Uuid,
    pub secret: Uuid,
}
impl std::fmt::Debug for FrontendCapability {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FrontendCapability")
            .field("engine", &self.engine)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Owner {
    Agent(Uuid),
    Frontend(Uuid),
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Credential {
    pub engine: Uuid,
    pub owner: Owner,
    pub secret: Uuid,
}
impl std::fmt::Debug for Credential {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Credential")
            .field("engine", &self.engine)
            .field("owner", &self.owner)
            .finish_non_exhaustive()
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    Active,
    Human,
    Ended,
    Expired,
    Revoked,
}
impl Status {
    pub fn live(self) -> bool {
        matches!(self, Self::Active | Self::Human)
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum View {
    #[default]
    Compare,
    Before,
    After,
    Difference,
    Overlay,
    Split,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Selection {
    pub project: ProjectId,
    pub run: Option<RunId>,
    pub analysis: Option<AnalysisId>,
    pub key: Option<TargetKey>,
    #[serde(default)]
    pub view: View,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Location {
    pub selection: Selection,
    pub project: ProjectRecord,
    pub evidence: Option<ProjectEvidence>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Session {
    pub id: Uuid,
    pub name: String,
    pub status: Status,
    pub remaining_seconds: u64,
    pub follow: Option<Uuid>,
    pub selection: Option<Selection>,
    pub selection_revision: u64,
    pub handoff: Option<String>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Snapshot {
    pub engine: Uuid,
    pub epoch: u64,
    pub controller: Option<Uuid>,
    pub sessions: Vec<Session>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "arguments", deny_unknown_fields)]
pub enum Command {
    Start { name: String, seconds: u64 },
    List,
    Show { session: Uuid },
    Renew { seconds: u64 },
    End,
    Handoff { message: String },
    RegisterFrontend,
    Heartbeat,
    Follow { session: Uuid },
    Takeover,
    Return,
    Detach,
}

pub fn validate(command: &Command) -> anyhow::Result<()> {
    use anyhow::ensure;
    match command {
        Command::Start { name, seconds } => {
            ensure!(
                !name.trim().is_empty() && name.len() <= 80 && !name.chars().any(char::is_control),
                "invalid_request: agent name must be 1–80 bytes without control characters"
            );
            lease(*seconds)?;
        }
        Command::Renew { seconds } => lease(*seconds)?,
        Command::Handoff { message } => ensure!(
            !message.trim().is_empty()
                && message.len() <= 1024
                && !message.chars().any(char::is_control),
            "invalid_request: handoff message must be 1–1024 bytes without control characters"
        ),
        _ => {}
    }
    Ok(())
}
pub fn lease(seconds: u64) -> anyhow::Result<()> {
    anyhow::ensure!(
        (MIN_LEASE..=MAX_LEASE).contains(&seconds),
        "invalid_request: lease must be 10–300 seconds"
    );
    Ok(())
}
