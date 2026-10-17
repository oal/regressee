# Agent CLI Phase 2 — independent verification

**Final verdict: PASS for Phase 2, protocol 2, within the reviewed source,
saved-fixture and Linux IPC scope.** All three initial HIGH findings are
independently closed. No new serious repair regression was found. This is not
native Windows, actual live browser resume/login, native GUI, clean-host or
release qualification, nor completion of Phase 3/4.

Reviewed 2026-10-17 against uncommitted Phase 2 work on
`9daf121b960f84506feedefcdf49869327861130`. Read the plan, Phase 2 status,
`local-engine.md`, and accepted Phase 1 review. I am the independent verifier,
not the implementor. Only this report is added to the repository. No production
source/test insertion, revert, commit, push, publication, actual user-store open,
outside-site capture, auth-secret injection, or sandbox change was performed.

## Final independent re-review — 2026-10-17

Re-read the current status/engine contract and inspected the repair against the
first review's frozen source, rather than accepting its implementor summary.
Base remains `9daf121b960f84506feedefcdf49869327861130`; Phase 2 is uncommitted.
Only this report was edited in the repository. Original findings and first-run
evidence are retained below as history; their source line numbers describe the
initial implementation, not the repaired layout.

### Findings closed and analogous paths checked

1. **Replacement confirmation bypass — CLOSED.**
   `crates/storage/src/lib.rs:502–542,584–591` now routes ordinary environment
   saves through the same non-replacement transaction policy as `Save`.
   `SaveReplacementEnvironment` is explicit combined intent and is covered by
   exact confirmation in `crates/app/src/protocol.rs:98–113`. Engine, product
   and legacy test dispatch share `jobs::environment_work` rather than divergent
   save policies. Independently verified rejection without persisted changes,
   successful token-free safe name/discovery/local-login saves, and successful
   explicitly confirmed replacement. Checked wrong operation/settings tokens,
   consumed consent, same-ID retry, all three protected field families,
   baseline-pointer protection and atomic rollback. GUI IPC facade also rejects
   the unsafe ordinary save and succeeds through the explicit combined intent.
2. **Representative-preview success misclassification — CLOSED.**
   All three capture-producing operations now require `Captured` and use the
   same classifier (`crates/app/src/protocol.rs:79–85,311–394`). The actual-state
   matrix covers Completed, CompletedWithErrors, Cancelled, Paused, Interrupted
   and unexpected Running for all three operations. Missing/foreign/ordinary
   workspace results, wrong intent/analysis/progress identities fail closed;
   structured browser/worker requirements and cancellation remain structured.
   Successful valid captures and visual differences are not blanket-rejected.
3. **Resumed-run substitution — CLOSED.**
   All three producers retain the returned record and call the shared exact-run
   helper (`crates/app/src/jobs.rs:642–657,734–778,800–815`). Storage selects that
   saved run and its original check reference, without borrowing the newest
   run/analysis (`crates/storage/src/capture.rs:831–939`). Checked older runs with
   five different outcomes followed by newer successful/failed runs, a paused
   check with no analysis after active-baseline change, producer equality/foreign-
   project guards, and the GUI separation of task completion from workspace
   refresh (`crates/gui/src/main.rs:1283–1299` and its simulation regression).
   Own positive source-linked probe returns **Paused for exact run A**, while
   rejecting newest run B as A's receipt. No live browser resume is claimed.

Also reviewed the new `Captured`/combined-save mappings, GUI task/cache activity
paths and protocol-2 upgrade handling. Legacy unbound capture receipts preserve
their original payload/submit as `Interrupted`/`UnverifiedCapture`, including a
second restart and exact retry; they are not replayed or retroactively certified.
Protocol 1 and future protocol 3 were refused on the **actual Unix wire** before
any work, while protocol 2 returned the correct owner/snapshot. Ownership,
transport, viewer and native platform files are byte-identical to the first
independent audit (hashes rechecked). No direct-Store GUI fallback was introduced.
Epoch hooks remain hooks, not finished Phase 4 authorization.

### Exact re-review checks

Fresh re-review logs/scripts/source receipts are under:

```text
/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase2-review/rereview
```

Use the original environment below, plus `export R="$E/rereview"` and
`export TMPDIR="$R"`. Heavy compilation was serial and verifier-owned.
`$R/checks.json` retains expanded arguments, exits and timings;
`$R/platform-results.json` retains lifecycle/wire transcripts and unchanged
platform hashes. `$R/source-receipt.json`/`source/` freeze 42 reviewed source/Cargo
files. The initial report and original external witness source remain preserved.

| Exact command | Independent re-review result |
| --- | --- |
| For each original witness name below: `CARGO_TARGET_DIR=/var/home/olav/Projects/Regressee/target cargo test --locked --offline --manifest-path "$E/probe/Cargo.toml" NAME -- --exact --nocapture --test-threads=1` | All **three exited 101 at their assertions of the old wrong behavior**, not compilation. Ordinary environment replacement now fails without persisting; preview returns Failed; unrelated latest evidence returns Failed. Original witnesses were not rewritten. |
| `CARGO_TARGET_DIR=/var/home/olav/Projects/Regressee/target cargo test --locked --offline --manifest-path "$R/positive-probe/Cargo.toml" -- --nocapture --test-threads=1 --skip source_protocol::` | **7 passed**. Separate correct-expectation probes include successful explicit replacement/deduplication, valid exact paused receipt, preview failure, hardlinked lock/owner mismatch, partial-frame timeout, epoch/restart invalidation, 64-job cap, cancellation and real receipt-write fault/restart. Six source-module unit tests are filtered here because they run in their owning app crate below. Only this separate positive copy updates endpoint version to negotiated `VERSION` and uses protocol-2 `Captured`; originals stay unchanged. |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1 --skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` | **230 passed, 0 failed, 29 ignored, 1 scale test filtered**; nested environment subprocess not counted twice. App library 56/1 ignored; engine integration 18/1 ignored; CLI 4 passed. Includes new positive state/identity/producer/atomic-save matrices and all existing ownership, deduplication, frozen-input, event-gap, overload and real child-process drain/crash checks. 26.49 s. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**; extended production IPC facade, explicit combined replacement and detach. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --bin regressee-desktop engine_refresh_tests:: -- --test-threads=1` | **2 passed**, 231 filtered; conflicting drafts and exact capture completion vs separate workspace refresh. GPUI simulations, not native windows. |
| `cargo test --locked --offline -p regressee-gui --all-features --lib -- --test-threads=1` | **97 passed, 1 ignored**. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Exit 0. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Exit 0. |
| `cargo test --locked --offline -p regressee-gui --features ui --lib --bin regressee-desktop --no-run` | Exit 0; UI-only feature compilation, not warning-free qualification. |
| `cargo tree --locked --offline -p regressee-cli --edges normal,build --prefix none --format '{p}'` | Exit 0; CLI closure remains GUI/GPUI-free. |
| `python3 -B -m unittest discover -s scripts -p 'test_*.py'` | **59 passed**. |
| `python3 -B "$R/platform_checks.py"` | Above dependency/script checks plus both rebuilt production binaries' `--build-info` and isolated no-display `engine start/status/stop`: **eight successful invocations**. Actual wire hello 1/3 refused, 2 accepted for each binary; same owner, zero admitted jobs, empty HOME, no credential/runtime creation, ownership released after stop. No native GUI or browser launched. |
| `git diff --check` | Exit 0 after report update. |

Exact original `NAME` values:

```text
replacement_settings_bypass_exact_confirmation_through_save_environment
preview_capture_failure_is_relabelled_succeeded_by_actual_receipt_classifier
resumed_older_run_result_is_not_the_project_latest_used_by_receipt_classifier
```

**Final disposition:** the initial repair handback is satisfied; no actionable
Phase 2 finding remains in this bounded review. The five inherited GUI failures,
format/strict-lint debt, unchanged scale-test exclusion and native/clean-host
limits at the end of this report remain open. Windows transport/platform hashes
match the independently type-checked source from the first review; this re-review
does not add a native Windows/full-app link receipt. Active managed-login and live
capture/resume lifetime still need native/browser qualification, not inference
from synthetic jobs, saved pixels or classifier tests. No Phase 3/4 work, commit,
push, auth injection, actual user-data open, outside capture or sandbox change.

## Initial findings — historical, all independently closed above

### 1. HIGH — `SaveEnvironment` bypasses replacement confirmation

**Locations:** `crates/app/src/protocol.rs:89–103`,
`crates/app/src/engine/executor.rs:46–57`; underlying alternate write path:
`crates/storage/src/lib.rs:502–520`.

`SaveReplacement` requires an exact prepared confirmation, but `SaveEnvironment`
does not. The latter accepts an entire project and calls
`save_project_inner(..., replacement = true, ...)`. Consequently it permits the
same active-baseline capture-setting replacement that ordinary `Save` rejects,
without any confirmation token. The expected project revision still protects
against stale writes; it does not enforce the missing confirmation.

**Independent IPC reproduction:** seed an isolated project with an active
baseline using the existing saved-pixel fixture; change its profile width;
submit the same project/revision through the three operations, without a token:

```text
SaveReplacement: admission refused (confirmation required)
Save:            terminal Failed (baseline capture settings are fixed)
SaveEnvironment: terminal Succeeded; changed profile persisted
```

Probe: `replacement_settings_bypass_exact_confirmation_through_save_environment`.
It uses the production engine/client over Unix IPC, not a service-only mock.

**Required repair:** enforce the same replacement-intent policy on every path
that can perform that write. For example, require exact confirmation for
`SaveEnvironment` when it exercises replacement semantics, or constrain it so
it cannot substitute for `SaveReplacement`. Preserve legitimate environment
saves, revision checks, and the GUI facade. Add an alternate-operation bypass
regression, not only another test of `SaveReplacement` itself.

### 2. HIGH — failed representative previews get successful terminal receipts

**Locations:** `crates/app/src/protocol.rs:299–317`,
`crates/app/src/jobs.rs:641–659`; preview capture behavior:
`crates/app/src/crawling.rs:530–561,812–847`.

`PreviewImages` is a real capture-producing operation. Its service can return
saved evidence with `CompletedWithErrors` or `Cancelled` rather than throwing an
error. Its handler returns `Outcome::Evidence`, just like the other captures.
However, `Job::terminal_state` inspects the run state only for `Capture` and
`CaptureInventory`. `PreviewImages` falls through to `Succeeded`.

**Independent source-linked reproduction:** the actual `protocol.rs` classifier
was compiled into the external probe and passed the same synthetic saved
`CompletedWithErrors` evidence for both operation types:

```text
Capture=Failed, PreviewImages=Succeeded
```

Probe: `preview_capture_failure_is_relabelled_succeeded_by_actual_receipt_classifier`.
This is a classifier/saved-evidence proof, **not** an executed native/browser
capture. The production service mapping above establishes the reachable path.

**Required repair:** classify every capture-producing operation from its actual
capture outcome, including representative preview failure/cancellation. Keep
visual differences distinct from operational failure. Cover these operations
in the terminal-state regression matrix.

### 3. HIGH — resumed jobs can report another run's evidence and outcome

**Locations:** `crates/app/src/jobs.rs:733–765`,
`crates/app/src/protocol.rs:309–315`; supporting behavior:
`crates/app/src/crawling.rs:584–650` and
`crates/storage/src/capture.rs:813–828`.

`capture_inventory_with_status` returns the exact `RunRecord` it executed, but
the handler discards it and returns `store.project_evidence(project)`. That read
selects the newest inserted run, not necessarily the resumed run. Resume accepts
any eligible paused/interrupted run of the project; it does not require the
newest run. The durable job classifier then trusts this unrelated `latest` run.
Thus an older resumed job that pauses/fails/cancels can receive a successful
receipt from a newer completed run, or the converse. Its evidence identity is
also wrong even when the states happen to match.

**Independent saved-storage/source-linked reproduction:** create paused run A,
then completed run B in an isolated real Store. `load_run(A)` reports `Paused`,
while the handler's `project_evidence` read reports B. Feed that returned evidence
to the actual classifier for `CaptureInventory { resume: Some(A), ... }`:

```text
requested resumed run A: Paused
returned evidence.latest: B
terminal receipt: Succeeded
```

Probe: `resumed_older_run_result_is_not_the_project_latest_used_by_receipt_classifier`.
No browser resume was executed: this proof exercises real saved-state selection
and the production classifier source, with the service's returned-record
discard verified directly in the handler.

**Required repair:** bind the job result and terminal state to the exact returned
run ID/record. Keep any separate GUI “refresh newest evidence” operation separate
from the job's authoritative receipt. Test resuming a non-newest eligible run
with differing terminal outcomes and verify both evidence identity and status.

## Initial independent evidence and commands

All scripts, logs, the external probe, source snapshots/hashes and isolated
stores are retained under:

```text
/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase2-review
```

Environment for shared-target commands:

```sh
export PATH=/var/home/olav/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH
export E=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase2-review
export TMPDIR="$E"
export LIBRARY_PATH=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase1/link-libs
```

Compilation was serial and verifier-owned. `checks.py`/`checks.json` retain exact
regression arguments and exits; `platform_checks.py`/`platform-results.json`
retain platform/dependency and production-entrypoint commands. `source-receipt.json`
records 39 source/Cargo hashes and the base commit; `source/` and
`source-tracked.diff` retain the reviewed source. No implementor test verdict was
substituted for running these checks.

| Exact command | Independent result |
| --- | --- |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Exit 0. |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1 --skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` | **220 passed, 0 failed, 29 ignored, 1 scale test filtered**, excluding the nested environment-isolation subprocess from the sum. App library 50/1 ignored; engine integration 14/1 ignored; CLI 3 unit + 1 lifecycle; core 56; runtime 34/1 ignored + 7 distribution; storage 40/2 ignored. Includes real engine child-process Ctrl-C drain/crash tests and six-process autostart races. 24.20 s total. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**. Production IPC facade, verified immutable artifact decoding/cache reuse, coalescing, saved-data/appearance refresh and prompt detach with pending work. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --bin regressee-desktop engine_refresh_tests:: -- --test-threads=1` | **1 passed**, 231 filtered. GPUI simulation: screen/stage preserved; conflicting drafts retain their old revision. Not native-window execution. |
| `cargo test --locked --offline -p regressee-gui --all-features --lib -- --test-threads=1` | **97 passed, 1 ignored**. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Exit 0, including production CLI/desktop and developer/prototype binaries. |
| `cargo test --locked --offline -p regressee-gui --features ui --lib --bin regressee-desktop --no-run` | Exit 0. UI-only feature compilation; includes an unused `AppPaths` import warning and the dependency future-compatibility notice. No warning-free claim. |
| `CARGO_TARGET_DIR=/var/home/olav/Projects/Regressee/target cargo test --locked --offline --manifest-path "$E/probe/Cargo.toml" -- --nocapture --test-threads=1` | **8 passed**: seven independent probes plus the source-linked existing classifier unit test. **Three are defect witnesses that assert the observed wrong behavior**, not safety acceptance tests. Final log `probe.log`. |
| `cargo tree --locked --offline -p regressee-cli --edges normal,build --prefix none --format '{p}'` | Exit 0; no GPUI or GUI crate in CLI normal/build closure. |
| In `$E/windows-scaffold`: `CARGO_TARGET_DIR="$E/windows-target" CARGO_TARGET_X86_64_PC_WINDOWS_MSVC_RUSTFLAGS="--sysroot /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase2/windows-api/sysroot" cargo check --locked --offline --all-targets --target x86_64-pc-windows-msvc` | Exit 0. Independent path-only compilation of actual `platform.rs`, Win32 process ownership and `engine/ipc.rs`/`windows.rs`; no copied transport stub. Retained official std archive SHA-256 independently rechecked against its manifest: `42727764a01a54db7423e43db08a9c818405679af6a9a677068e8cd4b8361242`. Not full-app linking or Windows execution. |
| `python3 -B -m unittest discover -s scripts -p 'test_*.py'` | **59 passed**. |
| `git diff --check` | Exit 0. |

The source-linked classifier probes use `#[path]` to compile the actual
`crates/app/src/protocol.rs` outside its owning crate. One **unused** readiness-
message shim supplies a crate-private dependency; it is not part of any tested
classifier path. `fixture-source.json` records the original saved-pixel fixture
hash; its only scratch rewrite binds `CARGO_MANIFEST_DIR` to the original corpus
location. No production or repository test was modified. Early external-harness
compile/fixture mistakes were repaired in scratch; the table reports final runs.

Both production `target/debug/regressee` and `target/debug/regressee-desktop`
were independently run with `--build-info` and `engine start/status/stop`, each
under a distinct isolated HOME/data/cache tree, no DISPLAY/Wayland and an invalid
session bus. All eight invocations exited 0. Metadata had no test/developer
feature leakage, HOME stayed empty, and no private-session or managed-runtime
directory was created. This exercises **engine-only entrypoints**, not a GUI or
browser. No fresh clean-target CLI compilation was repeated; Phase 1's separate
accepted clean-build evidence is not relabelled a new Phase 2 clean-host test.

## Initial boundary audit and hostile checks that held

- Ownership precedes Store open/migration, recovery and confirmed GC. Canonical
  symlink aliases share the owner/endpoint; contender tests reject duplicate
  owners before recovery. Legacy actors and flow tooling acquire the same lock;
  foundation requires a fresh isolated root. Production GUI `Jobs::start` uses
  IPC only; local actor constructors/cache are test-gated, with no silent fallback.
- Reviewed Linux abstract sockets and bidirectional kernel UID checks; non-Linux
  Unix private socket cleanup under the ownership lock; actual Windows protected
  current-user DACL, first-instance/remote-client rejection and detached startup
  flags. Endpoint UUID/version mismatch fails closed. Independent probes also
  rejected a hardlinked lock and mismatched private owner UUID without opening a
  second Store. The pre-engine-binary upgrade limitation is documented honestly.
- Typed operation/reply mapping covers the full existing Work inventory,
  reanalysis and both viewer modes. No whole operation namespace is silently
  omitted; findings 1–3 concern meaningful policy/result differences within that
  mapping. Phase 3 product parser/report omissions are not findings here.
- Broker status/control/events are separate from blocking product work. Own
  hostile probes saturated the **64-job** admission cap, confirmed prompt busy
  rejection/cancellation, and expired a partial frame at the five-second deadline
  while another job completed. Existing integration checks independently rerun
  the 16-connection cap, malformed/oversized frames, event overflow/resync and
  terminal retrieval despite stalled readers. The 64 MiB frame, 8 MiB input,
  4096 retained-job/512 MiB journal and bounded event/cache limits are present;
  they are not an exhaustive memory-pressure/large-journal benchmark.
- Exact action/path/preview/revision/owner/epoch token validation and original-ID
  deduplication work for the gated operations tested. Restart/epoch invalidation,
  frozen-project revision conflicts and non-replay of uncertain work passed.
  At the initial review this did not cure finding 1; the repaired alternate path
  is now independently accepted above.
- Own terminal-write fault injection changed only an owned synthetic receipt's
  permissions while work ran. Completion became explicit `Interrupted`, the
  engine drained, and restart returned `Interrupted` for the original submit
  rather than replaying it. Acknowledged read reclamation worked. Mutation and
  confirmation receipts are deliberately retained rather than silently evicted.
- Reviewed snapshot-before-events refresh, gap/owner resync, project-revision
  refresh and draft guards. IPC facade and shell simulation tests passed.
  Artifact DTOs originate from verified Store artifacts; the GUI hashes and
  bounds tile decoding without a live database handle. Display caches are not
  mutation/approval authority.
- Closing clients detaches; production facade teardown does not stop the engine.
  Explicit Stop rejects active work/login. Foreground signal handling drains
  without implicit cancellation; already-admitted work can finish, new mutations
  are refused and close-login remains admitted. The active-work executable test
  uses **synthetic TestWait**, not Chromium. The open-login guard and CloseLogin
  drain path were source-reviewed, **not dynamically qualified with a real managed
  login or active browser capture**.
- Epoch hooks are correctly documented as hooks, not finished session/lease
  authorization. No Phase 4 takeover/follow/control security PASS is inferred.

## Remaining qualification

1. The original Phase 2 implementor's repair handback is now independently
   accepted. Findings 1–3 are closed; Phase 2 PASS is bounded as stated above.
2. The five independently proven inherited GUI failures remain inherited/open.
   I did not rerun the costly full desktop suite, scale qualification, live
   browser tests or native renderer tests, nor demand unrelated UI fixes here.
   Workspace formatting/strict-clippy debt is not newly certified clean.
3. Native Windows ACL/peer exclusion, named-pipe concurrency/restart, console/job
   lifetime, installer and full-app build/link tests remain open. The source
   type-check above is not a native Windows receipt. Cross-user OS behavior and
   non-Linux Unix IPC were not executed on this Linux host.
4. Actual GUI presentation/input/accessibility, managed-login drain/capture
   lifetime, clean headless browser setup/fonts/sandbox, packaged-binary and
   clean-host qualification remain open. No outside website was captured and no
   sandbox weakening was used. Existing macOS capture limits remain unchanged.
5. Full CLI product grammar/JSON/report/crop UX (Phase 3), follow/leases/handoff UI
   (Phase 4), skill/packages/release qualification (Phase 5) remain future work.
