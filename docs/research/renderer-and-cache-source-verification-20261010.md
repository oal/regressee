# Independent renderer and cache source verification — 2026-10-10

**Current verdict: PASS — the sole P2 is independently confirmed fixed; retained-layout cache source review remains PASS.** No unresolved material P1/P2 finding within the reviewed scope. See the confirmation appended below for the independently executed pure test and current production opt-in. The initial review below is retained as historical evidence: at that stage only this report was written, with no source edits, builds, tests, GPU/native execution, OS input, or further delegation.

## Initial material finding — subsequently resolved

### P2 — Offscreen allocation limit can overflow before rejecting the extent

`vendor/gpui/src/platform/blade/blade_renderer.rs:380–382` (`new_offscreen`) and `:1060–1067` (`resize_offscreen`) compute `u64::from(width) * u64::from(height) * 4` without checked multiplication. The first product fits `u64`, but multiplication by four need not.

For example, width and height both equal to `2^31` produce a mathematical byte count of `2^64`: overflow-checked builds panic instead of returning the advertised validation error; ordinary release arithmetic wraps to zero, passes the 64 MiB guard, and reaches texture allocation with an enormous invalid extent. These values are exactly representable in the test window's `f32` bounds and survive the resize conversion through `i32` back to `u32`. The later checked readback pitch does not protect the earlier texture allocations.

This is restricted to the opt-in diagnostic renderer, not a demonstrated production regression, but invalidates its promised allocation admission bound. Fix both guards before accepting that bound: compare the safely representable `u64` pixel product against `(64 * 1024 * 1024) / 4`, or use checked byte-size multiplication and reject overflow. Parent-owned verification should cover oversized/overflowing extents without initializing a GPU, alongside accepted ordinary extents. No reproduction was executed in this review.

## Renderer checks that pass at source level

- `offscreen-capture` is absent from GPUI default features; the application capture feature is also default-off. Public `Window::capture_offscreen_frame` is Linux/feature-gated and downcasts exclusively to `TestWindow`; an ordinary test window without opt-in returns an error. Native windows cannot take this capture route.
- `BladeContext::new_offscreen` sets `presentation: false` on a fresh device context. No native display/window creation, environment mutation, global GPU selection policy, or registry modification appears in the reviewed additions. Native `BladeContext::new` remains byte-for-byte unchanged.
- The production renderer's surface becomes `Some(surface)` through the extracted constructor. Its format/alpha still come from the native surface. Draw still acquires, initializes, renders, presents, submits, and waits for prior work in the upstream sequence. Native pipelines/shaders are unchanged. `surface_info` is refreshed on reconfiguration. Ordinary `TestWindow::draw` stays a no-op and ordinary test DPI remains 2; only an installed offscreen renderer gets DPI 1.
- The opt-in platform installs one real `CosmicTextSystem` into both the platform and app text system before windows exist. The window adopts the renderer's actual atlas before painting. CPU scene construction and subsequent capture therefore use the same font system and atlas rather than independent/reconstructed glyph IDs.
- The actual target is `Rgba8Unorm`, with existing shader-produced RGB values and premultiplied alpha; no additional sRGB attachment encoding or channel/alpha correction is performed. The existing shader's `blend_color` multiplies RGB by alpha when requested. Native shader behavior is not rewritten. Returned bytes are copied into an owned RGBA vector; the diagnostic app requires opaque output rather than guessing an unpremultiplication.
- The target texture has `TARGET | COPY | RESOURCE` usage. Readback row pitch uses checked RGBA-byte arithmetic and 256-byte alignment; padded readback length is separately capped at 64 MiB. Rows are stripped into tightly packed RGBA without modifying pixels.
- The mapped buffer is read only after a successful submission sync wait. Blade 0.7.1 Vulkan's `wait_for` waits on the submitted timeline value; its memory selection excludes non-coherent host-visible memory (`vulkan/init.rs:686–703`), and `sync_buffer` is a no-op for this backend. The mapped slice remains valid until copying finishes, after which the buffer is destroyed.
- Capture timeout retains the pending buffer and sync point, preventing recapture and resize. `TestWindowState::drop` invokes renderer destruction; offscreen destruction waits again and deliberately skips resource destruction if work is still unfinished. This avoids explicitly freeing in-flight resources, at the cost of retention until device teardown. **The five-second wait bounds that capture wait, not total shutdown:** upstream Blade context destruction calls `queue_wait_idle`. This is not a whole-process hang guardian or a bounded aggregate GPU-memory claim.

## Retained unwrapped-layout cache: PASS

- `RetainedLayouts::default` disables retention. `WindowTextSystem::retain_unwrapped_layouts` configures a window-local side cache, clamping entries to 2,048 and accounted payload to 8 MiB. Either zero disables admission; reconfiguration replaces the table and drops its references without invalidating frame/live `Arc` holders.
- The upstream borrowed-key equality/hash implementation is unchanged: exact text, font size, complete `FontRun` sequence, wrap width and forced width participate. Retained insertion is exclusively in `layout_line`, whose wrap width is `None`; wrapped layouts and their wrap boundaries are not retained. Platform shaping and forced-width adjustment remain unchanged.
- Existing current/previous frame lookup, used-key vector insertion, layout indices, reuse, truncation and frame swap/clear behavior are preserved. Retained hits re-enter the same current-frame bookkeeping as previous-frame hits. The omitted `max_lines` in upstream wrapped keys is pre-existing, not a newly retained wrapped-cache hazard.
- New lock acquisition is frame → retained; configuration takes retained only. There is no retained → frame/platform callback path and no platform shaping under the new lock. The upstream current/previous-frame lock-order inversion is not introduced by this patch. No new material deadlock was found.
- Admission accounts string length, spilled key-run capacity, layout-run capacity and every glyph-vector capacity using checked arithmetic. Oversized/overflowing payloads are rejected before eviction/admission. Entry eviction and byte subtraction keep accounting within configured bounds; LRU clock rollover uses only a bounded temporary list. Hash-table/allocator overhead and other owners' layouts are not covered by the 8 MiB payload number; this is not an RSS limit.
- Text keys remain bounded in-memory data, with no new logging or persistence. A fixed font database is an explicit API precondition; font additions require retention reset plus ordinary frame invalidation. No `add_fonts` call was found in `crates/`. At review time the sole application opt-in was the `REGRESSEE_PROFILE_RETAIN_TEXT=1` profiling test, not normal application startup.
- Three new pure cache tests were inspected but **not run**. Compilation/test execution and any production enablement remain parent-owned.

## Provenance and evidence boundaries

Compared actual vendor files with `/path/to/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2`, not only implementation reports. Every `upstream_files` hash matches that registry source. There are exactly nine modified upstream files: `Cargo.toml`, `src/app/test_context.rs`, `src/platform/test/{platform.rs,window.rs}`, `src/platform/blade/{blade_context.rs,blade_renderer.rs}`, `src/window.rs`, `src/text_system.rs`, and `src/text_system/line_layout.rs`. No extra vendor files were found; `src/platform/blade.rs` is unchanged (there is no modified Blade `mod.rs`).

The known pending provenance refresh is not an additional blocker: `src/text_system.rs` and `src/text_system/line_layout.rs` are not yet declared, and the renderer/test-window modified hashes are stale. Refresh after the extent fix and final edits.

The parent's corrected-DPI calibration/manual image review and CPU timing improvements were supplied context, **not independently rerun evidence**. This review makes no claim that native presentation was tested, that no-op tests establish native performance, or that the user's full approximately one-second delay is fixed.

## Independent fix confirmation — 2026-10-10

**PASS.** Inspected the actual updated source and independently ran only the explicitly authorized existing-binary pure extent test. No cargo/build, GPU/window/display initialization, fixture capture, native/OS input, downloads, source edits, or subagents were used. The report is the only edited file.

### Resolved P2

- `vendor/gpui/src/window.rs:836–849`: Linux/offscreen-gated `OffscreenFrame::validate_dimensions` rejects either zero dimension, then compares `u64(width) * u64(height)` against 16,777,216 pixels. Every pair of `u32` dimensions has a product representable in `u64`; there is no subsequent multiplication by four in this admission guard.
- `vendor/gpui/src/platform/blade/blade_renderer.rs:373–395`: `new_offscreen` retains its depth-one check and calls the helper before `new_with_surface` can allocate textures.
- `vendor/gpui/src/platform/blade/blade_renderer.rs:1051–1063`: `resize_offscreen` retains its native/unfinished-work rejection and calls the same helper before drawable-size changes or texture allocation. Accepted nonzero dimensions are individually at most 16,777,216, so the later `i32` conversion is also safe.
- Readback's checked pitch arithmetic, padded-length admission, wait-before-mapped-read, owned row copy, and timeout retention remain intact. Updated frame documentation correctly describes `Rgba8Unorm` containing shader-produced sRGB values, not an sRGB attachment.

### Independently executed pure test

Source inspected: `crates/app/src/ui_renderer_capture.rs:13–27`. It calls only dimension validation, not GPU/device/window setup. Accepted cases are `(640,440)`, `(1120,800)`, `(820,600)`, and `(4096,4096)`; rejected cases are both zero-dimension variants, `(4096,4097)`, `(2^31,2^31)`, and `(u32::MAX,u32::MAX)`.

```text
target/debug/deps/regressee-9595b0a016b31187 ui_renderer_capture::offscreen_extent_limits_without_gpu --exact --nocapture --test-threads=1

running 1 test
test ui_renderer_capture::offscreen_extent_limits_without_gpu ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 24 filtered out; finished in 0.00s
```

SHA-256 of that exact existing executable, independently read both before and after execution:

```text
a86d87787ac9976e7f6b8edddc55c482f4e1e04a50a1004bf2988f2b41a7d5f9
```

This confirms the focused pure test, not the entire UI suite or a fresh build. The three GPUI-internal retention tests remain **NOT RUN** by this reviewer; the parent's excluded-workspace/package-selection failures do not constitute their execution.

### Current production cache opt-in and provenance

`crates/app/src/main.rs:5448–5455` now enables the existing bounded window-local retention API with 2,048 entries / 8 MiB accounted payload before constructing `Shell`. GPUI itself still defaults retention off. Rechecked the API's fixed-font-database/reset contract and searched `crates/` for font additions; no `add_fonts` call was found. No new material architecture, key-identity, wrapped-layout, resource-lifetime, logging, or persistence issue was identified in this enablement. Existing raw text remains in the bounded RAM cache; this review did not capture or output user frames/data.

Rechecked provenance against registry originals: all upstream hashes still match, the modified-file set remains the same nine expected files, and there are no extra vendor files. Known pending bookkeeping remains nonblocking: the two text-cache files are undeclared and modified hashes for `src/window.rs`, `src/platform/blade/blade_renderer.rs`, and `src/platform/test/window.rs` require refresh. No additional unexpected vendor modification was found.

Final same-geometry, corrected-DPI CPU A/B figures supplied by the parent are OFF guided 315–330 ms / advanced 409–422 ms; ON reset-cold 319–407 ms, warm guided 24–31 ms / advanced 28–36 ms. These are **parent-supplied, not independently measured here**: cold work is not eliminated, historical pre-layout-change timings are not the final comparison, and no full user-delay/native-presentation claim follows. Pending screenshot confirmation of the small UI changes remains parent-owned. The previously noted unbounded upstream device-teardown `queue_wait_idle` caveat still applies.
