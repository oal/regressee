# Assessment B — isolated native-desktop evidence review

**Assessment B only.** This agent did not read Assessment A or `assessment-a.md`, did not consult prior critique files, did not change application code, did not render new frames, and did not stop the shared gallery server. The parent should finish Assessment A before reading this report. This is evidence for synthesis, not a replacement design or a native usability acceptance verdict.

**Product target:** Rust/GPUI desktop application, not the gallery HTML and not a mobile app. The websites' Mobile/Tablet/Desktop profiles are capture profiles, not product device classes. Mode: Operate. Reviewed targets:

- `crates/app/src/main.rs`
- `crates/app/src/workflow.rs`
- `crates/app/src/review_ux.rs`
- `crates/app/src/options_ux.rs`
- `crates/app/src/workspace_history.rs`
- `crates/app/src/task_ux.rs`
- `crates/app/src/ui/controls.rs`
- `crates/app/src/ui/theme.rs`

Evidence root: `/var/home/olav/Data/opencode/tmp/opencode/regressee-whole-app-review-9inkus48`. Screenshot/artifact paths below are relative to that root. Source paths are relative to `/var/home/olav/Projects/Regressee`.

## 1. Mandatory detector: real runs, complete reported counts, limits

The bundled launcher was actually executed from the project root. Context was loaded once for `crates/app/src/main.rs`. No critique ignore file exists at `.impeccable/critique/ignore.md`.

### Authoritative explicit-target scan

```text
/var/home/olav/.config/opencode/skills/impeccable/scripts/impeccable detect --json crates/app/src/main.rs crates/app/src/workflow.rs crates/app/src/review_ux.rs crates/app/src/options_ux.rs crates/app/src/workspace_history.rs crates/app/src/task_ux.rs crates/app/src/ui/controls.rs crates/app/src/ui/theme.rs
```

- Exit code: **0**.
- Raw stdout: `[]` followed by a newline, saved unchanged as `detector.json`.
- Raw stderr: empty, saved as `detector.stderr.txt`.
- Command, working directory, exit status and output locations: `detector-run.json`.
- Total reported findings: **0**.
- Reported primary findings: **0**; reported advisory findings: **0**.
- Complete reported rule-count map: **`{}`**. No rule names were emitted.
- Complete reported finding-location list: **`[]`**. No source lines were emitted.
- False positives: **none returned; 0 findings to adjudicate**.

| Explicit source target | Reported findings/locations |
|---|---|
| `crates/app/src/main.rs` | 0 / none |
| `crates/app/src/workflow.rs` | 0 / none |
| `crates/app/src/review_ux.rs` | 0 / none |
| `crates/app/src/options_ux.rs` | 0 / none |
| `crates/app/src/workspace_history.rs` | 0 / none |
| `crates/app/src/task_ux.rs` | 0 / none |
| `crates/app/src/ui/controls.rs` | 0 / none |
| `crates/app/src/ui/theme.rs` | 0 / none |

An initial directory attempt, `impeccable detect --json crates/app/src`, also exited 0 with `[]` and empty stderr. Its original stdout and run metadata are retained as `detector-directory.json` and `detector-directory-run.json`. The explicit eight-file attempt removes ambiguity about the requested targets. The generated gallery was **not** scanned as product UI.

**Do not translate this into “the native UI is clean.”** `detect --help` describes static HTML/CSS analysis, non-HTML regex matching and Puppeteer URL rendering. It supplies no Rust/GPUI layout parser, native accessibility tree, native glyph bounds or OS-control evaluation. These results contain neither a scannable-file count nor an executed-rule inventory. It is not possible to prove which Rust constructs received meaningful coverage. The manual defects below are exactly the kind of native geometry and ownership problems this output cannot certify. The command did not crash or report a missing entry point; it ran and returned an empty finding array.

## 2. Browser requirement: attempted fresh tab; unavailable

I attempted an **own fresh tab** using `tools.browser.tabs.open({url: "http://127.0.0.1:40577/index.html"})`. The tool failed before creating a tab:

```text
[browser.disconnected] No desktop browser is connected to this session. Open this session in the desktop app and wait for it to connect. Then call browser.tabs.list({}). Repeating browser actions while disconnected will not help.
```

- Browser tab ID: **none**.
- Gallery browser inspection: **unavailable after a real attempt**; fallback was direct inspection of native PNG files through the image-read tool.
- Mutable-injection preflight, detector-script injection, browser console findings: **not performed**, because no connected browser was available.
- **Native overlay unavailable / not applicable.** GPUI controls inside these PNGs have no DOM. Injecting into the gallery would inspect the gallery's HTML scaffolding, not the application's controls. No reliable native or user-visible detector overlay was created or claimed.
- No new server was started; no shared server was stopped.

## 3. Pixel authenticity, source freshness and coverage reconciliation

Machine-readable verification is saved in `assessment-b-integrity.json`.

### Counts, dimensions and hashes

- `gallery-manifest.json`: **182 entries** = **71 whole-app labels × 2 sizes** + **20 task labels × 2 sizes**.
- Exactly **91 PNGs at 1280×800** and **91 PNGs at 820×600**.
- Each family/state has one of each size; **0 missing or malformed size pairs**.
- Every PNG's SHA-256 agrees with both its manifest hash and its individual capture JSON; PNG header dimensions agree with the manifest; **0 image-hash/dimension mismatches**.
- **174 unique PNG hashes**, with the seven duplicate groups listed below. Thus 182 labels are not 182 distinct visual states.
- `whole-app-matrix.json`: **142 whole-app entries**, comprising **83 retained entries without geometry** and **59 entries with native debug geometry**. Task PNGs are in the 182-entry gallery, not in this 142-entry geometry matrix.
- All 182 individual metadata files report `real_gpui_blade_offscreen=true`, `real_linux_fonts=true`, `synthetic_project_only=true`, `opaque=true`, `srgb=true`, and `premultiplied_alpha=true`.
- All 182 explicitly report `os_presentation_verified=false` and `os_input_verified=false`.
- I also inspected `renderer-proof.png`: the native font/button/field/radio test and primary-colour swatches are visibly populated, not an all-black OS-window capture. Its recorded hash and review receipt are in `proof-reviewed.json`. This calibration is evidence of the renderer's pixels, not a live input or OS presentation test.

### Source bindings

The 52 paths in `production-source-binding.json` were SHA-256 checked against the current workspace. **51 match. All eight requested product UI target files match.** All source-hash fields supplied by the 182 individual PNG metadata files also match the current corresponding source (main, controls, task UI/policy, icons and web links).

One binding differs, and it must not be hidden:

| Path | Binding file hash | Current hash |
|---|---|---|
| `crates/app/src/ui_review_matrix.rs` | `697c6f6edbe60a373b5f4af6c3b3655a4e5691ab625f1484b35486d8c3aac636` | `83ff46696871b1fb3a5c6ed46a69a5c5e15f8bf175e658a20a77616c7fcc1853` |

That is the test/capture harness, not one of the requested production UI files. Accordingly: production UI freshness is supported; a claim that **all** bound files are byte-identical would be false. The retained entries' missing geometry is another explicit limit; no exact selector bounds have been invented for them.

### Fixture truth, not website truth

- There are **12 stored references** for six page addresses at two screen sizes in the principal synthetic project.
- Discovery's **74 addresses** and mixed review's **148 page/profile targets** exercise presentation scale. The mixed queue clones captures/results in memory; it is **not** 148 independently captured pages or a storage-backed approval sample. `ui_review_matrix.rs:273–334` says this explicitly.
- **Four rich synthetic site images are intentionally repeated** across targets/profiles. Repetition is fixture construction, not proof of a duplicate-evidence bug in the app or real coverage of those websites.
- Approval screenshots demonstrate a **one-reference replacement / eleven inherited references** from the 12-reference fixture. They do not qualify bulk approval for 148 targets, 1,000 pages, or approval reports.
- Task fixtures use “Approval UI fixture,” explicit fixture profiles, localhost URLs and synthetic timestamps. Zero elapsed seconds, a 1970 setup timestamp and placeholder task context in some scenes are fixture assumptions, not observed runtime timing/date bugs.
- No real website was visited in this assessment. Stored-project/evidence comparisons in the capture harness protect its isolated fixture; this agent did not mutate user data.

### Exact duplicate groups and their significance

| Identical captures | What is **not** demonstrated |
|---|---|
| `all-1280x800-new-website` = `all-1280x800-website` | A clean, unbound first-project form at wide size |
| `all-820x600-new-website` = `all-820x600-website` | A clean, unbound first-project form at minimum size |
| `all-1280x800-review-details` = `all-1280x800-review-diagnostics` | The technical diagnostics contents, as a distinct visible state |
| `all-820x600-review-details` = `all-820x600-review-diagnostics` | Same limit at minimum size |
| `all-1280x800-review-analysis` = `all-1280x800-review-analysis-advanced` | Visible advanced-parameter controls |
| `all-820x600-review-analysis` = `all-820x600-review-analysis-advanced` | Same limit at minimum size |
| `all-820x600-review` = `all-820x600-review-overlay` = `all-820x600-review-heatmap` | Overlay/Heatmap rendering at minimum size |

These are **coverage gaps**, not seven automatic product defects. For New website there is a concrete harness origin: `ui_review_matrix.rs:490` navigates to Setup, then `:522` activates NewProject, while `main.rs:5238` only mounts NewProject on Projects. The resulting image still shows the existing project's fields. For the narrow Overlay/Heatmap labels, `main.rs:5949` intentionally removes those toolbar controls; `main.rs:5315` only mounts them in narrow mode when Analysis is open, and the fixture did not open it before activation. Advanced/diagnostics flags may be set without bringing their content into the bounded scroller's viewport. The filenames alone cannot establish that the named content was seen.

## 4. Inspection ledger — breadth versus full-size verification

I independently inspected **all 32 contact sheets**: `contact-sheets/all-1280-01.png` through `all-1280-12.png`, `all-820-01.png` through `all-820-12.png`, `task-1280-01.png` through `task-1280-04.png`, and `task-820-01.png` through `task-820-04.png`. This covers the actual pixels for all 182 labelled entries, at contact-sheet scale.

I then inspected **32 original product PNGs at full size**, plus the renderer calibration proof. This is not a claim that all 182 were individually inspected at native scale.

| Family | Full-size original PNGs inspected |
|---|---|
| Projects/setup | `all-820x600-projects.png`, `all-820x600-website-validation.png`, `all-820x600-access-login-actions.png`, `all-820x600-mapping-preview.png`, `all-820x600-timezone-popup.png` |
| Discovery | `all-820x600-discovery-options.png` |
| Normal review | `all-1280x800-review.png`, `all-820x600-review.png`, `all-820x600-review-details.png`, `all-1280x800-review-diagnostics.png`, `all-1280x800-review-analysis-advanced.png`, `all-820x600-review-ignored-area.png`, `all-820x600-review-note.png`, `all-820x600-review-no-matches.png` |
| Approval | `all-1280x800-review-approval-confirm.png`, `all-820x600-review-approval-confirm.png` |
| Capture/failed evidence | `all-820x600-review-failed-target.png`, `all-820x600-capture-failed.png`, `all-820x600-capture-empty.png` |
| History | `all-1280x800-history-runs.png`, `all-820x600-history-runs.png`, `all-820x600-history-bottom.png`, `all-820x600-history-baselines.png` |
| Sharing/storage/help/settings | `all-1280x800-export-preview.png`, `all-820x600-export-preview.png`, `all-820x600-import-preview.png`, `all-820x600-integrity-preview.png`, `all-820x600-app-settings.png`, `all-1280x800-keyboard-help.png` |
| Tasks | `task-820x600-capture-progress.png`, `task-820x600-capture-gap-details.png`, `task-820x600-capture-setup-details.png` |

Breadth checks included state differences, dialog containment, footers, lower scroll positions, minimum-width wrapping, title/control ownership, screenshot failures and duplicate labels. Full-size reads were concentrated on safety-critical approval, dense review geometry, history, scrolling disclosures and failure recovery.

## 5. Native geometry — measured versus estimated

The following coordinates are **recorded native debug bounds** from non-retained entries in `whole-app-matrix.json`, in the PNG coordinate space. They are not DOM measurements, OS window-frame bounds or estimates.

| 820×600 state / selector | x | y | w | h |
|---|---:|---:|---:|---:|
| Review / `review-queue` | 216 | 144 | 200 | 300 |
| Review / `viewer-toolbar` | 428 | 144 | 368 | 40 |
| Review / `phase5-before` | 428 | 228 | 178 | 216 |
| Review / `phase5-after` | 618 | 228 | 178 | 216 |
| Review / `ordinary-review-decisions` | 216 | 448 | 580 | 62 |
| Review / `approval-next-action` | 192 | 518 | 628 | 82 |
| Review / `page-list` | 216 | 284 | 200 | 160 |
| Review note / `phase5-before` | 428 | 228 | 178 | 172 |
| Review note / `phase5-after` | 618 | 228 | 178 | 172 |
| Review note / `page-list` | 216 | 284 | 200 | 116 |
| Failed target / `review-queue` | 216 | 144 | 200 | 275 |
| Failed target / `ordinary-review-decisions` | 216 | 423 | 580 | 62 |
| Failed target / `approval-next-action` | 192 | 493 | 628 | 107 |
| Discovery options / `review-queue` | 216 | 151 | 248 | 441 |
| Discovery options / `page-list` | 216 | 331 | 248 | 261 |
| History runs / `page-list` | 216 | 463 | 580 | 130 |
| History runs / `history-run-list` | 217 | 464 | 578 | 128 |
| History bottom / `history-run-list` | 217 | 56 | 578 | 128 |
| History runs / `selected-project-context` | 216 | 16 | 250 | 24 |
| Empty capture / reserved `review-queue` | 216 | 113 | 200 | 479 |

The default narrow pair contains **76,896 image-viewport pixels**: 2 × 178 × 216, **15.6% of the 820×600 window area**. These are pane bounds, not the amount of readable source content. A 1440px-wide screenshot is scaled to approximately **12.4%**, consistent with the displayed 12% zoom. Opening a note reduces each pane to 172px high and leaves only 116px of queue-list height (less than two 64px rows).

The 192px navigation rail, 24px work inset, 12px pair/list gap and width-dependent queue/chrome are source-backed in `workflow.rs:488–494, 663–700`, `ui/theme.rs:25–31` and the task workspace's rail in `task_ux.rs:1143–1188`.

**Approval modal bounds are not supplied by this matrix.** In the originals they are approximately `(260,190,760,420)` at 1280×800 and `(120,90,580,420)` at 820×600. These are **PNG/source corroborated estimates**, not recorded selector measurements. Source declares 760/580px width and 420px height (`workflow.rs:948`). Row/border positions below are likewise visual estimates, clearly labelled.

## 6. Source-backed visual findings for synthesis

No P0 is established by this still-image assessment. The following priorities describe visible difficulty and information risk, not proof of a failed storage transaction or a measured user-error rate. Suggested changes are critique recommendations only; none were implemented.

### B1 — [P1] The final baseline decision cannot cleanly display its own reference/provenance rows

**Evidence:** `all-1280x800-review-approval-confirm.png` and `all-820x600-review-approval-confirm.png`, centre modal, reference list directly above exact provenance.

- Wide: the first replacement row's requested→final URL is crossed by its row separator, around **y≈412**. A partial **“Keep · https://northline.example/”** row then appears immediately above the exact provenance for the first **Replace** item. The inspected item has no prominent selected-row treatment or “Inspecting Replace” heading to explain this association.
- Narrow: the replacement row's last URL line is visibly cut across its height at the list/provenance boundary, around **y≈334–338**. Exact provenance starts underneath, and its lower original-artifact content is also clipped into an independent scroller above the action footer. The next inherited reference is not visible in this initial list viewport.
- These are original native pixels, not thumbnail resampling. All modal action buttons remain contained and readable; the defect is **information containment/ownership**, not the dialog falling outside the window.

**Origin:** `workflow.rs:947–972` packs a header, impact copy, four-line virtualized rows, a fixed 100px exact-provenance scroller and four action buttons into a fixed 420px modal. Each row declares 100px height plus vertical padding and gaps (`:958`), but its four text children (`:959–962`) do not pin compatible line heights or individually bound their paint. The visible separator intersects the last line. The current PNGs do not support claiming that this layout is resolved.

**Why it matters:** This is the deliberate baseline-changing moment. A clipped source URL, partial inherited item and adjacent provenance for a different selection undermine the user's ability to verify what will change. Eleven inherited items can remain technically present yet practically opaque.

**Recommendation, not an edit:** give each reference row a content budget that actually contains its text, make the list and exact provenance unambiguously separate bounded regions, clearly identify the inspected row's Replace/Keep role, and expose overflow. Verify one, many and long-address selections at minimum width, including the last inherited item and the exact-artifact tail. Candidate commands: `/impeccable layout`, `/impeccable harden`.

### B2 — [P1] Result rows hide the page identifier while preserving the repeated origin

**Evidence:** `all-820x600-review.png`, `all-820x600-review-note.png`, `all-820x600-capture-failed.png`; page list at x=216, width 200. Also visible in `all-1280x800-review.png` at width 272.

- At minimum width, several rows present essentially the same **`https://northline.exa…`**. The actual path—the differentiator among 100–1,000 pages—is omitted before it can be read.
- The second line also loses its distinguishing status/explanation: **“Desktop · Changed — r…”**, **“Mobile · Changed — re…”**, and **“Mobile · Could not capt…”**.
- Wider rows still clip longer account/collection paths and their status text. The first-root item is readable, so a six-page fixture understates the larger list's recognition burden.

**Origin:** `main.rs:4315–4321` passes the entire requested URL first, with profile/classification/attempts in one details string. `ui/controls.rs:702–737` gives those strings fixed-height two-line ellipsis treatment. `page_row` does not attach a full-address tooltip or an alternate path-first label. `workflow.rs` supplies the constrained queue width.

**Why it matters:** Users must click and inspect each row to identify the page or failure they intend to act on. For a baseline-selection workflow this is not merely aesthetic truncation: the visual identifier should distinguish targets before a decision.

**Recommendation:** put the path or human-readable page identifier first, keep the common origin in shared context, preserve screen-size/status as separately budgeted facts, and provide deliberate full-address disclosure without silently changing evidence. Keep the original URL stored; this is a presentation change. Candidate commands: `/impeccable clarify`, `/impeccable layout`.

### B3 — [P1] Minimum-width comparison becomes a thumbnail task; open disclosures erase nearly all evidence

**Evidence:** `all-820x600-review.png`, `all-820x600-review-note.png`, `all-820x600-review-details.png`, `all-820x600-review-ignored-area.png`, and the measured geometry in §5.

- Each initial desktop image is **178×216px**, showing a 1440px website at 12% scale. Website text is not legible at this scale, even though both whole-image thumbnails are present.
- A note reduces image height to **172px** and list height to **116px**. A Details/Analysis flyout covers almost the entire 368px viewer region, while disposition and baseline-selection bars remain visible and actionable below it.
- The compact zoom button reads **“Zoom / 12%”**, not “Fit width”; automatic fit versus manual percentage is only explained inside its menu.
- The drag/scroll/zoom guidance visible at wide size is omitted at narrow size (`main.rs:5994`), precisely where users must use zoom/pan to inspect the evidence. The source supports those operations, but screenshots do not test their usability.

**Origin:** persistent 192px rail + 200px queue + side-by-side panes; fixed attached review/approval chrome; over-viewer flyouts (`workflow.rs:663–708`, `main.rs:5927–5998, 6373–6398`). Compact zoom display is `ui/zoom_select.rs:354–374`. This is the implemented native minimum-width layout, not a mobile breakpoint problem.

**Why it matters:** Non-developer users reviewing small release differences must first discover the navigation/zoom mechanics or hide the queue, then pan. A visible classification action is not a substitute for readable comparison evidence. This is substantial task friction, not proof the flow is impossible; Hide page queue and Zoom are real mitigations.

**Recommendation:** prioritize an inspectable original at minimum size, preserve an explicit fit/manual cue and a concise interaction cue, and ensure a details/analysis decision does not leave users judging hidden evidence. Retain synchronized originals and the evidence safety contract. Candidate commands: `/impeccable adapt`, `/impeccable layout`.

### B4 — [P2] History row paint is correctly full-width, but the browsing viewport hides loaded records with little overflow guidance

**Evidence:** `all-1280x800-history-runs.png`, `all-820x600-history-runs.png`, `all-820x600-history-bottom.png` and `all-820x600-history-baselines.png`.

- **Positive correction:** selection paint spans the full row, and the native measured narrow list is 578px wide inside a 580px container. No label-sized selected-row defect is present in these images.
- Runs says **“3 of 3 runs shown”** but only **two** 64px records are visible in a 128px inner viewport, at both widths. No visible thumb or “scroll for more records” accompanies this nested scroller in the initial screenshots.
- At 820×600 the outer list begins at **y=463**, leaving selected-run details and pagination below the initial screen. The bottom-position capture proves those details exist and can render; it does not demonstrate the nested third row or its selection.
- Baselines has just one record in this fixture. It cannot verify retained-version browsing, a Composite version's long details, or large lists.

**Origin:** `workspace_history.rs:529–563` caps the Runs scroller at **128px**, independent of the window's available height. Baselines is capped at **192px** (`:315–355`). Unlike task/disclosure surfaces, these lists do not draw their own visible overflow indicator. Shared history row geometry is correctly source-backed in `ui/controls.rs:744–760`.

**Why it matters:** Users trying to find earlier evidence must distinguish outer-page scroll, nested-list scroll and history-page navigation. Accurate loaded-record counts do not tell them which scrolling surface hides the third item.

**Recommendation:** let history browsing use a meaningful amount of available space and disclose list overflow explicitly. Clarify “loaded/matching” counts versus viewport-visible rows. Test selection of the last loaded item independently of outer-page scrolling. Candidate commands: `/impeccable layout`, `/impeccable clarify`.

### B5 — [P2, state/fixture-qualified] The generic missing-viewer branch loses the failed target's diagnosis and route to Details

**Evidence:** `all-820x600-review-failed-target.png` versus `all-820x600-capture-failed.png` and `task-820x600-capture-gap-details.png`.

- Comparison failed-target image: **“No screenshot for this target”**, with advice to select a captured target. The target URL, screen size, reason and recovery action are absent from this placeholder. The only viewer disclosure offered is **Analysis**, not Details. The selected failed target is also outside the visible top rows of the 148-target queue.
- Capture-only failed image: it correctly names **“Website took too long”**, preserves capture origin/profile and retains Details, with no useless zoom.
- Task gap Details is stronger still: labelled page/profile/attempts and a clear next step for timeout versus server failure.

**Origin:** the `viewer=None` branch in `main.rs:5899–5916` emits Analysis and `screenshot_placeholder()`. `loading_ux.rs:65` uses the generic MissingTarget copy. Meanwhile `viewer_ux.rs:334–383` already knows the requested page, profile, failure code and targeted recovery advice, but that information is not surfaced by this branch.

**Important qualification:** `ui_review_matrix.rs:529` explicitly selects an in-memory cloned failed target and calls `clear_viewer`; it does not exercise a real storage read for that failed comparison. This proves the **rendered missing-frame branch** is weak, not that every genuine failed comparison takes that branch. A valid capture-only ViewerFrame demonstrates a different, better fallback. Confirm real failure routing before generalizing or assigning P1.

**Recommendation:** show available target identity/reason/recovery even when no image frame exists, and expose the relevant Details path rather than inviting unrelated image analysis. Candidate commands: `/impeccable harden`, `/impeccable clarify`.

### B6 — [P2] Some safe-action copy does not match the action/control context

**Evidence / source-backed examples:**

1. `task-820x600-capture-paused.png` (contact sheet `task-820-01.png`): message says **“Return to the workspace to resume this paused capture”**, but the primary action says **“Review check results.”** It can lead to the correct workspace; the label nevertheless makes resuming harder to recognize. `task_ux.rs:199–204, 233–239` selects the action by Kind::Check even for paused outcomes.
2. `all-820x600-access-login-actions.png`: editable selection is login-required, yet the saved-environment summary says **“Public access selected”** and the browser actions are disabled. The text is accurately labelled **Saved environment**, but the page also says **“Open the browser…”** instead of making the immediate required Save step dominant near those actions. This is a saved-versus-draft clarity problem, not evidence credentials were mishandled.
3. `all-820x600-integrity-preview.png`: both **Confirm temporary-only cleanup** and **Confirm deletion of complete unreferenced recovery objects** remain visible when the report says all corresponding counts are **0**. The second, irreversible action is presented as a neutral secondary button, despite explicit permanent-deletion language. This creates unnecessary safety decisions and weakens consistency with History's red deletion treatment. Source and screenshot establish the presentation; this assessment did not activate either control.

**Recommendation:** make the primary label name the user's immediate next operation, distinguish saved access from pending access where disabled actions appear, and remove/disable zero-work cleanup confirmations with consistent danger treatment for actual irreversible work. Candidate commands: `/impeccable clarify`, `/impeccable harden`.

### B7 — [P2] Safety text and diagnostic vocabulary often consume the viewport before the user's target

**Evidence:** `all-820x600-import-preview.png`, `all-820x600-website-validation.png`, `all-820x600-mapping-preview.png`, `all-820x600-history-baselines.png`, and the contact sheets for Access/Pages advanced settings.

- Import begins with strong truthful privacy/provenance warnings, but at minimum height the preview's confirmation/cancel actions fall below the initial screen, after a long amber passage. This is a scrolling form, not an out-of-window modal; the action is not proven unreachable.
- Website validation's summary and next-correction control push the actual Starting addresses field below the initial viewport. Go to next correction is a real mitigation; its native activation was not tested.
- Mapping preview's pinned title and scope occupy most of its 280px panel, so the first source/capture mapping is mostly below the initial panel viewport. Labels are good, but the object of the preview is scarcely visible until scrolling.
- Integrity vocabulary includes **“dead-owner interrupted imports,” “complete unreferenced recovery objects,”** and **“retained/shared/active-baseline evidence.”** These are internal distinctions presented to somewhat tech-savvy non-developers. Similarly, approval's “inherited references” and “composite capture times” need plain-language consequences more than additional identifiers.

**Origin:** mapping is a fixed 280px disclosure (`options_ux.rs:279–330`); history/setup are outer scrollers with explanatory content and explicit actions; the full-size originals show the resulting composition. This is not a request to hide privacy consequences or drop safeguards.

**Recommendation:** retain the essential consequence nearest its action, reveal technical detail on demand, and give the item being inspected enough room before long explanatory passages. Candidate commands: `/impeccable distill`, `/impeccable clarify`.

### B8 — [P3] Empty-capture explanation inherits a blank queue column

**Evidence:** `all-820x600-capture-empty.png`; matrix records an empty `review-queue` at `(216,113,200,479)`.

The explanatory empty state starts at the viewer column rather than using the empty working surface. It leaves a large unexplained blank list-shaped area under the title. Find pages is clear and available, so this is secondary composition rather than a task blocker. `workflow.rs:672–700` reserves the queue even without latest evidence; `inventory_and_results` then has no mounted list content. Recommendation: do not reserve empty browsing chrome in the authoritative no-evidence state. Candidate command: `/impeccable layout`.

## 7. What the pixels support positively

These observations are not input/accessibility certification:

- The deep-blue persistent rail, white/light workspace, readable Linux system font and restrained semantic colours remain coherent across all screen families. Primary actions are visually distinguishable, and regular shared targets are source-backed at **40px** height (`ui/theme.rs:25`). Do not apply mobile-app thumb-zone or 44/48pt rules to this desktop assessment.
- Discovery options has a clear **Page options** title, attached Close, bounded scroll body, selected-page action and explicit Dismiss footer; the searchable list remains visible at minimum width. Result filters and mapping preview also have owned title/Close treatment. Lower content being scrollable here is intentional, not by itself clipping failure.
- Capture-only evidence correctly uses a single original, not a fake before/after pair. The failed capture names its timeout rather than calling it unchanged. Saved-baseline/unsaved-capture labels and coverage counts are present; approval remains separate from ordinary Intentional/Needs fix classification.
- Loading and read failure are visually distinguished from authoritative empty results. `all-820x600-capture-read-failed.png` offers Refresh and explicitly says saved screenshots/baseline were not changed. `loading_ux.rs:39–65` backs those distinctions.
- History inspection clearly says **inspection only / active baseline unchanged** in both recorded-check and recorded-capture contact-sheet states. Full-width selected rows are visually correct. Runs/Baselines/Storage are separate surfaces.
- ZIP export's original-screenshot privacy warning is correctly strong: ignored regions do not redact exported originals; sessions/cookies are excluded. The warning and explicit confirmation are rendered at both widths. Repeated synthetic originals mean “12 targets / 2 original PNGs” is a fixture storage fact, not a contradiction.
- Task workspaces have labelled saved/failed/skipped/remaining counts, explanatory reassurance and a fixed action footer. The narrow task body has a visible thumb, and gap Details distinguishes causes/attempts/next steps. Scrolling the body can clip the safety note at the viewport edge, but the thumb and stable footer make this legitimate bounded overflow rather than the approval row's paint defect.
- About and keyboard-help dialogs stay contained with readable titles/copy and clear Close at both widths in the contact sheets. The timezone popup is rendered above the underlying footer, with Current distinct from highlight and no popup extending beyond the image edge. Overlaying a background control is normal popup containment, not proof that its action is clickable through the popup.

## 8. Compile/render receipts and honest qualification boundary

The final receipts are successful **capture tests**, not a comprehensive test suite or a release build:

- `renderer-complete.log:27–30`: whole-app renderer capture **1 passed**, 116 filtered out.
- `task-renderer.log:26–29`: task renderer capture **1 passed**, 116 filtered out.

Earlier failures remain in the evidence root:

- `build.log:22–26`: E0599, missing `Store::preview_delete_run` at `ui_review_matrix.rs:130` in that attempted build.
- `renderer.log:37–51`: failed capture test; `Invalid/duplicate capture metadata binding` at `ui_review_matrix.rs:126`.
- Later successful receipts and verified PNG hashes support the completed gallery, not erasure of those initial failures or a claim that every build/test passes.

Warnings in successful logs:

- Two GPUI `float_literal_f32_fallback` warnings in `vendor/gpui/src/taffy.rs:312`, concerning literals at columns 68 and 77; compiler says these will become hard errors in a future release.
- `proc-macro-error2 v2.0.1` future-incompatibility warning.
- Earlier renderer logs also contain an unused `window` at `ui_renderer_capture.rs:1085`; this warning is not present in the final complete/task log output.

`all-820x600-app-settings.png` explicitly states **“Browser setup incomplete in this build”**, explains the missing trusted distribution, and shows disabled **Browser setup unavailable**. That is an honest preview-build limitation. Synthetic setup/download/task scenes do **not** prove that first-use browser installation, authentication, network/proxy/CA behavior or real website capture works in this build.

**Not verified by this assessment:** real mouse/keyboard navigation; window resize and below-minimum constraints through the OS; focus traps/restoration; clipboard/IME; screen-reader announcements or native accessibility roles/trees; high-contrast/dark appearance; OS text scaling; motion preference integration or animation smoothness; Windows/macOS glyph metrics/window chrome/prompts; actual login browser; production-like 100–1,000-page state navigation; corruption/missing file recovery through real storage; stale asynchronous replies; deletion/import/export side effects. Source implementations or static shortcut text do not constitute passes for these.

**Missing distinctive pixel coverage:** clean new-project form, visible advanced analysis parameters, visible technical capture diagnostics, narrow Overlay/Heatmap, many retained baseline versions, last-row History selection, many selected approvals, long/localized/RTL labels, native destructive/discard prompts and their keyboard containment. Some branches may be source-implemented; they were not established visually by these duplicate/limited fixtures.

## 9. Suggested synthesis order (no redesign performed)

1. Lead with **B1 final approval information containment**, because it occurs at a deliberate evidence-changing decision.
2. Connect **B2 target recognition** and **B3 readable evidence at minimum desktop width** as the main daily-review friction, with measured bounds rather than a vague “responsive” claim.
3. Preserve the positive full-width History geometry, but distinguish it from **B4 nested browsing/overflow**.
4. Qualify **B5** as a demonstrated no-frame fallback weakness, not a verified real-storage failed-comparison route.
5. Use the duplicate/fixture/build/browser/accessibility limits prominently. An empty detector result, 182 filenames and a successful offscreen renderer test are not native release qualification.

This agent returns only this saved-path receipt to the parent so deterministic findings remain isolated until Assessment A is finished.
