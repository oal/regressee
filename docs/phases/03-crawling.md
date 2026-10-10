# Phase 3 — guided crawling, projects, and capture jobs

## Outcome

A whole-site public preview with understandable discovery boundaries, a fixed
baseline inventory, multiple projects/profiles, and robust progress/recovery.

Prerequisite: Phase 2's dependent local supervisor gate was independently
re-verified in `02-reverification.md`; full native/public/platform acceptance is
still open. Read `PLAN.md`, Sections 3–7 and 13.

Implementation delivered locally on 2026-10-09. Independent verification
(`03-verification.md`) reproduced two P1 safety defects and a P2 preview defect;
bounded repairs are recorded in `03-corrections.md`, with **independent correction
reverification pending**. Checked work items mean implementation, not acceptance.
See `03-status.md` and `03-corrections.md` for tests, limits and verifier commands.

## Work

- [x] Implement base-URL and pasted-URL-list entry, validation, and list-only
  versus list-and-links discovery modes.
- [x] Implement canonical identity, query/tracking rules, hash routes,
  requested/final URLs, origin approvals, and deduplication with fixture tests.
- [x] Extract rendered links without arbitrary clicking/form submission.
- [x] Add page cap, website/section boundary, text exclusions, and advanced
  depth/query controls. Show why addresses were excluded.
- [x] Add robots behavior and explicit authorized overrides.
- [x] Implement discovery preview with URL search, include/exclude selection,
  coverage-limit warnings, page count, and screenshot/storage/time estimates.
- [x] Freeze an inventory revision when beginning baseline capture.
- [x] Add desktop/tablet/mobile selection and exact immutable profile metadata;
  offer full-page or viewport-only extent and advanced custom dimensions.
- [x] Implement bounded Gentle/Balanced/Fast scheduling, per-origin throttling,
  advanced parallelism, rate-limit backoff, and bounded retries.
- [x] Persist target queues, show discovery and capture as distinct phases,
  and report pages separately from screenshot targets.
- [x] Implement pause, cancel, interrupted-job recovery, and validated resume.
- [x] Expand comparison to the fixed baseline inventory only. Capture 404,
  redirect, access, and timeout outcomes without changing that inventory.
- [x] Add changed-first virtualized results, filters, URL search, profile
  grouping, and target-coverage accounting.
- [x] Support multiple saved projects and explicit new-baseline discovery.
- [x] Add basic available-space preflight and stop safely before disk failure.

## Deliverables

- Public whole-site workflow with understandable setup/progress/results.
- Inventory/scheduler/recovery tests and explicit coverage summaries.
- Scale measurements for 100 pages × 3 profiles; a first 1,000-page soak run.

Local actual measurement: **100 pages / 300 targets / 300 browser visits**,
105.09 s capture interval. **1,000-page / 3,000-target soak NOT RUN** (bounded
verification budget), so the scale deliverable is not fully met. Public runtime
and native/human/platform qualification remain pending.

## Completion gate

A user can bound discovery to a section, omit addresses, review the exact page
list, capture several screen sizes, pause/resume, and compare only the saved
targets. A deliberately added website page is not visited during a comparison.
Page limits, robots exclusions, failures, and cancellation are visible in
coverage. Restart does not lose valid evidence or silently duplicate targets.
Results remain responsive while capture and comparison work is running.

**Gate not fully passed:** implementer actual browser/service checks and simulated
GPUI focus/action checks passed; independent correction reverification, native responsiveness,
public workflow qualification and the 1,000-page soak remain open.

## Next

Phase 4 qualifies the same workflow for saved interactive sessions, internal
sites, alternate environments, and more reliable capture stabilization.
