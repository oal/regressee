# Native capture infrastructure gate — 2026-10-10

**BLOCKED pending rebuilt-source freeze and explicit authorization. No qualified
native visual pass; the prior black-frame root cause remains unknown.**

Read-only infrastructure investigation, not the parent's layout repair. No app
source edits, builds, tests, application/server launches, display connections or
input. Original reports and evidence are unchanged. No subagents/probe processes.

Owned physical, mode-0700 scratch (**N**):
`/path/to/private-tests/opencode/regressee-ui-sweep-hkonn1qy/native-capture-gate-f54858fc90a8/`

`N/investigation-manifest.json` records input hashes and zero launch/test counts.
Its HOME/XDG/TMP/store/cwd/bin/evidence directories are reserved, not initialized
with user data. All ancestors were checked for symlinks. Previous evidence (**E**):
`/path/to/private-tests/opencode/regressee-ui-sweep-hkonn1qy/verifier-a69456f1db/`.

## What the previous evidence actually proves

- `E/native_probe.py:8–27` discovers an exact PID/title client, subscribes to
  Exposure/Visibility/Structure events, waits **one second**, then takes its only
  screenshot and aborts on blank pixels. The reused definition file is
  `/path/to/private-tests/native-owned-window-20261010-sol-bd04e44032/owned_probe.py`.
  Its original entry point used bounded 0.5/2/5-second observations (`:437–442`),
  but that entry point was **not executed** by this isolated driver (`:3`). Thus
  there is no five-second observation for this isolated app.
- `E/result.json:6–60`: MapNotify, unobscured VisibilityNotify (state 0), and
  full-client Expose were **dequeued by the observer** at elapsed ~0.306 s.
  These are observer timestamps, not GPUI receipt/processing timestamps.
  The single sample began at 1.257429 s, ended at 1.384728 s; cleanup finished
  at 1.403647 s. The clock starts with the controller, not app Popen.
- The viewable client was 1120×800, depth 32, at root coordinates (142,100),
  border 0, override-redirect false. Server argv has a 1400×1000 screen.
  `[142,1262) × [100,900)` is wholly inside that screen. Together with
  unobscured visibility and successful XGetImage, this evidence does **not**
  support root-edge clipping/occlusion as the explanation for this sample.
- `E/projects-initial.png`: 896,000 black RGB pixels, one RGB color, SHA-256
  `0fbef93e6b721fb075c6a2846987f1c3789eeb44d26a0a0b050a6e6b500a82a0`.
  No native input was sent; no focus readback was recorded. The observer did
  **not** subscribe to FocusChange, so its absence of FocusIn cannot prove the
  app was unfocused. Only Xvfb/app were launched by the driver, not a WM.
- `E/projects-initial.log` identifies llvmpipe and adapter initialization only;
  it contains no frame completion/presentation evidence. `E/final-audit.json`
  records reaped app/server PIDs, no live owned PIDs and no remaining owned
  display socket/lock/keymap.

The original report `docs/research/ui-sweep-verification-20261010.md:107–144`
correctly limits its black-frame claim to this one sample. The user's working
plain desktop application is not contradicted or touched by this investigation.

## Pinned GPUI: mapping/expose/focus are different from presentation

Source root (**G**):
`/path/to/cargo/registry/src/index.crates.io-1949cf8c6b5b557f/gpui-0.2.2/src/`.
Cargo.lock pins GPUI 0.2.2 and blade-graphics 0.7.1; consulted files are hashed
in N. These are dependency reads, not modifications.

| Source | Concrete behavior / consequence |
| --- | --- |
| `G/platform/linux/x11/window.rs:426–438,645–665`; `G/window.rs:1018–1067,1205` | App selects Expose/Map/Visibility/Focus events; renderer and callbacks are initialized before natural mapping. Mapping alone still is not a completed frame. |
| `G/platform/linux/x11/client.rs:538–540,612–616,747–759,1698–1775` | Expose sets a presentation flag; Map/Visibility start the refresh loop when mapped and not fully obscured. **No active/focused/WM predicate gates this loop.** |
| Same `client.rs:1785–1813,1891–1899` | Refresh timer starts immediately and requests frames; Expose becomes `require_presentation`. Missing mode uses 60 Hz; zero dot-clock/totals use 16 ms. No source basis for a normal built-in one-second startup grace. Actual timer execution was not traced. |
| `G/window.rs:1037–1059,1978,2007–2009` | Dirty windows draw and present independently of active status. Expose or `needs_present` can present a clean scene. Active status only adds repeated presentation within one second of last input; it is **not** a first-frame requirement. |
| `G/platform/linux/x11/client.rs:906–928`; `window.rs:1290–1319` | FocusIn changes active callback/input routing. `activate()` sends a WM request **and** directly sets input focus. Adding a WM or forcing activation is not a source-supported remedy for black initial pixels. Focus readback is needed later for keyboard targeting. |
| `G/platform/blade/blade_renderer.rs:643–675,907–918`; sibling `blade-graphics-0.7.1/src/vulkan/mod.rs:481–527` | Actual drawing acquires a swapchain image, clears/renders, submits, and calls Vulkan queue_present. Observer XSync/Expose/adapter logs establish none of these stages. A black image alone cannot distinguish no presentation from a presented empty/black scene. |

`XSync` waits for the **calling connection's** X requests to be processed; it
does not wait for the independent GPUI event loop or Vulkan presentation.
Reference: [X.org XSync manual](https://www.x.org/archive/X11R7.5/doc/man/man3/XSync.3.html).
Selecting/draining observer events does not consume the app's separate event queue.

**Supported infrastructure correction:** remove the one-second-after-discovery
acceptance deadline and separate natural-map observation from a deliberately
bounded presentation grace. This corrects the measurement protocol, **not a
proven renderer defect**. There is no evidence yet that waiting longer produces
pixels. No hardware/GPU-flag/WM/compositor change is justified.

## XGetImage: the old black RGB extraction was not an alpha mistake

The reused `owned_probe.py:232–271` asks for exact-client ZPixmap, all 32 planes.
It asserts 32 bits/pixel, LSBFirst, and RGB masks `ff0000/ff00/ff`, respects
bytes_per_line, extracts bytes 2/1/0, and writes **RGB** PNG (color type 2).
For those checked masks/order this is correct: `R=(p>>16)&255`,
`G=(p>>8)&255`, `B=p&255`. The high byte is not multiplied into RGB and cannot
explain an entirely black RGB PNG by itself. The historical raw high bytes,
visual class/ID and full XImage metadata were not preserved, so do not invent
an alpha distribution or claim an independently cross-checked decoder.

Depth 32 is expected even on the depth-24 Xvfb root: GPUI `x11/window.rs:172–223,
403–411` prefers a visual with extra non-RGB bits. That does **not** mean the
client screenshot must be composited against the root. Its initial surface
config is opaque (`:654–663`); blade `vulkan/surface.rs:174–206` chooses OPAQUE
composite alpha when transparency is false. XGetImage returns client drawable
pixels, not a desktop/compositor screenshot.

For the authorized driver, retain raw client ZPixmap bytes plus dimensions,
depth, format, byte order, xoffset, stride, bpp, RGB masks and exact visual
class/ID/masks; require TrueColor and matching metadata. Decode using those
masks/order; normalize channel values to 8 bits. Check a fixed small coordinate
set against XGetPixel's normalized pixel values. Reject unsupported formats
instead of guessing, and never reinterpret RGB as premultiplied solely because
depth is 32. XRender alpha-mask information, if queried for this exact visual,
is metadata only; no compositor/root capture is authorized.

[XGetImage](https://www.x.org/archive/X11R7.5/doc/man/man3/XGetImage.3.html)
requires a viewable, on-screen drawable; obscured regions without backing store
are undefined. [XGetPixel](https://www.x.org/archive/X11R7.5/doc/man/man3/XGetPixel.3.html)
returns normalized pixel values. [XVisualInfo](https://www.x.org/archive/X11R7.5/doc/man/man3/XMatchVisualInfo.3.html)
provides exact visual class/masks. These are protocol references, not fresh runs.

## Private server setup already has one evidenced correction

The initial server-only attempt failed before any app launch because xkbcomp
was missing (`E/initial-setup-xvfb.log`). The corrected server successfully mapped
the app. Reuse the already extracted private Xvfb at
`/path/to/private-tests/opencode/regressee-ui-sweep-hkonn1qy/xvfb/`,
the private compiler `E/xkb/usr/bin/xkbcomp`, and `E/private_xkb.so` read-only.
The latter only changes Xorg's exported XkbBinDirectory (`E/private_xkb.c`).
Apply LD_PRELOAD/private library path **only to the owned server**, never the app.
Do not redownload, install system packages, change /tmp ownership or touch user
display/authentication. The corrected log's /tmp owner warning is not evidence
of this frame's cause: the server survived and mapped the client.

## ONE future native confirmation — not authorized or run

**Ready handoff required:** parent explicitly freezes the repaired source and
rebuilt binary, supplies source/binary hashes, exact PID/title expectation and
the existing ignored fixture-preparation invocation/isolated-store parameter,
and authorizes that fixture preparation plus **one** private server/app lifetime.
No source-freeze means no launch; this report itself is not permission.

1. Keep all writes under N. Copy only the handed-off repaired binary into N/bin
   and verify its hash. Use reserved physical HOME/XDG/TMP/cwd/store directories,
   fresh cookie, private Unix-only unused display, 1400×1000×24 Xvfb and the
   established lvp-only route. Collision means abort, not deleting a lock or
   moving to the user's display. No host session variables/Wayland/DBus/runtime/
   browser or GPU experiments. Hash original evidence/report before and after.
2. Only explicitly authorized fixture preparation may initialize N/store. Use
   existing `prepare_native_ui_sweep_fixture`/support fixture: synthetic loopback
   project, two saved image runs/analyses, baseline/history/mask, no browser or
   website capture. It is not a real-site claim. Inventory artifact hashes and
   project/baseline/run/analysis IDs **before** launch; never reuse E's mutable DB.
3. Launch the app once, discover only exact Popen PID + supplied title; record
   window ID, PID property, title and natural map. Subscribe to owned-client
   Structure/Visibility/Exposure/**FocusChange** events. Record event dequeue
   times and launch/map-relative clocks; inspect exact process environment/cwd/
   executable/private DB fds. No forceMap, WM startup, initial focus or input.
4. Maximum discovery/map budget: 15 s from app launch. After the first observed
   viewable map, allow **one predetermined 5 s grace**, independent of incoming
   events. Recheck ownership, 1120×800 geometry, viewability, unobscured status,
   root extent containment and no X error. Take **one** initial client image with
   metadata/raw bytes/decoder cross-check. Five seconds is a bounded diagnostic
   allowance, not a proven GPUI requirement or promise of successful pixels.
   No repeated black-frame sampling, delay extension, clear/expose injection,
   resize/focus workaround, alternate flags or relaunch on failure.
5. Require numerical nonblank gate (>8 RGB colors and <98% black), **then actual
   image review** of readable native labels/content tied to its SHA-256. Numeric
   diversity alone never grants a visual pass. Blank/unreadable means stop and
   reap: **BLOCKED, no qualified visual pass**; all screen/input checks NOT RUN.
6. Only after readable review, set focus on the owned client and read it back;
   no focus/window input outside the isolated display. Batch all five screens
   (`Projects`, `Project setup`, `Results`, `History`, `Settings`, per core
   `Screen::ALL`) at **1120×800 and 820×600**. After each navigation/resize require
   intended screen, exact geometry, containment/visibility and a bounded 1 s
   settle before its one client image. A transition failing its 3 s geometry
   budget or meaningful-pixel review ends the batch; do not rescue by retrying.
   Review all ten native images, especially Projects' Import baseline ZIP text
   and every button label, for character-column wrap, blank/full-width buttons,
   clipping/overlap, hierarchy, focus/selected/disabled state and narrow scrolling.
   Populated Results/viewer and History come from the synthetic fixture only.
7. Minimal real input within the same lifetime: Tab/Shift-Tab and Enter/Space
   activation with visible focus; type a distinct synthetic project name and
   loopback URL using key events (no clipboard), create it, edit/save a benign
   setting/name through the mounted UI, navigate away/back to confirm retained
   values. Return to the fixture for populated screens. Never invoke discovery,
   capture, login/browser or deletion/repair actions. Record native input targets,
   before/after client images and private SQL project revision/value changes.
   No second app launch or separately rerun settled regression suite is needed.
8. Bound entire server/app/controller lifetime to 15 min, including review pauses,
   with an independent exact-PID supervisor (an in-loop deadline alone cannot
   bound a blocked Xlib call); report any omitted screen/state explicitly.
   Quit through owned keyboard if
   supported, otherwise exact owned PID termination; bounded wait/reap, kill
   only exact owned PID if needed. Close X connection, stop/reap exact server,
   audit owned display lock/socket/keymap, no live owned executable PID, and
   read-only SQLite integrity/persisted create/save values after exit. Fixture
   immutable artifact/history IDs/hashes must remain unchanged except explicitly
   recorded UI edits. Local session/environment secret tables remain empty.

This batch can qualify only repaired native client layout/basic input on isolated
X11/llvmpipe with synthetic data. It cannot establish the old black-frame cause,
hardware/default-Wayland behavior, scaling/IME/accessibility, real-site capture,
human acceptance or release approval. If the delayed initial gate still fails,
handoff the preserved evidence and blocker; do not initiate a black-window loop.
