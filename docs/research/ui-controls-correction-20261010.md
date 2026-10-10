# User-reported control/layout correction — 2026-10-10

**Local implementation/checks passed; native inspection blocked.** This follows user
screenshots, not a successful private capture. Prior black-frame and swapchain
failures remain in their original reports.

## Scope and visual intent

Preserve the existing cool light/teal desktop identity and ordinary task flows.
Separate editable white, full-width fields from compact filled actions. Keep
radio/check selection indicators distinct from keyboard focus. Group feedback
and its dismissal as one surface. Use a searchable list for timezone selection,
not a free-form value. Browser setup is a separate actual product gap, not merely
an error-copy problem.

Parent source changes so far:

- `ui/controls.rs`: intrinsically measured flex action labels; non-navigation
  actions fit their content; secondary actions have neutral fill; action/choice
  heights do not flex-shrink. Full-width choice rows have balanced 12px insets.
- Message dismissal is inside the message surface, beside its copy. Long
  messages have a 160px scrolling text area so recovery controls and screen
  content remain available. Busy-operation actions are grouped with progress.
- `core/shell.rs`: environment traversal follows visible field/choice order,
  including the timezone selector before website-theme choices.
- Minimum-width footer actions wrap, and setup guidance shares available width.

## Evidence boundaries

The Import regression first failed with the exact reported character column:
852×40 action versus 0×400 label at 1120×800. The narrow flex-wrapper repair
passed at 1120×800 and 820×600 before this additional correction batch.
Those bounds are test-platform evidence, not native pixels.

New regression assertions passed for compact action width, wrapped choice text
insets, attached dismissal containment and bounded long feedback at both real
window sizes. The integrated bundled searchable timezone selector has selection,
search, keyboard, clipping, disabling and preservation checks. Its selected value
is authoritative for dirty detection and the atomic project/environment save.

The parent serial ordinary unit/simulated-UI command passed **160 tests / five
deliberate ignores** (app library 45, app shell 20, core 43, runtime 22, storage
30). All-target/all-feature Clippy passed with warnings denied. These are local
contracts, not a rendering/usability/native-input acceptance result. An initial
GPUI method-name compile failure, an inactive-window Tab/blur regression and one
test-only Clippy warning were corrected; those failures are not discarded.

The user additionally reports about one second navigating to Setup in debug.
Opt-in `REGRESSEE_UI_DIAGNOSTICS=1` now records navigation → shell construction,
layout/prepaint and CPU scene paint using only fixed screen labels and timings.
It does not time GPU presentation, identify the one-second cause, or prove a
release build fixes it. The native inspection batch must distinguish cold/warm
Setup from Projects; the authorized round failed before either scene could be
measured, so no native timing result or latency fix is claimed.

The normal Wayland route exposed no PID/UUID discovery or safe automation
interfaces. The user explicitly authorized a temporary KWin helper restricted
to a separate owned Regressee window. The one authorized normal-Wayland round
exited 101 with dmabuf import failure / `ERROR_SURFACE_LOST_KHR` before capture.
All 33 planned states are NOT RUN, no PNG exists, and no action/input was sent.
Exact helper/service removal and owned-process settlement were independently
confirmed; frozen source/binary and synthetic SQLite/CAS stayed unchanged.
See `ui-owned-wayland-inspection-20261010.md`. There was no retry or fallback.
App-internal diagnostic navigation would not qualify OS keyboard/mouse input.

After independent settlement, parent confirmed all 73 frozen inputs and original
binary unchanged, ended the freeze, passed `cargo fmt --all -- --check`, and
rebuilt the ordinary non-inspection `ui` executable with locked/offline inputs.
Current ordinary binary SHA-256:
`84240c1489965a60dc22373039f9d9568bb83dfe104d5f0aa9ed9a90531caa89`.
No native relaunch was performed; this is not a renderer repair or confirmation.

Independent source-only review also passed without a material P1/P2 fault:
`ui-controls-correction-source-verification-20261010.md`. Backend first-use setup
independently passed 7 Product + 3 Jobs + 5 public-API probes, within signed-inert
fixture scope: `automatic-runtime-setup-verification-20261010.md`.

`impeccable context` loaded existing PRODUCT/DESIGN. Its native phone references
do not change this desktop target. The layout detector returned `[]` on the Rust
targets; this is not evidence that the GUI is readable or well composed.

Runtime diagnosis: `automatic-runtime-setup-gap-20261010.md`. No public runtime
host, release key or qualified default is invented. Original TLS, sandbox,
signature, private-path, process-ownership and immutable-baseline checks stay in
force. Default all-platform automatic setup remains incomplete.
