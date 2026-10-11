# Unified zoom and loading feedback — 2026-10-11

This refinement supersedes the separate fit/percentage controls described in
`viewer-interactions-verification-20261011.md`. Earlier verification reports
remain historical records. The user requested committing and pushing all current
changes, including the preceding local-review, viewer, History and storage work.

## Changes

- One Zoom menu offers Fit width, Actual size (100%), and percentage presets.
  Desktop labels distinguish automatic fit from manual zoom. At 820×600 the
  same control stacks Zoom and the scale; the selected mode is marked in its
  menu. Fit follows resizing, while manual zoom stays fixed. Existing pointer
  anchoring, scroll, keyboard and paired-image navigation are preserved.
- Projects, saved evidence, individual screenshots and History distinguish
  loading, successfully loaded empty results, and failed reads. Opening a
  project no longer claims missing screenshots or an absent baseline before
  saved evidence arrives. Already loaded evidence remains visible on refresh.
- Queued work immediately names its task. Unknown-duration local reads use
  compact activity feedback rather than a dashed progress track; capture
  feedback and measured progress remain. Storage measurement has its own wait.
- Failed initial evidence reads offer Refresh. Dismissing an error does not
  turn a failed read into an idle loading state or enable baseline capture.

## Verification

- Locked all-features workspace suite: **315 passed, zero failed, 40 deliberately
  ignored**. Tests run serially with this host's existing linker-library path
  and approved private scratch directory.
- Formatting, diff whitespace checks, all-target/all-feature workspace Clippy
  with `-D warnings`, and the ordinary `ui` application build pass. Existing
  vendor GPUI and dependency future-compatibility warnings remain.
- Regressions cover zoom labels/options, keyboard and pointer selection,
  outside/Escape/Tab dismissal, disabled controls, compact label containment,
  fit versus fixed-scale resizing, mounted focus targets, initial evidence and
  screenshot waits, empty/error states, and dismissal followed by retry.
- After personal review of the GPU/font calibration, one initial and one
  confirmation GPUI/Blade offscreen batch captured nine states at 1280×800 and
  820×600: fit, open menu, manual zoom, coordinate editor, project loading,
  screenshot loading, History loading, empty screenshots and failed reads.
  Initial pixels and confirmation highlights were personally inspected.
  The confirmation's 18 PNG dimensions, SHA-256 hashes and renderer metadata
  match their capture records; the harness asserts saved evidence is unchanged.
- The initial batch exposed a wrapped compact Zoom label and missing explicit
  failure recovery. Both were corrected before the single confirmation batch.
  No further visual tuning or capture batch was performed.

Private evidence directories: `regressee-zoom-loading-12s5g0su/` and
`regressee-zoom-loading-confirm-sbvkb4ha/`. Final test, Clippy and build logs are
in the confirmation directory; screenshots and synthetic fixtures are not
included in the source commit.

These are synthetic local evidence and actual offscreen GPUI pixels, not native
OS presentation/input, screen-reader, high-DPI, Windows/macOS, real-site or
public-release qualification. No existing user project was modified and no
website was visited for these checks.
