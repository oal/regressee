# Independent automatic-runtime verification — 2026-10-10

## Verdict and ownership

**PASS within the tested backend logic/inert-fixture scopes below.** No material
fault was found requiring an implementation correction. This is not runtime,
release, native UI, browser security, OS supervision or accessibility qualification.

The verifier did not author the automatic-setup implementation. It read the
implementation handoff, the complete affected portions and maintained new tests
in `product.rs`, `jobs.rs`, `distribution.rs`, plus relevant unchanged resolver,
auth/crawl, storage and network/supervisor continuations. It did **not** independently
review its earlier timezone/UI work or adopt the parent's UI test results as
backend evidence. Only this new report and
`crates/app/tests/automatic_runtime_setup_independent.rs` were added. No backend,
UI, Cargo or original report was edited.

## Frozen source identity

Before/after SHA-256 manifests agree for all four protected inputs:

| Frozen input | SHA-256 |
|---|---|
| `crates/app/src/product.rs` | `ef8303c4215386d79f1cd031d185cdb15f47df6eea8ae6ddbbc64c2e40cef485` |
| `crates/app/src/jobs.rs` | `b36e20bfc69160f984d952b700381c28663d5c9d751302cabbbc8ebf15dce418` |
| `crates/runtime/src/distribution.rs` | `3cf20fada8cd820a27e2d68ed05c395c30020695b7de28ae83cc4043537216ff` |
| `docs/research/automatic-runtime-setup-implementation-20261010.md` | `93785ba3a871071b1883e60f14ec10015272e36483007a98c6e34b98fd04db28` |

Main/UI and unrelated parent work were intentionally **not** frozen by this
manifest. The original implementation report remains unchanged, including its
historical “tests NOT RUN” handoff; the execution results live here.

## Executed checks

Linux x86-64, default/non-UI features, Cargo `--locked --offline`, every test run
serialized with `--test-threads=1`. No full old-phase/full-workspace suite, clippy,
native, browser, guardian, soak or package-artifact qualification was run.

1. `cargo test --locked --offline -p regressee-app --lib product::automatic_runtime_tests -- --test-threads=1`
   — **7 passed**, 0 failed, 16 filtered out; execution 0.13s, compile 4.52s.
2. Three separately isolated `--lib` filters, each with `--exact --test-threads=1`:
   - `jobs::tests::first_use_policy_never_installs_for_comparison_or_resume`
   - `jobs::tests::setup_progress_reports_real_bytes_and_only_actual_phases`
   - `jobs::tests::readiness_and_missing_trust_first_use_use_the_same_owner_request`
   — **1 passed each**, 0 failed, 22 filtered out each; no older Jobs tests run.
3. New optional external public-API target:
   `cargo test --locked --offline -p regressee-app --test automatic_runtime_setup_independent -- --ignored --test-threads=1`
   — final version **5 passed**, 0 failed, 0 filtered out; execution 0.47s,
   compile 0.64s. The first bounded four-probe version also passed before adding
   the byte-transfer probe and no-baseline auth checks; no implementation changed.

Test/compile command time was well below the five-minute budget; no timeout was
exhausted. Source formatting was restricted to the new verifier file. No GPUI
feature or project link-library override was needed.

The new target is Linux x86-64-only and ignored by default. Re-run explicitly
with `REGRESSEE_INDEPENDENT_ROOT` pointing at a new physically safe 0700 directory
and `TMPDIR` inside it. For the one local transfer probe, clear upper/lower-case
`HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, and set `NO_PROXY`/`no_proxy` to `*` in
the **test child process only**. The verification runner did this; no global
environment, user configuration or installed trust was changed.

## Independently exercised public APIs

The optional target imports ordinary public library APIs only: no `include!` of
backend internals, private callback access, implementation edits or worker hooks.

- `Product::readiness`, `status`, `ensure_runtime`: unavailable/malformed trust
  stays unavailable with safe copy and no cache creation; valid absent state is
  NotInstalled with development scope, not Ready; pre-cancel refuses without
  setup progress. Invalid actor configuration returns typed
  `Reply::RuntimeReadiness` rather than a fabricated environment.
- `distribution::current_absent` and public Product ensure: regular malformed
  `current`, missing generation, dangling pointer and linked cache all refuse
  automatic replacement. Valid signed **inert** generation is reverified and
  reused without a downloader. Modified READY, modified payload and authentically
  re-signed expired public-support manifest return RepairRequired and preserve
  the damaged bytes/pointer rather than silently repairing.
- Actual `Jobs::start_with_distribution`/submit/reply routing: comparisons and
  both resume forms do not enter automatic setup; ExistingInventory succeeds
  without a runtime. Login/probe with an active baseline do not request setup;
  those same actions on a fresh no-baseline project do request the prerequisite.
  Explicit discovery/new legacy capture/new inventory capture request it even
  with an existing baseline. A deliberately invalid **private** network-settings
  file stops eligible setup before any downloader/worker, making routing visible
  without installing anything. Missing project fails before that prerequisite.
- Every observed actor reply retains its original request ID. Five undrained
  status terminals fill the four-slot reply channel and block the fifth publisher;
  discovery is then queued and cancelled before draining. Its terminal remains
  request 25 and reports cancellation: no second Setup request, fake setup progress,
  runtime directory or download connection occurs.
- Public `install_with_progress` on independently authored signed inert archives
  cancels at Verifying, Extracting, TestingBrowser and Ready. Pre-publication
  cancellation preserves the old pointer. Cancellation at Ready can retain a new
  valid generation/pointer, but still returns cancellation; the old generation
  remains verifiable. Smoke is explicitly `Ok(())`, never a worker launch.
- A separate private loopback HTTP fixture supplies **1536 inert non-archive bytes**
  to public `setup_network_with_progress`. The callback sees initial 0/unknown,
  declared denominator 1536, then actual positive received bytes ≤1536. It cancels
  on received bytes: all events remain Downloading, no install/smoke/current pointer
  occurs, and the partial temporary file is removed. This is the sole intentional
  fixture connection, not an external/runtime download or successful installation.

The first four probes' loopback sentinel sockets observe **zero accepted download
connections**. Readiness/status do not load network settings or construct a client;
this is both an exercised observation and an inspected call-path property.

## Evidence preservation and progress scope

An independently constructed 1×1 PNG and synthetic completed run were saved through
public Store/image APIs, activated as an old baseline, and snapshotted. After guarded
Jobs routing/queued cancellation, inert installer phase cancellation and the local
download cancellation, the old project revision/config/active baseline, baseline
reference records, run/evidence records, original/derived CAS addresses, byte lengths
and independently recomputed file SHA-256 hashes compare unchanged. Store loading
also re-verifies the referenced artifacts. This is actual isolated SQLite/CAS state,
**not** a real captured website, authentication session or screenshot qualification.

Jobs byte conversion is independently tested through the maintained targeted test:
1024/4096 stays 1024/4096, unknown total stays indeterminate, extreme u64 counters
share a safe common unit scale, and non-download phases have no denominator. Source
inspection additionally confirms zero denominators become `None`, per-chunk bytes
come from actual response chunks under the 2 GiB cap, and phase labels correspond
to real installer transitions rather than fake 0/1 completion percentages. Progress
is best-effort on the existing bounded channel; terminal publication is reliable
under the existing stop/backpressure rules. Parent UI must clear setup-phase units
when the continuation switches to page/target counts.

Queued cancellation is exercised through actual public Jobs. During/after-publication
cancellation is exercised through the public inert installer and maintained private
Product orchestration tests. **No end-to-end public Jobs download → actual worker
smoke → website continuation is claimed.** Successful comparisons/resumes, login
sessions and capture identity compatibility are not exercised: these guarded
requests stop before worker launch. The inspected continuation checks remain intact.
Low-level Product discover/auth/capture APIs remain resolve-only; the UI Jobs owner
supplies automatic first-use setup. The maintained injected test also checks a
successful installer returning as cancellation arrives cannot continue.

## Unqualified boundaries and cleanup

No public default catalog/hosting/key/artifact or end-user trust configuration was
fabricated. Missing trust is still **an incomplete build**, so usable default
automatic provisioning remains unfulfilled. `setup_available` means validated
configuration plus adapter eligibility, not release approval or public-site
permission. Typed readiness must not be replaced by an inferred bool/environment.
A signature and even a test `QualifiedPublic` label are not owner/release/platform
qualification. Real authorized origins, support/signature policy, sandbox and
runtime trust remain external requirements.

TLS verification and no-redirect policy were inspected in unchanged network code,
not qualified by the plain-HTTP loopback fixture. DevelopmentLoopback public-site
guards and pinned baseline/resume identity checks remain in continuations. Worker
source uses manifest-root Node, not system Node; it was not launched. Linux/Windows
x86-64 eligibility and Mac/extra-architecture refusal have a maintained logic test;
macOS actual status/first-use test is target-gated and **unrun here**. The existing
macOS owned-process-tree capture refusal is still present. All-three-OS/native,
browser/runtime, origin/security and release tuples remain OPEN.

Physical private root:
`/path/to/private-tests/opencode/automatic-runtime-independent-20261010-30pdwxfw`
(0700). All temporary app roots, fixture SQLite/CAS/runtime archives/generations and
partial transfers were TempDir-owned and removed; actor threads were stopped/joined,
the one fixture server joined and sockets dropped. Only before/after source manifests
and four text execution logs remain in the private root for parent inspection.
The first Product result is recorded in this session's tool output rather than a
separate retained log. No user database, capture, login session, installed runtime,
global environment or native-agent-owned inspection artifacts were touched.
