# Renderer-side screenshots — actual results, 2026-10-10

**Working:** actual GPUI/Blade GPU pixels with real Linux fonts and the same glyph/
image atlas used by the window. No wgpu rewrite, compositor permissions, desktop
capture, native-window launch, GPU selection flags or global registry edits.
This is offscreen rendering, **not OS presentation or OS-input qualification**.

## Delivery and calibration

- Pinned Apache-licensed GPUI 0.2.2 source override; nine modified files, 174
  unchanged; transitive package versions/dependencies unchanged. Hash provenance
  is in `vendor/gpui-provenance.json`.
- App feature `ui-renderer-capture` is default-off and test-only. Ignored explicit
  calibration/batch tests use isolated synthetic SQLite/CAS, never user projects.
- `TestAppContext::enable_offscreen_renderer()` installs real Cosmic/Blade before
  windows. `window.draw(cx)` constructs the actual scene;
  `window.capture_offscreen_frame()` submits it and returns owned RGBA pixels.
- Readback uses checked pitch/length math, strips padding, preserves bytes, and
  refuses nonopaque/blank output rather than guessing alpha/channel conversions.
  PNGs and JSON/hash sidecars are private; encoding is outside the UI callback.
- The first proof failed exact RGB calibration: upstream test scaling was 2× but
  the target was 1×. Its next readable image exposed double sRGB encoding.
  Offscreen-only scaling is now 1×, and the target is RGBA8 UNORM because GPUI's
  existing shaders already output sRGB values. Ordinary test DPI stays 2× and
  native formats/shaders are unchanged. No previous failure is relabeled PASS.
- Final calibration **PASS**, personally read: 640×440, opaque, readable actual
  text/field/filled action/radio and image atlas. Red/green/blue/midtone patches each
  have 1,024 expected RGB pixels (midtone tolerance ≤1 channel unit).
- Independent review found a P2 overflow in the extent admission guard. Both
  allocation paths now share a pixel-count guard, avoiding multiplication by four.
  A GPU-free boundary test (including 2³¹×2³¹ and u32::MAX²) independently **PASS**.
  Renderer/cache source verdict is now **PASS**; see its verification report.
- A five-second submission wait is not a process guardian: upstream device Drop
  can still wait for queue idle. No bounded whole-process shutdown claim is made.

## Actual UI inspection and bounded confirmation

The initial batch **PASS**, with **28 app PNGs** covering all five screens at
1120×800 and 820×600, guided/advanced Setup sections, environment/mapping/timezone,
timezone popup, lower network settings and validation feedback. These use real
rendering and synthetic fixture evidence. Import text, compact filled actions,
white inputs, choice insets and message-attached Dismiss are readable. This is
not blanket all-state/usability acceptance; Results/History lower states were
not part of that initial batch.

It exposed three defects: wide timezone value collapsed; environment login help
overlaps the next action; Create project inherits the preceding scroll offset.
One correction batch adds a flex value wrapper and explicit new-project scroll
reset. An ineffective full-width form trial was removed.

The one confirmation batch **FAILED** after **nine 1120×800 app PNGs**, before
the added lower Results/History states and all 820×600 confirmation states. UTC
is now visibly restored, but the wide selector chevron remains next to the value
instead of right-aligned; a strict geometry assertion stopped the batch. The
login-help overlap is still visible. The new-project scroll reset is implemented
but has no after-image in this confirmation. No third polish/capture loop run.

Therefore: **screenshots are delivered; whole-UI acceptance remains OPEN**.
The two visible wide-layout defects remain work, not an asserted UI PASS. The
original failed Wayland round's 33 OS-window/input states remain NOT RUN; these
offscreen images do not replace that evidence or diagnose surface loss.

Private evidence:

- `regressee-renderer-proof-xs17az7a`: retained failed first proof;
- `regressee-renderer-confirm-gk947cmp`: retained readable mis-encoded proof;
- `regressee-renderer-calibrated-y5e0o1nc`: accepted proof, 28 initial app images,
  pre-cache source/binary snapshot and historical CPU profiles;
- `regressee-renderer-final-66rr3ufh`: copied reviewed calibration, nine partial
  confirmation images, failure log, tests/Clippy and final A/B CPU profiles.

All are under `/path/to/private-tests/opencode/`; no private screenshots,
fixture stores or logs are committed. `source-snapshot.json` in the last folder
was recorded **before the extent fix/final diagnostic tests** and is not the
binding of its partial after-images. The confirmation contained the later-removed
width trial; no claim of complete final-source visual confirmation is made.

## Final local gates and settlement

Locked/offline serial workspace gate: **188 passed / 28 deliberate ignores**;
no new browser/runtime/guardian/scale qualification. Workspace/all-target Clippy
with `-D warnings` and format check pass. Its initial failure was three mechanical
`chunks_exact(4)` lint findings in the new diagnostic writer, corrected to
`as_chunks::<4>()`. Two unchanged upstream GPUI future-literal warnings remain
dependency warnings; none was silenced or promoted into a source correction.
Ordinary `ui` build passes and has **no inspection/capture feature**. It was not
natively launched. SHA-256:
`5c6150e292d36f9f5fa66dcd27295d4b98db79f327ef30cee845fda0dc47c22c`.
Vendor provenance and the exact two-line-only root lock change were rechecked.
Non-vendor staged whitespace check passes. The full staged check reports seven
space-before-tab lines in unchanged upstream macOS Metal source; its pinned hash
is preserved rather than formatting unrelated third-party shaders.
All agent/build/test work at that checkpoint was settled; remaining visual/native/
public-runtime/platform/release work was explicitly separate.

## Later user-reported overlap correction

After another native user screenshot reported text behind the button, a new
bounded correction session repaired the actual nested form constraints rather
than changing text alone. A standalone timezone trigger was correctly aligned,
but the same control in the scrolling Shell measured its arrow near the value.
The login hint measured 20px tall while the following action began only 2px below
its top. A focused full-Shell regression reproduced both failures.

The final correction gives scrolling form sections/fields explicit full width
and nonshrinking groups, uses a flex root for the selector, positions its drawn
chevron at the trigger's right edge, and reserves the help line's minimum height.
The full-Shell geometry regression passes at **1120×800 and 820×600**. Compact
action styling, field editability, selection/focus and the bounded text cache stay
unchanged. Obsolete public-setup-unavailable copy was removed only alongside the
actual official-component provisioning implementation.

The first new batch failed the strict arrow assertion after nine wide images;
that failure remains at `regressee-official-verification-r7ki95gq/`. The single
confirmation then **PASS** in **11.68s**, producing **36 app PNGs** (plus the copied
reviewed calibration) under that directory's `final-layout/`. It reaches both
sizes, all five screens, lower Results/viewer/analysis and History/network states,
the timezone popup, and new-project feedback/scroll-reset checks. The wide final
timezone image visibly shows separated login help and the next action; it was
personally reviewed and previewed to the user. At the smaller size this particular
scroll position places the login help below the viewport; its non-overlap is a
geometry result, not a separately visible after-image at that position.

The private `source-snapshot.json` binds the capture test executable and source
inputs. A later Clippy-only change replaces an equivalent byte-character array
with a byte-string array in a runtime test; no production UI changed afterward.
The subsequent backend-only post-commit cancellation-reporting repair changes
Product/Jobs sources after this capture binding; it changes neither UI markup nor
layout. The independent 36-image review and first-use run are preserved with
their own earlier bindings, not falsely rebound to the later executable.
All original black/native-surface failures remain preserved. These are real
GPUI/Blade offscreen pixels with synthetic evidence, **not OS presentation/input,
human usability, universal state coverage or full release acceptance**.

Final local suite: **204 passed / 29 deliberate ignores**, workspace/all-target/
all-feature Clippy with `-D warnings` and format check PASS. Initial Clippy failure
was the byte-character-slice spelling mentioned above; its log is retained.
Ordinary `ui` build PASS, no inspection/capture feature and not natively launched;
SHA-256 `cd93fd7d7648924b6dcf847468b731f5400cc5c269fbcca4ab5d6725be9c6d26`.

The later backend-only cancellation-reporting repair passes **206 tests / 29
deliberate ignores**, Clippy and fmt; its ordinary `ui` build SHA-256 is
`642db4c5c4e6951cf4d8824aa5c695fe6003cd42716ef74ade8cb7c5c033a3c7`.
That rebuild does not imply a new native or offscreen capture; no UI markup changed.
