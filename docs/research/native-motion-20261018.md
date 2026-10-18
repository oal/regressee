# Restrained native motion — 2026-10-18

## Approved direction

The user approved the app-wide motion proposal and requested implementation by
subagents followed by verification. This is an Operate-mode refinement of the
current unified-neutral/teal Clear wayfinding workspace, not a redesign.

- **Focal moment:** an authoritative task reply reveals its existing outcome
  mark and settles the next-action area. Success, attention and failure retain
  their truthful icons/copy. No automatic navigation or baseline approval.
- **Continuity:** native menus, attached inspectors and in-app dialogs gain
  short, restrained reveals. Shell context and evidence stay anchored.
- **Feedback:** ease hover/selection paint, acknowledge review choices, and
  retarget determinate progress only between actual reported values.
- **Budget:** finite native frame scheduling, shared timing/easing, no new
  animation dependency, no per-row list entrances or expensive full-window
  effects. Existing activity loops stop when hidden/unmounted/reduced.

## Preservation and accessibility

Focus rings, press feedback, disabled states, validation, operation/consent
guards and keyboard handling remain immediate. Hit targets and layout do not
move. Screenshot replacement, screenshot/DOM switching, evidence source text,
pan/zoom, split dragging, ignored-area drawing and filtering remain immediate.
Never blend, tint or animate captured evidence, including while selecting the
next result. Do not animate validation or disclosure height.

An explicit remembered machine-local Reduce motion setting must be available
in App settings. It applies immediately, preserves static state feedback and
does not change website capture preferences or exports. The existing
`REGRESSEE_REDUCED_MOTION=1` override remains authoritative. GPUI does not expose
the desktop motion preference, so no automatic OS following is claimed.

Logical dismissal, focus restoration and removal of handlers must not wait for
an animation. Do not retain an interactive closing overlay just to achieve an
exit effect. Rapid changes must retarget from current paint rather than replay
an unrelated entrance.

## Verification

**Independent scoped PASS.** Implementation, the single fix batch and the sole
confirmation are complete. All three findings below are resolved. This is not a
whole-app, performance, native-input, accessibility or release pass.

The first independent bounded pass returned **FIX**: 42 focused tests passed,
one Settings keyboard expectation failed, and one native capture test produced
100 real GPUI/Blade frames. The reviewer found no evidence mutation, accidental
approval, automatic continuation, control-layout animation or delayed logical
focus in the exercised paths. Actual Settings toggle/restart persistence passed.

All first-round findings were handled in one batch:

1. Wire the previously unused Feedback entrance around the actual feedback
   surface, not the working-area parent. Retain immediate message/dismissal
   state and layout; fade only that surface's paint.
2. Extend the existing complete Settings keyboard-order expectation with Normal
   and Reduced. Keep its two-size, expanded/collapsed and forward/reverse checks.
3. Persist and await the actual reduced-motion preference in the task-outcome
   regression, then assert the effective global after first paint. A manually
   set global before first render had been overwritten by the real preference.

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-native-motion-20261018-r7qx28e2/`.
`independent-review.md` retains the original FIX verdict and exact commands;
`inspection/` is not overwritten. `confirmation-review.md` records the final
scoped PASS; `confirmation/` contains the only final visual round after that
batch. The two-round visual budget is complete; no further polishing round.

### Current confirmation

- **25 focused tests passed, zero failed**, plus the passing native capture
  test. These are confirmation counts, not accumulated first-round totals.
- The production-feature desktop **`ui` build passed** after the final source
  change. No inspection/renderer feature is required by the shipped UI.
- **100 current-source GPUI/Blade PNGs**: both themes at 1280×800 and 820×600.
  All 24 normal-motion sequences have distinct intermediate frames at 40 ms;
  all 12 reduced feedback/dialog/completion sequences are pixel-identical at
  0/220 ms. Four additional Settings captures prove the actual toggle, flush and
  new-owner/new-shell restart path.
- The reviewer inspected all final frames using native-image contact sheets and
  full-resolution minimum-size feedback/Settings frames. No remaining material
  motion, clipping, legibility, focus-visibility or layout finding was identified
  in the exercised scope. The Settings regression retains its complete two-size,
  collapsed/expanded, forward/reverse traversal and reveal assertions.
- Screenshot-stage geometry/interior pixels remain unchanged during hover and
  feedback settling. Project/evidence JSON, drafts and original bytes are
  unchanged. The separate After-stage hover crop also remains unchanged; that
  crop's geometry is source-derived rather than independently measured.
- All 100 PNG hashes/dimensions/opacity checks passed. **109 embedded hashes per
  frame** match final sources/assets (10,900 comparisons); the source receipt
  binds 258 Rust/input hashes. Original inspection evidence remains preserved.

Exact commands/results, source/frame receipts, scope and qualification limits
live alongside `confirmation-review.md`. The permanent capture harness now fails
if any normal scene lacks a genuine intermediate state, or if sampled reduced
feedback/dialog/completion animates. The small synthetic 96×64 website fixtures
establish evidence preservation and shell behavior, not realistic website-content
coverage. No user project, real website or network capture was accessed.

Strict scoped Clippy did not pass in the first review:
`crates/gui/src/jobs.rs:564` reports
`clippy::infallible_destructuring_match` when the local-test backend is absent.
That source is unchanged; the identical known-blocked command was not repeated.
No A/B attribution was performed and no unrelated source was repaired. This is
not a whole-suite or whole-lint pass. The development-host checks do not qualify
native OS presentation/input, accessibility/IME, Windows/macOS, compositor frame
pacing or public release. Only synthetic offline projects were used; no user
project or website was accessed.

## Source-backed implementation

`crates/gui/src/ui/motion.rs` owns shared, monotone exponential easing and
interruptible native paint state. No animation dependency was added.

| Motion | Duration | Behavior |
| --- | --- | --- |
| Control feedback | 120 ms | Retarget hover/selected background from its current painted color; press, focus, disabled state, borders and check/radio marks remain immediate. |
| Determinate progress | 160 ms | Ease only between reported fractions. Changed totals and resets snap; a fill never leads the latest reported work. Numeric copy remains immediate. |
| Menu entrance | 120 ms | Native opacity-only reveal; View, Zoom and timezone menus retain their real bounds and keyboard behavior. |
| Panel / dialog entrance | 160 ms | Attached panels and assembled dialog surfaces retain their exact layout node and hitboxes. Dialog scrims fade separately with immediate occlusion. |
| Feedback entrance | 140 ms | Reveal an existing message in place; no animated layout or disappearing error state. |
| Task outcome | 180 ms | Reveal the authoritative outcome mark and next actions once per task identity. No automatic continuation. |

Entrances start at **94% opacity** rather than hiding interactive content. The
native public API cannot translate paint independently of hitboxes, so the
proposed spatial offsets were deliberately replaced by opacity-only arrivals.
Exits remain immediate: no stale handlers, retained focus stops or delayed focus
restoration. Normal theme changes snap rather than interpolating palettes.

Shared controls and toolbar tools inherit color feedback. The existing Results
queue, review choices, navigation and History controls retain their geometry and
semantic behavior rather than introducing per-row entrances. The viewer's
Details, Analysis, Image controls and note inspector animate only their flyout;
the screenshot and DOM evidence stages never become animation subjects.
Page options/mapping previews and View/Zoom/timezone menus use the same entrance
primitive. In-app confirmations, About/help, tasks and screenshot/DOM baseline
approval dialogs use the shared dialog presentation. Platform file pickers stay
platform-owned. Optional whole-page transitions and disclosure-height animation
were intentionally not added.

### Reduced motion and persistence

App settings exposes **Reduce motion → Normal / Reduced** after Appearance.
The choice applies immediately to cached children and existing activity clocks.
The explicit environment override forces Reduced and disables conflicting
choices with an explanation. No OS-motion following is invented.

`AppPreferences::reduced_motion` defaults to false for existing files. The
existing async preference worker and engine apply field-level updates so changing
motion preserves appearance and vice versa, including rapid choices, retries and
an older completion arriving after newer intent. Atomic protected machine-local
storage and truthful session-only failures remain. The local engine protocol is
**9**; an older running engine must restart. Project evidence, captures, baselines
and ZIP formats are unchanged.

## Post-confirmation startup correction

The user successfully stopped their engine, but the desktop still rejected the
leftover protocol-8 endpoint. Data-free executable metadata confirmed that the
CLI binary had remained at protocol 8 while the desktop was rebuilt at protocol
9. The shared upgrade client also recognized only protocols 6/7, omitting 8 from
both known-old explicit Stop negotiation and stopped-owner replacement.

`crates/app/src/engine/client.rs` now includes protocol 8 in those two narrow
lifecycle paths. Status/product operations still cannot cross versions; unknown
versions, busy refusal, identity/cache checks and live-owner locks remain
protected. A stopped endpoint is replaced only after proving the store unowned.
No project data, endpoint or lock file is deleted as a recovery shortcut.

- Three old-wire lifecycle unit tests pass, now exercising protocol 8 as well as
  earlier known versions, including active-owner, busy, identity/cache and unknown
  version refusals. The initial incorrectly named filter selected zero tests;
  the corrected `upgrade_tests` invocation ran all three.
- Two real-CLI lifecycle integration tests pass, including the new regression
  `stopped_protocol_eight_endpoint_upgrades_without_removing_data`.
- Both normal production-feature executables were rebuilt together and report
  protocol **9**, without developer/test/inspection features.
- An isolated smoke check using the actual desktop executable's engine-start
  path replaced a stopped protocol-8 descriptor, accepted the rebuilt CLI and
  preserved retained bytes. Only that isolated engine was stopped by the check.
  Fixture: `/var/home/olav/Data/opencode/tmp/opencode/regressee-protocol8-restart-6ml3yl3u/`.

This is a backend-only follow-up. Earlier visual receipts retain their original
source identities and are not relabelled as captures of this later client edit;
no motion/UI source changed and no third visual round was performed. No user
engine, project data or running work was stopped or modified by verification.
