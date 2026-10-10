# Shared controls sweep — 2026-10-10

Scope: `crates/app/src/ui/controls.rs` and `theme.rs`. This is a source-level
implementation and regression-test handoff, not a native visual approval or an
accessibility qualification. No application, browser, worker, build or test suite
was launched for this subtask. Existing historical evidence is unchanged.

## Direction and incumbent evidence

Read `PRODUCT.md`, `DESIGN.md` and `docs/foundation-surface.md` through the
Impeccable context loader; read the polish playbook and craft floor before edits.
The incumbent is an Operate-mode native desktop interface: cool light surfaces,
deep blue-green ink, teal action/selection, precise small-radius controls and
horizontal rules. Preserve that identity rather than add dashboard cards,
illustrations, metric tiles or decorative motion. The foundation surface brief
records unavailable meaningful native pixels; no committed screenshot fixtures
were found under `docs`. Source is the inspection evidence for this subtask.

## Exact focus growth cause: pinned GPUI 0.2.2

Inspected the locally cached, pinned `gpui-0.2.2`,
`gpui_derive_refineable-0.2.2` and `gpui_refineable-0.2.2` source:

- GPUI `src/style.rs:254–255`: `Style.text: TextStyleRefinement` **does not**
  carry `#[refineable]`, unlike size, padding and border widths. The generated
  `StyleRefinement.text` is an `Option<TextStyleRefinement>`.
- `gpui_derive_refineable/src/derive_refineable.rs`: non-recursively-refineable
  fields are assigned/replaced when an incoming refinement supplies a value;
  the individual properties of the nested text refinement are not merged.
- GPUI `src/styled.rs:387–398`: `.text_color(...)` creates a fresh default
  `TextStyleRefinement` when a state has no text refinement, then sets only color.
- GPUI `src/elements/div.rs:1018–1025`: `.focus(...)` receives a fresh default
  `StyleRefinement`, **not** the base control style.
- GPUI `src/elements/div.rs:2479–2557`: the renderer merges base, focus, hover,
  then active styles in that order.

The former control base specified 14px/medium. Its focus callback specified
only an ink text color (plus border and background). On focus, that color-only
text refinement replaced the entire 14px/medium refinement. Missing size and
weight then inherited the shell's 15px/normal text. Inherited proportional line
height also increased; text shaping/wrapping and the control's natural height
could change. Separately, the old callback replaced teal primary fill with pale
selection fill, flattening primary-action hierarchy.

`pinned_gpui_old_color_only_focus_replaces_the_entire_text_refinement` recreates
this with GPUI's real `Refineable` implementations, asserting loss of font
properties, inheritance of 15px/normal, and increased resolved line height.
It is an authored regression test, not a reported passing run.

## Shared implementation

- Action base: minimum 40px target, explicit 14px/medium text, fixed 20px line
  height, 12px horizontal / 8px vertical inset, invariant 1px border and 4px
  corners. Long labels may wrap naturally; interactions do not change wrapping.
- All state refinements are paint-only. No focus/hover/active text properties,
  border widths, font weights, padding or dimensions change.
- Focus: ink border plus a 2px exterior ink outline implemented with GPUI's
  zero-blur `BoxShadow` paint. This is a functional keyboard-focus outline,
  not decorative elevation. It changes neither layout nor selection. Hover and
  active modify only fill, so they cannot erase focus when merged later.
- Primary remains teal with white action text on focus, and deeper teal on
  hover/active. `Danger` is outlined error ink with error-surface hover/active;
  it is intentionally not another filled primary action.
- Disabled controls attach no focus tracking, cursor, hover or active styles.
  Their base metrics remain intact; opacity communicates availability. Callers
  still **must guard their click and action handlers** with the enabled state.

### Parent integration API (unchanged from requested signatures)

```rust
controls::choice(id, label, description, &focus, selected, enabled)
    // -> Stateful<Div>; callers attach click/actions
controls::check(id, label, &focus, checked, enabled)
    // -> Stateful<Div>; callers attach click/actions
controls::section(title, help)
    // -> Div; append children, no enclosing card
```

`choice` is a single-choice row with an always-visible 18px outlined radio
circle and an 8px teal dot only when selected. The label is invariant; optional
description uses explicit 13px/normal text and 18px line height. Selected
description is blue-green, not neutral gray on the teal surface. Multiline copy
is allowed to determine natural row height; selection/focus do not alter it.
Group exclusivity, focus order and activation remain caller responsibilities.

`check` uses an always-visible 18px square, teal fill and a white check stroke
when checked. The tick is a GPUI `PathBuilder` stroke painted by `canvas`, not a
Unicode glyph, emoji, font symbol or external asset. Its 12px drawing canvas has
constant geometry. Selected/checked surfaces are pale teal; focus independently
adds an ink outline. Labels do not flip between state/action text.

`section` provides a restrained top rule, 18px semibold heading, bounded helper
copy and space for appended controls. No background card or shadow.

Additional action variants are `ActionStyle::Danger` and
`ActionStyle::Choice { selected: bool }` (for concise tabs/choices that do not
need a described radio row). The exhaustive local color match covers both.
Only five reusable metric tokens were added to `theme.rs`; the existing palette
is unchanged. Neither backend nor screen-reader semantics are claimed.

## Verification handoff (parent owns execution)

Authored in `controls.rs`:

1. `pinned_gpui_old_color_only_focus_replaces_the_entire_text_refinement` — exact
   old failure with real merged style and inherited text.
2. `all_action_state_combinations_preserve_merged_text_and_layout` — seven
   variants × eight focus/hover/active combinations, GPUI merge order, resolved
   text equality, unchanged layout properties, surviving focus paint, preserved
   primary fill/text, and disabled metric equality.
3. `selection_and_description_metrics_do_not_depend_on_selection_or_focus` —
   merged row, description and indicator geometry across selection/focus.
4. `drawn_check_geometry_is_bounded_and_translates_with_the_canvas` — the actual
   tick points fit including half-stroke, translate with the canvas, and build
   successfully through GPUI's path tessellator.
5. With `ui-tests`,
   `actual_control_bounds_survive_focus_selection_and_disabled_states` — draws
   actual public action/radio/checkbox APIs in GPUI's test platform under a
   15px/normal ancestor; checks real element bounds while moving focus, changing
   selection and disabling controls, including wrapped description/label copy.

Suggested serial execution from the repository root:

```sh
cargo test -p regressee-app --bin regressee --features ui-tests ui::controls:: -- --test-threads=1
```

Formatting was performed with `rustfmt` on the two owned Rust files. Compilation
and all tests remain **unrun in this subtask**. Parent should run its normal
serialized compile/check gate and the filtered regressions above, then its
bounded native visual inspection. Inspect all states at representative desktop
widths, particularly focus-outline clipping near scroll containers, long labels,
multiline descriptions, selection versus focus and destructive-action hierarchy.
GPUI test-platform bounds and source merges are not a human visual/native-input
or screen-reader pass.
