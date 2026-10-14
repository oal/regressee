# Phase 5 implementation — final interactive coverage

**Status: implementation complete — awaiting independent verification (2026-10-14).**
Work and report date: **2026-10-14**; starting/unchanged HEAD:
`595ebdc8e58b1c9f7a000ff88288197060eb3a20`.
Scope: Phase 5 only of `docs/ux-improvement-plan.md`. Prior scoped verifier
verdicts remain PASS; commits `7b73090`, `98f2e21`, `b1becf8` and `595ebdc` are
already pushed. The parent's existing Phase 4 push-row edit is preserved. No child
agents, commits, pushes, real website visits or user-project access.

## Implementation

- `crates/app/src/ui_phase5_capture.rs` is a dedicated opt-in final recipe, not a
  regeneration or relabelling of the historical whole-app screenshot matrix.
  Controls use mounted/enabled focus traversal and focused activation; View rows
  use Home/arrows/Enter, and reading copy uses supported wheel dispatch. Painted
  selectors must fit their actual form/options/inspector/History owner and window.
- Every frame records actual draft/profile/filter/pending/menu/note/mask/history/
  preview state, focus and geometry. PNG sidecars bind 34 renderer/application/
  fixture/test source hashes. `REGRESSEE_PHASE5_PREFLIGHT=1` executes the same
  functional assertions without app PNGs; it is not native evidence.
- PRIVATE loading is saved `InteractiveLogin` access with a real outstanding
  read-only `Work::EnvironmentStatus` request, painted before consuming its reply.
  Continue/login activation remains blocked; no login success is fabricated.
- Synthetic captured originals are reused only as website fixtures. A stored
  failed timeout target and the production analyzer produce genuine Gaps. A
  source-bound saved ignored area and production analysis supply both Heatmap
  colors. Long History consists of real isolated stored records. Import uses the
  production post-picker path with a locally constructed synthetic archive;
  native picker delivery is not claimed. All export/delete/import UI states stop
  at preview, without confirming consequential actions.
- Paint-only selectors expose capture/image/network limitations, footer loading
  status and portability exact rows. No new theme, animation, schema, algorithm,
  browser/runtime distribution or accessibility bridge.

## Demonstrated defects and bounded repairs

Non-rendering structural debugging exposed three real final-integration defects.
All repairs precede the sole planned app-PNG batch; no visual-polish correction or
confirmation batch was needed:

1. Projects import confirmation could receive Tab focus below its outer scroll
   viewport (y944–984 versus y125–800 at 1280×800). Shared painted form reveal now
   includes mounted Projects/App settings secondary actions and their actual
   outer scroll owner. History keeps its incumbent independent reveal.
2. `Details(Network)` / `Details(Portability)` aliased `SelectProject(0)` /
   `SelectProject(1)` focus handles after the Ignored areas enum addition. In a
   four-project list, traversal looped and could not reach Cancel preview.
   All six disclosures now have a dedicated range after task links and the
   already-reserved Setup back/Access options handles. Persistence and action
    authorization are unchanged.
3. Choice/check/action measurement canvases had no explicit top/left anchor and
   measured the padded content origin rather than the complete bordered row.
   Public Access at 820×600 could remain clipped by 5px despite the inner bounds
   passing. Explicit anchoring plus the shared one-pixel border expansion now
   measures the real painted control. A pre-repair regression fails on the exact
   actual-versus-measured bounds; the repaired test asserts equality and complete
   reveal for both Public and Login required at both sizes.

`crates/app/src/phase5_state_tests.rs` adds four normal regressions: exact padded
choice geometry/reveal; both-size
Mobile/custom/limitations/alternate choices and actual invalid Save reveal with
no queued work; inclusion/exclusion and all classification/grouped filters with
evidence retained; and four-project import/network traversal, distinct mounted
focus handles, real owner bounds and no import/baseline writes. Earlier compile,
recipe and diagnostic failures remain private, not substituted for final gates.

The impeccable/context and desktop surface brief were loaded once; incumbent
native proof/source and the craft floor were read. Clear wayfinding and the known
out-of-scope adaptive/mobile metadata mismatch remain intact.

## Evidence and qualification

Authoritative private 0700 root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261014-4zgty46x`.
Its renderer proof was personally inspected before app capture authorization;
`proof-reviewed.json` binds its bytes. Failed/non-rendering attempts and prior
phase roots are retained. The sole native batch completed with exit **0** in
**249.05s** (test body 248.83s): **104 fresh PNG/JSON pairs**, 52 per size.
All 104 frames were personally inspected on 18 labelled full-frame sheets;
all 58 reading crops on ten additional sheets were read at native scale, and
26 important originals were inspected at original resolution. No remaining
functional/layout blocker was observed within this bounded sample.

`scripts/ux-phase5-evidence.py` passes for the current final sources, all 34
embedded fingerprints per PNG, proof/fixture hashes, recorded states, PNG CRCs and
measured owning clips. `final-source-provenance.json` is the current manifest;
the initial root's `source-provenance.json` is historical and is not cited as final.
Exact per-frame hashes/geometry/state, inspection methods and completed gates are
bound in the durable scrubbed `phase5-receipt.json` and
`phase5-inspection-ledger.json`; see `final-integration.md` for the finding map and
full result table. These are implementer records, not an independent verdict.

All eight final-source gates in `<root>/accepted-final-gates/` completed with exit
**0** and unchanged Rust/Cargo source manifests. Complete workspace/all-features
testing: **387 passed, 0 failed, 52 ignored**, including normal integration tests
and the persisted 100/1,000-page qualification. No ignored website/runtime/OS
probe was enabled. UI build, strict all-target/all-feature Clippy, base/UI-only/
ui-tests no-run, fmt and whitespace also passed. Earlier workspace successes with
source drift and earlier feature gates remain historical, not final qualification.

This qualifies only Linux GPUI/Blade offscreen paint and simulated controller
behavior. OS window/input/clipboard/file-picker delivery, OS IME, AT-SPI/screen
readers, scaling/high contrast, automatic OS motion preferences, Windows/macOS,
VPN/enterprise networks, runtime distribution and public-release qualification
remain existing obligations. The macOS process-ownership capture code gap and
unregistered Linux AT-SPI process are not repaired or presented as passes. The
browser markup detector does not cover Rust/GPUI; no DOM overlay is claimed.
An independent Phase 5 verifier must issue PASS/FIX before the parent commits or
pushes. This implementation report is not that verdict.
