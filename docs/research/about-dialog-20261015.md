# About dialog — 2026-10-15

User-requested scope: link to the GitHub repository and simplify the About text.

- `form_recovery.rs` now leads with one sentence explaining screenshot comparison.
  Version/license, local storage, ZIP/screenshot privacy and development limits
  remain explicit, with substantially less text.
- The underlined `https://github.com/oal/regressee` link uses the shared native
  web-link treatment and opens the default browser only on activation.
- The link and Close share the dialog's focus trap. Escape restores the About
  trigger. Keyboard help does not mount or enable the repository link.
- Existing branding, palette, dialog geometry and full-width Close are preserved.
  No new design tokens, assets or backend operations were introduced.

Scoped verification passed: 12 core focus tests, 3 form-recovery tests, the existing
shell dialog test and 4 help/validation tests. The explicit renderer test produced
1280×800 and 820×600 GPUI/Blade captures; final inspection confirmed complete copy,
a visible focused link and an unobstructed Close action. Scoped disposition: ship.

Private evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-about-20261015-nf69omfb/confirmation/`.

These are offline Linux layout/rendering and simulated interaction checks, not
OS-input, screen-reader, cross-platform or release qualification. Unrelated
concurrent work was preserved.

## Loading-state follow-up

The busy-state focus candidates omitted About while explicitly allowing keyboard
help. Both read-only information actions now remain available during App settings
reads, in their painted footer order. Native prompts and exclusive tasks retain
their guards; browser setup, capture and saves are not unlocked.

The entry flash came from automatic `InspectStorage` being classified as a modal
task. It now uses the existing inline storage-loading status, like network and
browser-status reads. Explicit repair previews and mutation dialogs are unchanged;
no delay was added to hide the symptom.

Verification: 13 core focus tests, 6 App settings tests, 4 task-policy tests and
11 task UI tests passed, as did the native app build. New coverage steps through
each actual App settings entry reply without creating a task or moving focus,
and holds a browser read pending while opening About and keyboard help at both
desktop sizes. Browser-check completion leaves an open About dialog intact.
An opt-in renderer test passed with four inspected GPUI captures showing enabled
footer actions and About open during verification at 1280×800 and 820×600.

Private evidence:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-settings-check-20261015-iwihj98s/`.
Scoped disposition: ship; the qualification limits above still apply.

## Shared link-hover follow-up

User-requested scope includes links everywhere, not only About. All external
links use `ui/web_link.rs`; About and task context/page/final-address links now
deepen only their underlined ink on hover/press, without a full-row mint fill.
Keyboard focus, destination hints, disabled guards and activation stay intact.
The shared label sets hover ink before GPUI text shaping while inheriting its
owner's typography and optional ellipsis; paint-only text-color refinements do
not recolor this pinned renderer's shaped glyphs.

The expanded About renderer test checks both 1280×800 and 820×600: hover changes
the glyph pixels, every previously white row pixel stays white, and hover/focus
preserve control geometry. Six final captures cover rest, hover and keyboard
focus. The first paint-only implementation failed the pixel comparison and was
replaced before confirmation. No unrelated control styling was changed.

Private confirmation evidence:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-link-hover-20261015-SRYZenjK/confirmation/`.
The native app build and 118 library tests passed (2 opt-in tests ignored).
The task-link keyboard assertion follows the current painted Details-before-page
links order; runtime navigation is unchanged. These remain offscreen Linux and
simulated-interaction checks, not native input/accessibility qualification.

## Perceptible hover and compact hints

User feedback supersedes the earlier hover acceptance: the teal shade change
was not perceptible enough, and the fixed-width two-line URL hint was oversized
and redundant. All shared links now switch to existing dark ink and a 2px
underline on hover, without a row fill or changed text metrics. Keyboard focus
remains distinct.

The hint is only **Open in default browser**, never a repeated URL. Its visible
bubble is content-sized (about 175×32px on this Linux host), with 13px/18px type
and 8px/6px horizontal/vertical insets. The existing 500ms delay avoids flashing
hints during pointer travel. A transparent 12px vertical gutter keeps it off the
link/cursor; framework edge flipping and stationary placement remain intact.
It does not follow the pointer, take focus, or introduce another activation path.
Complete addresses remain in website settings and page details. No new palette
tokens, shipping assets, or unrelated tooltip changes were introduced.

Verification: 123 library tests passed (2 opt-in tests ignored), including the
new two-size delayed/compact/stationary/edge-flipping/disabled hint test. Three
form-recovery tests and the task-link pointer/keyboard/guard test passed. The
expanded renderer test passed eight captures at 1280×800 and 820×600, asserting
darkened glyphs, unfilled trailing row space, stable geometry and a bounded hint
below the link. Both hint captures were inspected. These are Linux GPUI offscreen
and simulated-interaction checks, not native input/accessibility qualification.

Evidence:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-link-tooltip-20261015-9rZXuAOs/`.
