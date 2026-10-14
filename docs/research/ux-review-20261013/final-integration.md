# UX refinement — final integration (2026-10-14)

**Status: complete — independent whole Phase 5 and final integration PASS.**
Work/report date: **2026-10-14**. The review directory's `20261013` name is retained.
The independent PASS verdicts for Phases 1–4 remain authoritative for their scope;
this report does not retroactively replace them or qualify a public release.
The separate `phase5-verification.md` records independent inspection of all 104
originals and a repeated complete gate with **387 passed, 0 failed, 52 ignored**.
All five phases of `docs/ux-improvement-plan.md` are implemented and independently
verified; the parent commits and pushes the final phase after updating this status.

## Finding-to-test/evidence map

Final names are `phase5-<1280x800|820x600>-<scene>.png`. The matrix and receipt,
not filenames alone, establish actual states. Incumbent tests below are included
in the complete workspace/all-feature gate, not silently treated as new tests.

| In-scope finding / requirement | Shipped resolution and test ownership | Final targeted scenes / prior accepted evidence |
|---|---|---|
| A1 wizard hierarchy | Connected numbered progress, contextual origin consent, optional Access controls, essentials before advanced; `phase1_setup_tests`, `phase5_setup_tests`, `phase5_repair_tests` | `wizard-website`, `access-public`, `access-optional-mapping`, `invalid-advanced-revealed`; prior whole Phase 1 PASS includes gated/back/private states |
| A5 branding | No rail slogan; About Regressee with factual limits; `phase1_setup_tests`, runtime copy tests | Final full frames have no slogan; Phase 1 accepted About/rail evidence |
| B-02 runtime truth | Shared runtime-aware copy without install promises; `runtime_ux` tests and setup gates | `access-public`, `network-limits-closed/open`; Phase 1 accepted all seven runtime states |
| A2 evidence/menu geometry | Stable compact identity/toolbar, no height loss hiding Results, full-width originals, anchored menu; `phase2_evidence_tests` | `review-compare`, `results-hidden`, `navigation-toggled`, `full-before/after`, `view-menu`, `zoom-menu`; prior accepted drawing-exit repair |
| B-03 blocker explanations | Priority reasons without weakening guards; `phase2_evidence_tests::concise_blockers_are_painted_at_both_widths_without_relaxing_draft_guards` | `dirty-note-blocker`, `dirty-area-blocker`; Phase 2 PASS separately covers unapplied/pending/noncomparable/exhausted states |
| B-04 derived legend | Production significant/ignored legend, untouched originals; `phase2_evidence_tests` | `difference`, `overlay`, `heatmap-with-ignored` with real production mask analysis |
| Selection wording | Selection is distinct from eligibility/approval; `phase2_evidence_tests::selection_copy_names_ready_members_of_the_selection` | `baseline-selection`; Phase 2 accepted empty/selected states |
| A3 and B-01 note correctness | Wrapped multiline editor; one 8,192-byte limit, explicit rejection and exact save/exit; `phase3_note_tests`, generic/multiline input and exit-guard tests | `multiline-note`, `dirty-note-blocker`; Phase 3 PASS covers all boundaries, corrective edits, paste/selection and ownership |
| A4 Analysis hierarchy | Actual result/current policy, Sensitivity first, optional ignored/advanced tools and useful revision controls; `phase4_hierarchy_tests` | `analysis-sensitivity`, `analysis-ignored-coordinates`, `analysis-advanced-tail`; Phase 4 accepted every field/switch/validation/active/previous state |
| History/Settings minor findings | Sparse/bounded rows, loaded-page scope, nearby selected actions, optional exact provenance; `phase4_hierarchy_tests`, `phase6_history_tests` | `history-last-run`, `history-selected-actions`, `history-page-two`, `history-provenance`, network scenes; Phase 4 accepted zero/few/no-match/Settings cases |
| Q-03/evidence gaps: Mobile, invalid reveal, PRIVATE loading | Actual mounted traversal/owner checks; saved private access plus outstanding read-only request; new `phase5_state_tests` | `mobile-revealed-selected`, `invalid-advanced-revealed`, `private-login-loading` |
| Remaining customization/inclusion/filter gaps | Real control transitions and production classifications; new `phase5_state_tests` | `tablet-selected`, `custom-width/height`, `custom-added-actions`, both limitations tails, `visible-screen-selected`, `replacement-selected`, `fast-selected`, four discovery inclusion scenes, Changed/Gaps/no-significant-change/grouped filters |
| Network/portability details and final keyboard defects | Exact reading selectors, mounted owner reveal and non-aliasing disclosure focus; new `phase5_state_tests` | `network-limits-closed/open`, `delete/export/import-consequences`, `delete/export/import-exact`; no confirmation dispatched |

## Final native evidence and inspection

Authoritative private **0700** root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261014-4zgty46x`.

- The renderer proof was personally inspected before app capture authorization.
  Four reused website PNGs are explicitly synthetic/offline; no earlier app PNG
  was reused, renamed or regenerated as final evidence.
- The **sole planned native batch** completed with exit **0**, **249.05s** wall
  time (248.83s test body): **104 fresh app PNG/JSON pairs**, **52 at 1280×800** and
  **52 at 820×600**. No visual-polish repair or confirmation round was taken.
- All 104 frames were personally inspected on **18 labelled full-frame sheets**,
  all **58 native-scale reading crops** on **ten reading sheets**, and **26
  important originals** at original resolution. Normal-size full sheets are
  scaled; narrow full sheets and reading crops are native scale. Derivatives
  are labelled inspection aids, never substitute app captures.
- The durable `phase5-inspection-ledger.json` identifies every frame's inspection
  method, derivative hashes, full-resolution spot checks and observations.
  `phase5-receipt.json` binds original/sidecar/state/geometry hashes, final sources,
  proof, fixtures, exact commands/exits, gate logs and safety assertions.
- Current-source integrity/state verification passes for all **34 embedded
  fingerprints per PNG**, PNG hashes/dimensions/chunk CRCs, proof and fixture
  provenance, all asserted owning clips and both-size scene membership. The final
  manifest is `final-source-provenance.json`; the root's initial
  `source-provenance.json` is retained as historical, not final qualification.

Measured Before-pane heights at **1280×800 / 820×600** are **389px / 234px** in
Compare, Results hidden and full-width Before; full-width After has the same
height. Comparisons use the same navigation state. The complete invalid Parallel
active pages field is **123px** high, with its **18px** correction inside the actual
form owner at both sizes. Mobile is visibly selected and revealed; custom fields,
actions and both limitations tails are separate genuine reading positions, not
claimed simultaneously visible. PRIVATE loading has saved `InteractiveLogin`
and a real outstanding read-only request with login/Continue blocked. Heatmap
shows both significant and ignored colors produced by the production analyzer.

Discovery records **128 discovered**, with included counts **74 → 75 → 74 → 75**
after actual single-page and matching-page controls. Changed/Gaps/no-significant-
change/grouped filter outcomes have separately asserted count/classification/
grouping states; filtering deliberately retains the selected original pair.
The narrow filter screenshots show their heading/count and retained evidence;
with the saved-area success notice present, they do not establish simultaneous
visibility of an entire matching row. No such row visibility is claimed.
History selects the real last loaded run (**index 63 of 64**), reveals selected
actions, then loads the second page and optional recorded provenance. Exact
portability details and network limitations fit their own real reading clips.
Surrounding form/list text may intentionally be beyond the current viewport;
only the measured revealed selectors are claimed wholly visible.

Non-rendering preflight/debugging is separate from native inspection. Its final
complete two-size recipe exited **0**, with 104 state/geometry records and **no
app PNGs** (263.50s test body). Earlier structural failures and source-drifted
qualification attempts remain intact. The three demonstrated production repairs
and four ordinary regressions are described in `phase5-implementation.md`.

## Completed final-source qualification

All commands below completed in the foreground, with no jobs left running.
Common environment: `LIBRARY_PATH="$PWD/.runtime/link-libs"`,
`TMPDIR=/var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261014-4zgty46x`.
Gate records/logs are in `<root>/accepted-final-gates/`. Its Rust/Cargo manifest
matches before and after every command, and agrees with every embedded PNG
source. The evidence verifier and final handoff checks reconfirm that binding.

| Exact command | Completed result | Wall time / log |
|---|---|---|
| `cargo test --locked --offline --workspace --all-features -- --test-threads=1` | exit **0**; **387 passed, 0 failed, 52 ignored** | 658.39s; `workspace-all-features.log` |
| `cargo build --locked --offline --workspace --features ui --bins -j 4` | exit **0** | 1.58s; `ui-build.log` |
| `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings` | exit **0** | 1.78s; `clippy.log` |
| `cargo test --locked --offline --workspace --no-default-features --no-run` | exit **0**; compilation only | 0.13s; `base-no-default-no-run.log` |
| `cargo test --locked --offline -p regressee-app --no-default-features --features ui --lib --bin regressee --no-run` | exit **0**; compilation only | 1.08s; `ui-only-no-run.log` |
| `cargo test --locked --offline -p regressee-app --no-default-features --features ui-tests --lib --bin regressee --no-run` | exit **0**; compilation only | 2.81s; `ui-tests-no-run.log` |
| `cargo fmt --all -- --check` | exit **0** | 0.80s; `format.log` |
| `git diff --check` | exit **0** | 0.01s; `whitespace.log`; repeated after final documentation |

Workspace aggregates, including the normal integration targets:

| Target group | Passed / failed / ignored |
|---|---|
| App library / native binary | 98 / 0 / 2; 148 / 0 / 23 |
| `analysis_review` / `product_flow` / `qualification` integrations | 7 / 0 / 0; 1 / 0 / 13; 1 / 0 / 0 |
| Core / Runtime library / Runtime distribution | 52 / 0 / 0; 28 / 0 / 1; 6 / 0 / 0 |
| Runtime Linux ownership / Spike / Storage | 0 / 0 / 3; 11 / 0 / 0; 35 / 0 / 2 |
| Explicit website/runtime/native fixture probes, unchanged opt-in | 0 / 0 / 8 |
| Remaining binaries and doc tests | 0 / 0 / 0 |
| **Total** | **387 / 0 / 52** |

The normal persisted **100/1,000-page selective approval/privacy/retention**
qualification test passed in **543.85s**. This is isolated stored-data testing,
not the ignored real 1,000-page browser soak. All four new Phase 5 regressions and
incumbent Phase 1–4 note/IME/controller/consent/revision/history tests are in the
successful full gate. The separately opted-in native recipe passed one test;
it is not silently added to the 387-test aggregate. No ignored network, signed
runtime download, real OS-store or OS-input probe was enabled. The existing
`proc-macro-error2 v2.0.1` future-incompatibility notice persists; strict Clippy
introduces no warning. Prior 386-pass runs with source drift and earlier feature
gates are historical only, not substituted for these completed same-source gates.

## Reproduction and artifact policy

See `phase5-implementation.md` for the authoritative 0700 root, repair history and
fixture/pending-state provenance. Use a fresh canonical private root under the
approved scratch directory. Copy only the four hash-bound synthetic website
fixtures, not prior app PNGs. Run the proof, personally inspect it and write its
hash-bound `proof-reviewed.json` before the app recipe:

```sh
export LIBRARY_PATH="$PWD/.runtime/link-libs"
export TMPDIR=/var/home/olav/Data/opencode/tmp/opencode
export REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode
export REGRESSEE_RENDERER_CAPTURE_DIR=<new-private-0700-root>
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::renderer_capture_proof -- --ignored --exact --test-threads=1
# Inspect proof before authorizing captures; leave REGRESSEE_PHASE5_PREFLIGHT unset.
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::ui_phase5_capture::renderer_capture_phase5 \
  -- --ignored --exact --test-threads=1
python3 scripts/ux-phase5-evidence.py "$REGRESSEE_RENDERER_CAPTURE_DIR"
```

The script verifies current embedded sources, PNG/metadata/state/geometry hashes,
actual recorded classifications/counts/choices and owner bounds. It emits no
credentials, note text, project UUIDs or user URLs. It is an implementer integrity
check, not visual inspection or independent verification. No PNG, fixture, ZIP,
database or full log is committed. Durable small receipt/ledger accompany this
report; private originals, failures, proof and complete logs remain intact.

## Existing qualification limits

Offscreen paint plus simulated GPUI traversal/activation/wheel/clipboard is not
OS presentation/input/IME/clipboard/file-picker, assistive technology, scaling,
high contrast, automatic OS motion preferences, Windows/macOS, usability,
network/runtime distribution or release acceptance. Linux AT-SPI registration
and the macOS process-ownership capture gap remain documented blockers. No real
website, login/session, enterprise network or shipping-runtime probe is part of
this phase. The native app has no relevant DOM/markup detector coverage.
