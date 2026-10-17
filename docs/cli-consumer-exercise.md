# Reproducible packaged-CLI consumer exercise

This fixture serves three owned public (no login) loopback pages. Captures use
**actual managed Chromium**, not the signed synthetic workers in Rust workflow
tests. It neither installs a runtime nor changes OS/browser sandbox policy.
Requires Python 3.11+, a native extracted CLI/desktop package and an existing
private physical scratch parent. This is a development-host exercise, **not clean
server, Windows or native desktop qualification**.

Fixture **version 2** keeps the same three initial page paths, but Details contains
over 6,000 CSS pixels of real document content and a separate below-fold change
near its lower marker. Earlier version-1 900px screenshots remain valid short-page
evidence, **not** long-page qualification. Verify actual returned screenshot height
and inspect top and lower before/after/difference crops within its physical bounds.

From a source distribution, use `python3 scripts/cli-consumer-fixture.py --help`.
Set `EXERCISE` to a new absolute scratch path, `PACKAGE` to the extracted directory:

```sh
python3 scripts/cli-consumer-fixture.py setup --root "$EXERCISE" --package-dir "$PACKAGE"
```

An optional `--runtime-cache` explicitly selects a previously verified **managed
`product-runtime` cache**, not a project/data directory. The helper copies only
`current` and its named generation; no user sessions/data or links are copied.
The packaged CLI's `browser status` must verify the copy before use. Otherwise
request authorization for explicit `browser setup` (official managed downloads).
Never substitute a system browser or a synthetic worker when live capture blocks.

Give a **different fresh agent** only generated `TASK.md`, `exercise.json`,
installed `skills/regressee/` and public CLI help. Do not give it implementation
source, internal review documents, SQLite or RPC tools. The task includes exact
paths, capture budget, authorized baseline/selected approval/report boundaries,
image inspection, prompt-injection defense and teardown. Record the agent's own
transcript/result and exact binary/skill hashes; setup does not claim that this
independent exercise has happened.

The fresh task also requires an incomplete check with explicit `--save`: exit 3
must retain truthful stdout and create the raw capture DTO, including failed
targets. Compare it to `data.value`, recover the same job through show/wait saves,
and do not rerun capture just to recover output. HTML and Markdown reports must
carry their own visible sharing/privacy warning and distinguish original before/
after screenshots from derived differences and crops. The frontmatter is a
standard YAML 1.2 JSON mapping, fully parsed/schema-checked without a production
YAML dependency; direct Markdown reading alone does not qualify skill-host loading.

At the agent's requests, the orchestrator switches only the owned website:

```sh
python3 scripts/cli-consumer-fixture.py set-mode --root "$EXERCISE" --mode changed
python3 scripts/cli-consumer-fixture.py set-mode --root "$EXERCISE" --mode failed
```

Changed mode alters Details and adds a new link; fixed checks must not discover
that link. Failed mode additionally drops `/fail` connections, generating a real
failed navigation rather than an empty screenshot. `visits.jsonl` records owned
requests; inspect it separately from product receipts. The website includes an
explicit malicious-instruction specimen as untrusted evidence, not task authority.

After the agent ends any lease, finishes/cancels only its jobs and closes login:

```sh
python3 scripts/cli-consumer-fixture.py teardown --root "$EXERCISE"
```

Teardown uses public `engine stop`, refuses active authority/work, and asks only
its loopback server to stop using a private fixture token. It does not signal
unrelated processes or delete evidence/runtime files. Verify the recorded server
and engine have exited and no owned Node/Chromium descendants remain. Retain
source/package hashes, public receipts, reports and actual inspection conclusions;
never include the private site token or agent credentials in shared reports.

Reports are static local HTML with relative PNG links, no scripts, forms, remote
resources or required server. Open `index.html` locally when an authorized browser
is available. A disconnected/permission-blocked preview is an environmental limit:
read/parse HTML and Markdown and view actual PNGs, but do not claim rendered layout
or bypass browser/desktop permissions. Independent read-only visual confirmation
can be performed separately when an authorized browser becomes available.
