# DOM snapshot implementation — verification record

Started: 2026-10-17; continued 2026-10-18. Current verdict:
**BLOCKED — recorded-History DOM render confirmation fails; independent diagnosis pending**.

The full feature is not yet delivered or independently accepted. This record is
incremental; a passing subsystem test must not be represented as an end-to-end or
release qualification. Specification: `../dom-snapshot-plan.md`, including the
user-confirmed scope and acceptance matrix T01–T15.

## Independent collector/direct-pipeline review

Reviewer: `ses_eb4872ae8ffe9Sr25x2hw1N6qI`, not the implementer.
Delivered slice: `worker/product/src/dom.mjs`, direct `capture.mjs` integration
and their tests. Canonical encoding is still injectable; worker wire remains v2
and embedded runtime unchanged. Those missing parts were not scored as delivered.

Actually executed:

- Selected author source/browser suites: **23 passed, zero skipped**.
- Independent probes: **15 assertion groups passed**, including reproductions of
  defects. This is not a claim of 15 acceptance criteria passing.
- Catalog-hash-verified Node 24.21.0 / Chromium 156.0.8078.4, sandbox enabled.
- Isolated owned loopback server and private browser profile/output. Long-TMPDIR
  socket failure was resolved with a short private scratch alias, not disabled
  browser security. Browser/server cleanup confirmed; embedded fixture zero visits.

Scoped positive evidence includes recursive privacy exclusions in ordinary/open
shadow/template/SVG DOM, no reads of guarded password/live/checked/selection
properties, scalar-order attributes, Unicode/whitespace, source nonmutation,
ordinary resource-limit rejection, tall DOM-only pages, sibling failure isolation
with the publication callback, independent DOM/PNG instability and callback-before-
simulated-PNG-timeout ordering. This does **not** prove durable publication.

### Initial repairs — independently resolved

| ID | Finding and independent reproduction | State |
| --- | --- | --- |
| W1 | Excluded-subtree template/open-root node/depth accounting. | **Resolved independently**: actual 100,001-node and 600-depth probes now fail with DomLimit. |
| W2 | DOM sampling spends PNG's readiness deadline. | **Resolved independently**: all six short-budget pairs capture both channels. |
| W3 | Optional callback changes sibling failure behavior. | **Resolved independently**: without callback, DOM failure and PNG success both return. |

Independent evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-dom-snapshot-20261017/worker-verification`.
Reproduction command:

```sh
bash /var/home/olav/Data/opencode/tmp/opencode/regressee-dom-snapshot-20261017/worker-verification/reproduce.sh
```

Logs include `author-tests-shorttmp.log`, `independent-probe-final.log` and
`independent-6EuAJP/report.json`. These are private test artifacts, not shipping data.

### Expanded worker review — W1–W5 independently resolved

New independent source/browser/bundle evidence lives in `worker-verification-v2/`.
It includes 36 selected checks, 11 resource/browser/cancellation checks and 15
repaired-interface groups. Wire probes verified per-kind outcomes, metadata/hash/
permissions, actual-prefix/name handling, pixel-free handshake/DOM-only, explicit
PNG installation smoke and strict old-version/runtime rejection. The then-current
`a765…ff` bundle matched four catalogs, all seven source modules and unchanged
vendor bytes; source changes after that audit require a fresh bundle/audit.

- **W4 independently resolved:** timed-out async screenshot preparation emitted `ChannelWarning`
  after `ChannelFailed`; the tracker correctly rejects that sequence. Stalled-font
  extracted-bundle repro: 5/9 screenshot-only, 1/3 both-mode. Author lifecycle guards
  now block expired-stage emissions/writes, preserving strict tracker ordering.
  Author reports 56 Node/source/browser and 52 runtime Cargo tests passing (six
  intentional ignores), all-target/all-feature Clippy passing. Fresh independent
  verification passed all twelve real stalled-font cases, checked emissions through
  worker exit and retained DOM hashes, plus four lifecycle/collision/callback probes.
- **W5 independently resolved:** coordinator added the missing capture-events
  fixture to the development-pack list; a fresh exact-whitelist protocol subset
  passed **seven tests**, zero failures/skips. Evidence:
  `worker-verification-w5/subset-fe3s40do/{node-test.log,report.json}`.

No actual Rust supervisor or durable checkpoint crash/resume acceptance yet.

Current repaired bundle SHA-256:
`b5f900ff611bb8a7a28b5b19100675341774abadd3ada8b44ad912c8630f56a8`.
Author refreshed all four catalogs/seven source modules with vendor bytes unchanged.
Fresh independent audit matches all four catalogs/seven sources/vendor bytes on
this repaired bundle; the earlier `a765…ff` audit applies only to that earlier
candidate. Evidence: `worker-verification-w4/{bundle-audit.json,wire-qFPUQ2/report.json,
lifecycle-033v8E/report.json}`. Runtime tests do not alone prove app durability.

## Acceptance coverage at this checkpoint

| Criteria | Current evidence / remaining work |
| --- | --- |
| T01 | Core/default/legacy and nine saved/draft-mode ZIP pairs verified; final normal-umask public CLI and DOM-only native reopen reruns pending. |
| T02–T03 | Independent core/cross-language and real shipping-worker normalization, names/prefixes, templates/shadow/iframe scope probes pass. |
| T04 | Collector exclusions, semantic pre-CAS/ZIP rejection and actual report/export privacy probes pass at their recorded scopes. |
| T05 | Real Product/managed-browser 100001px DOM-only capture passes without image artifacts; normal CLI rerun pending. |
| T06 | Real app opposite verdicts, independent sibling failures, timeout/cancel/crash preservation and native corrupt-sibling IPC tests pass; final client gate pending. |
| T07 | Independent limits/instability, strict wire, semantic ingestion, corruption and cancellation probes pass; final integrated regressions pending. |
| T08 | Core/diff bounds and provenance independently verified; actual report rendering passes, native source/input/visual final gate pending. |
| T09–T10 | Storage/ZIP/history/retention independently pass, including selected-only export and same-check approvals; 42 native tests cover real-IPC independent/atomic approval; final public/native reruns pending. |
| T11 | 99 post-fix app/32 ordinary CLI tests and strict Clippy pass under 0022. Independent public consumer passes 224 CLI invocations across modes/reviews/approvals/frozen plans/sharing/failures (signed synthetic worker, actual CLI/engine). |
| T12 | Actual adversarial and three status-matrix engine reports pass independent HTTP/file Chromium checks; metadata/copy byte refusals preserve engine health. Final-render 64 MiB HTML/8 MiB Markdown refusal remains source/model-only. |
| T13 | Native production build/strict Clippy and 47 focused tests pass, including N1 actual typing/IPC save/reopen. Independent N1 rescore pending. Final pixels **BLOCKED**: recorded-History source bounds absent; only five current light/large scenes captured, no accepted complete matrix. |
| T14 | Post-fix core/storage/runtime/app focused regressions and app/CLI Clippy pass; whole integrated regression sweep pending, with pre-existing baseline shell failures retained below. |
| T15 | Final bundle/catalog audit and real Product checkpoint→app death→fresh recovery→pending-only resume pass independently; later auth repair gets a separate scoped probe. |

## Core foundation — independent scoped PASS, C1–C3 resolved

Implementation: `crates/core/src/dom_snapshot.rs` and additive domain/evidence/
archive contracts, plus `crates/core/tests/fixtures/dom-v1*`. The handoff records
56 existing unit tests, 16 new DOM contracts, two Node canonical-byte fixtures,
core Clippy with warnings denied and direct rustfmt checks passing. These are
author-run focused checks, not independent or integrated acceptance.

Fresh reviewer `ses_eb47db649ffewxlUIbw4SqI1FJ` independently reran 56 unit and
16 contract tests, all-target core Clippy and two Node goldens successfully.
Independent probes covered unsafe correctly hashed content without payload echo,
structural/UTF-8/boundary limits, mixed saved/intended slots, immutable review
sources, 45,360 diff-option combinations, cancellation and a four-million-cell
diff with about 32 MB combined input. Twelve probe groups included six passes and
six groups exposing these material issues:

| ID | Finding | State |
| --- | --- | --- |
| C1 | Logical versus exact mapped request validation. | **Resolved independently**: approved staging accepted; logical impersonation, arbitrary/missing/ambiguous mappings and altered query/fragment/final URL reject. |
| C2 | Browser names and null-namespace colon attribute identity. | **Resolved independently**: original browser cases and fresh actual-prefix fixtures round-trip; invalid prefixes reject; unprefixed golden bytes unchanged. |
| C3 | Legacy partial selective baseline frozen-source recovery. | **Resolved independently at core scope** after terminal-cycle repair: five targeted groups pass, including cycle rejection before loader calls/no mutation, older-parent acceptance and forged-chain/loader checks. Storage integration remains pending. |

Reproduction files are under approved scratch `core-verification/`, including
`independent.rs`, `run-probes.py`, `independent-results.log` and `html-names.mjs`.
The rescore independently reran 56 unit/20 contract/three Node golden tests and
core Clippy successfully; ten additional probe groups had nine passes and the
terminal-cycle counterexample. The core author then repaired that remaining edge
with 20 contract tests passing. Storage now owns the focused Cargo slot; the core
reviewer may use direct source-bound probes for the targeted recheck. These scopes
do not establish storage/application integration.

The targeted C3 recheck is now complete: direct rustc used a rebuilt source-bound
rlib containing the new guard, five groups passed and hashes stayed unchanged.
`core-verification/c3-terminal-fixed-binding.json` and
`c3-terminal-fixed-results.log` record proof. **C1–C3 resolved**, core only.

## Later worker and engine author handoffs — not acceptance

Worker author reports 52 Node/source/browser checks passing with W1–W3 regression
tests, wire 3 / worker 6.0.0, strict channel publications, detached validated output
before callbacks, pixel-independent handshake and refreshed embedded ZIP/catalogs.
Rust tests are written but not yet run; actual timeout/crash/resume durability is
not proven. Independent worker rescoring is running. Existing Phase 4 browser
fixture also lacks its required absolute DejaVu font path; no system change made.

Engine/CLI/report code is authored with wire 7 and focused paged DOM APIs, kind-
specific decisions/approvals and plaintext escaped report/source handling. Only
format/frontmatter checks have run so far. Fresh independent source reviewer
`ses_eb46b2cf8ffeP087u4Q8OTocx3` is checking security and bounds; compiled client
lifecycle and actual report-execution/resource-absence checks remain pending.

### Independent engine/report source review — FIX

Reviewer `ses_eb46b2cf8ffeP087u4Q8OTocx3` inspected source and independent byte
models without Cargo, network or app access. Compiled T09/T11/T12 acceptance is
blocked pending integration; source-only positive observations are not execution.

| ID | Finding | Assigned repair |
| --- | --- | --- |
| E1 | Screenshot terminal/agent/viewer aggregate reads fail on unrelated corrupt DOM. | E owner must use screenshot-scoped exact-run reader in screenshot-only routes. |
| E2 | Nested preview baseline/count fields can be tampered while contexts and combined baseline remain valid; misleading consent, not demonstrated wider mutation. | E and storage owners must regenerate/compare every displayed value and add tamper tests. |
| E3 | Row-only result paging permits a 169-row escape-heavy warning model above retained-job capacity, potentially failing/draining engine. | Byte-aware paging/bounded read failure without engine poisoning, with actual compiled regression. |
| E4 | DOM report diff cancellation becomes generic terminal failure. | Typed cancellation normalization; no user-string guessing or rewriting committed success. |

E2 storage source fix is delivered with nineteen nested-value tamper cases and
single-kind adapter checks. Formatting passes; **the earlier 55-test storage pass
does not cover this new fix**. Focused rerun/independent acceptance pending. E4 app
service now supplies typed `analysis_engine::Cancelled`; engine classification
repair and compiled cancellation evidence are pending.

Source-only positive findings: appropriate DOM authority/confirmation/lease/
idempotency classification, exact source identities, atomic storage delegation,
correct exit precedence, focused UTF-8/source and whole-diff byte budgets, body-free
aggregate/support data, inert report text/CSP/generated filenames and no-clobber
publication. No actual opened-report execution/resource-absence proof yet.
Private model/source hashes: `engine-verification/source_probes.py`.

Independent source/model rescore now marks **E1–E4 resolved at that scope**: scoped
reads retain exact bindings, 45 preview scalar tampers reject, the 169-row model
reconstructs over 17 byte-bounded pages (max 3,937,252 bytes), typed cancellation
spoofs remain ordinary errors and success remains success. Six probe groups passed
in `engine-verification/rescore_e1_e4.py`. Source/diff/hunk fields are unchanged;
results clients must follow `next_offset`. Compiled IPC/corruption/Prepare/Accept/
budget-health/cancel and generated-report browser acceptance still pending.

## Storage — independent scoped PASS, S1–S3 resolved

Migration/checkpoints, semantic snapshot ingestion, kind-scoped readers,
independent review/approval, legacy ancestry recovery, archive v1/v2 and retention
are delivered in `crates/storage`. Author ran **55 passing tests**, with two existing
profiling ignores, and all-target Clippy with warnings denied. App/GUI/workspace
builds were not part of that gate.

Fresh reviewer `ses_eb46764dfffebRoROhO36MZfiW` independently ran source-bound
existing tests (55 pass/two ignores) and fresh direct-rustc private probes
(14 pass/two failures). Child processes established per-kind committed-slot
survival through interruption/recovery/resume/sibling cancellation at storage
scope. Passing probes cover migration, semantic rejection before CAS, kind-scoped
corruption, exact identities, descendant/atomic approvals, selected-only ZIP,
fresh recipient IDs and session exclusion. **E2 storage consent guard passes five
independent altered-field probes on freshly compiled source.**

| ID | Reproduced defect | Status |
| --- | --- | --- |
| S1 | Historical single-page baseline with no inventory SQL row. | **Resolved independently**, absent-row-only verified fallback, read-only and fail closed on corrupt/conflicting rows. |
| S2 | Current draft mode differs from saved baseline during export. | **Resolved independently**, all nine mode pairs preserve separate frozen/future semantics. |
| S3 | New internal hints violate historical strict-v1 portable schema. | **Resolved independently**, actual old-reader compatibility, strict v1 input and lossless selected-only v2 when required. |

Latest independent rescore resolves **S1–S3**: 22 private tests plus two child
recovery executions pass; freshly direct-compiled current storage unit tests:
**60 pass, two profiling ignores**. All nine saved/draft mode pairs round-trip.
Zero-row legacy data now passes seven failed paths without mutation; corrupt or
conflicting rows/cycles reject. Actual v1 exports pass historical strict whole-
manifest types; even null/empty/default-valued newer input fields reject. Extended
provenance uses lossless v2 without unselected sibling originals. Five E2 tamper
cases remain passing. Results/bindings:
`storage-verification/{s1-s2-s3-final-results.log,v1-final-results.log,
current-storage-unit-results.log}`. Ordinary post-fix Cargo/Clippy gates remain
pending; app/UI acceptance is separate.

Reproduction: `storage-verification/run-private.py source-bound-final-recheck`;
matching results/binding and `historical-single-page-source.txt` record proof.
The original T01/T10/T14 compatibility defects are now resolved at storage scope.
Focused evidence supports T04/T07/T09, not full app/worker/UI
end-to-end acceptance. No Cargo/Clippy or repository source edits by this reviewer.

## Pre-feature baseline, not new-feature verification

Source commit: `8381759ae24e468db022864e610784a475ac3fc4`, product sources clean
when compiled. PATH uses `/var/home/olav/.cargo/bin`; private host-specific
`LIBRARY_PATH` is documented in the plan. Initial shim/linker setup failures were
environment prerequisites, not product regressions.

All-feature compilation succeeded. Reported passing suites include app unit 60,
agent control 6, analysis/review 9, engine 18, partial baseline 2, CLI workflows 19,
core 56 and GUI library 100. The synthetic 100/1000-page qualification test passed
in 702.71 seconds; no live website or release qualification is inferred.

The broad suite was interrupted after 1,131 seconds to release the compile slot.
It was **not a complete baseline pass**. Before interruption the native-shell test
binary reported these pre-feature failures:

- `changed_results_first_analysis_and_recorded_checks_default_without_resetting_refresh`
- `all_inline_disclosures_attach_content_restore_focus_and_do_not_mutate`
- `loading_feedback_has_one_owner_and_keeps_its_icon_and_copy_visible`

Further shell tests were not established. New failures must still be investigated;
the pre-existing list is not permission to dismiss unrelated regressions.

The exact baseline test binary and source/hash binding are preserved under the
private scratch `baseline/` directory documented in the plan. The original broad
log is
`/var/home/olav/.local/share/opencode/shell/90ea0b944e3e5aba5984218936aa6888d6e83ecd/sh_14b6e229f001YVeo4ECf7Ww3YC.out`.

## App integrated compilation checkpoint

Author app build/tests passed **85 app lib/new DOM tests, one ignore**, and
**20 engine/partial-baseline tests, one ignore**. Original analysis/review was
eight passes/one fixture setup failure: it deliberately changed a frozen Check
image mode. Parent adapted it to assert exact admission rejection/no baseline
mutation and retained the independent real-PNG material-mismatch gate. Rerun
pending; this is not a blanket screenshot regression pass.

Clippy needs an E-owned large-enum fix and deduplication of three pre-existing
test fixture modules (confirmed at HEAD). A single cfg-test root fixture is now
registered; E owns aliases and the focused rerun, then CLI and GUI gates follow.
No shipping fixture dependency is introduced. S1–S3 source fixes are delivered
but require fresh independent/Cargo verification.

## Compiled E gate — B1/B2 ordinary-capture fixes awaiting rerun

Passed author gates: app/CLI all-target/all-feature Clippy; app lib **75/one
ignore**; original analysis/review **9/9**; current storage **60/two profiling
ignores**; actual IPC byte-budget/engine-health and running/late cancellation
**2/2**; Python fixture **5/5**; formatting. These are not all CLI workflows.

Ordinary-umask workflows were **eight passes/15 failures**. Diagnostic private
umask yielded **22 passes/one failure**, isolating real B1/B2—not legitimizing a
workaround. E1 corruption and E2 consent cases passed diagnostically.

| ID | Defect and source repair | Status |
| --- | --- | --- |
| B1 | Capture staging was 0755; runtime correctly refuses it. New owned 0700-at-creation TempDir helper validates parent/ownership and serves both capture paths. | Source regressions/format pass; ordinary-umask public rerun pending. |
| B2 | Screenshot-only attempt checkpoint rejected DOM-only not-requested image slots. Scheduler now uses kind-aware attempt/skip append preserving settled siblings. | Source regressions/format pass; public DOM-only/resume rerun pending. |

Actual exported adversarial report for separate T12 browser proof:
`engine-report-evidence/actual-cli-dom-report-01/index.html`, with frozen preview,
successful receipt and source/hash binding. It was produced diagnostically; the
artifact is real, but does not prove the broken normal capture path. Full commands
and counts are in approved scratch `e-cargo-handoff.md`.

## Actual report-browser security — scoped PASS

Fresh independent reviewer loaded the actual engine export in sandboxed Chromium
156 via HTTP and `file://`. Six export hashes, source/review bindings and preview
fingerprint match. Adversarial source/notes render literally: no captured script/
event execution, resource attempts, active markup/forms or CSP violations. Four
downloads match bytes; eight direct source views are plaintext. Privacy warnings
are visible, seven exclusion canaries absent (not collector-redaction proof).
Browser/server closed, original exports unchanged; no remote attempts/page errors.

Evidence: `engine-report-verification/` includes `scope.md`, `reproduce.sh`,
`static-bindings-result.json`, `results.json`, `requests.json`,
`execution-sentinels.json` and both PNGs. Both-mode/failure reports, maximum-output
stress and compiled full-client acceptance remain pending; this is not complete
T12-matrix or whole-feature acceptance.

## Actual anonymous Product/Worker crash recovery — scoped PASS

Fresh independent normal-umask (`0022`) execution passed **12 real groups** through
`Product::discover` / `capture_inventory`, signed managed runtime and unmodified
supervisor/current shipping bundle—no manually inserted evidence rows. A committed
DOM/SQL pin was observed while PNG pending; SIGKILL targeted only owned app,
guardian reaped Node/Chromium, fresh process recovered Interrupted and resumed only
PNG from changed page B. DOM bytes/full metadata/hash/refs remained unchanged,
hash-valid and readable after another fresh process. B would fail any new DOM
attempt (257 attrs), proving settled DOM was not recaptured.

Timeout, cancellation and actual worker crash preserve DOM. Real 100001px DOM-only
avoids image artifacts/tool image-lazy-scroll mutations while image twin fails
ImageLimit; DOM limit preserves PNG; 404 fails both; hidden-DOM-only and canvas-
PNG-only changes produce opposite independent persisted app verdicts. Private
staging observed 0700; owned processes/browser scratch/server cleaned.

Runtime Node 24.21.0 / PW 1.64.0 / Chromium 156, sandbox enabled, owned loopback,
bundle `b5f…56a8`. Evidence/commands/exact source and binary hashes:
`live-durability-verification/final-report.md` and
`run-gieih5gi/{report.json,binding.json,independent-final-assertions.json}`.
**Later auth/B3 source is not covered**; saved-login replay, DOM-only saved-baseline
reopen/check after that repair, public network, CLI/GUI and release qualification
remain separate. Actual T15 app checkpoint/death/recovery/resume is proven at this
scope, not whole-feature acceptance.

## Native source handoff — actual GUI evidence pending

GUI D source is authored against real IPC, with saved modes, per-kind review and
consent, complete bounded source/diff pagination and image-control exclusion.
Author ran 13 source/model, 14 shell-model and three queue-model tests outside
Cargo, plus metadata/format checks. These are not integrated rendering/input tests.
Integrated GUI build, real-IPC/native workflow tests and the authored 32-capture
light/dark matrix remain pending. There is no native/visual PASS yet.

## First native compile/IPC gate — B3 remains open

All-feature native targets compile; author ran **42 focused tests**, including
real IPC root reopen/refresh with either sibling corrupt, valid-sibling approval,
same-check A→B→C independent approvals and combined consent. Kind-scoped workspace
recovery preserves immutable metadata and exposes read failures. No images yet.

DOM-only reopen panicked in auth's old first-screenshot reference lookup (**B3**).
App owner is replacing it with frozen source/saved-page metadata and updating four
auth worker requests from wire 2 to the shared wire-3 constant. BoundSession and
manifest version 2 are unrelated and stay unchanged. No session/key/origin guard
changes. DOM-only remaining cases and the two unchanged native Clippy warnings
still need focused reruns; no integrated visual acceptance is inferred.

### Post-fix app gate — 99 passing tests under normal permissions

B1/B2/B3 app regressions now pass under unchanged umask **0022**: library 79,
screenshot review 9, DOM review 8 and saved originals 3 = **99 pass, no failures,
one existing ignore**. App all-target/all-feature Clippy passes. Auth uses frozen
metadata/saved page union and wire-3 worker requests; encrypted session v2 remains
unchanged. Final ordinary CLI and native DOM-only reruns remain pending; fresh
independent actual-browser auth/reopen verification is being prepared.

### Final ordinary CLI gate — PASS, independent consumer exercise pending

Full CLI all-feature suite under unchanged **0022**: **32 pass, zero fail/ignore**,
including all 23 workflows, both IPC counterexamples and normal E1/E2 corruption/
consent checks. All-target/all-feature CLI Clippy passes. No source/API changes;
backend/E frozen. Retained report assets/bindings unchanged. Logs/commands:
`e-cli-final-handoff.md`, exact binary/test/326-source binding:
`e-cli-final-binding.json`. CLI SHA-256:
`041127702a1c424685cb3801162db141f1fa3d6e4ce3f849ba9c600cc218f5b3`.
Fresh independent small public-consumer exercise and final native gates remain.

### Report status matrix and byte preflight — independent scoped PASS

Three further actual CLI/engine exports cover both successful, image ImageLimit
with DOM changed, and DOM SerializationFailed with image changed. Exact sources,
dates/policies/selections/notes/fingerprints/hashes match; unavailable siblings are
explicit, not fabricated. Six HTTP/file browser loads, twenty plaintext views and
ten downloads pass with no captured execution/resources/CSP/page errors.
Actual metadata >8 MiB and DOM-copy >64 MiB preflight refusals publish nothing,
leave no stage, preserve engine health and permit subsequent reads. Only 174,958
report bytes and about 2.1 MB CAS were generated. Final-render 64 MiB HTML/8 MiB
Markdown refusal remains source/model-only, not an executed boundary check.
Evidence: `engine-report-verification/matrix-01/{verification-result.json,scope.md}`
plus full receipts, SDK bindings, requests and execution sentinels. No Cargo/capture
paths ran; the isolated driver used source-bound rlibs and public CLI/engine export.

## Qualification boundary

### Independent final public CLI consumer — PASS

Report `engine-verification/public-consumer-01/FINAL-CLIENT.md` records commands
and exact bindings for **224 public CLI invocations** across all three modes,
defaults, source/diff/review, kind-bound confirmations, sequential/atomic approvals,
frozen plans, v1/v2 ZIP imports and failure cases. Harness corrections are retained;
no product fixes/blockers found. Owned sessions/engines/server stopped, no Cargo or
production edits; earlier T12 artifacts unchanged. Scope: actual public CLI/engine
with a signed synthetic worker—not real-browser capture or native visual proof.

### Independent native source finding N1 — FIX

Reviewer `ses_eb421e937ffeRfnvRvQdW0EzAG` found screenshot-only mounting of
`AnalysisField(9)` disables/excludes the DOM note editor before its kind-specific
enabled branch. Changed DOM-only and unavailable-shot both-mode open a panel but
cannot type. Existing direct-`set_value` tests bypassed this user failure. GUI owner
must test real activation/focus/simulated typing and real IPC save/reopen, preserving
modal/history/agent guards. No Cargo/images/input were run by this source reviewer;
fresh repaired source/input and bound-PNG review remain required.

### Native final author gate — 47 tests pass, History pixels blocked

N1 author fix/tests pass actual user-control activation, focus/typing, dirty
close/reopen, IPC save and fresh-root exact note binding; modal/history/agent guards
remain closed. All **47 focused tests** pass, strict GUI Clippy and production
build pass. Independent reviewer is rescoring N1; author checks are not acceptance.

Recorded-History confirmation fails at `ui_dom_capture.rs:155`: no source bounds
after actual History/kind selection. Five current light 1280x800 scenes plus
calibration exist; note-editor, dark and small-screen confirmation was not reached.
Initial 32 frames are pre-repair and are not current-source acceptance. Both
failed confirmation attempts remain retained; the later change affected harness/
test only, with production bytes unchanged. No dropped assertions/fabricated DOM.
Full bound handoff: `gui-final-20261017/confirmation/HANDOFF.md`.

The planned visual repair/confirmation allowance is exhausted; independent
diagnosis uses existing source/tests/pixels only. A further edit/render round needs
new authorization. Coordinator runs bounded workspace gates against the frozen
candidate, excluding only the explicitly long pre-feature synthetic scale test;
results/source bindings go to `coordinator-final-gates/`. No whole-feature PASS.

### Later B3 actual auth/reopen verification — scoped PASS

Fresh independent source-bound build changes only `auth.rs` from the real 12-case
Product binding. DOM-only/mixed-partial baselines reopen and probe/check precisely
saved mapped page unions/frozen settings, with zero omitted/future-draft visits.
Real wire-3 `AuthState::install` replays cookie/localStorage/sessionStorage; missing
storage or ended sessions return NeedsLogin without anonymous fallback. Normal
0022, zero key-provider calls/no login window, all owned resources cleaned.
Evidence: `live-durability-verification/auth-final/final-report.md` and
`auth-final/run-05wvwzem/`, including commands and exact source/binary hashes.
Headed OpenLogin/ConfirmLogin, OS keyring and persistent encrypted restart remain
untested, not implied by this scoped result.

No new-feature native/offscreen acceptance, OS-input/clipboard, accessibility,
cross-platform installation, signing or release verdict has been issued. Future
verification must identify exact tested source/contracts, distinguish source,
simulated GPUI, actual offscreen pixels and real OS behavior, and explicitly list
blocked/unrun checks. Open material criteria prevent a full-feature PASS.
