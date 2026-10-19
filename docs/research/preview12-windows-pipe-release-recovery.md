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
