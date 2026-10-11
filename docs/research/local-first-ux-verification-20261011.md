# Local-first UX verification — 2026-10-11

## Scope

User-approved full speed pass, implemented by GPT‑6.1 Sol agents with separate
ownership of viewer/storage, the shell/controller, and History presentation.
The incumbent desktop design and concurrently edited capture-only behavior are
preserved. Review search keeps the inspected evidence and ignored-area drafts
when a filter hides its target. No existing user project, website visit, commit
or push is part of this implementation pass.

See `docs/local-first-ux.md` for the interaction contract. The integrated Rust
gate and bounded, independently reviewed renderer check pass.

## Integrated gate

- `cargo test --locked --workspace --all-features -j 4`: **267 passed, zero
  failures, 38 deliberately ignored**. This includes the 3,000-row live-search
  regressions, unchanged inspection/mask/approval selection, independent read
  waits, exact preview exclusions, stale viewer rejection, network draft loading,
  reference environment context, disabled-local-window policy handling, and
  current-page History refresh after another connection changes the baseline.
- Strict all-target/all-feature workspace Clippy passes (`-D warnings`).
- Normal UI workspace binaries build with `--locked --workspace --features ui
  --bins -j 4`.
- Formatting is checked. Existing vendor/dependency warnings remain.

The first integration run exposed five older tests that assumed viewer/storage
reads used the primary mutation wait. Their waits/known-network fixtures were
updated without dropping functional assertions. A new regression's debug-bound
selector was corrected to the pinned GPUI API; the empty second History page now
asserts its actual empty scope rather than indexing a nonexistent baseline.

The independent integrated source review found three P2 edge cases, now covered
by passing regressions: valid disabled-local-window policies rejected by numeric
validation; prefetch adjacency disagreeing with filtered/sorted queue order; and
no current-page History refresh action after separating the storage census.
History also carries the current saved active-baseline ID independently of the
loaded page and inspected evidence, so a refresh need not overwrite drafts or
mislabel an externally activated version.

## Bounded renderer result

**Final independent disposition: scoped PASS**, no remaining source-proven or
rendered P0/P1/P2 in the inspected scope. This is not whole-app acceptance.

After personally checking an opaque, readable GPU/font calibration with four
exact 1,024-pixel colour patches, the isolated matrix passed in **8.78 seconds**:
14 PNGs, seven states at both 1280×800 and 820×600. States cover ordinary review,
filtered-out inspection with a dirty draft, live discovery search, direct History,
filtered History runs, automatically loaded network settings and field-local
validation. History-run and network views were deliberately scrolled to the
inspected section; they are not claims that those sections open above the fold.
All application PNGs were personally opened and independently inspected.

The batch asserts the synthetic saved project/evidence are unchanged. Metadata
verifies opacity, exact dimensions and PNG hashes. Source bindings before/after
match **290 current working-tree source/config inputs** and the test executable,
independently verified by the reviewer. These images bind to the edited candidate,
not merely to the older committed checkpoint.

At 820×600, Discovery retains the earlier crowded first-viewport limitation;
this evidence does not qualify a complete visible first inventory row in every
state. Lower form corrections require scrolling. The draft fixture is a valid
empty-but-dirty ignored-area state and does not visually qualify nonempty mask
editing. Existing simulated mask-coordinate tests pass, but are not OS-input
certification. No further polish/correction round was requested or performed.

## Backend evidence

- Default-feature workspace **library** tests: 153 passed, 4 deliberately
  ignored, zero failures.
- Application library tests with `ui`: 53 passed, 2 deliberately ignored,
  zero failures. New regressions cover mailbox coalescing/priority, stale
  publication, silent prefetch, identity rejection, revision invalidation,
  immutable cached pixels, corrupt new reads, and LRU byte/entry budgets.
- A separate source reviewer found no source-proven P0/P1/P2 regression in
  the completed backend scope. This is not UI or runtime acceptance.
- Synthetic serial microprofile, 20 frames: old per-tile metadata path
  **321.703 ms**; warm cached path **0.284 ms**. Warm-session source counts:
  1 snapshot, 1 source, 1 plan, 7 decodes, 182 tile hits. This comparison uses
  the current decoder and synthetic CAS evidence. It excludes native event
  dispatch, GPUI layout/paint, GPU uploads/presentation and real-site capture.
  It establishes work reuse, not a whole-app speedup or latency guarantee.

An initial `cargo test --locked --workspace` attempt cannot compile existing
`product_flow` authentication fixtures without `ui-tests`: their helpers are
feature-gated. The library-only check avoids that pre-existing integration-test
configuration limitation; the integrated application gate uses all features.
Existing vendor GPUI floating-literal and dependency future-compatibility warnings
remain outside this pass.

## Evidence location and limits

Private logs/render outputs:
`/path/to/private-tests/opencode/regressee-local-ux-no8bR8/`.

The 64 MiB tile budget accounts for decoded payloads retained by the cache,
not external frame Arcs, GPU atlases or total process RSS. Metadata budgets are
also bounded. Cached data is for display only; approval, activation, export and
mutation integrity checks remain authoritative and uncached. A single storage
owner and bounded reply channel still mean background work can create limited
backpressure; this pass makes no universal instant-response claim.

Offscreen/simulated development-host evidence is not OS keyboard/pointer/IME,
screen-reader, high-contrast, Windows/macOS or packaging qualification. Historical
captures and scale timings are not substitutes for this candidate's evidence.
