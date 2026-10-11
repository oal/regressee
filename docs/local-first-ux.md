# Local-first UX pass

User-approved scope: the local-first speed pass and **all improvements from the
2026-10-11 UI critique**, preserving Clear wayfinding and the inspected screenshot
when a filter hides its target. Development-host checks are not native usability
or release qualification.

## Interaction contract

### Find pages without submitting a search

Discovery and review filter their loaded page/target lists as text changes.
Typing is read-only: it does not change screenshot selection, discard ignored
areas, clear approval selections, save project settings, or visit the website.
Clear restores the list, and Ctrl/Cmd+F focuses the mounted search field rather
than stealing focus from a dialog. Match counts and empty-state recovery refer
to the actual filtered inventory. Bulk exclusions still use exact matching IDs;
baseline capture remains a deliberate action on a reviewed inventory.

An inspected target outside the current filter stays visible with an explanatory
notice. This avoids replacing evidence underneath a user who is drawing ignored
areas or narrowing a queue.

Discovery owns a full-width inventory, included/excluded counts and its coverage
limits. Earlier check metrics and screenshots are not mixed into that view.
Representative image preview is explicit and can be hidden; only a matching
representative capture may appear beside the inventory.

### Review without opening technical tools

Intentional, Needs fix, Unreviewed, Next unreviewed, progress and explicit note
saving belong to ordinary review. Decisions and notes bind to the exact analysis
revision and page/profile, not whichever historical decision happens to load.
Switching targets retains note drafts under their original binding. Next
unreviewed refuses unsaved note/ignored-area drafts rather than losing them.

**Select for baseline update** is separate from classification. **Review approval**
previews selected reference changes; confirmation remains explicit. Ineligible
or stale evidence explains why it cannot be approved. Review alone never changes
a baseline.

The page queue can collapse to give evidence more width. Compact captions retain
before/after, origin and original screen dimensions; Details exposes exact
provenance. At narrow sizes the header and ordinary decisions compact, and only
one of note entry, provenance and technical Tools is expanded at a time. Closing
a disclosure preserves its drafts.

The capture/review header uses grouped, labelled counts rather than a prose
ledger. Capture-only, comparison and saved-baseline states are distinct; actual
gaps and limited/interrupted inventory remain visible. Repeated context, full
addresses and exact run IDs live in Details. Single-image inspection has one
compact origin/screen-size caption, not a second stack of explanatory text.

Tools provides keyboard creation and editing of ignored rectangles: Add ignored
area, whole-pixel X/Y/Width/Height, Apply area edit, then Save ignored areas.
Apply edits only the draft. Save revalidates original-coordinate geometry, exact
capture-pair binding and remaining comparable coverage. Unapplied, invalid or
all-masked geometry stays editable. Keyboard geometry focus scrolls into view.

### Leave drafts deliberately

Application Quit and the owned window's close request share Cancel / Save and
quit / Discard and quit. Save checks project/access, network, ignored areas and
all retained review-note drafts, then awaits each authoritative reply. Failures,
unexpected replies and newer edits leave the application open. Note saves retain
their original page/revision and disposition. Unapplied numeric geometry must be
applied first. This does not visit websites or approve/save a baseline.

Explicit discard drops drafts only; previously saved evidence and decisions
remain saved. Direct platform termination cannot be vetoed by GPUI's final
cleanup callback; native OS close/quit/input qualification remains pending.

### Keep screenshot reads proportional to the selected evidence

Read-only viewer requests have a separate, coalescing lifecycle from committed
storage work. A newer navigation request supersedes an older one; old responses
cannot show pixels under a newer target's captions. Response binding includes
the project, run, target, analysis and request generation.

Revision-bound metadata and a byte-bounded decoded-tile cache reduce repeated
project-wide loading and decoding. Speculative next-target reads are bounded and
lower priority than requested work. Cache accounting is not a whole-process or
GPU-memory claim. New disk reads remain verified, and cached pixels are display
data, not approval authority. Approval, activation, export and mutation retain
their authoritative integrity/revision checks.

The baseline environment is loaded with the actual referenced source run, not
inferred from today's project settings or dependent on opening History.

### Load saved configuration and preserve compatible browsing state

Saved network configuration loads automatically before editing is enabled.
Unloaded blank fields must not be interpreted as saved empty configuration.
Saving stays explicit; delayed reads must not overwrite an edited draft.

Returning to a local surface reuses appropriate presentation data instead of
discarding it solely because navigation occurred. Compatible refreshes preserve
the selected record and viewport. Evidence changes still invalidate approval
previews; remembering inspection position never preserves stale authorization.

### Choose history records directly

History separates **Runs**, **Baselines** and **Storage**, with direct selection,
full-width rows, capture dates, run types/outcomes and
active-versus-retained baseline state. Search/filter scope is explicit for the
loaded history page; unloaded pages must not be silently treated as searched.
Imported/composite provenance and deliberate baseline/deletion/ZIP actions stay
available next to the selected record.
Refresh history retrieves the current page again and refreshes baseline evidence
without turning the separately measured storage census into a prerequisite.

History retrieval is independent of measured storage totals. Unmeasured totals
are unavailable/pending, never zero; a separate refresh requests a filesystem
measurement. Browsing local rows should not require that census to finish.

Open check results / Open screenshots reopens the selected saved record for
inspection. A check uses its original baseline, analysis policy and ignored-area
revision; a capture shows only its originals. Inspection cannot write decisions,
reanalyze, edit masks, resume or approve a baseline. Return to latest evidence
leaves inspection. Missing/corrupt originals fail closed, and delayed replies
cannot replace a newer selection or draft. Historical reads do not change
baseline activation or persisted display pointers. Authoritative verification
loads each distinct source run once per read, not once per reference/report.

### Correct cheap mistakes beside their fields

Local syntax/range feedback belongs next to the affected field, with a useful
correction. Incomplete text while editing is not a global application failure.
Authoritative validation still runs on explicit save. File inspection, website
reachability and login checks remain asynchronous/deliberate rather than being
triggered by every keystroke.

Touched invalid fields produce a correction summary. Go to next correction
opens the appropriate setup stage/advanced fields and focuses/reveals the input
without submitting anything. F1 and App settings → Keyboard shortcuts show a
bounded help dialog, trap focus and restore it when dismissed. Network settings
state when configuration is loaded and disclose local versus capture behavior.

## Verification boundary

The previous speed-pass evidence is preserved in
`docs/research/local-first-ux-verification-20261011.md`. The critique follow-up's
separate gate, renderer checks and remaining qualifications are recorded in
`docs/research/ui-critique-improvements-verification-20261011.md`.
The initially incomplete visual corrections were carried through at the user's
request and confirmed in a fresh 36-image matrix at both sizes; see
`docs/research/ui-carry-through-verification-20261011.md`. History selection,
Tools evidence height, complete recovery/help and actual capture-only states
now have both regression and pixel confirmation. Tools uses a full-height
inspector in the queue's place and temporarily unmounts ordinary review actions;
closing it restores that review layout and retained drafts.
Long forms and technical Tools still require scrolling. Renderer fixtures use
synthetic evidence, not actual website results. No existing user project was
modified and no website was visited during this implementation pass.

Offscreen pixels and simulated input establish only development-host behavior.
Real OS pointer/keyboard/IME, accessibility, native presentation latency and
Windows/macOS qualification remain separate. Historical scale/capture timings
must not be described as native interaction latency.
