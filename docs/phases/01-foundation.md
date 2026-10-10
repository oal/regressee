# Phase 1 — UX, visual system, and application foundation

## Outcome

A maintained, licensed GPUI application shell with coherent product language,
domain contracts, storage, and an agreed sleek/modern visual system.

Prerequisite: Phase 0 gate passed. Read `PRODUCT.md` and `PLAN.md`, Sections
3–6, 10–11, and 14. Use Phase 0's tested versions and support matrix.

Local continuation exception: decision 0005 supersedes the earlier blanket stop
while external/native Phase 0 qualification is open. Phase 1 local implementation
is delivered; independent verification, human visual confirmation and broader
acceptance gates remain pending. See `01-status.md` for evidence and limitations.

## Work

- [x] Scaffold the Rust workspace and worker package with reproducible pins,
  lockfiles, format/lint/test tasks, and cross-platform build checks.
- [x] Add the official MPL-2.0 license, appropriate SPDX source headers, and
  third-party notice tracking. Do not relicense third-party code.
- [ ] Confirm the primary screen topology and user-facing vocabulary in the
  plan; sketch complete baseline/check/review/share flows without code first.
  Proposal/sketches documented; human confirmation is pending.
- [ ] Establish typography, spacing, semantic colors, surfaces, focus states,
  and components in a small documented visual system. Seek confirmation of a
  representative setup and comparison surface before broad UI implementation.
  Tokens/components delivered as a proposal; representative visual review pending.
- [x] Build reusable GPUI controls for forms, buttons, dialogs, status labels,
  lists, progress, errors, and empty states; respect host shortcuts/dialogs.
- [x] Implement the project shell and navigation with representative data.
  Mark demonstration data as such; it is not completed functionality.
  Actual saved project data / honest empty states, not fake capture results.
- [x] Define typed project, environment, inventory, capture profile/run/target,
  baseline, analysis, review, mask, artifact, and session-reference contracts.
- [x] Create SQLite migrations, immutable artifact storage, atomic writes,
  hashes/reference tracking, and OS-local data/cache directory handling.
- [x] Define versioned worker commands/events, error types, payload limits,
  and a process-supervisor interface.
- [x] Connect bounded background tasks and UI event updates; make blocking
  work impossible through the main UI action paths.
- [x] Define the session-secret abstraction and archive schema boundaries
  without prematurely implementing every feature.

## Deliverables

- Buildable app shell with project/setup/results/history/settings navigation.
- Documented visual system and confirmed representative surfaces.
- Maintained domain/protocol/schema contracts and migration tests.
- License and notice foundation; CI/build instructions for all launch OSes.

## Completion gate

The app launches on the support matrix, stores/reopens a project, migrates a
fixture database, and displays all material shell states consistently. Keyboard
navigation and focus work. Blocking file/image/process operations run in
background tasks. Mock content cannot be mistaken for real crawl results.

## Next

Phase 2 replaces one mocked flow with a real single-page capture/comparison
vertical slice, retaining the same domain boundaries.
