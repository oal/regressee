# Independent UI sweep verification — 2026-10-10

**PARTIAL: targeted regressions PASS; native visual verification BLOCKED by
black owned-window pixels; two source-level issues remain. No visual PASS,
human acceptance or release approval.** Application source stayed frozen.

## Scope and candidate

Independent render/input/UI verification, no subagents and no application edits.
Read `PRODUCT.md`, `DESIGN.md`, discovery audit and component/input handoffs.
Direction remains Operate: preserve the light/teal native desktop identity.
The context loader's mobile references are not desktop qualification criteria.
Only this repository report and unique private evidence were added. No guardian,
browser, scaling, public-runtime or settled fault-pipeline suites were rerun.

Private physical ext4 evidence root, abbreviated **E** below:

`/path/to/private-tests/opencode/regressee-ui-sweep-hkonn1qy/verifier-a69456f1db/`

Native binary copied from `target/debug/regressee`; SHA-256:
`bbf35aa050b7dc75b80018793a6503b5e15c673ac9544a5ba648578ba78c53dd`.
`E/freeze.json` and `E/final-audit.json` confirm before/after equality for main,
core shell, shared controls/theme/input/GPU diagnostics, jobs, storage, app Cargo,
lockfile and original binary. No rebuild or startup fix was attempted.

## Independent bounded regressions

All cargo commands used `--locked --offline`, physical `TMPDIR=E/tmp`,
`LIBRARY_PATH=<project>/.runtime/link-libs`, and serial test execution. Logs:
`E/test-{0..5}.log`, `E/additional-test-{0..3}.log`, `E/tests.json`.

**22 distinct ordinary tests passed**, plus the explicitly ignored synthetic
fixture-preparation test when deliberately invoked. One filtered fixture test was
deliberately ignored in the initial regression selection, not a failure.

| Filter / check | Result and qualification |
| --- | --- |
| App `ui_sweep_` | 2 PASS: Phase5/Phase6 handle identity, field availability, selected hidden/no-op refusals; coordinated website/environment save, idempotent choice, active baseline retention, dirty-mask Refresh refusal |
| Shared controls style tests | 4 PASS: exact old GPUI text-refinement defect, state merge metrics, choice descriptions, drawn check bounds |
| Actual control bounds (`ui_tests`) | 1 PASS: real GPUI **test-platform** layout across focus/selection/disabled; not native pixels |
| Text input tests | 7 PASS: Unicode/UTF-16, composition runs/underline, shift/mouse selection, clipboard semantics, horizontal reveal and fixed field bounds; simulated platform only |
| Core shell tests | 5 PASS: traversal, busy/dialog rules, hidden screen controls, no duplicate candidates |
| Storage coordinated-save test | 1 PASS: stale revision and unknown environment refusal, injected local-settings INSERT failure rolls back project name/revision, successful atomic save/reopen, login URL absent from project JSON |
| Jobs save/reopen without runtime | 1 PASS: ordinary persistence does not require a configured browser |
| Main save/navigation/conflict test | 1 PASS: real competing writer returns `ConflictOrWrite`; draft/navigation retained, no retry loop |

Supplemental read-only `E/focus_candidates.rs` imports the frozen core shell and
executes candidate enumeration. It confirms the residual candidate holes below;
it is **not** a GPUI mounted-element or native-keyboard test. Compilation warnings
are unused APIs in this narrow standalone probe, not application clippy results.

## One correction-batch handoff

### P1 — candidate filtering still permits unmounted controls

The disjoint handle correction is sound, but `control_enabled` ends in `_ => true`
and some broad candidate ranges still outlive their rendered elements.

- **History:** core `FocusState::controls` includes `Phase6(13)` through `(0..16)`
  (`crates/core/src/shell.rs:318–321`). Main supplies no screen/mount prerequisite
  for 13 (`crates/app/src/main.rs:3275–3302`). Its actual button is Settings-only
  (`main.rs:3446`), absent from `baseline_history`. Tab can therefore select an
  invisible handle and keyboard activation queues `Work::RepairPreview`
  (`main.rs:1421`). This is not an automatic deletion: confirmations remain.
- **Results:** core includes Guided(31–36) (`shell.rs:296`), but
  `environment_actions` is rendered only from Setup's `environment_setup`
  (`main.rs:2480–2482,2535,2538`). Guided(31), (35), (36) fall through enabled;
  they are invisible Results stops. Other entries depend on login state but
  still lack a mount check.
- **Empty Results:** `CrawlField(9)` remains enabled for any saved project
  (`main.rs:3312`), while `inventory_and_results` returns before mounting search
  when neither preview nor latest run exists (`main.rs:2661–2666`).
- **Baseline-only viewer:** Phase5(0–4), (6–9), and (14) pass default prerequisites
  when `focus.viewer=true`, even without a comparison. Analysis panel returns
  early without a comparison (`main.rs:4035–4037`); viewer-mode buttons mount
  only with one (`main.rs:3944–3965`).
- **Discovery preview:** Guided(19) passes when a preview exists
  (`main.rs:3183`), but the grouping checkbox mounts only in the captured-run
  branch (`main.rs:2770–2777`).

**Batch correction:** share explicit mount predicates with focus/render/dispatch,
or enumerate only the mounted section's controls. Keep backend refusal checks.
Add shell-level tests that compare enabled Tab candidates to actual mounted
handles for empty Results, baseline-only viewer, discovery preview, comparison,
and History without previews. Existing duplicate/no-op tests do not cover this
invariant. Native reproduction was NOT RUN after the pixel gate failed.

### P2 — advanced reanalysis bypasses the dirty-mask refusal

`activate_control` refuses presets/manual reanalysis/analysis-history changes when
`mask_dirty` (`main.rs:2886–2899`), but advanced text polling schedules analysis
without that guard (`main.rs:765–807`). Advanced toggles Phase5(27/28) also reach
`schedule_analysis` (`main.rs:1067–1070`); neither the scheduler nor submit path
checks dirty masks (`main.rs:731–736,822–855`).

This does **not** establish rectangle loss: `Reply::Viewer` explicitly retains
draft rectangles when dirty (`main.rs:603–610`). It does mean supposedly refused
reanalysis can run against persisted masks while unsaved rectangles are displayed,
making the policy inconsistent and analysis harder to interpret.

**Batch correction:** apply the intended dirty-mask policy at the shared scheduling
boundary (including already-due work), keep drafts intact, and show the same
save-first explanation. Test advanced text and both checkboxes with a dirty mask,
not only Refresh/preset activation. Alternatively explicitly document/support
saved-mask-only reanalysis rather than claiming universal refusal.

## Native attempt: concrete blocker, no retry loop

Owned Unix-only Xvfb `:197`, fresh MIT cookie, `-nolisten tcp`, private HOME/XDG/
data/cache/config/state/runtime/TMP/cwd roots, allowlisted app environment. No
host display connection, window manipulation, root screenshot, clipboard, browser,
worker, distribution configuration, system installation or host settings change.

The first **server-only** setup attempt failed because `/usr/bin/xkbcomp` was
missing; zero app launches. `E/initial-setup-{result.json,xvfb.log}` preserve it.
One concrete private correction downloaded/extracted Ubuntu's x11-xkb-utils under
E (`E/setup-correction.json` records HTTPS origin and SHA-256). A tiny server-only
preload points the exported `XkbBinDirectory` at that private compiler. It is not
loaded by the app. Server paths/transport use Xorg's own conventions; Rust TMPDIR
and application roots remained physical private disk paths. No host files were
installed or modified to supply the missing executable.

The corrected server started. The app launched **once**, PID **484755**; server
PID **484746**. App used only `VK_DRIVER_FILES=/usr/share/vulkan/icd.d/lvp_icd.json`
and `REGRESSEE_GPU_DIAGNOSTICS=1`, with no Wayland/ZED/headless/other GPU overrides
inherited. Actual log identifies **llvmpipe (LLVM 21.1.8, 256 bits)**. No app
`LD_PRELOAD` and no hardware/default-Wayland qualification.

The exact PID/title-owned client naturally mapped, with Map/Visibility/Expose
events. At ~1.26 seconds, geometry was **1120×800**, depth 32, map state 2,
`override_redirect=false`. `XGetImage` captured ONLY that client.

**`E/projects-initial.png` was read with the image tool: entirely black, no readable
labels, hierarchy or controls.** It has 896,000/896,000 black pixels and one RGB
color; SHA-256:
`0fbef93e6b721fb075c6a2846987f1c3789eeb44d26a0a0b050a6e6b500a82a0`.
The pixel gate stopped the attempt. Zero focus/click/key/drag events, zero forced
mapping, zero app confirmations. No longer-duration pixel claim is made from
this single sample; no speculative GPU flags or subsequent app launch followed.

This is **black pixels on this isolated software X11 test route**, not the earlier
shared-X11 swapchain crash, not a refutation of the user's working desktop app,
and not proof of the underlying renderer/compositor cause. Adapter initialization
does not prove presentation. `--check-renderer` was not used as visual acceptance.

## Fixture, privacy and cleanup

The allowed fixture-preparation command produced one **SYNTHETIC** project,
“Approval UI fixture”, loopback URL, one active baseline, two runs/two analyses,
one inventory and one mask. No website capture or browser launch. After exit,
SQLite integrity is `ok`; local environment/session tables are empty. Fixture
rows, image/artifact hashes and history remain private in E; see
`final-after-exit-sqlite.json`, `*-rows.json`, `evidence-inventory.json`.
No before-launch artifact hash inventory was made, so a before/after original
corpus immutability assertion is not claimed from this native attempt.

The app was stopped via exact owned PID SIGTERM (exit -15), reaped; Xvfb stopped
and reaped (exit 0); controller completed. Final exact app/server PID audit is
empty, owned display lock/socket and compiled keymap are absent. Private root and
ancestors are not symlinks. Host MemAvailable was ~38.9 GiB at launch; observed
cgroup limit was unlimited, OOM counters zero. Hidden ancestor limits were not
visible in this namespace and are not qualified.

## Final scope verdict

| Scope | Verdict |
| --- | --- |
| Frozen source/binary preservation, private isolation, cleanup | PASS |
| Exact old focus defect / fixed style and test-platform geometry | PASS |
| Save atomicity, conflict classification, baseline retention, dirty Refresh guard | PASS, targeted simulated/storage evidence |
| Complete enabled-candidate/mounted-control correspondence | **ISSUE**, source/probe evidence above |
| Universal dirty-mask reanalysis refusal | **ISSUE**, source-path evidence above; no loss claimed |
| Native mapped client | PASS on isolated software X11 only |
| Readable native rendered pixels | **BLOCKED/FAIL at sampled pixel gate** |
| All five screens, 820×600, focus/selected/disabled visuals, hierarchy/blandness judgment | **NOT RUN** |
| Actual keyboard/drag/IME, create/save/reopen, populated viewer/history inspection | **NOT RUN** |
| Hardware/default Wayland, OS scaling, screen readers, cross-platform, release/human acceptance | **NOT RUN / unclaimed** |

Hand off the two source issues together. The native blocker is a separate
evidence limitation, not an invitation to broad renderer experimentation or an
unbounded polish loop. No screenshot from a mock/browser replaces the missing
native visual review.
