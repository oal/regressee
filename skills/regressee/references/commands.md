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
| Projects/configuration | `project list/create/show/update`, `profiles/environments/scope show/set`, `environment select/save` |
| Discovery | `discovery start/status/inventory/existing/include/exclude` |
| Capture | `capture preview/baseline/check/resume` |
| Jobs | `jobs list/show/wait/pause/cancel/retry/ack` |
| Evidence | `results list/artifacts`, `crop preview/export` |
| Analysis/reviews | `analysis show/select/reanalyze`, `masks save`, `review set/history` |
| Baselines/history | `baseline save/preview/accept/restore`, `history list/show` |
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
  Outcome filter: changed/unchanged/failed/noncomparable. Capture-state filter:
  pending/captured/failed/skipped/cancelled. Project/history `--page` is zero-based
  (64 records, `more`). Review history `--limit` 1–200 / `--before` = `next_before`.

| Exit | Meaning |
| --- | --- |
| 0 | Success, admission or prepared ticket; inspect receipt state |
| 1 | Completed visual differences (`ok:true`), not a bug verdict |
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
