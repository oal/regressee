# Phase 4 — bounded Forget ownership correction

Date: 2026-10-10. GPT-6.1 Sol High implementer. One bounded P2 only; no nested
agents, Phase 5 integration or analysis changes. **Delivered for fresh independent
verification, not independently accepted.** Original origin-isolation P1 remains
fixed locally under `04-reverification.md`; that report and its evidence are retained.

Evidence root (`E`): `/tmp/opencode/regressee-p4-forget-fc7b23e40b7d/`.

## Repair and outcomes

- `crates/runtime/src/secrets.rs`: encrypted deletion now successfully decrypts
  with project/environment/session identity AAD **before unlinking ciphertext**.
  UUID existence, a private file's shape and reference owner claims do not authorize
  revocation. The decrypted temporary `SessionMaterial` is dropped/zeroized.
  Cipher format, key provider, installation identity and access/origin envelope
  are unchanged; no plaintext persistence or key deletion was added.
- `crates/app/src/auth.rs`: removal receives only a reference's session ID plus
  the operation's actual project/environment. Forget derives these from the loaded
  project and selected saved environment; replacement/rollback/pruning derive them
  from the established login's actual loaded environment. It reconstructs identity
  scope, not caller-supplied owner fields. Ephemeral removal checks that scope
  against its trusted in-memory owner before revocation.
- Ownership and access validity are deliberately separate. Forget does **not** call
  `auth_state` or require the current origin/access binding to match: the legitimate
  owner can forget expired, legacy/incompatible, or changed-origin state provided
  the immutable identity is authenticated. Replay still requires the original P1
  origin/access checks, unchanged by this repair.
- Copied encrypted references return `SecretError::Corrupt`, exactly **“Saved login
  could not be decrypted. Keep existing evidence and log in again.”** Both rows and
  ciphertext remain. AEAD cannot distinguish wrong ownership from wrong key or
  damaged ciphertext; no unauthenticated owner metadata was introduced to guess.
- Copied ephemeral references return `SecretError::ScopeMismatch`, exactly
  **“Saved login belongs to another environment. Log in again.”** Both the borrowed
  reference and the original in-memory owner remain. This is intentionally stricter
  than the prior successful-looking foreign ephemeral local-reference unlink.
- Locked/unavailable keystore returns `SecureStoreUnavailable`, preserving ciphertext
  and reference. Corruption also fails closed. If ciphertext is already missing, or
  ephemeral state ended, only the selected owner's stale local reference is removed;
  there is no foreign private state to revoke. Shared installation keys are never
  deleted by Forget. Full OS persistence remains unqualified on this host.
- Fresh-ID replacement still saves the new login and prunes authenticated old state.
  If a borrowed/corrupt old reference cannot authenticate, the new login remains
  saved/usable and the existing explicit error reports **“New login saved, but
  superseded private state could not be removed”**. The foreign owner is preserved.
  This is not a claim all replacement cleanup succeeded.

The actual Jobs `Work::ForgetLogin` delegates to Product and forwards errors through
`Reply::ProductFailure`; GPUI renders the reply and existing environment status.
Those service/UI paths were inspected, not redesigned or claimed human-qualified.
User-facing semantics are documented in `docs/auth-and-capture.md`.

## Maintained regression coverage

`crates/app/src/auth.rs` exercises both modes with same-origin wrong-environment
and wrong-project references, direct forged-owner-field controls, original owner
replay/reference/ciphertext preservation, foreign replacement pruning, valid owner
Forget after origin/access change, unrelated sessions, missing-state recovery and
locked/corrupt fail-closed behavior. Existing P1 restart/legacy/tamper/normalization
and ephemeral restart checks remain. `crates/runtime/src/secrets.rs` adds actual
AEAD deletion controls for wrong project/environment, unavailable key, tampering,
valid deletion and unrelated ciphertext/replay preservation.

`crates/app/tests/product_flow.rs` extends the existing serial actual signed-worker
fixture, without removing its original P1 assertions: a valid A session is probed,
B copies its reference, B probe/Forget refuse with exact reasons and **no additional
HTTP hits**, A's bytes/reference remain and A probes again successfully. This runs
wrong-project/wrong-environment controls in encrypted-test-provider and ephemeral
modes. Actual server-expired access is refused before owner Forget succeeds; a
fresh session then gets origin/access drift and owner Forget still succeeds without
visits. Existing unrelated ciphertext/references, serialized original baseline/run
and original CAS reference are retained. `forget-report.json` records exact reasons
and before/after per-path hit counters, not secret/reference material.

## Serial checks and audit

- Focused vault: **3 PASS, 1 intentionally ignored** real-OS check; 0.865 s command
  wall time. `E/focused-vault.log`, `E/focused-commands.json`.
- Focused auth: **7 PASS**, 1.767 s command wall time. `E/focused-forget.log`.
- Full locked foundation: **85 PASS** (13+6+20+15+5+11+15), with 11 deliberate
  ignored checks listed rather than counted as passes; UI build, format and strict
  all-target/all-feature clippy PASS; exit 0, **12.132 s**.
  `E/maintained/handoff/locked-ui-tests-clippy.log`.
- Actual fresh owned OS availability: exit 0, **0.214 s**, **unavailable branch**:
  “OS secure store unavailable: EPHEMERAL required; persistence qualification NOT
  PASSED”. This is not successful OS persistence/locked-store/restart qualification.
- Actual ignored Product auth/capture/Forget regression: **1 PASS**, exit 0,
  **371.288 s** command wall time. Its fresh signed Node suite: **33 PASS, zero
  failed/skipped**, 9.522 s reported Node duration. Original P1 assertions are retained.
  `E/maintained/handoff/{commands.json,signed-auth-capture.log}` and
  `E/maintained/handoff/product-flow/signed-worker-suite.log`.
- Existing serial wrapper completed naturally, **exit 0**: focused stage 0.264 s;
  nested locked/OS/browser stage **383.706 s**. No overlapping browser suite, native
  black-window loop, security/sandbox bypass, service or publication was started.
  `E/maintained/{commands.json,ownership.json}`.

The initial delivery correctly listed the signed stage as still running; this
settled handoff supersedes that provisional running status. Its one owned wrapper
was PID **3048864**, command `python3 scripts/check-phase4-corrections.py --output
E/maintained`. Completion was awaited via a bounded **pidfd exit event**, without
sleeping/polling suite status; the shell completion notification then confirmed
exit 0. No test cancellation or rerun was needed.

### Exact fresh Forget results

`E/maintained/handoff/product-flow/forget-report.json` records all per-path maps:

| Actual Product control | Production fixture total hits | Staging fixture total hits | Outcome |
| --- | --- | --- | --- |
| Encrypted copied ID, wrong environment | 4→4 | 18→18 | Probe NeedsLogin; Forget Corrupt with exact reason above; both references and A bytes preserved |
| Encrypted copied ID, wrong project | 4→4 | 18→18 | Same refusal/preservation; A subsequently probes successfully |
| Ephemeral copied ID, wrong environment | 4→4 | 21→21 | Probe NeedsLogin with “Ephemeral login belongs to another environment. Log in again.”; Forget ScopeMismatch; A memory/reference preserved |
| Ephemeral copied ID, wrong project | 4→4 | 21→21 | Same refusal/preservation; A subsequently probes successfully |
| Encrypted owner, expired and separately origin/access-changed | 4→4 | 20→20 | Both legitimate Forget operations succeed without visits |
| Ephemeral owner, expired and separately origin/access-changed | 4→4 | 23→23 | Both legitimate Forget operations succeed without visits |

Totals include earlier fixture setup (including the single runtime download);
expiry rejection and positive A replay visits occur deliberately **before** the
owner-Forget counters. Expiry returns typed NeedsLogin, exact message **“worker
failure: NeedsLogin”**. Changed-origin/access replay returns the original exact P1
binding-mismatch reason before visits. Ciphertext equality is byte-for-byte (and
therefore digest equality), not only file existence. Key use remains available to
A's successful post-refusal replay; Forget has no installation-key deletion path.

### Settled preservation and owned cleanup

The scoped audit prepared during implementation is now complete:
`E/final-audit.json`, `E/settlement-{resource-entry,exit-event,final}.json`.
No new unrelated audit or independent report was created/overwritten.

- Original serialized baseline/run and CAS reference remain identical across the
  Forget controls; two unrelated saved-session ciphertexts and references remain
  identical. Six retained CAS PNGs pass SHA-256/CRC/EOF/decompression checks.
- One isolated fixture DB, 13 metadata files and 8 logs pass private-token/reference
  checks. Only local-session rows retain references; two retained ciphertext files
  have RGS1 headers/0600 mode and the vault is 0700. This is not full export/import
  qualification or a plaintext persistence fallback.
- Ed25519 signature and **6,025 signed files / 10 maintained worker/test files**
  verify. Historical Node 22.20.0 / Playwright 1.56.1 / Chromium 141.0.7390.37 /
  worker 4.0.0 remain DevelopmentLoopback with the normal sandbox, not public-qualified.
- **32,124 entry-fingerprinted workspace inputs** have only the five expected scoped
  existing-file changes (three Rust files, auth help and status index); the new
  correction report is the only new repository file. Analysis work and all historical
  phase reports remain unchanged. **540 prior independent evidence files** retain
  their original hashes; no recursive history copying or deletion occurred.
- The wrapper reaped **98 of its own descendants**. All **89 owned PIDs** observed
  in the settlement snapshot, including the wrapper and temporarily adopted exited
  browser children, are now absent. **Zero remaining own children, live scoped
  product processes, managed worker scratch entries or pending files**. Fixture
  listeners close through their owned test lifecycle. No host-wide zombie cleanup,
  unowned signals, broad deletion or cancellation was attempted; evidence is retained.

## Fresh verifier handoff

Run serially into a **new unique** `/tmp/opencode` output:

```sh
cargo test --locked -p regressee-runtime secrets::tests:: -- --nocapture --test-threads=1
python3 scripts/check-phase4-corrections.py --output /tmp/opencode/NEW-UNIQUE-P4-FORGET
```

Use the normal local link-library configuration (`.runtime/link-libs`) and
`TMPDIR=/tmp/opencode`; the maintained Python runner supplies it. Independently
repeat Product Forget with actual selected B context and copied A ID for same
project and foreign project; also test forged claims, unavailable key, ephemeral
mode, owner expired/changed-origin deletion, unrelated preservation and fresh-ID
replacement cleanup. Check A remains usable after B refusal, not just file existence.
Retain exact P1 zero-request drift/restart/direct-worker/new-login controls and
original baseline/history. Do not replace historical independent reports or run
native loops, services, public fixtures, global installation or broad cleanup.

**Serial implementer handoff is settled; no owned suite/resource remains running.**
**Dependent Phase 5 auth/capture/session integration remains on hold until a fresh
independent verifier accepts this bounded P2.** Independent unwired analysis work
and its audit remain untouched. Real OS-key persistence/restart/locked-store recovery,
rendered GPUI/human login/visual approval, Windows/macOS/clean machines, enterprise
network/SSO, current public runtime/distribution/release, full export/import,
integrated Phase 5 and 1,000×3 soak are still pending/not run, not inferred passes.
