# DOM snapshot testing — implementation and verification plan

Date: 2026-10-17. Owner: coordinating coding agent. Status: implementation integrated;
final native/public-consumer verification in progress. This is the standalone specification and handoff contract;
implementation and verification must not depend on access to the chat transcript.

**Current checkpoint:** core/storage/worker findings independently resolved; real
managed-browser crash/resume and report security/status checks pass at recorded
scopes; post-fix app **99 tests**, normal-umask CLI **32 tests** and strict backend/
CLI Clippy pass. Final native interaction/rendering, independent public CLI/auth
checks and final regression sweep remain. GUI owns the build slot. The chronological
handoffs below preserve earlier failures; later scored repairs supersede them.
Current acceptance matrix: `research/dom-snapshot-verification-20261017.md`.

## 1. Goal and confirmed decisions

Extend Regressee, the local-first Rust/GPUI desktop website-regression application,
to check normalized live DOM snapshots as well as screenshots. Preserve its
no-code workflow, retained evidence, explicit baseline consent and local privacy.

The user confirmed all of the following on 2026-10-17:

1. Support **Screenshots only**, **Screenshots + DOM snapshots**, and **DOM
   snapshots only**. Existing and new projects default to screenshots only.
2. Snapshot the normalized **live DOM after JavaScript runs**, not the original
   server-response HTML and not an accessibility-only/content-only abstraction.
3. Screenshot and snapshot review decisions and baseline approvals are
   **independent**. Users may update either reference or both, explicitly.
4. Use safer capture defaults: exclude script bodies, inline event-handler code
   and form values from stored snapshots. This is data minimization, **not a
   guarantee that snapshots contain no secrets or personal information**.
5. Cover the main document and **open shadow roots**. Iframe documents and closed
   shadow roots are out of scope, with clear coverage limitations.
6. Implement the **full workflow**: native setup/review/diff viewer, engine and CLI,
   reports, approvals, history, ZIP sharing, retention, migration and tests.

### Boundaries

- Extend the existing unified-neutral/teal Clear wayfinding interface. Preserve
  its layout, tokens, system fonts, 40px controls, keyboard behavior and draft
  protection. This is not a redesign and introduces no decorative assets/motion.
- No cloud, accounts, new browser engine, executable snapshot preview, recorded
  journeys, source-HTML mode, cross-origin/frame traversal or closed-root probing.
- First-release viewer: native **read-only unified text diff**. A tree inspector,
  semantic prose explanations, side-by-side source diff, arbitrary user scripts
  and configurable CSS-selector ignore rules are follow-up work, not prerequisites.
- Existing screenshot sensitivity and ignored rectangles retain their exact
  meaning. They never suppress DOM differences or redact exported evidence.
- Never modify real user projects, sessions or baselines during development/tests.
  Use temporary local stores and an owned loopback fixture website.

## 2. Repository and execution context

Repository root: `/var/home/olav/Projects/Regressee`.

- `crates/core`: project/run/baseline/review domain, evidence and image analysis.
- `crates/storage`: SQLite, immutable SHA-256 artifact storage, capture/review
  validation, ZIP portability and retention.
- `worker/product`: maintained Node/Playwright browser worker. Capture currently
  prepares a page, applies screenshot transforms, requires image stability and
  returns one PNG plus metadata through a strict private protocol.
- `crates/runtime`: worker protocol/process supervision and runtime identities.
- `crates/app`: GUI-independent operations, capture/check queues, review and local
  engine/API/report services.
- `crates/gui`: native GPUI shell, setup, evidence workspace, review/history.
- `crates/cli`: packaged `regressee` command-line client of the same local engine.
- Product/visual authority: `PRODUCT.md`, `DESIGN.md`; preserve historical
  qualification limits. `crates/gui` is the current GUI location even where older
  docs mention `crates/app/src/main.rs`.

Toolchain is Rust 1.98.1 (`rust-toolchain.toml`). On this host prepend
`/var/home/olav/.cargo/bin` to PATH; the inherited mise Cargo shim has no selected
version. Node is available at
`/var/home/olav/.local/share/mise/installs/node/26.7.0/bin/node`.

Private test output root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-dom-snapshot-20261017`.
For native linking on this host set `LIBRARY_PATH` to its `link-libs` subdirectory;
it supplies private linker-name symlinks to installed `/usr/lib64` libraries
(`stdc++`, `xcb`, `xkbcommon`, `xkbcommon-x11`). Do not install system packages or
overwrite the existing `.runtime/link-libs` symlinks, which target another OS.

## 3. Shared behavioral and data contracts

Implementations may choose idiomatic Rust type names, but must preserve these
semantics and document their concrete public types here before downstream work.

### Capture and reference identity

- Keep page/screen-size identity (`TargetKey`) distinct from check kind. A review
  unit is **page + capture profile + check kind** (`Screenshot` or `DomSnapshot`).
- Add a validated capture selection with exactly the three modes above. Freeze
  it and the snapshot-policy version in each run/baseline's recorded configuration.
  Checking an old baseline uses its recorded selection, not today's project draft.
- Each requested kind has an independent outcome: pending, captured, failed,
  skipped/cancelled. Disabled is **not requested**, never failed or unchanged.
- Successful DOM evidence requires its own artifact, byte count, serializer/policy
  version, source target/run/runtime and capture diagnostics. It requires no PNG,
  image dimensions, thumbnail or image-analysis descriptor.
- A shared navigation/login failure may fail both requested kinds. A screenshot
  size/encoding/stability failure must not discard a successful snapshot; a DOM
  serialization/limit failure must not discard a successful screenshot.
- Generalize successful-target and partial-baseline counting without double
  counting a page as two pages. Separately expose per-kind capture/check coverage.
  Partial reference coverage always requires explicit consent, with exact omissions
  retained. A missing counterpart must remain visible on later checks.
- Independent references may come from different retained runs. Verify the exact
  kind/target/source artifact on every load, comparison, approval and export.
- Preserve screenshot-only legacy records, source hashes/fingerprints, decisions,
  masks and reports. Do not silently manufacture snapshots for historic captures.

### Comparison and review

- Screenshot comparison keeps existing image algorithms and policies unchanged.
- DOM comparison is exact equality of the normalized captured representation under
  matching snapshot policies, not a screenshot percentage or heuristic threshold.
- Return per-kind unchanged/changed/not-comparable outcomes plus explicit capture
  failures and not-requested states. Failures never count as unchanged/success.
- Aggregate navigation/filtering must not let screenshot success conceal DOM
  changes/failures. Changes-first includes either changed kind; coverage always
  exposes failures/missing/not-compared evidence with a route to inspect it.
- Review decisions, notes, progress, Next unreviewed and baseline selections bind
  to the actual check kind and immutable analysis/source identities. A screenshot
  decision does not review or approve the snapshot. Existing screenshot decisions
  remain screenshot decisions, without rewriting their original history.
- Baseline update preview names screenshot/snapshot kinds, exact replacement/keep
  counts, source dates and retained history. Commit only selected eligible kinds,
  atomically, after explicit confirmation. Preserve stale-preview/revision/draft,
  corruption, provenance and eligibility guards. Never approve an unrequested,
  failed, unreadable, truncated or incompatible artifact.
- New mode/capture-policy changes require a new baseline capture. Do not relabel
  missing old evidence as a comparable baseline or silently visit new page paths.

### Partial coverage, independent updates and review identity

Checks request exactly the baseline's saved reference slots. Preserve both its
intended slots and explicitly omitted slots; do not retry omissions automatically.
An omitted snapshot in a both-mode baseline is a **baseline coverage omission**,
not an intentionally disabled kind. Creating/filling new baseline coverage requires
a new baseline capture, not an After-only selective approval.

| Before | After | Per-kind result / selective approval |
| --- | --- | --- |
| Valid compatible reference | Valid stable capture | Exact comparison; eligible if other normal guards pass. |
| Missing/explicitly omitted reference | Any | Not comparable / baseline coverage omission; no selective approval. |
| Valid reference | Missing, failed or cancelled capture | Not comparable with actual cause; no approval. |
| Valid reference | Complete but unstable capture | Not comparable due to instability; no approval, even if bytes happen to match. |
| Different policy/runtime/material identity | Any | Not comparable; no approval. |
| Kind disabled in recorded mode | Not requested | Not requested, not a coverage failure or unchanged. |

Invalid selected kinds reject the entire approval transaction. Unselected failing
or omitted sibling kinds do not block an eligible selected kind, and remain
truthfully recorded. Screenshot approval followed by DOM approval from the same
check must work: a fresh preview can rebase onto a compatible descendant of the
checked baseline only when every selected Before reference still exactly matches
the analyzed source. Inherit all unselected current references. Bind both checked
baseline and current parent, project revision, per-kind analysis/display revisions,
selected target/kind and exact Before/After source-run/artifact/policy identities.
If a selected Before changed, require a new comparison instead of guessing.

One immutable analysis revision belongs to one check kind. Existing screenshot
review identity `(analysis_id, target_key)` may therefore remain intact; DOM uses
its own analysis IDs and exact source bindings. Source provenance, not equal
content hashes alone, determines identity. Screenshot reanalysis must not clear
DOM decisions; changing either DOM source must not inherit an earlier DOM decision.

## 4. Snapshot capture, normalization and privacy

Create a dedicated worker serializer rather than using `page.content()` as the
persisted comparison contract.

1. Reuse existing authorization, scope/redirect/login checks, managed context,
   locale/timezone/theme/viewport and bounded page-readiness handling.
2. Separate common preparation from image-only work. DOM-only must not call
   `page.screenshot()`, image replacement, PNG ingestion/tiling, pixel stability,
   image-area or screenshot-height gates. Retain independent time/node/byte limits.
3. Both-mode snapshots describe the untransformed page, not Regressee image
   replacements, forced eager-loading attributes, injected animation styles or
   other tool-owned DOM changes. Capture before those mutations or precisely
   exclude/restore only owned changes. Never remove website-owned nodes by a
   generic ID/class heuristic. Record that screenshot and DOM are nearby samples,
   not a transactionally atomic browser state.
4. Walk nodes in document order. Include element/tag/namespace, ordinary sorted
   attributes and text; preserve text whitespace and child order. Preserve IDs,
   classes, URLs, ARIA and hidden ordinary content. Do not guess dynamic values
   away. Omit comments. Use deterministic UTF-8/newline/escaping rules and explicit
   open-shadow-root boundary markers without flattening slots or duplicating text.
   Traverse inert template content with the same exclusions and resource limits.
5. Exclude all script element bodies (including JSON/hydration data), `on*`
   event-handler attributes (case-insensitive local names), nonce attributes,
   `value` attributes on input/select/option/button/textarea, all live form values
   and textarea contents. Do not read password values. Omit `checked` on input and
   `selected` on option; never read selection/checked properties. Preserve
   ordinary form structure,
   labels, input types, names and button text. Omit iframe `srcdoc` contents too:
   they are an out-of-scope embedded document, potentially containing excluded data.
   Omit executable `javascript:` URL attributes under the versioned policy, without
   discarding ordinary link destinations. Apply every rule inside templates and
   open roots. This policy intentionally does not promise to scrub custom-control
   data attributes or arbitrary private text.
6. Include iframe host elements and ordinary attributes, not their documents.
   Count observed iframe hosts in coverage diagnostics. State that closed shadow
   roots are excluded and not reliably discoverable; do not fabricate a count.
7. Exclusions apply **before writing any snapshot artifact or private worker
   output**, not just in the displayed diff/export. Do not keep a raw unsanitized
   HTML backup. Page text, URLs, data/meta attributes and screenshots can still be
   sensitive. Never put snapshot content into telemetry/support diagnostics/logs.
8. Sample the canonical DOM with bounded retries to assess stability independent
   of PNG stability. Record unstable/readiness/coverage warnings honestly; unstable
   evidence must not become an eligible baseline merely because an image settled.
9. Initial hard bounds: 8 MiB UTF-8 canonical snapshot, 100,000 visited nodes,
   depth 512, existing bounded readiness deadline. Enforce bounds during walking,
   including shadow/template content, and again on worker-output ingestion. Limit
   failures are explicit; never silently truncate and compare a prefix as complete.
10. Version the serializer/exclusions as a material snapshot policy. Exact format
    and cross-language fixtures are part of the Phase A contract. Replaying a
    saved result must not apply a new policy without a new recorded analysis or
    require revisiting the website. Privacy/capture-scope changes require capture.

Phase A must freeze the exact canonical grammar and provide cross-language golden
bytes before B/C depend on it: document/doctype handling, namespace-aware names and
attribute sorting, text-node boundaries, shadow/template placement, marker escaping,
UTF-8/lone-surrogate and CR/LF treatment. Prefer flat preorder records to deeply
nested JSON. Count visited nodes even when excluded; bound allocation/escaping,
attributes per node and string lengths as well as final serialized bytes. Collector
sampling uses at most three samples separated by a versioned quiet interval within
its deadline; no two equal normalized samples means unstable/noncomparable.

Rust worker-output and ZIP ingestion must semantically validate these structural
exclusions **before retained CAS publication**, not just trust a matching hash or
claimed policy. Correctly hashed malicious-format payloads must be rejected without
echoing their contents. Harmless text resembling markup is permitted and inert.

## 5. Text diff contract

- One shared, deterministic, cancellation-aware Rust diff implementation feeds
  GUI, engine/CLI and HTML reports. Do not recompute diffs on the UI thread.
- Compare verified complete snapshots first; derive Changed/Unchanged independently
  of how many lines the viewer can render. Carry before/after artifact identities
  and policy versions into persisted analysis evidence.
- Produce unified hunks with old/new line numbers, context (default three lines),
  added/removed/context row kinds and added/removed **line** counts. Do not label
  line counts as node/semantic change counts. Repeated content must be deterministic.
- Bound CPU, memory and output. Prefer common prefix/suffix trimming plus a bounded
  edit algorithm. If its work budget is exhausted, return an honest coarse
  replacement hunk/explicit display-limit explanation while retaining the correct
  comparison verdict and access to both complete captured originals. Never mark
  the comparison unchanged because the pretty diff is unavailable.
- Text is untrusted data, never HTML markup, active links, JavaScript or a webview.
  HTML reports must escape it, including `</script>`, `</pre>`, `&`, quotes and
  markup-looking text. No report should fetch captured website resources.
  Source downloads/access remain plaintext or canonical JSON with safe MIME and
  `nosniff` where served, never `.html` executable previews. Encode separately for
  each HTML/JSON/attribute context; do not rely on one generic escaping function.

## 6. Native user experience

### Setup, capture and history

- Add a labelled **What to check** three-choice control in the existing Capture
  plan settings, with short plain-language descriptions. Screenshots only remains
  selected in new drafts and old projects. Preserve explicit Save and dirty guards.
- Explain that enabling a different mode requires a new baseline; compare against
  a saved baseline continues to use its frozen checks. No hidden auto-recapture.
- Keep screen-size controls for DOM-only and explain their effect when needed.
  Image-only settings should not pretend to affect snapshots.
- Task outcomes, partial-save confirmation and History identify enabled kinds and
  actual per-kind coverage rather than saying every artifact is a screenshot.

### Evidence and review

- In the existing evidence workspace add **Screenshots / DOM snapshot** tabs for
  both-mode evidence. DOM-only opens its snapshot directly; screenshot-only stays
  familiar and does not display a useless empty DOM tab. Failed requested kinds
  remain selectable for their cause/recovery; a disabled kind is not a failure.
- Keep page/profile and saved-baseline/this-check context persistent when switching
  tabs or hiding Results. Per-kind outcomes remain legible together.
- DOM tab replaces the image stage, not the app shell. Hide image View/Zoom/pan,
  pixel sensitivity and rectangle controls; retain relevant Details/review actions.
- Native unified diff uses existing semantic surfaces/ink and readable monospace
  only for source. Mark additions/removals with `+`/`-` and words/line numbers as
  well as color. Long source lines are reachable via horizontal scrolling or an
  explicit wrapping mode. Virtualize/bound rendered rows, not stored evidence.
- Include working search, next/previous change, expand collapsed context, text
  selection and copy. Match the existing search field/focus/scrollbar conventions.
  Empty/no-change, loading, error, before-only/after-only and display-limit states
  have clear causes and useful recovery, without stale previous-target text.
- Keep independent Unreviewed/Intentional/Needs fix decisions where applicable;
  unchanged results need no manual review. Explicit **Select After screenshot** /
  **Select After DOM snapshot** controls identify baseline updates by kind. The
  consent dialog lists both kinds when both are selected; switching tabs never
  selects or saves anything implicitly.
- Preserve drafts and revision-bound replies across tab/target changes and modal
  occlusion. Keyboard traversal follows painted order; hidden/disabled controls
  leave focus order. Readable at 1280x800 and 820x600 in light and dark themes.

## 7. Storage, sharing, engine and CLI

- Reuse immutable content-addressed storage with explicit DOM artifact kinds.
  Verify hash, kind, encoding, policy, byte/node limits and source binding before
  trusting imported/worker-provided evidence. No arbitrary worker paths.
- Add forward-only transactional migrations. Legacy absent fields default to
  screenshot-only; preserve strict unknown-field/version rejection. Any necessary
  archive/engine/worker format version bumps must be explicit and tested.
- Retention/reference accounting protects snapshot originals referenced by current
  or retained baselines, run history or analyses, including mixed-run independent
  approvals. Cleanup previews bytes and removes only unreferenced eligible files.
- ZIP round-trips snapshot-only and both-mode baselines with all provenance and
  safe project settings, never sessions/cookies/keys. Preserve legacy ZIP imports;
  reject corrupt/missing/misbound/oversized snapshots and unknown future formats.
  Format selection must account for both the selected baseline and exported
  current project settings: a DOM-enabled replacement draft needs v2 even when
  exporting an older screenshot-only baseline. Keep its frozen check plan distinct
  from the recipient's future-capture settings; do not silently overwrite either.
- Export preview and reports warn that snapshots can expose private hidden text,
  attributes and URLs; exclusions are not complete redaction. Preserve explicit
  export consent. Imports create a new local project and never visit websites.
- Engine APIs/configuration expose the three modes and kind-aware evidence,
  analysis/review/approval and artifacts. Preserve one local application authority,
  idempotency/stale-state/agent-control safeguards, and payload/resource bounds.
- CLI users can configure mode, capture/check, inspect per-kind results, retrieve
  snapshot/diff text, review/note by kind and preview/confirm independently selected
  baseline updates. Existing commands without a kind retain screenshot semantics;
  snapshot-only commands give actionable errors, not fake image IDs. Document exact
  new flags/subcommands in `docs/cli.md` and the portable skill references.
  The legacy default-kind rule applies only to kind-specific reads/reviews/updates:
  capture/check always obey the recorded plan, never force screenshots. Check exit
  precedence: incomplete/failed/noncomparable requested evidence = existing incomplete
  result (exit 3), otherwise changed in either kind = exit 1, complete unchanged = 0.
- Local HTML reports include both independent statuses and escaped unified DOM
  diffs/original-source access. Image crop commands stay screenshot-specific and
  reject DOM artifacts. Keep bulk report output bounded and explicit about limits.

### Runtime distribution and compatibility

Worker protocol and engine protocol have independent versions (currently product
worker 2, engine 6). Evolve strict peers/fixtures together; never relax old exact-field
validation. Snapshot bodies stay outside worker frames (1 MiB) and aggregate engine
metadata (64 MiB transport ceiling).

The shipped worker is embedded, not read directly from `worker/product/src`.
Update `scripts/official-runtime.py`, `scripts/refresh-product-worker.py`, bundled
worker ZIP, platform catalogs/provenance and explicit development-pack file lists
as required. A source-only change is not delivered support. Use a distinct worker
identity; do not relabel old runtime evidence or substitute runtimes automatically.
Retained old screenshots/reviews/history remain readable. Checks of a legacy
baseline still require its exact recorded compatible runtime; if unavailable, show
the existing truthful matching-runtime-required recovery instead of pretending the
updated browser is equivalent. Multi-generation runtime management is not added
implicitly by this feature.

Per-kind publication must survive a later sibling timeout/cancellation: publish
channel results before the terminal request, validate/detach successful output to
app-owned staging outside worker scratch, and persist successful slot checkpoints
without overwriting them on resume. Preserve bounded process-tree/scratch cleanup.

## 8. Phases, ownership and handoffs

Use shared-working-tree agents with disjoint write ownership. No agent may revert
others' work, commit/push, change dependencies casually, launch the user's app,
read real sessions, run unbounded live crawls or spawn further agents. New files
are fine within an owned subsystem. Coordinate shared type changes first.

| Phase | Deliverable | Depends on | Owner / verification |
| --- | --- | --- | --- |
| A | Concrete core contracts, legacy defaults, canonical format/diff and tests | This plan | Foundation implementer; independent source/test review |
| B | Safe DOM worker and strict runtime protocol/output handling | A format/contracts | Worker/runtime implementer; loopback/browser verifier |
| C | Storage migration, capture/check/analysis, independent reviews/approvals, ZIP/retention | A; B for live capture | Backend implementer; independent transactional/legacy verifier |
| D | Native setup, per-kind review, unified diff and consent/history integration | A/C stable API | GUI implementer; independent native/offscreen UX reviewer |
| E | Engine/CLI/report parity and user documentation | A/C stable API | Engine/CLI implementer; public-client/escaping verifier |
| F | Bundled-runtime refresh, integration repairs, whole-scope verification and final evidence record | B–E | Coordinator plus fresh verifier(s), not self-certification |

Independent explorations may run concurrently. Implementation may overlap only
where file ownership and stabilized interfaces make that safe. One agent owns
`crates/app/src/engine` and related API/CLI work; one owns the large GUI shell.
One Cargo compile/test slot at a time; read-only/source and Node work may overlap.
Use focused tests during development; the coordinator runs shared full gates once
integrated, then the independent verifier runs targeted adversarial acceptance.

Before editing GUI: load `impeccable`, read existing surface context and its craft
floor; preserve the incumbent native world. Limit visual iteration to a batched
inspection, one correction batch and one confirmation. No HTML mock is native
rendering/input/accessibility evidence. Native accessibility/cross-platform release
qualification remains separate and must not be claimed from backend tests.

Every handoff states files/types/API added, tests actually run, open failures and
remaining dependencies. Record implementation progress below; do not call an
unfinished foundation a completed feature.

## 9. Acceptance matrix and verification commands

All acceptance items are required unless a limitation is explicitly listed in
Section 1. An implementation omission is not converted into a deferred feature.

| ID | Required proof |
| --- | --- |
| T01 | All three modes create/save/reopen/capture/check correctly; defaults and old project/ZIP/run data stay screenshot-only. |
| T02 | Hydrated/live DOM changes are detected; attribute order/comments/tool injections do not create false changes; IDs/classes/hrefs/ARIA/hidden text and significant whitespace changes remain detectable. |
| T03 | Open shadow roots and nested template/shadow content serialize deterministically; iframe host coverage is explicit; closed-root limits are truthful. |
| T04 | Script/JSON bodies, inline event code, password/input/option/textarea/live form values and iframe srcdoc secrets are absent from every persisted/worker snapshot file, diff, report and ZIP. Ordinary private text remains warned about. |
| T05 | DOM-only never calls screenshot/image replacement/PNG paths and succeeds on a page exceeding image geometry limits within DOM budgets. |
| T06 | In both mode, screenshot success + DOM failure, DOM success + screenshot failure, unchanged image + changed DOM and changed image + unchanged DOM retain independent truthful outcomes. Shared navigation failures account for both. |
| T07 | Timeout, cancellation, unstable DOM, huge/deep DOM, oversized/malformed UTF-8, corrupt hash/kind, unknown policy/version and missing evidence never become unchanged or baseline-eligible. |
| T08 | Diff fixtures cover insert/delete/replace/repeated lines/Unicode/multiline text/empty bounds/large input/cancellation; bounded fallback preserves the exact verdict and labels line counts correctly. |
| T09 | Approving screenshot only leaves snapshot source unchanged and vice versa; approving both is atomic; stale/failed/incompatible references reject without partial mutation. Independent notes/decisions and Next unreviewed bind to kind. |
| T10 | Fresh process/reopened store, history inspection, mixed-run selective baselines, partial coverage, ZIP export/import and cleanup preserve all referenced snapshot evidence and provenance, with no session export/site visits. |
| T11 | Public engine/CLI consumer can execute the complete modes→capture→baseline→check→diff→review→selective approval flow; defaults remain backward compatible and image crops reject DOM. |
| T12 | HTML reports escape adversarial source strings, never execute/load captured content, and expose correct per-kind statuses and sources. Reports/engine output respect byte/page bounds. |
| T13 | Native setup/both-mode/snapshot-only/DOM failure/loading/history/approval/long diff render at 1280x800 and 820x600, light/dark; focus, search, change navigation, context expansion, copy and scrolling work in supported test scope. |
| T14 | Existing screenshot capture/analysis/masks/warnings/approval/ZIP/retention tests still pass; baseline pre-existing failures are listed separately, never silently repaired or reclassified. |
| T15 | Refreshed embedded worker/catalog has a distinct verified identity and includes every new module; legacy runtime evidence is never rewritten. Successful DOM publication survives sibling timeout/crash cleanup and fresh-process resume. |

Repository gate commands (prepend host PATH/LIBRARY_PATH from Section 2):

```sh
cargo fmt --all -- --check
cargo test --locked --offline --workspace --all-features -j 4 -- --test-threads=1
cargo clippy --locked --offline --workspace --all-targets --all-features -j 4 -- -D warnings
cargo build --locked --offline -p regressee-gui --features ui --bin regressee-desktop -j 4
node --test worker/product/test/*.test.mjs
```

Use the existing private pinned browser/runtime for browser fixtures if available;
record exact executable/runtime and environment switches. Skip reasons and failed
prerequisites are evidence, not passes. No automatic runtime downloads, network
authorization expansion, disabled sandbox/security checks or OS desktop changes.
GUI evidence should use existing GPUI offscreen infrastructure where supported;
capture only isolated synthetic stores. Store private PNG/log/runtime data under
the approved scratch root, not as shipping assets or committed private artifacts.

Isolated synthetic GUI/engine/browser test processes are authorized under this
plan: scratch-only data/cache/profiles, owned loopback origins, explicit test-only
scope opt-ins and exact process cleanup. This does not authorize attaching to the
user's running app/engine, real sessions, external sites/redirects, global security
disable flags or desktop manipulation. Tests requiring those remain blocked pending
separate permission. Existing opt-in historical soak/network suites are not enabled
merely because their test binaries are built.

Final verification report: `docs/research/dom-snapshot-verification-20261017.md`.
It must map T01–T15 to actual commands/assertions/artifacts, record independent
findings and repairs, distinguish source/simulated/native/OS-input evidence and
give a scoped PASS, FIX or BLOCKED verdict. No full-feature PASS with open material
acceptance gaps. User-facing docs must describe the implemented behavior truthfully.

## 10. Progress and concrete interface handoffs

- Planning questions answered; confirmed decisions are recorded in Section 1.
- Read-only architecture agents are mapping foundation/storage, worker/runtime
  and GUI/engine/CLI seams before the implementation assignments.
- Initial all-feature baseline test attempt hit host linker-name prerequisites
  before tests. A private host-specific link directory was prepared; baseline
  execution with that environment reached the partial checkpoint below. This is
  not a complete passing baseline.
- Worker source dependencies were restored using `npm ci --offline --ignore-scripts`
  and the default local npm cache; no package/lockfile or browser download changed.
  Source unit baseline: 9 tests pass; four existing browser suites require explicit
  private evidence/runtime environment and fail their setup assertion without it.
- Independent plan review amendments above fix partial-slot semantics, semantic
  ingestion validation, immutable review identity, inert source access and scoped
  operational test authorization. Architecture review additionally identified
  independent-approval rebasing, source/embedded-worker distribution and partial
  channel publication as mandatory implementation gates.

### Active implementation ownership (restart handoff)

These sessions share the working tree; do not duplicate their work or overwrite
their changes. The baseline is stopped; **coordinator owns the bounded final
workspace format/Clippy/test slot** (`final-gates.py` under approved scratch).
All sources are frozen pending independent diagnosis of the History render blocker;
no further GUI edit/capture round is authorized at this checkpoint. Independent
reviewers use direct probes/source only for now.
All other owners must wait for explicit handoff before
Cargo work. Source/Node work may proceed.

| Session | Responsibility / write boundary |
| --- | --- |
| `ses_eb48db482ffeN4zt0OX1aTIcOm` | Phase A delivered, focused tests pass; core repairs only if independent review requests them. |
| `ses_eb48c8412ffesZYQ0tp3v5XEYK` | Phase B, `worker/product`, runtime protocol/supervisor, worker refresh scripts/official bundled assets. |
| `ses_eb48c487bffehN4Z7LUkeb7pBw` | Storage portion of C, all `crates/storage`, migrations, checkpoints, analyses/reviews/approvals, ZIP/history/retention. |
| `ses_eb48bc892ffevU8q6nMgiVBBA8` | App portion of C, app `crawling/product/review/task/lib`, new capture/DOM service modules and new integration tests. |
| `ses_eb48b8bebffe7Lzz7UnsVdnxt8` | E, app engine/protocol/jobs/control/terminal, CLI, CLI/engine docs and portable skill. |
| `ses_eb48e597fffeKcTis1we4j21AM` | Full D ownership now: all `crates/gui`, plus `crates/core/src/shell.rs` for native controls/focus; component and full workflow integration. |
| `ses_eb44b8395ffeYgXfMepIm5hzEV` | Fresh independent live app/runtime crash-resume verifier; private probes only, no repository edits or Cargo until granted. |

Read-only architecture sessions `ses_eb4930390ffePTPB8FblQwkHzB`,
`ses_eb493037effeLX14nJVh2I5cEM`, `ses_eb4930358ffe4PAmL7zWRtM24W` and independent
plan reviewer `ses_eb48fbc00ffez1872DD7N03v0A` are completed. Their findings are
incorporated above; no verification PASS for the implementation has been issued.

Coordinator owns this plan, cross-boundary integration/fixture repairs after
handoffs, user/product overview docs and the final independent verification report.
Send Phase A exact format/types and B/C/E public APIs to dependent agents before
their integration passes. Final verification must use fresh agents, not the author
of the subsystem being verified.

### Independent worker first-slice review — FIX, repairs assigned

Verifier `ses_eb4872ae8ffe9Sr25x2hw1N6qI` reviewed the first delivered collector/direct
capture API, not protocol/CAS/runtime shipping. It ran 23 selected author tests
and 15 independent assertion groups under catalog-verified sandboxed Chromium;
the assertion groups include reproducing defects, not 15 acceptance passes.

Open findings assigned back to worker implementer:

- W1: template/open-root descendants under an excluded ancestor bypass visited
  node/depth accounting (100,001 comments and 600-depth counterexamples).
- W2: DOM sampling consumes the PNG readiness deadline; both-mode PNG timeout
  reproduced six times where screenshot-only succeeds. Split channel stage budgets.
- W3: optional `onChannel` means a DOM failure aborts both-mode direct capture
  instead of attempting PNG. Make isolation an enforced API invariant.

Independent repro:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-dom-snapshot-20261017/worker-verification/reproduce.sh`.
Source/canonical/protocol/ingestion/bundle/durable-publication acceptance remains
open. The verifier must score repairs after the implementation handoff; callback
publication alone is not T15 crash/resume proof.

### Pre-feature baseline checkpoint

Source commit: `8381759ae24e468db022864e610784a475ac3fc4`, clean product sources at
build time. With host PATH/private linker names, all-feature compilation succeeded.
Baseline tests passed app unit (60), agent control (6), analysis/review (9), engine
(18), partial baseline (2), CLI workflows (19), core (56), GUI library (100), plus
the reported other small suites. The synthetic 100/1000-page qualification test
passed in 702.71 seconds; it is not a live website or release qualification.

The broad command was stopped after 1,131 seconds to release the compile slot.
GUI shell execution was incomplete and had already observed three failures:
`changed_results_first_analysis_and_recorded_checks_default_without_resetting_refresh`,
`all_inline_disclosures_attach_content_restore_focus_and_do_not_mutate`, and
`loading_feedback_has_one_owner_and_keeps_its_icon_and_copy_visible`. These are
pre-feature observations, not regressions attributed to DOM work, and not an excuse
to ignore new failures. Remaining tests were not established by this baseline.

Original log:
`/var/home/olav/.local/share/opencode/shell/90ea0b944e3e5aba5984218936aa6888d6e83ecd/sh_14b6e229f001YVeo4ECf7Ww3YC.out`.
An exact clean-source GUI test binary was preserved in approved scratch
`baseline/regressee-desktop-tests-clean-8381759`, SHA-256
`71bb07e2da1e9e6c88c7f75ac3e40421cc2454954c20e70ed8162026a521fcfe`, with
`baseline/binding.json`. It can establish a bounded pre-existing failure comparison
without rerunning the expensive whole baseline; it is not the current implementation.

### Phase A delivered — concrete contracts

Core-only author verification: 56 unit tests, 16 new DOM contracts, two Node
cross-language fixtures, core Clippy with warnings denied and direct rustfmt pass.
Independent review returned FIX; author repairs below await rescoring. These are
not end-to-end acceptance results.

- `CaptureMode::{ScreenshotsOnly, ScreenshotsAndDomSnapshots, DomSnapshotsOnly}`.
  `CapturePlan { version, mode, dom_policy }` defaults to version 1/screenshots/no
  DOM policy. Project/run/baseline `capture_plan` omits this default in legacy
  serialization. `CaptureProfile` is unchanged.
- `CaptureTarget.dom_outcome: DomTargetOutcome` is separate from the screenshot
  `outcome`; both have truthful not-requested semantics. DOM errors use
  `DomCaptureError::{Capture { code }, LimitExceeded, SerializationFailed}`.
- `RunRecord.dom_captures: Vec<DomCaptureEvidence>` holds metadata only, not text.
  Metadata exposes `verify_snapshot(bytes)`, `validate_for_run(run)` and
  `baseline_ready()`. `dom_source()` constructs immutable comparison identity.
- Baselines add `dom_references`, `source_run`, `intended_targets`; use
  `target_keys()` for the saved target union, `saved_units()`,
  `intended_units(frozen)`, `omitted_units(frozen)`, `frozen_source_run()`,
  `source_runs()`, `freeze_source(frozen)` and `validate_check_slots(run)`.
  **Freeze source before replacing legacy references.** DOM-only needs no PNG ref.
- `DomAnalysisId` is distinct. `dom_snapshot.rs` supplies `DomSourceIdentity`,
  `DomAnalysisRevision`, `DomAnalysisResult`, `DomReviewDecision` and
  `DomApprovalBinding`. Exact source provenance is bound; unstable/incompatible
  sources cannot get an unchanged verdict or approval. `CheckKind`/`ReviewUnitKey`
  describe selection; legacy screenshot reviews retain their original encoding.
- CAS adds `ArtifactKind::DomSnapshot` and `ArtifactOwner::DomAnalysis`. Storage
  must update both SQL constraints. Archive helpers retain screenshot v1; DOM v2.
- `compare_dom_snapshots()`, `diff_text()`, `format_source()` are the shared
  canonical/comparison/source/diff implementation. Read concrete signatures before
  adapting; do not create another normalizer or diff engine in downstream crates.

Canonical v1 is UTF-8 NDJSON with **exact final LF**:

```text
["regressee-dom",1]
["document",0]
["element",1,"http://www.w3.org/1999/xhtml","div",[]]
["text",2," a\r\nb "]
```

Other records: `["doctype",depth,name,publicId,systemId]`, `["shadow",depth]`,
`["template",depth]`. Element record name is the **actual local name**; append a
sixth field with actual prefix only when `element.prefix !== null`. For example
parser HTML name `a:b` remains `["element",depth,htmlNamespace,"a:b",attrs]`,
whereas a genuinely prefixed element is
`["element",depth,htmlNamespace,"b",attrs,"a"]`. Explicit null/empty/invalid prefix
fields reject. Element attributes remain `[namespace,qualifiedName,value]`,
strict UTF-8 `(namespace,name)` ordering, empty namespace `""`. Convert lone
surrogates to U+FFFD (`toWellFormed`) before bounds/sorting; use JSON.stringify-
compatible escaping. Preserve text boundaries/whitespace. Light children precede
a terminal open-shadow boundary. HTML templates contain one template boundary.
Script/textarea/iframe hosts have no serialized children. Golden bytes live under
`crates/core/tests/fixtures/dom-v1*` and are the cross-language source of truth.

Concrete bounds tighten the generic Section 4 string budget: 8 MiB canonical,
100,000 visited nodes, depth 512, 256 attributes/element, **64 KiB/string**,
256 KiB aggregate attributes/element and 1 MiB encoded record. Policy 1 allows
at most three samples with 100ms quiet interval. Collector visits still include
excluded reachable nodes, templates and shadow roots, without retaining their
private content. Rust rejects noncanonical/unsafe structure before CAS publication.

App first-pass service handoff (not compile-ready until worker/storage integrate):
`dom_snapshot::{original_source, compare_originals, analyze_run,
analyze_run_with_options}` and `review::analyze_saved_run`. Direct rustfmt/diff
checks passed; no Cargo/application acceptance claimed. Runtime event/detachment
integration and legitimate environment URL mapping need completion/verification.

Latest app handoff keeps `review::analyze_run` screenshot-only; use
`review::analyze_saved_run(store, run, screenshot_policy, cancel)` for dispatch.
DOM functions take `(store, run, cancel)` or `(store, run, options, cancel)`;
`task::target_counts(run)` preserves page/profile versus sibling-channel success
semantics. Exact saved-slot validation is adapted. `capture_one` is intentionally
not accepted/compile-ready until B provides final per-channel wire and typed
detach APIs. Core metadata mapping issue is independently tracked, not bypassed.

### Independent core review — FIX, repairs assigned

Reviewer `ses_eb47db649ffewxlUIbw4SqI1FJ` independently passed 56 unit/16 contract
tests, core all-target Clippy and two Node fixtures. Additional probes included
45,360 diff-option combinations and a four-million-cell diff. Twelve probe groups
comprised six passing groups and six counterexample groups exposing these defects:

- C1: metadata validation must require the exact approved mapped environment URL,
  not logical inventory URL or a permissive either/or rule.
- C2: XML-only name assumptions reject browser-observable `@click`, `123`,
  `[binding]`, `:class`, `a:b:c`, element local name `a:b` and doctype `123`, and
  conflate distinct null-namespace colon attributes. Preserve exact DOM identity,
  privacy, structure and canonical bytes with DOM-appropriate naming semantics.
- C3: legacy partially saved baselines whose original references were all replaced
  by selective approvals cannot recover frozen original intended coverage from the
  first current reference. Core and storage must use explicit verified ancestry
  recovery, not arbitrary same-inventory records or guessed coverage.

Repairs are assigned to core; storage has been notified of C3 integration, worker
of possible name/format clarification, and app's mapping blocker is C1. Independent
repro root: approved scratch `core-verification/` (including `independent.rs`,
`run-probes.py`, browser-generated canonical cases and `html-names.mjs`). The same
reviewer must rescore repaired source; focused author passes do not close findings.

Author C1–C3 repairs delivered: exact approved mapped-request binding; browser-DOM
name handling plus optional canonical element prefix (unprefixed golden bytes
unchanged); and `recover_legacy_source(&ancestors, &original_new_baseline_run,
verified_run_loader)`. Ancestors are nearest parent first, ending at original
capture boundary. The helper validates ancestry, exact originals/replacements,
coverage/configuration and Check Before/After provenance before mutating the copy.
Unresolved legacy descendants now return `None` from `frozen_source_run()`.
Storage must integrate verified ancestor loading before check/export/restore or
other frozen-config operations; direct first-reference guessing is forbidden.
Core author ran 56 unit, 20 contract, three Node goldens, core Clippy/rustfmt/diff
checks successfully. Independent rescoring is running; downstream integration is
not implied by those checks. New browser-name fixture is `dom-v1-html-names.ndjson`.

Rescore: **C1 resolved, C2 resolved, C3 partial**. The remaining terminal-boundary
parent cycle (`current -> intermediate -> original -> current`) was then repaired
with terminal-to-current/intermediate rejection and unrelated-older-parent positive
tests; 20 author contract tests pass. Targeted independent C3 recheck passed five
groups using direct rustc/source-bound artifacts without touching Storage's slot.
**C1, C2 and C3 are resolved at core-contract scope**. Binding/results are in
`core-verification/c3-terminal-fixed-{binding.json,results.log}`. Storage/app use
of the corrected contracts remains an independent integration gate.

### Phase B authored — integration contract

Product wire **3**, worker **6.0.0**:

- `CaptureTarget { key, address, profile, session, plan: CapturePlan }`.
- Nonterminal `ChannelCaptured { key, capture }`, `ChannelFailed { key, kind,
  code, retryable }`, `ChannelWarning { key, kind, code }`.
- Terminal `CaptureCompleted { key }`; capture is a strict kind-tagged
  `CapturedChannel::Screenshot { output_token, artifact, width, height }` or
  `DomSnapshot { output_token, artifact, policy }`. Bodies never enter frames.
- Runtime `Worker::request_detached(request, deadline, cancelled, staging,
  &mut FnMut(Event, Option<DetachedCapture>) -> Result<()>)` validates/detaches
  outside worker scratch **before** the callback. `DetachedCapture` has key,
  capture, artifact/metadata paths and typed `DetachedMetadata`. App must persist
  the corresponding successful channel checkpoint synchronously at this boundary.
- Use `protocol::capture_request_budget(profile, plan)` and kind-specific
  `output_path_for`/`metadata_path_for`. Handshake has no pixel dependency;
  `InstallSmoke` explicitly performs installation PNG smoke testing.

Author reports W1–W3 repaired and 52 Node/source/owned-browser checks passed;
independent rescoring is running. Rust tracker/detachment tests are authored but
not yet run. Existing Phase 4 browser fixture additionally needs an unavailable
absolute DejaVu font path; no system modification is authorized to fake that gate.

Offline bundled refresh produced ZIP SHA-256
`a765834f2aca7d6a681154b72780015f446f05a6515079f39b4858860d4128ff`, four verified
catalogs, seven source files and unchanged dependency files. Independent bundle
verification and actual supervisor timeout/crash/resume proof remain pending.
Coordinator added `src/dom.mjs` to the development-pack list and adapted legacy
app fixture requests/events to wire 3; runtime owner is adapting its ownership tests.

### Phase E authored — GUI/client contract

Engine wire **7** adds `Operation::Dom(engine::dom::Command)` returning
`Outcome::Dom` / `JobReply::Dom`. Exact public schemas are
`crates/app/src/engine/{dom,text}.rs` and `docs/local-engine.md`. Commands cover
focused results/artifacts/source/diff/inspection/analysis/review/history and
explicit-kind approvals. `SourcePage` uses UTF-8 byte paging (256 KiB max),
`DiffPage` contains summary and paged core hunk rows (1,000 rows max, also subject
to transport byte bounds). `control::Selection.kind` defaults to Screenshot;
DOM selections resolve separate immutable DOM analysis identities.

CLI exposes mode configuration, `--kind dom`, DOM source/diff/analysis/reviews,
`--dom-analysis`, and atomic `preview-kinds` / `accept-kinds`. Source/reports are
inert and bounded by contract. Author formatting/frontmatter checks passed; no
compile/CLI/actual-report-browser acceptance yet. CLI worker fixtures are being
synchronized with B. Storage must expose a screenshot-only exact-run reader that
permits capture inspection without an analysis and does not require sibling DOM.

Fresh independent engine/report source reviewer
`ses_eb46b2cf8ffeP087u4Q8OTocx3` is reviewing authority, byte bounds, exact consent,
privacy and escaping without Cargo. Actual compiled lifecycle/report-execution
verification follows integration. GUI owner consumes the real focused IPC API,
not Store or mock-only data.

Coordinator fixture adapters so far (no behavior/assertion weakening): existing
app approval/bulk/analysis/runtime-setup fixtures explicitly remain screenshot-only;
agent selection explicitly requests Screenshot; product-flow uses wire version
constant, default plan and new channel events. Direct formatting checks passed;
their compilation/execution remains part of integration gates.

Latest app source handoff is complete against B: typed detachment and synchronous
per-kind checkpoints while tasks run, sibling-success preservation, pending-slot
resume, DOM-only no image paths, Store-resolved frozen source/mapped URLs,
explicit installer smoke and screenshot analysis over screenshot slots only.
Formatting/diff checks pass; no Cargo/application tests yet. Wait for runtime/
storage build gates, then compile/test app together with the shared engine API.

### Storage C delivered — concrete Store handoff

Author verification: **55 storage tests pass, two existing profiling ignores**;
all-target storage Clippy with warnings denied, owned formatting/diff checks pass.
No app/GUI/workspace acceptance is inferred. Fresh independent reviewer
`ses_eb46764dfffebRoROhO36MZfiW` is reviewing storage/provenance/ZIP/retention and
adversarial private-store probes without Cargo while runtime owns the slot.

- `KindTargetCheckpoint`, `append_run_kind_targets`, `ingest_dom_snapshot`.
- `load_dom_run`, `load_dom_baseline`, `read_dom_snapshot`, `load_run_kind`,
  `load_baseline_kind`, `viewer_dom_evidence`, `dom_evidence_for_run`.
- **`screenshot_evidence_for_run(project, run)`** is screenshot-scoped exact-run
  evidence, permits absent analysis and ignores DOM files. Do not substitute the
  legacy aggregate reader in kind-specific engine/report/viewer paths.
- **`frozen_baseline_source(id)`** returns a normalized baseline plus metadata-only
  source run, recovering ancestry without rewriting historical JSON. Inspect exact
  signatures/types in storage; never fall back to the first current screenshot ref.
- **`baseline_eligible_coverage(run)`** excludes unstable DOM slots for save consent.
- DOM save/load/display/invalidate analysis methods, current reviews and paginated
  review history; separate typed analysis IDs keep decisions independent.
- `dom_approval_preview`, `accept_dom_selected`, and **`approval_preview_kinds` /
  `accept_selected_kinds`** for one atomic explicitly selected multi-kind update.

Migration 10, semantic pre-CAS ingestion, per-kind immutable checkpoints,
compatible-descendant approvals, screenshot-v1/DOM-v2 ZIPs, selected-only source
export and DOM-original retention/history are implemented. Author regressions
include legacy intended-two/saved-one after full reference replacement, reopen,
ZIP, further approval and terminal cycles; sibling corruption/omissions, atomic
rollback, cancellation and correctly hashed unsafe snapshots. Independent
transactional/source-bound verification is still pending.

### Expanded independent findings — worker and engine

Worker reviewer independently resolved **W1–W3**, with 36 selected checks, an
11-check resource/browser suite and 15 repaired-interface probe groups. Exact
shipping-bundle wire/metadata/privacy/permissions, pixel-free handshake/DOM-only,
installer smoke, names/prefixes and strict old-wire/runtime rejection were tested.
All four catalogs/seven source modules/vendor bytes matched the then-current
`a765…ff` bundle. New findings:

- **W4 open, worker owner repairing:** expired asynchronous screenshot preparation
  can emit a warning after `ChannelFailed`, correctly rejected by the Rust tracker.
  Stalled-font repro in extracted shipping bundle: screenshot-only 5/9 and both
  1/3. Suppress/stop expired-stage side effects; do not weaken protocol ordering.
  Rebuild/re-audit bundle after source fix. Evidence: `worker-verification-v2/`,
  especially `wire-a6b1hD/report.json`.
- **W5 resolved independently:** development-pack whitelist omitted
  `test/fixtures/capture-events.ndjson`, which bundled protocol tests read.
  Coordinator added it; fresh exact-whitelist subset passed seven Node tests,
  zero failures/skips. Evidence: `worker-verification-w5/subset-fe3s40do/`.

Fresh engine/report source review `ses_eb46b2cf8ffeP087u4Q8OTocx3` returned FIX:

- **E1:** screenshot terminal/agent/viewer reads still use aggregate readers and
  fail on unrelated DOM corruption. Replace screenshot-specific paths with the
  delivered `screenshot_evidence_for_run`; combined operations remain full-kind.
- **E2:** multi-kind acceptance binds contexts/combined baseline but not every
  nested preview/display field. Tampered `dom.inherited` or nested baseline can
  misstate consent. Engine and storage owners must validate/regenerate every
  displayed value; selected mutation widening was not demonstrated.
- **E3:** results are row-bounded but not byte-bounded. An independent escaping
  model of 169 valid maximum-warning rows exceeds retained-job capacity and can
  drain the engine. Add byte-aware paging/bounded failure; oversized reads must
  not poison engine durability/health.
- **E4:** recognized cancellation during DOM report diff becomes generic terminal
  failure. Use typed cancellation classification, never user-controlled string
  matching, and do not rewrite already committed success after late cancellation.

E owns E1–E4 engine repairs, storage owns E2 transactional guard, app capture owner
provides typed DOM cancellation. Source-only passing observations cover existing
authority/lease/idempotency guards, exact source bindings, body-free aggregate
data, focused source/diff budgets, inert report encoding/CSP and no-clobber paths.
They are not compiled CLI/IPC/browser acceptance. Repro/model hashes:
`engine-verification/source_probes.py`. Fresh storage reviewer was notified of E2.

E2 storage source repair is delivered (APIs unchanged): equality against the full
regenerated preview covers nested baselines/counts/provenance/display labels;
nested baselines share the outer generated version ID/date. Nineteen nested-field
tamper cases plus single-kind adapter checks assert rejection without mutation.
Formatting passes, but the prior 55-test result **predates this repair**; rerun and
independent rescore remain required. App now exposes typed DOM cancellation as
`regressee_core::analysis_engine::Cancelled`, while capture keeps
`WorkerFailure(Cancelled)`, for E4 engine normalization.

### Independent storage review — three compatibility repairs required

Reviewer `ses_eb46764dfffebRoROhO36MZfiW` ran the source-bound existing storage
suite (55 pass/two profiling ignores), plus fresh private direct-rustc probes
(14 pass/two failures). Separate child processes proved committed successful
slots survive interruption/recovery/resume and sibling cancellation at storage
scope. Passing probes include unsafe hashed ingestion before CAS, kind-scoped
corruption isolation, immutable source/review bindings, descendant/atomic approvals,
selected-only exports, fresh recipient IDs and no session export. **E2 storage
consent repair independently passes** five altered nested-field probes.

New open findings assigned to storage:

- **S1:** historical `Product::capture` at commit `8381759` wrote embedded immutable
  run inventory but no inventory SQL row; the schema permits this. New legacy
  resolver requires that row and breaks baseline/viewer/history/export/restore/check
  despite original run loading. Recover verified embedded inventory only when the
  row is absent, using verified frozen source/ancestry. Existing corrupt/mismatched
  rows remain errors. No arbitrary current-settings fallback or mutation on reads.
- **S2:** exporting an old screenshot baseline after changing the replacement
  project's mode to DOM selects archive v1 but includes DOM settings, then rejects
  itself. Select version from all exported requirements, preserve draft-versus-
  baseline plan semantics and test round-trip/future-capture behavior.

Repro/evidence: approved scratch `storage-verification/run-private.py
source-bound-final-recheck`, corresponding results/binding JSON and
`historical-single-page-source.txt`. Author source repairs are pending; T01/T10/T14
are not accepted while these faithful historical shapes fail.

Additional independently confirmed **S3**: a fresh screenshot/default-settings
archive labelled v1 includes `baseline.source_run` / `intended_targets`, which
the actual historical v1 `deny_unknown_fields` reader rejects. Default capture
plan and empty DOM fields are already omitted; these two internal hints are not.
Evidence: `storage-verification/v1-pre-repair.{zip,json}` and matching binding/
results, plus `historical-v1-schema-source.txt`. Storage owner must export genuine
strict-v1 portable fields (without mutating stored baselines), or explicitly choose
v2 when extended facts cannot be represented faithfully. Import must not accept
v2-shaped nondefault hints falsely labelled v1; reconstruct local hints only after
validating the portable input. Simple screenshot/default archives remain old-reader-
compatible v1; DOM-enabled current settings still require v2 under S2.

### Native D source handoff — awaiting integrated GUI build

GUI source integration is authored: three saved modes; per-kind statuses,
reviews/notes/Next; real IPC source/diff with complete bounded pagination; explicit
kind selections/consent; history/stale-source/unstable-save guards; DOM-only image
action and focus exclusion. Public GUI seams: `ui::dom_diff`,
`dom_source::{comparison, original}`, and jobs `DomViewerIdentity`,
`DomPresentation`, `submit_dom_viewer`, `cancel_dom_viewer`, `take_dom_viewer`.

Non-Cargo source/model gates: 13 source/model tests, 14 shell-model tests,
three queue-model tests, component metadata checks with warnings denied, owned
formatting/whitespace. These are **not** integrated GPUI/native acceptance.
Integrated build, authored real-IPC/native workflow tests and the planned
32-capture light/dark matrix have not run. App/shared engine gets the build slot
after runtime; GUI follows. Baseline shell failures remain recorded, not repaired
or independently rerun by this source handoff. No new dependency/assets/commits.

### Runtime Rust gate and W4 author repair delivered

Author ran runtime all-feature Cargo tests: **52 pass, six intentionally ignored**;
all-target/all-feature Clippy with warnings denied passes. Node/source/browser
checks: 56 pass. W4 lifecycle guards block expired-stage warnings/continuations;
screenshot expiry closes context, DOM expiry cannot resume writes/metadata, only
exclusively created aborted files are cleaned, and callback errors still propagate.
Tracker ordering/cardinality is unchanged. Author reran the reviewer wire probe on
the refreshed bundle: zero findings in 12 stalled-font cases. Independent W4
rescore/new bundle audit is running; app durability is still separate.

**Current final-candidate worker ZIP SHA-256**:
`b5f900ff611bb8a7a28b5b19100675341774abadd3ada8b44ad912c8630f56a8`.
Four catalogs/provenance/seven sources refreshed, vendor bytes unchanged. This
supersedes the earlier `a765…ff` candidate, whose audit does not cover W4 repairs.
Author evidence: `worker-b/w4-final-shipping-audit-moudm9qu/wire-mxg4n6/report.json`.

E source handoff now includes E1–E4 repairs and CLI wire-3 fixtures with core
canonical/prefix goldens; five Python fixture tests/format checks pass. Compiled
public workflows, budgets/health, cancellation and actual report-browser safety
remain pending. App/shared-engine compile is underway, then CLI and GUI gates.
An actual engine-generated adversarial report bundle will be retained under
approved scratch `engine-report-evidence/` for independent browser execution/
resource-attempt checks; no manually assembled mock counts as that evidence.

### Worker and E source rescores, app compile checkpoint

Independent W4 verification passed **12 real stalled-font cases** (nine screenshot,
three both after DOM publication), no warnings after settlement/no events after
completion through exit, valid retained DOM hashes, and four lifecycle/collision/
callback probes. The repaired `b5f…56a8` bundle matches all catalogs, sources and
vendor pins. **W1–W5 resolved in worker/bundle scope**. No actual app crash/resume
claim follows from this alone. Evidence: `worker-verification-w4/reproduce.sh`,
`wire-qFPUQ2/report.json`, `lifecycle-033v8E/report.json`, `bundle-audit.json`.

E reviewer resolved E1–E4 at **source/model scope**: full canonical preview rejects
45 scalar tamper cases, 169 heavy warning rows reconstruct losslessly over 17
pages (maximum 3,937,252 serialized bytes), typed cancellation rejects spoofed
strings/preserves success. Source/diff/hunk public fields are unchanged; results
consumers must follow `next_offset`, never assume `offset + requested limit`.
Compiled corruption/Prepare/Accept/health/cancel and report-browser proof still
required. Probe: `engine-verification/rescore_e1_e4.py`.

App integrated compilation succeeded. Author commands passed 85 app lib/new DOM
tests (one ignore), plus 20 engine/partial-baseline tests (one ignore). Eight of
nine original analysis/review tests passed; one fixture tried creating a Check
with a changed image-replacement profile, now rejected at frozen-plan admission.
Coordinator adapted it to assert the exact early rejection and unchanged baseline,
while independently exercising the same real-PNG `MaterialProfileMismatch` gate.
This preserves the safety assertion; its rerun is assigned to E's current slot.

App Clippy exposed one new large enum (E-owned) and three pre-existing duplicate
fixture modules (confirmed at HEAD). Coordinator registered one cfg-test-only
`crate::approval_fixture`; E aliases existing test names to it and fixes the enum.
No shipping fixture dependency or blanket lint suppression is intended. E now owns
focused app-Clippy/storage-regression/CLI gates; GUI follows. S1–S3 source fixes
are delivered with formatting only, APIs unchanged; fresh independent rescore and
focused post-fix Cargo tests remain required.

### S1–S3 independent rescore complete

**All three resolved at storage/direct-Rust scope.** Fresh private harness:
22 passing tests plus two child recovery executions; fresh direct-compiled current
storage unit suite: 60 pass/two profiling ignores. E2's five consent-tamper cases
remain passing. No Cargo or product edits by the independent reviewer.

- S1 zero-inventory-row historical data passes seven previously failing paths plus
  subsequent check/approval/export, without changing rows/revision/historical JSON.
  Existing malformed/unknown-field/conflicting rows and ancestry cycles reject.
- S2 all **nine** frozen-mode/current-draft-mode pairs ZIP-round-trip; subsequent
  checks retain frozen profile/address/mode while future settings remain distinct.
- S3 actual screenshot/default v1 exports deserialize under the historical strict
  whole-manifest/project/baseline/provenance schema. Null/empty/default-valued new
  fields reject on v1 input. Nonredundant hints use lossless v2 without unselected
  sibling originals; historical schema imports remain valid.

Bindings/results: `storage-verification/{s1-s2-s3-final-results.log,
v1-final-results.log,current-storage-unit-results.log}` and matching JSON.
E's current slot still needs ordinary Cargo/Clippy regressions. These resolved
storage findings do not establish actual app/worker crash recovery or native UI;
fresh verifier `ses_eb44b8395ffeYgXfMepIm5hzEV` is preparing that real-runtime probe.

### Compiled E gate and B1/B2 capture fixes

E author gate passed app/CLI all-feature/all-target Clippy, app lib 75 tests/one
ignore, original analysis/review **9/9** (including parent stronger-admission
fixture), storage **60/two profiling ignores**, real-IPC byte-budget/engine-health
and running/late-cancellation tests **2/2**, Python worker fixtures **5/5**.

Ordinary-umask CLI workflows exposed two actual capture bugs: **eight pass,
15 fail**. Diagnostic private umask isolated the second bug (**22 pass/one fail**),
not a public-path acceptance pass. E1 corruption/E2 tampering IPC cases passed
only diagnostically at that checkpoint. Root causes:

- **B1:** `crawling.rs` staging creation produced 0755, correctly refused by strict
  runtime private-directory validation. App source now uses explicit 0700-owned
  TempDir creation, validates parents, preserves exclusive ownership and lifetime.
  Legacy single-page capture uses the same helper. No umask/global guard changes.
- **B2:** pre-worker attempt/skip metadata still used screenshot-only checkpoints,
  rejecting DOM-only `NotRequested` screenshot slots. App source now routes these
  through `KindTargetCheckpoint` / `append_run_kind_targets`, preserving siblings.

New source regressions: private-at-creation/unique/drop lifetime; symlink/missing
parent rejection without writes; DOM-only attempt/resume preserving not-requested
shots; scheduler helper included in sibling-publication/resume test. Formatting
passes; **no post-fix Cargo/normal-umask workflow rerun yet**. GUI may compile but
must wait for corrected backend build/CLI rerun before binding final visual proof.

E handoff/logs: approved scratch `e-cargo-handoff.md`. Only Rust constructor change
is `Command::Review(Box<DomReviewDecision>)`; wire shape unchanged. Actual engine-
exported adversarial report, receipt/preview/source bindings and hashes are at
`engine-report-evidence/actual-cli-dom-report-01/`; browser reviewer is checking
that artifact independently. It was produced during diagnostic-umask verification,
not evidence that uncorrected ordinary capture flows worked.

### B1/B2 source repair and actual report-browser proof

App B1/B2 source repair is ready with no Cargo rerun yet. New regressions are
`capture_staging_is_private_at_creation_unique_and_owned_until_drop`,
`capture_staging_rejects_symlinked_or_missing_parents_without_writing`, and
`dom_only_scheduler_attempt_checkpoint_and_resume_preserve_not_requested_screenshot`.
Existing sibling-publication/resume also exercises the scheduler metadata helper.
No global umask changes, runtime weakening or invented screenshot work.

Independent T12 actual-report verification **passes at the tested artifact scope**:
sandboxed Chromium 156 loaded the actual export through HTTP and `file://`; six
export hashes/source/review bindings and independently recomputed fingerprint
match; no captured execution, resources, active markup/forms/CSP violations;
four downloads match bytes and eight direct source views are plaintext. Visible
privacy warnings and seven absent exclusion canaries are export evidence, not
standalone proof of live collector redaction. Original exports unchanged and all
owned browser/server processes closed.

Evidence: `engine-report-verification/{scope.md,reproduce.sh,
static-bindings-result.json,results.json,requests.json,execution-sentinels.json}`
and two PNGs. This is not full T12 matrix/client acceptance: both/failure reports
and bounded output stress checks remain pending; reviewer is extending bounded
checks without giant output. Live durability verifier has a final source-bound
normal-umask (`0022`) run in progress with B1/B2 included, **no verdict yet**.

### First native compile/IPC gate and B3 auth integration

All-feature native targets compile. GUI author ran **42 focused tests**, including
actual IPC root reopen/refresh with either sibling corrupt, valid-sibling review/
approval, same-check A→B→C independent approvals and combined-kind consent.
`workspace_jobs.rs` uses kind-scoped IPC recovery with immutable metadata and
explicit per-kind read errors. No images or full native/visual pass yet.

Actual DOM-only reopen exposed **B3**, an old `auth.rs:592` first-screenshot-ref
assumption. App owner now also owns narrow auth integration: use metadata-only
`frozen_baseline_source`, saved target-page union and frozen discovery for mapping,
not current drafts/physical sibling reads. Four production auth worker requests
still hard-code wire 2 (install/open/confirm/probe); change those to
`PROTOCOL_VERSION` 3. **BoundSession version 2 and manifest version 2 remain
unchanged**; no credential/session migration or weakened origin authorization.

App owns a short focused regression slot for B1/B2 plus B3; E ordinary CLI then
final native slot follow. Two unchanged strict GUI lint warnings (scrollbar type
complexity and one-element zoom assertion loop) are assigned mechanical type/test
cleanup only, not a visual change. Native DOM-only red cases must be rerun after
B3, and image capture waits for corrected backend source/binary binding.

### Independent live Product/Worker durability — scoped PASS

Final normal-umask (`0022`) run passed **12 real groups**, using source-bound app
code, actual `Product::discover` / `capture_inventory`, signed managed-runtime
resolution, unmodified supervisor and current shipping bundle. No manual evidence
rows were inserted. Node 24.21.0 / Playwright 1.64.0 / Chromium 156.0.8078.4,
sandbox enabled, owned loopback only.

An inspector observed committed DOM plus SQL Run artifact pin while PNG was
pending. Killing only the owned app child caused its real guardian to reap Node/
Chromium. A fresh process recovered Interrupted and resumed only PNG from changed
page B; original DOM bytes, full metadata, hash and refs were unchanged and readable
after another fresh-process read. Page B would fail a new DOM attempt (257 attrs),
so this is a positive proof of skipped settled DOM, not coincidentally equal data.

Other real cases passed: screenshot timeout/cancellation/worker crash retain DOM;
100001px DOM-only has no image artifacts or observed tool image/lazy/scroll writes
while screenshot twin fails ImageLimit; DOM limit retains PNG; shared 404 fails
both; hidden-DOM change with identical PNG and canvas-PNG change with identical
DOM persist independent app verdicts. Live staging was 0700 under normal umask;
B1/B2 work in this actual Product path. All owned processes/scratch/server cleaned.

Evidence/commands/exact sources/binaries:
`live-durability-verification/final-report.md` and
`run-gieih5gi/{report.json,binding.json,independent-final-assertions.json}`.
This closes actual app checkpoint/death/recovery/resume scope of T15, **not** public
network, saved-login replay, CLI/GUI/release or the later auth/B3 repair. Auth's
pre-repair source is explicitly bound in that evidence; later bytes are not silently
certified. Remaining targeted auth/reopen and ordinary client/native gates still run.

### B1/B2/B3 app regression gate passed; report matrix extended

App author ran the focused app/lib, saved-original, DOM review and original
screenshot-review targets under unchanged **0022**: **99 pass, zero fail, one
existing ignore** (library 79, screenshot review 9, DOM review 8, originals 3).
Staging-at-creation, kind-aware attempt/resume, auth wire and DOM-only/partial
reopen regressions are included. App all-target/all-feature Clippy passes.
Auth now uses frozen source metadata/saved page union and wire-3 envelopes, while
encrypted session v2 stays unchanged. E owns the final ordinary CLI rerun; GUI
will then rerun native DOM-only cases and the actual rendering matrix. A fresh
independent managed-browser auth/reopen probe is being prepared, without real
credentials, OS keychain or user profile access.

Report reviewer additionally verified **three actual CLI/engine exports**:
both successful (image unchanged/DOM changed), image ImageLimit + DOM changed,
and DOM SerializationFailed + image changed. Exact sources/dates/policies/notes/
hashes/fingerprints match; missing siblings remain explicit. Six HTTP/file loads,
20 plaintext views and ten downloads passed in sandboxed Chromium with no captured
execution/resource attempts/CSP errors. Actual 8 MiB metadata and 64 MiB DOM-copy
preflight refusals publish nothing, leave no stage and preserve engine health;
only about 175 KiB report output/2.1 MB seeded CAS were needed. Final-render HTML
64 MiB / Markdown 8 MiB refusal remains source/model-only, explicitly not executed.
Evidence: `engine-report-verification/matrix-01/{verification-result.json,scope.md}`.

### Final ordinary CLI gate — PASS

Under unchanged umask **0022**, full CLI suite passed **32 tests, no failures or
ignores**, including all 23 workflows, both byte-health/cancellation IPC cases
and normal-permission E1/E2 corruption/consent cases. Strict all-target/all-feature
CLI Clippy passes. No source edits/API changes/dependencies/new files in this gate;
E sources frozen. Existing T12 report exports/bindings remain hash-verified intact.

Commands/counts/logs: `e-cli-final-handoff.md`; binary/test/326-file source binding:
`e-cli-final-binding.json`. CLI SHA-256:
`041127702a1c424685cb3801162db141f1fa3d6e4ce3f849ba9c600cc218f5b3`.
GUI now owns final native slot (focused component/IPC tests, DOM-only reopening,
production build, strict Clippy and 32-frame offscreen matrix). Fresh independent
public-client verifier consumes a private copy of that frozen CLI; it does not
replace the native or real-worker tests with synthetic-client claims.

### B3 independent auth/reopen gate — scoped PASS

Fresh-process DOM-only and mixed-partial baselines reopen, preview exactly their
saved page union and complete mapped probe/check against frozen settings. No
omitted-page or future-draft visits. Real `AuthState::install` replays cookie,
localStorage and sessionStorage through wire 3; missing storage/ended sessions
return NeedsLogin without anonymous fallback. Normal umask 0022, no key-provider
calls or login window; all owned processes/server/scratch cleaned. Fresh app build
differs from the prior 12-case binding only in `auth.rs`.
Evidence/commands/exact hashes:
`live-durability-verification/auth-final/final-report.md`, raw `run-05wvwzem/`.
Headed login/confirmation, OS keyring and encrypted-login restart are not implied.

Fresh independent native reviewer `ses_eb421e937ffeRfnvRvQdW0EzAG` is checking
T13 source/input/consent guards without Cargo and will review the final 32-frame
matrix after author handoff. No new native captures or open-ended polish rounds
are authorized by that review; one batched finding list and one confirmation only.

### N1 native note-input defect — independent FIX, repair assigned

Fresh native reviewer found DOM-only **Review note** opens an editor that remains
disabled/unfocusable: `main.rs` mounts `AnalysisField(9)` under screenshot analysis
only, so its mounted-control gate rejects input before the DOM-specific branch.
The same applies with an unavailable image sibling. Existing DOM note tests used
direct `set_value`, hiding the actual user-input failure.

GUI owner is fixing DOM-aware mount/enabled/focus routing without weakening
modal/agent/history/dirty guards. Regression must activate the real note control,
assert mounted/enabled/focusable, **simulate typing**, save through real IPC and
reopen the persisted independent DOM note. Cover DOM-only and unavailable-shot
both-mode, plus modal/history rejection; direct content assignment is insufficient.
If the initial matrix already exists, its source binding remains pre-repair and
one final confirmation batch must include the note scene; no repeated polish loop.
N1 and native/visual acceptance remain open until fresh independent rescoring.

### Independent public CLI consumer — PASS (2026-10-18)

Fresh public-client exercise passed at signed synthetic-worker scope: three modes
and defaults, focused source/diff/review, kind-bound confirmations, same-check
sequential and atomic approvals, frozen baseline plans, v1/v2 ZIP imports and
negative cases. It recorded **224 CLI invocations**, with harness corrections
retained transparently. No product fixes/blockers. Owned sessions/engine/server
stopped; no Cargo/production edits or changed T12 artifacts. This is actual public
CLI/engine acceptance, not real-browser or native visual substitution.
Report/exact commands/source/binary bindings:
`engine-verification/public-consumer-01/FINAL-CLIENT.md`.

### Native final handoff — functional gates pass, visual confirmation blocked

N1 author source fix passes real activation/typing/IPC/fresh-reopen tests, including
DOM-only and corrupt-shot modes, dirty close/reopen, exact independent bindings
and modal/history/agent input guards. **47 focused tests pass** (18 component,
three adapters, nine workflows, three queue, fourteen shell), strict GUI Clippy
and ordinary production build pass. Independent N1 rescore is still running.

The finite visual cycle is **not accepted**: initial 32 frames predate N1 and a
short-window source-height repair; confirmation captured only five light 1280x800
scenes plus calibration before `ui_dom_capture.rs:155` found no
`dom-diff-source` bounds after actual recorded-History opening. A strengthened
test/harness-only retry also failed without production changes. Note-editor,
dark and 820x600 confirmation pixels were not reached. Never drop that assertion,
inject evidence or relabel the initial frames as final.

Source/binary/capture handoff:
`gui-final-20261017/confirmation/HANDOFF.md` (plus logs/manifests/bindings).
Production binary SHA-256:
`6d359b4c9bbe9f2ce546ae804765d677d584404f3146dfddfb2565ab655f8f30`.
Renderer-test binary SHA-256:
`0e809ac2122f12e0bc823df401ccc8b43354fef8a9cc6e1f46f8020c27ac801c`.
This is an unresolved render/harness failure, not an established backend failure.
Fresh independent reviewer is diagnosing it read-only and rescoring N1 against
the actual tests/pixels already retained. The planned correction/confirmation
allowance is exhausted: any additional targeted repair/render cycle requires a
new authorization, not another automatic polish loop. No whole-feature PASS.

While source is frozen, coordinator is running bounded final workspace format,
Clippy and ordinary tests. The prior 702.71s pre-feature synthetic 100/1000-page
qualification test is explicitly excluded from this final bounded sweep; ignored
live/native suites remain opt-in. Results/before-after source bindings will be
retained under `coordinator-final-gates/`; a partial or timed-out run is not PASS.
