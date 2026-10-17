# Tooltip restraint — 2026-10-16

## User direction

Remove tooltips that are not strictly necessary, especially the Zoom explanation
covering its open menu. Preserve the compact toolbar and the existing navy/teal
Operate-mode interface; this is not a layout or workflow redesign.

## Treatment

- The shared toolbar only registers hover/focus names for **enabled icon-only**
  actions. Labelled tools, including View/menu rows, Filter and History choices,
  never repeat their labels in a tooltip. Sidebar names are shortened to Show/Hide.
- Zoom keeps its current mode/scale, chevron and explicit menu choices, without a
  hover explanation whether closed or open. Image controls remains the explicit
  entry point for gestures and keyboard help.
- Page/discovery/result rows and inspection headings no longer open hover boxes.
  Page options / Page details already show complete wrapped addresses. Capture
  details keeps the full page and screen size and now includes full recorded
  Before/After environment names and origins (Capture environment for a single
  capture). This preserves access to information ellipsized in compact headings.
- Underlined external links retain their pointer, hover-ink and focus behavior,
  without explaining familiar link activation in a bubble.
- No target sizes, toolbar rows, evidence height, screenshot pixels, data writes,
  drafts, task boundaries or baseline consent rules change. No new token, font,
  dependency or shipping raster is introduced.

## Verification

Focused regressions cover delayed hover/focus on labelled, icon-only and disabled
tools; quiet Zoom with its menu open; short/long page rows; and quiet links at both
window edges. Existing keyboard selection, focus restoration, explicit image help,
historical source context, queue/draft and approval tests remain in place.

- Production UI build passed: `cargo build --locked -p regressee-app --features
  ui --bin regressee`.
- App library: **133 passed, 2 opt-in tests ignored**.
- Shell: **204 passed, 1 opt-in test ignored**. An earlier run caught a concurrent
  Projects title-layout regression; the complete latest-source rerun passes.
- `git diff --check` and scoped Rust formatting checks passed. Production tooltip
  registration now has one call site, restricted to enabled icon-only tools.

Native offscreen captures use synthetic website originals. Hovered Zoom, open
Zoom/View menus and long source metadata were rendered and inspected at 1440×900:
the menus have no explanatory bubble and the metadata remains quiet. The attempted
three-size matrix stopped at Capture details because the new harness incorrectly
prohibited the permitted icon name, then incorrectly required pointer activation
to move focus to Close. Pointer focus can remain on the Details icon. The harness
now allows its short identifying name; it has not been rerun. No completed
three-size screenshot-matrix or rendered Capture-details acceptance is claimed.
The passing regular tests do cover supported desktop geometry and complete
Capture-details source context. Further visual iteration was stopped rather than
turning this cleanup into an open-ended screenshot loop.

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-tooltip-cleanup-20261016-3_f_0dtj/`.
Fixture provenance is recorded locally. Test/build results and the four inspected
1440×900 scenes are the scoped finish record, not OS-input, screen-reader, Windows/macOS,
usability or public-release qualification. Unrelated concurrent focus-ring and
Projects-layout edits were preserved, not claimed as tooltip work.

## Mouse versus keyboard focus follow-up

The user reported that clicking Show/Hide sidebar left its name visible until
blur. The focus overlay checked actual focus without considering input mode.
The shell now tracks pointer/keyboard input per window. Only keyboard mode paints
the focus hint; pointer mode keeps GPUI's delayed hover tooltip. Actual focus,
focus rings, activation, geometry and operation guards remain unchanged. Keyboard
tracking runs before binding dispatch so consumed Tab/Enter keys restore hints.
The subscription is owned by the window's shell and released with it.

Verification for this follow-up:

- Production UI build passed; app library **138 passed, 2 opt-in tests ignored**.
- All **5 toolbar tests passed**, including mouse-click/leave, delayed re-hover,
  full Tab/Shift-Tab/Enter events, switching back to the pointer, retained focus,
  unchanged stored evidence and no new requests at 1280×800 and 820×600.
- The library tests also confirm input mode does not leak between windows.
- Six native offscreen captures were inspected: mouse-away, keyboard activation
  and return-to-mouse at both sizes. Only the keyboard scene has the identifying
  overlay; the focus ring remains in all three. These use synthetic originals,
  not actual website evidence or OS-input/accessibility qualification.
- The wider shell run encountered `disclosure_tests`'s `search: enabled disclosure`
  assertion, then reached the command timeout. A focused rerun reproduced that
  assertion; this follow-up does not change disclosure eligibility. No complete
  shell-suite pass is claimed.

Follow-up evidence:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-tooltip-input-9dsump3i/captures/`.
Temporary linker aliases are confined to that private evidence root; no host or
repository toolchain configuration changed. No shipping raster or token added.
