# Regressee UX improvement plan — 2026-10-13

## Goal and approved direction

Make the native desktop app easier to set up, inspect and customize. Replace the
wizard's disconnected `Current / Unavailable` buttons with a connected progress
component, remove `Private, local preview` branding, and resolve the actionable
findings from a fresh whole-app screenshot review. This is an **Operate-mode
refinement**, not a redesign. Preserve **Clear wayfinding**: deep-blue navigation,
white working surfaces, sky selection, system UI typography, shared native
controls and explicit next safe actions.

The user authorized this entire plan, sequential implementer/verifier delegation,
and a commit and push after every verified phase. No further scope interview is
needed. This file is the execution contract and progress record; the older
`PLAN.md` remains the product/release plan.

## Review evidence

- Starting commit: `5f136b7819c3663517d2f9bd5e17f7203e5b262e` on `main`.
- Fresh baseline: **238 Linux GPUI/Blade offscreen app screenshots**, 119 each at
  **1280×800** and **820×600**; synthetic isolated projects and offline website
  imagery. No user projects or public websites were accessed.
- Private evidence root:
  `/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-review-20261013-9g3qdv3j`.
  It contains unmodified PNG/JSON pairs, source provenance, proof, manifests,
  gallery, contact sheets and capture commands. Do not commit pixels/databases.
- Independent design assessment: `docs/research/ux-review-20261013/assessment-a.md`
  and `inspection-ledger-a.md`, 26/40 expert heuristic score.
- Independent technical assessment:
  `docs/research/ux-review-20261013/assessment-b.md` and its JSON receipt, 13/20
  provisional native score. The markup detector returned no findings but **does
  not cover Rust/GPUI**. There is no browser DOM or detector overlay for this app.
- Synthesis: `docs/research/ux-review-20261013/synthesis.md`.

Scores are review judgments, not usability measurements. Offscreen rendering and
simulated input do not establish real OS input, IME, assistive technology,
high-DPI/high-contrast, macOS/Windows or release qualification. Existing gaps stay
documented; this plan does not claim to implement a cross-platform accessibility
bridge or a new runtime distribution.

## Mandatory execution protocol

1. Run **one implementer subagent for one phase**, with no other agent active.
   Give it this complete plan, phase scope and incumbent evidence. It owns all
   implementation and repairs for that phase. Agents must not spawn child agents.
2. After implementation, run a **separate verifier subagent**. It independently
   inspects current source, relevant native screenshots, controller/geometry
   assertions and tests, and writes a PASS or FIX verdict with concrete evidence.
3. Send every FIX finding back to the **same implementer session**. Have the same
   verifier re-check the repairs. Do not progress, commit or push until it gives
   a scoped PASS. Structural tests and compilation corrections may run as needed;
   visual polishing is bounded to the planned inspection batch, one collective
   correction batch and at most one visual confirmation.
4. Update this file and phase reports, inspect the diff, run whitespace checks,
   **commit the phase and push to `origin/main`**. Never overwrite unrelated work,
   force-push, or commit private evidence. If a push fails, diagnose and retry a
   normal push; do not claim success without the actual result.
5. Start the next implementer only after the preceding phase is verified and
   pushed. Continue until all five phases are complete.

All UI-editing agents must load the **impeccable** skill and its context once per
session, inspect the incumbent native pixels/source, and read
`reference/craft-floor.md` immediately before UI edits. The product metadata's
adaptive/mobile routing mismatch is already known: this is a desktop app, not
iOS/Android. Do not repair unrelated context drift or replace its visual world.

## Invariants for every phase

- No automatic website visit, login, discovery, baseline activation or deletion
  from presentation/navigation changes. Saves and consequential actions remain
  explicit; clean Continue must not rewrite saved settings.
- Preserve draft/request/project/environment identities and asynchronous reply
  guards. Newer drafts must survive status reads, menu changes, close/reopen,
  window resize and target-navigation guard checks.
- Keep originals untinted. Derived modes and ignored coverage must be labeled.
  Missing/failed/noncomparable evidence cannot become unchanged or approvable.
- Preserve fixed baseline inventories, historical environments, source-bound
  ignored areas, saved analysis revisions and retained baselines/history.
- ZIPs never include sessions/cookies; screenshots may expose private pixels;
  ignored rectangles are not redaction. Deletion remains previewed and deliberate.
- Only mounted/enabled controls participate in focus traversal. Close/Escape
  restore their triggers; hidden/occluded mutations cannot activate.
- Keep I/O off the UI thread, lists bounded/virtualized, and use shared semantic
  tokens. No new app theme selector, decorative motion, mobile UI or backend
  comparison algorithm is requested.

## Phase 1 — Guided setup and product language

**Status: complete — independently verified PASS**

Implementer handoff: `docs/research/ux-review-20261013/phase1-implementation.md`.
Independent initial **FIX** and subsequent **PASS** are preserved in
`docs/research/ux-review-20261013/phase1-verification.md`. The same implementer
repaired V1-1/V1-2 and the same verifier accepted the entire bounded phase.
Four fresh-entry captures bind the repairs; the prior 60 captures remain
historical pre-repair visual evidence. Phase commit/push is performed by the
parent after this update; its hash is recorded in the next phase.

Findings: **A1, A5, B-02**.

### Implementation

- Build a reusable native connected stepper: **1 Website → 2 Access → 3 Pages &
  sizes**. Visually distinguish current, completed and upcoming stages with
  numbering/drawn marks and words, not disabled button styling or color alone.
  The current stage stays current when its draft is invalid. Future gated stages
  are normal upcoming progress, not `Unavailable`; give a concise prerequisite
  explanation beside the current action. Retain the actual save/login gates,
  safe back navigation and focus behavior.
- Show multi-website authorization only when multiple distinct permitted origins
  make it relevant. Preserve backend consent and consent reset when origins change.
- Lead Access with the public/login decision and required login workflow. Place
  optional staging/environment management, mappings/probes and rendering options
  behind a clearly named disclosure. Required provider/session options stay
  discoverable when login needs them; general Project settings remains complete.
- Keep Pages essentials and a concise capture-plan summary ahead of advanced
  scope/speed controls; do not remove custom profiles or capture limitations.
- Remove the rail's `Private, local preview` slogan with no replacement slogan.
  Rename `About this preview` to **About Regressee**. Preserve factual build/version,
  runtime constraints, local storage and sharing warnings in their proper context.
- Make setup's browser-availability explanation agree with the runtime state used
  in App settings. Never unconditionally promise installation in an unavailable,
  unsupported, repair-required or development-only build. Local saving still works.

### Likely ownership

`crates/app/src/{workflow,setup_ux,main,form_recovery}.rs`, shared UI controls,
`crates/core/src/shell.rs`, focused setup tests and capture recipes.

### Acceptance and verification

- Both sizes: blank website, one/multiple origins and invalid Website/Pages;
  completed/back/current step states; public Access; private unsaved/saved/browser
  open/ready; optional environment disclosure open/closed and keyboard reveal.
- State tests cover checking/not-installed/ready/development-only/repair/unavailable/
  unsupported runtime copy without altering capture authorization.
- Save/login gates, dirty-draft retention and mounted Tab/Enter actions pass.
- No rail preview slogan or `About this preview` entry. Supported-build factual
  caveats remain. Review one bounded native screenshot batch for changed surfaces.

**Commit suggestion:** `Improve guided setup and remove preview branding`

## Phase 2 — Evidence workspace, menus and action explanations

**Status: pending**

Findings: **A2, B-03, B-04**, A minor baseline-eligibility ambiguity.

### Implementation

- Consolidate target/profile/environment identity into one stable inspection
  heading. Put Show/Hide results in existing toolbar geometry, not an extra row.
  Compact full-width Before/After and secondary actions before they wrap away the
  viewer. Preserve identity with hidden queue, historical records and long paths.
- At 820×600, hiding Results must **not reduce evidence height** compared with the
  same clean Compare state. Full-width evidence must not become the current
  ~98px masthead letterbox. Prefer equivalent-or-greater height than default
  Compare. Measure actual painted pane bounds, not hand-estimated containers.
- Anchor the View menu to its trigger, use consistent full-width selected rows,
  clamp within the window and retain arrow/Home/End/Enter/Escape/Tab behavior.
- Display one concise highest-priority reason for blocked review/Next actions at
  both sizes: unsaved note/ignored geometry, pending analysis, noncomparable
  evidence, or exhausted matching queue. Preserve guards; never bypass them.
- Add a production Heatmap legend for **significant differences** and **ignored
  pixels**, making clear it is derived evidence, not confirmed defects. Check an
  actual ignored-area fixture showing both colors; leave originals unchanged.
- Empty baseline selection reads `0 selected`, not `0 eligible` as though none
  can be selected. For a selection, explicitly describe selected references ready
  for update. Keep eligibility calculation and approval confirmation unchanged.

### Likely ownership

`crates/app/src/{workflow,main,review_ux,viewer_ux}.rs`, selector patterns in
`ui/zoom_select.rs`, core focus controls and review tests/capture recipes.

### Acceptance and verification

- Measure default, queue hidden, navigation open/collapsed and full-width modes at
  both sizes, with a long target and recorded environment. No identity loss.
- View and Zoom menus remain attached, bounded and keyboard-dismissable.
- Dirty-note/area, pending/noncomparable/exhausted states explain recovery at
  820×600 and 1280×800. Compare, Difference, Overlay, Heatmap and Before/After
  remain truthful; only the active derived mode displays the legend.
- Existing draft, review-disposition and baseline-safety regressions pass.

**Commit suggestion:** `Give review evidence more room and clarify controls`

## Phase 3 — Readable, lossless review notes

**Status: pending**

Findings: **A3, B-01**.

### Implementation

- Provide a real bounded multiline native note editor with wrapped text, several
  visible lines, caret/selection/vertical scrolling and pasted line breaks.
  Do not merely increase the generic single-line input height. Preserve existing
  single-line fields and their shortcuts/clipboard/IME semantics.
- Use one shared **8,192 UTF-8 byte** storage-compatible note limit across editor,
  explicit Save and save-on-exit validation. Remove independent character-based
  truncation; save exact text or reject explicitly while preserving the draft.
- Loaded valid 5,000-byte notes remain ordinarily editable. Loaded oversized text
  permits corrective shrinking. Rejected insertion/paste provides actionable
  visible feedback, never silently ignores or truncates the user's text.
- Keep one clear draft/saved/invalid status beside Save. Show scroll hints only
  when the actual editor/disclosure body overflows. Closing/reopening retains the
  note draft and returns focus without review/baseline decisions behind the editor.

### Likely ownership

`crates/app/src/ui/text_input.rs` or a note-specific native component,
`crates/app/src/{main,review_ux,viewer_ux}.rs`,
`crates/storage/src/review.rs`, core review constants and focused tests.

### Acceptance and verification

- Exact round trips and edit/save/exit boundary tests: 4,096/4,097 and 8,192/8,193
  bytes, multibyte Unicode, pasted newlines, selection replacement, Backspace/
  Delete, loaded 5,000-byte and oversized legacy text. Invalid drafts stay visible.
- Test caret/selection, vertical keyboard movement, clipboard and simulated IME
  where supported. No claim of OS IME qualification from GPUI simulation.
- Both sizes: realistic two-sentence note, multiline note, long scrolled note,
  limit feedback, Save, Return/Escape, reopening and target/draft ownership.
- Existing generic input and review-draft tests still pass.

**Commit suggestion:** `Add multiline notes with consistent lossless validation`

## Phase 4 — Progressive controls and secondary-page hierarchy

**Status: pending**

Findings: **A4**, A minor History/Settings repetition and terminology findings.

### Implementation

- Lead Analysis with a concise actual-result/current-settings explanation and a
  named Sensitivity group. Presets precede masking; edits/reanalysis consequences
  are clear. Give Ignored areas its own disclosure containing Draw and the
  keyboard coordinate alternative. Keep advanced parameters/metrics below the
  common choices, and show Cancel/previous analysis only when meaningful.
- Preserve pair-bound mask edits, unapplied/all-masked validation, revision
  retention and mounted-control/focus reveal in the correct inner scroll owner.
- Condense repeated History search/help/section prose while preserving its loaded-
  page scope. Improve selected-record/action proximity and reduce fixed empty
  list space for few records without losing bounded long-list/pagination behavior.
- Align the App settings heading with its navigation label. Consistently explain
  `No significant change`/`Unchanged` as below thresholds, not identical pixels.
- Reduce redundant reassurances, keeping one task-level state statement and
  necessary local consequences. Preserve ZIP privacy, recipient-login, permanent
  deletion and retained/shared-source protections, exact details on demand.

### Likely ownership

`crates/app/src/{main,review_ux,viewer_ux,workspace_history,workflow}.rs`,
`crates/core/src/shell.rs` and existing focused review/history tests.

### Acceptance and verification

- Both sizes: Analysis basic/presets/ignored-area disclosure/coordinates/advanced
  field and tail/active reanalysis/previous revision, including actual mounted
  keyboard reachability and safe close/reopen behavior.
- History zero/few/many records, selected record, loaded-page search/no matches,
  pagination and optional provenance; sharing/deletion preview warnings preserved.
- App settings hierarchy and review classification copy agree without invented
  features. Existing analysis, mask, history and consent tests pass.

**Commit suggestion:** `Simplify analysis and secondary workspace controls`

## Phase 5 — Interactive-state coverage and final regression gate

**Status: pending**

Findings: A/B evidence gaps and final integration coverage for Phases 1–4.

### Implementation and evidence

- Repair/extend opt-in capture recipes so names describe asserted actual states:
  Mobile control revealed in its real clip; invalid Advanced field revealed;
  private-login status loading; full custom-profile/limitations tail; selected
  alternate image/screen-size/extent/speed choices; post-inclusion discovery;
  Changed/Gaps/Unchanged/grouped filter outcomes; network limitations; expanded
  portability details and long History selection. Use supported control activation
  and mounted/state/visibility assertions rather than filenames as proof.
- Produce one targeted final source-bound native batch at both desktop sizes,
  covering changed components and newly qualified toggle/customization states.
  Reuse synthetic website fixtures, not stale app PNGs. Preserve all baseline and
  failed-attempt evidence; use a fresh private 0700 output root per new batch.
- Run complete workspace/all-features tests, UI build, all-targets/all-features
  Clippy with `-D warnings`, formatting and whitespace checks. Feature-matrix
  checks should catch accidental coupling between base/UI/native test targets.
- Write a final integration report, durable source/hash/geometry/state receipt and
  screenshot inspection ledger. Update the relevant **DESIGN.md** behavior sections
  to reflect shipped components without unrelated drift repairs or release claims.

### Acceptance and verification

- All preceding verifier reports are PASS. Final verifier independently checks
  actual captured states, source binding/integrity, native layout and safety tests.
- Every in-scope A/B finding has an explicit resolved finding-to-test/evidence map.
  All plan requirements are implemented; no hidden TODO substituted for a fix.
- If real OS input/AT/platform qualification cannot be performed here, record the
  exact limit honestly as an existing release obligation, not a failed new feature
  or a claim that screenshots establish a pass. Do not expand this phase into
  unsupported platform development or visits to real websites.
- Final report distinguishes **implemented/verified in bounded scope** from
  **native-input/accessibility/release qualification still pending**.

**Commit suggestion:** `Verify interactive UX states and document refined workflows`

## Reproducible commands and artifact policy

From the repository root, for this Linux development host:

```sh
export LIBRARY_PATH="$PWD/.runtime/link-libs"
export TMPDIR=/var/home/olav/Data/opencode/tmp/opencode
cargo test --locked --offline --workspace --all-features -- --test-threads=1
cargo build --locked --offline --workspace --features ui --bins -j 4
cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings
cargo fmt --all -- --check
git diff --check
```

Opt-in screenshot commands require a **new**, canonical 0700 directory under the
approved scratch root. Inspect `ui_renderer_capture.rs` before running. Generate
and personally inspect `renderer_capture_proof` before writing `proof-reviewed.json`;
reuse only clearly labeled website-fixture imagery. Common renderer environment:

```sh
export REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode
export REGRESSEE_RENDERER_CAPTURE_DIR=<new-private-0700-evidence-root>
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  <exact-opt-in-test> -- --ignored --exact --test-threads=1
```

Read the existing test's state switches before using selective runs; do not
silently skip captures based on old filenames. Logs/private screenshots belong
under scratch or ignored `.impeccable/`/`artifacts/`. Commit only application/test
source, reproducible tooling and small scrubbed reports/receipts. No credentials,
user URLs, user project databases or generated shipping claims.

## Execution record

| Phase | Implementer | Verifier | Verdict | Commit / push |
|---|---|---|---|---|
| 1 | `ses_ec954afe7ffeEecoyP5c94slJU` — initial + repairs | `ses_ec9177899ffet7yjIRWlsr1DW8` — FIX → PASS | PASS | phase commit; hash/push recorded next phase |
| 2 | pending | pending | pending | pending |
| 3 | pending | pending | pending | pending |
| 4 | pending | pending | pending | pending |
| 5 | pending | pending | pending | pending |

The phase commit cannot contain its own hash. Record a preceding phase's hash in
the next phase's update; final delivery can name the final hash explicitly.
