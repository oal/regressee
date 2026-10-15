# Keyboard activation and visible focus — 2026-10-15

User intent: stop keyboard section toggles from immediately reversing, make
focused buttons unmistakable, preserve the existing interface, commit and push
all current workspace changes. Impeccable direction: **Operate**, narrow repair
of Clear wayfinding; no redesign, new operation, animation or shipping raster.

## Confirmed causes and changes

- Shell Enter/Space dispatches `Activate` on key down. GPUI's focused `on_click`
  also dispatches a keyboard click on key up. With an intervening redraw, one
  press toggles twice or clicks a newly focused dialog action. Existing
  `simulate_keystrokes` tests send only key down and missed this path.
- Added a shared pointer-click gate for actions, disclosures, choices, checks,
  setup steps, History rows and external links. Shell bindings remain the sole
  keyboard activation owner; text input and selector-owned bindings are intact.
- Blade's zero-blur shadow shader evaluated a zero-width Gaussian, yielding
  invalid alpha and hiding the outer focus outline on Linux. The contained
  vendored fix matches Metal's existing signed-distance hard-edge path; the
  Windows shader had the same defect and receives the equivalent correction
  (source-reviewed, not Windows-runtime qualified). Ordinary blurred shadows
  and clipping are unchanged. Both shader provenance hashes are updated.
- Shared focus is now a 3px ring with a 2px contrasting gap. Light surfaces use
  ink/white; the navy rail uses white/navy. Focus does not change control size,
  type, selection, hover behavior, primary fill or danger semantics. Existing
  semantic colors supply both edges, with source contrast assertions.

## Verification

The new full press/release regression reproduced the Browser-details failure
before the keyboard repair (`false` immediately after pressing Enter).

Coverage includes all 17 disclosure sites at 1280×800 and 820×600, Enter and
Space with a redraw before release, controller collapse (including focus on a
child), focus retention, mounted children, attached content, unchanged work
requests and persisted evidence.
Settings also checks Tab/Shift-Tab reading order, scroll reveal, dialog
open/close across key release, checkbox/radio activation, pointer activation
and spaces/Enter in a field without saving.

The opt-in renderer proof captures focused actions, hovered expanded disclosures
and selected rail navigation at both sizes. Pixel assertions sample both the
outer ring and its contrasting gap so an invisible shader outline cannot pass
on style metadata alone. Captures are private synthetic Linux GPUI/Blade
offscreen evidence, not website results or shipping assets.

The six focus captures and pixel assertions passed and were visually inspected:
`.runtime/keyboard-focus-20261015-press-release/keyboard-{size}-{scene}.png`
and their source-hash JSON receipts. No second visual repair was needed.

New shader hashes match their provenance. A full vendor hash scan found
unrelated pre-existing `src/taffy.rs` metadata drift; its current bytes match
HEAD exactly. This repair does not silently rewrite that older provenance.

Completed checks:

- `cargo fmt --all -- --check` and `git diff --check`: passed.
- `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings`: passed.
- `python3 -m unittest discover -s scripts -p 'test_*.py'`: 57 passed.
- All-feature app library in the final run: 124 passed, two existing opt-in tests
  ignored.
- All-feature native UI binary: 200 passed, 45 opt-in tests ignored, including
  both new keyboard tests and the 17-site/two-size disclosure matrix.
- Explicit `renderer_capture_keyboard_focus`: passed, six captures plus exact
  ring/gap pixel assertions at both sizes.

The initial all-feature workspace run hit its 120-second command limit during
the existing 100/1,000-page storage stress qualification, after the app tests
above passed. That attempt is not claimed as a full workspace pass. The bounded
follow-up excludes only
`phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention`.
It completed successfully: **481 passing Rust tests, zero failures**, 75 existing
opt-in tests ignored (aggregate includes the private-path child test). Its log
is `.runtime/keyboard-focus-workspace-bounded-tests.txt`; the initial timeout log
is `.runtime/keyboard-focus-workspace-tests.txt`.

Scoped disposition: **ship the keyboard activation and focus repair**. Both
reported defects are resolved within the tested native controller/render scope,
without broadening save, browser, approval or deletion permissions.

Native OS keyboard hardware, screen readers, other operating systems and
release qualification remain untested by this repair.
