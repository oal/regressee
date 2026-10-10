# Phase 4 — bounded P1 session-origin correction

Date: 2026-10-10. **Implemented; independent correction verdict PENDING.**
Scope: the origin-drift finding in `04-verification.md` only. No Phase 5
integration, standalone analysis-engine/report edits, nested agents or native
black-window attempts. The original verification and implementation evidence remain.

## Authorization repair

`crates/app/src/auth.rs` seals each **newly established** login in a private v2
payload containing its sorted, deduplicated canonical destination origins, access
policy and worker material. Existing AES-256-GCM authenticates this entire payload;
the original project/environment/session AAD is retained. Thus editing a mapping
cannot change the encrypted login's original authority. Both encrypted and
ephemeral sessions use this envelope. It is not project/run/export metadata or a
new plaintext file/schema. Bound buffers remain size-limited and zeroized.
Ephemeral owner metadata also retains the original project/environment/session
reference (the identity persistent state authenticates in AEAD AAD). A copied
local reference cannot read, replace/prune or forget another environment's
in-memory session, even when both environments intentionally share an origin.
The 16-session replacement limit counts only matching original scopes.

Every Product auth resolution compares the saved binding with the **current saved
environment** and the caller's environment snapshot before installation/visits.
Mismatch, legacy/incompatible payload, missing/undecryptable state or ended
ephemeral state fails closed; origin drift returns typed `NeedsLogin` with an
explicit re-login message. An existing invalid reference never falls back to
anonymous work, including after selecting Public. Discovery checks authorization
before its mapping restriction. The existing legacy single-page API continues to
refuse saved logins/changed mappings, not downgrade them.

`worker/product/src/sessions.mjs` also embeds a v1 origin binding when login is
confirmed. Installation requires that immutable material binding to match the
canonical install authority; it refuses unbound legacy state and origin-changing
reinstallation of an existing reference. Authenticated capture/discovery still
look up the exact session identity and requested origin before creating a replay
context. Cookie host rules are unchanged; they are **not** the authorization check.

Status now resolves binding validity rather than labeling any ciphertext ready.
Invalid state clears the probe receipt and displays re-login. A settings-stale
open login is stopped/reset when status is refreshed; confirmation already refuses
a changed settings snapshot. The existing GPUI Project reply clears environment/
mapping/preview state before requesting authoritative status. Service/status
assertions are not rendered GPUI or human-login qualification.

## Canonicalization, policy and compatibility choices

- URL origins include scheme, canonical hostname and effective port. Explicit
  `:80` for HTTP / `:443` for HTTPS and hostname case normalize identically;
  different effective ports or schemes do not. Rust URL serialization and worker
  URL origin normalization agree. Origin sets are sorted/deduplicated; adding or
  removing any destination origin requires a new login, even if one stays unchanged.
- Changing Public ↔ InteractiveLogin also invalidates reuse. Public is not an
  instruction to discard an existing incompatible login and visit anonymously.
  Explicit Forget plus Public is the existing deliberate public-access choice.
- Name-only edits, mapping order and **same-origin source/destination prefix** edits
  retain the login. This is an origin/session boundary, not a tenant/path-account
  boundary. Prefix changes remain explicit saved approval, subject to guarded URL
  mapping/action exclusions and a new whole-settings representative probe. They
  do not authorize redirects, new origins, logout/actions or inventory expansion.
- Other discovery/profile policies retain credentials but keep existing validation,
  frozen inventory and revision/runtime continuity checks. Login URL/IdP/representative
  edits reset setup/probe snapshots; already filtered saved state remains usable
  only at its bound capture origins. IdP approval is never capture authority.
- Refusal is non-destructive: old ciphertext/reference may remain until explicit
  Forget or successful fresh-ID re-login. Restoring the original security binding
  can use that originally bound login again, but needs a fresh probe when mappings
  require it. No drift session is rebound, migrated or rewritten to the new site.
- Existing RGS1 ciphertext containing the old unbound payload is intentionally
  **needs-login**, not silently upgraded. Cipher/header/AAD integrity failures stay
  fail-closed. No OS-key or plaintext fallback is introduced. Screenshots, active
  baselines, original CAS objects and run history are not deleted or rewritten.
  The corrected app and freshly signed binding-capable worker must be used together;
  an older cached worker's unbound confirmation is refused, not upgraded. Repair
  that trusted runtime before re-login. No renderer/profile version or baseline
  compatibility is changed merely for this private authorization correction.

## Maintained regression and serial evidence

`crates/app/tests/product_flow.rs::actual_phase4_auth_environment_and_capture`
extends the existing real signed Product/SQLite/browser flow with two separately
owned numeric-loopback ports and hit counters. It establishes the original
cookie-only encrypted login and baseline, changes that **same environment ID's**
destination, asserts typed `NeedsLogin` on probe/discovery/check, unchanged origin
hit maps (neither leaked-cookie nor anonymous destination visit), refusal after
owner restart and reset status. Direct signed-worker InstallSession drift is
`NeedsLogin`, followed by authenticated discovery `InvalidRequest` and zero visits.
Independently established staging still captures; explicit new-ID production
session at the changed destination then probes/captures successfully. Original
baseline/hash/inventory survive. Name-only edits retain login and reset the probe.

Focused auth tests cover encrypted restart, scheme/port drift, frozen-caller
authority, access-policy downgrade, encrypted legacy rejection without write,
wrong outer/inner bindings, ciphertext tamper and canonical default ports/case/
order/duplicates, plus ephemeral drift and deliberate Forget/Public handling.
The final small scope guard adds same-origin ephemeral reference-transplant and
other-environment-preserving Forget coverage.
Existing ephemeral restart/pruning and unavailable-store tests
remain. The signed Node auth suite adds cookie-only port/scheme/legacy/reinstall
and direct capture/discovery zero-destination-visit assertions, while preserving
the earlier storage, resources, redirect, readiness and cancellation suite.

Serial command (fresh directory required):

```sh
python3 scripts/check-phase4-corrections.py --output /tmp/opencode/regressee-p4-correction-UNIQUE
```

The wrapper uses four build jobs, one Rust test thread and an owned-descendant
Linux subreaper. It performs focused binding tests, then the locked build/fmt/
workspace/clippy, actual owned OS-store availability and freshly signed Node/real
browser Product flow through `scripts/check-phase4.py`. It does not signal host
processes or run a soak. Command/log/ownership records are retained separately.

Initial
`/tmp/opencode/regressee-p4-correction-5eba8f3e848d/` focused compilation **FAILED**
(exit 101, 1.566 s): a new canonicalization test borrowed its mappings twice.
The test now clones before pushing. No browser/locked batch ran in that failed
attempt. Its evidence remains; it is not relabeled a pass.
Second `/tmp/opencode/regressee-p4-correction-cb9611c3a85a/`: focused auth **3 PASS**
(1.516 s); UI build/format/workspace tests **80 PASS**, but final strict clippy
**FAILED** on two new collapsible-if style violations (whole locked batch exit
101, 12.832 s). Conditions are now collapsed; no OS-store or browser command ran
in this failed batch. Both failed outputs report zero remaining own children and
zero changes to the protected original verifier/analysis files.

Completed pre-final-scope-guard evidence:
`/tmp/opencode/regressee-p4-correction-bd403ad31709/` (`B`).

| Command / record | Exact outcome |
| --- | --- |
| `B/commands.json`: focused locked `auth::tests::`, `ui-tests`, one test thread | **4 PASS**, exit 0, 1.166 s |
| `B/handoff/commands.json`: `bash scripts/check-foundation.sh` | UI build/fmt/workspace **81 Rust PASS**, strict all-target/all-feature clippy PASS, exit 0, 10.830 s |
| same record: actual fresh owned OS-store availability | Exit 0, 0.214 s, **unavailable branch; OS persistence NOT QUALIFIED** |
| same record: actual ignored Phase 4 Product/browser regression | **PASS**, exit 0, 288.189 s; **33 signed Node tests**, zero failures/skips; complete real origin-drift/direct-worker/restart/new-session/staging/baseline regression |
| `python3 /tmp/opencode/regressee-p4-correction-audit-87eac.py B` → `B/data-runtime-audit.json` | **PASS**: 6 runs / 1 baseline / 5 analysis rows; 2 encrypted private-mode session files; 13 metadata and 6 logs checked; 6 original/derivative CAS PNGs hashed/CRC checked; Ed25519 signature + all **6,025** signed file hashes/sizes and all **10** maintained worker files checked |
| `B/ownership.json` and data/runtime audit | Zero remaining own children/live scoped processes, empty worker scratch, zero pending publications; original verifier and all analysis-owned protected files unchanged |

This successful B run is retained, not presented as final-source evidence for the
subsequent ephemeral original-scope metadata guard and added legacy-service
zero-visit assertion. The scope guard does not change persisted ciphertext or
worker schemas. Final-source checks were rerun rather than inferred:

**Final evidence (`E`): `/tmp/opencode/regressee-p4-correction-102d66451404/`.**

| Command / record | Exact final-source outcome |
| --- | --- |
| `python3 scripts/check-phase4-corrections.py --output E` → `E/commands.json` | Serial wrapper **PASS**, exit 0; focused stage 1.215 s; nested locked/OS/browser stage 355.878 s |
| `cargo test --locked -p regressee-app --features ui-tests --lib auth::tests:: -- --nocapture --test-threads=1` | **5 PASS**, no failed/skipped/ignored focused cases, including copied-reference/same-origin ephemeral isolation and Forget preservation |
| `bash scripts/check-foundation.sh` → `E/handoff/locked-ui-tests-clippy.log` | **PASS**, exit 0, 11.130 s: UI build, fmt, **82 Rust tests** (11+6+20+14+5+11+15), all-target/all-feature clippy `-D warnings` |
| `cargo test --locked -p regressee-runtime secrets::tests::actual_owned_os_store_availability -- --exact --ignored --nocapture --test-threads=1` | Exit 0, 0.214 s, **OS-store unavailable branch; persistence qualification NOT PASSED** |
| `cargo test --locked -p regressee-app --features ui-tests --test product_flow actual_phase4_auth_environment_and_capture -- --exact --ignored --nocapture --test-threads=1` | **PASS**, exit 0, 344.501 s; **33 signed Node tests**, zero failed/skipped; real two-origin Product/SQLite/session/capture regression and direct-worker/legacy-service zero-visit assertions pass |
| `python3 /tmp/opencode/regressee-p4-correction-audit-87eac.py E` → `E/data-runtime-audit.json` | **PASS**: 6 retained runs, 1 original baseline, 5 analysis rows; 2 encrypted RGS1/0600 session files in a 0700 vault; 13 metadata files / 6 logs checked for fixture secrets and local session references; 6 CAS PNG digests/CRCs verified; fresh signature, all **6,025** signed hashes/sizes and all **10** maintained worker source/fixture files verified |
| `E/ownership.json`, final data/runtime audit | **PASS**: all adopted own descendants reaped, zero remaining own children / live scoped product processes, empty worker scratch, zero pending files, all five protected original verifier/analysis inputs unchanged |

Read `E/handoff/product-flow/{phase4-report.json,auth-worker/report.json,resources/report.json}`
with the actual command records, not just exit status. The retained real resource
suite again executes cross-origin scripts/CSS/images, enforces zero forbidden
document destinations and completes signed IPC Cancel/Shutdown: worker resource
cancel **27 ms**, request/response-pause cancel **7/7 ms**. No full crawler/scale
soak, native window loop or new external network qualification was run. Original
historical zombies/processes were not targeted; the ownership claim is scoped to
this runner's descendants and runtime paths, not a globally clean host.

## Independent verifier handoff and remaining gates

Independent verdict remains **PENDING**; dependent Phase 5 auth/capture integration
still waits. Reproduce the original P1 with fresh owned origins/counters through
actual Product methods, independently test encrypted/ephemeral restart, unbound
legacy/wrongbinding/tamper, direct worker and no-anonymous fallback, canonical
default ports/scheme and other-environment preservation. Inspect encrypted-file
permissions/headers, project/run/evidence/log separation, original baseline/CAS
hashes, fresh installed signed source bytes and own-resource records. Do not touch
`crates/app/src/analysis_engine.rs` or its independent audit/handoff reports.

Test-provider encryption is **not OS-keychain evidence**. Actual OS availability,
native/human login/visual acceptance, platform ownership, enterprise networks,
qualified current public runtime/distribution and release gates remain as in the
original verification. No sandbox/TLS bypass, global install, publication, service
provisioning, sudo or broad cleanup is authorized or performed.
