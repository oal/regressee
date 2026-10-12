# Phase 7 — whole-workflow qualification implementation

**2026-10-13 · base `135d3eb` · sole implementation agent, no nested agents.**
Independent verification is the next gate. Qualification infrastructure and finite
repairs are implemented; **full native/accessibility/usability/release acceptance
is BLOCKED, not complete**. [Platform matrix](phase7-platform-qualification.md)
names actual failures and unavailable acceptance rather than implying a pass.
No commit/push/tag, CI repair, distribution download or real project/site visit.

## Implemented qualification and finite repairs

1. `crates/app/tests/support/bulk_fixture.rs` creates **100 and 1,000 distinct
   persisted PageIds/TargetKeys/long URLs**, immutable baseline/check runs and
   independent generated original PNGs. Every page number is encoded in source
   pixels; CAS cannot turn the dataset into a repeated-image presentation clone.
   Before originals are all distinct; each check has distinct captured originals
   plus one genuinely persisted NeedsLogin gap. Real comparison analysis produces
   the results; no cloned classifications are saved. Runtime/rendering metadata
   explicitly says synthetic. The portable capture-mode protocol value is not a
   claim that Chromium produced the images. This is **storage/safety scale**, not
   browser throughput, real websites or PM efficiency. The old 148/3,000-row
   presentation fixtures retain their previous labels/limitations.
2. `crates/app/tests/qualification.rs` exercises one and 20 selected replacements,
   exact unchanged references, failed-target rejection, duplicate/stale baseline
   identity rejection, explicit restoration before accepting an old check against
   its original baseline, three retained versions and retained/shared source-run
   deletion protection. Export/import verifies every selected/inherited original
   byte-for-byte and imports no session references. Corruption **after preview**
   must reject commit, then restoring exact bytes permits the legitimate commit.
   Optional `REGRESSEE_PHASE7_STORAGE_EVIDENCE=<fresh approved root>` retains both
   real stores, ZIPs and compact qualification receipts for independent inspection.
3. `phase7_qualification_tests.rs` uses real Jobs/Store at **both window sizes**:
   eligible selection, a one-item preview/cancel, a second selection, no-match
   filtering that preserves both selected identities and the inspected target,
   a two-item/98-kept preview, all **100 real references** via Previous/Next and
   actual last-row containment, cached preview reopening and an explicit worker
   commit. Only the final narrow pass writes a new baseline; all other operations
   are preview/cancel/read. Older baseline JSON and every unselected reference
   remain unchanged. Simulated control/focus dispatch is not OS keyboard input.
4. `ui_review_matrix.rs` adds a restricted supplemental batch: diagnostics body
   tail, each of **nine** advanced analysis fields and the long suppression control.
   It scrolls the actual disclosure owner, measures actual input content and whole
   painted field/checkbox text, and never equates a window wrapper with its clip.
5. First batched pixel inspection exposed **two functional defects**, repaired in
   one batch: a reopened consent retained its previous tail scroll, concealing the
   consequences; the long checkbox label clipped. `main.rs::begin_approval_dialog`
   resets reading scroll, inspected reference and exact disclosure for both fresh
   replies and cached previews. `controls.rs::check` gives text a finite flexible
   column so it wraps. Confirmation establishes visible opening consequences and
   a **270×40px two-line label inside a 326×66px control**, at both sizes. No
   aesthetic third render/inspection round or subsequent rendering edit occurred.
6. The genuine 1,000-page run exposed a **quadratic worker verification path**:
   analysis and consent loaded/rehashed a whole source run for every reference.
   `app/src/review.rs::analyze_run` and `storage/src/review.rs::approval_preview`
   now retain **one** fully verified source snapshot only within that operation;
   already-verified check provenance uses the current run. Memory does not retain
   arbitrary composite history; switching source identities reloads verification.
   Cancel is checked during request construction. No shared cache, snapshot across
   preview/commit, trust shortcut, UI-thread I/O or capture-pixel change. Commit
   calls a fresh preview under its transaction. This unpainted worker repair was
   made after pixel confirmation; the unchanged UI/harness is still source-bound,
   and final current-source tests separately qualify the backend. The old slow
   owned test processes were explicitly terminated and their failed logs retained.
7. `phase5_repair_tests.rs` now binds TextInput/TimezoneSelect exactly as startup
   does, asserts an **actual changed timezone**, and verifies replacement Custom
   **1300×700**, not merely dirty state/a single Custom count. Removed its unused
   mutable binding. `phase6_history_tests` and `task_ux` GPUI tests are correctly
   gated by `all(test, feature="ui-tests")`; production `ui` does **not** enable
   GPUI test support merely to make an unsupported test configuration compile.

## Actual validation

Final command/results, source-start hashes and full unfiltered logs are recorded
by `scripts/phase7-checks.py` and frozen in [receipt](phase7-receipt.json). It runs:

```bash
LIBRARY_PATH="$PWD/.runtime/link-libs" TMPDIR=/var/home/olav/Data/opencode/tmp/opencode
cargo test --locked --offline --workspace --all-features
cargo test --locked --offline --workspace --no-default-features
cargo check --locked --offline -p regressee-app --features ui --bin regressee
cargo test --locked --offline -p regressee-app --features ui --bin regressee
cargo test --locked --offline -p regressee-app --features ui-tests --bin regressee
cargo test --locked --offline -p regressee-app --features ui-inspection --bin regressee --no-run
cargo clippy --locked --offline --workspace --all-features --all-targets
cargo fmt --all -- --check
git diff --check
```

The environment block above describes assignments applied by the executable
script, not exports into the user's shell. It records every exit/duration and
preserves warnings/ignored tests. Its deadlines kill only the owned process group.
The final matrix source-start bindings must still equal current Rust sources.
Final root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase7-checks-9_07m0rr`.
**All nine commands exit 0**; source-start Rust hashes still match. Actual totals
sum each command's test-result lines (including workspace integration/doc suites):

| Final command | Passed | Failed | Ignored | Elapsed |
|---|---:|---:|---:|---:|
| Workspace all features | 358 | 0 | 45 | 573.296s |
| Workspace no default features | 177 | 0 | 14 | 548.927s |
| Production UI-only tests | 22 | 0 | 0 | 0.214s |
| UI-tests controller suite | 120 | 0 | 1 | 8.526s |

Counts overlap; they are not a combined unique-test total or directly comparable
to earlier package-only counts. Production UI check, inspection no-run build,
clippy, fmt and whitespace each pass separately. Clippy is **not warning-free**:
core range-pattern, task parity, main type-complexity/collapsible-if, new synthetic
fixture chunk-size suggestions (two compiled locations), vendored GPUI float
fallback and proc-macro-error2 future-incompatibility notices are retained. No new
`-D warnings` policy is claimed. No final error/failed-test lines were found.

Earlier root `regressee-phase7-checks-ygx_j4b7` is preserved/frozen: all-features
exited 101 after explicitly terminating its exact slow qualification child
(SIGTERM), while the other eight commands exited 0. It is **not** the final pass;
its source/test-binding snapshot predates the final named changes. Other aborted
logs remain evidence of attempts, not accepted final runs.

Separate targeted checks cover the real filtered approval controller and the
100/1,000-store safety regression. The production UI-only `--no-run` probe that
failed in Phase 6 now compiles; the intended UI-only feature actually executes its
non-GPUI tests without test support. `ui-tests` executes the GPUI controller suite.
All-features includes renderer support, but ignored capture tests execute only
through the two exact opt-in commands below. Ignored native/browser/runtime tests
are not upgraded to passes.

Retained-store rerun: **1 passed, 0 failed, 0 ignored**, 553.48s total;
100 pages took **10.163s**, 1,000 took **543.317s** including fixture construction,
analysis, approvals, corruption/stale rejection, export/import and reopen checks.
These are whole synthetic safety-test elapsed times, not capture throughput.
`phase7-storage-confirmed.log` and `storage-{100,1000}/qualification.json` under
the approved evidence parent/root retain actual outcomes. Both original stores,
recipient stores and ZIPs are preserved; the receipt hashes every retained file.

Maintained Node worker protocol/adapter boundary tests: **9 passed, 0 failed**,
without launching a browser. Initial direct source invocation failed because
Playwright is not installed beside `worker/product`. A private copied source
tree linked the retained local development dependency; 11 maintained source
hashes agree. Node is 22.20.0 and that dependency is older Playwright 1.56.1, versus
production-requested 1.64.0. `worker-boundary.json`/`.log` freeze this distinction;
it is not production browser/runtime tuple or installation qualification.

## Complete sequence — coverage and exact evidence boundary

| Workflow / safety gate | Current source/controller and offscreen qualification |
|---|---|
| Public/private setup → pages → discovery → capture/save | Phase 5 public/private saved/dirty/invalid/open/ready and identity-bound save/discovery worker tests rerun; every advanced setup capability traversed at both sizes, including actual timezone/custom replacement. Phase 5 six repair originals/38 historical confirmations remain source-bound historical evidence, not native login proof |
| Mixed changes, unchanged, gaps and real read failure | New 100/1,000 persisted distinct results and NeedsLogin gap; Phase 6 missing/corrupt original reads through Jobs, with retained validated identity, conservative update rejection and error-dismiss recovery. Phase 6 34 confirmation pairs reuse unchanged failure/History/task presentation owners |
| Compare/Difference/Overlay/Heatmap/full-width Before/After | Current all-mode controller/geometry/transform regressions; Phase 4 22 repair and 38 confirmation pairs retain untinted evidence, hidden-queue identity/environment, manual zoom/pan and narrow Compare. No blind rerender of unchanged modes |
| Review dispositions, explicit notes, masks, filtered target/selection | Current save/dirty/stale/pointer-coordinate/keyboard-mask/revision tests; image-mode/filter/collapse preserve target/drafts. Ignored rectangles affect analysis, **NOT redaction**; exact originals remain immutable/exported. Simulated pointer/IME callbacks do not qualify OS input |
| Consequential consent | New one/two/long 100-reference offscreen captures at both sizes, initial consequences, cached reopening, selection outside filters, actual tail owner, real worker commit and 1/20 storage approvals at both dataset sizes |
| Retained History last row/version | Phase 6 eight total baseline/source versions (seven added), actual last selected row contained/full width at both sizes; current History original-policy read-only/stale/filter/paging tests. New storage test retains three independent version maps. Loaded-page filtering remains explicitly not archive search |
| Share/import/delete/storage/cleanup | Current privacy/hostile archive/atomic import/shared-source/ref-safe GC/zero-impact and recovery tests; new distinct composite ZIP exact bytes/no imported session references. Phase 6 share/delete/import/storage pixels preserved. No new expanded technical ZIP-details pixel claim |
| Empty/loading/failed reads and task recovery | Current initial-loading/failed/dismissed/empty navigation, truthful states and real task destination/controller tests; Phase 4/6 preserved pixels. Resume is existing implemented recovery, not fabricated targeted retry/installer repair |
| Off-UI work and bounded controls | Jobs owns database/filesystem/process/image reads and mutation; viewers coalesce identity-bound reads; list virtualisation, 64-record History paging, 32 consent rows, bounded diagnostics/worker channels remain. The new source snapshot is worker-local and bounded to one extra run |

Reused captures remain bound to **their capture-time sources**; historical receipts
and FIX/blocked reports are not rewritten as current-source renders. Relevant
owners are unchanged except consent opening and shared long-checkbox wrapping,
which have fresh confirmation, and unpainted bounded worker/test-gating repairs,
which have current-source regression results. Full old receipt source equality is
not asserted after this phase. Archive/pixel preservation is independently checked.

## Supplemental pixels — one inspection, one correction, one confirmation

Root: `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase7-qualification-gzmro59k`.
Confirmation is its `confirmation/` directory. Four website fixture PNGs and
personally reviewed offscreen calibration are exact local copies, no browser or
download. Native offscreen calls use approved 0700 isolated roots:

```bash
REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode
REGRESSEE_RENDERER_CAPTURE_DIR=<root or root/confirmation>
REGRESSEE_RENDERER_PHASE7=1
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::ui_review_matrix::renderer_capture_whole_app \
  -- --ignored --exact --test-threads=1
# Omit REGRESSEE_RENDERER_PHASE7 for the separate exact bulk test:
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  phase7_qualification_tests::phase7_renderer_capture_distinct_bulk_consent \
  -- --ignored --exact --test-threads=1
python3 scripts/phase7-evidence-receipt.py <directory> --index
```

Each invocation passed **1 test**, no COMPLETE_BATCH skip/old pair reuse. Each
batch has **28 pairs** (11 tail/field states and three bulk states per size), **six
contact sheets**. Indexing merges the bulk scene records with the tail matrix;
gallery, matrix, pair names/metadata/dimensions/hashes agree. Initial 22-tail
matrix and all initial 28 originals/metadata/sheets remain preserved separately.

Inspection: all six initial sheets together. One corrective batch, then all six
confirmation sheets and these **eight confirmation originals** together:

```text
all-{1280x800,820x600}-review-diagnostics-tail.png
all-{1280x800,820x600}-review-analysis-switches-tail.png
all-820x600-bulk-one-long-consent.png
all-820x600-bulk-filtered-two-consent.png
all-{1280x800,820x600}-bulk-hundred-reference-tail.png
```

No third visual round. Full original read scope is eight, not all 28. The nine
fields have per-state actual-inner-input plus whole-label/input/help assertions.
Diagnostics is **2744px** content, not a magically fully visible element: its
actual owner is **358×496px wide / 358×278px narrow**, and the last exact pair
descriptor/mask end is visibly reached. Ordinary clipping of earlier rows/content
at that scroll edge is not row corruption. Long checkbox text wraps without
horizontal clipping; the return action remains attached. Disclosures are temporary
edit/read states with decisions disabled, not permission to judge obscured images.

Last real consent row 99 is **726×133px at (277,433)** within the current wide
**726×365px owner at (277,201)**; narrow **706×133px at (57,333)** within its
**706×365px owner at (57,101)**. Its time/profile/long path, separator and selected
paint are contained, not a window-level inference. Partial previous row at the top
is ordinary owner clipping. Initial one/two consent now visibly names future
checks, unchanged screenshots and earlier versions before the reference rows.
Long accented/CJK project identity and long URLs render; app translation/RTL and
OS text scaling are **not** qualified by those synthetic strings.

## Finding ledger — implementation versus unmet qualification

| IDs | Disposition |
|---|---|
| E1/E3 | Evidence corrections verified in Phase 1; production capability/mode regressions rerun. No native activation inferred |
| E2 | Original beginning/first-parameter gap supplemented with actual diagnostics end, all nine complete parameters and wrapped switch at both sizes; intermediate raw metadata not all individually screenshot-read |
| R1/R2/R4/R5 | Built/independently verified Phase 4 evidence-focus/local-owner/path/decision/editor repairs; current regressions. Small source changes still require manual zoom/pan. No PM readability/time-savings claim |
| R3 | Built/independently verified Phase 4; Phase 7 fixes retained-scroll opening and qualifies distinct 100-reference long consent/real selected update, plus 1/20 updates at 100/1,000 storage scale |
| S1/S2/S3 | Built/independently verified Phase 5, current worker/capability tests strengthened. Private setup images/simulated ready state are not actual authentication or installation |
| H1/H2/H3 | Built/independently verified Phase 6, current privacy/retention/filter/read-only/delete/zero cleanup tests and distinct composite ZIP byte identity. Technical expanded share details remain source/controller, not expanded-pixel acceptance |
| F1/F2 | Built/independently verified actual failure routing and correct recovery intention/destination; current real disk and task tests. Initial unread metadata is never invented |
| M1 | Built capture cap/empty/inspection/plural/Fit meaning, current regressions. No automatic responsive mode substitution or evidence tint introduced |
| Q1 | **Partially qualified, not closed acceptance.** Real distinct persisted scale/approval, retained/source protection, long consent, last History, real failures and tails now have checkable evidence. Native Linux input, AT bridge, Windows/macOS, OS scaling/high contrast/motion, actual PM participants and distributable runtime/release remain BLOCKED/NOT TESTED as separately named |

## Preserved failures, boundaries and next owner

Early fixture errors exposed real safeguards: a baseline without a queued capture
must start at the saved source; an old check cannot approve a changed active
baseline merely by reanalysis; portable metadata requires the protocol capture
mode. Tests corrected those assumptions rather than weakening guards. Early
controller cancellation asserted `approval=None`; the real design keeps a cached
preview but closes the modal, so the test now checks dismissal/no mutation.
Dynamic debug-selector lifetime compilation was corrected. Initial slow-scale
tests were not counted as passes; final bounded snapshots retain fresh-consent
verification. The initial Xlib default-error preflight was hardened to preserve
failure diagnostics/child cleanup; no surviving owned probe is left running.

Native attempts: default Vulkan `ERROR_INITIALIZATION_FAILED`; forced Lavapipe
X11 unmapped 1280×800 window; Wayland alive after eight seconds but inaccessible
for automated native tasks. Actual AT-SPI registry has one desktop and the owned
Wayland PID is **not registered**. That is a real accessibility integration
blocker; absent Orca does not absolve it. Windows/NVDA and macOS/VoiceOver are
unavailable, **NOT TESTED**. No host scale/theme changed, no native clipboard/IME
pass, **zero PM participants**. See the platform matrix for the remaining release
and browser-distribution blockers. Historical A's 24/40 is not rescored.

Final current-source production `ui` build also passes (locked/offline);
`phase7-final-native-build.log` retains two vendored GPUI float-fallback warnings
and the proc-macro-error2 future-incompatibility notice. A final owned Wayland
probe at `regressee-phase7-native-bkqjr8xr` again survives eight seconds with one
AT-SPI desktop and **owned PID unregistered**; no OS input is delivered. Its
binary hash and clean owned termination are preserved, not an acceptance pass.

**Next:** independent verifier reviews current source, exact logs/receipt, retained
stores and existing source-bound pixels without a new aesthetic render loop.
Parent handles any verified commit/push and later CI/release work. Do not mark the
whole-app native/release gate passed just because source/offscreen tests pass.
