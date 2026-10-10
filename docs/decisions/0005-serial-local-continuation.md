# 0005 — resume serial local implementation with qualification gates open

Date: 2026-10-09

## User direction

After the Phase 0 implementation and verification pipeline stopped, the user
approved Python controls and explicitly requested: “Please continue with the rest
of the phases.” The existing instruction remains one GPT 6.1 Sol High subagent
at a time, implementing a phase and then having a different agent verify it.

## Decision

Resume local implementation and independent verification through the remaining
phases. Pending external/platform qualification is tracked rather than falsely
declared passed or repeatedly used to stop all further development. This
supersedes the earlier blanket recommendation not to start Phase 1 while any
Phase 0 gate is open; it does not establish full Phase 0 acceptance.

Use a fresh implementation agent and a fresh verifier for each phase. Fix
material local correctness/security defects before depending on that work.
Use bounded corrective passes; do not enter indefinite native screenshot loops.
Every agent records actual test results and tests not run.

## Unchanged requirements and boundaries

- Rust + GPUI, Windows/macOS/Linux first public release, local-first operation,
  MPL-2.0, and the confirmed feature scope remain unchanged.
- Cross-platform/clean-machine evidence, default graphics and native interactions,
  accessibility, trusted automatic runtime delivery, secure session persistence,
  and actual release signing remain genuine gates. Implement locally testable
  components in their relevant phases; retain external checks as pending.
- Python-control approval permits actual app-owned control/testing attempts;
  it is not proof those controls, graphics, or native UI tests work.
- No user's visual approval is invented. Design proposals and pending human
  visual/usability review must be identified honestly.
- No permission to publish releases, create external services, disable browser
  sandboxing, install global tools, alter unrelated windows, or make privileged
  host changes is inferred.
- The application is not release-qualified until the full release gates pass.

## Evidence

`docs/phases/00-native-recheck-2026-10-09.md` records the newly approved native
attempts. They obtained owned-window screenshots but did not prove rendering or
input. Prior verification reports remain historical evidence, not acceptance.

`docs/IMPLEMENTATION_STATUS.md` is the current serial pipeline index. Phase
implementation and verification reports contain detailed evidence.
