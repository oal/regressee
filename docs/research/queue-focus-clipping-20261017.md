# Queue focus clipping — 2026-10-17

The user's Search and Filter screenshots show the same defect: their outward
focus ring meets the queue/viewer's `overflow_hidden` boundary. The control
itself is inside that boundary, but the left-hand ring is not.

The shared queue/viewer container now includes the existing 3px focus outset
as padding, offset by equal negative margins. This expands its clip boundary
without insetting the controls or changing the space allocated to evidence.
Overflow protection, focus styling, keyboard order, filtering and stored
evidence remain unchanged. The focus helper and container share one named
constant for the existing ring extent; no new visual treatment is introduced.

Changes follow the concurrent relocation to `crates/gui`; that refactor is not
part of this repair.

## Verification

- The existing search layout test now checks the entire focus outset for Search,
  Clear and Filter against the shared clip boundary at 1440×900, 1280×800 and
  820×600, including short no-match queues.
- A new opt-in native offscreen test checks Search and Filter in both themes at
  those sizes, asserting ring/gap pixels on all four sides and unchanged stored
  evidence. Synthetic fixtures are not customer website captures.
- `git diff --check` and scoped Rust formatting checks passed.
- The full-app test attempt is blocked by the concurrent GUI extraction: the
  new GUI crate currently references a missing `jobs` module and unresolved
  `analysis_engine`, `review` and `images` modules. These unrelated migration
  errors were not repaired as part of this change. The initial locked attempt
  also reported the migration's stale lockfile; the offline attempt resolved
  the workspace lockfile but could not compile the GUI.

The regression assertions are added, but no passing execution or fresh rendered
confirmation is claimed. Private capture preparation is under
`/var/home/olav/Data/opencode/tmp/opencode/regressee-queue-focus-qm0ty5bs/`;
no new frames were produced. Native OS input, accessibility, other platforms and
whole-app release qualification remain outside this scoped repair.
