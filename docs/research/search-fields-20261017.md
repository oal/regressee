# Search and list-filter fields — 2026-10-17

## Outcome and scope

The user rejected the queue's separate Search label and requested a consistent
app-wide search/filter-field review using Impeccable. This is a native Operate
refinement, not a redesign. The current neutral/teal themes and unrelated ongoing
UI work are preserved.

- Pages/results, loaded History and the timezone picker share a 16px native
  magnifier, 8px icon/text spacing and contextual placeholder, retaining the
  existing 40px input height, small corners, insets and accent focus.
- The queue no longer spends horizontal space on a separate Search label.
  History no longer repeats its prompt as a label; its loaded-page-only scope
  remains visible, including while a query is entered.
- Queue and History use the same compact search/clear row. Clear retains its
  existing named action and shell-owned keyboard position. Ordinary settings
  labels and non-text filter choices are intentionally unchanged.
- Placeholder text is shaped separately from the actual editable line, so it
  never becomes selected text, clipboard contents, an IME range or a stored value.
  The magnifier remains visible with a query, and clicking it focuses the input.
- The initial visual batch exposed an existing recovery problem at 820×600:
  no-match explanations could push the search field partly behind the review
  footer. The repair keeps recovery controls above a bounded, scrollable
  explanation. The changes-only notice says No matches when appropriate.

No capture, analysis, stored history, baseline, approval, draft, timezone-value,
dependency, asset, animation or palette behavior was changed. A pre-existing
theme-test compile error was repaired by comparing the solid Background value
directly instead of calling an unavailable `.color()` method; no theme behavior
was changed by that repair.

## Verification

- Final shared native UI component checks: **79 passed**, including two new search
  tests for presentation-only placeholders, Unicode/clipboard editing, icon
  focus, bounded long queries and horizontal caret scrolling.
- New queue/History tests: **2 passed** in both the initial and final run. They cover filtering,
  clear actions, evidence/draft preservation, unchanged saved evidence and layout
  at 1440×900, 1280×800 and 820×600. The final guard also checks that search,
  Clear and Filter are wholly inside the queue, not merely inside the window.
- Related pre-repair regressions: **37 passed, 1 ignored** across keyboard
  navigation, toolbar order, workflow refinement, options and History hierarchy.
- The existing filtered-empty recovery/evidence/draft regression passed again
  after the bounded-message repair.
- Initial native offscreen inspection: **28 PNGs**, both themes at three sizes,
  covering results, page discovery, loaded History and timezone search, plus
  focused no-match queries at the shortest size. All three contact sheets were
  inspected; the short-queue defect above was the only scoped visual correction.

- Final confirmation: **28 PNGs** passed and all three contact sheets were
  inspected. Search, Clear and Filter remain fully visible in the short no-match
  queue. No further visual correction round was needed or taken.
- Native production build and `git diff --check` passed. The final source receipt
  confirms all seven captured search/shell/theme source files match current bytes.
  No shipping raster or new design token was introduced.

**Scoped in-thread verdict: pass.** The search treatment is consistent across
the three search contexts; the reported side-label problem and the discovered
short-queue recovery defect are resolved. This is not independent whole-app or
native accessibility acceptance.

## Evidence and qualification limits

Private diagnostic root:
`/var/home/olav/Data/opencode/tmp/opencode/regressee-search-fields-p3h8vbpi/`.
The original batch remains in `inspection/`; the sole correction batch belongs
in `confirmation/`. Capture sidecars retain native renderer flags, PNG hashes and
source provenance; `search-manifest.json` records additional search-source hashes.
Fixture pixels are the existing synthetic offline website corpus, not the user's
project or a visited website. Temporary linker aliases stay in this private root;
existing project and system libraries were not overwritten.

These are Linux GPUI simulated-input/layout and actual offscreen rendering checks,
not OS-input, screen-reader, Windows/macOS, whole-app or release acceptance.
GPUI's native accessibility integration remains an existing qualification blocker:
a placeholder is not a programmatic accessible label, and no new screen-reader
support is claimed. Concurrent unrelated UI edits can invalidate whole-app source
freshness without invalidating the narrowly tested search implementation.
