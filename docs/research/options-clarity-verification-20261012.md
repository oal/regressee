# Options clarity follow-up — 2026-10-12

**Outcome: the final independent reviewer disposition is “ship — the four
reviewed fixes within discovery options, result filters and environment mapping
previews”; all four findings were scored resolved.** This is a scoped development
handoff, not blanket application usability or release approval.

## Confirmed scope

The user confirmed discovery options, result filters and environment mapping
previews in the **Options clarity follow-up — 2026-10-12** direction contract in
`.impeccable/surfaces/crates-app-src-main-rs.md`. This is an ordinary local
extension of **Clear wayfinding**, preserving the incumbent visual identity and
every backend operation, not a new visual world or design-system replacement.
The surface is **Operate**, native Rust/GPUI desktop. Phone/tablet profiles concern
captured websites; this is not mobile, iOS or Android product UI.

`PRODUCT.md`, existing `DESIGN.md` and `.impeccable/design.json` remain the product
and system authorities. This documentation pass creates only this report; it
does not edit those authorities, application source or evidence, perform drift
repair, build/render, spawn agents, or commit/push.

## What changed

- **Discovery:** a titled, bounded **Page options** disclosure sits beside the
  searchable page list. Selected-page inclusion and search-match exclusion/restore
  controls precede capture estimates. Estimates remain explicitly approximate;
  search alone does not change inclusion. **Discovery details** optionally reveals
  coverage and notes. The body scrolls independently while **Close** and
  **Dismiss page list** remain attached; dismissal does not change the saved
  baseline. Long notes no longer displace the usable list or exit actions.
- **Result filters:** a bounded disclosure overlays the viewer without shrinking
  or replacing the queue. Hidden viewer controls leave the mounted focus model.
  Filtering preserves the inspected screenshot identity, ignored-area drafts and
  review notes; zero matches explain that the screenshot and edits are kept and
  offer recovery. Closing/Escape restores the filter trigger.
- **Mapping previews:** **Mapped URL preview** has a protected title/Close header
  and independently scrollable body, with separately labelled **Source page** and
  **Capture address**. It reports the total and first-16 display scope, explicitly
  uses saved environment settings, and cautions that a representative probe visits
  at most three pages, not the whole website. Empty mappings explain the next
  step. Long URLs stay within the row width. Close, Tab/Shift-Tab navigation and
  Escape reveal the relevant control through the outer form scroller; dismissal
  restores **Preview mapped URLs** without a save or probe.
- **Draft safety:** saved-settings preview/status refresh no longer overwrites an
  already loaded environment draft. Initial hydration is bound to the exact
  request ID and environment draft signature; edits arriving after the read
  started are retained. Preview reads remain separate from explicit save, login
  and probe operations. Existing fixed inventories, evidence/history retention,
  approval and baseline safety constraints are preserved.

## Incumbent-system comparison and existing drift

Source and selected final native pixels agree with the existing system: white
`SURFACE`, blue-gray `INK`/`MUTED`, `LINE` grouping and `CONTROL_LINE` boundaries,
with shared sky-selected/blue-action states and the unchanged deep-blue rail.
`crates/app/src/ui/theme.rs` palette constants match the incumbent `DESIGN.md`
frontmatter. Panel headings use the existing section metrics (18/24 logical px);
controls use 14/20 and supporting copy 13/18. Shared actions retain the 40px
minimum height, 4px corners and paint-only focus/hover states. Installed OS UI
fonts, the spacing rhythm, ruled groups and flat layering remain intact.

The shared `disclosure_panel` is a local component extension, not a new decorative
card language. Options width caps (320/360px), mapping height (280px) and independent
scrolling are task-local geometry. No new palette, font, radius or motion scale
was introduced by this scoped extension; the incumbent system need not be
restated or replaced.

Pre-existing context/artifact mismatches remain untouched:

- `PRODUCT.md` says `adaptive`, although its prose and the surface contract
  explicitly describe desktop. Context loading consequently emitted iOS/Android
  guidance; that routing mismatch does not govern this desktop task.
- `DESIGN.md` and the 2026-10-11 sidecar retain an earlier broad **fix** disposition
  and reference-environment qualification. The sidecar predates subsequent source
  documentation. This narrower options **ship** verdict neither refreshes those
  records nor certifies their older outstanding scope. `DESIGN.md` also already
  had local modifications when the documenter began; they were preserved.

## Relevant source paths

| Source | Scoped responsibility |
| --- | --- |
| `crates/app/src/options_ux.rs` | Discovery/filter disclosures, mapping rows/scope and environment draft signature. |
| `crates/app/src/main.rs` | State, mounted actions, no-match recovery, focus/dismissal and request/reply hydration guards. |
| `crates/app/src/ui/controls.rs` | `disclosure_panel`: protected header, scroll body, visible overflow indicator; incumbent action primitives. |
| `crates/app/src/review_ux.rs` | Reveal focused options controls inside their bounded scroller. |
| `crates/app/src/form_recovery.rs` | Outer-scroll reveal for mapping Close and preview/probe controls. |
| `crates/app/src/workflow.rs` | Workspace placement/geometry and mounted-control filtering. |
| `crates/core/src/shell.rs` | Disclosure controls in shell keyboard traversal. |
| `crates/app/src/options_ux_tests.rs` | Four options regressions with isolated synthetic storage and real Jobs replies. |
| `crates/app/src/ui_renderer_capture.rs` | Actual offscreen GPUI capture matrix and unchanged synthetic project/evidence assertions. |

## Recorded verification

The completed implementation packet and reviewer handoff record:

- Final workspace/all-features suite: **334 passed, 0 failed, 42 deliberate
  ignores** (`workspace-tests.log`, `test-summary.json`). Ignored renderer and
  qualification tests are not ordinary suite passes.
- Normal UI build and workspace/all-targets/all-features Clippy with
  `-D warnings` passed (`build.log`, `clippy.log`). Formatting and whitespace checks
  also passed in the implementation handoff. Existing GPUI dependency warnings
  and Rust future-compatibility notices remain unchanged.
- Four `options_ux_tests.rs` regressions passed: long discovery content retains
  list/Close/Dismiss access; filtering retains evidence and drafts and reveals
  lower choices; mapping has readable bounded title/body, first-16 scope, long-URL
  containment, empty recovery and actual forward/reverse focus movement; dirty
  preview runs a real Jobs request/reply and preserves drafts, including newer
  edits during an initial read. Mapping checks assert title width at least 240px,
  title height at most 24px and body height at least 150px at both window sizes.

Final visual evidence comprises **20 actual Linux GPUI offscreen PNGs**: ten states
at **1280×800** and **820×600**. The matrix covers discovery, Page options,
Discovery details, options with no matches, Result filters, filters with no
matches, mapping preview, empty mappings, lower mapping rows and lower filter
controls. All final images were independently inspected. The documenter read
current relevant source and existing logs and inspected selected final pixels;
it did not repeat the full independent review or rerun verification.

## Evidence retention

- Private `regressee-options-20261012-uvzi4wyw` contains the valid initial
  16-state render and its known mapping defect. It remains initial evidence, not
  a substitute for the final corrected confirmation.
- Private `regressee-options-confirm-20261012-uo7b6fc6` contains `renderer.log`,
  `workspace-tests.log`, `clippy.log`, `build.log`, `integrity.json`,
  `test-summary.json` and `source-binding.json`. Integrity records the 20 image
  hashes, decoded sizes and opaque alpha. Source binding records the relevant
  source and executable hashes at its creation time.
- Final images and layout metadata were copied to
  `.impeccable/review/options-clarity-20261012` for review. These are private
  synthetic review artifacts, not shipping assets or user-site results; do not
  commit private pixels or data. This handoff records directory basenames only.

## Concurrency and qualification limits

Unrelated task/status UI was already being modified and continued changing
during this work. A temporary compiler error in `crates/app/src/task_ux.rs`
disappeared through that ongoing work before an authorized minimal fix could be
applied; the options task made no changes to that file. Whole-file `main.rs`
changed after source-binding creation through concurrent work. The independent
reviewer inspected the relevant correction paths directly, and its **ship**
verdict explicitly does not certify those unrelated changes. **The final
whole-worktree source/binary binding is not claimed to be frozen.**

Rendering and regressions used isolated synthetic projects only; no websites
were visited and no user projects were written. The capture harness asserts
unchanged synthetic authoritative project/evidence records. Offscreen pixels and
simulated GPUI focus movement support scoped Linux layout/behavior checks, not
real OS input, accessibility, IME, high-DPI, Windows/macOS, blanket app usability
or release certification. Previously retained broader qualifications and existing
dependency/future warnings are not cleared by this report. No new usability
score or measured user improvement is claimed.
