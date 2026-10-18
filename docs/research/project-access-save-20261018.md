# New-project access save — 2026-10-18

## Report and cause

The user entered a project name and URL and chose **No login required**. The
access save failed with generic advice about disk space, permissions and another
window, without the actual cause.

Reproduced in the GPUI controller with a real isolated storage actor: create one
public project, then a second in the same window. `create` retained the first
project's `environment_draft`. The first save creates a new environment ID, but
the stale selection prevents initial environment hydration. The next access save
then fails the storage check that the selected environment belongs to the project.
The job handler previously discarded that error as `ConflictOrWrite`.

## Correction

- Reset the project-local environment selection, status/read signatures, login
  fields and ephemeral-login choice when starting a genuinely new project.
  Revisiting an existing unsaved new-project form still preserves it.
- Preserve write-error chains through the existing `ProductFailure` transport,
  with an engine-owned `storage_write_failed` tag. The CLI retains its existing
  error code and exit status 6. Known permission/low-space I/O categories retain
  their structured recovery path.
- Show the underlying cause in the task message, keep full Details, and retain
  the draft. Internal error tags stay out of ordinary task/banner text.
- Describe this save as website access settings, not necessarily login settings.

## Verification

The new two-project regression failed before the correction with
`Failure(ConflictOrWrite)` and passes afterward. Focused checks pass:

- 6 setup tests, including fresh/sequential public projects and private-to-public
  project creation through the production GUI facade and real engine IPC.
- 5 setup-race/validation tests, preserving newer drafts and visit boundaries.
- 12 task tests, including the actual invalid-environment error, Details, retained
  draft/revision, and footer access at 1280×800 and 820×600.
- 7 app task-reply tests, including unchanged storage on a failed environment save,
  full nested error causes, and existing typed I/O recovery.
- 6 CLI unit tests, including stable write-failure classification.
- The existing conflicting-writer save/navigation regression.
- The storage transaction's atomic/private/revision-bound save regression.
- Normal desktop and CLI builds with the `ui` feature, without inspection features.

All data is isolated test data; no user project, login or screenshot was opened
or changed, and the tests do not visit the reported website. GPUI controller and
painted-bounds checks are not native OS-input, accessibility or visual acceptance.
The broader workspace formatting check reports unrelated in-progress formatting;
only the changed standalone files were formatted, leaving those edits intact.
