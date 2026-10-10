# 0008 — Phase 3 URL identity, policies and resumable inventories

Date: 2026-10-09. Local Phase 3 only, under decisions 0005/0006 and the
independently re-verified Linux supervisor. No public-runtime qualification.

## Identity and discovery

Guided identity version 2 preserves the URL parser's encoded path, trailing slash,
meaningful query bytes, ordering and repeated keys. Known `utm_*`, `gclid`,
`fbclid`, `msclkid`, `_ga`, `mc_cid`, `mc_eid` keys are removed by default without
re-encoding other pairs. Ordinary anchors are dropped; `#/…` and `#!/…` routes
remain separate pages. An advanced explicit-fragment setting preserves others.
No query sorting or blanket query deletion is implemented. Historical single-page
identity version 1 remains readable with its exact saved address; it is not
retroactively normalized or stripped of tracking/fragment information.

Only HTTP(S), credential-free addresses are accepted. Sources explicitly approve
their exact origins; the native list confirmation records the actual approved
origin set, not a reusable approval for future websites. No subdomain expansion.
A section uses an exact path or slash-boundary prefix, never a text-prefix match.
Exclusion text, optional link depth, page cap, 10,000 candidates and a bounded
candidate queue constrain discovery. Download and action-like addresses are
excluded. These policies are explainable safeguards, **not read-only guarantees**.

The worker samples actual rendered anchors after DOM readiness, never clicks
controls or submits forms. Discovery redirects become candidates before visiting
their destination; Rust applies scope/robots/action policy first. Self-redirect
and pagination cycles terminate by identity deduplication. Requested redirect
aliases and their destinations remain visible. Captures/checks refuse an unexpected
path/query/origin redirect rather than following it and matching another page.

Robots policy uses Regressee-specific groups when present, otherwise `*`, longest
matching allow/disallow rules (allow wins ties), wildcard/end markers and bounded
crawl delay. Missing robots (404/410) means no rules. Unavailable robots stops that
origin's targets, visibly; redirects are not silently followed. An authorized
override and an action-link override require explicit user choices saved in the
project. Robots rules are not authorization. Hidden/delayed links and other UI
states are not an exhaustiveness promise.

## Evidence and scheduling

Migration 004 copies old run/baseline/analysis JSON bytes transactionally while
raising their bounded records to 64 MiB and adding immutable inventories. Legacy
single-capture fields remain readable; new runs carry keyed capture arrays and
frozen queue settings. Every included page × selected profile has an intended
target. Failed/skipped/cancelled/pending targets are not unchanged results.
Activation remains an explicit transaction and requires all intended captures.

The blocking actor owns the scheduler/SQLite work. Presets cap active operations
at 1/2/4; advanced concurrency is 1–8, with a separate per-origin ceiling. A
process-global eight-worker permit and one active context/page per worker bound
browser operations. Popups are closed. Spaced origins use one slot per batch:
this is deliberately conservative and can under-use the global ceiling. Robots
delay can slow any preset. Temporary errors retry at most three times with
bounded exponential backoff; a crashed/deadline worker is stopped before replacement.

Targets are published and CAS references pinned at safe boundaries. Pause starts
no new batch and retains unfinished targets. Cancel uses the existing supervisor's
bounded active-work stop and interrupts image processing at its safe checks.
Restart interrupts only dead-owner jobs. Resume verifies evidence, frozen project
revision/settings, matching runtime/rendering identity and explicit continuity
confirmation. Its compare-and-swap claim transfers ownership to the current PID;
another concurrent claim cannot overwrite it. Resume timestamps disclose mixed
capture times. Completed/cancelled runs cannot resume or be edited.

Checks use the saved baseline inventory/profile snapshot, **never discovery or
current setup addresses**. Worker 3.0.0 is an explicit new runtime identity;
historical worker-2 evidence is retained, not silently migrated or compared with
the new worker. No session persistence, environment mapping, masks, selective
approval, archive or cleanup workflow is introduced.

Available-space preflight runs before capture and each batch with an explicit
reserve for worker originals/CAS/derivatives. Low space stops an interrupted queue
without activation or deleting evidence. It is not a reservation against other
applications consuming disk, a complete storage manager, or cross-OS qualification.
