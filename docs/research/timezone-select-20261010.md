# Native timezone select — 2026-10-10

## Scope and direction

Operate mode, extending the existing Regressee identity rather than rebranding.
The chosen value stays left-aligned in a 40px field, with an authored stroked
chevron at right. Its bounded native dropdown contains a separately labeled
search input and actual IANA identifiers. Current selection is marked with
“Current” and the shared selected palette; the keyboard candidate has an ink
outline. The shared palette, system font, 16px field type, stable line height,
4px corners, 12px horizontal inset and existing focus outline are inherited.
No modal, web view, decorative motion or full-form layout expansion is used.
PRODUCT.md, DESIGN.md, incumbent controls/input/theme and Impeccable craft floor
were read. Native rendering and human visual review are parent-owned follow-up,
not a qualification result from this implementation session.

## Files and integration API

- `crates/app/src/ui/timezone_select.rs`
- `crates/app/assets/timezones.txt`
- This research note. No existing module exports, Cargo files, parent shell,
  core validation, native adapters, limits or privacy behavior were changed.

Public API:

```rust
pub struct TimezoneSelect { pub selected: SharedString, /* private state */ }
impl TimezoneSelect {
    pub fn new(value: &str, cx: &mut Context<Self>) -> Self;
    pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>);
    pub fn set_enabled(&mut self, enabled: bool);
    pub fn focus_handle(&self, cx: &App) -> FocusHandle;
    pub fn bind(cx: &mut App);
}
pub fn bind(cx: &mut App); // equivalent module-level convenience
```

Parent exports the module and calls `timezone_select::bind(cx)` **after**
`text_input::bind(cx)`. Home/End in the popup search must have the selector's
binding precedence over the ordinary input's caret bindings. Enter/Escape have
nearer selector/search contexts than Shell, so choosing never becomes a shell
activation. Ordinary input editing, clipboard, Shift+Home/End and IME plumbing
remain those of the existing TextInput.

Render an `Entity<TimezoneSelect>` where the old timezone input was rendered.
The authoritative profile value is `.selected`, not the private search query.
Observe the selector entity; choosing an option calls `cx.notify()`. Opening,
filtering and dismissal also notify for paint/focus state, so read/compare the
value rather than treating every notification as a profile change. No schedule,
capture, save or job starts inside the selector.

Use `focus_handle(cx)` when building/evaluating the parent's active focus order:
it returns search focus when open and trigger focus when closed. Tab/Shift+Tab
are deliberately left to the parent's FocusState. Escape and committing a
selection restore trigger focus. Blur and outside pointer dismissal do not
restore focus or consume the outside event. Disabling immediately removes tab
stops and closes state; the caller should notify, as with shared controls.
The next render disables the private input and clears focus only if it is still
on this selector, never an unrelated newly focused control. Re-enabling does
not auto-open, auto-focus or edit the selected value.

## Bundled authoritative catalog and license

The asset contains **436 Zone identifiers plus the explicitly requested UTC**
(437 distinct options). Source: the installed `/usr/share/zoneinfo/tzdata.zi`,
which declares:

```text
# version 2026c
# redo posix_only
# ddeps backzone zone.tab
# This zic input file is in the public domain.
```

Source SHA-256:
`af5c1d3bebe136d372c131bb1a45725f955a8cc2a5ae2fc5a31d3b372e145f49`.

Authority: IANA Time Zone Database, <https://www.iana.org/time-zones>.
This is provenance for the locally installed data, not a claim that an upstream
release was downloaded or independently qualified in this session. Catalog
identifiers are public-domain data; the selector Rust source is MPL-2.0, and the
existing TextInput retains its Apache-2.0 attribution.

Derivation: select only records whose first token is `Z` (compact zic's `Zone`),
take the second token verbatim, deduplicate, add `UTC`, and sort. `L`/`Link`
aliases, Rules, host filenames, `posix/`/`right/` duplicate trees, user paths,
`localtime` and `posixrules` are not scanned or included. `UTC` is the deliberate
exception: the source explicitly links `Etc/UTC` to `UTC`. Backzone Zone records
are retained, including `Europe/Oslo`, rather than losing distinct historical
identifiers through zone1970's post-1970 consolidation. Etc fixed-offset Zones
and the source's `Factory` placeholder are retained rather than silently
inventing a narrower definition of the installed canonical Zone catalog.

The asset is `include_str!`-bundled on every OS. There is **no runtime access to
the host zoneinfo directory**, system timezone, process environment, network or
an external database. The catalog carries identifiers only, not offset or
transition calculations. Runtime display puts UTC first, then all other names
alphabetically. To refresh, regenerate from the desired public-domain IANA zic
input and record its version, build header, hash, count and provenance here.

## Interaction and compatibility

- Click, Enter, Space or an arrow opens the select. Open state focuses search.
- Search is case-insensitive matching of every whitespace-separated token;
  underscores and spaces are equivalent, so “buenos aires” finds its actual
  `America/Argentina/Buenos_Aires` identifier. Query is limited to 256 UTF-8
  bytes by the shared input. Search never changes `.selected`.
- Up/Down move the candidate; Home/End select the first/last candidate. Movement
  clamps at either end, uses native uniform-list scrolling to reveal candidates,
  and never commits until Enter or a pointer selection.
- An empty result displays “No matching timezones”; Enter cannot commit arbitrary
  search text. Reopening clears the query and highlights the known current
  option, or UTC as the first candidate when the historical value is unknown.
- `new` and `set_value` preserve unknown/legacy values exactly. They are shown
  with “Not in list”; presentation is capped at 256 Unicode scalars and replaces
  control/bidi-formatting characters without modifying stored bytes. No alias
  canonicalization or default substitution is performed. New pointer/keyboard
  selections can only be identifiers from the bundled catalog.
- The popup is generated in canvas prepaint from **this frame's** trigger bounds,
  then laid out through GPUI `anchored` plus `deferred`. It retains key ancestry,
  escapes ancestor content clips, chooses above/below according to available
  space, clamps width to the viewport, and bounds list height to 320px (eight
  40px rows). `uniform_list` virtualizes the complete catalog.

## Authored tests and limits

Tests are gated with `all(test, feature = "ui-tests")`. GPUI tests cover keyboard
selection against a competing Shell Enter binding, case-insensitive search,
no-match safety, pointer selection and entity notifications, mounted 40px trigger
and bounded popup geometry through a clipped form ancestor, virtualized End
navigation, Tab/Shift+Tab focus integration, blur/outside dismissal, unknown
value preservation and disabling an open dropdown. Built-in tests use
`core::prelude::v1::test`; there is no wildcard macro import of `gpui::test`.
Catalog and pure geometry tests cover count/uniqueness, multiple world regions,
canonical-versus-alias examples, token matching, above/below placement and
viewport bounds.

**No tests, build, native launch, browser, download or subagent were run.** The
parent owns serialized verification. Tests are test-platform/source contracts,
not native OS IME, accessibility, screen-reader, high-contrast, font-resolution
or Windows/macOS/Linux visual/input qualification. GPUI div primitives do not
add a new qualified OS combobox accessibility bridge here.

The supported native desktop minimum window is 820×600. Smaller geometry shrinks
the list; extremely tiny viewports with no space for the 80px search header are
not claimed to remain usable. There is no typeahead on the closed trigger:
first open, then search. No city translations or geographic display aliases are
invented. Newer IANA identifiers (and `Factory`) can differ from the capture
browser's ICU/timezone support; catalog membership is **not** a guarantee that
the pinned browser accepts every entry. Existing core/runtime timezone validation
and capture error reporting must remain authoritative and unchanged by this
component. A bundled list does not claim runtime timezone-rule parity.
