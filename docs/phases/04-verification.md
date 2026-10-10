# Phase 4 — independent authentication/capture verification

Date: 2026-10-09. Fresh independent GPT-6.1 Sol High verifier; **Phase 4 only**.

## Verdict

**OPEN: substantial local behavior verified, but one reproduced P1 session-origin
binding defect needs a bounded repair and independent recheck before dependent
Phase 5 auth/capture integration.** This is not a blanket stop of independent local
work under decisions 0005/both 0006. The isolated analysis auditor's files/work
were neither touched nor duplicated. No implementation edits or Phase 5 integration.

The ordinary handoff passes **78 locked Rust tests and 32 fresh signed Node tests**,
including real authenticated capture, separately selected production/staging,
mapping, injected-key restart, expiry pause and resume. This does **not** prove
that every saved session remains bound to its originally approved origin.

Real OS-keystore persistence remains **NOT QUALIFIED**: the actual fresh owned
availability test reports unavailable. Native GPUI/human login/visual acceptance,
cross-platform, enterprise and public-runtime/release gates remain open. These
pending qualifications alone are not the reason for the local integration hold.

Evidence root (`E`): `/tmp/opencode/regressee-p4-verifier-5e265486f7c9/`.
Read PRODUCT/PLAN, Phase 4 checklist/status, prior combined crawl/resource
verification, both 0006 decisions, decision 0005/0009, current secret/session/
environment/capture/protocol/storage and GPUI/Jobs ownership code and assertions.

## Reproduced finding — P1: live mapping rebinds an old saved login

**An existing environment's destination-origin edit silently reauthorizes its old
cookie-only state for the new origin.** The worker's normal wrong-origin checks
work, but Rust supplies the *new* approved-origin list when reinstalling *old* state.

Fresh real Rust Product/SQLite/signed-worker/browser reproduction:

1. Establish a synthetic cookie-only encrypted login with explicit test provider
   in an InteractiveLogin environment, identity-mapped to owned production.
2. Capture and activate two fixed targets, one with ordered/repeated query and
   hash route. Keep the environment ID and saved session reference unchanged.
3. Save an approved destination edit from production `/scope` to staging
   `/preview`, on another numeric-loopback **port** of the same hostname.
4. Call actual `Product::probe_environment`: **succeeds and mints a probe receipt**.
   Staging receives the old private cookie on both representative requests:
   `/preview/home?z=9&a=2&a=1` and `/preview/two`.
5. The subsequent actual fixed-target check **Completes**, preserving logical
   keys but publishing captures from the new origin, without establishing a new
   login there. Mapped capture provenance is `http://127.0.0.1:36035/preview/...`.

Evidence: `E/independent-rust/report.json`, its retained SQLite/CAS originals,
`E/probe.rs`, `E/independent-commands.json`. Actual commands:
`python3 E/run-independent.py`, which compiles against the locked build's exact
dependency fingerprints and runs `E/rust-probe E`, followed serially by the
signed-module Node probe. Existing evidence roots deliberately refuse overwrite.

Relevant paths:

- `crates/app/src/auth.rs:67–74,214–252`: `auth_state` decrypts using project/env/
  session identity, then constructs `AuthState.origins` from **current** mappings.
- `crates/app/src/auth.rs:39–54,462–516`: installation passes that current list;
  representative probing can therefore validate the unintentionally rebound state.
- `crates/runtime/src/secrets.rs:174–179`: AEAD authenticates project/env/session,
  **not the original approved origin set**; the private state has no saved binding.
- `worker/product/src/sessions.mjs:8–26`: cookie host filtering accepts the
  cookie-only state for the new port, then `get` trusts the newly installed list.
- `crates/storage/migrations/005-local-auth.sql`: local references do not retain
  the original session origin approval.

This is **not an AES-GCM integrity break**, automatic navigation to an unapproved
mapping, or a claim that HTTP cookies themselves are port-scoped. HTTP cookies
are host-scoped; the missing protection is Regressee's promised session-origin
binding above that browser behavior. Destination approval for a mapping is not
fresh consent to reuse an earlier site's credentials. Separate environment IDs
remain isolated, but retargeting one existing ID bypasses that protection.

**Bounded repair requirement, not implemented here:** retain/authenticate the
original approved capture-origin set with each saved/ephemeral session, and reject
changed-origin reuse **before any probe/discovery/capture visit**, requiring a fresh
login. Same-origin prefix-only mapping need not become rendering incompatibility.
Preserve old evidence, document/resource guards, encryption and explicit ephemeral
fallback. Add cookie-only same-host/different-port and relevant scheme/origin-drift
regressions through the actual Product owner; a direct `Sessions.get` test alone
does not cover this defect.

## Independently verified local boundaries

### Sessions, cryptography and replay

- Fresh signed maintained all-storage replay/consent confirmation and each of four
  independently removed storage kinds pass. A separate independent fixture also
  tests **cookie, localStorage, IndexedDB, sessionStorage individually as positives**;
  independently missing state returns exact **NeedsLogin**, with no output files.
- Wrong project/environment/session IDs, stale references and ordinary wrong-port
  origin requests return exact **InvalidRequest before HTTP visits**, not anonymous
  fallback. Wrong-host cookies and wrong-origin local/IndexedDB/session storage
  are refused at installation. These passes do not erase the P1 above.
- Fresh approved interactive IdP on `127.0.0.2` works through the exact signed
  controller's explicit headless test boundary. IdP cookies/local/session storage
  are filtered from returned state; capture at that IdP is InvalidRequest with
  zero added visits. Headless fixture setup is not human login qualification.
- Actual AES-256-GCM constructor-injected-provider tests verify persistence through
  a new owner, exact **Corrupt** for wrong key, changed project/environment/session
  AAD, and header/nonce/ciphertext/tag tampering. Two independently generated original
  nonces differ; deliberate renamed ciphertext is rejected. Headers are RGS1, modes
  0600, directories private; unavailable providers write **no session file**.
- Default production AuthOwner can explicitly replace an old encrypted reference
  with **Ephemeral** without obtaining an OS key. The old session file is pruned,
  ephemeral capture succeeds, no replacement session file exists, and a new owner
  fails exactly **Ephemeral login ended** before any added HTTP visit. No plaintext
  fallback was observed. Explicit test-provider persistence is not OS persistence.

Evidence: `E/independent-{rust,node}/report.json`, `E/{probe.rs,probe.mjs}`,
`E/handoff/product-flow/auth-worker/report.json`, `E/data-runtime-audit.json`.

### One actual headed service-chain attempt — PASS, not human acceptance

`python3 E/run-headed-once.py` executes the **actual production Rust
OpenLogin → ConfirmLogin → discover → capture_inventory → activate → forget**
methods with the unmodified signed worker. OpenLogin launches **headed** Chromium
on this host, not a headless replacement. The owned fixture page self-establishes
a synthetic cookie/consent value; no user's credentials or manual login is tested.
Confirmation replays in a fresh context, a real baseline completes/activates, and
forgetting the login then returns exact **Log in again before visits**.

This uses the default production AuthOwner with explicit Ephemeral, **no test key
provider or session injection**, and creates **no private-sessions directory**.
There was exactly one attempt, **36.111 s**, exit 0. Evidence:
`E/headed-{commands,reaped}.json`, `E/headed-once/report.json`, `E/headed-probe.rs`.
The GPUI buttons' Jobs/service wiring and saved-state actor assertions were read
and rerun in the locked batch; **no rendered GPUI click, human login or visual
approval is inferred** from this successful service-chain probe.

### Mapping, expiry, resume, comparability and readiness

- Longest-prefix precedence, slash-boundary rejection, explicit multi-origin maps,
  ambiguity rejection, encoded paths, ordered/repeated queries, trailing slash and
  hash preservation pass independent actual core assertions.
- Actual maintained unprobed mapped checks make zero visits. Mapping alone yields
  **NoSignificantChange** in the separately established mapped-environment fixture;
  inventory/target keys remain fixed, while metadata records actual mapped URLs.
  Independent two-target checks never visit their newly linked `/never-fixed` page.
- All **301/302/303/307/308** redirects to same-site login, staging and even an
  approved interactive IdP produce exact **NeedsLogin** for authenticated capture/
  discovery, **RedirectOutsideScope** for public capture, **zero destination visits**
  and no screenshot outputs. Actual Rust representative probing also returns typed
  NeedsLogin, no probe receipt and zero login visits—not a generic failure pass.
- Fresh real Product expiry pauses a single-concurrency two-target queue with one
  NeedsLogin failure, one pending target and no captures/login visits. Replacing
  the ephemeral session with a **new ID**, then resuming, completes the same run,
  inventory/profiles, both captures and mixed-time warning/resume timestamp.
  Changed saved settings cause exact **Project settings changed** before visits,
  preserving the active baseline and original PNG hash.
- Actual differing profile/browser inputs produce **MaterialMismatch**; differing
  OS/fonts produce **RenderingCaution**. Check uses frozen baseline profiles and
  requires matching runtime; resume checks revision/runtime/current renderer/font
  continuity. Source inspection confirms mismatch/incomplete/unstable comparisons
  become **NotComparable**, not a trustworthy unchanged verdict. New native cross-OS
  classification/approval workflows are not claimed; approval integration is Phase 5.
- Maintained delayed-font/image positives, unique incomplete font/image/resource
  warnings, unstable counters, 429 no-evidence failure and tall/infinite no-crop
  ImageLimit assertions pass. Independent full-page lazy loading actually requests
  and decodes its image, captures **1,925 px** with no warning, visits no linked new
  page, and rejects a **33,000 px** page with ImageLimit/no output. Bounds remain
  32,768 px, 64 million pixels, 64 scroll steps and three stability samples.
- Prior document/resource scope remains enforced. Signed retained resources really
  execute scripts/apply CSS/decode images across origins, preserve Loopback network
  provenance, forbid document/resource destinations and retain IPC cancellation
  (**27 ms**, native request/response pause **6/7 ms**). Interactive IdP approval does
  not become crawl authority. Robots/action/visible-ID locked regressions pass;
  the full prior Rust-owned browser policy/scale corpus was not rerun.

Evidence: `E/resume/report.json`, `E/resume-probe.rs`, independent Node records,
fresh maintained resource report/IPC events and inspected maintained assertions.

### Network and data separation

Owned TLS fixture with a valid leaf signed by a private synthetic CA:
default and unrelated CA return exact **InvalidCertificate(UnknownIssuer)** with
zero HTTP visits; explicitly configured matching CA yields HTTP 200 and exact body.
Chromium with that Node CA still returns **ERR_CERT_AUTHORITY_INVALID**, with no
HTTP visit or TLS bypass. Actual locked proxy-client unit test routes through owned
loopback; credential-bearing proxy/TLS-disable interfaces are rejected. Worker
launcher clears inherited TLS-disable/Node-options variables. No host trust changed.

Independent audit verifies the fresh Ed25519 signature, all **6,025 signed file
hashes/sizes**, and **ten** installed maintained worker/test files byte-for-byte.
Runtime is Node **22.20.0**, Playwright **1.56.1**, Chromium **141.0.7390.37**, worker
**4.0.0**, **DevelopmentLoopback**, sandbox enabled. Historical bytes remain confined
to owned numeric-loopback fixtures—not qualified for public sites.

Fresh SQLite project/inventory/run/baseline/analysis rows exclude session material
and references; only machine-local `local_sessions` retains references. Portable
schema output excludes sessions and rejects injected fields. Original PNG/CAS hashes,
**21 PNG CAS objects** (originals and derivatives), PNG CRC/EOF/dimensions/decompressed
lengths, screenshot metadata and product logs are audited across **four isolated app
databases**. Two retained product session files have encrypted RGS1 headers/0600 modes;
the actual headed Ephemeral owner created no session directory. Real ZIP export/import
is **NOT RUN/not implemented in Phase 4**; schema
separation is not an archive-export security qualification.

## Exact serial commands and diagnostic honesty

| Command/evidence | Actual result |
| --- | --- |
| `python3 scripts/check-phase4.py --output E/handoff` → `handoff/commands.json` | Locked native build/fmt/workspace tests/clippy **PASS**, **78 Rust**, 6.173 s; explicit owned OS-store test **unavailable branch**, 0.214 s; signed auth/product flow **PASS**, 212.186 s, **32 Node**, zero failed/skipped |
| `python3 E/run-independent.py` → `independent-commands.json` | Initial standalone compilation **failed** from mismatched cached dependency identities, 0.414 s; corrected verifier-only fingerprint selection compiles, 0.615 s; real independent Rust assertions finish, **120.821 s**, and reproduce P1; independent signed modules **PASS**, **6.023 s**, nine records with nested storage/identity/redirect cases |
| `python3 E/run-followup.py` → `followup-commands.json` | Resume/portable/material-caution assertions **PASS**, **79.516 s**; subsequent initial TLS fixture **failed**, not a network qualification pass |
| `python3 E/run-network-b.py` → `network-b/commands.json,report.json` | Valid signed-leaf TLS/download/Chromium trust rejection controls **PASS**; all exact rejection/positive-body/zero-visit assertions retained |
| `python3 E/run-headed-once.py` → `headed-commands.json,headed-once/report.json` | One actual headed production auth-owner/service lifecycle **PASS**, **36.111 s**; synthetic self-establishing fixture, default owner/explicit Ephemeral; not human/rendered GPUI acceptance |
| `python3 E/audit.py` → `data-runtime-audit.json` | Final independent signature/hash/data/PNG/log audit **PASS** |

Initial TLS fixture incorrectly used a CA certificate as a server leaf:
**CaUsedAsEndEntity** rejected even with explicit trust; its logs/keys/commands stay
in `E/network/`. It is not an UnknownIssuer or valid-CA success pass. The follow-up
uses a real signed end-entity certificate; no security flags or product fixes.
Audit setup failures (overbroad manifest glob included Chromium preloads, missing
Python cryptography, treating retained node_modules as maintained source) were
verifier tooling faults. Final audit targets the actual runtime manifest, uses
built-in Node Ed25519 verification and distinguishes maintained source from signed
historical dependencies; no package installation or integrity assertion weakened.

The one bounded actual headed-browser service attempt is described above;
no repeated native black-window loops or visual approval is inferred.

## NOT RUN / remaining gates and preservation

- Non-developer interactive login/consent/SSO/MFA and rendered GPUI pixels/input/
  resize/clipboard/IME/accessibility/human approval: **NOT RUN**. Simulated GPUI
  authoritative saved-environment actions pass; they are not native acceptance.
- Real OS-store persistence/locked-store recovery/restart on Linux/macOS/Windows:
  **PENDING**, host unavailable; injected-provider tests do not qualify the store.
- Real Chromium proxy/trusted OS/browser CA, VPN/managed enterprise networks,
  device-bound/rotating/service-worker/custom SSO sessions: **UNQUALIFIED**. Basic
  Auth/universal SSO are not promised or required passes.
- Supported clean machines, Windows/macOS ownership, current public runtime,
  distribution/signing/hosting, release qualification and 1,000×3 soak: **NOT RUN**.
- Full ZIP workflow, native cross-OS baselines and selective approval integration:
  not exercised or implemented here. Concurrent analysis-engine audit not touched.

Only this report and `docs/IMPLEMENTATION_STATUS.md` are repository edits. Entry
hashes cover **121** protected source/document/lock inputs; original reports/evidence
are preserved. Final scoped ownership/data audit is recorded in `E/final-audit.json`;
no broad deletion, recursive backup, unrelated process kill, global install, sudo,
sandbox/TLS bypass, publication or external service was used. Owned fixture servers
close, browsers/workers stop; only validated empty owned scratch is removed with
nonrecursive `rmdir`. No globally clean host-process-table claim.
The final snapshot reports **zero unexpected protected-source changes, zero live
scoped processes, zero new zombies, no pending files and no owned/default scratch
residue**. The **61 entry zombies** are historical, not claimed cleaned. The prior
pre-report scoped/data snapshots are retained separately; subreaper runners reap
only their own adopted descendants and do not signal unrelated harness processes.

**Next dependency gate:** repair and independently verify P1 origin drift. Then
local Phase 5 may depend on the locally verified auth/capture foundation while
native/human/OS/network/public-release qualifications remain honestly pending.
