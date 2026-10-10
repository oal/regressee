# Phase 6 — selected-baseline interchange and manual storage safety

Date: 2026-10-10. **Bounded P2 PNG/P3 schema corrections delivered; fresh
independent correction reverification PENDING. All implementation checks settled.**
Native/human/accessibility, real OS-keystore, clean-machine/cross-OS, public-runtime,
maximum-scale and release qualification remain open. No Phase 7, release claim,
nested agent or new native-window attempt is part of this work.

Phase 5's bounded review/storage dependency gate passed independently in
`05-reverification.md`. Its **P3 tiny-mask minimum-border footprint remains OPEN**:
correct PNG coordinates/origin, not exact sub-four-viewport-pixel outline. Prior
auth/origin/Forget, redirect/resource, scratch and immutable-history contracts
remain prerequisites, not displaced by portability.

## Delivered

### Current bounded correction disposition

`06-resumed-verification.md` is the preserved independent finding, not superseded
by an implementer acceptance claim. `06-corrections.md` records the fresh repair:
shared complete CRC/chunk/IDAT/zlib/Adler/exact-row validation before storage
ingestion/import/activation/selected publication/restore/export; strict nested
coverage schema; exact-specimen atomicity and unchanged-original regressions.
Unsupported palette/grayscale/16-bit/ancillary formats now fail closed consistently
with analysis, documented in the archive contract, without rewriting/deleting data.

A reproduced serial fixture idle-socket starvation was corrected with bounded
concurrent handlers. ONE actual complete composite/new-store/separate recipient
headed synthetic-login-ready/mapped frozen two-target check passed in **125.865 s**
within the unchanged 240 s deadline; both clean analyses, original width 600 and
future width 640 are retained. This is not human/native GPUI/OS-key/cross-OS approval.
Final focused checks: **22 distinct Rust + 33 Node**, locked all-target/all-feature
build/clippy/fmt. Final source's extra restore/export metadata dimension guards are
covered by focused service/Jobs regressions; the complete browser confirmation
preceded these narrow guards. Fresh independent final-source recipient and repair
recheck is still required. No Phase 7 reliance/implementation is authorized here.

The original implementation evidence below is historical; use the corrections
report and its unique evidence root for the current verifier handoff. Do not
repeat the old full wrapper as a substitute for focused correction verification.

| Area | Maintained behavior |
| --- | --- |
| Selected/composite export | `crates/storage/src/portability.rs`: complete displayed-baseline reference map, selected/inherited original PNGs and only necessary frozen source metadata. Current future project settings stay separate from historical profiles/discovery. Export assembly caches source metadata and distinct selected originals, retaining the existing baseline/source integrity gate; unrelated derivatives/history/database payloads are not exported. Export is explicit and no-clobber. |
| Privacy/schema | `crates/core/src/archive.rs`: version 1 strict portable types cannot carry local sessions/secret references. Explicit preview warns **ignored regions DO NOT redact raw screenshots**, confidential pixels and private URL queries remain visible. Replacement diagnostics are a strict numeric summary, not arbitrary logs. |
| Safe import | Private bounded snapshot/staging; exact archive-preview hash recheck; raw ZIP-directory/local-header validation before library deduplication; complete size/hash/PNG checks and derivative preparation before atomic graph publication. Fresh IDs, safe name conflicts and separate `imported_origins` provenance; no automatic website navigation/execution or credential/runtime setup. |
| Shared image gate | `crates/storage/src/images.rs`: bounded complete original PNG decoding and thumbnail/tile reconstruction, shared with app ingestion. Activation/restore validate originals; missing/corrupt evidence cannot establish a valid new active baseline. |
| History/storage | `crates/storage/src/retention.rs`: 64-row paged runs/versions, dates/profiles/rendering/imported origins/active reference state, actual deduplicated original/derived/shared/exclusive project bytes. Missing files count zero physical bytes, not healthy evidence. Database/WAL, private login, runtime caches and recovery stages are separate. |
| Manual deletion | Fresh authoritative impact confirmation, writer-locked graph/revision/file recheck. Active/retained baseline references, parent links, check dependencies and pair-bound mask history remain protected. A source run needed by any retained composite is refused with an explanation. Shared objects survive. |
| Recovery/low disk | Private sync/no-clobber CAS writes, Unix preflight plus 128 MiB reserve and during-ingestion checks. Explicit temporary/dead-owner-import cleanup is separate from confirmed deletion of complete unreferenced recovery evidence. Startup only finishes prior explicitly authorized `pending_gc`, rechecking references; never automatic scan-and-GC. Unsupported space-preflight platforms fail closed. |
| Actual GPUI integration | `crates/app/src/main.rs`, `jobs.rs`: native picker completion paths, preview/Confirm/Cancel, import notices, History and Settings actions use bounded actual Jobs; archive/SQLite/filesystem/image/GC work stays off the UI thread and does not require valid runtime configuration. No visual redesign or decorative raster. |

Archive schema, limits and user workflow: `docs/portable-baseline-archive.md`.
Surface/finish contract: `docs/portability-and-storage-surface.md`. The existing
proposed working-desk system is preserved. Native rendered/finish diagnosis is
separately owned; `docs/research/native-render-qualification.md` was not edited
or reproduced. Simulated GPUI actions do not qualify platform dialogs or pixels.

## Local verification

Evidence root (`E`): `/tmp/opencode/regressee-phase6-sol-f6b49801c595/`.
Reproducible wrapper: `scripts/check-phase6.py --output <fresh-unique-directory>`.
It runs locked UI/build/Rust/clippy/fmt, fresh development-signed auth/crawl/resource
safety, actual Phase 5 Product, actual Phase 6 recipient, read-only corpus and final
format **serially**. Local supervisor: `E/run-serial.py`, scoped lock, one-hour outer
deadline, Linux subreaper and owned-process accounting. No dependency install,
public-site capture, sandbox/TLS bypass, publication or broad cleanup.

Final `E/serial-a/commands.json`: **all six stages exit 0**, no overlap. The
completion notification was awaited before auditing and delivery.

| Final evidence | Result |
| --- | --- |
| `serial-a/locked-ui-build-test-clippy-fmt.log` | PASS, 22.199 s: UI build, fmt, **120 locked Rust tests** (13 + 10 + 7 + 39 + 15 + 5 + 11 + 20), all-feature/all-target clippy `-D warnings`. Thirteen ordinary-batch opt-in tests are ignored, not counted as passed. Includes the six additional Phase 6 service/fault/simulated-GPUI tests. |
| `serial-a/signed-safety/{commands.json,signed-auth-capture.log}` | PASS, outer 356.836 s: **one actual authenticated Rust Product flow + 33 freshly signed Node tests**, no failures/skips. Retained origin/session/Forget, fixed mapping/expiry/restart, document/resource denial/loading and cancellation controls remain covered. Explicit injected-key fixture, not real OS-store qualification. |
| `serial-a/signed-phase5-product.log` | PASS, 108.962 s: **one actual Rust Product flow + one signed Node replacement test**. Actual Chromium analysis/selective composite/recheck/history/restore and replacement-only geometry/readiness/activation contracts pass. |
| `serial-a/signed-phase6-recipient.log` | PASS, 109.912 s: **one actual Rust Product recipient flow**. Two original PNGs from two source runs match exact imported hashes/dates/profiles/rendering; import and missing-login paths make zero website requests; separate recipient runtime verification and explicit fresh fixture login/mapping/probe precede the exact two-target NoSignificantChange check. Owner history is unchanged. |
| `serial-a/readonly-independent-corpus.log` | PASS, 0.114 s, read-only independent 13-case corpus validation; corpus/evidence hashes unchanged. |
| `serial-a/final-format.log` | PASS, 0.314 s. Existing `proc-macro-error2 2.0.1` future-incompatibility notice remains; no dependency upgrade or release qualification is implied. |

Total: **120 ordinary locked Rust + 34 signed Node + three opt-in actual Rust
Product flows PASS**, plus UI build/clippy/fmt and corpus. These are implementer
checks, not a distinct verifier's acceptance.

### Freshness, preservation, privacy and settled ownership

`E/audit.py` → `E/{final-audit.json,final-audit-a.log}`: **PASS**, no failures.

- All **95 verification source/lock/script hashes** remain identical. This snapshot
  was taken during the final suite, before documentation completion; no subsequent
  code edits were made. It is not a whole-workspace preimplementation snapshot.
- **57,316 protected historical documentation/artifact/corpus entries** were
  checked. Only the two intentional Phase 6 checklist/index documentation updates
  differ; no unexpected overwrite. The separately owned native diagnostic was
  excluded and not touched. No claim of an unrecorded private-directory snapshot.
- Four fresh SQLite stores pass integrity/foreign-key checks, with **45 verified
  physical artifacts and 12 exact baseline reference bindings**, all schema v8,
  zero pending GC. Phase 6 owner: one project, two runs, two baselines, one analysis;
  recipient: one project, three runs, one baseline, one analysis, two marked
  imported origins. Each has its own separate local fixture-session reference.
- The actual ZIP has exactly `manifest.json` plus two hash-named original PNGs,
  two source origins and two reference bindings. Original SHA-256/bytes match;
  no fixture credential/local session-reference/private-path sentinel appears in
  the payload. The private query is deliberately visible, matching the warning.
  All **nine fresh logs** pass the scoped fixture-secret/reference scan; this is
  not a claim of universal arbitrary-secret detection. Private Phase 6 session
  directories/files and ZIP have no group/other permissions.
- Both separate installed runtime generations match their signed manifests and
  all **11 maintained worker source/test/fixture files**. Identity: Node **22.20.0**,
  worker **4.0.0**, Playwright **1.56.1**, Chromium **141.0.7390.37**;
  **DevelopmentLoopback only**, not qualified public distribution.
- `E/serial-a-supervisor.json`: exit 0 in **598.396 s**, **182 reparented descendants
  reaped**, no deadline breach, no forced cleanup and zero remaining descendants.
  Audit finds **zero remaining owned executable candidates, scratch entries,
  pending publications or import stages**. Product image caches are empty except
  the two deliberately retained signed runtime caches. No broad deletion or
  signaling was performed. No own background suite is handed off.

### What the tests exercise

- `analysis_review::phase6_real_composite_roundtrip_historical_settings_privacy_and_shared_deletion`:
  actual Store/analysis/approval services, composite inherited originals, portable
  masks, changed future settings, secret/log exclusion, identifier/name rebinding,
  exact metadata/hash/date/original preservation, active/source protection and
  stale/shared deletion plus physical byte accounting.
- `analysis_review::phase6_malicious_zip_schema_images_duplicates_and_control_paths_are_atomic`:
  26 generated path/schema/executable/bomb/image cases plus raw duplicate hidden by
  zip 2.4, case collision, symlink, changed archive, cancellation, overwrite and
  unsafe destination controls. Malformed PNG cases carry consistent hashes/sizes,
  so they test image validation rather than only hash mismatch. Existing recipient
  projects/baselines remain intact and failed staging is cleaned.
- Three storage fault/recovery tests inject disk-full/interrupted publication after
  a complete synced object, verify database rollback/reopen/source preservation,
  explicit orphan retention, stale cleanup refusal, dead/live stages, corrupt
  objects and GC reference rechecks. This is **fault injection**, not a physical
  disk-exhaustion or power-loss qualification.
- `tests::phase6_simulated_native_picker_confirmation_and_actual_jobs_dataflow`:
  actual owner/Jobs with simulated platform-returned paths; export preview writes
  nothing, explicit export works, import inspection/Cancel leaves one project,
  confirmed import creates a new project without browser work, provenance is shown,
  active deletion is refused and Settings cleanup goes through actual services.
- `actual_phase6_composite_zip_fresh_recipient_login_and_fixed_mapped_check`:
  two owned loopback origins, fresh owner/recipient stores and separate explicitly
  injected encrypted fixture keys/sessions. Export an actual selected/inherited
  authenticated Chromium composite; inspect every imported original/profile/hash/
  date/provenance; retain future width 640 separately from historical width 600;
  zero-navigation import/missing-login checks; separately download/verify recipient
  runtime, establish recipient fixture session, explicitly probe mapping and check
  the two fixed targets against the imported composite. **Not actual human login,
  OS-keystore or another physical machine.**

### Preserved intermediate failures

`E/focused-storage-a.log` records the read-only fixture `..` path rejection;
the bounded regular-file source read was corrected and focused B passed.
`E/locked-rust-a.log` records test-only key-provider/serialization compile errors;
B records the extra History reply in the old Phase 5 test initialization. The
helper now explicitly finishes that reply, without weakening its assertions;
full C passed. `E/clippy-{a,b,c}.log` preserve collapsible condition, mid-file test
module and test tuple complexity findings; aliases/order/style were corrected.
These failed attempts are not relabeled passes or removed.

## Qualification limits

- **Fresh independent Phase 6 integration/security/data/GPUI verification PENDING.**
- Other supported clean-machine recipient and Windows/macOS workflows NOT RUN;
  local Unix free-space behavior is not qualified on unsupported platforms.
- Native dialog/input/rendering, human usability and accessibility remain pending;
  the isolated native diagnostician's results are separate, not inferred here.
- Fresh browser flows use DevelopmentLoopback worker 4 and synthetic fixture
  sessions. Real OS-store persistence, public distribution/signing/hosting,
  enterprise proxy/trust/CA/VPN and release qualification remain open.
- 1 GiB ZIP, 8 MiB manifest, 10,000 originals, 128 MiB/object and 30,000
  reconstructed source-target limits are deliberate refusals, not scale passes.
  Maximum-size CPU/RSS/free-space and actual disk-full/power-loss qualification
  are NOT RUN; the historical 1,000-page/3,000-target soak stays unqualified.
- Sharing is not encryption, redaction or sender authenticity. Imported origins
  are portable claims, not independently captured local evidence. Pair-bound masks
  require explicit revalidation; corrupt originals require recovery, not invented
  replacements. Retained parent/check/mask dependencies can intentionally refuse
  deletion. No automatic retention policy or hidden evidence deletion exists.

## Independent-verifier handoff

Use a fresh unique directory and serial execution with an outer deadline/process
owner; do not run overlapping browser suites:

```sh
env PATH=/path/to/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH \
  python3 scripts/check-phase6.py --output /tmp/opencode/regressee-phase6-verifier-UNIQUE
```

Inspect all commands/completion logs and signed safety/auth/Forget/resource reports,
`phase5-product/{phase5-report.json,replacement-product.json}` and
`phase6-product/{phase6-report.json,selected-baseline.zip}`. Independently inspect
ZIP privacy/raw-directory failure cases; immutable frozen metadata and rebinding;
SQLite/reference/PNG/hash preservation; stale and shared deletion; interrupted
publication/staging/confirmed GC; Jobs/GPUI actions and off-thread ownership. Keep
the prior zero-request/auth/redirect/resource/scratch guards. Audit fresh sessions,
logs, temporary files and exact owned processes; report native/human/OS results
separately. Do not regenerate the corpus, mutate historical evidence, edit/retry
the isolated native diagnostic, spawn nested agents or perform broad cleanup.
