# Personal agent integrations — 2026-10-18

## User-confirmed scope

Add one-click setup for Claude Code, Codex and GitHub Copilot, with matching CLI
commands. The user chose account-wide personal skills, not repository setup, and
requested Impeccable for the native UX/UI. This extends the existing neutral/teal
Operate workspace; no new visual identity, font, dependency, motion or raster.

App settings exposes a named **Agent integrations** disclosure after Reduce motion.
Three ruled rows show the actual file-installation state, local path and applicable
Install skill / Update skill / Remove skill actions. Conflict/unavailable rows have
their own Check again. Existing browser setup, appearance, storage and support
groups retain their behavior.

## Installation and safety

- `crates/app/src/agent_integrations.rs` embeds the complete portable skill and
  references. `connection.json` binds this installation's absolute CLI/data/cache
  paths, schema and protocol; no PATH or repository-location inference is needed.
- Default personal locations: `.claude/skills/regressee`, `.agents/skills/regressee`
  and `.copilot/skills/regressee` under HOME (USERPROFILE on Windows). Custom host
  search overrides are not edited. Codex's shared location may also be discovered
  by other compatible agents; this is disclosed rather than claiming isolation.
- Setup is offline and local to the user. It does not install the host, change
  permissions, start a controller, enable following or approve a baseline.
- Bounded reads, an exact checksum inventory and ordinary-folder checks preserve
  local edits, additional files and unmanaged/linked installations. Updates stage
  complete files before publication; removal first moves the validated folder out
  of the search location. Cleanup failures name retained files. Locks fail closed.
- GUI file work uses a background worker; CLI integration setup never starts an
  engine. Existing shared-engine saved-data/progress refresh and explicit human
  Follow agent / Take control / handoff behavior remain unchanged.
- Installed means file verification, not proof that a host loaded the instructions,
  can view images or permits execution. No real-host qualification is claimed.

Host location references were checked against the public Claude Code skills,
OpenAI Codex skills and GitHub Copilot agent-skills documentation on this date.
Public usage and limits: [agent-skill.md](../agent-skill.md).

## Bounded UX review

Evidence root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-agent-integrations-vhH8eB/`.

The independent first review returned **fix**, with two material findings:
setup feedback could fall below the compact viewport, and activation moved focus
to an offscreen disclosure. `independent-review.md` preserves that original result.

One fix batch places pending/result/error feedback in the affected host row and
keeps focus there. Completion restores the next enabled local action only if the
initiating handle still owns focus; navigation, dialogs and deliberate focus moves
are never overridden. Post-measure reveal is bounded and verifies the actual scroll
owner. The native transition regression covers actual asynchronous installation,
removal and a stale-read failed installation at both desktop sizes.

`inspection/` retains 32 original GPUI/Blade captures. `confirmation/` contains the
sole final round: 48 frames covering 1280×800 and 820×600, both themes, install,
installed, attention, busy, success and error, plus true document-bottom captures.
The renderer checks nonblank/opaque frames and binds relevant sources in JSON
receipts. These are synthetic settings, not an actual host session or OS-window
input/accessibility test. No web detector was used: it cannot assess Rust/GPUI.

Final independent `confirmation-review.md` disposition: **ship for the two scored
fixes only, not the whole surface**. Row-local pending/result/error feedback and
visible, nonstealing local focus are both **resolved**; no material fix-batch
regressions were identified within that scope. Concurrent action-hierarchy edits
are outside the verdict. The reviewer checked all 48 required PNG/JSON pairs,
including dimensions, decoding, PNG hashes, opaque/nonblank content and current
source hashes, and inspected all frames through contact sheets with additional
full-resolution samples. The reviewer did not rerun compilation or tests.

The final `-tail` captures reach the actual document bottom, including About,
Keyboard shortcuts and a bottom-positioned scrollbar. At 1280×800 they show the
full Watch an agent work guidance. At 820×600 they show its final cloud-limitation
sentence before Storage, **not visual proof of every preceding guidance line**.
This coverage limit does not reopen either scored fix. The original **fix** review
and superseded inspection captures remain preserved; no further visual round ran.

## Verification

- All **37 CLI tests pass**, including three new subprocess installation tests.
- Application tests: **149 passed, 23 ignored, one long persisted-scale test
  filtered**. Five new installer tests cover all hosts, idempotence, path changes,
  missing CLI, locks, local edits, unrelated files and symlink refusal.
- Three integration GUI tests pass, including actual async operations and visible
  focus/feedback through lower-row installation, removal, stale-read failure and
  preservation of a deliberate focus move.
- Seven existing App settings tests and one press/release keyboard regression
  were rerun against the final capture test binary **after the final changes**;
  all pass.
- The native capture test passes with **48 final frames**: six scenes, both themes,
  both desktop sizes and corresponding true document-bottom tails.
- All **75 Python tooling/package tests pass**. `git diff --check` passes.
- Both production executables built successfully **after the final UI fix** with
  `cargo build --locked --offline -p regressee-cli --bin regressee -p regressee-gui --features regressee-gui/ui --bin regressee-desktop`.

These are final results supplied by the main session, not tests rerun by the
finish documenter. Source/system comparison used `agent_integrations_ux.rs`,
`app_settings_ux.rs`, the installer and shared native theme/controls against
`DESIGN.md` and `.impeccable/design.json`. The review's rendered neutral grounds,
ruled rows, system type, shared 40px controls and focus/scroll treatment conform
to that incumbent vocabulary; help uses the existing 13px/18px metrics. No new
token, asset, motion or dependency is needed. DESIGN.md gains only the durable
integration extension; inherited noncanonical section structure and historical
sidecar source-path drift are not repaired or canonized. The sidecar is unchanged.

Two initial combined bulk-test attempts hit the 120-second tool timeout: first
at the long scale qualification, then later in CLI workflows after excluding it.
Neither timed-out command is called a pass. Separate complete CLI and bounded
application runs subsequently passed as listed above. The skipped scale test is
not relevant new qualification for these installation controls.

Concurrent unrelated action-hierarchy changes were preserved. Evidence is Linux
GPUI offscreen rendering and simulated interaction only. No whole-GUI-suite,
strict-lint, scale, native OS input/accessibility, Windows/macOS, clean-host,
actual host loading/permissions or real-agent end-to-end certification is implied.
Verification used only disposable home/data directories: no actual user skills
were installed, and user projects and engine were untouched. No commit or publication.
