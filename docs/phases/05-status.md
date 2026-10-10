# Phase 5 — local analysis and immutable review implementation

Date: 2026-10-10. **Implemented locally; corrected-source serial checks PASS;
independent verification pending.** All owned tests are settled; no live suite is
handed off. This is not native/human/accessibility, real OS-key,
cross-platform, public-runtime or release acceptance. The non-developer completion
gate is **NOT QUALIFIED**. Phase 6 is not started; no nested agents were used.

Phase 4's independently verified origin/session/Forget repairs and Phase 3's
document/resource boundary remain prerequisites, not replaced by this work.

## Delivered behavior

| Area | Maintained source and behavior |
| --- | --- |
| Actual analysis | `crates/core/src/analysis_engine.rs`, `crates/app/src/review.rs`: the audited row-streamed sRGB/D65/ΔE00 engine is now wired after guided and legacy checks. Exact local windows, four-neighbor components, raw/significant/structural pixels, dimensions, eligibility and explicit gates remain separate. Failed capture is CouldNotCapture; failed analysis, stale masks, rendering/readiness caution and all-area masking cannot become clean. |
| Engine integration | The original app module now re-exports the core module to avoid a storage dependency cycle. Bounded row-observer plumbing generates heatmap pixels from the exact significance rows; no second RGB approximation replaces the actual metrics. The engine's earlier independent verdict applies to its historical standalone contract, **not this integration**. |
| Revisions and cache | `crates/storage/migrations/006-review.sql` permits multiple immutable analyses per run while retaining old JSON/RGB policy records. Typed perceptual evidence contains the actual complete policy, report/failure and cache identity. Cache keys include exact source captures, baseline, material/rendering identity and policy/mask revisions; displayed selection is separate from history. |
| GPUI viewer | `crates/app/src/main.rs`, `jobs.rs`: actual saved tiled originals, highlighted-after difference, before/after overlay with pointer slider, heatmap, shared zoom/pan and largest-region navigation. Decoding/I/O/analysis stay on the bounded owner; policy edits debounce for 300 ms and cancel superseded reanalysis. Metrics explain units, ignored coverage, missing bands and omitted regions. |
| Ignored areas | Real pointer draw on either image, select/redraw/delete, visible on both. Immutable page/profile masks bind exact capture pairs/dimensions. A new pair or changed geometry requires explicit inspect/revalidate; masks are not stretched or attached to moving DOM content. Overlapping coverage is a coordinate union. All masked means undefined 0/0 and NotComparable, never unchanged. |
| Capture mode | `worker/product/src/capture.mjs`: pre-capture Normal or `neutral-raster-v2`, representative preview, raster/picture/simple-background transformation, measured light-DOM geometry and explicit scope warnings. Preview cannot activate as a baseline. Normal/replacement/algorithm changes require a new baseline; replacement-only captures never separately save hidden original photos. |
| Replacement diagnostics | Exact known HTML image URLs whose reload starts during replacement and fails with Chromium `net::ERR_ABORTED` are counted separately in metadata/UI. Earlier readiness failures, missing images, HTTP errors, scope denials and unrelated failures remain ResourceFailed/readiness warnings. Source-dependent geometry and unsupported media cannot silently become trustworthy comparisons. Material placeholder/source/layout rules and algorithm identity are retained. |
| Review decisions | Revision-specific unreviewed/intentional/needs-fix states and optional bounded notes, independent of metric classification. Reanalysis does not migrate or erase old decisions. |
| Approval/history | `crates/storage/src/review.rs`, `capture.rs`: exact selected/inherited target references, impact/provenance preview, transactionally pinned composite baseline and optimistic project revision guard. Acceptance recomputes the preview and refuses foreign-project/stale/corrupt/failed/missing/noncomparable/material/rendering targets. Intentional site URL mapping alone remains eligible. Whole new-baseline activation and historical restore remain explicit actions; original runs/baselines/analyses stay immutable. |

Exact presets, 13-case annotation, 39 product-policy comparisons and limitations:
`docs/research/phase5-presets.md`. These are **fixture-calibrated-v1**, not measured
real-site defect accuracy. Surface/finish disposition:
`docs/analysis-and-review-surface.md`; the existing proposed visual system is
extended without claiming new human approval or shipping decorative assets.

## Evidence

Evidence root (`E`):
`/tmp/opencode/regressee-phase5-sol-dae197dbb085/`.
Commands use locked dependencies, four Rust build jobs, one test thread and fresh
development-signed literal-loopback fixtures. No tool/dependency installation, public browsing,
publication, sandbox/TLS bypass or broad cleanup was performed.

Final corrected-source serial run: **PASS**, `E/final-c/commands.json`, all five
stages exit 0. The wrapper is `scripts/check-phase5.py`; no suites overlapped.

| Final C evidence | Exact result |
| --- | --- |
| `locked-ui-build-test-clippy-fmt.log` | PASS, 12.282 s: UI build, format, **110 Rust tests** (13+7+4+39+15+5+11+16), zero failures; all-target/all-feature clippy with `-D warnings`. Twelve explicitly opt-in tests are ignored in this ordinary batch, not claimed as passed. Includes real saved-image simulated-GPUI pointer draw/redraw/delete/shared-coordinate actions, immutable review/cache/mask/approval guards and v5→v6 byte-preservation migration. |
| `signed-safety/{commands.json,signed-auth-capture.log}` | PASS, outer 380.894 s: **one actual Rust authenticated Product flow + 33 freshly signed Node tests**, zero failures/skips. Original origin-drift/Forget safety, mapped/restart/expiry capture, allowed resource loading, denied/automatic-document zero visits and IPC resource cancellation remain covered. Read `product-flow/{phase4-report.json,forget-report.json,resources/report.json}`; all ten retained resource cases pass. Explicit injected-key fixture, **not real OS-key qualification**. |
| `signed-phase5-product.log` / `phase5-product/phase5-report.json` | PASS, 119.473 s: **one actual Rust Product flow**, changed Chromium PNG analysis, revision-bound note/reanalysis, one selected + one inherited reference, exact composite recheck (both NoSignificantChange), old run/baseline preservation and historical restore. Also exercises actual representative replacement preview, activation refusal and replacement-only baseline/check. |
| `phase5-product/{signed-phase5-worker.log,replacement-worker/report.json,replacement-product.json}` | **One signed Node browser test PASS**, zero failures/skips, eight actual captures: normal before/after differ; replacement before/after/repeat match with three transformed media and preserved measured geometry. Source-selector drift emits SourceDependent + GeometryChanged; unsupported media emits UnsupportedMedia; missing image still emits ImagesIncomplete + ResourceFailed. Supported picture transforms each record one canceled source reload, not a completed resource. Product replacement check is NoSignificantChange with matching immutable mode/version. |
| `readonly-independent-corpus.log` | PASS, 0.114 s: all 13 cases, 26 PNGs, 78 CRCs, 26 digests, 576,512 decoded pixels, 1,737,266 encoded bytes and seven corruption checks; parsed result exactly equals retained `validation.json`. Corpus was validated read-only, not regenerated. |
| `final-format.log` | PASS, 0.264 s. Existing Cargo future-incompatibility notice for `proc-macro-error2 2.0.1` remains; no dependency upgrade or release qualification is inferred. |

### Freshness, preservation and settled ownership

`E/final-c-entry-source-hashes.json` records 86 source/lock/script files before C;
`E/final-audit.json` confirms **all 86 remain identical** after completion. Only
documentation/evidence was finished afterward. Final signed runtime:
`final-c/signed-safety/product-flow/app/cache/product-runtime/runtime-62c9ccca-70cd-4522-8305-dafae2c6653f`.
Identity: Node **22.20.0**, worker **4.0.0**, Playwright **1.56.1**, Chromium
**141.0.7390.37**, **DevelopmentLoopback only**, not a qualified public runtime.
All eleven installed worker source/test/fixture files match both current maintained
bytes and the fresh signed manifest; the Product tests resolve/verify that pack.

The read-only `E/audit.py` → `E/{final-audit.json,final-audit.log}` passes:

- **133 protected historical corpus/research/author-evidence files + 845 private
  independent-verifier evidence files** retain their recorded hashes. The moved,
  row-observer-extended engine is a deliberate change, not claimed byte-identical
  to the historical standalone source. No whole-workspace preimplementation
  snapshot is claimed.
- Six saved baseline references resolve to the exact source run/page/profile and
  original hash; all five distinct saved original files match CAS and metadata
  hashes. Signed Phase 5 DB: 2 projects, 6 runs, 3 baselines, 4 analyses and 1
  revision-specific review. Persistent mask behavior is exercised separately by
  the four synthetic real-storage integration tests, not fabricated browser rows.
- **Zero remaining owned product test/Node/browser executable candidates, zero
  scratch entries and zero pending publications/runtime temporaries.** Final
  Phase 5 image staging cache is empty; safety cache contains only the retained
  signed runtime. No broad deletion or process signaling was needed.

### Preserved failures and intermediate evidence

- `final-a/commands.json`: locked UI checks exit 0 (16.088 s), signed safety exit 0
  (386.863 s), signed Phase 5 **exit 101** (12.781 s). The supported picture
  replacement initiated a source reload and canceled it with `net::ERR_ABORTED`,
  producing an unexpected ResourceFailed. Later corpus/fmt steps did not run.
- `diagnostic-a/results.log` retains the precise canceled image request. The
  focused `diagnostic-b/results.log` passes after narrow accounting; the maintained
  test also asserts missing-image ImagesIncomplete and ResourceFailed remain.
- Earlier arithmetic/workspace/clippy failures and successful corrections remain
  in `E/*.log`; no failed log is relabeled as a pass.
- `final-b/commands.json` is settled: locked UI checks exit 0 (10.779 s), signed
  safety exit 0 (384.008 s), signed Phase 5 exit 0 (121.027 s), corpus exit 0
  (0.114 s), final format **exit 1** (0.264 s). The approval owner-binding guard
  and regression were added after B's initial build; the last format check found
  only the new regression's unformatted assertion. Formatting was corrected and
  the entire final C suite rerun. B is not final-source UI/clippy evidence.
  Its exact owned runner was awaited using a Linux pidfd completion notification,
  without sleep/poll or overlap, before C started.

## Remaining qualification and limits

- **Independent integrated verification PENDING.** The standalone engine audit is
  not fresh product/storage/UI acceptance. Do not infer authorization for Phase 6.
- **Native GPUI pixels/input, human visual/usability and accessibility NOT RUN.**
  Saved-image simulated pointer tests are not native/human evidence; historical
  blank/unmapped attempts were not repeated.
- Synthetic arithmetic/raster/color controls are not real photographic/font-AA
  diversity or measured user false-positive/negative rates. Optional horizontal
  blend suppression can hide a real blend-color edit and is not a font classifier.
- Capture/analysis bounds are retained; maximum 64-million-pixel CPU/RSS and the
  historical 1,000-page/3,000-target soak remain **NOT RUN/UNQUALIFIED**.
- Replacement cannot guarantee geometry with source-dependent CSS/script;
  canvas/video/frames/shadow DOM/pseudo-elements/complex or extensionless
  backgrounds remain unsupported/unqualified. Geometry is measured only within
  the bounded light-DOM scope, not visual styling or nested content.
- Historical Phase 4 unavailable secure storage remains **not qualified**. This
  phase's safety suite uses its explicit injected-key fixture, not a fresh real
  keystore success. Enterprise proxy/OS trust/custom CA/VPN, Windows/macOS,
  clean supported machines and public runtime/signing/release remain pending.
- Cache/history UI lookups are bounded (128 analyses/baselines, 256 review events),
  region navigation retains the largest 16; omitted counts are reported and old
  database records are not deleted. No portability/retention/export work was added.

## Independent-verifier handoff

Use a **fresh unique** directory and an outer deadline of at least 30 minutes:

```sh
env PATH=/path/to/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH \
  python3 scripts/check-phase5.py --output /tmp/opencode/regressee-phase5-verifier-UNIQUE
```

Read all `commands.json` and completion logs, the fresh signed-safety auth/resource
reports, `phase5-product/{phase5-report.json,replacement-product.json}`, and
`phase5-product/replacement-worker/report.json`. Verify maintained worker bytes
against the fresh signed manifest, original PNG hashes, historical JSON preservation,
SQLite reference inheritance and displayed/review revision identities.

Add independent actual-service probes for cache/advanced disabling/AA, canceled
and failed analyses, exact heatmap membership, stale/overlapping/all-area masks,
mapped selective approvals, preview ownership/staleness/rollback, multi-profile
composites and recheck/restore. Exercise authoritative GPUI actor/pointer flows,
including the actual overlay slider; report native qualification separately.
Retain previous zero-request auth/session/Forget/redirect/resource and private
scratch tests. Do not regenerate the independent corpus, edit its historical
reports, weaken security boundaries, retry blank native windows, use public sites,
spawn nested agents or broadly delete/kill processes. Inspect and preserve any
new failure; scoped resource audit is read-only unless exact owned cleanup is
separately justified.
