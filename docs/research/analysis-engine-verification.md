# Independent standalone analysis-engine verification

SPDX-License-Identifier: MPL-2.0

## Verdict and scope

**PASS for the exercised standalone contract; no reproducible material engine
bug found. Not production acceptance, calibrated presets, or Phase 5 completion.**

Independently reviewed `crates/app/src/analysis_engine.rs`,
`docs/research/analysis-engine-handoff.md`, `PLAN.md` Section 10,
`docs/research/analysis-corpus.md`, and the independent corpus/seed proposal.
The module remains unexported/unwired: no Rust module declaration or integration
reference was found. Classification is numerical inspection evidence, not a
semantic defect verdict.

Only this document was written in the repository. No implementation, manifests,
locks, corpus, status, or existing reports/evidence were edited. No workspace,
app/runtime, browser, native, UI, or shared-runtime tests were run. No nested
agents, installation, network service, publication, desktop control, or cleanup
was used. Concurrent Phase 4 verification is outside this report.

Private evidence root: `/tmp/opencode/analysis-engine-verifier-zon_dhmw/`.
The earlier author harness `/tmp/opencode/analysis-engine-sol-ddxvnb3v/` was
read/copied, not executed in place or modified.

Verified source SHA-256:
`0c14f6557c5e5f4333d269f674264bd9fa99dc0dd44e54b4c782a5fd4deb533e`.

## Isolated execution and reproducibility

Used a standalone edition-2024 package pointing directly at the unwired source,
installed Rust 1.98.1 binaries, and the existing exact dependency versions:
`png = 0.17.16`, `serde = 1.0.228` with derive,
`serde_json = 1.0.145`, `anyhow = 1.0.100`. Copied the existing private vendor
closure and lock; offline source replacement points to the verifier's copy.
Cargo home, target, generated inputs, and final-run temporary root are private.

Exact final invocation, working directory the private evidence root:

```sh
export TC=/path/to/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin
export CARGO_HOME="$PWD/cargo-home" CARGO_TARGET_DIR="$PWD/target"
export TMPDIR="$PWD/tmp" TMP="$PWD/tmp" TEMP="$PWD/tmp"
export RUSTC="$TC/rustc" RUSTDOC="$TC/rustdoc"
export REGRESSEE_ANALYSIS_CORPUS=/path/to/Regressee/tests/fixtures/analysis-proposal
export REGRESSEE_PRIVATE_ZLIB="$PWD/independent-zlib"
"$TC/cargo" test --offline --locked --manifest-path "$PWD/Cargo.toml" \
  -- --nocapture --test-threads=1 > final-suite-03.log 2>&1
```

**25/25 tests passed, zero failures/ignored tests:** 19 colocated engine tests
and six private harness tests. These are isolated tests, not app integration.
The private additions are:

- `tests/private_oracles.rs`: two tests covering 346 public-entry cases and
  16 separately bit-written DEFLATE cases.
- `tests/bounded_resources.rs`: two tests covering tracked file-stream
  allocations/region retention and cancellation/preflight/read limits.
- `tests/independent_perceptual.rs`: one test covering analytic achromatic
  distances, additional published references, and independent angular oracles.
- `tests/independent_zlib.rs`: the copied retained control test, 80 encodings.

Private generators are `generate-probes.py`, `generate-region-stress.py`,
`generate-deflate-probes.py`, and `generate-ciede-reference.py`. To recreate
inputs, copy these scripts, the private tests/package/vendor closure, and the
retained `independent-zlib/` controls into a **fresh owned directory**, update
its source-replacement path, and run the four Python scripts in that order.
The first generator refuses an existing `probes/` directory; no corpus
generation or overwrite is needed. Then use the invocation above with that
directory's private paths. Seeds are deterministic (`0x61_2026` and `6101`).

Retained logs include `baseline-rerun.log`, `private-oracles.log`,
`bounded-resources.log`, `final-suite.log`, `final-suite-02.log`, and
`final-suite-03.log`. The second final-suite attempt failed because this
verifier transcribed one rounded reference as **0.4441 instead of 1.4441**.
The independent complex-phase equation oracle gives
`1.4441290780930238`; the engine gives `1.4441290780930247`. Only the private
reference was corrected; the failed log remains, and no engine change was made.
The successful third run executed the entire suite, not just the corrected test.

## Assertions and independently derived numerical evidence

The colocated assertions were inspected, not accepted merely from test names.
They check exact counts, geometry, binding errors, explicit classification and
trigger rules, strict/inclusive boundaries, color references, exhaustive BFS
agreement, clipped windows, stream termination, and specific corruption reasons.
Generic IO failure is not counted as proof of a CRC or zlib integrity check.

### Corpus rerun

- All **13 cases × four policies = 52 analyses** passed the raw/structural,
  masked/unmasked, denominator, region, and geometry assertions.
- All **39 seed interpretations** matched the explicit synthetic hypotheses;
  seed constructors also matched the proposal JSON. This is not calibration.
- Independently reran `python3 -B tests/fixtures/analysis-proposal/corpus.py
  validate`, redirecting stdout/stderr only to the private root. Parsed output
  exactly equals retained `validation.json`: 26 PNGs, 78 CRCs, 26 digests,
  576,512 decoded pixels, 1,737,266 encoded bytes, and seven corruption checks.
- The inherited 80 Python-encoded controls passed all five filters, compression
  levels 0/1/6/9 and default/fixed/Huffman/RLE strategies; stored, fixed and
  dynamic first blocks were observed. Each reports 65,536 eligible pixels and
  zero raw/significant change. The colocated encoder controls also passed.

### New public-entry oracles: 346/346

`probes/manifest.json` and `probes/public-probe-results.json` retain requests,
byte inputs, expected fields, actual reports/errors and pass status.
There were **219 expected successes and 127 expected rejections**.

1. **200 deterministic black/white geometry cases.** The Python oracle builds
   coordinate sets, mask unions, direct clipped windows and separate BFS
   components. It does not call engine helpers or reproduce its rolling-row,
   prefix-sum, or union-find implementation. Assertions cover all numerical
   report fields, dimensions and disjoint bands, raw and significant component
   summaries including ordered top-16/omitted counts, strongest windows and
   strongest qualifying windows, gates, warnings and triggers. Cases include
   142 differing widths, 75 crossed-dimension absent corners, 171 positive
   ignored areas, nine zero-eligible results, 177 enabled local policies, and
   72 cases with zero-eligible local windows. Local sizes/strides, thresholds,
   criterion disabling, overlaps and clipping vary.

   Concrete crossed sentinel `coordinate-001`: before **21×10**, after
   **19×30**, bounding canvas **630**, overlap **190**, before-only **20**,
   after-only **380**, absent-in-both **40**, coordinate union **590**.
   Mask union is **22**, eligible **568**, raw changes **485**, masked raw
   **21**, significant/unmasked **464/568**. Missing bands are `(19,0,2,10)`
   and `(0,10,19,20)`. The width gate yields `NotComparable`, preserving
   inspection metrics. Absent-in-both pixels never enter the denominator.

2. **Six height-limit icon analyses.** Two independent 8×8 icons on
   **64×32,768** pages, one at `(30,16380)` crossing seams and one at
   `(56,32760)` touching the bottom/right edges, tested under all seeds.
   Every report is exactly **64/2,097,152**, one 64-member region at the
   original coordinates, **8,192** clipped windows, `Changed`, and no global
   trigger. Seam local qualification counts are **4/1/0** for
   Sensitive/Balanced/Relaxed; corner counts are **4/4/3**. Connected-region
   criteria protect all six. Overlapping windows do not inflate changed area.

3. **Eight perceptual/alpha cases.** +1 gray drift on 16×16 images retains
   256 raw changes but zero significant changes for each seed. Optional blend
   suppression retains raw evidence while suppressing only the stationary
   interior blend; moving edges and an off-segment color edit are not suppressed.
   Alpha in a masked pixel still produces `NonOpaquePixels` and
   `NotComparable`, not an ordinary clean result.

4. **36 other explicit cases.** Bad CRCs in IHDR/sRGB/tEXt/IDAT/IEND,
   including post-image text; compressed tails in the same/separate IDAT and a
   second zlib stream; short/excess decoded payload; invalid filter; bad Adler
   with repaired outer CRC; missing stream termination; nonconsecutive IDAT;
   unsupported metadata/APNG/unknown chunks; invalid/oversized text; chunk count;
   IHDR pixel budget/overflow; mask endpoint/count/stale capture binding;
   crossed-canvas budget; material gate and rendering caution. Rejections assert
   specific error text and exclude `Cancelled`/`CaptureFailure` masquerading
   as corruption. Rendering mismatch alone preserves `NoSignificantChange`
   **with a caution warning**, never a confirmed defect or trustworthy release pass.

5. **96 repaired-CRC compressed-bit mutations.** A separate bounded Python
   zlib EOF/trailing-data/decoded-length/filter oracle predicts **three valid
   controls and 93 rejections**. Engine acceptance agrees on all 96. This is
   finite differential probing, not general fuzz qualification.

### Independent DEFLATE branches: 16/16

`generate-deflate-probes.py` writes bits/canonical codes without using the
engine's parser. `probes/deflate-manifest.json` and
`probes/deflate-probe-results.json` retain Python zlib controls and exact errors.

Fourteen rejection cases cover reserved literal/distance/HLIT values,
backreference before output, declared-window overflow, oversubscribed/incomplete
code-length and literal trees, incomplete distance tree, missing EOB,
repeat-without-previous, repeat overflow, and legal singleton EOB with too few
image bytes. Two complete controls pass: literal-only dynamic coding with an
empty distance alphabet, and a 256-byte backreference in a declared 256-byte
window. The 257-byte distance is rejected specifically by the engine's window
check (Python zlib itself accepts that stream); the oracle is the declared
window rule, not blind agreement with zlib. A short singleton-EOB stream is a
decoded-length error, not falsely rejected as an invalid tree.

### Independent perceptual math

- **1,024** grayscale pairs checked against an analytic achromatic lightness-only
  CIEDE2000 formula: maximum absolute error **5.900647423118244×10⁻⁶**,
  below the `0.0001` tolerance allowing rounded matrix chroma.
- **Nine additional** rounded Sharma/Wu/Dalal Lab references, distinct from the
  16 colocated vectors, pass within `0.00005`, including symmetry.
- **500** deterministic Lab pairs agree with a separate Python oracle using
  complex phase/mean-angle geometry rather than the Rust hue-wrap branches:
  maximum absolute error **3.552713678800501×10⁻¹³**, below `1e-10`.
- Inherited strict pixel `>` and inclusive global/local/region `>=` tests pass,
  including identical images at zero global threshold and insufficient local
  coverage. Metrics remain rational integer counts; threshold semantics are
  binary64, not a claim of exact decimal calibration.

## Bounded storage, regions, limits and cancellation

Source audit found no whole-image decoded buffer or full-page change/label map
in production analysis. `analysis_engine.rs:540–638` keeps row-sized state;
`:841–954` compacts frontiers and retains only 16 completed components;
`:956–1056` bounds live local-window rows by validated size/stride;
`:1159–1306` bounds chunks/text/geometry; `:1493–1509` uses a 32 KiB
backreference ring. Region node **length** is at most twice width; vector
capacity growth remains O(width), not necessarily exactly that node count.
The pinned decoder's unfiltering buffer compacts consumed rows and its inflate
buffer maintains a bounded lookback; no `next_frame` whole-image API is used.

Private global-allocation instrumentation used actual **file streams**, not
whole-image inputs allocated by the test inside the measured interval:

| Pair (identical opaque RGB) | Eligible pixels | Peak extra requested heap bytes | Largest allocation |
|---|---:|---:|---:|
| 4,096×16 | 65,536 | 1,247,479 | 262,144 |
| 4,096×4,096 | 16,777,216 | 1,509,623 | 262,144 |
| 64×32,768 | 2,097,152 | 1,368,567 | 262,144 |

A separate full public-entry 64×32,768 alternating-islands image produces
**524,288 significant pixels/components**, largest member area **1**, retained
**16**, omitted **524,272**, for both raw and significant summaries. Peak extra
requested heap is **1,371,639 bytes**, largest allocation **262,144**.
The colocated frontier stress and exhaustive/random BFS tests also pass.

These measured samples and source bounds support absence of an unbounded
whole-page allocation. They do **not** establish process RSS, allocator overhead,
an 8 MiB process ceiling, every legal maximum-size input, or concurrency capacity.
Maximum width and height were exercised separately; the largest measured image
has 16,777,216 pixels, not the 64,000,000-pixel ceiling. Pixel/canvas budgets,
overflowing endpoints and the 128 MiB file cap reject before inappropriate IO or
allocation; a virtual oversized seekable file proves the file cap without
allocating that file.

Cancellation probes independently instrument reads/seeks and assert the public
typed `Cancelled` result during envelope, strict inflation, later row decoding,
and a seek callback. Observed maximum requested read is **8,192 bytes**
(seek-callback case: eight). Pre-cancelled requests and incomplete captures use
panic-on-IO streams to prove preflight rejection; incomplete capture returns
typed `CaptureFailure`, not an unchanged report. No partial report is returned.
Additional in-memory inflation cancellation and per-loop checkpoints were
inspected/rerun. Checks are not a real-time guarantee: an already-blocking caller
read/seek cannot be interrupted. Pure comparison-loop cancellation latency was
source-audited, not separately measured under a timed cancellation thread.

## Findings, limitations and Phase 5 recommendations

**Reproducible material bugs: none found in this scope.** The private reference
transcription error above was test-author error, not an implementation defect.
No severity-ranked implementation fix is proposed. This verdict is limited by
finite probes and the absence of production integration/security fuzzing.

The handoff's limitations are substantially truthful:

- Seeds and the ≥50% ignored-coverage warning are **uncalibrated**. Synthetic
  blocks/reference colors prove arithmetic, not user accuracy, defect detection,
  false-positive/negative rates, font/photo behavior or usability.
- PNG support is intentionally **non-interlaced, nonanimated RGB/RGBA8** with
  IHDR/consecutive IDAT/IEND, valid optional pre-IDAT sRGB and bounded tEXt.
  ICC/gAMA/cHRM/tRNS/palette/grayscale/16-bit/APNG and unknown ancillary chunks
  reject rather than silently lose semantics. Untagged input is assumed sRGB;
  there is no color-profile conversion. Actual Chromium artifact compatibility
  remains untested here.
- Alpha is a conservative comparability gate, including ignored pixels, not
  backdrop-aware compositing. Optional AA is a narrow **horizontal stationary
  RGB-blend heuristic**, off in every seed; it can hide a genuine blend-color
  edit and is not a calibrated text-noise detector.
- Gates override numerical triggers; `0/0` is undefined, never 0% unchanged.
  Structural areas/bands survive masks, including masked height changes.
  `NoSignificantChange` is coverage/policy-limited. Report metadata has no
  probability/confidence/defect/review-label field. It does not authenticate
  artifact digests or prove caller-supplied capture/environment identities.

Before Phase 5 acceptance:

1. Version/export/integrate deliberately; do not store Delta E00 in the existing
   RGB-percentage schema. Validate deserialized requests/reports and artifact
   identity/immutability at the integration boundary. Keep earlier algorithms,
   mask/policy revisions, originals and review history immutable/readable.
2. Persist and show exact coverage, dimensions, gates, warnings and triggers.
   Map capture failure separately from invalid/corrupt analysis. Do not turn
   analysis errors into clean results or semantic `NeedsFix` labels. Rendering
   caution must not become an automatic clean-release/approval signal.
3. Bind/revalidate masks to the exact capture pair/geometry; verify transactional
   history/selective approvals and derivative heatmap/tile agreement with these
   pixel semantics. None of those product operations were tested here.
4. Security-review and broadly fuzz the private PNG/DEFLATE parser, including
   maximal chunks/text/masks, fragmented/hostile IO and uncommon valid streams;
   qualify retained Chromium PNGs before relying on the narrow format subset.
5. Add independently annotated real/consented or authorized browser-generated
   fonts, photos, layout, DPR/environment and replacement-mode cases; calibrate
   presets with user examples. Measure 64-million-pixel CPU/RSS, worst-case
   high-contrast work, cancellation under blocking IO, and worker concurrency/
   backpressure. Workspace/browser/native/runtime/UI verification remains for
   the authorized main phase, not this verifier.

## Preservation and retained evidence

`preservation-before.json` and `preservation-after.json` cover **135 protected
files**, including every **32 corpus files**, engine source, handoff/corpus
reports, PLAN, repository manifest/lock and existing author-harness evidence
(excluding build/vendor/cache directories). **All protected hashes match.**
Read-only corpus validation also matches the retained JSON exactly.

Additional preserved digests:

- Handoff: `02db1d3a64febbf8e919e9c7e92ffeb9cb4f092820af8b30fbe67e1857ba886d`.
- Corpus report: `62ac6bf8fd488133fba4b41fd3ca06430054c80c1a8c6b534e1bdc90879e6db4`.
- Corpus validation: `f850a776adff0c36951bbec51fb9e16a1ff505d0ba41162d7201febfede3257f`.

Other machine-readable evidence is `verification-summary.json`,
`corpus-readonly-validation.json`, `probes/manifest.json`,
`probes/public-probe-results.json`, `probes/deflate-manifest.json`,
`probes/deflate-probe-results.json`, `probes/independent-ciede-vectors.json`,
`probes/perceptual-evidence.json`, `probes/allocation-evidence.json`,
`probes/cancellation-evidence.json`, and `probes/region-stress-generation.json`.
Generated PNG hashes are retained in the private manifests; final evidence
inventory/digests are in `verification-final-evidence.json`. These remain local
private verification materials, not published/shared product reports.
