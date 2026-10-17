# Regressee — product brief

<!-- impeccable:product-schema 1 -->

Regressee is the user-confirmed app name (2026-10-14). This document records the
original planning decisions and the product-context confirmation on 2026-10-11;
`PLAN.md` contains the implementation strategy and proposed defaults.

As of **2026-10-17**, Regressee is a **public beta** on
[GitHub](https://github.com/oal/regressee), as confirmed by the user. Beta status
does not imply stable-release or complete platform qualification; documented
compatibility and safety limits remain in effect.

## Platform

adaptive

Native desktop application for **Windows, macOS, and Linux**, all required for
the first public release. Respect host-OS conventions for window behavior,
keyboard shortcuts, dialogs, and file locations. This is not a mobile app;
mobile/tablet/desktop refer to the websites being captured.

## Stack

- Rust application, powered by GPUI. These are fixed user requirements.
- Playwright and Node provide browser automation; the app manages their setup.
- The app is fully local. No product account, hosted project storage, or cloud
  collaboration is required.
- Application license: **Mozilla Public License 2.0 (MPL-2.0)**.

## Users

Somewhat tech-savvy non-developers, particularly project managers, checking
websites and admin panels during development and before/after releases.
Developers may use advanced controls, but they are not the audience that the
default workflow should require.

## Product Purpose

Make it practical to capture a whole website as a visual baseline and inspect
what changed after development or deployment, without writing tests, using a
terminal, or manually installing developer tooling.

Typical project size is **100–1,000 distinct pages**, with one or more screen
sizes per page.

## Current Revamp Priorities

Confirmed by the user on **2026-10-11**: the audience, local-only approach, native
Rust/GPUI stack, and existing product scope remain correct. The request is a
**full interface and workflow revamp**, not a change of product or removal of
existing capabilities.

The user identified three connected sources of confusion:

1. **The whole workflow:** choosing a project, setup, finding pages, capturing
   and saving a baseline, and checking for changes do not connect clearly.
2. **Project setup:** website boundaries, capture options, login, and
   environments are overwhelming.
3. **Reviewing changes:** results, screenshots, failures, approvals, and baseline
   history are difficult to understand.

The revamp should make the next safe action clear throughout the workflow,
reduce the burden of initial configuration, and help users understand the
evidence and consequences of review decisions. Preserve the confirmed product
capabilities and safety constraints below. No replacement visual direction was
selected during this product-context interview.

## Positioning

User friendliness for non-programmers is the primary differentiator. The
product should explain crawl boundaries, capture progress, differences, and
failures in ordinary language rather than exposing an automation framework.

## Operating Context

- Public websites, logged-in web apps/admin panels, and staging/internal sites.
- Compare the same site before/after deployment, or compare matching page paths
  between production and staging.
- Users log in and handle cookie consent themselves in an app-managed browser.
  Saved sessions are reused; scripted login and recorded journeys are not part
  of the first release.
- Users can share a ZIP containing a project configuration and a selected
  baseline. Recipients import it and establish their own login sessions.

## Confirmed Capabilities and Constraints

### Projects and crawling

- Multiple local projects.
- A base URL or URL list supplies the starting addresses for discovery.
- Guided controls plus an advanced section. Plain-language controls include a
  page limit, website/section boundaries, skipped URL text, and speed presets.
- A discovered page-list preview before baseline capture.
- User-selectable crawl speed and parallelism.
- One or more mobile, tablet, and desktop capture profiles.
- Full-page screenshots by default; visible-screen-only capture is available.
- A baseline fixes the exact page list. Comparisons revisit only that list;
  they do not automatically discover new pages.

### Capture and comparison

- Users can capture a replacement baseline or compare against an existing one.
- Optional image/photo replacement is chosen before baseline capture. A
  comparison must use the same mode; changing it requires another baseline.
- Balanced default sensitivity, adjustable after capture without recrawling.
- Percentage difference and other change heuristics support result analysis.
- Users can draw ignored rectangles after capture, per page and screen size.
- Users can approve selected page/screen-size results into a new baseline
  version, or explicitly replace the whole baseline.
- Prior baseline versions remain available until the user deletes them.

### Installation, data, and sharing

- Automatically download the app's private Node, Playwright, and browser
  runtime on first use. Internet is required for that setup.
- No reliance on a system Node installation or developer package manager.
- Projects, screenshots, and review data remain local.
- ZIP export includes project settings and a selected baseline, not login
  sessions or cookies. Screenshots themselves can contain private information.
- Keep history until the user explicitly deletes it. Show storage usage,
  low-space warnings, and a cleanup workflow.

## Brand Commitments

- The interface must look **sleek and modern**.
- Visual polish must support clarity and confidence rather than make the app
  feel like a developer dashboard.
- On **2026-10-11**, the user selected **Clear wayfinding** for the replacement
  interface: deep-blue project navigation, white working surfaces, sky-tinted
  selection, readable system UI typography, and a clear next safe action. The
  user confirmed the workflow brief in `docs/workflow-revamp.md`. This is approval
  of the direction and brief, not native usability or release qualification.
- On **2026-10-14**, the user supplied light/dark-background logo variants and
  confirmed that the icon must accompany **Regressee**, never replace its name.
  That palette adaptation kept the existing layout and white workspaces,
  with navy navigation, readable teal primary actions and pale mint selection.
  Originals live in `assets/`; cropped, optimized embedded copies and provenance
  live in `assets/brand/`. That approval did not request a full dark workspace theme.
- On **2026-10-16**, the user approved an **app-wide appearance extension** of
  Clear wayfinding, not a redesign: follow the live desktop preference by default,
  with an immediately applied, remembered **System / Light / Dark** choice in
  App settings. This supersedes the earlier white-workspace-only scope while
  preserving its light palette, system type, geometry and background-appropriate
  icon/name lockups. Appearance is machine-local, stored independently in
  `app-preferences.json`, not project settings or baseline ZIPs. Read/write failures
  must describe the fallback or session-only choice truthfully. Website capture-theme
  options and screenshot pixels remain unchanged; no new dependency, animation or
  shipping raster is introduced. Bounded evidence and outstanding qualification:
  `docs/research/dark-mode-20261016.md`.
- On **2026-10-17**, the user rejected the actual native app's slate workspace /
  blue sidebar pairing and gradient header, and selected **Unified neutrals**.
  The current palette uses a continuous charcoal shell in dark mode and
  off-white/soft-gray layers in light mode; teal/mint is reserved for action and
  selection, with warning/error semantics retained. Headers use solid surfaces,
  not gradients. This supersedes the earlier navy-rail, blue-charcoal and
  gradient-chrome direction, not its dated approvals or verification records.
  It is a local Clear wayfinding refinement (inherited seed `6e22d8c4`), not a
  redesign: preserve layout, system type, control geometry, original screenshots,
  drafts, consent and the existing logo assets. Both navigation and About choose
  the supplied logo appropriate to the active light/dark background. System /
  Light / Dark following and independent atomic preferences remain unchanged;
  no dependency, motion or shipping raster is added. Production build and focused
  palette/appearance checks passed; five broader shell failures remain unattributed,
  so the whole suite is not a pass. Current-source native confirmation passed;
  fresh review returned **SHIP for the neutral palette/chrome refinement only**,
  not whole-app or release approval. Earlier scores and source contrast alone
  do not override this feedback. Scope and qualification limits:
  `docs/research/theme-coherence-20261017.md`.

## Product Principles

1. Ordinary language, sensible defaults, and a clear next safe action first;
   advanced controls remain available without dominating the experience.
2. Do not confuse a visual difference with a confirmed defect, or a failed
   capture with an unchanged page.
3. Preserve evidence and history; changing a baseline is deliberate and
   reversible while its prior version remains stored.
4. Be transparent about capture coverage, ignored areas, and comparability.
5. Keep automation local and sharing explicit; never export authentication.

## Evidence on Hand

The project directory was empty at the original planning interview, but that is
no longer the current state. The maintained native application lives in
`crates/gui`, with its GPUI shell in `crates/gui/src/main.rs`, shared controls in
`crates/gui/src/ui`, GUI-independent application services in `crates/app`, and
navigation/focus rules in `crates/core/src/shell.rs`. Its executable is
`regressee-desktop`; `regressee` is currently a separate CLI skeleton, not a
replacement product workflow.

`README.md` and `docs/first-run-guide.md` describe the implemented workflows:
guided discovery, authenticated capture, separate environments, fixed-baseline
checks, result analysis and ignored regions, selective approvals, history,
baseline ZIP sharing, browser setup, and manual storage cleanup. These are
implementation evidence, not proof of usability or public-release readiness.

`DESIGN.md` records the built **Clear wayfinding** replacement system. The user
approved the direction and workflow brief, not a native-usability acceptance
claim. Actual GPUI offscreen captures now demonstrate the development-host
layouts; earlier automated OS-window captures were black, and the first-run
guide separately records the user's successful plain launch/project creation.
Native input, accessibility, cross-platform and release qualification must not
be inferred from offscreen, backend or simulated tests.

The current user explicitly reports a confusing interface and requests a full
revamp, with the priorities recorded above. This is direct usability feedback,
not a formal usability study. Repository fixture screenshots are synthetic
evidence, not real project results or an approved visual reference.

## Open Decisions

These are implementation/design decisions, not unanswered core product scope:

- Exact supported OS versions, CPU architectures, and Linux package format.
- Specific pinned GPUI, Rust, Node, Playwright, and Chromium versions.
- Remaining native usability/accessibility qualification of the selected visual
  design and GPUI component behavior.
- Calibrated comparison preset values, informed by test fixtures.
- Precise supported image-replacement cases and limitations.
- Release artifact hosting.

`PLAN.md` assigns these decisions to explicit phase gates. No deadline or
additional distribution constraint was specified.
