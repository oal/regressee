# Task panels: clearer activity, outcomes and useful Details

## Authorized scope

Following the task-view critique and “go ahead”, apply the shared treatment to
exclusive task workspaces and dialogs. Preserve Stop/Pause capabilities,
authoritative completion and explicit baseline updates. Do not add a chronological
activity log. Existing unrelated working-tree changes are retained; no commit or
push is part of this work.

Review input:
`.impeccable/critique/2026-10-12T16-48-23Z__crates-app-src-task-ux-rs.md`.

## Implementation

- `crates/app/src/task_ux.rs` separates compact project/site/environment context,
  activity/progress metadata, outcome counts, one operation/state-aware safety
  note, and independently useful Details. Its footer remains outside the scrolling
  body. Opening Details reveals its content; closing it restores the summary.
- `crates/app/src/ui/status_icon.rs` provides native-drawn, consistently weighted
  status marks, always accompanied by words. No emoji or font-glyph substitutes.
- `crates/app/src/ui/controls.rs` owns the shared information note and labelled
  counts. Full count items wrap together. Explanatory copy has a 560px maximum
  measure. Text-containing flex rows give copy its own bounded, stretching column.
- Attention uses the existing amber token, failed targets alone use red, saved
  screenshots use ordinary ink, and zero counts are muted. A checkmark labels a
  saved screenshot, not baseline approval.
- `crates/app/src/task.rs` holds small frozen capture-status snapshots and bounded
  terminal gap details. Gaps include the requested page, screen size, recorded
  reason, attempts, distinct final URL and warnings, with reason-specific recovery.
  Failures lead; at most 20 gap rows are shown, with an explicit full-review route
  when truncated. Two failed screen sizes need not mean two different pages.
- `crates/app/src/crawling.rs` / `crates/app/src/jobs.rs` provide actual queue counts
  separately from phase prose. Existing public product progress callbacks remain
  unchanged. Resumed preparation starts from actual processed-target counts.
- `crates/app/src/main.rs` admits capture snapshots only for the pending request,
  capture/check/preview kind, matching project and same run. Terminal task outcomes
  consume their own worker reply, not previously displayed `self.evidence.latest`.
  A mismatched terminal run fails closed without displaying another run's counts.
  The displayed environment follows the frozen run rather than edited settings.
- Completion clears the task's progress tuple. Details no longer falls back to
  “Capturing…” or repeats the base URL. It is absent when there is no additional
  information. Redundant ordinary storage-failure diagnostics are not repeated.
- No storage/baseline policy, cancellation affordance, page-discovery scope or
  private-session handling is changed. No invented ETA or active-page telemetry.

`DESIGN.md` and `README.md` document the shared presentation and Details contract.

## Verification and provenance

Final evidence directory:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-task-final-lnWcT8/`.

All builds use the existing `.runtime/link-libs` linker workaround and approved
`TMPDIR`; no renderer fallback or GPU/compositor policy changes are made.

Final results: **333 workspace tests passed, 42 intentionally ignored, one
mapping-preview test excluded; the 40-frame task renderer matrix passed.** All
40 PNG checksums and task/policy/controls/icons/shell source bindings were verified
against the final source. Fourteen representative frames were personally reviewed,
including both attention/gap sizes and narrow startup, progress, setup, import,
failure, baseline-finishing, paused and complete states. `verification.json`
records exact hashes, frame names and the exclusion. `git diff --check` is clean.
Both P1 issues from the archived critique are cleared; that exact snapshot is
closed while its historical review is retained.

Commands:

```sh
cargo test --locked --workspace --all-features -- --test-threads=1 \
  --skip options_ux_tests::mapping_preview_has_bounded_rows_and_a_pinned_close_without_mutations
cargo test --locked -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::renderer_capture_tasks -- --ignored --exact --test-threads=1
```

The renderer command requires the existing private output-root and reviewed
calibration environment variables used in its test module, plus the linker setup
above. OS input/accessibility limits below apply even when these tests pass.

- The new task-model tests cover target-vs-success counts, multiple failed sizes
  on one page, frozen profiles, and failure-first truncation of large gap lists.
- Simulated GPUI tests cover absent/relevant Details, stale request/project/run
  updates, recorded failure attempts, terminal progress clearing, stop/focus
  behavior and unchanged committed-action cancellation policy.
- The opt-in native offscreen matrix uses synthetic, unsaved task states, not
  website work. It covers startup, retry, progress, pause/stop, partial/error/success
  results, capture setup and gap disclosures, ZIP dialogs and baseline finishing
  at 1280×800 and 820×600. Geometry checks include footer reachability, visible
  progress tracks, bounded copy and disclosure reveal/close behavior. The stored
  fixture evidence must be byte-equivalent before and after the matrix.
- PNG sidecars contain exact source hashes for task presentation, task policy,
  controls, icons and the shell. Calibration/readback was personally reviewed;
  its identical PNG and reviewed hash are copied into the final directory.
- A manual Impeccable detector scan of the four Rust task/control targets returned
  exit 0 and `[]`. Its regex coverage is **not native GPUI qualification**.

The first native batch exposed collapsed text columns and a disclosure reveal
problem; these were corrected. The confirmation guard initially over-constrained
short intrinsic title widths and caught unwrapped note text. Bounded stretching
copy columns and containment assertions address those functional layout defects.
An attempted reuse of the first evidence directory failed closed on `create_new`
(`File exists`); existing captures were not overwritten.

### Separate suite failure

The unfiltered workspace suite fails in
`options_ux_tests::mapping_preview_has_bounded_rows_and_a_pinned_close_without_mutations`.
It reproduces in isolation: the scope bounds are at `y=-1042` while the mapping
viewport is at `y=814`. The task pass does not edit the mapping panel or its tests.
This is recorded, not called a proven pre-existing defect or silently fixed.

### Limits

Offscreen captures use GPUI's native Blade renderer and Linux fonts; they establish
pixels and simulated geometry, **not OS presentation/input or screen-reader
accessibility**. No live website capture, official browser download, performance
qualification or new end-to-end storage authority is claimed. Historical-resume
selection remains the worker's existing evidence contract; a mismatched returned
run is refused rather than guessed.

## Follow-up: displayed web addresses are links

The task context website and recorded requested/redirected page addresses now use
`ui/web_link.rs`: accent-blue underlined text at rest, pointer cursor, unchanged
text styling through hover/focus, an ink focus outline and a full-destination
default-browser hint. Both clicks and keyboard activation use the shell's guarded
`TaskLink` controls. Tab reveals clipped links in the task's own scrolling body;
hidden gap links and links behind a native prompt cannot activate. Stop/Pause
remains the initial task focus rather than a newly introduced URL.

Only complete HTTP(S) URLs without embedded credentials are handed to GPUI's OS
URL opener, on explicit activation. Query ordering and fragments are retained.
The capture's private browser/session is not reused. URL input fields and raw
diagnostics remain literal data; unsafe schemes do not get a link affordance.
`DESIGN.md` now records this as the shared navigation-link rule. Other app surfaces
were not bulk-rewritten as a side effect of this task-view correction.

Final link-pass evidence directory:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-task-links-confirm-GnAPZZ/`.
The earlier 40-frame source bindings above identify the pre-link revision, not
this follow-up. New sidecars also hash the link component. Simulated pointer and
keyboard tests observe GPUI's **test-platform** URL opener, not a real browser;
they verify exact requested/final destinations and unchanged stored evidence.
The dedicated offscreen link matrix covers ordinary/focused context, gap links
and long wrapping/focused URLs at both desktop sizes. OS URL-handler availability
and actual browser opening remain platform/manual checks.

Final follow-up results: **335 workspace tests passed, 44 intentionally ignored,
the same mapping-preview test excluded; all eight link renderer frames passed.**
All eight frames were personally reviewed and their PNG checksums and six source
bindings verified against the current source. `verification.json` records the
hashes and evidence limits. Formatting and `git diff --check` pass. The initial
link render revealed an undersized context hit/focus region; full-width bounds
and a stable transparent border fix it. Tests now click inside the URL text and
assert the minimum hit width, rather than passing on an incidental left edge.
