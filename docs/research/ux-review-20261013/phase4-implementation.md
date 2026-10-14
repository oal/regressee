# Phase 4 implementation — progressive controls and secondary pages

**Status: implementation complete — awaiting independent verification (2026-10-14).**
Phase 4 only of `docs/ux-improvement-plan.md`, findings **A4** and the minor
History/Settings repetition and terminology findings. Work date: **2026-10-14**.
Starting HEAD: `b1becf830569e7c535cc3aafe346c78f4cecbe64`. No child agents, commit
or push. The parent's existing Phase 3 push-row edit and prior PASS/pushed phases
remain intact. Phase 5, `DESIGN.md` and unrelated metadata are not changed.

## Implementation and safety

- `crates/app/src/analysis_ux.rs` presents the selected target's actual saved
  classification, displayed policy and current sensitivity, including comparison
  limits and failures. **No significant change** means below analysis thresholds,
  not identical pixels; missing/noncomparable evidence never becomes clean.
- `crates/app/src/main.rs` leads Analysis with that summary and **Sensitivity**
  presets. Preset/valid parameter edits still use the existing debounced
  reanalysis of saved originals; earlier revisions and reviews are retained and
  no website is visited. Cancel mounts only for scheduled/uncancelled active work;
  previous analysis mounts only when another retained revision exists.
- **Ignored areas** is a purpose-specific disclosure (`Detail::IgnoredAreas` in
  `crates/core/src/shell.rs`), not a new generic draft/mutation owner. It contains
  Draw, Add by coordinates, the existing coordinate editor, Apply/Save and
  conditional selection/redraw/delete/revalidation actions. Original pixel
  coordinates, exact before/after ownership, coverage validation and the
  distinction between ignoring and redaction are retained.
- All nine advanced numeric fields and both existing switches follow the common
  choices. Metrics, exact revision information and historical reviews are later,
  optional reading. No analysis algorithm, preset values or persisted schema
  changes. Polling unmounted advanced fields cannot schedule hidden mutations.
  Existing field-local validation/IME semantics are unchanged; explicit Reanalyze
  still reveals a genuine invalid field without scheduling work.
- `crates/app/src/review_ux.rs` and `viewer_ux.rs` reveal complete labelled fields,
  buttons and switches using current painted geometry in the real bounded inner
  owner. The core candidate order follows the visual groups; the app still filters
  it by actual mounted/enabled controls. Close/Escape preserve drafts and restore
  the appropriate trigger. The note editor keeps its incumbent reveal behavior.
- `crates/app/src/workspace_history.rs` condenses section/search help, repeated
  types and dates. Loaded-page scope, no-match recovery and pagination stay
  explicit. Lists use actual 64px rows: one to four rows for sparse lists, then a
  256px bound for long loaded pages. Zero records mount no empty list. Selected
  record details and Open/export/deletion actions remain together. History action
  focus uses the outer owner; changing section resets only presentation scroll.
- App settings' heading now matches its navigation label. `workflow.rs` and the
  result summary use **no significant change** instead of an ambiguous
  **unchanged** classification shorthand. Shared Clear wayfinding tokens, native
  controls and system typography remain authoritative.
  Settings keeps actual optional runtime status plus its state/help, without an
  invented duplicate checking paragraph. Ignored-area status distinguishes
  unapplied coordinates, unsaved geometry and saved/empty areas with correct
  singular/plural wording; a closed disclosure still indicates a coordinate draft.
- ZIP privacy, raw/private screenshot warning, recipient's own login, explicit
  export/deletion previews, permanent-deletion consequences and active/retained/
  shared-source protections are unchanged. Exact provenance remains on demand.
  No automatic navigation, login, baseline activation, export or deletion is
  added. Existing request identities, stale-reply guards and draft owners remain.

Impeccable/context was loaded once; incumbent native pixels/source and the desktop
surface brief were read. The craft floor was read before UI editing. The known
adaptive/mobile metadata mismatch was neither applied nor repaired.

## Tests and finding map

Four focused regressions are in `crates/app/src/phase4_hierarchy_tests.rs`:

| Finding / requirement | Asserted coverage |
|---|---|
| A4 hierarchy and progressive disclosure | Actual summary/preset/disclosure order at both sizes; hidden mutation controls cannot queue work |
| Advanced controls in the real inner owner | Mounted Tab traversal and full labelled bounds of every numeric field and both tail switches at both sizes; actual Reanalyze validation reveals the complete error field/correction and queues nothing |
| Ignored-area ownership and validation | All four coordinate fields reachable; unapplied Save rejects; all-pixel coverage rejects before requests; hidden Apply does nothing; exact pair/draft survives close/reopen; no stored evidence change |
| Reanalysis/revisions | Real isolated worker, Sensitive saved policy, retained previous revision, scheduled Cancel, closed-editor polling cannot schedule work; active baseline preserved |
| History hierarchy and scope | Real stored 64-run page plus second page; last loaded row revealed in list and outer owner; selected action inside record details; no-match action is unmounted; optional provenance |
| Sparse/bounded lists and Settings terminology | Exact count-to-height cases 0/1/2/3/4/64/128 and App settings navigation label |

Incumbent drawing/coordinate tests now open the real Ignored areas disclosure.
The pointer test asserts drawing's completed state rather than overwriting it.
`phase3_note_tests.rs` now finds the exact pending key/text instead of assuming
unstable collection ordering; its byte/ownership assertions are preserved.

The full app gate also exercises the incumbent `phase4_review_tests` (eight),
`phase6_history_tests` (two), `options_ux_tests` (four), `phase3_note_tests` (three),
generic/multiline input tests and consent/setup regressions. In particular, it
retains exact review-note/target/reply ownership, historical saved-policy reads,
missing/corrupt-original failure, consent cancellation without writes, off-section
History controls, bounded options and drawing-toolbar escape coverage. No tests
from the separate Phase 5 plan are newly implemented here; incumbent modules with
older `phase5`/`phase7` names are simply included in the existing package gate.

Common environment: `LIBRARY_PATH="$PWD/.runtime/link-libs"` and
`TMPDIR=/var/home/olav/Data/opencode/tmp/opencode`. Cargo gates use
`--locked --offline`; executed tests use `-- --test-threads=1`.

All seven final-source gates completed successfully in the authoritative final
confirmation root below. `gates.json` records exact commands, exit codes, duration
and the same 22-source manifest match **before every gate**;
`final-check-receipt.json` binds their complete logs and asserts module counts and
the privacy/retention/consent successes. This is not a whole-workspace test claim.

| Exact final command | Result / log in final root |
|---|---|
| `cargo test --locked --offline -p regressee-app --all-features -- --test-threads=1` | **251 passed**: 98 library + 144 binary + 9 integration; zero failed, 45 opt-in tests ignored, doc-tests passed. `all-features-app.log` |
| `cargo test --locked --offline -p regressee-core shell:: -- --test-threads=1` | **11 passed**. `core-shell.log` |
| `cargo test --locked --offline -p regressee-app --features ui --lib --bin regressee --no-run` | Passed; no accidental ui-tests feature coupling. `ui-only-no-run.log` |
| `cargo build --locked --offline --workspace --features ui --bins -j 4` | Passed. `build.log` |
| `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings` | Passed. `clippy.log` |
| `cargo fmt --all -- --check` | Passed. `format.log` |
| `git diff --check` | Passed; also rerun after final documentation edits. `whitespace.log` |

The final full app command completed in **824.67 seconds**, including the incumbent
distinct 100/1,000-page persisted-storage qualification (**704.72 seconds**).
Explicit log checks found all four new hierarchy tests, all eight incumbent
Phase 4 review tests, both Phase 6 History tests, all four options tests, three
note controller/storage tests, 12 generic/multiline input/style tests, 11 exit
guards, five review/mask unit tests, 18 workflow-refinement tests and seven
History unit tests. Consent, ZIP privacy/shared-deletion and reanalysis/cache/
all-masked/review-history integration regressions also passed. The separately
executed opt-in proof and final 88-frame native recipe passed; no unrelated opt-in
runtime/website tests were silently treated as executed.

Earlier successful pre-repair gates are historical only, under
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase4-native-20261014-3d1n1zfh`
and `regressee-phase4-final-gates-20261014-4lng9096`. Their source receipts remain
unchanged and are not substituted for the final same-source results. The incumbent
`proc-macro-error2 v2.0.1` future-incompatibility notice remains; it is not a new
Clippy warning. No functional failure remained in the completed confirmation or
final gates.

## Native evidence and bounded review

`crates/app/src/ui_phase4_capture.rs` is an explicit ignored two-size recipe.
It constructs isolated offline few/empty/many projects, uses actual mounted
Tab/Enter-style activation for controls and supported wheel dispatch for static
reading. Capture records include actual geometry, clip ownership, focus, enabled
membership, policies/revision IDs, mask pair/draft, record counts, page/query,
preview/provenance and active-baseline state. Each coordinate field is captured
when actually revealed, not claimed to be simultaneously visible.

Active-analysis painting deliberately constructs a pending-worker state; real
worker completion/revision retention is separately tested. Export and deletion
are only previewed; no ZIP is exported and no evidence is deleted. Website imagery
is reused only from the explicitly labelled **synthetic offline** original review
fixtures. Every app PNG must be fresh, with a personally inspected renderer proof
and exact source binding; no user projects/websites/credentials are used.

Structural recipe attempts are retained, not substituted as accepted evidence:

1. `regressee-phase4-native-20261014-3d1n1zfh`: inner action visibility failure;
   current painted action reveal repaired.
2. `regressee-phase4-inspection-20261014-zg2bqhkk`: the recipe incorrectly required
   all coordinate rows after revealing the last field.
3. `regressee-phase4-batch-20261014-c3zry3tb`: revealing the first field alone did
   not establish simultaneous visibility of later rows; separate field captures
   replace that unsupported assertion.
4. `regressee-phase4-complete-20261014-4jv2asxn`: Close focus was checked before
   its presentation frame. The recipe now activates, draws, then checks actual
   trigger focus. Later no-PNG preflight exposed an active-baseline deletion
   attempt; the preview recipe now uses a selected check run instead of bypassing
   deletion protections. A 120-second preflight timeout is retained separately.
   Further preflight corrections use actual mounted empty-state ownership rather
   than cached absent selectors, filter a real loaded run ID rather than assume
   checks are on the insertion-ordered first page, and reset presentation scroll
   when constructing an unrelated empty-project scene. None changes production
   storage ordering, filtering or deletion authorization.

These roots are under `/var/home/olav/Data/opencode/tmp/opencode/`. Original pixels,
logs, proof and source metadata remain intact. No app pixels from these failed
attempts were used for visual polish. `REGRESSEE_PHASE4_CAPTURE_PREFLIGHT=1` runs
the same full state/geometry sequence without rendering app PNGs and writes only
`phase4-preflight-matrix.json`; this is not native screenshot evidence.

**Complete initial review:**
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase4-native-review-20261014-nlvuc1bp`.
Its native capture passed in 237.64 seconds: **86 fresh app PNG/JSON pairs**, 43 at
each desktop size. All originals passed hash/dimension/source/clip verification
(31 embedded source fingerprints per pair, 22 manifest source files). Every full
frame was personally reviewed on labelled contact sheets, all 50 Analysis
inspector crops at native scale, and 19 important History/settings originals at
original resolution. The exact list and observations are in
`inspection-ledger.json` (SHA-256
`eeb3f5481ab6da2dfec2a40e4e44a2abbbce88334effaa76236f522890377fff`).

One collective repair addresses its five findings:

- **P4-I1:** the invalid capture now invokes actual Reanalyze validation, rather
  than showing a first incomplete edit before shared input validation is exposed.
  Numeric values/touched/composition/errors are recorded. The strengthened test
  proves no work is queued and full correction geometry is inside the real owner.
- **P4-I2:** metrics capture now wheels to the actual report summary and a separate
  numeric/dimension tail, with measured selector/clip assertions, not just an open
  disclosure button.
- **P4-I3:** ignored-area task status uses actual unapplied/draft/saved/empty state
  and correct singular/plural; it no longer invents a saved revision for zero areas.
- **P4-I4:** History's task-level ZIP summary is shorter and does not wrap a lone
  semicolon; authoritative export-preview privacy/login warnings remain intact.
- **P4-I5:** App settings removes the duplicate checking fallback, retaining real
  optional runtime details and the shared state/help.

Structural test corrections within this repair use the real validation action
(rather than relying on deferred simulated blur callbacks) and the actual retained
rectangle count across both-size traversal. All four strengthened focused tests
passed in 57.41 seconds. No extra visual-polish round has been taken.

**Authoritative final confirmation root (0700):**
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase4-confirmation-20261014-gd98l_63`.
Its fresh renderer proof was personally inspected and hash-bound before captures.
The sole native confirmation passed in **224.20 seconds**: **88 fresh app PNG/JSON
pairs**, 44 at **1280×800** and 44 at **820×600**. Every full frame was personally
inspected on labelled contact sheets, all **52 Analysis crops at native scale**, and
**18 important History/Settings originals** at original resolution. The exact
hash-bound frame/method list and five repair observations are in
`inspection-ledger.json`. No new functional/layout blockers were observed; no
additional visual-polish/capture round was taken. This is the implementer's
confirmation, not an independent verdict.

`verify-evidence.py` passed all PNG/metadata hashes, dimensions, 31 embedded source
bindings per pair, all 22 current manifest sources, proof receipt and measured
selector clips. `check-final-state.py` passed both-size numeric error/touched/
composition/no-pending-work, preset/displayed-policy/revision retention, pair,
mounted focus, counts/filter/page, preview/provenance, active-baseline and synthetic
fixture hash checks. Their exact outputs are `verification-receipt.json` and
`state-verification-receipt.json` in this root.

The real Analysis inner clips are **358×496px** at normal size and **358×312px** at
minimum size. Each invalid labelled field is **326×144px**, with its complete
**326×18px** correction inside that owner. Few-run/baseline list heights are
**192px / 64px**; 64 loaded runs use **256px**, then the actual second page has
**7 runs**. A one-match and zero-match filter still have 64 loaded records on
page 1; they never claim project-wide search. Both actual metrics selectors fit
the same native reading view in this fixture; separate records do not imply a
different scroll position. Surrounding content may intentionally lie beyond the
current bounded viewport; only actually revealed selectors are claimed visible.

Private pixels/databases are not committed. A proof-only preliminary root
`regressee-phase4-review-20261014-5k2b5bfy` and the initial root's
`pre-final-recipe-proof/` are also preserved; neither is accepted app evidence.

Reproduce in a **new canonical 0700 root**, with the labelled synthetic website
fixtures copied there and their hashes/provenance recorded:

```sh
export LIBRARY_PATH="$PWD/.runtime/link-libs"
export TMPDIR=/var/home/olav/Data/opencode/tmp/opencode
export REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode
export REGRESSEE_RENDERER_CAPTURE_DIR=<new-private-root>
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::renderer_capture_proof -- --ignored --exact --test-threads=1
# Personally inspect the proof before writing its hash-bound proof-reviewed.json.
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::ui_phase4_capture::renderer_capture_phase4 \
  -- --ignored --exact --test-threads=1
```

Leave `REGRESSEE_PHASE4_CAPTURE_PREFLIGHT` unset for native captures. The recipe
renders 44 states per size, including separate metrics-summary and metrics-tail
reading states. Structural preflight is not a substitute for these pixels.

### Finding-to-capture map

Names below follow `phase4-<1280x800|820x600>-<scene>.png`; the matrix binds actual
state and measured visible selectors rather than relying on these names alone.

| Requirement | Scene suffixes at both sizes |
|---|---|
| Actual result/settings, common choices first | `analysis-basic`, `analysis-no-significant-change`, `preset-sensitive`, `preset-relaxed`, `preset-balanced` |
| Meaningful active/previous controls | `active-analysis`, `previous-analysis` |
| Optional ignored-area workflow and keyboard alternative | `ignored-disclosure`, `ignored-coordinate-0` through `ignored-coordinate-3` |
| Every advanced field and both tail switches | `advanced-field-0` through `advanced-field-8`, `advanced-height-switch`, `advanced-switch-tail` |
| Genuine numeric correction, no scheduled work | `advanced-invalid-revealed` |
| Optional actual metrics and dimensions | `analysis-metrics`, `analysis-metrics-tail` |
| Sparse/empty lists, selected-action proximity | `history-few-runs`, `history-selected-run`, `history-few-baselines`, `history-selected-baseline`, `history-zero-runs`, `history-zero-baselines` |
| Bounded lists and loaded-page filtering/pagination | `history-many`, `history-last-run`, `history-page-two`, `history-filter-one-run`, `history-no-matches`, `history-last-baseline` |
| Exact provenance only on demand | `history-provenance` |
| Sharing/privacy/login and protected deletion previews | `export-privacy`, `export-exact`, `delete-protections`, `delete-exact` |
| App settings hierarchy | `app-settings` |

## Exact final source bindings

The final root's `source-provenance.json` binds 22 application, test, capture and
incumbent UI sources. Each PNG additionally embeds 31 source fingerprints. The
current changed and supporting-source bindings are:

| Source | SHA-256 |
|---|---|
| `crates/app/src/main.rs` | `37b0beca6aeea7391d3545f6406a6a22007e58b51b50242aae762fe0172cae6b` |
| `crates/app/src/analysis_ux.rs` | `39698fd720fbea09d4c4fbc2ff73a34f7ef200f1440f31e212d63c62d25c0b76` |
| `crates/app/src/workspace_history.rs` | `be539e9f7ebc6e9a343be1ad620167e8e5684991d46b796c5a3654d88d65c474` |
| `crates/app/src/workflow.rs` | `31719c03e32dd5e698ce7cd1698b804cf9190b02f85c069c81f9d603951c6500` |
| `crates/app/src/review_ux.rs` | `625eff16731b37116aa7d24b295b25ae76bc6411bc03ed126a07554a15bf2b63` |
| `crates/app/src/viewer_ux.rs` | `bb8a1e5ee0ae752813eccc1f7c8d51cfc3def3c1af51674d92e8663b8f1709be` |
| `crates/app/src/phase4_hierarchy_tests.rs` | `6462d96bcfae273e5a8164006d219fdd8f84cba26851762b4f17bb4c85bb9317` |
| `crates/app/src/ui_phase4_capture.rs` | `e2cd1ed3f901e650b9e50f16bb6e5cc7cb060d4f88b05956e4a25f9d649e18a9` |
| `crates/app/src/ui_renderer_capture.rs` | `84d81012d647b0081e42946ff856f032cc7f1e8db3cff4040fb631532868fa0b` |
| `crates/app/src/phase2_evidence_tests.rs` | `098f28b69da8bb35fdcf1cc8dd0d4c475fde3fe5bba4df3e0b3cef816dc6231d` |
| `crates/app/src/phase3_note_tests.rs` | `b3ff14b3e45dfc98cc113f11b16750b43efbdfbd98bc986dc76aca85b2fdba3f` |
| `crates/app/src/workflow_refinement_tests.rs` | `5557a6a650975d1ed9cd8ecf79fc54b7960de6ca0fbe1a82b7d2d5eac14ce6ae` |
| `crates/core/src/shell.rs` | `1a06dd579330f10a078ac0ca5b1f732eed28742de6d8f2be7d2d5eb1a5877c7a` |
| `crates/app/src/evidence_workspace.rs` (unchanged) | `1f66704c0bd4d09c0b57e13ae631dc79fadae3e30ded0305e2393d35641eb9f4` |
| `crates/app/src/ui_review_matrix.rs` (unchanged fixture) | `8070496a821b965b45229278180c737c51fb517ad28d734f919ef39b57655d7d` |
| `crates/app/src/ui/controls.rs` (unchanged) | `0435e4ba22195bb18b2daef3f0f0238d601acbdc75ddc925baba2a38d62d21c5` |
| `crates/app/src/ui/theme.rs` (unchanged) | `68f6f1a3955c7bdb5dd5f684c2541d2c1ccd69553d3069209c0c81e0441debf4` |
| `crates/app/src/ui/text_input.rs` (unchanged) | `c81cb2c03508167f04dd73ad335b072ace95734304632480b272a6b5dcf1c003` |
| `crates/app/src/ui/text_input/multiline.rs` (unchanged) | `ffdbe32a94b360ba969c5540e5684f299538158c74132dbf1416e706b048b92c` |
| `crates/app/src/ui/selector.rs` (unchanged) | `5bb36ffd5c1fb00905234cfa58fb90b5bdde672bd134d38097e200c1a1fdd97f` |
| `crates/app/src/ui/zoom_select.rs` (unchanged) | `6b686974eb53139fc759f3c378b6d304ec74373be10a2bfdf08a5be5dcc791a1` |
| `crates/app/src/ui/timezone_select.rs` (unchanged) | `2b7b995fa3fc0962c4bb1c794ddd811e0beda54ac53d181ab1b90b1ea98e19a2` |

Final confirmation receipt hashes:

| Artifact in the final root | SHA-256 |
|---|---|
| `source-provenance.json` | `4768989fc9244417c27619fe80a2bb65b85fb73b9f6abcae450fdc83896cb53d` |
| `phase4-matrix.json` | `b710bcfcf612ba41a0bc97197da3c8d53694697c6cb0ecda839208da2ef3293b` |
| `inspection-ledger.json` | `809839bf409568c4f1e4d4a6727236fd0d6e7f07bcfc19e6df4e295367b29bbb` |
| `verification-receipt.json` | `b2b54af524fd13d7652d87488c3f91fe8b4646062566352c605bec1d81e3c7f4` |
| `state-verification-receipt.json` | `6d7d33febea14625065b271aa5cfc34c7ad90e9d04399ce975dd55e802291cc0` |
| `website-fixture-provenance.json` | `dd83355bbd9a5c9e42a7c7cd3ed81f1e956afc87e459712f80a8138c675641c4` |
| `renderer-proof.png` | `958c19dbf6f90b30df1f1ba9f7efd7b43dc03fc03edde2d4a3d9e89ac9c60c5d` |
| `proof-reviewed.json` | `08b852e825c40e64cbebc0a49e37daa9386baf260db0eb21318e811ad5df256f` |
| `gates.json` | `254fdea67f7a193aa7ef5514953603a89e39896df7dade6a4bdf0923f2f9f375` |
| `final-check-receipt.json` | `e60e45c203eb6a4feb4e5a5375be1671a731a8c1711282846887f529d79e2ac0` |

## Verifier handoff

Implementation, bounded native confirmation, personal inspection and final checks
are complete. The parent may now assign the independent Phase 4 verifier. Start
with this report, the exact current source table, final root's `phase4-matrix.json`,
original PNG/JSON pairs, `inspection-ledger.json`, `source-provenance.json` and
`final-check-receipt.json`. The root's `verify-evidence.py` and
`check-final-state.py` reproduce integrity/state checks against the current source;
`make-inspection.py` generates labelled derivatives without modifying originals.
The complete initial ledger and original failed/preflight evidence remain available
for the bounded repair history. There is no independent PASS/FIX verdict yet.

No child agent, commit or push has been made. Prior PASS/push rows and the parent's
Phase 3 push-row edit are preserved. Phase 5 is still pending and no Phase 5-only
implementation or DESIGN/metadata update has been performed.

## Qualification limits

This work qualifies Linux **GPUI/Blade offscreen paint and simulated controller
behavior**, not real OS window/input/clipboard delivery, OS IME, assistive
technology, high-DPI/high-contrast, Windows/macOS, usability or release acceptance.
No native markup detector is claimed: its browser coverage does not cover Rust/
GPUI. No accessibility bridge, new font/runtime distribution, theme, animation or
backend comparison algorithm is introduced. A separate verifier must issue the
Phase 4 verdict; this implementation report does not self-certify PASS.
