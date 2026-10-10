# Phase 3 — independent bounded correction reverification

Date: 2026-10-09. Fresh independent GPT-6.1 Sol High verifier; **Phase 3 only**.

## Verdict

**The original two P1 and one P2 findings are fixed locally. Correction acceptance
remains OPEN because a new P2 resource-loading regression is independently
reproduced.** The corrected document boundary prevents forbidden HTTP visits;
robots and stable visible selection pass. However, otherwise allowed cross-origin
loopback script resources stall under that boundary and make real captures time
out. This is an actual local defect, not an unrun public/native qualification gate.

**Phase 4 must not rely on the combined corrected capture/navigation/resource
boundary as locally accepted until a bounded resource correction and independent
recheck.** Retain the proven identity, robots, selection, immutable-inventory,
storage and queue evidence; no rewrite or new 100×3 run is needed. Decisions
0005/both 0006 permit continued independent local work and a narrow corrective
pass, not a blanket stop for missing external qualification.

Evidence root (`E`): `/tmp/opencode/regressee-p3-reverify-3f6f750926eb/`.

Read the original verification/correction reports, decisions 0005/both 0006/0008,
actual worker interception/discovery/protocol, Rust crawl policy/robots/scheduler,
GPUI actions/update handling and immutable storage code/tests. No implementation
edits, nested agents, Phase 4 implementation, public browsing, native retry,
installation, sandbox/security bypass, publication or production service.
The independent analysis fixture author's `tests/fixtures/analysis-proposal/`
and `docs/research/analysis-corpus.md` were neither touched nor duplicated.

## Original repairs: independently confirmed

### P1 HTTP redirect visitation — fixed for the exercised document boundary

- Fresh maintained origins: **127.0.0.1:36729** and **127.0.0.1:33109**.
  `/outside`, `/scope/private`, `/scope/wild/secret`, `/scope/delete`,
  `/scope/skipped`, `/scope/file.pdf`, `/scope/poison` and the unapproved second
  origin's `/identity-provider` each receive **ZERO HTTP requests**, including
  discovery, redirected fixed-target capture and redirected baseline check.
  Neutral action/text redirect sources really receive requests.
  A separately explicitly approved second origin receives one robots request and
  one `/scope/cross-allowed` request, with an included page and truthful alias;
  approval is never inferred from Location.
- Fresh independent origins: **127.0.0.1:39829** and **127.0.0.1:43591**.
  A two-hop allowed-source chain ending at forbidden `/other/forbidden` receives
  zero destination requests; a separate two-hop chain ending at the unapproved
  second origin's `/unapproved` also receives zero. Robots-excluded redirected
  query/path destinations receive zero. These are server counters before any
  destination visit, **not a final-URL-only assertion**.
- A four-hop changing-path chain exercises 302/307/308/303; another chain changes
  meaningful query `n=0…6`. Every intermediate address is requested once during
  discovery, with truthful next-hop aliases; the final rendered child is discovered.
  Two-address cycles terminate at one request per address; maintained self-loops
  also terminate. This is finite-chain coverage, not a fresh 10,000-hop stress test.
- Both independent final pages then capture successfully: exact ordered query
  `/scope/final?b=2&a=1&a=0` and `/scope/change?n=6`, **1440×900** PNGs, requested
  URL equals final URL in metadata and target records, exact frozen inventory.
  Independent SHA-256/size, PNG chunk CRC, EOF and decompressed-scanline checks
  pass for both originals. Discovery aliases are not mislabeled as captured URLs.
- A list-only *allowed* alias fails capture once with `RedirectOutsideScope`, no
  screenshot; its first allowed destination is not revisited. Maintained redirected
  checks produce `CouldNotCapture` and preserve the active baseline.
- Actual automatic script navigation (including delayed replacement), cross-origin
  assignment, GET/POST forms, popup, iframe, `srcdoc` parent navigation, hash change
  and HTTP Refresh cannot publish capture evidence. Forbidden destinations and
  the same-address automatic POST receive zero requests. Maintained meta refresh
  fails with `CaptureFailed`; no stronger typed-meta diagnostic is claimed.
- Code inspection agrees with actual counts: main document requests use
  `route.fetch({maxRedirects:0})`, never `continue`; HTTP Location is withheld from
  Chromium. Rust checks each proposed hop before explicitly navigating it.
  The resource exception below prevents a blanket adapter acceptance claim.

Evidence: `E/maintained/product-flow/{preview.json,policy-report.json}`,
`E/maintained-summary.json`, `E/independent-boundary-b/` (preview, runs, report and
per-port request snapshots), `E/independent-two-final-artifact-audit.json`.

### P1 terminal robots wildcard/end anchor — fixed

The original `/scope/wild/*$` rule excludes the intended descendant with the
specific **`Excluded by robots.txt`** reason and zero actual requests, not a generic
capture failure. Fresh maintained and independent fixtures both establish this.

The independent probe passes **15,264 cases**: explicit wildcard/anchor/query,
more-specific Allow and nested Disallow, equal-specificity Allow tie, reserved
`%2F` versus decoded separator and literal `%2A` versus wildcard cases, plus an
exhaustive short-pattern differential check against a separate recursive matcher.
Actual discovery excludes encoded/query descendants while visiting their intended
allowed counterparts. Maintained tests add punctuation literals, repeated stars
and wildcard backtracking. Complete RFC percent-encoding equivalence is not claimed.

### P2 filtered preview selection — fixed, including controlled async updates

The exact maintained simulated-GPUI action regression passes. An independent test
uses an exact snapshot of the current shell plus appended tests; production code
is unmodified. It exercises actual control activation while changing filters and
inventory order, same-request progress, busy mutation guards, stale terminal
updates, matching completion, zero matches, old-row callbacks and replacement
inventory IDs. Selection remains a visible stable ID, not a raw index. Matching
discovery completion deliberately resets the active search and selects the first
visible new ID; this is tested, not assumed to retain the previous filter.

Real SQLite freeze/load preserves the final ordered identities exactly: root,
hidden and match/a included; **only visible match/b excluded**. Re-freeze with
changed order is refused. This is GPUI action/state plus actual storage evidence,
not native rendering, input or usability qualification.

Evidence: `E/maintained-gpui-action.log`, `E/independent-gpui-async.log`,
`E/probe/source-snapshot.json`, `E/gpui-store/frozen-inventory.json`.

## New material finding: P2 allowed cross-origin resources stall

**Boundary:** `worker/product/src/capture.mjs:18–45`, especially document
fetch/fulfill versus non-navigation `route.continue()` at line 28;
capture waits for DOMContentLoaded at line 76. Discovery uses this same boundary.

1. The initial independent real **Product/Worker** capture of `/scope/resources`
   includes an ordinary script resource redirect to another owned literal-loopback
   origin. It returns **`TargetOutcome::Failed { code: Timeout }`**, no capture,
   despite policy allowing the resource. The positive-resource assertion fails;
   that run is retained, not relabeled passed.
2. A separate concurrent Node HTTP fixture using the **actual signed capture
   adapter** reproduces a **30,000 ms DOMContentLoaded timeout**. It removes the
   initial Rust server's serial/idle-socket timing as an explanation.
3. Fresh minimal controls on **127.0.0.1:43683 → 127.0.0.1:36825** isolate the
   failure. The same cross-origin redirected script loads and executes in both
   sandboxed raw Chromium and ordinary-route-continue controls (**19 ms each**).
   Under the corrected boundary, same-origin direct and redirected scripts work
   (**20 / 16 ms**), but allowed cross-origin direct and redirected scripts stall
   (**5,002 / 5,001 ms**, bounded diagnostic timeouts).
4. Actual request/response/route traces show the document fetched/fulfilled, the
   permitted script continued, and, for the redirect case, its 302 response.
   There is no additional final-script HTTP request in either failing corrected
   case: the second server receives exactly **two** requests, both from positive
   controls. `blocked=false`, `redirect=null`; this is not intentional document
   scope rejection. Browser internals/permission-layer root cause is **not proven**.

This is a correctness regression for otherwise permitted pages with cross-origin
resources; no unsafe screenshot activation or public request is observed. The
signed-adapter controls use the same sandboxed launch and do not grant permissions,
disable security flags or change product code. Do not repair it by reverting to
automatic document following, weakening auth/resource policy, ignoring failures,
or disabling browser protections.

The existing **direct** resource guard still rejects a denied numeric OS-loopback
IPv4-mapped IPv6 alias of an owned server, producing zero destination requests and
a truthful `ResourceFailed` warning on a stable capture. Its redirected-resource
case times out, so that experiment is **not** a blanket resource-redirect safety
pass. All endpoints remain owned numeric OS-loopback; no DNS/public browsing.
Actual 401/403 main-page outcomes remain `AccessDenied`, and a valid non-null
session reference is refused `InvalidRequest` **before any HTTP visit**, without
unauthenticated fallback. These passes do not qualify Phase 4 session behavior.

Evidence: `E/independent-boundary/automatic-capture.json`,
`E/independent-real-boundary.log`, `E/resource-policy.log`,
`E/resource-policy-command.json`, `E/resource-policy/` metadata and counters,
`E/resource-diagnostic{,-b}/report.json`, corresponding logs/commands/scripts.
Both diagnostic processes exit **1** on the required positive-resource assertions.

## Exact serial execution

All build/worker/browser suites were serial: locked dependencies, four build jobs,
one Rust test thread. Independent fixture/report preparation and read-only artifact
inspection overlapped tests, not another suite. Historical Node **22.20.0**,
Playwright **1.56.1**, Chromium **141.0.7390.37**, fresh signed worker **3.0.0**;
owned loopback fixtures only, sandbox enabled. `E` expands to the root above.

| Actual command/scope | Result |
| --- | --- |
| `REGRESSEE_WORKER_SCRATCH_PARENT=/tmp/opencode/p3rv-69fea578 python3 scripts/check-phase3.py --policy-corrections-only --output E/maintained` | **Exit 0**: fmt 0.262 s; locked UI build/workspace tests/clippy 7.866 s, **70 Rust passes / 8 ordinary-batch ignored**; **4 Node protocol passes**, 0.262 s; exact real-browser `actual_phase3_policy_corrections` 322.634 s, including **8 signed-worker Node passes** |
| `TMPDIR=/tmp/opencode LIBRARY_PATH=$PWD/.runtime/link-libs RUST_TEST_THREADS=1 CARGO_BUILD_JOBS=4 cargo test --locked -p regressee-app --features ui-tests --bin regressee tests::filtered_preview_actions_use_visible_ids_and_freeze_exact_selection -- --exact --test-threads=1` | **1 PASS**, exit 0, 0.262 s wall / 0.03 s test |
| Independent `cargo metadata --offline --filter-platform x86_64-unknown-linux-gnu --format-version 1`; locked/offline probe builds | **PASS**, zero registry versions outside product lock; no dependency changes/downloads |
| `cargo test --locked --offline --features ui-tests --bin gpui-reverifier independent_async_preview::independent_reordered_async_visible_id_and_frozen_inventory -- --exact --test-threads=1 --nocapture` in `E/probe` | **1 PASS**, exit 0, 1.878 s wall / 0.03 s test |
| `E/run-independent.py` initial real boundary probe | **Exit 1**, 208.954 s: positive resource capture fails with Timeout; earlier chain/zero-visit/robots assertions passed; later session assertion not reached |
| `E/run-followup.py`, fresh `independent-boundary-b` root/origins | **Exit 0**, 122.933 s probe: all listed non-resource assertions and session refusal pass; resource positive case explicitly omitted and separately retained as OPEN, not weakened into a pass |
| `REGRESSEE_WORKER_SCRATCH_PARENT=/tmp/opencode/p3rv-69fea578 python3 scripts/check-phase3.py --runtime-only --active-cancel-only --retained-flow E/maintained/product-flow --output E/cancel` | **Exit 0**, 106.334 s actual suite: startup pause/resume, 600×300 capture, failed-analysis rejection, cancellation after **one observed slow-page visit**, return **0.604032834 s**, baseline preserved; not native latency |
| Signed Node `E/resource-policy.mjs`, `resource-diagnostic.mjs`, `resource-diagnostic-b.mjs` | **FAIL**, exit 1; respective command/log roots retained. Latest six-case controls finish in **10.666 s** and isolate cross-origin resource stalls |

Exact commands, environments and failure chronology are in `E/run-*.py`,
`E/{independent,followup}-commands.json`, `E/{maintained,cancel}/commands.json`
and resource command records. The first unfiltered offline metadata attempt exits
101 for uncached `arbitrary 1.5.0`; Linux-filtered resolution succeeds. No install
or download was attempted. The initial Rust fixture's idle-socket read bound was
reduced only in the fresh verifier follow-up; independent Node controls establish
the resource failure without that fixture. All failed roots/logs remain intact.
The existing upstream `proc-macro-error2 2.0.1` future-Rust warning remains.

## Preservation, pending gates and handoff

Before report/index writes, **104 protected source/document/lock inputs were
byte-equal** to entry. All four signed worker source modules equal maintained
source. The only repository source/document edits are this new report and
`docs/IMPLEMENTATION_STATUS.md`; its exact prior version is
`E/IMPLEMENTATION_STATUS.before.md`. Build outputs remain ordinary retained caches.
Original verification/correction reports and evidence remain preserved, with no
recursive historical snapshot or wholesale historical byte-audit claim.

Final scoped audit (`E/final-audit.json`): zero owned live processes/zombies, zero
new browser-like zombies, empty own/default private scratch, zero pending fresh-root
CAS publications or install/download temporaries; zero new registry versions.
Fixtures stop/join; browser contexts/processes close. Only the validated empty own
short scratch parent is removed with non-recursive `rmdir`. No unrelated process
is signalled; fresh logs, screenshots and probes remain retained.

**NOT RERUN here:** independent 100×3 scale/storage/queue/migration corpus or native
black-window attempts. Prior reports are historical retained evidence, not these
tests. **Pending/NOT RUN:** 1,000×3 soak; native rendering/input/resize/clipboard/IME/
accessibility/latency/human approval; Windows/macOS ownership and clean supported
machines; trusted public runtime, signing/hosting/security migration and release
qualification; exhausted-disk/power-loss/concurrent-writer qualification.

**Next actor:** bounded Phase 3 resource correction, preserving the now-proven
pre-follow document safety boundary, then fresh independent positive-resource and
zero-forbidden-visit regressions. Carry pending external/native/soak gates forward;
the reason dependent Phase 4 still waits is this reproduced local P2, not those
unrun gates. No Phase 4 or further agent was started by this verifier.
