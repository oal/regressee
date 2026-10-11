# UI critique improvements — 2026-10-11

**Initial outcome: compiler/regression gates passed; visual acceptance was incomplete.**
The bounded renderer inspection found remaining narrow-layout defects and two
incorrect capture-only fixture states. This report does not certify that every
requested visual correction is complete.
The remaining issues were subsequently resolved and confirmed in the
user-requested [UI carry-through](ui-carry-through-verification-20261011.md).
This report preserves the initial failed confirmation rather than rewriting it
as a pass.

## Scope

Implementation changes address all five priorities and smaller recovery/keyboard/copy issues
from `.impeccable/critique/2026-10-11T14-09-18Z__crates-app-src-main-rs.md`.
Preserved Clear wayfinding. The user's follow-up screenshot also prompted
consolidating the capture prose ledger into grouped counts, a concise next action
and one compact image caption; exact context remains in Details.

No existing user project was edited, no website was visited, and no commit/push
was made by this implementation task. Unrelated working-tree/CI/runtime changes
were preserved. Earlier 26/40 and 24/40 reviews covered different scopes; this
implementation does not assert a new heuristic score or measured usability gain.

## Safety and regression coverage

- Shared owned-window close/application Quit guard; Cancel, sequential confirmed
  Save, explicit Discard, failure/unexpected reply/newer-edit rejection.
- Retained review notes bind to exact revision and page/profile; saving preserves
  disposition, saves every retained draft and never truncates an oversized note.
- Numeric ignored-area creation/editing in original pixels, Apply-before-save,
  capture-pair revalidation and exact union/all-masked rejection.
- Ordinary review decisions/progress/note actions, next-unreviewed draft guards,
  explicit baseline selection and ineligible approval explanations.
- Full-width discovery inventory and independent counts; no unrelated prior
  comparison; representative preview requires matching evidence identity.
- Queue collapse/refit, mounted focus-handle uniqueness and off-section controls
  excluded from Tab. Painted bounds verify keyboard geometry fields are revealed
  inside the 100px Tools scroller at the narrow desktop size.
- Runs/Baselines/Storage separation; stale history-open epochs and newer drafts
  cannot be overwritten. Reopening historical checks/captures is read-only and
  uses the original reference, saved policy/mask or original-only capture.
- Missing/corrupt historical originals fail closed. Reads do not change project,
  baseline activation or persisted display pointers. Each distinct source run is
  loaded once within an authoritative inspection/analysis read, with all later
  reference/report bindings still checked; 3,000 references/three loads regression.
- Correction cycling reveals the right setup stage without submitting; keyboard
  help traps/restores focus; loaded network configuration remains separate from
  editable drafts and capture settings.

## Integrated gate

Final gate evidence is retained privately under:
`/path/to/private-tests/opencode/regressee-ui-improvements-aFbsFi/`.

- `post-harness-tests.log`: current all-features workspace suite; **300 passed,
  zero failed, 38 ignored**.
- `post-harness-clippy.log`: strict workspace/all-targets/all-features Clippy passed.
- `post-harness-format.log`: `cargo fmt --all --check` passed.
- `post-harness-ui-build.log`: ordinary `ui` application build passed.
- Earlier `delivery-*` logs retain the pre-privacy-cleanup compiler gates.
- `confirmation-renderer.log`: first segment of the final GPUI/Blade batch.
- `confirmation-completion.log`: successful fixture-only completion of that batch.

The final compiler gates disable incremental workspace compilation to avoid
ambiguous older local metadata encountered in an earlier Clippy attempt. GPUI's
existing `f32` fallback warnings and the dependency future-incompatibility notice
are separate from application Clippy results.
The current compiler gates' 295-input before/after source records match. An
earlier final formatting attempt encountered a concurrently edited test constant;
the cleanup's subsequent formatting was preserved, not reverted.

## Bounded renderer check

The initial 30-image batch covered 15 states at 1280×800 and 820×600. All images
were independently inspected. Its old 14-entry harness-count assertion failed
after all images were emitted; it was corrected, not treated as an app failure.
The single correction batch targeted narrow image space, invisible keyboard
geometry focus, full-width History selection and focus-handle collisions. The
user's prose-density feedback was included before final confirmation.

The confirmation matrix adds capture-with-gaps and saved-baseline inspection to
the existing review, collapsed queue, note entry, keyboard area editing,
filtered-out draft, discovery/filter, three History sections, reopened check,
History filter, network, field validation and keyboard help states. These are
genuine GPUI/Blade pixels with real Linux fonts and isolated synthetic evidence,
not HTML replicas or website capture results. The proof image was personally
reviewed, opaque/readable and had four exact 1,024-pixel calibration patches.

The final batch initially emitted 15 desktop images, then stopped on a missing
History fixture initialization in the added capture-only state. The test harness
was repaired and completed the same batch, retaining the 15 PNGs byte-for-byte
and emitting only the remaining 19. The completion passed its 34-entry assertion;
there was no second UI correction or replacement capture round.

`confirmation/source-binding-before.json` binds the first segment's 295 inputs
and executable. Its only source difference from
`completion-source-binding-before.json` is `crates/app/src/ui_renderer_capture.rs`
(fixture repair and hash-checked resume); the executables consequently differ.
The completion binding equals `source-binding-after.json`, including executable
hash. These are not claimed as one unchanged executable across both segments.
Image JSON files record dimensions, opacity, hash and renderer facts. Synthetic
temporary state is checked against unchanged authoritative project/evidence
snapshots.

After capture completion, an unrelated privacy cleanup changed diagnostic/test
path literals in `ui_inspection.rs`, `ui_renderer_capture.rs`,
`authorized_blog_e2e.rs`, `native_inspection_fixture.rs` and
`official_runtime_first_use.rs`. Those edits were preserved. The captured pixels
remain bound to their recorded sources, not to that later diagnostic-path
configuration; no further renderer pass was performed. The current-tree compiler
checks have separate `post-gates-source-binding-*.json` records.

## Final image findings

Independent confirmation inspected all 34 PNGs. Their decoded dimensions,
opacity and SHA256 match the metadata and matrix; evidence integrity passes.
Its full report is retained as `independent-confirmation.md` in the private
evidence directory.

Confirmed visually: full-width discovery with four visible inventory rows at
820×600; ordinary review decisions/progress and explicit note entry; collapsed
queue providing wider evidence; distinct Runs/Baselines/Storage sections;
inspection-only presentation for reopened historical checks; grouped status
counts rather than the old prose ledger.

Not accepted / remaining:

- Both added capture-only states retain comparison evidence and controls at both
  sizes. Source inspection explains the fixture mismatch: it calls
  `receive_history_inspection` from Results, but that receiver correctly rejects
  replies outside History. These four images therefore do **not** verify the
  single-image caption or genuine capture-only guidance. This is not evidence
  that production capture-only flows necessarily fail.
- The ignored-area screenshots do not show focused numeric fields; the narrow
  open-Tools state compresses evidence into thin strips. The separate painted-
  bounds keyboard regression passes after opening/drawing Tools before creating
  an area, but does not make these screenshots valid visual focus proof.
- Narrow Runs, filtered Runs and Baselines still paint selected backgrounds only
  to their text-content width, not to the row's full bordered width.
- Filtered-empty review text and the last keyboard-help sentence are not fully
  visible in the default captures. This identifies presentation incompleteness,
  not proof that scrollable help content is unreachable.

No further UI corrections or recaptures were performed within that initial
pass. The user then explicitly requested carrying these remaining items through;
the separate follow-up above records their correction and fresh confirmation.
No new heuristic score or release-acceptance claim is made.

## Qualification boundary

This verifies development-host models, source and offscreen rendering only.
Real OS close/quit/pointer/keyboard/IME, assistive technology, high-DPI, Windows
and macOS remain unqualified. GPUI's final app-quit callback cannot veto direct
platform termination. Technical Tools and long forms still require scrolling;
small synthetic screenshots do not establish real-site inspection readability.
Display caches remain display-only, never mutation or approval authority.

No browser DOM detector applies to this native UI. The earlier detector's empty
result for unsupported `.rs` files was not used as a quality verdict. DESIGN.md
remains the design authority; its previously flagged sidecar drift was not
silently repaired. Evidence from this task is separate from the previous speed
pass in `local-first-ux-verification-20261011.md`.
