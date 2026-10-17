# Agent CLI implementation status

Phase 1 implementation is complete and ready for independent review, with the
inherited validation failures below explicitly retained. Independent verification
has **not** yet been supplied by the orchestrator. This record does not supersede the confirmed
requirements in [the plan](agent-cli-plan.md). `ACTIVE_WORK.md` is historical.

## Phase 1 scope and inventory

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

- Independent reviewer must confirm the extraction, package/feature boundaries,
  moved tests/assets and baseline-failure attribution. No known extraction-caused
  product or GUI-test failure remains.
- The inherited five UI failures and existing workspace formatting/strict-clippy
  warnings mean the entire tree is **not claimed fully green**. Resolve/waive
  those separately without changing UI behavior under this extraction task.
- Orchestrator must supply a different agent's independent review; repair its
  findings before Phase 1 acceptance. Do not start Phases 2–5 here.
