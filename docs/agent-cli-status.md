# Agent CLI implementation status

Phase 1 independently **PASS** at `9daf121`; see
[the independent report](agent-cli-phase1-review.md). Phase 2 independently
**PASS**, protocol 2, after its three HIGH findings were independently closed;
see [the verifier's final re-review](agent-cli-phase2-review.md). The earlier
NEEDS FIX/repair handoffs below are historical. Phase 3 independently **PASS**,
protocol 4, with all six findings independently closed; see
[the verifier's final report](agent-cli-phase3-review.md). Phase 4 independently
**PASS**, protocol 6, within reviewed source/Linux IPC/owned synthetic fixtures
and GPUI offscreen/simulated-input scope; all five original findings and the
retained-feedback follow-up are closed. See the unchanged
[final independent report](agent-cli-phase4-review.md). Native OS-window/input,
accessibility and platform/release qualification remain unqualified. Earlier
NEEDS FIX/repair entries below are historical, not the current verdict.
Phase 5 independently **PASS** within source/package/public-contract repair
scope, and the separate fresh-agent skill exercise independently **PASS** using
the repaired packaged CLI and actual sandbox-enabled Chromium. Both original
verifier findings and the consumer's repair findings were fixed by the original
implementor and independently rechecked. See the
[verifier's final disposition](agent-cli-phase5-review.md) and
[consumer's final exercise](agent-cli-skill-exercise.md). Earlier pending/NEEDS FIX
and repair-handoff entries below are chronological evidence, not current verdicts.
Implementation across all five phases is delivered; **full plan qualification
and production-release acceptance are not complete**.
The current wire contract is protocol 6 (both
entrypoints/engine must be rebuilt/restarted). This record does not supersede the confirmed
requirements in [the plan](agent-cli-plan.md). `ACTIVE_WORK.md` is historical.

## Orchestrator closeout — 2026-10-17

Each phase had a separate implementor and verifier. All material review findings
were returned to that phase's original implementor, then independently reverified.

| Phase | Implementation / independent disposition | Review |
| --- | --- | --- |
| 1: separate GUI and CLI crates | Delivered; PASS, extraction/dependency/build/fixture scope | [Phase 1](agent-cli-phase1-review.md) |
| 2: shared local engine | Delivered; PASS after three HIGH repairs, source/Linux IPC/fixture scope | [Phase 2](agent-cli-phase2-review.md) |
| 3: full CLI workflows and evidence | Delivered; PASS after six repairs, public CLI/synthetic-fixture scope | [Phase 3](agent-cli-phase3-review.md) |
| 4: exclusive following and human handoff | Delivered; PASS after five findings and the retained-feedback follow-up, source/IPC/GPUI offscreen/simulated-input scope | [Phase 4](agent-cli-phase4-review.md) |
| 5: portable skill and product packages | Delivered; PASS after packaging/skill/consumer repairs, bounded source/package/public-contract scope | [Phase 5](agent-cli-phase5-review.md) |
| Separate skill consumer | PASS: fresh public-CLI-only agent, actual managed Chromium, real long-page inspection, selective approval/history and incomplete-check recovery | [Consumer](agent-cli-skill-exercise.md) |

Latest repaired delivery artifacts and source/binary bindings are under
`/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/repair/`.
The separate consumer's final evidence is under
`/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/fresh-agent-consumer-v2/`.
Its Details captures are **1440×6617**, with nine independently viewed top/middle/
bottom before/after/difference crops. Baseline/check/incomplete exits remain
**0/1/3**; incomplete `--save` and same-job recovery preserve the exact result.
The consumer verified session termination, engine shutdown and fixture teardown.
No release/workflow publication or upload was performed.

The final repair verification includes **71 Python tests**, **22 CLI tests** and
**27 independently checked packaged invocations**, plus the separate actual-browser
consumer. These overlap earlier suites and are not summed as unique coverage.
Source/package checks and workspace builds are recorded with exact commands below.

### Remaining qualification and nonblocking advisories

- Clean Linux server/default OS dependencies/fonts/security-policy qualification
  is open; isolated development-host execution is not a clean-server test.
- Native GUI window/input/IME/accessibility and Windows IPC/ACL/Job/browser
  qualification remain open. GPUI offscreen rendering is not OS-input acceptance.
- The development-host desktop ELF requires **GLIBC 2.43** and does not meet the
  Ubuntu 24.04 floor. The production Ubuntu workflow enforces **GLIBC <= 2.39**,
  but was not dispatched. Do not distribute these local desktop bytes as an
  Ubuntu-qualified release. The local CLI's inspected floor is GLIBC 2.39.
- Rendered HTML report layout remains unqualified because the desktop browser
  tool is disconnected. Static structure, escaping, labels, privacy text and
  actual PNG evidence were independently checked.
- Signing/installers, complete native/transitive legal/security review and public
  release remain open; macOS capture support was not added.
- The five inherited GUI failures and existing global formatting/strict-lint debt
  remain recorded and unwaived. The consumer also noted nonblocking `--save`
  applicability/help, debug-style Markdown details and absent-engine error copy
  advisories; these are not open material repair findings in the accepted scope.
- Earlier accumulated work and Phase 1 were committed/pushed to `main` at
  `9daf121` on the user's instruction. The user subsequently explicitly authorized
  committing and pushing all implementation, skill, packaging and review changes
  to `main`. Historical dirty-source receipts below retain their original identities.

## Phase 1 scope and inventory (historical extraction snapshot)

- Extract only the native presentation; no shared engine, IPC, public product
  commands, control leases, following or new website/data operations yet.
- `regressee-app`: product/crawling/authentication, analysis, review, diagnostics,
  recovery, preferences, launch-path policy, image validation/decoding and the
  bounded application job owner. No GPUI dependency or UI Cargo features.
- `regressee-gui`: `regressee-desktop`, the entire current shell and sibling UX
  modules, widgets, screenshot/cache work, task/activity presentation, UI tests,
  inspection/renderer harnesses and embedded timezone data. Root brand assets
  remain in their existing location. Current untracked UI work was moved too.
- `regressee-cli`: `regressee` skeleton, help/version/build-info only. Unknown
  commands fail with exit 2 and stderr; it does not discover/open a data directory,
  perform recovery, start browsers or use a display.
- Existing developer `regressee-flow`/`regressee-foundation` remain in `app`;
  the historical `regressee-spike`/`regressee-ui` prototypes remain in `spike`.
  The latter now consumes controls from `gui`, never `app`.
- Backend fixtures stay in `app/tests/support`; native tests reference those
  same files rather than forking fixtures. Native inspection fixture moved to
  `gui/tests`. Product tile-integrity checks now run without GPUI.

### Current action boundary (inventory for later CLI coverage)

The existing GUI dispatches application jobs for project list/load/save and
replacement, environment/login/probe, network and browser setup/status,
discovery and existing inventory, capture/check/resume/pause/cancel, evidence and
run inspection, reanalysis/masks/review, baseline activation/approval/acceptance/
restore, history/storage/deletion/repair, baseline ZIP import/export and diagnostics.
Native-only actions include navigation/focus, drafts and confirmations, filters/
selection, task presentation, pan/zoom/compare/overlay/split, screenshot tile
prefetch, appearance, recovery destinations, native file prompts, clipboard/OS
links and exit guards. Failure causes and shared human-readable explanations
remain application services; recovery navigation now belongs to the GUI.
This is an extraction inventory, **not** a claim of implemented CLI coverage.

### Actor extraction constraint

The same blocking application actor still opens/recovers the store and runs
all commands, with identical capacities and mutation-before-viewer scheduling.
An in-process generic adapter owns the GUI's latest-intent mailbox, generation
guard, silent adjacent prefetch and cache invalidation. No native image or
viewer command types remain in `app`. This adapter is not the Phase 2 protocol;
engine ownership/recovery coordination remains a later gate. Preferences retain
their independent small-write owner and are started by the desktop adapter only.

### Build/release/documentation migration

Active build/test/CI/preview tooling has been migrated to package `regressee-gui`
and binary `regressee-desktop`; backend tests use `test-support` rather than the
removed `app/ui-tests`. Preview archive names identify the desktop binary.
Phase 5 still owns CLI-only archives and desktop packages containing both
executables. No workflow was dispatched and nothing was published/signed.
Historical research, frozen source/hash receipts and `ACTIVE_WORK.md` retain
their original identities; those are not current launch instructions.
The historical `ui-review-phase1-receipt.py` and `verify-phase4/5-receipt.py`
operate on their original frozen checkouts/receipts, not current source. Current
renderer/evidence collection scripts use the extracted GUI paths.

## Verification ledger (implementor, not independent review)

All runtime-test scratch/logs are under the caller-approved private temporary
root `agent-cli-phase1`. Heavy compilation into the shared target is serial;
already-built bulk tests can continue while the GUI compiles, and the clean CLI
proof uses its own completely separate target directory.
The host's `cargo` shim initially failed because mise had no configured Rust;
commands below use the installed Rust 1.98.1 toolchain on `PATH`.

Local command environment (not product configuration):

```sh
export PATH=/var/home/olav/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH
export TMPDIR=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase1
export LIBRARY_PATH="$TMPDIR/link-libs"
```

Raw logs, metadata, dependency proofs, test summaries and baseline comparison
receipts are retained in that scratch root. No actual user store was opened.

| Command | Result |
| --- | --- |
| `cargo check --locked --offline -p regressee-app -p regressee-cli -p regressee-gui --all-targets --all-features` | Passed; superseded by the full-workspace check below. |
| `cargo build --locked --offline -p regressee-gui --features ui --bin regressee-desktop` | Passed with local `LIBRARY_PATH` described below. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Passed on final source, including both new public executables and preserved developer/prototype binaries. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Passed after the recovery-navigation extraction (including the historical spike and native fixture target). |
| `cargo test --locked --offline -p regressee-gui --features ui --lib --bin regressee-desktop --no-run` | Passed after the feature-gate repair; compilation only. |
| `cargo test --locked --offline -p regressee-app --features test-support --lib -- --test-threads=1` | Final source: 48 passed, 1 intentionally ignored. |
| `cargo test --locked --offline -p regressee-gui --all-features --bin regressee-desktop recovery_ux_tests:: -- --test-threads=1` | Final recovery-navigation source: 3 passed, 1 renderer capture intentionally ignored, 291 filtered out. |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1` | **204 passed, 0 failed, 28 ignored** across top-level test targets (nested subprocess test not counted twice). Includes 9 analysis/review, 2 partial-baseline, product-flow fixture, and the persisted 100/1000-page qualification (846.05 s). First invocation hit a 120-second harness timeout; the complete rerun used no harness timeout. |
| `cargo test --locked --offline -p regressee-gui --all-features --lib --bin regressee-desktop -- --test-threads=1` | Library: **97 passed, 1 ignored**. Desktop: **228 passed, 5 inherited failures, 62 ignored** (477.13 s). All five failures reproduce in the pre-extraction binary, below. |
| `python3 -B -m unittest discover -s scripts -p 'test_*.py'` | 59 passed (packaging, environment boundary, source release, runtime pack and CI launcher). |
| `PKG_CONFIG=/bin/false LIBRARY_PATH= CARGO_TARGET_DIR=<fresh-private-target> cargo build --locked --offline -p regressee-cli --bin regressee`, with display/Wayland/session-bus variables unset | Passed from a completely fresh target; no GUI SDK/linker paths. |
| Same isolated target/environment: `cargo test --locked --offline -p regressee-cli` | 3 passed. |
| Fresh CLI executable: `--help`, `--version`, `--build-info`, `capture`, `engine serve` under an isolated home/XDG tree, no display and an invalid session bus | Expected exits 0/0/0/2/2; machine stdout clean, no data/home files created. ELF dependencies contain no X11/Wayland/Vulkan/OpenGL/C++ libraries. |
| Production `regressee-desktop --build-info` with isolated home/XDG, no display and invalid session bus | Passed without files or graphics startup; UI true, developer/test/inspection/capture features false. |
| `cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-unknown-linux-gnu` + traversal of CLI normal/build dependencies; `cargo tree --locked --offline -p regressee-cli --edges normal,build --prefix none --format '{p}'` | 244 metadata-closure packages; neither GPUI nor `regressee-gui` reachable. Unfiltered all-platform metadata cannot run offline because uncached `arbitrary 1.5.0` is requested; no download attempted. |
| `rustfmt --edition 2024 --check --config skip_children=true` on new/rewritten actor, image, task, CLI and GUI adapter modules | Passed; native recovery modules and extraction-edited shell/import hunks were formatted separately with `skip_children=true`. |
| `git diff --check`; `bash -n scripts/build-ui.sh` | Passed. |
| `cargo fmt --all -- --check` | Extraction hunks fixed. Remaining pre-existing/current-user formatting only: `ui/setup_progress.rs`, `ui/timezone_select.rs`, `options_ux.rs`, and the renderer harness's module order. Left untouched rather than broadly formatting user work. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features` | Exit 0. Existing moved-widget/test warnings remain (scrollbar type complexity, single-element loop, range patterns, redundant closures, collapsible conditions and bool/filter-map). No new adapter/CLI warning. `-D warnings` is not claimed clean. |

The first GUI link attempt failed on this host's missing unversioned linker
names (`stdc++`, `xcb`, `xkbcommon`, `xkbcommon-x11`). Private scratch symlinks to
the existing `/usr/lib64` shared libraries resolved it; no system installation,
runtime library replacement or sandbox setting was changed. This is local
build evidence, not Ubuntu ABI, Windows/macOS, desktop usability or clean-host
qualification.

The UI-only test build exposed an already-present/current typography GPUI test
in `ui/controls.rs` without its `ui-tests` gate. The test and its fixture now use
the same gate as other native tests (no test removed); the toolbar test import
is gated too. This is a feature-matrix repair, not a widget behavior change.

### Inherited GUI failures: differential baseline proof

Each failed test was rerun individually on the extracted source with:

```sh
cargo test --locked --offline -p regressee-gui --all-features \
  --bin regressee-desktop TEST_NAME -- --exact --test-threads=1
```

Each was then run through the existing **pre-extraction** libtest executable:

```sh
target/debug/deps/regressee-b2238189f1b8bd46 TEST_NAME --exact --test-threads=1
```

All ten individual invocations failed with the **same assertion and values** in
each pair. Exact `TEST_NAME` values:

1. `changed_results_tests::changed_results_first_analysis_and_recorded_checks_default_without_resetting_refresh` — recorded-history filter remains 0 instead of expected 1.
2. `disclosure_tests::all_inline_disclosures_attach_content_restore_focus_and_do_not_mutate` — existing search disclosure is disabled.
3. `loading_feedback_tests::loading_feedback_has_one_owner_and_keeps_its_icon_and_copy_visible` — loading copy exceeds its actual owner bounds.
4. `tests::project_selection_actions_and_long_names_fit_supported_window_sizes` — project-change action exceeds the expected right inset.
5. `workflow::tests::review_first_viewport_contains_complete_rows_and_bounded_images` — minimum-size queue does not fully contain the first row.

Baseline binary SHA-256:
`ae053aa1bc6d434d05fde4df8fd7e2c9b4234621436e8766305c586a4f1d83a9`.
Its libtest symbols were checked without launching the application. Its embedded
source blobs supplied **79 exact SHA-256 matches** against the inventory captured
before moving any files, including the entire original `main.rs`, affected
tests, workflow/history implementation and controls. Thus this is not merely an
older unrelated binary with similar failures. No source was reverted/restored;
only read-only artifact inspection and isolated fixture tests were used.
See `baseline-binary-source-proof.json`, `gui-failure-reruns.json`,
`gui-baseline-reruns.json`, and the five pairs of failure logs in scratch.

Other exact backend counts: application library 48 passed/1 ignored; core 56
passed; runtime library 34 passed/1 ignored and distribution 7 passed; storage 40
passed/2 ignored; CLI 3 passed. Remaining ignored targets require explicit live
browser, official download, native ownership or retained evidence inputs. They
were not enabled and no native desktop/clean-host qualification is inferred.

## Remaining Phase 1 gates

- Independent verifier confirmed the extraction, package/feature boundaries,
  preservation of all 188 inventoried paths, and all 79 baseline source matches.
  Independent backend subset: 203 passed/28 ignored/1 scale test deliberately
  filtered; GUI library 97 passed/1 ignored; focused native simulations and both
  sets of five inherited failures reproduced. Independent fresh CLI proof passed.
- The inherited five UI failures and existing workspace formatting/strict-clippy
  warnings mean the entire tree is **not claimed fully green**. Resolve/waive
  those separately without changing UI behavior under this extraction task.
- Native/clean-server/browser/sandbox/Windows/macOS/release gates remain open.
  The review did not rerun the 846-second persisted scale qualification or the
  entire desktop suite. Nothing in the Phase 1 PASS waives inherited failures.

## Phase 2 — initial handoff (superseded by independent findings and repair below)

Scope: a single locked application engine, private local IPC, explicit versioned
commands/results/events, durable detached jobs, exact confirmations/deduplication,
GUI client facade and engine lifecycle CLI. Full public command namespaces,
reports/crops (Phase 3), and GUI following/control banner (Phase 4) are deferred.
No additional commit/push is authorized. The Phase 1 commit/push was completed by
the orchestrator; the independent report is preserved untouched.

### Implemented boundary

- `crates/app/src/engine`: canonical-root, protocol-independent exclusive lock
  before migrations/recovery/confirmed GC; live broker separated from the single
  blocking Store/Product owner; private bounded Unix IPC and genuine Windows
  named pipes. Linux abstract sockets verify peer UID on both ends; Windows
  pipes use protected current-user-only DACLs, first-instance and remote-client
  rejection. No TCP fallback.
- `crates/app/src/protocol.rs`: explicit versioned command/result DTOs, durable
  job identity/status/frozen project revision, capture snapshots/progress,
  sequenced events/snapshot resync, exact action+preview+revision+epoch consent,
  original-input request deduplication and epoch-revocation hooks. All existing
  application operations have mappings; this is not full public CLI parsing.
- `crates/gui/src/jobs.rs`: production IPC facade, client-owned immutable tile
  decoding/cache and latest-intent mailbox; no production Store open, recovery,
  native worker owner or silent local fallback. Old native actor/cache policy
  lives behind explicit test support in `local_jobs.rs`/`viewer_cache.rs`.
  The engine maintains its own bounded revision-keyed display metadata cache.
- Shell refresh preserves screen/selection and dirty drafts. A conflicting draft
  keeps its original revision, not an updated overwrite precondition. Preferences
  use engine RPCs, including cross-client refresh. Background read waits can
  detach without waiting for or cancelling a blocking engine job.
- `regressee engine serve|start|status|stop`, before graphics initialization in
  either binary; absolute known-executable launch, Unix session separation,
  bounded version/owner handshake and startup races/reconnect handling. Lifecycle
  directory flags are order-independent; explicit data/cache paths need no HOME.
- Historical flow tooling now takes the same lock before Store/recovery;
  foundation tooling remains fresh-isolated-root-only and locks as well.

The detailed operating contract and limits are in [local-engine.md](local-engine.md).
There is deliberately no idle timeout: explicit stop refuses active jobs/login,
foreground Ctrl-C drains without cancelling, client/GUI disconnect only detaches.
Mutation/confirmation receipts are retained, bounded at 4096 jobs/512 MiB;
admission fails closed at quota rather than dropping retry authority. A crash
between product commit and receipt is explicitly uncertain/Interrupted, not
silently retried or claimed exactly-once. Old pre-engine executables must be
closed before upgrade; they cannot retroactively be made to honor the new lock.

### Phase 2 verification environment and in-progress receipts

Final runtime fixtures/logs use
`/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase2`; the early development
checks also left `check-engine-early.log`, `check-phase2-ui.log`,
`check-phase2-current.log`, and `test-engine-first.log` in the Phase 1 scratch
root. None is a native/release qualification receipt. Rust commands use:

```sh
export PATH=/var/home/olav/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH
export TMPDIR=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase2
export LIBRARY_PATH=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase1/link-libs
```

The shared-target build/test sequence is serial and owned by this implementor;
the CLI clean proof and Windows source check use separate private targets.
There has been no actual user-store access, browser/sandbox modification, native
window/renderer launch, network publication, commit or push. Both production
binaries' **engine-only** entrypoints were exercised without a display below.

- `engine-second.log`: 13 passed/1 subprocess-only helper ignored. Includes
  multi-client/alias/duplicate-owner checks, live cancellation, detached jobs,
  queued revision/epoch guards, confirmation wrong action/path/stale/replay,
  durable mutation retry/restart, lost-receipt uncertainty, bounded bad frames,
  ring-overflow resync, actual saved approval/ZIP services, and a real child
  process for Ctrl-C drain/crash recovery. Subsequent focused tests extend this
  with connection-overload, legacy-actor/live-row and native refresh checks.
- `cli-clean-proof/results.json`: fresh `PKG_CONFIG=/bin/false LIBRARY_PATH=`
  CLI build/test/tree proof with no display/session bus and a separate target;
  three CLI unit tests plus real six-contender autostart test passed. Help,
  version, metadata, rejected product command, missing-store status, then explicit
  start/status/stop gave exits **0/0/0/2/2/0/0/0**. Isolated HOME remained empty;
  no credential/session or managed runtime directory was created. ELF linkage
  and dependency tree exclude GUI/GPUI/X11/Wayland/Vulkan/OpenGL/C++ libraries.
- `windows-api/result.json`: the actual Win32 platform/Job and Tokio named-pipe
  source type-check passed using a checksum-verified official Rust Windows std
  target in private scratch (23,032,992 bytes; no rustup/system installation).
  `windows-api/final-source-check.log` also passed after the capacity repair.
  This is a path-only harness for transport/platform APIs, **not** full app
  linking or native Windows ACL/IPC/crash qualification.
- Two initial development checks caught missing `serde/rc` unification in the
  standalone app viewer cache and private shell-helper visibility; both were
  repaired rather than adding GUI/serde feature leakage or weakening test gates.

### Completed regression and final-source checks

The first complete bulk backend invocation below passed before the final
incremental hardening (separate retained capture progress, precise admission
errors, detach-aware background reads, and extra fault/legacy-owner tests).
The final backend rerun skipped only the already-passed, unchanged storage scale
case; final-source focused tests/builds then covered the last admission-failure
unit test and lint-only cleanup. Counts below are per command, **not summed with
duplicate tests or nested subprocess summaries**. Raw commands/exits/timings are
in `regression-results.json`, `final-results.json`, and `handoff-results.json`;
summaries are in `bulk-summary.json`/`handoff-summary.json`.

| Exact command (environment above) | Result / log |
| --- | --- |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1` | **218 passed, 0 failed, 29 ignored**, 642.87 s total. Persisted 100/1000-page selective approval/privacy/retention case passed in **616.65 s**. `backend-full.log`. |
| Same command with `--skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` appended after `--test-threads=1` | **219 passed, 0 failed, 29 ignored, 1 filtered**, 28.51 s. Includes added terminal-state and live-row/legacy-owner tests. `final-backend.log`. |
| `cargo test --locked --offline -p regressee-app --features test-support --lib --test engine -- --test-threads=1` | Final source: library **50 passed/1 ignored**, engine integration **14 passed/1 subprocess-only helper ignored**. Includes failed-worker admission receipt, overload bounds, actual running-row attach protection, real Ctrl-C/crash subprocess and exact consent/ZIP/approval services. `handoff-app.log`. |
| `cargo test --locked --offline -p regressee-cli -- --test-threads=1` | **4 passed**: 3 unit + real multi-process lifecycle test (six simultaneous autostarts, mismatch, stale restart, no HOME and order-independent paths). `handoff-cli.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**. Production facade over IPC, verified immutable files/native image reuse, coalescing, external saved-data/appearance refresh and prompt detach with an admitted read queued behind live work; work completes after client exit. `handoff-client.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --bin regressee-desktop engine_refresh_tests:: -- --test-threads=1` | **1 passed**, 231 filtered. Screen/stage preserved and conflicting drafts retain their old revision. `handoff-refresh.log`. |
| `cargo test --locked --offline -p regressee-gui --all-features --lib --bin regressee-desktop -- --test-threads=1` | Ran twice: each **326 passed, 5 inherited failures, 63 ignored**. Latest library 97 passed/1 ignored, desktop 229 passed/5 failed/62 ignored. `final-gui-suite.log` (292.07 s desktop). Same five names, assertions and values as Phase 1; no new GUI failure. Final facade/refresh checks above were rerun after last hardening. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Passed, `final-check.log`; final clippy also checked all targets/features after the last source changes. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Final source passed, `handoff-build.log`, with no test/developer/native-inspection feature leakage in either public binary's metadata. |
| `cargo test --locked --offline -p regressee-gui --features ui --lib --bin regressee-desktop --no-run` | Passed, `final-ui-only.log`; no missing test-support gate was papered over by the all-feature build. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features` | Final source exits **0**, `handoff-clippy.log`. Four new collapsible-if warnings were repaired. Remaining warnings are inherited widget/test patterns plus `proc-macro-error2` future-compatibility notice; `-D warnings` is not claimed clean. |
| `TMPDIR=…/agent-cli-phase2 python3 -B -m unittest discover -s scripts -p 'test_*.py'` | **59 passed**, `final-python-tests.log`, including explicit public/internal documentation allowlist updates. No publication performed. |
| `rustfmt --edition 2024 --check --config skip_children=true <all changed/new .rs paths>`; `git diff --check` | Passed. Exact expanded file list and source hashes: `source-handoff.json`; `handoff-rustfmt.log`. |
| `cargo fmt --all -- --check` | Still fails only the inherited formatting set: setup-progress/timezone widgets, options UX and renderer module order. `handoff-global-fmt.log`. No broad reformat of unrelated work. |

Final clean-target CLI tests were also rerun after the fresh no-GUI proof:
`PKG_CONFIG=/bin/false LIBRARY_PATH= CARGO_TARGET_DIR=…/cli-clean-target cargo test
--locked --offline -p regressee-cli -- --test-threads=1` passed 4 tests
(`cli-clean-proof/final-source-tests.log`). Windows transport source was rechecked
with the same verified private sysroot (`windows-api/final-transport-check.log`).

`handoff-smoke/results.json` records production `regressee` **and**
`regressee-desktop` `--build-info` and explicit `engine start/status/stop`, each
with isolated HOME/data/cache, no DISPLAY/Wayland and an invalid session bus:
all lifecycle exits **0/0/0**, no runtime or private-session directory, empty
HOME, and ownership lock released after stop. This is engine-only startup, not
a native-window test. CLI invalid UTF-8 arguments also fail cleanly with exit 2.
The maintained desktop uses its own known executable to host the engine, so a
stale CLI sibling in a development/partial-upgrade directory cannot break it.
Windows daemon startup requests detached group/job breakaway and fails closed
when an enclosing job forbids independent lifetime; native verification is open.

### Handoff limits and next gate

- **No known Phase 2-specific failing check remains.** The tree is not globally
  green because the five independently attributed GUI failures and inherited
  formatting/strict-lint debt remain. No inherited failure was hidden or fixed.
- Independent Phase 2 verification must review ownership/security, durability,
  consent/deduplication, live broker/control, production GUI boundary and the
  final source/receipts. Repair its task-related findings when supplied.
- No native GUI interaction/renderer, Windows/macOS functional IPC/ACL/console
  lifetime, clean headless browser/sandbox, installer/release or new public-site
  qualification is inferred. Windows evidence is the actual transport/platform
  source type-check, not full app cross-linking or OS execution.
- Retained-receipt quotas, conservative confirmation invalidation and uncertain
  crash windows are deliberate documented limits, not dropped outcomes or
  automatic retry. Close pre-engine executables before upgrading.
- Full product CLI namespaces/JSON/report/crop UX (Phase 3), exclusive sessions,
  leases/follow banner/handoff/takeover UI (Phase 4), skill/packages/qualification
  (Phase 5) remain deferred. Epoch hooks are not claimed as a completed lease or
  exclusive-follow system. No new commit/push was made; all work remains for the
  orchestrator's independent verifier.

## Phase 2 independent-review repairs — historical re-review handoff

The independent **NEEDS FIX** verdict remains authoritative until its author
re-reviews this repair. `docs/agent-cli-phase2-review.md` was read in full and is
unchanged (SHA-256 checked against `before-hashes.json`). No commit/push, actual
user-store access, outside-site capture, native window launch or sandbox change
was performed. No Phase 3/4 work was started.

Repair-touched paths (distinct from the already-uncommitted initial Phase 2 work):

```text
crates/app/src/lib.rs
crates/app/src/protocol.rs
crates/app/src/jobs.rs
crates/app/src/engine/mod.rs
crates/app/src/engine/executor.rs
crates/app/tests/engine.rs
crates/storage/src/lib.rs
crates/storage/src/capture.rs
crates/gui/src/main.rs
crates/gui/src/activity.rs
crates/gui/src/task.rs
crates/gui/src/task_ux.rs
crates/gui/src/local_jobs.rs
crates/gui/src/engine_refresh_tests.rs
crates/gui/tests/engine_client.rs
crates/cli/tests/engine_lifecycle.rs
docs/agent-cli-status.md
docs/local-engine.md
scripts/source_release.py
```

### Findings and analogous paths repaired

1. **Replacement policy bypass:** `SaveEnvironment` now uses the same protected
   active-baseline rules as ordinary `Save`, at the shared storage transaction
   boundary, not just a widget or protocol check. Source/profile/environment
   definition replacement cannot pass through that alternate operation.
   `SaveReplacementEnvironment` explicitly carries combined replacement/login
   intent and requires its own exact action/revision/settings confirmation.
   Engine, product and test-actor dispatch share the same environment-save
   handler. The GUI selects the explicit operation when saving baseline-bound
   replacements. Safe name/discovery/local-login saves remain token-free;
   transaction rollback, revision guards, no session references in project JSON,
   and the separate baseline activation policy are preserved. Tests cover all
   three protected field families, safe saves, wrong action/settings tokens,
   consumed consent, original-ID retry, activation bypass and both atomic-save
   entrypoints under an injected SQLite write failure.
2. **Preview success misclassification:** all three capture-producing operations
   (`Capture`, `CaptureInventory`, `PreviewImages`) use the explicit `Captured`
   result, centrally classify the actual run state, and fail closed on missing
   results or mismatched project/resume/progress/intent/analysis identities.
   The operation/state matrix covers Completed, CompletedWithErrors, Cancelled,
   Paused, Interrupted and unexpected Running, as well as empty/foreign results.
   Structured worker/browser requirements and product failures remain structured;
   a changed visual comparison still does not mean operational failure.
3. **Resumed-run substitution:** every capture handler retains the service's
   returned `RunRecord` and uses one shared `capture_reply` helper. Storage reads
   evidence for that exact saved run, not the newest inserted project run; checks
   retain their original baseline/analysis/reviews. Paused/failed checks need not
   have an analysis. Tests create older saved runs with five different outcomes
   followed by newer successful or failed runs, and check exact identity/status,
   no borrowed analysis, and reference binding when today's baseline differs.
   A producer-level test exercises the actual shared reply helper. GUI task
   completion consumes the exact receipt; workspace refresh is a separate read
   under existing draft/error guards. The GUI regression confirms that separation.

The changed result/operation contract is **protocol 2**. Old/future peers are
refused before submitting work; both entrypoints must be rebuilt/restarted.
The existing `engine-v1/` directory is still the on-disk container, not the wire
version. Legacy capture receipts with unbound workspace results are retained as
`Interrupted` / `UnverifiedCapture { message, prior }`, preserving their prior
payload and original submit rather than certifying their former success, losing
their evidence, or replaying a possibly committed mutation. Restart/idempotent
retry of that uncertain receipt is covered. Previously committed product data is
not rolled back and historical consent is not invented.

### Reproduction and regression receipts

All repair scratch is under
`/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase2-repair`, with the same
explicit Rust 1.98.1 PATH and Phase 1 `link-libs` directory. `TMPDIR` was set to
the owned review/repair root for runtime fixtures; heavy shared-target compilation
was serial and implementor-owned.

- Before changing behavior, reran the verifier's exact external probe command:
  `CARGO_TARGET_DIR=/var/home/olav/Projects/Regressee/target cargo test --locked
  --offline --manifest-path /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase2-review/probe/Cargo.toml
  -- --nocapture --test-threads=1`. All **8 passed**, reproducing the three wrong
  behaviors asserted by its defect witnesses (`independent-witnesses-before.log`).
- Added correct-expectation in-repo regressions before the fixes. Each failed
  **Succeeded vs Failed**, not a compile error: `regression-red-environment.log`,
  `regression-red-preview.log`, `regression-red-resumed.log` / `red-results.json`.
- Reran each of the original three external witness names after the fixes, with
  `--exact --nocapture --test-threads=1` and the same manifest/target. Each now
  exits **101 at its assertion of the old wrong behavior**, not compilation:
  SaveEnvironment fails and does not persist the replacement; preview fails;
  the unrelated newest run is rejected as a capture receipt. Their assertions
  were **not** rewritten or hidden. Exact commands/logs:
  `independent-witnesses-after.json` and `witness-after-*.log`. Positive in-repo
  tests additionally prove valid `Captured` receipts classify the actual older
  run correctly rather than merely rejecting every result.

| Exact command (Rust PATH / private TMPDIR / linker environment as above) | Repair result |
| --- | --- |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1 --skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` | Final source **230 passed, 0 failed, 29 ignored, 1 scale test filtered**, 28.75 s. App library **56/1 ignored**, engine integration **18/1 ignored**, CLI **4 passed**. Nested environment subprocess not double-counted. `handoff-backend.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**. Extended production IPC facade test rejects unsafe ordinary environment save, then performs confirmed explicit replacement. `client-final.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --bin regressee-desktop engine_refresh_tests:: -- --test-threads=1` | **2 passed**, 231 filtered. Includes exact capture completion vs separate workspace refresh. `refresh-final.log`. |
| `cargo test --locked --offline -p regressee-gui --all-features --lib --bin regressee-desktop -- --test-threads=1` | **327 passed, 5 inherited failures, 63 ignored**. Library 97/1 ignored; desktop 230/5 failed/62 ignored (290.27 s). Same five names/assertions/values retained from Phase 1, no new failure. `gui-regression.log`. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Final source passed, `check-final.log`. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Passed on final source, `handoff-build.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui --lib --bin regressee-desktop --no-run` | Passed, `ui-only.log`; inherited warning/feature-matrix limits remain. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features` | Exit **0**, `handoff-clippy.log`. New test complexity/needless-question-mark warnings repaired; inherited GUI warnings remain. No strict-warning-clean claim. |
| `TMPDIR=…/agent-cli-phase2-repair python3 -B -m unittest discover -s scripts -p 'test_*.py'` | **59 passed**, `python-final.log`. The new independent report is explicitly classified as internal documentation by source packaging, not published. |
| `rustfmt --edition 2024 --check --config skip_children=true <all changed/new .rs files>`; `git diff --check`; CLI normal/build `cargo tree --locked --offline -p regressee-cli --edges normal,build --prefix none --format '{p}'` | All passed; dependency closure remains GUI/GPUI-free. Expanded commands, final source/binary hashes and report-preservation proof: `handoff-source.json`. |

`regression-results.json`, `final-results.json`, `handoff-results.json` and
`summary.json` retain expanded commands/counts. Production CLI and desktop
engine-only start/status/stop were rerun with protocol **2**, private HOME/data/
cache, no display and an invalid session bus: six exits **0**, empty homes, no
runtime or credential directories, owner locks released (`smoke/results.json`).
The CLI's real six-process lifecycle regression also now explicitly rejects
protocol **1** as well as a future version, without starting a second owner.
For independent re-review, handcrafted endpoint records must use the negotiated
`VERSION` (now **2**): the old external probe's hardcoded `version: 1` now tests
version refusal, not UUID mismatch. The unchanged wrong-behavior witnesses are
not expected to become green acceptance tests; their three exact after-runs
are documented above, alongside the positive in-repo regressions.

### Re-review scope and preserved limits

- Ready for the orchestrator to resume the independent verifier. The report's
  verdict has not been changed to PASS; implementation tests are not independent
  acceptance. No known repair-specific failing check remains.
- The non-newest-run and representative-state matrices use real saved synthetic
  storage plus the actual producer/classifier; no actual browser resume/preview
  was executed in this repair. GUI coverage is IPC plus GPUI simulation, not
  native presentation/input qualification.
- Scale qualification was deliberately not rerun; its earlier receipt remains
  historical. Native Windows/macOS IPC, managed-login/capture lifetime, clean
  headless browser/fonts/sandbox, release and accessibility gates remain open.
- Five inherited GUI failures, global formatting/strict-lint debt, receipt quotas
  and uncertain crash windows remain documented. No next phase, commit or push.

## Phase 3 — implementation handoff (independent verification pending)

The orchestrator supplied the independent Phase 2 **PASS** before this phase.
Its report is preserved unchanged. No commit/push/publication, user-store access,
external-site capture, native window launch, system installation or sandbox change
was performed. No subdelegation. Existing UI work was retained; the desktop's
only new production presentation-adjacent change is public directory selection
before native startup, not widget/draft/navigation behavior.

### Implemented product surface

- `crates/cli/src/commands.rs` and `main.rs`: named typed workflows rather than
  a generic RPC facade. Project create/list/show/update and full bounded JSON/stdin
  configuration; profiles/environments/scope; atomic local environment settings
  and explicit combined replacement; discovery/list/include/exclude/existing
  inventory; representative preview, baseline capture, fixed-list check/resume;
  durable jobs/list/pagination/show/wait/pause/cancel/retry/read acknowledgement;
  results/filters/coverage/run+analysis selection; sensitivity/custom policies,
  ignored rectangles, review decisions/notes; baseline save/partial/selected
  approval/accept/restore; history/usage/deletion/repair; ZIP sharing; diagnostics,
  browser/network status/setup, desktop login and appearance.
- Versioned schema-1 JSON success/failure/parser envelopes and stable exit families
  distinguish observed differences (1), incomplete captures (3), requirements
  (4), stale conflicts (5), engine/IO errors (6), uncertainty (7), and detached
  Ctrl-C wait (130). Human output has a readable label/job identity and result.
  Metadata/help/capabilities do not touch data/display. No universal confirmation
  bypass or hidden prompts. Read-only commands do not install/download browsers.
- Consequential operations produce exact tickets, committed only with their
  explicit confirmation UUID. Engine operation/revision/preview/selection/path
  checks remain authoritative. Baseline save/restore/replacement tickets now also
  include actual engine-resolved preview details/counts. Additional engine
  pre-validation provides stable invalid/conflict/replacement failures before
  existing transaction checks; it does not replace those checks. Tests expecting
  the former ambiguous ConflictOrWrite response now assert the explicit
  `replacement_required:` failure, with unchanged no-write/safety expectations.
- Original complete submissions are privately, atomically published/fsynced before
  IPC and announced on stderr. Structured uncertain failures preserve request ID
  and retry path. `jobs retry` uses the original owner/epoch/revision/confirmation
  payload, never reconstructs today's settings. Read receipts are reclaimable;
  mutation/confirmation authority is retained with documented admission quotas.
  Ctrl-C only detaches a waiter. Pause is explicitly limited to checkpoint-aware
  inventory capture/check/resume jobs; cancel remains a separate cooperative
  request, never an implicit rollback or a rewrite of committed success.
  Confirmation use on a different token-free action is also rejected, not ignored.
- `crates/app/src/terminal.rs`: engine-owned exact original/before/after/difference
  metadata, saved analysis/region/mask/warning/comparability data, explicit failed
  capture gaps and device-scale/screenshot-pixel coordinates. Bounded derived
  crops validate containment, geometry, exact hashed source bytes and complete PNG
  streams without overwriting originals. Report previews bind selection, analysis,
  content fingerprint, revision and canonical new destination; static escaped
  HTML + concise Markdown + selected unchanged PNG copies. No JS, remote resources,
  auth-session/credential-store files, network configuration, private logs or
  upload. Private page pixels/URLs/query strings/notes/excluded inventory/local
  artifact paths are explicitly previewed as privacy risks, not silently redacted.
- `launch::configured_paths/desktop_paths`: shared public CLI/desktop data/cache
  policy and normal defaults; explicit paths override development hooks.
  A live owner refuses a mismatched canonical cache; a stopped owner's stale
  same-version descriptor does not prevent deliberately changing cache selection.
  No client Store/recovery or GPUI dependency was added. Linux no-display managed
  login fails explicitly before setup/credential initialization. Existing desktop
  login and baseline ZIP secret-exclusion service contracts are unchanged.
- [cli.md](cli.md): executable new/existing project examples, complete actual
  command/action coverage inventory, input DTO references, output/exit/error/job/
  Ctrl-C/retry/confirmation/evidence/privacy/limitations contracts. Relevant README,
  source-build, engine and source-release documentation classification updated.

### Verification environment and receipts

Owned scratch/logs: `/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase3`.
Rust uses the installed 1.98.1 toolchain on PATH, `TMPDIR` is this scratch root,
and GUI linking alone uses Phase 1 `link-libs`. Shared-target heavy compilation
was serial and implementor-owned. No fresh independent/no-GUI target rebuild is
claimed in this phase; dependency/build checks complement the prior fresh proof.

| Exact command (Rust PATH/private TMPDIR as above) | Result |
| --- | --- |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1 --skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` | **237 passed, 0 failed, 29 ignored, 1 scale test filtered**; nested environment subprocess not double-counted. App library **57/1 ignored**, engine **18/1 ignored**, CLI **10 passed** (1 exit-contract unit + 1 lifecycle + 8 workflow subprocess tests). `backend-complete.log`. Later cache-handshake tightening is covered by the final focused run below. |
| `cargo test --locked --offline -p regressee-cli -- --test-threads=1` | **9 passed** (1 lifecycle, 8 workflow), `cli-final.log`; final backend run above includes the same nine plus later memory/space/retry/launch validation changes. |
| `cargo test --locked --offline -p regressee-app -p regressee-cli --features regressee-app/test-support --lib --bins --test engine --test engine_lifecycle --test workflows -- --test-threads=1` | Final source **85 passed, 0 failed, 2 ignored**: app 57/1 ignored, engine 18/1 ignored, CLI 10 passed. Includes stopped-owner cache change, live-cache mismatch, read-receipt reclamation, token-free wrong-confirmation and Interrupted exit-precedence regressions. `app-cli-handoff.log`. Not added to duplicate bulk totals. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | Final source **1 passed**, `gui-client-delivery.log`: production GUI facade over shared IPC, including protected ordinary save rejection and confirmed replacement. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --bin regressee-desktop engine_refresh_tests:: -- --test-threads=1` | **2 passed**, 231 filtered, `gui-refresh-final.log`: saved-state refresh retains draft revisions; capture completion retains exact run rather than workspace refresh identity. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Passed, `check-workspace.log`; final all-target clippy also checks the expanded source. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Final source passed, `build-delivery.log`: CLI, desktop and maintained developer/prototype binaries. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features` | Final source exit 0, `clippy-delivery.log`; new large-enum/lazy-option/needless-question-mark warnings fixed. Inherited GUI/duplicate-fixture-module warnings remain; not strict-warning-clean. |
| `TMPDIR=…/agent-cli-phase3 python3 -B -m unittest discover -s scripts -p 'test_*.py'` | **59 passed**, `python-complete.log`; includes the new public CLI documentation allowlist. |
| `env -u DISPLAY -u WAYLAND_DISPLAY PKG_CONFIG=/bin/false LIBRARY_PATH= cargo build --locked --offline -p regressee-cli --bin regressee` | Passed using existing shared target (not a fresh-target proof), `cli-only-build.log`. CLI normal/build `cargo tree` excludes GPUI/GUI/X11/Wayland; ELF NEEDED is only libc/libm/libgcc/loader (`cli-dependencies.txt`, `cli-elf.txt`). |
| `rustfmt --edition 2024 --check --config skip_children=true <phase3 touched Rust paths>`; `git diff --check` | Passed. `cargo fmt --all -- --check` still reports only the four inherited GUI formatting paths in `global-fmt.log`; unrelated user formatting was not rewritten. |

The CLI workflow tests launch actual CLI subprocesses against a real engine and
real project/discovery/capture/check/analysis/review/baseline/report/ZIP/retention
services. A privately signed **synthetic Python worker**, not Chromium, supplies
owned link responses and PNG captures. Scenarios include new-project-to-cleanup,
preserved original check/baseline history, changed/failed captures, detached
Ctrl-C/pause/resume/cancel/retry, stdin/config/settings/diagnostics, stale revisions/
approval/report/confirmation IDs, private notes and HTML injection, forged crop
sources/corrupt CAS PNGs, geometry/overflow/symlink/data-root output refusal, input
budgets and output nonoverwrite. A separate production-autostart test verifies
status/network/diagnostics/list do not install a runtime or initialize sessions.
No claim of actual Chromium/public website/desktop login/browser-sandbox evidence.

`production-smoke-final/results.json`: production CLI metadata/capabilities/help, rejected
command, start/browser-status/status/stop with private HOME/data/cache, no display
and an invalid session bus: exits **0/0/0/0/2/0/0/0/0**, empty HOME, no runtime or
session directory, metadata UI/developer/test/inspection features all false.
The same receipt also verifies plain `--build-info` JSON for both final production
executables without graphics/data startup (desktop UI true, CLI UI false; all
developer/test/inspection flags false), each exit 0.

### Deliberate remaining gates/limits

- **Independent Phase 3 verification is pending.** No independent PASS or global
  green-tree claim. The unchanged expensive 100/1000-page scale case and the five
  independently attributed inherited GUI failures were deliberately not rerun.
- Native GUI interaction/accessibility, actual managed login/capture/resume,
  Windows/macOS IPC/ACL/lifetime, clean headless browser/fonts/default sandbox,
  release packages and portable agent skill qualification remain their existing
  Phase 4/5/platform gates. This phase adds no sandbox exception or upload.
- Follow/exclusive sessions/leases/semantic navigation/handoff/takeover are **not**
  implemented or advertised. Existing GUI saved-data refresh still does not move
  navigation or discard drafts.
- Receipt quotas/uncertain commit windows and conservative stale-consent
  invalidation remain explicit. Report selection is capped at 100 targets/256 MiB
  copied images/8 MiB metadata; crops at 4M pixels, source decode limits preserved.
  Output I/O failure may leave a partial new report directory, explicitly failed;
  originals are never overwritten. No automatic journal-deletion policy is added.

## Phase 3 independent-review repairs — ready for re-review

The independent **NEEDS FIX** verdict in `agent-cli-phase3-review.md` remains
authoritative until its author re-reviews these repairs. The report was read in
full and preserved byte-for-byte (before/after SHA-256 in the repair receipts).
No Phase 4 work, commit, push, user-store open, outside-site capture, system
installation, native window launch or sandbox change was performed. No agents
were delegated. Existing UI source/work was not changed by this repair.

### Findings and analogous paths addressed

1. **Original reference identity:** terminal inspection resolves one default run,
   or the run of an explicit analysis, then uses `project_evidence_for_run` for
   results, artifacts, crop and report paths. Check run/analysis/original baseline
   consistency is checked even when identity flags are omitted. Exact non-check
   reads do not borrow the active baseline; the focused viewer retains a check's
   original reference even if analysis is absent. Artifacts explicitly expose
   original `baseline` and `before_run` identities. Post-accept/restore regressions
   compare omitted/explicit selection, exact hashes, crop sources and immediate
   default preview→report export without changing the saved analysis/difference.
2. **Current notes versus bounded history:** focused storage SQL returns the last
   committed decision for each selected analysis/target independently of the
   latest 256 run-wide decisions. Result rows and reports use that current view
   and disclose its semantics. `review history --analysis ID [--page ID --profile
   ID --before DECISION_ID --limit N]` adds bounded keyset pagination (1–200),
   stable retained decision IDs, total and continuation cursor. Existing recent
   workspace/history windows expose `review_history_truncated`; review-save CLI
   replies disclose their recent-window limit/possible truncation. Tests retain
   a first target's note after 256 unrelated writes, retrieve all 257 decisions
   without duplicates and keep old/new analysis reviews separate.
3. **Prepared-request recovery:** one result formatter reconstructs the original
   typed operation plus original Confirmation for normal prepare, show, wait and
   retry. `--save` writes the same usable ticket, not a bare Confirmation. Original
   owner/epoch/revision/fingerprint/ID are checked/preserved; no preparation,
   consumption, authority refresh or mutation replay occurs. Tests recover through
   all three routes, confirm once and reject replay while recovery still returns
   the unchanged, now-consumed ticket.
4. **Interruptible prerequisites:** configuration helpers no longer call blocking
   `Client::execute`. Both `Load` and optional `EnvironmentStatus` go through the
   normal journaled interrupt-aware submit/wait path with their own public IDs and
   recovery files. Interruption returns 130 with `phase:configuration_prerequisite`
   and `continuation_submitted:false`, leaves admitted work running and prevents a
   later save after observing interruption. Startup/before-submit guards do the
   same without inventing an admitted job. Tests cover profiles, environments,
   scope, environment select with/without supplied login settings and its second
   read; they assert prompt exit, unchanged revision and no continuation save.
5. **Predictable validation:** local parse/input/domain/preflight errors have an
   explicit typed validation context; engine-tagged errors use exact categories,
   not incidental substrings in human prose. Names, URLs, full project/profile/
   environment/scope inputs, login settings, policies/masks, selected keys,
   pagination, non-nil IDs and prepare applicability are checked before directory
   discovery/autostart where they require no saved state. Stored-state/revision/
   geometry checks remain in the engine. Tests include the three reported domain
   failures, inappropriate prepare flags, valid-schema invalid settings and
   no-start assertions, while retaining clean schema-1 stdout.
6. **Frozen report provenance:** report preview/fingerprint/HTML now include the
   recorded queue/discovery settings, capture project revision and resume times,
   plus each selected reference source's recorded queue, profiles, environment
   and rendering settings. Analysis/masks/comparability and original baseline
   remain bound to the selected saved run. Legacy absence is null with explicit
   unknown limitations, never substituted from today's project. Privacy inventory
   names the added scope/authorization/pacing/revision/resume/source-setting data;
   the 8 MiB metadata bound still applies. A legacy-fixture test and a real CLI
   test after changing current discovery settings prove both cases.

Repair-touched source paths (all prior Phase 1/2/3 and user work retained):

```text
crates/cli/src/{commands,main}.rs
crates/cli/tests/{engine_lifecycle,workflows}.rs
crates/cli/tests/support/phase3_repairs.rs
crates/app/src/{terminal,protocol}.rs
crates/app/src/engine/mod.rs
crates/core/src/evidence.rs
crates/storage/src/{capture,history_inspection,lib,review}.rs
docs/{agent-cli-status,cli,local-engine}.md
scripts/source_release.py
```

The repaired DTO/operation contract is **protocol 4**; rebuild/restart both clients
and engine. The CLI envelope remains schema 1; storage/ZIP formats are unchanged.
Legacy stored DTO fields default for journal readability, but old report previews
must be regenerated; retained old read receipts are not retroactive certification.
The new independent report is classified as internal source-release documentation,
as its author requested. The report itself was not edited.

### Reproduction and verification receipts

All repair fixtures/logs live under the previously approved root:
`/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase3/repair` (`R` below).
Rust PATH is the installed 1.98.1 toolchain; `TMPDIR=$R`; GUI linking alone uses
Phase 1 `link-libs`. Shared-target heavy compilation was serial and implementor
owned. No fresh-target/no-SDK qualification is inferred from the cached build.

- Added six correct-expectation subprocess regressions **before** behavior fixes.
  `regressions-red-complete.log`: all **6 failed at their assertions**, not compile
  or fixture setup. An earlier red run timed out because the test teardown joined
  an engine with still-admitted work; teardown now explicitly cleans up only its
  own synthetic fixture work before joining. No production cancellation policy
  was changed to satisfy the test.
- The first post-fix bulk run passed. One later bulk rerun caught a test-monitor
  race: a snapshot-listed completed prerequisite read could be acknowledged before
  the monitor fetched it. The monitor now handles that documented `not_found`
  response, while retaining prompt-130, exact-read, no-save and revision assertions.
  `config-prerequisites-final.log` and the final full run pass; the intermediate
  failure log is retained rather than hidden.
- External reviewer scripts/host were copied to `$R/external`, not run against
  user data or used to overwrite the review evidence. The host was rebuilt against
  repaired source and the copied CLI is a **production** build (all developer/test/
  UI flags false). The copied parser's baseline-handoff path was adjusted only to
  resolve the original hash manifest from the new scratch location; reported source
  mismatches are the expected repair delta.
- The unchanged external default-reference scenario first failed on its assertion
  of the **old** guard result (`expect=5`, actual successful export **0**), not on
  bad evidence (`external-original-default-guard.log`). In the scratch copy only,
  that export expectation/comment was changed to require success; the original
  before-hash and baseline-identity assertions were retained and pass. No verifier
  report/source script was overwritten or represented as an unchanged green test.

| Exact command / environment above | Final repair result |
| --- | --- |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1 --skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` | **244 passed, 0 failed, 29 ignored, 1 scale test filtered**, excluding the nested environment subprocess from totals. App **58/1 ignored**, engine **18/1 ignored**, CLI **16 passed** (1 unit + 1 lifecycle + 14 workflows). `backend-delivery.log`. |
| `cargo test --locked --offline -p regressee-cli --test workflows phase3_repairs:: -- --test-threads=1` | Initial repair **6 passed**, 8 filtered (`regressions-first.log`); all six plus expanded analogous-path assertions are included in final bulk above. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**, `gui-client.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --bin regressee-desktop engine_refresh_tests:: -- --test-threads=1` | **2 passed**, 231 filtered, `gui-refresh.log`. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Passed, `check-workspace.log`. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Passed, `build-final.log`; no native window launch. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features` | Exit **0**, `clippy-final.log`; inherited GUI/fixture/future-compatibility warnings remain, no strict-warning-clean claim. |
| `env -u DISPLAY -u WAYLAND_DISPLAY PKG_CONFIG=/bin/false LIBRARY_PATH= cargo build --locked --offline -p regressee-cli --bin regressee` | Passed with shared/cached target, `cli-only-final.log`; copied production binary metadata/hashes/protocol in `external/final-binaries.json`. |
| `CARGO_TARGET_DIR=/var/home/olav/Projects/Regressee/target CARGO_PROFILE_DEV_DEBUG=0 cargo build --locked --offline --manifest-path "$R/external/probe/Cargo.toml"` | Rebuilt copied synthetic host against current source, `external-host-final.log`. |
| `REGRESSEE_REVIEW_CLI="$R/external/bin/regressee-production" python3 -B -u "$R/external/workflow_probe.py"` | **8 passed** with the disclosed corrected default-export expectation; `external-workflows-final.log`. |
| Same production-CLI environment, `python3 -B -u "$R/external/review_history_probe.py"`, `report_settings_probe.py`, `interrupt_config_probe.py`, `tall_crop_probe.py` | **1 passed each**, four additional scenarios; `external-review-final.log`, `external-settings-final.log`, `external-interrupt-final.log`, `external-tall-final.log`. **12 external scenario passes total**, not independent re-review acceptance. |
| `python3 -B "$R/external/parser_probe.py"` | **27/27 clean JSON, 0 wrong exits, no startup**; `external-parser-final.log` and copied parser results. |
| `python3 -B "$R/external/production_probe.py"` | **12 production invocations**, no-display/read-only/runtime/credentials checks pass; invalid prepare does **not** create storage. `external-production-final.log`. |
| `TMPDIR="$R" python3 -B -m unittest discover -s scripts -p 'test_*.py'` | **59 passed**, `python-tests.log`, including source-document classification. |
| `rustfmt --edition 2024 --check --config skip_children=true <repair .rs paths>`; `git diff --check` | Passed; final expanded commands/hashes and report-preservation/owned-lock checks are in `repair-handoff.json`. |

### Remaining limits and next gate

- Ready to resume the original independent reviewer; **NEEDS FIX has not been
  changed to PASS**. No known repair-specific product/test failure remains.
- Synthetic workers/local PNGs and owned loopback pack serving are not actual
  Chromium/public-site rendering, managed desktop login, native GUI input,
  clean-server fonts/libraries/sandbox, Windows/macOS, packaging or skill evidence.
  The unchanged 100/1000-page scale case and five inherited GUI failures were not
  rerun or waived. Existing global formatting/lint debt remains separate.
- Current reviews are complete for the selected analysis/targets; full history
  is explicitly paginated. Old recent windows are deliberately bounded/disclosed,
  not silently treated as current state. Deleting an analysis/run deliberately
  removes its associated history under the existing confirmed cleanup policy.
- Existing request quotas, uncertain commit windows, conservative confirmation
  invalidation, output size limits and partial-output IO failures remain documented.
  No new session/credential/remote markup export, baseline ZIP contract change,
  automatic upload, receipt deletion, sandbox bypass or Phase 4 control feature.

## Phase 4 — exclusive follow and human handoff

The orchestrator supplied the independent Phase 3 **PASS**, protocol 4, before
this phase. Both independent review reports were read and left unchanged. The
original handoff protocol was **5**; the public CLI envelope stays schema **1**. Rebuild
both entrypoints and restart the old engine explicitly as documented. No Phase 5,
commit, push, subdelegation, actual user-store access, external-site capture,
system installation, desktop-permission change or sandbox weakening occurred.
The implementor exclusively owned shared-target compilation.

### Implemented scope

- `app::control` and `engine::authority`: explicit named sessions, monotonic
  renewable leases (10–300 seconds, 90 default), engine-incarnation UUIDs and
  control epochs; one exclusive controller per canonical engine data root.
  At most 128 session records and 16 live GUI frontends per incarnation. A GUI
  frontend has a separate renewable 15-second heartbeat. CLI invocation exit is
  not logical agent disconnect. Restart invalidates ephemeral control, not jobs.
- Credentials travel in an authenticated private request envelope, not in saved
  submissions, job/event snapshots, logs, confirmation tickets or reports. Agent
  CLI credentials live in private `DATA/agent-sessions/UUID.json`; GUI credentials
  remain in memory. Agent credentials cannot change roles into GUI frontends.
  This is same-user application coordination, not a process/filesystem sandbox.
- Broker and actual queued dispatch check owner/session/frontend, engine, epoch
  and expiry. Exact confirmations additionally bind their owning principal and
  recheck current authority at use. Raw Submit, Prepare, direct appearance writes,
  Stop/epoch advance and cancel/pause cannot bypass an exclusive controller.
  Read-only inspection remains available. Current-incarnation read acknowledgement
  belongs to its originating session/frontend; orphaned old-incarnation reads can
  be reclaimed after restart without reviving obsolete credentials. Mutation and
  confirmation receipts are still retained. Identical old admitted submissions return only
  their existing truthful receipt; they never replay revoked writes.
- Immediate GUI takeover revokes new/queued agent authority. Handoff pauses
  agent writes/navigation and gives the following frontend full temporary human
  control; only that GUI can return control. End, expiry, lost frontend heartbeat,
  restart and failed/draining engine state release control without implicit job
  cancellation. Already-dispatched atomic work can commit and retain its actual
  result. Running capture/discovery keeps frozen inputs; baseline save is still
  separate and explicit.
- `agent start/list/show/renew/end/select/handoff`, global `--session`, help and
  capabilities. Project-only selection and exact project/run/analysis/page/profile/
  view selection work headlessly. Engine validation binds original references and
  target membership; selection never changes saved display pointers or approvals.
  No-GUI handoff pauses the agent but cannot invent a human frontend or return
  authority. No-following-GUI handoff returns structured `human_input`, confirms
  the agent is paused and exits `action_required`/4, rather than pretending a
  human frontend is available. There is no public CLI follow/return/takeover/raw-RPC bypass.
- Native GPUI: a persistent top banner names the controller/status/lease, with
  explicit **Follow agent**, immediate **Take control** (also Ctrl/Cmd+Shift+T),
  **Your turn — agent paused** and **Return to agent**. Default mode refreshes
  saved data/progress without following navigation or discarding drafts. Follow
  and return reject project/note/ignored-area/network/builder/page-list drafts,
  incomplete analysis-policy text and History-search IME composition,
  unfinished ignored-area gestures and selected baseline updates,
  focused IME composition and incompatible modal/task/pending-save states.
  Existing close/Quit save/discard guards remain usable rather than being trapped
  behind the input shield.
- Following disables mounted/focused fields at the control boundary, guards
  navigation/menu/action dispatch and shields ordinary pointer/scroll interaction.
  It does not tint evidence. Separate control/heartbeat, priority control-command,
  semantic-location and saved-data observers keep takeover independent of blocking
  product jobs and unrelated GUI busy state. Nested timezone-search IME is also
  suspended immediately, not only removed at the next paint. Idle heartbeat
  polls share immutable evidence and skip unchanged presentation revisions instead
  of cloning large runs or forcing an idle redraw. External activity displays actual
  job status/progress without manufacturing a GUI task or capture request.
- Semantic projection validates the complete selection before installing it as a
  unit, clears old identities while a new selection is being resolved, and checks
  incarnation/epoch/session/selection/data generations before accepting async
  metadata or pixels. Explicit older analyses use exact-run viewer reads rather
  than silently borrowing that run's current/default analysis. Reconnect does not
  silently resume following. Durable selection receipts retain only semantic IDs,
  not another copy of the entire run on each navigation; full GUI locations are
  separately reclaimable reads. This avoids amplifying the retained-receipt quota.
- Default saved-data refresh uses separate exact-run/analysis and History-page
  request/generation lanes. It keeps the current result and cached History page,
  remaps rows by ID, preserves History's separate baseline page and ignores late
  reads without clearing another request's busy/task state. A deleted/off-page
  selection becomes no selection, not row 0. Raw policy edits/IME, notes and
  ignored-area drafts defer evidence replacement. Clean saved note changes
  rehydrate even when their semantic identity is unchanged, without manufacturing
  an unsaved note from the previous saved text. Explicit load/evidence refresh
  and task Continue retain their intentional newest-workspace behavior.

### Visual scope and environmental limits

Loaded Impeccable directly, ran context **once** targeting
`crates/gui/src/main.rs`, read PRODUCT/DESIGN, the existing pre-extraction surface
brief and current shared controls, and read the craft floor before UI edits.
Existing Unified neutrals, system type, 40px actions, logo variants, original
screenshots, draft and consent rules were preserved. No new aesthetic interview,
sidecar/drift repair, palette, animation, dependency or shipping raster. The scoped
control UX is documented in `cli.md`, not an unrelated global identity rewrite.

All scratch, synthetic stores, credentials, signed synthetic-worker captures,
receipts and diagnostic PNGs are under
`/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase4`. Rust commands use the
explicit installed 1.98.1 toolchain; GUI linking alone uses Phase 1 `link-libs`.
Offscreen inspection uses the real GPUI/Blade renderer and native fonts, not HTML
or an OS-input claim. The first batched round captured 12 light/dark states at
1440×900, 1280×800 and 820×600, checking banner/action containment, focus, original
evidence retention and the appropriate logo. The final confirmation round
captured the same 12 states; **two inspection rounds total**, no cosmetic loop.
Subsequent changes suspend a nested IME handler, share immutable observations and
retain lightweight selection receipts and exact passive-refresh identities; they
do not change the inspected paint.

The native-window gate is **unqualified**: this session has neither DISPLAY nor
WAYLAND_DISPLAY, and no supplied authorized Wayland socket. No desktop connection
was invented and no KWin helper, permission change or sandbox exception was used.
Offscreen/simulated pointer, key and IME input is not OS input, accessibility or
Windows/macOS qualification. Native/release acceptance remains a separate gate.

### Verification ledger

The full inherited GUI suite and unchanged expensive persisted 100/1000-page case
were deliberately not rerun. The five independently attributed inherited GUI
failures and existing global formatting/strict-lint debt are not waived.

| Exact command (environment above) | Result / retained log |
| --- | --- |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1 --skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` | **253 passed, 0 failed, 29 ignored, 1 scale case filtered**; nested environment subprocess not double-counted. App library 60/1 ignored, new control integration 5 passed, existing engine integration 18/1 ignored, CLI 18 passed (1 unit, 1 lifecycle, 16 workflows). `backend-complete.log`. Before the final small role-registration/capability and GUI command-lane hardening; final focused runs cover those paths. |
| `cargo test --locked --offline -p regressee-app --features test-support --lib --test agent_control --test engine -- --test-threads=1` | Final backend source: **83 passed, 2 ignored** (60 library + 5 control + 18 engine integration). `current-app.log`. Includes lightweight receipt bounds, read-orphan reclamation, queued authority and exact consent/capture receipts. |
| `cargo test --locked --offline -p regressee-cli -- --test-threads=1` | **18 passed** (1 unit, 1 lifecycle, 16 workflows), `current-cli.log`. Includes real expiry, lightweight selection, complete public CLI workflow and explicit no-GUI `action_required`/4 after pausing. Backend/CLI source unchanged by the subsequent GUI-only draft/note hardening. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --lib --bin regressee-desktop agent_ -- --test-threads=1` | Final source **8 passed**: 1 nested-search IME test (98 filtered), 7 desktop tests (233 filtered). `current-gui-final2-gui-agent.log`. Includes real IPC/GPUI opt-in, exact passive result/History refresh, clean-note hydration versus drafts/IME, rapid selections, input/menu/IME lock, handoff/return/takeover, external jobs, immutable idle-cache reuse, disconnect/restart and prompt exit guards. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**; `current-gui-final2-gui-client.log`. Production facade, immutable evidence/cache, saved-data refresh and safe detach. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --bin regressee-desktop engine_refresh_tests:: -- --test-threads=1` | **2 passed**, 238 filtered; `current-gui-final2-gui-refresh.log`. Draft revisions and exact capture receipts versus separate workspace refresh. |
| Three focused History behavioral regressions, using the same desktop test command with each filter below | **1 passed per invocation**, 239 filtered each; `current-gui-final2-<filter>.log`. Exact commands in `current-gui-final2-commands.json`. No full inherited GUI suite or additional visual inspection. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Final source passed, `current-gui-final2-check.log`. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features` | Final source exit **0**, `current-gui-final2-clippy.log`; inherited widget/test/duplicate-fixture/future-compatibility warnings remain. No strict-warning-clean claim. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Final source passed, `current-gui-final2-build.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui --lib --bin regressee-desktop --no-run` | Passed, `current-gui-final2-ui-only.log`. Feature compilation only, not native window execution. |
| `env -u DISPLAY -u WAYLAND_DISPLAY PKG_CONFIG=/bin/false LIBRARY_PATH= cargo build --locked --offline -p regressee-cli --bin regressee` | Final source passed, `current-gui-final2-cli-only.log`; shared/cached target, **not** a fresh no-SDK-host proof. CLI normal/build `cargo tree` remains GPUI/GUI-free (`cli-dependencies.log`). |
| `cargo test --locked --offline -p regressee-gui --all-features --bin regressee-desktop ui_renderer_capture::ui_agent_capture::renderer_capture_agent_control -- --exact --ignored --test-threads=1`, with explicit private renderer roots | **1 passed per round**, 12 PNGs each at three widths × two themes × locked/human states. `visual-round1.log`, `visual-round2.log`; source-bound offscreen metadata and frozen saved evidence checks. Not OS input or shipping assets. |
| `TMPDIR=…/agent-cli-phase4 python3 -B -m unittest discover -s scripts -p 'test_*.py'` | **59 passed**; `python-final.log`. |
| `rustfmt --edition 2024 --check --config skip_children=true <24 Phase 4 Rust paths>`; `git diff --check` | Passed (`current-format-final.log`, `current-diff-final.log`); expanded paths/commands are retained in `current-format-final-commands.json`. Global `cargo fmt --all -- --check` still reports the four inherited paths: setup-progress/timezone controls, options UX and renderer-harness module order. No broad unrelated reformat. |

The additional History filters are
`confirmed_history_deletion_removes_only_the_reviewed_record`,
`history_inspection_reply_cannot_overwrite_new_review_or_numeric_drafts`, and
`stale_history_reply_is_rejected_and_recorded_check_is_readonly_with_saved_policy`.

`production-current/results.json` records **13 actual production invocations**
of the final rebuilt binaries (earlier smoke roots are retained separately):
CLI capabilities/build metadata/start/session start/show/renew/no-GUI handoff/end/
stop, plus desktop build metadata and engine-only start/status/stop. Twelve exits
are 0; the no-GUI handoff deliberately exits **4** with `agent_paused:true`.
Both homes stay empty, no managed runtime or OS-auth/session store is initialized,
protocol is 5 and neither binary leaks developer/test/inspection features. CLI
metadata is UI false, desktop UI true. Agent secrets are absent from those public
transcripts. Desktop engine Stop retains its existing plain-text acknowledgement;
an initial smoke-script JSON assumption was corrected, not a product defect.

Development logs are retained, not relabelled green. An early GUI run identified
missing environment-draft hydration after semantic projection; the projection now
initializes that exact project's saved-login hydration origin and the handoff/
return regression passes. One command-lane development run hit the combined GPUI
probe's 10-second wait limit before it had state diagnostics. The instrumented
probe passed **six consecutive repetitions** (`gui-race-results.json`); its short
external synthetic job then gained a 2-second observation window rather than
assuming sub-500-ms renderer scheduling. The final expanded control group passes.
The earlier timeout is retained for independent timing/race scrutiny, not claimed
as a native-input result or silently counted as a pass.
History-regression development also retained a test-only private-helper compile
error and an invalid seeded queue (evidence was supplied before completion); both
fixture issues were corrected without weakening the product invariant.

Expanded commands, final source/binary/review-report hashes, all diagnostic PNG
hashes and owned production-fixture lock-release checks are retained in
`source-handoff.json` under the scratch root. No secret credential content is
included in that receipt. The final focused runs are not added to the earlier
bulk totals as if duplicate tests were extra coverage.

Control regressions include bounded renewal/expiry and frontend heartbeat loss,
live 10-second CLI expiry, normal short-lived CLI reuse, raw nonowner controls,
forged principals, owner-bound confirmations, revoked queued writes/confirmations,
stale replays and restart receipts. A real signed **synthetic Python worker**
capture continues across takeover, retains its frozen project and successful
outcome, and does not silently save a baseline. This is not Chromium or an actual
outside website. GPUI regressions exercise actual mounted controls, pointer and
keyboard/IME guards, retained drafts, modal/task refusal, native prompt Cancel /
Discard-and-quit, exact older analyses, rapid project/analysis selections, default
saved-data refresh, external jobs and fail-safe engine loss/restart.

**Next gate:** separate independent Phase 4 verification, with findings returned
to this original implementor. No independent Phase 4 PASS or native gate PASS is
claimed by this implementation ledger. Phase 5 has not started.

## Phase 4 independent-review repairs — re-review handoff

The independent **NEEDS FIX** report is preserved unchanged. This is an
implementor repair record, not independent acceptance. No Phase 5, commit, push,
subdelegation, user-store access, compositor/helper/permission change or sandbox
weakening is authorized or performed. All repair fixtures, logs and diagnostic
captures use `/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase4-repair`.
The original review evidence remains unchanged under `agent-cli-phase4-review`.
Shared-target compilation is serial and implementor-owned.

### Repairs at the actual boundaries

1. **Anonymous GUI authority (HIGH):** protocol **6** adds a private bootstrap
   request carrying an engine-minted random capability, bound to the engine
   incarnation and published only in owner-private
   `DATA/engine-v1/frontend-capability.json`. Startup rotates it; clean owner
   shutdown removes it. Explicit GUI bootstrap proves possession and registers
   a separate in-memory frontend credential/15-second heartbeat. The legacy raw
   `Control(RegisterFrontend)` always rejects, as do missing/forged/old-incarnation
   proofs and authenticated role switches. Normal status/read/control calls and
   `uncredentialed()` never read the capability or establish a GUI role. Valid
   Renew/Heartbeat only update their own lease, without advancing navigation
   epochs, following, revoking controllers or changing saved data. They still
   prove the original valid principal and cannot revive expired/revoked sessions.
   Immediate legitimate GUI takeover remains independent of agent approval.
   This is a filesystem-capability coordination boundary, **not executable
   attestation or a strong same-user OS sandbox**: a program explicitly reading
   the private bootstrap file can imitate a frontend. Both entrypoints and the
   old engine must be rebuilt/restarted; protocol-5 peers fail negotiation.
2. **Retained input (HIGH):** a weakly registered per-window live input gate and
   gesture epoch covers shared vertical/reserved/both-axis scrollbar capture,
   page/wheel handling and the child scroll owner. Follow/Return synchronously
   disable the gate and cancel owned pan/split/mask/scroll/editor/dropdown state.
   Retained callbacks recheck current authority rather than their paint-time
   enabled flag; unlocking never revives a previously held gesture. Viewer
   zoom/pan/key/scroll, overlay, result/page selection, workspace disclosures,
   confirmation-reference callbacks, dropdown navigation and focused text
   selection/clipboard/IME/multiline scrolling also check the live boundary.
   Banner actions/details, Quit/close guards and programmatic engine projection
   remain separate permitted paths.
3. **Long handoff layout (HIGH):** the existing neutral/system-type banner has
   a finite 40px status/action row and a separate 22px scrollable details owner.
   Names ellipsize; complete request/error/activity text remains in the details
   reading area with wheel and focused arrow/page/Home/End navigation. Take
   control/Return never scroll away. The maximum valid 1,024-byte message cannot
   consume more height than the ordinary compact banner. No evidence tint,
   asset, animation, palette change or global layout redesign was introduced.
4. **Passive project IME (MEDIUM):** hydration defers while any affected project,
   builder/login/locale or nested timezone-search editor owns preedit, including
   equal-visible-text and empty compositions. The draft keeps its old revision.
   Clean metadata-only refresh uses `set_value_if_changed`, preserving unchanged
   field selection, touched/correction state and editor geometry instead of
   calling a reset setter. Explicit project loads retain their deliberate
   hydration behavior.
5. **Silent expiry (MEDIUM):** the last observed named session retains a compact,
   nonlocking expired/disconnected/taken-back/ended/restarted explanation, including
   truthful job-continuation/retained-outcome language and a keyboard/pointer
   **Dismiss** lifecycle. A new controller replaces it. Neither dismissal nor
   reconnect restarts the lease, automatically follows, cancels jobs or undoes
   completed work. The reading area uses the same finite banner bound.

`scripts/source_release.py` classifies the new independent review as internal
documentation alongside the earlier reports; this is the review's requested
integration follow-up, not Phase 5 packaging work.

### Durable regression inventory

- `app/tests/agent_control.rs`:
  `anonymous_ipc_cannot_mint_gui_authority_and_bootstrap_is_private_incarnation_bound`
  covers raw registration/Follow/Takeover/Return/Detach/Heartbeat/Renew refusal,
  forged capability and role-switch rejection, no anonymous write, no implicit
  registration by reads, legitimate takeover, private/redacted proof, epoch-neutral
  registration/renewal, immediate frontend retirement without unrelated-session
  revocation, shutdown removal and stale-bootstrap rejection after restart.
- `gui/src/agent_repair_tests.rs`: actual-IPC held-scrollbar input spanning Return
  transition and acknowledged Active; retained viewer function callbacks; all
  affected project fields and nested timezone search with same-text/empty IME;
  unchanged editor metadata/selection; terminal/disconnect/restart status and
  deliberate keyboard dismissal; bounded long-text layouts at all three widths.
- `ui/scrollbar_tests.rs`: a real previously acquired shared scrollbar capture,
  wheel refusal, no held-drag revival after unlock, then a fresh working gesture.
- `ui/text_input.rs`: retained key/selection/clipboard/scroll/IME handlers while
  the old entity still says enabled; no content, selection or preedit mutation.
- `ui_agent_capture.rs`: an explicitly ignored, bounded repair capture matrix for
  maximum name/request, long failure, expiry, disconnect and takeover states,
  three supported widths and both themes; viewport/action containment and frozen
  original-evidence assertions are part of the capture test.

### Verification and remaining gate

Final command/count/capture receipts are recorded below after the serial repair
batch. Native-window, OS input/IME, accessibility and Windows/macOS qualification
remain **UNQUALIFIED**: no DISPLAY/WAYLAND_DISPLAY or authorized socket was
supplied. Existing five inherited GUI failures and four global-formatting paths
are not waived; the unchanged expensive persisted scale test and full inherited
desktop suite are not rerun. Resume the independent verifier for scoped re-review;
this implementor does not declare a Phase 4 PASS.

| Repair command / scope (Rust 1.98.1, private TMPDIR, Phase 1 GUI link libraries) | Result / retained receipt |
| --- | --- |
| `cargo test --locked --offline -p regressee-app --features test-support --lib --test agent_control --test engine -- --test-threads=1` | **84 passed, 2 ignored**: 60 library, 6 control integration, 18 engine integration. Final authority/Detach source: `seal-app.log`. |
| `cargo test --locked --offline -p regressee-cli -- --test-threads=1` | **18 passed** (1 unit, 1 lifecycle, 16 workflows), `final-cli.log`. After the final frontend-retirement hardening, `--test workflows phase4_control::` additionally passed **2**, 14 filtered (`seal-cli-control.log`); not added to the full count. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --lib --bin regressee-desktop agent_ -- --test-threads=1` | Final source **15 passed**: 3 library, 12 desktop; `finish-final-gui-agent.log`. Includes finite nonzero title/status width, maximum message layout, full-request keyboard scrolling, actual-IPC retained drag and terminal status/dismissal. |
| GUI library filters `ui::text_input::tests::`, `ui::scrollbar::`, `ui::timezone_select::tests::` with `--features ui-tests --lib` | **11 / 5 / 9 passed**, `final-widgets-*.log`. Includes three tests also in the agent group; counts are not cumulative. The final agent group rechecks the new retained-input cases after final changes. |
| GUI `--test engine_client`; desktop `engine_refresh_tests::`, `image_controls_tests::`, `split_view_tests::` | **1 / 2 / 2 / 2 passed**. Final facade/refresh: `finish-final-client.log`, `finish-final-engine_refresh_tests.log`; image/split: `final-image_controls_tests.log`, `final-split_view_tests.log`. |
| `cargo check --locked --offline --workspace --all-targets --all-features`; corresponding `cargo clippy` | Final source exits **0**, `finish-final-check.log`, `finish-final-clippy.log`. Only inherited warnings remain; no `-D warnings` claim. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Final production source passed, `finish-final-build.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui --lib --bin regressee-desktop --no-run` | Passed, `finish-final-ui-only.log`; compile-only, not native input. |
| CLI-only build with DISPLAY/Wayland unset, `PKG_CONFIG=/bin/false LIBRARY_PATH= cargo build --locked --offline -p regressee-cli --bin regressee` | Passed, `finish-final-cli-only.log`. Cached shared target, not a fresh clean-host SDK proof. |
| Exact ignored capture filter `ui_renderer_capture::ui_agent_capture::renderer_capture_agent_control_repairs`, `--all-features --bin regressee-desktop -- --exact --ignored --test-threads=1` | **1 passed per repair batch**, 30 frames per batch, at 1440×900 / 1280×800 / 820×600 in light/dark, five edge states. `repair-captures-round1-recheck.log`, `seal-repair-captures-round1.log`; source/evidence limits below. |
| Python script unit suite; scoped `rustfmt --edition 2024 --check --config skip_children=true` on **36** owned Rust paths; `git diff --check` | **59 passed**; format/diff exit **0** (`python-tests.log`, `scoped-format.log`, `diff-check.log`). Global fmt still reports exactly the four inherited paths (`global-format.log`). |

Expanded argv/exits/timings are in `repair-commands.json`,
`repair-resume-commands.json`, `repair-seal-commands.json`,
`repair-finish-commands.json` and `format-commands.json`. The early external-probe
lockfile refusal was a scratch package rename under `--locked`, repaired by
generating that private probe's lockfile **offline**; it is not a product failure
or counted as a passing test. The early expiry test used GPUI's retained debug
selector map to assert disappearance; it now asserts the actual mounted-state
predicate and deliberate dismissal lifecycle instead of stale selector history.

### External witnesses, production smoke and visual limit

- Original independent report and both original witness sources match
  `before-hashes.json` and are untouched. An owned engine-probe copy updates only
  the **legitimate setup/cleanup** helper to read the private bootstrap capability;
  the anonymous spoof witness body is byte-identical. The GUI copy appends the
  unchanged original witness module and uses a distinct binary name. Exact
  adaptation: `witness-adaptation.json` / `prepare_witnesses.py`.
- The original anonymous spoof witness now exits **101 at its expectation of a
  Credential**, not compilation, including the final authority source
  (`final-anonymous-witness.log`). The four original GUI wrong-behavior witnesses
  for retained zoom, same-text IME and local/actual-IPC scrolling now fail at their
  old assertions (`witness-gui-recheck.log`). Three positive external engine probes
  passed (`witness-engine-positive-recheck.log`); final backend tests additionally
  cover the later immediate frontend retirement semantics.
- **Stale expiry witness caveat:** its original assertion checks only the old
  controller/message/error/activity predicate, not the new last-session banner.
  It therefore still passes unchanged; it is **not acceptance evidence** and was
  not rewritten to manufacture a failure. The durable expiry/dismissal test and
  the captured visible `Agent session expired — window unlocked` state cover the
  repaired mount path directly.
- Final production `production-repair/results.json`: **13 invocations**, twelve
  exit 0 and deliberate no-GUI handoff exit 4. Both homes stay empty; no managed
  runtime or OS-auth store is initialized. CLI UI=false, desktop UI=true, no
  developer/test/capture features; protocol 6/schema 1. Both bootstrap files are
  private while running and removed after clean stop. Neither bootstrap nor agent
  secrets appear in public transcripts. Owned engine locks are checked released.
- **Bounded visual evidence:** `captures-round1/` and `captures-confirmation/`
  each contain 30 PNG/metadata pairs, plus diagnostic contact sheets in the repair
  root. No third builder visual inspection was performed. The first batch exposed
  a long unbroken name wrapping into status; the second exposed GPUI resolving
  percentage-width clipped text to an ellipsis-only measure. The final repair uses
  a finite width derived from the actual window, reserves the pinned actions, and
  passes new nonzero-width/20px-line/viewport regressions and all final builds.
  **That last finite-width paint correction was not recaptured.** These PNGs prove
  the bounded viewport/terminal-state repair stages, not final-source visual
  acceptance. The independent verifier must confirm final title/status paint;
  metadata and the handoff manifest explicitly retain this evidence gap.

`source-handoff.json` binds final sources/binaries/review reports, all logs,
expanded commands, production receipts and both capture-stage hashes. No secret
contents are included. Re-review remains required; no independent Phase 4 PASS,
native gate PASS or Phase 5 advancement is claimed.

## Phase 4 retained-feedback viewport repair — scoped verifier return

The protocol-6 re-review independently closed authority, retained input, passive
IME and terminal status. This repair touches only the remaining combined business
feedback/control-banner layout and its regression/capture coverage. The updated
independent report and original acceptance source remain unchanged. No Phase 5,
commit, push, unrelated interface/feature change, asset, theme, permission or
sandbox change was made. Compilation remains serial and implementor-owned.

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase4-feedback-repair`.
`before-proof.json` verifies all **43** handed-off source hashes and
the **30** re-review PNG/metadata pairs before this repair. The source/report
identities are frozen there; the prior final-source paint acceptance is preserved.

### Exact defect and repair

- Added the verifier's genuine saved-settings fixture without clearing or
  replacing its `Settings saved on this computer.` notice. Before the repair,
  `agent_feedback_tests::agent_retained_feedback_must_not_collapse_handoff_evidence`
  fails at the unchanged **64px minimum**: no-agent evidence **72px**, handoff
  evidence **2px**, banner **87px** (`regression-red.log`).
- A named control/status banner in Results now shares its existing **22px**
  scrollable reader with retained error/notice text, instead of stacking the
  ordinary feedback panel above the evidence. Both messages remain in memory and
  are rendered in full, with error priority and explicit scroll hints. The
  existing **Dismiss feedback** action is pinned beside the control actions;
  Take control/Return remain pinned, and title width reserves all mounted actions.
- Ordinary feedback without a named banner, other screens, task progress and all
  input/approval/authority guards are unchanged. Feedback dismissal remains
  disabled while following and requires deliberate human activation during
  handoff; merely reading/entering Follow never erases a message or moves evidence.
- The exact retained-notice regression passes in **light and dark** with handoff
  evidence **75px** and the same **87px** banner. A second durable regression covers
  following/handoff × success/error/both × both themes × all three supported
  sizes, checking the same 64px floor, pinned actions, full reader End/Home extent,
  preserved messages/evidence identities, and guarded versus explicit dismissal.

### Final scoped verification and source identity

All commands use Rust 1.98.1, `--locked --offline`, the private scratch TMPDIR
and the existing Phase 1 GUI link libraries. Expanded argv, exits and timings
are in `commands.json`; compilation was serial. Counts below are separate scopes,
not additions to historical full-suite totals.

| Scope | Final result / receipt |
| --- | --- |
| GUI `--features ui-tests --lib --bin regressee-desktop agent_` | **17 passed**: 3 library / 14 desktop, including both new regressions and the existing authority/input/IME/status tests; `gui-agent.log`. |
| Ordinary feedback exact regression; `engine_refresh_tests::` | **1 / 2 passed**; `ordinary-feedback.log`, `refresh.log`. |
| Unchanged verifier `remaining_retained_feedback_must_not_collapse_handoff_evidence` | **1 passed**, no-agent **72px** → handoff **75px** with the same notice; `verifier-exact.log`. The private probe imports the unchanged verifier module and only changes the binary name/adds that module declaration; `prepare_probe.py`. This implementor run is not independent acceptance. |
| GUI all-target/all-feature `check` and `clippy`; production desktop `build --features ui`; UI-only test compilation | Exit **0** for all; `check.log`, `clippy.log`, `build.log`, `ui-only.log`. Inherited clippy/future-compatibility warnings remain, with no `-D warnings` claim. |
| Exact ignored `renderer_capture_agent_retained_feedback`, `--all-features` | **1 passed**, **16 final-source PNG/metadata pairs** in one scoped batch at 820×600: following/handoff × saved/error/both × light/dark, plus four combined-feedback End positions; `capture.log`, `captures/feedback-metrics.json`. |
| Scoped rustfmt on four changed Rust files; `git diff --check` | Exit **0**; `commands.json` / `format.log` / `diff.log`. |

The contact sheet (`feedback-contact.png`) and full-size light/dark saved-notice
handoff, combined-feedback reader-end and following-error representatives were
inspected. Every frame retains **75px** of evidence; the default-state banner is
**87px**, the reader **22px**, with pinned actions. End-position frames expose the
final request/error content. PNG hashes match metadata; compiled hashes for all
four changed Rust paths match current source. Frozen stored evidence is asserted
unchanged by the capture test. No code correction or recapture followed this batch.
These are GPUI offscreen proofs, **not** native OS input/accessibility qualification.

Current `crates/gui/src/agent_ux.rs` SHA-256:
`2245e7ccad04b54ead773e180a36e8a651a720212146d12392effdc503bac531`.
The four-file Rust source bundle SHA-256 is
`d5699161525299e2471a155f6a5a13bf79a5cdd852ca74ba9191cccaad48812c`
(SHA-256 of canonical sorted compact JSON path→SHA-256 in `capture-source.json`).
`source-handoff.json` records all **44** current Phase 4 source paths, scoped
changes, binary hashes, unchanged report/witness identities, command/log/capture
hashes and the capture-source match. Only `main.rs`, `agent_ux.rs`,
`ui_agent_capture.rs`, this ledger and new `agent_feedback_tests.rs` changed in this
scoped repair.

The independent **NEEDS FIX** verdict is not overwritten. Return this scoped
repair to the verifier for confirmation; no Phase 4 PASS, native gate PASS or
Phase 5 advancement is claimed. No compilation remains running.

## Phase 5 — skill, production product packages and local qualification

Implementation ongoing / independent Phase 5 verification pending. The final
independent Phase 4 **PASS**, protocol 6, supersedes the historical repair verdicts
above at its explicitly bounded source/Linux IPC/offscreen scope. Its report is
unchanged. Native OS-window/input/IME/accessibility and platform/release gates
remain unqualified. This ledger does not rewrite the plan or claim all gates met.

### Implemented paths and preserved boundaries

- `skills/regressee/SKILL.md` and three `references/` files: portable, tool-neutral
  CLI schema-1/protocol-6 skill. Covers authorized scope/budgets/new baselines and
  existing fixed inventories; actual before/after/difference inspection and bounded
  crops; warnings/coverage/comparability, saved policies/masks, authoritative current
  reviews/full keyset history, selected approval, report privacy, original IDs,
  exact confirmation/retry, renewable exclusive leases and GUI-only opt-in/return.
  Website/evidence instructions are untrusted; no source edits, unrelated data,
  automatic acceptance to clean a report, upload, headless login/session import or
  sandbox bypass. `docs/agent-skill.md` documents manual installation without a
  claim of verified host integration or independent agent exercise.
- `scripts/release-preview.py`: freeze selected sources, build the CLI and GUI in
  **separate exact production Cargo invocations**, recheck selected sources after
  each build and packaging, and bind executable hashes in a build receipt. This
  avoids both all-features leakage and GUI-driven dependency feature unification
  in the CLI. `release-candidate.py` delegates here; its older debug receipts stay
  historical, not retroactively changed. CLI-only and combined desktop products
  are distinct; desktop archives contain `regressee` + `regressee-desktop`.
- `scripts/package-preview.py`: native Linux/Windows x86_64 archives only, no
  standalone GUI EXE, runtime bundle, macOS claim, installer, signing or upload.
  Matching production binary/version/protocol/schema/feature checks, per-product
  target-specific normal/build dependency/license texts, source/build metadata,
  public help/skill and nested/outer checksums. Before/after build binding is
  explicit; separately supplied binaries are truthfully labelled no compile
  attestation. Metadata execution is isolated/deadline-bound but **not an OS
  sandbox for untrusted code**. Local dirty builds require `--allow-dirty` and bind
  exact selected working-tree hashes plus base commit, never claim to be the base.
- `source_release.py`: reviewed skill/public-document/vendor/brand/CI/embedded
  official-payload allowlists, actual custom Cargo crate roots and the explicit
  vendored-GPUI workspace exclusion. Private runtime/data/keys/logs, internal
  review/status/research/design context are omitted without walking private trees.
  The source archive now works against the actual extracted workspace, not merely
  synthetic pre-vendor fixtures. All legal/native audits remain unapproved.
- `.github/workflows/release-preview.yml`: integrated product builds/packages,
  separate CLI dependency/graphics inspection, both-executable ABI/DLL checks,
  source-identity equality before merging platform downloads, selected source
  archives rather than blanket `git archive`. `.gitattributes` freezes LF text
  across Windows/Linux. **No workflow dispatch, publication, signing, commit or
  push was performed.** Existing source/user work was retained.
- Data-free build metadata in `app/src/launch.rs`, `gui/src/launch.rs` and
  `cli/src/main.rs` now names the executable, schema, actual protocol and feature
  sets. GUI metadata includes unified app-layer developer/test flags. Production
  desktop autostart still uses its own known current executable; extracted sibling
  CLI cannot redirect it, no PATH/shell lookup or display initialization is needed
  for engine hosting. No accepted Phase 4 UI paint/input/permission code changed.
- `scripts/cli-consumer-fixture.py` + public exercise guide: fresh installed-like
  copies of checksum-verified packages, an owned three-page loopback HTTP site,
  changed/new-link/failed-navigation modes, exact generated task/path/budget
  instructions, optional explicit managed-runtime generation copy, and public-CLI
  engine/site teardown without deleting evidence or killing unrelated processes.
  It supplies **actual Chromium** work, never installs a synthetic worker. The
  orchestrator can launch a different fresh agent with only TASK.md, the installed
  SKILL/references and public CLI help. That independent exercise is **not yet run**.
- Independent-from-Phase-4 inline design-documenter pass:
  `research/agent-cli-feature-design-20261017.md`. Compared current source and
  final independent offscreen contact sheet to incumbent PRODUCT/DESIGN. No new
  tokens/assets/motion/UI identity and no global context rewrite. Pre-existing
  sidecar/moved-brief/PRODUCT CLI-skeleton drift is recorded, not repaired.

### Environment and exact local checks

All new fixtures, reports, packages, logs and receipts use:
`/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5` (`P` below).
Rust uses `/var/home/olav/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin`
on PATH; TMPDIR is P. GUI builds use Phase 1's private link-libs. Explicit-target
release linking additionally required `P/link-libs/libfreetype.so`, a private
symlink to the host's existing `/usr/lib64/libfreetype.so.6.20.6`; no system library,
package, permission/compositor helper or sandbox was changed. That initial missing
linker-name failure is retained in `build-package.log`, not counted as a pass.

| Exact command / scope | Result / receipt |
| --- | --- |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1 --skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` | **254 passed, 0 failed, 29 ignored, 1 scale case filtered**; nested environment subprocess not double-counted. App library 60/1 ignored, control 6, engine 18/1 ignored, CLI 18. `backend.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --lib --bin regressee-desktop agent_ -- --test-threads=1` | **17 passed** (3 library, 14 desktop), `gui-agent.log`. No full inherited suite or new UI capture loop. |
| Same package/features, `--test engine_client -- --test-threads=1` | **1 passed**, `gui-client.log`. Production IPC facade and saved-data/evidence handling. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Exit **0**, `check-workspace.log`. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Exit **0**, `build-workspace.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui --lib --bin regressee-desktop --no-run` | Exit **0**, `ui-only.log`; compile only. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features` | Exit **0**, `clippy.log`; inherited widget/test/future-compatibility warnings remain. No `-D warnings` claim. |
| `python3 -B -m unittest discover -s scripts -p 'test_*.py'` with private TMPDIR | **66 passed**, `python-tests-delivery.log`: CLI/desktop archive contents/modes/checksums/help/skill/source omissions, missing/malformed/mismatched/mutating binaries and inventories, feature/protocol leakage, path containment, exact production build/source binding, owned fixture failure modes and stale-stop handling. Synthetic packaging tests are not Windows execution. |
| `rustfmt --edition 2024 --check --config skip_children=true crates/app/src/launch.rs crates/gui/src/launch.rs crates/cli/src/main.rs`; `git diff --check` | Exit **0** (`scoped-format.log`). Global `cargo fmt --all -- --check` still fails exactly the inherited setup-progress/timezone/options/renderer-module-order paths (`global-format.log`), left unchanged. |

The unchanged costly persisted-scale/guardian tests and five independently
attributed inherited GUI failures were not rerun or waived. No globally green
tree, clean-host/no-SDK rebuild or native platform test is inferred.

### Actual packaged CLI / managed Chromium evidence (not synthetic)

`P/consumer-smoke` is an installed-like extracted CLI copy, isolated HOME/data/
cache/work/evidence, DISPLAY/Wayland/session-bus unset. No user project/session
was opened. Only the owned loopback site was captured. The existing independently
verified runtime generation (worker 5.0.0) was copied, but the current packaged
CLI correctly returned **UpdateRequired** and discovery **action_required/4**.
Only then was explicit managed `browser setup` used; it succeeded with
**OfficialPinned Node 24.21.0 / worker 5.0.1 / Playwright 1.64.0 /
Chromium 156.0.8078.4**. No distribution override/system browser or sandbox option
was used. The bound worker launches with `chromiumSandbox:true`.

- New project `404a7a9f-c841-48a5-9eef-30290f0d3aa8`: reviewed discovery inventory
  `253f219f-cf71-4310-9426-cbda782cbd2d`, exactly three included pages, one 1440×900
  DPR-1 desktop profile. Actual baseline run
  `a6623074-b9ed-4b57-88ec-e9913e2709b7`: **3/3 captured**, no capture warnings.
  Opened all originals before confirmed baseline save; original baseline
  `f6fd2db8-e064-40cf-99be-5d60ae40067c` retained.
- Existing-baseline changed check `b5a422f7-70aa-4bd1-9557-4468160c9a04`, analysis
  `6c9a9298-21a9-437a-b494-b05fb1bf22cb`: **3/3 captured, exit 1**. Visually
  inspected original before/after/difference evidence. Observed newly added nav
  link on all three pages, plus Details' mint→amber panel and changed text.
  Owned visit log proves **no `/new` visit**: fixed inventory was preserved.
- Failed check `a3e796e6-1168-4466-af4e-2c111e5b0e6f`, analysis
  `8a261d6f-cd5c-47af-b37b-8e70b268e6ee`: **2 captured / 1 CaptureFailed,
  CompletedWithErrors, exit 3**, not clean. `/fail` has its original before image
  and **null after/difference**, not a substitute. Captured targets' bytes match
  the already-inspected changed images; gates/warnings empty, zero ignored pixels.
- Exported and opened **six derived 640×420 crops**, rectangle `0,80,640,420`,
  screenshot pixels, Home/Details × before/after/difference. Exact source hashes,
  original-baseline/run IDs, previews/tickets/output identities are in work/evidence
  JSON. Home's crop excludes the nav change; its original diff was inspected too.
- Recorded an Intentional decision only for inspected Details; full selected
  history was queried. Read report privacy/settings/selection and exported
  `P/consumer-smoke/evidence/report/index.html` + `summary.md` + seven original
  PNG copies, including before-only failed target. It remains a local report with
  the failed check's **original baseline**, not a clean result or an upload.
- After export, exact selected approval accepted **one Details reference**,
  inherited the other two (including failed target's old reference), creating
  baseline `9ce42d49-9102-4911-8f9a-1d0776878e58`. History and exact old-artifact
  query retain original baseline/before bytes. No mask/sensitivity/cleanup shortcut.
- Explicit engine stop and fixture teardown completed. The owned observer sampled
  **205 CLI/engine/Node/Chromium descendant identities**; **0 surviving processes,
  0 zombies** (`owned-processes.json`). Site shutdown is separately checked; no
  cleanup signal, evidence deletion or unrelated-process action was needed.

Initial exercised packaged CLI SHA-256:
`5081437fc4a1fe194c32cdb0a898af38aba64c8f89494664e483a4fbb762118b`.
Its exact dirty source inventory is
`f676193cf72e06d113e5867a82aa92e4a6b7b32f978897bea11395dd53f480a0`,
base `9daf121b960f84506feedefcdf49869327861130`, not a claim of that commit's
unmodified sources. Initial packages and source/build receipts remain in
`P/packages` / `P/extracted`; final separately-built product packages/bindings are
recorded below. Actual browser evidence is distinct from the regression suite's
signed synthetic Python-worker captures and Phase 4 GPUI offscreen evidence.

`P/desktop-package-final/results.json`: **8** actual packaged CLI/desktop
metadata/engine invocations passed against the same private root without display:
desktop start → CLI status/stop, then CLI start → desktop status/stop. Both versions/
protocols/features truthful, HOME empty, runtime absent, bootstrap removed and
owner lock released. This proves engine-only compatible installed hosting, **not
a native window**. An earlier immediate restart raced the asynchronous Stop
acknowledgement and returned Connection refused; retained as a development smoke
failure, not a product completion or a passing restart test. Final smoke waits for
the owned lock to release without deleting it. The fixture teardown initially
refused the normal stale descriptor after a successful stop; its narrow refused/
missing-transport handling and regression are repaired, without accepting version,
permission or busy failures and without changing engine behavior.

### Unfulfilled gates / independent handoff

- A **different fresh real-agent** consumer exercise using only installed skill/
  public CLI/help/images remains required. This implementor smoke is not that
  exercise and no agent-host integration is claimed verified.
- No clean Linux host/container/VM/server image was supplied. Isolated HOME/unset
  DISPLAY on this development machine **does not qualify a clean server** or
  fresh OS-library/font/default-policy compatibility. Managed setup/capture above
  is a real sandboxed development-host result only.
- No authorized native display/socket, Windows host or macOS support was supplied.
  Packaged native desktop follow/handoff/takeover, OS input/IME/accessibility,
  Windows named-pipe ACL/Job lifetime, clean install and hardware/default graphics
  remain unqualified. macOS capture remains blocked, not a package target.
- Local ELF inspection: CLI links only libc/libm/libgcc/loader and requires at
  most GLIBC_2.39; desktop additionally links XCB/XKB and requires GLIBC_2.43 on
  this host. These local desktop packages therefore **do not meet Ubuntu 24.04's
  ABI floor**. The workflow's Ubuntu build/floor check remains required, and was
  not dispatched. No sandbox weakening can substitute for that gate.
- Signing, installer/update/uninstall, complete native/transitive license/source
  audit, legal/security approval and release publication remain open. Source and
  product archive checksums are integrity/provenance records, not authenticity or
  supported release approval. Resume the independent Phase 5 verifier and return
  task-related findings to this original implementor.

### Final packaged-source seal and consumer handoff

Final delivery artifacts are in **`P/packages-delivery`**, extracted at
**`P/installed-delivery`**:

```text
regressee-cli-linux-x86_64.tar.gz
regressee-desktop-linux-x86_64.tar.gz
regressee-source.tar.gz
regressee-source.json
SHA256SUMS-linux-x86_64
```

`package-delivery-source-seal.log` records the actual successful integrated
`python3 -B scripts/release-preview.py --target x86_64-unknown-linux-gnu --product
all --allow-dirty --offline --output P/packages-delivery`. Its **separate** builds
are `cargo build --release --locked -p regressee-cli --bin regressee --target
x86_64-unknown-linux-gnu --offline`, then the corresponding GUI command with
`-p regressee-gui --features ui --bin regressee-desktop`. All executable metadata,
inner and outer checksums were checked after extraction. Final inventories:
**CLI 219** third-party Cargo packages / **0 missing license texts**; **desktop
558** / **32 missing texts**. All are still **unreviewed**, native inventory
incomplete, no legal approval. CLI inventory and ELF contain no GPUI/GUI/graphics
closure. `PKG_CONFIG=/bin/false LIBRARY_PATH= cargo build --locked --offline
--release -p regressee-cli --bin regressee --target x86_64-unknown-linux-gnu`
also passed with the cached target (`cli-only-build.log`), not a fresh-host proof.

Exact final source inventory: **582 files**, SHA-256
`bdb0d4f7a5c0cdd43ebd2fe0a742929ef75eeed10683be37812a35d57bf88a13`.
The independent Linux source helper against the actual tree returns that same
identity (`source-candidate-sealed.log`, `P/source-candidate-sealed`); its archive
uses a separately explicit epoch, not the preview archive's commit epoch. Both
archives exclude private data/internal reviews and include all four skill files.
The final Python suite is again **66 passed** (`python-sealed.log`).

Final production executable SHA-256:

- CLI: `66d835ae134d3485db84500215ccab3968fdad8cd4d149a0e3ce94b5bad2a36a`
- Desktop: `82b3009289f091c08396e093c389eb8494a38e0d2593b27f04e6c5c9fca43da9`

The separately built CLI has different bytes from the earlier combined-invocation
smoke, so the **entire real-browser workflow was rerun**, not inferred, in fresh
`P/consumer-final`. Its previously provisioned managed runtime copy verifies
**Ready**, with the exact 5.0.1 identity above; no repeat setup/download was needed.
The final source seal after this exercise changed **only the workflow's Python
test invocations** from the exercised source inventory
`24ed35fbc1d6c735d757fe9b66c50541eed2466d525b24af3952c287272890ba`.
Final binary and skill bytes are unchanged, explicitly checked. Both source
snapshots/receipts are retained rather than relabelling earlier source evidence.

Final actual-browser identities/results:

| Evidence | Identity / outcome |
| --- | --- |
| Project | `0cd99d40-1fa8-4554-90aa-3913f2aad72c` |
| New baseline run | `86f088d9-805f-4161-8c47-9c3b36c9f712`, **3/3, exit 0** |
| Original baseline | `d4818a17-eaf4-4246-b5ec-f7f2d8df8bb6` |
| Changed check / analysis | `03d34092-b25a-4829-9d83-9f302d4b684e` / `e4dbabe4-cb99-42e9-8579-d704db092c0d`, **3/3, exit 1** |
| Failed check / analysis | `296a561f-4613-41b5-ae6b-9e4fd1913db8` / `08f5bed2-4450-4cbc-8ee3-eb4d3cf2373b`, **2/3, exit 3**, explicit before-only failed target |
| Selected Details update | `4aadcc8b-025f-4861-bae7-a314e9a6dc82`, **one changed reference / two inherited**, prior history/reference preserved |
| Exact report | `P/consumer-final/evidence/report/index.html`, `summary.md`, **seven unchanged PNG copies**; all three targets, original baseline and failed outcome retained |

All final original/difference PNGs and **six derived crops** match the actual PNG
bytes opened in the earlier inspection, verified by full hashes, dimensions/DPR
and exact pane membership. Crop/report previews, confirmations, source hashes,
rectangle `0,80,640,420` in screenshot pixels, exact output paths/hashes and
observations are aggregated in **`P/live-qualification.json`**. Final visit log
again proves no `/new` visit. History/read-after-selected-acceptance preserves the
failed check's original baseline and before bytes. No synthetic worker supplied
these images. The final owned observer sampled **190 process identities**, with
**0 surviving processes / 0 zombies** after explicit stop; both fixture sites are
gone and owned storage locks released. This is separate from the earlier 205-count
run, not additional unique coverage. No forcible cleanup or evidence deletion.

`P/desktop-final-executables/results.json` repeats **8** no-display compatible
metadata/engine invocations using these final executable bytes, empty HOME, no
runtime, removed bootstrap and released owner lock; CLI ELF remains graphics-free.
`P/packaged-control/results.json` records **5** final packaged named-session/start,
renew, no-GUI handoff (**exit 4, agent paused**), end and engine-stop invocations.
Neither is native GUI input/follow/handoff qualification.

For the **different fresh agent**, the orchestrator can run this exact setup
(root must not already exist), then supply only its generated TASK.md,
exercise.json, installed SKILL/references and public help/images to that agent:

```sh
python3 -B scripts/cli-consumer-fixture.py setup \
  --root /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/fresh-agent-consumer \
  --package-dir /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/installed-delivery/regressee-cli-linux-x86_64 \
  --runtime-cache /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/consumer-final/cache/product-runtime
```

The orchestrator uses the same helper's `set-mode --mode changed`, then `failed`,
at the agent's requests, and `teardown` after the agent ends authority/stops work.
The fresh root is **not created yet**, no fixture process is left running for it,
and **no independent real-agent skill exercise is claimed**. All unfulfilled
clean-server/native/Windows/legal/release gates listed above remain open. No
Phase 5 independent PASS, commit, push, workflow dispatch or publication.

Clean-host availability was checked without reading a user's container store:
`P/clean-host-probe/result.json` records Podman **5.8.4** `info` with explicitly
isolated HOME/config/runtime/storage roots and vfs storage, exit **0**. It reports
rootless + seccomp + SELinux, **0 images / 0 containers**. Docker and QEMU are not
installed. This is **not a Podman technical-refusal claim**: no reviewed clean OS
image/VM/native server was supplied, no external OS image was pulled, and no
container/browser was started by that probe. A container on this development
kernel would still need its own bounded default-sandbox qualification rather than
being silently called a clean server. The only real browser qualification above
remains the explicitly labelled isolated development-host public-fixture run.
Podman's private probe pause namespace was explicitly retired using `system
migrate` against that same **empty private store**; `clean-host-probe/cleanup.json`
verifies its owned PID exited. No user container store/containers were inspected
or stopped, and no probe process remains.

### Phase 5 independent NEEDS FIX — implementor repair handoff

The preceding Phase 5 receipts are **historical, pre-review evidence**. The
independent verifier and fresh consumer subsequently returned **NEEDS FIX**, not
PASS: `agent-cli-phase5-review.md` and `agent-cli-skill-exercise.md` remain
unchanged. In particular, earlier 1440×900 captures are not long-page proof, and
the earlier HTML report lacked its own privacy inventory and mislabelled derived
differences. Those original reports, packages, consumer root and receipts were
not rewritten to look repaired. This section supersedes the earlier “not yet
run” fresh-consumer status, **not** either independent verdict.

New repair root (`R` below):
`/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/repair`.

#### Repairs and durable regressions

- **Verifier 1:** SKILL frontmatter is a standard YAML 1.2 **JSON mapping**, not an
  ambiguous plain description with an unquoted colon. `skill_frontmatter.py`
  completely parses that deliberately restricted subset with stdlib JSON and
  validates name/description/body, duplicate keys, invalid scalars, UTF-8/control
  characters and bounded input/nesting. Source/package validation rejects invalid
  frontmatter before archive output. Tests parse the distributed skill too; no
  new unpinned production/parser dependency. The system's existing **PyYAML 6.0.3**
  independently parsed source, both extracted products and source-archive copies
  (`R/yaml-and-source-validation.json`). This is standard-parser compatibility,
  **not a claim that an agent host installed/loaded the skill**.
- **Verifier 2:** distributed preview notes now link to included public build and
  support guides, explicitly excluding internal research/reviews/private evidence.
  The archive regression inspects the actual notes and all their local Markdown
  link destinations. Both new independent reports are explicitly classified as
  internal by the source selector; neither is shipped to repair a missing link.
- **Consumer 1:** a single final persistence path runs after **both returned
  outcomes and errors**. Exit-3 captured evidence saves raw `data.value`, including
  incomplete/paused/cancelled capture state rather than a fabricated pass. Other
  failures, completion uncertainty and interrupted waits save the full schema-1
  diagnostic envelope, retaining request/job/retry/continuation information.
  Detached admission saves its receipt instead of null. Valid file-write failures
  preserve original data/code/exit under `command_outcome` and emit a visible
  warning with exit 6; invalid/existing destinations still reject before admission
  with exit 2, never overwrite or retry mutation. Regressions cover all exit
  families 0–7/130, scalar/detached data and write collisions; real subprocess
  checks cover incomplete check + same-job show/wait/retry, invalid/parser/product/
  conflict/action-required outcomes and interrupted wait. Public CLI/skill docs
  describe these distinctions rather than hiding nonzero diagnostics.
- **Consumer 2:** static HTML and Markdown both retain a prominent **Before sharing
  / LOCAL ONLY** privacy inventory, full-URL/query/local-path disclosure and explicit
  **Ignored areas are not redaction** warning. Original before/after screenshot
  links and derived difference links have distinct accurate labels. Crops retain
  pane-specific derived-source provenance, never “original unaltered” for the crop
  itself; they are exported separately, not silently included in reports. HTML
  escapes all evidence in its settings/details; Markdown's untrusted URL/note/
  warning text remains escaped inside inert preformatted details. Regression
  inputs include closing-pre/script/remote-image injection strings; image copies
  remain byte-identical to source artifacts. The document has explicit head/body,
  relative local PNG links, no scripts/forms/remote resources, and restrictive CSP.
- **Consumer 3:** fixture **version 2** retains the same **three** initial pages and
  fixed-inventory `/new` exclusion. Details now has a real ≥6,000-CSS-pixel content
  section, retained landmarks and a separate changed marker at section y=4800;
  failed mode still genuinely drops `/fail` capture. Tests check its long content,
  lower marker, initial/changed link counts and failed navigation. Generated task
  regression requires top/lower crops, incomplete `--save` + job recovery, standard
  frontmatter loading/parsing, public-only docs, privacy/derived labels, narrow
  Details-only approval and truthful render limits.
- **Consumer 4:** the disconnected desktop-browser preview is an environmental
  limit, not an invitation to bypass permissions or add production features.
  Reports are plain local static files; HTML structure, Markdown and actual PNGs
  were inspected, but **rendered HTML layout remains unqualified**. An independent
  authorized read-only browser can confirm that separately when available.

The red regressions before fixes are retained in `R/rust-red.log` and
`R/python-red.log`; intermediate compile/assertion mistakes in new tests are also
retained, not called passes. No accepted Phase 4 GUI paint/input code was changed.

#### Exact repair checks

Rust PATH is the same 1.98.1 toolchain above; TMPDIR is `R`. Offline GUI builds
reuse only the already recorded private Phase 1 / Phase 5 linker-name directories.

| Exact command / scope | Observed result / receipt |
| --- | --- |
| Same backend command as the preceding 254-test run, with the same costly scale case skipped | **257 passed / 0 failed / 29 ignored / 1 scale case filtered**, excluding the nested environment subprocess duplicate; `R/backend-complete.log`. Includes the new unit and initial two repair workflows. |
| `cargo test --locked --offline -p regressee-cli -- --test-threads=1` after the final additional diagnostics/Markdown regression | **22 passed** (2 unit, 1 lifecycle, 19 workflows); `R/cli-final-2.log`. Synthetic signed-worker IPC tests, not Chromium qualification. Do not add this overlapping count to the preceding backend run. |
| `python3 -B -m unittest discover -s scripts -p 'test_*.py'` | **71 passed**; `R/python-final.log`. Source/package/frontmatter/public-link/fixture/task regressions, including archive-format Windows fixtures, not Windows execution. |
| `cargo check --locked --offline --workspace --all-targets --all-features`; corresponding `cargo clippy` command | Both exit **0**; `R/workspace-check.log`, `R/workspace-clippy.log`. Existing warnings remain; no strict-warning/global-format claim. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Exit **0**; `R/workspace-build.log`. Production archives below use their own separate builds, not this unified development build. |
| `cargo test --locked --offline -p regressee-gui --features ui --lib --bin regressee-desktop --no-run` | Exit **0**, compile only; `R/ui-only.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**; `R/gui-client.log`. No full inherited GUI rerun. |
| `rustfmt --edition 2024 --check --config skip_children=true crates/cli/src/main.rs crates/app/src/terminal.rs crates/cli/tests/workflows.rs crates/cli/tests/support/phase5_repairs.rs`; `git diff --check` | Both exit **0**; `R/scoped-format-final.log`, `R/diff-check-final.log`. Inherited formatting paths and five GUI failures remain unwaived. |
| `/usr/bin/python3 -B R/validate-live.py` (R expanded) | Static HTML structure/privacy/link/CSP, escaped JSON == preview, 9/7 report PNG hashes, all six crop source-pixel rectangles, incomplete-save/recovery equality, fixed inventory and teardown passed; `R/live-validation.json`. Uses existing Pillow for evidence validation, not a new production dependency. |

#### Version-2 actual-browser evidence — implementor only

`R/implementor-consumer-v2` is a **new** owned root, separate from the original
`P/fresh-agent-consumer`. It used the rebuilt extracted CLI and an explicitly
copied existing runtime: OfficialPinned Node 24.21.0, worker 5.0.1, Playwright
1.64.0, Chromium 156.0.8078.4. Actual managed Chromium, `chromiumSandbox:true`;
no fresh download, override, system browser, external website or sandbox/permission
change. Private HOME/data/cache and unset desktop/session-bus environment remain.

| Evidence | New identity / actual outcome |
| --- | --- |
| Project | `129e47d1-fb44-481c-8927-29fb5f68636a` |
| Baseline run | `db0c59c7-4599-4920-9f42-7716cc2f0cdc`, **3/3, exit 0** |
| Original baseline | `aba580b2-ed36-4d60-9a87-aca8aed58027` |
| Changed run / analysis | `d7990b27-2450-44d7-b871-a40a8b6ff447` / `14022257-17c3-4301-a06a-b83cb2003a7b`, **3/3, exit 1** |
| Selected Details update | `e149a84b-3683-4644-937b-376af692bfb5`, **one selected / two inherited**, original history retained |
| Failed run / analysis | `e1780b02-7f4e-434a-b456-8fab65b16d0c` / `2445db24-4cf5-4009-8c8d-227eb1f79e69`, **2/3, CompletedWithErrors, exit 3** |
| Reports | `R/implementor-consumer-v2/evidence/{changed-report,failed-report}/index.html` and `summary.md`; **9** / **7** unchanged artifact copies, explicit missing panes on failed target |

Details before/after/difference are **1440×6617, DPR 1**, FullPage, not a 900px
viewport stretched into a long image. Opened original short-page baseline images
and eight actual exported crops: two baseline crops plus **six changed crops**,
before/after/difference at `0,0,720,560` and `0,5000,720,1000` screenshot pixels.
The top crops show the mint→amber panel, text and new link; lower crops show the
independent mint→amber marker and “Below-fold baseline”→“revised” text. Difference
images mark those actual changes. These are fixture observations, not production
bug causality, maximum-height/tile-boundary or exhaustive long-page qualification.

The failed check's explicit raw save **exists and equals stdout data.value**.
Subsequent jobs show/wait/retry each used a new output path, returned exit 3 and
the **same job/run/analysis/result**, not a new capture. `/fail` retains before
only, after/difference null. No `/new` visit, mask/sensitivity change, whole-site
acceptance or evidence deletion. Only inspected/authorized Details was reviewed
and accepted after saving the changed report. History retained the original
baseline. Report privacy and prepared confirmation payloads were inspected before
export; untrusted page/note instructions were not followed.

`R/live-validation.json` binds exact source artifact/crop/report paths/hashes,
dimensions, screenshot coordinates and static checks. `evidence/commands.jsonl`
and per-command stdout/stderr retain the public invocation receipts. Explicit
engine stop + fixture teardown completed; `R/owned-processes.json` sampled
**194** owned PID/start-tick identities with **0 survivors / 0 zombies**. Site
refuses connections and owned engine lock is released. No broad kill or lock/data
deletion. `R/packaged-engine/results.json` separately records **8** no-display
CLI/desktop metadata/start/status/stop invocations, compatible hosting and lock
release; this is not a native window/follow/input test.

#### New immutable package/source handoff and fresh consumer instructions

Final repaired packages: **`R/packages-delivery`**. Extracted, checksum-verified
copies: **`R/installed-delivery`**. Earlier `R/packages` / `R/installed` are the
live-exercised repair snapshot, retained separately. Final build adds only the
generated task's explicit frontmatter/doc-check instructions and stronger test
assertions; **no capture HTML/runtime/product source/skill changed** after the live
smoke. Final CLI/desktop/skill bytes are checked identical to the live snapshot;
`R/source-handoff.json` binds both selected source inventories and the exact diff.
No source identity is retroactively assigned to earlier screenshots.

Final repaired selected source: **585 files**, SHA-256
`5bd243894173c8701e84af59a5dbd239933101ebbdfd1d9f9766862736c72ae4`.
Executable SHA-256:

- CLI: `d70155684492ad7cff9d87fc9318bf91cf94956df02621e3038b5e69a88d317d`
- Desktop: `d679f044fbfc4e514f76d4146ac7d1d53f9c84eefbf1ec95960f9343eb628775`
- Skill: `58fa2622a4339a5a47f86a66e02da09fd183d537099e399e5be870216e485896`

| Repaired delivery file | SHA-256 |
| --- | --- |
| CLI archive | `22b3c3c4b98fc646b286b4f7c0fe467d41c8d19608a0279eeaea8b6df84620ca` |
| Desktop archive | `5876faac868a394a2b62e3ac5aca7e3e9b9d8b69bb666889bb191f417523d446` |
| Source archive | `5f47b68529bc461196e2335be5a219f175107f87ac8eb10882b550f5deede33f` |
| Source manifest | `9d68f31f59b865d6ec56cd45945e9bf17cf30a34819472bf2f373e8361fc9091` |
| Outer checksum file | `298215d96db88dec6cd65ed1f09c31c408378f5b63192f6263c9c62502363e9e` |

Final package names remain `regressee-cli-linux-x86_64.tar.gz`,
`regressee-desktop-linux-x86_64.tar.gz`, `regressee-source.tar.gz`,
`regressee-source.json`, `SHA256SUMS-linux-x86_64`. Integrated separate production
build/source-freeze command and hashes are in `R/build-packages-delivery.log` and
`R/source-handoff.json`; all remain dirty local unsigned preview bytes, based on
`9daf121b960f84506feedefcdf49869327861130`, not that commit's unmodified sources.

The **new independent re-exercise root is NOT created**. The orchestrator should
run exactly:

```sh
python3 -B scripts/cli-consumer-fixture.py setup \
  --root /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/fresh-agent-consumer-v2 \
  --package-dir /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/repair/installed-delivery/regressee-cli-linux-x86_64 \
  --runtime-cache /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/consumer-final/cache/product-runtime
```

Supply only its generated `TASK.md`, `exercise.json`, installed skill/references,
public help/docs and actual images to the different fresh consumer. Do **not**
give it implementor smoke code/results as independent proof. At its requests:

```sh
python3 -B scripts/cli-consumer-fixture.py set-mode \
  --root /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/fresh-agent-consumer-v2 --mode changed
python3 -B scripts/cli-consumer-fixture.py set-mode \
  --root /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/fresh-agent-consumer-v2 --mode failed
# After the consumer ends its lease/stops work:
python3 -B scripts/cli-consumer-fixture.py teardown \
  --root /var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase5/fresh-agent-consumer-v2
```

**Both independent re-verdicts are pending**; do not infer PASS from these repairs.
The original independent evidence/reports are preserved (353 frozen file bindings
checked unchanged). Clean-server/default-library/font/security-policy testing,
actual native GUI follow/handoff/input/IME/accessibility, Windows IPC/ACL/Job and
native platform qualification remain open. CLI remains graphics-free / GLIBC≤2.39;
the local desktop still requires **GLIBC 2.43**, failing Ubuntu 24.04's floor.
No workflow dispatched. CLI license texts remain 0 missing; desktop **32** missing;
all legal/native completeness/signing/install/update/uninstall/security/release
approvals remain unqualified. Rendered HTML remains unqualified. No scale/guardian
rerun, unrelated-user access, subdelegation, commit, push, upload or publication.
