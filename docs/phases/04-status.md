# Phase 4 — local authentication, mapping and capture implementation

**2026-10-10 correction addendum:** the original record below is retained.
Independent verification in `04-verification.md` reproduced P1 session-origin
drift. Its bounded repair and fresh implementer checks are in `04-corrections.md`;
final corrected source passes **82 locked Rust + 33 signed Node tests**, the real
two-origin zero-visit/new-login capture regression and scoped data/runtime audit.
**independent correction verdict PENDING**, and dependent auth/capture integration
still waits. Standalone analysis-engine work/audit remain untouched and unwired.

Date: 2026-10-09. **Implemented locally; independent verification pending.**
This is not native/human, cross-platform, enterprise-network, public-runtime or
release acceptance. The non-developer completion gate has **NOT PASSED**.
Phase 3's independently accepted document/resource boundary is retained.
No Phase 5 workflow or analysis-engine integration was performed, no nested agent
was spawned, and no release/signing/hosting authority was created.

## Delivered behavior

| Area | Maintained source and behavior |
| --- | --- |
| Login owner | `crates/app/src/auth.rs`, `jobs.rs`, `main.rs`: real saved-environment Open browser / I'm ready / Close / Forget actions on the blocking owner. Verified Chromium is headed in production; a fresh representative context confirms replay. Settings changes during login require a fresh setup. GPUI receives nonsecret status only. |
| Private sessions | `crates/runtime/src/secrets.rs`: pinned AES-256-GCM, random nonces, authenticated project/environment/session identity, private encrypted files and OS-key-provider interface. OS calls have a bounded owner/deadline. Linux nonblocking private pipe writes honor deadlines/cancellation and zeroize encoded secret buffers; escaped inbound/outbound frames are bounded. Unavailable storage fails closed with explicit Ephemeral. Re-login removes superseded material; at most 16 ephemeral references (512 KiB each); owner restart requires re-login. |
| Local versus portable data | Migration `005-local-auth.sql` / storage APIs keep selected-environment settings and references machine-local. `Environment.session` is skipped on serialization; legacy null is readable, persisted nonnull references are rejected. Project/run/evidence metadata contains no session material or reference. No Phase 6 export workflow was added. |
| Replay | `worker/product/src/sessions.mjs`: cookies/localStorage/IndexedDB and origin-scoped sessionStorage in fresh contexts; approved-site filtering excludes IdP origin storage. Conflicting same-origin tab state fails closed. Bad/missing references never silently become anonymous capture. Private requests/events are bounded, redacted and not diagnostic logs. |
| Expiry and recovery | Conservative password/access/redirect detection produces NeedsLogin and pauses queues; no login redirect visits/screenshots. Re-login/resume retains previous evidence and mixed-time warnings. Settings/runtime/OS/architecture/font continuity is checked. |
| Environments/mapping | `crates/core/src/environment.rs`: explicit origins/slash-boundary base prefixes, longest match, ambiguity failure, preserved ordered query/hash/encoded paths. Preview and first-three-page probe; changed mappings require a fresh in-memory receipt before a full check. Logical inventory/keys stay frozen; actual mapped provenance lives in capture metadata. Discovery currently requires identity maps. |
| Capture consistency | Fixed profile locale/timezone/theme/motion/caret/pointer; DOM/fonts/images/lazy-load/sample budgets. 32,768-pixel height / 64-million-pixel / 64-scroll ceilings fail rather than crop. Unique warnings are visible in plain language. Virtualized/nested-scroll content is not claimed as covered. |
| Comparability | Fresh current rendering fingerprint (bounded font inventory, not universal). Material profile/runtime versus OS/font caution is separate from intentional site mapping. Unstable/incomplete or rendering-caution comparisons are NotComparable, not a trustworthy unchanged verdict. Existing analysis APIs are retained; no parallel module was integrated. |
| Network interfaces | Machine-local credential-free proxy for downloads/Chromium; PEM CA for downloads/Node, separate Chromium OS trust prerequisite. Node robots fetch fails closed with configured proxy. No TLS bypass; inherited Node TLS-disable/options are cleared for product workers. VPN uses existing OS networking. |
| Legacy API boundary | Public single-page `Product::capture` remains available but rejects local login references/changed mappings instead of anonymous fallback. It records current rendering metadata and applies caution to incomplete/mismatched evidence. Native controls use guided inventory capture. |

Decision: `docs/decisions/0009-phase4-session-and-environment-boundaries.md`.
User help: `docs/auth-and-capture.md`; surface contract: `docs/auth-and-capture-surface.md`.
UI finish disposition preserves the proposed DESIGN.md world and adds plain warning
labels/limits/authoritative actions; **native visual/human acceptance remains open**.
There are no new shipping raster/font assets; test PNGs are synthetic evidence.

## Evidence and actual results

Workspace: `/path/to/Regressee`.
Evidence root (`E`): `artifacts/phase4-implementation-bd9bd6b69e95/`.
`entry-hashes.json` records entry source/document hashes (excluding the parallel
analysis-owned files). `handoff-source-hashes.json` and `final-audit-clean.json`
record final hashes/audit to distinguish subsequent edits. No initial
whole-artifact hash manifest exists; no claim of one is made.

Commands run serially with locked dependencies, four build jobs and one Rust test
thread. Implementer results below are **not an independent verdict**.

| Evidence | Actual result |
| --- | --- |
| `E/verification-f/locked-ui-tests-clippy.log` / `commands.json` | **Final source PASS**, 11.881 s: UI-inclusive build, format, **78 Rust tests** (7+6+20+14+5+11+15), all-target/all-feature clippy `-D warnings`. Explicit browser/read-only/OS-store tests remain ignored in the ordinary batch. Includes stalled simulated and actual Linux-pipe transfer deadline/cancel checks. |
| `E/verification-f/owned-os-store.log` | Harness passes its unavailable-store branch in 1.116 s. **OS store unavailable; persistence qualification NOT PASSED.** No real-store success is inferred from exit 0. |
| `E/verification-f/signed-auth-capture.log` / `product-flow/` | **Final source PASS**, 299.762 s: fresh development-signed pack, exact maintained worker bytes, sandboxed real Chromium; **32 Node tests**, no skipped/failed cases; full Rust authenticated product/storage flow. |
| `E/verification-d/`, `E/verification-e/` | Earlier successful implementation runs, preserved. D: 77 Rust, 32 Node and real flow (308.698 s); E: real flow 231.070 s. Superseded by the final F run after legacy API and bounded private-pipe hardening, not silently treated as final-source evidence. |
| `python3 E/final-audit.py` → `E/final-audit-clean.json` | PASS: zero live scoped processes, scratch entries, runtime temporaries/pending publications or unexpected protected-source changes. Nine project/evidence records exclude fixture secrets/local references. Both saved session files have RGS1 encrypted headers, private permissions and no plaintext fixture material. All ten installed worker sources/fixtures match maintained bytes. |

Signed auth cases include all four storage kinds together and independently removed,
consent/representative confirmation, bad references and staging/origin isolation,
ordered queries/hash routes, auth discovery/expiry with zero login destinations,
unapproved interactive IdP redirect with zero destination visits, delayed resources,
incomplete fonts/images/resource warnings, HTTP 429 no-evidence failure, unstable
counters, tall/infinite no-crop failures and even public login-form rejection.

The Rust product flow saves separate production/staging encrypted **test-provider**
states, captures/activates a real baseline, rejects unprobed checks before visits,
probes and checks mapped pages without changing logical identities, restarts the
explicit provider and decrypts/replays, pauses expiry without screenshots, resumes,
rejects missing state with zero visits and preserves the original baseline hash.
FixtureKey is constructor-injected; it is **not an OS-keychain pass**. Production
OpenLogin is headed; the signed module test uses an explicit headless fixture
boundary and programmatic fixture buttons, **not normal native human login**.

The unchanged signed resource suite again proves actual allowed cross-origin
script/CSS/image loading, zero forbidden document/resource visits, mapped native
network provenance and actual IPC cancellation. Its retained report records
28 ms worker cancel and 7/6 ms request/response-pause controls in verification-f.
This is not a repeat of the Phase 3 scale soak or external-network qualification.

### Preserved failures

- Earlier compile/test/clippy failures and their logs remain in `E`; simulated
  actor tests were corrected to wait for authoritative environment replies.
- `verification-a`: UI checks passed, but the external 120-second tool timeout
  interrupted signed integration. No completed signed result exists; **not a pass**.
- `verification-b`: formatting check failed after a non-idempotent first format
  pass. A second format pass repaired it; failed log retained.
- `verification-c`: auth/storage fixtures passed, but the retained resource suite
  refused a missing evidence variable. Wrapper corrected, safety assertions intact;
  this run **failed**, not a product resource-boundary pass.
- Initial `final-audit.json` failed because its deliberate-change allowlist omitted
  the Phase 4 storage dependency/provenance and focus-range edits, and it found the
  interrupted A download temporary. All three edits were inspected and accounted
  for; no source was reverted. The exact regular owned temporary was verified as
  a byte-prefix of A's signed development pack, then moved (not deleted) to
  `verification-a/interrupted-owned-runtime-download.zip`; its hash/ownership record
  is `interrupted-download-preservation.json`. The corrected audit passes. Both
  audit reports and all partial evidence remain. Evidence occupies about 6.11 GB;
  no broad process/scratch/artifact cleanup was performed.

## Remaining qualification and known limits

- Independent verification: **PENDING**; fresh distinct verifier required before
  dependent implementation relies on these Phase 4 changes.
- Real OS-key persistence/locked-store/restart across Linux/macOS/Windows: **PENDING**.
  Host availability was checked, not successfully qualified. Explicit injected-key
  encryption/restart/tamper/scope/wrong-key tests do pass.
- Native GPUI pixels/input/accessibility, normal headed human login/consent, visual
  review: **NOT RUN in this phase**. Historical blank/unmapped native attempts are
  not retried or relabeled. Simulated GPUI uses actual storage/background replies.
- Device-bound, rotating, Basic Auth, arbitrary SSO/MFA/service-worker/enterprise
  sessions: **UNSUPPORTED/UNQUALIFIED**, not a universal replay claim. Conservative
  login detection has false positives and cannot recognize every custom login UI.
- Rust download proxy is exercised on owned loopback. Real Chromium proxy,
  end-to-end custom CA, VPN and managed enterprise networks: **PENDING**.
- Windows/macOS process ownership, clean supported machines, current browser runtime,
  public distribution/signing, release license/dependency audit: **PENDING**.
- 1,000-page/3,000-target soak remains **NOT RUN**; no new scale claim. Capture cannot
  expand virtualized/nested scrolling or guarantee every asynchronous app is settled.
- Probe receipt is in-memory and bounded to the first three selected addresses;
  restart requires re-probing. Login setup fields apply to the selected environment;
  switching drafts clears them, rather than silently copying production URLs.

## Independent-verifier handoff

Do not modify/import/export `crates/app/src/analysis_engine.rs` or
`docs/research/analysis-engine-handoff.md`. They are parallel-owned and remain
unregistered. Keep public analysis APIs, immutable CAS/baseline/coverage and the
independently accepted crawler/scratch/integrity assertions intact. No nested agent,
public-site test, sandbox/TLS bypass, broad deletion/process kill or publication.

Run once with a **fresh unique** directory and an outer timeout over 20 minutes:

```sh
python3 scripts/check-phase4.py --output /tmp/opencode/regressee-phase4-verifier-UNIQUE
```

Read every completion record and the OS-store log, not just exit status. Inspect
the fresh `product-flow/{phase4-report.json,auth-worker/report.json,resources/report.json}`,
SQLite runs/local reference separation, encrypted-file headers/permissions and
original PNG hashes. Compare installed worker bytes with maintained source and
the signed manifest. Preserve failures and leave fixture servers/processes stopped.

Add genuinely independent probes for mapping ambiguity/multi-origin/query/hash,
wrong/stale references and unavailable secure storage, no-origin leakage, changed
login-settings confirmation, re-login/pruning/ephemeral restart, settings/renderer
continuity on resume and warning/caution classification. Exercise real GPUI actor
actions with authoritative saved replies; simulated focus is not native approval.
Retain actual resource interception/robots/action zero-visit tests. Extend proxy/CA
tests only within owned loopback and without altering host trust. Record native,
human, OS/enterprise/public/release gaps accurately instead of inventing acceptance.
