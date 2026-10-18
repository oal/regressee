# Review workspace contrast — 2026-10-18

## Scope and implementation

The user approved proceeding from the focused Impeccable contrast assessment.
Start with the review workspace and restrained, clearer hierarchy; preserve
unified neutrals, teal, system typography, existing layout and interaction safety.
This is not a new app-wide visual identity or a redesign of the ongoing workspace
changes already present in the working tree.

- `ui/theme.rs`: add `review_ground` (`#ededed` / `#111111`) and
  `result_selected` (`#ccebe3` / `#244c43`). The bounded Results workspace alone
  uses the review ground. Other screens, navigation and global chrome retain
  their existing tokens.
- `ui/controls.rs`: selected result rows have stronger mint, a 1px teal leading
  marker and semibold path. The marker is absolute paint; text insets, 84px
  virtual-row height and hit targets do not move. Ordinary page and History
  selection remain unchanged. Keyboard focus remains a separate existing ring.
- Screenshot viewports, DOM source and screenshot loading/missing-image frames
  reuse `control_line` for their outer boundary. Internal decorative separators
  retain `line`. No screenshot pixels, image transforms, source text or DOM
  syntax/search/selection colors are changed.
- `dom_ux.rs`: paired-kind status retains both outcomes and the same wording,
  emphasizing only the active kind's value with primary semibold ink. Plain
  zero totals remain supporting text; meaningful nonzero totals retain emphasis.
- Targeted DESIGN.md, native-role sidecar and surface-brief additions document
  these roles without regenerating unrelated design metadata or repairing drift.

No dependency, bundled font, shipping raster, new control, motion, saved status,
review decision, capture operation or approval behavior was introduced.

## Contrast and geometry checks

Token ratios (light / dark):

| Relationship | Before | After |
|---|---|---|
| Evidence surface / workspace ground | 1.10 / 1.10 | 1.17 / 1.20 |
| Selected result fill / evidence surface | 1.13 / 1.35 | 1.27 / 1.64 |
| Evidence outer boundary / evidence surface | 1.45 / 1.45 | 3.69 / 3.98 |

The new selection's supporting ink remains at 5.52:1 light / 4.83:1 dark.
Primary, supporting and interaction text on the review ground meet 4.5:1.
The marker meets 3:1 against the selected fill. Surface-to-surface ratios are
perceptual-hierarchy evidence, not a claim that every decorative surface must
meet 3:1. Existing primary/body and DOM-syntax contrast tests remain intact.

The added native row test covers selected/unselected, enabled/disabled and both
themes. It verifies unchanged row/text bounds, marker width and containment, and
no marker on ordinary page rows. The review-workspace integration test also
checks the actual selected result's marker and the paired outcome groups' bounds.

## Verification

Commands use `mise exec rust@1.98.1 --` and the existing host-only link aliases at
`/var/home/olav/Data/opencode/tmp/opencode/regressee-native-motion-20261018-r7qx28e2/link-libs`
through `LIBRARY_PATH`.

- `cargo test -p regressee-gui --features ui-renderer-capture,ui-inspection
  --lib ui:: -- --test-threads=1`: **125 passed**.
- The same feature set with `--bin regressee-desktop review_workspace_tests`:
  **1 passed**, in both the initial inspection and final confirmation. This
  exercises real local IPC and existing explicit-consent/draft/focus safeguards;
  the fixture stores remain unchanged.
- `dom_workflow_tests`: **9 passed**; `review_footer_tests`: **2 passed**.
- `toolbar_tests`: **5 passed**; `unchanged_review_tests`: **3 passed**.
- `result_navigation_tests`: **4 passed, 1 failed, 1 explicitly ignored**;
  see the existing expectation mismatch below. Do not count this as a clean suite.
- Production-feature build, `cargo build -p regressee-gui --features ui
  --bin regressee-desktop`: **passed**. Test/renderer features are not required.
- Scoped Rustfmt checks for `ui/theme.rs`, `ui/controls.rs` and the workspace
  test file pass. `git diff --check` and the new tokens' DESIGN/native-role
  mapping consistency checks pass.

The first combined validation wrapper reached its 120-second limit after DOM and
footer checks completed. The remaining scopes and production build completed in
a second wrapper without that time limit. Results and individual logs are in
`verification.json` under the evidence root.

### Existing navigation-test expectation mismatch

`result_navigation_tests::every_view_mode_keeps_its_toolbar_controls_in_place_while_loading`
fails at `result_navigation_tests.rs:339` while unwrapping an expected control's
bounds. Its unchanged control list still includes `action-Phase5(23)` (line 326),
the baseline-inclusion checkbox formerly in the persistent footer. Current
`main.rs` mounts that control only when `baseline_update_open` is true; opening
the on-demand inspector is intentionally absent from this loading-toolbar test.

This conflicts with the workspace work already present before the contrast pass:
the initial `review_workspace_tests.rs:94` explicitly asserts that this checkbox
is not mounted before opening the panel. The passing disclosure matrix retains
that assertion and exercises inclusion after opening. The navigation test file
has no working-tree diff, and the contrast pass does not change control mounting
or the baseline workflow. This is source evidence of an outdated expectation,
not a claim of a separately executed pre-change baseline. No unrelated test or
baseline behavior was rewritten to conceal the failure.

For `preview.10` release preparation, the loading-toolbar test was subsequently
updated to inspect the visible `BaselineUpdate` disclosure instead of the hidden
inclusion checkbox. The presentation-only disclosure remains available while
images load; image-dependent actions retain their disabled-state assertions.
The failure recorded above describes the earlier contrast-only verification.

## Native visual evidence

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-review-contrast-20261018-0kf_lmte/`.

Each of `inspection/` and `confirmation/` contains 60 Linux GPUI/Blade native
PNG/JSON pairs: screenshot and DOM review, light and dark, 1280×800 with navigation
shown, and 820×600 with navigation shown/hidden. The matrix includes closed,
open, selected-reference and confirmation states plus long profile names.

The single batched visual inspection used two contact sheets covering all eight
navigation-shown kind/theme/size combinations and full-resolution light/large
and dark/compact DOM frames. Selection, frame boundaries, supporting copy,
outcome wrapping and evidence space were inspected. No visual repair was needed.
Only formatting of an added test closure changed afterward. The final
confirmation's **60 PNGs are byte-identical** to the corresponding inspection
frames, so the inspected pixels are also the final pixels.

`confirmation-receipt.json` checks all 60 final PNG hashes, dimensions and opacity,
and **600 current-source hash matches** across ten relevant source/test files.
The initial receipt also records exact teal marker pixels in light/large and
dark/compact frames, rather than relying solely on debug rectangles. Logs and
`contrast.json` accompany the receipts. These are synthetic verification fixtures,
not shipping images or real user-project results.

## Limits

This is scoped source/model, simulated-native-interaction and Linux offscreen
visual verification. It does not qualify OS-presented input, native accessibility,
screen readers, Windows/macOS presentation, monitor conditions, user acceptance
or the whole application for release. Existing unrelated work and user projects,
websites, stored evidence, login state and approval authority remain untouched.
