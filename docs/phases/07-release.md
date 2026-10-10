# Phase 7 — usability, cross-platform hardening, and release

## Outcome

The first public release satisfies all confirmed requirements on the declared
Windows, macOS, and Linux matrix and is genuinely usable without developer help.

Prerequisite: Phase 6 gate passed. Read `PRODUCT.md`, all release-relevant
decisions, and `PLAN.md`, especially Sections 8 and 11–14.

## Current local disposition — incomplete

2026-10-10: [original local delivery](07-status.md), [independent original
verification NOT ACCEPTED](07-verification.md), and [final bounded local corrections](07-corrections.md).
Windows atomic Job/private quota/ACL/reparse APIs, contained runtime v3 bundles,
diagnostics/consent, exact tiny-mask outline and checked private source/app helpers
are implemented. Historical private Linux app/source/notices candidate is retained
and **stale for corrected source**, not a release installer/signature/license/ABI
qualification. Bounded Linux kernel-owned guardian/checkpoint-delta corrections
PASS locally:139 Rust/20 ignored,35 signed worker,2 signed ownership and complete
Product flow. The ONE authorized changed-implementation soak **PASS**:3000 actual
captures/visits in900.357 s, unchanged1200 s capture budget;1000 paused originals
preserved and separate1000-capture/2000-cancelled stage settles in0.303 s. Historical
1708/2188-capture failures remain failures. Historical replacement/readiness root
cause **OPEN**; no blanket cancellation suppression. Independent correction recheck
PENDING; original independent verdict is not retroactively accepted.
macOS process ownership and GPUI screen-reader backend remain code gaps; actual
native/OS/keystore/human/public-runtime/legal/clean-package gates remain open.
Checked documentation below does not mean the completion gate is passed.

## Work

- [ ] Run moderated/observed tasks with at least three representative
  non-developers: create project, constrain discovery, capture baseline, inspect
  change, ignore a region, approve selected change, and share/import baseline.
- [ ] Include an authenticated-site task with a non-implementer. Correct
  misleading labels, lost users, accidental destructive actions, and blockers.
- [ ] Validate keyboard-only paths, focus, contrast, non-color status cues,
  accessible labels/screen-reader support available through GPUI, scaling/high
  DPI, window resizing, and reduced-motion behavior. Document unresolved gaps.
- [ ] Verify the sleek/modern visual system across setup, progress, review,
  empty/error/expired-login, and storage states—not only selected happy screens.
- [ ] Run the 1,000-page/3,000-target soak suite and record time, memory, storage,
  coverage, interaction latency, retries, and recovery on reference hardware.
  Changed-implementation local full coverage/pause/resume/cancel PASS; resource
  samples retained with guardian-label/inherited-child IO caveats. Native interaction
  latency NOT MEASURED; reference-hardware/native/independent qualification remains
  open, so the full checklist item is not checked by this local soak.
  - [x] ONE bounded corrected-source loopback1000×3 soak, unchanged capture budget,
    complete activation, exact pause preservation and planned large-run cancellation.
  - [x] Final read-only current/historical CAS/privacy/process/source/runtime audit.
  - [ ] Fresh independent correction verification and native interaction qualification.
- [ ] Exercise clean-machine installation/runtime setup, network/proxy errors,
  repair, secure-store failures, restart, cancel, and process cleanup on every
  supported OS/architecture/package combination.
- [ ] Build reproducible app/runtime release artifacts with signatures/hashes,
  runtime manifests, source-release availability, dependency/license notices,
  and MPL-2.0 compliance checks. Respect Node/Chromium/other dependency licenses.
  Private unsigned Linux debug app/source candidate and inventory are delivered;
  GLIBC_2.43 floor FAIL, 32 unknown/missing and 512 unreviewed target licenses,
  native obligations/signatures/current authenticated runtime still blocked.
- [ ] Provide platform signing/notarization where required for the chosen
  distribution; account for the sandbox implications of Linux packaging.
- [ ] Define app/runtime update behavior, security support windows, migration
  notifications, and rollback/recovery without evidence loss.
- [x] Provide a first-run guide, clear capture/image-mode/auth limitations,
  storage/sharing privacy guidance, support matrix, and opt-in diagnostic export.
  Implemented locally with explicit support/code limitations; human/native
  comprehension and native save-picker behavior remain unqualified.
- [ ] Confirm product naming and release hosting before publishing artifacts.
  No publication or external service setup is implied by this planning phase.

## Deliverables

- Tested release candidate and platform-specific installer/packages.
- Signed/verified runtime packs and a supported-version matrix.
- User documentation, licensing/source/notices, diagnostics, and release notes.
- Recorded usability findings, performance results, and resolved critical defects.

## Completion gate

All confirmed scope is implemented and all earlier gates still pass. Users
complete the main workflows without a terminal or code, and understand changed
versus failed/non-comparable evidence. Clean-machine setup works on each claimed
platform. No critical data-loss, secret-export, process-cleanup, coverage, or
usability defects remain. Unsupported configurations and remaining limitations
are stated plainly; do not broaden claims beyond the tested support matrix.

## After the first release

Revisit enhancements only with evidence from actual usage: reusable element
masks, recorded journeys, Basic Auth, other browser engines, schedules/CLI/CI,
history/report export, additional device presets, and optional collaboration.
None is a prerequisite for this first release or authorization to build it now.
