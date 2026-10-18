# Agent instructions

Regressee is a local-first desktop app and CLI for checking websites for visual
and DOM changes. This file tells coding agents how the repository is organized,
which commands to run and which rules apply. Follow closer instructions in a
subdirectory if one exists; this file covers the whole repository.

## Repository map

- `crates/core`, `crates/storage`, `crates/runtime`, `crates/app` — shared
  services. `crates/app` owns the single local engine both clients attach to.
- `crates/gui` — native GPUI desktop app (`regressee-desktop`).
- `crates/cli` — GPUI-free terminal product (`regressee`); its dependency
  closure must stay free of GPUI and graphics libraries.
- `worker/` — embedded Node/Playwright product worker with its own versioned
  boundary and locked dependencies.
- `site/` — static landing page, deployed to GitHub Pages by
  `.github/workflows/pages.yml`.
- `docs/` — `docs/*.md` are published user guides unless classified internal;
  `docs/research/`, `docs/phases/` and `docs/decisions/` are internal and never
  distributed.
- `scripts/` — CI, packaging and release helpers; `tests/` holds fixtures.

## Build and check

```sh
cargo build --locked -p regressee-gui --features ui --bin regressee-desktop
cargo build --locked -p regressee-cli --bin regressee
cargo fmt --all -- --check
cargo test --locked --workspace --all-features -- --test-threads=1
cargo clippy --locked --workspace --all-features --all-targets -- -D warnings
```

Run tests serially; parallel synthetic workers can exceed the global
eight-browser safety cap. [docs/ci.md](docs/ci.md) records what CI covers and
which native and manual qualification gates remain open.

## Releases

**Before preparing or publishing any release, read and follow
[docs/release-checklist.md](docs/release-checklist.md).** It covers the preview
version and notes updates, the pre-tag gates, the tag and workflow triggers,
release verification, and the website link update. Two rules from it apply at
all times: never move, reuse or delete a published tag or release, and never
publish beyond the established unsigned preview workflow without the owner
asking.

## Guardrails

- Keep the product local-first: no uploads, telemetry or new network access
  without an explicit decision. Never point tests or developer overrides at
  real user data.
- Never disable OS or browser sandbox/security protections to make a check
  pass; `--no-sandbox` and similar bypasses are not acceptable workarounds.
- When adding a `docs/*.md` file, classify it in `scripts/source_release.py`
  (`USER_DOCS` ships with the source archive, `INTERNAL_DOCS` is omitted); the
  source-policy helper refuses unclassified documents.
- Keep claims in the README, docs and site truthful. The limits in
  `docs/support-matrix.md` and `docs/preview-release-notes.md` are deliberate;
  do not soften or drop them.
- Use `--locked` for Cargo commands and refresh `Cargo.lock` in the same commit
  when versions or dependencies change.
