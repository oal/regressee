# Phase 1 — independent correction re-verification

Date: 2026-10-09. Fresh independent GPT-6.1 Sol verifier; Phase 1 only.

## Verdict

**PASS for the three bounded local repairs. No material local blocker found in
this recheck. Phase 2 may proceed on the locally verified foundation under
decision 0005**, with the native/external/human qualification below still open.
This is not full Phase 1/platform acceptance or release readiness. Phase 2 was
not started; no implementation was changed and no agents were spawned.

Read the original verification, corrections and status reports, both decision
0006 documents and decision 0005, then inspected actual storage, wire parser/
encoder, shared fixtures, shell handlers, actor and regression assertions.
Original findings and failed native evidence remain preserved historical records.

Fresh retained evidence root (`E` below):
`/tmp/opencode/regressee-phase1-reverification-ea03f6rw/`.
It holds serial command logs/exit codes/durations, independent probe source and
lockfiles, paired wire bytes/results, headless persistence evidence, preservation
hashes and the prior pipeline-index snapshot.

## Independent reproductions

### P1 — database entries, unsafe ancestors and SQLite controls: repaired locally

Actual path dependencies call `Store::open`; these are not copied/fixed storage
functions. The independent probe tests **30 fresh unsafe layouts**:

- **16 database/control symlinks:** database, `-journal`, `-wal`, `-shm`, each live
  and dangling, for both first open and reopening an actual saved database.
- **10 directory symlinks:** data, cache, artifacts, higher data ancestor and
  higher cache ancestor, each live and dangling.
- **4 nonregular controls:** directories at each database/control entry.

Every case asserts the precise regular-file/real-directory error **and offending
path**, not an arbitrary failure. Outside-target tree snapshots compare entries,
bytes, modes, inode, mtime and ctime before/after. Outside missing targets remain
missing, existing sentinels remain unchanged, and links retain their targets.
Sidecar rejection also preserves the existing database snapshot or leaves a new
database absent. Directory preflight leaves the owned tree unchanged, not just
the outside sentinel. All cases passed; temporary unsafe-layout roots were
RAII-cleaned. No target outside verifier-owned temporary roots was used.

Source confirms `symlink_metadata` is unconditional; only genuine `NotFound`
means absence. Directory inspection is top-down and rejects parent traversal.
All three SQLite controls are checked before preparation/open/migration.
Maintained regressions also passed first-open/reopen, v1 migration through
`Store::open`, idempotence, rollback/future-schema refusal, live/dangling artifact
entry refusal, integrity/reference preservation and concurrent publication.

This verifies static unsafe layouts on local Linux. It does **not** qualify
malicious same-user TOCTOU replacement, hard-link attacks, Windows ACLs or
power-loss semantics. Symlink ancestors are intentionally refused rather than
canonicalized; supported OS directory layouts remain externally unqualified.

Evidence: `probe/src/main.rs`, `05-independent-storage-session.log`,
`probe-results/results.json`.

### P2 — actual Rust/Node nil-session contract: repaired locally

Rust independently deserializes the two maintained shared fixtures and asserts
its actual `encode_request` output is byte-for-byte identical newline-framed
JSON. Those emitted bytes are then read by the actual Node `parseRequest`.
The independent paired result file additionally records both commands with
each session `id`, `project`, `environment` changed to nil or removed, plus null
references: **16 paired cases**, all agreeing (3 accepted, 13 rejected).

- Both commands reject each nil ID at Rust's encoder boundary with exactly
  `invalid session reference: IDs must be non-nil`; Node rejects `InvalidRequest`.
- Both reject each missing ID; Rust's typed deserialization checks the specific
  missing-field error, rather than attempting to encode a fabricated reference.
- Null login is rejected; null unauthenticated capture is accepted and its actual
  Rust-emitted frame is accepted by Node.

The capture profile is synthetic protocol data, not an executed/qualified
capture. This establishes the corrected session-reference boundary, not universal
cross-language profile validation, login persistence or a product process adapter.

Evidence: `probe-results/paired-cases.ndjson`, `node-paired.mjs`,
`06-independent-node-paired.log`, `probe-results/node-results.json`.

### P2 — Projects navigation during save: repaired locally

A fresh GPUI test includes the **unchanged actual shell source**, uses the actual
enabled navigation/save handlers, real Jobs actor and real SQLite. It holds an
independent SQLite `BEGIN IMMEDIATE` writer lock before save, navigates to
Projects and polls while the save is blocked. Assertions prove the pending save
ID/busy state/form remain intact and no project has yet been committed. This is
stronger than merely delivering a previously completed save later.

After releasing the lock, the actual save completion schedules a new List
request without revisiting Projects. The production bounded polling path receives
the authoritative saved row/revision, clearing busy/dirty. Further assertions:

- Save again, visit Projects, then leave for History before completion: History
  remains selected; no hidden List request is forced and invalidation persists.
- Make an unsaved draft, visit Projects, then navigate to Results while List is
  pending: authoritative revision 2 arrives without replacing navigation, draft
  or saved project snapshot.
- A real competing writer creates revision 3. An unrelated completion cannot
  steal the pending save; the real `ConflictOrWrite` reply preserves the draft,
  selected screen and revision 2 snapshot, with no automatic retry loop.
- Explicit later navigation retrieves authoritative revision 3 while retaining
  the failed unsaved draft. The actor is stopped/joined and its temporary data
  directory cleaned.

**PASS: simulated GPUI plus actual persistence, not native desktop/input evidence.**
Evidence: `probe/src/lib.rs`, `07-independent-ui.log`.

## Actual serial checks

Working directory: `/path/to/Regressee`. `run.py` supplies `TMPDIR=E`
and the existing project-local linker directory. No installs/network jobs occurred.

| Command | Actual result |
| --- | --- |
| Impeccable context for `crates/app/src/main.rs` | Loaded once; source/state audit only, no design edits or native attempts; desktop scope retained |
| `bash scripts/check-foundation.sh` | Exit 0, 4.721 s: locked UI binaries link, fmt check, **49 Rust tests**, all-target/all-feature clippy `-D warnings` |
| `.runtime/installed/node/bin/node --test worker/product/test/protocol.test.mjs` | Exit 0, 0.064 s: **4 tests passed**, including shared session fixtures |
| `cargo run --locked --bin regressee-foundation -- E/headless` | Exit 0, 0.565 s: schema 2, real save/reopen, artifact integrity and referenced-object collection guard; synthetic text fixture honestly labeled |
| `cargo generate-lockfile --offline --manifest-path E/probe/Cargo.toml` | Exit 0, 0.565 s; generated a different transitive graph, deliberately retained as `generated-not-used.lock`, **not used to execute probes** |
| `cargo run --offline --manifest-path E/probe/Cargo.toml -j 4 -- E/probe-results` | Exit 0, 2.017 s: 30 unsafe-layout cases and Rust decisions for 16 paired session cases |
| `.runtime/installed/node/bin/node E/node-paired.mjs E/probe-results` | Exit 0, 0.032 s: all 16 Rust/Node decisions agree |
| `cargo test --locked --offline --manifest-path E/probe/Cargo.toml --features ui-tests -j 4 verifier_ -- --nocapture` | Exit 0, 2.520 s: **1 fresh integrated UI sequence passed**, 2 included maintained shell tests filtered out (already passed in product regressions) |
| `cargo test --locked --workspace --all-features -j 4` | Exit 0, 3.520 s: **49 tests passed again**, explicitly under unique `TMPDIR=E` |
| `python3 E/audit.py` (before and after report/index writes) | Both exit 0, 0.214 s each: **1,401 protected files unchanged**, no live project/private-runtime processes, worker scratch or setup stages; probe registry graph matches product lock |

The standard foundation script resets `TMPDIR` to `/tmp/opencode`; its tests use
unique temporary directories. The explicit final locked workspace test run uses
the unique evidence root directly. Probe builds reuse the existing target directory
with debug=0. Their final lock was seeded anew from the product lock after the
unused generation; the audit confirms every resolved registry package/version/
source/checksum is in the product lock. Probes supplement, not replace, locked
product checks. Existing upstream `proc-macro-error2 2.0.1` future-Rust warnings
remain. No executed test/build failures occurred in this run.

## Open qualification, preservation and next actor

- **Native rendering/input/screenshot review: NOT RUN here.** No retry of the
  historical unmapped-window failures for these backend/state fixes; no virtual
  display, forced mapping or sandbox workaround. Current product meaningful
  pixels, desktop keyboard/mouse/clipboard/IME/dialog/menu/resize/restart behavior
  remain open, not inferred from simulated GPUI or linked binaries.
- **Human visual/usability approval and accessibility: pending.** The light
  proposal is not approved. The original source-only control-border contrast
  observation (2.963:1), native typography/wrapping, screen-reader semantics and
  host theme/high-contrast behavior remain qualification work.
- **Windows/macOS, clean supported Linux, default Wayland/hardware graphics,
  OS directory layouts, CI dispatch, distribution/signing and complete release
  acceptance: NOT RUN/pending.** No full-platform claim is added.
- Retained fixture `regressee-spike protocol-test`: **NOT RUN in this corrective
  recheck**; its unchanged supervisor/shared framing path was inspected and
  retained spike unit regressions passed. Earlier fault-test evidence is preserved,
  not claimed as freshly rerun. No product process adapter is qualified by it.
- Secure runtime delivery/session persistence, real crawl/capture/comparison and
  later ownership/ZIP/release work remain their respective phase obligations, not
  new prerequisites manufactured for Phase 1.
- Only this new report and `docs/IMPLEMENTATION_STATUS.md` were written in the
  project. Source, lockfiles, original reports and artifacts match the pre-run
  preservation manifest. Evidence/build outputs are intentionally retained;
  transient fixture roots were cleaned and all owned actors completed. No
  unowned process was killed or path deleted.

**Next actor may start Phase 2 locally under decision 0005.** Decision
0006-controlled-parallel-delegation changes delegation policy, not acceptance
honesty. Carry these explicit external/native/human gates forward; do not restore
the blanket Phase 0 stop or describe the foundation as fully accepted.
