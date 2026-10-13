# Phase 2 implementation — evidence workspace and action explanations

**Implemented; verifier FIX repairs awaiting reverify.** The original independent
FIX is preserved in `phase2-verification.md`; no PASS is claimed. Only Phase 2 of
`docs/ux-improvement-plan.md` is implemented. Phase 1's PASS and pushed
`7b73090` record are preserved. No children, commits, pushes, user-project
access, public-site visits or Phase 3–5 implementation.

## Outcome and finding map

| Contract | Implementation / evidence |
|---|---|
| A2: small-window evidence height | The existing task-heading row now owns one stable inspected path/profile/Before/After environment heading. One workspace-wide 40px toolbar owns Show/Hide results, View, side switching, Zoom, Details and Analysis. No new Show results or repeated hidden-queue identity rows. Compact View and side controls remain in that row with navigation open. Ordinary pane captions are 18px. |
| A2: View menu | Uses the same extracted trigger-relative, below/above/window-clamped geometry as Zoom. Five full-width rows, selected `Current` mark, arrows/Home/End, Enter, Escape, Tab/Shift-Tab and outside dismissal. Actual current-frame trigger bounds, not viewer-pane bounds, anchor the popup. |
| B-03: blocked actions | One priority-ordered reason beside ordinary review actions at both widths: unapplied coordinates, unsaved areas, unsaved note, pending analysis, noncomparable evidence, exhausted matching queue. Copy names Analysis, Review note, Details or filters as recovery. Existing eligibility/navigation guards remain authoritative. |
| B-04: production Heatmap | The bounded production caption has labelled significant/ignored swatches and `Heatmap · derived, not defects`. Only Heatmap displays it. A saved source-pair mask is analyzed through production `save_mask`/`analyze_run`, not a painted fake heatmap. |
| Empty/selected eligibility ambiguity | Empty selection is `0 selected`; nonempty selection names `N selected · M ready to update`. Calculation and explicit baseline confirmation are unchanged. |
| Original evidence / occlusion | Saved ignored rectangles no longer tint ordinary originals or overpaint the derived heatmap; geometry overlay remains in the explicit Analysis/drawing context. View/Zoom occlusion prevents hidden review/baseline dispatch and removes disabled actions from traversal. |

Long paths and each profile/environment column truncate independently without
wrapping away evidence height. Full values remain in native hints and the existing
keyboard-accessible Details/provenance. Historical reference identity continues
to come from its inspected pair, never today's project settings. Recorded
inspection retains its explicit inspection-only heading. Missing/failed-image
fallbacks retain Show/Hide results without manufacturing usable image controls.

The persistent pan-help row becomes a native image-viewer hint; the existing
keyboard help and image keyboard/pointer/scroll behavior remain available. Drawing
keeps its explicit instruction. The inspector no longer reserves its old 48px
toolbar height internally; existing note/editor reachability tests pass. **No
multiline editor, note-limit change, Analysis hierarchy, History redesign or new
comparison algorithm is included.**

## Exact changed file inventory

Production:

- `crates/app/src/evidence_workspace.rs` (new): inspection heading, compact
  toolbar, attached View menu, selection copy and production Heatmap legend.
- `crates/app/src/ui/selector.rs` (new): shared native selector popup geometry.
- `crates/app/src/main.rs`: integration, styled shared-action entry point,
  selector focus/dismissal/occlusion guards, compact pane captions, navigation hint
  and editing-only geometry paint; removes the old detached chooser.
- `crates/app/src/workflow.rs`: stable heading/toolbar mounting, fallback queue
  recovery and precise selection label.
- `crates/app/src/review_ux.rs`: highest-priority blocking reason at both widths.
- `crates/app/src/viewer_ux.rs`: remove duplicate inspector toolbar reservation.
- `crates/app/src/ui/zoom_select.rs`: reuse selector geometry, expose read-only
  open state for shell safety; existing behavior/tests retained.
- `crates/app/src/ui/timezone_select.rs`: expose the existing drawn chevron for
  reuse, without changing timezone behavior.
- `crates/app/src/ui/mod.rs`: export selector helper.

Tests/evidence tooling:

- `crates/app/src/phase2_evidence_tests.rs` (new): four regression tests below.
- `crates/app/src/phase2_viewer_tests.rs` (new): selector occlusion/source-pan test.
- `crates/app/src/phase4_review_tests.rs`: persistent-identity containment now
  checks the workspace owner rather than its old viewer-only owner. This filename
  is an incumbent regression suite, not implementation of this plan's Phase 4.
- `crates/app/src/ui_phase2_capture.rs` (new): exact 20-state × two-size opt-in
  production-render recipe, actual activation/state/bounds/storage assertions.
- `crates/app/src/ui_renderer_capture.rs`: register recipe and extend every
  sidecar/proof with seven initial Phase 2 fingerprints plus the V2-1 drawing-input
  helper binding; original capture safety intact.
- `crates/app/src/ui_review_matrix.rs`: shared synthetic fixture optionally saves
  long historical environment names in actual frozen run records. Existing
  whole-app fixture continues using its original names.

Documentation: this report and Phase 2 status/execution row in
`docs/ux-improvement-plan.md`. `DESIGN.md`, core/storage/runtime algorithms and
later-phase source are not rewritten. Existing Phase 1 source/test module gates
and the parent's plan push-record change are retained.

## Measured native geometry

Definitive **same selected long-path Desktop target**, same saved analysis/mask,
same window, clean state. Bounds come from actual GPUI painted pane selectors.
Navigation changes use the mounted control; modes and queue visibility use their
supported activation routes. Each queue-hidden/full-width height is asserted
against its corresponding Compare pane; a native 180px floor is also asserted.

| Window / navigation | Compare `w×h` | Hidden Results `w×h` | Full Before / After `w×h` |
|---|---:|---:|---:|
| 820×600 / collapsed | 274×234 | 380×234 | 772×234 / 772×234 |
| 820×600 / open | 178×234 | 284×234 | 580×234 / 580×234 |
| 1280×800 / collapsed | 468×389 | 610×389 | 1232×389 / 1232×389 |
| 1280×800 / open | 372×389 | 514×389 | 1040×389 / 1040×389 |

**V2-2 correction record:** the original handoff incorrectly transcribed the
navigation-open hidden-Results widths as **274×234** and **468×389**. The original
source-bound `phase2-matrix.json` and PNG/JSON pairs are retained unchanged; their
actual measurements are **284×234** and **514×389**, independently confirmed in
`phase2-verification.md`. Only this table's transcription is corrected.

At 820×600 every clean pane begins at **y210** and the toolbar is **40px**.
The baseline assessment's default/hidden/full heights were 194/102/98px, with
navigation-open Compare 141px. Those historical numbers are not a same-target
experiment; the new within-source same-target assertions above are the acceptance
signal. The former full-width masthead-only letterbox is eliminated, not merely
traded for more width. Overlay retains its working opacity controls; Heatmap uses
an additional labelled caption line. Dirty/blocking states legitimately reserve
their recovery text and baseline-safety explanations.

Final View menu at 820×600: trigger `130,144,96,40`; popup `130,188,304,202`
(`x,y,w,h`). Rows are full-width inside the border. Zoom also asserts its attached
below/above gap and window bounds. Both menus remain display-only.

## Initial handoff tests and commands (pre-verifier correction)

Common environment from repository root:

```sh
export LIBRARY_PATH="$PWD/.runtime/link-libs"
export TMPDIR=/var/home/olav/Data/opencode/tmp/opencode
```

| Exact command | Initial handoff result |
|---|---|
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee -- --test-threads=1` | **135 passed, 0 failed, 19 ignored** |
| `cargo test --locked --offline -p regressee-app --all-features --lib ui:: -- --test-threads=1` | **43 passed**, including Zoom/timezone/input/shared controls |
| `cargo test --locked --offline -p regressee-app --features ui --bin regressee --no-run` | Exit 0; Phase 1's exact UI-only feature gate |
| `cargo check --locked --offline --workspace` | Exit 0 |
| `cargo build --locked --offline --workspace --features ui --bins -j 4` | Exit 0 |
| `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings` | Exit 0 |
| `cargo fmt --all -- --check` / `git diff --check` | Exit 0 |

These final regression/build logs are under the initial complete inspection root
below: `app-tests-final.log`, `ui-primitives.log`, `ui-only-no-run.log`, `check.log`,
`build.log`, `clippy.log`. The full app run includes Phase 2 **5**, incumbent review
**8**, options **4**, setup **4**, setup repair **5**, Phase 1 setup **4**, Phase 1
runtime **3**, and workflow **10** passing tests, plus approval, request/draft,
mask, loading and history regressions. No complete workspace test-suite pass is
claimed. Ignored captures/qualification tests are not silently called passes.
The existing `proc-macro-error2 v2.0.1` future-incompatibility notice remains.

New exact test names (under the modules listed above):

- `same_target_painted_height_and_toolbar_survive_queue_modes_navigation_and_long_identity`
- `view_menu_is_trigger_attached_full_row_and_native_keyboard_dismissable`
- `concise_blockers_are_painted_at_both_widths_without_relaxing_draft_guards`
- `selection_copy_names_ready_members_of_the_selection`
- `zoom_occlusion_and_image_keyboard_preserve_source_pan_across_views`

They test retained note/area drafts, storage unchanged after presentation,
historical identity versus today's settings, actual menu keyboard/pointer exits,
hidden-control rejection, zoom selection not dispatching shell approval, and a
single synchronized source transform retained through all five views/side switch.
All simulation modules remain gated by `test + ui-tests`, not merely `ui`.

## Initial handoff evidence and source binding (retained unchanged)

Private canonical **0700** root:

`/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase2-confirmation-20261013-x00dvmqr`

Contains **40 original app PNG/JSON pairs**, fresh `renderer-proof.{png,json}`,
personally inspected hash-bound `proof-reviewed.json`, `phase2-matrix.json`,
22-file `source-provenance.json`, `capture-integrity.json`, `inspection-ledger.json`,
ten labelled paired contact sheets, logs and fresh-session offline website
fixtures. No old app PNG was reused. Originals were not edited, tinted, resized
or corrected; contact sheets are labelled inspection derivatives.

Set the approved/root renderer variables and run these exact opt-in commands:

```sh
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::renderer_capture_proof -- --ignored --exact --test-threads=1
# Personally inspect proof before creating proof-reviewed.json.
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::ui_phase2_capture::renderer_capture_phase2 \
  -- --ignored --exact --test-threads=1
```

Both initial handoff commands exit 0 / 1 passed. The receipt rechecks all 40 app pairs
plus proof for PNG hashes/dimensions and **20 embedded current-source bindings per
frame**. All 22 manifest files matched the initial handoff source. The authorized
V2-1 correction below changes five of those files; this 40-frame batch is retained
as pre-correction evidence, not relabelled as current-source drawing proof.
Base commit: `7b730906cb752a397a0b5bc0d37a076c70a8432d`.

Key SHA-256 bindings:

```text
main.rs               9514b510b5bbe6f47349c63f8583e903d59377676b432a0c63dadffac17f11c5
workflow.rs           3a1ac139cdfa5024009c3f087370fc7edefe7646c5c39c650113f8d7479ef563
evidence_workspace.rs 98b34a14736c9c61edd698c41a5f72f03f47e2e44b8d47b0b1f4633582da8493
review_ux.rs          14d7ee8b30c4ad318f3d1481850283b0b25fb53c41309dd30b084949c7d56039
viewer_ux.rs          882ca4807da20c912d1e6fe813a9b60228b6f24060c36e654f9fb5b08af2b2a9
ui/selector.rs        5bb36ffd5c1fb00905234cfa58fb90b5bdde672bd134d38097e200c1a1fdd97f
ui/zoom_select.rs     6b686974eb53139fc759f3c378b6d304ec74373be10a2bfdf08a5be5dcc791a1
ui_phase2_capture.rs  88143a3393aa411687365ec60720840c05330e9c4494dc4c49fd129b193ee3eb
```

Source manifest SHA-256:
`a9365e7095a63ec039d8b7513f7697ac4c2b2adb1a1fbdc4eaff9dbd14e0d51b`.
Matrix SHA-256:
`a11977d4ebefec152fee6a0c12934591cfb12f49a5e182e88729341d9d3225f1`.

The saved ignored rectangle is `700,160,400,300` in original integer coordinates,
with **120,000 ignored pixels** and actual significant differences in the same
production analysis. Legend colors exactly match existing engine output:
significant `[220,50,70]`, ignored muted blue `[95,120,130]`; no algorithm or color
encoding change. Within the painted evidence panes only (excluding legend
swatches), native final Heatmaps contain respectively **9,951 / 51,548**
significant/ignored pixels at 1280×800 and **3,511 / 18,252** at 820×600. Fourteen native
samples inside the saved ignored rectangle match original website RGB values
(±1) in Compare/hidden/full/navigation states, and all original CAS hashes remain
unchanged. Storage evidence is unchanged after presentation/selection testing.

## Same-session verifier FIX correction — awaiting reverify

The original independent **FIX**, including both findings and its original evidence
references, remains unchanged in `phase2-verification.md`. The same implementer
repaired **V2-1 and V2-2 together in one authorized correction batch**. No independent
PASS, commit, push, child agent or Phase 3–5 work is claimed.

### V2-1: reachable drawing exit without a new toolbar row

While drawing, `evidence_workspace.rs` now replaces the two secondary disclosure
actions (Details and Analysis) with **Done drawing**, rather than appending it.
Show results, View, side switching and Zoom remain in the same **40px** row. Target,
profile, historical environments and Before/After identity remain in their existing
heading/caption. No clean-state layout or evidence-height reservation is added.

`main.rs` makes those two omitted disclosure controls **unmounted and disabled**
while drawing: they cannot be Tab stops or dispatch hidden changes. Done drawing
retains its actual enabled `Phase5(14)` control and existing geometry/current-pair
guards. Pointer/Enter completion restores the open Analysis editor; Escape retains
its existing close-and-focus-Analysis behavior, and the visible Analysis trigger
really reopens it. No safe exit, draft, save requirement or backend guard is removed.

New exact regression:
`phase2_evidence_tests::drawing_toolbar_full_width_exits_fit_mounted_bounds_and_preserve_pair_drafts`.
It activates navigation, hides Results, chooses full-width Before/After, opens
Analysis and starts drawing. **Eight combinations** cover both sides, both
navigation states and **820×600 / 1280×800**. It checks every active toolbar action's
actual GPUI bounds against its owner/window, actual mounting/enabling, hidden-action
rejection, pointer Done, real Tab/Enter Done, Escape/focus and actual Analysis
reopening. Real pane pointer drawing adds exactly `8,8,16,8` in source coordinates;
all prior rectangles, unsaved note, inspected/request capture-pair identity and
request counter remain attached and unchanged. Storage is unchanged.

The targeted opt-in recipe in `ui_phase2_capture.rs` reuses that same input/bounds
helper with the real Linux fonts/renderer. `ui_renderer_capture.rs` embeds the
helper's source hash too. These are the **five source files changed since the
initial handoff**; no later-phase source was modified for this correction.

### Targeted native confirmation and integrity

Fresh private **0700** correction root:

`/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase2-fix-20261013-kbx51xn2`

Contains **four fresh app PNG/JSON pairs**: navigation open, Results hidden,
full-width active drawing, Before and After at both sizes. Fresh renderer proof
was personally inspected before capture; all four original-resolution app PNGs
were personally inspected once. No whole matrix rerender or further polish round.
All five active toolbar actions fit; exact measured Done bounds are:

| Window / both sides | Toolbar `x,y,w,h` | Done drawing `x,y,w,h` | Dirty drawing pane `w×h` |
|---|---|---|---|
| 820×600 / navigation open | `216,144,580,40` | **`612,144,116,40`** | 580×165 |
| 1280×800 / navigation open | `216,152,1040,40` | **`786,152,116,40`** | 1040×316 |

These deliberately dirty drawing states retain both an unsaved note and area,
plus existing recovery/footer explanations and drawing instructions; their heights
are **not** substituted for the clean-state table above. The correction does not
add a chrome row or restore the clean full-width letterbox.

`phase2-drawing-fix.json` records actual bounds/state and successful pointer,
Tab/Enter, Escape/reopen, exact drawing, draft/pair and storage/CAS assertions.
`source-provenance.json` freezes **28 files**; each of the five fresh PNG/JSON pairs
(four app plus proof) has **21 embedded current-source bindings**. The standalone
receipt script `verify-evidence.py` rechecks source hashes, PNG hashes/dimensions,
bounds and private permissions. `inspection-ledger.json` records personal inspection.

Current source-manifest SHA-256:
`ed5559847acd80bbc4d9a0ff6495649f795ded8353d3e168ee15c62d4fbe84aa`.
Drawing matrix SHA-256:
`d6dc8c6ebe091d2da365894ede8f35d7f7e7ecfbc8a6228c79d8611c7c941b24`.

`pre-correction-files.json` and the receipt verify **90 retained file hashes**,
including all initial handoff PNG/JSON/proof/matrix files and the original FIX.
V2-2's original inaccurate widths and correct matrix bounds are also retained in
`v2-2-transcription-correction.json`; only the handoff transcription is changed.
The old 40-frame source manifest/sidecars remain pre-correction evidence, unedited
and unrelabeled. No application PNG reuse; `fixture-reuse.json` explicitly binds
unchanged offline synthetic **website-only** imagery. `website-fixtures.log`
preserves the unsuccessful new-fixture attempt (local worker lacks Playwright).

### Fresh focused gates

Common environment is unchanged. Exact commands and exit statuses are in
`focused-gates.json` / `run-gates.py` under the correction root. For test filters,
the common prefix is:

```sh
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  <filter> -- --test-threads=1
```

| Filter / gate | Fresh result / log |
|---|---|
| `phase2_` | **6 passed, 0 failed, 2 ignored** — `phase2-final.log` |
| `phase4_review_tests::` | **8 passed** — `phase4-review.log` |
| `workflow_refinement_tests::` | **18 passed** — `workflow-drafts.log` |
| `review_ux::` | **5 passed** — `review-ux.log` |
| `ui_sweep_dirty_masks_block_all_analysis_scheduling_and_stale_replies` | **1 passed** — `dirty-mask-stale.log` |
| `phase5_real_tile_pointer_mask_coordinates_and_synchronized_actions` | **1 passed** — `real-pointer-mask.log` |
| `phase5_saved_pixels_seams_actual_layout_pan_and_mask_actions` | **1 passed** — `saved-pixel-mask.log` |
| `local_compatible_evidence_and_history_refresh_preserve_ids_pan_zoom_and_dirty_masks` | **1 passed** — `compatible-refresh-drafts.log` |
| `local_disabled_windows_preserve_drafts_skip_validation_and_allow_reanalysis` | **1 passed** — `disabled-window-drafts.log` |
| `cargo test --locked --offline -p regressee-app --features ui --bin regressee --no-run` | Exit 0 — `ui-only-no-run.log` |
| `cargo test --locked --offline -p regressee-app --features ui-tests --bin regressee --no-run` | Exit 0 — `ui-tests-no-run.log` |
| `cargo check --locked --offline --workspace` | Exit 0 — `workspace-check.log` |
| `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings` | Exit 0 — `clippy.log` |
| `cargo fmt --all -- --check` / `git diff --check` | Exit 0 — `format.log` / `whitespace.log` |

**42 focused regression passes**, no failures; ignored capture recipes are not
counted as passes. Separately, fresh exact opt-in proof and targeted drawing capture
commands each pass **1 test**. Reproduce with the approved/root renderer variables
and a **new** private root; `commands.md` records the complete artifact setup.

```sh
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::renderer_capture_proof -- --ignored --exact --test-threads=1
# Personally inspect the original proof before writing proof-reviewed.json.
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::ui_phase2_capture::renderer_capture_phase2_drawing_fix \
  -- --ignored --exact --test-threads=1
```

Initial test-development failures are preserved, not promoted to final evidence:
two test-only borrow errors (`phase2-tests.log`), followed by an exact pointer
fixture still displaying its setup notice (`phase2-tests-retry.log`). The shared
helper now actually activates Dismiss feedback before the supported drawing
sequence; `drawing-regression.log` and all final focused gates pass. No production
UI edits or additional visual rounds followed the targeted confirmation.

The verifier's separate background workspace test was not interrupted; its
aggregate result is **not claimed** here. No fresh complete-workspace/full-app
test-suite result is substituted for these scoped gates. Existing future-
incompatibility and native-input/accessibility/release caveats remain.

## Bounded inspection, retained attempts and caveats

One planned complete native batch was personally inspected through ten paired
sheets, with relevant original-size spot checks. One collective repair restricted
mask editing paint, guarded Zoom-occluded decisions and preserved missing-image
queue recovery. **One** final confirmation inspected all 40 frames through its ten
paired sheets plus original-size 820×600 Heatmap, navigation-open Before, View
menu, dirty note and pending-analysis images. No additional polish loop followed.
That implementer inspection found no further clipping/overlap defect in its
captured states. The independent verifier subsequently confirmed V2-1 in an
uncaptured active-drawing combination; its FIX and the original pixels are retained.

Initial complete 40-frame root (historical pre-repair pixels):
`/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase2-inspection-20261013-ngdnmda5`.
Preserved earlier attempts:

- `regressee-ux-phase2-20261013-zs9lwavh`: fresh offline fixture generation,
  initial proof and structural regression history.
- `regressee-ux-phase2-batch-20261013-woint5zz`: proof; capture stopped before app
  pixels because extensionless CAS artifacts need format detection from bytes.
- `regressee-ux-phase2-render-20261013-q8h_avbu`: 22 partial app pairs; height floor
  caught an undismissed fixture setup notice, not a clean-review height result.
  Recipe now actually activates Dismiss before clean measurement. Three originals
  were inspected within the initial batch. These attempts were not overwritten,
  promoted to final evidence or used for additional UI-polishing iterations.

All scratch directories above are under the approved root. The final
`historical-evidence-intact.json` checks **242** original PNG/metadata pairs:
238 baseline-review images and four Phase 1 repair images, retaining their original
historical main-source bindings. No baseline or Phase 1 evidence was relabelled.

**Qualification boundary:** real Linux GPUI/Blade offscreen pixels and simulated
native control dispatch only. Real OS presentation/input, IME, clipboard delivery,
AT-SPI/screen readers, high-DPI/high-contrast, macOS/Windows, live network/login,
runtime distribution and release/usability qualification remain pending. Long
identity values require hints/Details rather than fully painting every character.
Pending/noncomparable evidence blocks review decisions; existing safe navigation
to another comparable unreviewed result remains allowed. This report is an
implementer handoff, not independent PASS or authorization to advance phases.
