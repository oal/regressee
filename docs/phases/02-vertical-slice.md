# Phase 2 — first real baseline and comparison

## Outcome

The earliest hands-on preview: a non-developer can capture a public page, save a
baseline, change the fixture/site, and inspect a real screenshot difference.

Prerequisite: Phase 1 local repairs independently verified; external/native/human
acceptance remains open under decisions 0005/0006. Read `PLAN.md`, Sections 3,
5–6, and 8–10, and `docs/phases/01-reverification.md`.
This preview is deliberately not the full first release.

## Implementation update — 2026-10-09

Local implementation delivered; **independent verification pending**, not full
acceptance. Checked items below describe implemented paths and local tests, not
clean-machine/public-runtime/native passes. The actual exercised runtime is
development/owned-loopback-only; public URL settings are supported but public-site
capture is refused until a current trusted runtime is qualified. See
`02-status.md` for exact tests, defensive follow-up scope and open gates.

## Work

- [x] Implement the real private-runtime download/verification/install/repair
  path, reusing Phase 0's qualified packaging decisions.
- [x] Implement typed worker handshake, capture, progress, cancellation,
  timeouts, diagnostics, and shutdown with safe process-tree supervision.
- [x] Create a real project with one public URL and one desktop profile (public
  capture qualification pending; development pack refuses non-loopback targets).
- [x] Implement basic bounded readiness/font/image/animation handling and a
  full-page capture, with transparent capture warnings.
- [x] Store original lossless screenshots, metadata, hashes, and thumbnail/
  viewer-tile derivatives.
- [x] Create/activate an immutable baseline only after successful capture and
  explicit confirmation.
- [x] Revisit the identical target and create a separate comparison run.
- [x] Implement initial changed-pixel metrics and heatmap generation using
  bounded background work. Version the initial analysis policy.
- [x] Build a synchronized before/after and diff viewer with zoom/pan (native
  rendering/input approval pending; bounded background tile decoding tested).
- [x] Distinguish unchanged/changed from timeout/access/capture failure.
- [x] Preserve completed evidence after cancellation or restart; never activate
  a partial baseline accidentally.
- [x] Add integration fixtures for unchanged content, changed content, a tall
  page, navigation failure, corrupt runtime download, and worker crash.

## Deliverables

- Real single-page project → baseline → comparison → viewer loop.
- Runtime setup/repair UI and persisted evidence.
- Cross-platform automated/smoke tests for the vertical slice.
- Known limitations clearly labeled in the preview.

## Completion gate

**OPEN:** local signed HTTP/product worker/headless evidence passes do not meet
the clean supported-machine/public-runtime/native/human gate below. Windows/macOS
process-tree adapters and external smoke tests are pending. Independent verifier
must check latest source and carry these gates honestly; no Phase 3 acceptance
is implied by local delivery.

On a clean supported machine, a tester installs/provisions from the app,
captures a baseline, sees a deliberate fixture change, and can reopen the
evidence after restart without a terminal. An unchanged fixture yields the
expected initial policy result, while a failed capture is never shown as
unchanged. A tall screenshot remains viewable without exceeding qualified
image/GPU limits. Cancel leaves no orphan browser processes.

## Next

Phase 3 expands the working loop to guided discovery, multiple projects,
multiple profiles, and resumable multi-page jobs.
