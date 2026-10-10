# Phase 5 — analysis controls, ignored regions, and baseline review

## Outcome

Users can distinguish meaningful changes from noise, inspect evidence, adjust
judgment after capture, and deliberately accept selected changes into history.

Prerequisite: Phase 4 gate passed. Read `PLAN.md`, Sections 6, 9–10, and 13.

## Work

Implementation checklist only; independent product verification and native/human/
external qualification remain pending. See `05-status.md` for exact evidence and
`../research/phase5-presets.md` for fixture-only calibration and limitations.

- [x] Complete perceptual changed-pixel metrics, local/tile concentration,
  connected regions, and explicit dimension/page-height evidence.
- [x] Calibrate Sensitive/Balanced/Relaxed policies with an annotated synthetic
  corpus; document exact numeric values and version them. Real-site/user accuracy
  calibration remains unqualified.
- [x] Implement advanced analysis parameters, background reanalysis,
  cancellation/debouncing, and cache keys that include policy/mask versions.
- [x] Keep original screenshots and past analysis revisions intact; associate
  displayed metrics/review decisions with the correct revision.
- [x] Add before/after overlay slider, heatmap, change-region navigation,
  synchronized zoom/pan, and plain-language metric explanations.
- [x] Add draw/edit/delete ignored rectangles per page/profile, visible on both
  images; report ignored coverage and warn/revalidate on geometry changes.
- [x] Implement the locally tested, measured dimension-preserving image-replacement
  scope with a representative-page preview and truthful supported-media explanations.
  Unsupported/source-dependent behavior is warned, not universally qualified.
- [x] Enforce matching normal/replacement profile modes and transformation
  versions; require new baseline capture for material changes.
- [x] Implement unreviewed/intentional/needs-fix states and optional notes,
  separate from metric-based changed/unchanged classification.
- [x] Implement **Accept selected changes** with impact preview, immutable
  baseline inheritance, provenance, and transactional activation.
- [x] Implement whole-baseline replacement and historical set-active/restore.
- [x] Block approval of failed or incompatible targets and cross-rendering-
  environment partial mixing; explain the full-replacement alternative. An
  intentional production/staging address mapping remains eligible for approval
  when capture/rendering settings are compatible.
- [x] Test tiny changes on long pages, synthetic anti-alias/noise and raster changes,
  height shifts, large/all-area masks, and composite-baseline references. Real
  photographic/font/platform diversity remains pending.

## Deliverables

- Complete comparison/review UI and versioned analysis policies.
- Tested image-replacement scope and limitations.
- Immutable selective-approval and historical-baseline workflows.
- Annotated corpus plus regression tests for every preset.

## Completion gate

Local implementation evidence is distinct from the non-developer completion gate;
that gate remains **NOT QUALIFIED** pending independent and native/human review.

Without recrawling, a user changes sensitivity and masks a dynamic area while
the original evidence remains recoverable. The test corpus flags a small
significant change on a long page rather than hiding it in the global percent.
Image mode is selected before capture and cannot silently differ from the
baseline. Approving one page/profile creates a new baseline containing that
capture plus unchanged inherited references, with provenance visible; the
previous baseline/run remains intact and can be restored.

## Next

Phase 6 makes this complete review state safe to retain, clean up, and share as
a portable selected-baseline project archive. It is **not started** by this work.
