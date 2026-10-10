# Phase 3 — independent resource and retained-boundary verification

Date: 2026-10-09. Fresh independent GPT-6.1 Sol High verifier; **Phase 3 only**.

## Verdict

**PASS locally: the resource-loading P2 is repaired, and the combined document /
resource boundary is independently accepted for the exercised pinned Linux
runtime. The original two P1 and one P2 corrections remain intact.** No material
local safety/correctness blocker was found in this focused recheck.

**Phase 4 may rely on this corrected crawler for authorized local continuation**
under decision 0005 and both 0006 decisions. This does not implement or qualify
login, saved sessions, public-site access or release security. Phase 4 was not
started. Native/OS/public/soak gates remain pending, not an automatic stop of local
development. Resource policy tests are **not public-release qualification**.

Evidence root (`E`): `/tmp/opencode/regressee-p3-resource-rv-e85a53c1ef12/`.

Read `03-reverification.md` including NewP2, `03-resource-corrections.md`, original
correction assertions, decisions 0005/both 0006, actual worker capture/discovery/
CDP/interception/IPC, Rust crawl policy and focused browser tests, robots matching,
stable visible-ID GPUI actions and freeze assertions. No implementation changes,
nested agents, real-site browsing, installation, sandbox/security-flag changes,
global host-policy changes, native retry or unrelated scale/soak audit.

## Fresh runtime authenticity, not old report metadata

Both focused commands create and install **new ephemerally signed development
packs** through the real resolver. `E/signed-pack-audit.json` independently verifies
both Ed25519 signatures against their fresh distribution keys and checks actual
installed Node/Chromium executables, Playwright package metadata, all four worker
modules and all three maintained test files against signed manifest hashes. Worker
and test bytes also equal current maintained source. Actual launch/IPC assertions
check Node **22.20.0**, Playwright **1.56.1**, Chromium **141.0.7390.37**, worker
**3.0.0**; scope remains **DevelopmentLoopback**, sandbox enabled.

The historical `.runtime/installed` fixture worker is only the verified runtime
byte source for packaging, **not** the worker tested as product version 3. The
fresh pack paths are recorded explicitly in the audit and exact command records.
No dependency, lock, browser protection or source change was used to get a pass.

## Resource repair: positive behavior and retained safety

### Maintained signed suite, independently inspected

Fresh concurrent origins **127.0.0.1:35615 / 127.0.0.1:36857**:

- Raw/ordinary-continue controls and five same-/cross-origin direct/redirect/chain
  variants pass. Scripts actually request and execute, computed CSS applies,
  PNGs decode with natural dimensions. Native document address-space extra-info
  remains **Loopback**, not the old fetch/fulfill **Unknown** classification.
- Actual capture and rendered-link discovery pass, with zero positive warnings,
  matching requested/final URLs and script-created links. Page and dedicated-worker
  fetches independently retain allowed and denied resource-redirect behavior.
- Direct, redirected and multi-hop script/CSS/image requests to the runtime-denied
  numeric OS-loopback IPv4-mapped IPv6 alias receive **ZERO destination hits**.
  Denied resources produce stable capture evidence with truthful `ResourceFailed`
  and `ImagesIncomplete`, not the former navigation timeout.
- Document redirects remain unvisited candidates; automatic navigation, child
  frames, popups, forms, reload and HTTP Refresh fail closed before destination
  HTTP visits. 401/403 are `AccessDenied`; valid non-null session references are
  `InvalidRequest` **before any visit**, with no unauthenticated fallback.
- Maintained pending-document IPC cancellation and native request/response-pause
  context closure pass; the latter take **8 / 10 ms**, contexts empty, subsequent
  resource capture succeeds and owned servers/browser close.

Evidence: `E/maintained-resources/product-flow/{worker-suite.log,resources/report.json,
resources/ipc/events.json}`, `E/maintained-resource-audit.json`. These conclusions
follow DOM assertions and actual server counters, not just the suite exit code.

### Fresh independent concurrent servers and input variants

Final independent origins **127.0.0.1:39073 / 127.0.0.1:40115**, with only their
runtime-denied numeric mapped-IPv6 alias as a negative resource target:

- Cross-origin direct script/CSS/PNG and **301/302/303/307/308** two-hop resource
  chains really request every intermediate and final URL. Six script tokens
  execute in order, six computed CSS values match, and all six images decode to
  **1×1**. Direct destination counts are JS **5**, CSS **4**, PNG **4** over the
  complete run. Extra-info retains Loopback with the browser permission policy
  unchanged. Resources outside document `/scope` remain allowed; page scope was
  not converted into a resource path/origin firewall.
- Cross-origin module/import redirects, CSS `@import` and redirected background
  images work. Actual discovery includes the module's exact ordered-query child.
  Positive captures have no resource/readiness warnings and preserve exact
  `/scope/matrix?b=2&a=1&a=0` requested/final provenance, **600×300** pixels.
- Direct and one-/two-hop denied JS/CSS/PNG destinations have **ZERO hits** while
  permitted redirect sources are actually requested. The denied-resource capture
  records **9 resource failures / 3 image failures** and the two truthful warnings.
- For each of the five redirect statuses, off-section and unapproved-origin
  document destinations have **ZERO hits**, in discovery and frozen-target capture.
  A separately explicitly followed five-hop cross-origin document chain exercises
  all five statuses: each next candidate is unvisited until explicit navigation;
  its final capture preserves the exact ordered-query requested/final URL. This
  adapter probe does not replace Rust policy approval; that is checked below.
- Immediate assignment, delayed replacement, meta refresh, GET/POST forms, popup,
  iframe, `srcdoc` parent navigation, hash transition and HTTP Refresh cannot
  publish screenshots. Forbidden destinations have **ZERO hits**; same-address
  POST adds zero requests (only the two initial GETs are counted). No timeout is
  accepted as a substitute for the expected fail-closed outcomes.
- Actual signed `src/index.mjs` IPC is exercised with a **verifier-only external
  scheduling preload** holding one native CDP continuation. Request pause holds
  the cross-origin script before network; response pause holds document headers.
  **Cancel / Shutdown** complete in **11 / 46 ms** at request pause and **9 / 40 ms**
  at response pause. Each pending capture returns `Cancelled`, emits no screenshot,
  and its held resource has zero hits. Both Cancel cases successfully recapture;
  all four workers exit **0**, without signal or hanging interception promises.
  The preload changes scheduling only, not installed bytes or policy/security.

Evidence: `E/independent-c/report.json` (**31 passed case records**), per-case
`ipc-*/events.json`, `E/independent-result-summary.json`, `E/followup-c-commands.json`.
Fresh-root artifact audit independently checks SHA-256/size, PNG chunk CRCs, EOF,
dimensions and decompressed scanline lengths for **20 retained originals**, including
earlier diagnostic runs; all pass (`E/artifact-audit.json`).

## Original three corrections: retained and rerun

- **P1 document visitation:** actual Rust-owned `actual_phase3_policy_corrections`
  passes on fresh **127.0.0.1:37639 / 127.0.0.1:40823**. `/outside`, private/wildcard/
  action/skipped/download/poison destinations and the unapproved identity-provider
  receive **ZERO requests** after discovery, capture and check. Neutral redirect
  sources are requested; finite allowed aliases and ordered queries work; self/
  two-address loops terminate. Explicit second-origin approval receives one robots
  and one page request. Frozen redirected targets/checks fail once with
  `RedirectOutsideScope`, no capture; check classification is `CouldNotCapture`
  and the active baseline is preserved (actual maintained assertions inspected).
- **P1 robots:** wildcard/end-anchor exact regression passes, including literals,
  backtracking, queries, precedence and reserved encoded paths. The real preview
  excludes `/scope/wild/secret` specifically as **Excluded by robots.txt** with zero
  visits, not merely capture failure. The prior 15,264 independent cases are retained,
  not newly rerun or claimed full RFC equivalence.
- **P2 selection:** exact simulated-GPUI action regression passes. Filter/toggle/
  previous/next/stale-row/zero-match/bulk actions operate on visible stable IDs;
  actual SQLite freeze/load preserves exactly the intended included IDs and refuses
  mutation of the frozen revision. This remains non-native actor/storage evidence.

Evidence: `E/retained-policy/product-flow/{preview.json,policy-report.json,
worker-suite.log}`, `E/retained-policy-audit.json`, exact robots/selection logs.

## Exact serial execution and diagnostic chronology

All build/worker/browser suites ran **serially**: locked Cargo, four build jobs,
one Rust test thread; maintained Node concurrency one. Read-only audits overlapped
tests, not another suite. `TMPDIR=/tmp/opencode`,
`LIBRARY_PATH=$PWD/.runtime/link-libs`, own product scratch
`REGRESSEE_WORKER_SCRATCH_PARENT=/tmp/opencode/prv-e85a53c1ef12`.

| Exact command/scope | Actual result |
| --- | --- |
| `python3 scripts/check-phase3.py --resource-corrections-only --output E/maintained-resources` | Exit **0**: fmt **0.271 s**; locked UI build/workspace/clippy **7.956 s**, **70 Rust passes / 9 ordinary-batch ignored**; **4 protocol passes**, **0.265 s**; exact `actual_phase3_resource_corrections` **1 pass**, **124.541 s**, including **17 signed-worker Node passes / zero failures**, **5.078 s** |
| `python3 scripts/check-phase3.py --runtime-only --policy-corrections-only --output E/retained-policy` | Exit **0**, exact `actual_phase3_policy_corrections` **1 pass**, **255.837 s**; **8 signed-worker Node passes** |
| `cargo test --locked -p regressee-core crawl::tests::robots_wildcards_anchors_literals_queries_and_precedence -- --exact --test-threads=1` | **1 PASS**, exit 0, **1.450 s** wall |
| `cargo test --locked -p regressee-app --features ui-tests --bin regressee tests::filtered_preview_actions_use_visible_ids_and_freeze_exact_selection -- --exact --test-threads=1` | **1 PASS**, exit 0, **0.305 s** wall / **0.03 s** test |
| `python3 E/run-independent.py` initial independent probe | Exit **1**, **4.021 s**: positive resource/denied/document-chain assertions pass; diagnostic rejects known raw meta-refresh `Execution context was destroyed` instead of accepting its fail-closed error |
| `python3 E/run-independent.py --followup` fresh independent-b | Exit **1**, **11.956 s**: preceding boundaries and request-pause IPC pass; verifier preload unintentionally holds the subsequent recapture's document response too, causing diagnostic event timeout |
| `python3 E/run-independent.py --followup-c` fresh independent-c | Exit **0**, **8.445 s**: all **31 case records** pass, including all four native-pause IPC Cancel/Shutdown cases |

The two diagnostic failures are **not passes** and all logs/counters/artifacts are
retained. `independent-a.mjs` preserves the first exact probe; `pause-preload-a.mjs`
preserves the repeated-per-context injector. The follow-up accepts only the known
meta-refresh raw error in addition to expected rejections, keeping zero-hit and
no-screenshot assertions. The preload then becomes once-per-worker, rather than
pausing the successful recapture again. No product code fix or test policy/security
bypass occurred. Final independent automatic cases all return
`RedirectOutsideScope`; the maintained meta case still honestly reports
`CaptureFailed`. The existing upstream `proc-macro-error2 2.0.1` future-Rust warning
remains. Exact commands/environments/process samples are in the command JSON files.

## Preservation, ownership and remaining gates

Before report/index writes, **123 protected source/document/lock inputs were
byte-equal** to entry. Only this report and `docs/IMPLEMENTATION_STATUS.md` are
repository edits; exact previous index retained at `E/IMPLEMENTATION_STATUS.before.md`.
Original reports/evidence and independent corpus-author files remain untouched.
There was no recursive historical snapshot or broad delete/signal operation.

`E/final-audit.json` records zero live owned product candidates, zero sampled owned
survivors, zero new Node/Chromium-like zombies, empty own/default private scratch,
and zero fresh-root pending CAS or install/download temporaries. Owned concurrent
servers stop and browsers disconnect. Only the validated empty own scratch parent
was removed using **non-recursive `rmdir`**. The independent Python runner uses
Linux subreaper ownership and reaps **40 adopted verifier descendants** across the
three runs; it never signals/reaps an unrelated parent-harness process.

The entry table already contained **143 zombies, including 32 browser-like**,
which are inherited, not live product candidates from these tests. Additional
git/sh zombies belong to `piforge`/`pi` harness parents; IDs and parent commands are
recorded in `E/{final-audit,harness-zombie-classification}.json`. They were not
misattributed to the tested worker or unsuccessfully targeted for cleanup. No
globally clean process-table claim is made.

The final ownership snapshot is serial, after the read-only Node signature audit
exits. An intentionally overlapping audit assertion is retained in
`E/audit-chronology.json`; no assertion/classifier was weakened or process killed
to obtain the subsequent clean owned-resource snapshot.

**Not rerun:** full 100×3 storage/queue/scale corpus or native/OS audits; their prior
independent evidence remains retained. **Pending / NOT RUN:** 1,000×3 soak; native
rendering/input/resize/clipboard/IME/accessibility/responsiveness/human approval;
Windows/macOS ownership and clean supported machines; qualified current public
runtime, signing/hosting/security and baseline migration/release qualification;
exhausted-disk/power-loss/concurrent-writer qualification. These remain genuine
gates without undoing this bounded local boundary acceptance. The next actor may
start **local Phase 4 implementation**, not public release or historical-runtime
real-site browsing. This verifier starts neither.
