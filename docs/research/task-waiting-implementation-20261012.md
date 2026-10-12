# Long-running task UI — 2026-10-12

Implemented the user-selected focused task screens across long-running flows,
with progress dialogs for bounded operations. This preserves Clear wayfinding;
it does not force OS fullscreen or introduce background-job concurrency.

## Interaction policy

| Operation | Waiting surface | Available actions |
| --- | --- | --- |
| Find pages | Focused in-window task workspace | Stop finding pages, Details |
| Capture / check changes | Focused task workspace | Stop capture/check, Pause when supported, Details |
| Explicit browser setup | Focused task workspace | Cancel setup, Details |
| Open login / confirm session / check access | Protected progress dialog | Operation-specific Cancel, Details |
| Representative screenshots | Protected progress dialog | Stop preview, Details |
| ZIP inspection / import / export | Protected progress dialog | Operation-specific Cancel, Details |
| Diagnostic preview / export | Protected progress dialog | Cancel preview/export, Details |
| Exclusive storage inspection / previews | Protected progress dialog | Details; no unsupported cancellation |
| Confirmed saves / baseline updates / deletion / cleanup | Protected progress dialog | Details; must finish safely |
| Ordinary project, evidence, history, runtime and environment reads | Existing lightweight loading feedback | Local cached filtering/selection remain available where supported |
| Viewer tiles, debounced analysis and background storage-size reads | Existing independent local feedback | Existing identity/staleness guards remain |

Exclusive tasks own their enabled/mounted controls and Tab order, regardless of
the underlying section. Focused workspaces replace the normal navigation and
working controls; dialogs occlude and guard them. Keyboard navigation, New,
Save, Find and Help cannot bypass the task. The scrollable body and fixed action
footer keep Stop/Continue reachable in the inspected desktop sizes.

Each task stores its request ID, operation identity, project/site/environment,
origin/return focus, state and progress. ZIP import identifies the incoming
project rather than attributing the previously selected website's context.
Elapsed time is informational, not an ETA. Percentages require a known nonzero
total; discovery uses visited-page counts, not invented coverage.

## Stop, pause and outcomes

- A Stop request enters **Stopping…** until the matching worker terminal reply.
  It does not dismiss the surface, clear busy state or claim that work rolled back.
- A Pause request enters **Pausing capture…** until the backend acknowledges the
  paused run. Stop remains available while pausing. Only inventory capture has
  backend pause support; other tasks do not borrow that capability.
- Discovery retains a limited/incomplete page preview where available, including
  cancellation during the last active page. The outcome explicitly asks the user
  to review found pages before capturing.
- Capture outcomes distinguish paused, cancelled, interrupted and completed-with-
  errors runs and show saved, failed, skipped and uncaptured screenshot counts.
  Paused/interrupted runs retain the existing resume path; cancelled runs do not
  falsely advertise resume. The saved baseline stays unchanged.
- Worker cancellation checkpoints produce `Reply::Cancelled`. Merely setting a
  cancellation token or mentioning cancellation in another error cannot override
  a successful export/import/install or a genuine failure.
- Successfully published ZIPs, saved imports and committed browser installations
  remain successful even if cancellation arrives late. Non-cancellable confirmed
  saves, baseline updates and deletion do not offer a false Cancel/rollback.
- Long tasks leave an explicit outcome and Continue/review action. Short saves
  and previews return directly to their existing result/confirmation UI on success.
  Failures and acknowledged cancellation remain readable outcomes.
- Details retain task-specific progress/error text even when a follow-up evidence
  read starts. Completion restores the original mounted action, or section
  navigation if that trigger disappeared. Escape requests Stop when supported;
  it returns from an acknowledged outcome, not from a still-running commit.

## Sources

- `crates/app/src/task.rs`: exhaustive Work-to-surface/capability policy and task
  lifecycle; independent of screenshot-oriented busy flags.
- `crates/app/src/task_ux.rs`: task identity, scoped progress/outcomes, task
  workspace/dialog, focus and stop handling, simulated GPUI regressions.
- `crates/app/src/main.rs`: queue/reply integration, control/shortcut guards and
  progress routing. `exit_guard.rs` retires task state for its own save pipeline.
- `crates/app/src/jobs.rs`: distinct acknowledged cancellation replies without
  masking committed success; representative-preview progress forwarding.
- `crates/app/src/crawling.rs`: representative progress and last-page discovery
  cancellation note. Capture/storage publication boundaries remain unchanged.
- `crates/core/src/shell.rs`: task-control identities and local cached History
  filtering/selection during ordinary reads.

The pre-existing `DESIGN.md`, `README.md` and shared progress-control changes
were preserved. No commit or push was made.

## Verification and limits

**PASS — source/test-platform behavior:**

- `cargo test --locked --workspace --all-features`: **323 passed**, zero failures;
  opt-in browser/native and other ignored suites are not implicitly accepted.
- Five opt-in `automatic_runtime_setup_independent` public-API tests: **5 passed**.
  Private inert runtime inventories, synthetic PNG evidence, local listeners and
  real Jobs queues establish queued cancellation, installer/download cancellation
  and retained evidence/pointers; no browser worker or website is launched.
- `cargo clippy --locked --workspace --all-targets --all-features -- -D warnings`.
- `cargo fmt --all -- --check`, `git diff --check` and the normal UI build.

**PASS — bounded actual Linux GPUI offscreen rendering:** calibration personally
inspected, then 12 task states at both **1280×800** and **820×600** (**24 images**),
plus the retained viewer/loading batch. The task matrix covers discovery,
stopping, setup download, capture progress, pausing, ZIP import/export, a protected
baseline commit, a long error, partial discovery, paused capture and late export
success. Scrollable body/action-footer bounds and task-control guards passed;
stored fixture evidence remained unchanged.

The initial task batch stopped after its first image on a stale-selector assertion:
GPUI diagnostic bounds retained a prior navigation selector. The correction uses
the actual mounted/enabled-control contract instead of inferring absence from
that retained diagnostic map. One bounded confirmation then passed all 24 states.
No open-ended renderer/polish loop was used. The images precede the final
identity/focus-only fixes; those corrections passed the final behavioral suite.

Private evidence directory on the development host: `regressee-tasks-pEf68r`,
with `final-tests.log`, `final-clippy.log`, `independent-cancellation.log`,
`proof.log`, `read-viewer-renderer.log`, `build.log` and
`task-confirmation/renderer.log` plus images. Linux tests/builds use the existing
`.runtime/link-libs` development linker workaround. Existing vendored-GPUI and
dependency future-compatibility warnings remain; no unrelated vendor edits.

**NOT RUN / not claimed:** live long-running website tasks, real OS keyboard or
pointer input, screen-reader/native accessibility, IME/high-contrast/OS scaling,
ordinary hardware/default-Wayland rendering, Windows/macOS, human usability
acceptance or release qualification. Synthetic task states are layout evidence,
not actual website results. Earlier native-presentation limitations are not
superseded by these offscreen captures.

## Follow-up: discovery startup feedback

The user observed a long wait at “Finding website pages” before the counter
started. Discovery now immediately says **Preparing to find pages…**, then
reports real preparation boundaries: reading saved settings, checking installed
browser files, checking website/login settings, starting the private browser,
preparing a saved session when present, fetching website crawl rules, and loading
the first page. Retry backoff and browser restarts are explicit as well.

The startup workspace explains that pages are counted after page checks finish;
browser preparation and waiting for the first response do not increase that
count. This helper is absent for supplied-address-only discovery and after
counted visits start. Counts and totals remain real; no timer-generated progress,
percentage or ETA was introduced. Startup phase delivery survives update-queue
backpressure but can be released immediately by Stop/Quit. Supplied-address-only
and zero-visit excluded candidates do not flood that reliable startup channel.

Verification in private `regressee-discovery-startup-xIV4Dl`:

- The complete workspace suite passed **326 tests** before the last zero-visit
  suppression and guarded integration-test update.
- Final library-only workspace suite: **216 passed**, zero failures, including
  preparation-before-runtime-failure, accurate loading/retry counts and startup
  backpressure/cancellation regressions. Library Clippy and targeted formatting
  checks passed; `git diff --check` passed.
- The expanded offscreen task matrix passed **32 images** (16 states at each
  desktop size), with eight new browser-check/crawl-rule/first-page/retry startup
  states. Startup helper and Stop action bounds passed. Startup pixels were
  personally inspected at both sizes. These precede unrelated concurrent edits.
- The UI binary built successfully before those concurrent edits.
- Final whole-app formatting/Clippy/build verification is **blocked by concurrent
  options-UI work**: a new missing `options_ux.rs` module, unresolved option-preview
  methods, a `ScrollAnchor` constructor and new control variants not yet handled
  by the shell. Those unrelated changes were left untouched.
- The optional runtime-safety suite initially rejected the newly legitimate
  preparation messages. Its guard now accepts only the two exact settings/file-
  verification messages for Discover, with zero visits and no total, while still
  forbidding setup/download progress. That final integration-test correction
  could not be reverified through the whole-app build while the concurrent work
  was incomplete; no new pass is claimed for it.

No live website/browser first-page run or native-input acceptance is claimed.
