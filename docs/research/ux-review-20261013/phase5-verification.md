# Phase 5 and final integration — independent verification

**Date: 2026-10-14. Verdict: PASS — the whole scoped Phase 5 and final integration
of `docs/ux-improvement-plan.md`.** No remaining in-scope acceptance repair is
identified. The parent may update completion status and commit/push; this verifier
has done neither. This is bounded source/controller/offscreen acceptance, **not
native-input, accessibility, cross-platform, runtime-distribution or release
qualification**.

## Scope and independence

Read the complete plan, both assessments, synthesis, preceding verifier reports,
Phase 5 handoffs/receipts, `DESIGN.md`, current production/test/capture source and
the full integration diff. Loaded impeccable/context for this native desktop
review. The latest **whole-scope PASS** verdicts for Phases 1–4 stand; their earlier
FIX reports and failed evidence remain preserved. Clear wayfinding, the approved
native controls and desktop platform remain intact; adaptive/mobile metadata was
not treated as the product platform.

HEAD remained `595ebdc8e58b1c9f7a000ff88288197060eb3a20`. No production/test-source
edits, child agents, commits, pushes, real website visits, user-project access or
additional app screenshot batch. This report is the verifier's only repository
addition. Independent diagnostics and logs are private:

```text
E = /var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-20261014-4zgty46x
V = /var/home/olav/Data/opencode/tmp/opencode/regressee-phase5-verifier-20261014-hgy6b04v
```

Both roots are **0700**. Existing implementer changes were preserved.

## Native pixels, state and provenance

- Personally inspected the renderer proof, **all 18 labelled full-frame sheets**
  containing all **104 final frames**, and **all ten native-scale reading sheets**
  containing **58 crops**. Also opened **all 104 original app PNGs directly**, at
  original resolution, including both-size customization/limitations, validation,
  History, network/privacy/exact-detail tails, Analysis, menus and blockers.
  Normal-size full sheets are 50% derivatives; narrow sheets and reading crops
  are native scale. None is counted as a fresh original capture.
- Independently cross-checked **104 PNG/JSON pairs**, 52 distinct scenes per size,
  PNG chunk CRCs and decoded dimensions, every original/sidecar/state hash,
  **34 current embedded source bindings per PNG**, proof/review receipt and four
  explicitly synthetic/offline website fixtures. The current **129-entry final
  source manifest**, **124-entry implementer gate manifest**, and **125-entry
  independent Rust/Cargo/GPUI manifest** match their actual files and agree on
  shared sources. The initial historical manifest is not used for acceptance.
- Cross-checked both durable repository receipts against their private copies,
  complete native/preflight/gate log hashes, and all **28 derivative hashes**.
  Independently extracted/reproduced the image regions: **104 full-frame regions
  and 58 reading regions are pixel-equal to the corresponding original pixels**
  or declared 50% resize. Derivatives do not manufacture missing content.
- Rechecked all **148 recorded selector/owner/window containment assertions**.
  Reviewed actual activation, mounted/enabled controls and fixture construction,
  rather than trusting scene filenames or the integrity script alone. Replayed
  `scripts/ux-phase5-evidence.py` successfully, then ran the verifier-owned
  `V/independent-final-check.py` successfully. Its receipt is
  `V/independent-final-check.json`.

No confirmed new clipping/overlap defect in the qualified revealed controls.
Long surrounding content may be above/below its genuine scroll clip. Distinct
custom fields, actions, limitation paragraphs and exact details are separate
reading positions, **not a claim that the entire long form fits simultaneously**.
Fitted website imagery is an overview, not fine-text readability qualification.

### Whole-plan finding and acceptance judgment

Final originals are named `E/phase5-{1280x800,820x600}-<scene>.png`.
The following judgments combine these current pixels, production source, the
independently completed tests and accepted preceding-phase evidence.

| Requirement / findings | Independent result and evidence |
|---|---|
| A1 connected setup, safe progress/back/save | **PASS.** Numbered connected Current/Completed/Upcoming progression, stage-preserving invalid drafts and pinned safe actions remain. `wizard-website`, Access and Pages frames corroborate the hierarchy. Current `phase1_setup_tests`, `phase5_setup_tests` and `phase5_repair_tests` pass, including exact-save continuation, clean Continue, safe Back and newer-draft/request races. |
| Contextual consent and public/private/optional Access | **PASS.** Distinct-origin consent/reset, required login controls, independent optional configuration and complete Project settings remain. `access-public` and `access-optional-mapping` use mounted traversal. `private-login-loading` has saved **InteractiveLogin**, absent status and a **real outstanding read-only EnvironmentStatus request**; Continue/Open login are blocked. It does not fabricate successful login or visit a website. Prior private unsaved/saved/browser-open/ready and consent acceptance is supported by unchanged relevant logic and current regressions. |
| A5 product language; B-02 runtime truth | **PASS.** No rail slogan or old About entry; About Regressee retains version/license/privacy/build limits. Shared runtime presentation and fresh-entry read remain truthful across all seven states, with no unconditional installation promise. Current runtime-entry/concurrent-save and copy tests pass. Final Access/network frames corroborate development-build limits; prior About/runtime pixels remain historical, not falsely current whole-file-bound evidence. |
| Pages essentials, Mobile/custom profiles/limitations/alternates; Q-03 | **PASS.** `mobile-revealed-selected` shows the actual selected, mounted row inside its form clip at both sizes; Tablet is independently activated. Custom **1024×768** edits/addition and Add/Remove actions are real. Both full capture/image limitation paragraphs are readable at their own offsets. Visible-screen, replacement-image and Fast choices are actually activated and assert resulting profile/speed values. Draft edits do not queue capture, discovery or baseline changes. |
| Invalid Advanced correction/reveal | **PASS.** `invalid-advanced-revealed` follows real Save rejection of **Parallel active pages = 9**; complete field/error/help fits its actual owner, Advanced is open, field focused and no work queued. The capture's subsequent traversal is distinguished from automatic Save reveal; the additional independent callback-delivery probe below verifies that automatic path separately. |
| Post-inclusion discovery | **PASS.** Four scenes show **128 discovered**, included counts **74 → 75 → 74 → 75** after actual single-page/matching-page actions. Search retains the outside-match selection and does not itself change inclusion. Mounted Restore cannot restore an independently ineligible page. Current regression also verifies unchanged persisted evidence. |
| A2 evidence space, identity and menus | **PASS.** `review-compare`, `results-hidden`, navigation toggle and full-width Before/After retain long-target/profile/recorded-environment identity and untinted originals. Measured pane heights are **389px / 234px** at normal/narrow size for Compare, hidden Results and either full-width side, comparing the same navigation state. View is trigger-attached with full selected rows; View/Zoom are bounded and keyboard-dismissable. Existing drawing-exit/source-pan tests pass. |
| B-03 blocker explanations; selection versus approval | **PASS.** Both-size dirty note/area reasons are visible; current tests also cover unapplied geometry, pending analysis, noncomparable evidence and exhausted matching queues without weakening guards. `baseline-selection` distinguishes **1 selected · 1 ready to update** from confirmation; empty selection is **0 selected**. No baseline confirmation occurs in the final recipe. |
| B-04 derived evidence/legend | **PASS.** Compare/Before/After originals are untouched; Difference/Overlay/Heatmap are named modes. `heatmap-with-ignored` uses a saved source-bound mask and the production analyzer with **120,000 ignored pixels and nonzero changed pixels**, displaying both production legend colors. Source artifacts are hash-checked after capture. |
| A3/B-01 readable, lossless notes | **PASS.** `multiline-note` uses the real wrapping editor, pasted line breaks, caret, byte status and explicit Save/Return. Dirty close/reopen and target/reply/occlusion ownership remain protected. Full gate includes exact **4,096/4,097 and 8,192/8,193-byte**, multibyte, loaded 5,000-byte/oversized corrective-edit, clipboard/selection, generic input and simulated IME tests. Shared 8,192-byte validation rejects explicitly, without truncation or hidden save. Prior long-scroll/error/exit acceptance is not replaced by a short final note alone. |
| A4 progressive Analysis and correct owner | **PASS.** Actual saved result/current policy precedes Sensitivity. Ignored areas exposes Draw/coordinates; advanced settings and switches remain below common choices. Final basic/coordinates/tail frames use the inspector owner. Current hierarchy tests traverse **all nine advanced fields, all four coordinate fields and both switches**, including correction, draft/revision preservation, hidden mutations, useful Cancel/previous and real debounced reanalysis. Originals/baseline remain protected. |
| Changed/Gaps/no-significant-change/grouped filters | **PASS.** Activated filters assert **10 / 1 / 1 / 12** results, production classifications and preserved selected pair. Gaps comes from a genuine stored synthetic timeout outcome, not relabelling success. Grouping yields six Desktop then six Mobile results. Narrow originals establish heading/count and retained evidence, **not simultaneous visibility of an entire matching row** with the saved-area notice present; the handoff accurately discloses that limit. |
| History/Settings minor hierarchy/scope/terminology | **PASS.** Actual stored first-page **64 runs**, selected last index **63**, selected-record actions, second-page loading and optional provenance are genuine transitions in their respective inner/outer owners. Current tests retain zero/few/many/no-match and loaded-page scope, sparse row heights and bounded pagination. App settings heading agrees with navigation. “No significant change” remains below thresholds, not identical pixels. |
| Network/portability/privacy/deletion consequences | **PASS.** Both-size network closed/open and export/import/delete consequence/exact reading positions preserve proxy/CA/VPN/enterprise limits, private-original exposure, **NOT redaction**, no sessions/cookies, recipient-own-login and permanent/no-undo warnings. Exact disclosure is distinct from History provenance. Import uses the production post-picker path with a locally constructed synthetic archive, not claimed OS picker delivery. UI stops at preview/cancel; original hashes, existing baseline, project count and selected run retention are asserted. |
| Final integration repairs and focus safety | **PASS.** Projects/App settings secondary actions now reveal in their real outer owner; six disclosures have a non-aliasing reserved focus range; explicitly anchored measurement includes complete bordered choice/check/action geometry. Actual-vs-measured Public/Login rows agree at both sizes. Four-project traversal verifies distinct enabled focus handles and reaches Cancel without import/baseline writes. Shared mounted/enabled/occlusion gates, draft/reply guards, off-thread I/O and bounded lists are preserved. |
| DESIGN/report/receipt completeness | **PASS.** `DESIGN.md` describes the shipped native refinements without inventing mobile UI, motion, themes, new algorithms or release acceptance. `final-integration.md` maps all A/B findings to tests/evidence; durable state/source/geometry/gate receipts and inspection ledger agree with the actual artifacts. No in-scope hidden TODO substitutes for a repair. |

The Phase 5 production delta is confined to reveal coverage, non-aliasing focus
handles, corrected painted measurement and paint-only selectors. Setup consent,
runtime observation, note editing/limits, review/analysis algorithms, History
storage and consequence authorization retain their preceding accepted logic.
Earlier app PNG whole-file hashes are **not claimed to match** today's changed
`main.rs`/capture harness. Carry-forward rests on scoped source-delta review,
unchanged relevant components, current regressions and this fresh final batch.

## Automatic Save-reveal: independent diagnostic

The ordinary final regression/capture invokes `assert_form` after Save, which
traverses away and back; that alone proves keyboard recovery, not automatic reveal
immediately caused by Save. Investigated this explicitly rather than accepting
the stronger interpretation.

- Stock GPUI `platform/test/window.rs:289` discards `on_request_frame` registration.
  `run_until_parked` drains tasks; manual `Window::draw` does **not** execute queued
  `Window::on_next_frame` callbacks. Therefore the first scratch probe's failed
  automatic geometry was not sufficient evidence of a production defect. A late
  frame-delivery attempt also retained callbacks from earlier traversal and is
  preserved as diagnostic, not production qualification.
- In an owned scratch GPUI copy only, retained that registration and explicitly
  delivered the **unchanged real Window frame handler** throughout ordinary draws
  and traversal. The handler drains callbacks before rendering
  (`vendor/gpui/src/window.rs:1051–1098`); no application reveal, refocus or scroll
  assignment was injected. Application production sources remained byte-identical.
- Independent sequence: edit invalid parallelism → **collapse Advanced** → activate
  mounted Save → ordinary frame delivery only. **PASS at both sizes:** Advanced
  automatically opens, the invalid field owns focus, full field/correction fits
  `screen_scroll`, no pending task/save request is created. Test-platform field
  bounds/owners are `216,603.5,680,123.5` / `192,234.5,1088,500.5` at 1280×800,
  and `216,350,580,144` / `192,259,628,243` at 820×600. These simulated-font bounds
  are not substituted for the final native PNG measurements.

Probe/driver/log/command receipt remain under `V/shadow/` and
`V/independent-save-reveal-continuous-frames*`; **1 passed, 0 failed**, exit 0,
7.28s wall time. This additional probe is **not added to the workspace total** and
does not establish OS frame scheduling/input. Repository-source app/test targets
were rebuilt afterward; focused Phase 5 and strict Clippy were rerun successfully.
All earlier probe failures remain intact.

## Independently completed final-source gates

Commands ran foreground to completion from the repository root with
`LIBRARY_PATH="$PWD/.runtime/link-libs"` and `TMPDIR=V`. Source hashes were checked
before/after; no source drift and **no command left running**. Complete logs and
exact command arrays/exits/timings are in `V/gates.json` and `V/restore-gates.json`.

| Exact command | Independent completed result |
|---|---|
| `cargo test --locked --offline --workspace --all-features -- --test-threads=1` | **Exit 0; 387 passed, 0 failed, 52 ignored; 841.31s.** No filtered tests. Independently counted every individual ok/ignored line and all 24 target summaries, also cross-checking the implementer's complete saved log. |
| `cargo build --locked --offline --workspace --features ui --bins -j 4` | **Exit 0**; repeated after scratch diagnostics. |
| `cargo clippy --locked --offline --workspace --all-targets --all-features -- -D warnings` | **Exit 0**; repeated after scratch diagnostics. |
| `cargo test --locked --offline --workspace --no-default-features --no-run` | **Exit 0**, compilation only. |
| `cargo test --locked --offline -p regressee-app --no-default-features --features ui --lib --bin regressee --no-run` | **Exit 0**, compilation only. |
| `cargo test --locked --offline -p regressee-app --no-default-features --features ui-tests --lib --bin regressee --no-run` | **Exit 0**, compilation only. |
| `cargo fmt --all -- --check` | **Exit 0**. |
| `git diff --check` | **Exit 0**; final report checked separately as an untracked addition. |
| `cargo test --locked --offline -p regressee-app --all-features --bin regressee phase5_state_tests:: -- --test-threads=1` | **4 passed**, exit 0, 16.59s; repeated after restoration: **4 passed**, 16.47s. These are included in, not added to, the 387. |
| `cargo test --locked --offline -p regressee-app --all-features --lib --bin regressee --no-run` | **Exit 0** after scratch probe; restores repository-source targets. |

Exact full-workspace target accounting:

| Target | Passed / failed / ignored |
|---|---|
| App library / native binary | 98 / 0 / 2; 148 / 0 / 23 |
| `analysis_review` / `product_flow` / `qualification` | 7 / 0 / 0; 1 / 0 / 13; 1 / 0 / 0 |
| Core / Runtime library / Runtime distribution | 52 / 0 / 0; 28 / 0 / 1; 6 / 0 / 0 |
| Runtime Linux ownership / Spike / Storage | 0 / 0 / 3; 11 / 0 / 0; 35 / 0 / 2 |
| Explicit website/runtime/native fixture targets | 0 / 0 / 8 |
| Remaining binaries/doc tests | 0 / 0 / 0 |
| **Total** | **387 / 0 / 52** |

The normal persisted **100/1,000-page selective approval/privacy/retention** test
actually completed successfully (**695.84s**). It is not the ignored real-browser
1,000-page soak. Incumbent consent/draft/revision/mask/notes/history and privacy/
retention regressions are present in the completed gate. The implementer's opted-in
native recipe completed one test, independently hash/log/state-checked here; it
is separate from the 387. **Ignored tests are not passes.** The existing
`proc-macro-error2 v2.0.1` future-incompatibility notice remains; strict Clippy
introduced no warning.

### Durable hash anchors

All per-frame hashes remain in the checked repository receipt/ledger; complete
private gate receipts and logs remain intact. Selected independent anchors:

| Artifact | SHA-256 |
|---|---|
| Final source manifest | `015615093ada2cf92af725c6ad4a1729845a24a9d98e99a3db0e213b9d8ce4be` |
| Final state matrix | `24346367961cf3c388c579cb7214c5338a7bd1fa5aac1e758c255698e467abf5` |
| Implementer gate source manifest | `d924369b3ee60c50270ca58e775b3a65872d0e28bd19b797c2530ea73132acd3` |
| Independent source manifest | `63837b200c021bc55c5a3a27d08ed13181213fd3649b1b459188a9dcd89fecfd` |
| Independent full-workspace log | `2cbe2570e451f1ef87ae2569685781983f2046adc6672a5b2e9f3bf733ed75b7` |
| Independent gate receipt | `622651b0b2e70611ff5b9d2d380118cbd3ea130346ea43118d364a17a4bd2995` |
| Independent source/receipt/pixel-region check | `aca26520d27cbeb1ef854dd5aa74b71ec28cf82f831218d11924e744c636f4a5` |
| Successful automatic-reveal probe log | `03588b661c0065ddc72c57e8e22eaa80ef7b684518947c115f5673580875e07b` |

## Unchanged qualification boundary

This PASS verifies the complete requested refinement in isolated Linux GPUI/Blade
offscreen paint and simulated controller/input scope. It does **not** qualify real
OS presentation/keyboard/pointer/IME/clipboard/file-picker delivery, assistive
technology, scaling/high contrast, automatic OS motion preferences, Windows/macOS,
usability, live websites/login/enterprise networks, signed runtime distribution or
public release. The existing Linux AT-SPI registration and macOS process-ownership
capture gaps remain documented obligations, not new Phase 5 repair requests or
claimed passes. Browser markup detectors provide no Rust/GPUI coverage.

**Final disposition: PASS; no collective FIX batch required.** Parent alone owns
plan completion, final diff review, commit and ordinary push.
