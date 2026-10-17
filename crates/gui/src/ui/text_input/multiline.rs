// SPDX-License-Identifier: MPL-2.0
//! Note-only wrapped native editing. All indices are UTF-8 boundaries; the
//! owning TextInput converts the OS/IME UTF-16 protocol at its boundary.
use super::*;
use gpui::{ScrollWheelEvent, TextAlign, WrappedLine};

pub const EDITOR_HEIGHT: f32 = 138.; // six 20px lines plus padding and border

#[derive(Clone)]
struct Row {
    range: Range<usize>,
    paragraph: usize,
    paragraph_start: usize,
    x: Pixels,
}
#[derive(Clone)]
pub(super) struct NoteLayout {
    text: SharedString,
    paragraphs: Vec<WrappedLine>,
    rows: Vec<Row>,
}
impl NoteLayout {
    fn shape(input: &TextInput, width: Pixels, w: &Window) -> Self {
        let style = w.text_style();
        let runs = input.text_runs(TextRun {
            len: input.content.len(),
            font: style.font(),
            color: style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        });
        // CR remains an exact stored byte (including CRLF clipboard text), but
        // does not draw a control glyph. No content normalization on editing/save.
        let display: SharedString = input.content.replace('\r', " ").into();
        let paragraphs = w
            .text_system()
            .shape_text(
                display,
                px(CONTROL_TEXT_SIZE),
                &runs,
                Some(width.max(px(1.))),
                None,
            )
            .expect("native note shaping")
            .into_iter()
            .collect::<Vec<_>>();
        let mut rows = Vec::new();
        let mut offset = 0;
        for (paragraph, line) in paragraphs.iter().enumerate() {
            let starts = std::iter::once(0)
                .chain(
                    line.wrap_boundaries
                        .iter()
                        .map(|b| line.runs()[b.run_ix].glyphs[b.glyph_ix].index),
                )
                .collect::<Vec<_>>();
            for (i, start) in starts.iter().copied().enumerate() {
                let end = starts.get(i + 1).copied().unwrap_or(line.len());
                rows.push(Row {
                    range: offset + start..offset + end,
                    paragraph,
                    paragraph_start: offset,
                    x: line.unwrapped_layout.x_for_index(start),
                });
            }
            offset += line.len() + 1;
        }
        Self {
            text: input.content.clone(),
            paragraphs,
            rows,
        }
    }
    fn row_for(&self, n: usize, upstream: bool) -> usize {
        let matches = |r: &Row| r.range.start <= n && n <= r.range.end;
        // A soft wrap has two visual positions for one byte boundary. End uses
        // upstream; Home/Down/pointer-at-column-zero use downstream. Hard line
        // breaks have distinct byte boundaries and need no affinity choice.
        (if upstream {
            self.rows.iter().position(matches)
        } else {
            self.rows.iter().rposition(matches)
        })
        .unwrap_or(self.rows.len().saturating_sub(1))
    }
    fn position(&self, n: usize, upstream: bool) -> Point<Pixels> {
        let row = self.row_for(n, upstream);
        let r = &self.rows[row];
        point(
            self.paragraphs[r.paragraph]
                .unwrapped_layout
                .x_for_index(n - r.paragraph_start)
                - r.x,
            px(row as f32 * CONTROL_LINE_HEIGHT),
        )
    }
    fn index(&self, p: Point<Pixels>) -> usize {
        let row = self.row_at_y(p.y);
        let r = &self.rows[row];
        let line = &self.paragraphs[r.paragraph];
        let n = r.paragraph_start + line.unwrapped_layout.closest_index_for_x(p.x + r.x);
        n.clamp(r.range.start, r.range.end)
    }
    fn row_at_y(&self, y: Pixels) -> usize {
        ((f32::from(y).max(0.) / CONTROL_LINE_HEIGHT) as usize)
            .min(self.rows.len().saturating_sub(1))
    }
    fn height(&self) -> Pixels {
        px(self.rows.len() as f32 * CONTROL_LINE_HEIGHT)
    }
}

pub(super) struct MultilineState {
    layout: Option<NoteLayout>,
    scroll_y: Pixels,
    reveal: bool,
    preferred_x: Option<Pixels>,
    upstream: bool,
}
impl Default for MultilineState {
    fn default() -> Self {
        Self {
            layout: None,
            scroll_y: px(0.),
            reveal: true,
            preferred_x: None,
            upstream: false,
        }
    }
}
impl MultilineState {
    fn current(&self, text: &str) -> Option<&NoteLayout> {
        self.layout.as_ref().filter(|l| l.text.as_ref() == text)
    }
    pub(super) fn index_for_position(
        &self,
        text: &str,
        bounds: Option<Bounds<Pixels>>,
        p: Point<Pixels>,
    ) -> Option<usize> {
        let b = bounds?;
        Some(
            self.current(text)?
                .index(point(p.x - b.left(), p.y - b.top() + self.scroll_y)),
        )
    }
    pub(super) fn bounds_for_range(
        &self,
        text: &str,
        r: Range<usize>,
        b: Bounds<Pixels>,
    ) -> Option<Bounds<Pixels>> {
        let l = self.current(text)?;
        let start = l.position(r.start, self.upstream);
        let end = l.position(r.end, self.upstream);
        let x = start.x.min((b.size.width - px(2.)).max(px(0.)));
        Some(Bounds::new(
            point(b.left() + x, b.top() + start.y - self.scroll_y),
            size(
                if start.y == end.y {
                    (end.x - x).max(px(2.))
                } else {
                    (b.size.width - x).max(px(2.))
                },
                px(CONTROL_LINE_HEIGHT),
            ),
        ))
    }
}

impl TextInput {
    pub fn multiline_scroll_offset(&self) -> Option<Pixels> {
        self.multiline.as_ref().map(|m| m.scroll_y)
    }
    pub fn multiline_overflows(&self) -> bool {
        self.multiline
            .as_ref()
            .and_then(|m| m.current(&self.content))
            .zip(self.bounds)
            .is_some_and(|(l, b)| l.height() > b.size.height)
    }
    pub(super) fn reveal_multiline_caret(&mut self) {
        if let Some(mode) = &mut self.multiline {
            mode.reveal = true;
            mode.preferred_x = None;
            mode.upstream = false;
        }
    }
    pub(super) fn line_edge(&self, end: bool) -> usize {
        if let Some(l) = self
            .multiline
            .as_ref()
            .and_then(|m| m.current(&self.content))
        {
            let r =
                &l.rows[l.row_for(self.caret(), self.multiline.as_ref().unwrap().upstream)].range;
            if end { r.end } else { r.start }
        } else if end {
            self.content.len()
        } else {
            0
        }
    }
    pub(super) fn line_end_affinity(&mut self) {
        if let Some(mode) = &mut self.multiline {
            mode.upstream = true;
        }
    }
    pub(super) fn pointer_affinity(&mut self, p: Point<Pixels>) {
        let caret = self.caret();
        if let Some(mode) = &mut self.multiline
            && let Some(b) = self.bounds
            && let Some(l) = mode.current(&self.content)
        {
            let r = &l.rows[l.row_at_y(p.y - b.top() + mode.scroll_y)].range;
            mode.upstream = caret == r.end && caret != r.start;
        }
    }
    pub(super) fn accept_replacement(
        &mut self,
        r: &Range<usize>,
        text: &str,
        cx: &mut Context<Self>,
    ) -> bool {
        if r.start > r.end {
            return false;
        }
        let next = self.content.len() - (r.end - r.start) + text.len();
        let note = self.multiline.is_some();
        let too_large = next > self.max_bytes && !(note && next < self.content.len());
        let controls = text
            .chars()
            .any(|c| c.is_control() && !(note && matches!(c, '\n' | '\r' | '\t')));
        if too_large || controls {
            if note {
                self.correction = Some(
                    if too_large {
                        "This note is too long. Shorten it or paste less. Nothing was added."
                    } else {
                        "This text could not be added. Try copying it as plain text."
                    }
                    .into(),
                );
                self.touched = true;
                cx.notify();
            }
            return false;
        }
        true
    }
    fn vertical(&mut self, rows: isize, select: bool, cx: &mut Context<Self>) {
        if !self.input_allowed() {
            return;
        }
        let Some(mode) = self.multiline.as_ref() else {
            return;
        };
        let Some(l) = mode.current(&self.content) else {
            return;
        };
        let p = l.position(self.caret(), mode.upstream);
        let x = mode.preferred_x.unwrap_or(p.x);
        let row = (l.row_for(self.caret(), mode.upstream) as isize + rows)
            .clamp(0, l.rows.len() as isize - 1);
        let n = l.index(point(x, px(row as f32 * CONTROL_LINE_HEIGHT + 1.)));
        let upstream = n == l.rows[row as usize].range.end && n != l.rows[row as usize].range.start;
        if select {
            self.select_to(n, cx);
        } else {
            self.move_to(n, cx);
        }
        self.multiline.as_mut().unwrap().preferred_x = Some(x);
        self.multiline.as_mut().unwrap().upstream = upstream;
    }
    fn page_rows(&self) -> isize {
        self.bounds.map_or(5, |b| {
            (f32::from(b.size.height) / CONTROL_LINE_HEIGHT)
                .floor()
                .max(1.) as isize
        })
    }
    pub(super) fn up(&mut self, _: &Up, _: &mut Window, cx: &mut Context<Self>) {
        self.vertical(-1, false, cx);
    }
    pub(super) fn down(&mut self, _: &Down, _: &mut Window, cx: &mut Context<Self>) {
        self.vertical(1, false, cx);
    }
    pub(super) fn select_up(&mut self, _: &SelectUp, _: &mut Window, cx: &mut Context<Self>) {
        self.vertical(-1, true, cx);
    }
    pub(super) fn select_down(&mut self, _: &SelectDown, _: &mut Window, cx: &mut Context<Self>) {
        self.vertical(1, true, cx);
    }
    pub(super) fn page_up(&mut self, _: &PageUp, _: &mut Window, cx: &mut Context<Self>) {
        self.vertical(-self.page_rows(), false, cx);
    }
    pub(super) fn page_down(&mut self, _: &PageDown, _: &mut Window, cx: &mut Context<Self>) {
        self.vertical(self.page_rows(), false, cx);
    }
    pub(super) fn select_page_up(
        &mut self,
        _: &SelectPageUp,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.vertical(-self.page_rows(), true, cx);
    }
    pub(super) fn select_page_down(
        &mut self,
        _: &SelectPageDown,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.vertical(self.page_rows(), true, cx);
    }
    pub(super) fn document_home(
        &mut self,
        _: &DocumentHome,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.move_to(0, cx);
    }
    pub(super) fn document_end(&mut self, _: &DocumentEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.content.len(), cx);
    }
    pub(super) fn select_document_home(
        &mut self,
        _: &SelectDocumentHome,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(0, cx);
    }
    pub(super) fn select_document_end(
        &mut self,
        _: &SelectDocumentEnd,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        self.select_to(self.content.len(), cx);
    }
    pub(super) fn newline(&mut self, _: &Newline, w: &mut Window, cx: &mut Context<Self>) {
        self.replace_text_in_range(None, "\n", w, cx);
    }
    pub(super) fn scroll_note(
        &mut self,
        e: &ScrollWheelEvent,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.input_allowed() {
            return;
        }
        let Some(b) = self.bounds else {
            return;
        };
        let Some(mode) = self.multiline.as_mut() else {
            return;
        };
        let Some(l) = mode.current(&self.content) else {
            return;
        };
        let max = (l.height() - b.size.height).max(px(0.));
        let next =
            (mode.scroll_y - e.delta.pixel_delta(px(CONTROL_LINE_HEIGHT)).y).clamp(px(0.), max);
        if next != mode.scroll_y {
            mode.scroll_y = next;
            mode.reveal = false;
            cx.stop_propagation();
            cx.notify();
        }
    }
}

pub(super) fn element(entity: Entity<TextInput>) -> impl IntoElement {
    use crate::ui::scrollbar::{self, ScrollMetrics};
    let read = entity.clone();
    let change = entity.clone();
    scrollbar::both(
        "note-scrollbar",
        MultilineElement(entity),
        move |bounds, cx| {
            let input = read.read(cx);
            let mode = input.multiline.as_ref().unwrap();
            let height = mode.layout.as_ref().map_or(px(0.), NoteLayout::height);
            ScrollMetrics {
                viewport: bounds,
                overflow: size(px(0.), (height - bounds.size.height).max(px(0.))),
                offset: point(px(0.), -mode.scroll_y),
            }
        },
        move |offset, _, cx| {
            change.update(cx, |input, cx| {
                if input.enabled {
                    let mode = input.multiline.as_mut().unwrap();
                    mode.scroll_y = -offset.y;
                    mode.reveal = false;
                    cx.notify();
                }
            });
        },
    )
}

struct MultilineElement(Entity<TextInput>);
impl IntoElement for MultilineElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for MultilineElement {
    type RequestLayoutState = ();
    type PrepaintState = NoteLayout;
    fn id(&self) -> Option<ElementId> {
        None
    }
    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }
    fn request_layout(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        w: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, ()) {
        let mut s = Style::default();
        s.size.width = relative(1.).into();
        s.size.height = relative(1.).into();
        (w.request_layout(s, [], cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        b: Bounds<Pixels>,
        _: &mut (),
        w: &mut Window,
        cx: &mut App,
    ) -> NoteLayout {
        // Leave an always-reserved tiny gutter, so scrollbar appearance never
        // changes wrapping, pointer mapping or caret geometry.
        let l = NoteLayout::shape(
            self.0.read(cx),
            b.size.width - px(crate::ui::scrollbar::GUTTER),
            w,
        );
        self.0.update(cx, |input, _| {
            let p = l.position(input.caret(), input.multiline.as_ref().unwrap().upstream);
            // Reflow should keep an already-visible caret in view, without
            // snapping a manually scrolled reading position back to the caret.
            let reflow_visible_caret = input.bounds.is_some_and(|old| {
                let mode = input.multiline.as_ref().unwrap();
                mode.current(&input.content).is_some_and(|layout| {
                    let y = layout.position(input.caret(), mode.upstream).y;
                    old.size != b.size
                        && y >= mode.scroll_y
                        && y + px(CONTROL_LINE_HEIGHT) <= mode.scroll_y + old.size.height
                })
            });
            let mode = input.multiline.as_mut().unwrap();
            if mode.reveal || reflow_visible_caret {
                if p.y < mode.scroll_y {
                    mode.scroll_y = p.y;
                }
                if p.y + px(CONTROL_LINE_HEIGHT) > mode.scroll_y + b.size.height {
                    mode.scroll_y = p.y + px(CONTROL_LINE_HEIGHT) - b.size.height;
                }
                mode.reveal = false;
            }
            mode.scroll_y = mode
                .scroll_y
                .clamp(px(0.), (l.height() - b.size.height).max(px(0.)));
            mode.layout = Some(l.clone());
        });
        l
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        b: Bounds<Pixels>,
        _: &mut (),
        l: &mut NoteLayout,
        w: &mut Window,
        cx: &mut App,
    ) {
        let input = self.0.read(cx);
        let focus = input.focus.clone();
        let selected = input.selection.clone();
        let caret = input.caret();
        let scroll = input.multiline.as_ref().unwrap().scroll_y;
        let upstream = input.multiline.as_ref().unwrap().upstream;
        if input.enabled {
            w.handle_input(&focus, ElementInputHandler::new(b, self.0.clone()), cx);
            let entity = self.0.clone();
            w.on_mouse_event(move |e: &MouseMoveEvent, phase, w, cx| {
                if phase == DispatchPhase::Capture && entity.read(cx).is_selecting {
                    entity.update(cx, |input, cx| input.on_mouse_move(e, w, cx));
                }
            });
        }
        let focused = input.enabled && focus.is_focused(w);
        w.with_content_mask(Some(ContentMask { bounds: b }), |w| {
            for (row, r) in l.rows.iter().enumerate() {
                let y = b.top() + px(row as f32 * CONTROL_LINE_HEIGHT) - scroll;
                if y + px(CONTROL_LINE_HEIGHT) < b.top() || y >= b.bottom() {
                    continue;
                }
                if focused
                    && !selected.is_empty()
                    && selected.start <= r.range.end
                    && selected.end > r.range.start
                {
                    let line = &l.paragraphs[r.paragraph].unwrapped_layout;
                    let start = selected.start.max(r.range.start);
                    let end = selected.end.min(r.range.end);
                    let x = line.x_for_index(start - r.paragraph_start) - r.x;
                    let right = line.x_for_index(end - r.paragraph_start) - r.x;
                    w.paint_quad(fill(
                        Bounds::new(
                            point(b.left() + x, y),
                            size((right - x).max(px(4.)), px(CONTROL_LINE_HEIGHT)),
                        ),
                        rgb(palette().selected),
                    ));
                }
            }
            let mut y = b.top() - scroll;
            for line in &l.paragraphs {
                let _ = line.paint(
                    point(b.left(), y),
                    px(CONTROL_LINE_HEIGHT),
                    TextAlign::Left,
                    None,
                    w,
                    cx,
                );
                y += line.size(px(CONTROL_LINE_HEIGHT)).height;
            }
            if focused {
                let p = l.position(caret, upstream);
                w.paint_quad(fill(
                    Bounds::new(
                        point(
                            b.left() + p.x.min(b.size.width - px(2.)),
                            b.top() + p.y - scroll,
                        ),
                        size(px(2.), px(CONTROL_LINE_HEIGHT)),
                    ),
                    rgb(palette().accent),
                ));
            }
        });
        self.0.update(cx, |input, _| {
            input.bounds = Some(b);
            input.multiline.as_mut().unwrap().layout = Some(l.clone());
        });
    }
}

pub(super) fn bind(cx: &mut App) {
    cx.bind_keys([
        KeyBinding::new("up", Up, Some("MultilineInput")),
        KeyBinding::new("down", Down, Some("MultilineInput")),
        KeyBinding::new("shift-up", SelectUp, Some("MultilineInput")),
        KeyBinding::new("shift-down", SelectDown, Some("MultilineInput")),
        KeyBinding::new("pageup", PageUp, Some("MultilineInput")),
        KeyBinding::new("pagedown", PageDown, Some("MultilineInput")),
        KeyBinding::new("shift-pageup", SelectPageUp, Some("MultilineInput")),
        KeyBinding::new("shift-pagedown", SelectPageDown, Some("MultilineInput")),
        KeyBinding::new("enter", Newline, Some("MultilineInput")),
    ]);
    for key in ["ctrl", "cmd"] {
        cx.bind_keys([
            KeyBinding::new(&format!("{key}-home"), DocumentHome, Some("MultilineInput")),
            KeyBinding::new(&format!("{key}-end"), DocumentEnd, Some("MultilineInput")),
            KeyBinding::new(
                &format!("{key}-shift-home"),
                SelectDocumentHome,
                Some("MultilineInput"),
            ),
            KeyBinding::new(
                &format!("{key}-shift-end"),
                SelectDocumentEnd,
                Some("MultilineInput"),
            ),
        ]);
    }
}

#[cfg(all(test, feature = "ui-tests"))]
#[path = "note_tests.rs"]
mod tests;
