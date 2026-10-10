# Phase 5 — bounded approval-context and tile-seam corrections

Date: 2026-10-10. Implementation of **only the two P2 findings** in
`05-verification.md`. Independent correction reverification remains **PENDING**;
the integrated gate is not independently accepted. No Phase 6, nested agents,
native-window retries, human visual approval or public-runtime qualification.
Historical reports, independent corpus and saved originals remain retained.

Evidence root (`E`):
`/tmp/opencode/regressee-p5-corrections-ad9c05076b12/`.

## 1. Confirmation is bound to the judgment that was previewed

- `crates/storage/src/review.rs`: `ApprovalContext` binds project ID/revision,
  run, active parent baseline, immutable analysis ID, displayed-analysis revision
  and exact selected page/profile keys. Preview also retains the exact resulting
  selected/inherited references and requested/final source provenance.
- `crates/storage/migrations/007-approval-context.sql`: additive v6→v7 display
  revision, with no rewrite of analysis/run/baseline JSON or CAS artifacts.
  Analysis display changes and `review::analyze_run` starts invalidate prior
  previews, including a cache hit that returns the same immutable analysis ID.
- `accept_selected_in_context` rejects a different expected context, stale
  display/reanalysis generation, changed project/active baseline, altered target
  references/coverage/provenance, failed/noncomparable captures and existing
  material/rendering/integrity failures. Within the immediate transaction it
  checks project revision, active baseline and displayed analysis/generation again
  before inserting/pinning/activating the **original explicitly confirmed** preview.
  Recomputing compatibility is validation, not a new confirmation or substitution.
  The existing `accept_selected(preview, revision)` compatibility entry point uses
  these same guards. Restore and immutable history remain unchanged.
- `crates/app/src/main.rs`, `jobs.rs`: the live project/run/baseline/analysis and
  selected-key set must match before confirmation dispatch. Selection, target,
  analysis, project/evidence context changes, scheduled reanalysis and completion
  invalidate the preview. A UI generation also refuses an approval reply that
  arrived after invalidation. History switching cancels superseded reanalysis;
  late analysis replies cannot replace a different run's displayed result.
  Confirmation sends the bound context to the actual backend, not just a button
  visibility decision. A fresh matching preview is always explicit.

### Assertions

- `phase5_approval_actions_refuse_old_preview_after_analysis_switch`: real GPUI
  `activate_control` sequence, actual Jobs replies and SQLite storage:
  old valid preview → View previous analysis (AllMasked/NotComparable) → confirm
  creates **zero** baselines and does not activate anything. Switching back and
  explicitly previewing/confirming creates exactly one baseline from the correct
  saved check run. Also tests selection invalidation, delayed approval after
  scheduling, actual asynchronous reanalysis start/completion, a queued stale
  confirmation and changed project revision. Synthetic capture metadata with
  actual saved corpus PNGs; not browser/native visual evidence.
- `approval_context_refuses_switch_reanalysis_selection_and_stale_baseline`:
  direct backend refusal after A→B→A, same-ID cached reanalysis, analysis-start
  invalidation, wrong expected selection/run/project, stale active parent and
  restore/revision changes. No baseline is inserted by refusals; a fresh selective
  acceptance inherits the exact unselected reference and preserves original bytes.
- Existing corpus/cache/masks/review, failed/material/rendering/missing-artifact,
  composite provenance/transaction/collision/recheck/restore guards remain tested.
  A new v6 display-row migration assertion preserves original record bytes and
  initializes revision 1; the retained v5 migration assertion now checks v7.
- `actual_phase5_product_analysis_composite_and_replacement`: actual signed
  Chromium originals now additionally test old preview → reanalysis switch →
  backend confirmation refusal with **zero** added baseline records/activation,
  followed by a fresh valid selective preview/confirmation and exact recheck.
  `phase5-report.json` records this refusal explicitly.

## 2. Retained saved images and pointer input share one transform

- `crates/app/src/review.rs`: `ViewportTransform` is the canonical
  `viewport = (source − pan) × zoom` transform and inverse pointer mapping.
- `main.rs::viewer_panes`: every before/after/difference/heatmap tile has an
  actual absolute source origin `(0, (ViewerFrame.tile + i) × TILE_HEIGHT)`.
  Retained frames therefore remain anchored during forward/backward seam crossing
  and delayed refetch, instead of jumping by `pan_y % 512`. Tile width/height come
  from the decoded saved image, including short final tiles. Masks use the same
  forward transform; pointer drawing uses its inverse. Horizontal pan is included
  in all layers, overlay and both pointer panes.
- The unchanged Jobs owner still loads at most three saved 512-row tiles per
  source; queue capacities, image limits, original CAS artifacts and GPU-image
  retirement remain bounded. No original is uploaded as a tall texture.

### Assertions

- `phase5_saved_pixels_seams_actual_layout_pan_and_mask_actions`: a saved
  96×1137 PNG encodes x/y in each pixel. All **109,152** actual decoded BGRA upload
  pixels are checked independently; the final tile has exactly 113 rows.
- **96 actual simulated-GPUI pointer-pan cases**: positions 511/512/513 and
  1023/1024/1025, zoom 0.5/1/2/4, retained frames starting at tile 0 or 1,
  forwards/backwards drags and nonzero horizontal pan. The old frame is deliberately
  retained during each drag, before any release/refetch.
- All four display modes measure actual GPUI tile-container bounds, not only
  `ViewerFrame` metadata or a computed offset. Saved-buffer pixels resolved through
  those bounds land at the same intended source coordinate in both panes and
  overlay/difference/heatmap layers. Actual pointer down/move/up on each pane draws
  rectangle **(21, pan_y + 8, 4, 4)**; measured mask bounds agree on both panes.
  The short final tile is not stretched, and original CAS bytes remain identical.
- The retained actual-corpus GPUI draw/redraw/delete/shared zoom/pan test also
  remains. These are saved-pixel and simulated GPUI layout/input proofs,
  **not native GPU framebuffer screenshots or human visual approval**.

## Final settled checks

**Final C PASS. All suites and cleanup are settled; no running test is handed off.**
`E/run-serial.py` runs the focused suites, then the unchanged maintained serial
wrapper, with a Linux subreaper and pinned Rust PATH. `E/final-c/serial-commands.json`
records exits 0, 0, 0; the maintained stage took **504.967 s**. Its five stages
all exit 0 in `E/final-c/maintained/commands.json`:

| Final C check | Actual settled result |
| --- | --- |
| Focused storage/service | **5 PASS**, 2.067 s including build; complete retained analysis/review suite plus the context regression |
| Focused simulated GPUI | **3 PASS**, 3.919 s including build; real actor confirmation/async sequence, retained pointer test and 96 seam cases |
| Locked UI build/test/clippy/fmt | Exit 0, **11.978 s**; **114 Rust PASS** (13+9+5+39+15+5+11+17), 12 explicitly ignored, not counted as passes; strict all-target/all-feature clippy and format PASS |
| Signed auth/crawl/resource safety | Exit 0, **374.346 s**; **one actual Rust Product flow + 33 signed Node PASS**. Retained origin-drift/restart/Forget, scope/resource and cancellation controls pass; all ten resource cases pass. Explicit injected fixture key, not OS-key qualification |
| Corrected signed Phase 5 Product | Exit 0, **118.222 s**; **one actual Rust Product flow PASS**, including stale-preview refusal, explicit valid selective acceptance/provenance, exact selected/inherited recheck, immutable history and restore. **One additional signed Node test PASS**, eight replacement/normal/readiness captures |
| Read-only independent corpus | Exit 0, **0.114 s**; retained 13 cases, 26 PNGs/digests, 78 CRCs, 576,512 decoded pixels, 1,737,266 encoded bytes and seven corruption checks; no regeneration |
| Final format | Exit 0, **0.264 s**. Existing `proc-macro-error2 2.0.1` future-incompatibility warning is retained, not a release qualification |

Read `E/final-c/maintained/signed-safety/product-flow/{phase4-report.json,
forget-report.json,resources/report.json,signed-worker-suite.log}` and
`phase5-product/{phase5-report.json,replacement-product.json,
replacement-worker/report.json,signed-phase5-worker.log}` under that maintained
root. The corrected Product report explicitly records zero-baseline stale refusal.

### Preservation, freshness and cleanup

`python3 E/audit.py` → `E/final-audit.json` is a read-only implementer audit:

- **85 final source/lock/script inputs** remain identical to the final-C entry
  fingerprints; **64 protected historical report/corpus/auth inputs** and **845
  private historical engine-verifier files** remain identical. The prior integrated
  verifier's **134 saved original paths** still match their original CAS digests.
  Of its 187 entry inputs, only the eight intended implementation/test files and
  status index differ. New files are the additive migration, UI fixture and this
  correction report; no auth/worker/security or historical report rewrite.
- All **6,026 freshly signed-manifest files** match hashes/sizes; all **11 worker
  source/test/fixture files** still match maintained bytes. Runtime remains Node
  **22.20.0**, worker **4.0.0**, Playwright **1.56.1**, Chromium **141.0.7390.37**,
  development-owned-loopback only, not public/release qualification.
- Both fresh Product DBs pass SQLite integrity/foreign-key checks at v7. All
  **five distinct saved original files** pass digest/metadata/complete PNG CRC/
  inflation checks; **six baseline references** resolve to exact source captures,
  run/page/profile, inventory and capture date. The Phase 5 DB retains 2 projects,
  6 runs, 3 baselines, 4 analyses and one revision-specific review.
- Two encrypted session files remain RGS1, 0600 in 0700 vaults. Nonprivate DB rows
  and 11 logs exclude the scanned fixture cookie/session-reference sentinels;
  this is bounded sentinel evidence, not general screenshot confidentiality.
- `E/final-c/outer-result.json`: **146 adopted descendants reaped, zero remaining
  children**. The completion notification was awaited, with a scoped pidfd event
  wait rather than a sleep/poll loop. Audit confirms **zero live scoped processes,
  scratch entries or pending files**. Fresh fixture staging was cleaned by its
  owners; signed runtime/evidence/history remain retained. No unowned signaling,
  recursive history copy, broad deletion, global installation, sudo, security/
  TLS/sandbox bypass, public browsing/services or publication.

These are **implementer-local** outcomes. The two repairs still need a fresh
independent integration recheck; native/human/external gates are not passed.

## Preserved intermediate failures

- First serial attempt: focused services/GPUI and all **114 Rust tests PASS**,
  but strict clippy exits 101 for two redundant selector conversions and the new
  constant-size `chunks_exact` assertion. `E/maintained/commands.json` and log
  retain the failure; no Node/Product stage ran in that attempt.
- `E/final-b`: services PASS, GPUI compilation exits 101 because the clippy
  conversion to `as_chunks` requires dereferencing its `[u8; 4]` reference in the
  assertion. Corrected without changing the assertion's pixel expectation.
  Its logs/commands/ownership result remain retained. No suites overlapped.

## Independent-verifier commands and boundary

Use fresh output directories, the pinned Rust PATH and `TMPDIR=/tmp/opencode`:

```sh
env PATH=/path/to/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH \
  LIBRARY_PATH="$PWD/.runtime/link-libs" TMPDIR=/tmp/opencode \
  cargo test --locked -p regressee-app --features ui-tests --test analysis_review \
  -- --nocapture --test-threads=1

env PATH=/path/to/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH \
  LIBRARY_PATH="$PWD/.runtime/link-libs" TMPDIR=/tmp/opencode \
  cargo test --locked -p regressee-app --features ui-tests --bin regressee phase5_ \
  -- --nocapture --test-threads=1

env PATH=/path/to/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH \
  python3 scripts/check-phase5.py --output /tmp/opencode/regressee-p5-recheck-UNIQUE
```

Run serially with a bounded owner/subreaper and at least 30 minutes' outer deadline.
Inspect stale confirmation dispatch/refusal, analysis/selection/context changes,
actual saved-image layout versus pointer coordinates at seams in both directions,
and expected-context/active-parent/reference transaction guards independently.
Keep prior auth/session/scope/scratch/integrity/resource checks and immutable
history/provenance/restore behavior. Retain original failure reports/evidence.
Do not retry historical black native windows or infer visual approval from these
simulated tests. Native/human/accessibility, real OS-key, maximum-scale, Windows/
macOS/clean-machine, public-runtime/signing/enterprise/release gates remain pending.
The integrated gate still awaits independent correction reverification; Phase 6
is not started or authorized by this handoff.
