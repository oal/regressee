# Real-desktop native UI inspection — 2026-10-10

**STOPPED / BLOCKED before native mapping. No images, no whole-UI inspection and
no native visual qualification.** One newly authorized real-desktop route was
attempted, separate from prior private Xvfb/llvmpipe attempts. No UI/source edits.
The subsequent explicit stop/source-freeze-end instruction was locally observed
at **2026-10-10T18:37:40.358838Z** (Unix 1791657460.3588383). This is the local
observation timestamp, not an asserted message transmission time. All owned
processes were already gone; no launch, input or screenshot occurred after stop.

Private physical evidence root:
`/path/to/private-tests/opencode/regressee-ui-sweep-hkonn1qy/real-native-1a0d621a8cca/`.

Frozen original/copied binary SHA-256:
`663653ffa193b77e7097f78faa6ea5b9eab51cec0fff658c074305e30a60f952`.

One fresh synthetic fixture preparation passed (explicit ignored test, serial,
locked/offline). No settled suite or renderer check was rerun. The authorized
route is an exact Popen-PID/title-owned X11 client on the user's actual KWin
desktop, DISPLAY :0, with WAYLAND_DISPLAY absent and default GPU selection.
All app HOME/XDG/TMP/data/cwd roots are private; only standard absolute Xauthority
and session bus transport are borrowed. No GPU selection flags, private server,
root/desktop capture, clipboard, browser, discovery/login/repair/deletion actions.

The planned first screenshot required natural mapping plus one fixed three-second
grace, numeric nonblank evidence, and image-tool review before any input. The
application exited **before mapping**, so that grace and pixel gate were never
reached. No XGetImage, screenshot, focus/input, resize, native picker or route/GPU
fallback was attempted. No shared X server/service/socket/lock was changed.

## Actual launch failure, not a black-frame sample

`N` below abbreviates the private evidence root above.

- Controller PID **700911**, independent watchdog **700915**, app **700916**.
- Exact PID-owned client **39845889**, WM_NAME
  `Regressee — Multi-page preview`, `_NET_WM_PID=700916`.
- App launch: elapsed 0.134299 s, Unix 1791657298.7280753.
- Exact client discovered: 0.185446 s. Observed geometry 1120×800, depth 32,
  root (1362,680), border 0, override-redirect false, **map state 0**.
- DestroyNotify dequeued at 0.235633 s. App exited **101**. Controller's complete
  final evidence was written by 0.361525 s (Unix 1791657298.955302).
- `N/app.log` identifies **AMD Ryzen 9 9950X 16-Core Processor
  (RADV RAPHAEL_MENDOCINO)**, then main-thread panic at pinned
  `blade-graphics-0.7.1/src/vulkan/surface.rs:344:90`:
  `called Result::unwrap() on an Err value: ERROR_INITIALIZATION_FAILED`.
  That line is Vulkan swapchain creation, not adapter enumeration or screenshot
  decoding. There was no usable native frame to inspect.

The parent's normal Wayland `--check-renderer` success is separate initialization
evidence; it was not rerun here and does not establish X11 swapchain compatibility
or native UI quality. This failure is also distinct from the prior isolated
llvmpipe black pixels: the actual desktop hardware route died before mapping.
No cause beyond the recorded swapchain failure is invented.

## Preservation and settlement

`N/freeze.json` and before/after freeze manifests cover 68 source/Cargo/binary
inputs and original consulted reports/evidence. `N/final-audit.json`, recorded
**before the source freeze ended**, confirms all were unchanged during this
attempt. No source hashes were rechecked after stop: parent now owns new edits.

N and ancestors are physical, not symlinks, and private roots were created mode
0700. Enclosing `/home/developer/Data` mount is ext4 `/dev/nvme0n1p1`, device 66309.
`N/launch-environment.json` preserves the explicit launch environment. The app
died before post-map `/proc` environment/DB-fd verification could run; that
additional runtime-isolation readback is **not claimed**.

`N/result.json` records closure of only the controller's X connection, app reaped
with exit 101, watchdog reaped with exit 0, zero server launches and zero input.
`N/supervisor-settled.json` confirms no timeout. The shared desktop server was
never signaled; no socket/lock was deleted. Independent post-exit PID audit found
controller/watchdog/app absent, and the explicit-stop status check at the
timestamp above again found all three absent. No outstanding owned window/process
needed closing after that instruction.

`N/before-launch-artifacts.json`, `N/after-exit-artifacts.json`, before/after SQL
snapshots and `N/final-audit.json` confirm:

- Synthetic artifact hashes/bytes, database-file hash, project/config/revision and
  table counts unchanged; SQLite integrity `ok`.
- One loopback synthetic project with one baseline, two runs/analyses, inventory
  and mask; no real website capture. Local session/environment secret tables empty.
- No screenshots created. Original user data/project/window were not accessed or
  modified. Only authorized display/authentication/session-bus transport was used.

## Requested visual scope: unperformed, not silently passed

| Scope | Result |
| --- | --- |
| Actual owned real-desktop hardware app launch | One attempted; **FAIL before mapping** |
| Projects/Setup/Results/History/Settings at 1120×800 and 820×600 | **NOT RUN** |
| Lower Setup/Settings/Results sections; advanced/login/environment/filter/viewer/mask states | **NOT RUN** |
| Horizontal Import label, focused geometry, radios/checks/errors/disabled states | **NOT RUN** |
| Real keyboard create/save/project reopen | **NOT RUN** |
| Readable actual UI screenshots to give parent | **NONE**; no images manufactured/substituted |
| Private evidence retention, pre-stop source preservation and owned process settlement | Audited |
| Whole UI quality, fix, release, hardware presentation/Wayland qualification | **NOT QUALIFIED** |

Parent's newly supplied user screenshots (detached Dismiss, radio spacing,
timezone input, missing automatic runtime provisioning, and ambiguous wide
buttons) are incumbent user evidence, **not findings from this failed native
inspection**. They were not captured or independently viewed in this subagent
session. No prior GPUI test bounds or fixture images substitute for them.

All existing failure evidence remains under N. There is no live controller to
resume and no further launch/input/capture permission inferred from the ended
freeze. Parent can proceed with the requested source edits and arrange a separate
post-edit native batch; this stopped attempt cannot supply a visual PASS.
