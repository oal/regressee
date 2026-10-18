# App-wide action hierarchy — 2026-10-18

## Approved scope

The user reported similar-looking buttons obscuring available actions, supplied
the DOM review screen, and requested an app-wide Impeccable analysis. Two isolated
assessments reviewed native source and existing renderer evidence. The full
critique is archived in
`.impeccable/critique/2026-10-18T11-29-48Z__crates-gui-src.md` (27/40 editorial
heuristic assessment, not usability certification). The web detector returned
`[]` on the all-Rust target; this does not establish native design coverage.

The user then selected **Full sweep** and **Adaptive labels**. This is a focused
Operate refinement of unified neutrals/teal, not a palette or workflow redesign.

## Implementation

- `ui/controls.rs`: shared flat underlined content Tab, connected Segment, quiet
  supporting command and document Anchor roles, alongside existing raised primary,
  secondary, danger, rail and descriptive form-choice primitives. Compact toggles
  share the existing drawn checkbox and keep focus distinct from selection.
- `action_hierarchy.rs`: guarded focus-only Left/Right/Home/End navigation for
  exclusive groups. Existing Enter/Space explicitly activates; Tab remains in
  painted reading order, with disabled/unmounted controls excluded. Arrowing over
  review decisions never records one. No new storage or command implementation.
- Evidence-kind tabs and DOM presentation segments replace raised choice buttons.
  DOM presentation moves into the full-width toolbar, recovering the formerly
  separate source-view row; `task_ux.rs` matches keyboard order to the new position
  without moving the complete-source commands inside Details.
- Screenshot and DOM review share grouped decision selectors, separate quiet
  Note and forward Next commands. Existing note eligibility is preserved.
- Baseline membership uses a checkbox with stable **Include After screenshot /
  DOM snapshot** value labels, rather than reversing the label when checked.
  Discovery similarly uses **Include page in capture**. Both footer variants explain future-reference consequences before
  selection; preview/confirmation/final-save and source/kind guards are unchanged.
- History content tabs differ from its segmented run-type filter. Project editing
  is quieter and separated from opening; visible deletion/confirmation remains.
  Native inspection also exposed the existing History hover layer covering row
  text. History now mounts its paint layer before transparent row content, keeping
  selection, focus, hover, row height and labels without the occlusion. The native
  matrix checks actual foreground pixels in both row text regions.
- Project settings keeps approved Jump to anchors over its continuous form;
  descriptive setup/App settings radios, disclosures, task/dialog footers and
  explicit save behavior remain. Sensitivity uses shared segments; discovery
  selected-page inclusion uses a checkbox, distinct from batch match commands.
- Toolbar labels appear at >=900px actual working width, or >=640px for DOM's
  smaller tool set. Compact tools retain enabled icon-only focus/hover names.
- Shared stroke Source and Lock icons distinguish DOM evidence and website
  access; History check opening uses Compare rather than Image. No icon library
  or dependency added. Consequential labels are not replaced with glyphs.
- Run check is secondary while unreviewed changes remain, returning to primary
  afterward. It remains available with exactly the same operation/consent guards.

## Verification plan and evidence

Use the finite ignored
`ui_renderer_capture::ui_action_hierarchy_capture::renderer_capture_action_hierarchy`
matrix at 1280×800 and 820×600 in both themes. It covers Projects, Project settings,
App settings, History runs/baselines, screenshot review/single image/Analysis,
page options, and DOM diff/source/baseline selection. Assert toolbar containment,
source visibility, new DOM reading order and unchanged persisted screenshot
evidence. Run focused keyboard, toolbar, DOM, review/baseline, settings and
disclosure regressions, plus the shared-control tests and production build.

Private evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-action-hierarchy-20261018-8u6_auqo/`.
At most one batched inspection, one fix batch and one confirmation are allowed.
The results below are bounded development-host checks, not release qualification.

### Initial bounded pass

- The 123 shared UI tests, two new group-navigation tests and five toolbar tests
  passed. The first combined runner hit its external 120-second wrapper deadline
  after those completed; the remaining suites were resumed with explicit bounded
  per-suite timeouts. This timeout was not a failing product assertion.
- DOM workflow (9), review footer (2), unchanged review (3), App settings (7),
  keyboard (2), page options (5), History (2), workflow refinement (21), removal
  (3), split view (2), and two Project settings tests passed.
- Two previously documented failures reproduced unchanged: Project settings'
  jump test expects `Guided(2)` instead of the existing
  `CaptureMode(ScreenshotsOnly)` destination (`project_settings_tests.rs:105`),
  and the discovery/search disclosure is unavailable in its test fixture
  (`disclosure_tests.rs:132`). See the earlier unresolved-gates record in
  `project-removal-20261018.md:108–128`. Neither assertion was weakened here.
- Production `ui` build and strict `ui-tests` library Clippy passed. Strict
  production-binary Clippy is blocked by unchanged
  `jobs.rs:564` (`infallible_destructuring_match`); allowing only that lint exposes
  the unchanged `appearance_ux.rs:115` (`manual_unwrap_or`). No unrelated lint
  source was repaired or global allowance added.
- The initial renderer invocation stopped before producing any frame because
  the harness needs `website-fixtures/`. Four existing synthetic fixture PNGs
  plus their provenance were then copied byte-exactly from the earlier search
  inspection; `fixture-reuse.json` records their hashes. No website was visited.
- The actual first visual batch passed its geometry/identity checks and produced
  48 GPUI/Blade frames, all inspected in four boards plus full-resolution DOM,
  History and compact-review frames. PNG hashes and ten relevant source inputs
  matched current source (`inspection-receipt.json`).
- One fix batch corrected checkbox value labels and the existing History text
  occlusion, and attached History's tab underline directly to its content rule.
  The sole confirmation adds an actual foreground-pixel assertion for History
  title/detail copy. No further UI polishing round is planned.

### Sole confirmation and finish

**Scoped PASS for the exercised action hierarchy and History text repair.**

- **188 focused tests passed**, with three existing opt-in tests ignored. This
  includes the shared UI library, new focus-only group navigation, all toolbar
  modes and navigation preferences, real-IPC DOM guards, review/baseline workflow,
  settings keyboard/scroll order, page options, History, removal and split view.
  The two known failing tests above were not counted as passing or rewritten.
- The final ignored native matrix passed and produced **48 final GPUI/Blade
  frames**, 12 scenes in both themes at 1280×800 and 820×600. Geometry, DOM reading
  order, minimum source visibility, unchanged saved screenshot evidence and actual
  History title/detail foreground pixels passed their assertions. All four final
  boards and full-resolution History/light/large and DOM-selected/dark/compact
  frames were inspected. History labels are readable; stable checkbox labels,
  separate selectors/commands and responsive toolbar treatments are visible.
- Final production-feature build and strict shared-UI-library Clippy passed.
  No renderer/test features are required by the production app. The unrelated
  strict production-binary lint blockers above remain.
- `confirmation-test-results.json`, `confirmation.log`, the production/lint logs,
  `confirmation-receipt.json` and `confirmation-boards/` retain the final evidence.
  The 48 PNG hashes, dimensions, opacity and synthetic/native-render flags verify.
  Of 816 targeted embedded-source comparisons, 720 match current source; the 96
  differences are `main.rs` and `action_icons.rs`, edited by concurrent integration
  work after the capture build. A wider 115-input comparison also finds concurrent
  drift in `agent_integrations_ux.rs`, `form_recovery.rs` and core `shell.rs`;
  `concurrent-source-drift.json` records it. All 15 other targeted inputs match.
  Thus the frames bind the tested build, **not the entire latest working tree**.
  Concurrent changes were preserved; no extra render/polish loop was added.
- Formatting checks on changed native files found only concurrent, unformatted
  Agent action branches in `main.rs`; those unrelated edits were not reformatted.
  The scoped files were formatted and `git diff --check` passed before finish.

The visual budget is complete: one real inspection, one batched correction and
one confirmation. No new shipping raster, dependency or user-data operation.

## Qualification limits

Only isolated synthetic projects and existing local IPC test engines are used.
No user website/project/login is visited, selected, rewritten, approved or deleted.
Source/simulated input/offscreen Linux GPUI pixels do not establish native OS
input/clipboard, screen-reader/accessibility, Windows/macOS or release readiness.
Pre-existing DOM History loading and other documented test failures are not
implicitly fixed or requalified. Concurrent unrelated app/CLI work is preserved.
