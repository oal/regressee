# Phase 2 — bounded inherited-TMPDIR correction

Date: 2026-10-09. **Implementation correction; independent re-verification
pending.** Phase 2 only. No nested agents, Phase 3 features, native retry,
external publication, global installation or sandbox/browser substitution.
The original P1 and its cause-certainty limit remain in
`docs/phases/02-verification.md`; that historical report is unchanged.

## Delivered policy

- `crates/runtime/src/scratch.rs`: Linux workers ignore inherited TMPDIR/TMP/TEMP
  when choosing scratch. The production default is `/tmp/rg2-EUID`; this harness
  uses `/tmp/opencode/rg2-EUID` when that real directory belongs to the effective
  user. The complete namespace can be explicitly configured using
  `REGRESSEE_WORKER_SCRATCH_PARENT`. There is no silent fallback for an invalid
  selected namespace and no global environment change.
- Validate absolute/non-root paths, each ancestor, symlinks (including dangling
  links), directory type, effective-UID ownership, exact 0700 namespace mode and
  the **40-byte parent / 49-byte scratch** Linux path budget. Shared ancestors
  require trusted ownership and sticky semantics (the OS-managed `/tmp` also
  supports this harness's rootless UID mapping); arbitrary foreign sticky parents
  are not trusted. Otherwise ancestors must be root/current-user-owned and not
  group/world writable. Missing final namespaces and unique `w-XXXXXX`
  directories are created with 0700 at creation, not exposed then chmodded.
  Invalid parents report their specific path/cause before worker/browser launch.
- `crates/runtime/src/supervisor.rs`: spawn and recovery use the same selected
  parent. A strict 0600 v3 owner marker binds namespace, UID, exact scratch,
  parent/worker PIDs and absolute runtime executable/browser root. Marker updates
  use a synced private pending file and rename. Relative verified runtime roots
  are still canonicalized before launch and ownership matching.
- Cancellation, shutdown, Drop and recovery use that same private-directory/
  marker/path-length/entry-name policy. Even the scratch RAII destructor refuses to remove an invalid
  registered owner. Recovery reads only the selected namespace, preserves live
  owners and requires ESRCH for a dead parent. Unmarked entries, foreign/legacy
  markers, unsafe permissions and links are not migration or deletion authority.
- Linux process cleanup matches effective UID, runtime executable and the full
  `TMPDIR=<exact unique scratch>` environment item. **No process-group signals**
  remain in the product adapter. Recovery authenticates an orphan worker too;
  repeated individual-browser cleanup handles detached children/launch races.
  The direct unreaped Child handle remains the authority for its own worker PID.
- `crates/runtime/src/lib.rs` wires the Linux-only module;
  `docs/runtime-distribution.md` documents the installation policy. No dependencies,
  lock versions, runtime pins, capture JavaScript or historical evidence changed.
- `scripts/check-phase2.py` audits entries of the explicit/per-user namespaces,
  rather than incorrectly counting the persistent empty namespace as leaked
  scratch; it also includes retained workspace runtime executables. It does no
  cleanup and does not inspect a caller's inherited temporary root.

This is an operational Linux short-path repair, not proof that the earlier
SIGTRAP's exact cause was a socket limit. It is not a malicious-same-user race or
power-loss guarantee. PID reuse/permission ambiguity conservatively preserves
work. Windows/macOS product launch still fails closed; no fake portable scratch
or process-tree qualification was added.

## Maintained assertions

Three Linux runtime regressions check specific relative/long/symlink/dangling/
non-directory/foreign-ownership/mode/unsafe-ancestor diagnostics, valid private
creation, conservative recovery of live/foreign/unmarked/linked entries, and
Drop refusing an invalid registered marker.

`crates/app/tests/product_flow.rs` adds
`retained_worker_long_and_short_tmpdir_regression` and its ignored subprocess
helper. It verifies the retained signed development generation read-only, then
for **both inherited environments** actually:

1. launches the unchanged sandbox-enabled private Chromium adapter, checks its
   pinned identity and observes live browser processes;
2. captures an owned loopback 1440×900 page, checks the PNG signature/dimensions,
   and retains the actual PNG/metadata in fresh evidence;
3. cancels an in-flight `/hang` capture after real progress, asserts exactly
   `WorkerFailure(Cancelled)`, and requires scratch and live processes gone;
4. SIGKILLs the actual worker after capture progress, asserts exactly
   `WorkerFailure(WorkerStopped)` and requires Drop to reap/clean its scratch;
5. SIGKILLs/reaps a separate app parent **after its browser is live**, then starts
   a real worker to exercise same-namespace dead-parent recovery;
6. checks another live owner/browser and a `/bin/sleep` with the same scratch env
   survive (no unrelated executable/group killing), **recaptures an identical PNG
   using that other live browser after all cleanup paths**, performs actual
   handshake/shutdown, and requires zero namespace entries/live scratch processes.

The unrelated inherited `rg2-unrelated/keep` entry is byte-preserved. Test parents
are uniquely created and explicit; no global environment mutation, public URL,
new pack download or invented capture output is used.

## Serial execution evidence

Fresh retained root (`E`):
`/tmp/opencode/regressee-p2-correction-f3428259eaeb/`.
All commands are serial, build jobs bounded to four, test threads to one.
Command arrays/environments/timing and `/usr/bin/time -v` logs are retained there.

| Final command/scope | Actual result |
| --- | --- |
| `cargo test --locked -p regressee-runtime scratch -- --nocapture --test-threads=1` | **3 focused tests PASS**, exit 0, 0.696 s. No browser dependency for invalid-parent causes. |
| Locked `retained_worker_long_and_short_tmpdir_regression --ignored --nocapture --test-threads=1` | **PASS**, exit 0, 79.976 s: both actual sandboxed inherited-TMPDIR cases, capture/cancel/worker SIGKILL/app-parent SIGKILL/recovery/shutdown, surviving browser recapture and unrelated process preservation. |
| `bash scripts/check-foundation.sh` | **PASS**, exit 0, 8.212 s: locked UI-inclusive binaries, format check, **60 Rust tests**, all-target/all-feature clippy `-D warnings`. Four heavy/helper integration tests plus the retained-PNG test remain ignored in this ordinary batch; the relevant integrations were explicitly run above/below. |
| Locked `retained_runtime_relative_supervision_smoke --ignored --nocapture --test-threads=1` with a fresh >108-byte inherited TMPDIR and **no scratch override** | **PASS**, exit 0, 15.694 s: retained signature/inventory verification, relative-root launch, actual sandboxed screenshot handshake, shutdown through the default `/tmp/opencode/rg2-1000` policy. |
| Private historical Node `--test worker/product/test/protocol.test.mjs` | **4 tests PASS**, exit 0, 0.068 s. |
| `python3 scripts/check-phase2.py --output E/checks-only-audit --checks-only` | **PASS**, exit 0, 7.273 s: actual three serial constituent commands, repeated 60 Rust/4 Node checks and maintained updated namespace accounting. No HTTP pack, browser or native repetition. |

Exact arrays/environments are in `E/commands-complete.json` and
`E/checks-only-audit-wrapper-command.json`; complete logs use `*-complete.log`.
Earlier passing batches (`commands-v3.json`, `commands-latest.json`) are retained.
The final batch also tests the added exact worker-crash error and functional
surviving-browser recapture. No source changed after that batch except the
read-only audit script and authorized documentation; its audit change was itself
exercised by the checks-only wrapper.

`E/browser-regression-complete/{long,short}/` contains child/crash logs, actual
capture PNG/metadata, identical post-cleanup survivor PNG, crash-ready PID/path
records and result JSON. Both captures are **1440×900 / 10,664 bytes**, with stable
screenshots, no warnings and the same actual SHA-256
`bf27889fba36ea11b8c9e9360e4d3381b9316fb544b5bbe296629f032edd00d9`.
The inherited long path is **214 bytes**, the short path **23 bytes**; their
private namespace parents are **23 bytes** and worker scratch paths **32 bytes**.
These are owned fixture captures under the unchanged Node 22.20.0 / Playwright
1.56.1 / Chromium 141.0.7390.37 signed development identity, not public targets.
The post-run `E/retained-capture-assertion-audit.json` additionally checks both
actual saved files' dimensions, length, metadata SHA-256, every PNG chunk CRC,
IEND-at-EOF and equality with the surviving browser's post-cleanup capture.

The final browser-regression command reports **875,016 KiB max RSS**, 69.91/3.34 s
user/system CPU (including compilation). Relative smoke: **183,604 KiB**, 14.59/
0.66 s. These are command maxima, not sampled concurrent Rust/Node/Chromium totals
or a performance qualification. The full 304,884,626-byte HTTP flow and native
attempt were **not repeated**: distribution/capture JavaScript, runtime bytes and
image/metric code are unchanged; their prior actual results remain historical.

Final audit: **13,551 pre-existing document/lock/historical-evidence files
checked; zero missing or unexpected changes**. Only the explicitly updated
status/index/runtime-interface documents differ among that preserved set.
**Zero live owned app/Node/browser/test candidates, zero worker scratch entries,
zero marked legacy scratch and zero pending setup/publication paths.** The default
0700 namespace is intentionally retained empty, not counted as a leaked worker.
No audit kills or deletes anything. See
`E/final-resource-preservation-audit-complete.json`, `E/final-source-sha256.json`
and the maintained wrapper's `checks-only-audit/resource-audit.json`.

Early attempts are preserved: an accidentally bounded 1-second background runner
timed out during compilation; no test/browser had run and no compiler remained
before retry. Initial focused tests exposed tempfile's non-private default mode;
creation now explicitly requests 0700 (including test namespaces). An initial
integration compile caught an invalid test TargetKey field; corrected before
browser launch. These are not counted as passes; their original logs/command
records remain in E.

## Fresh independent verifier handoff

Read the original P1, decisions 0005/both 0006/0007, this correction and actual
scratch/supervisor/test assertions. Reuse the signed retained pack read-only or
build a fresh trusted fixture if desired. Use a **new** evidence path, not E.
Run serially from the repository root:

```sh
export LIBRARY_PATH="$PWD/.runtime/link-libs${LIBRARY_PATH:+:$LIBRARY_PATH}"
export CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=1
cargo test --locked -p regressee-runtime scratch -- --nocapture --test-threads=1
REGRESSEE_PHASE2_RETAINED_FLOW="$PWD/artifacts/phase2-implementation-2026-10-09-d/product-flow" \
REGRESSEE_PHASE2_CORRECTION_EVIDENCE=/tmp/opencode/regressee-p2-reverify-UNIQUE \
  cargo test --locked -p regressee-app --features ui-tests --test product_flow \
  retained_worker_long_and_short_tmpdir_regression -- --ignored --nocapture --test-threads=1
bash scripts/check-foundation.sh
# Create a fresh, deliberately >108-byte inherited TMPDIR, then:
TMPDIR=YOUR_NEW_LONG_DIRECTORY \
REGRESSEE_PHASE2_RETAINED_FLOW="$PWD/artifacts/phase2-implementation-2026-10-09-d/product-flow" \
  cargo test --locked -p regressee-app --features ui-tests --test product_flow \
  retained_runtime_relative_supervision_smoke -- --ignored --nocapture --test-threads=1
```

Inspect exact error assertions, actual retained PNGs/metadata, crash-ready records,
live-owner/unrelated preservation, invalid marker conservatism, default versus
override policy, relative runtime roots and a final scoped resource/preservation
audit. No need to repeat every 300-MiB HTTP pack case for this Rust-only scratch
delta unless independent findings demand it. Do not edit the old reports/research,
attempt native retries, disable sandboxing or claim release/public/cross-platform
acceptance. Write a separate `02-reverification.md` with the actual verdict;
dependent Phase 3 supervisor work still waits for that fresh independent check.
