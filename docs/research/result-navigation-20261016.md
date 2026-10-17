# Stable result navigation — 2026-10-16

## Reported problems and repair

The user reported Compare/tools disappearing briefly while moving between results,
and the queue failing to follow the selected row when using its up/down buttons.
This is a narrow behavior repair within the existing native Operate surface.

- The existing inspection toolbar container stayed mounted, but its View, Zoom,
  image-help and Analysis mount predicates still required the loaded viewer.
  Presentation now accepts the current identity-bound pending read independently
  of activation. Image-dependent actions remain disabled until a frame is present.
- A second queue-owned Results toggle appeared whenever the frame was absent,
  even when the normal toolbar already owned it. It no longer appears during
  image reads, avoiding queue/header displacement and duplicate control identity.
- Eligible review/baseline actions retain their layout while reading and remain
  guarded. This also prevents the footer resizing the queue when a read completes.
- Split slider keeps its selected name/icon while loading; genuine missing-image
  fallback still depends on the accepted frame, not absence of pixels in transit.
- Arrow navigation, direct row selection and Next unreviewed request a non-strict
  reveal in the actual filtered/grouped list. Offscreen rows reveal at the nearest
  edge; visible rows do not recenter. Shorter-than-one-row viewports reveal the row's
  beginning. Focus remains on the navigation button. No render-time scroll reset
  fights subsequent manual scrolling.

No capture, pixel-analysis, database, cache, draft or baseline-consent policy was
changed. Previous-target screenshots are not held under the new target's heading.

## Verification

The strengthened existing screenshot-switch regression reproduced the missing
toolbar controls before the repair. The new queue regression reproduced a selected
row extending below its viewport before reveal was added.

- **5 new regression tests passed**: all six mode toolbars during reads; complete
  25-result traversal in both directions, including filtering/grouping; rapid
  navigation before replies arrive; distant Next unreviewed and wraparound;
  minimal reveal and preservation of subsequent manual scrolling.
- **21 workflow-refinement tests passed**, including loading/failure identity,
  stale-read rejection, missing-image recovery, draft protection and consent.
- **11 related toolbar, evidence-layout and review-footer tests passed** against
  the final source, including focus order, all view modes and approval guards.
- Native production build passed with the pinned Rust 1.98.1 toolchain.
- One bounded native GPUI/Blade visual batch passed: **18 PNGs**, loading/ready
  pairs for Compare, Single image and Split slider at 1440×900, 1280×800 and
  820×600. All three contact sheets were inspected. Controls remain in position;
  only image-dependent enabled state and the screenshot stage change. No visual
  correction round was needed.
- A source receipt confirms every captured frame matches the current main,
  evidence-workspace, review-UX and workflow sources. Existing capture sidecars
  preserve PNG hashes, native-renderer flags and additional source hashes.
- `git diff --check` passed. No shipping raster, asset or design token was added.

Private diagnostic root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-result-navigation-acmbtuy_/`.
Frames and `navigation-source-receipt.json` are in `inspection/`. Synthetic saved
corpus pixels are used, never the user's project or website captures. Linker aliases
for already installed libraries live only in that private root; existing project
symlinks and system packages were left untouched.

**Scoped verdict: pass.** This is in-thread Linux simulated-input and offscreen
rendering verification, not native OS-input, accessibility, cross-platform,
whole-app or release acceptance. Concurrent unrelated edits are preserved.
The broader shell run finished with **214 passed, 2 failed, 1 ignored**. Its failures
were `disclosure_tests::all_inline_disclosures_attach_content_restore_focus_and_do_not_mutate`
(the search disclosure's enabled-state assertion) and
`tests::project_selection_actions_and_long_names_fit_supported_window_sizes`
(Change project's right-edge assertion), outside this repair. That run started
before the additional manual-scroll regression was added; the final five navigation
tests passed separately. Full log: `shell-tests.log` in the diagnostic root.
No full-suite pass is claimed.
