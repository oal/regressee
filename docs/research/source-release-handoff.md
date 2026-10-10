# Source-release preparation handoff — isolated work settled

Prepared 2026-10-10. **Preparation only; not Phase 7 completion, public release,
MPL-completeness certification or legal/security clearance.** Phase 6 verdict
belongs to the independent verifier; parent integration must wait for that gate.
Only this report was changed in the shared workspace. Existing implementation,
manifests, locks, design/status/scripts/tests, history and verifier data were not
edited. No nested agents, Cargo commands, builds, dependency downloads,
application/runtime/browser/native suites, signing, hosting or publication.

## Integration paths and actual results

All implementation/probes/evidence below live in the owner-private directory
`/tmp/opencode/source-release-prep-WE9jFZEH/` (0700; files 0600).

| File beneath that directory | Purpose / actual result |
| --- | --- |
| `source_release.py` | Self-contained Python stdlib deterministic private source-candidate helper |
| `test_source_release.py` | Independent synthetic-tree suite; no repository tests executed |
| `test-results-04.txt` | **30 tests passed, 0 failures, 0 errors, 0 skips** |
| `INTEGRATION.md` | Usage, complete policy, filesystem assumptions, limitations and parent follow-up |
| `candidate-current-source/` | One read-only private current-source candidate: archive, manifest, checksums |
| `snapshot-observation-01.json` | Actual observation start/end, epoch, source/archive/manifest hashes |
| `audit_candidate.py` | Independent archive/hash/permission checks plus final policy validation of saved candidate |
| `candidate-audit-01.json` | **Integrity and final-policy audit PASS**, never release approval |
| `integration-evidence-01.json` | Final helper/test/docs/log/audit hashes and compression environment |

Earlier evidence is preserved: `test-results-01.txt` had 26 tests and one
diagnostic-only failure (directory symlink safely refused with a generic reason);
`test-results-02.txt` passed all 26. The expanded `test-results-03.txt` had 30
methods and two failing subcases in one method, due to expected refusal precedence
and test-fixture restoration. Both omissions were actually refused; expected
reason and finally-based restoration were corrected. Final expanded run passed.

Final helper SHA-256:
`18710d2e4aea4694db5149c17b9e5238a5a1446907c016297f243b1b9261c7d0`.
Final test SHA-256:
`5d8a1c44685f466def92560d69b8f9c75dd340704a12e9431adccaed21ec9447`.
Observed test/compression environment: Linux, Python **3.14.7**, zlib **1.3.2**;
helper requires Python 3.11+ and safe Linux filesystem primitives.

## Implemented guarantees and tests

- Sorted, regular-file-only USTAR under `regressee-source/`; fixed
  `SOURCE_DATE_EPOCH` mtime, uid/gid 0, empty owner names, normalized 0644 file /
  0755 Python-shell script modes, fixed gzip epoch and empty filename. Byte-identical
  repeat tests change source permissions, mtimes and directory entry creation order.
- Canonical inventory contains exact source-relative paths, sizes, normalized
  modes and SHA-256 hashes. Independent tests recompute each archive/source hash,
  canonical-inventory identity, archive/manifest checksum and tar/gzip headers.
  No invented Git commit (`git_commit:null`), source roots, home/temp paths or raw
  metadata are placed in the manifest. Source identity is inventory SHA-256.
- Crates/application/worker/test source, SQL migrations, both worker locks,
  Cargo manifest/lock/toolchain, official root MPL text, third-party notices and
  GPUI Apache text/adaptation, relevant user docs and build/packaging/CI recipes
  are included. Synthetic corpus source/fixtures are retained, decoder evidence
  reports are excluded. Tests assert explicit representative coverage.
- Structural omission without traversal: `.runtime`, `target`, `artifacts`,
  dependency/cache directories, private user data, credentials, logs, keys,
  session state, internal ACTIVE_WORK/status/research/decision/evidence reports.
  Sentinel tests prove omitted content is absent from archive and manifest,
  including an excluded symlink to outside private data.
- Needed symlinks (contained, outside or dangling), hard links, special files,
  unsafe/traversal/nonportable paths and case collisions fail closed. Unknown
  source roots/extensions and source omissions refuse. Root `.cargo` is not
  presumed disposable; it requires a reviewed config/credential policy. Names
  `build`/`dist` are not blanket-excluded source locations.
- Workspace/local Cargo paths, explicit targets/build/license/readme paths,
  literal Rust include/path references and ordinary implicit modules/entrypoints,
  required workers/docs/notices and npm lock identity/dependencies are checked.
  Unsafe local npm dependencies and omitted lock entries refuse. Included private
  key PEM markers refuse. Missing/local unknown license declarations stay unknown,
  never become approved based on SPDX labels.
- Output requires a new directory outside source beneath an existing physical
  owner-private parent. NOFOLLOW fd-relative reads, private synced staging and
  Linux `renameat2(RENAME_NOREPLACE)` publish the whole bundle atomically. Unsafe
  output paths, symlink parents, nonprivate parents, inside-source output and
  existing files/directories/symlinks refuse. Collision-race, pre-publication
  failure and original-output-preservation tests pass; no overwriting fallback.
- Per-file mutation checks and a second selected-source inventory detect changes
  during candidate preparation; synthetic concurrent modification test refuses.

## One saved source observation, not release-final

Observed **2026-10-10T07:31:54.805101Z–07:31:54.936657Z**; epoch **1791617514**.
**148 selected files**, **3,470,843 source bytes**. The saved candidate was created
before final extra omission checks; the read-only saved-archive audit subsequently
passed those final checks without rereading or re-snapshotting the live workspace.

| Identity | SHA-256 |
| --- | --- |
| Canonical source inventory | `c6ea9b9c893b938e719d899b88a1d62941f6365139ddb64fd6d6c1aab928ae85` |
| `source.tar.gz` | `24035950f65dc49d4de8c6dee69772d4fae2243a9ad5985ea5ede53eb1ac9b3d` |
| `manifest.json` | `e8ba5e44920621740aa411dfb5a965fedbca17838f2fb12fbd51816f7d8c8b31` |

Lock-only dependency inventory: **769 Cargo entries** (five local declaration
entries; **764 external license declarations unknown**) and **3 npm entries**.
Only public name/version/license/source/checksum fields are emitted. Cargo retains
actual SHA-256 checksums; npm retains actual SHA-512 SRI. **Every declaration is
unreviewed; unknown and release gates remain explicitly blocked/approved:false.**
No target-specific resolution, binary SBOM or redistribution clearance is claimed.

## Limits and parent follow-up after Phase 6

1. Review/integrate helper/tests where appropriate, and re-run the isolated suite
   against the integrated version. This preparation has no remaining unfinished
   local step; incorporation into shared release implementation is parent-owned.
2. Freeze reviewed source and create a new candidate for actual final artifacts.
   The saved observation cannot represent later edits or a future verifier verdict.
   Two reads are best-effort change detection, not an atomic multi-file snapshot.
3. Review preferred modification forms, generated inputs/dynamic build/Rust/JS
   references, third-party notices and content privacy. The conservative static
   parser is not a compiler, full dependency resolver or generic secret scanner.
   It never marks full MPL corresponding-source completeness approved.
4. Complete target/artifact-specific dependencies, actual license texts/notices,
   modifications and matching sources/configuration/relinking obligations. Node
   internals, exact browser/helpers/FFmpeg, bundled native libraries and packaging
   tools are **not bundled or cleared by this source-only candidate**. No Cargo
   metadata command was run; unknown registry licenses were not guessed.
5. Update public help/build/source-obtainment guidance when appropriate. Retained
   README/fixture docs have historical/internal links whose reports are intentionally
   absent; do not fix this by exporting private verifier reports. Public-doc
   readiness, exact executable-source binding and timely source retrieval remain gates.
6. Qualify platform publication/permissions, real disk-full/fsync/power-loss behavior
   and native runtime/build/signing separately. The helper is intentionally Linux-only;
   unsupported primitives fail closed. Private synced publication tests do not prove
   crash durability on every filesystem or Windows/macOS ACL behavior. Same-UID/root
   attackers and concurrent ancestor renames are outside its private-parent trust
   model. An interrupted pre-rename stage may remain private; clean only exact
   owned paths with explicit approval. Parent-fsync failure can leave a complete
   bundle with uncertain crash durability: retain and inspect, do not overwrite.
7. Pin Python/zlib for cross-machine byte reproducibility; compression can change
   across toolchain versions. Buffered limits are 64 MiB/file, 512 MiB selected
   total, 20,000 files; peak memory exceeds selected total. This is not binary or
   runtime-pack reproducibility and does not qualify macOS runtime bundle links.
8. Owner approval remains necessary for legal/security review, name/source hosting,
   release keys/signing/publication and all Phase 7 native/platform/human/scale gates.

Read-only inputs: root `LICENSE`, notices, Cargo/toolchain and worker
manifests/locks, relevant source/user docs; `docs/research/release-qualification.md`,
`docs/research/phase7-entry-checklist.md` and `docs/phases/07-release.md`.
No completion checkbox or Phase 6/7 status verdict is changed by this handoff.
