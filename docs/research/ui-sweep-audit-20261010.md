# Regressee UI sweep — source discovery, 2026-10-10

## Scope and evidence

Read-only discovery of the native Rust/GPUI shell, focus model, shared controls,
input and product/design briefs. Only this report was added. No builds, tests,
app launches, desktop inspection or subagents. This is **not final verification**.
Locations refer to source read during this audit; concurrent implementation may
move lines, so function/control names are included.

The user reports that plain `target/debug/regressee` opens and permits adding a
project. That is useful new user evidence, **not** a resolution or reattribution
of the historical native inspection failure. Historical reports stay untouched.
Rendered geometry, contrast, OS input and accessibility remain unverified here.

Direction: **Operate**, preserve the established light cool/teal palette and
native desktop structure. Improve task clarity, not branding or dashboard
decoration. Desktop/tablet/mobile are website capture profiles, not app layouts
(`PRODUCT.md:14–17,29–47`; `DESIGN.md:83–93,125–132`).

## Prioritized findings

P1 = substantial task difficulty or misleading behavior; P2 = important friction
with a workaround. No new native P0 failure is asserted from this source audit.

### P1 — fix first

1. **Environment and network fields are disabled by an unrelated disclosure.**
   `crates/app/src/main.rs:4190–4200` (`render`) enables crawl fields only when
   `i < 3 || i == 9 || focus.advanced`. This also disables visible login/mapping/
   locale/timezone fields 10–16 and Settings proxy/CA fields 17–18 until users
   open Setup's Advanced controls. `crates/core/src/shell.rs:162–182` nevertheless
   includes those fields in keyboard traversal. **Proposal:** determine input
   availability from its actual screen/section and job state; restrict the
   advanced flag to fields 3–8. Use the same availability predicate for rendering,
   focus and dispatch.

2. **Tab order contains invisible, duplicate and undefined actions.**
   `crates/core/src/shell.rs:151–166,185–233` enumerates broad numeric ranges:
   Setup includes Guided(11) twice when advanced; Results includes Phase5(30)
   twice with a viewer and Phase5(29)/(31), which have no action handler;
   AnalysisField(0–8) is listed even when its advanced section is hidden.
   History includes all Phase6 controls regardless of preview visibility.
   Worse, `main.rs:1530,1534` maps Phase5(40–42) and Phase6(0–2) to the **same
   handles 180–182**. History lists Phase5 first, so `focused()` (`1539–1543`)
   resolves keyboard activation of Import ZIP to Previous version, and activation
   of Preview ZIP contents to **Set displayed version active**. Pointer dispatch
   and keyboard dispatch can therefore perform different operations.
   `main.rs:1539–1554` also focuses handles without checking mounted elements.
   **Proposal:** build an ordered visible-control list per section, with explicit
   entries and prerequisites; eliminate duplicates and allocate disjoint handle
   ranges (prefer a typed Control-to-handle map). Follow rendered order and
   keep hidden/disabled items out of Tab. Busy viewer focus must also match the
   visible analysis section (`core/src/shell.rs:109–128`).

3. **Progress, errors, confirmation feedback and cancellation are after the
   entire page.** `main.rs:4216–4260` appends these below `self.content`, while
   Setup/Results/History can contain many screens of content. Save failure or
   capture progress may therefore be out of view; Pause/Cancel are equally
   remote. **Proposal:** put current work and its Pause/Cancel controls in a
   stable shell status area above the scrolling body; put actionable errors next
   to the relevant field/section and retain a short top-level summary. Keep
   unknown work indeterminate and the existing safe-boundary semantics.

4. **Clicks conceal selection or cycle through undisclosed options.**
   `main.rs:1857–1862,1969–1970,2045–2061` and
   `2271–2303,2332–2337,2577` use ordinary secondary buttons for discovery mode,
   profiles, speed, environment, filters, access and login persistence. Labels
   change length and mix the current value with the next action. Sensitivity and
   viewer-mode buttons have no selected styling (`3477–3489,3586–3596`), nor does
   per-target approval selection (`3682–3705`). **Proposal:** stable field labels
   plus explicit selected segmented options for short exclusive choices;
   checkable profile rows for multi-select; an environment popup/list for longer
   choices. Show a check/word and tonal selection independently of keyboard
   focus. State is not just a changing button sentence.

5. **Safety permissions look like reversible display toggles.**
   `main.rs:1865–1870,2301–2312` changes robots authorization/action-link
   permission directly on click; the important consequences are buried in long
   copy. **Proposal:** ordinary safe defaults; place exceptional permissions in
   clearly labeled advanced safety controls with a deliberate authorization
   confirmation. Keep origin approval, action-link exclusions, runtime ownership
   and network/TLS checks unchanged. Do not simplify by granting permission or
   expanding scope automatically.

6. **Two Save actions have incompatible scopes and can discard environment
   drafts.** `main.rs:1747–1767` saves the stored environment list rather than
   `draft_settings.environments`; `367–395` then replaces the entire draft with
   the saved project. Clicking the bottom "Save project settings" after changing
   environment/access/mappings can therefore lose those drafts. Conversely
   `2090–2135` saves environment/rendering changes from a different snapshot.
   **Proposal:** clearly bounded sections with section-specific dirty state and
   save labels, or one coordinated Save that preserves both draft scopes. Never
   reset unrelated unsaved fields after either save. Surface which section still
   needs saving and retain immutable-baseline behavior.

7. **Results is an action/prose stack before it is an evidence viewer.**
   `main.rs:3249–3252,3444–3741,3416–3439` places login controls, a 300px inventory,
   analysis metrics, masks, notes, reviews and potentially 64 approval references
   before the 280px-high screenshot panes. At the 820×600 minimum window
   (`4358`), users scroll extensively to see the evidence they are reviewing.
   **Proposal:** compact project/run/coverage header, target queue and prominent
   selected-target viewer; place view mode/zoom adjacent to images. Put advanced
   analysis, mask editing, provenance and acceptance details in expandable
   sections. Retain conspicuous capture/comparability warnings and explicit
   approval impact review. Measure pane bounds for a Fit width option; current
   default zoom is 50% (`215`), not a fit-to-pane value.

8. **Unavailable actions still look actionable or do nothing.** Examples:
   `core/src/shell.rs:164,186,209,218,230–233` broadly enables login/history/review
   actions; `main.rs:887–920,926–976,1009–1021,1072–1108,1390–1419` silently does
   nothing when there is no pending analysis, region, rectangle, request, preview,
   version or next history page. Login buttons always render (`2384–2395`), even
   when the login browser is closed. **Proposal:** explicit prerequisites shared
   by focus/render/dispatch: confirm/close only for an open login browser; region
   navigation only with regions; selected-mask operations only with selection;
   Cancel analysis only during analysis; history navigation only when it changes
   selection; acceptance only with current eligible selection. Explain blocked
   prerequisites nearby. Preserve backend refusal checks as the authority.

9. **Reopening evidence can erase unsaved ignored areas.** Target changes are
   guarded (`main.rs:2016–2020,2639–2641`), but RefreshEvidence dispatches freely
   (`2784–2787`); its reply resets `mask_dirty` and reloads mask state
   (`522–563,600–606`). **Proposal:** Save / explicitly Discard / Keep editing
   before evidence reload, project switch or any destructive draft replacement.
   Preserve exact page/profile/pair bindings; never carry rectangles to another
   target to avoid prompting. Extend dirty handling to review-note edits and
   explicitly guarded quit where supported (`4002–4090,4329–4348`).

10. **Native text editing lacks familiar selection and composition feedback.**
    `crates/app/src/ui/text_input.rs:411–426` only collapses selection on mouse
    down; there is no drag selection, Shift-click extension or double-click word
    selection. Bindings `439–448` omit Shift+arrow/Home/End. Marked IME text is
    tracked (`208–245`) but no marked underline is drawn (`319–331,356–378`).
    **Proposal:** preserve the existing UTF-16/UTF-8 conversion and clipboard
    path; add anchored mouse/keyboard selection, word selection and visible IME
    composition using ShapedLine offsets/TextRun underline. Keep caret/selection
    within fixed field geometry and clipping. Reject limits with clear feedback,
    rather than the current silent return (`190–195`).

### P2 — next usability pass

11. **Duplicate Results entry points compete.** `main.rs:3192–3247` offers both
    contextual Find/Capture and "Find pages for a replacement baseline";
    discovery preview adds another "Capture reviewed pages" (`2513`).
    Guided(16) is labeled "Review existing inventory" at the top even when its
    handler dismisses a preview (`1953–1959`), duplicating the preview's Dismiss
    control (`2514`). **Proposal:** one primary action per current stage (Find →
    Review/capture → Save baseline, or Check), with replacement options grouped
    separately. Render a given Control/ElementId/FocusHandle only once.

12. **Projects and History do not establish a stable project context.**
    Projects rows show only name/address and opening always routes to Setup
    (`main.rs:2681–2686,2902–2929`). The rail does not identify the current
    project (`4271–4276`); History lacks its project name (`3006–3007`).
    **Proposal:** show selected-project identity in the shell and every
    project-scoped heading; selected row treatment independent of focus. If
    authoritative summaries are available, add a truthful "No baseline yet" /
    baseline status, not invented activity. Make "Project settings" explicit
    when opening setup; offer a clear Results destination for existing work.

13. **History is raw identifiers rather than a selectable library.**
    `main.rs:3050–3082,3138–3169` foregrounds UUIDs, raw timestamps, `Some(...)`,
    artifact hashes and rendering internals; versions/runs are selected via
    wrapping Previous/Next (`1084–1094,1390–1399`). Storage shows raw bytes
    (`3091–3096`; Settings `2965–2968`). **Proposal:** selected version/run list
    with local-readable date, capture status, target count and Active/Imported/
    Composite labels; format byte counts as KB/MB/GB. Retain full IDs, source
    dates and integrity detail in an expandable evidence section. Put Export,
    Restore and deletion-impact actions adjacent to the selected item; keep
    privacy previews, irreversible deletion confirmation and reference protection.

14. **The first-use/settings narrative reads like a qualification report.**
    `main.rs:2297–2302,2338–2354,2892,2960–2975,1212–1217,4275–4278` mixes essential
    instructions with implementation caveats: DPR, DOM coverage, budgets,
    database/WAL, Node/robots constraints and process-ownership qualification.
    **Proposal:** short task-first summaries and contextual "Details and
    limitations" disclosures. Browser setup should visibly say Ready /
    Unavailable / Needs repair and the next supported step; retain honest
    development-only/public-runtime restrictions and accessible diagnostics.
    Never replace missing trusted configuration with a system browser/runtime.

15. **Long URLs and button sentences lack reliable layout policy.**
    `main.rs:2874–2889` does not wrap the Projects heading/action row;
    `2913–2922,3051–3076,3148–3167` expose potentially long strings without a
    consistent truncation/detail strategy. Fixed 60px target rows hide overflow
    (`2534–2558,2613–2635`). `controls.rs:29–40` has minimum rather than stable
    height and no width/wrapping contract. **Proposal:** bounded form columns,
    flexible/min-width-zero text, ellipsis plus full-address detail/copy, wrapping
    action groups and stable short control labels. Use a single-line fixed target
    height for ordinary actions; use a separate explicitly multiline treatment
    for longer consent text.

16. **Reported focus growth is not explained by a focus border change in the
    current source.** `controls.rs:34,51–55` always uses a one-pixel border and
    only changes colors on focus; `text_input.rs:392–399` likewise keeps geometry.
    Do not claim that a thicker focus border caused the observed growth. Dynamic
    labels (finding 4), wrapping/minimum-height layout (finding 15), or different
    running binary bytes require later native verification. **Proposal:** retain
    fixed padding/font/border dimensions across default/hover/focus/selected/
    disabled states; a painted inset outline can strengthen focus without
    affecting layout. Distinguish focus from selection; do not wash out the
    primary action's role merely because it is focused. Owner: shared-controls
    implementation agent; parent should stabilize call-site labels/layout.

17. **Search/filter navigation has no explicit empty/end state.**
    `main.rs:1962–1970,1991–2014,2476–2479,2516–2569,2577–2663` applies search by
    button, cycles filters, clamps Previous/Next, and renders an empty list with
    no reset/empty-match explanation. Search reconciliation handles preview
    selection but not the selected captured target (`1964–1967`). **Proposal:**
    show visible/total count, Clear search, named filter options and "No matching
    pages"; reconcile selection without discarding mask drafts; disable ends.
    Retain a UniformListScrollHandle to reveal keyboard-selected rows. Group by
    readable profile names with visible group headings, not just UUID sorting
    (`2457–2464`).

18. **Some displayed state contradicts the actual baseline state.**
    `main.rs:3299–3308` always labels a multi-target completed baseline capture
    "explicitly save to activate", even when already activated; the single-target
    branch distinguishes approved state (`3277–3283`). **Proposal:** derive
    status from intent, completion, active-baseline identity and candidate state,
    then show an accurate next action. Keep run-level coverage separate from
    selected-target classification and review decision.

19. **Inspect folder is a dead-end tool, not a setting.**
    `main.rs:2701–2709,2989–2999` opens a picker but only emits a notice; it neither
    opens the location nor changes data storage. **Proposal:** remove this
    exploratory control from normal Settings or explicitly call it "Choose a
    folder to inspect" with a meaningful supported follow-up. Prefer actual
    current data/cache paths with Copy path/Open folder if implemented. Do not
    imply relocation exists or move evidence as a side effect.

20. **Readability/accessibility is only partially represented by source tokens.**
    Disabled opacity is 0.5 (`controls.rs:57`, `text_input.rs:391`) and many
    operational explanations use 13px (`main.rs:2297–2302,2353–2354,3555–3558`).
    Target rows and overlay slider are pointer-only elements
    (`2532–2563,2611–2658,3600–3644`), though Previous/Next and Less/More give
    partial keyboard alternatives. **Proposal:** keep disabled text legible with
    explicit tokens and reason text; use 15px body for essential instructions;
    reserve 13px for secondary metadata. Give the active target and drawing mode
    visible non-color-only state. Validate keyboard geometry and OS scaling
    later. GPUI 0.2.2 does not expose an obvious general screen-reader semantics
    API in the inspected source: do not promise an ARIA-like fix or a screen-reader
    pass; track native accessibility integration as a separate qualification gap.

## GPUI implementation feasibility / ownership

- **Parent (`main.rs`, `core/src/shell.rs`):** visible-control predicates/order,
  bounded sections and dirty-save scopes, contextual action groups, explicit
  option selection, top-level feedback, viewer hierarchy, history formatting,
  accurate state labels and no-op suppression. Existing Div/FocusHandle/actions,
  native `Window::prompt`, `uniform_list`, `canvas` and `img` suffice; no webview
  or replacement UI framework is required.
- **Shared UI owner (`ui/controls.rs`, `ui/theme.rs`, `ui/text_input.rs`):** stable
  control geometry, selected-vs-focused treatment, disabled tokens, checkable/
  segmented primitives and native text selection/composition. GPUI 0.2.2 offers
  `Div::on_mouse_move`, `on_mouse_up`, `on_mouse_up_out`, `text_ellipsis`,
  `UniformListScrollHandle::scroll_to_item`, TextRun and canvas painting. Keep
  original screenshot/mask coordinate calculations unchanged when fitting panes.
- **Later independent verifier:** inspect representative first-use, saved setup,
  login-open/closed, preview, completed/incomplete check, mask draft, history and
  Settings at 1120×800 and 820×600. Check focused/selected/disabled bounds,
  control-to-focus correspondence, feedback visibility and text selection. This
  report intentionally performs none of those verification operations.

## Preserve

The existing local-first structure, virtualized target lists, fixed baseline
inventory, immutable originals/revisions, pair-bound masks, explicit baseline
activation/acceptance, stale-preview refusals, privacy-before-export preview,
separate environment sessions, no plaintext authentication fallback and no TLS
bypass are strengths. Progressive disclosure must not conceal essential scope,
privacy, coverage or destructive-action warnings, and UI convenience must never
silently relax runtime/safety checks. A cleaner working desk is the goal—not a
new brand, synthetic dashboard or invented qualification pass.
