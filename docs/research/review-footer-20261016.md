# State-aware review footer — 2026-10-16

User-confirmed scope: simplify the highlighted review/baseline area, preserving
the existing navy/teal identity and every explicit approval safeguard.

**Superseded for the reported missing-cause and viewer-focus states.** The user
subsequently demonstrated that the generic explanation/Details route did not
explain their real capture warnings, and image focus painted a teal fill behind
captions. The verification and verdict below describe the earlier snapshot, not
acceptance of those states or current source. Actual causes now appear inline;
supporting **Technical details** is optional, and viewer focus uses an ink border.
Fresh findings, source-bound evidence and qualification limits live in
`docs/research/comparison-recovery-20261016.md`.
The later user-directed advisory-warning policy and shorter caution supersede
that repair's warning veto; see `docs/research/advisory-warnings-20261016.md`.

## Initial implementation (historical)

- Comparable results have one Review heading/progress line and short selected
  disposition choices, with Review note and Next unreviewed grouped separately.
  Zero-valued secondary counts and the repeated baseline disclaimer are removed.
- Noncomparable results replace disabled decisions with one explanation and
  **View details**. Next unreviewed remains available when another matching,
  reviewable result exists; it disappears when there is no destination.
- The separate baseline footer mounts for an eligible current screenshot or an
  existing selection. Its confirmation-preview action mounts only after selection.
  An invalid selection remains removable and cannot be confirmed.
- View details opens the existing inspector, focuses Close and restores its own
  trigger on dismissal. Mounted controls match rendered actions; note precedes
  Next in the review keyboard order. Notes still require explicit saving.
- No capture, screenshot pixel, storage, comparison, review-decision binding or
  baseline-confirmation rule changes. No new tokens, assets or shipping rasters.

## Initial verification (historical)

- Native-shell suite: **199 passed, 0 failed, 1 ignored, 1 filtered out**, using
  `--skip qualification --skip bulk_consent`. This does not qualify bulk consent.
- New two-size shell test covers simulated pointer dispatch, note-draft retention,
  selection/removal, noncomparable states, pending analysis, stale selections,
  inspector focus restoration, mounted actions and painted control bounds.
  Evidence height increases when irrelevant footer controls disappear.
- App library (`ui-tests`): **124 passed, 2 ignored**. Core: **54 passed**.
- Strict `ui-tests` and `ui-renderer-capture` test-target Clippy, production `ui` build and diff whitespace
  check pass. The host's Rust toolchain is selected per command with
  `mise exec rust@1.98.1`; no global toolchain settings were changed.
- Actual Linux GPUI offscreen calibration passed with 1,024 exact pixels per
  calibration color. Personally inspected all **12** synthetic-project frames:
  ready, selected, mixed/all noncomparable, pending and stale selection at
  **1280×800 and 820×600**. PNG hashes and six current-source hashes per frame
  were verified.

Private evidence and per-frame metadata:

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-review-footer-xgib74eh
```

Reproduce with `review_footer_tests` and the opt-in
`ui_renderer_capture::ui_review_footer_capture::renderer_capture_review_footer`,
using the existing private-root, calibration and synthetic website-fixture
contract. `review-footer-manifest.json` lists the frames;
`review-footer-verified.json` records source/hash inspection. The renderer proof
and actual app frames are development evidence, not shipping assets.

These checks do **not** establish OS-input, assistive-technology, Windows/macOS
behavior, native usability or public-release acceptance. No real user project,
screenshots, login session or baseline was modified.

## Initial finish review (historical; superseded at the scope above)

Independent review inspected all 12 frames and the changed wiring, finding no
material footer layout or behavior defect. Its sole material finding was outdated
DESIGN guidance for the empty selection and footer mounting. After that prose
was corrected, the reviewer scored the finding **resolved**, disposition **ship**
for the persistence fix. Source and frame hashes stayed unchanged; no recapture
was required for this documentation-only correction. This is not whole-app or
cross-platform approval.

## Initial documentation check (historical)

- Compared the dated surface contract, `PRODUCT.md`, corrected `DESIGN.md`,
  `.impeccable/design.json` and `docs/first-run-guide.md` with
  `crates/app/src/{review_ux,workflow,main}.rs`, `crates/core/src/shell.rs`,
  shared `ui/{theme,controls}.rs` and `review_footer_tests.rs`. Personally inspected
  all 12 manifest-listed GPUI frames: ready, selected, mixed/all noncomparable,
  pending and stale selection at both sizes. Rechecked every PNG hash and all
  six recorded current-source hashes per frame; they match.
- Current footer guidance matches implementation: short selected dispositions,
  conditional View details/Next, explicit note saves, separate eligible-or-selected
  baseline footer, no empty confirmation action, and removable stale selections
  with confirmation disabled. Pending analysis retains disabled dispositions and
  its explanation. The corrected empty-selection heading is **Baseline**.
- Incumbent navy/teal/mint roles, system type, 14px/20px actions, 13px/18px help,
  40px minimum controls, small corners and ruled grouping are reused. DESIGN's
  token frontmatter and shared theme/controls are unchanged from HEAD; the existing
  sidecar is preserved. No new tokens, assets or shipping raster were introduced.
- The first-run guide's explicit selection/confirmation and draft-saving advice
  remains valid; no direct contradiction introduced by this task requires action.
  Pre-existing label drift remains at lines 271–273: **Review approval** is now
  **Review baseline update…**, and **Hide page queue** is **Hide results**. Both
  mismatches already exist in HEAD, not as a consequence of this extension.
- Unrelated sidecar drift is not canonized or repaired: `nativeGeometry` still
  records a 420px-high/580px-narrow approval dialog versus current 520px/740px;
  `nativeFocus` records a 2px spread instead of the shared 5px outer/2px gap paint.
  These predate this task; DESIGN and the sidecar were not edited by this pass.
- This is a bounded documentation/source/offscreen comparison, not a new finish
  verdict. Test/build/Clippy results above are independently supplied, not rerun
  here. OS input/presentation, assistive technology, native usability,
  Windows/macOS and release acceptance remain unqualified.
