# Private browser setup

The default Linux/Windows x86_64 path bundles Playwright and the worker with the
app, then downloads pinned official Node/Chromium archives privately on first use.
No Node/npm installation, terminal configuration or Regressee download server is
required. Exact HTTPS URLs, hashes, lengths and the complete resulting file
inventory are embedded application trust, not cache-generated metadata.
**Install browser now / Repair browser** uses the same background service and
sandboxed screenshot smoke before activation. macOS remains blocked by the
unimplemented owned-process-tree supervisor; catalog presence is not OS support.

Current pins are Node 24.21.0, Playwright 1.64.0 and Chromium 156.0.8078.4, with
support through 2027-01-10 UTC. `OfficialPinned` is distinct from release
qualification. See [decision 0011](decisions/0011-app-pinned-official-runtime.md)
and the [asset provenance](research/official-runtime-assets-20261010.md).

## Automatic first-use workflow

With the app-pinned default or an explicitly trusted supported distribution, new Find pages, new-baseline
capture and eligible login/probe work now install a genuinely missing runtime
within the original background request. Download, verification, installation and
browser-test progress are separate real stages. Cancellation also prevents the
original website action from continuing. Read-only status never downloads.

Damaged/expired installations require explicit repair; comparisons and resumes
do not install a different default behind a baseline's back. Settings reads typed
backend capability/readiness, not the presence of an environment variable.

The historical missing-default report is preserved as earlier evidence, not the
current architecture. Actual test/qualification scope is recorded in the latest
official-runtime reports; default availability alone is not clean-machine,
Windows/macOS, native UI or public-release acceptance.

## Optional legacy signed-pack override

Ordinary users need no override. For an explicitly configured signed installation,
`REGRESSEE_DISTRIBUTION` names a local
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
They use Regressee's per-user 0700 namespace `/tmp/rg2-EUID`, independent of
development tools or other applications' temporary directories.
Only that namespace is inspected for dead-owner recovery.
An optional installation override, `REGRESSEE_WORKER_SCRATCH_PARENT`, names the
**complete namespace**, not project data. It must be absolute, non-symlink,
current-user-owned mode 0700, with safe ancestors and at most 40 path bytes.
Missing final directories are created privately; invalid paths fail explicitly,
not by permission repair or silent fallback. Each unique `w-XXXXXX` worker path
is limited to 49 bytes, leaving room for Chromium's Linux socket/profile suffix.

Earlier previews incorrectly preferred `/tmp/opencode` when that directory existed
and belonged to the current user. Unsafe ancestor permissions could then block
browser setup after the download completed. That automatic preference is removed;
Regressee has no runtime dependency on OpenCode. No existing temporary directories
are migrated, deleted or permission-repaired by this change. Explicit overrides
still undergo the same strict validation and do not silently fall back.

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
REGRESSEE_PHASE2_EVIDENCE=/absolute/path/to/new-private-evidence \
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
