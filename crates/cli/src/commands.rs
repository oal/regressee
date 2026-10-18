// SPDX-License-Identifier: MPL-2.0
use anyhow::{Context, Result, bail, ensure};
use regressee_app::{
    analysis_engine::{AnalysisRequest, Policy, Preset},
    engine::dom,
    protocol::Operation,
    terminal::{self, Selection},
};
use regressee_core::{appearance::AppAppearance, crawl::DiscoveryPreview, domain::*};
use regressee_storage::*;
use serde::de::DeserializeOwned;
use std::{collections::BTreeMap, io::Read, path::PathBuf};
use uuid::Uuid;
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct EnvironmentSave {
    project: Project,
    revision: u64,
    environment: EnvironmentId,
    settings: LocalLoginSettings,
}

pub const MAX_INPUT: u64 = 8 * 1024 * 1024;
pub const HELP: &str = "Regressee — local visual evidence from the terminal

Usage: regressee [GLOBAL OPTIONS] <AREA> <ACTION> [OPTIONS]
  --json                       Versioned JSON on stdout (including errors)
  --data-dir DIR --cache-dir DIR  Same defaults as the desktop
  --request-id UUID             Retry identity; original submit is saved before IPC
  --detach                     Return admitted job, not a completed verdict
  --prepare                    Prepare exact consequential action, never commit
  --save FILE                   Save returned value/ticket to a NEW file
  --session UUID                Use a private named agent session credential

Commands:
  project create --name NAME --url URL [--mode screenshots|both|dom] | --file PROJECT.json
  project list [--page N] | show --project ID
  project update --project ID --expected-revision N --file PROJECT.json [--replacement]
  profiles|environments|scope show --project ID
  profiles|environments|scope set --project ID --expected-revision N --file JSON [--replacement]
  checks show --project ID
  checks set --project ID --expected-revision N --mode screenshots|both|dom [--replacement]
  environment status --project ID
  environment select --project ID --environment ID --expected-revision N [--login-settings JSON]
  environment save --file ENVIRONMENT_SAVE.json [--replacement]
  discovery start|existing --project ID
  discovery inventory (--job ID | --project ID)
  discovery status --job ID
  discovery include|exclude --file INVENTORY.json --pages ID,ID
  capture baseline --project ID --file INVENTORY.json
  capture preview --project ID --file INVENTORY.json
  capture check --project ID           Fixed active-baseline list, no discovery
  capture resume --project ID --run ID
  jobs list [--offset N --limit N] | show|wait|pause|cancel|ack --job ID
  jobs retry --request-id UUID         Reuse saved COMPLETE original submit
  results list --project ID [--kind screenshot|dom --run ID --analysis ID --page ID --profile ID --outcome CLASS --capture-state STATE --offset N --limit N]
  results artifacts --project ID --page ID --profile ID [--kind screenshot|dom --run ID --analysis ID]
  snapshot source --project ID --page ID --profile ID [--run ID --analysis ID --pane before|after --format plaintext|canonical --offset BYTE --limit BYTES]
  snapshot diff --project ID --page ID --profile ID [--run ID --analysis ID --context N --offset ROW --limit ROWS]
  analysis show|select --analysis ID | reanalyze --run ID [--preset sensitive|balanced|relaxed | --file POLICY.json]
  analysis show|select --kind dom --analysis ID | reanalyze --kind dom --run ID [--context N]
  masks save --project ID --file ANALYSIS_REQUEST.json
  review set --analysis ID --page ID --profile ID --state unreviewed|intentional|needs-fix [--kind screenshot|dom --note TEXT | --note-file FILE]
  review history --analysis ID [--kind screenshot|dom --page ID --profile ID --before DECISION_ID --limit N]
  baseline preview --analysis ID --file SELECTED_KEYS.json [--kind screenshot|dom]
  baseline preview-kinds --file EXPLICIT_KIND_SELECTION.json
  baseline save --run ID --expected-revision N [--partial] --prepare
  baseline accept --file APPROVAL.json [--kind screenshot|dom] --prepare
  baseline accept-kinds --file KIND_APPROVAL.json --prepare
  baseline restore --baseline ID --expected-revision N --prepare
  history list --project ID [--page N] | show --project ID --run ID [--kind screenshot|dom]
  storage usage [--project ID] | delete-preview --project ID (--run ID | --baseline ID)
  storage delete --file PREVIEW.json --prepare
  repair preview | clean|delete-unreferenced --file PREVIEW.json --prepare
  archive export-preview --project ID --baseline ID
  archive export --file PREVIEW.json --destination ZIP --prepare
  archive import-preview --source ZIP
  archive import --file PREVIEW.json --source ZIP --prepare
  report preview --project ID [--run ID --analysis SCREENSHOT_ID --dom-analysis DOM_ID] --file SELECTED_KEYS.json --destination DIR
  report export --file PREVIEW.json --prepare
  crop preview --project ID --page ID --profile ID [--run ID --analysis ID] --pane before|after|difference --rect X,Y,W,H --space screenshot-pixels --destination PNG
  crop export --file PREVIEW.json --prepare
  diagnostics preview | export --file PREVIEW.json --destination FILE --prepare
  browser status|setup | network status|set --file JSON
  login status|open|confirm|close|forget|probe --project ID (forget requires --prepare)
  appearance show|set [--value system|light|dark]
  confirm --file TICKET.json --confirmation UUID
  engine start|serve|status|stop
  agent start --name NAME [--lease SECONDS] | list | show --id UUID
  agent renew [--lease SECONDS] | end | handoff --message TEXT   (require --session)
  agent select --project ID [--kind screenshot|dom --run ID --analysis ID --page ID --profile ID --view compare|before|after|difference|overlay|split] (requires --session; DOM has no overlay/split)
  integrations status [--host claude-code|codex|github-copilot]
  integrations install|remove --host claude-code|codex|github-copilot
  capabilities | --version | --build-info | help [AREA]

Complex JSON accepts --file - for bounded stdin. No interactive prompts or --yes.
Kind-specific commands default to screenshot; capture/check obey the frozen plan.
Source offsets are UTF-8 bytes (4–262144 bytes/page); diff pages are 1–1000 rows.
Read-only commands do not install a browser. Public sites work headlessly; login
requires a desktop. Ctrl-C detaches waiting, never cancels the engine job.
Following is GUI-only opt-in. Take control immediately revokes the agent; handoff
pauses agent writes/navigation until the human explicitly returns control.
See docs/cli.md for schemas, evidence privacy, recovery and exit statuses.";

#[derive(Default)]
pub struct Args {
    pub words: Vec<String>,
    options: BTreeMap<String, String>,
}
impl Args {
    pub fn parse(args: &[String]) -> Result<Self> {
        let mut result = Self::default();
        let mut i = 0;
        while i < args.len() {
            let arg = &args[i];
            if arg.starts_with("--") {
                let (key, value) = if [
                    "--json",
                    "--detach",
                    "--prepare",
                    "--replacement",
                    "--partial",
                    "--help",
                    "--version",
                    "--build-info",
                ]
                .contains(&arg.as_str())
                {
                    (arg.clone(), "true".into())
                } else {
                    let value = args
                        .get(i + 1)
                        .context(format!("invalid_arguments: {arg} requires a value"))?;
                    ensure!(
                        !value.starts_with("--"),
                        "invalid_arguments: {arg} requires a value"
                    );
                    i += 1;
                    (arg.clone(), value.clone())
                };
                ensure!(
                    result.options.insert(key.clone(), value).is_none(),
                    "invalid_arguments: duplicate {key}"
                );
            } else {
                result.words.push(arg.clone());
            }
            i += 1;
        }
        Ok(result)
    }
    pub fn take(&mut self, key: &str) -> Option<String> {
        self.options.remove(key)
    }
    pub fn flag(&mut self, key: &str) -> bool {
        self.take(key).is_some()
    }
    pub fn need(&mut self, key: &str) -> Result<String> {
        self.take(key)
            .context(format!("invalid_arguments: required {key}"))
    }
    pub fn number(&mut self, key: &str, default: usize) -> Result<usize> {
        self.take(key)
            .map(|s| {
                s.parse().context(format!(
                    "invalid_arguments: {key} must be a nonnegative integer"
                ))
            })
            .unwrap_or(Ok(default))
    }
    pub fn uuid(&mut self, key: &str) -> Result<Uuid> {
        let id = Uuid::parse_str(&self.need(key)?)
            .context(format!("invalid_arguments: {key} must be a UUID"))?;
        ensure!(!id.is_nil(), "invalid_arguments: {key} must not be nil");
        Ok(id)
    }
    pub fn optional_id<T: DeserializeOwned>(&mut self, key: &str) -> Result<Option<T>> {
        self.take(key).map(|s| id(&s)).transpose()
    }
    pub fn id<T: DeserializeOwned>(&mut self, key: &str) -> Result<T> {
        id(&self.need(key)?)
    }
    pub fn file<T: DeserializeOwned>(&mut self) -> Result<T> {
        input(&self.need("--file")?)
    }
    pub fn strict_file<T: DeserializeOwned + serde::Serialize>(&mut self) -> Result<T> {
        strict_input(&self.need("--file")?)
    }
    pub fn done(&self) -> Result<()> {
        ensure!(
            self.options.is_empty(),
            "invalid_arguments: unknown/inapplicable options: {}",
            self.options.keys().cloned().collect::<Vec<_>>().join(", ")
        );
        Ok(())
    }
}
pub fn id<T: DeserializeOwned>(s: &str) -> Result<T> {
    ensure!(
        !Uuid::parse_str(s)?.is_nil(),
        "invalid_arguments: entity ID must not be nil"
    );
    serde_json::from_value(serde_json::Value::String(s.into()))
        .context("invalid_arguments: invalid entity ID")
}
pub fn bytes(path: &str) -> Result<Vec<u8>> {
    let mut bytes = Vec::new();
    if path == "-" {
        std::io::stdin()
            .take(MAX_INPUT + 1)
            .read_to_end(&mut bytes)?;
    } else {
        let m = std::fs::symlink_metadata(path)?;
        ensure!(
            m.is_file() && regressee_core::platform::ordinary(&m) && m.len() <= MAX_INPUT,
            "invalid_input: expected ordinary file at most 8 MiB"
        );
        std::fs::File::open(path)?
            .take(MAX_INPUT + 1)
            .read_to_end(&mut bytes)?;
    }
    ensure!(
        bytes.len() as u64 <= MAX_INPUT,
        "invalid_input: input exceeds 8 MiB"
    );
    Ok(bytes)
}
pub fn input<T: DeserializeOwned>(path: &str) -> Result<T> {
    let v: serde_json::Value =
        serde_json::from_slice(&bytes(path)?).context("invalid_input: expected JSON")?;
    // Accept raw DTOs and our documented envelope/save output, never arbitrary RPC.
    for candidate in [
        Some(&v),
        v.pointer("/data/value"),
        v.pointer("/data/ticket"),
        v.pointer("/value"),
        v.pointer("/data/value/value"),
    ]
    .into_iter()
    .flatten()
    {
        if let Ok(value) = serde_json::from_value(candidate.clone()) {
            return Ok(value);
        }
        if let Some(p) = candidate.get("project")
            && let Ok(value) = serde_json::from_value(p.clone())
        {
            return Ok(value);
        }
        if let Some(p) = candidate.get("value")
            && let Ok(value) = serde_json::from_value(p.clone())
        {
            return Ok(value);
        }
    }
    bail!("invalid_input: JSON does not match the command's documented input schema")
}
fn strict_input<T: DeserializeOwned + serde::Serialize>(path: &str) -> Result<T> {
    let value: serde_json::Value =
        serde_json::from_slice(&bytes(path)?).context("invalid_input: expected JSON")?;
    for candidate in [
        Some(&value),
        value.pointer("/data/value"),
        value.pointer("/value"),
        value.pointer("/data/value/value"),
    ]
    .into_iter()
    .flatten()
    {
        if let Ok(parsed) = serde_json::from_value::<T>(candidate.clone()) {
            ensure!(
                serde_json::to_value(&parsed)? == *candidate,
                "invalid_input: approval preview has unknown/missing/changed versioned fields; regenerate it without editing"
            );
            return Ok(parsed);
        }
    }
    bail!("invalid_input: expected the exact returned kind approval preview")
}
pub enum Plan {
    Integration {
        action: String,
        host: Option<regressee_app::agent_integrations::Host>,
    },
    Agent(regressee_app::control::Command),
    Operation(Operation),
    Configure {
        project: ProjectId,
        revision: u64,
        field: String,
        value: serde_json::Value,
        replacement: bool,
    },
    SelectEnvironment {
        project: ProjectId,
        revision: u64,
        environment: EnvironmentId,
        settings: Option<LocalLoginSettings>,
    },
    Request(regressee_app::protocol::Request),
    Wait(Uuid),
    Retry(Uuid),
    Confirm {
        ticket: crate::Ticket,
        id: Uuid,
    },
    Serve,
    Start,
    Appearance(AppAppearance),
}
impl Plan {
    /// Pure/bounded local validation. Anything needing saved revisions, source
    /// geometry or runtime state is still revalidated by the shared engine.
    pub fn validate_local(&self) -> Result<()> {
        use regressee_app::protocol::Request;
        match self {
            Self::Agent(command) => regressee_app::control::validate(command)?,
            Self::Operation(op) => validate_operation(op)?,
            Self::Configure {
                project,
                field,
                value,
                ..
            } => {
                let mut draft = Project::draft(
                    "Configuration validation".into(),
                    "https://validation.invalid/",
                )?;
                draft.id = *project;
                let mut v = serde_json::to_value(draft)?;
                v[field] = value.clone();
                serde_json::from_value::<Project>(v)?.validate()?;
            }
            Self::SelectEnvironment {
                settings: Some(settings),
                ..
            } => settings.validate()?,
            Self::Request(Request::Jobs { offset, limit }) => ensure!(
                *offset <= regressee_app::protocol::MAX_JOBS && (1..=200).contains(limit),
                "jobs limit 1–200; offset at most 4096"
            ),
            _ => {}
        }
        Ok(())
    }
}
fn selected(keys: &[TargetKey], limit: usize) -> Result<()> {
    ensure!(
        !keys.is_empty()
            && keys.len() <= limit
            && keys.iter().collect::<std::collections::BTreeSet<_>>().len() == keys.len()
            && keys
                .iter()
                .all(|k| !k.page.0.is_nil() && !k.profile.0.is_nil()),
        "select 1–{limit} unique non-nil page/profile keys"
    );
    Ok(())
}
fn validate_operation(op: &Operation) -> Result<()> {
    match op {
        Operation::Dom(command) => command.validate()?,
        Operation::SelectAgent(selection) => ensure!(
            selection.kind != CheckKind::DomSnapshot
                || matches!(
                    selection.view,
                    regressee_app::control::View::Compare
                        | regressee_app::control::View::Before
                        | regressee_app::control::View::After
                        | regressee_app::control::View::Difference
                ),
            "DOM text does not support image overlay/split views"
        ),
        Operation::Save { project, .. } | Operation::SaveReplacement { project, .. } => {
            project.validate()?
        }
        Operation::SaveEnvironment {
            project,
            environment,
            settings,
            ..
        }
        | Operation::SaveReplacementEnvironment {
            project,
            environment,
            settings,
            ..
        } => {
            project.validate()?;
            settings.validate()?;
            ensure!(
                project.environments.iter().any(|e| e.id == *environment),
                "selected environment is not in the supplied project"
            );
        }
        Operation::List { page } | Operation::History { page, .. } => {
            ensure!(*page <= 15625, "page must be at most 15625")
        }
        Operation::Reanalyze { policy, .. } => policy.validate()?,
        Operation::SaveNetwork(settings) => settings.validate()?,
        Operation::Mask { request, .. } => {
            request.validate()?;
            ensure!(request.mask.is_some(), "mask binding is required");
        }
        Operation::Approval { selected: keys, .. } => selected(keys, MAX_TARGETS)?,
        Operation::CaptureInventory {
            project,
            preview: Some(preview),
            ..
        }
        | Operation::PreviewImages { project, preview } => {
            preview.settings.validate()?;
            preview.inventory.validate()?;
            ensure!(
                preview.settings.id == *project && preview.inventory.project == *project,
                "inventory/project mismatch"
            );
        }
        Operation::Terminal(terminal::Command::InventoryEdit { preview, pages, .. }) => {
            preview.settings.validate()?;
            preview.inventory.validate()?;
            ensure!(
                !pages.is_empty()
                    && pages.len() <= 10000
                    && pages
                        .iter()
                        .all(|id| preview.inventory.pages.iter().any(|p| p.id == *id)),
                "select existing inventory page IDs"
            );
        }
        Operation::Terminal(terminal::Command::Results { limit, offset, .. }) => ensure!(
            (1..=200).contains(limit) && *offset <= MAX_TARGETS,
            "results limit 1–200; offset at most 30000"
        ),
        Operation::Terminal(terminal::Command::ReviewHistory { limit, before, .. }) => ensure!(
            (1..=200).contains(limit) && before.is_none_or(|id| id > 0),
            "review history limit 1–200; before must be a positive decision ID"
        ),
        Operation::Terminal(terminal::Command::ReportPreview { selected: keys, .. }) => {
            selected(keys, 100)?
        }
        Operation::Terminal(terminal::Command::Report(plan)) => selected(&plan.selected, 100)?,
        Operation::Terminal(terminal::Command::CropPreview { rectangle: r, .. }) => ensure!(
            r.width > 0
                && r.height > 0
                && r.x.checked_add(r.width).is_some()
                && r.y.checked_add(r.height).is_some()
                && u64::from(r.width) * u64::from(r.height) <= 4_000_000,
            "crop must be nonempty, fit unsigned coordinates and contain at most 4M pixels"
        ),
        _ => {}
    }
    Ok(())
}
fn selection(a: &mut Args) -> Result<Selection> {
    Ok(Selection {
        project: a.id("--project")?,
        run: a.optional_id("--run")?,
        analysis: a.optional_id("--analysis")?,
    })
}
fn dom_selection(selection: Selection) -> dom::Selection {
    dom::Selection {
        project: selection.project,
        run: selection.run,
        analysis: selection.analysis.map(|id| DomAnalysisId(id.0)),
    }
}
fn key(a: &mut Args) -> Result<TargetKey> {
    Ok(TargetKey {
        page: a.id("--page")?,
        profile: a.id("--profile")?,
    })
}
fn revision(a: &mut Args) -> Result<u64> {
    a.need("--expected-revision")?
        .parse()
        .context("invalid_arguments: expected revision must be an integer")
}
fn capture_mode(value: &str) -> Result<CaptureMode> {
    Ok(match value {
        "screenshots" => CaptureMode::ScreenshotsOnly,
        "both" => CaptureMode::ScreenshotsAndDomSnapshots,
        "dom" => CaptureMode::DomSnapshotsOnly,
        _ => bail!("invalid_arguments: mode screenshots|both|dom"),
    })
}
fn check_kind(a: &mut Args) -> Result<CheckKind> {
    Ok(match a.take("--kind").as_deref().unwrap_or("screenshot") {
        "screenshot" => CheckKind::Screenshot,
        "dom" => CheckKind::DomSnapshot,
        _ => bail!("invalid_arguments: kind screenshot|dom"),
    })
}
pub fn plan(a: &mut Args) -> Result<Plan> {
    use regressee_app::protocol::Request;
    let words = a.words.iter().map(String::as_str).collect::<Vec<_>>();
    ensure!(
        words.len() == 2 || words == ["confirm"],
        "invalid_arguments: choose AREA ACTION; see --help"
    );
    let area = words[0].to_owned();
    let action = words.get(1).unwrap_or(&"").to_string();
    let operation = match (area.as_str(), action.as_str()) {
        ("integrations", "status" | "install" | "remove") => {
            let host = a
                .take("--host")
                .map(|h| regressee_app::agent_integrations::Host::parse(&h))
                .transpose()?;
            ensure!(
                action == "status" || host.is_some(),
                "invalid_arguments: --host is required for install/remove"
            );
            return Ok(Plan::Integration { action, host });
        }
        ("agent", "start") => {
            return Ok(Plan::Agent(regressee_app::control::Command::Start {
                name: a.need("--name")?,
                seconds: a.number("--lease", 90)? as u64,
            }));
        }
        ("agent", "list") => return Ok(Plan::Agent(regressee_app::control::Command::List)),
        ("agent", "show") => {
            return Ok(Plan::Agent(regressee_app::control::Command::Show {
                session: a.uuid("--id")?,
            }));
        }
        ("agent", "renew") => {
            return Ok(Plan::Agent(regressee_app::control::Command::Renew {
                seconds: a.number("--lease", 90)? as u64,
            }));
        }
        ("agent", "end") => return Ok(Plan::Agent(regressee_app::control::Command::End)),
        ("agent", "handoff") => {
            return Ok(Plan::Agent(regressee_app::control::Command::Handoff {
                message: a.need("--message")?,
            }));
        }
        ("agent", "select") => {
            let page = a.optional_id("--page")?;
            let profile = a.optional_id("--profile")?;
            ensure!(
                page.is_some() == profile.is_some(),
                "page and profile must be supplied together"
            );
            let view = match a.take("--view").as_deref().unwrap_or("compare") {
                "compare" => regressee_app::control::View::Compare,
                "before" => regressee_app::control::View::Before,
                "after" => regressee_app::control::View::After,
                "difference" => regressee_app::control::View::Difference,
                "overlay" => regressee_app::control::View::Overlay,
                "split" => regressee_app::control::View::Split,
                _ => bail!("invalid view"),
            };
            let selection = regressee_app::control::Selection {
                project: a.id("--project")?,
                run: a.optional_id("--run")?,
                kind: check_kind(a)?,
                analysis: a.optional_id("--analysis")?,
                key: page
                    .zip(profile)
                    .map(|(page, profile)| TargetKey { page, profile }),
                view,
            };
            ensure!(
                selection.run.is_some()
                    || (selection.analysis.is_none() && selection.key.is_none()),
                "analysis/result needs exact run"
            );
            Operation::SelectAgent(selection)
        }
        ("engine", "serve") => return Ok(Plan::Serve),
        ("engine", "start") => return Ok(Plan::Start),
        ("engine", "status") => return Ok(Plan::Request(Request::Status)),
        ("engine", "stop") => return Ok(Plan::Request(Request::Stop)),
        ("project", "create") => {
            let mut project = if a.options.contains_key("--file") {
                a.file::<Project>()?
            } else {
                Project::draft(a.need("--name")?, &a.need("--url")?)?
            };
            if let Some(mode) = a.take("--mode") {
                project.capture_plan = CapturePlan::for_mode(capture_mode(&mode)?);
            }
            ensure!(
                project.active_baseline.is_none(),
                "invalid_input: a new project cannot import an active baseline pointer"
            );
            project.validate()?;
            Operation::Save {
                project: Box::new(project),
                expected_revision: None,
            }
        }
        ("project", "list") => Operation::List {
            page: a.number("--page", 0)?,
        },
        ("project", "show") => Operation::Load(a.id("--project")?),
        ("project", "update") => {
            let p: Project = a.file()?;
            let project: ProjectId = a.id("--project")?;
            ensure!(p.id == project, "invalid_input: project ID mismatch");
            let revision = revision(a)?;
            if a.flag("--replacement") {
                Operation::SaveReplacement {
                    project: Box::new(p),
                    expected_revision: revision,
                }
            } else {
                Operation::Save {
                    project: Box::new(p),
                    expected_revision: Some(revision),
                }
            }
        }
        ("profiles" | "environments" | "scope", "show") => Operation::Load(a.id("--project")?),
        ("checks", "show") => Operation::Load(a.id("--project")?),
        ("checks", "set") => {
            let capture_plan = CapturePlan::for_mode(capture_mode(&a.need("--mode")?)?);
            return Ok(Plan::Configure {
                project: a.id("--project")?,
                revision: revision(a)?,
                field: "capture_plan".into(),
                value: serde_json::to_value(capture_plan)?,
                replacement: a.flag("--replacement"),
            });
        }
        ("profiles" | "environments" | "scope", "set") => {
            let field = match area.as_str() {
                "profiles" => "profiles",
                "environments" => "environments",
                _ => "discovery",
            }
            .to_owned();
            let value = match area.as_str() {
                "profiles" => serde_json::to_value(a.file::<Vec<CaptureProfile>>()?)?,
                "environments" => serde_json::to_value(a.file::<Vec<Environment>>()?)?,
                _ => serde_json::to_value(a.file::<DiscoverySettings>()?)?,
            };
            return Ok(Plan::Configure {
                project: a.id("--project")?,
                revision: revision(a)?,
                field,
                value,
                replacement: a.flag("--replacement"),
            });
        }
        ("environment", "select") => {
            return Ok(Plan::SelectEnvironment {
                project: a.id("--project")?,
                revision: revision(a)?,
                environment: a.id("--environment")?,
                settings: a.take("--login-settings").map(|s| input(&s)).transpose()?,
            });
        }
        ("environment", "save") => {
            let s: EnvironmentSave = a.file()?;
            if a.flag("--replacement") {
                Operation::SaveReplacementEnvironment {
                    project: Box::new(s.project),
                    revision: s.revision,
                    environment: s.environment,
                    settings: s.settings,
                }
            } else {
                Operation::SaveEnvironment {
                    project: Box::new(s.project),
                    revision: s.revision,
                    environment: s.environment,
                    settings: s.settings,
                }
            }
        }
        ("environment" | "login", "status") => Operation::EnvironmentStatus(a.id("--project")?),
        ("login", "open") => Operation::OpenLogin(a.id("--project")?),
        ("login", "confirm") => Operation::ConfirmLogin(a.id("--project")?),
        ("login", "close") => Operation::CloseLogin(a.id("--project")?),
        ("login", "forget") => Operation::ForgetLogin(a.id("--project")?),
        ("login", "probe") => Operation::ProbeEnvironment(a.id("--project")?),
        ("discovery", "start") => Operation::Discover(a.id("--project")?),
        ("discovery", "inventory") if a.options.contains_key("--job") => {
            return Ok(Plan::Request(Request::Job {
                id: a.uuid("--job")?,
            }));
        }
        ("discovery", "inventory" | "existing") => Operation::ExistingInventory(a.id("--project")?),
        ("discovery", "status") => {
            return Ok(Plan::Request(Request::Job {
                id: a.uuid("--job")?,
            }));
        }
        ("discovery", "include" | "exclude") => {
            Operation::Terminal(terminal::Command::InventoryEdit {
                preview: Box::new(a.file()?),
                pages: a
                    .need("--pages")?
                    .split(',')
                    .map(id)
                    .collect::<Result<_>>()?,
                included: action == "include",
            })
        }
        ("capture", "baseline" | "check" | "resume") => Operation::CaptureInventory {
            project: a.id("--project")?,
            preview: if action == "baseline" {
                Some(Box::new(a.file::<DiscoveryPreview>()?))
            } else {
                None
            },
            check: action == "check",
            resume: if action == "resume" {
                Some(a.id("--run")?)
            } else {
                None
            },
        },
        ("capture", "preview") => Operation::PreviewImages {
            project: a.id("--project")?,
            preview: Box::new(a.file()?),
        },
        ("jobs", "list") => {
            return Ok(Plan::Request(Request::Jobs {
                offset: a.number("--offset", 0)?,
                limit: a.number("--limit", 50)?,
            }));
        }
        ("jobs", "show") => {
            return Ok(Plan::Request(Request::Job {
                id: a.uuid("--job")?,
            }));
        }
        ("jobs", "wait") => return Ok(Plan::Wait(a.uuid("--job")?)),
        ("jobs", "pause") => {
            return Ok(Plan::Request(Request::Pause {
                id: a.uuid("--job")?,
            }));
        }
        ("jobs", "cancel") => {
            return Ok(Plan::Request(Request::Cancel {
                id: a.uuid("--job")?,
            }));
        }
        ("jobs", "ack") => {
            return Ok(Plan::Request(Request::Acknowledge {
                id: a.uuid("--job")?,
            }));
        }
        ("jobs", "retry") => return Ok(Plan::Retry(a.uuid("--request-id")?)),
        ("results", "list") => {
            let kind = check_kind(a)?;
            let selection = selection(a)?;
            let page = a.optional_id("--page")?;
            let profile = a.optional_id("--profile")?;
            let requested_outcome = a.take("--outcome");
            let classification = requested_outcome
                .as_deref()
                .map(|s| match s {
                    "changed" => Ok(Classification::Changed),
                    "unchanged" => Ok(Classification::NoSignificantChange),
                    "failed" => Ok(Classification::CouldNotCapture),
                    "noncomparable" => Ok(Classification::NotComparable),
                    _ => bail!("invalid_arguments: outcome changed|unchanged|failed|noncomparable"),
                })
                .transpose()?;
            let capture_state = a
                .take("--capture-state")
                .map(|s| match s.as_str() {
                    "pending" => Ok(terminal::CaptureFilter::Pending),
                    "captured" => Ok(terminal::CaptureFilter::Captured),
                    "failed" => Ok(terminal::CaptureFilter::Failed),
                    "skipped" => Ok(terminal::CaptureFilter::Skipped),
                    "cancelled" => Ok(terminal::CaptureFilter::Cancelled),
                    _ => bail!(
                        "invalid_arguments: capture-state pending|captured|failed|skipped|cancelled"
                    ),
                })
                .transpose()?;
            let offset = a.number("--offset", 0)?;
            let limit = a.number("--limit", 50)?;
            if kind == CheckKind::DomSnapshot {
                return Ok(Plan::Operation(Operation::Dom(dom::Command::Results {
                    selection: dom_selection(selection),
                    page,
                    profile,
                    outcome: classification.map(|c| match c {
                        Classification::Changed => dom::OutcomeFilter::Changed,
                        Classification::NoSignificantChange => dom::OutcomeFilter::Unchanged,
                        Classification::CouldNotCapture => dom::OutcomeFilter::Failed,
                        Classification::NotComparable => dom::OutcomeFilter::NotComparable,
                    }),
                    capture_state,
                    offset,
                    limit,
                })));
            }
            Operation::Terminal(terminal::Command::Results {
                selection,
                page,
                profile,
                classification,
                capture_state,
                offset,
                limit,
            })
        }
        ("results", "artifacts") => {
            let kind = check_kind(a)?;
            let selection = selection(a)?;
            let key = key(a)?;
            if kind == CheckKind::DomSnapshot {
                Operation::Dom(dom::Command::Artifacts {
                    selection: dom_selection(selection),
                    key,
                })
            } else {
                Operation::Terminal(terminal::Command::Artifacts { selection, key })
            }
        }
        ("snapshot", "source" | "diff") => {
            let selection = dom_selection(selection(a)?);
            let key = key(a)?;
            if action == "source" {
                let pane = match a.take("--pane").as_deref().unwrap_or("after") {
                    "before" => dom::SourcePane::Before,
                    "after" => dom::SourcePane::After,
                    _ => bail!("invalid_arguments: source pane before|after"),
                };
                let format = match a.take("--format").as_deref().unwrap_or("plaintext") {
                    "plaintext" => dom::SourceFormat::Plaintext,
                    "canonical" => dom::SourceFormat::Canonical,
                    _ => bail!("invalid_arguments: source format plaintext|canonical"),
                };
                Operation::Dom(dom::Command::Source {
                    selection,
                    key,
                    pane,
                    format,
                    offset: a.number("--offset", 0)?,
                    limit: a.number("--limit", 65536)?,
                })
            } else {
                let options = regressee_core::dom_snapshot::DomDiffOptions {
                    context_lines: a.number("--context", 3)?,
                    ..Default::default()
                };
                Operation::Dom(dom::Command::Diff {
                    selection,
                    key,
                    options,
                    offset: a.number("--offset", 0)?,
                    limit: a.number("--limit", 200)?,
                })
            }
        }
        ("analysis", "show" | "select") => {
            let kind = check_kind(a)?;
            if kind == CheckKind::DomSnapshot {
                let analysis = a.id("--analysis")?;
                Operation::Dom(if action == "select" {
                    dom::Command::SelectAnalysis { analysis }
                } else {
                    dom::Command::Analysis { analysis }
                })
            } else if action == "select" {
                Operation::LoadAnalysis(a.id("--analysis")?)
            } else {
                Operation::Terminal(terminal::Command::Analysis {
                    analysis: a.id("--analysis")?,
                })
            }
        }
        ("analysis", "reanalyze") => {
            if check_kind(a)? == CheckKind::DomSnapshot {
                let options = regressee_core::dom_snapshot::DomDiffOptions {
                    context_lines: a.number("--context", 3)?,
                    ..Default::default()
                };
                return Ok(Plan::Operation(Operation::Dom(dom::Command::Reanalyze {
                    run: a.id("--run")?,
                    options,
                })));
            }
            let policy = if a.options.contains_key("--file") {
                a.file::<Policy>()?
            } else {
                Policy::seed(match a.take("--preset").as_deref().unwrap_or("balanced") {
                    "sensitive" => Preset::Sensitive,
                    "balanced" => Preset::Balanced,
                    "relaxed" => Preset::Relaxed,
                    _ => bail!("invalid_arguments: preset sensitive|balanced|relaxed"),
                })
            };
            policy.validate()?;
            Operation::Reanalyze {
                run: a.id("--run")?,
                policy,
            }
        }
        ("masks", "save") => {
            ensure!(
                check_kind(a)? == CheckKind::Screenshot,
                "invalid_arguments: ignored rectangles are screenshot-only and never suppress DOM differences"
            );
            Operation::Mask {
                project: a.id("--project")?,
                request: Box::new(a.file::<AnalysisRequest>()?),
            }
        }
        ("review", "set") => {
            let kind = check_kind(a)?;
            let note = a.take("--note");
            let file = a.take("--note-file");
            ensure!(
                note.is_none() || file.is_none(),
                "invalid_arguments: choose --note or --note-file"
            );
            let note = if let Some(file) = file {
                String::from_utf8(bytes(&file)?)?
            } else {
                note.unwrap_or_default()
            };
            validate_review_note(&note).map_err(anyhow::Error::msg)?;
            let decision = ReviewDecision {
                analysis: a.id("--analysis")?,
                key: key(a)?,
                state: match a.need("--state")?.as_str() {
                    "unreviewed" => ReviewState::Unreviewed,
                    "intentional" => ReviewState::Intentional,
                    "needs-fix" => ReviewState::NeedsFix,
                    _ => bail!("invalid_arguments: state unreviewed|intentional|needs-fix"),
                },
                note,
                reviewed_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)?
                    .as_secs() as i64,
            };
            if kind == CheckKind::DomSnapshot {
                Operation::Dom(dom::Command::SetReview {
                    analysis: DomAnalysisId(decision.analysis.0),
                    key: decision.key,
                    state: decision.state,
                    note: decision.note,
                    reviewed_at: decision.reviewed_at,
                })
            } else {
                Operation::Review(decision)
            }
        }
        ("review", "history") => {
            let kind = check_kind(a)?;
            let analysis: AnalysisId = a.id("--analysis")?;
            let page = a.optional_id("--page")?;
            let profile = a.optional_id("--profile")?;
            ensure!(
                page.is_some() == profile.is_some(),
                "review history target requires both --page and --profile"
            );
            let key = page
                .zip(profile)
                .map(|(page, profile)| TargetKey { page, profile });
            let before = a.take("--before").map(|s| s.parse::<i64>()).transpose()?;
            let limit = a.number("--limit", 50)?;
            if kind == CheckKind::DomSnapshot {
                Operation::Dom(dom::Command::ReviewHistory {
                    analysis: DomAnalysisId(analysis.0),
                    key,
                    before,
                    limit,
                })
            } else {
                Operation::Terminal(terminal::Command::ReviewHistory {
                    analysis,
                    key,
                    before,
                    limit,
                })
            }
        }
        ("baseline", "preview") => {
            let kind = check_kind(a)?;
            let analysis = a.id::<AnalysisId>("--analysis")?;
            let selected = a.file()?;
            if kind == CheckKind::DomSnapshot {
                Operation::Dom(dom::Command::Approval(dom::KindSelection {
                    screenshot: None,
                    dom: Some(dom::KindKeys {
                        analysis: DomAnalysisId(analysis.0),
                        selected,
                    }),
                }))
            } else {
                Operation::Approval { analysis, selected }
            }
        }
        ("baseline", "preview-kinds") => Operation::Dom(dom::Command::Approval(a.file()?)),
        ("baseline", "save") => Operation::Activate {
            run: a.id("--run")?,
            revision: revision(a)?,
            confirm_partial: a.flag("--partial"),
        },
        ("baseline", "accept") => {
            if check_kind(a)? == CheckKind::DomSnapshot {
                let p: KindApprovalPreview = a.strict_file()?;
                ensure!(
                    p.screenshot.is_none() && p.dom.is_some(),
                    "invalid_input: --kind dom accepts only explicitly selected DOM slots; use accept-kinds for an explicit multi-kind preview"
                );
                return Ok(Plan::Operation(Operation::Dom(dom::Command::Accept(
                    Box::new(p),
                ))));
            }
            let p: ApprovalPreview = a.file()?;
            let context = p.context.clone();
            Operation::Accept {
                preview: Box::new(p),
                context,
            }
        }
        ("baseline", "accept-kinds") => {
            Operation::Dom(dom::Command::Accept(Box::new(a.strict_file()?)))
        }
        ("baseline", "restore") => Operation::Restore {
            baseline: a.id("--baseline")?,
            revision: revision(a)?,
        },
        ("history", "list") => Operation::History {
            project: a.id("--project")?,
            page: a.number("--page", 0)?,
        },
        ("history", "show") => {
            let kind = check_kind(a)?;
            let project = a.id("--project")?;
            let run = a.id("--run")?;
            if kind == CheckKind::DomSnapshot {
                Operation::Dom(dom::Command::Inspect { project, run })
            } else {
                Operation::InspectRun { project, run }
            }
        }
        ("storage", "usage") => {
            if let Some(p) = a.take("--project") {
                Operation::ProjectStorage(id(&p)?)
            } else {
                Operation::InspectStorage
            }
        }
        ("storage", "delete-preview") => {
            let project = a.id("--project")?;
            let item = if let Some(r) = a.take("--run") {
                DeleteItem::Run(id(&r)?)
            } else {
                DeleteItem::Baseline(a.id("--baseline")?)
            };
            Operation::DeletePreview { project, item }
        }
        ("storage", "delete") => Operation::Delete(Box::new(a.file()?)),
        ("repair", "preview") => Operation::RepairPreview,
        ("repair", "clean") => Operation::CleanupTemporaries(Box::new(a.file()?)),
        ("repair", "delete-unreferenced") => Operation::DeleteUnreferenced(Box::new(a.file()?)),
        ("archive", "export-preview") => Operation::ExportPreview {
            project: a.id("--project")?,
            baseline: a.id("--baseline")?,
        },
        ("archive", "export") => Operation::Export {
            preview: Box::new(a.file()?),
            path: PathBuf::from(a.need("--destination")?),
        },
        ("archive", "import-preview") => {
            Operation::ImportPreview(PathBuf::from(a.need("--source")?))
        }
        ("archive", "import") => Operation::Import {
            preview: Box::new(a.file()?),
            path: PathBuf::from(a.need("--source")?),
        },
        ("report", "preview") => Operation::Terminal(terminal::Command::ReportPreview {
            selection: selection(a)?,
            dom_analysis: a.optional_id("--dom-analysis")?,
            selected: a.file()?,
            destination: PathBuf::from(a.need("--destination")?),
        }),
        ("report", "export") => Operation::Terminal(terminal::Command::Report(Box::new(a.file()?))),
        ("crop", "preview") => {
            ensure!(
                check_kind(a)? == CheckKind::Screenshot,
                "invalid_arguments: DOM snapshots are text; crop requires --kind screenshot. Use snapshot source or snapshot diff instead"
            );
            let selection = selection(a)?;
            let key = key(a)?;
            let pane = match a.need("--pane")?.as_str() {
                "before" => terminal::Pane::Before,
                "after" => terminal::Pane::After,
                "difference" => terminal::Pane::Difference,
                _ => bail!("invalid_arguments: pane before|after|difference"),
            };
            ensure!(
                a.need("--space")? == "screenshot-pixels",
                "invalid_arguments: --space must be screenshot-pixels (never CSS pixels)"
            );
            let rect = a
                .need("--rect")?
                .split(',')
                .map(|s| {
                    s.parse::<u32>()
                        .context("invalid_arguments: rect X,Y,W,H unsigned pixels")
                })
                .collect::<Result<Vec<_>>>()?;
            ensure!(rect.len() == 4, "invalid_arguments: rect X,Y,W,H");
            Operation::Terminal(terminal::Command::CropPreview {
                selection,
                key,
                pane,
                rectangle: Rectangle {
                    x: rect[0],
                    y: rect[1],
                    width: rect[2],
                    height: rect[3],
                },
                destination: PathBuf::from(a.need("--destination")?),
            })
        }
        ("crop", "export") => {
            ensure!(
                check_kind(a)? == CheckKind::Screenshot,
                "invalid_arguments: DOM snapshots are text and cannot be cropped"
            );
            Operation::Terminal(terminal::Command::Crop(Box::new(a.file()?)))
        }
        ("diagnostics", "preview") => Operation::DiagnosticPreview,
        ("diagnostics", "export") => Operation::DiagnosticExport {
            report: Box::new(a.file()?),
            path: PathBuf::from(a.need("--destination")?),
        },
        ("browser", "status") => Operation::RuntimeStatus,
        ("browser", "setup") => Operation::SetupRuntime,
        ("network", "status") => Operation::NetworkStatus,
        ("network", "set") => Operation::SaveNetwork(a.file()?),
        ("appearance", "show") => return Ok(Plan::Request(Request::Preferences)),
        ("appearance", "set") => {
            return Ok(Plan::Appearance(match a.need("--value")?.as_str() {
                "system" => AppAppearance::System,
                "light" => AppAppearance::Light,
                "dark" => AppAppearance::Dark,
                _ => bail!("invalid_arguments: appearance system|light|dark"),
            }));
        }
        ("confirm", "") => {
            return Ok(Plan::Confirm {
                ticket: a.file()?,
                id: a.uuid("--confirmation")?,
            });
        }
        _ => bail!("invalid_arguments: unknown command; see --help"),
    };
    Ok(Plan::Operation(operation))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(words: &[&str]) -> Result<Plan> {
        let mut args = Args::parse(&words.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>())?;
        let result = plan(&mut args)?;
        args.done()?;
        result.validate_local()?;
        Ok(result)
    }

    #[test]
    fn capture_modes_use_the_canonical_project_plan() -> Result<()> {
        for (mode, expected) in [
            ("screenshots", CaptureMode::ScreenshotsOnly),
            ("both", CaptureMode::ScreenshotsAndDomSnapshots),
            ("dom", CaptureMode::DomSnapshotsOnly),
        ] {
            let Plan::Operation(Operation::Save { project, .. }) = parse(&[
                "project",
                "create",
                "--name",
                "Owned fixture",
                "--url",
                "https://fixture.invalid/",
                "--mode",
                mode,
            ])?
            else {
                unreachable!()
            };
            assert_eq!(project.capture_plan, CapturePlan::for_mode(expected));
            let Plan::Configure { field, value, .. } = parse(&[
                "checks",
                "set",
                "--project",
                "11111111-1111-4111-8111-111111111111",
                "--expected-revision",
                "1",
                "--mode",
                mode,
            ])?
            else {
                unreachable!()
            };
            assert_eq!(field, "capture_plan");
            assert_eq!(
                serde_json::from_value::<CapturePlan>(value)?,
                project.capture_plan
            );
        }
        let Plan::Operation(Operation::Save { project, .. }) = parse(&[
            "project",
            "create",
            "--name",
            "Owned fixture",
            "--url",
            "https://fixture.invalid/",
        ])?
        else {
            unreachable!()
        };
        assert_eq!(project.capture_plan, CapturePlan::default());
        assert!(
            parse(&[
                "checks",
                "set",
                "--project",
                "11111111-1111-4111-8111-111111111111",
                "--expected-revision",
                "1",
                "--mode",
                "unknown"
            ])
            .is_err()
        );
        // A kind flag must never override a capture/check's frozen plan.
        assert!(
            parse(&[
                "capture",
                "check",
                "--project",
                "11111111-1111-4111-8111-111111111111",
                "--kind",
                "screenshots"
            ])
            .is_err()
        );
        Ok(())
    }
}
