# Logo and palette integration — 2026-10-14

User-confirmed scope: incorporate the supplied background-specific marks beside
**Regressee**, retain the existing interface structure, and adapt to navy
navigation, readable teal primary actions, mint selections and white workspaces.
The user also authorized moving originals to top-level `assets/`.

## Implementation

- Untouched originals: `assets/logo.png` and `assets/logo-dark.png`.
- Transparent UI-ready derivatives: `assets/brand/`, with embedded origin text,
  hashes and exact transforms in `provenance.json`. Regenerate with
  `python3 scripts/prepare-brand-assets.py` (developer-only Pillow dependency).
- `crates/app/src/ui/brand.rs` embeds both PNGs, retains stable cached image
  identities and lets GPUI decode off-thread. No runtime asset files or requests.
- Dark-background icon + name in the rail; light-background icon + About title
  on white. Existing dialog focus/close behavior is unchanged.
- Shared `theme.rs` roles propagate the palette through actions, navigation,
  fields, choices, queues and progress. Warning/error roles remain distinct;
  screenshot originals are untouched. White-on-teal contrast is 5.29:1.
- `PRODUCT.md`, `DESIGN.md` and the Impeccable contract/sidecar record the approved
  adaptation. Older composition references no longer govern the palette.

## Bounded verification

Native build passed:
`cargo build --locked -p regressee-app --features ui --bin regressee`.

Library UI tests: **110 passed, 2 ignored**:
`cargo test --locked -p regressee-app --features ui-tests --lib`.
Includes shared contrast/state-geometry checks and embedded-logo tests.

Full shell suite: **160 passed, 10 failed, 32 ignored** at the recorded run.
Failures concern workflow/copy assertions outside the logo/palette edits; they
were not repaired in this task. No whole-suite PASS is claimed.

The new opt-in test
`ui_renderer_capture::ui_brand_capture::renderer_capture_branding` passed both
inspection and confirmation. Ten real GPUI/Blade offscreen captures cover
Projects, Website, Access, About and Review at 1280×800 and 820×600. Assertions
check decoded assets, visible lockup bounds, rail fit and unchanged saved evidence.
Every confirmation image is pixel-identical to its inspected counterpart; final
sidecars bind the current theme, branding source, assets and harness by SHA-256.

Private evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-brand-20261014-dvt066cs/`
(separate `inspection/` and `confirmation/` batches).
Development-host commands used `LIBRARY_PATH=$PWD/.runtime/link-libs` and the
approved private temporary root. The first library invocation lacked that linker
path and failed to link; the corrected invocation passed.

Independent finish review found no material visual defects, but returned **fix**
for missing design-system persistence. The independent documenter updated the
design records; the scoped verdict scored that single fix **resolved** with
disposition **ship**. The two-size confirmation completed without pixel changes.

Evidence uses synthetic offline projects. It is not real-site, OS-input,
screen-reader, Windows/macOS or release qualification. No web detector ran:
the implementation is native Rust/GPUI. Unrelated existing work was preserved.

## Navigation lockup follow-up — 2026-10-14

The user found the horizontal navigation mark too small and requested a larger
mark centered above **Regressee** on its own line. This follow-up changes only
the rail identity layout; the palette, assets, embedding and provenance remain
unchanged, as does the compact horizontal About heading.

### Source-derived metrics

- `crates/app/src/ui/brand.rs::navigation_lockup()`: dark-background mark
  (96px wide, proportional height approximately 58.8px), matching image slot,
  centered above the centered name, with an 8px vertical gap.
- Navigation wordmark: installed system UI type, 20px size, 26px line height,
  semibold (600). `DESIGN.md` records the navigation-only `brand-wordmark` role;
  form `section` metrics remain 18px/24px, semibold.
- `crates/app/src/workflow.rs`: title wrapper padding is 12px above and below
  inside the existing 192px rail; rail width is unchanged.
- About: unchanged horizontal light-background mark (36px wide, approximately
  22px painted height in a 24px slot), 8px gap and 21px semibold title.
- `.impeccable/design.json` records navigation and About geometry separately in
  `extensions.nativeBrand`; no new asset derivation or palette update occurred.

### Verification status — deferred full-app capture

Independent source review found no material flaw in the sampled stacked lockup.
Its disposition is **recapture**, because no current full-app pixels exist; the
user explicitly deferred this gate and requested that App settings stay untouched.

Reported shared UI checks: **49 passed**, including logical GPUI component
bounds for the larger centered mark/name at 1280×800 and 820×600. These are
component checks, not proof that the complete navigation stack fits or that all
its controls remain reachable.

The recorded full-app build/capture attempt was blocked by an unrelated
`crates/app/src/app_settings_ux.rs:89` `AnyElement.w_full` compile error. The user
explicitly chose **leave it untouched** and defer full-app verification. That
file was not repaired as part of this documentation follow-up.

`crates/app/src/ui_brand_capture.rs` now includes guards for the larger stack in
normal and read-only review, including navigation-button bounds and reachability
with multiline saved-results context at both window sizes. Execution and capture
verification remain deferred until the build is unblocked; this is not a passed
whole-navigation guard.

**No new screenshot exists for the stacked lockup.** The ten inspection and
confirmation captures and scoped finish verdict recorded above concern the old
horizontal version only. They remain historical evidence, not new visual
evidence. No full-app navigation-fit, full visual PASS, native-approval or release
claim is made for this follow-up; all older qualification history is preserved.

## Task workspace carry-through — 2026-10-15

The separate task rail still rendered a text-only title. It now reuses
`brand::navigation_lockup()` for discovery, screenshot capture, checks and browser
setup; rail width, task content, actions and behavior are unchanged.

Verification passed: 11 task UI tests, 2 shared-brand tests and the native app
build. The expanded branding capture test passed 20 synthetic Linux GPUI/Blade
offscreen scenes at 1280×800 and 820×600, including all four task kinds and normal
and read-only navigation. Inspection of capture at both sizes, narrow browser
setup and narrow read-only review found the logo/name visible and controls clear.
Scoped disposition: **ship**. These captures also resolve the previously deferred
stacked-logo layout check, not native input/accessibility or release qualification.
No asset, palette or unrelated settings changes were needed.

Private evidence:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-task-brand-20261015-iicz8n8g/inspection/`.
