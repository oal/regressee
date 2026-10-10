# Bounded unwrapped-layout retention — implementation handoff

2026-10-10. **Implemented in source; uncompiled and untested.** No builds,
test execution, GPU/native/browser launches, downloads or subagents. Parent's
running immutable pre-cache binary is unaffected by these source edits.
This is the implementer's historical handoff; subsequent parent compilation,
public-API regression and measured A/B are in `setup-render-profile-20261010.md`.

## Ownership and API

Only these files changed for this implementation:

- `vendor/gpui/src/text_system/line_layout.rs`: default-off window-local side cache
  and three authored pure unit tests.
- `vendor/gpui/src/text_system.rs`: documented public forwarder:
  `WindowTextSystem::retain_unwrapped_layouts(&self, max_entries: usize,
  max_payload_bytes: usize)`.
- This report.

Parent can call `window.text_system().retain_unwrapped_layouts(2048, 8 * 1024 * 1024)`
before constructing/rendering the shell. No environment flag, app/Cargo,
provenance, renderer/backend or platform-font changes were made here.

## Behavior and bounds

- The forwarder calls `LineLayoutCache::configure_retained_layouts`, which resets
  the mutex-protected side cache. Arguments clamp to 2,048 entries and 8 MiB of
  **accounted payload**, not RSS. Either zero disables admission and drops retained
  references/table allocation. Existing frame maps/vectors and live Arcs survive.
- On a current/previous unwrapped miss, lookup uses the existing borrowed
  `dyn AsCacheKeyRef` Eq/Hash contract. A retained hit is promoted into current
  `lines` and `used_lines` exactly as a previous-frame hit. No text/size/font-run/
  force-width rounding or key changes. Wrapped layouts are not retained.
- Newly shaped layouts are admitted only after existing force-width adjustment.
  Platform shaping runs without the retention lock. New nested lock use is
  frame → retention; configuration takes only retention. Previous-frame lock is
  released before retained lookup. Existing frame reuse/truncate/finish logic is
  unchanged.
- Recency is updated on side-cache lookup/admission; frame-only and wrapped hits
  do not touch the side cache. Eviction scans at most 2,048 entries for the oldest
  recency value. No historical touch queue or dependency. Clock rollover rebases
  ordering with a temporary entry-bounded list.
- Checked accounting charges key/layout/retained-entry structs, text bytes and
  conservatively three Arc headers, spilled FontRun capacity, ShapedRun vector
  capacity and every populated run's glyph-vector capacity. Inline SmallVec
  storage is already in the key struct. Oversized/overflowing payloads are skipped
  without evicting existing entries. Table/allocator overhead is count-bounded,
  not included in the 8 MiB payload claim. Shared allocations may be conservatively
  charged more than once.
- Cache scope is the particular WindowTextSystem, never another font system.
  Strings remain in memory until eviction/reset/window teardown; no immediate RAM
  erasure promise, persistence, raw text/hash/URL logging or metrics added.

**Fixed font database only:** public docs require disabling/resetting retention
around font additions, with ordinary frame invalidation still the caller's
responsibility. No font-database epoch plumbing was added. Defaults remain off,
including ordinary Noop-font tests. Configuration does not seed entries from
already-existing frame layouts; opt in before the first desired cold render.

## Authored tests — not run

`retained_layout_tests` exercises only in-memory cache helpers, no font backend:

1. Default-off behavior, argument clamping, either-zero disable/reset, and a live
   layout Arc surviving reset.
2. Borrowed exact-key lookup, differing font size/force width/font runs, LRU entry
   eviction and clock rollover.
3. Payload-driven eviction, unused glyph capacity charging and oversized admission
   rejection without displacing the existing entry.

These are not executed or claimed passing. They do not establish frame-index,
real shaping, rendering, concurrency or performance acceptance.

## Parent-owned next verification

Compile/typecheck these two modified source files and execute appropriate tests
after source handoff. Refresh parent-owned provenance/source bindings as needed.
Then compare default-off versus explicit retention with identical real Cosmic
fonts, calibrated renderer, corrected capture DPI 1, widths and form states;
separate cold Setup from repeated Projects ↔ Setup cycles. Verify geometry,
wrapping, readable captures and frame-reuse/focus behavior as well as timing.

Parent's pre-cache manifest remains at
`/path/to/private-tests/opencode/regressee-renderer-calibrated-y5e0o1nc/pre-cache-source.json`;
this implementation does not update or reinterpret that immutable binary evidence.
No latency threshold, first-cold-render improvement, full one-second diagnosis,
GPU/native fix or cross-OS qualification is claimed before measured A/B.
