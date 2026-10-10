# Release packaging and qualification research

**Research date: 2026-10-09. Status: recommendations only; Phase 7 NOT RUN.**

This independent, read-only investigation owns only this document. No builds,
installations, runtime changes, fixtures, native windows, browser processes, CI
dispatches, publication, signing operations, credential access or agent spawning
were performed.
Primary-source HTTP GETs retrieved documentation/metadata; HTTP HEADs checked
published artifact availability without downloading binaries. Availability is
not integrity verification, native execution, security approval or redistribution
clearance. Live metadata must be checked again when selecting release pins.

## 1. Repository baseline and immediate conclusions

Read `PRODUCT.md`, `PLAN.md` (particularly §§8, 11–14),
`docs/IMPLEMENTATION_STATUS.md`, decisions 0001–0003/0005 and controlled-parallel
0006, Phase 7 checklist, native/foundation verification reports, Cargo/toolchain
and worker manifests/locks, notices, both CI workflows, and bootstrap, packaging,
qualification and native-probe sources. Another implementation agent may advance
the product while this report is written: this is a snapshot, not its status log.

- Windows, macOS and Linux are required at first public release. Exact OS/CPU
  support remains unqualified. Phase 7 follows Phase 6; neither this research nor
  a future compile-only matrix passes that gate.
- The inspected pins are Rust **1.98.1**, GPUI **0.2.2**, Node **22.20.0**,
  Playwright/core **1.56.1**, Chromium **141.0.7390.37 / revision 1194** and
  FFmpeg **1011**. Decision 0001 correctly labels the browser/runtime historical
  feasibility pins, not a 2026 public-site security-qualified stack.
- `scripts/bootstrap.py` is developer tooling, not first-run product delivery.
  It supports Linux x64, macOS arm64/x64 and Windows AMD64. It has committed Node
  hashes and an npm integrity lock but downloads browsers through Playwright.
  The Fedora-like/shared development environment and its explicit Ubuntu override
  are not clean supported-Linux evidence.
- `worker/src/capture.mjs` explicitly sets `chromiumSandbox:true` and uses
  `channel:'chromium'` for full-browser new-headless/headed consistency. Preserve
  that explicit isolation requirement in the product/candidate launch path;
  a bare Playwright launch or successful PNG is not a sandbox qualification.
- `crates/spike/src/runtime.rs` uses a public development signing key. Its
  checks demonstrate integrity mechanics only. Never ship that trust key or the
  fixture worker/fault commands in product packs. The maintained runtime
  supervisor inspected here is an interface, not finished cross-OS supervision.
- Decision 0003 provisionally selects AppImage/Ubuntu 24.04 x64. No inspected
  artifact or default-policy clean-host result makes that supported today.
  Direct Chromium launch in the pinned Flatpak SDK failed with a sandbox error;
  neither retrying that SDK nor loosening permissions is a release solution.
- **Priority order:** select and audit a maintained runtime; prove Linux browser
  sandbox under default host policy; preserve macOS bundle structure/signatures;
  qualify actual desktop input/rendering; then package/sign and run clean-machine
  acceptance. Do not defer the first real Windows/macOS launch until Phase 7.

## 2. Published runtime metadata, checked rather than recalled

All links in this report were consulted on **2026-10-09**, unless explicitly
identified as a historical source. These are upstream facts, not changed pins.

| Source | Observed published value | Consequence |
| --- | --- | --- |
| [Node release index](https://nodejs.org/dist/index.json) | Current v26.11.1, dated 2026-10-07, `lts:false`; newest indexed v24 is **24.21.0**, 2026-09-07, `lts:Krypton`; newest indexed v22 is **22.23.3**, 2026-09-23 | Prefer evaluating a maintained LTS rather than selecting the numerically largest major automatically. 22.20.0 is not the latest maintained 22 patch. |
| [Node release schedule](https://raw.githubusercontent.com/nodejs/Release/main/schedule.json) | v22 ends 2027-04-30; v24 maintenance begins 2026-10-20 and ends 2028-04-30; v26 LTS is **scheduled** for 2026-10-28 | v26 is not LTS as of this research. Schedule is not proof a future release has shipped. Node 20 has already reached scheduled EOL. |
| [npm latest metadata](https://registry.npmjs.org/playwright/latest), [registry publish times](https://registry.npmjs.org/playwright) | **Playwright 1.64.0**, published 2026-10-07T20:42:55.791Z, core exactly 1.64.0, package engine `node >=20`; `next` is a 1.65 alpha | A package engine range is not the upstream support policy; do not adopt a canary or Node 20 solely because it satisfies this range. |
| [Tagged browser metadata](https://raw.githubusercontent.com/microsoft/playwright/v1.64.0/packages/playwright-core/browsers.json) | `chromium` **156.0.8078.4**, revision **1248**, title **Chrome for Testing**; same headless-shell revision; FFmpeg **1013**, Windows dependency tool **1007** | Browser identity/layout and licensing inventory have changed since the spike. Upgrade the entire tested tuple, not only Node or an arbitrary Chrome executable. |
| [Tagged distribution registry](https://raw.githubusercontent.com/microsoft/playwright/v1.64.0/packages/playwright-core/src/server/registry/index.ts) | Full browser paths for Linux x64/arm64, Mac x64/arm64, Windows x64; CfT URLs use the browser version | Do not continue assuming old `chrome-linux` / `chrome-win` / `Chromium.app` directory layouts. Windows has no separate native arm64 Chromium download in this registry. |

The contemporary [Playwright requirements](https://playwright.dev/docs/intro#system-requirements)
list latest Node **22.x/24.x/26.x**, Windows **11+ / Server 2019+ / WSL**, macOS
**14+**, and Debian **12/13**, Ubuntu **22.04/24.04/26.04**, Linux **x86-64 or
arm64**. This does **not** retroactively make Playwright 1.56.1 support Ubuntu
26.04, nor make WSL a native Windows GPUI test, nor promise arbitrary Fedora or
Alpine compatibility. GPUI, OS vendor servicing and packaged dependencies impose
additional limits.

For candidate Node 24.21.0, [tagged BUILDING.md](https://raw.githubusercontent.com/nodejs/node/v24.21.0/BUILDING.md)
lists macOS x64/arm64 minimum 13.5, GNU/Linux x64/arm64 kernel ≥4.18 and glibc
≥2.28, and Windows x64 minimum Windows 10/Server 2016. Official GNU/Linux builds
require libstdc++ ≥6.0.25 (`GLIBCXX_3.4.25`). The stricter Playwright/macOS 14 and
Windows 11 floor wins for this product. Even a newly listed Node musl archive
does not establish Playwright/GPUI Alpine support.

### Availability actually checked, and its limits

- Node 24.21.0 Linux x64 and both Darwin `.tar.xz` archives, Windows x64 `.zip`,
  and detached `SHASUMS256.txt.sig` each returned **HTTP 200 to HEAD**.
  [Published hashes](https://nodejs.org/dist/v24.21.0/SHASUMS256.txt) were read;
  archives/signatures were not downloaded or cryptographically verified.
- Playwright 1.64.0 npm metadata publishes integrity and provenance/attestation
  references. Neither package bytes nor those attestations were verified here.
- Google's [known-good metadata](https://googlechromelabs.github.io/chrome-for-testing/known-good-versions-with-downloads.json)
  includes 156.0.8078.4; metadata timestamp was 2026-10-09T10:23:06.679Z.
  HEADs to the exact Playwright CDN paths below all returned **200**, redirecting
  to `storage.googleapis.com/chrome-for-testing-public`:

  `https://cdn.playwright.dev/builds/cft/156.0.8078.4/` plus:

  | Suffix | Published compressed bytes from HEAD |
  | --- | ---: |
  | `linux64/chrome-linux64.zip` | 200,345,244 |
  | `linux-arm64/chrome-linux-arm64.zip` | 199,750,508 |
  | `mac-x64/chrome-mac-x64.zip` | 205,332,100 |
  | `mac-arm64/chrome-mac-arm64.zip` | 194,831,853 |
  | `win64/chrome-win64.zip` | 207,073,013 |

  These are browser archive sizes, **not** total runtime disk/download estimates.
  FFmpeg/winldd and a complete candidate runtime pack were not availability-tested.
- Google's [channel metadata](https://googlechromelabs.github.io/chrome-for-testing/last-known-good-versions-with-downloads.json)
  listed Stable **155.0.8059.39** and Beta **156.0.8078.12**. Thus Playwright's
  pinned 156 build is not the latest Stable channel. Version matching is required;
  review security/backport status rather than equating “latest Playwright” with
  “latest Stable Chrome” or a completed vulnerability audit.
- The [CfT upstream README](https://raw.githubusercontent.com/GoogleChromeLabs/chrome-for-testing/main/README.md)
  now lists Linux arm64 availability from 153.0.8001.0, but no Windows arm64 CfT
  artifact. Neither a native Node arm64 archive nor x64 browser emulation licenses
  a native Windows ARM product-support claim.
- The pinned [Rust 1.98.1 distribution manifest](https://static.rust-lang.org/dist/channel-rust-1.98.1.toml)
  returned **200 to HEAD**. This confirms endpoint availability, not target
  installation or any current workflow result.

**Bounded recommendation:** evaluate Node **24.21.0 + Playwright/core 1.64.0 +
its exact full browser**, with an explicit legal/security review and fresh fixture,
headed-session and native-package tests. This is not an instruction to change
pins now, nor approval for real-site use. Keep the old spike reproducible and
labelled historical. Recheck patch availability before release; never substitute
an alpha or a guessed download URL if a required stable tuple is unavailable.

## 3. Candidate support matrix: release claims need row-specific evidence

Publish exact tested OS releases/builds, architecture, package hash, GPU/driver,
desktop/session, app/runtime tuple and qualification date. Do not advertise an
open-ended “14+” or “all Linux” claim from testing one version.

| Candidate row | Package direction | Evidence still required |
| --- | --- | --- |
| Windows 11 x64, on a vendor-serviced build chosen at release | Signed per-user installer and x64 MSVC executable; avoid a machine-wide runtime installer | Standard-user install/upgrade/uninstall, Defender/SmartScreen behavior, DPI/IME/dialogs, Chromium sandbox, Credential Manager/DPAPI integration, detached-child cleanup, no Node on PATH |
| macOS 14 minimum, Apple Silicon | Native arm64 `.app` in signed/notarized/stapled DMG | Oldest supported and current serviced macOS, Metal/default desktop, Gatekeeper quarantine, first-download runtime launch, Keychain restart/lock behavior |
| macOS 14 minimum, Intel **if retained after stack qualification** | Separate x64 `.app`/DMG and runtime first; universal app is optional | Actual Intel execution, including minimum OS; Apple Silicon/Rosetta is not Intel hardware evidence. If infeasible, decide the architecture scope explicitly before publication. |
| Ubuntu 24.04 x64 | Provisional audited AppImage plus signed private runtime | Default AppArmor/userns policy, FUSE or reviewed extract-and-run UX, default Wayland and X11 where claimed, Mesa and selected proprietary-driver coverage, host fonts, Secret Service availability |
| Actual Fedora desktop, exact release to be selected | Same Linux package, investigation row only | Native Fedora userspace/kernel, SELinux enforcing, drivers, sandbox, dialogs and secure store. Shared Fedora-named kernel alone is not this evidence. |

Upstream has runtime downloads for more tuples, but the current bootstrap has no
Linux arm64/Windows arm64 target. Keep Linux ARM, Windows ARM, Windows 10,
musl/Alpine, extra distros and older macOS **unclaimed** until separate gates pass.
All three OS families still remain mandatory; a single Mac architecture can
satisfy the family requirement only after an explicit support-scope decision.

## 4. Practical packaging paths without disabling browser isolation

### Windows

Start with a modest per-user installer into a stable app location, shortcut,
version metadata and clear uninstall behavior. Keep data/evidence and versioned
runtime outside installation files; uninstall must not silently delete history.
MSI/WiX or another audited native installer can be evaluated; do not add Store/
MSIX constraints before proving the private downloaded executables work under
that deployment model. A ZIP can support diagnostic testing but is not by itself
the non-developer installation experience.

Native linking/import inspection must identify the actual CRT/DLL requirements;
CI's Visual Studio and runtime presence can conceal missing end-user dependencies.
Preserve and verify valid upstream Authenticode signatures on unchanged Node/
browser payloads; sign Regressee executables and installer with its own identity.
Signing does not guarantee SmartScreen reputation. Do not recommend bypassing
security prompts as the normal setup path.

Use Windows [Job Objects](https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects)
as a supervision design candidate: attach the worker early, consider
`JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE`, prevent unnoticed breakaway, and test
Chromium's own nested jobs. A Unix process-group pass does not establish Windows
parent-death cleanup, file-lock handling or private directory ACLs.

### macOS — bundle preservation is a concrete early risk

Prefer separate native arm64/x64 artifacts initially. A universal Rust binary
does not make downloaded Node/browser helpers universal; runtime selection must
be explicit. Set the deployment target consistently and inspect every Mach-O
dependency/minimum OS, not just the Rust executable.

`crates/spike/src/runtime.rs::walk` **silently skips every symlink**; extraction
rejects links and normalizes executable permissions. Node's npm links may be
dispensable, but browser `.app`/framework symlinks can be essential and Apple code
signatures seal them. This is a **static packaging risk, not a reproduced Mac
failure**. Do not simply weaken all link validation. Evaluate a macOS-specific,
authenticated bundle transport: preserve a verified inner archive/bundle,
validate allowed relative in-bundle link targets, reject escapes/cycles, and
verify its signatures after extraction. Prove the exact candidate layout first.

Apple's [code-signing guidance, TN2206](https://developer.apple.com/library/archive/technotes/tn2206/_index.html)
requires standard nested-code locations, inside-out signing and read-only sealed
bundles; `--deep` is useful for **verification**, not a general signing recipe.
The signed app must not write runtimes/settings into its own bundle. Its code
signature does not automatically cover later downloaded runtime code.

For public distribution outside the Store, use Developer ID, secure timestamps,
Hardened Runtime and notarization. Audit Node's JIT and browser-helper entitlements
separately; do not blanket-disable library validation or invent entitlement needs.
Apple's [notarization requirements](https://developer.apple.com/documentation/security/notarizing-macos-software-before-distribution.md)
and [custom workflow](https://developer.apple.com/documentation/security/customizing-the-notarization-workflow.md)
explicitly cover network installers: **separately notarize the installer and what
it downloads**. A ZIP can be submitted but cannot itself receive a stapled ticket;
staple supported bundles/DMGs and repackage. Standalone binaries cannot currently
be stapled, so the runtime container choice affects offline first launch.

Do not copy CfT's suggested recursive extended-attribute removal into Regressee's
setup. That bypasses the qualification problem. Test a genuinely quarantined
installer and downloaded runtime with Gatekeeper enabled, including offline launch
after successful setup. Certificates, notarization submission and tickets are
owner-controlled external work, **not performed/authorized by this research**.

### Linux AppImage — useful distribution, not a sandbox solution

[AppImage best practices](https://docs.appimage.org/reference/best-practices.html)
require an old-enough build base, supplied non-base libraries, relocatable paths
and tests on each claimed base. Its historical CentOS example is not a 2026
recommendation. For the current narrow Ubuntu 24.04 target, build/audit against
that minimum userspace; inspect GLIBC/GLIBCXX requirements of **every** payload.
Do not bundle random host Vulkan ICDs/proprietary GPU drivers or replace the host
glibc as an improvised portability fix. Scope bundled libraries so they do not
accidentally pollute the managed browser's loader environment.

Use the candidate's [tagged native dependency list](https://raw.githubusercontent.com/microsoft/playwright/v1.64.0/packages/playwright-core/src/server/registry/nativeDeps.ts)
and the actual archive dependency metadata to explain missing libraries. Ubuntu
24.04 has t64 package names that differ from 22.04. These lists are diagnostic
inputs, not permission to run Playwright's privileged dependency installer.

An AppImage is not automatically an application sandbox. Chromium still needs
working host-supported renderer isolation. Ubuntu 24.04's [default namespace
restrictions](https://canonical.com/blog/whats-new-in-security-for-ubuntu-24-04-lts)
and current [AppArmor guidance](https://documentation.ubuntu.com/security/security-features/privilege-restriction/apparmor/)
mean a browser unpacked into a user's private path need not match Google's system
Chrome profile. **This is an early clean-machine blocker to investigate**, not
something bundling more `.so` files fixes. Launch from the actual chosen install
path as a normal user under default policy; record sandbox diagnostics and owned
renderer process restrictions, not just a successful screenshot.

If default policy blocks it, stop with an understandable diagnostic. A narrow
administrator-reviewed application profile/managed installation is a possible
future packaging decision but needs separate approval and UX/security review;
no privileged operations are authorized here. Do not turn off AppArmor/userns
restrictions, ship a setuid workaround, borrow another app's permissive profile,
or disable Chromium's sandbox. If a no-admin path cannot be proved, AppImage is
still conditional and must not be advertised as zero-prerequisite.

The [AppImage FUSE guide](https://docs.appimage.org/user-guide/troubleshooting/fuse.html)
notes Ubuntu 24.04's `libfuse2t64` naming and type-2 extract-and-run fallback.
Choose a pinned, audited AppImage runtime and determine its actual FUSE needs;
FUSE2 instructions are not universal to every new runtime. Extract-and-run avoids
a mount prerequisite, **not** Chromium's isolation policy. Budget extra disk and
authenticate before extraction. Terminal fallback instructions alone do not meet
the product's non-developer goal; provide a reviewed launch/install experience or
declare the prerequisite accurately. No FUSE packages were installed here.

### Flatpak — keep the failed direct-bundle result, bound alternatives

The pinned SDK probe is not a finished `.flatpak` test. Nevertheless, the observed
sandbox failure makes a simple “put the same binaries in a Flatpak” recommendation
unjustified. [Flatpak permissions](https://docs.flatpak.org/en/latest/sandbox-permissions.html)
limit syscalls/namespaces, filesystem and D-Bus. An outer app sandbox with network,
project data and credentials is not interchangeable with renderer isolation.

Two bounded research paths, neither accepted or exercised here:

1. A package-specific Chromium launcher using a separately reviewed sandbox
   integration such as [Zypak](https://github.com/refi64/zypak). Its upstream design
   redirects Chromium sandbox launches to Flatpak child sandboxes; it is **not**
   merely enabling nested namespaces. Prove Playwright pipe/launch compatibility,
   exact full-browser pin, renderer privileges, session privacy, GPU and scoped
   cleanup. Upstream says a fully installed package is required; another SDK-shell
   probe is insufficient. A change from Chromium's native sandbox needs an
   explicit security/architecture decision; no disabled-sandbox mode is acceptable.
2. A separately packaged, pinned local browser/worker broker with its own verified
   sandbox and private, authenticated IPC. This is more architecture than packaging
   and should not be undertaken unless AppImage/default-policy evidence warrants
   it. A portal that opens the user's default browser does **not** provide the
   required deterministic managed Playwright runtime or session replay.

If revived, select a maintained Freedesktop runtime **and exact commit** at release,
not the development SDK automatically. Start with Wayland/fallback-X11 and DRI
permissions as needed, narrowly scoped networking/IPC and per-app storage. Use
file portals for ZIP import/export and narrow, tested Secret Service access.
Do not grant host filesystem, unrestricted session bus or host-spawn access to
make tests pass. The existing 25.08 commit is historical probe evidence, not a
selected or qualified release runtime. No Flatpak publication is proposed now.

## 5. The native black-window/mapping issue: isolate evidence, not verdicts

`docs/phases/00-native-recheck-2026-10-09.md` records a viewable owned X11/lavapipe
window whose valid PNG contained **896,000 black pixels**, plus no proven keyboard
enqueue; a second probe saw an initial unmapped state without a mapping grace
period. `docs/phases/01-verification.md` records an owned foundation window still
unmapped after its bounded attempt. These are distinct observations. They do not
prove a persistent universal mapping failure, identify a root cause, or qualify
the current product shell. Backend captures and an old bench screenshot are not
substitutes. Default Wayland/hardware GPU is still untested in these reports.

Suggested later bounded investigation, only with owned resources and explicit
execution approval:

- **Isolated X11 diagnostic:** if already provisioned in a dedicated environment,
  use Xvfb with a private display/auth cookie, no TCP listener, a small owned
  EWMH-capable window manager, and the explicitly recorded Mesa lavapipe ICD.
  [Xvfb](https://www.x.org/releases/current/doc/man/man1/Xvfb.1.xhtml) is a virtual
  framebuffer, **not** a guarantee of the Vulkan WSI/presentation GPUI needs.
  Check extensions/renderer/present support first; stop after one meaningful
  attempt, do not endlessly retry. Never use `-ac`/global access-control changes.
- **Isolated Wayland diagnostic:** a pre-provisioned headless/nested compositor
  with its own mode-0700 `XDG_RUNTIME_DIR` and unique socket can isolate compositor
  state. Its renderer must support this GPUI/Vulkan path. This is an alternative
  test environment, not a promise it resolves black pixels or proves hardware
  rendering. Do not connect its automation to the shared desktop.
- **Owned desktop qualification:** dedicated test account/session or disposable
  machine, launch packaged app through the normal OS entry point under default
  graphics. On X11 identify title plus `_NET_WM_PID`; allow bounded normal mapping
  time, confirm viewability and focus (including legitimate owned child windows),
  then send input. On Wayland use approved compositor/test interfaces or manual
  operation; XTest is not a generic Wayland controller.
- Capture owned content, verify nonblank pixels **and actual expected UI**, then
  assert fresh observable actions (a saved project, successful owned job, native
  quit). Merely counting colors is a smoke prerequisite, not a semantic UI pass.
  Never force-map a window and label it ordinary launch. Reap only exact owned
  PIDs/process trees and retain fresh logs/screenshots without overwriting history.

A virtual-display pass would narrow the shared-display hypothesis, not absolve
the code or qualify default GPU/input/IME/accessibility. Failure would still need
triage among driver, Vulkan surface/present, compositor, timing and GPUI/app paths.
Real OS/default-desktop evidence and human review remain separate gates.

## 6. CI evidence versus release qualification

Current workflows already say “not runtime/desktop qualification.” Preserve that
honesty. [GitHub runner documentation](https://docs.github.com/en/actions/reference/runners/github-hosted-runners)
currently maps `windows-2025` to Windows Server x64, `macos-14` to Apple Silicon,
and explicit `macos-15-intel` / `macos-26-intel` to Intel. Thus the inspected
three-label matrix does **not** cover Windows 11 client or both Mac architectures.
Hosted images contain developer tools; Windows hosted runners run as administrator
with UAC disabled. Fresh CI VM does not mean clean standard-user end-user machine.

Later stages should be separate, with explicit **PASS / FAIL / NOT RUN / BLOCKED**:

| Stage | Automatable evidence | What it cannot establish |
| --- | --- | --- |
| Locked checks/native linking | Actual target unit/protocol tests, UI-inclusive lint/link, dependency imports, package structure | Native window rendering/input, supported minimum OS or installer UX |
| Owned fixture/runtime smoke | Exact Node/browser versions, sandbox-enabled launch, PNG semantics, headed/headless consistency, cancel/repair/cleanup | Enterprise SSO, actual physical display/driver, credential-store interaction |
| Isolated virtual desktop | Mapping, focused owned input, fresh persisted actions and pixels on that virtual backend | Default hardware graphics, screen reader/native interaction on every target |
| Clean OS install acceptance | Final installer hash, standard-user/no developer tools, first download and fault/restart paths | Full hardware coverage and representative non-developer comprehension |
| Reference-hardware/human gate | Default graphics/IME/DPI/dialogs/keychain, screen-reader checks, usability and declared soak performance | Other untested OS/CPU/GPU tuples |
| Trusted release/signing | Owner-reviewed source/artifact hashes, signing identity, notarization logs/tickets and distribution notices | Product behavior; a signature is not a functional test |

Select explicit build rows for Windows x64, Mac arm64 **and Intel if claimed**,
Ubuntu x64. Add minimum-OS self-hosted/dedicated test rows when hosted labels do
not provide them. Pin action revisions and record image version, SDK/linker,
architecture and GPU. Pre-provision build prerequisites in owner-managed images;
missing libraries remain BLOCKED/FAIL, not permission to install globally or
skip native linking. Use unique checkout/output/cache/runtime/fixture-port/display
roots per job. Existing spike scripts hardcode shared `.runtime`/`artifacts` and
loopback resources: do not run them concurrently unchanged.

Command ideas below are **later templates, not commands executed here**. They
assume tools and prerequisites already exist and an isolated checkout/output.
No new product smoke CLI is invented:

```sh
cargo fmt --all -- --check
cargo test --locked --workspace
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo build --release --locked -p regressee-app --features ui --bin regressee
cargo tree --locked --target <declared-target-triple> --edges normal
```

Target import/symbol inspection ideas (substitute the actual packaged paths):

```sh
# Linux: repeat for app, Node, browser and bundled ELF libraries.
readelf -d <packaged-elf>
readelf --version-info <packaged-elf>
# macOS: architecture, deployment floor and absolute/dependency paths.
lipo -info <packaged-mach-o>
otool -L <packaged-mach-o>
otool -l <packaged-mach-o>
# Windows SDK tools on the Windows build/test machine.
dumpbin /dependents <packaged-exe>
```

Use static inspection for untrusted payloads rather than executing `ldd` on them.
For a **reviewed private** developer build candidate, its private Node may later
invoke private npm `ci --ignore-scripts --no-audit --no-fund` and the matching
Playwright CLI `install --no-shell chromium`, with private npm/browser paths and
upstream integrity verification. This is a trusted pack-production step, never
an end-user/npm setup flow. No `install-deps`, `--with-deps`, system Chrome/Edge
installation or host-platform override belongs in a purported supported-host pass.

Verification templates for already signed artifacts:

```sh
# On macOS; preserve nested upstream signatures and verify each runtime bundle.
codesign --verify --deep --strict --verbose=2 Regressee.app
spctl --assess --type execute --verbose=2 Regressee.app
xcrun stapler validate Regressee.dmg
# On Windows; check app, installer and applicable upstream executable signatures.
signtool verify /pa /all /v Regressee.exe
signtool verify /pa /all /v RegresseeInstaller.exe
```

Notarization/timestamp verification can contact vendor infrastructure: defer it
to owner-approved release work. Neither local ad-hoc signing nor trusting a new
test certificate globally substitutes for production/Gatekeeper evidence.

Every qualification record should bind **app/installer/runtime hashes + source
snapshot + host fingerprint + exact command/action + logs + pixels + actual
assertions + cleanup audit**. OS-specific snapshots compare within that environment;
cross-OS portable ZIP review needs the existing rendering-caution policy, not a
pixel-identical portability promise. Soak results must account for all intended
targets and report per-component memory/disk/latency on declared reference hardware.

## 7. App/runtime authenticity and update hardening

These are bounded follow-on design/test tasks, not a new hosted service mandate:

1. **Separate trust layers.** Verify upstream inputs (Node's signed checksum list
   against independently reviewed release keys, npm lock integrity/provenance where
   available, browser exact archive digest/provenance and platform signatures).
   Hash final OS-signed/stapled bytes and create the Regressee runtime manifest
   afterward. A Regressee signature on arbitrary TLS-downloaded bytes is not
   upstream provenance. HEAD 200 and a locally calculated hash alone are not trust.
2. **Release-only verifier.** Ship only approved public verification keys; reject
   development-signed packs in release mode. Private signing material must never
   appear in source, npm packages, ordinary CI caches/logs or artifacts. Key
   custody, certificate identity, signing authorization and rotation/revocation
   are decisions for the user/release owner, not this agent.
3. **Versioned manifest.** Bind schema/app compatibility, OS/CPU, protocol/worker,
   exact Node/Playwright/browser, pack byte size/hash, decompression bounds,
   inventory/permissions, licenses/source inventory and key ID. Authenticate exact
   bytes before parsing/extracting, enforce redirect/TLS policy and fail closed.
   Specify expiry/rollback semantics and clock-error behavior rather than relying
   on TLS or allowing any once-signed old security-vulnerable runtime forever.
4. **Crash-safe activation.** Stream bounded downloads into private staging,
   reject path/case collisions, Windows reserved names/ADS and hostile links,
   test disk exhaustion/cancel/interruption at every activation boundary, and
   validate ACLs/permissions and sandboxed smoke before READY. The spike's rename
   rollback is not proven crash-durable across OSes. Preserve old ready runtime
   on setup failure and never touch evidence during repair.
5. **Runtime security policy.** Review available patches at candidate freeze and
   before publication; document a review cadence and user-facing update/support
   window. Security-invalidated runtimes may need blocking for further navigation
   while historical screenshot review stays available. Material browser changes
   require explicit compatible baseline migration, not silent replacement or
   indefinite insecure capture to maintain old pixels. Test recovery/rollback
   without rolling security policy or evidence backwards.

Windows [SignTool](https://learn.microsoft.com/en-us/windows/win32/seccrypto/signtool)
requires explicit digest algorithms; SHA-256 and an approved RFC 3161 timestamp
are sensible release choices. Platform signing, detached runtime signing and
notarization have different purposes and keys. Separate untrusted PR/build jobs
from protected, owner-approved signing jobs; no key/service setup is authorized.

## 8. MPL and third-party source/notice obligations

This is an engineering compliance checklist, **not legal advice or a completed
license audit**. `THIRD_PARTY_NOTICES.md` correctly says the resolved graph and
binary packages are not yet audited. A private worker package does not make its
eventual shipped bytes exempt from license obligations.

- **MPL-2.0:** [official text §§3.1–3.4](https://www.mozilla.org/en-US/MPL/2.0/)
  requires distributed Covered Software/modifications to be available in Source
  Code Form under MPL, recipients of executables to be told how to obtain that
  source by reasonable timely means, and notices to be preserved. Provide an
  exact matching source snapshot/archive and build/packaging instructions with
  locks; include `LICENSE` and an accessible About/licenses/source notice in the
  app and runtime release materials. An inaccessible future repository link is
  not source availability. This working directory is not a Git repository, so
  first establish a reviewed source identity; do not invent a release commit.
- MPL is file-level copyleft, not a requirement to relicense unrelated separable
  dependencies. Preserve `crates/app/src/ui/text_input.rs`'s Apache adaptation
  attribution and modified-file notice; do not label it MPL merely because the
  workspace is MPL. Check the source boundary before adding copied code.
- **Apache-2.0:** [§4](https://www.apache.org/licenses/LICENSE-2.0) requires the
  license copy, retained relevant source notices, prominent modification notices
  and readable upstream NOTICE attributions where supplied. Carry GPUI,
  Playwright/core and actual shipped dependency texts, not just SPDX labels.
- **Node:** retain the exact release's full [LICENSE](https://raw.githubusercontent.com/nodejs/node/v24.21.0/LICENSE),
  including embedded V8/OpenSSL/ICU and other component notices. “Node is MIT”
  alone misses that inventory. Avoid shipping developer npm/tests/docs unnecessarily,
  but removing them needs a reviewed shipping allowlist and does not remove the
  binary's embedded dependencies or their notices.
- **Browser:** Chromium's [source license](https://chromium.googlesource.com/chromium/src/+/main/LICENSE)
  has BSD binary attribution conditions, but that is not an audit of the entire
  browser archive. Preserve version-specific bundled third-party notices and
  inspect `chrome://credits`/distribution license files and component source terms
  in an owned test later. **Playwright 1.64's title is Chrome for Testing:** do not
  mechanically carry the spike's “Chromium = BSD” description forward or confuse
  the CfT dashboard repository's license with the downloaded binary's terms.
  Redistribution rights, branding, source obligations and signing modifications
  for that exact payload require review before mirroring it in a Regressee pack.
- **FFmpeg and other copyleft payloads:** [FFmpeg legal guidance](https://ffmpeg.org/legal.html)
  explains LGPL 2.1+ and optional GPL/nonfree build consequences. Exact binary
  build configuration and corresponding sources matter; “retain supplied notices”
  alone is insufficient where source/relinking rights apply. Audit both any
  standalone Playwright FFmpeg utility and FFmpeg-derived browser libraries, plus
  LGPL libraries bundled into AppImage. If the utility is unused for still-PNG
  capture, evaluate excluding it from a reviewed pack rather than shipping an
  unaudited binary; this does not remove browser-embedded obligations. Do not
  equate a revision number with a complete corresponding-source archive.

Generate a **target-and-artifact-specific** shipping inventory: resolved normal
Rust dependencies, native/static libraries and OS redistributables, Node internals,
npm runtime dependencies, browser/helpers/codecs, bundled Linux libraries/fonts,
and any installer/AppImage/Flatpak tools actually shipped. Build-only dependencies
and unselected target packages are not automatically runtime contents. Cargo/npm
metadata alone cannot inventory the browser or copied native libraries. Approved,
pinned local license/SBOM tools can assist later; no installation is done here.

Block release on unexplained/unknown license entries. Retain texts and required
matching sources/configuration/patches in independently retrievable, versioned
release material. Validate source retrieval and artifact notice presence before
publication; restrict reports to synthetic fixtures and exclude cookies, login
profiles, signing keys and private evidence. Hosting, product name and any
third-party redistribution uncertainty require user review, not autonomous upload.

## 9. Bounded Phase 7 handoff

| Next deliverable | Can be automated in an isolated local environment later | Needs external/native/owner evidence |
| --- | --- | --- |
| Maintained runtime candidate inventory and provenance record | Metadata/archive digest checks, package layout, lock/inventory/license extraction and negative manifest cases | Legal/security approval; all platform payloads; trusted release keys |
| Linux packaging feasibility verdict | Static ELF audit; already provisioned virtual-display diagnostics; owned fixture sandbox evidence | Clean default-policy Ubuntu and actual Fedora sessions; hardware drivers; any administrator-reviewed policy/installer decision |
| macOS runtime bundle round-trip proof | Archive/link/path tests and final inventory in fixtures | Native Mac framework/signature/entitlement/Gatekeeper validation on each claimed architecture; Developer ID/notarization custody |
| Final installer acceptance dossier | Standard-user clean-VM setup/failure/upgrade automation, private path/process accounting | Real default OS desktop, IME/DPI/accessibility/secure stores and hardware reference runs; no-terminal human tasks |
| Trusted candidate release set | Source snapshot, hashes, notice/source completeness and reproducible unsigned payload comparisons | Approval of signing/hosting/name, protected keys, authorized vendor submissions/publication |

Limit the next Linux rendering investigation to one owned isolated diagnostic
and one owner-approved ordinary desktop attempt per chosen backend, preserving
fresh evidence; report blockers rather than retrying indefinitely. Build a
candidate package for each committed OS family before investing in an update
service or additional architectures. Compare unsigned payload reproducibility
separately from timestamped OS signatures/notary tickets.

Release acceptance must still include all confirmed features, native/security
gates, three representative non-developer usability participants (including an
authenticated task with a non-implementer), the declared 1,000-page/3,000-target
soak and no critical data-loss/session-export/cleanup defects. Nothing in this
document marks those tests passed, solves the native failure, authorizes services
or publication, or completes Phase 7.
