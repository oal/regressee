# 0002 — Private local pack and ephemeral session prototype

`scripts/bootstrap.py` is developer tooling. It downloads exact Node archives via
TLS and verifies committed SHA-256 values; private Node invokes its private npm
using the committed integrity-locked worker lockfile, with scripts disabled.
Playwright downloads its exact browser revision to `.runtime/source/browsers`.
No global Node/npm, sudo, system browser, or sandbox override is used.

`regressee-spike pack` creates a platform-specific ZIP with every file's checksum,
size and executable bit. Rust setup verifies an Ed25519-signed manifest, OS/arch,
versions, names, file count, duplicates, symlinks, individual hashes and a 2 GiB
extraction budget. It checks free space, stages separately, performs a sandboxed
browser screenshot smoke, and switches runtime directories only after success.
Repair retains the old ready runtime on failure/cancel. UI work uses a serial
background actor and bounded queues. Stale setup stages/locks require explicit
inspection rather than arbitrary deletion. Runtime/evidence are separate.

**Development trust only:** the signing key is intentionally public in source.
This proves integrity mechanics, not production authenticity. Runtime pack
hosting, release key custody, browser provenance verification beyond TLS, streaming
download/proxy/custom CA support, rollback persistence across a mid-rename crash,
and a no-terminal clean-machine installer remain gates. The UI provisions from a
prebuilt local pack; it does not create a release service or run npm for the user.

Saved session data never leaves worker memory: cookies, localStorage, IndexedDB
and explicitly origin-scoped sessionStorage. Headed login uses the same full
Chromium revision as `channel: chromium` new-headless capture. The user chooses
**I’m ready**; synthetic automated button clicking occurs in tests only. Separate
contexts test replay and independent absence of all four storage mechanisms.
Browser closure/cancel clears worker state. Chromium profiles can contain session
data during browser use. Rust assigns a unique private mode-0700 scratch directory
via TMPDIR/TMP/TEMP, removes it after process-tree termination, and on Linux
recovers only marked app-owned scratch after its parent and live process group are
gone. Linux scratch uses the approved `/tmp/opencode/r0` short root: longer project
paths plus Playwright's internal Unix socket names caused Chromium SIGTRAP in a
reproducible probe. Short scratch paths are therefore a runtime prerequisite, not a
sandbox workaround. Crash cleanup still needs independent inspection on each OS. No unencrypted
saved JSON/session export.

OS credential-store encryption, representative enterprise SSO/MFA/device-bound
sessions and restart persistence are not implemented or claimed. Ephemeral mode is
the explicit safe alternative until credential-store qualification. Linux Secret
Service, Windows Credential Manager and macOS Keychain are proposed, untested.

Fault testing found that Playwright **detaches Chromium into its own Unix process
groups**. Node-group killing alone is not process-tree supervision. The worker now
tracks pending browser launches before cancel/EOF/shutdown and closes any launch
that resolves during cancellation. Linux Rust supervision also finds only verified
private-browser executables whose arguments contain the exact owned scratch path,
kills their groups (including crash helpers), and scans briefly after Node exits
for a mid-spawn child. Native quit uses the same scoped emergency cleanup. Failed
scratch deletion restores an ownership marker for recovery. Windows/macOS still
require actual qualification and stronger platform-specific supervision review.
