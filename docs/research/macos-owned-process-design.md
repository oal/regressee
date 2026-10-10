# macOS exact-owned process lifecycle — independent design research

Prepared **2026-10-10**. Research only: **NOT implemented, NOT tested on a Mac,
NOT platform-qualified, NOT a Phase 7 pass**. Application, Cargo, status,
decisions, plans, build scripts and verifier evidence were not changed. No build,
runtime, browser, native test, service installation or nested agent was run.

## Verdict

**Keep the existing macOS capture refusal.** A small unprivileged guardian can
provide a sound ownership boundary for its direct children and carefully pinned
private process groups. It cannot, by itself, guarantee bounded cleanup of the
entire **unchanged Playwright + full Chromium macOS bundle** tree.

There are two different escapes to address:

1. Playwright 1.56.1 launches browsers with `detached: true` on non-Windows;
   Chromium therefore leaves Node's session/group. This escape can be mediated
   by a private launcher/broker, before the browser runs. [P1, P2]
2. The full Chromium 141 macOS crash reporter starts Crashpad through a
   fork/`setsid`/spawn sequence whose intermediate parent exits. The handler is
   outside the browser group and no longer a descendant observable through its
   current PPID. Crashpad normally exits on a Mach no-senders notification, but
   that is cooperative execution, not a deadline-bounded external termination
   authority. A stopped/stalled handler is a concrete counterexample. [C1–C4]

The second escape is not fixed by polling faster, watching `NOTE_FORK`, inheriting
a scratch nonce, or killing Node's/browser's group. There is no established
public, unprivileged, automatic descendant-tracking-and-termination boundary in
the examined Darwin APIs comparable to the current Windows Job implementation.
This is a scoped conclusion about the examined primitives and upstream behavior,
not a claim that no future Apple API or differently cooperating browser could
solve it.

**Viable conditional path:** a private guardian plus mediated, pre-registered
launches, with a complete contract covering *every* group/session escape. That
requires upstream/browser cooperation for Crashpad and any other escaping
launcher, or primary proof of a suitable supported public OS primitive. Neither
is established here. A guardian-only change is consequently a bounded research
prototype, not sufficient code-gap closure or permission to enable capture.

## 1. What the actual repository does

Read-only observations, by file/function rather than mutable verifier results:

| Surface | Relevant current behavior |
| --- | --- |
| `crates/runtime/src/supervisor.rs`, `Worker::spawn_network` | Acquires a worker/browser permit, then rejects platforms other than Linux/Windows with the explicit macOS owned-tree message. Clears environment, uses verified private runtime paths/direct argv, creates private HOME/TMP roots and pipes. Unix launch calls `setsid`; only Linux has `PR_SET_PDEATHSIG` and the parent-race check. No launch deadline/cancel argument exists on `spawn_network`. |
| Same file, `request`, `stop`, `kill_tree` | Bounded request framing and correlated events; cancellation attempts a Cancel request before tree cleanup; stop sends Shutdown. Nonblocking pipe setup is **Linux-only**, so merely adding macOS to the allowlist would leave a blocking-write deadline hole. Linux-only browser sweeps surround killing/reaping Node. Reader joins and `Child::wait` are not a fully bounded macOS cleanup adapter. |
| `crates/runtime/src/scratch.rs` | Linux-specific short-path/socket budgets, 0700 namespace, 0600 atomic ownership marker, UID/executable/exact `TMPDIR` matching through `/proc`, dead-parent recovery. Snapshot checks followed by numeric `kill` are not a stable process handle or an atomic identity-checked macOS signal operation. This research does not reverify Linux. |
| `crates/runtime/src/windows_process.rs` | Restricted inherited handles, Job assignment in `CreateProcessW`, no breakaway, `KILL_ON_JOB_CLOSE`, Job termination. This kernel ownership model must not be inferred for Darwin groups. |
| `crates/runtime/src/windows_scratch.rs` | Exclusive file lease rather than stale PID authority; lease-free scratch recovery relies on the separate Job lifecycle guarantee. A macOS lease alone would not establish browser death. |
| `crates/core/src/platform.rs` | Unix filesystem space/permission utilities, not macOS process supervision. `crates/runtime/src/lib.rs` has no macOS ownership module. |
| `worker/product/src/capture.mjs`, `launch` | `chromium.launch({channel:'chromium', headless, chromiumSandbox:true, ...proxy})`; no product launch broker/gate or explicit product launch deadline. Requires the full Chromium channel, not a headless-shell substitution. |
| `worker/product/src/index.mjs`, `run` | Handshake awaits launch before assigning `browser`; OpenLogin separately awaits another headed launch before assigning `login`. Cancel closes the current context, but an in-flight launch may not yet have a context/browser handle. Shutdown/EOF/SIGTERM handlers close only handles already assigned. SIGKILL cannot run these handlers. |
| `.runtime/installed/worker/node_modules/playwright-core/lib/server/utils/processLauncher.js` | Actual retained package confirms detached launch and `process.kill(-spawnedProcess.pid, 'SIGKILL')`, as in tagged upstream [P1]. This package is Linux evidence, not macOS behavior qualification. |
| `crates/runtime/src/bundle.rs`, `distribution.rs::Manifest` | Authenticated v3 links and exact file modes preserve bundle layout. Manifest currently names Node, worker, browser root and an exact file inventory; it has no guardian/launcher entry point. Bundle preservation supplies no process ownership. |
| `crates/app/src/product.rs::spawn_worker` | Calls `Worker::spawn_network` without cancellation/deadline; setup passes cancellation later to handshake. The new launch contract must reach this call and the other worker-spawn callers, not just `handshake`. |

`docs/phases/07-status.md`, “Remaining code versus qualification gaps”, calls
macOS supervision a critical **code** gap and deliberately blocks capture.
`docs/support-matrix.md` targets macOS 14.7 / Apple Silicon, with Intel separate if
retained; `docs/release-building.md` specifies deployment target 14.0. This report
does not change either target or the independent verifier's active work.

## 2. Primary-source primitive inventory and limits

All links below were accessed **2026-10-10**. Apple source examples are pinned to
`xnu-10063.141.1`, not inferred from a Linux implementation. Exact SDK symbols,
shipped OS-build mapping and availability on the actual 14.7 machine still need
native confirmation. Availability of an individual function is not proof of an
ownership composition.

| Primitive | Availability/proof | What it does **not** prove |
| --- | --- | --- |
| `posix_spawn`, spawn file actions, `POSIX_SPAWN_SETPGROUP` | Apple manual dated Aug 9/Aug 22, 2007; public Darwin header [A1, A2]. Group attributes apply before user execution. | Parent death does not automatically cancel a pending spawn or kill the child/tree. A synchronous spawn syscall is not a cancellable hard-real-time operation. |
| `POSIX_SPAWN_START_SUSPENDED` | Documented Apple extension in the Aug 22, 2007 manual; present outside `PRIVATE` guards in the examined header [A1, A2]. Kernel stops the task before user execution [A3]. | Suspended launch is not ownership; a launcher killed before resuming/cleaning can strand a suspended child. It must be used by the surviving guardian, not as an app-to-guardian launch assumption. Resume/kill of an unreaped direct child can use normal signals; kernel signal handling provides resume behavior [A3]. Native proof is still required. |
| `POSIX_SPAWN_SETSID`, `setsid`, `setpgid` | `SETSID` is in the examined non-private header. Node's vendored libuv explicitly gates its use at Darwin 19 / macOS Catalina 10.15; older paths use fork/`setsid` [P2]. Session/group syscall rules and cross-session rejection are in Apple source [A4]. | A process group is not a containment object. Descendants can create another group or session. Do not invent a universal historical introduction version for the spawn flag from its constant alone. All proposed production use is at the existing macOS 14+ floor. |
| `kqueue` / `EVFILT_PROC`, `NOTE_EXIT`, `NOTE_EXEC`, `NOTE_FORK` | Public Darwin header/kernel [A5]. Can watch an already-bound process for exit; request registration results and handle `ESRCH`/permission failures. | `NOTE_FORK` does **not** return the child's PID in the delivered event. Events are not a reliable enumeration of each descendant; a watch is not a signal handle. |
| `NOTE_TRACK`, `NOTE_TRACKERR`, `NOTE_CHILD` | Apple header: “no longer supported as of 10.5”; the examined kernel explicitly returns `ENOTSUP` [A5]. | FreeBSD descendant-tracking semantics cannot be claimed for macOS merely because the constants remain defined. |
| `proc_pidinfo(...PROC_PIDTBSDINFO...)` | `libproc.h` availability annotation: macOS 10.5. Public `proc_bsdinfo` contains UID, PID, PPID, PGID and start seconds/microseconds [A6]. | Start-time + executable/UID/ancestry checks followed by `kill(pid, sig)` leave a check/use race. PID exit/reuse can occur between the final check and signal. Wall-clock start fields are not an atomic ownership capability. |
| `proc_listallpids`, `proc_listchildpids`, `proc_listpgrppids`, `proc_pidpath` | Header annotations: list convenience functions 10.7, path 10.5 [A6]. Useful for observation/refusal diagnostics. | Snapshots are not complete lineage history, a sandbox may restrict inspection, PPID changes on reparenting, and executable identity is not per-job ownership. `proc_pidfdinfo` inspects a target's FD; its name does not mean Linux `pidfd`. |
| `proc_terminate(pid, &sig)` | Present in examined Apple source [A7]. It implements sudden-termination policy and selects SIGTERM or SIGKILL. | Not an identity-conditioned general signal API, not a force-kill guarantee, not process-tree ownership. |
| `proc_signal_with_audittoken` | Important **qualified finding**: this actually exists in the examined Apple header/kernel. Kernel retains a process reference, checks the token's PID version and signal permissions, then signals it [A7]. | Do not falsely claim that Darwin has no identity-aware signaling internally. Conversely, the header describes its interfaces as private and this function has no introduction annotation there; supported **public** API status/version contract was not established from primary Apple documentation. No use is proposed under the no-private-API constraint. Even if approved, signaling a known occurrence is not discovery/ownership of all detached descendants. |
| `PROC_PIDUNIQIDENTIFIERINFO`, extended PID-info comparisons | Kernel implements unique-ID/version flavors; the extra flavor definitions/structures are in `proc_info_private.h`, not the examined public flavor list [A6, A8]. | Do not copy private constants/structs into Rust and call it public libproc support. Do not equate a readable unique ID with an atomic `kill` condition. |
| Mach task/name ports or XPC invalidation | Task-name information is not a task-control port. Apple kernel distinguishes task-name conversion for most `task_info` from stronger task-read access for dyld information [A9]. | No proven unprivileged, public, automatic descendant-control port acquisition was found. A port useful for exit/identity observation is not permission to `task_terminate`; `task_for_pid` entitlement/signature bypasses are excluded. XPC connection loss is not proof that all its process descendants were terminated. |

### Stable ownership that *can* be composed

An unreaped direct child is different from an arbitrary PID found in a snapshot.
The sole owner must retain its wait/reap authority, reset inherited SIGCHLD
disposition, and avoid `SIG_IGN`/`SA_NOCLDWAIT`, competing reapers or premature
`Child::try_wait` (which may reap). Apple PID allocation excludes existing process
IDs, including zombies, and existing group/session IDs [A10].

A direct child that establishes a **fresh session and leader group before it
launches any workload**, and remains live or unreaped until group cleanup finishes,
can pin that group's numeric identifier. Other sessions cannot join it [A4].
Consequently a guardian outside the group can signal that **established owned
group**, without the arbitrary-PGID recapture race, under this explicit invariant.
No previously persisted PGID can recreate that authority after the guardian dies.

This is an ownership argument for the pinned group, **not** for processes that
leave it. Same-session new groups are not reached by `kill(-pgid, ...)`; new
sessions are not reached either. The app's original session/group must never be
used. Group enumeration, `kill(..., 0)`, or successfully sending SIGKILL does not
by itself establish complete termination or bounded reaping.

## 3. Concrete approaches compared

### A. App/libproc watcher plus private scratch nonce — reject as exact supervision

Enumerate PID/PPID/UID/start-time/path; watch found PIDs with kqueue; scan the exact
private scratch/profile token and recheck before TERM/KILL.

Advantages: small adapter, useful diagnostics, preserves unrelated processes when
it refuses ambiguous observations. Problems: launch before registration;
unobserved short-lived intermediate parents; reparenting; sandbox/inspection
failures; separate snapshot-versus-signal TOCTOU; no app-SIGKILL actor. Adding an
ephemeral watcher fixes only the last item. No finite polling interval removes
the others. Safe policy: observe/refuse, **do not authorize uncertain signals**.

### B. Surviving guardian + brokered, pinned private groups — useful but incomplete

Concrete topology:

```text
native app -- sole liveness pipe --> guardian (outside worker/browser groups)
                                    |-- owned Node session/group
                                    |-- owned browser anchor session/group
                                    |       `-- actual Chromium .app executable
                                    `-- second anchor/group for headed login
Node/Playwright -- detached launcher shim --> guardian launch request
```

The detached Playwright child is only a trusted no-workload shim. It forwards
Playwright's pipe FDs/argv to the guardian and requests a browser; it must not
`exec Chromium` before registering. Each guardian-owned anchor establishes its
own session, gates workload launch, and pins its group until cleanup. Node's
automatic reaping of its own Playwright child cannot release the guardian's
anchor authority. The shim/anchor must preserve FD 3/4 remote-debugging pipe
semantics, stdout/stderr, exit notification and backpressure. Upstream uses five
stdio slots and `PipeTransport(stdio[3], stdio[4])` [P1, P3]; this is not a shell
wrapper or a pathname-only browser discovery scheme.

**Additional shim limitation:** unchanged Playwright still spawns that detached
shim before the guardian knows it exists. App/Node death, or SIGSTOP of the shim
before its first message, can strand the shim itself. A no-workload gate prevents
an unowned *browser* launch, not an exact no-leaked-*helper* guarantee. Thus an
executablePath wrapper alone is not an exact-owned implementation, even for
otherwise group-safe browsers.

An explicit Playwright launcher-source adaptation would instead have the
guardian create **all** launch roots, including any shim, before exposing their
transport to Node. It must replace upstream negative-PID force-kill with an
owned-close control operation, not return a numeric guardian-created PID and
continue using Node's original group-kill implementation. This is a source
adaptation requiring compatibility work, not an existing public launch option.
Simply setting `detached:false` is insufficient unless browser and all descendants
remain in the pinned owned group. Neither alternative should monkey-patch global
Node spawn functions.

Such a mediated launcher can close the known Playwright root-launch escape and
registration race; the wrapper alone only gates browser admission. The unchanged
macOS Crashpad sequence still bypasses it. Crashpad explicitly closes non-listed
FDs during spawn [C2], so “inherit a guardian/lease FD into every descendant” is
also not a solution. Renderer/GPU/utility launch source examined uses normal Mac
launch options and preserves the sandbox setup [C5], but that observation is not
an exhaustive proof of every executable/feature/relaunch path. No claiming an
escape-free full bundle from it.

### C. Fully cooperating spawn boundary — potentially sound, not a minimal delivery

Keep B, but arrange that **every** group/session-changing or orphaning launcher,
including Crashpad handler start/restart and its optional self-monitor, either:

- stays within a pinned owned group for its entire lifetime; or
- asks the guardian for a pre-registered launch into another pinned owned group,
  with a gate that cannot release workload after cancellation.

The guardian then has a finite registry of non-recapturable group authorities;
app/worker/browser death need not depend on the deceased process executing JS.
This is the only examined architecture with a plausible complete ownership
argument while staying unprivileged and keeping Chromium's renderer sandbox.

It requires source-level browser/cooperation work, a complete spawn/relaunch audit
and compatible transport/Mach rendezvous changes. The examined Mac Crashpad path
locates its handler inside the framework and launches it internally [C1]; it does
not provide a demonstrated public per-launch guardian callback. Do not replace
that signed in-bundle handler with a wrapper, patch bundle bytes, inject a dylib,
turn off the sandbox, or assume `--disable-breakpad` suppresses it. Chromium's
own source says Crashpad dumping is unconditional despite that switch [C3].

An owner-built and normally signed/requalified browser with explicit cooperation
is a possible separate proposal, **not** modifying an upstream bundle while
pretending its original signature remains valid. That exceeds “just add a helper”
and is not recommended as an already-sound bounded implementation commitment.

## 4. Conditional guardian contract and integration boundary

The following is a specification for a limited prototype/future cooperating
runtime. **It does not establish approach B as full-tree-safe.**

### Launch admission and death race

1. Validate verified runtime/helper entry points, private directory ownership and
   supported architecture before workload launch. Spawn a *running* guardian
   whose initial state cannot start Node/Chromium until an authenticated, bounded
   admission message is accepted. If the app dies before that message, no worker
   or browser may start. A suspended guardian with only the app able to resume it
   is not an acceptable parent-crash design.
2. Create the liveness pipe before guardian spawn. **Only the app retains its
   write end**; all unnecessary copies are closed and CLOEXEC. Guardian monitors
   EOF with kqueue. The worker, browser, shim and anchors must not inherit a copy
   that can mask app death. This detects death even before a PID-based exit watch
   could be registered; numeric parent PID/start time is not the liveness gate.
3. Guardian owns every admitted root from before its first workload instruction.
   Use gated anchors and, where appropriate, `START_SUSPENDED` for their direct
   workload children. Register/journal the ownership and available cancellation
   state **before** release. If recording/registration fails, kill the unreaped
   direct child/owned group without releasing it. No browser can self-launch from
   a shim whose guardian request fails.
4. Commit/EOF/spawn are not one atomic syscall. Treat a buffered commit followed
   by app death as an ordinary guardian cleanup path; recheck admission before
   release and retain authority if death occurs just after the check. Recheck
   deadlines after any slow spawn before release. No claim that a synchronous
   kernel exec/spawn is interruptible or has an absolute wall-clock completion
   bound; deadline expiry revokes release and bounds caller waiting, not OS work.

### Cancellation, deadline, graceful and forced cleanup

- Monotonic operation deadlines and cancellation must exist **before** spawn and
  apply to helper startup, every browser launch (including OpenLogin), request
  pipe writes, shutdown and teardown. Use separate worker-operation and bounded
  cleanup budgets; do not silently spend an unbounded new grace period after an
  expired deadline. No successful result after cancellation won admission.
- Guardian serializes `Launching -> Running -> Closing -> Closed/Incomplete`.
  Closing first revokes all new launch tickets, including an in-flight launch
  that has not yet returned a browser to JS. Worker launch results must check
  closing/cancel state, close late browsers and never publish Ready/LoginOpened
  after terminal cancellation. Preserve protocol v2 request correlation.
- Normally send existing Cancel/Shutdown and request browser/context close. EOF
  or worker failure invokes the same registry cleanup even if no JS handle was
  assigned. Then send TERM only to owned authorities, allow a configured grace
  within the cleanup budget, and KILL still-owned pinned groups/direct children.
  Stopping pinned groups before a final forced sweep can help quiesce known
  spawn sources; it is **not** a descendant discovery or escape-prevention API.
- Single owner reaps direct children **after the last group signal**. Never issue
  another numeric group signal after releasing its anchor/reap authority.
  Read/drain pipes nonblockingly; close endpoints and bounded reader loops rather
  than joining forever on a descendant holding stdout. Report `Incomplete` if
  termination/reaping cannot be confirmed in the budget. SIGKILL acceptance does
  not imply synchronous process disappearance, especially during kernel/I/O work.
- The guardian is a user process, not a kernel kill-on-owner-close object. The
  fault model needs it to survive and be scheduled after app death. Guardian
  SIGKILL/OS failure, or simultaneous death of all cooperative cleanup actors,
  cannot honestly have the same guarantee. Include this failure in tests and
  preserve dirty recovery state rather than advertising universal crash cleanup.

### Proposed shared-supervisor interface

Keep `ProcessSupervisor`'s framed requests/events and verified-runtime resolution.
Add a deadline/cancellation-bearing worker launch context, passed through
`Product::spawn_worker` and all capture/auth/crawl/setup launch sites. Introduce a
target-specific owned-process adapter rather than adding macOS to the existing
Unix fallback:

```text
OwnedLaunch(verified runtime, private scratch lease, deadline, cancellation)
  -> OwnedWorker { worker IPC, private lifecycle control, ownership authority }
Close(reason, monotonic deadline)
  -> Closed | CleanupIncomplete   // never equate a sent signal with Closed
```

`OwnedWorker` would cover `macos_process`/`macos_scratch` modules, a guardian
control channel distinct from site/session IPC, bounded pipe/readers, single
reaping and idempotent stop/Drop behavior. Guardian IPC should contain bounded
typed lifecycle records, not cookies, login material, URLs or raw browser logs.
Drop is best-effort local revocation; the surviving guardian supplies the
app-death actor. Exact helper startup/operation errors must remain visible.

### Minimum bundle/helper changes for **B only**

- One normal app-owned, non-setuid Rust helper with guardian/anchor/shim roles, or
  separate small helpers; resolved only from a reviewed app bundle location, not
  project argv/PATH. They need no daemon, LaunchAgent registration, system
  installation, administrative privilege or new entitlement for this design.
- Place helpers **outside** the upstream Chromium `.app`; launch its unchanged
  `Contents/MacOS/...` executable with direct arguments. Keep its complete
  framework links, modes and sandbox-related helper layout. No quarantine
  stripping, code-signature weakening or sandbox-default changes.
- Worker uses a validated macOS-specific executablePath/shim for admission-only
  experiments, or an explicit launcher-source adapter for an exact-root design;
  do not accept that path from project/portable data.
  Node-to-browser pipe compatibility and the full channel behavior require native
  testing; custom executablePath is not automatic upstream compatibility [P3].
- If helpers instead ship in the runtime pack, add explicit authenticated helper
  entry points/version compatibility to a reviewed manifest revision; merely
  finding a helper by basename in `files` is not the required resolver contract.
  If in the app, bind helper build/version to the runtime adapter. Existing v3
  layout support alone does not implement either option. Normal owner signing
  remains a separate gate, not performed by this research.
- Full-tree C needs browser-internal changes beyond this minimum. B must remain
  restricted to synthetic group-safe research workloads until that gap is solved.

## 5. Stale scratch and PID/PGID reuse safety

Use a new macOS-specific namespace/marker version. Do not import the Linux byte
limits: Darwin Unix-domain socket/path limits and Chromium profile suffixes must
be measured for this bundle. A short owner-private namespace can be prepared
under a validated macOS temporary root; do not hardcode Linux `/proc`, assume
`/tmp` is not an alias of `/private/tmp`, or scan a caller's arbitrary TMPDIR.

Proposed marker: bounded schema with namespace/version, UID, random per-worker
ID, validated scratch directory identity, authenticated runtime identity, helper
protocol version, diagnostic root PID/start observations and lifecycle state.
An atomic 0600 marker replacement plus directory-relative, no-follow operations
should preserve unmarked/foreign/malformed/linked paths. Keep retained DB/CAS and
runtime generations completely outside recovery's deletion scope. Bind directory
FD/device/inode through cleanup; fail on ambiguity rather than repairing somebody
else's permissions. Same-UID malicious tampering is not defeated just by a nonce
or 0700, and must not be newly claimed as covered.

An exclusive lease held by the guardian prevents sibling apps from recovering a
live owned scratch. Handoff must not have an unlocked interval, and workload
children must not mask lease release or app death. However:

- **Lease-free + dead app/helper does not mean no live Chromium descendant.**
  Unlike Windows, no Job closed automatically. It can mean the guardian crashed.
- A marker's PID/PGID/start time/path is diagnostic evidence, **not revived signal
  authority**. Never TERM/KILL based on a stale record even after rechecking.
- Allow automatic removal only when a validated marker records completed exact
  cleanup under a proven ownership contract and the lease is exclusively held by
  the recovery owner. Retain DIRTY/Incomplete or ambiguous entries for explicit
  recovery; a free lease or an empty ancestry snapshot is not sufficient.
- A reliably established reboot can establish absence of *previous-boot*
  processes, but this report selects no supported boot-identity API as proof.
  Do not infer it from PID reuse, timestamps alone, or a made-up boot UUID API.
  A future reboot-based recovery rule needs separate primary API proof and tests.
- For an old still-live guardian, reconnect through a protected, versioned control
  endpoint and let **that guardian** use its retained authority, rather than
  recapturing its group. Authentication/endpoint identity must fail closed; no
  broad-name process search fallback.

This preserves unrelated processes and scratch but may retain private dirty
profiles after an unresolvable guardian failure. That privacy/storage limitation
must be visible; deleting such scratch while an uncertain process may use it is
not a correct recovery “success”.

## 6. Bounded next step and real-Mac experiment gates

**Recommendation:** retain refusal, then seek authorization for a small isolated
guardian/group-safe fixture prototype. Do not start production integration or
claim a viable minimum-helper full-browser implementation until the escaping
Crashpad/cooperation question is resolved. Stop after the following bounded
questions rather than rewriting the browser speculatively:

1. Confirm actual macOS 14.7 headers/symbols and primary public-support status of
   any proposed identity-aware API; verify suspended-start behavior and pinned
   direct-child/group semantics as an ordinary user. No entitlement/signature
   workaround. If no acceptable all-descendant contract exists, record **blocked**.
2. With the exact verified full app bundle, observe every root/helper/relaunch
   path and Crashpad group/session/lifetime, both new-headless and headed login.
   Demonstrate the stopped-handler counterexample and whether any documented
   supported cooperation facility changes it. Healthy self-exit is not a pass
   for forced cleanup. Nothing in this step changes sandbox flags/bundle bytes.
3. Prototype pre-admission/gated launching and liveness EOF with synthetic
   fixtures only. Establish ownership/reaping/transport correctness independently
   of browser behavior. Only if the complete escape contract is solved should
   production adapter/worker/bundle integration be separately authorized.

Required **future** native fixtures (all NOT RUN here):

| Fixture | Required assertion |
| --- | --- |
| Cancel/deadline before spawn, during helper admission, suspended-child registration, before release, during Playwright launch, and before JS stores the handle | No unauthorized workload instruction; all admitted children remain cleanup-owned; bounded caller result; no late Ready/LoginOpened. Inject delays/barriers, not sleep-based timing guesses. Include SIGSTOP of an executablePath shim before its first guardian message; wrapper-only supervision must not claim no leaked helper. |
| App SIGKILL before guardian starts, before Node launch, during headless/headed launch, after capture, and while idle | Guardian EOF sees death without PID polling; registered roots/groups terminate; no leaked actual bundle helper, renderer, GPU, utility or Crashpad process. |
| Worker SIGKILL/crash while browser survives; browser-root, renderer, GPU and Crashpad crashes separately | Full owned-tree cleanup, preserved originals/DB/CAS, correct typed failure, then sibling and same-app recapture succeed. Include late/restart spawn. |
| Stalled SIGSTOP browser/root/helper, ignored TERM, blocked stdout/stderr/worker stdin, stopped Crashpad and monitor-self | Grace cannot hang forever; forced cleanup has exact authority or explicitly fails qualification. No “browser closed” claim from JS/parent exit alone. |
| Launch/exit/fork barriers and short-lived orphaning intermediate parent | Prove no descendant is missed before registration. Add a deliberate group-changing/`setsid` fixture; group-only supervision must reject it without a cooperation contract. |
| Two concurrent workers/apps, headed login alongside main browser, old surviving worker/browser plus new recapture | Cleanup affects only the selected job; sibling and unrelated session remain alive; no shared runtime-root/process-name targeting. |
| Unrelated same-UID same-executable process, foreign session, copied nonce, PID/start-time mismatch, stale PGID; real PID reuse when feasible | No signals to unrelated processes. A deterministic fake process-observer/signal-boundary fixture exercises check/use reuse; real OS PID turnover supplements it, never permission to reuse someone else's PID. Test early reaping explicitly. |
| Dirty marker, crash during marker publish, helper SIGKILL, free lease with surviving child, linked/malformed/foreign namespace, live sibling lease | No stale-record kill, no deletion of ambiguous/live scratch, no symlink traversal or retained data deletion. Reconnect only to authenticated still-live authority. |
| Actual full `.app`, both retained architectures independently | No flattening/signature weakening, no sandbox-disabled launch, no undocumented API dependence. macOS SDK/desktop execution and helper compatibility evidence are necessary; Linux/source inspection cannot substitute. |

Retain owned PID/group **observations** and timeline/state/return codes in private
evidence without site/auth contents. Use independent observation plus fixtures'
own liveness witnesses; don't derive a no-leak verdict solely from the supervisor's
registry, which is precisely what an escape can evade. No scale, packaging,
licensing or Phase 7 verifier suites were duplicated by this research.

## 7. Source ledger

Primary links and the versions examined; access date **2026-10-10** throughout.
Downloaded textual source snapshots/hash inventory are private research outputs:
`/path/to/private-tests/macos-owned-research-20261010-sol-233940/`.
They are not executable downloads or runtime artifacts. `xnu14-*` and
`pinned-*` snapshots are the controlling evidence; additional `main` snapshots
were exploratory and do not establish deployment availability.

- **P1** Playwright **v1.56.1**, [processLauncher.ts](https://github.com/microsoft/playwright/blob/v1.56.1/packages/playwright-core/src/server/utils/processLauncher.ts): detached spawn, stdio, exit/signal handling, negative-PID group kill. [browsers.json](https://github.com/microsoft/playwright/blob/v1.56.1/packages/playwright-core/browsers.json) pins full Chromium revision 1194 / **141.0.7390.37**.
- **P2** Node **v22.20.0**, [child_process.md, detached](https://github.com/nodejs/node/blob/v22.20.0/doc/api/child_process.md#optionsdetached) and its [vendored libuv process.c](https://github.com/nodejs/node/blob/v22.20.0/deps/uv/src/unix/process.c), `uv__spawn_init_can_use_setsid`/spawn flags.
- **P3** Playwright **v1.56.1**, [browserType.ts](https://github.com/microsoft/playwright/blob/v1.56.1/packages/playwright-core/src/server/browserType.ts), preparation/custom executable, pipe transport, close timeout; [chromium.ts](https://github.com/microsoft/playwright/blob/v1.56.1/packages/playwright-core/src/server/chromium/chromium.ts) and [chromiumSwitches.ts](https://github.com/microsoft/playwright/blob/v1.56.1/packages/playwright-core/src/server/chromium/chromiumSwitches.ts), channel/arguments including `--disable-breakpad`.
- **C1** Chromium **141.0.7390.37**, [crashpad_mac.mm](https://github.com/chromium/chromium/blob/141.0.7390.37/components/crash/core/app/crashpad_mac.mm), `PlatformCrashpadInitialization`; [chrome_main_delegate.cc](https://github.com/chromium/chromium/blob/141.0.7390.37/chrome/app/chrome_main_delegate.cc), `InitMacCrashReporter`; [bundled crashpad_client_mac.cc](https://chromium.googlesource.com/chromium/src/+/141.0.7390.37/third_party/crashpad/crashpad/client/crashpad_client_mac.cc), `HandlerStarter`/`SpawnSubprocess`, restart semantics.
- **C2** Same exact Chromium tag's bundled Crashpad [spawn_subprocess.cc](https://chromium.googlesource.com/chromium/src/+/141.0.7390.37/third_party/crashpad/crashpad/util/posix/spawn_subprocess.cc): intermediate fork, `setsid`, orphaning, Apple CLOEXEC-default spawn.
- **C3** Chromium **141.0.7390.37**, [crashpad.cc](https://github.com/chromium/chromium/blob/141.0.7390.37/components/crash/core/app/crashpad.cc), `SetUploadConsent` comment expressly distinguishes Crashpad dumping from the old disable-breakpad switch.
- **C4** Same exact Chromium tag's bundled Crashpad [exception_handler_server.cc](https://chromium.googlesource.com/chromium/src/+/141.0.7390.37/third_party/crashpad/crashpad/handler/mac/exception_handler_server.cc): Mach no-senders notification, server-loop termination. This proves intended cooperative exit, not a forced-termination deadline.
- **C5** Chromium **141.0.7390.37**, [launch_mac.cc](https://github.com/chromium/chromium/blob/141.0.7390.37/base/process/launch_mac.cc), `LaunchProcess`/optional new process group, and [child_process_launcher_helper_mac.cc](https://github.com/chromium/chromium/blob/141.0.7390.37/content/browser/child_process_launcher_helper_mac.cc), native child/sandbox/Mach rendezvous preparation.
- **A1** Apple archived [posix_spawn(2)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/posix_spawn.2.html), manual Aug 9, 2007; [posix_spawnattr_setflags(3)](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man3/posix_spawnattr_setflags.3.html), manual Aug 22, 2007. Archived documentation is historical API evidence, not current native qualification.
- **A2** Apple **xnu-10063.141.1**, [bsd/sys/spawn.h](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/sys/spawn.h), documented/public versus `PRIVATE`-guarded flags.
- **A3** Same XNU tag, [kern_exec.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/kern/kern_exec.c), START_SUSPENDED task stop; [kern_sig.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/kern/kern_sig.c), `psignal` resume/kill behavior and group signaling. Direct-child resume/cleanup composition is still an experiment gate.
- **A4** Same XNU tag, [kern_prot.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/kern/kern_prot.c), `setsid`/`setpgid`, same-session and session-leader constraints; Apple [setsid(2)](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/man/man2/setsid.2), [setpgid(2)](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/man/man2/setpgid.2), [kill(2)](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/man/man2/kill.2), [wait(2)](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/man/man2/wait.2).
- **A5** Same XNU tag, [event.h](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/sys/event.h), child PID not delivered and tracking unsupported since 10.5; [kern_event.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/kern/kern_event.c), `filt_procattach`, explicit `ENOTSUP` and registration permissions/edge behavior.
- **A6** Same XNU tag, [libproc.h](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/libsyscall/wrappers/libproc/libproc.h) and [proc_info.h](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/sys/proc_info.h), annotated function availability, `proc_bsdinfo` start fields and public flavor list. Header itself warns about private interfaces; SDK inclusion/annotations are not a blanket public-support assurance for every listed symbol.
- **A7** Same XNU tag, [proc_info.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/kern/proc_info.c), `proc_terminate_with_proc`, `proc_terminate`, `proc_signal_with_audittoken` and token-version comparison while holding a process reference.
- **A8** Same XNU tag, [proc_info_private.h](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/sys/proc_info_private.h), additional unique-ID/version structures/flavors. Private interfaces are evidence of implementation, **not** selected APIs.
- **A9** Same XNU tag, [osfmk/kern/task.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/osfmk/kern/task.c), `task_info_from_user`. No pidfd-like public process-control guarantee is inferred from these task information conversions.
- **A10** Same XNU tag, [kern_fork.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-10063.141.1/bsd/kern/kern_fork.c), `forkproc` PID allocation excludes live/zombie process hash, group and session IDs. This supports the pinned-authority argument while the sole guardian retains ownership, not stale-record recapture.

**Open decision, not settled by this report:** whether an owner will pursue
complete browser spawn cooperation, obtain primary public-API proof that changes
the boundary, or retain macOS refusal. No research observation authorizes removing
the current code guard.
