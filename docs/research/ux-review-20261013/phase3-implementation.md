# Phase 3 implementation — readable, lossless review notes

**Status: implemented; awaiting independent verifier.** This is the implementation
handoff for Phase 3 of `docs/ux-improvement-plan.md`, findings **A3 / B-01**. Work
spanned 2026-10-13–14. Starting HEAD:
`98f2e21a1ddb88e4fe5547dfdbe86260a85151c8`. No child agents, commit or push.
The parent's preexisting Phase 2 push-row edit is preserved; Phases 1/2 remain PASS.

## Implementation and preservation

- `crates/app/src/ui/text_input.rs`: adds the explicitly constructed
  `TextInput::review_note` mode. Other inputs retain their 40px single-line
  geometry, 4,096-byte limit, existing clipboard normalization and shortcuts.
  Shared native `EntityInputHandler` still owns UTF-16 conversion, IME marked
  ranges, replacement, clipboard, pointer selection and input registration.
- `crates/app/src/ui/text_input/multiline.rs`: owns native wrapped shaping,
  paragraph/visual-row byte mapping, multi-row selection paint, caret paint,
  pointer mapping, vertical scrolling and keyboard movement. This is not a tall
  single-line field or a browser/HTML reconstruction. The 138px bordered editor
  has a 300×120px painted inner viewport at both captured sizes: six 20px lines.
  A reserved gutter prevents overflow appearance from changing wrap geometry.
- Up/Down, anchored Shift variants and Page Up/Down move through visual rows
  while retaining the preferred horizontal column. Home/End address the visual
  row; Ctrl/Cmd+Home/End and their Shift variants address the document. Enter
  inserts a newline; Tab remains shell traversal. Pointer drag/Shift-click and
  the existing word-selection fallback work over wrapped rows. Explicit soft-wrap
  affinity distinguishes End from a Down/Home/pointer position at column zero.
- Paste preserves newline and CRLF bytes, Unicode, leading/trailing whitespace
  and blank lines. CR paints as a space without modifying its stored byte.
  Copy/Cut use the exact selected bytes. Existing scalar/word fallback behavior
  is retained, not represented as locale-aware grapheme segmentation.
- `crates/core/src/domain.rs`: one `REVIEW_NOTE_MAX_BYTES = 8192` constant and
  `validate_review_note` helper. `crates/storage/src/review.rs`, explicit Save
  in `crates/app/src/main.rs`, and exit snapshots in
  `crates/app/src/review_ux.rs` share this **UTF-8 byte** policy. The independent
  character count/truncation is removed. Storage serializes exact text; invalid
  saves reject before writes and leave the draft visible.
- Valid loaded 5,000-byte notes can be edited normally. Loaded oversized drafts
  accept strictly shrinking edits even while the result remains oversized;
  additions/equal-size replacements above the limit are rejected. Rejection
  preserves content, selection, marked range and direction and sets visible
  actionable feedback. Unsupported inserted control characters also reject
  visibly; LF/CR/tab are permitted in note mode. Rejection feedback remains until
  an accepted edit or deliberate value load resets it.
- `crates/app/src/viewer_ux.rs`: removes repeated draft prose and the inferred
  “Scroll for more” row. Save and a single status are pinned together outside the
  disclosure body. Status prioritizes invalid byte length, rejected-edit feedback,
  composition, then actual saved/draft state. Scroll indicators paint only from
  actual editor/body overflow. The disclosure body's focus reveal includes the
  note input, including when existing feedback reduces its available height.
- Successful note saving while its editor is open uses the local status instead
  of adding a second saved banner. Ordinary review disposition saves retain their
  existing notification. Newer text is never labeled saved merely because a reply
  arrived; saved/draft state compares exact current text and decision identity.
- Close/Return/Escape retain the draft and restore the note trigger. Target and
  analysis ownership, retained-note reconciliation, save-on-exit snapshot/reply
  guards, baseline eligibility and occluded-control guards remain intact. Phase 2
  blocking copy/priorities are unchanged; a closed dirty note still explains
  “Save your draft in Review note before Next unreviewed.” No website visits,
  captures, login, baseline changes or deletion are introduced.

`DESIGN.md` and unrelated Phase 4/5 surfaces were not rewritten. The maintained
Clear wayfinding tokens, native desktop controls and system type remain the
authority. Impeccable/context ran once; the existing surface brief, incumbent
two-size note PNGs and craft floor were read. The known adaptive/mobile metadata
mismatch was not repaired or applied to this desktop work.

## Tests

New focused tests live in `crates/app/src/ui/text_input/note_tests.rs`,
`crates/app/src/phase3_note_tests.rs`, the shared core helper tests, and one exit
guard regression. Existing `phase4_review_tests.rs` note assertions now require
the actual multiline geometry and caret containment on both axes.

| Coverage | Test evidence |
|---|---|
| 4,096/4,097, 5,000, 8,192/8,193 **bytes**, multibyte overflow | Core helper, native replacement, explicit Save/exit snapshots and real Store round trips |
| Loaded valid 5,000-byte note | Saved through Store, loaded through the real Shell reader, opened/mounted and edited through the native handler |
| Oversized legacy correction | 8,193→8,192 Backspace; 9,000→8,999 Backspace, Delete, shrinking selection replacement; valid corrected exit snapshot |
| Clipboard/newlines | Real test clipboard Paste/Copy/Cut, LF/CRLF/blank lines/Unicode; rejected paste shown in native pixels |
| IME/UTF-16 | Multiline Japanese/emoji composition, selected/marked ranges, exact range reads and rejection preserving selection/mark/direction |
| Native controller geometry | Wrapped row count, Up/Down including consecutive Down from column zero, preferred column, Shift anchor, Page Down/caret reveal, pointer drag, wheel scrolling, resize rewrap and stable height |
| Save/storage | Exact Unicode/whitespace/newline bytes; invalid insert/save does not rewrite history; authoritative Save retains disposition |
| Exit/ownership | Invalid multibyte Save and quit queues nothing; valid exact exit snapshot; stale target/newer retained draft/reply checks; existing exact-reply and termination guards |
| Close/reopen/occlusion | Both-size Return/Escape/focus restoration, dirty-note blocking reason; review/baseline controls cannot activate behind the editor |

Common environment: `LIBRARY_PATH="$PWD/.runtime/link-libs"` and
`TMPDIR=/var/home/olav/Data/opencode/tmp/opencode`. Cargo commands were
`--locked --offline`, with test execution `-- --test-threads=1`.

| Command / gate | Result |
|---|---|
| `cargo test -p regressee-app --all-features --lib ui::text_input::` | 12 passed: 3 note tests, all 8 incumbent generic input UI tests and shared style test |
| `cargo test -p regressee-app --all-features --bin regressee phase3` | 4 focused controller/storage/exit tests passed; opt-in capture excluded by normal test execution |
| `cargo test -p regressee-core review_note_tests` | 1 passed |
| `cargo test -p regressee-app --all-features --bin regressee phase4_review_tests::` | 8 passed, including the updated note geometry/caret test |
| `cargo test -p regressee-app --all-features --lib --bin regressee` | 98 library + 140 binary passed; review, Phase 2 and all exit safeguards included |
| `cargo test -p regressee-storage` | 35 passed, 2 opt-in profiles ignored |
| `cargo test -p regressee-app --features ui --lib --bin regressee --no-run` | Passed; no accidental ui-tests coupling |
| `cargo test -p regressee-app --all-features` | Passed on final source: 98 library + 140 binary + 9 integration tests; 44 opt-in tests ignored; doc-tests passed |
| `cargo build --workspace --features ui --bins -j 4` | Passed on final source |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | Passed on final source |
| `cargo fmt --all -- --check`, `git diff --check` | Passed |

The full package run completed successfully, including the existing distinct
100/1,000-page storage qualification (537.09 seconds). Its complete output is
`full-app-tests.log` in the authoritative final evidence root below. The handoff's
13 source-table entries and three receipt hashes were programmatically checked
against that root and the unchanged final source before delivery.

The preexisting `proc-macro-error2 v2.0.1` future-incompatibility warning remains;
it is not a new Clippy warning. Initial compile/test corrections (module privacy,
settled focus-reveal geometry, opening the real editor before testing enabled
input) were resolved, rather than weakening input/mount/clip assertions.

## Private native evidence and bounded review

**Authoritative final evidence root (0700):**
`/var/home/olav/Data/opencode/tmp/opencode/regressee-phase3-confirmation-20261013-zqltq4ua`

20 fresh original app PNG/JSON pairs, ten at **1280×800** and ten at **820×600**:
`phase3-<size>-{two-sentences,wrapped-selection,newline-paste,long-top,long-scrolled,limit-feedback,legacy-invalid,saved,closed-draft,reopened-draft}`.
Every final PNG was personally inspected at full resolution in the single
collective-correction confirmation. The native input, readable copy, actual
selection/caret/scroll, pinned Save/status, Return, hidden decision actions and
closed-draft recovery match the named states. No further UI polish/captures follow.

- `renderer-proof.png` was freshly generated and personally inspected before
  writing `proof-reviewed.json`: real fonts including æøå, white input, blue
  action/selection, correct RGB blocks and image-atlas sample.
- `phase3-matrix.json` records actual inner/panel bounds, editor/body overflow,
  scroll offsets, status, exact text byte count/hash, target and baseline guards.
  Every inner viewport is **300×120px**. Panel geometry is **360×563px** at normal
  size and **360×379px** at minimum size. The long note's top/end offsets are
  **0 / 2,000px**, with identical text hashes. Disclosure overflow is **0px** at
  normal size and **5px** at minimum size—its small minimum-size scrollbar is real,
  not an unconditional hint. Short notes do not paint an editor scrollbar.
- `source-provenance.json`, per-PNG JSON metadata, `verification-receipt.json`
  and `inspection-ledger.json` bind all 20 originals to current source and record
  PNG hash/dimension checks. All 13 source fingerprints still match.
- `website-fixture-provenance.json` identifies the four reused **synthetic offline
  website images** from the original review root. Only those underlying website
  fixtures were reused; every app PNG is freshly rendered. No user project,
  website, credentials or real website pixels were accessed.
- `capture.log` records the final exact opt-in test passing (42.47 seconds).
  `full-app-tests.log` records the separate final-source package run.

Preserved prior roots:

1. `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase3-20261013-z7gx6cfk`:
   structural capture attempt, 17 app frames retained. It stopped on the narrow
   Saved geometry assertion: the redundant saved banner reduced the body clip.
   The local saved status replaced that banner before the valid review batch.
   This failed attempt was not silently reused as accepted evidence.
2. `/var/home/olav/Data/opencode/tmp/opencode/regressee-phase3-native-20261013-j9nvtfne`:
   successful initial 18-frame review, all personally inspected. One collective
   correction added explicit soft-wrap affinity and consecutive-column-zero Down
   coverage. The final 20-frame confirmation above supersedes it; the original
   pixels/source bindings remain intact.

Reproduce in a **new canonical 0700 root** under the approved scratch directory,
with the labeled website fixtures copied there and their provenance recorded:

```sh
export REGRESSEE_RENDERER_APPROVED_ROOT=/var/home/olav/Data/opencode/tmp/opencode
export REGRESSEE_RENDERER_CAPTURE_DIR=<new-private-root>
export LIBRARY_PATH="$PWD/.runtime/link-libs"
export TMPDIR=/var/home/olav/Data/opencode/tmp/opencode
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::renderer_capture_proof -- --ignored --exact --test-threads=1
# Personally inspect the proof before writing its hash-bound proof-reviewed.json.
cargo test --locked --offline -p regressee-app --all-features --bin regressee \
  ui_renderer_capture::ui_phase3_capture::renderer_capture_phase3 \
  -- --ignored --exact --test-threads=1
```

## Exact final source bindings

| Source | SHA-256 |
|---|---|
| `crates/app/src/main.rs` | `1da0fdf0b5d7844be64e3760481c137cfb29257ef3c8b7409c74077e60a1e05a` |
| `crates/app/src/viewer_ux.rs` | `2d57a60a755a084d3ead4c0bc566ec1b297aa51d9d20c7d66e5dccd2e5b46451` |
| `crates/app/src/review_ux.rs` | `6a3bf2c385530b8c681c7d72cbd1e4d68a09739b1e1e7ec58c9ce99760929fed` |
| `crates/app/src/ui/text_input.rs` | `c81cb2c03508167f04dd73ad335b072ace95734304632480b272a6b5dcf1c003` |
| `crates/app/src/ui/text_input/multiline.rs` | `ffdbe32a94b360ba969c5540e5684f299538158c74132dbf1416e706b048b92c` |
| `crates/app/src/ui/text_input/note_tests.rs` | `0e6a206c84e65bf222c9696061de79a3c3a01a4b55914031c2af75978635abeb` |
| `crates/app/src/phase3_note_tests.rs` | `69be12161176d7f6722cab3ce2609b2c411e086a906b6f638e4c76de438b8e25` |
| `crates/app/src/exit_guard.rs` | `8d6143b037b9f99ec38b53b80638eaab8a6e70a810edc9347a1034124869e588` |
| `crates/app/src/ui_renderer_capture.rs` | `9790fc1ce731d6e12a8af045a0da9a23b5d7a2a3d8e5467615df094e13e7c923` |
| `crates/app/src/ui_phase3_capture.rs` | `aae559e0dbd0bee0288e4c05c053fc213a90b4245ede32ff3655306bad21751e` |
| `crates/app/src/ui_review_matrix.rs` (unchanged fixture) | `8070496a821b965b45229278180c737c51fb517ad28d734f919ef39b57655d7d` |
| `crates/core/src/domain.rs` | `8bd984923b6131aa38e2b93a2b8e04350d668d34c7be16d3c167b89c40b73795` |
| `crates/storage/src/review.rs` | `6b1c0e381322e384093c0c7273fea2b5b9e301e71573f0dc6e512c43afdc7612` |

Receipt hashes: source provenance
`e0a5ff86cf6fc3e877de3d7f64007a6d33ad32f4a317b57a62dce791ad7cbfa2`;
matrix `880ce95165702a5c10db6daa1bb5a1ea3b97c9cfd4a4bd3254a79f26adcdd781`;
inspection ledger `6d90fbf9f58340def64a8265237d183ccf58379e3103eae3596457c20dbb5aed`.

## Qualification limits

Evidence establishes Linux **GPUI/Blade offscreen paint and simulated native
controller behavior**, not OS window presentation, real keyboard/pointer/clipboard
delivery, OS IME, assistive technology, high contrast/DPI, Windows/macOS or release
acceptance. Unicode bytes and simulated UTF-16/IME ranges are tested; native font
coverage is not qualified (the captured host does not visibly paint the sample
crab emoji). No accessibility bridge, new font/runtime distribution, undo system
or locale-aware editor engine is introduced by this bounded phase. No native
markup detector ran: it does not cover Rust/GPUI. The independent verifier must
issue the phase verdict; this report does not self-certify PASS.
