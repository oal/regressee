# Native owned-window qualification on disk — 2026-10-10

**FINAL: native startup/rendering FAIL on the current Phase 7 binary.** One actual
owned-window launch crashed during Vulkan swapchain creation, before mapping or
pixels. **Input/create/save/reopen NOT RUN; zero screenshots and zero input
events. Cleanup PASS.** This is not a black-pixel reproduction, a human approval
or a Phase 7 release pass. Earlier checkpoints below preserve the draft sequence.

Independent bounded GPT-6.1 Sol attempt during Phase 7 verification. Only this
repository report and a unique physical disk evidence root are owned. No shared
source, Cargo, lock, design, status or phase-report edits; no build, runtime,
browser, worker or soak suites; no nested agents.

## Frozen candidate and evidence root

Root: `/path/to/private-tests/native-owned-window-20261010-sol-bd04e44032/`

Copied already-built `target/debug/regressee` into `bin/regressee`, 131,117,928
bytes. Source inode/size/mtime stayed stable through copy; source/copy SHA-256:

`06daab40656e04a64c3ec692bcf46cb1be6630ec0fa90259a5d3cc0fe6e0de2b`

This equals the binary recorded in Phase 7's final private candidate; no rebuild
or broader source/clean-machine qualification is inferred. `freeze.json` records
the snapshot. The root and its ancestors were checked for symlinks; it is the
physical disk path, not RAM-backed `/tmp` scratch. Fresh private app, HOME, XDG
data/cache/config/state/runtime, TMPDIR and cwd roots have been created.

## Planned bounded stages

1. Read native history, Phase 7 status, README and actual startup/data/UI source
   read-only. Verify explicit data isolation before any app launch.
2. Check installed isolated displays and current available/cgroup memory. Prefer
   an installed owned display; otherwise use the explicitly approved shared
   DISPLAY only for discovering/control/capture of the exact app PID/title.
   Abort rather than contend if the fresh reserve is insufficient.
3. One native batch using the copied binary and a clean environment allowlist.
   Record actual PID/environment/open paths, map/exposure timestamps, geometry,
   backend environment and cropped owned-window PNG pixels at fixed offsets.
   Never root-screenshot, force-map, inspect unrelated window content or touch
   clipboard, host settings or security/sandbox policy.
4. Only if actual visible readable Projects/Setup render: focus/read back the
   owned window, perform actual bounded click/key create/save/reopen actions and
   verify authoritative fresh SQLite state. No configured worker is required.
5. Stop if black/unmapped/crashed; bounded read-only diagnostics, not a polish
   loop. At most one confirmation only after a concrete owned-setup fix. Settle
   all owned processes and preserve fresh data/images/diagnostics. Finalize exact
   PASS/FAIL/NOT RUN scopes, input/action proof and resource audit.

Checkpoint 0 (DRAFT, before any launch): MemAvailable 43,043,796 KiB (~41 GiB), cgroup
`memory.max=max`, current 815,890,432 bytes, OOM counters zero. Swap is low;
immediately-before-launch reserve will be rechecked. Xvfb/Xephyr/Weston/Cage are
not on PATH; shared DISPLAY is `:0`. No display connection/input/capture has yet
occurred. No ordinary hardware, cross-OS, accessibility, human visual/usability
approval or Phase 7 release pass is implied by this attempt.

## Checkpoint 1 — startup isolation verified, ready for the one batch

Source-backed startup is **zero CLI arguments** and explicit environment
`REGRESSEE_DEV_DATA=<root>/app` (`main.rs:4281–4288`). `AppPaths::isolated`
(`storage/src/lib.rs:143–152`) maps this to `<root>/app/data`,
`<root>/app/cache` and `data/regressee.sqlite3`. Jobs opens that Store on its
background thread (`jobs.rs:255–264`). `Product::configured` has no distribution
when `REGRESSEE_DISTRIBUTION` is absent (`product.rs:26–44`); project list/save/load
does not require or launch a worker. Exact window title is
`Regressee — Multi-page preview` (`main.rs:4345–4347`).

The bounded filename/PATH inspection found no Xvfb/Xorg/Xephyr/Xwayland/Xvnc/
Weston/Sway/Cage/Wayfire/labwc/Mutter/GNOME Shell/KWin/gamescope/xpra executable
in the inspected locations. This is missing isolated-display tooling, not a
render failure. X11/XTest libraries load. `findmnt` confirms the owned physical
root is on `/home/developer/Data`, `/dev/nvme0n1p1`, **ext4**.

The one batch will use approved **shared `DISPLAY=:0`, forced X11, default GPUI
Vulkan selection**, not the historical forced lavapipe setup. No Vulkan/software
override is passed, so the actual hardware device is not asserted without proof.
Wayland/headless/session-bus/distribution/test overrides are not inherited.
The only borrowed host path is read-only XAUTHORITY for approved X11 transport;
its contents are neither copied nor reported. All application roots/cwd/temp are
private and fresh. `owned_probe.py` and `preflight.json` preserve the driver and
read-only source hashes. The driver refuses reuse and requires ≥8 GiB available
host memory and ≥2 GiB headroom in any finite own cgroup limit at launch.

No app launch, screenshot or input has occurred as of this checkpoint.

## Checkpoint 2 — one actual launch settled, startup failure

The one batch ran `python3 <root>/owned_probe.py`. Copied app PID **346119**,
owned X11 window **46137345**, exact current title and `_NET_WM_PID=346119`.
Discovery occurred 0.202 s after launch. Two observed samples were unmapped
(`map_state=0`), client **1120 × 800**, root origin **(1362,680)**, depth 32,
border 0, `override_redirect=false`. No forced mapping or input was performed.

App exited **101** while the driver was waiting for ordinary mapping/frame grace;
total driver elapsed ~0.426 s. `app.log` reports Blade's
`src/vulkan/surface.rs:344:90` panic, `ERROR_INITIALIZATION_FAILED`.
No Map/Expose/Visibility events were observed before exit. Zero screenshots and
zero focus/click/key events: pixels never became safely available. This is a fresh
startup/presentation failure, **not an observed black PNG** or persistent full
10-second mapping-grace result. Final fresh SQLite integrity is `ok`, all project/
run/artifact/session counts zero. Initial live-process environment/fd inspection
was not reached, so that runtime isolation subcheck is not claimed as PASS.

The app was already exited and was reaped; the X11 connection closed. No cleanup
signal was necessary. The driver is settled, with no exact-owned-binary live PID.
No confirmation is planned: no verified owned-test-setup fix has been identified.
The bounded failure-site and final resource/source-preservation audits below are
now complete; there are no outstanding app/control tasks.

## Final qualification scope and concrete failure

| Subcheck | Actual verdict / evidence |
| --- | --- |
| Stable physical-disk binary snapshot | PASS; frozen/copy/current source binary hashes match |
| Installed isolated display | NOT RUN; tooling unavailable in inspected locations |
| Approved shared-X11 native app launch | Executed once, PID 346119; no software override |
| Exact title/PID window discovery | PASS; window 46137345, matching title/PID |
| Visible mapped rendering/presentation | **FAIL**; startup panic before viewable state |
| Actual owned-window pixel screenshots | NOT RUN after crash; **0 PNGs**, not invented black-pixel evidence |
| Frame/map/exposure evidence | Two unmapped samples; no observed Map/Expose/Visibility events, no measured GPU frame |
| Actual focus/click/key action proof | NOT RUN; **0 events**; no mock or dispatch-only proof |
| Fresh native project create/save/reopen | NOT RUN after rendering prerequisite failure |
| Authoritative SQLite project persistence | NOT RUN for project action; initialized DB is empty and integrity `ok` |
| Actual app environment/fd readback | NOT RUN; app crashed before this scheduled subcheck |
| Owned cleanup / runtime scratch | PASS; app/driver settled, no worker/browser/runtime or transfer scratch |
| Hardware/default Wayland/cross-OS/accessibility/human/release | NOT RUN / unclaimed |

The concrete error is not a missing X11 tool or an older-host loader failure:

```text
thread 'main' (346119) panicked at .../blade-graphics-0.7.1/src/vulkan/surface.rs:344:90:
called `Result::unwrap()` on an `Err` value: ERROR_INITIALIZATION_FAILED
```

Read-only cached source confirms line 344 is
`surface.device.create_swapchain(&create_info, None).unwrap()`.
`failure-source-excerpt.txt` preserves lines 316–355 and `final-audit.json`
binds the dependency file hash. Thus the failure is **Vulkan swapchain creation**;
no rendered frame or presented Projects UI was reached. This does not identify
the underlying driver/compositor/configuration cause, prove every default Linux
backend fails, or explain the historical lavapipe black PNGs. No instrumentation,
host display modification or speculative backend retry was made. **One attempt,
zero confirmations**; no verified private setup fix justified a confirmation.

Exact child argv was solely `<root>/bin/regressee`, cwd `<root>/cwd`, with
the environment preserved in `launch-environment.json`. App/HOME/XDG/cache/tmp
paths are private; only DISPLAY and read-only XAUTHORITY transport refer to the
approved shared display. No WAYLAND_DISPLAY, ZED_HEADLESS, REGRESSEE_DISTRIBUTION,
DBUS_SESSION_BUS_ADDRESS, VK_ICD_FILENAMES, LIBGL_ALWAYS_SOFTWARE or GALLIUM_DRIVER
was inherited. This is **forced shared X11 with default Vulkan selection**, not
ordinary default-Wayland qualification, isolated virtual display or forced
software rendering. Fresh `mesa_shader_cache` / `radv_builtin_shaders` files
suggest RADV involvement but do not prove a particular physical GPU selection;
maps/device diagnostics were not reached before the crash.

Host `getconf GNU_LIBC_VERSION` reports **glibc 2.43**; ELF metadata requires
**GLIBC_2.43**. The binary did execute here. Neither observation qualifies older
Linux, Ubuntu 24.04, a packaged candidate or a clean machine.

## Final evidence, persistence and resource audit

All outputs remain in the physical ext4 root above. Important files:

- `freeze.json`, `preflight.json`, `launch-environment.json`, `owned_probe.py`;
- `attempt-marker.json`, `checkpoint.json`, `result.json`, `app.log`;
- `final-after-exit-sqlite.json`, actual fresh `app/data/regressee.sqlite3`;
- `failure-source-excerpt.txt`, `final-audit.json`, `settled-resource-audit.json`;
- read-only `metadata-{findmnt,getconf,readelf}.txt`.

`result.json` SHA-256:
`c3bb86a937e6bcbfb4f400e306685945b5dc140ef57cbc8a08680406f24e755f`.
`app.log` SHA-256:
`bb0e36a525054a0c897820bbf77a38deffbcce43e1ff0bdb28df2f3410e32baf`.
The Python driver's metadata completion/exit is **not a native PASS**: its saved
outcome is FAIL and the actual native app exit is 101.

No before/after UI/action screenshot exists. The initialized SQLite has **zero
projects, runs, captures/artifact records, sessions, baselines and pending GC**;
the authoritative read-only table audit and integrity check are retained. DB
file is 139,264 bytes, mode 0600, SHA-256
`1ed136508d80c77d65cda5ef7ef91ba8654d87786d8f6472147c27a0e1d8154f`.
Empty WAL and a 32,768-byte SHM are retained; inspecting the fresh DB may create
SQLite's own sidecars, not project-action proof. There was no pre-input live
database snapshot because mapping failed before that stage.

Immediately before launch: MemAvailable **42,908,012 KiB (~40.9 GiB)**; immediately
after cleanup **42,921,856 KiB**. Visible cgroup limit was `max`, sampled current
~813 MB, OOM/OOM-kill counters zero before and after. The namespace's
`/proc/self/cgroup` includes `../` components whose ancestor paths are unavailable
at this mount; `result.json` preserves those ENOENTs. Therefore hidden ancestor
limits are not proven. The ample observed host reserve passed the 8-GiB guard;
the observed crash is an explicit Vulkan error, not evidence of RAM OOM. No other
verifier was waited on, polled, interrupted or inspected for desktop content.

Actual app PID was reaped without any signal; its `/proc` entry is absent. The
background driver completion was received before finalization. Final exact
app/driver process match audit is empty. The first broader audit also matched its
own synchronous bash wrapper (PID 352876); the final audit confirms that wrapper
has exited and excludes only its own current synchronous audit processes.
No persistent helper, display/compositor/bus, browser, fixture or worker was
started. Private TMPDIR, HOME, cwd, XDG data/config/state/runtime, app cache and
artifact directory remain empty. Only fresh initialized DB/sidecars and private
Mesa shader-index files were created by app initialization; they are preserved.

All nine preflight-hashed read-only README/history/status/startup/storage/input
sources match their final hashes, as does the original binary. No shared code,
Cargo/lock/design/status/phase report or historical artifact was changed. There
were no unrelated title/content reads, root/desktop screenshots, clipboard use,
synthetic input, forced mapping, global hotkeys, unrelated window moves, host
settings changes, installs, sandbox/security flags or unscoped termination.
