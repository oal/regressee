# Phase 5 — V5-1/V5-2/V5-3 repaired; independent re-verification pending

## Finite functional follow-up — 2026-10-12

The independent [FIX](phase5-verification.md) and the original
[blocked report](phase5-report.blocked.md)/[receipt](phase5-receipt.blocked.json)
remain unchanged. This section supersedes the **current status**, not the historical
findings below. No commit/push, Phase 6, new agent, broad render or aesthetic cycle.

**V5-1 substantiated repair:** one routing helper preserves guided Website/Access/
Pages versus full Settings for Save and correction-summary activation. Advanced
discovery fields disclose Pages; login corrections disclose Access even for public
draft access. Painted whole-field bounds include label, actual input border,
attached correction and help. Immediate and next-frame reveal use the current
scroll owner, not cached inner-input coordinates from another stage. Both-size
cross-stage tests cover Address, fields 10–12 and 0/3–6, Settings preservation,
mounted stages, affected focus, retained draft, corrected next-action availability,
and **zero request on invalid Save**. No new field silently saves or visits.

**V5-2 substantiated repair:** status observation is independent of permission to
hydrate a form. An immutable project-bound form origin is captured when settings
load; initial hydration requires that origin **and** the request snapshot still
match the current draft/environment. Taking a new snapshot after newer edits is
no longer permission. Status becomes available without rehydrating dirty fields.
Project/environment-keyed saved login values track URL, representative URL,
approved providers and retention even while status is absent; successful Save
records the submitted values, not newer ones. Save replies reject another selected
project's identity rather than adopting its record. Login/status replies are also
project-bound. Continuation signatures include stage/screen and IME composition;
clean explicit discovery now rejects composition too.

The exact independent failing worker probe was rerun from a **new isolated probe
workspace**: **2 passed, 0 failed**, using its byte-identical original test file.
Repository tests additionally exercise fields 11/12 and retention, edits during
the follow-up read, login-only dirtiness without status, and untouched initial
hydration from deliberately cleared fields/retention. Continue and Save & find
pages each cover newer/invalid/composing drafts, stale replies, actual save conflict,
stage/screen switches, another environment and another selected project. No
unauthorized discovery/login, selected-draft overwrite or baseline/evidence change.
The authorized exact save→discovery and ordinary Settings-no-visit controls still
pass. Late composition events are simulated from the pre-save enabled frame; this
does not claim live OS IME qualification.

**V5-3 substantiated repair:** setup actions, choices and whole fields record actual
painted bounds; Tab/focus reveal uses those bounds and the current scroll clip.
Tests at both sizes traverse **mounted/enabled/core candidates via actual Shell
Tab**, reveal and dispatch every standard profile, custom add/replace/remove,
extent/images, discovery mode/bounds/skips, robots/speed/depth/parallelism/retries,
link/tracking/fragments, source consent, access/session choices, mapping approval/
same-site/preview/Close, locale/timezone/theme, environment creation/selection,
capture limitations and explicit login/ready/close/forget/probe paths. Mobile is
wholly revealed after scrolling; its dispatch changes the painted selected-size
plan without a request. Session maintenance uses the unavailable runtime safely;
in-memory open/session prerequisites establish dispatch, not successful real login.

Renderer assertions now name actual form or mapping scroll clips. Initial Mobile
may correctly emit `fully_visible:false` / `requires_scroll:true`; old immutable
window-only claims are historical, not repaired qualification. Pinned heading/
footer controls correctly use window bounds.

### Actual checks / narrow evidence

- Final packages: **308 passed, 0 failed, 40 ignored** (binary 119, library 95,
  integration analysis 7, product flow 1, core 51, storage 35).
  [Log](phase5-repair-tests.txt).
- Separate serial `phase5_` tests (setup, repair and existing Phase 5 safety):
  **12 passed, 0 failed, 0 ignored**.
  [Log](phase5-repair-targeted.txt).
- Exact external independent probes: **2 passed, 0 failed**, not zero filtered
  execution. [Log](phase5-repair-probe.txt). New workspace:
  `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261012/repair-probe`.
  Original probe file SHA-256 remains
  `90b21416b69db173c9e5a4aa3aaf4ead88a0a07c665f0fbef01d902e2e2e3e25`.
- Narrow source-bound native repair: **1 passed, six pairs, 31.42s**. Only Access
  validation, Pages advanced validation and Mobile revealed, at both sizes.
  [Log](phase5-repair-render.txt). Root:
  `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261012/repair-v5`.
  All **six originals individually read together once**. Field border/error/help
  is **680×123 / 580×123px**, wholly inside current clips. Mobile is **680×46 /
  580×46px**, wholly inside the form. No new broad batch or aesthetic correction.
- Capture binds 66 paths. Only `phase5_repair_tests.rs` changed afterward, adding
  stricter cleared-form hydration and painted-plan assertions; shipping source
  and capture harness are unchanged. The capture-time test snapshot and both
  hashes remain in the receipt, not silently rebound to new code.
- Format, whitespace and integrity checks exit 0. Receipt preserves 12 immutable
  archives, 182 originals, blocked 38 pairs, original FIX, six repair pairs and
  66 final source bindings. Initial failed compile/functional attempts are retained.
  Test-only unused-mut and vendor/future-compatibility warnings are not acceptance.

Commands use the same approved TMPDIR and `.runtime/link-libs` as the historical
commands below. Targeted filter: `phase5_ -- --test-threads=1`; packages unchanged.
Narrow renderer uses `REGRESSEE_RENDERER_PHASE5_REPAIR=1` and `.../repair-v5` as
capture root. External probes use `--manifest-path .../repair-probe/Cargo.toml`,
absolute library path, existing `CARGO_TARGET_DIR`, `independent_phase5_probe`
filter and serial execution. `cargo fmt --all -- --check`, `git diff --check` and
`python3 scripts/verify-phase5-receipt.py` complete the handoff.

V5-1/2/3 are repaired in source/controller/current-clip/native-offscreen scope;
**independent acceptance remains pending**. Original OS/AT/platform/runtime/
usability/large-site/release limitations remain. No further correction is requested
in this implementation handoff; next is independent re-verification.

## Historical blocked implementation (superseded current status only)

# Phase 5 implementation handoff — BLOCKED, independent verification required

**2026-10-12 · base `2f09a6b` · S1–S3 implemented in substantial part, not accepted.**
No commit/push, Phase 6, nested agents, downloads or real website visits.
Clear wayfinding, 820×600 minimum, fixed-baseline semantics and advanced operations remain.

## Built behavior

- Guided Website → Access → Pages & sizes is now pinned above the scrolling form,
  with Current / Completed / Next / Unavailable labels. Clean Website/Access uses
  Continue without a redundant write. New/dirty Website saves before advancement.
- Public/private access is visible at the minimum size. Dirty public Access saves
  before continuation; dirty private Access says **Save settings first**. Saved
  private access emphasizes **Open login browser**, then **I'm ready** while open;
  draft/saved/browser prerequisites are beside those footer actions. Open is disabled
  while already open. Login is a separate explicit visit, never promised by Save.
- `setup_ux.rs` binds every settings save to its request and exact project/revision,
  environment, form values, draft settings, retention, timezone and screen/stage.
  Exact success alone can advance or dispatch explicitly authorized discovery.
  Newer draft replies update the saved record but preserve the draft; stale replies
  do not complete pending work. Failure clears continuation and retains edits.
- **Save & find pages** names its visit and waits for authoritative settings success.
  Existing-project Settings still says **Save settings** and does not implicitly
  visit. No combined Save & open login action was introduced.
- Sizes lead Pages. The capture plan now lists selected dimensions on separate
  readable lines, then extent/image policy. Mobile/tablet remain web emulation.
  Full-page/visible-screen, normal/replaced images, bounds/exclusions, robots,
  speed, retries, depth, global/per-site parallelism, custom sizes, action-like
  links, tracking/fragments and capture limitations remain.
- Concrete source/capture mapping pairs precede scope/probe notes; Close stays
  pinned. Included pages × profiles and the actual coverage cause/rediscovery next
  step appear beside Capture pages, without opening Page options.
- Bitmap/photo and Only for this app session wording replaces exposed raster/
  Ephemeral labels; exact unsupported cases and secure-store/no-plaintext warnings
  remain. Actual input paint bounds, rather than wrappers, are measured.
- Form recovery uses painted input bounds for immediate reveal and the new frame's
  anchor for stage changes. Storage/browser work still runs on workers.

## Unmet contract / next focused repair

**Do not mark Phase 5 complete or begin Phase 6.** The finite visual cycle ended.

1. **Access validation still switches to `SetupStage::Settings`** in
   `main.rs::save_settings` for CrawlField(10..12). The affected input is readable,
   but the guided three-stage strip is absent. Confirmation `access-invalid` proves
   this at both sizes. Route guided validation to Access (and Pages for discovery
   inputs), retain the disclosure/input reveal, and add explicit stage-preservation
   assertions. Settings validation must remain Settings.
2. **Visibility assertions are not all clip-owner assertions.** Capture assertions
   for chooser buttons use the whole window, not `screen_scroll.bounds()`. At
   820×600 the Mobile checkbox is below the pinned footer, while its selected
   dimensions and policy are visibly present in the plan. This is ordinary scrolling,
   not proof that every chooser is initially visible. No claim of complete advanced
   pixel qualification is made. Follow-up must assert actual scroll-owner bounds
   when naming a visible control, including field borders and help where relevant.
3. **Unqualified newer-save/initial-status interaction:** the preserved-draft branch
   calls `refresh_visible`; if environment status was absent, later initial hydration
   can copy saved login fields into the current draft. The new tests exercise newer
   project-name drafts with an already loaded environment, not this absent-status
   race. Independently reproduce and fail closed before accepting the user's full
   newer-draft guarantee. Screen/environment identity is included in the continuation
   signature, but screen-switch and identity-switch races need dedicated probes.

The user delegated judgment; these are named implementation gaps, not requests for
another product interview. A verifier should assess the existing evidence and source
first, then the parent can authorize a finite functional follow-up to this same agent.

## Actual tests and commands

Final locked offline packages: **303 passed, 0 failed, 40 ignored**:
app library 95, binary 114, analysis integration 7, product flow 1, core 51,
storage 35. Four new GPUI controller/paint tests cover clean public progression,
dirty authoritative saves, stale/newer name replies, invalid website focus,
private dirty/open/not-ready/ready gates, failed save/task boundaries, ordinary
Settings save without visit, explicit discovery continuation and actual advanced
field widths/reveal at both sizes. Fixtures are synthetic; injected failures and
in-memory private readiness are not real login qualification.

```bash
LIBRARY_PATH=.runtime/link-libs TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
  cargo test --locked --offline -p regressee-core -p regressee-storage \
  -p regressee-app --all-features

LIBRARY_PATH=.runtime/link-libs TMPDIR=/var/home/olav/Data/opencode/tmp/opencode \
REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode \
REGRESSEE_RENDERER_CAPTURE_DIR=/var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261012/confirmation \
REGRESSEE_RENDERER_PHASE5=1 \
  cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::ui_review_matrix::renderer_capture_whole_app \
  -- --ignored --exact --test-threads=1

python3 scripts/ui-review-gallery.py /var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261012/confirmation
cargo fmt --all -- --check
git diff --check
python3 scripts/verify-phase5-receipt.py
```

The final package rerun is `packages-handoff.log`; durable mirror
[phase5-tests.txt](phase5-tests.txt). Renderer confirmation: **1 passed, 38 pairs,
both 1280×800 and 820×600**, log [phase5-render.txt](phase5-render.txt).
Source/hash integrity is separate from acceptance; [receipt](phase5-receipt.json).
Final formatting and whitespace checks exit 0. Receipt checker exits 0: 12
immutable archives, 182 original PNG hashes, 38 confirmation pairs and 65 current
source bindings. Confirmation renderer completed in 60.51s. These checks do not
clear the named Phase 5 blockers.

## Bounded native evidence

New isolated root: `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261012`.
Four website PNG fixtures and previously reviewed calibration were copied from
Phase 4 repair; no browser/download/network fixture regeneration occurred.
Original 182-image evidence and Phase 4 roots were not overwritten.

- Setup attempt failed because the copied proof receipt was missing; no pixels.
- Partial root batch stopped on invalid Access field visibility; pairs/log preserved.
  Input reveal/finite form width were repaired before the first visual inspection.
- `complete/`: 38 pairs, **one first batched inspection of all eight sheets plus six
  originals**: narrow login, mapping, sizes, discovery, invalid Access; wide browser
  open. It exposed hidden narrow private status and clipped capture-plan text.
- One correction batch pinned guided heading/stages, attached access status to footer,
  removed duplicate Access heading, and split the plan into readable lines.
- `confirmation/`: 38 pairs, **one confirmation inspection of all eight sheets plus
  six originals**: narrow login, sizes, browser open, invalid Access, mapping;
  wide saved-private. No later UI correction or third inspection/render round.
- Confirmation's 65-path source binding matches current Rust/lock files. No shipping
  source changed after confirmation. First batch contains the renderer's incumbent
  main/task/control source hashes, not a retrospectively invented 65-path binding.

Actual inner input content is 654px wide at 1280 and 554px at 820 for bounded forms;
Settings' login field can use the larger available width. The tests exercise all
nine Pages numeric/text fields individually after reveal; screenshots do not show
the complete advanced tail simultaneously. Retained mounted/enabled/core candidate
controls use existing IDs; no new focus ID or schema/capture-pixel change.

Known vendor fallback and proc-macro-error2 warnings remain. OS presentation/input,
clipboard/IME, AT, native dialog/resize, text scaling/localization, Windows/macOS,
runtime install/actual authentication, large storage throughput, PM usability and
release qualification are **not tested**. Missing trusted browser distribution is
not repaired by synthetic login-state images. Historical score is not rescored.
