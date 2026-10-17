# Keyboard focus reading order — 2026-10-16

The user reported that Tab jumped between Filter, Compare, Hide results,
Analysis, Image controls and Details rather than following the interface.
This is a narrow native Operate-mode accessibility repair, not a redesign.

## Cause and correction

`FocusState::controls` sorted result controls into broad type buckets. Stable
sorting preserved historical insertion order inside each bucket, which no
longer matched the full-width toolbar, queue or inspector. `Screen::ALL` also
describes shortcut numbering, not the actual rail's visual order.

- Replace bucket sorting with an explicit reading sequence: rail, project
  header, task actions, toolbar, queue, evidence/inspector, review, baseline.
- Toolbar: Results → View → optional Before/After → Zoom → Image controls
  (wide windows) → Details → Analysis. Compact Image controls remains in Zoom.
- Queue: Search → optional Clear → Filter → Previous/Next. Filter-panel Close
  precedes its choices and grouping control; discovery options retain selected
  page, search-match actions, details and the discard footer in that order.
- Inspector: Close → content → pinned Save note / Copy diagnostics. Draw remains
  inside Ignored areas; Done moves to its actual drawing-toolbar position.
- Page previews and unavailable captures retain their evidence-column placement
  instead of borrowing the loaded comparison's full-width toolbar order.
- Existing mounted/enabled filtering, menu arrow navigation, Tab dismissal,
  reverse traversal, task/dialog boundaries and mutation guards remain intact.

No visual styling, labels, tokens, screenshots, capture/save/delete operations,
review drafts, baseline eligibility or consent rules change.

## Verification

Regression coverage in `toolbar_tests.rs` checks a separately specified reading
order against mounted controls and toolbar geometry, then dispatches full Tab
and Shift-Tab press/release events including wraparound. Cases cover 1280×800 and
820×600, expanded/hidden navigation, visible/hidden results, compare, overlay,
single-image sides and split slider; separate cases exercise View/Zoom dismissal,
filters, diagnostics, note editing, expanded analysis and drawing. Stored evidence
and unsaved note text are checked for preservation.

Core shell tests cover candidate ordering, uniqueness, disabled/busy controls and
modal boundaries. Existing toolbar, input, disclosure and mounted-control tests
provide adjacent regression coverage. Execution results are recorded below.

### Execution results and current blocker

- All 14 core shell tests passed, including the new ordering assertion.
- The first UI build succeeded. Existing toolbar geometry and single-image tests
  passed, as did the new full menu-exit / inspector traversal test at both sizes.
- The new ordinary-review traversal test passed its initial Compare, Overlay and
  Single image states, then its geometry assertion encountered GPUI's retained
  debug bounds for now-unmounted Before/After controls. The assertion now checks
  mount state before reading geometry. Its full confirmation run is **blocked**,
  not passed.
- Two keyboard press/release tests and the 25-state enabled-candidate / actual
  mounted-handle sweep passed using that successfully compiled test binary.
- A subsequent concurrent change to `crates/app/src/ui/scrollbar.rs` prevents
  rebuilding: E0446 at line 110 exposes private `scrollbar::Axis` in a public
  associated type. This file was unchanged at task start and is outside this
  repair; its ongoing edits have not been reverted or modified here. Final UI
  confirmation and production-build qualification remain blocked on that error.
- Scoped Rust formatting and `git diff --check` passed. No screenshot/render or
  whole-suite pass is claimed.

This addresses the reported logical-order defect relevant to WCAG 2.4.3 (Level A).
Simulated GPUI input/layout tests do not establish OS keyboard, screen-reader,
other-platform or whole-application WCAG conformance. No such claim is made.

The host's missing linker names are supplied only by temporary aliases under
`/var/home/olav/Data/opencode/tmp/opencode/regressee-focus-order-ufe1jpze/`.
The repository's existing aliases and toolchain configuration are unchanged.
