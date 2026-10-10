# Phase 4 — independent Forget ownership verification

Date: 2026-10-10. Fresh independent GPT-6.1 Sol High verifier, bounded to the
copied-reference P2 correction. No implementation edits, nested agents, Phase 5
integration, native black-window attempts or repeated historical audits.

## Verdict

**P2 copied-reference Forget: FIXED LOCALLY. Original origin-transfer P1 remains
FIXED LOCALLY. No material local blocker was found in this bounded recheck.**
Dependent **local Phase 5 auth/capture/session integration may now proceed** under
decision 0005 and both 0006 records. This is not full Phase 4 acceptance, native
approval, real OS-keystore qualification or permission to publish/use public sites.

Evidence root (`E`): `/tmp/opencode/regressee-p4-forget-verifier-eda7cfe98e53/`.
Historical `04-reverification.md`, `04-forget-corrections.md`, prior evidence and
the standalone/unwired analysis implementation/audit are preserved.

## Actual ownership boundary inspected

- `crates/runtime/src/secrets.rs:174–179,219–244`: AES-GCM identity AAD is
  project/environment/session. Delete now successfully loads/decrypts with that
  identity **before unlinking**. The RGS1 header contains no editable owner claim;
  nonce/ciphertext/tag are not an alternative deletion authority. Temporary
  decrypted `SessionMaterial` is dropped/zeroized. Forget never deletes a shared key.
- `crates/app/src/auth.rs:173–202,204–268,565–575`: only the session **ID** and
  storage mode come from the local row. Removal reconstructs scope from the loaded
  project and selected saved environment, including replacement/pruning/rollback.
  Ephemeral removal compares this with its trusted in-memory owner. Access/origin
  validity is deliberately not required for legitimate owner Forget.
- `crates/storage/src/lib.rs:210–228,259–308,406–415` and
  `crates/core/src/domain.rs:139–159,299–325`: rows reconstruct reference owner
  fields from their lookup context; loaded project ID must match its database key,
  duplicate/nil environment IDs are rejected, and persisted non-null session claims
  in project JSON are rejected. UUID/file presence alone does not authenticate state.
- `crates/app/src/jobs.rs:347–349,507` delegates `Work::ForgetLogin(ProjectId)` to
  the same Product service and forwards errors through `Reply::ProductFailure`.
  `crates/app/src/main.rs:267–275,293–328,1036–1056` passes only the project ID and
  displays failures/status. Direct **Product::forget_login** is itself guarded;
  bypassing the UI does not bypass ownership. Jobs/GPUI routing was inspected, not
  independently qualified as rendered/human UI or an injected-provider actor test.

## Fresh independent actual Product/SQLite/AEAD probes

`E/probe.rs` links the freshly maintained actual crate artifacts; dependency hashes
and exact rustc/execute commands are retained. Its successful second attempt uses
fresh isolated databases and owned listeners A `http://127.0.0.1:45055`, B
`http://127.0.0.1:35383`, with a constructor-injected provider. It creates an actual
authenticated one-target baseline in **each** storage mode, not a synthetic run.
`E/attempt2/independent-report.json` records **26 successful control records**.

### Exact copied-reference controls

| Mode / actual selected borrower context | Probe result | Forget result | HTTP totals during refusal |
| --- | --- | --- | --- |
| Encrypted, wrong environment | NeedsLogin; decryption reason below | Corrupt; same reason | A 4→4, B 0→0 |
| Encrypted, wrong project | NeedsLogin; decryption reason below | Corrupt; same reason | A 5→5, B 0→0 |
| Ephemeral, wrong environment | NeedsLogin; ephemeral reason below | ScopeMismatch; owner reason below | A 16→16, B 1→1 |
| Ephemeral, wrong project | NeedsLogin; ephemeral reason below | ScopeMismatch; owner reason below | A 17→17, B 1→1 |

Exact messages:

- Corrupt: **“Saved login could not be decrypted. Keep existing evidence and log in again.”**
- Ephemeral replay: **“Ephemeral login belongs to another environment. Log in again.”**
- ScopeMismatch: **“Saved login belongs to another environment. Log in again.”**

Both A and borrower references remain identical after refusal. Encrypted A remains
byte-identical, SHA-256
`b715962f7d02981d8e87ff5aa7184507872a30b1acb62721b585cb1c7c8eb448`.
Unrelated ciphertext/reference and test installation identity remain identical.
After **each** copied-reference refusal, A successfully probes again through the
actual signed worker with the original cookie. Ephemeral preservation is confirmed
by original-owner replay/reference checks, not a claimed direct heap-byte hash.
The earlier B request is a deliberate **fresh intended-B fixture establishment**
after encrypted owner Forget, not old-reference replay. No unauthorized or anonymous
home-page request occurs; all refusal counters are unchanged.

### Caller claims and stored ownership/header mutations

These are executable fresh probes, beyond reading the maintained assertions:

- Inject A's complete reference/owner fields into saved project JSON: actual Forget
  rejects **“Saved sessions cannot come from project/evidence JSON”** (record includes
  serde line/column), preserving A. Replace a local UUID with an owner JSON object:
  **“invalid length: found 147”**, before removal.
- Change stored foreign project config to claim A's project ID: exact
  **“stored project ID mismatch”**. Duplicate A's environment identity within its
  project: exact **“duplicate or empty environment ID”**. Both preserve A.
- A foreign project with a saved environment UUID equal to A's still cannot revoke
  A: **Corrupt / ScopeMismatch**, respectively. Project identity remains required.
- Independently mutate RGS1 **header, nonce, ciphertext and tag**. Both selected B
  and actual A Forget return **Corrupt**, retaining the altered bytes and references;
  no visits occur. Restoring only the verifier's own original fixture bytes restores
  successful authenticated A probing. The malformed file is not silently deleted.
- Encrypt an A-owned legacy/incompatible payload that falsely claims B ownership
  **inside** its plaintext. B Product Forget and direct `EncryptedSessions::delete`
  with B scope both return **Corrupt**, preserving ciphertext. Actual A Forget
  succeeds without visits despite incompatible replay. Authentication is the
  identity AAD, not reference/header/payload owner text.

The maintained fresh auth regression additionally executes its private removal
path with a forged A reference ID but actual B project/environment context in both
modes; it cannot bypass the same guards. These controls cover static copied/corrupt
machine-local references and stored claims, not an OS adversary able to delete files
directly or race arbitrary same-user filesystem mutations.

### Legitimate owners, pruning, keys and P1

- Foreign-reference replacement reports exactly **“New login saved, but superseded
  private state could not be removed”**. A's ciphertext/reference/use survive;
  B's fresh reference is saved and really probes successfully. Subsequent legitimate
  replacement prunes its own predecessor; owning Product Forget succeeds.
- Fresh independent origin **and** access drift returns exact typed NeedsLogin:
  **“Saved login does not match this environment's origins or access policy. Log in
  again before visits.”** Probe, discovery and actual fixed-baseline capture all
  refuse; encrypted restart also refuses. Counters: encrypted **A 11→11/B 0→0**,
  ephemeral **A 19→19/B 1→1**. Valid owner Forget then succeeds with those same
  counters. No old-cookie transfer or anonymous fallback.
- The fresh maintained actual Product regression separately induces server-side
  expiry: exact **“worker failure: NeedsLogin”**; owning Forget succeeds afterward,
  with production **4→4**, staging **20→20 encrypted / 23→23 ephemeral**. Changed-
  origin/access owner Forget also succeeds with those unchanged counts.
- An independently unavailable injected provider returns **SecureStoreUnavailable**
  in owner and borrower contexts, with ciphertext/rows retained and no visits or
  plaintext fallback. Fresh maintained missing-state/ephemeral-ended recovery and
  locked/corrupt controls pass. Legacy replay refuses without migration; authenticated
  original identity still authorizes legitimate legacy Forget.
- Unrelated state remains byte/reference-identical and successfully probes **after**
  A Forget. The same test key still decrypts unrelated ciphertext and encrypts fresh
  state for another successful probe. An explicitly synthetic private installation-
  identity sentinel stays byte-identical. These are injected-key/identity controls,
  **not** measurements of a real OS keychain entry. Source has no key-revocation path.
- Both independent original serialized runs/baselines remain identical. Maintained
  original run/baseline/CAS reference and two unrelated sessions are also preserved.

## Serial commands and honest outcomes

| Command / evidence | Result |
| --- | --- |
| `cargo test --locked -p regressee-runtime secrets::tests:: -- --nocapture --test-threads=1` | **3 PASS**, 1 deliberately ignored OS check; 0.164 s |
| `python3 scripts/check-phase4-corrections.py --output E/maintained` | Exit 0; **380.393 s**; focused auth **7 PASS**, then serial nested checks |
| Nested `bash scripts/check-foundation.sh` | UI build/fmt/strict all-target/all-feature clippy PASS; **85 locked Rust PASS** (13+6+20+15+5+11+15), 11 deliberately ignored; 6.172 s |
| Nested fresh owned OS-store availability check | Exit 0, 0.214 s; **unavailable branch, persistence NOT QUALIFIED** |
| Nested actual ignored Product auth/capture/Forget regression | **1 PASS**, 373.592 s; signed Node **33 PASS**, 0 failed/skipped, 9.428 s Node duration |
| `python3 E/run-independent.py`, initial attempt | Compile PASS, 1.066 s; probe exit **101**, 95.342 s: verifier requested a fixed check without establishing a baseline, so the error was not NeedsLogin. Not a product defect or security pass; original logs/database/resources retained |
| Same verifier harness corrected; fresh `E/attempt2` data | Compile PASS, 0.815 s; probe **exit 0**, 261.416 s, **26 control records**; actual baselines make fixed-check P1 assertions applicable |
| `python3 E/audit.py` | Fresh-evidence scoped secret/source/resource audit PASS; no historical/full runtime audit repeated |

All maintained and independent tests run serially. The existing signed maintained
suite retains original P1 drift/restart/direct-worker/new-login assertions. No old
suite result is substituted for a fresh run. Normal `.runtime/link-libs`,
`TMPDIR=/tmp/opencode` and browser sandbox are retained; no global install, sudo,
security/TLS/sandbox bypass, external service or publication.

## Preservation, owned resources and remaining qualification

`E/final-audit.json` verifies **142** entry-fingerprinted source/document/script/lock
inputs: only the requested status index changes; the new verification report is
the only new repository file. Implementation, analysis and historical reports are
unchanged. Fresh databases and encrypted files have no independent private token in
plaintext; nonprivate tables, **18 metadata files and 12 logs** exclude private
tokens/local session IDs. Retained session files are RGS1, 0600, in 0700 vaults.

Owned lifecycle records show **98** maintained, **32** initial-probe and **80**
successful-probe descendants reaped. **Zero remaining own children, live scoped
product processes, worker scratch entries or pending files.** Listener/server
threads close and stop. Final audit is read-only: no unowned signals, host-wide
zombie cleanup, broad deletes or recursive history backups. Failed verifier fixture
evidence is retained, not cleaned away.

Still **PENDING / NOT RUN**: real OS-keystore persistence/restart/locked recovery;
rendered GPUI/human login/consent/SSO/MFA/visual approval/accessibility; Windows,
macOS and clean supported machines; enterprise proxy/OS trust/custom CA/VPN,
device-bound/rotating/service-worker/custom SSO; qualified current public runtime,
distribution/signing/hosting/release; full export/import, integrated Phase 5 native
approval/cross-OS baselines and 1,000×3 soak. The historical signed runtime remains
**DevelopmentLoopback only**, not public-qualified.

**Bounded local gate closed:** original P1 and corrected Forget P2 are independently
fixed locally. Phase 5 may safely start dependent local integration; none was added
here, and all external/native/OS/release gates remain explicit.
