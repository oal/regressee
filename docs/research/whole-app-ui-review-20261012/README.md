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

## Pending deliverables

1. Fixture correction receipt and corrected coverage addendum (plan Phase 1).
2. `synthesis.md`: complete final critique with reconciled evidence (Phase 2).
3. User-approved interaction blueprint before production UI changes (Phase 3).

No production UI improvement is implemented or authorized by this archive.
