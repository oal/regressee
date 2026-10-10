# Phase 1 — local foundation implementation delivered

Date: 2026-10-09. **Bounded corrections delivered; independent re-verification
pending; not fully accepted.**
This implementation session stops at Phase 1. It did not spawn agents, implement
Phase 2, or claim Phase 0/external/native/human-design gates passed. Decision
0005 authorizes local continuation while those gates remain open.

## Corrective follow-up — 2026-10-09

The independent report `docs/phases/01-verification.md` found a P1 dangling
database-symlink safety defect and two P2 session-contract/stale-Projects issues.
That report is preserved unchanged. The separate bounded implementation in
`docs/phases/01-corrections.md` repairs all three, adds exact-cause regressions,
and passes the locked foundation checks (49 Rust tests), four Node tests,
headless persistence and retained supervisor protocol tests. Corrections are
delivered, **not independently re-verified yet**. No Phase 2 or repeated native
attempts occurred; all original native/external/human gates remain open. The
implementation evidence below is the original delivery record.

## Delivered files and behavior

| Area | Files | Actual behavior / boundary |
| --- | --- | --- |
| Maintained workspace | `Cargo.toml`, `Cargo.lock`, existing `rust-toolchain.toml`; new core/storage/runtime/app manifests | Exact Rust 1.98.1, GPUI 0.2.2 and direct dependency pins; locked builds/tests |
| Product contracts | `crates/core/src/{domain,archive,shell}.rs` | Typed project/environment/inventory/profile/run/target/baseline/analysis/review/mask/artifact/session references; validation and focus-state tests |
| SQLite | `crates/storage/src/lib.rs`, `migrations/001-projects.sql`, `002-artifacts.sql` | Atomic versioned migrations, future-schema refusal, actual project save/reopen/edit with revision-conflict checks |
| Evidence storage | same storage module | Streamed SHA-256 objects, no-clobber atomic publication, fsync, immutable metadata, verified references, explicit unreferenced collection only; 128 MiB/object cap |
| OS directories | storage `AppPaths` | OS-local data and separate cache, no cwd defaults; explicit isolated developer override; private Unix permissions |
| Product runtime boundaries | `crates/runtime/src/{protocol,supervisor,secrets}.rs` | v2 command/event envelopes, safe integer IDs, bounded newline decoding, correlation/ack checks; supervisor trait/config and fail-closed secret abstraction, not a product process/login implementation |
| Worker contract package | `worker/product/{package.json,package-lock.json,src/protocol.mjs,src/protocol.d.ts,test/protocol.test.mjs}` | Independently locked browser-free product protocol declarations and decoder/whitelist tests; separate from fixture worker v1 |
| Bounded storage owner | `crates/app/src/jobs.rs` | DB/file work on one background thread; four-command/four-update channels, nonblocking submit, terminal reply retry, bounded UI drains; no UI-thread join |
| Native shell | `crates/app/src/main.rs`, `ui/{controls,theme,text_input}.rs` | Projects/setup/results/history/settings; real create/save/reopen actions, paging, storage summary, native menu/titlebar/picker/discard prompts; honest not-built/empty states |
| Reusable native UI | same UI modules | Form, action, status, error, empty, list, information-dialog and progress primitives; active-screen focus order, dialog trap/restore, shortcuts, clipboard and IME tests |
| Headless verification | `crates/app/src/foundation.rs` | Creates a real isolated DB/project/object, closes/reopens it and emits a report; refuses an existing evidence directory |
| Tooling / matrix checks | `scripts/check-foundation.sh`, `scripts/foundation-native.py`, `scripts/build-ui.sh`, `.github/workflows/foundation.yml` | Locked UI build + tests/lint; one owned-window native inspection; three-OS CI definitions, **remote jobs not run** |
| Design / architecture | `DESIGN.md`, `.impeccable/design.json`, `docs/foundation-surface.md`, `docs/phase0-design.md`, decision 0006, README | Built-source visual proposal, preserved original bench record, topology/vocabulary/flow sketches and verifier review boundary |
| Licensing | existing official `LICENSE`, `THIRD_PARTY_NOTICES.md`, source SPDX headers | MPL foundation; shared Apache GPUI input remains Apache with attribution, not relicensed; release-wide license audit pending |

### Phase 0 reuse, without product/fixture confusion

The fixture bench is retained in `crates/spike` and root `worker/src`. Its
supervisor now calls the maintained bounded frame reader; existing cancellation,
owned scratch, launch-race and detached-browser cleanup fixes remain intact.
Fault tests exercise that actual supervisor. The Apache input adaptation moved
to app UI and is reused by the bench; product code never depends on the spike.
Two old fixed artifact test locations and PID-based test scratch were converted
to unique RAII temporary directories, so repeated unit tests no longer write
into historical artifact trees. No existing history was recursively copied or
deleted. Initial dependency-resolution tests had still used the old two fixed
unit-fixture locations; no historical qualification report/capture was overwritten.

Projects contain saved settings only. New capture profiles are explicitly
`Unconfigured`; no mocked discovery, fake successful capture or manufactured
comparison is persisted or displayed as real evidence. The headless artifact is
text labeled as a synthetic **storage fixture**, not a website screenshot/run.

## Decisions

`docs/decisions/0006-foundation-boundaries.md` records the minimal four-crate
foundation, narrowly used SQL tables rather than speculative future entity
tables, writer-lock/reference safety, off-thread IO, separate v2 protocol,
session/portable boundaries and proposed native visual world.

Impeccable context and new-work/Operate guidance were read; the incumbent native
capture was inspected (black, not valid visual evidence), the concept seed was
run and assigned candidate 5 acknowledged. Craft-floor guidance was read before
UI work. The question catalog was actually queried and returned no question tool;
the explicit local proposal authorization was used instead of claiming an answer.
There is **no invented human visual approval**. The source-derived documentation
pass ran inline because concurrent reviewer/documenter agents are forbidden by
the user's serial policy. This is not independent finish review. Native detector
was not run: its HTML/CSS detector cannot qualify GPUI. No shipping raster assets.

## Actual commands and results

Working directory: `/path/to/Regressee`.
Fresh retained evidence:
`artifacts/phase1-implementation-2026-10-09-1791557069898518533/`.
`commands.json`, per-command stdout/stderr, headless report, native report,
preservation hashes, source contrast and resource audit are in that directory.

| Command | Result |
| --- | --- |
| `cargo test --workspace -j 4` (initial dependency resolution) | PASS; generated the new lock graph; initial baseline tests before UI-test additions |
| `cargo fmt --all` | PASS; formatting applied before final checks |
| `bash scripts/check-foundation.sh` | PASS, 9.629 s total; commands detailed below |
| ↳ `cargo build --locked --workspace --features ui --bins -j 4` | PASS, product and retained bench linked on local Linux with project-local linker-name symlinks |
| ↳ `cargo fmt --all -- --check` | PASS |
| ↳ `cargo test --locked --workspace --all-features -j 4` | **43 Rust tests PASS**: 32 foundation tests plus 11 retained spike tests; includes simulated GPUI input and actual shell focus/actions |
| ↳ `cargo clippy --locked --workspace --all-targets --all-features -j 4 -- -D warnings` | PASS, UI-inclusive; upstream future-Rust warning remains |
| `cargo run --locked --bin regressee-foundation -- artifacts/phase1-implementation-2026-10-09-1791557069898518533/headless` | PASS, 1.065 s; DB schema 2; actual save/reopen, object integrity and referenced-object collection refusal |
| `.runtime/installed/node/bin/node --test worker/product/test/protocol.test.mjs` | **3 tests PASS**, 0.064 s; whitelist/version/correlation/frame/UTF-8/address boundaries |
| `cargo run --locked --bin regressee-spike -- protocol-test` | PASS, 61.402 s; browser crash/relaunch, bounded cancellation, worker crash/restart, command whitelist, oversized frame rejection using the shared decoder |
| `python3 scripts/foundation-native.py --output artifacts/phase1-implementation-2026-10-09-1791557069898518533/native` | **FAIL / native evidence open**, driver exit 1; details below |
| Read-only Python source-token contrast calculation | PASS for nine normal text pairs, minimum 5.15:1; **source-only**, not rendered/accessibility evidence |
| Read-only `qualify.scoped_audit()` and broader `/proc/exe` project/private-runtime scan | PASS, zero live app/Node/Chromium processes, empty owned scratch/setup stages |
| SHA-256 checks of 11 protected historical report/capture/runtime files | PASS, all unchanged |

Initial implementation checks caught and fixed a GPUI element-ID conversion,
format-related clippy warning and test-macro glob-import recursion. These were
intermediate failed checks, not hidden final passes. The final locked command
batch above passed. `proc-macro-error2 2.0.1` still emits its existing upstream
future-incompatibility warning; do not claim a warning-free future Rust graph.

The initial product native attempt used only X11/lavapipe, explicitly a developer
probe, not default Wayland/hardware graphics. It found owned PID **764681**, window
**37748737**, but it remained `map_state=0` after the mapping grace period. No
forced map, virtual display, focus/input, screenshot, second attempt or broad
graphics diagnosis was made. The own process was terminated and reaped. A small
test-driver text-entry correction was made afterward; that unreachable input
branch has **not been exercised**. This report does not relabel failure as success
because the wrapper retained its JSON. The driver exit and native outcome are both
recorded. No safe scope-local mapping fix was apparent.

## Limitations and open qualification

- **Human visual review pending:** setup/results composition, proposal tokens,
  native typography, real wrapping, disabled/hover/focus/pressed states and ordinary
  task comprehension. Source contrast is not a visual verdict.
- **Native rendering/input open:** no meaningful product pixels or desktop input
  evidence acquired here. Simulated GPUI tests prove only their stated action /
  focus / UTF-16 logic. File/discard dialogs, OS menus and screen readers untested.
- **Windows/macOS / supported clean Linux / hardware graphics NOT RUN.** Defined
  CI jobs were not dispatched; no OS/architecture support claim was added.
- This shell is a fixed light proposal, not qualified host-theme/high-contrast
  integration. Advanced grapheme selection/drag/word navigation and close/quit
  confirmation for unsaved edits remain UX work. Navigation retains drafts;
  project switches use discard confirmation, but process exit does not autosave.
- Immutable future run/baseline ownership commits, crash repair, storage pressure
  monitoring, large result virtualization, image ingestion/decode in product,
  cleanup UI and full ZIP validation are later-phase work. Only narrow project /
  artifact tables exist now. No automatic evidence GC.
- Artifact API guards cooperating local writers and accidental damage, not a
  malicious same-user filesystem race. Windows directory-sync/ACL/power-loss
  semantics require actual qualification. A publication/metadata crash can leave
  a complete orphan; that is not silently deleted.
- The product supervisor remains an interface. Phase 0 Linux lifecycle fixes must
  be carried into the actual Phase 2 adapter and independently exercised. Product
  stderr bounds/deadlines and cross-platform whole-process-tree ownership are not
  qualified by the bench's Linux pass.
- No persistent secure sessions, keychain implementation, automatic runtime
  delivery, signed product pack, ZIP feature or public release is implemented.
  The historical fixture runtime is not qualified for real websites.

## Independent-verifier handoff — next strictly serial agent

Do **not** implement Phase 2. Read PRODUCT/PLAN, Phase 1 checklist, decisions 0005 /
0006, this report, DESIGN and the surface brief. Check the maintained source and
honest product/fixture boundaries rather than accepting this implementer's summary.

```sh
bash scripts/check-foundation.sh
# Use a fresh directory; keep the implementer's headless DB/evidence unchanged:
cargo run --locked --bin regressee-foundation -- /tmp/opencode/regressee-phase1-verifier-UNIQUE
.runtime/installed/node/bin/node --test worker/product/test/protocol.test.mjs
cargo run --locked --bin regressee-spike -- protocol-test
```

Inspect migrations, revision conflicts, duplicate/corrupt/interrupted/oversized
artifacts, reference preservation across independent stores, protocol versions /
frame limits, secret redaction/export exclusions and UI-thread blocking paths.
Re-test the same UI actions, modal/disabled/hidden-control focus and Unicode/IME
boundaries through the simulated GPUI tests without calling them desktop evidence.

Native command, **only if a bounded ordinary owned-window test is meaningful on
the verifier's available desktop**:

```sh
bash scripts/build-ui.sh
python3 scripts/foundation-native.py --output /tmp/opencode/regressee-phase1-native-verifier-UNIQUE
```

This helper deliberately selects software X11; it cannot qualify default hardware
graphics. On an actual supported desktop also exercise the ordinary `regressee`
binary, create/save/restart/reopen, Tab/Shift-Tab/Enter/Space/shortcuts, resize,
errors/empty/loading state, native prompts, focus restoration and Quit; capture
owned setup/results/history/settings windows and request actual human visual
confirmation. Follow the craft floor and one batched inspection/fix plus at most
one confirmation. If pixels are black/unmapped, stop; do not repeat mapping loops
or call test-only workarounds ordinary qualification. Carry unavailable tests open.

Preserve all earlier reports and artifact files; use fresh report paths, no
recursive history duplication, no global installs/sudo/sandbox disabling/services.
Audit/reap only owned processes/scratch. Write a separate Phase 1 verification
report and update `docs/IMPLEMENTATION_STATUS.md` with the actual scoped verdict.
Human visual and external platform acceptance stay pending unless real evidence
changes them; local delivery is not full release acceptance.
