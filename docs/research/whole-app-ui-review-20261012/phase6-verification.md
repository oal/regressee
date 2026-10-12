# Phase 6 — independent verification

**2026-10-13 · PASS — substantive source/controller/offscreen exit gate.**
Base: `a37cf8c`. H1–H3, F1–F2 and remaining M1 pass within the evidence scope
below. No blocking Phase 6 product defect found. This is not native-input,
accessibility, usability, platform or release qualification.

Read the plan, workflow blueprint, synthesis, implementation report/receipt,
complete tracked diff and new test/receipt files. Checked the changed production
owners against the real worker, storage, focus and inspection paths, rather than
accepting labels or the implementation report as proof. Existing user/implementer
changes were preserved. No product/test source edit, render, polish, commit, push,
delegation, network visit or Phase 7 work.

## Independent execution

Cargo commands used `LIBRARY_PATH="$PWD/.runtime/link-libs"` and
`TMPDIR=/var/home/olav/Data/opencode/tmp/opencode` in the repository root:

```bash
cargo test --locked --offline -p regressee-app -p regressee-core -p regressee-storage --all-features
cargo test --locked --offline -p regressee-app --all-features --bin regressee phase6 -- --test-threads=1
cargo test --locked --offline -p regressee-app --all-features --bin regressee history -- --test-threads=1
cargo test --locked --offline -p regressee-storage --all-features portability -- --test-threads=1
cargo check --locked --offline -p regressee-app --features ui --bin regressee
cargo fmt --all -- --check
git diff --check
python3 scripts/phase6-evidence-receipt.py /var/home/olav/Data/opencode/tmp/opencode/regressee-phase6-5nm4hts3
```

- Packages: **311 passed, 0 failed, 40 ignored**, exit 0. App library 95,
  binary 122, integrations 8, core 51, storage 35. Ignored tests are not passes.
- Serial Phase 6: **4 passed**, exit 0. Actual missing/corrupt original reads,
  independent exact-details disclosure, task continuation dispatch and actual
  ZIP/import/deletion-preview worker dataflow, with simulated picker completion.
- Serial History regressions: **18 passed**, exit 0. Both-size last-row reveal/
  selection and width, loaded filters/action guards, page refresh identity,
  stale inspection/newer drafts and saved-policy read-only inspection included.
- Serial storage portability/recovery regressions: **3 passed**, exit 0.
  Dead/live/partial stages, stale snapshots, explicit recovery GC/reference
  journal rechecks and disk-full/interrupted rollback included.
- Production `ui` feature check, formatting and whitespace: exit 0.
- Receipt: exit 0; **12 immutable archives, 182 original PNGs unchanged,
  88 source bindings, 34 initial + 34 confirmation pairs**.

Independent logs reside in `/var/home/olav/Data/opencode/tmp/opencode/`:
`verifier-phase6-{packages,targeted,history,storage,ui-check,ui-feature}.log`.
They are deliberately outside the implementation receipt's `phase6-*.log` glob.
An initial receipt invocation detected my newly named verifier log in that glob;
renaming that verifier-owned file removed the sole drift. No frozen log, receipt
or evidence file was rewritten to obtain the pass.

### Supplemental failed feature probe — not hidden

`cargo test --locked --offline -p regressee-app --features ui --bin regressee
--no-run` exited **101**. That test configuration omits `gpui/test-support` and
`ui-tests`-gated fixtures, but includes ungated GPUI test modules. Diagnostics name
`TestAppContext`, `approval_fixture` and `workflow_refinement_tests`, including
the new Phase 6 tests. `git show HEAD:crates/app/src/task_ux.rs` establishes that
the base already contains the ungated GPUI test module and fixture-dependent
tests; the baseline executable was not separately built. The new module repeats
that existing harness pattern. This is a recorded feature-configuration limitation,
not a newly demonstrated product-build regression or a substantive H/F/M failure.
The intended `ui-tests`/all-features runs and production `ui` check pass. Do not
claim every test-feature combination works; qualification should account for this.

## Substantive gate findings

| Gate | Verified result |
|---|---|
| **H1 — record discovery and identity** | Runs/Baselines have actual 256px inner owners. Last loaded run `HistoryRun(9)` and last retained baseline `HistoryBaseline(7)` are selected, fully painted and contained at both sizes. Loaded-page filter copy is truthful; storage pages 64 records independently for runs/baselines and exposes a shared `more` flag. Both sections dispatch real History paging. Selected date/type/coverage/environment and actions are associated; IDs remain retrievable. Filtered-out selections do not expose item actions. |
| **H2 — consequences/privacy/exact detail** | Default deletion names comparison/capture type and UTC date or baseline save date, not `Run(RunId(...))`. Irreversibility, protected shared data and retained/active sources precede explicit danger confirmation. ZIP/import preserve original-pixel privacy, ignored areas NOT redaction, no sessions/cookies, recipient login, new-project/no-visits and mask revalidation. `Detail::Portability` is independently mounted on Projects/History/Settings, toggles without queuing work and is cleared by Cancel; exact bytes/record and recovery lists remain available. Storage leads with human units and discloses exact accounting. |
| **H3 — zero/danger/distinct safety** | Zero temporary/interrupted-stage and zero complete-recovery work are disabled by separate `control_enabled` predicates. Shared Tab filtering excludes disabled controls; `activate_control` rejects before dispatch. The actual zero temporary activation regression preserves request count and preview. Both real deletion buttons use Danger. Temporary cleanup cannot delete complete recovery originals; the separate explicit recovery deletion rechecks its snapshot and reference-safe GC journal. Storage tests confirm stale/reference protections. |
| **F1 — real failures** | Fixture persists NeedsLogin outcomes through `begin_run`/`finish_run` and real comparison analysis; selected viewer is loaded through Jobs, not forcibly cleared. A valid before-only frame cannot make the failed target look like evidence or expose Analysis/Zoom: presentation and control mounting consult the selected recorded outcome. Local path/profile/environments, cause, login recovery and Details survive. Real cold missing/corrupt-original worker reads with retained validated metadata fail closed, preserve selection/cause and block update eligibility. `viewer_read_failed` survives feedback dismissal; success clears it. Initial unread metadata is not invented. Refresh reads disk; a new check and explicit rediscovery/capture/save replacement are described, not unsupported per-target retry or fake installation repair. |
| **F2 — actual intention/dispatch** | Paused/interrupted matched capture outcomes say Return to capture recovery; completed checks say Review check results. Serial tests exercise actual `continue_task` destination dispatch and confirm workspace Resume remains mounted for recoverable runs. Failure headings retain the operation title. No new Resume/Open folder behavior is implied by a renamed button. |
| **Remaining M1 / inspection** | Authoritative empty capture has no blank queue or unloaded target/profile identity. Recorded capture/check keeps the persistent inspection-only heading/Latest exit without a duplicate banner. Existing original-policy read-only guards reject mutation controls and late direct actions; stale epoch/project/run/selection/draft checks reject asynchronous inspection replacement. Capture cap, manual enlargement and mode/zoom regression tests remain passing. |

The package run additionally exercises real composite ZIP privacy/shared deletion,
hostile archives/atomic refusal, missing/corrupt historical originals, analysis/
approval guards, stale identities and loading/failed-read recovery. Simulated
controller input is not OS input. Some incumbent large-list/task fixtures are
presentation clones, not real throughput or archive-scale product results.

## Existing pixels and independent integrity/paint checks

Root: `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase6-5nm4hts3`.
Only existing files were inspected; no renderer was invoked.

Read all **8 confirmation sheets** (`all-{1280,820}-01..03` and
`task-{1280,820}-01`) and these **7 confirmation originals** once:

```text
all-820x600-review-failed-target.png
all-820x600-history-baselines.png       all-1280x800-history-baselines.png
all-820x600-history-bottom.png          all-1280x800-history-bottom.png
all-820x600-delete-preview.png          all-820x600-import-preview.png
```

An independent read-only Python check cross-checked both batches' 34 gallery
entries against PNG hashes, metadata, image dimensions and HTML; each 28-entry
matrix agrees with its gallery. Confirmation matrix harness hashes match current
`ui_review_matrix.rs`; receipt binds current production and test sources, and
confirmation pair metadata binds main/task source hashes. Initial hashes are
historical, not silently asserted to be final-source captures.

Parsed the four recorded bounds numerically, independently checking row inclusion
in the **actual inner owner and current screen clip**, not the whole-window wrapper:

| Size / row | Painted row x,y,w,h | Actual inner owner x,y,w,h |
|---|---|---|
| 1280×800 / run 9 | 217,260,998,64 | 217,68,998,256 |
| 1280×800 / baseline 7 | 217,257,1038,64 | 217,65,1038,256 |
| 820×600 / run 9 | 217,257,578,64 | 217,65,578,256 |
| 820×600 / baseline 7 | 217,257,578,64 | 217,65,578,256 |

PNG samples near **both row edges** at each selected tail are the real sky
selection `#e7f2fa`; inspection confirms text remains inside the row, rather than
mistaking a label-sized selection or wrapper width for full-width paint.

## Limits and disposition

- At 820×600 initial History extends below the outer fold. Exposing the full
  inner owner requires normal outer scrolling; the selected-tail captures prove
  the exposed owner, not an initially full visible History list.
- Long failure recovery needs its bounded body scroll; narrow import privacy and
  exact provenance need outer scrolling. Exact-details expansion is controller/
  source-tested, **not separately expanded-pixel-qualified**.
- Report wording says “Eight additional” baseline versions/source runs. The actual
  fixture has **eight total baseline versions/source runs, seven added** by
  `1..8`, plus the two check runs. This small accounting correction does not alter
  retained-tail coverage; none are distinct live websites.
- No OS presentation/input, native keyboard/IME/clipboard or AT, text scaling,
  Windows/macOS, PM usability or complete release qualification. The old complete
  task's 148 targets remain explicitly synthetic presentation-only. Trusted browser
  distribution/install limitations are not resolved by these checks.

**Disposition:** all requested substantive Phase 6 gates clear. Update only the
plan's verified checkpoint. Preserve implementation report/receipt as historical
handoff evidence. No finite product remediation is required in this scope; the
feature-test limitation and fixture accounting correction above remain explicit.
**Next: parent reviews and commits/pushes verified Phase 6.** This verifier does
not commit/push or start Phase 7.
