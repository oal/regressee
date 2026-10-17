# Comparison screenshot

`regressee-comparison.png` is an actual **1440 × 900** render of Regressee's native
comparison workspace in the **Unified neutrals / Light** theme, using GPUI's real
Blade offscreen renderer and installed Linux fonts. It is not a mockup or
generative image. Offscreen rendering does **not** qualify OS window presentation,
native input or accessibility.

The website pixels are unmodified managed-Chromium screenshots from the owned
loopback demo in [`scripts/cli-consumer-fixture.py`](../../scripts/cli-consumer-fixture.py),
not a customer project or external production website. An isolated documentation
project reuses those captures, rebuilds its baseline/check linkage and derives its
difference analysis through the normal app services. The pictured side-by-side
view shows the two originals; the app's View selector also offers the derived
difference. The demo's intentional page changes are not product-quality verdicts.

Only lossless PNG compression and embedded origin metadata were applied to the
renderer readback. No cropping, compositing, recoloring, retouching or replacement
of UI or evidence pixels was performed. The original browser captures are not
modified by the harness.

## Capture

The opt-in test is
[`ui_readme_capture.rs`](../../crates/gui/src/ui_readme_capture.rs).
Use a private, physical, mode-0700 `CAPTURE_ROOT` with a mode-0700 `captures/`
subdirectory. `consumer-copy/` contains **only a copy of an owned demo fixture**:
its `data/regressee.sqlite3`, `data/artifacts/`, and `fixture.json` identifying
`kind: actual-managed-Chromium-owned-loopback-demo`, `project`, `baseline_run`
and `changed_run`. Select the successful changed check, not the failure exercise.
Never point this test at an actual user's store. No website is visited during
the GUI render.

With Rust **1.98.1** and GPUI's native build dependencies already installed
(`RUST_1981_BIN` selects that toolchain; `GPUI_LINK_LIBS` supplies nonstandard
native-library locations when needed), the capture command is:

```sh
PATH="$RUST_1981_BIN:$PATH" \
TMPDIR="$CAPTURE_ROOT" \
LIBRARY_PATH="$GPUI_LINK_LIBS" \
REGRESSEE_RENDERER_APPROVED_ROOT="$CAPTURE_ROOT" \
REGRESSEE_RENDERER_CAPTURE_DIR="$CAPTURE_ROOT/captures" \
REGRESSEE_README_DEMO_SOURCE="$CAPTURE_ROOT/consumer-copy" \
cargo test --locked --offline -p regressee-gui --all-features \
  --bin regressee-desktop \
  ui_renderer_capture::ui_readme_capture::renderer_capture_readme_comparison \
  -- --exact --ignored --test-threads=1
```

The test checks loaded before/after/difference pixels, contained identity and
comparison panes, unchanged stored evidence, and writes one PNG with source-bound
renderer metadata to the private output directory. Diagnostic receipts, local
paths and runtime credentials are not shipped with this image.
