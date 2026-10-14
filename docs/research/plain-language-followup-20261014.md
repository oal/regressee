# Plain-language follow-up — 2026-10-14

The earlier wording pass missed the note editor's storage terminology and its
nonstandard closing label. This follow-up uses Impeccable's clarify workflow,
preserving the existing native visual direction and product behavior.

## Changes

- All viewer panels use **Close**. Closing a note panel keeps its draft and
  does not save it; the helper names **Save note** as the saving action.
- Notes show **No changes**, **Unsaved changes** or **Saved**, not byte counts.
  Length rejection asks users to shorten the note or paste less and explains
  whether anything was added or saved. Invalid saved/legacy drafts stay editable.
- Setup and validation use pages at once, approved login websites, example
  website/section addresses, pixels and website language rather than parallel
  targets, origins or unexplained locale terminology.
- Advanced comparison settings explain color tolerance, small-area checks,
  spacing and connected changes. The optional blending filter explains its
  actual behavior and risk rather than an internal RGB-noise label.
- Capture warnings, missing screenshots, History, browser/login recovery,
  storage and the support report use task-oriented language. Internal build
  phases, bindings, inventory/evidence labels and renderer tile counts are not
  routine instructions. Exact source records and diagnostics remain available.

The internal 8,192-byte limit is unchanged and is **not** relabelled as a
character limit. Save, quit, paste and storage still validate exact text;
rejected input is not silently truncated. Approval, deletion, login, privacy,
website boundaries and baseline eligibility are unchanged.

Writing rules are reinforced in `docs/ux-writing.md` and `DESIGN.md`. Two new
copy tests guard the missed phrases and exercise actual close/reopen behavior,
including Unicode drafts at both supported desktop sizes. Existing exact
storage/save/quit/editor tests retain their behavioral assertions; outdated
wording expectations and capture selectors are updated to match the UI.

## Verification

App, core and storage library tests passed (**113 + 52 + 35**). The seven
`analysis_review` integration tests passed. Production build and strict production
Clippy passed. Formatting of `ux_copy_tests.rs` and `git diff --check` passed.

After the first corrections, the native shell run passed **181 tests**, with
the known workflow failure excluded and one remaining stale message assertion
failing. Both assertions in that test are now updated; its isolated final run
passed. Final same-source reruns also passed all **five copy tests** and **eight
review/containment tests**. This is not a claim of a clean, unfiltered suite.

The full native-shell run reproduced the previously documented failure in
`workflow::tests::discovery_owns_its_scope_and_review_preserves_provenance`:
`!s.workspace_control_mounted(Control::Guided(16))`. Its mounting logic was not
changed. Stale wording expectations and a longer setup hint found in the same
run were corrected; no assertions about behavior or containment were weakened.

The first note captures are retained at
`/var/home/olav/Data/opencode/tmp/opencode/regressee-clarity-go97pnh3`.
The accompanying app capture stopped before producing app frames because its
old mapping-field selector no longer matched. The harness now opens the actual
optional disclosures before inspecting them.

The second capture attempt is retained in
`/var/home/olav/Data/opencode/tmp/opencode/regressee-clarity-final-12qsajw3`.
It produced the twelve note frames and three setup frames before a stale
analysis-field selector stopped the app harness. Analysis captures now use
the stable parameter IDs rather than labels that the parent element replaces.
The final pixel-unit wording also removes CSS jargon from routine captions.

The final confirmation succeeded at
`/var/home/olav/Data/opencode/tmp/opencode/regressee-clarity-confirm-ayzk_qk7`.
Both capture tests passed, producing **12 note frames and 16 app frames**.
All 28 final frames were personally inspected. Close, draft-retention help,
Save note, the saved/unsaved states and both length warnings are readable at
both sizes. The closed/reopened captures and controller assertions retain the
draft, selected result and unchanged baseline. Setup, advanced comparison,
settings and support wording fit their inspected panels; longer sections remain
scrollable. `review-receipt.md` binds that inspection to the PNG hashes.

The eight review/containment tests, five copy tests, corrected actor test,
production build and strict production Clippy logs are retained in
`/var/home/olav/Data/opencode/tmp/opencode/regressee-clarity-final-12qsajw3`
as `final-profile.log`, `final-copy.log`, `final-actor.log`, `final-build.log`
and `final-clippy.log`. The production executable was rebuilt; no running app
was restarted. Changes remain uncommitted and unpushed.

Confirmation captures use synthetic local projects and byte-identical synthetic
website fixtures. The already inspected calibration is copied byte-for-byte,
with a reuse record; app frames are newly rendered from the final source.
`REGRESSEE_RENDERER_CLARITY=1` selects six note states and eight setup/analysis/
settings/support scenes, each at **1280×800** and **820×600**. These are Linux
GPUI offscreen layouts and simulated controller checks, not OS-input,
accessibility, usability, cross-platform or release qualification. No user
project, login, screenshot or database was used.
