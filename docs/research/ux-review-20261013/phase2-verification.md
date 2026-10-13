# Phase 2 independent verification — 2026-10-13

## Verdict: FIX

**Phase 2 only.** A2's clean-state evidence height, B-03's explanations, B-04's
production legend and the selected-reference wording meet their focused gates.
However, the new non-wrapping toolbar has an uncovered drawing-state overflow
regression (V2-1). Correct that and the handoff's two inaccurate width entries
(V2-2) together; do not advance or commit this phase yet.

No subagents, application/test source edits, commits, pushes, user-project access
or website visits were performed. The only repository addition by this verifier
is this report. The parent owns plan status and commit/push. Phase 1's pushed
`7b73090` remains the base; this is not a review of Phases 3–5.

## FIX batch

### V2-1 — P2: appended drawing exit overflows the minimum-window toolbar

**Location:** `crates/app/src/evidence_workspace.rs:115–123,215–245,251–284`;
shared non-shrinking actions in `crates/app/src/ui/controls.rs:42–61`.

**Reachable sequence:** at 820×600, open navigation, hide Results, choose
Before/After (full width), open Analysis, then activate Draw ignored area. Drawing
does not reset the mode (`crates/app/src/main.rs:1831–1837`). The toolbar retains
Show results, View, side switching, Zoom, Details and Analysis, then appends
**Done drawing**. Its owner is fixed at 40px high, with no wrapping, overflow
recovery or drawing-specific compaction; all shared action buttons have
`flex_shrink_0`.

**Evidence:** the current full-resolution
`phase2-820x600-navigation-before.png` and its painted matrix give a 580px
toolbar, x216–796. Existing controls reach approximately x759 (the Analysis
right-border pixel is x758). After the 4px gap, only about 33px remain. The added
label alone measures approximately 98px in 14px DejaVu Sans, before its 16px
horizontal padding and two borders. It therefore cannot fit in this row and
extends beyond the window. This is a **source/constraint-confirmed regression**,
not a claim that a drawing-state screenshot or exact Done drawing bounds were
captured. The provided 20-state recipe never enters active drawing, and existing
drawing tests exercise image geometry rather than this toolbar combination.
Private corroboration: `drawing-toolbar-source-proof.json` under the verifier
root below. Pillow font measurement is supporting evidence, not a substitute for
the requested actual GPUI bounds.

**Impact:** the visible pointer exit and its mounted focus stop can be offscreen
precisely in the compact full-width workspace. Escape remains a workaround; it
does not justify an unreachable rendered control.

**Acceptance:** compact or replace secondary actions while drawing, without
restoring the clean-state letterbox or losing side/target identity. Add an actual
mounted regression that activates this sequence and its After-side equivalent
at 820×600, both navigation states and 1280×800. Assert every active toolbar
action—including Done drawing—is inside its painted owner/window. Exercise
pointer and Tab/Enter completion plus Escape; retain exact pair-bound geometry,
unsaved drafts and restored Analysis access. A small source-bound confirmation
of the repaired combination is sufficient; do not regenerate the whole gallery.

### V2-2 — P3: correct the handoff's navigation-open hidden-pane widths

`docs/research/ux-review-20261013/phase2-implementation.md:83,86` states
274×234 and 468×389 for hidden Results with navigation open. The definitive
current-source matrix and full-resolution pixels instead give **284×234** and
**514×389**. Update those two entries; do not rewrite or relabel the evidence.
The height acceptance remains satisfied. The independently checked table below
is authoritative for this verification.

## Independent method and integrity

Loaded impeccable and ran context once; read the native audit reference and
existing surface brief. Applied the actual **desktop Operate / Clear wayfinding**
contract, not the known adaptive-to-mobile metadata routing. Read the complete
`docs/ux-improvement-plan.md`, A2, B-03, B-04, the selection ambiguity, synthesis,
handoff, full current diff, new production/tests/capture modules and relevant
incumbent controller/viewer/focus/approval source. No UI editing was undertaken.

Definitive supplied private root:

`/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase2-confirmation-20261013-x00dvmqr`

Personally inspected **all 40 original app PNGs at full resolution**, directly,
not only contact sheets: Compare, hidden queue, both full-width sides, the four
navigation-open variants, Difference, Overlay, Heatmap, View and Zoom menus, dirty
note/coordinates/areas, pending, noncomparable, exhausted and selected states at
both widths. Also inspected the renderer proof and two incumbent 820×600
letterbox captures from the original review. This was one bounded inspection
batch; no new renders or polishing loop were performed.

Independent Python/Pillow checks—not the implementer's integrity assertions—
verified all **40 PNG/JSON pairs plus proof**, PNG SHA-256/dimensions, **20 embedded
current-source bindings per frame**, all **22 source-manifest hashes**, a common
selected target across the matrix, and private-root mode 0700. Zero integrity
errors. Source manifest SHA-256:
`a9365e7095a63ec039d8b7513f7697ac4c2b2adb1a1fbdc4eaff9dbd14e0d51b`;
matrix SHA-256:
`a11977d4ebefec152fee6a0c12934591cfb12f49a5e182e88729341d9d3225f1`.

Verifier logs and independent receipts (0700 root):

`/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase2-verifier-20261013-wWmrf8fn`

Includes `independent-integrity.json`, `original-pixel-checks.json` and
`drawing-toolbar-source-proof.json`. No private pixels or databases are added to
the repository.

## Accepted evidence and safety gates

### A2: same-target painted panes, not estimated containers

These are the actual `phase5-before` / `phase5-after` pane bounds from the
source-bound native recipe, checked against full-size pixels. Each row compares
the same long Desktop target, saved mask/analysis and clean state.

| Window / navigation | Compare w×h | Results hidden w×h | Full Before / After w×h |
|---|---:|---:|---:|
| 820×600 / collapsed | 274×234 | 380×234 | 772×234 / 772×234 |
| 820×600 / open | 178×234 | **284×234** | 580×234 / 580×234 |
| 1280×800 / collapsed | 468×389 | 610×389 | 1232×389 / 1232×389 |
| 1280×800 / open | 372×389 | **514×389** | 1040×389 / 1040×389 |

All clean narrow panes begin at y210; all these toolbars remain 40px high.
Neither hiding Results nor either full-width side reduces evidence height.
Full-width originals now show document content rather than the former 98px
masthead letterbox. This clean-state acceptance does not excuse V2-1.

One heading retains path/query/fragment, source profile and separate Before/After
environment slots without wrapping away the evidence. Long values use deliberate
ellipsis, native hints and keyboard-accessible Details, rather than claiming
every character paints at minimum width. The displayed run/reference—not today's
project environment—supplies context. Fresh passing incumbent tests independently
exercise recorded-check read-only policy, source-only historical baseline
inspection, capture-only identity, stale replies and draft retention. The 40-frame
batch uses frozen recorded environment names; it is **not** a separate visual
qualification of every read-only or capture-only variant.

### View, Zoom, focus and occlusion

The View popup uses the current-frame trigger canvas, the shared below/above and
window-clamped selector geometry, and five full-width selected rows with Current.
At 820×600 its trigger is (130,144,96,40), popup (130,188,304,202). Fresh tests
exercise arrows/Home/End/Enter, Escape restoration, Tab continuation and outside
dismissal. Shift-Tab follows the same reverse shell traversal. Zoom's existing
passing tests also cover above-opening/clamping, pointer choice, blur, disabled
state and no shell activation. These conventional menus dismiss by selection,
trigger toggle or Escape; the removed extra View Close row is not a hidden focus
stop. Details/Analysis/note and filter panels retain their actual Return/Close
controls and Escape focus restoration in passing incumbent suites.

View removes occluded review/baseline actions from mounting. Zoom rejects them
through the authoritative enabled/dispatch guard; the new test attempts hidden
mutations and verifies request/storage stability. Compare, Difference, Overlay,
Heatmap and both full-width sides still share one source transform; fresh tests
exercise manual scale, synchronized pan, pointer zoom, resize fit/manual behavior,
side switching and untinted source navigation. No approval, discovery, capture,
baseline activation or deletion is introduced by presentation.

### B-03: visible reasons without a safety bypass

At both sizes the actual captures paint one ordered reason for unapplied
coordinates, unsaved areas, unsaved note, pending analysis, noncomparable evidence
or no matching unreviewed result. Recovery names Analysis, Review note, Details,
filters or result selection. Dirty drafts retain their target and reject Next;
pending/noncomparable evidence rejects disposition/baseline decisions. Existing
safe Next navigation to a different comparable unreviewed result remains allowed
for pending/noncomparable current evidence—the copy does not falsely say Next is
always blocked. Filtering keeps the inspected target and drafts. No backend or
eligibility guard was relaxed.

### B-04 and selection wording

Only the active production Heatmap displays **Heatmap · derived, not defects**,
**Significant differences** and **Ignored pixels**. Its colors match unchanged
production `review.rs` output: red [220,50,70], muted blue [95,120,130]. Independent
counts inside the painted image pane, excluding borders/caption/swatches:

| Window | Red significant pixels | Blue ignored pixels |
|---|---:|---:|
| 1280×800 | 9,951 | 51,548 |
| 820×600 | 3,511 | 18,252 |

The recipe saves a genuine capture-pair mask and calls production analysis for
120,000 ignored source pixels; it does not fake the derived image. Independent
sampling also matched **32 mask-interior original pixels** (±1 RGB) to the
synthetic website originals across Compare/hidden/full/navigation states. Saved
geometry paint is restricted to explicit editing; ordinary originals remain
untinted. Difference and Overlay retain their distinct treatments and no legend;
no engine/encoding change was made.

Empty selection paints **0 selected**; a selection paints **N selected · M ready
to update**. M still counts eligible members of that selection, not all targets.
ReviewIndex, eligibility, revision/provenance validation and explicit approval
confirmation are unchanged and their regressions pass.

## Commands and current results

From the repository root with
`LIBRARY_PATH="$PWD/.runtime/link-libs"` and
`TMPDIR=/var/home/olav/Data/opencode/tmp/opencode`. Logs are in the verifier root.

| Command | Independent result |
|---|---|
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee -- --test-threads=1` | **135 passed, 0 failed, 19 ignored** |
| `cargo test --locked --offline -p regressee-app --all-features --lib ui:: -- --test-threads=1` | **43 passed, 0 failed** |
| `cargo test --locked --offline -p regressee-app --features ui --bin regressee --no-run` | Exit 0 |
| `cargo test --locked --offline -p regressee-app --features ui-tests --bin regressee --no-run` | Exit 0 |
| `cargo check --locked --offline --workspace` | Exit 0 |
| `cargo build --locked --offline --workspace --features ui --bins -j 4` | Exit 0 |
| `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings` | Exit 0 |
| `cargo fmt --all -- --check` / `git diff --check` | Exit 0 |
| Additional `cargo test --locked --offline --workspace --all-features -- --test-threads=1` | **370 passed, 0 failed, 48 ignored** across 24 target/doc-test results; exit 0. Includes 7 analysis-review integration tests and the synthetic 100/1,000-page storage safety test. |

The app run includes all five new Phase 2 tests, all eight incumbent
`phase4_review_tests`, four options tests, viewer/zoom/ignored-area tests,
request/draft/exit and baseline-approval safety tests. A green suite does not
cover the missing V2-1 drawing-toolbar case. New simulation modules are gated by
`test + ui-tests`; the capture module is gated by `test + ui-renderer-capture +
Linux`, and that feature explicitly includes ui-tests. UI-only and simulation-only
no-run builds independently pass, not just all-features compilation.

The existing `proc-macro-error2 v2.0.1` future-incompatibility notice remains;
no new Clippy warnings were accepted. Ignored renderer/native tests were not
counted as successful tests.

## Qualification boundary

Current Linux GPUI/Blade offscreen pixels plus fresh simulated mounted-controller
tests only. No real OS keyboard/pointer/IME, clipboard delivery, native window
presentation, AT-SPI/screen-reader, high-DPI/high-contrast, macOS/Windows,
network/login/runtime distribution or usability/release acceptance is claimed.
V2-1's active drawing layout requires the explicit mounted/native confirmation
described above; this report does not fabricate that missing frame.

## Re-verification — 2026-10-13: PASS for the whole scoped Phase 2

**V2-1 and V2-2 are resolved. Phase 2 now receives independent PASS**, including
the prior A2, B-03, B-04, menu, selection and safety acceptance—not merely a PASS
for the drawing patch. No remaining in-scope FIX item was found. The original
230-line FIX above is preserved verbatim as historical evidence; its references
to “current” describe the original verification, not the corrected source.

Read the appended implementer handoff, current diff and corrected production/test
paths. Independently compared all original manifest hashes with current files:
exactly five source files changed—`evidence_workspace.rs`, `main.rs`,
`phase2_evidence_tests.rs`, `ui_phase2_capture.rs` and `ui_renderer_capture.rs`.
Only the first two change production behavior. No later-phase work is accepted
by this report. No application/test edits, subagents, commits or pushes were
performed; the only verifier repository edit remains this report. The parent
owns plan status, phase commit/push and advancement after that push. Phase 1's
pushed `7b73090` remains unchanged.

### Repair acceptance and independent reproduction

**V2-1:** `evidence_workspace.rs:251–289` replaces Details/Analysis with Done
drawing while drawing; it does not append a sixth control or reserve another row.
`main.rs:5711,5758–5763` removes the omitted controls from mounting, enabling,
focus traversal and authoritative dispatch. The existing draw/current-pair,
geometry, save, review and baseline guards remain intact.

Independently reran the complete app suite, including
`phase2_evidence_tests::drawing_toolbar_full_width_exits_fit_mounted_bounds_and_preserve_pair_drafts`.
Its **eight combinations** cover Before/After, navigation open/collapsed and both
window sizes. Inspected the assertions rather than relying on its name: all five
active actions have actual positive GPUI bounds inside their 40px owner/window,
are mounted/enabled, and the omitted disclosures reject activation. Actual
`simulate_click` on painted Done, dispatched Tab/Enter, Escape/focus restoration
and clicking restored Analysis all pass. Setup uses supported controller routes;
this is not a claim that every setup action was an OS click.

Also independently ran the exact real-font proof and targeted capture recipe in
a **new 0700 verifier-owned root**, using only byte-verified offline synthetic
website fixtures—not reused application PNGs. Personally inspected the fresh
proof before capture and all four resulting app PNGs at full resolution, in one
bounded repair-confirmation batch. Actual measured bounds match the supplied
correction matrix on **both sides**:

| Window / navigation open | Toolbar x,y,w,h | Done drawing x,y,w,h | Dirty drawing pane w×h |
|---|---|---|---|
| 820×600 | `216,144,580,40` | **`612,144,116,40`** | 580×165 |
| 1280×800 | `216,152,1040,40` | **`786,152,116,40`** | 1040×316 |

All five controls visibly fit without clipping/overlap. Target, source profile,
recorded Before/After environments and the active side caption remain attached;
long values still deliberately ellipsize. These are intentionally **dirty active
drawing** states with an unsaved note, area, recovery footer and drawing hint.
Their 165/316px heights are not substituted for the historical clean 234/389px
acceptance measurements.

Both independently executed recipes retain selected key, exact viewer request /
capture-pair binding, inspection identity, request counter and unsaved note.
Pointer drawing adds exactly **`8,8,16,8` source pixels**, retains prior rectangles
and dirty state, and Escape does not save/discard them. Storage evidence remains
unchanged; the native recipe also checks original CAS hashes. Completion restores
the Analysis editor, while Escape closes it, restores Analysis focus and permits
actual pointer reopening.

**V2-2:** handoff table lines 86/88 now correctly report **284×234** and
**514×389**. Independently matched both to the retained original matrix. The
original inaccurate transcription is explicitly documented; no old matrix,
sidecar or PNG was rewritten or relabelled.

### Whole-Phase-2 acceptance retained

| Gate | Re-verification basis |
|---|---|
| A2: clean evidence height and identity | The non-drawing toolbar, heading, pane/caption layout and sizing paths are unchanged by the repair. Fresh same-target mounted height/toolbar tests pass for both sizes, navigation states and hidden/full-width modes; incumbent recorded-check, baseline-only, capture-only and long-identity/draft tests pass. The original native clean table is retained as historical evidence, not claimed as newly captured. |
| View / Zoom menus and source navigation | Selector, Zoom, viewer and workflow source hashes are unchanged. Fresh menu, occlusion, synchronized pan/zoom, resize and original-image navigation regressions pass, including hidden-mutation rejection and Escape/Tab behavior. |
| B-03: blockers and recovery | Blocker source is unchanged. Fresh two-width mounted blocker tests and incumbent dirty-note/geometry/area, pending, noncomparable, stale-reply, filter and safe-navigation guards pass. |
| B-04: truthful derived Heatmap / untinted originals | Legend, engine encoding and ordinary-original paint paths are unchanged. The original independently counted red/blue pixels and untinted mask-interior samples remain historical acceptance evidence; the correction does not claim a fresh Heatmap capture. Viewer/source-transform regressions pass. |
| Selection and approval safety | Selection wording/calculation paths are unchanged. Fresh selection, exact revision/eligibility, approval-preview and explicit baseline-confirmation regressions pass. Presentation still does not visit websites or activate/delete baselines. |

### Integrity and fresh gates

Independent receipt checks verified the supplied **five PNG/JSON pairs**,
**21 embedded current-source bindings per frame**, all **28 current manifest
hashes**, actual owner/window bounds, common target and 0700 permissions. The
same checks pass for all five verifier-rendered pairs; their PNG hashes and
painted geometry reproduce the supplied results. The drawing matrices have
different synthetic IDs, so their hashes are not misrepresented as identical.
Checked all **90 pre-correction file hashes** before appending this section;
the original report is retained separately for byte-prefix verification.

Supplied current manifest SHA-256:
`ed5559847acd80bbc4d9a0ff6495649f795ded8353d3e168ee15c62d4fbe84aa`.
Independent drawing matrix SHA-256:
`189b69a34259f21240c9b79cf27b5b558e31023f7d36071a7ce3448bfe55e10b`.
The old 40-frame manifest/bindings are **pre-correction**, not current hashes.

Independent logs, receipt, inspection ledger and reproduction:

`/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase2-verifier-20261013-wWmrf8fn/reverification`

Fresh commands use the same `LIBRARY_PATH` / approved `TMPDIR` as above. Exact
gate commands and exit statuses are in `independent-gates.json`; native output is
under `native/` with both renderer root variables explicitly set.

| Fresh independent command / gate | Result |
|---|---|
| Full app/all-features binary tests, same command as above | **136 passed, 0 failed, 20 ignored**; `../app-tests-after-fix.log` |
| All-features `--lib ui::` tests | **43 passed, 0 failed** |
| Exact ignored `ui_renderer_capture::renderer_capture_proof` | **1 passed** |
| Exact ignored `ui_renderer_capture::ui_phase2_capture::renderer_capture_phase2_drawing_fix` | **1 passed**, including real-font mounted pointer/key/draft assertions |
| UI-only / ui-tests-only binary `--no-run` | Both exit 0 |
| `cargo check --locked --offline --workspace` | Exit 0 |
| `cargo build --locked --offline --workspace --features ui --bins -j 4` | Exit 0 |
| Workspace/all-targets/all-features Clippy with `-D warnings` | Exit 0 |
| `cargo fmt --all -- --check` / `git diff --check` | Exit 0 |

The app background result was read **only after its completion notification**;
no ongoing jobs were polled or interrupted. The earlier **370/0/48 workspace
aggregate remains pre-correction** and was not rerun or claimed current. Fresh
full-app/UI tests and workspace build/lint/feature gates cover this repair;
unchanged storage/engine acceptance is carried forward explicitly. Ignored tests
are not counted as passes except the two deliberately executed exact recipes.
The existing `proc-macro-error2 v2.0.1` future-incompatibility notice remains.

**Qualification boundary unchanged:** Linux GPUI/Blade offscreen pixels and
simulated mounted input, not real OS pointer/keyboard/IME or window presentation.
No accessibility, high-DPI/high-contrast, macOS/Windows, network/runtime delivery,
usability or release qualification is implied. This is the scoped whole-Phase-2
PASS required before the parent updates status, commits/pushes and starts Phase 3.
