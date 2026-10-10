# Setup navigation profiling — 2026-10-10

The user reports a repeatable roughly one-second Projects → Setup delay and
requests renderer-side screenshots. This is not resolved by the prior passing
regressions or failed compositor capture. Profiling now proceeds independently
of screenshot availability.

## First measured CPU baseline

`tests::setup_cpu_render_profile` uses the ordinary Shell navigation handler and
`Window::draw` with an isolated saved project, actual storage actor, both supported
viewport sizes, guided/advanced forms and three navigation cycles each. Storage
completion is outside the timed draw; there is no browser or display launch.
These measurements are **GPUI test-platform CPU**, not native fonts, GPU
presentation, input latency or proof that the user's observation is absent.

Locked/offline debug test, `REGRESSEE_UI_DIAGNOSTICS=1`, serial execution:

| Setup form / size | Draw times (ms), three cycles |
| --- | --- |
| Guided 1120×800 | 16.662, 16.889, 16.873 |
| Advanced 1120×800 | 19.276, 20.197, 21.826 |
| Guided 820×600 | 15.971, 15.823, 15.646 |
| Advanced 820×600 | 27.948, 33.795, 33.794 |

Setup handler: **0.006–0.013 ms**; shell construction: **0.571–0.881 ms**.
Most draw time is layout/prepaint (approximately 13–29 ms); CPU scene paint adds
approximately 3–5 ms. Projects draw: **2.696–3.952 ms**. One profiling test passed;
its success is not a performance acceptance threshold. Timings are observations
from this machine, not stable CI budgets.

An explicit body-width constraint trial did not demonstrate an improvement and
was removed. No speculative geometry change was kept as a latency fix.

The test-platform path does not reproduce a one-second delay. No speculative
runtime/download/database or release-build explanation is claimed. Next work
must distinguish native font/layout, GPU submission and frame scheduling rather
than requiring an OS screenshot before investigating any of them.

## Renderer capture

Source investigation of the pinned GPUI renderer/readback path is complete in
`renderer-capture-feasibility-20261010.md`. Any direct/offscreen renderer image
must be labeled separately from compositor-presented pixels and native input.
Only app-owned synthetic UI is in scope; no desktop or existing user project
capture, compositor permission bypass, GPU flag experimentation or replacement
of the application's rendering engine is authorized by this profiling work.

Blade already supports texture readback; GPUI's private surface-bound plumbing
does not expose it. A contained pinned GPUI override is implemented with
an opt-in offscreen TestWindow, existing shaders/atlas and real CosmicTextSystem.
Default test/native rendering remains unchanged. First proof is an explicitly
requested calibration image, then personal image review before a synthetic app
batch. Real-font CPU profiling is separate from GPU readback/presentation timing.

## Measured real-font correction

The Noop-font baseline hid the dominant cost. Real Linux Cosmic shaping and the
actual Blade sprite atlas reproduce expensive Setup layout in the **debug** build.
Source inspection identified the frame cache evicting Setup-only text after a
Projects frame. A default-off GPUI API now allows bounded window-local retention
of **unwrapped** geometry: 2,048 entries / 8 MiB accounted payload (not RSS).
Regressee enables it once on window creation; its font database is fixed.
Wrapped geometry, exact font/text/size keys, decorations and frame indices remain
unchanged. No on-disk cache or text logging is added.

Final same-binary A/B at calibrated 1× test scaling, same UI, real fonts, three
cycles per size/form. In ON cycle 0 the side cache is explicitly reset to measure
cold shaping; cycles 1–2 are repeat Projects → Setup navigation:

| Form / size | Retention OFF, ms (three cycles) | ON cold, ms | ON repeat, ms |
| --- | --- | --- | --- |
| Guided 1120×800 | 326.868, 317.155, 327.947 | 308.890 | 26.695, 26.590 |
| Advanced 1120×800 | 413.784, 419.927, 364.791 | 386.538 | 30.867, 32.084 |
| Guided 820×600 | 363.754, 349.794, 320.002 | 301.867 | 22.818, 22.305 |
| Advanced 820×600 | 395.180, 391.220, 400.165 | 396.086 | 35.945, 36.179 |

Repeat draw cost falls from **317–420 ms to 22–36 ms** (roughly an order of
magnitude). First cold cost remains **302–396 ms**. These are CPU layout/prepaint
and paint observations, not a fixed CI budget or GPU-visible/input latency.
The complete user's ~1s native delay is **not closed** by this measurement.

The parent public-API regression confirms underlying layout Arc identity across
unrelated frames, exact size/forced-width misses, reset and tiny-payload refusal.
It initially compared wrapper addresses incorrectly; switching to `Arc::ptr_eq`
corrected the test, which passed. Three vendor-internal authored pure tests remain
NOT RUN: GPUI is excluded from the workspace, so the attempted package-target
commands did not run them; no dependency-graph/lock expansion was performed.
Independent source review accepts cache semantics and production enablement in
`renderer-and-cache-source-verification-20261010.md`.

An explicit form-width trial was also tried during image correction but did not
fix the observed overlap; it was removed. Timings above are after removal. Renderer
calibration, actual image results and remaining UI defects are recorded separately
in `renderer-capture-results-20261010.md`.
