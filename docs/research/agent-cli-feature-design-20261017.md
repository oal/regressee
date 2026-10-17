# Agent companion control — scoped post-build documentation

Date: 2026-10-17. Documenter is the Phase 5 implementor, **not the Phase 4 UI
author**. The documenter role was run inline without delegation. This is an
existing-world native desktop **Operate** extension, not a redesign or global
token extraction. No UI source, DESIGN.md, PRODUCT.md, sidecar or brief was edited.

## Checked evidence

- Read Impeccable `reference/document.md` and
  `reference/degraded/documenter.md`; ran context once for `agent_ux.rs`.
- Compared incumbent PRODUCT.md / DESIGN.md / `.impeccable/design.json`, the
  pre-extraction surface brief, current `crates/gui/src/ui/theme.rs` and shared
  control metrics against `agent_ux.rs` / `main.rs` banner/feedback ownership.
- Read the final independent Phase 4 **PASS**, protocol 6, including its final
  20-frame source-bound offscreen batch. Inspected its complete
  `agent-cli-phase4-review/feedback-final/contact.png` without recapture or polish.
  The independent report retains exact final source/PNG hashes and tests. This
  documentation does not promote those offscreen results into native input,
  accessibility, clean-server, Windows/macOS or release qualification.

## Feature contract grounded in the build

One persistent neutral top banner names the agent/control state/remaining lease.
The existing primary teal **Take control** stays pinned, with ordinary secondary
**Follow agent** / **Return to agent** and feedback dismissal when applicable.
The human chooses Follow; passive refresh does not silently move the viewer or
discard drafts. Following disables ordinary native interaction; handoff restores
full temporary human operation while the agent pauses. Expiry/disconnect/end
retains a nonlocking explanation and explicit Dismiss. No animation, imagery,
new palette or evidence tint was introduced.

The 40px name/status row and 22px independent details reader bound untrusted long
requests/errors/activity. In Results, retained business feedback shares that
reader rather than stacking another evidence-height panel. Action ownership and
error semantic ink remain explicit. The final independent minimum-size fixture
retains 75px of original evidence with an 87px banner, including genuine saved
feedback; this is a **tested constraint of that 820×600 fixture**, not a new global
layout token or a recommendation to shrink evidence to 75px on future surfaces.
Complete messages remain reachable with wheel/keyboard End/Home. Existing guards,
original image identities, review/baseline consent and history remain unchanged.

## Documenter output

**No global changes.** Source/system files checked are named above; this scoped
record is the only design-documentation path written.

1. Palette: Unified neutrals, teal/mint action and selection, existing error/warning ink.
2. Type: incumbent system UI; 15/22 body, 14/20 controls and 13/18 help roles.
3. Geometry: existing 40px controls and small native corners; bounded reading owner.
4. Named rule: make the next safe action obvious, with explicit control ownership.
5. Named rule: preserve untinted originals, drafts, exact provenance and deliberate approval.

**Drift not canonized/repaired:** sidecar and surface-brief targets still name
`crates/app/src/ui` / `main.rs` rather than extracted `crates/gui` paths; the brief
retains historical blue-world prose with later neutral supersession, and PRODUCT's
evidence section still calls the CLI a skeleton. These pre-existing records are
not rewritten by a narrow feature documenter. Five inherited GUI failures and
global formatting/lint debt remain separate, not new design-system rules.
