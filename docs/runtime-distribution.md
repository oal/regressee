# Product runtime setup — implemented interface, pending public distribution

Regressee’s **Install browser now / Repair browser** action calls the actual signed
download/install/smoke/repair service on its background owner. It does not run npm,
use a global Node or download an executable without verifying its signed file
inventory. No production host/key is currently bundled. This is not a release
configuration or a claim that the historical browser is safe for public sites.

## Automatic first-use workflow

With an explicitly trusted supported distribution, new Find pages, new-baseline
capture and eligible login/probe work now install a genuinely missing runtime
within the original background request. Download, verification, installation and
browser-test progress are separate real stages. Cancellation also prevents the
original website action from continuing. Read-only status never downloads.

Damaged/expired installations require explicit repair; comparisons and resumes
do not install a different default behind a baseline's back. Settings reads typed
backend capability/readiness, not the presence of an environment variable.

**No trusted public default is bundled yet.** Ordinary public-site automatic setup
is still an unfinished product requirement, not a request for users to install
Node/npm or configure developer tools. The implementation tests use signed inert
inventories and do not qualify a live runtime or all three operating systems.
See [gap analysis](research/automatic-runtime-setup-gap-20261010.md) and
[implementation scope](research/automatic-runtime-setup-implementation-20261010.md).

## Installation configuration

For an explicitly configured installation, `REGRESSEE_DISTRIBUTION` names a local
JSON file (at most 16 KiB; regular file; unknown fields rejected):

```json
{
  "version": 1,
  "url": "https://YOUR_APPROVED_DISTRIBUTION_HOST/EXACT_PINNED_PACK.zip",
  "public_key": "YOUR_64_LOWERCASE_HEX_ED25519_PUBLIC_KEY",
  "scope": "QualifiedPublic"
}
```

This is a template, **not usable trust material or an existing endpoint**. The
key is public; no production private key belongs in the app/project/settings.
QualifiedPublic requires matching signed scope, an unexpired support deadline
and independently qualified supported runtime. Hosting/key custody/rotation and
platform qualification remain release gates. Changing a runtime requires matching
baseline compatibility or an explicit new complete baseline.

Development uses `DevelopmentLoopback`, with HTTP allowed only for literal
loopback download endpoints. Captures are also restricted to owned literal-loopback
fixtures, including resource/navigation safeguards. Do not use this mode to
claim arbitrary local/internal or public-site safety. A dev-scope signed pack
cannot be unlocked merely by changing the configuration to QualifiedPublic.

## Signed pack v2

ZIP contains `manifest.json`, its 64-byte detached `manifest.sig`, and exactly its
listed regular files. Manifest fields are `version`, `id`, `os`, `arch`, `scope`,
`support_until`, `identity` (Pinned Node/worker/Playwright/browser), `node`, `worker`,
`browser_root`, and `files` (SHA-256, bytes, executable). Entry points are relative,
not project-controlled paths. Absolute/traversing/aliased/duplicate/case-colliding
names, unsupported directory/link entries, unlisted files, oversized or corrupt
streams and incompatible platform/scope/signature are rejected before smoke.

Limits: 2 GiB compressed/extracted, 20,000 listed files and 4 MiB manifest. Extract
to private owned staging, verify complete hashes and inventory, perform actual
sandboxed product screenshot smoke, then publish a fresh ready generation and
atomic `current` pointer in the app cache. Failed download/extraction/smoke leaves
the old pointer intact; originals live separately in app data. Old cache generations
are retained rather than recursively deleted. Verification refuses damage and
offers repair. Proxy environment support comes from the HTTP client; dedicated
managed-network/custom-CA UI is not implemented and TLS verification is never
disabled.

## Linux worker scratch policy

Product workers do **not** inherit `TMPDIR`/`TMP`/`TEMP` for browser scratch.
They use a per-user 0700 namespace `/tmp/rg2-EUID`; if the real, current-user-owned
`/tmp/opencode` harness directory exists, its approved `/tmp/opencode/rg2-EUID`
is used instead. Only that namespace is inspected for dead-owner recovery.
An optional installation override, `REGRESSEE_WORKER_SCRATCH_PARENT`, names the
**complete namespace**, not project data. It must be absolute, non-symlink,
current-user-owned mode 0700, with safe ancestors and at most 40 path bytes.
Missing final directories are created privately; invalid paths fail explicitly,
not by permission repair or silent fallback. Each unique `w-XXXXXX` worker path
is limited to 49 bytes, leaving room for Chromium's Linux socket/profile suffix.

The private 0600 v3 marker binds UID, exact scratch path, app/worker PID and
canonical runtime executable/browser root. Cancellation, shutdown, Drop and
dead-parent recovery share this policy. Recovery preserves live owners, malformed
or foreign markers, symlinks and unmarked/historical entries. Linux cleanup signals
only individual owned PIDs matching UID, executable and the complete scratch
environment item; it never signals process groups. PID reuse/permission ambiguity
is conservative, not permission to delete evidence. This is not protection from a
malicious same-user filesystem racer, nor Windows/macOS ownership qualification.

## Reproduce the actual local signed distribution flow

```sh
# Requires the already verified historical development source pack, never a
# system Node/npm. Builds UI/headless binaries first, then runs serially.
bash scripts/build-ui.sh
REGRESSEE_PHASE2_EVIDENCE=/tmp/opencode/regressee-product-flow-UNIQUE \
  cargo test --locked -p regressee-app --features ui-tests --test product_flow \
  actual_signed_http_product_flow -- --ignored --nocapture --test-threads=1
```

The test verifies historical source signatures/hashes, signs the new product pack
with an ephemeral development key, serves it on an owned loopback HTTP endpoint,
installs via the product service and tests real captures, failures, cancellation,
crashes, repair and persistence. It retains only the public trust configuration,
pack and evidence; no production signing key or external service is created.
The fixture server ends with the test, so its retained URL is not an enduring host.

`regressee-flow DATA_ROOT setup|status|new URL|capture PROJECT_ID|check PROJECT_ID|
activate RUN_ID REVISION|evidence PROJECT_ID` uses the same services as GPUI. It is
developer verification, not a terminal requirement for the intended app workflow.
Python scripts are optional owned-control/test drivers only, not the app path.
