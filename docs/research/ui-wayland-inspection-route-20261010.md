# Wayland whole-UI inspection route preparation — 2026-10-10

**PREPARATION ONLY. Exact-window capture API exists, but the prepared autonomous
whole-UI route is currently BLOCKED by unavailable PID/UUID discovery and safe
input. No app, window, service, fixture, build, test, screenshot or input was
launched/performed. No UI/source edits and no native visual qualification.**

Owned mode-0700 physical scratch (**P**):
`/path/to/private-tests/opencode/regressee-ui-sweep-hkonn1qy/wayland-route-f3e1aee0757f/`

Only this report and P's scripts/metadata/evidence were created. Earlier failed
routes/reports are untouched. Parent source is not frozen during this phase;
no waiting, polling or launch permission is inferred. All preparation commands
completed in the foreground; no owned probe/application processes remain.

## Verified live metadata — no window enumeration/content

Used **`/usr/bin/python3`**, not the default Python. `gi`, `dbus`, `PIL` and
`libwayland-client.so.0` are available. One read-only Wayland connection sent only
`wl_display.get_registry` and `sync`, then closed. It bound **no** plasma-window,
fake-input, seat, output, surface or content interface. One session-bus connection
used GetNameOwner, Introspect and Screenshot2 Version Get with **NO_AUTO_START**.
No `getWindowInfo`, interactive selector, capture, authentication or activation
request was made. No window UUIDs/PIDs/captions/URLs were enumerated or retained.

Evidence: `P/metadata/live-metadata.json`, observed Unix **1791658349.5126958**,
and `live-org.kde.KWin*.xml`.

| Capability | Live result / implication |
| --- | --- |
| `org.kde.KWin.ScreenShot2` | Existing owner `:1.22`; **Version 5**; `CaptureWindow(s handle, a{sv} options, h pipe) -> a{sv}`. Transport/signature confirmed, **not capture permission/pixels**. |
| `org.kde.KWin` | Same existing owner; `/KWin.getWindowInfo(s uuid) -> a{sv}`. No PID lookup/window-list/resize/active-window getter appears in the live interface. `queryWindowInfo` is interactive and was not invoked. |
| `org_kde_plasma_window_management` | **Not offered to this actual metadata client.** Consequently the PID-first UUID/owned-active-state discovery path cannot currently bind. |
| `org_kde_kwin_fake_input` | **Not offered.** No authentication/injection attempted. |
| Other toplevel discovery | No `ext_foreign_toplevel_list_v1` or `zwlr_foreign_toplevel_manager_v1` in this client's offered globals. No alternate-title/window probing performed. |

This resolves the parent's earlier “likely available” registry uncertainty: it
is **not available on this connection**. It does not prove those globals are
absent for every compositor client, nor identify the installed policy/config
reason. No different executable identity, desktop-file spoof, privileged client,
sandbox escape, interface-ID guessing or compositor configuration change is
proposed to obtain them.

## Official source evidence and qualification

P/metadata contains HTTPS snapshots from official KDE/Qt repositories, with URL,
retrieval time, byte size and SHA-256 in `sources.json`. These are **upstream
master/dev snapshots**, not asserted identical to the installed compositor.
Live introspection/version/global visibility is the current-host evidence.

- [Screenshot2 implementation](https://raw.githubusercontent.com/KDE/kwin/master/src/plugins/screenshot/screenshotdbusinterface2.cpp),
  saved `:264–284`: CaptureWindow resolves **QUuid(handle)**, duplicates the
  caller's FD, captures that named window and sends a delayed reply. No
  `checkPermission`/desktop-file authorization occurs in this method in this
  snapshot. That does **not** prove all installed D-Bus/compositor policy grants
  access. Any actual Denied/InvalidWindow/Cancelled/protocol error must stop;
  do not apply old-version assumptions or bypass a denial.
- Same file `:195–200,456–463`: reply contains `type=raw`, Qt QImage `format`,
  width, height, stride, scale and **windowId** equal to internal UUID. The FD
  contains raw `QImage::constBits()` bytes, **not PNG**. Reply precedes background
  pipe writing; receiver must consume the pipe with a deadline/size cap.
- [Named-window render path](https://raw.githubusercontent.com/KDE/kwin/master/src/plugins/screenshot/screenshot.cpp)
  `:192–246`: renders only the requested window item to an offscreen target,
  excludes decoration/shadow/cursor according to options, reads RGBA bytes and
  returns **RGBA8888_Premultiplied**. Window scale is targetScale; setting
  native-resolution false does not promise scale 1 in this path. Do not silently
  rescale to make the screenshot dimensions look like logical client dimensions.
- [Plasma protocol XML](https://raw.githubusercontent.com/KDE/plasma-wayland-protocols/master/src/protocols/plasma-window-management.xml)
  specifies UUID window events, per-window PID, initial_state, state ACTIVE=1,
  client_geometry since 18, mapped since 21, and per-window `set_state(flags,state)`.
  It warns that this is desktop-private and regular clients must not assume it
  is available. No guessed private request or interface version is allowed.
- [Plasma implementation](https://raw.githubusercontent.com/KDE/kwin/master/src/wayland/plasmawindowmanagement.cpp)
  `:374–428` sends PID before title in initial state. This supports **PID-first
  filtering**, not title-only ownership. Prime a legitimate management binding
  before future app spawn and discard all initial existing UUID announcements
  without requesting their window resources. Only subsequent candidate PID
  matches may have caption/geometry decoded or exact `getWindowInfo` called.
  Unmatched queued title/app-ID/menu events must be discarded without decoding
  or logging. Binding is **not performed in preparation** and is currently gated
  off by the live missing global.
- [KWin D-Bus implementation](https://raw.githubusercontent.com/KDE/kwin/master/src/dbusinterface.cpp)
  `:118–185`: getWindowInfo looks up one UUID; queryWindowInfo opens an interactive
  selector. The upstream returned map includes PID/caption/UUID/geometry but
  **no active-focus field**. Live introspection leaves the map keys unspecified;
  no live window map was read. Therefore do not infer focus from this method.
- [Fake-input XML](https://raw.githubusercontent.com/KDE/plasma-wayland-protocols/master/src/protocols/fake-input.xml)
  gives **no target UUID/surface** on pointer/button/axis/key requests and no
  authentication-granted event. [Its backend](https://raw.githubusercontent.com/KDE/kwin/master/src/backends/fakeinput/fakeinputbackend.cpp)
  routes device events globally. Activation/readback on another protocol or
  connection is **not an atomic “deliver only to this window” guard**. The
  keysym implementation can even adjust/reconfigure the global keymap; avoid it.
  Even if fake_input is later offered, no unsafe synthetic event may be used as
  a permission probe. Honest authenticate identity/reason would be required for
  a separately approved safe path, never masquerading as KDE/Spectacle.

**Safety result:** own ACTIVE state could acknowledge an allowed activation
request, but focus can change between readback and global injection; pointer
focus is not proven by keyboard ACTIVE. Neither the missing interface nor this
race is solved by repeating getWindowInfo. Prepared code contains **no fake-input
binding/authentication/injection implementation**.

## Prepared scripts — dormant capture/discovery, not a launcher

All paths relative to P; no application controller or launcher was created.

- `scripts/probe_metadata_only.py`: the **one executed** metadata probe above.
- `scripts/wayland_owned_metadata.py`: bounded native Wayland wire framing,
  read-only registry client and **future** XML-opcode/PID-first discovery. Direct
  wire framing avoids compiling a generated client or unsafe ctypes callback
  string decoding. OwnedLease requires explicit future freeze authorization,
  exact copied executable/private cwd, PID/start ticks and executable hash.
  Missing management global fails **before any window-resource request**.
  Future activation is per owned resource only, after readable-image review;
  it never enables global input. No discovery/activation method was executed.
- `scripts/kwin_owned_capture.py`: **future** exact-UUID-only Gio UnixFDList
  capture/strict raw QImage decoder. No active-window/area/screen/workspace
  fallback. Requires the PID-discovery proof and frozen owned lease, verifies
  PID/caption/UUID/process/service owner before/after capture and returned
  windowId, client dimensions and scale. Separate 3 s D-Bus and 5 s pipe
  deadlines, 64 MiB cap, no overwrite/retry. Uses pipe FD index (`h`) via
  UnixFDList, not the numeric process FD as a D-Bus handle.
  Keeps raw bytes, metadata, PNG hashes and RGB/alpha statistics. Qt enum support
  is explicit (RGB32/ARGB32/premultiplied, RGB888, RGBX/RGBA8888/premultiplied),
  row stride/byte order honored; premultiplied channels are unpremultiplied,
  not multiplied again. Unsupported formats fail, with no guessed color order.
  No tint, resizing or desktop background composition. **Capture/decoder never
  executed or tested on a real image**, and permission remains unverified.

Source provenance for enum values:
[Qt qimage.h](https://raw.githubusercontent.com/qt/qtbase/dev/src/gui/image/qimage.h).
Only AST syntax inspection is appropriate now; it is not a native/UI test or
qualification. Prepared scripts must not be promoted to a working route merely
because their declarations parse.

## Future bounded round — authorization and blockers must come first

**Do not launch on receipt of this report.** Parent must finish edits, rebuild,
freeze source/binary hashes, explicitly authorize the one normal-Wayland round,
and resolve exact PID/UUID discovery plus a safe navigation/resize plan. Current
prepared code cannot obtain a trustworthy UUID on the observed connection.
An independently legitimate PID-proven owned UUID source or genuine supported
permission grant may be investigated separately, not manufactured by identity
spoofing. No UUID guesses or probing unrelated windows.

If those prerequisites are met:

1. Fresh physical private HOME/XDG data/cache/config/state/TMP/cwd/store; one
   ignored synthetic fixture preparation only after authorization. Preserve
   the **valid shared XDG_RUNTIME_DIR**, WAYLAND_DISPLAY and session bus solely
   for native transport. No private fake Wayland socket, X11 fallback, renderer
   check, GPU flags, runtime/browser downloads or user data. Inventory fixture
   bytes/IDs and source/original/copied binary hashes before launch.
2. One normal Wayland app lifetime, actual default hardware; record exact owned
   PID/start time/executable/private DB use and PID-proven UUID/caption. Natural
   mapping and one fixed 3 s grace; one CaptureWindow with decoration/shadow/
   cursor false. Raw metadata/UUID/size/scale checks, numerical nonblank gate,
   then **functions.read of the actual PNG** before any action. Error/black/
   unreadable means stop; no GPU/delay/capture-method/identity retry.
3. **No autonomous synthetic input under the current evidence.** Parent could
   arrange explicit user-operated navigation/resizing of only the owned window;
   capture remains exact UUID and verifies PID/caption/state/geometry each time.
   This requires a new concrete handoff, not an implicit request to touch the
   user's existing window. Plasma request_resize is interactive, not an exact
   size setter; the live D-Bus interface has no exact window-size setter.
4. After a safe mechanism is authorized, batch Projects, Setup, Results, History,
   Settings at logical **1120×800 and 820×600**, plus lower sections, advanced/
   environment/login choices without browser launch, timezone dropdown, saved
   fixture viewer/filter/analysis/mask/history, focused/selected/disabled/error
   states and ordinary empty/new project. Keep raw physical dimensions/scale;
   review every representative native image, not only the first viewport.
   Real create/save/project reopen requires that same safe input mechanism;
   SQL persistence alone is not a substitute.
5. Independent exact-owned-PID 15-minute watchdog. Close controller metadata/
   D-Bus connections, terminate/reap only the owned app/controller/watchdog;
   never signal KWin/shared display/services or delete shared locks/sockets.
   After exit, audit source freeze, original fixture artifact retention, synthetic
   DB integrity/persisted values/privacy and absence of owned processes.

The ceiling is one whole-UI inspection batch and at most one later correction
confirmation if warranted—not a launch/capture/polishing loop. Every omitted
screen/input state must be labeled NOT RUN. Currently **the next autonomous
batch remains blocked**, with no readable pixels, no UI quality PASS and no
release/human/cross-OS qualification invented.
