// SPDX-License-Identifier: MPL-2.0
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use url::Url;
use uuid::Uuid;

pub const SCHEMA_VERSION: u32 = 1;
pub const MAX_PROJECT_BYTES: usize = 256 * 1024;
pub const MAX_TARGETS: usize = 30_000;

macro_rules! ids {
    ($($name:ident),+) => {$ (
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
        #[serde(transparent)]
        pub struct $name(pub Uuid);
        impl $name {
            pub fn new() -> Self { Self(Uuid::new_v4()) }
        }
        impl Default for $name { fn default() -> Self { Self::new() } }
        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { self.0.fmt(f) }
        }
    )+};
}
ids!(
    ProjectId,
    EnvironmentId,
    InventoryId,
    ProfileId,
    RunId,
    BaselineId,
    AnalysisId,
    MaskId,
    SessionId,
    PageId
);

/// Validated content address, not a caller-supplied filesystem path.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct ArtifactId(String);
impl ArtifactId {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<String> for ArtifactId {
    type Error = anyhow::Error;
    fn try_from(value: String) -> Result<Self> {
        ensure!(
            value.len() == 64
                && value
                    .bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
            "invalid artifact hash"
        );
        Ok(Self(value))
    }
}
impl From<ArtifactId> for String {
    fn from(id: ArtifactId) -> Self {
        id.0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub version: u32,
    pub id: ProjectId,
    pub name: String,
    pub sources: Vec<Url>,
    pub discovery: DiscoverySettings,
    pub environments: Vec<Environment>,
    pub profiles: Vec<CaptureProfile>,
    pub active_baseline: Option<BaselineId>,
}
impl Project {
    pub fn draft(name: String, source: &str) -> Result<Self> {
        ensure!(
            (source.to_ascii_lowercase().starts_with("https://")
                || source.to_ascii_lowercase().starts_with("http://"))
                && !source.chars().any(char::is_whitespace)
                && !source.contains('\\'),
            "Use a complete http:// or https:// address without spaces or backslashes"
        );
        let source = Url::parse(source)?;
        validate_address(&source)?;
        let mut origin = source.clone();
        origin.set_path("/");
        origin.set_query(None);
        origin.set_fragment(None);
        let project = Self {
            version: SCHEMA_VERSION,
            id: ProjectId::new(),
            name,
            sources: vec![source],
            discovery: DiscoverySettings::default(),
            environments: vec![Environment {
                id: EnvironmentId::new(),
                name: "Website".into(),
                mappings: vec![OriginMapping {
                    source: origin.clone(),
                    destination: origin,
                }],
                session: None,
                access: AccessPolicy::Public,
            }],
            profiles: vec![CaptureProfile::desktop()],
            active_baseline: None,
        };
        project.validate()?;
        Ok(project)
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == SCHEMA_VERSION,
            "unsupported project version"
        );
        ensure!(!self.id.0.is_nil(), "project ID is empty");
        validate_label(&self.name, 120)?;
        ensure!(
            (1..=1000).contains(&self.sources.len()),
            "choose 1–1000 starting addresses"
        );
        for address in &self.sources {
            validate_address(address)?;
        }
        self.discovery.validate()?;
        ensure!(
            (1..=16).contains(&self.environments.len()),
            "choose 1–16 environments"
        );
        ensure!(
            (1..=16).contains(&self.profiles.len()),
            "choose 1–16 screen sizes"
        );
        let mut envs = BTreeSet::new();
        for env in &self.environments {
            ensure!(
                !env.id.0.is_nil() && envs.insert(env.id),
                "duplicate or empty environment ID"
            );
            validate_label(&env.name, 120)?;
            ensure!(
                !env.mappings.is_empty() && env.mappings.len() <= 1000,
                "invalid origin mappings"
            );
            for mapping in &env.mappings {
                crate::environment::validate_mapping(mapping)?;
            }
            if let Some(session) = &env.session {
                ensure!(
                    session.project == self.id
                        && session.environment == env.id
                        && !session.id.0.is_nil(),
                    "session scope mismatch"
                );
            }
        }
        let mut profiles = BTreeSet::new();
        for profile in &self.profiles {
            profile.validate()?;
            ensure!(profiles.insert(profile.id), "duplicate screen-size ID");
        }
        ensure!(
            serde_json::to_vec(self)?.len() <= MAX_PROJECT_BYTES,
            "project settings exceed size limit"
        );
        Ok(())
    }
}
pub fn validate_label(label: &str, limit: usize) -> Result<()> {
    ensure!(
        !label.trim().is_empty()
            && label.chars().count() <= limit
            && !label.chars().any(char::is_control),
        "enter a name of 1–{limit} characters"
    );
    Ok(())
}
pub fn validate_address(address: &Url) -> Result<()> {
    ensure!(
        matches!(address.scheme(), "http" | "https") && address.host_str().is_some(),
        "use a complete http or https website address"
    );
    ensure!(
        address.username().is_empty() && address.password().is_none(),
        "login details must not be part of a website address"
    );
    ensure!(
        address.as_str().len() <= 4096,
        "website address is too long"
    );
    Ok(())
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct DiscoverySettings {
    pub mode: DiscoveryMode,
    pub page_limit: u32,
    pub section: Option<String>,
    pub skipped_text: Vec<String>,
    pub speed: Speed,
    pub respect_robots: bool,
    #[serde(default)]
    pub advanced: CrawlAdvanced,
}
impl Default for DiscoverySettings {
    fn default() -> Self {
        Self {
            mode: DiscoveryMode::AddressesAndLinks,
            page_limit: 500,
            section: None,
            skipped_text: vec![],
            speed: Speed::Balanced,
            respect_robots: true,
            advanced: CrawlAdvanced::default(),
        }
    }
}
impl DiscoverySettings {
    pub fn validate(&self) -> Result<()> {
        self.advanced.validate()?;
        ensure!(
            self.respect_robots || self.advanced.robots_authorized,
            "Confirm you are authorized before overriding robots.txt"
        );
        ensure!(
            (1..=10_000).contains(&self.page_limit),
            "page limit must be 1–10,000"
        );
        ensure!(
            self.skipped_text.len() <= 100 && self.skipped_text.iter().all(|s| s.len() <= 1024),
            "too many or oversized exclusions"
        );
        ensure!(
            self.section.as_ref().is_none_or(|s| s.starts_with('/')
                && !s.starts_with("//")
                && s.len() <= 4096
                && !s.contains(['?', '#', '\\'])
                && !s.chars().any(char::is_control)
                && !s.split('/').any(|p| p == ".." || p == ".")),
            "section must be a path starting with /"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(default, deny_unknown_fields)]
pub struct CrawlAdvanced {
    pub max_depth: Option<u32>,
    pub strip_tracking: bool,
    pub preserve_fragments: bool,
    pub allow_action_links: bool,
    pub robots_authorized: bool,
    pub concurrency: Option<u8>,
    pub per_origin: u8,
    pub retries: u8,
}
impl Default for CrawlAdvanced {
    fn default() -> Self {
        Self {
            max_depth: None,
            strip_tracking: true,
            preserve_fragments: false,
            allow_action_links: false,
            robots_authorized: false,
            concurrency: None,
            per_origin: 2,
            retries: 2,
        }
    }
}
impl CrawlAdvanced {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.max_depth.is_none_or(|d| d <= 100)
                && self.concurrency.is_none_or(|n| (1..=8).contains(&n))
                && (1..=8).contains(&self.per_origin)
                && self.retries <= 3,
            "Depth must be 0–100, parallel pages 1–8, and retries 0–3"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum DiscoveryMode {
    OnlyAddresses,
    AddressesAndLinks,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Speed {
    Gentle,
    Balanced,
    Fast,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Environment {
    pub id: EnvironmentId,
    pub name: String,
    pub mappings: Vec<OriginMapping>,
    // Local references never enter project, evidence or export JSON.
    #[serde(skip_serializing, default, deserialize_with = "no_persisted_session")]
    pub session: Option<SessionReference>,
    pub access: AccessPolicy,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct OriginMapping {
    pub source: Url,
    pub destination: Url,
}
fn no_persisted_session<'de, D: serde::Deserializer<'de>>(
    d: D,
) -> std::result::Result<Option<SessionReference>, D::Error> {
    let value = Option::<SessionReference>::deserialize(d)?;
    if value.is_some() {
        return Err(serde::de::Error::custom(
            "Saved sessions cannot come from project/evidence JSON",
        ));
    }
    Ok(None)
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum AccessPolicy {
    Public,
    InteractiveLogin,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct SessionReference {
    pub id: SessionId,
    pub project: ProjectId,
    pub environment: EnvironmentId,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CaptureProfile {
    pub version: u32,
    pub id: ProfileId,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub dpr: f32,
    pub mobile: bool,
    pub touch: bool,
    pub user_agent: String,
    pub extent: CaptureExtent,
    pub image_mode: ImageMode,
    pub locale: String,
    pub timezone: String,
    pub dark: bool,
    pub reduced_motion: bool,
    pub stabilization: Stabilization,
    pub runtime: RuntimeIdentity,
}
impl CaptureProfile {
    pub fn tablet() -> Self {
        Self {
            name: "Tablet".into(),
            width: 768,
            height: 1024,
            touch: true,
            ..Self::desktop()
        }
    }
    pub fn mobile() -> Self {
        Self {
            name: "Mobile".into(),
            width: 390,
            height: 844,
            mobile: true,
            touch: true,
            ..Self::desktop()
        }
    }
    pub fn desktop() -> Self {
        Self {
            version: 1,
            id: ProfileId::new(),
            name: "Desktop".into(),
            width: 1440,
            height: 900,
            dpr: 1.,
            mobile: false,
            touch: false,
            user_agent: "Unconfigured — requires a qualified runtime before capture".into(),
            extent: CaptureExtent::FullPage,
            image_mode: ImageMode::Normal,
            locale: "en-US".into(),
            timezone: "UTC".into(),
            dark: false,
            reduced_motion: true,
            stabilization: Stabilization {
                version: 1,
                navigation_ms: 30_000,
                readiness_ms: 15_000,
                max_height: 100_000,
            },
            runtime: RuntimeIdentity::Unconfigured,
        }
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.version == 1 && !self.id.0.is_nil(),
            "unsupported or empty profile"
        );
        validate_label(&self.name, 120)?;
        if let RuntimeIdentity::Pinned {
            node,
            worker,
            playwright,
            browser,
        } = &self.runtime
        {
            for version in [node, worker, playwright, browser] {
                validate_label(version, 128)?;
            }
        }
        if let ImageMode::ReplaceRaster { algorithm } = &self.image_mode {
            validate_label(algorithm, 128)?;
        }
        validate_label(&self.locale, 80)?;
        validate_label(&self.timezone, 120)?;
        ensure!(
            (1..=8192).contains(&self.width) && (1..=8192).contains(&self.height),
            "screen size must be 1–8192 pixels"
        );
        ensure!(
            self.dpr.is_finite() && (1.0..=4.0).contains(&self.dpr),
            "invalid pixel ratio"
        );
        ensure!(
            self.user_agent.len() <= 1024
                && !self.user_agent.is_empty()
                && self.locale.len() <= 80
                && self.timezone.len() <= 120,
            "invalid browser settings"
        );
        ensure!(
            self.stabilization.version == 1
                && (1..=120_000).contains(&self.stabilization.navigation_ms)
                && self.stabilization.readiness_ms <= 120_000
                && (1..=100_000).contains(&self.stabilization.max_height),
            "invalid readiness limits"
        );
        Ok(())
    }
    pub fn ready_for_capture(&self) -> bool {
        matches!(self.runtime, RuntimeIdentity::Pinned { .. })
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RuntimeIdentity {
    Unconfigured,
    Pinned {
        node: String,
        worker: String,
        playwright: String,
        browser: String,
    },
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CaptureExtent {
    FullPage,
    VisibleScreen,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ImageMode {
    Normal,
    ReplaceRaster { algorithm: String },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Stabilization {
    pub version: u32,
    pub navigation_ms: u32,
    pub readiness_ms: u32,
    pub max_height: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InventoryRevision {
    pub id: InventoryId,
    pub project: ProjectId,
    pub identity_version: u32,
    pub pages: Vec<InventoryPage>,
    pub coverage: DiscoveryCoverage,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct InventoryPage {
    pub id: PageId,
    pub canonical: Url,
    pub included: bool,
    pub reason: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub enum DiscoveryCoverage {
    Complete,
    Limited { reason: String },
    Interrupted,
}
impl DiscoveryCoverage {
    pub fn label(&self) -> String {
        match self {Self::Complete=>"Visible-link queue exhausted within these settings; not proof of every website page.".into(),Self::Limited{reason}=>format!("Limited coverage: {reason}"),Self::Interrupted=>"Discovery interrupted; coverage is incomplete.".into()}
    }
}
impl InventoryRevision {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            matches!(self.identity_version, 1 | 2)
                && !self.id.0.is_nil()
                && !self.project.0.is_nil(),
            "unsupported or empty inventory identity"
        );
        ensure!(self.pages.len() <= 10_000, "inventory exceeds page budget");
        let mut ids = BTreeSet::new();
        let mut addresses = BTreeSet::new();
        for page in &self.pages {
            validate_address(&page.canonical)?;
            ensure!(
                !page.id.0.is_nil()
                    && ids.insert(page.id)
                    && addresses.insert(page.canonical.as_str()),
                "duplicate or empty inventory page"
            );
            ensure!(
                page.reason
                    .as_ref()
                    .is_none_or(|reason| reason.len() <= 1024),
                "oversized exclusion reason"
            );
        }
        Ok(())
    }
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TargetKey {
    pub page: PageId,
    pub profile: ProfileId,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CaptureRun {
    pub id: RunId,
    pub project: ProjectId,
    pub intent: RunIntent,
    pub inventory: InventoryRevision,
    pub profiles: Vec<CaptureProfile>,
    pub environment: Environment,
    pub rendering: RenderingEnvironment,
    pub state: JobState,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
    pub targets: Vec<CaptureTarget>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RunIntent {
    RepresentativePreview,
    NewBaseline,
    Check { baseline: BaselineId },
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RenderingEnvironment {
    pub os: String,
    pub architecture: String,
    pub browser: String,
    pub worker_version: String,
    pub profile_version: u32,
    pub fonts_fingerprint: Option<String>,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum JobState {
    Queued,
    Running,
    Pausing,
    Paused,
    Completed,
    CompletedWithErrors,
    Cancelled,
    Interrupted,
}
impl JobState {
    pub fn terminal(self) -> bool {
        matches!(
            self,
            Self::Completed | Self::CompletedWithErrors | Self::Cancelled | Self::Interrupted
        )
    }
    pub fn after_restart(self) -> Self {
        if matches!(self, Self::Running | Self::Pausing) {
            Self::Interrupted
        } else {
            self
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct CaptureTarget {
    pub key: TargetKey,
    pub requested: Url,
    pub final_url: Option<Url>,
    pub attempts: u8,
    pub outcome: TargetOutcome,
    pub warnings: Vec<String>,
}
impl CaptureRun {
    pub fn validate(&self) -> Result<()> {
        self.inventory.validate()?;
        ensure!(
            self.project == self.inventory.project && !self.id.0.is_nil(),
            "run project/inventory mismatch"
        );
        ensure!(
            !self.targets.is_empty()
                && self.targets.len() <= MAX_TARGETS
                && !self.profiles.is_empty()
                && self.profiles.len() <= 16,
            "invalid run target/profile count"
        );
        let pages: std::collections::BTreeMap<_, _> = self
            .inventory
            .pages
            .iter()
            .filter(|p| p.included)
            .map(|p| (p.id, &p.canonical))
            .collect();
        let mut profiles = BTreeSet::new();
        for profile in &self.profiles {
            profile.validate()?;
            ensure!(
                profile.ready_for_capture() && profiles.insert(profile.id),
                "run requires unique configured profiles"
            );
        }
        ensure!(
            self.targets.len() == pages.len() * profiles.len(),
            "run must account for the fixed intended target list"
        );
        let mut keys = BTreeSet::new();
        for target in &self.targets {
            ensure!(
                pages.contains_key(&target.key.page)
                    && profiles.contains(&target.key.profile)
                    && keys.insert(&target.key),
                "duplicate or foreign capture target"
            );
            validate_address(&target.requested)?;
            ensure!(
                pages
                    .get(&target.key.page)
                    .is_some_and(|url| **url == target.requested),
                "requested URL differs from frozen page identity"
            );
            if let Some(url) = &target.final_url {
                validate_address(url)?;
            }
            ensure!(
                target.attempts <= 8
                    && target.warnings.len() <= 64
                    && target.warnings.iter().all(|w| w.len() <= 1024),
                "capture diagnostics exceed limits"
            );
            if let TargetOutcome::Captured { width, height, .. } = &target.outcome {
                ensure!(
                    *width > 0
                        && *width <= 32768
                        && *height > 0
                        && *height <= 100_000
                        && target.final_url.is_some()
                        && target.attempts > 0,
                    "invalid successful capture metadata"
                );
            }
        }
        ensure!(
            self.finished_at
                .is_none_or(|finished| self.started_at.is_some_and(|started| finished >= started)),
            "invalid run timestamps"
        );
        if self.state == JobState::Completed {
            ensure!(
                self.targets
                    .iter()
                    .all(|t| matches!(t.outcome, TargetOutcome::Captured { .. })),
                "completed run cannot conceal failed or missing coverage"
            );
        }
        if matches!(
            self.state,
            JobState::Completed | JobState::CompletedWithErrors
        ) {
            ensure!(
                self.finished_at.is_some()
                    && self
                        .targets
                        .iter()
                        .all(|t| !matches!(t.outcome, TargetOutcome::Pending)),
                "finished run has pending targets or no completion time"
            );
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TargetOutcome {
    Pending,
    Captured {
        artifact: ArtifactId,
        width: u32,
        height: u32,
    },
    Failed {
        code: CaptureError,
    },
    Skipped {
        reason: String,
    },
    Cancelled,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum CaptureError {
    NeedsLogin,
    AccessDenied,
    NotFound,
    RedirectOutsideScope,
    Timeout,
    Unstable,
    ImageLimit,
    WorkerStopped,
    LowSpace,
    CaptureFailed,
    Cancelled,
    RateLimited,
    ServerError,
}
impl CaptureError {
    pub fn label(self) -> &'static str {
        match self {
            Self::NeedsLogin => "Login is needed",
            Self::AccessDenied => "Website access denied",
            Self::NotFound => "Page not found (404 or 410)",
            Self::RedirectOutsideScope => "Unexpected redirect — not followed",
            Self::Timeout => "Website took too long",
            Self::Unstable => "Page did not settle",
            Self::ImageLimit => "Screenshot exceeds safe image limits",
            Self::WorkerStopped => "Private browser stopped",
            Self::LowSpace => "Low disk space",
            Self::CaptureFailed => "Could not capture this page",
            Self::Cancelled => "Cancelled",
            Self::RateLimited => "Website requested slower visits (429)",
            Self::ServerError => "Website server unavailable",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BaselineVersion {
    pub id: BaselineId,
    pub project: ProjectId,
    pub parent: Option<BaselineId>,
    pub inventory: InventoryId,
    pub created_at: i64,
    pub references: Vec<BaselineReference>,
    pub intended_count: u32,
    pub explicitly_partial: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct BaselineReference {
    pub key: TargetKey,
    pub run: RunId,
    pub artifact: ArtifactId,
    pub captured_at: i64,
}
impl BaselineVersion {
    /// Validation boundary, not baseline creation/activation functionality.
    pub fn validate(&self) -> Result<()> {
        ensure!(
            !self.references.is_empty() && self.references.len() <= MAX_TARGETS,
            "baseline requires successful captures"
        );
        ensure!(
            self.intended_count > 0 && self.references.len() <= self.intended_count as usize,
            "invalid intended coverage"
        );
        ensure!(
            self.explicitly_partial || self.references.len() == self.intended_count as usize,
            "partial coverage requires explicit confirmation"
        );
        let mut keys = BTreeSet::new();
        for reference in &self.references {
            ensure!(keys.insert(&reference.key), "duplicate baseline target");
        }
        ensure!(
            self.parent != Some(self.id),
            "baseline cannot inherit itself"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRevision {
    pub id: AnalysisId,
    pub baseline: BaselineId,
    pub run: RunId,
    pub algorithm: String,
    pub policy: AnalysisPolicy,
    pub mask: Option<MaskId>,
    pub results: Vec<AnalysisResult>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AnalysisPolicy {
    pub color_tolerance: f32,
    pub changed_percent: f32,
    pub region_pixels: u32,
}
impl AnalysisPolicy {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.color_tolerance.is_finite()
                && (0.0..=100.0).contains(&self.color_tolerance)
                && self.changed_percent.is_finite()
                && (0.0..=100.0).contains(&self.changed_percent),
            "analysis thresholds must be finite percentages"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct AnalysisResult {
    pub key: TargetKey,
    pub classification: Classification,
    pub eligible_pixels: u64,
    pub changed_pixels: u64,
    pub dimensions_changed: bool,
}
impl AnalysisResult {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.changed_pixels <= self.eligible_pixels,
            "changed pixels exceed eligible pixels"
        );
        ensure!(
            self.classification != Classification::NoSignificantChange || self.eligible_pixels > 0,
            "no eligible pixels cannot establish unchanged evidence"
        );
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum Classification {
    Changed,
    NoSignificantChange,
    CouldNotCapture,
    NotComparable,
}
impl Classification {
    pub fn label(self) -> &'static str {
        match self {
            Self::Changed => "Changed — review the evidence",
            Self::NoSignificantChange => "No significant change",
            Self::CouldNotCapture => "Could not capture — not verified",
            Self::NotComparable => "Could not compare",
        }
    }
}
/// Storage and every native note-save path use UTF-8 bytes, never scalar counts.
pub const REVIEW_NOTE_MAX_BYTES: usize = 8192;
pub const REVIEW_NOTE_LIMIT_MESSAGE: &str =
    "Shorten to 8,192 UTF-8 bytes. Your draft is kept; nothing saved.";
pub fn validate_review_note(note: &str) -> Result<(), &'static str> {
    if note.len() <= REVIEW_NOTE_MAX_BYTES {
        Ok(())
    } else {
        Err(REVIEW_NOTE_LIMIT_MESSAGE)
    }
}

#[cfg(test)]
mod review_note_tests {
    use super::*;
    #[test]
    fn shared_note_limit_is_utf8_bytes_and_preserves_valid_text() {
        for note in [
            "x".repeat(4096),
            "x".repeat(4097),
            "x".repeat(5000),
            "x".repeat(8192),
            "🦀".repeat(2048),
            "\n日本語\r\ncafé\n".into(),
        ] {
            assert!(validate_review_note(&note).is_ok());
        }
        for note in [
            "x".repeat(8193),
            "é".repeat(4097),
            format!("{}x", "🦀".repeat(2048)),
        ] {
            let exact = note.clone();
            assert_eq!(validate_review_note(&note), Err(REVIEW_NOTE_LIMIT_MESSAGE));
            assert_eq!(note, exact);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct ReviewDecision {
    pub analysis: AnalysisId,
    pub key: TargetKey,
    pub state: ReviewState,
    pub note: String,
    pub reviewed_at: i64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ReviewState {
    Unreviewed,
    Intentional,
    NeedsFix,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct MaskRevision {
    pub id: MaskId,
    pub parent: Option<MaskId>,
    pub regions: Vec<TargetMask>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TargetMask {
    pub key: TargetKey,
    pub width: u32,
    pub height: u32,
    pub rectangles: Vec<Rectangle>,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl TargetMask {
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.width > 0 && self.height > 0 && self.rectangles.len() <= 1000,
            "invalid mask dimensions or count"
        );
        for rect in &self.rectangles {
            ensure!(
                rect.width > 0
                    && rect.height > 0
                    && rect
                        .x
                        .checked_add(rect.width)
                        .is_some_and(|x| x <= self.width)
                    && rect
                        .y
                        .checked_add(rect.height)
                        .is_some_and(|y| y <= self.height),
                "ignored area is outside the screenshot"
            );
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Artifact {
    pub id: ArtifactId,
    pub bytes: u64,
    pub kind: ArtifactKind,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq)]
pub enum ArtifactKind {
    Screenshot,
    Thumbnail,
    Tile,
    Difference,
}
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum ArtifactOwner {
    Run(RunId),
    Baseline(BaselineId),
    Analysis(AnalysisId),
}

/// Avoid map keys made from structs in JSON. Baseline references remain arrays.
pub type CoverageCounts = BTreeMap<String, u32>;

#[cfg(test)]
mod tests {
    #[test]
    fn discovery_coverage_strict_schema_and_all_valid_variants_roundtrip() {
        use super::DiscoveryCoverage;
        for coverage in [
            DiscoveryCoverage::Complete,
            DiscoveryCoverage::Interrupted,
            DiscoveryCoverage::Limited {
                reason: "bounded fixture".into(),
            },
        ] {
            let bytes = serde_json::to_vec(&coverage).unwrap();
            assert_eq!(
                serde_json::from_slice::<DiscoveryCoverage>(&bytes).unwrap(),
                coverage
            );
        }
        let error = serde_json::from_str::<DiscoveryCoverage>(
            r#"{"Limited":{"reason":"fixture","script":"unexpected executable instruction"}}"#,
        )
        .unwrap_err();
        assert!(
            error.to_string().contains("unknown field `script`"),
            "{error}"
        );
        assert!(
            serde_json::from_str::<DiscoveryCoverage>(r#"{"Limited":{}}"#)
                .unwrap_err()
                .to_string()
                .contains("missing field `reason`")
        );
    }
    use super::*;
    #[test]
    fn drafts_are_real_settings_not_ready_runs() {
        let p = Project::draft(
            "Release website".into(),
            "https://example.test/docs?a=2&a=1",
        )
        .unwrap();
        assert!(!p.profiles[0].ready_for_capture());
        assert_eq!(p.sources[0].query(), Some("a=2&a=1"));
        let round: Project = serde_json::from_slice(&serde_json::to_vec(&p).unwrap()).unwrap();
        assert_eq!(round, p);
    }
    #[test]
    fn rejects_credentials_schemes_names_and_duplicate_profiles() {
        for url in [
            "file:///etc/passwd",
            "javascript:alert(1)",
            "https://me:secret@example.test",
        ] {
            assert!(Project::draft("Site".into(), url).is_err());
        }
        assert!(Project::draft(" \n".into(), "https://example.test").is_err());
        let mut p = Project::draft("Site".into(), "https://example.test").unwrap();
        p.profiles.push(p.profiles[0].clone());
        assert!(p.validate().is_err());
    }
    #[test]
    fn rejects_session_scope_and_unknown_schema() {
        let mut p = Project::draft("Site".into(), "https://example.test").unwrap();
        p.environments[0].session = Some(SessionReference {
            id: SessionId::new(),
            project: ProjectId::new(),
            environment: p.environments[0].id,
        });
        assert!(p.validate().is_err());
        assert!(serde_json::from_str::<ArtifactId>("\"../x\"").is_err());
        p.version = 2;
        assert!(p.validate().is_err());
    }
    #[test]
    fn masks_are_bounded_no_overflow() {
        let mut m = TargetMask {
            key: TargetKey {
                page: PageId::new(),
                profile: ProfileId::new(),
            },
            width: 10,
            height: 10,
            rectangles: vec![Rectangle {
                x: 8,
                y: 0,
                width: 2,
                height: 10,
            }],
        };
        m.validate().unwrap();
        m.rectangles[0].x = u32::MAX;
        assert!(m.validate().is_err());
    }
    #[test]
    fn partial_and_duplicate_baselines_are_not_full_coverage() {
        let reference = BaselineReference {
            key: TargetKey {
                page: PageId::new(),
                profile: ProfileId::new(),
            },
            run: RunId::new(),
            artifact: "a".repeat(64).try_into().unwrap(),
            captured_at: 1,
        };
        let mut b = BaselineVersion {
            id: BaselineId::new(),
            project: ProjectId::new(),
            parent: None,
            inventory: InventoryId::new(),
            created_at: 2,
            references: vec![reference.clone()],
            intended_count: 2,
            explicitly_partial: false,
        };
        assert!(b.validate().is_err());
        b.explicitly_partial = true;
        b.validate().unwrap();
        b.references.push(reference);
        assert!(b.validate().is_err());
    }
    #[test]
    fn restart_does_not_call_active_work_completed() {
        assert_eq!(JobState::Running.after_restart(), JobState::Interrupted);
        assert_eq!(JobState::Completed.after_restart(), JobState::Completed);
    }
    #[test]
    fn inventories_and_analysis_do_not_hide_missing_evidence() {
        let page = InventoryPage {
            id: PageId::new(),
            canonical: Url::parse("https://example.test/a").unwrap(),
            included: true,
            reason: None,
        };
        let mut inventory = InventoryRevision {
            id: InventoryId::new(),
            project: ProjectId::new(),
            identity_version: 1,
            pages: vec![page.clone()],
            coverage: DiscoveryCoverage::Complete,
        };
        inventory.validate().unwrap();
        inventory.pages.push(page);
        assert!(inventory.validate().is_err());
        assert!(
            AnalysisPolicy {
                color_tolerance: f32::NAN,
                changed_percent: 1.,
                region_pixels: 20
            }
            .validate()
            .is_err()
        );
        let result = AnalysisResult {
            key: TargetKey {
                page: PageId::new(),
                profile: ProfileId::new(),
            },
            classification: Classification::NoSignificantChange,
            eligible_pixels: 0,
            changed_pixels: 0,
            dimensions_changed: false,
        };
        assert!(result.validate().is_err());
    }
    #[test]
    fn run_contract_accounts_for_all_fixed_targets() {
        let project = Project::draft("Site".into(), "https://example.test").unwrap();
        let page = InventoryPage {
            id: PageId::new(),
            canonical: project.sources[0].clone(),
            included: true,
            reason: None,
        };
        let mut profile = project.profiles[0].clone();
        profile.runtime = RuntimeIdentity::Pinned {
            node: "test".into(),
            worker: "test".into(),
            playwright: "test".into(),
            browser: "test".into(),
        };
        let mut run = CaptureRun {
            id: RunId::new(),
            project: project.id,
            intent: RunIntent::NewBaseline,
            inventory: InventoryRevision {
                id: InventoryId::new(),
                project: project.id,
                identity_version: 1,
                pages: vec![page.clone()],
                coverage: DiscoveryCoverage::Complete,
            },
            profiles: vec![profile.clone()],
            environment: project.environments[0].clone(),
            rendering: RenderingEnvironment {
                os: "test".into(),
                architecture: "test".into(),
                browser: "test".into(),
                worker_version: "test".into(),
                profile_version: 1,
                fonts_fingerprint: None,
            },
            state: JobState::Queued,
            started_at: None,
            finished_at: None,
            targets: vec![CaptureTarget {
                key: TargetKey {
                    page: page.id,
                    profile: profile.id,
                },
                requested: page.canonical,
                final_url: None,
                attempts: 0,
                outcome: TargetOutcome::Pending,
                warnings: vec![],
            }],
        };
        run.validate().unwrap();
        run.state = JobState::Completed;
        run.started_at = Some(1);
        run.finished_at = Some(2);
        assert!(run.validate().is_err());
        run.state = JobState::Queued;
        run.targets.push(run.targets[0].clone());
        assert!(run.validate().is_err());
    }
}
