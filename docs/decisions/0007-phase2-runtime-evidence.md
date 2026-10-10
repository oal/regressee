# 0007 — configured runtime trust and immutable single-target evidence

Date: 2026-10-09. Phase 2 only; independent verification pending.

## Runtime authority

The product does not embed the historical development private key or invent a
production host/key. A local installation configuration explicitly supplies its
distribution URL, Ed25519 public key and scope. Signed manifest v2 specifies the
same scope, pinned Node/worker/Playwright/browser identity, platform, relative
entry points, complete file hashes/sizes/executable modes and (for qualified
public runtimes) an unexpired support deadline. HTTPS is mandatory except for
literal-loopback development HTTP. Redirects and URL credentials are refused.

Configured QualifiedPublic is externally established release authority, **not**
local certification from a Boolean or a screenshot. No such release pack has
been qualified here. Missing trust fails closed with an actionable explanation.
The actual test distribution is development-only: verified historical runtime
bytes plus the new product worker, signed with a fresh ephemeral test key and
served by an owned loopback fixture server. Public targets/resources are refused;
the original fixture worker behavior is not the product capture implementation.

Verified fresh generations are smoke-tested with sandboxed actual product worker
handshake/screenshot/shutdown, then marked ready and atomically selected. Repair
does not overwrite originals or silently erase previous runtime generations.
Hash/installed-inventory checks run before execution. Download cancellation is
polled while awaiting headers/body; extraction is bounded and disk-preflighted.
No system Node/npm, unverified executable launch, privileged setup or production
signing private key is part of this product path.

## Evidence and concurrency

SQLite schema 3 adds only used run/baseline/analysis ownership, preserving existing
project/artifact migrations. A run freezes one target, inventory, immutable
profile, requested URL and rendering identity. Completed runs are immutable.
Successful originals/metadata and CAS thumbnails/512-row tiles are retained and
verified. Reference pinning is in the final domain transaction; corruption never
creates an active baseline. Explicit successful capture confirmation creates an
immutable baseline and updates the project revision/active pointer atomically.
Checks use that baseline’s frozen target and create another run, never discovery.

The serial background owner handles DB/files/network/process/decode/diff. Four
command/update slots, nonblocking UI submission, bounded terminal backpressure
and independently signaled cancellation keep blocking work off GPUI. Worker
frames/queues are bounded. Linux ownership includes private short scratch,
parent-death and pre-exec parent checks plus detached-browser cleanup. Other OS
process-tree qualification is not silently simulated: product launch fails closed
there pending an adapter. Restart interrupts dead-owner runs only; no startup GC.

## Initial analysis and viewing

`rgba-max-channel-v1` compares aligned rows with an explicit 16/255 channel
tolerance; any changed pixel flags change. Missing height bands are changed even
when their visible color is white. This is versioned initial evidence, not a
calibrated/perceptual probability or a defect judgment. Width/runtime/profile
mismatch does not establish unchanged. Failures have their own classification.

The GPUI viewer shares zoom/pan across before/after/diff and requests at most two
512-row BGRA tiles per pane from the background owner. Originals are never whole
GPU textures. Retired RenderImage IDs are explicitly removed from the sprite
atlas. Native rendering/input and human approval remain pending; simulated tests
and built binaries cannot qualify them. Phase 3+ remains separate.
