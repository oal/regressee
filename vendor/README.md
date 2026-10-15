# Pinned GPUI offscreen patch

`gpui/` is the exact cached crates.io **gpui 0.2.2** source package, with a
contained opt-in Linux renderer-readback patch. Registry package checksum:
`979b45cfa6ec723b6f42330915a1b3769b930d02b2d505f9697f8ca602bee707`.

The original `LICENSE-APACHE`, build script, resources, examples, shader sources
and attribution are preserved. Only the registry's `.cargo-ok` marker was omitted.
`gpui-provenance.json` records all 183 original file hashes, the twelve modified
files (nine Rust files, Cargo.toml and the later Blade/Windows shader corrections),
and patched hashes; 171 files are unchanged. The Rust files include the existing
explicit-float lint correction in `src/taffy.rs`.

The root Cargo patch selects this same version by path. Cargo.lock removes only
GPUI's registry source/checksum; every other package/version/dependency is retained.
The directory is excluded from the workspace to avoid formatting all upstream code.

`offscreen-capture` implies `test-support`; its new API is Linux-only and must be
explicitly enabled on a fresh TestAppContext. Default tests still use placeholder
fonts/atlas, and ordinary native windows still use their existing surface renderer.
No graphics backend, shader or Blade dependency was replaced.

The Blade and Windows shadow shaders now handle zero blur explicitly, matching Metal's
signed-distance hard-edge path. A zero-width Gaussian previously produced NaNs,
leaving only a 1px border instead of the app's keyboard focus outline.
Nonzero-blur shadows, clipping and color handling are unchanged.

The additional window-local unwrapped text-layout retention API is default-off;
Regressee explicitly enables its bounded 2,048-entry / 8 MiB accounted-payload
cache with a fixed font database. Wrapped/frame-cache semantics are unchanged.
Offscreen targets use 1× test-window scaling and RGBA8 UNORM: the existing GPUI
shaders already produce sRGB values, so the attachment must not encode them twice.
Extent validation compares bounded pixel counts without overflowing byte math.

See `docs/research/renderer-capture-implementation-20261010.md` for the exact API,
compile checks, ownership and GPU proof limitations. No renderer calibration or
actual screenshot was run while implementing this patch.
