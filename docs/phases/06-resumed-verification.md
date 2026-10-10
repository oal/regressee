# Phase 6 — resumed independent verification

Date: 2026-10-10. **FINAL bounded independent verdict: local gate OPEN (P2).**
All owned checks are settled; final classified preservation/resource audit PASS.
The complete recipient acceptance flow remains unproven, not silently skipped.
Verifier: independent GPT-6.1 Sol High session, resumed after repeated server
restarts. No implementation edits, nested agents or Phase 7 work.

## Current local dependency verdict

**Do not treat Phase 6 as locally accepted yet.** A material PNG-integrity defect
is independently reproduced: an original declaring 600×300 pixels but inflating
to 301 rows passes storage validation, ingestion, ZIP import and baseline restore.
The existing analysis engine rejects the identical bytes with
`before PNG: zlib excess decoded bytes`. This requires bounded correction and
independent recheck before dependent Phase 7 reliance. External qualification is
not the reason for this local hold.

**Phase 7 cannot safely proceed as dependent local implementation on the current
Phase 6 gate.** Use a separate bounded repair and independent recheck for the P2;
also resolve/reproduce the complete recipient check before calling portability
accepted. This verifier did not start repair or Phase 7. Existing passing services
and Phases 0–5 evidence do not need wholesale repetition.

Locations: `crates/storage/src/images.rs:104–114` and `:161`; import uses this gate
in `crates/storage/src/portability.rs`. The core analysis engine already documents
why `png::Reader::finish()` alone is insufficient at
`crates/core/src/analysis_engine.rs:1077–1079`.

### Reproduction and impact — P2, OPEN

`O/hostile/long-inflate.zip` has internally consistent artifact SHA-256, byte
counts, PNG chunk CRCs and manifest bindings. Its RGBA original declares 600×300
but inflates to **722,701 bytes**, versus **720,300** required bytes. Inspection
and import accept it, retain the malformed original and publish an active
baseline. `E/invalid-restore.json` proves restore accepts it too.

`E/png-gap.rs` independently calls actual storage validation, ingestion and the
existing strict core analysis against those exact original bytes.
`E/png-gap.log` records `Ok((600, 300))`, successful ingestion, and the exact
analysis error above. The existing core implementation explicitly anticipated
this PNG-library behavior; Phase 6's separate storage gate regresses that contract.
This is an unusable active-reference/integrity acceptance, **not** a demonstrated
arbitrary-code-execution or filesystem escape. Require complete bounded PNG/
zlib validation consistently at import, ingestion, activation/restore and export,
then recheck excess rows, trailing compressed data, checksum failures and valid
browser originals atomically. No corrective code was changed here.

Reproduce in a fresh owned store with the retained verifier probe:

```sh
printf '%s' '{"action":"import","root":"<fresh-owned-root>","path":"/tmp/opencode/regressee-p6-recovery-f013b87b4eca/hostile/long-inflate.zip"}' | /tmp/opencode/regressee-p6-recovery-f013b87b4eca/probe
```

Then use that returned project/active-baseline ID for the probe's `restore`
action; `E/additional.py` contains the full exact reproduction and strict-analysis
oracle. Do not point the probe at existing user or historical stores.

A second, narrower schema mismatch is reproduced: unknown fields inside
`inventory.coverage.Limited` are silently discarded, despite the documented
strict schema. `crates/core/src/domain.rs:503–508` lacks unknown-field refusal.
An added `script` field is accepted and dropped, **not executed**; no code
execution or credential disclosure is claimed.
This is **P3, OPEN**: `O/hostile/unknown-coverage-field.zip` and
`O/reproduced-defects.json` show both the input field and saved coverage without
it. Correct the advertised nested schema contract; this finding alone is not
the material Phase 7 dependency blocker.

## Evidence and attribution

- `I = /tmp/opencode/regressee-p6-independent-f5e83c04c514/`
- `O = /tmp/opencode/regressee-p6-recovery-f013b87b4eca/`
- `E = /tmp/opencode/regressee-p6-resumed-fe180c8d1dc8/`

`I/input-hashes.json` records 209 source/lock/script/document/corpus inputs. All
209 were independently rehashed unchanged on initial resumption; the snapshot is copied
to `E/input-hashes.json`. `O/recovery-input-hashes.json` separately preserves four
earlier verifier evidence hashes. These are independent verifier runs, **not**
the implementer evidence under `regressee-phase6-sol-*`. The generic qualification
string in the reused maintained wrapper does not change this attribution.

## Settled coverage retained — do not repeat

| Evidence | Actual result |
| --- | --- |
| `I/maintained/locked-ui-build-test-clippy-fmt.log` | Independent PASS: UI build, 120 ordinary locked Rust tests, clippy/fmt. Includes actual simulated-GPUI Phase 6 Jobs confirmation and three storage recovery/fault tests. Ignored opt-in tests are not passes. |
| `I/maintained/signed-safety/` and signed safety log | Independent PASS: actual Rust auth Product flow plus 33 signed Node tests; existing origin/Forget/resource guards retained. |
| `I/maintained/signed-phase5-product.log` | Independent PASS: actual Phase 5 Product flow plus one signed Node replacement test. |
| `O/hostile-results.json`, `hostile-calls.json`, `hostile.py` | 56 independent archive cases: 54 safe refusals, two unexpectedly accepted inputs above. Two refusal-message expectations differ (`bzip2`, nil profile), but actual errors identify unsupported compression/profile. No generic unrelated error is counted as a security pass. |
| `O/reproduced-defects.json` | Both accepted inputs independently imported in separate stores, preserving unrelated recipient project rows. |
| `O/cleanup-continue-outer.log`, `cleanup-results.json`, `cleanup-calls.json` | Previously missing cleanup tail PASS: baseline activation/restore, explicit inactive-baseline/run deletion, low-space preservation, stale repair refusal, dead/live stages and separate complete-orphan deletion. |
| `O/corpus-remaining-owner.json`, `format-remaining-owner.json` | Read-only corpus and format PASS. |
| `E/cleanup-prefix-results.json`, `cleanup-prefix-calls.json` | Fresh missing cleanup prefix PASS: active/retained protection, independent actual-byte oracle, stale revision and newly shared graph refusal, confirmed shared-object preservation. Three exclusive objects total 522,891 bytes before another real project acquires them. |
| `E/png-gap.log`, `png-gap.rs`, `invalid-restore.json` | Fresh incremental PNG defect proof PASS: storage validates/ingests bad PNG, strict analysis rejects exact bytes; fresh import and restore both accept it. |

The archive matrix checks raw duplicates, path/case collisions, symlinks/special
types/executable permissions/compression, encoded/manifest/object/count/expansion/
reconstructed-target budgets, raw directory/header bounds, schema/version/source/
reference/profile binding, correctly hashed malformed PNGs, stale archive and
cancellation. Refused import cases compare all SQLite rows and physical CAS hashes
and check staging removal. See the scripts/results for each exact cause.

## Failures / interrupted checks — not passes

- Phase 6 actual recipient flow failed twice with `worker failure: Timeout`:
  `I/maintained/signed-phase6-recipient.log` (386.45 s test) and
  `O/recipient-recheck-outer.log` (84.00 s). The first preserved complete owner
  composite/export/import and reached an interrupted recipient check; the second
  stopped during owner capture. These do not close the complete recipient gate.
- Old cleanup prefix failed due to the verifier's synthetic clone helper; its
  corrected probe B and the bounded fresh prefix above passed. Failure retained.
- `O/ui-remaining-outer.log` records a 300 s extra-GPUI build timeout; its process
  owner settled/reaped the exact adopted children. No extra UI assertions ran.
- `E/ui-probe-compile.log` / `recipient-probe-compile.log` record verifier-only
  compile failures (missing explicit app linkage / interrupted dependency rlibs).
  They do not invalidate the completed maintained GPUI pass or qualify the new
  extra assertions.
- `E/targeted-cargo-no-run.log` / `commands.json`: recovered actual result is a
  300 s deadline, exit −15, two terminated children reaped, zero remaining children.
  Its exact wrapper/Cargo PIDs 3439261/3439343 were confirmed absent. The server
  cancellation notification was not the actual process result; no exit-0 claim.
- Final direct execution of the exact original recipient executable via
  `E/recipient-direct.py` **settled at its 240 s deadline**, exit −15 after
  254.570 s including owned cleanup. `E/commands.json` records reaped descendants
  and zero remaining children. It preserved owner/export/recipient data but no
  final `phase6-report.json`; complete recipient acceptance remains unproven.
  The scoped executable/root scan on recovery found no surviving owned checks.
  Do not rerun these completed attempts.

Persisted stage evidence distinguishes the recipient attempts: the original
independent run has a separate recipient local-session reference, two imported
origins and an interrupted actual check, so it reached separate fixture-login /
mapping/probe and the source's zero-navigation assertions before failing. The
final direct attempt has complete owner composite/export/import but **zero
recipient session references and no recipient check**, consistent with stopping
during explicit recipient setup. Neither has a complete final report or clean
two-target recipient analysis. No exact root cause beyond the recorded worker
timeouts / outer deadline is proven; no security pass is inferred from failure.

The retained independent maintained GPUI test actually calls Shell picker-completion
paths, explicit export, import inspection/Cancel/Confirm, History and Settings via
Jobs, with authoritative new-project/provenance/active-delete assertions
(`crates/app/src/main.rs:4234–4324`). Source inspection independently confirms
SQLite/archive/image/GC ownership in the spawned Jobs thread
(`crates/app/src/jobs.rs:247–318,679–719`), not the UI thread. Those storage actions
work without a configured browser distribution. This is actual simulated GPUI /
actor coverage, **not** native dialogs, rendered pixels, responsive maximum-scale
timing, accessibility or human acceptance. The extra private UI test never compiled
and is not counted.

## Final independent preservation / privacy / ownership audit

`E/final-audit.json` completed after all tests settled. Its data/privacy/resource
checks passed: 14 SQLite stores, 108 physical artifact records and 38 baseline
bindings; two normal composite ZIPs preserved exact originals/dates/rebound
metadata/frozen profiles; two installed loopback runtime manifests/source checks;
30 logs scanned; private session/ZIP modes; no live owned processes or short-worker
scratch. Complete recipient reports are absent, not inferred passed.

That first audit exited 1 because its blanket preservation oracle flagged
concurrent documentation changes. `docs/ACTIVE_WORK.md:11–14` attributes README's
recovery update to the orchestrator. Observed metadata changes are `docs/phases`
(this new report), `docs/research` (separately owned preparation handoff), and
`docs/ACTIVE_WORK.md` (orchestrator recovery notes). No implementation source,
prior phase report or corpus hash changed. First audit output stays preserved.
The classified audit **`E/final-audit-b.json` PASS, zero failures**, completed after
the index update: of 209 original hashes only the intentional index update and
documented orchestrator README update differ. All **104 crate/worker/script hashes**
and original lockfiles remain identical. The **68,649 historical metadata entries**
show only the explicitly classified documentation/index/directory changes above;
this is a metadata preservation check, not a claim of hashing every historical
artifact. Original protected phase-report and corpus content hashes are unchanged.

Both normal ZIPs contain exactly a manifest and two original hash-named PNGs.
Independent CRC/inflation/hash/byte checks, source/recipient file equality and
complete reference rebinding preserve dates, every capture metadata field and
historical width 600 separately from future width 640. Manifests exclude the
scoped source/recipient session references, fixture credential, local session and
runtime paths. Private queries deliberately remain visible. The 30-log scan is
for these known fixture secrets/references, not universal secret detection.
Fourteen SQLite databases pass integrity/foreign-key/schema-v8 checks; 108 physical
artifact records and 38 baseline bindings pass SHA/bytes/reference/active-parent
checks, with zero pending GC. The deliberately accepted malformed PNG specimens
remain retained as defect evidence, not mislabeled valid image acceptance.

Two installed runtime generations match every signed-manifest file's SHA/bytes
and the checked maintained worker source/test files. Historical identity is
Node 22.20.0 / worker 4.0.0 / Playwright 1.56.1 / Chromium 141.0.7390.37,
**DevelopmentLoopback only**; this does not qualify current public distribution.
All checked local-session files and normal export ZIPs have no group/other
permissions. Final process scan finds **zero owned executable candidates**;
short-worker scratch is empty, and every recovered process-owner record reports
zero remaining children. No unrelated process was signaled, no old PID was reused
or killed, and no unfinished test is handed off.

The only retained import stage is `O/cleanup/recipient/data/.import-staging-live`,
the old live-owner-preservation fixture, now dead. It is intentional historical
test evidence, not an active import; it was not swept or deleted. No new unexpected
pending publication/import path was found.

## Remaining acceptance gates

- **Local P2 correction/recheck:** complete PNG integrity before active publication
  and restore; preserve atomic refusal and valid original/reference/privacy guards.
- **Complete actual local recipient flow:** current independent attempts do not
  prove the final separately authenticated fixed mapped two-target clean check.
- **P3 schema contract:** reject or explicitly document nested unknown-field handling.

Native rendered/input/human/accessibility, another supported clean machine and
cross-OS recipient, real OS-keystore, current supported public runtime/signing/
distribution, maximum-scale/soak and physical disk-exhaustion/power-loss remain
**PENDING / NOT RUN**, not blanket local-development halts under decision 0005.
The tested historical runtime is **DevelopmentLoopback only**. Masks do not redact
raw screenshot pixels; private query strings remain visible. Phase 5's tiny-mask
border P3 remains open. No release or Phase 7 implementation is authorized here.
