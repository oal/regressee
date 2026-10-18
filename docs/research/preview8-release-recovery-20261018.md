# Preview.8 release recovery — 2026-10-18

Preview.6 and .7 tags remain immutable; neither published a release. The first
preview.8 manual candidate (`39123328355`, commit `8560a3a`) failed its new early
Windows lifecycle check. Its remaining work was cancelled after that required
gate failed. No preview.8 tag or release was created.

## Observed Windows failure

Both old-endpoint upgrade and six concurrent autostarts returned the same
structured CLI exit 6: `Cannot start installed engine executable: Access is
denied. (os error 5)`. The production launcher requires `CREATE_BREAKAWAY_FROM_JOB`
to keep engine lifetime independent of its client. It intentionally refuses a
Windows job that denies breakaway, rather than falling back to inherited lifetime.
Do not drop that flag, weaken permissions, alter the runner's job limits or call
an unsupported host's refusal a detached-success qualification.

The lifecycle tests now independently call Windows `IsProcessInJob` and
`QueryInformationJobObject`, checking both explicit and silent breakaway flags.
Only an OS-confirmed non-breakaway job changes the expected contract: all starts
must return the precise startup/access-denied failure and actual exit 6, leave no
new store/engine owner, and retain an existing upgrade descriptor/data unchanged.
An unrelated startup failure, policy-query failure, or failure on an eligible host
still fails. Eligible hosts retain all original success/stop/upgrade assertions.
The Windows API dependency is test-only and already pinned in the lockfile.

The initial .7 Windows confirmation failure (`Interrupted` instead of `Failed`)
did not reproduce in its one diagnostic rerun; that rerun progressed to the
detachment failure above. The assertion is not relaxed, and now includes the
actual job receipt when it fails. Its durability cause is not claimed fixed.

## Other retained facts

Candidate `39123766440` at `70f9fcc` passed the OS-aware Windows lifecycle tests,
then failed two engine contracts at **result durability**, not detachment. The
receipts explicitly said `Result durability failed; the operation may have
committed`. The .7 unexpected Interrupted outcome therefore remains relevant;
accepting Interrupted as success or discarding its prior receipt is not allowed.

The next correction adds bounded retry only around Windows atomic publication
of the exact already-written and synced temporary journal file. Win32 errors
5/32/33 may retry 20 times at 10 ms, revalidating existing owner/path constraints.
No worker operation or admission is replayed, no delete-then-rename is used,
and persistent/other failures retain the old receipt and existing Interrupted
semantics. Unix publication is unchanged. Windows-only tests hold actual reader
handles without FILE_SHARE_DELETE: one releases during the retry window, the
other remains locked and must fail without changing old bytes. The early Windows
engine gate must execute these; no local Linux execution claims those branches.
Test-support-only diagnostics now retain the actual receipt-write cause.

- Linux .7 passed its entire ordinary application contract suite, then packaging
  failed because `artifacts/` did not exist. Commit `584b8e7` creates only that
  parent, preserving the packager's fresh-output requirement and privacy gates.
- The original local full invocation passed persisted 100/1,000-page qualification
  but failed 11 parallel CLI fixtures at the global eight-browser cap. All 23 CLI
  workflows pass serially in the isolated .7 source/build directory. A preceding
  rerun shared artifacts with concurrent source work and failed before tests;
  that compilation is not a product regression or a passing test.
- Manual version tags are deferred to publication: the requested new tag is
  validated and must not exist. Both platform jobs, full suites, packaging and
  dependency/ABI checks remain mandatory. Failed candidates do not consume tags;
  the existing push-tag entrypoint remains supported. No partial local upload.
- Updated source includes the independently committed protocol-9/native-motion,
  inline browser-update and native DOM highlighting work. Public notes describe
  these and retain native-input/accessibility/clean-machine qualification limits.
