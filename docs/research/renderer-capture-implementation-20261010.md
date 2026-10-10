# GPUI offscreen frame readback implementation — 2026-10-10

**Implemented and compile-checked. GPU calibration, pixels and visual proof NOT RUN.**
This is the implementer's historical handoff. Subsequent parent execution,
calibration fixes and limits are in `renderer-capture-results-20261010.md`.
This is a local pinned GPUI 0.2.2 patch, not a wgpu rewrite, compositor capture,
CPU approximation or OS-presentation verification. Earlier reports remain intact.

## API and parent calling recipe

Parent enables `gpui/offscreen-capture` through its own app feature. On Linux:

```rust
let mut cx = gpui::TestAppContext::single();
cx.enable_offscreen_renderer()?; // before windows, views/text use or cx.clone()
// Bind actions/fonts and open the ordinary app view using this context.
// Inside the window update, after notifying/settling the intended state:
window.draw(app); // normal GPUI layout/prepaint/CPU scene construction
let frame = window.capture_offscreen_frame()?; // explicit GPU draw + readback
// Encode frame.rgba outside the UI thread after this callback returns.
```

Exact public APIs:

- `TestAppContext::enable_offscreen_renderer(&mut self) -> anyhow::Result<()>`:
  refuses existing windows, cloned contexts and repeated enable. Initializes one
  Blade context with `presentation: false`; replaces both app/context TextSystem
  and platform font system with the real Linux Cosmic/Swash system before windows.
- `Window::capture_offscreen_frame(&mut self) -> anyhow::Result<OffscreenFrame>`:
  refuses native or non-opted-in test windows. Renders the **current CPU scene**;
  it does not itself call `Window::draw` or change application state. Caller must
  build the intended scene first. Uses the same window BladeAtlas that received
  actual glyph/SVG/image pixels during CPU paint; no private Scene exposure.
- `OffscreenFrame { width: u32, height: u32, rgba: Vec<u8>, srgb: bool,
  premultiplied_alpha: bool }`: tightly packed, top-to-bottom RGBA8 data, fixed
  test-window scale **1**, target **Rgba8UnormSrgb**, premultiplied alpha.
  Opaque app pixels need no alpha conversion; the first calibration must check
  alpha is 255 for the opaque scene. For translucent PNG pixels, any conversion
  must respect sRGB transfer and the shader's premultiplication space; ordinary
  byte division is not established as correct here. Do not guess BGRA or multiply
  alpha a second time. Translucent PNG encoding remains uncalibrated.

Regular TestWindow draw remains empty; the explicit accessor is the GPU draw.
Thus real-font CPU profiling can measure Window::draw separately from GPU
submission/readback. Simulated resize updates the owned target/intermediates at
capture time while preserving atlas IDs. No actual OS window is opened.

## Renderer/readback and resource handling

Native construction still makes the existing surface and uses the same batch
loop/shaders, atlas, MSAA/path intermediates, blending and present/submit path.
The shared loop now selects either that native frame or an owned offscreen
texture/view; the offscreen branch never acquires/presents a surface or imports
Wayland dmabufs. Neither GPUI nor Blade registry/system sources were modified.

Readback uses a Shared buffer, 256-byte-aligned row pitch, texture-to-buffer copy,
submit and a **5-second bounded wait**, then returns owned bytes with padding
removed and frees the staging buffer. Maximum frame/readback allocation is
64 MiB. Unfinished work is retained and further capture/resize refused; no retry,
fallback, GPU flag or compositor policy change.

**Coherency is source-established**, not assumed: Blade 0.7.1 Vulkan
`src/vulkan/init.rs:686–703` excludes HOST_VISIBLE memory lacking HOST_COHERENT;
Shared allocation maps that filtered memory. Its `sync_buffer` is a no-op, so
reading occurs only after the completed GPU fence. Actual color/channel/alpha/
row-pitch correctness still requires the parent's authorized calibration PNG.

Normal resize/drop frees target, staging, intermediates, atlas, belts, pipelines,
sampler and encoder after completed work. A timed-out capture keeps its staging
handle and sync point; drop makes one additional bounded wait. **If the GPU
remains hung, in-flight resources are deliberately not freed unsafely** and a
constant error is logged; driver/device teardown is the remaining boundary.
No claim of successful cleanup on an indefinitely hung device is invented.
Device/pipeline initialization and scene construction are not covered by the
five-second fence timeout. The API can propagate initialization errors; existing
low-level Blade resource/pipeline operations may still panic on driver failures.

## Scope, provenance and checks

- Added `vendor/gpui`, `vendor/README.md`, `vendor/gpui-provenance.json`; preserved
  upstream Apache-2.0 license and resources/build/shaders. Manifest hashes all
  183 copied upstream files and modified files; 176 unchanged.
- Changed only six GPUI Rust files: `src/app/test_context.rs`, `src/window.rs`,
  `src/platform/test/{platform,window}.rs`,
  `src/platform/blade/{blade_context,blade_renderer}.rs`, plus its feature manifest.
- Root Cargo.toml adds the path patch/workspace exclusion. Root Cargo.lock changes
  **only** GPUI source/checksum removal, verified against the pre-change parsed
  lock; all transitive versions and dependency entries remain identical.
- No parent app/main/UI/backend/app-Cargo source edits, optimizations, profiles,
  tests, GPU/display/app launch, renderer calibration, screenshots, KWin helper,
  user-project access, registry changes, network/install or nested agents.
- Compile slot: `cargo check --locked --offline -p regressee-app --features
  ui-tests,gpui/offscreen-capture -j4` **PASS**; ordinary `--features ui-tests`
  **PASS**. Both used approved TMPDIR and absolute `.runtime/link-libs`.
  First two checks caught/fixed private-module access and missing public-field
  docs; final checks passed. No GPU code was executed by these checks.
- The now-local upstream emits two pre-existing float-literal future-compatibility
  warnings in **unmodified `src/taffy.rs:312`**, plus the existing proc-macro-error2
  notice. No warning suppression or unrelated upstream cleanup was added.
  Windows/macOS targets were not compiled; APIs are Linux-gated.

**Compile slot released at handoff.** Parent owns app feature/integration wiring
and one separately authorized real-font/atlas/color calibration followed by the
finite whole-UI batch and personally read images. There is no visual or native
input/IME/OS-presentation/one-second Setup-latency claim from this implementation.
