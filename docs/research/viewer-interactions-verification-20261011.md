# Direct viewer interactions — 2026-10-11

This user-requested refinement supersedes the queue-replacing Tools behavior
recorded in `ui-carry-through-verification-20261011.md`, not its historical evidence.

## Changes

- Removed six dedicated zoom/pan buttons. Drag and ordinary two-axis scroll move
  the paired images together; Ctrl/Cmd+scroll zooms around the pointer, and
  Shift+scroll moves sideways. Focused images support +/−, arrows, Page Up/Down,
  0 for fit width and 1 for actual size. Fit width remains visible; the desktop
  percentage returns to actual size. Missing images have no navigation controls.
- Replaced generic Tools with comparison-only Analysis. Its bounded flyout and
  Details overlay the viewer rather than replacing the searchable queue or
  reducing image height. Queue geometry, image transform, ordinary review
  actions and note drafts are retained. Close/Escape restore the trigger.
- Details presents the page, screen size, screenshot dimensions, capture status,
  readable UTC run time and actionable failure advice. Exact IDs, debug metadata
  and source descriptors require an explicit Technical diagnostics disclosure.
- Ignored-area drawing temporarily folds away Analysis. Completing a rectangle
  returns to the inspector; coordinate edits and saves remain explicit. Numeric
  focus reveals the corresponding field inside the bounded inspector.
- Updated `DESIGN.md` and `docs/first-run-guide.md` with the interaction model.

## Verification

Private external evidence: `regressee-viewer-ux-WRBI4w/`.

- Locked all-features workspace suite: **306 passed, zero failed, 39 ignored**.
  Regressions cover pointer anchoring, ordinary scroll, keyboard navigation,
  dialog suppression, missing-image mounting, readable summaries, historical
  baseline-source identity, stable disclosure geometry and draft preservation.
- Formatting, all-targets/all-features Clippy with `-D warnings`, and ordinary
  UI application build pass. Existing vendor/dependency notices remain.
- One initial and one confirmation native GPUI/Blade offscreen batch: eight
  states at both 1280×800 and 820×600 (16 images per batch). States include
  ordinary review, Analysis, numeric area editing, Details, capture-only
  Details, failed capture, failed-capture Details and keyboard help.
- Confirmation PNG dimensions, hashes and opaque alpha were checked against
  capture metadata. Actual pixels were inspected; synthetic authoritative
  evidence is asserted unchanged by the capture harness.
- After pixel confirmation, a summary-only factual correction recognizes a
  historical screenshot referenced by a retained baseline even when no current
  comparison baseline pointer exists. The regression verifies exact run/key/
  artifact matching; screenshots precede that status-text correction. No further
  visual tuning or capture batch was performed.

At narrow widths flyouts cover most of the viewer while open, and lower fields
require scrolling. Closing restores unobstructed images without refitting them.
This is not native OS input, pinch-gesture, screen-reader, high-DPI,
Windows/macOS, real-site or public-release qualification. No user projects were
modified, no website was visited, and no commit or push was made.
