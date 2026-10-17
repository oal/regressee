# App-wide scroll affordances — 2026-10-16

## Scope

User requests discoverable scrollbars throughout the app, at least on hover or
scroll. The follow-up explicitly excludes actual screenshot panes: keep their
drag-to-pan interaction, without scrollbars. This refines the existing native
navy/teal Operate interface; no workflow, backend operation or visual-world change.

## Implementation

- `ui/scrollbar.rs` decorates existing scroll owners, including virtual lists,
  without changing their layout or focus order. It reads actual geometry after
  prepaint, so first paint, disclosure changes, shrinking content and resizing
  determine whether a bar exists. No overflow means no bar or scrollbar hitbox.
- Rounded muted thumbs remain visible while content overflows. Hover/drag uses
  stronger ink and reveals a quiet track. The 16px hit area surrounds an 8px thumb
  with a surface keyline; the normal 32px minimum reduces for very short clips.
  Track clicks page by 90%; dragging clamps to the ends and releases outside;
  wheel events use the same owner. Scrolling never takes focus or activates a row.
- Results, discovered/fixed page queues, both History lists, outer workspaces,
  timezone/Zoom menus, options, review inspectors, task bodies, feedback, approval
  and information/confirmation dialogs share the treatment. Existing reserved
  settings/dialog gutters stay reserved; other bars overlay without reflow.
  Small untracked reading surfaces retain their handles in element state.
- Missing/loading screenshot **text explanations** retain a reading scrollbar.
  Actual screenshot panes have none; their existing drag/wheel/keyboard/zoom
  navigation and untinted source pixels are unchanged.
- The multiline note uses the same interactive scrollbar, without moving its
  caret/selection or changing/saving the draft. An already-visible caret remains
  visible after reflow; manual reading scroll does not snap back to that caret.
- No new dependency, palette, motion or shipping raster. Existing unrelated
  changes in the worktree were preserved.

## Verification

Private evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-scrollbars-asy1xw2q/`.

- Library suite: **138 passed, 0 failed, 2 intentionally ignored**. This includes
  shared scrollbar geometry, ordinary/virtual list track paging, dragging beyond
  the panel, outside release, wheel input, focus/row preservation, retained offsets
  across repaint and shrinking-to-fit content. Note tests cover caret/selection,
  byte-exact drafts, wrapping/reflow and its new scrollbar interaction. Existing
  menu, dialog, loading/placeholder and control tests also pass.
- Project-settings scoped suite: **4 passed**, including existing two-size track
  clicking, outside dragging/release, wheel, section tracking and Tab/reveal tests.
- Additional scoped shell regressions: **19 passed**, covering App settings,
  options/filter disclosures, baseline consent, review footer, placeholders,
  unchanged viewer gestures and History selection/last-row access. Exact filters
  and results are retained in `regressions.json` and `regression-*.log`.
- Production UI build and `git diff --check` passed. Only the existing dependency
  future-compatibility warning for `proc-macro-error2` remained.
- The opt-in native renderer test passed with **21 complete GPUI/Blade frames**:
  review, queue hover, queue scrolled, Analysis, multiline note, nested History and
  expanded App settings at 1440×900, 1280×800 and 820×600. It checks queue scrolling
  without changing the selected screenshot/pan, and unchanged persisted evidence.
  Long History repeats synthetic records in the test view only, not storage.
- All complete-frame PNG hashes, opacity/nonblank status and nine relevant source
  receipts were checked. Eight source hashes match current files; `main.rs` changed
  concurrently after compilation, so these captures qualify their embedded source
  identity, not every later shell edit. See `source-receipt.json`.
- The preliminary capture run stopped because its fit-to-width website fixture
  had no image overflow, correctly yielding no bar. Those two preliminary frames
  remain in `inspection/`; the complete user-corrected, image-bar-free set is in
  `confirmation/`. No incomplete run is presented as a full pass.

## Bounded visual finish review

Inspected all three contact sheets and full-size narrow note/History frames.
The queue thumb changes with actual scroll position, nested History exposes its
own extent separately from the outer workspace, settings reveal further content,
and the note keeps Save pinned. Hover strengthens the track without reflow;
actual image panes remain clear. No material scoped visual defect found; no
additional visual polishing round needed. **Ship for this bounded refinement.**

This is an in-session development review, not independent usability acceptance.
Fixtures are offline synthetic website images reused byte-for-byte from the
previous single-image inspection; `fixture-provenance.json` records their origin
and hashes. Linux GPUI offscreen rendering and simulated input do not qualify
actual OS input, screen readers, Windows/macOS or public-release usability.
