# App-wide subtle depth — 2026-10-16

## User-confirmed scope

The user described the app as bland/flat, supplied Xcode as a reference for
subtle shadows and depth, and chose **App-wide** rather than comparison-only.
This refines Clear wayfinding, preserving layout, content, navy/teal identity,
mint selection, system fonts, evidence and all operation/approval boundaries.

## Implementation

- `ui/depth.rs` supplies ink-based, soft directional shadows for controls,
  opaque working surfaces, floating panels, dialogs and shell boundaries.
  Small contact shadows distinguish controls; temporary overlays have a broader
  ambient shadow. No zero-offset glow, decorative animation or dependency.
- `ground` is a slightly cooler `#edf2f4`, separating white content from its
  workspace without a new accent. The project header and inspector/disclosure
  titlebars use a near-white `surface` → `action-surface` vertical highlight.
- Labelled actions, selected navigation and enabled Zoom receive light lift.
  Disabled actions, form choices, individual list rows and transparent toolbar
  tools stay grounded. White editable fields keep their existing boundaries.
- Continuous page/result lists and individual opaque screenshot stages receive
  surface elevation. Menus, tooltips, inspectors and option disclosures share
  floating elevation; shared dialog surfaces have the highest elevation.
  Setup/baseline footers and the project rail retain their existing dimensions.
- Focus rings replace ordinary control elevation; hover does not overwrite them.
  The transparent image viewer retains its border-only focus and no shadow
  behind captions or pane gaps. Screenshot content, transforms and image bytes
  are unchanged. Selection is never communicated by a shadow alone.
- The design document, local native sidecar and surface brief record this
  deliberate replacement of the flat-only rule. No unrelated drift is repaired.

## Verification

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-depth-20261016-kw7otdgv/`.

- App library and shell tests with `ui-renderer-capture`: **335 passed, 0 failed**
  (131 library, 204 shell; 52 opt-in tests left ignored in that general run).
  Coverage includes shared action/focus/state geometry, contrast, text wrapping,
  toolbar navigation, dialogs and existing workflow/draft/approval protections.
  Two new primitive checks cover directional, soft elevation and grounded form
  choices/unselected navigation. See `tests.log`.
- The two opt-in capture tests then passed explicitly: **42 workspace frames**
  (14 scenes at 1440×900, 1280×800 and 820×600) and **16 dialog frames**
  (eight scenes at 1280×800 and 820×600). They retain immutable synthetic evidence
  assertions and dialog containment/dismissal checks. See `workspace-complete.log`
  and `dialogs-complete.log`.
- All 58 PNGs passed CRC/dimension/hash validation, opacity and real-renderer
  checks, plus 11 current UI source hashes per frame. `source-receipt.json` binds
  the complete `confirmation/` set. `source-hashes.json` confirms final sources
  did not change during inspection. An earlier runner timed out after 31 frames;
  those partial `inspection/` frames are retained, not substituted for the
  complete run. No styling edits were made between the two capture invocations.
- Both 1440×900 rendered screenshot-pane interiors are pixel-identical to the
  corresponding earlier flat-UI capture using the same frozen website fixture.
  See `untinted-evidence.json`. This checks rendered evidence, not just its files.
- Production `cargo build --locked -p regressee-app --features ui --bin regressee`
  passed. Scoped UI formatting, design-sidecar JSON/palette consistency and
  `git diff --check` passed. The existing dependency future-compatibility warning
  for `proc-macro-error2` is unrelated to these changes.

## In-session visual finish review

All four full-frame contact sheets were inspected together, with full-size
inspection of wide/narrow review, View menu, note editor, both settings surfaces,
narrow approval and task dialog. The requested depth reads as light control lift,
white evidence surfaces above a cooler ground, and stronger temporary overlays.
No new clipping, geometry change, competing card wrappers, focus-ring regression
or screenshot tint was found in the inspected states. No follow-up visual edits
or open-ended polish loop was needed. Verdict: **ship for this bounded visual
refinement**. This is an in-session review, not an independent qualification.

Native rendering uses synthetic offline fixtures copied byte-for-byte from the
previous action/icon inspection; `fixture-provenance.json` records their hashes.
No customer captures or new shipping rasters are introduced. GPUI/Blade offscreen
pixels and simulated input qualify development-host rendering only, not actual
OS input, accessibility, Windows/macOS, usability or release acceptance.
