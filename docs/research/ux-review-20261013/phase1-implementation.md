# Phase 1 implementation — guided setup and product language

**Status: implemented/awaiting re-verification.** This is the implementer's handoff,
not an independent PASS, usability result or release qualification. Only Phase 1
of `docs/ux-improvement-plan.md` was implemented. No commit or push was made.

**Latest handoff:** see [Repair batch 1](#repair-batch-1--independent-fix-v1-1-and-v1-2)
below. The independent verdict in `phase1-verification.md` remains **FIX** until
the verifier rechecks these repairs. The original sections below record the
pre-repair implementation/evidence; their “current-source” descriptions refer to
that historical snapshot, not the repaired source. Its 60 PNGs/sidecars are
preserved unchanged and have not been relabeled as current.

## Changes and finding-to-evidence map

| Requirement / finding | Implementation | Tests and final capture states |
|---|---|---|
| Connected progress, A1 | Shared numbered Website → Access → Pages & sizes component with drawn connectors and explicit Current/Completed/Upcoming words. Location derives from the current stage, not draft validity. Gated future steps stay legible noninteractive progress. Back preserves drafts and focuses the destination step. | `progress_location_back_and_future_gates_keep_invalid_drafts`; Website, Access and Pages states, including invalid drafts and `website-back-dirty` |
| Contextual website consent, A1 | Mount authorization only for multiple distinct origins. Address observation resets consent when the origin set changes, including while a save reply preserves a newer draft. Backend save consent remains enforced. | `consent_is_distinct_origin_relevant_and_resets_without_visits`; `website-blank`, `website-single-origin`, `website-multiple-{unapproved,approved}`, `website-origins-changed` |
| Access hierarchy, A1 | Public/login decision first, with selected-environment context. Optional environments, staging/mappings/probes, locale/timezone and website theme behind “Environments & capture configuration (optional)”. Private-login provider/session fields remain in the required workflow. Validation can reveal hidden public-login corrections. | `optional_configuration_mount_focus_reveal_and_drafts_are_independent`; public/private/loading/browser-open/ready, disclosure open/closed, mapping/provider reveal |
| Pages hierarchy, A1 | Capture-plan summary, screen sizes, extent, page-list choices/page limit and image mode precede optional scope/speed/custom-size controls. Custom profiles and capture limitations are retained. Access and Pages disclosures have separate state. | Existing setup geometry and validation-recovery tests; `pages-essentials`, `pages-invalid-{summary,revealed}`, `pages-custom-tail`, `pages-limitations` |
| Product language, A5 | Remove the rail slogan without replacement; rename entry and dialog to About Regressee. Include package version/license and retain local storage, ZIP/session privacy, screenshot exposure, development-runtime and platform/qualification caveats. | Both-size rail and `about-regressee` inspection |
| Runtime truthfulness, B-02 | Setup and App settings use one status-aware copy helper. Checking, supported not-installed/ready, development-only, repair-required, unavailable and unsupported states do not make contradictory installation promises. Setup availability is a separate request-bound, off-thread read, not a busy save gate, installer or visit. | `runtime_copy_covers_all_states_and_development_scope`; `setup_runtime_read_is_once_and_preserves_newer_drafts_without_visits`; seven `runtime-*` scenes |

### Exact implementation file inventory

Production code:

- `crates/app/src/main.rs`: state, address observer, reply/activation wiring,
  setup grouping, shared runtime presentation and About entry.
- `crates/app/src/setup_ux.rs`: progress, stage gates, Back, disclosure,
  login-field visibility and distinct-origin consent refresh helpers.
- `crates/app/src/runtime_ux.rs` (new): runtime copy and independent observation.
- `crates/app/src/workflow.rs`: progress mounting, disclosure-aware controls,
  footer Back and slogan removal.
- `crates/app/src/form_recovery.rs`: disclosure measurement/validation reveal
  and About dialog wording.
- `crates/app/src/ui/setup_progress.rs` (new): reusable native numbered steps.
- `crates/app/src/ui/mod.rs`: export the component.
- `crates/core/src/shell.rs`: `SetupBack` / `EnvironmentOptions` and focus candidates.

Tests and opt-in evidence tooling:

- `crates/app/src/phase1_setup_tests.rs` (new): four focused regressions.
- `crates/app/src/phase1_runtime_tests.rs` (new): independent runtime-read regression.
- `crates/app/src/options_ux_tests.rs`: intentionally open the optional disclosure
  for mapping tests.
- `crates/app/src/phase5_setup_tests.rs`: drain the independent availability read
  before asserting an exact explicit-discovery request. This existing filename
  does not signify implementation of plan Phase 5.
- `crates/app/src/workflow_refinement_tests.rs`: job-drain helper includes runtime reads.
- `crates/app/src/ui_renderer_capture.rs`: register Phase 1 captures and embed
  eight relevant source fingerprints in every PNG sidecar, including renderer proof.
- `crates/app/src/ui_phase1_capture.rs` (new): opt-in 30-state × two-size native
  matrix with state, mounted/enabled, focus and actual geometry assertions.

Documentation added/updated by this handoff:

- `docs/research/ux-review-20261013/phase1-implementation.md` (this report).
- `docs/ux-improvement-plan.md`: only Phase 1 status/execution row updated.

The plan and five incumbent assessment/synthesis files were already untracked
user work. They were not deleted, rewritten or treated as newly authored work.
Phases 2–5, `DESIGN.md` context drift, comparison/storage algorithms and runtime
distribution remain outside this implementation.

## Safety and regression coverage

No backend/storage schema or comparison changes. Presentation does not authorize
login, discovery, baseline activation or deletion. Save/login/discovery actions
retain their existing request/project/environment/draft guards; clean Continue
does not rewrite settings. Hidden optional controls cannot activate or participate
in traversal. Escape closes the appropriate disclosure and restores its trigger;
reopening retains mapping-editor drafts. Required login fields remain mounted
without opening optional configuration. Validation expands the correct stage and
scrolls the actual field into its owning form clip.

The five new regressions are listed above. Existing `phase5_setup_tests.rs`
save/login/failed-continuation and exact-save/discovery tests still run, as does
`validation_correction_reveals_the_right_stage_without_submission_and_help_restores_focus`
in `workflow_refinement_tests.rs`. Full-suite coverage also retains evidence,
review/baseline, history, privacy, deletion and draft regressions; this is not an
independent audit of every invariant or a claim of native OS-input qualification.

### Intermediate corrections retained honestly

- Updated stale tests that assumed mapping controls or multi-origin consent were
  always mounted; retained full Project settings mounting behavior.
- Moved setup runtime observation out of ordinary `pending` into `runtime_pending`:
  its initial implementation blocked normal save/request tests. Replies are now
  matched independently and stale runtime IDs are rejected.
- Preserved validation access to otherwise-hidden public-login fields with
  `setup_login_correction`.
- Fixed save-reply hydration reapproving a newer origin set, including the
  preserved-draft reply path; the consent race regression now exercises this.
- Initial captures used a review fixture with a baseline. Final recipes instead
  save a new isolated standard Desktop/Mobile project with no baseline.
- After the initial visual batch, the single collective correction batch added
  real setup availability observation, simplified public optional configuration,
  environment context and actual limitations-text geometry.
- Intermediate capture/test failures were repaired before definitive evidence.
  Earlier artifacts/logs remain retained; no failed attempt was substituted for
  the final successful capture command.

## Commands and results

All cargo commands ran from the repository root with:

```sh
export LIBRARY_PATH="$PWD/.runtime/link-libs"
export TMPDIR=/var/home/olav/Data/opencode/tmp/opencode
```

| Command | Observed result / log |
|---|---|
| `cargo test --locked --offline --workspace --all-features -- --test-threads=1` | All executed suites succeeded, no failures; `regressee-ux-phase1-confirmation-20261013-Gsap3SO6/workspace-tests.log`. Ran after final production corrections, before final capture-harness-only import/format changes. |
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee -- --test-threads=1` | Final source: 128 passed, 0 failed, 17 ignored; definitive `app-tests.log`. |
| `cargo build --locked --offline --workspace --features ui --bins -j 4` | Final source succeeded; definitive `build.log`. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings` | Final source succeeded; definitive `clippy.log`. |
| `cargo fmt --all -- --check` | Succeeded; repeated at documentation handoff. |
| `git diff --check` | Succeeded; repeated at documentation handoff. |

An existing future-incompatibility notice remains for `proc-macro-error2 v2.0.1`;
it is not a Clippy failure. Ignored tests include opt-in captures/qualification
work, not silently asserted passes. No additional platform/feature-matrix gate
or live runtime-install/login qualification is claimed here.

For the definitive evidence root below, set both renderer variables, run and
personally inspect proof, then create its hash-bound `proof-reviewed.json` before
running the matrix:

```sh
export REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode
export REGRESSEE_RENDERER_CAPTURE_DIR=<new canonical private 0700 evidence root>
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::renderer_capture_proof -- --ignored --exact --test-threads=1
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::ui_phase1_capture::renderer_capture_phase1 \
  -- --ignored --exact --test-threads=1
```

Proof and matrix each succeeded (1 passed); logs are `proof.log` and `capture.log`.
Captures assert actual synthetic states rather than relying on filenames. No
public website, real project, credentials or user website pixels were used.
Browser-open/session-ready and runtime-copy scenes are explicitly constructed,
not evidence of an actual browser opening, login or runtime installation.

## Evidence, source binding and bounded visual inspection

**Definitive private 0700 root:**
`/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase1-final-20261013-AiZyyVOR`

Contains 60 original PNG/JSON pairs (30 at 1280×800, 30 at 820×600),
`phase1-matrix.json`, `source-provenance.json`, `capture-integrity.json`, current
`renderer-proof.{png,json}`, `proof-reviewed.json`, logs and ten paired contact
sheets. Integrity checks matched PNG hashes/dimensions, all eight embedded
Phase 1 source bindings, proof hash and private directory mode. The 17-file source
manifest was rechecked against current files at handoff; it is not a whole-tree
fingerprint. Only documentation changed after these captures.

Core render SHA-256 fingerprints (also embedded in each sidecar):

```text
main.rs              960f291564d4d24caea2d8b3c80055990329ae3c6fb247068315ba9d7434276c
workflow.rs          fc008adb32ec1fa9a9b4d3c1b951bf3e4c3f236dc72ae17a3c01bce0d50b3a1c
setup_ux.rs          9c3f506c6bfdf61662c00dbf5e05fcb9c11bf14f658dbe2892a9c5a20ced48be
runtime_ux.rs        c498cf64584de2eaf6e79b205806aabe369e05b9a264612cafdb85aac3ce0dab
form_recovery.rs     dfb3295cac3e6299f1be36d500cdcfd8c45d442d5df08f31a0b8319e47aa8b83
ui/setup_progress.rs 6a1cb34a4eab4e57508c8d88663b46ffd334c40ed9b61ac3ff4cbee26c018f03
core/src/shell.rs    c86b567082b9e9ec585bf77b70b7882685cbe69d15c07bb53153a9f1423dab00
ui_phase1_capture.rs ec87c7c0b5146ae25ee477d54671064c02522436a14d5c30cccc60034b60432b
```

Base commit: `5f136b7819c3663517d2f9bd5e17f7203e5b262e`.
Proof PNG SHA-256:
`958c19dbf6f90b30df1f1ba9f7efd7b43dc03fc03edde2d4a3d9e89ac9c60c5d`.
Source-provenance manifest SHA-256:
`9e4c5a843c2f0dcb11e89b401564ff9bfd35a1259b30d762d0cc499f5e2f1506`.

### Confirmation inspection ledger

The planned initial 60-image batch was inspected through ten paired contact
sheets plus targeted originals. Its collective correction batch was followed by
**one** final confirmation: definitive `contact-sheets/phase1-{01..10}.png`,
targeted originals and enlarged progress details from the same PNGs. No new
polishing loop or additional renderer capture followed confirmation.

| Final paired sheet | Inspected states / observation |
|---|---|
| 01 | Blank/single-origin/multi-origin unapproved Website; numbered current/upcoming states, consent relevance and stable footer. |
| 02 | Approved/changed origins and invalid Website; visible consent reset and correction styling without losing current location. |
| 03 | Dirty Back, public Access and private unsaved; simpler decision-first hierarchy and explicit save/login gates. |
| 04 | Private saved/loading/browser-open; footer copy/actions reflect the asserted state. |
| 05 | Ready, optional environment open and mapping revealed; environment group and real focused mapping field. |
| 06 | Closed options retaining mapping text, provider revealed, Pages essentials; private correction remains reachable; concise plan precedes customization. |
| 07 | Pages invalid summary/revealed and custom-size tail; current stage retained, invalid field revealed and custom controls preserved. |
| 08 | Fully revealed limitations, runtime checking/not-installed; factual limitations readable; installation promise confined to supported state. |
| 09 | Runtime ready/development-only/repair; distinct bounded explanations. |
| 10 | Runtime unavailable/unsupported and About; no unsupported installation promise, no rail slogan, factual/privacy caveats retained. |

Original-size checks: 820×600 Access public/mapping/provider, Website multiple
unapproved, Pages essentials/invalid-summary/limitations, runtime not-installed
and About; 1280×800 Pages essentials. Enlarged progress details were also inspected.
Some displayed full-frame previews initially appeared to omit a step. Raw crops
showed identical RGB bytes to correctly displayed steps and
`contact-sheets/confirmation-progress-detail.png` visibly confirmed their numbered
labels. No missing-step defect is established in the actual PNGs; this check did
not modify originals or application code.

No confirmed in-scope clipping/overlap blocker remains from this inspection.
At 820×600 optional disclosures and additional login fields may sit below the
first viewport; scrolling/Tab reveal is required and tested, not disguised as
first-viewport visibility. Geometry assertions complement pixel inspection, not
replace it. Examples from the actual 820×600 form bounds (`x,y,w,h`):

| Revealed element | Measured bounds | Owning scroll clip |
|---|---|---|
| Multi-origin consent | `229,481,578,44` | `192,235,628,298` |
| Mapping field | `216,424,580,101` | `192,235,628,298` |
| Identity-provider field | `216,371,580,123` | `192,235,628,267` |
| Invalid parallel-pages field | `216,381,580,123` | `192,235,628,277` |
| Capture limitations text | `216,261,580,159` | `192,235,628,277` |

All three progress bounds and Save were asserted within each setup viewport;
public/login choices were asserted within their actual form clip. Mapping,
provider, consent and invalid-field evidence uses the production shell Tab route
and focused reveal. Limitations uses the mounted trigger plus actual form scroll
to the tail. Full details are in `phase1-matrix.json`.

Earlier retained evidence (not substitutes for definitive current-source pixels):

- `/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase1-20261013-pytvAb36`:
  initial inspected batch and correction/test history.
- `/var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase1-confirmation-20261013-Gsap3SO6`:
  workspace-suite log and intermediate corrections/capture attempts. Intermediate
  captures were not used as an extra visual polishing round.

## Verifier handoff and limits

No known functional or confirmed visual blocker. Independently inspect the
changed source, draft/request/origin guards, disclosure mounting/focus, runtime
copy, current-source screenshots and integrity receipts before issuing a scoped
verdict. The implementer has **not** issued PASS or authorized starting another
phase. No subagents were spawned, and no private evidence was added to git.

This evidence covers Linux GPUI/Blade offscreen rendering, real Linux fonts and
simulated native control/input paths only. It does not qualify real OS
presentation/input, IME, assistive technology/accessibility bridges, high-DPI,
high-contrast, other platforms, live networks/login or runtime distribution.
Those remain the existing release obligations documented in the plan/About;
screenshots and successful tests are not a substitute.

## Repair batch 1 — independent FIX V1-1 and V1-2

**Correction status: implemented/awaiting re-verification, not PASS.** Read the
complete 299-line independent report before repairing both findings together.
`docs/research/ux-review-20261013/phase1-verification.md` and all verifier evidence
remain unchanged. Its report SHA-256 is
`8c6d18d52e10af9a88b04b056db759e1a78b6a739a2b87b386f0fbfbcf2b646b`.
The original handoff's “no known blocker” statement did not qualify the normal
entry/build paths; the independent FIX correctly superseded that statement.

### Exact repair delta

Only these three source files changed relative to the verifier's snapshot:

- `crates/app/src/main.rs`: after the guarded `create` transition establishes
  Website and Name focus, call existing `observe_setup_runtime`. Both the mounted
  New Project control and existing CreateProject shortcut/action use this path.
  Gate `phase1_runtime_tests` and `phase1_setup_tests` with `test + ui-tests`,
  matching neighboring simulation tests. No Cargo feature dependency workaround,
  production test-only behavior, paint/style/copy changes or weakened gates.
- `crates/app/src/phase1_runtime_tests.rs`: remove the older test's initiating
  refresh shortcut; enter through mounted NewProject after settled Projects.
  Add `settled_projects_control_and_shortcut_start_once_without_refresh` and
  `fresh_entry_runtime_reply_does_not_own_a_concurrent_project_save`.
- `crates/app/src/ui_phase1_capture.rs`: add the opt-in
  `renderer_capture_phase1_fresh_entry` recipe, without altering or rerunning the
  original 60-state recipe.

Documentation delta is this appended handoff/status notice and Phase 1's status
and execution row in `docs/ux-improvement-plan.md`. No verifier report edit,
other-phase work, commit, push or subagent.

### V1-1: real entry, once-only read and save ownership

The new entry regression starts from a genuinely empty isolated store with its
initial Projects request fully completed, no selected project, no runtime status
and no pending availability request. It asserts mounted/enabled NewProject and
uses its supported activation route; a separate case simulates **Ctrl+N** using
the existing Shell CreateProject binding. Both paths run at both desktop sizes.
Neither uses a test-side entry refresh, preselected/saved fixture or injected
successful runtime result.

Assertions establish exactly one new availability request; Checking is backed by
that actual pending ID, then the real worker reply supplies authoritative
UnavailableDistribution for this unconfigured build. Five ordinary draw/poll
passes and Projects → NewProject re-entry do not submit another read. The Name
focus, name/address drafts and local save capability survive; a real local save
then succeeds and deliberately continues to Access, without evidence or a task.

The concurrent-save regression starts the read through the same actual control,
then saves while the read is outstanding. It consumes the **real earlier worker
reply**, without fabricating a matching success. The independent runtime reply
does not clear ordinary save pending/busy, replace the submitted project identity
or exact draft signature, or change focus. The later real save persists the
submitted project/name while preserving newer draft text and Website location.
The older stale-ID test still rejects an unrelated runtime reply.

The runtime result is build/app availability, not project settings: retain its
existing once-only app cache rather than invent a per-project capability or
hydrate a project from it. Runtime replies remain independently ID-bound;
project saves retain their request/submitted-project/draft ownership. The worker
still receives only `Work::RuntimeStatus`, whose read-only readiness path does
not install, log in, discover or visit a website. Ordinary `pending`, task,
authorization and validation gates were not changed.

Before the production repair, all three actual-entry runtime tests failed with
the missing read (`runtime-before.log`). The UI-only build failed with the same
missing simulation dependencies (`ui-only-before.log`: 24 errors after adding
the two new GPUI tests; the verifier's original snapshot had 18). An intermediate
capture-harness compile error from a temporary debug-selector borrow is retained
in `runtime-after.log`; fixed with static selectors before captures. The repaired
runtime tests then passed in `runtime-after-2.log` and final focused/full runs.

### V1-2 and Phase 1 fallout gates

All commands below ran from the repository root with the same approved
`LIBRARY_PATH` / `TMPDIR` environment documented above. Exact command arrays,
exit codes and log names are in the new root's `repair-commands.json`.

| Exact command / filter | Final result |
|---|---|
| `cargo test --locked --offline -p regressee-app --features ui --bin regressee --no-run` | Exit 0; `ui-only-final.log`. The verifier's exact previously failing command. |
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee phase1_runtime_tests -- --test-threads=1` | 3 passed; includes real entry/shortcut and concurrent real read/save. |
| Same all-feature bin command, `phase1_setup_tests` | 4 passed: progress/back/current/dirty gates, consent/race, disclosure/focus/drafts and runtime copy. |
| Same all-feature bin command, `phase5_setup_tests` | 4 passed: clean Continue, exact-save continuation, private/save/login and discovery boundaries. |
| Same all-feature bin command, `phase5_repair_tests` | 5 passed: mounted capabilities, validation reveal and draft/save guards. |
| Same all-feature bin command, `options_ux_tests` | 4 passed: bounded mappings, optional controls and newer environment drafts. |
| Same all-feature bin command, `validation_correction_reveals_the_right_stage_without_submission_and_help_restores_focus` | 1 passed. |
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee -- --test-threads=1` | 130 passed, 0 failed, 18 ignored; `app-regressions.log`. |
| `cargo test --locked --offline -p regressee-app --features ui-tests --bin regressee phase1_ -- --test-threads=1` | 7 passed; proves corrected module gates retain execution under the qualified feature, without requiring all features. |
| `cargo test --locked --offline -p regressee-app --all-features --lib product:: -- --test-threads=1` | 12 passed. |
| Same all-feature lib command, `jobs::` | 13 passed. |
| `cargo test --locked --offline -p regressee-core shell:: -- --test-threads=1` | 11 passed. |
| `cargo build --locked --offline --workspace --features ui --bins -j 4` | Exit 0; native UI build. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings` | Exit 0. |
| `cargo fmt --all -- --check` / `git diff --check` | Exit 0, repeated after documentation handoff. |

Full Phase 1 requirements were rechecked for fallout: connected progress/current
validity and safe Back, origin-relevant consent/reset/save race, required private
login versus independent optional disclosure, complete Project settings, Pages
summary/custom sizes/limitations, slogan removal/About facts/privacy and all
shared runtime-copy states remain unchanged. Their relevant regression filters
above and the full app suite pass. UI source outside the entry call/test gates
has the same bytes as the verifier's snapshot; no accepted visual layout was
reworked. No full workspace test rerun or OS/live-runtime qualification is
claimed for this repair. Prior workspace results are historical; Clippy/build
and the stated current tests are newly run. The existing dependency
future-incompatibility notice remains; ignored tests are not passes.

### Small, freshly source-bound confirmation

**New private 0700 root:**
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase1-repair-20261013-a0koej7c`

Generated and personally inspected a fresh repaired-source renderer proof, then
ran **only** this bounded recipe using the approved renderer variables:

```sh
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::renderer_capture_proof -- --ignored --exact --test-threads=1
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::ui_phase1_capture::renderer_capture_phase1_fresh_entry \
  -- --ignored --exact --test-threads=1
```

Both exit 0 / 1 passed (`proof.log`, `fresh-entry-capture.log`). Four app captures:
`phase1-fresh-entry-{1280x800,820x600}-{checking,authoritative-unavailable}.{png,json}`.
The recipe settles the empty Projects store, activates mounted NewProject, paints
the actual pending read and then its actual authoritative reply. It never loads
a fixture project or injects runtime readiness. It scrolls the real form to its
tail after layout to reveal the note, retaining Name focus and Save capability.
The Checking capture is taken before the real reply is consumed by controller
polling; it is not a fake/delayed backend status.

Personally inspected all four originals in **one targeted confirmation round**.
The runtime label/help and stable Save footer are legible and contained; no new
confirmed clipping/overlap blocker. At 820×600 the earlier Website section is
partly above the real scroll clip after scrolling to the note; this is expected
scroll position, not a claim that all content fits the first viewport. No further
visual polishing or full-matrix regeneration occurred.

`fresh-entry-matrix.json` asserts settled entry, request 1 → runtime request 2,
Checking pending 2 → authoritative pending none, no ordinary pending/task/busy,
no project/environment/website visit, Name focus and Save enabled. All three
progress bounds and Save are within the viewport. Actual note/scroll geometry:

| Size, both states | Runtime note `x,y,w,h` | Actual form scroll clip `x,y,w,h` |
|---|---|---|
| 1280×800 | `216,575,680,60` | `192,224,1088,511` |
| 820×600 | `216,460,580,60` | `192,224,628,311` |

New sidecars and fresh proof bind **all 13 embedded source fingerprints** to the
repaired source (eight Phase 1 + five incumbent). Integrity checks match all four
PNG hashes/dimensions, proof/receipt, note containment and 18 current manifest
files; `capture-integrity.json` records the inspected inventory/results.
`source-provenance.json` records new/old hashes, exact three-file repair delta,
app/test binary hashes and the unchanged FIX-report hash. Current changed hashes:

```text
crates/app/src/main.rs
  44154ace58a66ad9f004a7f007584276fe1b64705e92426b11774586c7891a53
crates/app/src/phase1_runtime_tests.rs
  2002ee1e45dd272928f5e779e34aa3d2adce28a2337b2ecb31bde3e4f26930f2
crates/app/src/ui_phase1_capture.rs
  e12c2dccdc202700b3152030bb34e18f63d9d322b3e90b04ad926205cbc0c115
```

Current source-manifest SHA-256:
`e1636ac4429a7fb61e63062b1b0f927fa2d7e23fb7a165caa891fc1120cdb272`.
The new proof PNG happens to have the same pixel hash as the prior proof, but was
**actually rerendered** and has a new repaired-source sidecar/receipt.

The original 60 PNGs and old sidecars remain in the prior root. Their hashes and
original `main.rs` bindings were rechecked unchanged. They are **historical
pre-repair evidence**, not current-source screenshots. No source hash in them was
rewritten, no capture copied into the new root, and no unchanged-paint inference
was disguised as recapturing all Phase 1 scenes.

**Hand back to the same verifier for V1-1/V1-2 re-verification.** No implementer
PASS, no phase advancement and no commit/push. Existing Linux offscreen/simulated
input versus OS/AT/cross-platform/live-runtime qualification limits still apply.
