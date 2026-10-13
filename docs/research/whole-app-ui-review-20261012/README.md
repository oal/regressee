# Whole-app UI review — preserved initial evidence

**Date:** 2026-10-12. **Continuation:** [phased plan](../../whole-app-ui-plan.md).

This directory preserves the independent reports and initial evidence metadata
before correcting misleading capture fixtures. The reports are copied
**byte-for-byte**, not rewritten after synthesis. See `archive-receipt.json` for
source locations, SHA-256 hashes, byte counts, current source hashes and the
working-tree snapshot at plan creation.

| File | Purpose |
|---|---|
| `assessment-a.md` | Independent whole-workflow design critique; 24/40; per-screen findings and inspection ledger |
| `assessment-b.md` | Independent native pixel/source/integrity assessment; geometry and fixture caveats |
| `assessment-b-integrity.json` | Original batch hash/dimension/coverage verification |
| `gallery-manifest.initial.json` | Original 182 gallery entries and image hashes |
| `whole-app-matrix.initial.json` | Original 142 whole-app entries; geometry where supplied |
| `production-source-binding.original.json` | Original source bindings; harness mismatch described in Assessment B |
| `proof-reviewed.json` | Renderer calibration review receipt |
| `detector.json`, `detector.stderr.txt`, `detector-run.json` | Actual empty detector output and command receipt; not native UI certification |
| `renderer-complete.log`, `task-renderer.log` | Successful whole-app/task capture receipts; not a complete test-suite result |
| `archive-receipt.json` | Archive integrity and continuation baseline |

PNG originals and contact sheets remain outside Git at:

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-whole-app-review-9inkus48
```

The root's `index.html` is the searchable evidence gallery. It displays native
offscreen captures; it is not the application's implementation. The prior server
URL is only a convenience, not an evidence dependency.

## Important interpretation

- 182 labels include duplicate/misleading fixtures. They do not establish 182
  distinct verified states. See Assessment B §3 and plan E1–E3.
- Both reports cover the **original batch**. Corrections must have a separate
  receipt/addendum and preserve superseded PNG/JSON pairs before recapture.
- Website imagery, projects and task data are synthetic. The 148-target review
  exercises presentation scale only, not real high-volume approval or throughput.
- All native captures are Linux offscreen evidence. No OS input, accessibility,
  Windows/macOS or release pass is implied.
- The initial matrix has missing geometry for retained captures. Do not invent
  exact bounds or rewrite the preserved matrix as if every row had measurements.

## Continuation deliverables

The early handoff below is historical; Phases 4–7 and preview delivery have since
completed within the documented feasible scope. See the final checkpoint below.

1. [Phase 1 report](phase1-report.md) and [independent verification](phase1-verification.md): bounded evidence PASS, committed/pushed `ac4de65`.
2. [Complete synthesis](synthesis.md) and [independent verification](phase2-verification.md): documentation/evidence PASS, committed/pushed `d4b3ddb`.
3. [Workflow blueprint](workflow-blueprint.md), [Phase 3 report](phase3-report.md) and [independent verification](phase3-verification.md): documentation/inventory PASS; selected under explicit delegated user judgment. Not participant approval or built UI.

No production UI improvement is implemented by these phases. Phases 1–7 are
authorized by the later 2026-10-12 user instruction recorded in the plan, not by
the original archive or assessments. Next: authorized Phase 4A consequential consent;
parent commits/pushes verified docs before Phase 4A.

## Final delivery checkpoint — 2026-10-13

All seven phases are implemented, feasibly verified, committed and pushed.
Cross-platform Foundation/Phase 0 and Linux/Windows preview publication passed at
`1911c3b9905d727ac830a420eb6935ceb4d380e9`.
[Published unsigned experimental preview](https://github.com/oal/regressee/releases/tag/v0.1.0-preview.2)
has five independently downloaded and integrity-checked assets.

- [Final independent delivery verdict and direct downloads](delivery-release-verification.md)
- [Hosted main CI evidence](delivery-hosted-verification.json)
- [Publication/download receipt](delivery-release-receipt.json)
- [CI implementation and historical timeout report](delivery-report.md)
- [Independent local/timeout verification](delivery-verification.md)
- [Phase 7 feasible verification and remaining platform gates](phase7-verification.md)

This closes implementation/feasible validation and preview availability, **not**
native input/AT, clean installation/runtime first-use, participant usability,
complete license or supported-release acceptance. Final delivery docs postdate the
released source archive; the release remains bound to the tested commit above.
