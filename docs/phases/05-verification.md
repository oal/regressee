# Phase 5 — independent integrated verification

Date: 2026-10-10. Fresh independent GPT-6.1 Sol High verifier. Phase 5 only;
no implementation edits, nested agents or Phase 6 work.

## Verdict

**Local integrated review gate OPEN: two reproduced P2 UI state/coordinate defects
need bounded repairs and independent recheck.** The exercised analysis, immutable
storage, migration, reference inheritance, capture-mode and retained auth/resource
contracts pass. No screenshot/history loss or persistence corruption was found.

Phase 6 should **not yet treat the integrated review/confirmation workflow as
accepted**. Its underlying retained-reference/storage contracts have positive local
evidence, not a reason to reset them or demand ZIP implementation now. Repair the
two UI boundaries below before dependent local work relies on the complete review
workflow. Native/human/public/platform gates remain separately pending, not a
blanket prohibition on authorized local development.

Evidence root (`E`):
`/tmp/opencode/regressee-p5-integrated-verifier-23dfb8d3b86a/`.
Earlier phase/status/research reports and verifier evidence remain historical and
unchanged. This report and `docs/IMPLEMENTATION_STATUS.md` are the only repository
edits.

## Reproduced material findings

### P2 — historical analysis selection leaves a stale approval confirmable

Locations: `crates/app/src/main.rs:534–541,881–913,3047–3078`;
`crates/storage/src/review.rs:316–327`.

Actual saved SQLite evidence, authoritative Jobs replies and enabled GPUI
`activate_control` actions reproduce:

1. Display a valid changed analysis and use **Preview acceptance impact**.
2. Use **View previous analysis** to display a different immutable revision for
   the same run, with every pixel masked: **NotComparable / AllMasked**.
3. The old preview and **Confirm acceptance into new baseline** remain available.
4. Confirm: the actual storage actor activates a new baseline using the *older*
   preview analysis, despite the different, noncomparable displayed revision.

Final evidence: `E/attempt5/independent-data/stale-approval-reproduction.json`.
Displayed revision `71f29ca2-74e0-4839-a9d9-ec4eac257336`; preview revision
`b80a3868-810a-484f-9304-e0da2b33e66e`; new active baseline
`1a7867e9-0b9b-45f4-8919-575322a47cd9`. Also independently reproduced in
attempts 3 and 4.

`Reply::Analysis` does not invalidate the approval. Confirmation checks pending
work, not preview-versus-display identity. Storage correctly validates the older
revision named by the preview; this is **not** a demonstrated bypass of failed
capture/material/rendering guards or arbitrary database tampering. The defect is
that the user's displayed judgment and confirmation no longer refer to the same
analysis, and the impact panel does not expose that mismatch.

Bounded repair: invalidate impact previews whenever displayed analysis changes;
bind confirmation to that displayed revision and selection, refusing stale queued
confirmation. Preserve explicit historical analysis inspection/approval after a
fresh matching preview, immutable history and existing transaction guards.

### P2 — seam-crossing pointer pan misaligns saved tiles and capture coordinates

Locations: `crates/app/src/main.rs:3093–3108,3254–3264,3292–3305`;
`crates/app/src/jobs.rs:615–632`.

The actual Jobs viewer loads saved tiles for a 96×2048 before/after pair. Starting
at y=1000 with frame tile 1, a real simulated GPUI pointer drag moves shared pan
to y=1100, crossing into tile 2. Before release/refetch, the retained frame still
starts at row 512, but the layer offset uses `pan_y % 512` (76), rather than the
retained frame's starting row. Measured GPUI layer bounds show the first visible
saved-image row is **588**, while capture-pointer mapping uses **1100**: exactly
**512 pixels** apart. Both panes share the error.

Evidence: `E/attempt5/independent-data/pan-seam-reproduction.json`, repeated in
attempts 3 and 4. These are saved-image layout/input assertions, **not native
pixel/human approval**. The mismatch is transient during drag/refetch, not an
incorrect persisted analysis or an always-wrong settled viewer. Drawing controls
are also available during viewer loading; this makes reliable visual-to-mask
positioning depend on correctly anchoring retained frames.

Bounded repair: position retained images relative to `ViewerFrame.tile * 512`
and the current pan, or prevent coordinate-sensitive interaction until matching
tiles are available. Test positive/negative seam crossings, both panes, zoom,
overlay, region navigation and mask drawing during bounded delayed refetch.

## Exact serial handoff, freshly run

The handoff was run unchanged, once, in a fresh unique output directory:

```sh
env PATH=/path/to/rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH \
  python3 scripts/check-phase5.py --output E/maintained
```

`E/run-serial.py` supplies that PATH and a bounded outer deadline/subreaper.
`E/outer-result.json`: exit 0, **510.921 s**, 146 adopted descendants reaped,
zero remaining children. All stages ran serially, without overlapping suites.
The wrapper's embedded “implementer” qualification text is unchanged; this
session independently executed and inspected its actual assertions/evidence.

| Fresh maintained stage | Actual result |
| --- | --- |
| Locked UI build/test/clippy/fmt | Exit 0, 7.925 s. **110 Rust tests PASS** (13+7+4+39+15+5+11+16); 12 explicitly ignored, not counted as passes. Strict all-target/all-feature clippy and format pass. |
| Signed auth/crawl/resource safety | Exit 0, 382.960 s. **1 actual Rust Product flow + 33 signed Node tests PASS**. Origin drift/restart refuse before visits; copied-reference Forget preserves its authenticated owner in both modes. All ten retained resource cases pass, including real script/CSS/image loading, forbidden/automatic-document zero visits and request/response-pause cancellation. |
| Signed Phase 5 Product flow | Exit 0, 119.615 s. **1 actual Rust Product flow PASS**; actual Chromium PNGs produce 6,400/180,000 Changed pixels and 0/180,000 NoSignificantChange on the other target. Selected acceptance inherits exactly one reference; subsequent fixed recheck uses both sources and reports both unchanged; old run/baseline and restore survive. |
| Signed Phase 5 Node browser test | **1 PASS**, eight actual captures. Normal source changes differ; replacement-before/after/repeat match. Raster/picture/background count 3, measured geometry preserved. Source-selector drift, unsupported media and missing-image readiness/resource warnings remain explicit. |
| Read-only independent corpus | Exit 0, 0.114 s. Exact retained validation: 13 cases, 26 PNGs/digests, 78 CRCs, 576,512 decoded pixels, 1,737,266 encoded bytes and seven corruption checks. No regeneration. |
| Final format | Exit 0, 0.264 s. Existing `proc-macro-error2 2.0.1` future-incompatibility warning is not release qualification. |

Read `E/maintained/commands.json`, nested safety commands/logs,
`signed-safety/product-flow/{phase4-report.json,forget-report.json,resources/report.json}`,
and `phase5-product/{phase5-report.json,replacement-product.json,replacement-worker/report.json}`.
The fresh safety flow uses its explicit injected key, **not** a real OS keychain.

## Additional independent integration probes

`E/service-probes.rs`, `ui-probes.rs`, `prepare-probes.py` and `run-probes.py`
retain the assertions and commands. Final `attempt5/independent-commands.json`:
five service tests pass in 2.967 s; three UI assertion cases complete in 0.865 s.
**Two of those UI cases assert the defects above; exit 0 is not an acceptance
verdict.** Crates are linked from fresh locked Cargo artifact/fingerprint
resolution, including exact host/target dependency variants, not an old verifier
executable or a reimplemented engine.

### Analysis, masks, cache and revisions

- **39 actual storage/service corpus analyses**, all three product presets over
  all 13 retained cases. Assertions compare literal independent raw/structural/
  mask-union/denominator annotations and policy expectations, not only one engine
  invocation against another. All **872,448 heatmap pixels** match the engine's
  exact significant/eligible/ignored coordinate membership.
- Tiny long-page icon: **64/196,608**, connected-region trigger in all presets,
  no global trigger; qualifying local windows **4/1/0**. Overlapping corpus masks:
  **448 ignored**, **16/5,696** remain. Near-total: **6,048 ignored**, **96/96**
  plus coverage warning. All masked: undefined **0/0**, NotComparable, no clean
  classification or matching-revision approval. Added/removed background-colored
  height bands retain **1,536 structural pixels** and explicit height triggers.
- Advanced disabling keeps 400 significant pixels/visual evidence while no
  enabled criterion fires. Exact repeated policy returns the original cached
  revision; policy and mask edits create distinct revisions/cache identities.
  Source identities include exact run/artifact, profile, rendering and mask pair.
- New overlapping masks count their **512-pixel union** once. Zero-area,
  overflowing and out-of-bounds rectangles refuse. Same-sized *new capture pair*
  and changed-height pair both refuse stale masks as NotComparable with explicit
  revalidation failure. Rebinding the retained rectangles to the new geometry
  preserves the 1,536-pixel structural band and +16 height evidence; no stretch or
  DOM-content attachment occurs.
- Pre-cancel does not add an analysis; a separate 512×4096 midflight probe returns
  typed cancellation in **9 ms**, publishes no revision and leaves staging empty.
  This bounded sample is not maximum-size/blocked-I/O latency qualification.
- Optional horizontal blend handling retains 64 raw changes, suppresses 64
  significant blend pixels only when enabled, and changes cache identity. It is
  not a font classifier or real-AA accuracy measurement.
- A valid grayscale saved-image fixture accepted by ingestion but unsupported by
  the strict engine persists **analysis failure / NotComparable**, with no heatmap
  or approval; it does not become CouldNotCapture or unchanged.
- Original analysis JSON, original run JSON and original screenshot bytes remain
  identical. All three review states and notes stay bound to the original
  analysis; changed rules/cache selection do not erase their history.
- A fresh **real legacy RGB analysis record**, its referenced images and run are
  placed in the v5 schema, then opened through actual Store migration 006. The
  complete old record bytes remain identical; it loads alongside a new perceptual
  revision in v6. This supplements the maintained synthetic migration assertion.

### Immutable approval, provenance and subsequent checks

The independent **two-page × two-profile** store probe selects one mapped capture
and inherits three exact references, including a target whose current check failed.
The failed target itself, empty/duplicate selections, conflicting project revision,
baseline-ID collision and repeat/stale-parent approval refuse without extra
activation or baseline records. Intentional production/staging address mapping
alone remains eligible, with requested/final provenance.

The next four-target check reports all NoSignificantChange and binds its before
descriptors to the selected new run for one key and the original inherited run
for the other three. Restore in both directions succeeds; stale revision restore
refuses. Maintained material-mode/rendering mismatch, foreign-project preview,
corrupt/missing artifact and non-activatable representative-preview controls also
pass. Transactional storage guards are intact; the stale **UI preview identity**
finding is distinct.

### Saved-image GPUI and replacement

- The independent UI viewer ingests the **exact fresh Product before/after
  Chromium originals**, not demo images, into fresh fixture records, and obtains
  tiles/request/heatmap through real `Jobs::Viewer`. The fixture metadata is
  synthetic; the screenshot bytes are actual Product outputs. At zoom 2, pan
  (20,40), a real simulated pointer draws capture rectangle **(30,50,20,10)**.
  Measured mask bounds agree on both panes. The actual overlay pointer bar reaches
  approximately 25% and 80%; actual actor/region navigation and heatmap mode work.
  The maintained draw/redraw/delete and zoom/pan actions also pass. These prove
  source/layout/input behavior on GPUI's simulated platform, not native rendering.
- Private UI source retains all maintained logic; only diagnostic selectors and
  one bounds log are added. `E/shell-instrumentation.diff` records the differences.
  No copied or independently revised engine is tested: the app re-export links
  the current core module. Historical standalone engine verification remains a
  separate dated contract, not byte-identical/current integration acceptance.
- `E/independent-photo.log`: **one additional Node test PASS**, 1.516 s, five
  actual signed-worker-module captures plus a live computed-style control.
  Normal is not relabeled; replacement is chosen before capture with matching
  actual runtime identity and `neutral-raster-v2`. Different raster sources yield
  equal replacement originals. Raster/picture/simple-background count is 3;
  vector image remains retained. Measured rectangles, object-fit **contain**,
  object-position, borders/radius/margins and background size/position remain
  identical before/after transformation. Algorithm v3 refuses validation.
- Source-dependent CSS emits both SourceDependent and GeometryChanged; a >2,048
  element page emits GeometryNotMeasured with **geometryPreserved:null**, not true.
  The maintained missing-image test still emits ImagesIncomplete/ResourceFailed;
  only exact transform-started aborted source reloads are separately counted.
  Actual Product replacement preview cannot activate as a baseline; replacement
  baseline/check modes and algorithm remain immutable and matching. No post-hoc
  normal-to-placeholder reanalysis or separately saved hidden original photo is
  introduced. Unsupported/source-dependent media remains an explicit limitation.

## Preserved verifier failures and final audit

The first private compile selected incompatible cached host/target dependency
variants (including png 0.18 rather than 0.17); the next artifact resolver found
both serde variants and stopped before tests. Correct exact dependency fingerprint
resolution fixes the **verifier**, not product code. Attempts 3/4 pass all service
probes and reproduce both defects, but their overlay assertion fails: the private
panel was drawn as an unconstrained standalone root (2-pixel slider/zero-width
canvas), unlike its actual flex-column parent. Attempt 5 supplies that parent:
800-pixel slider, 798-pixel canvas, pointer test passes. Original failed logs/data
remain; no slider product defect or visual approval is inferred from them.

`E/audit.py` / `final-audit.json` independently confirm:

- **187 entry-fingerprinted inputs** unchanged except the requested status index;
  all implementation/lock/script/corpus/prior report bytes remain unchanged.
  **845 historical private engine-verifier files** still match their retained hashes.
- All **6,026 freshly installed signed-manifest files** match hashes/sizes;
  all **11 worker source/test/fixture files** also match current maintained bytes.
  Actual Product resolves/verifies the signed pack. Identity remains Node
  **22.20.0**, worker **4.0.0**, Playwright **1.56.1**, Chromium **141.0.7390.37**,
  **DevelopmentLoopback only**, not public/runtime/release qualification.
- Across retained fresh databases, **134 distinct saved original paths** pass
  digest/metadata/complete-PNG CRC/inflation checks; **96 baseline references**
  resolve to their exact run/page/profile/artifact/date/inventory. SQLite integrity
  and foreign-key checks pass, including retained failed-probe fixture databases.
- Two retained session files are encrypted RGS1, 0600 in 0700 vaults. Nonprivate
  rows, **27 logs and 18 metadata files** exclude the scanned fixture cookie
  payloads/private session references. This is a bounded sentinel/reference audit,
  not a general assurance that screenshots cannot contain confidential content.
- **Zero live scoped processes, remaining owned children, scratch entries or
  pending files**. The independent photo owner reaps four descendants. Audit is
  read-only: no unowned signals, recursive copies/deletion, global installations,
  sudo, security/TLS/sandbox bypass, public browsing/services or publication.

## Qualification still pending

**NOT QUALIFIED / NOT RUN:** native GPUI pixels/input, human visual/usability and
accessibility approval; non-developer/real-site preset accuracy and representative
licensed photographic/font-AA diversity; maximum 64-million-pixel CPU/RSS and
1,000-page/3,000-target soak; real OS-key persistence/locked recovery; Windows,
macOS/clean supported machines; enterprise proxy/OS trust/custom CA/VPN/SSO and
device-bound/rotating-session breadth; qualified current public runtime/signing/
hosting/release. No native attempt was repeated: historical blank/unmapped evidence
offers no new useful basis, and simulated GPUI success is not human approval.

Phase 6 ZIP/retention work is **not required or implemented by this verification**.
Next local gate: the two bounded P2 repairs and a fresh independent integration
recheck, with the passing immutable evidence/storage/auth boundaries preserved.
