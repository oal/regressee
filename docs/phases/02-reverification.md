# Phase 2 — independent scratch/supervisor re-verification

Date: 2026-10-09. Fresh independent GPT-6.1 Sol verifier; **bounded Phase 2
safety-dependency recheck only**. No implementation changes, nested agents,
Phase 3 work, native retry, public browsing, sandbox bypass or publication.

## Verdict

**The inherited-long-TMPDIR P1 is fixed and independently verified locally. No
material local safety issue remains in this bounded recheck. Phase 3 may rely on
the repaired Linux supervisor with its validated short private namespace.**
Invalid installation overrides deliberately fail closed; this is not a promise
to launch under unsafe paths or on unqualified operating systems. Decision 0005
permits dependent local continuation, not full Phase 2/release acceptance.
Native/human/public-runtime/Windows/macOS/clean-machine gates remain pending.
The original failure and its limited SIGTRAP cause certainty remain unchanged in
`02-verification.md`; this operational repair does not prove its socket-limit cause.

Read both Phase 2 reports, decisions 0005/both 0006/0007, current scratch,
supervisor, recovery, distribution resolver, worker adapter and actual maintained
assertions. Earlier core/product/storage/viewer verification is retained, not
rerun as a broad UI audit.

Fresh retained evidence root (`E` below):

`/tmp/opencode/regressee-p2-reverification-9b438e4f52d0/`

## Actual serial execution

All builds/tests/browser commands ran serially from the repository root, with
four build jobs and one Rust test thread. Used the retained development signed
runtime read-only; no new pack download or signer. Every real browser used the
unchanged sandbox-enabled product adapter and only owned literal-loopback content.

| Command/scope | Actual result |
| --- | --- |
| `cargo test --locked -p regressee-runtime scratch -- --nocapture --test-threads=1` | **3 focused tests PASS**, exit 0, 0.729 s |
| Locked app `retained_worker_long_and_short_tmpdir_regression --ignored --nocapture --test-threads=1` | **PASS**, exit 0, 69.265 s; fresh long/short inherited environments, actual capture/cancel/worker-and-app SIGKILL/recovery/surviving-browser recapture |
| `bash scripts/check-foundation.sh` | **PASS**, exit 0, 7.358 s; locked UI-inclusive build, format **check**, **60 Rust tests**, all-target/all-feature clippy `-D warnings` |
| Locked app `retained_runtime_relative_supervision_smoke --ignored --nocapture --test-threads=1`, fresh long TMPDIR, **no scratch override** | **PASS**, exit 0, 16.988 s; actual signed relative-root sandboxed handshake/screenshot/shutdown |
| Private Node `--test worker/product/test/protocol.test.mjs` | **4 tests PASS**, exit 0, 0.063 s |
| Independent path-dependency `cargo build --locked --offline` | **PASS**, exit 0, 14.565 s; lock contains only **173 existing product registry tuples**, no new versions |
| Independent `probe default`, long / short TMPDIR, no override | **Both PASS**, exit 0, 15.999 / 17.051 s; fresh relative-root actual captures, marker/path/mode checks, shutdown and inherited-original preservation |
| Independent `probe safety` | **PASS**, exit 0, 17.304 s; 11 exact-cause parent rejections, independently authored recovery entries, outside-data and unrelated-process preservation |
| Strengthened independent default probes, after locked/offline rebuild | **Both PASS**, exit 0, 16.941 / 14.911 s; actual Chrome descendant observation and post-stop PID state assertions, with fresh captures |
| Signed retained private Node `--test` installed `protocol.test.mjs` and `capture.test.mjs` | **6 tests PASS**, exit 0, 0.230 s; current source/test bytes match the signed worker |
| Independent preservation/PNG/process audit | **PASS** before authorized documentation writes and after them; see `E/post-doc-audit.json` |

Exact command arrays, environments, timing and `/usr/bin/time -v` logs are in
`maintained-commands.json`, `independent-commands.json`, `proctree-commands.json`
and `signed-worker-suite-command.json`. The heavy signed HTTP setup/repair flow
was **not repeated**: this Rust-only repair did not alter runtime delivery,
worker JavaScript, pack bytes or image/analysis code.

Two verifier setup/observation limitations are preserved, **not counted as passes**:

- Offline `cargo metadata` exited 101 on uncached `arbitrary 1.5.0`. The generated
  lock introduced no registry versions; the actual locked/offline Linux build
  succeeded without downloading it. See `independent-lock-resolution.*`.
- An additional exact-browser-argv0 observation assertion exited 1 after a
  successful real handshake; it did not find Chrome immediately in that scan.
  This was an observational verifier assumption, not reproduced product launch
  failure. Its exact scan omission was not diagnosed. The subsequent bounded
  worker-descendant `/proc/stat` observation found **seven live Chrome PIDs** in
  each environment, recorded their full command lines, and required every PID
  absent or zombie/reaped after stop. No `--no-sandbox` or
  `--disable-setuid-sandbox` argument was present. Initial source versions/logs
  remain in `E/probe/src/main.v{1,2}.rs` and `browser-audit-*`.

## Independent behavioral findings

### Exact original failure, without an override

The independent probe calls the **current public Rust signed resolver and Worker**,
not a synthetic executable, copied implementation or test-name proxy. It resolves
the complete signed inventory, supplies a genuinely **relative runtime root**,
and launches with a **194-byte inherited TMPDIR** and then a **25-byte** short one.
Both use default `/tmp/opencode/rg2-1000` and a **31-byte** unique worker path,
rather than the inherited root. The v3 marker contains the matching UID, exact
scratch, parent/worker PIDs and canonical executable/browser root; namespace and
marker modes are 0700/0600. Handshake verifies Node 22.20.0 / worker 2.0.0 /
Playwright 1.56.1 / Chromium 141.0.7390.37.

Both capture the independent owned page as an actual **1440×900, 5,990-byte PNG**,
SHA-256 `592a84f761bcd5e8f7910fbebe6f003c2c346c364f1b4233336e5737f838ff12`.
The strengthened repeat has identical bytes. Metadata hash/size/dimensions,
every PNG chunk CRC, IEND-at-EOF and complete decompressed scanlines were checked
independently. Captures are stable with no warnings. Stop removes only the unique
scratch, and inherited `rg2-unrelated/keep` remains byte-preserved.

### Active cancellation, worker/app death and another live browser

Read and freshly executed the maintained assertions, rather than accepting their
names: cancellation occurs **after real capture progress** and requires exactly
`WorkerFailure(Cancelled)`; worker SIGKILL likewise follows progress and requires
exactly `WorkerFailure(WorkerStopped)`. Drop reaps its worker and removes scratch.
A separate actual app parent is SIGKILLed/reaped only after handshake establishes
a real browser; the next Worker spawn recovers its dead-owner scratch in the
**same explicit namespace**.

Another live owner is preserved throughout all paths. More importantly, its
browser **recaptures an identical actual PNG afterward**, not merely a surviving
Node/crashpad count. Both inherited cases retain the 10,664-byte original and
identical survivor PNG; metadata/CRC/scanline/hash assertions pass. A `/bin/sleep`
with the same scratch env survives cleanup/shutdown, then is explicitly reaped by
the test that created it. Final namespaces have zero worker entries.

The maintained `/proc` helper counts readable executable/environment items;
sandboxed Chrome may be absent from that count. I did not treat its `>=2` assertion
alone as full browser-PID accounting. Independent descendant checks above and
the final read-only `/proc/stat` audit supplement it: **zero live Chrome/chromium
processes** remained, as well as zero owned executable/argv0 candidates.

### Unsafe namespaces and recovery cannot authorize outside cleanup

The independent public-Worker probe asserts specific diagnostics for **11 cases**:
relative/root/over-budget/non-normalized parent, final/ancestor/dangling symlink,
regular file, non-0700 namespace, unsafe writable ancestor and actual foreign-owned
`/etc`. No fallback/worker launch occurs. Outside symlink targets and ordinary files
remain unchanged; dangling targets and rejected traversal children are not created.
No privileged ownership changes were used.

In a fresh private selected namespace, independent markers exercise legacy scope,
foreign UID, wrong scratch path, permissive directory/marker modes, unknown fields,
live owner, unmarked directory and linked marker, plus a symlinked entry pointing
to outside retained data. All are preserved byte-for-byte/link-for-link by recovery.
A **valid dead v3 entry in a different namespace** is also preserved, proving the
selected namespace, not inherited TMPDIR or other private parents, is scanned.

Valid dead entries are removed, but a live `/bin/sleep` referenced as the worker
with the same TMPDIR **survives** because its executable is different. A separately
created signed private Node with the same runtime executable and a **TMPDIR with
the scratch path plus a suffix** also survives: full environment-item matching,
not substring matching. Those exact owned test Child handles are then reaped.
Outside original/marker bytes remain intact. Maintained Drop's invalid-marker
preservation test also passes; RAII does not bypass marker validation.

Source confirms no process-group signals in the product adapter. Cleanup/recovery
require UID, executable boundary and exact scratch env; recovery additionally
requires a valid private v3 marker and ESRCH for the parent. These results are not
protection against a malicious same-user path/PID race or power-loss qualification.

## Preservation and remaining gates

The independent hash/size/mode/UID manifest covers **12,218 pre-existing files**:
implementation/docs/locks plus the retained product flow, its originals and both
runtime generations. Before report/index writes: **zero changed or missing files**.
Afterward, only the authorized pipeline index differs among that set. The sole
pre-existing file snapshot is `E/IMPLEMENTATION_STATUS.before.md`; no recursive
history copying. The standalone probe's build lock is a dependency input, not a
history snapshot. Old verification/correction reports are untouched.

Final audit: **zero live owned app/Node/browser/probe candidates, zero live browser
comm observations, zero scratch entries**, and no remaining short/inert probe
fixtures. The default private namespace is retained empty. Only explicitly created
inert fixtures were cleaned; fresh captures/logs, signed generations, inherited
long-root sentinel and outside-evidence controls remain. No unowned process was
signalled or retained evidence deleted.

**Pending/NOT RUN:** supported arbitrary-public runtime/security provenance,
production trust/signing/hosting/support, native GPUI rendering/input/accessibility
and human visual/usability approval, Windows/macOS process ownership and packaging,
clean supported-machine/default-sandbox qualification, and remaining release gates.
Local sandbox-enabled launch does not certify Chromium's security or distribution.
The historical development pack remains **owned-loopback-only**.

**Next actor:** may start dependent **local Phase 3** under decisions 0005/0006,
carrying those gates forward. No Phase 3 implementation was started here; no full
Phase 2 acceptance or release/public/cross-platform readiness is claimed.
