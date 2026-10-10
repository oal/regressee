# 0009 — local sessions, mapped identity and bounded capture

Date: 2026-10-09. Phase 4 only, under decision 0005 and both 0006 records.

## Session ownership

The blocking Jobs owner manages login/capture processes and encrypted session
files; GPUI receives only environment names, settings, typed status and evidence.
Private bounded stdio carries state, never project JSON or screenshot metadata.
Linux pipe writes are nonblocking with deadline/cancellation checks; encoded secret
buffers are zeroized. Both incoming and outgoing escaped frames include their
newline in the 1 MiB budget.
Production and staging use different environment IDs and separate local references.
Migration 005 stores local selections/references separately from project/run JSON.
Legacy `session: null` is readable; persisted non-null sessions are rejected.

AES-256-GCM is provided by pinned `aes-gcm 0.10.3`, with fresh OS-random nonces
and project/environment/session identity as authenticated associated data.
`keyring 3.6.3` provides OS storage (Linux encrypted Secret Service, macOS Keychain,
Windows Credential Manager). No custom cryptography, credential enumeration or
plaintext fallback. A bounded owner/deadline contains slow OS-store calls. Unavailable
or locked storage offers explicit **Ephemeral**, not silently persistent state.
Ephemeral state disappears with its Product owner; a stale local reference requires
login again after restart. Keys are never stored in the project/database/session file.

The test key-provider constructor and fixture session injection are explicit
development-test boundaries, not switches selected by imported data or environment
variables. A test-provider encryption/restart pass is not an OS-keychain pass.
Real credential tests address only fresh Regressee-owned entries and remove them.

## Interactive setup and supported replay

Open browser launches the verified managed Chromium headed, in a fresh context.
The user logs in and handles consent normally, then chooses **I'm ready**. State
is filtered to approved site origins; IdP state does not become crawl state.
Confirmation visits a representative application page in another isolated context.
Cookies, localStorage, Playwright-supported IndexedDB and explicitly origin-scoped
sessionStorage are replayed. Conflicting same-origin tabs are rejected rather than
guessing which tab's state is authoritative. Session transfer is bounded to 512 KiB.

Interactive setup permits explicitly approved IdP origins, but capture/discovery
retain the independently accepted exact-document boundary. Resources stay on
Chromium's native network path; document redirects remain unvisited candidates.
Access failures, obvious login forms and authenticated redirect failures produce
NeedsLogin/pause, never login screenshots as baseline evidence. This is conservative
detection, not semantic recognition of every enterprise login page. Basic Auth,
rotating/device-bound tokens, untested SSO/MFA and service-worker-dependent apps
remain unsupported/unqualified, not universal promises.

## Mapping versus rendering

Mappings are explicit origin plus slash-boundary base-prefix replacements; the
longest matching source prefix wins and equal-specificity ambiguity fails closed.
Encoded paths, ordered/repeated query parameters and hash routes remain intact.
Multiple origins need explicit mappings; there is no automatic subdomain/IdP approval.
Logical inventory identities and target keys remain frozen. Capture metadata records
the actual mapped requested/final URL without changing the original inventory.
Changed maps require a fresh representative probe before a full fixed-target check.

Site mapping alone is not rendering incompatibility. Material profile/browser
differences are distinct from OS/architecture/font caution. Current rendering
metadata is recorded rather than copying an old baseline's machine fingerprint.
The font fingerprint is a bounded inventory/hash of available system font files,
not a promise to identify every font provider or guarantee portable pixels.
Caution or unstable/incomplete comparisons cannot report a trustworthy unchanged release check.
Selective approval remains Phase 5; no analysis module is integrated here.

## Capture and networking limits

Captures fix locale, timezone, website theme, reduced motion, disabled animation,
hidden caret and pointer-away behavior. Readiness is bounded: DOM completion,
fonts, image decoding, incremental lazy-load scrolling and up to three screenshot
samples. Full-page height/pixel/scroll budgets fail instead of silently cropping.
Virtualized/nested-scroll content and unvisited UI states are not whole-page coverage.
Read-only accounts are recommended: GET visits can still change server data.

Machine-local proxy/trusted-CA interfaces do not disable TLS. Configured Chromium
and runtime-download proxy paths are separate from Node 22 robots fetch (which fails
closed with a proxy; an authorized robots override must be explicit). Custom PEM
trust applies to downloads/Node; Chromium must also trust the CA through its OS/browser
trust store. VPN is the existing OS network, not an implemented enterprise feature.
Actual managed-network, Windows/macOS, native/human and current public-runtime
qualification remain separately recorded gates.

See `docs/phases/04-status.md` for actual test outcomes and remaining limitations.
