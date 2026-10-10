# Official runtime provisioner: independent source review — 2026-10-10

## Verdict and scope

**Source-only, conditional verdict: the approved trust separation and primary
installation/verification flow are sound on inspection, but this is not a clean
acceptance of all claimed inventory/archive invariants. Three P2 findings below
need disposition. No P1 execution-integrity or TLS bypass was identified.**

This reviewer did not implement the provisioner. Review was read-only except for
this document. No build, test, download, native execution, browser, worker, or
subagent was run. Findings are source deductions, not executed reproductions.
No existing guardian/scale qualification was repeated.

Reviewed `crates/runtime/src/official.rs`, `distribution.rs`, `bundle.rs`,
`network.rs`, `crates/core/src/zip_directory.rs`, core platform helpers, runtime
dependencies, and the provisioner API note. Locally present sources for pinned
`tar 0.4.44`, `zip 2.4.2`, and `flate2 1.1.10` were read to check parser behavior.
A narrow read of the current `crates/app/src/product.rs` first-use/callback flow
is provisional: app integration is independently being finished. Embedded
catalogs and bundled assets were **not reviewed or qualified**.

The app-embedded catalog's exact official URLs, archive hashes, and complete
resulting inventory are the explicitly approved trust architecture. Absence of
a downloaded signed manifest is not a defect. This review does not replace that
architecture with self-authored remote manifests or dummy signatures.

## Findings requiring disposition

### P2-01 — Unix inventory normalization accepts extra, unverified leaves

Location: `crates/runtime/src/official.rs:155-165`.

`inventory()` replaces every backslash in an actual relative filesystem path
with `/` before looking up the catalog. On Unix a backslash is a literal filename
character, not a path separator. `safe_name()` prohibits it in catalog paths,
but inventory does not apply that check to the actual path before replacement.

Concrete source-derived case: a valid installed tree contains its catalog file
`node/bin/node`. Add a separate root-level regular file whose literal name is
`node\bin\node`, with arbitrary contents and mode. Inventory converts that
extra filename to `node/bin/node` and accepts it as catalog-listed. The later
hash/size/mode loop reads only the real `node/bin/node`; the extra file is never
hashed or mode-checked. Likewise a directory with backslashes can masquerade as
a listed directory prefix. `READY` and all real catalog files can remain valid,
so `resolve()` has no subsequent check that catches the extra leaf.

Impact: the promised complete inventory/unlisted-file refusal is false on Unix.
This is not proof of executing the added file or bypassing any real catalog
file's hash. It does not require an attacker-controlled catalog, and can affect
read-only verification of an otherwise valid cache.

Suggested fix: derive a slash-separated relative name from validated normal
path components, rejecting literal backslashes on Unix, rather than replacing
characters in an arbitrary path string. Add an inert Unix resolver fixture with
the valid file plus the extra literal-backslash leaf and directory. The legacy
verifier has the same pre-existing normalization pattern; this change did not
introduce a signed-pack signature bypass.

### P2-02 — TAR metadata allocation and physical entry work escape the outer entry guard

Location: `crates/runtime/src/official.rs:309-318`; pinned TAR dependency
`src/archive.rs:378-441,445-540` and `src/entry.rs:296-300`.

`archive.entries()` is the interpreted, non-raw iterator. Before returning an
entry to the provisioner's `count_entry()`, TAR consumes GNU longname/longlink
and local PAX headers into `Vec<u8>` with `read_to_end()`, and parses GNU sparse
extension headers into an in-memory block list. This happens **before** the
provisioner decides that an entry is unselected and should be ignored.

A checksum-pinned TAR containing a large unselected GNU longname or local PAX
payload can therefore cause allocation approaching the 2 GiB decompression
ceiling inside a single iterator advance. The initial 128 KiB allocation cap in
TAR's `read_all()` is not a final cap. A GNU sparse extension chain can similarly
perform many physical-header reads and allocate many block descriptors while
the application's entry count advances only once. Hidden extension headers are
not included in its advertised 20,480-entry counter.

The outer `Bounded` reader **does** cancel each read and impose a finite 2 GiB
decompressed-byte ceiling. The defect is not infinite decompression or link
extraction: it is that the nominal entry guard and 64 KiB streaming description
do not bound parser metadata memory/work appropriately for a desktop installer.
Allocation failure can terminate the process before a clean installer error or
cleanup. An authentic, checksum-matching archive is required; hash verification
prevents an ordinary network peer from substituting such a payload. No claim is
made that the unfinished real catalog contains it.

Suggested fix: use a raw, streaming header iterator for this selected-Node-only
use case, with an explicit supported-header policy, or implement small separate
metadata budgets and count physical extension/sparse headers before interpreting
them. Preserve the selected-path semantics deliberately; do not silently change
which Node entry a legitimate longname/PAX header denotes. Add bounded inert
fixtures for oversized longname/PAX metadata and sparse extension chains.

### P2-03 — Raw ZIP duplicate guard and ZIP decoder can disagree on effective names

Location: `crates/core/src/zip_directory.rs:45-54` and
`crates/runtime/src/official.rs:258-270`; pinned ZIP dependency
`src/read.rs:70-75,1310-1353,1494-1501`.

The raw guard validates/deduplicates central-directory raw UTF-8 filename bytes,
then skips extra fields without interpreting them. ZIP 2.4.2 subsequently honors
Info-ZIP Unicode Path extra field `0x7075`, replacing the filename, and builds an
`IndexMap` keyed by that effective name. Inserting the same effective name again
silently replaces the earlier entry.

Concrete source-derived case: two different safe ASCII raw names have matching
local headers and valid Unicode Path extras (with the correct raw-name CRC), both
denoting the same catalog destination. The raw validator considers the names
distinct. `ZipArchive::new()` can drop one before the provisioner's loop, so
exclusive creation and the link duplicate set never see the duplicate. If the
surviving entry matches the catalog and all other required leaves are supplied,
final inventory verification does not reject the archive's effective-name
collision. Non-UTF-8-flag CP437 interpretation is another name-semantics mismatch
that should be considered when choosing a supported-name policy.

Impact: the explicit raw pre-deduplication duplicate/collision refusal is not
complete. The surviving file still must match the catalog's length/hash/mode;
this is **not** an unsigned-executable path or a weakening of legacy Ed25519.
The underlying helper behavior predates this provisioner, which inherits it.
Again, an archive matching an app-owned pin is needed, not just a malicious
download response.

Suggested fix: reject unsupported filename-changing extra fields, or validate
the same effective name semantics and raw-to-decoded entry count before any
deduplication. Add an inert two-entry Unicode Path collision fixture. Do not
remove the existing local-header/range validation when addressing this.

## Source assessment by requirement

### Catalog and trust boundary

- Official manifests require version 3, `OfficialPinned`, the current OS/arch,
  a safe ID of at most 128 bytes, safe entrypoint paths, future support expiry,
  and nonempty bounded pinned identity strings without controls.
- File hashes are lowercase SHA-256 strings, byte totals are checked with
  overflow detection, Node/worker must be present, Node executable, and the
  browser tree must have a file. Bundle validation adds safe/case-collision-free
  leaves and directory prefixes, no leaf-as-directory, complete mode maps, and
  contained/non-dangling/non-cyclic browser bundle links. Windows links refuse.
- At most eight components; positive pinned archive lengths and the sum of
  lengths are at most 2 GiB. Bundled bytes have their own hash and size check.
  Destinations cannot overlap case-insensitively. Selective extraction must
  produce the catalog's Node path; unrestricted TAR extraction refuses.
- HTTPS hosts are explicitly allowlisted; userinfo, query, fragment and
  non-default ports refuse. URL parsing canonicalizes an explicit `:443` to the
  default port, so the note's blanket wording “custom ports” should mean
  non-default ports, not a textual ban on `:443`.
- Validation checks structural bindings, not real upstream provenance or the
  correspondence between identity labels and executable versions. Those remain
  the embedded catalog owner's responsibility. Serializable public APIs do not
  themselves enforce that a caller used compile-time-owned data.

### Archive, type, mode and link integrity

- Every official download is exact-length checked and SHA-256 checked before
  parsing or execution. The selected/extracted file's declared size and actual
  streamed size/hash must match the catalog. File creation is exclusive.
- The ZIP raw helper checks classic directory bounds/count, encryption flags,
  local/central names, flags, method and sizes/CRC or descriptors, and contiguous
  non-overlapping local ranges with no prepended/unlisted physical payload.
  It is intentionally restrictive about ZIP64, multi-disk and non-UTF-8 raw
  names. See P2-03 for its filename-interpretation boundary.
- Selected ZIP leaves reject unsupported file types; listed directories must
  be empty and directory-typed or unspecified. Node TAR selection requires a
  regular file and never calls TAR unpack. Unselected Node metadata is not
  materialized, though the library interprets some of it (P2-02).
- Files are normalized to catalog modes, not copied from archive permission
  bits. Bundle validation excludes set-ID, sticky, group/other-write and other
  unsupported mode bits, and checks executable-flag consistency. Exact Unix
  file modes are verified afterward; Windows uses file-type/reparse checks and
  private-directory ACLs, not Unix mode verification.
- Every ZIP link payload must exactly equal its bounded catalog target. Link
  duplicates are checked when visible to the extractor. All expected links
  must be seen, and links are created last, so they cannot be extraction parents.
  Bundle link resolution stays within the enclosing `.app`/`.framework`, has
  a 32-link chain bound, and resolves to a catalog file or directory.

### Read-only verification and no automatic repair

- `official::resolve()` only reads. It checks physical ancestors, a small
  regular `current` pointer naming a safe `runtime-…` generation, a matching
  `READY`, inventory, all declared file bytes/hashes and Unix modes, and exact
  declared link types/targets. Unix regular reads use `O_NOFOLLOW`.
- Local manifests/signatures are not inputs to official verification. Extra
  ordinary names generally refuse; P2-01 is an exception to that guarantee.
- These checks are not a directory-handle-based race-proof snapshot. Concurrent
  same-user mutation, hardlink mutation, and verify-to-execute races remain
  outside the expressly stated private same-user cache boundary.
- The setup API is an explicit installation/replacement operation; the library
  resolver does not initiate it on damage. The provisional app flow uses
  `current_absent()` before first-use setup and does not convert present broken,
  linked or unreadable pointers into automatic replacement. Final app behavior,
  serialization of concurrent operations, and no-repair on check/resume still
  need verification after the app owner freezes those sources.

### Cancellation, publication and cleanup

- HTTP send/chunk awaits poll cancellation every 50 ms. Hashing, extraction,
  TAR read/skip/tail consumption and inventory loops check cancellation.
  ZIP raw validation and `ZipArchive::new()` have no cancellation checkpoints
  while synchronous metadata parsing runs. Smoke cancellation and bounded worker
  shutdown are caller responsibilities, not enforced by the callback type.
- Temporary downloads/staging are RAII-owned. All extracted files are synced;
  the stage is verified before smoke. Smoke success and another cancellation
  check precede `READY`. The pointer temp file and marker are synced; pointer
  safety and cancellation are rechecked immediately before generation rename.
- Before the rename/commit window, failures leave the existing pointer and
  generation untouched. A failed pointer persist may leave an orphan generation.
  `Ready` follows successful pointer replacement and cache-directory sync. The
  official API intentionally does not return cancellation after that commit.
- A cache-directory sync failure after replacement can still return an error
  although `current` has changed; this is a durability/commit-reporting boundary,
  not transactional rollback. App-level post-setup cancellation/status checks
  must also avoid misrepresenting a committed installation; the app is not frozen.
- Nested extraction directories are not individually fsynced. Ordinary logical
  atomic publication is supported by the source; power-loss durability of every
  child directory entry is not established by syncing only files/stage/cache.
- Disk preflight conservatively includes all archive lengths plus all catalog
  file bytes and 128 MiB. It is not a quota reservation, and does not constrain
  parser RSS (P2-02). No successful post-smoke re-verification is performed;
  smoke must not mutate the staged inventory.

### Network and legacy signature preservation

- `NetworkSettings::client()` retains certificate verification, disables
  redirects, has 20-second connection and 120-second request timeouts, and
  optionally adds a managed PEM CA/proxy. No TLS-ignore interface is exposed.
  Official requests additionally require successful status and exact final
  byte count/hash; a redirect, truncated response or checksum failure refuses.
- A custom CA changes the download trust roots deliberately; it does not remove
  archive pins. Default reqwest/system proxy behavior is not explicitly disabled.
  Large/slow downloads may safely time out and require explicit retry; exact
  upstream endpoint behavior has not been exercised.
- The examined diff for `distribution.rs` only adds `OfficialPinned` and rejects
  it at the signed distribution boundary. Existing scopes keep their serialized
  names. Legacy `signed()` still uses `verify_strict` on the exact bounded manifest
  bytes before parsing/validation; the new path does not call it or relax it.
  No legacy public HTTPS/development literal-loopback rule was weakened.

## Unverified boundaries / permitted follow-up after source freeze

No claim is made about real archive URLs/hashes/lengths, completed catalogs,
bundled Playwright/worker contents, selected-path/archive layout compatibility,
artifact mode/link fidelity, sandboxed Node/Chromium execution, screenshot
smoke, supported-platform process behavior, Windows publication/ACL behavior,
app/UI handling, crash durability, or successfully run regression tests.

Useful later bounded checks are inert fixtures for the three findings, parser
refusals/type/mode/link integrity, cancellation and failed/successful publication,
read-only damage/no-auto-repair, legacy signature preservation, and an independently
bounded real pinned-runtime smoke only after explicit resumption authorization.
Nothing in this document authorizes starting those checks now.

## Reviewed source fingerprints

SHA-256 captured at handoff (not artifact qualification):

| File | SHA-256 |
| --- | --- |
| `crates/runtime/src/official.rs` | `899349a0222d71faf9d69e01344e6292b5fd66092666872bea28b7ead07d7d14` |
| `crates/runtime/src/distribution.rs` | `5974722baca4a34228d01b8f7d4d294da730532ae9e314bf0dbc40278d40448d` |
| `crates/runtime/src/bundle.rs` | `ab9c0a09f0903bbd6538f61310d9d98451b6434ddd436675c887f93694ae5cf3` |
| `crates/runtime/src/network.rs` | `4aee1ecc634b503510fa0ce41ce24aef703951c2246e72a1bf55e13d4c8c28b2` |
| `crates/core/src/zip_directory.rs` | `e4a9ca16613f29a920f59f6803fcb70015ec60f39ec44c42bca42425a0e4b906` |
| `crates/core/src/platform.rs` | `94719fa3e846c04e2104c8c6db786febe247593b4caa493591dbdbf26ce8827f` |
| `crates/runtime/Cargo.toml` | `c58957780cd9f3db970de46c984148c95d38169b383eb16afa696241982f6871` |
| `docs/research/official-runtime-provisioner-20261010.md` | `c0af96efb66c10637f8c98e6743ebb086b84b60ae1d189e4608513771f850b0a` |

## Resumed verification — repaired implementation and completed assets

This section appends the resumed review; it does **not** erase the original
findings or retroactively turn source deductions into executed test results.
The parent declared runtime/app implementation frozen and authorized local pure
Python asset ZIP/hash/catalog checks, but not builds, Rust tests, network access,
archive code execution, native windows/compositors, or worker/browser launches.
This reviewer stayed within that boundary and changed only this report.

**Updated verdict:** P2-01, P2-02 and P2-03 are closed **at source review level**.
The completed source/assets enable real default end-user runtime provisioning,
not just different messaging. Local pure-data verification succeeded for all
four catalogs, the complete embedded worker ZIP, retained archive pins, all four
browser ZIP inventories and the selected Windows Node ZIP file. No new P1 or
execution-integrity/TLS defect was identified. One app integration P2 remains
for disposition (P2-04 below). Rust inert tests, the independent complete live
first-use flow and pixel inspection are still **NOT RUN** by this reviewer.

### Disposition of the original three findings

| Finding | Current source repair | Source verdict / authored coverage |
| --- | --- | --- |
| P2-01 | `official.rs:227-247` builds inventory names from normal, UTF-8 physical components, validates each before joining, then validates the whole name. Literal Unix backslashes are rejected instead of normalized. | Closed on inspection. `resolver_rejects_literal_backslash_file_and_directory_aliases` uses both the original extra-leaf case and directory case, and asserts the existing pointer is unchanged. Test not executed here. |
| P2-02 | `official.rs:527-674` walks physical 512-byte TAR headers directly; checksum/header format/type are checked before payload. Every physical metadata header counts. GNU sparse/global PAX are rejected. Local extension payloads are discarded with fixed buffers, capped at 16 KiB each and 1 MiB total. The selected Node entry must be unique, plain regular, and not extension-qualified. | Closed on inspection. No interpreted `tar::Archive` iterator or metadata `read_to_end` remains. Authored tests assert refusal after only 512 bytes for oversized L/K/x and sparse/global metadata, safe skipping of unselected extensions, refusal of extended Node selection, and entry-counter refusal before payload. Tests not executed here. |
| P2-03 | `zip_directory.rs:12-31,73-85,121-123` checks strict bounded extra-field framing/allowlist in both headers, rejects Unicode Path/ZIP64/AES/unknown/duplicate fields, and requires the UTF-8 flag for non-ASCII names. ASCII without that flag has identical CP437/UTF-8 name semantics. | Closed on inspection. Original physical-range/name checks remain. Authored tests include the valid-CRC effective-name collision, local-only name-changing extra, encoding ambiguity, malformed/duplicate extras and original collision/range cases. Tests not executed here. |

The fixed TAR walker deliberately supports less than a general-purpose TAR
extractor. It requires two zero terminators plus zero-only decompressed tail;
unselected local extensions cannot alter the selected Node's interpretation.
The existing 2 GiB decompressed-byte ceiling and cancellation-aware reader still
apply. Selected bytes and layout compatibility for retained Node TARs have not
been independently parsed/executed in this resumed pass. The single-member gzip
decoder and synchronous ZIP metadata cancellation boundaries remain as described
in the initial review; no arbitrary unpinned archive can reach them.

### New P2-04 — explicit app setup can report cancellation after successful publication

Locations: `crates/app/src/product.rs:344-346` and
`crates/app/src/jobs.rs:523-528`; successful official commit at
`crates/runtime/src/official.rs:842-850`.

The official provisioner correctly publishes `current`, syncs the cache, emits
`InstallProgress::Ready`, and returns success without another cancellation check.
However `Product::setup_with_progress()` immediately performs
`check_cancel(&cancel)` and then a cancellation-aware `status(&cancel)`. The
explicit `Work::SetupRuntime` handler checks cancellation yet again and calls
cancellation-aware readiness. Consequently cancellation after commit (including
one requested while the post-commit inventory is being rehashed) converts the
successful installation into `Reply::ProductFailure("operation cancelled")`.
The published pointer and runtime remain selected. The current generic setup
error wording/terminal failure does not communicate that committed outcome.

A direct source-derived reproduction is a progress callback that sets the
operation token when it receives `Ready`: the official call returns `Ok`, but
the enclosing Product setup call deterministically returns cancellation. No
network execution of that case was performed here.

Impact is honest commit reporting/recovery, not damaged evidence, TLS bypass,
or automatic repair. The prior review called out the app post-commit boundary
while app work was provisional; it is now a concrete frozen integration finding.

Suggested disposition: distinguish a committed setup outcome and use an
uncancelled read-only readiness check for explicit setup's terminal success, or
return an explicit “installed successfully; following action cancelled” outcome.
Do **not** globally ignore cancellation: a first-use discovery/capture must still
stop before continuation when its shared token is cancelled after installation.
An inert/mock commit-aware app test should cover both explicit-setup success and
cancelled first-use continuation without another download or worker launch.

### Network timeout and host-validation-marker updates

- `NetworkSettings::download_client()` uses a 20-minute total request timeout;
  the ordinary client retains 120 seconds. Both share 20-second connection and
  30-second read timeouts, no redirects, normal TLS verification and the same
  validated optional proxy/added PEM trust root. The official download path
  actually calls the long-timeout client. Legacy signed downloads still use the
  ordinary client. These are source findings, not measured timeout tests.
- `dependency_marker()` derives the permitted marker's exact parent from the
  pinned Chromium component destination. With the verified catalogs it is only
  `browsers/chromium-1248/DEPENDENCIES_VALIDATED`. Inventory requires an ordinary
  zero-length file and, on Unix, no execute/set-ID/sticky bits. Symlinks, reparse
  points, nonempty markers, other revisions and other locations refuse.
- Catalog validation rejects declaring this marker as a file or link. Neither
  extraction nor provisioning manufactures it, changes its timestamp, or sets
  a host-validation skip flag. The complete bundle also contains no marker.
  The supervisor starts from `env_clear()`, sets the private browser root and
  does not forward a skip-validation environment variable. The worker still
  launches `channel: 'chromium'` with `chromiumSandbox: true` and checks all four
  pinned identity labels before its in-memory screenshot smoke.
- The marker exception is intentionally mutable upstream housekeeping, not an
  additional catalog-authenticated artifact. The authored marker test covers
  accepted empty ordinary marker and refused content, executable mode and link;
  this reviewer has not executed that test or triggered actual host validation.

### Executed pure-data asset checks

Two bounded local Python invocations used only standard-library file reads,
JSON, ZIP parsing/streaming decompression and hashes. No generator invocation,
network fetch, extraction to disk, subprocess crypto command, or archive code
execution occurred. Retained archives were read only from the existing private
0700 directory
`/path/to/private-tests/opencode/regressee-official-assets-5gsnd0p3`.

Results:

- Embedded ZIP is exactly **4,177,469 bytes**, **184 regular entries**,
  **18,751,596 uncompressed bytes**, SHA-256
  `0646531342f9a4c0c00253df68e4702dd9bc366fd487e8a299f2a492b3dd9fe9`.
  Sorted unique safe paths, ordinary 0644/0755 modes, deterministic timestamp,
  UTF-8 flag requirements, types, lengths, streamed hashes and strict raw
  local/central bounds/ranges were checked. No ZIP extra fields or fabricated
  validation marker are present.
- All **nine** local worker source/package/lock/license files match the bundled
  bytes exactly. Vendor package manifests are pinned to 1.64.0; the bundle has
  60 Playwright and 115 playwright-core published files, matching recorded npm
  metadata. Embedded `browsers.json` matches the recorded Chromium 156.0.8078.4,
  revision 1248. Package execution was not attempted.
- Each catalog's entire bundled worker subtree matches ZIP file hashes, sizes,
  executable flags and modes. All other leaves are the selected Node file or
  the pinned browser destination, with no unexpected subtree. Complete mode
  maps, budget/expiry/scope/identity/entrypoints, portable safe names,
  case-colliding directories/leaves, leaf-as-directory, component mappings,
  official URLs and contained/non-dangling/cycle-bounded Mac links were checked.
- Each catalog hash matches `provenance.json`; archive component metadata agrees
  exactly. Every retained Node/browser archive matches its catalog's complete
  raw SHA-256 and byte count. All four Node pins also match the retained plain
  checksum records. Plain and clear-signed checksum-file hashes match provenance.
- All **four browser ZIPs** independently match every final file hash/size,
  normalized mode, link payload/type and explicit directory's catalog descendant.
  The Windows Node ZIP's selected `node.exe` independently matches its catalog
  hash/size/type/mode. Raw ZIP local/central header/range and allowed-extra checks
  succeed on all five official ZIPs, not just the embedded bundle. This is a pure
  Python correspondence check, not evidence that the Rust tests executed.
- Both retained npm tarball SHA-256 and pinned SHA-512 SRI match recorded metadata
  and the generator's embedded SRI constants on source inspection. GPG and npm
  registry ECDSA verification are recorded as successful by the independent
  asset owner; this reviewer did not independently rerun those cryptographic
  signature operations. No separate publisher/Sigstore claim is inferred.

| Catalog | JSON bytes | Files / links | Download / installed bytes | Browser ZIP physical entries |
| --- | ---: | ---: | ---: | ---: |
| Linux x86_64 | 144,906 | 486 / 0 | 258,433,266 / 566,268,773 | 309 (8 directories, 301 leaves) |
| Windows x86_64 | 144,717 | 490 / 0 | 244,691,932 / 569,957,372 | 305 (0 directories, 305 leaves) |
| macOS arm64 | 243,105 | 529 / 5 | 247,741,846 / 523,665,903 | 673 (324 directories, 349 leaves) |
| macOS x86_64 | 241,702 | 529 / 5 | 259,536,079 / 537,892,183 | 673 (324 directories, 349 leaves) |

Windows Node has 2,459 physical ZIP entries; exactly one maps to the installed
inventory. Browser ZIP extras use only `0x5455` and `0x7875`; Windows Node uses
`0x000a`; Windows browser and embedded bundle have no extras. All are accepted
by the repaired source allowlist.

Generator source confirms signature/checksum/SRI verification before archive
inventory, no lifecycle/install scripts or downloaded binary execution, exact
official CfT URL cross-checking, complete npm source retention and platform
catalog generation. This review did not regenerate assets or independently
fetch official metadata. Chrome generation-time trust remains official verified
HTTPS transformed into app-owned pins, as approved, not a publisher signature.

### Frozen app trust and first-use policy

- `Product::configured()` defaults to `official_runtime: true` only when
  `REGRESSEE_DISTRIBUTION` is absent. `official_catalog.rs` uses `include_bytes!`
  and `include_str!`, a validated immutable `OnceLock`, literal tuple comparison,
  and a fresh expiry check on every access. No remote/cache catalog or environment
  runtime pin is consulted. Missing/malformed/unsafe/invalid explicit legacy
  overrides return errors and cannot fall back to official provisioning.
- `Jobs::start()` remains the production default. The source scan found only
  that production call in `main.rs` and the explicitly ignored real-first-use
  integration test using plain `Jobs::start()`. Unit/UI/capture fixtures use
  `start_without_runtime()` or explicit injections. That helper is available
  only with test/UI-test configuration; it does not disable production setup.
- First-use setup runs in the same serialized owner/operation cancellation slot
  as eligible discovery, fresh baseline capture, login and representative probe.
  It is not on the UI thread and does not require a separate Regressee server,
  system Node, npm install or an end-user distribution/key configuration.
- Comparison and resume commands are excluded from first-use provisioning.
  Login/probe for a project with an existing baseline are also excluded.
  Guided capture verifies baseline/resume profile identities against the resolved
  installed tuple before continuing; resume additionally checks rendering/fonts.
  Existing invalid/damaged/linked/missing-generation pointers refuse rather than
  trigger a replacement. Readiness/resolution remain read-only. Explicit setup
  is a separate repair action; P2-04 concerns its terminal outcome only.
- Eligibility is still Linux/Windows x86_64. Both Mac catalogs are preparation;
  readiness, setup capability and first use refuse Mac before download. Their
  presence does not activate the unsupported Mac owned-tree adapter.
- Both trust paths use the same supervisor/screenshot-handshake/stop callback.
  Product now checks the reported worker/browser identity before accepting smoke.
  No sandbox disable, launch fallback or special unowned worker was introduced.

### UI source assessment, not a rendered acceptance

Operate-mode review preserves the incumbent shared controls/theme/focus model.
No UI source was edited and no launcher, detector, renderer, native compositor
or user window was run by this reviewer.

- `main.rs:2483` gives the setup form full constrained width and non-shrinking
  column geometry. Shared `controls::section()` and `field()` are full-width and
  non-shrinking, preventing long sections/fields from consuming less than their
  intrinsic height in the scrolling form. `environment_actions()` also no longer
  shrinks. These are appropriate structural repairs to the overlap mechanism.
- The timezone root is a fixed-height non-shrinking flex column. Its trigger is
  relative/full-width, reserves 40px at the right, and anchors the 16px chevron
  absolutely 12px from top/right. The overlay canvas remains absolute and does
  not become another in-flow form child. Long values retain a min-width-zero,
  ellipsized flex area rather than pushing the chevron into the label.
- The login-help element has full width, `min_h(20px)`, fixed 20px line height
  and no shrink, so it can grow for wrapping without collapsing into the next
  action. The new source-only layout test covers 1120×800 and 820×600 geometry;
  the standalone trigger test checks the chevron's right-edge position. Neither
  was executed here, and they are not pixel evidence.
- The changes do not replace the helper's keyboard behavior: timezone Tab remains
  shell-owned, the current selector focus handle tracks open search versus closed
  trigger, and blur/frame reconciliation closes a stranded popup. Shared selected
  `Choice` colors remain unchanged, as do the Light/Dark website-theme controls'
  actual selected-state binding and persisted `profile.dark` assignment.
- No new UI source defect was identified in this bounded overlap/selector/help
  scope. Actual containment, wrapping, arrow placement, selected paint and native
  input/rendering remain unverified until the parent supplies current images.

### NOT RUN / next handoff boundary

The parent's earlier actual run is **partial evidence only**: the parent reports
official HTTPS installation, owned smoke and discovery completed, with two-page
capture entered before a 120-second shell deadline interrupted the final reply.
The provided log location is
`/path/to/private-tests/opencode/regressee-official-verification-r7ki95gq/official-first-use.log`.
This reviewer did not rerun it, inspect processes, or convert it into a complete
PASS. Parent-reported absence of remaining owned processes is not an independently
repeated containment/scale qualification.

Still NOT RUN here: Cargo/builds, any Rust inert regression, official installer
execution, independent complete first-use/reopen/no-repeat-download/two-page
capture test, cancellation/commit reproduction, legacy signature regression
execution, native input/window/compositor, pixel review, Windows/Mac execution,
guardian/scale qualification, user stores, managed helpers and system settings.

The ignored real test requires an explicitly supplied private directory and
rejects an override; it uses an owned literal-loopback two-page site and the real
production Jobs default. It asserts phase reporting, successful discovery and
captures, unchanged project settings, read-only reopen and no repeat download.
It remains ignored in ordinary suites. After parent serial compile/provenance
handoff and explicit authorization, one fresh finished-source run can use a
**1,500-second process budget** (discovery has its own 1,200-second limit and
capture 180 seconds). No run was started in anticipation of that handoff.

### Resumed source / asset fingerprints

| File | SHA-256 |
| --- | --- |
| `crates/runtime/src/official.rs` | `614ba5c4948e09129e744dd039f3b7ce0c47007cfdd11ef6a3b21134f098781a` |
| `crates/core/src/zip_directory.rs` | `79b756da84047103646ca706bb234572644989909f2cba0381f444593c1e5fb8` |
| `crates/runtime/src/network.rs` | `4aee1ecc634b503510fa0ce41ce24aef703951c2246e72a1bf55e13d4c8c28b2` |
| `crates/runtime/src/distribution.rs` | `5974722baca4a34228d01b8f7d4d294da730532ae9e314bf0dbc40278d40448d` |
| `crates/app/src/product.rs` | `598c086aa1338ae2931dcd00987ae11660674defc691e03fdc50152251b2b6a4` |
| `crates/app/src/official_catalog.rs` | `26aabca049c1b09c8a867cd1bd2ba0f14f488bfa255db03804122b286ffdfd44` |
| `crates/app/src/jobs.rs` | `18dc359569cdcc39c1ce48d22a69429dec1396c26720ab7d920ca4fd0c7cace1` |
| `crates/app/src/crawling.rs` | `d17d66d6eefe6299ef40d72abfa5ea73393323f68a816f652fa7e8ac6652e645` |
| `crates/app/src/main.rs` | `73a2631d5d2d71be29ecea3c34af6f2f0ddb9e536917ec976603312a870594df` |
| `crates/app/src/ui/controls.rs` | `c619267676413f78c4bb0005439245950504ee60acdd7e942302d11c570c4daa` |
| `crates/app/src/ui/timezone_select.rs` | `f735dcd9431fa9f7f6474fda13981a8740a5cc5e09939d1468521eb8f096a81a` |
| `crates/app/tests/official_runtime_first_use.rs` | `92f7fa6c4a52b668b7f03511134547beb530635f4b410c5335075542a3759edc` |
| `scripts/official-runtime.py` | `20a0b5263e71eb3f1adaf73ead46dd86026dfe7da1e2d028dc1a9575ef1e1e57` |
| `crates/runtime/assets/official/bundled-worker.zip` | `0646531342f9a4c0c00253df68e4702dd9bc366fd487e8a299f2a492b3dd9fe9` |
| `crates/runtime/assets/official/provenance.json` | `1788a1f6c3b7ad58d2d6cfe28dce8de30ba60d919d172afc76e2c2dd33e6e1cb` |
| `crates/runtime/assets/official/linux-x86_64.json` | `bb68be7ecba7a8640e4eee58166a0b1c5285023d5bec8ba3319587991ca432ee` |
| `crates/runtime/assets/official/windows-x86_64.json` | `9a544571f1a92aade03639cc4ded6eaf24a14b21ef02c74a7b320c6eb2b2ab2e` |
| `crates/runtime/assets/official/macos-aarch64.json` | `969d6961bd458fe6ae0c3da1df3da953d94d34514097a5c7ba8c06969b726191` |
| `crates/runtime/assets/official/macos-x86_64.json` | `919bca85ad394775e8e643811e8d9aca0b22f18220cf67d90f068a143e65bc72` |

## Final bounded follow-up — P2-04 repair and independent inert execution

Follow-up date: **2026-10-11**. Earlier findings, qualifications and fingerprints
above remain historical records of the source reviewed at those handoffs.

**Verdict:** P2-04 is closed on the current official/default app source path.
All four reported P2 findings now have source-level fixes. The original three
archive/inventory repairs additionally have independently executed bounded inert
coverage: **9 passed, 0 failed**. The two newly authored Product/Jobs commit tests
have been reviewed but **NOT RUN by this reviewer**, pending the parent's freshly
compiled app test binary. No new material source fault was identified in this
bounded follow-up. This is not blanket platform/release acceptance.

### P2-04 fix: committed installation versus cancelled continuation

- `Product::setup_with_progress()` now directly returns
  `committed_setup_result(installed)` after the installer. That helper propagates
  installer errors and formats a successful `Installed` via `installed_status()`;
  it has no late cancellation check or filesystem rehash.
- `Product::status()` remains a separate read-only/cancellable resolver, then uses
  the same status formatter. Removing post-commit work from setup did not make
  ordinary status bypass verification.
- `Work::SetupRuntime` now calls `explicit_setup_reply()`. `setup()?` must succeed
  before readiness is called; a failed/pre-commit/cancelled setup cannot be
  upgraded into a committed success. Only this explicit terminal-readiness path
  receives a new false `AtomicBool`. It re-verifies/reports actual committed state
  without reusing a late-cancelled operation token.
- `ensure_runtime_with()` still checks the **original** operation token immediately
  after first-use setup and during resolution, and the owner checks it again before
  website continuation. The new uncancelled token does not reach discovery,
  capture, login or probe. Shared-slot cancellation semantics therefore remain
  intact, with no automatic repair or cancellation exception for website work.

Authored coverage reviewed:

1. `product::automatic_runtime_tests::committed_setup_result_does_not_recheck_a_late_cancellation`
   supplies an inert `Installed` with a deliberately unread root and asserts
   successful formatting despite a late-cancelled token, plus error propagation.
2. `jobs::tests::explicit_setup_reports_committed_state_despite_late_cancel`
   models a committed callback setting the original token, asserts the terminal
   verification token is false, and checks that pre-commit failure never invokes
   readiness.

These are **mock/inert commit-reporting tests**, not a real installation cancelled
at its publication boundary. Neither new app test was executed here. The existing
first-use cancellation test and original-token checks remain present; that test
was not rerun in this follow-up either.

### Shared smoke identity recheck

Source inspection reconfirms the same owned worker handshake and bounded stop
callback for both trust paths. The stop call is made before unwrapping handshake
success; reported worker/browser versions must match the installed manifest before
the callback succeeds. The embedded worker handshake additionally checks actual
Node, Playwright, worker and launched-browser versions, performs its screenshot
in memory, and retains `chromiumSandbox: true`. No alternate worker/launch path,
fabricated dependency marker, skipped host validation or system setup was added.
No smoke/worker/browser execution was repeated by this reviewer.

### Independently executed inert tests

The explicitly authorized existing binaries were located in the parent's
`final-workspace-tests.log`, then copied into a new physical 0700 private directory
to bind execution to stable bytes while the parent prepares its fresh compile:

```text
/path/to/private-tests/opencode/regressee-official-inert-independent-s8n9eimm
```

Each copied binary was SHA-256 checked against its original before execution.
Only the narrow listed filter ran, once, with `--nocapture --test-threads=1` and
a **60-second subprocess timeout**. `TMPDIR` pointed to that private evidence
directory. No Cargo/build, network fixture, signature tool, downloaded executable,
worker, browser, GPU, compositor, native user window, managed helper or system
setting was used. No repository source was edited.

| Existing binary | Filter | Independent result | Executed binary SHA-256 |
| --- | --- | --- | --- |
| `target/debug/deps/regressee_core-7ac2597a75674186` | `zip_directory::tests::` | **3 passed, 0 failed**, exit 0; 43 unrelated tests filtered out | `a8d237972de20fe653ad1388484dec3df2605af3135a12508d876fe23d6afc9c` |
| `target/debug/deps/regressee_runtime-42ff8216066cf79e` | `official::tests::` | **6 passed, 0 failed**, exit 0; 23 unrelated tests filtered out | `299fbb765c942835e7993d504ac383d1eaa763a1eb3aa6f707138c24db5b2fb1` |

Executed core cases:

- `existing_name_and_physical_range_checks_remain_enforced`
- `non_ascii_names_require_utf8_flag_and_extra_fields_are_strict`
- `unicode_path_effective_collision_is_refused_before_decoder_dedup`

Executed runtime cases:

- `catalog_refuses_untrusted_urls_paths_hashes_modes_and_expiry`
- `inert_zip_is_verified_against_catalog_not_local_metadata`
- `only_empty_ordinary_pinned_host_validation_marker_is_permitted`
- `resolver_rejects_literal_backslash_file_and_directory_aliases`
- `tar_refuses_large_extensions_and_sparse_before_payload_reads`
- `tar_streams_unselected_extensions_but_never_selects_extended_node`

Retained evidence: `core-inert.log`, `runtime-inert.log`, `execution.json`, and
`binding-before.json` / `binding-after.json`. The before/after hashes of both
original binaries and the six relevant runtime/core source files matched exactly;
`bound_input_changes` is empty. The wrapper measured about 0.004s and 0.064s for
the two processes; the runtime harness reported 0.04s. Existing compile provenance
comes from the parent's log, not a build by this reviewer. The parent's only
reported runtime-source adjustment after earlier capture was the semantically
equivalent `cfg(test)` array spelling `*b"LKxSg"`; production code is unaffected.

### Separate independent live/image evidence, without duplication

Read `docs/research/official-runtime-first-use-verification-20261010.md`, which
records the separate asset verifier's **one complete independent live first-use
PASS**: 114.42 seconds, exit 0, 258,433,266 pinned transfer bytes, two discovered
pages, two completed captures, reopened Ready, no repeat installation, unchanged
project settings and repeat-use pointer, 127 bound files unchanged, and zero
observed remaining owned processes. This supersedes the parent's earlier
interrupted attempt **as a separate complete result**, not by rewriting that
attempt into a pass. I did not execute another live/network/guardian/scale check.

That same independent report records review of 36 source-bound UI PNGs plus
calibration, with no new sampled P0/P1 blocker. It explicitly does **not** claim
minimum-window login help visibility at its sampled scroll position, exact intended
680px form-width compliance, native input/window behavior or universal UI coverage.
No UI source changed after that capture per the parent's handoff. Those image
observations belong to the named independent reviewer; I did not view or recapture
them in this follow-up. The initial source-only UI verdict is not relabeled as my
own pixel inspection.

### Remaining NOT RUN and current binding

NOT RUN here: the two new app P2-04 tests, a real post-commit cancellation event,
fresh complete workspace suite/Clippy/ordinary build, another live/network/GPU
attempt, other-platform execution, native input/window/accessibility, human login,
public-site/release qualification, guardian/scale, or user-store/system changes.
The parent owns its fresh serial compile/formatter gate and may hand off the app
test binary afterward. The parent's 206-test goal is not claimed as an already
completed result here.

Current source hashes captured for this logic review (a later formatter/compile
handoff must carry its own final binding):

| File | SHA-256 |
| --- | --- |
| `crates/app/src/product.rs` | `b512518938a3a3d0ce2259e7834547e2e50d3fc945c073ebbdb18df6663c43c6` |
| `crates/app/src/jobs.rs` | `dbc962054b251541236c6b5a87c4870aecf7a82b946fb01e53810d175ed9ed05` |
| `crates/runtime/src/official.rs` | `614ba5c4948e09129e744dd039f3b7ce0c47007cfdd11ef6a3b21134f098781a` |
| `crates/core/src/zip_directory.rs` | `79b756da84047103646ca706bb234572644989909f2cba0381f444593c1e5fb8` |
| `worker/product/src/index.mjs` | `c9234994102c37321da6eb51ca4172014f6d84b10669f015f1b4919d41b8dffa` |

The report itself was SHA-256
`a238c81343012b545a2bcaf94c8c4427198a5fb452e516e80118b8f108386ac7`
before this append. Its after-append hash is captured in the final handoff output,
outside this self-referential document. This reviewer changed only this report in
the repository; private inert evidence and binary copies remain under the approved
temporary root.

## Final compiled-app handoff: independently executed P2-04 tests

**Final scoped result: all four P2 findings closed; 12 independently executed
inert tests passed, 0 failed.** This consists of the nine core/runtime tests above
plus the following **three additional exact app tests**, each run once after the
parent handed off its freshly compiled final app test binary. No pending test
execution remains in this review's authorized inert scope. This does not claim
a real browser installation cancelled at its commit boundary.

Existing compiled app binary:

```text
/path/to/Regressee/target/debug/deps/regressee_app-75b1d9a35a3dc20f
SHA-256 b8b17d0087907f9c1a9bc5ef586e057c7acc840b32b5b151a52055f9a4eeaab9
```

It was copied into a new physical 0700 approved scratch directory and hash-checked
before executing. One **foreground** shell invocation (120-second tool deadline)
ran three direct-binary commands with `--exact <full-name> --nocapture
--test-threads=1`, each with a 20-second subprocess timeout. `TMPDIR` pointed to
the evidence directory. No background/pending work was launched.

| Exact test | Independent result |
| --- | --- |
| `product::automatic_runtime_tests::committed_setup_result_does_not_recheck_a_late_cancellation` | **1 passed, 0 failed**, exit 0; 53 unrelated tests filtered out |
| `jobs::tests::explicit_setup_reports_committed_state_despite_late_cancel` | **1 passed, 0 failed**, exit 0; 53 unrelated tests filtered out |
| `product::automatic_runtime_tests::default_catalog_first_use_callback_stays_in_request_cancellation_slot` | **1 passed, 0 failed**, exit 0; 53 unrelated tests filtered out |

Evidence directory:

```text
/path/to/private-tests/opencode/regressee-commit-reporting-independent-j7x3icl9
```

`1-inert.log`, `2-inert.log`, `3-inert.log` and `execution.json` retain results;
`binding-before.json` and `binding-after.json` bind the original binary and seven
relevant final source files. **All eight before/after hashes match; no source or
binary drift occurred.** Current Product/Jobs hashes match the final logic-review
hashes above, as do the runtime/core hashes. Only this report was changed in the
repository by the reviewer. The immediate pre-append report hash was
`6fd9865ae9ae93e25a8e723dd6e1e7cc9627b1f0f38c183a318ce5ebf56c12fe`;
its final hash is captured separately in the handoff output.

These app tests establish **source plus mock/inert commit-reporting semantics**:
successful installation formatting does not re-read the root or check late
cancellation; explicit committed readiness uses its separate token; failed setup
is not relabeled committed; first-use keeps its original cancellation slot and
blocks continuation. They did not cancel an actual HTTPS installer or launch a
browser. The smoke identity source assessment and standalone trust/no-auto-repair
assessment above remain unchanged; no new material fault was found.

The parent reports its fresh **206 passed / 29 ignored** workspace gate and
all-target/all-feature Clippy `-D warnings` PASS. Those are parent-run results,
not another suite/build executed by this reviewer. The separately documented
independent live first-use PASS and bounded sampled PNG review remain attributed
to their independent owner; no live/network/GPU/native attempt was repeated here.

Remaining qualification **NOT RUN by this reviewer**: real late-cancel installation
publication, Cargo/build/formatter/Clippy/full-suite execution, legacy-signature
regression execution beyond source inspection, public-site or public-release
qualification, other-platform execution, native OS input/window/accessibility,
human login, guardian/scale/soak, user-store or system changes. The report does not
downgrade these boundaries, fabricate native acceptance, or infer a broader pass
from the settled scoped inert results.
