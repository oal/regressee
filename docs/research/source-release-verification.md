# Isolated source helper — independent parent check

Date: 2026-10-10. **PASS within private source-candidate helper scope only.**
Not Phase 7 completion, a public release, corresponding-source certification,
dependency license/security approval, or app/runtime packaging qualification.

## Evidence and checks actually run

Evidence: `/tmp/opencode/source-release-independent-byyehc77/`.
The helper and original test suite were copied into this new private directory;
their hashes matched the settled preparation handoff before execution:

- Helper: `18710d2e4aea4694db5149c17b9e5238a5a1446907c016297f243b1b9261c7d0`
- Tests: `5d8a1c44685f466def92560d69b8f9c75dd340704a12e9431adccaed21ec9447`

Commands, run against isolated copies, not repository/application suites:

```sh
python3 -B test_source_release.py --report /tmp/opencode/source-release-independent-byyehc77/maintained-results.txt
python3 -B independent_probes.py
```

**30 maintained tests and 12 new independent probes passed**, zero errors,
failures or skips. Logs, the independent probe source, and JSON results remain in
the evidence directory. Test-owned synthetic trees were removed by their scoped
temporary-directory cleanup; no application/browser/display processes launched.
The preparation directory, saved candidate, and shared implementation were not
modified. No workspace/Cargo/Node/native tests or live-source snapshot ran.

The independent probes exercised:

- Exact archive/source/inventory bytes and SHA-256, normalized headers, explicit
  unapproved release/license gates, and absence of the synthetic local root in
  manifest metadata.
- An omitted runtime-directory symlink targeting a private outside sentinel:
  not traversed, not archived, and outside bytes unchanged.
- Referenced omitted private inputs and required Cargo license files: exact
  refusal before publication.
- Directory case collision, needed directory symlink, and FIFO: exact refusal;
  no FIFO hang or outside write.
- Credential-bearing registry URL: sanitized, exact refusal.
- Injected total-byte bound and source addition during snapshot: no publication.
- Existing complete output collision: every existing output byte preserved.
- Dynamically generated Rust include: candidate creation remains possible, but
  completeness/owner-review gates explicitly remain blocked.

## Integration constraints

No material defect was reproduced within these tests. This is deliberately a
Linux-only, owner-private, no-clobber source-candidate tool; no Windows/macOS
filesystem qualification is implied. The fixed inclusion policy must be reviewed
when Phase 7 introduces build/help/package files. Real power-loss/disk-full and
cross-toolchain compression reproducibility remain unqualified.

**Static source checks are not complete dependency/build interpretation.** The
new dynamic-include probe confirms the documented boundary: a private candidate
may be emitted without generated source inputs, but never marked release-ready.
Final source completeness, runtime resources, generated inputs, public help
links, artifact-specific third-party notices/sources, and matching executable
binding require a separate review against the actual final artifacts.

The preparation's saved 148-file candidate predates future repairs; it is not
corresponding source for a later binary. Its 764 unknown external Cargo license
declarations remain blocked, not cleared by this check. No publish/sign/hosting
operation was requested or performed. Phase 6's local integrity and recipient
gates remain independent prerequisites to dependent Phase 7 work.
