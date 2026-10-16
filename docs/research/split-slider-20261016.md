# Screenshot split slider — 2026-10-16

User-confirmed scope: add **View → Split slider**, keeping **Compare** as the
default and preserving all existing modes. One full-width image stage shows
Before on the left and After on the right, separated by a draggable divider.

## Behavior and implementation

- Before is an opaque clipped reveal over After. Both original tile layers use
  the existing source transform and fixed tile dimensions: moving the divider
  changes neither zoom nor pan, never blends/resizes the originals and does not
  generate another screenshot derivative.
- The divider starts halfway across, has a 40px-wide pointer target and a
  native-drawn paired-arrow handle. Capture-phase movement and release continue
  beyond the image. Both endpoint handles remain grabbable inside the stage.
- Tab reaches the divider after the image. Left/Right adjusts by 1%, Shift by
  10%; Home reveals all After and End all Before. Focus uses existing ink/surface
  roles. Ordinary image dragging, scrolling and pointer-anchored zoom remain.
- The chosen fraction survives mode changes and resizing. Missing pairs disable
  the menu option and fall back to separate panes without discarding the user's
  preferred mode. A genuinely shorter page has an opaque blank tail, not pixels
  leaking through from the other version, and retains image navigation.
- Inspector/modal/task/drawing guards prevent divider interaction. No capture,
  save, note, mask, review decision or baseline operation is introduced.

## Verification

- Native-shell suite: **198 passed, 0 failed, 1 ignored, 1 filtered out**, using
  `--skip bulk_consent --skip qualification`. This does not run the separate
  bulk-consent qualification. Includes two new split tests, the six-mode source
  pan/zoom regression and the expanded View-menu geometry/keyboard regression.
- App library (`ui-tests` feature): **124 passed, 2 ignored**; the `ui`-only run
  also passed (**82 passed, 2 ignored**). Core: **54 passed**.
- Strict `ui-tests` test-target Clippy and the production `ui` build pass.
- Actual GPUI Linux offscreen calibration passed, with 1,024 exact pixels for
  each calibration color. Personally inspected all 12 screenshot captures at
  **1280×800 and 820×600**: mode menus, centered reveal, left/right dragging,
  keyboard focus and 100% zoom with Results hidden. Menu/toolbar/evidence bounds
  remain contained; the split retains evidence height.
- Verified every PNG SHA-256 and eight current-source hashes per frame.

Private synthetic evidence, per-frame metadata and `split-manifest.json`:

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-split-0erqx73d
```

Reproduction: `split_view_tests`, `phase2_viewer_tests`,
`phase2_evidence_tests::view_menu_is_trigger_attached_full_row_and_native_keyboard_dismissable`,
and the opt-in
`ui_renderer_capture::ui_split_view_capture::renderer_capture_split_view`.
The capture uses the established private-root/calibration contract and local
synthetic Northline website fixtures. No user project, screenshot or session was
used. This host needed private linker-name symlinks to its installed libraries;
no system dependency or global toolchain configuration was changed.

## Documentation check

- Checked the split instructions in `docs/first-run-guide.md` against
  `split_view.rs`, the clipped original layers in `main.rs`, the View menu in
  `evidence_workspace.rs` and the viewer/focus wiring in `viewer_ux.rs`,
  `workflow.rs` and `crates/core/src/shell.rs`. The instructions match; no large
  user-guide section is duplicated here. `DESIGN.md` now records the optional
  mode, keyboard operation and missing-pair fallback alongside existing guidance.
- Compared `ui/theme.rs`, shared `ui/controls.rs` and the divider paint with
  `PRODUCT.md`, `DESIGN.md` and `.impeccable/design.json`: the incumbent
  navy/teal/mint palette, native control treatment and system typography remain
  unchanged. The divider reuses existing ink/surface/hover roles; no new tokens,
  dependencies, assets or shipping rasters were introduced. Frontmatter and the
  sidecar are unchanged; unrelated pre-existing documentation drift is preserved.
- Independent bounded finish review found only the stale viewer guidance, with
  no material functional or visual defect in the code or 12 captures. This
  documentation-only correction was scored **resolved** in the reviewer
  continuation, with disposition **ship** for that scored fix. Source hashes
  remained unchanged; tests and captures were not rerun for documentation edits.
  This is not whole-app, cross-platform or native-accessibility qualification.

This evidence does **not** qualify OS input, assistive technology, Windows/macOS
behavior or public release readiness. No new shipping raster, semantic token,
dependency or replacement design system was added. Restart an already running
app to use the rebuilt `target/debug/regressee`.
