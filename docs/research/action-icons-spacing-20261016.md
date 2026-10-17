# Action iconography and inspection grouping — 2026-10-16

## Confirmed scope

After approving the compact toolbar, the user requested icons for more buttons
and identified the loose inspection metadata as incohesive. They selected
**App-wide consistency** for the follow-up walkthrough. Two isolated assessments
agreed on the header's equal-flex columns and the narrow review row's capacity
as the main structural issues. This extends Clear wayfinding; no new identity,
palette, type family, dependency, animation or shipping raster is introduced.

## Built treatment

- `ui/controls.rs::action_with_icon` adds an 18px native stroke slot with 8px
  spacing to the existing action frame. Glyph ink follows Primary, Secondary,
  Danger, Navigation and Choice text. Labels retain their selectors and wrap in
  a finite column; selection, focus and disabled states do not change geometry.
- `action_icons.rs` maps actual controls to recurring action meanings, not label
  substrings. Review choices, notes, next actions, navigation, project actions,
  settings saves, baseline selection, ZIP previews and deletion keep explicit
  wording and their existing guards. Form choices and disclosures stay distinct.
  A native `ActionLabel` measures its natural line width from the effective window
  text style, retaining intrinsic sizing while allowing constrained wrapping.
- The 40px inspection heading groups path/profile and the paired Before/After
  sources with a 16px gap. A native `RenderOnce` component uses window-local
  shaped text to size capped groups instead of equal-width columns or zero-width
  ellipsis children. Original dimensions have a nonshrinking slot; long names
  yield first. Source context remains evidence-derived and available in Details.
- Review note sits near the three classification choices; only Next occupies
  the trailing position. Below 720px **actual working width**, whole groups form
  two action rows. With expanded navigation at 820px, this deliberately costs
  48px of evidence height rather than dropping labels, shrinking targets or
  overriding the user's sidebar preference. Wider workspaces retain one row.
- The global selected-project minimum is 180px rather than 240px, avoiding an
  unnecessary wrapped header when icon-labelled actions are present.

## Verification and limits

The first native batch exposed premature header ellipsis and a long-action-label
wrapping defect. They were corrected together, along with the narrow global
header minimum, before the final confirmation batch. First-pass pixels are not
qualification for those corrected states.

The final-source app library suite passed **129 tests, 2 opt-in tests ignored**,
including 66 shared UI checks. The new primitive tests cover all action styles,
wrapped labels and state-invariant bounds. The six-mode toolbar/header matrix
passes at 1440×900, 1280×800 and 820×600 with navigation both expanded and hidden.
The review-footer state matrix now uses the same three sizes and both navigation
preferences, preserving drafts, baseline-selection and consent guards.

The independent finish review identified a remaining material defect in the
second batch: a bare wrapping column lost its preferred width inside intrinsic
action containers. The full shell run had **200 passed, 3 failed, 12 ignored**;
Projects import and App settings actions painted collapsed character columns.
Those valid failure captures are retained in `before-fix/`, with the original
review in `finish-review-before-fix.md`; they are not current acceptance evidence.
The shared `ActionLabel` correction above fixes the primitive without per-button
widths, ellipsis, removed icons or relaxed assertions. All three original failing
tests pass unchanged, and additional intrinsic-row/finite-column tests cover
short and long labels, wrapping groups, selection, focus and disabled states.
The confirmation set was rerendered over the same paths for the reviewer's verdict.

## Final verification and finish record

- Final production UI build passed with `cargo build --locked -p regressee-app
  --features ui --bin regressee`.
- Full final-source shell suite: **203 passed, 0 failed, 12 opt-in tests ignored**.
  The app library suite is **129 passed, 0 failed, 2 opt-in tests ignored**.
- The repaired renderer test passed and retained unchanged stored evidence:
  **42 current GPUI/Blade PNGs**, 14 scenes at each of 1440×900, 1280×800 and
  820×600. Image validity/hashes and eight current UI source hashes were checked
  for every frame; `source-receipt.json` binds the current `confirmation/` set.
  All full-frame sheets and the affected Projects/App settings views were read.
- The independent reviewer scored its sole shared-label sizing finding
  **resolved**, disposition **ship** for that listed fix. This is a verdict pass,
  not fresh whole-app, native input, accessibility or release certification.
- The documentation handoff narrowly updated `DESIGN.md` and the existing local
  native sidecar, preserving previous records, palette primitives and unrelated
  drift. Reports: `finish-review.md` and `documentation-report.md` under the
  private evidence root. `git diff --check` and scoped formatting checks passed.

Private evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-action-icons-20261016-9e4osmog/`.
Synthetic website fixture provenance is recorded in `fixture-provenance.json`;
no customer captures or new shipping images are represented by these samples.

Evidence is Linux GPUI/Blade rendering and simulated input, not OS-input,
screen-reader, Windows/macOS, usability or public-release certification. The
existing native accessibility integration gap remains. Unrelated workspace
changes, previous verification records and backend behavior are preserved.
