Method: independent Assessment B — native source, existing GPUI/Blade pixels, one detector attempt, and focused existing tests. Assessment A and earlier critique reports were not read or synthesized.

# Regressee native UX — Assessment B

**Date:** 2026-10-13 · **Platform:** Linux desktop · **Mode:** Operate  
**Reviewed commit:** `5f136b7819c3663517d2f9bd5e17f7203e5b262e`  
**Primary target:** `crates/app/src/main.rs` and its native presentation/control modules.

Short source paths in tables are relative to `crates/app/src/`; findings use fully qualified repository paths. Capture names use the `CAP/` root defined below.

## Outcome

The native working interface has a credible safety foundation: selection is distinct from disclosure, editors have reachable exits, drafts survive presentation changes, and review decisions do not silently replace baselines. The remaining confirmed problems concern note-input limits and explanatory copy, not a demonstrated broken comparison or destructive-action bypass.

- **Actionable findings:** **0 P0 · 1 P1 · 3 P2 · 0 P3**. All four are detailed below.
- **Provisional native technical score:** **13/20 — Acceptable**, with accessibility and actual desktop presentation/input still unqualified. This is not a release certificate.
- **Detector:** exit `0`, stdout `[]\n`, empty stderr; **zero reported findings, no native Rust coverage**. This is not a native pass.
- **No production edits, commits, pushes, subagents, website visits, or user-project mutations.** Only this report and its JSON receipt were authored. Existing tests used isolated synthetic temporary data.

## Evidence and boundaries

Capture root, abbreviated **`CAP/`** below:

`/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-review-20261013-9g3qdv3j/`

The existing gallery was usable; no new captures were generated. All **238** gallery PNGs were checked against `gallery-manifest.json` and their individual JSON metadata: PNG SHA-256, metadata PNG SHA-256, dimensions, and current `main.rs` binding matched. There are **119 captures at 1280×800 and 119 at 820×600**, with zero integrity errors. All **13** source-provenance hashes also match current files and the current commit matches the recorded capture commit. The additional source files supporting the note finding are fingerprinted in the receipt.

All 34 `contact-sheets/all-*` sheets and all eight `contact-sheets/task-*` sheets were visually inspected, with full-resolution spot checks for notes, analysis fields/switches, failed targets, review modes, disclosures, setup, and settings. Contact sheets establish broad state coverage; full-resolution PNGs establish the specific pixel observations below. No Assessment A inspection ledger or notes were consumed.

All 238 metadata files declare real GPUI/Blade offscreen rendering, real Linux fonts, and synthetic project data. They also explicitly declare **OS presentation unverified** and **OS input unverified**. These are authentic native-renderer pixels, not an HTML reconstruction, but neither the images nor simulated GPUI tests prove window-manager presentation, real keyboard/IME delivery, clipboard integration, or assistive-technology behavior.

### Detector and browser applicability

The bundled detector was attempted once:

```text
/var/home/olav/.config/opencode/skills/impeccable/scripts/impeccable detect --json crates/app/src
exit: 0
stdout: []\n
stderr: empty
```

Its markup-oriented rules do not scan this Rust/GPUI implementation. There are no rule names, file findings, or false positives to adjudicate. The absence of findings supplies no accessibility, geometry, performance, or platform clearance.

Browser navigation, DOM injection, console detector collection, and overlays were **inapplicable**, not successful: this target is a native GPUI app with no relevant DOM. No browser tab, local visualization server, website visit, or HTML proxy was used. No user-visible detector overlay exists.

## Five-dimension native technical table

Scores use Linux desktop equivalents: keyboard traversal/IME/AT-SPI, desktop window resizing, desktop control conventions, semantic tokens, and native rendering. iOS gestures, Android Back, phone safe areas, and mobile touch-target thresholds do not apply. Captured “Mobile” profiles describe website emulation, not Regressee's host platform.

| Dimension | Score /4 | Evidence and qualification |
|---|---:|---|
| Accessibility | **1** | Persistent field labels, visible focus outlines, radio dots/checkbox ticks, keyboard actions and simulated focus recovery are present (`ui/controls.rs:68–95,205–276`; `ui/text_input.rs:332–474`). No app-side accessibility role/state bridge was found in the inspected source; actual AT-SPI, screen-reader order/announcements, OS text scaling and OS IME remain unqualified. About already acknowledges native accessibility as pending (`form_recovery.rs:312`). This is not a claim that a tested screen reader failed. |
| Performance | **3** | Discovery/results use `uniform_list` (`main.rs:4406,4488,4562`); work and decoded-tile caching run on a worker (`jobs.rs:410–459`). The renderer accepts bounded 512-row derivatives, not full tall originals (`images.rs:13–18,135–147`), and cache payloads have explicit budgets (`viewer_cache.rs:13–24,42–54`). No startup, frame-pacing, memory or large-project benchmark was run; offscreen draw/readback timings are not OS presentation measurements (`ui/performance.rs:1–3`). |
| Appearance & theming | **3** | A coherent, intentionally light native working surface and dark navigation use shared semantic colors, control metrics and a Linux font (`ui/theme.rs:4–39`), with contrast assertions in `ui/theme.rs:63–87`. Captures show readable controls and consistent selected/focused states. No exhaustive painted-state contrast audit or OS high-contrast test was performed. No app dark mode or theme selector is required by this assessment. Light/dark **website capture** is existing configuration, not app theming. |
| Platform conformance | **3** | The app reads as desktop tooling, not a ported website: native text/IME handlers, keyboard navigation, named menu/dialog exits, and focus restoration. No phone gestures are missing requirements. Real desktop input/OS prompts and AT remain unqualified; the note limit defect is a concrete custom-control inconsistency. |
| Adaptivity | **3** | Both requested window sizes retain primary workflow controls. Navigation and results can collapse independently; inspectors and option panels have bounded scroll owners (`workflow.rs:349–386,512–520`; `viewer_ux.rs:225–277`; `options_ux.rs:115–148`). Compact review omits useful disabled-action explanations (B-03). The claimed Mobile-reveal screenshot is insufficient evidence (Q-03). Smaller windows, fractional/high DPI and enlarged system text were not qualified. |
| **Total** | **13/20** | **Acceptable, provisional; significant qualification work remains.** |

**Platform verdict:** visually and structurally native desktop implementation, **not an OS-qualified pass**. The main gap is native qualification, not an obligation to replace the chosen visual identity or introduce mobile patterns.

## Severity-ranked actionable findings

### B-01 · P1 — Review notes have incompatible limits; valid larger notes cannot be normally edited

**Category:** control correctness / error prevention / desktop input consistency.  
**Evidence level:** confirmed source behavior, supported by existing input tests; the large-note edge case was not reproduced with OS input.

**Exact locations:**

- `crates/app/src/main.rs:372–378`: note field index 9 is created with the generic `TextInput::with_value`; no note-specific limit override is present.
- `crates/app/src/ui/text_input.rs:54–72`: generic maximum is **4,096 UTF-8 bytes**.
- `crates/app/src/ui/text_input.rs:74–84`: programmatic loading preserves the supplied value; it does **not** truncate to that maximum.
- `crates/app/src/ui/text_input.rs:367–400,418–423`: edits exceeding `max_bytes` return silently, including deletions whose resulting value remains over the limit.
- `crates/app/src/review_ux.rs:426–440`: saved notes are loaded through `set_value`.
- `crates/storage/src/review.rs:323–326`: storage accepts **8,192 bytes**.
- `crates/app/src/review_ux.rs:220–223`: exit validation instead counts **8,192 characters**; `main.rs:1911–1916` independently takes up to 8,192 characters when saving a review.
- `crates/app/src/viewer_ux.rs:185–188`: the visible note help says “Draft until saved,” without the input limit or rejection feedback.

**Impact:** a paste exceeding 4,096 bytes silently does nothing. A storage-valid 5,000-byte note can load intact, but ordinary typing, one-character replacement, or even a single Backspace still results in more than 4,096 bytes and is rejected. Bulk-selecting enough text to get below the generic limit is the workaround. Unicode makes the character/byte disagreement harder to understand. This is **silent rejection and inconsistent editability**, not demonstrated truncation or loss of an existing saved note.

**Screenshot/state:** `CAP/all-820x600-review-note.png` shows a usable 40px single-line field with horizontally scrolled text, Save note and Return to evidence. It establishes the actual field and missing limit explanation; its short synthetic note does not exercise the size boundary. `phase4_review_tests.rs:71–154` checks geometry/caret/draft recovery, not the conflicting note limits. The existing library input test at `ui/text_input.rs:908–964` confirms rejected oversized edits preserve state.

**Fix:** use one shared **byte-based storage-compatible note limit** in the editor, explicit save and exit guard. Permit corrective shrinking of a loaded oversized value. Explain the limit and announce/display rejected paste or invalid save without changing the draft. Remove independent character truncation as a validation policy; validate and preserve exact text instead. A multiline redesign is not required to fix this defect.

**Acceptance:** test 4,096/4,097 and 8,192/8,193-byte notes, multibyte text, a loaded 5,000-byte saved note, ordinary deletion, selection replacement, paste, explicit Save and save-on-exit. Valid text round-trips exactly; invalid text stays visible with an actionable explanation. **Suggested command:** `/impeccable harden`.

### B-02 · P2 — Setup promises automatic browser installation even when this build cannot provide it

**Category:** truthful status / setup recovery.  
**Evidence level:** confirmed pixel/copy contradiction and source condition mismatch.

**Exact locations:** `crates/app/src/main.rs:4033–4034` unconditionally says “The private browser is installed automatically when needed.” The status-aware presentation at `main.rs:5935–5973` explicitly handles unavailable distribution, unsupported platform, development-only scope and repair-required states, and correctly qualifies first-use installation with “A supported build…”.

**Screenshot/state:** `CAP/all-1280x800-access-status-loading.png` displays the unconditional installation promise in setup; `CAP/all-820x600-project-settings-bottom.png` shows it in project settings. `CAP/all-820x600-app-settings.png` says “Browser setup incomplete in this build” and explains that automatic setup has no trusted distribution configured.

**Impact:** a first-time user is reassured at the point of setup but must visit another section to discover that the current build cannot fulfill that promise. The missing trusted distribution is an **existing preview limitation**, not a newly discovered backend defect; the inconsistency in user-facing copy is the actionable issue.

**Fix:** share the existing runtime-readiness explanation with setup. At minimum qualify the sentence with “In supported builds”; when readiness is known, show the actual unavailable/development-only/repair state before a browser-dependent action. Continue allowing local project saves and retained-evidence inspection. Do not promise unrestricted public-site capture or install developer tools as a workaround.

**Acceptance:** setup and App settings agree for checking, not-installed, ready, development-only, repair-required, unavailable-distribution and unsupported-platform states. Saving locally still does not visit a website. **Suggested command:** `/impeccable clarify`.

### B-03 · P2 — Compact review hides the reason “Next unreviewed” or review decisions are disabled

**Category:** adaptivity / state explanation / recovery.  
**Evidence level:** confirmed source omission for a retained-draft edge state; not an OS-reproduced interaction.

**Exact locations:** `crates/app/src/review_ux.rs:714–717` renders both “Save your note and ignored-area drafts…” and the comparable-analysis prerequisite **only when `!narrow_workspace`**. `crates/app/src/main.rs:5324–5331` still disables those controls at narrow width. `review_ux.rs:549–555` independently prevents Next when drafts are dirty. `ui/controls.rs:110–117` gives disabled actions opacity but no explanatory feedback.

**Impact/state:** at 820×600, write a note, return to evidence without saving, and retain another unreviewed result. The draft is safely kept, but Next unreviewed is disabled without the explanation available at 1280×800. The user has to infer that reopening Review note and saving will unblock navigation. The same omission applies to unresolved analysis/edit prerequisites. Correct safety guards should remain; their reason should not disappear to save space.

**Screenshot/state:** `CAP/all-820x600-review-note.png` explicitly promises Close preserves text. `CAP/all-820x600-review.png` establishes the compact decision/action layout; it is a clean-state image, not the dirty-note reproduction. `phase4_review_tests.rs:402–447` confirms draft preservation after dismissal; the enabled-state and omitted-copy branches establish the narrow dirty-state finding. `CAP/all-820x600-review-failed-target.png` already has a good failure-specific explanation, so this is **not** a claim that every disabled state is unexplained.

**Fix:** expose one concise, highest-priority blocking reason at both widths, close to the affected action. Distinguish unsaved note/area edits, running or noncomparable analysis, and no remaining matching unreviewed result. Keep dirty drafts, current selection and baseline eligibility unchanged; do not enable unsafe navigation to avoid explaining it.

**Acceptance:** at both sizes, dirty note, unapplied geometry, pending analysis, noncomparable result and exhausted queue have distinct, truthful reasons and a discoverable next step. **Suggested command:** `/impeccable clarify`.

### B-04 · P2 — The production heatmap omits the color meaning that the test-only viewer describes

**Category:** comparison interpretation / recognition / redundant color explanation.  
**Evidence level:** confirmed at both requested sizes and in the production rendering branch.

**Exact locations:** `crates/app/src/main.rs:6359` uses `viewer_panes_layout(view, true, …)`. In that bounded production path, the caption comes from `crates/app/src/workflow.rs:883–890`, which returns only “Heatmap.” The useful “Heatmap · red significant, blue ignored” label exists in `main.rs:7138–7156` only in the other branch; the wrapper calling the unbounded path is test-only (`main.rs:6802–6808`).

**Screenshot/state:** both `CAP/all-820x600-review-heatmap.png` and `CAP/all-1280x800-review-heatmap.png` show the Heatmap label and red rendered differences, with ordinary review decisions available, but no legend. The current fixture does not show blue ignored pixels; that meaning is established by source, not by pretending this image demonstrates it.

**Impact:** a reviewer has to guess what a derived color means, especially whether it denotes a significant difference or ignored coverage. The distinction matters when deciding intentional versus needs-fix. This is not evidence that originals are tinted, that the comparison algorithm is wrong, or that red automatically means a defect.

**Fix:** show a compact textual legend in the active heatmap caption: significant differences versus ignored pixels, with labeled swatches if useful, and a brief indication that the heatmap is derived evidence. Do not rely solely on color or move the legend into technical diagnostics. Keep untouched originals available through Compare/Before/After.

**Acceptance:** both sizes show the legend without obscuring the evidence or decision controls; include an ignored-area fixture so both meanings are visibly represented. **Suggested command:** `/impeccable clarify`.

## Interaction and geometry coverage

| Surface | What the evidence supports | Source / representative captures |
|---|---|---|
| Selected versus expanded | Checkbox ticks and radio dots communicate persistent choice; Details/Analysis/Page options communicate expanded state separately. Labels generally remain stable when selection changes. Selected, hover and focus paint do not change shared dimensions. | `ui/controls.rs:24–61,68–100,205–276`; `main.rs:6332–6347`; `CAP/all-820x600-pages-images.png`, `CAP/all-820x600-review-analysis-field-8.png`. |
| Page options/result filters | Bounded independent body scroll, pinned Close, and a separate pinned Dismiss page list action. Opening filters closes competing editors; Close/Escape return focus to the opener without changing evidence, inclusion or drafts. | `options_ux.rs:99–148`; `main.rs:3285–3297,4663–4666,5041–5044`; `CAP/all-820x600-discovery-options.png`, `CAP/all-820x600-review-filters.png`, `CAP/all-820x600-review-no-matches.png`; four passing options tests. |
| Mapping preview | 280px bounded panel; first-16 scope is explicitly disclosed. Close/Escape restore the saved-settings preview opener; status refresh is not permission to overwrite newer environment drafts. | `options_ux.rs:281–332`; `main.rs:4672–4676,5045–5049`; `CAP/all-820x600-mapping-preview.png`; `options_ux_tests.rs:171–309,313…`. |
| Details/Analysis/note | Pinned Return to evidence, independent body scroll, finite input width. Close/Escape retain notes and area edits. Editor state removes review/baseline decision controls from activation; it is not just a visual overlay over active destructive buttons. | `viewer_ux.rs:169–188,225–277`; `review_ux.rs:618–640,646–649`; `main.rs:5054–5068,5458–5462`; `CAP/all-820x600-review-note.png`, `CAP/all-820x600-review-ignored-area.png`, `CAP/all-820x600-review-diagnostics-tail.png`; eight passing review tests. |
| Review modes/navigation/results | Compare, Difference, Overlay, Heatmap and full-width Before/After are distinct. The chooser supports arrows/Home/End and closes back to View. Hiding results retains evidence identity; Ctrl+F reopens local search. Native input supports image pan/zoom actions in source/simulation. | `main.rs:3208–3247,6407–6467`; `workflow.rs:291–321,377–386,512–520`; `viewer_ux.rs:78–165`; `CAP/all-820x600-review-mode-chooser.png`, `CAP/all-820x600-review-full-width-before.png`, `CAP/all-820x600-review-full-width-after.png`, `CAP/all-820x600-review-queue-hidden.png`; `phase4_review_tests.rs:380–465`. |
| Guided setup | Saves are distinct from login/discovery. Continuations are request/draft-bound; validation keeps the stage and reveals the actual field/error/help. Public access can continue while separate browser status is absent; that is not by itself a broken login gate. | `setup_ux.rs:26–65,67–107`; `workflow.rs:1406–1528`; `CAP/all-820x600-new-website.png`, `CAP/all-820x600-website-validation.png`, `CAP/all-820x600-pages-invalid.png`, `CAP/all-820x600-access-login-actions.png`; five passing repair tests. |
| History | Runs/Baselines/Storage are separate selected sections. Search is explicitly limited to the loaded page, with Clear and next-page controls. Exact provenance is optional. Recorded checks/captures are inspection-only; current baseline actions are not inferred from historical screenshots. | `workspace_history.rs:444–649,662–709,713–737`; `main.rs:5464–5469`; `CAP/all-820x600-history-no-matches.png`, `CAP/all-820x600-history-bottom.png`, `CAP/all-820x600-history-recorded-check.png`, `CAP/all-820x600-history-recorded-capture.png`; two passing history tests. |
| Sharing/cleanup/task feedback | Export warns that originals may contain private pixels and masks are not redaction; login sessions/cookies are excluded. Deletion is previewed and distinct from repair/recovery cleanup. Activity distinguishes stopping from stopped/finished and preserves the baseline promise. | `main.rs:2381–2436`; `workspace_history.rs:434–438,620–621,701–703`; `CAP/all-820x600-export-preview.png`, `CAP/all-820x600-delete-preview.png`, `CAP/all-820x600-integrity-preview.png`; `CAP/task-820x600-capture-pausing.png`, `CAP/task-820x600-partial-discovery.png`, `CAP/task-820x600-export-late-success.png`. |

**1280×800:** controls, queue and evidence panes remain bounded; advanced parameter and diagnostic tails have separate reachable scroll positions. The inspector deliberately occludes the right evidence pane while editing. Decision controls are removed until return; this is not comparison clipping masquerading as approval safety.

**820×600:** navigation defaults collapsed in review, results remain independently collapsible, and full-width Before/After provides a larger original. A fit-width desktop capture at about 19% is an overview, not readable body text; actual-size/manual zoom and panning are available. This is not proof of silently cropped capture content. Setup's pinned save/discovery footer remains visible while the long form scrolls. Advanced switch labels wrap within the panel. Claims about scroll/reveal usability are limited as noted in Q-03.

## Existing qualification limits and evidence corrections — not new product defects

### Q-01 · Native accessibility and actual desktop input remain unqualified

About already says native accessibility, cross-platform and public-release qualification are pending (`form_recovery.rs:312`). The inspected source does not establish an AT-SPI semantic tree, state announcements or system-text adaptation. GPUI tests demonstrate simulated focus/selection/control behavior only. Test names containing “native” do not turn simulated input into OS input. Actual Linux screen reader, keyboard-only traversal, focus restoration, IME, clipboard, native file prompts and scaled-font behavior need a bounded real-desktop qualification run before making those claims.

The app has an explicit reduced-motion environment override (`main.rs:7884–7885`; `ui/controls.rs:7–11`); it does not demonstrate automatic reading of an OS motion preference. No missing mobile gesture or app-theme feature is filed.

### Q-02 · Preview/runtime constraints are real, already surfaced limitations

The gallery's incomplete browser distribution, development-local-fixture restriction, unqualified enterprise networks and macOS process-ownership gap are not new defects discovered by this review. The report does not declare public-site capture, secure-store behavior on every OS, or public release ready. B-02 concerns contradictory copy about the existing runtime restriction, not a request to remove it.

### Q-03 · Authentic captures do not prove every named interaction/state

- At **820×600**, `pages-mobile-revealed` is byte-identical to `pages-sizes`; the Mobile checkbox is below the pinned footer/body boundary and not shown. That image cannot be cited as a successful Mobile reveal. At 1280×800 the identical pair is compatible with already-visible content, not proof of a distinct interaction. The relevant reveal recipe is `ui_review_matrix.rs:1021–1044`; the visibility assertion is conditional at `1218–1258`. Existing focused setup tests support controller-level reveal behavior, but do not repair the mislabeled screenshot's evidentiary gap.
- At 820×600, `access-status-loading` is byte-identical to `access-public`. The recipe removes environment status (`ui_review_matrix.rs:760`), but public access can legitimately continue (`setup_ux.rs:48–65`) and the compact viewport does not expose an additional loading message. Do not claim this image validates private-login loading/recovery.
- At 1280×800, `review` and `review-navigation-open` are identical because navigation is already open; the recipe toggles only when collapsed (`ui_review_matrix.rs:769`). This is harmless duplication, not a failed navigation implementation.
- At both sizes, `review-analysis-advanced` and `review-analysis-field-0` are identical top-field views. Subsequent field captures show different offsets; the duplicate does not add interaction coverage.
- Section/disclosure tails are sometimes positioned by direct scroll offsets (`ui_review_matrix.rs:906–924,1046–1063`). These images prove painted content at constructed positions, not that a person used a mouse/keyboard to reach those positions. The passing focused tests are the separate simulated interaction signal.

No evidence files were altered to resolve these gaps. A future verification pass should correct labels or add explicit post-action state/focus/current-scroll-owner evidence, rather than treating filenames or whole-window bounds as proof.

## Focused test evidence

Common environment: `LIBRARY_PATH="$PWD/.runtime/link-libs"` and `TMPDIR=/var/home/olav/Data/opencode/tmp/opencode`. Tests ran locked/offline, with all features and one test thread. No full-suite pass is claimed.

| Cargo target/filter | Result | Qualification |
|---|---|---|
| `--bin regressee options_ux_tests::` | **4 passed, 0 failed** | Disclosure bounds, close/focus, draft-preserving filters and environment refresh. |
| `--bin regressee phase4_review_tests::` | **8 tests, successful exit** | Review geometry, note caret/draft restoration, navigation/read states, modes and consent. The displayed aggregate output was truncated; successful completion and the eight-test run were observed. |
| `--bin regressee phase5_repair_tests::` | **5 passed, 0 failed** | Validation, composition, asynchronous save-continuation/draft races and simulated setup-control reveal. |
| `--bin regressee phase6_history_tests::` | **2 passed, 0 failed** | Mounted exact-details actions and missing/corrupt comparison originals fail locally. |
| `--lib ui::text_input::tests::` | **8 passed, 0 failed** | Unicode/UTF-16 composition, rejection preservation, selection, clipboard normalization and caret scroll. This validates existing primitives, not a new regression test for B-01. |

The initial attempt to run `ui::text_input::tests::` on the **binary** matched zero tests (139 filtered); it supplied no test coverage. The corrected **library** target ran the eight tests listed above. All commands completed without a new build failure; Cargo emitted the existing `proc-macro-error2 v2.0.1` future-incompatibility warning.

## Positives to preserve

1. **Control state and exits are engineered, not cosmetic.** Stable labels, drawn selection marks, focus outlines, pinned Close/Return actions, explicit mount/enabled checks and draft-preserving dismissal work together. Do not replace these with overlays that leave hidden controls active.
2. **Evidence and acceptance remain separate.** Untouched before/after images stay available; derived modes are explicit. Missing captures are not unchanged results. Baseline preview identifies replaced/kept screenshots and exact provenance, and confirmation is separate from review disposition. Masks are capture-pair-bound, not stretched onto new evidence.
3. **Risky promises are generally unusually clear.** Discovery is bounded rather than exhaustive, history search admits its page scope, imports are inspection-qualified, ZIPs expose original privacy risk, manual cleanup has previews, and cancellation/late success use distinct truthful states. Fix the specific remaining contradictions without burying these useful explanations.

## Finite implementation phases

These are recommendations, not changes made during this assessment. All in-scope findings are already authorized; no scope/priority question is necessary.

1. **Note correctness — B-01.** Share a byte-based limit, add rejection feedback/corrective editing, and add the enumerated boundary/Unicode/save/exit regression tests. Exit when exact round-trips and retained-invalid-draft assertions pass; do not redesign unrelated inputs.
2. **Explanation consistency — B-02 through B-04.** Reuse runtime state copy, keep one compact disabled-action reason at both sizes, and put the heatmap legend in the production caption. Check the finite runtime/review state sets above; preserve controls, guards, palettes and baseline semantics.
3. **One bounded verification/qualification pass.** Correct the Mobile-reveal/loading evidence labels; capture the changed surfaces together at 1280×800 and 820×600, including dirty-note and blue-ignored heatmap states. Re-run the affected existing tests and new limit tests. Separately record actual Linux desktop input/accessibility/scaling outcomes before making OS claims. Review one batch, fix demonstrated regressions together, confirm once, and stop. Do not regenerate the entire gallery or launch an open-ended polish loop.

No backend comparison redesign, dark-mode/theme-selector work, mobile gestures, automatic cleanup, public-site visits, or restriction bypass is introduced by these phases. Finish with a focused `/impeccable polish` only after correctness and state explanations are resolved.

**Questions skipped: the user already authorized all discovered in-scope issues and requested no scope/priority questions.**

**Receipt:** `docs/research/ux-review-20261013/assessment-b-receipt.json`.
