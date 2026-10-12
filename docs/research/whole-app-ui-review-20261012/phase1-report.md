# Phase 1 — truthful capture fixtures

**2026-10-12 · implementation complete, independent verification pending.**
Only `crates/app/src/ui_review_matrix.rs` and evidence tooling changed. No
production UI edit, commit, push, website visit or tool download. Parent's existing
plan authorization changes were preserved. Phase 2 has not started.

## Outcome and scope

| Finding | Correction and verified pixels | Remaining limit |
|---|---|---|
| E1 | Navigate to Projects, assert NewProject mounted, activate supported create route. Assert Setup/Website, absent project/saved state/evidence, blank name/address. Both fields have fully visible bounds at both sizes. | No save or live first-run browser exercised. |
| E2 diagnostics | Open Details and activate mounted Phase5(38), assert details/diagnostics flags. Scroll **review_details_scroll**, not screen_scroll, to capture-provenance-details. Assert positive width/height, horizontal containment and visible beginning within its own viewport. | Provenance is longer than viewport; only its beginning was inspected. Narrow frame shows baseline/run/analysis identifiers, not the complete artifact/environment tail. |
| E2 advanced | Open Analysis, activate mounted Phase5(3), assert review_tools/analysis_advanced. Scroll same bounded body to field-Color tolerance (ΔE00); assert entire first field (label/input/help) visible. | Narrow frame establishes first complete parameter only. Other eight parameters/checks are not all inspected. Wide frame shows additional controls, but is not a full advanced-settings qualification. |
| E3 modes | At narrow width open Analysis first, assert Phase5(8/9) mounted, activate, finish reads, assert viewer_mode=2/3, close flyout. Assert visible positive full pane bounds: phase5-before for Overlay; phase5-difference for Heatmap. Originals visibly label each mode and paint overlay/heatmap, not Compare. | Simulated control activation, not OS pointer/keyboard input. Wide mode captures were already valid and were not regenerated. |
| E3 zoom | Bind zoom component actions; retain Guided(46), which source actually routes to ZoomSelect::toggle. Assert positive fully visible zoom-popup, zoom-option-0 (Fit width) and zoom-option-1 (Actual size), both sizes. | Old zoom images already showed the open menu; new PNG hashes are identical. This correction adds explicit assertion/source binding, not a visual change or native-accessibility pass. |

New matrix entries retain geometry and record visibility assertions including the
actual clipping owner's bounds, plus the compiled harness SHA-256. Retained entries
now preserve their existing matrix geometry rather than losing it during partial
recapture. Existing immutable archive records are untouched.

## Evidence and preservation

Root (all paths below relative to it):

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-whole-app-review-9inkus48
```

`superseded-phase1-20261012/` contains original affected PNG/JSON pairs,
`whole-app-matrix.json`, `gallery-manifest.json`, `index.html`,
`production-source-binding.json` and all 32 original contact sheets.
`preservation.json` records the 20 affected-file hashes and 172 unaffected PNG
hashes before rendering. Originals were **moved out of active names**, not left
for COMPLETE_BATCH to silently skip. No unrelated capture was removed.

Exactly 10 active pairs were regenerated:

- Both 1280×800 and 820×600: `new-website`, `review-diagnostics`,
  `review-analysis-advanced`, `review-zoom-menu`.
- Only 820×600: `review-overlay`, `review-heatmap`.

The full-resolution originals above were read together in **one batched inspection
of all 10**. Clean fields, disclosed technical identifiers, advanced color parameter,
open fit/actual-size menu and distinct named Overlay/Heatmap were visible. No
post-inspection edit or corrective confirmation was necessary. No claim that all
182 original images were individually inspected is made.

Fresh gallery: `index.html`, `gallery-manifest.json`, `contact-sheets/` (32 sheets).
182 entries, 142 whole-app entries, **182 unique PNG hashes**; hash nonidentity is
not the state proof. Eight corrected PNGs differ; the two zoom PNGs match their
already-open originals. All 172 unaffected PNGs, including all task captures and
wide Overlay/Heatmap, remain unchanged. Gallery/individual metadata PNG hashes and
header dimensions agree. The original A/B reports assessed the **original batch**,
not this corrected one. Their critiques and inspection ledgers remain unchanged.

Durable old/new PNG and metadata hashes, current 52-file source binding, visibility
geometry and coverage are in [phase1-correction-receipt.json](phase1-correction-receipt.json),
also copied into the evidence root. The original binding's historical harness
mismatch remains intact; current binding is separately recorded, never substituted
into the original record.

## Commands and results

Context loaded once using:

```bash
/var/home/olav/.config/opencode/skills/impeccable/scripts/impeccable context
```

Read full plan, full A/B reports, README/receipt, workflow brief, resolved
PRODUCT/DESIGN/surface context and native audit guidance. The context loader's
adaptive mobile references were not applied to desktop GPUI controls. No production
UI edit occurred, so no craft-floor or redesign workflow was required.

Preservation used Python pathlib/shutil: create the named superseded directory with
`exist_ok=False`; hash then move the ten named PNG/JSON pairs; copy the four metadata/
gallery/binding files and entire contact-sheets directory; hash all remaining
all-/task- PNGs into preservation.json. This operation is deliberately not repeated.

Exact capture command, run with tool timeout **0** twice:

```bash
LIBRARY_PATH=/var/home/olav/Projects/Regressee/.runtime/link-libs \
TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode \
REGRESSEE_RENDERER_CAPTURE_DIR=/var/home/olav/Data/opencode/tmp/opencode/regressee-whole-app-review-9inkus48 \
REGRESSEE_RENDERER_COMPLETE_BATCH=1 \
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::ui_review_matrix::renderer_capture_whole_app \
  -- --ignored --exact --test-threads=1
```

1. `phase1-renderer.log`: exit **101**, assertion failure after 44.41s. Test
   mistakenly expected phase5-after for Overlay; production correctly uses
   phase5-before for Overlay and phase5-difference for Heatmap. Mode state assertion
   had passed. This was an assertion-selector error, not an unavailable product route.
   Attempt-generated affected pairs were moved into
   `superseded-phase1-20261012/assertion-attempt-1/`, preserving them; all ten affected
   active names were absent before the next run. No visual inspection had yet run.
2. Corrected those two test selectors. `phase1-renderer-final.log`: exit **0**,
   **1 passed**, 116 filtered out, 49.95s. All ten new pairs passed state/visibility
   assertions; end-of-run serialized stored project and project evidence equal
   their pre-render values. In-memory presentation clones were not persisted.

```bash
python3 scripts/ui-review-gallery.py /var/home/olav/Data/opencode/tmp/opencode/regressee-whole-app-review-9inkus48
python3 scripts/ui-review-phase1-receipt.py /var/home/olav/Data/opencode/tmp/opencode/regressee-whole-app-review-9inkus48
LIBRARY_PATH=/var/home/olav/Projects/Regressee/.runtime/link-libs \
TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  -- --test-threads=1
git diff --check
```

- Gallery exit **0**: 182 images, 32 sheets; `phase1-gallery.log`.
- Receipt exit **0**: 10 regenerated pairs, 8 changed PNGs, 172 unchanged PNGs;
  12 original archive records and all eight product UI hashes byte-identical.
- Binary tests exit **0**: **102 passed, 0 failed, 15 ignored**, 27.31s;
  `phase1-bin-tests.log`. Not the complete workspace/integration/product test suite.
- Diff whitespace check exit **0**. Python receipt is executable via python3 and
  can be rerun by the verifier without rendering.

Existing warnings: two vendored GPUI f32 fallback future-compatibility warnings,
and proc-macro-error2 future incompatibility. No unrelated warning fixes attempted.

## Qualification / next handoff

E1–E3's misleading named states are now established at their relevant sizes, with
the explicitly bounded E2 coverage above. This is **synthetic Linux GPUI/Blade
offscreen evidence only**. Real OS presentation/input/IME/accessibility,
Windows/macOS, live browser installation/login/capture, 100–1,000 independent
comparisons, bulk baseline approval, runtime throughput and usability/release
acceptance remain untested. Synthetic fixtures remain four projects, twelve
stored references and four reused site images; 148-target clones remain
presentation-only. Do not infer general UI acceptance from 182 unique hashes.

Next: separate independent Phase 1 verifier checks code/assertions, immutable
archive/product hashes, corrected originals/receipt and gallery agreement. Parent
commits/pushes only after verification, then begins Phase 2. No blocker to that
verification; native/runtime/scale qualifications remain later-phase limits.
