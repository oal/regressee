# Shared dialogs — 2026-10-15

## Outcome

The reported baseline dialog was GPUI's fallback renderer, not the app's shared
controls. It now uses the existing Clear wayfinding typography, spacing, teal
primary actions, secondary controls and error-colored destructive actions.

`crates/app/src/ui/dialog.rs` owns the common scrim, surface, heading, footer
and confirmation renderer. About/help, selective approval and task dialogs use
the same primitives. Native file/save pickers are unchanged. Confirmation roles
are declared at the five prompt call sites; no label matching, new dependency,
backend operation or consent-copy replacement was introduced.

Confirmations have 24px insets, 16px section separation, a 560px maximum width,
21px/28px headings and 15px/22px body text. Label-sized 40px-minimum actions sit
in a ruled footer. Long prose scrolls with a visible interactive overflow thumb,
wheel and arrow/Page/Home/End input, independently of the actions. All dialogs
retain at least 16px clearance from the supported window edges.

Safe dismissal receives initial focus. Tab and Shift-Tab wrap within the dialog,
Enter/Space activate the focused choice, Escape cancels and focus returns to the
trigger. Outside clicks neither dismiss nor activate the workspace. Quit's
Discard / Cancel / Save layout preserves the existing answer-index mapping.
Selective approval also starts on Keep reviewing; approval eligibility, stale
preview guards and explicit confirmation remain unchanged.

## Bounded visual review

One batched Linux GPUI/Blade offscreen inspection covered eight states at both
1280×800 and 820×600: whole baseline, partial baseline, unsaved-edit discard,
three-way quit, About, keyboard help, selective approval and a local-storage task.
The sixteen frames show readable complete content, attached action groups,
visible safe focus, consistent surfaces and no dialog/control clipping. The
keyboard-help body was increased to 360px when its new shared type metrics made
the previous 320px cap too short; the existing full-copy bounds test now passes.
No visual polishing loop or decorative assets were added.

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-dialogs-20261015-Lb7Oy8/`.
Each original frame has a source-fingerprinted JSON receipt. Captures come from
isolated synthetic offline fixtures, not actual customer projects.

## Verification

- Native UI `cargo check --locked -p regressee-app --features ui`: passed.
- Native executable build and app library/binary Clippy with warnings denied:
  passed. Formatting and whitespace checks passed.
- App library: 123 passed, two existing opt-in tests ignored (before the final
  pointer/wheel regression additions).
- Actual styled baseline flow: safe Enter and Escape do not save; Tab/Enter
  explicitly saves the partial baseline. Background actions/quit stay guarded,
  and focus restores to Save baseline.
- Shared confirmation tests cover action roles, both desktop sizes, target sizes,
  focus cycling, three-way answer indices, cancellation, long-copy scrolling and
  pinned actions. All four final shared-confirmation tests passed, including
  pointer activation and wheel scrolling.
- Existing About/link focus, shell dialog restoration and complete keyboard-help
  bounds tests passed. The sixteen-state renderer test passed.
- An initial full binary test attempt exceeded its 120-second command limit while
  running an unrelated setup-capability test; it was not reported as a pass.
- The completed four-thread binary suite reported **197 passed, 1 failed,
  45 ignored**. The failure is the concurrently added
  `keyboard_navigation_tests::keyboard_choices_checks_and_text_keep_one_activation_owner`
  missing `check-Guided(2)` bounds on the Pages setup screen, not a dialog path.
  That unrelated test/source was not changed by this task; no whole-suite pass
  is claimed.

Scoped disposition: the shared dialog treatment passes the bounded visual review.
Linux offscreen rendering and synthetic event tests are not OS-input,
screen-reader, Windows/macOS or public-release qualification. The app's existing
native accessibility qualification limits remain. Concurrent unrelated edits
were preserved.
