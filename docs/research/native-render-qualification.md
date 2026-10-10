# Bounded native-render diagnostic — 2026-10-10

**Verdict: isolated native rendering/input NOT RUN; native qualification remains
open/failed on historical evidence.** This investigation cannot explain the
previous black window conclusively. No valid fresh pixels, app input, visual
review, human approval or Phase 7 completion is claimed.

Independent GPT-6.1 Sol diagnostic while another agent implements Phase 6. The
only repository write is this report. No implementation, design, status,
manifest, lock or history changes; no builds, test suites, browser/worker
launches, installs, sudo, host graphics changes or nested agents.

## 1. Isolation preflight and stop decision

Approved Python/ctypes X11 and XTest controls are available (`libX11.so.6`,
`libXtst.so.6`), and `/usr/share/vulkan/icd.d/lvp_icd.json` exists. Its metadata
advertises API `1.4.335`, library `libvulkan_lvp.so`; that is **not** a loaded-device
or Vulkan presentation test. `vulkaninfo` is unavailable.

No `Xvfb`, `Xorg`, `Xephyr`, `Xwayland`, Xvnc variants, Weston, Sway, Cage,
Wayfire, labwc, Mutter/GNOME Shell, KWin Wayland, gamescope or xpra was found on
PATH. Bounded filename inspections of `/usr/bin`, `/usr/sbin`, `/usr/libexec`,
`/usr/lib/xorg`, `/usr/local/bin`, `/app/bin` found no available candidate.
Read-only `flatpak list/info` and corresponding `files/bin`, `files/libexec`,
`files/lib/xorg` inspections found no candidate in the installed Freedesktop
SDKs 25.08/26.08, GNOME Platform 50 or Fedora KDE6Platform f44. This is not an
exhaustive search of every host file, nor permission to install missing tools or
enter the host desktop through another namespace.

**No owned display could be established, so app startup was aborted before any
launch or XOpenDisplay call.** The shared display was never opened. Removing
DISPLAY/Wayland variables and calling GPUI's headless client would not produce
the requested native window/presentation evidence; it was not used as a substitute.
There was one batched preflight, **zero app attempts and zero confirmations**;
no repeated black-window loop.

| Requested observation | Fresh result |
| --- | --- |
| Stable isolated copy of existing product UI binary | PASS, matching SHA-256 before/after copy |
| Fresh owned data/cache/HOME/XDG/cwd/temp roots | PASS preparation; all remain empty |
| Owned virtual display/compositor | Unavailable in inspected locations |
| Runtime proof of isolated app DB/cache use | NOT RUN; app never started |
| Title + PID ownership, mapping/visibility/exposure timestamps | NOT RUN |
| Actual owned-window and owned-root PNGs, frame timing | NOT RUN; zero screenshots |
| Native input changes UI / persisted project state | NOT RUN; zero events |
| Default hardware/shared-host, Windows/macOS, human approval | NOT RUN |

## 2. Preserved local evidence and exact commands

All fresh outputs live only in:

`/tmp/opencode/native-diagnostic-20261010-sol-9d6af1c2/`

The driver, `preflight.py`, is a single-use **metadata preflight**, not a runnable
native acceptance harness. It refuses an existing result; preserve it and use a
new output directory for future work. The driver includes the exact filesystem
inspection and binary copy/hash operations. The command was:

```sh
python3 /tmp/opencode/native-diagnostic-20261010-sol-9d6af1c2/preflight.py
```

`result.json` records each subprocess argv, exit code and retained stdout/stderr:

```text
flatpak list --runtime --columns=application,branch,installation
flatpak info --show-location org.freedesktop.Sdk//25.08
flatpak info --show-location org.freedesktop.Sdk//26.08
flatpak info --show-location org.gnome.Platform//50
flatpak info --show-location org.fedoraproject.KDE6Platform//f44
file /tmp/opencode/native-diagnostic-20261010-sol-9d6af1c2/bin/regressee
readelf -h /tmp/opencode/native-diagnostic-20261010-sol-9d6af1c2/bin/regressee
readelf -n /tmp/opencode/native-diagnostic-20261010-sol-9d6af1c2/bin/regressee
readelf -d /tmp/opencode/native-diagnostic-20261010-sol-9d6af1c2/bin/regressee
readelf --version-info /tmp/opencode/native-diagnostic-20261010-sol-9d6af1c2/bin/regressee
```

All those metadata commands exited 0; **driver exit 0 is not a native test pass**.
Earlier inline availability checks performed the same bounded tool/location
inspection; none launched an app, display, Flatpak sandbox or browser.

Frozen source: `target/debug/regressee`, **128,903,960 bytes**, SHA-256:

`97a14c2ca223f4ee5d2cac0f17d64ba536b1fc0e9b7ad1bd254e2b039d74d709`

Copy: `bin/regressee` under the evidence directory; source inode/size/mtime were
stable during copy and both hashes matched. ELF x86-64 PIE, unstripped, loader
`/lib64/ld-linux-x86-64.so.2`, build ID
`72500102b72ba3e247606d842d4c354f0a944d86`. Embedded strings include
`REGRESSEE_DEV_DATA`, `REGRESSEE_DISTRIBUTION` and
`Regressee — Multi-page preview`. This is an already-built snapshot, **not proof
that it matches concurrently changing source or completes Phase 6**.

Direct ELF requirements include libxcb, libxkbcommon, libxkbcommon-x11, libgcc_s,
libm, libc and the loader; there is no recorded RPATH/RUNPATH. Dynamically loaded
Vulkan/font/display dependencies are not exhaustively represented by DT_NEEDED.
**Concrete packaging warning:** the version-needs table requires
`GLIBC_2.43` from libm (and `GLIBC_2.39` from libc). Do not move this development
binary into an older-glibc clean target or the pinned SDK and count a loader
failure as a GPUI black-window reproduction. Build a qualification candidate
against the intended minimum userspace and inspect its ELF requirements first.
The Linux 3.2 ABI note is not a glibc compatibility claim. This does not explain
the older shared-display black captures, whose apps did reach window creation.

`planned-environment.json` records an **unused** minimal environment: private
HOME, XDG data/cache/config/state/runtime, TMPDIR/TMP/TEMP, explicit
REGRESSEE_DEV_DATA and the lavapipe ICD. No DISPLAY was assigned, since no display
was owned. A future launch must add only an owned display/authentication path and,
if needed, an owned session bus; never inherit WAYLAND_DISPLAY, ZED_HEADLESS,
REGRESSEE_DISTRIBUTION, test-runtime overrides, host bus addresses or host data
paths. No security/sandbox flags were used.

## 3. Source-backed hypotheses, not reproduced causes

Read README, `00-native-recheck-2026-10-09.md`, `00-reverification.md`, the Phase 2
native JSON, Linux packaging decision and release-qualification research. Read
native launch/data/product/job sources and cached primary GPUI 0.2.2 / Blade 0.7.1
backend code. Relevant file hashes are retained in `result.json`.

1. **Mapping/exposure/frame scheduling on the shared desktop remains plausible.**
   GPUI `src/platform/linux/x11/window.rs:1372–1377` submits ordinary MapWindow.
   In `x11/client.rs:740–759`, MapNotify/UnmapNotify/VisibilityNotify update state.
   At `1702–1774`, periodic refresh requires mapped and not fully obscured;
   RandR resource-query failure returns without starting that initial timer,
   whereas missing mode info after a successful query falls back to 60 Hz.
   Expose is collected at `538–539,612–615`; the timer passes it as
   `require_presentation` at `1792–1799`. A single XGetWindowAttributes sample
   cannot prove those application-side events or a submitted/presented frame.
   The previous diagnostic's discovery-time map_state=0 omitted a grace period;
   the ordinary harness did wait and acquired an entirely black viewable window.

2. **Vulkan WSI/presentation or readback remains plausible, not established.**
   Blade `src/vulkan/surface.rs:58–95` creates a platform surface and explicitly
   checks device queue presentation support. Lavapipe availability alone does not
   establish successful swapchain acquisition/submission/presentation to X11.
   Compare actual window pixels with the same owned root's pixels at that window's
   translated bounds, after mapping/exposure grace, before attributing black
   XGetImage output to app layout. Neither path was exercised here.

3. **Input/focus was never sufficiently separated from render failure.**
   Historical XTest dispatch with no enqueue marker is not proof of a broken
   app action or a valid focused window. Read back focus before each event batch;
   require a visible semantic state change and, for save, a fresh isolated DB
   record. Do not send input when meaningful pixels are unavailable.

4. **Harness drift is a separate, confirmed source mismatch.**
   `scripts/phase2-native.py:33` expects `Regressee — Single-page preview`;
   current `crates/app/src/main.rs` and this binary contain
   `Regressee — Multi-page preview`. Reusing that title selector would miss this
   product window. `ui-smoke.py` instead launches the Phase 0 bench, writes shared
   artifact names and can start its fixture runtime; it is inappropriate for this
   isolated foundation test. Neither script was executed or modified. Title
   drift does **not** explain the previously PID-selected black Phase 0 image.

5. **Empty app logs do not rule out backend diagnostics.** The inspected app
   source has no logger initialization, while GPUI/Blade use `log` calls. Merely
   setting RUST_LOG is not guaranteed to expose those messages. If a properly
   isolated reproduction still fails, propose narrowly scoped event/frame/log
   instrumentation to the implementation owner; do not retrofit it in this task.

GPUI `src/platform.rs:127–150` selects Wayland before X11, and ZED_HEADLESS before
either. Backend selection therefore requires a clean allowlist environment, not
just changing DISPLAY in an inherited shell.

## 4. Bounded local path recommended for Phase 7

These are **future recommendations, not commands run or implementation changes**.

1. Use an administrator-provided diagnostic machine/image with an existing
   authenticated virtual X11 server and a normal window manager, software Vulkan,
   RandR and XTest. Do not install tools globally in this shared session or loosen
   access/sandbox policy. Select an unoccupied display atomically, verify its
   process/authentication ownership, and never use the inherited desktop socket.
2. Freeze/hash a built product binary before concurrent implementation builds.
   Create a new private evidence directory and fresh HOME/XDG/app/cwd/temp roots.
   Set REGRESSEE_DEV_DATA explicitly. Source `AppPaths::isolated` maps to
   `<root>/data` and `<root>/cache`; ordinary discovery otherwise uses OS locations.
   Jobs opens Store on those paths, and Product's unconfigured distribution fails
   closed. Verify actual process environment/open DB paths after launch; abort if
   isolation cannot be confirmed. No worker/runtime distribution is needed.
3. **One batch, ≤30 seconds for discovery/mapping/paint prerequisites.** Select
   only exact product title plus `_NET_WM_PID` matching the copied process. Sample
   geometry/map/visibility/focus and observe Map/Expose events with monotonic
   timestamps. Use ordinary mapping, no override-redirect or synthetic MapWindow.
   Capture owned window and owned root with approved Python XGetImage controls at
   fixed post-map offsets (e.g. 0.5, 2 and 5 seconds). Record hashes, dimensions,
   PNG CRCs and pixel distributions; inspect readable Projects/Setup content.
   Pixel sampling measures observed paint timing, **not GPU frame latency**.
4. Only after valid paint: read back focus, XTest Ctrl+N to Create project,
   type a unique synthetic name and a reserved owned-loopback address without
   clipboard use, then Ctrl+S. Saving alone must not Find pages/capture/install.
   Capture Setup and Projects after input; verify the new record only in the fresh
   DB after clean Ctrl+Q. Semantic content/persistence, not dispatch success or
   mere PNG byte difference, establishes app-state change. Test keyboard quit and
   account for only owned app/display/WM/bus processes. No real project/session
   data or managed browser is involved.
5. Stop on black/unmapped output and retain the failed evidence. At most **one
   confirmation** is appropriate after a concrete, isolated cause is identified;
   no cosmetic retry loop. An unavailable diagnostic image remains NOT RUN.
6. A virtual/software pass would isolate one path only. Then separately qualify
   ordinary Linux default Wayland/X11 and hardware drivers on supported clean
   targets, the actual package and libraries, Windows and macOS native desktops,
   accessibility and human review. Follow decision 0003's provisional AppImage
   path; do not treat its historical Flatpak sandbox failure as justification for
   overrides. Keep all these gates explicit and unclaimed.

## 5. Resource and preservation accounting

No app/display/compositor/session-bus child was started, so there are **no owned
live resources to terminate** and no kill operations. Metadata subprocesses were
synchronous and exited. Empty private roots, the inert binary copy, driver,
planned environment and metadata/results remain in the unique temporary directory.
No unrelated windows, focus, clipboard, settings or shared runtime/data were
accessed or altered. Historical reports/artifacts were read only, never replaced
or recursively copied. This report does not override their failures or certify
the concurrent Phase 6 implementation.
