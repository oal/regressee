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
