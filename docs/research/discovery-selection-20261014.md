# Discovery page selection — 2026-10-14

## Outcome

Page selection now opens the existing **Page options** panel immediately. The
selected page's complete address, capture status, exclusion reason and explicit
Include/Exclude control appear together. **Previous page** and **Next page** open
the same details and reveal the selected row, instead of only moving a highlight.
Navigation does not change inclusion. Hover explains the row's action rather than
repeating the address; complete addresses remain available in the panel.

Discovery-blocked pages keep their existing inclusion guard and explain the
settings/rediscovery recovery path. Search-hidden stale clicks do not steal
selection or open the panel. Changing the selected page resets the panel's scroll
to its details; new discovery lists reveal their first row. Saved evidence and
baseline state remain untouched by local page selection/inclusion edits.

The existing Clear wayfinding palette, row geometry, virtualized list, bounded
panel and keyboard controls are preserved. This is a focused interaction repair,
not a replacement visual system or a new capture capability.

## Verification

All five `options_ux_tests` regressions pass; the opt-in renderer test is separate
from that ordinary run. The normal UI build and UI-binary Clippy check with
`-D warnings` also pass.

`options_ux_tests::renderer_capture_discovery_selection` passed using isolated
synthetic storage. Its shared regression covers simulated row clicks, panel
opening, explicit exclusion/restoration, blocked inclusion, Previous/Next,
offscreen-row reveal, scroll reset, stale clicks and unchanged saved records.

Four actual Linux GPUI offscreen captures were inspected together: included and
blocked selection at **1280×800** and **820×600**. Addresses wrap inside the panel;
page actions/recovery copy and the searchable list remain accessible. Private
synthetic pixels are retained under `regressee-discovery-selection-GKncavGm`, not
committed as user-site evidence. This establishes development-host layout and
simulated interaction only, not OS-input or accessibility qualification.

The broader app-package test attempt failed five unrelated browser-setup/catalog
tests (99 passed, 2 deliberately ignored in its library suite). No runtime policy
was changed to make those tests pass. One missing helper qualification in
`task_ux.rs` was corrected solely to unblock compilation.
The separate full UI-binary suite exceeded its 120-second bound during the
existing Phase 7 qualification test; it is not claimed as a complete suite pass.
