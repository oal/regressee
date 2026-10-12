# Assessment A — independent whole-app design review

**Method:** isolated Assessment A; detector findings, Assessment B and prior critique archives were not consulted. This is an Operate-mode review of the native Rust/GPUI desktop workflow, not a mobile-app or browser-interface review. **Verdict: significant usability fixes needed; 24/40.** No product-code or design-document edits were made.

## Design specificity — judgment made before detector evidence

**Coherent, category-fluent, and partially product-specific; not yet a convincing non-developer review workstation.** The deep-blue rail, system type, white surfaces and small outlined controls are appropriate familiar desktop-tool conventions. Their familiarity is not the problem and does not call for a new visual identity. The authored parts are the distinctions between capture, comparison, saved baseline and historical inspection; coverage accounting; and explicit baseline consequences.

The interaction hierarchy is less product-specific than the documentation claims. Much of the application still looks like a collection of settings and record-inspection panels. It asks the PM to operate the tool, translate its storage vocabulary and navigate its disclosures before evaluating the website. The dominant review arrangement privileges permanently visible application chrome over legible evidence. Repeated full origins consume the page queue while the useful path disappears. The signature should be **confident release review**, not an accumulation of safety paragraphs and database provenance.

The biggest opportunity is not more personality or decoration. It is assigning every control a clear owner, making the inspected page unmistakable, and giving screenshot evidence enough space to support a decision. The incumbent Clear wayfinding world can survive all of these fixes.

## Evidence and limits

- Visually inspected **all 32 contact sheets**, covering all **182 captures**: 71 whole-app states at 1280×800 and 820×600, plus 20 task states at both sizes.
- Read **44 selected full-resolution PNG originals**, including both ordinary review sizes, both 148-target presentation-scale reviews, capture-mobile at both sizes, discovery options at both sizes, setup stages and lower settings, approval at both sizes, historical inspection, sharing/deletion, error states and task states. A complete original-read inventory is below.
- These are actual calibrated Linux GPUI/Blade offscreen pixels, not a browser reconstruction. The source supplies behavior explanations, not usability or accessibility proof. Current `main.rs` SHA-256 `391182da75358bcfb85bdb886d6e290dbd1b5136a3aa8b567c8faa97ceaf7bdd` and `task_ux.rs` SHA-256 `3ccc64babe9f1d850d5d8982c00310fff4986bea400198e0965723a102ae882f` match inspected capture metadata.
- The rich mock cohort contains four local projects, twelve stored comparison references, 74 discovered pages and desktop/mobile website imagery. `review-mixed-148` is **presentation-only scale evidence with repeated imagery**: it does not establish 148 independent comparisons, approval safety at scale, throughput or 1,000-page usability. Task images use the older explicitly synthetic cohort without website imagery. Historical dates are illustrative, not current-live-data defects.
- A new gallery tab was attempted with `browser.tabs.open`; the tool returned `browser.disconnected`. **No tab was created; new-tab ID: none.** No further disconnected browser actions were attempted. The shared gallery server was not stopped.
- No new rendering, QA loops, website visits or user-project access. Windows/macOS rendering, actual Linux OS presentation, native keyboard/pointer/IME input, screen-reader semantics, high contrast, OS text scaling and cross-platform file/dialog conventions remain unverified. Static frames cannot establish speed, task success or error recovery in practice.
- This assessment does not claim detector findings or a native-image detector overlay. The scoring is a reasoned design assessment, not release or accessibility certification. No P0 task-completion blocker is established by this evidence.

## Design health score

All ten heuristics apply to this desktop task tool. Scores assess visible design and supported intent; they do not certify unexercised native interactions.

| # | Nielsen heuristic | Score /4 | Assessment |
|---|---|---:|---|
| 1 | Visibility of system status | 3 | Capture/comparison/inspection states, truthful counts and progress are strong. Generic task failure titles, terse limited-inventory status and some scope ambiguity prevent excellence. |
| 2 | Match between system and real world | 2 | Ordinary-language task titles coexist with references/inherited/composite, raster, Ephemeral, raw UUIDs, object counts and a Rust debug representation at consequential moments. |
| 3 | User control and freedom | 3 | Visible cancel/close, explicit saves, retained versions, clear-search and inspection-only framing support control. A paused task does not offer the action its copy asks for; actual undo/focus/escape behavior was not exercised. |
| 4 | Consistency and standards | 2 | Strong visual consistency, weaker interaction consistency: setup stage visibility changes, narrow modes move into Details, forward actions lose primary emphasis, and cleanup deletion looks secondary. Host-OS behavior remains unqualified. |
| 5 | Error prevention | 3 | Explicit baseline confirmation, blocked missing evidence, invalid-address feedback, privacy warnings and deliberate deletion are valuable safeguards. Dense consent copy can defeat comprehension; no operational safety certification is implied. |
| 6 | Recognition rather than recall | 2 | Labels and context are generally present, but truncated same-origin addresses make rows hard to distinguish; narrow captions omit environment context; historical/deletion identities require translation and sometimes context reconstruction. |
| 7 | Flexibility and efficiency | 2 | Search, page inclusion bulk actions, grouping, zoom, next-unreviewed and visible keyboard help are promising. Efficient high-volume review/selection is not demonstrated, and native accelerators have not been exercised. |
| 8 | Aesthetic and minimalist design | 2 | Restrained, readable application styling is a strength. Too much equal-weight chrome and repeated caution compete with the evidence; minimum-width analysis obscures almost the entire screenshot workspace. |
| 9 | Recognize, diagnose and recover from errors | 3 | Invalid-address corrections and capture-gap reasons offer concrete next steps and preserve evidence. Generic operation failure and indirect resume/retry routes remain weak. Static frames do not prove successful recovery. |
| 10 | Help and documentation | 2 | Useful inline guidance and a keyboard-help dialog are visible. Advanced terminology is inconsistently explained, help is wordy, and task-focused searchable support is not demonstrated. |
| **Total** | | **24/40** | **Acceptable — significant improvements needed. Not a release pass.** |

## What is working

1. **Evidence integrity is reflected in the visual language.** Before/after originals remain untinted; Difference/Overlay/Heatmap are distinct analysis modes. Capture-only work has one original rather than an invented comparison. Gaps are explicitly not unchanged pages. See `all-1280x800-review.png`, `all-1280x800-review-difference.png`, `all-820x600-capture-gaps.png`, `all-820x600-review-failed-target.png`.
2. **Persistent context and explicit consequences are better than a generic dashboard.** Selected-project context, actual coverage, saved-vs-unsaved baseline state and inspection-only history distinguish things PMs must not conflate. Four projects demonstrate useful selected-state labeling, not just a blank shell. See `all-1280x800-projects.png`, `all-820x600-history-recorded-check.png`, `all-820x600-capture-mobile.png`.
3. **The task workspaces are comparatively calm and honest.** Progress counts include failed processed targets; unknown-duration discovery does not invent a percentage; protection notes explain what cancellation preserves. Gap details name specific failures and recovery steps. See `task-820x600-capture-progress.png`, `task-820x600-capture-gap-details.png`, `task-1280x800-setup-download.png` and the complete task sheets.

## Priority issues

### 1. [P1] Review is not sized for inspection at the minimum desktop window

**What:** At 820×600, the 192px global rail, 200px result queue and fixed surrounding chrome leave before/after panes roughly **178px wide**, with Fit width at **12%** for a 1440px desktop capture. The ordinary review shows only about two complete queue rows plus a partial third. The displayed website text is microscopic. In the 148-target frame, the queue retains those same limits, and full same-origin URL prefixes erase the distinguishing path. At 1280, 26% originals are more useful, but Analysis wraps to another toolbar line while two decision areas and repeated prose remain underneath.

**Why it matters:** The primary job is to inspect changes, not merely display images correctly. Zoom/pan exists, but the starting view asks the PM to discover and repeatedly operate it before seeing small content changes. Hiding the queue improves the narrow default only to about 20%. Details/Analysis/filter flyouts obscure most or all of the narrow evidence region. This is a geometric/usability defect, **not evidence of a Blade rendering failure**. Correctly drawn unreadable evidence is still bad product design.

**Evidence:** `all-820x600-review.png`, `all-1280x800-review.png`, `all-820x600-review-mixed-148.png`, `all-1280x800-review-mixed-148.png`, `all-820x600-review-queue-hidden.png`, `all-820x600-review-analysis.png`, `all-820x600-review-ignored-area.png`, `all-820x600-review-filters.png`. The 148-target fixture supports presentation geometry only.

**Source explanation:** `crates/app/src/workflow.rs:303–352` fixes fit/queue sizing; `workflow.rs:488–490` selects the narrow arrangement; `workflow.rs:662–706` keeps queue/viewer and the ordinary decision bar mounted. `crates/app/src/main.rs:5926–5998` builds the toolbar and removes the manipulation hint at narrow width. `main.rs:3959–4085` builds the queue chrome. `crates/app/src/review_ux.rs:660–715` keeps five review actions visible.

**Fix, preserving functionality:**
- Provide a genuine **evidence-focus desktop layout** at small widths: a collapsible global project rail and a locally collapsible queue, without removing any navigation destination or the side-by-side Compare mode.
- Offer a clearly labeled full-width before/after switch using the same target, zoom and pan position as an alternative to narrow side-by-side inspection. Keep Difference/Overlay/Heatmap available and explicitly named; do not silently replace evidence modes.
- Make queue labels **path-first**, with origin secondary and the complete URL still retrievable/openable. Preserve profile and concise classification; do not spend the status line repeating a caution that is already elsewhere.
- Keep review state and baseline-selection state distinct, but compact their resting chrome; group the three decision states, make Next unreviewed a clear separate advance action, and disclose notes/technical explanations when needed.
- Keep a concise manipulation hint or visible help affordance at narrow width. Do not solve the problem by reducing application text size.

**Suggested command:** `/impeccable adapt`, then `/impeccable layout`.

### 2. [P1] Baseline consent is a provenance browser rather than a clear decision

**What:** The confirmation dialog leads with “Confirm selected reference changes” and “selected references replaced / inherited references kept.” Its limited height then splits attention between a virtualized reference list and a separate 100px exact-provenance scroller. Even the one-selected-screenshot example shows raw run UUIDs and original-artifact text while rows/lines terminate at scroll boundaries. At narrow width, much of the available body is occupied by technical data rather than a comprehensible update summary.

**Why it matters:** This is the consequential moment: the PM is changing the future reference, not merely acknowledging an alert. The dialog technically exposes provenance but does not make the decision emotionally or linguistically easy. “1 selected · 1 eligible” is not a substitute for naming the screenshot and describing what becomes the new baseline. Exposing a database ledger does not equal informed consent. This is information architecture and scroll geometry, not proof of overlapping renderer text or broken approval safety.

**Evidence:** `all-1280x800-review-approval-confirm.png`, `all-820x600-review-approval-confirm.png`, with selection states on contact sheets `all-1280-07.png` / `all-820-07.png`.

**Source explanation:** `crates/app/src/workflow.rs:925–972` constructs exact provenance, the fixed 420px dialog, 100px rows and separate 100px provenance scroller. `workflow.rs:1109–1165` presents selection/eligibility and the approval action. The source-backed safety checks are valuable, but their existence does not prove comprehension.

**Fix, preserving functionality:** Use a task-facing title such as **“Update the baseline for 1 screenshot?”** with correct singular/plural handling. Lead with path, screen size, environment and readable capture time for the selected replacements; summarize unchanged screenshots separately. Explain in one plain sentence that future checks use these chosen screenshots and older versions remain in History. Put UUIDs, hashes, exact requested/final URLs and source-run provenance under a labeled Technical provenance disclosure, still fully available. Use one clearly bounded reading flow instead of two tiny adjacent scrolling regions; keep the selected inspected row unmistakable. Keep Keep reviewing/Escape and explicit Confirm baseline; retain all eligibility and stale-preview protections.

**Suggested command:** `/impeccable clarify`, then `/impeccable layout`.

### 3. [P1] Setup still makes operating settings more prominent than advancing safely

**What:** Website setup has a useful Save website & continue, but Access and Pages split Save settings from Continue/Find pages. Save remains the filled primary even in the pictured clean, saved public-access state. At 820×600 the No login required choice is only partly visible at the bottom of the first Access viewport; the forward control is secondary. The Pages & sizes stage disappears from the stage strip while on Access. Lower login settings offer several equally weighted actions plus repeated baseline safety prose. The minimum-width mapped-URL preview spends most of its height explaining scope before revealing its first source URL.

**Why it matters:** First-time PMs should make a small sequence of decisions, not infer a save-state machine and browser/session protocol. The extra explanation is not helping if it hides the decisive control or the evidence the disclosure promises. Returning users editing an existing project need settings, but onboarding should not inherit the full settings-panel hierarchy. This is workflow/scroll hierarchy, not a claim that settings fail to persist.

**Evidence:** `all-820x600-website.png`, `all-820x600-access-public.png`, `all-1280x800-access-login.png`, `all-820x600-access-login-actions.png`, `all-820x600-pages-screen-sizes.png`, `all-820x600-project-settings-bottom.png`, `all-820x600-mapping-preview.png`. Other setup stages were inspected on all-01 through all-04 sheets at both sizes.

**Source explanation:** `crates/app/src/workflow.rs:1176–1223` changes the visible stage strip; `workflow.rs:1250–1295` always renders Save as primary and continuation as secondary. `crates/app/src/main.rs:3780–3786` includes the login-retention/“Ephemeral” explanation; `main.rs:3720` uses “Replace raster images.”

**Fix, preserving functionality:**
- In guided setup, expose one state-appropriate next action: Continue when saved, Save & continue for changed ordinary settings, and Save & open login browser when login must be established. Make the website visit explicit in that label; never navigate merely because the form is saved.
- Keep all three named stages visible with current/completed/unavailable states instead of making the future stage disappear.
- Put the public/private access decision and its next action in the first minimum-size viewport; disclose session retention, mapping, language and website theme only when relevant.
- Keep full environment controls in the existing-project settings route. Place the browser status beside its immediate action and disable irrelevant controls with a nearby reason.
- Use “replace photos and other bitmap images” with concise limitations instead of requiring the default audience to know “raster”; use the visible “Only for this app session” label in help instead of switching to “Ephemeral.”
- In mapping preview, show at least one labeled source→capture URL pair before extended scope/probe notes. Preserve exact mappings, separate login sessions and representative-probe limitations.

**Suggested command:** `/impeccable onboard`, then `/impeccable clarify`.

### 4. [P2] “Hide page queue” is detached from the panel it controls

**What:** The user’s complaint is correct. Hide page queue is in the **top-right** workspace action group beside Run check and Refresh, while the queue itself is at the **left** of the working area. The same mismatch occurs in capture inspection. Show page queue stays in that global group after the queue disappears.

**Why it matters:** Spatial ownership is an affordance. This is not a whole-workspace data action like Run check, and its location makes users translate a right-side global button into a left-side layout change. It wastes attention exactly when they are trying to reclaim evidence space. The toggle is visibly functional in the provided hidden-queue frame; the issue is locality, not renderer correctness or alleged absence of the capability.

**Evidence:** `all-1280x800-review.png`, `all-820x600-review.png`, `all-820x600-review-queue-hidden.png`, `all-1280x800-capture-mobile.png`, `all-820x600-capture-mobile.png`.

**Source explanation:** `crates/app/src/workflow.rs:533–597` adds the queue toggle to the shared actions group; `workflow.rs:615–629` places that group opposite the title. `workflow.rs:355–364` actually toggles the queue and refits the image.

**Fix, preserving functionality:** Move Hide pages/Hide results into the relevant queue heading beside its title, with a clear local scope. When collapsed, leave a labeled restoration affordance at the queue/viewer’s **left edge** or viewer heading, not beside Run check. Preserve collapse state, zoom-refit behavior and keyboard focus restoration; qualify actual input separately. A visible text label is preferable to an unexplained chevron alone.

**Suggested command:** `/impeccable layout`.

### 5. [P2] History, sharing and cleanup expose a storage ledger at PM decision points

**What:** History’s basic sections are sensible, but the search filters only the loaded page; actions sit above selection while useful record details may be below the fold. The deletion preview literally renders `Run(RunId(...))`. Storage shows human units and enormous raw-byte values together. Import/export combine targets, original PNGs, mask configurations and provenance caveats into paragraphs. Integrity cleanup exposes destructive actions even when every displayed count is zero, and deletion of recovery objects uses ordinary secondary styling.

**Why it matters:** A PM wants to know which check or baseline they are opening, sharing or deleting, what is kept, and whether screenshots expose private information. Raw Rust representations and object counts make the user translate implementation details into consequences. Honest loaded-page search is preferable to a false global search, but this remains a weak high-volume archive workflow. Deduplicated fixture PNG counts are not erroneous; their unexplained relationship to screenshot targets is a copy problem, not a capture defect.

**Evidence:** `all-1280x800-history-runs.png`, `all-820x600-history-baselines.png`, `all-820x600-history-storage.png`, `all-820x600-delete-preview.png`, `all-820x600-export-preview.png`, `all-1280x800-import-preview.png`, `all-820x600-integrity-preview.png`.

**Source explanation:** `crates/app/src/workspace_history.rs:245–255` states loaded-page-only filtering; `workspace_history.rs:285–350` places baseline actions/list identity. `crates/app/src/main.rs:2261–2285` mixes consequence copy and exact storage counts; `main.rs:2277` formats deletion identity with `{:?}` and `main.rs:2285` styles recovery deletion as Secondary.

**Fix, preserving functionality:** Label the current capability “Filter this history page” and keep page scope/paging near the filter; do not pretend it searches the archive. Put readable date/type/environment and screenshot coverage on selected record previews and deletion confirmations, with IDs in Technical details. Separate a short What is shared/What is never shared summary from expanded exact counts/provenance; keep the non-redaction warning prominent. Display human units by default, exact bytes on demand. Attach the selected record identity and Keep/Delete consequences to the preview. Disable zero-impact cleanup actions; give irreversible recovery-object deletion the shared danger treatment. Keep deliberate cleanup, shared-object protection, old baselines and every privacy safeguard.

**Suggested command:** `/impeccable clarify`, then `/impeccable harden`.

## Cognitive load

**Checklist failures: 6/8 in the core review/decision workflow — high extraneous load.** This is not a claim that every screen has high load: the focused task workspaces are substantially calmer. Counting visible controls alone is not a universal usability law; here the failure is equal emphasis, weak scope and the need to reconcile multiple decisions while inspecting tiny evidence.

| Checklist item | Result | Specific evidence |
|---|---|---|
| Single focus | Fail | Evidence, capture-running actions, analysis modes, review classification, notes and baseline selection compete in the resting review. |
| Chunking | Pass, with exceptions | Setup choices and task outcomes generally have sensible named groups of two to four; approval’s split scrolling is a notable weak group. |
| Grouping | Fail | Queue collapse is remote from its queue; consent data has two tiny scrolling owners; history/deletion selection identity is not attached strongly enough to the consequential action. |
| Visual hierarchy | Fail | Clean Access still emphasizes Save over Continue; the review’s equal-weight action rows outrank the actual screenshot detail. |
| One thing at a time | Fail | Users reconcile visual classification, review decision, note state, eligibility and baseline selection without a sufficiently clear order. These are distinct necessary concepts, not one interchangeable “approve.” |
| Minimal choices | Fail | Seven wide viewer actions, five review actions and a dense lower-access action cluster are presented with little hierarchy. |
| Working memory | Fail | Same-origin truncated rows lose page identity; before/after environments disappear from narrow captions; a scrolled history/deletion preview forces users to reconstruct the chosen record. |
| Progressive disclosure | Pass, with exceptions | Advanced settings, options, Details and Analysis exist. Disclosure is useful, but at minimum width it often hides the evidence or the first mapping it was meant to clarify. |

### Concrete decision points with more than four visible actions

1. **Wide viewer toolbar: 7** — Compare, Difference, Overlay, Heatmap, Zoom, Details, Analysis. The four modes are a natural subgroup; the other three should not look like additional modes. `all-1280x800-review.png`.
2. **Ordinary review action row: 5** — Mark intentional, Mark needs fix, Mark unreviewed, Next unreviewed, Review note. Three state choices, one navigation action and one writing tool currently read as peers. `all-1280x800-review.png`; narrow equivalents in `all-820x600-review.png`.
3. **Lower access actions: 6** — Open login browser, I’m ready, Close login browser, Forget login, Preview mapped URLs, Probe representative pages. Disabled states reduce immediately actionable choices, but the cluster still demands explanation and translation. `all-820x600-project-settings-bottom.png`, `all-820x600-access-login-actions.png`.
4. **Support/cleanup cluster: at least 5 in the captured lower area** — confirm temporary cleanup, confirm recovery deletion, cancel preview, choose folder, About, with keyboard help immediately below. Destructive and informational actions share the same region. `all-820x600-integrity-preview.png`.

**Action points:** group modes separately from tools; separate decision states from advance/note actions; disclose browser maintenance and mapping beneath one next action; attach queue controls to the queue; reduce approval to one consequence-first reading flow; retain page/record identity throughout decisions. Do not delete capabilities to make the action count smaller.

## Emotional journey

- **Arrival:** calm and credible. The blue rail and explicit project selection reduce fear of acting on the wrong project. The populated four-project list is understandable, although similarly named Northline projects expose how much the truncated rail depends on the top context.
- **Setup valley:** reassuring prose accumulates into procedural work. Save, continue, login-ready, environment mapping and consent are all individually sensible; together they feel like configuring infrastructure. The PM wonders which controls are mandatory and whether clicking Save moves them forward.
- **Discovery:** a useful sense of control emerges: actual visited/included counts, searchable pages and explicit exclusions. Minimum-width Page options replaces breadth with a small scrollable reading space; the estimate is below the initial options viewport while Dismiss page list remains pinned.
- **Capture:** probably the strongest emotional section. Progress and explicit preservation notes prevent panic. Sparse desktop space is acceptable here because waiting is the task; no decorative activity log or invented ETA is needed.
- **Review valley / task peak:** real website images make the product feel real, but narrow evidence is too small, page identity is repetitive, and the analyst-facing controls demand attention. A smooth renderer cannot make this cognitively smooth.
- **Baseline peak:** deliberate confirmation correctly introduces friction, but that friction is technical rather than decision-supportive. The PM should feel “I know exactly what future checks will use,” not “I read the reference ledger.”
- **End:** check completion and saved-ZIP states give useful closure, but they mainly hand the user to another workspace. A paused capture points toward resumption while the button says Review check results. Finished/failed states need an action label matching the user’s next intention, plus concise reviewed/remaining/needs-fix/updated coverage where applicable. Do not turn check completion into an unearned “release passed.”

## Per-screen critique — all whole-app states covered

State names in the table map directly to `all-{1280x800|820x600}-{state}.png`. Every listed state was visually covered at **both** sizes. Full-resolution reads are separately enumerated below; contact-sheet coverage is not claimed as full-resolution inspection of all 182 originals.

| Screen / captured states | What works | Specific critique / recommended change |
|---|---|---|
| Projects: `projects`, `project-chooser` | Four real local mock projects; selected words/paint; View results versus Edit settings; destination-preserving chooser. | Ordinary row actions are clear, but rows reveal little about readiness or last check. Long selected project names truncate in the rail; retain an easy full identity, especially for similar-name projects. |
| Projects states: `projects-empty`, `projects-loading`, `projects-failed` | Empty/load/failure are separate, and retry/create/import routes are explicit. | Good foundations. The error is technically detailed; prioritize a plain reason and concrete retry, keep raw diagnostics optional. No native recovery pass inferred. |
| Website: `new-website`, `website`, `website-validation` | Explicit URL/save behavior, staged flow, correction summary and Go to next correction. | At minimum height, headings/step strip push address guidance or the error field below the fold. The summary’s reveal action is useful; keep the affected input and concise correction visibly connected. Space-separated URL lists are less natural than a clearly described list input. |
| Public access: `access-public` | Plain No login required; saved environment and explicit continuation. | Forward step is secondary, Save remains primary in a clean state, and minimum viewport cuts the access choice. Optimize for the public-site path first. |
| Private access: `access-login`, `access-login-actions` | Session retention is explicit; browser readiness cannot be silently assumed. | Long procedural form; draft versus saved environment copy can conflict visually; “Ephemeral” switches vocabulary from the plain option. Co-locate browser state, one next action and retention choice. |
| Access extensions: `access-options`, `mapping-preview`, `timezone-popup` | Environment mapping, website language/theme and timezone remain available; mapping disclosure has its own title/Close. | Narrow mapping viewport spends its first space on scope and probe status rather than an actual pair. Explain optional controls in PM language; make first mapping visible without a second scroll. Popup pixels alone cannot qualify picker keyboard behavior. |
| Pages setup: `pages-sizes`, `pages-screen-sizes` | Defaults and three website sizes; mobile/tablet correctly described as emulation; full-page versus visible-screen meaning. | Useful choices, but a long page of sections obscures the guided order and exact current configuration. Keep a compact current capture plan with explicit save/next state, without hiding available profiles. |
| Capture policy: `pages-images`, `pages-speed` | Image-mode consequence is stated before capture; speed presets explain concurrency; robots override is deliberate. | “Raster” and millisecond spacing are implementation-facing. Keep preset names/default first, exact timing in detail. Permission and read-only-account warnings matter; avoid surrounding them with repeated generic caution. |
| Advanced capture: `pages-advanced` | Technical controls are separate; ranges and defaults are explained. | Dense labels such as global/per-website parallelism and exponential backoff assume developer knowledge. Preserve the advanced controls, group bounds/speed/retry and explain consequences rather than mechanisms. |
| Existing-project settings: `project-settings`, `project-settings-bottom` | Explicit saves protect current baseline; all environment/capture functions preserved. | Long scrolling settings still blur capture setup, access and browser maintenance. Lower actions are a wall of buttons. Organize by the future task affected; keep draft state and current project visible. |
| Fixed inventory: `pages-baseline` | Recorded page list is distinct from discovery; saved reference coverage and replacement route are explicit. | Make “fixed for future checks” and “discover new pages only by explicit replacement workflow” immediately comprehensible. Do not let Page list/Find pages suggest the saved baseline grows automatically. |
| Discovery: `discovery` | 74-page list fills available space; search/count/inclusion are useful; limited coverage is disclosed. | Limited coverage needs a nearby concrete cause/next step rather than a generic caution. Path-first labels matter more than repeated origins at this scale. |
| Discovery options: `discovery-options` | Selected page, search matches and estimates are grouped; Close and Dismiss are attached. | 820px leaves a narrow page list and shows only initial options; capture estimates are below the fold, while dismissal takes a fixed footer. Put included-target estimate and relevant limit cause near Capture pages. Do not pretend scroll clipping is renderer failure. |
| Discovery recovery: `discovery-no-matches` | Clear filter recovery; no silent inventory mutation. | Good direction. Preserve selected-page context when filtering it out, and attach inclusion actions to a named selected target. |
| Ordinary review: `review`, `review-mobile` | Untinted original pair, labeled before/after, profile, count summary, review classification and explicit baseline selection. | Narrow desktop evidence is too small; status/path are truncated; wide toolbar spills Analysis onto a second row. Mobile images are easier to read, but the application should not optimize only for mobile website screenshots. |
| Scale review: `review-mixed-148` | Changed/unchanged/gap totals and limited-inventory warning demonstrate a truthful presentation vocabulary. | Same two/three visible narrow rows and repeated URLs do not demonstrate workable 100–1,000-page triage. A fixture with repeated imagery cannot demonstrate throughput or actual high-volume approval. |
| Failed comparison target: `review-failed-target` | No-image state and disabled baseline update prevent “missing means unchanged.” | The main pane tells the user to choose another target instead of diagnosing/retrying this one; the visible Analysis button is a weak owner for failure recovery. Show failed path/profile, reason and specific safe recovery route locally. |
| Queue collapse: `review-queue-hidden` | Evidence expands and restoration remains visible. | Toggle ownership is wrong; the remaining 20% fit is still not a satisfactory small-detail inspection view. Keep a local restoration affordance. |
| Analysis views: `review-difference`, `review-overlay`, `review-heatmap` | Wide labeled modes and difference evidence are purposeful, not decorative tinting. | Narrow Overlay/Heatmap captures show Compare rather than proving those modes executed. Source places those actions in Details at narrow width. Treat this as a **coverage limitation**, not a proven renderer bug; make mode discovery/location stable. Explain the color meaning without relying solely on color. |
| Evidence disclosure: `review-details`, `review-diagnostics` | Plain capture details precede opt-in technical data; close stays attached. | The flyout covers after evidence on wide and nearly all evidence on narrow. It should help a decision without making users reconstruct the comparison; minimize repeated capture facts. |
| Comparison settings: `review-analysis`, `review-analysis-advanced` | Sensitivity and ignored areas are scoped to analysis; drawings do not mutate originals. | Narrow opening mainly shows ignored-area tools and requires scrolling for sensitivity. Order the default settings by the likely task; do not require a large overlaid panel for every adjustment. |
| Ignored areas: `review-ignored-area` | Coordinate alternative exists and unsaved geometry prevents inappropriate review/update. | At narrow size only X/Y fit initially; width/height/apply/save need another scroll while screenshots are obscured. Keep selected region, complete geometry, save state and preview legible together; native keyboard/drawing success remains unverified. |
| Notes: `review-note` | Explicit note saving and current-analysis context are useful. | Note expansion consumes evidence height, and the warning/action hierarchy must clearly distinguish saving text from selecting an update. Maintain drafts when closing/filtering. |
| Result filtering: `review-filters`, `review-no-matches` | Filtering preserves inspected evidence and drafts; no-match text explains recovery. | All is already a natural default; large choice rows make only All/Changed initially visible at narrow size. Keep Changed/Gaps immediately discoverable, attach filter ownership to the queue, retain truthful outside-filter notice. |
| Approval: `review-approval-selection`, `review-approval-confirm` | Selection is explicitly separate from review state; retention and inherited references are disclosed. | Consequence-first consent, clear target identity and one readable flow are missing. “Eligible” counts need explicit scope; singular grammar and references jargon are avoidable. |
| Zoom: `review-zoom-menu` | Labeled fit/actual-size/presets are stronger than unexplained icon buttons. | Narrow resting label shows Zoom/12% without Fit width; the selected mode is disclosed only after opening. Keep Fit/Actual/percentage meaning visible and a discoverable manipulation hint. Do not infer native menu accessibility from its drawing. |
| Incomplete capture: `capture-gaps`, `capture-failed` | Incomplete evidence is not saved automatically as a baseline; gaps and next action are present. | “Filter → Gaps, then Find pages to retry” is an indirect diagnosis/recovery chain. Attach a named gap/recovery action and explain whether the next operation rediscovers/replaces rather than only retries. Never imply backend targeted retry if it is not implemented. |
| Ready/saved capture: `capture-ready`, `capture-mobile`, `capture-saved` | One original; explicit Save baseline versus saved reference; mobile-web dimensions. | Ready capture title “Inspect baseline” precedes actual saving and slightly muddies capture/reference status. Wide mobile Fit width upscales to 194% and shows mostly the page top; prefer an initial actual-size-capped overview with enlargement still available. |
| Evidence loading/recovery: `capture-empty`, `capture-loading`, `capture-read-failed` | Separate authoritative empty, pending read and failed read; no invented missing baseline before data loads. | Good status design. Make failed read identify the evidence/read and safest retry concisely, not raw diagnostics first. These frames do not establish recovery behavior. |
| Runs/history: `history-runs`, `history-bottom`, `history-empty`, `history-no-matches` | Capture/check/preview distinctions; useful read-only opening and honest search scope. | List height shows only two run rows in the wide populated frame; selected detail and pagination are separated by scrolling. Filter this page is honest but weak at archive scale. Search-empty must not mean “no saved history.” |
| Baselines/storage: `history-baselines`, `history-storage`, `history-provenance` | Active versus retained, composite provenance, storage accounting and manual cleanup are preserved. | Selected baseline details are largely below the narrow fold. Human version/date/coverage should dominate IDs; raw bytes, shared/exclusive objects and recovery terms belong in details. |
| Recorded inspection: `history-recorded-check`, `history-recorded-capture` | Inspection-only banner and active-baseline-unchanged framing protect against accidental activation. | Banner and repeated cautions further reduce evidence; the narrow recorded comparison is especially tiny. Use compact persistent read-only context, preserve the original environment/policy/masks and obvious Latest evidence exit. |
| Sharing/deletion: `export-preview`, `import-preview`, `delete-preview` | No sessions exported; originals/privacy, recipient login and irreversible deletion are explicit. | Dense warning paragraphs and `Run(RunId(...))` do not support PM consent. Keep the exact safety content, reorganize it around readable record identity and clear consequences. |
| App settings/network: `app-settings`, `app-settings-network` | Honest browser readiness/build limitation; explicit install/repair/network saving. | Browser setup unavailable is a development-build fact, not an inferred UI P0. It needs a product-facing explanation/action path. Long local paths and CA/Node/proxy details crowd ordinary settings; reserve them for optional managed-network/support work. |
| Support: `app-settings-diagnostics`, `diagnostic-preview`, `integrity-preview`, `keyboard-help`, `about` | Explicit local diagnostic export, temporary-vs-valid-recovery distinction, visible shortcut help and preview qualification. | Advanced support reads like an engineering console; zero-count cleanup still offers confirmation and dangerous recovery deletion lacks danger styling. Keep support capability, but do not make routine users parse integrity/storage internals. Keyboard dialog pixels do not certify shortcuts. |

## Task screens — all 20 states, both desktop sizes

State names map to `task-{1280x800|820x600}-{state}.png`. These are the older labeled synthetic task fixtures, not the rich website-image cohort.

| Task states | Assessment |
|---|---|
| `baseline-finishing` | Clear committed save and “cancellation cannot undo” notice. Correctly not pretending a committed database operation is cancellable; static frame cannot prove the commit guard. |
| `capture-progress`, `capture-pausing` | Real processed-target arithmetic and saved/failed/remaining counts communicate progress well. Stop check versus Pause capture mixes operation vocabulary, and narrow safety note crosses the body scroll boundary. Keep one operation identity with the stage named separately. |
| `capture-paused` | Honest paused title and retained screenshot note. Copy says return to resume, but the primary says Review check results: label the actual next intention, and expose Resume where implemented. This is an indirect recovery route, not proof the capture cannot resume. |
| `capture-attention`, `capture-complete` | Captured/failed counts and baseline-unchanged reminder are appropriate; a completed capture is not an approved baseline or clean release. End action should support unresolved gaps or review first, without claiming an all-clear. |
| `capture-gap-details`, `capture-setup-details` | Specific page/profile/reason/attempts and next-step recovery are useful. Details scrolls the operation identity away; preserve a compact context header. Failure links open the default browser, not necessarily the saved authenticated session; explain that limitation at the recovery point. |
| `discovery-browser-check`, `discovery-crawl-rules`, `discovery-first-page`, `discovery-first-retry` | Honest indeterminate preparation and retry status, no fabricated pages/percent. “Pages are counted…” repeats below every early status; shorten it after the user understands the stage. robots.txt belongs in optional detail beside a plain explanation. |
| `discovery`, `discovery-stopping`, `partial-discovery` | Actual pages visited, stop acknowledgment and incomplete-inventory outcome are distinguishable. Partial outcome correctly routes to reviewing included pages. Cause/limit and what to do next should stay beside the incomplete state. |
| `setup-download` | Known download amount and explicit Cancel setup are good. “Step progress: 78 / 246” repeats the MiB data but loses its unit; one labeled download amount/fraction is enough. No actual installer qualification inferred. |
| `failed-task` | Work preservation, red failure and concrete storage troubleshooting in Details. “Could not finish / The operation” loses the task name. Use an operation-specific heading and a named recovery route; retain exact diagnostic in Details. Source: `crates/app/src/task_ux.rs:261–276`. |
| `zip-export-stopping`, `export-late-success`, `zip-import` | Clear stopping semantics, kept-file caveat, explicit import cancellation, safe new-project import and privacy reminder. Export success should name the saved destination and offer an explicit open-folder route if available; never imply anything was uploaded. |

## Persona red flags

### Jordan — first-time, somewhat tech-savvy PM

Jordan can choose a project and understand Create project. They stumble on **Access’s filled Save versus secondary Continue**, the stage that disappears from the strip, “Ephemeral” in retention help and “Replace raster images.” The lower access cluster asks them to infer which browser/session action is mandatory. On first review, Changed is correctly not a defect, but Mark intentional and Select for baseline are separate decisions whose order is not obvious. Confirmation then switches to “selected references / inherited references / composite capture times” and UUIDs. The red flag is not lack of intelligence: the tool is teaching its internal model instead of supporting the PM’s decision.

### Alex — frequent high-volume reviewer

Alex gets useful search, grouping, Next unreviewed, zoom and visible keyboard help. They still face **two/three queue rows with nearly identical URL prefixes at narrow width**, 12% evidence, two different “Next” concepts, a remote collapse control and baseline selection separate from classification. Efficient selection/review of 100–1,000 pages is not established by the 148-target presentation fixture; no visible bulk-review or bulk-baseline-selection path was demonstrated in the inspected resting workspace. History search is loaded-page-only and the run list is small. Do not report “no shortcuts”: shortcuts are documented and visible, but native execution was not tested.

### Sam — low vision, keyboard or screen-reader dependent desktop user

Sam’s visually evident problem is **shrunk website evidence and truncated page identity**, particularly when before/after environments disappear from narrow captions. Existing text labels, a visible focus outline in a collapsed-queue frame, shortcut help and coordinate-entry alternative are encouraging. They do not establish complete keyboard traversal, native accessible naming, announcement of status changes, focus trapping/restoration, OS scaling or high-contrast behavior. The flyout with just X/Y initially visible is a serious keyboard/cognitive navigation risk, but it is not a proven inaccessible-control finding. Test the actual GPUI app with NVDA/VoiceOver and supported Linux AT plus keyboard-only input; never import web ARIA assertions into this native review.

## Minor observations

- **Fit width is not always an appropriate starting zoom.** `capture-mobile` at 1280 upscales a 390px original to 194%, crops the page heavily and exposes softened enlargement. This is a zoom policy/image-scaling choice, not proof of a corrupt original. Cap the initial overview at actual size or provide an explicit alternate view; keep enlargement possible.
- **Narrow mode naming is weaker.** Diff abbreviates Difference; Zoom/12% omits Fit width in the resting label; Overlay/Heatmap move into Details. Do not save pixels by erasing state meaning.
- **“12 of 12 screenshots shown” includes a missing target in a gaps state.** The list is of screenshot targets/results, not necessarily twelve existing images. Prefer “12 targets shown” or a clearly contextual result count. The underlying captured/gap count is already separate and truthful.
- **“1 gaps” and “1 selected references” are avoidable grammar defects.** These are polish, but they occur at meaningful trust points rather than decorative copy.
- **A failed target’s recovery should identify that target.** The main pane’s “select a captured target” protects honesty but sidesteps the failure; keep exact path/profile/reason and a safe retry/login/check route where relevant.
- **The application can be over-reassuring.** “Saving never changes baseline,” “baseline unchanged,” “not a saved baseline,” “not a defect,” “not redaction,” and “not automatically deleted” are individually important. Repetition makes users skim them all. Keep the one currently relevant consequence in place and the rest in contextual detail.
- **Destructive support actions need the same visual grammar as ordinary deletion.** A recovery-object deletion should not look like Choose folder. Zero-impact confirmations add friction without protection.
- **History inspection earns a banner, not a second onboarding block.** Compact read-only state should remain persistent without further starving evidence.
- **Task recovery labels should match intent.** Paused→Review check results and generic failed→Return to workspace unnecessarily make the user translate their next action. Source: `task_ux.rs:199–238`, `task_ux.rs:261–276`.
- **Do not treat compact lists and properly clipped scroll regions as render corruption.** A row/line ending at the viewport boundary is normal bounded scrolling. Critique whether the reading area is adequate and the scroll owner is clear.
- **Do not claim the narrow Overlay/Heatmap frames validate those modes.** They visibly show Compare. That is a coverage limitation requiring clarification in the parent evidence report, not an independently proven app bug.

## Coverage inventory

### All contact sheets read

`contact-sheets/all-1280-01.png` through `all-1280-12.png`; `contact-sheets/all-820-01.png` through `all-820-12.png`; `contact-sheets/task-1280-01.png` through `task-1280-04.png`; `contact-sheets/task-820-01.png` through `task-820-04.png`. **32/32.**

### Full-resolution originals read — 44

All paths below are under the evidence root.

- `all-1280x800-review.png`
- `all-820x600-review.png`
- `all-820x600-review-mixed-148.png`
- `all-1280x800-review-mixed-148.png`
- `all-820x600-discovery-options.png`
- `all-1280x800-discovery-options.png`
- `all-1280x800-capture-mobile.png`
- `all-820x600-capture-mobile.png`
- `all-820x600-review-approval-confirm.png`
- `all-1280x800-review-approval-confirm.png`
- `all-820x600-website.png`
- `all-820x600-access-login-actions.png`
- `all-1280x800-access-login.png`
- `all-820x600-pages-screen-sizes.png`
- `all-1280x800-pages-images.png`
- `all-820x600-project-settings-bottom.png`
- `all-1280x800-history-runs.png`
- `all-820x600-history-baselines.png`
- `all-820x600-export-preview.png`
- `all-1280x800-import-preview.png`
- `all-820x600-delete-preview.png`
- `all-820x600-history-storage.png`
- `all-1280x800-projects.png`
- `all-820x600-access-public.png`
- `all-820x600-mapping-preview.png`
- `all-820x600-pages-advanced.png`
- `task-820x600-capture-paused.png`
- `task-820x600-failed-task.png`
- `all-820x600-review-ignored-area.png`
- `all-820x600-review-analysis.png`
- `all-1280x800-review-difference.png`
- `all-820x600-history-recorded-check.png`
- `all-1280x800-app-settings.png`
- `all-820x600-integrity-preview.png`
- `task-820x600-capture-progress.png`
- `task-820x600-capture-gap-details.png`
- `task-1280x800-setup-download.png`
- `all-820x600-review-failed-target.png`
- `all-820x600-capture-gaps.png`
- `all-820x600-review-filters.png`
- `all-820x600-review-no-matches.png`
- `all-820x600-website-validation.png`
- `all-820x600-review-queue-hidden.png`
- `all-1280x800-review-details.png`

## Provocative questions for synthesis

1. If the purpose is visual release review, why does the minimum desktop window reserve more reliable space for controls than for the details being judged?
2. Is the baseline dialog asking the PM to make a decision, or asking them to verify the storage model? What exact plain sentence should they be able to repeat before Confirm baseline?
3. Why does a clean public-site setup make Save more prominent than its next safe step? Could the existing save/visit safety constraints be represented as action state instead of additional instructions?
4. At 500 pages, what proves progress: a changed-image count, a reviewed-target count, or a defensible list of unresolved decisions and coverage gaps? The interface currently asks users to reconcile all three.
5. Why should a PM understand shared objects and a Rust `RunId` in order to delete one old check safely?

### Targeted choices the parent can ask after synthesis

- **First priority:** evidence/queue geometry; guided setup; consequence-first approval/history copy.
- **Scope:** top three workflow issues; all five priorities; locality/copy-only refinements before a broader layout pass.
- **Narrow-window policy:** preserve permanent side-by-side with compact collapsible chrome; add an explicit full-width before/after alternative while retaining side-by-side; require a larger review window only if the product’s minimum-window contract is deliberately changed.

No edits are authorized by these questions or recommendations. Final parent synthesis should keep the report and its questions together; this subassessment is not a substitute for Assessment B or native qualification.
