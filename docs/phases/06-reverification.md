# Phase 6 independent reverification

Status: **FINAL — P2/P3 FIXED LOCALLY; bounded local Phase 6 gate PASS** (2026-10-10).

**Dependent Phase 7 may rely locally on these corrected contracts** under decisions
0005/both 0006. This verifier did not start Phase 7. Native, human, OS/platform,
public-runtime, real-keystore and soak qualification remain separately pending;
this is not release acceptance.

Evidence root: `/tmp/opencode/phase6-reverification-20261010-sol-independent`.

Scope: independent bounded local verification of the Phase 6 corrections. No application edits, nested agents, Phase 7 execution, or external/native qualification.

## Planned / completed stages

1. [x] Read the final handoff, earlier verification, decisions, implementation and assertion coverage; inventory source and historical evidence.
2. [x] Run the handoff's focused locked format/build/clippy/tests serially, retaining exact commands and results.
3. [x] Run fresh private malformed-PNG, geometry/compression, import/refusal and strict-schema probes, checking exact causes and unchanged state.
4. [x] Verify current worker/signed-pack hashes and run one complete independent recipient fixture with fresh stores/runtime/evidence and the existing 240-second bound. It passed; no failure-diagnosis/retry was needed.
5. [x] Audit persisted evidence, original bytes, privacy, resource cleanup and implementation immutability; publish the local gate verdict and update the status index.

## Completed stages

- Stage 1 settled: read the correction handoff, prior independent verdict, decisions 0005/both 0006, shared validator, storage gates and maintained assertions/recipient fixture. Captured 221 input hashes in `input-hashes.json`. Initial process scan found only its own shell, no overlapping suite. Exact historical specimen is 600×300 declaring 300 rows but inflating 301 (not 601).
- Source review: prevalidation uses the same open file and strict 32 KiB DEFLATE window before row decoding; derivatives are bounded to 512-row tiles and cancellation is checked. Supported contract is noninterlaced RGB/RGBA8 with IHDR/consecutive IDAT/IEND, sRGB and bounded tEXt; unsupported formats refuse, not convert. Selected/inherited publication, restore and export validate originals and frozen dimensions before publication. Fresh behavior probes completed below.

## Qualification boundaries

Native/human/accessibility/cross-OS/public-runtime/real-OS-keystore/soak checks are NOT RUN and separately pending. Synthetic headed readiness and owner fixture evidence will not be represented as human or native qualification.

## Runtime input checkpoint

Independent Ed25519 verification with existing OpenSSL PASS; all 6,026 signed files match both retained input ZIP and installed runtime, and all 11 maintained worker files match current source. `runtime-preflight.json` records hashes and historical DevelopmentLoopback identity. Initial Python crypto-module preflight failed before any browser/test (module unavailable); no package was installed, existing OpenSSL supplied the signature oracle. ONE final-source recipient run completed below.

## Recipient command checkpoint

ONE final-source independent recipient command **PASS**, exit 0 in **121.350 s** within unchanged **240 s** bound. No deadline or forced cleanup; 45 adopted children reaped, zero remaining children. Fresh `recipient-data/` contains both stores, installed recipient runtime, ten persisted stages and final report. Persisted-data/privacy/source assertions also passed the independent read-only audit. No recipient rerun occurred. Wrapper's generic implementer qualification label is not attribution: this run is the independent verifier's own invocation.

## Additional settled checkpoints

- `composite-restore/`: maintained immutable-composite/provenance/restore/refusal/recheck test PASS, exit 0, 0.394 s; zero remaining children.
- `fixture-private-command/`: independent assertions against the exact extracted final-source maintained Site PASS. Eight sockets deliberately held idle during a ready request: ready response <1 ms, then fixture Drop joined shutdown in 1,956 ms with sockets still held open, all reached EOF and listener refused new connections. Source has a 64-handler cap, 2 s read / 3 s write deadlines and joined shutdown; no timeout relaxation. The earlier maintained four-idle regression also passed.
- Private probe compile A failed before any behavioral probe due to accidentally selecting incompatible cached serde/anyhow artifacts. Compile B uses the project's matching dependencies, succeeds in 0.604 s; original failure/log preserved. No application source was edited.

- Focused command `core-png` settled: exit 0, 0.134s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `core-png/`.

- Focused command `core-schema` settled: exit 0, 0.090s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `core-schema/`.

- Focused command `storage` settled: exit 0, 2.500s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `storage/`.

- Focused command `storage-portability` settled: exit 0, 0.122s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `storage-portability/`.

- Focused command `services` settled: exit 0, 0.470s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `services/`.

- Focused command `auth` settled: exit 0, 0.230s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `auth/`.

- Focused command `jobs` settled: exit 0, 0.279s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `jobs/`.

- Focused command `fixture` settled: exit 0, 0.249s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `fixture/`.

- Focused command `build` settled: exit 0, 0.188s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `build/`.

- Focused command `clippy` settled: exit 0, 0.202s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `clippy/`.

- Focused command `format` settled: exit 0, 0.264s, deadline=False, forced=0, remaining children=[]. Exact command/log/result: evidence `format/`.

- Fresh independent streams/schema/geometry stage PASS: 21 malformed/unsupported/budget PNG cases × five real gate calls with exact causes and all-table/CAS/cache preservation; strict nested script refusal/all three coverage variants; five valid compression/geometry/tile probes, cancellation, both Chromium-original byte/date roundtrip and valid-image/wrong-manifest refusal. `private-streams-results.json`, `private-valid-results.json`, calls retained.

- Fresh independent legacy publication stage PASS: 22 separately manufactured hash-consistent retained cases, each refused activation/restore/export for the exact PNG/frozen-metadata cause; no destination, prior active baseline and unrelated project/all tables/CAS/cache unchanged. Generic byte-store insertion is deliberately NOT ingestion acceptance. `private-legacy-results.json`.

- Fresh independent selected/inherited gate stage PASS: excess-row and valid-image/wrong-metadata retained originals independently refuse both selected and inherited composite publication for exact cause (four cases), preserving prior active baseline/all rows/CAS/cache/unrelated project. Consistent legacy reports were explicitly manufactured after clean analysis, not counted as fresh malformed analysis acceptance. `private-selected-results.json`.

- Signed source-matched worker `worker-protocol-capture` settled: exit 0, 0.222s, deadline=False, forced=0, remaining children=[]. Fresh evidence/logs under the unique root.

- Signed source-matched worker `worker-auth-resource` settled: exit 0, 9.306s, deadline=False, forced=0, remaining children=[]. Fresh evidence/logs under the unique root.

- Persisted audit B settled **PASS, zero failures**, exit 0, 0.543 s. Both recipient stores: 21 physical SHA/byte/complete-PNG-verified artifacts, six date/metadata/baseline-pin bindings, two distinct correctly scoped private encrypted sessions. All 28 private probe stores pass SQLite integrity/foreign-key/physical-hash checks. Fresh recipient runtime matches all 6,026 signed input files; both final captures/results and full DB/report equality checked. No pending GC/staging, known session material in 23 scoped logs, live owned processes or short-worker scratch. Audit A remains failed evidence: classified orchestrator planning-checklist delta and incorrect verifier oracle comparing two equal actual metadata URLs instead of logical target → actual mapped URL; audit B corrected those oracles, not product data.

## Exact maintained results

All commands were **serial**, through `scripts/check-phase6-corrections.py`, with
separate evidence outputs. Cargo commands used Rust 1.98.1, `--locked`, and tests
used `--nocapture --test-threads=1`. Ordinary command bounds were 180 s, worker
commands 150 s, complete recipient 240 s. Full commands are in every `result.json`
and `focused-results.json` / `worker-results.json`; every success below has a
nonzero assertion/test count, not merely a successful empty filter.

| Output | Exact test filter / operation | Passed |
| --- | --- | ---: |
| `core-png` | core `complete_png_integrity_checks_have_specific_failure_oracles` | 1 |
| `core-schema` | core `discovery_coverage_strict_schema_and_all_valid_variants_roundtrip` | 1 |
| `storage` | storage `phase6_` | 4 |
| `storage-portability` | storage `portability::` fault/recovery | 3 |
| `services` | app `--features ui-tests --test analysis_review phase6_` | 2 |
| `auth` | app `--features ui-tests --lib auth::tests::` | 7 |
| `jobs` | app `--features ui-tests --bin regressee phase6_simulated_native_picker_confirmation_and_actual_jobs_dataflow` | 1 |
| `fixture` | app product_flow `phase6_fixture_speculative_connections_do_not_block_ready_requests --exact` | 1 |
| `composite-restore` | app analysis_review `immutable_composite_provenance_restore_failure_guards_and_recheck --exact` | 1 |
| `recipient-command` | app product_flow `actual_phase6_composite_zip_fresh_recipient_login_and_fixed_mapped_check --exact --ignored` | 1 |
| `worker-protocol-capture` | signed Node `protocol.test.mjs capture.test.mjs`, concurrency 1 | 9 |
| `worker-auth-resource` | signed Node `phase4.browser.test.mjs resources.browser.test.mjs`, concurrency 1 | 24 |
| `build` | `cargo build --locked --workspace --all-features --all-targets` | PASS |
| `clippy` | `cargo clippy --locked --workspace --all-features --all-targets -- -D warnings` | PASS |
| `format` | `cargo fmt --all -- --check` | PASS |

Total: **22 distinct focused Rust tests + 33 signed Node tests**; no failure,
skip or cancellation in those invoked tests. The recipient was explicitly opted
in, not counted from ordinary ignored output. Existing Cargo future-incompatibility
notice for `proc-macro-error2 2.0.1` remains a notice, not clippy failure or fixed
release-stack qualification. No Phases 0–5/full-600-second suite was repeated.

## Independent exact-cause reproductions

`probe.rs` calls the **current actual core/storage/app APIs**, linked to the checked
final-source build (`probe-linkage.json`). `private-probes.py` generates independent
PNG/ZIP bytes using Python zlib/CRC, not the maintained Rust generator. Retained
per-call JSON and per-case snapshots require all SQL tables, exact file hashes,
directories, prior active pointer and unrelated project preservation.

| Specimen | Required observed cause |
| --- | --- |
| Exact maintained `long-inflate.zip` and fresh excess row | `zlib excess decoded bytes` |
| 299 rows under a 600×300 IHDR | `zlib decoded byte count mismatch` |
| Compressed tail / concatenated zlib streams | `zlib trailing compressed bytes` |
| Adler / truncated checksum | `zlib Adler32 mismatch` / `zlib truncated stream` |
| Chunk CRC | `PNG chunk CRC mismatch` |
| IEND payload / missing IEND / post-IEND bytes | `PNG invalid IEND` / `PNG truncated chunk` / `PNG trailing bytes after IEND` |
| Invalid row filter | `PNG invalid scanline filter` |
| Valid 600×299 image claimed 600×300 | import `PNG dimensions disagree with manifest`; publication/restore/export `Original screenshot PNG dimensions conflict` |
| Nested `inventory.coverage.Limited.script` | `Malformed/unsupported baseline manifest: unknown field \`script\`` |

The original specimen declares **600×300**, inflates **722,701 bytes = 301×2,401**,
not the required 720,300 bytes; independent CRC/geometry/inflation checks confirm
those exact retained bytes. Its archive hash is
`b4f0f2404da5652c164662a19ce3d887c36637a0f76f255a50af08d20c732e0e`.

**21** malformed/unsupported/geometry cases each independently refuse core strict
validation, storage validation, ingestion, archive inspection and actual import
(including bypass with an old valid preview). **22** legacy retained cases each
refuse activation, restore and export, with absent output destination. Four extra
selected/inherited cases refuse excess rows and wrong dimensions. All application
publication calls refuse; none publishes a malformed baseline or changes prior
active pointers. To exercise pre-fix inherited evidence, two private scaffolds
explicitly manufacture corruption in a preexisting active reference via low-level
SQL; these intentionally invalid retained scaffolds are not app acceptance and
are kept only in owned evidence, not mistaken for valid recipient baselines.

Unknown nested `script` refuses exactly on inspection/import and direct enum
decode; Complete, Interrupted and Limited variants round-trip. Neither the script
string nor archive content executes. No unrelated refusal is counted as the fix.

Five valid RGB/RGBA8 probes exercise 1×1, 4096×1, 1×32768, 19×513 and 31×1025,
stored/fixed/dynamic DEFLATE, split consecutive IDAT, all five row-filter values,
sRGB/tEXt and 1/1/64/2/3 tiles. Exact original bytes/hashes survive ingestion.
Zero, over-width, over-height and >64M geometry refuse. Palette, grayscale,
16-bit, interlace and unsupported gAMA safely refuse rather than conversion.
The public contract remains this narrow supported subset, not all legal PNGs.
Pre-set cancellation refuses without state changes; row/inflate checkpoints and
bounded same-file/32 KiB/512-row behavior are also source-reviewed, not a claimed
maximum-scale memory/soak benchmark. Both valid Chromium originals independently
ingest and export/import byte-for-byte with hashes and capture dates unchanged.
A valid image alone has no claimed manifest metadata at ingest; mismatch rejection
is proved at the metadata-bearing import/publication/restore/export gates.

## Recipient evidence and honest boundaries

Ten stage durations (seconds): discovery **10.977**, owner baseline **12.638**,
owner check **12.990**, export **0.049**, new-project import **0.109**, fresh runtime
**37.223**, separate headed OpenLogin **11.070**, explicit ready **0.788**, mapped
probe **11.352**, fixed check **12.902**. Every stage settled successfully before
its original deadline; no unchanged old 240/600 s failed attempt was rerun.

The final physical three-entry ZIP contains manifest + two originals only. The
owner composite draws from TWO source runs. The recipient project/inventory/
profile/run/baseline IDs are rebound; original bytes/dates and complete capture
metadata (apart from intended ID rebinding) plus rendering provenance survive.
Actual final DB run/analysis equal the report: TWO Captured outcomes, TWO stored
captures, TWO positive-coverage NoSignificantChange reports, zero gates/warnings/
failures. Targets remain the frozen logical inventory, mapped to the separate
recipient fixture origin; historical width **600**, future project width **640**.

Import and missing-login **zero-navigation** are actual source assertions in the
executed test; the report persists their result, not independent HTTP counter
history. The owner injected fixture session is not human authentication. Recipient
uses actual separate headed Open/explicit-ready with a self-establishing synthetic
cookie and separate test vault/key. Two distinct scoped local session IDs and
private ciphertext are independently inspected. This is **not native GPUI,
human login/visual/accessibility, real OS keystore or cross-machine qualification**.
The ZIP omits sessions/cookies/private paths/runtime; known session material is
absent from scoped command logs. Private URL queries truthfully remain visible;
masks do not redact original pixels. This is scoped known-sentinel privacy checking,
not universal arbitrary-secret detection.

## Final preservation, ownership and handoff

`audit-b.json` records independent data/runtime/privacy/source checks. Application,
worker, script, corpus, original lockfiles and old verifier reports remain
byte-identical to the recorded input scope. Intentional changes are this report,
the current status index, and orchestrator-owned `docs/ACTIVE_WORK.md` /
`docs/research/phase7-entry-checklist.md` planning updates; these are classified,
not false implementation deltas. Finished source-helper/test hashes match their
historical handoff. No historical report/artifact/helper/corpus was overwritten,
no recursive backup or deletion was performed. The final closure evidence records
the immutable owned-evidence hash ledger and all settled wrapper/resource results:
`settled-evidence-sha256.json` / `final-closure.json`. **23 owned wrappers settled,
57 adopted children reaped, zero remaining children, no deadline/forced cleanup,
no live owned processes and empty short-worker namespace.** The private compile-A
and audit-A helper failures remain failures, not added to passing test totals.
All 221 recorded source/doc/lock/corpus inputs are checked; only the four classified
current documentation/index files differ. This is bounded recorded-input hashing,
not a claim to hash every historical artifact tree. The frozen evidence ledger
retains probe results, store bytes and failed logs; runtime bytes are separately
covered by the complete signed-manifest hash audit.

Local P2 PNG-integrity and P3 schema findings are **FIXED**; complete final-source
recipient gate is now independently **PASS**. No new material local blocker was
found. **Phase 7 may rely locally on the exercised corrected contracts**, subject
to the existing user authorization. No Phase 7 implementation started here.

Separately **PENDING / NOT RUN**: native rendering/input/dialogs/human/accessibility,
actual human authentication, real OS-keystore restart/lock, supported clean machine
and Windows/macOS/cross-OS ownership, current public runtime/signing/hosting,
maximum-scale/soak and physical exhaustion/power-loss. Historical Node22.20.0 /
worker4.0.0 / Playwright1.56.1 / Chromium141.0.7390.37 remains **literal-loopback-only
DevelopmentLoopback**. Phase 5 tiny-mask-border P3 remains open. These limits are
not blanket local-development halts and are not silently converted to passes.
