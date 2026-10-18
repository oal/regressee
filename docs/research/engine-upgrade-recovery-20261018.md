# Engine upgrade recovery — 2026-10-18

The user reported `regressee-desktop` failing with
`version_mismatch: installed engine protocol differs` after rebuilding.
Engine lifetime is intentionally independent of desktop lifetime; closing the
window does not stop the old engine. Protocol 8 was introduced for deletion
scopes/receipts. Previously both startup and the rebuilt `engine stop` rejected
the prior protocol, leaving only the old executable or explicit foreground serve
after shutdown as a recovery route.

## Correction

- Keep all product/status/authority operations strictly version-matched.
- Permit **explicit, uncredentialed Stop only** with the unchanged protocol-6/7
  Hello/Stop envelope. Verify the private descriptor, peer user, owner UUID and
  configured cache exactly as before. Do not guess other old/future protocols.
- Preserve the old server's busy/login/agent-session refusal. Do not kill a
  process, cancel work, remove an endpoint, delete a lock or stop anything merely
  because a new desktop starts.
- Replace a stopped previous-6/7 descriptor during autostart only after acquiring
  the protocol-independent ownership lock. A live owner still fails closed with
  actionable upgrade instructions. Startup contenders still defer to the lock.
- User recovery after rebuilding both binaries: finish old work, close old
  windows, run `./target/debug/regressee engine stop`, then reopen the desktop.
  Explicit data/cache flags must match the prior engine. No project-data changes
  are necessary. The user's actual engine was not stopped during testing.

## Checks

Three isolated old-wire peer tests pass: explicit-only lifecycle negotiation;
busy refusal plus owner/cache mismatch; unsupported old/future refusal. Existing
CLI lifecycle integration now also replaces a stopped protocol-7 descriptor
while selecting a new cache. No native launch or real user-store test is claimed.

## Preview.6 preflight

Existing release naming is `v0.1.0-preview.N`, with package crate version `0.1.0`.
The next tag is `v0.1.0-preview.6`; its push runs the established two-platform
workflow and publishes an unsigned experimental prerelease only after both builds
and packaging gates pass. Do not publish partial/mismatched local binaries or
bypass failed gates. Public notes retain the known UI/native qualification limits.

Preflight corrections also include the public DOM help document in source/binary
packages, classify its internal design plan as omitted, replace a personal
test-evidence root with an explicit private approved-root environment setting,
and apply rustfmt-only changes to three previously nonconforming UI files.
Retaining test-report evidence now requires both
`REGRESSEE_DOM_REPORT_APPROVED_ROOT` and `REGRESSEE_DOM_REPORT_EVIDENCE`; neither
has a default and no prior evidence is moved or rewritten.
