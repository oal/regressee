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
