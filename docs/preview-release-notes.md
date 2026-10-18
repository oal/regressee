# DOM comparisons and project management — preview.7

This preview adds independent DOM snapshot comparisons, complete project-removal
controls, and fixes for new-project access saves and engine upgrade recovery,
alongside the existing desktop and CLI workflows.
It remains an **unsigned experimental prerelease**, not a supported
release. The release's source commit and each archive's `build-info.json` identify
the exact build; the application crate version remains `0.1.0`.

## What changed

- This retries the unpublished `preview.6` build with the required Linux source
  policy report argument and a canonical Windows test-fixture destination.
  Source/privacy checks and application tests remain mandatory on both platforms.
- Capture **Screenshots only**, **Screenshots + DOM snapshots**, or **DOM snapshots
  only**. Baselines freeze that mode. DOM source is inert, read-only text with
  kind-specific comparison, review and approval; missing or failed checks are not
  silently called unchanged. CLI reports and baseline ZIPs support retained DOM
  originals. See [DOM snapshots](dom-snapshots.md) for coverage and privacy limits.
- **Delete project…** is available in Projects and Project settings. One named,
  measured-impact confirmation removes local settings, captures, baselines,
  reviews and logins, while retaining other projects, shared files, the installed
  browser and exported ZIPs. Locked or invalid login ownership blocks deletion.
  Incomplete physical file cleanup is reported separately and retried on startup.
- Added environments and individual address mappings now have removal controls.
  The last environment/mapping is protected. Saved environment removal preserves
  recorded History; mapping removal remains a draft until Save settings.
- Creating another project clears the previous project's environment/login state,
  fixing a **No login required** save failure. Save failures retain their causes.
- Upgrading from engine protocol 6/7 has an explicit graceful-stop path in the new
  executable: close old desktop windows, finish active work, run `regressee engine
  stop`, and reopen. No active engine is automatically killed, and no project data
  needs deleting. Keep both packaged executables together and upgrade both.

- Source builds now use `regressee-desktop` for the existing native application.
  The separate `regressee` CLI includes public project/discovery/capture/check,
  evidence/crops/reviews/baselines/history/local-report/sharing/runtime commands,
  durable jobs and named renewable agent sessions. GUI following is a human opt-in
  with exclusive control, immediate takeover and temporary handoff. Both clients
  use one shared local engine, not independent store owners (protocol 8/schema 1).
  `regressee-cli-<platform>` archives contain only the CLI;
  `regressee-desktop-<platform>` archives contain **both** separate executables.
  Portable skill, public help, product-specific license inventories, source/build
  metadata and checksums are included. There is no standalone Windows GUI EXE
  download in this package layout. See [build/install instructions](release-building.md).

- Optional sections now use leading disclosure arrows and explicit **Show / Hide**
  labels throughout settings, setup, analysis, History and confirmation previews.
  Revealed content is indented directly under its heading, without a divider that
  makes it look like a separate section. True selection menus remain unchanged.
- Expanding a section brings its heading and the start of its content into view.
  Collapsing restores focus to the heading and preserves drafts. Task details now
  opens beside its content rather than from the unrelated action footer.
- Troubleshooting keeps support-report tools together. Requested saved-data check
  results stay available even if the optional tools are subsequently collapsed.
- Enter and Space activate controls once per full key press, without immediately
  reversing an expansion or closing a newly opened dialog. Keyboard focus has a
  stronger contrasting outline, including a Linux renderer correction.
- Confirmations, About, keyboard help and task dialogs share consistent styling
  while keeping safe initial focus and explicit consent. About includes the
  repository link; external links have clearer hover feedback and compact hints.
- Browser setup failures retain useful local diagnostics and name the failed step.
  Missing screenshots have a clear, bounded explanation and recovery guidance.
- A partially successful baseline capture can be saved with explicit confirmation
  of skipped screenshots. Future checks retain its reduced, truthful coverage.
- Storage locations show the actual data and cache paths; the unused folder-picker
  preview has been removed. About and keyboard help remain explicit actions.
- Project settings now keeps a labelled **Jump to** row visible for Website,
  Capture plan, and Access & environments. It is one continuous form with one
  explicit Save action, not separate tabs.
- A visible interactive scrollbar shows the remaining settings. Drag its thumb,
  click its track, or scroll normally; the Save footer stays available.
- Settings Tab order follows the displayed groups, and section jumps focus the
  first enabled destination without saving, opening a browser or changing the
  baseline. Local tests and two-size native offscreen inspection cover the change,
  not native OS input or accessibility. Current qualification limits are in the
  included [support matrix](support-matrix.md).
- Review gives evidence more space, keeps page/profile/environment context visible,
  and places the queue's collapse/restore controls beside the queue.
- Review decisions, notes and baseline selection have clearer task ownership;
  baseline consent preserves selected versus inherited references explicitly.
- Guided setup distinguishes saved settings from drafts, routes corrections to
  the relevant stage, and preserves login drafts across asynchronous saves.
- History, sharing, cleanup and failure recovery use clearer task language while
  retaining explicit privacy and irreversible-action warnings.
- Persisted synthetic 100/1,000-page regressions exercise distinct originals,
  selective approval, stale/corrupt evidence refusal, retained references and
  byte-identical baseline export/import. These are **not browser throughput or
  native-input benchmarks**.

## Limits and privacy

Focused source/controller/storage/IPC and Linux GPUI offscreen checks passed.
The wider UI-inclusive suite is not green: recorded-History filter/default-source,
Discovery-details availability and a long-name project-header layout assertion
remain known issues. A Project-settings Tab test also retains an outdated
Capture-plan jump expectation. These are not hidden by the focused removal checks.
Native desktop input, accessibility, clean installation, default graphics,
cross-platform usability and full release/license qualification remain pending
or blocked. Windows/NVDA and macOS/VoiceOver have not been qualified; macOS is
not a downloadable target and still has a process-supervision code gap. CI
compilation or simulated controls do not establish those capabilities.

Data remains local unless explicitly shared. Baseline archives contain website
pixels, DOM text/attributes when enabled, and addresses, but do not export login
sessions/cookies. DOM filtering is **not complete redaction**; hidden text, URLs
and attributes may still contain private information. Ignored regions are
comparison policy, **not redaction**. Browser installation on first use
needs internet access; downloads do not include the development `.runtime` tree.
Do not disable operating-system or browser security protections to run a preview.

See the included [build/install requirements](release-building.md) and
[support matrix](support-matrix.md) for evidence scope and outstanding gates.
The source archive includes these public guides under `docs/`; internal research,
independent review reports and private evidence are deliberately not distributed.
