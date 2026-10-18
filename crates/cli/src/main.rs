// SPDX-License-Identifier: MPL-2.0
mod commands;
use anyhow::{Context, Result, bail, ensure};
use commands::{Args, Plan};
use regressee_app::{
    engine::{Client, Launcher, Rejected, ownership},
    protocol::*,
};
use regressee_core::{
    dom_snapshot::DomComparisonOutcome,
    domain::{Classification, Project, RunIntent, TargetOutcome},
    evidence::ProjectEvidence,
};
use regressee_storage::AppPaths;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::ExitCode,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Duration,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ticket {
    pub operation: Operation,
    pub confirmation: Confirmation,
}
struct Output {
    code: String,
    exit: u8,
    data: Value,
}
#[derive(Debug)]
struct InvalidRequest;
impl std::fmt::Display for InvalidRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("invalid_request: invalid CLI arguments or input")
    }
}
impl std::error::Error for InvalidRequest {}
impl Output {
    fn ok(data: impl Serialize) -> Result<Self> {
        Ok(Self {
            code: "ok".into(),
            exit: 0,
            data: serde_json::to_value(data)?,
        })
    }
}
#[derive(Default)]
struct Options {
    json: bool,
    data: Option<PathBuf>,
    cache: Option<PathBuf>,
    request: Option<Uuid>,
    detach: bool,
    prepare: bool,
    save: Option<PathBuf>,
    save_rejected: bool,
    session: Option<Uuid>,
}
#[derive(Debug)]
struct SubmitLost {
    request: Uuid,
    path: PathBuf,
    message: String,
}
impl std::fmt::Display for SubmitLost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} Request {}; saved original: {}. Inspect jobs show or use jobs retry with this request ID, never a new mutation ID.",
            self.message,
            self.request,
            self.path.display()
        )
    }
}
impl std::error::Error for SubmitLost {}
fn paths(o: &Options) -> Result<AppPaths> {
    regressee_app::launch::configured_paths(o.data.clone(), o.cache.clone())
}
fn request(client: &Client, r: Request) -> Result<Response> {
    match client.request(r)? {
        Response::Error { code, message } => Err(Rejected { code, message }.into()),
        r => Ok(r),
    }
}
fn job(client: &Client, id: Uuid) -> Result<Job> {
    match request(client, Request::Job { id })? {
        Response::Job(j) => Ok(*j),
        _ => bail!("internal: unexpected job response"),
    }
}
fn interrupted() -> Arc<AtomicBool> {
    let stop = Arc::new(AtomicBool::new(false));
    let flag = stop.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();
        if let Ok(rt) = rt {
            rt.block_on(async {
                if tokio::signal::ctrl_c().await.is_ok() {
                    flag.store(true, Ordering::Release);
                }
            });
        }
    });
    stop
}
fn wait(client: &Client, mut j: Job, stop: &AtomicBool) -> Result<Output> {
    while !j.state.terminal() {
        if stop.load(Ordering::Acquire) {
            return Ok(Output {
                code: "wait_detached".into(),
                exit: 130,
                data: json!({"job":j,"message":"Wait interrupted; job continues. Use jobs wait/show; cancellation is a separate explicit command."}),
            });
        }
        std::thread::sleep(Duration::from_millis(75));
        j = job(client, j.id).map_err(|e| SubmitLost {
            request: j.id,
            path: client
                .paths
                .data
                .join("cli-requests")
                .join(format!("{}.json", j.id)),
            message: format!("Wait disconnected; the detached job was not cancelled: {e:#}."),
        })?;
    }
    result(j)
}
fn result(j: Job) -> Result<Output> {
    let (mut code, mut exit) = match j.state {
        JobState::Queued | JobState::Running => ("job_admitted", 0),
        JobState::Succeeded => ("ok", 0),
        JobState::Paused => ("capture_paused", 3),
        JobState::Cancelled => ("cancelled", 3),
        JobState::Interrupted => ("completion_uncertain", 7),
        JobState::Failed => ("operation_failed", 6),
    };
    if let Some(outcome) = &j.outcome {
        match outcome {
            Outcome::Failure(regressee_app::jobs::StorageFailure::InvalidProject) => {
                code = "invalid_request";
                exit = 2;
            }
            Outcome::Failure(regressee_app::jobs::StorageFailure::ConflictOrWrite) => {
                code = "storage_write_failed";
                exit = 6;
            }
            Outcome::Failure(regressee_app::jobs::StorageFailure::MissingOrCorrupt) => {
                code = "not_found_or_corrupt";
                exit = 6;
            }
            Outcome::ActionRequired(_) => {
                code = "action_required";
                exit = 4;
            }
            Outcome::ProductFailure(message) => {
                let c = error_code(message);
                code = c.0;
                exit = c.1;
            }
            Outcome::Captured(e) => {
                if j.state == JobState::Succeeded {
                    (code, exit) = capture_verdict(e);
                } else if j.state == JobState::Failed {
                    code = "incomplete_capture";
                    exit = 3;
                }
            }
            _ => {}
        }
    }
    if j.state == JobState::Interrupted {
        code = "completion_uncertain";
        exit = 7;
    }
    let value = j.outcome.as_ref().map(|o| {
        let v = serde_json::to_value(o).unwrap_or(Value::Null);
        let v = v.get("value").cloned().unwrap_or(v);
        if matches!(o, Outcome::Terminal(_) | Outcome::Dom(_)) {
            v.get("value").cloned().unwrap_or(v)
        } else if matches!(o, Outcome::History { .. }) {
            v.get("history").cloned().unwrap_or(v)
        } else {
            v
        }
    });
    let mut output = Output {
        code: code.into(),
        exit,
        data: json!({"job":j,"value":value}),
    };
    // Every recovery route renders the original preparation identically. This
    // only reconstructs a DTO from the durable receipt; it never prepares or
    // consumes confirmation authority and does not refresh a stale ticket.
    if j.state == JobState::Succeeded
        && let (Operation::Prepare(operation), Some(Outcome::Confirmation(confirmation))) =
            (&j.submitted.operation, &j.outcome)
    {
        ensure!(
            operation.fingerprint()? == confirmation.fingerprint
                && j.submitted.engine == confirmation.engine
                && j.submitted.epoch == confirmation.epoch,
            "internal: preparation receipt binding mismatch"
        );
        output.data["ticket"] = serde_json::to_value(Ticket {
            operation: (**operation).clone(),
            confirmation: confirmation.clone(),
        })?;
        output.code = "confirmation_prepared".into();
    }
    if let Some(Outcome::SavedReview(reviews)) = &j.outcome {
        output.data["review_history"] = json!({"semantics":"recent run-wide decisions, not authoritative current target state; use review history for full pagination","limit":256,"may_be_truncated":reviews.len()==256});
    }
    Ok(output)
}
fn capture_verdict(evidence: &ProjectEvidence) -> (&'static str, u8) {
    let Some(record) = &evidence.latest else {
        return ("incomplete_capture", 3);
    };
    let run = &record.run;
    if run.state != regressee_core::domain::JobState::Completed
        || run.targets.is_empty()
        || run.targets.iter().any(|t| !t.requested_kinds_captured())
    {
        return ("incomplete_capture", 3);
    }
    if !matches!(run.intent, RunIntent::Check { .. }) {
        return ("ok", 0);
    }
    let Some(baseline) = &evidence.baseline else {
        return ("incomplete_capture", 3);
    };
    // Preserve legacy screenshot-only fixed-subset check exits. DOM-enabled
    // analyses explicitly expose intended-slot omissions as noncomparability.
    if baseline.capture_plan.mode.dom_snapshots()
        && baseline
            .capture_plan
            .kinds()
            .any(|kind| baseline.reference_count(kind) < baseline.intended_count as usize)
    {
        return ("incomplete_capture", 3);
    }
    let screenshots = evidence
        .comparison
        .as_ref()
        .map(|a| a.review_classifications())
        .unwrap_or_default();
    let dom = evidence
        .dom_comparison
        .as_ref()
        .map(|a| {
            a.revision
                .results
                .iter()
                .map(|r| (&r.key, &r.outcome))
                .collect::<std::collections::BTreeMap<_, _>>()
        })
        .unwrap_or_default();
    let mut verdicts = Vec::new();
    for target in &run.targets {
        if !matches!(target.outcome, TargetOutcome::NotRequested) {
            verdicts.push(match screenshots.get(&target.key) {
                Some(Classification::Changed) => Some(true),
                Some(Classification::NoSignificantChange) => Some(false),
                _ => None,
            });
        }
        if !target.dom_outcome.is_not_requested() {
            verdicts.push(match dom.get(&target.key) {
                Some(DomComparisonOutcome::Changed) => Some(true),
                Some(DomComparisonOutcome::Unchanged) => Some(false),
                _ => None,
            });
        }
    }
    classify_check_outcomes(run.capture_plan.mode.dom_snapshots(), &verdicts)
}
fn classify_check_outcomes(dom_enabled: bool, verdicts: &[Option<bool>]) -> (&'static str, u8) {
    if verdicts.is_empty() || verdicts.iter().any(Option::is_none) {
        return ("incomplete_capture", 3);
    }
    if verdicts.contains(&Some(true)) {
        (
            if dom_enabled {
                "check_differences"
            } else {
                "visual_differences"
            },
            1,
        )
    } else {
        ("ok", 0)
    }
}
fn error_code(message: &str) -> (&'static str, u8) {
    // Engine-owned tagged failures, not incidental words in prose. Local
    // parser/domain validation is independently classified by InvalidRequest.
    match message.split(':').next().unwrap_or("").trim() {
        "confirmation_required" => ("confirmation_required", 4),
        "replacement_required" => ("replacement_required", 4),
        "display_required" => ("display_required", 4),
        "unsupported" => ("unsupported", 4),
        "confirmation_conflict"
        | "revision_conflict"
        | "request_conflict"
        | "cache_directory_conflict"
        | "stale_epoch"
        | "authority_conflict"
        | "confirmation_replayed" => ("conflict", 5),
        "incomplete_capture" => ("incomplete_capture", 3),
        "response_too_large" => ("response_too_large", 2),
        "invalid_request" | "invalid_input" | "invalid_arguments" | "invalid_path"
        | "request_too_large" => ("invalid_request", 2),
        "not_found" => ("not_found", 6),
        "storage_write_failed" => ("storage_write_failed", 6),
        "busy" | "job_quota" | "engine_draining" => ("busy", 4),
        "version_mismatch" => ("version_mismatch", 6),
        _ => ("operation_failed", 6),
    }
}
fn spool(client: &Client, id: Uuid) -> Result<PathBuf> {
    let directory = client.paths.data.join("cli-requests");
    if !directory.exists() {
        let mut b = std::fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            b.mode(0o700);
        }
        b.create(&directory).or_else(|e| {
            if e.kind() == std::io::ErrorKind::AlreadyExists {
                Ok(())
            } else {
                Err(e)
            }
        })?;
        #[cfg(windows)]
        regressee_core::platform::protect(&directory)?;
    }
    ownership::private_directory(&directory)?;
    Ok(directory.join(format!("{id}.json")))
}
fn submit(
    client: &Client,
    operation: Operation,
    confirmation: Option<Uuid>,
    o: &Options,
    stop: &AtomicBool,
) -> Result<Output> {
    if stop.load(Ordering::Acquire) {
        return Ok(not_submitted());
    }
    if operation.needs_confirmation() && confirmation.is_none() && !o.prepare {
        bail!(
            "confirmation_required: repeat with --prepare, inspect the exact ticket and confirm --file TICKET --confirmation UUID; no --yes bypass"
        );
    }
    let operation = if o.prepare {
        ensure!(
            operation.needs_confirmation(),
            "invalid_arguments: --prepare applies only to consequential commands"
        );
        Operation::Prepare(Box::new(operation))
    } else {
        operation
    };
    let id = o.request.unwrap_or_else(Uuid::new_v4);
    let path = spool(client, id)?;
    let submitted = if path.exists() {
        ownership::private_file(&path, commands::MAX_INPUT)?;
        let original: Submit = serde_json::from_slice(&std::fs::read(&path)?)?;
        ensure!(
            original.operation.fingerprint()? == operation.fingerprint()?
                && original.confirmation == confirmation,
            "request_conflict: use jobs retry with the saved original request, never change a request payload"
        );
        original
    } else {
        let s = client.snapshot()?;
        let submitted = Submit {
            request: id,
            engine: s.engine,
            epoch: s.epoch,
            operation,
            confirmation,
            expected_revision: None,
        };
        let entries =
            std::fs::read_dir(path.parent().unwrap())?.collect::<std::io::Result<Vec<_>>>()?;
        ensure!(
            entries.len() < MAX_JOBS
                && entries
                    .iter()
                    .map(|e| e.metadata().map_or(0, |m| m.len()))
                    .sum::<u64>()
                    < 512 * 1024 * 1024,
            "job_quota: retained CLI request journal is full"
        );
        use std::io::Write;
        let mut temp = tempfile::NamedTempFile::new_in(path.parent().unwrap())?;
        #[cfg(windows)]
        regressee_core::platform::protect(temp.path())?;
        let bytes = serde_json::to_vec(&submitted)?;
        ensure!(
            bytes.len() as u64 <= commands::MAX_INPUT,
            "invalid_input: submit exceeds 8 MiB"
        );
        temp.write_all(&bytes)?;
        temp.as_file().sync_all()?;
        match temp.persist_noclobber(&path) {
            Ok(_) => {}
            Err(e) if e.error.kind() == std::io::ErrorKind::AlreadyExists => {
                ownership::private_file(&path, commands::MAX_INPUT)?;
                let original: Submit = serde_json::from_slice(&std::fs::read(&path)?)?;
                ensure!(
                    serde_json::to_vec(&original)? == serde_json::to_vec(&submitted)?,
                    "request_conflict: concurrent request ID payload differs"
                );
            }
            Err(e) => return Err(e.error.into()),
        }
        #[cfg(unix)]
        std::fs::File::open(path.parent().unwrap())?.sync_all()?;
        submitted
    };
    // The payload is fsynced before submit; even a lost reply keeps its identity.
    eprintln!("Request {id} — {}", path.display());
    if stop.load(Ordering::Acquire) {
        let mut out = not_submitted();
        out.data["request_id"] = json!(id);
        out.data["retry_file"] = json!(path);
        return Ok(out);
    }
    let admitted = match client.submit(submitted.clone()) {
        Ok(j) => j,
        Err(e) if e.is::<Rejected>() => return Err(e),
        Err(e) => {
            return Err(SubmitLost {
                request: id,
                path,
                message: format!("Admission uncertain: {e:#}."),
            }
            .into());
        }
    };
    let mut output = if o.detach {
        result(admitted)?
    } else {
        wait(client, admitted, stop)?
    };
    output.data["request_id"] = json!(id);
    output.data["retry_file"] = json!(path);
    // Completed reads are not durable mutation authority; reclaim them so ordinary
    // inspection does not exhaust the engine's bounded retained receipt quota.
    if !submitted.operation.mutates()
        && !matches!(submitted.operation, Operation::Prepare(_))
        && output.data["job"]["state"] != "Queued"
        && output.data["job"]["state"] != "Running"
        && request(client, Request::Acknowledge { id }).is_ok()
    {
        let _ = std::fs::remove_file(path);
        output.data["retry_file"] = Value::Null;
        output.data["read_receipt_reclaimed"] = json!(true);
    }
    Ok(output)
}
fn not_submitted() -> Output {
    Output {
        code: "wait_detached".into(),
        exit: 130,
        data: json!({"continuation_submitted":false,"message":"Interrupted before submission; no continuation mutation was sent. Existing engine jobs were not cancelled."}),
    }
}
fn prerequisite(client: &Client, operation: Operation, stop: &AtomicBool) -> Result<Output> {
    let mut out = submit(client, operation, None, &Options::default(), stop)?;
    out.data["phase"] = json!("configuration_prerequisite");
    out.data["continuation_submitted"] = json!(false);
    if stop.load(Ordering::Acquire) && out.exit == 0 {
        out.code = "wait_detached".into();
        out.exit = 130;
        out.data["message"] = json!(
            "Prerequisite finished, but the command was interrupted. No continuation mutation was submitted; no job was cancelled."
        );
    }
    Ok(out)
}
fn save_new(path: &Path, value: &impl Serialize) -> Result<()> {
    use std::io::Write;
    let bytes = serde_json::to_vec_pretty(value)?;
    ensure!(
        bytes.len() <= MAX_FRAME,
        "invalid_input: output exceeds bounded file budget"
    );
    let mut o = std::fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    let mut f = o
        .open(path)
        .context("invalid_path: output must be a new file in an existing directory")?;
    #[cfg(windows)]
    regressee_core::platform::protect(path)?;
    f.write_all(&bytes)?;
    f.sync_all()?;
    #[cfg(unix)]
    std::fs::File::open(
        path.parent()
            .filter(|p| !p.as_os_str().is_empty())
            .unwrap_or(Path::new(".")),
    )?
    .sync_all()?;
    Ok(())
}
fn envelope(output: &Output) -> Value {
    json!({"schema_version":1,"ok":output.exit<=1,"code":output.code,"exit_status":output.exit,"data":output.data})
}
fn plaintext_destination(path: &Path) -> Result<()> {
    let extension = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    ensure!(
        ["txt", "json", "jsonl", "ndjson"].contains(&extension.as_str()),
        "invalid_path: DOM source/diff saves require .txt, .json, .jsonl or .ndjson, never executable HTML/SVG/Markdown previews"
    );
    Ok(())
}
fn persist_output(mut output: Output, destination: Option<&Path>) -> Output {
    let Some(path) = destination else {
        return output;
    };
    // Capture DTOs explicitly preserve incomplete/paused/cancelled run evidence.
    // Other failures/uncertain or detached waits must retain their *whole*
    // diagnostic envelope, not a scalar error, null or apparently successful DTO.
    let capture_evidence =
        output.exit == 3 && output.data["job"]["outcome"]["result"] == "Captured";
    let saved = if let Some(ticket) = output.data.get("ticket").filter(|v| !v.is_null()) {
        ticket.clone()
    } else if output.exit <= 1 || capture_evidence {
        output
            .data
            .get("value")
            .filter(|v| !v.is_null())
            .unwrap_or(&output.data)
            .clone()
    } else {
        envelope(&output)
    };
    let focused_dom = output.data["job"]["outcome"]["result"] == "Dom"
        && matches!(
            output.data["job"]["outcome"]["value"]["kind"].as_str(),
            Some("Source" | "Diff")
        );
    let write = (|| -> Result<()> {
        if focused_dom {
            plaintext_destination(path)?;
        }
        save_new(path, &saved)
    })();
    match write {
        Ok(()) => {}
        Err(error) => {
            if !output.data.is_object() {
                output.data = json!({"value": output.data});
            }
            output.data["command_outcome"] =
                json!({"code":output.code,"exit_status":output.exit,"ok":output.exit<=1});
            output.data["output_warning"] = json!(format!(
                "Operation outcome and request recovery are retained, but --save failed: {error:#}"
            ));
            output.code = "output_write_failed".into();
            output.exit = 6;
        }
    }
    output
}
fn run(args: &[String], o: &mut Options) -> Result<Output> {
    let mut a = Args::parse(args).context(InvalidRequest)?;
    o.json = a.flag("--json");
    o.data = a.take("--data-dir").map(PathBuf::from);
    o.cache = a.take("--cache-dir").map(PathBuf::from);
    if a.flag("--help")
        || a.words.first().is_some_and(|s| s == "help")
        || (a.words.is_empty() && args.iter().all(|s| s == "--json"))
    {
        return Output::ok(json!({"help":commands::HELP}));
    }
    if a.flag("--version") {
        a.done().context(InvalidRequest)?;
        return Output::ok(json!({"version":env!("CARGO_PKG_VERSION"),"protocol_version":VERSION}));
    }
    if a.flag("--build-info") {
        a.done().context(InvalidRequest)?;
        let mut info = regressee_app::launch::build_info();
        info["executable"] = "regressee".into();
        info["version"] = env!("CARGO_PKG_VERSION").into();
        return Output::ok(info);
    }
    if a.words == ["capabilities"] {
        a.done().context(InvalidRequest)?;
        return Output::ok(
            json!({"cli_schema_version":1,"protocol_version":VERSION,"product_workflows":true,"capture_modes":["screenshots","both","dom"],"default_kind":"screenshot","capture_check_kind":"frozen_plan","independent_kind_reviews":true,"explicit_kind_approvals":true,"dom_source_page_max_bytes":regressee_app::engine::text::MAX_SOURCE_PAGE_BYTES,"dom_diff_page_max_rows":regressee_app::engine::text::MAX_DIFF_PAGE_ROWS,"dom_results_max_json_bytes":regressee_app::engine::text::MAX_RESULT_PAGE_BYTES,"max_read_result_bytes":MAX_READ_RESULT_BYTES,"dom_source_offsets":"utf8_bytes","dom_diff_offsets":"emitted_rows","static_local_reports":true,"crop_coordinate_space":"screenshot_pixels","headless":"public_sites_only","desktop_login":true,"agent_sessions":true,"agent_follow":true,"agent_follow_policy":"gui_opt_in_only","human_handoff":"full_temporary_gui_control","headless_handoff":"agent_paused_action_required_gui_follow_or_session_end","lease_seconds":{"default":90,"minimum":10,"maximum":300},"upload":false,"max_input_bytes":commands::MAX_INPUT,"max_results_page":200,"commands":commands::HELP}),
        );
    }
    o.detach = a.flag("--detach");
    o.prepare = a.flag("--prepare");
    o.save = a.take("--save").map(PathBuf::from);
    if let Some(path) = &o.save {
        o.save_rejected = true;
        (|| -> Result<()> {
            ensure!(
                std::fs::symlink_metadata(path)
                    .is_err_and(|e| e.kind() == std::io::ErrorKind::NotFound),
                "invalid_path: --save must name a new file; no command was submitted"
            );
            let parent = path
                .parent()
                .filter(|p| !p.as_os_str().is_empty())
                .unwrap_or(Path::new("."));
            ensure!(
                parent.is_dir(),
                "invalid_path: --save parent must already exist; no command was submitted"
            );
            Ok(())
        })()
        .context(InvalidRequest)?;
        o.save_rejected = false;
    }
    o.session = a.optional_id("--session").context(InvalidRequest)?;
    // jobs retry owns this flag: it never regenerates current engine/epoch fields.
    if a.words != ["jobs", "retry"] {
        o.request = a
            .take("--request-id")
            .map(|s| {
                let id = Uuid::parse_str(&s)?;
                ensure!(!id.is_nil(), "request-id must not be nil");
                Ok(id)
            })
            .transpose()
            .context(InvalidRequest)?;
    }
    let plan = (|| -> Result<Plan> {
        ensure!(
            !(o.prepare && o.detach),
            "invalid_arguments: --prepare must finish to return a ticket; do not detach it"
        );
        let plan = commands::plan(&mut a)?;
        a.done()?;
        plan.validate_local()?;
        if matches!(
            &plan,
            Plan::Operation(Operation::Dom(
                regressee_app::engine::dom::Command::Source { .. }
                    | regressee_app::engine::dom::Command::Diff { .. }
            ))
        ) && let Some(path) = &o.save
        {
            o.save_rejected = true;
            plaintext_destination(path)?;
            o.save_rejected = false;
        }
        if matches!(
            &plan,
            Plan::Agent(
                regressee_app::control::Command::Renew { .. }
                    | regressee_app::control::Command::End
                    | regressee_app::control::Command::Handoff { .. }
            ) | Plan::Operation(Operation::SelectAgent(_))
        ) {
            ensure!(
                o.session.is_some(),
                "--session is required for agent control/selection"
            );
        }
        ensure!(
            !matches!(
                &plan,
                Plan::Agent(regressee_app::control::Command::Start { .. })
            ) || o.session.is_none(),
            "start creates a new session; omit --session"
        );
        let can_prepare = match &plan {
            Plan::Operation(op) => op.needs_confirmation(),
            Plan::Configure { replacement, .. } => *replacement,
            _ => false,
        };
        ensure!(
            !o.prepare || can_prepare,
            "--prepare applies only to consequential actions (replacement is explicit)"
        );
        if !matches!(
            &plan,
            Plan::Operation(_)
                | Plan::Configure { .. }
                | Plan::SelectEnvironment { .. }
                | Plan::Confirm { .. }
                | Plan::Appearance(_)
        ) {
            ensure!(
                !o.prepare && o.request.is_none() && (!o.detach || matches!(&plan, Plan::Retry(_))),
                "invalid_arguments: submit flags are not applicable to this command"
            );
        }
        Ok(plan)
    })()
    .context(InvalidRequest)?;
    if let Plan::Confirm { ticket, id } = &plan {
        ensure!(
            *id == ticket.confirmation.id
                && ticket.operation.needs_confirmation()
                && ticket.operation.fingerprint()? == ticket.confirmation.fingerprint,
            "confirmation_conflict: ticket ID/action/payload changed"
        );
    }
    if let Plan::Integration { action, host } = &plan {
        ensure!(o.session.is_none(), "invalid_arguments: integrations setup is local to your account; omit --session");
        let installer = regressee_app::agent_integrations::Installer::for_user(&paths(o)?, std::env::current_exe()?)?;
        let statuses = match action.as_str() {
            "install" => vec![installer.install(host.unwrap())?],
            "remove" => vec![installer.remove(host.unwrap())?],
            _ => host.map_or_else(|| installer.statuses(), |host| vec![installer.status(host)]),
        };
        return Output::ok(json!({"integrations":statuses,"scope":"user","host_access_tested":false,"message":"Local skill setup only. No host permissions, agent session or Follow agent setting was changed.","next_step":host.map(|h| h.next_step())}));
    }
    let stop = interrupted();
    let paths = paths(o)?;
    if matches!(plan, Plan::Serve) {
        ensure!(
            !o.detach && !o.prepare,
            "invalid_arguments: engine serve is foreground"
        );
        regressee_app::engine::Server::open(paths)?.run()?;
        return Output::ok(json!({"message":"Engine stopped"}));
    }
    let existing = a.words.first().is_some_and(|s| s == "engine")
        && matches!(&plan, Plan::Request(Request::Status | Request::Stop));
    let client = if existing {
        Client::existing(paths)?
    } else {
        let result = Client::autostart_while(paths, Launcher::current()?, || {
            !stop.load(Ordering::Acquire)
        });
        match result {
            Ok(client) => client,
            Err(_) if stop.load(Ordering::Acquire) => return Ok(not_submitted()),
            Err(e) => return Err(e),
        }
    };
    let client = if let Some(id) = o.session {
        let path = client
            .paths
            .data
            .join("agent-sessions")
            .join(format!("{id}.json"));
        ownership::private_file(&path, 4096)?;
        let credential: regressee_app::control::Credential =
            serde_json::from_slice(&std::fs::read(path)?)?;
        ensure!(
            credential.owner == regressee_app::control::Owner::Agent(id),
            "authority_conflict: credential identity differs"
        );
        client.with_credential(credential)
    } else {
        client
    };
    match plan {
        Plan::Agent(command) => {
            let handoff = matches!(command, regressee_app::control::Command::Handoff { .. });
            let shown = if let regressee_app::control::Command::Show { session } = &command {
                Some(*session)
            } else {
                None
            };
            match client.control(command)? {
                Response::Credential(credential) => {
                    let regressee_app::control::Owner::Agent(id) = credential.owner else {
                        bail!("internal: unexpected frontend credential")
                    };
                    let directory = client.paths.data.join("agent-sessions");
                    if !directory.exists() {
                        let mut b = std::fs::DirBuilder::new();
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::DirBuilderExt;
                            b.mode(0o700);
                        }
                        b.create(&directory).or_else(|e| {
                            if e.kind() == std::io::ErrorKind::AlreadyExists {
                                Ok(())
                            } else {
                                Err(e)
                            }
                        })?;
                        #[cfg(windows)]
                        regressee_core::platform::protect(&directory)?;
                    }
                    ownership::private_directory(&directory)?;
                    let path = directory.join(format!("{id}.json"));
                    ownership::write_json(&path, &credential)?;
                    Output::ok(
                        json!({"session":id,"credential_file":path,"control":client.uncredentialed().control(regressee_app::control::Command::List)?,"message":"Session survives individual CLI exits. Renew before expiry; GUI following requires explicit local opt-in."}),
                    )
                }
                Response::Control(mut control) => {
                    if let Some(id) = shown {
                        control.sessions.retain(|s| s.id == id);
                    }
                    let following = control
                        .sessions
                        .iter()
                        .find(|s| Some(s.id) == control.controller)
                        .and_then(|s| s.follow);
                    let mut output = Output::ok(control)?;
                    if handoff {
                        output.data["human_input"] = json!({"required":true,"agent_paused":true,"following_frontend":following,"return_to_agent":"gui_only","next_action":if following.is_some() {"Complete the human task, save drafts and choose Return to agent in the GUI."} else {"No GUI is following. Open the desktop on the same data/cache directories and explicitly Follow this named session, or end the session. The agent remains paused."}});
                        if following.is_none() {
                            output.code = "action_required".into();
                            output.exit = 4;
                        }
                    }
                    Ok(output)
                }
                _ => bail!("internal: unexpected agent response"),
            }
        }
        Plan::Operation(op) => submit(&client, op, None, o, &stop),
        Plan::Configure {
            project,
            revision,
            field,
            value,
            replacement,
        } => {
            let read = prerequisite(&client, Operation::Load(project), &stop)?;
            if read.exit != 0 {
                return Ok(read);
            }
            let p: regressee_storage::ProjectRecord =
                serde_json::from_value(read.data["value"].clone())?;
            ensure!(
                p.revision == revision,
                "revision_conflict: reload settings before editing"
            );
            let mut v = serde_json::to_value(&p.project)?;
            v[&field] = value;
            let project: Project = serde_json::from_value(v)?;
            project.validate().context(InvalidRequest)?;
            submit(
                &client,
                if replacement {
                    Operation::SaveReplacement {
                        project: Box::new(project),
                        expected_revision: revision,
                    }
                } else {
                    Operation::Save {
                        project: Box::new(project),
                        expected_revision: Some(revision),
                    }
                },
                None,
                o,
                &stop,
            )
        }
        Plan::SelectEnvironment {
            project,
            revision,
            environment,
            settings,
        } => {
            let read = prerequisite(&client, Operation::Load(project), &stop)?;
            if read.exit != 0 {
                return Ok(read);
            }
            let p: regressee_storage::ProjectRecord =
                serde_json::from_value(read.data["value"].clone())?;
            ensure!(p.revision == revision, "revision_conflict: reload settings");
            let settings = if let Some(s) = settings {
                s
            } else {
                let read = prerequisite(&client, Operation::EnvironmentStatus(project), &stop)?;
                if read.exit != 0 {
                    return Ok(read);
                }
                let s: regressee_app::auth::EnvironmentStatus =
                    serde_json::from_value(read.data["value"].clone())?;
                s.settings
            };
            submit(
                &client,
                Operation::SaveEnvironment {
                    project: Box::new(p.project),
                    revision,
                    environment,
                    settings,
                },
                None,
                o,
                &stop,
            )
        }
        Plan::Request(r) => {
            let response = request(&client, r)?;
            if let Response::Job(j) = response {
                result(*j)
            } else {
                let value = serde_json::to_value(response)?;
                Output::ok(value.get("value").unwrap_or(&value))
            }
        }
        Plan::Appearance(value) => submit(&client, Operation::SetAppearance(value), None, o, &stop),
        Plan::Wait(id) => wait(&client, job(&client, id)?, &stop),
        Plan::Retry(id) => {
            if stop.load(Ordering::Acquire) {
                return Ok(not_submitted());
            }
            let path = spool(&client, id)?;
            ownership::private_file(&path, commands::MAX_INPUT)?;
            let original: Submit = serde_json::from_slice(&std::fs::read(&path)?)?;
            ensure!(
                original.request == id,
                "request_conflict: saved request identity mismatch"
            );
            eprintln!("Retry request {id} — {}", path.display());
            let j = match client.submit(original) {
                Ok(j) => j,
                Err(e) if e.is::<Rejected>() => return Err(e),
                Err(e) => {
                    return Err(SubmitLost {
                        request: id,
                        path,
                        message: format!("Retry transport disconnected: {e:#}."),
                    }
                    .into());
                }
            };
            let mut output = if o.detach {
                result(j)?
            } else {
                wait(&client, j, &stop)?
            };
            output.data["request_id"] = json!(id);
            output.data["retry_file"] = json!(path);
            Ok(output)
        }
        Plan::Confirm { ticket, id } => {
            ensure!(
                !o.prepare,
                "invalid_arguments: confirm cannot prepare a different action"
            );
            ensure!(
                id == ticket.confirmation.id,
                "confirmation_conflict: use the exact confirmation ID printed in this ticket"
            );
            ensure!(
                ticket.operation.needs_confirmation()
                    && ticket.operation.fingerprint()? == ticket.confirmation.fingerprint,
                "confirmation_conflict: ticket action/payload changed"
            );
            submit(&client, ticket.operation, Some(id), o, &stop)
        }
        Plan::Start => Output::ok(client.snapshot()?),
        Plan::Serve => unreachable!(),
        Plan::Integration { .. } => unreachable!(),
    }
}
fn main() -> ExitCode {
    let raw = std::env::args_os().skip(1).collect::<Vec<_>>();
    let json = raw.iter().any(|s| s == "--json");
    let args = raw
        .into_iter()
        .map(|s| {
            s.into_string()
                .map_err(|_| anyhow::anyhow!("invalid_arguments: CLI arguments must be UTF-8"))
        })
        .collect::<Result<Vec<_>>>();
    let mut options = Options {
        json,
        ..Default::default()
    };
    let output=args.and_then(|args|run(&args,&mut options)).unwrap_or_else(|e|{if let Some(lost)=e.downcast_ref::<SubmitLost>(){return Output{code:"admission_uncertain".into(),exit:7,data:json!({"request_id":lost.request,"retry_file":lost.path,"message":lost.to_string()})};}let message=format!("{e:#}");let (code,exit)=if e.is::<InvalidRequest>() {("invalid_request",2)} else if let Some(rejected)=e.downcast_ref::<Rejected>() {let tagged=error_code(&rejected.code);if tagged.0=="operation_failed"{error_code(&rejected.message)}else{tagged}}else{error_code(&message)};Output{code:code.into(),exit,data:json!({"message":message})}});
    let output = if options.save_rejected {
        output // Invalid destination: preserve pre-admission exit 2 and never overwrite.
    } else {
        persist_output(output, options.save.as_deref())
    };
    if options.json {
        println!("{}", envelope(&output));
    } else if let Some(help) = output.data.get("help").and_then(Value::as_str) {
        println!("{help}");
    } else if output.data.get("features").is_some() {
        println!("{}", serde_json::to_string(&output.data).unwrap());
    } else if let Some(version) = output.data.get("version").and_then(Value::as_str) {
        println!("regressee {version}");
    } else {
        let mut text = if let Some(j) = output.data.get("job") {
            format!(
                "{} — job {} ({})\n{}",
                output.code,
                j["id"].as_str().unwrap_or("?"),
                j["state"].as_str().unwrap_or("?"),
                serde_json::to_string_pretty(
                    output
                        .data
                        .get("ticket")
                        .or_else(|| output.data.get("value"))
                        .unwrap_or(&output.data)
                )
                .unwrap()
            )
        } else {
            format!(
                "{}\n{}",
                output.code,
                serde_json::to_string_pretty(&output.data).unwrap()
            )
        };
        if let Some(warning) = output.data.get("output_warning").and_then(Value::as_str) {
            text.push_str(&format!(
                "\n{warning}\nOriginal command outcome: {}",
                output.data["command_outcome"]
            ));
        }
        if output.exit > 1 {
            eprintln!("{text}");
        } else {
            println!("{text}");
        }
    }
    ExitCode::from(output.exit)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn detailed_save_failures_keep_the_storage_write_exit_code() {
        assert_eq!(
            error_code(
                "storage_write_failed: The selected environment does not belong to this project."
            ),
            ("storage_write_failed", 6)
        );
    }

    #[test]
    fn recovered_dom_body_cannot_be_saved_as_an_active_preview() -> Result<()> {
        let root = tempfile::tempdir()?;
        for extension in ["html", "htm", "svg", "xhtml", "md"] {
            let path = root.path().join(format!("unsafe.{extension}"));
            let output = Output {
                code: "ok".into(),
                exit: 0,
                data: json!({"job":{"outcome":{"result":"Dom","value":{"kind":"Source"}}},"value":{"page":{"text":"</pre></script><script>evil()</script>"}}}),
            };
            let output = persist_output(output, Some(&path));
            assert_eq!(output.exit, 6);
            assert_eq!(output.data["command_outcome"]["exit_status"], 0);
            assert!(!path.exists());
        }
        for extension in ["txt", "json", "jsonl", "ndjson"] {
            plaintext_destination(&root.path().join(format!("safe.{extension}")))?;
        }
        Ok(())
    }
    #[test]
    fn requested_kind_failure_precedes_either_change_and_disabled_kinds_are_absent() {
        for outcomes in [
            vec![Some(true), None],
            vec![None, Some(true)],
            vec![Some(false), None],
            vec![],
        ] {
            assert_eq!(
                classify_check_outcomes(true, &outcomes),
                ("incomplete_capture", 3)
            );
        }
        assert_eq!(
            classify_check_outcomes(true, &[Some(false), Some(true)]),
            ("check_differences", 1)
        );
        assert_eq!(
            classify_check_outcomes(true, &[Some(false), Some(false)]),
            ("ok", 0)
        );
        // Disabled siblings are not represented as failed/unchanged verdicts.
        assert_eq!(classify_check_outcomes(true, &[Some(false)]), ("ok", 0));
        assert_eq!(
            classify_check_outcomes(false, &[Some(true)]),
            ("visual_differences", 1)
        );
    }
    #[test]
    fn explicit_save_preserves_every_exit_family_and_uncertain_diagnostics() -> Result<()> {
        let root = tempfile::tempdir()?;
        for (code, exit) in [
            ("ok", 0),
            ("visual_differences", 1),
            ("invalid_request", 2),
            ("incomplete_capture", 3),
            ("action_required", 4),
            ("conflict", 5),
            ("operation_failed", 6),
            ("completion_uncertain", 7),
            ("wait_detached", 130),
        ] {
            let data = json!({"request_id":Uuid::new_v4(),"retry_file":"retained-original.json",
                "value":{"latest":{"run":{"state":"CompletedWithErrors"}}},
                "job":{"state":if exit==7 {"Interrupted"} else {"Failed"},"outcome":{"result":"Captured"}},
                "message":"Keep diagnostics and frozen identity","continuation_submitted":false});
            let output = Output {
                code: code.into(),
                exit,
                data: data.clone(),
            };
            let expected = if exit <= 1 || exit == 3 {
                data["value"].clone()
            } else {
                envelope(&output)
            };
            let file = root.path().join(format!("{exit}.json"));
            let output = persist_output(output, Some(&file));
            assert_eq!(output.exit, exit);
            assert_eq!(output.code, code);
            assert_eq!(output.data, data);
            assert_eq!(
                serde_json::from_slice::<Value>(&std::fs::read(&file)?)?,
                expected
            );
            let collision = persist_output(output, Some(&file));
            assert_eq!(collision.exit, 6);
            assert_eq!(collision.data["command_outcome"]["exit_status"], exit);
            assert_eq!(collision.data["command_outcome"]["code"], code);
            assert_eq!(
                serde_json::from_slice::<Value>(&std::fs::read(&file)?)?,
                expected
            );
        }
        // Direct scalar DTOs (appearance) and detached receipts must not panic
        // or save null. A failed save retains the original result too.
        for data in [
            json!("dark"),
            json!({"job":{"state":"Queued"},"value":null,"request_id":"original"}),
        ] {
            let output = Output {
                code: "ok".into(),
                exit: 0,
                data: data.clone(),
            };
            let path = root.path().join(format!("{}.json", Uuid::new_v4()));
            let output = persist_output(output, Some(&path));
            assert_eq!(
                serde_json::from_slice::<Value>(&std::fs::read(&path)?)?,
                data
            );
            let failed = persist_output(output, Some(&path));
            assert_eq!(failed.exit, 6);
            assert_eq!(failed.data["command_outcome"]["exit_status"], 0);
            assert_eq!(
                serde_json::from_slice::<Value>(&std::fs::read(&path)?)?,
                data
            );
        }
        Ok(())
    }
    #[test]
    fn interrupted_receipt_uncertainty_takes_precedence_over_its_failure_message() -> Result<()> {
        let id = Uuid::new_v4();
        let j = Job {
            owner: None,
            id,
            submitted: Submit {
                request: id,
                engine: Uuid::new_v4(),
                epoch: 0,
                operation: Operation::List { page: 0 },
                confirmation: None,
                expected_revision: None,
            },
            state: JobState::Interrupted,
            frozen: None,
            frozen_revision: None,
            progress: None,
            capture: None,
            outcome: Some(Outcome::ProductFailure(
                "Operation may have committed before engine crash".into(),
            )),
            sequence: 0,
        };
        let output = result(j)?;
        assert_eq!(output.exit, 7);
        assert_eq!(output.code, "completion_uncertain");
        Ok(())
    }
}
