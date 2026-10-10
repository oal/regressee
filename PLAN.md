# Regressee — self-contained implementation plan

## 1. Purpose and how to use this plan

Build a local, multiplatform Rust + GPUI desktop app that lets non-developers
capture websites, compare releases, review differences, and maintain/share
baselines without writing tests or installing developer tools themselves.

This plan does not depend on the interview transcript. Read `PRODUCT.md` for
confirmed product requirements, this document for shared contracts, and the
appropriate file in `docs/phases/` for an executable phase checklist.

**Authority:** confirmed requirements take precedence over proposed defaults.
Defaults below are recommendations to make implementation concrete, not extra
user commitments. Any change to confirmed scope requires an explicit decision.

**License:** MPL-2.0. **Working name:** Regressee, not yet separately confirmed.
**Delivery:** all three desktop operating systems in the first public release.

### Execution update — 2026-10-09

The user approved Python controls and requested continuation through the
remaining phases after the earlier Phase 0 stop. Implement and independently
verify phases serially while tracking unrun external/native qualification;
never call those checks passed. The current execution policy is
`docs/decisions/0005-serial-local-continuation.md`, and the pipeline index is
`docs/IMPLEMENTATION_STATUS.md`. Full release acceptance still requires the
original scope and gates. Earlier stop recommendations remain historical, not
the current instruction to halt all local development.

## 2. Confirmed scope

- Multiple local projects; public, authenticated, staging, and internal sites.
- Baseline sources: a base URL or list of URLs used for discovery.
- Typical scale: 100–1,000 pages, multiplied by selected screen sizes.
- Guided crawl controls, page-list preview, and advanced controls.
- Selectable speed/parallelism and mobile/tablet/desktop profiles.
- Full-page captures by default; visible-screen captures as an option.
- A baseline establishes a fixed page inventory. Comparison runs do not crawl
  beyond it or discover newly added pages.
- Compare the same address over time or matching paths in another environment.
- Interactive browser login and consent setup; no scripted journeys.
- Optional photo/image replacement selected before capture, consistent between
  baseline and comparison.
- Post-capture sensitivity adjustments, multiple heuristics, and drawn ignored
  rectangles per page/screen size.
- Selected-result approval creates a new baseline version; whole-baseline
  replacement is also possible.
- Local history remains until manually deleted.
- ZIP sharing of project configuration plus a selected baseline; no credentials
  or saved sessions.
- Automatic first-use download of a private runtime with no terminal required.
- Sleek, modern, approachable UI; MPL-2.0 application licensing.

### Explicitly outside the first release

Cloud accounts, shared online review, schedules/CI integrations, arbitrary user
scripts, recorded journeys, automatic login, automatic discovery during a
comparison, reusable selector-based ignore rules, AI verdicts, real-device
testing, additional browser engines, and comparison-history export.

HTTP Basic Auth configuration is not included in the initial authentication
scope. Staging/internal sites must initially be reachable through ordinary
navigation and the chosen interactive session mechanism. Document unsupported
protection schemes rather than pretending interactive login handles all of them.

## 3. User experience and workflows

### Product language

Use labels such as **Create project**, **Find pages**, **Save baseline**,
**Check for changes**, **Accept selected changes**, and **Capture new baseline**.
A brief explanation can call a baseline “the approved reference screenshots.”
Show “Changed,” “No significant change,” “Could not capture,” and “Needs login,”
not a misleading universal pass/fail label. A difference is evidence to review,
not proof of a defect.

### Primary screens

1. **Projects:** project names, site addresses, current baseline, last check,
   status, and one clear next action. Search/filter for larger project lists.
2. **Project setup:** addresses → scope → screen sizes → optional login/consent
   → discovery preview → capture confirmation. Allow editing without recreating
   the project. Show advanced settings only when requested.
3. **Capture progress:** phase, completed/total targets when known, page and
   screenshot counts separately, errors, thumbnails, and pause/cancel controls.
4. **Results:** changed-first queue, filters, URL search, screen-size grouping,
   clear incomplete-coverage indicators, and summary counts.
5. **Comparison viewer:** synchronized before/after, overlay slider, diff
   heatmap, zoom/pan, next-change navigation, sensitivity, ignored rectangles,
   notes/review state, and explicit acceptance.
6. **History and storage:** baseline versions, runs, restore/set-active actions,
   storage usage, deletion impact preview, and ZIP export/import.
7. **Settings/setup repair:** runtime status, downloads, proxy configuration,
   diagnostics, and recovery actions in ordinary language.

### Visual direction boundary

The surface is an operational tool: clarity, scanability, predictable behavior,
and native conventions outrank decoration. “Sleek and modern” is confirmed;
specific typography, colors, spacing tokens, and component styling are not.
Phase 1 establishes and confirms a small visual system rather than baking in an
unreviewed aesthetic. Design all material states, not only happy-path mockups.

### Core flow: new baseline

Create project → enter source addresses → choose scope and screen sizes →
log in/handle consent if needed → find pages → review/include/exclude pages →
confirm page count and estimated capture cost → capture → review coverage →
explicitly save the baseline.

A URL list gets a plain-language choice: **Only these addresses** or **These
addresses and their links**. Proposed default: links enabled, matching the
original crawl requirement. This is an intentional discovery mode, not a
change to comparison behavior.

### Core flow: release check

Choose baseline → select same or alternate environment → validate runtime,
capture settings, login, and access → visit the baseline's fixed target list →
inspect results → adjust analysis/ignore regions without recapture → mark
intentional/unwanted changes → optionally accept selected changes into a new
baseline version.

### Core flow: replacement baseline

Choose **Capture new baseline** → choose existing inventory or explicit new
discovery → confirm settings and coverage → capture → explicitly activate the
new version. Existing baselines are never silently overwritten.

### Core flow: sharing

Select baseline → preview included screenshots/settings and privacy warning →
export ZIP → recipient imports as a local project → checks compatibility →
sets their own environment/login → checks the fixed baseline targets.

## 4. Proposed user-facing defaults

These are starting values to validate in Phases 0–5, not performance promises.

| Setting | Proposed default and control |
| --- | --- |
| Page limit | 500; clearly state when discovery stopped at the limit; allow 1,000+ explicitly |
| Discovery scope | Starting origin(s) only, no automatic subdomain expansion; optional section prefix |
| Exclusions | Text-based URL exclusions with a preview and explanations |
| Depth | Unlimited within the page cap; optional “links away from the starting page” limit |
| Tracking parameters | Ignore known tracking parameters; preserve other query parameters |
| Speed | Gentle: 1 active page, 1 s start spacing; Balanced: 2, 350 ms; Fast: 4, no added spacing |
| Advanced parallelism | 1–8 active page operations initially; bounded by memory and per-site safety |
| Screen sizes | Desktop 1440×900, tablet 768×1024, mobile 390×844 CSS pixels; select one or more |
| Device settings | DPR 1 initially; fixed desktop/mobile Chromium context settings recorded in the profile |
| Capture extent | Full page; optional viewport only |
| Comparison sensitivity | Balanced preset, with plain-language explanations and advanced numeric controls |
| Photos/images | Normal content; optional replacement mode requires matching baseline |
| History | Keep until explicitly deleted; no automatic evidence retention policy |

Concurrency means active navigations/captures, not the number of individual
HTTP requests; each page can load many resources. Warn about load on production
sites and slow down on rate limits. Performance estimates should be ranges
based on sampled captures, never false-precision completion times.

The screen-size presets use Chromium emulation, not actual Safari/iOS devices.
Record exact user agent, `isMobile`, touch, viewport, screen, and DPR values;
these must never change silently after a baseline is established.

## 5. Architecture

### Responsibility split

**Rust owns:** GPUI interface, domain state, crawl policy/scheduling, job
supervision, runtime setup, URL identity/environment mapping, SQLite storage,
artifact lifecycle, image analysis, review/baseline versions, ZIP interchange,
and secret management.

**A private Node worker owns:** Playwright browser launch, interactive login,
page navigation, in-browser link extraction, stabilization, screenshot capture,
and image-replacement transformations. Use the Playwright library rather than
generating test files or delegating the product to Playwright Test's report UI.

### Suggested repository layout

```text
Cargo.toml                       # workspace; exact dependency pins/lockfiles
rust-toolchain.toml
LICENSE                         # official MPL-2.0 text
crates/
  app/                          # GPUI screens, actions, view models
  core/                         # domain types, URL policy, job orchestration
  storage/                      # SQLite, migrations, artifacts, history
  runtime/                      # verified downloads, secrets, process supervisor
  compare/                      # diff metrics, masks, analysis policies
  interchange/                  # archive schemas, export/import validation
worker/
  src/                          # TypeScript Playwright worker
  package.json
  package-lock.json
tests/
  fixtures/                     # deterministic local test websites
  images/                       # licensed/generated comparison corpus
  integration/
docs/
  phases/
  decisions/                    # short records for consequential choices
```

Modules may begin smaller than this layout. Preserve the responsibility
boundaries; do not create a microservice architecture or unnecessary crates.

### GPUI and background work

Pin a tested GPUI/gpui_platform version or exact compatible revision, not a
floating git branch. GPUI is pre-1.0 and APIs can break. Evaluate components for
text input, focus, menus, dialogs, and accessibility before committing to a
component library; validate licenses of any reused code.

Database I/O, image decoding, diffing, ZIP work, and process waits must stay off
the UI thread. Use bounded channels/work queues and GPUI-safe event updates.
Virtualize result lists and render large screenshots using tiled derivatives;
never upload an arbitrarily tall image as one GPU texture.

### Worker protocol

Versioned newline-delimited JSON over private stdio, with request IDs, job IDs,
typed events, payload-size limits, and explicit acknowledgements. Commands
include handshake, login setup, discover page, capture target, cancel, and
shutdown. Events include progress, links found, target captured, warning,
requires-login, and typed failure. Diagnostic logs go to stderr, not the
protocol stream, and must redact secrets.

The Rust scheduler owns the authoritative queue. The worker does not silently
start extra crawl jobs. Pass command arguments directly, not through a shell.
Imported projects cannot add executable scripts or arbitrary worker commands.

## 6. Domain and persistence contracts

Use SQLite with versioned migrations and immutable on-disk artifacts. Store
artifacts outside the database using content-addressed IDs, atomic writes,
integrity hashes, and reference tracking. Use OS-appropriate app-data/cache
directories; no global npm, PATH edits, or writes to the user's website source.

| Entity | Required responsibility |
| --- | --- |
| Project | Name, sources, discovery settings, environments, selected profiles, active baseline |
| Environment | Explicit approved origin/base-prefix mapping, secret-session reference, access policy |
| Inventory revision | Fixed canonical page identities, query/hash rules, inclusion decisions, discovery coverage |
| Capture profile | Immutable viewport/emulation, extent, stabilization, image mode, browser/runtime version |
| Capture run | Intent, inventory/profile snapshot, environment fingerprint, targets, lifecycle, timestamps |
| Capture target | Logical page + screen/profile key; requested/final URL, status, attempts, artifact and warnings |
| Baseline version | Immutable map of target keys to successful source captures; parent, provenance, coverage |
| Analysis revision | Baseline/run IDs, algorithm version, policy thresholds, mask revision, metrics and classification |
| Review decision | Per-result intentional/needs-fix/unreviewed state, note, date, local reviewer label if useful |
| Ignore-mask revision | Rectangles keyed by logical page + profile, coordinates, provenance, visible coverage |
| Artifact | Original lossless screenshot, derived tiles/thumbnails/diffs, size, hash, references |
| Secret session | Encrypted browser state outside exported project data, scoped to project/environment |

### Baseline invariants

- Original screenshots and completed run records are immutable.
- A baseline contains successful captures, not screenshots of capture errors.
- Creating/replacing/approving a baseline requires explicit confirmation.
- The default activation path requires full intended coverage. If a user saves
  a partial baseline, explicitly define its reduced coverage; never imply the
  omitted pages were verified.
- Partial approval creates a new version inheriting unapproved references and
  replacing only selected page/profile keys. Show the inherited count and
  capture dates; a composite baseline is not one simultaneous website snapshot.
- Failed, missing, or materially incompatible captures cannot be approved.
- Partial approvals cannot mix incompatible capture profiles or rendering
  environments. Require a complete compatible replacement when changing those
  conditions. An intentional production/staging URL mapping alone is not a
  rendering mismatch and does not prevent selective approval.
- Setting a historical baseline active does not mutate past runs or review
  records. Reference preservation makes it reversible until manual deletion.

### Job lifecycle and recovery

Use persisted states: queued, running, pausing, paused, completed,
completed-with-errors, cancelled, and interrupted. Target outcomes are separate
from whole-job lifecycle. On restart, mark previously active jobs interrupted,
verify stored artifacts, and offer resume of unfinished targets.

Pause stops new work and lets active tasks reach a safe boundary. Cancel stops
active work after a bounded grace period, preserves valid completed evidence,
and does not activate a partial baseline. Retries are bounded and limited to
recoverable failures. Resuming requires the same frozen settings and validates
session/runtime availability; changes create a new run.

The resumed run records interruption/resume times and warns that evidence was
captured at different times. Do not silently merge captures across a changed
release/environment; offer a fresh run instead when continuity is uncertain.

## 7. URL discovery, identity, and environment mapping

- Extract actual links from rendered pages. Do not click arbitrary controls,
  submit forms, follow JavaScript action links, or guess hidden admin routes.
- Limit page navigations to explicitly approved source origins and optional
  path sections. CDN resources may load normally; page scope is not a blanket
  resource firewall.
- Resolve relative URLs, deduplicate safely, skip non-HTTP(S) schemes/downloads,
  and keep an exclusion reason for preview/debugging.
- Drop ordinary in-page anchors but preserve recognized hash-router routes and
  explicit fragment URLs when necessary. Test SPA identity, query parameters,
  repeated query keys, encoded paths, trailing slashes, and redirects.
- Preserve meaningful query parameters. Tracking removal rules are explicit
  and versioned; do not sort/rewrite URLs in a way that changes server behavior.
- An explicit URL list can seed multiple origins, but each origin must be
  visible/approved. Multi-origin environment mapping is explicit.
- Keep requested and final URLs. Deduplicate discovery aliases cautiously;
  during comparison, report unexpected redirects instead of silently matching
  a different page.
- A production/staging mapping replaces an explicitly selected origin/base
  prefix and preserves the relevant relative path/query/route. Preview mapped
  URLs and probe representative pages before a large run.
- Do not discover new pages during comparisons. HTTP 404/410, access denial,
  unexpected redirect, and login expiry are their own outcomes.
- Proposed public-site policy: respect robots.txt by default; a visible,
  intentional override is available for authorized staging/internal testing.
  Robots rules are not authorization. Explain any resulting coverage limit.

### Admin-panel safety

Warn that visits can themselves have server-side effects; “we do not click
buttons” is not a guarantee of read-only behavior. Recommend a read-only test
account. Surface suspicious logout/delete/action-like links as excluded by
default, with an explicit override and exclusion preview. Never guess that a
URL is safe merely because navigation uses GET.

Do not automatically authorize an external redirect just because it appeared
in discovered content. Interactive login may navigate to an identity provider;
capture navigation remains scoped and signals an expired session instead of
turning that identity provider into crawl content.

## 8. Runtime installation and authentication

### Private runtime

Build platform-specific runtime packs in trusted release CI: pinned Node,
compiled worker, Playwright library and metadata, and matching Chromium.
Users do not run npm or receive a generated developer project.

The app downloads a versioned manifest/pack on first use, verifies its signature
and checksums, extracts safely, smoke-tests browser launch plus a screenshot,
and atomically marks it ready. Show disk/download estimates, progress, cancel,
retry, and repair. Downloads use TLS; do not “fix” certificate failures by
disabling verification. Support a configured proxy/custom trusted CA where
feasible, with diagnostics for managed networks.

Use the full pinned Chromium build with consistent new-headless capture and
headed interactive setup. Validate this choice in Phase 0. Do not rely on an
unmanaged system browser or change headless mode between baseline and check.

Keep runtime versions available when needed for existing compatible baselines,
subject to a documented security/support policy. A security update can require
baseline migration; never silently change browser rendering beneath a baseline.
Runtime caches are separate from immutable evidence and may be repaired or
redownloaded without deleting runs.

### Cross-platform qualification

The three OS families are committed; exact versions/architectures are not.
Phase 0 must publish a tested support matrix. Suggested qualification targets
are Windows 11 x64, macOS 14+ on Apple Silicon and Intel if supported by the
chosen GPUI/runtime build, and Ubuntu 24.04 x64. Include the Fedora-based
development environment in Linux packaging investigations.

Playwright officially supports particular Linux distributions, not arbitrary
Linux installations. Compare AppImage and Flatpak/runtime approaches against
clean-machine, browser sandbox, filesystem, GPU, and session requirements.
Choose and document the supported packaging path in Phase 0. Additional distro
or architecture support is a tested claim, not an automatic consequence of Rust.

Linux OS libraries/GPU drivers can remain host prerequisites. Prefer packaging
that supplies user-space dependencies; otherwise provide a guided, explicit
permission/install flow on supported distributions. Never run opaque privileged
commands silently or disable Chromium's sandbox as a universal workaround.

### Interactive sessions

Open the managed browser for the chosen environment; the user logs in, handles
MFA/SSO and consent, then clicks **I'm ready** in the app. Confirm with a
representative page before accepting the session.

Save cookies, local storage, supported IndexedDB state, and specifically tested
session-storage state as necessary. Playwright storage-state alone is not a
universal solution: rotating tokens, device-bound sessions, or some enterprise
flows may not replay successfully. Test real fixtures and expose unsupported
cases. For expiry, pause/signal **Log in again** rather than diffing login pages.

Encrypt session material with a per-install key protected by the OS credential
store. If a secure store is unavailable on Linux, offer an ephemeral session or
an explicit secure alternative, never silent plaintext persistence. Use private
temporary files only when unavoidable, with permissions and cleanup; keep
secrets out of logs, URLs where possible, and archives. Production and staging
use separate sessions. Do not export browser profiles.

## 9. Capture consistency and image replacement

### Stabilization pipeline

1. Navigate with bounded timeouts and record status/final URL.
2. Check access and obvious login/redirect failures before accepting evidence.
3. Apply fixed profile settings: viewport/emulation, locale, timezone, color
   scheme, reduced motion, animation/caret handling, and hover avoidance.
4. Wait for DOM/application readiness using bounded checks; do not rely solely
   on `networkidle`, which can hang on live apps.
5. Wait for fonts and relevant image decoding, with failure diagnostics.
6. For full-page mode, bounded incremental scrolling triggers lazy loading.
   Bound time, height, and iterations so infinite-scroll sites cannot run forever.
7. Return to the capture position, apply the selected replacement transform,
   and check bounded consecutive screenshot stability.
8. Save the screenshot and capture metadata. An unstable page gets a warning
   or a failed target according to the selected policy, never hidden confidence.

Prototype defaults: 30 s navigation timeout, 15 s additional stabilization
budget, at most three screenshot stability attempts. Calibrate these with
fixtures. Record actual settings, retries, duration, readiness warnings,
image dimensions, runtime, and environment fingerprint.

Full-page limits must be explicit. Use lossless originals, bounded decoding,
row/tile-based comparison where necessary, and generated viewer tiles. If a
page exceeds browser/image safety limits, fail with an understandable action
or use a qualified tiled capture mode. Never silently crop and call it full-page.
Nested scroll containers and virtualized lists require documented limitations;
full-page capture is not a promise to reveal every hidden app state.

### Photo/image replacement contract

“Photo” is not a reliable browser element type. Start with a clearly explained
**Replace image content** mode, not a claim of perfect photo recognition.

Proposed first implementation: neutral placeholders for HTML raster images,
responsive/picture images, and supported raster CSS backgrounds. Preserve
element size, layout, object-fit/position behavior, borders, and spacing. Retain
SVG/vector icons and explain that raster logos/illustrations may be replaced.
Use a preview on representative pages before baseline capture.

Do not indiscriminately block image downloads before layout is established:
intrinsic dimensions and lazy-loading logic can affect geometry. Apply the
transformation after readiness, using tested dimension-preserving rules.
Canvas, video, cross-origin frame contents, shadow DOM, pseudo-elements, and
complex composited backgrounds are limitations unless explicitly qualified.
Document unsupported cases, rather than silently claiming all photos were removed.

The Phase 0 source-replacement implementation also does not support CSS or scripts
whose styling/layout depends on `src`, `srcset`, or picture-source attributes:
those attributes change. Detected light-DOM rectangle drift emits
`ReplacementGeometryChanged`; over-budget geometry emits
`ReplacementGeometryNotMeasured` with `geometryPreserved:null`. Stable screenshot
pixels do not establish preserved geometry or supported source-dependent styling.

The replacement algorithm/version is part of the immutable capture profile.
Normal-mode and replacement-mode screenshots cannot be interchanged. Changing
the mode or a material transformation rule requires a compatible new baseline.
Original hidden image content is not separately saved in replacement-only runs.

## 10. Comparison and review contracts

### Capture settings versus analysis settings

**Require recapture/new baseline:** screen size/emulation, capture extent,
image replacement, material stabilization rules, browser/rendering mode, or
other conditions that change the screenshot's meaning.

**Can change after capture:** perceptual color tolerance, significant-pixel
thresholds, regional heuristics, anti-alias handling, ignored rectangles, and
review labels. Keep originals intact and create a versioned analysis revision.

### Metrics and classification

Implement transparent, versioned heuristics:

- Per-pixel perceptual color difference with optional anti-alias noise handling.
- Changed pixel count and percentage of eligible, non-ignored pixels.
- Local/tile change concentration, so a small button on a very long page is
  not hidden by a tiny overall percentage.
- Connected changed regions: counts, bounds, and largest affected area.
- Screenshot dimension/page-height changes as explicit structural evidence.

Advanced users can tune these; ordinary users choose **Sensitive**,
**Balanced**, or **Relaxed** with examples. Phase 5 calibrates and documents
numeric presets against a corpus before release. Do not describe percentage
difference as a probability or use an unexplained combined confidence score.

Flag significant change when enabled criteria are exceeded; always preserve
the metrics and visual evidence. Capture failures and non-comparable targets
are classified separately, never as unchanged. No automatic defect verdicts,
OCR, or semantic “this text is wrong” inference are required.

### Image geometry and masks

Compare aligned coordinates without automatically warping/matching away layout
changes. Different heights use an explicit comparison canvas; unmatched bands
count as structural differences, not white padding that might accidentally
match. Unexpected width/DPR changes trigger profile-integrity checks.

Ignored rectangles are in capture pixel coordinates, associated with the exact
page/profile and a mask revision. Show them on both images and report ignored
area. Revalidate on dimension changes; never silently stretch a mask to hide
new content. A rectangle is geometric, not attached to a moving DOM element.
Changing masks does not change screenshot evidence. Warn when masking removes
all or a large fraction of eligible pixels.

### Environment comparability

Distinguish the **site environment** (production/staging URL mapping) from the
**rendering environment** (computer, browser, fonts, and capture configuration).
Comparing explicitly mapped production and staging addresses is expected; that
address difference alone must not trigger incompatibility or prevent approval.

Store OS, architecture, browser build, capture mode, app/worker/profile version,
and available font/rendering metadata. Matching browser versions do not
guarantee pixel-identical output across computers or operating systems.

- Material profile mismatch: block ordinary automatic comparison/approval and
  explain how to capture a matching baseline.
- OS/rendering-environment mismatch: allow inspection in clearly marked
  caution mode, without presenting the run as a trustworthy clean release check.
- Prefer the same capture environment for ongoing checks. Do not promise
  portable ZIPs solve cross-OS rendering differences.
- A new rendering-environment baseline requires a complete compatible
  replacement; do not partially mix source captures from incompatible rendering
  environments.

### Review operations

Keep unreviewed, intentional, and needs-fix labels separate from metric-based
classification. Changing sensitivity does not erase review history, but make
the applicable analysis revision clear. Accepting selected results shows an
impact summary and creates a new immutable baseline transactionally. Original
run results remain attached to the original baseline and analysis.

## 11. Sharing, retention, and security

### ZIP interchange

Versioned manifest containing project settings, inventory, selected baseline's
complete reference map, required capture metadata, and referenced screenshots.
A composite baseline export includes all of its selected/inherited assets,
even when those originated in different runs. No raw application database dump.

Exclude session state, credential-store keys, browser profiles, private logs,
machine-local paths, and runtime binaries. Export a portable snapshot with
provenance, not all project history or comparison runs. Warn about confidential
page content, site addresses, and sensitive URL/query data.

Validate schema versions, counts, size budgets, normalized paths, duplicate
entries, unsupported symlinks, and artifact hashes before import. Defend against
path traversal, ZIP bombs, malformed images, and unexpected executable content.
Import transactionally as a new local project; never run imported content
automatically. Do not overwrite an existing project on a name collision.

### Retention and storage pressure

Never automatically delete screenshots, runs, or baseline history. Show actual
project sizes and a deletion preview. Protect the active baseline and any
artifacts still referenced by other retained versions. Garbage-collect only
unreferenced artifacts resulting from explicit deletion, using reference checks.
Derived caches may be bounded/rebuilt, but are not the only copy of evidence.

Preflight available space, monitor during capture, and stop/pause safely on
low disk. Explain incomplete runs and cleanup choices. Do not activate a
baseline with missing/corrupt artifacts. Cancellation and crashes may leave
temporary files; repair must distinguish these from valid retained evidence.

### Release security

Keep Chromium's sandbox enabled. The managed browser visits potentially
untrusted websites; use current supported runtime releases and an explicit
baseline-migration policy for security updates. Local-only does not mean
offline: the app contacts runtime hosts, target sites, and optionally update
hosts. No product telemetry is required; diagnostics sharing is opt-in.

## 12. Phases and milestones

| Phase | Outcome | Detailed checklist |
| --- | --- | --- |
| 0 | Prove cross-platform GPUI, runtime setup, large-image viewing, and session feasibility | `docs/phases/00-feasibility.md` |
| 1 | Establish UX, visual system, domain contracts, storage, and app shell | `docs/phases/01-foundation.md` |
| 2 | Working single-page baseline → check → diff loop | `docs/phases/02-vertical-slice.md` |
| 3 | Guided multi-page discovery, projects, inventory, and resumable capture jobs | `docs/phases/03-crawling.md` |
| 4 | Authenticated/internal sites, environment mapping, and capture consistency | `docs/phases/04-auth-and-capture.md` |
| 5 | Post-capture analysis, ignore areas, image replacement, and selective approvals | `docs/phases/05-analysis-and-review.md` |
| 6 | Complete baseline ZIP interchange, history, cleanup, and storage safety | `docs/phases/06-portability-and-storage.md` |
| 7 | Clean-machine packaging, usability validation, hardening, and first release | `docs/phases/07-release.md` |

Each phase depends on its predecessor. Cross-platform smoke tests run from
Phase 0 onward; Phase 7 is release qualification, not the first attempt to run
on a second or third OS. Implement foundational schema/protocol hooks early,
but do not implement all future features prematurely.

**Earliest hands-on preview:** end of Phase 2, limited to a public page.
**Whole-site preview:** end of Phase 3, public sites and fixed inventories.
**Feature-complete candidate:** end of Phase 6, all confirmed capabilities.
**First public release:** after Phase 7 gates on Windows, macOS, and Linux.

## 13. Test and acceptance strategy

### Deterministic fixture sites

Build local fixtures for ordinary links, SPA/hash routes, tracking and meaningful
queries, redirect aliases, robots rules, pagination loops, lazy-loaded images,
sticky headers, long/infinite pages, nested scrolling, delayed fonts, animation,
unstable timestamps, HTTP errors, rate limiting, offline interruptions, and
login expiry. Include cookie/localStorage/IndexedDB/sessionStorage login cases
and two mapped environments. Use test/read-only accounts, never real secrets.

### Image corpus

Include unchanged images, anti-alias noise, changed photos, tiny icon changes on
long pages, text/layout shifts, added/removed page bands, masked regions, and
normal versus replacement mode. Annotate expected evidence/classification
under each versioned policy. Fixtures must be generated or have suitable licenses.

### Automated coverage

- Unit/property tests for URL identity/mapping, scheduling, masks, metrics,
  baseline inheritance, references, migrations, and archive validation.
- Worker integration tests for protocol, capture, login, timeouts, and process
  crashes against local fixtures.
- App tests for focus/actions and material view states where GPUI supports them.
- Cross-platform packaged smoke tests for setup, login browser, screenshot,
  import/export, and cleanup.
- Fault injection: corrupt download/archive/image, insufficient disk, missing
  secret store, child-process hang/crash, interrupted writes, and app restart.

### Usability gates

At least three representative non-developers should independently create a
project, establish a baseline, inspect an intentional change, ignore a dynamic
region, and export/import a baseline without using a terminal or writing code.
Observe errors and comprehension, not only whether they like the screenshots.
Resolve critical failures before release. An authenticated-site task must also
be tested with someone who did not implement the login flow.

### Scale/performance gates

Qualify 100 pages × 3 profiles and a 1,000-page/3,000-target soak test on declared
reference hardware. Record total time, peak Rust/Node/browser memory separately,
disk growth, retry/error counts, and interaction latency. Proposed UI target:
95th-percentile input response below 100 ms during background work. Establish
realistic memory/storage budgets in Phase 0; bounded queues and virtualized/tiled
viewing are mandatory even before precise budgets are calibrated.

Every run summary must account for intended targets as successful, failed,
skipped, cancelled, or pending. No green/unchanged summary may conceal missing
coverage. No phase is complete with orphan browser processes or lost evidence
after ordinary cancel/restart paths.

## 14. Decision log and implementation discipline

Record consequential decisions under `docs/decisions/`: supported-platform
matrix, GPUI pin/component approach, runtime pack/signing/distribution,
Linux packaging, image artifact limits/tiling, auth persistence, URL identity,
capture-profile versioning, calibrated diff presets, and archive schema.

Before starting a phase, read its checklist and the referenced contracts above.
At completion, record what shipped, tests run, unresolved limitations, and any
proposed scope change. Do not claim a gate passed from a mock/demo alone.

This planning deliverable contains no application implementation. Begin with
Phase 0; resist implementing the polished full UI before setup, rendering,
browser control, and screenshot comparability have been proven.

## 15. Technical sources checked during planning

Checked on 2026-10-08; pin/verify actual versions again in Phase 0.

- [GPUI](https://www.gpui.rs/) and [GPUI README](https://github.com/zed-industries/zed/blob/main/crates/gpui/README.md): active pre-1.0 development, platform backends, and dependencies.
- [Playwright installation/system requirements](https://playwright.dev/docs/intro): supported Node and OS versions.
- [Playwright browser management](https://playwright.dev/docs/browsers): version-matched browsers, private paths, Linux dependencies, proxies, and headed/headless variants.
- [Playwright authentication](https://playwright.dev/docs/auth): state persistence, sensitivity of authentication files, and session-storage limitations.
- [Playwright visual comparisons](https://playwright.dev/docs/test-snapshots): rendering-environment variability and comparison concepts.
- [MPL 2.0](https://www.mozilla.org/en-US/MPL/2.0/): obtain the official license text during scaffolding and retain third-party notices separately.
