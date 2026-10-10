# Phase 1 — independent foundation verification

Date: 2026-10-09. Independent GPT-6.1 Sol verifier; Phase 1 only.

## Verdict

**Local safety repair required before Phase 2 builds on this storage.** The
locked foundation checks and fresh behavioral probes passed their positive
assertions, but an independent negative probe reproduced a **P1 database-path
safety defect**. Broad exit-code passes do not erase that finding. Two P2
contract/UI issues are also recorded below. No implementation was changed.

Decision 0005 remains in force: native/external qualification gaps do **not**
restore the blanket Phase 0 stop. Repair and independently reverify the narrow
local defect, then Phase 2 may continue locally. This verifier neither starts
Phase 2 nor declares full Phase 1 acceptance or release readiness.

Evidence root (fresh, retained, private):
`/tmp/opencode/regressee-phase1-verifier-1791557923692541570/`.
It contains command JSON/logs, probe source/locks/data, native failure evidence,
source contrast, a resource audit, and the pre-edit pipeline-index snapshot.

## Findings

### P1 — dangling database symlink bypasses the regular-file guard

- **Location:** `crates/storage/src/lib.rs:117–126`, especially
  `if database.exists()` at line 120; permissions at lines 133–136 also follow
  the resulting database target.
- **Reproduction:** prepare isolated `AppPaths`; create a symlink at
  `paths.database()` pointing to a nonexistent file outside `paths.data`; call
  `Store::open(paths)`. No concurrent path replacement or adversarial race is
  needed. `Path::exists()` returns false for the dangling link, skipping the
  `symlink_metadata` guard. SQLite follows the link.
- **Actual result:** `open_ok=true`, `outside_target_created=true`. Read-only
  inspection confirmed schema version 2 and `projects`, `artifacts`,
  `artifact_refs` tables in `probe-data/outside-created.sqlite3`; the original
  `probe-data/dangling-db/data/regressee.sqlite3` remains a symlink. Both paths
  are inside the verifier's owned root; no unrelated data was touched.
- **Impact:** a pre-existing unsafe data layout silently writes a database outside
  the intended data directory instead of failing closed. This contradicts the
  implemented regular-file boundary, distinct from the documented exclusion of
  malicious same-user filesystem *races*.
- **Required repair:** inspect the database directory entry with
  `symlink_metadata` even when its referent does not exist; treat only a genuine
  `NotFound` as absence, reject links/nonregular entries, propagate other errors.
  Add a regression asserting refusal and no outside-target creation. Reverify
  ordinary first open, reopening, existing-link rejection and migrations. Do not
  claim this alone solves hostile same-user TOCTOU races or Windows ACLs.
- **Evidence:** `probe/src/main.rs`, `06-independent-probe.log`,
  `final-audit.json` in the evidence root.

### P2 — Rust and Node disagree on session-reference validity

- **Location:** `crates/runtime/src/protocol.rs:104–124`; paired checks in
  `worker/product/src/protocol.mjs:15,31,34`.
- **Reproduction:** construct protocol-v2 `SetupLogin` with a valid job/request
  envelope but nil UUID `session.id`; call `encode_request`, then feed its exact
  emitted bytes into Node `parseRequest`.
- **Actual result:** Rust encoding succeeds; Node rejects `InvalidRequest`.
  The encoder does not validate session references in either session-bearing
  command. The maintained sides therefore do not share the same validity rules.
- **Impact/action:** reject nil session/project/environment IDs at the Rust wire
  boundary and add cross-language fixtures before relying on session commands.
  This is not a demand to implement login or a credential store in Phase 1.
- **Evidence:** `probe-data/nil-login-request.ndjson`, `07-node-probe.log`,
  `node-probe.mjs`.

### P2 — navigating to Projects during save leaves the list stale

- **Location:** `crates/app/src/main.rs:95–109,169–183`.
- **Reproduction:** start with no projects; create/save a valid project; while
  `pending`/`busy` is set, navigate to Projects (navigation remains enabled).
  The List request is skipped. Save completion updates the selected project but
  neither refreshes the project list nor schedules a Projects refresh.
- **Actual result:** stored project exists, but Projects retains the empty list;
  explicitly revisiting Projects refreshes it and returns the saved row.
- **Impact/action:** confusing false empty state, not data loss. Schedule a
  deferred refresh after pending work completes when Projects is visible.
- **Evidence:** `ui-probe/src/lib.rs`, `08-ui-probe.log`. This is a simulated
  GPUI/action-path reproduction with the real save actor; its test mirrors the
  inspected completion branch. It is **not** a native-desktop observation.

## Actual commands, run serially

Working directory: `/path/to/Regressee`. `E` below denotes the
fresh evidence root above. Logs record complete output; no failures were hidden.

| Command | Actual result |
| --- | --- |
| `/home/developer/.config/opencode/skills/impeccable/scripts/impeccable context --target crates/app/src/main.rs` | Context loaded once; source/native audit guidance used, no design edits or mobile-product requirements imposed |
| `bash scripts/check-foundation.sh` | Exit 0, 3.820 s: locked UI binaries linked, fmt check, **43 Rust tests**, UI-inclusive clippy `-D warnings` passed |
| `cargo run --locked --bin regressee-foundation -- E/headless` | Exit 0, 0.126 s: schema 2, actual project save/close/reopen and referenced-artifact guard; synthetic text fixture honestly labeled |
| `.runtime/installed/node/bin/node --test worker/product/test/protocol.test.mjs` | Exit 0, **3 tests passed** |
| `cargo run --locked --bin regressee-spike -- protocol-test` | Exit 0, 61.164 s: actual retained fixture-supervisor crash/relaunch, cancellation, worker restart, whitelist and frame-budget behavior |
| `python3 scripts/foundation-native.py --output E/native` | Exit 1: owned product window found but unmapped; native gate remains open |
| `cargo run --offline --manifest-path E/probe/Cargo.toml -- E/probe-data` | Exit 0; fresh positive behavioral assertions passed; explicitly printed both negative-boundary defects rather than disguising them as passes |
| `.runtime/installed/node/bin/node E/node-probe.mjs E` | Exit 0; independent rejection assertions passed and exact Rust→Node mismatch confirmed |
| `cargo test --offline --manifest-path E/ui-probe/Cargo.toml --features ui-tests -j 4 -- --nocapture` | Exit 0, **4 tests passed**: 3 fresh tests plus original included shell test; stale-list issue reproduced |
| Read-only source contrast/SPDX scan and `qualify.scoped_audit()` plus `/proc/exe` scan | Nine normal text pairs ≥5.15:1; no missing inspected source SPDX; no live app/private Node/browser/probe processes or owned scratch/setup stages |

The temporary Cargo probes use actual path dependencies, not copied or fixed
implementation. The UI probe includes the actual shell source; its lock starts
from the product lock. The smaller non-UI probe resolves its own offline lock;
it is behavioral evidence, not a substitute for the locked product build.
Generated probe-only dead-code/unused-import warnings are in its log. The locked
product checks retain upstream `proc-macro-error2 2.0.1` future-Rust warnings.

## Behavioral and source review

- **Project persistence/validation:** actual non-default discovery settings,
  Unicode names, meaningful repeated query order and route round-trip through
  independent stores; revision conflicts do not overwrite. Invalid schemes,
  embedded credentials, bad DPR/page limits/session scope and unsupported
  baseline activation are rejected. Unknown/oversized persisted JSON fails
  loading; restoring valid fixture JSON recovers the record.
- **Migrations:** fresh/v1 databases retain project settings and migrate to v2;
  repeated migration is idempotent. Injected conflicting DDL rolls back both
  tables and `user_version`; future versions are rejected without schema changes.
  Narrow tables are appropriate; later-phase run/baseline transactions are not
  demanded here.
- **Artifacts:** deduplication preserves bytes/kind; conflicting kinds and
  same-length hash corruption are rejected, never overwritten. Two independent
  stores/two owners preserve shared references. Corruption cannot acquire a new
  reference or be collected while still pinned. Empty/partially interrupted
  streams leave no metadata/pending file. Existing object links are rejected and
  outside bytes preserved. Existing tests additionally exercise 128 MiB bounds
  and concurrent publication. No automatic history GC exists.
- **Worker:** inspected Rust envelopes, Node declarations/parser and actual
  assertions, not just counts. Version/command whitelists, safe request IDs,
  complete newline/UTF-8/frame limits, bounded links/progress, typed error fields,
  acknowledgement/correlation and post-terminal rejection are exercised.
  Product supervisor is honestly an interface; fixture passes do not prove a
  product process adapter, stderr/deadline policy or cross-OS tree cleanup.
- **Secrets/archives:** portable conversion drops session references; injected
  session/cookie/script/executable/local-path fields and noncanonical artifact
  paths are rejected in the tested schema positions. Unavailable load/save/delete
  fail closed; debug is redacted. Zeroization/nonserialization are inspected
  source boundaries, not keychain or forensic-memory qualification. No ZIP
  feature or persistent login implementation is claimed.
- **UI-thread work/queues:** native handlers perform bounded in-memory validation
  and `try_send`, not synchronous SQLite/filesystem/process/image operations.
  `Store::open` and all persistence work run on the storage thread. Both channels
  have capacity four; UI drains at most four updates per 50 ms. Fresh real
  SQLite-writer-lock probe fills four command slots, rejects a fifth in <100 ms,
  then receives all four replies after releasing the lock. Terminal replies
  survive backpressure; no UI-thread joins (`join_after_ui` is after the loop).
  Native prompts are awaited asynchronously.
- **Shell/action state:** original tests assert active-screen focus, busy guards,
  modal trap/restoration, hidden/disabled controls and Unicode/UTF-16 IME behavior.
  Fresh actual-handler save/edit test preserves advanced settings and updates
  default origin mappings; reverse/forward focus, input non-submission and
  read-only input behavior are asserted. This is simulated GPUI only. Aside from
  the stale-list finding, source/state rules are coherent at this phase's scope.
- **Evidence honesty:** Projects use stored settings with “No captures yet”;
  Results/History explicitly state that capture/comparison/share workflows are
  unavailable. Default profiles are `Unconfigured`. No real crawl, successful
  comparison, coverage verdict or baseline is manufactured by this foundation.

## Native, design and license qualification

The one bounded developer-native batch found PID **806432**, owned title
`Regressee — Foundation`, X11 window **37748737**, `map_state=0`. It stopped
without forced mapping, focus/input, screenshots, virtual display or retries.
Its owned process was terminated and reaped. Native render/input and current
Phase 1 screenshot review: **NOT RUN / blocked before meaningful pixels**.

Inspected historical `artifacts/verifier-native-initial.png`: it genuinely shows
the earlier **Phase 0 feasibility bench**, not today's product shell/design.
Neither that old render nor backend/simulated tests qualify the current shell.

`DESIGN.md`, theme/controls and `docs/foundation-surface.md` describe a consistent
light operational proposal with ordinary task language, limited list paging,
honest empty states and visible focus. Setup/results topology, palette and type
remain **proposed, not human-approved**. Normal text-token contrast passes the
source calculation; the inactive control border is 2.963:1 against white
(just below 3:1), to address in accessibility qualification. Wrapping, native
typography, pressed states, resize behavior, high contrast/theme integration,
screen-reader semantics and native dialogs/menus remain unverified. Do not
invent a visual/a11y score or approval from source alone.

MPL-2.0 text, workspace licensing, exact Rust/GPUI/direct dependency pins,
lockfiles, source SPDX headers, attribution and retained GPUI Apache license
were inspected. The adapted input remains Apache-2.0, not relicensed. Product
worker is private/dependency-free with a Node 22.20.0 engine pin. Three-OS CI is
defined but not dispatched. Complete target-specific third-party/license audit,
signing and distribution checks remain release work.

## Not run, preservation and next actor

- Windows/macOS, clean supported Linux, default Wayland/hardware graphics,
  native keyboard/mouse/clipboard/IME/menu/dialog/resize/restart interaction,
  accessibility and user visual/usability approval: **NOT RUN**.
- Runtime distribution, persistent OS-protected sessions, full ZIP validation,
  real crawl/capture/comparison, later-phase ownership transactions, disk-pressure
  recovery and release packaging: not implemented/qualified by Phase 1, not
  manufactured prerequisites for this narrow verification.
- Only the new report and pipeline index are written in project documentation;
  implementation is untouched. The original index was snapshotted specifically
  in `E/IMPLEMENTATION_STATUS.md`; no history was recursively cloned or deleted.
  Eleven protected historical files still match the implementer's recorded
  SHA-256 manifest (`final-audit.json`); this is a comparison with that historical
  manifest, not an invented fresh before/after snapshot of all project files.
- Owned native process is reaped, temporary test workers/actors finished,
  scoped/broader process audits are empty. Probe/evidence roots and generated
  Cargo test/build outputs are intentionally retained for reproduction.

**Next:** repair P1 database-entry handling in a separate Phase 1 corrective
session and independently reverify. Fix the two P2 issues in that bounded pass
where practical. Once the material local safety defect is repaired, decision
0005 permits Phase 2 to build on the locally checked foundation while the
listed native/external/human gates stay explicitly pending.
