# Page-list recovery and destructive-action consent — 2026-10-14

## Reported failure and repair

The user confirmed clicking **Dismiss page list**. That action immediately
cleared the unsaved discovery preview, while leaving Pages & baseline showing a
recorded inventory. Its rows were deliberately inert and its details/navigation
controls were unmounted. The recovery button additionally required an active
baseline, so an incomplete first capture had no way to reuse its recorded list.
The screenshot's Filter → Gaps instruction was also unavailable on that surface.

Recorded rows now open bounded, read-only **Page details**, with Previous/Next,
full addresses, inclusion/exclusion reasons, coverage, selection and row reveal.
Search-hidden stale selections cannot be activated. **Use recorded pages** loads
the displayed latest run's inventory into a new unsaved plan, with a new inventory
identity. It works without a baseline, preserves discovery-blocked reasons and
does not visit websites, capture pixels, mutate recorded evidence or save a
baseline. Capture-error guidance on this surface names available recovery paths.

The user requested retaining an abort action, with clear consequences and a
confirmation popup, and auditing other destructive actions. Page options now has
the danger-styled **Discard discovered pages…** action. Its warning names the URL
count, loss of unsaved inclusion choices, preserved screenshots/baseline and
recovery paths. **Keep page list** is first/default. Close/Escape preserves the
plan. Stale consent cannot discard a newer list.

## Consent audit

| Action | Confirmation boundary |
| --- | --- |
| Discard discovered pages | New warning popup; exact preview snapshot |
| Find pages again while a plan exists | New warning popup before replacement or save-and-find |
| New project / switch project | Existing discard dialog now also covers discovery choices |
| Quit / window close | Existing guard now covers discovery; list-only quit has Keep page list / Discard pages and quit |
| Same-project settings save | Keeps discovery instead of silently deleting it; stale plans cannot capture or preview |
| Forget selected environment's login | New warning popup; project/environment snapshot and saved-selection guard |
| Use a retained baseline | New warning popup; selected baseline/project revision; both versions retained |
| Delete history record | Existing impact preview plus new warning popup; preview fingerprint/revision and backend reference protection |
| Discard interrupted temporary files | New warning popup after integrity preview; complete originals and active writes protected |
| Delete complete recovery originals | Separate new irreversible-deletion popup; referenced evidence protected |
| Repair/update private browser | New warning popup; existing baselines kept, matching-browser caveat explicit |
| Save whole baseline | Existing native confirmation preserved |
| Approve selected baseline references | Existing protected approval dialog preserved |
| Discard project/review edits | Existing switch/quit dialogs preserved |

Ordinary editable-field changes, reversible local inclusion/exclusion and search
clearing do not each gain a disruptive popup. Stop/Pause retain their existing
task lifecycle: completed originals are kept, not deleted. ZIP import creates a
separate project; export retains its privacy preview and native path selection.
No deletion capability, automatic retry or baseline approval was introduced.

## Verification

- Nine focused `safety_ux_tests`: cancel, explicit commit, stale consent, real
  isolated history deletion, retained-baseline restoration, login maintenance,
  browser-repair cancellation, quit/rediscovery warnings, settings-save retention,
  and a 579-URL incomplete capture with no baseline.
- Existing discovery handoff and options regressions pass. Core shell focus
  regressions pass (11 tests); storage passes 35 tests with its two explicit
  profiling tests ignored. The normal UI binary build and Clippy pass with
  `-D warnings`.
- Six actual Linux GPUI offscreen frames inspected together: recorded-page
  recovery and included/blocked discovery details at 1280×800 and 820×600. The list,
  details, recovery controls and pinned danger action remain bounded/reachable.
  Synthetic pixels and JSON provenance remain in the private scratch directory
  `regressee-page-recovery-EF4npdQT`, not as user-site evidence in the repository.
- The broader UI-binary run was bounded at 120 seconds and did not complete;
  it is not a full-suite pass. Its image-control failure passed when isolated.
  No unrelated test or runtime policy was weakened to make this repair pass.

Browser setup was not downloaded, websites were not visited and the user's data
was not opened or altered. Prompts and input were simulated through GPUI; renders
establish development-host layout only, not OS-input, accessibility, cross-OS or
release qualification.
