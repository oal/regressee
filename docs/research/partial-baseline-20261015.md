# Partial baseline recovery — 2026-10-15

## Outcome and scope

A single failed screenshot no longer blocks saving the rest of a finished
baseline capture. This is a focused Operate-mode refinement of Clear wayfinding:
existing teal actions, native confirmation, amber coverage, shared controls and
untinted evidence. No new tokens or shipping raster assets.

- **Save baseline…** remains available when a finished baseline capture has at
  least one successful screenshot. The next-step message gives actual coverage
  and explicitly offers saving with missing screenshots skipped.
- **Review failures** clears local search and applies the existing missing-result
  filter, revealing the queue without changing the inspected screenshot or drafts.
- The native confirmation names saved/skipped counts, explains that skipped
  page/screen-size pairs will not be checked, and distinguishes replacement of
  the current baseline from creating the first one. Keep reviewing never saves.
- Saving is explicit background work. The original run, failures, intended
  coverage, and previous baseline versions remain intact. The existing partial
  baseline model is used; no storage migration is needed.
- **Run check** becomes the primary next action after saving. Accepted coverage
  is not repeated across multiple persistent warning surfaces. Subsequent checks
  retain a reduced-baseline coverage label, and History retains saved/intended
  counts. The first-run guide explains how to capture a replacement later.

Checks schedule exactly the saved reference keys, not every original page ×
screen-size combination. New baseline captures still require the full intended
target list. Storage verifies a check's exact baseline membership before starting;
missing or extra pairs are rejected. Empty, unfinished, interrupted, cancelled,
preview and check runs cannot be saved as a new baseline. Explicit partial consent
does not bypass original-image verification, revision checks, imported-source
restrictions or selective-approval eligibility.

## Verification and finish verdict

**Scoped pass for partial-baseline recovery.** The action remains obvious at both
desktop sizes, cancellation leaves the current baseline untouched, and confirmed
saving leads to a usable saved baseline rather than another blocked workflow.
The broader UI suite is not completely green; see the limitation below.

- Core/storage run: **53 + 35 passed**, two profiling tests ignored.
- App library: **118 passed**, two opt-in tests ignored.
- UI binary: **187 passed**, one opt-in test ignored, with the known failing
  missing-image scrolling test explicitly skipped in the confirmation run.
- Existing analysis/approval/portability integration: **7 passed**.
- New offline partial-baseline integration: **2 passed**. Covers explicit consent,
  stale revision refusal, preserved parent and immutable failures, exact three-of-
  four page/size membership, rejection of extra/omitted pairs, successful analysis,
  ZIP export/import, no-success refusal and corrupt-image refusal.
- Native controller test covers both sizes, action targets, painted keyboard
  action order, singular/plural and 149/150 confirmation copy, prompt cancellation,
  explicit background save, saved-state recognition and enabled checking. It and
  all **12 core focus tests** passed after the final keyboard-order adjustment.
- Production native build and the explicit renderer test passed.

The full UI run also found
`screenshot_placeholder_tests::screenshot_placeholder_fills_the_stage_and_keeps_recovery_reachable`
failing its long-explanation scroll assertion (help below its owner viewport).
It reproduces in isolation. That concurrently present missing-image test and its
scroll implementation were not edited by this task; no whole-suite pass or fix
for that separate surface is claimed.

## Visual evidence and limits

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-partial-baseline-20261015-gGZP1w/`.
The initial renderer attempt stopped on a stale debug-selector absence assertion;
the assertion now checks mounted controller state instead. `confirmation/` holds
the first complete visual batch; `final/` holds the six reviewed final frames:
ready to save, failure filtering, and saved baseline at **1280×800** and **820×600**.
The final visual refinement removes repeated accepted-coverage warnings and makes
Run check primary. The subsequent keyboard-only ordering change was verified by
controller/focus tests, not another visual-polishing loop.

These are actual Linux GPUI/Blade offscreen frames over synthetic, isolated local
corpus evidence. They are not real website captures, OS-input, screen-reader,
Windows/macOS, native-dialog rendering or public-release qualification. Native
confirmation behavior is exercised through GPUI's simulated prompt API. Existing
unrelated changes remain intact.
