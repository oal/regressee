Method: dual-agent, sequential per user request (A: `ses_ec97d7bf7ffe04uN1LFh9vOS36` · B: `ses_ec9685592ffewOeGHDSojwukwN`).

# Whole-app UX review synthesis — 2026-10-13

## Outcome

Preserve Clear wayfinding and improve task hierarchy, not the visual identity.
238 fresh native screenshots establish the current layouts at 1280×800 and
820×600. Independent design review scored 26/40 (Acceptable); the native technical
assessment scored a provisional 13/20. These are expert judgments, not measured
usability or release qualification. The actionable findings are assigned to the
self-contained execution contract in `docs/ux-improvement-plan.md`.

## What's working

- Consistent readable native controls, selected marks and restrained visual tokens.
- Deliberate save/login/discovery/baseline/ZIP/deletion boundaries and draft guards.
- Untinted original evidence, truthful missing-capture states and retained history.

## Priorities and resolution ownership

| Priority | Finding | Concrete correction | Plan phase |
|---|---|---|---|
| P1 | A1: disconnected wizard/advanced Access overload | Connected numbered progression, stable current state, contextual consent, optional environments | 1 |
| P2 | A5: persistent preview branding | Remove slogan, About Regressee; preserve factual caveats | 1 |
| P2 | B-02: contradictory browser-install promise | Runtime-aware setup explanation | 1 |
| P1 | A2: hiding Results reduces evidence height | Stable compact identity/toolbar and trigger-anchored View menu | 2 |
| P2 | B-03: narrow disabled-action reasons omitted | Highest-priority actionable reason at both widths | 2 |
| P2 | B-04: Heatmap legend absent in production | Significant/ignored derived-pixel legend with real ignored fixture | 2 |
| P2 | Selection reads `0 eligible` misleadingly | Label eligible members of the actual selection | 2 |
| P1 | B-01: incompatible note limits/silent rejection | Shared byte limit, exact save/exit validation and rejection feedback | 3 |
| P2 | A3: single-line review notes | Wrapping multiline native editor with retained drafts | 3 |
| P2 | A4: masking precedes sensitivity | Common comparison choices first, ignored-area disclosure, advanced controls later | 4 |
| P2 | History/Settings repetition and sparse-list geometry | Compact task hierarchy without losing search scope/consequences | 4 |
| P3 | Settings/classification terminology | App settings, clear below-threshold classification wording | 4 |
| Evidence | Misleading/missing toggle/reveal capture coverage | Assert actual transitions/clip bounds, final targeted batch and complete tests | 5 |

Design and technical assessments agree that safe native behaviors should be
preserved. B establishes an additional concrete correctness defect in note limits:
4,096-byte input, 8,192-byte storage and character-based exit/save policies conflict.
A's proposed multiline editor alone would not resolve it. Both belong in Phase 3.

## Nielsen heuristic table (independent A)

| Heuristic | Score /4 | Principal issue |
|---|---:|---|
| Visibility of system status | 3 | Current-stage validity and selection wording |
| Match with real-world language | 2 | Optional technical setup in the basic path |
| User control and freedom | 3 | Good draft-safe exits, OS qualification pending |
| Consistency and standards | 2 | Wizard, View menu and note-input conventions |
| Error prevention | 3 | Strong guards, contradictory note limits |
| Recognition rather than recall | 3 | Locked progression and compressed context |
| Flexibility and efficiency | 3 | Useful customization, poor compact evidence height |
| Aesthetic/minimalist design | 2 | Competing Access/Analysis options |
| Error recovery | 3 | Strong recovery, compact explanations missing |
| Help and documentation | 2 | Useful but scattered contextual guidance |
| **Total** | **26/40** | **Acceptable; focused improvement needed** |

## Cognitive and persona risks

Basic Access and Analysis demand simultaneous decisions that should be staged.
First-timers must infer prerequisites from `Unavailable`; power reviewers lose
image height when hiding UI and cannot read notes at once; keyboard/low-vision
operators need reachable disclosures, visible blocking reasons and preserved
context. The plan reduces extraneous decisions without removing expert options.

## Evidence integrity and limitations

Both isolated agents inspected native source and the current-source screenshots.
B verified every PNG/metadata hash/dimension and all 13 baseline source fingerprints.
No user projects were changed, real websites visited or production code edited in
the assessment. Full inspection coverage/limitations are retained in the A ledger
and B receipt. The Rust-unsupported markup detector returned `[]`: this is neither
a native accessibility pass nor evidence that no issues exist. No DOM overlay was
attempted for the native app.

Real OS input/IME, AT-SPI/screen readers, system scaling/high contrast, automatic OS
motion preferences, Windows/macOS and runtime/public-release qualification remain
existing obligations. They are not disguised as newly implemented UX scope.

Questions skipped: the user has already selected wizard/product-language fixes,
authorized all in-scope review improvements and specified uninterrupted sequential
implementation and verification. No additional scope or priority decision is needed.
