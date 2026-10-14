# Browser-setup failure details — 2026-10-14

## Outcome

The user reported a Windows first-run failure after several minutes of apparently
steady download progress, followed by a generic red error. This change makes a
retry diagnosable; it does **not** establish the cause of that reported failure.

- `SetupAttempt` records the real installer phase, last received-byte count and
  total, and elapsed time. `SetupFailure` keeps the underlying error chain.
- The Jobs boundary expands only typed setup failures. Other operation errors,
  privacy-filtered exports and cancellation classification retain their previous
  behavior. Details include the app version and OS/architecture.
- The task names the failed setup stage and opens Error details automatically for
  both explicit installation and first-use discovery/login.
- Official download errors identify Node/Chromium and the approved source host;
  refused responses include HTTP status. File creation/write/verification errors
  identify the catalog-relative file and preserve underlying OS errors.
- Browser testing adds context for starting Node, launching Chromium/taking the
  test screenshot, and stopping the test. Raw worker/browser/site stacks remain
  excluded, so a worker failure can still be a typed category rather than the
  original Chromium error.
- Final directory/pointer publication has its own phase. Cancellation requested
  by that new progress callback is checked before publication; the old generation
  and pointer remain usable.

Request URLs are omitted from reqwest errors without dropping HTTP status or
transport/TLS/OS causes. Detail length is bounded. Local filesystem paths can
still appear, so users should review screenshots before sharing. No logging,
upload, authentication export, TLS/sandbox bypass, retry-policy change, runtime
pin change or release publication was added by this work.

## Verification and boundaries

Unrelated uncommitted discovery work briefly prevented the shared app from
compiling. Validation therefore used a detached checkout of `d6069e1` plus only
the browser-setup changes, retained at:

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-setup-diagnostics-l94lqyeb
```

On that Linux development checkout:

- App library: **101 passed**, two unrelated opt-in tests ignored.
- Runtime library: **31 passed**, one OS-secret-store test ignored.
- Runtime distribution integration tests: **7 passed**, including publication
  ordering and cancellation preserving the existing verified pointer.
- Native-shell controller/layout tests: **147 passed**, 24 opt-in tests ignored.
  This includes explicit/discovery/login error disclosure and existing
  cancellation, task focus and draft-preservation regressions.
- Strict scoped Clippy: **passed**. The existing dependency future-compatibility
  warning remains. Formatting checks passed for the isolated changed sources;
  unrelated concurrent shared-file formatting was not rewritten.
- Offscreen calibration: **passed**, personally inspected actual text/controls
  and all RGB patches. One four-image setup-error batch at **1280×800** and
  **820×600** passed its layout/action checks and was personally inspected.
  These are synthetic download/publication failures, not Windows failure
  reproduction. Images and provenance are retained under `evidence/` in the
  detached checkout. The later changes only add backend cancellation checks and
  fix a test-only lint; the reviewed UI code is unchanged.

The tests cover a real loopback HTTP 503 with credential/path/query omission,
nested OS-code preservation, real missing-CA-file preparation failures through
explicit and automatic setup, cause preservation across extra context, and
unmodified generic error handling for non-setup operations.

**Bounded verdict: diagnostic behavior and sampled Linux presentation pass.**
Windows desktop setup, merged concurrent work, real browser/download failure
reproduction and a new published Windows executable are **not verified by this
checkout**. Existing downloaded executables will not gain these details. See
`docs/first-run-guide.md` for the Windows retry instructions and information to
retain: release tag, Windows version and the complete error details.
