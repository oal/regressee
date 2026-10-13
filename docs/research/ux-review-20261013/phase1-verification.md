# Phase 1 independent verification — 2026-10-13

## Verdict: FIX

**Phase 1 is not accepted yet.** Two concrete acceptance repairs are listed below.
Return both to the same implementer for one repair batch, then return the repairs
to this verifier. Do not advance phases, commit or push before a scoped PASS.

The connected wizard, contextual consent, Access/Pages hierarchy and product
language are otherwise coherent in the inspected native pixels. No additional
confirmed visual clipping/overlap blocker was found. Passing implementation tests
and source-bound screenshots did **not** conceal the two functional/build gaps.

### V1-1 — [P2] Normal New Project entry never starts browser availability observation

**Source:** `crates/app/src/main.rs:2927–2971` (`create`) changes the screen to Setup
and notifies, but never calls `observe_setup_runtime` or `refresh_visible`.
The new read is started only by `refresh_visible` at `main.rs:1290–1293`.
`poll_storage` (`main.rs:533–562`) does not independently refresh an idle screen.
After the initial Projects read has finished, creating a project therefore leaves
`runtime_status`, `runtime_readiness` and `runtime_pending` all absent indefinitely
until another action, such as saving or navigation, happens to trigger a refresh.
`runtime_ux.rs:11–14` presents that absence as an active “Checking browser setup”.

**Independent functional reproduction:** a scratch-only GPUI test settles the
initial Projects read, activates the actually mounted/enabled `NewProject`
control, draws and polls five times, without injecting a refresh. It fails:

```text
NewProject after draw/poll: runtime_pending=None, runtime_status=None,
runtime_readiness=None, next_request=1
actual mounted NewProject entry must schedule availability without calling
refresh_visible from the test
```

**Why the passing evidence missed it:**
`phase1_runtime_tests.rs:16–17` explicitly calls `refresh_visible` after `create`.
The capture recipe loads a saved synthetic project before its `website-blank`
branch (`ui_phase1_capture.rs:94–107`), so availability is already read before
New Project activation. The final blank Website pixels truthfully show the
constructed/read fixture state, but do not qualify the normal first-run entry.
The seven runtime scenes inject presentation states, not this entry transition.

**Required repair / acceptance:** start the existing once-only, independent,
off-thread availability observation from the real fresh-project entry. Preserve
`pending`/save/login gates and draft/focus identities; do not install, visit a site
or turn this into an exclusive task. Add a regression that starts from fully
settled Projects and uses mounted `NewProject` (also cover the existing new-project
shortcut path), with no test-side refresh helper. Observe Checking → authoritative
availability/failure, verify local saving remains enabled, and verify no second
read on ordinary redraw/re-entry. In the bounded confirmation, use an actual
fresh-entry recipe at both sizes; reveal the runtime note in its real scroll clip
where necessary. Keep the original batch, and bind confirmation evidence to the
repaired source rather than silently relabeling old pixels.

### V1-2 — [P2] New test-module gates break the ordinary UI-only test build

**Source:** `crates/app/src/main.rs:8069–8072` gates both new Phase 1 test modules
on `test + ui`. Their GPUI simulation, fixtures, `start_without_runtime`, and
`workflow_refinement_tests` imports require `ui-tests`/`test-support`, as surrounding
test modules already declare. All-features execution masks the mismatch.

**Exact current-source failure:**

```sh
cargo test --locked --offline -p regressee-app --features ui --bin regressee --no-run
```

Exit **101**, **18 compiler errors**, including missing `gpui::TestAppContext`,
`gpui::run_test`, `workflow_refinement_tests`, `approval_fixture` and
`Jobs::start_without_runtime`. This is introduced by the new modules; their
all-features tests and the ordinary UI application build do pass.

**Required repair / acceptance:** gate simulation-dependent tests consistently
with the existing `ui-tests` feature, or explicitly provide their required feature
dependencies. Do not weaken assertions, add production test-only runtime behavior,
or strip activation guards. The command above must compile successfully, while
`phase1_setup_tests` (4) and `phase1_runtime_tests` (1 plus the entry regression)
still execute under the qualified test feature. Re-run the focused regressions,
native UI build, formatting and whitespace checks.

## Scope and independent method

- Read the complete `docs/ux-improvement-plan.md`, Phase 1 implementation handoff,
  incumbent assessment and all changed source/test files, including new files.
- Loaded **impeccable** and ran its context once. Preserved native desktop
  **Clear wayfinding**; did not follow the known adaptive/mobile metadata mismatch
  into iOS/Android work or repair unrelated design-context drift.
- Base/HEAD: `5f136b7819c3663517d2f9bd5e17f7203e5b262e`.
- Reviewed only Phase 1. No production/test-source edits in the repository, child
  agents, phase advancement, plan-status edits, commits, pushes or new native
  capture batch. The only repository addition by this verifier is this report.
- Independent probes were added only to a private scratch copy of the current app
  sources. Production logic was unchanged; the copy merely includes one additional
  test module. Current-source application/test binaries were rebuilt afterward
  and again match the original evidence manifest hashes.

Private evidence aliases used below:

```text
B = /var/home/olav/Data/opencode/tmp/opencode/regressee-ux-review-20261013-9g3qdv3j
E = /var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase1-final-20261013-AiZyyVOR
V = /var/home/olav/Data/opencode/tmp/opencode/regressee-phase1-verifier-20261013-n4gvslww
```

## Pixels, source binding and geometry

Personally inspected **all 60 final originals at full resolution**, as well as all
ten paired `E/contact-sheets/phase1-{01..10}.png`. These are the existing complete
fresh batch: 30 states each at **1280×800** and **820×600**. Also inspected the
renderer proof and eight original baseline images. No handoff filename or passing
assertion was treated as sufficient state evidence: inspected source construction,
guarded activation/reveal, manifest state and actual paint together.

Independent integrity command:

```sh
python3 /var/home/olav/Data/opencode/tmp/opencode/regressee-phase1-verifier-20261013-n4gvslww/verify-integrity.py
```

Result: **PASS**. Matched **17/17 current source fingerprints**, **60/60 original
PNG hashes and dimensions**, **13 embedded current source bindings per PNG**
(eight Phase 1 bindings plus five incumbent bindings), proof/receipt hashes,
recorded geometry containment and progress-state consistency, HEAD, and private
0700 root. Current app/test binaries match `source-provenance.json`. The eight
inspected baseline PNG hashes/dimensions match their sidecars and their embedded
`main.rs` fingerprint matches `git show HEAD:crates/app/src/main.rs`.
The receipt is `V/independent-integrity.json`; the complete personally inspected
inventory is `V/inspection-ledger.json`. This validates provenance and internal
geometry consistency, not real OS input or the missing entry transition.

Key current-source fingerprints:

```text
main.rs              960f291564d4d24caea2d8b3c80055990329ae3c6fb247068315ba9d7434276c
workflow.rs          fc008adb32ec1fa9a9b4d3c1b951bf3e4c3f236dc72ae17a3c01bce0d50b3a1c
setup_ux.rs          9c3f506c6bfdf61662c00dbf5e05fcb9c11bf14f658dbe2892a9c5a20ced48be
runtime_ux.rs        c498cf64584de2eaf6e79b205806aabe369e05b9a264612cafdb85aac3ce0dab
ui/setup_progress.rs 6a1cb34a4eab4e57508c8d88663b46ffd334c40ed9b61ac3ff4cbee26c018f03
```

Source-provenance manifest SHA-256:
`9e4c5a843c2f0dcb11e89b401564ff9bfd35a1259b30d762d0cc499f5e2f1506`.
Proof PNG SHA-256:
`958c19dbf6f90b30df1f1ba9f7efd7b43dc03fc03edde2d4a3d9e89ac9c60c5d`.

### Complete bounded inspection ledger

Each row below covers both sizes, on its contact sheet **and in the originals**,
named `E/phase1-<size>-<state>.png`.

| Sheet | States | Independent observation |
|---|---|---|
| 01 | `website-blank`, `website-single-origin`, `website-multiple-unapproved` | Numbered, connected, readable Current/Upcoming progression. Blank/single origin omit consent; multiple origins show an unchecked authorization. Fixture blank does not qualify fresh-entry observation: V1-1. |
| 02 | `website-multiple-approved`, `website-origins-changed`, `website-invalid` | Real checked/unchecked marks track consent/reset. Invalid address/error/help is revealed; current Website retains full selected treatment. |
| 03 | `website-back-dirty`, `access-public`, `access-private-unsaved` | Back retains dirty access; future stages gate. Public/login decision leads, staging/probes do not compete. Unsaved private footer promotes Save first and disables login/ready. |
| 04 | `access-private-saved`, `access-private-status-loading`, `access-browser-open` | Saved private promotes Open login; absent status disables it; constructed browser-open promotes I'm ready. States and prerequisites agree with code/manifest. |
| 05 | `access-ready`, `access-options-open`, `access-mapping-revealed` | Constructed saved session permits Continue. Optional environments are grouped; focused mapping field and help fit the actual scroller. |
| 06 | `access-options-closed-draft`, `access-provider-revealed`, `pages-essentials` | Disclosure hides optional fields without erasing mapping-editor text. Required private provider field remains reachable without optional disclosure. Plan names dimensions, extent, images, page mode/limit and speed before customization. |
| 07 | `pages-invalid-summary`, `pages-invalid-revealed`, `pages-custom-tail` | Invalid Pages stays Current; recovery reveals the real invalid field/error/help. Custom width/height/action remain available. |
| 08 | `pages-limitations`, `runtime-checking`, `runtime-not-installed` | Full limitations text retained/readable; checking promises no download/visit; supported not-installed alone promises first-use installation. |
| 09 | `runtime-ready`, `runtime-development-only`, `runtime-repair` | Ready is explicit-action only; development is local-fixtures only; repair requires App settings. Copy wraps legibly at the minimum window. |
| 10 | `runtime-unavailable`, `runtime-unsupported`, `about-regressee` | No installation promise in unavailable/unsupported states. About names Regressee/version/license and preserves local storage, ZIP/session privacy, screenshot exposure and qualification caveats. No rail slogan. |

Baseline originals inspected in `B`: `all-{1280x800,820x600}-new-website`,
`all-{1280x800,820x600}-access-public`, `all-820x600-pages-invalid`,
`all-1280x800-pages-sizes`, and `all-{1280x800,820x600}-about`.
They establish the original disconnected/dimmed step buttons, always-present
consent, default staging/probe clutter and preview branding. The final batch
visibly improves those concerns without replacing the visual world.

Example measured bounds, independently containment-checked and matched to paint
(`x,y,width,height`, logical pixels; not estimates):

| Element / 820×600 scene | Painted bounds | Actual form scroll clip |
|---|---|---|
| Multi-origin consent | `229,481,578,44` | `192,235,628,298` |
| Mapping field | `216,424,580,101` | `192,235,628,298` |
| Identity-provider field | `216,371,580,123` | `192,235,628,267` |
| Invalid parallel-pages field | `216,381,580,123` | `192,235,628,277` |
| Complete capture limitations | `216,261,580,159` | `192,235,628,277` |

The progress component is pinned above the scroll owner, with three 52px-high
stages and actual 16px connectors. Current derives from stage position, not draft
validity. Gated Upcoming text is not faded disabled-button paint. Both-size Save
footers remain contained. At the minimum size, optional disclosure/provider/custom
controls legitimately require scrolling/Tab reveal; this is not claimed to be
first-viewport visibility. A full-frame preview appeared to omit a progress label;
an independently decoded crop of the same original shows all three stages:

```sh
ffmpeg -v error -i /var/home/olav/Data/opencode/tmp/opencode/regressee-ux-phase1-final-20261013-AiZyyVOR/phase1-820x600-access-public.png -vf 'crop=580:64:216:163,scale=1160:128:flags=neighbor' -frames:v 1 /var/home/olav/Data/opencode/tmp/opencode/regressee-phase1-verifier-20261013-n4gvslww/progress-access-public.png
```

Inspected that crop; no missing-step defect in the original is established. It is
a derivative of existing pixels, not an extra renderer/polish batch.

## Functional and async audit

- **Progress/back/save:** `setup_ux.rs:75–94` allows safe backward/current
  navigation, gates unsaved/composing future stages and retains draft fields.
  Existing exact-save continuation and clean Continue tests pass; clean Continue
  does not rewrite settings. Busy/task/modal guards were not removed.
- **Consent:** `setup_ux.rs:29–45` derives distinct origins and clears consent
  when the set changes. Save still validates the origin set at `main.rs:3054–3062`.
  Both ordinary and preserved-draft save replies refresh origins. The new supplied
  race test passes; an independent probe additionally uses the actual address
  entity observer, without manual origin refresh, then edits origins during save:
  observer/reset/newer draft all survive and no visit starts.
- **Mounted focus and disclosures:** `workflow.rs:1342–1382`, core stage mounting,
  `control_mounted`/`control_enabled` and `field_enabled` agree with rendered
  optional/public/private groups. Tab uses enabled visible controls and reveals
  the owned form bounds, not disconnected test handles. Existing two-size
  capability traversal/activation tests pass. Hidden mapping/probe/environment
  controls reject activation; Escape restores disclosure trigger and retains text.
  Private provider/session controls stay discoverable. Full Project settings
  retains its environment/login operations; Pages retains custom sizes and limits.
- **Runtime separation:** `runtime_ux.rs:85–129` submits only `Work::RuntimeStatus`,
  records a separate request ID, rejects stale replies and avoids save hydration,
  focus/selection changes and installer/task creation. `apply_update` consumes
  that read before ordinary pending mutation handling. An independent concurrent
  runtime-read/save probe passes: matching runtime reply does not clear the save
  pending ID/busy state or signature; later save preserves newer fields and stays
  on Website. The worker's `RuntimeStatus` calls read-only readiness
  (`jobs.rs:784`, `product.rs:110–227`); it does not call setup/ensure-runtime,
  login, discovery or website requests. Entry scheduling remains V1-1.
- **Runtime/product copy:** Setup and App settings share `runtime_presentation`.
  Checking, supported not-installed, ready, development-only, repair, unavailable
  and unsupported wording is differentiated; local saving stays available. The
  helper also covers development not-installed and installation-unavailable cases.
  Supplied presentation tests plus backend readiness/capability tests pass. These
  are capability/presentation tests, not actual runtime installation qualification.
- **About/branding:** rail slogan removed with no replacement; entry/dialog
  “About Regressee”; factual package version/license and privacy/platform caveats
  retained. The remaining Settings-heading terminology belongs to Phase 4, not
  a new Phase 1 defect.

## Exact commands and observed results

Repository cargo commands ran from `/var/home/olav/Projects/Regressee` with:

```sh
export LIBRARY_PATH="$PWD/.runtime/link-libs"
export TMPDIR=/var/home/olav/Data/opencode/tmp/opencode
```

| Command | Independent result |
|---|---|
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee phase1_setup_tests -- --test-threads=1` | 4 passed, 0 failed. |
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee phase1_runtime_tests -- --test-threads=1` | 1 passed, 0 failed; has the entry-coverage gap described above. |
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee phase5_setup_tests -- --test-threads=1` | 4 passed, 0 failed. |
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee phase5_repair_tests -- --test-threads=1` | 5 passed, 0 failed, including two-size mounted capabilities/validation and save races. |
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee options_ux_tests -- --test-threads=1` | 4 passed, 0 failed. |
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee validation_correction_reveals_the_right_stage_without_submission_and_help_restores_focus -- --test-threads=1` | 1 passed, 0 failed. |
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee -- --test-threads=1` | 128 passed, 0 failed, 17 ignored; run before probes and again from current source afterward. |
| `cargo test --locked --offline -p regressee-app --all-features --lib product:: -- --test-threads=1` | 12 passed, 0 failed. |
| `cargo test --locked --offline -p regressee-app --all-features --lib jobs:: -- --test-threads=1` | 13 passed, 0 failed. |
| `cargo test --locked --offline -p regressee-core shell:: -- --test-threads=1` | 11 passed, 0 failed. |
| `cargo test --locked --offline -p regressee-app --features ui --bin regressee --no-run` | **Exit 101; 18 compiler errors — V1-2.** |
| `cargo build --locked --offline --workspace --features ui --bins -j 4` | Exit 0; repeated after scratch probes. |
| `cargo fmt --all -- --check` | Exit 0. |
| `git diff --check` | Exit 0. |

Logs are under `V`, named for the test filters, plus `app-regressions.log`,
`product-runtime.log`, `jobs-runtime.log`, `core-shell.log`, `ui-only-test-build.log`,
`ui-build-restored.log`, `fmt.log` and `diff-check.log`. Exact command arrays/results
are in `V/commands.json` and `V/additional-commands.json`.
The existing `proc-macro-error2 v2.0.1` future-incompatibility warning is not a
new error. Ignored opt-in captures/qualification tests are not counted as passes.
Full workspace/Clippy/final integration gates remain the later planned phase;
the implementer's reported runs were not substituted for independent results.

Independent scratch probes (same production logic, extra test module only):

```sh
cargo test --locked --offline --manifest-path /var/home/olav/Data/opencode/tmp/opencode/regressee-phase1-verifier-20261013-n4gvslww/shadow/Cargo.toml --target-dir /var/home/olav/Projects/Regressee/target -p regressee-app --all-features --bin regressee independent_phase1_probes -- --test-threads=1 --nocapture
```

Result: **2 passed, 1 failed**, exit **101**. Consent-observer/save race and
runtime-reply/save separation pass; actual fresh New Project observation fails
(V1-1). Exact source, command receipt and output are retained in
`V/shadow/crates/app/src/independent_phase1_probes.rs`,
`V/independent-probes-command.json` and `V/independent-probes.log`.
Initial scratch scaffolding lacked the timezone asset and used a private library
constructor; those verifier-only scaffolding errors were corrected before the
reported three-test run, without changing production code or guards.

## Qualification boundary and handoff

This is independent Linux GPUI/Blade offscreen paint/source/controller verification
with simulated GPUI input and isolated local synthetic projects. It does **not**
establish real OS keyboard/pointer/IME, assistive technology, accessibility bridge,
high-DPI/high-contrast, Windows/macOS, live login/network, runtime distribution,
usability or release qualification. Those existing limits are not additional
Phase 1 acceptance repairs.

**Collective repair list is V1-1 and V1-2 only.** Preserve the accepted visual
improvements and safety guards; no further aesthetic iteration is requested.
The parent owns plan status and any later commit/push after re-verification.

---

## Re-verification — 2026-10-13, same implementer, repair batch 1

### Latest verdict: PASS — whole Phase 1, bounded scope

**V1-1 and V1-2 are resolved. Phase 1 meets its scoped requirements; no remaining
Phase 1 acceptance repair is identified. The parent may now update phase status
and commit/push Phase 1.** The original FIX above is preserved as historical
evidence, not rewritten into a retroactive PASS.

Read the appended correction handoff and current source, compared the repair
against the retained verifier snapshot, independently ran the reproductions and
regressions below, and personally inspected the four fresh entry originals and
fresh renderer proof. No production/test-source or plan-status edits, commits,
pushes, subagents, extra native capture batch or aesthetic iteration. This
appended report is the only repository change by the re-verifier.

Evidence aliases for this confirmation:

```text
R = /var/home/olav/Data/opencode/tmp/opencode/regressee-phase1-repair-20261013-a0koej7c
RV = /var/home/olav/Data/opencode/tmp/opencode/regressee-phase1-reverification-20261013-2s806wbd
```

### V1-1 closure — actual entry, independent read and concurrent save

- `main.rs:2971–2973` now invokes the existing `observe_setup_runtime` after the
  guarded create transition and Name focus. Mounted NewProject dispatch
  (`main.rs:4779`) and the CreateProject action/production Ctrl+N binding
  (`main.rs:7961`) reach this same path. Existing busy/task/modal/dirty-project
  guards remain intact.
- Independently executed
  `phase1_runtime_tests::settled_projects_control_and_shortcut_start_once_without_refresh`
  with `--exact`. It repeats the original failing sequence: fully settle empty
  Projects, activate mounted/enabled NewProject, then perform five ordinary
  draw/poll passes. It also simulates Ctrl+N; both routes run at both sizes.
  No fixture project, initiating test-side refresh or injected runtime result is
  used. Request 1 becomes runtime request 2; Checking has a real pending ID, and
  the real worker supplies authoritative UnavailableDistribution. Redraw and
  re-entry submit no second read. Name focus/drafts survive and an explicit local
  save succeeds, continuing to Access without a task or captured evidence.
- Independently executed
  `phase1_runtime_tests::fresh_entry_runtime_reply_does_not_own_a_concurrent_project_save`
  with `--exact`. A real earlier runtime reply leaves the concurrent save's
  pending ID, busy state, submitted project, draft signature and focus intact.
  The later real save persists the submitted identity/name while retaining newer
  draft text and Website location. The retained stale-ID runtime test also passes.
- Rechecked `runtime_ux.rs:85–129`, `main.rs:564–567`, `jobs.rs:784` and
  `product.rs:142–227`: this is only off-thread, read-only RuntimeStatus, handled
  before ordinary save replies. It does not install, log in, discover or visit a
  website, hydrate project settings or acquire ordinary mutation/task ownership.
  The existing once-only app availability cache is preserved.

### V1-2 closure — ordinary UI-only and qualified tests

`main.rs:8072–8075` now gates both simulation-dependent modules on `test +
ui-tests`, consistently with neighboring modules and Cargo's existing test-support
dependencies. No feature dependency workaround, weakened assertion, production
test-only behavior or activation-guard removal was introduced.

The exact formerly failing command now exits **0**:

```sh
cargo test --locked --offline -p regressee-app --features ui --bin regressee --no-run
```

The qualified `--features ui-tests` Phase 1 run executes **all seven tests**
(four setup, three runtime), without requiring all features.

### Fresh pixels, binding and historical evidence

Personally inspected `R/phase1-fresh-entry-{1280x800,820x600}-{checking,authoritative-unavailable}.png`
and `R/renderer-proof.png`. All four runtime labels/help and stable Save footers
are legible and contained. Current/Upcoming step labels remain readable and
connected. At 820×600, the form is genuinely scrolled to the runtime note; earlier
Website content partly above the scroll clip is expected, not a new overlap or
first-viewport-fit claim. No new confirmed visual clipping/overlap blocker.

The fresh recipe (`ui_phase1_capture.rs:336–467`) settles empty Projects and uses
actual mounted entry and worker replies. It neither preloads a saved project nor
manually refreshes the entry or injects readiness. Recorded note bounds fit the
real form scroll owner in both states:

| Size | Runtime note `x,y,w,h` | Actual scroll clip `x,y,w,h` |
|---|---|---|
| 1280×800 | `216,575,680,60` | `192,224,1088,511` |
| 820×600 | `216,460,580,60` | `192,224,628,311` |

Independent integrity command:

```sh
python3 /var/home/olav/Data/opencode/tmp/opencode/regressee-phase1-reverification-20261013-2s806wbd/verify-repair-integrity.py
```

**PASS:** 18 current manifest source hashes; four fresh PNG hashes/dimensions;
all 13 current source bindings in each fresh sidecar and proof; proof/receipt;
recorded state and note/Save containment; private 0700 roots; unchanged HEAD.
After independent tests/build, application and all-feature test binary hashes
also match the repair capture provenance.

Compared **63 retained snapshot files**: only `main.rs`,
`phase1_runtime_tests.rs` and `ui_phase1_capture.rs` differ. Verified that the
complete `main.rs` repair is exactly the entry-read call and two test gates;
the other changes replace the test-side entry refresh/add regressions and append
the bounded capture recipe. Cargo files, setup/core/layout/control/style/copy
sources and the old 60-state recipe are unchanged.

The previously inspected 60 PNGs still match the prior independent hash inventory
and their **original**, pre-repair source bindings. They remain **historical
evidence**, not newly rendered or repaired-source whole-file-bound screenshots.
In particular, their old `main.rs` and capture-harness hashes do **not** match the
repaired files. Continued acceptance of their previously inspected visual content
rests on the verified no-paint/style/copy repair delta, plus the four genuinely
fresh repaired-source entry states—not a claim that historical hashes now match.

Receipts: `RV/independent-repair-integrity.json`,
`RV/repair-inspection-ledger.json`, `RV/*-repair.diff` and
`RV/independent-results-summary.json`. Current source-provenance SHA-256:
`e1636ac4429a7fb61e63062b1b0f927fa2d7e23fb7a165caa891fc1120cdb272`.
The preserved original 299-line FIX report has SHA-256
`8c6d18d52e10af9a88b04b056db759e1a78b6a739a2b87b386f0fbfbcf2b646b`.

### Independently repeated tests and complete Phase 1 fallout check

All commands ran from the repository root with the same `LIBRARY_PATH` and
approved `TMPDIR` environment documented above. Exact command arrays, exit codes
and log names are retained in `RV/commands.json`; every command exited **0**.
Binary tests below use `--all-features --bin regressee -- --test-threads=1`
unless separately specified.

| Current-source command/filter | Independent result |
|---|---|
| Exact UI-only `--no-run` command above | Compiles successfully. |
| `--features ui-tests --bin regressee phase1_ -- --test-threads=1` | 7 passed. |
| Exact fresh-entry control/shortcut test, `--exact --test-threads=1 --nocapture` | 1 passed; both routes, both sizes. |
| Exact fresh-entry concurrent-save test, same exact-test flags | 1 passed. |
| `phase1_runtime_tests` / `phase1_setup_tests` | 3 / 4 passed. |
| `phase5_setup_tests` / `phase5_repair_tests` | 4 / 5 passed. |
| `options_ux_tests` / validation-correction focus regression | 4 / 1 passed. |
| Full app binary suite | 130 passed, 0 failed, 18 ignored. |
| App library `product::` / `jobs::`, all features | 12 / 13 passed. |
| Core `shell::` | 11 passed. |
| `cargo build --locked --offline --workspace --features ui --bins -j 4` | Exit 0. |
| `cargo fmt --all -- --check` / `git diff --check` | Exit 0. |

The preserved visual/source review plus unchanged relevant source and current
regressions satisfy **all six Phase 1 implementation requirements**: connected
current/completed/upcoming progress and safe Back/future/save gates; distinct-origin
consent/reset including newer-save drafts; public/private Access hierarchy,
required login prerequisites and independent optional disclosure; retained Pages
summary/custom profiles/limitations and complete Project settings; removed rail
slogan and factual About Regressee/privacy copy; shared capability-correct runtime
language with local saving available. Two-size mounted Tab/reveal/activation,
hidden-control refusal, disclosure focus restoration, clean Continue, exact-save
continuation and newer-draft/request ownership regressions still pass.

Ignored capture/qualification tests are not passes. The existing dependency
future-incompatibility warning remains. No new full-workspace test or independent
Clippy run is claimed here; neither substitutes for this scoped acceptance.

**Qualification boundary is unchanged:** Linux GPUI/Blade offscreen paint and
simulated controller/input with isolated synthetic projects only—not actual OS
keyboard/pointer/IME, assistive technology/accessibility bridge, high-DPI/contrast,
Windows/macOS, live login/network, browser distribution or release qualification.
Those remain existing release obligations, not unresolved Phase 1 FIX items.
