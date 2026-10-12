---
name: Regressee — Clear wayfinding
description: User-approved direction, source-derived native Rust/GPUI system; native and release qualification pending
colors:
  ground: "#f6fafd"
  surface: "#ffffff"
  rail: "#183f65"
  rail-ink: "#ffffff"
  rail-muted: "#d7e8f5"
  rail-hover: "#23517a"
  ink: "#22313d"
  muted: "#536778"
  line: "#ced9e2"
  control-line: "#7e93a6"
  action-surface: "#f7fafc"
  action-hover: "#e7f2fa"
  accent: "#1765ac"
  accent-hover: "#12558f"
  selected: "#e7f2fa"
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

Make the next safe action obvious. Deep-blue project navigation anchors a bright,
precisely aligned working surface for non-developers reviewing website releases.
The signature is self-labelling task state with baseline consequences attached,
not a developer dashboard, decorative charts or a mobile-app shell.

The user approved this direction and the full workflow brief on 2026-10-11.
This document records the built source system, not approval of its usability,
native accessibility or public release. The chosen HTML sketch is illustrative;
production GPUI source supplies the normative tokens above. The extensions-only
sidecar carries native behavior and geometry, not a browser implementation.

**Key Characteristics:**
- Persistent project context and a clear next safe action.
- Real coverage beside a searchable queue and untinted comparison evidence.
- Deliberate baseline decisions, protected drafts and retained history.

Authority: `PRODUCT.md`, `docs/workflow-revamp.md`,
`.impeccable/surfaces/crates-app-src-main-rs.md` and the selected
`.impeccable/mocks/decision/pick-6e22d8c4.html`. Implementation sources:
`crates/app/src/ui/{theme,controls,text_input,timezone_select}.rs`,
`crates/app/src/{main,workflow,workspace_history}.rs` and
`crates/core/src/shell.rs`. Earlier direction remains in `docs/phase0-design.md`.

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

## Colors

### Primary
Deep-blue `rail` is the navigation anchor; `accent` marks the next task action.
`accent-hover` deepens action hover/press. Sky `selected` identifies the current
navigation, choice or page, never successful evidence by itself. Rail text uses
`rail-ink` and `rail-muted`; unselected navigation hover uses `rail-hover`.

### Neutral
`ground`, `surface` and `action-surface` separate the workspace, editable fields
and secondary actions. `ink` carries readable content; `muted` carries supporting
copy. `line` is a separator, not an interactive boundary. `control-line` outlines
controls; production darkens it from the sketch to meet the source contrast
test's (3:1) threshold against field and secondary-action surfaces. This is not
native accessibility certification. Keep semantic aliases even when values match.

Error and warning ink/surface pairs always accompany explanatory words.
Informational feedback stays neutral white, distinct from selected navigation;
information status labels may use the sky tint.

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

**The Stable State Rule.** Focus, hover, press and selection change paint, never text metrics or control geometry.

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
Review now owns local **Hide navigation / Show navigation** and **Hide results /
Show results** controls. At narrow sizes the project navigation starts collapsed;
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
Inspection result rows are now 84px, with path/query/fragment first and separate
profile and status lines. The full URL remains in a native hover hint and the
selected screenshot's keyboard-accessible Details. Discovery and History keep
their previous row geometry. Authoritative empty captures reserve no blank queue.
Automatic capture-only overview fit is capped at 100%; manual enlargement remains
available. Compact Zoom explicitly distinguishes Fit from Manual.

## Elevation & Depth

Flat tonal layering and precise boundaries, not decorative shadows. GPUI's
zero-blur spread paint provides a functional focus outline, not elevation.
Information and approval dialogs dim the workspace; approval occludes background
interaction. Folder/discard prompts use platform prompts where supported.
No invented motion scale: state changes are immediate. Unknown-duration activity
uses a seamless 1.2-second linear pattern, driven by native animation frames only
while visible, independently of worker updates. This communicates activity, not
completion. `REGRESSEE_REDUCED_MOTION=1` keeps the pattern static; automatic OS
motion-preference integration, theme and high-contrast behavior remain
qualification work.

## Shapes

Use small control corners and straight horizontal list rules; status labels are
slightly tighter. Dialogs share control corners. Draw radio dots, checkbox ticks
and the timezone chevron with native geometry. No nested decorative card stacks
or icon-tile dashboard. No shipping visual assets were added for this revamp.
CAS screenshots are untinted product evidence with provenance—not decorative
rasters, a fixed image library or synthetic demonstration results.

**Local exploration.** Discovery, review and loaded History records filter as
text changes, with truthful match counts/scope and Clear. Ctrl/Cmd+F focuses the
mounted search without escaping a dialog. Filtering never silently changes the
inspected screenshot or discards ignored-area drafts; an outside-filter notice
explains why the pair is still shown. Explicit navigation changes the target.
Viewer reads coalesce independently of mutation work and reject stale identities.
Historical environment context travels with its reference, not today's settings.
History rows support direct selection; storage totals load separately and missing
measurement is not zero. Local syntax/range feedback stays beside touched fields,
while saving, browser visits, baseline changes, exports and deletion stay explicit.

## Components

### Guided setup — Phase 5 independently re-verified in bounded scope

Website → Access → Pages & sizes is pinned above the guided scrolling form, with
Current/Completed/Next/Unavailable state. Public/private choices and the next safe
action fit the minimum window. Clean guided steps Continue without rewriting;
dirty steps await authoritative save before advancement. Private login remains
separate: Save settings first, Open login browser, then I'm ready. The footer
co-locates draft/saved/browser prerequisites. Saving never implicitly opens login;
Save & find pages explicitly authorizes discovery after its matching save.
Existing-project Settings remains explicitly saved with all advanced operations.
Sizes lead Pages, with selected dimensions, extent and image policy in readable
plan lines. Mobile/tablet are website emulation, not real devices. Mapping pairs
lead their scope notes; included-target counts, coverage cause and rediscovery
next step sit beside Capture pages. Photo/bitmap and session-only wording keeps
the secure-store and replacement limitations truthful.

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

**Actions and choices.** Shared actions have a minimum target height (40px).
Buttons fit their labels; navigation fills its column; choice/check rows fill
their available width with balanced `spacing.row` insets even when help wraps.
Checkbox label owners now use a finite flexible column so long labels wrap rather
than clip. The analysis noise-suppression label paints on two lines at both
qualified logical window sizes (270×40px inner text, 326×66px complete control).
Primary keeps its filled blue identity through focus; danger uses error ink and
surface, never primary fill. Secondary hover uses `action-hover`. Focus is distinct
from selection: ordinary actions use ink outlines, rail navigation a white outer
outline with a dark inner border on its selected light surface. Disabled controls
fade, leave the focus model and reject activation. Shared outlines extend (2px)
without changing layout or replacing inherited GPUI text refinements.

**Links.** Presented website/page addresses that offer navigation must be real
links: blue accent ink plus an underline at rest, a pointer cursor, visible
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

**Feedback and progress.** Dismiss belongs inside the feedback surface, centered
beside the message. Long diagnostics scroll within a bounded area; errors retain
their red treatment. Known nonzero totals alone produce a clamped completion
fraction. Unknown/zero totals pair honest text with a smoothly moving patterned
activity track;
discovery shows the actual pages visited. No fabricated percentage or coverage.
Short local reads use a compact activity label instead of a patterned completion
track. Every queued operation names its task immediately, before worker updates.
Projects, screenshots and History distinguish initial loading, authoritative
empty results and failed reads. Opening a project never announces missing
screenshots or an absent baseline before its saved evidence is read; refresh
keeps already loaded evidence visible.

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

**Comparison and approval.** Coverage comes from actual inventory, captures and
current analysis, including missing and unclassified evidence. Preserve before/
after origins, screen size, warnings, sensitivity and ignored-area revisions.
Original screenshots remain untinted; difference, overlay and heatmap are labelled
analysis modes, not brand treatments of the originals.
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
Only comparisons expose Analysis (sensitivity and ignored areas); capture-only
views have no generic Tools mode. A flyout has an attached Close action and Escape
restores its trigger. Drawing an ignored area folds away the flyout and returns
after a completed rectangle, with drafts still requiring explicit save.
Image navigation is direct: drag or two-axis scroll to pan, Shift+scroll for
horizontal movement, Ctrl/Cmd+scroll to zoom at the pointer. Before/after stay
synchronized. A single focus stop provides +/− zoom, arrows/Page Up/Page Down,
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
evidence is occluded; Return to evidence/Escape preserves unsaved note text.
The finite V4 repair gives the inspector scroll body a finite content width;
the actual note input is 326px wide (300px inner content), with caret scrolling
and explicit Save/Return. Saved-project Results loading/failed-read candidates
now permit local navigation restoration without bypassing task/modal guards.
When results are hidden, an evidence-owned path/query/fragment and source profile/
environment heading persists across all modes and capture inspection. Narrow
Compare names Before/After historical environments above their respective panes.
The earlier blocked confirmation and independent FIX verdict remain intact.
The separate 22-pair functional repair evidence received independent bounded
re-verification, not OS-input/accessibility/usability/release acceptance. See
`docs/research/whole-app-ui-review-20261012/phase4-report.md`.

**History, consequences and recovery — Phase 6 independently verified in bounded scope.**
Runs and Baselines use 256px inner scroll viewports with unchanged full-width
selected paint. The outer workspace remains scrollable: at 820×600 the initial
list extends below the fold; scrolling exposes its full four-row viewport.
Overflow help explains scrolling and keyboard reveal, including the last loaded
run/retained version. **Filter this history page** explicitly excludes unloaded
pages. Both Runs and Baselines have page controls, disabled when no page exists.
Readable UTC dates, record types, captured/intended coverage and environment lead;
the selected identity and its open/share/delete actions sit together. Exact IDs
and byte accounting remain under advanced provenance, including Storage.

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
evidence, without a redundant notice banner. An authoritative empty capture
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
Failed task titles retain the operation name. No Open folder, targeted retry or
installer-repair functionality was introduced.

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
