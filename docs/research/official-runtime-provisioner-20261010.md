# Official runtime provisioner — 2026-10-10

## Trust boundary

`crates/runtime/src/official.rs` implements the approved alternate trust model:
an **app-owned embedded catalog**, not an unsigned substitute for a configured
signed pack. Authenticated app/source updates change the catalog. A developer
source build trusts that local source; it does not establish public release
qualification. `OfficialPinned` is explicitly distinct from `QualifiedPublic`.
`Distribution::validate` rejects `OfficialPinned`, including environment-provided
legacy distribution configuration. Existing signed-pack verification and
development-loopback guards remain unchanged.

The caller supplies the immutable catalog and bundled ZIP bytes compiled into
the app. Catalog URLs are exact HTTPS pins on Node/Playwright official hosts;
credentials, queries, fragments, custom ports, redirects and TLS bypasses are
not accepted. Each archive has an exact SHA-256 and byte length, checked before
any extraction or execution. Bundled library bytes have their own SHA-256.
The version-3 manifest supplies the complete resulting file hashes, lengths,
modes and contained bundle links; cache-generated inventories are never trusted.
Pinned runtime identity strings come from that same catalog. Generating the
real tuple and archive/library bindings is the catalog owner's responsibility.

## API and actual installation

- `validate(&Catalog, bundled: &[u8]) -> Result<()>`
- `resolve(cache, &Catalog, cancel) -> Result<Installed>`
- `setup_network_with_progress(cache, &Catalog, bundled, cancel, progress,
  smoke, &NetworkSettings) -> Result<Installed>`

`Catalog`, `Component` and `ArchiveFormat::{Zip, TarGz}` are public, serializable,
and reject unknown struct fields. ZIP components strip their pinned prefix and
retain relative paths under `destination`. Selected Node TAR entries likewise
retain `bin/node`; other entries, including npm links, are never materialized.
TAR parsing/decompression is cancellation-aware and bounded. ZIP raw-directory
validation rejects duplicate/colliding paths, mismatched local headers and
unlisted payloads before extraction. File creation is exclusive, streaming
in 64 KiB blocks, size/hash checked, permission-normalized to catalog modes and
synced. Link payloads must equal declared targets; links are created last.
File/entry/unpacked/download budgets are bounded, with a combined archive plus
unpacked size and 128 MiB free-space preflight before downloading.

Downloads and staging stay under the private cache, with automatic temporary
cleanup on failure/cancellation. Progress reports real aggregate downloaded
bytes, verification, extraction, then browser testing. The complete catalog
inventory is verified before invoking the caller's sandboxed screenshot smoke.
Only successful smoke and a final cancellation check allow a synced `READY`
marker, UUID generation rename and atomic `current` replacement. Failures before
publication do not change the old pointer/generation. A failed pointer commit can
leave an unreferenced verified generation, but never selects it. `Ready` occurs
only after successful publication; cancellation after commit is not falsely
reported as an uncommitted failure.

Resolution checks physical ancestors, regular safe `runtime-…` pointers,
matching readiness markers, every catalog file hash/size/type/mode, declared
links and unlisted directories/files. No local manifest or signature is needed
or accepted on this path. Unix regular-file reads additionally use `O_NOFOLLOW`.
The private cache is the existing same-user filesystem boundary, not protection
against a malicious process already controlling the user's account.

## Handoff limitations and verification status

No builds, tests, official artifact execution or platform qualification were
run by this implementation owner, per coordination instructions. Inert unit
tests are authored for catalog refusals and ZIP inventory/exclusive extraction.
The parent owns actual official archive pins, embedded catalog/library assets,
default app integration, real Node/Chromium smoke and independent qualification.
The smoke callback must use the existing sandbox and keep its screenshot in
memory; the provisioner does not bypass Chromium sandboxing or OS policy.
Mac bundle links are representable but do not establish Mac process support.

Dependencies: `tar = 0.4.44` with default features disabled (no xattr extraction),
and existing-lock `flate2 = 1.1.10`. Only the new TAR package and runtime edges
were added to `Cargo.lock`; TAR checksum verified against the official registry
index. Existing `filetime` and `libc` satisfy its nonoptional dependencies.
The parent may need `cargo fetch` for TAR. No broad dependency resolution ran.

The existing `NetworkSettings::client()` retains its 120-second request timeout;
slow large downloads can fail safely and require a retry. A longer dedicated
download timeout is a small follow-up for the network owner, not a TLS or
redirect-policy relaxation. This provisioner never selects a different catalog,
falls back to unsigned packs, invokes npm, or downloads an unpinned executable.

## Independent-review fixes — 2026-10-10 source-only handoff

All three P2 source findings in
`docs/research/official-runtime-source-verification-20261010.md` have corresponding
implementation changes and authored inert regression fixtures. **No build, test,
download, launch, asset regeneration or archive qualification was run for this
follow-up.** The parent must run serial verification against the frozen sources;
an already-running check may still contain the earlier implementation.

- **P2-01, physical inventory aliases:** inventory names now derive from validated
  normal filesystem components, joining them with `/` only after validation.
  Literal Unix backslashes are rejected, not rewritten. An inert resolver fixture
  supplies a complete valid generation plus an extra `node\bin\node` leaf, then
  a `node\bin` directory. It demonstrates the former normalization collision and
  asserts refusal without changing the pointer or extra bytes.
- **P2-02, TAR parser metadata:** selected-Node extraction no longer uses any
  interpreted or raw `tar::Archive` iterator. A streaming physical 512-byte-header
  walker uses only fixed-header checksum, size and bounded name accessors. Each
  nonterminal physical header, including GNU/PAX metadata, consumes the shared
  20,480-entry budget before payload processing. Sparse types and global PAX are
  rejected before payload reads; no sparse chain is interpreted. Unneeded npm
  GNU longname/longlink and local PAX bodies may be skipped in fixed 64 KiB buffers,
  at most 16 KiB per body and 1 MiB aggregate metadata. Their contents are never
  interpreted or allocated as variable-sized metadata. A following entry with
  extension-altered semantics cannot be selected as Node; selected Node must be
  a unique ordinary short-path header. Ustar/GNU/old fixed headers are supported,
  with explicit regular/link/directory/local-extension type policy. Physical
  sizes plus padding must fit the remaining 2 GiB decoded-byte budget. Two zero
  terminators and zero-only tail padding are checked while draining the decoder
  through its gzip checksum. Inert fixtures assert oversized GNU/PAX and sparse
  headers refuse after exactly 512 input bytes, count extensions physically,
  permit streamed unselected npm metadata, and refuse extension-qualified Node.
  Compatibility with the captured real platform archives still needs the parent's
  serial extraction checks; no archive was reread/regenerated by this owner.
- **P2-03, ZIP effective-name disagreement:** the core raw-directory guard now
  validates both local and central extra-field framing and permits only inert
  NTFS/Unix/timestamp field IDs (`000a`, `000d`, `5455`, `5855`, `7855`, `7875`),
  each at most once per header. Unicode Path, Unicode Comment, ZIP64, AES and
  unknown fields refuse before `ZipArchive` can reinterpret names or metadata.
  Non-ASCII raw names must be valid UTF-8 with the UTF-8 flag; ASCII names remain
  supported with either encoding flag, avoiding CP437/UTF-8 disagreement. Existing
  local/central equality, CRC/descriptor and contiguous range checks remain.
  Hand-encoded inert ZIP fixtures include two distinct raw names with correctly
  CRC-bound Unicode Path extras naming the same effective leaf, local-only Unicode
  extras, unflagged non-ASCII names, malformed/duplicate extras, and positive
  supported names/metadata. Existing duplicate/name/range refusals are also covered.

The parent's `network.download_client()` call is preserved (superseding the
earlier timeout follow-up note above). Exact empty ordinary pinned Chromium
`DEPENDENCIES_VALIDATED` acceptance and its tests are preserved. No such marker
is bundled or created by the provisioner, and host validation is not skipped.
Mac bundle link rules, legacy signature functions and all files outside the
two provisioner/core sources and this appended note were untouched in this
follow-up. No new dependencies were needed.
