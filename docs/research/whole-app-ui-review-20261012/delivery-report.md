# Post-plan CI and preview delivery implementation

Date: 2026-10-13. Base: `b46dff59c77b00fd36249cb42c680af4b5312516`.
Implementation only; independent verification and parent commit/push/tag remain
next. No commit, push, workflow dispatch, tag, release or external-run cancellation
was performed by this agent. No product layout/style changes or new renders.

**Final local verdict: READY for independent parent verification.** All listed
local gates finished successfully; no local suite remains running or pending.
This is not a hosted CI, native accessibility or supported-release verdict.
The original handoff below is preserved as historical evidence. The appended
**post-push timeout follow-up** records the later hosted outcomes at `4f84363`
and supersedes its former unpushed/pending-hosted status.

## Diagnosis and changes

Foundation run **38315044213** (`135d3eb`) failed on **all three hosts** at strict
Clippy: `Control::Guided(55 | 56 | 57)` was a manual range pattern. Contract tests
had passed before linting. Phase 0 **38315044275** passed. The base's Foundation
**38321620062** and Phase 0 **38321620015** were still running at inspection;
their outcomes are not claimed as passing this patch.

Local all-target/all-feature linting additionally exposed task-link parity,
setup paint type complexity, setup continuation's nested conditional and the
bulk fixture's constant-size mutable chunks. Fixed these without suppression,
feature exclusions, disabled checks or changed behavior. The bulk fixture still
encodes the same 16 page-identity RGB pixels and persists distinct 100/1,000-page
datasets. GPUI's two `taffy.rs` float fallback warnings now use explicit `f32`
literals, preserving the same grid dimensions.

Foundation now runs offline launcher/preview packaging tests on every host and
the existing **Linux-only** source archive tests on Linux. Phase 0 and preview
builds also test the CI launcher. New launcher tests check argument/exit-code
propagation, owner-before-child order, fail-closed ownership setup and refusal
outside hosted CI. They do not establish native Windows ACL qualification.

The default persisted bulk tests remain enabled on every platform. `ci-run.py`
has no per-process timeout; GitHub previously bounded only whole jobs. Rust
contract steps and Foundation's UI build/test step now have explicit **30-minute**
step limits inside unchanged 60/45/90-minute Foundation/Phase 0/preview job limits.
No tests were ignored or omitted to achieve green status.

`docs/preview-release-notes.md` describes the completed whole-app workflow changes,
synthetic scale-test scope, local-data/sharing privacy and outstanding native/AT/
release gates. It is appended to GitHub release notes and included in platform
help archives; the private source helper's user-doc allowlist includes it.
Application version remains `0.1.0`; no forced version bump.

## Local commands and results

Rust commands use the pinned Rust **1.98.1**, existing cached prerequisites,
`LIBRARY_PATH="$PWD/.runtime/link-libs"` and
`TMPDIR=/var/home/olav/Data/opencode/tmp/opencode`. `--offline` is the only added
Cargo flag compared with Foundation, to enforce the available local cache.

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | PASS |
| `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings` | PASS; no repository/vendored lint warnings |
| `python scripts/ci-run.py cargo test --locked --offline --workspace --features regressee-app/test-support` | PASS, exit 0: **178 passed / 0 failed / 27 ignored** |
| `cargo build --locked --offline --workspace --features ui --bins` | PASS; native link, not desktop launch |
| `python scripts/ci-run.py cargo test --locked --offline --workspace --all-features` | PASS, exit 0: **358 passed / 0 failed / 45 ignored** |
| `python -B scripts/test_ci_run.py` | PASS, 6 tests on Linux |
| `python -B scripts/test_package_preview.py` | PASS, 8 tests, including notes/help inclusion and no `.runtime` collection |
| `python -B scripts/test_source_release.py --report <approved-temp>/delivery-source-release-report.json` | PASS, 32 tests, 0 failures/errors/skips |
| `python scripts/ci-run.py cargo run --locked --offline --bin regressee-spike -- smoke` | PASS; existing private runtime, sandbox true, owned loopback self-tests |
| `python scripts/ci-run.py cargo run --locked --offline --bin regressee-spike -- protocol-test` | PASS; browser crash/relaunch, bounded cancel, worker crash/restart and command whitelist/frame budget |
| `git diff --check` | PASS |

An initial contract run was interrupted by the local harness's **120-second**
command timeout, not a Rust assertion failure; the complete suite was restarted
with a **1,800-second** bound, without excluding bulk qualification. Local logs
are in the approved temporary directory: `delivery-clippy.log`,
`delivery-contract-tests.log`, `delivery-ui-build.log`,
`delivery-all-features-tests.log`, `delivery-phase0-smoke.log` and
`delivery-phase0-protocol.log`. They are not packaged product data.
The two already-launched suites were awaited using blocking Linux `pidfd_open`
and `poll` readiness on their exact owned shell PIDs, not repeated status/log
polling and not duplicate suites. Both completion notifications reported success.
Both logs contain the passing
`phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention`
test: the qualification test binary took **553.27s** in the contract suite and
**526.43s** in the all-features suite. Totals sum the final Rust test-result lines
within each log; the suites overlap and must not be added as unique coverage.
Ignored tests are the existing opt-in native/browser/evidence tests; no new
ignore or test exclusion was added. Formatting and diff checks were rerun after
the suites and passed.
Phase 0 smoke/protocol commands also set `REGRESSEE_PHASE0_WORKER_STDERR=1`.
They reuse the existing installed private runtime; bootstrap, pack and repair
were not rerun over the user's existing `.runtime` tree. This is not a fresh
runtime-install or complete Phase 0 workflow qualification.

Cargo retains an **upstream future-compatibility advisory** for
`proc-macro-error2 2.0.1`: private `extern crate proc_macro` re-export (E0365,
Rust issue 127909), shown by `cargo report future-incompatibilities --id 1`.
It is not a repository lint failure and is not suppressed or called warning-free.
A dependency upgrade/patch needs separate locked cross-host review before a
future toolchain upgrade; the pinned toolchain currently accepts it.

## Cross-host and release readiness review

- Only Linux is installed locally (`rustup target list --installed`); Windows/
  macOS compilation and runtime outcomes for this patch require hosted CI.
  A final read-only status inspection still showed the **base** Foundation
  38321620062 and Phase 0 38321620015 in progress on all three hosts, with no
  completed failure step. Status JSON is retained in the approved temp directory
  as `delivery-head-foundation-status.json` and `delivery-head-phase0-status.json`.
  This patch has not been pushed and therefore has no hosted validation yet.
- The shared bulk fixture uses portable Rust/PNG/Store APIs. Optional retained
  evidence uses an explicitly local absolute path only when its opt-in environment
  variable is set; workflows do not set it. Synthetic rendering metadata is
  deliberately synthetic, not a claim about the host. Linux renderer capture
  calls/tests remain Linux-cfg guarded; simulated GPUI tests remain cross-host.
- The shared CI launcher changes only the hosted Windows process token's default
  owner before tests, not product ACL rules or existing paths. New fixtures use
  that same launcher in Foundation/Phase 0/preview. macOS supervision remains a
  known product gap; no preview is produced for macOS.
- Existing checkout/setup/cache/artifact actions remain immutable SHA-pinned.
  Permissions remain read-only for builds and write only for publishing; checkout
  credentials are not persisted. No action pin was replaced or loosened.
- Windows preview still builds with `-C target-feature=+crt-static`, discovers SDK
  `fxc.exe`/MSVC `dumpbin.exe`, compiles release shaders, and refuses MSVC DLL
  dependencies. Linux still builds on Ubuntu 24.04 and enforces GLIBC ≤2.39.
- Publishing still waits for **both** builds, includes the exact tracked source
  via `git archive "$GITHUB_SHA"`, checks all expected platform assets and creates
  `SHA256SUMS`. It creates an immutable unsigned **prerelease** via a draft and
  does not overwrite an existing release. Runtime/session/private trees are not
  swept into platform archives. Native/transitive license audit remains pending.
- Existing downloadable prerelease: `preview-1-e7eab2a` (2026-10-11). No `v0.1.0-preview.*`
  tags were returned by `git ls-remote --tags origin` at inspection, and
  `gh release view v0.1.0-preview.2` returned `release not found`. Recommend
  **`v0.1.0-preview.2`**, rechecking remote tag/release availability immediately
  before parent creation. This does not imply supported release qualification.

## Parent's remaining actions

1. Independently inspect the patch and rerun the required local gates before
   committing. Local implementation results above are complete, but do not replace
   independent verification. Preserve earlier bounded evidence.
2. Commit/push verified changes. Observe the new Foundation and Phase 0 runs on
   Ubuntu, Windows and macOS, and diagnose actual failures without dropping tests
   or bypassing ownership/sandbox checks. Old base runs cannot validate this patch.
3. Only then create/push a fresh preview tag to the verified commit. Observe the
   Linux/Windows release build, static CRT/shaders/DLL and ABI checks, publication,
   complete downloadable assets and hashes/source binding.
4. Verify download availability for the work computer. Retain unsigned/experimental,
   internet-on-first-use, Linux shared-library and blocked native/AT/full-release
   limitations. Windows Server compilation is not Windows 11/NVDA qualification.

Full native/accessibility/usability/security/installer/license release acceptance
remains **BLOCKED or NOT TESTED**, as recorded in Phase 7 and the support matrix.

## Post-push timeout follow-up — 2026-10-13

Base: **`4f843635c8632ba725cb360e1e69d3d8e80e7c50`**, committed/pushed by the
parent after independent verification. This implementation agent performed only
read-only hosted inspection and the finite local budget adjustment below. No
commit/push/dispatch/tag/release, external cancellation or old-run alteration.

### Actual hosted outcomes (not reclassified)

`gh run view <run> --json status,conclusion,headSha,jobs` confirms both runs
completed against that exact commit. Durations below are calculated from the
returned job/step `startedAt` and `completedAt` timestamps:

| Workflow / run | Host | Job outcome / duration | Contract step | UI build + all-features step |
|---|---|---|---|---|
| Foundation **38324688298** | Ubuntu 24.04 | **SUCCESS / 54m39s** | SUCCESS / 24m52s | SUCCESS / 27m15s |
| Foundation **38324688298** | macOS 14 | **SUCCESS / 52m55s** | SUCCESS / 24m49s | SUCCESS / 25m27s |
| Foundation **38324688298** | Windows 2025 | **SUCCESS / 40m51s** | SUCCESS / 17m03s | SUCCESS / 20m01s |
| Phase 0 **38324688313** | Ubuntu 24.04 | **SUCCESS / 16m43s** | SUCCESS / 13m05s | Separate Phase 0 sequence passed |
| Phase 0 **38324688313** | macOS 14 | **SUCCESS / 33m18s** | SUCCESS / 25m14s | Separate Phase 0 sequence passed |
| Phase 0 **38324688313** | Windows 2025 | **FAILURE / 31m12s** | **30m step timeout** | Subsequent runtime/native steps did not run |

The preserved Windows failure log is
`/var/home/olav/Data/opencode/tmp/opencode/delivery-hosted-phase0-failure.log`.
It reaches `tests\\qualification.rs` and starts
`phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention`
at **2026-10-12 23:58:32Z**. It is still running when the runner interrupts the
launcher at **2026-10-13 00:27:08Z**, about **28m37s** later. The final annotation
is exactly `The action 'Rust core tests' has timed out after 30 minutes.`
No Rust assertion failure appears in that log; the interrupted suite is **not a
pass**, and the unexecuted Windows runtime stages remain unverified by that run.
The same complete default suite passed Foundation Windows in 17m03s. This
demonstrates substantial hosted timing variance; no product performance or I/O
profiling diagnosis is claimed.

Metrics JSON is retained in the approved temp directory as
`delivery-timeout-foundation-metrics.json` and
`delivery-timeout-phase0-metrics.json`. Foundation really is green on all three
hosts (including strict Clippy); Phase 0 really is failed on Windows. Neither
old run is altered or relabeled.

### Finite scoped repair

| Workflow | Previous job / relevant step limits | New job / relevant step limits |
|---|---|---|
| Foundation | 60m job; 30m contracts and 30m UI build/tests | **90m job; 45m contracts and 45m UI build/tests** |
| Phase 0 | 45m job; 30m core contracts | **90m job; 45m core contracts** |
| Preview build | 90m job; 30m application contracts | **90m job unchanged; 45m application contracts** |

The 45-minute step budget adds 50% headroom over the observed interrupted
30-minute Windows step while remaining finite. Foundation's 90-minute job cap
adds cold-host headroom over its observed 54m39s maximum, rather than relying on
just 5m21s of spare time. Phase 0 needs room for both contracts and its subsequent
runtime installation/smoke/protocol/native compilation. Preview retains its
existing 90-minute job ceiling for contracts plus release compilation/packaging;
no release-build timing claim is invented. The job ceilings still override the
sum of individual step maxima, so two 45-minute steps do not grant unlimited
setup/lint time. A further timeout must be diagnosed from its actual logs, not
automatically increased indefinitely.

Only timeout values/comments in three workflows and this report/plan change in this
follow-up. All test commands, the distinct 100/1,000-page default qualification,
strict Clippy, sandbox/owner/ACL behavior, build matrix, immutable action pins,
permissions and preview publication gates remain unchanged. No retries, ignored
tests, product performance redesign or UI changes are introduced.

### Follow-up local validation and exact next matrix

- **PASS, exit 0:** `actionlint` **v1.7.7** on
  `.github/workflows/foundation.yml`, `.github/workflows/phase0.yml` and
  `.github/workflows/release-preview.yml`, with its default checks. Installed only
  in the approved temp area; no repository dependencies changed.
- **PASS, exit 0:** `TMPDIR=<approved-temp> python -B scripts/test_ci_run.py`
  (**6 tests**) and `python -B scripts/test_package_preview.py` (**8 tests**).
- **PASS, exit 0:** `git diff --check`.
- Full Rust suites were **not duplicated** for a YAML/docs-only follow-up.
  Their complete local results and newly passing hosted Foundation checks remain
  separately recorded above. This unpushed timeout patch has no hosted result yet.

**Scoped local verdict: READY for independent timeout-patch verification.**
The parent verifier should check the YAML diff and bounded values, then the parent
may commit/push. The resulting exact new commit must run **Foundation and Phase 0
on Ubuntu 24.04, Windows 2025 and macOS 14**; require all six jobs to succeed,
especially complete Windows bulk contracts followed by runtime smoke/protocol and
GPUI compile. Do not replace the normal matrix with Windows-only evidence or
claim the old timeout succeeded. Only after the repaired matrix succeeds should
the parent proceed to a fresh preview tag, then require the preview's **Ubuntu
24.04 Linux x86_64 and Windows 2025 x86_64** build/publication/asset checks. Native,
AT and supported-release limitations remain unchanged.
