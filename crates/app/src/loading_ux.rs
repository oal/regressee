// SPDX-License-Identifier: MPL-2.0
//! Distinguish not-yet-read evidence from an authoritative empty result.
use super::*;

#[derive(Debug, PartialEq, Eq)]
pub(super) enum ScreenshotState {
    LoadingImage,
    LoadingEvidence,
    Capturing,
    Unavailable,
    Empty,
    MissingTarget,
}

impl Shell {
    pub(super) fn selected_capture_failed(&self) -> bool {
        self.displayed_run()
            .and_then(|r| {
                r.run
                    .targets
                    .iter()
                    .find(|t| Some(&t.key) == self.selected_key.as_ref())
            })
            .is_some_and(|t| {
                matches!(
                    t.outcome,
                    regressee_core::domain::TargetOutcome::Failed { .. }
                        | regressee_core::domain::TargetOutcome::Skipped { .. }
                        | regressee_core::domain::TargetOutcome::Cancelled
                )
            })
    }
    pub(super) fn initial_evidence_unavailable(&self) -> bool {
        self.project.is_some()
            && matches!(self.focus.screen, Screen::Results | Screen::History)
            && self.evidence_dirty
            && self.evidence.latest.is_none()
            && self.preview.is_none()
            && !self.mask_dirty
            && (self.error.is_some() || self.evidence_read_failed || self.pending.is_none())
    }

    pub(super) fn initial_evidence_loading(&self) -> bool {
        self.project.is_some()
            && matches!(self.focus.screen, Screen::Results | Screen::History)
            && self.evidence_dirty
            && self.pending.is_some()
            && self.evidence.latest.is_none()
            && self.preview.is_none()
            && !self.focus.capturing
            && !self.mask_dirty
            && !self.evidence_read_failed
            && self.error.is_none()
    }

    pub(super) fn screenshot_state(&self) -> ScreenshotState {
        if self.viewer_pending.is_some() {
            ScreenshotState::LoadingImage
        } else if self.initial_evidence_loading() {
            ScreenshotState::LoadingEvidence
        } else if self.focus.capturing {
            ScreenshotState::Capturing
        } else if self.initial_evidence_unavailable()
            || self.viewer_read_failed
            || self.evidence_read_failed
            || (self.error.is_some() && (self.evidence_dirty || self.focus.viewer))
        {
            ScreenshotState::Unavailable
        } else if self.evidence.latest.is_none() {
            ScreenshotState::Empty
        } else {
            ScreenshotState::MissingTarget
        }
    }

    pub(super) fn screenshot_placeholder(&self) -> AnyElement {
        if self.selected_capture_failed()
            || matches!(
                self.screenshot_state(),
                ScreenshotState::MissingTarget | ScreenshotState::Unavailable
            )
        {
            let target = self.displayed_run().and_then(|r| {
                r.run
                    .targets
                    .iter()
                    .find(|t| Some(&t.key) == self.selected_key.as_ref())
            });
            if let Some(target) = target {
                let reason = match &target.outcome {
                    regressee_core::domain::TargetOutcome::Failed { code } => format!(
                        "Capture failed: {code:?}. {}",
                        regressee_app::task::capture_recovery(*code)
                    ),
                    regressee_core::domain::TargetOutcome::Skipped { reason } => {
                        format!("Capture skipped: {reason}")
                    }
                    regressee_core::domain::TargetOutcome::Cancelled => {
                        "This target was not captured before the task stopped.".into()
                    }
                    _ => self.error.clone().unwrap_or_else(|| {
                        "The recorded screenshot is missing or could not be read.".into()
                    }),
                };
                return div().debug_selector(|| "unavailable-target-recovery".into()).flex().flex_col().gap_2()
                    .child(div().text_size(px(21.)).child("Screenshot unavailable — not an unchanged result"))
                    .child(reason)
                    .child("Refresh reads saved evidence again; it does not recapture this target. For a new check, return to Review changes. To replace the baseline page list, use Pages & baseline, find pages and explicitly capture/save a replacement. Earlier versions remain in History.")
                    .into_any_element();
            }
        }
        match self.screenshot_state() {
            ScreenshotState::LoadingImage => controls::loading("Loading screenshot…", "Reading the selected screenshot from disk. The page queue stays available."),
            ScreenshotState::LoadingEvidence => controls::loading("Loading saved screenshots…", "Reading this project’s page list, captures and comparison results from disk."),
            ScreenshotState::Capturing => controls::loading("Preparing screenshots…", "Completed captures will appear here when they are ready to inspect."),
            ScreenshotState::Unavailable => controls::empty("Screenshots could not load", "Use Refresh to try reading saved screenshots again. Saved screenshots and the baseline have not been changed."),
            ScreenshotState::Empty => controls::empty("No screenshots yet", "Find pages, then capture them to inspect this website. No baseline is saved automatically."),
            ScreenshotState::MissingTarget => controls::empty("No screenshot for this target", "Select a captured target to inspect its screenshot. A failed or missing capture is not an unchanged result."),
        }.into_any_element()
    }
}
