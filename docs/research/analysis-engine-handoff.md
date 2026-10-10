# Standalone analysis engine — integration handoff

SPDX-License-Identifier: MPL-2.0

## Scope and status

Prepared independently for later Phase 5, reading `PRODUCT.md`, `PLAN.md`
Section 10, the current image/domain/evidence/storage interfaces, and the finished
`tests/fixtures/analysis-proposal/` and `docs/research/analysis-corpus.md`.

**This is not Phase 5 completion or independent product verification.** The new
`crates/app/src/analysis_engine.rs` is deliberately **not exported or wired**.
Existing `images::compare`, domain types, persisted algorithms, and product APIs
are untouched. No UI, approval, history, capture, browser, runtime, workspace,
native builds/tests, manifests, lockfiles, shared reports, or corpus files were
changed. Repository writes are only this document and the new module. Builds,
dependency copies, generated PNGs, and logs are in the private harness below.

The suggested presets are **uncalibrated seeds**. Passing this synthetic corpus
does not establish real-site accuracy, false-positive/negative rates, defect
detection, accessibility, or release readiness.

## Public API and schema

Entry point:

```rust
pub fn analyze_pngs<A: std::io::Read + std::io::Seek,
                    B: std::io::Read + std::io::Seek>(
    before: A,
    after: B,
    request: &AnalysisRequest,
    cancel: &std::sync::atomic::AtomicBool,
) -> anyhow::Result<AnalysisReport>;
```

It reads seekable **immutable** byte streams from offset zero, validates both
complete PNGs, then compares aligned rows. It writes nothing, creates no
derivatives, and returns no partial report on failure/cancellation. Pass verified
original artifact files, not viewer tiles or thumbnails. Input immutability and
artifact digest authenticity are caller responsibilities; the engine checks
encoded dimensions against the descriptors, not storage identities or hashes.

Public supporting functions/methods:

- `Policy::seed(Preset::{Sensitive,Balanced,Relaxed})`, `Policy::validate()`.
- `AnalysisRequest::validate()` for policy, capture, target, mask and geometry
  validation. Serde deserialization alone does **not** perform this validation.
- `Dimensions::area() -> u64`.
- `Fraction::value() -> Option<f64>`: `None` when the denominator is zero.
- `srgb_to_lab([u8; 3]) -> [f64; 3]`,
  `delta_e00([f64; 3], [f64; 3]) -> f64`. The latter expects finite Lab inputs;
  the actual analysis obtains them only from bounded sRGB samples.
- Typed errors `Cancelled` and `CaptureFailure`, usable with
  `anyhow::Error::is::<...>()`. The public entry point normalizes cancellation
  from IO/decoder wrappers to `Cancelled`. Other invalid inputs/IO errors reject
  analysis; do not convert them to unchanged results or count them as capture
  successes.

Public serializable types:

- Inputs: `Dimensions`, `Target`, `CaptureDescriptor`, `Rectangle`,
  `MaskBinding`, `Preset`, `AntiAliasPolicy`, `LocalCriterion`, `Policy`,
  `AnalysisRequest`.
- Evidence: `Fraction`, `Region`, `RegionSummary`, `WindowEvidence`,
  `LocalSummary`, `DimensionEvidence`, `AnalysisReport`.
- Interpretation: `Classification`, `GateReason`, `Warning`, `Trigger`.

The schema integer is `SCHEMA_VERSION = 1`; the algorithm string is
`ALGORITHM_ID = "aligned-srgb-d65-ciede2000-rows-v1"`. Policy validation rejects
other versions/algorithms. Structs reject unknown JSON fields; antialias JSON is
tagged by `mode`. Policies use **Delta E 00**, not old RGB-channel percentages;
fractions are **0–1**, not 0–100 percentages. No confidence or defect-probability
field exists.

`Policy.revision` must name an immutable policy revision. Seed revisions are
`seed-sensitive-v1`, `seed-balanced-v1`, `seed-relaxed-v1`. The caller must assign
a new revision on every edit; the engine cannot enforce revision uniqueness
across stored records. Algorithm changes affecting numerical semantics need a
new algorithm version. Incompatible serialization needs a new schema version.
The report embeds the entire request, so policy and capture/mask bindings are
preserved alongside the metrics. Do not deserialize foreign reports as trusted
analysis without validating their schema/bindings and evidence.

## Geometry, masks and comparability

`Target { page, profile }` must match across captures. `CaptureDescriptor`
includes immutable `capture_id`, declared dimensions, finite positive DPR,
`material_profile_id`, `rendering_environment_id`, and `completed`.

The integration layer must derive material/rendering identities from actual
capture settings and environment metadata. Production/staging site URLs are
**not** material or rendering incompatibilities. A supplied identity is not an
independent audit of those settings.

`MaskBinding` has `revision`, target, **both exact capture IDs and both exact
dimensions**, plus half-open capture-pixel rectangles. Masks are geometric and
stationary, never scaled or DOM-attached. New capture pairs or dimension changes
require explicitly constructing a revalidated binding, even if the geometry
still fits. This deliberately does not silently reuse a baseline-only mask.
Rectangles must be nonempty, use checked endpoint arithmetic, and fit at least
one image. This excludes absent-in-both corners of crossed dimensions. At most
1,000 rectangles are accepted. Overlaps count once, including unchanged pixels.

The canvas bounds are `max(width) × max(height)`, but the comparison denominator
is the **coordinate union**, minus geometric mask union. An absent-in-both corner
is not eligible content. Height-only pairs match the existing corpus canvas
definition. Missing coordinates always count as structural changes, even when
the other image's color equals the background.

`DimensionEvidence` preserves before/after/canvas sizes, signed width/height
deltas, overlap, before-only and after-only areas, absent-in-both area, and up to
two disjoint missing bands per side. Structural areas/bands remain visible even
when masked. With the seed policies, any nonzero height delta triggers review
regardless of masking.

Gates override numerical triggers:

- `UnexpectedWidth`, `DeviceScaleMismatch`, `MaterialProfileMismatch`:
  `NotComparable`, with inspection metrics retained.
- `AllMasked`: `NotComparable`, undefined `0/0`, never zero-percent unchanged.
- `NonOpaquePixels`: `NotComparable`. RGB/RGBA8 decode is supported, but any
  alpha other than 255 anywhere, including ignored content, blocks ordinary
  comparison. Raw RGBA differences remain inspection evidence; non-opaque
  overlap does not receive invented backdrop-dependent perceptual metrics.
  Missing-band structural evidence is still counted. This is an explicit
  conservative limitation, not a claim to compare alpha correctly.
- A descriptor with `completed = false` produces typed `CaptureFailure` before
  decoding. Map this to the existing `CouldNotCapture` outcome separately.
- Wrong target, stale/invalid masks, corrupt/unsupported PNGs or inconsistent
  declared dimensions reject analysis; no clean result is manufactured.

`RenderingEnvironmentMismatch` is a warning/caution, not a numerical gate. A
`NoSignificantChange` result with this warning **must not** be presented as a
trustworthy clean release check or used to mix incompatible rendering baselines.
`LargeIgnoredCoverage` is inclusive at the policy's warning fraction (seed 0.5).
Always present actual ignored/eligible coverage, even without a warning.

## Numerical semantics and evidence

- Opaque 8-bit sRGB → linear sRGB → XYZ/D65 → CIELAB → CIEDE2000,
  `kL = kC = kH = 1`. No ICC/gamma conversion, resampling, alignment, OCR,
  semantic matching, dilation or layout compensation.
- Significant opaque overlap: **ΔE00 strictly greater than** the tolerance.
  Counts are exact integers; policy thresholds/calculations use binary64.
- Global fraction, local fraction/count/minimum coverage and connected member
  count use inclusive **`>=`**. A zero global threshold still requires at least
  one significant pixel; identical images do not become changed by `0 >= 0`.
- `None` disables a global/local/region criterion. Local sizes are ≤256, stride
  is positive and ≤size, and at most four window rows can be simultaneously
  active. Seed windows are 32×32, stride 16, origins `(16*i,16*j)`, clipped at
  the right/bottom, requiring ≥64 eligible pixels and both local thresholds.
- Local evidence includes total, zero-eligible, insufficient-coverage and
  qualifying window counts, strongest covered window, and strongest qualifying
  window. Overlaps are never summed as distinct changed area. Strongest ties
  prefer more changed members, then stable scan order.
- Exact **4-neighbor** raw and significant connected components after masking,
  no dilation. Area is actual member count, never bounding-box area. Each
  summary preserves total component count, largest member area/bounds, largest
  16 components, and the omitted count. Equal-area ordering uses y, x, height,
  width. Arbitrarily many per-component bounds are deliberately not retained.
- Raw total/overlap/masked/unmasked counts, ignored coverage, structural total
  and unmasked counts, significant count/fraction, and antialias-suppressed count
  remain explicit. Low-amplitude raw noise does not automatically imply a
  significant coherent region.
- `triggers` explicitly names `GlobalFraction`, `LocalConcentration`,
  `ConnectedRegion`, `HeightChange`. After gates, classification is an **OR** of
  the enabled criteria. A criterion may fire on inspection-only metrics while
  a gate still makes the result `NotComparable`.
- `NoSignificantChange` means only that no configured criterion fired within
  eligible coverage; nonzero below-threshold differences can still exist.
  Neither `Changed` nor any metric means confirmed defect, `NeedsFix`, or
  `Intentional`. Review labels/history remain completely separate.

Antialias suppression is **Off for all seeds**. Optional
`SharedHorizontalBlendV1 { max_delta_e00, channel_residual }` suppresses only
bounded ΔE changes at an opaque interior pixel whose stationary left and right
neighbors match exactly across images, have ≥64 channel contrast, and whose
centers are interior blends on that same RGB segment (t between 0.05 and 0.95,
residual ≤2 channels). It never matches pixels to other coordinates. It can
suppress a real small stationary blend-color edit; it is not a calibrated font
noise detector. Vertical/diagonal and genuine browser text antialias behavior
remain future work. Tests retain raw evidence and show a shifted edge is not
matched away.

## Complete PNG checks and bounded work

The supported screenshot subset is deliberately narrow: **non-interlaced,
non-animated RGB/RGBA8**, default PNG compression/filter methods, and sRGB sample
interpretation. Allowed chunks are IHDR, consecutive IDAT (including empty
chunks), IEND, optional valid pre-IDAT sRGB, and validated tEXt. Untagged captures
are assumed sRGB by this input contract. ICC, gAMA (including a nominal sRGB
gamma), cHRM, tRNS, palette/grayscale/16-bit, APNG, interlacing, unknown ancillary
and critical chunks are rejected, not silently normalized or stripped. Real
retained Chromium screenshots still need an integration compatibility check.

Envelope validation checks signature, first/unique IHDR, supported header
fields, file/chunk bounds/order/count, CRC **for every supported chunk**, complete
IEND at exact EOF, text syntax/limits and sRGB order/intent. Strict independent
zlib/DEFLATE validation covers header/window/dictionary rules, stored/fixed/
dynamic blocks, code-tree validity, distances, reserved symbols, exact decoded
byte count, filter bytes, Adler32, stream termination, and no compressed trailing
bytes. It retains only a 32 KiB circular backreference buffer. The pinned PNG
decoder then unfilters rows with checksums explicitly enabled and completes the
stream. This avoids `png 0.17.16`'s intentional acceptance of IDAT trailing bytes
and discarded excess image data. The private parser merits additional fuzzing
and security review before production integration.

Limits: width ≤4,096; height ≤32,768; each image and bounding canvas ≤64,000,000
pixels; each file ≤128 MiB; ≤10,000 chunks; each tEXt payload ≤64 KiB; ≤1,000
mask rectangles. Geometry multiplication/counts use u64; dimensions and rectangle
endpoints are validated before indexing. Crossed geometry is bounded even when
its coordinate union is smaller than the bounding canvas.

No engine full-image decoded allocation or all-page changed bitmap exists.
Owned live state is O(width + chunk-count + max-text-size + mask-count): two
pixel rows, a few row flags/prefix/difference arrays, compact union-find
frontiers, ≤four rows of horizontal window counters, 16 retained components per
summary, and bounded validation buffers. Each region frontier has ≤width live
components; a row's union-find has ≤2×width nodes and uses rank/path compression.
Mask union uses a per-row difference array, not repeated full-width rectangle
painting. PNG decoder accounting limits are 8 MiB per decoder; this is **not** a
measured process-RSS ceiling or a promise that allocator/dependency overhead is
exactly that size.

CPU includes envelope CRC scans, one strict inflation and one row-decoder
inflation per input, per-unequal-opaque-pixel perceptual math, frontier unions and
bounded top-16 ordering, and local prefix/window counters. Mask work is
O(height × (width + rectangle-count)); local work is
O(height × (width + active-window-rows × horizontal-window-count)). Component
summary retention has fixed bounded sorting overhead per completed component.
These are work/storage bounds, **not** measured maximum-capture latency,
throughput, energy or multi-worker concurrency qualifications.

Cancellation is checked before/after entry, at each row, each 256 pixel
comparisons, each 128 mask rectangles, envelope/decoder reads capped at 8 KiB,
each 8 KiB strict-inflate output and each DEFLATE block. Frontier/window work is
bounded by the width cap and checked at the next row boundary. Blocking caller
IO/seek cannot be forcibly interrupted; no real-time cancellation guarantee is
claimed. No report is published after observed cancellation.

## Isolated retained validation

Private harness: `/tmp/opencode/analysis-engine-sol-ddxvnb3v/`.

- Standalone Cargo package, edition 2024, Rust 1.98.1, with a path module pointing
  only at the owned source.
- Exact dependencies: `png = 0.17.16`, `serde = 1.0.228` (derive),
  `serde_json = 1.0.145`, `anyhow = 1.0.100`.
- Separate `Cargo.lock`, `target/`, `cargo-home/`, private copied `vendor/` closure
  from existing registry sources; offline source replacement. No shared target,
  Cargo cache, workspace dependency lock, runtime or browser was written.
- Direct installed toolchain binaries bypass the environment's mise shim; an
  initial shim attempt failed offline without installing anything.

Harness invocation (from that private directory):

```sh
export TC=/path/to/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin
export CARGO_HOME="$PWD/cargo-home" CARGO_TARGET_DIR="$PWD/target"
export RUSTC="$TC/rustc" RUSTDOC="$TC/rustdoc"
export REGRESSEE_ANALYSIS_CORPUS=/path/to/Regressee/tests/fixtures/analysis-proposal
export REGRESSEE_PRIVATE_ZLIB="$PWD/independent-zlib"
"$TC/cargo" test --offline --locked --manifest-path "$PWD/Cargo.toml" -- --nocapture
```

Observed results:

- **19/19 colocated unit tests passed**; no ignored/skipped tests.
- **13/13 exact corpus cases** passed raw, overlap, structural, masked/unmasked,
  denominator, component-count/largest-area and geometry oracles, under a
  zero-tolerance control and all three seeds: **52 analysis runs**.
- **39/39 seed interpretations** matched the explicit proposal hypotheses.
  Seed JSON values were separately compared to the Rust seed constructors.
- The seam-crossing long-page icon is exactly **64/196,608**, one region of 64
  members at `(30,1020,8,8)`. Qualifying local windows are Sensitive **4**,
  Balanced **1**, Relaxed **0**; all three trigger the region criterion. None
  triggers the global criterion.
- +1 gray noise retains **6,144 raw** changes but **0 significant** for every
  seed. Scattered noise retains **192 area-1 components**: Sensitive/Balanced
  changed by global fraction, Relaxed below configured criteria.
- Middle insertion preserves **3,072** changes and **3** aligned components;
  background-colored added/removed bands preserve **1,536** structural members
  and signed height deltas. A separate masked-new-band test retains explicit
  height review with **0 significant** unmasked pixels.
- Overlapping masks retain the exact **448** union, **16/5,696** remaining
  changes. All-masked is **0/0 NotComparable**. Near-total mask retains
  **96/96**, with ignored coverage **6,048/6,144** and a warning.
- 16 published rounded CIEDE2000 reference vectors (including hue wrap/zero
  chroma), five sRGB/D65 Lab references, symmetry/equality, strict pixel and
  inclusive global/local/region boundaries passed.
- Streaming components match a separate BFS for **all 512 3×3 bitmaps** and
  **400 deterministic 31×27 bitmaps**, including mergers and diagonal separation.
  A 32,768-row frontier stress test counts **524,288** isolated components while
  retaining only 16 and keeping ≤32 active components. This checks bounded
  retention, not a full maximum-size PNG benchmark.
- Local-window arithmetic separately matches direct clipped-window/mask counting.
  Cancellation is tested before work and during envelope, inflation and later
  row reads, with typed rejection. Capture failures, DPR/material gates,
  rendering caution, non-opaque pixels, stale bindings and overflow are separate
  interpretation tests, not falsely counted as corruption tests.
- Specific malformed PNG tests assert particular failure reasons (signature,
  chunk CRC including IEND/post-image text, truncation/missing IEND, trailing or
  concatenated files, duplicated header, valid-CRC invalid Adler32, excess/short
  decoded payload, invalid filter, stored length mismatch, truncated zlib,
  reserved block/header, nonconsecutive IDAT and concatenated/trailing compressed
  streams). Limits and unsupported formats have separate explicit rejection
  expectations; generic IO errors are not used as integrity evidence.
- All five PNG filters × three encoder compression choices pass row equality.
  An additional **1/1 private integration test** checks **80 independently
  Python-encoded 128×512 RGB PNG variants** across all filters, zlib levels
  0/1/6/9 and default/fixed/Huffman/RLE strategies against an unfiltered reference.
  Observed first-block types include **stored, fixed and dynamic (0/1/2)**;
  every case reports **0 raw changes** and **65,536 eligible pixels**. This
  private harness test is not a shared app/workspace integration test.
- Read-only Python corpus validation reproduced the retained JSON exactly:
  26 PNGs, 78 CRCs, 26 SHA-256s, 576,512 decoded pixels, 1,737,266 PNG bytes and
  seven specific corruption checks. SHA-256 inventory confirms **all 32 corpus
  files unchanged**. An initial `--report` attempt outside the corpus was
  deliberately refused by its helper; the successful rerun captured stdout to
  the private harness instead, without changing the helper or corpus.

Retained private evidence: `test-results-01.log`,
`generate-private-zlib.py`, `tests/independent_zlib.rs`,
`independent-zlib/manifest.json` and generated PNGs,
`corpus-validation-02.json`, `corpus-preservation-02.json`,
`corpus-before-sha256.json`, `test-results-02.log`, `clippy-results-02.log`,
`final-evidence-02.json`.

Final isolated rerun: **19 unit tests + 1 private harness test passed**, zero
failures/ignored tests. Observed test-run durations were 0.51 seconds and 2.69
seconds respectively; these are small-test-suite timings, not product performance
qualification. Isolated `cargo clippy --offline --locked --all-targets --
-D warnings` passed, and direct toolchain `rustfmt --edition 2024 --check` passed
for the owned source only. An initial clippy run found three style suggestions;
they were fixed in the owned module before this final rerun.

Final owned source SHA-256:
`0c14f6557c5e5f4333d269f674264bd9fa99dc0dd44e54b4c782a5fd4deb533e`.

## Integration work still pending

1. Export/wire this module only during the main Phase 5 integration. Map IDs and
   verified `CaptureMetadata` into descriptors, preserving exact page/profile,
   artifact identities, dimensions, DPR and capture/environment compatibility.
2. Extend/version domain and storage schemas rather than serializing Delta E00
   in `AnalysisPolicy.color_tolerance`'s existing percentage meaning. Current
   `rgba-max-channel-v1` persisted validation does not accept this algorithm.
   Keep old revisions readable without silently recomputing their meaning.
3. Adapt `Changed`, `NoSignificantChange`, `NotComparable` to existing domain
   classifications and typed capture rejection to `CouldNotCapture`. Preserve
   gates/warnings/triggers, mask coverage and dimensions in persisted evidence.
   Treat analysis failure as failed analysis, not automatically failed capture.
4. Implement explicit pair-bound mask revalidation/revision creation. Never
   allow changed settings or masks to erase originals or earlier evidence.
5. Add bounded heatmap/tile generation from the same significance semantics,
   viewer/UI explanations, caution handling, storage transactions, review and
   approval/history behavior. None are implemented here.
6. Independently review/fuzz the strict parser; qualify actual Chromium PNG
   chunk/encoding compatibility and malformed adversarial cases. Expanding the
   format subset needs explicit color/alpha semantics and new tests, not simply
   ignoring unknown metadata.
7. Add real licensed/consented or browser-generated text/font/photo/layout,
   antialias, DPR/scaling, environment, stale-mask and replacement-mode cases;
   annotate them independently and calibrate with users. The present synthetic
   arithmetic corpus and reference colors are not representative user accuracy.
8. Measure maximum-capture CPU/RSS, cancellation under blocking IO, multi-target
   concurrency/backpressure and integration artifact/history correctness in the
   authorized main phase. Workspace/browser/native integration tests remain
   **pending**; none were run concurrently with Phase 4.
