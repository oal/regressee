# Phase 0 — independent focused image verification, 2026-10-08

**Findings 9 and 10: FIXED locally within the documented boundary. Full Phase 0
gate: NOT PASSED. Do not start Phase 1.** No material unexpected defect found.

Read `PRODUCT.md`, `PLAN.md` §9, `00-reverification.md` findings 9/10,
`00-image-corrections.md`, current status, and actual worker implementation/tests.
This verifier ran serially without delegation or implementation edits. Only
synthetic loopback pages and the installed private Node 22.20.0 / Playwright 1.56.1 /
Chromium 141.0.7390.37 were used, with Chromium sandbox enabled. No native graphical
attempts, downloads, global installations, sudo, publication or later-phase work.

## 9 — silent source-selector geometry drift: FIXED; preservation unsupported

Recreated the exact independent page, not the maintained test's oracle:

```html
<style>img{width:80px;height:40px}img[src="/asset.png"]{width:160px}</style>
<img src="/asset.png">
```

All **three** new replacement captures measure the image rectangle
`[x,y,width,height] = [8,8,160,40] → [8,8,80,40]`, with six measured light-DOM
elements. Each has **`stable:true`, `geometryPreserved:false`, and exactly
`warnings:["ReplacementGeometryChanged"]`**. Capture evidence remains available;
it no longer claims warning-free geometry preservation. The normal capture is
stable, warning-free and has a different screenshot hash.

An independent 2,106-element page exceeds the 2,048-element budget: before/after
arrays are empty, measured count **0**, scope **NOT MEASURED**, preservation
**null**, and exactly **`ReplacementGeometryNotMeasured`**, despite stable pixels.

`PLAN.md` §9 and returned limitations correctly exclude CSS/scripts dependent on
`src`, `srcset` or picture-source attributes. These attributes change; layout is
not frozen or universally preserved. Source-dependent visual styling with no
rectangle drift is also unsupported and is not detected by this geometry check.
The supported HTML-raster fit cases below use ordinary non-source-dependent CSS.
Canvas/video/frames/shadow DOM/pseudo-elements/complex backgrounds remain limited;
this pass does not qualify them or broaden simple raster-background support.

## 10 — artificial SVG letterboxing: FIXED; actual decoded pixels

Reused the **original valid 96×1,800 raster bytes**, SHA-256
`37a74b151762b46609d05913443f27d3595ce7ba5a906544d4c5599be03db99d`, on the original
`display:block;width:160px;height:40px;object-fit:fill` page. Independently decoded
the PNG signature/chunks/CRCs, zlib stream and PNG filters with Python stdlib,
without importing the maintained PNG helper. At screenshot coordinates
**x=8…167, y=8…47**, original: **6,400 nonwhite / 0 white**; replacement:
**6,400 exact `#cbd5e1` / 0 white / 0 other**. Every fill pixel was checked.
Before/after rectangles are `[8,8,160,40]`, preservation true, stable true,
warnings empty. Rectangle metadata alone was not counted as pixel proof.

Fresh independent normal/replacement captures of eight fit/position cases, plus
independent decoding of all 16 maintained fit screenshots, agree. The following
locations are **inclusive, relative to each 160×40 box**:

| Replacement case | Exact neutral / white pixels | Neutral location |
|---|---:|---|
| Tall fill | 6,400 / 0 | Whole box |
| Tall contain left | 80 / 6,320 | x=0…1, y=0…39 |
| Tall contain center | 80 / 6,320 | x=79…80, y=0…39 |
| Tall contain right | 80 / 6,320 | x=158…159, y=0…39 |
| Wide contain top | 1,440 / 4,960 | x=0…159, y=0…8 |
| Wide contain bottom | 1,440 / 4,960 | x=0…159, y=31…39 |
| Tall cover bottom / wide cover right | 6,400 / 0 each | Whole box |

Normal captures have matching nonwhite-content/white-letterbox locations.
Pixel assertions check 6,240–6,400 pixels per screenshot, excluding only fractional
contain edge cells; counts/location bounds include all pixels. Fill/cover exclude
nothing. Uniform placeholders cannot prove original-detail crop offsets; coverage,
intrinsic aspect and unchanged fit/position are proven, not detail retention.

Directly inspected decoded placeholder SVGs: **`preserveAspectRatio="none"`**;
width/height/viewBox and browser natural dimensions remain **96×1,800** and
**1,800×96** respectively. Both retain computed `contain` / `100% 0%`, retain the
160×40 CSS box, and return to their original intrinsic rectangles with auto sizing.
The SVG no longer adds its own second aspect-fit step.

Fingerprint and raw installed-worker stored captures record **neutral-raster-v2**;
normal stored captures record **normal-v1**. v1/v2 comparison returns
**blocked-profile-mismatch**. Two fresh NDJSON workers each shut down with exit 0;
stored PNG hashes match response and on-disk metadata. Existing v1 replacement
baselines require a compatible new baseline.

## Commands and results

From `/path/to/Regressee`, serially. For readable paths below,
`E=artifacts/image-verifier-1791495155544931433` and
`T=/tmp/opencode/regressee-image-verifier-1791495155544931433` abbreviate the actual
arguments. Stdout/stderr were saved to the corresponding logs under `E`.

```sh
env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode \
  REGRESSEE_IMAGE_EVIDENCE="$PWD/$E/focused-suite" \
  PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" \
  .runtime/installed/node/bin/node --test --test-concurrency=1 \
  .runtime/installed/worker/test/image-replacement.test.mjs \
  .runtime/installed/worker/test/readiness.test.mjs \
  .runtime/installed/worker/test/policy.test.mjs

env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode \
  PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" \
  .runtime/installed/node/bin/node --test --test-concurrency=1 \
  .runtime/installed/worker/test/*.test.mjs

cargo test --locked
python3 "$T/pixels.py" --make-wide
env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode \
  PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" \
  .runtime/installed/node/bin/node "$T/probe.mjs"
python3 "$T/pixels.py"
python3 "$T/stored.py"
```

- Exact prescribed focused/full installed suites: **4/4 PASS, 7.001 s** and
  **10/10 PASS, 9.954 s**, exits 0. Rust: **11/11 PASS**, exit 0.
- Independent probe: **21 fresh captures**, truthful selector/budget diagnostics
  and eight normal/replacement fit cases; intrinsic/version assertions PASS.
- Pixel decoder: **32 fresh screenshots PASS**; rerun after reducing the decoder's
  overly verbose original-color report to counts (temporary probe only).
- Stored-profile probe: **2 captures PASS**. Final exact documented Python snippet
  importing `scripts/qualify.py:scoped_audit`, printing the audit and asserting
  `not any(audit.values())`: **PASS**. An additional final preservation/freshness/
  managed-executable scan also passes (`final-audit.json`).

## Evidence, preservation and resources

All new evidence is under **`artifacts/image-verifier-1791495155544931433/`**:
`focused-suite/`, `independent/`, `stored-normal/`, `stored-replacement/`, command
logs, `pixel-audit.json`, `provenance.json`, byte/hash manifests, `final-audit.json`
and copies of the three new temporary probe sources. The original raster copy is
an input, not a newly generated screenshot. No existing artifact/history tree was
copied or regenerated. Outputs use unique paths; independent and maintained
image-test evidence writes refuse overwrite.

Final audit: **1,097 prior files hash-unchanged**, including earlier reports,
evidence/history, worker source/tests, product/plan and installed READY. All **12**
worker source/test/package/lock files match source, builder and installed bytes;
no refresh/repack/repair was needed. **43 new screenshot hashes** match metadata;
all new output timestamps are after this run's start. Identical deterministic
hashes do not substitute for freshness checks.

**Zero live managed executables**, empty scoped worker scratch/setup stage/lock/
previous paths, and no probe/maintained image-test/Playwright scratch directories.
Owned contexts/browsers/loopback server closed normally; own scratch was asserted
empty then removed with `rmdir`. No unscoped kills or unrelated cleanup. Non-secret
temporary probe sources and deliberate evidence remain; no credentials accessed.

## Not run / full gate remains OPEN

No builder suite, pre-fix negative reproduction, pack/repair, broader backend/
security/actor qualification, native harness, clippy/fmt, remote CI, downloads,
real-site browsing, near-limit/fuzz/soak, power-loss or hostile-filesystem testing
was rerun. Earlier findings 1–8 retain their **historical** verification scope;
this is not a fresh end-to-end re-verification of those corrections.

Windows/macOS and clean supported Linux/package/sandbox qualification remain
unavailable/open. Native mapping/input prerequisites remain **FAILED/OPEN**;
downstream capture/render/cancel/wheel/login/dialog/menu assertions are not passes.
Default graphics recovery and accessibility remain unresolved. Trusted automatic
no-terminal runtime distribution, production signing/current release-safe browser
pins, secure restart-persistent sessions and representative SSO/MFA remain
incomplete/unqualified; portable process/parent-crash semantics also remain open.
These two local fixes do **not** close Phase 0 or authorize Phase 1.
