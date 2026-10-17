# Compact workspace toolbars — 2026-10-16

## User direction and scope

The user rejected the generic text-heavy viewer toolbar, chose **Compact +
labelled**, and asked for the same examination of other toolbar-like rows.
This is an Operate-mode extension of Clear wayfinding, not a new visual identity.

- View keeps an icon, current mode name and chevron. Menu rows have matching
  icons, a full-width mint selection and a drawn check instead of “Current”.
- Zoom removes the repeated “Zoom:” prefix and keeps fit/manual scale explicit.
  Compact fit shows “Fit · 26%” on one line. Menu choices and keyboard behavior
  are unchanged; compact Image controls remains in the Zoom menu.
- Results, Image controls, Details, Analysis, sidebar, refresh and navigation
  arrows use quiet 40px native tools. Mint plus a teal boundary denotes selected
  tools; keyboard focus retains the existing ink outline. Every icon-only action
  names itself on hover and keyboard focus, without another focus stop.
- Results filter and page options/details retain short labels. Queue navigation,
  History section/type selectors and History/project pagination share the same
  stroke family; sections remain named, and pagination shows its page number.
- Capture, save, review classifications, baseline consent, export and deletion
  retain explicit wording and their existing guards. Screenshots are untinted;
  task logic, source transforms, drafts and persistence rules are unchanged.

`ui/toolbar.rs` owns the native path family and tool presentation. The shared
action frame in `ui/controls.rs` retains focus, hover and disabled behavior without
an empty label child. No package, font, palette or shipping raster was added.

## Verification

- Production UI build passed: `cargo build --locked -p regressee-app --features
  ui --bin regressee`.
- Shared UI: **64 passed**, including icon geometry, invariant 40px targets,
  keyboard name hints, disabled tools, compact zoom and selector interaction.
  The complete final-source app library suite also passed: **127 passed,
  2 opt-in tests ignored**.
- Shell regression suite: **203 passed, 12 opt-in tests ignored**. This run
  preceded only the final hint-sizing and diagnostic-harness corrections.
- Final-source focused shell checks: **20 passed**, covering all six view modes
  at 1440×900, 1280×800 and 820×600 with both navigation preferences, drawing,
  menus, image help, split slider, queue/draft preservation, History hierarchy,
  sidebar and project actions. Unchanged stored evidence is asserted.
- Renderer calibration passed with exact RGB channels and readable native fonts.
  Final capture test passed: **30 Linux GPUI/Blade frames**, ten scenes at each
  required desktop size. Each has source hashes and an image hash sidecar.
  All final PNGs were checked opaque/nonblank with matching hashes and inspected
  as full-frame sheets, with menu, drawing and focus states also read full-size.
- GPUI's diagnostic bounds map retains removed selectors. Disabled-hint absence
  is tested in a fresh window, not inferred from a stale bounds entry.

The first inspection's menu was captured closed and its drawing state was forced
without the real tool lifecycle; those two captures do not qualify those states.
The final harness uses actual activation and asserts the open menu and enabled
drawing exit. The confirmation set supersedes the entire first set.

Private evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-toolbars-20261016-o4mpprj8/`.
Final images and manifest are in `confirmation/`; fixture provenance is recorded
in `inspection/fixture-provenance.json`. Synthetic website fixtures are reused
offline, not represented as real customer captures or shipping assets.

This is development-host rendering and simulated input, not OS-input,
screen-reader, Windows/macOS, usability or release qualification. The existing
GPUI/native-accessibility integration gap is not resolved by visible tool hints.
Unrelated workspace changes were preserved. Diagnostic-only linker aliases were
created under the private evidence root; project toolchain settings are unchanged.

## Independent finish record

The independent reviewer identified no material visual defect in the 30 final
frames. Its sole **fix** finding was outdated toolbar/Zoom documentation.
The documenter merged the source-backed toolbar component into `DESIGN.md` and
the existing local native sidecar without changing palette primitives or unrelated
drift. The reviewer then scored that documentation fix **resolved**, with
disposition **ship** for the listed fix only. Reports are `finish-review.md` and
`documentation-report.md` under the private evidence root. The user's subsequent
request for more button icons and a grouped inspection header starts a separate
iteration; this earlier verdict does not qualify that work.

## Vertical queue navigation follow-up — 2026-10-16

User pointed out that left/right arrows contradicted the vertical results list.
Previous/Next result and page-item tools now use matching up/down native chevrons.
History/project pagination and setup Back remain horizontal. Labels, 40px targets,
selection, focus order, disabled boundaries and navigation behavior are unchanged.

Scoped verification: production UI build, four toolbar/layout tests, three shared
toolbar tests and the screenshot-switch regression passed. The existing renderer
recipe passed with 30 synthetic-evidence frames at 1440×900, 1280×800 and 820×600.
Inspected full-size comparison frames at 1280×800 and 820×600, plus the 1440×900
page list and History pagination: vertical item chevrons and horizontal pagination
are distinct, with no scoped layout defect found. Formatting and diff checks pass.
This is in-thread Linux offscreen verification, not OS-input/accessibility approval.
No new shipping asset, palette or operation was introduced.

Qualification: formatting passed before concurrent scrollbar and keyboard-test
edits landed. The final workspace formatting check reports those separate changes,
which were left untouched. Captures and tests qualify their compiled source
snapshot, not subsequent edits to shared `main.rs` or `toolbar_tests.rs`.

Private evidence: `/var/home/olav/Data/opencode/tmp/opencode/regressee-vertical-queue-9q1wby7i/`.
Synthetic website fixtures and development-only linker aliases were reused from
the earlier single-image inspection; no host/toolchain settings were changed.
