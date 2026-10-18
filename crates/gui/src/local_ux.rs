// SPDX-License-Identifier: MPL-2.0
//! Cheap local queue indexing. Rebuild metadata only when evidence changes.
use regressee_core::dom_snapshot::DomComparisonOutcome;
use regressee_core::{domain::*, evidence::ProjectEvidence};

type SearchablePages = (InventoryId, usize, Vec<(PageId, String)>);

/// Reanalysis/refresh keeps an explicit filter; a different run (or its first
/// completed analysis) starts with changes. Capture-only evidence stays unfiltered.
pub(super) fn result_filter_scope(evidence: &ProjectEvidence) -> Option<(RunId, bool)> {
    evidence.latest.as_ref().map(|run| {
        (
            run.run.id,
            evidence
                .comparison
                .as_ref()
                .is_some_and(|a| a.revision.run == run.run.id)
                || evidence
                    .dom_comparison
                    .as_ref()
                    .is_some_and(|a| a.revision.run == run.run.id),
        )
    })
}

#[derive(Default)]
pub(super) struct HistoryFilterCache {
    pub query: String,
    pub kind: usize,
    pub identity: (usize, usize, usize, usize),
    pub baselines: Vec<usize>,
    pub runs: Vec<usize>,
}

#[derive(Default)]
pub(super) struct QueueIndex {
    pub preview: Option<SearchablePages>,
    revision: Option<QueueRevision>,
    searchable: Vec<String>,
    kinds: Vec<u8>,
    order: Vec<usize>,
    grouped: Vec<usize>,
}
#[derive(Clone, Copy, PartialEq, Eq)]
struct QueueRevision {
    run: RunId,
    screenshot: Option<AnalysisId>,
    dom: Option<DomAnalysisId>,
    targets: usize,
    target_pointer: usize,
    dom_pointer: usize,
    unreadable: [bool; 2],
}

// Changed and missing are independent flags: a changed screenshot + failed DOM
// belongs in both filters. Unchanged means every requested kind is unchanged.
fn aggregate_status(statuses: &[u8]) -> u8 {
    let flags = statuses.iter().copied().fold(0, |a, b| a | b);
    (flags & 3)
        | if !statuses.is_empty() && statuses.iter().all(|&s| s == 4) {
            4
        } else {
            flags & 8
        }
}
fn status_matches(flags: u8, filter: usize) -> bool {
    match filter {
        0 => true,
        1 => flags & 1 != 0,
        2 => flags & 2 != 0,
        3 => flags == 4,
        _ => false,
    }
}
impl QueueIndex {
    pub fn results(
        &mut self,
        evidence: &ProjectEvidence,
        query: &str,
        kind: usize,
        grouped: bool,
        unreadable: [bool; 2],
    ) -> Vec<usize> {
        let Some(run) = &evidence.latest else {
            return vec![];
        };
        let analysis = evidence
            .comparison
            .as_ref()
            .filter(|a| a.revision.run == run.run.id);
        let dom = evidence
            .dom_comparison
            .as_ref()
            .filter(|a| a.revision.run == run.run.id);
        let revision = QueueRevision {
            run: run.run.id,
            screenshot: analysis.map(|a| a.revision.id),
            dom: dom.map(|a| a.revision.id),
            targets: run.run.targets.len(),
            target_pointer: run.run.targets.as_ptr() as usize,
            dom_pointer: run.dom_captures.as_ptr() as usize,
            unreadable,
        };
        if self.revision != Some(revision) {
            let classifications: std::collections::BTreeMap<_, _> = analysis
                .map(|a| a.review_classifications())
                .unwrap_or_default();
            let dom_outcomes: std::collections::BTreeMap<_, _> = dom
                .into_iter()
                .flat_map(|d| d.revision.results.iter())
                .map(|r| (&r.key, &r.outcome))
                .collect();
            let screenshots: std::collections::BTreeSet<_> =
                run.all_captures().map(|c| &c.metadata.key).collect();
            let dom_captures: std::collections::BTreeSet<_> =
                run.dom_captures.iter().map(|c| &c.metadata.key).collect();
            self.searchable = run
                .run
                .targets
                .iter()
                .map(|t| t.requested.as_str().to_lowercase())
                .collect();
            self.kinds = run
                .run
                .targets
                .iter()
                .map(|t| {
                    let mut statuses = Vec::with_capacity(2);
                    if run.run.capture_plan.mode.screenshots() {
                        statuses.push(
                            if unreadable[0]
                                || !matches!(t.outcome, TargetOutcome::Captured { .. })
                                || !screenshots.contains(&t.key)
                            {
                                2
                            } else {
                                match classifications.get(&t.key) {
                                    Some(Classification::Changed) => 1,
                                    Some(
                                        Classification::NotComparable
                                        | Classification::CouldNotCapture,
                                    ) => 2,
                                    Some(Classification::NoSignificantChange) => 4,
                                    _ => 8,
                                }
                            },
                        );
                    }
                    if run.run.capture_plan.mode.dom_snapshots() {
                        statuses.push(
                            if unreadable[1]
                                || !matches!(t.dom_outcome, DomTargetOutcome::Captured { .. })
                                || !dom_captures.contains(&t.key)
                            {
                                2
                            } else {
                                match dom_outcomes.get(&t.key) {
                                    Some(DomComparisonOutcome::Changed) => 1,
                                    Some(DomComparisonOutcome::Unchanged) => 4,
                                    Some(
                                        DomComparisonOutcome::NotComparable { .. }
                                        | DomComparisonOutcome::CouldNotCapture { .. }
                                        | DomComparisonOutcome::NotRequested,
                                    ) => 2,
                                    None => 8,
                                }
                            },
                        );
                    }
                    aggregate_status(&statuses)
                })
                .collect();
            let rank = |i: usize| {
                if self.kinds[i] & 1 != 0 {
                    0
                } else if self.kinds[i] & 2 != 0 {
                    1
                } else if self.kinds[i] == 4 {
                    3
                } else {
                    2
                }
            };
            self.order = (0..run.run.targets.len()).collect();
            self.order.sort_by(|a, b| {
                (rank(*a), run.run.targets[*a].requested.as_str())
                    .cmp(&(rank(*b), run.run.targets[*b].requested.as_str()))
            });
            self.grouped = self.order.clone();
            self.grouped.sort_by(|a, b| {
                (
                    run.run.targets[*a].key.profile,
                    rank(*a),
                    run.run.targets[*a].requested.as_str(),
                )
                    .cmp(&(
                        run.run.targets[*b].key.profile,
                        rank(*b),
                        run.run.targets[*b].requested.as_str(),
                    ))
            });
            self.revision = Some(revision);
        }
        let order = if grouped { &self.grouped } else { &self.order };
        order
            .iter()
            .copied()
            .filter(|i| status_matches(self.kinds[*i], kind) && self.searchable[*i].contains(query))
            .collect()
    }
}

pub(super) fn adjacent_visible<T: PartialEq>(
    rows: &[T],
    selected: Option<&T>,
    previous: bool,
) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }
    match selected.and_then(|selected| rows.iter().position(|row| row == selected)) {
        Some(i) if previous => Some(i.saturating_sub(1)),
        Some(i) => Some((i + 1).min(rows.len() - 1)),
        None if previous => Some(rows.len() - 1),
        None => Some(0),
    }
}

pub(super) fn integer(value: &str, min: u32, max: u32, optional: bool) -> bool {
    (optional && value.trim().is_empty())
        || value
            .trim()
            .parse::<u32>()
            .is_ok_and(|n| (min..=max).contains(&n))
}
pub(super) fn name(value: &str) -> Option<&'static str> {
    regressee_core::domain::validate_label(value.trim(), 120)
        .err()
        .map(|_| "Enter a name of 1–120 characters.")
}
pub(super) fn addresses(value: &str) -> Option<&'static str> {
    regressee_core::crawl::sources(value.trim())
        .err()
        .map(|_| ADDRESS_CORRECTION)
}
pub(super) const ADDRESS_CORRECTION: &str =
    "Use full http:// or https:// addresses, separated by spaces, without passwords.";
const MAPPING_CORRECTION: &str = "Use a complete website or section address, such as https://example.com/app. Leave out endings starting with ? or #.";
const PROXY_CORRECTION: &str = "Use the proxy's server address, such as http://proxy.example:3128. Leave out page paths, endings starting with ? or #, and login details.";
pub(super) fn optional_addresses(value: &str) -> Option<&'static str> {
    if value.trim().is_empty() {
        None
    } else {
        addresses(value)
    }
}
pub(super) fn optional_address(value: &str) -> Option<&'static str> {
    if value.trim().is_empty() {
        return None;
    }
    url::Url::parse(value.trim())
        .ok()
        .filter(|u| regressee_core::domain::validate_address(u).is_ok())
        .is_none()
        .then_some("Use one complete http:// or https:// address without an embedded password.")
}
pub(super) fn mapping_base(value: &str) -> Option<&'static str> {
    if value.trim().is_empty() {
        return None;
    }
    if let Some(error) = optional_address(value) {
        return Some(error);
    }
    url::Url::parse(value.trim())
        .ok()
        .is_none_or(|u| u.query().is_some() || u.fragment().is_some())
        .then_some(MAPPING_CORRECTION)
}
pub(super) fn proxy_origin(value: &str) -> Option<&'static str> {
    if value.trim().is_empty() {
        return None;
    }
    if let Some(error) = optional_address(value) {
        return Some(error);
    }
    url::Url::parse(value.trim())
        .ok()
        .is_none_or(|u| u.path() != "/" || u.query().is_some() || u.fragment().is_some())
        .then_some(PROXY_CORRECTION)
}
/// Known longest correction for each form control, including optional fields
/// whose empty value is valid and therefore cannot supply reservation text.
pub(super) fn crawl_correction_reserve(index: usize) -> Option<&'static str> {
    match index {
        10 | 11 => optional_address("invalid"),
        12 => Some(ADDRESS_CORRECTION),
        13 | 14 => Some(MAPPING_CORRECTION),
        17 => Some(PROXY_CORRECTION),
        _ => crawl_validator(index).and_then(|validate| validate("invalid")),
    }
}
pub(super) fn crawl_validator(index: usize) -> Option<fn(&str) -> Option<&'static str>> {
    Some(match index {
        0 => |s| (!integer(s, 1, 10_000, false)).then_some("Enter a page limit from 1 to 10,000."),
        3 => |s| {
            (!integer(s, 0, 100, true))
                .then_some("Use 0–100 for depth, or leave blank for automatic.")
        },
        4 => |s| {
            (!integer(s, 1, 8, true))
                .then_some("Use 1–8 pages at once, or leave blank for the speed preset.")
        },
        5 => |s| (!integer(s, 1, 8, false)).then_some("Use 1–8 pages at once per website."),
        6 => |s| (!integer(s, 0, 3, false)).then_some("Use 0–3 retries."),
        7 => |s| {
            (!integer(s, 1, 4096, false))
                .then_some("Use a whole-number width from 1 to 4,096 pixels.")
        },
        8 => |s| {
            (!integer(s, 1, 8192, false))
                .then_some("Use a whole-number height from 1 to 8,192 pixels.")
        },
        10 | 11 => optional_address,
        13 | 14 => mapping_base,
        17 => proxy_origin,
        12 => optional_addresses,
        _ => return None,
    })
}
pub(super) fn analysis_validator(index: usize) -> Option<fn(&str) -> Option<&'static str>> {
    fn percent(s: &str) -> Option<&'static str> {
        (!s.trim()
            .parse::<f64>()
            .is_ok_and(|n| n.is_finite() && (0.0..=100.0).contains(&n)))
        .then_some("Use a number from 0 to 100.")
    }
    fn optional_percent(s: &str) -> Option<&'static str> {
        if s.trim().is_empty() {
            None
        } else {
            percent(s)
        }
    }
    Some(match index {
        0 | 8 => percent,
        1 | 2 => optional_percent,
        3 | 4 => |s| {
            (!integer(s, 1, 65_536, false)).then_some("Use a whole-number pixel count from 1 to 65,536. It must fit within the small-area size.")
        },
        5 | 6 => |s| {
            (!integer(s, 1, 256, false)).then_some("Use 1–256 pixels. Spacing must be no more than the small-area size and no less than one quarter of it, rounded up.")
        },
        7 => |s| {
            (!integer(s, 1, 64_000_000, true))
                .then_some("Use 1–64,000,000 pixels, or leave blank to disable this threshold.")
        },
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn independent_kind_filters_do_not_hide_changes_or_failures() {
        let mixed = aggregate_status(&[1, 2]);
        assert!(status_matches(mixed, 1));
        assert!(status_matches(mixed, 2));
        assert!(!status_matches(mixed, 3));
        assert_eq!(aggregate_status(&[4, 1]), 1);
        assert_eq!(aggregate_status(&[4, 4]), 4);
        assert!(!status_matches(aggregate_status(&[4, 8]), 3));
        assert!(!status_matches(aggregate_status(&[]), 3));
    }
    #[test]
    fn hidden_selection_enters_visible_queue_without_skipping_first_match() {
        let rows: Vec<_> = (0..3000).collect();
        assert_eq!(adjacent_visible(&rows, Some(&4000), false), Some(0));
        assert_eq!(adjacent_visible(&rows, Some(&4000), true), Some(2999));
        assert_eq!(adjacent_visible(&rows, Some(&1200), false), Some(1201));
        assert_eq!(adjacent_visible::<usize>(&[], None, false), None);
    }
    #[test]
    fn local_numeric_ranges_reject_incomplete_and_out_of_range_values() {
        assert!(!integer("", 1, 10_000, false));
        assert!(integer("", 0, 100, true));
        assert!(!integer("10001", 1, 10_000, false));
        assert!(integer("8192", 1, 8192, false));
        assert!(!integer("1.5", 1, 8192, false));
    }
}
