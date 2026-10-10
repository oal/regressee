# Scoped owned Wayland inspection — 2026-10-10

## Latest authorized round: STOPPED at first native failure

**One app launch; exit 101; zero screenshots, stage files, acknowledgements or
input. All 33 UI states NOT RUN. No whole-UI, visual, OS-input or latency PASS.**

The parent explicitly authorized one normal-Wayland round after source/binary
freeze and delegated personal PNG review to this investigator. Parent reported
successful feature wiring, Linux-only gating, 67 passing tests / 3 ignored and
all-feature/all-target Clippy with warnings denied; those are **parent reports**,
not this investigator's native evidence.

Fresh physical 0700 round root (**R**):
`/path/to/private-tests/opencode/regressee-owned-round-f97f93978fcd/`.
Evidence is retained under `R/evidence/`, with final result at `R/result.json`.

### Execution and blocker

1. Verified all source hashes in
   `/path/to/private-tests/opencode/regressee-ui-source-freeze-20261010.json`,
   original binary and physical private copy. Frozen executable SHA-256:
   **`4e18b7c67e565a6ac29b353071fc6b77201f099fb23baeb630ba68f30734b29d`**.
2. Ran only the newly authorized ignored fixture preparation:
   `cargo test --locked --offline -p regressee-app --features ui-inspection --test native_inspection_fixture prepare_owned_native_inspection_fixture -- --ignored --exact --test-threads=1`.
   With private approved TMPDIR, absolute project `.runtime/link-libs` and four
   build jobs, it passed **1 fixture preparation**; source and original/copied
   binary hashes still matched afterward. This is synthetic storage preparation,
   **not browser capture or native UI verification**.
3. Prepared separate private HOME/XDG config/data/cache/state/TMP/cwd/store/ack
   roots and copied binary; preserved existing Wayland runtime/session bus.
   No GPU overrides, renderer-check, X11 or private compositor. The app was
   launched once with `--inspect-ui` and `REGRESSEE_UI_DIAGNOSTICS=1`.
4. Exact app PID **928923**, start ticks **1216765**; controller **928907**,
   independent watchdog **928924**. Registered/started only temporary script
   **0**, `/Scripting/Script0`, unique name
   `regressee-owned-inspection-882d7882b3274d368ebd814e53a5d2be`.
   KWin sender/service owner **`:1.22`**, PID **7228**, start ticks **1808**
   were verified. No unmatched-window property beyond PID was requested by the
   helper. No trustworthy owned UUID/geometry was confirmed before the exit.
5. The application exited **101** while the controller waited for exact owned
   1120×800 client geometry. The batch ended in approximately **1.055 seconds**.
   Controller refused the exited/zombie PID and immediately entered cleanup.
   **No capture API call was reached**, no image existed to read, and no stage
   was advanced. There was no retry, alternative identity, GPU/WM adjustment,
   permission bypass, second app/window launch, browser/runtime action or input.

`R/evidence/app.log` contains the complete five-line native failure:

```text
[destroyed object]: error 7: importing the supplied dmabufs failed

thread 'main' (928923) panicked at .../blade-graphics-0.7.1/src/vulkan/surface.rs:226:18:
called `Result::unwrap()` on an `Err` value: ERROR_SURFACE_LOST_KHR
```

This is the recorded **private normal-Wayland attempt's early surface failure**,
not proof about the user's existing working app/window. Actual adapter identity
was not logged in this attempt; default hardware selection/no GPU override is
the measured environment fact. No speculative graphics/root-cause investigation
or workaround was performed. Mapping state, GPU-presented pixels and
Screenshot2 capture permission remain unverified.

### Visual and timing findings: none obtained

There are **no PNG paths for parent review**. All five screens at both sizes,
action/input differentiation, radio/checkbox padding, attached Dismiss, form
focus/disabled/error states, lower panels, advanced/environment choices,
timezone popup/list, Results/history/analysis/masks and create/save/reopen are
**NOT RUN / NOT VISUALLY VERIFIED**. No stage label or passing source test is
substituted for readable native pixels. No UI repair was made in this round.

The log contains **no `UI timing` markers** and no later Setup/Projects scene.
Cold/warm Setup versus Projects construction/layout/prepaint/CPU paint cannot
be compared; no conclusion about the user's one-second navigation observation,
OS-input latency or GPU presentation timing follows from this failed launch.
The parent may keep that performance question open without treating the fixed
500 ms driver settling time as evidence.

### Cleanup and independent settlement

- `R/evidence/helper-cleanup.json` and `watchdog-settlement.json` record removal
  of only the owned script; app and watchdog were reaped by the controller.
- Independent audit at **2026-10-10T19:28:42.507201Z** confirmed `/proc` absence
  of all three owned PIDs; KWin owner/PID/start ticks unchanged; exact script name
  **not loaded**; private callback service **has no owner**; generated
  `R/owned-helper.js` **absent**. Preparation templates remain as evidence.
- Python emitted a secondary `BusName.__del__` interpreter-shutdown ImportError
  (`sys.meta_path is None`). It was not concealed or used to justify a retry.
  Independent name-owner verification confirmed no private service remained,
  so this warning does **not** leave known helper/service/process cleanup open.
- `before.json` / `after.json` and `final-independent-audit.json` confirm frozen
  sources and original/copied binaries unchanged; all **12** original CAS
  artifacts/hash bytes retained; all synthetic SQLite tables/rows **exactly
  unchanged** (one project, one baseline, two runs, two analyses, one inventory,
  one mask). Integrity check **ok**, foreign-key check empty. No save/persistence
  exercise ran, and no user store/window/project was read or changed.

**Settlement complete; no live controller remains. Parent may end the source
freeze and resume their work after this handoff.** The one authorized round is
consumed. Any future native attempt needs a separate explicit authorization and
a viable route selected before launch; this report does not grant a fallback or
confirmation run. Prior failed routes/evidence remain untouched.

## Historical preparation record (before the round above)

**At preparation handoff: not built, tested, launched or natively exercised.**
The user explicitly allowed a temporary KWin diagnostic helper identifying and
capturing only a separate Regressee test window, removed afterward, without
changing their projects/windows or capturing the desktop. This authorizes the
scoped temporary scripting route, not permission spoofing, global input,
persistent configuration or immediate launch. Parent still owns all UI/backend
edits, serial build/tests, source/binary freeze, launch and image review.

## Deliverables and parent wiring

- New `crates/app/src/ui_inspection.rs`: inner `cfg(feature = "ui-inspection")`,
  **33 finite stages**, no arbitrary commands, native Shell handlers and GPUI
  action dispatch; no rendering/style changes. Includes two **unrun pure protocol
  tests**, not fake-window visual assertions.
- Parent adds `ui-inspection = ["ui"]` in `crates/app/Cargo.toml`,
  `#[cfg(feature = "ui-inspection")] mod ui_inspection;` in `main.rs`, and calls
  `ui_inspection::start(shell.clone(), window, cx)` only for explicit
  `--inspect-ui`. Parent already added real `screen_scroll` to Shell, bound to
  the native screen-scroll container. Normal builds/startup must not invoke this
  driver. An inspection CLI must never silently fall back to ordinary user data.
- Private physical **0700** preparation scratch (**P**):
  `/path/to/private-tests/opencode/regressee-owned-wayland-96ebf72b4e28/`.
  Scripts: `owned_window.js.in`, `owned_controller.py`, `owned_watchdog.py`.
  Templates/imports have no native side effects; controller deliberately has no
  standalone launch mode. Scripts are **unexecuted** in this preparation.
- `P/metadata/` pins official upstream KWin scripting/window headers and source;
  these are master snapshots, **not asserted identical to installed KWin**.
  Previous native failures, research, evidence, and prior scripts remain intact.

No builds/tests, app/native/helper/service registration, display probe, capture,
input, runtime/browser setup, user-window content reads or background processes
were performed in this phase. Only this module/report and new private preparation
files were written. No source edits outside the owned new module/report.

## Temporary KWin helper scope

`owned_window.js.in` uses exactly
`workspace.windowList().filter(w => w.pid === OWN_PID)` and an equivalent
`windowAdded` PID guard. **Only PID is accessed on unmatched windows.** If there
is more than one matching window, it stops; no title/UUID guessing. Caption,
internal UUID, client/frame geometry and minimized state are read only after
the exact PID guard. No window list, other-window titles/URLs/content, active
desktop, pixels, shortcuts, config or other scripts are read or modified.

The helper sends metadata to a fresh unique private D-Bus service containing the
round nonce. The Python bridge validates the **actual sender unique name**, KWin
well-known owner, its bus PID and `/proc` start ticks, and the exact app
PID/start/executable/cwd/private environment. UUID must remain constant. Callbacks
are capped at 4 KiB, with strict keys and a 200 ms asynchronous response cadence.

The **only** returned command is stop or one of logical client sizes
**1120×800 / 820×600**. The helper preserves the owned frame position and adjusts
its dimensions by the observed client/frame difference. One request per size,
then the controller waits for actual client geometry, rather than faking a GPUI
viewport. No activation or global keyboard/pointer injection is implemented.
Natural mapping/geometry failure is a blocker, not a reason to touch another
window, change display/GPU settings or try X11.

Registration uses the live existing `org.kde.KWin` owner, `/Scripting`
`org.kde.kwin.Scripting.loadScript(privateFile, uniqueName)`, then **only that
returned script's** `/Scripting/Script<ID>` `org.kde.kwin.Script.run`.
Upstream `scripting.cpp:100–126,793–829` provides the exact path and lifecycle;
`window.h` exposes PID, internalId, clientGeometry and writable frameGeometry.
Live scripting introspection/permission and JS property conversions have **not**
been exercised. If host methods/object path differ, or access is denied, stop
and unload the unique name—no legacy path or policy bypass/relaunch loop.
Never call the global scripting start/reconfigure operation.

Parent uses `Session.close()` in **finally even if begin fails**. It stops only
the returned owned script, unloads only the nonce-qualified name, checks that
name no longer loaded, releases the bridge/service, and deletes only the generated
`root/owned-helper.js` after confirmed removal. Preparation template and evidence
are retained. Cleanup uncertainty is recorded, not called successful.

## Exact-window capture and no-backdoor review gate

Controller reuses by **pinned SHA-256/read-only reference** the previous prepared
Gio/PIL named-window implementation:
`.../regressee-ui-sweep-hkonn1qy/wayland-route-f3e1aee0757f/scripts/kwin_owned_capture.py`.
Its prior plasma discovery requirement is replaced by the sender-verified KWin
script metadata adapter, not weakened to title-only ownership.

- Require Screenshot2 **Version 5**; named UUID CaptureWindow only, never active,
  desktop, area, output or interactive selector capture. The prior report proves
  only live API/version availability, **not actual capture permission**.
- PID/start/executable/cwd/private roots/UUID/caption/service owner checked
  before/after each shot, frozen copied binary rehashed, returned windowId
  verified, actual client dimensions/scale verified. Fresh metadata ≤1 s old.
- Raw QImage pipe decoder honors format, byte order, stride and premultiplied
  alpha; unsupported formats fail. Separate 3 s D-Bus/5 s pipe deadlines,
  64 MiB cap, no overwrite/repeated attempt. Preserve raw/PNG/metadata/hashes
  and color/alpha counts. No tint/rescale/desktop compositing.
- First capture follows one fixed **3 s** post-map grace; later shots use a
  fixed 400 ms settle after confirmed geometry. Black/error/unreadable means
  stop the batch, not GPU/WM/focus workaround experiments.
- Numeric nonblank checks are **necessary but not sufficient**. Parent must
  call `functions.read` on the real PNG and personally establish readability
  before `acknowledge_after_personal_image_review(stage, png_sha256=...)`.
  Every stage requires that separate acknowledgement; stage zero cannot advance
  before it. A capture alone **does not automatically acknowledge** it.
- App and controller enforce exact numbered labels/dimensions; no arbitrary
  IPC requests or user/project strings enter the command protocol. Acks include
  nonce, stage, constant label, PNG SHA and `personally_reviewed=true`, written
  atomically. This is a trusted development coordination convention, **not**
  a claim that software can prove a human/tool actually read an image.

## Frozen-round directory and manifest contract

After explicit freeze/authorization parent creates a **fresh** physical 0700
round root under `/path/to/private-tests/opencode`, with private 0700
siblings `store`, `inspection`, `evidence`, `cwd`, `home`, `config`, `data`,
`cache`, `state`, `tmp`; copied frozen executable `bin/regressee`.
Do not use the preparation directory or incumbent user projects as app data.
`REGRESSEE_DEV_DATA=root/store`, `REGRESSEE_NATIVE_INSPECTION_DIR=root/inspection`;
HOME/XDG config/data/cache/state/TMPDIR correspond to the private siblings.
Preserve the **valid shared XDG_RUNTIME_DIR**, WAYLAND_DISPLAY and session bus
for native transport only. Default AMD hardware; no driver/GPU flags, private
compositor, install, renderer-check prelaunch or X11 fallback.

`inspection/lease.json` is a private ≤4 KiB object with **exact keys**:

```json
{
  "version": 1,
  "nonce": "<32 hexadecimal characters>",
  "exe_sha256": "<64 lowercase hexadecimal characters>",
  "fixture_project": "<the synthetic fixture project UUID>"
}
```

The app validates physical roots/modes, separate sibling store, private copied
binary path/hash, explicit CLI, nonce and fixture UUID **in the background
executor**. All lease/stage/ack/poll/write/fsync operations are background work;
no file polling or image decoding on the UI thread. A validation failure halts
the driver with a constant privacy-safe log message, never alternate roots.
Stage readiness/diagnostics log only constant labels/durations.

Parent prepares the ignored synthetic Approval UI fixture only after separate
authorization, records fixture ID/artifact hashes/SQL baseline state before
launch, and keeps its identity distinct from the saved-new-project exercise.
Native screenshots display **synthetic images**, not browser-produced captures;
this must stay explicit in the result.

The attached controller `Session(manifest, authorization=...)` requires the
same lease fields plus **exact keys** `root`, `pid`, `start_ticks`,
`expected_caption` (`Regressee — Multi-page preview`) and
`authorization` (`explicit-parent-frozen-wayland-round`). PID comes from parent's
exact owned Popen, not window-title search. App argv must include `--inspect-ui`
and exclude `--check-renderer`; cwd, environment and executable are checked.
Store/ack lifetime must outlive the app and cleanup audit.

## Fixed stage protocol and timing interpretation

Stages **0–15**, wide:
Projects initial; fixture Setup cold; Projects/Setup warm cycles ×2;
Setup advanced; timezone closed/open; login/session choices (draft only);
Results viewer/advanced analysis; History top/lower; Settings top/lower.

Stages **16–27**, small:
all five screens, advanced Setup, timezone closed/open, login/session choices,
Results analysis and lower History/Settings.

Stages **28–32**, small:
ordinary empty new project (disabled controls); normal save validation error;
synthetic field values/focus; normal Save action; reopen that newly saved project
through the ordinary storage worker. No fixture saves, capture/discovery/login,
provisioning, repair/delete/export/visit actions. Fixture baseline/artifacts must
remain identical in post-run SQL/CAS audit. New-project persistence must be
audited separately. New/save actions dispatch the normal GPUI actions, but
synthetic field setters and handlers are **not OS keystrokes or IME acceptance**.

Each immutable `stage-NNN.json` includes constant label, target size, observed
GPUI viewport, handler-to-idle duration and explicit negative OS-input/GPU-timing
claims. App awaits all active worker/analysis work, adjusts the **real** ScrollHandle
and pauses for review. There are no extra menu/navigation commands accepted.
`complete.json` follows the final reviewed ack; it does not close the app or
declare visual quality.

Parent's opt-in `REGRESSEE_UI_DIAGNOSTICS=1` produces CPU Shell construction,
layout/prepaint and CPU scene-paint markers. Compare Projects/Setup warm cycles
and correlate those constant-screen logs with native readable captures.
**Cold fixture Setup includes ordinary project-loading work**, whereas warm
navigation reuses it; handler-to-idle includes predetermined poll/settle time
and is not a precise paint benchmark. **No GPU presentation timestamp or native
OS-input latency is claimed.** A readable snapshot proves presentation at capture
time, not the instant a frame became visible. Neither renderer initialization
nor GPUI test bounds answer the reported one-second Setup latency.

## Independent ceiling, cleanup and remaining limitations

Parent must write their attached controller to physical `root/controller.py`
and start **independent** `owned_watchdog.py` before helper start, with explicit
`--authorize-frozen-round MANIFEST CONTROLLER_PID:START_TICKS owned-only`.
The watchdog verifies that exact private controller cmdline and app lease,
caps the lifetime at **15 minutes**, signals only the owned app (TERM then,
if still identically owned, KILL), and removes only the recorded helper name/ID
if the controller disappears or the ceiling is reached. Controller has a
supplementary local ceiling, not a replacement for the independent watchdog.
Parent must reap their exact app/controller/watchdog Popen objects and stop
the watchdog normally after successful settlement; it never signals KWin.
An intent record precedes load and the returned-ID registration record precedes
run, so a controller crash in the small registration gap can still unload only
its unique name without guessing an ID.
All stop/unload/connection/file-removal errors are retained in settlement JSON.

Unverified assumptions/blockers for parent to resolve **serially before freeze**:

- New Rust module and pure protocol tests are not compiled/run; parent wires
  and checks them alongside their UI changes. No investigator build permission.
- Live KWin scripting path/API/permission, JS QUuid/RectF conversion, callback
  sender identity, natural mapping/resize and capture pixels are unexercised.
  First actual failure must be reported as a blocker, without alternate identity.
- Predetermined scroll fractions (advanced .23, timezone .94, environment .57,
  lower panels 1.0) are sampling positions, **not guaranteed exact section
  alignment or exhaustive every-pixel coverage**. Popups/focus/list content must
  actually be seen in native images. If a requested section is absent, mark it
  NOT VERIFIED and report the mismatch; never claim coverage from stage names.
  No arbitrary scroll-command backdoor or screenshot micro-adjustment loop.
- This includes the default timezone list, not search typing/selection changes;
  no OS pointer/keyboard/IME acceptance, public-site capture, download, live login,
  native persistence via actual OS typing, cross-OS or release qualification.
- Owned app can still close naturally; after any PID/UUID/caption/window mismatch
  stop without trying another window. No still-running controller exists now.

Ceiling: **one future whole-UI batch**, followed by at most **one** separately
authorized correction confirmation if real reviewed images show defects.
No permission/GPU/launch/polishing loops. Preserve failed-route evidence and
source/fixture hashes; no visual PASS until actual meaningful images are read.
