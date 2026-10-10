# Phase 2 — independent product-loop verification

Date: 2026-10-09. Fresh independent GPT-6.1 Sol verifier; **Phase 2 only**.

## Verdict

**The fixed-scope Linux/short-temporary-root product loop passes. One P1 local
worker-launch robustness repair is required before Phase 3 relies on the
supervisor without that restriction.** This is not full Phase 2, arbitrary-public
preview, native/platform or release acceptance. No implementation was edited,
agents spawned, or Phase 3 work started.

This is a reproduced local failure, not a stop based on unrun external checks.
Decision 0005 still permits local continuation with qualification honestly open;
repair and independently reverify the narrow supervisor issue before dependent
work assumes general worker startup. The verified storage, capture, analysis and
viewer paths can otherwise be retained; no rewrite or historical evidence
deletion is warranted.

Read PRODUCT/PLAN, Phase 2 checklist/status, original foundation verification and
reverification, decisions 0005, both 0006 documents and 0007, runtime interface
documentation, and the completed read-only packaging research. Inspected actual
Rust product/runtime/protocol/storage/image/GPUI code, JavaScript product worker,
and maintained test assertions. Research metadata was not treated as qualified
runtime pins.

Fresh retained evidence root (`E` throughout):

`/tmp/opencode/regressee-phase2-verification-20261009-7e91fe70/`

It contains serial command records/logs, independent probe source/lock, signed
fixture packs, actual SQLite/CAS evidence, negative-fixture results, diagnostic
logs, preservation hashes and the prior pipeline-index snapshot. Independent
real data is under `E/independent-app-b/data`; its runtime cache is the fresh
verifier-owned `E/handoff/product-flow/app/cache`, not the implementer's history.

## Reproduced issue

### P1 — inherited long `TMPDIR` breaks the supposedly short Linux worker scratch

- **Location:** `crates/runtime/src/supervisor.rs:77–100,378–417`;
  `worker/product/src/capture.mjs:13`.
- `Worker::spawn` describes its scratch as short but uses
  `tempfile::Builder::tempdir()`, inheriting the full caller `TMPDIR`. It then
  supplies that longer scratch to Playwright/Chromium as TMPDIR/TMP/TEMP.
  There is no length policy/fallback. Recovery also scans the inherited parent.
- **Actual reproduction:** run the unchanged
  `retained_runtime_relative_supervision_smoke` with
  `TMPDIR=E/scratch`, the existing linker path, and the handoff's retained-flow
  path. Signature/inventory verification succeeds, but the real worker handshake
  fails with `worker failure: CaptureFailed` (exit 101, 17.140 s).
  The same command, same signed generation and current source passes with the
  unique short parent `/tmp/opencode/rv2-7e91fe70` (exit 0, 16.739 s).
- **One bounded diagnostic:** the actual sandbox-enabled product `launch`
  adapter under `E/scratch/rg2-diagnostic` fails before its owned `setContent`
  screenshot. Chromium PID 1322653 exits with **SIGTRAP**; Playwright closes/reaps
  it and cleans its temporary directories. No public URL, native window,
  sandbox workaround or implementation modification was involved.
- **Cause certainty:** the long/short environment-dependent launch failure is
  reproduced. The Unix socket-path limit is consistent with the Phase 0 issue
  and its explicit short-parent fix in `crates/spike/src/supervisor.rs:196–201`,
  but this diagnostic did **not** capture a fatal socket-limit message. The exact
  Chromium SIGTRAP cause is not claimed proven.
- **Impact:** a valid inherited temporary-directory layout can prevent private
  browser setup/capture despite an intact runtime. Re-downloading the same pack
  does not resolve this environment-dependent failure. No false unchanged result,
  lost baseline or leaked process was observed.
- **Required bounded repair:** establish an actually short, private owned Linux
  scratch-parent policy (with explicit validation/diagnostics), and make recovery
  use the same ownership namespace/parent. Independently recheck long inherited
  TMPDIR, ordinary short TMPDIR, relative runtime roots, cancellation/crash
  cleanup, and preservation of existing evidence. Do not fix this by disabling
  the sandbox, moving/deleting evidence or changing the browser silently.
- **Evidence:** `E/independent-01.log`, `E/short-root-00.log`,
  `E/long-temp-diagnostic.{py,mjs,json,log}` and `E/final-audit.json`.

## Actual serial commands

Working directory: `/path/to/Regressee`. No build/browser/test
commands overlapped. Background execution of a serial runner allowed source
reading only, not concurrent tests. Build jobs were bounded to four; tests to
one thread. The handoff wrapper's formatting command made **no source changes**.

| Command/scope | Actual result |
| --- | --- |
| `python3 scripts/check-phase2.py --output E/handoff` | **Exit 0**; all five constituent commands below executed serially |
| ↳ `cargo fmt --all` | Exit 0, 0.103 s; preservation hashes unchanged |
| ↳ `bash scripts/check-foundation.sh` | Exit 0, 5.608 s: locked UI-inclusive binaries, format check, **57 Rust tests**, all-target/all-feature clippy `-D warnings` |
| ↳ private Node `--test worker/product/test/protocol.test.mjs` | Exit 0, 0.047 s; **4 Node tests** |
| ↳ locked `actual_signed_http_product_flow --ignored --nocapture --test-threads=1` | **Exit 0**, 349.328 s; current-source real signed HTTP/product/browser integration; installed private Node suite **6 tests** |
| ↳ locked `regressee-foundation -- E/handoff/foundation-reopen` | Exit 0, 0.855 s; schema 3 and real settings/artifact reopening; text object remains a synthetic storage fixture |
| `bash scripts/build-ui.sh` | Exit 0, 0.264 s; explicit additional handoff binary build |
| Retained relative-root supervisor smoke with long TMPDIR | **Exit 101**, reproduced P1 above; not hidden or counted as a security pass |
| Same retained smoke with unique short TMPDIR | **Exit 0**, 16.739 s; actual signed worker/sandbox-enabled screenshot/shutdown |
| Independent locked/offline path-dependency `probe -- flow` | **Exit 0**, 113.092 s; fresh real tall-site actor/product/storage/viewer flow |
| `probe -- trust` | Exit 0, 0.464 s; exact-cause trust/hash/deadline/repair/header-cancellation probes |
| `python3 E/archive-cases.py`, then `probe -- archives` | Exit 0 / 0.032 s and exit 0 / 0.264 s; six independently constructed raw ZIP cases |
| `probe -- wire` | Exit 0, 0.264 s; exact acknowledgement/correlation/token/path/hash boundary assertions |
| Locked/offline included actual shell, `verifier_ --nocapture --test-threads=1` | **1 fresh GPUI test PASS**, exit 0, 5.823 s; three included maintained tests filtered out, already exercised by the locked workspace run |
| `probe -- artifacts` | Exit 0, 71.084 s; accepted synthetic control plus **eight single-fault worker-output rejections**, not browser/site qualification |
| `python3 E/long-temp-diagnostic.py` | Driver exit 0 records actual Node/browser **exit 1**, 0.715 s; diagnostic is a reproduced failure, not a screenshot pass |
| `python3 E/audit.py` | Exit 0 before report/index writes; no changed pre-existing source/docs, live owned candidates, scratch, pending stages or new registry dependency versions |

Full command arrays, timing and logs are in `handoff/commands.json`,
`independent-commands.json`, `short-root-commands.json`, `bounded-commands.json`.
An initial independent probe stopped because my helper incorrectly treated a
five-second quiet receive interval during full runtime hashing as its overall
deadline. That is **a verifier-harness error**, not a product failure. The helper
now retries receive timeouts within a 75-second deadline; its corrected run uses
a fresh `independent-app-b` root. The failed log and initial data root remain
preserved. No implementation or old report was changed to make tests pass.

## Behavioral evidence

### Real project → explicit baseline → separate unchanged/changed checks

The maintained latest-source suite actually downloads **304,884,626 bytes** from
its owned signed HTTP fixture, verifies/installs the product worker/runtime,
smoke-tests a sandbox-enabled browser and operates actual actor/services/storage.
It asserts no active baseline before explicit activation, separate run IDs,
unchanged **0 / 1,296,000** pixels, changed **6,400 / 1,296,000**, an actual saved
heatmap, failed same-target check classified `CouldNotCapture`, 20,000-pixel
original/40 tiles, reopening, cancel and real worker/app SIGKILL paths.

My independent fixture serves a different **5,000-pixel** page with a 37×29
change near y=4200. Real actor capture leaves the project unactivated; explicit
actor Activate establishes its baseline. Separate checks preserve the exact
inventory/profile and produce **0** then **1,073 / 7,200,000** changed pixels.
Original, thumbnail, ten viewer tiles and difference evidence are stored in the
actual CAS. Completed baseline/run JSON remains byte-for-byte unchanged after
checks, failure and cancellation; all retained originals/derivatives verify on
reopening. Ordinary project edits cannot smuggle activation or change the frozen
address, and re-finishing a terminal run is refused.

An actual same-target **403** becomes `AccessDenied` / `CouldNotCapture`, with no
capture and no activation. The maintained suite separately exercises 404 and
503. Failed, comparison, cancelled and interrupted captures cannot activate a
baseline. Cancellation at the analysis boundary retains completed screenshot
evidence with **no analysis**, keeps the prior active baseline, and is not called
unchanged. This is the documented pending-analysis case, not Phase 3 resume.

Evidence: `handoff/product-flow/flow-report.json`, `independent-real-flow.json`,
`probe/src/main.rs` and `bounded-00.log`.

### Actual saved viewer images; analysis/background boundaries

The real Jobs Viewer action returns before/after/diff tiles at index 8, at most
two per pane. I decoded the hash-verified saved PNG tiles independently, converted
RGBA→BGRA and compared **every byte** with the returned GPUI RenderImage buffers
in all three panes. They are actual saved images, not placeholder/demo results.

A fresh simulated GPUI test includes the **unchanged actual `main.rs`**, opens
that persisted project through its real handler/actor, visits Results, uses zoom
and 33 PanDown actions to y=4224/tile 8, pans horizontally, and reopens evidence
to reset position and retrieve the same saved run. This proves actual actions and
saved buffers, **not native rendering/input/visual approval**.

Source and maintained assertions confirm Jobs' serial blocking owner performs
SQLite, download/process waits, row decode, diff and BGRA conversion, rather than
GPUI actions/render. Command/update queues each have four slots; UI submission is
nonblocking and polling drains at most four updates per 50 ms. Terminal replies
survive backpressure. Worker frames are 1 MiB with an eight-entry stdout queue;
stderr is drained without retaining site stacks. Image work uses bounded rows
and 512-row derivatives; originals are never giant GPUI textures. Retired sprite
atlas images are explicitly evicted in render. Real GPU memory/latency and atlas
eviction on a native backend remain unqualified.

### Trust, paths, hashes and repair — actual assertions, not generic failures

- Independent equal-length payload alteration reaches **`runtime checksum
  mismatch`**, wrong key **`runtime signature rejected`**, changed configured
  versus signed scope **`incompatible runtime manifest`**, and signed expired
  public support **`public runtime support expired or not qualified`**. No smoke
  callback executes and the prior pointer stays unchanged in each rejection.
- Equal-length installed damage yields **`runtime damaged; repair required`**.
  Repair selects a fresh generation and retains the old one. A held-header HTTP
  request cancels with exactly **`operation cancelled`** within two seconds;
  no extraction stage or pointer change results. Maintained tests also exercise
  mid-body cancellation, oversized/corrupt download, failed smoke and unsafe
  pointer/file layouts; the real flow explicitly rejects redirects and repairs
  a damaged worker file while retaining originals.
- Six independently generated raw ZIPs reject duplicate names, case collisions,
  parent traversal and reserved names with **`duplicate or unsafe ZIP entry`**;
  same-name symlink and FIFO entries reject **`unsafe runtime archive type`**.
  Smoke is never invoked, the pointer and cache entry count do not change.
- The maintained test called `checksum` changes payload **length**, so its actual
  assertion is `size mismatch`, not proof of a same-length SHA-256 failure. The
  independent equal-length case above closes that coverage distinction. Some
  maintained HTTP/error assertions check only failure/preservation; I do not
  count them as exact-cause security assertions.
- Independent wire probes assert acknowledgement ordering, foreign job
  correlation, nil/non-UUID output tokens, unknown injected paths and malformed
  canonical hashes. The live process adapter additionally rejects eight
  single-fault output variants: wrong final metadata URL, unknown metadata field,
  screenshot hash, symlink output, foreign target key, injected event artifact
  path, nil token and false decoded dimensions. A matching positive control
  verifies that unrelated setup/launch failure is not causing every rejection.
  Product deliberately exposes these as `CaptureFailed`; the report does not
  pretend it exposes eight distinct integrity failure codes.
- That adversarial worker is a **separate signed verifier fixture**, uses an
  inert browser entry and an explicitly no-op installation smoke callback, never
  browses, and its positive control reuses a saved PNG. It is labelled synthetic
  in source/results/projects, **never activated or supplied to the real UI
  test**, and is not evidence of real capture/runtime qualification. All eight
  negative records lack capture evidence and refuse activation.
- Current complete PNG/CRC/first-IEND-at-EOF gates were exercised by fresh real
  Chromium originals and maintained truncated/corrupt/concatenated/tall-image
  regressions. Capture metadata/profile/runtime/dimensions/request/final URL and
  content hashes are checked before a capture can become baseline evidence.

Evidence: `independent-trust.json`, `independent-archives.json`,
`independent-wire.json`, `independent-artifact-boundaries.json`, probe source and
`adversarial-worker.mjs`.

## What arbitrary targets and runtime gates actually permit

Project settings accept credential-free HTTP(S) URLs; that is **not public-site
capture qualification**. The actually exercised historical Node **22.20.0** /
Playwright **1.56.1** / Chromium **141.0.7390.37** pack is development-only.
Rust refuses non-literal-loopback captures before launching it. The product
worker also restricts external HTTP(S) resources/navigation and closes WebSockets
in that scope; localhost/DNS/private-network/public addresses are not silently
treated as literal loopback. Tests used only owned loopback fixtures or owned
in-memory screenshot smoke, never real-site browsing with that runtime.

Missing/invalid configured trust fails closed. QualifiedPublic requires a
configured public verification key, HTTPS download, matching signed scope and an
unexpired signed deadline. Worker identity must match the signed Node/worker/
Playwright/browser tuple; checks also refuse a different baseline runtime.
Product worker launch currently fails closed off Linux.

**There is no automatic numerical security-version floor or vulnerability
qualification.** An independently signed **inert** public-scope manifest using
historical version strings and a future deadline was accepted by the low-level
installer with a no-op fixture smoke; no executable or public browser was run.
Thus the authority is the externally configured signer/support policy, exactly
as decision 0007 says, not a local certification from metadata or a scope label.
No production trust/host or qualified public executable pack exists here.
`docs/research/release-qualification.md` offers current upstream metadata and
recommendations, not qualified replacement pins or a release pass.

## Qualification gaps, preservation and next actor

- Secure supported arbitrary-public runtime, upstream provenance/security review,
  signing/hosting/support and migration policy, automatic first-use on a clean
  supported machine: **pending**, not passed by locally signed fixture packs.
- Native GPUI rendering/input/screenshots, synchronized visual inspection,
  keyboard/mouse/IME/clipboard/dialog/resize, hardware/default Wayland, accessibility
  and human sleek/modern/usability approval: **NOT RUN here / still open**. Zero
  new native attempts: this source/backend verification offered no meaningful new
  condition for retrying the previous unmapped-window evidence. No virtual
  display, forced mapping, black-window loop or invented approval occurred.
- Windows/macOS owned-process adapters, runtime/bundle/ACL/durability behavior,
  clean supported Linux packaging/default sandbox policy and dispatched platform
  CI: **NOT RUN/pending**. Sandbox-enabled local launch is not full sandbox or
  distribution qualification.
- Continuous low-space handling, managed networks/custom CA, calibrated
  sensitivity, cross-rendering/fonts, representative performance and release
  notice/source audit remain pending in their planned scope. No crawl, auth,
  multi-page resume, masks/selective approvals or ZIP features are demanded as
  new Phase 2 blockers.
- `/usr/bin/time` reports the full product suite at **301,000 KiB maximum RSS**,
  332.80/7.72 s user/system CPU. These are command maxima, not independently
  sampled concurrent Rust/Node/Chromium totals or hardware/scale qualification.
- All previous source/docs/locks and research matched the pre-run hashes before
  the authorized report/index writes. The final audit permits only the pipeline
  index to differ among pre-existing files. Original reports/evidence were not
  overwritten or recursively copied. Probe dependencies resolve only registry
  tuples already in the product lock. Upstream `proc-macro-error2` future-Rust
  warnings remain, not new product lint failures.
- Scoped audit: **zero live workspace/verifier app/Node/browser candidates, zero
  remaining product scratch and zero pending publication/extraction stages**.
  No unowned process was killed. Only the explicitly created diagnostic scratch
  and empty verifier scratch parents were removed; packs, generations, failed
  attempt logs and immutable evidence remain retained.

**Next:** a separate bounded Phase 2 corrective pass for the P1 scratch-parent
failure, followed by fresh independent re-verification. Phase 3 must not assume
that issue repaired or this delivery fully accepted. After that local repair,
decision 0005 allows dependent local continuation while carrying all the external,
native, human and public-runtime gates above honestly forward.
