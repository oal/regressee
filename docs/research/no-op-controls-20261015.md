# No-op controls audit — 2026-10-15

## Finding and correction

**P2: the App settings folder-picker preview was a dead end.** Its handler opened
a native directory dialog and copied the first selected path into a notice. There
was no filesystem inspection, saved preference, relocation operation or downstream
consumer. Earlier copy changes made this technically explicit but did not make it
a useful task.

Removed the picker, helper copy, completion notice, `Control::ChooseFolder`, focus
candidate, handle mapping, mount rule and form-reveal registration. **Storage
locations** retains actual project-data/cache paths and the existing storage-total
limitations. There is no replacement demo or disabled future relocation control.
The optional network and troubleshooting groups follow immediately afterward.

This is a narrow Operate-mode correction using Impeccable's existing-context,
clarity and craft guidance. Navy/teal identity, system typography, native action
metrics, disclosures and scrolling are unchanged. No tokens or shipping rasters
were added. The current surface brief supersedes the old preview-only instruction;
the earlier report is retained with a supersession note.

## Broader source audit

Traced rendered controls through mounted/enabled guards, activation handlers and
background operations in `main.rs`, the workflow/analysis/review/settings modules,
`safety_ux.rs` and `jobs.rs`. Searched app/runtime/worker code for dummy, stub, TODO,
unimplemented, demo, no-op and preview-only patterns, distinguishing test fixtures,
informational help, genuine limitations and noninteractive loading placeholders.

No additional active picker demo or dummy preview was identified in that scope.
The remaining previews deliver real information or an explicit decision:

| Surface | Actual operation / result |
| --- | --- |
| Discovered pages | `Work::Discover` returns real visited addresses and include/exclude choices consumed by capture. |
| Sample screenshot | `Work::PreviewImages` captures the first included page at the configured screen sizes and stores a `RepresentativePreview` run. It is intentionally ineligible as a baseline. |
| Mapped URLs | `Work::EnvironmentStatus` computes guarded source/destination pairs from saved starting addresses or baseline pages; access checking separately visits sample pages. |
| Baseline update | `Work::Approval` computes revision-bound reference changes; `Work::Accept` commits only after explicit consent. |
| Baseline ZIP export/import | Storage inspects the selected baseline/archive; explicit confirmation dispatches real archive creation/import. The file pickers are connected to these operations. |
| Support report | `Work::DiagnosticPreview` collects privacy-filtered local information; the save dialog feeds `Work::DiagnosticExport`, which writes the reviewed JSON. |
| Deletion / saved-data check | Storage calculates real impact or interrupted/corrupt-file findings; cleanup/deletion uses separate confirmed operations. |

Retained existing protections against stale consent, shared-file deletion,
unapproved baseline changes and exporting login material. Platform/build browser
unavailability is an actual capability limitation, not a fake install action.
Synthetic project/pixel fixtures belong to test-only capture modules, not normal
application results. Inactive legacy numeric action candidates are filtered out
by mounting/enabling rules; they are not exposed as features.

## Verification

- Replaced the old test that protected the no-op's wording with a two-size GPUI
  regression: disclosure changes layout, adds no action, opens no dialog, submits
  no work and preserves stored evidence.
- Updated collapsed/expanded Tab and reverse-Tab expectations and the native
  offscreen settings recipe to inspect the real paths rather than the removed
  picker.
- A pre-existing integration-test initializer lacked the recently added
  `Work::Activate.confirm_partial` field. Set it to `false` for its completed,
  full-baseline case so the broader test suite can compile; no baseline behavior
  or consent rule changed.

### Results

- Current focused App settings tests: **4 passed**, including both-size disclosure
  and forward/reverse keyboard traversal. Core: **53 passed**. Storage: **35
  passed, 2 ignored**.
- The broader run passed **118 app-library tests** and **188 native-shell tests**
  (2 and 1 ignored respectively), plus the completed analysis/sharing/privacy
  integration tests. It then exceeded the command's 120-second budget in the
  existing `phase7_distinct_persisted_100_and_1000_page_selective_approval_privacy_and_retention`
  qualification test. **No full-suite PASS is claimed.** External browser/download
  tests remained explicitly ignored; they were not substituted with fake results.
- Production `cargo build --locked -p regressee-app --features ui --bin regressee`
  passed with `LIBRARY_PATH=$PWD/.runtime/link-libs`.
- Native GPUI/Blade offscreen recipe: **18 captures passed** at **1280×800** and
  **820×600**, covering six browser states plus storage, network and support.
  Inspected the affected paths and neighboring groups in both sizes: no picker,
  clipped storage copy or stranded focus target; native focus remains visible.
- One confirmation batch passed after concurrent About-dialog edits changed a
  shared source binding. All 18 confirmation PNG hashes and current bindings for
  settings implementation/tests/capture recipe, `main.rs`, core shell and form
  recovery were verified. The fixture evidence remained unchanged. No further
  visual-polish loop was run.
- `git diff --check` passed. Repository-wide formatting checking also encountered
  pre-existing unformatted in-progress changes; only this task's standalone
  settings files were formatted, rather than rewriting unrelated work.

Private synthetic captures and per-frame source metadata:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-no-op-20261015-UVk8Fe/`
(`inspection-YGqoGC/` and `confirmation-RMAnYB/`). No user project data or screenshot
was used. Scoped UX verdict: **ready for this removal**; the real paths, useful
previews and deliberate mutation boundaries remain intact.

Native GPUI simulation/offscreen evidence does not qualify OS dialogs/input,
assistive technology, Windows/macOS or a public release. This source audit is not
a claim that every real-world app workflow is defect-free.
