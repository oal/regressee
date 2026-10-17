# Agent CLI Phase 1 — independent verification

**Verdict: PASS for the Phase 1 extraction at `9daf121`.** No
extraction-attributable repair finding was identified. This is **not** an
all-tests-green, native-desktop, clean-server, or release qualification result.
The five inherited GUI failures and existing formatting/lint debt remain open.

Date: 2026-10-17. Independent verifier, not the implementor. Reviewed
`agent-cli-plan.md` and `agent-cli-status.md` in full and retained the historical
native/sandbox/scale limits in `ACTIVE_WORK.md`. No production source or status
ledger was edited; no commit, push, CI dispatch, publishing, user-data access,
internet capture, native launch, or sandbox change was performed. Phase 2 was
not implemented or treated as a Phase 1 requirement.

## Scope and source findings

- **Dependency boundary:** `crates/app/Cargo.toml:7–25` has no UI features or
  GPUI/image dependency; `crates/cli/Cargo.toml:7–13` targets the independent
  `regressee` skeleton. Independent normal/build graph traversal found **244**
  packages reachable from CLI and **243** from app, neither including `gpui` or
  `regressee-gui`. The app's all-features normal/build tree also excludes them.
  A workspace lockfile containing GPUI is not a CLI dependency violation.
- **Complete native move:** `crates/gui` owns the native executable, sibling
  shell/UX modules, widgets, viewer/cache/image conversion, activity/task
  presentation, recovery navigation, native fixture, renderer/inspection
  harnesses, and timezone asset. The historical spike/UI prototypes remain
  separate and now obtain controls from GUI, not app. Native feature gates and
  production metadata are preserved. Shared fixtures remain under app rather
  than being copied and allowed to diverge.
- **Preservation:** independently reconciled all **188** pre-move inventory
  paths: **zero missing**. All **23** inventoried files absent from the previous
  committed tree `b879d0e` remain; **18** are byte-identical. The other five
  contain relocation imports, the native-test gate, or the separately documented
  concurrent search/focus repair. The commit's large UI diff against `b879d0e`
  must not be mistaken for extraction-authored UI redesign.
- **Baseline identity:** independently read the preserved ELF, located source
  blobs, and recalculated all **79** claimed SHA-256 matches against the
  pre-move inventory; **56** remain byte-identical in current source. Inspected
  the remaining source diffs, including the entire main module. The queue focus
  padding/constant changes are documented as concurrent work in
  `research/queue-focus-clipping-20261017.md`, not attributed to extraction.
  The old desktop test list has **292** names, all still present among the
  current **295**; the additional names are two inspection-feature tests and
  the concurrent opt-in queue-focus capture.
- **Actor adapter:** reviewed `crates/app/src/jobs.rs:228–559` together with
  `crates/gui/src/jobs.rs`. One storage owner, capacities 4/4, command-before-viewer
  scheduling, latest-intent replacement, generation-guarded publication,
  silent speculative success/failure, before-work cache invalidation,
  cancellation, terminal backpressure and shutdown behavior are preserved.
  Concrete native work/reply/image types no longer live in app. The generic
  extension is an in-process implementation boundary, **not** the future IPC
  protocol. Store opening/recovery still occurs in the existing actor as before;
  multi-client ownership/recovery coordination remains Phase 2.
- **Task/recovery/image extraction:** frozen capture status remains in app;
  task policy, activity labels and navigation destinations live in GUI. Recovery
  destination mappings and existing task capabilities are preserved. Verified
  bounded tile validation/decoding remains application code and GPUI BGRA
  conversion is GUI-only; a coordinate-coded pixel/seam test passed.
  Appearance preferences retain their separate owner, started by the desktop
  wrapper, not the CLI skeleton.
- **Entrypoints/tooling:** checked active README/first-run/build/CI/release
  instructions, workflows, packaging targets, artifact names, feature flags and
  current evidence collectors. Remaining old app/UI references in frozen
  research/receipts are historical, not current launch instructions. Packaging
  tests cover the renamed Linux and Windows preview payloads. Phase 5 packages
  containing both executables are explicitly not claimed delivered here.

## Independent commands and results

Raw proof, scripts and logs are retained at:

```text
/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase1/independent-review
```

For ordinary shared-target commands:

```sh
export PATH=/var/home/olav/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH
export E=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase1/independent-review
export TMPDIR="$E"
export LIBRARY_PATH="${E%/*}/link-libs"
```

These are verifier environment settings, not product configuration. Heavy
shared-target compilation was serial. `checks.json`, `audit-summary.json`,
`cli-proof.json`, `failure-pairs.json` and `source-proof.json` retain exact
arguments/counts. Counts below do not add the nested environment-test subprocess
twice and do not imply distinct coverage when suites overlap.

| Command | Independent result |
| --- | --- |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Exit 0. Linux all-target/all-feature API check. |
| `cargo build --locked --offline --workspace --features regressee-gui/ui,regressee-spike/ui --bins` | Exit 0; includes native desktop, CLI and retained developer/prototype binaries. Host GUI link uses only the supplied private linker symlinks. |
| `cargo test --locked --offline -p regressee-gui --features ui --lib --bin regressee-desktop --no-run` | Exit 0; production-UI feature test compilation, not execution. |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1 --skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` | **203 passed, 0 failed, 28 ignored, 1 explicitly filtered scale test**; 21.733 s including compilation. App library 48/0/1; core 56/0/0; runtime library 34/0/1 plus distribution 7/0/0; storage 40/0/2; CLI 3/0/0. Includes nine analysis/review and two partial-baseline tests. |
| `cargo test --locked --offline -p regressee-gui --all-features --lib -- --test-threads=1` | **97 passed, 0 failed, 1 ignored**. Includes four actual adapter/mailbox tests, four task/status tests, native widget simulations and viewer-cache/history/integrity tests. |
| `cargo test --locked --offline -p regressee-gui --all-features --bin regressee-desktop recovery_ux_tests:: -- --test-threads=1` | **3 passed, 0 failed, 1 ignored, 291 filtered**. |
| Same desktop command with filter `phase2_viewer_tests::` | **1 passed, 0 failed, 0 ignored, 294 filtered**. |
| Same desktop command with filter `task_ux::tests::` | **11 passed, 0 failed, 2 ignored, 282 filtered**. |
| Same desktop command with exact filter `tests::phase5_saved_pixels_seams_actual_layout_pan_and_mask_actions -- --exact --test-threads=1` | **1 passed, 0 failed, 0 ignored, 294 filtered**; actual decoded BGRA bytes and tile seams checked, no native window. |
| `python3 -B -m unittest discover -s scripts -p 'test_*.py'` | **59 passed**, exit 0. |
| `bash -n scripts/build-ui.sh`; `git diff --check` | Both exit 0. |
| `rustfmt --edition 2024 --check --config skip_children=true` on app jobs/images/task/recovery/launch/lib, CLI main, GUI jobs/viewer_images/task/recovery/activity/launch/lib | Exit 0. Output: `fmt-extraction.log` (empty, no formatting errors). |
| `cargo fmt --all -- --check` | **Exit 1**: existing moved formatting in `gui/src/ui/setup_progress.rs:101`, `ui/timezone_select.rs:428`, `options_ux.rs:302`, and renderer module order at `ui_renderer_capture.rs:46`. Not silently fixed or called green. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features` | Exit 0 **with warnings** in moved UI/test code. This is not a `-D warnings` PASS; strict-clippy CI remains qualified. |

### Fresh, independent CLI-only proof

`python3 -B "$E/cli_proof.py"` created its own previously nonexistent
`fresh-cli-target`, not the implementor's target or shared `target`. It removed
GUI/display/session and native include/link environment variables, set
`PKG_CONFIG=/bin/false`, and did **not** use `link-libs`:

```sh
cargo build --locked --offline -p regressee-cli --bin regressee
cargo test --locked --offline -p regressee-cli
cargo tree --locked --offline -p regressee-cli --edges normal,build --prefix none --format '{p}'
cargo tree --locked --offline -p regressee-app --all-features --edges normal,build --prefix none --format '{p}'
cargo metadata --locked --offline --format-version 1 --filter-platform x86_64-unknown-linux-gnu
```

All exited 0; build took **23.625 s**, CLI tests **3 passed**. This proves the
selected build does not need GPUI development prerequisites, not that browser
capture has been qualified on a clean server. No downloads were attempted.

With a new isolated HOME/XDG tree, no display, and a deliberately nonexistent
session bus, ran `[]`, `--help`, `--version`, `--build-info`, `capture`,
`engine serve`, `projects list`, and `--data-dir <isolated-path>`. Exact exits:
**0/0/0/0/2/2/2/2**. Success wrote only stdout; rejection wrote only stderr.
All metadata feature flags were false; **zero files** were created in the
isolated home. ELF dependencies: only `libgcc_s.so.1`, `libc.so.6`, and
`ld-linux-x86-64.so.2`—no graphics or C++ runtime dependency.

Also independently ran the production `target/debug/regressee-desktop
--build-info` with isolated HOME/XDG/no display/nonexistent session bus: exit 0,
only UI enabled, no developer/test/inspection/capture features, **zero files**.
The normal startup body was compared with the recovered pre-extraction source;
the native application itself was not launched.

## Five inherited failures — independently confirmed, not extraction findings

For every test below, ran both executables with `TEST --exact --test-threads=1`:

```text
target/debug/deps/regressee_desktop-4c924631ab205bce
target/debug/deps/regressee-b2238189f1b8bd46
```

The current target was selected through Cargo's `--no-run
--message-format=json` artifact, not guessed from filenames. All **10** test
invocations exited **101**, each with **0 passed / 1 failed**. Each pair has
identical panic message, assertion and reported values. Baseline SHA-256 was
independently checked as
`ae053aa1bc6d434d05fde4df8fd7e2c9b4234621436e8766305c586a4f1d83a9`.
The 79 source matches include the full main module, affected tests and relevant
workflow/history/controls, supporting attribution to the actual pre-move work
rather than an unrelated older binary.

| Exact test | Current failure location and unchanged symptom |
| --- | --- |
| `changed_results_tests::changed_results_first_analysis_and_recorded_checks_default_without_resetting_refresh` | `crates/gui/src/changed_results_tests.rs:218`: filter value **0**, expected **1**. |
| `disclosure_tests::all_inline_disclosures_attach_content_restore_focus_and_do_not_mutate` | `crates/gui/src/disclosure_tests.rs:132`: `search: enabled disclosure`. |
| `loading_feedback_tests::loading_feedback_has_one_owner_and_keeps_its_icon_and_copy_visible` | Containment helper `crates/gui/src/phase1_setup_tests.rs:8`: copy `(453,213; 318×120)` outside owner `(429,189; 366×136)`. |
| `tests::project_selection_actions_and_long_names_fit_supported_window_sizes` | `crates/gui/src/main.rs:9247`: `change.right() <= working_area.right() - px(24.)` fails. |
| `workflow::tests::review_first_viewport_contains_complete_rows_and_bounded_images` | `crates/gui/src/workflow.rs:2728`: at width 820, row `(25,309; 198×84)` is not fully contained in queue `(24,188; 200×186)`. |

Their repair/waiver belongs to separate UI work, not a demand to redesign the
interface during extraction. They are **not** waived or relabeled passing by
this report. The implementor's preserved full desktop run reports **228 passed,
5 failed, 62 ignored**; I reviewed that evidence and independently reran the five
failure pairs and the focused suites above, **not** that entire desktop suite.

## Remaining qualification

- No extraction-specific production fix is requested.
- Full-tree tests/format/strict-warning CI are not claimed green. Existing five
  GUI failures and formatting/lint debt remain visible.
- The persisted-scale test was explicitly skipped; its prior 846-second evidence
  was not re-created. No opted-in live/native/browser/download/keychain/guardian
  qualification was repeated. Deliberate ignored tests remain ignored.
- Linux host build/simulated GPUI tests do not establish OS presentation/input,
  accessibility, graphics reliability, Windows/macOS, Ubuntu ABI, clean browser
  setup/sandbox support, packaging/legal acceptance, or release readiness.
- CLI has no product commands, shared engine or control sessions yet. Shared
  engine ownership/recovery and later workflow/follow/skill/package gates remain
  future phases, not completed capability.

Only this report is added to the repository. Isolated proof scripts and raw logs
remain under the approved scratch root; the implementation status ledger is
unchanged.
