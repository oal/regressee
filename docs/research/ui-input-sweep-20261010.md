# Native input refinement — 2026-10-10

## Scope and evidence

Only `crates/app/src/ui/text_input.rs` and this note were changed. Direction:
Operate, native desktop, preserving the incumbent light/teal field appearance.
Read PRODUCT/DESIGN through Impeccable context, its hardening reference and craft
floor, and findings 10/20 of `ui-sweep-audit-20261010.md`. The context launcher's
mobile guidance does not apply to this desktop product.

API evidence is the installed **GPUI 0.2.2** source: `examples/input.rs` for
normalized/reversed selection and composition TextRuns, `src/elements/div.rs`
for pointer dispatch, `src/window.rs` for frame-scoped capture listeners and
content masks, and `src/app/test_context.rs` for regression-test APIs. Original
Apache-2.0 attribution, SPDX line and shared-input comment are retained.

No builds, tests, application/runtime launches, screenshots or subagents were
run by this implementation session. Only the owned Rust file was formatted with
rustfmt. This is implementation and source review, **not rendered, native-input
or accessibility acceptance**.

## Implemented

- Shift+Left/Right/Home/End extend from a stable anchor, including shrinking,
  crossing the anchor and reversing direction. Cmd+Shift+Left/Right complement
  the existing Cmd+Left/Right bindings. Unmodified navigation still collapses
  selection as before. Platform selection queries now report reversed direction.
- Left-button drag selection and anchored Shift-click. A paint-time capture
  listener handles moves outside the field; inside/outside mouse-up ends the
  drag. Lost focus, a missing pressed button, disabling, replacement and reset
  stop active dragging. Stale shaped text is not used to index edited content.
- Double-click selects a contiguous alphanumeric/underscore, whitespace or
  punctuation group, with valid UTF-8 boundaries and an empty-field guard.
- Marked IME composition is shaped with a one-pixel straight TextRun underline
  in the current text color. Prefix/suffix remain unmarked; unmark requests
  repaint. UTF-16 selection within composition retains its direction.
- Horizontal reveal follows the **active** selection endpoint, not always its
  sorted end. Text, caret, selection fill and composition decoration share an
  explicit content mask intersected with the existing outer clipping. Border,
  padding, font size and the 28px inner layout height do not change by state.

## Preserved

Existing actions and bindings remain. UTF-16-to-UTF-8 conversion still rounds an
interior surrogate offset forward and clamps out-of-range offsets to text end.
The existing byte budget, control-character rejection, disabled edit guard and
rejected-composition preservation remain. Text is not normalized or rewritten
by selection, word lookup or decoration. Copy still writes the exact selected
UTF-8 substring, **including an empty selection**; Cut follows its existing
copy/replace path. Paste retains its existing independent CR/LF-to-space rule.
No Cargo, theme, shared-control, main or core files were edited.

## Regression tests added / parent verification

All are under the existing `#[cfg(all(test, feature = "ui-tests"))]` module:

- `shift_selection_retains_anchor_across_direction_changes`: Unicode extension,
  anchor crossing, Home/End, collapse, UTF-16 direction and typed replacement.
- `mouse_selection_shift_click_word_and_outside_release`: shaped hit testing,
  forward/reversed drag, move/release outside, Shift-click, double-click and a
  disabled pointer guard.
- `composition_runs_limits_and_utf16_conversion_are_preserved`: surrogate and
  overlong conversion, reversed IME selection, marked-run byte lengths and
  underline, control/byte-limit refusal and removal of marking.
- `clipboard_keeps_exact_selected_bytes_and_existing_paste_rules`: Unicode copy,
  reversed selection, CR/LF paste, Cut and empty Copy semantics.
- `active_selection_edge_scrolls_without_changing_field_geometry`: long content,
  active-edge horizontal reveal and identical focused/selected/disabled bounds.

The two incumbent Unicode/action and IME rejection tests remain.

Parent's first serial run reported **25 app-library tests passed, one ignored,
and one failed**: the geometry test's first `debug_bounds("text-input")` lookup
returned `None`. All six other input tests passed in that run. This is parent
test evidence, not a native-input or accessibility qualification.

Source inspection confirmed that GPUI 0.2.2 registers measured bounds from
`InteractiveElement::debug_selector` during painting; an ordinary element `id`
does not populate that map. The field now provides that debug selector (a no-op
without GPUI test/test-support configuration). The geometry test now mounts its
160px wrapper as the actual window view instead of relying on a one-off draw
that root redraws can replace. It keeps all active-edge scroll and state-bounds
comparisons, additionally asserts the 160×38px outer field size, and requests
repaint explicitly when disabling the field. **This correction has not been
rerun by this session; parent owns the next serial check.**

Parent should run serially when its shared changes are ready:

```sh
cargo test -p regressee-app --features ui-tests --lib ui::text_input::tests -- --test-threads=1
```

Later native checks should cover keyboard/drag selection in a long URL, release
outside the field, Shift-click across the anchor, real platform IME replacement
and candidate positioning, and unchanged field bounds at OS text/display scales.
GPUI simulated tests do not substitute for those checks.

## Known limitations / finding 20

- Existing scalar-based Left/Right/Delete/Backspace behavior is retained; it is
  not grapheme-aware. Combining marks and joined emoji need a future segmentation
  pass. Double-click is a dependency-free fallback, not locale-aware word
  breaking; CJK runs may select together and combining marks may split words.
- Double-click-followed-by-drag extends by character position, not whole words;
  triple-click selection and Ctrl/Option word navigation are not added.
- Horizontal reveal advances on pointer movement; there is no timed edge
  autoscroll while the pointer remains stationary. Release outside the native
  window depends on platform event delivery and is unqualified here.
- Real IME behavior, bidirectional visual navigation/selection, caret blinking,
  high contrast, OS scaling, focus contrast and rendered underline placement
  remain native verification obligations.
- Oversize/control-character edits still refuse silently. This narrow input
  refinement does not add field-specific rejection messages or resize fields to
  display errors. Existing disabled opacity remains; legibility is unqualified.
- No screen-reader semantics were added and no screen-reader/accessibility pass
  is claimed. Finding 20 remains a separate qualification gap, not resolved by
  selection tests or source tokens.
