// SPDX-License-Identifier: MPL-2.0
//! Native, read-only DOM source inspection. No HTML is interpreted or linked.
//!
//! # Integration contract
//!
//! Prepare `DomDiffDocument::new(mode, rows, hunks, notice)` **off the UI thread**.
//! Rows are the ordered unified source stream, without newline terminators. They
//! include unchanged source needed for expansion; a hunk's half-open row range
//! includes its surrounding context (normally three lines). Non-hunk context is
//! folded automatically. Numbers are one-based; an absent side has no number.
//! `Changed` is an authoritative verdict, not inferred from displayed row counts.
//! For a bounded/coarse backend diff, supply an honest `notice`; never relabel it
//! Unchanged. The shell must retain access to both verified complete originals.
//!
//! Create one `Entity<DomDiff>`. `begin_load(identity, cx)` immediately clears old
//! source/search/selection and returns a token. Only `finish_load(&token, result,
//! cx)` with the current token may publish a reply. The identity should include
//! target, kind and immutable analysis/artifact identity. Reuse the entity across
//! targets; do not call begin_load from every render. No I/O or diff algorithm is
//! performed here. `set_enabled` suspends all input during shell occlusion/locks;
//! `focus_handles(cx)` supplies mounted/enabled stops in painted order to the
//! shell's focus traversal. `focus_search` supports the shell's search shortcut.
//! Local bindings install once in `new`; no startup registration is necessary.
//!
//! Source wraps explicitly at 80 Unicode scalars, with a shared horizontal bar
//! for narrow panes. Only viewport rows (plus two) are shaped/painted. Selection
//! uses original UTF-8 byte boundaries, spans folded/offscreen context and copies
//! source only, without gutters or added/removed prefixes. Search is literal,
//! case-sensitive, includes folded source, and runs in cancellable background
//! work. Format-aware syntax is indexed on the document worker. Syntax, diff,
//! search and selection are independent paint layers with unchanged source bytes.
//! This component has no review, baseline, capture, image or file actions.

use std::{
    collections::BTreeSet,
    fmt,
    ops::Range,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
};

#[path = "dom_syntax.rs"]
mod syntax;

// This dependency-free model is also exercised by standalone rustc tests when
// the workspace's serialized Cargo slot is unavailable.
mod model {
    use super::syntax;
    use super::{Arc, AtomicBool, Ordering, Range, fmt};

    pub const WRAP_COLUMNS: usize = 80;
    const MAX_ROWS: usize = 250_000;
    const MAX_SOURCE_BYTES: usize = 16 * 1024 * 1024;
    pub const MAX_MATCHES: usize = 10_000;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DomDiffRowKind {
        Context,
        Added,
        Removed,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct DomDiffRow {
        pub kind: DomDiffRowKind,
        pub old_line: Option<u32>,
        pub new_line: Option<u32>,
        pub text: Arc<str>,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct DomDiffHunk {
        /// Half-open range in `rows`, including context. Sorted, disjoint.
        pub rows: Range<usize>,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub enum DomDiffMode {
        Changed,
        Unchanged,
        BeforeOnly,
        AfterOnly,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct DomDiffError {
        pub message: Arc<str>,
        pub recovery: Arc<str>,
    }

    impl DomDiffError {
        pub fn new(message: impl Into<Arc<str>>, recovery: impl Into<Arc<str>>) -> Self {
            Self {
                message: message.into(),
                recovery: recovery.into(),
            }
        }
    }
    impl fmt::Display for DomDiffError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{} {}", self.message, self.recovery)
        }
    }
    impl std::error::Error for DomDiffError {}

    fn invalid() -> DomDiffError {
        DomDiffError::new(
            "The DOM diff could not be displayed safely.",
            "Reload the saved result or inspect its complete captured source in Details.",
        )
    }

    #[derive(Clone, Debug)]
    pub(super) enum Block {
        Lines(Range<usize>),
        Fold(Range<usize>),
        Hunk(usize),
    }

    /// Immutable, bounded presentation data, not a persisted analysis contract.
    #[derive(Debug)]
    pub struct DomDiffDocument {
        pub(super) mode: DomDiffMode,
        pub(super) rows: Vec<DomDiffRow>,
        pub(super) hunks: Vec<DomDiffHunk>,
        pub(super) notice: Option<Arc<str>>,
        pub(super) blocks: Vec<Block>,
        pub(super) segments: Vec<(usize, Range<usize>)>,
        pub(super) row_segments: Vec<Range<usize>>,
        pub(super) added: usize,
        pub(super) removed: usize,
        pub(super) max_columns: usize,
        pub(super) number_digits: usize,
        pub(super) syntax: Vec<Vec<syntax::Span>>,
    }

    impl DomDiffDocument {
        /// Validate and index on a background worker. This does not compare text.
        /// Limits reject a presentation, never truncate evidence or alter verdicts.
        pub fn new(
            mode: DomDiffMode,
            rows: Vec<DomDiffRow>,
            hunks: Vec<DomDiffHunk>,
            notice: Option<Arc<str>>,
        ) -> Result<Self, DomDiffError> {
            if rows.len() > MAX_ROWS
                || rows
                    .iter()
                    .try_fold(0usize, |n, row| n.checked_add(row.text.len()))
                    .is_none_or(|bytes| bytes > MAX_SOURCE_BYTES)
                || notice.as_ref().is_some_and(|s| s.len() > 8192)
            {
                return Err(DomDiffError::new(
                    "This DOM diff exceeds the source viewer's display limit.",
                    "The saved comparison result has not been altered. Inspect the complete captured originals in Details.",
                ));
            }
            let mut added = 0;
            let mut removed = 0;
            let mut old = 0;
            let mut new = 0;
            for row in &rows {
                if row.text.contains(['\n', '\r'])
                    || row.old_line == Some(0)
                    || row.new_line == Some(0)
                    || row.old_line.is_some_and(|n| n <= old)
                    || row.new_line.is_some_and(|n| n <= new)
                {
                    return Err(invalid());
                }
                let sides_valid = match (mode, row.kind) {
                    (DomDiffMode::BeforeOnly, DomDiffRowKind::Context) => {
                        row.old_line.is_some() && row.new_line.is_none()
                    }
                    (DomDiffMode::AfterOnly, DomDiffRowKind::Context) => {
                        row.old_line.is_none() && row.new_line.is_some()
                    }
                    (DomDiffMode::Changed | DomDiffMode::Unchanged, DomDiffRowKind::Context) => {
                        row.old_line.is_some() && row.new_line.is_some()
                    }
                    (DomDiffMode::Changed, DomDiffRowKind::Added) => {
                        added += 1;
                        row.old_line.is_none() && row.new_line.is_some()
                    }
                    (DomDiffMode::Changed, DomDiffRowKind::Removed) => {
                        removed += 1;
                        row.old_line.is_some() && row.new_line.is_none()
                    }
                    _ => false,
                };
                if !sides_valid {
                    return Err(invalid());
                }
                old = row.old_line.unwrap_or(old);
                new = row.new_line.unwrap_or(new);
            }
            if mode != DomDiffMode::Changed && !hunks.is_empty() {
                return Err(invalid());
            }
            if mode == DomDiffMode::Changed && hunks.is_empty() && notice.is_none() {
                return Err(invalid());
            }
            let mut end = 0;
            for hunk in &hunks {
                if hunk.rows.start < end
                    || hunk.rows.start >= hunk.rows.end
                    || hunk.rows.end > rows.len()
                    || !rows[hunk.rows.clone()]
                        .iter()
                        .any(|row| row.kind != DomDiffRowKind::Context)
                    || rows[end..hunk.rows.start]
                        .iter()
                        .any(|row| row.kind != DomDiffRowKind::Context)
                {
                    return Err(invalid());
                }
                end = hunk.rows.end;
            }
            if rows[end..]
                .iter()
                .any(|row| row.kind != DomDiffRowKind::Context)
            {
                return Err(invalid());
            }
            let mut blocks = Vec::new();
            let mut end = 0;
            for (i, hunk) in hunks.iter().enumerate() {
                context_blocks(end..hunk.rows.start, &mut blocks);
                blocks.push(Block::Hunk(i));
                blocks.push(Block::Lines(hunk.rows.clone()));
                end = hunk.rows.end;
            }
            if matches!(mode, DomDiffMode::BeforeOnly | DomDiffMode::AfterOnly) {
                blocks.push(Block::Lines(0..rows.len()));
            } else {
                context_blocks(end..rows.len(), &mut blocks);
            }
            let mut segments = Vec::new();
            let mut row_segments = Vec::with_capacity(rows.len());
            let mut max_columns = 0;
            for (row_index, row) in rows.iter().enumerate() {
                let start = segments.len();
                for bytes in wrap_ranges(&row.text) {
                    max_columns = max_columns.max(row.text[bytes.clone()].chars().count());
                    segments.push((row_index, bytes));
                }
                row_segments.push(start..segments.len());
            }
            // This constructor runs on the display worker, never during paint.
            // Bound cosmetic metadata independently from the source/evidence.
            let mut budget = syntax::MAX_SPANS;
            let syntax = rows
                .iter()
                .map(|row| {
                    let spans = syntax::highlight(&row.text, budget);
                    budget -= spans.len();
                    spans
                })
                .collect();
            Ok(Self {
                mode,
                rows,
                hunks,
                notice,
                blocks,
                segments,
                row_segments,
                added,
                removed,
                max_columns,
                number_digits: old.max(new).max(1).ilog10() as usize + 1,
                syntax,
            })
        }

        pub fn mode(&self) -> DomDiffMode {
            self.mode
        }
        pub fn rows(&self) -> &[DomDiffRow] {
            &self.rows
        }
        pub fn hunks(&self) -> &[DomDiffHunk] {
            &self.hunks
        }

        pub(super) fn search(&self, query: &str, cancel: &AtomicBool) -> SearchResults {
            let mut results = SearchResults::default();
            if query.is_empty() {
                return results;
            }
            for (row, source) in self.rows.iter().enumerate() {
                if cancel.load(Ordering::Relaxed) {
                    break;
                }
                for (start, _) in source.text.match_indices(query) {
                    if results.matches.len() == MAX_MATCHES {
                        results.limited = true;
                        return results;
                    }
                    results.matches.push(SearchMatch {
                        row,
                        bytes: start..start + query.len(),
                    });
                }
            }
            results
        }

        pub(super) fn segment_at(&self, pos: SourcePosition) -> usize {
            let range = self.row_segments[pos.row].clone();
            let offset = self.segments[range.clone()]
                .partition_point(|(_, bytes)| bytes.end <= pos.byte && bytes.end != bytes.start);
            range.start + offset.min(range.len() - 1)
        }

        pub(super) fn selected_text(&self, selection: Selection) -> String {
            let (start, end) = selection.ordered();
            let mut out = String::new();
            for row in start.row..=end.row {
                if row != start.row {
                    out.push('\n');
                }
                let text = &self.rows[row].text;
                let from = if row == start.row { start.byte } else { 0 };
                let to = if row == end.row { end.byte } else { text.len() };
                out.push_str(&text[from..to]);
            }
            out
        }

        /// Resolve only the visible segment. All offsets remain original UTF-8
        /// bytes; colors never affect layout, hit testing, search or copying.
        pub(super) fn styled_segment(
            &self,
            row: usize,
            bytes: Range<usize>,
            selection: Selection,
            results: &SearchResults,
            current_match: Option<usize>,
        ) -> Vec<StyledSpan> {
            let syntax = &self.syntax[row];
            let start = syntax.partition_point(|span| span.bytes.end <= bytes.start);
            let syntax = &syntax[start..];
            let syntax = &syntax[..syntax.partition_point(|span| span.bytes.start < bytes.end)];
            let start = results.matches.partition_point(|found| {
                found.row < row || (found.row == row && found.bytes.end <= bytes.start)
            });
            let matches = &results.matches[start..];
            let matches = &matches[..matches
                .partition_point(|found| found.row == row && found.bytes.start < bytes.end)];
            let (from, to) = selection.ordered();
            let selected = if (from.row..=to.row).contains(&row) {
                (if row == from.row { from.byte } else { 0 })..(if row == to.row {
                    to.byte
                } else {
                    self.rows[row].text.len()
                })
            } else {
                0..0
            };
            // Visiting a match historically also selects it for Copy. Keep that
            // behavior, but give this exact search-owned selection its own ink.
            let search_selection = current_match
                .and_then(|index| results.matches.get(index))
                .is_some_and(|found| {
                    from == (SourcePosition {
                        row: found.row,
                        byte: found.bytes.start,
                    }) && to
                        == (SourcePosition {
                            row: found.row,
                            byte: found.bytes.end,
                        })
                });
            let mut boundaries = vec![bytes.start, bytes.end];
            for range in syntax
                .iter()
                .map(|span| &span.bytes)
                .chain(matches.iter().map(|found| &found.bytes))
                .chain(std::iter::once(&selected))
            {
                boundaries.push(range.start.clamp(bytes.start, bytes.end));
                boundaries.push(range.end.clamp(bytes.start, bytes.end));
            }
            boundaries.sort_unstable();
            boundaries.dedup();
            let mut spans: Vec<StyledSpan> = Vec::new();
            for pair in boundaries.windows(2) {
                let at = pair[0];
                let kind = syntax
                    .iter()
                    .find(|span| span.bytes.contains(&at))
                    .map_or(syntax::Kind::Plain, |span| span.kind);
                let found = matches.iter().position(|found| found.bytes.contains(&at));
                let emphasis = if selected.contains(&at) && !search_selection {
                    Emphasis::Selection
                } else if found.is_some_and(|index| current_match == Some(start + index)) {
                    Emphasis::CurrentMatch
                } else if found.is_some() {
                    Emphasis::Match
                } else {
                    Emphasis::None
                };
                if let Some(last) = spans
                    .last_mut()
                    .filter(|span| span.kind == kind && span.emphasis == emphasis)
                {
                    last.bytes.end = pair[1];
                } else {
                    spans.push(StyledSpan {
                        bytes: pair[0]..pair[1],
                        kind,
                        emphasis,
                    });
                }
            }
            spans
        }
    }

    fn context_blocks(range: Range<usize>, blocks: &mut Vec<Block>) {
        if range.len() > 8 {
            blocks.push(Block::Lines(range.start..range.start + 3));
            blocks.push(Block::Fold(range.start + 3..range.end - 3));
            blocks.push(Block::Lines(range.end - 3..range.end));
        } else if !range.is_empty() {
            blocks.push(Block::Lines(range));
        }
    }

    pub(super) fn wrap_ranges(text: &str) -> Vec<Range<usize>> {
        let mut ranges = Vec::new();
        let mut start = 0;
        for (column, (byte, _)) in text.char_indices().enumerate() {
            if column > 0 && column % WRAP_COLUMNS == 0 {
                ranges.push(start..byte);
                start = byte;
            }
        }
        ranges.push(start..text.len());
        ranges
    }

    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    pub(super) struct SourcePosition {
        pub row: usize,
        pub byte: usize,
    }
    #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
    pub(super) struct Selection {
        pub anchor: SourcePosition,
        pub caret: SourcePosition,
    }
    impl Selection {
        pub fn ordered(self) -> (SourcePosition, SourcePosition) {
            (self.anchor.min(self.caret), self.anchor.max(self.caret))
        }
        pub fn is_empty(self) -> bool {
            self.anchor == self.caret
        }
    }
    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(super) struct SearchMatch {
        pub row: usize,
        pub bytes: Range<usize>,
    }
    #[derive(Default)]
    pub(super) struct SearchResults {
        pub matches: Vec<SearchMatch>,
        pub limited: bool,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub(super) enum Emphasis {
        None,
        Match,
        CurrentMatch,
        Selection,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub(super) struct StyledSpan {
        pub bytes: Range<usize>,
        pub kind: syntax::Kind,
        pub emphasis: Emphasis,
    }
}

use model::{Block, Emphasis, SearchResults, Selection, SourcePosition, WRAP_COLUMNS};
pub use model::{
    DomDiffDocument, DomDiffError, DomDiffHunk, DomDiffMode, DomDiffRow, DomDiffRowKind,
};

use super::{
    controls, interaction, scrollbar,
    text_input::TextInput,
    theme::*,
    toolbar::{self, ToolIcon},
};
use gpui::{prelude::*, *};

const SOURCE_SIZE: f32 = 13.;
const SOURCE_HEIGHT: f32 = 20.;
const MONO: &str = if cfg!(target_os = "windows") {
    "Consolas"
} else if cfg!(target_os = "macos") {
    "Menlo"
} else {
    "DejaVu Sans Mono"
};

actions!(
    dom_diff,
    [
        Activate,
        CopySelection,
        SelectAll,
        Find,
        NextMatch,
        PreviousMatch,
        NextHunk,
        PreviousHunk,
        ClearSelection
    ]
);

#[derive(Default)]
struct Bindings;
impl Global for Bindings {}

fn bind(cx: &mut App) {
    if cx.try_global::<Bindings>().is_some() {
        return;
    }
    cx.set_global(Bindings);
    cx.bind_keys([
        KeyBinding::new("enter", Activate, Some("DomDiffControl")),
        KeyBinding::new("space", Activate, Some("DomDiffControl")),
        KeyBinding::new("f3", NextMatch, Some("DomDiff")),
        KeyBinding::new("shift-f3", PreviousMatch, Some("DomDiff")),
        KeyBinding::new("alt-down", NextHunk, Some("DomDiff")),
        KeyBinding::new("alt-up", PreviousHunk, Some("DomDiff")),
        KeyBinding::new("enter", Activate, Some("DomDiffSource")),
        KeyBinding::new("space", Activate, Some("DomDiffSource")),
        KeyBinding::new("escape", ClearSelection, Some("DomDiffSource")),
        KeyBinding::new("enter", NextMatch, Some("DomDiff && TextInput")),
        KeyBinding::new("shift-enter", PreviousMatch, Some("DomDiff && TextInput")),
    ]);
    for modifier in ["ctrl", "cmd"] {
        cx.bind_keys([
            KeyBinding::new(
                &format!("{modifier}-c"),
                CopySelection,
                Some("DomDiffSource"),
            ),
            KeyBinding::new(&format!("{modifier}-a"), SelectAll, Some("DomDiffSource")),
            KeyBinding::new(&format!("{modifier}-f"), Find, Some("DomDiff")),
        ]);
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DomDiffLoadToken {
    generation: u64,
    identity: SharedString,
}

enum LoadState {
    Empty,
    Loading,
    Failed(DomDiffError),
    Ready(Arc<DomDiffDocument>),
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Control {
    Clear,
    PreviousMatch,
    NextMatch,
    PreviousHunk,
    NextHunk,
    Context,
    Copy,
}
const CONTROLS: [Control; 7] = [
    Control::Clear,
    Control::PreviousMatch,
    Control::NextMatch,
    Control::PreviousHunk,
    Control::NextHunk,
    Control::Context,
    Control::Copy,
];

#[derive(Clone, Debug)]
enum Item {
    Source(usize),
    Hunk(usize),
    Fold(usize),
}
impl Item {
    fn height(&self) -> f32 {
        match self {
            Self::Source(_) => SOURCE_HEIGHT,
            _ => CONTROL_HEIGHT,
        }
    }
}

#[derive(Clone)]
struct PaintedLine {
    segment: usize,
    bounds: Bounds<Pixels>,
    line: ShapedLine,
}

/// An owned, native source viewer; only the separate search field is editable.
pub struct DomDiff {
    state: LoadState,
    token: DomDiffLoadToken,
    enabled: bool,
    search: Entity<TextInput>,
    search_focus: FocusHandle,
    source_focus: FocusHandle,
    state_focus: FocusHandle,
    state_scroll: ScrollHandle,
    tools_scroll: ScrollHandle,
    tools_focus_row: Option<usize>,
    controls: Vec<FocusHandle>,
    _search_observer: Subscription,
    search_task: Option<Task<()>>,
    cancel_search: Arc<AtomicBool>,
    query: SharedString,
    search_generation: u64,
    searching: bool,
    results: SearchResults,
    current_match: Option<usize>,
    current_hunk: Option<usize>,
    expanded: BTreeSet<usize>,
    items: Vec<Item>,
    tops: Vec<f32>,
    segment_items: Vec<Option<usize>>,
    hunk_items: Vec<usize>,
    offset: Point<Pixels>,
    viewport: Option<Bounds<Pixels>>,
    content_width: Pixels,
    gutter_width: Pixels,
    painted: Vec<PaintedLine>,
    painted_generation: u64,
    selection: Selection,
    selecting: bool,
    selection_pointer: Option<Point<Pixels>>,
    autoscrolling: bool,
    selection_epoch: u64,
    preferred_column: Option<usize>,
    pending_reveal: Option<SourcePosition>,
    copied: bool,
    hints: Option<toolbar::HintInput>,
}

impl DomDiff {
    pub fn new(cx: &mut Context<Self>) -> Self {
        bind(cx);
        let search = cx.new(|cx| {
            let mut input = TextInput::search("Search DOM source…", 0, cx);
            input.max_bytes = 512;
            input.debug_name = Some("dom-diff-search");
            input.set_enabled(false);
            input
        });
        let search_focus = search.read(cx).focus.clone();
        let observer = cx.observe(&search, |this, input, cx| {
            let query = input.read(cx).content.clone();
            if !input.read(cx).composing() && this.query != query {
                this.query = query;
                this.start_search(cx);
            }
        });
        Self {
            state: LoadState::Empty,
            token: DomDiffLoadToken {
                generation: 0,
                identity: "".into(),
            },
            enabled: true,
            search,
            search_focus,
            source_focus: cx.focus_handle().tab_stop(false),
            state_focus: cx.focus_handle().tab_stop(false),
            state_scroll: ScrollHandle::new(),
            tools_scroll: ScrollHandle::new(),
            tools_focus_row: None,
            controls: CONTROLS
                .iter()
                .map(|_| cx.focus_handle().tab_stop(false))
                .collect(),
            _search_observer: observer,
            search_task: None,
            cancel_search: Arc::new(AtomicBool::new(false)),
            query: "".into(),
            search_generation: 0,
            searching: false,
            results: SearchResults::default(),
            current_match: None,
            current_hunk: None,
            expanded: BTreeSet::new(),
            items: Vec::new(),
            tops: vec![0.],
            segment_items: Vec::new(),
            hunk_items: Vec::new(),
            offset: point(px(0.), px(0.)),
            viewport: None,
            content_width: px(0.),
            gutter_width: px(112.),
            painted: Vec::new(),
            painted_generation: 0,
            selection: Selection::default(),
            selecting: false,
            selection_pointer: None,
            autoscrolling: false,
            selection_epoch: 0,
            preferred_column: None,
            pending_reveal: None,
            copied: false,
            hints: None,
        }
    }

    /// Clear old evidence immediately, before dispatching the asynchronous read.
    pub fn begin_load(
        &mut self,
        identity: impl Into<SharedString>,
        cx: &mut Context<Self>,
    ) -> DomDiffLoadToken {
        self.token = DomDiffLoadToken {
            generation: self.token.generation.wrapping_add(1),
            identity: identity.into(),
        };
        self.cancel_search.store(true, Ordering::Relaxed);
        self.search_task = None;
        self.search_generation = self.search_generation.wrapping_add(1);
        self.state = LoadState::Loading;
        self.query = "".into();
        self.results = SearchResults::default();
        self.searching = false;
        self.current_match = None;
        self.current_hunk = None;
        self.expanded.clear();
        self.selection = Selection::default();
        self.selecting = false;
        self.selection_pointer = None;
        self.autoscrolling = false;
        self.preferred_column = None;
        self.pending_reveal = None;
        self.copied = false;
        self.offset = point(px(0.), px(0.));
        self.viewport = None;
        self.state_scroll.set_offset(point(px(0.), px(0.)));
        self.tools_scroll.set_offset(point(px(0.), px(0.)));
        self.tools_focus_row = None;
        self.content_width = px(0.);
        self.painted.clear();
        self.items.clear();
        self.tops = vec![0.];
        self.segment_items.clear();
        self.hunk_items.clear();
        self.search.update(cx, |input, cx| {
            input.set_value("", cx);
            input.set_enabled(false);
        });
        self.sync_focus(cx);
        cx.notify();
        self.token.clone()
    }

    /// Returns false for a stale reply, with no state/focus/clipboard effects.
    pub fn finish_load(
        &mut self,
        token: &DomDiffLoadToken,
        result: Result<Arc<DomDiffDocument>, DomDiffError>,
        cx: &mut Context<Self>,
    ) -> bool {
        if token != &self.token || !matches!(self.state, LoadState::Loading) {
            return false;
        }
        self.state = match result {
            Ok(document) => LoadState::Ready(document),
            Err(error) => LoadState::Failed(error),
        };
        self.rebuild_items();
        self.sync_focus(cx);
        cx.notify();
        true
    }

    /// Suspend on hidden tabs, inspectors, modal occlusion or remote-control locks.
    pub fn set_enabled(&mut self, enabled: bool, cx: &mut Context<Self>) {
        if self.enabled != enabled {
            self.enabled = enabled;
            self.selecting = false;
            self.selection_pointer = None;
            self.autoscrolling = false;
            self.sync_focus(cx);
            cx.notify();
        }
    }

    /// Integrate these stops at the DOM stage's position in shell reading order.
    pub fn focus_handles(&self, cx: &App) -> Vec<FocusHandle> {
        if !self.enabled {
            return Vec::new();
        }
        if self.document().is_none() {
            return if matches!(self.state, LoadState::Failed(_)) {
                vec![self.state_focus.clone()]
            } else {
                Vec::new()
            };
        }
        let mut stops = vec![self.search.read(cx).focus.clone()];
        stops.extend(
            CONTROLS
                .into_iter()
                .filter(|&c| self.control_enabled(c))
                .map(|c| self.handle(c)),
        );
        if self.document().is_some_and(|d| !d.rows.is_empty()) {
            stops.push(self.source_focus.clone());
        }
        stops
    }

    pub fn focus_search(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.allowed(window, cx) && self.document().is_some() {
            window.focus(&self.search_focus);
        }
    }

    fn document(&self) -> Option<&Arc<DomDiffDocument>> {
        match &self.state {
            LoadState::Ready(document) => Some(document),
            _ => None,
        }
    }
    fn allowed(&self, window: &Window, cx: &App) -> bool {
        self.enabled && interaction::allowed(window, cx)
    }
    fn handle(&self, control: Control) -> FocusHandle {
        self.controls[CONTROLS.iter().position(|&c| c == control).unwrap()].clone()
    }
    fn control_enabled(&self, control: Control) -> bool {
        let Some(document) = self.document().filter(|_| self.enabled) else {
            return false;
        };
        match control {
            Control::Clear => !self.query.is_empty(),
            Control::PreviousMatch | Control::NextMatch => {
                !self.searching && !self.results.matches.is_empty()
            }
            Control::PreviousHunk | Control::NextHunk => !document.hunks.is_empty(),
            Control::Context => document.blocks.iter().any(|b| matches!(b, Block::Fold(_))),
            Control::Copy => !self.selection.is_empty(),
        }
    }
    fn sync_focus(&mut self, cx: &mut Context<Self>) {
        let ready = self.enabled && self.document().is_some();
        self.search.update(cx, |input, _| input.set_enabled(ready));
        self.state_focus = self
            .state_focus
            .clone()
            .tab_stop(self.enabled && matches!(self.state, LoadState::Failed(_)));
        self.source_focus = self
            .source_focus
            .clone()
            .tab_stop(ready && self.document().is_some_and(|d| !d.rows.is_empty()));
        for (i, control) in CONTROLS.into_iter().enumerate() {
            self.controls[i] = self.controls[i]
                .clone()
                .tab_stop(self.control_enabled(control));
        }
    }

    fn rebuild_items(&mut self) {
        let Some(document) = self.document().cloned() else {
            return;
        };
        self.items.clear();
        self.tops = vec![0.];
        self.segment_items = vec![None; document.segments.len()];
        self.hunk_items = vec![0; document.hunks.len()];
        for (block_index, block) in document.blocks.iter().enumerate() {
            match block {
                Block::Hunk(hunk) => {
                    self.hunk_items[*hunk] = self.items.len();
                    self.push_item(Item::Hunk(*hunk));
                }
                Block::Fold(_) if !self.expanded.contains(&block_index) => {
                    self.push_item(Item::Fold(block_index))
                }
                Block::Lines(rows) | Block::Fold(rows) => {
                    for row in rows.clone() {
                        for segment in document.row_segments[row].clone() {
                            self.segment_items[segment] = Some(self.items.len());
                            self.push_item(Item::Source(segment));
                        }
                    }
                }
            }
        }
        self.clamp_offset();
        self.painted.clear();
    }
    fn push_item(&mut self, item: Item) {
        self.tops
            .push(self.tops.last().copied().unwrap_or(0.) + item.height());
        self.items.push(item);
    }
    fn overflow(&self, bounds: Bounds<Pixels>) -> Size<Pixels> {
        size(
            (self.content_width - bounds.size.width).max(px(0.)),
            (px(*self.tops.last().unwrap_or(&0.)) + px(scrollbar::GUTTER) - bounds.size.height)
                .max(px(0.)),
        )
    }
    fn clamp_offset(&mut self) {
        if let Some(bounds) = self.viewport {
            let overflow = self.overflow(bounds);
            self.offset.x = self.offset.x.clamp(-overflow.width, px(0.));
            self.offset.y = self.offset.y.clamp(-overflow.height, px(0.));
        }
    }
    fn reveal_item(&mut self, item: usize) {
        let Some(bounds) = self.viewport else { return };
        let top = px(self.tops[item]);
        let bottom = px(self.tops[item + 1]);
        let visible = (bounds.size.height - px(scrollbar::GUTTER)).max(px(SOURCE_HEIGHT));
        if top < -self.offset.y {
            self.offset.y = -top;
        } else if bottom > -self.offset.y + visible {
            self.offset.y = visible - bottom;
        }
        self.clamp_offset();
    }
    fn reveal_position(&mut self, pos: SourcePosition) {
        let Some(document) = self.document().cloned().filter(|d| !d.rows.is_empty()) else {
            return;
        };
        let segment = document.segment_at(pos);
        self.pending_reveal = Some(pos);
        if self.segment_items[segment].is_none() {
            for (i, block) in document.blocks.iter().enumerate() {
                if matches!(block, Block::Fold(rows) if rows.contains(&pos.row)) {
                    self.expanded.insert(i);
                }
            }
            self.rebuild_items();
        }
        if let Some(item) = self.segment_items[segment] {
            self.reveal_item(item);
        }
        // An 80-scalar segment can still exceed a narrow pane. Reveal its caret
        // approximately now; shaped geometry corrects this on the next frame.
        let bytes = &document.segments[segment].1;
        let columns = document.rows[pos.row].text[bytes.start..pos.byte]
            .chars()
            .count();
        let x = self.gutter_width + px(columns as f32 * SOURCE_SIZE * 0.61);
        if let Some(bounds) = self.viewport {
            let right = bounds.size.width - px(scrollbar::GUTTER + 8.);
            if x + self.offset.x > right {
                self.offset.x = right - x;
            }
            if x + self.offset.x < self.gutter_width {
                self.offset.x = self.gutter_width - x;
            }
            self.clamp_offset();
        }
    }

    fn start_search(&mut self, cx: &mut Context<Self>) {
        self.cancel_search.store(true, Ordering::Relaxed);
        self.search_task = None;
        self.search_generation = self.search_generation.wrapping_add(1);
        self.results = SearchResults::default();
        self.current_match = None;
        self.searching = false;
        let Some(document) = self.document().cloned().filter(|_| !self.query.is_empty()) else {
            self.sync_focus(cx);
            cx.notify();
            return;
        };
        self.searching = true;
        let query = self.query.clone();
        let generation = self.search_generation;
        let token = self.token.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        self.cancel_search = cancel.clone();
        let job = cx
            .background_executor()
            .spawn(async move { document.search(&query, &cancel) });
        self.search_task = Some(cx.spawn(async move |this, cx| {
            let results = job.await;
            let _ = this.update(cx, |this, cx| {
                if this.token == token && this.search_generation == generation {
                    this.searching = false;
                    this.results = results;
                    if !this.results.matches.is_empty() {
                        this.visit_match(0);
                    }
                    this.sync_focus(cx);
                    cx.notify();
                }
            });
        }));
        self.sync_focus(cx);
        cx.notify();
    }
    fn visit_match(&mut self, index: usize) {
        let Some(found) = self.results.matches.get(index).cloned() else {
            return;
        };
        self.current_match = Some(index);
        self.selection = Selection {
            anchor: SourcePosition {
                row: found.row,
                byte: found.bytes.start,
            },
            caret: SourcePosition {
                row: found.row,
                byte: found.bytes.end,
            },
        };
        self.copied = false;
        self.reveal_position(self.selection.anchor);
    }
    fn navigate_match(&mut self, forward: bool, cx: &mut Context<Self>) {
        let count = self.results.matches.len();
        if count == 0 || self.searching {
            return;
        }
        let index = match self.current_match {
            Some(index) => (index + if forward { 1 } else { count - 1 }) % count,
            None => {
                if forward {
                    0
                } else {
                    count - 1
                }
            }
        };
        self.visit_match(index);
        self.sync_focus(cx);
        cx.notify();
    }
    fn navigate_hunk(&mut self, forward: bool, cx: &mut Context<Self>) {
        let Some(document) = self.document() else {
            return;
        };
        let count = document.hunks.len();
        if count == 0 {
            return;
        }
        let index = match self.current_hunk {
            Some(index) => (index + if forward { 1 } else { count - 1 }) % count,
            None => {
                if forward {
                    0
                } else {
                    count - 1
                }
            }
        };
        self.current_hunk = Some(index);
        self.reveal_item(self.hunk_items[index]);
        cx.notify();
    }
    fn toggle_context(&mut self, cx: &mut Context<Self>) {
        let Some(document) = self.document().cloned() else {
            return;
        };
        if self.expanded.is_empty() {
            self.expanded.extend(
                document
                    .blocks
                    .iter()
                    .enumerate()
                    .filter_map(|(i, b)| matches!(b, Block::Fold(_)).then_some(i)),
            );
        } else {
            self.expanded.clear();
        }
        self.rebuild_items();
        cx.notify();
    }
    fn copy_selection(&mut self, window: &Window, cx: &mut Context<Self>) {
        if !self.allowed(window, cx) || !self.control_enabled(Control::Copy) {
            return;
        }
        if let Some(document) = self.document() {
            cx.write_to_clipboard(ClipboardItem::new_string(
                document.selected_text(self.selection),
            ));
            self.copied = true;
            cx.notify();
        }
    }
    fn activate(&mut self, control: Control, window: &mut Window, cx: &mut Context<Self>) {
        if !self.allowed(window, cx) || !self.control_enabled(control) {
            return;
        }
        match control {
            Control::Clear => {
                self.search.update(cx, |input, cx| input.set_value("", cx));
                self.query = "".into();
                self.start_search(cx);
                window.focus(&self.search_focus);
            }
            Control::PreviousMatch => self.navigate_match(false, cx),
            Control::NextMatch => self.navigate_match(true, cx),
            Control::PreviousHunk => self.navigate_hunk(false, cx),
            Control::NextHunk => self.navigate_hunk(true, cx),
            Control::Context => self.toggle_context(cx),
            Control::Copy => self.copy_selection(window, cx),
        }
    }

    fn control(
        &self,
        control: Control,
        id: &'static str,
        label: &'static str,
        cx: &Context<Self>,
    ) -> Stateful<Div> {
        let target = cx.entity();
        let frame = if control == Control::Context {
            controls::toggle(
                id,
                label,
                &self.handle(control),
                !self.expanded.is_empty(),
                self.control_enabled(control),
            )
        } else {
            toolbar::button(
                id,
                match control {
                    Control::Clear => ToolIcon::Close,
                    Control::PreviousMatch | Control::PreviousHunk => ToolIcon::Up,
                    Control::NextMatch | Control::NextHunk => ToolIcon::Down,
                    Control::Copy => ToolIcon::Copy,
                    Control::Context => unreachable!(),
                },
                None,
                label.into(),
                &self.handle(control),
                false,
                self.control_enabled(control),
                self.hints.as_ref().is_some_and(|hints| hints.is_keyboard()),
            )
        };
        frame
            .key_context("DomDiffControl")
            .on_click(move |event, window, cx| {
                if matches!(event, ClickEvent::Mouse(_)) {
                    target.update(cx, |this, cx| {
                        if this.allowed(window, cx) && this.control_enabled(control) {
                            window.focus(&this.handle(control));
                            this.activate(control, window, cx);
                        }
                    });
                }
                cx.stop_propagation();
            })
    }

    fn position_for_pointer(&self, point: Point<Pixels>) -> Option<SourcePosition> {
        let document = self.document()?;
        if self.painted_generation != self.token.generation {
            return None;
        }
        let line = self.painted.iter().min_by(|a, b| {
            let distance = |line: &PaintedLine| (f32::from(point.y - line.bounds.center().y)).abs();
            distance(a).total_cmp(&distance(b))
        })?;
        let (row, bytes) = &document.segments[line.segment];
        let byte = line.line.closest_index_for_x(point.x - line.bounds.left());
        Some(SourcePosition {
            row: *row,
            byte: bytes.start + byte.min(bytes.len()),
        })
    }
    fn mouse_down(&mut self, event: &MouseDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.allowed(window, cx)
            || self.document().is_none()
            || self.painted_generation != self.token.generation
        {
            return;
        }
        let Some(viewport) = self.viewport else {
            return;
        };
        // Leave the shared scrollbar's reserved hit areas to its own handler.
        if event.position.x >= viewport.right() - px(scrollbar::GUTTER)
            || event.position.y >= viewport.bottom() - px(scrollbar::GUTTER)
        {
            return;
        }
        let y = f32::from(event.position.y - viewport.top() - self.offset.y);
        let item = self.tops.partition_point(|&top| top <= y).saturating_sub(1);
        if let Some(Item::Fold(block)) = self.items.get(item) {
            self.expanded.insert(*block);
            self.rebuild_items();
            window.focus(&self.source_focus);
            cx.notify();
            return;
        }
        let Some(position) = self.position_for_pointer(event.position) else {
            return;
        };
        window.focus(&self.source_focus);
        self.selection_epoch = interaction::epoch(window, cx);
        self.selecting = true;
        self.selection_pointer = Some(event.position);
        self.preferred_column = None;
        self.pending_reveal = None;
        self.copied = false;
        if event.modifiers.shift {
            self.selection.caret = position;
        } else if event.click_count >= 3 {
            let document = self.document().unwrap();
            self.selection = Selection {
                anchor: SourcePosition {
                    row: position.row,
                    byte: 0,
                },
                caret: SourcePosition {
                    row: position.row,
                    byte: document.rows[position.row].text.len(),
                },
            };
        } else if event.click_count == 2 {
            let text = &self.document().unwrap().rows[position.row].text;
            let bytes = word_range(text, position.byte);
            self.selection = Selection {
                anchor: SourcePosition {
                    row: position.row,
                    byte: bytes.start,
                },
                caret: SourcePosition {
                    row: position.row,
                    byte: bytes.end,
                },
            };
        } else {
            self.selection = Selection {
                anchor: position,
                caret: position,
            };
        }
        self.sync_focus(cx);
        cx.notify();
        cx.stop_propagation();
    }
    fn mouse_move(&mut self, event: &MouseMoveEvent, window: &mut Window, cx: &mut Context<Self>) {
        if !self.selecting {
            return;
        }
        if !self.allowed(window, cx)
            || self.selection_epoch != interaction::epoch(window, cx)
            || event.pressed_button != Some(MouseButton::Left)
            || !self.source_focus.is_focused(window)
        {
            self.selecting = false;
            self.selection_pointer = None;
            return;
        }
        self.selection_pointer = Some(event.position);
        if let Some(position) = self.position_for_pointer(event.position) {
            self.selection.caret = position;
            self.copied = false;
            self.sync_focus(cx);
            cx.notify();
        }
    }
    fn scroll(&mut self, event: &ScrollWheelEvent, window: &Window, cx: &mut Context<Self>) {
        if !self.allowed(window, cx) {
            return;
        }
        let mut delta = event.delta.pixel_delta(px(SOURCE_HEIGHT));
        self.pending_reveal = None;
        if event.modifiers.shift && delta.x == px(0.) {
            delta.x = delta.y;
            delta.y = px(0.);
        }
        self.offset += delta;
        self.clamp_offset();
        cx.notify();
        cx.stop_propagation();
    }

    fn state_key(&mut self, event: &KeyDownEvent, window: &Window, cx: &mut Context<Self>) {
        if !self.allowed(window, cx) || !self.state_focus.is_focused(window) {
            return;
        }
        let page = self.state_scroll.bounds().size.height * 0.9;
        let offset = self.state_scroll.offset();
        let y = match event.keystroke.key.as_str() {
            "home" => px(0.),
            "end" => -self.state_scroll.max_offset().height,
            "up" => offset.y + px(BODY_LINE_HEIGHT),
            "down" => offset.y - px(BODY_LINE_HEIGHT),
            "pageup" => offset.y + page,
            "pagedown" => offset.y - page,
            _ => return,
        }
        .clamp(-self.state_scroll.max_offset().height, px(0.));
        self.state_scroll.set_offset(point(offset.x, y));
        cx.notify();
        cx.stop_propagation();
    }

    fn move_caret(&mut self, position: SourcePosition, extend: bool, cx: &mut Context<Self>) {
        self.selection.caret = position;
        if !extend {
            self.selection.anchor = position;
        }
        self.copied = false;
        self.reveal_position(position);
        self.sync_focus(cx);
        cx.notify();
    }
    fn source_key(&mut self, event: &KeyDownEvent, window: &Window, cx: &mut Context<Self>) {
        if !self.allowed(window, cx) {
            return;
        }
        let Some(document) = self.document().cloned().filter(|d| !d.rows.is_empty()) else {
            return;
        };
        let key = &event.keystroke;
        // Cut, paste, Delete, Backspace, text and IME never have a mutation path.
        if key.modifiers.alt {
            return;
        }
        let mut pos = self.selection.caret;
        let text = &document.rows[pos.row].text;
        let segment = document.segment_at(pos);
        let mut vertical = false;
        match key.key.as_str() {
            "left" if !key.modifiers.control && !key.modifiers.platform => {
                if !key.modifiers.shift && !self.selection.is_empty() {
                    pos = self.selection.ordered().0;
                } else if pos.byte > 0 {
                    pos.byte = text[..pos.byte]
                        .char_indices()
                        .next_back()
                        .map_or(0, |(n, _)| n);
                } else if pos.row > 0 {
                    pos.row -= 1;
                    pos.byte = document.rows[pos.row].text.len();
                }
            }
            "right" if !key.modifiers.control && !key.modifiers.platform => {
                if !key.modifiers.shift && !self.selection.is_empty() {
                    pos = self.selection.ordered().1;
                } else if pos.byte < text.len() {
                    pos.byte += text[pos.byte..].chars().next().map_or(0, char::len_utf8);
                } else if pos.row + 1 < document.rows.len() {
                    pos.row += 1;
                    pos.byte = 0;
                }
            }
            "home" => {
                if key.modifiers.control || key.modifiers.platform {
                    pos.row = 0;
                }
                pos.byte = 0;
            }
            "end" => {
                if key.modifiers.control || key.modifiers.platform {
                    pos.row = document.rows.len() - 1;
                }
                pos.byte = document.rows[pos.row].text.len();
            }
            "up" | "down" | "pageup" | "pagedown"
                if !key.modifiers.control && !key.modifiers.platform =>
            {
                vertical = true;
                let range = &document.segments[segment].1;
                let column = *self
                    .preferred_column
                    .get_or_insert_with(|| text[range.start..pos.byte].chars().count());
                let step = if matches!(key.key.as_str(), "pageup" | "pagedown") {
                    self.viewport.map_or(10, |b| {
                        (f32::from(b.size.height) * 0.9 / SOURCE_HEIGHT).max(1.) as usize
                    })
                } else {
                    1
                };
                let next = if matches!(key.key.as_str(), "up" | "pageup") {
                    segment.saturating_sub(step)
                } else {
                    segment
                        .saturating_add(step)
                        .min(document.segments.len() - 1)
                };
                let (row, bytes) = &document.segments[next];
                pos = SourcePosition {
                    row: *row,
                    byte: bytes.start
                        + document.rows[*row].text[bytes.clone()]
                            .char_indices()
                            .nth(column)
                            .map_or(bytes.len(), |(n, _)| n),
                };
            }
            "escape" => {
                self.selection.anchor = self.selection.caret;
                self.sync_focus(cx);
                cx.notify();
                cx.stop_propagation();
                return;
            }
            _ => return,
        }
        if !vertical {
            self.preferred_column = None;
        }
        self.move_caret(pos, key.modifiers.shift, cx);
        cx.stop_propagation();
    }
}

impl Drop for DomDiff {
    fn drop(&mut self) {
        self.cancel_search.store(true, Ordering::Relaxed);
    }
}

fn word_range(text: &str, byte: usize) -> Range<usize> {
    fn class(c: char) -> u8 {
        if c.is_alphanumeric() || c == '_' {
            0
        } else if c.is_whitespace() {
            1
        } else {
            2
        }
    }
    let byte = if byte == text.len() {
        text.char_indices().next_back().map_or(0, |(n, _)| n)
    } else {
        byte
    };
    let Some(c) = text[byte..].chars().next() else {
        return 0..0;
    };
    let kind = class(c);
    let mut start = byte;
    for (n, c) in text[..byte].char_indices().rev() {
        if class(c) != kind {
            break;
        }
        start = n;
    }
    let mut end = byte;
    for c in text[byte..].chars() {
        if class(c) != kind {
            break;
        }
        end += c.len_utf8();
    }
    start..end
}

fn source_colors(kind: syntax::Kind, emphasis: Emphasis) -> (u32, Option<u32>) {
    let p = palette();
    let dom = dom_palette();
    match emphasis {
        Emphasis::Selection => (p.ink, Some(dom.selection)),
        Emphasis::CurrentMatch => (p.on_accent, Some(p.accent_hover)),
        Emphasis::Match => (p.ink, Some(dom.search)),
        Emphasis::None => (
            match kind {
                syntax::Kind::Plain => p.ink,
                syntax::Kind::Metadata => p.muted,
                syntax::Kind::Structure | syntax::Kind::Element => dom.structure,
                syntax::Kind::Attribute => dom.attribute,
                syntax::Kind::Value => dom.value,
            },
            None,
        ),
    }
}

impl DomDiff {
    fn source_surface(&self, cx: &Context<Self>) -> impl IntoElement + use<> {
        let layout_entity = cx.entity();
        let paint_entity = cx.entity();
        let metrics_entity = cx.entity();
        let scroll_entity = cx.entity();
        scrollbar::both(
            "dom-diff-scrollbars",
            div()
                .id("dom-diff-source")
                .debug_selector(|| "dom-diff-source".into())
                .relative()
                .size_full()
                .min_h_0()
                .min_w_0()
                .overflow_hidden()
                .key_context("DomDiffSource")
                .when(
                    self.enabled && self.document().is_some_and(|d| !d.rows.is_empty()),
                    |el| el.track_focus(&self.source_focus),
                )
                .cursor(CursorStyle::IBeam)
                .on_mouse_down(MouseButton::Left, cx.listener(Self::mouse_down))
                .on_scroll_wheel(
                    cx.listener(|this, event, window, cx| this.scroll(event, window, cx)),
                )
                .on_key_down(
                    cx.listener(|this, event, window, cx| this.source_key(event, window, cx)),
                )
                .child(
                    canvas(
                        move |bounds, window, cx| {
                            layout_entity
                                .update(cx, |this, cx| this.prepare_frame(bounds, window, cx))
                        },
                        move |bounds, frame, window, cx| {
                            paint_entity
                                .update(cx, |this, cx| this.paint_frame(bounds, frame, window, cx));
                        },
                    )
                    .size_full(),
                ),
            move |bounds, cx| {
                let this = metrics_entity.read(cx);
                scrollbar::ScrollMetrics {
                    viewport: bounds,
                    overflow: this.overflow(bounds),
                    offset: this.offset,
                }
            },
            move |offset, window, cx| {
                scroll_entity.update(cx, |this, cx| {
                    if this.allowed(window, cx) {
                        this.pending_reveal = None;
                        this.offset = offset;
                        this.clamp_offset();
                        cx.notify();
                    }
                });
            },
        )
    }

    fn prepare_frame(
        &mut self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Vec<FrameRow> {
        self.viewport = Some(bounds);
        if !self.allowed(window, cx) || self.selection_epoch != interaction::epoch(window, cx) {
            self.selecting = false;
            self.selection_pointer = None;
        }
        let style = window.text_style();
        let mut font = style.font();
        font.family = MONO.into();
        let mono_run = |len, color| TextRun {
            len,
            font: font.clone(),
            color: rgb(color).into(),
            background_color: None,
            underline: None,
            strikethrough: None,
        };
        let cell = window
            .text_system()
            .shape_line(
                "M".into(),
                px(SOURCE_SIZE),
                &[mono_run(1, palette().ink)],
                None,
            )
            .width;
        let Some(document) = self.document().cloned() else {
            return Vec::new();
        };
        // Reserve enough space for the actual largest one-based line number,
        // rather than clipping seven-digit original positions into six slots.
        let digits = document.number_digits.max(4);
        self.gutter_width = cell * (digits * 2 + 4) as f32 + px(16.);
        self.content_width = self.content_width.max(
            self.gutter_width + px(scrollbar::GUTTER + 12.) + cell * document.max_columns as f32,
        );
        let previous_offset = self.offset;
        if let Some(pointer) = self.selection_pointer.filter(|_| self.selecting) {
            if pointer.y < bounds.top() + px(4.) {
                self.offset.y += px(SOURCE_HEIGHT);
            }
            if pointer.y > bounds.bottom() - px(scrollbar::GUTTER + 4.) {
                self.offset.y -= px(SOURCE_HEIGHT);
            }
            if pointer.x < bounds.left() + self.gutter_width {
                self.offset.x += px(12.);
            }
            if pointer.x > bounds.right() - px(scrollbar::GUTTER) {
                self.offset.x -= px(12.);
            }
        }
        self.clamp_offset();
        if let Some(pos) = self.pending_reveal {
            let segment = document.segment_at(pos);
            if let Some(item) = self.segment_items[segment] {
                self.reveal_item(item);
            }
        }
        self.autoscrolling = self.selecting && self.offset != previous_offset;
        self.painted.clear();
        self.painted_generation = self.token.generation;
        let start = self
            .tops
            .partition_point(|&top| top <= -f32::from(self.offset.y))
            .saturating_sub(1);
        let bottom = -f32::from(self.offset.y) + f32::from(bounds.size.height);
        let end = self
            .tops
            .partition_point(|&top| top < bottom)
            .saturating_add(1)
            .min(self.items.len());
        let mut rows = Vec::with_capacity(end.saturating_sub(start));
        // This upper bound is defensive against erroneous/unbounded host layout.
        // A real desktop viewport needs far fewer than 512 20px source rows.
        for index in start..end.min(start + 512) {
            let top = bounds.top() + self.offset.y + px(self.tops[index]);
            let row_bounds = Bounds::new(
                point(bounds.left(), top),
                size(bounds.size.width, px(self.items[index].height())),
            );
            match self.items[index] {
                Item::Source(segment) => {
                    let (row_index, bytes) = &document.segments[segment];
                    let row = &document.rows[*row_index];
                    let text: SharedString = row.text[bytes.clone()].to_owned().into();
                    let mut runs = document
                        .styled_segment(
                            *row_index,
                            bytes.clone(),
                            self.selection,
                            &self.results,
                            self.current_match,
                        )
                        .into_iter()
                        .map(|span| {
                            let (ink, background) = source_colors(span.kind, span.emphasis);
                            let mut run = mono_run(span.bytes.len(), ink);
                            run.background_color = background.map(|color| rgb(color).into());
                            run
                        })
                        .collect::<Vec<_>>();
                    if runs.is_empty() {
                        runs.push(mono_run(0, palette().ink));
                    }
                    let line = window
                        .text_system()
                        .shape_line(text, px(SOURCE_SIZE), &runs, None);
                    let source_bounds = Bounds::new(
                        point(bounds.left() + self.gutter_width + self.offset.x, top),
                        size(line.width, px(SOURCE_HEIGHT)),
                    );
                    self.content_width = self
                        .content_width
                        .max(line.width + self.gutter_width + px(scrollbar::GUTTER + 12.));
                    let first = bytes.start == 0;
                    let old = if first {
                        row.old_line.map_or(String::new(), |n| n.to_string())
                    } else {
                        String::new()
                    };
                    let new = if first {
                        row.new_line.map_or(String::new(), |n| n.to_string())
                    } else {
                        String::new()
                    };
                    let mark = if !first {
                        " "
                    } else {
                        match row.kind {
                            DomDiffRowKind::Added => "+",
                            DomDiffRowKind::Removed => "−",
                            DomDiffRowKind::Context => " ",
                        }
                    };
                    let ink = match row.kind {
                        DomDiffRowKind::Added => palette().accent_hover,
                        DomDiffRowKind::Removed => palette().error,
                        DomDiffRowKind::Context => palette().muted,
                    };
                    let gutter: SharedString =
                        format!("{old:>digits$} {new:>digits$}  {mark}").into();
                    let gutter = window.text_system().shape_line(
                        gutter.clone(),
                        px(SOURCE_SIZE),
                        &[mono_run(gutter.len(), ink)],
                        None,
                    );
                    self.painted.push(PaintedLine {
                        segment,
                        bounds: source_bounds,
                        line: line.clone(),
                    });
                    rows.push(FrameRow {
                        bounds: row_bounds,
                        content: FrameContent::Source {
                            segment,
                            source_bounds,
                            line,
                            gutter: Box::new(gutter),
                        },
                    });
                }
                Item::Hunk(hunk) => {
                    let range = &document.hunks[hunk].rows;
                    let first_old = document.rows[range.clone()].iter().find_map(|r| r.old_line);
                    let first_new = document.rows[range.clone()].iter().find_map(|r| r.new_line);
                    let label = format!(
                        "Change group {} · Before {} · After {}",
                        hunk + 1,
                        first_old.map_or("—".into(), |n| n.to_string()),
                        first_new.map_or("—".into(), |n| n.to_string())
                    );
                    rows.push(FrameRow {
                        bounds: row_bounds,
                        content: FrameContent::Label {
                            line: label_line(&label, window),
                            fold: false,
                            active: self.current_hunk == Some(hunk),
                            hitbox: None,
                        },
                    });
                }
                Item::Fold(block) => {
                    let Block::Fold(range) = &document.blocks[block] else {
                        continue;
                    };
                    let label = format!("Show {} unchanged lines", range.len());
                    rows.push(FrameRow {
                        bounds: row_bounds,
                        content: FrameContent::Label {
                            line: label_line(&label, window),
                            fold: true,
                            active: false,
                            hitbox: Some(window.insert_hitbox(
                                row_bounds.intersect(&bounds),
                                HitboxBehavior::Normal,
                            )),
                        },
                    });
                }
            }
        }
        if let Some(pos) = self.pending_reveal.take() {
            let segment = document.segment_at(pos);
            if let Some(painted) = self.painted.iter().find(|line| line.segment == segment) {
                let bytes = &document.segments[segment].1;
                let x = painted.bounds.left() + painted.line.x_for_index(pos.byte - bytes.start);
                let left = bounds.left() + self.gutter_width;
                let right = (bounds.right() - px(scrollbar::GUTTER + 8.)).max(left);
                let previous = self.offset.x;
                self.offset.x += if x < left {
                    left - x
                } else if x > right {
                    right - x
                } else {
                    px(0.)
                };
                self.clamp_offset();
                let delta = self.offset.x - previous;
                for row in &mut rows {
                    if let FrameContent::Source { source_bounds, .. } = &mut row.content {
                        source_bounds.origin.x += delta;
                    }
                }
                for line in &mut self.painted {
                    line.bounds.origin.x += delta;
                }
            }
        }
        if let Some(pointer) = self.selection_pointer.filter(|_| self.autoscrolling)
            && let Some(position) = self.position_for_pointer(pointer)
        {
            self.selection.caret = position;
            self.sync_focus(cx);
        }
        rows
    }

    fn paint_frame(
        &mut self,
        bounds: Bounds<Pixels>,
        rows: Vec<FrameRow>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(document) = self.document().cloned() else {
            return;
        };
        let (selected_start, selected_end) = self.selection.ordered();
        let focused = self.enabled && self.source_focus.is_focused(window);
        window.with_content_mask(Some(ContentMask { bounds }), |window| {
            for row in rows {
                match row.content {
                    FrameContent::Source {
                        segment,
                        source_bounds,
                        line,
                        gutter,
                    } => {
                        let (row_index, bytes) = &document.segments[segment];
                        let background = match document.rows[*row_index].kind {
                            DomDiffRowKind::Added => Some(dom_palette().added),
                            DomDiffRowKind::Removed => Some(dom_palette().removed),
                            DomDiffRowKind::Context => None,
                        };
                        if let Some(background) = background {
                            window.paint_quad(fill(
                                Bounds::new(
                                    row.bounds.origin,
                                    size(
                                        (row.bounds.size.width - px(scrollbar::GUTTER)).max(px(0.)),
                                        row.bounds.size.height,
                                    ),
                                ),
                                rgb(background),
                            ));
                        }
                        let source_clip = Bounds::new(
                            point(bounds.left() + self.gutter_width, bounds.top()),
                            size(
                                (bounds.size.width - self.gutter_width - px(scrollbar::GUTTER))
                                    .max(px(0.)),
                                (bounds.size.height - px(scrollbar::GUTTER)).max(px(0.)),
                            ),
                        );
                        window.with_content_mask(
                            Some(ContentMask {
                                bounds: source_clip,
                            }),
                            |window| {
                                let _ = line.paint_background(
                                    source_bounds.origin,
                                    px(SOURCE_HEIGHT),
                                    window,
                                    cx,
                                );
                                if *row_index >= selected_start.row
                                    && *row_index <= selected_end.row
                                {
                                    let from = if *row_index == selected_start.row {
                                        selected_start.byte
                                    } else {
                                        0
                                    };
                                    let to = if *row_index == selected_end.row {
                                        selected_end.byte
                                    } else {
                                        document.rows[*row_index].text.len()
                                    };
                                    let start = from.max(bytes.start).min(bytes.end) - bytes.start;
                                    let end = to.min(bytes.end).max(bytes.start) - bytes.start;
                                    // A selected newline on an empty/end-of-line
                                    // segment still needs a visible hit of ink.
                                    if start == end
                                        && *row_index < selected_end.row
                                        && to >= bytes.end
                                    {
                                        let x = line.x_for_index(start);
                                        let width = (line.x_for_index(end) - x).max(px(3.));
                                        window.paint_quad(fill(
                                            Bounds::new(
                                                point(source_bounds.left() + x, row.bounds.top()),
                                                size(width, px(SOURCE_HEIGHT)),
                                            ),
                                            rgb(dom_palette().selection),
                                        ));
                                    }
                                }
                                if focused
                                    && self.selection.is_empty()
                                    && self.selection.caret.row == *row_index
                                    && document.segment_at(self.selection.caret) == segment
                                {
                                    let x = source_bounds.left()
                                        + line.x_for_index(self.selection.caret.byte - bytes.start);
                                    let on_current_match = self
                                        .current_match
                                        .and_then(|index| self.results.matches.get(index))
                                        .is_some_and(|found| {
                                            found.row == *row_index
                                                && found.bytes.contains(&self.selection.caret.byte)
                                        });
                                    window.paint_quad(fill(
                                        Bounds::new(
                                            point(x, row.bounds.top()),
                                            size(px(1.), px(SOURCE_HEIGHT)),
                                        ),
                                        rgb(if on_current_match {
                                            palette().on_accent
                                        } else {
                                            palette().accent
                                        }),
                                    ));
                                }
                                let _ =
                                    line.paint(source_bounds.origin, px(SOURCE_HEIGHT), window, cx);
                            },
                        );
                        let gutter_bounds = Bounds::new(
                            point(bounds.left(), row.bounds.top()),
                            size(self.gutter_width - px(8.), px(SOURCE_HEIGHT)),
                        );
                        window.with_content_mask(
                            Some(ContentMask {
                                bounds: gutter_bounds,
                            }),
                            |window| {
                                let _ = gutter.paint(
                                    point(bounds.left() + px(8.), row.bounds.top()),
                                    px(SOURCE_HEIGHT),
                                    window,
                                    cx,
                                );
                            },
                        );
                    }
                    FrameContent::Label {
                        line,
                        fold,
                        active,
                        hitbox,
                    } => {
                        if let Some(hitbox) = hitbox {
                            window.set_cursor_style(CursorStyle::PointingHand, &hitbox);
                        }
                        window.paint_quad(fill(
                            row.bounds,
                            rgb(if active {
                                palette().selected
                            } else {
                                palette().action_surface
                            }),
                        ));
                        let _ = line.paint(
                            point(
                                bounds.left() + px(if fold { 32. } else { 12. }),
                                row.bounds.top() + px(10.),
                            ),
                            px(CONTROL_LINE_HEIGHT),
                            window,
                            cx,
                        );
                        if fold {
                            let mut path = PathBuilder::stroke(px(1.8));
                            let origin = point(bounds.left() + px(12.), row.bounds.top() + px(12.));
                            path.move_to(origin + point(px(2.), px(2.)));
                            path.line_to(origin + point(px(7.), px(7.)));
                            path.line_to(origin + point(px(2.), px(12.)));
                            if let Ok(path) = path.build() {
                                window.paint_path(path, rgb(palette().ink));
                            }
                        }
                    }
                }
            }
            if focused {
                window.paint_quad(quad(
                    bounds,
                    px(0.),
                    transparent_black(),
                    px(1.),
                    rgb(palette().accent),
                    BorderStyle::Solid,
                ));
            }
        });
        let entity = cx.entity();
        if self.autoscrolling && self.selecting {
            window.request_animation_frame();
        }
        window.on_mouse_event(move |event: &MouseMoveEvent, phase, window, cx| {
            if phase == DispatchPhase::Capture && entity.read(cx).selecting {
                entity.update(cx, |this, cx| this.mouse_move(event, window, cx));
            }
        });
        let entity = cx.entity();
        window.on_mouse_event(move |event: &MouseUpEvent, phase, _, cx| {
            if phase == DispatchPhase::Capture && event.button == MouseButton::Left {
                entity.update(cx, |this, _| {
                    this.selecting = false;
                    this.selection_pointer = None;
                    this.autoscrolling = false;
                });
            }
        });
    }

    fn summary(&self, document: &DomDiffDocument) -> String {
        match document.mode {
            DomDiffMode::Changed if document.hunks.is_empty() => {
                "DOM snapshots changed. The detailed diff is unavailable.".into()
            }
            DomDiffMode::Changed => format!(
                "{}{} added {} · {} removed {} · {} change {}",
                if document.notice.is_some() {
                    "Displayed: "
                } else {
                    ""
                },
                document.added,
                if document.added == 1 { "line" } else { "lines" },
                document.removed,
                if document.removed == 1 {
                    "line"
                } else {
                    "lines"
                },
                document.hunks.len(),
                if document.hunks.len() == 1 {
                    "group"
                } else {
                    "groups"
                },
            ),
            DomDiffMode::Unchanged => "DOM snapshots are identical.".into(),
            DomDiffMode::BeforeOnly => "Before DOM snapshot only — no comparison.".into(),
            DomDiffMode::AfterOnly => "Captured DOM snapshot — no comparison.".into(),
        }
    }
    fn search_status(&self) -> String {
        if self.searching {
            return "Searching DOM source…".into();
        }
        if self.query.is_empty() {
            return "Case-sensitive search includes collapsed lines.".into();
        }
        if self.results.matches.is_empty() {
            return "No matches. Try different text or clear search.".into();
        }
        let qualifier = if self.results.limited { "first " } else { "" };
        format!(
            "Match {} of {qualifier}{} · Includes collapsed lines",
            self.current_match.map_or(0, |n| n + 1),
            self.results.matches.len()
        )
    }
}

struct FrameRow {
    bounds: Bounds<Pixels>,
    content: FrameContent,
}
enum FrameContent {
    Source {
        segment: usize,
        source_bounds: Bounds<Pixels>,
        line: ShapedLine,
        gutter: Box<ShapedLine>,
    },
    Label {
        line: ShapedLine,
        fold: bool,
        active: bool,
        hitbox: Option<Hitbox>,
    },
}
fn label_line(label: &str, window: &mut Window) -> ShapedLine {
    let style = window.text_style();
    window.text_system().shape_line(
        label.to_owned().into(),
        px(CONTROL_TEXT_SIZE),
        &[TextRun {
            color: rgb(palette().ink).into(),
            ..style.to_run(label.len())
        }],
        None,
    )
}

impl Render for DomDiff {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        self.sync_focus(cx);
        if self.hints.is_none() {
            self.hints = Some(toolbar::HintInput::new(window, cx));
        }
        if !self.enabled
            && (self.source_focus.is_focused(window)
                || self.state_focus.is_focused(window)
                || self.search_focus.is_focused(window)
                || self.controls.iter().any(|f| f.is_focused(window)))
        {
            window.blur();
        }
        let ready = self.document().cloned();
        let root = div()
            .id("dom-diff")
            .debug_selector(|| "dom-diff".into())
            .key_context("DomDiff")
            .relative()
            .size_full()
            .min_h_0()
            .min_w_0()
            .flex()
            .flex_col()
            .bg(rgb(palette().surface))
            .border_1()
            .border_color(rgb(palette().line))
            .font_family(FONT)
            .text_size(px(CONTROL_TEXT_SIZE))
            .line_height(px(CONTROL_LINE_HEIGHT))
            .text_color(rgb(palette().ink))
            .child(self.hints.as_ref().unwrap().tracker())
            .on_action(cx.listener(|this, _: &Activate, window, cx| {
                if let Some(control) = CONTROLS
                    .into_iter()
                    .find(|&c| this.handle(c).is_focused(window))
                {
                    this.activate(control, window, cx);
                }
            }))
            .on_action(
                cx.listener(|this, _: &CopySelection, window, cx| this.copy_selection(window, cx)),
            )
            .on_action(cx.listener(|this, _: &ClearSelection, window, cx| {
                if this.allowed(window, cx) {
                    this.selection.anchor = this.selection.caret;
                    this.sync_focus(cx);
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|this, _: &Find, window, cx| this.focus_search(window, cx)))
            .on_action(cx.listener(|this, _: &SelectAll, window, cx| {
                if !this.allowed(window, cx) {
                    return;
                }
                if let Some(document) = this.document().filter(|d| !d.rows.is_empty()) {
                    let row = document.rows.len() - 1;
                    this.selection = Selection {
                        anchor: SourcePosition::default(),
                        caret: SourcePosition {
                            row,
                            byte: document.rows[row].text.len(),
                        },
                    };
                    this.copied = false;
                    this.sync_focus(cx);
                    cx.notify();
                }
            }))
            .on_action(cx.listener(|this, _: &NextMatch, window, cx| {
                if this.allowed(window, cx) {
                    this.navigate_match(true, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &PreviousMatch, window, cx| {
                if this.allowed(window, cx) {
                    this.navigate_match(false, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &NextHunk, window, cx| {
                if this.allowed(window, cx) {
                    this.navigate_hunk(true, cx);
                }
            }))
            .on_action(cx.listener(|this, _: &PreviousHunk, window, cx| {
                if this.allowed(window, cx) {
                    this.navigate_hunk(false, cx);
                }
            }));
        if let Some(document) = ready {
            // Short desktop windows must retain readable source rows. The tools
            // keep their 40px targets and a separate, visible reading scroller;
            // keyboard focus reveals the real search/change row, not a proxy.
            let short = f32::from(window.viewport_size().height) <= 640.;
            let tools_focus_row = if self.search_focus.is_focused(window)
                || self.controls[..3]
                    .iter()
                    .any(|focus| focus.is_focused(window))
            {
                Some(0)
            } else if self.controls[3..]
                .iter()
                .any(|focus| focus.is_focused(window))
            {
                Some(2)
            } else {
                None
            };
            if short
                && tools_focus_row != self.tools_focus_row
                && let Some(row) = tools_focus_row
            {
                self.tools_scroll.scroll_to_item(row);
            } else if !short {
                self.tools_scroll.set_offset(point(px(0.), px(0.)));
            }
            self.tools_focus_row = tools_focus_row;
            let legend = || {
                div()
                    .flex_shrink_0()
                    .px(px(8.))
                    .py(px(4.))
                    .border_t_1()
                    .border_b_1()
                    .border_color(rgb(palette().line))
                    .text_color(rgb(palette().muted))
                    .text_size(px(HELP_TEXT_SIZE))
                    .line_height(px(HELP_LINE_HEIGHT))
                    .child("Before / After line numbers · + Added · − Removed · Unmarked: context")
            };
            let tools = div()
                .id("dom-diff-tools")
                .debug_selector(|| "dom-diff-tools".into())
                .flex_shrink_0()
                .p(px(8.))
                .flex()
                .flex_col()
                .gap(px(8.))
                .when(short, |el| {
                    el.max_h(px(46.))
                        .py(px(3.))
                        .overflow_y_scroll()
                        .track_scroll(&self.tools_scroll)
                })
                .child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .flex_wrap()
                        .items_center()
                        .gap(px(8.))
                        .child(div().flex_1().min_w(px(180.)).child(self.search.clone()))
                        .child(self.control(Control::Clear, "dom-diff-clear", "Clear search", cx))
                        .child(self.control(
                            Control::PreviousMatch,
                            "dom-diff-previous-match",
                            "Previous match",
                            cx,
                        ))
                        .child(self.control(
                            Control::NextMatch,
                            "dom-diff-next-match",
                            "Next match",
                            cx,
                        )),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .text_size(px(HELP_TEXT_SIZE))
                        .line_height(px(HELP_LINE_HEIGHT))
                        .text_color(rgb(palette().muted))
                        .child(self.search_status()),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.))
                        .items_center()
                        .child(div().text_color(rgb(palette().muted)).child("Changes"))
                        .child(self.control(
                            Control::PreviousHunk,
                            "dom-diff-previous-change",
                            "Previous change",
                            cx,
                        ))
                        .child(self.control(
                            Control::NextHunk,
                            "dom-diff-next-change",
                            "Next change",
                            cx,
                        ))
                        .child(self.control(
                            Control::Context,
                            "dom-diff-context",
                            "Unchanged lines",
                            cx,
                        ))
                        .child(self.control(Control::Copy, "dom-diff-copy", "Copy selection", cx)),
                )
                .child(
                    div()
                        .flex_shrink_0()
                        .flex()
                        .flex_wrap()
                        .gap(px(8.))
                        .child(self.summary(&document))
                        .when(self.copied, |el| {
                            el.child(
                                div()
                                    .text_color(rgb(palette().accent_hover))
                                    .child("Copied"),
                            )
                        }),
                )
                .when_some(document.notice.clone(), |el, notice| {
                    el.child(scrollbar::scrollable(
                        "dom-diff-notice-scrollbar",
                        div()
                            .id("dom-diff-notice")
                            .max_h(px(72.))
                            .overflow_y_scroll()
                            .pr(px(scrollbar::GUTTER))
                            .text_color(rgb(palette().warning))
                            .text_size(px(HELP_TEXT_SIZE))
                            .line_height(px(HELP_LINE_HEIGHT))
                            .child(notice.to_string()),
                    ))
                })
                .when(short, |el| el.child(legend()));
            root.child(scrollbar::vertical("dom-diff-tools-scrollbar", &self.tools_scroll, tools))
            .when(!short, |el| el.child(legend()))
            .child(div().flex_1().min_h_0().min_w_0().when(!document.rows.is_empty(), |el| el.child(self.source_surface(cx))).when(document.rows.is_empty(), |el| el.p(px(24.)).child(if document.mode == DomDiffMode::Changed { "No diff rows are available. Inspect the complete captured originals in Details." } else { "This DOM snapshot contains no source lines." })))
             .child(div().flex_shrink_0().px(px(8.)).py(px(4.)).when(short, |el| el.py(px(0.))).border_t_1().border_color(rgb(palette().line)).text_size(px(HELP_TEXT_SIZE)).line_height(px(HELP_LINE_HEIGHT)).text_color(rgb(palette().muted)).child(if short { format!("Read-only · Wrap: {WRAP_COLUMNS} · Copy includes collapsed context") } else { format!("Read-only · Wrap: {WRAP_COLUMNS} characters · Selection includes collapsed context") }))
            .into_any_element()
        } else {
            // Loading/error bodies use their own bounded reading owner. No stale
            // source, image tools or controls from the previous target are mounted.
            if self.source_focus.is_focused(window)
                || self.search_focus.is_focused(window)
                || self.controls.iter().any(|f| f.is_focused(window))
                || (!matches!(self.state, LoadState::Failed(_))
                    && self.state_focus.is_focused(window))
            {
                window.blur();
            }
            let body = match &self.state {
                LoadState::Loading => controls::loading(
                    "Loading DOM snapshot…",
                    "Reading saved source and its comparison. No website is visited.",
                )
                .into_any_element(),
                LoadState::Failed(error) => controls::note(
                    "DOM snapshot unavailable",
                    error.to_string(),
                    controls::Tone::Error,
                )
                .into_any_element(),
                _ => controls::note(
                    "No DOM snapshot selected",
                    "Choose a recorded DOM snapshot to inspect its source.",
                    controls::Tone::Neutral,
                )
                .into_any_element(),
            };
            root.child(scrollbar::vertical(
                "dom-diff-state-scrollbar",
                &self.state_scroll,
                div()
                    .id("dom-diff-state")
                    .debug_selector(|| "dom-diff-state".into())
                    .flex_1()
                    .min_h_0()
                    .overflow_y_scroll()
                    .track_scroll(&self.state_scroll)
                    .when(
                        self.enabled && matches!(self.state, LoadState::Failed(_)),
                        |el| el.track_focus(&self.state_focus),
                    )
                    .border_1()
                    .border_color(transparent_black())
                    .focus(|s| s.border_color(rgb(palette().accent)))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            if this.allowed(window, cx)
                                && matches!(this.state, LoadState::Failed(_))
                            {
                                window.focus(&this.state_focus);
                            }
                        }),
                    )
                    .on_key_down(
                        cx.listener(|this, event, window, cx| this.state_key(event, window, cx)),
                    )
                    .p(px(24.))
                    .child(body),
            ))
            .into_any_element()
        }
    }
}

#[cfg(test)]
#[path = "dom_diff_tests.rs"]
mod tests;
