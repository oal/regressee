# Phase 0 local corrections — 2026-10-08

**Full acceptance gate: NOT PASSED. Do not start Phase 1.** This is implementation
regression evidence, not fresh independent re-verification. No subagents were
spawned. `00-verification.md` is unchanged; its reproduced failures remain part of
the history. Windows/macOS/clean-machine, packaging, default graphics,
accessibility, trusted runtime distribution and secure saved-session gates remain
open. Historical Chromium is still restricted to synthetic loopback fixtures.

## Findings and bounded corrections

| Independent finding | Correction and maintained regression |
|---|---|
| 1: failed decode changes displayed tiles | `images.rs` writes into a uniquely named generation staging directory. Only after validation does a same-parent directory rename publish it. Returned paths name that immutable generation; the native viewer swaps its `Tiles` only on success. Failed/cancelled stages are removed, never old generations. Tests compare **every prior tile's actual bytes**, retain the prior paths, and assert no failed stage remains. Old derivative generations are intentionally retained; no history deletion policy was introduced. |
| 2: PNG stream unfinished | Row decode now calls `Reader::finish()` before publication, validating final IDAT CRC, ancillary/final chunks and IEND. Generated regressions separately reject truncated bytes, missing IEND, bad last IDAT CRC and bad IEND CRC with EOF/CRC-specific causes. The tests first establish a valid displayed generation and prove preservation after each rejection. |
| 3: cancellation too late / idle Cancel inert | Shared `actor.rs` is used by the native controls and `actor-test`. Activation resets the flag **before** enqueue; the actor never resets it on receive. Idle cancellation is polled, drops the supervised worker/login tree and acknowledges only after cleanup. Cancelled queued commands are drained rather than replayed. Verification inventory/hash work checks the flag between bounded chunks; decode checks each row and wraps decoder reads (including finish) in cancellation-aware I/O. A deterministic test cancels after a real first staged tile exists and proves preservation. Hash-reader tests cancel midstream. Quit/window-close join the actor instead of racing an unregistered pending launch. |
| 4: installed invariants weaker | Install and installed verification share schema/runtime/platform/file-count/size/path/checksum validation. Installed traversal rejects links (root/ancestors, payloads and controls), nonregular/unlisted files and unexpected directories; only signed files plus manifest/signature and the exact optional READY marker are allowed. Control reads are size-bounded. Tests exercise signed bad schema/platform/budget/count/path trees, ancestor/control links and unlisted files. Both install rejection and `Worker::spawn` rejection assert exact reasons and a test-only spawn-attempt counter stays unchanged. This is not production authenticity: the development key is public. Concurrent hostile local mutation/openat-style race hardening is not qualified. |
| 5: ZIP reader hides duplicates | A bounded raw central-directory pass runs **before** `ZipArchive`, stage creation or worker launch. Duplicate raw names are rejected explicitly. Multi-disk/ZIP64 packs are deliberately unsupported; compressed and extracted budgets are bounded. A real four-entry ZIP is constructed with duplicate local/central names, then checked for the exact `duplicate raw pack entry: worker/file` cause, zero spawn attempts, old-runtime preservation and no stage/lock. Signature, symlink-entry, checksum and manifest rejection tests also assert early causes, not generic errors. |
| 6: null crashes / dispatch rejection lost | Worker validates null, primitives, arrays, envelope and payload fields before access. Async dispatch has a rejection handler. JSON/UTF-8, oversized and incomplete framing produce typed failures and exit 64; deliberate worker crash is exactly exit 86. Cleanup is an awaited shared promise, so EOF cannot short-circuit an in-progress framing cleanup. Rust retains worker event categories in `WorkerFailure`; unknown-command testing checks its typed `UnknownCommand` code. Fault tests require EOF interruption plus crash exit 86, and the exact framing reason. |
| 7: readiness swallowed / geometry vacuous | Captures record image decode failures, errored fonts and failed/HTTP-error resources separately from pixel stability. Skipped undecoded replacement images have explicit reasons. Replacement placeholder decoding is no longer swallowed. Geometry measures actual light-DOM element rectangles, not `#geometry`; above 2,048 elements it returns **null / NOT MEASURED**, never empty-array “preserved.” Tests cover broken resources with stable pixels, a non-fixture page with **two real replacements**, changed screenshot hash and nonempty unchanged rectangles, the instrumentation budget, three repeated fixture replacements and width-budget rejection. Preserved session-origin positive/negative probes are now maintained tests. Frames/shadow DOM and the other recorded unsupported cases remain limitations. |
| 8: weak evidence/harness labels | Qualification requires exact exits/reasons, records stdout/stderr, checks cancellation preserves original evidence/READY, validates fresh captures against actual CRCs/dimensions/hashes and performs a final scoped process/scratch/setup-path scan. Timeout/orphan cleanup targets observed owned PIDs with start-time checks. Native window selection requires the app PID and viewable map state; render checks require actual fixture raster pixels; wheel uses XTest rather than core button dispatch. Native failures write fresh failure evidence and stop downstream assertions. No dispatched event alone is labeled success. **These current native render/input assertions did not reach completion on this display.** |

### Actual actor lifecycle measurements

Final aggregate run (`artifacts/corrections-actor.json`):

- First capture activation → runtime-verification cancellation → cleanup: **220 ms**;
  Cancel → cleanup **20 ms**, specifically `runtime verification cancelled`.
- Completed headed open-login → idle Cancel → cleanup: **572 ms**, specifically
  `Cancelled. Managed browser closed; completed evidence preserved.`
- Quit after an owned Chromium executable/profile launch was observed, **before**
  login result: **722 ms**. All are below the asserted five-second bounds.
- Original `public-before.png` hash and READY bytes are preserved. The aggregate
  process monitor observed no live owned children afterwards. Native key delivery
  is **NOT RUN by this actor test**; it is not a substitute for the UI gate.

## Exact commands and results

All commands were run **serially** from `/path/to/Regressee`.
Managed builder refresh used the existing pinned dependency tree; no bootstrap,
download, npm install, PATH change, global install or sandbox override was used.

| Command | Final observed result |
|---|---|
| `cargo fmt --all` | PASS. |
| `cargo test --locked -- --test-threads=1` | **11 passed, 0 failed** (including actual stream rejection/preservation and no-launch assertions). |
| `cargo clippy --locked --all-targets -- -D warnings` | PASS. |
| `env LIBRARY_PATH="$PWD/.runtime/link-libs" cargo clippy --locked --features ui --all-targets -- -D warnings` | PASS; existing upstream proc-macro-error2 future-Rust warning remains. |
| `python3 scripts/refresh-worker.py` | PASS: copies only managed builder worker src/test; requires unchanged package/lock files and existing dependencies. |
| `env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/source/browsers" .runtime/source/node/bin/node --test --test-concurrency=1 .runtime/source/worker/test/*.test.mjs` | **8 passed, 0 failed**, 5.790 s in final builder run. |
| `cargo run --locked --bin regressee-spike -- pack` | PASS: final development pack **6,025 files**. |
| `cargo run --locked --bin regressee-spike -- repair` | PASS: validated staged repair and sandboxed screenshot smoke. |
| `cargo run --locked --bin regressee-spike -- actor-test` | PASS: first standalone run 221/591/768 ms; aggregate rerun measurements above supersede it for the final backend evidence. |
| `python3 scripts/qualify.py --headed` | PASS at **Linux development/backend scope**: final **10 outcomes**, exact expected failures, fresh capture evidence, parent SIGKILL/restart recovery, zero observed live owned processes and empty final scoped audit. |
| `env TMPDIR=/tmp/opencode TMP=/tmp/opencode TEMP=/tmp/opencode PLAYWRIGHT_BROWSERS_PATH="$PWD/.runtime/installed/browsers" .runtime/installed/node/bin/node --test --test-concurrency=1 .runtime/installed/worker/test/*.test.mjs` | **8 passed, 0 failed**, 6.541 s in final installed run. |
| `cargo run --locked --bin regressee-spike -- tiles artifacts/tall.png > artifacts/corrections-tiles.json` | PASS: a newly published generation, 1,440×20,492 → **41 tiles**. |
| `python3 scripts/evidence.py --tiles artifacts/corrections-tiles.json` | PASS: actual PNG CRCs/final chunks, metadata hashes/dimensions, identical public bytes; tile height sum 20,492, maximum 512. |
| `bash scripts/build-ui.sh` | PASS: current native binary built; existing private linker names only. |
| `python3 scripts/ui-smoke.py` | **FAIL**: PID-owned window `map_state=0`; input/capture/cancel/render checks **NOT RUN**. It now fails before unsafe focus instead of X11 BadMatch. |
| `python3 scripts/ui-smoke.py --test-map` | **FAIL**, explicitly conditional on test-only mapping: XTest keyboard delivery produced no fresh cancellation completion. This does not demonstrate a cancellation-code failure or an input pass. Capture/tile render, wheel and idle-login native checks **NOT RUN** after this failed prerequisite. |
| `python3 scripts/qualify.py --audit-only` | **PASS**: `live_private_processes: []`, `scratch: []`, `setup_paths: []`; scans both managed source/installed runtimes and project GPUI/CLI executables. |

The first aggregate run also passed 10 outcomes; a final rerun included the new
freshness/CRC evidence assertions. Earlier 9/10/11-test iterations and 6/8-worker-test
iterations were development checkpoints, not replacements for the final results.
Native attempts stopped after the two bounded probes; there is no native success
claim. The final rebuild includes later nonvisual helper changes; the conditional
initial native screenshot is only initial-state evidence, not a current tile golden.
The final harness also requires a fresh app-written activation marker before active
Cancel, so idle Escape cannot masquerade as successful capture cancellation. This
additional assertion and per-native-attempt final audit await fresh native execution;
only compilation/static checks were available after the two failed probes.

### Final aggregate timings / resource limits

Final `corrections-qualification.json`: headed smoke 23.356 s; protocol suites
87.733/92.954 s; setup-cancel process including monitor grace 0.559 s; damage
rejection 17.581 s; repair 38.906 s; actor suite 35.041 s (including two full
runtime verifications). Browser summed RSS peaked at **1,944,715,264 bytes** in
headed smoke, not PSS or a scale qualification. Uncancelled full runtime verification
still takes seconds; the fix is observing cancellation during it, not eliminating
verification. Filesystem syscalls, cleanup scans and platform launch still have
coarser cooperative boundaries; hostile/hung filesystem latency is not qualified.

Same-machine headless public captures are byte-identical, with SHA-256
`f3882ad0cb97f69b20bf5758ab3e4455741de23e2f2fe5b8ff685242c9b2c7e8`.
Current public/replacement originals were visually inspected: raster placeholders
replace raster content; fitted box/spacing and SVG icon remain. **Headed versus
headless byte inequality is still observed**, not explained or fixed; earlier pixel
counts remain historical, not a new measurement. Cross-OS equivalence is NOT RUN.

## Evidence preservation / remaining limitations

Before regeneration, **40** original PNG/JSON/log files were copied byte-for-byte to
`artifacts/history/pre-corrections-1791490783127977154/`, with
`snapshot-hashes.json`. That includes original implementation and independent
verifier screenshots/metadata. Existing `qualification.json`, independent reports,
verifier-native images and durable `00-linux-evidence.json` were not rewritten.
New aggregate output is `artifacts/corrections-qualification.json`; actor, tiles,
image inspection and default/conditional native failure evidence use the
`corrections-*` prefix. Standard synthetic backend fixture outputs are regenerated;
their original bytes remain in the snapshot. No baseline-history deletion occurred.
Final Python AST checks passed for all four maintained scripts. A final byte audit
confirmed all 40 snapshot files match their recorded hashes and maintained/builder/
installed worker src/test files agree. The inspected empty protocol-test artifact
directory was removed with `rmdir`; no unknown verifier files were deleted.
Final process/scratch/setup audit is empty (`corrections-final-audit.json`).

Remaining failures are substantive: ordinary native mapping/input/render qualification
is still blocked here; wheel, human login → ready → private capture, dialogs, native
menus and accessibility remain unqualified. Other OS process trees, clean installers,
default GPU, Linux packages, secure persistent sessions and a current supported,
trusted automatically distributed runtime require external machines/decisions.
No installer power-loss recovery, malicious-image fuzz campaign, near-limit memory,
page-scale soak or GUI latency claim was added. Raw ZIP64 packs are intentionally
rejected. No Phase 1 feature, new visual system or full product foundation was built.

## Fresh independent re-verification handoff

Do not accept this document as independent verification. Read the unchanged
`00-verification.md`, then review the correction paths/tests and recreate the
reported malicious/invalid inputs independently. Run the table's final commands
serially, preserving current evidence before regeneration.
The old temporary native driver imports helper modules in a different way, and its
tile probe assumes root-level tile names; adapt those drivers to the current helper
imports/returned generation paths rather than counting script incompatibility as
a product failure. Their original inputs and observations remain useful.

In particular:

1. Reject missing IEND/final CRC/truncation after an existing displayed generation;
   compare old **tile bytes/paths**, not just counts. Check cancellation mid-decode.
2. Time user activation → cancellation through installed verification and actual
   idle login/browser cleanup. Review native Quit's joined actor and the observed
   launch-boundary test. Independently audit live processes and scratch after each.
3. Re-sign synthetic bad manifests using only the documented development key;
   test ancestor/control links, unexpected files and duplicate **raw** ZIP names.
   Assert precise causes and no launch, not any error.
4. Check null/primitive/array requests, malformed UTF-8/JSON, incomplete/oversized
   framing, actual exit-86 crash and typed UnknownCommand. The framing/openLogin
   test covers active dispatch/cleanup, not proof that Chromium actually spawned;
   the actor quit test observes the actual owned launch.
5. Independently serve broken images/fonts/resources and a page without `#geometry`
   containing actual replaceable content. Inspect warnings, nonempty rectangles,
   pixel changes, skipped causes and null geometry when measurement is out of budget.
6. Run ordinary native harness first. `--test-map` is a disclosed verifier-only
   workaround, not native-window qualification. Investigate real input delivery;
   no wheel/menu/dialog/cancel/render pass may follow from dispatch alone.

Do not start Phase 1 or close the full gate without real external qualification.
