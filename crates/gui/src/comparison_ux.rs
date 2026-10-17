// SPDX-License-Identifier: MPL-2.0
//! One evidence-derived explanation for the workspace, Details and Analysis.
//! Capture cautions stay separate from genuine pixel-comparison failures.
use super::*;
use regressee_app::analysis_engine::{GateReason, Warning};
use regressee_core::evidence::CaptureEvidence;

pub(super) struct ComparisonIssue {
    pub heading: &'static str,
    pub reason: String,
    pub recovery: &'static str,
}

pub(super) struct CaptureNotice {
    pub summary: &'static str,
    pub detail: String,
}

fn capture_reasons(capture: Option<&CaptureEvidence>) -> Vec<String> {
    let Some(capture) = capture else {
        return Vec::new();
    };
    let mut reasons = Vec::new();
    for warning in &capture.metadata.warnings {
        let text = capture_warning_label(warning).to_owned();
        if !reasons.contains(&text) {
            reasons.push(text);
        }
    }
    if !capture.metadata.stable && !capture.metadata.warnings.iter().any(|w| w == "Unstable") {
        reasons.push("The page was still changing when captured".into());
    }
    reasons
}

impl Shell {
    fn comparison_captures(&self) -> (Option<&CaptureEvidence>, Option<&CaptureEvidence>) {
        let Some(comparison) = &self.evidence.comparison else {
            return (None, None);
        };
        let Some(key) = &self.selected_key else {
            return (None, None);
        };
        let reference = self
            .evidence
            .baseline
            .iter()
            .chain(self.evidence.baselines.iter())
            .find(|b| b.id == comparison.revision.baseline)
            .and_then(|b| b.references.iter().find(|r| &r.key == key));
        let before = reference.and_then(|reference| {
            self.evidence
                .before
                .as_ref()
                .filter(|c| &c.metadata.key == key && c.image.original == reference.artifact)
                .or_else(|| {
                    self.history
                        .as_ref()
                        .and_then(|h| h.runs.iter().find(|r| r.run.id == reference.run))
                        .and_then(|r| r.evidence(key))
                        .filter(|c| c.image.original == reference.artifact)
                })
        });
        (before, self.displayed_run().and_then(|r| r.evidence(key)))
    }

    /// Cautions are useful context, not a veto on inspecting captured pixels.
    pub(super) fn capture_notice(&self) -> Option<CaptureNotice> {
        let (before, after) = self.comparison_captures();
        let loading_only = [before, after].into_iter().flatten().all(|c| {
            c.metadata.stable
                && c.metadata
                    .warnings
                    .iter()
                    .all(|w| regressee_core::evidence::loading_warning(w))
        });
        let before = capture_reasons(before);
        let after = capture_reasons(after);
        if before.is_empty() && after.is_empty() {
            return None;
        }
        let summary = if !loading_only {
            "These captures have warnings."
        } else if before.is_empty() {
            "Some content didn’t load in After."
        } else if after.is_empty() {
            "Some content didn’t load in Before."
        } else {
            "Some content didn’t load in both captures."
        };
        let detail = if before == after {
            format!("Both screenshots: {}.", before.join("; "))
        } else {
            let mut details = Vec::new();
            if !before.is_empty() {
                details.push(format!("Before: {}.", before.join("; ")));
            }
            if !after.is_empty() {
                details.push(format!("After: {}.", after.join("; ")));
            }
            details.join(" ")
        };
        Some(CaptureNotice { summary, detail })
    }

    pub(super) fn comparison_issue(&self) -> Option<ComparisonIssue> {
        let comparison = self.evidence.comparison.as_ref()?;
        let key = self.selected_key.as_ref()?;
        if self.viewer_read_failed {
            return Some(ComparisonIssue {
                heading: "Screenshot could not be loaded",
                reason: "A saved screenshot is missing or unreadable.".into(),
                recovery: "Restore the screenshot file, then refresh. Your baseline is unchanged.",
            });
        }
        let result = comparison.revision.results.iter().find(|r| &r.key == key);
        if result.is_some_and(|r| {
            r.eligible_pixels > 0
                && matches!(
                    comparison.review_classification(key),
                    Some(Classification::Changed | Classification::NoSignificantChange)
                )
        }) {
            return None;
        }
        let run = self.displayed_run();
        if let Some(TargetOutcome::Failed { code }) = run
            .and_then(|r| r.run.targets.iter().find(|t| &t.key == key))
            .map(|t| &t.outcome)
        {
            return Some(ComparisonIssue {
                heading: "Screenshot not captured",
                reason: format!("After: {}.", code.label()),
                recovery: regressee_gui::task::capture_recovery(*code),
            });
        }

        let perceptual = comparison.perceptual.as_ref();
        let report =
            perceptual.and_then(|p| p.reports.iter().find(|(k, _)| k == key).map(|(_, r)| r));
        let mut reasons = Vec::new();
        let mut heading = "Comparison unavailable";
        let mut recovery = "Reanalyze originals in Analysis to create a new result from the saved screenshots. This does not visit the website.";
        if let Some((_, failure)) =
            perceptual.and_then(|p| p.failures.iter().find(|(k, _)| k == key))
        {
            heading = "Screenshot analysis failed";
            reasons.push(failure.clone());
        }
        if let Some(report) = report {
            for gate in &report.gates {
                let (title, reason, next) = match gate {
                    GateReason::UnexpectedWidth => (
                        "Screenshot widths differ",
                        format!("Before is {} pixels wide; After is {} pixels wide.", report.dimensions.before.width, report.dimensions.after.width),
                        "Use matching screen-size settings for both captures. Replacing the saved Before screenshot requires capturing and saving a new baseline.",
                    ),
                    GateReason::DeviceScaleMismatch => (
                        "Screenshot scales differ",
                        format!("Before uses a pixel scale of {}; After uses {}.", report.request.before.device_scale_factor, report.request.after.device_scale_factor),
                        "Use the same pixel scale for both captures. Replacing the saved Before screenshot requires capturing and saving a new baseline.",
                    ),
                    GateReason::MaterialProfileMismatch => (
                        "Capture settings do not match",
                        "Before and After use different capture settings or image-replacement modes.".into(),
                        "Use matching capture settings for both versions. A change to the baseline's capture settings requires capturing and saving a new baseline.",
                    ),
                    GateReason::AllMasked => (
                        "All screenshot content is ignored",
                        "The saved ignored areas leave no pixels to compare.".into(),
                        "In Analysis, reduce or remove ignored areas, then save them to recalculate the comparison.",
                    ),
                    GateReason::NonOpaquePixels => (
                        "Screenshot transparency is unsupported",
                        "At least one saved screenshot contains transparent pixels, which this comparison method does not support.".into(),
                        "Capture opaque screenshots before comparing again. Replacing the saved Before screenshot requires explicitly saving a new baseline.",
                    ),
                };
                if reasons.is_empty() {
                    heading = title;
                    recovery = next;
                }
                reasons.push(reason);
            }
            if report
                .warnings
                .contains(&Warning::RenderingEnvironmentMismatch)
            {
                if reasons.is_empty() {
                    heading = "Capture environments do not match";
                    recovery = "Use matching browser, computer and font settings for both versions. A new check does not change the saved Before screenshot.";
                }
                reasons.push("Before and After were captured in different rendering environments (computer, fonts or browser version).".into());
            }
        }

        if reasons.is_empty() {
            reasons.push(
                if result.is_none() {
                    "No comparison result was saved for this screenshot."
                } else {
                    "No reason was recorded for this result."
                }
                .into(),
            );
        }
        Some(ComparisonIssue {
            heading,
            reason: reasons.join(" "),
            recovery,
        })
    }
}
