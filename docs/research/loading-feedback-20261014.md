# Loading feedback — 2026-10-14

## Finding and scoped change

The user’s “Opening project…” screenshot showed the shared compact activity
control, not a list. Its static 6px filled dot visually behaved like a bullet.
Related loading states in Projects, History and the screenshot/evidence viewer
had only text; browser checking used a static clock. Several initial reads also
repeated a global status above their inline loading explanation.

- Replace the dot with a shared 16px native stroke spinner and readable task text.
- Give global operations a quiet white status surface, minimum height 40px.
- Use the same indicator for loading placeholders, browser checks and storage
  reads. Align supporting help beneath its heading text.
- Keep one status owner for the same initial read. Preserve feedback for saves,
  errors and refreshes that keep previously loaded evidence visible.
- Use the existing 1.2-second native activity clock; reduced motion keeps a
  recognizable static mark. No fabricated percentage, timer or worker polling.
- Preserve palettes, navigation, backend operations, safety guards, task progress,
  passive statuses, error recovery and untinted screenshot evidence.

## Bounded verification and finish verdict

**Scoped verdict: pass.** The compact indicator is distinguishable from a list
marker, labels wrap beside a stable icon, and inline initial reads no longer
duplicate their global status. Both desktop sizes retain navigation and task
access. This is an in-thread finish review, not independent or native release
qualification.

- Production native build passed: `cargo build --locked -p regressee-app
  --features ui --bin regressee`.
- Shared UI suite passed: **51 tests**, including contrast, indicator geometry,
  long-label wrapping, reduced-motion stability and existing progress semantics.
- New shell ownership/layout test and explicit renderer test passed. The shell
  test covers nine states at 1280×800 and 820×600, checks error/save feedback is
  not suppressed, and verifies unchanged saved evidence.
- Existing `project_loading_never_claims_missing_screenshots_before_evidence_arrives`
  regression passed.
- Confirmation contains **18 real GPUI/Blade offscreen captures**: opening a
  project; initial Projects, History and saved evidence reads; screenshot read;
  browser verification; storage read; saving; and a deliberately long operation
  label. Storage is scrolled into view when needed. Reduced motion freezes the
  indicator for deterministic captures.
- The first inspection batch was incomplete because its fixture exposed a second
  unrelated storage-loading indicator below the viewport. The fixture now supplies
  known browser/storage states and explicitly reveals the storage case; the final
  batch passed. An initial long-label assertion incorrectly applied the bounded
  review limit to Projects and was corrected to check available window geometry.
- Full library run at verification time: **112 passed, 1 failed, 2 ignored**.
  The unrelated failure is
  `product::automatic_runtime_tests::missing_trust_is_build_incompleteness_and_status_never_installs`,
  whose expected recovery wording differs from concurrent runtime-copy changes.
  No whole-suite pass is claimed or unrelated runtime edits repaired.
- A concurrent History syntax error briefly blocked the build. The other agent
  fixed it; this task did not edit History to unblock verification.

Commands used `LIBRARY_PATH=$PWD/.runtime/link-libs`. Renderer commands additionally
set the approved capture root and used `--features ui-renderer-capture --bin
regressee loading_feedback -- --include-ignored`.

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-loading-20261014-k9KCk9nb/`.
The root holds partial inspection evidence; `confirmation/` holds all 18 final
frames, JSON source-hash sidecars and the manifest. Two size-specific contact
sheets summarize the final batch. No new shipping raster was created.

These are synthetic offline Linux states, not actual website results, OS-input,
screen-reader, cross-platform or public-release acceptance. Concurrent agents’
changes remain intact. `DESIGN.md` and its native sidecar record the shared pattern.
