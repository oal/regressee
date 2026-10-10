# Regressee — local visual regression checks

> Privacy: historical machine paths and authorized-site URLs use redacted
> example placeholders. They are not runnable configuration.

Native Rust + GPUI, MPL-2.0. **Phase 6's bounded local independent gate and
Phase 7's scoped safety/checkpoint/scale verification passed. Phase 7 remains
incomplete; not release-qualified.**
Guided crawling, authenticated capture, separate environment sessions, mapped
fixed-baseline checks, adjustable analysis, ignored regions, selective baseline
approvals, ZIP interchange and manual storage cleanup are real services wired to
GPUI. Current evidence and open qualifications are tracked in the status index.
The exercised historical browser is **development/owned-loopback-only**;
public-site capture is refused pending current secure-runtime qualification.

Start with the [first-run guide](docs/first-run-guide.md),
[private support matrix](docs/support-matrix.md),
[update/recovery policy](docs/updates-and-recovery.md) and
[private build/source instructions](docs/release-building.md).
Read [`docs/phases/07-status.md`](docs/phases/07-status.md),
the latest [independent repair and scale report](docs/phases/07-guardian-verification.md),
[`docs/IMPLEMENTATION_STATUS.md`](docs/IMPLEMENTATION_STATUS.md) and
[`DESIGN.md`](DESIGN.md). The sleek/modern visual system is a source-derived
proposal, **not human-approved**. Native rendering/input and external OS /
clean-machine gates remain open; decision 0005 permits continuing local work.

## Product foundation (developer commands)

```sh
bash scripts/check-foundation.sh       # locked UI build, format, tests, clippy
cargo run --locked --features ui --bin regressee
# On this Linux host, use the built binary after the linker-name helper:
target/debug/regressee
cargo run --locked --bin regressee-foundation -- /tmp/opencode/new-foundation-check
.runtime/installed/node/bin/node --test worker/product/test/protocol.test.mjs
# Full serial signed HTTP setup + actual product worker + headless loop (~6 min):
python3 scripts/check-phase2.py --output /tmp/opencode/regressee-phase2-UNIQUE
# Serial Phase 4 build/tests and fresh signed owned-loopback fixtures (~5 min here):
python3 scripts/check-phase4.py --output /tmp/opencode/regressee-phase4-UNIQUE
# Phase 6 serial product/recipient/archive checks; always use a fresh output:
python3 scripts/check-phase6.py --output /tmp/opencode/regressee-phase6-UNIQUE
```

The headless verifier requires a **new** directory and leaves its actual SQLite
project plus a labeled synthetic storage object there for inspection. It does
not capture a website. Default product data/cache use OS-local locations, never
the project cwd. `REGRESSEE_DEV_DATA=/absolute/fresh-test-directory` explicitly
isolates native developer checks. The app never launches the fixture worker.
Its product runtime must come through verified configured distribution trust.

## Guided workflow and setup boundary

Create/save a project → **Find pages** → review the inventory → **Capture baseline** → inspect warnings/evidence
→ explicitly **Save baseline…** → **Check for changes**. Checks create a separate
run of the same fixed targets/profiles. Before/after/diff panes share zoom/pan and bounded
viewer tiles; failed capture is never an unchanged verdict. Originals and prior
records are preserved after cancellation/restart. Sensitivity and ignored regions
can change after capture while originals and past analysis revisions remain
intact. Presets are fixture-calibrated, not proven real-site accuracy or automatic
defect verdicts. Selected compatible changes create a new immutable baseline;
previous versions remain available until explicitly deleted.

History supports selected-baseline ZIP export and import as a new local project.
Login state is excluded; recipients establish their own sessions. **Ignored
regions do not redact exported screenshots**, and private URL queries remain
visible. See [portable baseline help](docs/portable-baseline-archive.md) for
privacy, compatibility, archive limits, and protected manual cleanup.

For login/consent, save an environment, choose **Open browser**, finish normally,
then choose **I'm ready** to confirm replay on an application page in a fresh
context. Saved sessions are encrypted using an OS-protected key; unavailable
secure storage requires explicitly choosing **Ephemeral**, never plaintext.
Production and staging logins remain separate. Preview/probe changed mappings
before a full check. See [login and capture help](docs/auth-and-capture.md) for
read-only-account advice, exact limits and unqualified auth/network cases.

Settings → **Install browser now / Repair browser** uses real signed download/install
and sandboxed screenshot smoke on the background owner. No production host/key
is bundled yet; absent trust is explained and fails closed. Explicit installation
configuration uses `REGRESSEE_DISTRIBUTION`; see
[`docs/runtime-distribution.md`](docs/runtime-distribution.md). The actual local
test flow signs/serves a development pack on an owned loopback fixture endpoint,
never a public distribution service. Windows private Job ownership is implemented
but not OS-qualified; macOS capture still fails closed because exact owned-tree
supervision is an explicit code gap. No public-site/runtime/native
acceptance is inferred from these tests.

New unpinned work now starts a genuinely missing browser setup automatically when
a trusted supported distribution is available. Default public setup is still
missing from this build—not something end users should solve by installing
Node/npm. Damaged installations need explicit repair; checks/resumes preserve
their pinned runtime. See the [first-use gap and scope](docs/research/automatic-runtime-setup-gap-20261010.md).

Settings → **Preview diagnostic contents** → review the exact sanitized structured
JSON → **I reviewed this — export locally…** saves only after explicit consent.
Cancelling the preview or save picker saves nothing; no diagnostic is uploaded.
Screenshots, URLs/queries, sessions/references/secrets, names, paths and private
logs are structurally excluded. Tiny-mask outlines now retain exact scaled bounds.

Phase 7 adds authenticated contained bundle links/file modes, native Windows
quota/ACL/reparse handling, checked private source/app/runtime-pack helpers and
target build recipes. Unreviewed/unknown licenses and owner signing remain blocked.
The current Linux development binary is not an Ubuntu 24.04/AppImage qualification.
Earlier maximum-scale attempts failed and remain recorded. After incremental
checkpoint and Linux guardian repairs, independent verification completed
**1,000 pages × three profiles: 3,000 actual captures and visits**, with preserved
pause/reopened resume evidence and a separate large-run cancellation. Original
budgets were unchanged. This is local backend scale evidence, not native UI,
cross-platform, public-browser, or release qualification.

Native launch on this test display failed during Vulkan swapchain creation;
see the [owned-window report](docs/research/native-render-on-disk.md). macOS
supervision and accessibility remain code gaps; Windows integration, Linux
packaging/ABI, current public runtime, real keychain, human usability and signing/
licensing gates remain open. The earlier private candidate is stale for corrected
source and must not be presented as the verified current app.

Use **Create project**, enter a name and complete http(s) address, then **Save
project**. Projects → **Open project** reopens saved settings. Ctrl/Cmd+N, S and
1…5 create/save/switch sections; Tab/Shift-Tab traverse active controls,
Enter/Space activate buttons, Escape closes the information dialog, Ctrl/Cmd+Q
quits. Native menu/titlebar/folder/discard prompts are used where supported.

Windows/macOS developer checks: `cargo build --locked --workspace --features ui
--bins`, `cargo test --locked --workspace --all-features` and `cargo clippy --locked
--workspace --all-targets --all-features -- -D warnings`. Linux needs GPUI native
development libraries; `scripts/build-ui.sh` only creates local linker-name
symlinks on this existing x64 userspace. It does not install host packages.
`.github/workflows/foundation.yml` defines checks for all three OSes; those remote
jobs were **not run** here. CI compilation is not desktop launch qualification.

One optional bounded owned-window probe is available as
`python3 scripts/phase2-native.py --output /absolute/new-report-directory`.
It explicitly uses X11/lavapipe, does not force mapping or use a virtual display,
and stops before input if rendering is unavailable. Its result is not an ordinary
desktop or hardware-GPU pass. Do not repeat black-window loops.

## Separate Phase 0 feasibility bench

Native Rust + GPUI bench, MPL-2.0. **Phase 0 is not accepted yet.** See
[`docs/phases/00-status.md`](docs/phases/00-status.md) for real evidence and gates.
The bench itself does not provide product project management, crawling,
baselines or comparisons. It remains separate from the maintained foundation.

## Reproduce (developer harness)

Run from the repository root; Python is needed only to build/test the development
pack. Rust 1.98.1 is pinned. No system Node/npm is used by these commands.

```sh
cargo test --locked
python3 scripts/bootstrap.py
# On THIS unsupported Ubuntu 26.04 userspace only, explicit experimental probe:
python3 scripts/bootstrap.py --linux-compat ubuntu24.04-x64
cargo run --locked --bin regressee-spike -- pack
cargo run --locked --bin regressee-spike -- repair
cargo run --locked --bin regressee-spike -- smoke --headed
cargo run --locked --bin regressee-spike -- protocol-test
.runtime/installed/node/bin/node --test worker/test/fixtures.test.mjs
.runtime/installed/node/bin/node --test .runtime/installed/worker/test/policy.test.mjs
python3 scripts/qualify.py --headed  # Linux /proc process/RSS evidence
cargo run --locked --bin regressee-spike -- tiles artifacts/tall.png
```

`bootstrap.py` uses exact TLS Node downloads and checksums, private npm `ci`, exact
Playwright and the matching full Chromium, never a global installation or sudo.
Only synthetic loopback fixtures are allowed by this spike worker.

## Native window

```sh
bash scripts/build-ui.sh             # this Linux x64 linker-name workaround
target/debug/regressee-ui            # normal GPU/display attempt
# Developer software-renderer probe only; native mapping/input remains unqualified:
env -u WAYLAND_DISPLAY VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/lvp_icd.json target/debug/regressee-ui
python3 scripts/ui-smoke.py          # captures only its own native X11 window
bash scripts/flatpak-probe.sh        # pinned existing SDK probe; sandbox launch fails here
```

On Windows/macOS, use `cargo build --locked --features ui --bins` and the resulting
`regressee-ui` executable. Those targets are **not runtime-qualified here**.

The latest native harness fails window mapping/input in this shared environment.
The software-renderer command is a probe, not a proven end-user workaround. See
[`docs/phases/00-reverification.md`](docs/phases/00-reverification.md).

With a prebuilt `.runtime/pack.zip`, choose **Repair runtime**, **Open sign in**,
click **Sign in** in the managed fixture browser, then **I’m ready**. Enter
`/private` and choose **Capture address**, or capture `/public` or the tall fixture.
Open PNG uses the native file dialog. Scroll the preview to page through 512-row
tiles. Tab/Shift-Tab traverse controls; Enter activates focused buttons;
Ctrl/Cmd+O opens an image, Escape cancels, Ctrl/Cmd+Q quits. Sessions are ephemeral.
This is a test bench, not a release installer; no download service is configured.

## Artifacts and limitations

Runtime/cache/build artifacts are ignored. Test screenshots and metadata live in
`artifacts/`; they are generated synthetic content, not real sessions. Pack signing
uses a disclosed **development key**, not release authenticity. The historical
browser pin must be replaced/qualified before accessing real websites.
Session secrets use bounded private product IPC, never diagnostic output,
project/evidence JSON or credential export. Saved state files are encrypted.

Core tests cover image limits, row tiling, pack integrity/path safety and failed
repair preservation. Worker smoke covers stability, replacement geometry,
cookie/localStorage/IndexedDB/sessionStorage replay and expiry. Headed/headless
pixel equality is recorded as an investigation, **not assumed or disguised as a
passing equality assertion**. Qualification also tests live-process cleanup,
parent crash, runtime corruption and repair. The UI smoke's claims are explicitly
limited in its JSON result. Full clean-machine, cross-OS, hardware-GPU, credential
store and screen-reader gates remain separate.

Image replacement does not support source-dependent CSS/scripts. Stable pixels
can coexist with `ReplacementGeometryChanged` or `ReplacementGeometryNotMeasured`;
neither is dimension-preserving success. Current replacement pixels/profile use
`neutral-raster-v2`, incompatible with v1 replacement baselines. See
[`00-image-corrections.md`](docs/phases/00-image-corrections.md) for focused tests.
