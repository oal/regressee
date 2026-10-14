---
name: Regressee — Clear wayfinding
description: User-approved direction, source-derived native Rust/GPUI system; native and release qualification pending
colors:
  ground: "#f5f9fa"
  surface: "#ffffff"
  rail: "#082b50"
  rail-ink: "#ffffff"
  rail-muted: "#d1e4f2"
  rail-hover: "#123d65"
  ink: "#1b3044"
  muted: "#506579"
  line: "#cddcdf"
  control-line: "#7b919f"
  action-surface: "#f4f8fa"
  action-hover: "#e2f5f0"
  accent: "#007970"
  accent-hover: "#00685f"
  selected: "#e2f5f0"
  error: "#9c3038"
  error-surface: "#fff2f2"
  warning: "#775311"
  warning-surface: "#fff7df"
typography:
  headline:
    fontFamily: "Segoe UI, .AppleSystemUIFont, DejaVu Sans"
    fontSize: "28px"
    fontWeight: 600
  review-heading:
    fontSize: "24px"
    fontWeight: 600
  title:
    fontSize: "21px"
    fontWeight: 600
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
    textColor: "{colors.surface}"
    typography: "{typography.action}"
    rounded: "{rounded.control}"
    padding: "8px 12px"
  button-primary-hover:
    backgroundColor: "{colors.accent-hover}"
    textColor: "{colors.surface}"
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
    textColor: "{colors.rail}"
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
---

# Design System: Regressee — Clear wayfinding

## Overview

**Creative North Star: "Clear wayfinding"**

Make the next safe action obvious. Navy project navigation anchors a bright,
precisely aligned working surface for non-developers reviewing website releases.
The signature is self-labelling task state with baseline consequences attached,
not a developer dashboard, decorative charts or a mobile-app shell.

The user approved this direction and the full workflow brief on 2026-10-11.
This document records the built source system, not approval of its usability,
native accessibility or public release. The chosen HTML sketch remains an
illustrative composition reference, not authority for the approved 2026-10-14
palette; production GPUI source supplies the normative tokens above. The
extensions-only sidecar carries native behavior and geometry, not a browser
implementation.

**Logo and palette adaptation — 2026-10-14.** The user approved navy navigation,
readable teal actions, pale mint selection and white workspaces, with supplied
background-appropriate icons alongside the readable Regressee name. This narrow
extension preserves structure, workflows, system UI fonts, error/warning roles,
focus and immediate state changes; screenshot originals remain untinted. It does
not introduce a full dark workspace theme.

**Key Characteristics:**
- Persistent project context and a clear next safe action.
- Real coverage beside a searchable queue and untinted comparison evidence.
- Deliberate baseline decisions, protected drafts and retained history.

Authority: `PRODUCT.md`, `docs/workflow-revamp.md`,
`.impeccable/surfaces/crates-app-src-main-rs.md` and the selected
`.impeccable/mocks/decision/pick-6e22d8c4.html`. Implementation sources:
`crates/app/src/ui/{brand,theme,controls,text_input,timezone_select}.rs`,
`crates/app/src/{main,workflow,form_recovery,workspace_history}.rs` and
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

## Colors

### Primary
Navy `rail` is the navigation anchor; readable teal `accent` marks the next task
action. `accent-hover` deepens action hover/press. Pale mint `selected` identifies the current
navigation, choice or page, never successful evidence by itself. Rail text uses
`rail-ink` and `rail-muted`; unselected navigation hover uses `rail-hover`.
The artwork's luminous turquoise is not a white-text action fill; source uses
the darker teal accent for readability.

### Neutral
`ground`, `surface` and `action-surface` separate the workspace, editable fields
and secondary actions. `ink` carries readable content; `muted` carries supporting
copy. `line` is a separator, not an interactive boundary. `control-line` outlines
controls; production darkens it from the sketch to meet the source contrast
test's (3:1) threshold against field and secondary-action surfaces. This is not
native accessibility certification. Keep semantic aliases even when values match.

Error and warning ink/surface pairs always accompany explanatory words.
Informational feedback stays neutral white, distinct from selected navigation;
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

**The Stable State Rule.** Focus, hover, press and selection change paint, never text metrics or control geometry.

Screenshot switches keep the selected path/profile heading and inspection toolbar
mounted during the matching image read; zoom stays disabled until pixels arrive.

## Layout

This is a resizable native desktop application. Default window (1280×800 logical
px), minimum (820×600); fixed project rail (192px), working horizontal inset
(24px). Setup forms cap at (680px), with an explicit pinned save footer.
The pinned shared header names **Selected project** and offers **Change project**.
Results and History offer an in-place chooser when no project is selected.
Changing projects preserves the destination and confirms draft discard; reopening
the same project must not reload away its draft. Projects separates **View results**
from **Edit settings** and identifies the selected project with words and paint.

Comparison review remains side by side, including small desktop windows: queue width (200px)
below viewport width (1100px), otherwise (272px), separated from the flexible
viewer by (12px). Do not describe a stacking layout that is not implemented.
The logo/name lockup is always the first item in the project rail, at the same
top position across sections. Its navigation-only title wrapper has padding
above and below (12px each) inside the existing rail; the mark and name are
centered on separate rows. Whole-navigation fit verification for this larger
stack is deferred as recorded above. The review header owns **Hide sidebar / Show
sidebar** in one consistent location, with a hint explaining the extra screenshot
space. This control is absent from project choosers, where navigation stays
visible. Review also owns local **Hide results / Show results** controls.
At narrow sizes the project navigation starts collapsed;
an explicit workspace preference survives resize. Selected project context remains
visible. Working geometry uses the actual mounted rail width, not a permanent
192px reservation. Ctrl/Cmd+F restores a hidden queue before focusing its search.
Compare remains the default at both sizes. **Before/After (full width)** is an
explicit, untinted one-original alternative with a named current side and shared
source pan/zoom; switching sides preserves manual scale and target. A separate
View chooser exposes Compare, Difference, Overlay, Heatmap and this alternative.
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

Discovery and captured-target lists use virtualized, full-width rows (64px) with
aligned horizontal insets (16px). Address and details truncate independently;
selection and hover preserve row height. Lists fill their bounded viewport, not
a fixed five-row panel. Zero search matches show an explanation, not an empty
bordered list. Capture profiles describe mobile **web emulation**, tablet and
desktop websites—not app devices or a mobile product layout.
Discovery selection opens **Page options** immediately, with the complete wrapped
address, capture status and explicit Include/Exclude action. **Previous page** and
**Next page** inspect these details and reveal the selected row in the list; they
never change inclusion themselves. A row's hover hint explains this action instead
of repeating its URL. Discovery-blocked pages retain their reason and explain the
settings/rediscovery route, without enabling an unsafe inclusion override.
Inspection result rows are now 84px, with path/query/fragment first and separate
profile and status lines. Result rows have no redundant address tooltip obscuring
the screenshots; the full URL remains in the selected screenshot's
keyboard-accessible Details. Remaining page-list hints fit their content, capped
at 480px and the window width rather than reserving a fixed width. Discovery and
History keep their previous row geometry. Authoritative empty captures reserve
no blank queue.
Automatic capture-only overview fit is capped at 100%; manual enlargement remains
available. Compact Zoom explicitly distinguishes Fit from Manual.

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

## Elevation & Depth

Flat tonal layering and precise boundaries, not decorative shadows. GPUI's
zero-blur spread paint provides a functional focus outline, not elevation.
Information and approval dialogs dim the workspace; approval occludes background
interaction. Folder/discard prompts use platform prompts where supported.
No invented motion scale: state changes are immediate. Unknown-duration activity
uses a seamless 1.2-second linear pattern for long tasks and a rotating native
arc for compact reads, driven by native animation frames only while visible,
independently of worker updates. This communicates activity, not completion.
`REGRESSEE_REDUCED_MOTION=1` keeps both indicators static; automatic OS
motion-preference integration, theme and high-contrast behavior remain
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

The noninteractive lockup pairs `assets/brand/logo-dark.png` with **Regressee**
on the navy rail, and `assets/brand/logo-light.png` with **About Regressee** on
white. The mark supplements, never replaces, the name. Navigation uses
`navigation_lockup()`: a larger proportional mark (96px wide, approximately
58.8px high) with matching height reservation, centered above the name with the
related gap (8px) and `brand-wordmark` system type in `rail-ink`. About keeps the
horizontal `lockup()`, logo width (36px), proportional painted height
(approximately 22px), height reservation (24px), related gap (8px) and existing
`title` system type in `ink`. Navigation retains the existing rail width and
About the existing dialog structure. Derived PNGs are transparent RGBA
(294×180px), with shared padding crop and Lanczos downsampling only; no
recoloring or artwork changes.
Compile-time embedded bytes and cached `Arc<Image>` identities let GPUI decode
and cache off-thread; no filesystem/network lookup or image decoding occurs in
the render callback.

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

**Actions and choices.** Shared actions have a minimum target height (40px).
Buttons fit their labels; navigation fills its column; choice/check rows fill
their available width with balanced `spacing.row` insets even when help wraps.
Checkbox label owners now use a finite flexible column so long labels wrap rather
than clip. The analysis noise-suppression label paints on two lines at both
qualified logical window sizes (270×40px inner text, 326×66px complete control).
Primary keeps its filled teal identity through focus; danger uses error ink and
surface, never primary fill. Secondary hover uses `action-hover`. Focus is distinct
from selection: ordinary actions use ink outlines, rail navigation a white outer
outline with a dark inner border on its selected light surface. Disabled controls
fade, leave the focus model and reject activation. Shared outlines extend (2px)
without changing layout or replacing inherited GPUI text refinements.
Keyboard reveal measures complete bordered choice/check/action rows, not their
padded content alone. Disclosure focus handles are distinct from project rows.

**Links.** Presented website/page addresses that offer navigation must be real
links: teal accent ink plus an underline at rest, a pointer cursor, visible
keyboard focus and Tab/Enter activation. Do not rely on color or hover alone, and
never make ordinary noninteractive text look like a link. External page links
open the OS default browser only on activation; capture-session credentials are
not transferred. Truncated addresses disclose the full destination on hover.
Only complete HTTP(S) addresses without embedded credentials are openable;
unsupported schemes remain plain data. Editable URL fields, literal code and raw
diagnostics are not automatically linkified. Task context and gap links share
this treatment and scroll into view when reached by keyboard.

**Fields and timezone.** Native input stays white when focused, with accent focus
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
resembles a list bullet. Global operations use a white, thin-bordered status
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
than separating numbers from labels. Small native-drawn stroke icons always sit
beside words. Attention is amber, actual failures are red, saved captures use
normal ink, and zero counts stay muted. A saved-screenshot checkmark never means
baseline approval. Screenshot progress accounts for processed page/profile
targets, including failures, not successful captures or an invented ETA.
An information note stays white and explains the relevant operation/state;
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
evidence or no next matching target. Empty baseline selection reads **0 selected**;
nonempty selection names references ready for update without changing eligibility.
The protected approval dialog has a bounded height (520px) and widths (740px)
at narrow desktop sizes or (760px) otherwise, capped by its available viewport.
It asks “Update the baseline for 1 screenshot?” with correct plural forms, names
the future-check consequence and retained versions, and distinguishes Replace
from Keep. One scroll owner contains the inspected path/profile/source environment/
UTC time, on-demand exact provenance and the reference rows; actions remain pinned.
Rows grow with long addresses rather than painting separators through text.
At most 32 reference rows mount on a page. Previous/Next inspects every reference
and advances the visible page, keeping all unchanged references accessible.
Exact page/profile IDs, run/artifact identities and requested/final URLs remain
available, including the technical tail. Keep reviewing/Escape dismisses without approving;
Confirm baseline is explicit. Each new or cached consent opening resets the
inspected reference, exact-details disclosure and actual reading scroll to the
top, so consequences cannot disappear behind a previous tail inspection/resize.
Focus is trapped, background mutations are guarded,
and stale previews are invalidated. Failed/missing/incompatible evidence cannot
be approved; replacing the whole baseline remains a separate workflow.

Ordinary review exposes revision-bound decisions, explicit note saving, progress
and Next unreviewed without opening Analysis. Classification is not approval:
**Select for baseline update** names reference selection explicitly. Discovery
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
beside Zoom opens an explicit, bounded flyout with readable gesture/shortcut rows
and an attached Close action; Escape restores its trigger, and an outside click
closes without taking focus from the clicked control. At narrow widths the help
action lives at the end of the Zoom menu rather than adding a toolbar row. One
muted drag/zoom hint shares the existing caption only when there is room; no new
strip reduces evidence height. macOS names Command, other hosts name Ctrl. Help
shares the mutually exclusive inspector lifecycle, preserves drafts/scale/target,
and never triggers a capture or baseline operation.
A single focus stop provides +/− zoom, arrows/Page Up/Page Down,
0 for fit width and 1 for actual size. One labelled Zoom menu combines fit width,
actual size (100%) and percentage presets at both desktop widths. Its desktop
trigger shows “Zoom: Fit width · 52%” in automatic mode and “Zoom: 75%” after manual zoom;
a drawn chevron makes the selection affordance explicit. Fit follows resizing;
fixed percentages do not. Arrows/Home/End navigate the menu, Enter chooses,
Escape dismisses, and Tab continues the shell order without trapping focus.
Small windows stack “Zoom” and the scale inside the same compact 40px control;
the selected mode remains explicit in its menu, without adding toolbar rows.
Missing images have no useless zoom/pan controls. A hand/crosshair cursor conveys
the current interaction.
Compact no-match recovery precedes the search controls, with expanded
filters scrolling separately.

Phase 4 separates the selected disposition group from Review note and the primary
Next unreviewed action. Baseline selection/preview remains a separate footer.
Notes now use the owned temporary editor rather than a permanent evidence-height
row. Details, notes and filters remove or disable review/baseline decisions while
evidence is occluded; Close/Escape preserves unsaved note text without saving it.
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

The user-requested carry-through confirms the previously incomplete History,
Tools, empty-message/help and capture-only corrections in fresh source-bound
pixels at 1280×800 and 820×600. See
`docs/research/ui-carry-through-verification-20261011.md`; this is offscreen-layout
acceptance, not native input, accessibility or public-release certification.

## Do's and Don'ts

- **Do** preserve evidence versus settings distinctions and disclose missing work.
- **Do** use shared native primitives and named semantic tokens for future screens.
- **Do** keep filesystem, database, process and image I/O off the UI thread.
- **Do** preserve project context, drafts, approval consequences and retained history.
- **Don't** present synthetic fixture screenshots as actual project results.
- **Don't** tint screenshot evidence or replace it with decorative imagery.
- **Don't** call simulated focus/IME tests a native or screen-reader pass.
- **Don't** equate direction approval, source contrast checks or offscreen captures with usability or release approval.
