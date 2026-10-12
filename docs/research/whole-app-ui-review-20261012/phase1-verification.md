# Phase 1 independent verification

**2026-10-12 · Verdict: PASS — Phase 1 evidence-correction gate only.**

Independent GPT 6.1 Sol verifier reviewed the plan, implementation report,
correction/archive receipts, full harness diff and receipt checker, mounted-control
source paths, stored-data preservation assertions and available corrected pixels.
No production implementation changed. No commit/push or Phase 2 work performed.
No blocking remediation is required for the implementation agent.

## Independently executed checks

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-whole-app-review-9inkus48`.

```bash
python3 scripts/ui-review-phase1-receipt.py /var/home/olav/Data/opencode/tmp/opencode/regressee-whole-app-review-9inkus48
LIBRARY_PATH=/var/home/olav/Projects/Regressee/.runtime/link-libs \
TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  -- --test-threads=1
git diff --check
```

- Receipt checker **exit 0**: 10 corrected pairs, eight changed PNGs, 172 unchanged
  PNGs. The checker deterministically rewrites the correction receipt; repository
  and evidence-root copies agree.
- Binary tests **exit 0: 102 passed, 0 failed, 15 ignored**, 21.13s (build 0.41s).
  Full output: `phase1-verifier-bin-tests.log` in the evidence root. Includes
  creation/save, disclosures, zoom, geometry, draft preservation and stale-evidence
  tests. This is not the full workspace/integration suite. Renderer tests remained
  ignored; no independent recapture was run.
- Whitespace check **exit 0**.
- Supplemental read-only Python assertions **passed**: every original gallery
  PNG hash maps to its preserved affected file or unchanged active file; all 142
  current matrix entries agree field-for-field with their gallery entries; all
  corrected visibility rectangles satisfy the recorded owner bounds; all 182 HTML
  cards, PNG sources and metadata links agree with the gallery; 32 original contact
  sheets exist; all 32 current sheet hashes and every evidence-artifact hash match
  the receipt; receipt copies agree; current PNGs have 182 unique hashes.

Existing GPUI f32 fallback and proc-macro-error2 future-compatibility warnings
remain. They do not establish a Phase 1 failure or release readiness.
Bookkeeping clarification: the original binding has 52 entries; the current
receipt has **53**, including the new receipt-checker script. The implementation
report's “current 52-file source binding” describes the original source set, not
the full current receipt count; this does not affect hash integrity.

## Gate results

| Requirement | Independent result |
|---|---|
| Test-only source changes | **PASS.** Only `ui_review_matrix.rs` changes under `crates/`. Its enclosing module is gated by `cfg(all(test, feature = "ui-renderer-capture", target_os = "linux"))`. Other changes are evidence tooling/documentation. |
| Product hashes / immutable archive | **PASS.** All eight archived product UI hashes match. All 12 archive records, including original A/B, initial manifests and original source binding, remain byte-identical. Original 52-file binding matches current files except its explicitly historical harness mismatch; the new harness binding is separate. |
| E1 creation state | **PASS, bounded.** Projects route activates mounted NewProject; source `Shell::create` clears binding/evidence and fields. Assertions require Setup/Website, no saved project/evidence and blank inputs; both complete field bounds are visible at both sizes. No save/browser route qualified. |
| E2 diagnostics | **PASS, bounded.** Mounted Phase5(38) is activated after Details; flags asserted; actual `review_details_scroll` owner is scrolled. Positive provenance bounds begin inside that viewport and are horizontally contained. The long element is explicitly not fully visible. |
| E2 advanced | **PASS, bounded.** Mounted Phase5(3) opens advanced Analysis; flags asserted; the same bounded body is scrolled to the complete Color tolerance field. That field is fully inside its viewport at both sizes. Not all advanced controls qualified. |
| E3 Overlay/Heatmap | **PASS.** Narrow Analysis exposes mounted Phase5(8/9); actual mode 2/3 asserted, reads finished and flyout closed. Positive fully visible pane bounds use production selectors `phase5-before` / `phase5-difference`. Wide captures are unchanged, not falsely claimed as recaptured. |
| E3 zoom | **PASS.** `activate_control(Guided(46))` really calls `ZoomSelect::toggle`, not the unrelated direct `guided(46)` helper. Popup and Fit width/Actual size bounds are fully visible at both sizes. Old/new zoom PNG hashes are identical because the original menu was already open. |
| Preservation / affected-only capture | **PASS.** Ten old PNG/JSON pairs are preserved and hashed under `superseded-phase1-20261012/`; original matrix/gallery/binding match archived originals. All 172 unaffected PNGs retain original hashes. Only the ten intended matrix entries are fresh. Initial failed-attempt pairs remain separately preserved. |
| No persisted presentation clones | **PASS within fixture scope.** `mixed_queue` mutates shell-owned evidence in memory and invokes no storage write. Successful recorded renderer run reaches end-of-test equality assertions on serialized stored project and project evidence. The verifier reviewed these assertions and the matching successful log; it did not rerun the renderer. No 148-independent-result or bulk-approval claim is supported. |
| Gallery / manifests / pairs | **PASS.** 182 entries, 142 whole-app entries, 182 unique PNG hashes; PNG SHA-256 equals gallery and individual metadata, PNG header dimensions agree, native-offscreen/synthetic and unverified-OS flags remain explicit. HTML card agreement and derivative sheet hashes independently checked. |

## Bounded visual inspection and unresolved coverage

Read the ten existing corrected full-resolution originals in one verifier batch:
both sizes of `new-website`, `review-diagnostics`, `review-analysis-advanced` and
`review-zoom-menu`, plus narrow `review-overlay` and `review-heatmap`.
No renders, new contact sheets, polishing or corrective confirmation loop.
This read-only verification is separate from the implementation's one documented
inspection batch and does not change its inspection ledger.

Pixels show blank unbound creation fields, baseline/run/analysis provenance,
complete first advanced parameter, explicitly named distinct Overlay/Heatmap and
open Fit width/Actual size menus. Narrow diagnostics clips the provenance tail;
narrow advanced exposes only the first complete parameter. Wide frames show more,
but do not establish every lower control. **These tails remain unresolved coverage
allowed by the Phase 1 gate**, not fully inspected diagnostics/advanced-settings
acceptance. Unique hashes alone are not state proof.

All evidence remains synthetic Linux GPUI/Blade offscreen. Simulated activation,
focus and layout tests establish neither real OS input/presentation/IME nor native
accessibility. Windows/macOS, live browser installation/login/capture, genuinely
distinct large-scale comparisons, storage-backed bulk approvals, throughput,
participant usability and release qualification remain untested. Original A/B
reviewed the original batch, not corrected pixels.

**Next authorized handoff:** parent reviews this PASS and commits/pushes the
verified Phase 1 work before assigning Phase 2. This verifier does neither.
