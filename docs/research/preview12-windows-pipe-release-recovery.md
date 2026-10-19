# Windows pipe preview release recovery — 2026-10-19

The owner requested committing, pushing and publishing the Windows OS error 231
connection fix through the established unsigned experimental preview workflow.

## Failed preview.11 attempt

- Connection fix: `6b54662a7da04d1d0924b32c996467a8d6c418df`.
- Release input: `ec6381ff09a071f293f9e83d57650ad915156e64`.
- Run: <https://github.com/oal/regressee/actions/runs/39299778016>.
- Linux and Windows application contract tests passed. The early Windows engine
  gate passed the real named-pipe contention regression. This is Windows Server
  evidence, not Windows 11 desktop qualification.
- Both packaging jobs refused `site/` with `unclassified-root-entry` before
  production compilation. The publish job was skipped; no tag or release was
  created by this manual-dispatch attempt. Preserve the attempt unchanged and use
  `v0.1.0-preview.12`, rather than reusing its requested number.

## Scoped correction

The static marketing website is deployed independently by the Pages workflow and
is not a product build input. Explicitly omit `site/` from application source
archives, record that omission in both source manifests, and retain refusal when
required application source references an omitted website file. New synthetic
regressions cover the Linux source helper and both portable package targets.
The portable regression reproduced the original failure for both targets before
the correction. An actual-checkout source-selection regression now runs in the
early packaging gate, before the lengthy application tests. A local preflight
confirmed that portable and fd-pinned inventories match, both include the pipe
fix and regression sources, and neither includes the separate website.

Local format, all-target/all-feature build, Clippy, and the seven release helper
gates passed after adding private linker aliases for already installed host
libraries. The all-feature run reproduced the same 15 native UI assertion failures
as previous-main Foundation run `39151176537` (263 GUI tests passed, 75 ignored).
The release notes retain those qualification limits and other pre-existing
Foundation matrix failures. No tests or security guards were disabled.

The next workflow must still pass both platform jobs and all packaging, source
identity, dependency/ABI, and publish gates before a release can be claimed.

## Published recovery

- Release input and immutable tag target:
  `3ed467d734048d41e52420f872681d0b0b217bb8`.
- Successful workflow: <https://github.com/oal/regressee/actions/runs/39306501017>.
- Published unsigned experimental prerelease:
  <https://github.com/oal/regressee/releases/tag/v0.1.0-preview.12>.
- Both platform application tests, production builds, CLI dependency boundaries,
  native ABI/DLL inspections, source identity merge, and publishing passed. The
  Windows contention regression passed in both early and full application tests.
- All four product/platform archives, source archive, source manifest and
  checksums are present. Published asset digests match `SHA256SUMS`; the downloaded
  source manifest's checksum and pipe-fix source hashes match the tagged sources.
  The manifest records the website omission and excludes internal research.
- Desktop archive sizes still round to 36.1 MB (Windows) and 42.9 MB (Linux), so
  website size labels remain accurate. Only download/version references change;
  the owner's separately committed website redesign is preserved.
- The downloaded Windows desktop ZIP's SHA-256 is
  `e42b7f154781bf3d3c49196adceb73d76bf4f1a6da9f01804e116d4ad7aab579`,
  matching `SHA256SUMS`. ZIP integrity, every internal checksum, both production
  executable hashes/features, source identity and build receipt were verified
  without executing the Windows binaries. Bundled notes identify preview.12 and
  the OS error 231 correction.
- Website update: `c8159813ce4eddbe7bedbabffec95b4f8539688d`.
  Pages run <https://github.com/oal/regressee/actions/runs/39315832625> passed.
  The live HTML and platform script point to preview.12; all four deployed
  desktop/CLI download URLs return HTTP 200.
