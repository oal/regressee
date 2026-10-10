# Import-label repair: one native inspection round — 2026-10-10

**BLOCKED. No readable native after-image, no native label/layout PASS, and no
claim that the GUI is fixed.** The one authorized private launch remained black
at its predetermined five-second-after-map gate. I opened the actual PNG with
the image tool: entirely black, no readable labels or controls. No input,
activation workaround, resize, relaunch or GPU experiments followed.

## Keep the three evidence classes separate

| Evidence | What it establishes |
| --- | --- |
| User's real desktop before-image, described in parent handoff | Import baseline ZIP appeared as a one-character vertical column with a blank full-width button. This session did not recapture or manipulate the user's window/project/display. |
| Parent's actual GPUI **test-platform** shell bounds and targeted tests | Handoff reports old 1120×800 action 852×40 at (236,183.5), block label 0×400 at (662,3.5); repaired label 168×20 inside the action at 1120×800 and 820×600. The new containment regression failed old shape and passed repair; shared/style/bounds checks and scoped clippy passed. These are **parent-reported**, not tests rerun here, and are not native readable pixels. |
| This session's frozen repaired binary, exact-owned native client PNG | Naturally mapped 1120×800 client, but black RGB pixels after fixed grace. Does not demonstrate button text containment/readability or any other visual repair. |

The repair handed off by the parent is narrow: `.flex()` on the shared action
label wrapper, retaining min-width/normal wrapping and stable focus typography;
test selectors only, no redesign. No source changes were made in this round.

## Frozen candidate, physical private evidence

Evidence root (**N**):
`/path/to/private-tests/opencode/regressee-ui-sweep-hkonn1qy/native-label-6749ca19fbc5/`

Native binary SHA-256 (original `target/debug/regressee` and private copy):
`663653ffa193b77e7097f78faa6ea5b9eab51cec0fff658c074305e30a60f952`.

`N/freeze.json` hashes 68 source/Cargo/binary inputs and the original consulted
reports/evidence. `N/before-launch-freeze-audit.json` and `N/final-audit.json`
show **no changes** to those inputs or old evidence. No old report was edited.
N and its ancestors are physical, not symlinks; N/HOME/XDG/TMP/cwd/bin/evidence
roots were created mode 0700. `/proc/self/mountinfo` confirms the enclosing
`/home/developer/Data` mount is ext4 (`/dev/nvme0n1p1`), device 66309; initial
available disk space was 1,263,963,910,144 bytes.

Infrastructure rationale is in `docs/research/ui-native-capture-gate-20261010.md`:
GPUI Map/Visibility/Expose schedule frames without requiring WM/focus. Observer
XSync is not Vulkan presentation acknowledgment. Replacing the old one-second
sample deadline with a fixed five-second grace was a supported measurement
correction, **not** a proven rendering fix. No source evidence justified focus
before the zero-input pixel gate, a WM, forced mapping, compositor changes or
additional flags.

## Exactly what ran

1. **One ignored synthetic fixture preparation**, serial, locked/offline:
   `cargo test --locked --offline -p regressee-app --all-features --bin regressee
   tests::prepare_native_ui_sweep_fixture -j1 -- --exact --ignored --test-threads=1`.
   `REGRESSEE_NATIVE_UI_FIXTURE=N/app`, physical `TMPDIR=N/tmp`, project-private
   link-library path. `N/fixture.log`: one PASS, 18 filtered out; reused existing
   test executable, no native application rebuild or ordinary suites rerun.
2. **One owned Xvfb**, Unix-only `:198`, 1400×1000×24, fresh MIT cookie,
   `-nolisten tcp`, private HOME/TMP/cwd. Read-only private Xvfb package plus
   previously evidenced private xkbcomp path preload **on the server only**.
   No distribution/system installation or authentication borrowing.
3. **One copied native application launch**, exact environment allowlist in
   `N/launch-environment.json`. Private HOME/XDG/TMP/cwd, `REGRESSEE_DEV_DATA=N/app`,
   owned DISPLAY/authentication; only established
   `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json` and renderer diagnostics.
   No app LD_PRELOAD, Wayland/DBus/session/browser/GPU experiments. The actual
   process environment/executable/cwd/private database fds were checked in
   `N/initial-process.json`; maps retained in `N/initial-maps.txt`.
4. Exact client naturally mapped; observer subscribed to client
   Structure/Exposure/Visibility/FocusChange. **One** predetermined capture after
   five seconds from the first observed viewable map. No duplicate controller,
   black-frame loop or second launch. An owned 15-minute supervisor was launched
   for the lifetime bound; this attempt stopped in about 5.6 seconds instead.

`N/app.log` identifies **llvmpipe (LLVM 21.1.8, 256 bits)** and initialization
capabilities only, not a completed presentation. `N/xvfb.log` contains the known
/tmp X11 directory-owner warning; the server nevertheless mapped the client.
No browser, website capture, clipboard, root capture or user-display connection.

## Client ownership and finite timing evidence

| Item | Recorded value |
| --- | --- |
| Controller / supervisor / Xvfb / app PIDs | 648963 / 648965 / 648967 / **648993** |
| Exact client XID | **4194305** |
| Exact WM_NAME and PID property | `Regressee — Multi-page preview`; **648993** |
| App launch, controller-relative elapsed / Unix time | 0.245396 s / 1791656501.448646 |
| First observed viewable map | 0.396337 s / 1791656501.5995827 |
| Map/Visibility/Expose observer dequeue | ~0.3961 s; visibility state 0 (unobscured); full 1120×800 Expose |
| Actual image metadata time | 5.405159 s / 1791656506.6084108, ~5.009 s after observed map |
| PNG encoding completion | 5.540398 s / 1791656506.7436502 |
| Client geometry | 1120×800, root (142,100), border 0, depth 32, map state 2; override-redirect false |
| Root containment | `[142,1262) × [100,900)` inside 1400×1000; unobscured confirmed |

These are **observer dequeue/capture clocks**, not GPUI processing or GPU
completion timestamps. First frame/presentation time was not observed. There is
no current-launch screenshot at 1.26 seconds and no claim of permanent blackness.
The former isolated binary was black at ~1.26 seconds; this repaired binary was
black at the later bounded gate. The cause remains unknown; a five-second wait
alone did not produce qualified pixels on this route.

### Actual native PNG for parent review

`/path/to/private-tests/opencode/regressee-ui-sweep-hkonn1qy/native-label-6749ca19fbc5/projects-1120-initial.png`

SHA-256: `0fbef93e6b721fb075c6a2846987f1c3789eeb44d26a0a0b050a6e6b500a82a0`.
It matches the old black PNG because both have the same uniform black dimensions,
not because old pixels were reused: new captured raw bytes/metadata are retained.
The filename denotes the expected initial route, not proof of a visible Projects
screen. Numeric gate: **1 RGB color; 896,000/896,000 black pixels**. Image-tool
review confirmed that no Import label, button, navigation or task content is visible.

`N/projects-1120-initial.zpixmap` preserves 3,584,000 raw bytes;
`N/projects-1120-initial-ximage.json` records ZPixmap, depth/bpp 32,
LSBFirst, xoffset 0, stride 4480, TrueColor class 4 / visual ID 64,
RGB masks `ff0000/ff00/ff`. Four fixed pixel positions cross-check the decoder
against XGetPixel; both return `0xff000000`. Post-exit full raw-byte audit finds
**all 896,000 pixels are `0xff000000`**, not hidden nonblack RGB with zero alpha.
RGB PNG encoding does not multiply the high byte into the color channels.
Depth 32 is not a reason to substitute a desktop/root/compositor screenshot.

## Secondary controller failure and independent cleanup audit

After the failed numeric gate, the controller's finally block terminated/waited
the app and server, **then** called XCloseDisplay. Closing the observer connection
after its X server stopped caused fatal XIO, exiting the controller with code 1
and preventing its final `result.json`/in-memory exception from being written.
The shell completion preserves: `fatal IO error ... on X server ":198"`, after
45 requests. `N/checkpoint.json` is a retained **intermediate** checkpoint (its
`NOT RUN` outcome is stale), not a successful completion record.

This is an additional infrastructure cleanup defect, **not** an explanation of
the earlier saved black pixels. No controller fix/relaunch was attempted.
An owned supervisor-stop marker was added for settlement; its own final marker
was not produced, so watchdog successful-finalization is not claimed.

`N/audit_after_exit.py` performed metadata/read-only SQL audits only, no display
connection or app launch. `N/final-audit.json` records the actual final verdict:

- Exact controller, supervisor, app and server PIDs are **absent**; no live owned
  processes or zombies at audit. App/server waits precede the failed XCloseDisplay
  in the controller source; exit codes were not durably saved, so none are invented.
- Owned `:198` socket/lock and private keymaps absent; no global deletion/cleanup.
- Frozen source/original binary and prior reports/evidence unchanged.
- Private SQLite integrity **ok**; database-file hash, project/config/revision,
  table counts, active baseline and fixture artifact inventory unchanged.
- Synthetic corpus: one project, one baseline, two runs, two analyses, one
  inventory/mask; width-96 synthetic profile. Local session/environment tables
  empty. Fixture PNGs are saved test bytes with synthetic loopback metadata,
  never real website/native-after evidence.

## Scope verdict and stop

| Check | Verdict |
| --- | --- |
| One authorized ignored synthetic preparation | PASS; not native capture/input |
| Frozen binary/source, private process/DB evidence and post-exit preservation | PASS |
| Exact-owned naturally mapped client, bounded later capture, decoder cross-check | PASS, infrastructure observations only |
| Readable repaired Projects / Import text containment at 1120×800 | **BLOCKED** |
| Projects at 820×600; Setup/History/Settings/Results batch | **NOT RUN** after pixel gate |
| Real keyboard create/save/reopen; visible focused-button bounds | **NOT RUN** after pixel gate |
| Controller finalization | **FAIL**: shutdown-order XIO; no surviving processes after independent audit |
| Native fix, general visual quality, user approval, hardware/default Wayland/release | **NOT QUALIFIED** |

Stop here. Parent may inspect the provided PNG and retain the test-platform
containment regression as narrower evidence, but must not call the label fixed
from this native round. No mock, test bounds, renderer-init output or synthetic
fixture image replaces the missing readable native after-image. Any different
capture route would require separate explicit authorization; this round grants
none. No further refactor or native retry is proposed under this handoff.

Context-only note: the design skill loader reported a stale
`.impeccable/design.json` sidecar; it was not changed. A separately requested
`document` refresh could address that drift, unrelated to this blocked gate.
