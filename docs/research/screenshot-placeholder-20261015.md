# Missing screenshot placeholder — 2026-10-15

## Scope

The supplied Inspect capture screenshot showed an unframed failure paragraph
where screenshot evidence would normally sit. This narrow Operate-mode refinement
preserves the navy/teal identity, recorded reasons, recovery wording and actions.

- Failed, skipped and cancelled captures, unreadable saved screenshots and missing
  image selections use a white, thin-bordered screenshot stage, not a page-level
  empty-state paragraph.
- A native 48px missing-image icon, compact 18px heading and readable 14px reason
  explain the absence. Failure marks retain error ink. Shared muted help and a
  thin separator distinguish recovery guidance from the recorded cause.
- A centered, maximum-440px message with 24px insets fills the available image
  frame. Long messages use safe auto margins and scroll inside that fixed frame,
  with an overflow-only thumb indicating more content. Wrapped text is measured
  at its final finite width so the entire recovery message remains reachable.
- Details and path/profile identity stay outside the stage. Long context is
  truncated with full-text hover disclosure instead of expanding the evidence
  layout. The inspector retains its own existing scroll owner.
- Loading and first-run empty states, screenshot pixels, capture operations and
  baseline approval guards are unchanged. No new shipping raster or design token.

## Verification

**Scoped verdict: pass.** An in-thread finish review of the confirmation captures
found a clear missing-image treatment, retained screenshot geometry, legible
reasons and reachable recovery guidance. This is not independent review or whole-
application acceptance. No further visual polishing pass was performed.

- Native app build passed: `cargo build --locked -p regressee-app --features ui
  --bin regressee`.
- Shell UI suite passed: **188 tests**, with one unrelated native-fixture
  preparation test intentionally ignored. The new test exercises eight states at
  1280×800 and 820×600, including image limits, timeouts, skipped/cancelled
  captures, unreadable files, long reasons, hidden results and open filters.
- Shared UI suite passed: **55 tests**, including the new fixed-frame scrolling
  regression, native icon geometry and existing contrast checks.
- The explicit renderer test passed and produced **24 real GPUI/Blade offscreen
  captures**, with manifests and source-hash sidecars. These include the initial
  states and scrolled recovery messages wherever content overflows. Filters are
  captured open, then closed through their existing control before testing the
  underlying message's scroll reachability.
- Tests verify the frame fills its owner, all text fits its measured content
  extent, short messages center, long messages reach their help text by wheel
  scrolling, and no zoom/approval control becomes available for missing pixels.
  Stored project evidence is unchanged.
- The initial inspection revealed GPUI percentage/max-width measurement could
  underreport a long message's height. Explicit final-width measurement, stable
  child sizing and a dedicated screenshot scroll owner fix the underlying issue;
  the confirmation checks pass at both sizes.
- Concurrent baseline-workflow edits briefly blocked compilation; they were not
  repaired or reverted by this task. Final builds and tests above passed after
  those edits settled. Other unrelated changes remain untouched.

Commands used `LIBRARY_PATH=$PWD/.runtime/link-libs`. Renderer capture used
`--features ui-renderer-capture --bin regressee
renderer_capture_screenshot_placeholders -- --ignored`, with explicit approved
and output roots.

Evidence is synthetic offline Linux GPUI layout/rendering, not actual website
results, OS-input, accessibility, cross-platform or release qualification.

Private capture root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-screenshot-placeholder-20261015-789QELhw/`.
The root contains the initial inspection; `confirmation/` contains the 24 final
frames, JSON sidecars and `placeholder-manifest.json`. No shipping raster was added.
