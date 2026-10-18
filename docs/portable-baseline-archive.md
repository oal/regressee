# Portable baseline ZIP — versions 1 and 2

This is selected-baseline interchange, **not** an application backup or a signed
claim of capture authenticity. Implementation: `crates/core/src/archive.rs` and
`crates/storage/src/portability.rs`. Local SQLite schema version 10 is independent
of the portable archive version. Historical strict-v1 imports remain supported.

## Contents and privacy

The ZIP contains exactly `manifest.json` and `artifacts/<lowercase SHA-256>` for
each distinct referenced original PNG or canonical DOM snapshot. Export resolves
the selected baseline's complete per-kind reference map, including inherited
originals from other source runs. Unapproved siblings are not included merely
because they share a source run.
Ancestors and unrelated run/analysis/review history are not included.

The manifest's required top-level fields are:

| Field | Meaning |
| --- | --- |
| `version` | `1` for losslessly representable legacy screenshot/settings shape; `2` when DOM settings/evidence or extended frozen provenance require it; unknown versions reject |
| `project` | Name, source URLs, future discovery/profiles, portable environment names/mappings/access policy |
| `inventory` | Frozen page identities, canonical URLs, included flags, coverage and URL identity version |
| `baseline` | Selected version/date, full target-to-source/hash/date map, intended coverage/partial status; parent is null |
| `artifacts` | Unique original hash, encoded byte count and `Screenshot` / `DomSnapshot` kind |
| `provenance` | Necessary source-run identity/dates, frozen profiles/discovery/environment/rendering and selected capture metadata only |
| `masks` | Target, image dimensions and ignored rectangles; geometry, not recipient pair authority |

Unknown schema fields, missing bindings and unselected payloads are refused.
V1 exports omit newer internal fields so historical strict readers can open them;
v1 inputs reject those fields even when null, empty or default-valued. V2 carries
independent DOM references and necessary frozen configuration. A DOM-enabled
future-capture draft can require v2 when sharing an older screenshot-only baseline;
import preserves the draft and baseline's different modes rather than rewriting one.
Current project settings can differ from the historical capture. Both remain
separate; a fixed-baseline check/probe uses the historical source configuration,
not newly edited future profiles or discovery settings.

**Ignored regions DO NOT redact raw screenshots.** Originals can reveal names,
account data and other confidential information. Addresses and private URL queries
remain visible. The export preview requires explicit acknowledgement. Review
content before sharing; a ZIP is neither encrypted nor a privacy sanitizer.

**DOM text can expose private hidden content, attributes and URLs.** Script bodies,
handlers and form values are excluded before capture, but this is not complete
redaction. Image ignored rectangles do not redact snapshots. See
[DOM snapshot checks](dom-snapshots.md) for the exact coverage/privacy contract.

Portable types have no session/secret-reference fields. Export includes no cookies,
credentials, browser profile, local secret store, login settings, logs, absolute
storage/runtime paths, binaries or database dump. Capture replacement metadata is
reduced to a strict numeric summary. Hash validation detects corruption, not a
dishonest sender; imported origins stay marked as portable provenance.

## Validation and publication

Import inspection snapshots the external ZIP into private staging. Raw central
and local directories are checked before ZIP-library name deduplication: canonical
names, case collisions, duplicates, local/central CRC and sizes, data descriptors,
overlap, unlisted/prepended payload and directory bounds. Symlinks, special files,
executable entries, encryption, multidisk and ZIP64 are refused. Only stored or
deflated entries are accepted; output uses stored entries with private file modes.

Limits are deliberately bounded:

- 1 GiB encoded ZIP and total expanded entries; export reserves metadata/ZIP overhead.
- 8 MiB manifest, 10,000 distinct originals, 128 MiB per object.
- Per-entry expansion at most `compressed bytes × 200 + 1 MiB`.
- At most 30,000 reconstructed source-target combinations for a composite.
- Images: 4096 wide, 32,768 tall, 64 million pixels; row-bounded PNG decoding,
  16 MiB decoder allowance. The shared strict analysis/storage gate accepts
  noninterlaced RGB/RGBA **8-bit** browser originals with IHDR, consecutive IDAT,
  IEND, optional sRGB and bounded tEXt. CRCs, the complete single zlib stream,
  Adler32, filters and the exact inflated row count are independently checked
  with a 32 KiB window before decoding or publication. Excess/short rows,
  trailing/concatenated compressed streams and malformed terminators are refused.
  Palette/grayscale/16-bit, interlaced/animated and other ancillary semantics are
  safely unsupported: older storage decoding could transform some of these, but
  strict analysis cannot interpret them. Refusal preserves existing evidence;
  no original is converted, rewritten or automatically deleted. Decoded
  dimensions must match metadata.

Every original's size/SHA-256 and complete PNG stream or canonical DOM structure/
privacy policy is validated before recipient publication. DOM has its own 8 MiB,
node/depth/attribute/string bounds; it never creates image thumbnails/tiles.
New screenshot thumbnails/tiles are constructed privately. Import always creates
a new project and rebinds project, inventory, page, profile, baseline, run,
environment and mask IDs. Name conflicts receive a separate `(imported N)` name.
It never overwrites a project or automatically visits/executes a website.

One SQLite transaction publishes the new project graph after complete synced,
immutable CAS objects exist. Cancellation, interrupted writes or disk-full errors
cannot publish a partial new active baseline. A failure after a complete object
write can leave an unreferenced recovery object, never a partial published object.
The original source store/history is read-only during export/import.

After import, explicitly configure the recipient environment/mapping, obtain a
matching verified runtime, establish a fresh login when needed, confirm the probe
and then request a check. OS/fonts/runtime differences still need compatibility
review. A successful local fixture round trip is not cross-machine qualification.

## History, deletion and recovery

History pages contain up to 64 runs and 64 baseline versions, with original dates,
active state, reference map, rendering/profile metadata and imported provenance.
Setting a retained version active independently validates its original files.
Per-project actual evidence bytes deduplicate hashes; originals and rebuildable
derivatives are separated, as are shared and exclusive ownership. Database/WAL,
runtime/network caches, login storage and recovery staging are not included in
that project evidence total. Exclusive bytes are not automatically deletable.

Deletion is manual and requires a fresh impact preview. Active baselines, retained
parent/child links, check dependencies and source runs needed by **any retained
composite** are protected. Pair-bound ignored-region history can also refuse a
source-run deletion. Shared objects survive; deletion authority fingerprints the
records/reference graph/revision/file impact and is rechecked under the writer lock.

Only explicitly confirmed unreferenced objects enter `pending_gc`. Startup may
finish this existing authorization journal, rechecking references for each object;
it does not scan and delete old history. No age/size-based automatic policy exists.

Settings separates these user-directed actions:

1. Inspect hashes/missing files, interrupted temporary writes and dead-owner import
   stages. Live, unowned or unsafe stages are retained. Corrupt referenced originals
   cannot establish a valid activation; repair does not invent missing evidence.
2. Confirm **temporary-only** cleanup, preserving complete recovery originals.
3. Separately confirm deletion of complete hash-valid unreferenced recovery objects.

Each action rechecks its preview. Artifact writes hold the SQLite writer lock, use
private temporary files and sync/no-clobber publication. Unix free-space preflight
retains a 128 MiB reserve and checks artifact writes during ingestion. Unsupported
free-space platforms fail closed. Real filesystem exhaustion, maximum-scale,
Windows/macOS and clean-recipient qualification remain separate outstanding gates.
