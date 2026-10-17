# Baseline action clarity — 2026-10-16

## User-confirmed scope

The user supplied a review-screen screenshot with an arrow at **Select for
baseline update**, reporting that a test participant did not understand its
purpose. They chose **Clarify this action**, not an explanation/redesign of the
whole review flow. Preserve the native Operate surface and navy/teal identity.

## Change

- **Update baseline** replaces the generic Baseline heading. Attached help says
  **Use After as the reference for future checks. Review before saving.**
- **Select After screenshot** / **Deselect After screenshot** identify the original
  being selected. Existing selected/ready counts remain evidence-derived.
- **Review update…** appears only after selection. The confirmation asks **Use
  this After screenshot for future checks?**, with the plural form for multiple
  selections. It explains future comparisons, unchanged references and History.
- **Save baseline update** names the committing action. Selection, preview and
  Keep reviewing still do not save. Original capture cautions, source details,
  Replace/Keep references and pinned confirmation actions remain.
- Copy and actions stack below 900px working width. Shared type, ink, spacing,
  button targets, focus, shadows and selection styling are reused.

No storage, analysis, eligibility, capture, screenshot pixel, review-decision,
stale-preview, draft or explicit-consent rule changes. No new assets or tokens.
The first-run guide and UX-writing rules use the new action labels.

## Verification status

The expanded footer regression covers zero/one selection, preview without saving,
Keep reviewing, stale-selection removal, blocked confirmation, note drafts and
action/help containment at 1440×900, 1280×800 and 820×600, with navigation shown
and hidden. A second regression covers multiple eligible screenshots and verifies
that preview/cancel leaves stored evidence unchanged. The native offscreen
capture harness adds confirmation and 1440×900 coverage to its existing states.

Initial simulated-layout inspection found a narrow footer wrapping problem; the
footer was changed to an explicit stacked layout, with its retest pending. Final
verification is blocked because
concurrent unrelated theme changes currently remove color constants still used
by shared controls. Those files have not been reverted or repaired by this task.
Scoped Rust formatting and `git diff --check` passed. Finish verdict: **blocked**,
not visually accepted; rerun the scoped footer tests and capture after that
refactor compiles.

Private diagnostic root (synthetic fixture copies only):
`/var/home/olav/Data/opencode/tmp/opencode/regressee-baseline-clarity-rmd1t8l5/`.
No final test, visual, native input, accessibility, cross-platform or usability
acceptance is claimed while verification is blocked.
