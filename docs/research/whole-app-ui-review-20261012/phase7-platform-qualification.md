# Phase 7 — platform and acceptance matrix

**2026-10-13 · independently verified PASS: implementation + feasible gate only.**
Windows, macOS and Linux are product targets, not an assertion of qualified OS
versions, package formats or runners. This phase does not choose missing release
support policy or publish a release. See [report](phase7-report.md) and
[checkable receipt](phase7-receipt.json). The final
[independent verdict](phase7-verification.md) preserves exact results, independent
native traces and verifier-induced volatile SQLite SHM deltas; none is native or
release acceptance.

## Separate gates

| Gate | Actual evidence / outcome | Unmet acceptance |
|---|---|---|
| Source/controller/storage | **Independent feasible PASS:** all nine commands exit 0; workspace 358/0/45 and 177/0/14, UI-only 22/0/0, UI-tests 120/0/1. Real Jobs approvals and distinct 100/1,000-page original pixels/maps independently checked | Synthetic fixtures are not website/browser throughput; counts overlap and ignored tests remain unexecuted; no native/release acceptance |
| Linux GPUI/Blade offscreen layout | Both 1280×800 and 820×600; 28 initial + 28 confirmation pairs; actual inner input/scroll owners, nine analysis parameters, long checkbox, exact diagnostics tail and 100-reference consent tail | Not OS presentation, full-page screenshot readability or task success |
| Real Linux native input | **BLOCKED / NOT TESTED.** Owned production-process attempts, real display/XTEST libraries; default Vulkan launch panics. Forced Lavapipe X11 creates an unmapped 1280×800 window; input is refused safely. Wayland process survives startup, but no owned native screenshot/input/IME automation is available | No pointer/keyboard full path, shortcuts, focus trap/restoration, Escape/discard, resize, clipboard or IME pass. TestAppContext and gallery are not substitutes |
| Linux accessibility | **BLOCKED.** Real AT-SPI bus/registry is available; owned running Wayland app is not registered. Vendored Linux GPUI has no AT-SPI/accessibility bridge. Orca executable is absent | This is a framework/integration deficiency, not merely an unavailable screen reader: no names/roles/states/announcements or accessible task pass. Requires native accessibility engineering and subsequent AT testing |
| Windows / NVDA | **NOT TESTED:** no Windows runner/desktop or NVDA | Native input, focus, clipboard/IME, fonts, scaling/high contrast, announcements, installer/runtime and package acceptance |
| macOS / VoiceOver | **NOT TESTED:** no macOS runner/desktop or VoiceOver | Native input/host conventions, focus, clipboard/IME, fonts, scaling/appearance/motion, announcements and signed distribution acceptance |
| OS scaling/high contrast/motion | **NOT TESTED as host behavior.** Read-only host preferences: text scale 1.0, display scale 1, Adwaita, animations enabled. No global preferences changed. Source has fixed logical typography/light semantic tokens, no automatic text/high-contrast/motion preference integration; explicit `REGRESSEE_REDUCED_MOTION=1` exists | No 150%/200% text-scale, high-contrast theme or OS Reduce Motion acceptance. Window resize/offscreen logical pixels are not text scaling. The environment override is not an automatic OS-preference bridge |
| PM task usability | **NOT TESTED: zero participants** | Recruit actual first-use/frequent-review/low-vision keyboard users; measure page identification, small-change inspection, review-versus-update understanding, consent explanation and gap recovery. No invented task rates/time savings |
| Runtime / release | **BLOCKED / NOT QUALIFIED.** This normal build has no trusted production browser distribution. Old local development Node/Playwright dependency can support boundary tests only | Actual public/private first-run install/login/capture, production Playwright/runtime tuple, each OS package, CI assets, signatures/support policy and release usability/AT acceptance. CI/release work remains with parent after phase verification |

## Reproducible native attempts — no real projects/sites

`scripts/phase7-native-probe.py` uses a fresh 0700 approved temporary directory and
`REGRESSEE_DEV_DATA`, clears inherited runtime overrides, launches only its owned
binary and targets its exact PID/title. X11 pointer/keyboard events require an
owned, viewable, focused window. It never types into another app or launches a
site. It terminates only its child and preserves log/receipt. Failed preflights
are not counted as native input. The script is an executable qualification path
for a capable future display, not proof that this host completed it.

```bash
LIBRARY_PATH="$PWD/.runtime/link-libs" TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
  cargo build --locked --offline -p regressee-app --features ui --bin regressee
python3 scripts/phase7-native-probe.py --binary target/debug/regressee \
  --parent /var/home/olav/Data/opencode/tmp/opencode
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json \
  python3 scripts/phase7-native-probe.py --binary target/debug/regressee \
  --parent /var/home/olav/Data/opencode/tmp/opencode
VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json \
  python3 scripts/phase7-native-probe.py --backend wayland --binary target/debug/regressee \
  --parent /var/home/olav/Data/opencode/tmp/opencode
```

Available: DISPLAY `:0`, WAYLAND_DISPLAY `wayland-0`, session D-Bus, X11/XTEST and
libatspi. Not available: Xvfb/Xephyr/Weston/xdotool/Orca/ibus, physical uinput write
permission, Windows/macOS runners. A browser screenshot of the gallery would not
fix any native qualification gap. No tooling download, project/network visit,
host theme/scale mutation, nested agent, commit, push, tag or CI repair occurred.

Final locked/offline current-source production build passed; its preserved log is
`/var/home/olav/Data/opencode/tmp/opencode/phase7-final-native-build.log`.
Repeated owned Wayland probe `regressee-phase7-native-bkqjr8xr` again lived eight
seconds but was unregistered on the real AT-SPI desktop. No OS input was sent.

## Native completion checklist (all externally unaccepted)

On an actual distributable build/desktop, execute keyboard-only creation/public
baseline and private login baseline, check a fixed inventory, search/filter while
keeping selected updates, inspect each mode/zoom/pan, save a note and integer mask,
review/preview/cancel/confirm, browse the last retained run/version, ZIP share/new
recipient login, deletion and separate cleanup/recovery. Repeat at normal/minimum
windows with long labels and supported OS text scaling. Exercise real pointer,
shortcuts, Tab/Shift-Tab reveal, menu/dialog focus trapping/restoration, Escape and
discard, resize, reversible clipboard roundtrip and an actual IME engine. Inspect
accessible names/roles/state announcements and status changes with Linux AT,
NVDA and VoiceOver. Participant observation and distribution tests are separate
from agent input. Each unexecuted cell remains NOT TESTED, not a release waiver.
