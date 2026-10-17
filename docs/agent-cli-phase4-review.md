# Agent CLI Phase 4 — independent verification / finish review

## Final independent confirmation — PASS, protocol 6

**Final verdict: PASS for Phase 4 within the reviewed source, Linux IPC,
owned synthetic-fixture and GPUI offscreen/simulated-input scope.** All five
original findings and the retained-feedback follow-up are independently
**CLOSED**. No remaining repair-specific actionable defect was found in this
bounded confirmation. Earlier NEEDS FIX sections below are preserved as history.

**Finish disposition: SHIP at this tested source/fixture/offscreen scope.** This
is **not** native-window, OS-input/IME, accessibility, platform or public-release
qualification. Those gates remain **UNQUALIFIED**. No Phase 5 work, commit, push
or publication is authorized or performed by this verdict.

### Finding scores / closure

| Finding | Final independent disposition |
| --- | --- |
| 1. Anonymous IPC frontend/lease bypass | **CLOSED** — prior protocol-6 bootstrap/lifecycle/forged-proof acceptance stands; relevant source is unchanged. Private capability possession is coordination, not an OS sandbox. |
| 2. Retained input callbacks and gesture ownership | **CLOSED** — live gate/epochs, actual-IPC Return, widgets and retained viewer/editor checks remain passing. |
| 3. Long strings and useful evidence viewport | **CLOSED**, including the retained-business-feedback follow-up independently reproduced below. |
| 4. Same-text/empty project IME preedit | **CLOSED** — unchanged independent acceptance passes again. |
| 5. Visible nonlocking terminal status | **CLOSED** — actual expiry/disconnect/restart/dismissal acceptance passes again; the obsolete original predicate is not used as proof. |
| **Open review findings** | **0** at the explicitly qualified scope. |

### Exact failing case and combined-state confirmation

Read the updated repair ledger, the four changed Rust paths, new durable tests
and `agent-cli-phase4-feedback-repair/source-handoff.json`. Independently matched
**all 44 source hashes**, both initial production-binary hashes, the preserved
report and original witness/acceptance sources. The pre-confirmation report hash
was `28dd23a9d2a8f645f4177f3b29e5a21a6020de0eb20223dff982c62edd5ae39c`.
Only this report was edited in the repository; no production/test source, theme,
asset, permission, sandbox or unrelated aesthetics were edited. Compilation was
verifier-owned and serial; there was no subdelegation or user-store access.

New evidence root (`F` below):

```text
/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase4-review/feedback-final
```

The **unchanged** external
`remaining_retained_feedback_must_not_collapse_handoff_evidence` now passes with
the same genuine `Settings saved on this computer.` notice:
**no-agent 72px → handoff 75px**, rather than the previous 2px. The original
**64px minimum**, notice, eligibility check and assertion are intact. It is
imported directly from the original external module; no test rewrite, message
clearing or threshold reduction was needed. All five tests in that original
acceptance module pass again (`original-acceptance.log`). The durable exact-case
regression independently confirms **75px in both Light and Dark**.

The actual repair is narrow:
`crates/gui/src/agent_ux.rs:435–473,569–575,649–688` shares retained business
feedback with the existing bounded banner reader;
`crates/gui/src/main.rs:8687–8741,8827–8831` avoids stacking its ordinary panel
above the same evidence stage. Error and notice strings remain present and are
rendered in full, with error priority. Take control, Return and Dismiss feedback
stay pinned. Dismiss feedback still uses the existing live control guard:
**disabled while following**, available only through deliberate human activation
during handoff. Reading or entering control does not erase feedback or change
evidence identities/settings. Ordinary feedback without this named banner and
other-screen/task behavior retain their existing path.

Added separate independent acceptance, without altering the original module:
**80-byte unbroken agent name + valid 1,024-byte handoff request + retained
success notice + long error**, in both themes at **820×600, 1280×800 and
1440×900**. It verifies the unchanged minimum evidence floor, finite title/status,
contained pinned actions, full reader End/Home extent, unchanged messages and
selected evidence, explicit dismissal, and unchanged stored evidence. The
implementor's passing success/error/both × following/handoff matrix additionally
covers guarded dismissal. The existing ordinary-feedback regression and saved-
data refresh tests pass. No ordinary-feedback regression was observed.

### Final-source paint / floor / documentation

Independently rendered **one bounded batch of 20 frames**: 16 production-source
feedback states at 820×600 (both themes, following/handoff, success/error/both and
combined reader-end positions), plus four independent maximum-name/request/error/
notice start/end frames. Inspected the complete contact sheet and full-size
representatives. Evidence remains **75px**, the banner **87px**, and the reader
**22px** in the minimum-size production frames. Long names truncate within their
reserved measure rather than overlapping the three pinned actions. Error text
remains readable in its existing semantic ink; complete content remains reachable
through the reader. Existing system type, 40px controls, neutral/teal palette,
logo, untinted screenshots and approval/input guards are preserved.

Production capture metadata and compiled metrics bind current main source
`d1847d398b0c3bcc41607aeb5441120ffc96bea441ebbbb135de088e37be00dc`
and agent presentation source
`2245e7ccad04b54ead773e180a36e8a651a720212146d12392effdc503bac531`.
The four-file source bundle matches
`d5699161525299e2471a155f6a5a13bf79a5cdd852ca74ba9191cccaad48812c`.
All 20 PNG hashes match metadata. The additional independent maximum-state frames
come from a byte-identical GUI copy except for the distinct test binary name and
test-module declarations; that distinction and all probe hashes are explicit in
`final-evidence.json`. Captures assert frozen stored evidence unchanged. No code
correction or aesthetic recapture loop followed this independent batch.

Documentation check: repair ledger accurately names the shared-reader behavior
and retained qualification limits; CLI/engine protocol-6 and private-bootstrap
contracts remain unchanged. No new design token, shipping raster or sidecar/brief
repair is needed. This report is already classified as internal documentation.

### Final scoped commands / counts

Environment: Rust 1.98.1 PATH from earlier sections,
`TMPDIR="$F"`,
`LIBRARY_PATH=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase1/link-libs`.
Probe commands use `CARGO_TARGET_DIR=/var/home/olav/Projects/Regressee/target`.
Expanded argv/exits/timings are in `$F/commands.json`.

| Command / exact scope | Independent result / log in `$F` |
| --- | --- |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --lib --bin regressee-desktop agent_ -- --test-threads=1` | **17 passed**: 3 library, 14 desktop; `gui-agent.log`. |
| Same desktop test command with `tests::feedback_dismiss_is_attached_and_long_errors_leave_a_working_viewport -- --exact --test-threads=1` | **1 passed**, 246 filtered; `ordinary-feedback.log`. |
| Desktop filter `engine_refresh_tests::` | **2 passed**, 245 filtered; `refresh.log`. |
| `cargo test --locked --offline --manifest-path "$F/probe/Cargo.toml" -p regressee-gui --features ui-tests --bin phase4-feedback-review-desktop original_acceptance:: -- --nocapture --test-threads=1` | **5 passed**, 248 filtered, including the unchanged previously failing case; `original-acceptance.log`. |
| Same probe command with `final_acceptance::combined_maximum_identity_request_and_feedback_preserve_viewport_and_deliberate_dismissal -- --exact --nocapture --test-threads=1` | **1 passed**, 252 filtered; `combined-acceptance.log`. |
| `cargo test --locked --offline -p regressee-gui --all-features --bin regressee-desktop ui_renderer_capture::ui_agent_capture::renderer_capture_agent_retained_feedback -- --exact --ignored --test-threads=1` | **1 passed**, 313 filtered; **16 PNGs**, `capture.log`. |
| All-feature probe binary, `final_acceptance::renderer_combined_maximum_feedback -- --exact --ignored --test-threads=1` | **1 passed**, 320 filtered; **4 PNGs**, `combined-capture.log`. |
| `cargo check --locked --offline -p regressee-gui --all-targets --all-features` | Exit **0**, `check.log`. |
| `git diff --check` | Exit **0** at final report validation. |

Final confirmation: **26 positive scoped tests passed, 0 failed**, plus two
explicit renderer tests producing the 20 frames. These are not added to prior
overlapping totals as new coverage. Unchanged backend authority/CLI/engine proof
from the preceding re-review stands; no full inherited GUI suite or expensive
scale rerun was needed. Existing inherited failures/global formatting/lint debt
are not waived. Original witnesses, original acceptance source and prior reports
remain unchanged; only this report's new independent disposition supersedes its
earlier NEEDS FIX verdicts.

**Qualification boundary:** DISPLAY and WAYLAND_DISPLAY remain unset; no
authorized display/socket was supplied. GPUI/Blade offscreen rendering, simulated
input and Linux IPC do not qualify actual OS-window input/IME, accessibility,
Windows/macOS behavior, managed login, clean headless browser/fonts/sandbox or
release packages. The Phase 4 PASS is confined to the reviewed source and tested
scope, with those native/platform/release gates explicitly outstanding.

---

## Independent repair re-review — protocol 6 (historical)

**Final disposition: NEEDS FIX — one remaining viewport defect.** Original
findings **1, 2, 4 and 5 are independently CLOSED**. Finding **3 is partially
closed**: final-source maximum-string paint is repaired and independently
inspected, but retained ordinary business feedback can still collapse the
minimum-size handoff evidence pane. This is a failing correct-expectation test,
not a native-platform qualification blocker or an obsolete witness assertion.

**Finish disposition: FIX the remaining layout budget, then rebuild and perform
one scoped confirmation of that state.** Native-window/OS-input/accessibility
remain **UNQUALIFIED**. No Phase 5 advancement, commit or push is implied. The
initial review below remains historical evidence.

### Final source and proof identity

Re-read the repair ledger, protocol/client/authority/bootstrap lifecycle, live
input gate and retained callbacks, hydration, banner, regressions, CLI/engine
contracts and `agent-cli-phase4-repair/source-handoff.json`.

Independently matched **all 43 final source hashes**, both initial production
binary hashes, the prior reports, and both original external witness-source
hashes. The original report's pre-re-review hash was
`8f2332e085bd30ede7f67fc6517b572dc36319c31b6481debe104321a76ab549`.
Production source remains unchanged by this review. Only this report was edited
in the repository; all probes/stores/receipts are private and external. No
subdelegation, user-store access, outside-site capture, system/compositor/helper
change, permission change, sandbox weakening, commit, push or Phase 5 work.
Heavy shared-target compilation was verifier-owned and serial.

New independent evidence root (`V` below):

```text
/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase4-review/rereview
```

`identity-before.json` and `final-evidence.json` bind source, initial binary
identity, external probes and all final-source PNGs. The external GUI is a fresh
copy of the repaired crate, with only test-module declarations and the distinct
binary name `phase4-rereview-desktop`. The original witness module is imported
**unchanged**, not overwritten. The engine's legitimate setup/cleanup helper alone
was adapted to explicitly read the private GUI bootstrap; the anonymous attack
body remains unchanged. The diff is retained in
`legitimate-bootstrap-adaptation.patch`. New acceptance tests are additive and
separate; they do not replace the original wrong-behavior assertions.

### Findings independently closed / remaining

1. **Anonymous GUI authority — CLOSED.**
   `crates/app/src/engine/authority.rs:33–62,157–160`,
   `engine/mod.rs:170–225`, `engine/client.rs:271–288` and
   `engine/ownership.rs:125–158` establish the new boundary. Ordinary anonymous
   registration always rejects; explicit bootstrap proves a private engine-minted
   capability. Independently checked forged/current-engine secrets, old incarnation
   and old secret with a substituted new incarnation, authenticated role switches,
   anonymous control variants, redaction/public-read non-disclosure, 0600 file
   permissions, clean-shutdown removal, rotation/restart, immediate Detach
   retirement and unrelated-frontend non-revocation. Registration/renewal do not
   alter navigation epochs or silently Follow. The unchanged spoof witness now
   exits **101 at its expected-Credential branch**, not compilation/setup.
   Three original positive engine probes still pass, including actual queued
   lease expiry, exact entity rejection, retained confirmations and frozen evidence.
   This is a **private-filesystem-capability coordination boundary**: a same-user
   program explicitly reading that file can imitate a frontend. It is not GUI
   executable attestation or an OS sandbox, as the repaired docs correctly state.

2. **Retained input — CLOSED in reviewed GPUI/IPC scope.**
   `crates/gui/src/agent_ux.rs:144–151,265–311`,
   `ui/interaction.rs:18–58`, `ui/scrollbar.rs:280–395` and
   `viewer_ux.rs:87–95` enforce live authority rather than a painted boolean.
   Replayed all original zoom/local-drag/actual-IPC drag witnesses unchanged:
   each now fails at its assertion of unwanted movement, with equal before/after
   values. Correct-expectation real-IPC tests cover both Return transition and
   acknowledged Active, and the widget regression covers a lock/unlock interval
   **without an intervening event**, preventing held-gesture revival via epochs.
   Fresh gestures work after unlock. Inspected/tested text selection, clipboard,
   multiline scrolling and IME callbacks against the live gate even when the old
   editor reports enabled; nested timezone/zoom suspension and viewer helpers are
   guarded. Banner details, immediate takeover, semantic hydration and close/Quit
   guards remain deliberately permitted paths. This is not OS-input qualification.

3. **Long strings / finite viewport — PARTIALLY CLOSED; one MEDIUM / P2 remains.**
   The final measured-width correction in
   `crates/gui/src/agent_ux.rs:435–496,605–644` is real and visually verified:
   names/status paint with finite nonzero width, names do not wrap into status,
   actions stay pinned, and maximum 1,024-byte requests remain in the 22px
   independently readable details owner. Clean-banner evidence heights measured
   **75 / 307 / 407px** at 820×600 / 1280×800 / 1440×900. Keyboard End/Home reaches
   the full details extent and returns to the start.

   **Remaining actionable defect — retained success feedback collapses handoff
   evidence to 2px at 820×600.** Relevant paths:
   `crates/gui/src/main.rs:8687–8690,8730–8739,8827–8831,8945–8955`
   (feedback plus the new banner share the fixed viewport); the banner is 87px
   high and the evidence body is allowed to shrink to effectively nothing.
   `agent_can_follow` does not require dismissing feedback, nor should it silently
   discard it. Routine saved-settings feedback can survive Follow/handoff.

   **Repro:** retain the genuine `Settings saved on this computer.` notice when
   opening the saved comparison at 820×600; enter named temporary human control
   without dismissing that notice. The independent controller fixture proves
   `agent_can_follow() == true` before entry. With **the same notice and evidence**,
   no-agent pane height is **72px**; human-handoff pane height is **2px**. The clean
   fixture is 75px with the banner. This controlled differential isolates the
   added control chrome rather than inferring a problem from an inherited list
   assertion. It is actual GPUI layout evidence, not an OS-window claim.

   **Proof:** `gui-acceptance-final.log`, test
   `independent_acceptance::remaining_retained_feedback_must_not_collapse_handoff_evidence`.
   It exits **101 at the retained minimum-height assertion**. The original
   long-text captures and builder regressions clear `s.notice`/`s.error`; they
   cannot certify this combination. My clean-banner positive test now explicitly
   states that isolation, while this separate correct-expectation failing test
   preserves the real notice. The failure was not hidden by clearing it globally.

   **Required repair:** budget a useful minimum evidence stage with both control
   chrome and routine/error feedback present, keeping messages reachable in a
   bounded reading owner. Do not erase feedback, hide enabled business actions
   over an unusable evidence stage, relax input/approval guards or redesign the
   palette. Test both following and human handoff, success/error feedback and the
   minimum supported size. No unrelated aesthetics need repair.

4. **Passive project IME — CLOSED.**
   `agent_refresh.rs:7–11`, `main.rs:1542–1579,3690–3711` and
   `ui/text_input.rs:127–134` defer hydration during affected editor composition
   and preserve unchanged editor state. The original same-text-loss witness now
   fails at its assertion that composition was cleared. Independent positive
   tests additionally exercise same-text **and empty preedit** across name,
   address, all crawl/builder/login/locale fields and nested timezone search,
   retaining content, selection and the original save revision. Once composition
   safely ends, the clean external update applies. Metadata-only refresh retains
   unchanged touched/correction/selection state in the durable regression.

5. **Silent terminal status — CLOSED.**
   `agent_ux.rs:7–77,195–208,544–580` mounts a last-observed-session explanation
   independently of a current controller. **The original expiry witness is
   obsolete**, and still passes unchanged: it asserts the old controller/error/
   message/activity predicate and directly assigns state without the new observed-
   session lifecycle. It never asserts the new banner's visibility or content.
   Its pass is neither a continuing defect nor repaired-state acceptance.

   I independently replaced that evidentiary gap with a **new, separate actual
   10-second engine/GUI lease-expiry test**: the named expired explanation is
   visible, includes continuation truth, is nonlocking, and leaves ordinary GUI
   navigation usable. Pointer Dismiss changes neither epoch nor expired authority;
   renewal still rejects. A new session replaces the old status without following,
   session end remains readable, engine loss stays unlocked, restart is separately
   explained, and keyboard activation dismisses it without resuming control.
   The existing takeover/ended/disconnect regressions and final-source captures
   also pass. No stale `debug_bounds` disappearance predicate was used as acceptance.

### Independent final-source paint / finish review

The implementor's two capture stages were **not** accepted as final paint: the
last finite-width fix postdated them. I rendered **one new batched set of 30
frames** from the unchanged final production source: three desktop sizes × both
themes × maximum-name/request, long failure, expiry, disconnect and takeover.
All 30 PNG hashes match metadata and all record current main-source SHA-256
`3bb974f239c7ed6c8a85eab1a06775564641f59c858c11915312e8360cbc62b2`.
The same frozen source receipt binds `agent_ux.rs`
`8729de2b4bf91e988c00d24c60a578ec5c11c01d0da10c94a6f9d9851280d807`.
No UI edit or aesthetic recapture loop occurred.

Inspected the complete contact sheet and full-size representative/max-length
frames. Title/status are actually readable, not ellipsis-only or overlapping;
Take control/Return/Dismiss remain contained, terminal states visibly unlock,
and originals retain their pixels. The common 87px banner, 40px targets, system
type, Unified neutrals/teal, focus treatment and logo are preserved. Source-token
contrast continues to meet the earlier documented floor. Native accessibility
and OS keyboard/IME presentation are not inferred. The **retained-feedback
combination remains a failing layout measurement outside those isolated capture
fixtures**, so these 30 good frames are not a full viewport/Phase 4 PASS.

CLI/engine documentation now correctly describes protocol 6, private bootstrap,
capability lifecycle and same-user limitations. The review is classified as
internal documentation in `scripts/source_release.py`. No new token/asset or
sidecar/moved-brief repair was needed. The remaining unconditional useful-viewport
claim needs the layout repair above, not another documentation-only waiver.

### Exact commands and results

Environment is the initial review's Rust 1.98.1 PATH and Phase 1 `LIBRARY_PATH`,
with `TMPDIR="$V"`. Probe commands also use
`CARGO_TARGET_DIR=/var/home/olav/Projects/Regressee/target`. Commands run from the
repository. Expanded argv/timings: `commands.json`, `probe-commands.json`.

| Command | Independent result / log in `$V` |
| --- | --- |
| `cargo test --locked --offline -p regressee-app --features test-support --lib --test agent_control --test engine -- --test-threads=1` | **84 passed, 2 ignored** (60 library, 6 control, 18 engine); `app.log`. |
| `cargo test --locked --offline -p regressee-cli -- --test-threads=1` | **18 passed** (1 unit, 1 lifecycle, 16 workflows); `cli.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --lib --bin regressee-desktop agent_ -- --test-threads=1` | **15 passed** (3 library, 12 desktop); `gui-agent.log`. |
| Same GUI library command with filters `ui::text_input::tests::`, `ui::scrollbar::`, `ui::timezone_select::tests::` | **11 / 5 / 9 passed**; `text.log`, `scroll.log`, `timezone.log`. Three overlap with the preceding group, not additional unique passes. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**; `gui-client.log`. |
| GUI desktop command with `engine_refresh_tests::` | **2 passed**, 243 filtered; `gui-refresh.log`. |
| `cargo test --locked --offline --manifest-path "$V/engine-probe/Cargo.toml" -- --nocapture --test-threads=1 --skip witness_anonymous_ipc` | **4 positive tests passed**, 1 filtered; `engine-positive.log`. Three original positives plus independent bootstrap lifecycle. |
| Same engine manifest with `witness_anonymous_ipc -- --nocapture --test-threads=1` | Expected **101**, original assertion branch rejects the former anonymous credential mint; `engine-original-witness.log`. |
| `cargo test --locked --offline --manifest-path "$V/gui-probe/Cargo.toml" -p regressee-gui --features ui-tests --bin phase4-rereview-desktop original_witnesses:: -- --nocapture --test-threads=1` | Expected **101**: four old wrong-behavior assertions fail; obsolete expiry predicate still passes. `gui-original-witnesses.log`. |
| Same GUI probe command with `independent_acceptance::` | **4 passed, 1 remaining failure**, 250 filtered; `gui-acceptance-final.log`. Includes actual expiry/reconnect, empty/same-text IME, retained viewer input and short/max-name details. |
| `cargo test --locked --offline -p regressee-gui --all-features --bin regressee-desktop ui_renderer_capture::ui_agent_capture::renderer_capture_agent_control_repairs -- --exact --ignored --test-threads=1` | **1 passed**, 310 filtered; **30 final-source PNGs**, `captures.log`. Private approved/capture roots are `$V` / `$V/captures`. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Exit **0**, `check-workspace.log`. |
| `git diff --check` | Exit **0** at final report validation. |

There are **150 distinct passing positive tests** in these groups after accounting
for the three widget overlaps, plus **one remaining correct-expectation failure**.
The render test and obsolete/wrong-behavior witnesses are not added to that total.
No full inherited desktop suite or unchanged 1,000-page scale rerun/waiver.
Probe development's initial clean-banner test inadvertently retained the saved-
settings notice; its failing 2px measurement was investigated, not suppressed.
The independent no-agent/handoff differential above now preserves it as the
explicit remaining regression. Original proofs and intermediate logs are retained.

**Native/platform limit:** DISPLAY and WAYLAND_DISPLAY remain unset; no authorized
display/socket was supplied. Final-source GPUI/Blade offscreen pixels, simulated
input and real Linux IPC do not certify OS input, accessibility, Windows/macOS,
managed login, clean-server/browser/fonts/sandbox or packaged releases. Return
the one remaining actionable viewport finding to the original implementor; do
not turn the general test verdict green solely because the isolated captures pass.

---

## Initial independent review — historical

**Verdict: NEEDS FIX.** Five actionable findings remain: an anonymous IPC path can
mint GUI authority and revoke a live controller; retained scrolling bypasses the
GUI lock; a valid handoff message removes the review viewport; passive project
refresh clears live IME composition; and expiry has no visible explanation.

**Disposition: FIX, then REBUILD and scoped RECAPTURE.** Ordinary short-banner
paint is acceptable at the inspected offscreen sizes, but that is not acceptance
of control enforcement or the complete handoff flow. The actual native-window,
OS-input and accessibility gate remains **UNQUALIFIED**, not PASS.

Date: 2026-10-17. Reviewed protocol **5**, CLI schema **1**, base HEAD
`9daf121b960f84506feedefcdf49869327861130` plus the handed-off working tree.
Prior independent Phase 1–3 acceptance is preserved, not reopened or superseded.

## Scope and evidence identity

- Read the plan, Phase 4 ledger, CLI/engine/control documentation, actual protocol,
  engine/client/authority/viewer implementation, GUI control/refresh/input/exit
  paths, tests and `agent-cli-phase4/source-handoff.json`.
- Independently matched **all 30 source hashes**, both initial production binary
  hashes and all three prior independent-report hashes. Final source remains
  identical to that handoff. Receipts: `identity-before.json`,
  `review-evidence.json` under the review root below.
- Only this report was added in the repository. No production or in-repository
  test edit, revert, commit, push, subdelegation, user-store access, external-site
  visit, compositor helper, permission change or sandbox modification occurred.
  Shared-target compilation was exclusively verifier-owned and serial.
- Used isolated local stores, saved synthetic PNG fixtures and the existing signed
  synthetic-worker CLI scenarios. No Chromium, managed-login, clean-server,
  package or release qualification is inferred.
- Impeccable context was invoked once; its initially nonexistent target was
  resolved by reading the actual `crates/gui/src/main.rs`, incumbent PRODUCT/DESIGN,
  pre-extraction surface brief, native audit/polish guidance and craft floor.
  This is a narrow native desktop **Operate** extension, not mobile UI or a
  redesign. Unified neutrals, system fonts, geometry, logo and drafts are the
  preservation contract. Existing sidecar/moved-brief drift was not repaired.

All external proofs are retained at:

```text
/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase4-review
```

`R` below denotes that directory. The GUI probe copies the GUI crate into
`$R/gui-probe`, symlinks unchanged supporting crates/assets/fixtures, appends only
an external test module, and gives its test binary the distinct name
`phase4-review-desktop`. Its only original-file differences are `Cargo.toml`
(binary name) and `src/main.rs` (test-module declaration); no production behavior
was repaired in the copy. The engine probe calls the real public IPC client and
server against owned stores. Its copied fixture changes only the fixture-root
lookup. Exact hashes and these deltas are in `review-evidence.json`.

## Numbered actionable findings

### 1. HIGH / P1 — anonymous IPC can manufacture GUI authority and defeat a live lease

**Paths:** `crates/app/src/engine/authority.rs:119–138,216–242`;
`crates/app/src/engine/mod.rs:172–200,215–230`.

`RegisterFrontend` accepts an **uncredentialed** request and returns a new usable
frontend credential. The only role-switch restriction is `owner.is_none()`.
That fresh credential may Follow a session and Takeover without any GUI or human
action. An otherwise nonowning client can consequently revoke the exclusive
agent and submit ordinary anonymous writes. Direct nonowner Submit/Prepare checks
are therefore insufficient.

**Repro:** start a 90-second named agent with no GUI; from a separate anonymous
client send `Control(RegisterFrontend)`; use the returned credential for
`Follow { session }`, then `Takeover`; submit an anonymous project Save. The probe
observed a live lease with more than 80 seconds remaining, successful Follow,
`Revoked` controller status, and a **Succeeded** anonymous Save. No credential
file was read, secret stolen, owner UUID forged, or filesystem permission bypassed.
Calling `uncredentialed()` also defeats the test that merely rejects registration
while an Agent credential is attached.

**Proof:** `engine-gui-spoof.log`, test
`witness_anonymous_ipc_mints_gui_authority_and_bypasses_a_live_lease_without_a_gui`.
It deliberately asserts the defective behavior and currently passes.

**Required repair:** frontend-role establishment needs a trusted GUI/human
bootstrap capability, not anonymous role minting. Test that raw nonowner requests
without that capability cannot Follow/Takeover/Return or acquire equivalent
authority. Preserve immediate legitimate human takeover and headless operation.
This finding concerns the advertised **protocol coordination boundary**, not a
demand to sandbox programs with the user's filesystem/process permissions: the
bypass requires only the normal unauthenticated IPC surface, not access to any
protected secret. The documented same-user limitation does not explain it away.

### 2. HIGH / P1 — held scrollbar drag remains live while the GUI is locked

**Paths:** `crates/gui/src/ui/scrollbar.rs:317–338,433–457`;
`crates/gui/src/agent_ux.rs:73–76,180–188,213–223`;
related missing callback guard: `crates/gui/src/viewer_ux.rs:81–89`.

The input shield stops ordinary bubbled events, but a scrollbar drag already
owns a capture-phase callback. Neither entering the lock nor its drag callback
cancels/rechecks that authority. Disabling fields and clearing pan/split/mask
drags does not cover this state.

**Repro:** Follow a project-only semantic selection, request human handoff, open
its overflowing Project settings at 820×600, hold its scrollbar thumb, then invoke
the real **Return to agent** handler while the pointer remains held. Moving the
pointer changes the screen despite `agent_locked() == true`. The independent
real-engine/IPC probe observed offset **0 → −3720px** during the transition and
**−3720 → −260.4196px** after the engine reported **Active**. This is simulated
GPUI pointer input over actual IPC, not native OS qualification.

A separate retained-callback witness also changes zoom to 75% while locked:
`viewer_input_allowed()` does not test `agent_locked()`. This proves a callback
guard gap; it is not claimed as a separately demonstrated OS/menu zoom exploit.

**Proof:** `gui-witnesses-confirmed.log`, tests
`witness_real_ipc_return_transition_keeps_scrollbar_drag_live`,
`witness_retained_scrollbar_drag_moves_locked_window`, and
`witness_locked_retained_viewer_callbacks_change_zoom`.

**Required repair:** cancel retained scrolling/gesture ownership at Follow/Return
and enforce the current lock in actual callback execution, including shared
scrollbar capture handlers and viewer input helpers. Keep Take control, Quit/close
guards, data refresh and agent-driven semantic projection usable. Add regressions
for held input across both the transition and acknowledged Active state. This is
a GUI interaction defect; it does not demonstrate unauthorized database writes.

### 3. HIGH / P1 — valid long handoff text collapses the evidence/review viewport

**Paths:** `crates/gui/src/agent_ux.rs:347–358,411`;
`crates/gui/src/main.rs:8880–8893`.

The nonshrinking banner renders the entire allowed handoff message without a
bounded reading area. A valid **1,012-byte** message makes it **263px high** at
820×600. The result queue, screenshots and ordinary review choices disappear;
**Select After screenshot** and **Run check** remain visible/enabled. The human
cannot perform the requested visual review in this full temporary handoff.
Take control remains reachable, but revoking the session is not a substitute for
usable temporary human control.

**Repro:** at the minimum supported size, Follow the named session and hand off
with `"Please inspect the saved screenshots and verify the website login before
returning control. "` repeated 11 times. This fits the engine's 1–1,024-byte
message contract. Compare the ordinary Human frame with the long-message frame.

**Proof:** `edge-captures/review-human-long-message-820x600.png`,
`edge-captures-final.log`; ordinary control frame:
`captures/agent-light-820x600-Human.png`.

**Required repair:** bound/disclose or independently scroll long request/error
content, with a compact status and pinned Take control/Return actions. Preserve
a useful working/evidence viewport at 820×600; don't solve this by dropping the
request or changing the palette/layout identity. Cover maximum-length names,
messages and failures in the bounded recapture.

### 4. MEDIUM / P2 — passive project refresh clears live same-text IME composition

**Paths:** `crates/gui/src/main.rs:1534–1568`;
`crates/gui/src/exit_guard.rs:151–230`.

`refresh_saved_project` checks text/settings dirtiness but not active project-field
composition before calling `set_value`/`set_crawl_fields`. A live preedit can equal
the saved text and therefore be "clean" by those comparisons. An external saved
update then replaces the field and clears its composition. Note/analysis passive
refresh has stronger composition guards; the project hydration path does not.

**Repro:** without following, load Project settings; begin an IME replacement of
the saved name with the same text (a zero-length preedit is another relevant
case). Assert `composing() == true` and `exit_project_dirty() == false`; deliver
a newer external ProjectRecord with a changed name through the real refresh
helper. The field becomes `Changed externally` and `composing()` becomes false.

**Proof:** `gui-witnesses-confirmed.log`,
`witness_passive_project_refresh_clears_live_same_text_ime`. This is a direct
GPUI hydration-path fixture, not an OS IME test or a claimed live-IPC IME pass.

**Required repair:** defer project hydration while any affected project/builder/
login/timezone input owns composition, including nested search input. Preserve
the draft's original expected revision and apply external updates only after a
safe deliberate/clean boundary. Test equal-visible-text and empty compositions,
not only obviously changed strings.

### 5. MEDIUM / P2 — expiry unlocks correctly but silently removes control status

**Paths:** `crates/gui/src/agent_ux.rs:7–14,359–412`;
`crates/gui/src/main.rs:8880–8885`.

The engine retains the session as Expired and clears `controller`, correctly
revoking authority. The GUI finds only `controller`, and mounts the banner only
for that controller/error/message/external activity. An otherwise idle expiry
therefore removes **all** explanation of the named session's status. There is no
readable "lease expired / you have control / jobs continue" state.

**Repro:** allow a following idle session to expire, with no other error or
external job. The independent expired-snapshot frame mounts no banner or Take
control control; the business UI simply becomes enabled. The separate actual
10-second CLI and queued-expiry probes verify that engine expiry itself works.

**Proof:** `edge-captures/review-expired-820x600.png`,
`edge-captures-final.log` (`expired: banner=None take=None`), and
`witness_expiry_removes_all_control_status`.

**Required repair:** retain a compact, nonlocking last-session/expiry explanation
with a deliberate dismissal or replacement lifecycle. Do not resurrect the lease,
restore Follow automatically, cancel jobs, or imply that completed work was undone.

## State/IPC checks that did hold

These positives do **not** waive finding 1's frontend-bootstrap bypass:

- Bounded named leases and renewal; short CLI invocations do not end the logical
  session. Actual 10-second expiry and a queued mutation crossing expiry reject
  new/queued stale work. In-flight synthetic work finishes truthfully; identical
  admitted retries return receipts without replay.
- Direct nonowner Submit/Prepare/epoch advance/preferences/pause/cancel/stop,
  credential owner-ID/role substitution, cross-frontend confirmation use, retained
  pre-session confirmation reuse and wrong engine credentials fail closed.
- During handoff, agent mutations and semantic navigation reject; only the
  following credential can write/return. GUI dirty/modal/pending-task guards,
  explicit Return, immediate takeover and close/Quit draft prompts pass the
  supplied scoped tests. The raw frontend minting path remains the exception.
- Restart revokes ephemeral credentials/sessions and retains committed receipts;
  disconnect/heartbeat expiry unlock without implicit job cancellation.
- Headless selection validates real stored identities, not UUID syntax alone.
  Seven independent invalid project/run/analysis/page/profile/run-required cases
  fail without publishing a replacement cursor. Exact older analysis and rapid
  semantic changes, stale viewer replies/cache identities, passive result/History
  pages, clean-note refresh versus drafts/IME are covered by passing scoped tests.
- Independently compared exact saved evidence before/after selection, handoff,
  return and takeover: original reference, run/settings, analyses and evidence
  remain unchanged. The CLI synthetic running-capture scenario retains its frozen
  inputs/outcome and does not save a baseline on takeover.

## Commands and counts

Environment, from the repository unless a manifest is specified:

```sh
export PATH=/var/home/olav/.rustup/toolchains/1.98.1-x86_64-unknown-linux-gnu/bin:$PATH
export R=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase4-review
export TMPDIR="$R"
export LIBRARY_PATH=/var/home/olav/Data/opencode/tmp/opencode/agent-cli-phase1/link-libs
```

| Exact command | Independent result / log under `$R` |
| --- | --- |
| `cargo test --locked --offline -p regressee-app --features test-support --lib --test agent_control --test engine -- --test-threads=1` | **83 passed, 0 failed, 2 ignored**: library 60/1 ignored, control 5, engine 18/1 ignored. `app.log`. |
| `cargo test --locked --offline -p regressee-cli -- --test-threads=1` | **18 passed**: 1 unit, 1 lifecycle, 16 workflows, including real CLI expiry and synthetic capture across takeover. `cli.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --lib --bin regressee-desktop agent_ -- --test-threads=1` | **8 passed**: 1 library/98 filtered; 7 desktop/233 filtered. `gui-agent.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --test engine_client -- --test-threads=1` | **1 passed**, `gui-client.log`. |
| `cargo test --locked --offline -p regressee-gui --features ui-tests --bin regressee-desktop engine_refresh_tests:: -- --test-threads=1` | **2 passed**, 238 filtered, `gui-refresh.log`. |
| `CARGO_TARGET_DIR="$PWD/target" cargo test --locked --offline --manifest-path "$R/engine-probe/Cargo.toml" -- --nocapture --test-threads=1` | Before the separate spoof witness was added: **3 positive tests passed**, including real queued expiry, seven invalid identities, frozen evidence and retained/forged token checks. `engine-extra-corrected.log`. |
| `CARGO_TARGET_DIR="$PWD/target" cargo test --locked --offline --manifest-path "$R/engine-probe/Cargo.toml" witness_anonymous_ipc -- --nocapture --test-threads=1` | **1 deliberate defect witness passed**, 3 filtered; demonstrates finding 1, not acceptance. `engine-gui-spoof.log`. |
| `CARGO_TARGET_DIR="$PWD/target" cargo test --locked --offline --manifest-path "$R/gui-probe/Cargo.toml" -p regressee-gui --all-features --bin phase4-review-desktop independent_review:: -- --nocapture --test-threads=1` | **5 deliberate defect witnesses passed, 1 capture test ignored**, 305 filtered. These assert current wrong behavior; they are not five acceptance passes. `gui-witnesses-confirmed.log`. |
| `cargo test --locked --offline -p regressee-gui --all-features --bin regressee-desktop ui_renderer_capture::ui_agent_capture::renderer_capture_agent_control -- --exact --ignored --test-threads=1` | **1 passed**, 304 filtered; **12 PNGs**, `current-captures.log`. Renderer environment below. |
| `CARGO_TARGET_DIR="$PWD/target" cargo test --locked --offline --manifest-path "$R/gui-probe/Cargo.toml" -p regressee-gui --all-features --bin phase4-review-desktop independent_review::renderer_capture_review_edges -- --exact --ignored --nocapture --test-threads=1` | **1 passed**, 309 filtered at capture time; **4 PNGs**, `edge-captures-final.log`. |
| `cargo check --locked --offline --workspace --all-targets --all-features` | Exit **0**, `check-workspace.log`. |
| `git diff --check` | Exit **0** before/final report validation. |

Thus **112 in-repository positive test passes**, plus **3 independent positive
engine probes**; repeated runs and defect witnesses are not added to that total.
Full expanded argv/timings are in `commands.json`, `probe-commands.json`,
`final-batch-commands.json`, `edge-confirmation-commands.json`; final standalone
witness commands are reproduced above. `test-summary.json` preserves per-log
counts. No unchanged 1,000-page scale run or full suite containing the five
independently attributed inherited GUI failures was repeated or waived.

Probe development logs are retained honestly: missing scratch imports, an initial
scrollbar coordinate using the content rather than reserved gutter, a comparison
that requested a different analysis than its pre-state, and a stale shared-target
binary yielding **zero** edge tests were corrected only in scratch. Renaming the
probe binary produced the actual four edge captures. An initial real-drag
negative-control assumption failed: input moves both during transition and after
Active, as the final witness demonstrates. Earlier zero-test/fixture failures are
not represented as successful product verification.

## Finish review: floor, readability, keyboard and documentation

- Inspected supplied round-2 captures (contact sheet plus full-size representative
  frames); all **12 PNG hashes match their metadata**. Their recorded main-source
  hash predates the final handoff, so they alone do not certify final source.
- Independently rendered the current production source's **12** normal frames:
  1440×900, 1280×800, 820×600 × Light/Dark × Active/Human. All metadata binds the
  current main-source hash; each PNG hash verifies. Four additional edge fixtures
  cover no-follow, expiry, failure and the long human request. This was a bounded
  evidence batch, not an aesthetic repair loop. No production UI was edited.
- Ordinary banner is a readable **87px** at 820×600, with **40px** named actions,
  15px body/system type, existing neutral/teal roles and a visible focus ring.
  No new assets, gradients, tint on screenshot originals or geometry redesign.
  Long-message compactness and expiry-state completeness fail as described above.
- Source-token contrast ratios: Light body **15.52:1**, help **7.00:1**, primary
  **5.29:1**; Dark body **13.42:1**, help **7.92:1**, primary **8.73:1**. This passes
  the text floor, not screen-reader or native presentation qualification.
- Existing mounted-control tests exercise pointer Take control, Tab focus and
  Ctrl+Shift+T takeover over real IPC; menu/shell shortcuts and focused-field/IME
  ordinary editing reject in the tested lock states. Return draft/modal refusal
  and Cancel/Discard quit paths pass. Held scrolling is a real uncovered exception.
  Native keyboard dispatch, IME and accessibility are still unqualified.
- CLI/engine documentation accurately describes lease bounds, named/headless
  selection, explicit confirmations, no-GUI handoff, frozen jobs, restart and
  same-user limits. Its claims that anonymous clients cannot bypass control and
  all scrolling is locked need repair to become true. Long/expired UI states need
  coverage before the complete UX claims can pass. No new design tokens are needed.
- Integration follow-up: classify this new report as internal documentation in
  `scripts/source_release.py` alongside the other review reports. The reviewer did
  not edit that production packaging policy. Existing sidecar/brief drift remains
  outside scope.

Renderer commands set `REGRESSEE_RENDERER_APPROVED_ROOT="$R"` and
`REGRESSEE_RENDERER_CAPTURE_DIR="$R/captures"` or `"$R/edge-captures"`; those
explicit roots are private 0700 directories. All 16 PNG/metadata identities,
probe hashes, contrast values and unchanged-source proof are recorded in
`review-evidence.json`. Contact sheets are diagnostic derivatives, not shipping
assets or screenshot originals.

## Native/platform boundary and repair return

Both `DISPLAY` and `WAYLAND_DISPLAY` are unset; no authorized display/socket was
supplied. Actual native-window input was **not attempted or invented**. GPUI/Blade
offscreen rendering, simulated pointer/keyboard/IME and real Linux IPC do not
qualify compositor input, accessibility, Windows/macOS IPC or desktop behavior,
managed login, browser sandbox/fonts, clean headless deployment or release packages.

Return findings to the original implementor. Repair all five at their actual
authority/input/hydration/layout boundaries, add correct-expectation regressions,
rebuild both protocol-5 clients/engine and recapture the affected bounded states.
Preserve the original external witnesses and this report; after repairs those
wrong-behavior witnesses should cease to pass, rather than being relabelled
acceptance tests. Resume this independent reviewer for a scoped re-review.
**No Phase 4 PASS, native gate PASS or Phase 5 authorization is implied.**
