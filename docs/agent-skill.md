# Portable Regressee agent skill

The distribution ships `skills/regressee/SKILL.md` with three optional references.
It is tool-neutral: it uses the public CLI, structured receipts and the host's
ordinary image viewer, not an MCP integration, internal RPC or website code edits.
Read [CLI contracts](cli.md) before operating real projects.

## Manual installation

1. Extract the CLI-only or desktop archive into a private/administrator-owned
   directory. Desktop archives contain **both** `regressee` and
   `regressee-desktop`; keep them together. Use an absolute CLI path or add that
   directory to PATH. Check `--version`, `--build-info` and `--json capabilities`.
2. If your agent host supports portable SKILL.md directories, consult that host's
   installation instructions and copy the **entire** `skills/regressee` directory
   into its configured local skill search directory. Keep `references/` relative
   to SKILL.md. There is no installer, symlink requirement or automatic host edit.
3. Alternatively, explicitly provide SKILL.md and its references to an agent as
   task instructions. Supply the executable, private data/cache paths, authorized
   site scope/budget and allowed consequences. Ensure it can open local PNGs.

No particular agent-host integration has been verified by this packaging work.
The frontmatter uses a **standard YAML 1.2 JSON mapping**: complete parsing and
name/description schema checks use Python's standard JSON parser in source/package
validation, with no production YAML dependency. This avoids ambiguous unquoted
colons/implicit types; a normal YAML skill loader can read the same mapping.
Copying files or an implementor smoke is not an independent real-agent exercise.
The [consumer exercise](cli-consumer-exercise.md) supports a separate agent using
only packaged binaries/public help/this skill and an owned loopback website.

The skill targets CLI schema 1/protocol 7. It teaches fixed-inventory checks,
actual before/after/difference and derived-crop inspection, warnings/coverage,
privacy previews, exact confirmations/retries and optional human-opt-in following.
It never grants permission to accept baselines simply to get a clean report,
open unrelated data, weaken a browser sandbox or upload evidence.

It also documents the three canonical check modes, bounded normalized DOM
source/diff pages, distinct per-kind analysis/review identities, explicit
single-/multi-kind baseline selections and inert local DOM report evidence.
DOM-only workflows require text inspection, not invented screenshot/crop IDs;
hidden text and attributes remain potentially sensitive despite exclusions.
