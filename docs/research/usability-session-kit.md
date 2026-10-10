# Native usability sessions — facilitator kit

Prepared: 2026-10-10. **Protocol only; no participant sessions conducted.**
Use after local safety verification and meaningful native rendering/input work.
This does not satisfy Phase 7's three-person usability or accessibility gates.
The visual system remains the proposal in `DESIGN.md`; no redesign or visual
approval is implied. Website mobile/tablet profiles are not mobile app layouts.

## Participants and safe setup

- Observe at least three somewhat tech-savvy non-developers representative of
  project managers. Implementers, scripted agents and synthetic accounts do not
  count as representative human participants.
- Include an authenticated-site task with at least one non-implementer. Use an
  owned test account and inert site, not live customer/admin credentials or
  actions. Explain that a browser visit can have side effects.
- Ask consent before observation or recording. Record anonymous session IDs, not
  names, passwords, session state, private queries or account data. Screen/video
  capture needs separate explicit consent and a reviewed private retention plan.
- The facilitator prepares an installed candidate, private fresh application
  data, an owned public fixture and an owned authenticated staging fixture. A
  development loopback runtime must never be pointed at real public sites.
- The participant uses native UI only: no terminal, JSON, developer tools, or
  manual Node/browser install. Facilitator setup is not evidence that public
  first-use installation works; test that separately on clean supported machines.
- Use a small, finite fixture: three pages, a coherent changed icon, a larger
  changed region, one dynamic region, and one explicit inaccessible target. Give
  the facilitator an expected page/profile/result/reference map before the test.
  Do not silently substitute mocked jobs or a web prototype for the native app.

Record exact binary/runtime/source hashes, OS/version/architecture, display scale,
GPU/backend, candidate data root, fixture version, network setup and whether the
runtime was already installed. Keep these private; public summaries omit paths.

## Opening script

“We are testing the app, not you. Please say what you think is happening and
what you expect next. You can stop whenever you want. I may wait before helping
so we can see where the app needs to be clearer. Do not enter real passwords or
use a production website.”

Give goals one at a time. Do not demonstrate the buttons or name the expected
navigation route first. If help is needed, record the intervention before giving
it. Timing is descriptive, not a substitute for correct understanding.

## Task cards and facilitator success checks

| Task given to participant | Observable success; check independently |
| --- | --- |
| “Create a project for this site, but only include its Help section. Leave out the page marked Draft.” | Correct scope and excluded URL in the final preview; no out-of-scope navigation. Participant can explain the limit and incomplete coverage. |
| “Save how those pages look now, on a computer and a phone.” | Explicit successful baseline with exact two profiles; participant distinguishes pages from screenshots. For three included pages, expect six target screenshots. |
| After the facilitator applies the predetermined fixture change: “Check what changed without adding new pages.” | Separate comparison uses the fixed baseline list. Participant distinguishes changed, unchanged, failed and not comparable; failed page is not described as clean. |
| “Inspect the small change and then the larger one. Find the same spot in the old and new screenshots.” | Before/after/diff navigation, zoom and pan remain synchronized; correct source content visible, including a tile seam. No mock viewer state. |
| “This marked dynamic area is expected to vary. Stop it influencing the comparison, without changing the saved screenshots.” | Correct page/profile rectangle, coverage explanation and new analysis revision. Participant understands originals remain unchanged and excessive masking is not a clean pass. |
| “The icon change is intentional, but the other change still needs fixing. Accept only the icon into the next baseline.” | Impact preview and explicit confirmation select the intended page/profile. New immutable composite baseline inherits all others; old version remains available. |
| “Share that baseline, then open it as a separate project.” | Privacy preview understood before export; ZIP excludes login, but raw screenshots and private address queries remain visible. Import creates a new project and does not visit the site. |
| “Set up login for the imported staging project and check its saved pages.” | Participant completes Open browser/I'm ready with their own fixture session; separate destination/session preview. No copied owner authentication or login-screen baseline. |
| “Find the old baseline. What could you remove to reclaim space without losing the active one?” | Participant reads impact and active/shared protection correctly. They may cancel; destructive deletion is never required just to finish a task. Explicit consent is needed before any actual removal. |

Ask after relevant tasks, without coaching:

- “What does this result tell you, and what does it not tell you?”
- “If you change sensitivity, what happens to the screenshots and past review?”
- “What will the recipient see in this ZIP? What must they set up themselves?”
- “Which version is active? How would you recover your previous choice?”

## Session record template

```text
Session: U01 / U02 / U03     Native/platform candidate identity:
Representative role:      Consent: observation / recording separately
Task:
Started / completed / stopped:
Outcome: independent success / helped success / failure / not attempted
Observed clicks, focus, words and misunderstanding:
Help or facilitator intervention:
Saved-state oracle: project / run / analysis / baseline identity (private)
Expected vs actual target counts, references, masks and outcomes:
Issue severity and reproducible sequence:
Participant's description of sharing/deletion/result meaning:
```

Do not prefill records, success counts, quotes or satisfaction scores. A stopped,
unattempted, assisted or interrupted task must not be reported as independent
success. Separate human observations from backend assertions and simulation.

## Keyboard/accessibility companion pass

Record native evidence independently on every claimed platform. Inspect real
focus and reading order, control labels/state, Tab/Shift-Tab, Enter/Space/Escape,
shortcuts, dialog focus return, clipboard/IME, resizing, large text/high-DPI,
contrast/non-color status cues, reduced motion and available screen-reader APIs.
Include error, expired login, low space and destructive confirmation states.
An action-model test is not screen-reader or rendered focus evidence.

Known black/unmapped-window evidence is not a usable session environment: stop
that session and report the native blocker, rather than coaching through invisible
controls. Use one batched inspection and at most one confirmation after a bounded
correction; no open-ended screenshot/polish loops.

## Triage and release evidence

- Blocking: task cannot finish, unintended navigation/credential use, evidence
  loss, or accidental approval/deletion. Repair and independently verify first.
- Major: repeated confusion about failed versus unchanged, active baseline,
  scope, masking or export privacy; unreachable keyboard controls or inaccessible
  state. Fix before release and retest the affected human task.
- Minor: recoverable friction with correct final meaning/state; prioritize by
  recurrence, not cosmetic preference. Preserve the incumbent visual identity.

Report individual task outcomes, interventions, participant count and exact
candidate scope. Phase 7 requires the observed tasks, authenticated human flow,
platform/accessibility evidence and resolved critical findings—not just this
script, clean fixture outputs, or an average completion time. External gates
remain unrun until actual people and qualified native environments are available.
