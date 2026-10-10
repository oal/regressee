# Phase 7 guardian corrections — FINAL LOCAL / PHASE INCOMPLETE

Implementer: GPT-6.1 Sol, 2026-10-10. Bounded to the three findings in
`07-reverification.md`; independent acceptance and Phase7/release remain pending
and NOT ACCEPTED respectively. Original stopped-browser ownership, delta append,
and signed worker/Product passes remain attributed to their original evidence.
Historical reports, runtimes, CAS, candidates and corpus are preserved. The original
candidate is STALE for corrected source. No fresh full soak is authorized here.

## Stages / checkpoints

0. SETTLED: allocate unique physical owner-private evidence/TMPDIR and short
   worker parent; snapshot only specifically changed files; read actual contracts.
1. SETTLED / LOCAL PASS: complete post-fork async-signal-safe guardian path and tracing regressions.
2. SETTLED / LOCAL PASS: trusted launch-bound settlement and fail-closed recovery regressions.
3. SETTLED / LOCAL PASS: full original/derivative hashes at explicit compaction boundaries.
4. SETTLED / LOCAL PASS: serial focused independent probes, authenticated worker faults, bounded
   differential/performance checks, locked workspace tests/UI build/clippy/fmt.
5. SETTLED / LOCAL PASS: scoped privacy/source/history/CAS/resource audit and fresh handoff.

Historical ResourceFailed root cause remains OPEN. Native/macOS/accessibility/GPU,
ABI/current-public-runtime/real-keystore/human/licensing gaps remain explicit;
passing local fixtures are neither diagnosis nor release acceptance.

## Checkpoint 0

Evidence: `/path/to/private-tests/phase7-guardian-c2f405cfcd9a4a159b2f2f8c0ff093e9`.
TMPDIR: its `tmp` child. Short physical worker parent:
`/path/to/private-tests/g7-c2f4` (40 bytes). Owner-private creation,
affected-only before snapshots retained: four implementation files and two status
documents in `preflight.json`, then two paired test files in `additional-before.json`.
Read both Phase7 reports, all requested decisions, guardian/supervisor/scratch,
capture/checkpoint assertions and actual independent guardian probe. No old tests
were repeated as diagnosis. The first compile-only command passed (1.297 s),
using the prior verifier's rebuildable Cargo target cache; retained evidence,
runtime generations and stores were not edited. Further builds use the workspace
Cargo target, not retained verifier outputs.

## Checkpoints 1 / 2 — guardian and recovery focused checks settled

Linux no longer uses `Command::spawn`/`pre_exec`: BOTH forks are raw Linux clone
syscalls without CLONE_VM, atfork or libc errno TLS. Parent prepares absolute
executable, argv/env/cwd CStrings/pointer arrays, descriptors, token and buffers.
The child never returns/unwinds to parent Rust. All inherited handlers are blocked
in the forking thread; guardian keeps them blocked, Node resets dispositions and
restores the original mask before raw execve. Fixed-buffer raw syscalls cover
descriptor duplication/closure, birth registration, errors, time, poll, proc reads,
waitid(P_PIDFD), pidfd signals, wait4/ECHILD, receipt write/fsync and exit. Parent
has an exact unreaped child/pidfd wait handle and bounded exec-error reporting.
No child allocator/formatting/locking/destructor or errno TLS is used. Fixed memcpy/
zero-initialization is allowed async-signal-safe memory work, not a libc-lock claim.
The implemented raw ABI is x86_64/aarch64; only x86_64 was executed here.

Maintained tracing registers prepare/parent/child callbacks, proves invocation in
a harmless libc control, and proves zero additional calls in multithreaded raw
launch. Separate original verifier-module probes (only launch invocation adapted
to the new API) PASS: atfork, linker-wrapped malloc/calloc/realloc/free/aligned
allocation/mutex lock/unlock/fork/errno-TLS guard, receipt fd CLOEXEC, control EOF,
setsid/double-fork descendants, exec-error cause, initially closed stdio0–2 and
orphan adoption BEFORE stop. No intentionally locked/hanging process or rescue.

Recovery v5 uses a separate private `.launches` journal, created/synced BEFORE
fork, with full launch/path/UID/boot/device/inode/generation binding and a SHA256
commitment to a private unpredictable guardian-held preimage (not the marker UUID).
Guardian writes its actual PID/kernel birth to a separately preopened CLOEXEC file
before Node clone. Marker updates must match this independent registration. Only
kernel ECHILD publishes the48-byte identity+preimage receipt; its identity must also
match the registration (including marker-worker-zero crash recovery). Receipt + exited guardian + complete binding
permit scratch removal, followed by exact journal retirement. Unregistered,
unknown/legacy/foreign/corrupt/copied/boot-wrong records refuse BEFORE spawn with
cause and original bytes retained; the whole namespace's browser budget is
conservatively reserved. Missing/orphan/incomplete journals also refuse. Recovery
NEVER signals. A crash between scratch removal and journal retirement conservatively
requires reviewed recovery; it is not silently discarded.

**Trust boundary:** the journal is independent application-created launch state,
not untrusted scratch metadata. SHA256 is a vetted commitment, NOT a custom signature
or filesystem-UUID authentication. Ordinary single-record corruption/copy cannot
change the independently registered bindings or produce a new launch's preimage.
Coordinated malicious same-UID rewriting of journal AND marker/receipt, ptrace/
process-memory access, hostile kernel/loader and malicious filesystem races are
NOT protected/claimed. No persistent signing key, plaintext session fallback or
OS-keystore qualification is introduced. The unpublished preimage is zeroized in
parent memory and never emitted in diagnostics/logs/export.

Focused signed worker5 Linux faults **2 PASS,52.750 s**: stopped main/reader,
worker crash, active cancel, Drop, app SIGKILL and acknowledged launch race;
same-TMPDIR sleep and other actual browser's identical recapture preserved.
Two naturally exited guardians adopted/reaped; zero forced/remaining children.
Exact original stopped-browser probe **PASS,19.372 s**, no rescue. Original eight
independent recovery negatives **8/8 PASS,11.291 s**, exact marker bytes retained;
private scaffold writes inert forged bytes rather than invoking launch registration.
Maintained two genuine kernel-launch positives plus14 binding/receipt negatives PASS,
including genuinely copied settlement and pre-update marker/birth tampering.
This is implementation reuse of independent probe sources, not independent acceptance.

Initial runtime run failed an obsolete fake-settled test setup (`initial launch
already registered`); retained, corrected to refusal assertions. Compile-only private
storage linkage initially selected host serde instead of the target graph; retained,
corrected only in probe linkage. Neither failure launched unsafe/rescued children.
Three pre-retirement inert unit-test journals were moved intact into evidence;
none was deleted or credited as product recovery.

## Checkpoint 3 — compaction focused checks settled

`checkpoint_inner` now loads a fully hashed previous run before ANY compacted
record/head/ref changes. `load_run` hashes each distinct original/thumbnail/tile
once per boundary, with no across-call cache. Per-target append remains unchanged:
only changed rows/new pins, no historical full scan. Maintained48 negatives cover
pause/resume/finish/cancel × original/thumbnail/tile × same-size bytes/missing/
replacement/symlink, with exact persisted runs/heads/deltas/refs/project rows unchanged.
Existing strict PNG/frozen metadata/publication and immutable history remain intact.

Independent public-API differential **700 boundaries at100/200/400 PASS**, including
exact refgraphs, unchanged raw snapshots during append, reopened overlays/pause/
resume/finish, foreign Store/stale/duplicate claims, interrupted/cancelled evidence
and non-PNG activation refusal. Same-size original compaction refusal PASS (the
original probe's resume branch is conditional; direct resume coverage is in the
48 maintained negatives, not falsely attributed to that conditional branch).
Bounded differential command37.608 s; this is synthetic storage evidence, not
browser scale/native/UI latency. No new full soak started.

## Checkpoint 4 — final focused/build checks settled

Final signed fault suite after namespace serialization: **2 PASS,52.413 s**;
two natural exited guardian adoptions, no forced/remaining. Fresh exact unchanged
original P1 probe linked to settled source **PASS,19.573 s**, no rescue. Final eight
negative actual `Worker::spawn` probes **8/8 PASS,10.944 s**, exact bytes/cause retained.
Final independent48 hash refusals **PASS,1.008 s**, including same-size NEW-INODE
replacement, missing and symlink at ALL four boundaries and ALL three artifact roles.
Actual committed-delta abrupt `_exit(77)`/reopen/interrupted/resume/cancel probe
**PASS,0.053 s**, no signals/destructors/compaction credited as crash proof.

Final registered guardian linker guard **PASS,0.016 s**: no inherited atfork,
allocator/mutex/fork/errno-TLS calls;48-byte receipt PID/birth matches independently
observed kernel stat and separate registration. Six descriptor/adoption/exec-error
contract cases remain PASS. Node inputs unchanged; authenticated worker5/35 Node
passes remain independently attributed, not redundantly rerun as diagnosis.

Final actual Product analysis/composite/restore/preview/baseline/check **1 PASS,
117.928 s**,240 s bound, four natural exited Node-suite adoptions, no intervention.
Three failed attempts remain FAIL, with their partial private stores and logs:

- `product-final` (26.265 s) and `product-lifecycle-final` (24.567 s): strict recovery
  correctly refused the test's unmanaged/unmarked Node-suite directory INSIDE the
  Product namespace. Initial concurrency hypothesis was not this observed cause.
- A bounded private namespace flock now serializes launch/recovery/cleanup across
  cooperating threads/processes, with parent-only nested-drop reentrancy and15 s
  lock bound. It prevents the source-audited enumerate/remove race; that race was
  NOT independently manufactured/credited. Child closes inherited lock descriptors;
  it never invokes the parent TLS/locking code.
- Only the unmanaged Node-suite fixture was moved outside the Product namespace
  into a separate short physical0700 directory. No production unknown-marker
  bypass or Product/profile/capture assertion was weakened.
- `product-separated-final` (40.275 s):0 captures /2 truthful CaptureFailed targets,
  because the command's optional trace destination had not been created. Source
  requires an existing private directory; fixed ONLY the private command setup.
  This is not the historical ResourceFailed readiness cause and is not suppressed.

Latest locked workspace: **143 PASS /20 deliberate ignores**,18.025 s. UI-inclusive
build1.906 s, fmt0.301 s, all-target/all-feature clippy `-D warnings`1.350 s PASS.
Two earlier clippy style failures (collapsible-if and numeric Option question-mark)
are retained, not removed/relabeled. The final stronger replacement-only test edit
was included in these final handoff rechecks. No runtime/browser/test remains running.

Delta-only finite100/200/400 probe:1 PASS; exact100/200/400 rows/validations/
serializations,300/600/1200 new pin checks. Independent differential delta time
0.088/0.186/0.363 s vs legacy0.830/3.131/12.052 s. Combined-oracle IO includes BOTH
paths and repeated load/refgraph checks; process wchar is not exclusive physical
device traffic. No browser throughput/native latency/guardian RSS inference.

**No new full soak:** the prior3000-target implementation pass is historical;
the independent verifier's unused ONE allowance is not consumed by this task.

## Checkpoint 5 — final scoped audit and resource settlement PASS

`final-audit-command` exited0 in **2.047 s**, within240 s, with no signals,
adoptions, forced children or remaining children. `final-audit.json` records:

- All135 baseline implementation inputs checked; exactly SIX code/test deltas,
  zero unexplained changes. Final affected-only snapshots are in `after/`.
  Worker bytes, dependencies, migrations, lockfile and toolchain are unchanged.
- All46 historical reports and6026 historical runtime files unchanged; current
 6028-file DevelopmentLoopback runtime signature/bytes/modes and13 maintained
  worker-source hashes verified. No new fixture private key, pack or candidate.
  Historical candidate digests are unchanged; it remains **STALE** for this source.
- All109 fresh private SQLite stores pass integrity/FK and exact reconstructed
  target/capture/profile/dimension/provenance/reference-graph checks; zero pending
  GC/local sessions.315 CAS objects are represented.97 hash-negative stores each
  retain exactly ONE deliberately damaged/missing/symlink object, including the
  original differential corruption fixture. These are expected refusal evidence,
  not repaired/deleted corruption or successful captures. Other objects match.
- Successful `product-ready-data`:6 completed runs,3 baselines,4 analyses,
  18 intact CAS objects /186059 B, no pending deltas. The three failed Product
  stores remain retained, including the CompletedWithErrors zero-capture run.
- Eight historical stores, including both failed soaks and the sole successful
  implementation soak, have intact SQLite/FKs and all19349 CAS hashes/sizes
  (291001631 B). No historical store was opened writable by the audit.
- Nine private typed traces are bounded to65536 B /128 events each and contain
  no URL/path/query/cookie/secret/stack fields. Raw images/stores are private
  evidence, NOT redacted exports; ignored regions still do not redact pixels.
- Physical ext4 evidence/TMPDIR/40-byte scratch paths remain owner-private0700.
  Scratch has zero worker entries or launch records; only the empty `.launches`
  journal directory and zero-length `.launch-lock` remain. No unknown evidence
  was silently removed or credited as recovery.

The audit enumerated69 prior bounded commands, including SEVEN retained failures;
its own settled wrapper is the **70th**. Across all70:22 naturally exited adopted
children reaped, **zero deadlines, forced children or remaining children**, and
recorded command roots no longer have the same kernel birth. Natural adoptions
are not interventions. `handoff-settlement.json` checks the completed audit wrapper,
final source freeze and documentation hashes without rerunning behavioral suites.
No process-name/global kill, pidfd rescue, privilege, install, publication or new
soak was used. The documented test faults remain exact-owned fault injection.

Final source manifest: `source-final.json`,135 inputs, SHA256
`bc98263717ad44196b3f16245dde5dfccd944781829a9eaa9edbf204fdb064b6`.
Documentation is deliberately outside that implementation freeze; final document
hashes are separately recorded in `handoff-settlement.json`.

### Exact changed-file scope

| File | Bounded correction |
| --- | --- |
| `crates/runtime/src/linux_guardian.rs` | Both raw forks/exec/error/cleanup, exact child handle, registered identity receipt, atfork regression |
| `crates/runtime/src/scratch.rs` | Independent v5 launch journal/commitment/binding, conservative cleanup and bounded lifecycle lock |
| `crates/runtime/src/supervisor.rs` | Raw-launch integration, fail-closed namespace recovery/budget, genuine settlement controls |
| `crates/storage/src/capture.rs` | Full old-CAS boundary hashing, per-call hash deduplication, atomic refusal regressions |
| `crates/runtime/tests/linux_ownership.rs` | Paired receipt assertion36→48 bytes; other ownership assertions unchanged |
| `crates/app/tests/product_flow.rs` | Only unmanaged Node-suite scratch moved outside strict Product namespace |

Intentional documentation changes: this NEW report, `docs/IMPLEMENTATION_STATUS.md`
and the current handoff section of `docs/ACTIVE_WORK.md`. Historical preceding
reports/status text remain attributed and preserved. Private scripts/probe sources,
logs, failed outputs and before/after snapshots are retained in the evidence root.

## Fresh independent-verifier handoff

This is implementation evidence, **not independent acceptance**. Read final
`07-reverification.md`, this report and decisions0005/both0006/0010. Independently
review both raw forks and every child/error/exec path, signal masks/dispositions,
CLOEXEC/control EOF/closed stdio, kernel `waitid(P_PIDFD)` authority, adoption and
bounded settlement. Do not substitute `/proc` ancestry or UUID equality for kernel
ownership/receipt provenance. Audit journal creation/birth registration/commitment/
receipt ordering, boot/device/inode/path/UID bindings, unknown-record preservation,
exact causes and conservative eight-browser budget. Explicitly evaluate the stated
non-malicious-same-UID trust boundary rather than claiming filesystem authentication.

Recheck old original AND thumbnail/tile bytes at pause/resume/finish/cancel, including
same-size NEW-INODE replacement, missing and symlink specimens. Refusal must preserve
records/heads/deltas/refs/project pointers atomically. Independently retain linear
append counters, Store/revision claims, abrupt committed-delta exit/reopen, interrupted
resume/cancel, exact refgraphs, strict PNG and immutable historical evidence.

Recipes below are **unexecuted fresh-verifier commands**, not outstanding work.
Allocate NEW physical private `V` and ≤40-byte `S` under
`/path/to/private-tests`; verify ext4, real ancestors, UID/modes and length.
Create `V/tmp` and `V/traces`0700 before any launch; never reuse an evidence/output
directory. Snapshot only affected files. Set `V`/`S` to these fresh paths, then:

```sh
set -eu
E=/path/to/private-tests/phase7-corrections-37f933c15147413e8207c02498004d90
G="$E/final-generation"
export V S
export TMPDIR="$V/tmp" CARGO_TARGET_DIR="$V/target"
export REGRESSEE_WORKER_SCRATCH_PARENT="$S"
export LIBRARY_PATH=/path/to/Regressee/.runtime/link-libs
export CARGO_BUILD_JOBS=4 RUST_TEST_THREADS=1

# Copy ONLY the bounded wrapper, redirecting its default TMPDIR/lock to fresh V.
python3 -B - "$E/bounded.py" <<'PY'
import os, pathlib, sys
v = pathlib.Path(os.environ['V'])
old = str(pathlib.Path(sys.argv[1]).parent)
text = pathlib.Path(sys.argv[1]).read_text().replace(old, str(v))
with (v / 'bounded.py').open('x') as output:
    output.write(text)
PY

python3 -B "$V/bounded.py" --output "$V/fmt-command" --timeout 120 -- \
  cargo fmt --all -- --check
python3 -B "$V/bounded.py" --output "$V/build-command" --timeout 600 -- \
  cargo build --locked --offline --workspace --all-features
python3 -B "$V/bounded.py" --output "$V/workspace-command" --timeout 600 -- \
  cargo test --locked --offline --workspace --all-features -- --test-threads=1 --nocapture
python3 -B "$V/bounded.py" --output "$V/clippy-command" --timeout 600 -- \
  cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings

# Independently check the retained signed generation/13 worker hashes BEFORE launch.
python3 -B "$V/bounded.py" --output "$V/ownership-command" --timeout 240 -- \
  env REGRESSEE_OWNERSHIP_DISTRIBUTION="$G/distribution.json" \
  REGRESSEE_OWNERSHIP_CACHE="$G/cache" REGRESSEE_OWNERSHIP_EVIDENCE="$V/ownership" \
  cargo test --locked --offline -p regressee-runtime --test linux_ownership signed_ \
  -- --ignored --nocapture --test-threads=1

python3 -B "$V/bounded.py" --output "$V/product-command" --timeout 240 -- \
  env REGRESSEE_PHASE5_DISTRIBUTION="$G/distribution.json" \
  REGRESSEE_PHASE5_RUNTIME_CACHE="$G/cache" REGRESSEE_PHASE5_EVIDENCE="$V/product" \
  REGRESSEE_CAPTURE_TRACE_EVIDENCE="$V/traces" \
  cargo test --locked --offline -p regressee-app --features ui-tests --test product_flow \
  actual_phase5_product_analysis_composite_and_replacement \
  -- --exact --ignored --nocapture --test-threads=1

python3 -B "$V/bounded.py" --output "$V/delta-profile-command" --timeout 120 -- \
  cargo test --locked --offline -p regressee-storage \
  bounded_delta_checkpoint_profile -- --ignored --nocapture --test-threads=1
```

Run ALL commands serially, with no other runtime/browser/process suite active.
Expected maintained totals:143 PASS/20 deliberate ignores; ownership2 PASS,
Product1 PASS, delta profile1 PASS. The workspace includes the maintained raw-atfork,
recovery-positive/negative and48 compaction negatives. Verify test selection/counts;
zero tests or wrapper intervention is NOT a pass. Retain every failure and its
actual cause; do not extend bounds or relabel failed attempts.

Supplement maintained tests with fresh independent probes. Existing sources are
available read-only in this evidence root (`guardian-probe.rs`,
`registered-forbidden-probe.rs`, `guardian-contracts.rs`, `marker-probe.rs`,
`hash-probe.rs`, `crash-probe.rs`, `storage-probe.rs`, `original-probe.rs`) and the
original verifier root. Recompile against the FRESH locked target graph; do not
reuse old executables/rlib paths or run implementation preparation drivers. Original
P1/marker/hash probes used90 s bounds; differential240 s. Registered linker guard
needs the recorded `--wrap` options from its compile `start.json`; independently
check its coverage/limitations and add controls. Reusing sources is not by itself
an independent verdict. Signed Node35-pass recipes remain in `07-corrections.md`;
if independently needed, use a SEPARATE ≤40-byte Node scratch parent, never `S`.

Finish with fresh scoped privacy/source/runtime/history/CAS/refgraph/process-birth
audits and settled wrappers. Do NOT run this implementation's `audit.py` in place
as a verifier: it writes only to this implementation evidence root. Keep historical
data read-only. Receipt/preimage bytes must not be exported as diagnostics/secrets.

**Soak gate:** this repair task must NOT consume the unused verifier ONE-soak
allowance. Only a separately authorized independent verifier may use it AFTER
all repaired-source safety/hash prerequisites pass, with unchanged1200-second
capture budget, profiles/readiness/worker5 and eight-browser limit. No fresh soak,
candidate, pack, signing key, native/backend/OS/ABI/license work or publication here.

**Final disposition:** all THREE findings implemented with bounded LOCAL PASS;
all owned work SETTLED. Independent correction acceptance **PENDING**, historical
ResourceFailed root cause **OPEN**, original candidate **STALE**, Phase7/public
release **NOT ACCEPTED**. macOS/accessibility code gaps, full Windows integration,
native GPU/input/latency, ABI GLIBC_2.43, current public-runtime/trust-hosting,
real OS-keystore, human review, packages/signatures/key custody and licensing remain
open. No malicious-same-UID, aarch64 execution, native or release claim is inferred.
