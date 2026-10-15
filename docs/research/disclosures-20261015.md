# Inline disclosure clarity — 2026-10-15

## User intent and direction

The user reported that Troubleshooting tools and similar expandable sections
looked like select dropdowns. Their content opened beyond a horizontal rule,
appearing unrelated. They requested an app-wide Impeccable review and repair,
followed by committing all current work, pushing and tagging a new preview build.
The user selected **Grouped disclosures**: leading disclosure arrow, stable
heading, explicit Show/Hide and indented content immediately beneath its owner.

Mode: **Operate**. Preserve Regressee’s Clear wayfinding palette, native system
type, explicit saves, privacy/approval boundaries, independently open sections,
drafts and scroll owners. No identity replacement, animation or new capability.

## Review

Two isolated layout assessments reviewed the native source. Assessment A found
select-shaped full-width triggers, inconsistent grouping and child headings that
overpowered their owner. Troubleshooting inserted a new ruled Support report
section after its trigger. Assessment B found no automated layout detections
(`[]`), but the scanner does not validate Rust/GPUI painted geometry. Neither
the clean detector nor source review establishes native usability/accessibility.

Spatial thesis: **the heading owns everything immediately beneath it; rules
separate independent groups, not a heading from the content it just revealed.**

## Implementation

- Shared `disclosure_toggle` / `disclosure_group` primitives and shell plumbing
  preserve enabled/mounted guards, measured controls, scroll anchors and focus.
- Leading native right/down chevrons, stable wrapping titles and explicit
  Show/Hide replace bordered secondary buttons and selected-choice expansion.
- A 32px title gutter attaches help/body, with no intermediate rule, new card or
  nested scroll owner. Compact headings use the incumbent control typography;
  App-settings section headings use the incumbent section typography.
- Expansion reveals the owner and the beginning of its content together;
  collapsing returns focus to the owner, including pointer collapse of a field.
- All 17 inline trigger sites are covered: Browser details, Storage locations,
  Network settings, Network limitations, Troubleshooting, scope/speed/custom
  sizes, capture limitations, environments/configuration, ignored areas,
  advanced parameters, metrics/revisions, technical capture diagnostics,
  page-search details, History details, exact preview details, baseline source
  details and task details.
- Task Details moves from the pinned action footer to its reading body. Its
  focus order now follows heading, revealed links, then pinned task actions.
- Requested saved-data results retain their visible ownership and Cancel preview
  even when optional tools are collapsed. Existing report/export/delete/privacy
  decisions stay explicit. Real selectors, flyouts and dialogs are not converted.

## Verification

The synthetic matrix covers both 1280×800 and 820×600. It checks every inline
trigger, attached body geometry, visible headers, collapse/focus, hidden controls,
unchanged work-request counts and byte-equivalent persisted evidence. Enter and
Space include a redraw before key release, covering the native duplicate-click
path repaired by the concurrent keyboard work.

- Two bounded GPUI/Blade capture rounds passed. Batched inspection found a
  collapsed History heading with no finite width and insufficient expansion
  scroll reveal; both were repaired before the confirmation round. All first
  round scenes were inspected in contact sheets; final Troubleshooting and
  History were confirmed at both sizes. No further cosmetic capture loop.
- The confirmation matrix produced 36 PNG/JSON pairs: all 17 expanded scenes at
  both sizes, plus collapsed Troubleshooting at both sizes. Evidence remains
  private, synthetic and separate from shipping assets:
  `/var/home/olav/Data/opencode/tmp/opencode/regressee-disclosures-confirm-20261015-vO5bXP/`.
- The shared-library UI suite passed: **123 passed, 2 opt-in tests ignored**,
  including long disclosure titles, dialog focus and scrolling, and link hints.
- The non-UI workspace suite passed with `regressee-app/test-support`, including
  persisted synthetic 100/1,000-page qualification. Packaging and environment
  Python checks passed: **22 tests**.
- Impeccable's layout detector returned `[]` before and after implementation;
  it does not validate native painted geometry or accessibility.

### Integration verification

- Strict workspace Clippy passed with all features and targets and `-D warnings`.
  The scoped shared-fixture allowance and equivalent form-width clamp removed
  the two integration warnings without changing behavior. Cargo still reports
  upstream `proc-macro-error2` future-Rust compatibility debt.
- Core **54 passed**; storage **35 passed, 2 profiling tests ignored**; the new
  partial-baseline integration **2 passed**. Packaging/environment checks were
  reconfirmed at **22 passed**.
- The production-feature native Linux build passed. The final styled-confirmation
  subset passed **4 tests**, including the added pointer and wheel regressions.
- Workspace formatting and `git diff --check` passed.
- All **183** vendored GPUI file hashes match the provenance manifest. Its
  patched-file list now includes the previously committed explicit-float lint
  correction in `src/taffy.rs`; no upstream source was reverted.

Native input, accessibility, clean installation, other operating systems and
release qualification remain pending; Linux GPUI offscreen evidence is not an
OS-window, keyboard hardware or screen-reader pass.

## Verdict

The scoped disclosure repair is ready for an experimental preview: optional
content has a clear owner, real selectors retain their meaning, and expansion
does not save, visit websites, or modify evidence. Concurrent recovery, partial
baseline, dialog, link and keyboard/focus work is preserved in the same requested
release; its dedicated research notes describe its separate evidence.

Impeccable reported a stale `.impeccable/design.json` sidecar. It was not refreshed
as a side effect; an explicit `document` pass can regenerate it later.
