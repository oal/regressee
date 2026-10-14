# App settings clarity — 2026-10-14

## Confirmed scope

The user missed the installed-browser integrity check among unstructured settings
copy and interpreted the disabled setup button as unexplained unavailability.
They confirmed keeping the navy/teal identity, prioritizing status and its disabled
reason, and collapsing optional technical/support details. A follow-up asks what
the folder-picker button actually does.

## Implementation

- `crates/app/src/app_settings_ux.rs` owns the settings layout. Browser checking,
  ready, missing, update, repair, unsupported-build/platform and failed-read
  presentations retain the runtime's authoritative setup capability.
- Healthy-browser repair is a secondary maintenance action; installation,
  required updates and required repairs retain primary treatment and consent.
- Checking has a prominent state heading, neutral disabled **Checking browser…**
  action and attached **Setup is paused until this check finishes** explanation.
  The duplicate global browser-check activity is suppressed only on App settings.
- Storage totals remain visible. Browser details, storage locations/folder
  preview, optional network settings and troubleshooting disclose independently.
  Hidden children are not mounted or eligible for activation/focus; disclosure
  never submits work. Network drafts remain intact, and correction navigation
  opens the network group before revealing its field.
- **Preview a folder path…** replaces **Choose a folder to inspect…**. Its visible
  description, native-picker prompt and result notice explain that it only shows
  the selected path: no files are inspected or moved, and changing the data
  location is not available. The backend behavior is unchanged.
- Saved-data checking explains its read-only inspection and explicit cleanup
  boundary. Diagnostic preview/export privacy and all replacement/deletion consent
  guards remain in place.
- New disclosure focus handles have a disjoint range; painted controls reveal
  inside the existing outer scroll owner at both desktop sizes. Settings Tab order
  follows the rendered groups, interleaves disclosed children with their trigger,
  and ends with About/keyboard help. Reverse traversal is independently exercised.

This is a local extension of Clear wayfinding, not a new design system. It uses
existing semantic palette, system typography, control/focus/disabled metrics,
4px corners and native-drawn stroke icons. No shipping raster was added or changed
for this task. Existing unrelated work, including rail branding, was preserved.

## Bounded verification

- New settings/controller/copy/traversal tests: **4 passed**.
- Core library tests: **52 passed**, including 11 shell/navigation tests.
- Shared app library/UI tests at the final recorded run: **113 passed, 2 ignored**.
- Shell suite after adapting five older tests to explicitly open the new optional
  groups: **175 passed, 1 failed, 1 ignored**. The remaining failure is the unrelated
  `workflow::tests::discovery_owns_its_scope_and_review_preserves_provenance`
  assertion about `Guided(16)`; it was not repaired in this task. A later suite run
  with additional in-progress shared-loader work reported **176 passed, 2 failed,
  1 ignored**: the discovery assertion plus
  `loading_feedback_tests::loading_feedback_has_one_owner_and_keeps_its_icon_and_copy_visible`.
  The four scoped settings tests still passed. No whole-suite PASS is claimed.
- Production native binary build passed:
  `LIBRARY_PATH="$PWD/.runtime/link-libs" cargo build --locked -p regressee-app --features ui --bin regressee`.
  An intermediate post-review build encountered a syntax error in unrelated,
  concurrently edited `crates/app/src/workspace_history.rs:595`; that edit was left
  untouched. Once the separate edit was complete, the final production build passed.
- Opt-in `renderer_capture_app_settings` passed: 18 real GPUI/Blade offscreen
  captures cover six browser states plus storage/network/support at **1280×800**
  and **820×600**. Assertions check visible browser state/reason and reveal expanded
  controls within the actual outer scroll bounds. Stored evidence stays unchanged.
  State scenes start at the top; expanded-tool scenes are intentionally scrolled
  to their revealed action/field. Paths shown in fixtures are synthetic.

Inspection source-bound PNGs/JSON are under
`/var/home/olav/Data/opencode/tmp/opencode/regressee-settings-20261014-Ck4DI1/inspection/`,
also copied to `.impeccable/review/app-settings-20261014/inspection/`.
The same 18-scene confirmation passed after one correction batch; PNGs/JSON live
under the corresponding `confirmation/` directories. Independent initial review
returned **fix** for healthy repair's primary emphasis and settings keyboard order.
The scoped verdict scored **both resolved**, disposition **ship**; see
`.impeccable/review/app-settings-20261014/finish-verdict.md`. This scores the two
findings, not the whole app or native qualification. The whole `main.rs` hash later
differed because of other in-progress work; current focus integration was checked
directly by the reviewer. Pre-existing design-sidecar drift was reported, not
refreshed as a side effect of this settings task.

Independent documentation verification confirmed the inspected settings as an
ordinary extension using the incumbent system, with **no token changes needed**;
see `.impeccable/review/app-settings-20261014/documentation-verification.md`.
It recorded later concurrent copy/shared-source edits affecting five whole-file
bindings. Current grouping, maintenance hierarchy, folder boundary and focus logic
were retained in its bounded reread, but these captures do not establish the latest
wording or whole-checkout freshness. No further UI edits or unasked drift repair
were made in this task. Build/test results above describe their recorded runs.

No web detector ran: this is native Rust/GPUI. Evidence is offline/synthetic and
development-host offscreen only, not real-site, OS-input, screen-reader,
Windows/macOS or release qualification. No whole-suite PASS is claimed.
