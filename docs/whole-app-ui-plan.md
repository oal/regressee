# Whole-app UI review and improvement plan

Created: **2026-10-12**. Platform: **native Rust/GPUI desktop**, Windows,
macOS and Linux. Mode: **Operate**. This is the continuation document for this
task; it does not replace the existing implementation `PLAN.md`.

## Current checkpoint — read this first

- **Phase 0 complete:** both independent assessments have been read in full and
  copied byte-for-byte into the repository, with an integrity receipt.
- **Phase 1 verified PASS (bounded evidence gate).** Test-only
  E1–E3 corrections and 10 corrected native pairs are recorded in
  `research/whole-app-ui-review-20261012/phase1-report.md` and its correction receipt.
- Independent results: `research/whole-app-ui-review-20261012/phase1-verification.md`.
  Diagnostics tails and remaining advanced controls are not fully qualified.
- **Phase 1 committed/pushed:** `ac4de65`; no production UI changed.
- **Phase 2 independently verified PASS (documentation/evidence gate):**
  `research/whole-app-ui-review-20261012/synthesis.md` and the dated Impeccable
  critique archive contain the complete evidence-qualified review, original A's
  24/40, every finding, all screens/tasks and corrected coverage limitations.
  [Git-eligible byte-identical archive mirror](research/whole-app-ui-review-20261012/critique-20261012.md)
  preserves the ignored skill archive without force-adding local metadata.
  [Independent verdict](research/whole-app-ui-review-20261012/phase2-verification.md).
- **Phase 2 committed/pushed:** `d4b3ddb`; no production UI changed.
- **Phase 3 independently verified PASS (documentation/inventory gate):**
  [workflow blueprint](research/whole-app-ui-review-20261012/workflow-blueprint.md)
  records the recommended evidence-focus direction under delegated user judgment,
  capability/safety inventory, concrete owners and Phases 4–7 build/validation order.
  [Implementation report](research/whole-app-ui-review-20261012/phase3-report.md).
  [Independent verdict](research/whole-app-ui-review-20261012/phase3-verification.md).
- **Next executable step: authorized Phase 4A consequential consent**;
  parent commits/pushes verified documents before Phase 4A. No Phase 4 started.
- **Phases 1–7 authorized on 2026-10-12:** the user explicitly requested the
  complete plan be implemented and validated without clarification questions.
  Use the recommended interaction direction and preserve Clear wayfinding.
  Each phase has one GPT 6.1 Sol implementation agent followed by a separate
  verification agent; commit and push verified work before the next phase.
- No production UI changes, new renders or product tests were performed while
  creating this plan. Archive checks are not app qualification.
- Earlier work was preserved and pushed with explicit user authorization in
  `c2685a6`. Runtime files, local configuration, databases, logs and generated
  pixel evidence remain excluded by `.gitignore`. Never discard unfamiliar work.

### Original job

Render every app screen and meaningful state using realistic mock data, then
give a thorough Impeccable review of the **complete workflow**, not isolated
components. The user specifically questioned why a top-right button controls
the left-hand results list. They then requested a concrete, multiphased,
self-contained plan so assessment findings would survive context loss.

This document preserves the findings and separates three different activities:
**repair evidence → finish the critique → approve and implement improvements**.
Do not skip directly to moving the reported toggle.

## Durable evidence and authority

Repository paths:

- [Assessment A](research/whole-app-ui-review-20261012/assessment-a.md): independent
  design/workflow review, **24/40**, per-screen critique, personas and read ledger.
- [Assessment B](research/whole-app-ui-review-20261012/assessment-b.md): independent
  source/pixel/integrity review, measured geometry and fixture coverage warnings.
- [Archive index](research/whole-app-ui-review-20261012/README.md) and
  [integrity receipt](research/whole-app-ui-review-20261012/archive-receipt.json).
  The archive includes initial manifests, detector output and successful capture
  receipts. Keep these initial records unchanged.
- `PRODUCT.md`, `DESIGN.md`, `docs/workflow-revamp.md` and
  `.impeccable/surfaces/crates-app-src-main-rs.md`: existing product and direction
  authority. **Clear wayfinding is already user-approved.** Do not restart a
  visual-identity interview or silently replace these documents.

Large local pixel evidence, not copied into Git:

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-whole-app-review-9inkus48
```

The root contains `index.html`, `gallery-manifest.json`, `whole-app-matrix.json`,
182 PNG/JSON pairs, 32 contact sheets, renderer calibration and website fixtures.
`.impeccable/review/whole-app-current-root.txt` also records this location. The
previous gallery server was `http://127.0.0.1:40577/index.html`; availability is
not guaranteed. The local file and native PNGs do not require that server.

### Evidence limits — carry these into every verdict

1. **182 labels are not 182 distinct verified states.** They comprise 71 whole-app
   labels and 20 task labels, each at **1280×800** and **820×600**. Assessment B
   found 174 unique image hashes and seven duplicate groups. Misleading labels
   are coverage gaps until their named state is visibly established.
2. Both assessors inspected all 32 contact sheets. A read 44 selected originals;
   B read 32 selected product originals plus calibration. Do not claim all 182
   originals received individual full-resolution inspection.
3. These are actual Linux GPUI/Blade offscreen pixels with real Linux fonts,
   not HTML recreations. They establish neither OS input/presentation nor native
   accessibility, cross-platform usability or release readiness.
4. Four synthetic projects, 12 stored references and four repeatedly reused
   website images exercise presentation. Discovery has 74 mock addresses;
   `review-mixed-148` clones targets/results **in memory only**. It is not 148
   independent comparisons, storage-backed bulk approval or proof of efficient
   100–1,000-page review. Never save those clones as real evidence.
5. Task captures use older explicitly synthetic fixtures. Illustrative dates,
   zero elapsed times and deduplicated PNG counts are not runtime defects.
6. The detector actually ran and returned `[]`, exit 0. Meaningful Rust/GPUI
   rule coverage is not established; **zero findings is not a native UI pass**.
7. Browser automation failed with `browser.disconnected`. No browser overlay or
   DOM-based examination of native controls was performed. Reading gallery HTML
   would examine the gallery, not the product.
8. Capture tests passed; a complete product test suite did not run for this
   assessment. Initial failed attempts and future-compatibility warnings remain
   in the original evidence. The app's browser-setup build limitation is real;
   synthetic setup scenes do not prove installation/login/capture works.
9. All eight reviewed product UI targets matched Assessment B's source checks.
   The original binding for `ui_review_matrix.rs` differed from its current
   harness hash. Preserve this distinction rather than claiming all 52 bound
   files match. The archive receipt records current hashes at plan creation.

## Synthesis to preserve

The problem is **task hierarchy and spatial ownership, not lack of decoration**.
The current world is coherent, but the PM must reconcile page navigation, image
tools, review decisions and baseline updates across competing areas while the
evidence gets the least useful space. At the minimum window the measured
before/after panes are each **178×216 px**; a 1440 px screenshot fits at about
**12%**. Their combined viewport occupies **15.6%** of the window. Opening a note
reduces pane height to 172 px and queue-list height to 116 px. Correct rendering
does not make that evidence readable.

The baseline confirmation is the highest-consequence design issue: technical
reference/provenance language and two small scrolling areas obscure what the
user is changing. Assessment A emphasizes reading-flow inadequacy; B also
identifies a separator crossing the last URL line. Preserve both observations;
verify row paint bounds before treating every ordinary scroll boundary as a
renderer defect.

The user is right about the queue toggle: it changes a left-hand panel from a
top-right global action group. Moving it is necessary but not a sufficient fix
for the surrounding workflow.

### Strengths that must survive

- Untinted originals; explicitly named Difference, Overlay and Heatmap modes.
- Missing/failed captures never presented as unchanged or baseline-eligible.
- Explicit saving, baseline selection and confirmation; retained older versions.
- Honest progress/coverage, inspection-only history and separate loading/empty/
  failed-read states. Task workspaces are comparatively calm and useful.
- Local-only data; explicit sharing; sessions/cookies never exported; ignored
  rectangles never portrayed as pixel redaction.
- Existing searchable queues, selection paint, zoom/pan, shortcut help,
  draft protection and all supported advanced capabilities.

## Finding ledger — implementation must account for every row

Priorities are design assessments, not measured failure rates. No P0 is
established by the available evidence. Source line numbers in the original
reports are historical; locate the symbol again before editing.

| ID | Priority | Finding / evidence | Proposed phase |
|---|---|---|---|
| E1 | Coverage | `new-website` duplicates `website` at both sizes; NewProject was activated from Setup where it is not mounted. A real blank, unbound creation form is not established. B §3. | 1 |
| E2 | Coverage | `review-diagnostics` duplicates `review-details`; `review-analysis-advanced` duplicates `review-analysis`. Flags do not prove lower disclosed content was visible. B §3. | 1 |
| E3 | Coverage | Narrow Overlay/Heatmap duplicate Compare. The fixture did not reach mounted narrow-mode controls. `review-zoom-menu` activates Guided(46), which needs checking against the real menu action. B §3 plus harness inspection. | 1 |
| R1 | P1 | Minimum-window evidence becomes thumbnails; notes/disclosures further reduce or obscure it. A priority 1; B3, measured geometry §5. `all-820x600-review.png`. | 4 |
| R2 | P1 | Repeated full origins erase distinct page paths; profile/status also truncate. A priority 1; B2. Normal and mixed-148 queues. | 4 |
| R3 | P1 | Baseline consent is technical, split-scrolling and ambiguously associates row/provenance; B reports URL-line containment defect. A priority 2; B1. Both approval-confirm originals. | 4 |
| R4 | P2 | Queue collapse/restore is spatially detached from its panel, also in capture inspection. A priority 4. Normal/hidden queue originals. | 4 |
| R5 | P2 | Modes/tools, decision states, advance and note actions look like peers. Baseline selection remains a different necessary concept. A cognitive-load table and per-screen review. | 4 |
| S1 | P1 | Guided setup emphasizes Save over advancing even when clean; stages disappear; access/sizes fall below the first viewport. A priority 3. Access/public/pages originals. | 5 |
| S2 | P2 | Saved versus draft access and disabled login controls do not emphasize the immediate Save step; lower actions overwhelm. A priority 3; B6. | 5 |
| S3 | P2 | Safety/technical prose precedes the mapping/page/configuration being inspected; raster/Ephemeral vocabulary switches. A setup critique; B7. | 5 |
| H1 | P2 | History browsing has small nested scrollers and weak overflow guidance; loaded-page filtering is not archive search. Full-width selection paint is already correct. A priority 5; B4. | 6 |
| H2 | P2 | Deletion exposes `Run(RunId(...))`; sharing/storage/provenance foreground implementation terms and raw bytes. Preserve privacy consequences. A priority 5; B7. | 6 |
| H3 | P2 | Zero-impact cleanup still asks for confirmation; irreversible recovery deletion has neutral styling. A priority 5; B6. | 6 |
| F1 | P2, qualified | Rendered comparison `viewer=None` fallback omits failed-target identity/reason/recovery and offers Analysis. Fixture explicitly clears a cloned viewer; genuine storage-failure routing is not proven. A failed-target critique; B5. | 6 |
| F2 | P2 | Paused capture tells users to resume but says Review check results; generic failure loses operation identity. A task critique; B6. | 6 |
| M1 | P3 | Empty capture reserves an unexplained blank queue column; wide mobile capture initially upscales to 194%; plural grammar and narrow zoom/mode meaning need care. A minor observations; B8. | 4 / 6 |
| Q1 | Qualification | Many retained baselines, last History row, many/long approval items, real failures, long/localized labels and native input/accessibility are unverified. B §8. | 7 |

## Phased execution

### Phase 0 — preserve the assessments and checkpoint **COMPLETE**

Deliverables: this plan; both raw reports; initial manifests/detector/receipts;
current source hashes and dirty-tree snapshot. Archive receipt confirms 12 files
copied byte-for-byte. No report text was silently rewritten into consensus.

Exit gate: reports and plan are readable from the repository without relying on
conversation memory or a running subagent. PNG storage location and limitations
are explicit. This gate is met.

### Phase 1 — correct misleading evidence **VERIFIED PASS; BOUNDED COVERAGE**

**Scope:** test-only `crates/app/src/ui_review_matrix.rs` and evidence tooling if
necessary. No production interface redesign. Do not rerender all 182 images just
because some labels are wrong.

1. Check the actual mounted control paths, `Shell::create`, disclosure flags and
   the real zoom component action. Reach a clean new-project form through the
   supported Projects/create path rather than activating an unavailable control.
2. For diagnostics and advanced analysis, open the correct disclosure, scroll its
   **own** bounded body to the named controls, and assert their visible bounds.
   Scrolling the outer screen is not a substitute. For narrow Overlay/Heatmap,
   reach the actually mounted control and verify the selected viewer mode.
   Verify zoom-menu openness rather than simply setting Fit Width.
3. Preserve affected old PNG/JSON pairs and the initial matrix/gallery before
   recapture, in a clearly named superseded evidence directory. Existing complete
   files are skipped by `REGRESSEE_RENDERER_COMPLETE_BATCH=1`; never claim new
   captures when the renderer retained old ones. Do not delete unrelated captures.
4. Render only affected states at their relevant sizes. Assert intended shell
   state plus visibility/geometry; image nonidentity alone is not sufficient.
   Keep fixture project/evidence unchanged assertions. New images must remain
   marked synthetic, native offscreen and not OS-input-qualified.
5. Inspect the corrected originals in **one batch** at both desktop sizes, then
   allow at most one corrective confirmation batch. If still incomplete, document
   the gap instead of starting an open-ended render loop.
6. Regenerate gallery/contact sheets and write a correction receipt listing old
   and new hashes, source binding and corrected coverage. Keep the original A/B
   reports immutable and say they assessed the **original** batch.

Exit gate: E1–E3 are either genuinely visible with assertions and inspected
originals, or explicitly retained as unresolved coverage gaps. Product UI hashes
remain unchanged relative to the archive. No presentation clones were stored as
real site results. The gallery and manifests agree with the actual PNG/JSON pairs.

### Phase 2 — deliver the complete critique **VERIFIED PASS; DOCUMENTATION/EVIDENCE ONLY**

Produce `docs/research/whole-app-ui-review-20261012/synthesis.md` and a dated
Impeccable critique archive following the skill's critique format. Read that
reference before writing; do not run redundant independent assessments merely to
recreate preserved reports.

The deliverable must contain:

- Overall verdict, A's 24/40 heuristic table with attribution, strengths and
  workflow-level diagnosis. Do not invent a fresh higher score after fixture fixes.
- Prioritized problems with actual screenshots and source owners, including the
  user's toggle complaint in the context of the wider review geometry.
- Screen-by-screen coverage: Projects → Website/Access/Pages → Discovery → Capture
  → Compare/analysis/decisions → Baseline confirmation → History/sharing/storage
  → App settings/help → task waiting/paused/completed/failed states.
- PM first-use, frequent high-volume review and low-vision/keyboard persona risks;
  separate visual evidence from behavior not exercised.
- Explicit reconciliation of A/B, especially ordinary scroll clipping versus
  approval row containment, the no-frame failed-target qualification and correct
  full-width History selection paint.
- Corrected evidence addendum, original inspection ledgers, detector/browser/
  build/scale/accessibility limitations, and links to this plan.

Exit gate: the user has the detailed critique originally requested, not merely a
list of proposed edits. Implementation is now authorized by the 2026-10-12
instruction; proceed through the documented Phase 3 blueprint and verification.

### Phase 3 — confirm the workflow blueprint **VERIFIED PASS; DOCUMENTATION/INVENTORY ONLY**

Selected direction and build contract:
[workflow-blueprint.md](research/whole-app-ui-review-20261012/workflow-blueprint.md).
This records delegated judgment, not participant approval or already-built behavior.
Implementation/check results:
[phase3-report.md](research/whole-app-ui-review-20261012/phase3-report.md).

Keep the existing Clear wayfinding identity and every product capability. Agree
the interaction structure in a short document before coding:

- **Recommended minimum-window policy:** evidence-focus layout with locally
  collapsible project navigation and results queue; an explicitly named full-width
  Before/After alternative that preserves target and synchronized zoom/pan.
  Keep side-by-side Compare and all analysis modes available. This is a proposal,
  not a silently selected replacement for permanent side-by-side review.
- Queue owns search, filters, target navigation and collapse/restore. Viewer owns
  image modes, zoom, Details and Analysis. Review owns the three decision states,
  note and a separately emphasized Next unreviewed. Baseline update remains a
  separate, deliberate selection and confirmation, never an ambiguous Approve.
- Consent leads with screenshot identity and future-baseline consequences; exact
  technical provenance remains available on demand.
- Guided setup has a state-aware next action; existing-project settings remain
  complete and explicitly saved. Saving settings never implicitly visits a site.

Decisions to confirm: implementation scope, narrow-window policy and priority
order. Raising the 820×600 minimum is **not** an assumed fix. Do not ask again
about settled audience, local-only storage, stack or visual identity.

Exit gate: record the recommended direction/scope selected under the user's
explicit delegation of judgement, and verify the capability/safety inventory.
Read the Impeccable craft floor immediately before the first actual UI edit.

### Phase 4 — fix review and baseline consent **AUTHORIZED; PENDING**

Dependencies: Phases 1–3. Treat the whole inspect→classify→select→confirm sequence
as one interaction, while implementing in bounded batches.

**Batch A — consequential consent (R3):** readable path/profile/environment/time;
plain singular/plural title such as “Update the baseline for 1 screenshot?”;
clear selected replacements and unchanged screenshots; one readable flow;
unambiguous inspected-row role; rows that contain their text; technical IDs,
requested/final URLs, artifacts and source runs disclosed without losing detail.
Preserve Keep reviewing/Escape, explicit confirmation, stale-preview rejection,
eligibility guards and retained versions. Test one, many and long-address items.

**Batch B — daily inspection (R1/R2/R4/R5/M1):** reclaim evidence space according
to the approved blueprint; path-first rows with full URL retrieval; locally owned
collapse/restore; modes distinct from tools; decision states distinct from
navigation/notes/selection. Keep fit/manual meaning and concise manipulation help
visible. Opening a note or details must not make the next decision depend on
hidden evidence. Cap initial capture overview enlargement if approved while
keeping manual enlargement available.

Likely owners: `workflow.rs`, `main.rs`, `review_ux.rs`, `viewer_ux.rs`,
`ui/controls.rs`, `ui/zoom_select.rs`, `crates/core/src/shell.rs` and targeted tests.
Do not change capture pixels or approval semantics to simplify the display.

Exit gate: at both window sizes, users can identify a target, reach a readable
inspection view, use all modes, classify it, separately select an eligible update
and understand confirmation. Verify drafts, selection outside filters,
synchronized pan/zoom, collapse/restore and dialog containment. Native input
remains separately qualified in Phase 7.

### Phase 5 — simplify first baseline setup and discovery **AUTHORIZED; PENDING**

Address S1–S3 across the complete first-use sequence:

- Keep all three setup stages visible, with current/completed/unavailable state.
- Put public/private access and the next action in the first minimum viewport.
  Use Continue when saved; Save & continue when changed; an explicit
  Save & open login browser only where the workflow supports it. No implicit visit.
- Co-locate saved/draft/browser status and the required immediate action. Disclose
  browser maintenance, session retention and mapping where relevant, without
  deleting controls or weakening consent.
- Surface chosen screen sizes and a compact current capture plan. Retain mobile
  web-emulation descriptions, visible-screen/full-page, replacement mode,
  boundaries, robots controls, speed presets and advanced retry/parallelism.
- Show a concrete mapping pair before extended scope notes; make included-target
  estimates and limited-discovery cause/next step available near Capture pages.
  Improve raster/Ephemeral wording without changing capture behavior.

Likely owners: `main.rs`, `workflow.rs`, `options_ux.rs`, `form_recovery.rs`,
`crates/core/src/shell.rs` and targeted tests.

Exit gate: public and private first-baseline routes have an understandable next
safe action; screen-size selection is discoverable; validation points to the
affected input; dirty settings and explicit login/save boundaries are preserved.
Existing-project settings still expose every confirmed capability.

### Phase 6 — history, sharing, cleanup and recovery **AUTHORIZED; PENDING**

Address H1–H3, F1–F2 and remaining M1 across record selection and consequences:

- Meaningful History viewport and clear overflow; retain full-width selection.
  Say “Filter this history page” until real archive search exists. Keep scope,
  paging, selected identity and actions together; readable dates/types/coverage
  precede technical IDs and exact byte counts.
- Consequence-first share/import/delete summaries with expandable exact detail.
  Preserve the original-screenshot privacy warning, recipient login requirement,
  no-session export, shared-object protections and deliberate deletion.
- Disable zero-impact cleanup; use consistent danger treatment for actual
  irreversible deletion. Never collapse distinct recovery/storage safety rules.
- Verify real failed-comparison routing before generalizing the cloned no-frame
  fixture. Show available target/profile/reason/recovery locally without a fake
  image or irrelevant analysis action. Do not promise targeted retry unless it
  exists; explain rediscovery/replacement consequences accurately.
- Make paused/failed/completed actions name the actual next intention. Do not
  invent Resume, Open folder or installation repair functionality. Compact
  inspection-only context and remove empty queue reservation where appropriate.

Likely owners: `workspace_history.rs`, `main.rs`, `viewer_ux.rs`, `loading_ux.rs`,
`task_ux.rs`, `ui/controls.rs` and targeted tests.

Exit gate: last loaded/retained records are discoverable; deletion names its
record and consequences; privacy copy remains clear; failure actions match
implemented recovery; inspection never activates a baseline or rewrites evidence.

### Phase 7 — whole-workflow and native qualification **AUTHORIZED; PENDING**

Validate the complete sequence, not just edited components. Use isolated data,
normal/minimum desktop windows and a declared supported-platform matrix.

1. Batched native captures: public/private setup, mixed comparisons/gaps, notes,
   masks, all modes, one/many/long approvals, multiple retained baseline versions,
   last-row History selection, sharing/deletion, empty/loading/failed states and
   task recovery. Include long text and OS text-scaling cases where supported.
2. Production-like distinct 100–1,000-page fixtures with real storage-backed
   identities for selection/approval testing. Do not silently upgrade the
   presentation-only 148-target fixture into throughput or approval proof.
3. Real OS input: pointer, keyboard-only full paths, shortcuts, focus reveal,
   trap/restoration, Escape/cancel/discard, resize, clipboard and IME. Offscreen or
   simulated focus tests are useful but not substitutes.
4. Native accessibility and host conventions on supported Linux AT,
   NVDA/Windows and VoiceOver/macOS; status announcements, high contrast, readable
   scaling and motion preferences. Mark unavailable platforms **not tested**.
5. Safety regression: failed/missing/incompatible evidence blocked from approval;
   stale asynchronous identities rejected; settings/inspection do not mutate
   baselines; older versions/shared objects protected; export excludes sessions;
   ignored areas do not redact originals; database/image/process work remains
   off the UI thread.
6. Task-based PM usability checks: can users identify the page, inspect a small
   change, distinguish review from baseline updates, explain consent, and recover
   a gap? State actual participants/results; screenshots cannot establish time
   savings or task-success rates.

Exit gate: separate outcomes for source tests, offscreen layout, real native input,
accessibility, usability and release readiness. Update verification documents and
DESIGN.md to describe approved, actually built behavior only. Unavailable runtime
distributions or OS runners remain named blockers, not implied passes.

## Execution and recovery notes

### Authorized post-plan delivery

The user additionally requested on 2026-10-12: after all phase changes are
implemented and pushed, fix the GitHub Actions pipeline and tag a new downloadable
release for the next working day. Inspect real workflow failures, repair and
validate CI/release automation, push fixes, then create a fresh version tag and
verify the published assets. Do not present build availability as native usability
or cross-platform acceptance. Record workflow run IDs, release URL and any actual
distribution limitations in the final checkpoint.

### Known capture commands (reference; not run during planning)

The existing website fixture files are already available. If the root is lost,
generate them with `scripts/ui-review-fixtures.mjs` using the installed worker's
Playwright/browser and an explicit absolute `REGRESSEE_RENDERER_CAPTURE_DIR`.
The script blocks external page requests. Do not install/download tooling or
visit real websites as an unnoticed side effect of this review.

Known successful whole-app invocation:

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

Use the retained-complete behavior carefully: preserve then move only the
affected old pairs out of active filenames to allow their recapture. If a
metadata file is missing, do not leave its PNG in the skip path. Preserve the
initial geometry matrix before it is replaced. Use tool timeout 0 for a long
render, and wait for completion rather than starting parallel retries.

Rebuild the evidence gallery after corrections:

```bash
python3 scripts/ui-review-gallery.py \
  /var/home/olav/Data/opencode/tmp/opencode/regressee-whole-app-review-9inkus48
```

Task capture receipt is archived; do not rerender unchanged task states during
Phase 1. Product-target detector output is already archived; no native DOM overlay
or zero-finding certification should be attempted.

### Restart recipe for a fresh session

1. Read **this plan**, then the two repository assessment reports and the archive
   README/receipt. These files, not a compressed conversation, carry the task.
2. Read the latest checkpoint below and any phase-specific receipts. Check Git
   status and source hashes to distinguish new work from preserved earlier work.
3. Verify the evidence root exists. If not, retain the reports, say pixels are
   unavailable and regenerate only with an explicit isolated capture path. Never
   substitute illustrative HTML for native product evidence.
4. Execute **only the next authorized phase**. Do not treat critique recommendations
   or the proposed implementation phases as user approval.
5. After each meaningful batch, update the checkpoint: actions actually completed,
   changed paths, evidence/log paths, checks and results, blockers, decisions and
   exact next action. Save it before handing off or approaching context limits.
6. Visual verification is bounded: one batched inspection, one fix batch, at most
   one confirmation inspection. Remaining defects are documented, not hidden by
   endless polishing. Do not discard unfamiliar working-tree changes.

### Checkpoint log

**2026-10-12 — independent Phase 3 verification: PASS.** Reviewed the complete
plan/blueprint/report/synthesis against PRODUCT/DESIGN/approved workflow and current
production owners at `d4b3ddb`. Confirmed delegated evidence-focus direction,
unchanged minimum/default Compare/all modes, synchronized full-width alternative,
local queue ownership, consent/review/update separation, safe state-aware actions,
complete advanced capability/safety inventory and Phases 4–7 focus/test handoff.
Read-only checks: 40 relative links (17 blueprint), 54 enumerated owner methods plus
viewer pane helper/ZoomSelect, nine analysis fields/two switches, 12 immutable
archive hashes/lengths and eight production hashes; production/tooling/authority
unchanged, no untracked source, whitespace clean. No blocking defects. See
`research/whole-app-ui-review-20261012/phase3-verification.md` for scope/limits.
No product edit, render/app test, commit/push or Phase 4 work. **Next: parent
commits/pushes verified Phase 3 docs, then Phase 4A; craft floor before UI editing.**

**2026-10-12 — Phase 3 implementation, documentation only:** Phase 2 is pushed as
`d4b3ddb`. Read the complete plan/synthesis/original A/B and existing product/design/
workflow authority; loaded Impeccable context once plus shape/operate references.
Selected full-plan scope and recommended evidence-focus policy under explicit user
delegation: local navigation/queue collapse, explicit synchronized full-width
Before/After, retained default Compare/all modes, unchanged 820×600 minimum and
Clear wayfinding. Blueprint specifies ownership, consent-first flow, state-aware
save/login boundaries, advanced capability inventory/safety matrix, mounted/focus
rules, source owners and migration/tests/bounded render handoff for Phases 4–7.
Read-only checks located 56 owner methods, nine advanced fields/two switches and
17 blueprint links; production/tooling/authority diff against HEAD is empty.
See `research/whole-app-ui-review-20261012/phase3-report.md` for final checks/limits.
No production edit, render, product test, commit/push or Phase 4 work. **Next:
independent Phase 3 verification; parent commits/pushes verified docs before 4A.**

**2026-10-12 — independent Phase 2 verification: PASS.** Complete original A/B and
Phase 1/2 documentation reviewed; all 18 findings, 71 whole-app/20 task tokens,
original ten A scores (24/40), exact 44/32 read inventories, source/pixel ownership,
personas and reconciled evidence limits verified. Independent read-only checks pass:
12 immutable archive hashes, eight production hashes, 53 current bindings, 182
current/original PNG accounting, 10 corrected pairs and report links/body/metadata.
Created byte-identical Git-eligible `research/whole-app-ui-review-20261012/critique-20261012.md`
mirror of the ignored skill archive; no force-add of local metadata. Two original
ignored logs/pixels remain local-only; native/runtime/scale/tail limits retained.
See `research/whole-app-ui-review-20261012/phase2-verification.md` for actual checks.
No product edit, render, app test, commit/push or Phase 3 work. **Next: parent
commit/push verified Phase 2 documents including the mirror, then authorized Phase 3
blueprint implementation and independent verification.**

**2026-10-12 — Phase 2 implementation:** read full original A/B, archive index/
receipt, Phase 1 report/correction receipt/independent PASS; loaded Impeccable
context once and read critique/native guidance before writing. Produced complete
`research/whole-app-ui-review-20261012/synthesis.md` and dated Impeccable archive
for `crates/app/src/main.rs`; preserved original 24/40 without rescoring. Includes
every E/R/S/H/F/M/Q ledger row with source symbols/pixel links, all whole-app and
20 task states, three personas, reconciliations and original inspection ledgers.
Phase 1 was committed/pushed as `ac4de65` before this work. Documentation integrity,
links and unchanged archive/product hashes checked; see
`research/whole-app-ui-review-20261012/phase2-validation.md`.
No production UI edits, new screenshots, new app tests, redundant assessments,
commit/push, CI/release action or Phase 3 work. **Next: independent Phase 2 verifier;
parent commits/pushes verified work before Phase 3.** User's no-question/full-plan
authorization supersedes historical ask/stop instructions; native/runtime/scale
and advanced/diagnostics-tail limits remain, not silently accepted.

**2026-10-12 — independent Phase 1 verification: PASS.** Independently reran receipt
checker (10 pairs, eight changed PNGs, 172 unchanged) and binary tests (102 passed,
0 failed, 15 ignored; 21.13s). Reviewed test-only diff, mounted routes, state/owner
bounds and stored-project/evidence equality assertions. Supplemental checks confirm
all original PNG hashes accounted for, matrix/gallery/HTML/pair agreement and
receipt/artifact/contact-sheet hashes. Read ten corrected originals in one read-only
verifier batch; no render or polish loop. Diagnostics tail and remaining advanced
controls stay unresolved coverage, explicitly allowed by this phase's gate. No OS
input/accessibility/platform/runtime/scale acceptance inferred. Durable verdict:
`research/whole-app-ui-review-20261012/phase1-verification.md`. **Next: parent
commit/push before Phase 2. Verifier did not commit, push or start Phase 2.**

**2026-10-12 — Phase 1 implementation:** corrected only the test harness and added
receipt verification tooling. Preserved 10 original pairs, initial matrix/gallery/
binding and all contact sheets under the evidence root's
`superseded-phase1-20261012/`. Corrected creation, visible diagnostics/first advanced
parameter, narrow Overlay/Heatmap and asserted real zoom popup at both sizes.
One batched full-resolution inspection of 10 originals; no confirmation batch.
Renderer: 1 passed; binary tests: 102 passed, 15 ignored; receipt: 12 archive files
and eight product UI hashes unchanged, 172 unaffected PNGs unchanged, 182 manifest
entries/unique hashes. Diagnostics/advanced tails remain uninspected; native input,
accessibility/platform/runtime/scale limits remain. See `phase1-report.md` for exact
commands, initial assertion failure and logs. **Next: independent Phase 1 verifier;
parent commits/pushes only after verification. Phase 2 not started.**

**2026-10-12 — execution authorized:** the user requested sequential GPT 6.1 Sol
implementation and independent verification agents for every phase, no questions,
and commits/pushes between phases. Preserved all eligible existing changes in
`c2685a6` and pushed to `origin/main`. The old approval-stop instructions are
superseded by this explicit authorization. Select the recommended evidence-focus
minimum-window policy, retain side-by-side and all modes, and report unavailable
native/platform/usability qualification honestly rather than implying passes.
**Next: Phase 1 implementation agent, then independent verification.**

**2026-10-12 — plan creation:** read both complete original assessments and key
narrow review/consent originals; archived 12 evidence files unchanged; recorded
current source hashes and Git status; wrote this plan. No new renderer/product
test run, no fixture corrections, no product UI change. **Next: Phase 1 E1–E3
fixture truth correction, then Phase 2 final critique. Phases 3–7 await approval.**
