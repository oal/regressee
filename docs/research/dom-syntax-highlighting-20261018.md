# DOM snapshot syntax highlighting — 2026-10-18

## Approved brief

The user approved a focused viewer UX pass with calm, familiar syntax coloring,
not a viewer redesign. Audience: somewhat technical non-developers inspecting
website changes. Mode: Operate. Preserve the native unified-neutral/teal world,
current controls/layout and read-only evidence/explicit approval boundaries.

## Implementation

- `crates/gui/src/ui/dom_syntax.rs` lexes the actual normalized record format
  emitted by `DomSnapshot::format_source`, not HTML. Blue structure/element names,
  violet attributes and warm quoted values distinguish roles. Depth, namespaces
  and punctuation are readable muted ink; page text remains primary ink. Quoted
  escape sequences, Unicode, prefixed elements, doctype, shadow and template
  markers are covered. Unknown/malformed rows remain literal plain text.
- `DomDiffDocument::new` prepares syntax ranges on the existing display worker.
  Ranges use original UTF-8 bytes and contain no theme-specific paint. Cosmetic
  indexing is independently bounded to 2,048 spans per row / 262,144 per document;
  exhausting that budget removes coloring, not evidence or comparison authority.
- `ui/dom_diff.rs` resolves only visible wrapped segments into native text runs.
  Same fonts, metrics, original offsets and viewport-only shaping. Quiet full-row
  added/removed backgrounds complement existing numbers and `+`/`−` gutters.
  Search/selection backgrounds override syntax with contrast-tested foregrounds;
  current matches differ from other matches. Existing match-to-Copy selection is
  preserved. Manual selections take precedence. Caret/newline selection remain
  visible. No source reinterpretation, active links or new controls.
- `ui/theme.rs` owns explicit light/dark source-only tokens. Each syntax foreground
  meets 4.5:1 on ordinary, added and removed rows; interaction foregrounds meet
  4.5:1 against their actual background. Live theme changes recolor without
  changing geometry or stored source.

No new dependency, font, raster, animation, source/storage/wire format, capture,
review or baseline operation. Only isolated synthetic projects are used in tests.

## Verification

**Scoped PASS for syntax/readability in the live review source and diff views.**
Recorded-History inspection remains **BLOCKED by its pre-existing DOM loading
failure**; this is not a whole-DOM-feature, whole-app or release pass. No claim
that the new highlighting repairs that independent failure.

Commands use `mise exec rust@1.98.1 --` and the existing host-only linker aliases
at `/var/home/olav/Data/opencode/tmp/opencode/regressee-native-motion-20261018-r7qx28e2/link-libs`.

- `cargo test -p regressee-gui --features ui-tests --lib ui:: -- --test-threads=1`:
  **122 passed**. Includes both source-format goldens; malformed/bounded plain
  fallback; Unicode/wrapped-token and search/selection precedence checks; byte-exact
  copy; native text-run geometry/theme invariance; existing stale-load, input-gate,
  folding, bounded-viewport, keyboard and pointer regressions.
- `cargo test -p regressee-gui --features ui-renderer-capture,ui-inspection
  --bin regressee-desktop dom_workflow_tests -- --test-threads=1`: **9 passed**.
  Real local IPC covers independent kinds, notes, frozen sources, corruption,
  explicit approval and short source viewports. These are not OS-input tests.
- Strict library Clippy is **blocked by two unchanged engine-client warnings**:
  `crates/app/src/engine/client.rs:212,239`, `clippy::manual_range_patterns`.
  The same command with only that lint allowed passes:
  `cargo clippy -p regressee-gui --features ui-tests --lib -- -D warnings
  -A clippy::manual_range_patterns`. No unrelated source was repaired.
- Production-feature build passed: `cargo build -p regressee-gui --features ui
  --bin regressee-desktop`. Test/renderer features are not required by the app.
- The final ignored native capture test
  `ui_renderer_capture::ui_dom_syntax_capture::renderer_capture_dom_syntax`
  **passed**, producing **20 current-source GPUI/Blade frames**: Diff, Before,
  After, search and selection, each in light/dark at 1280×800 and 820×600.
  The capture uses the `ui-renderer-capture,ui-inspection` features and the explicit
  approved/output root environment variables required by the shared harness.
  Its selection scene checks simulated clipboard text against the entire original.

Native GPUI/Blade evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-dom-syntax-20261018-wr8_quuo/`.
The first batched inspection halted after five light/large frames at the already
documented recorded-History missing-source assertion. The harness continuation
collected the remaining scene diagnostics rather than abandoning other themes and
sizes. All four History cases failed before source painting, with
`DOM viewer selection changed`, no loaded DOM frame and a visible failure message.
This matches the pre-existing blocker in `dom-snapshot-verification-20261017.md`.
Engine selection/identity guards were not relaxed. The permanent, separately
ignored `renderer_capture_dom_syntax_history` test retains the source-presence
assertion; it is not included in the passing syntax-matrix result.

The one final confirmation uses explicit search terms to reveal a real removed
attribute row and current/passive search states rather than only top-of-document
context. No production UI change was needed after the initial visual inspection.
All 20 final frames were inspected in two contact sheets, plus full-resolution
light/large and dark/compact diff frames. Syntax, removed-row treatment, search,
selection, compact-source clipping/scrolling and existing controls are legible in
the exercised views. Added-row token contrast and paint logic are covered by
source/model tests; an added row was not independently inspected in these PNGs.
The normal compact toolbar remains scrollable with the existing source-height
reservation; no layout/toolbar rewrite was introduced.

`confirmation/` contains the sole final matrix; `inspection/` retains the initial
24 scenes, including the four actual History failures. The two visual rounds are
complete; no further polishing loop was run. `confirmation-receipt.json` verifies
all 20 PNG hashes/dimensions/opacity flags and **180 current-source checks** across
nine relevant source inputs. Logs live alongside the receipt as `ui-library.log`,
`dom-workflow.log`, `confirmation.log`, `inspection-history-blocked.log`,
`production-build.log` and both Clippy logs. These are synthetic verification
artifacts, not shipping rasters or actual user results. Earlier frames retain
their original harness hashes and are not relabelled as final confirmation.

## Qualification limits

Source/unit tests, simulated native interactions and offscreen renderer pixels
do not qualify real desktop input/clipboard, screen readers, OS presentation,
Windows/macOS, compositor performance or release readiness. No user project,
website, engine or login was accessed, stopped, rewritten or approved.
