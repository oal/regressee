# Explicit image controls — 2026-10-14

Replaced the automatic viewer-hover instructions with on-demand help, preserving
the existing Clear wayfinding system and all screenshot/baseline operations.

- Wide desktop: **Image controls** beside Zoom; one muted drag/zoom hint shares
  the caption when its pane has sufficient room. No additional toolbar or hint row.
- Narrow desktop: **Image controls…** at the end of Zoom's keyboard-navigable menu.
  The help action emits a separate request, not a zoom selection.
- Help shares the bounded inspector, with gesture/shortcut rows and an attached
  Close action. Escape restores the available trigger, including after resize;
  an outside click preserves the clicked control's focus. Other disclosures are
  mutually exclusive and opening help does not change target, zoom or drafts.
- The screenshot container has no help tooltip. Full-address/context hints on
  their existing labels remain unchanged. macOS uses Command; other hosts use Ctrl.

## Bounded verification

The two new native-shell tests pass for explicit pointer/keyboard opening, Close,
Escape, outside image clicks, resize, disclosure lifecycle, unavailable pixels,
draft preservation and unchanged persistent evidence. Zoom's seven tests pass,
including a new pointer/keyboard help-event test. App library: **109 passed**,
two opt-in tests ignored. Core library: **52 passed**. Production-feature build
and strict production Clippy pass; `git diff --check` passes.

Strict test-target Clippy encountered an unrelated pre-existing redundant closure
in the concurrently edited `task_ux.rs`; that work was not rewritten here.

The broader native-shell batch reported **165 passed, 2 failed, 30 ignored**,
with the existing bulk-consent qualification filtered out. Failures were History
deletion's retained-mask protection and discovery's `Guided(16)` mount assertion.
The deletion test passed on an isolated recheck after its concurrent fixture
correction; the discovery assertion still fails. Neither failure exercises the
changed help paths, and neither workflow was altered by this change. This report
does not claim a clean whole-app suite.

Personally inspected calibration and one 14-image native offscreen batch at
1280×800 and 820×600: capture/comparison evidence, explicit help, compact Zoom
menus and help tails. Toolbars remain one row, the inline hint fits, every help
row and attached Close are readable, and ordinary evidence stays unobstructed.
All PNG hashes and eight current viewer/help source bindings were checked.

Private synthetic evidence, per-frame source provenance, manifest and review
receipt are retained at:

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-image-controls-axc7u3f2
```

Reproduction: `image_controls_tests`, `ui::zoom_select::tests` and the opt-in
`ui_renderer_capture::ui_image_controls_capture::renderer_capture_image_controls`.
Use the existing private-root/calibration recipe and four synthetic website
fixtures; no user screenshot, session or database was used.

**Bounded verdict: PASS for this interaction and sampled Linux offscreen layout.**
This is not OS-input, assistive-technology, Windows/macOS or release qualification.
The rebuilt `target/debug/regressee` requires restarting an already running app.
