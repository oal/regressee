# Bounded independent native recheck — 2026-10-09

**Native qualification: FAIL / still open.** Approved Python controls were actually
used, rather than assuming the previous result. No implementation or later-phase
changes were made; no agents were spawned. This is not another broad audit.

## Access and exact tests

Read README, status, re-verification, native build/smoke/qualification scripts and
the relevant UI/actor/fixture paths. Available: Python 3.14.7, `DISPLAY=:0`,
`WAYLAND_DISPLAY=wayland-0`, Wayland session, loadable `libX11.so.6` and
`libXtst.so.6`, existing private runtime/binaries and lavapipe ICD. Initial scoped
process/scratch/setup audit was empty. Browser tools expose browser tabs, not a
native desktop controller; direct Python/ctypes controls were available here.

Commands, serially, from `/path/to/Regressee`:

1. `bash scripts/build-ui.sh` — **PASS**, cached locked UI build, 0.36 s. Existing
   upstream `proc-macro-error2 2.0.1` future-Rust warning remains.
2. `python3 scripts/ui-smoke.py` — **FAIL, exit 1**. One ordinary harness attempt,
   PID **641886**. This script's `default` label means ordinary mapping, **not
   default graphics**: it explicitly removes Wayland and selects X11/lavapipe.
   The title + `_NET_WM_PID` selection found its own window, and the harness passed
   its `map_state == 2` prerequisite without conditional mapping. It captured the
   window, sent Ctrl+A, `/public`, four Tabs and Enter through XTest, then failed:
   `native XTest activation NOT PROVEN: app did not enqueue work`.
3. `python3 /tmp/opencode/regressee-native-recheck-diagnostic.py` — **FAIL** in the
   written diagnostic result (driver exit 0 is not a test pass), PID **648446**,
   owned window **37748737**, matching `_NET_WM_PID`. Its first discovery-time
   attributes sample was `map_state=0`; it stopped before focus/input/capture.
   **Diagnostic limitation:** this probe omitted the ordinary harness's mapping
   grace period. It therefore does **not** establish persistent mapping failure
   and did not exercise its planned owned-window activation request. No override
   mapping or third attempt was made. The two-attempt budget is exhausted.
4. Read-only Python PNG inspection — **PASS for image integrity**, **FAIL for
   meaningful rendered pixels**: fresh 1120 × 800 PNG, valid IHDR/IDAT/IEND CRCs,
   exactly **896,000 black pixels**, one RGB color `(0,0,0)`. The fresh image was
   also visually inspected; it contains no visible bench or fixture content.
5. `python3 scripts/qualify.py --audit-only` plus a broader read-only `/proc/exe`
   scan of all private source/installed runtime executables — **PASS**, empty.

## Native check accounting

| Check | Fresh result |
|---|---|
| Native build | PASS |
| Ordinary owned-window discovery / viewable state | PASS in the first attempt; no forced mapping |
| Owned-window PNG acquisition and CRCs | PASS, but entirely black |
| Meaningful bench rendering | FAIL: black pixels, not rendered UI evidence |
| Focus + keyboard activation | FAIL qualification: XSetInputFocus/XTest attempted, no fresh enqueue marker; actual focus readback NOT RUN |
| Public/tall capture and actual fixture tile pixels | NOT RUN after input prerequisite failure |
| Wheel / tile navigation | NOT RUN |
| Active cancellation | NOT RUN: active work never established |
| Managed login idle cancellation | NOT RUN |
| Managed sign-in → I’m ready → `/private` | NOT RUN: no managed browser launched |
| Keyboard Quit | NOT RUN; harness termination is not Ctrl+Q evidence |
| Default Wayland/hardware GPU or X11/hardware GPU | NOT RUN in this bounded pass |
| Final owned-resource cleanup | PASS |

**What changed:** permission now permits actual local Python window identification,
capture and synthetic native input. The first attempt advanced beyond yesterday's
ordinary mapping failure, but neither rendered pixels nor app input activation
qualified. Do not attribute the transient mapping difference to permission or
claim the GPU/input issue is fixed. The current evidence cannot distinguish a
shared display/compositor/input limitation from a native stack/code issue.

## Evidence preservation and resource audit

Fresh evidence is under
`artifacts/native-recheck-2026-10-09-1791554740438976801/`:

- `ordinary-command.{stdout,stderr}`, `ordinary-corrections-ui-smoke-default.json`,
  empty `ordinary-corrections-ui-smoke-default.log`,
  `ordinary-corrections-ui-initial.png`, `ordinary-pixel-inspection.json`;
- `diagnostic-result.json`, empty `diagnostic-app.log`, diagnostic driver copy;
- `audit-command.{stdout,stderr}`, `final-qualify-audit.json`,
  `final-resource-and-preservation-audit.json`;
- `preserved-specific-files/hashes.json` and ten exact pre-existing generated
  files, plus `protected-hashes.json`. No artifact/history trees were copied.

The black PNG SHA-256 is
`0fbef93e6b721fb075c6a2846987f1c3789eeb44d26a0a0b050a6e6b500a82a0`.
Original standard generated files overwritten by this run were restored **only
after** retaining their fresh results; all ten preserved hashes verify. README,
the two prior reports, all three requested scripts, public screenshot and installed
READY hashes remain unchanged. Runtime/source implementation was not edited.

Both own app PIDs were terminated and reaped by their harnesses; no Node/Chromium
was launched. Final scans found **zero live project app/private-runtime processes**,
no `/tmp/opencode/r0/worker-*`, and no installed stage/lock/previous paths. Only the
two owned app PIDs were terminated. The synthetic diagnostic source and evidence
path marker remain under `/tmp/opencode`; they are inert files, not running
processes or browser/session scratch. No unrelated windows/settings were changed,
global installation/sudo used, sandbox disabled, real sites accessed or services
published.

Windows/macOS, supported clean machines, default hardware graphics, packaging,
accessibility, trusted automatic distribution and secure persistent sessions
remain **pending / NOT RUN**, not qualified by Python permission. Keep these
release gates explicit. This bounded verdict does not revoke the user's renewed
direction for the parent to continue local implementation.
