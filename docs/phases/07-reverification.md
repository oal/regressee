# Phase 7 correction reverification — FINAL / LOCAL GATE NOT ACCEPTED

Independent verifier: GPT-6.1 Sol, 2026-10-10. No implementation edits,
nested agents, publication, or release acceptance.

**Verdict:** the original stopped-browser P1 is FIXED in its independently
reproduced scope; linear checkpoint work and current signed worker/Product
contracts PASS. **Combined local guardian/checkpoint correction acceptance FAILS**
on three fresh negative findings below. No fresh soak was started because its local
safety/correctness prerequisites failed. All owned checks and resources are settled.
Historical failures are not rewritten; Phase7/public release remains NOT ACCEPTED.

| Scope | Independent result |
| --- | --- |
| Original stopped real Chromium / signed fault suite | PASS: exact original probe,2 maintained signed tests, no rescue |
| Guardian post-fork syscall-only boundary | FAIL / P1: inherited pthread_atfork handler actually invoked |
| Recovery receipts / malformed current records | FAIL / P2:2/8 refusal controls pass,6 fail |
| Delta/legacy differential / refgraph / work | PASS:700 target-boundary comparisons at100/200/400 |
| Hash verification at pause/resume/finish | FAIL / P2: same-size prior CAS corruption accepted |
| Locked fmt/UI build/UI clippy/all-feature tests | PASS:139 Rust /20 deliberately ignored |
| Authenticated worker5 / actual Product flow | PASS:35 signed Node +1 full actual Product |
| Typed trace/privacy / warning-bearing guard | PASS: bounded schema negatives,9 typed files, explicit NotComparable control |
| Fresh maximum-scale soak | NOT RUN: local prerequisite failure; zero attempts |
| Final scoped source/history/CAS/resource audit | PASS, with intentional corrupt fixture identified, not laundered |

Physical owner-private evidence root:
`/path/to/private-tests/phase7-reverify-3bd2fafe83aa4fd5b622ff228f608d6a`.
Short physical scratch namespace:
`/path/to/private-tests/r7-3bd2` (40 bytes).
No compatibility symlink is a Store/runtime ancestor. Preflight/environment and
135 source SHA256 values are retained in `preflight.json` and
`source-baseline.json`; manifest SHA256
`bba2a696dc3eeacd18118b1f5aedf7fe9f6ea2e0ddff4ea39c8e41e360868576`.

## Stages / checkpoints

0. SETTLED: fresh private physical paths and source/environment baseline created.
1. SETTLED / LOCAL FAIL: independent guardian/pidfd/receipt/scratch and persistence source audit;
   inspect original P1, maintained assertions, decisions and signed worker5 inputs.
2. SETTLED / LOCAL FAIL: fresh independent bounded ownership/receipt/race/adoption probes and
   maintained signed fault suite; no rescue or wrapper intervention counts as pass.
3. SETTLED / PERSISTENCE BOUNDARY FAIL: persistence differential/claim/corruption/recovery probes and exact
   work/refgraph checks; optional typed trace/privacy and signed worker/Product checks.
4. SETTLED / PASS: serial locked fmt/UI build/all-feature tests/UI clippy, nonzero counts.
5. GATED OFF: at most ONE final-source 1000-page/3-profile soak, only after local
   ownership and snapshot correctness pass; unchanged 1200 s capture, 1500 s total,
   1560 s outer bounds. Separate guardian RSS; IO caveats remain explicit.
6. SETTLED / SCOPED PASS: final privacy/CAS/source/history/owned-resource audit and index.

Focused signed ownership/Product bounds: 240 s each; signed Node bound: 180 s.
All commands serial and owned-child bounded; retain every failure and checkpoint
after every stage. Historical reports/runtimes/CAS/candidate and failed 1708/2188
soaks are preserved. Frozen candidate is stale for changed source, not release binding.

Phase7/public release remains NOT ACCEPTED. macOS/accessibility/native GPU/ABI,
current public runtime, OS keychain, human and licensing gaps are not converted
into a blanket stop on bounded local verification. Historical ResourceFailed cause
is UNPROVED even if current Product checks pass; native UI latency is NOT RUN.

## Checkpoint 1a — independent guardian boundary FAIL, settled

Fresh locked/offline UI-inclusive build PASS (57.733 s), no forced/remaining child.
Private `guardian-probe.rs` includes the unchanged actual `linux_guardian.rs`.
In a multithreaded parent, a safe tracing-only `pthread_atfork` prepare callback
recorded both the app and its already-forked guardian calling it. The second call
comes from `linux_guardian.rs:68` (`libc::fork()`), disproving the syscall-only
post-fork claim. Arbitrary inherited handlers/libc fork internals can use locks or
allocator state; this probe deliberately does not deadlock or rescue processes.
`guardian-result.json`: guardian exit0, exact receipt, probe exit1 / FAIL, 0.004 s,
zero deadline/forced/remaining children. The ownership prerequisite for the soak
is not met; no scale attempt started. A linkage scaffold ambiguity (two libc
feature variants, before any compile/probe launch) is retained separately.

## Checkpoint 2a — signed P1 passes; recovery negative probes FAIL

Independent signature/pack/mode/source preflight PASS: 6028 installed/ZIP files,
13 maintained worker/test inputs, worker5, 129 frozen inputs, 46 historical reports;
new fixture private keys absent. Literal-loopback scope only. One private OpenSSL
CLI option typo failed before input verification finished; retained, then corrected.
Exact original P1 source freshly linked to current code PASS (19.525 s), no rescue.
Maintained signed stopped-main/Node-reader/worker-crash/cancel/Drop/app-crash/
acknowledged-launch-race and identical second-browser recapture suite: 2 PASS,
59.715 s; two naturally exited guardian adoptions, zero forced/remaining children.
This confirms the original stopped-browser reproduction is fixed in this scope,
not the post-fork safety claim.

Fresh actual `Worker::spawn` recovery probes: 2/8 refusal cases PASS; 6 FAIL.
Unproved valid and legacy records block launch and retain exact bytes. Matching
but synthetic/forged/copied receipts and an old-boot record are accepted and
deleted; foreign UID field, unknown current schema and wrong scratch binding are
retained but silently skipped, allowing a fresh worker. `marker-results.json`
retains exact causes. All inert specimens are test-owned; foreign filesystem
ownership was not fabricated/chowned. No foreign signal or live-tree escape is
inferred from these inert cases. Requirements for boot-bound, fail-closed recovery
are unmet at `scratch.rs:272-304` / `supervisor.rs:796-816`.
Probe exit1, 11.909 s, zero deadline/forced/remaining. Surviving inert markers were
moved into private evidence; the scratch namespace is clear. Two external-linkage
scaffold failures (mismatched Cargo serde feature graphs) were corrected only in
private linkage selection, not maintained source or assertions.

## Checkpoint 2b — additional descriptor/adoption checks settled

Fresh unchanged actual guardian-module probes PASS (5 cases): receipt fd absent
after exec, control-pipe EOF cleanup, setsid descendant adoption through parent
death during stop, exec-error propagation to Command caller, and stdio0/1/2 closed
before descriptor creation. Exact observed pidfds all report death, exact receipt
bytes and guardian exit0; no wrapper rescue. This does not establish safety of
every libc wrapper on the post-fork path. Source audit also finds libc `open`,
`waitid`, `waitpid`, `poll`, `clock_gettime` and errno TLS, not raw syscall-only code.
No process-group/global-app-subreaper/name/env/UID authority is used in production;
waitid P_PIDFD confirms direct children before signaling. The private command
wrapper alone is a dedicated test subreaper and validates pidfds with waitid.
An additional fresh orphan-before-stop case PASS explicitly observes the detached
grandchild reparented to the exact guardian before stop, then exact pidfd death,
receipt and guardian exit. Total6 independent positive guardian contract cases.

## Checkpoint 3a — differential persistence PASS, compaction hash boundary FAIL

Fresh independent public-API `storage-probe.rs`: 700 exact per-target comparisons
against legacy full snapshots and complete reference graphs at100/200/400 targets.
Raw `runs.record` bytes remain unchanged during overlay appends. Pause/reopened
Store/resume/finish equal legacy; terminal mutation and synthetic non-PNG activation
refused. Rows/validations/serializations100/200/400; new pin checks300/600/1200;
serialized bytes172260/344960/690360. Delta work0.103/0.180/0.367 s versus full
snapshots0.855/3.176/12.342 s; combined-oracle IO includes BOTH paths and repeated
load/hash/refgraph checks, not exclusive delta device traffic.

Foreign Store, concurrent stale revision, duplicate atomic updates, reopened old
claim, reopen after committed delta before compaction, dead-owner recovery,
captured-evidence preservation and partial cancelled activation refusal PASS.
No process crash was simulated by unchecked PID signaling: dead ownership is an
inert explicit test SQL change; reopened persisted overlay is actual.
Supplemental fresh actual abrupt-exit probe PASS (0.053 s): child commits one
delta, then `_exit(77)` skips Store/SQLite destructors and compaction. A separately
reopened Store reconstructs the exact record, recognizes its actually exited
owner as Interrupted, explicitly resumes/cancels and preserves captured bytes;
partial activation refuses. `crash-result.json` distinguishes this actual persisted
crash boundary from the earlier inert SQL-owner control. No signal/rescue used.

New same-size CAS negative: after one capture append, corrupt original bytes, then
pause, resume and finish each SUCCEED while `load_run` and activation correctly
REFUSE. `capture.rs:328,410-415` loads an unhashed snapshot and skips pins for old
captures. Full hash verification at these compaction boundaries is not delivered;
no stale verified cache or PNG-policy bypass was introduced. `storage-results.json`
retains all outcomes; overall exit1,46.110 s, zero deadline/forced/remaining.
The deliberately corrupt private fixture remains corrupt as failure evidence.
One verifier compile-only failure (CaptureEvidence lacks PartialEq) is retained;
private oracle now compares serialized bytes, with no maintained-code change.

## Checkpoints 3b / 4 — serial maintained contracts, signed worker and Product PASS

All-feature locked/offline workspace:139 PASS /20 deliberately ignored (19.697 s),
fmt PASS (0.288 s), UI-inclusive all-target clippy `-D warnings` PASS (25.584 s),
earlier UI-inclusive locked build PASS. Fresh isolated maintained delta-only profile
1 PASS at100/200/400:0.096/0.186/0.356 s, same exact rows/work counters;
process wchar increments1,664,480 /3,370,160 /7,580,608 B. These synthetic process
counters are not exclusive/additive physical device writes or browser throughput.
Migration9 preservation and maintained strictPNG/import/export/restore contracts
run with actual nonzero assertions in the workspace suite; no historical bytes
are rewritten by migration's two new tables.

Current authenticated worker5 six-file suite35 PASS (13.525 s),20 natural exited
adoptions, no forced/remaining children. Controlled positive image/picture/srcset/
CSS background cases have clean warnings; negative genuine fetch ERR_ABORTED,
missing image/script and forbidden-resource cases retain warnings. Every traced
intentional image cancellation matches the unchanged narrow replacement matcher.
Full actual Product analysis/composite/restore/representative preview/baseline/check
1 PASS (110.165 s),4 natural exited adoptions, no forced/remaining children.
Historical ResourceFailed cause remains UNPROVED; no old warned NotComparable is
relabeled. The actual warning-bearing NotComparable guard is exercised by maintained
analysis tests and unchanged `review.rs` semantics. neutral-raster-v2 identity and
transform geometry are unchanged; runtime still literal-loopback-only.

Fresh direct actual trace-decoder probes PASS:18 exact field/enum/timing/status/ID
refusals,128-event positive boundary,129-event and65537-byte refusals;9 actual
Product typed trace files decode under the bounded schema with no arbitrary
URL/path/query/cookie/secret/error-stack fields. Raw fixture stores and Node test
reports contain private URLs by design and are not redacted exports.
All of this stage's command wrappers show no deadline, forced or remaining child.
Additional actual warning control PASS (0.102 s): ONLY a fresh private copy of this
Product store received an explicit ResourceFailed metadata warning. Actual
`review::analyze_run` returns NotComparable although its raw perceptual report is
NoSignificantChange. Originals/refgraphs remain intact; this is a synthetic guard
control, not a reconstruction or diagnosis of the historical warning.

## Checkpoint 5 — no fresh soak started

The conditional local ownership and compaction correctness gate FAILED before
scale execution. Authorized allowance remains unused by this verifier; no retry,
deadline increase, large cancellation run or misleading new memory measurement.
Implementer's3000-target/900.357 s capture+activation and1120.446 s total remain
attributed historical implementation evidence, not this verifier's scale PASS.
Recorded guardian RSS0 in that attempt is NOT zero RSS: inherited test-thread comm
was grouped as HarnessBuild. /proc waited-child IO is not exclusive/additive device
traffic. Native UI latency/input/GPU/ABI/current-public qualification is NOT RUN.

## Material findings and next bounded work

1. **P1 — post-fork safety / unbounded launch hazard.**
   `crates/runtime/src/linux_guardian.rs:59-85`, especially68. The nested libc fork
   actually calls inherited pthread_atfork prepare handlers after the first fork
   from a multithreaded Rust process. Those handlers may use locks/allocator/libc
   state; a stuck handler precedes launch reporting and cannot be bounded by later
   guardian stop logic. No deadlock/live-tree escape was deliberately manufactured:
   the safe tracing callback proves invocation, not a fabricated hanging process.
   `crates/runtime/src/supervisor.rs:260-267` also contains an allocating
   `Error::other` on the post-fork parent-death branch (source-audited, not forced).
   Bounded repair should make the complete child path actually async-signal-safe,
   including fork, error construction, descriptor setup and exec-error handling;
   independently prove inherited handlers/allocator/mutex code cannot run there.

2. **P2 — recovery receipt trust and invalid-marker fail-closed boundary.**
   `crates/runtime/src/scratch.rs:237-304` and
   `crates/runtime/src/supervisor.rs:796-816`. `settled()` compares a writable UUID
   file with a UUID contained in the marker; deletion accepts absent/reused guardian
   PID plus this receipt without enforcing its boot binding. Inert stale-boot and
   manufactured matching/copied receipt records are deleted. Invalid current
   schema/UID-field/scratch-binding records are skipped rather than reserving or
   blocking unknown prior ownership. Test data has owner-private physical paths;
   no malicious same-user path-race protection, actual foreign UID chown, privilege
   escalation or foreign signal is claimed. Repair rejection/receipt provenance
   and conservative browser-budget handling; preserve unknown records and add
   exact negative tests. Genuine uninterruptible trees were not safely manufactured;
   current no-receipt retention code is source-reviewed, not a behavioral pass.

3. **P2 — compaction does not verify old CAS hashes.**
   `crates/storage/src/capture.rs:319-435`, especially328/410-415. Structural
   reconstruction and optimistic revision protection pass, but old captures bypass
   re-pinning/hashing at pause/resume/finish. All three calls accept deliberately
   same-size-corrupt prior original bytes. Load/activation refuse correctly, so
   this is not a demonstrated malformed baseline publication or stale-cache bypass.
   Restore full old-evidence verification at explicit compaction boundaries without
   reverting to per-target full-snapshot work, then recheck interrupted/cancelled/
   concurrent/refgraph/strictPNG contracts independently.

Repair these bounded local defects first, with a separate implementation and fresh
independent verifier. Do not repeat unchanged suites/soaks as a substitute. A later
scale attempt requires an explicitly authorized repaired-source prerequisite pass;
no deadline increase is recommended. Independently progressing platform/backend/
licensing work need not stop merely because external gates remain open.

## Checkpoint 6 — final audit and resource handoff settled

`final-audit-complete.json` independently checks135 maintained input hashes unchanged;
11 fresh private SQLite9 stores have intact SQLite/FKs, exact target/capture/profile/
dimension/provenance/refgraph bindings and zero pending deltas/GC/local sessions.
Both actual Product stores contain18 CAS objects /186059 B each; the original has
6 runs/3 baselines/4 analyses, the warned private copy5 analyses. Nine synthetic
stores contain one62-byte object each. The deliberately corrupted hash-boundary
fixture has exactly ONE same-size hash mismatch, retained explicitly as failure
evidence; every other audited object matches its hash/size.

Reviewed read-only retained audit PASS separately:46 historical reports,6026 old
runtime files,6028 current signed files, historical pack/candidate digests, six
historical stores and the implementer's complete soak remain preserved. The
1708/2188 failures and original observer's rescue/forced-child FAIL remain failures.
The frozen candidate is stale for corrected code; no new candidate/signature/public
runtime was created. Fresh signature/ZIP/modes/13 maintained source hashes were
independently verified BEFORE executing worker5; new private keys remain absent.

All35 bounded command results are settled, including compile/preflight failures:
zero wrapper deadlines, forced children or remaining children;26 naturally exited
adoptions reaped. Same-birth command-root exit checks PASS, short physical scratch
namespace empty. No hidden pidfd rescue, unchecked PID kill, global service/install,
sudo, TLS/sandbox bypass, publication or broad deletion. Only inert exact-owned
markers were moved to evidence; historical stores/runtimes were never edited.
Supplemental topology observers double-check birth around pidfd acquisition in
their assertion code; their raw pre-stop birth text is not separately archived.
Original P1 observer retains full executable/profile/birth evidence independently.

Intentional repository changes are this report and `docs/IMPLEMENTATION_STATUS.md`.
Orchestrator-owned `docs/ACTIVE_WORK.md` documentation is separately attributed;
no implementation/worker/migration/lock/toolchain input changed. Private failed
scaffolds remain visible; no zero-test filter is called proof. Exact commands,
bounds, exit/log/cause records are in each evidence command directory. The initial
`final-audit.json` is retained. Supplemental crash/final-audit enumeration includes
34 prior commands; its own settled wrapper is the35th. No completed behavioral
suite or soak was repeated after the interruption.

**Readiness:** bounded local correction gate NOT ACCEPTED; Phase7/release NOT
ACCEPTED. Historical readiness cause UNPROVED. macOS owned-tree/accessibility code
gaps, full Windows integration/native OS, native GPU/input/latency, GLIBC_2.43 target
ABI, public current runtime/trust-hosting, real OS-keystore, human tasks/review,
packaging/signatures/key custody and license obligations remain genuinely open.
No headless/native qualification or human/licensing approval is invented.
