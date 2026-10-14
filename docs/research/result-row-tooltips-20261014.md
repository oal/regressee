# Result-row tooltips — 2026-10-14

## Outcome

The user-reported overlay came from a shared page-row tooltip with a fixed
480px width. Review result rows no longer show an address tooltip: their path
is already visible, and the full address remains in keyboard-accessible Details.
Hover selection styling, row dimensions, navigation and saved evidence are unchanged.

Remaining discovery/page hints use their actual text width plus existing insets,
capped at 480px and the window width. A finite inner width lets long addresses
wrap rather than overflow. Discovery keeps its action explanation, not a URL.
External-link destination and inspection-heading hints are outside this change.

## Bounded verification — PASS

- All 19 shared-control tests pass, including two new regressions for enabled/
  disabled result, discovery and page hover, content-sized hints and long-address
  wrapping. Simulated hover/layout coverage includes 1280×800, 820×600 and a
  320px tooltip-boundary case.
- The production `ui` build passes. Commands use the existing
  `LIBRARY_PATH=$PWD/.runtime/link-libs`; no system dependencies were installed.
- The opt-in `renderer_capture_row_hints` test passes. Eight final Linux GPUI
  offscreen captures cover result hover, short page hints, discovery hints and
  long page hints at both desktop sizes. Short hints measure 241px and 335px,
  not 480px; the long hint wraps within 480×246px. The six repeated short/review
  frames are byte-identical to the first inspection batch. Saved fixture
  evidence is unchanged.

Evidence lives under
`/var/home/olav/Data/opencode/tmp/opencode/regressee-row-hints-20261014-syacVMQY/confirmation/`,
with source-hash sidecars. This is isolated synthetic development-host evidence,
not real-site data, OS-input, accessibility or cross-platform qualification.
No full application-suite pass is claimed.
