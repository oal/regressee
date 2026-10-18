// SPDX-License-Identifier: MPL-2.0
//! Focused source reads. Bodies never belong in aggregate evidence DTOs.
use anyhow::{Result, ensure};
use regressee_core::dom_snapshot::{DomDiffFallback, DomDiffRow, DomTextDiff};
use serde::{Deserialize, Serialize};

pub const MAX_SOURCE_PAGE_BYTES: usize = 256 * 1024;
pub const MAX_DIFF_PAGE_ROWS: usize = 1000;
pub const MAX_RESULT_PAGE_BYTES: usize = 4 * 1024 * 1024;

/// Count encoded JSON without allocating an oversized escaped output buffer.
pub(crate) fn json_size_within(value: &impl Serialize, limit: usize) -> Option<usize> {
    struct Counter {
        bytes: usize,
        limit: usize,
    }
    impl std::io::Write for Counter {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            if bytes.len() > self.limit.saturating_sub(self.bytes) {
                return Err(std::io::Error::other("JSON response budget exceeded"));
            }
            self.bytes += bytes.len();
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let mut counter = Counter { bytes: 0, limit };
    serde_json::to_writer(&mut counter, value).ok()?;
    Some(counter.bytes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffPage {
    /// Hunk headers and line-count summaries describe the complete bounded diff;
    /// rows are the requested window of that diff's emitted rows.
    pub summary: DiffSummary,
    pub hunks: Vec<HunkPage>,
    pub offset: usize,
    pub next_offset: Option<usize>,
    pub total_rows: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffSummary {
    pub changed: bool,
    pub old_line_count: u32,
    pub new_line_count: u32,
    pub added_lines: u32,
    pub removed_lines: u32,
    pub fallback: Option<DomDiffFallback>,
    pub display_limited: bool,
    pub total_rows: usize,
}
impl From<&DomTextDiff> for DiffSummary {
    fn from(diff: &DomTextDiff) -> Self {
        Self {
            changed: diff.changed,
            old_line_count: diff.old_line_count,
            new_line_count: diff.new_line_count,
            added_lines: diff.added_lines,
            removed_lines: diff.removed_lines,
            fallback: diff.fallback,
            display_limited: diff.display_limited,
            total_rows: diff.hunks.iter().map(|h| h.rows.len()).sum(),
        }
    }
}
/// A window, not a mutated canonical hunk. Original headers stay immutable.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HunkPage {
    pub old_start: u32,
    pub old_lines: u32,
    pub new_start: u32,
    pub new_lines: u32,
    pub row_offset: usize,
    pub total_rows: usize,
    pub rows: Vec<DomDiffRow>,
}

pub fn diff_page(diff: DomTextDiff, offset: usize, limit: usize) -> Result<DiffPage> {
    diff.validate()?;
    ensure!(
        (1..=MAX_DIFF_PAGE_ROWS).contains(&limit),
        "invalid_request: diff page row limit must be 1–1000"
    );
    let total_rows = diff.hunks.iter().map(|h| h.rows.len()).sum::<usize>();
    ensure!(
        offset <= total_rows,
        "invalid_request: diff offset exceeds emitted rows"
    );
    let end = offset.saturating_add(limit).min(total_rows);
    let mut start = 0;
    let summary = DiffSummary::from(&diff);
    let hunks = diff
        .hunks
        .into_iter()
        .filter_map(|hunk| {
            let hunk_start = start;
            start += hunk.rows.len();
            let row_offset = offset.saturating_sub(hunk_start).min(hunk.rows.len());
            let row_end = end.saturating_sub(hunk_start).min(hunk.rows.len());
            (row_offset < row_end).then(|| HunkPage {
                old_start: hunk.old_start,
                old_lines: hunk.old_lines,
                new_start: hunk.new_start,
                new_lines: hunk.new_lines,
                row_offset,
                total_rows: hunk.rows.len(),
                rows: hunk
                    .rows
                    .into_iter()
                    .skip(row_offset)
                    .take(row_end - row_offset)
                    .collect(),
            })
        })
        .collect();
    Ok(DiffPage {
        summary,
        hunks,
        offset,
        next_offset: (end < total_rows).then_some(end),
        total_rows,
    })
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SourcePage {
    /// UTF-8 byte offsets, not character or line indexes.
    pub offset: usize,
    pub next_offset: Option<usize>,
    pub total_bytes: usize,
    pub text: String,
}

pub fn source_page(text: &str, offset: usize, limit: usize) -> Result<SourcePage> {
    ensure!(
        (4..=MAX_SOURCE_PAGE_BYTES).contains(&limit),
        "invalid_request: source page byte limit must be 4–262144"
    );
    ensure!(
        offset <= text.len() && text.is_char_boundary(offset),
        "invalid_request: source offset must be a valid UTF-8 byte boundary"
    );
    let mut end = offset.saturating_add(limit).min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    Ok(SourcePage {
        offset,
        next_offset: (end < text.len()).then_some(end),
        total_bytes: text.len(),
        text: text[offset..end].to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn encoded_budget_counts_escaping_without_building_an_oversized_frame() {
        let text = "\0\n\"<&>".repeat(1024);
        let expected = serde_json::to_vec(&text).unwrap().len();
        assert_eq!(json_size_within(&text, expected), Some(expected));
        assert_eq!(json_size_within(&text, expected - 1), None);
    }

    #[test]
    fn source_paging_is_lossless_bounded_and_utf8_safe() -> Result<()> {
        let source = "a🦀é\n</pre></script><img src=x onerror=evil()>\r\n";
        let mut rebuilt = String::new();
        let mut offset = 0;
        loop {
            let page = source_page(source, offset, 5)?;
            assert!(page.text.len() <= 5);
            assert_eq!(page.total_bytes, source.len());
            rebuilt.push_str(&page.text);
            if let Some(next) = page.next_offset {
                assert!(next > offset);
                offset = next;
            } else {
                break;
            }
        }
        assert_eq!(rebuilt, source);
        assert!(source_page(source, 2, 5).is_err());
        assert!(source_page(source, source.len() + 1, 5).is_err());
        for limit in [0, 3, MAX_SOURCE_PAGE_BYTES + 1, usize::MAX] {
            assert!(source_page(source, 0, limit).is_err());
        }
        assert!(source_page(source, source.len(), 4)?.text.is_empty());
        assert!(source_page("", 0, 4)?.next_offset.is_none());
        Ok(())
    }

    #[test]
    fn diff_pages_preserve_verdict_counts_and_numbered_rows() -> Result<()> {
        use regressee_core::dom_snapshot::{DomDiffOptions, diff_text};
        let diff = diff_text("a\nb\nc\n", "a\nx\nc\n", &DomDiffOptions::default(), || {
            false
        })?;
        let all = diff
            .hunks
            .iter()
            .flat_map(|h| h.rows.clone())
            .collect::<Vec<_>>();
        let mut rebuilt = Vec::new();
        for offset in 0..all.len() {
            let page = diff_page(diff.clone(), offset, 1)?;
            assert_eq!(page.total_rows, all.len());
            assert_eq!(page.summary.changed, diff.changed);
            assert_eq!(page.summary.added_lines, diff.added_lines);
            assert_eq!(page.summary.removed_lines, diff.removed_lines);
            rebuilt.extend(page.hunks.into_iter().flat_map(|h| h.rows));
        }
        assert_eq!(rebuilt, all);
        assert!(diff_page(diff.clone(), all.len() + 1, 1).is_err());
        assert!(diff_page(diff.clone(), 0, MAX_DIFF_PAGE_ROWS + 1).is_err());
        assert!(diff_page(diff, 0, 0).is_err());
        Ok(())
    }
}
