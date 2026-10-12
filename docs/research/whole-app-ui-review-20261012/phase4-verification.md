# Phase 4 independent verification — FIX

**2026-10-12 · base `4e48813` · current uncommitted implementation reviewed.**
Phase 4 does **not** meet its exit gate. Return the three finite defects below
to the sole implementation agent **`ses_ece6028deffex3GeZBVhFk37aQ`**. Do not
commit/push as verified, start Phase 5, or treat the bounded aesthetic cycle as
permission to accept inaccessible controls.

## Independent result

Read the full [plan](../../whole-app-ui-plan.md), [blueprint](workflow-blueprint.md),
[implementation report](phase4-report.md), complete working-tree diff, new
`phase4_review_tests.rs`, package/capture logs and [receipt](phase4-receipt.json).
Loaded existing PRODUCT/DESIGN/surface context; preserved the approved native
desktop Clear wayfinding direction. No product source edits or new renders.

| Check actually executed by this verifier | Result |
|---|---|
| Core/storage/app, locked offline all-features package tests | **295 passed, 0 failed, 40 ignored** |
| App library / binary | 95 passed, 2 ignored / 106 passed, 15 ignored |
| Analysis integration / product-flow | 7 passed / 1 passed, 13 ignored |
| Core / storage | 51 passed / 35 passed, 2 ignored |
| Independent navigation required-behavior probe | **1 passed, 3 failed**, test binary exit 101; confirms defect V4-2 |
| `python3 scripts/verify-phase4-receipt.py` | PASS **integrity only**: 12 immutable archives, 182 original-root PNGs unchanged since Phase 1, 38 pairs, 62 source bindings |
| `cargo fmt --all -- --check`; `git diff --check` | Both exit 0 |
| Phase 4 acceptance | **FIX**, not a conditional PASS |

Package command:

```sh
LIBRARY_PATH=.runtime/link-libs TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
  cargo test --locked --offline -p regressee-core -p regressee-storage \
    -p regressee-app --all-features
```

Independent package log:
`/var/home/olav/Data/opencode/tmp/opencode/phase4-independent-tests.log`.
Binary suite finished in 5.85s; app library 0.66s, analysis 0.64s, core 0.44s,
storage 2.17s. Known vendored GPUI float fallback and proc-macro future-compatibility
warnings remain. Ignored runtime/browser/OS tests were not silently run or passed.

Independent probe, retained outside product source:
`/var/home/olav/Data/opencode/tmp/opencode/phase4-independent-probe.py`, generated
`.rs`, binary and `.log`. Reproduce with `python3` on that path after building.
It compiles against the freshly built core rlib and copies the **unchanged actual
`return matches!` expressions** from the two app workspace allowlists, rather
than inventing a mock policy. Final results:

```text
ready_restore_is_a_core_candidate                  ok
loading_restore_must_be_a_core_candidate           FAILED
loading_workspace_must_mount_restore               FAILED
failed_read_workspace_must_mount_restore           FAILED
1 passed; 3 failed; 0 ignored; exit 101
```

An initial probe mistakenly inherited `FocusState::default().busy == true` for
its ready-state control and had four failures. Correcting the external probe to
explicit `busy: false` made that positive control pass. No app code was changed.
These are source/controller probes, **not native pointer or keyboard tests**.

## Finite correction handoff

### V4-1 — note field is functionally clipped at both sizes — BLOCKING

**Evidence:** individually inspected `all-1280x800-review-note.png` and
`all-820x600-review-note.png` in the receipt's confirmation root. The input border
is approximately **26px wide × 40px high**, despite a 360px owned inspector and
a nonempty release-owner draft. The text is entirely unreadable, not merely
ellipsized at a normal field boundary. Save/Return exist and draft storage is
preserved, but users cannot read/edit the note meaningfully.

**Owners:** `crates/app/src/viewer_ux.rs:169–180,256–265`
(`viewer_inspector` note/body/wrapper sizing), in conjunction with
`ui/controls.rs:599–609` (`field`) and `ui/text_input.rs:490–500,595–641`
(percentage-width native input/root). The field already has `w_full`; repeating
that on the outer label wrapper did not repair the inner layout.

**Required correction:** establish a finite, stretching available width through
the inspector scroll body, note wrapper, field and native input. Diagnose the
actual inner percent/flex sizing instead of passing the outer `field-Note` bounds.
Keep the 40px field, horizontal caret scrolling, native selection/clipboard/IME,
explicit Save, retained unsaved text and attached Return/Escape behavior.

**Required regression:** at 1280×800 and 820×600, render a nonempty long note;
assert `analysis_fields[9].read(cx).rendered_bounds()` has useful content width
and lies inside its own scroll viewport. Also measure the painted input border
or a uniquely scoped inner selector; do not use the shared `text-input` selector
or label wrapper alone. Confirm readable text, focus/caret reveal, Save/Return
access and unchanged draft after Escape. Preserve the existing blocked decisions.
Current renderer asserts only `field-Note`, Save and Return; it misses this defect.

### V4-2 — local navigation restoration disabled on loading/failed reads — BLOCKING

**Reproduction:** a saved project in narrow Results starts navigation collapsed.
Initial evidence is dirty with no latest run; while its read is pending, or after
that read fails, the header paints Show navigation but activation is rejected.
The same bug affects a wide workspace explicitly collapsed before these states.

**Owners and exact causes:**

1. `crates/app/src/workflow.rs:414–431::workspace_control_mounted` returns from
   both initial-loading and initial-unavailable allowlists without `Guided(53)`.
2. `crates/core/src/shell.rs:283–314::FocusState::controls` returns from the busy
   branch without that candidate. `can_activate` checks candidate membership.
3. `crates/app/src/main.rs:4914–4919::control_enabled` requires both predicates;
   the later `control_mounted` special case at 5236 cannot override either guard.

**Required correction:** include the presentation-only navigation toggle in the
appropriate saved-project Results loading/failed-read app allowlists and core
busy candidate branch. Retain modal/task/native-prompt boundaries and mutation
locks; do not broadly enable busy-screen actions or bypass `control_enabled`.

**Required regression:** real Shell mounted/enabled/Tab/pointer-dispatch tests
for initial pending read, failed read, dismissed error with
`evidence_read_failed` retained, and ready/empty states at both sizes; exercise
explicit wide collapse too. Show must reopen the actual rail and focus a mounted
local control, Hide must restore local Show, resize must preserve preference.
Assert no request dispatch, target change, evidence/baseline write or draft loss;
modals/tasks must still reject background restoration. The three independent
failure probes above must then pass. OS input remains separate qualification.

### V4-3 — persistent inspection identity disappears — Phase 4 contract FIX

**Evidence:** both individually inspected `review-queue-hidden` originals and all
four full-width originals show project/profile/state but **no selected target
path**. A common origin such as `https://northline.example` is not page identity.
The narrow hidden-queue Compare also shows **neither Before nor After environment**.
Opening Details retrieves the URL, but occludes evidence and blocks decisions;
that is not the blueprint's persistent target/context requirement (§ topology,
lines 40–41,61–66). This is not a request to expand technical provenance by default.

**Owners:** `crates/app/src/main.rs::viewer_section` and
`viewer_panes_layout:6786–6846`; `crates/app/src/workflow.rs:832–912::pane_caption`.
Captions intentionally use `capture_origin(final_url)` rather than target path,
and the `indices.len() == 1 || !narrow_workspace` conditional suppresses origin/
environment altogether for narrow side-by-side Compare. No alternative selected
target heading is added when the queue is removed. Existing hidden-queue captures
select `/`; source establishes this omission also for distinct non-root targets.

**Required correction:** provide a compact always-visible inspected-target
path/query/fragment plus screen-size identity in the evidence owner when the
queue is hidden (including full-width and capture inspection). Keep Before/After
environment identity readable in narrow Compare, even with navigation closed;
do not replace source environment with current project settings. Full URL remains
retrievable in Details/hover; preserve evidence geometry and action separation.

**Required regression:** select a non-root long path with query/fragment and
different source/check environments; hide queue and rail and switch all modes/
full-width sides at both sizes. Assert the selected identity and appropriate
environment labels are visibly contained and survive layout changes without
rebinding target, resetting shared transforms or changing drafts/selection.
Add these named identity assertions; current capture geometry alone cannot pass
this gate. No extra branding or broad redesign is needed.

## Remaining gate assessment (bounded source/offscreen scope)

| Gate | Independent substantive assessment |
|---|---|
| Consequential consent, one/many/long | **PASS in exercised scope.** Singular/12-item plural, Replace/Keep, inspected role/profile/environment/UTC and future-baseline consequences are legible. One scroll owner; pinned cancel/confirm. Last long Keep rows contain title/profile/time with separators below text at both sizes. Exact requested/final URLs, run/artifact and final IDs are readable in exact-tail originals. Partial first row at an ordinary scroll boundary is not a separator-through-text defect. |
| All reference access / modal safety | **PASS controller/source scope.** Selected-first ordering, 32-row mounting bound and Previous/Next pagination preserve access. Rerun long-row test reaches presentation-only reference 96 of 97 and cancels without writes; real native many fixture uses 12 stored references, not 97 real approvals. Core/app focus trap, cancellation/restoration, background mutation and stale-preview/commit checks pass. No 100–1,000-page bulk throughput claim. |
| Compare, all modes, explicit full-width | **PASS transforms/mode access; identity FIX V4-3.** Default Compare retained at both sizes. Chooser exposes all five named modes within the window, with arrows/Home/End/Enter/Escape handling. Before/After is untinted and explicit. `viewer_pane_indices` selects originals; all panes use one `viewer_transform` with shared zoom/pan. Manual 75%/source anchor, target, note and baseline selection survive tested switches. Fit recomputes available width; manual scale survives resize. Full-width narrow remains a short viewport needing pan, not whole-page readability or usability proof. |
| Local queue/rail ownership, preference, focus, Ctrl/Cmd+F | **PASS ready-state behavior; FIX V4-2/V4-3.** Local Hide/Show controls replace detached global toggle. Narrow rail starts closed, queue open. Explicit rail preference survives resize. Hidden queue controls are unmounted; search restores queue before focusing mounted search and respects modal boundaries. Unique new handle slots do not collide with task links. Preference is in-memory workspace/window presentation, not persisted settings. |
| Path-first rows and full URL retrieval | **PASS open-queue scope.** 84px results have separate path/query/fragment, profile and status lines. HTTP(S) parsing keeps query/fragment and non-web fallback; original full URL is retained for hint and selected Details. Discovery/History remain 64px. Truncation does not erase retrieval, but cannot substitute for hidden-queue identity (V4-3). |
| Editor/disclosure occlusion and decision safety | **PASS blocking/preservation; note usability FIX V4-1.** Notes/Details/Analysis/filter panels have owned bounded scrolling and attached Return/Close. Review and baseline decisions are removed or disabled during occlusion; mode chooser disables them too. Tests check disabled activation, not paint alone. Escape preserves unsaved note; mask drawing folds owned tools and requires explicit Apply/Save. Existing coordinate reveal/draft guards pass. |
| Capture-only initial overview and empty queue | **PASS exercised scope.** Mobile capture originals show Fit 100%, one original, no comparison modes/Analysis. Controller test permits manual 150% and preserves it through resize. Authoritative no-run empty originals on sheets reserve no queue/collapse action and do not falsely claim missing evidence during a read. Valid stored CaptureRuns require nonempty targets; no claim of a storage-backed zero-target run. |
| Approval semantics, drafts, privacy | **PASS reviewed regression scope.** Storage diff adds only serde-default read-only source-environment preview metadata. No acceptance policy/schema/capture/ZIP changes. Rerun tests cover missing/incompatible evidence, stale selection/analysis/baseline, immutable composites/retained versions, note/mask explicit bindings and failures, historical read-only inspection, ZIP/session exclusion and shared-object deletion. Layout/filter tests compare stored project/evidence before/after; synthetic 97-row clones are never saved. Source tests are not privacy deployment certification. |

## Evidence ledger and limits

Read **all eight existing confirmation contact sheets** (covering all 38 pairs)
and **20 full-resolution originals**, one read-only inspection batch. For each
of the following ten stems, read both `all-1280x800-<stem>.png` and
`all-820x600-<stem>.png`:

`review-note`, `review-approval-confirm`, `review-approval-many`,
`review-approval-last-row`, `review-approval-exact-tail`,
`review-full-width-before`, `review-full-width-after`, `review-mode-chooser`,
`review-queue-hidden`, `capture-mobile`.

Root: `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase4-20261012/confirmation`.
Receipt establishes current shipping source binding with the explicit post-capture
Zoom **unit-assertion-only** substitution. Original archives/evidence remain
unchanged. The reported renderer run (not rerun here) passed one test/38 pairs
in 73.55s. Its outer-bounds assertions are **not** proof of inner note readability.

No renderer/polish loop, product edit, browser visit/download, commit, push,
delegation or Phase 5 work. Real OS pointer/keyboard/focus, clipboard/IME, native
accessibility, scaling/localization, Windows/macOS, PM usability, true bulk scale,
advanced/diagnostic tails, runtime distribution and release remain **not tested**
or previously unresolved. No new heuristic score or task-success claim.

**Next:** same implementation agent corrects V4-1, V4-2 and V4-3 as one finite
functional follow-up, adds the named regressions and narrowly source-bound evidence,
then returns for independent verification. Preserve this FIX and existing receipt;
do not overwrite blocked captures or relabel the prior cycle as accepted.
