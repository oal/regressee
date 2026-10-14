# Failure recovery hardening — 2026-10-14

## Why previous reviews missed this

The prior bounded reviews checked painted states, focus, draft guards and
individual error disclosures, but not the full **upgrade → task failure →
understand cause → reach correction → safely retry** journey. Runtime verification
collapsed a different installed release into repair, the Jobs boundary discarded
the typed cause, and the shared task outcome hid the explanation in Details.
Passing those bounded reviews was not evidence that upgrade/recovery journeys
worked. The reported title-save/discovery sequence exposes that coverage gap.

## Repair and scope

- Official runtime resolution distinguishes a different, ordinary bounded
  release marker from damage/unsafe state. This is classification only: it grants
  no trust, executes nothing, and does not replace files. Matching installations
  still require complete inventory/hash verification; linked directories remain
  refused.
- Runtime prerequisites, reported worker categories and permission/disk-full
  I/O causes survive extra error context as safe structured replies. Invalid
  build configuration also takes this route. Project saves retain known I/O
  causes before falling back to the existing conflict/write category.
- Primary messages name known causes and correction steps. The footer navigates
  to the real relevant settings or History Storage; it does not run repair,
  login, retry, capture or cleanup. Closing the task clears its blocking banner
  on those routes so settings reads can complete. Draft guards remain intact.
- The exact save-and-find continuation confirms **Your project settings were
  saved** if the subsequent task fails. A browser failure cannot masquerade as a
  failed title save or discard a newer draft.
- Unknown operation failures show their bounded existing outer explanation,
  followed by task-specific recovery; nested private website/session stacks are
  not expanded. Setup failures retain the earlier bounded local diagnostic
  disclosure and now add stage-specific correction guidance.
- Export permission recovery returns to the export origin to choose a writable
  destination. A projectless low-space setup opens App settings, not empty
  project-dependent History. Checks warn that repairing/updating components does
  not automatically make an older baseline comparable.

| Known cause | Correction route |
| --- | --- |
| Different installed release / missing or damaged browser | App settings; explicit Update / Install / Repair |
| Invalid configuration / unsupported platform | Supported build/platform; local work still available |
| Expired login, access denial, missing page, rate limit, outside-scope redirect, image-size limit | Project settings; appropriate login/address/speed/extent guidance |
| Timeout, website server error, stopped worker, payload limit | Return to workspace; check availability/limits before retrying |
| Local permission failure | App settings folder information; export uses a writable destination instead |
| Disk full | History → Storage, or App settings without a selected project; no automatic deletion |
| Unknown operation failure | Existing cause plus operation-specific safe recovery, with diagnostics separate |

Discovery can return a limited inventory rather than fail the entire operation,
and captures can record per-target gaps rather than return a fatal worker error.
Those continue through the existing limited-list and capture-gap UX; this change
does not relabel partial work as successful or alter the scheduler's retry policy.
No user's cache, baseline or stored project was repaired as part of this work.

## New regression gate

Every new failure presentation must prove the journey, not just its screenshot:

1. Trigger through the actual public Jobs/controller boundary where feasible;
   include old installations, wrapped causes and save-then-task continuations.
2. Verify what was saved, what remains a draft, and what did not happen.
3. Show cause and recovery without Details at both desktop sizes.
4. Activate the labelled action and verify the real mounted correction control
   or destination. Check that an old banner does not block status reads.
5. Confirm that navigation alone never downloads, visits, retries, activates a
   baseline or deletes anything, and that drafts/retained evidence survive.
6. Treat unsupported configuration, permission, disk-full and cancelled work as
   distinct conditions. Never promise that a blind retry fixes all of them.

Tests live in `crates/app/src/recovery_ux_tests.rs`, shared cause tests in
`jobs.rs`, and release/refusal tests in `product.rs` and runtime `official.rs`.

## Verification boundary

Final Linux development verification:

- App library: **106 passed**, two opt-in checks ignored.
- Native-shell controller/layout suite with renderer support: **153 passed**,
  28 opt-in captures/native checks ignored, one bulk-consent qualification test
  deliberately filtered out. The initial smaller-feature suite also passed all
  152 enabled controller tests.
- Runtime library: **32 passed**, one OS-secret-store check ignored.
- Runtime distribution integration: **7 passed**.
- Opt-in inert public-API runtime verifier: **5 passed**, including pinned-work
  refusal, cancellation, pointer/evidence preservation and zero unwanted network
  connections. Its first attempt used a non-private output root and was refused;
  the retry used a fresh owned 0700 directory without changing other permissions.
- Strict scoped **all-targets Clippy passed**; the existing dependency
  future-compatibility warning remains. `git diff --check` passed. New recovery
  modules were formatted without rewriting unrelated shared-file formatting.
- Production-feature Linux debug executable rebuilt successfully at
  `target/debug/regressee`; an already running process must restart to use it.
- Calibration was personally inspected. One **20-image** initial batch at
  **1280×800** and **820×600** was personally inspected, followed by one finite
  confirmation batch with eight changed-element samples personally rechecked.
  All 20 final PNG hashes and current UI source bindings were verified. Messages
  fit without opening Details, footer actions remain visible, browser correction
  controls are above the fold, and App settings respects its 680px measure.

Final captures, per-frame source provenance and `review-receipt.json` are retained
at:

```text
/var/home/olav/Data/opencode/tmp/opencode/regressee-recovery-final-yyak1i8w
```

The first batch remains at `regressee-recovery-sgekr9j2` beside that directory.
Earlier broad commands hit their 120-second budget; the relevant app suite was
rerun with a sufficient bound and passed. The separate persisted 100/1000-page
storage qualification that timed out was not rerun or claimed as qualified.
Real browser/download, public-site, OS-input and platform checks stay opt-in.

**Bounded verdict: PASS for the shared fatal-error recovery journeys and sampled
Linux offscreen presentation.** This is not Windows/macOS native input,
accessibility, real website failure reproduction, whole-app usability or release
qualification. No automatic retry or browser repair was performed on the user's
installation. Existing evidence, baselines and prior qualification limits remain
unchanged.
