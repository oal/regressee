# Agent CLI Phase 3 — independent verification

**Final verdict: PASS for Phase 3, protocol 4, within the reviewed source,
Linux IPC and owned synthetic-fixture scope.** All six initial findings are
independently closed. No new repair-specific blocking defect was found. This is
not native/live-browser, desktop-login, clean-server, release or Phase 4
qualification. The initial NEEDS FIX review below is retained as history.

## Independent repair re-review — 2026-10-17

Re-read the repair ledger, CLI/engine contracts, actual source and regressions,
and `/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase3/repair/repair-handoff.json`.
All handoff source hashes matched independently, including this report's preserved
pre-re-review SHA-256
`9d1d81bb0532520481d6d8bdaeb656f881afc425bbcad6072cb56308362c1d7e`.
Only this report was edited in the repository. No production/test edits, commits,
pushes, subdelegation, user-store access, external-site capture, publication,
system installation, sandbox changes or Phase 4 implementation occurred.
Heavy shared-target compilation was exclusively verifier-owned and serial.

### Original witness adjustment: justified, not a weakened identity check

Independently compared the implementor's scratch scripts with the original proofs.
The default-reference test's only behavioral edit was `expect=5` → `expect=0`
for immediate confirmed report export; its original before-hash and baseline-ID
assertions were unchanged. The accompanying comment changed too. The parser
scratch copy only adjusted its relocated manifest lookup. All other original
witness/host sources compared byte-for-byte. The original independent proof files
and original receipts remain untouched; hashes are recorded in the re-review.

I did **not** simply accept the edited test as green evidence:

1. Reran a byte-identical copy of the **unmodified** original eight-scenario
   workflow script against the rebuilt production CLI/engine. Seven passed. The
   default-reference scenario failed specifically at its old guard assertion:
   **expected export 5, actual 0**, not compilation, fixture setup or an identity
   assertion. Its original guard had intentionally described the defective
   inconsistent preview, not the desired repaired behavior.
2. Created my own separate acceptance copy with **only that one expected exit
   changed to 0**. The original hash and baseline-ID assertions then executed
   unchanged and passed. The one-line diff is retained as
   `rereview/acceptance-delta.patch`; the original was not rewritten.
3. Added separate, stronger independent acceptance tests. After selected
   acceptance and restoration of both retained versions, omitted and explicit
   identity flags return identical full artifact DTOs. The before/after/difference
   PNGs in the exported report match the original exact-run files byte-for-byte;
   their hashes match the descriptors. The decoded HTML content equals its frozen
   preview, the baseline is B0, the analysis remains A, and `analysis show A`
   remains unchanged. Default before crops retain the exact original source.

The adjustment is therefore **required and justified**: a correct, unchanged
preview should export successfully. The original repair guidance explicitly
required that success. Identity/hash checks were neither removed nor bypassed;
stale/forged/replayed/cross-owner confirmations still fail in the rerun tests.

### Findings independently closed and analogous paths checked

1. **Original reference identity — CLOSED.**
   `crates/app/src/terminal.rs:218–268,324–414,481–576` resolves a run first,
   including an explicit older analysis, and uses the exact-run reader.
   `crates/storage/src/capture.rs:839–908` no longer attaches the active baseline
   to a non-check exact read; the viewer also retains a check's original reference
   without analysis (`992–1007`). Independently verified default/explicit parity
   after acceptance/restoration, successful report/crop export, unchanged saved
   analysis/difference, older-analysis selection after a newer run, explicit
   run/analysis and cross-project rejection, a candidate with no invented before
   reference, and a paused check with no analysis after another baseline becomes
   active. All result/artifact/crop/report paths share the same resolver.
2. **Current review loss / incomplete history — CLOSED.**
   `crates/storage/src/review.rs:380–466` provides focused current decisions and
   stable-ID keyset history. `crates/app/src/terminal.rs:292–305,527–532,675–691`
   attaches current decisions for the selected analysis/targets rather than the
   recent run-wide window. The unchanged 257-review witness now retains the first
   target's `needs-fix` note in results and both exported formats. Independently
   checked filtered pages, no duplicate/missing older entries during a concurrent
   append, live totals, most-recent committed decisions and old/new analysis
   isolation. Recent workspace/history evidence explicitly exposes truncation;
   reports explicitly describe current decisions, not full history. All 257
   decisions are reachable in the new in-repository CLI pagination regression.
3. **Prepared-ticket recovery — CLOSED.**
   `crates/cli/src/main.rs:223–241` reconstructs the same typed ticket centrally
   for prepare/show/wait/retry; `--save` saves it. Independently recovered all three
   routes, compared complete tickets/submissions, confirmed once, rejected replay,
   and recovered again after use and engine restart. Owner/epoch/revision/ID stay
   unchanged, receipt reads do not advance the engine sequence, and the restarted
   owner's confirmation refusal remains a conflict. No new prepare or mutation
   replay is used to recover authority.
4. **Configuration prerequisite Ctrl-C — CLOSED.**
   `crates/cli/src/main.rs:299–435,595–660` replaces blocking `Client::execute`
   prerequisites with journaled interrupt-aware reads and guards continuation
   admission. The original witness exits 130 **0.053 seconds** after SIGINT,
   with `continuation_submitted:false`. Five independently started profile,
   environment, scope and environment-selection commands all detached together
   in **0.067 seconds**, exposed their actual Load/read identities, did not cancel
   the capture and did not change the project revision. The inspected/rerun
   in-repository regression also targets the second `EnvironmentStatus` read.
   Its monitor's handling of an acknowledged/reclaimed read is legitimate;
   prompt-exit, exact-read and no-continuation assertions remain intact.
5. **Validation/error/startup contract — CLOSED.**
   `crates/cli/src/main.rs:35–42,247–268,506–561` supplies typed local validation
   and pre-start flag checks; `commands.rs:264–392` validates bounded DTO/domain
   inputs locally while saved-state checks remain in the engine. The original
   27-case parser probe has no wrong exits and no startup. An additional independent
   **38-case** matrix covers valid-schema invalid project/profile/environment/
   scope values, robots authorization, nil IDs, bounds, review cursors/filters,
   selected keys, note length/UTF-8 and analogous inappropriate prepare flags:
   all return clean schema-1 `invalid_request`/2 without creating data or cache.
   Fresh production `project list --prepare` no longer creates storage. Existing
   conflict, requirement, incomplete-capture and visual-difference regressions pass.
6. **Frozen report settings/provenance — CLOSED.**
   `crates/app/src/terminal.rs:172–194,527–576,799–814` freezes queue/discovery/
   project-revision/resume data and selected reference-source settings; absent
   legacy provenance is explicitly unknown. The unchanged settings witness passes.
   Independently changed today's configuration after capture and verified the
   preview/export still contain the recorded capture and reference queues, not
   today's values. Injected malicious scope text remained data in the decoded
   frozen JSON and produced no script/remote resource element. Privacy inventory,
   fingerprint/revalidation and metadata bounds include these new fields. The
   rerun app regression verifies legacy absence without guessing current settings.

No new direct-Store CLI path or recovery owner was introduced. Product operations,
including prerequisite reads and review history, still use the shared engine.
The source/action inventory from the first review remains applicable; `review
history` now fills its previously incomplete history semantics. Protocol 4 rejects
protocol-1/2/3 peers rather than bypassing ownership; the CLI envelope remains
schema 1. Internal source-release classification of this report is now present
and its script tests pass. Follow/leases/semantic agent navigation/handoff remain
explicitly absent and deferred only to Phase 4.

### Exact independent re-review evidence

```sh
R=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase3-review/rereview
export PATH=/var/home/olav/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH
export TMPDIR="$R"
# GUI link commands alone use the existing Phase 1 link-libs directory.
export REGRESSEE_REVIEW_CLI="$R/bin/regressee-production"
```

The rebuilt/frozen production CLI SHA-256 is
`1f260b677c811653a6df4c8487c3de8762654c8003555496dfea01ed825273bb`,
matching the repair handoff; protocol **4**, UI/developer/test/inspection features
all **false**. The separate signed synthetic engine host intentionally uses test
support. `entry-proof.json`, `binaries.json`, `checks.json`, `summary.json`,
`source/` and `report-before.md` bind source, original-proof preservation,
commands, results and released owned fixture locks. Per-process transcripts and
witness reports are under `original/` and `acceptance/`, not the first review's
stores or original proof locations.

| Exact command (environment above) | Independent re-review result |
| --- | --- |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1 --skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` | **244 passed, 0 failed, 29 ignored, 1 scale case filtered**, excluding the nested environment subprocess. App 58/1 ignored, engine 18/1 ignored, CLI 16 passed including all six repair regressions. `backend.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**; `gui-client.log`. Production IPC facade, not native input qualification. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --bin regressee-desktop engine_refresh_tests:: -- --test-threads=1` | **2 passed**, 231 filtered; `gui-refresh.log`. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Passed; `check-workspace.log`. Inherited future-compatibility warning remains. |
| `env -u DISPLAY -u WAYLAND_DISPLAY PKG_CONFIG=/bin/false LIBRARY_PATH= cargo build --locked --offline -p regressee-cli --bin regressee` | Passed; `production-build.log`. Cached/shared target, **not** a fresh-host or fresh-target proof. Dependency/ELF checks from the independent production probe remain GUI/GPUI-free. |
| `CARGO_TARGET_DIR=/var/home/olav/Projects/Regressee/target CARGO_PROFILE_DEV_DEBUG=0 cargo build --locked --offline --manifest-path "$R/probe/Cargo.toml"` | Passed; `host-build.log`. Rebuilt byte-identical copies of the original independent host/worker source against repaired app code. |
| `python3 -B -u "$R/original/workflow_probe.py"` | Unmodified original source: **7 passed / 1 obsolete guard assertion failed** (expected 5, actual 0), explicitly analyzed above; `original-workflows.log`. No failure of setup/compilation was hidden. |
| `python3 -B -u "$R/acceptance/workflow_probe.py" WorkflowReview.test_default_selection_must_keep_original_check_reference` | **1 passed**, with only the documented export-exit correction; original identity/hash assertions unchanged. `acceptance-default.log`. |
| `python3 -B -u "$R/original/review_history_probe.py"`; likewise `report_settings_probe.py`, `interrupt_config_probe.py`, `tall_crop_probe.py` | **1 passed each**, unchanged original sources. `original-history.log`, `original-settings.log`, `original-interrupt.log`, `original-tall.log`. |
| `python3 -B -u "$R/positive_probe.py"` | **6 new independent acceptance tests passed**; full evidence/output parity, older/foreign/paused/non-check identities, append-safe history/current notes, recovery after consume/restart, five parallel interrupted helpers, frozen/escaped source settings. `positive.log`. |
| `python3 -B "$R/original/parser_probe.py"` | **27/27 clean JSON, no wrong exits, no startup**; `parser-summary.json` and original-copy parser receipts. Reported differences from the old initial source manifest are the expected repair delta, not failed source binding. |
| `python3 -B -u "$R/validation_probe.py"` | **38/38 clean/correct/no-start cases**; `validation.log`, `validation-results.json`. |
| `python3 -B "$R/original/production_probe.py"` | **12 production invocations**; no display/runtime/keychain initialization on reads, no invalid-prepare store creation; `production-summary.json` and production receipts. |
| `python3 -B -m unittest discover -s scripts -p 'test_*.py'` | **59 passed**; `python-tests.log`. |
| `python3 -B "$R/summarize.py"`; `git diff --check` | Original independent proof hashes unchanged; repair source hashes match (apart from this authorized report update); all owned fixture engine locks released. 797 recorded workflow subprocess calls including the 256-review loop, plus 38 extra validation cases; original parser/production invocations are additional. |

**Disposition:** all six Phase 3 findings closed; Phase 3 may advance. No full GUI
or unchanged expensive scale rerun was needed. The five known inherited GUI
failures, existing formatting/lint debt, receipt quotas, conservative stale
confirmation invalidation and uncertain commit windows are not waived. Synthetic
workers/local PNGs/owned loopback setup do **not** qualify Chromium/public-site
rendering, native managed login/input/accessibility, clean-server dependencies/
fonts/default sandbox, Windows/macOS IPC/ACL/lifetime, packages or a portable agent
skill. Those native/live/release and Phase 4/5 gates remain explicitly open.

## Historical initial review — superseded by the re-review above

**Initial verdict: NEEDS FIX.** Six actionable findings remained: one HIGH and five MEDIUM.
The named CLI surface is substantially complete, but evidence identity, review
completeness, recovery and interruption contracts prevent Phase 3 acceptance.
This is not a rejection of the independently accepted Phases 1/2.

Reviewed 2026-10-17 against uncommitted source above
`9daf121b960f84506feedefcdf49869327861130`. Every file hash in the implementor's
`agent-cli-phase3/source-handoff.json` matched at entry and final source freeze.
Read the plan, Phase 3 ledger, CLI/engine guides, GUI dispatches, protocol,
handlers, storage/evidence code and actual tests, not just the coverage table.
Only this report was added to the repository. No production edits, test insertion,
revert, commit, push, subdelegation, user-store access, external-site capture,
publishing, system installation or sandbox changes occurred.

## Initial findings (historical; all independently closed above)

### 1. HIGH — Default evidence substitutes the active baseline for the check's original reference

**Paths:** `crates/app/src/terminal.rs:192–215,280–315,433–468`;
`crates/storage/src/capture.rs:813–845,854–867`.

Without `--run`, `evidence()` uses the workspace snapshot, whose baseline is
today's active baseline. It then freezes the latest run and its saved analysis
without reconciling that baseline with `RunIntent::Check`/analysis identity.

**Independent reproduction:** create/save B0, run changed check R/A, accept one
selected target into B1, then call `results artifacts --project P --page PAGE
--profile PROFILE`, omitting the optional run/analysis flags. The before artifact
becomes the accepted **after** artifact, although the difference/report still
belongs to R/A against B0. Explicit `--run R --analysis A` correctly retains B0.
Observed original before SHA-256 starts `918dff98`; default before instead starts
`f3530f41` and equals after. `report preview` without `--run` likewise returns B1
beside an analysis bound to B0. Immediate confirmed export, with no intervening
write, then fails `conflict`/5 because revalidation uses the now-explicit R and
resolves B0. Thus reads/previews misrepresent evidence and a valid-looking preview
cannot be exported; **the export guard did prevent publishing that bad report**.

Reproduce: `python3 "$E/workflow_probe.py" WorkflowReview.test_default_selection_must_keep_original_check_reference`.
See `production-workflows.log` and the production fixture's
`reference-witness.json` listed in `summary.json`.

**Repair:** resolve the default run first, then load the same exact-run evidence
used for an explicit run. Enforce run/analysis/original-baseline consistency for
every results/artifact/crop/report path. Add post-accept/restore tests comparing
omitted and explicit identity flags, including immediate preview→export success.
Do not fix this by silently changing the saved analysis or difference image.

### 2. MEDIUM — Current review decisions and report notes silently disappear after unrelated reviews

**Paths:** `crates/app/src/terminal.rs:217–247,451–463,695–714`;
`crates/storage/src/review.rs:360–364`;
`crates/storage/src/capture.rs:929–932`.

The CLI derives each target's reviews from the latest **256 decisions for the
whole run**, not the target's current decision or a paginated complete history.
One target's current decision can disappear entirely after activity elsewhere.

**Independent reproduction:** save `needs-fix`/`KEEP_FIRST_TARGET_NOTE` for target
one; make 256 successful review saves for target two. `results list --run R
--analysis A --page FIRST_PAGE` now returns `reviews: []`, without a truncation
indicator. A successfully exported selected-target HTML/Markdown report also
omits target one's saved decision/note. The save history is append-only; these
unrelated writes must not make its current decision appear absent.

Reproduce: `python3 "$E/review_history_probe.py"`.
See `production-review-history.log` and `review-history-witness.json`.

**Repair:** retrieve authoritative current decisions per selected analysis/target
independently of the bounded recent-history window. Provide bounded history
pagination or explicit truncation metadata where history is promised. Reports
must include the selected targets' current decisions/notes and disclose any
deliberately bounded history, not silently return an empty review state.

### 3. MEDIUM — Retrying a prepared request cannot recover a publicly usable confirmation ticket

**Paths:** `crates/cli/src/main.rs:126–214,370–379,598–626,662–665`.

The normal prepare path synthesizes `{operation,confirmation}` in `data.ticket`.
`jobs retry`, `jobs show` and `jobs wait` only call the ordinary result formatter;
they return the bare Confirmation and never reconstruct that ticket.

**Independent reproduction:** prepare `baseline restore --baseline B
--expected-revision N --request-id REQUEST --prepare`; simulate losing its output.
Run `jobs retry --request-id REQUEST --save recovered.json`, then
`confirm --file recovered.json --confirmation ORIGINAL_CONFIRMATION_ID`.
Retry exits 0 but saves only the Confirmation; confirm exits 2 with
`JSON does not match ... input schema`. Recovering requires manually assembling
an internal Operation/Ticket from wire fields, contrary to the public recovery
workflow. No restart or stale confirmation is involved.

Reproduce: `python3 "$E/workflow_probe.py" WorkflowReview.test_prepared_request_retry_must_recover_usable_ticket`.
See `production-workflows.log` and `prepare-recovery-witness.json`.

**Repair:** centralize prepared-job result formatting so show/wait/retry and
`--save` recover the exact original typed operation plus its original confirmation
as the same usable ticket. Preserve owner/epoch/revision/ID; never regenerate
confirmation authority or replay a mutation to make recovery convenient.

### 4. MEDIUM — Configuration prerequisite waits ignore Ctrl-C behind a live job

**Paths:** `crates/cli/src/main.rs:517–585`;
`crates/app/src/engine/client.rs:249–274`.

Named configuration helpers call blocking `client.execute(Load(...))`, and
environment selection may additionally execute `EnvironmentStatus`, before
entering the CLI's interrupt-aware submit/wait path. Those waits never inspect
the installed Ctrl-C flag. They can also continue to submit the later mutation
after the user has interrupted the prerequisite wait.

**Independent reproduction:** start a detached synthetic capture with 1.8-second
target delays, start `profiles set ...` behind it, then SIGINT that CLI process.
It remains alive until its internal Load reaches the front of the worker queue:
**4.87 seconds after SIGINT**, then eventually returns `wait_detached`/130. This
is not engine unresponsiveness: ordinary job controls remain responsive. The
internal client wait itself ignores interruption; a real slow job can hold it
much longer. The later save was admitted despite the earlier interrupt.

Reproduce: `python3 "$E/interrupt_config_probe.py"`.
See `production-config-interrupt.log` and `interrupt-witness.json`.

**Repair:** make all public-command prerequisite waits interrupt-aware and give
their receipts/recovery a coherent public identity. Exit promptly with truthful
detached status, do not cancel existing engine jobs, and do not newly submit a
continuation mutation after observing interruption. Cover profiles/environments/
scope set and both environment-select prerequisite reads.

### 5. MEDIUM — Public validation/error contract is not consistent, and an inapplicable flag starts storage

**Paths:** `crates/cli/src/commands.rs:289–300`;
`crates/cli/src/main.rs:216–247,278–288,466–514`;
`docs/cli.md:20–24,323–333`.

**Independent reproductions:** all emit one clean schema-1 JSON document, but
these domain-invalid invocations return `operation_failed`/6, not documented
`invalid_request`/2:

```sh
regressee --json project create --name '' --url http://127.0.0.1:43191/
regressee --json project create --name x --url not-a-url
regressee --json project create --name x --url https://user:secret@example.invalid/
```

They do not create storage; no request/site visit is necessary. The error mapping
depends on incidental words in human-readable error strings, so ordinary domain
validation messages are mistaken for engine failures.

Separately, with fresh explicit private paths, `project list --prepare` returns
`invalid_request`/2 **after autostarting the engine and creating/migrating the
store**. Applicability is checked inside `submit()` after autostart, contradicting
the advertised pre-start rejection of inapplicable flags. This did not download
a runtime, but the invalid request still had persistent startup side effects.

Reproduce: `python3 "$E/parser_probe.py"` and
`python3 "$E/production_probe.py"` (use fresh probe output roots for a repeat of
the absent-store assertion). Receipts: `parser-results.json`,
`parser-summary.json`, `production-results.json`.

**Repair:** classify parser/domain validation with explicit typed/contextual error
categories rather than substring guesses. Check global submit-flag applicability
against the resolved public plan before any data discovery/autostart. Extend the
error matrix to invalid domain values and inappropriate prepare flags, not only
unknown command names and malformed UUIDs. Preserve clean stdout and no prompts.

### 6. MEDIUM — Reports omit available frozen discovery/settings provenance while claiming completeness

**Paths:** `crates/app/src/terminal.rs:162–168,461–468,678–689,735–739`;
`docs/cli.md:217–220`.

`ReportContent` retains only `RunRecord.run` (`CaptureRun`), discarding its saved
`queue`. The exported “complete frozen settings” therefore contain no recorded
discovery settings (scope/search mode, limits, exclusions, robots authorization,
pacing/retries), capture project revision, or resume timestamps. This information
is available in the same exact historical record; it is not an unavoidable
legacy-data absence. Profiles, rendering settings and inventory are present,
but do not reconstruct those missing settings.

**Independent reproduction:** export an explicit R/A report, then compare
`history show --project P --run R`'s `evidence.latest.queue` with report preview
content and exported HTML. The former contains the actual frozen DiscoverySettings
and revision; the latter contain neither queue nor discovery settings, while the
summary directs readers to the HTML for full recorded settings.

Reproduce: `python3 "$E/report_settings_probe.py"`.
See `production-report-settings.log` and `frozen-settings-witness.json`.

**Repair:** include available immutable recorded queue/discovery/revision/resume
provenance in preview, fingerprint and report. For legacy absence explicitly say
unknown; never substitute today's project settings. Recheck privacy inventory
and metadata bounds for the added fields. Retain the distinction between saved
inventory coverage and proof of whole-site coverage.

## Actual action coverage audit

Compared `jobs::Command`, protocol operations, engine dispatch and production GUI
actions (including `main.rs`, `safety_ux.rs`, `exit_guard.rs`, history and task
paths) with actual CLI plans/handlers. No wholly missing current product-action
namespace was found. The defects above are semantic gaps, not a request to defer
legitimate product actions to Phase 4.

| Product action family | Actual named path / independent coverage and disposition |
| --- | --- |
| Project create/list/load/save, expected revisions | `project create/list/show/update`; real subprocess create/update/list/read; stale revision and protected-pointer/replacement backend cases pass. Invalid-domain exit defect is #5. JSON/stdin bounds also exercised by the independently rerun in-repo subprocess tests. |
| Profile/environment CRUD, scope and capture settings | `profiles/environments/scope show/set`, full project JSON; independent read/set round trips and settings revision checks. Fields are real domain DTOs, not shadow flags. Ctrl-C prerequisite gap is #4. |
| Selected environment/local login settings, combined replacement | `environment status/select/save [--replacement]`; independent ordinary atomic save, protected ordinary-save refusal and confirmed combined replacement. Source and engine regressions check source/profile/environment families and distinct confirmations. |
| Discovery, warnings, existing inventory, selection | `discovery start/status/inventory/existing/include/exclude`; independent detached discovery/job reopening, stable page IDs, include/exclude round trip and recorded-list reopening. |
| Representative screenshots, baseline capture, check, resume | `capture preview/baseline/check/resume`; independent preview cannot activate, fixed-list checks, paused original run and new resume job. Synthetic successes, changed checks, partial and wholly failed captures remain distinct. |
| Async status/progress, wait, pause, cancel, retry/ack | `jobs list/show/wait/pause/cancel/retry/ack`, detach/request ID; independent SIGINT wait exits 130 without cancellation, live stop refusal, pause/resume/cancel and original-input retry across restart. In-repo lifecycle/engine checks cover acknowledgement, lost-receipt uncertainty, concurrent owners and crash windows. Recovery defect #3 and helper wait defect #4 remain. |
| Result filters, pagination, identity and coverage | `results list/artifacts`, run/analysis/page/profile/outcome/capture-state/offset/limit; independent changed/captured/failed filters and next page, limits, exact original/difference metadata. Backend classification tests cover noncomparability/absent analysis, not false unchanged. Default-identity defect #1 remains. |
| Sensitivity/custom policy, saved analysis/display selection | `analysis reanalyze/show/select`; independent presets, policy-file round trip, old/new immutable identities and explicit display selection. |
| Ignored rectangles and clearing | `masks save`, reanalyze; independent exact descriptor binding, invalid bounds refusal, retained mask, empty clear revision and unmodified original hashes. |
| Review decisions, notes and history | `review set`, results/history evidence; independent all three states, note-file and malicious text, stale approval. Silent current-review/history loss is #2. |
| Baseline save/full/partial, selected acceptance, restore | `baseline save/preview/accept/restore`; independent confirmation required, representative refusal, explicit partial target subset, selected acceptance, original check reference/history and retained-version restore. |
| History/usage/reference-aware deletion | `history list/show`, `storage usage/delete-preview/delete`; independent retained baseline/check history, global/project usage and confirmed inactive baseline plus disposable run deletion. Shared-reference protections covered by engine/storage regressions. |
| Repair inspection/temporary/unreferenced cleanup | `repair preview/clean/delete-unreferenced`; independent confirmed no-op and real service round trips; backend malicious/stale/missing/corrupt/GC tests pass. No replacement evidence is invented. |
| Existing baseline ZIP | `archive export-preview/export/import-preview/import`; independent confirmed local round trip; backend hostile ZIP, source provenance and secret-exclusion regressions pass. No report contract was substituted for ZIP. |
| Diagnostics/network/browser setup | `diagnostics preview/export`, `network status/set`, `browser status/setup`; independent sanitized preview/confirmed export, network round trip and explicit setup from an owned loopback-served signed synthetic pack. Production status/diagnostics/list do not install anything. |
| Appearance | `appearance show/set`; independent durable read/write; GUI facade/refresh regressions pass. |
| Desktop login and headless rejection | `login status/open/confirm/close/forget/probe`; headless open rejected before runtime/keychain initialization; other named routes exercised with public synthetic state. Actual desktop login source service preserved; native headed login/credential-store usability is **not qualified** here. |
| Exact artifacts, bounded derived crops, local report/privacy | Independent original paths/hashes/dimensions/DPR 1, screenshot-vs-CSS refusal, zero/overflow/out-of-bounds rectangles, forged sources, data/cache/alias/traversal/existing/symlink destinations, original nonoverwrite, derived labels and tall 4M boundary pass. HTML/Markdown injection and private output modes pass. Report identity/settings/review defects #1/#2/#6 remain. |
| Native dialogs/clipboard/link opening/viewer gestures/drafts | Explicit paths/JSON and original/crop evidence are legitimate terminal equivalents; GUI presentation and unsaved drafts stay GUI-owned. No fake terminal pan/overlay or automated external link launch is claimed. |
| Agent sessions/semantic navigation/follow/lease/handoff/takeover | Correctly absent from parser/capabilities and explicitly deferred to **Phase 4 only**. Unknown agent/follow commands rejected without startup; no false exclusive-control claim. |

All public product calls inspected use the engine. The CLI reads/writes its own
request/input/output files, not Store/SQLite; no CLI Store open/recovery path was
found. The exact-run fix already accepted in Phase 2 remains correct when used;
finding #1 is its incorrect default selection by the new terminal adapter.

## Exact independent checks and evidence

Owned external root (all fixtures, scripts, reports, crops, packs and logs):

```sh
E=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase3-review
export PATH=/var/home/olav/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH
export TMPDIR="$E"
# Only GUI link commands needed:
export LIBRARY_PATH=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase1/link-libs
```

Heavy compilation was serial and verifier-owned. `summary.json` records hashes,
per-target totals, production subprocess transcripts, witness locations, memory
receipt and released locks. `source/` freezes all handoff-manifest source files.
The production CLI used for final independent probes has SHA-256
`be830a8a809b2cd164833ea3c8e35fa50fa175c790812124bc65de139786a959`, matching
the handoff production binary; UI/developer/test/inspection feature flags are
all false. The separate synthetic engine host intentionally uses test support.
The workflow/witness scripts default to that frozen CLI copy. For repair checks,
rebuild the external host and set `REGRESSEE_REVIEW_CLI` to the newly built CLI;
do not mistake rerunning the frozen defective executable for testing a repair.

| Command | Independent result |
| --- | --- |
| `cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-runtime -p regressee-app -p regressee-cli --features regressee-app/test-support -- --test-threads=1 --skip phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention` | **237 passed, 0 failed, 29 ignored, 1 scale case filtered**. Nested environment subprocess not counted twice. App 57/1 ignored; engine 18/1 ignored; CLI 10 passed, including its eight real subprocess workflows. `backend.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**. Production IPC facade and protected saves/replacement/detach; `gui-client.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --bin regressee-desktop engine_refresh_tests:: -- --test-threads=1` | **2 passed**, 231 filtered; `gui-refresh.log`. GPUI simulations, not native input. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Passed; `check-workspace.log`. Inherited future-compatibility warning remains. |
| `env -u DISPLAY -u WAYLAND_DISPLAY PKG_CONFIG=/bin/false LIBRARY_PATH= cargo build --locked --offline -p regressee-cli --bin regressee` | Passed, shared/cached target, **not a fresh-target proof**; `cli-production-build.log`. Normal/build dependency tree excludes GUI/GPUI; ELF excludes X11/Wayland/Vulkan/OpenGL/C++. Prior Phase 1/2 fresh proof is not recharacterized as this phase's work. |
| `CARGO_TARGET_DIR=/var/home/olav/Projects/Regressee/target CARGO_PROFILE_DEV_DEBUG=0 cargo build --locked --offline --manifest-path "$E/probe/Cargo.toml"` | Passed; final `probe-build-tall.log`. Owned signed Python worker/real engine, no Chromium. Initial external manifest lock generation also used offline Cargo. |
| `python3 -u "$E/workflow_probe.py"` | **6 passed, 2 failed at correct-expectation assertions** (#1/#3), not fixture setup or compilation. `production-workflows.log`; 243 public CLI calls. |
| `python3 -u "$E/review_history_probe.py"` | **1 failed at correct-expectation assertion** (#2), after 257 successful review saves; `production-review-history.log`. |
| `python3 -u "$E/report_settings_probe.py"` | **1 failed at correct-expectation assertion** (#6); `production-report-settings.log`. |
| `python3 -u "$E/interrupt_config_probe.py"` | **1 failed at correct-expectation assertion** (#4); `production-config-interrupt.log`. |
| `python3 -u "$E/tall_crop_probe.py"` | **1 passed**: 2048×4000 original, exact 1000×4000 (4M) crop succeeds, 1001×4000 refused, unchanged original hash. Engine lifetime VmHWM 45,696 KiB includes ingestion/IPC, not isolated allocation/browser measurement. `production-tall-crop.log`. |
| `python3 "$E/parser_probe.py"` | **27/27 clean single JSON documents**, no data startup; **3 wrong documented domain-error exits** (#5). Missing args, UUIDs, unknown/inapplicable flags, duplicates, deferred commands and metadata tested. `parser-results.json`. |
| `python3 "$E/production_probe.py"` | 13 no-display production invocations; explicit isolated paths/HOME/invalid session bus. Read-only product operations create no runtime or credentials; #5's inappropriate prepare flag creates a store. `production-results.json`, dependency/ELF receipts. |
| `python3 "$E/summarize.py"` | All handoff source hashes still match; all owned fixture engine locks released. Final independent workflow probes made **546 production CLI subprocess calls**, including 256 separately recorded review-loop calls; parser/smoke invocations are additional. |

External Python tests total **7 passed / 5 failed** across 12 scenario tests;
parser/production validation findings are recorded separately. Earlier
`independent-workflows*.log` files include fixture bring-up (unsupported DPR 2 and
a missing synthetic warning event); they are not final product failure counts.
The final production logs above use corrected fixtures and the matching
production executable. No second run of the expensive scale case occurred.

## Bounded disposition and remaining limits

- **Original implementor should repair #1–#6, add in-repository regressions and
  hand back for independent re-review.** Do not advance this gate to PASS merely
  because the existing suite is green. No Phase 4 implementation is requested.
- Positive adversarial checks found no confirmation bypass: changed action,
  selection or destination, wrong UUID, stale storage revision, consumed ticket
  and restarted owner fail. Ordinary settings replacement remains blocked;
  explicitly confirmed combined replacement succeeds. Failed captures are missing
  evidence, not substituted images or unchanged results. Operational/incomplete
  outcomes take precedence over visual differences.
- Crops use bounded verified encoded bytes frozen to a private temporary file,
  complete PNG validation and row-wise decoding; 4M crop cap and checked rectangle
  arithmetic are real. Native non-1 DPR capture remains unsupported by the existing
  product; no synthetic fixture is represented as real DPR/browser qualification.
- Reports have escaped untrusted notes/URLs/text, restrictive CSP, no executable
  remote resources, unchanged original PNG copies and private output permissions.
  Static/forged/symlink/existing output attacks tested here did not overwrite
  originals. Privacy previews explicitly expose private URL/query/note/local-path
  risks; no credentials, session payloads, network settings or logs were found in
  the tested reports. This is not a sandbox against other same-user programs.
- Synthetic workers/local PNGs and loopback pack download **do not qualify actual
  Chromium/public-site rendering, managed desktop login, native windows/input,
  clean-server fonts/libraries/default sandbox, Windows/macOS IPC/ACL/lifetime,
  packaged binaries or a portable agent skill**. Existing Phase 4/5/platform gates
  remain. Five known inherited GUI failures were not blanket-rerun or waived.
- This new review document is internal. Before a future source-package check,
  the implementor should classify it alongside the Phase 1/2 review documents in
  `scripts/source_release.py:41–45`; the verifier intentionally did not edit that
  production policy or create/publish a package.
