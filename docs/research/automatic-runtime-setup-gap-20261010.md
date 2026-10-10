# Automatic first-use runtime: exact gap and safe implementation proposal

**2026-10-10 — read-only source/evidence diagnosis; only this report written.**
No builds, tests, downloads, web requests, app/native/worker/browser launches,
public-site visits, runtime changes, signing, publication or subagents. Source is
a snapshot while the parent edits the UI. Existing native inspection remains
unqualified; this report does not investigate GPU behavior.

## Outcome

The reported error is real product incompleteness, not a user configuration
mistake. `PRODUCT.md:89–91` and `PLAN.md:52,333–375` require app-managed first-use
setup without Node/npm/terminal prerequisites, on all three desktop OS families.

There are **two independent missing pieces**:

1. **Defaults/trust:** `Product::configured` loads only
   `REGRESSEE_DISTRIBUTION`; otherwise `distribution` is `None`. There is no
   embedded production URL, public key, signed release manifest or platform
   catalog. `Product::trust` produces the exact reported error.
2. **Scheduling:** the inspected `Find pages` route goes through
   `Work::Discover` → `Product::discover` → `distribution::resolve`. It does
   **not** call `Product::setup`. Configuring a distribution alone does not make
   first use automatic. A separately invoked setup/repair must already have
   installed a valid ready generation.

Automatic orchestration and honest setup UI can be implemented locally now.
**A usable, no-configuration public runtime for all three OSes cannot be enabled
from the inspected artifacts.** In addition to missing release trust/artifacts,
macOS deliberately blocks even setup smoke because owned-tree supervision is
not implemented. Do not remove that refusal.

## Actual paths and artifacts

| File / entry | Current behavior / implication |
| --- | --- |
| `crates/app/src/product.rs:26–48` | Optional installation-owned local JSON configuration; regular-file/size/type checks, then `Distribution::validate`. No fallback. |
| `crates/app/src/product.rs:68–100` | Status verifies installed files. Setup downloads through configured network settings, invokes private worker handshake/screenshot smoke and bounded stop, then resolves ready status. |
| `crates/app/src/crawling.rs:113–143,450` | Discovery and inventory capture resolve existing runtime, enforce worker identity/scope, then launch. No provisioning. |
| `crates/app/src/auth.rs:433,621`; `product.rs:109` | Login/probe and legacy capture also resolve existing runtime. Apply first-use orchestration to these entry points, not discovery alone. |
| `crates/app/src/jobs.rs:255–263,489–529` | Serial background owner creates Product once. `SetupRuntime` and `Discover` are separate operations; cancellation already belongs to the operation slot. Configuration changes need explicit reload/restart, not an assumed live environment change. |
| `crates/app/src/main.rs` (inspected snapshot) | Setup action queues `SetupRuntime`; availability/status wording checks environment-variable presence. Parent must replace those checks with backend capability/readiness, not assume an embedded default exists. |
| `crates/runtime/src/distribution.rs` | Blocking cancellable streamed ZIP download; signature, OS/arch, scope/support deadline, complete hashes/inventory/modes/contained links, private staging, free-space check, smoke, atomic ready generation/current pointer. Existing verified generations remain separate from evidence. |
| `crates/runtime/src/network.rs:70–82` | TLS stays enabled; redirects rejected; proxy/custom download CA supported. Current total request timeout is 120 seconds, potentially too short for a large first-use pack on a slow connection. |
| `crates/runtime/src/supervisor.rs:167–170` | Explicit macOS owned-process-tree refusal. A valid signed Mac pack still cannot complete product smoke today. |
| `worker/product/src/index.mjs:29–35` | Real handshake verifies exact Node/Playwright/browser/worker identities, launches browser and screenshots an in-memory document. Setup smoke needs no public website. |
| `worker/product/src/capture.mjs:9,43`; `crates/runtime/src/protocol.rs:8` | Maintained product worker `5.0.0`; full `channel:'chromium'`, explicit `chromiumSandbox:true`. Keep headed/headless tuple consistent. |

`.runtime/source/{node,worker,browsers}` is developer builder input;
`.runtime/node-extract`, `npm-cache`, `node-linux-x64.tar.xz`, `link-libs` are
development inputs. `.runtime/pack.zip` and `.runtime/installed` are the historical
spike v1 pack/runtime, signed using the intentionally public development key in
`crates/spike/src/runtime.rs`. They are **not** product release defaults.
Product cache generations live under the installation cache's `product-runtime`,
not `.runtime/installed`. Retained fixture distributions under `artifacts/` use
ephemeral development keys and owned-loopback hosts that need not still exist.
`crates/app/tests/product_flow.rs:250–355` assembles such a development pack from
historical binaries/dependencies plus current product worker sources. None of
these artifacts establishes a qualified public distribution.

## Pins actually present; available is not qualified

`scripts/bootstrap.py` pins Node **22.20.0** with committed archive SHA-256 values
for Linux x64, macOS arm64/x64 and Windows x64. Its exact URL formula is
`https://nodejs.org/dist/v22.20.0/node-v22.20.0-<suffix>`; suffixes and hashes are
in `TARGETS` (lines 8–13). It runs private npm only as developer tooling, with
scripts disabled, and delegates browser downloads to Playwright. **Do not wire
bootstrap.py, npm or Playwright's installer into the end-user setup action.**

`worker/package{,-lock}.json` pins Playwright/core **1.56.1**, with npm tarball
integrity values. Retained `playwright-core/browsers.json` pins full Chromium
**141.0.7390.37 / 1194**, FFmpeg **1011**, and winldd **1007**. Browser metadata
names revisions but does not provide trusted archive SHA-256 values. Historical
distribution registry paths include `builds/chromium/1194/chromium-linux.zip`,
`chromium-mac.zip`, `chromium-mac-arm64.zip` and `chromium-win64.zip`; this report
did not check their current availability. Installed per-file hashes under a
public development signing key do not authenticate a public-safe browser.

Important build gap: `worker/product/package.json` and its lock have **no
Playwright dependency**. Product source imports `playwright`; local fixture packs
borrow `worker/node_modules` from the historical feasibility pack. A release
builder needs its own explicit reviewed product dependency lock/bundle, not this
implicit developer-source dependency.

Retained research, not newly verified here:
`docs/research/release-qualification.md:54–123` and
`artifacts/phase7-local-20261010-sol-7e91/runtime-metadata/observation.json` observed
Node **24.21.0 LTS**, Playwright/core **1.64.0**, full CfT **156.0.8078.4** / PW
revision **1248**, source revision **1706539**. They did not authorize or execute
that tuple. Linux browser `.sha256` and `.sha256sum` probes returned 404; reviewed
Node release-key trust and authenticated browser digest were not obtained.

The retained research reported official browser download availability via
Playwright's CDN redirecting to this direct origin (not a Regressee-owned host):

`https://storage.googleapis.com/chrome-for-testing-public/156.0.8078.4/`

| Candidate Rust OS/arch | Browser suffix from retained metadata | Node archive suffix |
| --- | --- | --- |
| `linux` / `x86_64` | `linux64/chrome-linux64.zip` | `linux-x64.tar.xz` |
| `macos` / `aarch64` | `mac-arm64/chrome-mac-arm64.zip` | `darwin-arm64.tar.xz` |
| `macos` / `x86_64` if supported | `mac-x64/chrome-mac-x64.zip` | `darwin-x64.tar.xz` |
| `windows` / `x86_64` | `win64/chrome-win64.zip` | `win-x64.zip` |

Candidate Node URL prefix is `https://nodejs.org/dist/v24.21.0/node-v24.21.0-`.
These are **candidate upstream locations, not embeddable qualified defaults**.
No new metadata fetch or complete artifact authentication was done here. Do not
substitute CfT paths for the historical Playwright tuple. Modern full-browser
layout, browser ancillary files, optional dependencies and native requirements
must be qualified together. Do not select `/latest`, guess hashes, or treat TLS
availability/npm SRI alone as qualification of the complete runtime.

## Distribution choices without fabricated trust

### A. Existing signed-pack API: smallest product change

Keep `Distribution { version:1, url, public_key, scope:QualifiedPublic }` and
installer. **`url` is the complete ZIP endpoint, not a manifest URL.** The ZIP
contains signed manifest plus payload. To embed usable defaults requires:

- Owner-approved real Ed25519 **public** key (32-byte hex) and externally supplied
  signature; no private key in source or user installation, no agent-generated
  development key promoted as production trust.
- One immutable publicly reachable HTTPS **direct, non-redirecting** pack URL per
  supported OS/architecture. An existing authorized release host can suffice;
  there is no known configured Regressee host here. Ordinary GitHub release/CDN
  redirecting URLs do not work with the current no-redirect client unchanged.
- Matching v3 signed manifest: `id`, `os`, `arch`, `scope`, unexpired
  `support_until`, exact Pinned identity, Node/worker/browser entry points, all
  file hashes/sizes/executable flags, modes and contained bundle links.
- Reviewed upstream provenance, maintained/security-reviewed tuple, target
  licenses and actual sandbox/first-use/native platform qualification. A signed
  `QualifiedPublic` label is an assertion, not independent qualification proof.

`scripts/runtime-pack.py` already prepares unsigned private v3 candidates and
assembles using an **externally supplied** signature/public key. It does not
download, qualify, generate release keys or publish. Neither workflow publishes
trusted packs: `.github/workflows/phase0.yml` is historical developer smoke;
`release-local.yml` is explicitly private checks without uploads/signing.

### B. Bundle worker/library; fetch official components: viable conditional alternative

This can avoid a **new Regressee runtime hosting service**. Ship the reviewed
product ESM worker and resolved Playwright library as app-owned payload (no npm
at runtime), together with an externally owner-signed embedded platform manifest
that authenticates fixed official Node/browser/required ancillary archive URLs,
archive hashes/sizes, target tuple and resulting installed file/mode/link inventory.
The public key and detached manifest signature may ship in the app; the private
key stays with the release owner. No hosted manifest is necessary for fixed
app-release pins. Authenticated app updates deliver future catalog changes.

This is **not compatible with today's one-ZIP downloader merely by changing its
URL**. It needs a separate bounded component provisioner, safe TAR/XZ/ZIP handling,
archive digest checks before extraction/execution, layout normalization matching
the exact Playwright registry, existing file/bundle verification, private staging,
owned screenshot smoke and atomic publication. Preserve native signatures and
contained Mac framework links. Do not run installers, npm scripts or privileged
dependency commands; do not remove quarantine. Use exact direct HTTPS origins,
not generic redirect-following. Bind bundled worker/library bytes into the same
runtime identity/inventory; adding source code alone does not provide Playwright.

**Still blocked:** no reviewed current component digests/signature set and no
qualified all-platform tuple are present. Missing browser sidecar checksums are
not proof the design is impossible: an owner-approved provenance/release process
could authenticate exact upstream artifacts and sign their independently reviewed
digests. That process is external authorized work, not a local self-signing shortcut
or merely hashing an unverified download. It is not performed/approved here.
This alternative also revises PLAN's prebuilt-pack distribution choice and needs
an explicit architecture decision. Prefer A for minimal changes if approved
existing hosting is available; B is more implementation, not immediate unblock.

## Minimal safe API/orchestration parent can wire now

Suggested additions (proposal only; no app source changed):

1. **Backend capability/readiness**, in `crates/app/src/product.rs`: typed status
   separating `UnavailableDistribution`, `UnsupportedPlatform`, `NotInstalled`,
   `Verifying`, `Ready { identity, scope }`, `RepairRequired`, `Expired`, `Failed`.
   Keep existing error causes for diagnostics. Read capability from Product, not
   `std::env::var_os` in UI. No default entry must still mean unavailable, not
   silently development-ready. A catalog API can return `None` until real release
   material exists; placeholders must not become enabled download entries.
2. **First-use prerequisite**, e.g. `Product::ensure_runtime(cancel, progress)`:
   use verified ready runtime when present; if current pointer is genuinely absent
   and supported trusted distribution exists, invoke existing setup once. Recheck
   cancellation before continuing the original action. Distinguish absence from
   damage/expired signatures/path refusals; do not catch every resolver error and
   blindly download/overwrite. Explicit repair remains available and atomic.
3. **One owner operation**, in `crates/app/src/jobs.rs`: prerequisite + original
   Discover/OpenLogin/Probe/Capture action under the same request ID/cancel slot,
   not two independently queued requests that reset cancellation or use stale
   UI state. Keep persisted project revision/selected environment checks. For a
   baseline check/resume, require its pinned runtime; never automatically install
   a new default and silently change rendering. New discovery may use the selected
   supported default. Runtime status/diagnostics must not trigger downloads.
4. **Typed setup progress**, in `crates/runtime/src/distribution.rs` and Jobs:
   `Downloading { received, total }`, `Verifying`, `Extracting`, `TestingBrowser`,
   `Ready`, with bounded/throttled background events. Today the download callback
   reports bytes only; Jobs discards the denominator into `0 / Some(1)` and labels
   downloading as verifying. Emit phase transitions around actual stages; retain
   byte totals in replies. Unknown total means indeterminate progress, not a fake
   percentage. App/catalog can supply reviewed size estimates; current installer
   learns manifest size only after the pack download. Cancellation stops both
   setup and the requested website action, leaving old runtime/evidence untouched.
5. Parent-owned UI can show the first-use explanation/progress/retry/cancel and
   resume Find pages only after Ready. When trust is absent, clearly say this build
   cannot yet install its required browser; do not present environment-variable
   instructions as the intended non-developer product solution.

The existing free-space check runs after download; an authenticated catalog with
pack/extracted sizes would enable a meaningful early disk preflight. Separately
review bounded download inactivity/overall deadlines for slow networks rather
than disable timeouts/TLS. Neither enhancement manufactures trust.

## Hard dependencies and honest completion boundary

- **Release trust/artifacts:** owner-selected current tuple, authenticated pins,
  signing custody/rotation/support policy and qualified platform artifacts;
  either authorized existing pack hosting (A) or approved signed component catalog
  (B). None exists as an app default in this snapshot.
- **macOS code, not just CI:** `crates/runtime/src/supervisor.rs` refuses launch;
  `docs/research/macos-owned-process-design.md` explains detached browser/Crashpad
  ownership and deadline gaps. Do not equate jobs/compilation/bundle v3 support
  with owned-tree closure. Needs a proven supported ownership design and actual
  Mac implementation/testing, plus signatures/Gatekeeper/notarization.
- **Windows and Linux:** Windows Job code exists but full standard-user native
  runtime/installer qualification is not established. Ubuntu package/default-policy
  Chromium isolation and host-library prerequisites remain unqualified;
  `docs/support-matrix.md` records current Linux ABI mismatch. Do not use the
  Fedora-like development source cache as a supported distribution.
- All three OS families remain required; exact supported architectures must be
  declared. Existing native/UI/GPU inspection remains separately unqualified;
  no broad guardian/soak/phase rerun is proposed by this report.

**Locally deliverable now:** automatic orchestration for explicitly trusted
supported packs, typed setup/readiness/progress and actionable first-use UI,
release-catalog plumbing with no fake enabled defaults, explicit product worker
dependency packaging design. **Not deliverable truthfully now:** default public
Find pages on all three OSes using the repository's historical runtime, an invented
release endpoint/key, or a browser obtained/executed without approved trust.
Never weaken TLS, sandbox, signatures, private-path/link validation, process
ownership, baseline identity checks or DevelopmentLoopback public-capture guards.
