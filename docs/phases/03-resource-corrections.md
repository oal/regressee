# Phase 3 — bounded cross-origin resource correction

Date: 2026-10-09. GPT-6.1 Sol High implementer; **one Phase 3 regression only**.

## Local verdict

**The reproduced P2 is fixed in maintained source and passes fresh local signed-
runtime/browser/worker tests. Independent acceptance remains pending.** Permitted
cross-origin direct and redirected scripts now execute; CSS applies and images
decode. Forbidden document and runtime-denied resource destinations receive **zero
HTTP requests** in the maintained fixtures. Original policy/robots/visible-selection
checks still pass. Phase 4 was not started and should wait for the independent
combined-boundary recheck, not be represented as already authorized by this report.

Evidence root (`E`): `/tmp/opencode/regressee-p3-resource-fix-6aa3bf974bcd/`.
Original reports/evidence are preserved, including the independent failing resource
experiments in `03-reverification.md`. These are implementation results, not a
replacement independent verdict or public-site qualification.

## Actual cause and repair

Read the independent `resource-diagnostic-b.mjs` and `resource-policy.mjs`
variants before implementing. A fresh concurrent two-server probe compares raw
Chromium, the prior actual boundary, and the repaired boundary, with CDP Network
extra-info attached. All servers bind owned **127.0.0.1**, with distinct ephemeral
ports; there are no DNS/public requests or security-flag/permission changes.

The old document `route.fetch({maxRedirects:0})` / `route.fulfill()` path is **not
network-transparent**. Chromium reports subresource
`clientSecurityState.initiatorIPAddressSpace: "Unknown"` for the fulfilled
document, versus **"Loopback"** for both raw navigation and the repaired boundary.
The old cross-origin redirected script stalls until the 3,000 ms diagnostic bound;
raw/repaired cases execute it. The repaired document redirect is blocked with an
unvisited destination. Exact traces/counters: `E/network-probe.json`,
`E/network-probe-b.log`, `E/probe/{old-capture,capture,network}.mjs`.
This proves a changed Chromium address-space classification and the behavioral
regression; it does **not** claim a Chromium-internal permission-stack backtrace.

`worker/product/src/capture.mjs` now installs two complementary native guards:

1. A context-wide Playwright request route admits only the initial explicit GET
   to the exact primary main-frame target (and section for discovery). It rejects
   subsequent automatic navigations, frames, forms and popups before network
   access, and retains the direct runtime resource-scope check.
2. A primary-page Chromium CDP `Fetch` session checks requests at **every resource
   redirect hop**, and intercepts document responses at **headers**. Document
   301/302/303/307/308 Locations become unvisited policy candidates and are failed
   before Chromium follows; Refresh responses are failed too. Normal document
   responses use `Fetch.continueResponse`, not fetch/fulfill, retaining actual URL,
   network and address-space semantics.

The page is created before the boundary is installed; **no `goto` occurs until
both guards are ready**. Capture and discovery both use this ordering. Only
guarded native document continuation is permitted; automatic unguarded document
following has not been restored. Redirect policy remains Rust-owned: origin,
section, actions, robots, depth and cap are checked before explicit next visits.
Capture still refuses redirects away from its frozen target, and stores exact
requested/final provenance for the approved final target.

Interception errors other than context cancellation record `state.error` and
close the context fail-closed. Context destruction releases paused network work;
expected destroyed-session errors do not become unhandled promises. Launch still
uses sandboxed Chromium with unchanged security settings; service workers remain
blocked, loopback WebSockets closed, session references refused, and truthful
resource/image warnings retained. No accepted page/resource class was narrowed to
avoid the positive case. Existing robots and selection source is unchanged.

## Maintained regressions

`worker/product/test/resources.browser.test.mjs` runs against the **fresh signed
private pack** and its actual pinned runtime identity, with two concurrent Node
HTTP servers. It retains the useful independent raw/ordinary-continue and four
direct/redirect script input variants, plus finite cross-origin chains. It checks:

- Five same-/cross-origin direct/redirect/three-hop variants for scripts, CSS and
  PNGs: actual server requests, script execution, computed CSS, decoded natural
  image dimensions, and Loopback initiating address space.
- Actual capture metadata, zero positive-case warnings, rendered discovery links
  inserted by scripts, and final requested/final URL equality.
- Direct, redirected and multi-hop runtime-denied script/CSS/image destinations:
  **zero hits** on the owned numeric OS-loopback IPv4-mapped IPv6 alias rejected by
  the runtime predicate. Stable captures retain `ResourceFailed` and
  `ImagesIncomplete`, not timeout/false success. Page and dedicated-worker fetches
  also test allowed and denied redirects.
- Unvisited off-section/unapproved-origin document redirect candidates; a finite
  explicitly-followed approved chain and correct final capture provenance.
- Client navigation, iframe, popup, POST form, reload, response Refresh and direct
  off-section navigation: zero forbidden destination visits.
- Actual `src/index.mjs` IPC handshake/resource capture/redirect events, 401/403
  `AccessDenied`, and valid non-null session refusal **before any HTTP visit**.
- IPC cancellation after one observed pending-document request; deterministic
  cancellation while a cross-origin resource request or document response is
  paused in native interception, each under 1,500 ms; empty browser contexts and
  successful subsequent capture. Contexts, worker child and both servers close.

`capture.test.mjs` now asserts native response blocking for all five redirect
statuses rather than incorrectly requiring fetch/fulfill. `product_flow.rs` signs
the browser test into development packs and adds the ignored exact test
`actual_phase3_resource_corrections`. `scripts/check-phase3.py` adds the mutually
exclusive `--resource-corrections-only` focused entry point. No ordinary unit test
silently activates a browser or public endpoint.

## Exact serial results

All suites were serial, locked, four Rust build jobs, one Rust test thread and
Node test concurrency one. Runtime: Node **22.20.0**, Playwright **1.56.1**,
Chromium **141.0.7390.37**, signed development worker **3.0.0**. Each signed run
uses a new ephemeral local-development key/root, not production publication.

| Command / evidence | Actual result |
| --- | --- |
| `REGRESSEE_WORKER_SCRATCH_PARENT=/tmp/opencode/p3res-6aa3bf python3 scripts/check-phase3.py --resource-corrections-only --output E/locked-resources` | **Exit 0**, wrapper 123.825 s. fmt 0.262 s; locked UI build/workspace tests/clippy 9.721 s: **70 Rust passes, 9 ordinary-batch ignored**; protocol **4 passes**, 0.265 s; exact signed-resource Rust test **1 pass**, 113.542 s, including **17 Node passes / zero failures**, 5.369 s. |
| `REGRESSEE_WORKER_SCRATCH_PARENT=/tmp/opencode/p3res-6aa3bf python3 scripts/check-phase3.py --runtime-only --policy-corrections-only --output E/retained-policy` | **Exit 0**, exact original policy test **1 pass**, 260.968 s; **8 signed-worker Node passes**. Actual Rust-owned policy aliases, forbidden section/robots/action/download/origin destinations, loops, automatic navigations, approved cross-origin redirect and failed frozen-target/baseline provenance remain correct. |

The locked workspace run includes robots wildcard/end-anchor and stable visible-ID
selection regressions. No 100×3/native/soak run was repeated. The existing upstream
`proc-macro-error2 2.0.1` future-Rust warning remains.

Exact command/timing/environment records: `E/{locked-resources,retained-policy}/
commands.json`, wrapper command records/logs, `product-flow/worker-suite.log`,
`E/locked-resources/product-flow/resources/{report.json,ipc/events.json}`, and
`E/retained-policy/product-flow/policy-report.json`. Reports retain actual captures
and counters, not recursively copied histories.

Debug chronology is retained too: the first probe launch exited 1 because the
browser-path environment was missing; it was rerun using the **already-installed**
signed browser, without download/install. Initial source-suite A passed; source
suite B exposed missing CORS headers on the new fixture's **302 responses**, not a
product permission bypass. Correct fixture response headers were added; source
suite C and the final signed suite passed. These earlier failed logs/output roots
were not deleted, ignored or relabeled passed. Fixture CORS headers do not change
browser or product security policy. An initial formatter check requested multiline
Rust argument formatting; the final locked formatter check passes.

## Preservation, cleanup and limitations

`E/before/` contains only the six directly touched pre-existing inputs captured at
entry (not historical evidence trees). `E/final-audit.json` confirms **39 protected
report/lock/corpus files unchanged**, both fresh packs' four worker modules equal
maintained source, empty own/default private scratch, and no live owned-process
candidates. The validated empty own scratch parent alone was removed by `rmdir`.
Browser/server cleanup is recorded in the resource report. No sudo, install,
global service, unrelated signal or broad/recursive cleanup was used.

The `/proc` comparison starts **before the locked runs**, not before all early
source probes. It finds **zero new Node/Chromium-like zombies**, but also **48 new
git/sh zombies (46 git, 2 sh), all parented by the harness `pi` PID 2301869**.
These cannot be reaped by this process and are not attributed to test children;
their IDs/parent are recorded rather than falsely claiming a globally clean
process table. No harness/unrelated parent was signalled.

This is pinned-Chromium, owned numeric-loopback Linux evidence. Other engines,
newer browser versions, trusted public runtime/security/signing/hosting migration,
native rendering/input/resize/clipboard/IME/accessibility/latency/human approval,
Windows/macOS ownership/clean supported machines and 1,000×3 soak remain **OPEN /
NOT RUN**. Exhausted-disk/power-loss/concurrent-writer qualification remains open.
Prior independent storage/queue/100×3 evidence remains retained, not newly rerun.
The comparison corpus author's files were not edited. No nested agents or Phase 4
implementation were introduced.

## Fresh independent verifier commands

Use fresh output paths (the runner refuses reuse), serially from repository root:

```sh
REGRESSEE_WORKER_SCRATCH_PARENT=/tmp/opencode/p3res-fresh-rv \
  python3 scripts/check-phase3.py --resource-corrections-only \
  --output /tmp/opencode/regressee-p3-resources-FRESH-UNIQUE

REGRESSEE_WORKER_SCRATCH_PARENT=/tmp/opencode/p3res-fresh-rv \
  python3 scripts/check-phase3.py --runtime-only --policy-corrections-only \
  --output /tmp/opencode/regressee-p3-policy-FRESH-UNIQUE
```

Inspect the freshly signed worker source, actual resource server counters and DOM
assertions, IPC events/metadata, Loopback address-space trace, zero forbidden
document/resource hits, cancellation and process cleanup independently. Add fresh
input variants as useful; do not treat these owned fixtures as public-site or full
platform/native qualification. Accept or reject the combined boundary locally
before dependent Phase 4; carry all external/native/soak gates forward.
