# Inline browser update — 2026-10-18

## Outcome

The user's new-project discovery task reported **Browser update required** but
only offered navigation to App settings. This narrow Operate-mode refinement
keeps the current neutral/teal native workspace and performs the update in context.

- A non-installer task blocked by `UpdateRequired` mounts the existing download
  control as **Update browser…**, beside **Back to setup** / **Return to workspace**.
  The message names the download/verification and keeps saved-settings reassurance.
- The existing Cancel-first replacement prompt still precedes installer submission.
  Cancel keeps the blocked task, returns focus to Update and leaves installation
  markers unchanged. Duplicate activation and unavailable setup are blocked.
- After consent, retire only the blocked presentation, start the normal verified
  background installer, and carry forward the original setup/focus destination.
  Its real progress and cancellation stay intact. Failed submission retains the
  blocked task rather than losing the setup context.
- Successful installation offers **Continue setup**. No website task is retried,
  draft saved, evidence deleted or baseline changed automatically. Installer
  failures keep diagnostics and App settings recovery rather than an update loop.
  Repair and unsupported-build recovery are unchanged; older baselines still
  require matching browser components.

No new token, dependency, control type, shipping raster or animation was added.

## Scoped verification

Before concurrent unrelated source changes, checks passed:

- 5 recovery tests, including the user's save-then-discovery path with a real
  isolated previous-release marker, cancelled replacement and retained saved title.
- Explicit-confirmation dispatch to the existing Setup operation, retained newer
  draft/origin/focus, no duplicate submission, and safe unavailable-installer failure.
- Synthetic ready/cancelled replies at 1280×800 and 820×600 in both themes, with
  no automatic retry/save and byte-equivalent stored project evidence afterward.
- 12 task-presentation, 9 safety/consent, 7 App settings and 2 keyboard tests.
- Package all-target Clippy with warnings denied.

One native offscreen inspection produced 16 source-bound PNGs: update-required,
running, ready and cancelled at both sizes/themes. The two footer actions fit
the short window; the primary action retains the existing download icon, focus
and contrast, while the secondary back action remains reachable. No visual fix
or additional polishing round was needed. All browser/progress/outcome states
are synthetic; tests do not download browser components or visit websites.

The ordinary `ui` desktop build then encountered concurrent task-motion edits:
`task_ux.rs` imported `regressee_gui::ui::motion` before that module was available.
Those unrelated edits, plus concurrent release/protocol/storage work, were left
untouched. The passing test executable and captures precede those changes; no
current whole-workspace build or whole-suite pass is claimed.

Commands used the pinned Rust 1.98.1 through mise, the existing private Fedora
linker aliases in `regressee-access-save-link-libs`, and the approved temporary
root. No system dependency, repository linker setting or user project was changed.

**Scoped in-thread verdict: pass for inline update behavior and inspected layout.**
This is Linux GPUI controller/simulated-input and actual offscreen rendering,
not a live download test, native OS-input/accessibility pass, cross-platform or
release qualification.

## Evidence

Private diagnostic root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-inline-browser-update-20261018-3VBEE0Ag/`.
The original PNGs and renderer source/hash sidecars are diagnostic artifacts,
not shipping assets or actual website evidence.
