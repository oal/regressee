# Screenshot loading treatment — 2026-10-16

## Scope

The user requested a loading state similar to the improved missing-screenshot
view. This narrow Operate-mode refinement keeps the existing native navy/teal
identity, loading copy and all capture/read/approval behavior.

- Screenshot reads, initial saved-evidence loading and screenshot preparation
  reserve the same white, thin-bordered stage as missing screenshots.
- A centered 32px native teal stroke spinner, 18px heading and muted 14px
  explanation replace the small top-aligned loading row. No percentage, skeleton
  screenshot, new control or additional animation is introduced.
- Loading and missing-image messages share their frame, 440px maximum measure,
  24px insets and overflow-only scrolling. Short messages center; constrained
  messages start at the top and remain reachable by scrolling.
- Existing compact status indicators remain 16px. The existing reduced-motion
  override stops the same spinner clock. Initial loading keeps one status owner.
- Queue search remains enabled while reading a selected screenshot, and zoom
  remains unavailable until pixels arrive. Missing-image failure semantics,
  source context, drafts and baseline guards are unchanged.

## Verification and finish verdict

**Scoped verdict: pass.** One in-thread batched inspection covered 1280×800 and
820×600 GPUI/Blade frames: normal loading, capture-only inspection, hidden Results,
initial evidence loading and the filters overlay. The framed state fills the
available stage, centers correctly and preserves readable copy at both sizes.
No further visual correction round was needed. This is not independent review
or whole-app acceptance.

- Native UI build passed: `cargo build --locked -p regressee-app --features ui
  --bin regressee`.
- Shell UI suite: **200 passed, 1 intentionally ignored** native-fixture test.
  This includes screenshot switches, loading/failure distinctions, missing-image
  recovery scrolling, draft protection and approval guards.
- Shared UI suite: **62 passed**. The added loading-frame test covers centering,
  small-height scrolling and reduced motion; native arc geometry covers both
  16px and 32px sizes. Existing contrast and missing-image checks still pass.
- The loading matrix now covers 12 scenes at both desktop sizes, including
  hidden Results, capture-only inspection and open filters. It checks stage
  geometry, copy bounds, one status owner and unchanged stored project evidence.
- Explicit renderer test passed and produced **24 offscreen PNGs**, each with a
  source-hash metadata sidecar, plus `loading-manifest.json`.
- `git diff --check` passed. No shipping raster, font or design token was added.

Commands used the repository's pinned Rust 1.98.1 through `~/.cargo/bin`.
The host's old `.runtime/link-libs` symlinks did not resolve, so diagnostic-only
linker aliases were created under the private evidence root; project toolchain
configuration and existing aliases were not changed. Renderer commands set
`REGRESSEE_RENDERER_APPROVED_ROOT` to that root and
`REGRESSEE_RENDERER_CAPTURE_DIR` to its `inspection` child.

Private evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-loading-stage-20261016-n9_5rqyb/`.

Evidence is synthetic offline Linux GPUI layout/rendering, not real website
results, OS-input, accessibility, cross-platform or release qualification.
Concurrent unrelated workspace changes were retained; capture sidecars bind
the evidence to the sources used to compile it.
