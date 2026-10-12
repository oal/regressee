# Whole-app workflow blueprint — Phase 3

**2026-10-12 · selected under the user's delegated judgment · planning only.**
This is the short build contract for [Phases 4–7](../../whole-app-ui-plan.md),
not participant approval, a usability result or a description of already-built UI.
Authority remains [PRODUCT](../../../PRODUCT.md), [DESIGN](../../../DESIGN.md)
and the [approved workflow](../../workflow-revamp.md); rationale/evidence is the
[complete synthesis](synthesis.md), with immutable [A](assessment-a.md)/[B](assessment-b.md).

## Decision and job

For a PM establishing a first baseline or reviewing 100–1,000 distinct pages,
make the next **safe** action clear and leave enough evidence to inspect a change.
Keep Clear wayfinding: deep-blue navigation, white surfaces, sky selection,
system text, shared 40px controls and untinted originals. No identity interview,
new decoration, mobile-app shell, backend scope expansion or larger minimum.
**820×600 remains supported; 1280×800 remains the normal qualification size.**

Select full-plan scope, in order: **4A consequential consent → 4B daily inspection
→ 5 first-baseline setup/discovery → 6 history/sharing/recovery → 7 qualification**.
Historical ask/approval gates are superseded by explicit user delegation, not by
invented participant agreement. Read the Impeccable craft floor immediately before
the first production UI edit; do not rewrite DESIGN's built-behavior claims early.

## Ownership and evidence-focus topology

| Owner | Owns | Must not imply |
|---|---|---|
| Project navigation | Selected project/full identity, destinations, local Collapse navigation/Show navigation | Changing destination or project merely because a panel collapses |
| Queue | Path-first target rows; separate profile/status; complete URL retrieval/opening; search, filters, grouping, target navigation; local Hide results/Show results | Filtering changes the inspected target, drafts or baseline selection |
| Viewer | Compare, full-width Before/After, Difference, Overlay, Heatmap; fit/manual zoom, synchronized pan, concise manipulation help; Details and Analysis | An analysis tint is an original or missing evidence is unchanged |
| Review | Intentional / Needs fix / Unreviewed as one disposition group; explicit note save; separately emphasized Next unreviewed and review progress | Intentional automatically selects or updates a baseline |
| Baseline update | Select for baseline update, selected/eligible scope and consequence preview; explicit Confirm baseline | A generic Approve combining review, selection and commit |

**1280×800:** navigation and queue initially open, side-by-side Compare retained.
**820×600 (narrow layout below the existing 1100px breakpoint):** start inspection
with project navigation collapsed to a labeled local restoration affordance; queue
remains open initially so the target is discoverable, with Hide results in its own
heading. When hidden, Show results stays at the viewer's left edge. Keep selected
project, target path/profile, before/after environment and evidence state visible
even when their navigation panels are closed. Both collapses are presentation-only.

Keep **Compare** as the default comparison mode at both sizes; never silently swap
it on resize. Add an explicit **Before/After (full width)** alternative, with labeled
Before and After selection and current side always visible. It shows one untinted
original at a time, not a blend or replacement of Compare. Difference/Overlay/Heatmap
remain explicitly named in the same viewer mode owner at both widths, not hidden
inside Analysis. Use a bounded mode chooser if labels cannot fit, distinct from
Zoom/Details/Analysis tools; no smaller application text or unlabeled chevrons.

Before↔After preserves the exact target, zoom policy, manual percentage and source
pan anchor. Both originals share one transform. For layout changes, automatic Fit
recomputes to actual available pane width; manual zoom stays fixed and pan retains
the inspected source point, clamped only to valid geometry. Switching modes or
collapse must not reload a different target, reset drafts or write evidence.
Entering full-width view may refit only if already in automatic Fit; switching
its Before/After side does not refit. Restore panel preferences within the workspace;
resize does not repeatedly override a user's explicit collapse choice. New controls
and this presentation state are additions to implement, not existing source symbols.

Notes, Details, filters and Analysis must not become a smaller-evidence decision
trap: use a bounded owned editor/disclosure with Close and visible overflow; keep
target/context and a reachable Return to evidence. Preserve the inspection viewport
rather than adding a permanent note row below it. At minimum width, a disclosure
that must cover evidence is an explicit temporary editing/reading state; do not
leave classification/confirmation actionable behind it as though images are visible.
Closing notes preserves unsaved text; mask drawing folds the editor and returns
after the rectangle without saving implicitly. Coordinate editing exposes X/Y/
width/height, Apply and Save through its own focus-revealing scroll owner.
Capture-only has one original and no comparison modes/Analysis; authoritative empty
capture reserves no blank queue. Select an initial capture overview capped at 100%
for narrow originals; manual enlargement and explicitly chosen zoom remain available.

## Consent first, provenance second — Phase 4A

Title: **“Update the baseline for 1 screenshot?”** (correct plural for many).
Lead with selected replacement path, screen size, environment and readable UTC
capture time. Plain consequence: **“Future checks will use these screenshots.
Other baseline screenshots stay unchanged. Earlier versions remain in History.”**
Mixed capture times remain disclosed: this is not one simultaneous site snapshot.

Use one bounded reading flow with pinned Keep reviewing and Confirm baseline.
Selected replacements lead; unchanged screenshots have their own summary and
inspectable rows. Row role **Replace/Keep** and **Inspecting …** association remain
explicit, with full-width inspected paint distinct from focus. Technical provenance
is an on-demand section of that same flow, not a second tiny always-open scroller:
exact IDs, source runs, artifacts and requested/final URLs remain retrievable.
Budget row line heights/padding/paint so separators never cross text; show overflow
and make the last row/tail reachable. Preserve explicit commit, Escape cancellation,
focus trap/restoration, stale-preview invalidation and backend eligibility checks.
Whole-baseline replacement remains a separate discovery/capture/save route.

## Guided next action — Phase 5

Always show Website → Access → Pages & sizes with current/completed/unavailable
state. Public/private choice and immediate action must fit the first minimum
viewport. Existing-project Settings remains complete with explicit Save settings.

| State | Dominant action and boundary |
|---|---|
| New/changed Website | Save website & continue; validate locally, advance only after authoritative save success; no site visit |
| Clean public Access | Continue to pages & sizes; no unnecessary save or browser visit |
| Dirty Access / ordinary stage edits | Save & continue; await exact save reply, retain failed/newer draft and reveal invalid input |
| Private Access not ready, dirty | Save settings first, then Open login browser as separate explicit action by default |
| Private Access saved, not ready | Open login browser; names the visit; while open, I'm ready explicitly requests readiness check |
| Private Access ready | Continue to pages & sizes; readiness belongs to that saved environment, not another draft |
| Pages clean / dirty | Find pages & preview / Save & find pages; this label explicitly authorizes the visit, never settings save alone |
| Discovery ready/limited | Capture included pages with screen-size target estimate and nearby limitation cause/next step; baseline still requires explicit save |

Combined **Save & open login browser** is permitted only if Phase 5 implements and
tests an identity-bound continuation after successful save. Never merely relabel
Save to promise a visit it does not perform; on failure, stale reply, environment
switch or newer edits, no browser opens. Show saved/draft/browser state and immediate
prerequisite together. Maintenance (close/forget), retention, mapping and probes
stay available but subordinate. Show a concrete source→capture mapping before long
scope text. Describe bitmap/photo replacement and “Only for this app session” in
plain language without weakening limitations or secure-store behavior.

## Capability inventory and concrete build owners

All paths below are current source owners, not historical line numbers. Inventory
was checked against source in Phase 3; availability is not operational qualification.
`Shell` below means the app shell unless explicitly labeled core.

| Retained capability (including advanced) | Concrete owner / controls |
|---|---|
| Multiple projects, chooser, destination-preserving switch, draft protection, fixed inventory/check versus replacement | [main.rs](../../../crates/app/src/main.rs): `activate_control`, `save_settings`, `guided`; [workflow.rs](../../../crates/app/src/workflow.rs): `results`, `project_rail`, `setup_heading`, `setup_footer` |
| Starting URL list, multi-origin consent, only-addresses/linked discovery, page limit, section, skipped text; inclusion/search/no-match; estimate, stop/partial inventory, representative preview | `crawl_setup`, `inventory_and_results`, `preview_page_ids`; [options_ux.rs](../../../crates/app/src/options_ux.rs): `discovery_options`, `discovery_coverage_hint`, `options_panel` |
| Desktop/tablet/mobile web profiles, custom width/height add/replace/remove; full-page/visible-screen; normal/replaced images and frozen runtime/DPR/emulation limitations | `crawl_setup`; `Guided(1..=3,12,25)`, `CrawlField(7..=8)`, `Choice::{FullPage,VisibleScreen,OriginalImages,ReplaceImages}` |
| Gentle/Balanced/Fast, robots override, link depth, global/per-site parallelism, retries, action-like links, tracking stripping, explicit fragments | `crawl_setup`; `CrawlField(3..=6)`, `Guided(7..=10)`, speed choices; do not remove advanced bounds/help |
| Separate production/staging sessions, public/login access, secure/ephemeral retention, login/representative URLs, approved identity-provider origins, open/ready/close/forget, mappings/preview/probe, locale/timezone/website theme | `environment_setup`, `environment_actions`, `guided`; `CrawlField(10..=15)`, `Environment`, `Guided(27,31..=38)`; `mapping_preview`; `stage_control_mounted` |
| All four existing image modes plus new full-width alternative; fit/actual/presets, pointer/keyboard pan/zoom, overlay fraction, change-region navigation, Details/technical provenance | `viewer_section`, `viewer_panes_layout`, `viewer_transform`; `viewer_pane_indices`, `pane_caption`, `configure_review_geometry`; [viewer_ux.rs](../../../crates/app/src/viewer_ux.rs): `viewer_key`, `viewer_scroll`, `zoom_viewer_at`, `clamp_viewer_pan`; [zoom_select.rs](../../../crates/app/src/ui/zoom_select.rs): `ZoomSelect` |
| Sensitive/Balanced/Relaxed, reanalyze/cancel/previous analysis, revision metrics/warnings; all nine advanced fields: ΔE00, global %, local %, minimum changed/eligible pixels, window size/stride, connected-region pixels, ignored-coverage warning %; page-height trigger and RGB-blend suppression | `phase5_panel`, `phase5_action`; `AnalysisField(0..=8)`, `Phase5(0..=5,26..=28)`; retain optional-criterion and suppression-risk explanations |
| Ignored rectangles draw/select/redraw/delete/revalidate, integer geometry Apply/Save, three review states, explicit notes, next-unreviewed, historical decisions, selective update preview/confirmation | [review_ux.rs](../../../crates/app/src/review_ux.rs): `keyboard_mask_editor`, `review_ux_action`, `ordinary_review_bar`, `capture_provenance_details`, `navigation_preserves_drafts`; `approval_bar`, `approval_decision`, `approval_selection_eligible`; `approval_matches`, `phase5_action` |
| Runs/baselines/storage, loaded-page filter/type/paging, retained/composite provenance, read-only original-policy inspection, ZIP preview/import/export, deletion/shared-object protection, human/exact storage usage and low-space/manual cleanup | [workspace_history.rs](../../../crates/app/src/workspace_history.rs): `baseline_history`, `open_history_run`, `receive_history_inspection`, `history_bytes`; `portability_confirmation`, `phase6_action` |
| Browser setup/integrity/repair, explicit proxy/CA settings, local diagnostics, interrupted-write versus complete-recovery cleanup, folder/help/About, loading/empty/failed reads, pause/resume/stop/completed tasks | `guided`, `portability_confirmation`, `phase6_action`; [loading_ux.rs](../../../crates/app/src/loading_ux.rs): `screenshot_state`, `screenshot_placeholder`; [task_ux.rs](../../../crates/app/src/task_ux.rs): `Task::outcome`, `task_panel`, `continue_task`; `Control::{SetupRuntime,PauseCapture,Shortcuts,About}` |

## Safety and focus matrix — invariants, not claimed test passes

| Boundary | Preserve / later proof |
|---|---|
| Evidence and approval | Missing/failed/incompatible/preview-only or stale analysis cannot update; pending analysis/dirty masks block update; preview and commit identities agree; older versions retained; fixed inventory/image mode unchanged |
| Draft/save/login | Save never implicitly visits or changes a baseline; settings/note/mask saves remain distinct; IME composition, invalid/newer drafts and save failures preserved; project/environment/quit guards await authoritative replies |
| Inspection/filter/layout | History inspection never activates a baseline or writes display pointers; filters/collapse/mode switches preserve target/drafts/selection outside filters; stale viewer/history replies rejected; no capture pixel edits |
| Sharing/privacy/network | Originals can contain secrets; ignored areas are not redaction; ZIP excludes sessions/cookies; recipient establishes own login; explicit local export; secure store has no plaintext fallback; TLS stays verified |
| Cleanup/tasks | Manual deliberate deletion only; active/shared/retained evidence protected; temporary cleanup distinct from permanent recovery deletion; zero-work disabled; committed save cannot be canceled retroactively; honest coverage/no fabricated ETA |
| Runtime/performance | Database/filesystem/process/image work remains off UI thread; bounded lists/worker requests retained; build's missing trusted browser distribution is a blocker, not a UI pass |

At **both sizes**, update [core shell](../../../crates/core/src/shell.rs)
`FocusState::{controls,next,can_activate}` and app `control_mounted`,
`control_enabled`, `workspace_control_mounted`, `stage_control_mounted` together
with new UI controls. Tab order follows visible ownership: navigation restoration/
project context → queue controls/rows (when open) → viewer mode/tools/image → review
→ separate baseline selection. Hidden/disabled/occluded controls leave traversal
and reject activation; collapse moves focus to its local restoration control.
Reopening restores an appropriate queue/navigation control and reveals it.
Ctrl/Cmd+F restores the owning queue if necessary, then focuses mounted search,
never escapes a modal. Menus/pickers use arrows/Home/End/Enter/Escape; disclosure
Escape restores its trigger, dialog Escape keeps reviewing, and background mutation
is blocked. Preserve Ctrl/Cmd shortcuts, field clipboard/IME, draft/quit protections
and focus-revealing scroll anchors. Native AT names/status announcements and real
OS input are Phase 7 obligations, not established by simulated focus tests.

## Migration and verification handoff

No schema, evidence, baseline inventory or capture-policy migration is planned.
Add presentation-only state and named controls in bounded production batches;
refresh measured geometry from actual mounted rail/queue/pane widths, not the old
fixed `width - 240` assumption. Preserve existing saved data, drafts, references
and exports; do not persist presentation fixture clones or auto-save baseline edits.

1. **Phase 4A R3:** consent copy/flow/paint plus one/many/long-address replacement
   and last unchanged row/technical tail tests. **4B R1/R2/R4/R5/M1:** both collapse
   owners, full-width side switching, all modes, synchronized source-point pan/zoom,
   fit/manual resize, filtered-out selection, notes/masks/eligibility and initial
   capture overview. Extend existing workflow/review/viewer/core-shell tests.
2. **Phase 5 S1–S3:** public/private clean/dirty/invalid/open/not-ready/ready routes,
   all three stages, authoritative save continuation failures, mapping-first view,
   inclusion/limit estimates and full advanced inventory traversal.
3. **Phase 6 H1–H3/F1–F2/remaining M1:** retained versions, last loaded History row,
   filter scope and overflow, share/import/delete safety, zero/permanent cleanup;
   exercise actual storage-backed failed-comparison routing before generalizing
   the cloned `viewer=None` fixture; recovery labels name implemented destinations.
4. **Phase 7 Q1:** isolated distinct storage-backed 100–1,000-page fixtures, full
   workflow safety regression and complete advanced/diagnostic tails; native input,
   clipboard/IME, resize, keyboard/AT, scaling/high contrast/motion on declared OS
   matrix. Linux AT/NVDA/VoiceOver unavailable runs say **not tested**; actual PM
   participants/results required before usability/throughput claims.

For each changed batch, source-bind native GPUI captures at **1280×800 and 820×600**,
assert named state and visible owner/paint bounds, preserve superseded pairs, and
inspect originals once together. One fix batch and at most one confirmation batch;
remaining defects become documented limits. Reuse unchanged evidence, not a blind
182-frame rerender. Phase 7 combines the complete sequence. Offscreen layout,
source tests, real OS input, accessibility, usability and release outcomes remain
separate. The 148 repeated-image clones are presentation only; original 24/40 is
not rescored here. Diagnostics/advanced tails, runtime and platform limits remain
open. CI/release work follows all phases; Phase 3 neither renders nor starts Phase 4.
