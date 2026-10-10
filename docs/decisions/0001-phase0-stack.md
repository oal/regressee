# 0001 — Provisional feasibility pins and native controls

Rust **1.98.1**, GPUI **0.2.2** from crates.io (Apache-2.0); Cargo.lock freezes
transitives. Exact direct pins are in `crates/spike/Cargo.toml`. No floating git
branches or external component framework. One crate with runtime, supervisor,
image and UI modules is sufficient for this spike; no Phase 1 database/domain/UI
foundation is introduced.

GPUI provides Windows, macOS and Linux backends in this pin. Native text handling
uses an attributed adaptation of its Apache-2.0 input example. Focus traversal,
clipboard, IME composition, native file prompt, OS menu registration, keyboard
actions and tile rendering have runnable controls. This is not a qualified text
component: grapheme movement, selection dragging and screen-reader semantics need
further work. Searching GPUI 0.2.2 source found no accessibility tree/accesskit
bridge; keyboard access is not proof of screen-reader support. Treat accessibility
as a material unresolved stack risk, not a hidden Phase 1 checkbox.

Node **22.20.0** (bundled npm **10.9.3**, developer pack building only), Playwright
and playwright-core **1.56.1**, full Chromium **141.0.7390.37**, revision **1194**,
FFmpeg revision **1011**. These are reproducible historical feasibility pins,
**not a 2026 security-qualified release runtime**. Only generated loopback fixtures
are exposed in this bench. Before real sites, qualify a current maintained runtime
and require new compatible baselines for material rendering changes.

The worker is plain ESM JavaScript, supported natively by the pinned Node version;
no TypeScript bundler or developer package manager is needed at runtime. Rust owns
supervision and setup; Playwright is used as a library, not a report UI. Test-only
fault commands exist only in this feasibility worker and must not ship in product
runtime packs.
