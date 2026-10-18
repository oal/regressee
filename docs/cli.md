# Regressee terminal guide (CLI schema 1 / engine protocol 7)

Build `cargo build --locked -p regressee-cli --bin regressee`, then use
`./target/debug/regressee` (or put the installed binary on your PATH). No GPUI,
display, desktop process or keychain is required for public-site operations.
Regressee is a public beta; [platform limits](support-matrix.md) still apply.
There is no upload command. Named agent leases, headless semantic selection and
GUI-opt-in exclusive following/handoff are available. CLI-only and combined
desktop [package paths](release-building.md) include the [portable skill](agent-skill.md).
Native GUI input, clean-server and release qualification remain separate gates;
an isolated development-host browser run is not a clean-server qualification.

## Start here

```sh
regressee --version
regressee --json capabilities
regressee help
regressee help capture
regressee --build-info
```

Help currently prints the complete command reference, also for `help AREA` and
`AREA ACTION --help`. These commands do not discover/create data directories,
start an engine, download anything or initialize graphics. Unknown commands,
unknown/inapplicable flags, missing values and malformed IDs fail without startup.
There are no prompts in either output mode, and **no `--yes` bypass**.
Local domain validation (including project names/URLs, bounded configuration
files, policy/mask shapes and pagination bounds) also fails with `invalid_request`
/2 before data discovery/autostart. These errors have an explicit validation
category, not a guess based on words in a human message. Checks requiring saved
entities, revisions, artifact geometry or runtime state remain engine operations.

Global flags may appear before or after the command:

| Flag | Contract |
| --- | --- |
| `--json` | One schema-versioned JSON document on stdout, including parser/failure responses. Logs/request announcements only on stderr. |
| `--data-dir DIR` | Public storage selection, shared with the desktop engine. Without flags both use the same OS user directories. With this flag alone, cache is `DIR/cache`. |
| `--cache-dir DIR` | Explicit cache location; same engine data root has one owner. |
| `--detach` | Submit operation and return its durable job. Admission is **not** a completed check. Later use `jobs wait`. |
| `--request-id UUID` | Choose a submission ID (or recover the generated one from output/stderr). Do not reuse it for different input. |
| `--prepare` | Prepare a consequential operation, without performing it; returns exact confirmation ticket. Incompatible with `--detach`. |
| `--save FILE` | Save to a **new** file regardless of the command's exit. Successful/incomplete capture DTOs and confirmation tickets stay raw; other failures, uncertainty or interrupted waits save the complete schema-1 diagnostic envelope. Detached admission saves its job/request data, never null. Does not overwrite. |
| `--session UUID` | Authenticate mutations/control as the named agent, using its private local credential file. Not a name or a GUI-lock command. |

Examples use absolute, physical, private data/cache directories, not an actual
user store. The engine enforces private directory/IPC ownership. Keep any
configuration, ticket and report files private too. `REGRESSEE_DEV_DATA` is **not**
the public CLI interface. No CLI command opens SQLite or performs recovery;
the single shared engine does that under its ownership lock. See
[local-engine.md](local-engine.md) for IPC, shutdown and journal limits.
The desktop also accepts `regressee-desktop --data-dir DIR --cache-dir DIR` to
attach to the same isolated engine; these public flags override developer hooks.
An active data-root owner has one canonical cache directory. A mismatched cache
selection is a conflict, not silently ignored; use its configured cache or stop
that engine deliberately before changing the cache path.

## New-project workflow

The following are executable shell examples. Substitute a website you own or are
authorized to capture; creating or visiting a project does not establish consent
to crawl an unrelated site. `jq` is used only for selecting returned fields.
Use a fresh output directory; `--save` will not replace existing files.

```sh
R=./target/debug/regressee
DATA=/absolute/private/data
CACHE=/absolute/private/cache
r() { "$R" --json --data-dir "$DATA" --cache-dir "$CACHE" "$@"; }

r project create --name 'Release website' --url https://your-authorized-site.example/ --save project.json
PROJECT=$(jq -r '.project.id' project.json)
r browser status                    # read-only; never installs
r browser setup                     # explicit, detached is supported

# Optional: edit full project settings (profiles/environments/scope) before capture.
jq '.project | .discovery.page_limit = 20' project.json > settings.json
r project update --project "$PROJECT" --expected-revision 1 --file settings.json --save updated.json

r discovery start --project "$PROJECT" --save inventory.json
# Inspect coverage, notes, aliases and pages. Exclude/include by stable page ID:
PAGE_TO_SKIP=$(jq -r '.inventory.pages[-1].id' inventory.json)
# Only if you actually intend to exclude this page:
# r discovery exclude --file inventory.json --pages "$PAGE_TO_SKIP" --save selected-inventory.json

r capture baseline --project "$PROJECT" --file inventory.json --detach > capture-job.json
JOB=$(jq -r '.data.job.id' capture-job.json)
r jobs wait --job "$JOB" > baseline-result.json
RUN=$(jq -r '.data.value.latest.run.id' baseline-result.json)
# Inspect the actual captures/warnings before approving; no baseline is saved yet.
r baseline save --run "$RUN" --expected-revision 2 --prepare --save save-ticket.json
cat save-ticket.json
CONFIRMATION=$(jq -r '.confirmation.id' save-ticket.json)
r confirm --file save-ticket.json --confirmation "$CONFIRMATION" --save saved-project.json

# After the authorized site changes, this checks ONLY saved baseline targets:
r capture check --project "$PROJECT" --detach > check-job.json
JOB=$(jq -r '.data.job.id' check-job.json)
r jobs wait --job "$JOB" > check-result.json
# Exit 1 means observed visual differences, 3 means incomplete coverage; neither
# means "confirmed bug". Do not use `set -e` to discard their JSON receipts.
```

Discovery is a bounded search, **not proof of whole-site coverage**. Review the
returned scope, warnings and included flags. `capture baseline` requires that
reviewable inventory; `capture preview` captures representative discovery images
without activating anything. A later `discovery existing` copies the most recent
recorded inventory into a fresh preview without visiting the website. Settings
and project revision must still match when capture starts.
`discovery inventory --job UUID` reopens a discovery job's exact preview (including
a detached discovery); `--project UUID` instead uses the recorded-inventory path.

`baseline save --partial --prepare` is the separate explicit intent to omit failed
targets from a finished baseline candidate. The engine ticket includes the actual
run, successful/intended counts, settings revision and partial flag. A paused,
interrupted or representative-preview run cannot become a baseline. Changing
baseline-bound source/profile/environment definitions requires
`project update --replacement --prepare`, or the equivalent named settings
command, then confirming its ticket. This retains historical evidence; separately
capture and save a new baseline. It does not silently activate replacement pixels.

## What to check

New and legacy projects default to screenshots only. Select the canonical capture
plan with `project create --mode screenshots|both|dom`, or update saved settings:

```sh
r checks show --project "$PROJECT"
r checks set --project "$PROJECT" --expected-revision REVISION --mode both
```

`checks show` returns the full project record, including `project.capture_plan`.
Changing the mode of a project with a baseline requires `--replacement --prepare`
and exact confirmation, then a separate new baseline capture/save. Existing checks
continue using the baseline's frozen mode and policy, not today's draft. Capture,
check and resume do not accept a kind override. Screen sizes still affect DOM-only
captures; image sensitivity and ignored rectangles do not affect DOM comparisons.
Snapshots can contain private hidden text, ordinary attributes and URLs despite
the versioned script/event/form exclusions. Treat them as sensitive evidence.

### DOM results, originals and review

Kind-specific operations default to `--kind screenshot`. For DOM use
`results list/artifacts`, `analysis show/select/reanalyze`, `review set/history`,
`history show` and `baseline preview/accept` with **`--kind dom`**. A DOM-only run
does not manufacture image IDs, dimensions or crop panes. Crops and ignored
rectangles reject DOM. `analysis reanalyze --kind dom --run ID [--context N]`
uses retained originals, never screenshot sensitivity, masks or a website revisit.
DOM result pages have both a 200-row ceiling and a 4 MiB encoded-JSON budget;
escape-heavy warnings/notes can produce fewer rows. Always follow `next_offset`,
not `offset + requested limit`. An individually oversized row/coverage block
returns `response_too_large`; use focused target artifacts/source reads. Read
responses above 16 MiB fail boundedly without draining the engine.

```sh
DOM_ANALYSIS=$(jq -r '.data.value.dom_comparison.revision.id' check.json)
r results list --kind dom --project "$PROJECT" --run "$RUN" --analysis "$DOM_ANALYSIS"
r results artifacts --kind dom --project "$PROJECT" --run "$RUN" --analysis "$DOM_ANALYSIS" --page "$PAGE" --profile "$PROFILE"
r snapshot source --project "$PROJECT" --run "$RUN" --analysis "$DOM_ANALYSIS" --page "$PAGE" --profile "$PROFILE" --pane after --format plaintext --offset 0 --limit 65536
r snapshot diff --project "$PROJECT" --run "$RUN" --analysis "$DOM_ANALYSIS" --page "$PAGE" --profile "$PROFILE" --context 3 --offset 0 --limit 200
r review set --kind dom --analysis "$DOM_ANALYSIS" --page "$PAGE" --profile "$PROFILE" --state intentional --note 'Reviewed this DOM change'
r baseline preview --kind dom --analysis "$DOM_ANALYSIS" --file selected-keys.json --save dom-approval.json
r baseline accept --kind dom --file dom-approval.json --prepare --save dom-ticket.json
```

Source output is `{selection,key,pane,format,source,page}`. `source` binds exact
run/target/artifact/policy/material/rendering/stability identities; `page` contains
`text,offset,total_bytes,next_offset`. Offsets are **UTF-8 bytes**, must be valid
character boundaries, and limits are 4–262144 bytes. Use returned `next_offset`
without arithmetic guesses and pin the returned run/analysis on subsequent pages.
`--format canonical` returns exact normalized NDJSON;
`plaintext` is the readable deterministic record representation. Neither is HTML.
`--save` for source/diff bodies requires `.txt`, `.json`, `.jsonl` or `.ndjson`;
HTML/SVG/Markdown filenames are rejected. Recovering a focused read through a job
receipt retains this safeguard. Saved CLI values are JSON DTOs, not active previews.

Diff output includes both exact sources, comparison outcome, options and `page`
containing `summary,hunks,offset,total_rows,next_offset`. Limits are 1–1000 emitted rows; offsets index
the bounded diff, not source lines. Hunk headers/counts describe the full bounded
diff; each hunk page supplies `row_offset,total_rows` and shared core rows for the
requested window. `--context` is 0–100. Core work/output limits
can return coarse/limited detail without changing the exact verdict. Complete
originals remain available through source pages. Noncomparable sources produce an
incomplete result, never an unchanged diff. Aggregate capture/workspace/results
responses retain outcomes and sources, **not hunk text**; focused results/artifacts
include text-free `diff_summary` counts/limits.

DOM reviews have distinct immutable analysis IDs and exact Before/After source
bindings. Screenshot reanalysis/masks do not review DOM or clear its notes.
`review history --kind dom` uses the same bounded keyset pagination as screenshots.

### Explicit approval of multiple kinds

`baseline preview-kinds --file selection.json` accepts this strict shape (use
`null` for an unselected kind, never an empty list meaning “all”):

```json
{"screenshot":{"analysis":"SCREENSHOT_ANALYSIS_UUID","selected":[{"page":"PAGE_UUID","profile":"PROFILE_UUID"}]},"dom":{"analysis":"DOM_ANALYSIS_UUID","selected":[{"page":"PAGE_UUID","profile":"PROFILE_UUID"}]}}
```

Inspect its `KindApprovalPreview`, then `baseline accept-kinds --file preview.json
--prepare`, inspect the ticket and confirm its exact UUID. The commit is atomic;
an invalid selected kind rejects everything. An unselected failing sibling is
neither approved nor silently replaced. Fresh previews support sequential sibling
approvals from the same check only while selected Before sources still exactly
match a compatible descendant baseline. Old confirmations never rebase themselves.
Filling a baseline coverage omission requires a new baseline capture.

## Existing-project investigation and selected acceptance

```sh
r project list --page 0
r project show --project "$PROJECT" --save current-project.json
r history list --project "$PROJECT" --page 0
r capture check --project "$PROJECT" > check.json    # may exit 1 or 3
RUN=$(jq -r '.data.value.latest.run.id' check.json)
ANALYSIS=$(jq -r '.data.value.comparison.revision.id' check.json)
r results list --project "$PROJECT" --run "$RUN" --analysis "$ANALYSIS" --outcome changed --offset 0 --limit 50 > results.json
# Inspect further pages with next_offset. Select actual intended target keys:
jq '[.data.value.rows[0].target.key]' results.json > selected-keys.json
PAGE=$(jq -r '.[0].page' selected-keys.json)
PROFILE=$(jq -r '.[0].profile' selected-keys.json)
r results artifacts --project "$PROJECT" --run "$RUN" --analysis "$ANALYSIS" --page "$PAGE" --profile "$PROFILE" --save evidence.json

r review set --analysis "$ANALYSIS" --page "$PAGE" --profile "$PROFILE" --state intentional --note 'Reviewed against the authorized change'
# Prepare the local report BEFORE sharing any pixels/URLs/notes:
r report preview --project "$PROJECT" --run "$RUN" --analysis "$ANALYSIS" --file selected-keys.json --destination /absolute/new/report-dir --save report-preview.json
cat report-preview.json
r report export --file report-preview.json --prepare --save report-ticket.json
r confirm --file report-ticket.json --confirmation "$(jq -r '.confirmation.id' report-ticket.json)"

# Only if the task authorizes accepting THESE changes:
r baseline preview --analysis "$ANALYSIS" --file selected-keys.json --save approval.json
cat approval.json
r baseline accept --file approval.json --prepare --save accept-ticket.json
r confirm --file accept-ticket.json --confirmation "$(jq -r '.confirmation.id' accept-ticket.json)"
```

Never accept a baseline merely to turn a check green. Approval binds the exact
selected page/profile keys, original baseline, run, analysis, displayed-analysis
revision and project revision. New analysis, masks, baseline/settings changes or
changed selection can make it stale. Refresh the preview and deliberately prepare
again; do not remove revision fields. `analysis show` is read-only;
`analysis select` deliberately changes the displayed-analysis pointer used by
approval. `history show --project ID --run ID` retains the check's original baseline
and saved analysis, not today's baseline. For interrupted checks without analysis,
use `results list/artifacts --run ID`; no unchanged verdict is invented.

## Evidence, sensitivity and ignored regions

`results list` returns resolved project/run/analysis IDs, original baseline ID,
inventory/coverage/run state, total filtered rows and `next_offset`. Filters:
`--page UUID`, `--profile UUID`, `--outcome changed|unchanged|failed|noncomparable`
(analysis classification), `--capture-state pending|captured|failed|skipped|cancelled`
(actual capture outcome, also useful when no analysis exists). Limit is 1–200;
offset ≤30,000. Pagination is a live snapshot; pin `--run` and `--analysis` for
repeatable evidence, and restart pagination if the underlying selection changes.
Without `--run`, the engine resolves a recorded run first and reads that exact
run's evidence. An explicit `--analysis` can select its older recorded run; an
explicit run/analysis mismatch fails. For a check, before images, baseline,
analysis and differences always use its original baseline, including after
acceptance or restoration of another active version. A baseline-candidate or
representative-preview run has no invented comparison reference.
Project/history pages are zero-based fixed service pages (64 items, `more`);
page index is capped at 15,625. Job pages use offset/limit 1–200.

`results artifacts` returns exact content-addressed original paths for **before,
after and difference**, their hashes/bytes/kinds/dimensions, target outcome,
original `baseline` and `before_run` identities,
capture/profile settings, scale, saved analysis report (including comparability
gates, warnings, bounded region hints and ignored rectangles), and explicit gaps.
No image is substituted for a failure. `difference` is a derived analysis image;
before/after originals are unmodified. `screenshot_pixels_per_css_pixel` is the
recorded DPR; difference scale is null because it is in aligned screenshot pixels.
Neither capture cautions nor region hints establish exhaustive correctness.

```sh
r crop preview --project "$PROJECT" --run "$RUN" --analysis "$ANALYSIS" \
  --page "$PAGE" --profile "$PROFILE" --pane after --rect 0,100,600,400 \
  --space screenshot-pixels --destination /absolute/new/crop.png --save crop-preview.json
r crop export --file crop-preview.json --prepare --save crop-ticket.json
r confirm --file crop-ticket.json --confirmation "$(jq -r '.confirmation.id' crop-ticket.json)"

r analysis reanalyze --run "$RUN" --preset sensitive --save analysis.json
# Or --file policy.json for the complete Policy DTO.
```

Crop rectangles **must** be unsigned `X,Y,W,H` in physical screenshot pixels,
inside the selected original, at most 4M output pixels. CSS coordinates are
rejected, not guessed/rounded. Streaming PNG validation/decoding is bounded to
4096 × 32768 and 64M source pixels, ≤128 MiB encoded source, one bounded verified
encoded buffer frozen to a private temporary file, row-wise decoding plus
≤16 MiB crop pixels (no full-page RGBA allocation). Original hashes and
complete PNG streams are verified. Crop/report outputs must be new absolute
paths with existing canonical non-symlink parents, outside data/cache. They
never overwrite originals or existing files. Crop PNG metadata labels derived
evidence and output includes source/rectangle/coordinate space.

To save ignored rectangles, use `evidence.json.analysis_report.request` as the
`AnalysisRequest` input for `masks save --project ID --file request.json`.
Set `mask` to `{revision,target,before_capture_id,after_capture_id,
before_dimensions,after_dimensions,rectangles}` using the exact before/after
descriptor fields in that request. Allocate a new mask revision; rectangles use
screenshot pixels (≤1000, bounded by source geometry). To clear a mask save a new
binding with an empty rectangles array. Then `analysis reanalyze` creates a new
analysis identity using the saved masks; originals/reviews/history are not erased.
Custom policies use immutable revision IDs and `Policy.validate` limits, not
undocumented numeric CLI tolerances. Presets are uncalibrated seeds, not automatic
bug judgments. Review states are `unreviewed`, `intentional`, `needs-fix`;
`--note-file FILE` accepts UTF-8 text, ≤8192 bytes; it is exclusive with `--note`.

Result rows' `reviews` contain **zero or one authoritative current decision for
that exact analysis/target**, not the most recent decisions elsewhere in a run.
Unrelated review activity cannot hide a current note. Reports include those same
current decisions and explicitly disclose that they do not contain full history.

```sh
r review history --analysis "$ANALYSIS" --limit 100 > review-page.json
# Optional exact target filter: --page "$PAGE" --profile "$PROFILE"
# If next_before is non-null, continue with --before THAT_DECISION_ID.
```

History entries are `{id,decision}` with stable append-only numeric decision IDs;
pages return `analysis`, `run`, optional `key`, `total`, `items`, `next_before`.
Limit is 1–200. Older-page keyset cursors are not shifted by newly appended reviews;
`total` reflects the current count. `history show`/workspace evidence retain a
bounded 256-entry recent **run-wide** window, with `review_history_truncated` when
more exist. `review set` preserves its recent-array reply and adds
`data.review_history` semantics/limit/possible-truncation metadata. Use the paginated
command, not the recent window, for complete selected-analysis history.

## Local sharing and privacy

Report preview freezes exact selection, resolved identities, storage revision,
content fingerprint, destination and privacy/content inventory. Confirming a
different path, target selection or analysis requires a new ticket. Exports
revalidate the content and revision at execution, after engine consent checks.
Reports select 1–100 unique target keys, ≤256 MiB total original copies (DOM
canonical plus readable files ≤64 MiB) and ≤8 MiB preview metadata.
They contain static `index.html`, concise `summary.md` and selected unchanged PNG
copies: original before/after screenshots and derived differences. The HTML includes the saved `CaptureRun`, its available `queue`
(`discovery`, capture `project_revision`, `resumed_at`), and separate
`reference_settings` for each selected before-image source run (queue, profiles,
environment and rendering settings). Analysis policy/masks/gates, original baseline
identity, coverage/excluded inventory and selected current reviews are included.
These fields are previewed and fingerprinted, never filled from today's project
configuration. Missing legacy queue provenance is null and explicitly described
as **unknown**, not complete. Failed captures
are explicit missing images. Original image bytes are unchanged.

**Reports may expose private screenshots, URL query strings, selected notes,
excluded URLs, recorded scope/exclusions/robots authorization/pacing/retries,
project revisions/resume timestamps, baseline-source settings/provenance and local
original paths/usernames.** Masks
are analysis exclusions, not redaction. Inspect the full preview before confirming.
Both HTML and Markdown visibly retain **Before sharing / LOCAL ONLY**, the full
privacy inventory and the explicit ignored-areas-are-not-redaction warning. Before
and After links identify **original screenshots**; Difference links identify
**derived analysis**, even though their saved PNG bytes are copied unchanged.
Separately exported crops are always derived (including crops of a difference),
with pane-specific source/rectangle/coordinate metadata; they are not implicitly
included in reports. Regenerate older report/crop previews after this repair;
their exact privacy payload changed, not the retained original images/reports.
No cookies, credential/session files, network configuration or private logs are
included. Page/URL/note data is HTML-escaped, Markdown text is inside escaped
preformatted blocks, and HTML has a restrictive CSP, no scripts, forms, remote
resources or executable page markup. No upload occurs. An output I/O failure can
leave a partial export directory; it is an explicit failed job, never a report
success. Inspect it before choosing a **new** destination.

Reports include every enabled kind, independent DOM outcomes/notes and bounded
escaped unified diffs. DOM-only reports contain no missing screenshot panes.
`report preview --dom-analysis UUID` pins an older exact DOM analysis separately
from `--analysis SCREENSHOT_UUID`. The preview freezes both identities, original
source runs/artifacts/policies/byte counts, diff options, privacy and destination.
DOM files use generated `.txt` names: original canonical JSON records and derived
readable plaintext, never captured executable `.html`. Captured URLs are inert
escaped text, not links. Report DOM diffs are bounded to 500 rows/16 KiB per target;
HTML is capped at 64 MiB and Markdown at 8 MiB, with explicit display-limit labels.

Baseline ZIP sharing is the **unchanged existing secret-exclusion contract**:

```sh
r archive export-preview --project "$PROJECT" --baseline BASELINE_UUID --save zip-preview.json
r archive export --file zip-preview.json --destination /absolute/new/baseline.zip --prepare --save zip-ticket.json
r confirm --file zip-ticket.json --confirmation "$(jq -r '.confirmation.id' zip-ticket.json)"
r archive import-preview --source /absolute/baseline.zip --save import-preview.json
r archive import --file import-preview.json --source /absolute/baseline.zip --prepare --save import-ticket.json
r confirm --file import-ticket.json --confirmation "$(jq -r '.confirmation.id' import-ticket.json)"
```

ZIPs contain requested original images/normalized DOM evidence and page addresses,
not recipient login sessions. Reports
are a separate surface and do not add notes/sessions to the baseline ZIP format.
See [portable-baseline-archive.md](portable-baseline-archive.md).

## Durable jobs, recovery and confirmations

`jobs list`, `jobs show --job UUID`, `jobs wait --job UUID`,
`jobs pause --job UUID`, `jobs cancel --job UUID`, `capture resume --project ID
--run ID` are distinct operations. Capture/check/resume use checkpoint-aware
inventory jobs. Pause requests stop at a safe target boundary; in-flight work may
finish first. Pause applies only to these inventory jobs (not discovery/setup,
representative preview or arbitrary reads). Resume is a **new job** on the same
run, with original frozen configuration/reference. Cancellation is cooperative,
can preserve completed captures, and cannot rewrite an already-committed success.
Cancelling a queued job prevents its dispatch. Repeating controls on a finished
job is harmless. Controls acknowledge the request, not a terminal outcome; wait
or show the receipt. There is no claim of instant rollback.

Ctrl-C during wait exits **130**, returns `wait_detached`, and sends **no cancel**.
The job/engine continue. Terminal/GUI exit also detaches. Explicit `engine stop`
refuses live jobs/login. Foreground `engine serve` Ctrl-C drains rather than
silently cancelling. No idle timer exists. Browser setup/capture are explicit
operations; status, help, diagnostics and evidence reads do not silently install.

This includes configuration prerequisites: `profiles/environments/scope set` and
`environment select` load saved settings using separately journaled read jobs.
Ctrl-C while either `Load` or `EnvironmentStatus` waits returns promptly with that
read's ID/retry path, `phase:configuration_prerequisite` and
`continuation_submitted:false`. The read and other live work are not cancelled,
and observing interruption prevents the later save from being submitted. Waiting
or retrying that read **does not resume the abandoned configuration command**.
Reissue the named configuration command deliberately if still desired. `--detach`
applies to its final save, not to resolving prerequisite settings. Startup/just-
before-submit interruption can return 130 without a job if none was admitted.

Before IPC submission the CLI atomically persists and fsyncs the complete original
`Submit` in private `DATA/cli-requests/UUID.json`, then announces its ID/path on
stderr. Success envelopes include `request_id`, `retry_file` and `job.id`. A lost
submit/wait response returns `admission_uncertain` with structured recovery ID/path.
`jobs retry --request-id UUID` reads **that original payload**, including original
engine/epoch, operation, revision and confirmation; it does not reconstruct it
using today's settings. `jobs show --job UUID` inspects any retained engine receipt.
An identical retry returns the existing job even after restart; changed input
under an ID conflicts. For CLI-generated project/review IDs or timestamps prefer
`jobs retry` to re-parsing the original command, which may generate different data.
If a process dies before printing anything, use your chosen `--request-id`,
`jobs list`, or inspect the private request filenames. Do **not** create a new ID
merely because the response disconnected.

Preparing is recoverable through **all** receipt routes: `jobs show`, `jobs wait`
and `jobs retry` return the same `data.ticket`, and `--save` saves the same
`{operation,confirmation}` as the original prepare. For example:

```sh
r jobs retry --request-id "$PREPARE_REQUEST" --save recovered-ticket.json
r confirm --file recovered-ticket.json --confirmation "$(jq -r '.confirmation.id' recovered-ticket.json)"
```

Recovery never re-prepares, refreshes owner/epoch/revision, consumes a token or
replays a mutation. A consumed, expired or restarted-owner ticket remains the
original invalid ticket; deliberately obtain fresh consent rather than treating
recovery as a bypass.

A crash after product commit but before a durable terminal receipt is
`completion_uncertain`/Interrupted: inspect saved state. No automatic replay or
exactly-once claim. An original submit that was never admitted cannot acquire a
new owner's epoch silently; a deliberate new request after inspection is required.
Mutation/confirmation receipts are retained, not evicted. The engine caps 4096
receipts/512 MiB and 64 queued/running jobs; admission fails closed at quota. The
CLI request journal has corresponding bounds. Completed synchronous read receipts
are reclaimed (`read_receipt_reclaimed:true`, `retry_file:null`); `jobs ack` reclaims completed detached **reads only**, never mutation
or confirmation authority. There is no automatic destructive journal cleanup.

All consequential commands use `--prepare` → inspect ticket → `confirm --file
TICKET --confirmation EXACT_UUID`. Ticket carries exact typed operation and engine
confirmation (fingerprint, storage revision, engine UUID, epoch, confirmation ID;
baseline save/restore/replacement also include engine-resolved preview details).
It is bound to the whole operation, including existing domain preview, selection
and destination. It expires on storage changes, restart, epoch change or use.
Wrong ID/action/payload/revision fails. A confirmation expresses deliberate intent;
it is not proof of human approval or an authorization boundary against same-user
programs. A universal bypass is intentionally absent.

## Output and input schemas / exit contract

`--json` stdout is always exactly one JSON document for normal CLI handling:

```json
{"schema_version":1,"ok":true,"code":"job_admitted","exit_status":0,
 "data":{"job":{"id":"UUID","state":"Queued","submitted":{}},
         "request_id":"UUID","retry_file":"/private/data/cli-requests/UUID.json",
         "value":null}}
```

For a waited operation, `data.value` is the typed result DTO (without wire enum
wrappers); `data.job` retains the complete durable submission/outcome/progress,
frozen settings and capture status. A prepared operation additionally returns
`data.ticket`. Job show/wait return the same job/value shape; jobs list returns
`data.items,total,next_offset`. Engine status/start return `data.engine` and its
snapshot. Direct broker control commands return an acknowledgement; preferences
show returns its value. Parser/pre-submit errors have `data.message`; uncertain
transport failures also have structured recovery ID/path. Human mode prints a
readable label/job identity and pretty result or error (stderr on failure).
`--build-info` without `--json` retains plain data-free JSON build metadata.

Explicit `--save` is not success-only: an incomplete capture (exit 3) saves its
returned capture DTO, including actual run state, failures/missing targets and
analysis, identically to stdout `data.value`. The same applies when recovering
that receipt with jobs show/wait/retry; no new capture is required. Other failed,
uncertain (7) or interrupted (130) operations save the **complete schema-1
envelope**, retaining exit/code, job diagnostics, request/retry identity and any
continuation-not-submitted state rather than an ambiguous scalar/null. Parser or
pre-submit failures save diagnostics when a valid `--save` path was parsed.
If file creation/write itself fails, stdout retains the original receipt/data,
`data.command_outcome` preserves its original code/exit, `output_warning` explains
the file failure, and the CLI returns `output_write_failed`/6. Human output also
shows that warning. Existing files are never replaced, and no mutation is retried
or cancelled to repair an output file.
An already-existing/invalid save destination is rejected **before submission**
with `invalid_request`/2 and an explicit explanation; no operation is started and
no diagnostic file can replace that destination. Exit 6 describes a later write
failure after the initially valid destination was checked.

| Exit | Stable code families / meaning |
| --- | --- |
| 0 | `ok`, `job_admitted`, `confirmation_prepared`: command succeeded/admitted/prepared, **not** necessarily a completed check. |
| 1 | `visual_differences` (screenshot-only) / `check_differences` (DOM-enabled): completed comparable check has a change in either requested kind; operational success (`ok:true`), not confirmed defects. |
| 2 | `invalid_request`, `response_too_large`: parser/input/schema/limit/path/domain failure, or an explicitly bounded read response. Use smaller pages/focused reads. |
| 3 | `incomplete_capture`, `capture_paused`, `cancelled`: failed/missing/noncomparable coverage, absent check analysis, explicit pause or cancellation. Not unchanged. |
| 4 | `action_required`, `confirmation_required`, `replacement_required`, `display_required`, `unsupported`, `busy`: structured product requirement, setup/human/runtime/consent requirement, or bounded admission refusal. |
| 5 | `conflict`: stale revision/preview/authority, changed payload under request ID, consumed confirmation. Reload and deliberately re-prepare as appropriate. |
| 6 | `operation_failed`, `storage_write_failed`, `not_found`, `not_found_or_corrupt`, `version_mismatch`, `output_write_failed`: engine/storage/IO failure; no successful check verdict. Read `data.job.outcome`/message. |
| 7 | `admission_uncertain`, `completion_uncertain`: lost transport/durable completion uncertainty; recover original request, inspect saved state before any new mutation. |
| 130 | `wait_detached`: interrupted local wait; job was not cancelled. |

`ok` is true only for exits 0/1. Operational, capture and uncertainty codes take
precedence over differences. `jobs show/wait` propagate the job outcome's exit
meaning; queries (`results list`, history, analysis inspection) succeed as reads
even when the returned evidence contains failures/differences. Consumer code
should select explicit fields, preserve unknown added fields and check
`schema_version`/capabilities. Protocol 7 peers are required; old engines must be
stopped/rebuilt, not bypassed with a second store owner.
For an upgrade from protocol 2/3/4/5/6, stop the old binary's owner, then explicitly start
the rebuilt `engine serve` once to publish the new descriptor under the ownership
lock. See [the upgrade/lifetime contract](local-engine.md); do not delete locks.
Old report previews should be regenerated for the repaired identity/settings/review
contract; retained old receipts are not retroactive certification of their content.

Complex inputs are UTF-8 JSON `--file PATH` or `--file -` (bounded stdin), ≤8 MiB,
ordinary files only, domain bounds also apply (project ≤256 KiB). `--save` stores
raw DTO/ticket; inputs also accept the documented CLI envelope. IDs are UUIDs,
never labels. Schemas are the actual serde DTOs:

| Input | Rust schema |
| --- | --- |
| Project create/update | `core::domain::Project`; create can generate a valid draft from `--name/--url`; update needs exact ID and expected revision. `ProjectRecord {project,revision}` from show is accepted. |
| Profiles/environments/scope set | `Vec<CaptureProfile>`, `Vec<Environment>`, `DiscoverySettings`. Profiles contain extent/image mode/locale/timezone/DPR/mobile/stabilization/runtime. |
| Environment save | `{project: Project, revision: u64, environment: UUID, settings: LocalLoginSettings}`; `--replacement` selects combined explicit replacement/login intent. Session references cannot be imported from JSON. |
| Environment select | Project/environment IDs + expected revision; optional `--login-settings FILE` is `LocalLoginSettings`, else preserves current settings. |
| Inventory | `core::crawl::DiscoveryPreview` from start/existing/include/exclude, with revision/settings/inventory/notes/aliases/visited. |
| Selected keys | Array of `{page: UUID, profile: UUID}`; never interpreted as all targets. |
| Analysis / masks | `core::analysis_engine::Policy` / `AnalysisRequest`. |
| Baseline acceptance | `storage::ApprovalPreview` including its `ApprovalContext`. |
| Deletion / repair | `storage::DeletionPreview` / `RepairPreview` unchanged from engine preview. |
| ZIP / diagnostics | `storage::ExportPreview`, `ImportPreview` / `app::diagnostics::Report`. |
| Crop/report exports | `app::terminal::CropPreview` / `ReportPreview`, unchanged from preview. |
| Network | `runtime::network::NetworkSettings`; explicit network configuration only, never copied to reports. |
| Confirm | `{operation: protocol::Operation, confirmation: protocol::Confirmation}` emitted by the engine/CLI prepare path, with exact `--confirmation UUID`. Not a generic RPC authoring interface. |

## Full product action coverage

All following paths call the shared application engine; none is a CLI Store
adapter. This table distinguishes product operations from GUI-only presentation.

| Existing product action | Actual CLI command path |
| --- | --- |
| Project list/create/load/save | `project list/create/show/update` |
| Baseline-bound settings replacement | `project update --replacement --prepare` → `confirm` |
| Environments/profile CRUD, scope/capture settings | `environments/profiles/scope show/set`; full `project update`; capture settings are profile fields, not a second shadow schema |
| Atomic selected environment + login settings, combined replacement | `environment select`, `environment save [--replacement --prepare]` |
| Discover / inspect warnings / existing list / include/exclude | `discovery start/status/inventory/existing/include/exclude`; status uses `--job` |
| Representative discovery screenshots | `capture preview` |
| Baseline candidate / fixed-list check / resume | `capture baseline/check/resume` |
| Task progress / pause / cancel / detach | `jobs list/show/wait/pause/cancel`, `--detach` |
| Reconnect / original request retry / read receipt reclaim | `jobs retry/ack` |
| Current/historical results, coverage/filtering/pagination | `results list`, `history show`, explicit run/analysis filters |
| Before/after/difference originals, regions, scale | `results artifacts`, `crop preview/export` → `confirm` |
| Analysis sensitivity / saved analysis / display selection | `analysis reanalyze/show/select` |
| Ignored rectangles / clear mask | `masks save`, empty rectangles for new clear revision, then `analysis reanalyze` |
| Review state / notes / history | `review set`, authoritative current `results list` rows, `review history` keyset pagination; `history show` discloses its bounded recent window |
| Save/activate full or explicitly partial baseline | `baseline save [--partial] --prepare` → `confirm` |
| Selected approval preview / accept / restore version | `baseline preview/accept/restore`, prepare/confirm for commits |
| Run/baseline history / storage summary / usage | `history list/show`, `storage usage` |
| Reference-aware deletion preview/commit | `storage delete-preview/delete`, prepare/confirm commit |
| Repair inspection / temporaries / unreferenced deletion | `repair preview/clean/delete-unreferenced`, prepare/confirm mutations; missing/corrupt evidence is reported, not replaced |
| Baseline ZIP preview/import/export | `archive export-preview/export/import-preview/import`, prepare/confirm commits |
| Local review report (new) | `report preview/export` → `confirm`; never uploads |
| Diagnostics preview/export | `diagnostics preview/export` → `confirm`; sanitized support format unchanged |
| Runtime/browser readiness/setup | `browser status/setup`; setup explicit, detached/waitable |
| Network settings | `network status/set` |
| Managed desktop login status/open/confirm/close/forget/probe | `login status/open/confirm/close/forget/probe`; forget prepared/confirmed |
| Relevant appearance setting | `appearance show/set --value system/light/dark` |
| Engine lifecycle | `engine start/serve/status/stop` |
| Native file dialogs/clipboard/OS link opening | Explicit paths and JSON output replace dialogs; no simulated native input or automatic external link launch |
| Native viewer pan/zoom/split/overlay/tiles | Exact originals/scale/regions/crops replace presentation gestures; optional `agent select` mirrors semantic identities/view modes, not simulated native input |
| Unsaved GUI drafts / native focus / exit guards | Remain GUI-owned; revision conflicts preserve saved-data safety; CLI does not edit drafts |
| Agent sessions/selection/exclusivity/lease/handoff | `agent start/list/show/renew/end/select/handoff`, global `--session`; following, immediate takeover and return are GUI-only local choices. |

## Agent control and optional GUI following

For personal skill installation (without starting the engine), use
`integrations status [--host claude-code|codex|github-copilot]`,
`integrations install --host HOST` or `integrations remove --host HOST`.
Install also updates an unchanged managed skill. These commands accept `--json`
and storage path flags, but reject session/submission flags. They do not modify
permissions or start a controller. See [agent setup](agent-skill.md) for installation
paths, conflict recovery, host invocation and the meaning of verified-file status.

```sh
r agent start --name 'Release investigation' --lease 90 > agent.json
SESSION=$(jq -r '.data.session' agent.json)
r agent show --id "$SESSION"
r agent list
r --session "$SESSION" agent renew --lease 120
r --session "$SESSION" agent select --project "$PROJECT" --run "$RUN" \
  --analysis "$ANALYSIS" --page "$PAGE" --profile "$PROFILE" --view split
# All ordinary product mutations/confirmations/job controls use the same session:
# r --session "$SESSION" capture check --project "$PROJECT" --detach
r --session "$SESSION" agent handoff --message 'Please confirm the website login.'
# Human works in the GUI, saves drafts, then chooses Return to agent.
r --session "$SESSION" agent end
```

`agent start` reserves **one exclusive controller per canonical data directory**,
including headlessly. Other clients may inspect saved data but may not write,
prepare consequential confirmations, change preferences, pause/cancel jobs or
stop/revoke the engine through ordinary control requests. The GUI remains locally
navigable unless its human explicitly chooses **Follow agent** in the top banner;
outside following, saved data/progress still refresh and local drafts are kept.
Passive refresh retains the current project, run, saved analysis, result and
History page/selected IDs; it never substitutes a newer run. If a selected
History row is removed or moves off the page, no other row is selected for you.
Result drafts (including incomplete policy edits and IME composition) defer
replacement. Use an explicit load/refresh or task Continue to choose newer work.
Take control before performing human saved-data mutations outside a handoff.
There is no public CLI `follow`, `return`, `takeover` or universal bypass command.

The lease is 10–300 seconds (default 90), renewable before expiry, including during
human handoff. Each CLI process normally exits; that does **not** end the logical
session. A stopped agent eventually expires rather than locking a GUI forever.
An ended/expired/revoked session cannot regain control by renewal: deliberately
start another. Session history is in-memory, capped at 128 entries per engine
incarnation; restarting clears control sessions and invalidates their credentials,
not saved evidence or durable job receipts. Explicit engine stop refuses a live
control session; end it first. A lost start response can leave a bounded lease:
inspect `agent list`, take control in the GUI or wait for expiry; do not infer
that an unknown start failed. No idle-shutdown policy is added.

Selection is headless application state, separate from the stored displayed-
analysis pointer and from GUI navigation. Select a project alone to show settings,
or supply an **exact run** plus optional saved analysis and page/profile pair.
The engine verifies project/run/original-baseline/analysis/target membership before
publishing the selection. Omitted analysis resolves once to that run's saved
analysis, not a later run. Views: compare, before, after, difference, overlay, split.
`agent select --kind dom` selects the independent DOM analysis namespace and
supports compare/before/after/difference; image overlay/split are rejected.
No pixels, baseline, review or saved display pointer change merely by selection.
The durable selection receipt contains semantic IDs only. Full viewer evidence is
a separate reclaimable read, so browsing results does not duplicate a whole run in
the retained mutation journal on every selection.

### Desktop control UX (scoped Operate extension)

The persistent top banner uses the existing system type, unified neutral surfaces,
teal actions and 40px native controls, without new assets, animation or evidence
tints. It names the agent, current status and remaining lease.
The compact header keeps Take control/Return pinned; long names are truncated,
and full handoff/error/activity details scroll independently in a bounded reading
line (Tab to details, then Up/Down/Page Up/Page Down/Home/End, or use the wheel).
Even a maximum 1,024-byte request cannot grow the banner into the evidence viewport.
**Follow agent**
is an explicit GUI opt-in for that named session. It declines entry while project,
review-note, ignored-area (including an unfinished drawing), selected baseline
update, network, builder or discovered-page drafts exist, or an
incompatible task/modal/save is pending. Nothing is silently discarded.
Incomplete analysis-policy text and History-search IME composition also block
Follow and Return to agent. While following, normal pointer, menu, keyboard and
focused-field/IME actions cannot
navigate or edit; exact engine-validated semantic identities drive the viewer.

**Take control** is always available in the banner and with
**Ctrl/Cmd+Shift+T**. It immediately revokes the agent's new/queued authority,
without waiting for its approval. Already-dispatched atomic operations may finish
and keep their actual result; already-running discovery/capture jobs continue with
their frozen inputs. No implicit cancel, rollback or baseline acceptance occurs.

`agent handoff` pauses agent navigation/mutations and gives the following GUI full
temporary human control. The banner says **Your turn — agent paused**, includes
the request and offers **Return to agent**. Save/discard drafts deliberately and
finish incompatible dialogs/tasks before return; return refuses otherwise.
The agent cannot resume itself. With no following GUI, handoff still pauses the
agent and returns **`action_required` / exit 4** with structured
`data.human_input` (including `agent_paused:true` and the absent following
frontend). The handoff has paused the session; it is not an uncommitted failure
to retry blindly. **There is no human frontend authorized to write or return**: open the
desktop against the same data/cache paths, explicitly Follow that named session,
or end the session. This never launches a GUI automatically. Capabilities declare
GUI-only opt-in and full temporary human handoff; session snapshots expose the
following frontend (null when absent).

Window close and Quit retain their normal draft/save/discard guards and remain
escapable while following. GUI disconnect/heartbeat loss (15 seconds), agent
expiry or engine restart fail safe and unlock following; they never cancel jobs.
A lost engine connection also unlocks the local GUI immediately when detected.
Reconnecting/restarting never silently opts the window back into following.
Expiry, disconnect, session end and takeover retain a compact, nonlocking status
with **Dismiss**. A new controller replaces that explanation; dismissal does not
renew/restart a lease or cancel/undo work. Start a new session deliberately to resume.
Real external job activity is observed separately from a GUI's own busy request
or task, so unrelated reads are not misrepresented as capture progress.

Agent secrets are local/private in `DATA/agent-sessions/UUID.json` (0600 on Unix),
not stdout, jobs, reports, logs or confirmation tickets. Do not copy these files
into reports or archives. Credentials coordinate same-user clients, not a sandbox
against programs that already have that user's filesystem/process permissions.
GUI-role establishment additionally requires the private engine-minted bootstrap
capability in `DATA/engine-v1/frontend-capability.json`. The engine rotates it for
each incarnation and removes it on clean shutdown. Only the GUI's explicit local
bootstrap path reads that file; ordinary CLI/status/read/renew calls never do.
Raw anonymous `RegisterFrontend`, Follow, Takeover, Return, Detach and Heartbeat
requests cannot acquire a GUI role. The bootstrap is exchanged for a separately
registered, in-memory frontend credential with its own 15-second heartbeat lease.
Registration and lease renewal do not opt into Follow, revoke a controller,
advance a control epoch or change saved data. Restart invalidates both secrets.
A same-user program that explicitly reads the private bootstrap file can imitate
that GUI capability; this is **not** executable attestation or a strong OS sandbox.
GUI close/Detach retires its own registered credential; only its own following
session is revoked. Closing an unrelated GUI does not revoke another controller.
Confirmation authority additionally binds its owning session/frontend. Stale
epochs/leases, a different owner and cross-session tokens fail with `conflict`/5.
Identical admitted request retries return only the original receipt, never replay
old writes. After restart, inspect receipts without the now-invalid `--session`;
do not refresh old confirmation authority or replay unknown operations with a new
request ID. In the current incarnation, `jobs ack` must use the read receipt's
original session/frontend. Completed read-only receipts orphaned by an engine
restart may be acknowledged without the obsolete credential; mutation and
confirmation receipts remain protected and retained.

## Desktop versus headless

Linux no-display `login open` explicitly returns `display_required` **before**
browser setup/download or credential initialization. Headless session import,
scripted login, remote GUI and browser-sandbox bypasses are unsupported. On a
desktop, use the managed headed login browser, then `login confirm`; close it
explicitly before stopping the engine. Secure-session storage remains the existing
OS-protected service; no plaintext/session JSON import is added.

Public-site capture still needs supported managed Node/Chromium components, OS
libraries/fonts, network access and the default sandbox. macOS capture remains
blocked; Windows/Linux native/browser qualification is not inferred from CLI
builds. Tests in `crates/cli/tests/workflows.rs` use real subprocess CLI/IPC/product
services with a signed **protocol-3 synthetic Python worker** producing independent
PNG and core-golden normalized DOM channels and synthetic link responses, not
actual Chromium or external websites. Python-only fixture tests cover all modes,
channel failures, exact goldens/metadata and pixel-independent handshake; this is
not browser qualification. They do not
qualify browser rendering, desktop login, a clean server or published packages.
