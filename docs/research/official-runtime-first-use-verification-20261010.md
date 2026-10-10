# Independent official-runtime first-use verification — 2026-10-10

## Scope

One independent execution of the frozen compiled integration test:

```text
/path/to/Regressee/target/debug/deps/official_runtime_first_use-4c6e7c790ea00a08
  --ignored --exact fresh_default_discovery_installs_and_reopens_verified_browser
  --nocapture --test-threads=1
```

The process is bounded by `timeout --signal=TERM --kill-after=5s 1500s`, with the shell's background timeout explicitly disabled. It uses a new physical mode-0700 `regressee-official-independent-*` directory under the approved private temporary root. Only `TMPDIR` and `REGRESSEE_OFFICIAL_CHECK_DIR` are set for the child; no distribution/download override, browser/worker/GPU switch, sandbox bypass, system installation, or existing project is used.

The integration test starts `Jobs::start`, submits discovery against two owned synthetic pages on an ephemeral loopback HTTP server, and exercises default official HTTPS installation, the shared sandboxed worker screenshot smoke, discovery, two screenshot captures, repeated-runtime resolution, and reopened readiness. It asserts unchanged persisted project settings and runtime identity, an unchanged pointer across repeat `ensure_runtime`, and no repeat installation progress.

SHA-256 inventories of the exact binary, all files under `crates/` and `worker/` (including official catalogs/payload), and root Cargo/toolchain manifests are recorded before and after execution. Documentation is deliberately excluded. A read-only process observer records the owned timeout/test/worker descendant identities and checks for surviving matching process identities after completion; it never signals processes.

## Result

**PASS — one attempt, exit 0, 1 passed / 0 failed.** The test finished in **114.42 seconds**; the wrapper measured **114.422242127 seconds**. It started at `2026-10-10T21:52:43.324830Z` and its final metadata was written at `2026-10-10T21:54:37.791591Z`. The 1,500-second bound did not fire. No retry or alternative launch was performed.

Private evidence directory (retained, not committed):

```text
/path/to/private-tests/opencode/regressee-official-independent-_8cn2hk9
```

The completion log records, in order: the two official component downloads, archive verification/extraction, final inventory verification, **Testing the private browser**, setup finished, finding rendered links, starting the bounded browser pool, and **Capturing 2 pages**. The two archive transfers total **258,433,266 bytes**: Node **58,088,022 bytes** and full Chrome **200,345,244 bytes**, as pinned by the Linux catalog and enforced by the installer's exact received-size/checksum checks. The worker/library ZIP is embedded and is not another network transfer. The phase recorder deliberately collapses byte-progress updates; it is not an independently instrumented network byte counter.

The test's `official-first-use-result.json` confirms:

- **2 discovered owned pages; 2 captures**, with capture run state asserted `Completed`.
- **OfficialPinned** identity: Node `24.21.0`, Playwright `1.64.0`, Chromium `156.0.8078.4`, worker `5.0.0`.
- **Reopened Ready**, same verified runtime identity, and **no repeat download**.
- No persisted project-setting changes; the exact before/after serialized project equality assertion passed.
- Pointer preservation across repeat `ensure_runtime`; its byte-equality assertion passed with no installation progress. At completion, the cache contained only `current` and one generation, `runtime-d935a992-3578-4d6d-896f-123ac186853a`; `current` named that generation. Reopening did not create another generation. The test does not separately serialize a second post-reopen pointer snapshot, so this report does not invent one.
- **0 observed owned processes remaining** after normal RAII shutdown, including no surviving sampled PID/start-time identities. No cleanup signals or deletion commands were issued by the observer.
- **127 bound binary/source/manifest files unchanged**, with no added/removed/mismatched paths between the before/after inventories.

Executed binary SHA-256:

```text
12347febcacec6162501892fe0ce8bd28c7092695a4e2e0c9984b70394aaae13
```

Retained authorities are `first-use.log`, `official-first-use-result.json`, `independent-execution.json`, `source-binding-before.json`, and `source-binding-after.json`. The installed runtime, synthetic project database, and screenshot artifacts remain under `first-use-store/`. The observer is bounded process-settlement evidence, not a new guardian/sandbox qualification suite.

## Visual review boundary

**PASS for the bounded sampled-render regression review; not universal UI acceptance.** After the explicit completed-batch handoff, I personally inspected all **36 application PNGs** (18 states at **1120×800** and **820×600**) and the renderer calibration image once, in one review pass. No recapture, native/headed/desktop helper, browser rerun, source repair, or build was performed. The parent's `ui-capture.log` independently reads **1 passed / 0 failed**, **11.68 seconds**.

Image directory:

```text
/path/to/private-tests/opencode/regressee-official-verification-r7ki95gq/final-layout
```

Reviewed pairs: `projects`, `setup`, `setup-screenshots`, `setup-images`, `setup-speed`, `setup-advanced`, `setup-environment`, `setup-mapping`, `setup-timezone`, `timezone-popup`, `new-project-feedback`, `results`, `results-viewer`, `results-analysis`, `history`, `history-lower`, `settings`, and `settings-lower`. Each pair is the corresponding `1120x800-*.png` / `820x600-*.png`.

| Focus | Independent observation and limit |
| --- | --- |
| Timezone closed trigger | `UTC` is legible and the chevron sits at the right edge at both sizes; no value/arrow collision. |
| Timezone popup | Search field, current selection, and list are legible and stay within the window at both sizes. The minimum-size scroll list ends near the window bottom and overlays the pinned footer while open; its partially visible last row is the list viewport boundary, not evidence of horizontal overflow. |
| Login help / next action | In the wide timezone image, the login instruction is visibly above and separated from **Capture details and limitations**, with neither collapsed help nor text/action overlap. At 820×600 this help/action pair is below the captured viewport: **minimum-size help rendering is not visually verified**. The source-bound capture test asserts value/chevron containment and at least an 8-pixel help/action separation at both sizes; the completed full-Shell unit log also reports `setup_help_and_timezone_are_contained_in_the_scrolling_form ... ok`. These are geometry assertions, not OS interaction. |
| New-project feedback | Both failure banners wrap their instructions within the workspace; Dismiss remains attached. Project setup and the pinned Save settings action remain recognizable, with no banner/heading overlap. |
| Projects / setup | Headings, selections, fields, and primary actions are readable. Form sections and wrapped choice help stay within the workspace at both sizes. Scrolled or below-fold controls are not asserted visible merely because another image shows them. |
| Results / viewer / analysis | The two original-image panels remain side-by-side within the workspace at both sizes; fixture image content is synthetic. Analysis controls and explanatory text wrap without right-edge escape or mutual overlap. Scroll-boundary clipping does not prove complete below-fold coverage or native scrolling. |
| History / lower History | Storage summaries, provenance explanations, and destructive actions remain distinct; action groups wrap at minimum size without horizontal overflow. |
| Settings / lower Settings | Network fields, helper text, and action groups are readable and contained. **These screenshots intentionally show an unconfigured/no-runtime UI fixture**, not the default official runtime's live Ready state. The fresh-browser test supplies that separate result. |

No new P0/P1 visual blocker was found in these sampled renders. The wide fields are not demonstrated to use the intended 680-pixel maximum form width: several fill the available wider column. This review confirms containment, **not exact intended form-width compliance**, whole-state coverage, or accessibility conformance.

Capture binding was independently checked: all **36 application PNG hashes match their sidecars**; the UI test binary matches SHA-256 `1d237518505aab1ddc0c45af5a67f167632349165883badae2506025fc748011`. Of **298** capture-bound source inputs, **297** still match. The only mismatch is `crates/runtime/src/official.rs` (`c402549356dc77808f447db4d51ac932ce204754efc17f7071110f440284fffe` captured; `614ba5c4948e09129e744dd039f3b7ce0c47007cfdd11ef6a3b21134f098781a` current). The parent identifies this as the post-capture Clippy-only byte-character-array to `*b"LKxSg"` equivalence under `cfg(test)`; the current test-module location was inspected. No UI/production-source capture binding mismatch was found. This is separate from the live execution's **127 before/after matches**.

`independent-image-review-binding.json` in the retained independent evidence directory records all 37 PNG hashes and the capture/current source comparison. The calibration shows actual text, controls, selected radio styling, and color swatches, not an all-black image. This is **real GPUI Blade offscreen rendering with Linux fonts at 1× DPI**, not proof of an actual OS window, user input, accessibility, or headed login. Historical native zero-PNG/black-capture failures are not superseded by this offscreen pass. No third platform, public website, or human login was exercised.

## Qualification boundary

This is Linux x86_64 owned-fixture **OfficialPinned** first-use verification only. It cannot establish clean-machine dependency coverage, a **QualifiedPublic** runtime release, other-platform support, public-site correctness, or human login/session behavior. No build, Cargo, formatter, runtime source, application/UI source, or asset changes are part of this execution task.
