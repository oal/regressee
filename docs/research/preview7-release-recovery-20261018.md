# Preview.7 release-gate recovery — 2026-10-18

The user requested all pending source committed/pushed to main and a new tagged
release. Commit `80ef55c`, tag `v0.1.0-preview.6`, and workflow run `39117310493`
are retained unchanged. The workflow failed and its publish job was skipped;
no preview.6 release assets were published.

## Observed failures and corrections

- Linux `Test private-source omission and completeness policy`: test CLI exited
  2 because `--report` is required. Supply the new private report path under
  `RUNNER_TEMP`, matching the existing Foundation workflow. Add an offline
  workflow-wiring regression; retain all source privacy/completeness checks.
- Windows `Run application contract tests`: 82 app unit tests passed, one was
  deliberately ignored, and
  `terminal::tests::legacy_reports_expose_unknown_queue_without_borrowing_current_settings`
  failed with `invalid_path: destination parent must be canonical (no symlink
  aliases)`. Canonicalize that test's temporary directory before constructing its
  export destination, including Windows verbatim paths. Do not weaken the
  production destination guard or skip the test.

Use the next tag, `v0.1.0-preview.7`; never move/rewrite the failed tag. Only these
scoped corrections and release notes belong in the correction commit. Preserve
new concurrent GUI/guide edits in the working tree without staging them. Both
platform builds and the existing publish gate must pass before claiming a release.
