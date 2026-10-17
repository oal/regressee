# Unchanged comparisons do not require review — 2026-10-16

## Diagnosis

Read-only inspection of the user's saved inorge.net check verified all 25 original
image pairs: 24 are pixel-for-pixel identical, including `/side/arbeidsmarked/`.
The homepage differs in its random-joke box; its broken-image row is identical.
No user screenshots, decisions, analyses or baseline references were changed.

The UI conflated **comparable** with **needs a decision**. Progress already counted
only unreviewed `Changed` results, but the footer mounted disposition buttons for
both comparable classifications and Next also visited `NoSignificantChange`.

## Implementation plan

1. Use the saved effective pixel verdict, including legacy warning-vetoed reports,
   to distinguish changed results from results that need no manual review.
2. Give `NoSignificantChange` a compact success footer: **No significant change —
   no review needed**. Do not mount Unreviewed / Intentional / Needs fix there.
   Keep optional notes, stored decisions and explicit baseline selection intact.
   Pending analysis and missing/unreadable/noncomparable evidence must not claim
   success. Below-threshold differences must not be called identical pixels.
3. Restrict Next unreviewed to actual, captured, comparable `Changed` results
   without a current decision. Preserve filters, wraparound, revision binding,
   drafts, modal guards and the separate baseline-confirmation workflow.
4. Make progress distinguish automatic no-review-needed results from manual
   decisions. Keep loading cautions advisory without inviting unnecessary review.
5. Add regressions for mixed/all-unchanged runs, matching loading warnings,
   historical reports/decisions, notes, filters, pending/failed states and keyboard
   mounting. Run scoped and existing shell tests, production build and a bounded
   two-size GPUI offscreen inspection. Record results and limits below.

## Verification

- Implemented the plan in `review_ux.rs`, mounted/enabled guards in `main.rs`,
  status/progress in `workflow.rs` and advisory copy in `comparison_ux.rs`.
  No comparison algorithm, stored verdict, decision or baseline mutation policy
  changed. Notes may still save an `Unreviewed` note record without inventing
  human approval; automatic completion is derived from the comparison result.
- All **3 new GPUI regression tests pass**. They use genuinely matching and
  changed saved corpus pairs with the same loading warnings, normal and legacy
  summaries, a failed target, real note/decision saves and frozen-evidence checks.
  The mixed matrix covers 1280×800 and 820×600, filtered and wrapped navigation,
  hidden-action rejection and actual simulated Tab skipping the decision group.
  Pending analysis, unreadable images and existing saved Needs fix notes are
  covered separately. No customer data is used by these mutation tests.
- The production binary builds with `cargo build --locked -p regressee-app
  --features ui --bin regressee`. A pre-existing/in-progress scrollbar addition
  exposed a private `Axis` through GPUI's public Element associated type. The only
  repair to that separate implementation is making the enum public; its behavior
  and other concurrent work were preserved. This host also required private
  development linker-name symlinks; no system libraries/configuration changed.
- Renderer calibration passed with **1,024 exact pixels per calibration colour**.
  The bounded offscreen test passed and all **10 frames** were personally
  inspected in one batch: success with warnings, navigation to a real change,
  no-change filter, pending analysis and unreadable evidence at both sizes.
  Success is explicit, dispositions are absent and warnings remain secondary;
  no scoped visual correction was needed. All PNG hashes and six task-source
  hashes per frame matched the inspected current sources. No shipping raster,
  new palette, token or asset was introduced.
- Scoped formatting and `git diff --check` passed. Wider-suite results are
  recorded below rather than being implied by the scoped pass.

Private synthetic evidence:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-unchanged-review-20261016-mzah39p6/`.
`unchanged-review-manifest.json` lists the frames; `source-receipt.json` binds
their PNGs and relevant sources. The website images are unchanged copies of the
existing synthetic advisory-warning fixture, not the user's screenshots.

**Wider-suite results:** all **8 analysis/review integration tests pass**,
including warning-bearing pixel review, masks, saved decisions and explicit
baseline consent. The final shell run (`--skip qualification --skip bulk_consent`)
completed with **207 passed, 3 failed, 1 ignored, 1 filtered**. The remaining
failures are outside unchanged-review handling:

- `app_settings_tests::browser_integrity_check_keeps_about_and_shortcuts_available`
  — App settings action exceeds its scroll owner.
- `tests::project_selection_actions_and_long_names_fit_supported_window_sizes`
  — Change project button exceeds the asserted workspace inset.
- `disclosure_tests::all_inline_disclosures_attach_content_restore_focus_and_do_not_mutate`
  — page-search disclosure is not enabled in its fixture state.

The app-library run completed with **132 passed, 1 failed, 2 ignored**: the
unrelated multiline note caret/scroll geometry test failed
(`ui/text_input/note_tests.rs:268`). Shared settings, focus and text-input code
was being edited separately during this run; this task does not replace or
claim acceptance of that work. The combined command stopped before integration,
which is why that suite was run separately. Earlier fixture-construction and
candidate-control assertions in the new tests were corrected; their final scoped
and full-shell runs pass without relaxing the mounted/enabled or Tab assertions.

**Scoped finish verdict:** the user's unchanged-review defect is resolved in the
inspected states. This is Linux GPUI/Blade offscreen rendering and simulated
input, not OS-input, accessibility, cross-platform, usability or release
acceptance. Wider-suite failures must remain visible; this is not an assertion
that the entire workspace is green.

## User-directed copy follow-up

The user found “no review needed” redundant. The current status is simply a check
mark and **No significant change**. The duplicate unchanged count is omitted from
footer progress because the comparison strip already provides that count; the
footer retains actual changes to review, saved decisions and gaps. The automated
completion, notes, navigation and baseline-confirmation behavior is unchanged.
All 3 scoped regression tests and the production build pass with the shorter
copy. Earlier captures above document the earlier wording, not the final copy;
fresh bounded two-size evidence is recorded in `concise-copy/` under the same
private evidence root. Wider-suite qualification and limitations above remain.
