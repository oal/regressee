# 0003 — Linux packaging decision remains conditional

Qualification target: Ubuntu 24.04 x64; Fedora host testing is additionally required.
The available session has Ubuntu 26.04.1 userspace, Fedora-named 7.2.4 kernel and
shared desktop/device sockets. **This is not clean Ubuntu 24.04 or a native Fedora
run.** Playwright 1.56.1 refuses `ubuntu26.04-x64`; the developer builder's explicit
`--linux-compat ubuntu24.04-x64` requests the supported build for investigation,
never claims the host is supported. Chromium sandbox stays enabled.

| Consideration | AppImage | Flatpak/runtime |
|---|---|---|
| User-space libraries | must bundle/audit manually; does not solve arbitrary glibc | pinned runtime provides a predictable user-space base |
| Browser sandbox | host user namespaces and policies still matter | nested namespaces/seccomp must be qualified; may need a portal/browser architecture decision |
| Files/dialogs | normal host access; AppImage mount/FUSE may need prerequisites | portals and scoped app-data permissions; export/import must use portals |
| GPU | host Mesa/NVIDIA matching remains | extension/runtime/host driver matching remains |
| Secret store | host Secret Service bus availability | explicit Secret Service portal/bus policy qualification |
| Fedora | no distro support implied by single-file delivery | promising library portability, not proven nested browser support |

**Actual probe:** the existing Freedesktop SDK 25.08 commit
`0cc82216a407cc993941b5ddabd446becc3c9a6219d9bcf50125c60101dcad46`
was run with only a read-only project bind and private temporary files. The exact
Node/Chromium library launch **failed with “Chromium sandboxing failed”** inside
that sandbox. `scripts/flatpak-probe.sh` reproduces the pinned SDK probe and refuses
a different SDK commit. No sandbox override, privileged change, or runtime install
was attempted. This is a direct-bundle architecture probe, not packaged-app or
all-Flatpak-design evidence; GPU/dialog/secret-service access remain untested.

**Decision after evidence:** take **AppImage on Ubuntu 24.04 x64** as the next
qualification path, supplying audited user-space libraries plus the signed private
runtime pack and retaining the host Chromium sandbox. Direct-bundle Flatpak is
blocked under the tested SDK configuration; an alternative Flatpak architecture
would need a separate sandbox-preserving design and qualification. No AppImage
artifact exists yet and no clean-host claim is made. This choice is provisional
until clean-machine sandbox/GPU/library tests pass, not a declaration of current
support. The nested-sandbox failure is a concrete packaging risk, not proof that
the mandatory Rust + GPUI product stack is infeasible.

Development build currently relies on `libxkbcommon`, fontconfig/freetype and
runtime XCB/Wayland/Vulkan libraries. `scripts/build-ui.sh` supplies only private
linker-name symlinks for this userspace's existing libxcb/libxkbcommon-x11; it does
not install libraries. On a normal Ubuntu build host, install the explicit
development packages through the user's administrator-managed workflow:
build-essential, pkg-config, libxcb1-dev, libxkbcommon-dev, libxkbcommon-x11-dev,
libfontconfig1-dev, libfreetype6-dev, libwayland-dev, libvulkan-dev, libssl-dev.
Chromium additionally needs NSS/NSPR, GLib/GIO, D-Bus, ATK/AT-SPI, CUPS, X11/XCB,
xkbcommon, GBM/DRM, Cairo/Pango, ALSA and fonts. Use Playwright's pinned dependency
listing as a diagnostic, not `install-deps` or a silent privileged command.
