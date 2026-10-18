# Local application engine (protocol 8)

This is the shared application foundation. Phase 3 adds the public
[CLI workflows and evidence contract](cli.md); the
Phase 4 adds named exclusive leases and GUI-opt-in follow/human handoff. See
[implementation status](agent-cli-status.md) for test results and platform limits.

Protocol 2 repaired the independently reviewed confirmation and capture-receipt
defects and independently passed Phase 2. Protocol 3 added typed terminal
inspection/crop/report operations, paginated job reads, durable appearance updates
and engine-resolved baseline confirmation preview details. Protocol 4 repairs
Phase 3 evidence selection and adds focused current reviews, keyset review-history
pagination, explicit recent-window truncation and frozen report queue/reference
settings. Phase 3 independently passed its repaired scope. Protocol 5 adds private
session/frontend authority envelopes, owner-bound confirmations, control lifecycle
DTOs and exact agent viewer selections. Protocol 6 closes anonymous frontend-role
minting with an engine-minted private GUI bootstrap capability. Rebuild/restart
both entrypoints and the engine; older peers are
refused for product operations rather than silently attached. The private `engine-v1/` directory name
remains the on-disk journal/endpoint location, not the negotiated wire version.
For a protocol upgrade from 6 or 7, close old desktop windows, finish active work,
then run the rebuilt `regressee engine stop` against the same data/cache paths and
reopen the desktop. The explicit stop negotiates only the known unchanged
Hello/Stop envelope; it verifies owner identity and cache and retains the old
engine's active-job, login and agent-session refusal. No product operation or
credential crosses versions. Other incompatible versions require their matching
executable to stop. Autostart replaces a previous-6/7 descriptor only after
acquiring the ownership lock and proving no engine still owns the store. It never
deletes a lock/descriptor or automatically stops a live engine. Alternatively,
explicit `engine serve` publishes a fresh descriptor while holding the same lock.

Protocol 7 adds canonical screenshot/DOM capture plans and kind-aware semantic
agent selections. `Selection.kind` defaults to `Screenshot` for legacy records;
its analysis UUID belongs exclusively to that kind. Capture/check operations do
not apply that legacy kind default: they obey the frozen run/baseline plan.
Mode/policy replacement is protected like other baseline-bound configuration.

Protocol 8 adds project/environment deletion scopes, impact details, and a receipt
that distinguishes committed deletion from pending physical file cleanup.

## Entrypoints and lifetime

```sh
cargo build --locked -p regressee-cli --bin regressee
regressee engine start --data-dir /absolute/private/data --cache-dir /absolute/private/cache
regressee engine status --data-dir /absolute/private/data --cache-dir /absolute/private/cache
regressee engine stop --data-dir /absolute/private/data --cache-dir /absolute/private/cache
```

`engine serve` is the foreground equivalent of `start`. Without explicit paths,
CLI and desktop use the same discovered per-user data/cache locations. With only
`--data-dir`, the cache defaults to `DATA/cache`. Public CLI directory arguments
apply to all product commands too.
`status` and `stop` do not autostart or migrate a missing store. Status does not
install/download a browser, unlock credentials or initialize graphics. Actual
capture/setup/login operations retain their explicit product requirements.

The desktop attaches/autostarts through `engine::Client`, using a known installed
sibling `regressee` or its own current executable. The maintained desktop prefers
its current executable so a stale CLI sibling cannot break a partial upgrade.
Both binaries recognize
`engine serve` before desktop initialization. Launch uses argument arrays and an
absolute, checked executable, never a shell, PATH lookup or user-supplied command
string. Unix autostart starts a new session without inheriting terminal IO;
Windows requests a detached process group and job breakaway; startup fails
closed if an enclosing host job forbids independent lifetime. Native Windows
console/job lifetime qualification remains outstanding.

**Idle policy: remain running until explicitly stopped.** No idle timer abandons
a job or a managed login. Closing a GUI, dropping a client, losing an IPC reader,
or interrupting a waiting CLI process does not cancel jobs. Cancellation/pause
are explicit protocol requests. `engine stop` refuses while jobs or a managed
login are active; finish/cancel jobs and explicitly close login first. In
foreground `serve`, Ctrl-C drains: new mutations are refused, status/control,
read requests and close-login remain usable, and shutdown waits for active work
and login to end. A second Ctrl-C does not turn that into implicit cancellation.
Forced OS termination is a crash, not a graceful completion.

## Ownership, endpoint and framing

- Canonicalize the data root. A protocol-independent `engine-owner.lock` is
  exclusively locked **before** Store migration, interrupted-run recovery,
  confirmed-GC completion or journal recovery. Never unlink/recreate the lock
  to bypass an owner. Aliases to the same root share its lock and endpoint key.
- Unix directories/files are owner-owned, private (0700/0600), ordinary objects;
  lock files reject symlinks and multiple hard links. Existing unsafe permissions
  fail closed rather than being silently repaired. Windows uses the existing
  current-user ownership check and protected private ACL implementation.
- Linux uses a UID+canonical-root-hash abstract Unix-domain socket. Both peers
  verify kernel peer UID; no socket-file path-length/stale-file race. Other Unix
  targets use `engine-v1/socket` in the private directory; only a stale socket
  owned by this user may be removed, while the data lock is held.
- Windows uses `\\.\pipe\regressee-engine-HASH`, an explicit protected
  current-user-only DACL, first-instance protection and remote-client rejection.
  This is real Tokio/Win32 code, not a TCP fallback. Native ACL/IPC/crash
  qualification is still required; a source type-check is not that qualification.
- `engine-v1/endpoint.json` is a private, atomically published version/owner UUID
  record, also declaring the canonical cache directory. A client refuses a
  different requested cache (rather than ignoring it or spawning another owner)
  and verifies the hello UUID before sending commands.
  A replacement record racing reconnect is re-read once; an unmatched identity
  or protocol is refused. Startup contenders may launch children, but only the
  lock winner can open/recover storage. Losing children do not recover anything.
- One versioned hello and one request/response per connection. Length-prefixed
  JSON (big-endian u32); 64 MiB frame cap, five-second read/write/connect deadlines,
  16 admitted connections. Oversized/malformed/partial frames cannot admit jobs.
  No network listener or remote API. Startup has a 30-second client deadline.

This coordinates one user's installation; it is not a sandbox against programs
already running as that user or an administrator.
Close older **pre-engine** desktop/developer binaries before upgrading: they do
not implement this lock protocol and cannot safely coexist with a new engine.
The new lock does not retroactively constrain old executables that ignore it.

## Commands, jobs and consent

`crates/app/src/protocol.rs` explicitly defines versioned operations/results,
job snapshots and events. Internal actor Work/Reply and GPUI objects never cross
IPC. It covers the existing application actions: projects/settings, discovery,
capture/check/resume, authentication, analysis/review/baselines, history/storage,
ZIP sharing, diagnostics, network/runtime setup and immutable viewer DTOs.
Public product CLI parsing, result reports/crops and stable CLI exit semantics
are specified in [cli.md](cli.md). Static evidence exports run on the engine
worker with exact preview/content/destination confirmation; the CLI does not
open the live database or read arbitrary paths as evidence.

Protocol-4 terminal evidence resolves the selected/default recorded run first
(or the run of an explicit analysis), then uses the exact-run store reader. A
check's reference is its frozen original baseline, never the current active
pointer; non-check exact reads do not borrow that pointer. The focused native
viewer also retains a check's reference when no saved analysis is available.
Selected-analysis current reviews are queried per target, independently of the
256-entry recent run-history window. That window exposes truncation; the readonly
`review history` operation provides 1–200-entry keyset pages with stable decision
IDs. Reports freeze capture queue/discovery/revision/resume data and selected
reference-source settings; legacy absence is explicitly unknown. These fields,
current reviews, source identities and privacy inventory are fingerprinted with
the selection and destination. Old report previews require regeneration rather
than retroactive certification.

The broker and the blocking product worker are separate. Only the latter owns
Store/Product/browser workers. Status, cancellation, epoch changes and event
reads do not queue behind browser/analysis work. Filesystem durability work runs
off the async reactor; bounded admission/receipt filesystem operations can still
briefly serialize broker access. This is not a hard real-time system.

Jobs use a UUID request/job identity and engine-incarnation UUID/control epoch.
The admitted serializable input is immutable. For operations that consult saved
capture/discovery/login settings, admission pins the known project revision;
execution loads and persists that exact configuration or fails a revision
conflict, never silently substituting a newer configuration. Explicit revisions,
inventory/run/analysis identities, preview fingerprints and the services' own
transaction-time validation remain authoritative.

Authority-bearing queued commands check epochs and cancellation before work; consequential
dispatch checks epoch again and simple handlers check cancellation at entry.
Advancing an epoch revokes queued authority, not already dispatched jobs. Their
real committed success/failure remains truthful. Protocol 5 extends this earlier
hook with the exclusive session/frontend authority system described below.

Consequential actions require `Prepare(exact operation)` then a new submit with
its confirmation UUID. Confirmation binds action+all arguments (including the
specific preview, selection and destination), storage revision, owner UUID and
epoch. Wrong action, changed preview/revision, consumed token or restarted/revoked
owner is refused. Existing GUI confirmations feed these exact operations through
the facade; application-level preview validation still runs at commit. Preparing
a token is deliberate command intent, not proof of human consent. There is no
universal bypass flag.

Environment saves have two distinct intents. `SaveEnvironment` uses the same
active-baseline protection as ordinary `Save`: source addresses, profiles and
environment definitions cannot be replaced through it. Safe project-name,
discovery and private local-login settings can still be saved atomically without
a replacement token. `SaveReplacementEnvironment` explicitly combines replacement
settings and private login settings in one revision-bound transaction and requires
its own exact confirmation; neither a `SaveReplacement` token nor a token for
different login settings authorizes it. The GUI selects that explicit operation
for baseline-bound replacement saves. No save operation can activate/clear a
baseline pointer; activation/restore remain separate confirmed operations.

Mutation retry must reuse the **same request UUID and complete original submit**.
All CLI receipt views (show/wait/retry) reconstruct a prepared ticket from its
original durable operation and Confirmation, without preparing again, consuming
authority or refreshing owner/epoch/revision. Stale/consumed tickets stay stale.
Configuration prerequisite reads use the same journaled, interrupt-aware client
submit/wait path as other CLI operations. Ctrl-C detaches that read and prevents
observed-interruption continuation saves; recovering the read does not silently
continue the abandoned configuration command. Local argument/input validation
and submit-flag applicability are checked before data discovery/autostart; stored
state and transaction-time validation remain the engine's responsibility.
An identical retry returns the existing job; changed input under that ID conflicts.
Admissions and terminal receipts are atomically written/fsynced in private job
files; progress is checkpointed roughly once per second when updates arrive.
Terminal results remain queryable independently of event-reader backpressure.
Read results may be acknowledged/reclaimed; mutation and confirmation receipts
from any incarnation remain protected. Current-incarnation reads require their
originating session/frontend; completed reads orphaned by restart may be explicitly
acknowledged without resurrecting the obsolete credential. Mutation receipts
are not silently evicted. Limits are 64 queued/running jobs, 4096 retained jobs
and a 512 MiB journal, 8 MiB admitted input/frozen project budgets. At capacity,
new admission is refused rather than losing idempotency. A future explicit receipt
archival policy must preserve that property.

A crash between product commit and receipt is **uncertain**, not exactly-once
completion. Restart marks nonterminal receipts `Interrupted` and never replays
them automatically. Inspect saved state before deliberately issuing another
mutation. A lost response is not permission to generate a new request ID.
Failures, cancellation, paused capture, incomplete capture and interrupted runs
remain distinguishable from success; a late cancel never rewrites committed
success, nor is a visual difference itself labelled a product failure.

Every capture-producing operation (`Capture`, `CaptureInventory`, `PreviewImages`)
returns `Captured`, not the ordinary workspace `Evidence` result. Its record is
the exact run returned by the service, verified against that run's saved record,
not the newest inserted project run. For checks, reference/analysis/reviews come
from that run and its original baseline; paused/failed checks can legitimately
have no analysis. Receipts validate project, explicit resume ID and known progress
run ID before classifying the actual run state. Missing or mismatched capture
results fail closed; structured service failures/cancellation are preserved.

On restart, old capture receipts that contain an unbound workspace result are
retained as `Interrupted` with `UnverifiedCapture { message, prior }`. Their prior
result payload and original submit remain available for inspection; neither its
former success nor its alleged run identity is retroactively certified. Retries
return that same uncertain receipt without rerunning work. This does not roll
back any previously committed product data or invent retrospective consent.

## Reconnect and the GUI

Snapshots include owner UUID, epoch, sequence, recent/active job summaries and
saved project revisions. Events use monotonically increasing per-owner sequence
numbers. The ring is capped at 512 events/8 MiB; a gap, future cursor or changed
owner demands snapshot resync. Job IDs let a client recover any retained terminal
receipt even after the event ring rolled over. No unbounded subscriber queue.

The production GUI facade submits the same operations as a headless client. Its
observer obtains a snapshot cursor then pulls later events; saved-data refresh
does not move screens or discard drafts. Conflicting project drafts keep their
old expected revision rather than overwriting newer saved settings. Local
appearance choices are saved through the engine and refreshed across clients.
Agent progress/follow banners and semantic navigation now use a separate control
observer and exact-run/analysis reads, not the GUI's unrelated busy request.
Passive evidence and History refresh also have independent request/generation
lanes: they retain the current run/analysis/result and the same History page,
remap selected rows by ID, and never retire another GUI task or busy request.
A deleted/off-page History selection becomes no selection, not another row.
History's paged baseline list is not replaced by an exact-run metadata read.
Notes, ignored areas and raw analysis-policy/IME drafts defer evidence replacement.

### Exclusive authority (protocol 7)

See [agent CLI and desktop control usage](cli.md#agent-control-and-optional-gui-following).
Named agent sessions have renewable monotonic 10–300-second leases (90 default).
Only one owns a canonical data root. GUI registration has a separate 15-second
renewable heartbeat; its private credential identifies the consenting frontend,
not the agent. The CLI cannot opt a GUI into following. Short-lived IPC/CLI
disconnects are normal and do not end a logical agent lease.

The engine creates `engine-v1/frontend-capability.json` under its private data
directory while holding the owner lock (0600, current UID/single link/no symlinks
on Unix; current-user protected ACL/no reparse point on Windows). It contains a
fresh random secret bound to this engine incarnation; it is removed on normal
owner shutdown and rotated after a crash/restart. It is never returned by an
anonymous IPC read or included in public status, jobs, receipts or reports.
The explicit local GUI bootstrap reads that private file and proves possession
with `Request::RegisterFrontend`; the engine registers a new frontend credential
and heartbeat deadline in memory. A client-type label or the legacy anonymous
`Control(RegisterFrontend)` does not establish authority. An already authenticated
agent/frontend cannot switch roles through this bootstrap request.
GUI Detach retires that frontend credential immediately. It revokes a controller
only if that frontend was following it; closing an unrelated GUI does not revoke
another session. A lost frontend is bounded by its heartbeat expiry instead.

Status/list/show/read calls never register or renew clients. Registration and
authenticated Renew/Heartbeat change only their own bounded lifecycle state,
not product data, selection, following, the controller or control epoch. Renew
and Heartbeat do not require the current navigation epoch, but still require
their valid original principal; an expired/revoked session cannot be revived.
Raw Follow/Takeover/Return/Detach/Heartbeat and all ordinary mutations still
require their registered authority. Immediate legitimate GUI takeover remains
independent of agent approval or blocking work. Filesystem possession is the
bootstrap trust boundary, not executable attestation: same-user code that reads
that private capability can emulate a frontend. No OS-sandbox claim is made.

Every mutation/control request is checked by the broker and authority-bearing
queued commands are checked again at dispatch. Confirmations bind the owning
session/frontend as well as exact action/revision/engine/epoch, and are rechecked
at actual use. Raw submissions cannot substitute an owner field; proof travels
only in the private Authorized envelope and is not persisted in receipts/events.
Stop, epoch advance, direct preference writes and cancel/pause cannot bypass an
exclusive controller. Reads remain available. Frontend/session expiry, end,
takeover, handoff and return invalidate queued old-epoch authority. An operation
already dispatched across the authority check may finish atomically: its actual
success is retained. Capture/discovery jobs deliberately continue, with frozen
inputs, until a separately authorized explicit cancel. Baseline save stays explicit.

During handoff only the following frontend may mutate; agent navigation/writes
pause. Only that GUI may return control, subject to its local draft/modal/task
guard. Immediate human takeover revokes the agent rather than extending or
renewing its lease. Frontend disconnect/expiry, engine restart or failed/draining
engine state release control without cancelling jobs. Control state is ephemeral
and bounded (128 session records, 16 live frontends); job receipts remain durable.
Secrets never appear in public snapshots, jobs, reports or logs. Same-user local
credentials are coordination, not a process sandbox. Native OS input/accessibility
and Windows/macOS control qualification are not inferred from Linux IPC/offscreen
tests; see the exact Phase 4 ledger.

The GUI finishes a capture task from its `Captured` record and frozen details.
Workspace refresh is a separate read, subject to the existing draft/error guards.
Passive updates pin the inspected run/analysis; deliberately loading evidence or
continuing a completed task may choose the newest workspace evidence. The task's
outcome and run identity are never substituted with that workspace run. Ordinary
refresh/history/viewer reads do not produce authoritative capture receipts.

Viewer reads return bounded metadata and immutable verified tile file DTOs.
The engine owns revision-bound evidence/source/plan caches (128/32/16 MiB); the
GUI reads/hashes/decodes those files without a live database handle and owns a
64 MiB/256-tile native image cache, latest-intent mailbox and silent prefetch.
Caches are display-only, never approval/export/deletion authority. Historical
inspection retains recorded analysis settings rather than borrowing today's.

### Focused DOM client contract

`Operation::Dom(engine::dom::Command)` and `Outcome::Dom(Box<engine::dom::Output>)`
also map to native `jobs::Command::Dom` / `JobReply::Dom`. The strict public DTOs
live in `crates/app/src/engine/dom.rs`; core owns every canonical source, policy,
comparison, diff and review model. `Selection {project,run,analysis}` uses
`DomAnalysisId`, independently of screenshot `AnalysisId`.

| Native request | Focused reply |
| --- | --- |
| `Source {selection,key,pane: Before\|After,format: Canonical\|Plaintext,offset,limit}` | `Source {selection,key,pane,format,source: DomSourceIdentity,page: SourcePage}` |
| `Diff {selection,key,options: DomDiffOptions,offset,limit}` | `Diff {selection,key,before,after,outcome,options,page: DiffPage}` |
| `Artifacts {selection,key}` | Exact optional `Original {source,artifact,metadata}` Before/After, canonical result without body, text-free `diff_summary`, capture outcome/warnings/privacy/coverage |
| `Results {selection,offset,limit,page,profile,outcome,capture_state}` | Resolved selection, state/baseline/coverage, bounded rows with canonical result, `diff_summary`, authoritative current DOM reviews, omissions and next offset |

`SourcePage = {offset,next_offset,total_bytes,text}`. `DiffPage =
{summary,hunks,offset,next_offset,total_rows}`; `DiffSummary` preserves exact
changed/old/new/added/removed line counts, fallback/display-limit state and total
emitted rows. `HunkPage` carries original `old_start,old_lines,new_start,new_lines`
plus `row_offset,total_rows,rows: Vec<core::DomDiffRow>`. It is explicitly a view
window, not an invalid partially rewritten canonical hunk. GUI replies can be
bound to returned immutable selection/source identities and the echoed diff options.
Native code should request relevant pages off the UI thread and reject replies
for an older selection/generation. `ViewerSelection` remains screenshot-specific;
DOM-only runs reject that image route with an actionable error.

Commands: `Results`, `Artifacts`, `Source`, `Diff`, `Inspect`, `Analysis`,
`SelectAnalysis`, `Reanalyze`, `Review`, `SetReview`, `ReviewHistory`, `Approval`,
`Accept`. Source/Artifacts resolve the check's original DOM reference using
kind-specific storage readers; screenshot corruption is not a fake DOM image.
Results/artifacts carry text-free count/limit summaries; inspection retains
canonical outcomes and source identities without embedded diffs. Source pages
are at most 256 KiB, byte-offset UTF-8 safe; diff pages at most 1,000 emitted core
rows within the core's 2 MiB text budget. Hunk headers describe full bounded hunks;
only the row window changes. Replies stay below the 64 MiB IPC ceiling, including
JSON escaping; jobs/events lists remain body-free metadata. Focused read receipts
are reclaimable like other reads, not mutation authority.

DOM results also enforce 4 MiB of encoded JSON per page, including escaping and
current notes. The returned cursor advances by actual rows, which can be fewer
than the requested limit. Counting is streaming/bounded, not an allocation of the
oversized frame. A 16 MiB pure-read response guard replaces unexpected oversized
output with `response_too_large` before receipt persistence; it does not label a
read as uncertain, set failed/draining, or prevent later jobs.

Consequential kind acceptance regenerates and compares every displayed field:
nested baselines, inherited counts, provenance and environment labels as well as
eligibility contexts. Only the proposal's generated new-version ID/date are
carried across a stateless refresh, consistently in every nested view. Consent
displays the freshly validated canonical view, never unchecked supplied labels.
Terminal report cancellation recognizes established typed cancellation through
the error chain. A completed `Ok` export is never changed by a later cancel flag.

`Approval(KindSelection)` explicitly supplies optional screenshot and DOM
`{analysis,selected}` lists. `Accept(KindApprovalPreview)` requires exact engine
preparation/confirmation and revalidates project/run/display/current-parent/
checked-baseline/policy/source identities before the atomic storage commit. An
unselected sibling is inherited, never approved. A fresh preview can safely rebase
after sibling approval; stale tokens cannot. Failed/noncomparable requested
coverage takes CLI exit 3 precedence over either kind's change (1); complete
unchanged coverage is 0.

Only explicitly named test-support constructors use the old in-process actor;
production never silently falls back. The historical `regressee-flow` developer
tool must acquire the same data lock before opening/recovering a store; the
foundation tool remains fresh-isolated-root-only and locks too. Tests and
developer fixture tools are not authorization to open an actual user store.
