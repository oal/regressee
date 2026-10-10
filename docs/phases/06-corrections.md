# Phase 06 bounded corrections — COMPLETED LOCAL REPAIR / independent recheck pending

Started 2026-10-10. This is a new implementation checkpoint, not independent
verification and not authorization for Phase 07 or external qualification.

Unique evidence root: `/tmp/opencode/regressee-p6-corrections-20261010-8a67c1e4`.

## Planned / completed stages

- [x] Reidentify running checks/processes; inspect resumed verification, decisions,
  storage/analysis PNG gates, strict schema, and preserved recipient diagnostics.
- [x] Repair P2 bounded complete PNG validation and add maintained atomicity and
  valid-original regressions.
- [x] Repair P3 nested coverage unknown-field refusal and roundtrip regressions.
- [x] One focused recipient diagnosis/correction/confirmation with bounded stage
  evidence; otherwise record the exact remaining blocker without a fake pass.
- [x] Settle serial locked fmt/build/clippy and focused Rust/worker/product checks.
- [x] Audit owned processes/scratch, preservation/privacy, and update Phase 06
  status/checklist/index with independent reverification explicitly pending.

Every settled stage is checkpointed here, including failures and open limits. Historical
verifier reports and the finished source-release handoff will not be edited.

## Checkpoint 1 — diagnosis settled

Initial process identification found no active Cargo/worker/product suites.
Read decisions 0005/0006, resumed verifier and exact probe, storage ingestion/
import/export/activation/restore and core validator, recipient source and preserved
attempt logs. Historical attempts contain no useful per-stage worker events;
the attempted verifier instrumentation never compiled. No wholesale suite rerun.

The maintained `Site` accepts and reads HTTP connections serially. A fresh bounded
test with four speculative idle sockets and a ready GET **reproduced request
starvation** (800 ms read deadline, `Resource temporarily unavailable`). This is
a real owned fixture defect, not proof that it alone explains every historical
timeout. `fixture-diagnosis/command.log` preserves the Cargo/test failure (13.60 s
build, 0.89 s test). The harness cut off the wrapper's response; its own result
JSON is absent, so no exit code is invented. A fresh `/proc` reidentification
found zero surviving wrapper/Cargo/rustc/product processes. All diagnostic work
is settled. Subsequent wrappers will use tool timeout 0 and their own deadlines.

Implementation in progress: shared strict PNG prevalidation (no new inflater),
nested coverage refusal, selected/inherited publication gate, bounded concurrent
fixture handlers, real separate recipient headed Open/explicit-ready flow with a
self-establishing synthetic account, persisted stage deadlines. Confirmation
has not run. Recipient human login is NOT claimed.

## Checkpoint 2 — focused storage and fixture confirmation settled

- `focused-storage-d/{command.log,result.json}`: **3 focused maintained storage
  regressions PASS**, exit 0, 2.024 s, no deadline or forced children, zero children.
  Exact hostile ZIPs refuse `zlib excess decoded bytes` and `unknown field
  \`script\`` on inspection/import. Generated excess/short/trailing/concatenated/
  Adler/IEND/CRC cases agree with core validation and refuse ingestion before CAS.
  All-table and exact retained-file snapshots stay equal on refusal. A deliberately
  manufactured pre-fix hash-consistent local run cannot activate/restore/export;
  a valid active baseline stays unchanged. Actual Chromium originals still ingest
  and round-trip byte-for-byte with original hashes and capture dates intact.
- `focused-storage-{a,b,c}` preserve test-helper failures (missing qualified
  encoder import, duplicate fixed targets, synthetic timestamp ordering). These
  were corrected without relaxing refusal or preservation assertions. Not passes.
- `fixture-confirmation/{command.log,result.json}`: idle-socket starvation regression
  **PASS**, exit 0, 2.948 s including build; bounded 64 concurrent handlers with
  bounded socket reads/writes and joined shutdown. Zero surviving children.
- `format-apply/result.json`: formatter exit 0, 0.290 s. Final format/clippy/build
  and focused core/app/auth/worker checks still planned.

Two exact verifier specimens (19,620 and 26,664 bytes only) are maintained under
`tests/fixtures/phase6-corrections/`, with documented provenance/SHA-256; no
historical tree copied. The PNG contract deliberately refuses previously decoded
palette/grayscale/16-bit/unsupported ancillary semantics, which strict analysis
cannot use; documented safe refusal preserves, never converts/deletes originals.

The **single** full recipient confirmation is now active in
`recipient-confirmation/`, with unchanged 240 s outer limit and individual
persisted stage cancellation deadlines. It reuses the already verified unchanged
loopback-only worker/runtime as input, but installs a separate fresh recipient
runtime and creates two new stores. No result or acceptance is claimed yet.

## Checkpoint 3 — ONE recipient confirmation settled, PASS locally

`recipient-confirmation/{command.log,result.json}`: **exit 0, 125.865 s**, unchanged
240 s outer limit, no deadline, no forced cleanup, 45 adopted children reaped and
zero remaining children. `recipient/phase6-report.json` exists and records the
actual composite export → new-store import → separate recipient runtime install
→ headed OpenLogin → explicit confirm/ready → mapped probe → frozen TWO-target
check. Both targets were captured successfully and both final results are
NoSignificantChange. Original profiles stay width 600, future project width 640.

All ten stages have persisted starts/settlements in `recipient/stages.jsonl`:
owner discovery 11.307 s, baseline 13.450 s, check 13.631 s, export 0.048 s,
import 0.108 s, fresh recipient runtime 38.016 s, headed open 11.282 s, explicit
ready 0.814 s, mapped probe 11.584 s, fixed check 13.647 s. No stage deadline fired.
Import and missing-login checks retain zero navigation; original bytes/dates/
metadata and owner history are preserved. No unchanged 240/600 s retry was run.

The synthetic account self-establishes a fixture cookie in a separate headed
browser; it is NOT human authentication, native GPUI/visual acceptance, real OS
keystore, cross-machine/OS or public runtime qualification. Owner setup remains
explicit injected fixture material; recipient persistence uses a separate test
key and actual ready flow. A fresh independent verifier must still recheck fixes
and this now-maintained complete recipient gate before dependent Phase 7 reliance.

## Checkpoint 4 — final focused Rust/build and worker checks settled

`rust-checks.json` records all 12 initial focused stages exit 0: **20 focused Rust
tests**, locked all-feature/all-target build and clippy `-D warnings`, format.
Counts: core PNG 1, core coverage 1, final storage 4, storage fault/recovery 3,
Phase 6 services 2, composite/restore 1, auth/origin/Forget 7, simulated GPUI Jobs 1.
Together with the separate fixture regression and actual recipient test there
are **22 distinct Rust tests**, not a count of repeated attempts.

Final narrow strengthening additionally binds restore/export PNG dimensions to
the frozen metadata (as activation/import already do), and tests a valid 600×299
PNG falsely claimed 600×300. `rust-final.json` records all 8 affected stages exit
0 after those last edits: 4 storage + 2 Phase 6 service + 1 composite/restore +
1 actual simulated Jobs tests, locked workspace/all-feature/all-target build,
clippy `-D warnings`, fmt check. No production source edits after this final run.
The ONE recipient browser confirmation above preceded these extra metadata
dimension comparisons; it is not mislabeled a second final-source browser run.
Its original dimensions are checked read-only in the final audit; fresh independent
final-source recipient confirmation remains required. No retry was substituted.

`worker-checks.json`: **33 Node tests PASS**, 9 protocol/capture plus 24 actual
auth/resource/browser tests, 0 failed/skipped/cancelled. Durations 0.214 s and
9.275 s. Existing signed loopback runtime reused as an explicit input; all 11
maintained worker source/test/fixture hashes equal its signed manifest and current
source (`worker-inputs.json`). No worker source/dependency/runtime upgrade or new
public distribution claim. `worker-preflight-failure.txt` preserves a helper's
overbroad node_modules predicate failure before any test started; corrected to
the exact maintained src/test prefixes. All owned children settled: worker 12
adopted descendants reaped, zero forced cleanup/deadlines/remaining children.

Only focused regressions ran: no Phase 0–5/full600s suite, 100×3/soak, native black
window, OS-keychain or unrelated download was used as a diagnosis substitute.
Final preservation/privacy/process audit and status/handoff updates are next.

## Checkpoint 5 — final classified audit and delivery settled

`audit-b.json`: **PASS, zero failures**, completed after all checks and status
updates. Two fresh schema-v8 stores pass SQLite integrity/foreign keys with
**21 physically hash/byte/PNG-verified artifacts and six baseline bindings**, zero
pending GC, pending publication/import paths or owned live processes. The short
worker namespace is empty. No cleanup/signaling of unrelated resources occurred;
owned evidence is retained for the verifier, not recursively removed.

Both installed runtime inputs match every signed-manifest file and all 11 current
maintained worker files. Two distinct local session IDs, private encrypted-state
and export modes, ZIP original equality/CRC/inflation, privacy omissions, original
600/future640, ten settled stages and **two reports with zero gates/warnings/
failures** pass the independent read-only audit oracles. Thirty scoped logs contain
none of the actual session IDs or known fixture material sentinels. This is not
universal arbitrary-secret detection or runtime-release qualification.

Of 211 initially recorded source/doc/lock/corpus inputs, **198 remain byte-identical**.
Twelve deltas are the explicitly owned repair/status files, and one is the
orchestrator's documented `docs/ACTIVE_WORK.md` active-session/preparation update.
Historical verifier reports, the finished source-release handoff/helper, original
corpus and lockfiles remain unchanged. This is a recorded-input hash check, not a
claim of hashing every historical artifact tree. `audit-a.json` stays retained:
its initial blanket check flagged that unrelated orchestrator document update and
a helper incorrectly assumed JSON session refs rather than actual UUID columns.
The classified schema-correct audit fixes those oracles, not product evidence.

All 29 recorded owned command wrappers are settled with zero remaining children,
no outer deadlines or forced cleanup (failed test-helper attempts stay failed).
The early harness-interrupted diagnostic wrapper is separately classified in
checkpoint 1; its completed failing test log and absent owned processes are
preserved, not mislabeled exit 0. No live suite is handed off.

### Files changed / added

- `crates/core/src/analysis_engine.rs`: expose the existing audited strict validator;
  its CRC/DEFLATE implementation remains unchanged.
- `crates/core/src/domain.rs`: `DiscoveryCoverage` unknown-field refusal and
  valid-variant/schema cause tests.
- `crates/storage/src/images.rs`: strict same-open-file prevalidation, cancellation,
  explicit decoder CRC checks, row/tile limits preserved; no full-frame inflater.
- `crates/storage/src/review.rs`: validate all selected/inherited originals before
  atomic active publication; restore also binds bytes/dimensions to frozen metadata.
- `crates/storage/src/portability.rs`: export complete validation also binds PNG
  dimensions to every referencing portable capture before no-clobber publication.
- `crates/storage/src/{lib.rs,phase6_corrections_tests.rs}`: four maintained exact
  specimen/generated complete-stream/atomicity/valid-original/legacy gates tests.
- `crates/app/tests/product_flow.rs`: bounded concurrent owned fixture, maintained
  idle-socket regression, recipient headed synthetic Open/explicit-ready, stage
  persistence/deadlines and nonvacuous completed two-capture/two-result assertions.
- `scripts/check-phase6-corrections.py`: serial bounded owned-command runner with
  Linux subreaper/pidfd settlement, unique logs/results and no global cleanup.
- `tests/fixtures/phase6-corrections/{README.md,long-inflate.zip,unknown-coverage-field.zip}`:
  exact bounded verifier specimens, provenance/hashes.
- `docs/phases/{06-corrections.md,06-status.md,06-portability-and-storage.md}`,
  `docs/portable-baseline-archive.md`, `docs/IMPLEMENTATION_STATUS.md`: corrected
  contract, actual evidence, local/external limits and independent pending gate.

### Fresh independent verifier handoff — focused, serial, new outputs

Reidentify running tests first. Do not execute old pre-fix probes against existing
stores or repeat `scripts/check-phase6.py`/Phases 0–5 as a substitute for diagnosis.
Use the maintained exact tests (they create their own isolated stores), a distinct
output per command, and the bounded runner. Example:

```sh
export PATH=/path/to/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH
python3 scripts/check-phase6-corrections.py \
  --output /tmp/opencode/regressee-p6-reverify-UNIQUE-storage --timeout 180 -- \
  cargo test --locked -p regressee-storage phase6_ -- --nocapture --test-threads=1
```

Other exact focused commands, each in its own runner output:

```sh
cargo test --locked -p regressee-core complete_png_integrity_checks_have_specific_failure_oracles -- --nocapture --test-threads=1
cargo test --locked -p regressee-core discovery_coverage_strict_schema_and_all_valid_variants_roundtrip -- --nocapture --test-threads=1
cargo test --locked -p regressee-storage portability:: -- --nocapture --test-threads=1
cargo test --locked -p regressee-app --features ui-tests --test analysis_review phase6_ -- --nocapture --test-threads=1
cargo test --locked -p regressee-app --features ui-tests --lib auth::tests:: -- --nocapture --test-threads=1
cargo test --locked -p regressee-app --features ui-tests --bin regressee phase6_simulated_native_picker_confirmation_and_actual_jobs_dataflow -- --nocapture --test-threads=1
cargo test --locked -p regressee-app --features ui-tests --test product_flow phase6_fixture_speculative_connections_do_not_block_ready_requests -- --exact --nocapture --test-threads=1
cargo build --locked --workspace --all-features --all-targets
cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
cargo fmt --all -- --check
```

For ONE independent final-source complete recipient confirmation, verify the
retained signed loopback input/current-worker hashes first (see `worker-inputs.json`),
then use a new evidence directory and 240 s outer deadline:

```sh
export REGRESSEE_PHASE6_DISTRIBUTION=/tmp/opencode/regressee-p6-independent-f5e83c04c514/maintained/signed-safety/product-flow/distribution.json
export REGRESSEE_PHASE6_RUNTIME_CACHE=/tmp/opencode/regressee-p6-independent-f5e83c04c514/maintained/signed-safety/product-flow/app/cache/product-runtime
export REGRESSEE_PHASE6_EVIDENCE=/tmp/opencode/regressee-p6-reverify-UNIQUE-recipient-data
python3 scripts/check-phase6-corrections.py \
  --output /tmp/opencode/regressee-p6-reverify-UNIQUE-recipient-command --timeout 240 -- \
  cargo test --locked -p regressee-app --features ui-tests --test product_flow \
  actual_phase6_composite_zip_fresh_recipient_login_and_fixed_mapped_check \
  -- --exact --ignored --nocapture --test-threads=1
```

Inspect `stages.jsonl`, final report, the three-entry original-only ZIP, both actual
SQLite stores, separate recipient session/reference and fresh installed runtime.
Independently probe exact CRC/row/compressed-tail/Adler/terminator refusals before
every active gate, including inherited selected publication and legacy restore/
export, without loss of existing rows, active baseline or CAS files. Exercise safe
unsupported refusal and unchanged RGB/RGBA originals, and classify each exact
cause rather than accepting unrelated errors. Reuse current source-matched worker
regression commands from `worker-checks.json` with fresh output env vars, serially.
Audit final source hashes, private state/logs/scratch and all owned processes.

## Remaining open gates / final verdict

**P2/P3 implemented and maintained locally; recipient local confirmation PASS.
Fresh independent FINAL-SOURCE reverification PENDING.** Dependent Phase 7 remains
held for that separate verdict; no Phase 7 code/agents/features were started.

Native GPUI rendering/input/human/accessibility, actual human login, real OS-key
persistence, another supported clean machine/cross-OS, current public runtime/
signing/distribution, maximum-scale/soak and physical exhaustion/power-loss remain
PENDING / NOT RUN. Historical Node22.20.0/worker4/Playwright1.56.1/Chromium141.0.7390.37
is DevelopmentLoopback only. Phase 5 tiny-mask P3 remains open. Masks never redact
raw originals; private queries deliberately remain visible. No publication, global
installs, privileged operations, sandbox/security flag bypass, services or
unscoped cleanup were performed.
