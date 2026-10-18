# Project and settings removal — approved scope, 2026-10-18

## Operate-mode brief

The user wants to remove a test project and start over. They approved a complete
removal-UX pass: projects, added environments, individual address mappings, and
clearer names for existing History deletion. Preserve the native neutral/teal
workspace, controls, keyboard access and capture/baseline safeguards.

- Projects: visible **Delete project…** alongside each project's management
  actions, plus a separated **Delete project** section in Project settings.
  Inspect measured impact before a named, Cancel-first permanent confirmation.
  Remove local settings, runs, screenshot/DOM captures, baselines, review history,
  ignored areas and logins. Keep other projects, shared files, installed browser
  components and exported ZIPs. Success returns to Projects without auto-selecting
  another site. Failures retain the project and explain any partial login cleanup.
- Environments: **Remove environment…** beside the selected environment's
  configuration, never the final remaining environment. Saved removal is
  revision-bound and confirmed, removes its own saved login, and preserves frozen
  History. Unsaved additions can be removed as draft changes.
- Address mappings: a named **Remove** action on each mapping, not an unlabeled
  trash icon. This changes only the draft until **Save settings**; the final
  mapping is protected. Recorded captures/baselines are never rewritten.
- Existing lifecycle controls: clarify **Delete run… / Delete baseline…** while
  retaining their reference-aware protections. Keep login forgetting, discovered
  page discard, screen-size removal, and global recovery cleanup in their existing
  scopes. No ambiguous Reset everything, bulk deletion or archive feature.

## Lifecycle review

| Surface | Finding and decision |
| --- | --- |
| Projects / Project settings | Missing permanent removal. Added both discoverable entry points; one named measured-impact confirmation, not stacked preview and warning steps. |
| Access & environments | Adding staging had no counterpart. Added confirmed saved-environment removal and local removal of an unsaved addition. Protect the last environment and retain frozen captures/baselines. |
| Address mappings | Add/replace existed without individual removal. Added row-level Remove; protect the final mapping and require the ordinary settings save. |
| Screen sizes | Existing built-in toggles and Remove Custom screen size already change the draft. Keep them, without an unnecessary permanent-delete warning. |
| Discovery | Existing Discard discovered pages has consent and preserves recorded captures. No second reset action. |
| Saved logins | Existing Forget login has exact-scope and encrypted-ownership checks. Reuse these safeguards in project/environment removal. |
| History | Keep reference-aware run/baseline deletion, but name the triggers Delete run / Delete baseline and distinguish their final confirmation titles. Whole-project deletion does not require deleting dependency chains by hand. |
| Notes / ignored areas | Existing edit/clear/remove plus explicit Save already supplies the reversible counterpart. Keep the retained audit history rather than add an ambiguous erase-history action. |
| Global storage / browser | Keep recovery-original cleanup and browser replacement separate from project deletion. Add truthful pending-cleanup status; do not couple project removal to uninstalling shared browser components. |

Browser uninstall, bulk History retention and an archive feature remain separate
potential storage-management work, not silent additions to the approved scope.

## Implementation

- `DeleteItem::Project` and `Environment` extend the existing reference-aware
  preview/confirmation path. Local engine protocol is **8**; database schema and
  recorded evidence formats are unchanged.
- Impact authority includes project revision/configuration, the reference graph,
  file facts, reviews, masks, inventories, display pointers and local login
  references. Revalidate under the SQLite writer lock; refuse active captures,
  stale/tampered previews and replayed engine confirmation tickets.
- Remove relational children with foreign keys enabled. Queue only exclusively
  owned artifacts for confirmed GC, which rechecks sharing before unlinking.
  Exported ZIPs and other projects are outside the operation.
- Authenticate all private login scopes before removing the first credential.
  Forget persistent and ephemeral material and close only the affected login
  browser. Locked/corrupt/foreign state blocks metadata deletion. External secret
  cleanup cannot be rolled back; failure copy explicitly preserves the project
  while warning that some logins may already have been forgotten.
- A committed project deletion with failed physical unlink has a distinct
  `ScopeDeleted { cleanup_pending: true }` receipt. Its journal survives. Startup
  retries without making unrelated projects unavailable; App settings discloses
  the remaining file count. Permanent removal is not forensic disk wiping.
- Cancel/Escape preserves drafts and restores the initiating control. New edits
  or control takeover while confirmation is open cannot authorize an old intent.
  Deleting an unselected project preserves the selected project's draft.

## Verification and verdict

**PASS within the scoped source/controller/IPC/offscreen inspection below. Not a
whole-app, native-input, accessibility, cross-platform or release pass.**

- **57 core tests PASS**; **64 storage tests PASS**, two existing profile ignores.
  Includes active-run/last-item/stale-impact guards, rollback on login-cleanup
  failure, shared screenshot/DOM imports and ZIP retention, and journaled physical
  cleanup after a changed/unsafe file entry.
- **18 auth/task-reply tests PASS**, including locked and forged persistent
  login references, ephemeral environment isolation, and existing failure paths.
- **3 scoped engine integration tests PASS**: exact consent/replay protection,
  frozen baseline/run retention on environment removal, and restart/browsing with
  pending project cleanup.
- **6 CLI unit tests PASS**, and scoped all-target Clippy (core, storage, app,
  GUI and CLI, including UI tests) passes with warnings denied.
- Normal **`regressee-desktop` and `regressee` builds PASS** with `ui` only,
  without inspection or renderer-capture features; not launched on user data.
- **34 selected GUI tests PASS / 1 separate long-name header failure**. The three
  new removal tests pass, along with setup, safety, wording, App settings and
  project-settings Tab-order tests. Selected/unselected/last project deletion,
  cancellation, newer-draft rejection and same-name/URL recreation run through
  the production GUI facade and real engine IPC. Mapping and environment
  last-item/save boundaries are separately exercised.
- Bounded offscreen inspection: **20 initial + 28 final PNGs**, both Light/Dark
  and 1280×800 / 820×600. Final scenes cover Projects actions, project confirmation,
  Project settings deletion, environments, environment confirmation, mapping
  removal and pending-cleanup feedback. Real GPUI/Blade/Cosmic output, not a web
  mockup. The pending-cleanup visual count is a labelled synthetic presentation;
  cleanup and engine restart are independently exercised above.

Private evidence root under the approved scratch directory:
`regressee-project-removal-20261018-50k909tv/`. Initial and confirmation receipts
retain their exact test-binary hashes; the final receipt also binds scoped source
files. No shipping rasters or dependencies were added. No user data or website
was opened or removed. Visual budget is complete; no further polishing round.

### Unresolved broader GUI gates

The full 265-test GUI invocation timed out after 120 seconds. Its early failures
were individually diagnosed, not hidden or relabelled as passes:

- `changed_results_first_analysis_and_recorded_checks_default_without_resetting_refresh`:
  `changed_results_tests.rs:218`, recorded-History filter is 0 rather than 1.
- `all_inline_disclosures_attach_content_restore_focus_and_do_not_mutate`:
  `disclosure_tests.rs:132`, Discovery-details/search disclosure is not enabled.
- A selected Project settings test expects the old `Guided(2)` jump destination;
  the existing Capture-plan jump uses `CaptureMode(ScreenshotsOnly)`.
- `project_selection_actions_and_long_names_fit_supported_window_sizes`: project
  rows (including the added action) fit, but the later selected-project header's
  Change project inset assertion fails at `main.rs:9916` before final formatting.

These paths are outside removal behavior and remain unresolved. The final scoped
visual inspection does not claim long-name header qualification. Initial local
build setup also needed the existing Linux linker-name helper adapted in private
scratch, and offscreen capture needed `ui-inspection` alongside its renderer
feature for the existing optional `libc` dependency. Neither changes host packages
or shipping features.
