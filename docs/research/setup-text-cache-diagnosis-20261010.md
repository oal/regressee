# Setup real-font layout latency — source diagnosis and bounded patch plan

**2026-10-10 — source-only investigation; no code changed or execution performed.**
Only this report written. No builds/tests, downloads, native/GPU/browser activity
or subagents. Parent owns capture/DPI/backend/app changes and measured verification.

## Finding

**Confirmed mechanism:** GPUI 0.2.2's per-window line-layout cache retains the
current and immediately preceding frame, not recently visited screens. A complete
Projects frame discards Setup-only layouts; the next Setup visit must shape those
lines again. This is a concrete candidate for the repeated real-font navigation
cost, **not a measured attribution of all 249–332 ms**.

Parent-reported measurements, not reproduced by this reviewer: real Cosmic/Blade
guided Setup 253–269 ms, advanced 327–337 ms, Projects 14–24 ms over repeated
wide/small cycles; navigation handler 0.01–0.05 ms, shell construction <1 ms,
layout/prepaint 249–332 ms, paint 3–5 ms. Noop-font 16–34 ms Setup is not a meaningful
real-font latency acceptance result. The approximately one-second user observation
is **not fully reproduced or diagnosed**. Parent is separately correcting the
capture-only default DPI 2 versus target 1; comparisons must use the corrected,
identical DPI. GPU presentation/native-surface faults are outside this report.

## Exact lifetime and hot call path

All paths below are under `vendor/gpui/src/` unless otherwise specified.

1. `elements/text.rs:97–106` creates fresh `TextLayout` state for SharedString
   elements when requesting layout. Main's newly constructed screen/form cannot
   depend on an old screen element's local measured state surviving navigation.
2. `taffy.rs:199–228` calls the measured closure with known dimensions and
   Definite/MinContent/MaxContent space, converted back to logical pixels.
   `elements/text.rs:346–378` maps intrinsic space to `wrap_width:None`, finite
   width to `Some(width)`, and has a local size shortcut for compatible requests.
3. Otherwise `elements/text.rs:380–401` performs truncation if needed, then calls
   `WindowTextSystem::shape_text`. `text_system.rs:425–485` resolves font runs
   and asks `LineLayoutCache::layout_wrapped_line` for each actual newline line.
4. `text_system/line_layout.rs:480–529` looks up exact wrapped keys in current/
   previous maps. A wrapped miss calls `layout_line` with **no wrap width**
   (`504`), then computes wrap boundaries on that shape (`505–506`).
5. `line_layout.rs:544–590` checks current/previous unwrapped maps before invoking
   `PlatformTextSystem::layout_line` (`564–566`). Linux's public adapter takes a
   write lock (`platform/linux/text_system.rs:183–184`); its implementation builds
   spans, constructs `ShapeLine::new(...Shaping::Advanced...)`, lays it out without
   Cosmic wrapping, and converts glyphs/runs (`375–456`). This is real shaping
   work and allocation, not just retrieving font metrics.
6. `window.rs:1952–1954` clears the layout engine and calls text `finish_frame`.
   `line_layout.rs:458–465` swaps maps and clears the old previous frame.
   Timeline: finish Setup → previous contains Setup; finish Projects → previous
   contains Projects, old Setup-only maps are cleared; return Setup → misses.
   The `Arc` inside a discarded WrappedLineLayout does not make its unwrapped
   shape discoverable by a future cache lookup.

**Do not misdiagnose intrinsic requests as unconditional repeated shaping.**
Different finite wrap widths miss the *wrapped* cache, but identical text/size/
font runs normally hit the unwrapped cache within that same frame. MinContent and
MaxContent both use `None` here. Actual repeated platform shapes require distinct
keys, loss of unwrapped lookup entries, or direct wrapper character-width calls.
There is an additional retention wrinkle: promotion of an existing wrapped layout
(`493–500`) does not promote its unwrapped entry into `current_frame.lines`.
Thus a later new width can need a fresh shape after the original unwrapped map
entry has aged out, even though a wrapped Arc still held the geometry.

`LineWrapper::truncate_line` uses pooled per-font/size character advances;
`line_wrapper.rs:193–223` bypasses LineLayoutCache only on missing character widths.
The wrapper pool survives frames (`text_system.rs:291–305`), so this is not proof
that truncation cold-shapes every character on every navigation.

## Keys and fallback: preserve semantics

- Existing `CacheKey`/borrowed `CacheKeyRef` include exact text bytes, logical
  `Pixels` font size, ordered FontRun lengths/FontIds, wrap width and force width
  (`line_layout.rs:595–659`). Keep the existing Eq/Hash implementation; no width
  bucketing, font-size rounding, text-only key or color-sensitive reshaping fix.
- FontId binds the resolved face/features in the current text-system instance.
  `TextSystem` caches both successful and failed font resolution (`text_system.rs:
  109–129`); Linux caches family/features candidates (`linux/text_system.rs:93–119`).
  The app explicitly requests **DejaVu Sans** on Linux
  (`crates/app/src/ui/theme.rs:29–35`), not Inter. Missing Inter or repeated font
  database scanning is **not established** as this screen's cause.
- Font fallback selected for glyphs is recorded in output runs (`linux/text_system:
  415–420`). Keep cache window/text-system-local; do not share FontIds across
  backends/instances. No font substitutions or per-OS backend changes are proposed.
- **Important wrapped-cache hazard:** `layout_wrapped_line` accepts `max_lines`
  but omits it from the key (`480–485`), while wrap computation uses it (`505–506`,
  `compute_wrap_boundaries:186–191`). Extending wrapped retention unchanged could
  extend stale clamp-dependent reuse. Do not make that the latency patch. This
  report does not reproduce a visible existing clamp bug.

## Minimal one-file correction proposal

Target **`vendor/gpui/src/text_system/line_layout.rs` only**: add a separate bounded
LRU of **unwrapped `Arc<LineLayout>`** to `LineLayoutCache`, leaving both FrameCaches
and wrapped-cache policy intact. No inspected public API currently enables
cross-frame retention. For controlled A/B, a **new proposed**, default-off process
flag read once by `LineLayoutCache::new` could opt this application into retention;
that is not an existing GPUI method/property. Parent decides production activation
after measurement. A public application setter would require another API file and
is not necessary for the one-file A/B candidate.

Patch requirements:

1. In `layout_line`, after current/previous misses and **before** platform shaping,
   look up the exact unwrapped key in the retained LRU. Promote a hit into current
   `lines` and append its key to `used_lines` exactly like the previous-frame hit.
   Admit successfully shaped/force-width-adjusted layouts; touch existing retained
   hits as appropriate. Do not retain WrappedLineLayout or decorated ShapedLine.
2. Bound to **2,048 entries plus an 8 MiB accounted-payload budget**. Charge key
   text, FontRun storage, layout/run structs and glyph vector *capacities*, using
   checked arithmetic. Skip a single oversize entry rather than exceed the budget.
   Hash-table/allocator overhead remains separately count-bounded: do not advertise
   that payload accounting as a hard cap on total process RSS. No unbounded
   duplicate-key touch queue; a bounded entry map with recency counters and oldest
   eviction is sufficient without a new dependency (bounded eviction scanning is
   an implementation tradeoff to measure).
3. Preserve `layout_index`, `reuse_layouts`, `truncate_layouts` and `finish_frame`
   behavior (`421–465`). The side cache is memoized geometry, never the authoritative
   frame-use/index vectors. Eviction drops only side-cache Arc references; it must
   not mutate layouts still referenced by frame maps or live elements. Keep lock
   ordering consistent; do not call platform shaping while holding the new LRU
   lock. Existing shape/style/mount/Tab semantics stay unchanged.
4. Retain only in process memory, scoped to the window/text-system lifetime;
   no persistence or text/URLs/FontRun payloads in diagnostics. Retention prolongs
   in-memory strings until eviction/window teardown, so privacy claims must not
   imply immediate destruction when a screen is hidden. Opt in only for this
   application's **fixed font database during the window lifetime**: no
   `add_fonts` call was found in `crates/app` source. Dynamic font additions can
   change glyph fallback without changing the requested FontId; the existing key
   has no font-database generation. General dynamic-font support would require
   explicit invalidation/epoch plumbing in `text_system.rs::add_fonts` as well,
   exceeding the one-file proposal. Do not enable persistent retention generally
   for other GPUI applications or promise unchanged dynamic-font behavior.

This avoids changing Taffy/text measurement contracts and addresses repeat visits
and width changes without multiplying cached variants. It does **not** eliminate
the first cold Setup render's necessary shaping, or guarantee all repeated latency
is due to cache misses. If warm shaping falls but layout remains expensive, report
that result rather than expanding this into speculative virtualization/style edits.

## Parent's bounded A/B evidence needed

Use the same real Cosmic text system, font database, corrected DPI, widths and
guided/advanced form state. Separate cold first render from repeated Projects ↔
Setup visits. Optional aggregate counters in the candidate file: current/previous/
retained hits, platform-shape count and accumulated time, wrapped misses, retained
entries/accounted bytes and evictions. No text/key hashes/URLs need be logged.
Counter expectations: width variants may increase wrapped misses **without**
increasing unwrapped platform shapes; returning screen content should move cold
shape misses to retained hits. Compare unchanged frame geometry/wrapping, focus
contracts and readable offscreen captures alongside timing, not elapsed time alone.

Parent owns one batched correction/profile and whole-UI capture validation plus
independent verification. This source investigation establishes the cache-loss
mechanism and a safe narrow candidate, **not a latency improvement, GPU fix,
one-second diagnosis, release-build cure or cross-OS/native qualification**.
The pinned Apache vendored-source provenance/binding must include any adopted
one-file correction; no dependencies, guardian/soak gates or public-runtime claims
need to change for this experiment.
