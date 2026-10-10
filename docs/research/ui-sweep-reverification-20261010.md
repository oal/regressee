# UI sweep correction reverification — 2026-10-10

**PASS for the two bounded source/test-platform repairs.** Independently rerun:
**26 tests passed, zero failures, one deliberately ignored fixture-preparation
test not invoked.** No new issue found within this correction scope.
**Native visual/input verification remains BLOCKED / NOT RUN.** No renderer repair
was made or tested; this is not full UI, accessibility, human or release acceptance.

The original `ui-sweep-verification-20261010.md` and its private evidence remain
unchanged. This report closes its P1/P2 source findings at the qualification
level below; it does not overwrite the original native failure.

## Ownership, frozen candidate and evidence

No application edits, subagents, app/Xvfb/native/browser/worker launches, renderer
flags/retries, uploads or broader guardian/scale/fault/public-runtime suites.
Only this report and a fresh physical private correction directory were added:

`/path/to/private-tests/opencode/regressee-ui-sweep-hkonn1qy/correction-verifier-de0834b4da/`

Abbreviated **C** below. Source/binary before/after SHA-256 manifests:
`C/freeze-before.json`, `C/freeze-after.json`; every entry matches.

- `main.rs`: `8f4c7a1acf397b940558e74f5c00512fb7cf6b85472fa20cbdb7fd2ca95d3804`
- `core/src/shell.rs`: `8daf81f7c2c369b37bc4d9acf629d03416b53abd571e4216abf3e6e1ac9268e6`
- `target/debug/regressee`: `627778310caebba912cee8de068ccbef1b1f883fa7e6627e3ea06f5b359c6e99`

The manifests also cover controls/theme/input/GPU diagnostics, jobs, storage,
app Cargo and lockfile. Those eight files still match the original verification's
source hashes. The original report SHA-256 remains
`923f8e374e5942d4bb9dfd8a798f22fa7ac51c943911013fb6555a384e4a18f7`.
Every file listed in the original private evidence inventory still matches its
recorded hash. `C/final-audit.json` records these preservation checks.

## P1 — enabled candidates versus mounted controls: repaired

Read the actual renderer section boundaries and dispatch paths, not only the new
`control_mounted` helper. `control_enabled` now checks mount before readiness
(`crates/app/src/main.rs:3164–3168`); button rendering, keyboard/pointer dispatch,
Tab filtering and analysis-field enablement share that result. Crawl fields also
check mount before becoming editable (`main.rs:3464–3478,5051–5056`).

The original holes are now refused:

- Settings-only repair Phase6(13) cannot activate from History/Results.
- Setup-only login/mapping actions are removed from Results candidates and denied
  by the screen predicate.
- Empty Results search is not mounted/enabled without preview or captured run.
- Baseline-only viewers do not expose comparison-only view modes or analysis.
- Preview grouping is denied; advanced analysis and region navigation follow
  their actual section/report conditions.
- Unknown indexed actions deny; portability confirmation candidates now follow
  the screens where those panels actually render.

The regression `ui_sweep_enabled_candidates_match_actual_mounted_tab_handles`
**passed**. It draws the actual Shell into GPUI's test-platform frame, enumerates
`Window::focus_next` handles, and checks both directions of set equality with
enabled candidates plus matching counts. The observed oracle is GPUI's rendered
focus map, not another copy of `control_mounted`. It also verifies wrong-screen
repair activation submits zero requests.

Covered cases: empty Results, baseline viewer, discovery preview, comparison,
advanced/loading comparison, History, Settings, Projects, Setup and advanced
Setup. The deliberate **1120×8000 test viewport** exposes the whole form; it is
not a native screenshot or minimum-window/readability check. Equality verifies
membership/handle identity, not visual focus placement or traversal-order quality
for every possible state/confirmation combination. Source review supports the
additional mount branches; exhaustive state-space acceptance is not claimed.

## P2 — dirty-mask analysis scheduling: repaired

Reviewed every scheduler call, the advanced text/toggle paths, submission boundary,
analysis reply handling and successful SaveMask transition.

- `refuse_dirty_mask_analysis` cancels an existing token, clears due work,
  invalidates approval and retains a save-first explanation (`main.rs:740–755`).
- The shared scheduler refuses dirty masks. Advanced text does not erase its
  refusal message; already-due/pending analysis is guarded before submission
  (`main.rs:756–765,794–889`).
- Matching asynchronous analysis replies and their generic errors cannot replace
  displayed analysis when cancelled/dirty (`main.rs:308–348`).
- SavedMask clears dirty before normal scheduling (`main.rs:648–650`).

`ui_sweep_dirty_masks_block_all_analysis_scheduling_and_stale_replies` **passed**:
advanced text, toggles 27/28 and already-due work submit zero new Jobs request IDs;
pending token is cancelled; a synthetic late analysis response cannot replace the
displayed revision or rectangles. Dirty rectangles, persisted analysis history
and active baseline remain intact. Clearing dirty restores normal scheduling.
The cancelled/dirty generic-error refusal is source-reviewed; the new test's
late-reply injection specifically exercises an Analysis response.

## Foreground bounded test results

Commands/logs and exact owned cargo PIDs are in `C/tests.json` and
`C/test-{0..8}.log`. All ran foreground and serially with `--locked --offline`,
`--all-features`, physical `TMPDIR=C/tmp`,
`LIBRARY_PATH=<project>/.runtime/link-libs`, and `--test-threads=1`.

| Selection | Passed |
| --- | ---: |
| Main `ui_sweep_` (two new and two retained) | 4 |
| Legacy real-tile pointer/mask coordinates and synchronized actions | 1 |
| Legacy filtered preview / exact frozen inventory selection | 1 |
| Core shell | 5 |
| Controls style and actual test-platform bounds | 5 |
| Text input | 7 |
| Save/navigation with real competing-writer conflict | 1 |
| Storage coordinated atomic/private/revision-bound save | 1 |
| Jobs save/reopen without configured runtime | 1 |
| **Total** | **26** |

The ignored native-fixture preparation test was **not** explicitly run here.
Retained contracts were rerun despite source equality: exact GPUI focus-refinement
cause/fixed metrics, selected/disabled bounds, Unicode/selection/IME run logic,
atomic project+local-login save/rollback, conflict classification, active-baseline
retention and dirty Refresh preservation.

Legacy fixtures now supply a real project/comparison through Jobs Load before
pointer assertions, or the matching ProjectRecord for their synthetic preview.
Read source confirms original coordinate/filter/inventory/mask assertions remain;
the tests passed without relaxing the new guards. Parent-reported earlier legacy
fixture failures are historical implementation evidence, not failures observed
in this independent rerun. Parent's broader 115-pass/clippy/fmt/build gate was not
rerun or independently promoted to release evidence.

## Cleanup and final limits

All nine owned cargo processes exited successfully and were reaped; final owned
PID checks are empty. Tests used GPUI TestAppContext, local saved corpus and local
Jobs threads, not an ordinary native app/backend or external browser worker.
No background verification process remains.

**Native five-screen rendering, focus/selected/disabled appearance, 820×600 layout,
actual keyboard/drag/IME, native create/save/reopen and viewer/history visual
inspection remain NOT RUN after the prior black-pixel gate.** No new screenshot
was produced, no native second round ran, and no human hierarchy/blandness judgment
is invented. The previously captured all-black owned X11/llvmpipe PNG remains the
actual blocker evidence for its original binary; the corrected binary above was
hashed but never natively executed. The user's successful plain launch/create report is not
explained or contradicted by this correction run.

No hardware/default-Wayland, cross-platform, OS-scaling, screen-reader,
accessibility, human acceptance or release pass. **Close the two code-level
findings as verified repairs; keep native visual/input acceptance open.**
