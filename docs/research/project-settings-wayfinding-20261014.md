# Project settings wayfinding — 2026-10-14

## User-confirmed scope

A user test found that Project settings appeared to end above the fixed Save
footer: lower settings were clipped without a visible scrollbar. The user chose
section jumps plus an interactive scrollbar rather than separate tabs, and asked
to align Tab order with the painted form. The navy/teal Clear wayfinding identity,
one draft, explicit saving and all backend operations remain unchanged.

## Implementation

- `crates/app/src/project_settings_ux.rs` owns the pinned **Jump to** row:
  **Website**, **Capture plan**, **Access & environments**. All sections remain
  mounted in one continuous form. The current destination follows the actual
  reading position; measured content coordinates handle scrolling, resizing and
  disclosure height changes. No section navigation dispatches worker operations.
- An explicit jump focuses the first enabled destination control, so subsequent
  Tab continues locally. The sole selected Desktop size cannot be unchecked;
  Capture plan therefore focuses Tablet in that state, not the disabled Desktop
  control. Passive scrolling never steals focus or edits a field.
- `crates/app/src/ui/scrollbar.rs` attaches to the existing `screen_scroll` owner.
  A reserved 16px gutter carries an 8px thumb with a 32px minimum height, painted
  whenever measured overflow exists. Track clicks page 90% of the viewport;
  dragging remains captured outside the gutter and clamps to both ends. Release
  outside ends the drag. Wheel scrolling works over the gutter as well as the
  existing form. Geometry is measured after scroll layout; no-overflow layouts
  mount no interactive canvas hitbox or painted track/thumb.
- `crates/core/src/shell.rs` introduces separate local `SettingsSection`
  destinations, not guided setup stages. Capture and access candidates now follow
  painted order, with disclosed children beside their trigger and footer actions
  last. Guided stages and busy/modal/picker guards still remove unavailable
  controls. New section handles use their own range.
- `crates/app/src/main.rs` pins the settings heading/navigation above the form
  and retains Save below it. The narrow form deducts the gutter before applying
  its 680px cap, preserving the 24px right inset. Capture details now sit inside
  the capture group before the separate access group.

No new key handlers intercept field-editing shortcuts. Existing whole-control
validation reveal, draft retention, explicit login/discovery, baseline consent,
storage and privacy protections are unchanged. No shipping raster or normative
palette/type/control-token change was introduced.

## Bounded verification

Final command:

```sh
LIBRARY_PATH="$PWD/.runtime/link-libs" \
TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
cargo test --locked -p regressee-core -p regressee-app \
  --features regressee-app/ui-tests --lib --bin regressee -- --test-threads=1
```

- App library: **116 passed, 2 ignored**.
- Native shell/controller suite: **183 passed, 1 ignored**.
- Core library: **53 passed**. Total: **352 passed**, no failures.
- The three new simulated GPUI tests cover section destinations and next Tab,
  pinned navigation/footer, narrow insets, retained drafts/unchanged stored
  evidence/no worker dispatch, track paging, outside dragging/release, wheel,
  no-overflow resize and forward/reverse traversal at both desktop sizes.
- Independent core adjacency assertions cover capture/access order; scrollbar
  geometry tests cover proportional extent, minimum thumb and drag clamping.
- The first full UI run exposed an old assertion requiring capture details after
  login help. It now verifies the intentional new grouping: login help remains
  within Access and capture details precede that group. The final suite passes.
- Production build passed with `cargo build --locked -p regressee-app --features
  ui --bin regressee` using the same local link-library environment.
- Opt-in `renderer_capture_project_settings` passed in two bounded batches.
  Eight final native GPUI/Blade PNGs and source-hash sidecars cover Website,
  Capture, Access and expanded-capture starts at **1280×800** and **820×600**.
  The expanded scene demonstrates the section start/extent, not lower advanced
  fields; traversal tests cover their reachability.

Evidence lives in `.impeccable/review/project-settings-20261014/inspection/` and
`confirmation/`, also under the authorized private root
`/var/home/olav/Data/opencode/tmp/opencode/regressee-project-settings-20261014-SYzRpMzf/`.
The original review is `.impeccable/critique/2026-10-14T18-59-01Z__crates-app-src-main-rs.md`.
The initial isolated design and mechanical reviews identified missing overflow
affordances, undisclosed sections and a painted-versus-keyboard order mismatch.
The initial native finish review additionally requested enabled jump focus,
explicit Jump to wording and gutter-aware form width. Its verdict scored all
three **resolved**, disposition **ship** for those findings. Independent
documentation verification confirmed an ordinary extension using the incumbent
system. `DESIGN.md` records the interaction; pre-existing design-sidecar drift
was reported and deliberately not repaired.

These are offline synthetic fixtures, simulated controller/input checks and
Linux offscreen layout evidence—not real-site usability, physical OS input,
screen-reader, Windows/macOS, clean-install or public-release qualification.
The existing upstream future-Rust compatibility warning remains. Hosted preview
builds are separate from these local verification results.
