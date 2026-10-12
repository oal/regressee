# Phase 4 independent re-verification — PASS

**2026-10-12 · base HEAD `4e48813` · uncommitted finite repair reviewed.**
All substantive Phase 4 gates clear **within source/controller/offscreen scope**.
V4-1, V4-2 and V4-3 are resolved; no remaining Phase 4 blocker was found.
The [initial FIX](phase4-verification.md) is preserved byte-for-byte as history,
not rewritten into a pass. This is not native-input, usability or release certification.

## Independently executed checks

| Check | Result |
|---|---|
| Locked offline all-features core/storage/app packages | **299 passed, 0 failed, 40 ignored** |
| App library / binary | 95 passed, 2 ignored / 110 passed, 15 ignored |
| Analysis / product-flow / core / storage | 7 / 1 / 51 / 35 passed |
| Separate serial `phase4_review_tests::` rerun | **8 passed, 0 failed, 0 ignored** |
| External required-behavior navigation probe against rebuilt core and actual app allowlists | **4 passed, 0 failed, exit 0** |
| `python3 scripts/verify-phase4-receipt.py` | PASS integrity only: 12 immutable archives, 182 original-root PNGs, 38 blocked pairs + 22 repair pairs |
| `cargo fmt --all -- --check`; `git diff --check` | Both exit 0 |

Commands used (no renderer rerun):

```sh
LIBRARY_PATH=.runtime/link-libs TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
  cargo test --locked --offline -p regressee-core -p regressee-storage \
    -p regressee-app --all-features
LIBRARY_PATH=.runtime/link-libs TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
  cargo test --locked --offline -p regressee-app --all-features --bin regressee \
    phase4_review_tests:: -- --test-threads=1
```

Independent logs under `/var/home/olav/Data/opencode/tmp/opencode/`:
`phase4-reverification-tests.log`, `phase4-reverification-targeted.log`,
`phase4-reverification-probe.log`; generated probe `.rs`/binary share the latter
stem. The historical Python probe was executed with only output-stem and stale
printed commentary substitutions; required assertions were unchanged. It uses
fresh core code and extracted actual workspace allowlist expressions. All four
ready/loading/failed membership probes pass. Historical failing artifacts remain.
Known vendored GPUI float fallback and proc-macro future-compatibility warnings
remain; ignored runtime/browser/OS tests were not promoted to passes.

## Finite defects and regression assessment

### V4-1 — readable actual inner note input — PASS

Reviewed `viewer_ux.rs::viewer_inspector`, the stretching field owners in
`ui/controls.rs`, and native input sizing/debug naming in `ui/text_input.rs`.
A finite inspector-body width now propagates through the intermediate flex owners.
The independently rerun test measures **`analysis_fields[9].rendered_bounds()`**,
not the label wrapper: at both 1280×800 and 820×600 its content width is at least
270px and wholly inside its scroll viewport. The uniquely scoped painted border
is at least 300px wide and exactly 40px high. The implementation's reported
300×20px content / 326×40px border is consistent with the inspected pixels;
the independent assertions enforce useful bounds, not those exact width values.
Both full-resolution note originals show readable nonempty horizontally scrolled
draft text. Home/End caret-range bounds stay inside the actual input; focused
editor, enabled Save/Return, disabled decisions, Escape focus restoration and
unchanged long draft all pass. Native clipboard/IME is not established by this test.

### V4-2 — restoration while evidence loads or fails — PASS

Reviewed both `workspace_control_mounted` initial-read allowlists, core busy
candidate membership, and app mounted/enabled/activation guards. Guided(53) is
allowed only as the appropriate saved-project Results presentation control;
this does not bypass the normal safety predicates or generally unlock mutations.
Real Shell regressions at both sizes exercise ready, tracked pending read,
failed reply, dismissed error retaining read-failed state, and authoritative empty,
including explicit wide collapse. Tab reaches the mounted/enabled toggle;
controller activation and separately simulated GPUI pointer dispatch reopen the
actual rail, then Hide restores local Show. Resize retains explicit preference.
Requests, target and draft stay unchanged; stored project/evidence equality passes.
Dialog, approval-modal, native-prompt and active-task boundaries reject background
restoration. External probes independently confirm the formerly failing branches.
Loading/failed/dismissed originals at both sizes visibly retain Show navigation.
Simulated GPUI events are not native OS input qualification.

### V4-3 — persistent inspected identity and historical environments — PASS

Reviewed `inspection_identity`, `pane_caption`, reference-frame identity checks,
and the header/pane owners in `main.rs`. With results and navigation hidden, the
evidence owner retains path/query/fragment, saved capture profile/CSS dimensions
and source Before/After environments; narrow Compare no longer suppresses its
environment captions. Source identity is not borrowed from today's project settings.
The non-root long-address regression checks full identity and named visible bounds
inside the viewer at both sizes for Compare, Difference, Overlay, Heatmap, and
both full-width sides, plus capture-only environment context. Manual zoom/pan,
selected target, baseline selection and retained draft survive these transitions.
All corresponding full-resolution repair originals show legible identity/context.
The longer customer-service test address is assertion evidence, not a claim that
the shorter repair screenshot fixture captured that exact address. Full URL
retrieval in Details/hover remains available; no target or source-pixel mutation.

## Other substantive Phase 4 gates

The [initial verification's gate assessment](phase4-verification.md#remaining-gate-assessment-bounded-sourceoffscreen-scope)
remains applicable, with its three FIX qualifications resolved above. Reviewed the
updated report/receipt, complete working-tree diff and new untracked regression
file; checked consent, baseline/draft safety and source binding rather than relying
on package totals alone.

- **Consent PASS:** original one/many/long and exact-tail evidence remains immutable.
  Singular/plural Replace/Keep consequences, source context, one scroll owner,
  pinned cancel/confirm and technical retrieval remain. Serial consent regression
  still reaches the last of 97 presentation-only references, keeps exact addresses,
  cancels without writes, and preserves the bounded mounted-row contract. This is
  not 97 real approvals or bulk-throughput qualification.
- **Inspection/ownership PASS:** default Compare, all named modes, untinted
  explicit full-width originals, shared transform semantics, fit/manual distinction,
  path-first rows, full URL retrieval, local rail/queue preference and modal-safe
  search restoration remain covered. Narrow viewports are still short and require
  pan/zoom; this pass does not assert whole-page readability or task success.
- **Occlusion/drafts PASS:** owned tools remove/disable consequential decisions;
  note Escape retains unsaved text; masks still require explicit Apply/Save.
  Local presentation tests compare stored project/evidence before and after.
- **Capture/empty PASS:** initial capture overview stays capped, explicit manual
  enlargement survives resize, and authoritative empty reserves no false queue.
- **Baseline/approval/privacy semantics PASS in reviewed scope:** the storage diff
  remains serde-default read-only preview environment metadata, not policy/schema
  or capture/ZIP behavior changes. Package regressions for stale previews,
  eligibility, retained immutable references, draft binding/failures, historical
  inspection, session/ZIP exclusion and shared-object deletion pass. No repair
  changed these semantic owners to make presentation easier.

## Evidence ledger and honest bindings

Read all **four repair contact sheets and all 22 full-resolution originals** in
one read-only inspection batch, under
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase4-20261012/repair-v4-complete`.
For each stem below, inspected both `all-1280x800-<stem>.png` and
`all-820x600-<stem>.png`:

`review-note`, `review-queue-hidden`, `review-difference`, `review-overlay`,
`review-heatmap`, `review-full-width-before`, `review-full-width-after`,
`capture-mobile`, `capture-loading`, `capture-read-failed`, `capture-read-dismissed`.

The integrity checker validates current hashes for **61 captured source files**
and **62 final source files**, all repair PNG/metadata pairs and matrix harness
bindings. Only `phase4_review_tests.rs` has documented post-capture assertion
strengthening; shipping source and capture harness match the captured hashes.
Blocked receipt is an exact hash-bound historical copy; prior FIX, failed/partial
attempt logs and old capture bindings remain honest historical evidence, not
claims of current-source rendering. Original archives and PNG roots remain intact.

The receipt/report's **PENDING independent re-verification** fields describe the
implementation handoff and are intentionally preserved; this separate dated
verdict and the plan checkpoint supply the subsequent acceptance. The checker
is an integrity tool, not the acceptance authority. The reported successful
renderer run (1 test, 22 pairs, 42.91s) was inspected, **not independently rerendered**.

No product edit, new render/polish loop, commit, push, delegation or Phase 5 work.
OS pointer/keyboard/focus, accessibility, clipboard/IME, scaling/localization,
Windows/macOS, PM usability, true bulk scale and release qualification remain
outside this gate; Phase 7 and earlier diagnostic-tail limitations are not waived.

**Next:** parent reviews and commits/pushes the verified Phase 4 work and its
verification documents, then starts Phase 5 under the plan. No further finite
Phase 4 correction is required by this re-verification.
