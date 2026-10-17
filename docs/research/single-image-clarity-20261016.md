# Single-image switching clarity — 2026-10-16

The user could not discover how to switch from Before to After in the full-width
view. This is a narrow Operate-mode refinement of the existing native toolbar.

- View now names the mode **Single image (full width)** rather than Before/After,
  avoiding the suggestion that both originals appear together.
- The toolbar names the mode **Single image** (**Single** below 720px actual
  working width) beside always-visible, labelled **Before** and **After** choices.
  Mint selection and a teal boundary identify the displayed original. No swap
  icon, tooltip tutorial, new toolbar row, smaller targets or evidence tinting.
- Each choice selects its named side, including when it is already selected.
  Before precedes After in keyboard order. Both controls leave traversal outside
  single-image comparison inspection, using the existing mounted/enabled guards.
- Switching sides preserves automatic/manual scale, source pan, inspected target,
  unsaved notes and persisted evidence; it never captures or saves anything.

Coverage lives in `toolbar_tests.rs`, `phase2_evidence_tests.rs` and the opt-in
`ui_toolbar_capture::renderer_capture_single_image_sides` recipe. The renderer
uses offline synthetic website fixtures reused from the previous icon-spacing
inspection, not customer screenshots or new shipping artwork.

## Bounded verification

- Production UI build passed with the pinned Rust toolchain.
- Both toolbar tests passed: all six modes at 1440×900, 1280×800 and 820×600,
  navigation expanded/hidden, pointer selection/reselection, Tab/Shift-Tab,
  Enter/Space and retained transforms/drafts/evidence.
- All 13 core shell/focus tests passed.
- The explicit native renderer recipe passed: 12 GPUI/Blade frames covering
  the View menu, Before, After and focused Before at all three desktop sizes.
  Inspected the full contact sheet and the relevant full-size frames: labels
  remain readable, both choices fit, selection/focus are distinct, and the
  toolbar stays on one 40px row. No material scoped visual defect found.
- Every captured PNG hash, opaque/nonblank status and six relevant current-source
  hashes were verified. Formatting and `git diff --check` passed.
- The additional full shell-suite run exceeded its 120-second command limit
  during the History hierarchy tests. All completed tests passed, including the
  existing full-width/drawing/height/menu/source-pan regressions. This is not a
  full-suite pass; the independently completed focused checks above remain valid.

This is an in-thread scoped development review, not independent usability or
accessibility approval. No additional visual refinement round was needed.

Private development evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-single-image-40nsv5ia/`.

The host's missing development linker names are supplied by diagnostic-only
aliases inside that root; existing repository aliases and toolchain configuration
are unchanged. Linux GPUI offscreen pixels and simulated keyboard/pointer tests
do not qualify OS input, screen readers, other platforms or release usability.
