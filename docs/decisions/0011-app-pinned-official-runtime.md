# 0011 — App-pinned official runtime components

Date: 2026-10-10. **Approved architecture; implementation is not release qualification.**

The user approved replacing the default prebuilt signed-runtime-ZIP plan with
official Node/Chromium downloads and Playwright bundled in the app. The missing
download server/key must not remain an end-user configuration requirement.

## Decision

The application embeds a complete, immutable per-platform catalog and compressed
Playwright/worker source payload. First use downloads exact official HTTPS archive
URLs, verifies pinned SHA-256 and byte counts before extraction, verifies every
installed file against the embedded inventory, and runs the existing sandboxed
owned-worker screenshot smoke before atomic activation. No system Node, npm,
shell installer, generated signing key or Regressee download server is needed.

Current pins: Node 24.21.0, Playwright 1.64.0, Chromium 156.0.8078.4/revision 1248,
worker 5.0.0. Catalogs expire on 2027-01-10 UTC. Updating these pins is a reviewed
application/source update, never an automatic rendering change beneath a baseline.

`OfficialPinned` identifies this trust path; it is not a `QualifiedPublic`
certification. A source build trusts its source tree. Public app signing, licensing,
supported OS/clean-machine qualification and accessibility remain separate gates.

The existing signed-pack override remains supported and signature-verified. An
explicit malformed override never falls back to official downloads; a legacy
distribution cannot select `OfficialPinned`. `DevelopmentLoopback` restrictions
are unchanged. Existing damaged or mismatched installations require explicit
repair, and comparisons/resumes never install a different runtime automatically.

Linux/Windows x86_64 have default catalogs. macOS catalogs are preparation only:
the missing owned-process-tree supervisor still blocks setup before downloading.

## Trust and maintenance consequences

Node checksums were verified with an official release-key signature, and npm
tarballs with SHA-512 SRI and registry signatures. Chrome's official API supplies
no independent publisher checksum; authenticated generation-time HTTPS bytes are
converted to embedded application-owned pins. These distinctions are recorded in
the asset provenance, not hidden behind an invented signature.

Playwright's empty `DEPENDENCIES_VALIDATED` file is permitted only as ordinary,
non-executable bookkeeping at the exact pinned browser root. It is never bundled
or fabricated, and host validation/sandboxing are not disabled. All executable and
library evidence remains immutable and fully verified.

See the asset, integration, provisioner and independent-verification reports in
`docs/research/official-runtime-*20261010.md` and
`docs/research/default-official-runtime-integration-20261010.md` for actual results.
