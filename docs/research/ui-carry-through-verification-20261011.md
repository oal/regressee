# UI carry-through — 2026-10-11

The Tools/Details interaction described below was subsequently replaced by the
user-requested direct-viewer refinement; see
`viewer-interactions-verification-20261011.md`. This record preserves the earlier
implementation and its verification evidence.

**Outcome: requested corrections complete; compiler/regression gates and
independent offscreen-layout confirmation pass.**

This is the user-requested completion of the remaining issues recorded in
`ui-critique-improvements-verification-20261011.md`, not a new visual direction
or a replacement for that earlier failed confirmation evidence.

## Corrections

- History now uses the full-width selection-row primitive, not an intrinsically
  sized action. The containing list has explicit column geometry; selected and
  focused paint cover the row rather than just its text.
- Tools temporarily replaces the queue with a full-height
  inspector. It no longer consumes screenshot height in a second bottom panel.
  Closing Tools restores the previous queue state and preserves drafts. Hidden
  queue controls are excluded from focus navigation, and fit geometry accounts
  for the inspector's width. While Tools is open, review state/progress remain
  visible but decision/note actions are temporarily unmounted; closing Tools
  restores ordinary review and any retained note draft.
- Details/Tools use invariant, selected-state labels, avoiding the extra toolbar
  row caused by longer open-state labels. Keyboard area editing is first in
  Tools; original-pixel fields have compact labels, with Apply and Save together.
- Empty search results have compact recovery copy instead of a large padded
  empty-state block and a repeated outside-filter paragraph. Expanded filter
  options scroll separately below the recovery message, without changing the
  inspected evidence or drafts. Recovery is placed before the search controls;
  disabled Previous/Next actions are omitted when there are no matches.
- Keyboard help is concise and uses bounded, scrollable dialog content with
  protected title/Close geometry. The complete final caution remains in the
  default view at the inspected window sizes.
- Capture fixtures use the real guarded History selection/open route before
  making presentation-only gap/ready/saved clones. They assert the original run,
  absence of comparison and pair-analysis request, single-pane presentation and
  original viewer identity. No guard was weakened to make a fixture pass.
- The diagnostic renderer can accept an explicitly supplied physical approved
  scratch root, while retaining the privacy-cleaned placeholder default. It
  verifies canonical paths and private output-directory permissions; no
  developer home path was restored in project source.

## Regression and renderer gates

Evidence is retained in the private external `regressee-ui-carry-through-0vJeaL/`
directory. Compiler and renderer results are recorded there separately.

- `format.log`: `cargo fmt --all --check` passed.
- `clippy.log`: locked workspace/all-targets/all-features Clippy with `-D warnings`
  passed. Existing vendor GPUI fallback/dependency notices remain separate.
- `tests.log`: **303 passed, zero failed, 38 ignored** in the all-features workspace
  suite. The three new layout regressions and extended numeric-focus test pass.
- `ui-build.log`: ordinary UI application build passed.
- `renderer.log`: fresh 36-image batch passed all state/layout/storage invariants.
- `integrity.json`: all 36 hashes, decoded dimensions and opaque alpha pass.
- `source-binding-before.json` equals `source-binding-after.json`: 295 bound
  source/configuration inputs and both executable hashes were unchanged.

New/extended GPUI regressions cover selected History row/list width equality,
all four numeric-field focus positions, at least 96px of original-image viewport
height with Tools open, visible empty-result recovery with unchanged evidence
and drafts, and complete default keyboard-help content with protected controls.

The fresh renderer matrix covers 18 states at both 1280×800 and 820×600,
including capture-with-gaps, ready-to-save and saved-baseline originals. The
harness asserts relevant state and painted layout, not merely that a PNG exists.
No PNGs are retained from the earlier failed confirmation. Before/after records
bind source/configuration inputs, the actual test executable and ordinary UI
executable. The synthetic project's authoritative project/evidence snapshots
must remain unchanged.

## Independent pixel confirmation

`independent-confirmation.md` records acceptance after inspection of every one of
the 36 actual PNGs, with critical states reopened at native size. No remaining
blocking pixel defect was found in the requested corrections or inspected
regression states. It independently confirms the hashes/dimensions/alpha and
matrix, matching before/after executable bindings, and unchanged live source
inputs.

Capture gaps/ready/saved states show one original, compact origin/dimension
captions and no comparison or approval chrome. History paints the full selected
row. Numeric focus has a visible border/caret, with roughly 120px of original
image height at the narrow size and 248px at desktop size. Default review and
notes remain intact with Tools closed; complete empty recovery and keyboard-help
caution/Close are visible. Discovery retains four visible inventory rows at the
narrow size, and historical inspection remains visibly read-only.

Lower narrow inspector controls still require scrolling (Apply is partly visible
in the initial X-field-focused capture; Save is below that viewport). This is a
bounded inspector, not another thin-strip screenshot or missing-focus failure.
The earlier failed evidence remains unchanged. This follow-up used one fresh
confirmation batch, with no post-confirmation UI edits or additional polishing.

## Qualification

Offscreen GPUI/Blade application pixels and headless focus models are scoped to
this Linux development host. They are not real-site screenshot results or native
OS close/quit/input, IME, screen-reader, high-DPI, Windows/macOS or public-release
acceptance. This task does not claim a new usability score or measured user gain.
Existing user projects, unrelated changes and the privacy cleanup are preserved;
no website was visited and no commit/push was made by this task.
