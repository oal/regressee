# Investigation recipes

First read SKILL.md and commands.md. `r`, paths and authorized `URL` are as defined
there. Run each command deliberately; retain nonzero-result envelopes. These are
recipes with returned IDs, not authorization to accept changes or broaden scope.

## New project / baseline

```sh
r project create --name 'Authorized visual review' --url "$URL" --save project.json
PROJECT=$(jq -r '.project.id' project.json)
r browser status
# Only if setup is needed and downloads are authorized:
r browser setup
```

Review full project scope/profiles before discovery. For a bounded page limit,
edit the returned project DTO and save against its actual revision:

```sh
jq '.project | .discovery.page_limit = 5' project.json > settings.json
r project update --project "$PROJECT" --expected-revision "$(jq -r '.revision' project.json)" --file settings.json
r discovery start --project "$PROJECT" --save inventory.json
```

Read inventory pages, inclusion flags, scope, aliases, notes and warnings. It is
a bounded search, not whole-site coverage. Use `discovery exclude/include --file
inventory.json --pages "$PAGE" --save selected-inventory.json` only for the intended
UUIDs; capture that selected file instead. Do not automatically exclude failures.

```sh
r capture baseline --project "$PROJECT" --file inventory.json --detach > capture-job.json
JOB=$(jq -r '.data.job.id' capture-job.json)
r jobs wait --job "$JOB" > baseline-result.json
RUN=$(jq -r '.data.value.latest.run.id' baseline-result.json)
r results list --project "$PROJECT" --run "$RUN" --offset 0 --limit 50
r project show --project "$PROJECT" --save current-project.json
```

Inspect originals for representative/all relevant targets and every gap before
saving. A candidate is not active until explicitly confirmed. If the task
authorizes establishing this reviewed baseline:

```sh
r baseline save --run "$RUN" --expected-revision "$(jq -r '.revision' current-project.json)" --prepare --save baseline-ticket.json
# Read the ticket, especially actual successful/intended counts and settings.
r confirm --file baseline-ticket.json --confirmation "$(jq -r '.confirmation.id' baseline-ticket.json)"
```

`--partial` is separate deliberate authority to omit failed targets; never use it
to disguise incomplete coverage. Failed/paused/interrupted/preview runs are not
automatically baseline candidates. Baseline-bound settings changes require
explicit `--replacement --prepare`, followed by fresh capture/save, not reuse of
incompatible pixels.

## Existing project / exact evidence

```sh
r project list --page 0
r project show --project "$PROJECT"
r history list --project "$PROJECT" --page 0
r capture check --project "$PROJECT" > check.json
RUN=$(jq -r '.data.value.latest.run.id' check.json)
ANALYSIS=$(jq -r '.data.value.comparison.revision.id' check.json)
r results list --project "$PROJECT" --run "$RUN" --analysis "$ANALYSIS" --offset 0 --limit 50 > results.json
jq '[.data.value.rows[0].target.key]' results.json > selected-keys.json
PAGE=$(jq -r '.[0].page' selected-keys.json)
PROFILE=$(jq -r '.[0].profile' selected-keys.json)
r results artifacts --project "$PROJECT" --run "$RUN" --analysis "$ANALYSIS" --page "$PAGE" --profile "$PROFILE" --save evidence.json
```

Select the actual intended row, not blindly row zero. Repeat/paginate to cover the
task. If no analysis exists, omit `--analysis` and inspect capture states/gaps.
Artifacts expose original baseline/before-run IDs, hashed original before/after
and derived difference paths, dimensions/DPR, warnings, gates, regions and masks.
Open those images; keep hypotheses separate from visible changes.

Crop each relevant pane when needed (rectangle must fit its returned dimensions;
this 600×400 recipe is valid only for images at least that size):

```sh
r crop preview --project "$PROJECT" --run "$RUN" --analysis "$ANALYSIS" --page "$PAGE" --profile "$PROFILE" --pane after --rect 0,0,600,400 --space screenshot-pixels --destination "$CROP" --save crop-preview.json
r crop export --file crop-preview.json --prepare --save crop-ticket.json
r confirm --file crop-ticket.json --confirmation "$(jq -r '.confirmation.id' crop-ticket.json)"
```

`CROP` must be a new absolute PNG path outside data/cache with an existing physical
parent. Crop **before/after/difference** separately as needed; pane flag for diff
is `difference`. ≤4M output pixels; screenshot pixels only, not CSS coordinates.
Label crops as derived, retain source hash/rectangle and inspect originals too.

## Review, history, selected approval, report

```sh
r review history --analysis "$ANALYSIS" --page "$PAGE" --profile "$PROFILE" --limit 100
r review set --analysis "$ANALYSIS" --page "$PAGE" --profile "$PROFILE" --state needs-fix --note 'Observed change; cause not established'
```

Current result-row reviews are authoritative for that exact analysis/target;
recent run-wide history is bounded. Follow `next_before` for full review history.
Sensitivity reanalysis uses saved originals (`analysis reanalyze --run "$RUN"
--preset sensitive`); it creates a new analysis ID. Never mask a defect simply to
remove its verdict. `analysis select` changes the displayed-analysis pointer;
`analysis show` does not. After reanalysis, refresh exact approval/report previews.

Only if task authority covers accepting the inspected keys:

```sh
r baseline preview --analysis "$ANALYSIS" --file selected-keys.json --save approval.json
# Inspect selected keys, original baseline, run, analysis and retained history.
r baseline accept --file approval.json --prepare --save acceptance-ticket.json
r confirm --file acceptance-ticket.json --confirmation "$(jq -r '.confirmation.id' acceptance-ticket.json)"
```

Reports can precede acceptance; they preserve the check's **original** reference,
not today's active baseline. `REPORT` is a new absolute directory outside data/
cache with an existing physical parent. Inspect privacy inventory and full content:

```sh
r report preview --project "$PROJECT" --run "$RUN" --analysis "$ANALYSIS" --file selected-keys.json --destination "$REPORT" --save report-preview.json
r report export --file report-preview.json --prepare --save report-ticket.json
r confirm --file report-ticket.json --confirmation "$(jq -r '.confirmation.id' report-ticket.json)"
```

Include failed targets deliberately, not only changed screenshots. Selection is
1–100 unique keys; split larger reports (≤256 MiB copied images/8 MiB metadata).
Local static HTML + Markdown + unchanged PNG copies; no upload. A failed export
may leave a partial new directory: inspect it, use a new destination, never claim
success from its existence. ZIP baseline sharing is a separate previewed workflow,
does not export login sessions, and does not add report notes to its format.

## Optional desktop collaboration

```sh
r agent start --name 'Authorized visual investigation' --lease 90 > agent.json
SESSION=$(jq -r '.data.session' agent.json)
r --session "$SESSION" agent renew --lease 120
r --session "$SESSION" agent select --project "$PROJECT" --run "$RUN" --analysis "$ANALYSIS" --page "$PAGE" --profile "$PROFILE" --view split
r --session "$SESSION" agent handoff --message 'Please review this change in the desktop.'
r --session "$SESSION" agent end
```

After start, all mutations, preparations, confirmations and job controls need
`--session "$SESSION"`. Renew before expiry even during handoff. Ask the human
to open `regressee-desktop` with the same paths and explicitly Follow the named
session. Drafts must be deliberately saved/discarded. Never read the private GUI
bootstrap or imitate frontend authority. Headless handoff exits 4 **after pausing**;
it cannot create a human writer/Return control. End the session or wait for a
human-opted-in following GUI. Take control/expiry/end/restart invalidate new agent
authority; do not circumvent them with another session. Running jobs may continue.
