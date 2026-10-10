# Updates, evidence preservation and recovery

There is no automatic app updater or hosted update service in this candidate.
Default browser updates require reviewed application-embedded official component
pins and complete file inventories; no separate runtime endpoint/key is required.
Optional signed-pack overrides retain their separate owner-managed endpoint/key.
App signing, vulnerability response, support expiry and migration remain gates.

Runtime repair installs a **new generation** after embedded official pins (or the
configured pack signature), inventory/hash,
space and sandboxed smoke checks. Failure retains the existing generation.
Generation selection never upgrades the runtime frozen into an existing baseline.
Do not roll an expired/revoked public runtime back into navigation eligibility.
Historical screenshot review remains local even when further capture is blocked.

Before a material runtime/browser change, explain that rendering may change.
The user chooses a complete compatible new baseline, explicitly reviews coverage,
then activates it. Partial approvals cannot mix incompatible capture settings.
Original runs/baselines/screenshots are never automatically migrated or deleted.
Keep old screenshots and source/runtime identity for recovery and provenance.

For a future app/database migration: back up the exact database/WAL using a
consistent supported backup operation plus referenced immutable objects before
schema changes; present migration impact and recovery location first. Do not
downgrade a newer schema in-place. Restore only a consistent snapshot into a new
private data location, preserving the original for diagnosis. Automatic app
rollback/backup UI is **not implemented** here.

Build/install/upgrade/uninstall must only replace application installation files
and shortcuts. Never automatically remove projects, baselines, sessions or runtime
history; any data removal is a separate explicit user choice with impact preview.
No uninstaller/data-removal implementation is shipped in this private tar candidate.

Interrupted runs reopen as interrupted and retain valid captures. Resume validates
frozen runtime/settings/session and records mixed capture times. Integrity repair
distinguishes temporary writes, unreferenced complete objects and referenced
evidence; deletion is deliberate, bounded and reference-aware.

Release owner policy still needed: supported patch cadence (review at freeze and
before publication; at least monthly while supported), vulnerability response,
end-of-support notice, trusted key rotation/revocation and migration wording.
No 2026 public security qualification is inferred from historical loopback tests.
