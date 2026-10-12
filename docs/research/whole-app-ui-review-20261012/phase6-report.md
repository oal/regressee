# Phase 6 — implementation handoff

Started **2026-10-12**, completed **2026-10-13**, base **a37cf8c** (verified/pushed
Phases 1–5). **Implementation complete; independent verification pending.** No
commit, push, nested agent, Phase 7, browser visit, download or release work.

## Built scope

- **H1:** 256px Runs/Baselines scroll owners; full-width selection preserved.
  Clear local filter label/scope, overflow/Tab help and paging on both sections.
  Selected type/date/coverage/environment and actions together; IDs opt-in.
- **H2:** share/import/delete lead with consequences. Deletion uses record
  type/date rather than Rust debug identity; exact identities/bytes/recovery lists
  remain available through a separate Portability disclosure. Storage leads with
  human units, exact bytes under advanced provenance. Sessions/cookies exclusion,
  original-pixel privacy, ignored rectangles NOT redaction, recipient login,
  import-as-new/no-visits and shared/retained source protections preserved.
- **H3:** actual temporary/recovery deletions use danger treatment. Zero-impact
  actions are disabled, excluded from Tab and reject activation without queuing.
  Temporary-only cleanup remains distinct from complete recovery-original deletion.
- **F1:** a *real persisted failed check target* can return a frame with a valid
  baseline original but no after image. The former cloned `viewer=None` scene was
  not representative. The production outcome now selects local identity/cause/
  recovery, no fake image and no Analysis/Zoom. Relevant Details remains mounted.
  Real cold actor reads of missing/corrupt originals preserve known validated
  metadata and fail closed; update presentation is blocked until image-read success.
  Refresh is a disk read, not recapture; no targeted retry is promised.
- **F2:** paused/interrupted continuations say Return to capture recovery;
  completed checks say Review check results. Actual continuation destination and
  existing workspace Resume stay intact. Failures retain the operation name.
- **Remaining M1:** recorded inspection no longer repeats its persistent read-only
  heading in a banner; authoritative empty captures have no blank queue or
  misleading unloaded target/profile labels. Phase 4 cap/modes/zoom remain intact.

Owners: `crates/app/src/{workspace_history,main,loading_ux,workflow,task_ux}.rs`,
`crates/core/src/shell.rs`. Test-only changes:
`phase6_history_tests.rs`, `ui_review_matrix.rs`, `ui_renderer_capture.rs` and
strengthened incumbent tests. `DESIGN.md` records built behavior, not acceptance.

## Checks and actual results

All Cargo invocations used:

```bash
LIBRARY_PATH="$PWD/.runtime/link-libs"
TMPDIR=/var/home/olav/Data/opencode/tmp/opencode
```

These are environment assignments on the actual commands, not exports changing
the user's shell. Commands:

```bash
LIBRARY_PATH="$PWD/.runtime/link-libs" TMPDIR=/var/home/olav/Data/opencode/tmp/opencode cargo test --locked --offline -p regressee-app -p regressee-core -p regressee-storage --all-features
LIBRARY_PATH="$PWD/.runtime/link-libs" TMPDIR=/var/home/olav/Data/opencode/tmp/opencode cargo test --locked --offline -p regressee-app --all-features --bin regressee phase6 -- --test-threads=1
cargo fmt --all -- --check
git diff --check
python3 scripts/phase6-evidence-receipt.py /var/home/olav/Data/opencode/tmp/opencode/regressee-phase6-5nm4hts3 --write
python3 scripts/phase6-evidence-receipt.py /var/home/olav/Data/opencode/tmp/opencode/regressee-phase6-5nm4hts3
```

- Packages: **311 passed, 0 failed, 40 ignored**, exit 0. App library 95, binary
  122, app integration 8, core 51, storage 35. Ignored is not tested.
- Serial Phase 6: **4 passed, 0 failed**, exit 0. Missing/corrupt comparison
  originals through actual worker reads; separately mounted exact details/baseline
  paging; paused/interrupted/completed outcome action and continuation dispatch;
  native-picker simulation with real export/import/deletion-preview jobs and
  zero-impact activation rejection. Simulated input is not OS input.
- Incumbent regression coverage passes: last History row reveal/selection,
  full-width paint at both sizes, loading/failed reads, filters and selection
  guards, original-policy read-only inspection/stale replies/newer drafts,
  approval/storage protections, privacy and hostile archive checks, interrupted
  temporary versus explicit recovery GC, reference/journal rechecks and resume.
- Formatting/whitespace: exit 0. Receipt: **12 immutable archives, 182 original
  PNGs unchanged, 88 source bindings, 34 inspection + 34 confirmation pairs**.
  Confirmation main/task hashes match pair metadata; four tail assertions bind
  actual selected-row paint to the current inner owner and screen clip.

Logs (local, frozen by receipt): `phase6-packages.log`, `phase6-targeted.log`,
`phase6-render.log`, `phase6-tasks.log`, `phase6-confirmation.log`,
`phase6-confirmation-tasks.log` under `/var/home/olav/Data/opencode/tmp/opencode/`.

## Native evidence — bounded, isolated, no network

Root: `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase6-5nm4hts3`.
Confirmation: its `confirmation/` directory. Existing website fixture PNGs and
personally inspected renderer calibration were copied locally; nothing downloaded.
Prior roots untouched. Eight additional storage-backed baseline versions/source
runs reuse expressly synthetic imagery; this is retention/selection coverage,
not distinct real websites or a throughput benchmark. The failed account targets
are genuinely stored NeedsLogin outcomes analyzed by the real comparison code.

For **each** root, actual successful renderer commands used the environment above
plus `REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode`,
`REGRESSEE_RENDERER_CAPTURE_DIR=<root>` and `REGRESSEE_RENDERER_PHASE6=1`:

```bash
cargo test --locked --offline -p regressee-app --all-features --bin regressee ui_renderer_capture::ui_review_matrix::renderer_capture_whole_app -- --ignored --exact --test-threads=1
cargo test --locked --offline -p regressee-app --all-features --bin regressee ui_renderer_capture::renderer_capture_tasks -- --ignored --exact --test-threads=1
python3 scripts/ui-review-gallery.py <root>
```

Each renderer passed **1 test**, exit 0. Each gallery: **34 images, 8 sheets**.
Both 1280×800 and 820×600: 14 whole-app states and three relevant tasks each.
Read all eight first-batch sheets once; one corrective batch addressed the disabled
import disclosure, duplicate read-only notice, empty identity copy and scrollable
failure body. Read all eight confirmation sheets once and six originals:

```text
all-820x600-history-baselines.png       all-1280x800-history-bottom.png
all-820x600-import-preview.png          all-820x600-review-failed-target.png
all-820x600-history-recorded-check.png  task-820x600-capture-paused.png
```

Confirmation tail assertions: last actual stored run/retained baseline row is
fully visible, fills the actual inner scroll owner within 1px, and lies inside
the current screen clip at **both sizes**. No whole-window-wrapper substitution.
No production edit or new capture after this confirmation inspection.

## Attempt failures and corrections (not hidden passes)

Early compilation caught a missing `std::fs` import, sibling-method visibility,
static-string versus dynamic-copy use, and a temporary selector lifetime. Fixed.
Early tests revealed that unread initial metadata cannot name a target; the cold
image-read test now explicitly retains a real validated storage snapshot before
external disk corruption, not a cloned frame. It also exposed UI eligibility
after an image-read failure; the conservative failure guard was implemented.
The former zero-cleanup test expected a pointless mutation; it now proves disabled
activation preserves request count/preview. Existing analysis-switch tests now
load the real viewer before activating viewer-owned tools; failed-capture Details
reachability remains preserved rather than deleting that capability.

Capture preflights failed for absent calibration in the new root and 0755 rather
than required 0700 confirmation permissions; fixed locally. The first failed-target
capture assertion incorrectly expected `viewer=None`; actual storage returned a
before-only frame, leading to the outcome-bound production correction. These
pre-inspection failures are recorded here; the named local logs contain the final
successful invocations, not every overwritten attempt. No aesthetic render loop.

## Limits and verifier handoff

Not OS presentation/input, keyboard/IME/clipboard accessibility, Linux AT,
Windows/macOS, localization/text scaling, PM usability, release or runtime-install
qualification. The build's trusted-browser distribution limitation remains.
The task-complete renderer retains its old explicitly synthetic 148-target
presentation fixture; it is not 148 independent storage-backed comparisons.
The History filter is still page-local, not archive search. At minimum size the
initial History list extends below the outer fold; the fully exposed owner shows
four 64px rows and last-row selection is measured. Long failure/replacement advice,
import privacy and technical provenance require ordinary bounded/outer scrolling.
Exact-details opening is controller-tested; it has no separate expanded native
capture. Initial unread project metadata cannot provide an unknown target/profile;
it remains a truthful failed-read/Refresh state rather than fabricated evidence.

No known failing functional regression remains in tested scope. **Next:** separate
independent Phase 6 verification of source, tests, receipt and existing pixels;
do not mark verified or commit/push/start Phase 7 on this implementation report.
