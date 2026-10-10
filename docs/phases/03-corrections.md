# Phase 3 — bounded navigation, robots and preview corrections

Date: 2026-10-09. **Corrective implementation; independent reverification pending.**
This does not supersede `03-verification.md` or erase its reproduced failures.
No Phase 4, nested agents, public browsing, native-window retry, installation,
sandbox bypass, publication or production service was introduced.

Evidence root (`E`): `/tmp/opencode/regressee-p3-corrections-suhoywkt/`.

## Exact corrections

### P1: prevent redirect destinations from being requested

`worker/product/src/capture.mjs` now supplies the shared `navigationBoundary`
used by actual discovery and capture. Playwright does not route subsequent HTTP
redirect hops, so an ordinary `route.continue()` guard is insufficient. The new
boundary **never continues a document request**: it obtains just the first
response using `route.fetch({maxRedirects:0, timeout})`. For 301/302/303/307/308
with Location, it disposes the response and aborts the browser request; Chromium
never receives that redirect. It also refuses HTTP Refresh headers.

Discovery returns the **unvisited** destination through the existing
`RedirectFound` event. Rust remains authoritative for exact approved origins,
section, skipped text, action/download exclusions, robots, depth/candidate/page
budgets and scheduler spacing before starting its next explicit navigation.
Allowed chains therefore remain separate policy-checked visits with requested
alias/destination provenance; they are not automatically followed inside a
browser request. `crates/app/src/crawling.rs` distinguishes a redirect event from
rendered-link evidence, excludes self-redirects without visiting again, and keeps
the forbidden destination in the preview reason. Existing identity deduplication
terminates multi-address cycles.

Capture/check retains the existing stricter contract: only the one exact frozen
target may become evidence; an HTTP redirect fails `RedirectOutsideScope`, with
no screenshot or baseline activation. Allowed aliases are resolved during
discovery, not silently substituted during comparison.

Only the initial explicit main-page GET is admitted. Automatic subsequent
navigations, including same-address form POST, script/meta navigation, child
documents and popup pages, are blocked before HTTP requests and invalidate that
page's evidence. This deliberately excludes embedded-frame document rendering
in this bounded Phase 3 adapter. Ordinary subresources remain subject to the
existing development loopback restriction; this is not a public resource
firewall or an authentication implementation. Browser sandbox remains enabled.

### P1: terminal robots wildcard/end-anchor matching

`crates/core/src/crawl.rs` replaces the split/find matcher with a byte-wise glob
with wildcard backtracking. Matching starts at the first byte of serialized path
plus meaningful query; `*` consumes zero or more bytes, **only the final `$`**
anchors the end, and everything else is literal. A terminal `*$` now consumes
the entire suffix, including queries. Reserved percent-encoded paths remain
literal and distinct from decoded separators; this is not a claim of complete
RFC percent-encoding equivalence. Most-specific matching literal-byte rule wins;
allow wins equal specificity. An internal literal dollar contributes specificity.

Maintained tests cover terminal/internal/repeated wildcards, zero-length matches,
backtracking, anchored/unanchored queries, literal regex punctuation and escaped
reserved paths, more-specific allows and equal-specificity allow/disallow ties.
The real fixture requires `/scope/wild/secret` to have the specific
`Excluded by robots.txt` reason and **zero requests**, not an unrelated failure.

### P2: filtered preview selection uses stable visible identities

`crates/app/src/main.rs` stores an optional `PageId`, not a raw inventory index.
Search, previous/next, highlighting and selected include/exclude share the same
visible-ID list. Search retains a still-visible selection or selects the first
match; zero matches clears selection. Row click callbacks carry the stable ID
and reject stale hidden rows. Mutations check visibility as well as identity.
Clear-filter and bulk exclude/restore do not change hidden pages accidentally.

The simulated GPUI regression exercises actual control activation, filter,
toggle, previous/next, stale-row selection, zero matches, restore and clearing
the filter. It then uses real SQLite `freeze_inventory`/`load_inventory` to assert
the exact final included identities and immutable re-freeze rejection. The small
read-only storage loader validates identity/schema; no migration was changed.
This test is **not native rendering or usability qualification**.

## Maintained verification and results

All Rust/worker/browser suites run serially, with locked Cargo dependencies,
four Rust build jobs and one test thread. The focused browser suite uses a fresh
ephemerally signed development worker with retained verified historical Node
22.20.0 / Playwright 1.56.1 / Chromium 141.0.7390.37, owned literal-loopback only.
`scripts/check-phase3.py --policy-corrections-only` adds a maintained bounded
entry point without running the already independently verified scale corpus.

| Serial command/evidence | Result |
| --- | --- |
| `RUST_TEST_THREADS=1 bash scripts/check-foundation.sh` → `E/locked-checks-4.log` | PASS: locked UI build, fmt, 70 Rust tests, all-target/all-feature clippy `-D warnings`; 8 explicitly ignored helper/browser tests in the ordinary batch |
| `python3 scripts/check-phase3.py --policy-corrections-only --output E/focused-a` | PASS: fmt 0.265 s, locked Rust/GPUI/clippy 7.906 s, 4 protocol tests 0.260 s; focused actual signed-worker/browser test 233.617 s, including 8 signed-worker Node tests. This first pass predates the final stronger fixtures/self-loop distinction and is not the latest-source verdict. |
| `python3 scripts/check-phase3.py --policy-corrections-only --output E/focused-b` | **PASS on final corrective Rust/worker source**: fmt 0.271 s; locked build, 70 Rust tests (including 5 simulated GPUI tests), 8 ordinary-batch ignored tests and clippy 17.939 s; 4 protocol tests 0.270 s; actual focused browser suite 317.192 s, including **8 signed-worker Node tests**. |
| `REGRESSEE_WORKER_SCRATCH_PARENT=/tmp/opencode/p3-suhoywkt python3 scripts/check-phase3.py --runtime-only --active-cancel-only --retained-flow E/focused-b/product-flow --output E/cancel` | **PASS**, 99.706 s: actual startup pause/resume, 600×300 visible-screen capture, failed-analysis rejection, and cancellation after one observed slow-page HTTP request. Return after **0.599237391 s**; baseline preserved. This is service cancellation latency, not native UI latency. |
| `TMPDIR=/tmp/opencode LIBRARY_PATH=$PWD/.runtime/link-libs RUST_TEST_THREADS=1 CARGO_BUILD_JOBS=4 cargo test --locked -p regressee-app --features ui-tests --bin regressee tests::filtered_preview_actions_use_visible_ids_and_freeze_exact_selection -- --exact --test-threads=1` → `E/final-gpui-action.log` | **1 PASS**, 0.03 s test time; focused actual-control/simulated-GPUI plus real persisted inventory regression. |
| `cargo fmt --all -- --check` → `E/final-format.log`; `python3 E/final-audit.py` → `E/final-audit.json` | **PASS**; scoped cleanup/preservation results below. |

The final strengthened pass adds neutral source URLs that are themselves
allowed before redirecting to action/text-excluded destinations, one-hop self
loops, an explicitly approved second-origin redirect, capture-side automatic
navigations, and stronger bulk-selection/immutable-inventory assertions.
Preliminary failed
compile/assertion logs remain in `E/locked-checks-{1,2,3}.log`; they are not passes.
The GPUI fixture initially omitted the saved-project focus flag, preventing the
control from activating; the fixture was corrected rather than weakening the
production guard. `E/locked-checks-4.log` passes the first corrective batch.

`E/focused-b/product-flow/policy-report.json`, `preview.json` and
`E/latest-result-summary.json` establish actual HTTP counters:

- **Zero** `/outside`, `/scope/private`, `/scope/wild/secret`, `/scope/delete`,
  `/scope/skipped`, `/scope/file.pdf`, `/scope/poison` and second-origin
  `/identity-provider` requests, after discovery **and** capture/check fixtures.
- Neutral redirect sources `/scope/to-danger` and `/scope/to-omit` actually
  receive requests, so action/text tests are not passing by rejecting the source.
- Allowed chain-one, chain-two and final ordered-query page each receive one
  discovery request, with both aliases retained. Self/two-address loops receive
  one request per address and are excluded. An explicitly approved second origin
  receives one robots request and one `/scope/cross-allowed` request; approval is
  a separate project, never inferred from an external Location.
- Same-address automatic POST adds **zero** requests: the form source count is
  exactly two (one discovery GET and one capture GET). Automatic navigation
  captures have no screenshots. Script/form/frame/cross/action cases report
  `RedirectOutsideScope`; the meta-refresh case reports `CaptureFailed` from the
  aborted navigation, with **zero** destination requests. This is not claimed to
  be a typed meta-refresh diagnostic.
- A genuine stable capture activates a baseline. Its subsequent redirected check
  has one failed intended target, no screenshots, `CouldNotCapture`, and preserves
  the active baseline. A list-only redirected capture likewise fails once with
  `RedirectOutsideScope` and cannot supply baseline evidence.

These are HTTP request assertions, not an all-socket network firewall claim:
closed/preconnected sockets without an HTTP request line are not counted as
requests. Public networking qualification remains open.

## Preservation, scope and verifier handoff

`E/protected-inputs.json` records existing document/policy/lock hashes;
`E/before/` retains exact prior versions of the status/index/checklist files
updated for this handoff. Earlier verification reports and evidence remain
untouched. This is not a whole historical artifact-tree byte audit.

Final audit checks **47 protected documents/locks**: only the authorized index,
Phase 3 status addendum and checklist changed; the original independent findings
and all other checked inputs are byte-equal. `E/corrected-source-hashes.json`
records the corrective source/lock versions. All four signed worker source files
match maintained source. **Zero live owned processes/zombies, zero default or
owned private worker scratch entries, zero pending CAS publications and zero
runtime install/download temporaries** remain. Fixture threads stop/join on Drop;
workers reap their owned trees. Only the empty, specifically created
`/tmp/opencode/p3-suhoywkt` parent was removed with non-recursive `rmdir`; no
unrelated process was signalled and no historical captures/logs were deleted.

The historical upstream `proc-macro-error2 2.0.1` future-Rust warning remains;
there were no dependency/lock changes or new registry versions introduced.

Fresh independent verifier: read the original findings and this correction
record, then run from `/path/to/Regressee` with unused output paths:

```sh
python3 scripts/check-phase3.py --policy-corrections-only \
  --output /tmp/opencode/regressee-p3-reverify-UNIQUE
```

Independently require **zero real requests** to same-origin forbidden sections,
robots/action/text/download destinations and the unapproved second origin, valid
approved redirect chains and cross-origin approval, loop termination, automatic
form/frame/client-navigation rejection, fixed-inventory redirected capture/check
failures, and exact filtered-preview-to-frozen-inventory selection. Do not infer
safety from a post-navigation error or from mocked route assertions alone.

The existing independently verified 100×3 scale/storage/queue evidence is retained;
another scale run is unnecessary for these bounded repairs. Native rendering/
input/accessibility/latency/human review, trusted public runtime/signing/hosting,
Windows/macOS ownership/clean-machine packaging and 1,000×3 soak stay pending.
Dependent Phase 4 still waits for fresh independent **local correction**
reverification; those external gates are not a blanket halt under current policy.
Write the fresh verdict in `docs/phases/03-reverification.md` and update the
index; leave the original `03-verification.md` findings intact.
