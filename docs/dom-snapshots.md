# DOM snapshot checks

DOM snapshots check page structure and text, independently of appearance. They
record the **live DOM after JavaScript runs**, not the server's original HTML.
Screenshots remain the default; existing projects do not enable DOM automatically.

## Choose what to capture

In the capture plan, choose **What to check** and save the settings:

| Mode | Saved evidence |
| --- | --- |
| Screenshots only | The existing visual comparison workflow. |
| Screenshots + DOM snapshots | Independent visual and normalized-DOM checks. |
| DOM snapshots only | Normalized DOM without screenshots or image derivatives. |

Screen sizes still matter: responsive scripts can produce different DOM at
different widths. Capture and explicitly save a new baseline to change its mode.
Checks of an older baseline keep its frozen mode, page list and capture profiles,
even after you edit future capture settings. Missing historical DOM is never
manufactured from a screenshot.

## What is included

- Main document, template contents and accessible open shadow roots.
- Elements, namespaces, sorted attributes and text in document order. IDs,
  classes, ordinary URLs, ARIA attributes, hidden text and text whitespace remain.
- Explicit shadow/template boundaries; slot content is not duplicated.

Iframe documents, closed shadow roots, computed styles, canvas pixels, JavaScript
memory and unmounted/virtualized content are not captured. Closed roots cannot
reliably be counted. The snapshot is a bounded observation, not complete website
coverage. In both mode, DOM is sampled before screenshot-only page transformations;
the two captures are nearby observations, not an atomic browser state.

Capture limits are 8 MiB canonical UTF-8, 100,000 visited nodes, depth 512,
256 attributes per element and 64 KiB per string. Exceeding a limit fails that
check instead of silently truncating it. DOM stability is sampled independently
of screenshot stability; an unstable snapshot is retained but cannot establish
an unchanged result or become a baseline reference.

## Privacy

Before snapshots leave the page collector, capture excludes script bodies
(including hydration/JSON data), inline event code, nonces, form values and
selection state, textarea contents, iframe `srcdoc` and executable `javascript:`
URL attributes. Import and ingestion validate the canonical structure again.
No raw unfiltered HTML backup is retained.

**This is not complete redaction.** Text, hidden content, URLs, styles, metadata
and ordinary attributes can still contain personal data or secrets. Review
snapshots, reports and export previews before sharing. Screenshot ignored
rectangles neither ignore DOM changes nor redact either original. Login sessions,
cookies and secret keys are not included in baseline ZIPs.

## Review and approve

With both checks enabled, use **Screenshots / DOM snapshot** in the evidence
workspace. DOM-only opens source directly, without image zoom, masks or sensitivity.
The source viewer is native, read-only text—not a web page. It offers line numbers,
`+`/`−` changes, collapsed context, source search, change navigation, selection and
copy. Before/After source inspection remains available when a comparison is absent
or detailed diff output reaches its limit.

DOM **Identical** means complete compatible normalized captures match exactly.
Screenshot sensitivity and ignored rectangles do not affect it. A failure or
missing/incompatible/unstable capture stays explicit, even if its sibling passes.
Line counts describe source lines, not semantic or element-change counts.

Review notes and decisions belong to the selected kind and immutable comparison.
Selecting **After screenshot** does not select **After DOM snapshot**, or vice
versa. Preview the update and explicitly confirm either kind or both. A fresh
preview can accept the remaining kind from the same check after a sibling update,
but only while its original Before reference is unchanged. Old versions remain
in History. Reviewing or switching tabs never approves anything automatically.

A partial baseline records which intended checks were omitted. Future checks
visit only its saved reference slots; omitted DOM is not called disabled or
unchanged. Use a new baseline capture to fill missing coverage.

## Saved evidence, sharing and automation

Reanalysis uses retained originals without revisiting the website. History and
manual cleanup protect references shared across kinds and source runs. ZIP export
includes only selected baseline originals, not an unapproved sibling from the
same capture. Screenshot-only legacy archives remain supported; DOM or extended
portable settings/provenance use archive v2. Import creates a new local project
and does not execute captured source or visit its URLs.

The CLI and local engine expose the same modes, kind-specific review/approval and
bounded source/diff reads. Reports present escaped inert text with plaintext
originals. See [CLI usage](cli.md), [archive details](portable-baseline-archive.md)
and [login/capture limits](auth-and-capture.md).

Implementation and verification scope: [plan](dom-snapshot-plan.md) and
[verification record](research/dom-snapshot-verification-20261017.md). Local tests
do not imply native OS-input, accessibility or cross-platform release qualification.
