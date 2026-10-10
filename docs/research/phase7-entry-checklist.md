# Phase 7 entry checklist — orchestration preparation

Prepared: 2026-10-10. This is a static implementation inventory, not release
qualification. Phase 6 independent verification is still being recovered; do
not infer its acceptance from this preparation.

## Concrete implementation/packaging work

1. **Portable product supervision.**
   `crates/runtime/src/supervisor.rs:105–108` deliberately refuses non-Linux
   process supervision. Implement Windows/macOS ownership paths before claiming
   multiplatform functionality. Preserve Linux exact ownership and crash cleanup;
   do not replace safe failure with an unqualified broad kill fallback.
2. **Windows storage-space handling.**
   `crates/storage/src/portability.rs::require_space` uses Unix `statvfs` and
   refuses non-Unix platforms. Add a real Windows implementation with overflow,
   error, and safe-write handling; compile-only evidence is not actual disk-full
   or Windows runtime qualification.
3. **Current runtime candidate.** Recheck primary distribution metadata, then
   build and exercise a pinned maintained Node/Playwright/full-browser tuple.
   The historical tested runtime remains loopback-only. Test the complete signed
   product pipeline after any upgrade, particularly native CDP interception,
   session isolation, photo replacement, archives, and baseline/profile version
   boundaries. A version number alone is not security approval.
4. **Mac bundle preservation.** Review symlinks, file modes, helper signatures,
   executable paths, and framework layout in the candidate runtime pack. The
   historical spike packer skips symlinks; do not silently reuse that behavior as
   a release-quality Mac packer.
5. **Linux ABI/package baseline.** The diagnostic snapshot required
   `GLIBC_2.43`, so it cannot be called an Ubuntu 24.04 package. Build against the
   intended minimum userspace or explicitly leave the package unqualified.
   AppImage/Flatpak must preserve Chromium sandboxing under default policy.
6. **Native rendering.** No isolated X server/compositor was available to the
   read-only diagnostic. An owned, verified private test tool/image may enable a
   bounded native attempt; no sudo/global install or host desktop changes are
   implied. Software/virtual display success is not ordinary hardware or human
   usability acceptance. Avoid repeated black-window loops.
7. **Scale.** Run actual 1,000-page/3,000-target capture with observed visits and
   persisted coverage. Record reference hardware, wall time, separate memory,
   disk growth, responsiveness scope, and cleanup. Never substitute generated
   queue rows for actual navigations.
8. **Small-mask visual precision.** Phase 5's remaining P3 is the minimum 4×4
   visible outline footprint; exact PNG geometry is correct. Separate selection
   hit area from actual drawn bounds if refined, and retain seam/pan oracles.

## Delivery controls and documentation

- Reproducible app/runtime/source artifacts, checksums/signature manifests,
  dependency inventory and MPL/third-party source/notice obligations.
- Owner-configured release key custody, signing/notarization and runtime hosting;
  no committed production private key, invented signer, or unapproved publishing.
- First-run and repair guidance, compatibility/support matrix, update/security
  policy and baseline migration rather than silent runtime replacement.
- Opt-in diagnostic export with structural omission of screenshots, login state,
  secret references, website/private query data, and machine-local paths.
- Upgrade/uninstall must retain evidence unless the user explicitly deletes it.
- Clean-machine/platform CI recipes and manual/native task scripts; a recipe is
  not an executed OS qualification result.
- Integrate the already checked private source helper from
  `docs/research/source-release-handoff.md` and
  `docs/research/source-release-verification.md`; adapt the inclusion policy to
  final build/help assets and keep unknown licenses unapproved.
- Use `docs/research/usability-session-kit.md` for actual observed human tasks.
  It is a protocol with blank records, not fabricated usability evidence.

## External/human gates that must stay explicit

Three representative non-developer task observations; authenticated human flow;
real OS credential-store restart/lock behavior; Windows/macOS/clean supported
Linux native input/rendering/dialogs/high-DPI/accessibility; default GPU and
sandbox; actual platform signing/notarization and trusted runtime distribution.
These are not passed by local unit tests or synthetic identities.

Sources: `docs/research/release-qualification.md`,
`docs/research/native-render-qualification.md`, current product source,
`docs/phases/07-release.md`, and `docs/IMPLEMENTATION_STATUS.md`.
