# Phase 3 independent verification — 2026-10-14

## Verdict: PASS — entire scoped Phase 3

**A3 and B-01 are resolved in the bounded source/controller/storage/Linux
offscreen scope of `docs/ux-improvement-plan.md`.** No in-scope repair findings.
This is not real OS input, IME, accessibility, cross-platform or release acceptance.
The review directory remains `20261013`; this verification was performed on
**2026-10-14**, independently of the implementation handoff.

Read the complete plan, A3/B-01 assessments, Phase 3 handoff, current diff and
relevant source. Loaded Impeccable/context once and read the existing desktop
surface brief. Preserved native desktop **Clear wayfinding**; the known
adaptive/mobile metadata mismatch was not applied or repaired. No child agents,
application/test source edits, commits or pushes. Only this repository report was
added. Parent owns the plan/status update and subsequent commit/push.

Verified HEAD: `98f2e21a1ddb88e4fe5547dfdbe86260a85151c8`, with the uncommitted
Phase 3 implementation. No broader phase or release verdict is implied.

## Independent evidence integrity

Authoritative final implementation evidence, inspected **all 20 original PNGs at
full resolution**, ten at each of **1280×800 / 820×600**, plus renderer proof:

`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase3-confirmation-20261013-zqltq4ua`

- Root permissions independently checked: **0700**.
- All **13** handoff/source-provenance fingerprints match current source.
- All **27** embedded source fingerprints in **each** app frame's JSON match the
  actual `include_bytes!` sources: **540** source-binding checks.
- All **20** PNG hashes, dimensions and opaque alpha match their metadata.
- Source provenance, matrix and implementation inspection-ledger hashes match
  the three hashes recorded in the handoff/receipt.
- Proof PNG hash matches both its metadata and personal-review receipt. Its
  fonts, æøå, white field, selected control, RGB blocks and atlas sample are visible.
- Four underlying **synthetic offline website fixture** copy/original hash pairs
  match the fixture ledger. These reused website images are not reused app PNGs
  or real-user/site evidence. No websites or user projects were accessed.

Independent logs and hash/pixel receipt (private; not committed):

`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase3-verifier-20261014`

`evidence-integrity.json` SHA-256:
`b7ba5c5039c965f6cef68518e99012366eb2cb2079bf2187a32799e60d34cac1`.
It records individual PNG hashes, source checks, actual text-ink row bands,
overflow/scroll observations and verifier log hashes. The failed initial and
superseded review roots remain historical evidence, not substituted final proof.
No new render/polish cycle was needed.

## Requirement-to-source/test verification

### A3: a native multiline editor, not a tall single-line field

- Only analysis field **9** uses `TextInput::review_note`
  (`crates/app/src/main.rs:388–397`). Generic fields still use `with_value`.
- `crates/app/src/ui/text_input/multiline.rs:23–112,399–518` shapes wrapped native
  paragraphs into visual rows with UTF-8 byte ranges. Paint uses those rows for
  multi-row selection, native wrapped text, a visible caret and clipped vertical
  viewport. Native input registration remains `ElementInputHandler`; no HTML or
  one-line shaping masquerades as multiline input.
- Pointer mapping subtracts viewport origin and adds actual scroll offset;
  row hit-testing uses shaped glyph positions. Drag selection shares anchored
  selection semantics with the incumbent input. Soft-wrap upstream/downstream
  affinity distinguishes visual End from the next row's Home/column-zero Down.
- Up/Down retain the preferred horizontal column; Shift variants retain the
  selection anchor; Page navigation reveals the caret. Home/End address visual
  rows; Ctrl/Cmd+Home/End address the document. Enter inserts LF; Tab remains shell
  traversal. Wheel scrolling is bounded and consumed when the editor moves.
- Note Paste keeps LF, CRLF, blank lines, Unicode and whitespace exactly; Copy/Cut
  use exact selected bytes. CR's paint-only space substitution preserves storage
  bytes. UTF-16 conversion, marked ranges and composition underlining stay in the
  shared native handler (`ui/text_input.rs:134–148,286–312,375–517`).
- Independently rerun `note_native_vertical_selection_caret_scroll_pointer_and_resize`
  checks actual wrapped row count, consecutive column-zero Down, preferred column,
  anchored Shift changes, Page Down/caret containment, simulated pointer drag,
  wheel movement, width rewrap, unchanged text and stable height.
- `note_clipboard_newlines_unicode_limit_and_ime_are_exact` exercises the GPUI test
  clipboard and native replacement/mark handlers, including Japanese/emoji UTF-16
  selected/marked ranges, exact range reads, rejection preservation and Enter.
  These are **simulated native-controller** passes, not OS IME/clipboard passes.

### B-01: one exact, lossless 8,192 UTF-8-byte save policy

- Shared `REVIEW_NOTE_MAX_BYTES` / `validate_review_note`:
  `crates/core/src/domain.rs:882–891`. Note editor construction uses that constant;
  storage, explicit review/Note Save and exit snapshots use that helper
  (`storage/src/review.rs:323–326`, `main.rs:1938–1968`,
  `review_ux.rs:199–229,412–423`, `exit_guard.rs:374–395,460–476`).
- Independent character-count/truncation is removed. Valid saves serialize the
  supplied exact text; invalid saves reject before writes or queuing review work.
- Native replacement checks resulting **bytes**, not Unicode scalar count.
  Strictly shrinking replacements of loaded oversized notes are accepted even
  while still oversized; additions/equal-size replacements above the limit reject
  (`ui/text_input/multiline.rs:220–250`). Content/selection/mark/direction remain
  intact on rejected insertion/composition, with actionable visible correction.
- Boundary tests cover **4,096 / 4,097 / 5,000 / 8,192 / 8,193 bytes**, multibyte
  overflow and exact LF/CRLF/blank-line/whitespace round trips. A real Store-saved
  5,000-byte note is loaded through the Shell reader and ordinarily edited through
  the native handler. Backspace 8,193→8,192 and 9,000→8,999, Delete and shrinking
  selection replacement remain accepted; corrected text becomes saveable.
- `phase3_note_tests` independently passes real Store round trips/rejected-history
  preservation, explicit Save and pending exit-snapshot boundary checks. The new
  exit regression rejects ASCII and multibyte overflow before any save request,
  preserves draft text and settles a valid exact Unicode/newline save. Core byte
  policy and existing storage tests independently pass as well.

### Draft identity, asynchronous safety and occluded decisions

- `sync_review_note` keeps the active `(analysis, target)` draft before switching
  identities; the same identity is not rehydrated away. Save checks that identity;
  exit snapshots bind mounted text to **active**, not a newly selected target.
  Exact authoritative replies retire only matching retained text, not newer drafts
  (`review_ux.rs:197–257,412–482`).
- Exit saving rejects composition/invalid notes, snapshots each target and saved
  disposition, awaits the matching request/kind/exact note confirmation and checks
  the complete signature. A failure, mismatched confirmation or newer draft keeps
  the app open (`exit_guard.rs:348–395,460–476,566–683`).
- New both-size close/Escape/reopen/identity test preserves text, restores the
  Review note trigger, keeps Next blocked for dirty text, rejects wrong-binding
  saves, retains another target's newer note through an older reply and verifies
  disabled review/baseline decisions while the editor is open.
- Existing two-target exit/disposition, wrong-request/progress/failure,
  unconfirmed/newer-edits, retained-note and stale History-reply tests all pass.
  Existing Phase 4 review tests pass, including actual both-axis caret/editor
  containment and draft restoration at both sizes.
- Inspector `.occlude()`, mounted/enabled guards and hidden approval/review bars
  remain in production (`viewer_ux.rs:235–291`, `main.rs:5585–5589,7654–7662`).
  Hidden decisions cannot be activated through the shared mounted-control path.
  Save note remains explicit and preserves disposition; it is not baseline approval.

### Shared single-line behavior is retained

Generic `with_value` still has the 4,096-byte limit, 40px border, one shaped line,
horizontal caret reveal, LF/CR-to-space clipboard normalization and incumbent
shortcuts/IME handling. Note-specific key handlers and newline preservation mount
only in multiline mode. All **eight incumbent input UI regressions**, the shared
style test and the three new multiline tests independently pass (**12 total**).

## Two-size original-pixel inspection ledger

Every listed suffix was inspected in both `phase3-1280x800-*` and
`phase3-820x600-*` originals; filenames alone were not accepted as state evidence.

| Scene suffix | Actual observed/checked result at both sizes |
|---|---|
| `two-sentences` | Entire 87-byte realistic note readable on **three wrapped lines**, not only its tail; no horizontal navigation or editor scrollbar. |
| `wrapped-selection` | Sky selection spans the first two visual rows; caret appears at the next row's start, without obscuring text. |
| `newline-paste` | Separate paragraphs, blank line, wrapped confirmation sentence and café/Japanese visible; exact 163-byte input asserted by native clipboard capture. |
| `long-top` | Six visible text rows; top editor scrollbar; beginning of 2,896-byte note readable. |
| `long-scrolled` | Final Review 35 and trailing-line caret visible; scrollbar reaches bottom; **2,000px** offset and identical text hash to `long-top`. |
| `limit-feedback` | Exact 8,192-byte draft retained after rejected native paste; readable red “Not inserted … Shorten the note or paste less” beside pinned Save. |
| `legacy-invalid` | 10,587-byte invalid draft remains visible; red byte-limit / kept-draft / nothing-saved explanation fits beside Save. |
| `saved` | Full realistic note remains readable; one local “Saved · 87 bytes” status, no redundant saved banner consuming editor space. |
| `closed-draft` | Editor absent; untinted before/after evidence and Review note trigger restored; dirty-note recovery reason visible; Next disabled. |
| `reopened-draft` | 125-byte draft, including the newly entered delivery sentence, reappears intact with visible caret and Unsaved status; same text hash as closed state. |

The input border is **326×138px**, with a **300×120px** painted inner viewport
(six 20px rows). Independent pixel extraction confirms **three** text-ink bands
for the realistic note and **six** for long-top/limit/invalid scenes, agreeing with
personal inspection and native row/layout tests—not an inference from field height.
Normal inner origin is **(926,323)**; minimum is **(466,311)** or **(466,306)** after
the actual disclosure focus-reveal scroll. Panel bounds are **(896,200,360,563)**
and **(436,188,360,379)**. Save and Return remain fully visible in all open-editor
captures; the red feedback is not clipped and review/baseline decisions are absent.

Only actual overflow draws indicators: short editors have none; long editors do.
Disclosure-body overflow is **0px** at normal size and **5px** at minimum size,
where the thin body scrollbar is genuinely justified. Production uses measured
`max_offset` and shaped-text height, not inferred window/panel type or unconditional
“Scroll for more” copy. White fields, deep-blue navigation, system type and sky
selection preserve the existing desktop design.

Evidence interpretation caveats (not application repair findings): minimum-size
short-note scenes correctly say Saved because the shared isolated fixture was
saved in the preceding normal-size sequence; dirty states are separately verified.
The matrix's closed-frame panel/input coordinates are retained debug geometry,
**not proof of a mounted closed editor**; the original closed PNGs and mounting
guards establish the absence of the editor. The sample crab emoji does not
visibly paint on this host; byte preservation passes, font coverage does not.

## Independently executed tests and gates

All Cargo commands used `--locked --offline`, with
`LIBRARY_PATH="$PWD/.runtime/link-libs"` and
`TMPDIR=/var/home/olav/Data/opencode/tmp/opencode`; executed tests used
`-- --test-threads=1`. Logs are in the verifier root above.

| Command (Cargo flags as above) | Actual result |
|---|---|
| `cargo test -p regressee-app --all-features --lib --bin regressee` | **98 library + 140 binary passed**, 23 opt-in tests ignored; includes all new note regressions, existing input, Phase 4 review, draft/review and quit/exit tests. |
| `cargo test -p regressee-core review_note_tests` | **1 passed**. |
| `cargo test -p regressee-storage` | **35 passed**, 2 opt-in profiles ignored; doc-tests pass. |
| `cargo test -p regressee-app --features ui --lib --bin regressee --no-run` | **PASS**; UI-only targets compile without `ui-tests` coupling. |
| `cargo build --workspace --features ui --bins -j 4` | **PASS**. |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | **PASS**. |
| `cargo fmt --all -- --check`; `git diff --check` | **PASS**. |

The existing `proc-macro-error2 v2.0.1` future-incompatibility advisory is not a
new Clippy warning. Full workspace/all-features integration remains the plan's
Phase 5 gate; this verifier did not represent the implementer's separate long
integration run as an independently rerun test.

## Qualification boundary and parent handoff

**PASS authorizes the parent to mark only Phase 3 verified and perform its normal
diff/status/commit/push protocol.** No repair batch is requested. Tests and fresh
pixels support the complete Phase 3 scope, not a general usability claim.

Real OS keyboard/pointer/clipboard delivery, IME candidate/commit integration,
window-close/platform termination, assistive technology, high-DPI/high-contrast,
font coverage and Windows/macOS qualification remain existing documented limits.
The controller tests do not qualify those systems. No locale-aware grapheme/word
engine, undo system, accessibility bridge, runtime distribution, theme replacement
or unrelated release blocker is added to this phase.
