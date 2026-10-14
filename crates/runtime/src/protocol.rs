// SPDX-License-Identifier: MPL-2.0
//! Product protocol v2 is deliberately not the fixture bench's v1 protocol.
use anyhow::{Result, ensure};
use regressee_core::domain::*;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Read};
pub const PROTOCOL_VERSION: u32 = 2;
pub const WORKER_VERSION: &str = "5.0.1";
pub const MAX_FRAME: usize = 1024 * 1024;
pub const MAX_LINKS: usize = 2000;
pub const MAX_REQUEST_ID: u64 = 9_007_199_254_740_991; // Node safe integer range.
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
    let trace: Vec<CaptureTrace> = serde_json::from_slice(bytes)?;
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
    TargetCaptured {
        key: TargetKey,
        output_token: uuid::Uuid,
        artifact: Artifact,
        width: u32,
        height: u32,
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
        } => {
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
    let event: Event = serde_json::from_value(value)?;
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
        EventKind::TargetCaptured {
            artifact,
            width,
            height,
            output_token,
            key,
        } => ensure!(
            !output_token.is_nil()
                && !key.page.0.is_nil()
                && !key.profile.0.is_nil()
                && artifact.kind == ArtifactKind::Screenshot
                && artifact.bytes > 0
                && artifact.bytes <= 128 * 1024 * 1024
                && *width > 0
                && *width <= 32768
                && *height > 0
                && *height <= 100_000,
            "invalid captured artifact limits"
        ),
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
pub struct RequestTracker {
    request_id: u64,
    job: RunId,
    acknowledged: bool,
    terminal: bool,
}
impl RequestTracker {
    pub fn new(request: &Request) -> Self {
        Self {
            request_id: request.request_id,
            job: request.job,
            acknowledged: false,
            terminal: false,
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
        self.terminal = matches!(
            event.event,
            EventKind::Ready { .. }
                | EventKind::LinksFound { .. }
                | EventKind::RobotsFound { .. }
                | EventKind::RedirectFound { .. }
                | EventKind::TargetCaptured { .. }
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
            version: 2,
            request_id: 1,
            job: RunId::new(),
            command: Command::Handshake,
        };
        let mut tracker = RequestTracker::new(&request);
        let mut event = Event {
            version: 2,
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
            version: 2,
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
            },
        };
        assert!(encode_request(&request).is_err());
        let event = Event {
            version: 2,
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
            version: 2,
            request_id: 1,
            job: RunId::new(),
            event: EventKind::TargetCaptured {
                key: TargetKey {
                    page: PageId::new(),
                    profile: ProfileId::new(),
                },
                output_token: uuid::Uuid::new_v4(),
                artifact: Artifact {
                    id: "a".repeat(64).try_into().unwrap(),
                    bytes: 16,
                    kind: ArtifactKind::Screenshot,
                },
                width: 1440,
                height: 900,
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
            v["event"]["payload"][field] = bad;
            assert!(decode_event(v).is_err());
        }
        let mut v = value.clone();
        v["event"]["payload"]["key"]["page"] = serde_json::json!(uuid::Uuid::nil());
        assert!(decode_event(v).is_err());
        let mut v = value.clone();
        v["event"]["payload"]["artifact"]["kind"] = serde_json::json!("Thumbnail");
        assert!(decode_event(v).is_err());
        let mut v = value;
        v["event"]["payload"]["artifact"]["path"] = serde_json::json!("../../outside.png");
        assert!(decode_event(v).is_err());
    }
}
