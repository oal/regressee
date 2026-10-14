# Interface wording

Regressee is for non-developers reviewing website changes. Explain the outcome,
its consequence and the next available action, not the implementation.

## Rules

- Lead with what happened: “Baseline saved.” Not “Saved explicitly.”
- Name the object: screenshots, pages, comparison results, files or login.
  Avoid “evidence,” “targets” and “objects” in routine messages.
- Use the same labels in instructions as on the actual controls.
- Keep success brief. Explain consequences when baselines, sharing or deletion
  are involved. Errors explain what failed and how to recover.
- Say “choose Save” rather than “saving is explicit.” Explain where unsaved
  edits go without talking about bindings, revisions or authoritative replies.
- Describe missing screenshots and failed comparisons separately from unchanged
  pages. A visual difference is not a confirmed defect.
- Keep privacy and recovery warnings factual. Ignored areas remain visible in
  original screenshots, including ZIPs. Deletion cannot be undone.
- Exact IDs, pixel algorithms, source records and runtime diagnostics remain
  available in technical details. Do not rename persisted values or change
  eligibility checks to improve wording.
- Check full messages and action labels at both desktop window sizes. Do not
  shorten an action until its consequence becomes unclear.
- Check dynamic text with zero, one and multiple items. Use singular wording for
  one item and finish status sentences before appending the next instruction.
- Use **Close** for a panel that simply closes. Explain draft retention beside
  the editor; do not disguise closing as navigation or imply that it saves.
- Text fields must not expose UTF-8, byte budgets or storage accounting. Keep
  technical limits internal. Never relabel a byte limit as a character limit.
  A rejected edit says what to change and whether text was added or saved.
- Optional settings still need clear labels, examples and consequences. Being
  under “Advanced” is not permission to use internal implementation language.

## Terminology

| Concept | Interface wording |
| --- | --- |
| Baseline | Saved screenshots used for future comparisons |
| Check | Capture the same page list and compare against the baseline |
| Inventory | Page list |
| Target | Page and screen size, or planned screenshot |
| Active / retained baseline | Current baseline / saved or earlier version |
| Representative probe | Access check on sample pages |
| Pair-bound mask revision | Ignored areas saved for these before-and-after screenshots |
| Provenance disclosure | Screenshot source details or technical details |
| Unreferenced recovery object | Complete file not used by saved records; may contain a recoverable screenshot |

“Capture,” “baseline,” “sensitivity,” “environment,” and “ignored areas” remain
product terms. Explain them where users make the relevant choice; do not replace
them with multiple competing synonyms.
