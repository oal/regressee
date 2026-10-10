// SPDX-License-Identifier: MPL-2.0
//! Bounded analysis engine. Phase 5 integration preserves the audited row semantics.
//! Coordinates are never registered, warped, rescaled, padded, or matched away.
use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use std::{
    io::{BufReader, Read, Seek, SeekFrom},
    sync::atomic::{AtomicBool, Ordering},
};

pub const ALGORITHM_ID: &str = "aligned-srgb-d65-ciede2000-rows-v1";
pub const SCHEMA_VERSION: u32 = 1;
pub type RowObserver<'a> = dyn FnMut(u32, &[bool], &[bool]) -> Result<()> + 'a;
const MAX_WIDTH: u32 = 4096;
const MAX_HEIGHT: u32 = 32768;
const MAX_PIXELS: u64 = 64_000_000;
const MAX_BYTES: u64 = 128 * 1024 * 1024;
const IO_CHUNK: usize = 8192;
const RETAINED_REGIONS: usize = 16;

#[derive(Debug)]
pub struct Cancelled;
impl std::fmt::Display for Cancelled {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("analysis cancelled")
    }
}
impl std::error::Error for Cancelled {}
#[derive(Debug)]
pub struct CaptureFailure;
impl std::fmt::Display for CaptureFailure {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("capture failed; no analysis result")
    }
}
impl std::error::Error for CaptureFailure {}
fn checkpoint(cancel: &AtomicBool) -> Result<()> {
    if cancel.load(Ordering::Relaxed) {
        return Err(Cancelled.into());
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Dimensions {
    pub width: u32,
    pub height: u32,
}
impl Dimensions {
    fn validate(self) -> Result<()> {
        ensure!(
            self.width > 0
                && self.width <= MAX_WIDTH
                && self.height > 0
                && self.height <= MAX_HEIGHT
                && self.area() <= MAX_PIXELS,
            "geometry exceeds analysis limits"
        );
        Ok(())
    }
    pub fn area(self) -> u64 {
        u64::from(self.width) * u64::from(self.height)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    pub page: String,
    pub profile: String,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CaptureDescriptor {
    /// Immutable artifact/capture revision identity, checked by the integration layer.
    pub capture_id: String,
    pub target: Target,
    pub dimensions: Dimensions,
    pub device_scale_factor: f64,
    /// Identity of all material capture settings, NOT production/staging URL.
    pub material_profile_id: String,
    pub rendering_environment_id: String,
    pub completed: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rectangle {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}
impl Rectangle {
    fn fits(self, d: Dimensions) -> bool {
        self.width > 0
            && self.height > 0
            && self.x.checked_add(self.width).is_some_and(|x| x <= d.width)
            && self
                .y
                .checked_add(self.height)
                .is_some_and(|y| y <= d.height)
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaskBinding {
    pub revision: String,
    pub target: Target,
    pub before_capture_id: String,
    pub after_capture_id: String,
    pub before_dimensions: Dimensions,
    pub after_dimensions: Dimensions,
    pub rectangles: Vec<Rectangle>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Preset {
    Sensitive,
    Balanced,
    Relaxed,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "mode", deny_unknown_fields)]
pub enum AntiAliasPolicy {
    Off,
    /// Only varying blend values between identical, stationary horizontal endpoints.
    /// Never suppresses endpoints, missing bands, alpha pixels or moving neighbors.
    SharedHorizontalBlendV1 {
        max_delta_e00: f64,
        channel_residual: u8,
    },
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalCriterion {
    pub size: u32,
    pub stride: u32,
    pub min_eligible: u64,
    pub min_changed: u64,
    pub changed_fraction: f64,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Policy {
    pub schema_version: u32,
    pub algorithm: String,
    /// Caller must allocate a new immutable revision whenever any setting changes.
    pub revision: String,
    pub pixel_tolerance_delta_e00: f64,
    pub antialias: AntiAliasPolicy,
    pub global_changed_fraction: Option<f64>,
    pub local: Option<LocalCriterion>,
    pub region_min_pixels: Option<u64>,
    pub height_change_triggers: bool,
    pub ignored_warning_fraction: f64,
}
impl Policy {
    /// Uncalibrated seeds, not claims of user accuracy or confirmed defects.
    pub fn seed(preset: Preset) -> Self {
        let (name, tolerance, global, local, count, region) = match preset {
            Preset::Sensitive => ("sensitive", 1.0, 0.0005, 0.025, 4, 8),
            Preset::Balanced => ("balanced", 2.0, 0.002, 0.05, 8, 16),
            Preset::Relaxed => ("relaxed", 4.0, 0.02, 0.1, 16, 64),
        };
        Self {
            schema_version: SCHEMA_VERSION,
            algorithm: ALGORITHM_ID.into(),
            revision: format!("seed-{name}-v1"),
            pixel_tolerance_delta_e00: tolerance,
            antialias: AntiAliasPolicy::Off,
            global_changed_fraction: Some(global),
            local: Some(LocalCriterion {
                size: 32,
                stride: 16,
                min_eligible: 64,
                min_changed: count,
                changed_fraction: local,
            }),
            region_min_pixels: Some(region),
            height_change_triggers: true,
            ignored_warning_fraction: 0.5,
        }
    }
    pub fn validate(&self) -> Result<()> {
        ensure!(
            self.schema_version == SCHEMA_VERSION && self.algorithm == ALGORITHM_ID,
            "unsupported policy schema/algorithm"
        );
        identity(&self.revision)?;
        ensure!(
            self.pixel_tolerance_delta_e00.is_finite()
                && (0.0..=100.0).contains(&self.pixel_tolerance_delta_e00),
            "invalid Delta E tolerance"
        );
        fn fraction(v: f64) -> bool {
            v.is_finite() && (0.0..=1.0).contains(&v)
        }
        ensure!(
            self.global_changed_fraction.is_none_or(fraction)
                && fraction(self.ignored_warning_fraction),
            "invalid fraction threshold"
        );
        ensure!(
            self.region_min_pixels
                .is_none_or(|v| v > 0 && v <= MAX_PIXELS),
            "invalid region threshold"
        );
        if let Some(l) = &self.local {
            ensure!(
                l.size > 0
                    && l.size <= 256
                    && l.stride > 0
                    && l.stride <= l.size
                    && l.size.div_ceil(l.stride) <= 4
                    && l.min_eligible > 0
                    && l.min_eligible <= u64::from(l.size).pow(2)
                    && l.min_changed > 0
                    && l.min_changed <= u64::from(l.size).pow(2)
                    && fraction(l.changed_fraction),
                "invalid bounded local criterion"
            );
        }
        if let AntiAliasPolicy::SharedHorizontalBlendV1 {
            max_delta_e00,
            channel_residual,
        } = self.antialias
        {
            ensure!(
                max_delta_e00.is_finite()
                    && max_delta_e00 > self.pixel_tolerance_delta_e00
                    && max_delta_e00 <= 10.0
                    && channel_residual <= 2,
                "invalid antialias policy"
            );
        }
        Ok(())
    }
}
fn identity(s: &str) -> Result<()> {
    ensure!(
        !s.trim().is_empty() && s.len() <= 512 && !s.chars().any(char::is_control),
        "invalid revision/target identity"
    );
    Ok(())
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisRequest {
    pub before: CaptureDescriptor,
    pub after: CaptureDescriptor,
    pub policy: Policy,
    pub mask: Option<MaskBinding>,
}
impl AnalysisRequest {
    pub fn validate(&self) -> Result<()> {
        if !self.before.completed || !self.after.completed {
            return Err(CaptureFailure.into());
        }
        self.policy.validate()?;
        for d in [&self.before, &self.after] {
            d.dimensions.validate()?;
            for s in [
                &d.capture_id,
                &d.target.page,
                &d.target.profile,
                &d.material_profile_id,
                &d.rendering_environment_id,
            ] {
                identity(s)?;
            }
            ensure!(
                d.device_scale_factor.is_finite()
                    && d.device_scale_factor > 0.0
                    && d.device_scale_factor <= 8.0,
                "invalid device scale factor"
            );
        }
        ensure!(
            self.before.target == self.after.target,
            "different page/profile targets"
        );
        Dimensions {
            width: self
                .before
                .dimensions
                .width
                .max(self.after.dimensions.width),
            height: self
                .before
                .dimensions
                .height
                .max(self.after.dimensions.height),
        }
        .validate()?;
        if let Some(m) = &self.mask {
            identity(&m.revision)?;
            ensure!(
                m.target == self.before.target
                    && m.before_capture_id == self.before.capture_id
                    && m.after_capture_id == self.after.capture_id
                    && m.before_dimensions == self.before.dimensions
                    && m.after_dimensions == self.after.dimensions,
                "stale mask binding; explicit pair/geometry revalidation required"
            );
            ensure!(m.rectangles.len() <= 1000, "too many mask rectangles");
            for r in &m.rectangles {
                ensure!(
                    r.fits(m.before_dimensions) || r.fits(m.after_dimensions),
                    "mask rectangle outside coordinate union or overflowing"
                );
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Classification {
    Changed,
    NoSignificantChange,
    NotComparable,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GateReason {
    UnexpectedWidth,
    DeviceScaleMismatch,
    MaterialProfileMismatch,
    AllMasked,
    NonOpaquePixels,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Warning {
    LargeIgnoredCoverage,
    RenderingEnvironmentMismatch,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Trigger {
    GlobalFraction,
    LocalConcentration,
    ConnectedRegion,
    HeightChange,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Fraction {
    pub numerator: u64,
    pub denominator: u64,
}
impl Fraction {
    pub fn value(self) -> Option<f64> {
        (self.denominator > 0).then(|| self.numerator as f64 / self.denominator as f64)
    }
    fn at_least(self, threshold: f64) -> bool {
        self.denominator > 0 && self.numerator as f64 >= threshold * self.denominator as f64
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Region {
    pub pixels: u64,
    pub bounds: Rectangle,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RegionSummary {
    pub count: u64,
    pub largest: Option<Region>,
    /// Largest 16, deterministic area/bounds ordering; total count remains exact.
    pub retained: Vec<Region>,
    pub omitted: u64,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WindowEvidence {
    pub bounds: Rectangle,
    pub changed: u64,
    pub eligible: u64,
}
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LocalSummary {
    pub windows: u64,
    pub insufficient_coverage: u64,
    pub zero_eligible: u64,
    pub qualifying: u64,
    pub strongest: Option<WindowEvidence>,
    pub strongest_qualifying: Option<WindowEvidence>,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DimensionEvidence {
    pub before: Dimensions,
    pub after: Dimensions,
    pub canvas: Dimensions,
    pub width_delta: i64,
    pub height_delta: i64,
    pub overlap_pixels: u64,
    pub before_only_pixels: u64,
    pub after_only_pixels: u64,
    pub absent_in_both_pixels: u64,
    /// Up to two disjoint half-open bands per side, even when masked.
    pub before_only_bands: Vec<Rectangle>,
    pub after_only_bands: Vec<Rectangle>,
}
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnalysisReport {
    pub schema_version: u32,
    /// Complete policy/capture/mask bindings, never an opaque confidence score.
    pub request: AnalysisRequest,
    pub classification: Classification,
    pub gates: Vec<GateReason>,
    pub warnings: Vec<Warning>,
    /// Numerical criteria that fired; gates take precedence over these.
    pub triggers: Vec<Trigger>,
    pub dimensions: DimensionEvidence,
    pub coordinate_union_pixels: u64,
    pub ignored_pixels: u64,
    pub ignored_coverage: Fraction,
    pub eligible_pixels: u64,
    pub raw_changed_pixels: u64,
    pub raw_overlap_changed_pixels: u64,
    pub masked_raw_changed_pixels: u64,
    pub unmasked_raw_changed_pixels: u64,
    pub structural_pixels: u64,
    pub unmasked_structural_pixels: u64,
    pub changed_pixels: u64,
    pub changed_fraction: Fraction,
    pub antialias_suppressed_pixels: u64,
    /// All present coordinates, including ignored pixels. Non-opaque gates comparison.
    pub nonopaque_pixels: u64,
    pub raw_regions: RegionSummary,
    pub regions: RegionSummary,
    pub local: LocalSummary,
}

fn bands(a: Dimensions, b: Dimensions) -> Vec<Rectangle> {
    let mut out = Vec::new();
    if a.width > b.width {
        out.push(Rectangle {
            x: b.width,
            y: 0,
            width: a.width - b.width,
            height: a.height.min(b.height),
        });
    }
    if a.height > b.height {
        out.push(Rectangle {
            x: 0,
            y: b.height,
            width: a.width,
            height: a.height - b.height,
        });
    }
    out
}

/// Analyze two seekable immutable PNG byte streams. No files, storage, runtime,
/// heatmap artifacts, history or review labels are modified. Failures return no report.
/// PNG support is intentionally strict: non-interlaced RGB/RGBA8, sRGB samples.
pub fn analyze_pngs<A: Read + Seek, B: Read + Seek>(
    before: A,
    after: B,
    request: &AnalysisRequest,
    cancel: &AtomicBool,
) -> Result<AnalysisReport> {
    let result = analyze_inner(before, after, request, cancel, &mut |_, _, _| Ok(()));
    // Normalize cancellation from std::io/png wrappers to the same typed error
    // as direct checkpoints. Never publish a result after observed cancellation.
    checkpoint(cancel)?;
    result
}

fn analyze_inner<A: Read + Seek, B: Read + Seek>(
    before: A,
    after: B,
    request: &AnalysisRequest,
    cancel: &AtomicBool,
    pixels: &mut RowObserver<'_>,
) -> Result<AnalysisReport> {
    checkpoint(cancel)?;
    request.validate()?;
    let mut a = PngRows::open(before, cancel).context("before PNG")?;
    let mut b = PngRows::open(after, cancel).context("after PNG")?;
    ensure!(
        a.dimensions == request.before.dimensions && b.dimensions == request.after.dimensions,
        "PNG dimensions disagree with capture evidence"
    );
    let da = a.dimensions;
    let db = b.dimensions;
    let canvas = Dimensions {
        width: da.width.max(db.width),
        height: da.height.max(db.height),
    };
    let overlap = u64::from(da.width.min(db.width)) * u64::from(da.height.min(db.height));
    let union = da.area() + db.area() - overlap;
    let dims = DimensionEvidence {
        before: da,
        after: db,
        canvas,
        width_delta: i64::from(db.width) - i64::from(da.width),
        height_delta: i64::from(db.height) - i64::from(da.height),
        overlap_pixels: overlap,
        before_only_pixels: da.area() - overlap,
        after_only_pixels: db.area() - overlap,
        absent_in_both_pixels: canvas.area() - union,
        before_only_bands: bands(da, db),
        after_only_bands: bands(db, da),
    };
    let structural = dims.before_only_pixels + dims.after_only_pixels;
    let mut out = AnalysisReport {
        schema_version: SCHEMA_VERSION,
        request: request.clone(),
        classification: Classification::NotComparable,
        gates: vec![],
        warnings: vec![],
        triggers: vec![],
        dimensions: dims,
        coordinate_union_pixels: union,
        ignored_pixels: 0,
        ignored_coverage: Fraction {
            numerator: 0,
            denominator: union,
        },
        eligible_pixels: 0,
        raw_changed_pixels: 0,
        raw_overlap_changed_pixels: 0,
        masked_raw_changed_pixels: 0,
        unmasked_raw_changed_pixels: 0,
        structural_pixels: structural,
        unmasked_structural_pixels: 0,
        changed_pixels: 0,
        changed_fraction: Fraction {
            numerator: 0,
            denominator: 0,
        },
        antialias_suppressed_pixels: 0,
        nonopaque_pixels: 0,
        raw_regions: RegionSummary::default(),
        regions: RegionSummary::default(),
        local: LocalSummary::default(),
    };
    let w = canvas.width as usize;
    let mut raw_regions = Regions::new(w);
    let mut regions = Regions::new(w);
    let mut windows = Windows::new(request.policy.local.as_ref(), canvas);
    let mut mask = vec![false; w];
    let mut mask_delta = vec![0i32; w + 1];
    let mut raw = vec![false; w];
    let mut changed = vec![false; w];
    let mut eligible = vec![false; w];
    for y in 0..canvas.height {
        checkpoint(cancel)?;
        let ar = if y < da.height { Some(a.row()?) } else { None };
        let br = if y < db.height { Some(b.row()?) } else { None };
        mask_delta.fill(0);
        if let Some(m) = &request.mask {
            for (i, r) in m.rectangles.iter().enumerate() {
                if i % 128 == 0 {
                    checkpoint(cancel)?;
                }
                if y >= r.y && y < r.y + r.height {
                    mask_delta[r.x as usize] += 1;
                    mask_delta[(r.x + r.width) as usize] -= 1;
                }
            }
        }
        let mut coverage = 0;
        for x in 0..w {
            coverage += mask_delta[x];
            mask[x] = coverage > 0;
        }
        for x in 0..w {
            if x % 256 == 0 {
                checkpoint(cancel)?;
            }
            let ap = ar.as_ref().and_then(|r| r.get(x));
            let bp = br.as_ref().and_then(|r| r.get(x));
            let present = ap.is_some() || bp.is_some();
            eligible[x] = present && !mask[x];
            if !present {
                raw[x] = false;
                changed[x] = false;
                continue;
            }
            if mask[x] {
                out.ignored_pixels += 1;
            } else {
                out.eligible_pixels += 1;
            }
            if ap.is_some_and(|p| p[3] != 255) || bp.is_some_and(|p| p[3] != 255) {
                out.nonopaque_pixels += 1;
            }
            let missing = ap.is_none() || bp.is_none();
            let unequal = missing || ap != bp;
            if unequal {
                out.raw_changed_pixels += 1;
                if !missing {
                    out.raw_overlap_changed_pixels += 1;
                }
                if mask[x] {
                    out.masked_raw_changed_pixels += 1;
                }
            }
            raw[x] = unequal && eligible[x];
            if raw[x] {
                out.unmasked_raw_changed_pixels += 1;
            }
            if missing && eligible[x] {
                out.unmasked_structural_pixels += 1;
            }
            // Non-opaque samples are raw-only inspection evidence, not composited guesses.
            let delta = match (ap, bp) {
                (Some(ap), Some(bp)) if unequal && ap[3] == 255 && bp[3] == 255 => delta_e00(
                    srgb_to_lab([ap[0], ap[1], ap[2]]),
                    srgb_to_lab([bp[0], bp[1], bp[2]]),
                ),
                _ => 0.0,
            };
            let significant = missing || delta > request.policy.pixel_tolerance_delta_e00;
            let suppressed = significant
                && !missing
                && eligible[x]
                && suppress_blend(
                    x,
                    ar.as_deref().unwrap(),
                    br.as_deref().unwrap(),
                    delta,
                    &request.policy.antialias,
                );
            if suppressed {
                out.antialias_suppressed_pixels += 1;
            }
            changed[x] = significant && eligible[x] && !suppressed;
            if changed[x] {
                out.changed_pixels += 1;
            }
        }
        raw_regions.row(&raw, y);
        regions.row(&changed, y);
        windows.row(&eligible, &changed, y);
        // Derivatives observe the very same significance/mask rows. The caller
        // stages output privately and may publish only after complete validation.
        pixels(y, &eligible, &changed)?;
    }
    a.finish()?;
    b.finish()?;
    checkpoint(cancel)?;
    out.raw_regions = raw_regions.finish();
    out.regions = regions.finish();
    out.local = windows.summary;
    out.changed_fraction = Fraction {
        numerator: out.changed_pixels,
        denominator: out.eligible_pixels,
    };
    out.ignored_coverage.numerator = out.ignored_pixels;
    if da.width != db.width {
        out.gates.push(GateReason::UnexpectedWidth);
    }
    if request.before.device_scale_factor != request.after.device_scale_factor {
        out.gates.push(GateReason::DeviceScaleMismatch);
    }
    if request.before.material_profile_id != request.after.material_profile_id {
        out.gates.push(GateReason::MaterialProfileMismatch);
    }
    if out.eligible_pixels == 0 {
        out.gates.push(GateReason::AllMasked);
    }
    if out.nonopaque_pixels > 0 {
        out.gates.push(GateReason::NonOpaquePixels);
    }
    if out
        .ignored_coverage
        .at_least(request.policy.ignored_warning_fraction)
    {
        out.warnings.push(Warning::LargeIgnoredCoverage);
    }
    if request.before.rendering_environment_id != request.after.rendering_environment_id {
        out.warnings.push(Warning::RenderingEnvironmentMismatch);
    }
    if out.changed_pixels > 0
        && request
            .policy
            .global_changed_fraction
            .is_some_and(|t| out.changed_fraction.at_least(t))
    {
        out.triggers.push(Trigger::GlobalFraction);
    }
    if out.local.qualifying > 0 {
        out.triggers.push(Trigger::LocalConcentration);
    }
    if request
        .policy
        .region_min_pixels
        .is_some_and(|t| out.regions.largest.as_ref().is_some_and(|r| r.pixels >= t))
    {
        out.triggers.push(Trigger::ConnectedRegion);
    }
    if request.policy.height_change_triggers && da.height != db.height {
        out.triggers.push(Trigger::HeightChange);
    }
    out.classification = if !out.gates.is_empty() {
        Classification::NotComparable
    } else if out.triggers.is_empty() {
        Classification::NoSignificantChange
    } else {
        Classification::Changed
    };
    Ok(out)
}

/// Bounded row observer for product derivatives. No report on error/cancellation;
/// callers must not publish staged pixels until this returns successfully.
pub fn analyze_pngs_with_rows<A: Read + Seek, B: Read + Seek>(
    before: A,
    after: B,
    request: &AnalysisRequest,
    cancel: &AtomicBool,
    pixels: &mut RowObserver<'_>,
) -> Result<AnalysisReport> {
    let result = analyze_inner(before, after, request, cancel, pixels);
    checkpoint(cancel)?;
    result
}

fn suppress_blend(
    x: usize,
    a: &[[u8; 4]],
    b: &[[u8; 4]],
    delta: f64,
    policy: &AntiAliasPolicy,
) -> bool {
    let AntiAliasPolicy::SharedHorizontalBlendV1 {
        max_delta_e00,
        channel_residual,
    } = policy
    else {
        return false;
    };
    if delta > *max_delta_e00 || x == 0 || x + 1 >= a.len().min(b.len()) {
        return false;
    }
    let left = a[x - 1];
    let right = a[x + 1];
    if left != b[x - 1]
        || right != b[x + 1]
        || [left, right, a[x], b[x]].iter().any(|p| p[3] != 255)
    {
        return false;
    }
    let mut strongest = 0;
    for c in 1..3 {
        if left[c].abs_diff(right[c]) > left[strongest].abs_diff(right[strongest]) {
            strongest = c;
        }
    }
    if left[strongest].abs_diff(right[strongest]) < 64 {
        return false;
    }
    for center in [a[x], b[x]] {
        let t = (f64::from(center[strongest]) - f64::from(left[strongest]))
            / (f64::from(right[strongest]) - f64::from(left[strongest]));
        if !(0.05..=0.95).contains(&t) {
            return false;
        }
        for c in 0..3 {
            let predicted = f64::from(left[c]) + t * (f64::from(right[c]) - f64::from(left[c]));
            if (f64::from(center[c]) - predicted).abs() > f64::from(*channel_residual) {
                return false;
            }
        }
    }
    true
}

// Canonical sRGB transfer, XYZ/D65 (IEC sRGB matrix), Lab, kL=kC=kH=1.
pub fn srgb_to_lab(rgb: [u8; 3]) -> [f64; 3] {
    let linear = rgb.map(|v| {
        let c = f64::from(v) / 255.0;
        if c <= 0.04045 {
            c / 12.92
        } else {
            ((c + 0.055) / 1.055).powf(2.4)
        }
    });
    let [r, g, b] = linear;
    let xyz = [
        (0.4124564 * r + 0.3575761 * g + 0.1804375 * b) / 0.95047,
        0.2126729 * r + 0.7151522 * g + 0.0721750 * b,
        (0.0193339 * r + 0.1191920 * g + 0.9503041 * b) / 1.08883,
    ];
    let [x, y, z] = xyz.map(|v| {
        if v > 216.0 / 24389.0 {
            v.cbrt()
        } else {
            (24389.0 / 27.0 * v + 16.0) / 116.0
        }
    });
    [116.0 * y - 16.0, 500.0 * (x - y), 200.0 * (y - z)]
}
pub fn delta_e00(a: [f64; 3], b: [f64; 3]) -> f64 {
    let [l1, a1, b1] = a;
    let [l2, a2, b2] = b;
    let c1 = a1.hypot(b1);
    let c2 = a2.hypot(b2);
    let cm = (c1 + c2) / 2.0;
    let g = 0.5 * (1.0 - (cm.powi(7) / (cm.powi(7) + 25f64.powi(7))).sqrt());
    let ap1 = (1.0 + g) * a1;
    let ap2 = (1.0 + g) * a2;
    let cp1 = ap1.hypot(b1);
    let cp2 = ap2.hypot(b2);
    let hue = |bb: f64, aa: f64| {
        if aa == 0.0 && bb == 0.0 {
            0.0
        } else {
            bb.atan2(aa).to_degrees().rem_euclid(360.0)
        }
    };
    let h1 = hue(b1, ap1);
    let h2 = hue(b2, ap2);
    let dl = l2 - l1;
    let dc = cp2 - cp1;
    let dh = if cp1 * cp2 == 0.0 {
        0.0
    } else if (h2 - h1).abs() <= 180.0 {
        h2 - h1
    } else if h2 <= h1 {
        h2 - h1 + 360.0
    } else {
        h2 - h1 - 360.0
    };
    let d_h = 2.0 * (cp1 * cp2).sqrt() * (dh.to_radians() / 2.0).sin();
    let lm = (l1 + l2) / 2.0;
    let cpm = (cp1 + cp2) / 2.0;
    let hm = if cp1 * cp2 == 0.0 {
        h1 + h2
    } else if (h1 - h2).abs() <= 180.0 {
        (h1 + h2) / 2.0
    } else if h1 + h2 < 360.0 {
        (h1 + h2 + 360.0) / 2.0
    } else {
        (h1 + h2 - 360.0) / 2.0
    };
    let cos = |v: f64| v.to_radians().cos();
    let t = 1.0 - 0.17 * cos(hm - 30.0) + 0.24 * cos(2.0 * hm) + 0.32 * cos(3.0 * hm + 6.0)
        - 0.20 * cos(4.0 * hm - 63.0);
    let sl = 1.0 + 0.015 * (lm - 50.0).powi(2) / (20.0 + (lm - 50.0).powi(2)).sqrt();
    let sc = 1.0 + 0.045 * cpm;
    let sh = 1.0 + 0.015 * cpm * t;
    let rt = -2.0
        * (cpm.powi(7) / (cpm.powi(7) + 25f64.powi(7))).sqrt()
        * (60.0 * (-((hm - 275.0) / 25.0).powi(2)).exp())
            .to_radians()
            .sin();
    ((dl / sl).powi(2) + (dc / sc).powi(2) + (d_h / sh).powi(2) + rt * (dc / sc) * (d_h / sh))
        .max(0.0)
        .sqrt()
}
// Frontier-only union-find. Compact every row, finalize components as soon as
// they leave the frontier; no all-page labels or changed bitmap is retained.
struct Regions {
    previous: Vec<usize>,
    active: Vec<Region>,
    summary: RegionSummary,
}
impl Regions {
    fn new(width: usize) -> Self {
        Self {
            previous: vec![usize::MAX; width],
            active: vec![],
            summary: RegionSummary::default(),
        }
    }
    fn retain(&mut self, r: Region) {
        self.summary.count += 1;
        self.summary.retained.push(r);
        self.summary.retained.sort_by_key(|r| {
            (
                std::cmp::Reverse(r.pixels),
                r.bounds.y,
                r.bounds.x,
                r.bounds.height,
                r.bounds.width,
            )
        });
        self.summary.retained.truncate(RETAINED_REGIONS);
    }
    fn row(&mut self, changed: &[bool], y: u32) {
        fn root(p: &mut [usize], mut i: usize) -> usize {
            while p[i] != i {
                p[i] = p[p[i]];
                i = p[i];
            }
            i
        }
        fn merge(a: &mut Region, b: &Region) {
            let right = (a.bounds.x + a.bounds.width).max(b.bounds.x + b.bounds.width);
            let bottom = (a.bounds.y + a.bounds.height).max(b.bounds.y + b.bounds.height);
            a.bounds.x = a.bounds.x.min(b.bounds.x);
            a.bounds.y = a.bounds.y.min(b.bounds.y);
            a.bounds.width = right - a.bounds.x;
            a.bounds.height = bottom - a.bounds.y;
            a.pixels += b.pixels;
        }
        let mut data = std::mem::take(&mut self.active);
        let mut parent: Vec<usize> = (0..data.len()).collect();
        let mut rank = vec![0u8; data.len()];
        let mut current = vec![usize::MAX; changed.len()];
        for (x, &on) in changed.iter().enumerate() {
            if !on {
                continue;
            }
            let node = data.len();
            data.push(Region {
                pixels: 1,
                bounds: Rectangle {
                    x: x as u32,
                    y,
                    width: 1,
                    height: 1,
                },
            });
            parent.push(node);
            rank.push(0);
            let mut r = node;
            for neighbor in [
                if x > 0 { current[x - 1] } else { usize::MAX },
                self.previous[x],
            ] {
                if neighbor == usize::MAX {
                    continue;
                }
                let n = root(&mut parent, neighbor);
                r = root(&mut parent, r);
                if r != n {
                    let (keep, remove) = if rank[r] > rank[n] { (r, n) } else { (n, r) };
                    let other = data[remove].clone();
                    merge(&mut data[keep], &other);
                    parent[remove] = keep;
                    if rank[r] == rank[n] {
                        rank[keep] += 1;
                    }
                    r = keep;
                }
            }
            current[x] = r;
        }
        let mut map = vec![usize::MAX; data.len()];
        for label in &mut current {
            if *label == usize::MAX {
                continue;
            }
            let r = root(&mut parent, *label);
            if map[r] == usize::MAX {
                map[r] = self.active.len();
                self.active.push(data[r].clone());
            }
            *label = map[r];
        }
        for (i, r) in data.into_iter().enumerate() {
            if parent[i] == i && map[i] == usize::MAX {
                self.retain(r);
            }
        }
        self.previous = current;
    }
    fn finish(mut self) -> RegionSummary {
        for r in std::mem::take(&mut self.active) {
            self.retain(r);
        }
        self.summary.largest = self.summary.retained.first().cloned();
        self.summary.omitted = self.summary.count - self.summary.retained.len() as u64;
        self.summary
    }
}
struct WindowRow {
    y: u32,
    counts: Vec<(u64, u64)>,
}
struct Windows<'a> {
    criterion: Option<&'a LocalCriterion>,
    canvas: Dimensions,
    active: Vec<WindowRow>,
    summary: LocalSummary,
}
impl<'a> Windows<'a> {
    fn new(criterion: Option<&'a LocalCriterion>, canvas: Dimensions) -> Self {
        Self {
            criterion,
            canvas,
            active: vec![],
            summary: LocalSummary::default(),
        }
    }
    fn row(&mut self, eligible: &[bool], changed: &[bool], y: u32) {
        let Some(c) = self.criterion else {
            return;
        };
        let cols = self.canvas.width.div_ceil(c.stride) as usize;
        if y.is_multiple_of(c.stride) {
            self.active.push(WindowRow {
                y,
                counts: vec![(0, 0); cols],
            });
        }
        let mut ep = vec![0u64; eligible.len() + 1];
        let mut cp = vec![0u64; changed.len() + 1];
        for x in 0..eligible.len() {
            ep[x + 1] = ep[x] + u64::from(eligible[x]);
            cp[x + 1] = cp[x] + u64::from(changed[x]);
        }
        for window in &mut self.active {
            for (col, (e, d)) in window.counts.iter_mut().enumerate() {
                let x = col * c.stride as usize;
                let end = (x + c.size as usize).min(eligible.len());
                *e += ep[end] - ep[x];
                *d += cp[end] - cp[x];
            }
        }
        let mut done = Vec::new();
        self.active.retain_mut(|r| {
            if y + 1 == (r.y + c.size).min(self.canvas.height) {
                done.push(WindowRow {
                    y: r.y,
                    counts: std::mem::take(&mut r.counts),
                });
                false
            } else {
                true
            }
        });
        for r in done {
            for (col, (e, d)) in r.counts.into_iter().enumerate() {
                self.summary.windows += 1;
                if e == 0 {
                    self.summary.zero_eligible += 1;
                }
                if e < c.min_eligible {
                    self.summary.insufficient_coverage += 1;
                    continue;
                }
                let x = col as u32 * c.stride;
                let evidence = WindowEvidence {
                    bounds: Rectangle {
                        x,
                        y: r.y,
                        width: c.size.min(self.canvas.width - x),
                        height: c.size.min(self.canvas.height - r.y),
                    },
                    changed: d,
                    eligible: e,
                };
                fn keep(slot: &mut Option<WindowEvidence>, evidence: &WindowEvidence) {
                    if slot.as_ref().is_none_or(|old| {
                        evidence.changed * old.eligible > old.changed * evidence.eligible
                            || (evidence.changed * old.eligible == old.changed * evidence.eligible
                                && evidence.changed > old.changed)
                    }) {
                        *slot = Some(evidence.clone());
                    }
                }
                keep(&mut self.summary.strongest, &evidence);
                if d >= c.min_changed
                    && (Fraction {
                        numerator: d,
                        denominator: e,
                    })
                    .at_least(c.changed_fraction)
                {
                    self.summary.qualifying += 1;
                    keep(&mut self.summary.strongest_qualifying, &evidence);
                }
            }
        }
    }
}

// Strict PNG envelope and independent bounded zlib/DEFLATE validation. png 0.17
// accepts compressed trailing data and may discard excess rows; Reader::finish
// alone is insufficient for this module's integrity contract.
struct CheckedRead<'a, R> {
    inner: R,
    cancel: &'a AtomicBool,
}
impl<R: Read> Read for CheckedRead<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(std::io::Error::other(Cancelled));
        }
        let n = buf.len().min(IO_CHUNK);
        self.inner.read(&mut buf[..n])
    }
}
struct PngRows<'a, R: Read> {
    reader: png::Reader<BufReader<CheckedRead<'a, R>>>,
    dimensions: Dimensions,
    samples: usize,
}
impl<'a, R: Read + Seek> PngRows<'a, R> {
    fn open(mut input: R, cancel: &'a AtomicBool) -> Result<Self> {
        validate_png(&mut input, cancel)?;
        input.seek(SeekFrom::Start(0))?;
        let mut decoder = png::Decoder::new_with_limits(
            BufReader::with_capacity(
                IO_CHUNK,
                CheckedRead {
                    inner: input,
                    cancel,
                },
            ),
            png::Limits {
                bytes: 8 * 1024 * 1024,
            },
        );
        decoder.ignore_checksums(false);
        decoder.set_ignore_text_chunk(true);
        decoder.set_ignore_iccp_chunk(true);
        let reader = decoder.read_info()?;
        let i = reader.info();
        let dimensions = Dimensions {
            width: i.width,
            height: i.height,
        };
        let samples = i.color_type.samples();
        Ok(Self {
            reader,
            dimensions,
            samples,
        })
    }
    fn row(&mut self) -> Result<Vec<[u8; 4]>> {
        let row = self.reader.next_row()?.context("PNG row missing")?;
        ensure!(
            row.data().len() == self.dimensions.width as usize * self.samples,
            "PNG row length mismatch"
        );
        Ok(row
            .data()
            .chunks_exact(self.samples)
            .map(|p| [p[0], p[1], p[2], if self.samples == 4 { p[3] } else { 255 }])
            .collect())
    }
    fn finish(&mut self) -> Result<()> {
        ensure!(self.reader.next_row()?.is_none(), "PNG excess rows");
        self.reader.finish().context("PNG completion")
    }
}
const fn crc_table() -> [u32; 256] {
    let mut table = [0; 256];
    let mut i = 0;
    while i < 256 {
        let mut c = i as u32;
        let mut j = 0;
        while j < 8 {
            c = if c & 1 != 0 {
                0xedb88320 ^ (c >> 1)
            } else {
                c >> 1
            };
            j += 1;
        }
        table[i] = c;
        i += 1;
    }
    table
}
const CRC_TABLE: [u32; 256] = crc_table();
fn crc_update(mut crc: u32, data: &[u8]) -> u32 {
    for &b in data {
        crc = CRC_TABLE[((crc ^ u32::from(b)) & 255) as usize] ^ (crc >> 8);
    }
    crc
}
fn read_u32<R: Read>(r: &mut R) -> Result<u32> {
    let mut b = [0; 4];
    r.read_exact(&mut b).context("PNG truncated chunk")?;
    Ok(u32::from_be_bytes(b))
}
/// Complete bounded integrity gate for browser-original screenshots and analysis.
/// Accepts noninterlaced RGB/RGBA8 with IHDR/consecutive IDAT/IEND, sRGB and
/// bounded tEXt only. Other formats/ancillary semantics fail closed rather than
/// being transformed into evidence the analysis engine cannot interpret.
/// Uses bounded chunk buffers, a 32 KiB inflate window, and cancellation checks;
/// the caller must run this blocking I/O outside the UI thread.
pub fn validate_png<R: Read + Seek>(input: &mut R, cancel: &AtomicBool) -> Result<()> {
    let bytes = input.seek(SeekFrom::End(0))?;
    ensure!(
        (45..=MAX_BYTES).contains(&bytes),
        "PNG file length outside limits"
    );
    input.seek(SeekFrom::Start(0))?;
    let mut signature = [0; 8];
    input.read_exact(&mut signature)?;
    ensure!(signature == *b"\x89PNG\r\n\x1a\n", "PNG signature mismatch");
    let mut dimensions = None;
    let mut samples = 0usize;
    let mut segments = Vec::new();
    let mut idat_ended = false;
    let mut seen_srgb = false;
    let mut chunks = 0;
    loop {
        checkpoint(cancel)?;
        chunks += 1;
        ensure!(chunks <= 10000, "PNG chunk count exceeds limit");
        let n = read_u32(input)? as usize;
        let mut kind = [0; 4];
        input
            .read_exact(&mut kind)
            .context("PNG truncated chunk type")?;
        let start = input.stream_position()?;
        ensure!(
            start
                .checked_add(n as u64 + 4)
                .is_some_and(|end| end <= bytes),
            "PNG truncated chunk payload"
        );
        ensure!(
            chunks != 1 || (&kind == b"IHDR" && n == 13),
            "PNG IHDR must be first and length 13"
        );
        // Unsupported ancillary semantics are rejected, not silently assumed sRGB.
        ensure!(
            matches!(&kind, b"IHDR" | b"IDAT" | b"IEND" | b"sRGB" | b"tEXt"),
            "PNG unsupported chunk {:?}",
            kind
        );
        if &kind == b"IHDR" {
            ensure!(chunks == 1 && n == 13, "PNG duplicate/malformed IHDR");
        }
        if &kind == b"IDAT" {
            ensure!(
                dimensions.is_some() && !idat_ended,
                "PNG nonconsecutive IDAT"
            );
            segments.push((start, n as u64));
        } else if !segments.is_empty() {
            idat_ended = true;
        }
        if &kind == b"sRGB" {
            ensure!(
                !seen_srgb && segments.is_empty() && n == 1,
                "PNG invalid sRGB order/length"
            );
            seen_srgb = true;
        }
        if &kind == b"tEXt" {
            ensure!(n <= 65536, "PNG text exceeds metadata limit");
        }
        if &kind == b"IEND" {
            ensure!(
                n == 0 && !segments.is_empty(),
                "PNG invalid IEND/missing IDAT"
            );
        }
        let mut crc = crc_update(!0, &kind);
        let mut remaining = n;
        let mut buffer = [0; IO_CHUNK];
        let mut small = Vec::new();
        while remaining > 0 {
            checkpoint(cancel)?;
            let count = remaining.min(IO_CHUNK);
            input
                .read_exact(&mut buffer[..count])
                .context("PNG truncated payload")?;
            crc = crc_update(crc, &buffer[..count]);
            if &kind != b"IDAT" {
                small.extend_from_slice(&buffer[..count]);
            }
            remaining -= count;
        }
        ensure!(read_u32(input)? == !crc, "PNG chunk CRC mismatch");
        match &kind {
            b"IHDR" => {
                let d = Dimensions {
                    width: u32::from_be_bytes(small[0..4].try_into().unwrap()),
                    height: u32::from_be_bytes(small[4..8].try_into().unwrap()),
                };
                d.validate()?;
                ensure!(
                    small[8] == 8 && matches!(small[9], 2 | 6) && small[10..13] == [0, 0, 0],
                    "PNG unsupported depth/color/compression/filter/interlace"
                );
                samples = if small[9] == 2 { 3 } else { 4 };
                dimensions = Some(d);
            }
            b"sRGB" => ensure!(small[0] <= 3, "PNG invalid sRGB rendering intent"),
            b"tEXt" => {
                let zero = small
                    .iter()
                    .position(|&v| v == 0)
                    .context("PNG text missing keyword separator")?;
                ensure!(
                    (1..=79).contains(&zero)
                        && small[..zero]
                            .iter()
                            .all(|&v| (32..=126).contains(&v) || v >= 161)
                        && small[0] != b' '
                        && small[zero - 1] != b' '
                        && !small[..zero].windows(2).any(|p| p == b"  ")
                        && !small[zero + 1..].contains(&0),
                    "PNG invalid text keyword/value"
                );
            }
            b"IEND" => {
                ensure!(
                    input.stream_position()? == bytes,
                    "PNG trailing bytes after IEND"
                );
                break;
            }
            _ => {}
        }
    }
    let d = dimensions.context("PNG missing dimensions")?;
    let raw_row_bytes = d.width as usize * samples + 1;
    let expected = raw_row_bytes as u64 * u64::from(d.height);
    let idat = IdatRead {
        input,
        segments,
        index: 0,
        offset: 0,
    };
    validate_zlib(
        CheckedRead {
            inner: idat,
            cancel,
        },
        expected,
        raw_row_bytes,
        cancel,
    )?;
    Ok(())
}
struct IdatRead<'a, R> {
    input: &'a mut R,
    segments: Vec<(u64, u64)>,
    index: usize,
    offset: u64,
}
impl<R: Read + Seek> Read for IdatRead<'_, R> {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if out.is_empty() {
            return Ok(0);
        }
        while let Some(&(start, count)) = self.segments.get(self.index) {
            if self.offset == count {
                self.index += 1;
                self.offset = 0;
                continue;
            }
            self.input.seek(SeekFrom::Start(start + self.offset))?;
            let n = out.len().min((count - self.offset) as usize);
            self.input.read_exact(&mut out[..n])?;
            self.offset += n as u64;
            return Ok(n);
        }
        Ok(0)
    }
}
struct Bits<R> {
    reader: BufReader<R>,
    byte: u8,
    remaining: u8,
}
impl<R: Read> Bits<R> {
    fn new(r: R) -> Self {
        Self {
            reader: BufReader::with_capacity(IO_CHUNK, r),
            byte: 0,
            remaining: 0,
        }
    }
    fn byte(&mut self) -> Result<u8> {
        let mut b = [0];
        self.reader
            .read_exact(&mut b)
            .context("zlib truncated stream")?;
        Ok(b[0])
    }
    fn bits(&mut self, n: u8) -> Result<u32> {
        let mut out = 0;
        for i in 0..n {
            if self.remaining == 0 {
                self.byte = self.byte()?;
                self.remaining = 8;
            }
            out |= u32::from(self.byte & 1) << i;
            self.byte >>= 1;
            self.remaining -= 1;
        }
        Ok(out)
    }
    fn align(&mut self) {
        self.remaining = 0;
    }
}
struct Huffman {
    counts: [u16; 16],
    symbols: Vec<u16>,
}
impl Huffman {
    fn new(lengths: &[u8], permit_single_or_empty: bool) -> Result<Self> {
        let mut counts = [0u16; 16];
        for &len in lengths {
            ensure!(len <= 15, "deflate invalid code length");
            if len > 0 {
                counts[len as usize] += 1;
            }
        }
        let used: u16 = counts.iter().sum();
        let mut left = 1i32;
        for count in counts.iter().skip(1) {
            left = left * 2 - i32::from(*count);
            ensure!(left >= 0, "deflate oversubscribed Huffman tree");
        }
        ensure!(
            left == 0 || (permit_single_or_empty && (used == 0 || (used == 1 && counts[1] == 1))),
            "deflate incomplete Huffman tree"
        );
        let mut symbols = Vec::with_capacity(used as usize);
        for len in 1..=15 {
            for (symbol, &n) in lengths.iter().enumerate() {
                if n == len {
                    symbols.push(symbol as u16);
                }
            }
        }
        Ok(Self { counts, symbols })
    }
    fn symbol<R: Read>(&self, b: &mut Bits<R>) -> Result<u16> {
        let mut code = 0u32;
        let mut first = 0u32;
        let mut index = 0usize;
        for len in 1..=15 {
            code = (code << 1) | b.bits(1)?;
            let count = u32::from(self.counts[len]);
            if code >= first && code - first < count {
                return Ok(self.symbols[index + (code - first) as usize]);
            }
            index += count as usize;
            first = (first + count) << 1;
        }
        bail!("deflate invalid Huffman symbol")
    }
}
fn trees<R: Read>(bits: &mut Bits<R>, dynamic: bool) -> Result<(Huffman, Huffman)> {
    if !dynamic {
        let lens: Vec<u8> = (0..288)
            .map(|i| match i {
                0..=143 => 8,
                144..=255 => 9,
                256..=279 => 7,
                _ => 8,
            })
            .collect();
        return Ok((Huffman::new(&lens, false)?, Huffman::new(&[5; 32], false)?));
    }
    let nl = bits.bits(5)? as usize + 257;
    let nd = bits.bits(5)? as usize + 1;
    let nc = bits.bits(4)? as usize + 4;
    ensure!(nl <= 286, "deflate reserved HLIT");
    let order = [
        16, 17, 18, 0, 8, 7, 9, 6, 10, 5, 11, 4, 12, 3, 13, 2, 14, 1, 15,
    ];
    let mut code_lengths = [0; 19];
    for &i in &order[..nc] {
        code_lengths[i] = bits.bits(3)? as u8;
    }
    let codes = Huffman::new(&code_lengths, false)?;
    let mut lens = Vec::with_capacity(nl + nd);
    while lens.len() < nl + nd {
        match codes.symbol(bits)? {
            v @ 0..=15 => lens.push(v as u8),
            v @ 16..=18 => {
                let (len, extra, base) = match v {
                    16 => (
                        *lens
                            .last()
                            .context("deflate repeat without previous length")?,
                        2,
                        3,
                    ),
                    17 => (0, 3, 3),
                    _ => (0, 7, 11),
                };
                let repeat = bits.bits(extra)? as usize + base;
                ensure!(
                    lens.len() + repeat <= nl + nd,
                    "deflate code-length repeat overflow"
                );
                lens.resize(lens.len() + repeat, len);
            }
            _ => bail!("deflate invalid code-length symbol"),
        }
    }
    ensure!(lens[256] != 0, "deflate missing end-of-block code");
    Ok((
        Huffman::new(&lens[..nl], true)?,
        Huffman::new(&lens[nl..], true)?,
    ))
}
fn validate_zlib<R: Read>(
    input: R,
    expected: u64,
    row_bytes: usize,
    cancel: &AtomicBool,
) -> Result<()> {
    let mut bits = Bits::new(input);
    let cmf = bits.byte()?;
    let flg = bits.byte()?;
    ensure!(
        cmf & 15 == 8
            && cmf >> 4 <= 7
            && (u16::from(cmf) * 256 + u16::from(flg)) % 31 == 0
            && flg & 32 == 0,
        "zlib invalid header/dictionary"
    );
    let window_limit = 1usize << ((cmf >> 4) + 8);
    let mut ring = [0u8; 32768];
    let mut produced = 0u64;
    let mut adler_a = 1u32;
    let mut adler_b = 0u32;
    let mut emit = |byte: u8, ring: &mut [u8; 32768], produced: &mut u64| -> Result<()> {
        if (*produced).is_multiple_of(IO_CHUNK as u64) {
            checkpoint(cancel)?;
        }
        ensure!(*produced < expected, "zlib excess decoded bytes");
        if (*produced).is_multiple_of(row_bytes as u64) {
            ensure!(byte <= 4, "PNG invalid scanline filter");
        }
        ring[*produced as usize % 32768] = byte;
        *produced += 1;
        adler_a = (adler_a + u32::from(byte)) % 65521;
        adler_b = (adler_b + adler_a) % 65521;
        Ok(())
    };
    const LB: [usize; 29] = [
        3, 4, 5, 6, 7, 8, 9, 10, 11, 13, 15, 17, 19, 23, 27, 31, 35, 43, 51, 59, 67, 83, 99, 115,
        131, 163, 195, 227, 258,
    ];
    const LE: [u8; 29] = [
        0, 0, 0, 0, 0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3, 4, 4, 4, 4, 5, 5, 5, 5, 0,
    ];
    const DB: [usize; 30] = [
        1, 2, 3, 4, 5, 7, 9, 13, 17, 25, 33, 49, 65, 97, 129, 193, 257, 385, 513, 769, 1025, 1537,
        2049, 3073, 4097, 6145, 8193, 12289, 16385, 24577,
    ];
    const DE: [u8; 30] = [
        0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
        13, 13,
    ];
    loop {
        checkpoint(cancel)?;
        let last = bits.bits(1)? != 0;
        match bits.bits(2)? {
            0 => {
                bits.align();
                let n = u16::from(bits.byte()?) | (u16::from(bits.byte()?) << 8);
                let inv = u16::from(bits.byte()?) | (u16::from(bits.byte()?) << 8);
                ensure!(n == !inv, "deflate stored length complement mismatch");
                for _ in 0..n {
                    emit(bits.byte()?, &mut ring, &mut produced)?;
                }
            }
            kind @ 1..=2 => {
                let (lit, dist) = trees(&mut bits, kind == 2)?;
                loop {
                    let v = lit.symbol(&mut bits)?;
                    match v {
                        0..=255 => emit(v as u8, &mut ring, &mut produced)?,
                        256 => break,
                        257..=285 => {
                            let i = (v - 257) as usize;
                            let len = LB[i] + bits.bits(LE[i])? as usize;
                            let d = dist.symbol(&mut bits)? as usize;
                            ensure!(d < 30, "deflate reserved distance symbol");
                            let distance = DB[d] + bits.bits(DE[d])? as usize;
                            ensure!(
                                distance <= window_limit && distance as u64 <= produced,
                                "deflate invalid backward distance"
                            );
                            for _ in 0..len {
                                let byte = ring[(produced as usize - distance) % 32768];
                                emit(byte, &mut ring, &mut produced)?;
                            }
                        }
                        _ => bail!("deflate reserved literal/length symbol"),
                    }
                }
            }
            _ => bail!("deflate reserved block type"),
        }
        if last {
            break;
        }
    }
    bits.align();
    let checksum = u32::from_be_bytes([bits.byte()?, bits.byte()?, bits.byte()?, bits.byte()?]);
    ensure!(
        checksum == adler_b * 65536 + adler_a,
        "zlib Adler32 mismatch"
    );
    ensure!(produced == expected, "zlib decoded byte count mismatch");
    let mut extra = [0];
    ensure!(
        bits.reader.read(&mut extra)? == 0,
        "zlib trailing compressed bytes"
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{fs::File, io::Cursor, path::PathBuf};

    fn request(a: Dimensions, b: Dimensions) -> AnalysisRequest {
        let target = Target {
            page: "page-fixture".into(),
            profile: "desktop-fixture".into(),
        };
        let descriptor = |dimensions, id: &str| CaptureDescriptor {
            capture_id: id.into(),
            target: target.clone(),
            dimensions,
            device_scale_factor: 1.0,
            material_profile_id: "material-v1".into(),
            rendering_environment_id: "rendering-v1".into(),
            completed: true,
        };
        AnalysisRequest {
            before: descriptor(a, "capture-before"),
            after: descriptor(b, "capture-after"),
            policy: Policy::seed(Preset::Balanced),
            mask: None,
        }
    }
    fn mask(req: &mut AnalysisRequest, rectangles: Vec<Rectangle>) {
        req.mask = Some(MaskBinding {
            revision: "mask-v1".into(),
            target: req.before.target.clone(),
            before_capture_id: req.before.capture_id.clone(),
            after_capture_id: req.after.capture_id.clone(),
            before_dimensions: req.before.dimensions,
            after_dimensions: req.after.dimensions,
            rectangles,
        });
    }
    fn encoded(
        d: Dimensions,
        pixels: &[[u8; 4]],
        filter: png::FilterType,
        compression: png::Compression,
    ) -> Vec<u8> {
        let mut bytes = Vec::new();
        {
            let mut encoder = png::Encoder::new(&mut bytes, d.width, d.height);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder.set_filter(filter);
            encoder.set_compression(compression);
            let flat: Vec<u8> = pixels.iter().flatten().copied().collect();
            encoder
                .write_header()
                .unwrap()
                .write_image_data(&flat)
                .unwrap();
        }
        bytes
    }
    fn png(d: Dimensions, pixels: &[[u8; 4]]) -> Vec<u8> {
        encoded(d, pixels, png::FilterType::NoFilter, png::Compression::Fast)
    }
    fn run(a: Vec<u8>, b: Vec<u8>, req: &AnalysisRequest) -> AnalysisReport {
        analyze_pngs(Cursor::new(a), Cursor::new(b), req, &AtomicBool::new(false)).unwrap()
    }
    fn corpus_root() -> PathBuf {
        std::env::var_os("REGRESSEE_ANALYSIS_CORPUS")
            .map(PathBuf::from)
            .unwrap_or_else(|| {
                PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("../../tests/fixtures/analysis-proposal")
            })
    }

    #[test]
    fn corpus_exact_oracles_and_seed_interpretations() {
        let root = corpus_root();
        let manifest: serde_json::Value = serde_json::from_reader(
            File::open(root.join("manifest.json"))
                .expect("set REGRESSEE_ANALYSIS_CORPUS in an isolated harness"),
        )
        .unwrap();
        let cases = manifest["cases"].as_array().unwrap();
        assert_eq!(cases.len(), 13);
        for case in cases {
            let id = case["id"].as_str().unwrap();
            let dims = |side: &str| Dimensions {
                width: case[side]["width"].as_u64().unwrap() as u32,
                height: case[side]["height"].as_u64().unwrap() as u32,
            };
            let mut req = request(dims("before"), dims("after"));
            let rectangles: Vec<Rectangle> = serde_json::from_value(case["masks"].clone()).unwrap();
            if !rectangles.is_empty() {
                mask(&mut req, rectangles);
            }
            let counts = &case["reference_counts"];
            let count = |s: &str| counts[s].as_u64().unwrap();
            for preset in [
                None,
                Some(Preset::Sensitive),
                Some(Preset::Balanced),
                Some(Preset::Relaxed),
            ] {
                req.policy = Policy::seed(preset.unwrap_or(Preset::Balanced));
                if preset.is_none() {
                    req.policy.pixel_tolerance_delta_e00 = 0.0;
                    req.policy.revision = "raw-oracle-v1".into();
                }
                let out = analyze_pngs(
                    File::open(root.join(case["before"]["path"].as_str().unwrap())).unwrap(),
                    File::open(root.join(case["after"]["path"].as_str().unwrap())).unwrap(),
                    &req,
                    &AtomicBool::new(false),
                )
                .unwrap();
                macro_rules! exact {
                    ($field:ident, $name:literal) => {
                        assert_eq!(out.$field, count($name), "{id}: {} {:?}", $name, preset);
                    };
                }
                exact!(coordinate_union_pixels, "canvas_pixels");
                exact!(eligible_pixels, "eligible_unmasked_pixels");
                exact!(ignored_pixels, "ignored_union_pixels");
                exact!(raw_changed_pixels, "raw_changed_pixels");
                exact!(raw_overlap_changed_pixels, "raw_overlap_rgb_changed_pixels");
                exact!(masked_raw_changed_pixels, "masked_raw_changed_pixels");
                exact!(unmasked_raw_changed_pixels, "unmasked_raw_changed_pixels");
                exact!(structural_pixels, "raw_structural_pixels");
                exact!(unmasked_structural_pixels, "unmasked_structural_pixels");
                assert_eq!(
                    out.raw_regions.count,
                    count("unmasked_raw_4_connected_components"),
                    "{id}"
                );
                assert_eq!(
                    out.raw_regions.largest.as_ref().map_or(0, |r| r.pixels),
                    count("largest_unmasked_raw_component_pixels"),
                    "{id}"
                );
                assert_eq!(
                    out.changed_fraction.denominator,
                    count("eligible_unmasked_pixels")
                );
                let significant_count = if preset.is_some() && id == "small-rgb-noise" {
                    0
                } else {
                    count("unmasked_raw_changed_pixels")
                };
                assert_eq!(out.changed_pixels, significant_count, "{id} {preset:?}");
                assert_eq!(
                    out.regions.count,
                    if significant_count == 0 {
                        0
                    } else {
                        out.raw_regions.count
                    },
                    "{id}"
                );
                if preset.is_none() {
                    assert_eq!(
                        out.changed_fraction.numerator,
                        counts["unmasked_changed_fraction"]["numerator"]
                            .as_u64()
                            .unwrap()
                    );
                    assert_eq!(
                        out.changed_fraction.value().is_some(),
                        counts["unmasked_changed_fraction"]["defined"]
                            .as_bool()
                            .unwrap()
                    );
                } else {
                    let expected = match id {
                        "entire-image-masked" | "unexpected-width-change" => {
                            Classification::NotComparable
                        }
                        "unchanged" | "small-rgb-noise" | "ignored-only-change" => {
                            Classification::NoSignificantChange
                        }
                        "ignored-rectangles" | "scattered-high-contrast-noise"
                            if preset == Some(Preset::Relaxed) =>
                        {
                            Classification::NoSignificantChange
                        }
                        _ => Classification::Changed,
                    };
                    assert_eq!(out.classification, expected, "{id} {preset:?}");
                }
                match id {
                    "tiny-icon-long-page" => {
                        assert_eq!(
                            out.changed_fraction,
                            Fraction {
                                numerator: 64,
                                denominator: 196608
                            }
                        );
                        assert_eq!(
                            out.regions.largest.as_ref().unwrap().bounds,
                            Rectangle {
                                x: 30,
                                y: 1020,
                                width: 8,
                                height: 8
                            }
                        );
                        assert!(out.triggers.contains(&Trigger::ConnectedRegion));
                        assert!(!out.triggers.contains(&Trigger::GlobalFraction));
                        assert_eq!(out.local.windows, 768);
                        assert_eq!(out.local.strongest.as_ref().unwrap().changed, 64);
                        if preset != Some(Preset::Relaxed) {
                            assert!(out.triggers.contains(&Trigger::LocalConcentration));
                        }
                    }
                    "large-coherent-region" => assert_eq!(
                        out.regions.largest.as_ref().unwrap().bounds,
                        Rectangle {
                            x: 16,
                            y: 16,
                            width: 80,
                            height: 48
                        }
                    ),
                    "scattered-high-contrast-noise" => assert_eq!(out.regions.omitted, 176),
                    "entire-image-masked" => {
                        assert_eq!(out.changed_fraction.value(), None);
                        assert!(out.gates.contains(&GateReason::AllMasked));
                        assert_eq!(out.local.zero_eligible, out.local.windows);
                    }
                    "near-entire-image-masked" => {
                        assert_eq!(out.changed_fraction.value(), Some(1.0));
                        assert_eq!(
                            out.ignored_coverage,
                            Fraction {
                                numerator: 6048,
                                denominator: 6144
                            }
                        );
                        assert!(out.warnings.contains(&Warning::LargeIgnoredCoverage));
                    }
                    "unexpected-width-change" => {
                        assert!(out.gates.contains(&GateReason::UnexpectedWidth))
                    }
                    "added-height-band" => assert_eq!(out.dimensions.height_delta, 16),
                    "removed-height-band" => assert_eq!(out.dimensions.height_delta, -16),
                    "inserted-middle-band" => {
                        assert_eq!(out.regions.count, 3);
                        assert_eq!(out.changed_pixels, 3072);
                    }
                    _ => {}
                }
                println!(
                    "corpus {id} {preset:?}: raw={} significant={}/{} regions={} largest={} local_qualifying={} {:?} {:?}",
                    out.raw_changed_pixels,
                    out.changed_pixels,
                    out.eligible_pixels,
                    out.regions.count,
                    out.regions.largest.as_ref().map_or(0, |r| r.pixels),
                    out.local.qualifying,
                    out.classification,
                    out.triggers
                );
                let json = serde_json::to_string(&out).unwrap();
                assert_eq!(serde_json::from_str::<AnalysisReport>(&json).unwrap(), out);
            }
        }
    }

    #[test]
    fn ciede2000_reference_vectors_and_srgb_d65() {
        // Sharma/Wu/Dalal supplemental reference examples (published ΔE00 rounded to 4 decimals).
        let vectors = [
            ([50.0, 2.6772, -79.7751], [50.0, 0.0, -82.7485], 2.0425),
            ([50.0, 3.1571, -77.2803], [50.0, 0.0, -82.7485], 2.8615),
            ([50.0, 2.8361, -74.0200], [50.0, 0.0, -82.7485], 3.4412),
            ([50.0, -1.3802, -84.2814], [50.0, 0.0, -82.7485], 1.0000),
            ([50.0, -1.1848, -84.8006], [50.0, 0.0, -82.7485], 1.0000),
            ([50.0, -0.9009, -85.5211], [50.0, 0.0, -82.7485], 1.0000),
            ([50.0, 0.0, 0.0], [50.0, -1.0, 2.0], 2.3669),
            ([50.0, 2.49, -0.001], [50.0, -2.49, 0.0009], 7.1792),
            ([50.0, 2.49, -0.001], [50.0, -2.49, 0.0010], 7.1792),
            ([50.0, 2.49, -0.001], [50.0, -2.49, 0.0011], 7.2195),
            ([50.0, -0.001, 2.49], [50.0, 0.0011, -2.49], 4.7461),
            ([50.0, 2.5, 0.0], [73.0, 25.0, -18.0], 27.1492),
            ([50.0, 2.5, 0.0], [61.0, -5.0, 29.0], 22.8977),
            ([50.0, 2.5, 0.0], [56.0, -27.0, -3.0], 31.9030),
            ([50.0, 2.5, 0.0], [58.0, 24.0, 15.0], 19.4535),
            (
                [2.0776, 0.0795, -1.1350],
                [0.9033, -0.0636, -0.5514],
                0.9082,
            ),
        ];
        for (a, b, expected) in vectors {
            assert!(
                (delta_e00(a, b) - expected).abs() < 0.00005,
                "{a:?} {b:?}: {} != {expected}",
                delta_e00(a, b)
            );
            assert!((delta_e00(a, b) - delta_e00(b, a)).abs() < 1e-12);
            assert_eq!(delta_e00(a, a), 0.0);
        }
        for (rgb, expected) in [
            ([0, 0, 0], [0.0, 0.0, 0.0]),
            ([255, 255, 255], [100.0, 0.0, 0.0]),
            ([255, 0, 0], [53.2408, 80.0925, 67.2032]),
            ([0, 255, 0], [87.7347, -86.1827, 83.1793]),
            ([0, 0, 255], [32.2970, 79.1875, -107.8602]),
        ] {
            let actual = srgb_to_lab(rgb);
            for c in 0..3 {
                assert!(
                    (actual[c] - expected[c]).abs() < 0.0002,
                    "{rgb:?}: {actual:?}"
                );
            }
        }
        let drift = delta_e00(srgb_to_lab([240; 3]), srgb_to_lab([241; 3]));
        assert!(drift > 0.1 && drift < 1.0);
    }

    #[test]
    fn pixel_strict_boundary_and_inclusive_global_region_thresholds() {
        let d = Dimensions {
            width: 2,
            height: 1,
        };
        let a = [[120, 120, 120, 255]; 2];
        let b = [[140, 140, 140, 255], a[1]];
        let delta = delta_e00(srgb_to_lab([120; 3]), srgb_to_lab([140; 3]));
        let mut req = request(d, d);
        req.policy.pixel_tolerance_delta_e00 = delta;
        req.policy.global_changed_fraction = Some(0.5);
        req.policy.region_min_pixels = Some(1);
        let out = run(png(d, &a), png(d, &b), &req);
        assert_eq!(out.changed_pixels, 0);
        assert_eq!(out.raw_changed_pixels, 1);
        req.policy.pixel_tolerance_delta_e00 = delta - 1e-10;
        let out = run(png(d, &a), png(d, &b), &req);
        assert_eq!(out.changed_fraction.value(), Some(0.5));
        assert_eq!(
            out.triggers,
            vec![Trigger::GlobalFraction, Trigger::ConnectedRegion]
        );
        req.policy.global_changed_fraction = Some(0.5 + 1e-10);
        req.policy.region_min_pixels = Some(2);
        assert_eq!(
            run(png(d, &a), png(d, &b), &req).classification,
            Classification::NoSignificantChange
        );
        req.policy.global_changed_fraction = Some(0.0);
        assert_eq!(
            run(png(d, &a), png(d, &a), &req).classification,
            Classification::NoSignificantChange
        );
    }

    #[test]
    fn optional_stationary_blend_suppression_does_not_align_layout() {
        let d = Dimensions {
            width: 3,
            height: 1,
        };
        let a = [[0, 0, 0, 255], [128, 128, 128, 255], [255, 255, 255, 255]];
        let b = [a[0], [132, 132, 132, 255], a[2]];
        let mut req = request(d, d);
        req.policy.pixel_tolerance_delta_e00 = 0.0;
        req.policy.region_min_pixels = Some(1);
        assert_eq!(run(png(d, &a), png(d, &b), &req).changed_pixels, 1);
        req.policy.antialias = AntiAliasPolicy::SharedHorizontalBlendV1 {
            max_delta_e00: 3.0,
            channel_residual: 1,
        };
        let out = run(png(d, &a), png(d, &b), &req);
        assert_eq!(out.raw_changed_pixels, 1);
        assert_eq!(out.changed_pixels, 0);
        assert_eq!(out.antialias_suppressed_pixels, 1);
        let shifted = [[0, 0, 0, 255], a[0], a[1]];
        let out = run(png(d, &a), png(d, &shifted), &req);
        assert_eq!(out.changed_pixels, 2);
        assert_eq!(out.antialias_suppressed_pixels, 0);
        let colored = [a[0], [132, 80, 132, 255], a[2]];
        assert_eq!(run(png(d, &a), png(d, &colored), &req).changed_pixels, 1);
    }

    #[test]
    fn masked_missing_band_keeps_structural_evidence_and_pair_binding() {
        let a = Dimensions {
            width: 4,
            height: 2,
        };
        let b = Dimensions {
            width: 4,
            height: 4,
        };
        let mut req = request(a, b);
        mask(
            &mut req,
            vec![Rectangle {
                x: 0,
                y: 2,
                width: 4,
                height: 2,
            }],
        );
        let out = run(png(a, &[[255; 4]; 8]), png(b, &[[255; 4]; 16]), &req);
        assert_eq!(out.structural_pixels, 8);
        assert_eq!(out.unmasked_structural_pixels, 0);
        assert_eq!(out.changed_pixels, 0);
        assert_eq!(out.eligible_pixels, 8);
        assert_eq!(out.classification, Classification::Changed);
        assert_eq!(out.triggers, vec![Trigger::HeightChange]);
        assert_eq!(
            out.dimensions.after_only_bands,
            vec![Rectangle {
                x: 0,
                y: 2,
                width: 4,
                height: 2
            }]
        );
        assert_eq!(out.warnings, vec![Warning::LargeIgnoredCoverage]);
        req.mask.as_mut().unwrap().after_dimensions.height = 2;
        assert!(
            req.validate()
                .unwrap_err()
                .to_string()
                .contains("stale mask binding")
        );
    }

    #[test]
    fn crossed_dimensions_do_not_count_absent_corner_as_content() {
        let a = Dimensions {
            width: 4,
            height: 2,
        };
        let b = Dimensions {
            width: 2,
            height: 4,
        };
        let req = request(a, b);
        let out = run(png(a, &[[255; 4]; 8]), png(b, &[[255; 4]; 8]), &req);
        assert_eq!(out.coordinate_union_pixels, 12);
        assert_eq!(out.dimensions.absent_in_both_pixels, 4);
        assert_eq!(out.eligible_pixels, 12);
        assert_eq!(out.structural_pixels, 8);
        assert_eq!(out.changed_pixels, 8);
        assert_eq!(out.regions.count, 2);
        assert_eq!(out.classification, Classification::NotComparable);
        let mut req = req;
        mask(
            &mut req,
            vec![Rectangle {
                x: 2,
                y: 2,
                width: 1,
                height: 1,
            }],
        );
        assert!(
            req.validate()
                .unwrap_err()
                .to_string()
                .contains("outside coordinate union")
        );
    }

    #[test]
    fn capture_failure_integrity_and_rendering_caution_are_separate() {
        let d = Dimensions {
            width: 1,
            height: 1,
        };
        let mut req = request(d, d);
        req.before.completed = false;
        assert!(
            analyze_pngs(
                Cursor::new([]),
                Cursor::new([]),
                &req,
                &AtomicBool::new(false)
            )
            .unwrap_err()
            .is::<CaptureFailure>()
        );
        req.before.completed = true;
        req.after.device_scale_factor = 2.0;
        req.after.material_profile_id = "changed-material-v2".into();
        req.after.rendering_environment_id = "other-os-fonts-v1".into();
        let out = run(png(d, &[[255; 4]]), png(d, &[[255; 4]]), &req);
        assert_eq!(out.classification, Classification::NotComparable);
        assert_eq!(
            out.gates,
            vec![
                GateReason::DeviceScaleMismatch,
                GateReason::MaterialProfileMismatch
            ]
        );
        assert_eq!(out.warnings, vec![Warning::RenderingEnvironmentMismatch]);
        req.after.device_scale_factor = 1.0;
        req.after.material_profile_id = req.before.material_profile_id.clone();
        let out = run(png(d, &[[255; 4]]), png(d, &[[255; 4]]), &req);
        assert_eq!(out.classification, Classification::NoSignificantChange);
        assert_eq!(out.warnings, vec![Warning::RenderingEnvironmentMismatch]);
        // No site URL is used as an incompatibility identity.
    }

    #[test]
    fn alpha_never_produces_a_trustworthy_clean_or_defect_verdict() {
        let d = Dimensions {
            width: 1,
            height: 1,
        };
        let req = request(d, d);
        let out = run(png(d, &[[255; 4]]), png(d, &[[255, 255, 255, 0]]), &req);
        assert_eq!(out.classification, Classification::NotComparable);
        assert_eq!(out.gates, vec![GateReason::NonOpaquePixels]);
        assert_eq!(out.raw_changed_pixels, 1);
        assert_eq!(out.changed_pixels, 0);
        let out = run(png(d, &[[0; 4]]), png(d, &[[0; 4]]), &req);
        assert_eq!(out.classification, Classification::NotComparable);
    }

    #[test]
    fn policy_schema_and_safe_mask_geometry_rejections() {
        let d = Dimensions {
            width: 4,
            height: 4,
        };
        let mut req = request(d, d);
        for r in [
            Rectangle {
                x: u32::MAX,
                y: 0,
                width: 2,
                height: 1,
            },
            Rectangle {
                x: 0,
                y: 0,
                width: 0,
                height: 1,
            },
            Rectangle {
                x: 0,
                y: 4,
                width: 1,
                height: 1,
            },
            Rectangle {
                x: 0,
                y: u32::MAX,
                width: 1,
                height: u32::MAX,
            },
        ] {
            mask(&mut req, vec![r]);
            assert!(
                req.validate()
                    .unwrap_err()
                    .to_string()
                    .contains("outside coordinate union")
            );
        }
        mask(
            &mut req,
            vec![Rectangle {
                x: 0,
                y: 0,
                width: 4,
                height: 4,
            }],
        );
        req.mask.as_mut().unwrap().target.profile = "other-profile".into();
        assert!(
            req.validate()
                .unwrap_err()
                .to_string()
                .contains("stale mask binding")
        );
        req.mask = None;
        for tolerance in [f64::NAN, f64::INFINITY, -1.0, 101.0] {
            req.policy.pixel_tolerance_delta_e00 = tolerance;
            assert!(
                req.validate()
                    .unwrap_err()
                    .to_string()
                    .contains("Delta E tolerance")
            );
        }
        req.policy = Policy::seed(Preset::Balanced);
        req.policy.schema_version += 1;
        assert!(
            req.validate()
                .unwrap_err()
                .to_string()
                .contains("unsupported policy")
        );
        req.policy = Policy::seed(Preset::Balanced);
        req.policy.algorithm = "rgba-max-channel-v1".into();
        assert!(
            req.validate()
                .unwrap_err()
                .to_string()
                .contains("unsupported policy")
        );
        let mut json = serde_json::to_value(Policy::seed(Preset::Balanced)).unwrap();
        json["confidence"] = serde_json::json!(0.9);
        assert!(
            serde_json::from_value::<Policy>(json)
                .unwrap_err()
                .to_string()
                .contains("unknown field")
        );
    }

    fn brute_regions(bitmap: &[bool], width: usize) -> Vec<Region> {
        let height = bitmap.len() / width;
        let mut visited = vec![false; bitmap.len()];
        let mut regions = Vec::new();
        for i in 0..bitmap.len() {
            if !bitmap[i] || visited[i] {
                continue;
            }
            visited[i] = true;
            let mut queue = vec![i];
            let mut next = 0;
            let (mut minx, mut miny, mut maxx, mut maxy) =
                (i % width, i / width, i % width, i / width);
            while next < queue.len() {
                let j = queue[next];
                next += 1;
                let (x, y) = (j % width, j / width);
                minx = minx.min(x);
                maxx = maxx.max(x);
                miny = miny.min(y);
                maxy = maxy.max(y);
                let neighbors = [
                    x.checked_sub(1).map(|x| y * width + x),
                    (x + 1 < width).then_some(j + 1),
                    y.checked_sub(1).map(|y| y * width + x),
                    (y + 1 < height).then_some(j + width),
                ];
                for n in neighbors.into_iter().flatten() {
                    if bitmap[n] && !visited[n] {
                        visited[n] = true;
                        queue.push(n);
                    }
                }
            }
            regions.push(Region {
                pixels: queue.len() as u64,
                bounds: Rectangle {
                    x: minx as u32,
                    y: miny as u32,
                    width: (maxx - minx + 1) as u32,
                    height: (maxy - miny + 1) as u32,
                },
            });
        }
        regions.sort_by_key(|r| {
            (
                std::cmp::Reverse(r.pixels),
                r.bounds.y,
                r.bounds.x,
                r.bounds.height,
                r.bounds.width,
            )
        });
        regions
    }
    #[test]
    fn streaming_components_match_independent_bfs_exhaustively() {
        for code in 0..512 {
            let bitmap: Vec<bool> = (0..9).map(|i| code & (1 << i) != 0).collect();
            let expected = brute_regions(&bitmap, 3);
            let mut tracker = Regions::new(3);
            for (y, row) in bitmap.chunks(3).enumerate() {
                tracker.row(row, y as u32);
            }
            let out = tracker.finish();
            assert_eq!(out.count, expected.len() as u64, "bitmap {code}");
            assert_eq!(out.retained, expected, "bitmap {code}");
        }
        let mut state = 1u64;
        for _ in 0..400 {
            let bitmap: Vec<bool> = (0..31 * 27)
                .map(|_| {
                    state = state.wrapping_mul(6364136223846793005).wrapping_add(1);
                    state >> 63 != 0
                })
                .collect();
            let expected = brute_regions(&bitmap, 31);
            let mut tracker = Regions::new(31);
            for (y, row) in bitmap.chunks(31).enumerate() {
                tracker.row(row, y as u32);
                assert!(tracker.active.len() <= 31);
            }
            let out = tracker.finish();
            assert_eq!(out.count, expected.len() as u64);
            assert_eq!(out.largest, expected.first().cloned());
            assert_eq!(
                out.retained,
                expected[..expected.len().min(RETAINED_REGIONS)]
            );
        }
    }

    #[test]
    fn local_windows_match_direct_oracle_with_masks_and_clipped_edges() {
        let dims = Dimensions {
            width: 49,
            height: 45,
        };
        let c = LocalCriterion {
            size: 32,
            stride: 16,
            min_eligible: 64,
            min_changed: 8,
            changed_fraction: 0.05,
        };
        let mut eligible = vec![true; dims.area() as usize];
        let mut changed = vec![false; eligible.len()];
        for y in 0..dims.height {
            for x in 0..dims.width {
                let i = (y * dims.width + x) as usize;
                eligible[i] = !(x < 32 && y < 32);
                changed[i] = eligible[i] && x % 3 == 0 && y % 2 == 0;
            }
        }
        let mut tracker = Windows::new(Some(&c), dims);
        for y in 0..dims.height as usize {
            let start = y * dims.width as usize;
            let end = start + dims.width as usize;
            tracker.row(&eligible[start..end], &changed[start..end], y as u32);
            assert!(tracker.active.len() <= 2);
        }
        let mut oracle = LocalSummary::default();
        for y in (0..dims.height).step_by(16) {
            for x in (0..dims.width).step_by(16) {
                let (mut e, mut d) = (0u64, 0u64);
                for yy in y..(y + 32).min(dims.height) {
                    for xx in x..(x + 32).min(dims.width) {
                        let i = (yy * dims.width + xx) as usize;
                        e += u64::from(eligible[i]);
                        d += u64::from(changed[i]);
                    }
                }
                oracle.windows += 1;
                if e == 0 {
                    oracle.zero_eligible += 1;
                }
                if e < 64 {
                    oracle.insufficient_coverage += 1;
                    continue;
                }
                if d >= 8 && d as f64 / e as f64 >= 0.05 {
                    oracle.qualifying += 1;
                }
            }
        }
        assert_eq!(tracker.summary.windows, oracle.windows);
        assert_eq!(tracker.summary.qualifying, oracle.qualifying);
        assert_eq!(tracker.summary.zero_eligible, oracle.zero_eligible);
        assert_eq!(
            tracker.summary.insufficient_coverage,
            oracle.insufficient_coverage
        );
        assert!(tracker.summary.insufficient_coverage > 0);
        assert!(tracker.summary.zero_eligible > 0);
    }

    fn chunk(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&(payload.len() as u32).to_be_bytes());
        out.extend_from_slice(kind);
        out.extend_from_slice(payload);
        out.extend_from_slice(&(!crc_update(crc_update(!0, kind), payload)).to_be_bytes());
        out
    }
    fn stored_zlib(raw: &[u8]) -> Vec<u8> {
        assert!(raw.len() <= 65535);
        let mut out = vec![0x78, 0x01, 0x01];
        let n = raw.len() as u16;
        out.extend_from_slice(&n.to_le_bytes());
        out.extend_from_slice(&(!n).to_le_bytes());
        out.extend_from_slice(raw);
        let (mut a, mut b) = (1u32, 0u32);
        for &v in raw {
            a = (a + u32::from(v)) % 65521;
            b = (b + a) % 65521;
        }
        out.extend_from_slice(&(b * 65536 + a).to_be_bytes());
        out
    }
    fn header(width: u32, height: u32) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(&width.to_be_bytes());
        out.extend_from_slice(&height.to_be_bytes());
        out.extend_from_slice(&[8, 2, 0, 0, 0]);
        out
    }
    fn from_chunks(chunks: &[Vec<u8>]) -> Vec<u8> {
        let mut out = b"\x89PNG\r\n\x1a\n".to_vec();
        for c in chunks {
            out.extend_from_slice(c);
        }
        out
    }
    fn with_zlib(z: &[u8]) -> Vec<u8> {
        from_chunks(&[
            chunk(b"IHDR", &header(1, 1)),
            chunk(b"IDAT", z),
            chunk(b"IEND", &[]),
        ])
    }
    fn reject(bytes: Vec<u8>, expected: &str) {
        let error = validate_png(&mut Cursor::new(bytes), &AtomicBool::new(false)).unwrap_err();
        assert!(
            format!("{error:#}").contains(expected),
            "expected {expected:?}, got {error:#}"
        );
    }

    #[test]
    fn complete_png_integrity_checks_have_specific_failure_oracles() {
        let raw = [0, 240, 240, 240];
        let z = stored_zlib(&raw);
        let valid = with_zlib(&z);
        validate_png(&mut Cursor::new(&valid), &AtomicBool::new(false)).unwrap();
        let mut bad = valid.clone();
        bad[0] ^= 1;
        reject(bad, "PNG signature mismatch");
        let mut bad = valid.clone();
        bad[29] ^= 1;
        reject(bad, "PNG chunk CRC mismatch");
        let mut bad = valid.clone();
        let len = bad.len();
        bad[len - 1] ^= 1;
        reject(bad, "PNG chunk CRC mismatch");
        let mut bad = valid.clone();
        bad.truncate(bad.len() - 1);
        reject(bad, "PNG truncated chunk payload");
        let mut bad = valid.clone();
        bad.truncate(bad.len() - 12);
        reject(bad, "PNG truncated chunk");
        let mut bad = valid.clone();
        bad.push(0);
        reject(bad, "PNG trailing bytes after IEND");
        let mut bad = valid.clone();
        bad.extend_from_slice(&valid);
        reject(bad, "PNG trailing bytes after IEND");
        reject(
            from_chunks(&[
                chunk(b"IHDR", &header(u32::MAX, 1)),
                chunk(b"IDAT", &z),
                chunk(b"IEND", &[]),
            ]),
            "geometry exceeds analysis limits",
        );
        reject(
            from_chunks(&[
                chunk(b"IHDR", &header(1, 1)),
                chunk(b"IHDR", &header(1, 1)),
                chunk(b"IDAT", &z),
                chunk(b"IEND", &[]),
            ]),
            "PNG duplicate/malformed IHDR",
        );
        reject(
            from_chunks(&[
                chunk(b"IHDR", &header(1, 1)),
                chunk(b"acTL", &[0; 8]),
                chunk(b"IDAT", &z),
                chunk(b"IEND", &[]),
            ]),
            "PNG unsupported chunk",
        );
        let mut interlaced = header(1, 1);
        interlaced[12] = 1;
        reject(
            from_chunks(&[
                chunk(b"IHDR", &interlaced),
                chunk(b"IDAT", &z),
                chunk(b"IEND", &[]),
            ]),
            "PNG unsupported depth/color/compression/filter/interlace",
        );
        let mut bad = z.clone();
        let len = bad.len();
        bad[len - 1] ^= 1;
        reject(with_zlib(&bad), "zlib Adler32 mismatch"); // Outer CRC is VALID.
        let mut bad = z.clone();
        bad.push(0);
        reject(with_zlib(&bad), "zlib trailing compressed bytes");
        reject(
            with_zlib(&stored_zlib(&[0, 1, 2, 3, 4])),
            "zlib excess decoded bytes",
        );
        reject(
            with_zlib(&stored_zlib(&[0, 1, 2])),
            "zlib decoded byte count mismatch",
        );
        reject(
            with_zlib(&stored_zlib(&[5, 1, 2, 3])),
            "PNG invalid scanline filter",
        );
        let mut bad = z.clone();
        bad[5] ^= 1;
        reject(with_zlib(&bad), "deflate stored length complement mismatch");
        let mut bad = z.clone();
        bad.truncate(bad.len() - 2);
        reject(with_zlib(&bad), "zlib truncated stream");
        let mut bad = z.clone();
        bad[2] = 7;
        reject(with_zlib(&bad), "deflate reserved block type");
        let mut bad = z.clone();
        bad[0] = 0x79;
        reject(with_zlib(&bad), "zlib invalid header/dictionary");
        reject(
            from_chunks(&[
                chunk(b"IHDR", &header(1, 1)),
                chunk(b"IDAT", &z[..4]),
                chunk(b"tEXt", b"test\0value"),
                chunk(b"IDAT", &z[4..]),
                chunk(b"IEND", &[]),
            ]),
            "PNG nonconsecutive IDAT",
        );
        let mut bad_text = chunk(b"tEXt", b"test\0value");
        let n = bad_text.len();
        bad_text[n - 1] ^= 1;
        reject(
            from_chunks(&[
                chunk(b"IHDR", &header(1, 1)),
                chunk(b"IDAT", &z),
                bad_text,
                chunk(b"IEND", &[]),
            ]),
            "PNG chunk CRC mismatch",
        );
        // Concatenated zlib streams in IDAT are not mistaken for one complete image.
        let mut doubled = z.clone();
        doubled.extend_from_slice(&z);
        reject(with_zlib(&doubled), "zlib trailing compressed bytes");
    }

    #[test]
    fn valid_split_idat_empty_chunks_and_post_image_text_are_checked() {
        let z = stored_zlib(&[0, 240, 240, 240]);
        let mut chunks = vec![
            chunk(b"IHDR", &header(1, 1)),
            chunk(b"sRGB", &[0]),
            chunk(b"IDAT", &[]),
        ];
        for byte in z {
            chunks.push(chunk(b"IDAT", &[byte]));
            chunks.push(chunk(b"IDAT", &[]));
        }
        chunks.push(chunk(b"tEXt", b"Regressee test\0valid post-image metadata"));
        chunks.push(chunk(b"IEND", &[]));
        let bytes = from_chunks(&chunks);
        let d = Dimensions {
            width: 1,
            height: 1,
        };
        let out = run(bytes.clone(), bytes, &request(d, d));
        assert_eq!(out.classification, Classification::NoSignificantChange);
        assert_eq!(out.eligible_pixels, 1);
    }

    #[test]
    fn independent_png_encoder_all_filters_and_compressions() {
        let d = Dimensions {
            width: 137,
            height: 29,
        };
        let pixels: Vec<[u8; 4]> = (0..d.area())
            .map(|i| {
                [
                    (i % 256) as u8,
                    (i * 73 % 256) as u8,
                    (i * 139 % 256) as u8,
                    255,
                ]
            })
            .collect();
        let reference = png(d, &pixels);
        for filter in [
            png::FilterType::NoFilter,
            png::FilterType::Sub,
            png::FilterType::Up,
            png::FilterType::Avg,
            png::FilterType::Paeth,
        ] {
            for compression in [
                png::Compression::Fast,
                png::Compression::Default,
                png::Compression::Best,
            ] {
                let out = run(
                    reference.clone(),
                    encoded(d, &pixels, filter, compression),
                    &request(d, d),
                );
                assert_eq!(out.raw_changed_pixels, 0, "{filter:?} {compression:?}");
                assert_eq!(out.eligible_pixels, d.area());
            }
        }
    }

    struct CancelOnRead<'a> {
        input: Cursor<Vec<u8>>,
        read: usize,
        trigger: usize,
        cancel: &'a AtomicBool,
    }
    impl Read for CancelOnRead<'_> {
        fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
            let n = self.input.read(out)?;
            self.read += n;
            if self.read >= self.trigger {
                self.cancel.store(true, Ordering::Relaxed);
            }
            Ok(n)
        }
    }
    impl Seek for CancelOnRead<'_> {
        fn seek(&mut self, p: SeekFrom) -> std::io::Result<u64> {
            self.input.seek(p)
        }
    }
    #[test]
    fn cancellation_is_observed_in_envelope_inflate_and_analysis_rows() {
        let d = Dimensions {
            width: 96,
            height: 2048,
        };
        let bytes =
            std::fs::read(corpus_root().join("images/tiny-icon-long-page-before.png")).unwrap();
        let req = request(d, d);
        let cancel = AtomicBool::new(true);
        assert!(
            analyze_pngs(Cursor::new(&bytes), Cursor::new(&bytes), &req, &cancel)
                .unwrap_err()
                .is::<Cancelled>()
        );
        // Bound IO requests in all passes; force cancellation during envelope,
        // strict inflation, and later row decoding (two envelope/inflate passes).
        for trigger in [100, bytes.len() + 100, 2 * bytes.len() + 3 * IO_CHUNK] {
            let cancel = AtomicBool::new(false);
            let input = CancelOnRead {
                input: Cursor::new(bytes.clone()),
                read: 0,
                trigger,
                cancel: &cancel,
            };
            let error = analyze_pngs(input, Cursor::new(&bytes), &req, &cancel).unwrap_err();
            assert!(error.is::<Cancelled>(), "trigger {trigger}: {error:#}");
        }
        let raw = vec![0; 40000];
        let z = stored_zlib(&raw);
        let cancel = AtomicBool::new(false);
        let reader = CancelOnRead {
            input: Cursor::new(z),
            read: 0,
            trigger: 100,
            cancel: &cancel,
        };
        let error = validate_zlib(reader, raw.len() as u64, 4, &cancel).unwrap_err();
        assert!(format!("{error:#}").contains("cancelled"));
    }

    #[test]
    fn many_components_and_tall_frontier_do_not_accumulate_page_labels() {
        let mut tracker = Regions::new(64);
        let row: Vec<bool> = (0..64).map(|x| x % 2 == 0).collect();
        for y in 0..32768 {
            tracker.row(if y % 2 == 0 { &row } else { &[false; 64] }, y);
            assert!(tracker.active.len() <= 32);
            assert!(tracker.summary.retained.len() <= RETAINED_REGIONS);
        }
        let out = tracker.finish();
        assert_eq!(out.count, 524288);
        assert_eq!(out.largest.unwrap().pixels, 1);
        assert_eq!(out.omitted, 524272);
    }

    #[test]
    fn local_thresholds_are_inclusive_and_low_coverage_is_excluded() {
        let dims = Dimensions {
            width: 8,
            height: 8,
        };
        let eligible = [true; 8];
        let mut c = LocalCriterion {
            size: 16,
            stride: 16,
            min_eligible: 64,
            min_changed: 8,
            changed_fraction: 0.125,
        };
        for (threshold, count, coverage, expected, excluded) in [
            (0.125, 8, 64, 1, 0),
            (0.125 + 1e-10, 8, 64, 0, 0),
            (0.125, 9, 64, 0, 0),
            (0.125, 8, 65, 0, 1),
        ] {
            c.changed_fraction = threshold;
            c.min_changed = count;
            c.min_eligible = coverage;
            let mut w = Windows::new(Some(&c), dims);
            for y in 0..8 {
                w.row(&eligible, &[y == 0; 8], y);
            }
            assert_eq!(w.summary.qualifying, expected);
            assert_eq!(w.summary.insufficient_coverage, excluded);
        }
    }

    #[test]
    fn seed_values_match_the_read_only_proposal_not_old_percentage_schema() {
        let seeds: serde_json::Value =
            serde_json::from_reader(File::open(corpus_root().join("seed-policies.json")).unwrap())
                .unwrap();
        for (i, preset) in [Preset::Sensitive, Preset::Balanced, Preset::Relaxed]
            .into_iter()
            .enumerate()
        {
            let p = Policy::seed(preset);
            let oracle = &seeds["presets"][i];
            assert_eq!(
                p.pixel_tolerance_delta_e00,
                oracle["pixel_tolerance_delta_e_00"].as_f64().unwrap()
            );
            assert_eq!(
                p.global_changed_fraction.unwrap(),
                oracle["global_changed_fraction"].as_f64().unwrap()
            );
            let local = p.local.as_ref().unwrap();
            assert_eq!(
                local.changed_fraction,
                oracle["local_changed_fraction"].as_f64().unwrap()
            );
            assert_eq!(
                local.min_changed,
                oracle["local_min_changed_pixels"].as_u64().unwrap()
            );
            assert_eq!(
                p.region_min_pixels.unwrap(),
                oracle["connected_region_min_pixels"].as_u64().unwrap()
            );
            assert_eq!(p.antialias, AntiAliasPolicy::Off);
            assert_eq!(local.size, 32);
            assert_eq!(local.stride, 16);
            assert_eq!(local.min_eligible, 64);
            p.validate().unwrap();
        }
    }

    #[test]
    fn png_chunk_metadata_and_geometry_limits_are_not_generic_errors() {
        let z = stored_zlib(&[0, 240, 240, 240]);
        let mut chunks = vec![chunk(b"IHDR", &header(1, 1))];
        for _ in 0..10000 {
            chunks.push(chunk(b"IDAT", &[]));
        }
        chunks.push(chunk(b"IDAT", &z));
        chunks.push(chunk(b"IEND", &[]));
        reject(from_chunks(&chunks), "PNG chunk count exceeds limit");
        reject(
            from_chunks(&[
                chunk(b"IHDR", &header(4096, 20000)),
                chunk(b"IDAT", &z),
                chunk(b"IEND", &[]),
            ]),
            "geometry exceeds analysis limits",
        );
        reject(
            from_chunks(&[
                chunk(b"IHDR", &header(1, 1)),
                chunk(b"tEXt", &vec![b'x'; 65537]),
                chunk(b"IDAT", &z),
                chunk(b"IEND", &[]),
            ]),
            "PNG text exceeds metadata limit",
        );
        reject(
            from_chunks(&[
                chunk(b"IHDR", &header(1, 1)),
                chunk(b"tEXt", b"missing-separator"),
                chunk(b"IDAT", &z),
                chunk(b"IEND", &[]),
            ]),
            "PNG text missing keyword separator",
        );
        reject(
            from_chunks(&[
                chunk(b"IHDR", &header(1, 1)),
                chunk(b"gAMA", &45455u32.to_be_bytes()),
                chunk(b"IDAT", &z),
                chunk(b"IEND", &[]),
            ]),
            "PNG unsupported chunk",
        );
        let d = Dimensions {
            width: 2,
            height: 1,
        };
        let error = analyze_pngs(
            Cursor::new(with_zlib(&z)),
            Cursor::new(with_zlib(&z)),
            &request(d, d),
            &AtomicBool::new(false),
        )
        .unwrap_err();
        assert!(
            error
                .to_string()
                .contains("PNG dimensions disagree with capture evidence")
        );
    }
}
