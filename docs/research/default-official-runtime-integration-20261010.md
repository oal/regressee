# Default official runtime integration — 2026-10-10

## Application behavior

`Product::configured` selects the immutable app-embedded official catalog when
`REGRESSEE_DISTRIBUTION` is absent. The application pins Node **24.21.0**,
Playwright **1.64.0**, Chromium **156.0.8078.4**, and its protocol worker version.
The catalog contains the official TLS archive URLs, archive sizes/checksums,
complete installed file/link/mode inventory, support deadline, and checksum of
the embedded worker/library bundle. It is not an externally supplied catalog
and does not impersonate a signed distribution with a substitute key.

An explicit `REGRESSEE_DISTRIBUTION` continues to select **only** the legacy
signed-distribution path. Invalid, missing, unsafe, or app-pinned-scope override
configuration is an error, never a reason to silently select official downloads.
The legacy `Product::trust` accessor remains legacy-only.

Linux x86_64 and Windows x86_64 retain adapter eligibility. macOS and other
architectures remain unsupported; the presence of macOS catalog assets does not
override the owned-process-tree supervision gate.

## Setup and evidence boundaries

Readiness and `resolve_runtime` are read-only. A valid uninstalled default says
“The private browser will be installed automatically on first use.” Verified
installed state says “Private browser is ready.” Neither path creates a cache,
downloads, or executes a browser just to answer readiness.

Existing eligible first-use actor requests call `ensure_runtime` under the same
request ID and cancellation slot, then continue with discovery/login/probe or
new-baseline work. No extra setup request is required. Both installers use the
same owned-worker handshake (including its in-memory headless screenshot smoke),
network settings, progress phases, and five-second shutdown deadline.

The existing baseline/check/resume policy is unchanged: pinned comparisons only
resolve; they never automatically install a replacement. Existing generation
verification checks the complete app-pinned inventory. A damaged/unsafe pointer,
missing pointed generation, expired catalog, or mismatched runtime identity
cannot trigger automatic repair or trust-path fallback. Explicit repair remains
separate. The cache stays `product-runtime`, including when old signed-pack
generations exist, so a trust-path change cannot quietly replace old evidence's
runtime. Archives, login storage, and evidence schemas are unchanged.

Diagnostics use the finite `VerifiedOfficialPinned` state rather than calling
app pins `QualifiedPublic`. Only numeric versions are copied; catalog URLs,
keys, hashes, paths, and arbitrary errors are not diagnostic fields.

## Test isolation and qualification

Literal development/unconfigured fixtures explicitly set
`official_runtime: false`. `Jobs::start_with_product` and
`Jobs::start_without_runtime` are available only under `cfg(test)` or `ui-tests`;
production `Jobs::start` always uses the real configured source. UI fixture tests
must opt into the no-runtime actor rather than accidentally exercising downloads.

Added focused tests cover default read-only readiness, explicit override
precedence/rejection, unsafe and missing pointers, cancellation, catalog expiry,
and diagnostic privacy without any network/browser execution. Existing signed
inert-fixture installation guards remain on the legacy path.

This source integration is not an all-OS public release or browser-sandbox
qualification claim. Fresh Linux owned-browser setup smoke and platform/release
qualification belong to the parent verification flow. This agent performed no
build, test run, download, browser execution, or public-site visit.
