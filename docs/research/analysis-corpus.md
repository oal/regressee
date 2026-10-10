# Independent synthetic comparison corpus and seed policy proposals

SPDX-License-Identifier: MPL-2.0

## Scope and provenance

Corpus: `tests/fixtures/analysis-proposal/`. Prepared independently for later
Phase 5, reading `PRODUCT.md`, `PLAN.md` Section 10 (and image-corpus requirements),
the Phase 5 checklist, and current comparison/domain/persistence interfaces.
This is **not** a Phase 5 completion report or a Phase 3 verification result.

All 26 PNGs are original algorithmic RGB color blocks licensed under the project's
MPL-2.0, with preferred source `corpus.py` and a directory license notice. They
are **not real-site screenshots**, photos, actual font/anti-alias samples, or
evidence of crawl/auth correctness. No secrets or external imagery are embedded.
Only the corpus directory and this report were authored; no Rust, worker,
workspace, runtime, UI, shared status, phase-report, or design changes/tests.

## Exact measurements, independent of perceptual policy

Raw change means any unequal RGB sample at aligned coordinates or a coordinate
present in only one image. There is no color tolerance, padding, resampling, or
automatic alignment. The comparison canvas is the coordinate union: for these
pairs, `max(width) * max(height)` with no absent-in-both corner. All PNGs are
opaque RGB8, so transparency/compositing is outside this corpus.

Ignored rectangles are half-open capture-pixel coordinates. Count their union
once, including unchanged pixels under masks. The denominator is canvas area
minus mask-union area. Report exact numerator/denominator; **0/0 is undefined**.
Height-only unmatched bands count even when their RGB equals the background.
Width-mismatch measurements below are illustrative and do not bypass integrity
gates. No dimension-change pair has masks: Phase 5 must separately test revision
binding, stale masks, mask revalidation, and masks on new/unmatched content.

`manifest.json` includes dimensions, mask rectangles, literal count derivations,
exact raw/overlap/structural/masked counts, rational unmasked fractions, and
annotations. Its component counts are **raw 4-neighbor** counts, not perceptual
components. `policy_classification` is explicitly unevaluated.

| Case | Before → after (pixels) | Raw changed | Ignored union | Unmasked changed / eligible | Unmasked raw components / largest |
|---|---|---:|---:|---:|---:|
| unchanged | 96×64 → 96×64 | 0 | 0 | 0 / 6,144 | 0 / 0 |
| small-rgb-noise | 96×64 → 96×64 | 6,144 | 0 | 6,144 / 6,144 | 1 / 6,144 |
| tiny-icon-long-page | 96×2048 → 96×2048 | 64 | 0 | 64 / 196,608 | 1 / 64 |
| large-coherent-region | 128×96 → 128×96 | 3,840 | 0 | 3,840 / 12,288 | 1 / 3,840 |
| scattered-high-contrast-noise | 128×96 → 128×96 | 192 | 0 | 192 / 12,288 | 192 / 1 |
| added-height-band | 96×64 → 96×80 | 1,536 | 0 | 1,536 / 7,680 | 1 / 1,536 |
| removed-height-band | 96×80 → 96×64 | 1,536 | 0 | 1,536 / 7,680 | 1 / 1,536 |
| inserted-middle-band | 96×96 → 96×112 | 3,072 | 0 | 3,072 / 10,752 | 3 / 1,536 |
| ignored-rectangles | 96×64 → 96×64 | 400 | 448 | 16 / 5,696 | 1 / 16 |
| ignored-only-change | 96×64 → 96×64 | 384 | 448 | 0 / 5,696 | 0 / 0 |
| entire-image-masked | 96×64 → 96×64 | 6,144 | 6,144 | 0 / 0, undefined | 0 / 0 |
| near-entire-image-masked | 96×64 → 96×64 | 6,144 | 6,048 | 96 / 96 | 1 / 96 |
| unexpected-width-change | 96×64 → 104×64 | 512 | 0 | 512 / 6,656, illustrative only | 1 / 512 |

Important constructions:

- Low-amplitude noise is uniform RGB `(240,240,240)` → `(241,241,241)`;
  it intentionally demonstrates why raw count/connectivity alone is inadequate.
- The 8×8 icon begins at `(30,1020)`, crossing horizontal and vertical 32-pixel
  grid seams. Its exact global fraction is `1/3072`, about **0.0325521%**.
- Scattered noise is 16×12 isolated high-contrast pixels on an 8-pixel grid.
- Added/removed height bands are background-colored; unmatched area is 96×16.
- Middle insertion moves a full-width 8-row bar from y=32 to y=48; aligned
  change consists of two 768-pixel bars plus a 1,536-pixel unmatched tail.
- Ignored rectangles overlap by 128 pixels; their 448-pixel union includes
  64 unchanged pixels. One case retains a separate unmasked 4×4 region.
- The near-total mask ignores **98.4375%** of the canvas. Its 100% remaining-area
  difference must not imply whole-page comparison coverage.

## Retained validation: PASS

Command run, without shared builds/runtime/browser/native tests:

```sh
python3 -B tests/fixtures/analysis-proposal/corpus.py validate \
  --report tests/fixtures/analysis-proposal/validation.json
```

Exact retained results:

- **13/13** decoded case-oracle comparisons passed.
- **26/26** complete PNGs decoded and SHA-256/file sizes matched.
- **78/78** chunk CRCs verified (IHDR, IDAT, IEND for every file).
- **576,512** decoded image pixels; **1,737,266** total PNG bytes.
- **7/7** malformed in-memory examples rejected: bad signature, chunk CRC
  mismatch, truncated chunk, missing IEND, trailing bytes, invalid zlib Adler32
  with a valid outer PNG CRC, and oversized IHDR with a valid CRC.

A second read-only helper run reproduced `validation.json` exactly and regenerated
**26/26 byte-identical PNGs in memory**. Attempting `generate` on the existing
corpus was rejected; SHA-256 snapshots confirmed all **32 corpus files** unchanged
by those checks. Seed JSON structure and monotonic threshold ordering also passed;
no perceptual classification was evaluated.

The writer builds PNG chunks with CRCs, filter-0 scanlines, complete stored
DEFLATE/zlib streams and Adler32 checksums. Stored blocks remove variation from
zlib compressor tuning. The independent reader parses bytes, verifies all chunk
CRCs, bounded inflation and full stream termination, then measures coordinate
sets from decoded samples. It does not call painting/generation functions to
derive expected mathematical counts. Literal oracles come from geometry rather
than a successful round trip alone. Bounds are width ≤256, height ≤2048,
area ≤262,144 pixels, file ≤2 MiB. Actual maximum area is 196,608 pixels.

The decoder is intentionally restricted to this corpus format; these results
do not qualify a production decoder or establish perceptual accuracy. Another
agent should rerun it and verify independently in Phase 5. `generate` and report
writing refuse existing outputs; reproduction belongs in a fresh owned directory.

## Transparent starting policies — proposal only

Machine-readable values: `tests/fixtures/analysis-proposal/seed-policies.json`.
**These seeds require Rust implementation and independent verification. They
are NOT proven user-calibrated accuracy, defect detection, or release presets.**

Proposed pixel metric: CIEDE2000 (`kL=kC=kH=1`), opaque 8-bit sRGB converted
to CIELAB using a D65 reference white. A pixel is significant when ΔE00 is
**strictly greater than** the tolerance. ΔE00 is not RGB-channel percentage,
not probability, and not a combined confidence score. Unmatched coordinates
remain structural regardless of RGB. Begin with anti-alias suppression **off**;
any later suppression needs separate versioning and adversarial coverage.

| Seed | Pixel ΔE00 tolerance | Overall fraction ≥ | Local fraction ≥ | Local changed pixels ≥ | Connected region pixels ≥ |
|---|---:|---:|---:|---:|---:|
| Sensitive | 1.0 | 0.0005 (0.05%) | 0.025 (2.5%) | 4 | 8 |
| Balanced | 2.0 | 0.002 (0.2%) | 0.05 (5%) | 8 | 16 |
| Relaxed | 4.0 | 0.02 (2%) | 0.1 (10%) | 16 | 64 |

All fractions use **eligible, unmasked** pixels. Compare thresholds inclusively
(`>=`) except pixel ΔE00 (`>`). Local windows are 32×32 capture pixels, stride
16 in both axes, origins `(16*i,16*j)` for nonnegative i,j within the canvas,
clipped at right/bottom edges. Each qualifying window needs **both** its
fraction and minimum changed count, and at least **64 eligible pixels**.
Report excluded/low-coverage windows instead of dividing by zero. Overlapping
windows are concentration evidence, never summed as distinct changed area.

Connected regions use 4-neighbor connectivity of the significant bitmap after
masking, without dilation; threshold actual member count, not bounding-box area.
Report counts, bounds, largest member area, and which criteria fired. Proposed
classification after gates is an explicit **OR** of global, local, connected,
and dimension criteria. Keep all raw/perceptual metrics even when no criterion
fires, and retain mask/policy/algorithm revision identities and originals.

Common gates and dimension behavior:

1. Capture failures are `CouldNotCapture`; material profile mismatch or
   unexpected width/DPR mismatch blocks ordinary comparison/approval. Allow
   inspection, not a trustworthy clean result. Explicit production/staging URL
   mapping alone is not an incompatibility.
2. Rendering-environment mismatch is caution mode; matching browser build alone
   does not establish cross-OS equivalence.
3. Revalidate capture-coordinate masks on dimension changes; never silently
   stretch or reuse them to hide new content. Preserve signed height delta,
   before/after dimensions, and unmatched-band areas independently of masking.
4. Any validated nonzero height delta triggers structural review evidence for
   all seeds. Geometry must not be averaged into RGB or hidden by small percent.
5. Zero eligible pixels means insufficient evidence, never unchanged or 0%.
   The current enum could use `NotComparable` plus an explicit all-masked reason;
   this is not a capture failure. Warn when ignored coverage is **≥50%**, and
   always report coverage; 50% is itself an uncalibrated warning seed.
6. Metric classification never sets `Intentional` or `NeedsFix`. A visual
   difference is not a confirmed defect; masks and sensitivity do not erase
   review history or prior analysis evidence.

### Conditional targets for later Rust verification, NOT observed results

Assuming a verified perceptual implementation suppresses the +1 gray drift and
keeps the high-contrast samples significant, the seeds should yield:

| Case group | Sensitive | Balanced | Relaxed | Evidence rationale |
|---|---|---|---|---|
| unchanged; small-rgb-noise | No significant change | Same | Same | Zero significant pixels after verified tolerance |
| tiny-icon-long-page | Changed | Changed | Changed | Connected area 64; overlapping local windows additionally protect seams |
| large-coherent-region | Changed | Changed | Changed | Large global/region evidence |
| scattered-high-contrast-noise | Changed | Changed | No significant change | 1.5625% global; isolated area-1 components; local density below thresholds |
| added/removed/middle height bands | Changed | Changed | Changed | Explicit height evidence, regardless of background-colored bands |
| ignored-rectangles | Changed | Changed | No significant change | Remaining 16-pixel region; 16/5696 ≈0.280899% globally |
| ignored-only-change | No significant change outside masks | Same | Same | Positive eligible denominator with zero remaining difference |
| entire-image-masked | Insufficient evidence | Same | Same | Undefined denominator; no clean verdict |
| near-entire-image-masked | Changed + coverage warning | Same | Same | 96/96 changed, but almost no coverage |
| unexpected-width-change | Not comparable | Same | Same | Profile-integrity gate precedes numerical policy |

This table is a hypothesis to test, not `validation.json` output. The helper
does not implement CIEDE2000, significant-pixel tiles, or perceptual regions.

## Current interface gap and Phase 5 handoff

Read-only interface observations at preparation:

- `crates/app/src/images.rs::compare` accepts store/before/after/key/cancel,
  checks equal widths, counts any RGBA channel difference **>16**, treats height
  bands as changed, and flags any changed count. It accepts no mask/preset.
- `crates/app/src/product.rs::analysis` records `rgba-max-channel-v1` with
  `color_tolerance=16/255*100`, `changed_percent=0`, `region_pixels=1`.
- `crates/core/src/domain.rs::AnalysisPolicy` has only color tolerance,
  changed percent, and region pixel fields; current validation calls the first
  two percentages. ΔE00 tolerances must **not** be silently serialized using
  that old percentage meaning. `AnalysisResult` lacks tile/region/reason details
  and explicitly disallows `NoSignificantChange` when eligible pixels are zero.
- `TargetMask` validates rectangles against recorded geometry, but broader
  revision binding, dimension-change revalidation, union semantics, and warning
  behavior need Phase 5 verification. `ComparisonEvidence` stores heatmaps.
- `crates/storage/src/capture.rs` currently restricts persisted analysis to
  `rgba-max-channel-v1` with no mask. Seed JSON is **not** a loadable current
  Rust `AnalysisPolicy` or `AnalysisRevision` fixture.

Phase 5 should implement/version the intended metric and policy schemas, compare
decoded raw oracles independently, verify perceptual conversion against known
color reference vectors, and add equality/boundary, tile-seam, diagonal
connectivity, mask-union, cancellation, and no-eligible-evidence tests. Do not
treat today's raw components as proof of significant-component implementation.

Broader calibration must add real licensed/consented captures or deliberately
generated browser fixtures, anti-alias/font differences, representative photos,
text/layout shifts, capture scaling/DPR, transparency/color metadata, stale masks
on height changes, normal/replacement-mode pairs, and rendering-environment
mismatches. This small synthetic corpus is an arithmetic/edge-case starting
point, not representative usability research or proof of false-positive/negative
rates. User review and a larger independently annotated corpus are needed before
claiming calibrated presets or satisfying the full Phase 5 gate.
