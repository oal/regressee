# Comparison explanations and viewer focus — 2026-10-16

**Historical warning-veto repair; superseded policy.** The user subsequently
rejected the long warning and the blanket veto on reviewing useful screenshots.
Loading warnings are now advisory and valid saved pixel verdicts are usable
immediately. The viewer-focus repair remains. Current behavior and verification:
`docs/research/advisory-warnings-20261016.md`. Statements below about unchanged
warning/approval eligibility describe the earlier implementation, not current policy.

The user rejected the earlier footer as unfinished: it redirected them to Details
without explaining why comparison was blocked, and clicking an image filled its
caption and gap area teal. The earlier generic/synthetic inspection did not
qualify either reported state. This record supersedes its acceptance for these
two findings, not unrelated functionality.

## Recorded cause and repair

A read-only query restricted to the reported project's latest displayed analysis
found 25 `NotComparable` results. Both capture sets recorded `ImagesIncomplete`,
`ResourceFailed` and `EmbeddedContentBlocked`, with stable captures, no analysis
gates, no rendering warnings and no analysis failures. The existing analysis
service rejects capture warnings separately from its pixel-analysis gates.
Neither capture-only Details nor the previous footer explained that distinction.
The user's database, screenshots and baseline were not changed.

- `comparison_ux.rs` derives one selected-result explanation from saved failures,
  comparison gates and correctly bound Before/After capture metadata. Shared
  warnings say **Both screenshots**; different reasons name their side. Unknown
  reasons remain explicitly unknown, rather than guessing a browser/profile issue.
- The workspace shows the actual cause and safe next step directly below the
  screenshots. Details and Analysis use the same explanation. Supporting
  diagnostics include the selected saved result, report and failure; the optional
  **Technical details** action expands them. Copying remains explicit.
- An incomplete saved Before cannot be repaired by another check. The guidance
  explains that replacing it requires capturing and explicitly saving a baseline.
  No comparison, warning, note, selection or approval eligibility rule is relaxed.
- Long recorded reasons remain complete in a bounded 108px scroll owner with an
  overflow-only thumb; the recovery stays outside it and evidence remains visible.
- The teal came from a filled shadow behind a transparent focused viewer. Focus
  now changes only a reserved 1px border to ink, retaining the captions' background,
  text, pane gaps, original pixels and layout. Pointer and keyboard share it.

## Verification

- Native shell: **201 passed, 0 failed, 1 ignored, 1 filtered**, with
  `--skip qualification --skip bulk_consent`. Two new tests cover all five
  comparison gates, rendering mismatch, saved failures, unknown reasons,
  Before/After source binding, consistent Details, simulated pointer/Tab input,
  restoration and long-reason scrolling. Tests assert no stored evidence changes.
- The capture fixture saves the same three warnings and runs the actual analysis
  service. It reproduces `NotComparable` despite empty engine gates/warnings and
  no failures. This is a synthetic equivalent, not the user's private imagery.
- Personally inspected all **12** final Linux GPUI offscreen frames at
  **1280×800 and 820×600**: inline, pointer focus, keyboard focus, Details, Analysis
  and scrolled long failure. First inspection prompted removing competing numeric
  trigger reasons from the blocked Analysis summary and bounding long messages;
  the second batch confirms those repairs.
- All PNG hashes and seven current-source hashes per final frame match. RGB pixel
  crops of both captions and the pane gap are identical before/after focus in
  all four pointer/keyboard captures. The border remains visible independently.
- Offscreen calibration passed with 1,024 exact pixels per calibration colour.
  Strict `ui-renderer-capture` test-target Clippy, production `ui` build and diff
  whitespace check pass. App-library tests: **125 passed, 2 ignored**.

Final private evidence:

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-comparison-confirm-868mn_0f
```

`comparison-recovery-manifest.json` lists the frames;
`comparison-recovery-focus-probes.json` records crop bounds;
`comparison-recovery-verified.json` records RGB/hash checks. Reproduce through
`comparison_ux_tests` and the opt-in
`ui_renderer_capture::ui_review_footer_capture::renderer_capture_comparison_recovery`,
with the established private-root/calibration/website-fixture contract.

Unrelated screenshot-loading edits appeared concurrently in shared controls and
loading-specific files. They were not changed or reverted by this repair. Test
results describe the observed combined working tree, not ownership or acceptance
of that separate work.

These are actual GPUI offscreen frames and simulated input, **not** OS-input,
window-presentation, assistive-technology, Windows/macOS, whole-app usability or
release qualification. No new assets, shipping rasters, palette or dependencies.

## Fresh finish review

A fresh independent reviewer opened all 12 confirmation frames, rechecked PNG and
seven source hashes per frame, and independently confirmed identical caption/gap
RGB pixels in all four focus captures. It found no material user-facing layout,
cause-truth or focus-paint defect within this repair. Its **fix** disposition
identified only the scheduled documentation update: replace the obsolete
View-details-first guidance, record the transparent-viewer border exception and
distinguish the rejected earlier evidence. Those prose changes are now recorded
in DESIGN and the earlier report; their verdict pass is separate from UI review.
The production `ui` build also completed successfully after the review packet was
sent. No source changed after confirmation capture.

The reviewer subsequently scored the sole persistence fix **resolved**, final
disposition **ship** for that scored fix, with all 12 PNG and seven-source hashes
unchanged. This does not expand the original scoped UI verdict into whole-app or
native-input/accessibility/release approval.

## Documentation check

- Checked `PRODUCT.md`, the surface's rejection/repair addendum, corrected
  `DESIGN.md` and `.impeccable/design.json` against
  `crates/app/src/{comparison_ux,review_ux,viewer_ux,analysis_ux,capture_diagnostics,main}.rs`
  and the warning-fixture/test wiring. The prose matches saved, side-bound inline
  causes, optional Technical details, shared inspector explanations, the 108px
  reason scroller with recovery outside it, and border-only viewer focus. No
  task-introduced contradiction requires attention; earlier View-details-first
  guidance is superseded by the repair addendum and historical-report notice.
- Reused incumbent navy/teal/mint and semantic ink roles, installed system type,
  14px/20px actions, 13px/18px help, 40px-minimum controls, small corners and ruled
  grouping. `ui/theme.rs` agrees with DESIGN's unchanged token frontmatter. This
  is an ordinary extension, not a new visual world; no system files were rewritten.
- Personally inspected all 12 manifest frames at both desktop sizes; independently
  rechecked their PNG hashes, seven scoped source hashes per frame and caption/gap
  RGB equality in all four focus captures. These private synthetic GPUI rasters
  are review evidence, not shipping assets or the user's screenshots. Reused logo
  originals/derivatives match `assets/brand/provenance.json`; no new raster,
  decoration, token or dependency is needed or introduced by this repair.
- Pre-existing sidecar drift remains recorded, not canonized or repaired:
  `nativeGeometry.approvalDialog` says 420px/580px instead of DESIGN's 520px/740px;
  `nativeFocus` says 2px spread rather than the ordinary shared 5px outer/2px gap
  treatment and does not describe the viewer exception. DESIGN remains authority.
  Concurrent screenshot-loading/shared-control work is outside this check and is
  neither changed nor accepted here.
- Documentation/source/offscreen check only, not another finish verdict. Reported
  build, Clippy and test results were not rerun; OS input/presentation, assistive
  technology, Windows/macOS and release qualification remain outstanding.
