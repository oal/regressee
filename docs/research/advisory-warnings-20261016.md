# Advisory capture warnings — 2026-10-16

The user rejected the lengthy warning and the blanket rule preventing review
when both screenshots had loading failures. This supersedes the warning-veto
policy in the earlier footer and comparison-recovery records, not their retained
history or border-only viewer-focus fix.

## Actual reported state

A query-only connection to the reported project's latest saved displayed
analysis found **24 of 25 reports with zero changed pixels** and one `Changed`
report. All 25 summaries had been vetoed as `NotComparable`; no hard comparison
gates were present. The earlier query also established stable captures and the
same image/resource/embedded-content warnings on both sides. Warning codes do
not prove identical imagery; the saved pixel measurements establish that here.
The user's database, screenshots and baseline were not changed.

## Current behavior

- `review::analyze_run` uses pixel-analysis results rather than blanket capture
  warning/stability rejection. Actual gates and rendering-environment mismatch
  remain noncomparable. A cache-version change separates the new policy.
- `ComparisonEvidence::review_classification(s)` recovers a useful saved pixel
  verdict from an older vetoed summary only when there is a matching nonempty
  report, agreeing pixel counts, no failure, no gates and no rendering mismatch.
  Storage verifies source/report bindings and artifact integrity. This is a
  read-only interpretation: original reports, summaries, analysis IDs, review
  decisions and notes stay intact. No migration or forced reanalysis is needed.
- Queue labels/filtering, coverage, progress, decisions, notes, Next, Details,
  Analysis and selective approval use that same verdict. Both identical and
  genuinely different images with matching warning codes work normally.
- The ordinary footer has one 13px/18px muted caution with a native amber warning
  mark: **Some content didn’t load in both captures. You can still review.**
  No warning paragraph, diagnostic list or mandatory Details detour. The optional
  Details panel retains the complete, side-bound explanation.
- Stable captures with ordinary loading warnings can be explicitly selected for
  baseline update. Its existing consent dialog adds one short caution about
  retaining their captured appearance. Selection/preview alone never updates it.
  Unknown/replacement warnings and instability still prevent selective baseline
  replacement; missing/corrupt captures, incompatible profiles/environments,
  hard comparison gates and stale revision/consent protection remain separate.

## Verification

- Regression coverage uses real saved warning-bearing captures and the production
  analysis service, including zero-change, changed and failed targets. It persists
  and reopens an old vetoed summary, saves a review/note, verifies no analysis
  rewrite or implicit baseline update, rejects missing/unsafe selections, then
  explicitly confirms a valid selection while retaining the earlier baseline.
- The offscreen recipe also persists old vetoed summaries with valid reports.
  At **1280×800 and 820×600**, it exercises pointer and Tab viewer focus, Details,
  Analysis, actual decision/note saves, baseline selection/preview/cancel,
  Next unreviewed and long genuine failures. Baseline/artifact/analysis records
  remain unchanged; intentional review/note saves are the only evidence changes.
- All **18** Linux GPUI offscreen frames were personally inspected in one batch.
  The shorter caution, usable decisions, note saving and consent fit both sizes;
  genuine long failures retain bounded scrolling. No scoped visual repair needed.
- Shell suite: **201 passed, 1 ignored, 1 filtered**, using
  `--skip qualification --skip bulk_consent`. App library: **125 passed, 2 ignored**.
  Analysis/review integration: **8 passed**. Core: **54 passed**.
  Storage: **35 passed, 2 ignored**. Strict capture-feature test-target Clippy passed.
- Production `ui` binary build and `git diff --check` passed. All 18 PNG hashes
  and 73 current source/asset hashes per frame agree; caption/gap RGB crops remain
  identical in all four pointer/keyboard-focused frames. Results are recorded in
  `comparison-recovery-verified.json` beside the captures.

Private synthetic evidence (not the user's screenshots or shipping assets):

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-review-advisories-hxreh4jn
```

`comparison-recovery-manifest.json` names the frames and per-frame JSON records
bind their PNGs to source hashes. The established renderer calibration passed
with 1,024 exact pixels for each calibration colour. Reproduce with
`comparison_ux_tests` and opt-in `renderer_capture_comparison_recovery`, using the
existing approved-root, calibration and synthetic website-fixture contract.

This is simulated-input and Linux offscreen-rendering evidence, not OS input,
presentation, assistive-technology, Windows/macOS, usability or release approval.
No new tokens, dependencies or shipping rasters. Concurrent screenshot-loading
work in shared/loading files was preserved, not reverted or accepted here.

The bounded finish pass ran inline, not through an independent reviewer. Scoped
disposition: **ship**, with no material fixes. Its five-section record is
`.impeccable/review/advisory-warnings-20261016/finish-review.md`. This does not
replace the outstanding native/platform qualification.
