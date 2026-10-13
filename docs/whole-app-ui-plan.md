# Whole-app UI review and improvement plan

Created: **2026-10-12**. Platform: **native Rust/GPUI desktop**, Windows,
macOS and Linux. Mode: **Operate**. This is the continuation document for this
task; it does not replace the existing implementation `PLAN.md`.

## Current checkpoint — read this first

- **Final delivery PASS (2026-10-13): all seven phases implemented, feasibly
  verified, committed and pushed.** Cross-platform Foundation/Phase 0 are green;
  [unsigned experimental `v0.1.0-preview.2`](https://github.com/oal/regressee/releases/tag/v0.1.0-preview.2)
  is published from tested `1911c3b9905d727ac830a420eb6935ceb4d380e9`.
  [Independent final delivery verdict](research/whole-app-ui-review-20261012/delivery-release-verification.md)
  confirms fresh downloads, all five asset digests, exact source and inventories.
  Native input/graphics, clean installation/runtime first-use, AT, usability and
  full supported-release/license gates remain blocked or untested, not closed.
- **Phase 0 complete:** both independent assessments have been read in full and
  copied byte-for-byte into the repository, with an integrity receipt.
- **Phase 1 verified PASS (bounded evidence gate).** Test-only
  E1–E3 corrections and 10 corrected native pairs are recorded in
  `research/whole-app-ui-review-20261012/phase1-report.md` and its correction receipt.
- Independent results: `research/whole-app-ui-review-20261012/phase1-verification.md`.
  Phase 1 left diagnostics tails and remaining advanced controls unqualified;
  Phase 7 supplements this with both-size tail and all-nine-field geometry evidence.
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
  **Phase 3 committed/pushed:** `4e48813`.
- **Historical Phase 4A and 4B handoff: visual gate BLOCKED** by the inner note input's
  width at both sizes. Source/storage package tests pass (295; 40 ignored), native
  confirmation has 38 pairs, but this is not an exit-gate pass. See
  [Phase 4 report](research/whole-app-ui-review-20261012/phase4-report.md) and
  [receipt](research/whole-app-ui-review-20261012/phase4-receipt.json).
  Source handoff also identifies disabled local navigation restoration during
  initial evidence loading/failed reads; both blockers need correction.
- **Historical independent Phase 4 verification: FIX.**
  [Verdict](research/whole-app-ui-review-20261012/phase4-verification.md) confirms
  both blockers and a third contract gap: selected target path disappears with
  the queue hidden; narrow Compare omits environment context. Independent package
  rerun: 295 passed, 0 failed, 40 ignored; required navigation probes: 1 passed,
  3 failed. Existing evidence inspected without new rendering.
- **Finite V4 functional repair implemented and independently re-verified PASS:**
  actual note content/border bounds and caret reveal, real Shell navigation
  mounted/enabled/Tab/simulated-pointer/dispatch paths, and persistent non-root
  target/profile/source environments pass at both sizes. Packages: **299 passed,
  0 failed, 40 ignored**; original independent probe rerun: **4 passed, 0 failed**;
  separate serial targeted rerun: **8 passed, 0 failed, 0 ignored**. Verifier inspected
  **22 repair originals and 4 sheets** without rerendering. Integrity: 12 immutable
  archives, 182 original-root PNGs, 38 blocked + 22 repair pairs. Prior FIX preserved.
  [Re-verification verdict](research/whole-app-ui-review-20261012/phase4-reverification.md).
  All substantive Phase 4 gates clear within source/controller/offscreen scope;
  native input, usability and release qualification are not inferred.
- **Phase 4 committed/pushed:** `2f09a6b`, as confirmed by the Phase 5 parent handoff.
- **Phase 5 independently re-verified PASS (source/controller/offscreen scope):**
  independent packages **308 passed, 0 failed, 40 ignored**; serial Phase 5 tests
  **12 passed**; original external worker probes **2 passed**; strengthened external
  repair tests **5 passed**, including production-bound timezone selection.
  V5-1–V5-3 clear: guided correction routing/current-clip focus, newer login drafts
  and identity-bound save continuations, complete setup capability reachability.
  Six repair originals inspected; immutable FIX/blocked evidence and 66 source
  bindings checked. [Verdict](research/whole-app-ui-review-20261012/phase5-reverification.md).
  Native input, accessibility, usability and release acceptance are not inferred.
- **Phase 5 committed/pushed:** `a37cf8c`, confirmed by the Phase 6 parent handoff.
- **Phase 6 independently verified PASS (source/controller/offscreen gate):**
  independent packages **311 passed, 0 failed, 40 ignored**; serial Phase 6 tests
  **4 passed**, History regressions **18 passed**, storage recovery regressions
  **3 passed**. Existing eight confirmation sheets/seven originals inspected;
  four selected tails checked against actual inner owners/clips and PNG edge paint.
  Integrity: 12 archives, 182 original PNGs unchanged, 88 sources, 34 initial +
  34 confirmation pairs. [Verdict](research/whole-app-ui-review-20261012/phase6-verification.md)
  retains narrow-scroll/expanded-detail limits, the supplemental UI-only test-feature
  failure and corrected fixture accounting. Production `ui` check passes; native
  input/AT/usability/platform/release acceptance is not inferred.
  Phase 6 committed/pushed as `135d3eb`, confirmed by the Phase 7 parent handoff.
- **Phase 7 independently verified PASS (IMPLEMENTATION + FEASIBLE gate):**
  [report](research/whole-app-ui-review-20261012/phase7-report.md) and
  [platform matrix](research/whole-app-ui-review-20261012/phase7-platform-qualification.md)
  record genuinely distinct persisted 100/1,000-page safety qualification, real
  filtered Jobs approval, supplemental both-size tails/fields/consent and finite
  consent-scroll/checkbox/source-verification repairs. Source/controller/offscreen
  final matrix: **nine commands exit 0**, workspace **358/0/45** all-features and
  **177/0/14** no-default (passed/failed/ignored); UI-only **22/0/0**, UI-tests
  **120/0/1**. Counts overlap, not a unique sum. Retained-store rerun **1/0/0**.
  These results are separate from **BLOCKED native input/accessibility/release** and
  **NOT TESTED Windows/NVDA, macOS/VoiceOver, host scaling and PM usability**.
  [Final independent verdict](research/whole-app-ui-review-20261012/phase7-verification.md):
  all nine independent commands exit 0; workspace **358/0/45** and **177/0/14**,
  UI-only **22/0/0**, UI-tests **120/0/1**; targeted review **8/0/0**, note/mask/history
  **18/0/0**, setup **5/0/0**, approval/privacy **7/0/0**, Node boundary **9/0/0**.
  Counts overlap. Both independent workspace runs execute the distinct 100/1,000-page
  regression. SQLite/CAS/ZIP and all 2,198 original pixel identities independently
  checked; retained maps replace exactly 0/1/20. Six sheets/twelve existing originals
  inspected without rendering; actual inner-owner tails and current source bindings
  pass. Clippy warnings retained, not warning-free. Initial exact receipt passes;
  subsequent verifier SQLite read-only WAL inspection changes only two volatile
  recipient SHM files, explicitly recorded, not restored or silently refrozen.
  Zero participants; **no full native/accessibility/usability/release acceptance**.
  **Phase 7 committed/pushed:** `b46dff5` (parent-confirmed delivery base).
- **Post-plan CI/preview delivery complete at `1911c3b`: pipeline fixed, all
  intended platforms PASS, preview published.** Semantic-preserving strict lint
  repairs and bounded budget changes retain default distinct 100/1,000-page tests,
  all features, security checks and normal matrices. Earlier Windows Phase 0
  timeout **38324688313** remains a failure in the historical
  [report](research/whole-app-ui-review-20261012/delivery-report.md) and
  [verification](research/whole-app-ui-review-20261012/delivery-verification.md).
  Main Foundation **38329625059** / Phase 0 **38329625072** now pass all three
  hosts, including Windows runtime stages. Tagged Foundation **38333821918** /
  Phase 0 **38333821912** also pass all three; release **38333821892** passes both
  Linux/Windows builds and publication: **9/9 tagged jobs**, not just workflow
  status. Source SHA is `1911c3b9905d727ac830a420eb6935ceb4d380e9` in every run.
  [Final verification](research/whole-app-ui-review-20261012/delivery-release-verification.md)
  independently re-downloads all five assets and verifies sizes/digests/manifest,
  exact 616-file tracked source, x64 headers/build metadata, standalone EXE/ZIP
  equality and 785 Windows / 902 Linux notice hashes. Windows help's CRLF-only
  difference is explicitly retained, not a product defect or rewritten hash.
  [Published release and download links](https://github.com/oal/regressee/releases/tag/v0.1.0-preview.2)
  are available. Local test counts remain **178/0/27**, **358/0/45**, launcher
  **6**, packaging **8**, source helper **32** (overlapping coverage); no new
  Rust/render cycle or native binary launch for final delivery verification.
  **Final bookkeeping:** this documentation/evidence update is committed and
  pushed separately from the tested release source; provide the published download
  links. No implementation or delivery verification process remains pending.
  Do not retag or overwrite verified artifacts; final docs postdate the released
  source archive. Native/AT/clean-install/usability/full-license and supported
  release gates remain blocked or untested, despite green hosted CI.
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

### Phase 4 — fix review and baseline consent **VERIFIED PASS (SOURCE/OFFSCREEN SCOPE)**

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

### Phase 5 — simplify first baseline setup and discovery **VERIFIED PASS (SOURCE/CONTROLLER/OFFSCREEN SCOPE)**

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

### Phase 6 — history, sharing, cleanup and recovery **VERIFIED PASS (SOURCE/CONTROLLER/OFFSCREEN SCOPE)**

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

### Phase 7 — whole-workflow and native qualification **VERIFIED PASS: IMPLEMENTATION + FEASIBLE GATE; NATIVE/AT/RELEASE BLOCKED**

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

**2026-10-13 handoff:** Phases 1–7 implementation and their feasible independent
verification gates have passed; full native/accessibility/usability acceptance
has not. Phase 7 independently passed all nine feature-matrix commands, including
358 workspace all-feature tests and real distinct 100/1,000-page storage safety
regressions. Commit/push this phase before CI repairs. The existing Foundation
workflow fails its strict Clippy step (confirmed run `38315044213`); preserve
`-D warnings` and repair the underlying lint/build issues rather than weakening CI.
Then validate hosted checks, create a fresh preview version tag, and verify actual
Windows/Linux downloadable assets. No supported-production-release claim is made.

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

**2026-10-13 — FINAL DELIVERY: independent PASS (availability/integrity and hosted
gates, not supported-release acceptance).** All seven implementation phases are
feasibly verified, committed and pushed; tested source is
`1911c3b9905d727ac830a420eb6935ceb4d380e9`. Main Foundation **38329625059** and
Phase 0 **38329625072**, tagged Foundation **38333821918** and Phase 0
**38333821912**, plus release **38333821892** are genuinely completed/success:
**15/15 jobs**, with all three CI hosts and both preview build hosts retained.
The annotated `v0.1.0-preview.2` tag peels to that exact source. Published draft=false,
prerelease=true release has five downloadable assets. Independent fresh downloads
match GitHub sizes/digests, parent bytes and `SHA256SUMS`; source has 616 exact
files/modes, platform metadata binds the same SHA, binaries are x64 ELF/PE, EXE
equals ZIP bytes and all 785/902 notice hashes pass. No runtime/private recursive
packaging; Windows CRLF notes diagnosis preserved. Commands and all five direct
[asset URLs are in the final verdict](research/whole-app-ui-review-20261012/delivery-release-verification.md),
alongside [hosted evidence](research/whole-app-ui-review-20261012/delivery-hosted-verification.json)
and [release receipt](research/whole-app-ui-review-20261012/delivery-release-receipt.json).
No new renders, Rust reruns, binary launches or verifier commit/push/tag/release
edits; all checks complete. Native/runtime clean-install, input/graphics, AT,
participant usability, security/installer/signing and full native/transitive
license acceptance remain blocked/untested. macOS has no preview and retains its
supervision gap. **Next: parent commits/pushes final evidence/docs and sends user
download links; preserve the tested tag and archives.** Earlier checkpoints below
remain historical evidence, not current unfinished handoffs.

**2026-10-13 — independent Phase 7 verification: PASS (IMPLEMENTATION + FEASIBLE gate).**
Final [verdict](research/whole-app-ui-review-20261012/phase7-verification.md) records
the full plan/blueprint/diff/source/evidence review and nine-command independent
locked/offline matrix at `regressee-phase7-checks-82rqditj`: all exits 0; workspace
all **358/0/45**, no-default **177/0/14**, UI-only **22/0/0**, UI-tests **120/0/1**.
Both independent workspace runs execute actual distinct storage-scale tests;
counts overlap. Separate serial suites: modes **8**, notes/masks/history **18**,
setup **5**, nonvacuous approval/privacy **7**, all zero failures/ignored; Node
boundary **9/0**, not production-browser qualification. Actual stores have unique
PageIds/URLs/original pixels (2,198 independently decoded), real 50/49/1 and
500/499/1 outcomes, 0/1/20 replacement maps, exact inherited originals, three
retained versions and byte-exact ZIPs. Source/feature gating, startup timezone
selection and 1300×700 Custom replacement pass. Six existing sheets/twelve originals
and actual inner clips checked without a renderer/polish loop. Clippy warnings
retained. Exact frozen receipt initially passes; stable content still matches,
except two recipient SQLite SHM headers changed by verifier read-only WAL reads;
preserved/disclosed, not refrozen. Fresh preview verifies disk before the final
SQL transaction, which then guards current identity; no atomic filesystem snapshot
claim. Independent native probes reproduce default Vulkan failure, unmapped X11
and live-but-unregistered Wayland AT-SPI PID; production build passes, no OS input
sent. Linux native/AT and release remain **BLOCKED**; Windows/NVDA, macOS/VoiceOver,
host scaling/appearance/motion and zero-participant usability remain **NOT TESTED**.
No product edits, renders, commit/push/tag/release; no new feasible functional
remediation required. **Next: parent verified commit/push, then authorized GitHub
pipeline repair/release; publish actual distribution limits, never native acceptance.**

**2026-10-13 — Phase 7 implementation handoff; independent verification pending.**
Based on verified/pushed `135d3eb`; sole agent, no nested agents or delivery/CI work.
Distinct persisted 100/1,000-page PNG/identity/result fixtures qualify real selective
1/20 updates, fresh corruption/stale rejection, retained/shared-source protection
and byte-exact ZIP/session exclusion checks. Both-size real Jobs selection survives
filtering and supports preview/cancel/explicit commit. One batched visual inspection,
one finite consent-scroll/checkbox correction and one confirmation: 28+28 pairs,
six sheets per batch; eight confirmation originals inspected. Actual diagnostics
tail, nine fields and reference 99 use actual inner bounds, not window wrappers.
Finite worker-local source snapshot repair avoids repeated whole-run verification;
commit remains freshly verified. UI-only test gating and production-bound timezone/
custom replacement tests repaired. Final locked/offline nine-command matrix passes:
workspace all **358/0/45**, no-default **177/0/14**, UI-only **22/0/0**, UI-tests
**120/0/1**; production check/inspection build/clippy (warnings retained)/fmt/
whitespace exit 0. Counts overlap. Retained-store regression **1/0/0**, 553.48s;
Node boundary **9/0**, not production browser qualification. Final source build and
owned Wayland/AT-SPI probe preserve the unregistered-PID blocker; X11 attempts
remain unmapped/default Vulkan failed. Native input/AT/release **BLOCKED**;
Windows/NVDA/macOS/VoiceOver/host scaling/usability **NOT TESTED**, zero participants.
Evidence root `regressee-phase7-qualification-gzmro59k`, final logs
`regressee-phase7-checks-9_07m0rr` under the approved local parent; full paths,
commands, preserved earlier failures and receipt in the
[report](research/whole-app-ui-review-20261012/phase7-report.md).
DESIGN records built behavior only; original/superseded evidence stays immutable.
**Next: independent verifier, then parent delivery; do not infer native acceptance.**

**2026-10-13 — independent Phase 6 verification: PASS.** Read plan/blueprint/
synthesis/report/receipt, full diff/new files and actual storage/worker/control
owners. Packages **311 passed, 0 failed, 40 ignored**; serial Phase 6 **4 passed**,
History **18 passed**, storage recovery **3 passed**. Production `ui` check,
formatting/whitespace and receipt pass. Both 34-pair batches agree with metadata,
gallery/matrix/HTML; current confirmation harness bindings match. Read eight
confirmation sheets/seven originals without rendering; last actual loaded run and
retained baseline paint fills the current inner owner and lies within the current
screen clip at both sizes, independently sampled at both row edges. H1–H3/F1–F2/
remaining M1 and read-only/stale inspection gates pass in bounded scope. Supplemental
`ui`-only test compilation fails on GPUI test-support/fixture gating (including
preexisting ungated tests); production `ui` and intended all-features tests pass.
Fixture correction: eight total baseline/source runs, seven additional, not eight
additional. No product/source edits, render/polish, commit/push or Phase 7. Native
input/AT/platform/usability/scaling/release remain unqualified. See
[verdict](research/whole-app-ui-review-20261012/phase6-verification.md).
**Next: parent reviews and commits/pushes verified Phase 6.**

**2026-10-13 — Phase 6 implementation handoff (started 2026-10-12).** Based on
verified/pushed `a37cf8c`. Built 256px History owners/full-width selected tail,
loaded-page filter scope and paging on both record sections, identity/action
association, readable dates/coverage and opt-in exact bytes/IDs. Added consequence-
first ZIP/import/delete, independently mounted exact-details disclosure, disabled
zero-impact temporary/recovery cleanup and danger treatment for real deletions.
Real persisted failed targets can produce a before-only frame: outcome-bound local
identity/cause/recovery now replaces the irrelevant empty image/Analysis path.
Real missing/corrupt reads with retained validated metadata block update
presentation; initial unread metadata is not fabricated. Read-only headings replace
duplicate notices; empty queues/unloaded identity labels removed; paused task
continuations name capture recovery while preserving implemented Resume.
Packages **311 passed, 0 failed, 40 ignored**; serial targeted **4 passed**.
Fresh isolated inspection 34 pairs plus one correction/34-pair confirmation, eight
sheets each and six confirmation originals inspected. Actual selected tail paint
is contained in its current inner owner and screen clip at both sizes. Receipt
checks 12 archives, 182 original PNGs unchanged, 88 source bindings and both batches.
No post-confirmation production edit, network/download, commit/push/nested agent or
Phase 7. Native input/AT/usability/platform/release remain unqualified; narrow
History needs outer scrolling and long failure recovery uses its bounded body.
See [report](research/whole-app-ui-review-20261012/phase6-report.md) for commands,
attempt failures, safety/fixture scope and limits. **Next: independent verifier.**

**2026-10-12 — independent Phase 5 re-verification: PASS.** Read repaired source,
tests, report/receipt and actual diff; reran packages 308/0/40, serial Phase 5 tests
12/0, original exact external probes 2/0, and strengthened external repair tests
5/0. The last adds cross-stage correction-summary routing, custom replacement
dimensions and actual timezone change with production keybindings. Inspected all
six existing repair originals without rendering. Both reproduced defects are
resolved; newer fields 10–12/retention, absent-status/follow-up hydration, untouched
initial hydration, Continue/discovery stale/failed/newer/invalid/composing/stage/
screen/environment/project branches and current-clip capability traversal pass.
Original FIX/blocked evidence preserved; capture versus final test-only binding
difference retained. Pre-verification plan hash archived before this checkpoint
update; production/source bindings unchanged. See
[verdict](research/whole-app-ui-review-20261012/phase5-reverification.md) for actual
results and simulated-input/offscreen limitations. No product edit, new renderer,
polish, commit/push or Phase 6. **Next: parent commits/pushes verified Phase 5.**

**2026-10-12 — Phase 5 finite functional follow-up complete.** V5-1 preserves guided
validation and reveals border/error/help; V5-2 separates status from hydration and
preserves newer login-only drafts through actual worker save/read races; V5-3
qualifies mounted/enabled Tab/reveal/dispatch against current scroll owners at
1280×800 and 820×600. Packages 308/0/40; Phase 5 tests 12/0; original external probes 2/0;
six narrow source-bound native pairs inspected once. No broad renders or aesthetic
correction. Original FIX and blocked report/receipt remain unchanged. Current
report/receipt bind capture-time tests separately from final strengthened tests.
**Next: independent re-verification, not acceptance.** No commit/push or Phase 6.

**Historical 2026-10-12 — Phase 5 implementation, BLOCKED handoff.** Based on pushed/verified
Phase 4 `2f09a6b`; implemented pinned three-stage wayfinding, clean Continue/dirty
authoritative save continuation, separately explicit private login, Save & find
pages, finite guided form width/actual-input reveal, sizes-first compact plan,
mapping-first preview, nearby included-target/coverage/next-step estimate and plain
bitmap/session-only wording. Existing explicit Settings/advanced controls remain.
Packages: **303 passed, 0 failed, 40 ignored**. Isolated first complete 38 pairs,
one correction and one 38-pair confirmation; eight sheets/six originals inspected
per round. No post-confirmation UI edits. **Unmet:** Access validation changes to
Settings, hiding guided stages; newer-save/absent-status hydration lacks its race
probe; chooser bounds are window-level and narrow Mobile needs scrolling although
chosen dimensions are visible. No acceptance, native/AT/usability/release claim,
commit/push, delegation or Phase 6. Exact commands/logs/receipt/limits:
[Phase 5 report](research/whole-app-ui-review-20261012/phase5-report.md).
**Next: independent verification first; do not advance as verified.**

**2026-10-12 — independent Phase 4 re-verification: PASS.** V4-1/2/3 clear after
source review, actual inner-input/caret and real Shell regression execution,
all-mode historical identity checks at both sizes, and inspection of all 22 repair
originals/4 sheets. Independent packages: **299 passed, 0 failed, 40 ignored**;
serial targeted tests: **8 passed, 0 failed, 0 ignored**; external required navigation
probes: **4 passed, 0 failed, exit 0**. Integrity checker: 12 immutable archives,
182 original-root PNGs, 38 blocked + 22 repair pairs; 61 capture/62 final source
bindings validated. Formatting/whitespace checks exit 0. Initial FIX, blocked
receipt and implementation's PENDING handoff remain unchanged historical evidence.
[Dated verdict](research/whole-app-ui-review-20261012/phase4-reverification.md)
records consent/baseline/draft regression scope and OS/AT/usability/release limits.
No product edit, new render, polish, commit/push or Phase 5 work.
**Next: parent commits/pushes verified Phase 4, then Phase 5.**

**2026-10-12 — finite Phase 4 functional repair, re-verification pending.**
V4-1 finite scroll/body width gives actual note border/content 326/300px at both
sizes, with retained readable draft and caret reveal. V4-2 adds navigation restore
to both initial-read allowlists and saved Results core busy candidates; real Shell
pending/failed/dismissed/ready/empty mounted/enabled/Tab/dispatch/simulated-click
tests preserve requests/storage/drafts and task/modal boundaries. V4-3 keeps
non-root path/query/fragment, profile and historical environments visible in all
modes/full-width/capture, with named bounds/source assertions. Packages 299 passed,
40 ignored; original independent probe rerun 4 passed/0 failed. One focused new
22-pair capture, four sheets/eight originals inspected; previous blocked evidence
and FIX untouched. Short narrow viewports and OS/AT/usability limits remain.
No commit/push/Phase 5. **Next: independent re-verification, not acceptance here.**

**2026-10-12 — independent Phase 4 verification: FIX.** Read full plan/blueprint/
report/diff/tests/receipt and inspected all eight existing confirmation sheets plus
20 originals, without a renderer loop. Independently reran all core/storage/app
packages: 295 passed, 0 failed, 40 ignored. Independent source-bound navigation
probes: ready candidate passes; busy candidate and both actual initial-loading/
failed-read workspace allowlists fail (1 passed, 3 failed). Receipt/format/diff
checks pass. Confirmed unreadable approximately 26px-wide inner note field at both
sizes; additionally found missing persistent target path after queue collapse and
suppressed environment captions in narrow Compare. Finite source-owner fixes and
named regression requirements are in `research/whole-app-ui-review-20261012/phase4-verification.md`.
Consent one/many/long/tail, mode/shared-transform, ready-state ownership/focus,
draft/storage/privacy and capture-cap/empty gates pass only in their stated
source/offscreen scope; overall phase does not. No product edits, new capture,
commit/push or Phase 5. Native/AT/platform/usability/scale/release limits retained.
**Next: sole implementation agent `ses_ece6028deffex3GeZBVhFk37aQ` corrects V4-1,
V4-2 and V4-3 in one finite functional follow-up, then independent re-verification;
do not advance or commit as verified.**

**2026-10-12 — Phase 4 implementation, both 4A and 4B:** based on `4e48813`, kept
Clear wayfinding and implemented consequence-first single-flow consent, source
environment metadata, contained long rows/bounded reference paging, local rail/
queue collapse, path-first rows/full URL retrieval, all modes via owned chooser,
explicit synchronized full-width Before/After, disposition/navigation/update
separation, temporary editor decision guards, capture cap and authoritative empty
handling. Added four targeted controller/layout/safety tests and updated mounted
focus paths together. Final packages: 295 passed, 0 failed, 40 ignored; binary
106 passed, 15 ignored. Fresh isolated first 34-pair batch, one corrective batch,
one 38-pair confirmation; final 8 sheets/20 originals read. **BLOCKER:** note input
still paints at padding-only width at both sizes; the editor's outer bounds test
does not establish inner input readability. Bounded visual loop stopped; no pass
or Phase 5. Receipt verifies 12 immutable archives, 182 original-root PNGs unchanged
since Phase 1, 38 source-bound pairs and 62 source hashes, with one explicit
post-capture Zoom unit-assertion-only change. Format/diff checks pass. Exact report,
log mirrors, receipt and integrity checker are in `research/whole-app-ui-review-20261012/phase4-*`
and `scripts/verify-phase4-receipt.py`. No commit/push, nested agent, new browser
visit/download, capture pixel/schema/approval-semantic change or native/AT/platform/
usability/release qualification. Source handoff also identifies missing Guided(53)
loading/failed-read restoration allowlists. **Next: independent Phase 4 verification
and named note-width/loading-navigation follow-ups; do not mark the phase verified
or start Phase 5.**

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
