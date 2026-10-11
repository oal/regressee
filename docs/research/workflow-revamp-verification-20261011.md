# Workflow revamp verification — 2026-10-11

## Scope

User-approved **Clear wayfinding** replacement of the native Rust/GPUI shell,
guided initial baseline setup, project workspace, review and history. Product
scope, local storage, existing services and deliberate baseline/privacy/deletion
guards remain. See `docs/workflow-revamp.md` for the approved brief.

GPT-6.1 Sol agents implemented independently owned workflow, shared-control and
history modules. A separate GPT-6.1 Sol reviewer inspected the actual rendered
candidate. No commits, existing-project mutations or release publication were
requested or performed.

## Initial evidence and correction

The initial locked/offline workspace all-features suite passed **227 tests**;
34 opt-in tests were deliberately ignored. Actual GPUI/Blade offscreen rendering
was calibrated with opaque, readable Linux-font pixels and four exact 1,024-pixel
colour patches. Sixteen application captures covered eight surfaces at 1280×800
and the 820×600 minimum desktop size.

The independent review disposition was **fix**, not acceptance. It found:

1. Review controls displaced the queue and viewer below the first viewport.
2. Comparison captions needed the actual reference/check/environment identity.
3. Exact approval consequences were detached from pinned confirmation.
4. Coverage reconstruction used quadratic render-time classification lookups.

The source correction is implemented. Locked/offline workspace all-features
tests pass **232 tests, zero failures, 35 deliberately ignored**. Strict workspace
Clippy passes for application targets; existing GPUI floating-literal and
dependency future-compatibility warnings remain. The normal UI workspace binaries
build, and formatting is checked.

**Final independent disposition: fix — three findings resolved, one partial.**
The queue/viewer geometry, attached protected approval preview and quadratic
coverage lookup findings are resolved. Evidence identification is partial:
baseline/check IDs, timestamps, origin and current environment are visible, but
the **before environment remains “environment not loaded” on default review**.
Its actual source-run environment must be loaded alongside the displayed
reference, including inherited/composite references, without requiring a History
visit or inferring it from today's project settings. The bounded review is
complete; no whole-interface acceptance or further polish round is claimed.
Development-only detailed verdict:
`.impeccable/review/finish-review-verdict.md`.

The complete confirmation batch passed: **34 images**, 17 states at each size,
in **22.80 seconds**, with source inputs unchanged across execution. It covers
the initial surfaces plus mixed results, selected approvals, a genuine actor
impact preview, fixture candidate inspection, discovery, finite progress,
missing evidence and dirty login access. Every PNG was personally opened.
Synthetic state modifications are explicitly identified by the fixture/manifest;
approval confirmation, browser opening and baseline saving were not dispatched.

Two photographed limitations also remain explicit: the candidate-inspection
fixture reuses computed pixels in Difference mode, so it does not qualify a real
first-baseline original-image viewer or save execution; at 820×600 the discovery
preview's first complete inventory row is below the initial visible region.
The repaired check-review geometry is not whole-guided-workflow viewport
acceptance. This development state is committed at the user's request, not
promoted to a native/public-release qualification.

An incomplete diagnostic attempt emitted 14 wide images before a GPUI atlas
panic: simulated window draws had queued uploads without submitting a GPU frame
before the missing-evidence fixture retired images. The harness now explicitly
submits the loaded state before changing presentation evidence. No application
layout or vendor renderer change was made for this validity repair. The retained
incomplete batch is not substituted for the complete matrix.

Final private renderer evidence:
`/path/to/private-tests/opencode/regressee-workflow-final-gi5cl4yh/`.
Build/test/Clippy logs:
`/path/to/private-tests/opencode/regressee-workflow-confirmation-s3qi4wsd/`.

The 3,000-target regression confirms indexed coverage and eligibility rather than
quadratic per-target lookups. A debug-build CPU construction benchmark with
1,200 selected eligible targets measured 12 samples: **48.490ms minimum,
51.830ms median, 53.747ms maximum**. This excludes layout, paint and GPU readback;
it is not a 16ms-frame or native responsiveness qualification.

Initial private evidence:
`/path/to/private-tests/opencode/regressee-workflow-revamp-box494ik/`.
Development-only review:
`.impeccable/review/finish-review-initial.md`.

## Authorized real-site verification

The user expressly permitted crawling **https://example.test**. The prepared test uses
an isolated project, a bounded same-origin page inventory, gentle single-worker
capture and the app-pinned sandboxed browser. It must exercise discovery,
reviewed-inventory baseline capture, explicit baseline activation, a fixed-
inventory check and analysis, without form submissions, login, action links,
export, deletion or existing-project changes.

The clean confirmation execution **passed** in **207.14 seconds**. It captured
**12 baseline and 12 check screenshots**, then analyzed and recorded review
triage for the same fixed targets: **11 NoSignificantChange, 1 NotComparable,
zero failed captures**. The non-comparable target remains distinct from unchanged
evidence. No live-site changes were approved. Earlier baseline/original bytes,
saved project settings after explicit activation, the retained runtime cache and
service-source inputs remained unchanged. This is a headless product-service
pipeline test, not native interaction acceptance.

Confirmation private evidence:
`/path/to/private-tests/opencode/regressee-authorized-blog-confirmation-dewj3_0a/`.

The first execution discovered 12 included pages (39 excluded candidates),
captured all 12 successfully and explicitly saved the baseline. It then stopped
on a verifier assertion that incorrectly expected activation not to adopt the
captured profiles. Source inspection confirmed the intended behavior: activation
pins the captured runtime/user agent and the 32,768px renderer height ceiling.
The corrected verifier permits precisely those captured-profile changes while
still asserting unchanged unrelated profile/project settings. This failed first
attempt remains retained; it is not reported as a completed end-to-end pass.
Its service inputs and retained official runtime cache were unchanged.

First-attempt private evidence:
`/path/to/private-tests/opencode/regressee-authorized-blog-820gj9f8/`.

Evidence stays under the approved long private temporary root. The Linux worker
requires a physical socket scratch path of at most 40 bytes; `/tmp/opencode` is a
symlink to that long root on this host. The verifier therefore uses a separately
created fresh, owned mode-0700 `/tmp/regressee-blog-*` directory solely for worker
sockets. It does not recover or reuse any existing scratch namespace.

A successful bounded pipeline test is not exhaustive whole-blog coverage and
must not require a live site to remain visually unchanged. Capture failures and
non-comparable results must remain distinct from unchanged classifications.

## Qualification boundary

GPUI offscreen pixels establish rendered layout on this development host, not OS
window presentation, native pointer/keyboard/IME behavior, screen-reader support,
high contrast, clean-machine packaging or Windows/macOS qualification. Synthetic
layout fixtures and real-site browser evidence are separate and explicitly
identified. Historical black native-window captures are not superseded by an
offscreen pass. No universal UI, accessibility or public-release claim follows.
