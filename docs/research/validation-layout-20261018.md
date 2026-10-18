# Stable validation feedback — 2026-10-18

## Outcome

The user's screenshot and blur report exposed two independent layout changes:
the form-wide summary mounted above Website, while the input mounted a second
correction above its existing helper. This is a narrow Impeccable Operate-mode
refinement, preserving the current native neutral/teal light and dark identity.

- Website and settings fields use one shared help/correction region. Invisible
  intrinsic grid copies reserve the helper and known validation message's wrapped
  height at the actual field width. Feedback does not animate height, overlay
  another control, truncate, or use a universal fixed line count.
- Invalid fields gain the existing semantic error border; keyboard focus retains
  its teal border/ring. Corrections replace the routine helper rather than adding
  another paragraph. Starting-address website/subdomain scope remains separate.
- The repeated top summary is removed. **Fix 1 field / Fix N fields** and
  **Not saved** replace setup footer supporting status inside its allocated space.
  App settings reserves the compact summary's space while Network is expanded;
  errors remain reachable through that summary after the disclosure collapses.
- Correction navigation follows the footer/disclosure's painted keyboard order
  and still reveals the affected stage/field without submission. Deliberate
  correction navigation may scroll; ordinary blur and correction do not.
- No validator acceptance rule, editable value, IME handling, storage operation,
  capture, baseline, approval, palette, dependency or shipping raster changes.

## Bounded verification

Independent read-only source assessment identified summary placement, duplicate
copy, long-message wrapping and focus-order risks before implementation.

Final focused checks: **39 passed** (25 GUI/input checks and 14 core focus checks).
These include the two new simulated-native-input regressions:

- Both themes at 1280×800 and 820×600: edit an invalid starting address, actually
  blur the active window's input, reject Save, use Fix field, and correct it.
  Fields, Save/footer geometry and scroll offsets remain unchanged on blur;
  correction retains the position chosen by explicit recovery. Rejected saves
  and correction navigation dispatch no worker operation.
- Both sizes: optional proxy/path validation retains the field and following
  groups' geometry. Invalid network Save dispatches no operation; correction
  recovery reopens the collapsed Network disclosure and targets the proxy field.

Related setup/keyboard/settings checks initially returned **12 passed, 1 failed**.
The remaining broader failure is
`project_settings_tests::project_settings_jumps_preserve_drafts_and_keep_wayfinding_pinned`:
it expects the Capture jump to focus Tablet, while the current DOM-snapshot
implementation focuses **What to check → Screenshots only**. This task did not
change that jump implementation or its expectation. No whole-suite pass is claimed.

Native offscreen inspection produced 16 captures covering editing, blur,
save-rejected and corrected states at both sizes/themes. The one correction
shortened the address message so the entire correction fits above the short
window's footer. The sole confirmation produced another 16 captures; the contact
sheet and full-size short-dark/wide-light corrections were inspected. No further
visual round was taken.

Package all-target Clippy with warnings denied, the ordinary `ui` desktop build,
and the scoped layout detector passed. The shell's unconfigured Cargo shim was
bypassed using the project's pinned Rust 1.98.1 through mise. Existing private
Fedora linker aliases were reused; system libraries/project aliases were not
modified. Renderer diagnostics also enable the existing `ui-inspection` feature
because their current output-root helper uses its optional libc dependency.

**Scoped in-thread verdict: pass for the validation layout refinement.** This is
Linux GPUI simulated input/layout and actual offscreen rendering, not OS-input,
screen-reader, cross-platform, whole-app or release acceptance.

## Evidence

Private root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-validation-20261018-UIaczFsK/`.
Original and confirmation PNGs/renderer source/hash sidecars live under
`inspection/` and `confirmation/`; the latter includes a resized contact sheet.
These are diagnostic native app renders of offline draft values, not website
captures or shipping assets. The entered hostname is never visited.

GPUI retains debug-selector entries from previous frames, so disappearance is
verified through current validation state, disabled action behavior and
fresh pixels, not a stale debug selector's absence. Draft, clipboard, Unicode,
composition and search presentation regressions remain covered by the focused
native input checks.
