# Phase 0 bounded image corrections — 2026-10-08

**Full acceptance gate: NOT PASSED. Do not start Phase 1.** This is serial
implementation/regression evidence for findings **9 and 10** in the unchanged
[`00-reverification.md`](00-reverification.md), not independent acceptance.
No agents were delegated, native mapping/UI polish retried, or later-phase work
started. Only synthetic loopback fixtures and the existing fixture-only private
Node 22.20.0 / Playwright 1.56.1 / Chromium 141.0.7390.37 were used, with Chromium's
sandbox enabled. No downloads, global installs, sudo or external services.

## Exact corrections and limits

### 9: layout drift is now an explicit warning; preservation is not implemented

`worker/src/capture.mjs` emits **`ReplacementGeometryChanged`** when the measured
replacement light-DOM rectangles differ. `stable:true` still describes repeated
screenshot bytes only; it does not suppress this warning or change
`geometryPreserved:false` to a success. Captures remain available as warning-bearing
evidence, not dimension-preserving results.

Over the existing 2,048-element instrumentation budget, geometry remains **null /
NOT MEASURED**, now also with **`ReplacementGeometryNotMeasured`**. This is not
preservation. `PLAN.md` Section 9 and returned replacement limitations explicitly
exclude styling/scripts dependent on `src`, `srcset`, or picture-source attributes.
No fixture-specific width override, CSS freezing, or broad replacement rewrite
was introduced. Source-dependent visual styling without rectangle changes is
still unsupported and is not detected by rectangle measurement.

The exact independent selector reproduction still changes **160×40 → 80×40**,
three times. Each fresh result has `stable:true`, `geometryPreserved:false`, and
`warnings:["ReplacementGeometryChanged"]`. Finding 9's silent-confidence defect
is corrected; universal source-dependent geometry preservation is **not claimed**.

### 10: remove the SVG's second aspect-fit step, retain intrinsic dimensions

The neutral SVG now has **`preserveAspectRatio="none"`**, keeping its original
`width`, `height`, and viewBox dimensions equal to the decoded raster's natural
dimensions. This stops the SVG's default `meet` behavior from adding internal
letterboxing when the outer image uses `object-fit:fill`. The image element's CSS
`object-fit`/`object-position` continues to perform fitting/placement using the
retained source aspect ratio; no element layout or fit/position override is added.

This changes replacement pixels materially. `IMAGE_ALGORITHM` is now
**`neutral-raster-v2`**, shared by `fingerprint()` and `index.mjs` stored capture
metadata. Normal captures remain `normal-v1`. The comparability regression blocks
v1/v2 interchange; existing replacement baselines need a compatible new baseline.

## Maintained regressions and actual pixels

- `worker/test/image-replacement.test.mjs`: three selector captures; eight
  object-fit/aspect/position cases, each captured in normal and replacement mode;
  separate intrinsic dimensions/computed-fit/position and auto-size assertions.
- `worker/test/png-helpers.mjs`: synthetic opaque red 96×1,800 and 1,800×96 PNGs;
  dependency-free actual screenshot PNG CRC/completion checks, decompression,
  unfiltering, and RGB pixel lookup. No screenshot hash or rectangle equality is
  substituted for the pixel assertions.
- Pixels in each 160×40 box are checked against an independent fit/placement
  calculation: red inside for the original, neutral `#cbd5e1` for replacement,
  white outside for contain. Only fractional internal contain-edge pixels are
  excluded; **6,240–6,400 pixels per screenshot** are checked. Fill/cover check all
  6,400 pixels exactly, with no exclusions.
- `readiness.test.mjs`: null geometry now also requires the not-measured warning.
  Existing readiness, repeated img/picture/background, session and protocol tests
  continue to pass. `policy.test.mjs` requires v2 and blocks the old v1 fingerprint.

Installed replacement screenshot counts (all normal captures also pass their
red/white pixel oracle):

| Case in 160×40 box | Exact neutral pixels | Exact white pixels | Result |
|---|---:|---:|---|
| Tall fill | 6,400 | 0 | No incorrect white letterboxing |
| Tall contain left / center / right | 80 each | 6,320 each | Correct narrow content and distinct horizontal placements |
| Wide contain top / bottom | 1,440 each | 4,960 each | Correct short content and distinct vertical placements |
| Tall cover bottom / wide cover right | 6,400 each | 0 each | Correct full-box coverage |

These are opaque neutral placeholders. Cover crop offsets cannot be inferred from
uniform pixels; coverage and unchanged CSS are tested, not recovery of original
image details. The direct intrinsic probe retains **96×1,800**, `contain`, and
`100% 0%`, and becomes 96×1,800 when CSS sizing is returned to auto.

## Commands actually run, serially

From `/path/to/Regressee`:

| Command / probe | Observed result |
|---|---|
| Fresh new regression tests copied to `/tmp/opencode/.../old-tests/`, importing the pre-fix installed capture module | **Expected FAIL: 0/2**. Selector warning was absent; first replacement fill pixel was white instead of neutral. Demonstrates both regressions catch the old defects. |
| `python3 scripts/refresh-worker.py` | PASS; source/test copy only, unchanged package/lock/dependencies. |
| `env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode REGRESSEE_IMAGE_EVIDENCE="$PWD/artifacts/image-corrections-1791494633721241553/builder-probes" PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/source/browsers" .runtime/source/node/bin/node --test --test-concurrency=1 .runtime/source/worker/test/*.test.mjs` | **10/10 PASS**, 9.874 s. |
| `cargo run --locked --bin regressee-spike -- pack` | PASS; development-signed **6,027-file** pack. |
| `cargo run --locked --bin regressee-spike -- repair` | PASS; staged installed validation and sandboxed screenshot smoke. |
| Same serial private-Node suite using `.runtime/installed/{node,worker,browsers}` and `installed-probes` evidence directory | **10/10 PASS**, 9.977 s. |
| Copies of the original selector and fill probes, imported from the corrected installed worker; fresh output root and selector warning assertion | PASS: selector 3/3 truthful drift warnings; fill rectangle preserved with no warnings. |
| Additional inline Python CRC/decompression/unfilter pixel probe on the independent fill screenshots | PASS: original **6,400 nonwhite / 0 white**; replacement **6,400 exact neutral / 0 white**, versus the old report's 6,240 white pixels. |
| `/tmp/opencode/regressee-image-corrections-1791494633721241553/profile-probe.py` | PASS: real installed worker NDJSON captures store `normal-v1` / `neutral-raster-v2`; replacement count 4, stable pixels, measured preserved geometry, no warnings, verified screenshot hashes, clean shutdown exit 0. |
| Inline prior-hash / source-copy / scoped and full managed-executable process audits | PASS: **968 prior files unchanged**, all **10** maintained worker files match builder and installed bytes; no live managed processes, private worker scratch or pending setup paths. |

Rust implementation/tests, native harnesses, broad qualification, downloads and
other-OS tests were **not rerun** in this worker-only correction scope. The repair
command exercised the existing Rust installer without changing its code.

## Fresh evidence and preservation

All fresh evidence is under
**`artifacts/image-corrections-1791494633721241553/`**:

- `builder-probes/`, `installed-probes/`: 20 screenshot/metadata pairs per runtime
  (selector normal + three replacements; eight normal/replacement fit pairs),
  plus `intrinsic.json`. Metadata includes actual screenshot hashes and pixel
  counts/check totals where applicable.
- `independent-probes/`: separately regenerated original selector/fill probes,
  result JSON and **`fill-pixel-audit.json`** from the additional Python decoder.
- `stored-profile-probe/`: actual installed-worker events, normal image copy,
  replacement image and normal/replacement metadata including algorithm versions.
- `prior-hashes.json`, `final-audit.json`, copies of the three temporary probe
  sources. Temporary sources and old-code regression copies also remain in
  `/tmp/opencode/regressee-image-corrections-1791494633721241553/`.

The prior hash manifest was recorded **before any runtime refresh**, covering old
artifact files and the unchanged independent reports and prior corrections report.
Every recorded file still matches. No old evidence was overwritten or deleted.
Each new image test's own short `/tmp/opencode/ri-*` browser scratch, the original
probe copies' `/tmp/opencode/rs-*` / `rf-*`, and the profile probe's `rp-*` scratch
were asserted empty and removed with `rmdir`. Browsers/contexts and owned worker
were closed normally; no unscoped process kill or unrelated cleanup occurred.

## Fresh focused independent verifier

Review the two findings and the bounded source diff, then run **serially**. The
already repaired installed runtime matches current sources; refreshing/repacking
is only necessary after another source change. Use unique new evidence paths:

```sh
env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode \
  REGRESSEE_IMAGE_EVIDENCE="$PWD/artifacts/image-verifier-$(date +%s%N)" \
  PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" \
  .runtime/installed/node/bin/node --test --test-concurrency=1 \
  .runtime/installed/worker/test/image-replacement.test.mjs \
  .runtime/installed/worker/test/readiness.test.mjs \
  .runtime/installed/worker/test/policy.test.mjs

env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode \
  PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" \
  .runtime/installed/node/bin/node --test --test-concurrency=1 \
  .runtime/installed/worker/test/*.test.mjs

python3 - <<'PY'
import sys, json
sys.path.insert(0, 'scripts')
from qualify import scoped_audit
audit = scoped_audit()
print(json.dumps(audit))
assert not any(audit.values()), audit
PY
```

Independently decode the fresh fill pixels, check contain letterbox **locations**
as well as counts, and recreate the selector fixture without using this test's
oracle. Stable selector screenshots must still carry the drift warning; geometry
budget exhaustion must remain null plus warning. Do not rerun the untouched old
selector script expecting `warnings:[]`, or overwrite its old outputs. A raw stored
replacement capture must record v2, and comparability with v1 must block.

## Remaining gates

These fixes do not solve unavailable Windows/macOS/clean-machine validation,
native display/input/render prerequisites, default graphics/accessibility,
sandbox-preserving packaging, trusted automatic distribution/current release-safe
browser pins, or secure restart-persistent sessions/SSO/MFA. Historical native
failures and other unresolved Phase 0 gates remain as documented in
[`00-status.md`](00-status.md) and [`00-reverification.md`](00-reverification.md).
Source-dependent CSS/scripts remain an explicit replacement limitation. No full
product masking system, Phase 1 work or full Phase 0 pass is claimed.
