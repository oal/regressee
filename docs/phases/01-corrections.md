# Phase 1 — bounded verification corrections delivered

Date: 2026-10-09. GPT-6.1 Sol corrective implementation, strictly serial.
**All three recorded findings repaired locally; independent re-verification
pending.** No Phase 2 work, agents, native screenshot attempts, external jobs,
publication, services, privileged changes or global installs were performed.
Decision 0005 remains in force; this is not full Phase 1/release acceptance.

`docs/phases/01-verification.md` is preserved unchanged. Its independently
reproduced defects and failed native evidence remain historical findings, not
rewritten into passes by this implementation report.

Fresh retained logs and headless evidence:
`/tmp/opencode/regressee-phase1-corrections-vmtiu4mu/`.
`commands.json` records actual commands, exit codes and durations; numbered logs
include intermediate failures. Temporary unit-test directories were unique and
RAII-cleaned; this evidence root and its headless database are retained.

## Corrections and exact regressions

### P1 — database/control entries and existing ancestors

Changed `crates/storage/src/lib.rs`:

- Removed the referent-following `database.exists()` gate. Directory-entry
  inspection always uses `symlink_metadata`; only `ErrorKind::NotFound` means
  absence. Other IO errors propagate with path context. Existing nonregular
  files, including dangling/live symlinks, fail before SQLite opens.
- Anchor explicit relative developer overrides for inspection (without adding
  cwd defaults or canonicalizing away unsafe entries). Validate traversal-free
  directory paths top-down, rejecting
  non-directory entries/symlink ancestors before inspecting children or creating
  directories. Preflight **all** data/cache/artifact directory paths before
  `create_dir_all` or permission changes. Unsafe cache/artifact layouts cannot
  cause directory creation or permission changes through their links.
- Inspect analogous SQLite-owned `-journal`, `-wal` and `-shm` control entries
  before opening/migrating the database. These could otherwise redirect SQLite
  recovery/WAL work. Artifact publication/collection now use the same
  regular-file-or-absent check rather than a referent-following existence gate.
  Publication already had no-clobber protection; dangling entries now fail
  explicitly, and collection cannot silently discard their metadata.

New regression tests assert the **specific unsafe-path error**, not just any
failure:

- `database_and_sqlite_control_symlinks_fail_before_outside_writes`: database and
  each of three control paths, both dangling and live (eight cases). Assert the
  symlink stays a symlink; outside targets are not created; existing outside
  bytes/mode stay unchanged; no outside auxiliary files appear; sidecar refusal
  occurs before even creating the database.
- `storage_rejects_symlink_directories_and_ancestors_before_creation`: data,
  cache, artifacts and a higher ancestor, dangling/live (eight cases). Outside
  sentinel contents, directory entries and mode remain unchanged; no nested
  outside data/database is created.
- `dangling_artifact_entry_is_not_published_or_collected`: both operations report
  the regular-file error, retain metadata/link, create no outside object and
  leave no pending file.
- `nonregular_database_and_invalid_directory_layout_fail_closed`: directory at
  DB entry, file at parent and explicit parent traversal produce their intended
  errors without creating traversal children.
- Strengthened the existing v1 migration regression: migration now happens
  through **Store::open**, checks schema 2, then repeats migration and reopens
  while retaining the project. Existing first-open/reopen, revision-conflict,
  rollback/future-schema, artifact-integrity/concurrency/reference tests pass.

Boundary: paths with symlink ancestors are intentionally rejected, not silently
canonicalized. This repairs static unsafe layouts, **not hostile same-user
TOCTOU races**, hard-link attacks or Windows ACL/power-loss semantics. No
descriptor-relative filesystem redesign or schema/version change was made.
Cross-platform directory-layout qualification remains pending.

### P2 — paired session-reference contract

Changed `crates/runtime/src/protocol.rs` and worker protocol comments/types/tests.
Both `SetupLogin` and session-bearing `CaptureTarget` now reject nil session,
project and environment IDs at Rust's actual encoder boundary. Contract:

- A session reference contains all three non-nil UUIDs, not credential material.
- `SetupLogin` requires a reference; null is invalid.
- `CaptureTarget` permits an explicitly null reference for unauthenticated
  capture, but never a partially populated/nil-ID reference.

Shared `worker/product/test/fixtures/session-requests.ndjson` contains login and
capture requests. Rust deserializes and **re-encodes byte-for-byte identical
newline frames**; Node parses those exact shared bytes. Both sides mutate each
of the three IDs to nil (six rejections), reject each missing ID, reject null
login and accept null capture. Rust nil tests require the exact
`invalid session reference: IDs must be non-nil` error; Node requires
`InvalidRequest`. The capture profile is explicitly synthetic protocol data,
not a qualified runtime or successful capture. No login/session persistence or
product process adapter was implemented.

### P2 — deferred authoritative Projects refresh

Changed `crates/app/src/main.rs` only. The list has an invalidation flag;
navigation to Projects marks it dirty even while storage is pending. A matching
project reply invalidates list/summary state. The real completion handler queues
an authoritative List request when Projects is visible and idle, rather than
losing navigation's skipped request. List completion clears invalidation. Other
screens stay selected; refresh is deferred until Projects is visited. Failures
keep input and stop automatic retry; mismatched completions cannot steal pending
work. A List reply does not replace the setup draft or saved project snapshot.

`save_navigation_completion_refreshes_real_list_without_losing_draft` uses the
actual GPUI shell save/navigation-button handlers, real Jobs actor/SQLite and
the **same apply_update path** used by the bounded periodic drain:

1. Receive initial authoritative empty page; create/fill/save a project.
2. Navigate to Projects with save still pending in the shell. Assert navigation
   does not replace the pending ID, busy guard or form.
3. Deliver actual actor save completion. Assert a new List request is queued
   without revisiting Projects; receive it and assert stored project/id/revision
   replaces the false empty state and clears busy/dirty.
4. Edit/save, visit Projects, then Results before completion. Assert Results
   stays selected, list stays dirty and no hidden list request is scheduled.
5. Make another unsaved edit; visit Projects and receive the refreshed saved row.
   Assert the unsaved input and saved revision are preserved.
6. Introduce an actual competing-writer revision conflict, save/navigate again.
   Assert an unrelated completion cannot steal work; the real ConflictOrWrite
   failure retains edit/navigation/revision and does not start a retry loop.

This is simulated GPUI plus real storage, **not native desktop/input evidence**.
No tokens, layout, visual proposal or screenshot files were changed.

## Actual serial checks

Working directory: `/path/to/Regressee`. The retained runner sets
`TMPDIR` to the unique evidence root and `LIBRARY_PATH` to the existing local
link directory; the standard foundation script sets TMPDIR to `/tmp/opencode`
and its tests likewise use unique temporary directories.

| Command | Actual final result |
| --- | --- |
| Impeccable context for `crates/app/src/main.rs` | Loaded once; incumbent proposal retained; state hardening only, no native attempts |
| `cargo fmt --all` | PASS (three runs, final after all source edits) |
| `cargo test --locked -p regressee-storage -j 4` | Final PASS, 13 tests, 3.526 s; initial unused-mut warning fixed and covered by final all-target lint |
| `cargo test --locked -p regressee-runtime -j 4` | PASS, 6 tests including exact shared frames |
| `.runtime/installed/node/bin/node --test worker/product/test/protocol.test.mjs` | PASS, 4 tests |
| `cargo test --locked -p regressee-app --all-features --bin regressee save_navigation_completion -j 4 -- --nocapture` | Final PASS, 1 integrated sequence; 1.584 s |
| `bash scripts/check-foundation.sh` | PASS twice; final 10.705 s: locked UI binary build, fmt check, **49 Rust tests**, all-target/all-feature clippy `-D warnings` |
| `cargo run --locked --bin regressee-foundation -- /tmp/opencode/regressee-phase1-corrections-vmtiu4mu/headless` | PASS, 0.468 s: schema 2, save/reopen, verified referenced-artifact guard; synthetic text fixture only |
| `/path/to/Regressee/target/debug/regressee-foundation relative-headless` (cwd: evidence root, binary from final locked build) | PASS, 0.003 s: same real persistence/reference assertions with an explicit relative override; all data remains inside the owned `/tmp/opencode` root |
| `cargo run --locked --bin regressee-spike -- protocol-test` | PASS, 59.970 s: real retained fixture supervisor browser crash/relaunch, bounded cancel, worker crash/restart, whitelist and frame budget |
| Read-only preservation/process/scratch audit | PASS: 1,324 hashed historical evidence/runtime/package files unchanged; no live project/private runtime processes, worker scratch or setup stages |

Intermediate failures are retained, not counted as passes: focused UI test first
failed compilation because `Entity::read` needed the underlying App context,
then its navigation assertion failed because simulated keyboard dispatch through
disabled input focus did not invoke navigation. Corrected the context read and
used the actual enabled navigation-button activation handler (not a fabricated
screen assignment). The test then passed, and passed again after adding the
real conflict/mismatched-reply assertions. Existing upstream
`proc-macro-error2 2.0.1` future-Rust warnings remain.

Final source review retained documented explicit relative developer overrides:
inspection anchors them without canonicalizing away symlinks. Focused storage
and standard locked checks were rerun after that compatibility refinement; the
relative headless invocation independently exercised ordinary creation/migration
and reopening under the owned evidence root.

Preservation baseline was taken after the initial bounded source edits and before
checks/documentation updates; it covers existing artifacts, independent Phase 1
verification, the native recheck report, private Node binary and worker package
manifests/lock. It is not a claimed pre-session snapshot of the entire workspace.
No protected hash changed. No unowned processes or paths were cleaned.

## Fresh verifier commands and pending gates

Run strictly serially from the workspace; use a new evidence directory, never
overwrite the implementation or prior verifier's databases/reports:

```sh
E="$(mktemp -d /tmp/opencode/regressee-phase1-reverify-XXXXXX)"
export TMPDIR="$E"
export LIBRARY_PATH="$PWD/.runtime/link-libs${LIBRARY_PATH:+:$LIBRARY_PATH}"
cargo test --locked -p regressee-storage -j 4
cargo test --locked -p regressee-runtime -j 4
cargo test --locked -p regressee-app --all-features --bin regressee save_navigation_completion -j 4 -- --nocapture
.runtime/installed/node/bin/node --test worker/product/test/protocol.test.mjs
bash scripts/check-foundation.sh
cargo run --locked --bin regressee-foundation -- "$E/headless"
cargo run --locked --bin regressee-spike -- protocol-test
```

Independently reproduce the original dangling-DB/nil-session/save-navigation
probes against the repaired implementation; inspect precise causes and outside
non-mutation, not just aggregate exit codes. Check the strict ancestor policy,
SQLite sidecars, ordinary migration and deferred-refresh failure/correlation
branches. Preserve `01-verification.md`; write a separate re-verification report
and update the pipeline with the independent verdict.

**Pending/NOT RUN:** independent correction re-verification; meaningful native
rendering/input and screenshot review; human visual/usability approval;
accessibility (including the verifier's control-border contrast observation);
Windows/macOS/clean supported Linux/default hardware graphics and OS-directory
layouts; CI dispatch; secure runtime delivery/session persistence, complete
third-party audit/signing/distribution and full release gates. None were
reclassified as passed. No repeated native attempts are needed for these
backend/state corrections. Phase 2 remains not started; decision 0005 permits
subsequent local continuation after the narrow safety repair is independently
rechecked while external gates remain explicitly open.
