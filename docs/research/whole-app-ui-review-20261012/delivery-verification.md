# Independent post-plan CI / preview delivery verification

Date: 2026-10-13. Base: `b46dff59c77b00fd36249cb42c680af4b5312516` plus
the uncommitted delivery patch described in [delivery-report.md](delivery-report.md).

**PASS — locally verified, ready for the parent to commit/push and observe hosted
CI. No implementation fixes required.** This is not hosted CI, native desktop,
accessibility, fresh-runtime installation or supported-release acceptance.
All independent suites have completed; none is pending. No product edits, renders,
commit, push, dispatch, tag, publication or cancellation were performed.

The original verification below is historical. The appended **timeout follow-up**
records hosted outcomes at the subsequently pushed `4f84363` and supersedes the
original unpushed status and 30-minute budget assessment.

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

## Independent timeout follow-up — 2026-10-13

Base: `4f843635c8632ba725cb360e1e69d3d8e80e7c50`, committed/pushed by the
parent. **PASS LOCAL — finite scoped budget patch ready for parent commit/push;
no fixes required.** This uncommitted follow-up has no hosted result yet.

### Actual hosted evidence

Independently retrieved `gh run view <run> --json status,conclusion,headSha,jobs`
for both runs: each is completed against the exact base SHA above. Durations
were independently calculated from returned timestamps and agree with the
implementation report.

| Run | Host | Actual outcome / job duration | Contract duration |
|---|---|---|---|
| Foundation **38324688298** | Ubuntu 24.04 | SUCCESS / 54m39s | SUCCESS / 24m52s |
| Foundation **38324688298** | macOS 14 | SUCCESS / 52m55s | SUCCESS / 24m49s |
| Foundation **38324688298** | Windows 2025 | SUCCESS / 40m51s | SUCCESS / 17m03s |
| Phase 0 **38324688313** | Ubuntu 24.04 | SUCCESS / 16m43s | SUCCESS / 13m05s |
| Phase 0 **38324688313** | macOS 14 | SUCCESS / 33m18s | SUCCESS / 25m14s |
| Phase 0 **38324688313** | Windows 2025 | FAILURE / 31m12s | FAILURE / 30m00s timeout |

Foundation's combined UI build/all-feature steps also succeeded on all three
hosts (Ubuntu 27m15s, macOS 25m27s, Windows 20m01s). Its strict Clippy gate passed.
Fresh `gh run view 38324688313 --log-failed` retrieval confirms Windows started
the qualification binary at 23:58:32Z, reported the distinct persisted
100/1,000-page test still running after 60 seconds, and interrupted it at
00:27:08Z with `The action 'Rust core tests' has timed out after 30 minutes.`
No Rust assertion failure was found. **The interrupted suite did not pass**;
Windows's subsequent runtime setup/smoke/protocol/native compile stages were
skipped. Foundation's Windows success does not replace those Phase 0 stages or
prove why this run was slower; a hang/performance diagnosis is not established.

Independent JSON and failure log evidence is retained in the approved temporary
directory as `delivery-verifier-followup-foundation.json`,
`delivery-verifier-followup-phase0.json`, and
`delivery-verifier-followup-phase0-failure.log`.

### Scope, bounds and local checks

- Workflow diff normalization independently proves **only timeout values and
  comments changed**. Commands, full default bulk tests, workspace features,
  strict all-target/all-feature `-D warnings`, host matrices, launcher/ownership,
  sandbox/ACL checks, SHA pins, permissions and preview publication safeguards
  are identical to the base. No retries, failure swallowing, ignored tests or
  conditional gate bypasses were added. Product/scripts and Phase 7 historical
  evidence are unchanged; delivery docs correctly preserve earlier evidence.
- Foundation now has a **90-minute job** and **45-minute contract and combined
  UI build/test steps**. Phase 0 has a **90-minute job / 45-minute core-test
  step**. Preview's contract step is **45 minutes**, with its build job still
  **90 minutes** and publish job still **10 minutes**. These provide reasonable
  measured-host headroom while retaining hard limits; 45 minutes is not a
  demonstrated completion time for the interrupted Windows test. Job limits
  still cap total setup/lint/runtime/build time even if individual step maxima
  sum to the entire job budget. Further timeouts require actual-log diagnosis,
  not unlimited budget escalation or a fabricated pass.
- **PASS, exit 0:**
  `/var/home/olav/Data/opencode/tmp/opencode/delivery-validation-bin/actionlint
  .github/workflows/foundation.yml .github/workflows/phase0.yml
  .github/workflows/release-preview.yml`, using installed **v1.7.7** and default
  checks, with no checks disabled.
- **PASS, exit 0:** `TMPDIR=/var/home/olav/Data/opencode/tmp/opencode python -B
  scripts/test_ci_run.py` (**6 tests**) and the same environment with `python -B
  scripts/test_package_preview.py` (**8 tests**); `git diff --check` also passed.
  No local verification process remains pending. Full Rust suites were not
  rerun for this YAML/docs-only patch; their prior local and actual hosted
  Foundation results remain separately scoped evidence.

### Remaining requirements

Parent may commit/push this locally verified patch. Require the **new exact
commit's Foundation and Phase 0 normal three-host matrices** to complete
successfully: six jobs, including complete Windows bulk qualification followed
by its previously skipped runtime and GPUI stages. The old Windows timeout
remains a failure, not a waived or retroactively repaired result. Only after
those outcomes should the parent proceed to a fresh preview tag and verify both
Linux/Windows release builds, ABI/static-CRT/shaders/DLL checks, exact-source and
checksum binding, publication and actual downloads. No tag availability was
rechecked or reserved during this follow-up. Existing native/accessibility,
fresh-installation, macOS supervision and supported-release limitations remain.

**Finite defect list: none.** No commit, push, dispatch, tag, publication,
cancellation, product edit or render performed by this verifier.
