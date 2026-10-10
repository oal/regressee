# 0010 — disk-backed qualification scratch

Date: 2026-10-10

The user reported that `/tmp/opencode` had been RAM-backed, causing OOM kills and
other issues, and that it has now been moved to disk-backed storage. A fresh
orchestrator check confirms `/tmp/opencode` resolves onto the ext4 filesystem
mounted at `/home/developer/Data`, with about 1.2 TiB free at the observation time.
This corroborates the current disk backing; it does not retrospectively prove
the cause of every earlier timeout, rendering failure, or defect.

- Use a new owner-private directory beneath the physical
  `/path/to/private-tests` path for retained verification evidence,
  runtime generations, compiler outputs, and explicit TMPDIR overrides.
- Preserve store/runtime ancestor symlink and reparse protections. Resolve the
  harness compatibility alias at orchestration boundaries; do not weaken app
  safety checks to accept that alias as a trusted storage ancestor.
- Preserve prior reports, failed attempts and valid artifacts. Do not reset or
  discard earlier independently reproduced correctness/security findings.
- Phase 7 verification may run one new scale attempt on explicitly recorded
  disk-backed paths after checking the prior fixture/runtime/source identities.
  Keep the original 1,200-second capture budget; no invented throughput pass.
  Capture memory/disk/coverage/recovery evidence and distinguish changed storage
  conditions from application changes.
- The disk move does not qualify native UI, Windows/macOS, human usability,
  signing, current public browser distribution or accessibility.

Observation commands: `df -h /tmp/opencode /path/to/Regressee` and
`findmnt -T /tmp/opencode -o TARGET,SOURCE,FSTYPE,OPTIONS`. No mount, global
environment, host settings, or historical evidence was modified by this check.
