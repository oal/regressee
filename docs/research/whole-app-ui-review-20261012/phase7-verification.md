# Phase 7 — independent verification

**2026-10-13 · base `135d3eb` · independent verifier.**

## Verdict

**PASS — IMPLEMENTATION + FEASIBLE SOURCE/CONTROLLER/STORAGE/OFFSCREEN GATE.**
No newly reproduced functional blocker requires implementation remediation.
**Native/accessibility/usability/release acceptance is NOT PASSED:** the separate
blockers and not-tested platforms below remain real, not waived by this verdict.
No product edits, new render, aesthetic loop, commit, push, tag or release by this
verifier. The implementation session remains `ses_ecdd56411ffeArzBSzG14zigDB` if
a finite functional remediation is needed.

Read the complete `PLAN.md`, whole-app plan and workflow blueprint, Phase 7 report,
platform matrix/receipt, actual changed/new source and tests, and relevant
storage/controller/worker owners. Loaded existing Impeccable context once; desktop
GPUI requirements, not the launcher's mobile-platform inference, govern this review.
Original critique A's 24/40 is not rescored.

## Independent execution

Full locked/offline nine-command matrix: `python3 scripts/phase7-checks.py`.
Independent launcher log:
`/var/home/olav/Data/opencode/tmp/opencode/phase7-independent-checks.log`.
All commands use the repository's `.runtime/link-libs` and approved `TMPDIR`;
ignored tests remain ignored. Waited for the already launched process with a
blocking Linux pidfd readiness wait, not a result-polling loop or redundant run.
**All nine independent commands exit 0.** Full commands, environment, elapsed times,
98 current-source-start bindings and unfiltered logs:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase7-checks-82rqditj/`.
All 98 Rust source hashes still equal current files after completion.

| Independent command | Passed | Failed | Ignored | Elapsed |
|---|---:|---:|---:|---:|
| `cargo test --locked --offline --workspace --all-features` | 358 | 0 | 45 | 564.908s |
| `cargo test --locked --offline --workspace --no-default-features` | 177 | 0 | 14 | 548.635s |
| `cargo check --locked --offline -p regressee-app --features ui --bin regressee` | — | — | — | 0.214s |
| `cargo test --locked --offline -p regressee-app --features ui --bin regressee` | 22 | 0 | 0 | 0.214s |
| `cargo test --locked --offline -p regressee-app --features ui-tests --bin regressee` | 120 | 0 | 1 | 8.426s |
| `cargo test --locked --offline -p regressee-app --features ui-inspection --bin regressee --no-run` | build only | — | — | 0.214s |
| `cargo clippy --locked --offline --workspace --all-features --all-targets` | check only | — | — | 0.264s |
| `cargo fmt --all -- --check` | check only | — | — | 0.765s |
| `git diff --check` | check only | — | — | 0.008s |

Counts sum workspace integration/doc-suite result lines and overlap between
feature configurations; do not add them into a unique-test total. The distinct
100/1,000-page test actually executes and passes in **both workspace runs**.
The real filtered Jobs/controller test passes in all-features and UI-tests.
Ignored browser/runtime/native/offscreen opt-in tests were not executed by this
matrix, not silently counted as passes. Existing source-bound supplemental capture
results are checked separately below.

Clippy exits 0 **with warnings**, without a new `-D warnings` policy: core OR/range
pattern; task manual parity; main type complexity and collapsible `if`; the new
bulk fixture's constant `chunks_exact_mut` suggestion at both compiled locations;
two vendored GPUI `f32` literal fallbacks; proc-macro-error2 future incompatibility.
Full warning locations and duplicate notices remain in `clippy-all.log`.

Additional independent serial suites, all exit 0:

| Suite | Passed | Failed | Ignored | Command elapsed |
|---|---:|---:|---:|---:|
| `phase4_review_tests::` | 8 | 0 | 0 | 2.467s |
| `workflow_refinement_tests::` | 18 | 0 | 0 | 3.419s |
| `phase5_repair_tests::` | 5 | 0 | 0 | 12.080s |
| `--test analysis_review` | 7 | 0 | 0 | 2.067s |

These use `cargo test --locked --offline -p regressee-app --all-features`, with
`--bin regressee <filter>` for the first three, and `-- --test-threads=1`.
Logs/commands/results:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase7-independent-targeted-n9cijgze/`.
Counts overlap the workspace run, not extra unique tests.

Maintained Node adapter/protocol boundary tests independently pass **9/0/0**.
Copied current `worker/product` to an isolated root, linked the already installed
development dependency, ran `node --test test/capture.test.mjs test/protocol.test.mjs`.
No browser/network/install. Node 22.20.0; retained Playwright 1.56.1, **not** the
production-requested 1.64.0 tuple. Source hashes, command and full log:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-independent-worker-52li41e_/`.

Independent production `cargo build --locked --offline -p regressee-app --features
ui --bin regressee` exits 0. Full log: approved parent's
`phase7-independent-native-build.log`. Vendored GPUI float-fallback and
proc-macro-error2 future-incompatibility warnings remain; not warning-free.

## Distinct actual storage evidence — not presentation clones

Independently inspected both retained owner/recipient SQLite databases, CAS files,
run/analysis/baseline records and ZIPs under:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase7-qualification-gzmro59k/`.

| Actual retained dataset | 100 pages | 1,000 pages |
|---|---:|---:|
| Distinct PageIds and requested URLs | 100 | 1,000 |
| Distinct baseline original PNG hashes | 100 | 1,000 |
| Distinct successful check originals | 99 | 999 |
| Real analysis Changed / No significant change / Could not capture | 50 / 49 / 1 | 500 / 499 / 1 |
| Retained baseline versions | 3 | 3 |
| Final selected replacements / exact inherited references | 20 / 80 | 20 / 980 |
| Owner CAS hashes/lengths independently checked | 456 | 4,506 |
| ZIP entries | 101 | 1,001 |

All four databases pass SQLite integrity and foreign-key checks. Both check gaps
are actually persisted `Failed { NeedsLogin }`, not a cleared viewer. Independently
decoded **199 + 1,999 original PNGs**: every actual source image encodes its own
page number in pixels, and check centers have the expected changed/unchanged
colors. Hash uniqueness is not just different ancillary PNG metadata. Final
unselected references equal their original complete reference records. Independently
checked all three retained maps at each size: exactly **0, 1 and 20 replacements**,
with every inherited reference exactly unchanged in both accepted versions.
Read-only audit outputs in the approved parent:
`phase7-independent-store-audit.json`, `phase7-independent-pixel-identity.json`.

New tests execute the real Store analysis/preview/accept/restore/export/import
paths: single and 20 updates at each size, corruption after preview rejection,
stale active-baseline rejection, explicit restoration, older baseline immutability,
retained/shared source deletion protection and byte-exact imported originals.
The both-window Jobs test additionally retains the inspected identity and two
selected identities outside a no-match filter, previews/cancels, traverses all 100
references, resets cached opening scroll and commits only by explicit worker action.

**Limits:** generated 96×64, one-profile synthetic data is storage/identity/safety
qualification, not real-site captures, 100×3/1,000×3 browser throughput, production
runtime installation or PM review efficiency. The bulk fixtures contain no saved
sessions: their session-reference assertion alone is vacuous as secret exclusion
proof. Independently rerun `analysis_review` supplies the nonvacuous complement:
saved session ID, private login sentinel and private log are absent from export;
a real ignored rectangle survives import while originals remain byte-identical;
private URL queries remain intentionally included with the privacy warning.

## Whole-workflow and source semantics

| Gate | Independent disposition |
|---|---|
| Public/private setup, local save → explicit discovery/login boundaries | Current setup/save-race/controller suites; no simulated readiness promoted to real authentication |
| Advanced capability reachability | Both-size current-clip mounted/enabled/focus/dispatch checks; startup-bound TextInput/TimezoneSelect; actual timezone changes; Custom replacement is exactly 1300×700, then removable |
| Modes, notes/masks, filters and collapse | All four analysis/original modes plus Before/After; persistent target/profile/historical environment, shared manual transform, retained note/selection/mask drafts and presentation-only store equality |
| Missing/failed/incompatible/stale approval | Actual storage/context and genuine failed-read regressions fail closed; display revision prevents A→B→A revival; selected/run/project/active-baseline identity checked |
| Original screenshot/mask/privacy contract | Explicit mask Apply versus Save, exact original-coordinate geometry and unchanged original descriptors; nonvacuous composite archive byte checks; ignored areas are NOT redaction |
| History/shared deletion/cleanup/recovery | Read-only original-policy inspection and stale replies, last loaded/retained record owner, shared references, zero-impact actions and truthful operation/recovery routing |
| Worker ownership/bounds | Database/filesystem/process/image work remains in Jobs/blocking services; virtualized queues, bounded History/consent pages and disclosure owners retained |

The source-run snapshot changes are operation-local, one-extra-run bounds; no
cross-preview cache or trust shortcut. Fresh preview and artifact verification run
again on acceptance. **Report wording correction:** `accept_selected_in_context`
performs its fresh preview/artifact checks **before** opening the SQL transaction
(`crates/storage/src/review.rs:517–566`), not “under its transaction.” The transaction
then rechecks project revision, active baseline and displayed analysis identity
before writing/pinning. Corruption-after-preview and stale-context tests pass.
Do not interpret this as a new atomic filesystem snapshot or throughput guarantee;
accept/restore still contain repeated per-reference source reads.

Production `ui` intentionally does not activate GPUI test support. The repaired
`all(test, feature="ui-tests")` gates are consistent with Cargo feature intent;
`ui-tests` owns simulated controllers and `ui-inspection` has a separate no-run
build. No native input claim follows from any of them.

## Supplemental captures, actual inner geometry and historical freshness

No new capture was rendered. Independently read **six existing confirmation
contact sheets and twelve confirmation originals**, together as one evidence
review. Originals: both-size diagnostics tail, suppression-switch tail, analysis
fields 0 and 8, both reference-99 tails, and narrow single/filtered-two consent.
All nine fields appear across the sheets and have source-bound per-state complete
field plus actual inner-input assertions. Intermediate fields were not all read
as individual full-resolution originals.

Confirmation contains 28 pairs at 1280×800/820×600; initial 28 remain preserved.
Diagnostics is 2,744px content whose end, including exact policy/mask descriptor,
is reached within the actual **358×496 / 358×278** disclosure owners. This is tail
reachability, not whole-content simultaneous visibility. The suppression label
wraps inside its **270×40** inner copy / **326×66** checkbox. Consent consequences
are visible on initial and cached opening. Reference 99 is **726×133 at (277,433)**
inside **726×365 at (277,201)**, and **706×133 at (57,333)** inside **706×365 at
(57,101)**. Complete last-row path/time/profile/paint and separator are contained;
the partial previous row at the top is ordinary scroll clipping. Pinned cancel/
confirm remain outside the reading flow. Temporary editors disable decisions
while evidence is occluded.

The unchanged mode/note/setup/history/failure pixels remain **historical
capture-time evidence**, not fresh renders of all current sources. Checked relevant
unchanged owners and current regressions; fresh Phase 7 evidence covers the named
consent-opening/shared-checkbox changes. No historical FIX was rewritten to PASS.
Phase 4's 22 repair PNG/JSON pairs and Phase 5's 243 local evidence files retain
their frozen hashes; current DESIGN/source hashes are not substituted into old
receipts.

## Integrity caveat — preserve, do not silently refreeze

Before any independent database reads, the exact command
`python3 scripts/phase7-evidence-receipt.py <qualification-root> --checks
<regressee-phase7-checks-9_07m0rr>` **passed**: 12 archives, 182 original PNGs,
68 historical Phase 6 pairs, 28+28 Phase 7 pairs and final implementation matrix.
Afterward independently checked **113 source bindings, 8,328 frozen files, 15
reused reports and all 56 Phase 7 pair/metadata/gallery/matrix/sheet bindings**.

Two volatile files now differ: `storage-{100,1000}/recipient/data/regressee.sqlite3-shm`.
The verifier's SQLite `mode=ro` integrity reads initialized WAL shared-memory
headers; read-only SQLite can write these coordination files. All database/WAL,
CAS/ZIP/PNG/source/report bytes still match. No restoration or receipt rewrite was
performed. Later decoding used `mode=ro&immutable=1`. New independent logs/native
roots also expand the receipt collector's broad glob: re-running whole collection
is not exact frozen-receipt equality. Use the frozen-entry checks and retain this
disclosed verifier-induced SHM delta. Audit: approved parent's
`phase7-independent-integrity.json`. This is not a product evidence-loss defect.

## Native acceptance and external qualifications — separate outcomes

Independent owned-process attempts reproduce the actual failures, without visiting
sites or typing into another application:

- Default X11/Vulkan: launch exits 101 with `ERROR_INITIALIZATION_FAILED`.
- Lavapipe X11: owned 1280×800 window remains `map_state=0`; guarded input refused.
- Lavapipe Wayland: alive after eight seconds; actual AT-SPI initialization succeeds,
  one desktop exists, **owned PID is not registered**; no OS events sent.

First three probe roots: `regressee-phase7-native-{gtb65a1e,vhcc8o5a,mc4_8kve}`
under the approved parent. These bind binary `2996c340…` from the all-features
workspace build, not the production-ui binary. After the independent production-ui
build, a separate owned Wayland probe at `regressee-phase7-native-0ram27_c` binds
**`5b0ff0d4ea8a31007cea96e2afd95e369e37b5f58b1a9003382fedc43b16af6a`** and
reproduces the same live/unregistered-PID blocker. Children are waited/terminated,
not left running. Probe script exit 0 means a receipt was written, **not acceptance**.
Full traces: `phase7-independent-native-attempts.json` and each root's `native.json`
and `launch.log`.

| Acceptance gate | Actual outcome / next requirement |
|---|---|
| Linux OS pointer/keyboard/focus/shortcuts/resize/clipboard/IME tasks | **BLOCKED / NOT TESTED**: no viewable qualified X11 surface; no permitted Wayland task automation. Simulated callbacks are not OS delivery |
| Linux accessibility | **BLOCKED integration**: absent AT-SPI bridge/registration is a source/framework deficiency, not merely absent Orca. Requires native accessibility engineering and subsequent real AT tasks |
| Windows/NVDA | **NOT TESTED**, no Windows desktop/runner/NVDA; OS input, appearance, AT, runtime/package remain unqualified |
| macOS/VoiceOver | **NOT TESTED**, no macOS desktop/runner/VoiceOver; host conventions, AT, signing/runtime/package remain unqualified |
| OS text scaling/high contrast/reduced motion | **NOT TESTED as host behavior**; fixed logical typography/light tokens and explicit motion override do not supply automatic OS-preference integration |
| PM task usability | **NOT TESTED: zero participants**; requires observed representative non-developer and non-implementer authenticated tasks. No task-success/time-savings claim |
| Production runtime/clean-machine distribution/release | **BLOCKED / NOT QUALIFIED**; development dependency/boundary tests do not qualify the production tuple, signatures, OS packages, licenses/support policy or downloadable assets |

External desktops/participants cannot be manufactured by source tests; the Linux
AT and host-preference integration gaps also require engineering, not an external
qualification waiver. Q1 remains partly qualified, not closed native acceptance.

**Next:** parent reviews this final bounded PASS and handles verified commit/push.
Subsequently perform the user-authorized
GitHub pipeline repair/release work, recording actual run IDs/assets and limitations.
Build availability must never be called native usability or cross-platform acceptance.
