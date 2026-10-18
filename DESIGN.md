---
name: Regressee — Clear wayfinding
description: User-approved direction, source-derived native Rust/GPUI system; native and release qualification pending
colors:
  ground: "#f4f4f4"
  surface: "#ffffff"
  rail: "#eaeaea"
  rail-ink: "#242424"
  rail-muted: "#595959"
  rail-hover: "#dedede"
  nav-selected-ink: "#00685f"
  ink: "#242424"
  muted: "#595959"
  line: "#d6d6d6"
  control-line: "#858585"
  action-surface: "#f8f8f8"
  action-hover: "#e8e8e8"
  accent: "#007970"
  accent-hover: "#00685f"
  on-accent: "#ffffff"
  selected: "#e2f5f0"
  error: "#9c3038"
  error-surface: "#fff2f2"
  warning: "#775311"
  warning-surface: "#fff7df"
  shadow: "#181818"
  dark-ground: "#1b1b1b"
  dark-surface: "#232323"
  dark-rail: "#202020"
  dark-rail-ink: "#ededed"
  dark-rail-muted: "#b8b8b8"
  dark-rail-hover: "#303030"
  dark-nav-selected-ink: "#bceee4"
  dark-ink: "#ededed"
  dark-muted: "#b8b8b8"
  dark-line: "#3d3d3d"
  dark-control-line: "#808080"
  dark-action-surface: "#2b2b2b"
  dark-action-hover: "#333333"
  dark-accent: "#65d9bf"
  dark-accent-hover: "#85e7d1"
  dark-on-accent: "#072c29"
  dark-selected: "#203e38"
  dark-error: "#ffacb2"
  dark-error-surface: "#40282f"
  dark-warning: "#f1ca7d"
  dark-warning-surface: "#382d1b"
  dark-shadow: "#080808"
typography:
  headline:
    fontFamily: "Segoe UI, .AppleSystemUIFont, DejaVu Sans"
    fontSize: "28px"
    fontWeight: 600
    lineHeight: "36px"
  review-heading:
    fontSize: "24px"
    fontWeight: 600
    lineHeight: "32px"
  title:
    fontSize: "21px"
    fontWeight: 600
    lineHeight: "28px"
  brand-wordmark:
    fontSize: "20px"
    fontWeight: 600
    lineHeight: "26px"
  section:
    fontSize: "18px"
    fontWeight: 600
    lineHeight: "24px"
  body:
    fontSize: "15px"
    fontWeight: 400
    lineHeight: "22px"
  action:
    fontSize: "14px"
    fontWeight: 500
    lineHeight: "20px"
  help:
    fontSize: "13px"
    fontWeight: 400
    lineHeight: "18px"
rounded:
  status: "3px"
  control: "4px"
spacing:
  tight: "4px"
  related: "8px"
  row: "12px"
  group: "16px"
  section: "24px"
components:
  button-primary:
    backgroundColor: "{colors.accent}"
    textColor: "{colors.on-accent}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    padding: "8px 12px"
  button-primary-hover:
    backgroundColor: "{colors.accent-hover}"
    textColor: "{colors.on-accent}"
  button-secondary:
    backgroundColor: "{colors.action-surface}"
    textColor: "{colors.ink}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    padding: "8px 12px"
  button-danger:
    backgroundColor: "{colors.error-surface}"
    textColor: "{colors.error}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    padding: "8px 12px"
  navigation:
    backgroundColor: "{colors.rail}"
    textColor: "{colors.rail-ink}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    padding: "8px 12px"
  navigation-selected:
    backgroundColor: "{colors.selected}"
    textColor: "{colors.nav-selected-ink}"
    rounded: "{rounded.control}"
  field:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    padding: "0px 12px"
    height: "40px"
  choice-selected:
    backgroundColor: "{colors.selected}"
    textColor: "{colors.accent-hover}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    padding: "{spacing.row}"
  toolbar-icon:
    backgroundColor: "transparent"
    textColor: "{colors.ink}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    padding: "0px 8px"
    height: "40px"
    width: "40px"
  toolbar-icon-selected:
    backgroundColor: "{colors.selected}"
    textColor: "{colors.accent-hover}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    padding: "0px 8px"
    height: "40px"
    width: "40px"
---

# Design System: Regressee — Clear wayfinding

## Overview

**Creative North Star: "Clear wayfinding"**

Make the next safe action obvious. Unified neutral navigation and working
surfaces form one precisely aligned environment for non-developers reviewing
website releases, with teal/mint reserved for action and selection.
The signature is self-labelling task state with baseline consequences attached,
not a developer dashboard, decorative charts or a mobile-app shell.

The user approved this direction and the full workflow brief on 2026-10-11.
This document records the built source system, not approval of its usability,
native accessibility or public release. The chosen HTML sketch remains an
illustrative composition reference, not authority for the subsequent palette
refinements or appearance extension; production GPUI source supplies the
normative tokens above. The extensions-only sidecar carries native behavior and
geometry, not a browser implementation.

**Historical logo and palette adaptation — 2026-10-14.** The user approved navy navigation,
readable teal actions, pale mint selection and white workspaces, with supplied
background-appropriate icons alongside the readable Regressee name. This narrow
extension preserves structure, workflows, system UI fonts, error/warning roles,
focus and immediate state changes; screenshot originals remain untinted. At that
date the approval did not introduce a full dark workspace theme.

**Historical desktop appearance extension — 2026-10-16.** The user approved app-wide
System / Light / Dark, extending Clear wayfinding (seed `6e22d8c4`) rather than
redesigning it. This superseded the historical white-only scope above. At that
date, light retained its palette and dark added layered blue-charcoal surfaces,
soft light ink, mint actions/selection and distinct semantic states. The palette
is superseded below; the System / Light / Dark behavior remains current.
Installed system type, geometry, focus, depth recipes and immediate guarded
behavior are inherited.
App appearance never changes website capture-theme options or screenshot pixels;
no dependency, animation, shipping raster or unrelated visual change is added.

**Unified neutral themes — 2026-10-17.** The user rejected the actual native
app's slate workspace against its blue sidebar and the gradient header, and
selected Unified neutrals. Dark uses charcoal layers and matching neutral ink;
light uses off-white/soft-gray equivalents. Navigation and workspace differ by
lightness, not competing hue families. Teal/mint action and selection, semantic
warning/error roles and restrained directional depth remain; header and
inspector/disclosure chrome is solid `surface`, never a gradient. Both rail and
About select the existing background-appropriate logo. This local Operate-mode
refinement inherits Clear wayfinding's seed `6e22d8c4`, type, geometry, behavior,
drafts and consent, with no new identity, dependency, motion or shipping raster.
It supersedes earlier navy-rail, blue-charcoal and gradient-chrome guidance,
not its dated verification records or unchanged appearance-preference behavior.
Both palettes and foreground/shadow aliases are normative in the frontmatter;
the native sidecar maps each semantic role to its light or `dark-` sibling.

**Key Characteristics:**
- Persistent project context and a clear next safe action.
- Real coverage beside a searchable queue and untinted comparison evidence.
- Deliberate baseline decisions, protected drafts and retained history.

Authority: `PRODUCT.md`, `docs/workflow-revamp.md`,
`.impeccable/surfaces/crates-app-src-main-rs.md` and the selected
`.impeccable/mocks/decision/pick-6e22d8c4.html`. Implementation sources:
`crates/gui/src/ui/{brand,theme,controls,toolbar,text_input,timezone_select,zoom_select}.rs`,
`crates/gui/src/{main,appearance_ux,action_icons,evidence_workspace,review_ux,workflow,form_recovery,workspace_history}.rs`,
`crates/app/src/appearance.rs`,
`crates/core/src/appearance.rs`, `crates/storage/src/preferences.rs` and
`crates/core/src/shell.rs`. Asset authority: `assets/README.md` and
`assets/brand/provenance.json`. Earlier direction remains in `docs/phase0-design.md`.

Qualification boundary: Phases 4, 5 and 6 independently passed their bounded
source/controller/offscreen gates. Phase 7 qualification infrastructure and its
finite consent/long-label repairs are built; independent verification is pending.
Historical FIX/blocked evidence remains preserved; none of these results is full
native UI or release acceptance. The running Linux Wayland process did not
register with the available AT-SPI registry; accessibility integration is a named
blocker, not a screen-reader pass. See
`docs/research/whole-app-ui-review-20261012/phase7-platform-qualification.md`.
Offscreen Linux GPUI pixels establish only development-host rendering;
they do not supersede earlier black OS-window captures or establish real input,
IME, screen-reader, high-contrast, Windows/macOS or release qualification.

The 2026-10-13 UX refinement plan is a separate iteration: all five phases received
independent bounded PASS verdicts, completed on 2026-10-14. Final interactive-state
integration and its independent verifier report are documented under
`docs/research/ux-review-20261013/`. The behavior descriptions below include those
shipped refinements without changing the qualification boundary.

**Logo/palette finish record — 2026-10-14.** Independent finish review found no
visual defects across ten Linux offscreen captures of Projects, Website, Access,
About and Review at 1280×800 and 820×600. Its FIX disposition concerned only the
missing design-system persistence updates recorded here.
The subsequent scoped verdict scored that persistence fix **resolved** with
disposition **ship**; all ten confirmation captures were pixel-identical to the
inspected originals. This verdict covers the branding/documentation adaptation,
not the whole application or its outstanding shell assertions. Artifact source:
`crates/app/src/main.rs`; captures:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-brand-20261014-dvt066cs/inspection/brand-{1280x800,820x600}-{projects,website,access,about,review}.png`.
These are actual GPUI pixels from synthetic offline states, not real project
results or OS-input/accessibility/cross-platform/release qualification. Reported
library UI checks: 110 passed, 2 ignored; shell checks: 160 passed, 10 failed,
32 ignored, with unrelated workflow/copy assertions failing. No whole-suite PASS
is claimed; older qualification records remain unchanged.

This finish record is historical evidence for the earlier horizontal navigation
lockup; its captures are not visual evidence for the stacked follow-up below.

**Navigation lockup follow-up — 2026-10-14; capture verification deferred.**
The user found the horizontal rail mark too small and requested a larger mark
centered above the Regressee name on its own line. Source now records that
two-line identity; About, the palette, assets, embedding and provenance are
unchanged. Reported shared UI checks: **49 passed**, including logical GPUI
component bounds at 1280×800 and 820×600. These component checks do not establish
full-app navigation fit or control reachability.
The recorded full-app build/capture attempt was blocked by the unrelated
`crates/app/src/app_settings_ux.rs:89` `AnyElement.w_full` compile error. The user
explicitly chose to leave it untouched and defer full-app verification.
No new screenshot exists for this lockup. The extended
`crates/app/src/ui_brand_capture.rs` guard awaits an unblocked build: check the
larger stack in normal and read-only review, including navigation-button bounds
and reachability with multiline saved-results context at both window sizes.
Capture verification and that whole-navigation guard remain deferred, not a
full visual PASS or native approval.

**Appearance finish status — 2026-10-16.** Following the initial **FIX**,
independent `finish-verdict.md` scores **both findings resolved**, disposition
**SHIP for those two fixes only**: About identity and design persistence, not
the whole surface. The complete second native GPUI/Blade capture passed
`capture-verdict.log`; all 48 frames remain at their original `confirmation/`
paths, with the actual background-appropriate logos visible in all six About
captures. Frame hashes/dimensions/opacity and all 180 source hashes were verified;
current `source-receipt.json` binds this final capture. The only source change
since the first complete round is the test capture's image-readiness wait/assert,
not shipping Rust UI or a presumed production-logo repair. First-round images,
sheets and receipt remain in `review-round-1/`, `boards-round-1/` and
`source-receipt-round-1.json`. Renewed `untinted-evidence.json` confirms identical
original-image strips at all three sizes, not every original pixel.
Current reported tests: core
56 passed; storage 40 passed/2 ignored; app library 140 passed/2 ignored; shell
223 passed/**5 failed**/59 ignored. The broader filtering, loading bounds,
project-action bounds, minimum queue bounds and disclosure-enablement failures
have no A/B attribution; no whole-suite pass or pre-existing-failure claim is
made. Production build succeeds. No OS-input/live-desktop-toggle, accessibility,
cross-platform, usability or release acceptance is established. Detailed evidence:
`docs/research/dark-mode-20261016.md`.

**Unified-neutral finish record — 2026-10-17; scoped SHIP.** The earlier finish
scores qualify only their recorded work, not this user-rejected palette or the
new refinement. `build-final.log` records a successful production build;
`tests-confirmation.log` records app library **145 passed / 0 failed / 2 ignored**
and shell **226 passed / 5 failed / 61 ignored**. Neutral-shell, solid-chrome,
both-palette contrast, appearance live/override/focus and settings keyboard checks
pass. The five broader failures match the prior categories but lack A/B attribution;
they are not labelled pre-existing, and the whole suite is not a pass. Intermediate
compile/startup-timing failures were corrected only in tests, not shipping chrome
or preference behavior.

The final `capture-final.log` passed: **48 current-source GPUI/Blade frames** in
`confirmation/`, eight states in both themes at 1440×900, 1280×800 and 820×600,
using copied synthetic website fixtures with provenance. The first `inspection/`
round is superseded because shared Rust sources changed during verification.
`validate_frames.py` passed PNG integrity and all 13 embedded critical-source
hashes per frame; `source-receipt.json` binds current Rust sources. Header samples
at Y5/Y45 match in each size/theme, and original-image 100px strips match between
light/dark at all three sizes: sampled proof, not every pixel.

Fresh `finish-review.md` disposition is **SHIP for the neutral palette/chrome
refinement only**. The reviewer inspected all 48 final captures and all six current
boards, confirmed actual background-appropriate logos, and found no material
palette findings. No source-theme/header/layout change followed this capture.
This does not clear the five shell failures or establish whole-app, actual OS
input/accessibility/live OS-switch, Windows/macOS, usability or release approval.
Current evidence root, exact failures and qualification limits:
`docs/research/theme-coherence-20261017.md`.

## Colors

### Primary
Teal `accent` marks the next task action; the rail itself is neutral.
`accent-hover` changes action hover/press without changing geometry. `selected`
is pale mint in light and restrained dark mint in dark; it identifies the current
navigation, choice or page, never successful evidence by itself. Rail text uses
`rail-ink` and `rail-muted`; unselected navigation hover uses `rail-hover`.
Always use `on-accent` for filled-action text/icons and checkbox ticks, and
`nav-selected-ink` for selected navigation—not `surface` or `rail` aliases.
Light actions use darker readable teal than the luminous artwork; dark actions
use bright mint with deep teal foreground.

### Neutral
`ground`, `surface`, `rail` and `action-surface` separate workspace, editable
fields, navigation and secondary actions by lightness within one achromatic
family. Shell surfaces, hover, ink, lines and shadow bases have equal RGB
channels in each theme. `rail-ink` equals `ink`, and `rail-muted` equals `muted`;
retain the semantic aliases. `action-hover` and `rail-hover` are neutral, not
selection tints. `line` is a separator, not an interactive boundary;
`control-line` outlines controls. Source contrast requirements remain (3:1)
against field and secondary-action surfaces, not native certification.

Unprefixed frontmatter colors are the exact `ui/theme.rs::LIGHT` palette;
`dark-` siblings are the exact `DARK` palette. Component references name the
light roles; `.impeccable/design.json` `extensions.nativeAppearance.roleMapping`
records native palette resolution for both themes. `shadow` is independent of
content `ink`. Confirmed source tests pass body/semantic text (4.5:1) and
control/focus-boundary (3:1) requirements in both palettes. These checks are not
visual approval, do not establish native accessibility and do not override the
user's feedback.

Error and warning ink/surface pairs always accompany explanatory words.
Informational feedback uses neutral `surface`, distinct from selected navigation;
information status labels may use the mint tint.

**The Evidence Rule.** A change classification is never rendered as a confirmed defect, and missing screenshots are never rendered as an unchanged result.

## Typography

Use installed system UI type, not bundled brand fonts. The frontmatter family
list is an OS inventory, not a CSS fallback stack: source requests Segoe UI on
Windows, .AppleSystemUIFont on macOS and DejaVu Sans on Linux. Qualify font
resolution and host-native affordances separately on each desktop platform.

Task headings use `headline`; bounded review uses `review-heading`. `title`
serves empty states/information dialogs, and `section` groups forms. Body copy is
regular; actions, editable fields and page addresses share the medium `action`
metrics. Help and page details use `help`. Keep long form explanations bounded
(660–680 logical px). No decorative display face or technical monospace hierarchy.
The navigation wordmark uses the dedicated `brand-wordmark` role; `section`
remains unchanged for form groups. About retains the existing `title` role.
Both lockups use semibold system type; branding adds no font family.

**Explicit leading — 2026-10-17.** Pair font size and line height at every local
size override; do not inherit GPUI's 1.618 default or an ancestor's absolute
heading/control leading. The body root is 15/22, help and wrapped rail status
13/18, compact labels 14/20, titles 21/28, review headings 24/32 (21/28 when
compact), and task headings 28/36. Field labels own their body metrics and hints
own regular help metrics. Preserve intentional compact inspection treatments,
40px action targets and the multiline editor's 14/20 shaping/caret geometry.
Font families, palette, copy and operations are unchanged. Scoped evidence and
qualification limits: `docs/research/typography-leading-20261017.md`.

**The Stable State Rule.** Focus, hover, press and selection change paint, never text metrics or control geometry.

Screenshot switches keep the selected path/profile heading and inspection toolbar
mounted during the matching image read; zoom stays disabled until pixels arrive.

## Layout

This is a resizable native desktop application. Default window (1280×800 logical
px), minimum (820×600); fixed project rail (192px), working horizontal inset
(24px). Setup forms cap at (680px), with an explicit pinned save footer.
The pinned shared header names **Selected project** and offers **Change project**.
It has no permanent workspace-type badge: all projects are local. Only actual
unsaved project settings add a status. Its flexible project-context text has a
minimum width (180px), avoiding an extra
narrow-header row with icon-labelled actions while retaining project identity.
Results and History offer an in-place chooser when no project is selected.
Changing projects preserves the destination and confirms draft discard; reopening
the same project must not reload away its draft. Projects separates **View results**
from **Edit settings** and identifies the selected project with words and paint.
The row's **Selected** status follows the project name with an 8px gap, separate
from its trailing action group. Native text measurement keeps short names at their
natural width; long names ellipsize without shrinking the status. The existing
mint row selection, URL and actions are unchanged.

**Copy restraint — 2026-10-16:** fixed product facts and restatements of visible
controls are clutter, not status. App settings omits its section-summary subtitle,
the healthy browser's repeated readiness explanation and the idle network-loaded/
save instruction. Storage keeps the History destination, not a local-only claim.
Baseline selection omits the persistent confirmation disclaimer. Restricted-browser
warnings, failure/loading states, selected counts, eligibility explanations and
protected confirmation remain. The focused header regression and eight synthetic
GPUI/Blade frames passed at 1280×800 and 820×600 (App settings, saved/unsaved Project
settings and comparison review); headers fit one row. Evidence:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-quiet-copy-p080b4vh/inspection/`.
Offscreen layout only, not native input, accessibility or release qualification.
No new token, asset, backend operation or consent rule.
Production UI build and 14 focused shell tests passed (one opt-in capture ignored).
The browser-check test now paints the resized/current state before traversing focus,
rather than using stale layout bounds for successive Tab actions. The broader shell
run timed out at 120 seconds with a page-search disclosure failure still unresolved;
no whole-suite pass is claimed.

**Project-status placement — 2026-10-16:** scoped UI build and four sidebar/project
tests passed. Six synthetic GPUI/Blade confirmation frames cover short/long names
and a second unselected project at 1440×900, 1280×800 and 820×600, under
`/var/home/olav/Data/opencode/tmp/opencode/regressee-project-status-crocnp3u/confirmation/`.
This is offscreen layout verification, not native-input or release qualification;
no palette, asset or backend change.

Comparison review remains side by side, including small desktop windows: queue width (200px)
below viewport width (1100px), otherwise (272px), separated from the flexible
viewer by (12px). Do not describe a stacking layout that is not implemented.
The logo/name lockup is always the first item in the project rail, at the same
top position across sections. Its navigation-only title wrapper has padding
above and below (12px each) inside the existing rail; the mark and name are
centered on separate rows. The earlier deferred stack verification remains
recorded above; unified-neutral capture and scoped review are complete, not
whole-navigation qualification. The review
header owns one icon-only sidebar
control in a consistent location, named **Hide sidebar / Show sidebar** on hover
and keyboard focus. This control is absent from project choosers, where navigation
stays visible. Review also owns an icon-only **Hide results / Show results** toggle.
At narrow sizes the project navigation starts collapsed;
an explicit workspace preference survives resize. Selected project context remains
visible. Working geometry uses the actual mounted rail width, not a permanent
192px reservation. Ctrl/Cmd+F restores a hidden queue before focusing its search.
Compare remains the default at both sizes. **Single image (full width)** is an
explicit, untinted one-original alternative. Its toolbar pairs **Single image**
(**Single** below 720px working width) with persistent labelled **Before** and
**After** choices; the selected choice has a mint surface and teal boundary.
Both destinations remain visible, rather than hiding side switching behind a
swap icon. Choosing the current side is a no-op; shared source pan/zoom, manual
scale, target and drafts survive either choice. A separate
View chooser exposes Compare, Difference, Overlay, Heatmap, this alternative and
the optional **Split slider**. Split slider clips untinted Before-left and
After-right originals in one full-width stage with shared source pan/zoom; dragging
the vertical divider changes only the reveal. Tab from Viewer reaches the divider:
Left/Right moves it by 1%, Shift+Left/Right by 10%; Home shows all After and End
all Before. Missing pairs disable Split slider and fall back to Compare without
clearing the preferred mode. A shorter page retains an opaque blank tail and image
navigation. These view changes never save or discard drafts or update a baseline.
Queue and image panes are independently bounded. Details and comparison
adjustments use a bounded, scrollable flyout over the viewer; neither replaces
the queue nor consumes image height below the evidence. Review does not
use the ordinary full-screen scrolling form. Project context, coverage and the
conditional approval bar remain attached to the task.
Without a comparison, show one untinted capture pane, not an empty before/after
pair. Distinguish incomplete captures, captures ready for explicit baseline save,
and saved baseline screenshots. Capture coverage reports captured/not captured/
pending targets, not changed/unchanged or analysis awaiting a nonexistent check.
Keep the save requirement and gap-recovery instructions visible above the queue;
screenshots are never automatically activated as a baseline.

Ordinary review keeps the three classifications together, with Review note nearby
and Next unreviewed trailing. Below actual working width (720px), classifications
and note/next form two complete action rows separated by (8px), rather than losing
labels or shrinking targets. At the minimum window with navigation expanded this
deliberately costs (48px) of evidence height; do not claim unchanged evidence height
for that state. The user's navigation preference remains authoritative. This
threshold uses available workspace width after the mounted rail, not window width.

Discovery and captured-target lists use virtualized, full-width rows (64px) with
aligned horizontal insets (16px). Address and details truncate independently;
selection and hover preserve row height. Lists fill their bounded viewport, not
a fixed five-row panel. Zero search matches show an explanation, not an empty
bordered list. Capture profiles describe mobile **web emulation**, tablet and
desktop websites—not app devices or a mobile product layout.
Discovery selection opens **Page options** immediately, with the complete wrapped
address, capture status and explicit Include/Exclude action. **Previous page** and
**Next page** inspect these details and reveal the selected row in the list; they
never change inclusion themselves. Rows use selection and pointer feedback, not
hover help. Discovery-blocked pages retain their reason and explain the
settings/rediscovery route, without enabling an unsafe inclusion override.
Inspection result rows are now 84px, with path/query/fragment first and separate
profile and status lines. Result rows have no redundant address tooltip obscuring
the screenshots; the full URL remains in the selected screenshot's
keyboard-accessible Details. Discovery and recorded-page lists expose complete
addresses in Page options / Page details, with no hover overlays. Discovery and
History keep their previous row geometry. Authoritative empty captures reserve
no blank queue.
Automatic capture-only overview fit is capped at 100%; manual enlargement remains
available. Compact Zoom keeps **Fit** or the fixed percentage visible.

**Destructive-action consent — 2026-10-14.** Page options offers **Discard
discovered pages…**, not the ambiguous Dismiss page list. Its warning names the
unsaved URL list and inclusion choices that will be lost, and the recorded
screenshots/baseline that will be kept. Keep page list is the safe first choice;
Close/Escape only closes the panel. After confirmed discard, recorded URLs still
open read-only Page details with Previous/Next. **Use recorded pages** creates a
new editable capture plan from the displayed run, even without a saved baseline,
without visiting websites or weakening discovery exclusions. Settings saves keep
the plan; changed settings disable capture until explicit rediscovery, whose
replacement is also confirmed. Project switching/new-project entry and quitting
warn before losing discovery choices; an otherwise clean quit offers Keep page
list versus Discard pages and quit rather than pretending that the list can be
saved as settings.

Forget login, retained-baseline activation, permanent history deletion,
interrupted-file cleanup, recovery-original deletion and private-browser
repair/update now require warning popups. Existing whole-baseline save,
selective-approval, unsaved-edit and save/discard-on-quit dialogs remain intact.
Cancel dispatches no mutation; consent is checked against its preview/context
before dispatch, with backend revision/reference protections retained. This is
simulated controller and Linux offscreen-layout evidence, not OS-input or native
accessibility qualification; see `docs/research/page-list-recovery-20261014.md`.

### Shared dialogs — 2026-10-15

All in-app confirmations, information dialogs, selective baseline approvals and
task dialogs share `ui::dialog`'s palette `surface`, control boundary, 4px corners,
16px window clearance and a `shadow`-colored scrim (alpha 102/255). Content uses 24px insets, 16px
section gaps, a 21px/28px semibold heading and 15px/22px body copy. Actions use
the existing 40px-minimum shared controls, with a ruled, pinned footer. Task
workspaces retain their larger page headings; their modal variants use dialog
heading metrics. Information dialogs cap at 520px wide, confirmations at 560px;
task and approval dialogs retain their task-appropriate bounded dimensions.
Keyboard help fits its 360px reading body at the supported minimum window.

GPUI's unstyled fallback is replaced with the app-owned prompt renderer on all
desktop targets; OS file pickers remain native. Confirmation call sites declare
Cancel (secondary), Ok (teal primary) or destructive warning actions (error ink
and surface), rather than guessing roles from labels. Cancel/Keep reviewing
receives initial focus, including selective approval. Tab/Shift-Tab wrap in
painted order, Enter/Space activate only the focused choice, Escape selects the
safe dismissal and the original trigger regains focus. Backdrop clicks do not
confirm or dismiss. Three-way quit places Discard, Cancel, Save in that order
without changing the returned answer indices or existing save/exit guards.
Long confirmation prose scrolls independently with an overflow-only interactive
scrollbar, wheel and keyboard scrolling; actions stay visible. Existing consent
copy, snapshot checks and background mutation boundaries are preserved.

Scoped verification and qualification limits:
`docs/research/shared-dialogs-20261015.md`. No new palette tokens, decorative
assets or shipping rasters were introduced.

## Elevation & Depth

**App-wide depth refinement — 2026-10-16.** The user requested subtle shadows
and depth, referenced Xcode, and selected app-wide consistency. This supersedes
the former flat-only treatment, not Clear wayfinding's layout or workflow.
The 2026-10-17 unified-neutral refinement supersedes its cool ground and
gradient chrome, not the historical review or directional shadow geometry.
Neutral `ground` separates opaque working surfaces from the workspace in both
themes. Shared native recipes in `ui/depth.rs` establish restrained hierarchy:

| Role | Shadow: x / y / blur / spread / shadow alpha | Use |
| --- | --- | --- |
| Control | 0 / 1 / 2 / 0 / 24 | Enabled labelled actions and Zoom; selected navigation |
| Surface | 0 / 2 / 6 / 0 / 20; 0 / 1 / 2 / 0 / 12 | Screenshot stages and continuous page/result lists |
| Floating | 0 / 8 / 24 / 0 / 38; 0 / 2 / 6 / 0 / 24 | Menus, hints, inspectors and option panels |
| Dialog | 0 / 18 / 48 / 0 / 56; 0 / 4 / 12 / 0 / 36 | Shared information, task and protected confirmation surfaces |
| Header / Footer | 0 / ±2 / 6 / 0 / 16 | Persistent shell header and pinned setup/baseline footer |
| Rail | 2 / 0 / 6 / 0 / 28 | Navigation-to-workspace boundary |

Distances are logical pixels; alpha is out of 255 and all shadows use `shadow`
(historically equal to light `ink`, now a separate neutral theme role). The header
and disclosure/inspector titlebars use solid theme `surface`, not a vertical
gradient, glass or blurred content. Layer lightness, thin rules and the existing
edge shadows supply hierarchy. The earlier highlight treatment is historical
only. Keep editable fields on `surface` and grounded. Individual list rows, form choices and transparent
toolbar tools do not cast shadows. Disabled actions lose their lift. Geometry,
40px targets, type, semantic colors, selection and mounted-control rules remain.

GPUI's zero-blur spread paint remains a functional focus outline, not elevation;
it replaces ordinary control elevation and hover never overwrites the ring.
The transparent screenshot viewer is an exception: reserve a stable 1px border
and change only its colour to ink on focus. Never put a filled focus shadow behind
its captions, pane gaps or original screenshot pixels. Pointer and keyboard use
the same boundary without changing text colours or layout. Elevate only each
opaque screenshot frame; do not tint, round, blur or resample the source pixels.
Shared dialogs retain their scrim, occlusion, focus lifecycle and explicit consent.
File pickers remain platform-owned. No decorative assets or new motion are added.
Scoped evidence and qualification limits belong to
`docs/research/subtle-depth-20261016.md` for the historical depth pass, and
`docs/research/theme-coherence-20261017.md` for the scoped SHIP neutral/solid-chrome
refinement.

No invented motion scale: state changes are immediate. Unknown-duration activity
uses a seamless 1.2-second linear pattern for long tasks and a rotating native
arc for compact reads, driven by native animation frames only while visible,
independently of worker updates. This communicates activity, not completion.
`REGRESSEE_REDUCED_MOTION=1` keeps both indicators static; automatic OS
motion-preference integration and high-contrast behavior remain
qualification work.

## Shapes

Use small control corners and straight horizontal list rules; status labels are
slightly tighter. Dialogs share control corners. Draw radio dots, checkbox ticks
and the timezone chevron with native geometry. No nested decorative card stacks
or icon-tile dashboard. Shipping branding now consists of two transparent,
cropped/downsampled logo PNGs in `assets/brand/`, embedded in the native binary.
The user-consented move kept the supplied originals untouched in `assets/`;
`assets/brand/provenance.json` records source/output hashes and exact transforms.
CAS screenshots are untinted product evidence with provenance—not decorative
rasters, a fixed image library or synthetic demonstration results.

**Local exploration.** Discovery, review and loaded History records filter as
text changes, with truthful match counts/scope and Clear. Ctrl/Cmd+F focuses the
mounted search without escaping a dialog. Filtering never silently changes the
inspected screenshot or discards ignored-area drafts; an outside-filter notice
explains why the pair is still shown. Explicit navigation changes the target.
Viewer reads coalesce independently of mutation work and reject stale identities.
Discovery plans survive initial saved-evidence reads and Refresh, including empty
projects and older captures; only a non-representative capture of the exact reviewed
inventory consumes the plan. Pages & baseline shows a next-action empty state when
neither a discovery plan nor a recorded page list exists, never a blank workspace.
Historical environment context travels with its reference, not today's settings.
History rows support direct selection; storage totals load separately and missing
measurement is not zero. Local syntax/range feedback stays beside touched fields,
while saving, browser visits, baseline changes, exports and deletion stay explicit.

## Components

### Product identity — background-appropriate icon and readable name

The noninteractive navigation and **About Regressee** lockups both select the
active theme's existing asset through `Background::workspace()`:
`assets/brand/logo-light.png` on the light soft-gray rail or light `surface`,
and `assets/brand/logo-dark.png` on the dark charcoal rail or dark `surface`.
This changes background selection, not the artwork or lockup geometry. The mark
supplements, never replaces, the name. Navigation uses
`navigation_lockup()`: a larger proportional mark (96px wide, approximately
58.8px high) with matching height reservation, centered above the name with the
related gap (8px) and `brand-wordmark` system type in `rail-ink`. About keeps the
horizontal `lockup()`, logo width (36px), proportional painted height
(approximately 22px), height reservation (24px), related gap (8px) and existing
`title` system type in `ink` (light) or `rail-ink` (dark, equal to dark `ink`).
Navigation retains the existing rail width and About the existing dialog
structure. Exclusive task workspaces (finding pages,
capturing screenshots, checking for changes and browser setup) use the same
`navigation_lockup()` in their 192px rail, not a separate text-only heading.
Derived PNGs are transparent RGBA
(294×180px), with shared padding crop and Lanczos downsampling only; no
recoloring or artwork changes.
Compile-time embedded bytes and cached `Arc<Image>` identities let GPUI decode
and cache off-thread; no filesystem/network lookup or image decoding occurs in
the render callback.

### Desktop appearance — immediate, machine-local preference

Browser status remains first in App settings; one labelled **Appearance** group
immediately follows, with shared native radio choices **System / Light / Dark**
in painted keyboard order. The choices retain the existing settings scroller,
complete-control keyboard reveal, focus/selection distinction and modal guards.
System is the absent-file default and follows GPUI window-appearance notifications
live, including vibrant variants. Light/Dark ignore desktop changes until System
is chosen again. Palette changes refresh all windows/cached child entities and
prompts, not only the shell; state changes are immediate, with no theme animation.

The independent serial `appearance.rs` worker owns machine-local
`app-preferences.json` in the app-data directory, separate from project artifacts,
capture profiles and baseline ZIPs. It coalesces rapid choices without stale
completions replacing newer intent; protected temporary-file write, sync and
atomic replacement persist the choice. Startup loads before showing the window;
shutdown flushes after UI exit. File I/O never occurs in a UI callback or waits
behind capture work. Missing settings default to System; unreadable settings
explain the System fallback. A failed write keeps the session choice and reports
that it was not saved, with retry guidance. Choosing appearance queues no project,
capture, review or baseline operation. See `docs/research/dark-mode-20261016.md`.

### Stable field validation — 2026-10-18

Blur-time validation belongs to its field, not a paragraph inserted above the
form. Website and settings inputs share one help/correction region below the
40px control. An intrinsic grid measures the helper and known correction at the
actual wrapping width; only the current copy is painted. Error ink and a thin
error border identify invalid inputs; teal keyboard focus remains authoritative.
Starting-address website/subdomain scope stays separately visible. No motion is
needed: feedback changes without moving the form or its scroll position.

The setup footer replaces its supporting status with **Fix 1 field / Fix N fields**
and **Not saved**, in the same allocated space. Detailed corrections stay inline,
not repeated above the form. App settings reserves equivalent correction space
while Network settings is expanded and keeps correction recovery available after
collapse. Tab follows the painted footer/disclosure order; correction navigation
still reveals the actual field/stage without submitting. Save, draft and IME
guards are unchanged. Both themes/two-size native offscreen confirmation and
qualification limits: `docs/research/validation-layout-20261018.md`.

### Guided setup — native connected progression

**1 Website → 2 Access → 3 Pages & sizes** is a connected, numbered progress
component pinned above the guided scrolling form. Numbered native marks and
Current/Completed/Upcoming words distinguish progress without disabled-button styling.
Invalid drafts keep their current stage; gated future stages explain the
prerequisite beside the current action, not as `Unavailable` steps.
Public/private choices remain keyboard/scroll-reachable at the minimum window;
the footer keeps the next safe action pinned.
Clean guided steps Continue without rewriting; dirty steps await authoritative
save before advancement. Private login remains
separate: Save settings first, Open login browser, then I'm ready. The footer
co-locates draft/saved/browser prerequisites. Saving never implicitly opens login;
Save & find pages explicitly authorizes discovery after its matching save.
Existing-project Settings remains explicitly saved with all advanced operations.
Optional environments, mappings and rendering options use a named Access
disclosure; required login controls remain in the main path. Multiple-website
authorization appears only for multiple permitted origins. Sizes lead Pages, with
selected dimensions, extent and image policy in readable plan lines, before
advanced scope/speed settings. Custom sizes and capture/replacement limitations
remain scroll-reachable. Mobile/tablet are website emulation, not real devices.
Mapping pairs lead their scope notes; included-target counts, coverage cause and
rediscovery next step sit beside Capture pages. Photo/bitmap and session-only wording keeps
the secure-store and replacement limitations truthful. Browser availability uses
the same runtime-aware state/help as App settings, never an unconditional install
promise. The navigation rail has no preview slogan; **About Regressee** retains
version, local-storage, screenshot-privacy and development/release caveats.

The finite V5-1–V5-3 repair preserves guided validation stages versus full Settings,
reveals whole painted fields with error/help inside the current scroll clip, and
separates saved-status observation from draft hydration permission. Project-bound
origin/request snapshots and saved login identities preserve newer URL/provider/
representative/retention edits across absent-status saves and follow-up reads.
Both-size mounted/enabled Tab/reveal/dispatch tests cover setup capabilities;
Mobile remains scroll-reachable rather than forced into the initial narrow view.
The exact external worker probes pass, as do 308 package tests. Six narrow native
repair pairs substantiate Access/Pages validation and Mobile reachability; original
FIX/blocked evidence remains intact. Independent re-verification passed;
OS-input/AT/usability/release qualification remains pending. See
`docs/research/whole-app-ui-review-20261012/phase5-report.md`.

**Project settings wayfinding — 2026-10-14.** A labelled **Jump to** row pins
**Website**, **Capture plan** and **Access & environments** above one continuous
form, preserving its existing scroll owner, draft and explicit Save footer rather
than introducing tabs. Current-section selection follows measured content starts,
including disclosure and resize changes. Explicit jumps focus the first enabled
destination; Capture plan focuses Tablet when the sole selected Desktop cannot
be unchecked. Passive scrolling never steals field focus. An overflow-only native
scrollbar reserves a gutter (16px), with a thumb (8px wide, minimum 32px high),
using existing surface, line and muted/ink roles. Track clicks page by 90% of the
reading viewport; thumb dragging continues outside the gutter and releases there;
wheel scrolling uses the same owner. Tab/Shift-Tab follow painted order, skipping
unmounted/disabled controls and retaining complete-control reveal and validation.
Capture details and limitations belong inside Capture plan, before Access.
Navigation neither saves, opens login nor changes a baseline, and adds no key
handlers. This ordinary extension needs no new system tokens or shipping rasters.
Scoped SHIP covers the three reviewed corrections, not the whole app; two-size
synthetic Linux GPUI offscreen evidence and its limits are recorded in
`docs/research/project-settings-wayfinding-20261014.md`.

**App-wide scroll affordances — 2026-10-16.** Scrollable lists, forms, menus,
disclosures and reading panels share the overflow-only native scrollbar in
`ui/scrollbar.rs`. A rounded muted thumb stays visible while content overflows;
hover/drag strengthens its ink and reveals the quiet track. The stable 16px hit
area surrounds an 8px thumb with a thin surface keyline. Existing reserved
settings/dialog gutters remain reserved; elsewhere overlays preserve layout and
full-row selection. The thumb normally has a 32px minimum, reducing only for
very short viewports. Track clicks page by 90%; dragging releases outside the
panel; wheel input and keyboard reveal retain the original scroll owner and focus.
No overflow means no painted or interactive bar. Virtual lists remain virtual.
The same treatment covers both outer and nested History scrolling, Results and
page queues, settings/setup/project workspaces, options/inspectors, task bodies,
feedback, approval and information dialogs, placeholder explanations and the
multiline note editor. Note scrolling does not move its caret, edit or save a
draft; reflow keeps an already-visible caret visible without snapping a manually
scrolled reading position back to it.

**User-directed exception:** actual screenshot/image panes have **no scrollbars**.
They retain direct drag-to-pan, wheel/keyboard navigation and synchronized zoom;
image pixels and existing gestures are unchanged. Text explanations in missing/
loading screenshot stages still use the shared reading scrollbar when needed.
No new palette, motion, dependency or shipping raster. Scoped verification and
qualification limits: `docs/research/scrollbars-20261016.md`.

**Actions and choices.** Shared actions have a minimum target height (40px).
Buttons fit their labels; navigation fills its column; choice/check rows fill
their available width with balanced `spacing.row` insets even when help wraps.
Checkbox label owners now use a finite flexible column so long labels wrap rather
than clip. The analysis noise-suppression label paints on two lines at both
qualified logical window sizes (270×40px inner text, 326×66px complete control).
Primary keeps its filled teal identity through focus; danger uses error ink and
surface, never primary fill. Secondary hover uses `action-hover`. Focus is distinct
from selection: ordinary controls use `accent` outlines, rail navigation a
`rail-ink` outline separated from its selected surface by a `rail` gap. Disabled controls
fade, leave the focus model and reject activation. Shared focus rings use a
2px outline beyond a 1px contrasting gap (3px total extent): `accent`/`surface`
in the workspace and `rail-ink`/`rail` on the rail. Keep the normal control border, rather than
adding a second dark stroke; editable fields retain their teal focused border.
This user-requested refinement supersedes the oversized 3px/2px focus treatment
documented in `docs/research/keyboard-focus-20261015.md` without changing keyboard
activation or the hard-edge renderer fix. Focus remains visible through hover without
changing layout, selection, primary/danger identity or inherited GPUI text.
Keyboard reveal measures complete bordered choice/check/action rows, not their
padded content alone. Disclosure focus handles are distinct from project rows.
Current refinement evidence and limits: `docs/research/restrained-focus-20261016.md`.

**Labelled action icons — 2026-10-16.** Ordinary icon-plus-label actions retain
their Primary, Secondary, Danger, Navigation or Choice styling; they do not become
quiet toolbar tools. `action_icons.rs` maps semantic `Control` identities to the
shared `ToolIcon` paths, not translated label substrings. This carries recurring
review, project, settings and History actions through one language while keeping
consequential save, capture, baseline, export and delete wording explicit. The
native path slot (18px) and related gap (8px) supplement the full label; icon ink
follows the variant's text, including `on-accent` primary and error-colored danger ink.
The existing minimum target (40px), focus, disabled and selected treatments remain.

Consequential labels must wrap, not disappear behind clipping, ellipsis or forced
no-wrap. `controls::action_with_icon` reuses `action_frame` and a `RenderOnce`
`ActionLabel`: a bare flex-column text child can collapse to zero preferred width
in intrinsic-width GPUI containers. Measure the maximum natural line width with
the effective `window.text_style()` font, size and weight through
`window.text_system().shape_line`; expose that preferred width with `max_w_full`,
`min_w_0` and column wrapping. The parent then subtracts the icon, gap, border and
padding before wrapping. Do not replace this shared measurement with hardcoded
per-button widths. Preserve exact `action-{id}` and `action-label-{id}` selectors
and the existing mounted/enabled, focus, reveal and guarded click/keyboard wiring.

**Compact workspace toolbars — 2026-10-16.** `ui::toolbar` supplies shared native
stroke tools, not generic outlined text buttons. Icon-only targets use the
`toolbar-icon` variants above; labelled tools retain the same height and grow to
their content. Icons are drawn paths (18px, 1.6px stroke), with related icon/label
spacing (8px). Quiet idle tools have transparent fill and boundary with `ink`;
selection pairs `selected` mint with an `accent` boundary and `accent-hover` ink.
Hover uses `action-hover`; the shared teal focus ring remains distinct from
selection. Shared `controls::action_frame` preserves focus/disabled/activation
behavior without reserving an empty label child. Disabled tools leave traversal.

The viewer uses one 40px row: Results, an icon/current-name/chevron View chooser,
the labelled Before/After choices in Single image mode, and Zoom, then trailing
inspector tools. View menu rows pair matching icons with
names, full-width mint selection and a drawn check, not a repeated **Current**
label. Results, Image controls, Details and Analysis are icon-only; each enabled
icon-only tool discloses its action name on hover and keyboard focus without
another focus stop. Hints fit their text and stay within the window. Labelled tools
and menu rows have no redundant tooltip; disabled tools have none. Drawing
replaces secondary inspector tools with labelled **Done**, not an extra row.

Queue/page arrows, sidebar, refresh, History section/type choices and History/
project pagination share this tool language. Previous/Next item controls use
up/down chevrons to match the vertical results and page lists; History/project
pagination keeps left/right chevrons. History sections/types, Filter and
page options/details retain short names; pagination retains its page number.
Capture, save, review classifications, baseline consent, export and deletion stay
explicit text actions with existing guards. This extends the incumbent palette,
system type and geometry; no new font, dependency or shipping raster is needed.
Visible name hints do not resolve the native accessibility integration gap.
Source-bound three-size development evidence and reported tests are recorded in
`docs/research/compact-toolbars-20261016.md`; independent final visual review is
separate, and no OS-input, screen-reader or release approval is implied.

**Grouped inspection context — 2026-10-16.** One compact heading (40px) places
path/profile in a two-line identity stack beside a Before/After source stack,
separated by the group gap (16px), not equal-width columns or decorative cards.
Capture-only evidence names Capture instead of inventing a Before source.
`InspectionHeading` is a native `RenderOnce` component: shaped text determines
preferred group widths, capped at (440px) for identity and (320px) for sources,
with the source cap reduced to (180px) below actual working width (720px). These
are caps, not fixed widths to reserve for short content. Source context does not
shrink; original dimensions have a nonshrinking slot and the profile name yields
first. Long path/name/source text may ellipsize in this compact metadata; complete
keyboard-accessible metadata lives in existing Details, never a hover overlay.
Details includes full Before/After environment names and origins, or the Capture
environment for capture-only inspection, alongside the full page and screen size.
Identity and environments come from the inspected run/reference, never today's
project configuration or a filtered-out replacement target. The heading persists
across modes, hidden Results and matching pending image reads.

**Tooltip restraint — 2026-10-16.** User feedback rejects unsolicited explanation
over working content. Only enabled icon-only tools retain short identifying names
on delayed hover and keyboard focus. Mouse clicks retain actual focus, but never
pin the identifying tooltip after the pointer leaves. Pointer input dismisses
keyboard hints; keyboard input restores them, independently per window. Visible
labels, View menu rows, Zoom, page/result rows,
links and inspection metadata never spawn explanatory bubbles. Zoom keeps its
mode/scale visible and its choices explicit; gesture/keyboard help stays behind
Image controls. Long addresses and evidence-derived source context remain in
Page options, Page details and Capture details. No workflow, action guard, target
size or evidence-height change is needed. Scoped verification is recorded in
`docs/research/tooltip-restraint-20261016.md`; earlier tooltip-specific acceptance
records do not override this user-directed policy.

This source-backed extension adds no palette, font, dependency, motion or shipping
raster; visible names do not close the GPUI accessibility integration gap. Tests,
capture confirmation and independent finish status belong to
`docs/research/action-icons-spacing-20261016.md`; no final outcome is claimed here.

**Keyboard activation repair — 2026-10-15.** Shell Enter/Space key bindings are
the single keyboard activation owner; click wiring accepts pointer clicks only.
GPUI's synthesized key-release click must not re-toggle a section, undo a check
or activate the newly focused Close button after opening a dialog. Text spaces,
explicit saves, disabled/mounted guards and selector-owned keys are unchanged.
The Linux Blade shader handles zero-blur outlines explicitly rather than
evaluating a zero-width Gaussian. Full press/release tests and source-bound
offscreen focus pixels are recorded in `docs/research/keyboard-focus-20261015.md`;
these do not establish OS-input or assistive-technology qualification.

**Inline expandable sections — 2026-10-15.** Optional content uses a shared
disclosure heading, not the bordered field shape or trailing chevron of a select.
A native-drawn leading chevron points right when closed and down when open;
the stable title is paired with explicit **Show / Hide** text. Neutral headers
have a 40px minimum target, wrapping titles, existing hover treatment and visible
focus. App-settings section headers use 18px/24px; compact nested headings use
14px/20px. Selection tint means selection, not expansion.

Header, persistent explanation and expanded body share one layout owner. Help
and content align below the title with a 32px marker gutter; 4px separates related
rows and the body uses an 8px inset above/below. Section rules belong before
independent groups, never between a disclosure and its first child. Ordinary
expanded content stays in its existing screen/inspector/dialog scroller. Expansion
reveals the heading with the beginning of the body; collapse restores its focus
without saving or discarding drafts. Independent sections remain independently
openable. Real timezone/zoom/View selectors and bounded flyouts keep their
selection, keyboard and dismissal behavior.

This pattern covers App settings, capture/setup options, ignored areas, advanced
analysis, metrics, capture diagnostics, History, exact preview details, baseline
source details, page-search details and task details. Task details is located
above its content in the reading body, while Stop/Pause/Continue remain pinned.
Requested saved-data check results remain available under Troubleshooting even
after its optional tools collapse. Support-report children no longer start a new
ruled section or outrank their owner. No new colors, rasters, motion, backend
operations or approval permissions are introduced. Verification and its native
qualification limits are recorded in `docs/research/disclosures-20261015.md`.

**Links.** Presented website/page addresses that offer navigation must be real
links: teal accent ink plus an underline at rest, a pointer cursor, visible
keyboard focus and Tab/Enter activation. Hover/press changes only the link ink to
`ink` and thickens its underline from 1px to 2px, never filling its row or changing
text metrics. Do not rely on color or hover alone, and
never make ordinary noninteractive text look like a link. External page links
open the OS default browser only on activation; capture-session credentials are
not transferred. No hover explanation repeats this familiar link behavior.
Website settings and page details retain complete addresses.
Only complete HTTP(S) addresses without embedded credentials are openable;
unsupported schemes remain plain data. Editable URL fields, literal code and raw
diagnostics are not automatically linkified. Task context and gap links share
this treatment and scroll into view when reached by keyboard.

**Fields and timezone.** Native input stays on `surface` when focused, with accent focus
paint and stable action metrics. Disabled fields reject editing and leave focus
order; storage work locks affected fields. Clipboard, IME composition, pointer
selection, Home/End and anchored Shift selection belong to fields. Double-click
word selection is a basic fallback, not locale-aware segmentation. Enter does not
submit settings; saving is explicit. Timezone is selection-only: search never
becomes a profile value, legacy imported identifiers remain unchanged until a
choice is made. The neutral trigger reserves space for its drawn chevron. Its
bounded popup chooses available space above/below, virtualizes options and keeps
current selection distinct from keyboard highlight. Arrows/Home/End navigate,
Enter chooses, Escape dismisses/restores focus; Tab follows the shell order and
outside dismissal does not steal its target's focus.

**Search and list filters — 2026-10-17.** The user rejected the queue's separate
Search label and requested an app-wide search/filter-field refinement. Pages and
results, loaded History and the timezone picker share the native input's 40px
height, 4px corners, 12px insets, a 16px drawn magnifier and 8px icon/text gap.
Use contextual placeholders (**Search pages…**, **Filter this history page…**,
**Search region or city…**) in the theme's readable muted ink, not a separate
visual field label. Search text uses ordinary editable ink; the magnifier remains
visible. Placeholders are paint only, never selected/copied/committed or included
in IME ranges. Shared queue/History rows place their existing named clear actions
beside the field; History's loaded-page scope stays persistently visible below.
In short result queues, search/filter/clear controls precede a bounded scrollable
no-match explanation; recovery text must not push those controls out of view.
The changes-only notice names **No matches** when searching yields none.
Preserve ordinary settings labels, filter choices, keyboard order, draft/evidence
retention and all explicit capture/save/approval boundaries. No palette, asset,
dependency or animation change. Native accessibility integration remains the
existing qualification blocker; a placeholder is not a screen-reader label.
Scoped verification: `docs/research/search-fields-20261017.md`.

**Queue focus clipping — 2026-10-17.** The queue/viewer overflow boundary must
include the shared focus ring's full 3px paint outset. Equal padding and negative
margins expand only the clip boundary, preserving queue alignment, control sizes
and screenshot geometry. Do not remove evidence/list overflow protection or
special-case Search: Filter and other edge controls need the same clearance.
Scoped verification: `docs/research/queue-focus-clipping-20261017.md`.

**Interface wording — 2026-10-14.** Routine messages explain the outcome and next
action in ordinary language, following `docs/ux-writing.md`. Baseline saving says
what future checks compare against and keeps the same-page-list consequence.
Setup, capture, review, History, ZIP sharing and cleanup use pages, screenshots,
results and files rather than internal inventory, binding or object terminology.
Current versus saved baseline versions remain distinct. Source records stay
available as **Screenshot source details** or **Technical details**; exact IDs,
diagnostics, privacy warnings and all operation/approval guards are preserved.

**Plain-language follow-up — 2026-10-14.** Closing any viewer panel is **Close**,
not a navigation instruction. Notes show **No changes**, **Unsaved changes** or
**Saved**, without byte counts. Their helper explains draft retention and Save
note; rejected edits explain how to recover without UTF-8 or storage budgets.
Setup and advanced settings use pages at once, approved login websites,
website/section addresses and small-area comparison settings with examples.
Capture, History, browser recovery and the support report describe screenshots,
files and actions rather than internal evidence, inventories, bindings or build
phases. Exact source records and diagnostics remain in their named disclosures.
The existing palette, controls, editing limits and explicit mutation boundaries
are preserved. Copy regression coverage lives in `ux_copy_tests.rs` and the exact
note-save/editor/quit tests.

**No-op controls — 2026-10-15.** App settings exposes actual data/cache paths under
**Storage locations**. The former folder-picker preview and its activation/focus
wiring are removed: choosing an unused path is not a product task. Do not expose
picker demos, disabled future features or success notices without a useful result.
Keep genuine previews that inspect real data, capture a sample, or lead to an
explicit import/export/baseline/cleanup decision. The existing settings hierarchy,
native controls, palette and keyboard reveal remain unchanged; no new tokens or
shipping rasters. See `docs/research/no-op-controls-20261015.md`.

**Context and action wording — 2026-10-16.** The user rejected **Before Website /
After Website** and requested an app-wide review of similar wording. Inspection
headings now name the screenshots' roles: **Before — Saved baseline** and
**After — This check** (or **Recorded check** in read-only History). Generated
“Website” names and identical environment names do not repeat as distinctions;
meaningful differing recorded names remain alongside the roles, and a shared
meaningful name appears once. Full environment
names, origins and dates remain in Details, and missing source records stay
explicitly unknown. Capture-only context distinguishes saved baselines, confirmed
unsaved candidates and screenshot previews; History uses neutral **Recorded
capture** when baseline status is not established. This supersedes the earlier compact heading's
unconditional paired environment names, not its provenance safeguards.
Setup saves a **project**, not a website. **Find pages**, **Confirm login**,
**Page address mapping** and **Add / replace address mapping** describe the actual
operations. Login instructions and recovery use the same control name; the
privacy-filtered **Support report** has consistent preview/save/progress/recovery
copy, distinct from capture diagnostics. Skipped/stopped screenshots do not claim
an attempted capture failed. Network loading text appears only during its actual
read; recovery points to the OS for folder permissions. Count templates use
singular/plural correctly. Generic task-workspace labels are omitted. No palette,
layout-system, saved-record, eligibility or consent-policy change is introduced.
See `docs/ux-writing.md` and `docs/research/ui-wording-20261016.md` for the scope
and bounded verification record.

**Feedback and progress.** Dismiss belongs inside the feedback surface, centered
beside the message. Long diagnostics scroll within a bounded area; errors retain
their red treatment. Known nonzero totals alone produce a clamped completion
fraction. Download amounts appear once in readable byte units above the bar;
the supporting row shows elapsed time, not a duplicate unitless step counter.
Unknown/zero totals pair honest text with a smoothly moving patterned
activity track;
discovery shows the actual pages visited. No fabricated percentage or coverage.
Routine discovery visits and short pacing pauses share a steady Finding pages
status with the actual visit count. Only crawl-rule waits of at least two seconds
show a waiting phase; retries, preparation and finishing retain explicit status.
Short local reads use a compact activity label instead of a patterned completion
track. Every queued operation names its task immediately, before worker updates.
Projects, screenshots and History distinguish initial loading, authoritative
empty results and failed reads. Opening a project never announces missing
screenshots or an absent baseline before its saved evidence is read; refresh
keeps already loaded evidence visible.

**Loading feedback refinement — 2026-10-14.** Compact activity uses a 16px native
stroke spinner beside readable, medium-weight task text, never a filled dot that
resembles a list bullet. Global operations use a palette `surface`, thin-bordered status
surface with a 40px minimum height; longer labels wrap without moving the icon
away from their first line. Projects, History, screenshot/evidence placeholders,
browser verification and storage reads share this treatment. Placeholder help
aligns with its heading text. An initial read has one visible status owner:
suppress its duplicate global label when that same read is already explained in
the page. Saves, errors and reads of already-loaded content retain their feedback.
Known-total progress bars, passive status labels, error semantics, operation
guards and screenshot originals are unchanged. Bounded Linux offscreen inspection
and scoped tests passed; see `docs/research/loading-feedback-20261014.md` for the
verdict, source-bound captures and unrelated full-suite failure.

**Task workspaces and dialogs.** One shared panel separates compact project/site/
environment context, current activity, labelled outcomes and a single safety note.
Supporting prose and notes cap at 560px; whole count items wrap together rather
than separating numbers from labels. Small native-drawn task-status icons sit
beside words. Attention is amber, actual failures are red, saved captures use
normal ink, and zero counts stay muted. A saved-screenshot checkmark never means
baseline approval. Screenshot progress accounts for processed page/profile
targets, including failures, not successful captures or an invented ETA.
An information note stays on `surface` and explains the relevant operation/state;
finished results do not inherit cancellation advice. The independently scrollable
body has a visible overflow indicator; Stop/Pause/Continue stay in its fixed footer.
Details appears only for independent information: a run's frozen capture setup,
recorded gaps/reasons/attempts/warnings, discovery notes or an operation diagnostic.
It never replays the progress label or repeats the base URL as its only extra fact.
Opening Details reveals its scroll anchor. Gap previews prioritize failures, cap
at 20 rows and disclose truncation with a route to the full review workspace.
No chronological activity log or fabricated active-target telemetry is supplied.

Capture completion reconciles the frozen URL list with included/excluded counts;
URLs excluded before capture are not failed or missing screenshot targets. Zero
successful captures say **No screenshots captured**, expose a shared recorded
failure cause when available, and offer **Review capture failures**, not a promise
of saved screenshots. Inspection uses ordinary-language failure and recovery copy;
starting capture removes stale discovery instructions. Blocked embedded frames
produce a capture warning rather than a false main-page redirect failure.

Browser-setup failures name the actual failed installer stage and open Error
details automatically, including automatic setup during discovery or login.
Local details retain elapsed time, received bytes, platform, app version and
available nested HTTP/OS/worker causes; final installation publication is separate
from the browser test. Request URLs and raw browser/site stacks are excluded.
These on-screen errors are not added to the privacy-filtered diagnostic export;
local paths can still appear and screenshots require review before sharing.

**Failure recovery — 2026-10-14.** Known browser, worker and local-I/O causes
cross the background-job boundary as structured categories, not guessed substrings.
Different installed browser releases say **Browser update required**, while an
unverifiable matching installation says repair; neither is trusted or replaced
automatically. The main message explains the cause and next safe action, with
**Open App settings**, **Open Project settings** or **Open History** as real local
navigation. History recovery opens Storage. Navigation clears the dismissed
blocking banner so read-only settings can load, but preserves drafts, evidence
and explicit save/login/install/cleanup boundaries. Save-and-find failures
explicitly confirm the preceding successful project save. Known causes do not
require opening Details; unknown causes keep a bounded explanation and
operation-specific recovery, with full available diagnostics separate. Installer
recovery follows its actual stage. Updating does not make an old baseline
automatically comparable: checks still require matching browser components.

**Partial baseline recovery — 2026-10-15.** A finished baseline capture with at
least one successful screenshot keeps **Save baseline…** available even when
other captures failed. The compact amber next step states actual saved/intended
counts and the option to skip missing screenshots; **Review failures** filters
the queue without changing inspection or drafts. Explicit native confirmation
names saved/skipped counts, exclusion from future checks, replacement consequences
and retained History. Keep reviewing never saves. After saving, **Run check** is
the primary next action; coverage remains truthful without repeating the warning
across several persistent surfaces. Checks visit exactly the saved page/size
pairs, with reduced baseline coverage visible in subsequent results and History.
Original discovery, failed outcomes and prior versions are retained. Empty,
unfinished, interrupted, cancelled, preview and check runs remain ineligible;
corrupt evidence and stale revisions still fail closed. This separate whole-
baseline workflow does not approve failed evidence or loosen selective approval.
No new tokens or shipping rasters. See `docs/research/partial-baseline-20261015.md`
for scoped tests, native offscreen evidence and qualification limits.

**Comparison and approval.** Coverage comes from actual inventory, captures and
current analysis, including missing and unclassified evidence. Preserve before/
after origins, screen size, warnings, sensitivity and ignored-area revisions.
Original screenshots remain untinted; difference, overlay and heatmap are labelled
analysis modes, not brand treatments of the originals. Heatmap alone displays the
production legend for significant differences and ignored pixels; these are
derived results, not confirmed defects. The View menu is trigger-anchored, uses
full-width selected rows and stays within the window. One stable path/profile/
recorded-environment heading and compact toolbar persist with Results hidden.
Hiding Results does not reduce evidence height; full-width Before/After preserves
the inspection identity and source scale. One highest-priority reason explains
blocked review/Next actions: dirty note/areas, pending analysis, noncomparable
evidence or no next matching target. The baseline footer appears only for an
eligible current screenshot or an existing selection. Its heading is **Update
baseline**, with the attached explanation **Use After as the reference for future
checks. Review before saving.** **Select After screenshot** and **Deselect After
screenshot** name the actual original being selected. Nonempty selection names
selected references ready for update, and mounts **Review update…**; an empty
selection has no confirmation action. Below 900px working width, explanation and
actions stack instead of squeezing or overflowing. Eligibility is unchanged.
The protected approval dialog has a bounded height (520px) and widths (740px)
at narrow desktop sizes or (760px) otherwise, capped by its available viewport.
It asks “Use this After screenshot for future checks?” (or “these After screenshots”), names
the future-check consequence and retained versions, and distinguishes Replace
from Keep. One scroll owner contains the inspected path/profile/source environment/
UTC time, on-demand exact provenance and the reference rows; actions remain pinned.
Rows grow with long addresses rather than painting separators through text.
At most 32 reference rows mount on a page. Previous/Next inspects every reference
and advances the visible page, keeping all unchanged references accessible.
Exact page/profile IDs, run/artifact identities and requested/final URLs remain
available, including the technical tail. Keep reviewing/Escape dismisses without approving;
**Save baseline update** is explicit. Each new or cached consent opening resets the
inspected reference, exact-details disclosure and actual reading scroll to the
top, so consequences cannot disappear behind a previous tail inspection/resize.
Focus is trapped, background mutations are guarded,
and stale previews are invalidated. Failed/missing/incompatible evidence cannot
be approved; replacing the whole baseline remains a separate workflow.

Ordinary review exposes revision-bound decisions, explicit note saving, progress
and Next unreviewed without opening Analysis. Classification is not approval:
**Select After screenshot** names reference selection explicitly. Discovery
owns the current inventory and optionally a matching representative screenshot,
never an earlier comparison. Narrow review compacts chrome, prioritizes original
dimensions in two-line captions and can collapse the queue. Details and Analysis
are mutually exclusive; neither hides ordinary review decisions or discards drafts.
Capture/review counts form a compact labelled status strip, not prose paragraphs
or decorative KPI cards. Limited/interrupted inventory and genuine gaps stay
visible. One concise next action replaces repeated cautions; exact context/URLs
are disclosed in Details. The default summary names the page, screen size,
capture status, readable UTC run time and recovery advice. Exact IDs, source
records and descriptors are opt-in Technical diagnostics, not default copy.
Technical diagnostics uses ruled, labelled groups and indented monospace JSON
instead of Rust debug output. Dates show UTC with the exact Unix seconds retained;
missing records are explicit, never inferred from current settings. An expanded
diagnostics panel pins **Copy diagnostics** and a full-URL/settings sharing caution
below its one reading scroller. Copy takes the complete report, including offscreen
records, and confirms inline without moving the button. It does not save, export
files, run a capture, alter evidence or replace the privacy-filtered support report.
Single-image captions combine capture identity
and screen size, with one origin line.
Keyboard ignored-area creation/editing uses original integer coordinates, explicit
Apply and Save, and focus-revealing scroll anchors. Unapplied or all-masked areas
cannot be saved.
Analysis leads with the actual saved result/current-settings summary and
**Sensitivity** presets. **Ignored areas** has its own disclosure containing Draw
and Add by coordinates. Advanced parameters/switches and metrics follow the common
choices. Complete labelled controls reveal within the actual inspector scroll
owner, not just the whole window. Reanalysis uses saved originals, retains earlier
revisions and does not visit websites; Cancel and previous-analysis actions mount
only when meaningful. **No significant change** means below thresholds, not
identical pixels.
Only comparisons expose Analysis (sensitivity and ignored areas); capture-only
views have no generic Tools mode. A flyout has an attached Close action and Escape
restores its trigger. Drawing an ignored area folds away the flyout and returns
after a completed rectangle, with drafts still requiring explicit save.
Image navigation is direct: drag or two-axis scroll to pan, Shift+scroll for
horizontal movement, Ctrl/Cmd+scroll to zoom at the pointer. Before/after stay
synchronized. Screenshot hover never opens navigation help. **Image controls**
in the trailing toolbar group opens an explicit, bounded flyout with readable
gesture/shortcut rows and an attached Close action; Escape restores its trigger,
and an outside click closes without taking focus from the clicked control. At
narrow widths the help action lives at the end of the Zoom menu rather than
adding a toolbar row. One
muted drag/zoom hint shares the existing caption only when there is room; no new
strip reduces evidence height. macOS names Command, other hosts name Ctrl. Help
shares the mutually exclusive inspector lifecycle, preserves drafts/scale/target,
and never triggers a capture or baseline operation.
A single focus stop provides +/− zoom, arrows/Page Up/Page Down,
0 for fit width and 1 for actual size. One Zoom menu combines fit width,
actual size (100%) and percentage presets at all supported desktop widths. Its
trigger pairs a zoom icon with “Fit width · 26%” in automatic mode or “75%” after
manual zoom; the compact form shortens only the fit name to “Fit · 26%”. These
percentages illustrate the actual current scale, not fixed fit values. A drawn
chevron makes the selection affordance explicit. Fit follows resizing;
fixed percentages do not. Arrows/Home/End navigate the menu, Enter chooses,
Escape dismisses, and Tab continues the shell order without trapping focus.
Both forms keep the name/scale on one line in a 40px control (188px ordinary,
132px compact); the selected mode remains explicit in its menu. This supersedes
the former text-heavy toolbar and two-line narrow zoom treatment, without
changing their historical verification records or adding toolbar rows.
Missing images have no useless zoom/pan controls. A hand/crosshair cursor conveys
the current interaction.
Compact no-match recovery precedes the search controls, with expanded
filters scrolling separately.

**Changes-first result queue — 2026-10-16.** Opening a saved comparison or recorded
check defaults to Changed, using the effective saved pixel verdict (including
legacy warning-bearing reports). The queue explains **Only changed results shown.**
with a quiet **Show all** action. Capture-only evidence remains unfiltered. A
refresh or another analysis of the same run preserves the explicit filter choice;
a new run starts with the appropriate default. Show all changes only the status
filter, retaining search, grouping, inspected evidence, notes and ignored-area
drafts. Its disappearing keyboard stop returns focus to Filter. A no-change queue
says **No changed results**, not that every capture passed; the unfiltered coverage
strip still calls out missing/not-compared results, available through Show all or
Filter. The compact notice replaces the queue heading; redundant shown counts
are omitted in changes-only mode. No evidence, review or baseline is changed.
Verification and limits: `docs/research/changed-results-default-20261016.md`.

**Unchanged results complete automatically — 2026-10-16.** The saved effective
`NoSignificantChange` verdict displays a check mark and **No significant change**,
not Unreviewed / Intentional / Needs fix choices. The user rejected the redundant
“no review needed” suffix and footer count; the comparison strip already counts
these results. Footer progress names only manual review work and gaps. The verdict means
below the saved thresholds and outside ignored areas, not necessarily identical
pixels. Next unreviewed visits only genuinely changed, captured results without a
current decision, respecting filters, wraparound and draft guards. Progress
distinguishes manual decisions from changes still to review.
Optional notes and explicit baseline selection remain available; a successful
check never creates an Intentional decision or updates a baseline. Existing saved
decisions/notes remain visible and retained. Loading warnings stay independent
cautions, without inviting a redundant review. Pending analysis and unreadable,
missing or noncomparable evidence never use the success footer. Legacy saved
pixel reports work immediately without a recapture, reanalysis or history rewrite.
This supersedes the earlier comparable-result disposition/navigation rule below;
the other approval, provenance and inspector safeguards remain. Plan and scoped
verification: `docs/research/unchanged-review-20261016.md`. No new tokens or assets.

**State-aware review footer — 2026-10-16.** Comparable results have a compact
Review/progress line above selected **Unreviewed / Intentional / Needs fix**
choices, separate from Review note and the primary Next unreviewed action.
Secondary zero counts and repeated baseline disclaimers are omitted. Loading
warnings are advisory: one quiet caution beside Review, not a blocking paragraph.
Pixel comparison, queue status/filtering, progress and review actions use the
actual saved pixel verdict, including older warning-vetoed summaries with valid
reports. Matching warning codes alone never establish identical pixels. Saved
reports, revision IDs, decisions, notes and baseline versions are not rewritten;
no forced reanalysis or recapture is needed. Details attributes cautions to Before,
After or both, bound to this analysis's baseline reference, not today's settings.
Stable captures with ordinary loading warnings remain eligible for deliberate
baseline selection; confirmation explains that their captured appearance becomes
the baseline. Failed, missing/corrupt or incompatible evidence, hard comparison
gates and unstable/replacement-problem baseline updates remain protected.
Genuine comparison failures replace disabled decisions with the actual saved
cause and safe next step inline. Absent explanations stay explicitly unknown.
Details and Analysis share that explanation; numeric difference triggers do not
masquerade as rejection reasons. Long failure reasons scroll within 108px with an
overflow thumb while recovery remains visible.
Optional **Technical details** opens the existing inspector with supporting
records expanded, focuses Close and restores its trigger on Close/Escape.
Next unreviewed mounts only when another matching reviewable result exists.
Baseline selection remains in its
separate conditional footer; **Review update…** mounts only after
selection. Stale selections stay removable, with confirmation disabled.
Explicit consent, draft and revision-binding guards remain. Notes use the owned
temporary editor rather than a permanent evidence-height row. Details, notes and
filters remove or disable review/baseline decisions while evidence is occluded;
Close/Escape preserves unsaved note text without saving it. Scoped verification
and qualification limits are in `docs/research/review-footer-20261016.md`.
The user's rejection of its missing-cause/focus states and fresh scoped repair
evidence are recorded in `docs/research/comparison-recovery-20261016.md`; the earlier
generic-state verdict does not qualify these reported states. The subsequent
user-directed advisory-warning correction supersedes the earlier warning-veto
policy; current verification is in `docs/research/advisory-warnings-20261016.md`.
No new palette, asset or shipping raster.
The finite V4 repair gives the inspector scroll body a finite content width;
the actual note input is 326px wide (300px inner content). The current review-note
editor is bounded and genuinely multiline, with wrapping, caret/selection and
vertical scrolling. Pasted newlines are preserved. Editor, Save and save-on-exit
share the internal **8,192 UTF-8 byte** storage limit: exact save or explicit rejection with
the draft retained, never silent truncation. The interface never exposes that
encoding or byte budget, nor relabels it as a character limit. Notes explain that
closing keeps the draft and saving requires Save note. Length errors ask users
to shorten the note or paste less and state whether anything was added or saved.
One plain-language draft/saved/invalid status sits
beside Save; overflow hints reflect actual overflow. Close/reopen preserves the
owned draft and restores its trigger. Saved-project Results loading/failed-read
candidates now permit local navigation restoration without bypassing task/modal guards.
When results are hidden, an evidence-owned path/query/fragment and source profile/
environment heading persists across all modes and capture inspection. Narrow
Compare names Before/After historical environments above their respective panes.
The earlier blocked confirmation and independent FIX verdict remain intact.
The separate 22-pair functional repair evidence received independent bounded
re-verification, not OS-input/accessibility/usability/release acceptance. See
`docs/research/whole-app-ui-review-20261012/phase4-report.md`.

**History, consequences and recovery — Phase 6 independently verified in bounded scope.**
Runs and Baselines use one to four actual 64px rows for sparse lists and a 256px
inner viewport for longer loaded pages; zero records mount no empty list.
Full-width selected paint is unchanged. The outer workspace remains scrollable:
at 820×600 the initial list extends below the fold; scrolling exposes its full
four-row viewport.
Overflow help explains scrolling and keyboard reveal, including the last loaded
run/retained version. **Filter this history page** explicitly excludes unloaded
pages. Both Runs and Baselines have page controls, disabled when no page exists.
Readable UTC dates, record types, captured/intended coverage and environment lead;
the selected identity and its open/share/delete actions sit together. Exact IDs
and byte accounting remain under advanced provenance, including Storage.
App settings uses the same heading as its navigation entry. Projects import and
App settings secondary actions reveal their painted bounds in the actual outer
scroll owner when traversed by keyboard; preview/confirm boundaries are unchanged.

ZIP/import/deletion previews lead with consequences. Originals may expose private
pixels; ignored rectangles are NOT redaction. ZIPs never carry sessions/cookies;
recipients establish their own login. Import is a separate new local project and
does not visit websites. Exact byte counts, record identities and recovery lists
use a separate **Show exact details** disclosure; it does not open History's
advanced provenance. Deletion names the run type/date or baseline save date,
requires explicit danger confirmation and preserves shared/retained source
protections. Temporary-only cleanup and permanent recovery-original deletion
remain distinct. Both reject zero-impact activation and leave disabled actions
out of keyboard traversal. Both actual deletion actions use danger treatment.

Recorded inspection uses its persistent read-only heading and Return to latest
results, without a redundant notice banner. An authoritative empty capture
reserves neither an empty queue nor unloaded target/profile labels. Actual failed
target outcomes (including a real before-only frame) show the local path, profile,
known cause and recovery instead of an empty image or irrelevant Analysis/Zoom.
Details remains available. The no-image body scrolls independently when long;
Refresh only reads disk, not recapture. Replacement remains explicit discovery,
capture and save. Genuine missing/corrupt image-read failures retain their failure
state through feedback dismissal and conservatively block baseline-update
presentation until a successful image read. An initial unread project cannot
invent an unknown target's metadata. Paused/interrupted task continuations say
**Return to capture recovery**; completed checks say **Review check results**.
The real workspace Resume control is retained, not fabricated by the task label.
Failure headings name the known problem, with the operation retained in task
context; unknown failures keep their operation-specific heading. No Open folder
or targeted retry was introduced. Browser recovery links to the existing explicit
installer in App settings, rather than automatically replacing components.

**Missing screenshot treatment — 2026-10-15.** Failed captures and unavailable
images reserve the remaining screenshot stage, with the same palette `surface` and
thin boundary as the image panes. A 48px native missing-image mark accompanies
an 18px heading, the recorded reason and quieter recovery guidance separated by
a rule. Failure marks retain error ink. The message centers within a 440px maximum
measure with 24px insets; long explanations start at the top and scroll inside
the fixed frame with an overflow-only thumb. Finite final-width text measurement
keeps the full recovery guidance reachable. Path/profile context and Details remain outside the placeholder;
truncated context is available in the explicit Details panel. No retry action, screenshot
pixels, capture operation or approval rule changes. See
`docs/research/screenshot-placeholder-20261015.md` for scoped verification.

**Screenshot loading treatment — 2026-10-16.** Screenshot reads, initial saved-
evidence loading and screenshot preparation share the missing-image view's palette `surface`,
thin-bordered stage, centered 440px message measure and 24px insets. A 32px teal
native stroke spinner sits above the 18px heading and existing muted explanation;
it denotes activity, not progress. Compact loading elsewhere stays 16px. The same
fixed-frame scrolling handles constrained heights, and the existing reduced-motion
override stops the spinner. The queue and inspection context remain available;
no fake screenshot, new action, palette, asset or approval behavior is introduced.
Scoped tests and two-size offscreen inspection passed; see
`docs/research/screenshot-loading-20261016.md` for evidence and limits.

**Stable result navigation — 2026-10-16.** Changing the selected result keeps the
View/Compare selector, Single-image sides, Zoom, image/inspection tools and eligible
review/baseline actions in place while pixels load. Image-dependent actions stay
disabled until the current read succeeds; loading never shows the previous
target's pixels. Split slider retains its name during the read, and a duplicate
queue-owned Results toggle no longer appears. Previous/Next result and Next
unreviewed reveal the selected row in the actual filtered/grouped queue without
moving keyboard focus. Already visible rows do not recenter; manual scrolling
after selection is retained when the read finishes. Draft, failed-read, stale-reply
and explicit approval guards remain. No palette, token, asset or animation change.
Scoped tests and three-size offscreen inspection:
`docs/research/result-navigation-20261016.md`.

**History and keyboard.** Runs, Baselines and Storage are separate sections.
Baseline versions and capture/check runs are distinct,
with active versus retained reference state and composite provenance explained.
Keep ZIP privacy previews, separate recipient login, storage totals, shared-source
protections and deliberate deletion; ignored areas are not pixel redaction.
Open selected evidence is read-only, uses its original reference/policy/mask and
does not activate a baseline or write display pointers. Capture-only records show
only their originals. Missing/corrupt evidence and stale selection/draft replies
fail closed. History selection and focus use full-width row geometry, confirmed
at both inspected window sizes.
Tab/Shift-Tab traverse only mounted, enabled controls. Enter/Space activate actions
without intercepting spaces in inputs. Ctrl/Cmd+N opens new-project setup,
Ctrl/Cmd+S saves, Ctrl/Cmd+1…5 switches sections and Ctrl/Cmd+Q quits. Dialog
dismissal restores focus; Escape never pretends to cancel a committed database
save. Unsaved switches require explicit discard confirmation. Owned window close
and application Quit use the same Cancel / Save and quit / Discard and quit guard;
save awaits exact authoritative replies and preserves invalid or newer drafts.
F1/App settings exposes keyboard help; correction summaries reveal the affected
form stage/field without submitting. Real OS input, platform termination behavior
and native accessibility remain qualification obligations.

**Focus reading order — 2026-10-16.** User keyboard testing found illogical jumps
between Filter, Compare, Results and the inspectors. Do not sort focus candidates
by control type. The rail follows its painted destinations; review follows the
project header and task actions, then the full-width toolbar left to right
(Results, View, optional Before/After, Zoom, Image controls, Details, Analysis),
queue search/filter/navigation, evidence, review choices/note/next and the baseline
footer. Inspectors follow Close, content, pinned save/copy actions. Page previews
and unavailable captures retain their evidence-column controls after the queue;
Done follows its drawing-toolbar position. Hidden and disabled controls leave
traversal. Tab exits View/Zoom from their trigger, with Shift-Tab reversing the
same order. Preserve appearance, drafts and all operation/consent guards.
Scoped verification and limits: `docs/research/focus-order-20261016.md`.

The user-requested carry-through confirms the previously incomplete History,
Tools, empty-message/help and capture-only corrections in fresh source-bound
pixels at 1280×800 and 820×600. See
`docs/research/ui-carry-through-verification-20261011.md`; this is offscreen-layout
acceptance, not native input, accessibility or public-release certification.

## DOM snapshot evidence — 2026-10-17

User-confirmed extension of the current neutral/teal Clear wayfinding workspace,
not a redesign. **What to check** offers Screenshots only (default), Screenshots +
DOM snapshots, and DOM snapshots only. The saved baseline's frozen choice remains
distinct from a future-capture draft.

Both-mode review adds **Screenshots / DOM snapshot** with independent outcomes.
DOM-only does not mount image tools, mask editing or pixel sensitivity, including
keyboard actions. The existing project/page/profile/source context and queue stay
attached while the evidence stage shows native read-only source. Literal source
uses monospace, old/new line gutters and `+`/`−` alongside semantic color. Bounded
rows, wrapping, collapsed context, search, change navigation and selection/copy
use the incumbent controls, focus and scroll treatment. Captured markup never
becomes an active web page or linkified executable source.

Notes, review decisions, Next unreviewed and selected references bind to the
check kind and immutable analysis/source identities. Consent names the selected
screenshot/DOM references and inherited ones; either kind or both may be updated.
Updating one must not strand the other from the same check while its original
Before is unchanged. Dirty drafts, occlusion, stale replies and explicit save/
approval guards remain. A successful sibling never hides missing/corrupt/unstable
evidence. History, partial coverage and ZIP previews disclose kinds and DOM privacy.

No new palette, decorative raster, font dependency or motion. Current source/model
and real-IPC tests pass within their recorded scope; final native/offscreen finish
review is pending. Verification and remaining OS-input/accessibility/platform
limits: `docs/research/dom-snapshot-verification-20261017.md`.

**New-project access save — 2026-10-18.** A new project starts with its own
environment selection and empty private login fields, never the previous
project's selection or login draft. Access-save activity says **Saving website
access settings** without implying login is required. Save errors show the
reported cause and preserve the full local error chain in Details; do not replace
an unknown cause with guesses about disk space. Drafts and explicit save/visit
boundaries remain unchanged. Scoped regressions and limits:
`docs/research/project-access-save-20261018.md`.

**Project and settings removal — 2026-10-18.** Preserve the current neutral/teal
Operate workspace. Projects rows expose **Delete project…**, with the same action
in a separated Project settings section. Read the actual impact off the UI
thread, then show one named, Cancel-first confirmation with scrollable details
and pinned actions. Distinguish permanent whole-project deletion, confirmed saved
environment removal, and draft-only mapping removal. Keep the last environment
and mapping; retain frozen History when removing an environment. History actions
say **Delete run… / Delete baseline…**. No generic reset, bulk erase or archive.
Locked/foreign credentials block deletion, not silently orphan private state.
Committed deletion with incomplete file cleanup reports that distinction; App
settings shows pending cleanup and engine startup keeps other projects usable.
Scope, app-wide lifecycle review, offscreen inspection and qualification limits:
`docs/research/project-removal-20261018.md`.

## Do's and Don'ts

- **Do** preserve evidence versus settings distinctions and disclose missing work.
- **Do** use shared native primitives and named semantic tokens for future screens.
- **Do** keep filesystem, database, process and image I/O off the UI thread.
- **Do** preserve project context, drafts, approval consequences and retained history.
- **Don't** present synthetic fixture screenshots as actual project results.
- **Don't** tint screenshot evidence or replace it with decorative imagery.
- **Don't** call simulated focus/IME tests a native or screen-reader pass.
- **Don't** equate direction approval, source contrast checks or offscreen captures with usability or release approval.
