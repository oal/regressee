---
name: Regressee foundation proposal
description: Native operational visual system; human review and rendered qualification pending
colors:
  primary: "#08786f"
  primary-hover: "#05645c"
  selected: "#d9f0ec"
  ground: "#f6f8f9"
  surface: "#ffffff"
  rail: "#edf2f4"
  ink: "#183138"
  muted: "#526970"
  divider: "#c8d4d8"
  control-border: "#87999f"
  action-surface: "#e2e9ec"
  action-hover: "#d3dfe3"
  error: "#9c3038"
  error-surface: "#fff2f2"
  warning: "#775311"
  warning-surface: "#fff7df"
typography:
  headline:
    fontFamily: "Segoe UI, .AppleSystemUIFont, DejaVu Sans"
    fontSize: "28px"
    fontWeight: 600
  title:
    fontSize: "21px"
    fontWeight: 600
  body:
    fontSize: "15px"
    fontWeight: 400
  field:
    fontSize: "16px"
    fontWeight: 400
  action:
    fontSize: "14px"
    fontWeight: 500
  label:
    fontSize: "13px"
    fontWeight: 400
rounded:
  status: "3px"
  control: "4px"
  dialog: "6px"
spacing:
  tight: "4px"
  related: "8px"
  row: "12px"
  group: "16px"
  section: "24px"
  inset: "32px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.surface}"
    rounded: "{rounded.control}"
    padding: "8px 12px"
    height: "40px minimum"
  button-secondary:
    backgroundColor: "{colors.action-surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    padding: "8px 12px"
  navigation-selected:
    backgroundColor: "{colors.selected}"
    textColor: "{colors.primary-hover}"
    rounded: "{rounded.control}"
  field:
    backgroundColor: "{colors.surface}"
    textColor: "{colors.ink}"
    rounded: "{rounded.control}"
    padding: "4px 12px"
---

# Design System: Regressee foundation proposal

## Overview

**Proposal, not user-approved.** This records the maintained GPUI source system,
not a rendered/accessible/native-qualified design. Human review of representative
setup and results surfaces is pending. The earlier provisional bench record is
preserved verbatim in `docs/phase0-design.md`; decision 0005 allows local work
without pretending its visual gate passed.

**Creative North Star: "A clear working desk" — proposed.** A project manager
preparing a release in an ordinary well-lit office needs a bright, legible working
surface, not a dark developer console. Modernity comes from exact alignment,
confident type and clear states, not decorative charts, animation or fake imagery.
Operate mode permits native system UI type. `docs/foundation-surface.md` holds
the screen-specific topology, seed direction and flows; it is not brand approval.

**Key Characteristics:**
- Readable local task state, one primary action per task.
- Horizontal list structure and restrained tonal navigation.
- Ordinary language and honest incomplete-feature/evidence states.

## Colors

Cool near-white working surfaces, deep blue-green ink and one teal action accent.
Primitives are extracted from `crates/app/src/ui/theme.rs`.

### Primary
Teal identifies actions and current selection. The deeper variant carries hover
and selected text; pale teal marks selection. Keyboard focus uses a separate ink
outline, never a typography or geometry change.

### Neutral
Ground and rail establish navigation/content separation. Ink is body text,
muted is secondary copy, divider separates groups, and control-border outlines
interactive fields/buttons. Error and warning have both ink and pale surfaces;
their words explain the state without relying on color.

**The Evidence Rule.** A change classification is never rendered as a confirmed
defect, and missing screenshots are never rendered as an unchanged result.

## Typography

One installed UI family carries task headings, body and controls. The frontmatter
family list is an OS inventory, not a browser fallback stack: code requests Segoe
UI on Windows, .AppleSystemUIFont on macOS and DejaVu Sans on this Linux target.
Windows/macOS font resolution is not qualified here. No font assets are bundled.

Headings are semibold, actions medium, prose regular. No decorative display type,
uppercase kickers or technical monospace labels. Body explanations are bounded
to about 660–680 logical pixels; exact wrapping at desktop scale remains a visual
review obligation, not a passing screenshot assertion.

## Layout

The product shell uses a 204px navigation rail and 32px working inset. The initial
window is 1120×800, minimum 820×600; OS window resize conventions remain native.
Project context and feedback sit above the scrolling content. Setup's explicit
Save settings action sits in a pinned bottom bar. Form width is capped at 680px,
and project lists
are paged to 64 rows. There is no mobile product layout: website capture profiles
are not the devices on which Regressee runs. Large result virtualization arrives
with actual result data, rather than rendering thousands of placeholders now.

## Elevation & Depth

Flat by default, tonal rail separation and ordinary boundaries. No decorative
shadows or animated entrance. The information dialog has a dimmed background;
discard confirmation and folder selection use GPUI's native platform prompts
where supported. Reduce-motion behavior is immediate because this shell does
not animate. OS theme/high-contrast integration remains qualification work.

## Shapes

Small control corners, slightly larger dialog corners, straight list rules. No
nested cards or icon-tile grids. There is no shipping icon or raster asset corpus.

## Components

`crates/app/src/ui/controls.rs` supplies action variants, drawn radio choices and
checkboxes, section headings, status, error, empty, form field, list row, dialog
and progress. `text_input.rs` is an attributed Apache-2.0 native input, not a web
field. Primary/secondary/danger/navigation/choice actions share stable 40px-minimum
targets, 14px medium text and fixed 20px line height. Focus/hover/press refinements
change paint only. Focus remains distinct from selection and preserves primary
teal styling. Destructive actions use error ink rather than primary fill.
Disabled actions lose focus/click
activation and fade; inputs are read-only during storage work. Actions fit their
labels instead of stretching to field width; navigation alone fills its column.
Secondary actions use a cool neutral fill, while editable fields stay white and
full-width. Choice rows retain full width and balanced 12px insets, including
below wrapped help. Dismiss is inside its feedback surface; long messages scroll
within a bounded area rather than consuming the working viewport. These are
source changes responding to user screenshots, not a native acceptance claim.
Unknown-duration
work uses textual progress, never an invented percentage.

Tab/Shift-Tab follow the active screen's explicit focus model. Enter/Space
activate buttons; spaces remain input text. Ctrl/Cmd+N creates/visits setup,
Ctrl/Cmd+S saves, Ctrl/Cmd+1…5 switches sections, Ctrl/Cmd+Q quits. Clipboard,
IME, Home/End, anchored Shift selection and pointer selection are scoped to
fields. Double-click word selection is a basic fallback, not locale-aware
segmentation. Escape closes the
information dialog and restores focus; it does not falsely cancel a committed
SQLite save. Save must be clicked/shortcutted explicitly; Enter in an input does
not accidentally submit. Unsaved switches request explicit discard confirmation.

## Do's and Don'ts

- **Do** preserve evidence versus settings distinctions and disclose missing work.
- **Do** use the shared primitives and named semantic tokens for future screens.
- **Do** keep filesystem, database, process and image I/O off the UI thread.
- **Don't** present synthetic fixture screenshots as actual project results.
- **Don't** call simulated focus/IME tests a native or screen-reader pass.
- **Don't** treat this proposal or source contrast checks as human visual approval.

Not canonized: native rendering/input, accessibility semantics, high-contrast /
dark-theme behavior, resized wrapping and quit-with-unsaved-edit behavior remain
open. The 2026-10-10 UI sweep's source/test-platform repairs passed independent
checks, but its isolated X11/software capture was entirely black. No native
screen/input or hierarchy verdict is invented; the user's working plain launch
is separate evidence. See `docs/research/ui-sweep-reverification-20261010.md`.
