# Native UI sweep — implementation, 2026-10-10

## Scope and disposition

The user requested a full UI sweep after reporting that focused buttons grow,
setup buttons change text without explaining their alternatives, and the app
looks bland/unfriendly. This work covers Projects, Setup, Results, History and
Settings in the native Rust/GPUI app. It preserves the light/teal operational
identity and strengthens hierarchy, states and ordinary control behavior.

**IMPLEMENTED LOCALLY; two subsequent findings independently repaired;
native visual/input acceptance blocked.** The built
`target/debug/regressee` includes this sweep. Local tests/builds are not native
visual acceptance. No commits, publication, release qualification or public-site
runtime approval is implied.

User-reported plain launch success is distinct from earlier retained Vulkan
failures. No renderer repair was made and the original startup cause remains
unknown. New filtered renderer diagnostics and `--check-renderer` help isolate
backend initialization without opening project storage; that command does not
verify a presented window or input.

## Delivered changes

- **Shared controls:** exact GPUI text-refinement regression reproduced in a
  test. Color-only focus replaced the whole text refinement, losing 14px/medium
  settings and inheriting 15px/normal. State styling now changes paint only;
  keyboard focus has a separate outline and primary actions stay teal. Drawn
  radio/check indicators, stable labels, 40px-minimum targets and an outlined
  destructive-action variant are shared primitives.
- **Setup:** discovery, capture extent, image mode, speed, access, login retention
  and website theme are explicit choices. Screen sizes, website approval and
  advanced boolean settings use checkboxes. Approval still validates website
  boundaries and does not grant permission to visit another site implicitly.
- **Application shell:** current project context and errors/progress are outside
  the scrolling screen. A pinned Setup footer holds the single **Save settings**
  action and discovery entry point. Messages can be dismissed explicitly.
- **Persistence:** the single save commits project and selected local login
  configuration in one SQLite transaction, with the same revision checks.
  A failed local-settings write rolls the project write back. Authentication
  settings remain outside portable project JSON; active baseline references and
  frozen capture inventory are not changed by setup edits.
- **Focus/actions:** History baseline controls no longer share handles with ZIP
  controls. Pointer activation, keyboard activation and Tab filtering share
  prerequisites. Hidden/non-actionable controls are filtered. Environment and
  network fields no longer depend on an unrelated Advanced toggle.
- **Inputs:** anchored Shift navigation, mouse drag, Shift-click and basic
  double-click word selection; composition underline and active-edge horizontal
  reveal. UTF-16 conversion, byte limits, clipboard bytes and disabled guards
  remain. Locale/grapheme navigation and real native IME qualification remain
  separate limitations.
- **Results:** selected viewer modes and sensitivity presets are visible;
  screenshot panes precede inventory and technical analysis. Result filters are
  explicit choices, visible counts and empty-search guidance are provided, and
  numeric detail is disclosed rather than always competing with the screenshots.
  Refresh/reanalysis refuses to discard unsaved ignored rectangles.
- **Projects/History/Settings:** selected-project context, clearer task labels,
  human-readable storage units and run states, provenance disclosure, destructive
  styling and shorter primary guidance. Privacy, incomplete-coverage and import
  provenance warnings remain visible; advanced limitations are still available.

## Local implementation checks

From the repository root, using the approved physical TMPDIR and development
linker-library directory:

```sh
env TMPDIR="$(realpath /path/to/private-tests/opencode)" \
  LIBRARY_PATH="$PWD/.runtime/link-libs${LIBRARY_PATH:+:$LIBRARY_PATH}" \
  cargo test --locked --offline -p regressee-core -p regressee-storage \
    -p regressee-app --all-features --lib --bins -j 4 -- --test-threads=1
```

Initial sweep run: **113 passed, four deliberate ignores**. This includes
merged GPUI style/geometry, input selection, distinct focus handles, coordinated
save, revision conflict, write rollback, local-login privacy, baseline preservation
and unsaved-mask preservation tests. The ignored native fixture preparer writes
synthetic saved-corpus pixels and metadata, not browser capture evidence.

Also passed: scoped all-target/all-feature Clippy with warnings denied, formatting,
and the locked offline `ui` executable build. An upstream `proc-macro-error2`
future-compatibility warning remains; it is not treated as a passed release gate.

### Preserved implementation failures

These were corrected before the source freeze, not reported as first-run passes:

- Shared-controls test attribute resolution initially recursed through GPUI's
  wildcard-imported test macro. Explicit built-in test macro imports corrected it.
- The input geometry test used an ordinary ID instead of GPUI's measured debug
  selector and an unstable test mount. The corrected test retains bounds and
  scrolling assertions and adds an exact 160×38px field check.
- Main regression expectations needed the new focus order and deliberate
  refusal to replace unsaved masks. Coordinated saves also needed the existing
  typed conflict response rather than generic Product errors.
- Clippy rejected a deliberately reversed literal selection range and two minor
  main test/match forms; the reversed-range probe remains via explicit fields.

## Independent/native results and one correction batch

Verifier `ses_ed92df55affecZAXJ6ZNa0t6ih` completed the initial source-matched
check and native attempt in `docs/research/ui-sweep-verification-20261010.md`.
Twenty-two targeted tests passed. A naturally mapped isolated X11/llvmpipe
window captured entirely black pixels; it was read as an image, and the input
gate stopped without clicks/keys. This does not explain or contradict the user's
working plain-launch report. Owned app/server/controller processes were settled.

Its two findings were corrected in one source batch:

1. Explicit per-screen/per-section mount predicates now guard rendering-enabled
   states, dispatch and Tab. A new test compares actual rendered-frame GPUI Tab
   handles with enabled candidates in both directions across 11 states, including
   empty Results, baseline-only viewer, discovery, advanced/loading comparison,
   History, Settings, Projects and simple/advanced Setup. It deliberately uses a
   tall test viewport to inspect membership, not native layout/readability.
2. The shared analysis scheduler and due-work boundary refuse dirty masks, cancel
   pending work, retain a save-first explanation and discard cancelled/dirty
   asynchronous analysis replies. Advanced text and both checkboxes are covered;
   drafts, displayed analysis, baseline and persisted history remain intact.
   Clearing dirty restores normal scheduling.

Correction integration initially exposed two old tests that supplied viewer or
project-saved flags on an empty shell without the now-required mounted project
sections. Fixtures now supply the matching project and actual Jobs load/settlement;
all original coordinate/filter/geometry/mask assertions remain. A second pointer
fixture failure exposed its uninitialized refresh, corrected through Jobs Load
rather than weakening guards. A new-test unused draw result warning was corrected.

Final parent gate: **115 passed / four deliberate ignores**, Clippy with warnings
denied and locked offline UI build PASS. Final independent correction gate:
**26 targeted tests PASS**, source/binary/evidence preservation and owned cleanup
PASS in `docs/research/ui-sweep-reverification-20261010.md`. Both source findings
are closed at this scope; no new native launch or screenshot was made. The current
binary has not received a five-screen native visual/input review. All sessions
and owned test processes are settled.

The native attempt used an owned private X11 display and isolated synthetic store,
not the user's open app or data. Test-only Xvfb libraries were privately
downloaded/extracted without installation. The software Vulkan backend
does not qualify default hardware, Wayland, high-DPI, another OS or a package.
One batched screen/input inspection and at most one concrete correction
confirmation are allowed. If pixels cannot be presented, native acceptance must
remain blocked rather than replaced with test-platform geometry claims.

Separate local guardian and 3,000-capture checks are settled and are not rerun
for this UI task. Public runtime, macOS ownership, full Windows integration,
Linux ABI/package, secure-store, accessibility, representative human usability
and signing/release gates remain outside this sweep's acceptance.
