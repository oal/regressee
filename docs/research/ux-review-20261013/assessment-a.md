# Independent Assessment A — whole-app native UX review

Date: 2026-10-13. Method: isolated Assessment A, native source plus fresh Linux GPUI/Blade offscreen pixels. Review/capture only; no production edits, commits, pushes, website visits or real user project access. Assessment B, previous critiques and detector reports were not consulted.

## Authority and evidence boundary

- Preserve the approved **Clear wayfinding** identity: deep-blue rail, bright work surfaces, sky selection, readable desktop system typography, explicit safe actions. This is refinement, not redesign.
- Product mode: **Operate**. Native Rust/GPUI desktop, 1280×800 normal and 820×600 minimum logical windows. The skill context's `adaptive`/iOS/Android routing is a metadata mismatch; mobile guidance is not applied. Mobile/tablet below refer only to website capture profiles.
- Source commit: `5f136b7819c3663517d2f9bd5e17f7203e5b262e`; initial worktree clean.
- Fresh private root (0700): `/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-review-20261013-9g3qdv3j`.
- Source hashes and fixture provenance: `<root>/source-provenance.json`. Capture commands: `<root>/capture-commands.json`. Individual unmodified PNGs have matching `.json` SHA-256 metadata. Databases, website fixtures and pixels remain outside the repository.
- Personally inspected `<root>/renderer-proof.png` before creating `proof-reviewed.json`: legible real fonts including æøå, white field, blue primary action, selected radio, correct RGB calibration blocks and image-atlas sample. Proof note: `<root>/proof-review-note.md`.
- Website imagery is freshly generated, synthetic offline HTML/CSS imagery from `scripts/ui-review-fixtures.mjs`, using the already installed private Node/Playwright runtime; all page requests aborted. It is not actual customer evidence. **Every app screenshot is a fresh current-source native render**, not an HTML mockup.
- Offscreen captures establish development-host layout/paint only. They do **not** qualify OS window presentation, real keyboard/pointer/IME, accessibility/screen readers, motion, other OSes, high contrast or release readiness. No native accessibility pass is claimed.

## Source fingerprints

| File | SHA-256 |
|---|---|
| `crates/app/src/main.rs` | `b95ccd05cf4db2c368c2a8e9568ca484054a9701d1128b38ce530ac6ea87a59b` |
| `crates/app/src/workflow.rs` | `6e0b17533303cdc8c12c7b1b29ca19fd920fca2a5d749ee080aabf66cd4dc0bc` |
| `crates/app/src/workspace_history.rs` | `fcd461e5eb94dc6461e6c597a99e0fb671e2abd625de28d1db4285e6cba48da8` |
| `crates/app/src/options_ux.rs` | `8d63d4720a7643bdd72e3ab1d79ae3f1679185bbda05027379f83422fd20b970` |
| `crates/app/src/ui_renderer_capture.rs` | `d6da0a41b7887de459ce36f7ff2fffc62b25faf02ea30524e429001e283b1450` |
| `crates/app/src/ui_review_matrix.rs` | `9bc1b6e241bee10c51fed0b35a9e1b087b656fdfd11866bea3aa310f8855198e` |
| `crates/app/src/ui/controls.rs` | `0435e4ba22195bb18b2daef3f0f0238d601acbdc75ddc925baba2a38d62d21c5` |
| `crates/app/src/ui/theme.rs` | `68f6f1a3955c7bdb5dd5f684c2541d2c1ccd69553d3069209c0c81e0441debf4` |
| `crates/app/src/ui/timezone_select.rs` | `3709edfa37d4dcbfe45c76ba4e197c70062ea5c7a9213e6c18bec81e123cb277` |
| `crates/app/src/ui/zoom_select.rs` | `741eecf03fc0e635eb9639599fd34ae9815a912fdb92208024d37c523f013bba` |
| `crates/core/src/shell.rs` | `7ac600fbb9e8671d1fb6ba83f01be338987760fe5df2d6fb3620c8ccddce220b` |

## Outcome

**Keep Clear wayfinding; make setup read like a wizard and give the evidence more room.** The identity is coherent and appropriate for a serious desktop tool. The principal problems are task hierarchy, excess setup decisions, and presentation that consumes the space needed to inspect changes—not a need for a new visual world.

**Design health: 26/40, Acceptable.** Two P1 issues and three P2 priorities below; no P0 task blocker established by this assessment. This is Assessment A only, not the parent’s combined critique or a release certification. Scores are expert judgments from this evidence, not user-study measurements.

### Design specificity, judged independently

The workflow is genuinely Regressee-specific: fixed page lists, explicit saved references, paired website evidence, missing-capture cautions, per-target review, and separately confirmed baseline changes. An unrelated dashboard could not use that structure unchanged. The blue rail and rectangular controls are conventional, but convention is a strength in **Operate** mode. Preserve their quiet precision rather than adding expressive animation, mobile patterns, or decorative cards.

The missed opportunity is to let that product structure govern the hierarchy. Setup currently looks like a settings page with three state buttons; the review viewer is surrounded by more persistent chrome than its pixels deserve; development-stage copy makes the product feel provisional everywhere. Clear wayfinding should mean “where am I, what is needed now, and what happens next,” not merely blue selected controls.

## What is working

1. **A coherent, legible native desktop system.** Deep-blue rail, bright surfaces, restrained sky selection, visible text labels, and actual radio/check marks provide reliable visual recognition. The app does not use color alone for selection. Normal-window layouts have a calm typographic hierarchy and generous separation.
2. **Safety is attached to consequential actions.** The login footer promotes Save → Open login browser → I’m ready → Continue as appropriate; saving settings does not silently visit a website. Review disposition is distinct from baseline selection/confirmation. Export warns that ignored areas are not redaction; deletion states permanence and protects retained sources. Preserve these semantics while shortening repetition. Sources: `crates/app/src/workflow.rs:1393–1528`, `crates/app/src/main.rs:2381–2436`.
3. **Evidence is treated honestly.** Before/after originals are untinted, alternative render modes are optional, gaps are not labeled unchanged, and recorded evidence/history remains inspectable. Attached panels have explicit return/close actions. Task states expose progress, pause/stop, retained evidence and recovery rather than pretending that an interrupted job succeeded.

## Five priority issues

Severity: P1 = significant task difficulty/confusion; P2 = meaningful friction with a workaround. Phase numbers are recommendations for the parent, not implementation performed here.

### A1 — [P1] Setup is staged, but it does not read as a connected wizard

**Observed.** `new-website`, `access-public`, `access-login`, `pages-sizes`, and `pages-invalid`, at both sizes. The stage strip is three independent labels such as `Website · Current` and `Access · Unavailable`, without numbered progression. The current Pages button dims when the draft is unsaved/invalid. A first website sees `Approve all listed websites` and a multi-website explanation before any addresses exist. The public-access path also presents Production, Add staging environment, mapping options, mapped-URL preview and a representative probe alongside its basic login choice. The private path has a good footer sequence, but too much surrounding configuration competes with it.

**Impact.** A first-time operator must infer the sequence, distinguish navigation state from form validity, and decide whether staging/probing/mapping are prerequisites. The safe path exists, but its explanation competes with advanced work.

**Fix.** Retain the three stages and existing save/login gates. Present one connected numbered strip: `1 Website → 2 Access → 3 Pages & sizes`, with current/completed/locked treatment. The current location remains visually current even when Continue is unavailable. Give blocked future steps a short reason. Use one next action and a consistent Back action; preserve drafts and the explicit Save/login boundary. Reveal multi-origin approval only when it is relevant, without weakening authorization. Make the default Access decision simply “Do these pages need login?” Put environment creation, mapping, probe and rendering locale/theme under a clearly named optional disclosure. Keep necessary identity-provider/session controls accessible when login requires them. In Pages, show a concise plan summary before the next action and keep speed/scope customization grouped below essentials.

**Exact ownership.** `crates/app/src/workflow.rs:1312–1367` (stage presentation), `crates/app/src/main.rs:5146–5152` (stage enablement), `crates/app/src/main.rs:3947–3951` (always-visible website approval), `crates/app/src/main.rs:4048–4077` and `4080–4156` (Access choices, environments and actions), `crates/app/src/workflow.rs:1393–1528` (footer sequence). Do not replace backend permission gates with cosmetic availability.

**Phase / acceptance.** Phase 1. New public website and private-login setup should each have an obvious single next step at 1280×800 and 820×600; invalid Pages keeps its current-stage indication. Complete those paths without staging/mapping knowledge. Suggested commands: `/impeccable onboard`, `/impeccable clarify`, `/impeccable distill`.

### A2 — [P1] “More room for evidence” actions paradoxically reduce inspection height

**Observed and measured.** In `all-820x600-review`, each original viewport is **274×194 px**. Hiding Results (`review-queue-hidden`) expands width but adds a separate Show results row plus repeated target identity: the viewport becomes **380×102 px**. Full-width Before becomes **772×98 px**; most of the visible evidence is the website masthead. Opening navigation with the queue retained gives **178×141 px** originals and wraps Analysis onto a second toolbar row. These numbers are current painted geometry in `whole-app-matrix.json`, cross-checked against the actual full-resolution PNGs. They are not the preliminary 92 px estimate.

The View chooser is also detached from its trigger. At 1280×800 the trigger begins around **x500/y152**, while the panel begins around **x884/y268** over the After pane. It is anchored in code, but to viewer bounds rather than the activating control. Unlike the Zoom selector, it looks like a displaced stack of differently sized buttons.

**Impact.** Operators who hide a list or choose full width to inspect changes get a shallow letterbox and must pan more. The detached menu weakens cause-and-effect and obscures precisely the evidence being compared. A 19–26% fit of a desktop webpage is appropriate for orientation, not fine-detail verification; zoom must remain easy to find.

**Fix.** Establish a compact inspection header with one stable target-identity location. Integrate Show/Hide results into the toolbar instead of creating an extra row. Do not remove target/environment identity: consolidate it, with exact information still available. Make full-width Before/After a short labeled mode/side selector rather than multiple long actions. Adapt secondary toolbar actions before they displace the viewer; navigation can be a temporary overlay at the minimum window size. Anchor the View popup to the trigger, use full-width menu rows with a clear selected mark, and clamp it to the window using the already conventional Zoom pattern. Preserve keyboard dismissal and return focus.

**Exact ownership.** `crates/app/src/workflow.rs:349–362`, `512–520`, `617–738` (workspace/queue geometry), especially `721–727` (extra Show results row); `crates/app/src/main.rs:6267–6291` (extra identity block), `6293–6348` (wrapping tools), `6389–6470` (chooser origin/presentation); `crates/app/src/ui/zoom_select.rs:47–70` is the incumbent trigger-anchored pattern. `crates/app/src/workflow.rs:922–940` owns persistent inspected identity.

**Phase / acceptance.** Phase 2. With the same target and 820×600 window, hiding Results should not reduce evidence height; full-width mode should offer a usable document viewport, not just a masthead. Check long paths and historical environments without losing identity. The mode menu should remain adjacent to its trigger in both layouts. Suggested commands: `/impeccable layout`, `/impeccable adapt` (native desktop), `/impeccable clarify`.

### A3 — [P2] A review note is edited as one horizontally scrolling line

**Observed.** The fresh `review-note` fixture contains “Autumn headline and CTA are intentional. Product grid needs release-owner confirmation.” At both sizes the field displays only its tail, despite ample panel height. The editor also says `Scroll for more` when its short body and Save note action are already visible.

**Impact.** A user cannot read the evidence rationale while editing it, must move the caret to recover context, and may interpret horizontal hiding as truncation. This is a usability defect, not demonstrated loss of stored text.

**Fix.** Use a bounded multiline native note field with wrapping, a visible caret and vertical scrolling. Preserve validation, draft ownership, saved/unsaved feedback and return-to-evidence behavior. Size it to several readable lines. Show overflow hints only when the actual scroll owner overflows; replace repeated `Draft until saved` copy with one status beside Save.

**Exact ownership.** `crates/app/src/viewer_ux.rs:185–188`, `264–277`, `328–340`; `crates/app/src/main.rs:372–378`, `480–482` (shared TextInput construction). `crates/app/src/ui/text_input.rs:323` flattens pasted newlines, `526` shapes one line, and `664–677` uses fixed control height: a taller container alone will not create a multiline editor.

**Phase / acceptance.** Phase 2, after viewport hierarchy. A realistic two-sentence note is readable without horizontal navigation, paste supports line breaks, and changing target/closing/reopening retains the correct draft. Suggested commands: `/impeccable harden`, `/impeccable layout`.

### A4 — [P2] Analysis begins with a specialized tool, not the common decision

**Observed.** `review-analysis`, `review-analysis-advanced`, `review-ignored-area`, and the supplemental nine parameter/tail scenes. The Adjust comparison panel begins with Draw ignored area / Add by coordinates before Sensitive/Balanced/Relaxed. Those two masking actions, three presets, Advanced parameters, Reanalyze, Cancel and View previous analysis coexist with similar visual weight. Advanced labels and caveats are truthful but require technical knowledge; they are not all appropriate as the first answer to “why was this marked changed?”

**Impact.** A novice is invited to hide areas before understanding sensitivity. Experts get many nearby actions rather than a concise account of the current analysis and what their edit will change.

**Fix.** Start with current comparison summary/reasons and a named Sensitivity group (three presets, brief explanation). Give Ignored areas its own disclosure with Draw and coordinate entry inside it; preserve the keyboard alternative and pair-binding/revalidation safety. Put advanced parameters and technical metrics below that. Explain whether edits reanalyze automatically; show an active Cancel only while useful, and previous-analysis access only when available, rather than using disabled actions as documentation. Keep original images unchanged and prior revisions retained.

**Exact ownership.** `crates/app/src/main.rs:6503–6537`, `6553–6613`, `6621–6691`, `6757–6779`; `crates/app/src/review_ux.rs:720–775`; `crates/app/src/viewer_ux.rs:220–221` (panel content). Source already supports safety-preserving coordinate editing and retained analyses—change hierarchy, not their guarantees.

**Phase / acceptance.** Phase 3. The initial Analysis viewport explains the current result and offers presets before masking. Advanced fields remain reachable in their actual inner scroll owner at both sizes. Suggested commands: `/impeccable distill`, `/impeccable clarify`.

### A5 — [P2] Persistent preview branding undermines the finished product voice

**Observed.** `Private, local preview` remains at the bottom of the rail on major pages; App settings and the information dialog say `About this preview`. This is explicitly recommended for removal from primary UI.

**Impact.** Repeated development-stage branding takes permanent space and makes even ordinary local project operations feel provisional. “Private” can also be overread as a guarantee that screenshots/ZIPs are safe to share, which the product correctly warns is not true.

**Fix.** **Remove `Private, local preview` from the rail**, rather than replacing it with another slogan. Rename the entry/dialog `About Regressee`. Keep factual local-storage and screenshot-sharing cautions near the relevant actions. Keep build/version, development-browser restrictions, incomplete-runtime errors and qualification limitations visible in appropriate App settings/About details. Removing a slogan must not imply that accessibility, macOS or public-release qualification is complete.

**Exact ownership.** `crates/app/src/workflow.rs:1647–1654`, `crates/app/src/main.rs:6013–6017`, `crates/app/src/form_recovery.rs:306–313`. Runtime readiness feedback in `crates/app/src/main.rs:5940–5973` is not a branding defect and must remain truthful.

**Phase / acceptance.** Phase 1, alongside wizard copy. No persistent rail preview slogan or `About this preview` entry; real build limitations still discoverable. Suggested command: `/impeccable clarify`.

## Nielsen heuristic scores

All ten apply to this desktop **Operate** surface. 0 = absent/broken; 4 = genuinely excellent. Accessibility/input are unqualified, so visual and source evidence cannot earn a blanket 4.

| # | Heuristic | Score / 4 | Evidence and main gap |
|---|---|---:|---|
| 1 | Visibility of system status | 3 | Loading, failure, task progress and baseline state are explicit. Current-stage dimming and ambiguous selected/eligible counts weaken precision. |
| 2 | Match between system and real world | 2 | Pages, login, screen sizes and before/after are natural; origin, probe, mapping and numerical analysis terminology intrude on basic paths. |
| 3 | User control and freedom | 3 | Explicit Close/Return, cancel/pause/stop and draft-preserving logic are present; baseline switching is reversible while retained. Real OS interaction not verified. |
| 4 | Consistency and standards | 2 | Strong shared native controls, but View chooser differs from Zoom, wizard looks like unrelated tabs, and a note uses a single-line field. |
| 5 | Error prevention | 3 | Authorization/login gates and deliberate baseline/export/delete confirmations are strong. Setup’s visual prerequisites need clearer explanation. |
| 6 | Recognition rather than recall | 3 | Text navigation, selected states, evidence identity and local history help. Locked wizard stages and compressed/truncated context still require inference. |
| 7 | Flexibility and efficiency | 3 | Keyboard help, viewer zoom/pan, queue/navigation hiding, profile customization and coordinate editing exist. Minimum-window geometry reduces the value of hiding UI. |
| 8 | Aesthetic and minimalist design | 2 | Coherent restrained identity, but Access and Analysis expose competing secondary work; repeated statuses and chrome consume inspection space. |
| 9 | Recognize, diagnose and recover from errors | 3 | Correction summary/next-correction, no-matches recovery, retained failed evidence and task recovery are constructive. Invalid hidden-field reveal is not visually qualified in this batch. |
| 10 | Help and documentation | 2 | Inline hints, consequences and F1 help are real and useful. General workflow guidance is scattered; no searchable task-help surface was established. |
| **Total** | **Acceptable** | **26 / 40** | **Significant workflow/presentation improvement needed; no visual-world replacement.** |

## Cognitive load and emotional journey

### Checklist

Assessment of the **basic setup and initial Analysis paths**, not a claim that every page is overloaded:

| Checklist item | Result | Reason |
|---|---|---|
| Single focus | Fail | Basic Access is interrupted by environment creation, mapping/probing and statuses. |
| Chunking | Fail | Initial Analysis presents nine distinct actions/options before technical detail. |
| Grouping | Pass, with weakness | Sections/disclosures are real; sensitivity and ignored areas need distinct subgroups. |
| Visual hierarchy | Fail | Wizard progress and secondary analysis actions have near-peer button weight. |
| One thing at a time | Fail | Access mixes login choice with optional environment administration. |
| Minimal choices | Fail in Analysis | Its action cluster exceeds four; View and Zoom also exceed four when deliberately opened. |
| Working-memory support | Pass, with weakness | Draft/state/evidence identity persists; reducing repeated identity must not remove it. |
| Progressive disclosure | Partial | Many advanced fields/details are disclosed, but environment actions and masking are promoted too early. |

Five clear failures here indicate **high extraneous load in those paths**. The ordinary review path is moderate: grouped controls and defaults work, but competing disposition/baseline actions and repeated identity require interpretation. Task dialogs are comparatively focused. This is a qualitative checklist, not measured mental effort.

Decision points with more than four visible options: initial Analysis has **9** named choices/actions (2 masking + 3 presets + Advanced + Reanalyze + Cancel + previous analysis); View chooser has **5 modes** plus Close; Zoom menu has **7 choices**. The latter two are deliberately opened specialist menus, so do not arbitrarily delete useful modes or zoom levels to satisfy a count. Grouping, conventional menus and predictable anchoring are the correct fixes. The basic login picker itself has only **2** choices; the problem is surrounding simultaneous actions, not an oversized picker.

**Emotional sequence.** Projects and the clean rail start calmly. New website setup then introduces permission language, “Unavailable” stages and baseline concepts before the user has a result. Login restores confidence through explicit status/next actions. The peak is seeing truthful before/after evidence, but the small-window letterbox and note editor weaken that moment. The end is cautious and trustworthy: deliberate approval, retained references and no automatic deletion. Keep that confidence; make “what next?” shorter and clearer.

## Persona red flags

- **Jordan, first-time operator:** New website → Access → Pages. Must interpret disabled future stages, an empty multi-website approval, Production/staging/probe, and the distinction between saving settings and saving a baseline. A1 is the main risk; a connected stage strip and optional environment disclosure address it without hiding safety information.
- **Alex, power reviewer:** Review changes → hide Results → full-width Before/After → note. Useful accelerators and keyboard help exist, so “no shortcuts” would be false. The concrete red flags are reduced vertical evidence space, a displaced View menu and horizontal note editing. Bulk approval is not proven by the all-selected fixture: it seeds selection directly. No real interaction-time claim is made.
- **Sam, keyboard/low-vision operator:** First-time setup and evidence inspection. Visible text labels, check marks, focus treatments in native controls, F1 help and coordinate entry are positive. Risks are low-emphasis locked-stage explanations, long prose at minimum size, and small fitted website text. The evidence does **not** establish screen-reader roles, announced states, native scaling, IME or keyboard-only task completion. Those are mandatory qualification work, not invented observed accessibility failures.

No project-specific persona is fabricated: no applicable `AGENTS.md` Design Context was found. Mobile thumb-zone personas are inappropriate for this native desktop target.

## Minor observations and follow-ups

- **[P2] Baseline eligibility wording is ambiguous, not mathematically wrong.** Default review reads `0 selected · 0 eligible` while Select can be enabled. `crates/app/src/workflow.rs:1245–1278` counts eligible members **of the current selection**, not all eligible targets. Use `0 selected` when empty and `N selected · M ready to update` when relevant, or explicitly label “eligible selected.” Do not report a backend eligibility defect.
- **[P2] Long Settings/History reading paths.** At 820×600, History’s heading, tabs, full-width search/help and subsection text place the first baseline row near the bottom; actions/details are below it. Fixed 256 px nested record lists also leave blank space with few items. Keep loaded-page search scope honest, but condense its repeated explanation and make selected-record context/actions easier to scan. `crates/app/src/workspace_history.rs:247–286`, `289–341`, `488–531`, `615–649`. No last-row reachability defect is established by this default fixture.
- **[P2] Repeated reassurance needs hierarchy.** Baseline-unchanged, saved-local and browser-status reminders are important, but often repeat in top context, body and footer. Keep one persistent task-level statement plus specific consequence text at the action; move detailed limitations into existing disclosures.
- **[P3] Terminology alignment.** `No significant change` in filters becomes `Unchanged` in queue summary (`crates/app/src/main.rs:4214–4219`, `4266–4274`). Keep the short summary, but consistently explain that this means below thresholds, not identical pixels. `Settings` heading versus `App settings` rail label should be aligned (`crates/app/src/main.rs:5964`).
- **[P3] Technical identifiers are correctly opt-in.** History provenance contains hashes, rendering structs and exact bytes; retain that separation rather than making it the default history view. Add copy/reveal affordances only if real user workflows justify them.
- The local runtime reports “Browser setup incomplete in this build.” This is observed development-build status, **not evidence of a shipped install failure**. Preserve the truthful message and qualify distribution separately.
- No contrast ratio, rendering performance, motion, copy-to-clipboard, RTL, dark-mode, high-DPI or native accessibility result is inferred from these static captures. Readable screenshots are not equivalent to passing those tests.

## Recommended implementation phases

| Phase | Scope | Preserve / gate |
|---|---|---|
| 0 · Evidence and contracts | Use this source/capture snapshot; freeze native identity and explicit operation boundaries. Resolve coverage gaps in the next planned QA batch, not an endless polish loop. | No redesign; no safety/authorization relaxation; exact capture provenance. |
| 1 · Wayfinding and first success | A1 connected wizard, current-stage validity separation, contextual multi-origin approval, simpler Access; A5 remove rail preview slogan and rename About. | Public/private login paths, draft retention, saved/unsaved states and truthful build limitations. |
| 2 · Inspection workspace | A2 compact stable evidence header and trigger-anchored mode menu; A3 real multiline notes; clarify selected eligibility. | Inspected target/environment identity, untinted originals, independent review vs baseline operations. |
| 3 · Secondary work | A4 sensitivity/ignored-area hierarchy; condense Settings/History help and repeated status; retain exact details on demand. | Inner-scroll keyboard reveal, masks bound to exact pairs, history pagination/search truth, sharing/deletion warnings. |
| 4 · Native qualification | Real OS input, keyboard-only flow, IME/text editing, screen-reader semantics/announcements, scaling/high contrast and supported-platform presentation. | Offscreen success is not a substitute for these release gates. |

Each implementation phase should have one planned batched inspection across the two desktop sizes and relevant states, one defect-fix batch, and at most one confirmation round. This assessment itself performed no fixes or rerender/polish loop.

## Inspection coverage and honest limits

**Personally inspected 238 fresh native app PNGs**: 176 base matrix + 22 supplemental analysis/diagnostics + 40 task states. Every image was inspected via labeled contact sheet in one bounded batch; **20 app PNGs** were additionally examined at full resolution for specific findings. Renderer proof was examined separately before authorizing the matrix. Counts exclude the proof and four freshly generated synthetic website fixture PNGs.

The complete durable inventory and exact contact/full-resolution mapping are in [inspection-ledger-a.md](inspection-ledger-a.md). Private machine-readable per-PNG ledger: `<root>/inspection-ledger-a.json`; original image hashes: `<root>/gallery-manifest.json`; gallery: `<root>/index.html`. Sheets are labeled scaled derivatives, not substituted production screenshots. The static gallery was created as an index; this review inspected PNGs directly, not a running web rendition of the app.

| Surface | Actually inspected | Limits / unqualified behavior |
|---|---|---|
| Projects | Populated, empty, loading, failed, chooser, New website | No actual user project/open/save operations. |
| Wizard Website | Blank new project, existing values, address validation, footer | No submission to a website; unsupported invalid-input extremes not sampled. |
| Wizard Access | Public/login choices, saved/private, browser open, ready, loading, invalid, options, mapping preview, timezone popup | Browser/login/session outcomes are synthetic. `access-options` visibly renders mapping/locale/timezone fields; its earlier thumbnail ambiguity is resolved, not a missing disclosure. |
| Pages & sizes | Default plan, screen-size checkboxes, extent radios, image choices, speed presets, advanced controls and tail, invalid summary, mobile-labeled state | **Not every alternate value was activated.** Custom actions/complete limitations tail, image-mode alternate outcome and capture limitations disclosure not visually exhausted. |
| Project settings | Top and bottom, combined configuration | No persisted-edit or destructive-login tests performed by this assessment. |
| Discovery / Pages & baseline | Fixed page list, preview list, page options, no matches | Inclusion/exclusion/search-match actions are visible; their post-action changed outcomes are not each captured. Default fixture does not show a live website preview. |
| Ordinary review | Desktop/mobile website targets, mixed 148-target queue, failed target, hidden queue/navigation open, no matches | Website images and queue outcomes synthetic. Scroll/pan and keyboard behavior are not personally exercised in a real OS window. |
| View / zoom | Compare, Difference, Overlay, Heatmap, full-width Before/After, mode chooser and zoom menu | Mode states are rendered; pointer hit testing/zoom-anchor behavior/motion not OS-qualified. |
| Attached inspectors | Capture details, diagnostics and diagnostics tail; Analysis, all nine advanced fields, switches tail, ignored-area coordinates; realistic note | Supplemental scene assertions use actual inner scroll geometry. Ignored-area drawing, rectangle edits/save and preset-reanalysis outcomes not end-to-end exercised. |
| Result filters | Default All choices, Group by screen size option, no-search-match recovery | Changed/Gaps/Unchanged selection and grouped outcomes are **not separate captured states**. |
| Approval | One selected, confirmation, all selected, last provenance reference/row/exact tail | “Many” seeds selection directly (`crates/app/src/ui_review_matrix.rs:807–816`), so it does not prove a bulk-selection UI. No baseline change confirmed. |
| Capture/evidence states | Ready, saved, mobile, gaps/failure, empty, loading, read failure/dismissed | No live capture, login requirement, disk exhaustion or real recovery run. |
| History | Runs/top/bottom, baselines, storage, empty/no matches, recorded check/capture, advanced provenance | Default has few records/one baseline; retained/composite baseline variants and last loaded record behavior are not visually qualified. Provenance does render distinctly. |
| Portability | Export/import/deletion/integrity previews | No confirm/delete/export/import executed; exact-detail expanded previews not sampled. |
| App settings / help | Top, network, diagnostics, diagnostic/integrity previews, F1 keyboard help and About | Network limitations toggle/alternate runtime states not separately captured. Keyboard-help fixture really calls `show_keyboard_help` (`crates/app/src/ui_review_matrix.rs:844`); earlier contrary suspicion rejected. |
| Long-running tasks | Discovery first-page/retry/browser check/rules, partial/stopping, capture progress/pausing/paused/attention/completion/details, baseline finishing, setup download, ZIP import/export-stop/late success, failed task | Synthetic state presentations, not actual ongoing network/file operations or cancellation races. |

### Capture-harness gaps that must not be promoted to production defects

1. `pages-invalid` paints the actionable error summary but **not the invalid Parallel active pages field**. Default scene does not set `focus.advanced`; the explicit reveal branch is gated by the unused phase5 mode. `crates/app/src/ui_review_matrix.rs:726`, `761`, `1021–1044`. Current-stage dimming is visible and source-confirmed; hidden-invalid-field reachability is not qualified.
2. `pages-mobile-revealed` is a label, not proof that keyboard reveal happened: the reveal path runs only in phase5. Its screenshot was inspected as an ordinary Pages state, not certified focused-control coverage.
3. `pages-advanced-tail` scrolls to Retries, not the final custom-profile actions/complete limitations text (`crates/app/src/ui_review_matrix.rs:894–895`). Parts below the captured fold remain source-reviewed only.
4. Default History uses short lists. Phase6 last-record/composite variants exist but were not used; default screenshots cannot establish their reachability. This is an evidence limit, not a claim that selecting those rows fails.
5. State construction sometimes assigns drafts/statuses directly. Source inspection and rendered states support presentation findings, but do not substitute for real user-driven transitions, backend tests or OS interaction qualification.

## Capture commands and verification

Common environment, from repository root:

```sh
export REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode
export REGRESSEE_RENDERER_CAPTURE_DIR=/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-review-20261013-9g3qdv3j
export LIBRARY_PATH="$PWD/.runtime/link-libs"
export TMPDIR=/var/home/olav/Data/opencode/tmp/opencode
```

Exact test form: `cargo test --locked --offline -p regressee-app --all-features --bin regressee <test> -- --ignored --exact --test-threads=1`.

| Test / command | Result |
|---|---|
| `ui_renderer_capture::renderer_capture_proof` | Passed; proof personally inspected before matrix authorization. |
| `.runtime/installed/node/bin/node scripts/ui-review-fixtures.mjs` with `REGRESSEE_REVIEW_WORKER=.runtime/installed/worker` and `PLAYWRIGHT_BROWSERS_PATH=$PWD/.runtime/installed/browsers` | Fresh offline fixtures generated; all page requests aborted. Initial system-node attempt lacked Playwright, resolved using existing installed runtime; no install. |
| `ui_renderer_capture::ui_review_matrix::renderer_capture_whole_app` | Initial tool timeout after 61 fresh PNGs; resumed **same batch** with `REGRESSEE_RENDERER_COMPLETE_BATCH=1`, no command timeout. Passed, 186.93 s; total base 176. |
| Same whole-app test with `REGRESSEE_RENDERER_PHASE7=1` | Passed, 65.49 s; 22 supplemental images. |
| `ui_renderer_capture::renderer_capture_tasks` | Passed, 24.69 s; 40 task images. |
| `python3 scripts/ui-review-gallery.py <root>` | Indexed 238 original app PNGs; wrote hashes and labeled contact sheets. |

Retained base and supplemental manifests: `<root>/whole-app-base-matrix.json` (176), `<root>/whole-app-supplemental-matrix.json` (22); merged `<root>/whole-app-matrix.json` (198). Same-root PNG retention was hash-checked by the unchanged harness. No stale previous-root app images were reused. Source fingerprints were rechecked after assessment; production source remains unchanged. No commit or push.

Final artifact verification: `<root>/assessment-a-verification.json`. All 238 original/metadata hashes and PNG dimensions match; the durable inventory matches the manifest; all 13 source/script fingerprints are unchanged; 43 distinct inspected contact sheets resolve; the root remains 0700. SHA-256 of the expanded private inspection ledger: `0be8cc1fcd4e19e662e2023b93c6a12b601fcce45842b243517b28f469b8fea8`. The only repository additions are this report and `inspection-ledger-a.md`.

## Questions for parent synthesis (non-blocking)

1. Which first implementation slice best serves the next users: **public/private setup first**, **small-window review first**, or **both as the core-path phase**?
2. For expert controls, should the default be **simple with named disclosures** or **remember the operator’s expanded preferences**? Either must keep permissions, evidence identity and explicit baseline confirmation unchanged.

These are handoff questions, not requests to reopen this bounded visual-review batch. Assessment B and the parent’s prioritization remain independent inputs.
