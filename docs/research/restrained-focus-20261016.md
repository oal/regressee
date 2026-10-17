# Restrained focus rings — 2026-10-16

The user rejected the oversized focus outline around Compare. This is a narrow
Impeccable **Operate** refinement, not a layout or interaction redesign.

## Treatment

- Replace the 3px ink ring and 2px gap (5px outward extent) with a crisp 2px teal
  ring and 1px white gap (3px outward extent).
- Keep normal control borders instead of adding another dark focus stroke.
  The navy rail uses a white ring with a navy gap, including selected navigation.
  Editable fields retain their teal focused border.
- The shared helper carries this through actions, disclosures, toolbar tools,
  choices, History rows, fields, Zoom, timezone, setup steps and links. Focus
  replaces ordinary elevation, remains visible through hover and preserves text,
  layout, selection, primary/danger fills and 40px targets.
- Preserve the transparent screenshot viewer's existing 1px border-only focus;
  never shadow captions, gaps or screenshot pixels.
- Preserve keyboard activation ownership and the earlier hard-edge shader fix.
  No new palette, motion, dependency, asset, capture operation or approval rule.

## Verification scope

Shared style tests check all focus/hover/pressed combinations, unchanged borders,
geometry and inherited text; semantic contrast checks retain the existing palette.
The opt-in renderer checks both ring pixels, the separating gap and absence of the
old oversized extent. It covers Compare, Zoom, a selected review choice and a
primary action at 1440×900, 1280×800 and 820×600, plus settings actions, hovered
disclosures and selected navigation at the latter two sizes. The viewer fixture
also checks that focusing controls leaves persisted evidence unchanged.

Evidence directory:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-focus-20261016-xoqj8utg/`.
Website fixtures are frozen synthetic images copied byte-for-byte from the prior
depth inspection; `fixture-provenance.json` records the source and SHA-256 hashes.
These are diagnostic captures, not shipping rasters or actual customer evidence.

Completed automated checks:

- App library and shell tests with `ui-renderer-capture`: **338 passed, 0 failed**
  (133 library, 205 shell; 54 opt-in tests ignored in this general run).
- Both opt-in focus renderer tests passed explicitly: **18 frames**, with exact
  ring/gap/extent pixel assertions, stable control bounds and unchanged evidence.
  `source-receipt.json` binds every PNG to its checksum and current focus sources;
  all frames are opaque actual GPUI/Blade output, not browser mockups.
- Production `cargo build --locked --offline -p regressee-app --features ui
  --bin regressee`: passed.
- Scoped Rust formatting, design-sidecar JSON parsing and `git diff --check`:
  passed. Existing uncommitted work is preserved.

The default mise shims had no selected toolchain, and the repository's old
Debian linker-name symlinks do not resolve on this host. Verification uses the
already-installed project-pinned Rust 1.98.1 toolchain and the previous depth
inspection's existing host linker aliases, without changing global configuration
or repository runtime symlinks. The existing dependency future-compatibility
warning for `proc-macro-error2` is unrelated to focus styling.

## Bounded visual finish

Inspected the complete 18-frame sheet, plus full-size 1280×800 Compare,
1440×900 Zoom, 820×600 primary action and selected navigation. The teal outline
is thinner, removes the heavy dark double boundary and stays distinguishable
from mint selection. Primary fills and the sidebar remain readable; no new
overlap, clipped focus or control movement was found in these states. No second
visual edit or re-render loop was needed. In-session verdict: **ship this scoped
focus refinement**; not an independent whole-app review.

Development-host GPUI/Blade offscreen rendering and simulated keyboard input do
not qualify real OS input, screen readers, other operating systems or release
accessibility. No such acceptance is claimed.
