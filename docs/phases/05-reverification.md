# Phase 5 — independent bounded correction reverification

Date: 2026-10-10. Fresh independent GPT-6.1 Sol High verifier. No implementation
edits, nested agents or Phase 6 work. Original findings, reports and evidence remain
historical, not relabeled as passes.

## Verdict

**Both original P2 defects are independently FIXED LOCALLY.** Stale confirmations
cannot create/activate a baseline; retained saved tiles and pointer input agree on
absolute source coordinates across seams. The bounded local review/storage
dependency gate passes. **Phase 6 may safely depend on these exercised local
contracts**, subject to the existing execution authorization; it is not started
by this verification.

One ancillary **P3 small-mask display-precision issue remains OPEN**: the fixed
2-pixel mask border imposes a minimum 4×4 viewport-pixel footprint. Tiny masks do
not have an exactly scaled visual footprint, although their source origin and
drawn absolute PNG rectangle are correct. This is not stale approval, tile drift,
wrong persisted masking or history corruption; it does not block local storage
dependency. It prevents a blanket claim that every mask's displayed dimensions
are pixel-exact. Native/human/full Phase 5 qualification remains pending.

Evidence root (`E`):
`/tmp/opencode/regressee-p5-reverifier-6f178f54a437/`.
Only this report and `docs/IMPLEMENTATION_STATUS.md` are repository edits.

## Approval-context repair: fresh reproduction and bypass

Inspected actual `crates/app/src/main.rs` dispatch/replies/async slot, `jobs.rs`,
`review.rs`, `crates/storage/src/review.rs` and migration 007. The immediate
transaction rechecks project revision, active parent, displayed analysis ID and
display generation before baseline insertion/pinning/activation. Recomputing
compatibility validates the explicitly previewed references; it does not replace
the user's selection or silently substitute a fresh confirmation.

Fresh maintained action regression and independently authored actual-Store/Jobs/
simulated-GPUI probe exercise:

- Valid preview → historical AllMasked/NotComparable analysis → Confirm:
  no dispatch from the invalidated UI preview, **zero baseline insertions and
  unchanged active baseline**. A→B→A cannot revive the original preview.
- Direct Store compatibility-entry bypass returns **“Displayed analysis changed;
  preview again”**. Direct `Work::Accept` bypass returns typed
  **`Reply::ProductFailure`**, with that truthful error applied to the UI, not a
  successful Project reply. Context-mismatched selection/run/project/analysis/
  parent/generation and altered references/provenance/coverage also refuse.
  Storage errors are contextual `anyhow` errors; no dedicated approval-error enum
  is claimed.
- Deselect the old target, navigate through actual target controls and select a
  different target → Confirm: **no auto-preview or autoaccept** of that new
  selection. Delayed Approval replies after scheduling are discarded.
- Actual asynchronous reanalysis submission, pending slot and completion are
  exercised. Confirmation refuses while pending; completion clears the preview.
  Cached identical-policy analysis returns the **same immutable ID**, but its new
  display generation rejects old authority. Direct queued-old-preview bypass after
  completion also refuses. Maintained history switching cancels superseded work.
- Changed UI project revision/active-parent context cannot dispatch confirmation.
  Actual subsequent parent activation and restore/revision changes reject old
  backend previews, with no additional baseline records.
- **Explicit fresh UI Preview → Confirm succeeds exactly once.** Independent
  three-target fixture selects the second mapped capture, inherits the exact first
  and third references (the third current capture failed), and checks run, artifact,
  date, key, inventory and requested/final provenance. Baseline count remains one
  throughout stale controls, then becomes two only after explicit fresh acceptance.
  Original run/baseline/analysis records and revision-bound review note remain equal.

Final independent evidence:
`E/private-attempt5/data/approval-results.json`, saved SQLite/CAS siblings and
`E/private-attempt5/independent-tests.log`. Maintained `analysis_review` also reruns
material/rendering/missing-artifact/collision/composite/recheck/restore controls.

Fresh real signed Chromium Product flow independently rerun by the handoff adds
the same stale-preview refusal with zero insertions/activation, then fresh selective
acceptance. Actual changed page is **6,400/180,000**; the other is **0/180,000**.
The next fixed-baseline check uses selected/inherited sources and reports both
unchanged; previous run/baseline bytes and restore survive. Read
`E/maintained/phase5-product/phase5-report.json` and `E/result-summary.json`.

## Tile-seam repair: actual saved pixels, layout and input

The maintained fresh regression independently rerun here covers **96 actual GPUI
pointer-pan cases**: y=511/512/513 and 1023/1024/1025, both directions, retained
frames 0/1, zoom 0.5/1/2/4, horizontal pan, all four modes and both drawing panes.
The old frame is deliberately retained before release/refetch. Its newly encoded
96×1137 source and every actual BGRA upload pixel are checked; the final tile is
113 rows, not stretched.

Additional independent probe does not merely agree with `ViewerFrame` metadata:

- Encodes fresh **96×1679** before/after PNGs with x/y-coded pixels, distinct edits
  across three seams and independently specified ignored regions. Actual saved
  captures, analysis and heatmap are obtained through current Store/services;
  frames come through **actual `Work::Viewer`**, not manually copied derivatives.
- Checks **1,008,288 buffer-pixel occurrences** across the overlapping bounded
  Jobs frames against literal before/after and significant/eligible/ignored color
  oracles. This includes repeated pixels across frames, not that many unique
  source pixels. The source pair has 161,184 pixels each; last tile is **143 rows**.
- **135 affine/input cases**: y=511/512/513, 1023/1024/1025 and 1535/1536/1537;
  zoom 0.5/1/1.5/2/4; horizontal pans 0/13/37. Before, after, overlay, difference
  and heatmap containers use measured GPUI bounds.
- Resolves the **actual tile containing the intended source row**, then uses its
  real buffer pixel and measured origin/size. It does not extrapolate the bounds
  of tile zero to a later tile. Viewport expectation is independently
  `origin + border + (source − pan) × zoom`; pointer actions invert that same
  measured content position.
- Real down/move/up on each original pane draws absolute PNG rectangles
  **(23 or 49, pan_y+2, 3, 5)** at intended visible content. Measured mask origins
  agree on both panes. Short final tiles retain exact decoded scaled dimensions.
  Both original CAS files remain byte-identical.

Read `E/private-attempt5/data/pixel-layout-results.json` and retained assertions
`E/private-attempt5/private-tests.rs`. Private shell behavior is unchanged: the
only source adjustment makes the maintained fixture include path absolute;
independent tests are appended. `shell-source.diff`, `source-provenance.json` and
`linked-dependencies.json` retain provenance and exact current Cargo-fingerprint
dependency resolution. No copied/reimplemented analysis engine is exercised.

Jobs still loads **at most three 512-row tiles per source**; command/update channels
remain four each. Current rendering drains retired images through `drop_image`.
Locked cancellation/backpressure, row/inflate, incomplete/CRC/trailing-PNG and
artifact integrity controls pass. This is not GPU-memory soak or maximum-latency
qualification. These are actual saved-buffer/simulated-layout/input proofs, **not
native framebuffer screenshots or human rendered approval**.

### Open P3: tiny mask border footprint

Location: `crates/app/src/main.rs:3233–3246` (`border_2`). Fresh exact-footprint
assertion fails at zoom 0.5: a correctly drawn **3×5 source-pixel** mask requests
1.5×2.5 viewport pixels but measures **4×4**. At zoom 1 its footprint is 4×5 instead
of 3×5. Final matrix retains **108 observations**, across both panes, with correct
origins and absolute PNG rectangles; masks large enough for the border scale as
requested. Minimum outline visibility is distinguishable from exact ignored-area
coverage. A bounded future presentation refinement could separate a minimum-size
outline/affordance from the exact filled geometry. No implementation edit is made.

`private-attempt4` preserves this real failed assertion and database. Final attempt
5 explicitly asserts and records the observed border minimum; it **does not erase
or mark this issue fixed**. Attempts 1–3 are verifier-only compile failures (missing
GPUI test import, API scalar/context types and static selector lifetime); their
source/logs remain, no product verdict is inferred from them.

## Exact focused and serial handoff results

All suites ran serially in fresh unique roots, with pinned Rust 1.98.1 PATH,
`TMPDIR=/tmp/opencode`, current link libraries and a bounded 40-minute subreaper
owner. Exact commands are `E/run-serial.py` / `serial-commands.json`; focused commands
are the unchanged three commands in `05-corrections.md:173–185` (fresh output
`E/maintained`). The unchanged wrapper's embedded “implementer” label is retained;
this independent session executed it and inspected the actual assertions/logs.

| Fresh check | Result |
| --- | --- |
| Focused analysis/review services | **5 PASS**, 1.365 s including Cargo |
| Focused simulated GPUI | **3 PASS**, 2.918 s; real async confirmation and 96 seam/pan cases |
| Locked UI build/test/clippy/fmt | Exit 0, 10.932 s; **114 Rust PASS** (13+9+5+39+15+5+11+17), **12 ignored**, not passes; strict clippy/format PASS |
| Signed auth/crawl/resource safety | Exit 0, 375.000 s; **1 Rust Product + 33 signed Node PASS**, retained zero-request origin/Forget/scope/resource/cancellation controls |
| Signed Phase 5 Product | Exit 0, 117.971 s; **1 Rust Product + 1 signed Node PASS**, stale refusal, fresh acceptance, provenance/recheck/restore; eight replacement/normal/readiness captures |
| Read-only corpus and final format | Exit 0, 0.114/0.264 s; retained 13 cases/26 PNGs and corruption controls; no regeneration |
| Final independently authored probes | **2 PASS**, 4.119 s, with the explicitly OPEN tiny-mask footprint recorded separately |

The maintained wrapper settles in **504.330 s**. No suites overlap. Existing
`proc-macro-error2 2.0.1` future-incompatibility warning remains, not release approval.

## Scoped audit and limits

`E/final-audit.json` / `audit.py` confirm all implementation/lock/script/corpus/prior
report inputs unchanged; among **199 entry inputs**, only the requested status
index changes. **845 historical private files**, **134 prior integrated original
paths** and **five correction Product original paths** retain hashes. All **6,026
fresh signed-manifest files** and **11 worker files** match current source/runtime.
Identity remains Node 22.20.0, worker 4.0.0, Playwright 1.56.1, Chromium 141.0.7390.37,
**DevelopmentLoopback only**.

All six retained fresh databases (including failed-probe evidence) pass SQLite
integrity/foreign-key checks at v7. **13 distinct original paths** pass digest,
metadata, complete-PNG CRC/inflation checks; **20 baseline references** resolve to
exact run/key/artifact/inventory/date. Two encrypted session files remain RGS1,
0600 in 0700 vaults; nonprivate rows and **23 logs** exclude scanned fixture
cookie/private-reference sentinels. This is bounded sentinel evidence, not general
screenshot confidentiality or real OS-keychain qualification.

**146 adopted descendants reaped; zero remaining owned children/live scoped
processes/scratch/pending files.** Completion notification was received; one scoped
pidfd event wait used no polling/signaling. Fixture owners cleaned their staging;
evidence, originals and signed runtime remain. No recursive history copying,
unowned cleanup/signaling, global installation, sudo, security/TLS/sandbox bypass,
public browsing/services or publication occurred.

**NOT RUN / NOT QUALIFIED:** native GPUI pixels/input, human/usability/accessibility
approval, real-site preset and photographic/font-AA diversity, maximum 64M-pixel
CPU/RSS and 1,000-page/3,000-target soak, real OS-key persistence/locked recovery,
Windows/macOS/clean-machine ownership, enterprise proxy/trust/CA/VPN/SSO breadth,
qualified current public runtime/signing/hosting/release. The historical blank
native-window attempt was not repeated. Earlier broad independent corpus-service,
photo/style and historical probes are retained, **not claimed freshly rerun here**
unless included in the exact maintained suites above. Missing external gates are
pending, not a blanket halt on authorized dependent local work.
