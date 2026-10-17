# Changes-first results — 2026-10-16

The user usually wants to review visual-regression failures rather than successful
comparisons. The comparison queue now defaults to Changed and explains **Only
changed results shown. Show all**. This is a local presentation change, not a
change to comparison thresholds, saved evidence, review decisions or baseline
consent.

- Saved comparisons, a run's first analysis and recorded-check inspection use the
  effective saved verdict, including valid reports behind legacy warning vetoes.
- Capture-only evidence stays unfiltered. Refresh/reanalysis of the same run
  preserves an explicit choice; a newly opened run gets its appropriate default.
- Show all preserves search, screen-size grouping, selection, image state and
  note/ignored-area drafts. It neither queues work nor saves anything. Focus
  returns to the persistent Filter action after the inline control disappears.
- **No changed results** is distinct from search's **No matches**. The unfiltered
  coverage strip retains real missing/not-compared counts; those results remain
  available through Show all and the existing Missing or not compared filter.
- The notice takes the queue heading's place; changes-only mode omits a duplicate
  shown count. No extra evidence-height strip, palette, asset or motion is added.

## Verification

Verification uses only synthetic saved corpus pairs, including real changed and
identical images with matching loading warnings plus a failed capture. The user's
project data is untouched. Regression coverage is in
`crates/app/src/changed_results_tests.rs`; existing full-queue navigation tests
now explicitly select All and keyboard expectations include the new action.

### Results and current blocker

- The initial **3 new GPUI regressions passed**, including normal/legacy verdicts,
  three desktop sizes, the no-change queue with a real failure, Show all,
  independent filters, draft/selection preservation and unchanged stored evidence.
- All **14 core shell/focus tests passed** with the new named control.
- The latest completed broad shell run had **217 passed, 6 failed, 1 ignored**.
  Changes-first tests and all five navigation tests passed. The failures were the
  known project-picker inset and discovery-disclosure tests, review-footer layout
  and multi-selection expectations, a capture-context wording expectation, and a
  narrow first-row fit assertion. This was not a whole-workspace pass.
- Renderer calibration passed with **1,024 exact pixels per colour**. Nine real
  GPUI/Blade frames (changed-only, Show all, and no-change, each at 1440×900,
  1280×800 and 820×600) were inspected together. They confirmed the interaction
  and exposed cramped narrow queues/redundant empty-state copy. A single repair
  batch removes the duplicate shown count, tightens narrow queue spacing and
  shortens the empty state. Those frames precede that final refinement.
- A fourth regression now also covers first analysis, same-run reanalysis and
  recorded-check defaults. Its final execution, the final compact-layout
  confirmation and production build are **blocked**, not claimed as passing:
  concurrent work replaced `ui/theme.rs`'s exported colour constants with a
  `Palette`, while shared controls still import the old constants. The current
  app build stops with unresolved theme symbols before compiling the shell tests.
  This task does not rewrite or revert that separate migration. One earlier
  capture-helper field mismatch was also being repaired concurrently.

**Scoped verdict: functionality verified before the last compact refinement;
final-source verification blocked by concurrent theme migration.** No final
visual or whole-app acceptance is claimed. Linux GPUI simulated input and Blade
offscreen rendering do not establish native OS-input, accessibility,
cross-platform usability or release qualification.

Private synthetic captures and calibration:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-changed-results-ynf4oiqp/`.
Frame sidecars retain the actual compiled source and PNG hashes; they are not
silently relabelled as final-source captures. No shipping raster was added.
