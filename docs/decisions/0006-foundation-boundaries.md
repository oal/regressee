# 0006 — maintained foundation, proposal UI and narrow persistence

Date: 2026-10-09. Implements Phase 1 only, under decision 0005.

## Structure

- `core`: typed immutable capture/review contracts, portable schema, testable
  shell focus/navigation. Domain invariants are not capture implementations.
- `storage`: bundled SQLite 0.37.0, two transactional versioned migrations,
  optimistic project edits, OS paths, bounded content-addressed evidence.
- `runtime`: typed product protocol v2, supervisor and secret-store interfaces.
- `app`: serial blocking storage owner and native GPUI shell. Product binaries
  never depend on the spike, browse fixture pages, or launch its historical runtime.
- `spike`, root `worker/src`, scripts: retained Phase 0 fixture bench. Its bounded
  JSON reader is extracted to runtime and reused, not rewritten. Linux owned
  scratch/process-tree and cancellation/launch-race fixes remain in the bench
  supervisor until the Phase 2 product adapter exists.
- `worker/product`: independently locked v2 wire contract package/declarations,
  no browser implementation, dependencies or auto-launched executable yet.

GPUI remains exactly 0.2.2; Rust stays 1.98.1. The Apache-2.0 GPUI text input
adaptation is shared by the bench and product, retains its attribution/license,
and is not relabeled MPL. UI composition stays native. There is no webview or new
component framework. The input adds scoped clipboard/home/end bindings, UTF-16
composition tests, bounded replacement and horizontal caret tracking. Host
window/titlebar/menu/file picker and discard confirmation use GPUI's OS paths.

## Persistence and concurrency

Only actually used entities have SQL tables: projects plus artifact metadata and
references. Future inventories/runs/baselines/analysis/masks have typed snapshot
contracts, not speculative empty SQL tables or completed workflows. Add their
tables in the phase that owns the transaction semantics. A project JSON document
is versioned, validated and capped at 256 KiB. SQLite edits require the expected
revision; concurrent edits cannot silently overwrite. Future databases are
rejected without schema mutation. Schema migrations are atomic/idempotent.

Artifacts are capped at 128 MiB, streamed through 64 KiB buffers, SHA-256 named,
synced and atomically published with no-clobber semantics. Metadata is immutable.
The SQLite writer lock serializes artifact writes, reference changes and explicit
collection across stores. A referenced artifact cannot be collected; adding a
reference verifies the bytes first. A crash between publication and metadata can
leave a complete unreferenced object. Repair/retention must not delete it silently.
There is no startup GC. Reference owner IDs reserve future domain ownership;
owner-row foreign keys arrive with future entity tables. Put then pin can fail
if explicit collection wins between calls; it cannot establish a missing reference.
Future run/baseline commits must combine pinning with their domain transaction.

Data and rebuildable cache follow `directories::ProjectDirs`: local application
data on Windows, Application Support/Caches on macOS, XDG data/cache on Linux.
Unix directories are 0700 and database/object files private; SQLite WAL is FULL
sync. Filesystem integrity protects against accidental corruption and concurrent
cooperating writers, not a malicious same-user process racing path replacement.
Do not claim Windows power-loss/directory-sync or ACL qualification from Linux.

Commands and terminal updates are separately bounded to four entries. The UI
uses `try_send` and drains at most four updates each 50 ms. A blocked terminal
reply is retried, never discarded; shutdown breaks the wait. No DB/artifact/decode/
process work, joins, or disk test markers run in the UI action paths. Thread joins
occur only after the native event loop exits or in headless tests.

## Security and protocol boundaries

Protocol v2 uses exact commands/events, typed error categories (not browser stack
or cookie text), acknowledged correlated requests, 1 MiB newline frames, 2,000-link
event limit and Node-safe request IDs. The product and fixture protocols cannot
be confused. Product captures require a configured immutable runtime profile;
new-project profiles are explicitly Unconfigured, not the old browser pin.
No imported schema contains executable paths or custom scripts. Supervisor
implementation, stderr bounds, whole-tree cross-OS cleanup and deadlines are
explicit future adapter obligations, not claimed executed product behavior.

Session secrets are non-serializable/redacted/zeroized. The unavailable store
fails closed. There is no plaintext session persistence, OS keychain integration,
login flow or session export. Portable environment types have no session-reference
field and reject unexpected fields; artifact paths are only canonical hashes.
The ZIP schema boundary is not ZIP import/export security qualification.

## Visual authority

The user requested a sleek/modern non-developer-first shell and approved local
continuation, not a palette or rendered design. The source-based readiness
checklist proposal is described in `docs/foundation-surface.md`. Human setup /
results confirmation, meaningful native pixels, desktop input, accessibility,
external OS and clean-machine checks remain open. A simulated GPUI test is not
native desktop evidence. No finish reviewer is spawned concurrently; the next
strictly serial independent verifier receives the design/focus evidence packet.
