# Portable Regressee agent skill

The distribution ships `skills/regressee/SKILL.md` with three optional references.
It is tool-neutral: it uses the public CLI, structured receipts and the host's
ordinary image viewer, not an MCP integration, internal RPC or website code edits.
Read [CLI contracts](cli.md) before operating real projects.

## One-click setup for local agents

Open **App settings → Agent integrations**, then choose **Install skill** beside
Claude Code, Codex or GitHub Copilot. This installs instructions for your user
account across local projects, not the agent application itself. Setup is offline
and does not change host permissions, start an agent session or enable following.
Desktop packages must keep `regressee` beside `regressee-desktop`.

The same setup is available from the CLI, without starting the local engine:

```sh
regressee integrations status
regressee integrations install --host claude-code
regressee integrations install --host codex
regressee integrations install --host github-copilot
regressee integrations remove --host claude-code
```

Use `--json` for structured output. Supply `--data-dir` and `--cache-dir` when
using nondefault storage. The installed `connection.json` binds the absolute CLI
executable and those directories, so the agent uses the same data as the desktop
without relying on PATH or the current repository. These paths do not grant
permission to inspect unrelated projects. Re-run Install/choose **Update skill**
after moving the executables, changing storage locations or upgrading the skill.

| Host | Personal installation | Start using it |
| --- | --- | --- |
| Claude Code | `~/.claude/skills/regressee` | Ask it to use `/regressee`. |
| Codex | `~/.agents/skills/regressee` | Ask it to use `$regressee`. |
| GitHub Copilot | `~/.copilot/skills/regressee` | Ask a local agent session to use the `regressee` skill. |

`~` means HOME on Unix and USERPROFILE on Windows. These are the hosts' documented
default personal-skill locations; custom host search-directory overrides are not
rewritten. Codex's `.agents` location is shared: other compatible agents, including
Copilot, may discover it too. Installing in both locations may expose duplicate
skills depending on the host. Reload skills or restart the host if a new skill is
not listed. Cloud agents do not automatically access your desktop or these files.

**Installed** means the bundled files and local connection match this build. It
does not verify that a host has loaded the skill, can open local images, or permits
CLI execution. Test that in your chosen host before authorizing real website work.
No real-host end-to-end qualification is implied by file installation tests.

**Update skill** and **Remove skill** affect only unchanged managed installations.
A checksum record protects local edits; unknown files, linked directories and
unmanaged skills are kept, with a recovery explanation instead of a force option.
Move or preserve such a folder yourself before retrying. Removal does not revoke
permissions, stop sessions or remove instructions already loaded in a host; begin
a fresh host session when needed. Interrupted setup can leave a stage/backup or
lock folder beside the skill. Inspect it before removing it; never erase unrelated
host configuration. Cleanup failures name the retained folder rather than claiming
it was deleted.

### Live desktop updates

The CLI and GUI share one local engine when their data/cache paths match. Saved
changes and external discovery/capture/check activity refresh without an app
restart. Passive refresh keeps your selected project/run and unsaved drafts.
To follow the agent's navigation, ask it to start/renew a named session and use
`agent select` as it moves; explicitly choose **Follow agent** in the top banner.
**Take control** and human handoff keep their existing safeguards. Installing a
skill never enables following or grants baseline approval authority.

## Manual portable installation

1. Extract the CLI-only or desktop archive into a private/administrator-owned
   directory. Desktop archives contain **both** `regressee` and
   `regressee-desktop`; keep them together. Use an absolute CLI path or add that
   directory to PATH. Check `--version`, `--build-info` and `--json capabilities`.
2. If your agent host supports portable SKILL.md directories, consult that host's
   installation instructions and copy the **entire** `skills/regressee` directory
   into its configured local skill search directory. Keep `references/` relative
   to SKILL.md. Manual setup requires no symlink or host permission edit.
3. Alternatively, explicitly provide SKILL.md and its references to an agent as
   task instructions. Supply the executable, private data/cache paths, authorized
   site scope/budget and allowed consequences. Ensure it can open local PNGs.

The original packaging work did not qualify any particular agent host.
The frontmatter uses a **standard YAML 1.2 JSON mapping**: complete parsing and
name/description schema checks use Python's standard JSON parser in source/package
validation, with no production YAML dependency. This avoids ambiguous unquoted
colons/implicit types; a normal YAML skill loader can read the same mapping.
Copying files or an implementor smoke is not an independent real-agent exercise.
The [consumer exercise](cli-consumer-exercise.md) supports a separate agent using
only packaged binaries/public help/this skill and an owned loopback website.

The skill targets CLI schema 1/protocol 9. It teaches fixed-inventory checks,
actual before/after/difference and derived-crop inspection, warnings/coverage,
privacy previews, exact confirmations/retries and optional human-opt-in following.
It never grants permission to accept baselines simply to get a clean report,
open unrelated data, weaken a browser sandbox or upload evidence.

It also documents the three canonical check modes, bounded normalized DOM
source/diff pages, distinct per-kind analysis/review identities, explicit
single-/multi-kind baseline selections and inert local DOM report evidence.
DOM-only workflows require text inspection, not invented screenshot/crop IDs;
hidden text and attributes remain potentially sensitive despite exclusions.
