# Private candidate support matrix

No row below is a public-release support claim. All three desktop OS families
remain required before first release. Build recipes do not establish execution.

| Candidate target | Locally implemented / observed | Remaining code or execution gate |
| --- | --- | --- |
| Development Linux x86-64, glibc 2.43, kernel 7.2.4 | Actual locked GPUI build, owned loopback worker/capture/storage; exact private scratch cleanup | Native rendering/input, secure store, default GPU and installed package still unqualified |
| Ubuntu 24.04 x86-64, glibc ≤2.39 build floor | Minimum-userspace AppImage direction only | Current development ELF requires GLIBC_2.43; a fresh minimum-userspace build and default AppArmor/Chromium sandbox package test are required |
| Windows 11 24H2 x64 (qualification target, serviced build to be frozen) | Atomic private Job Object tree ownership, native quota-space API, protected current-user DACL and reparse rejection implemented; API/std type-check scaffold | Full app/runtime SDK linking, actual standard-user ACL/crash/cancel/nested-browser jobs, installer, DPI/IME/secure store and native execution NOT RUN |
| macOS 14.7 / Apple Silicon, x64 separately if retained | Authenticated contained framework links/exact file modes preserve bundle entries; native signature verification remains owner/platform work | **Code gap: product process-tree ownership/crash recovery is not implemented; capture is blocked.** Native SDK/build, signatures/Gatekeeper/Keychain and desktop tests NOT RUN |

Unclaimed: Windows ARM/10, Linux ARM/musl/Alpine, older macOS, arbitrary Linux
distributions, Rosetta-as-Intel qualification and Flatpak. Do not bypass OS
security policies, use `--no-sandbox`, remove quarantine or silently install system
Node/npm/browser to make a row pass. Host fonts/libraries/drivers are prerequisites
until an actual package supplies and qualifies them.

The sleek GPUI working-desk visual system is a proposal, not human-approved.
Screen-reader semantics/backend coverage must be qualified independently, not
inferred from textual labels or simulated keyboard actions.

The 2026-10-10 UI sweep has independently passing focus/layout/action and
persistence regressions. Its single isolated X11/llvmpipe native attempt mapped
but captured entirely black pixels, stopping before input. The corrected binary
was not natively relaunched. User-reported plain launch success remains separate;
no default-GPU, five-screen visual/input or human acceptance follows from either
the source tests or that test display. See `research/ui-sweep-reverification-20261010.md`.

Latest independent local maximum-scale check on the development reference host:
**1000 pages × three profiles, 3000 actual visits/persisted captures PASS** after
guardian/checkpoint corrections. It preserved 1000 paused originals across reopened
resume, activated the complete baseline, then captured 1000/cancelled 2000 in a
separate run. Total1114.862 s; original budgets unchanged. See
`docs/phases/07-guardian-verification.md` for scope and resource caveats. Earlier
1708/2188-target failed attempts remain retained, not reclassified. Native
interaction latency, hardware rendering and cross-platform scale remain unqualified.
