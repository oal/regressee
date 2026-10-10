# Phase 2 — real local single-page loop delivered

Date: 2026-10-09. **Implementation and bounded P1 correction delivered;
independent correction re-verification pending.
Not full Phase 2/platform/release acceptance.** Phase 3+ was not implemented and
no agents were spawned. The concurrent researcher’s release-qualification file
was neither edited nor duplicated. Decisions 0005/0006 permit local continuation;
they do not turn pending public-runtime/native/human checks into passes.

## Bounded correction addendum

The independent verifier's original long-inherited-TMPDIR P1 is preserved in
`02-verification.md`. The product now uses an explicitly validated short private
Linux namespace, with the same UID/path/marker policy for launch, cancellation,
Drop and dead-parent recovery, and no process-group signals. Maintained actual
sandbox-enabled long/short inherited-TMPDIR capture/cancel/app-crash regressions
and the retained relative-runtime smoke are included in `02-corrections.md`, with
fresh evidence and exact serial results. Historical execution evidence below is
retained, and the original verifier report is unchanged. **This is delivery, not an independent repair acceptance**;
Phase 3 must still wait for a fresh verifier before relying on unrestricted
supervisor startup. Native, public-runtime and other platform gates remain open.

## Product behavior delivered

| Area | Maintained implementation / actual behavior |
| --- | --- |
| Private setup | `crates/runtime/src/distribution.rs`: configured Ed25519 public trust key, signed manifest v2, pinned identity/entry points, HTTP(S) streaming with cancellation, size/hash/platform/path/type/duplicate checks, disk preflight, private extraction, real sandboxed screenshot smoke, atomic ready-generation pointer, verify/repair. Failed repair keeps the old pointer; previous runtime generations are retained. |
| Real product worker | `worker/product/src/{index,capture,protocol}.mjs`, `crates/runtime/src/{protocol,supervisor}.rs`: acknowledged/correlated v2 handshake, actual browser identity check and screenshot smoke, capture/progress/warnings/typed errors, cancel/shutdown; bounded stdout queue/frame reader, redacted-by-discard stderr drain, deadlines, direct arguments, parent-death/launch-race handling and owned detached-browser cleanup. No spike dependency or fixture-address substitution. |
| Single target | `crates/app/src/product.rs`: one saved HTTP(S) address, public/no-session environment and desktop profile. Baseline runs freeze one-address inventory, profile, rendering identity and requested URL. Checks revisit that baseline target/profile, not edited settings or discovery. Runtime mismatch is refused. |
| Readiness | Bounded DOM-content navigation, fixed locale/timezone/DPR/desktop settings, reduced motion plus animation/transition/caret suppression; bounded fonts, image decoding, incremental lazy-load scrolling, three screenshot-stability attempts and explicit warnings. Access/HTTP failure, unexpected redirects, image limits and timeouts are not unchanged results. |
| Evidence | `crates/app/src/images.rs`, `crates/core/src/evidence.rs`, storage migration 003 and `crates/storage/src/capture.rs`: immutable lossless original, strict metadata, SHA-256 CAS thumbnail and 512-row tiles. Complete PNG/CRC/IEND validation, no trailing data/interlace/APNG, 4096-wide/32768-tall/64M-pixel/128-MiB compressed budgets; row/tile work rather than whole-image decoding. |
| Baseline ownership | Successful full coverage plus explicit **Save baseline…** confirmation creates a new immutable baseline and transactionally updates the project revision/active pointer and verified references. Failed/cancelled/interrupted captures and comparison runs cannot be activated. Existing runs/baselines/originals stay retained. Ordinary project saves cannot smuggle an active-baseline change or alter its frozen address/profile. |
| Initial analysis | Separate immutable comparison run plus analysis record and tiled heatmap. `rgba-max-channel-v1`: aligned RGBA channel delta above 16/255; any changed pixel flags Changed. Percentage/count denominator is explicit. Unmatched height bands count as changed even if white; width mismatch is refused. This is not calibrated/perceptual sensitivity or an automatic defect verdict. |
| GPUI | `crates/app/src/{main,jobs}.rs`, core focus model: actual setup/repair, capture, cancel, evidence reopen, explicit activation and check actions use the same real services/storage as headless verification. Before/after/diff panes share zoom and capture-pixel pan; at most two decoded tiles per pane. Background BGRA conversion avoids UI-thread decoding. Retired GPUI sprite-atlas images are explicitly evicted, not merely dropped as Arcs. |
| Persistence/recovery | Running rows carry owner PID; startup interrupts only dead-owner work on Linux, never a live concurrent owner. Completed evidence is verified on reopening, no startup GC and no automatic activation. Cancellation after successful capture can retain that capture without completed analysis; UI distinguishes pending analysis. |
| Developer verification | `regressee-flow` is a headless entry to product services, not the spike. `scripts/check-phase2.py` logs serial checks, time/RSS, preservation and process/scratch accounting. `scripts/phase2-native.py` is a bounded owned-window inspection. |

SQLite migrates schema 1/2 to 3 transactionally, retaining existing projects and
artifact behavior. Future schemas remain refused. New run/analysis records are
capped at 2 MiB. Run completion, baseline activation and analysis ownership pin
verified CAS references in their domain transaction. Terminal runs/analysis rows
are not edited to manufacture a new result. Originals/derivatives can remain
complete unreferenced objects after interrupted publication; they are not deleted
automatically or presented as a complete capture.

### Trust and historical-runtime boundary

**Public-site capture is not qualified in this delivery.** The shipped source has
no production host or signing key. `REGRESSEE_DISTRIBUTION` names an explicit local
trusted configuration; missing/invalid trust fails closed and the UI says why.
QualifiedPublic requires matching signed scope, HTTPS and an unexpired signed
support deadline. This represents externally configured release authority, not
an automatic local certification or a UI switch to bypass browser security.

The actual integration pack is newly signed with an ephemeral **development**
key and contains the product worker, verified historical Node 22.20.0 /
Playwright 1.56.1 / Chromium 141.0.7390.37 bytes. It is expressly **owned literal
loopback fixtures only**, never public-site support. Rust refuses public targets;
the worker also restricts external HTTP(S) resources/navigation and closes
WebSockets in development scope. The old fixture worker itself is not imported
as the product implementation. Credentials, sessions, scripts and executable
paths cannot be introduced through project data.

Current npm metadata for Playwright 1.64.0 was fetched, but metadata availability
alone is neither executable authenticity nor runtime qualification. No current
browser was downloaded/executed on that basis, and no production keys or hosted
distribution services were invented. Secure supported-public-runtime acquisition,
signing/hosting/support policy and clean-machine qualification remain explicit
gates. See decision 0007 and `docs/runtime-distribution.md` for implemented
interfaces, not a second packaging research report.

## Exact serial checks and results

Working directory: `/path/to/Regressee`.

Primary retained evidence:
`artifacts/phase2-implementation-2026-10-09-d/` (`E`).
Final container-boundary checks:
`artifacts/phase2-implementation-2026-10-09-final-boundaries/` (`F`).
Latest locked source checks and relative-root runtime confirmation:
`artifacts/phase2-implementation-2026-10-09-final-supervision-confirmation/` (`S`).
Earlier defensive-batch checks are retained separately in `...-final-checks/`.

| Command / scope | Actual result |
| --- | --- |
| Initial `cargo check --workspace --all-features` / later all-target check | Dependency resolution and compile performed; exact lock updated. First check caught unhandled new Reply variants; subsequent check passed. |
| `python3 scripts/check-phase2.py --output E --native` | Build/backend/runtime commands below passed; wrapper exit **1**, correctly reflecting native failure, not a blanket pass. |
| ↳ `cargo fmt --all` | PASS, 0.139 s. |
| ↳ `bash scripts/check-foundation.sh` | PASS, 9.327 s: locked UI-inclusive binary build, fmt check, **56 Rust tests passed**, all-target/all-feature clippy `-D warnings`. The heavy browser integration is intentionally ignored in the ordinary test batch and run explicitly next. |
| ↳ `.runtime/installed/node/bin/node --test worker/product/test/protocol.test.mjs` | **4 tests PASS**, 0.071 s. |
| ↳ `cargo test --locked -p regressee-app --features ui-tests --test product_flow actual_signed_http_product_flow -- --ignored --nocapture --test-threads=1` | **1 substantive end-to-end suite PASS**, 346.293 s wall time. Its captured application-flow interval is 301.052 s; source verification/pack construction accounts for additional time. Actual private product Node suite inside the installed signed pack: **6 tests PASS** (4 protocol + 2 adapter/profile/scope). |
| ↳ `cargo run --locked --bin regressee-foundation -- E/foundation-reopen` | PASS, 4.689 s: schema 3, actual settings reopen and CAS reference guard; its text object is still honestly labeled a synthetic storage fixture, not a browser capture. |
| ↳ `python3 scripts/phase2-native.py --output E/native` | **FAIL / native evidence open**, 5.150 s. Owned PID 1111453/window 37748737 remained `map_state=0`; no screenshot, input, forced map, virtual display or second attempt. Owned process terminated/reaped. |
| `python3 scripts/check-phase2.py --output ...-final-checks --checks-only` | PASS: fmt 0.168 s; locked UI build/fmt/**57 Rust tests**/UI-inclusive clippy 15.359 s; 4 protocol tests 0.062 s. No browser/native repetition. |
| `python3 scripts/check-phase2.py --output F --checks-only` | PASS: fmt 0.143 s; locked UI build/fmt/**57 Rust tests**/UI-inclusive clippy 14.615 s; 4 protocol tests 0.066 s. Includes final artifact/container boundary cases. |
| `REGRESSEE_PHASE2_PNGS=ACTUAL_RETAINED_ORIGINAL_PATHS cargo test --locked -p regressee-app --features ui-tests --lib retained_product_originals_pass_current_row_gate -- --ignored --nocapture --test-threads=1` | **1 additional PASS**, 5.954 s: current complete PNG gate/row decoding/tiling accepted all three actual retained product Chromium originals (unchanged, changed, tall), read-only. Fresh temporary store was cleaned; no browser/native relaunch. Exact paths/hashes are in `F/retained-png-confirmation.json`. |
| `python3 scripts/check-phase2.py --output S --checks-only` | Latest PASS: fmt 0.130 s; locked UI build/fmt/**57 Rust tests**/UI-inclusive clippy 7.893 s; 4 protocol tests 0.070 s. Includes canonical runtime-root supervision and the corrected targeted test. |
| `REGRESSEE_PHASE2_RETAINED_FLOW=E/product-flow cargo test --locked -p regressee-app --features ui-tests --test product_flow retained_runtime_relative_supervision_smoke -- --ignored --nocapture --test-threads=1` | **1 additional actual-runtime PASS**, 17.313 s: signature/full inventory verification of the retained signed development generation, relative-root worker launch, sandboxed browser screenshot handshake and bounded shutdown. No new pack, public target or native-window attempt. Exact command/log: `S/retained-supervision.{json,log}`. |

The real product suite exercises signed HTTP download/install with a sandboxed
worker screenshot smoke, corrupt and redirect downloads keeping the old pointer,
damaged installed-file detection and repair into a fresh signed generation;
real actor capture/activation, separate unchanged and changed checks, same-target
HTTP failure, tall PNG/tiles/background viewer decode, cancel, actual worker
SIGKILL, actual headless-app SIGKILL after a persisted Running row, live-owner
recovery refusal, dead-owner interruption, failed/cancelled/interrupted activation
refusal, public-target refusal under development trust and DB/original reopening.

Actual results: unchanged **0 / 1,296,000 pixels**; changed **6,400 /
1,296,000 pixels**; failed check **CouldNotCapture**, never unchanged; tall page
**20,000 px / 40 tiles**. These are deterministic generated loopback website
screenshots produced by the actual product browser, not fabricated app outcomes.

Earlier retained attempts `...-a`, `...-b`, `...-c` stopped before browser/native
work. They found one foundation-test expectation needing the new visible Results
evidence refresh and three clippy findings (constant chunks, collapsible if,
redundant closure). Those were fixed; logs remain intact. After the sole native
inspection, one source-only defensive batch added explicit sprite-atlas eviction,
pan bounds using the full comparison canvas, runtime extraction-space preflight,
StorageFull categorization and malformed artifact assertions. Final container
boundary checks also reject concatenated PNGs (first IEND must be exact EOF),
bound PNG chunk count, reject platform-reserved/aliased archive names and special
Unix archive modes, and bound a test server’s accept wait. `F` verifies that
batch, including real retained browser PNG interop. The final supervisor delta
canonicalizes the verified runtime root before launching Node and comparing
browser ownership against absolute `/proc` executable paths. `S` verifies the
latest source and exercises a real relative-root launch/screenshot/shutdown.
The preceding `...-final-supervision/` checks passed, but its first targeted test
failed **before worker launch** because the test assumed Cargo retained the
repository-root working directory. The test now constructs its relative path
from the actual crate working directory; the failed log is retained unchanged.
The expensive full browser suite was not repeated for these deltas;
worker/capture JavaScript and the metric algorithm tested in `E` are unchanged.
Independent verification should repeat the full latest-source flow.

## Resource accounting and preservation

- All build/runtime/browser commands were **serial**; Rust build parallelism was
  bounded to four jobs and Rust tests to one thread in the runner. One browser
  operation at a time. No global Node/npm, app-path Python dependency, sudo,
  sandbox disabling, plaintext credentials, external publishing or remote jobs.
- `/usr/bin/time -v` retained command resource data: `E` locked checks max RSS
  **1,030,560 KiB**; complete product suite max RSS **304,060 KiB**, CPU user/system
  **303.88 / 8.95 s**. These are command-reported maxima, **not** separately
  sampled Rust/Node/Chromium concurrent totals or a performance qualification.
  The final relative-root smoke reports max RSS **183,112 KiB**, CPU user/system
  **16.30 / 0.56 s**; most of its interval verifies the retained runtime inventory.
- Signed HTTP pack download: **304,884,626 bytes**. `E` intentionally retains
  **1.9 GiB**: signed test pack, both repaired runtime generations, SQLite and
  captured evidence. Failed preliminary evidence directories are 32/36/36 KiB.
  Runtime retention here is intentional test evidence, not a leaked browser.
- Audits after `E`, `F` and the final `S` smoke: **zero live owned app/Node/Chromium candidates and
  zero product scratch directories**. Owned fixture server threads and child
  processes were closed/joined; failed extraction/download temporaries cleaned.
  `S/final-resource-preservation-audit.json` scans executable ownership across
  the entire workspace, all retained product-runtime caches and pending artifact
  publication paths, and compares protected document hashes with `E`.
- Original Phase 0/1 reports, decisions, PRODUCT/PLAN/DESIGN hashes match the
  pre-check manifests. Historical runtime bytes were signature/hash-verified
  before read-only packaging. No historical reports/captures were overwritten or
  recursively copied. Only specifically overwritten README/surface/index/checklist
  documents/notices were snapshotted; test outputs use unique paths.

## Explicit limitations and acceptance gaps

- **Secure arbitrary-public-site runtime and hosted release configuration:
  pending.** The current locally exercised pack is old, development-only and
  deliberately refused for public sites. Public URL settings can be saved; this
  is not yet a public-site hands-on/release preview.
- **Native GPUI rendering/input, visual approval, accessibility, OS dialogs,
  clipboard/IME on a real desktop, resize/hardware/Wayland: open.** Simulated
  GPUI tests prove stated action/focus/storage behavior only. The sleek/modern
  light proposal is inherited, not newly human-approved. No webview substitution.
- Windows/macOS process-tree adapters and clean supported Linux/platform/package
  qualification: **NOT RUN / pending**. Product worker launch currently fails
  closed off Linux. Existing three-OS compilation CI definitions were not
  dispatched; no cross-platform smoke pass is claimed.
- Only one target/profile, normal images and public/no-session access. No crawl,
  login, mapped environments, resume, masks, sensitivity calibration, selective
  acceptance, ZIP, full history/cleanup UI or later-phase features. Navigation
  can itself have effects; captures do not click controls or submit forms.
- PNG/image/readiness budgets and bounded full-page scrolling can reject very
  tall/infinite/nested/virtualized content. Font/image/motion warnings are not
  assurances of every app’s semantic readiness. The initial color metric is
  deliberately simple; rendering/font portability remains unqualified.
- Analysis cancellation can leave a completed capture with no analysis. Evidence
  is retained and Results labels analysis pending; start a fresh check. There is
  no Phase 3 resume or Phase 5 reanalysis implementation.
- Static path/integrity checks protect against unsafe layouts/corruption and
  cooperating writers, not a malicious same-user filesystem/DB attacker or all
  hard-link/TOCTOU attacks. PID reuse can conservatively leave stale work Running;
  it does not authorize deletion/activation. Linux local tests do not establish
  Windows ACL/power-loss semantics or total machine-crash guarantees.
- Runtime generations are retained; no automatic cache/evidence GC. Setup checks
  extraction space and writes fail safely, but continuous low-space monitoring,
  storage cleanup UX, managed-network/custom-CA UI and complete dependency/license
  release audit remain qualification/later obligations. Existing upstream
  `proc-macro-error2 2.0.1` future-Rust warning remains.

## Independent verifier handoff

Read PRODUCT/PLAN, Phase 2 checklist, decisions 0005/0006/0007, the Phase 1
reverification, this report and actual product sources. Do not edit the concurrent
researcher’s file, infer external passes or implement Phase 3 during verification.

```sh
# Fresh paths; preserve all implementer evidence. About six minutes / ~2 GiB for
# the full signed fixture pack/browser suite on this host; commands are serial.
python3 scripts/check-phase2.py --output /tmp/opencode/regressee-phase2-verifier-UNIQUE
# The individual heavy command (build UI binaries first for app-crash fixture):
bash scripts/build-ui.sh
REGRESSEE_PHASE2_EVIDENCE=/tmp/opencode/regressee-phase2-flow-UNIQUE \
  cargo test --locked -p regressee-app --features ui-tests --test product_flow \
  actual_signed_http_product_flow -- --ignored --nocapture --test-threads=1
# Read-only retained-pack confirmation of the latest relative-root supervisor:
TMPDIR=/tmp/opencode LIBRARY_PATH="$PWD/.runtime/link-libs" \
  REGRESSEE_PHASE2_RETAINED_FLOW="$PWD/artifacts/phase2-implementation-2026-10-09-d/product-flow" \
  cargo test --locked -p regressee-app --features ui-tests --test product_flow \
  retained_runtime_relative_supervision_smoke -- --ignored --nocapture --test-threads=1
```

Inspect configured versus development trust, raw ZIP duplicate/path/type rejection,
immutable snapshot/reference transactions, metadata/token/hash/PNG rejection,
failed-check classification, cancellation before/during download/capture/analysis,
live/dead owner recovery, detached-browser/launch-race cleanup, bounded UI queues,
background tile decode and sprite-atlas eviction. Repeat the latest defensive
deltas behaviorally, not just from source. Native attempt only on a meaningful
available desktop: `python3 scripts/phase2-native.py --output NEW_DIRECTORY`;
stop on unmapped/blank, no black-window loops. Ordinary supported-OS native and
human review remain distinct gates. Write a separate verification report and
update the pipeline index with the actual scoped verdict.
