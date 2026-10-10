# Phase 0 — cross-platform feasibility and risk retirement

## Outcome

Prove the requested Rust + GPUI + privately managed Playwright stack can deliver
the core experience on Windows, macOS, and Linux before investing in the full UI.

Read `PRODUCT.md` and `PLAN.md`, especially Sections 5, 8, 9, and 13. This is a
small executable spike, not a production feature implementation. Preserve
useful spike code only after review; no framework replacement without approval.

## Work

- [ ] Select an exact GPUI/gpui_platform pin and Rust toolchain. Build/open a
  window on each proposed supported OS with the appropriate graphics backend.
- [ ] Exercise text input, focus traversal, file dialogs, scrolling, menus,
  image rendering, keyboard shortcuts, and available accessibility support.
- [x] Test large full-page screenshots with tiled viewing and bounded decoding.
  Establish safe image-dimension/pixel budgets and a reference hardware profile.
- [x] Select compatible pinned Node, Playwright, and Chromium versions.
- [ ] Create a prototype private runtime pack and setup/repair flow. Prove
  screenshot capture on clean machines that have no Node or developer tools.
- [x] Keep browser sandboxing enabled. Identify unavoidable OS prerequisites.
- [ ] Evaluate Linux AppImage versus Flatpak/runtime packaging, including the
  Fedora development environment, browser launch, GPU, and secret-store access.
- [ ] Validate headed login followed by equivalent pinned headless capture.
- [ ] Validate saved-session replay with cookie, localStorage, IndexedDB, and
  sessionStorage fixtures; test a representative SSO/MFA flow where available.
- [x] Investigate dimension-preserving image replacement on img/picture and
  raster-background fixtures. Record difficult/unsupported cases.
- [ ] Compare unchanged captures on the same machine and across OS families;
  establish honest environment-mismatch handling.
- [x] Test a minimal supervised worker protocol, cancellation, process-tree
  cleanup, and recovery from a crashed browser.

## Deliverables

Implementation/evidence: [`00-status.md`](00-status.md). Checked entries mean
the stated investigation/pin/budget was delivered locally, **not** that all OS or
clean-machine gates passed. Remaining mixed-scope entries are deliberately open;
the status document distinguishes implemented subparts from missing qualification.
Independent findings remain in [`00-verification.md`](00-verification.md); local
corrections and serial regression outcomes are in [`00-corrections.md`](00-corrections.md).
Neither corrections nor backend tests close the full external/native gate.

- Tested OS/version/architecture and packaging matrix, including limitations.
- Exact dependency/toolchain choices and a private-runtime pack strategy.
- Evidence from clean-machine runs, not just development-machine success.
- Image/memory budgets and rendering approach.
- Decision records for GPUI, runtime packaging, Linux setup, authentication,
  image replacement, and comparability risks.

## Completion gate

A tester can open the GPUI spike, provision its private runtime, interactively
log into a fixture, capture a public and authenticated page, view a tall image,
and cancel without orphan processes on every declared launch OS. No terminal
is required in the intended end-user setup path. Unsupported Linux prerequisites
and distributions are stated explicitly.

If any committed OS or fundamental session/setup requirement cannot meet this
gate, stop and resolve that decision; do not hide it as Phase 7 packaging work.

## Next

Phase 1 uses the validated dependencies and decisions to create the maintained
foundation and confirm the visual system.
