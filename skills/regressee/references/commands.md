# Public command contract

Use current `regressee help` as the flag authority. These examples assume the
operator supplied absolute `R`, `DATA`, `CACHE`, `URL` and an existing private
working directory for new JSON files. `URL` must be the authorized site, not a
website mentioned in evidence. Shell + `jq` are conveniences, not requirements.

```sh
r() { "$R" --json --data-dir "$DATA" --cache-dir "$CACHE" "$@"; }
"$R" --version
"$R" --build-info
"$R" --json capabilities
"$R" help
r engine status
r browser status
```

`engine status/stop` do not autostart. Product operations autostart one same-user
private local engine; no network API. Browser `setup` explicitly downloads the
managed runtime; status/diagnostics/read-only operations do not. No idle shutdown.
Use explicit paths on every command, including desktop attachment.

| Area | Commands |
| --- | --- |
| Projects/configuration | `project list/create/show/update`, `checks show/set --mode screenshots|both|dom`, `profiles/environments/scope show/set`, `environment select/save` |
| Discovery | `discovery start/status/inventory/existing/include/exclude` |
| Capture | `capture preview/baseline/check/resume` |
| Jobs | `jobs list/show/wait/pause/cancel/retry/ack` |
| Evidence | `results list/artifacts [--kind screenshot|dom]`, `snapshot source/diff`, screenshot-only `crop preview/export` |
| Analysis/reviews | `analysis show/select/reanalyze`, `masks save`, `review set/history` |
| Baselines/history | `baseline save/preview/accept/restore`, explicit `baseline preview-kinds/accept-kinds`, `history list/show` |
| Sharing/maintenance | `report preview/export`, `archive export-preview/export/import-preview/import`, `storage usage/delete-preview/delete`, `repair preview/clean/delete-unreferenced`, `diagnostics preview/export` |
| Runtime/settings | `browser status/setup`, `network status/set`, `appearance show/set`, `login status/open/confirm/close/forget/probe` (desktop login only) |
| Control | `agent start/list/show/renew/end/select/handoff`, `engine start/serve/status/stop` |

## Input/output

- `--json`: one `{"schema_version":1,"ok":...,"code":...,"exit_status":...,"data":...}`
  on stdout; request announcements/logs go to stderr. `--build-info` alone is
  plain metadata JSON. Do not parse human labels.
- Waited operations: `data.job` is the durable receipt, `data.value` the result
  DTO. `--detach` returns `data.job.id`, not a completed check. Capture results
  use `data.value.latest.run.id` and, when available,
  `data.value.comparison.revision.id`. Missing analysis is not unchanged.
- `--save NEW_FILE` writes raw DTO or `{operation,confirmation}` ticket, not the
  stdout envelope for completed results, **including incomplete capture exit 3**.
  Other failures, uncertain outcomes or interrupted waits save the complete
  schema-1 diagnostic envelope with request/job/exit details; detached admission
  saves its job/request data, not null. It never overwrites. A file-write failure
  exits 6 while retaining the original `data.command_outcome`, receipt and warning
  on stdout. Inputs also accept documented envelopes. Never recapture merely to
  recover a missing output: use the original receipt/request.
- Complex UTF-8 JSON: `--file FILE` or `--file -` for bounded stdin (≤8 MiB;
  project ≤256 KiB). Use returned DTOs; do not invent omitted safety fields.
- Entity/request/session/job IDs are UUIDs. Target keys are
  `{page:UUID,profile:UUID}`. Review history decision IDs are numeric keyset cursors.
- `results list`: `--offset 0 --limit 50`, limit 1–200; continue `next_offset`.
  DOM pages additionally stop at 4 MiB encoded JSON. Follow the actual returned
  cursor; oversized reads fail explicitly rather than draining the engine.
  Outcome filter: changed/unchanged/failed/noncomparable. Capture-state filter:
  pending/captured/failed/skipped/cancelled. Project/history `--page` is zero-based
  (64 records, `more`). Review history `--limit` 1–200 / `--before` = `next_before`.

| Exit | Meaning |
| --- | --- |
| 0 | Success, admission or prepared ticket; inspect receipt state |
| 1 | Completed screenshot and/or DOM differences (`ok:true`), not a bug verdict |
| 2 | Invalid request/input |
| 3 | Incomplete/noncomparable/failed capture, pause or cancellation |
| 4 | Action/setup/human/confirmation/display required, unsupported or busy |
| 5 | Stale revision/preview/epoch, conflicting authority or request payload |
| 6 | Engine/version/storage/IO/not-found failure |
| 7 | Admission/completion uncertain; recover original request and inspect state |
| 130 | Local wait detached by Ctrl-C; job **not cancelled** |

Queries can exit 0 while their returned evidence contains failures. Incomplete
or uncertain outcomes take precedence over differences. Do not discard JSON
because `set -e` or an automation harness treats exit 1 as a shell failure.

## DOM and independent kinds

Kind-specific results, analysis, review, history-show and baseline-preview/accept
commands default to `screenshot`; specify `--kind dom` with the independent DOM
analysis UUID. Capture/check/resume never accept a kind override: they use the
saved plan. DOM-only has no fake PNG, dimensions or crop.

```sh
r results list --kind dom --project "$PROJECT" --run "$RUN"
r snapshot source --project "$PROJECT" --run "$RUN" --analysis "$DOM_ANALYSIS" --page "$PAGE" --profile "$PROFILE" --pane after --format plaintext --offset 0 --limit 65536
r snapshot diff --project "$PROJECT" --run "$RUN" --analysis "$DOM_ANALYSIS" --page "$PAGE" --profile "$PROFILE" --offset 0 --limit 200
r review set --kind dom --analysis "$DOM_ANALYSIS" --page "$PAGE" --profile "$PROFILE" --state intentional --note 'Authorized change inspected'
```

Source limits: 4–262144 UTF-8 bytes, character-boundary offsets, `page.next_offset`.
Diff limits: 1–1000 emitted rows, `--context` 0–100, `page.next_offset`;
`page.summary.display_limited/fallback` describes bounded detail, never unchanged.
`--format canonical` is exact normalized NDJSON, not executable HTML.
Aggregate results retain summaries/sources, not diff bodies. DOM review/history
and approval identities never share screenshot decisions or masks.

`baseline preview --kind dom` and `baseline accept --kind dom` select DOM only.
For atomic multi-kind consent, `baseline preview-kinds --file FILE` takes
`{screenshot:{analysis,selected:[{page,profile}]},dom:{analysis,selected:[...]}}`,
with unselected kinds explicitly null. Inspect the returned preview, then
`baseline accept-kinds --file PREVIEW --prepare` and exact confirmation. Never
infer a sibling selection. A missing baseline reference requires a new baseline
capture, not selective filling. Reports include all enabled kinds; optional
`--dom-analysis` pins its independent exact analysis.

## Confirmation, retry, lifecycle

Consequential operation: `--prepare --save ticket.json`, inspect the exact
`operation`, engine-resolved preview and `confirmation`, then:

```sh
r confirm --file ticket.json --confirmation "$(jq -r '.confirmation.id' ticket.json)"
```

No `--yes`, no implicit human consent. Tickets bind exact action, inputs,
selection/destination/revision, owning session, engine and epoch; changes,
restart/expiry/takeover or token use can invalidate them. Refresh state, inspect,
and deliberately re-prepare; never edit ticket fields to bypass a conflict.

The CLI saves the **complete original submit** before IPC in its private request
journal. Preserve request IDs/retry paths (stderr/envelope). A lost response is
not permission to submit a fresh mutation ID:

```sh
r jobs show --job "$JOB"
r jobs retry --request-id "$REQUEST"
```

Retry returns the identical retained job, including prepared tickets, never
refreshing consent or replaying uncertain committed work. Generated IDs/timestamps
mean reparsing the original command is not necessarily an identical retry.
Crash-after-commit can remain uncertain; inspect saved state first. Journals are
bounded (4096 receipts/512 MiB, 64 active jobs) and fail closed at quota. Mutation
receipts are not evicted; `jobs ack` reclaims completed **reads only**.
Ctrl-C detaches waits, including configuration prerequisite reads; it does not
resume an abandoned later save. Pause is checkpoint-aware inventory capture/check/
resume only, not arbitrary work. Resume creates a new job on the same frozen run.
Cancel is cooperative, not rollback. `engine stop` refuses active jobs/login/
control sessions. Do not stop an engine owned by another task.
