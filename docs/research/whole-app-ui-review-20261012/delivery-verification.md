# Independent post-plan CI / preview delivery verification

Date: 2026-10-13. Base: `b46dff59c77b00fd36249cb42c680af4b5312516` plus
the uncommitted delivery patch described in [delivery-report.md](delivery-report.md).

**PASS — locally verified, ready for the parent to commit/push and observe hosted
CI. No implementation fixes required.** This is not hosted CI, native desktop,
accessibility, fresh-runtime installation or supported-release acceptance.
All independent suites have completed; none is pending. No product edits, renders,
commit, push, dispatch, tag, publication or cancellation were performed.

## Independent checks

Rust used pinned `rustc 1.98.1`, the existing local cache, and
`LIBRARY_PATH="$PWD/.runtime/link-libs"` with
`TMPDIR=/var/home/olav/Data/opencode/tmp/opencode`. Only `--offline` was added to
the Foundation Cargo commands; no feature, target category or default test was
dropped. Native Linux is the only installed target.

| Exact command | Completed result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings` | PASS |
| `python scripts/ci-run.py cargo test --locked --offline --workspace --features regressee-app/test-support` | PASS: **178 passed, 0 failed, 27 ignored** |
| `cargo build --locked --offline --workspace --features ui --bins` | PASS: native link, not desktop launch |
| `python scripts/ci-run.py cargo test --locked --offline --workspace --all-features` | PASS: **358 passed, 0 failed, 45 ignored** |
| `python -B scripts/test_ci_run.py` | PASS: 6 tests on Linux |
| `python -B scripts/test_package_preview.py` | PASS: 8 tests |
| `python -B scripts/test_source_release.py --report /var/home/olav/Data/opencode/tmp/opencode/delivery-verifier-source-report.json` | PASS: 32 tests, no failures/errors/skips |
| `git diff --check` | PASS |

The default
`phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention`
test passed in both complete suites; its binaries took **544.52s** and **531.37s**,
respectively. Counts overlap and are not unique coverage totals. Existing ignored
native/browser/evidence tests remain ignored; the patch adds no ignores.
Independent completed logs are `delivery-verifier-contracts.log`,
`delivery-verifier-ui-build.log`, and `delivery-verifier-all-features.log` in the
approved temporary directory above. The sequential suite shell was awaited with
blocking `pidfd_open`/`poll` on its exact owned PID, then its completed notification
and final logs were inspected; no pending-run result is represented as passing.

Cargo still reports the existing upstream future-incompatibility advisory for
`proc-macro-error2 2.0.1`. This is not a strict Clippy failure, is not suppressed,
and requires separately reviewed dependency work before a future toolchain upgrade.

## Patch review

- **Underlying warnings fixed, not pipeline weakened.** `55 | 56 | 57` and
  `55..=57` match exactly the same controls. The setup paint alias has the same
  private type, and the let-chain preserves short-circuit ordering and all setup
  continuation/discovery effects. Task-link parity remains identical; index zero
  returns before subtraction. No API, safety, privacy, focus or visual-style
  change was introduced.
- **Bulk pixels unchanged.** A separate pinned-Rust executable compared the old
  `chunks_exact_mut(3)` loop with `as_chunks_mut::<3>().0.iter_mut()` for both
  sequences of all 1,000 pages, including the center-region writes: all complete
  RGB buffers were byte-identical, and all 1,000 baseline buffers were distinct.
  The fixed buffer length is divisible by three; the same first 16 triples are
  visited in the same order. PNG encoding and persistence are unchanged.
- **Vendored literal meaning preserved.** Taffy 0.9's `length` and `fr` helpers
  accept `Input: Into<f32>`; explicit `0.0_f32` and `1.0_f32` preserve the same
  exactly representable values and grid dimensions without a lint suppression.
- **Foundation equivalence and host coverage.** Foundation retains workspace
  test-support contracts, all-target/all-feature strict Clippy, UI-bin native
  build and workspace all-feature tests on Ubuntu 24.04, Windows 2025 and macOS
  14. Phase 0 retains the same three hosts. Preview retains Linux and Windows
  targets and test-support contracts. Source-helper tests are Linux-only because
  the helper requires Linux safe filesystem primitives; launcher and preview
  packaging tests run on every intended host.
- **Correct bounds.** Contract steps have 30-minute limits in all three
  workflows; Foundation's combined UI build/all-feature test step also has a
  30-minute limit. Existing job limits remain Foundation 60, Phase 0 45, preview
  build 90 and publish 10 minutes. The launcher itself has no child timeout.
  Local durations fit, but hosted performance remains to be measured.
- **Portable, safe launcher mocks.** Windows branches mock owner setup and child
  spawning; POSIX routing is explicitly mocked for its contract. The real child
  test skips on Windows (five tests there, six on Linux/macOS). Refusal explicitly
  sets `GITHUB_ACTIONS=false` before any WinDLL lookup. An additional independent
  run of all five non-child contracts with inherited `GITHUB_ACTIONS=true` and a
  fail-on-call `ctypes.WinDLL` mock passed without any native token API call.
  `ci-run.py` itself is unchanged; product ownership/ACL checks are not weakened.
- **Packaging and notes.** Platform help explicitly includes the new public
  preview notes and refuses missing help before creating output. Packaging reads
  an explicit binary/docs/notices list, not a recursive runtime/data tree; tests
  verify both names and private fixture bytes are absent. The private source
  helper adds only `preview-release-notes.md` to its user-doc allowlist and keeps
  research/internal docs, runtime, session and data exclusions unchanged. The
  preview publisher instead uses exact tracked `git archive "$GITHUB_SHA"`;
  these are distinct archive policies. No tracked `.runtime`, target, artifacts,
  SQLite databases/sidecars or key/PEM files were found in the checkout inventory.
  New documents must be committed by the parent to enter the source archive.
- **Release safeguards retained.** Existing checkout/setup/cache/upload/download
  action SHA pins, non-persisted checkout credentials and read-only build
  permissions remain unchanged; only publish has contents-write. Publication
  waits for both builds, verifies all three expected platform downloads, archives
  the exact commit, and computes `SHA256SUMS`. Build metadata checks full SHA
  equality and hashes the binary. Draft-first unsigned prerelease creation does
  not overwrite an existing release; no replacement/upload-clobber path was
  added. This preserves the workflow's no-overwrite policy, not a newly verified
  GitHub service-side immutability setting. Windows static CRT, shader tool
  discovery and MSVC DLL refusal, plus Linux GLIBC <=2.39 checks, are unchanged.

## Remaining gates and parent handoff

1. Commit/push the verified patch and documents. Require new Foundation and Phase
   0 outcomes on **all three hosts**; old base runs cannot validate this patch.
   Windows owner-token behavior and cross-host compile/runtime outcomes remain
   hosted gates, not established by Linux mocks or this local verdict.
2. Recommend **`v0.1.0-preview.2`**. Independent read-only checks found no remote
   tag (`git ls-remote --exit-code --tags origin refs/tags/v0.1.0-preview.2` exited
   2 with no matches) and no release (`gh release view` returned `release not
   found`). Recheck immediately before creation; nothing was reserved/published.
3. After hosted checks, bind the new preview tag to the verified commit. Observe
   both release builds and actual CRT/shader/DLL/ABI gates, exact-source archive,
   checksum coverage, draft-to-prerelease publication and complete downloads.
   This review verifies the mechanism, not newly produced release artifacts.
4. Native input/default graphics, Windows 11/NVDA, macOS/VoiceOver, clean install,
   fresh runtime bootstrap/pack/repair, usability/security/installer and complete
   native/transitive license acceptance remain **BLOCKED or NOT TESTED** per
   existing evidence/support documents. macOS has no preview download and retains
   its supervision gap. Existing implementation smoke/protocol logs were read and
   contain successful sandboxed-runtime results, but those commands were not
   independently repeated here and do not qualify a fresh installation.

**Finite defect list: none in the reviewed delivery patch.** Proceed with the
parent's verified commit/push and hosted validation, not a supported-release claim.
