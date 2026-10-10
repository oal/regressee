# Phase 0 — independent re-verification, 2026-10-08

## Verdict

**Phase 0 acceptance: NOT PASSED. Do not start Phase 1.**

The earlier PNG, derivative-publication, cancellation, runtime-validation and
protocol defects no longer reproduce in the tested Linux paths. The readiness
diagnostics and non-vacuous geometry instrumentation also reproduce correctly.
The tightened backend qualification passes; **native qualification still fails**.
Independent additional image-replacement probes found two bounded code defects
below. These are separate from the display/infrastructure constraints.

This is a new independent verification, not acceptance of the implementation
report. Read `PRODUCT.md`, `PLAN.md`, Phase 0 feasibility/status/verification/
corrections, README, decisions 0001–0004, relevant Rust/worker code, assertions,
qualification/image/native scripts and the CI recipe. Commands ran one at a time.
No agents were spawned, implementation code edited, later phases started, remote
CI run, publication/service created, privileged/global installation attempted or
Chromium sandbox disabled. Tests use private runtimes and synthetic loopback sites.

## Evidence preservation

Before regeneration, copied **386 existing evidence/report files** with SHA-256
records to `/tmp/opencode/regressee-reverification-1791493226336818367/`; subsequently
preserved those exact copies under
`artifacts/history/pre-reverification-1791493226336818367/`.
`snapshot-hashes.json` verifies every copy. The original **40-file**
`artifacts/history/pre-corrections-1791490783127977154/` snapshot also still matches
all its hashes. Historical `qualification.json`, verifier artifacts and reports
were not replaced. Original `00-verification.md` SHA-256 remains
`52e911fed3bd80904fc56b94b5819a66ed9788797215de2ba02bd983b880b80e`.

Recommended harness commands regenerate their standard `corrections-*` and
synthetic capture outputs. Copies identifying this verification are:

- `artifacts/reverification-qualification.json`, `reverification-actor.json`;
- `artifacts/reverification-tiles.json`, `reverification-image-evidence.json`;
- `artifacts/reverification-ui-smoke-default.json` and `-conditional.json`,
  corresponding logs, and `reverification-ui-initial.png`;
- `artifacts/reverification-independent-security-results.json`,
  `reverification-independent-protocol-results.json`,
  `reverification-independent-capture-results.json` and
  `reverification-independent-captures/`;
- selector/fill reproductions and `artifacts/reverification-final-audit.json`
  described below.

The conditional native image is **initial-state evidence only**, not a screenshot
of a successfully captured/rendered tile. No stale native result was reused as a
pass.

## Commands actually run

From `/path/to/Regressee`, serially:

| Command | Independently observed result |
|---|---|
| `cargo test --locked -- --test-threads=1` | PASS: 11 tests; read preservation, rejection-reason and no-launch assertions. |
| `cargo test --locked` | PASS: 11 tests, additionally exercising the exact requested default command. |
| `cargo fmt --all -- --check` | PASS. Used the non-writing check instead of `cargo fmt --all`, because implementation edits are prohibited. |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS. |
| `env LIBRARY_PATH="$PWD/.runtime/link-libs" cargo clippy --locked --features ui --all-targets -- -D warnings` | PASS; existing upstream `proc-macro-error2 2.0.1` future-compatibility warning remains. |
| `python3 scripts/refresh-worker.py` | PASS; only managed builder source/test copies refreshed, using existing dependencies and unchanged package/lock files. |
| `env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/source/browsers" .runtime/source/node/bin/node --test --test-concurrency=1 .runtime/source/worker/test/*.test.mjs` | PASS: 8 tests, 6.443 s. |
| `cargo run --locked --bin regressee-spike -- pack` | PASS: development-signed pack, 6,025 files. |
| `cargo run --locked --bin regressee-spike -- repair` | PASS: staged installation and sandboxed screenshot smoke. |
| `cargo run --locked --bin regressee-spike -- actor-test` | PASS: activation/idle-cancel/launch-quit cleanup 220/563/696 ms; original hash and READY preserved. |
| `python3 scripts/qualify.py --audit-only` | PASS after standalone actor test. |
| `python3 scripts/qualify.py --headed` | PASS **at Linux development/backend scope**: 10 outcomes, specific expected failure causes, fresh capture evidence, parent SIGKILL/restart recovery, clean final audit. |
| `env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" .runtime/installed/node/bin/node --test --test-concurrency=1 .runtime/installed/worker/test/*.test.mjs` | PASS: 8 tests, 5.046 s. |
| `cargo run --locked --bin regressee-spike -- tiles artifacts/tall.png > artifacts/corrections-tiles.json` | PASS: new immutable generation, 41 tiles. |
| `python3 scripts/evidence.py --tiles artifacts/corrections-tiles.json` | PASS: actual CRCs, final PNG chunks, hashes/dimensions; height sum 20,492, maximum tile height 512. |
| `bash scripts/build-ui.sh` | PASS; existing private linker names only. Same upstream warning. |
| `python3 scripts/ui-smoke.py` | **FAIL, exit 1:** owned GPUI window not viewable, `map_state=0`. Native input/capture/render/cancel assertions NOT RUN. |
| `python3 scripts/ui-smoke.py --test-map` | **FAIL, exit 1:** `native XTest activation NOT PROVEN: app did not enqueue work`. Explicit verifier-only mapping, not ordinary desktop qualification. Downstream cancel/capture/tile/wheel/idle-login assertions NOT RUN. |
| `python3 scripts/qualify.py --audit-only` | PASS after both native attempts, and again after the independent probes. |

### Additional independent temporary probes

Compiled a fresh driver against the current library, not the old verifier binary:

```sh
rustc --edition=2024 /tmp/opencode/regressee-reverify-probe.rs \
  -L dependency=target/debug/deps \
  --extern regressee_spike=target/debug/deps/libregressee_spike-921faf45d754d4e5.rlib \
  -o /tmp/opencode/regressee-reverify-probe
python3 /tmp/opencode/regressee-reverify-security.py
python3 /tmp/opencode/regressee-reverify-protocol.py
env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode \
  PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" \
  .runtime/installed/node/bin/node /tmp/opencode/regressee-reverify-capture.mjs
env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode \
  PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" \
  .runtime/installed/node/bin/node /tmp/opencode/regressee-reverify-selector.mjs
env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode \
  PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" \
  .runtime/installed/node/bin/node /tmp/opencode/regressee-reverify-fill.mjs
/tmp/opencode/regressee-reverify-probe verify .runtime/installed
```

Security: **64 recorded checks/observations passed**, including direct public-library
calls, independent signed fixtures and PNGs. Protocol: **7 scenarios passed**, with
multiple invalid requests in one scenario. Readiness/geometry: **5 captures passed
their explicit assertions**, including correct detection of deliberate geometry
drift. Selector/fill probes reproduced the defects below, not product passes.
An additional inline Python PNG CRC/decompression/unfilter probe measured actual
fill pixels; preservation/source-copy/process audits were also run inline. No new
dependency or system browser was used. Temporary sources/inputs remain under
`/tmp/opencode`; their result evidence is copied to the artifact paths above.

## Earlier findings: fixed/open accounting

### 1–2. PNG completion and generation publication — FIXED locally

**Code:** `crates/spike/src/images.rs:42–118`; native swap
`crates/spike/src/ui.rs:36–43`.

Independently generated a valid 96×1,800 RGB PNG, decoded an existing displayed
generation, recorded **every tile path and actual SHA-256**, then separately tried
truncation, missing IEND, final IDAT CRC corruption and IEND CRC corruption.
All four reject with EOF/CRC-specific causes. All prior bytes/paths, the original
source hash and the published-generation directory set remain unchanged; no failed
stage remains. This independently reproduces the maintained test's stronger claims.

A separate 2,048×6,000 image cancelled **after a real first staged tile existed**:
the cancellation cause is explicit, the previous generation is unchanged and the
cancelled stage is removed. A successful decode of that image produced 12 tiles.
A concurrent observation thread inspected the output during publication: nine
observations saw a new published directory only with complete metadata/all tile
files, while prior tile hashes remained unchanged. This is an observed Linux
generation-atomic publication, not power-loss durability or native GPU-cache proof.

### 3. Hash/decode cancellation, idle login Cancel, launch Quit — FIXED locally;
native/other-OS delivery remains OPEN

**Code:** `crates/spike/src/cancel.rs`, `runtime.rs:40–52,252–299`,
`images.rs:84–109`, shared `actor.rs`, joined UI Quit/window-close in `ui.rs`.

The hash-reader assertion actually sets cancellation during a real read; it then
requires `runtime verification cancelled`. Decode coverage includes cancellation
after staged output, rather than only an already-set flag. Activation resets the
flag before enqueue, not when the actor receives work; idle actor cancellation
drops the supervised worker and drains cancelled queued work.

Standalone and aggregate actor commands both reproduced actual lifecycle cleanup.
Final aggregate actor evidence: **220 ms activation→cleanup**, including the first
200 ms before cancellation; **20 ms Cancel→cleanup**, specifically runtime
verification cancellation; **568 ms** completed headed-login idle Cancel→cleanup;
**659 ms** Quit→cleanup after an owned Chromium executable/profile was observed
before the login result. READY bytes and the original public screenshot remain
unchanged. Aggregate process monitoring and fresh scoped audits find no live owned
process/scratch afterwards. This is the controls' shared actor, **not successful
native key delivery**. Hostile/hung filesystem calls and other-OS trees are untested.

### 4. Shared installed/pack invariants and links — FIXED locally

**Code:** `runtime.rs:159–301`; `supervisor.rs:181–189`.

Independent signed trees and ZIPs reject bad schema, runtime ID, **OS and architecture
separately**, extraction budget, integer size overflow, file count, empty inventory,
unsafe/reserved control paths and invalid checksum syntax. Installed `verify` and
`Worker::spawn` return the required validation cause rather than reaching a later
missing-Node error. A valid minimal signed tree passes inventory verification.

Root links, a linked ancestor outside the runtime root, payload-directory links,
manifest/signature/READY links, unexpected files/directories and invalid READY
contents all reject. Installed controls are bounded and installation/verification
share `validate_manifest`. Maintained rejection tests additionally assert unchanged
spawn-attempt counters. Invalid installation fixtures preserve the old sentinel and
leave no stage/lock. This remains development-key integrity, **not production
authenticity or hostile concurrent filesystem-mutation race qualification**.

### 5. Raw duplicate ZIP names — FIXED locally

**Code:** `runtime.rs:303–365,405–409`.

Python's raw-preserving ZIP writer created four entries, including two identical
`worker/file` names; independently verified that raw count/name multiplicity.
Installation rejects specifically **`duplicate raw pack entry: worker/file`**, not
checksum/spawn failure. The existing runtime sentinel survives, stage/lock are
absent, and the maintained test proves no spawn attempt. Signature, ZIP symlink and
checksum negatives also reject with their specific causes. ZIP64/multi-disk packs
remain explicitly unsupported.

### 6. Typed protocol/schema/framing/crash handling — FIXED locally

**Code:** `worker/src/index.mjs:79–130`, typed Rust `WorkerFailure` in
`crates/spike/src/supervisor.rs`.

Sent null, booleans, numbers, strings, arrays, empty/bad envelopes and malformed
payload fields, then a valid handshake and unknown command **through the same live
worker**. Invalid requests produce `InvalidRequest`; the worker remains usable;
unknown command produces `UnknownCommand`. No TypeError/unhandled-rejection
diagnostic occurs. Invalid UTF-8/JSON, oversized and incomplete frames produce their
specific typed failure and exactly exit **64**. Deliberate crash exits **86**.
The Rust fault suite additionally requires EOF interruption + actual exit 86 and
downcasts `UnknownCommand` to the typed Rust category.

The suite's same-chunk framing/openLogin test alone is not proof of browser spawn.
The independent probe waited until an actual owned private Chromium PID/profile
was observed, **then** supplied malformed framing. It still emitted `MalformedJson`,
exited 64, closed its observed owned processes and left its private scratch empty.

### 7. Broken-resource readiness / non-vacuous geometry — original finding FIXED;
replacement-contract limitations below remain OPEN

**Code:** `worker/src/capture.mjs:20–45,47–98`.

Independently served invalid raster/font bytes, a 404 stylesheet and a socket-reset
image. Stable pixels correctly coexist with **ImageDecodeFailed, FontLoadFailed,
ResourceRequestFailed and ResourceHttpError** warnings. Metadata records two image
failures, one font failure and two explicit `ImageNotDecoded` replacement skips.
Thus stable content is no longer silently equated with loaded resources.

A separate page without `#geometry` has **three real replacements**, 11 measured
light-DOM element rectangles, nonempty identical before/after geometry and changed
actual screenshot bytes/hash. Over-budget instrumentation returns **null / NOT
MEASURED**, not an empty-array preservation success. Deliberately changed source-
dependent geometry is truthfully detected as false. Maintained suites also pass
their repeated four-replacement fixture, width-budget and session-origin probes.
That instrumentation does not establish that all supported replacement cases
preserve layout or visual object-fit behavior; see the new defects.

### 8. Expected causes, fresh evidence and honest native assertions — backend
corrections VERIFIED; native gate still OPEN/FAILED

**Code:** `scripts/qualify.py`, `scripts/evidence.py`, `scripts/ui-smoke.py`,
`supervisor.rs:425–486`.

Qualification now requires exits/reasons, records stdout/stderr, protects original
evidence/READY through setup cancellation, validates fresh image CRCs/hashes/
dimensions, and ends with a scoped process/scratch/setup scan. Damaged-runtime
rejection has no Node/browser RSS or launch observed. Exact crash/framing categories
are asserted, not a generic error. No unrelated expected failure was counted.

Native selection uses the app PID and viewable map state. Real rendered-raster
threshold checks and XTest input exist, and a fresh **app-written enqueue marker**
is required before active Cancel. Neither native attempt reached these downstream
assertions: ordinary software-X11 launch remains unmapped; conditional mapping
does not prove key activation. Both write fresh failure evidence and finish with
empty scoped audits. These failures neither prove a cancellation implementation
defect nor count as input/render passes. Wheel outcome is still conditional on
actual pixel change/inspection, not merely event dispatch.

## Newly reproduced bounded code defects

### 9. P2 — Replacement changes source-selector-driven layout without a warning

**Path:** `worker/src/capture.mjs:35–36,43–44,86–97`.

Minimal independent page:

```html
<style>
  img { width:80px; height:40px }
  img[src="/asset.png"] { width:160px }
</style>
<img src="/asset.png">
```

Replacement changes `img.src` to the placeholder data URL, so the original CSS
selector stops matching. **Three repeated captures** measure 160×40 before and
80×40 after. `geometryPreserved:false` is truthful, but `stable:true`, `warnings:[]`
and a successful capture still result. Source-dependent styling is not listed in
the replacement limitations. This contradicts an unrestricted dimension-
preservation claim, though it does **not** revive the old vacuous-geometry defect.

Evidence: `artifacts/reverification-selector-results.json`,
`reverification-selector-normal.png`, and
`reverification-selector-replacement-{0,1,2}.png`; normal/replacement visually
inspected. A bounded correction should surface detected geometry drift as a
warning/failure and explicitly constrain unsupported selector-dependent styling
unless preservation is actually implemented. Do not call a stable pixel result
dimension-preserving when the measured metadata says otherwise.

### 10. P2 — Neutral SVG introduces letterboxing for a supported `object-fit:fill`
image

**Path:** `worker/src/capture.mjs:29,34–36`.

Independent valid 96×1,800 raster, with ordinary, non-source-dependent CSS:

```html
<style>img { display:block; width:160px; height:40px; object-fit:fill }</style>
<img src="/asset.png">
```

The normal raster fills its 160×40 box. The replacement SVG's default viewBox
aspect-ratio behavior introduces white letterboxing and only a narrow gray stripe.
The element rectangles are still 160×40 and `geometryPreserved:true`, demonstrating
why rectangle equality alone is not visual/object-fit preservation evidence.
Capture again reports `stable:true`, no warnings.

Independently validated/decompressed/unfiltered screenshot bytes: the original
box has **6,400 nonwhite pixels**, while the replacement has **6,240 white pixels**
and only **160 nonwhite pixels** (80 exactly neutral-gray pixels). Visually inspected
the replacement. Evidence: `artifacts/reverification-fill-{false,true}.png`,
`reverification-fill-results.json`, `reverification-fill-pixels.json`.

This is within the claimed HTML-raster/fitted-image scope, not frames/shadow DOM or
complex backgrounds. A bounded local correction should make the neutral source
respect the raster's object-fit/position behavior and add an aspect-ratio-mismatch
rendered-pixel regression. No full replacement redesign or Phase 1 work is needed
to address these two cases. **No quick fixes were made by this verifier.**

## Fresh backend artifacts and measurements

Final `reverification-qualification.json`: headed smoke **22.278 s**; protocol
suites **67.846 / 66.195 s**; setup-cancel process plus monitor grace **0.515 s**;
integrity rejection **12.026 s**; repair **30.328 s**; actor suite **26.702 s**.
Browser summed RSS peaked at **1,961,177,088 bytes** in headed smoke. This is
sampled summed RSS, not PSS, near-limit memory or page-scale qualification.

Fresh public captures are byte-identical, SHA-256
`f3882ad0cb97f69b20bf5758ab3e4455741de23e2f2fe5b8ff685242c9b2c7e8`.
Public/replacement originals were visually inspected: fixture raster content
changes to gray placeholders; fitted boxes/spacing and SVG icon survive. Fresh
tall evidence is 1,440×20,492 with 41 valid tiles. Headed versus headless login pixels
remain unequal in the fresh headed self-test; headless login/replay equality holds
in the headless tests. No cross-OS equivalence or new explanation of the headed
difference is claimed.

## Tests not run / full acceptance blockers

- Windows/macOS native windows, real inputs, process-tree/parent-death semantics,
  packaging and credential stores: **NOT RUN**, no machines available.
- Clean supported Ubuntu 24.04 and native Fedora/packaged AppImage or alternative
  sandbox-preserving Flatpak runtime: **NOT RUN**. Current unsupported-host
  development backend evidence is not clean Linux qualification.
- Default hardware Wayland/X11 graphics were **not rerun** in this pass; their
  historical failures remain preserved. Software Vulkan testing does not resolve
  default graphics. Accessibility/screen-reader qualification: **NOT RUN/open**.
- Current native cancel/capture/tile rendering/wheel, human login→ready→private
  capture, file dialog completion and actual native menus: **NOT RUN after failed
  native prerequisites**, not silently passed by actor/backend tests.
- Trusted automatic no-terminal first-use distribution, current supported browser
  pins/signing custody, secure restart-persistent sessions and representative
  SSO/MFA: **not implemented/qualified**, remain Phase 0 blockers. An ephemeral
  fixture session and a public development signing key do not meet those gates.
- Bootstrap/network downloads, remote CI, Flatpak probe, power-loss installer
  recovery, hostile filesystem races, image fuzz/near-limit memory, UI latency and
  page-scale soak were **NOT RUN**. CI configuration is only a recipe.

## Final resource accounting and recommendation

`artifacts/reverification-final-audit.json` records **zero live managed app/Node/
browser processes**, empty `/tmp/opencode/r0/worker-*`, and no installed stage,
lock or previous directory. An additional independent `/proc` executable scan
covers all managed-runtime executables, not only harness ancestry. Audits exclude
zombies from *live* process counts; they do not claim control over the shared
environment's unrelated processes. All raw/capture probe private scratch directories
were individually asserted empty and removed. Native harnesses closed only their
own app processes. No unscoped process kills or unrelated cleanup occurred.

Final installed-runtime verification passes. All eight maintained worker source/
test files match builder and installed copies byte-for-byte. Originals/history
remain preserved; synthetic independent inputs/reports remain in the documented
temporary/evidence directories, not as secret-bearing browser profiles. No real
credentials/session state were accessed or exported.

**Recommendation:** keep Phase 0 open. Address the two bounded replacement
defects/diagnostics and rerun their regressions; resolve native mapping/input on an
appropriate desktop rather than treating override-redirect as acceptance. Then
obtain the actual external OS/clean-package/default-graphics/accessibility,
trusted-distribution and secure-session evidence required by the existing gate.
Even fixing both new local defects does **not** authorize Phase 1.
