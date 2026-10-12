# Independent Phase 5 re-verification — PASS

**2026-10-12 · base `2f09a6b` · S1–S3 accepted within source/controller/Linux
offscreen scope.** V5-1, V5-2 and V5-3 clear. No finite blocking defect remains in
the implemented behavior examined here. Original [FIX](phase5-verification.md),
[blocked report](phase5-report.blocked.md) and [blocked receipt](phase5-receipt.blocked.json)
are unchanged historical evidence, not retrospectively upgraded passes.

## Actual independent results

| Check executed by verifier | Result |
|---|---|
| Locked offline core/storage/app, all features | **308 passed, 0 failed, 40 ignored**, exit 0 |
| Separate serial binary `phase5_` filter | **12 passed, 0 failed, 0 ignored**, exit 0 |
| Byte-identical original external failing probes against fresh repaired-source copy | **2 passed, 0 failed, 0 ignored**, exit 0 |
| External strengthened `phase5_repair_tests` | **5 passed, 0 failed, 0 ignored**, exit 0 |
| Receipt checker, immutable evidence and current source binding | PASS |
| Formatting and whitespace | PASS, exit 0 |

Packages comprise app library 95, binary 119, analysis integration 7, product
flow 1, core 51 and storage 35. Ignored tests were not silently executed or accepted.
Logs under `/var/home/olav/Data/opencode/tmp/opencode/`:
`phase5-reverifier-packages.log`, `phase5-reverifier-targeted.log`,
`phase5-reverifier-probes.log`, `phase5-reverifier-strengthened-bound.log`.

Commands use locked/offline Cargo, `.runtime/link-libs` (absolute for external
workspace) and the approved TMPDIR. Packages:
`cargo test --locked --offline -p regressee-core -p regressee-storage -p regressee-app --all-features`.
Targeted binary: `cargo test --locked --offline -p regressee-app --all-features --bin regressee phase5_ -- --test-threads=1`.
External runs use `--manifest-path <probe-root>/Cargo.toml`, existing CARGO_TARGET_DIR,
the `independent_phase5_probe` or `phase5_repair_tests` filter and serial execution.
No renderer command was run.

Fresh verifier workspace:
`/var/home/olav/Data/opencode/tmp/opencode/phase5-reverifier-probe-vdu97ad4`.
The original `independent_phase5_probe.rs` is byte-identical, SHA-256
`90b21416b69db173c9e5a4aa3aaf4ead88a0a07c665f0fbef01d902e2e2e3e25`;
both named tests actually executed (not zero-filtered). Production files were
copied unchanged, with only an external test-module declaration added to main.
Further assertions were added only to the external copy of repair tests.

## Substantive gate verification

### V5-1 — guided validation and affected input: PASS

`form_recovery.rs::route_form_correction` now consistently routes Save and the
correction summary: guided Name/Address → Website, discovery → Pages (disclosing
advanced controls as needed), login/mapping → Access. Full Settings stays Settings.
Login fields remain mounted for correction even on public access with advanced
options disclosed. Existing control/focus identities and saving semantics remain.

Both-size tests execute invalid Address, login fields 10/11/12 and discovery
0/3/4/5/6 through actual Shell Save, checking retained values, expected stage,
mounted strip, presentation/actual input focus, no queued request, whole painted
field and inner border in a freshly read current clip, and successful correction.
The independent strengthened run additionally moves back to Website before
activating the summary to prove cross-stage correction routing, rather than only
repeating the same-stage route. Original external Access-stage assertion passes.

Both-size repair originals show the persistent strip and readable affected field,
attached error and help. Field geometry: **680×123px** wide, **580×123px** minimum.
These bounds represent the whole field, not a padding-only wrapper.

### V5-2 — save/visit/draft/hydration boundaries: PASS

Status observation no longer grants hydration merely because its request snapshot
matches an already edited draft. Immutable project-bound hydration origin and
request/draft checks gate hydration; saved login forms are separately keyed by
project/environment. Successful save records submitted login values, while newer
form values remain dirty. Project-mismatched save records are rejected. Request
identity is checked before processing terminal replies; screen/stage/composition
are part of the exact save-continuation signature.

Independently rerun tests establish:

- Clean public Continue advances without a write; dirty ordinary stages await
  authoritative save. Private dirty access requires Save first, then explicit
  Open login browser; saved/open/not-ready/ready gates remain separate. Settings
  Save has no implicit visit; exact Save & find pages success explicitly dispatches
  discovery. Session readiness is saved-environment bound, not proof of real login.
- The exact formerly failing local worker/storage probe retains the newer login
  URL through successful save and subsequent absent-status hydration.
- Expanded worker tests retain newer fields **10, 11, 12 and retention**, including
  a representative URL edited during the follow-up read. Status still loads;
  login-only drafts remain dirty without status, block browser opening, and can
  later be saved explicitly. Untouched initial hydration is a positive control:
  deliberately cleared login fields/retention restore the saved values on load.
- Both Continue and Save & find pages execute the nine-case matrix: newer,
  invalid, composing, stage switch, screen switch, environment switch, project
  switch, actual save conflict/failure and stale request. No unauthorized discovery,
  selected-project draft overwrite or continuation; failed/newer edits survive.
  Stored active baseline and complete evidence JSON remain unchanged.
- Invalid/composing drafts before either action queue nothing. Late composition
  tests simulate a pre-save-frame IME event; they are not real OS IME acceptance.

### V5-3 — profiles and complete advanced reachability: PASS in tested scope

Measured setup action/choice/whole-field paint feeds reveal against the current
scroll owner. The existing setup tests no longer reuse a pre-stage-switch clip.
Renderer assertions use the form/mapping clip for scrolling controls and the
window for pinned controls; initial Mobile can truthfully require scrolling.

The both-size capability test traverses mounted/enabled core candidates via real
Shell focus movement, checks actual focus, redraws/reveals, reads a **fresh clip**,
and dispatches profiles, custom add/replace/remove, extent/images, discovery modes,
robots/speed/depth/parallelism/retries/link/tracking/fragments, access/retention,
mapping approval/same-site/preview/Close, locale/timezone/theme, environment
creation/selection, limitations and login/ready/close/forget/probe actions. All
website/bound/exclusion/advanced fields are focus-reachable. Mobile is wholly
revealed and its dispatch changes the painted selected-size plan without saving.
External assertions additionally verify custom replacement **1300×700** and an
actual changed timezone selection.

**Qualification correction:** the repository capability test calls simulated
timezone keystrokes without installing startup keybindings and checks only general
dirty state. A first strengthened external assertion therefore failed (UTC stayed
UTC); this was a test setup gap, not a reproduced product defect. After binding
`text_input::bind` and `timezone_select::bind` exactly as production startup does,
the strengthened both-size test changes timezone and passes. That final bound run
is counted; the failed diagnostic log `phase5-reverifier-strengthened.log` remains.
An upstream test-only improvement can install these bindings/assert selection;
it is not a remaining functional exit-gate blocker because independently qualified
production-bound dispatch passes here.

Session dispatch uses unavailable-runtime-safe local workers and explicit synthetic
open/session prerequisites. It proves control reachability/dispatch, not successful
authentication. Tab tests position the predecessor then use Shell traversal; they
do not claim an uninterrupted real OS keyboard task or accessibility acceptance.

### Remaining S1–S3 contract: PASS within evidence boundary

Public/private choice and next action remain in the first minimum viewport;
three stages retain current/completed/unavailable state; saved/draft/browser
prerequisites stay attached. Sizes lead Pages and selected dimensions, extent and
image policy remain readable. All retained advanced capabilities are still in
source; no schema, capture-policy, screenshot pixel or baseline semantic change
was introduced. Concrete mapping pairs lead scope notes; included-target estimate,
coverage cause and rediscovery advice remain adjacent to Capture. Prior scoped
mapping/discovery inspection in the original verifier report remains applicable:
repair did not replace those sections. Copy preserves web-emulation, secure-store,
no-plaintext, bitmap replacement and capture limitation warnings. Workers continue
to own database/filesystem/process/image operations.

## Existing captures, integrity and documentation

Read updated report/receipt and actual tracked diff, new setup/repair tests and
affected status/save/routing/paint/focus owners. One read-only batched inspection of
**all six existing repair originals**, no new rendering or polish:
`repair-v5/all-{1280x800,820x600}-{access-invalid,pages-invalid,pages-mobile-revealed}.png`
under `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261012`.
Wide Mobile matches an older image because no scroll was needed; state and
actual-owner geometry, not image nonidentity, establish its scope. Narrow Mobile
is now fully revealed, **580×46px** (wide **680×46px**).

Receipt checker validates **12 immutable assessment archives, 182 original-root
PNGs, 38 historical blocked confirmation pairs, original FIX, six repair pairs and
66 final source bindings**. Supplemental check confirms repair source-binding JSON
equals its capture-source map. Only `phase5_repair_tests.rs` differs between capture
and final source; its capture-time snapshot/hash and strengthened final hash remain
distinct. Shipping/capture source was not edited by this verifier.

The plan now records this independent accepted status only. Its exact pre-verifier
bytes were preserved at
`/var/home/olav/Data/opencode/tmp/opencode/phase5-plan.pre-reverification.md`
(SHA-256 `77d59610ac8786a545a1b4961b1eae9d5d37c85cbb6f418845766f297fb76617`).
Receipt documentation binding is explicitly separated from this later checkpoint
update; no historical pixel or production-source hash is rewritten.

## Limits and next action

PASS is **not** native-window/input, accessibility, Windows/macOS, text scaling,
clipboard/IME, high contrast/motion, real runtime install/login, PM usability,
large-site throughput or release qualification. Synthetic Linux GPUI/Blade
offscreen pixels and simulated input stay explicitly bounded. Trusted-browser
distribution and vendor/test-only warnings remain documented limitations.

**Next: parent reviews and commits/pushes verified Phase 5 before Phase 6.**
Verifier changed only verification/checkpoint documentation and its documentation
integrity annotation; no product edit, new renderer, polish, delegate, commit/push
or Phase 6 work.
