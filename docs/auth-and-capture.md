# Login, environments and reliable captures

This is local implementation help, not release acceptance. The historical
development runtime only visits owned literal-loopback fixtures. A current
trusted public runtime and native/human workflow still need qualification.

## Log in and handle consent

1. Save a project and its website environment. Use a read-only test account.
2. Enter a login/consent URL and a representative **application** page if needed.
   Approve exact identity-provider origins only when you trust them. They do not
   expand the discovery/capture scope.
3. Choose encrypted persistence, or explicitly choose **Ephemeral** if your OS
   credential store is unavailable/locked. Ephemeral state ends when the app closes.
4. Save environment settings, choose **Open browser**, and complete login/consent
   normally. Choose **I'm ready** only when the application is ready.
5. Regressee replays the filtered state in a fresh isolated context and checks the
   representative page. A failed check does not save a usable login; finish setup
   or use **Close login browser** before starting over.

Cookies, localStorage, Playwright-supported IndexedDB and origin-scoped
sessionStorage pass the owned fixture tests. Conflicting same-origin tabs are
rejected. Site cookies follow browser domain rules, not per-port origin semantics;
contexts and environment references are separate. Identity-provider origin storage
is not retained for crawl replay. Private transfer is limited to 512 KiB and
encoded IPC frames to 1 MiB. At most 16 ephemeral environment logins are retained;
re-login replaces the previous material. **Forget this environment's login** removes
its reference and private state, not its captured evidence.

Forget authenticates the session's immutable project/environment owner, not its
current access or origin binding: you can forget your own expired or origin-changed
login. A copied/foreign reference, undecryptable ciphertext, or locked/unavailable
credential store returns an error and preserves both the reference and private
state. Unlock the store before retrying; there is no blind ciphertext deletion or
plaintext fallback. Already-missing ciphertext (or ended ephemeral state) permits
removing only the selected environment's stale reference. Foreign ephemeral state
is also refused. Forget never deletes the shared installation key. Re-login uses
a fresh session ID; if old-state ownership cannot be authenticated, the new login
is saved but the cleanup error explicitly reports that old private state remains.

Expired access pauses the capture queue with **Log in again**. It does not follow
login redirects or save obvious login forms as baseline content. Complete a new
login for the same saved environment, then resume. Earlier captures remain;
resumed targets disclose mixed capture times. This detection is conservative:
password fields can cause false positives, and custom passwordless login pages
without a detectable access failure may require human inspection. Basic Auth,
device-bound/rotating tokens, arbitrary SSO/MFA, service-worker-dependent apps
and enterprise identity systems are **not universally supported or qualified**.

## Production and staging

Each environment has its own local session reference and isolated replay.
Saved logins are bound to their originally approved destination origins and access
policy, not merely cookie hostnames. Changing a destination's scheme, host or
effective port (or changing access policy) requires **Log in again** before visits;
it never silently visits the new address with old cookies or anonymously. Older
unbound saved logins also require fresh setup. Name and same-origin prefix edits
retain the login but reset mapping probes. This is not a per-path tenant boundary.
Switching the draft environment clears login URL/representative/IdP fields;
enter the intended site's values and save before opening a browser. Those setup
fields describe the currently selected environment, not a portable credential.

Approve source and destination origin/section bases explicitly, for example
`https://production.example/app` → `https://staging.example/preview`. The longest
slash-boundary prefix wins. Ordered/repeated query pairs, encoded path segments
and hash routes remain intact. Every additional origin needs a mapping; subdomains
and identity providers are not inferred. Preview mapped baseline addresses and
probe representative pages before a changed-map full check. Probes inspect at
most the first three selected pages and are not proof of access to every page.

Checks keep the approved logical inventory and profiles fixed; evidence records
the actual mapped requested/final URL. Intentional website mapping is not a browser
or OS mismatch. Discovery currently requires same-website mappings; use mapping
for checks of an existing baseline, not automatic alternate-site rediscovery.

## Capture limits and caution

- Frozen locale, timezone, theme, reduced motion, disabled CSS animations and
  transitions, hidden caret and pointer-away capture reduce incidental variation.
- Defaults: 30 seconds navigation and 15 seconds readiness. Profiles persist their
  exact budgets. DOM completion, font readiness, image decoding and screenshot
  sampling are bounded, not an assurance that every asynchronous app has settled.
- Full page: at most **32,768 pixels tall**, **64 million pixels**, **64 incremental
  scroll steps**, or a lower profile height limit. Width is at most 4,096 pixels.
  Tall/infinite pages fail instead of silently cropping. Choose visible-screen
  capture deliberately when that is the intended coverage.
- At most 512 images are decoded; incomplete fonts/images/resources and changing
  screenshots produce visible warnings. Up to three screenshot samples are used.
- Virtualized rows, nested scrolling, hidden tabs and unvisited UI states are not
  expanded. A screenshot is not proof of complete website coverage.
- OS/architecture/font differences or unstable/incomplete captures produce
  **Rendering caution / Not comparable**, not a trustworthy unchanged release
  verdict. Material profile/runtime differences require matching settings or an
  explicit replacement baseline. The font fingerprint is bounded, not universal.

Read-only accounts are strongly recommended. Discovery does not click buttons or
submit forms, but **GET visits and page scripts can still change server data**.
Action/logout/download links are excluded by default; overriding that policy is
an explicit authorization. Screenshots can contain private website content even
though session material is excluded from project/evidence metadata and exports.

## Existing network, proxy and custom CA

Machine-local settings can configure a credential-free HTTP(S) proxy for runtime
downloads and Chromium, and a PEM CA file (regular file, at most 1 MiB) for runtime
downloads/Node. Chromium also needs that CA trusted by its OS/browser trust store.
There is no TLS-verification bypass. Inherited Node TLS-disable settings are cleared.
Node 22 robots fetch fails closed when a proxy is configured; an authorized robots
override must be explicit. Proxy authentication/automatic VPN setup are unsupported.

VPN uses the existing OS network. A local Rust download-proxy fixture passes;
end-to-end Chromium proxy/custom-CA, real VPN/enterprise networks, OS keychain
persistence, Windows/macOS ownership and current public runtime remain pending.
