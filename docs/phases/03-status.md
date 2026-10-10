# Phase 3 — guided multi-page capture delivered locally

## Corrective update — 2026-10-09

Fresh independent verification in `03-verification.md` reproduced two P1
navigation/robots defects and a P2 filtered-selection defect. Bounded repairs
and maintained regressions are recorded separately in `03-corrections.md`.
**Independent correction reverification is pending; dependent Phase 4 has not
started.** The implementation evidence below is historical and is not rewritten
as a safety pass. Native/public/platform/soak gates remain open under the current
local-continuation policy.

## Historical implementation record (retained below)

Date: 2026-10-09. **Phase 3 implementation delivered; independent verification
pending. Not full Phase 3, native, public-runtime, platform or release acceptance.**
The prescribed 1,000-page/3,000-target soak is **NOT RUN**. No Phase 4+ workflow,
nested agent, public-site navigation, global installation or publication was added.
The independently re-verified Phase 2 scratch/supervisor remains the ownership
foundation; Phase 3 adds a process-global browser-operation ceiling above it.

## Delivered behavior and source map

| Area | Maintained behavior |
| --- | --- |
| Guided native setup | `crates/app/src/main.rs` / `jobs.rs`: real saved base/list addresses, list-only versus links, approved origin set, website/section boundary, skipped text, cap, screen sizes, extent and Gentle/Balanced/Fast. Advanced depth, queries/fragments, custom dimensions, concurrency/per-origin limits and explicit authorized policy overrides. No JSON-editor replacement or dead mock controls. |
| Identity and scope | `crates/core/src/crawl.rs`: guided identity **v2**, meaningful ordered/repeated query pairs and encoded paths/trailing slashes retained, known tracking pairs removed by default, ordinary anchors dropped, hash routes retained. Credential-free HTTP(S), exact approved origins and slash-boundary sections; action/download exclusions with explanations. Legacy **v1** inventories retain their saved identity. |
| Discovery | `crates/app/src/crawling.rs`, `worker/product/src/discover.mjs`: actual rendered anchors, no arbitrary clicking/form submission; serial bounded discovery, robots allow/disallow/delay, page/depth/candidate budgets, deduplication and loop termination. Redirect destinations are candidates checked by Rust before visiting. Missing robots permits; unavailable robots stops that origin unless explicitly overridden. |
| Preview | Searchable, virtualized candidate list, include/exclude and keyboard selection, requested redirect aliases, reasons, completeness warnings and page/profile/storage/time estimates. Estimates are approximate, not measured completion promises. Capture freezes a separate immutable inventory revision. |
| Multi-profile jobs | Intended targets are included pages × selected desktop/tablet/mobile/custom profiles. Exact frozen profile/runtime/extent metadata, keyed originals/tiles and coverage persist. Scheduler presets use ceilings 1/2/4; advanced 1–8, global eight-browser permits, separate per-origin limits, spacing, bounded temporary-error retries/backoff. Spaced origins deliberately use one slot per batch. |
| Pause/recovery/cancel | Targets publish at safe boundaries. Pause stops new batches; cancel interrupts active work with the existing supervisor. Dead-owner restart interruption preserves evidence; validated resume requires settings/runtime continuity and an exclusive compare-and-swap owner claim. Mixed capture times are disclosed. Completed/cancelled runs are not resumable. |
| Baseline/check | Complete intended coverage plus explicit confirmation is required for activation. Checks visit only the saved baseline inventory/profile snapshot, never discovery or edited setup. Redirect/access/404/timeout/failed/missing targets cannot be classified unchanged. Failed-target analysis cannot invent compared pixels. Previous baselines and originals remain retained. |
| Results/viewer | Changed-first virtualized rows, search/filters/profile grouping, separate page and screenshot counts, selected-target tiled before/after/diff viewer. Search/filter/group focus remains available during background work; mutations remain disabled. Failure completion refreshes authoritative evidence/coverage. |
| Storage safety | Migration `004-crawling.sql` preserves old record JSON bytes transactionally, adds immutable inventories and raises bounded evidence records to 64 MiB. Available-space preflight and per-batch checks stop safely without activation/deletion. This is not disk reservation or a cleanup manager. |

Policy detail: `docs/decisions/0008-phase3-crawl-identity-and-queues.md`.
Surface contract: `docs/crawling-surface.md`, retaining `DESIGN.md`'s proposal.

### Runtime boundary

The new product worker has explicit identity **3.0.0**. Historical worker-2
evidence is preserved, not silently migrated or compared under worker 3.
Integration packs use ephemeral development signatures and the historical
Node **22.20.0**, Playwright **1.56.1**, Chromium **141.0.7390.37** bytes.
They remain **owned literal-loopback fixtures only**, with sandboxing and
development network restrictions. Source support for configured release trust
does not qualify a public runtime, create signing/hosting authority or approve
those old bytes for websites. No public-runtime gate has been relabeled passed.

## Exact verification scope

Working directory: `/path/to/Regressee`.
Evidence root (`E`): `artifacts/phase3-implementation-9190a5be4a89/`.
`E/handoff-source-hashes.json` records the delivered source/document hashes and
the cited final logs/reports, so a verifier can distinguish subsequent changes.
Commands below were serial; Rust build parallelism was bounded to four jobs and
Rust tests to one thread. These are implementer checks, **not independent verdicts**.

| Retained command/evidence | Actual result |
| --- | --- |
| `cargo fmt --all`, followed by `RUST_TEST_THREADS=1 /usr/bin/time -v bash scripts/check-foundation.sh` → `E/latest-source-locked-checks.log` | **PASS**, 12.37 s: locked UI-inclusive binary build, fmt check, **68 Rust tests**, all-target/all-feature clippy `-D warnings`. Seven explicit read-only/browser/helper tests remain ignored in the ordinary batch. |
| `/usr/bin/time -v .runtime/installed/node/bin/node --test worker/product/test/protocol.test.mjs` → `E/latest-source-protocol-suite.log` | **4 PASS**, 0.05 s command wall time. |
| `python3 scripts/check-phase3.py --runtime-only --output E/runtime-c` | **PASS**, 587.82 s wall: fresh signed current-worker full browser suite, **6 signed-worker Node tests**, real discovery, baseline, unchanged/changed/failed checks, pause/resume, actual app crash/recovery and actual 100×3 capture. Exact Cargo command/RSS in `runtime-c/commands.json`. |
| `python3 scripts/check-phase3.py --runtime-only --active-cancel-only --retained-flow E/runtime-c/product-flow --output E/runtime-d` | **PASS**, 102.64 s: startup-paused queue, actual custom viewport, failed-analysis accounting rejection, actual slow-page navigation cancellation and baseline preservation. `runtime-d/commands.json`, `actual-phase3.log`, `product-flow/active-cancel-report.json`. |
| `python3 E/final-audit.py` | Read-only audit: **zero live owned executable candidates, zero scratch entries, zero runtime temporaries/pending artifact publications**. Protected historical hashes and exact overwritten-file snapshots match. No signals/deletion. |

The full browser suite captures **11 included pages × 3 profiles** after bounded
section discovery. It verifies rendered links, ordered/repeated queries, trailing
slashes, hash routes, a redirect alias, redirect/pagination loops, robots/action/
section exclusions, a rate-limited retry and a missing page. Robots/action/outside
excluded fixture addresses receive zero visits. Pause retains **2** captures;
actual app kill/restart retains **2** captures and resumes without replacing their
originals. A newly added page receives **0 visits** during checks. A failed page
accounts for **3 failed profiles**, all `CouldNotCapture`, while the active baseline
is unchanged. Limited rediscovery does not mutate the prior inventory.

The targeted final suite exercises a paused-before-capture queue, resumes to an
actual **600×300** viewport PNG and rejects a fabricated unchanged/compared-pixels
analysis for a failed target. Active navigation to `/slow` is observed by the
fixture before cancellation: **1 actual visit**, **0.596776409 s** from observed
navigation/cancel trigger to return. That is service cancellation latency on this
fixture, **not native input latency**.

### Chronology and retained failures

`runtime-a` failed after 146.53 s on an incorrect expected redirect-alias count
(expected 2, actual 1); the self-redirect is correctly excluded. The expectation
was corrected, not the evidence. `runtime-b` reused that retained signed pack and
passed in 450.54 s, including 300 real visits in 89.79 s. Its identical fixture
screens deduplicated to zero artifact growth; it is preliminary, not a cold corpus.
`runtime-c` rebuilds/signs the current worker and uses distinct page content.
Earlier compile/clippy/locked-check failures and subsequent passes are retained
under `E`; no failed log was overwritten or turned into a pass.

After `runtime-c`, startup pause and failed-analysis guards were exercised by
`runtime-d`. Failure-triggered GPUI evidence refresh passed locked checks.
The final delta only keeps Results search/filter/group input/focus usable while
busy and adds its focus-model regression; the latest locked checks cover it.
The expensive full suite was **not repeated after those Rust/UI deltas**.
Independent verification should run both suites against latest source.

## Scale, resources and preservation

`runtime-c/product-flow/phase3-report.json` records a real list-only Fast run:

- **100 distinct pages, 3 profiles, 300 completed captures, 300 actual visits**;
  one attempt per target, **105.092793220 s** capture interval.
- Artifact-store growth **12,504,571 bytes**. This is CAS artifact growth, not
  total SQLite/runtime/evidence-root growth; earlier desktop captures can deduplicate.
- Full-suite 250 ms sampled peak group RSS: Rust/Cargo **183,517,184 bytes**,
  Node **412,995,584 bytes**, Chromium **3,397,697,536 bytes**. These are separate
  observed group peaks across the suite, not isolated-scale memory or a summed
  simultaneous peak. Shared pages may be double-counted and short-lived processes
  missed. `runtime-d` peaks: **155,910,144 / 97,075,200 / 836,186,112 bytes**.
- Final locked command `/usr/bin/time` maximum RSS **1,052,288 KiB**, user/system
  CPU **22.82 / 8.44 s**. This command statistic is distinct from sampled groups.
- `safe_boundary_intervals_seconds` includes startup/dispatch/progress callbacks,
  not solely persisted boundaries. **Native UI latency/scroll responsiveness:
  NOT MEASURED.** Source virtualization and focus tests do not qualify rendering.
- **1,000-page / 3,000-target soak: NOT RUN** because it exceeds this bounded
  implementation-verification budget. It remains an unpassed completion gate.

Audit host: AMD Ryzen 9 9950X, 32 logical CPUs, `MemTotal: 129208228 kB`, Linux
7.2.4-200.fc44 x86_64, glibc 2.43. Available workspace bytes at audit:
**765,139,169,280**. Evidence allocation at audit: **2,349,846,528 bytes** (`du`),
intentionally including signed packs, runtimes, SQLite, captures and failed logs.
This is a developer machine measurement, not a minimum supported-machine result.

`E/final-resource-preservation-audit.json` compares Phase 0/1 reports, existing
decisions and PRODUCT/PLAN/DESIGN with the retained Phase 2 pre-check manifest;
all match. `E/preservation.json` and `before/` retain only specifically overwritten
source/checklist/index files and match exactly. Phase 2 reports remain untouched;
their handoff hashes are recorded but no pre-implementation whole-artifact hash
manifest exists, so a whole-history byte-for-byte audit is **not claimed**.
Historical runtime inventory/signatures were verified before packaging; outputs
use unique paths. No broad copy/delete, privileged tooling, unowned kill, repository
reset, commit or publication occurred. This directory has no Git metadata.

## Open qualification and limits

- **Independent Phase 3 verification: pending.** Implementation checkboxes are
  not acceptance or authorization for Phase 4.
- **Public runtime/signing/hosting/support qualification: pending**; no public
  browsing or public-site hands-on preview was performed.
- **Native rendering/input, resize, clipboard/IME, accessibility, hardware/Wayland
  and human visual approval: pending.** No native attempt was repeated after the
  earlier unmapped/blank outcomes. GPUI simulated actor/focus/storage tests prove
  only those contracts. No native platform score/visual approval is invented.
- **Windows/macOS ownership and clean supported-machine/package checks: NOT RUN.**
  Launch fails closed on unqualified platforms; CI definitions were not dispatched.
- **1,000-page soak and measured native responsiveness: NOT RUN/NOT MEASURED.**
  Low-space helper tests plus real preflight wiring are not an actual exhausted-
  filesystem or competing-disk-writer qualification. Disk checks are not reservations.
- Rendered-link discovery is bounded, not exhaustive. Robots and action heuristics
  are not authorization/read-only guarantees. Navigation itself can have effects.
  Dynamic hidden/delayed states, infinite/tall/virtualized pages and existing
  image/readiness budgets retain their documented limits.
- The inherited simple pixel metric is unchanged, not perceptually calibrated.
  No login/session reuse, alternate environment mapping, image replacement,
  masks/reanalysis, selective approval, ZIP sharing or history cleanup UI was added.
  Interrupted publication can retain unreferenced artifacts; no automatic GC.
  Same-user filesystem/DB adversaries and full power-loss semantics remain outside
  local qualification. The upstream `proc-macro-error2 2.0.1` future-Rust warning remains.

## Independent-verifier handoff

Read PRODUCT/PLAN, `03-crawling.md`, decisions 0005/0006/0007/0008,
`02-reverification.md`, this report and the listed sources. Use fresh paths, keep
all historical evidence/research documents untouched, run serially and do not
implement Phase 4. The wrapper builds the real flow binary before its crash test.
Allow roughly ten minutes and several GiB for the fresh full suite on this host;
the cancellation suite adds roughly two minutes. No runtime download/global
installation/public URL/native repetition is required for these local fixtures.

```sh
# From /path/to/Regressee; choose unused names.
python3 scripts/check-phase3.py \
  --output /tmp/opencode/regressee-phase3-verifier-UNIQUE

# Use the fresh verifier pack, not a worker-2 pack. Run after the first succeeds.
python3 scripts/check-phase3.py --runtime-only --active-cancel-only \
  --retained-flow /tmp/opencode/regressee-phase3-verifier-UNIQUE/product-flow \
  --output /tmp/opencode/regressee-phase3-verifier-cancel-UNIQUE

# Optional budgeted completion gate; NOT RUN by the implementer.
python3 scripts/check-phase3.py --runtime-only --scale-pages 1000 \
  --retained-flow /tmp/opencode/regressee-phase3-verifier-UNIQUE/product-flow \
  --output /tmp/opencode/regressee-phase3-verifier-soak-UNIQUE
```

For a bounded retained-pack repeat of the full suite, add
`--retained-flow "$PWD/artifacts/phase3-implementation-9190a5be4a89/runtime-c/product-flow"`
to the first command. It verifies signature/inventory and exact maintained worker
bytes; it still performs actual captures, not retained-image substitution.

Independently probe URL/query/hash/redirect policies before destination visits,
robots unavailable/override behavior, rate-limit/retry caps, per-origin spacing
and the global permit; distinguish source inspection from measured scheduling.
Exercise preview inclusion and coverage, incomplete activation rejection,
failed/missing-profile analysis guards, owner-claim races and live/dead recovery,
settings/runtime continuity, comparison-only fixed inventory, immutable originals,
legacy-record migration and low-space stops. Check real GPUI action wiring and
busy Results keyboard controls separately from native rendering. Audit owned
processes/scratch and preservation again without unscoped kills/deletes.

Native verification is a separate gate only on a meaningful available desktop;
stop on unmapped/blank, never loop black-window probes. Write a new
`docs/phases/03-verification.md` with the actual scoped independent verdict and
update the index. Do not infer a public/platform/native/soak pass from these results.
