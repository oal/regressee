# Phase 4 implementation — consent and daily inspection

**2026-10-12 · base HEAD `4e48813` · finite V4 repair implemented; separate re-verification pending.**
No commit, push, Phase 5, release action or nested agent. Independent verification
returned [FIX](phase4-verification.md); that document is preserved unchanged.
This report does **not** claim the Phase 4 exit gate is met.

## Finite functional remediation of V4-1 / V4-2 / V4-3

The following supersedes the current-status claims below, **not the historical
blocked evidence or verdict**. No Phase 5, commit/push, delegation, browser visit,
download, capture-pixel or persistent-schema/approval-semantic change.

- **V4-1 repaired in exercised scope:** finite inspector content width and
  stretching intermediate flex owners resolve the native percent-width input.
  Uniquely named `review-note-input-border` measures **326 × 40px**; actual
  `TextInput::rendered_bounds()` measures **300 × 20px**, contained within its
  scroll viewport at both sizes. Long retained draft, Home/End caret-range bounds,
  editor focus, Save/Return access, disabled decisions and Escape restoration pass.
  Both new note originals visibly show readable horizontally scrolled draft text.
- **V4-2 repaired in exercised scope:** Guided(53) belongs to both app initial-read
  allowlists and saved-project Results core busy candidates. Real Shell tests
  cover ready, tracked pending read, failed reply, dismissed error with read-failed
  retained, and authoritative empty at both sizes, including explicit wide collapse.
  Mounted/enabled/Tab/action dispatch and separately simulated native click paths
  reopen the actual rail and restore local Show. Resize preference, target/draft,
  request count and stored project/evidence remain unchanged. Dialog/consent/native
  prompt/task background activation remains rejected; mutation locks are unchanged.
- **V4-3 repaired in exercised scope:** hidden-queue evidence header retains selected
  non-root path/query/fragment, source capture profile and Before/After historical
  environments in Compare, Difference, Overlay, Heatmap, both full-width sides,
  and capture-only inspection. Narrow Compare also labels both pane environments.
  Named identity bounds are contained at both sizes; source-state assertions reject
  unrelated current project settings. Manual zoom/pan, selected target, baseline
  selection and note draft survive mode/size changes.

Four named `repair_v4_*` tests in `phase4_review_tests.rs` pass. Final locked offline
all-features packages: **299 passed, 0 failed, 40 ignored** (binary 110, library 95,
analysis 7, product flow 1, core 51, storage 35). Final log:
[phase4-repair-tests.txt](phase4-repair-tests.txt). Known vendor warnings remain.

Original independent probe rerun against rebuilt core and actual app allowlist
expressions: **4 passed, 0 failed, exit 0**, resolving its three failures. Its
runner is copied with only its output directory relocated, preserving original
probe/source/binary/log files. The inherited printed parenthetical "failure
expected for current defects" is stale commentary, not the actual exit result.
New runner/log reside under `repair-v4-complete/`; durable log:
[phase4-repair-probe.txt](phase4-repair-probe.txt).

### Narrow source-bound evidence and limits

Root: `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase4-20261012/repair-v4-complete`.
Renderer: **1 passed, 22 pairs, 42.91s**, both 1280×800 and 820×600; durable log:
[phase4-repair-render.txt](phase4-repair-render.txt). Set
`REGRESSEE_RENDERER_PHASE4_REPAIR=1` instead of `REGRESSEE_RENDERER_PHASE4=1` in
the earlier command and use the new root. Calibration proof and four website
fixtures are exact local copies from confirmation; no visits or downloads.

One focused inspection: all four new contact sheets (all 22 states) plus eight
full-resolution originals: note and hidden-queue Compare at both sizes, narrow
full-width Before, narrow mobile capture, wide loading and narrow failed-read.
The note is readable; identities/environments and Show navigation are legible.
Narrow image viewports remain short (pan/zoom required), especially Overlay;
no whole-page readability, aesthetic acceptance or task-success claim. No
post-inspection UI correction or additional polishing/confirmation loop.

Two setup attempts failed before captures (missing copied calibration, then
website fixtures). A partial attempt failed on a harness-only ordering assertion:
wide collapse preference had not yet been applied before toggling. Apply geometry
before exercising the toggle; partial pairs remain under `repair-v4/`. Complete
run used a fresh root, no overwrites/skips. Later test-only strengthening caught
a mistaken 1440×900 fixture-profile assumption and private-helper compile use;
tests now assert actual saved capture profile. All failed logs remain hash-listed.
Only `phase4_review_tests.rs` changed after the successful capture; shipping source
and capture harness are unchanged and separately bound by the receipt.

The receipt adds a repair section; [phase4-receipt.blocked.json](phase4-receipt.blocked.json)
is the exact earlier receipt. All old capture hashes, source hashes, blocked gate,
log mirrors, immutable archives and independent FIX remain historical and intact.
Checker validates old evidence integrity separately from current repair bindings.
Final `cargo fmt --all -- --check`, `git diff --check`, and
`python3 scripts/verify-phase4-receipt.py` all exit 0. Integrity checker confirms
12 immutable archives, 182 original-root PNGs, 38 blocked pairs and 22 repair pairs.
Phase 4 acceptance remains **PENDING independent re-verification**. Real OS input,
clipboard/IME, AT, scaling/localization, Windows/macOS, bulk throughput and usability
remain untested. Simulated GPUI events are not OS qualification.

## Historical implementation outcome and blockers (before finite repair)

R3 consent, R2 path-first identities, R4 local panel ownership, R5 action grouping,
the R1 evidence-focus/full-width alternative and M1 capture overview/empty handling
are built and exercised. **The confirmation note editor still paints its actual
input at approximately padding-only width at both sizes.** Draft text is retained
and the editor/Save/Return controls exist, but the note is not visibly readable.
This is a blocking R1/R5 presentation defect, not a cosmetic limitation or a pass.

The single correction batch tried a full-width field wrapper; confirmation pixels
show it did not resolve the inner native input's width. The bounded visual cycle
has ended. Do not advance to Phase 5 or commit this as verified Phase 4. The next
owner must resolve that named defect under an explicit follow-up verification
scope, measuring the **actual TextInput rendered bounds**, not just `field-Note`.
Likely owner: `viewer_ux.rs::viewer_inspector`'s note/body flex sizing. No third
render/polish round was started here.

**Additional source-level blocker found during handoff:** `workspace_control_mounted`
does not include Guided(53) in its initial evidence loading/failed-read allowlists,
and core `FocusState::controls` returns from its busy branch before adding that
new control. Narrow Results can therefore omit the rail while its local Show
navigation action is disabled. This is a restore/navigation regression requiring
the same focused follow-up, with loading and failed-read mounted/enabled tests.
No pixel/input qualification of those affected states is claimed. Existing Ctrl
shortcuts are not a substitute for the disabled local restoration affordance.

## Built changes

### 4A — consequential consent, first

- `workflow.rs::approval_decision`: correct singular/plural title, future-check
  consequence, unchanged screenshots and retained versions. Path/profile/source
  environment/UTC time precede optional exact provenance. Replace/Keep and the
  inspected screenshot number are explicit; inspected paint spans the whole row.
- One bounded scroll owner, pinned Keep reviewing/Confirm baseline and overflow
  guidance. Long rows grow naturally; separators do not cross their text. No
  permanently open second 100px provenance scroller remains.
- All references remain accessible. At most 32 rows mount per visible page;
  Previous/Next inspects every reference and advances pages. Exact requested/final
  URLs, page/profile IDs, source run and original artifact remain retrievable.
- `storage/review.rs` resolves source environment names while it already reads
  immutable source runs on the worker. This adds read-only **ephemeral preview
  metadata**, with a serde default; no database/capture/ZIP schema migration,
  evidence identity, approval policy or commit semantic changes were made.
- Existing eligibility, stale-preview rejection, background mutation guard,
  cancellation/focus restoration, explicit acceptance and retained versions remain.

### 4B — daily inspection

- `main.rs`/`workflow.rs`: locally owned Hide/Show navigation and Hide/Show results.
  Narrow review starts with navigation collapsed, Compare and the queue still open.
  Explicit navigation preference survives resizing; selected project stays visible.
  Geometry follows the actual mounted rail. Ctrl/Cmd+F restores a hidden queue.
- `controls.rs::result_row`: 84px path/query/fragment-first rows, separately budgeted
  profile/status lines, complete URL hover hint. Selected Details provides the full
  URL through keyboard-accessible disclosure. Discovery/History retain 64px rows.
- One View chooser at both sizes, separate from Zoom/Details/Analysis. It exposes
  Compare, Difference, Overlay, Heatmap and **Before/After (full width)**. The latter
  selects one untinted original with its side named; both sides share target,
  zoom policy/manual percentage and source pan. Default Compare is unchanged.
  Mode-menu arrows/Home/End, Enter activation, Close and Escape use owned focus.
- Review dispositions are a selected group, distinct from Review note and primary
  Next unreviewed; Select for baseline update and Review baseline update remain a
  separate footer. Notes move to the temporary owned editor, not an image-height
  row. Details/notes/filters remove or disable decisions while evidence is occluded;
  closing preserves notes. **The inner note width remains the blocker above.**
- `zoom_select.rs`: compact Fit/Manual meaning and percentage; manipulation help
  visible at both widths. Capture-only automatic overview is capped at 100%, with
  manual 150% and resize retention tested. Authoritative empty captures reserve no
  blank queue and expose no meaningless collapse control.
- `core/shell.rs`, app mounted/enabled rules and unique handle allocation changed
  together. Hidden navigation/queue controls leave traversal, collapse restores
  local focus, modal consent traps focus, and review decisions leave temporary
  editors. Existing advanced analysis, masks, zoom/pan, drafts and shortcuts stay.

## Tests and evidence

New `phase4_review_tests.rs` covers URL identity, default narrow policy, explicit
collapse preference/resize, local search restoration, every mode/full-width side,
manual source anchor and target/draft/selection preservation, filtered-out target,
editor decision occlusion/Escape, initial capture cap/manual enlargement, empty
ownership, one/long consent containment, optional exact disclosure, 97-reference
presentation pagination, cancellation and unchanged local stored evidence.
The 97-reference expansion is **in-memory presentation only**, never stored or
called real bulk approval. The native fixture has **12 stored references** across
six distinct synthetic addresses and two profiles; many consent selects those 12.
Existing tests retain storage approval/stale/eligibility/versions, mask/note saves,
history read-only routing, sharing/session exclusion and draft protections.

Final checks:

| Check | Actual result |
|---|---|
| App binary all-features tests | 106 passed, 0 failed, 15 ignored; final standalone run 9.26s |
| Core + storage + app packages, all features | **295 passed, 0 failed, 40 ignored**, including app library 95, binary 106, analysis integration 7, product-flow 1, core 51, storage 35 |
| Final native renderer | 1 passed, 0 failed, 120 filtered; **38 pairs**, 73.55s |
| Receipt checker | 12 immutable archive files, all 182 original-root PNGs unchanged since Phase 1, 38 pair hashes/metadata/manifests, 62 source bindings |
| Format and whitespace | `cargo fmt --all -- --check` and `git diff --check`: exit 0 |
| Visual gate | **BLOCKED: note input width**, despite renderer assertions passing |
| Additional source gate | **BLOCKED: navigation restore during initial loading/failed reads** |
| Native input / AT / Windows / macOS / usability / release | **Not tested** |

Exact durable logs: [all package tests](phase4-tests.txt), [confirmation renderer](phase4-render.txt).
Machine receipt: [phase4-receipt.json](phase4-receipt.json).
Recheck: `python3 scripts/verify-phase4-receipt.py` (integrity only, not acceptance).

Commands (working directory `/var/home/olav/Projects/Regressee`):

```bash
LIBRARY_PATH=.runtime/link-libs TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
  cargo test --locked --offline -p regressee-app --all-features --bin regressee

LIBRARY_PATH=.runtime/link-libs TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
  cargo test --locked --offline -p regressee-core -p regressee-storage \
    -p regressee-app --all-features

LIBRARY_PATH=.runtime/link-libs TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode \
REGRESSEE_RENDERER_CAPTURE_DIR=/var/home/olav/Data/opencode/tmp/opencode/regressee-phase4-20261012/confirmation \
REGRESSEE_RENDERER_PHASE4=1 \
  cargo test --locked --offline -p regressee-app --all-features --bin regressee \
    ui_renderer_capture::ui_review_matrix::renderer_capture_whole_app \
    -- --ignored --exact --test-threads=1

python3 scripts/ui-review-gallery.py \
  /var/home/olav/Data/opencode/tmp/opencode/regressee-phase4-20261012/confirmation
cargo fmt --all -- --check
git diff --check
python3 scripts/verify-phase4-receipt.py
```

## Bounded capture/inspection ledger

Isolated evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase4-20261012`.
First batch: 34 pairs/17 states, all six contact sheets read together. One corrective
batch followed. Confirmation lives in its new `confirmation/` directory: 38 pairs/
19 states, all eight contact sheets plus **20 named full-resolution originals**
read together; the receipt enumerates those exact 20. Not all 38 originals were
read individually. First pairs and copied `superseded-batch1/` remain intact.
Original 182-frame root, archived reports and initial/corrected evidence untouched.

States at **both 1280×800 and 820×600**: ordinary review, hidden queue, Difference,
Overlay, Heatmap, Details, note, no-matches/filter, one replacement consent,
full-width Before, full-width After, mode chooser, explicitly open navigation,
12 replacements, last unchanged inspected exact provenance, last unchanged row,
exact provenance tail, mobile-web capture and authoritative empty capture.

Confirmation individually establishes the named full-width sides, readable last
long Keep row and complete exact tail (including final IDs), capped mobile capture
and empty owner. Ordinary narrow originals measure **274×194px each** versus the
original 178×216px; full-width narrow is **772×142px**, at 54% fit for a 1440px
source. This is more readable width, not a whole screenshot or a task-success claim;
manual zoom/pan is still required to inspect many changes. Explicitly reopening
the narrow rail deliberately restores the old space tradeoff.

## Failed attempts and source binding

All local logs are under the evidence root and hash-listed in the receipt:
`phase4-build.log`/`phase4-tests-first.log` also remain in the parent temp directory.
Early compiler corrections: RunRecord target path, private fixture eligibility,
and qualification of `gpui::KeyDownEvent`. Test failures exposed old inline-mode/
420px consent/64px row/fixed-rail assumptions; tests now use mounted chooser/editor
paths, preserve History geometry and verify new consent flow. A new bounds test
also learned that GPUI caches old debug selectors; absence of an old selector is
not a mounted-state oracle. Final package tests caught the old compact Zoom **60px
test assertion**; it was updated to **112px** after capture. This is the sole
post-capture Rust edit and changes a unit assertion only, not shipping rendering.
The receipt/checker retains both hashes and verifies that exact substitution;
all capture-time shipping source bodies remain identical.

Initial render attempts failed on compilation and the isolated root's required
0700 permissions. First actual native batch passed. Confirmation attempted an
existing filename and correctly failed with `File exists` before replacing any
pair; the same confirmation batch then used its own new root and passed. Neither
failed setup attempt was treated as a visual inspection round or a new capture.
No COMPLETE_BATCH skip was used. No new visit, download or browser automation;
existing website fixtures and renderer calibration were reused.

Known vendored GPUI fallback and `proc-macro-error2` future-compatibility warnings
remain. No trusted browser distribution/install/login qualification was added.
Native pointer/keyboard/IME/AT, scale/localization, genuine 100–1,000-page bulk
approval, advanced/diagnostic tails and task-based PM usability remain Phase 7
obligations. DESIGN describes built behavior and names the blocker, not acceptance.
