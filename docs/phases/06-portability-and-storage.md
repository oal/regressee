# Phase 6 — baseline sharing, history, and storage safety

## Outcome

Feature-complete candidate: a local project and selected baseline can be safely
shared/imported, and large retained histories have understandable manual cleanup.

Prerequisite: Phase 5 gate passed. Read `PLAN.md`, Sections 6, 8, and 11–13.

Implementation checklist below is local implementation, **not independent or
cross-machine acceptance**. Phase 5's bounded dependency gate passed independently
in `05-reverification.md`; its nonblocking tiny-mask border P3 remains open.
See `06-status.md` for final evidence and qualification disposition, and
`../portable-baseline-archive.md` for the archive/storage contract.
The resumed independent findings are retained in `06-resumed-verification.md`.
Bounded repair evidence and the fresh independent handoff are in `06-corrections.md`;
implementation passes do not close independent acceptance.

## Work

- [x] Finalize/version the archive manifest and round-trip schema tests.
- [x] Export project configuration, fixed inventory, selected baseline's full
  reference map, necessary metadata, and every referenced original screenshot.
- [x] Resolve composite baselines correctly across source runs; omit unrelated
  run history, sessions, browser profiles, secret-reference IDs, logs, secrets,
  and runtime/machine-local paths. Keep portable capture-profile metadata.
- [x] Show a privacy/export preview including confidential screenshots and
  potentially sensitive addresses/query data. Export nothing silently.
- [x] Add transactional import as a new project, naming-conflict handling,
  support-version checks, and explicit post-import environment/login setup.
- [x] Validate limits, normalized paths, duplicate entries, symlinks, hashes,
  images, and archive schemas. Test traversal/ZIP-bomb/malformed payload cases.
- [x] Preserve compatibility warnings for imported baselines; import never
  starts navigation or executes content automatically.
- [x] Complete baseline/run history, provenance, active-version visibility,
  and protected manual deletion/restore workflows.
- [x] Show actual per-project storage and deletion impact; count shared blobs
  correctly and protect anything still referenced by retained baselines.
- [x] Garbage-collect only unreferenced evidence after explicit deletion;
  distinguish discardable derived caches from original evidence.
- [x] Complete preflight/during-run low-space handling, atomic interrupted
  write recovery, artifact-integrity repair, and user-directed cleanup.
- [x] Prove session/credential data is excluded structurally, not merely hidden
  in the export UI. Audit logs and temporary files too.
- [x] Share analysis's bounded complete PNG CRC/chunk/zlib/Adler/exact-row gate
  before ingestion/import/active publication/restore/export; exact malformed
  specimens, all-row/file preservation and valid original roundtrips maintained.
- [x] Refuse unknown `inventory.coverage.Limited` fields with their schema cause;
  maintain valid coverage-variant roundtrip tests.
- [x] Diagnose owned recipient fixture starvation, add bounded concurrency/stage
  deadlines and prove one separate headed synthetic-login-ready two-target local
  recipient check; original width 600 remains separate from future width 640.

## Deliverables

- ZIP export/import UI and documented portable archive schema.
- Manual history/storage management with safe reference-aware cleanup.
- Security/fault-injection tests and clean-machine recipient tests.

## Completion gate

Export a composite baseline, import it on another machine, inspect every
reference image, establish a fresh session, and run the fixed target list.
No session/credential/private-log data appears in the ZIP. Malicious or partial
archives fail safely without writes outside the staging area or project
overwrites. Deleting a source run does not destroy retained baseline references.
Low disk never produces an apparently valid but corrupt/partial active baseline.
History persists until the user explicitly deletes it.

- [ ] Fresh independent Phase 6 services/security/GPUI/data verification.
- [ ] Fresh independent final-source P2/P3 corrections and complete recipient
  reverification; dependent Phase 7 reliance remains held pending this verdict.
- [ ] Actual other supported clean machine / cross-OS recipient flow.
- [ ] Native rendered/input, human usability and accessibility acceptance.
- [ ] Real OS-keystore, supported public runtime/distribution and maximum-scale
  storage/real filesystem-exhaustion qualification.

The local two-store signed fixture flow does not close these gates. Raw screenshots
are not redacted by masks; private URL queries remain visible in shared archives.

## Next

No Phase 7 implementation is part of the repair. After independent local repair
acceptance, Phase 7 validates the feature-complete candidate with actual non-developers and
qualifies signed/reproducible release artifacts across the support matrix.
