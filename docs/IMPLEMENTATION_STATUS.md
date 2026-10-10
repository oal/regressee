# Regressee implementation status

Updated: 2026-10-10

The user requested continuation after approving Python controls. See
`docs/decisions/0005-serial-local-continuation.md` for the current execution
policy. Development continues locally; unrun release/platform checks are not
reported as passed. All delegated agents use `openai/gpt-6.1-sol#high`.
The user subsequently allowed multiple agents where useful; see
`docs/decisions/0006-controlled-parallel-delegation.md`. Parallel work requires
independent scope and non-overlapping ownership; dependent safety repairs still
wait for independent verification.

| Phase | Implementation | Independent verification | Remaining qualification |
| --- | --- | --- | --- |
| 0 — feasibility | Linux spike and local corrections implemented | Backend/correction passes; native recheck failed/open | Native rendering/input, external OS/clean-machine, secure distribution/persistence and release stack qualification |
| 1 — foundation | Local implementation and bounded P1/P2 corrections delivered | All three repairs independently verified locally: 49 Rust + 4 Node tests, 30 unsafe-path probes, 16 paired session cases, fresh real-storage/simulated-GPUI sequence and headless persistence pass | Human visual review, meaningful native rendering/input, external OS/clean-machine/accessibility and release checks remain open; not fully accepted |
| 2 — vertical slice | Real local product loop and bounded P1 short private Linux scratch-namespace correction delivered | Earlier independently verified core retained. **P1 independently re-verified fixed locally:** 60 locked Rust + 6 signed-worker tests; fresh sandboxed long/short capture/cancel/worker-and-app-crash recovery and surviving-browser recapture; independent default-parent relative-root capture/shutdown, actual Chrome-PID cleanup and 11 unsafe-parent/recovery-preservation probes. **Dependent local Phase 3 supervisor work may proceed** | Not full acceptance: supported public runtime/signing-host, native rendering/input/human review, Windows/macOS ownership and clean supported-machine gates remain open; historical runtime stays owned-loopback-fixture-only |
| 3 — crawling | Guided multi-page/profile discovery, immutable inventories, resumable queues and fixed-baseline checks delivered; bounded redirect/robots/visible-selection and native-network resource corrections added | **Combined document/resource boundary independently accepted locally; resource P2 fixed and original three repairs retained.** Fresh 70 locked Rust + 17 signed-worker/browser passes; exact retained policy suite + 8 Node passes; exact robots/visible-ID tests; fresh independent concurrent-loopback 31-case matrix confirms real cross-origin script/CSS/image loading, all five redirect statuses, zero forbidden document/resource visits, exact final provenance and native-pause IPC Cancel/Shutdown (9–46 ms). Both fresh signed packs match source/runtime hashes. **Dependent local Phase 4 may rely on the corrected crawler**; historical 100×3/storage/queue evidence retained | 1,000-page/3,000-target soak NOT RUN; native responsiveness/rendering/input/human review, qualified public runtime/signing/hosting, Windows/macOS ownership and clean-machine gates open; not full acceptance or public resource qualification |
| 4 — auth/capture | Local authentication/capture, immutable origin/access binding and bounded P2 authenticated Forget ownership corrections delivered (`04-corrections.md`, `04-forget-corrections.md`) | **Original P1 and corrected Forget P2 independently FIXED LOCALLY.** Fresh **7 auth + 3 vault + 85 locked Rust + 33 signed Node PASS**, actual Product auth/Forget regression PASS; **26 independent Product/SQLite/AEAD controls** confirm copied/forged/stored-owner/header tamper refusal, A bytes/hash/reference/replay preservation, legitimate expired/changed-origin Forget, unrelated/key preservation and retained zero-request P1. Scoped fresh-evidence/resource audit PASS, zero remaining owned processes/scratch/pending files. **Dependent local Phase 5 auth/capture/session integration may proceed** (`04-forget-verification.md`) | Human/rendered GPUI login and visual approval, real OS-key persistence (fresh check unavailable, NOT QUALIFIED), platform ownership, Chromium proxy/OS trust/custom CA/VPN/enterprise and supported public-runtime/release gates remain open; owned valid-CA download and Chromium certificate-rejection tests are local only; not full acceptance |
| 5 — analysis/review | Local analysis/review/immutable approval implemented; two bounded P2 confirmation-context and retained-tile transform corrections delivered (`05-corrections.md`) | **Both P2s independently FIXED LOCALLY; bounded local review/storage dependency gate PASS** (`05-reverification.md`). Fresh **5 focused services + 3 simulated-GPUI + 114 locked Rust + 34 signed Node + 2 actual Product flows PASS**; independent context/Jobs bypass and 135 containing-tile affine/input cases check 1,008,288 real buffer-pixel occurrences. Stale confirmations add zero baselines; fresh explicit acceptance preserves exact inherited references/history/provenance. **Dependent local Phase 6 may rely on these contracts.** One nonblocking **P3 tiny-mask minimum-border footprint OPEN**; correct PNG rectangle/origin, not exact scaled outline below 4 viewport pixels. Serial audit/cleanup settled | Fixture-only presets; no full native/human/accessibility, maximum-scale, real keystore, external OS/public runtime/release qualification. Tiny-mask display precision remains limited; carried into Phase 6 |
| 6 — portability/storage | Selected/composite ZIP v1, transactional new-project import, imported provenance, GPUI/history/manual storage safety; SQLite v8. Bounded shared strict-PNG/P3 nested-schema repairs delivered (`06-corrections.md`) | **P2/P3 independently FIXED LOCALLY; bounded local dependency gate PASS** (`06-reverification.md`). Fresh serial **22 focused Rust + 33 signed Node**, locked build/clippy/fmt PASS; independent exact-cause PNG/schema/geometry/compression and activation/restore/export/selected/inherited refusal probes preserve rows/CAS/prior active pointers. ONE final-source composite → new project → separate headed synthetic recipient Open/ready → mapped frozen TWO-target check PASS in **121.350 s**, unchanged **240 s** bound, two successful captures/clean analyses, original600/future640. Independent persisted-data/source/privacy/runtime/cleanup audit PASS; all checks settled. **Dependent local Phase 7 may rely on corrected contracts** | Native GPUI/human/accessibility/cross-OS/real-keystore/current public-runtime/maximum-scale/physical-disk/release remain PENDING / NOT RUN, not blanket local-development halts. Synthetic headed login/test vault is not human/native/OS-key qualification. Historical runtime literal-loopback-only. Masks do not redact raw pixels; private queries remain visible. Phase 5 tiny-mask P3 carried; Phase 7 not started here |
| 7 — release hardening | Original deliveries/sole3000-target implementation soak retained (`07-status.md`, `07-corrections.md`). Separate THREE-finding repair delivered (`07-guardian-corrections.md`): both raw clone paths, trusted v5 launch journal/identity receipt, complete old-CAS compaction hashes without per-target full scans | **THREE repairs independently FIXED LOCALLY; scoped correction gate ACCEPTED** (`07-guardian-verification.md`). Fresh143 Rust/20 deliberate ignores,2 signed ownership/full actual Product,8 original refusal controls +18 actual journal/crash controls,60 atomic hash negatives and700 differential boundaries PASS; UI build/clippy/fmt PASS. ONE independent1000×3 soak PASS:3000 actual captures/visits,1000 paused/reopened resume, baseline activation896.790 s; total1114.862 s including1000 captured/2000 cancelled second run, unchanged1200/1500/1560 s budgets. Final source/history/CAS/privacy/resource audit PASS; all owned checks settled, zero forced/rescue/deadline/remaining children. Prior failures preserved | **Not full Phase7/release accepted.** Historical ResourceFailed root cause OPEN; old candidate STALE, NOT corrected binary binding. macOS/accessibility code gaps; full Windows integration UNCOMPILED; native Vulkan/GPU/input/UI latency NOT RUN. Historical GLIBC_2.43 ABI failure; current public-browser/trust-hosting, real OS keystore, human/package/signature/key-custody/licensing gates remain open, not blanket local halt. Guardian RSS separately observed nonzero; sampled peaks are not exact/unique physical memory, parent IO includes waited children and is not exclusive/additive |

## Current evidence

### Renderer-side capture and Setup timing — latest continuation

**Readable screenshots now work:** a default-off diagnostic feature uses the
pinned GPUI/Blade renderer, real Linux Cosmic fonts and the actual glyph/image
atlas without a compositor or OS-window launch. Calibrated GPU readback passed;
the first synthetic batch produced **28 app images across all five screens at
both sizes**. The bounded correction confirmation stopped after nine wide images
on a remaining timezone-chevron geometry fault; login-help overlap also remains.
Whole-UI acceptance is still OPEN, not a rendered-frame/input PASS.

The dominant **repeat** Setup cost is now demonstrated: GPUI discarded its text
geometry between screens. Production explicitly enables bounded window-local
unwrapped-layout retention. Final real-font debug A/B shows **317–420 ms →
22–36 ms** CPU draw on repeat navigation; cold draw remains **302–396 ms**.
The complete user's ~1s native delay and OS presentation/input remain unqualified.
Independent source review passes renderer/cache; its extent-overflow P2 was
fixed and independently retested without GPU initialization. See
`renderer-capture-results-20261010.md`, `setup-render-profile-20261010.md` and
`renderer-and-cache-source-verification-20261010.md` under `docs/research/`.
No native relaunch, GPU policy change, compositor helper or public runtime
distribution is supplied by this work. Existing platform/release gaps below remain.

### Previous native-window correction round — historical scoped evidence

**The native UI sweep is unfinished.** Later user screenshots identify severe
layout/affordance defects beyond the earlier focus fix. The collapsed Import
label was reproduced in GPUI test-platform bounds and narrowly repaired; one
private after-capture stayed black and the actual desktop X11 attempt failed
before mapping. Neither qualifies the UI. A new correction batch targets compact
filled actions versus white inputs, balanced choice padding, attached feedback
dismissal, a timezone selector and automatic first-use runtime scheduling.
Ownership/current handoffs are in `docs/ACTIVE_WORK.md`. The new serial local
gate passed **160 tests / five deliberate ignores**, Clippy with warnings denied,
and a locked offline UI build. The inspection-feature follow-up passed 67 app
library/shell/protocol tests / three deliberate ignores, plus Clippy/build. These
are not native visual/input acceptance. The user's authorized exact-owned-window
KWin round exited 101 before capture with dmabuf import failure /
`ERROR_SURFACE_LOST_KHR`. Zero PNGs/stages/input; all 33 states and Setup timing
comparison NOT RUN. Helper/service removal, owned-process settlement and frozen
source/binary/synthetic SQLite/CAS preservation were independently confirmed.
No retry/fallback or renderer fix is claimed. Native acceptance remains blocked.
Previous passing checks below remain historical scoped evidence.

The runtime diagnosis proved two product gaps: no embedded trusted public
distribution **and** no installation prerequisite on Find pages. The missing
prerequisite is now implemented for eligible new unpinned work and independently
verified with **7 Product tests, 3 isolated Jobs tests and 5 public-API probes**.
Comparison/resume pins, no-auto-repair, cancellation and actual byte progress
guards passed in backend/inert-fixture scope. See
`docs/research/automatic-runtime-setup-gap-20261010.md`. Default public browser
setup is not delivered by this build; requiring end users to configure a
developer distribution is not the promised automatic setup workflow.

Capture timezone now uses a bundled searchable native selector, not free text.
Unknown saved values are retained; catalog/ICU compatibility is not universal.
At that earlier checkpoint, the user's ~1s debug Setup navigation was undiagnosed. Optional
constant-label timing markers separate construction, layout/prepaint and CPU
scene paint; they do not measure GPU-visible or real OS-input latency.
The latest real-font diagnosis and repeat-navigation repair are documented above;
they do not retroactively qualify the failed native-window round.

- `docs/research/ui-controls-correction-20261010.md` — current shared control,
  feedback, selector integration and local regression scope.
- `docs/research/ui-controls-correction-source-verification-20261010.md` —
  independent source-only PASS, no material P1/P2 fault; frozen source unchanged.
  No native rendering/input or latency conclusion follows from this review.
- `docs/research/timezone-select-20261010.md` — selector/catalog provenance and
  compatibility limits; parent integration checks supersede its unrun handoff.
- `docs/research/automatic-runtime-setup-implementation-20261010.md` — actual
  first-use/backend readiness/progress APIs, not public default provisioning.
- `docs/research/automatic-runtime-setup-verification-20261010.md` — independent
  scoped backend PASS; real browser/platform/public runtime qualification open.
- `docs/research/ui-owned-wayland-inspection-20261010.md` — feature-gated finite
  synthetic-scene inspection and exact-PID helper; one actual round failed early,
  zero images/input; settled cleanup and explicit timing/visual limits.

The latest scoped Linux repair and 1,000-page/3-profile scale gate passed
independently in `07-guardian-verification.md`; its agents and suites are settled.
This does not close Phase 7. Separately, the owned-window attempt in
`docs/research/native-render-on-disk.md` executed and failed at Vulkan swapchain
creation; native rendering/input is not merely an unattempted backend check.
The old private candidate is stale. Remaining code gaps and qualification gates
are distinct from the accepted local correction contracts.

The user subsequently requested a full UI sweep after reporting plain launch
success and showing focus-induced button growth/unclear setup controls. The
sweep is implemented locally; **native visual/input acceptance remains blocked**:
explicit choices and selected states, stable paint-only focus, familiar input
selection, disjoint History focus handles, one atomic project/environment save,
top-level feedback/pinned setup actions and refinements to all five screens.
Parent final scoped checks passed **115 tests / four deliberate ignores**, Clippy and
locked offline UI build. User-reported startup success does not explain the
historical GPU failure; no renderer fix is claimed. The independent verifier's
initial 22 targeted passes found two source issues; one correction batch and
**26 independent targeted passes** now close both. Its isolated X11/llvmpipe
window naturally mapped but captured entirely black pixels, so no input or
five-screen visual review was performed. The corrected binary was not natively
relaunched. All work processes are settled; this is not full UI/human acceptance.

- `docs/research/ui-sweep-audit-20261010.md` — source-only discovery, not acceptance.
- `docs/research/ui-sweep-implementation-20261010.md` — delivered full-screen
  changes, serialized local checks, preserved implementation failures and source freeze.
- `docs/research/ui-controls-sweep-20261010.md` — exact GPUI text-refinement cause,
  stable shared action/choice/check implementation and authored regression handoff.
- `docs/research/ui-input-sweep-20261010.md` — native input implementation and
  known segmentation/IME/accessibility limits; source tests are not OS acceptance.
- `docs/research/ui-sweep-verification-20261010.md` — initial independent source
  checks, two findings and retained black owned-window native evidence.
- `docs/research/ui-sweep-reverification-20261010.md` — final scoped PASS for both
  corrections, 26 targeted tests, frozen-source/evidence/cleanup audit; native
  visual/input acceptance remains blocked, not silently substituted with simulated tests.

- `docs/phases/07-guardian-verification.md` — FINAL independent THREE-repair local
  acceptance and ONE final-source maximum-scale PASS. Fresh physical private paths,
  verified signed worker5/13 source inputs; complete raw-child/signal/descriptor/
  early-parent-death/kernel-ownership probes,8 original refusals +18 journal/crash
  controls,60 hash refusals/700 differential boundaries;143 Rust/2 signed faults/
  actual Product/UI build/clippy/fmt PASS.3000 captures/visits,1000 paused/reopened
  resume, complete activation896.790 s then1000 captured/2000 cancelled; total1114.862 s,
  unchanged budgets. All final8365 CAS hashes/19565 refs verified, actual activation
  strictly validates3000 originals,48 additional original/derivative PNG decodes.
 71 fresh/eight historical stores, source/history/signatures/privacy/resources audited;
  no implementation edits, nested agents, publication, forced cleanup or retry.
  Historical ResourceFailed remains OPEN, candidate STALE, native/platform/public/
  real-keystore/human/licensing gaps OPEN; NOT full Phase7/public release acceptance.

- `docs/phases/07-guardian-corrections.md` — separate bounded implementation of the
  three final reverification findings: raw async-safe Linux launch and kernel ownership,
  independent bound journal/guardian-held commitment and identity receipt/fail-closed
  recovery, complete old-CAS compaction hashing. Latest local143 Rust/2 signed ownership/
  actual Product PASS;8 original negative controls,48 independent hash refusals and700
   differential boundaries; UI build/clippy/fmt and final scoped source/history/runtime/
   CAS/privacy/resource audit PASS. All70 bounded checks settled, zero deadlines/forced/
   remaining children;22 natural adoptions. No fresh soak.
  Independent acceptance was PENDING at that implementation handoff; the subsequent
  FINAL independent report above accepts these bounded repairs. Historical readiness
  cause OPEN, candidate STALE; the implementation evidence is not reattributed.

- `docs/phases/00-status.md`
- `docs/phases/00-reverification.md`
- `docs/phases/00-image-verification.md`
- `docs/phases/00-native-recheck-2026-10-09.md`
- `docs/phases/01-status.md` — implementation evidence and independent-verifier handoff
- `docs/phases/01-verification.md` — fresh independent behavior probes, local safety repair verdict and native failure evidence
- `docs/phases/01-corrections.md` — implementation record of three bounded repairs, exact-cause regressions and serial locked checks
- `docs/phases/01-reverification.md` — fresh independent confirmation of all three repairs; Phase 2 local continuation permitted, native/external/human gates remain open
- `docs/decisions/0006-foundation-boundaries.md`
- `docs/phases/02-status.md` — actual product implementation, serial tests, native failure, resource accounting and independent-verifier commands
- `docs/phases/02-verification.md` — fresh independent real-product/CAS/viewer and trust probes; fixed-scope local core passes, P1 inherited long-TMPDIR startup failure requires bounded repair; external/native/public-runtime gates remain open
- `docs/phases/02-corrections.md` — historical bounded P1 implementation correction and fresh-verifier handoff
- `docs/phases/02-reverification.md` — fresh independent P1 repair confirmation, actual default long/short relative-root captures, cancellation/crash/recovery and preservation probes; no material local safety blocker; dependent local Phase 3 permitted with external/native/public/platform gates open
- `docs/decisions/0007-phase2-runtime-evidence.md`
- `docs/phases/03-status.md` — Phase 3 implementation, exact latest-source checks, actual 100×3 captures/resource accounting, unrun soak and independent-verifier handoff
- `docs/phases/03-verification.md` — independent real 100×3 reproduction, scheduling/profile/queue/CAS/migration/GPUI evidence; two reproduced P1 navigation/robots defects and P2 filtered-preview selection require bounded corrections before dependent Phase 4
- `docs/phases/03-corrections.md` — bounded pre-follow redirect interception, robots wildcard/anchor and stable visible preview identity repairs; maintained serial tests and independent-reverification handoff
- `docs/phases/03-reverification.md` — fresh independent confirmation of all three original repairs; new reproduced P2 allowed cross-origin resource stall keeps combined capture-boundary acceptance open before Phase 4; pending external gates are not a blanket stop
- `docs/phases/03-resource-corrections.md` — bounded Chromium native-network resource repair; implementer signed capture/discovery/IPC, resource loading, zero forbidden visits and cancellation evidence; its then-pending independent combined-boundary recheck is recorded next
- `docs/phases/03-resource-verification.md` — fresh independent combined-boundary acceptance: actual positive resources, zero forbidden visits, retained Rust-owned policy/robots/selection, signed-pack/source checks, independent native-pause IPC Cancel/Shutdown and scoped ownership audit; local Phase 4 may proceed, public/native/OS/soak gates remain pending
- `docs/decisions/0008-phase3-crawl-identity-and-queues.md` — versioned URL identity, conservative policy/scheduling, immutable coverage and resume ownership
- `docs/crawling-surface.md` — native guided setup/results extension; rendered/human qualification remains pending
- `docs/research/release-qualification.md` — completed read-only packaging research; current upstream versions are metadata/recommendations, not qualified pins
- `docs/phases/04-status.md` — Phase 4 implementation evidence, explicit failed/interrupted runs, qualification limits and fresh independent-verifier handoff
- `docs/phases/04-verification.md` — independent Phase 4 local verification, one actual headed synthetic auth/capture service lifecycle, fresh crypto/ephemeral/storage/mapping/expiry/resume/TLS/data audits and reproduced P1 saved-session origin drift; dependent auth/capture integration needs bounded repair/recheck
- `docs/phases/04-corrections.md` — bounded P1 immutable origin/access binding, fail-closed legacy state, two-owned-origin real Product/worker regressions, serial implementer evidence and pending independent correction handoff
- `docs/phases/04-reverification.md` — independent original P1 fixed-local verdict, fresh exact zero-request Product/worker/restart/new-login probes and preservation audit; new P2 copied encrypted-reference Forget deletes original session ciphertext, requiring bounded repair/recheck before dependent integration
- `docs/phases/04-forget-corrections.md` — bounded P2 authenticated actual-owner deletion/pruning and ephemeral refusal; settled serial implementer results (7 auth/3 vault/85 locked Rust/33 signed Node and actual Product PASS), exact counters/preservation, zero live owned resources, explicit missing/locked-store semantics and fresh-verifier handoff; its then-pending independent verdict is recorded next
- `docs/phases/04-forget-verification.md` — fresh independent P2 fixed-local verdict; actual Product/SQLite/AEAD copied/forged/stored-owner/header controls, exact zero-request P1, original-owner replay/legitimate Forget/unrelated-key preservation and serial maintained checks pass. Dependent local Phase 5 integration permitted; native/OS/public/platform/release gates still pending
- `docs/decisions/0009-phase4-session-and-environment-boundaries.md` — private owner, encrypted/ephemeral storage, explicit mappings and capture/network boundaries
- `docs/auth-and-capture.md` — user-facing login/consent/mapping/read-only-account and capture/network limitations
- `docs/auth-and-capture-surface.md` — GPUI extension contract; simulated actor evidence does not imply native/human approval
- `docs/research/analysis-engine-handoff.md` — finished standalone/unwired engine; unchanged by the Phase 4 correction
- `docs/research/analysis-engine-verification.md` — independent standalone contract PASS, not Phase 5 integration/completion; unchanged by the Phase 4 correction
- `docs/phases/05-status.md` — Phase 5 local integration, saved-image pointer/service tests, immutable approval/history, preserved failures and independent-verifier handoff
- `docs/phases/05-verification.md` — fresh independent serial/integration/GPUI/photo/storage audit; two reproduced P2 stale-approval/tile-seam UI defects require bounded repairs/recheck before dependent local Phase 6 treats integrated review as accepted; immutable storage/history/auth boundaries pass, native/human/external gates remain pending
- `docs/phases/05-corrections.md` — two bounded P2 implementation repairs, actual actor/backend stale-confirmation and saved-pixel/seam-layout regressions, final serial 114 Rust/34 Node/two Product passes and settled preservation/cleanup audit; independent correction reverification was pending at delivery
- `docs/phases/05-reverification.md` — fresh independent P2 fixed-local verdict, exact focused/serial reruns and additional actual Store/Jobs context/bypass plus saved-buffer/containing-tile/pointer oracles; one nonblocking P3 tiny-mask border footprint open. Dependent local Phase 6 may rely on exercised review/storage contracts; native/human/external gates remain pending
- `docs/research/phase5-presets.md` — exact versioned fixture-calibrated policies, numerical tradeoffs and unqualified real-site accuracy
- `docs/analysis-and-review-surface.md` — proposed GPUI extension and truthful native/human finish disposition
- `docs/phases/06-status.md` — Phase 6 local services/GPUI/security/fault implementation, serial evidence, explicit limits and independent-verifier handoff
- `docs/phases/06-resumed-verification.md` — resumed independent verdict/evidence: reproduced P2 accepted malformed PNG active baseline, P3 nested schema gap, retained passing stages, bounded recipient failures and final ownership/preservation audit
- `docs/phases/06-corrections.md` — bounded P2/P3 repair, exact atomic refusal/unchanged-original regressions, reproduced fixture starvation and one successful actual complete headed synthetic recipient flow, focused settled checks/audit and fresh independent final-source reverification handoff
- `docs/phases/06-reverification.md` — final independent P2/P3 fixed-local verdict, fresh exact-cause/refusal/preservation probes and ONE final-source complete recipient PASS within unchanged 240 s; scoped persisted-data/runtime/privacy/ownership audit. Dependent local Phase 7 reliance permitted; external/native/human/OS-key/soak gates remain pending
- `docs/phases/07-status.md` — final local hardening/private-candidate evidence, exact passed/failed/unrun scopes, failed actual maximum-scale coverage, remaining code/platform gates and fresh independent-verifier commands; not independent acceptance
- `docs/phases/07-verification.md` — complete independent verdict: verified local runtime/privacy/diagnostic/tiny-mask/recipient/candidate contracts, reproduced P1 Linux ownership failure, open actual replacement/readiness regression, failed single disk-backed2188-capture soak/ABI gates, preserved history and settled resources; Phase7/release NOT ACCEPTED
- `docs/phases/07-corrections.md` — FINAL bounded implementer Linux guardian/checkpoint corrections and local PASS;139 Rust/35 signed Node/2 ownership/full Product, sole changed-source3000-target soak PASS within unchanged budget, final read-only audit and fresh independent handoff. Historical readiness root cause OPEN; corrected-source independent acceptance pending; candidate stale
- `docs/phases/07-reverification.md` — FINAL independent correction verdict: original stopped-browser P1 fixed in tested scope and linear delta/signed worker/Product contracts pass; new P1 post-fork inherited-handler hazard, P2 recovery receipt/refusal and compaction hash boundaries fail.700 differential/refgraph boundaries,139 Rust/35 signed Node/2 ownership/full Product; zero fresh soaks/rescues, final scoped audit and all resources settled. Next bounded repair/recheck; Phase7/release NOT ACCEPTED
- `docs/first-run-guide.md`, `docs/support-matrix.md`, `docs/updates-and-recovery.md`, `docs/release-building.md` — implemented user/private-release documentation; no platform/public/human approval inferred
- `docs/portable-baseline-archive.md` — archive v1 schema/privacy/limits, safe recipient setup and manual history/storage/recovery contract
- `docs/portability-and-storage-surface.md` — preserved native working-desk extension; simulated action tests do not imply native/human acceptance

Phase 1 implemented actual project persistence, migration/artifact storage and
native shell contracts only. Its separate fixture bench was not a product run.
The independent verifier found a
local database-path safety defect: a pre-existing dangling database symlink was
followed and its outside target created/migrated, plus session-contract and
stale-list issues. All three bounded repairs now pass fresh independent behavioral
re-verification and locked regressions. Original findings remain preserved,
not relabeled as historical passes. No material local blocker was found in this
recheck. Phase 2's fresh independent verification now passes the real Linux
product loop with a short temporary parent, including actual saved before/after/
diff pixels and simulated-GPUI actions. It also reproduced a P1 worker startup
failure under a valid long inherited TMPDIR. That bounded repair is now independently
re-verified: ordinary long/short inherited environments work with the validated
short Linux namespace; cancellation/crash recovery preserves unrelated processes
and retained evidence. No material local safety blocker remains in this recheck,
so dependent local Phase 3 may proceed under decision 0005. Invalid scratch overrides
and unqualified operating systems still fail closed. Real services are wired to GPUI, but the exercised historical pack is
development/owned-loopback-only and public-site capture remains refused pending
secure runtime qualification. The implementer's native attempt stayed unmapped;
the verifier did not repeat it or invent visual approval. That was the Phase 2
scope; Phase 3 is now implemented locally as recorded below. Native/external/human
and release gates remain open; completed packaging research does not qualify a
current runtime.

Phase 3 extends the real product services and GPUI controls with bounded rendered-
link discovery, explicit origins/robots policy, multi-profile immutable inventories,
pause/cancel/crash-resume and fixed-target comparison. Its development worker is
explicitly version 3.0.0; historical worker-2 evidence remains retained. Actual owned-
loopback verification captured 100 distinct pages × 3 profiles (300 visits) in
105.09 s, rejected incomplete activation and failed-target unchanged accounting,
preserved crash/pause evidence and made zero visits to a newly added page during
checks. Pre-correction locked checks pass 68 Rust tests; the full browser suite predates
small targeted Rust/UI deltas documented and separately checked in `03-status.md`.
Those historical results are implementer evidence, not native approval. Fresh
independent verification now reproduces **300 actual visits/captures in 98.108 s**
and validates all 300 PNG originals, plus authoritative saved-project actions,
exact browser profiles, scheduling/backoff and permission-fault queue recovery.
It also reproduces two **P1** defects: HTTP redirects reach robots-excluded and
unapproved destinations before policy rejection, and terminal `*$` robots rules
permit forbidden descendants. A **P2** filtered-preview action toggles a hidden
inventory row. Bounded repairs now use single-response redirect interception,
correct wildcard/end-anchor matching and stable visible preview page identities;
implementer checks pass 70 locked Rust tests, 8 signed-worker Node tests and
focused real-browser zero-request/allowed-chain regressions. See
`03-corrections.md` for implementation tests/results. Fresh independent correction
reverification in `03-reverification.md` confirms all three original repairs,
including real zero-request chains, 15,264 robots cases and async visible-ID/frozen
selection. It also reproduces a new **P2**: allowed cross-origin loopback resources
stall with document fetch/fulfill interception; real captures time out while
sandboxed raw/ordinary-route controls succeed. The bounded implementer repair in
`03-resource-corrections.md` now preserves Chromium's real document network/address-
space semantics and intercepts document response headers and resource redirect
requests. Fresh signed tests pass actual resource execution/loading, zero forbidden
destination visits, truthful diagnostics, final provenance and cancellation;
70 locked Rust tests and the original policy suite pass too. Fresh independent
verification in `03-resource-verification.md` now accepts the **combined document /
resource boundary locally**: both fresh signed packs match maintained source and
runtime hashes, the exact policy/robots/visible-selection regressions pass, and
31 independent concurrent-loopback cases confirm real resource execution/loading,
all five redirect statuses, zero forbidden destinations, exact final provenance
and actual IPC Cancel/Shutdown during native request/response pauses. Diagnostic
fixture failures remain preserved, not relabeled passes. **Dependent local Phase 4
may rely on this corrected crawler** under decisions 0005/both 0006; this is not
public runtime/resource or release qualification. Proven storage/queue evidence
is retained; no new scale audit is required.
The required 1,000-page soak remains explicitly NOT RUN, and native/public/platform/
human gates remain open. That Phase 3 verification introduced no Phase 4+ work or nested agents.

Phase 4 now implements environment-specific login/consent confirmation, OS-keyed
encrypted persistence interfaces and explicit ephemeral sessions, separate local
production/staging references, authenticated discovery/capture, expiry pause and
re-login resume. Explicit prefix/origin mappings preserve the fixed logical baseline
and require representative probes; site mapping is not rendering incompatibility.
Capture readiness/tall-page limits and current rendering fingerprints produce honest
warnings/caution. Local tests use signed worker 4.0.0 on owned literal-loopback,
with explicit injected-key/headless fixture boundaries. OS secure storage is
unavailable on this host; native/human, enterprise, platform and public-runtime
qualification remain pending. Independent Phase 4 verification now confirms these
bounded local behaviors, including one actual headed Rust login-ready-capture
service lifecycle with a self-establishing synthetic account and default-owner
Ephemeral; this is not native GPUI/human/visual acceptance. It also reproduces a
**P1 origin-binding defect**: retargeting one existing environment ID can send old
cookie-only login state to a new same-host/different-port origin, pass probing and
complete mapped captures without a fresh login. **Dependent Phase 5 auth/capture
integration must wait for bounded repair and independent recheck**, not merely the
passing ordinary suites. Independent non-overlapping analysis work can continue
under the existing delegation policy. No Phase 5 feature or parallel analysis-engine
integration was added by this verifier. See `docs/phases/04-verification.md`.

The bounded Phase 4 origin-drift correction is now delivered in
`docs/phases/04-corrections.md`: the original normalized destination-origin set and
access policy are authenticated inside each newly established encrypted session
(also retained for ephemeral state), checked against current saved settings before
visits, and independently checked against worker material at installation. Legacy
or incompatible sessions require re-login without migration, plaintext fallback or
evidence deletion. Its fresh implementer checks are **not an independent verdict**.
Final corrected source passes 82 locked Rust and 33 freshly signed Node tests,
the actual Product origin-drift regression and scoped data/runtime/ownership audit;
the real OS-store test still reports unavailable, not qualified persistence.
That implementer handoff held dependent integration for a distinct verifier. The
completed standalone analysis engine and independent audit stay unwired/unchanged;
no Phase 5 integration was done by this correction implementer.

Fresh independent correction reverification on 2026-10-10 now confirms the original
P1 **fixed locally**: probe/discovery/fixed capture with old A credentials after a
same-environment B edit returns exact NeedsLogin with zero B requests, including
after encrypted restart; direct signed-worker refusal and a new intended B login
also pass. Fresh 5 focused, 82 locked Rust and 33 signed Node tests pass, with an
independent data/runtime/resource audit and unchanged original baseline/history.
However, a new **P2** is reproduced: a copied machine-local encrypted reference
cannot decrypt/replay under a wrong project/environment, but that owner's Forget
deletes the original scope's ciphertext. This precondition is an explicit local
reference transplant, not an ordinary environment edit or portable import. No
credential leakage or screenshot/baseline deletion occurs; ephemeral copied-reference
Forget correctly preserves the original owner. **Overall correction verdict remains
OPEN, and dependent local Phase 5 auth/capture/session integration should wait for
this bounded deletion-ownership repair and independent recheck**. The original P1
is not relabeled unfixed, prior dates/reports remain historical, and external gates
remain honestly pending. See `docs/phases/04-reverification.md`.

A bounded P2 implementation is now delivered in `04-forget-corrections.md`:
encrypted removal authenticates immutable identity AAD using the actual selected
project/environment, and ephemeral removal checks the trusted in-memory owner.
Copied/forged references cannot revoke the original owner's state; locked or corrupt
state fails closed. Valid owners can Forget even after access expiry or origin/access
changes. Missing material permits only local-reference recovery; installation keys
are never removed by Forget. Fresh focused **7 auth + 3 vault**, **85 locked Rust**
and **33 signed Node** tests pass; the actual Product regression completed with
exit 0 in 371.288 s, including all new Forget controls and retained P1 checks.
The same serial suite is now fully settled: 98 own children reaped, zero remaining
owned product processes, scratch entries or pending files. Original baseline/session
preservation and scoped source/independent-history fingerprints pass. The actual
OS-store check again reports unavailable, not qualified persistence.
This is an implementer result, **not fresh independent acceptance**. The original
P1 and its evidence remain retained; dependent integration waits for the new P2
recheck. No analysis integration or external/native/OS qualification was added.

That bounded P2 recheck is now independently complete in
`docs/phases/04-forget-verification.md`: copied and forged references cannot revoke
another identity's encrypted or ephemeral state. Fresh stored ownership/header
tampering controls confirm deletion authority comes from authenticated identity
AAD/trusted memory, not caller or payload owner fields. Actual owners can Forget
expired/changed-origin/legacy state; unrelated sessions and the injected installation
key remain usable. Fresh serial **7 auth + 3 vault + 85 locked Rust + 33 signed Node**,
actual Product regression and **26 independent controls** pass; zero unauthorized
drift requests, preserved original evidence and zero remaining scoped resources.
The initial verifier-only missing-baseline assertion failure is retained and not
counted as a pass. **P1/P2 are fixed locally; dependent local Phase 5 integration may
proceed.** No implementation/analysis integration or full/native/OS/public/platform
acceptance is inferred; the fresh OS-store check still reports unavailable.

Phase 5 now integrates the previously standalone audited analysis engine through
the core crate, preserving the original app re-export and historical analysis JSON.
Actual saved-image services and GPUI controls support reanalysis, geometric masks,
revision-specific review and transactional selected/inherited composite baselines,
including exact-reference rechecks and historical restore. Pre-capture
`neutral-raster-v2` and non-activatable representative previews expose supported
scope/readiness/geometry limits without storing separate hidden original photos.
The independent corpus/research evidence stays historical; numeric seed values are
now versioned as fixture-calibrated presets, not proven user accuracy. Final serial
implementer checks **PASS on corrected source**: 110 locked Rust, 34 freshly signed
Node tests and two actual Rust Product flows, UI build/clippy/fmt, unchanged corpus
and scoped freshness/preservation/ownership audit. All tests are settled; earlier
replacement and intermediate formatting failures remain preserved. **Fresh
independent integration verification was pending at that handoff**. Native/human/accessibility,
real OS-key/platform/public-runtime and
maximum-scale qualification remain open; no Phase 6 work or full acceptance is
implied. See `docs/phases/05-status.md` for exact final outcomes and retained failures.

Fresh independent integrated Phase 5 verification is now recorded in
`docs/phases/05-verification.md`. The unchanged exact serial handoff passes again:
110 locked Rust, 34 freshly signed Node and two actual Product flows. Independent
actual-storage probes verify 39 corpus analyses and every heatmap pixel, advanced
cache/mask/cancellation/review/migration invariants, mapped multi-profile composite
inheritance/recheck/restore, and an additional actual replacement geometry/style
control. Saved fresh Chromium originals support authoritative actor/simulated-GPUI
pointer masks and the real overlay slider, not native/human approval. However,
two **P2 defects are reproduced**: historical analysis switching retains a
confirmable approval for a different revision; crossing a tile seam during pointer
pan shifts saved pixels relative to capture coordinates by 512 pixels until refetch.
The local integrated review gate remains **OPEN for bounded repairs and independent
recheck before dependent Phase 6 reliance**. The exercised immutable persistence,
reference/history and retained auth/resource guards pass; no code was changed or
Phase 6 started by the verifier. Fresh source/manifest/PNG/reference/secret/resource
audit passes, with zero remaining owned processes/scratch/pending files. Prior
reports and all separately pending native/human/OS/public/release gates are preserved.

The two bounded P2 corrections are now delivered in `05-corrections.md`.
Confirmation binds the immutable project/run/baseline/analysis/selection context
and display generation, invalidates asynchronous/stale previews and checks the
same context plus active parent and exact references in the backend/transaction.
Saved image layers, masks and pointer mapping share one source/viewport transform,
anchored to the retained frame's actual tile origin rather than pan modulo 512.
Final corrected-source serial implementer checks **PASS**: 114 locked Rust, 34
signed Node and two actual Product flows, five focused service and three simulated
GPUI tests (96 seam/pan cases), build/clippy/fmt, read-only corpus and data/runtime/
preservation audit. All completion notifications were awaited; 146 descendants
reaped, zero remaining owned processes/scratch/pending files. Historical failures,
reports, originals and passing auth/session/resource/storage contracts are retained.
**At that implementer handoff, independent correction reverification remained
PENDING and the integrated gate was OPEN.** No Phase 6 or native/human/public/platform
acceptance was implied.

Fresh independent bounded correction reverification is now complete in
`docs/phases/05-reverification.md`: **both original P2s are FIXED LOCALLY**. Exact
focused and serial handoff reruns pass (5 services, 3 simulated-GPUI, 114 locked
Rust, 34 signed Node and two actual Product flows). Additional independent actual
Store/Jobs/GPUI controls reject historical/cached/async/selection/context/parent
stale confirmations and direct backend bypass without activation; fresh explicit
acceptance preserves exact selected/inherited provenance and immutable history.
Fresh patterned before/after/heatmap buffers, actual containing-tile bounds and
both-pane pointer actions pass 135 affine cases at three seams/five zooms/three
horizontal pans, including short final tiles. One ancillary **P3 tiny-mask border
footprint remains OPEN**: correct absolute PNG rectangle/origin, but the 2-pixel
border imposes a minimum 4×4 viewport-pixel outline. This nonblocking display limit
is not hidden by the final passing probe. Source/history/runtime/PNG/reference
audit passes; 146 descendants reaped, zero remaining owned processes/scratch/
pending files. **Dependent local Phase 6 may safely rely on the exercised local
review/storage contracts**, subject to existing authorization; no Phase 6 work is
started here. Full native/human/accessibility/real-keystore/platform/maximum-scale/
public-release qualification remains pending, not a blanket local-development halt.

Phase 6 now implements selected/composite-baseline ZIP interchange and explicit
manual history/storage cleanup on top of the independently accepted bounded
Phase 5 dependency contracts. Portable v1 types exclude local sessions/secret
references; strict snapshots, raw ZIP/image/hash/schema limits, rebinding and
transactional new-project publication prevent automatic execution or overwrite.
Frozen source profiles/discovery stay separate from future settings. Imported
origins are marked rather than presented as independently captured local evidence.
Privacy previews state that ignored regions do not redact original screenshots
and private URL queries remain visible. Reference-aware impact confirmations
protect retained/active/composite evidence; temporary repair and complete recovery
object deletion are separate explicit actions, never automatic retention/scan-GC.
The actual Jobs/native-picker completion paths are exercised by simulated GPUI,
not a native pixel/dialog or human acceptance claim. Final implementer serial
results and scoped preservation/resource audit are in `docs/phases/06-status.md`;
At that implementer handoff, **fresh independent Phase 6 verification remained
PENDING**. The resumed independent verifier now reproduces a material **P2**:
storage/import/restore accept correctly hashed PNG originals with excess decoded
rows and activate an unusable baseline; the existing strict analysis engine rejects
the same bytes. A narrower **P3** silently discarded nested coverage field also
violates the advertised strict schema, without execution or credential leakage.
Passing independent archive/privacy/reference/frozen-profile, manual deletion,
actual-byte, fault and simulated-GPUI Jobs coverage is retained; the full recipient
check remains unproven after bounded failures. **Dependent Phase 7 reliance waits
for bounded PNG correction and independent recheck**, not for a blanket external
qualification halt. See `06-resumed-verification.md`. The Phase 5 tiny-mask P3 and
external/native/human gates remain open; Phase 7 is not started.

The subsequent bounded implementation repair is delivered in `06-corrections.md`:
strict analysis/storage PNG validation and nested schema refusal, exact hostile
specimens and atomic all-row/file preservation tests. A reproduced owned fixture
idle-socket starvation was fixed, and one complete separately headed synthetic
recipient-ready flow now produces two successful clean analyses with historical
600 versus future 640 preserved. Focused 22 distinct Rust/33 Node plus final locked
build/clippy/fmt checks are settled. At that repair handoff, independent final-source
reverification was still pending; the historical independent reports remain unchanged.

Fresh independent final-source reverification is now complete in
`docs/phases/06-reverification.md`: **PNG P2 and nested-schema P3 FIXED LOCALLY;
bounded Phase 6 dependency gate PASS**. Serial 22 focused Rust / 33 signed Node,
locked build/clippy/fmt and additional independent exact-cause malformed-stream,
geometry/compression, schema, legacy and selected/inherited publication probes pass.
ONE complete actual recipient test passes in 121.350 s within unchanged 240 s,
with separate headed synthetic Open/ready, fresh installed runtime/stores, TWO
successful frozen captures and TWO clean analyses. Original bytes/dates, complete
metadata/provenance, width600 versus future640, session/privacy/reference and scoped
ownership/preservation audits pass. All verifier checks are settled. **Dependent
local Phase 7 may rely on the corrected contracts** under decisions 0005/both 0006;
Phase 7 was not started by the verifier. Native/human/accessibility/cross-OS,
real OS-key, current public runtime, maximum-scale/soak and release gates remain
separately PENDING / NOT RUN, not blanket local-development halts. Phase 5's tiny-mask
display P3 remains open; historical browser use remains literal-loopback-only.

Phase 7 local delivery exists in `07-status.md`. At that implementer handoff,
the tiny-mask outline used
exact inside-bound painting and retained transform/seam regressions pass, but a
fresh independent visual/geometry check is still required. Windows atomic Job
API scaffolding passes; full scratch/supervisor conditional integration stopped
before compilation on a missing offline target dependency and actual OS behavior
is NOT RUN. macOS ownership and screen-reader backend remain code gaps, not mere
test-only qualifications. Native availability batch made zero app/input attempts.
Maximum-scale gate was attempted and failed at its unchanged budget; its complete
coverage must not be called NOT RUN or PASS. All 1000 paused captures survive
reopened-Store resume exactly and the partial run cancels cleanly without baseline
activation. Candidate packaging/integrity passes but required GLIBC_2.43 exceeds
the target floor; all 544 normal Linux dependency declarations remain unapproved.
All owned checks are settled; private candidates, failures and historical runtime/
reports are preserved. No publication, owner key custody, service setup or nested
agents were used. These are retained implementer results, not the independent verdict.

Fresh independent verification is now complete in `docs/phases/07-verification.md`.
Substantive bounded local archive/trust/path/privacy/diagnostic/Jobs and tiny-mask
contracts pass, including48 independent actual-API operations and96 real drawn
selectable-zoom mask cases; the prior minimum-border P3 is fixed locally. Fresh
132 Rust/14 ignored,34 signed worker and32+3 helper tests pass. ONE complete separate
portable recipient passes126.630 s within the unchanged240 s bound, with two frozen
mapped captures/clean analyses, fresh explicit synthetic login and exact provenance.
These scripted checks are not native/human/real-keystore/public approval.

**Phase7 remains NOT ACCEPTED.** A fresh stopped-real-Chromium probe reproduces
**P1 Linux ownership failure**: Chromium's overwritten `/proc/environ` loses the
exact TMPDIR cleanup marker, the app misses its own stopped browser and removes
scratch, and exact test pidfd rescue plus one wrapper intervention are required.
An unrelated process is preserved; zero final live children is not app cleanup PASS.
Current dependent fault/crash-ownership reliance requires bounded repair and recheck;
historical Phase2 acceptance is not silently rewritten. The Product replacement
regression also fails: identical original hashes/profiles but one ResourceFailed
warning in the baseline produces truthful NotComparable; exact request cause remains
untraced, so no material-profile mismatch or clean regression pass is invented.

Under decision0010, the single authorized physical-disk1000-page/three-profile soak
fails at the original1200 s capture budget:2188 captures/visits,730 distinct pages,
729 complete triples,812 unstarted targets cancelled;1000 paused originals survive
reopened-Store resume adding1188. Independent SQLite/FK/all6121 CAS object checks
pass, but full coverage/activation and separate >=1000-capture cancellation were
not reached. Prior1708-capture failure remains preserved; no further soak is authorized.
Retained rates/CPU/IO support investigating growing full-snapshot Rust checkpoint
overhead, not a measured function-level attribution or proof that RAM caused old failures.

All125 maintained files/46 historical reports/6026 runtime files remain unchanged;
fresh/historical candidate and CAS audits pass scoped integrity. All owned work is
settled, with162 adopted children reaped, zero remaining processes/pending files/
private-namespace entries. Two verifier Node launch timeouts and UI oracle failures
remain retained separately from final passing corrected probes. Windows API scaffold
passes, but authentic full compile stops at missing Microsoft `lib.exe`, not an
application integration pass. macOS owned-tree/accessibility code gaps, GLIBC_2.43
ABI failure, complete platform/native/OS-key/public runtime/human/licensing/signing/
package gates remain open. No implementation edit, nested agent or publication occurred.

Bounded implementer corrections are now settled in `docs/phases/07-corrections.md`.
At that implementer handoff, Linux cleanup was described as a dedicated per-launch
syscall-only subreaper guardian, signaling
only `waitid(P_PIDFD)`-confirmed children. Kernel ECHILD receipt plus exited guardian
authorizes marker/scratch deletion; recovery waits on boot/birth-verified pidfd and
never signals/recaptures legacy or unknown processes. Unknown ownership fails closed
and reserves budget. Maintained stopped-main/reader, cancel/Drop, worker/app crash,
acknowledged-launch race, recovery and unrelated identical-browser-recapture checks
PASS locally. Fresh independent correction recheck was PENDING at that delivery;
the later actual independent findings below supersede those unverified safety claims.

Migration9 target deltas remove repeated full-snapshot checkpoint work without
changing immutable profiles/evidence/provenance/reference graphs or strict PNG/CAS
publication/restart boundaries. Synthetic100/200/400-target timings improve from
1.024/3.925/15.715 s to0.101/0.187/0.369 s with independently linear row/validation/
serialization/pin counts. ONE explicitly authorized changed-implementation soak
passes all3000 targets in900.357 s with unchanged1200 s capture budget; total1120.446 s,
wrapper1132.039 s/1560 s unchanged outer bound. Separate large run captures1000 and
cancels2000,0.303 s settled latency, baseline unchanged. No second correction soak.
The earlier prohibition/failed attempts above are historical, not erased.

Historical readiness warning remains untraced: current diagnostics observe correctly
attributed intentional replacement aborts, but no proven matcher race was repaired
and genuine ERR_ABORTED/resource warnings/NotComparable remain intact. Final audit
checks129 frozen inputs/46 historical reports/6026 historical runtime files, current
6028 signed runtime files, current/historical CAS and exact fresh reference graphs;
zero pending deltas/files/scratch or live recorded process births. All work settled.
Resource caveats: guardians inherited test-thread comm and fell under HarnessBuild,
so recorded RustGuardian zero is not zero RSS; kernel IO includes waited children.
No native responsiveness or release acceptance inferred. Historical candidate stays
preserved but stale for corrected code; all platform/native/public/signing/human/
licensing/ABI gates remain open.

Fresh independent correction verification is now FINAL in `07-reverification.md`.
The exact original stopped-main probe and both signed ownership tests PASS without
rescue; current35 signed Node/full actual Product and139 all-feature Rust tests
PASS.700 fresh100/200/400-target boundaries exactly equal legacy snapshots and
reference graphs, with linear rows/validation/serialization/pin counts. However,
an actual multithreaded-parent probe proves the nested libc fork invokes inherited
pthread_atfork handlers after fork (**P1**, no hanging process manufactured).
Fresh actual recovery accepts stale/matching forged/copied receipts or skips invalid
current records (6/8 fail-closed controls FAIL, **P2**). Same-size old CAS corruption
is accepted by pause/resume/finish compaction (**P2**), though load/activation correctly
refuse. An explicit warned-baseline private-copy control still yields NotComparable;
historical ResourceFailed cause remains UNPROVED.

The combined local correction gate is NOT ACCEPTED. No fresh soak was started
because local ownership/hash-boundary prerequisites failed, not because external
gates imposed a blanket stop.135 maintained inputs and historical reports/runtime/
candidate/CAS remain unchanged;11 new private stores have exact refgraphs, including
actual abrupt-exit-after-delta/reopened recovery/resume/cancel, with the
one deliberately corrupted object retained as failure evidence. All35 bounded
commands settled:26 natural exited adoptions, zero deadlines/forced/remaining
children or hidden rescue. Next work is a separate bounded repair and independent
recheck of these three findings. Public release/full Phase7 remains NOT ACCEPTED;
platform/native/ABI/public/OS-keychain/human/licensing gates stay open.

## Status conventions

“Implemented” means code exists, not that all acceptance checks passed.
“Verified locally” means a different agent exercised the listed actual tests.
“Pending/NOT RUN” is neither a pass nor a product defect by itself.
Native/backend/actor checks are distinct. No published-release readiness is
implied by local development progress.
