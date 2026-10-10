# Direct renderer capture feasibility — 2026-10-10

**Yes: Blade already supports GPU texture → host-buffer readback. No wgpu
replacement or compositor permission is necessary. GPUI 0.2.2 needs a contained
offscreen-rendering patch; it does not expose this as an app-level screenshot API.**
Source inspection only: no build, test, app/display probe, capture or source edit.
Earlier failed captures remain unchanged. This does not diagnose Setup latency.

## Pinned implementation, not newer API assumptions

Paths below are relative to the installed registry directories
`~/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2/` (**G**) and
`.../blade-graphics-0.7.1/` (**B**). Line numbers identify inspected source.

| Question | Actual pinned source |
| --- | --- |
| Real Linux draw path | G `src/window.rs:2008` passes `rendered_frame.scene` to `PlatformWindow::draw`; G `src/platform/linux/wayland/window.rs:1020–1032` forwards to `BladeRenderer` and returns its sprite atlas. |
| Why current renderer still needs presentation | G `src/platform/blade/blade_renderer.rs:327–363`: mandatory `gpu::Surface`, constructor `create_surface_configured`; `:647–651` acquires a surface frame; `:665–905` draws actual GPUI batches to its texture view; `:909–917` presents/submits. Adding readback **after** this path does not avoid the recorded surface failure. |
| Public scene/pixel capture | No `CaptureScene`/`capture_scene` symbol found in pinned G `src`. `Scene` is `pub(crate)` (`src/scene.rs:24`), as are Platform/PlatformWindow/PlatformTextSystem/PlatformAtlas (`src/platform.rs:163,460,577,762`). `App::new_app` is crate-private (`src/app.rs:596`). An external custom Platform cannot simply be plugged into this release. A scene-command dump would not contain rendered pixels or sufficient atlas textures. |
| Headless application | Public `Application::headless()` exists (`src/app.rs:143–151`), but Linux `HeadlessClient::open_window` **bails** (`src/platform/linux/headless/client.rs:94–99`). It is not an offscreen window renderer. |
| Existing test graphics | `TestWindow::draw` is empty (`src/platform/test/window.rs:269`). TestAtlas discards pixel bytes from `build()` (`:329`) and makes dummy texture IDs (`:339–351`). It cannot feed a real Blade renderer unchanged. |
| Existing test fonts | `TestPlatform::new` installs `NoopTextSystem` (`src/platform/test/platform.rs:105`). `src/platform.rs:654–664` supplies zero raster bounds/empty glyph pixels; `:666–710` supplies placeholder advances. Adding fonts to this system does not replace it. Test bounds/timings are not real-font rendering evidence. |
| Real Linux fonts without display | G `src/platform/linux/text_system.rs:22,53–64`: crate-private `CosmicTextSystem::new()` uses `cosmic_text::FontSystem::new()` / SwashCache. Native Linux selects it at `linux/platform.rs:111–114`. Window rasterizes glyphs at `window.rs:2980,3036`. This can be reused inside GPUI without a Wayland window. |

## Readback operations already present in Blade 0.7.1

- `unsafe gpu::Context::init(gpu::ContextDesc { presentation: false, ..Default::default() })`:
  B `src/lib.rs:94–108`, `src/vulkan/init.rs:297`. Presentation-gated surface/
  swapchain extensions occur at `init.rs:363,427–436,467,564–565`. This avoids
  swapchain creation, Wayland dmabuf import and compositor presentation—not GPU
  device/pipeline initialization itself. GPUI's existing `BladeContext::new`
  hardcodes `presentation: true` (G `blade_context.rs:24–35`); patch an opt-in
  constructor, do not change normal rendering or use GPU environment workarounds.
- `create_texture(TextureDesc)` / `create_texture_view(TextureViewDesc)`;
  `TextureUsage::TARGET | TextureUsage::COPY`, ordinary owned texture with
  `external: None`. GPUI already uses these operations (`blade_renderer.rs:927–950`).
  Formats `Rgba8UnormSrgb` / `Bgra8UnormSrgb` exist (B `lib.rs:309–312`).
- `create_buffer(BufferDesc { name, size, memory: Memory::Shared })`:
  B `lib.rs:201–205`; Shared supports DOWNLOAD and mapped host access
  (`vulkan/resource.rs:29–34,129–143`).
- Encoder `start`, `init_texture`, `render`; then drop the render pass and call
  `command_encoder.transfer(label).copy_texture_to_buffer(texture.into(), buffer.into(), bytes_per_row, extent)`.
  Actual signature: B `traits.rs:80–86`; transfer constructor
  `vulkan/command.rs:332`; implementation `:665–680`.
- `gpu.submit(&mut encoder)` → SyncPoint; `gpu.wait_for(&sync, timeout_ms)` → bool
  (B `traits.rs:40–48`); after completion, `gpu.sync_buffer(buffer)` and
  `Buffer::data()` / `BufferPiece::data()` expose mapped bytes
  (`lib.rs:219–233`, `vulkan/mod.rs:184–187`). **Not wgpu `map_async`.**
  Vulkan `sync_buffer` is a no-op (`vulkan/resource.rs:344`), so a proof must
  check the readback/coherency behavior rather than assume that call invalidates
  noncoherent memory. Respect row pitch, format channel order, sRGB and alpha;
  copy owned bytes before destroying GPU resources; encode PNG off the UI thread.

## Smallest practical change: opt-in offscreen TestWindow inside GPUI

Recommended: one **pinned local GPUI override**, feature-gated, changing approximately
six files plus feature/dependency wiring—not a new renderer or handwritten CPU
rasterizer. No changes to Blade are established as necessary by this inspection.
Keep GPUI's Apache-2.0 license/attribution and unchanged shader sources; preserve
a small reviewable patch against 0.2.2. Nothing was vendored or modified here.

1. **`platform/blade/blade_context.rs`**: add an offscreen constructor with
   `presentation: false`. Keep existing native constructor untouched.
2. **`platform/blade/blade_renderer.rs`**: factor its existing batch loop into a
   target-view/extent/`SurfaceInfo` path shared by normal and offscreen draws.
   Add an owned target variant and readback; make constructor/resize/drop avoid
   Surface calls in that variant. Reuse `BladePipelines::new` (`:137–165`),
   WGSL, path/MSAA intermediates, blending/gamma and BufferBelt. Explicit
   `SurfaceInfo { format, alpha }` is an existing Blade type (`B lib.rs:1166–1169`),
   not something that requires a real surface. Replace capture-path infinite GPU
   waiting (`G :433–452`) with a bounded error; never fall back on timeout.
3. **`platform/test/platform.rs`**: opt-in real `CosmicTextSystem` and shared
   offscreen context before TextSystem creation. Default tests stay unchanged.
4. **`platform/test/window.rs`**: opt-in renderer instead of empty `draw`; return
   that **same renderer's BladeAtlas** from `sprite_atlas`. Store/read back only
   this window's completed target. Atlas cannot be swapped only after paint:
   scene sprite IDs must reference the atlas populated during that window's paint.
   BladeAtlas uploads genuine glyph/SVG/image bytes (`blade_atlas.rs:83–100`),
   and draw already manages before/after-frame uploads (`blade_renderer.rs:645,914`).
5. **`app/test_context.rs`**: small opt-in builder/readback access that constructs
   the above platform **before** both TextSystem/App are made (`:124–139`).
   Keep scene/atlas plumbing internal; expose owned RGBA bytes, dimensions and
   format metadata to the app harness, not a new public private-Scene dependency.
6. **Cargo feature wiring**: opt-in Linux/test-support renderer capture, retaining
   existing Blade and real Linux text dependencies. Reuse ordinary Shell draw/
   layout/state flow at fixed logical size/scale; do not reimplement app UI.

These constructors/readback accessors are **proposed patch APIs**, not methods
already available. Source signatures were inspected; no new Rust was type-checked.
Existing headless/test facilities provide scaffolding, not an existing complete
headless screenshot implementation.

## Bounded next step and honest confidence

**High confidence in feasibility; implementation/readback correctness unproved.**
Authorize the contained dependency patch separately. First proof: one offscreen
GPUI frame containing real DejaVu Sans text, filled action/white input, radio path
and a tiny synthetic image; read its PNG and confirm font/atlas/color correctness.
Then the finite app UI batch can use the same real renderer without KWin helpers,
OS input or a display server. No swapchain-based capture fallback or GPU loop.

Those images are **real GPUI/Blade-rendered offscreen UI**, useful for actual
layout/clipping/style review, not synthetic bounds diagrams. They **do not verify
OS presentation, native focus/IME/input, or GPU-visible navigation latency**.
The immediate blocker is GPUI's private surface-bound plumbing, not lack of a
GPU copy API. Offscreen GPU initialization/rendering could still fail; report
that once if it does. Do not postpone the parent's separate Setup CPU investigation.
