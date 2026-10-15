# Readable, copyable capture diagnostics — 2026-10-15

## Scope

The user reported that Capture details exposed unformatted technical output that
could not be copied. This is a narrow extension of Clear wayfinding, not a viewer
redesign or a change to capture, comparison, approval or support-export behavior.

## Implementation

- `crates/app/src/capture_diagnostics.rs` creates one report for presentation and
  clipboard output. Labelled, ruled groups separate capture context, baseline
  provenance, the captured run and comparison inputs. Capture metadata and exact
  pair/policy/mask descriptors use indented JSON in platform monospace type.
- Dates use the existing UTC formatter and retain exact Unix seconds. Missing
  timestamps say **Not recorded** rather than exposing Rust `Option` syntax.
  Unloaded source records remain explicit; current project settings never stand
  in for historical metadata. Failed captures retain their recorded target outcome.
- Expanded diagnostics pins **Copy diagnostics**, inline **Copied.** feedback and
  a full-URL/settings sharing caution below the existing reading scroller. Copy
  includes every report section, not just visible text. It is a plain-text report
  with JSON records, not a single JSON document and not the privacy-filtered support
  export. Text selection was not added to ordinary native labels.
- Shared native action/focus states and mounted-control guards cover pointer and
  keyboard dispatch, including read-only History inspection. The disclosure gains
  a measured scroll anchor; expansion aligns it at the top of the reading clip so
  the new heading is exposed. Close/Escape retains the incumbent return focus.
- Copy writes only to the system clipboard. It queues no worker request, writes
  no file, changes no screenshot or baseline, and does not save review drafts.
  Existing palette, spacing, help typography and controls are reused. There are no
  new shipping raster assets or decorative effects.

## Bounded verification and finish review

- Production-feature `cargo build --locked -p regressee-app --features ui --bin
  regressee` passed using the repository's local link-library environment.
- The two new GPUI tests **passed**. They cover exact metadata/descriptor
  round-trips, IDs, UTC/raw dates, absent historical records, failed/missing
  selection states, Unicode and
  reports longer than the review-note limit. They exercise pointer/keyboard copy,
  inner-scroll reveal, pinned geometry through the tail and confirmation, explicit
  dispatch guards, Escape restoration, no queued work and unchanged stored evidence.
- One 12-frame native GPUI/Blade inspection batch covered capture/comparison
  starts, JSON records and copied tails at **1280×800** and **820×600**. It found
  that expansion initially left diagnostics below the reading clip. That defect
  was corrected in one batch and protected with a geometry assertion.
- One 12-frame confirmation batch passed. The four starts, narrow comparison tail
  and wide long-URL JSON were personally inspected again. The new heading appears
  immediately; the smallest feedback-bearing layout requires scrolling to read
  its fields. Copy, confirmation and the sharing caution remain available. All 12
  PNG hashes and current report/viewer/review/shell source bindings were verified.
- `git diff --check` passed. Concurrent changes outside this task were preserved;
  early compilation attempts encountered their temporarily incomplete helpers.
  An initial broad test run hit its 120-second command limit after the new tests
  had passed; it is not a completed whole-suite result.
- The unrestricted broad rerun finished: app library **117 passed, 2 ignored**;
  native shell **187 passed, 1 failed, 1 ignored**. Both diagnostics tests passed.
  The failure is the concurrently added
  `screenshot_placeholder_tests::screenshot_placeholder_fills_the_stage_and_keeps_recovery_reachable`:
  its recovery element was outside the expected screenshot-stage bounds. That
  placeholder work was not changed here. Cargo stopped before the core library
  suite, so this is not a clean whole-suite result.
- Strict production Clippy is blocked by an unrelated existing `manual_clamp`
  finding in the Project settings form-width calculation in `main.rs`. That
  calculation was not changed here. The existing upstream future-Rust warning for
  `proc-macro-error2` also remains.

**Bounded finish verdict: PASS for the implemented diagnostics interaction and
sampled Linux offscreen layout.** These are synthetic local fixtures, simulated
controller/clipboard checks and offscreen native pixels—not OS clipboard/input,
assistive-technology, Windows/macOS or public-release qualification. No real-site
capture, login session or user database was used.

Evidence, original inspection frames, confirmation frames and per-frame source
provenance are retained under:

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-capture-diagnostics-20261015-e7JA05Vn/
```

Reproduce the scoped tests with `cargo test --locked -p regressee-app --features
ui-tests --bin regressee capture_diagnostics -- --test-threads=1`. The opt-in
`capture_diagnostics::renderer_capture_diagnostics` uses the existing explicit
private-root renderer environment and needs no external website fixtures.
Restart the rebuilt `target/debug/regressee` to use the change.
