# Phase 0 status — 2026-10-08

**Full acceptance gate: NOT PASSED.** A reproducible Linux
development spike exists; it is not three-OS, clean-machine or end-user setup
qualification. No delegated agents were spawned by this implementation agent.

**Execution update, 2026-10-09:** the user approved Python controls and requested
continuation of the remaining phases. Local development resumes under
[`0005-serial-local-continuation.md`](../decisions/0005-serial-local-continuation.md)
without inventing acceptance. The new native recheck is
[`00-native-recheck-2026-10-09.md`](00-native-recheck-2026-10-09.md). Stop statements
below are retained as historical recommendations from the earlier pipeline.

## Latest independent verdict

The orchestration used six GPT 6.1 Sol High agents strictly serially: initial
implementation, independent verification, local corrections, independent
re-verification, focused image corrections, and independent image verification.

[`00-reverification.md`](00-reverification.md) independently confirmed the earlier
local integrity, cancellation, runtime-validation, protocol, and readiness fixes.
[`00-image-verification.md`](00-image-verification.md) independently confirmed the
two remaining image corrections, with fresh **11/11 Rust** and **10/10 installed
worker** tests, actual screenshot-pixel assertions, and clean scoped resource
audits. That focused final pass did not rerun the broader native/backend suite.

**The pipeline stopped before Phase 1.** Native mapping/input remains failed/open;
real cross-platform/clean-machine qualification, trusted automatic runtime
delivery, and secure persistent sessions are still incomplete. Passing local
regressions is not full Phase 0 acceptance.

## Latest bounded image corrections after re-verification

The independent [`00-reverification.md`](00-reverification.md) remains unchanged.
[`00-image-corrections.md`](00-image-corrections.md) records serial implementation
regressions for its findings 9 and 10: measured source-selector layout drift now
emits `ReplacementGeometryChanged`; null geometry emits
`ReplacementGeometryNotMeasured`. Source-dependent CSS/scripts remain unsupported,
not magically preserved. Stable pixels do not establish valid geometry.

Neutral SVGs retain raster intrinsic dimensions and now avoid their own internal
aspect-fit letterboxing. Actual screenshot-pixel fill/contain/cover regressions
pass; the independent tall fill reproduction has **6,400 neutral / 0 white pixels**.
Replacement fingerprint/stored metadata is **neutral-raster-v2**, incompatible
with v1 replacement baselines. Normal mode remains normal-v1.

Refreshed builder, 6,027-file development pack and repaired installed runtime match
current worker sources. Both private-Node serial worker suites pass **10/10**;
both new regressions failed against the pre-fix worker for their expected causes.
Fresh outputs use `artifacts/image-corrections-1791494633721241553/`; **968 prior
evidence/report files** remain hash-identical. Final live managed-process, scratch
and setup audits are empty. No native harness or broader qualification was rerun.
This is local correction evidence, **not a new independent verification or full
Phase 0 acceptance**. External gates and native prerequisites below remain open.

## Local corrections after independent verification

`00-verification.md` remains unchanged. [`00-corrections.md`](00-corrections.md)
records the bounded fixes and fresh implementation regression evidence; it is not
the next independent verification. Complete PNG finish/CRC validation, immutable
atomic tile generations, cancellable hashing/decoding, idle login Cancel and joined
quit, shared runtime validation/raw duplicate ZIP rejection, guarded typed protocol
handling and readiness diagnostics are implemented. No Phase 1 work was started.

Prior correction-run serial Rust tests: **11/11**; installed worker tests: **8/8**.
Tightened Linux backend qualification: **10 outcomes**, exact negative reasons,
fresh image CRC/hash checks and zero observed live owned processes/final scratch.
Actor activation cancellation / idle login cancellation / quit during observed
launch: **220 / 572 / 722 ms** in the final aggregate run. These use the native
controls' shared actor, but are **not native event-delivery evidence**.

Current native harness **FAILS**: ordinary owned window is unmapped; even explicit
test-only mapping did not establish XTest keyboard delivery/completion. Native
capture/render/cancel/wheel assertions downstream are **NOT RUN**, not passed.
The historical native success below must not be treated as current qualification.
Original screenshots/logs/JSON are preserved in
`artifacts/history/pre-corrections-1791490783127977154/` (40 files + hashes).
New evidence has `corrections-*` names; the original independent report and durable
`00-linux-evidence.json` are retained. External gates below remain open.
The final `qualify.py --audit-only` found zero live private source/installed runtime
or project app processes, empty `/tmp/opencode/r0`, and no stage/lock/previous
setup paths. Snapshot hashes and current builder/installed worker bytes were checked.

## What exists

- Rust 1.98.1 + GPUI 0.2.2 workspace and lockfile, official MPL-2.0 text and
  third-party attribution. Small native bench, not a web UI or Phase 1 app shell.
- Native address input/IME prototype, Tab/Shift-Tab and keyboard activation,
  focus styles, clipboard, OS menus, native file prompt, setup/login/capture/cancel
  controls. Bounded background actor/queues and 512-row image tiles with a one-tile
  cache. GPU upload never receives a full arbitrarily tall screenshot.
- Private Node 22.20.0 / npm 10.9.3 builder, Playwright/core 1.56.1, full Chromium
  141.0.7390.37 revision 1194, FFmpeg revision 1011. Integrity-locked npm and Cargo
  dependencies. These historical browser pins are fixture-only, not release secure.
- Developer-signed platform-specific pack; Rust verified staged install/repair,
  disk/size/path/hash checks, sandboxed screenshot smoke and cancelled-setup
  preservation. No global npm/Node, sudo, system browser or `--no-sandbox` fix.
- Version-1 private NDJSON protocol with request/job IDs, acknowledgements,
  command whitelist, typed event names/failures, 1 MiB frames, bounded queues,
  request deadline, cancellation grace and supervised child process groups.
- Deterministic loopback public/auth/expired/error/slow/tall and raster fixtures;
  ephemeral cookie/localStorage/IndexedDB/origin-scoped sessionStorage replay.
- Public repeated capture, image replacement/geometry, session-negative, expiry,
  crash/cancel and environment-comparability tests; native X11 harness and
  cross-OS CI recipe. **CI configuration is not runtime evidence.**

## Tested support matrix

| Target | Actual evidence | Qualification |
|---|---|---|
| Available Linux x64: Ubuntu 26.04.1 userspace, Fedora-named kernel 7.2.4-200.fc44.x86_64 | Rust/GPUI build; managed sandboxed Chromium capture; native X11 software-Vulkan bench | Development probe only; Playwright requires explicit Ubuntu 24.04 fallback download here |
| Same environment, default Wayland/hardware Vulkan | GPUI panics: dmabuf import failure / ERROR_SURFACE_LOST_KHR | FAILED in this shared GPU/display environment |
| Same environment, default X11/hardware Vulkan | GPUI panics: ERROR_INITIALIZATION_FAILED | FAILED; software lavapipe works |
| Ubuntu 24.04 x64 clean machine | No machine available | NOT RUN; proposed Linux qualification target |
| Native Fedora host/packaged runtime | Shared kernel/desktop visibility is not native Fedora userspace evidence | NOT RUN |
| Windows 11 x64 | No machine available | NOT RUN; committed OS, proposed version/arch |
| macOS 14+ Apple Silicon / Intel | No machine available | NOT RUN; committed OS, arch qualification pending |

Linux prerequisites and the AppImage qualification-path decision are in
`docs/decisions/0003-linux-packaging.md`. Neither app package has been built/tested;
the pinned existing Flatpak SDK direct-browser probe failed nested sandbox launch.

## Historical implementation commands and results (before independent verification)

Run from `/path/to/Regressee`. Developer paths contain no real
site/session secrets; fixture credentials are synthetic. Regenerated runtime and
screenshots are ignored; durable conclusions are recorded here.

| Command | Actual result |
|---|---|
| `cargo test --locked` | PASS: 5 tests (image budgets/40-tile row decode, path traversal, signature tamper, corrupt-pack preservation/cleanup) |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS after mechanical lint fixes |
| `LIBRARY_PATH="$PWD/.runtime/link-libs" cargo clippy --locked --features ui --all-targets -- -D warnings` | PASS; upstream proc-macro-error2 2.0.1 has a future-Rust compatibility warning, not a current compile failure |
| `python3 scripts/bootstrap.py` | Initial attempt FAILED explicitly: Playwright does not support ubuntu26.04-x64 |
| `python3 scripts/bootstrap.py --linux-compat ubuntu24.04-x64` | PASS: checksum-verified private Node, private npm ci, exact full Chromium download; unsupported-host warning retained |
| `cargo run --locked --bin regressee-spike -- pack` | PASS: local development-signed 6022-file runtime pack |
| `cargo run --locked --bin regressee-spike -- repair` | PASS: verification, extraction, sandboxed public screenshot and ready switch |
| `cargo run --locked --bin regressee-spike -- smoke --headed` | PASS: headed synthetic login and pinned headless replay, four storage positives/independent negatives, expiry, HTTP errors, stable public captures, geometry replacement, tall capture; **headed/headless pixels differ** |
| `cargo run --locked --bin regressee-spike -- protocol-test` | PASS: browser crash/relaunch, bounded active cancel, worker crash/restart, unknown command and oversized frame rejection |
| `.runtime/installed/node/bin/node --test .runtime/installed/worker/test/policy.test.mjs` | PASS: 1 policy test; origin mapping alone permitted, material profiles blocked, OS cautioned |
| `.runtime/installed/node/bin/node --test worker/test/fixtures.test.mjs` | PASS: generated PNG CRC/chunk/decode and 1x/2x dimensions, 1 regression test |
| `cargo run --locked --bin regressee-spike -- tiles artifacts/tall.png` | PASS: 1440×20492 image → 41 tiles; generated 64×20000 image → 40 tiles in unit test |
| `bash scripts/build-ui.sh` | PASS: private linker names for existing libxcb/xkbcommon-x11, native GPUI executable |
| `python3 scripts/ui-smoke.py` | PASS at its declared scope: opens 1120×800 X11 lavapipe window, edits input, Tab/Enter public/tall capture, bounded tile rendering, Escape dispatch and Ctrl+Q exit |
| `python3 scripts/qualify.py --headed` | PASS: all 9 recorded outcomes have zero live owned processes; cancel/corruption failures are expected, repair preserves/revalidates evidence, parent SIGKILL recovers, private scratch is empty after restart |
| `bash scripts/flatpak-probe.sh` | FAIL (expected investigation result): pinned SDK 25.08 cannot launch the private Chromium sandbox under the tested configuration; no override attempted |
| Default GPUI Wayland/X11 launch | FAIL as above, not hidden by software-rendering evidence |

The first native capture inspection found a screenshot taken before tiled decoding
finished and an image-cache invalidation gap. The batched fix was confirmed: tall
capture status/dimensions and 41-tile preview are visible in the final screenshots.
Synthetic wheel delivery **did not change the displayed tile**. Actual wheel
navigation, file dialog completion, native menu interaction and screen-reader
support are not proven. The separate verifier must check them independently.
Native screenshots predate the final backend-only responsive-fixture/scratch fixes;
UI composition is unchanged, but the verifier should regenerate current evidence.

Repeated backend tests found an intermittent replacement count of 3 instead of 4.
Initial lazy-readiness mitigation was insufficient: diagnostics identified a
malformed responsive fixture (the same 2×1 asset labeled 1x and 2x could select
naturalHeight=0) and an invalid PNG IDAT CRC. Generated valid density-matched
2×1/4×2 rasters and a CRC/dimension regression test replace that fixture. A process
monitor's first run also accidentally included the harness's shared process group;
the corrected monitor tracks only owned worker groups, never the Rust parent’s
inherited group. These failed investigations are not clean passing evidence.
Private scratch initially made Chromium abort (SIGTRAP) when long project paths
exceeded Unix-domain socket naming limits. A reproducible short/long-path launch
probe confirmed the path-length limitation; Linux scratch now uses the approved
short `/tmp/opencode/r0` root. The short-path fix does not weaken sandboxing.
The subsequent complete fault run passed all capture/cancel/tamper/repair and live
process checks but **failed private-profile cleanup**. Playwright's detached browser
groups and a cancellation-during-launch race were identified. Pending launches are
now tracked, and Linux cleanup targets private browser groups by exact owned scratch
path plus verified browser-root executable paths; native quit shares that cleanup.
This is a real defect fix, not an infrastructure excuse. The final aggregate run
passed, with zero live owned processes and an empty private scratch root after
cancel/crash/restart. Earlier false/partial runs were not used as the final report.

### Evidence paths

- `artifacts/self-test-headed.json`, `artifacts/self-test.json`: fixture outcomes,
  exact browser/environment and the headed/headless investigation, no session state.
- `artifacts/public-before.png`, `public-after.png`, `replacement.png`,
  `authenticated.png`, `tall.png` and associated JSON metadata.
- `artifacts/auth-headed.png`, `auth-replay-headless.png`,
  `headed-headless-investigation.json`: observed **1496 unequal RGB pixels,
  0.115432%**, all within text bounds `[24,29,150,55]` on 1440×900. Cause is not
  proven; do not describe it as resolved. Both baseline and check must use the same
  pinned headless mode; headed mode is only login setup.
- `artifacts/ui-initial-linux-x11.png`, `ui-public-linux-x11.png`,
  `ui-tall-linux-x11.png`, `ui-tall-next-tile-linux-x11.png`, `ui-smoke.json`.
- `artifacts/qualification.json` and durable `docs/phases/00-linux-evidence.json`:
  passing process/RSS report, 9 outcomes. Expected cancellation/corruption/SIGKILL
  exits are distinguished from success. This is Linux development evidence only.

## Reference hardware and budgets

AMD Ryzen 9 9950X, 16 cores/32 threads; about 123 GiB RAM, 8 GiB swap; repository
filesystem 892 GiB total, about 728 GiB initially available. Shared host workload
was already substantial (~91 GiB RAM used). Exposed DRM devices report AMD vendor
0x1002/device0x13c0 and NVIDIA 0x10de/device0x2b85; these are not a qualified GPU
model/driver test. Actual working GPUI test uses Mesa lavapipe, not hardware GPU.

Image limits: 4096 wide / 32768 tall / 64M pixels / 256 MiB compressed PNG.
Non-interlaced row decoding has a 16 MiB PNG decoder budget plus at most 8 MiB
RGBA tile buffer at maximum width. One 512-row GPU tile is at most 8 MiB RGBA.
The 1440×20492 capture has ~29.5M pixels (~113 MiB raw RGBA); CPU/browser screenshot
memory still exceeds raw evidence size. Final process sampling observed
~7.5 MiB Rust CLI, ~101.8 MiB Node and **~1.82 GiB summed browser-process RSS** on the
headed/tall smoke. These sums double-count shared pages and are not PSS or scale
promises. No UI p95 latency, 100-page or 3000-target soak is claimed.

Observed development pack: ~291 MiB ZIP, ~802 MiB installed; builder plus pack plus
installed runtime uses roughly 2 GiB before extra caches. Setup's extracted-file
cap is 2 GiB plus 128 MiB free-space reserve; retaining old+stage+pack needs extra
space. No near-budget 64M-pixel screenshot or malicious-image fuzz campaign is
qualified. Final per-command measurements are retained in `00-linux-evidence.json`:
headed smoke 18.618 s, repair 41.591 s, two full fault suites 81.811/84.362 s.
The measured 100-ms setup-cancel probe took 11.382 s because it waited for the
current archive entry. Extraction now checks cancellation/hash incrementally in
64 KiB chunks; the follow-up `repair --cancel-after-ms 100` probe passed in **0.103 s**,
preserved the ready runtime, and left no stage/lock/private scratch. This separate
measurement does not replace the original report. Verification and filesystem
cleanup still have coarser bounds. The stricter follow-up protocol test also passed:
the hang error must specifically be cancellation within 5 s, and an unknown command
must return UnknownCommand after handshake rather than merely fail pre-handshake.

## Remaining gates / defects

1. **Infrastructure validation:** real Windows/macOS and clean Ubuntu/Fedora
   machines; hardware-GPU/default backend; packaged AppImage/Flatpak sandbox,
   filesystem/portal/secret-store tests. The direct Flatpak SDK sandbox probe failed;
   AppImage/Ubuntu 24.04 is the next qualification path, not a supported release.
   Code review/CI cannot close these gates.
2. **Incomplete end-user setup:** prebuilt local pack only. No trusted release key,
   distribution manifest/hosting, automatic pack download, proxy/custom CA flow
   or complete no-terminal clean-machine installation. Public development key is
   deliberately not a security boundary. Interrupted install lock/stage recovery
   requires explicit inspection; crash between directory renames is not qualified.
3. **Stack/UI risk:** no qualified accessibility-tree/screen-reader support found
   in GPUI 0.2.2; text input is a prototype (no grapheme/drag-selection qualification).
   Wheel/dialog/menu tests remain open. Software fallback is a developer command,
   not a friendly automatic GPU-recovery UX.
4. **Sessions:** ephemeral only; no credential-store encryption/restart persistence,
   SSO/MFA/rotating or device-bound tokens. The idle/login Cancel defect was corrected
   with shared actor cleanup and local lifecycle regression evidence (see supplement).
   Native event delivery/user login chain is still unqualified; other OS cancellation
   and launch/quit semantics require real qualification.
5. **Process portability:** Unix process-group supervision, Linux parent-death and
   marked-scratch recovery are implemented. Windows taskkill fallback is not a
   Job Object parent-crash guarantee; macOS parent-crash behavior is untested.
   The bench has no durable product run database; crash outcomes are typed errors,
   not the later-phase persisted interrupted-run lifecycle.
6. **Capture scope:** fixtures only, not arbitrary public sites. No full crawler,
    scope policy, enterprise auth detector, nested scrolling or advanced replacement.
    Rendering fingerprint fonts are family/check metadata, not font-file/GPU hashes.
    Cross-OS comparison is untested; historical browser pin is not release-safe.
    Source-dependent styling remains unsupported; drift/not-measured diagnostics
    and corrected fit pixels are documented in `00-image-corrections.md`.

**Interpretation:** absent machines/clean installers are infrastructure-only
limitations. The shared GPU failures and GPUI accessibility gap are genuine stack
risks requiring resolution; neither proves the mandatory product stack impossible.
Stop at Phase 0 and qualify/resolve these risks before investing in Phase 1.

## Verifier handoff

Use `00-image-corrections.md` for the focused image verifier, and `00-corrections.md`
for the broader prior commands/assertions. Preserve evidence first.
The available managed builder is already provisioned; refresh only worker sources
instead of repeating bootstrap/download/PATH modifications in this correction scope.
Run commands serially; native harness failure blocks downstream native claims.

```sh
cargo test --locked -- --test-threads=1
cargo clippy --locked --all-targets -- -D warnings
python3 scripts/refresh-worker.py
cargo run --locked --bin regressee-spike -- pack
cargo run --locked --bin regressee-spike -- repair
python3 scripts/qualify.py --headed
bash scripts/build-ui.sh
python3 scripts/ui-smoke.py
python3 scripts/qualify.py --audit-only
```

Inspect screenshots and JSON rather than relying on an exit code. Manually test
the managed sign-in → **I’m ready** → `/private` flow, actual wheel tile navigation,
dialog cancellation/selection and quit during work. Confirm no live owned Node/
Chromium processes and no app-owned private scratch remain after cancel/crash/
restart. Qualification injects corruption only into its app-managed runtime and
preserves screenshot evidence. Recheck headed/headless mismatch and lazy readiness.
Keep the full Phase 0 gate open without actual clean-machine three-OS evidence.
