# Stable sidebar identity — 2026-10-14

## User-reported problem

Review changes inserted a disabled **Hide navigation** action before the logo
when no project was selected. Its render condition checked only the screen,
while its activation rules required a saved project. This both exposed an
irrelevant control and displaced the identity compared with other sections.

## Scoped correction

- Keep the logo/name as the first rail item in every section.
- Put **Hide sidebar / Show sidebar** in the shared review header, in the same
  location for both states. Its hover hint explains that hiding project
  navigation gives screenshots more room.
- Share one availability rule across header rendering, control mounting and
  review geometry. Project choosers have no sidebar toggle and keep navigation
  visible, including when reached from a collapsed review workspace.
- Returning from the chooser retains the explicit review sidebar preference.
  Resize, keyboard focus, local drafts and evidence protections are preserved.
- Existing branding artwork, palette and backend operations are unchanged by
  this correction; concurrent branding/settings work is not replaced.

## Bounded verification

- Production native build passed (`ui`, `regressee` binary).
- Three new sidebar controller/layout tests and the explicit renderer test
  passed; the two existing `repair_v4_2` navigation/read-state/guard tests passed.
- App library UI tests: 110 passed, 2 ignored. Core library tests: 52 passed.
- Eight GPUI offscreen captures cover no-project Review changes, open review
  sidebar, collapsed review sidebar and Change project, at 1280×800 and 820×600.
  The confirmation batch is byte-identical to the inspection batch. The renderer
  asserts stable logo bounds, header-owned toggle bounds and unchanged evidence.
- The simulated renderer retains old selector bounds after unmount; controller
  tests use the shared mount contract for absence. Fresh-window captures verify
  actual visual absence separately.

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-sidebar-20261014-eLSQGnRd/`
(inspection at the root, confirmation under `confirmation/`). JSON sidecars bind
the production sources and capture harness. These are synthetic, offline Linux
GPUI pixels, not real-site, OS-input, accessibility or cross-platform acceptance.
No whole-shell-suite acceptance is claimed.
