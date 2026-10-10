# Phase 4 — independent session-binding reverification

Date: 2026-10-10. Fresh independent GPT-6.1 Sol High verifier; bounded Phase 4
correction only. No implementation edits, Phase 5 integration or nested agents.

## Verdict

**Original P1 credential/origin-transfer defect: FIXED LOCALLY. Overall correction
verdict: OPEN for a newly reproduced P2 encrypted-session deletion ownership gap.**
The fresh exact old exploit returns typed **NeedsLogin before any destination B
request**, with no old-cookie transfer or anonymous fallback. Encrypted restart,
worker installation, fresh intended-origin login and evidence preservation pass.

However, a copied machine-local encrypted reference cannot replay, but **Forget
from its wrong project/environment deletes the original owner's ciphertext**.
Ephemeral copied-reference Forget correctly preserves the original session.
Dependent Phase 5 auth/capture/session integration should **remain on hold for a
bounded repair and independent recheck of this P2**. Independent, unwired analysis
work may continue under decision 0005/both 0006. This is not a renewed blanket stop
for pending external qualifications, nor a claim the original P1 remains unfixed.

Evidence root (`E`): `/tmp/opencode/regressee-p4-reverifier-b893acff56d6/`.
Historical `04-verification.md`, `04-corrections.md`, their dates and evidence, and
the separate analysis implementation/audit are unchanged. Read the actual owner,
encrypted vault, environment storage/mapping, crawl/capture authorization, signed
worker and maintained assertions, plus decisions 0005/both 0006/0009.

## Exact old exploit, independently repeated

Fresh actual Product/SQLite/signed-browser fixtures:

- A: `http://127.0.0.1:34421`; B: `http://127.0.0.1:44227`.
- Establish cookie-only encrypted state at A with the explicit constructor-injected
  test provider, capture two fixed targets and activate their baseline. One target
  has ordered/repeated queries and a hash route.
- Edit the **same environment ID** from A `/scope` to B `/preview`, keeping its old
  local reference. Actual probe, discovery and fixed-target capture each return
  **NeedsLogin**, reason: “Saved login does not match this environment's origins or
  access policy. Log in again before visits.” Each preserves **A hits 4→4, B 0→0**.
- A new encrypted owner still refuses with those exact counters. No probe receipt
  is minted; the old ciphertext is not migrated or rewritten.
- Switching Public does not anonymize the invalid reference. Same-origin access
  downgrade, scheme drift and origin-set addition also return exact NeedsLogin
  with **A 4→4, B 0→0**. Separate origin-set removal returns the same reason with
  **zero accepted connections** on both reserved owned loopback listeners.
- The unrelated environment retains its reference and identical ciphertext and
  probes successfully. Name-only and approved same-origin source/destination prefix
  edits preserve the original ciphertext/login but invalidate whole-settings probes.
  The new prefix receives authenticated requests only after the deliberate probe.
  `/logout` mapping is rejected exactly as “Mapped page is excluded before
  navigation: Potentially destructive or sign-out address — visits can have effects”,
  with zero added requests.
- Explicit **new-ID intended B establishment** then probes and completes two mapped
  captures with the original logical inventory. Only successful re-login prunes
  the old ciphertext. Restarted encrypted B state replays successfully.

Evidence: `E/{probe.rs,independent-rust/report.json,independent-commands.json}`.
The Rust establishment above is a development-test boundary, not human login.
A separate fresh signed-controller browser probe really opens synthetic A/B login
pages and clicks their owned login buttons in headless test contexts:
`http://127.0.0.1:46417` and `http://127.0.0.1:46293`. Before NEW B login, all drift/
wrong-reference tests preserve **A 6→6, B 0→0** and create no outputs. NEW B login
confirms replay, captures the ordered-query/hash URL and discovers successfully;
the original A session still captures. B records **7 requests, zero old-A-cookie
requests**. No production headed/human/GPUI qualification is inferred.

## Worker, cryptography and private-state boundaries

- Independent actual signed worker IPC InstallSession with A-bound material and B
  authority returns **NeedsLogin**, A 4→4/B 0→0. Subsequent authenticated discovery
  and CaptureTarget return **InvalidRequest**, still zero added requests.
- Separate signed-module browser probes reject cookie-only same-host/different-port
  and scheme installs on both existing and fresh references; unbound legacy state
  and rewritten-binding reinstallation of an existing reference are **NeedsLogin**.
  Wrong project/environment/session identities and old-session B discovery/capture
  are **InvalidRequest before context visits**, with no evidence outputs.
- Actual encrypted legacy state returns the explicit legacy/incompatible **NeedsLogin**
  reason without changing its ciphertext. Wrong outer/inner bindings are NeedsLogin
  before visits. Wrong project/environment AAD and renamed session ciphertext are
  exactly **Corrupt** at the vault. Header, nonce, ciphertext and tag mutations are
  Corrupt at the vault and typed NeedsLogin at Product, **A 10→10/B 0→0**.
- Copied encrypted same-origin references cannot replay. Copied ephemeral references
  fail exactly “Ephemeral login belongs to another environment. Log in again.”;
  wrong-environment/project Forget preserves their original in-memory owner. Restart
  fails exactly “Ephemeral login ended. Log in again.” before added visits.
- Both actual persisted-owner and worker normalization probes preserve HTTP `:80` /
  HTTPS `:443` equivalence, while scheme changes fail closed. The maintained focused
  tests also pass hostname case, sorted/deduplicated origins and frozen-caller checks.
  Default-port probes test binding normalization, not a port-80/443 server or TLS trust.
- The v2 origin/access envelope is authenticated inside existing AES-GCM; identity
  AAD remains project/environment/session. Worker v1 material binding must agree
  with installation authority. No silent reattachment, plaintext fallback or leakage
  into project/run/screenshot metadata was observed. The deletion issue below is
  **not** an AEAD break or a replay/credential-disclosure bypass.

## New P2 — encrypted Forget ignores original scope before deletion

**Reproduced through actual Product::forget_login**, not merely inferred from code.
Two same-origin environments in one project:

1. Establish encrypted state under environment A, retaining its immutable session ID.
2. Explicitly transplant that ID into environment B's local Store reference.
3. B probe returns typed **NeedsLogin**, “Saved login could not be decrypted. Keep
   existing evidence and log in again.” Original ciphertext is unchanged at this point.
4. Select B and call Forget. B's reference disappears, **A's ciphertext also disappears**,
   while A's reference remains unchanged.
5. Select A: probe returns typed **NeedsLogin**, “Saved login is missing. Log in again.”
   The reserved real `http://127.0.0.1:38707` listener accepts **zero connections**.

The separate wrong-project variant reproduces the same deletion on owned loopback
fixtures. Evidence: `E/{forget-probe.rs,independent-forget/report.json,forget-commands.json}`
and `E/{metadata-probe.rs,independent-metadata/report.json,metadata-commands.json}`.
The diagnostic binaries exit 0 because they assert/report the actual defect; that
exit is **not a security pass**.

Precondition is a copied/corrupt **machine-local reference**, explicitly created via
the test Store API. No portable import, ordinary name/mapping edit, malicious website
or cross-user access route was demonstrated. Impact is loss of another environment's
saved login, not screenshot/baseline/history deletion or secret disclosure. Ordinary
unrelated-environment edits and replay isolation pass. This is nevertheless a concrete
non-destructive scope-ownership gap within the requested wrong-binding boundary.

Source: `crates/app/src/auth.rs:173–192,556–563` delegates encrypted removal to
`crates/runtime/src/secrets.rs:237–240`. `delete` addresses only the session ID and
checks file shape/permissions, **not successful original-scope AAD authentication**,
before removing it. Replacement/pruning uses that same removal path; its foreign-
reference deletion variant was not separately executed and is not claimed reproduced.

Bounded repair requirement: encrypted deletion/pruning must refuse a mismatched
original project/environment/session scope without deleting its file; preserve
explicit valid Forget, successful fresh-ID replacement, missing-state recovery and
ephemeral isolation. Add same-project/wrong-environment and wrong-project regression
controls through actual Product, then obtain independent verification. No fix here.

## Serial maintained checks and independent commands

| Command / evidence | Actual result |
| --- | --- |
| `python3 scripts/check-phase4-corrections.py --output E/maintained` | PASS; focused **5**, 0.264 s; nested serial stage 350.614 s; zero remaining own children/protected changes |
| Nested `bash scripts/check-foundation.sh` | PASS: UI build/fmt, **82 locked Rust** (11+6+20+14+5+11+15), strict all-target/all-feature clippy; 6.021 s |
| Nested actual fresh owned OS-store test | Exit 0, 0.214 s, **unavailable branch; persistence NOT QUALIFIED** |
| Nested actual ignored Product/browser regression | PASS, 344.306 s; **33 signed Node**, zero failed/skipped; ordinary Product drift/restart/new-login/other-environment/readiness/resources retained |
| `python3 E/run-independent.py --compile-only` | Initial verifier-only compile FAIL, 0.164 s: missing ProcessSupervisor trait import; no browser probe ran in this attempt |
| `python3 E/run-independent.py` | Corrected harness compile PASS, 0.665 s; independent Rust PASS, **149.661 s**; signed-module browser PASS, **1.716 s**; full exact commands/dependency fingerprints retained |
| `python3 E/run-metadata.py` | Compile PASS, 0.615 s; bounded metadata probe 0.007 s; origin-removal/wrong-project replay refusal pass; encrypted foreign-Forget defect recorded |
| `python3 E/run-forget.py` | Compile PASS, 0.615 s; same-project/wrong-environment actual Product Forget **reproduces P2**, 0.008 s |
| `python3 E/audit.py` | Scoped data/signature/source/PNG/resource/secret audit PASS; not a pass of the newly reproduced deletion behavior |

Resource regression really executes scripts, applies CSS and decodes images across
origins; forbidden-document controls remain zero-visit. Actual IPC cancel takes
**30 ms**, request-/response-pause cancel **7/6 ms**. No unrelated crawl/full soak.

## Preservation, cleanup and remaining qualifications

Fresh audit independently verifies Ed25519 signature, all **6,025** signed file
hashes/sizes and **10** maintained worker/test files byte-for-byte. Runtime remains
Node **22.20.0**, Playwright **1.56.1**, Chromium **141.0.7390.37**, worker **4.0.0**,
DevelopmentLoopback with the normal sandbox. This historical runtime is restricted
to owned literal-loopback fixtures, not qualified for public capture.

Across **four** isolated databases: private fixture tokens/references are absent
from portable/evidence tables; only local-session rows retain references. **Nine**
retained session files have encrypted RGS1 headers, 0600 files and 0700 vaults.
**17 metadata files / 14 logs** pass secret/reference checks; **12 CAS PNGs** pass
digest/CRC/EOF/decompression checks. The independent original run/baseline serialized
bytes, active baseline and **three original CAS objects** remain byte-identical.
The P2 diagnostics affect only their newly owned fixture-session ciphertext.

`E/final-audit.json` records **138 protected source/document/lock inputs**, **51
historical report/database/CAS inputs unchanged**, zero live scoped product processes,
zero pending files, empty worker scratch and zero newly observed host zombies.
The **61 entry zombies** are historical and were neither signaled nor claimed cleaned.
Owned subreapers reap only their descendants; fixture listeners close and workers/
browsers stop. Only the validated empty owned Node scratch directory is removed by
nonrecursive `rmdir`. No recursive history copying, broad deletes/kills, global
installs, sudo, security flags, sandbox/TLS bypass, services or publication.
Repository edits are only this report and `docs/IMPLEMENTATION_STATUS.md`.

Still **PENDING / NOT RUN**: real OS-keystore persistence/locked-store recovery and
restart; rendered GPUI/human login/consent/SSO/MFA/visual approval/accessibility;
Windows/macOS/clean supported machines; Chromium proxy/OS trust/custom CA/VPN/
managed enterprise networks, device-bound/rotating/service-worker/custom SSO;
qualified current public runtime/distribution/signing/hosting/release; full ZIP
export/import, integrated Phase 5 approval/native cross-OS baselines and 1,000×3 soak.
No repeated native black-window attempt was made. Historical headed/TLS/scale
evidence is retained, not rerun or upgraded into new qualification.

**Next local gate:** bounded encrypted-removal scope repair and independent recheck.
Original origin-transfer P1 is fixed; full dependency acceptance remains open for
the distinct P2, not for the separately pending external qualifications.
