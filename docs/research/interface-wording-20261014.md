# Clearer interface wording — 2026-10-14

Reviewed Regressee's language for non-developers using Impeccable's clarity
workflow. The existing visual direction and product behavior were preserved.

The baseline-save message now reads:

> Baseline saved. Future checks will compare against these screenshots using the
> same page list. Earlier versions are available in History.

## Scope

- Setup, page search, sample-page access checks and browser status.
- Capture outcomes, comparison coverage, review and ignored-area instructions.
- Baseline saving, updating and restoring; History and screenshot source details.
- ZIP privacy and compatibility, deletion, recovery files and unsaved-work prompts.

Routine messages use pages, screenshots, results and files instead of internal
inventory, target, evidence and object terminology. Exact diagnostic information
remains available. Missing captures and screenshots that could not be compared
are not described as unchanged pages. Privacy warnings, confirmation gates and
saved-baseline protections remain intact. History accepts the old search terms
as well as its clearer current labels.

Writing rules and terminology are recorded in `docs/ux-writing.md` and referenced
from `DESIGN.md`. Three copy regression tests exercise real success replies,
page-list status punctuation and deletion prompts with zero, one and two items.

## Bounded verification

- App library: **110 passed**, two opt-in tests ignored.
- Native shell: **170 passed**, 32 opt-in tests ignored, one known test excluded.
- Core library: **52 passed**.
- Storage library: **35 passed**, two opt-in profiles ignored.
- `analysis_review` integration suite: **7 passed**.
- Production UI build and strict production Clippy passed.
- `git diff --check` and formatting of `ux_copy_tests.rs` passed.

The excluded test is
`workflow::tests::discovery_owns_its_scope_and_review_preserves_provenance`.
An isolated final recheck still fails at
`!s.workspace_control_mounted(Control::Guided(16))`. This existing failure was
already documented in `image-controls-20261014.md`. Its stale copy expectations
were updated, but its control-mount assertion and behavior were not changed.
This is not a claim of a clean whole-app test suite.

Personally inspected the first 28-image native offscreen batch across 14 scenes
at **1280×800** and **820×600**. Fixed the identified singular counts and missing
sentence separation in one final correction batch. Regenerated all 28 frames and
personally inspected 16 targeted confirmation images covering baseline success,
capture gaps, page-list options, History, export, import, deletion and recovery
previews. The corrected summaries wrap readably, and the baseline message and
confirmation actions remain usable at both sizes. All final PNG hashes and six
current source bindings per frame were verified.

Synthetic captures, per-frame provenance and confirmation sheets are retained at:

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-copy-confirm-h46267yv
/var/home/olav/Data/opencode/tmp/opencode/regressee-copy-final-hl1ycw0a
```

Reproduction uses the existing private-root/calibration recipe and
`REGRESSEE_RENDERER_COPY=1` with the opt-in
`ui_renderer_capture::ui_review_matrix::renderer_capture_whole_app` test.
No user screenshots, logins or database were used.

**Bounded verdict: PASS for the wording and sampled Linux offscreen layouts.**
This is not OS-input, assistive-technology, cross-platform or release
qualification. Restart the rebuilt `target/debug/regressee` to load the changes.
