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
    /// Give a read one visible status owner. Global feedback remains for saves
    /// and other operations, and for reads that keep already-loaded content.
    pub(super) fn activity_is_inline(&self) -> bool {
        let Some((label, _, _)) = &self.progress else {
            return false;
        };
        if self.error.is_some() {
            return false;
        }
        if self.focus.shows_project_picker() {
            return self.projects.items.is_empty()
                && self.projects_dirty
                && self.pending.is_some()
                && label == "Loading projects…";
        }
        match self.focus.screen {
            Screen::Settings => {
                (self.runtime_readiness.is_none() && label == Work::RuntimeStatus.activity_label())
                    || (self.summary.is_none() && label == Work::InspectStorage.activity_label())
            }
            Screen::History => {
                self.history_section != 2
                    && self.history.is_none()
                    && self.pending.is_some()
                    && label == "Loading capture and baseline history…"
            }
            Screen::Results => {
                self.initial_evidence_loading()
                    && matches!(
                        label.as_str(),
                        "Loading saved screenshots and results…"
                            | "Loading login and environment settings…"
                    )
            }
            _ => false,
        }
    }

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
                        "{}. {}",
                        code.label(),
                        regressee_app::task::capture_recovery(*code)
                    ),
                    regressee_core::domain::TargetOutcome::Skipped { reason } => {
                        format!("Capture skipped: {reason}")
                    }
                    regressee_core::domain::TargetOutcome::Cancelled => {
                        "This screenshot was not taken before capture stopped.".into()
                    }
                    _ => self.error.clone().unwrap_or_else(|| {
                        "The recorded screenshot is missing or could not be read.".into()
                    }),
                };
                return div().debug_selector(|| "unavailable-target-recovery".into()).flex().flex_col().gap_2()
                    .child(div().text_size(px(21.)).child(if self.selected_capture_failed() {
                        "This page could not be captured"
                    } else {
                        "Screenshot unavailable"
                    }))
                    .child(reason)
                    .child("Find pages creates a new page list for another capture. Refresh only rereads saved results; it does not take screenshots. Your saved baseline is unchanged.")
                    .into_any_element();
            }
        }
        match self.screenshot_state() {
            ScreenshotState::LoadingImage => controls::loading("Loading screenshot…", "Reading the selected screenshot from disk. The page list stays available."),
            ScreenshotState::LoadingEvidence => controls::loading("Loading saved screenshots…", "Reading this project’s page list, captures and comparison results from disk."),
            ScreenshotState::Capturing => controls::loading("Preparing screenshots…", "Completed captures will appear here when they are ready to inspect."),
            ScreenshotState::Unavailable => controls::empty("Screenshots could not load", "Use Refresh to try reading saved screenshots again. Saved screenshots and the baseline have not been changed."),
            ScreenshotState::Empty => controls::empty("No screenshots yet", "Find pages, then capture them to inspect this website. No baseline is saved automatically."),
            ScreenshotState::MissingTarget => controls::empty("No screenshot for this page and screen size", "Select a page and screen size to view its screenshot. A failed or missing capture does not mean the page is unchanged."),
        }.into_any_element()
    }
}
