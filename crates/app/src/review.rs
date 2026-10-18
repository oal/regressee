// SPDX-License-Identifier: MPL-2.0
//! Actual post-capture services. Called only by the bounded blocking owner.
use crate::analysis_engine as e;
use anyhow::{Result, ensure};
use regressee_core::{domain::*, evidence::*};
use regressee_storage::Store;
use std::{
    fs::File,
    io::Write,
    sync::atomic::{AtomicBool, Ordering},
};

pub fn preset(preset: e::Preset) -> e::Policy {
    let mut policy = e::Policy::seed(preset);
    policy.revision = format!(
        "fixture-calibrated-{}-v1",
        match preset {
            e::Preset::Sensitive => "sensitive",
            e::Preset::Balanced => "balanced",
            e::Preset::Relaxed => "relaxed",
        }
    );
    policy
}
pub fn edited_policy(mut policy: e::Policy) -> Result<e::Policy> {
    policy.revision = "advanced-v1".into();
    policy.validate()?;
    policy.revision = format!("advanced-v1-{}", identity_hash(&policy)?);
    Ok(policy)
}
pub fn analyze_run(
    store: &mut Store,
    id: RunId,
    policy: e::Policy,
    cancel: &AtomicBool,
) -> Result<ComparisonEvidence> {
    policy.validate()?;
    if cancel.load(Ordering::Acquire) {
        return Err(e::Cancelled.into());
    }
    let run = store.load_screenshot_run(id)?;
    ensure!(
        run.run.capture_plan.mode.screenshots(),
        "This run has no requested screenshots. Analyze its saved DOM snapshots instead."
    );
    // Beginning an analysis invalidates previews, including cancellation/cache hits.
    store.invalidate_approval(id)?;
    let baseline_id = match run.run.intent {
        RunIntent::Check { baseline } => baseline,
        _ => anyhow::bail!("A check against a baseline is required"),
    };
    ensure!(
        run.run.state.terminal(),
        "Capture must finish before reanalysis"
    );
    let baseline = store.load_screenshot_baseline(baseline_id)?;
    ensure!(
        !baseline.references.is_empty(),
        "This baseline has no saved screenshot references. Its screenshot coverage is omitted; inspect DOM snapshots or capture a new baseline."
    );
    let mut requests = Vec::new();
    // Keep one fully verified source snapshot within this worker operation.
    // A 1,000-page baseline otherwise rehashes every source artifact 1,000 times.
    // This is bounded, never shared across operations, and never bypasses the
    // next analysis/consent's fresh disk verification.
    let mut source: Option<RunRecord> = None;
    for reference in &baseline.references {
        ensure!(!cancel.load(Ordering::Acquire), "analysis cancelled");
        if source.as_ref().is_none_or(|s| s.run.id != reference.run) {
            source = Some(store.load_screenshot_run(reference.run)?);
        }
        let old = source.as_ref().unwrap();
        if let (Some(before), Some(after)) =
            (old.evidence(&reference.key), run.evidence(&reference.key))
        {
            requests.push((
                reference.key.clone(),
                e::AnalysisRequest {
                    before: descriptor(old, before)?,
                    after: descriptor(&run, after)?,
                    policy: policy.clone(),
                    mask: store.latest_mask(run.run.project, &reference.key)?,
                },
            ));
        }
    }
    // Includes exact source run/artifact IDs, material/rendering identities,
    // complete policy semantics/revision and mask pair/dimensions/revision.
    let cache_key = identity_hash(&(
        "capture-cautions-advisory-v2",
        baseline_id,
        id,
        &policy,
        &requests,
    ))?;
    if let Some(cached) = store.cached_analysis(id, &cache_key)? {
        if cancel.load(Ordering::Acquire) {
            return Err(e::Cancelled.into());
        }
        store.display_analysis(cached.revision.id)?;
        return Ok(cached);
    }
    let mut results = Vec::new();
    let mut reports = Vec::new();
    let mut failures = Vec::new();
    let mut heatmaps = Vec::new();
    let reference_keys: std::collections::BTreeSet<_> =
        baseline.references.iter().map(|r| &r.key).collect();
    for target in run
        .run
        .targets
        .iter()
        .filter(|t| reference_keys.contains(&t.key))
    {
        if cancel.load(Ordering::Acquire) {
            return Err(e::Cancelled.into());
        }
        let mut result = AnalysisResult {
            key: target.key.clone(),
            classification: Classification::CouldNotCapture,
            eligible_pixels: 0,
            changed_pixels: 0,
            dimensions_changed: false,
        };
        if let Some((_, request)) = requests.iter().find(|(key, _)| key == &target.key) {
            let before = baseline
                .references
                .iter()
                .find(|r| r.key == target.key)
                .unwrap();
            let after = run.evidence(&target.key).unwrap();
            match analyze_pair(
                store,
                &before.artifact,
                &after.image.original,
                request,
                cancel,
            ) {
                Ok((report, heatmap)) => {
                    result.classification = if report
                        .warnings
                        .contains(&e::Warning::RenderingEnvironmentMismatch)
                    {
                        Classification::NotComparable
                    } else {
                        match report.classification {
                            e::Classification::Changed => Classification::Changed,
                            e::Classification::NoSignificantChange => {
                                Classification::NoSignificantChange
                            }
                            e::Classification::NotComparable => Classification::NotComparable,
                        }
                    };
                    result.eligible_pixels = report.eligible_pixels;
                    result.changed_pixels = report.changed_pixels;
                    result.dimensions_changed =
                        report.dimensions.height_delta != 0 || report.dimensions.width_delta != 0;
                    reports.push((target.key.clone(), report));
                    heatmaps.push((target.key.clone(), heatmap));
                }
                Err(error) => {
                    if cancel.load(Ordering::Acquire) || error.is::<e::Cancelled>() {
                        return Err(e::Cancelled.into());
                    }
                    result.classification = Classification::NotComparable;
                    failures.push((
                        target.key.clone(),
                        format!("Analysis failed: {error:#}")
                            .chars()
                            .take(2048)
                            .collect(),
                    ));
                }
            }
        }
        results.push(result);
    }
    if cancel.load(Ordering::Acquire) {
        return Err(e::Cancelled.into());
    }
    let a = ComparisonEvidence {
        revision: AnalysisRevision {
            id: AnalysisId::new(),
            baseline: baseline_id,
            run: id,
            algorithm: e::ALGORITHM_ID.into(),
            // Historical RGB schema is retained only for backcompat; never stores ΔE.
            policy: AnalysisPolicy {
                color_tolerance: 0.,
                changed_percent: 0.,
                region_pixels: 0,
            },
            mask: None,
            results,
        },
        heatmap: None,
        heatmaps,
        perceptual: Some(PerceptualEvidence {
            policy,
            cache_key,
            created_at: regressee_runtime::distribution::now(),
            reports,
            failures,
        }),
    };
    store.save_analysis(&a)?;
    Ok(a)
}

/// Analyze each requested kind from its retained originals. This never starts a
/// browser, changes the frozen capture selection, or applies image masks or
/// sensitivity settings to DOM evidence. Legacy `analyze_run` remains image-only.
pub fn analyze_saved_run(
    store: &mut Store,
    id: RunId,
    policy: e::Policy,
    cancel: &AtomicBool,
) -> Result<()> {
    if cancel.load(Ordering::Acquire) {
        return Err(e::Cancelled.into());
    }
    // Each kind has its own integrity authority. Do not replace a failed read
    // with an aggregate fallback or let an unrelated corrupt sibling suppress
    // the healthy kind's analysis. Both operations run before returning errors.
    let screenshot = match store.load_run_kind(id, CheckKind::Screenshot) {
        Ok(run)
            if run
                .run
                .targets
                .iter()
                .any(|t| !matches!(t.outcome, TargetOutcome::NotRequested)) =>
        {
            analyze_run(store, id, policy, cancel).map(|_| ())
        }
        Ok(_) => Ok(()),
        Err(error) => Err(error),
    };
    let dom = match store.load_run_kind(id, CheckKind::DomSnapshot) {
        Ok(run) if run.run.capture_plan.mode.dom_snapshots() => {
            crate::dom_snapshot::analyze_run(store, id, cancel).map(|_| ())
        }
        Ok(_) => Ok(()),
        Err(error) => Err(error),
    };
    screenshot.and(dom)
}
fn analyze_pair(
    store: &mut Store,
    before: &ArtifactId,
    after: &ArtifactId,
    request: &e::AnalysisRequest,
    cancel: &AtomicBool,
) -> Result<(e::AnalysisReport, ImageEvidence)> {
    request.validate()?;
    let before = store.artifact_path(before)?;
    let after = store.artifact_path(after)?;
    let stage = tempfile::tempdir_in(&store.paths().cache)?;
    let path = stage.path().join("perceptual.png");
    let w = request
        .before
        .dimensions
        .width
        .max(request.after.dimensions.width);
    let h = request
        .before
        .dimensions
        .height
        .max(request.after.dimensions.height);
    let mut encoder = png::Encoder::new(
        File::options().create_new(true).write(true).open(&path)?,
        w,
        h,
    );
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    let mut writer = encoder.write_header()?;
    let report;
    {
        let mut stream = writer.stream_writer_with_size(64 * 1024)?;
        report = e::analyze_pngs_with_rows(
            File::open(before)?,
            File::open(after)?,
            request,
            cancel,
            &mut |_, eligible, changed| {
                let mut row = Vec::with_capacity(w as usize * 4);
                for (&eligible, &changed) in eligible.iter().zip(changed) {
                    row.extend_from_slice(if changed {
                        &[220, 50, 70, 255]
                    } else if eligible {
                        &[238, 242, 244, 255]
                    } else {
                        &[95, 120, 130, 255]
                    });
                }
                stream.write_all(&row)?;
                Ok(())
            },
        )?;
        stream.finish()?;
    }
    writer.finish()?;
    if cancel.load(Ordering::Acquire) {
        return Err(e::Cancelled.into());
    }
    // Nothing staged is ingested until BOTH complete PNGs pass the engine.
    Ok((
        report,
        crate::images::ingest(store, &path, ArtifactKind::Difference, cancel)?,
    ))
}

/// The single source/viewport transform for images, masks and pointer input.
#[derive(Debug, Clone, Copy)]
pub struct ViewportTransform {
    pub zoom: f32,
    pub pan: (f32, u32),
}
impl ViewportTransform {
    pub fn source_to_viewport(self, source: (f32, f32)) -> (f32, f32) {
        (
            (source.0 - self.pan.0) * self.zoom,
            (source.1 - self.pan.1 as f32) * self.zoom,
        )
    }
    pub fn capture_point(
        self,
        pointer: (f32, f32),
        origin: (f32, f32),
        dims: e::Dimensions,
    ) -> Option<(u32, u32)> {
        if !self.zoom.is_finite() || self.zoom <= 0. || pointer.0 < origin.0 || pointer.1 < origin.1
        {
            return None;
        }
        let x = (pointer.0 - origin.0) / self.zoom + self.pan.0;
        let y = (pointer.1 - origin.1) / self.zoom + self.pan.1 as f32;
        if !x.is_finite()
            || !y.is_finite()
            || x < 0.
            || y < 0.
            || x > dims.width as f32
            || y > dims.height as f32
        {
            return None;
        }
        Some((x.floor() as u32, y.floor() as u32))
    }
}

/// Compatibility helper; pointer input uses the same transform as layout.
pub fn capture_point(
    pointer: (f32, f32),
    origin: (f32, f32),
    zoom: f32,
    pan: (f32, u32),
    dims: e::Dimensions,
) -> Option<(u32, u32)> {
    ViewportTransform { zoom, pan }.capture_point(pointer, origin, dims)
}
pub fn rectangle(a: (u32, u32), b: (u32, u32)) -> Option<e::Rectangle> {
    let r = e::Rectangle {
        x: a.0.min(b.0),
        y: a.1.min(b.1),
        width: a.0.abs_diff(b.0),
        height: a.1.abs_diff(b.1),
    };
    (r.width > 0 && r.height > 0).then_some(r)
}
