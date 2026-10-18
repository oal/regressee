// SPDX-License-Identifier: MPL-2.0
//! Display-only adaptation of the shared core diff. Run off the UI thread.
//! Complete formatted originals must come from verified engine text reads; this
//! module never opens storage, normalizes DOM, or computes another edit script.
use crate::ui::dom_diff::{
    DomDiffDocument, DomDiffError, DomDiffHunk, DomDiffMode, DomDiffRow, DomDiffRowKind,
};
use regressee_core::dom_snapshot::{DomDiffFallback, DomTextDiff};
use std::sync::Arc;

fn invalid() -> DomDiffError {
    DomDiffError::new(
        "The DOM source does not match this saved comparison.",
        "Refresh the saved result before reviewing it. The baseline has not been changed.",
    )
}

pub fn original(
    source: &str,
    before: bool,
    notice: Option<Arc<str>>,
) -> Result<DomDiffDocument, DomDiffError> {
    let rows = source
        .split_inclusive('\n')
        .enumerate()
        .map(|(index, line)| DomDiffRow {
            kind: DomDiffRowKind::Context,
            old_line: before.then_some(index as u32 + 1),
            new_line: (!before).then_some(index as u32 + 1),
            text: line.strip_suffix('\n').unwrap_or(line).into(),
        })
        .collect();
    DomDiffDocument::new(
        if before {
            DomDiffMode::BeforeOnly
        } else {
            DomDiffMode::AfterOnly
        },
        rows,
        vec![],
        notice,
    )
}

pub fn comparison(
    before: &str,
    after: &str,
    diff: &DomTextDiff,
) -> Result<DomDiffDocument, DomDiffError> {
    diff.validate().map_err(|_| invalid())?;
    let old: Vec<_> = before.split_inclusive('\n').collect();
    let new: Vec<_> = after.split_inclusive('\n').collect();
    if old.len() != diff.old_line_count as usize
        || new.len() != diff.new_line_count as usize
        || diff.changed != (before != after)
    {
        return Err(invalid());
    }
    let mut rows = Vec::new();
    let mut hunks = Vec::new();
    let (mut old_next, mut new_next) = (1u32, 1u32);
    for hunk in &diff.hunks {
        let old_start = hunk
            .rows
            .iter()
            .find_map(|r| r.old_line)
            .unwrap_or(hunk.old_start.saturating_add(1));
        let new_start = hunk
            .rows
            .iter()
            .find_map(|r| r.new_line)
            .unwrap_or(hunk.new_start.saturating_add(1));
        append_context(
            &old,
            &new,
            &mut old_next,
            &mut new_next,
            old_start,
            new_start,
            &mut rows,
        )?;
        let start = rows.len();
        for row in &hunk.rows {
            for (number, source) in [(row.old_line, &old), (row.new_line, &new)] {
                if let Some(number) = number {
                    let original = source.get(number as usize - 1).ok_or_else(invalid)?;
                    if original.strip_suffix('\n').unwrap_or(original) != row.text
                        || original.ends_with('\n') != row.has_newline
                    {
                        return Err(invalid());
                    }
                }
            }
            if let Some(number) = row.old_line {
                if number != old_next {
                    return Err(invalid());
                }
                old_next += 1;
            }
            if let Some(number) = row.new_line {
                if number != new_next {
                    return Err(invalid());
                }
                new_next += 1;
            }
            rows.push(DomDiffRow {
                kind: match row.kind {
                    regressee_core::dom_snapshot::DomDiffRowKind::Context => {
                        DomDiffRowKind::Context
                    }
                    regressee_core::dom_snapshot::DomDiffRowKind::Added => DomDiffRowKind::Added,
                    regressee_core::dom_snapshot::DomDiffRowKind::Removed => {
                        DomDiffRowKind::Removed
                    }
                },
                old_line: row.old_line,
                new_line: row.new_line,
                text: row.text.as_str().into(),
            });
        }
        if rows[start..]
            .iter()
            .any(|r| r.kind != DomDiffRowKind::Context)
        {
            hunks.push(DomDiffHunk {
                rows: start..rows.len(),
            });
        }
    }
    if !diff.display_limited {
        append_context(
            &old,
            &new,
            &mut old_next,
            &mut new_next,
            diff.old_line_count + 1,
            diff.new_line_count + 1,
            &mut rows,
        )?;
    }
    let notice = diff.fallback.map(|fallback| Arc::<str>::from(match fallback {
        DomDiffFallback::WorkBudget => "The shared diff reached its work limit and uses a coarse line replacement. Complete Before and After source remain available.",
        DomDiffFallback::InputLineLimit => "Detailed diff exceeds the line limit. The recorded changed result is retained; inspect complete Before and After source.",
        DomDiffFallback::OutputLimit => "Only part of this diff is displayed. The recorded changed result is retained; inspect complete Before and After source.",
        DomDiffFallback::Cancelled => "Detailed diff was interrupted. The recorded changed result is retained; inspect complete Before and After source.",
    })).map(|notice| if diff.display_limited { Arc::<str>::from(format!("{notice} Counts for the shared comparison: {} added lines · {} removed lines. Search covers only the displayed diff; choose Before or After source to search a complete original.", diff.added_lines, diff.removed_lines)) } else { notice });
    DomDiffDocument::new(
        if diff.changed {
            DomDiffMode::Changed
        } else {
            DomDiffMode::Unchanged
        },
        rows,
        hunks,
        notice,
    )
}

#[allow(clippy::too_many_arguments)]
fn append_context(
    old: &[&str],
    new: &[&str],
    old_next: &mut u32,
    new_next: &mut u32,
    old_end: u32,
    new_end: u32,
    rows: &mut Vec<DomDiffRow>,
) -> Result<(), DomDiffError> {
    if old_end < *old_next || new_end < *new_next || old_end - *old_next != new_end - *new_next {
        return Err(invalid());
    }
    while *old_next < old_end {
        let left = old.get(*old_next as usize - 1).ok_or_else(invalid)?;
        let right = new.get(*new_next as usize - 1).ok_or_else(invalid)?;
        if left != right {
            return Err(invalid());
        }
        rows.push(DomDiffRow {
            kind: DomDiffRowKind::Context,
            old_line: Some(*old_next),
            new_line: Some(*new_next),
            text: left.strip_suffix('\n').unwrap_or(left).into(),
        });
        *old_next += 1;
        *new_next += 1;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use regressee_core::dom_snapshot::{DomDiffOptions, diff_text};
    #[test]
    fn shared_hunks_restore_collapsed_context_without_a_second_diff() {
        let before = (0..100).map(|i| format!("line {i}\n")).collect::<String>();
        let after = before.replace("line 50\n", "changed 🦀\n");
        let diff = diff_text(&before, &after, &DomDiffOptions::default(), || false).unwrap();
        let document = comparison(&before, &after, &diff).unwrap();
        assert_eq!(document.mode(), DomDiffMode::Changed);
        assert_eq!(document.hunks().len(), diff.hunks.len());
        assert!(document.rows().iter().any(|row| row.old_line == Some(1)));
        assert!(document.rows().iter().any(|row| row.old_line == Some(100)));
        assert!(comparison(&before, &before, &diff).is_err());
    }
    #[test]
    fn empty_one_sided_and_display_limited_results_keep_the_verdict() {
        for (before, after) in [("", "one\n"), ("one\n", ""), ("", ""), ("same\n", "same\n")] {
            let diff = diff_text(before, after, &DomDiffOptions::default(), || false).unwrap();
            let document = comparison(before, after, &diff).unwrap();
            assert_eq!(document.mode() == DomDiffMode::Changed, before != after);
        }
        let diff = diff_text("before\n", "after\n", &DomDiffOptions::default(), || true).unwrap();
        assert_eq!(
            comparison("before\n", "after\n", &diff).unwrap().mode(),
            DomDiffMode::Changed
        );
    }

    #[test]
    fn shared_limits_and_context_only_prefixes_do_not_block_originals() {
        let samples = [
            "",
            "a\n",
            "a\nb\nc\n",
            "a\nb\nchanged 🦀\n",
            "x\na\nb\nc\n",
            "a\nb\n",
        ];
        for before in samples {
            for after in samples {
                for output in [0, 1, 2, 3, 100] {
                    let diff = diff_text(
                        before,
                        after,
                        &DomDiffOptions {
                            max_output_lines: output,
                            ..Default::default()
                        },
                        || false,
                    )
                    .unwrap();
                    let document = comparison(before, after, &diff).unwrap();
                    assert_eq!(document.mode() == DomDiffMode::Changed, before != after);
                    assert!(original(before, true, None).is_ok());
                    assert!(original(after, false, None).is_ok());
                }
            }
        }
    }
}
