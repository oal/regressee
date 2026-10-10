# 0006 — controlled parallel delegation

Date: 2026-10-09

The user subsequently granted permission to use multiple subagents when useful.
This supersedes the strict one-at-a-time restriction in decision 0005, not its
qualification honesty, product scope, or local-continuation policy.

- Continue to use GPT 6.1 Sol High for delegated work.
- A feature's independent verifier remains separate from its implementation agent.
- Dependent work waits for material local safety/correctness repairs to be checked.
- Parallel implementation is appropriate only for independent units with explicit,
  non-overlapping file ownership and agreed interfaces.
- Read-only research/verification can overlap unrelated implementation, but tests
  sharing runtimes, processes, or fixtures must be serialized or isolated.
- Shared architecture, integration, and status edits remain coordinated; no agent
  may overwrite another's unreviewed work or spawn uncontrolled nested agents.
- External/platform checks remain pending unless actually executed successfully.

The orchestrator chooses when parallelism is useful; permission does not require
starting agents for work that has unresolved dependencies.
