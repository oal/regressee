# 0004 — Bounded image evidence and comparability

Spike capture/decoder budgets: **4096 px width, 32768 px height, 64 million pixels**;
compressed PNG at most **256 MiB**. Oversize images fail, not crop. Capture is full
page, bounded navigation/readiness/scroll and three consecutive-byte stability
attempts. Nested scroll containers and virtualized content are not qualified.

Rust decodes non-interlaced PNG rows with a 16 MiB decoder limit and writes **512
row tiles**. Tile buffer at maximum RGBA width is 8 MiB; compressed originals are
preserved. Viewer retains one tile texture, clearing its GPUI cache on tile
navigation; maximum tile RGBA upload is 8 MiB, not one arbitrarily tall texture.
This is a tile pager operated by the scroll wheel, not production seamless pan/
zoom or a calibrated performance promise. All decode/process waits run off the
UI thread. Peak browser screenshot memory still needs testing near the budget.

Neutral-raster-v1 replaces loaded HTML img/picture sources with intrinsic-size
neutral SVG placeholders, preserving CSS size/object-fit/position/borders and
tested element rectangles. Existing SVG icons remain intact. Single known raster
CSS URL backgrounds use a neutral source while retaining background properties.
The fixture includes intrinsic, fitted, lazy and responsive images plus a raster
background. Complex layers, pseudo-elements, shadow DOM, frames, extensionless
URLs, canvas/video and raster/vector detection beyond known URL types are explicit
limitations. Raster logos may be replaced. No hidden original image is saved in
replacement-only output; original browser resources still load to establish layout.

Repeated fixture runs exposed an intermittent 3/4 replacement count. Initial
investigation added eager completion readiness after scrolling. Subsequent natural
dimension diagnostics identified the deeper fixture defect: the same 2×1 asset had
been declared as both 1x and 2x, allowing Chromium to select a density-corrected
image with naturalHeight truncated to zero. The old embedded PNG also had an invalid
IDAT CRC. Fixtures now generate CRC-correct 2×1/4×2 density-matched rasters, with a
regression test. Eager readiness remains explicit; no image downloads are blocked.

Profiles and metadata record viewport/screen/DPR, mobile/touch, UA, locale/timezone,
color/reduced motion, browser/Node/worker/algorithm versions, OS release/architecture,
actual screenshot dimensions and available computed-font metadata. Font-family
names and `document.fonts.check` are not font-file/GPU fingerprints. Material
profile/browser/algorithm mismatch blocks normal comparison; OS/architecture/
font metadata differences require caution. Site address mapping alone is not a
rendering mismatch. Same recorded environment is **not** a proof of identical GPUs
or installed font versions. Cross-OS visual equality remains untested here.
