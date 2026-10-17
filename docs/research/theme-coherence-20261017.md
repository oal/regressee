# Unified neutral themes — 2026-10-17

## User-confirmed scope

The user rejected the actual native app's dark slate workspace against the blue
sidebar and its gradient header, pointing to the header and rail/workspace
boundary. They selected **Unified neutrals**: one charcoal working environment
in dark mode and off-white/soft-gray equivalents in light mode, with teal/mint
reserved for action and selection. Warning/error roles remain semantic.

This is a local **Operate** refinement of **Clear wayfinding**, inherited seed
`6e22d8c4`, not a redesign or a new identity. It explicitly supersedes earlier
navy-rail, blue-charcoal and gradient-chrome guidance, not their historical dated
approvals, captures, test counts or review records. Earlier review scores and
source contrast alone do not override the user's feedback. Fresh review now
returns **SHIP for this neutral palette/chrome refinement only**, without rewriting
those historical records.

## Source-backed implementation

- `crates/app/src/ui/theme.rs::LIGHT` and `DARK` are the exact color authority.
  Existing **22 semantic roles / 44 per-theme frontmatter colors** are retained;
  no new token is needed. Shell surfaces, ink, separators, control boundaries,
  hover and shadow RGB are achromatic. Navigation/content ink and muted aliases
  match within each theme; layers differ by lightness rather than hue family.
  Teal/mint action/selection and warning/error values retain their semantic roles.
- `crates/app/src/ui/depth.rs::chrome()` now returns solid theme `surface`, with
  no linear gradient. Project headers and inspector/disclosure titlebars use that
  shared recipe. The **14 per-theme shadow recipes** retain the same directional
  offsets, blur, spread and alpha; only their palette-resolved RGB base is neutral.
  Focus outlines, transparent evidence-parent treatment and grounded controls
  remain unchanged.
- `crates/app/src/ui/brand.rs::navigation_lockup()` now uses
  `Background::workspace()`, as About already does. Both light rail/About use the
  existing light-background PNG; dark rail/About use the existing dark-background
  PNG. The stacked navigation and horizontal About geometry, embedded bytes,
  image caching and asset provenance are unchanged. Capture readiness checks the
  actual active-theme workspace logo; brand/sidebar assertions use that variant.
- `PRODUCT.md`, `DESIGN.md` and `.impeccable/design.json` persist this current
  contract. The frontmatter remains normative; the sidecar retains all **22 role
  mappings**, updates current color names and exact shadow RGB, records solid
  chrome and theme-aware identity, and mirrors the DESIGN overview.

## Preservation

Keep the 192px rail, native window/layout geometry, existing type ramp, spacing,
radii, 40px action targets, directional depth and focus geometry. No workflow,
control order, drafts, operation/consent boundary or screenshot evidence changes.
Original screenshots stay untinted; website capture-theme settings are separate.

**System / Light / Dark behavior is unchanged:** live desktop following by
default, immediate explicit override, return to following with System, cached
child refresh, machine-local serial preference ownership, protected atomic
`app-preferences.json` writes, truthful fallback/session-only errors, and no
preference I/O on the UI thread. No new dependency, motion, image production or
shipping raster; `assets/brand/provenance.json` remains the asset record.

## Verification handoff

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-neutral-20261017-vzemwkx2/`.

- **Production build passed** in `build-final.log`.
- `tests-confirmation.log`: app library **145 passed / 0 failed / 2 ignored**;
  shell **226 passed / 5 failed / 61 ignored**. Neutral-shell, solid-chrome
  `Background` equality, existing body/control contrast across both palettes,
  both appearance live/override/focus tests and settings keyboard/Tab checks pass.
  The new checks are
  `theme::tests::both_shells_share_neutral_surfaces_and_ink_not_separate_hue_families`
  and `depth::tests::chrome_is_a_solid_semantic_surface_in_both_themes`.
  **The whole suite is not a pass.** Five failures match the prior task's
  categories but are not labelled pre-existing without A/B attribution:
  - `changed_results_tests::changed_results_first_analysis_and_recorded_checks_default_without_resetting_refresh`
  - `loading_feedback_tests::loading_feedback_has_one_owner_and_keeps_its_icon_and_copy_visible`
  - `tests::project_selection_actions_and_long_names_fit_supported_window_sizes`
  - `workflow::tests::review_first_viewport_contains_complete_rows_and_bounded_images`
  - `disclosure_tests::all_inline_disclosures_attach_content_restore_focus_and_do_not_mutate`
- The first `tests.log` failed to compile a new assertion using nonexistent
  `Background.color()`. The current assertion compares actual
  `Background::from(rgb(palette().surface))` with `chrome()` and passes; shipping
  `chrome()` did not change. Intermediate `tests-final.log` also had one extra
  timing-dependent App settings order fixture failure: its initial preferences
  read had not completed. The fixture now waits `owned.appearance.wait_loaded()`
  before creating the window, matching production startup. The confirmation run
  resolves that fixture failure, not the five failures above; no shipping
  preference/backend change was made.
- The first **48 native GPUI/Blade frames** completed in `inspection/` via
  `capture.log`. Shared Rust sources changed during verification and were
  preserved rather than reverted; that first round is superseded, not acceptance
  evidence. The **current-source 48-frame confirmation passed** under
  `confirmation/` via `capture-final.log`. Both rounds use
  eight implemented states × two themes × three sizes, **1440×900, 1280×800 and
  820×600**, with exact copied synthetic website fixtures and provenance, not
  real project results. `confirmation/appearance-manifest.json` identifies the
  final frames: review, settings, project-settings, history, menu, about, warning
  and missing. All six current overview boards are in `boards/`.
- `validate_frames.py` **passed** all 48 PNG hashes, dimensions, opaque/nonblank
  metadata and embedded hashes for the current **13 critical Rust sources** per
  frame, with no exceptions required. Generated `source-receipt.json` binds all
  current Rust files. The fresh reviewer independently matched all **183 source
  files** and **624 embedded critical-source comparisons**. No source-theme,
  header or layout changes followed this final capture.
- `pixel-checks.json` records identical header RGBA at **Y5/Y45** for each
  size/theme: white in light and **#232323** in dark. Original-image **100px
  strips** are pixel-identical between light/dark at all three sizes. This is
  sampled proof, not proof for every header or original-image pixel.
- Documentation validation is recorded separately in `documentation-review.md`
  under the evidence root. It checks YAML/JSON uniqueness, source values and token
  references, current chrome/brand rules, unchanged non-color metrics and
  preservation of historical records. Revalidation retains **44 colors / 22 role
  mappings / 101 token references / 14 shadow recipes**, with unique-key YAML/JSON
  parsing. It is not a visual verdict or test-suite pass.
- **Fresh finish review: scoped SHIP.** `finish-review.md` inspected all **48
  final captures** and **six current boards**, confirmed actual background-
  appropriate rail/About logos in all variants and found **no material palette
  findings**. The disposition covers only this neutral palette/chrome refinement.
  It does not clear the five unresolved, unattributed shell failures, the existing
  constrained minimum-size queue or any whole-app qualification. No whole-suite
  or release approval is claimed; older approvals/verification remain historical.

These are development-host native offscreen checks using synthetic fixtures.
Actual OS input, accessibility/screen-reader integration, live OS theme switching,
Windows/macOS, usability and release remain unqualified. No HTML/CSS detector
applies to the native artifact or this documentation-only pass.

## Documentation notes

Unrelated existing sidecar drift remains outside this refinement: inspection
`priority` still mentions full hover text despite later tooltip restraint, and
`nativeGeometry.approvalDialog` differs from DESIGN's current component geometry.
Neither was repaired; concurrent typography/search documentation is preserved.
