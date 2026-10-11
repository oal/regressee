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

Qualification boundary: the final independent verdict is **fix**: review geometry,
attached approval and indexed coverage are resolved, but the reference's source
environment still reads “environment not loaded” in the default review workspace.
The complete 34-image renderer confirmation passed; this is not full UI acceptance.
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

Review remains side by side, including small desktop windows: queue width (200px)
below viewport width (1100px), otherwise (272px), separated from the flexible
viewer by (12px). Do not describe a stacking layout that is not implemented.
Queue, image pane and viewer tools are independently bounded/scrolled; optional
capture details have their own bounded scroll region (100px). Review does not
use the ordinary full-screen scrolling form. Project context, coverage and the
conditional approval bar remain attached to the task.

Discovery and captured-target lists use virtualized, full-width rows (64px) with
aligned horizontal insets (16px). Address and details truncate independently;
selection and hover preserve row height. Lists fill their bounded viewport, not
a fixed five-row panel. Zero search matches show an explanation, not an empty
bordered list. Capture profiles describe mobile **web emulation**, tablet and
desktop websites—not app devices or a mobile product layout.

## Elevation & Depth

Flat tonal layering and precise boundaries, not decorative shadows. GPUI's
zero-blur spread paint provides a functional focus outline, not elevation.
Information and approval dialogs dim the workspace; approval occludes background
interaction. Folder/discard prompts use platform prompts where supported.
No invented motion scale: state changes are immediate, and unknown-duration
activity advances only with real work updates, not a timer-driven animation.
OS theme/high-contrast behavior remains qualification work.

## Shapes

Use small control corners and straight horizontal list rules; status labels are
slightly tighter. Dialogs share control corners. Draw radio dots, checkbox ticks
and the timezone chevron with native geometry. No nested decorative card stacks
or icon-tile dashboard. No shipping visual assets were added for this revamp.
CAS screenshots are untinted product evidence with provenance—not decorative
rasters, a fixed image library or synthetic demonstration results.

## Components

**Actions and choices.** Shared actions have a minimum target height (40px).
Buttons fit their labels; navigation fills its column; choice/check rows fill
their available width with balanced `spacing.row` insets even when help wraps.
Primary keeps its filled blue identity through focus; danger uses error ink and
surface, never primary fill. Secondary hover uses `action-hover`. Focus is distinct
from selection: ordinary actions use ink outlines, rail navigation a white outer
outline with a dark inner border on its selected light surface. Disabled controls
fade, leave the focus model and reject activation. Shared outlines extend (2px)
without changing layout or replacing inherited GPUI text refinements.

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
fraction. Unknown/zero totals pair honest text with a patterned activity track;
discovery shows the actual pages visited. No fabricated percentage or coverage.

**Comparison and approval.** Coverage comes from actual inventory, captures and
current analysis, including missing and unclassified evidence. Preserve before/
after origins, screen size, warnings, sensitivity and ignored-area revisions.
Original screenshots remain untinted; difference, overlay and heatmap are labelled
analysis modes, not brand treatments of the originals.
The protected approval dialog has a bounded height (420px) and widths (580px)
at narrow desktop sizes or (760px) otherwise, capped by its available viewport.
It states selected references replaced versus inherited references kept, retained
earlier versions and composite capture times. A virtualized reference list plus
scrollable exact provenance exposes page/profile, source run, capture time,
original artifact and requested/final URLs where recorded. Previous/Next supports
keyboard provenance inspection. Keep reviewing/Escape dismisses without approving;
Confirm baseline is explicit. Focus is trapped, background mutations are guarded,
and stale previews are invalidated. Failed/missing/incompatible evidence cannot
be approved; replacing the whole baseline remains a separate workflow.

**History and keyboard.** Baseline versions and capture/check runs are distinct,
with active versus retained reference state and composite provenance explained.
Keep ZIP privacy previews, separate recipient login, storage totals, shared-source
protections and deliberate deletion; ignored areas are not pixel redaction.
Tab/Shift-Tab traverse only mounted, enabled controls. Enter/Space activate actions
without intercepting spaces in inputs. Ctrl/Cmd+N opens new-project setup,
Ctrl/Cmd+S saves, Ctrl/Cmd+1…5 switches sections and Ctrl/Cmd+Q quits. Dialog
dismissal restores focus; Escape never pretends to cancel a committed database
save. Unsaved switches require explicit discard confirmation. Quit-with-unsaved
edits and real OS input remain qualification obligations.

## Do's and Don'ts

- **Do** preserve evidence versus settings distinctions and disclose missing work.
- **Do** use shared native primitives and named semantic tokens for future screens.
- **Do** keep filesystem, database, process and image I/O off the UI thread.
- **Do** preserve project context, drafts, approval consequences and retained history.
- **Don't** present synthetic fixture screenshots as actual project results.
- **Don't** tint screenshot evidence or replace it with decorative imagery.
- **Don't** call simulated focus/IME tests a native or screen-reader pass.
- **Don't** equate direction approval, source contrast checks or offscreen captures with usability or release approval.
