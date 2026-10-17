# Context and action wording — 2026-10-16

## Scope and method

User feedback: “Before Website / After Website” presents a default environment
name as if it explained the comparison. The follow-up requested an Impeccable
review of similar cases throughout the application. This is an Operate-mode
clarification pass, not a redesign or a change to product capabilities.

Two independent read-only source reviews covered setup, access, discovery,
comparison/review, Projects, App settings, History, sharing/storage and recovery.
Findings were checked against actual handlers before editing. Meaningful
user-defined names, original screenshot pixels, historical source attribution,
technical diagnostics, privacy warnings, saved data and approval guards stay intact.

## Corrections

- Comparison context leads with **Saved baseline / This check**, or **Recorded
  check** during read-only inspection. It no longer repeats the generated
  “Website” name or the same environment on both sides. A meaningful shared name
  appears once; different recorded names still distinguish Production/Staging
  and other sources. Details retains full
  names and origins; missing historical sources are not filled from current settings.
- Capture-only context distinguishes **Saved baseline**, **Not yet a baseline**
  and **Screenshot preview**. Unsaved captures say **Inspect capture**, not
  **Inspect baseline**; stored preview intent is not renamed. Historical captures
  use neutral **Recorded capture** when their baseline status is not established.
- Setup says **Save project & continue** and confirms saving project settings,
  not saving a website. **Find pages** no longer conflates page discovery with the
  separate screenshot-preview action. Default “Access for Website” is omitted.
- **Page address mapping** does not assume Production/Staging. **Add / replace
  address mapping** describes a draft edit, without suggesting approval or saving.
- **Confirm login** replaces the ambiguous **I'm ready** across both actions,
  instructions and recovery. Progress names checking and saving; encrypted versus
  app-session-only retention behavior is unchanged.
- **Support report** is consistent across preview, save, progress, cancellation,
  failure and success. The privacy preview matches the actual Save support report
  action. The filename/schema/content and separate full-URL capture diagnostics
  are unchanged.
- Network settings claim to be loading only while their read is pending; the
  unavailable state names Retry loading saved settings only when that action is
  mounted. Other pending work gets a waiting instruction. Permission recovery points
  to the OS; App settings only shows storage locations.
- Skipped/cancelled screenshots say **Screenshot not captured**, while actual
  failures retain their failure heading and recorded cause.
- Approval-impact, page-count and search-count templates handle one item correctly.
  Review progress uses **marked Needs fix**, and retained filtered screenshots use
  screenshot language rather than internal “evidence”. The concurrent changed-only
  navigation work had already replaced ambiguous “left” with “changes to review”;
  this pass preserves that separate behavior.
- Remove the generic **Local task workspace** label. The concurrent removal of
  **Local project workspace** is preserved, not claimed as this pass's change.

## Prevention

`docs/ux-writing.md` records the role-versus-environment distinction, accurate
save/action terminology, meaningful context, loading truth and actionable recovery.
`ux_copy_tests.rs` rejects the identified phrases across production presentation
sources and exercises real source-role, historical/missing-source, unsaved/saved/
preview, network-read and stopped/skipped states. Store-backed tests assert that
the persisted evidence is unchanged.

## Verification

- The eight focused `ux_copy_tests` passed, including role/default-name,
  historical-source preservation, unsaved/saved/preview, network-read truth,
  skipped/stopped headings, note/save and deletion-copy coverage. This run
  preceded the concurrent theme refactor; it is not a final-source whole-app pass.
- Final UI shell/library tests and the two-size offscreen capture attempt are
  **blocked at compilation** by unrelated in-progress theme changes: the palette
  constants were replaced before their consumers were migrated (`ui/brand.rs`
  still imports `theme::INK`/`RAIL_INK`, with equivalent errors in other controls).
  This task did not restore those constants, revert the refactor or claim fresh
  screenshots. The new capture harness remains opt-in for the next buildable state.
- `git diff --check` and scoped formatting checks passed. Existing unrelated
  scrollbar-formatting drift was not reformatted as part of this copy pass.
- The non-UI app library suite passed: **51 passed, 0 failed, 1 opt-in test
  ignored**, including authentication, support-report privacy and task-policy tests.

The independent finish review found four semantic gaps in the first patch: an
already-saved baseline capture opened through History could be called unsaved;
matching meaningful environment names were both omitted; a retry instruction
could name an unmounted action; and two worker failures retained diagnostic-file
terminology. They were corrected in one batch. The same reviewer verified all
four as **fixed in source**, with no direct semantic regression found. Regression
coverage now includes the real `Store::inspect_run` capture response and mounted
retry guards. **Those final corrections have source verification, not a new UI
test or render pass**, because the theme integration remains blocked. No whole-app
or visual PASS is claimed.

Development-host commands use the installed Rust 1.98.1 toolchain. The checked-in
development convenience links in `.runtime/link-libs` pointed to absent Debian
paths on this Fedora host; isolated linker-name symlinks under the private
evidence root point to the already-installed `/usr/lib64` libraries. No system
dependencies or repository linker settings were changed.

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-ui-copy-20261016-8x6s75dq/`.
The opt-in `renderer_capture_ui_copy` test is prepared to use synthetic saved
corpus screenshots and Linux GPUI/Blade rendering at 1280×800 and 820×600. **No
frames were produced** because compilation stopped first. No customer website was
visited and no shipping raster was added. The completed checks are not OS-input,
accessibility, cross-platform, usability or release qualification.
