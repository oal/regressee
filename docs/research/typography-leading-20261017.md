# Typography and leading — 2026-10-17

## Scope and assessment

User reported that the wrapped **25 baseline screenshots** label looked like
disconnected text, and requested an app-wide typography/legibility pass using
Impeccable. This is an Operate-mode refinement, not a redesign.

An independent type assessment and a separate mechanical scan agreed on scope:
keep the installed OS UI family, sizes, hierarchy and existing tuned controls.
The native detector returned no findings, but source inspection found omitted
leading throughout the shell. GPUI's default is 1.618×: a 13px label therefore
used roughly 21px leading rather than the existing help role's 18px. Main
headings, History, form hints and empty states had similar inherited defaults.
Compact inspection metadata and the editor already had intentional metrics.

## Treatment

| Role | Logical-pixel size / line height |
| --- | --- |
| Task heading | 28 / 36 |
| Review heading | 24 / 32; compact 21 / 28 |
| Title | 21 / 28 |
| Section | Existing 18 / 24, unchanged |
| Body and field label | 15 / 22 |
| Controls and compact labels | Existing 14 / 20, unchanged |
| Help and wrapped rail status | 13 / 18 |

Shared role metrics live in `crates/app/src/ui/theme.rs`. The shell owns body
leading, while local size overrides explicitly pair their own line heights.
Field labels and hints no longer inherit an enclosing control/heading's metrics;
hints keep regular weight. Rail lines retain their natural wrapping and 8px
project-name/status separation. Other missing pairs were completed across setup,
recovery, review, filtering, inspection details, History, badges and progress.
Existing intentional 13/20 inspection treatments and the editor's 14/20
shaping/caret/selection/scroll model remain unchanged.

No family replacement, font-size reduction, new assets/dependencies, palette
change, copy change, screenshot modification or backend operation was introduced.
Concurrent user-directed neutral-theme and search-control work is preserved;
that work's verification is not silently included in this typography pass.

## Verification

Private evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-typography-20261017-nkuj8qxq/`.

- New simulated regressions cover wrapped 1/25/1,000 baseline counts at all
  three window sizes and help/field leading under an oversized ancestor.
  Navigation keeps its existing minimum target; simulated-font labels may wrap.
- The first library run passed **145 tests**, with **2 opt-in tests ignored**.
- The first broader shell run passed **224**, with **7 failures / 61 ignored**.
  One failure was the new test's incorrect assumption that every navigation
  label is single-line; its assertion now checks the existing 40px minimum.
  The corrected three-size test passes. No existing assertion was relaxed.
  Independent triage found five matching failures in October 16 logs (changed-
  results/History epoch fixture, disclosure mount fixture, loading-message
  containment, header/scroll-owner containment and constrained queue height).
  The remaining settings-focus failure passed an isolated rerun; its fixture
  does not await the independent appearance-preference worker. The two logged
  vertical containment overflows improved slightly rather than worsening.
  These issues were not repaired as an unrelated side effect. Historical logs
  are not a controlled typography-only A/B; the shell suite is not all green.
- A source-pair scan found **154 explicit size overrides in 24 production
  files**, all with one local leading value. This is source evidence, not a
  rendered/accessibility result.
- Both native mechanical type scans returned no findings. The source-pair scan
  catches missing native leading that this detector does not report.
- `git diff --check` and scoped formatting checks passed. The production UI
  build (`cargo build --locked -p regressee-app --features ui --bin regressee`)
  passed with the existing dependency future-compatibility warning.
- The first bounded capture batch produced **41/42** frames before the command's
  120-second timeout. Six contact sheets and selected full-resolution frames
  were inspected. No additional UI refinement was needed. A fresh confirmation
  batch completed successfully in **106.24 seconds** with **42/42** native
  frames: review, project settings, History, App settings, note editor, About and
  Projects-unavailable, each in Light/Dark at 1440×900, 1280×800 and 820×600.
  All 41 overlapping PNGs are byte-identical to the first batch; the remaining
  dark 820×600 helper frame was read at full resolution. All 42 PNG hashes and
  dimensions were verified. Eight key captured UI source hashes match current
  source, including the concurrently updated `main.rs`. Every recorded rail
  status is 36px high (two 18px lines); task-heading bounds are 36px high.
  Stored synthetic evidence equality passed after the full matrix. Details:
  `confirmation/typography-manifest.json` and `confirmation-verification.json`
  under the private evidence root.

## Independent finish verdict

**SHIP for this typography refinement only.** The reviewer inspected all six
first-batch sheets and seven full-resolution frames. The wrapped rail count is
cohesive across themes/sizes; settings hierarchy, History text, About paragraphs
and the existing note editor remain legible without new overlapping lines or
clipped glyphs. No priority defect attributable to the typography patch was
found and no further type edit was recommended. The missing first-batch frame is
dark 820×600 Projects-unavailable. The fixture named `empty` actually represents
Projects-unavailable, not an authoritative empty state. The six broader failures
and all qualification limits remain open; this is not whole-app acceptance.

## Evidence and limits

Captures use isolated synthetic Northline website fixtures copied from the
existing neutral-theme inspection's `website-fixtures/`. They are not customer
captures or shipping images. The 25-count rail projection exists only in the
capture fixture's in-memory presentation; stored evidence is compared before
and after the capture matrix and must remain identical. Each PNG has renderer
metadata/source hashes; `typography-manifest.json` records inspected bounds.

Development-host GPUI/Blade rendering and simulated input are not OS-input,
screen-reader, Windows/macOS, usability or release qualification. Linux currently
substitutes Noto Sans for the requested DejaVu Sans through host font resolution;
the established family request is unchanged. Exact font/weight resolution on
each supported OS, native text scaling and accessibility remain separately
unqualified. No claim is made that screenshot Zoom scales the application UI.
