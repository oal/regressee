# Phase 0 — independent verification, 2026-10-08

## Verdict

**Phase 0 acceptance gate: NOT PASSED. Phase 1 must not start.**

The Linux development spike has reproducible backend successes, but also actual
code defects. Missing Windows/macOS/clean-machine evidence is a separate blocker,
not an explanation for those defects. The implementation's explicit non-acceptance
in `00-status.md` and `README.md` is correct. Its earlier native smoke pass is **not
reproduced on the current display**.

This was a fresh verification pass, not implementation-agent evidence reuse. Read
`PRODUCT.md`, `PLAN.md`, Phase 0 checklist/status, README, decisions 0001–0004,
the Rust/worker implementation, test assertions, scripts and CI recipe. Commands
were run serially. No implementation code changed, no subagents, remote CI,
publication, global installs, privileged changes or browser sandbox bypasses.
Temporary probes are under `/tmp/opencode`; generated fixture evidence is under
`artifacts/`. This report is the only durable project-file change.

## Tests actually run

| Command / probe | Independently observed result |
|---|---|
| `cargo test --locked` | PASS: 5 tests. Signature unit test checks the crypto primitive, not the complete installer. |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS. |
| `.runtime/installed/node/bin/node --test .runtime/installed/worker/test/policy.test.mjs` | PASS: 1 test. |
| `.runtime/installed/node/bin/node --test worker/test/fixtures.test.mjs` | PASS: 1 test, including raster CRC/decompression/density dimensions. |
| `python3 scripts/qualify.py --headed` | First attempt exceeded the verifier's 240-second tool deadline after seven outcomes: incomplete, not a pass. Repeated with 480-second allowance: PASS, all 9 outcomes, including parent SIGKILL/restart recovery. |
| `bash scripts/build-ui.sh` | PASS. Existing private linker-name workaround only. |
| `LIBRARY_PATH="$PWD/.runtime/link-libs" cargo clippy --locked --features ui --all-targets -- -D warnings` | PASS. Upstream `proc-macro-error2 2.0.1` future-compatibility warning remains. |
| `python3 scripts/ui-smoke.py` | FAIL twice: X11 `BadMatch`, opcode 42 (`X_SetInputFocus`). App window was unmapped. Neither attempt is a native UI pass. |
| Default Wayland GPUI launch | FAIL: exit 101, dmabuf import error / `ERROR_SURFACE_LOST_KHR`. |
| Default X11 GPUI launch | FAIL: exit 101, `ERROR_INITIALIZATION_FAILED`. |
| `bash scripts/flatpak-probe.sh` | FAIL: pinned existing SDK cannot launch Chromium's sandbox. No override attempted. This is not a packaged Flatpak test. |
| Independent native X11/lavapipe probes | Conditional successes after **test-only override-redirect mapping of the verifier's own GPUI window**, bypassing the non-mapping window-manager path. Keyboard public/tall capture, actual tile pixels, Tab/Shift-Tab, Menu help action, active cancellation and quit exercised. Not ordinary native launch qualification. |
| Independent library/security probes | Signature/path/symlink-entry/size/hash/OS/schema install rejection and old-runtime preservation passed. Installed ancestor-symlink/schema checks, duplicate-entry detection and PNG final-stream validation exposed defects below. |
| Independent private-Node capture probes | Authenticated origin positive; different-port origin and fresh-context negatives; three additional replacement runs; width-budget rejection passed. Broken-image readiness diagnostic failed. |
| Independent raw protocol probes | Invalid version and missing payload returned `InvalidRequest`; JSON `null` crashed worker with no typed failure; malformed JSON closed worker with exit 64. |
| Direct `target/debug/regressee-spike repair --cancel-after-ms 100` reason/preservation check | Expected exit 1, specifically `runtime setup cancelled`, 0.105 s. Ready-runtime file and original public screenshot hashes unchanged; stage/lock absent. |

The ready runtime was already available. Bootstrap/download and `pack` were **not
rerun**: they were not needed for these tests. Qualification performed a real
repair/extraction/sandboxed screenshot smoke after deliberate app-owned runtime
corruption. Cargo build found the binaries current; their timestamps were newer
than the implementation sources. Installed worker sources were byte-identical to
`worker/src/` before/after verification. No new release/download trust is implied.

### Regenerated evidence, inspected beyond exit codes

- `artifacts/qualification.json`: actual rerun, not the earlier durable
  `00-linux-evidence.json`. Headed smoke 22.145 s, protocol suite 84.716 s, setup
  cancellation 0.530 s (including harness overhead), repair 39.024 s, final
  protocol/recovery suite 77.130 s. Each recorded outcome had zero observed live
  owned processes. Peak summed browser RSS was 1,942,196,224 bytes, about 1.81 GiB;
  this is not PSS or a scale guarantee.
- `self-test-headed.json`, `self-test.json`, public/auth/replacement/tall PNGs and
  JSON were regenerated. Independently checked PNG chunk CRCs, actual dimensions,
  metadata hashes and byte equality of public-before/public-after.
- `artifacts/verifier-image-evidence.json`: public captures identical; tall image
  1440×20492, 41 generated tiles, each at most 512 rows and heights summing to the
  original. Visually inspected public, replacement and native public/tall previews.
  Raster content changes to neutral placeholders; fitted geometry and SVG icon
  survive in the fixture.
- Headed login versus headless replay still differs by **1496 RGB pixels,
  0.115432%, bounds [24,29,150,55]**. This does **not** fail the baseline/check
  contract: both evidence captures use the identical pinned headless mode. The
  different headed mode is for login. Cross-OS equivalence is still untested.
- `verifier-native-*.png`, `verifier-native-cancel.json`: current native evidence,
  explicitly conditional on test-only mapping. Tall and wheel-after screenshots
  are pixel-identical and still show tile 1. Actual wheel navigation is not proved.
- `verifier-security.json`, `verifier-protocol.json`, `verifier-capture.json`,
  `verifier-default-display.json`, `verifier-cleanup.json`: independent probe results.
  Generated/ignored evidence may be overwritten; the observations here are durable.
- `verifier-setup-cancel.json`: separate strict reason/hash/cleanup assertion, not
  qualification's generic nonzero-exit assertion.

The independent temporary probes actually executed were:

```sh
python3 /tmp/opencode/regressee-security-probes.py
python3 /tmp/opencode/regressee-protocol-probes.py
python3 /tmp/opencode/regressee-native-verify.py
python3 /tmp/opencode/regressee-native-cancel.py
env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode \
  PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" \
  .runtime/installed/node/bin/node /tmp/opencode/regressee-capture-probes.mjs
```

The security driver uses `/tmp/opencode/regressee-probe.rs`, compiled with pinned
`rustc`, `-L dependency=target/debug/deps` and `--extern regressee_spike=<current
workspace rlib>`. Its `verify`, `install` and `tile` modes call the existing library
functions with temporary paths and report the exact error. It signs only synthetic
manifest fixtures using Node's built-in Ed25519 implementation and the documented
development seed `[17;32]`. Raw ZIP entry counts were separately compared with
Python `zipfile` and pinned Rust `zip::ZipArchive` (four versus three). These scripts
remain available for handoff now, but are temporary, not new maintained project
tests; the input variants, observations and required assertions are recorded below.

## Reproducible findings

Severity: **P1** = material correctness/task-completion issue to resolve at this
gate; **P2** = scoped hardening/diagnostic/test defect. No remote exploit or
unavailable-machine failure is invented.

### 1. P1 — Failed decoding changes the image currently displayed

**Paths:** `crates/spike/src/images.rs:52–78`;
`crates/spike/src/ui.rs:75–77,89–92,145–152`.

All UI decodes write to the same `artifacts/ui-tiles/tile-00000.png`, etc.
The old viewer remains active while background decoding truncates/replaces those
files. If decoding fails, `Update.tiles` is absent and the old `Tiles` object is
retained. Cache invalidation only happens on successful new tiles/navigation.
Thus old dimensions/status can coexist with partially replaced derivatives, and
the cached current tile can disagree with newly navigated tiles. Originals remain
intact, but displayed evidence is not reliable.

**Reproduction actually run:** decode a valid 64×1200 RGB PNG into a tile directory;
save tile 0's SHA-256; decode a different PNG truncated to 75% of its compressed
bytes into that same directory. Decode returns `unexpected end of file`, but tile
0's hash has changed. `verifier-security.json` records this, not just a predicted
race. Use fresh generation-specific staging directories and publish the complete
validated tile set atomically; preserve the prior displayed generation on failure.

### 2. P1 — PNG stream integrity is not finished/validated

**Path:** `crates/spike/src/images.rs:56–79`.

Row decoding stops after the expected image rows without finishing the PNG stream.
Independently generated 64×1200 PNGs with **missing IEND**, and with **bad final IDAT
CRC**, both returned success and three tiles. Width/pixel/decode-memory limits are
real, but they do not establish valid complete image evidence.

**Reproduce:** generate a valid PNG; remove its last 12 bytes for missing IEND, or
flip one byte of the IDAT CRC immediately before IEND; call `images::tile` (or the
`tiles` CLI). Both were accepted by the pinned decoder path. Finish/validate the
stream before committing derivatives; regression tests must assert rejection and
preservation of an already displayed image, not only tile counts.

### 3. P1 — Cancellation does not cover all work, and idle login Cancel is a no-op

**Paths:** `crates/spike/src/ui.rs:51–54,79–87,196–200`;
`crates/spike/src/supervisor.rs:178–180,304–325`;
`crates/spike/src/runtime.rs:33–44,164–184,305`;
`crates/spike/src/images.rs:56–69`.

- First public capture: press Escape 0.2 s after activation, while installed runtime
  verification is running. Independent native probe measured **14.690 s after
  Escape** before cancellation completed, with no children left afterwards.
  Hashing/verification cannot observe the flag; cancellation only takes effect
  once the subsequent worker request starts. Tiling likewise never checks it.
  Qualification's <5 s assertion starts *after* worker spawn/verification, so it
  does not cover this delay.
- Open sign in, wait for the app's instruction, then Escape while idle. All **11
  child processes remained alive with identical PIDs** after one second; the UI
  falsely says “Stopping capture and managed browser…”. The actor is waiting for
  work, not polling the flag. Starting the next action resets it to false.
- Quit during a subsequent capture exited normally in 0.695 s; final independent
  process/scratch audit was clean. This does not repair the Cancel behavior.

Add a real idle close/cancel action and cancellation checks to bounded hash/decode
work. Test cancellation from user activation through cleanup, including setup
verification and the open-login idle state. Do not silently close a login session
only on the next unrelated operation.

### 4. P2 — Installed-runtime validation has weaker invariants than installation

**Path:** `crates/spike/src/runtime.rs:164–184` (contrast `133–161,249–290`).

`verify` checks only runtime ID, signed bytes, final-component symlinks and hashes.
It does not recheck manifest schema/OS/architecture/count/size budgets, nor reject
symlinked ancestor directories. A signed synthetic tree with `worker` symlinked
outside the runtime root passed. The same tree signed with `version:99` and
`os:"windows"` also passed on Linux. No signature forgery was needed: these tests
used the deliberately disclosed development seed to construct synthetic fixtures.

This is an integrity/repair boundary gap, **not demonstrated privilege escalation
or a bypass of production signatures**. The public development key is already
documented as non-trust. Share manifest validation between install/launch, reject
links in every traversed component (including control files), and define treatment
of unexpected installed files. Do not market this prototype as release authenticity.

### 5. P2 — Duplicate ZIP detection is ineffective with the pinned ZIP reader

**Path:** `crates/spike/src/runtime.rs:239–257`;
pinned `zip-2.4.2/src/read.rs:70–75`.

The reader constructs a name-keyed map, overwriting duplicate names *before*
`z.len()`/`by_index`. The installer's `seen` set never sees both entries.
Independent signed fixture had four raw directory entries (manifest, signature,
two `worker/file` entries); Rust reported only three. It proceeded through archive
validation and failed later at `launch private worker` because this intentionally
minimal fixture lacks Node. A generic `install(...).is_err()` would incorrectly
“prove” duplicate rejection.

Reject duplicates using a raw-entry-preserving directory validation path or an
appropriate reader. Assert the rejection reason and that browser/worker launch
was never reached. Size/hash validation still rejects the selected bad payload;
this finding is parser ambiguity, not proof of arbitrary extraction outside root.

### 6. P2 — Valid JSON `null` crashes instead of producing a typed protocol error

**Path:** `worker/src/index.mjs:79–80,117–123`.

`dispatch` dereferences `r.v` before object validation/its try block. The caller
fires the async dispatch without catching a rejected promise. Send `null\n` to the
private Node worker: exit 1, TypeError on stderr, **no `InvalidRequest` event**.
Invalid version/missing payload do produce the expected typed failure. Malformed
JSON exits 64 by design; no claim that it currently emits a typed event.

**Minimal reproduction** (synthetic input; use private Node):

```sh
printf 'null\n' | env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode \
  REGRESSEE_ARTIFACTS="$PWD/artifacts" \
  .runtime/installed/node/bin/node .runtime/installed/worker/src/index.mjs
```

Guard null/array/non-object inputs before property access, handle dispatch rejection
and define framing/schema failure behavior explicitly. No browser is launched by
this reproduction; do not confuse a worker crash with successful typed rejection.

### 7. P2 — Broken image decoding has no readiness warning

**Path:** `worker/src/capture.mjs:29,67–68`.

Serve a synthetic page with `<img src="/bad.png">`, where the PNG response contains
invalid image bytes. Capture in replacement mode. Independently observed:
`stable:true`, `warnings:[]`, replacement count 0, image complete but natural size
0×0. `complete` includes failed images, and each decode rejection is swallowed.
This is stable broken content, not proof of image readiness. The successful
generated fixtures do not test this case.

Record relevant decode/resource failures and explain skipped replacement content.
Preserve the distinction between “stable” and “successfully loaded.” Geometry
instrumentation is fixture-specific (`#geometry` at lines 22/39); without that
container, its equality can be vacuously true. Do not promote that metadata to a
general geometry-preservation guarantee.

### 8. P2 — Harness success labels exceed some assertions

**Paths:** `scripts/qualify.py:45–54,63–74,79–86`;
`scripts/ui-smoke.py:19–39,59–67,79–95`;
`crates/spike/src/supervisor.rs:435–455`.

- Qualification's expected failure accepts **any nonzero exit**, and omits
  stderr/reasons from evidence. Cancel and corruption checks should specifically
  assert cancellation and integrity rejection; otherwise unrelated setup/spawn
  errors can count as those tests passing. The current independent archive probe
  demonstrates why a generic error is insufficient.
- Process sampling checks observed descendants/groups; a short-lived spawn can be
  missed. Parent-crash testing takes one snapshot before SIGKILL. Add a final
  scoped process/scratch scan and robust failure cleanup, not unscoped process kills.
- Native window selection uses title alone, not PID, and focuses without checking
  map state. This explains this display's `BadMatch` failure. Tall status completion
  is not an assertion of rendered pixels. The independent images did establish
  actual tile rendering under the conditional mapping path.
- The smoke sends core X11 button events for wheel input. Pinned GPUI X11 handles
  `XinputButtonPress`/`Release` in its backend, not those core events. The original
  wheel delivery therefore is not a navigation test. Independent XTest-assisted
  delivery also produced no visual change here: actual human-wheel behavior is
  still unqualified, not conclusively diagnosed as a pager code bug.
- `escape_and_quit:true` is idle Escape dispatch followed by quit, not active cancel.
  Browser-crash/oversized-frame checks should assert specific interruption/framing
  failures as the unknown-command and hang tests already do.

The existing README/status appropriately limit many of these claims. The complete
backend smoke also has meaningful positive assertions (bytes, actual replay,
independent missing-storage negatives); it is not a universally false pass.

## Native interaction coverage and stack risks

- Public and tall keyboard capture, native input/focus styles and actual bounded
  image rendering worked under X11/lavapipe **only after special test mapping**.
  No production seamless-scroll/zoom or latency claim is made.
- Open sign in command completed, but a managed fixture browser window could not
  be identified/accessed through the available native X11 controls. Thus the
  requested **human sign-in → I'm ready → `/private` UI chain was not completed**.
  Automated headed synthetic login → headless `/private` passed in qualification;
  it does not substitute for that UI chain. SSO/MFA was not run.
- Ctrl+O/native file-prompt request and Escape were dispatched. File selection,
  dialog cancellation outcome and portal behavior were **not verified**.
- Menu button visibly changes help/status text, not a dropdown/menu
  (`ui.rs:229–231`). Native menu registration exists at `320–328`; actual native
  OS menu interaction was **not verified**. This remains a feasibility UX gap.
- Accessibility/screen-reader, IME/grapheme/drag selection, resized-window behavior,
  UI p95 latency, near-limit malicious-image memory and page-scale soak were not
  qualified. Fixed light styling is provisional Phase 0, not Phase 1 design approval.
- Both default GPU failures and the pinned SDK nested-sandbox failure were actually
  reproduced. These are genuine stack/packaging risks in this environment, not
  proof that GPUI or every Flatpak architecture is universally impossible.

## Acceptance-gate accounting

| Gate | Position after verification |
|---|---|
| Three declared OS families; real windows/native inputs | Linux conditional software-rendering probe only. Windows/macOS absent. FAIL/open. |
| Private setup without Node/developer tools or terminal | Real local staged repair works; prebuilt developer pack and developer commands still required. No automatic trusted download. FAIL/open. |
| Login, representative replay and persistence | Positive/negative ephemeral fixture replay works; human UI chain, secure restart persistence, credential stores and enterprise flows not qualified. Open. |
| Tall image and bounded memory | Row/tile approach and dimensions demonstrated. Stream-validation and atomic derivative defects must be corrected. Not a complete pass. |
| Cancel/quit without orphan processes | Linux exercised paths ultimately clean, but idle cancel and whole-operation latency defects remain. Other OS process-tree semantics unqualified. Not a complete pass. |
| Runtime/capture comparability | Same-machine headless public equality works. Headed difference is correctly diagnostic. Cross-OS/rendering qualification absent. Open. |
| Linux packaging/prerequisites/accessibility | AppImage path only provisional; no package exists; tested Flatpak SDK direct bundle fails sandbox. Accessibility/default GPU open. |

Historical Node/Playwright/Chromium pins are verified and fixture-only. Do not use
this 2025 browser pin on real sites as a 2026 release runtime. CI YAML is a recipe,
not evidence, and its GPUI compile does not open a native window on another OS.
No scope change authorizing a Linux-only product or insecure plaintext sessions
has been made.

## Bounded next work — do not implement later phases

### Code corrections implementable here, in priority order

1. Validate the complete PNG stream; stage generation-specific tiles and atomically
   swap the viewer generation. Add failure-preserves-preview tests.
2. Make cancellation observe hashing/decoding and implement an idle/login close
   action. Add activation-to-cleanup timing tests and quit-during-launch coverage.
3. Unify installed/pack manifest invariants and safe path traversal; reject raw
   duplicate ZIP entries before extraction. Test exact rejection reasons.
4. Guard protocol inputs/catch dispatch rejection; retain typed failure categories.
   Add null/primitive/array/framing and actual crash-cause assertions.
5. Surface broken image/font readiness failures and qualify non-vacuous replacement
   geometry tests; keep unsupported cases explicit.
6. Tighten negative-test reasons, current artifact freshness/render assertions,
   owned-window identification/map readiness, native input delivery and final
   scoped cleanup checks. Do not claim cancel/wheel/menu/dialog success from dispatch.

### External validation or consequential decisions still needed

1. Obtain real Windows, macOS and clean supported Linux/Fedora machines. Qualify
   native GPU/display, process trees/parent death, installer, dialog/portal and
   secret-store behavior; strengthen platform-specific supervision where required.
2. Resolve default GPUI graphics/accessibility support and Linux packaging with
   sandbox-preserving designs. Software forcing in a developer command is not
   friendly GPU recovery or ordinary native-window evidence.
3. Establish current supported browser pins, trusted signing key custody/hosting
   and automatic no-terminal runtime distribution, including managed-network
   diagnostics and crash-safe installer recovery. Local developer signing is not
   a security substitute.
4. Qualify secure session persistence or explicitly decide an acceptable session
   strategy consistent with the product; complete the human login/ready/private
   task and representative auth-flow testing on supported platforms.

These are Phase 0 corrections/qualification, not a request to build Phase 1
storage, projects, crawler, visual system or product shell. Even after local fixes,
external gates cannot be declared passed from code review or CI configuration.

## Resource accounting

The first timed-out aggregate attempt left a marked dead-worker scratch directory;
the complete rerun exercised recovery and removed it. All later probes closed or
terminated only their own processes/windows. Final independent `/proc` audit found
**zero live private app/Node/browser processes**, an empty `/tmp/opencode/r0`
worker namespace, no installed stage/lock/previous directory, and installed worker
bytes matching source. Synthetic original screenshots remain. No fixture session
state was logged or exported; no non-fixture secrets were accessed.

**Recommendation: keep Phase 0 open; do not start Phase 1.**
