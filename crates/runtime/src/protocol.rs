// SPDX-License-Identifier: MPL-2.0
//! Product protocol v3: independent nonterminal capture channels, then completion.
use anyhow::{Result, ensure};
use regressee_core::domain::*;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Read};
pub const PROTOCOL_VERSION: u32 = 3;
pub const WORKER_VERSION: &str = "6.0.0";
pub const MAX_FRAME: usize = 1024 * 1024;
pub const MAX_LINKS: usize = 2000;
pub const MAX_REQUEST_ID: u64 = 9_007_199_254_740_991; // Node safe integer range.
/// Navigation, bounded common readiness, DOM sampling (one stage), screenshot
/// preparation + pixel sampling (two stages), and supervisor/cleanup slack.
/// The worker's watchdog uses the same sum with five rather than ten seconds.
pub fn capture_request_budget(
    profile: &CaptureProfile,
    plan: &CapturePlan,
) -> Result<std::time::Duration> {
    profile.validate()?;
    plan.validate()?;
    let readiness = u64::from(profile.stabilization.readiness_ms);
    let stages =
        if plan.mode.dom_snapshots() { 1 } else { 0 } + if plan.mode.screenshots() { 2 } else { 0 };
    Ok(std::time::Duration::from_millis(
        u64::from(profile.stabilization.navigation_ms)
            + readiness.min(3000)
            + stages * readiness
            + 10_000,
    ))
}
/// Optional private diagnostic sidecar, never portable evidence or kill authority.
/// Exact enums prevent a worker from injecting site strings into diagnostic logs.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureTrace {
    pub event: TraceEvent,
    pub elapsed_ms: u32,
    pub request: Option<u32>,
    pub kind: Option<TraceKind>,
    pub replacing: Option<bool>,
    pub known_replacement_source: Option<bool>,
    pub failure: Option<TraceFailure>,
    pub attributed_replacement: Option<bool>,
    pub request_start_ms: Option<f64>,
    pub status: Option<u16>,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TraceEvent {
    Request,
    Response,
    Failed,
    ReplacementStart,
    ReplacementEnd,
}
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TraceKind {
    Document,
    Stylesheet,
    Image,
    Font,
    Script,
    Xhr,
    Fetch,
    Other,
}
#[derive(Serialize, Deserialize)]
pub enum TraceFailure {
    Aborted,
    Other,
}
pub fn decode_capture_trace(bytes: &[u8]) -> Result<Vec<CaptureTrace>> {
    ensure!(bytes.len() <= 65536, "private capture trace exceeds budget");
    let trace: Vec<CaptureTrace> = serde_json::from_slice(bytes)
        .map_err(|_| anyhow::anyhow!("private capture trace schema invalid"))?;
    ensure!(
        trace.len() <= 128
            && trace.iter().all(|e| e.elapsed_ms <= 300000
                && e.request.is_none_or(|r| (1..=128).contains(&r))
                && e.status.is_none_or(|s| (100..=599).contains(&s))
                && e.request_start_ms
                    .is_none_or(|s| s.is_finite() && (-1.0..=300000.0).contains(&s))),
        "private capture trace invalid"
    );
    Ok(trace)
}
pub fn retain_capture_trace(
    directory: &std::path::Path,
    source: &std::path::Path,
    job: RunId,
    token: uuid::Uuid,
) -> Result<()> {
    use std::io::Write;
    crate::distribution::safe_directory(directory)?;
    let meta = std::fs::symlink_metadata(directory)?;
    ensure!(
        directory.is_absolute() && meta.is_dir(),
        "private capture trace requires existing physical directory"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        ensure!(
            meta.uid() == unsafe { libc::geteuid() } && meta.mode() & 0o7777 == 0o700,
            "private capture trace directory must be owner-only"
        );
    }
    let meta = std::fs::symlink_metadata(source)?;
    ensure!(
        meta.is_file() && regressee_core::platform::ordinary(&meta) && meta.len() <= 65536,
        "unsafe private capture trace"
    );
    let trace = decode_capture_trace(&std::fs::read(source)?)?;
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(directory.join(format!("{job}-{token}.json")))?;
    file.write_all(&serde_json::to_vec(&trace)?)?;
    file.sync_all()?;
    Ok(())
}
/// Private stdio only. Not an evidence/domain/export type; Debug is redacted.
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct SecretPayload(pub String);
impl Clone for SecretPayload {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}
impl std::fmt::Debug for SecretPayload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("[REDACTED session]")
    }
}
impl Drop for SecretPayload {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        self.0.zeroize();
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Request {
    pub version: u32,
    pub request_id: u64,
    pub job: RunId,
    pub command: Command,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum Command {
    Handshake,
    /// Explicit installer pixel smoke; ordinary handshake never needs pixels.
    InstallSmoke,
    SetupLogin {
        session: SessionReference,
    },
    OpenLogin {
        session: SessionReference,
        address: String,
        representative: String,
        origins: Vec<String>,
        identity_origins: Vec<String>,
    },
    ConfirmLogin {
        session: SessionReference,
    },
    InstallSession {
        session: SessionReference,
        origins: Vec<String>,
        material: SecretPayload,
    },
    DiscoverAuthenticated {
        address: String,
        section: Option<String>,
        session: SessionReference,
    },
    DiscoverPage {
        address: String,
        section: Option<String>,
    },
    FetchRobots {
        address: String,
    },
    CaptureTarget {
        key: TargetKey,
        address: String,
        profile: Box<CaptureProfile>,
        session: Option<SessionReference>,
        plan: CapturePlan,
    },
    Cancel,
    Shutdown,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Event {
    pub version: u32,
    pub request_id: u64,
    pub job: RunId,
    pub event: EventKind,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "payload", deny_unknown_fields)]
pub enum EventKind {
    Acknowledged,
    Ready {
        worker_version: String,
        browser_version: String,
    },
    Progress {
        completed: u32,
        total: Option<u32>,
    },
    LinksFound {
        addresses: Vec<String>,
        truncated: bool,
        final_url: String,
    },
    RobotsFound {
        text: String,
        status: u16,
    },
    RedirectFound {
        address: String,
    },
    ChannelCaptured {
        key: TargetKey,
        capture: CapturedChannel,
    },
    ChannelFailed {
        key: TargetKey,
        kind: CheckKind,
        code: FailureCode,
        retryable: bool,
    },
    ChannelWarning {
        key: TargetKey,
        kind: CheckKind,
        code: WarningCode,
    },
    CaptureCompleted {
        key: TargetKey,
    },
    Warning {
        code: WarningCode,
    },
    RequiresLogin,
    LoginOpened,
    SessionReady {
        material: SecretPayload,
    },
    Completed,
    Cancelled,
    Failure {
        code: FailureCode,
        retryable: bool,
    },
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", deny_unknown_fields)]
pub enum CapturedChannel {
    Screenshot {
        output_token: uuid::Uuid,
        artifact: Artifact,
        width: u32,
        height: u32,
    },
    DomSnapshot {
        output_token: uuid::Uuid,
        artifact: Artifact,
        policy: regressee_core::dom_snapshot::DomSnapshotPolicy,
    },
}
impl CapturedChannel {
    pub fn kind(&self) -> CheckKind {
        match self {
            Self::Screenshot { .. } => CheckKind::Screenshot,
            Self::DomSnapshot { .. } => CheckKind::DomSnapshot,
        }
    }
    pub fn output_token(&self) -> uuid::Uuid {
        match self {
            Self::Screenshot { output_token, .. } | Self::DomSnapshot { output_token, .. } => {
                *output_token
            }
        }
    }
    pub fn artifact(&self) -> &Artifact {
        match self {
            Self::Screenshot { artifact, .. } | Self::DomSnapshot { artifact, .. } => artifact,
        }
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(!self.output_token().is_nil(), "invalid output token");
        let artifact = self.artifact();
        match self {
            Self::Screenshot { width, height, .. } => ensure!(
                artifact.kind == ArtifactKind::Screenshot
                    && artifact.bytes > 0
                    && artifact.bytes <= 128 * 1024 * 1024
                    && (1..=4096).contains(width)
                    && (1..=32768).contains(height)
                    && u64::from(*width) * u64::from(*height) <= 64_000_000,
                "invalid screenshot channel limits"
            ),
            Self::DomSnapshot { policy, .. } => {
                policy.validate()?;
                ensure!(
                    artifact.kind == ArtifactKind::DomSnapshot
                        && artifact.bytes
                            > regressee_core::dom_snapshot::DOM_CANONICAL_HEADER.len() as u64
                        && artifact.bytes <= regressee_core::dom_snapshot::MAX_DOM_BYTES as u64,
                    "invalid DOM channel limits"
                );
            }
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum WarningCode {
    ReadinessBudget,
    Unstable,
    ReplacementGeometryChanged,
    ReplacementGeometryNotMeasured,
    ReplacementSourceDependent,
    ReplacementUnsupportedMedia,
    FontsIncomplete,
    ImagesIncomplete,
    ResourceFailed,
    EmbeddedContentBlocked,
    IframeDocumentsExcluded,
    ClosedShadowRootsExcluded,
    NearbySamplesNotAtomic,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum FailureCode {
    UnsupportedVersion,
    InvalidRequest,
    NeedsLogin,
    Timeout,
    CaptureFailed,
    WorkerStopped,
    PayloadLimit,
    Cancelled,
    AccessDenied,
    NotFound,
    RedirectOutsideScope,
    ImageLimit,
    DomLimit,
    DomSerialization,
    RateLimited,
    ServerError,
}
pub fn encode_request(request: &Request) -> Result<Vec<u8>> {
    ensure!(
        request.version == PROTOCOL_VERSION
            && (1..=MAX_REQUEST_ID).contains(&request.request_id)
            && !request.job.0.is_nil(),
        "invalid request envelope"
    );
    match &request.command {
        Command::DiscoverPage { address, section } => {
            validate_wire_address(address)?;
            ensure!(
                section
                    .as_ref()
                    .is_none_or(|s| s.starts_with('/') && s.len() <= 4096),
                "invalid discovery section"
            );
        }
        Command::FetchRobots { address } => validate_wire_address(address)?,
        Command::SetupLogin { session } => validate_wire_session(session)?,
        Command::OpenLogin {
            session,
            address,
            representative,
            origins,
            identity_origins,
        } => {
            validate_wire_session(session)?;
            validate_wire_address(address)?;
            validate_wire_address(representative)?;
            validate_origins(origins)?;
            validate_origins(identity_origins)?;
        }
        Command::ConfirmLogin { session } => validate_wire_session(session)?,
        Command::InstallSession {
            session,
            origins,
            material,
        } => {
            validate_wire_session(session)?;
            validate_origins(origins)?;
            ensure!(
                material.0.len() <= 512 * 1024,
                "session transfer exceeds private limit"
            );
        }
        Command::DiscoverAuthenticated {
            address,
            section: _,
            session,
        } => {
            validate_wire_session(session)?;
            validate_wire_address(address)?;
        }
        Command::CaptureTarget {
            address,
            profile,
            key,
            session,
            plan,
        } => {
            plan.validate()?;
            if let Some(session) = session {
                validate_wire_session(session)?;
            }
            validate_wire_address(address)?;
            profile.validate()?;
            ensure!(
                key.profile == profile.id && !key.page.0.is_nil(),
                "target/profile mismatch"
            );
            ensure!(
                profile.ready_for_capture(),
                "capture requires a pinned runtime profile"
            );
        }
        _ => (),
    }
    let mut bytes = zeroize::Zeroizing::new(serde_json::to_vec(request)?);
    ensure!(bytes.len() < MAX_FRAME, "request exceeds frame limit");
    bytes.push(b'\n');
    Ok(std::mem::take(&mut *bytes))
}
fn validate_wire_session(session: &SessionReference) -> Result<()> {
    // A reference is three non-nil IDs, not credentials. Capture may explicitly
    // carry no reference (null); SetupLogin must always carry a valid one.
    ensure!(
        !session.id.0.is_nil() && !session.project.0.is_nil() && !session.environment.0.is_nil(),
        "invalid session reference: IDs must be non-nil"
    );
    Ok(())
}
fn validate_origins(origins: &[String]) -> Result<()> {
    ensure!(origins.len() <= 16, "too many session origins");
    for origin in origins {
        validate_wire_address(origin)?;
        let u = url::Url::parse(origin)?;
        ensure!(
            u.path() == "/" && u.query().is_none() && u.fragment().is_none(),
            "session scope must be exact origins"
        );
    }
    Ok(())
}
fn validate_wire_address(address: &str) -> Result<()> {
    // Parsing through Project also enforces credential/scheme/length policy.
    Project::draft("Wire validation".into(), address)?;
    Ok(())
}
/// Shared with the Phase 0 supervisor: bound allocation before JSON decoding,
/// require a complete newline, and reject frames at the transport boundary.
pub fn read_json_frame(reader: &mut impl BufRead) -> Result<Option<serde_json::Value>> {
    let mut bytes = Vec::new();
    let count = reader
        .take((MAX_FRAME + 1) as u64)
        .read_until(b'\n', &mut bytes)?;
    if count == 0 {
        return Ok(None);
    }
    ensure!(
        bytes.len() <= MAX_FRAME && bytes.ends_with(b"\n"),
        "worker frame exceeds limit or is incomplete"
    );
    Ok(Some(serde_json::from_slice(&bytes)?))
}
pub fn decode_event(value: serde_json::Value) -> Result<Event> {
    let event: Event = serde_json::from_value(value)
        .map_err(|_| anyhow::anyhow!("invalid worker event schema"))?;
    ensure!(
        event.version == PROTOCOL_VERSION
            && (1..=MAX_REQUEST_ID).contains(&event.request_id)
            && !event.job.0.is_nil(),
        "unsupported or invalid event envelope"
    );
    match &event.event {
        EventKind::SessionReady { material } => ensure!(
            material.0.len() <= 512 * 1024,
            "private session transfer exceeds limit"
        ),
        EventKind::LinksFound {
            addresses,
            final_url,
            ..
        } => {
            validate_wire_address(final_url)?;
            ensure!(addresses.len() <= MAX_LINKS, "too many links in one event");
            for address in addresses {
                validate_wire_address(address)?;
            }
        }
        EventKind::RobotsFound { text, status } => ensure!(
            text.len() <= 256 * 1024 && (100..=599).contains(status),
            "invalid robots response"
        ),
        EventKind::RedirectFound { address } => validate_wire_address(address)?,
        EventKind::Progress { completed, total } => ensure!(
            total.is_none_or(|total| *completed <= total),
            "invalid progress count"
        ),
        EventKind::ChannelCaptured { key, capture } => {
            validate_target_key(key)?;
            capture.validate()?;
        }
        EventKind::ChannelFailed {
            key, kind, code, ..
        } => {
            validate_target_key(key)?;
            ensure!(
                match kind {
                    CheckKind::Screenshot =>
                        !matches!(code, FailureCode::DomLimit | FailureCode::DomSerialization),
                    CheckKind::DomSnapshot => *code != FailureCode::ImageLimit,
                },
                "wrong channel failure kind"
            );
        }
        EventKind::ChannelWarning { key, kind, code } => {
            validate_target_key(key)?;
            validate_channel_warning(*kind, *code)?;
        }
        EventKind::CaptureCompleted { key } => validate_target_key(key)?,
        EventKind::Ready {
            worker_version,
            browser_version,
        } => ensure!(
            worker_version.len() <= 128 && browser_version.len() <= 128,
            "oversized worker identity"
        ),
        _ => (),
    }
    Ok(event)
}
fn validate_target_key(key: &TargetKey) -> Result<()> {
    ensure!(
        !key.page.0.is_nil() && !key.profile.0.is_nil(),
        "invalid capture target"
    );
    Ok(())
}
fn validate_channel_warning(kind: CheckKind, code: WarningCode) -> Result<()> {
    ensure!(
        match kind {
            CheckKind::DomSnapshot => matches!(
                code,
                WarningCode::ReadinessBudget
                    | WarningCode::Unstable
                    | WarningCode::ResourceFailed
                    | WarningCode::EmbeddedContentBlocked
                    | WarningCode::IframeDocumentsExcluded
                    | WarningCode::ClosedShadowRootsExcluded
                    | WarningCode::NearbySamplesNotAtomic
            ),
            CheckKind::Screenshot => !matches!(
                code,
                WarningCode::IframeDocumentsExcluded
                    | WarningCode::ClosedShadowRootsExcluded
                    | WarningCode::NearbySamplesNotAtomic
            ),
        },
        "wrong channel warning kind"
    );
    Ok(())
}
pub struct RequestTracker {
    request_id: u64,
    job: RunId,
    acknowledged: bool,
    terminal: bool,
    command: Command,
    published: std::collections::BTreeSet<CheckKind>,
    tokens: std::collections::BTreeSet<uuid::Uuid>,
    warnings: std::collections::BTreeMap<CheckKind, Vec<WarningCode>>,
}
impl RequestTracker {
    pub fn new(request: &Request) -> Self {
        Self {
            request_id: request.request_id,
            job: request.job,
            acknowledged: false,
            terminal: false,
            command: request.command.clone(),
            published: Default::default(),
            tokens: Default::default(),
            warnings: Default::default(),
        }
    }
    pub fn accept(&mut self, event: &Event) -> Result<()> {
        ensure!(
            event.version == PROTOCOL_VERSION
                && event.request_id == self.request_id
                && event.job == self.job,
            "event correlation mismatch"
        );
        ensure!(!self.terminal, "event after completion");
        if matches!(event.event, EventKind::Acknowledged) {
            ensure!(!self.acknowledged, "duplicate acknowledgement");
            self.acknowledged = true;
            return Ok(());
        }
        ensure!(self.acknowledged, "event before acknowledgement");
        if let Command::CaptureTarget { key, plan, .. } = &self.command {
            let mut publication = false;
            let channel = match &event.event {
                EventKind::ChannelCaptured {
                    key: actual,
                    capture,
                } => {
                    capture.validate()?;
                    if let CapturedChannel::DomSnapshot { policy, .. } = capture {
                        ensure!(Some(*policy) == plan.dom_policy, "capture policy mismatch");
                    }
                    ensure!(
                        self.tokens.insert(capture.output_token()),
                        "duplicate output token"
                    );
                    publication = true;
                    Some((actual, capture.kind()))
                }
                EventKind::ChannelFailed {
                    key, kind, code, ..
                } => {
                    ensure!(
                        match kind {
                            CheckKind::Screenshot => !matches!(
                                code,
                                FailureCode::DomLimit | FailureCode::DomSerialization
                            ),
                            CheckKind::DomSnapshot => *code != FailureCode::ImageLimit,
                        },
                        "wrong channel failure kind"
                    );
                    publication = true;
                    Some((key, *kind))
                }
                EventKind::ChannelWarning { key, kind, code } => {
                    validate_channel_warning(*kind, *code)?;
                    let seen = self.warnings.entry(*kind).or_default();
                    ensure!(
                        seen.len() < 64 && !seen.contains(code),
                        "duplicate or excessive channel warnings"
                    );
                    seen.push(*code);
                    Some((key, *kind))
                }
                EventKind::CaptureCompleted { key: actual } => {
                    ensure!(
                        actual == key && self.published.len() == plan.kinds().count(),
                        "early or foreign capture completion"
                    );
                    self.terminal = true;
                    return Ok(());
                }
                EventKind::Cancelled => {
                    self.terminal = true;
                    return Ok(());
                }
                EventKind::Progress { .. } => None,
                _ => anyhow::bail!("unexpected capture event"),
            };
            if let Some((actual, kind)) = channel {
                ensure!(
                    actual == key && plan.kinds().any(|k| k == kind),
                    "foreign or unrequested capture channel"
                );
                ensure!(
                    !self.published.contains(&kind),
                    "duplicate or late channel publication"
                );
                if publication {
                    self.published.insert(kind);
                }
            }
            return Ok(());
        }
        ensure!(
            !matches!(
                event.event,
                EventKind::ChannelCaptured { .. }
                    | EventKind::ChannelFailed { .. }
                    | EventKind::ChannelWarning { .. }
                    | EventKind::CaptureCompleted { .. }
            ),
            "capture event for noncapture request"
        );
        ensure!(
            match &event.event {
                EventKind::Ready { .. } => matches!(self.command, Command::Handshake),
                EventKind::Completed => matches!(
                    self.command,
                    Command::Shutdown | Command::InstallSession { .. } | Command::InstallSmoke
                ),
                EventKind::LinksFound { .. } | EventKind::RedirectFound { .. } => matches!(
                    self.command,
                    Command::DiscoverPage { .. } | Command::DiscoverAuthenticated { .. }
                ),
                EventKind::RobotsFound { .. } =>
                    matches!(self.command, Command::FetchRobots { .. }),
                EventKind::LoginOpened => matches!(self.command, Command::OpenLogin { .. }),
                EventKind::SessionReady { .. } =>
                    matches!(self.command, Command::ConfirmLogin { .. }),
                _ => true,
            },
            "wrong terminal event for command"
        );
        self.terminal = matches!(
            event.event,
            EventKind::Ready { .. }
                | EventKind::LinksFound { .. }
                | EventKind::RobotsFound { .. }
                | EventKind::RedirectFound { .. }
                | EventKind::Completed
                | EventKind::LoginOpened
                | EventKind::SessionReady { .. }
                | EventKind::Cancelled
                | EventKind::Failure { .. }
        );
        Ok(())
    }
    pub fn terminal(&self) -> bool {
        self.terminal
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn capture_budget_accounts_for_independent_bounded_stages_and_slack() {
        let mut profile = CaptureProfile::desktop();
        profile.stabilization.navigation_ms = 3000;
        profile.stabilization.readiness_ms = 180;
        for (mode, expected) in [
            (CaptureMode::ScreenshotsOnly, 13540),
            (CaptureMode::ScreenshotsAndDomSnapshots, 13720),
            (CaptureMode::DomSnapshotsOnly, 13360),
        ] {
            assert_eq!(
                capture_request_budget(&profile, &CapturePlan::for_mode(mode))
                    .unwrap()
                    .as_millis(),
                expected
            );
        }
        profile.stabilization.navigation_ms = 120_000;
        profile.stabilization.readiness_ms = 120_000;
        assert_eq!(
            capture_request_budget(
                &profile,
                &CapturePlan::for_mode(CaptureMode::ScreenshotsAndDomSnapshots)
            )
            .unwrap()
            .as_millis(),
            493_000
        );
    }
    #[test]
    fn private_capture_trace_is_bounded_typed_and_string_free() {
        let valid=br#"[{"event":"failed","elapsed_ms":50,"request":2,"kind":"image","replacing":false,"known_replacement_source":true,"failure":"Aborted","attributed_replacement":true,"request_start_ms":-1}]"#;
        assert_eq!(decode_capture_trace(valid).unwrap().len(), 1);
        for invalid in [
            br#"[{"event":"private-url","elapsed_ms":1}]"#.as_slice(),
            br#"[{"event":"request","elapsed_ms":1,"url":"secret"}]"#,
            br#"[{"event":"request","elapsed_ms":1,"kind":"secret"}]"#,
            br#"[{"event":"request","elapsed_ms":300001}]"#,
        ] {
            assert!(decode_capture_trace(invalid).is_err());
        }
        assert!(decode_capture_trace(&vec![b' '; 65537]).is_err());
    }
    #[test]
    fn shared_session_frames_encode_exactly_and_reject_each_nil_id() {
        // Node consumes these same exact NDJSON bytes. Exact re-encoding proves
        // they are emitted by the Rust wire boundary, not just similar JSON.
        for frame in include_str!("../../../worker/product/test/fixtures/session-requests.ndjson")
            .split_inclusive('\n')
        {
            let request: Request = serde_json::from_str(frame).unwrap();
            assert_eq!(encode_request(&request).unwrap(), frame.as_bytes());
            for field in ["id", "project", "environment"] {
                let mut value = serde_json::to_value(&request).unwrap();
                value["command"]["payload"]["session"][field] =
                    serde_json::json!(uuid::Uuid::nil());
                let invalid: Request = serde_json::from_value(value).unwrap();
                assert_eq!(
                    encode_request(&invalid).unwrap_err().to_string(),
                    "invalid session reference: IDs must be non-nil"
                );
            }
            for field in ["id", "project", "environment"] {
                let mut value = serde_json::to_value(&request).unwrap();
                value["command"]["payload"]["session"]
                    .as_object_mut()
                    .unwrap()
                    .remove(field);
                assert!(serde_json::from_value::<Request>(value).is_err());
            }
            let mut value = serde_json::to_value(&request).unwrap();
            value["command"]["payload"]["session"] = serde_json::Value::Null;
            if matches!(request.command, Command::CaptureTarget { .. }) {
                let public: Request = serde_json::from_value(value).unwrap();
                let bytes = encode_request(&public).unwrap();
                assert!(serde_json::from_slice::<serde_json::Value>(&bytes).unwrap()["command"]["payload"]["session"].is_null());
            } else {
                assert!(serde_json::from_value::<Request>(value).is_err());
            }
        }
    }
    #[test]
    fn bounded_frames_reject_partial_oversized_and_malformed() {
        assert!(read_json_frame(&mut &b"{}"[..]).is_err());
        assert!(read_json_frame(&mut &b"not-json\n"[..]).is_err());
        let oversized = vec![b' '; MAX_FRAME + 1];
        assert!(read_json_frame(&mut oversized.as_slice()).is_err());
        assert!(read_json_frame(&mut &b""[..]).unwrap().is_none());
        assert_eq!(
            read_json_frame(&mut &b"{}\n"[..]).unwrap().unwrap(),
            serde_json::json!({})
        );
    }
    #[test]
    fn correlation_ack_and_terminal_are_enforced() {
        let request = Request {
            version: PROTOCOL_VERSION,
            request_id: 1,
            job: RunId::new(),
            command: Command::Shutdown,
        };
        let mut tracker = RequestTracker::new(&request);
        let mut event = Event {
            version: PROTOCOL_VERSION,
            request_id: 1,
            job: request.job,
            event: EventKind::Completed,
        };
        assert!(tracker.accept(&event).is_err());
        event.event = EventKind::Acknowledged;
        tracker.accept(&event).unwrap();
        assert!(tracker.accept(&event).is_err());
        event.job = RunId::new();
        assert!(tracker.accept(&event).is_err());
        event.job = request.job;
        event.event = EventKind::Completed;
        tracker.accept(&event).unwrap();
        assert!(tracker.terminal());
        assert!(tracker.accept(&event).is_err());
    }
    #[test]
    fn versions_and_unknown_commands_are_not_fixture_compatible() {
        let value = serde_json::json!({"version":1,"request_id":1,"job":RunId::new(),"event":{"type":"Completed"}});
        assert!(decode_event(value).is_err());
        let request = serde_json::json!({"version":2,"request_id":1,"job":RunId::new(),"command":{"type":"runArbitraryScript","payload":{"script":"x"}}});
        assert!(serde_json::from_value::<Request>(request).is_err());
    }
    #[test]
    fn unconfigured_profiles_and_bad_progress_cannot_be_presented_as_capture() {
        let request = Request {
            version: PROTOCOL_VERSION,
            request_id: 1,
            job: RunId::new(),
            command: Command::CaptureTarget {
                key: TargetKey {
                    page: PageId::new(),
                    profile: ProfileId::new(),
                },
                address: "https://example.test".into(),
                profile: Box::new(CaptureProfile::desktop()),
                session: None,
                plan: CapturePlan::default(),
            },
        };
        assert!(encode_request(&request).is_err());
        let event = Event {
            version: PROTOCOL_VERSION,
            request_id: 1,
            job: request.job,
            event: EventKind::Progress {
                completed: 2,
                total: Some(1),
            },
        };
        assert!(decode_event(serde_json::to_value(event).unwrap()).is_err());
    }
    #[test]
    fn captured_artifacts_reject_non_screenshots_nil_keys_and_injected_paths() {
        let event = Event {
            version: PROTOCOL_VERSION,
            request_id: 1,
            job: RunId::new(),
            event: EventKind::ChannelCaptured {
                key: TargetKey {
                    page: PageId::new(),
                    profile: ProfileId::new(),
                },
                capture: CapturedChannel::Screenshot {
                    output_token: uuid::Uuid::new_v4(),
                    artifact: Artifact {
                        id: "a".repeat(64).try_into().unwrap(),
                        bytes: 16,
                        kind: ArtifactKind::Screenshot,
                    },
                    width: 1440,
                    height: 900,
                },
            },
        };
        let value = serde_json::to_value(event).unwrap();
        decode_event(value.clone()).unwrap();
        for (field, bad) in [
            ("width", serde_json::json!(0)),
            ("height", serde_json::json!(100001)),
            ("output_token", serde_json::json!(uuid::Uuid::nil())),
        ] {
            let mut v = value.clone();
            v["event"]["payload"]["capture"][field] = bad;
            assert!(decode_event(v).is_err());
        }
        let mut v = value.clone();
        v["event"]["payload"]["key"]["page"] = serde_json::json!(uuid::Uuid::nil());
        assert!(decode_event(v).is_err());
        let mut v = value.clone();
        v["event"]["payload"]["capture"]["artifact"]["kind"] = serde_json::json!("Thumbnail");
        assert!(decode_event(v).is_err());
        let mut v = value;
        v["event"]["payload"]["capture"]["artifact"]["path"] =
            serde_json::json!("../../outside.png");
        assert!(decode_event(v).is_err());
    }
    fn channel_fixture() -> (Request, Vec<Event>) {
        let mut request: Request = serde_json::from_str(
            include_str!("../../../worker/product/test/fixtures/session-requests.ndjson")
                .lines()
                .nth(1)
                .unwrap(),
        )
        .unwrap();
        if let Command::CaptureTarget { plan, .. } = &mut request.command {
            *plan = CapturePlan::for_mode(CaptureMode::ScreenshotsAndDomSnapshots);
        }
        let events = include_str!("../../../worker/product/test/fixtures/capture-events.ndjson")
            .split_inclusive('\n')
            .map(|frame| {
                let event = decode_event(serde_json::from_str(frame).unwrap()).unwrap();
                assert_eq!(
                    format!("{}\n", serde_json::to_string(&event).unwrap()),
                    frame
                );
                event
            })
            .collect();
        (request, events)
    }
    #[test]
    fn shared_v3_channel_frames_publish_independently_before_terminal_completion() {
        let (request, events) = channel_fixture();
        let mut tracker = RequestTracker::new(&request);
        for (i, event) in events.iter().enumerate() {
            tracker.accept(event).unwrap();
            assert_eq!(tracker.terminal(), i == events.len() - 1);
        }
    }
    #[test]
    fn timeout_failure_accepts_prior_warnings_progress_and_completion_but_not_late_warnings() {
        let (request, events) = channel_fixture();
        let key = match &request.command {
            Command::CaptureTarget { key, .. } => key.clone(),
            _ => unreachable!(),
        };
        let event = |kind| Event {
            event: kind,
            ..events[0].clone()
        };
        let warning = event(EventKind::ChannelWarning {
            key: key.clone(),
            kind: CheckKind::Screenshot,
            code: WarningCode::FontsIncomplete,
        });
        let failure = event(EventKind::ChannelFailed {
            key: key.clone(),
            kind: CheckKind::Screenshot,
            code: FailureCode::Timeout,
            retryable: true,
        });
        let progress = event(EventKind::Progress {
            completed: 1,
            total: Some(1),
        });
        let mut tracker = RequestTracker::new(&request);
        for prior in [
            &events[0], &events[1], &events[2], &warning, &failure, &progress,
        ] {
            tracker.accept(prior).unwrap();
            assert!(!tracker.terminal());
        }
        let late = event(EventKind::ChannelWarning {
            key,
            kind: CheckKind::Screenshot,
            code: WarningCode::ImagesIncomplete,
        });
        assert_eq!(
            tracker.accept(&late).unwrap_err().to_string(),
            "duplicate or late channel publication"
        );
        tracker.accept(events.last().unwrap()).unwrap();
        assert!(tracker.terminal());
        assert!(tracker.accept(&late).is_err());
    }
    #[test]
    fn capture_cardinality_foreign_unrequested_and_wrong_kind_are_rejected() {
        let (request, events) = channel_fixture();
        let mut tracker = RequestTracker::new(&request);
        tracker.accept(&events[0]).unwrap();
        assert!(tracker.accept(events.last().unwrap()).is_err());
        assert!(
            tracker
                .accept(&Event {
                    event: EventKind::Completed,
                    ..events[0].clone()
                })
                .is_err()
        );
        tracker.accept(&events[2]).unwrap();
        assert!(tracker.accept(&events[2]).is_err());
        let mut duplicate_kind = events[2].clone();
        if let EventKind::ChannelCaptured {
            capture: CapturedChannel::DomSnapshot { output_token, .. },
            ..
        } = &mut duplicate_kind.event
        {
            *output_token = uuid::Uuid::new_v4();
        }
        assert!(tracker.accept(&duplicate_kind).is_err());
        for mutation in [
            "foreign",
            "unrequested",
            "wrong-artifact",
            "wrong-policy",
            "path",
            "dimensions",
        ] {
            let mut value = serde_json::to_value(&events[2]).unwrap();
            match mutation {
                "foreign" => {
                    value["event"]["payload"]["key"]["page"] = serde_json::json!(PageId::new())
                }
                "wrong-artifact" => {
                    value["event"]["payload"]["capture"]["artifact"]["kind"] =
                        serde_json::json!("Screenshot")
                }
                "wrong-policy" => {
                    value["event"]["payload"]["capture"]["policy"]["version"] = serde_json::json!(2)
                }
                "path" => {
                    value["event"]["payload"]["capture"]["path"] =
                        serde_json::json!("../../private")
                }
                "dimensions" => {
                    value["event"]["payload"]["capture"]["width"] = serde_json::json!(100)
                }
                _ => (),
            }
            let mut selected = request.clone();
            if mutation == "unrequested"
                && let Command::CaptureTarget { plan, .. } = &mut selected.command
            {
                *plan = CapturePlan::default();
            }
            let mut tracker = RequestTracker::new(&selected);
            tracker.accept(&events[0]).unwrap();
            assert!(
                decode_event(value)
                    .and_then(|event| tracker.accept(&event))
                    .is_err(),
                "{mutation}"
            );
        }
        let mut value = serde_json::to_value(&events[3]).unwrap();
        value["event"]["payload"]["kind"] = serde_json::json!("DomSnapshot");
        assert!(decode_event(value).is_err());
    }
    #[test]
    fn malformed_worker_fields_never_echo_private_values_in_diagnostics() {
        let (_, events) = channel_fixture();
        let mut value = serde_json::to_value(&events[2]).unwrap();
        value["event"]["payload"]["capture"]["kind"] = serde_json::json!("DO_NOT_ECHO_DOM_BODY");
        let error = decode_event(value).unwrap_err();
        assert_eq!(error.to_string(), "invalid worker event schema");
        assert!(!format!("{error:#}").contains("DO_NOT_ECHO_DOM_BODY"));
        let Err(trace) = decode_capture_trace(br#"[{"event":"DO_NOT_ECHO_DOM_BODY"}]"#) else {
            panic!("malformed private trace accepted");
        };
        assert!(!format!("{trace:#}").contains("DO_NOT_ECHO_DOM_BODY"));
    }
}
