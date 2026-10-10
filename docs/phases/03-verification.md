# Phase 3 — independent local crawling verification

Date: 2026-10-09. Fresh independent GPT-6.1 Sol High verifier; **Phase 3 only**.
No implementation edits, nested agents, Phase 4 work, native retry, public-site
browsing, sandbox bypass, installation, publication or production hosting.

## Verdict

**Bounded local corrections required before dependent Phase 4 work.** The fresh
handoff, real 100-page/300-target capture and substantial storage/queue/UI checks
pass, but independent fixtures reproduce **two P1 crawl/navigation safety defects
and one P2 preview-selection defect**. Maintained green tests do not erase them.

The immutable storage, normal fixed-inventory comparisons and Linux queue recovery
evidence can be retained; no rewrite or history deletion is warranted. Repair and
independently reverify the narrow findings below before treating Phase 3 crawling
as a safe dependency. This is a correctness-based local verdict, **not** the
superseded blanket halt for missing external/native qualification. Decisions
0005/0006 still permit remaining local phases once material dependencies are fixed.
Full Phase 3/public/native/platform/release acceptance remains open.

Evidence root (`E` below):

`/tmp/opencode/regressee-p3-verifier-f863bbf0ec09/`

Read PRODUCT/PLAN, Phase 3 checklist/status, prior foundation and Phase 2
verification/reverification, decisions 0005/both 0006/0007/0008 and actual Rust
crawler, scheduler, supervisor, storage, product actor, GPUI and worker assertions.
Loaded Playwright testing guidance; used the signed product worker rather than a
CLI-installed browser. All actual website requests were owned literal-loopback
fixtures, including a separate **unapproved project origin** on another owned port.

## Reproduced material findings

### P1 — HTTP redirect destinations are visited before policy validation

- **Files:** `worker/product/src/discover.mjs:23–32`,
  `worker/product/src/capture.mjs:37–56`,
  `crates/app/src/crawling.rs:281–310`.
- The worker relies on navigation routing to intercept redirect destinations.
  In the actual pinned browser, HTTP redirects reach their destinations before
  the later final-URL/Rust policy checks. Rejecting the resulting preview/evidence
  is too late to prevent navigation side effects.
- **Real discovery:** approved source `http://127.0.0.1:32827/scope` links to
  `/scope/redirect-private`, redirecting to robots-disallowed `/scope/private`, and
  `/scope/cross`, redirecting to `/outside` on owned port **33525**, which was not
  approved. Both forbidden destinations receive **one actual HTTP visit**. The
  preview nevertheless excludes the originating candidates as outside policy.
- **Real capture:** `/errors/redirect` redirects to `/redirect-destination`;
  that destination receives **one actual visit** before the target fails with
  `RedirectOutsideScope`, one attempt. No erroneous screenshot/baseline activation
  was accepted. Checks use this same capture adapter; a separate redirected-check
  fixture was not run.
- **Impact:** origin/section/robots safeguards and off-inventory visitation are
  not enforced before HTTP redirect visits. Normal fixed-list comparison tests
  do not qualify this path. No public redirect was tested; the development
  browser's claimed network restriction must not be assumed proven for redirects.
- **Required bounded correction:** enforce destination policy **before any
  destination visit**, retain requested/final/alias diagnostics, and add actual
  server-count assertions for approved aliases, robots/action/section exclusions,
  cross-origin redirects, redirect chains/loops and fixed-target checks. Do not
  replace this with a post-navigation error or a sandbox/network bypass.
- **Evidence:** `E/fixture-visits.json`, `independent-preview.json`,
  `discovery-policy-findings.json`, `service-tail/fixture-visits.json`,
  `service-tail/probe-report.json`, `required-zero-redirect-visits.json`.
  `redirect-required-zero.py` **fails**, exit 1, on the required zero-visit assertion.

### P1 — robots terminal wildcard/end-anchor rules fail open

- **File:** `crates/core/src/crawl.rs:268–287` (`robot_match`).
- `Disallow: /private/*$` incorrectly allows `/private/secret`. The matcher skips
  the empty terminal wildcard part but still requires the remaining text empty;
  the wildcard therefore does not consume the suffix before the end anchor.
- **Real discovery:** robots returns `Disallow: /scope/wild/*$`;
  `/scope/wild/secret` is included in the preview and receives **one actual visit**,
  with robots respect enabled and no override.
- **Required bounded correction:** correct wildcard/end-anchor matching and add
  positive/negative tests for terminal/internal wildcards, anchored/unanchored
  patterns, queries and allow/disallow precedence, plus a real zero-visit fixture.
- **Evidence:** `E/early-findings.json`, `fixture-visits.json`,
  `discovery-policy-findings.json`; independent path-dependency
  `terminal_wildcard_robots_must_deny` **fails**, exit 101, in
  `E/independent-robots-unit.log`. This is not merely an unrun robots gate.

### P2 — filtered preview selection can change an invisible page

- **File:** `crates/app/src/main.rs:774–803,804–810,1007–1054`.
- Applying URL search resets `selected` to raw inventory index zero, not the
  first matching row. Include/exclude-selected still operates on that raw index;
  preview previous/next also traverses the unfiltered inventory.
- **Fresh simulated GPUI reproduction:** load an actual discovery preview, search
  for the included `/scope/rendered` row (index **13**, not zero), then activate selected
  include/exclude without clicking another row. The visible searched page stays
  included; the hidden root page is toggled instead.
- **Impact/correction:** the reviewed capture inventory can differ from the user's
  visible selection. Keep selection/navigation within visible rows, or disable
  the action when no visible selection exists. Reverify search, zero matches,
  previous/next, click selection, exclusion/restoration and frozen capture inventory.
- **Evidence:** unchanged actual shell source plus independent tests in
  `E/probe/src/gpui.rs`; `E/independent-gpui.log`: **1 pass / 1 failure**, exit 101.
  A further bounded diagnostic, `independent-gpui-selection-diagnostic.log`, also
  fails exit 101 and records actual raw selection **0**, visible match **13**,
  hidden root inclusion **true → false**, visible-page inclusion still **true**.
  This is an action/state reproduction, not a native-rendering observation.

## Actual serial execution

Working directory: `/path/to/Regressee`. Browser/worker/process
suites never overlapped. Builds used four jobs; Rust tests one thread. Read-only
source/artifact inspection overlapped a serial runner, not another browser suite.
Fresh development signatures/current worker **3.0.0** were used with historical
Node **22.20.0**, Playwright **1.56.1**, Chromium **141.0.7390.37**; sandbox enabled.

| Command/scope | Actual result |
| --- | --- |
| `python3 scripts/check-phase3.py --output E/handoff` | **Exit 0**; all constituent checks below ran serially |
| ↳ format check; `bash scripts/check-foundation.sh` | **PASS**, 0.266 / 8.130 s: locked UI-inclusive build, fmt check, **68 Rust tests**, all-target/all-feature clippy `-D warnings`; seven explicitly ignored helper/browser tests in the ordinary batch |
| ↳ private Node protocol suite | **4 PASS**, exit 0, 0.265 s |
| ↳ actual full Phase 3 product/browser suite | **PASS**, exit 0, **530.497 s**, including fresh signed-worker **6 Node boundary tests** and actual browser captures/recovery/scale |
| `check-phase3.py --runtime-only --active-cancel-only --retained-flow E/handoff/product-flow --output E/cancel` | **PASS**, exit 0, 102.352 s; actual startup pause, 600×300 viewport, failed-analysis rejection, observed active-navigation cancellation |
| Independent locked/offline path-dependency build | **PASS** after verifier lock resolution; final graph introduces **zero registry versions** outside the product lock |
| Independent scope/list/depth/cap/robots fixture | Positive identity/preview assertions passed, but **P1 navigation/robots failures reproduced**; initial whole probe did not finish (chronology below) |
| Independent `tail` fixture (scheduler/errors/disk/resume) | **Exit 0**, 224.782 s; positive assertions pass, capture redirect visitation remains explicitly a finding, not a safety pass |
| Independent `profiles` fixture | **PASS**, exit 0, 58.648 s; six actual baseline/check visits, browser-observed dimensions/screen/DPR/touch/UA and frozen edited-setup comparison |
| Independent GPUI tests | **1 PASS, 1 FAIL**, exit 101, 4.425 s; real stored multi-project/results sequence passes, filtered-preview selection fails |
| Independent robots unit / required-zero redirect assertions | **FAIL**, exit 101 / 1; reproduced P1s above |
| Independent scale PNG and populated migration audits | **PASS**; actual artifacts/records, not mocked scale jobs |

Exact commands/environments/logs: `E/handoff/commands.json`,
`E/cancel/commands.json`, `E/independent-commands.json`,
`E/independent-commands-b.json`, `serial-followup.py`, `serial-b.py` and probe sources.

### Independently supported behavior (not blanket acceptance)

- Real rendered anchors, list-only **zero discovery visits**, list-plus-links,
  ordered/repeated meaningful queries with tracking removal, two distinct hash
  routes, alias preview, section/text/action/download exclusions, page cap and
  depth-zero coverage. Direct blocked links receive zero visits; **redirect and
  terminal-wildcard exceptions above invalidate a blanket policy-safety pass**.
- Unavailable robots (503) stops the origin; explicit authorized override permits
  visits. Re-enabling respect on a saved baseline produces one **skipped** intended
  check target, zero compared pixels, `CouldNotCapture`, and preserves the baseline.
- Desktop **1440×900**, tablet **768×1024**, mobile **390×844**, DPR 1: actual PNG
  dimensions and in-page reports agree with frozen screen, touch and UA metadata.
  Baseline/check each report all three profiles. Custom **600×300** visible-screen
  capture passes the targeted maintained suite. These are Chromium emulations.
- Normal comparisons retain the frozen inventory/profiles despite edited source
  and selected profiles. The maintained newly added page receives **zero visits**;
  independent edited `/never-discover` setup also receives zero visits. Failure
  coverage is not unchanged, and incomplete activation/fabricated failed analysis
  is refused. HTTP redirect safety is separately failed above.
- Actual pause retains two targets; actual app kill/dead-owner restart retains
  two originals and resumes pending targets without replacing them. Live-owner
  recovery returns zero. Active `/slow` navigation is observed before cancel:
  **one visit**, **0.5912 s** to return; this is service latency, not native input.
  Cancelled queues cannot activate a baseline.
- Independent real partial queues reject changed project revision/settings and
  mutated frozen profile. Removing write permission from the **owned CAS directory**
  causes actual publication failure: interrupted queue, one successful original,
  two pending targets, no activation. Restoring permissions permits validated
  resume to three captures. Server counts prove saved originals are not revisited;
  only the unpublishable target is retried. Original bytes remain equal.
- Gentle actual starts are **1.082 / 1.109 s** apart. Fast/per-origin-two visits
  start in two overlapping pairs, with **1.316 s** between batches; no third
  delayed fixture request starts before the preceding pair can finish. Configured
  one-retry cases make exactly two rate-limit/server/timeout visits (intervals
  **2.107 / 2.111 / 3.446 s**); 404/403/redirect make one attempt, with exact typed
  outcomes. Eight real **unhandshaken worker processes** exhaust the global permit;
  the ninth fails specifically and Drop releases it. **Eight simultaneously active
  browsers were not exercised**; source and these permits are not that measurement.
- Actual `Store::open` migrates populated v3 tables containing **3 projects,
  7 runs, 1 baseline, 3 analyses**, preserving every selected record byte. This
  supplements the maintained migration assertion, which only populated projects.
  The fixture rows were copied from the fresh handoff into the old SQL schema;
  historical worker-2 project reopening was not separately repeated.
  Maintained two-store claim rejection/CAS invariants pass; a new simultaneous
  cross-process resume race was not separately run.
- Fresh GPUI actor tests open **three real saved projects**, retrieve authoritative
  evidence, put **30 Changed** results before **3 CouldNotCapture**, exercise all
  four filters, URL search, profile-group control and busy input guards. Source
  actions submit `Work` to real `Store`/`Product`; no demo results/settings replace
  discovery/capture/check/resume/activation. The test is simulated GPUI plus actual
  storage, not native desktop usability/rendering qualification.

### Real 100-page × 3-profile measurement

`E/handoff/product-flow/phase3-report.json` and
`E/independent-scale-artifact-audit.json` establish **100 distinct pages,
300 intended/completed targets, 300 actual HTTP visits**, one attempt each;
**98.108 s** capture interval and **12,504,571 bytes** CAS artifact-store growth.
Independently verified all **300 original PNGs**: SHA-256/size, chunk CRCs,
IEND-at-EOF, complete decompressed scanlines, exact profile dimensions;
**100 originals per profile**, 3,730,929 summed original PNG bytes.

Full-suite 250 ms sampled group RSS peaks, **not isolated-scale or simultaneous
peaks**: Rust/Cargo **185,966,592**, Node **411,045,888**, Chromium
**3,406,028,800 bytes**. Shared pages can be double-counted; short processes can be
missed. Callback intervals are not native latency. Host: Ryzen 9 9950X, 32 logical
CPUs, 129,208,228 kB MemTotal, Fedora-kernel Linux x86_64/glibc 2.43. This is a
developer-host fixture measurement, not minimum-hardware/public-site qualification.
**1,000-page / 3,000-target soak remains NOT RUN.**

## Retained verifier failures and scope limits

The initial independent build refused a mismatched external lock under `--locked`;
offline resolution against product lock versions then succeeded. The unused initial
generated lock and failed log are retained. No product dependency was changed.
The first whole fixture reached the two real P1 findings but stopped on **my
unbreakable diagnostic text** causing mobile overflow/`ImageLimit`; only the fixture
was given wrapping. The next whole probe hit its **360 s driver budget** after
completed profiles/scheduling/errors and a real partial pause; later checks were
not claimed passed. Separate bounded `tail`/`profiles` runs completed the required
checks. Original failed roots/logs remain, not overwritten. An initial audit also
incorrectly counted two unrelated desktop crashpad helpers; the scoped audit
explicitly preserves them. No unrelated process was signalled.

**NOT RUN/pending:** 1,000/3,000-target soak; native rendering/input/resize/clipboard/
IME/accessibility/human visual and usability approval; native interaction latency;
Windows/macOS ownership and clean supported-machine packaging; trusted current
public runtime, security/support migration, production signing/hosting and release
qualification; genuinely exhausted filesystem/ENOSPC, competing disk writers and
power-loss semantics. Permission-fault recovery and low-space helper assertions
are not an exhausted-disk qualification or reservation guarantee. Earlier Phase 2
long/short-TMPDIR/process boundary suites were retained, not broadly rerun here.
Later auth/environment/masks/approval/archive/cleanup features were not demanded
or implemented as Phase 3 acceptance conditions.

## Preservation and handoff

Before authorized report/index writes, the **117 hashed existing source/document/
lock inputs** were unchanged. Only `docs/IMPLEMENTATION_STATUS.md` is regenerated
among them; its exact prior version is `E/IMPLEMENTATION_STATUS.before.md`.
Old verification/status/research documents and artifacts were not rewritten or
recursively snapshotted. No whole historical artifact-tree byte audit is claimed.
Final scoped audit: **zero live owned app/Node/browser/fixture candidates, empty
private worker namespace, zero pending fresh-root publications, zero new executed
registry versions**. Fixtures shut down, owned fault permissions were restored;
fresh captures/logs/probes and old evidence remain retained. See `E/final-audit.json`.

**Next actor:** bounded Phase 3 corrective pass for the three findings, then a fresh
independent verifier. Retain proven storage/inventory/queue evidence; recheck actual
zero-visit redirect/robots fixtures and filtered-preview-to-capture actions before
Phase 4 depends on the crawler. Carry unrun native/public/platform/soak gates forward
without relabeling them passes or restarting blanket feasibility halts.
