# Whole-app workflow test preview

This preview includes the whole-app UI implementation and feasible validation
from Phases 1–7 on 2026-10-12–13. It remains an **unsigned experimental prerelease**, not a supported
release. The release's source commit and each archive's `build-info.json` identify
the exact build; the application crate version remains `0.1.0`.

## What changed

- Review gives evidence more space, keeps page/profile/environment context visible,
  and places the queue's collapse/restore controls beside the queue.
- Review decisions, notes and baseline selection have clearer task ownership;
  baseline consent preserves selected versus inherited references explicitly.
- Guided setup distinguishes saved settings from drafts, routes corrections to
  the relevant stage, and preserves login drafts across asynchronous saves.
- History, sharing, cleanup and failure recovery use clearer task language while
  retaining explicit privacy and irreversible-action warnings.
- Persisted synthetic 100/1,000-page regressions exercise distinct originals,
  selective approval, stale/corrupt evidence refusal, retained references and
  byte-identical baseline export/import. These are **not browser throughput or
  native-input benchmarks**.

## Limits and privacy

Source/controller/storage and Linux GPUI offscreen checks passed independently.
Native desktop input, accessibility, clean installation, default graphics,
cross-platform usability and full release/license qualification remain pending
or blocked. Windows/NVDA and macOS/VoiceOver have not been qualified; macOS is
not a downloadable target and still has a process-supervision code gap. CI
compilation or simulated controls do not establish those capabilities.

Data remains local unless explicitly shared. Baseline archives contain website
pixels and addresses, but do not export login sessions/cookies. Ignored regions
are comparison policy, **not pixel redaction**. Browser installation on first use
needs internet access; downloads do not include the development `.runtime` tree.
Do not disable operating-system or browser security protections to run a preview.

See `docs/release-building.md`, `docs/support-matrix.md`, and
`docs/research/whole-app-ui-review-20261012/phase7-verification.md` in the exact
source archive for requirements, evidence scope and outstanding gates.
