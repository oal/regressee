// Adapted from GPUI 0.2.2 examples/input.rs (Apache-2.0; Zed contributors).
// SPDX-License-Identifier: Apache-2.0
// Shared native input supports IME composition, selection and clipboard.
use super::{controls::focus_outline, theme::*};
use gpui::{
    App, Bounds, ClipboardItem, ContentMask, Context, CursorStyle, DispatchPhase, Element,
    ElementId, ElementInputHandler, Entity, EntityInputHandler, FocusHandle, GlobalElementId,
    InspectorElementId, KeyBinding, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent,
    MouseUpEvent, Pixels, Point, ShapedLine, SharedString, Style, TextRun, UTF16Selection,
    UnderlineStyle, Window, actions, div, fill, point, prelude::*, px, relative, rgb, size,
};
use std::ops::Range;
mod multiline;
pub use multiline::EDITOR_HEIGHT as REVIEW_NOTE_HEIGHT;
use multiline::{MultilineElement, MultilineState};
actions!(
    entry,
    [
        Backspace,
        Delete,
        Left,
        Right,
        Home,
        End,
        SelectLeft,
        SelectRight,
        SelectHome,
        SelectEnd,
        SelectAll,
        Paste,
        Copy,
        Cut,
        Up,
        Down,
        SelectUp,
        SelectDown,
        PageUp,
        PageDown,
        SelectPageUp,
        SelectPageDown,
        DocumentHome,
        DocumentEnd,
        SelectDocumentHome,
        SelectDocumentEnd,
        Newline
    ]
);
pub struct TextInput {
    pub focus: FocusHandle,
    pub content: SharedString,
    selection: Range<usize>,
    selection_reversed: bool,
    is_selecting: bool,
    marked: Option<Range<usize>>,
    bounds: Option<Bounds<Pixels>>,
    line: Option<ShapedLine>,
    scroll_x: Pixels,
    pub enabled: bool,
    pub max_bytes: usize,
    pub validator: Option<fn(&str) -> Option<&'static str>>,
    pub touched: bool,
    pub correction: Option<String>,
    pub debug_name: Option<&'static str>,
    blur_observer: Option<gpui::Subscription>,
    multiline: Option<MultilineState>,
}
impl TextInput {
    pub fn new(cx: &mut Context<Self>) -> Self {
        Self::with_value("", 0, cx)
    }
    pub fn with_value(value: &str, tab_index: isize, cx: &mut Context<Self>) -> Self {
        Self {
            focus: cx.focus_handle().tab_index(tab_index).tab_stop(true),
            content: value.to_owned().into(),
            selection: value.len()..value.len(),
            selection_reversed: false,
            is_selecting: false,
            marked: None,
            bounds: None,
            line: None,
            scroll_x: px(0.),
            enabled: true,
            max_bytes: 4096,
            validator: None,
            touched: false,
            correction: None,
            debug_name: None,
            blur_observer: None,
            multiline: None,
        }
    }
    pub fn review_note(value: &str, tab_index: isize, cx: &mut Context<Self>) -> Self {
        let mut input = Self::with_value(value, tab_index, cx);
        input.max_bytes = regressee_core::domain::REVIEW_NOTE_MAX_BYTES;
        input.multiline = Some(MultilineState::default());
        input
    }
    pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
        self.content = value.to_owned().into();
        self.selection = value.len()..value.len();
        self.selection_reversed = false;
        self.is_selecting = false;
        self.marked = None;
        self.line = None;
        self.scroll_x = px(0.);
        self.touched = false;
        self.correction = None;
        if let Some(mode) = &mut self.multiline {
            *mode = MultilineState::default();
        }
        cx.notify();
    }
    pub fn validation_error(&self) -> Option<&'static str> {
        if self.marked.is_some() {
            return None;
        }
        self.validator.and_then(|validate| validate(&self.content))
    }
    pub fn composing(&self) -> bool {
        self.marked.is_some()
    }
    /// Last painted input bounds, for form-level keyboard scroll recovery.
    /// The shell checks mounting before using these retained coordinates.
    pub fn rendered_bounds(&self) -> Option<Bounds<Pixels>> {
        self.bounds
    }
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        if !enabled {
            self.is_selecting = false;
        }
        self.focus = self.focus.clone().tab_stop(enabled);
    }
    fn utf16_offset_to_utf8(&self, n: usize) -> usize {
        let mut count = 0;
        for (i, c) in self.content.char_indices() {
            if count >= n {
                return i;
            }
            count += c.len_utf16();
        }
        self.content.len()
    }
    fn to_utf16(&self, n: usize) -> usize {
        self.content[..n].encode_utf16().count()
    }
    fn convert(&self, r: Range<usize>) -> Range<usize> {
        self.utf16_offset_to_utf8(r.start)..self.utf16_offset_to_utf8(r.end)
    }
    fn prior(&self) -> usize {
        self.prior_to(self.selection.start)
    }
    fn prior_to(&self, n: usize) -> usize {
        self.content[..n]
            .char_indices()
            .last()
            .map_or(0, |(i, _)| i)
    }
    fn next(&self) -> usize {
        self.next_to(self.selection.end)
    }
    fn next_to(&self, n: usize) -> usize {
        self.content[n..]
            .chars()
            .next()
            .map_or(self.content.len(), |c| n + c.len_utf8())
    }
    fn caret(&self) -> usize {
        if self.selection_reversed {
            self.selection.start
        } else {
            self.selection.end
        }
    }
    fn move_to(&mut self, n: usize, cx: &mut Context<Self>) {
        self.reveal_multiline_caret();
        self.selection = n..n;
        self.selection_reversed = false;
        cx.notify();
    }
    fn select_to(&mut self, n: usize, cx: &mut Context<Self>) {
        self.reveal_multiline_caret();
        let anchor = if self.selection_reversed {
            self.selection.end
        } else {
            self.selection.start
        };
        self.selection = anchor.min(n)..anchor.max(n);
        self.selection_reversed = n < anchor;
        cx.notify();
    }
    fn select_left(&mut self, _: &SelectLeft, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.prior_to(self.caret()), cx);
    }
    fn select_right(&mut self, _: &SelectRight, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.next_to(self.caret()), cx);
    }
    fn select_home(&mut self, _: &SelectHome, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.line_edge(false), cx);
    }
    fn select_end(&mut self, _: &SelectEnd, _: &mut Window, cx: &mut Context<Self>) {
        self.select_to(self.line_edge(true), cx);
        self.line_end_affinity();
    }
    fn index_for_position(&self, p: Point<Pixels>) -> usize {
        if let Some(mode) = &self.multiline {
            return mode
                .index_for_position(&self.content, self.bounds, p)
                .unwrap_or(self.caret());
        }
        match (&self.line, self.bounds) {
            // A shaped line can be stale between an edit and the next paint.
            (Some(line), Some(bounds)) if line.text == self.content => {
                line.closest_index_for_x(p.x - bounds.left() + self.scroll_x)
            }
            _ => self.caret(),
        }
    }
    fn word_range(&self, n: usize) -> Range<usize> {
        // A dependency-free scalar fallback, not locale-aware word segmentation.
        fn class(c: char) -> u8 {
            if c.is_alphanumeric() || c == '_' {
                0
            } else if c.is_whitespace() {
                1
            } else {
                2
            }
        }
        if self.content.is_empty() {
            return 0..0;
        }
        let n = if n == self.content.len() {
            self.prior_to(n)
        } else {
            n
        };
        let kind = class(self.content[n..].chars().next().unwrap());
        let mut start = n;
        for (i, c) in self.content[..n].char_indices().rev() {
            if class(c) != kind {
                break;
            }
            start = i;
        }
        let mut end = n;
        for c in self.content[n..].chars() {
            if class(c) != kind {
                break;
            }
            end += c.len_utf8();
        }
        start..end
    }
    fn on_mouse_down(&mut self, e: &MouseDownEvent, w: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled {
            return;
        }
        w.focus(&self.focus);
        self.is_selecting = true;
        let n = self.index_for_position(e.position);
        if e.modifiers.shift {
            self.select_to(n, cx);
        } else if e.click_count == 2 {
            self.selection = self.word_range(n);
            self.selection_reversed = false;
            cx.notify();
        } else {
            self.move_to(n, cx);
        }
        self.pointer_affinity(e.position);
    }
    fn on_mouse_move(&mut self, e: &MouseMoveEvent, w: &mut Window, cx: &mut Context<Self>) {
        if self.is_selecting {
            if self.enabled && self.focus.is_focused(w) && e.dragging() {
                self.select_to(self.index_for_position(e.position), cx);
                self.pointer_affinity(e.position);
            } else {
                self.is_selecting = false;
            }
        }
    }
    fn on_mouse_up(&mut self, _: &MouseUpEvent, _: &mut Window, _: &mut Context<Self>) {
        self.is_selecting = false;
    }
    fn text_runs(&self, run: TextRun) -> Vec<TextRun> {
        if let Some(marked) = self.marked.as_ref().filter(|r| !r.is_empty()) {
            vec![
                TextRun {
                    len: marked.start,
                    ..run.clone()
                },
                TextRun {
                    len: marked.end - marked.start,
                    underline: Some(UnderlineStyle {
                        color: Some(run.color),
                        thickness: px(1.),
                        wavy: false,
                    }),
                    ..run.clone()
                },
                TextRun {
                    len: self.content.len() - marked.end,
                    ..run
                },
            ]
            .into_iter()
            .filter(|run| run.len > 0)
            .collect()
        } else {
            vec![run]
        }
    }
    fn backspace(&mut self, _: &Backspace, w: &mut Window, cx: &mut Context<Self>) {
        if self.selection.is_empty() {
            self.selection = self.prior()..self.selection.end;
        }
        self.replace_text_in_range(None, "", w, cx);
    }
    fn delete(&mut self, _: &Delete, w: &mut Window, cx: &mut Context<Self>) {
        if self.selection.is_empty() {
            self.selection = self.selection.start..self.next();
        }
        self.replace_text_in_range(None, "", w, cx);
    }
    fn left(&mut self, _: &Left, _: &mut Window, cx: &mut Context<Self>) {
        let n = if self.selection.is_empty() {
            self.prior()
        } else {
            self.selection.start
        };
        self.move_to(n, cx);
    }
    fn right(&mut self, _: &Right, _: &mut Window, cx: &mut Context<Self>) {
        let n = if self.selection.is_empty() {
            self.next()
        } else {
            self.selection.end
        };
        self.move_to(n, cx);
    }
    fn home(&mut self, _: &Home, _: &mut Window, cx: &mut Context<Self>) {
        self.move_to(self.line_edge(false), cx);
    }
    fn end(&mut self, _: &End, _: &mut Window, cx: &mut Context<Self>) {
        let n = self.line_edge(true);
        self.move_to(n, cx);
        self.line_end_affinity();
    }
    fn cut(&mut self, _: &Cut, w: &mut Window, cx: &mut Context<Self>) {
        self.copy(&Copy, w, cx);
        self.replace_text_in_range(None, "", w, cx);
    }
    fn all(&mut self, _: &SelectAll, _: &mut Window, cx: &mut Context<Self>) {
        self.reveal_multiline_caret();
        self.selection = 0..self.content.len();
        self.selection_reversed = false;
        cx.notify();
    }
    fn paste(&mut self, _: &Paste, w: &mut Window, cx: &mut Context<Self>) {
        if let Some(t) = cx.read_from_clipboard().and_then(|v| v.text()) {
            if self.multiline.is_some() {
                self.replace_text_in_range(None, &t, w, cx);
            } else {
                self.replace_text_in_range(None, &t.replace(['\r', '\n'], " "), w, cx);
            }
        }
    }
    fn copy(&mut self, _: &Copy, _: &mut Window, cx: &mut Context<Self>) {
        cx.write_to_clipboard(ClipboardItem::new_string(
            self.content[self.selection.clone()].into(),
        ));
    }
}
impl EntityInputHandler for TextInput {
    fn text_for_range(
        &mut self,
        r: Range<usize>,
        actual: &mut Option<Range<usize>>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<String> {
        let r = self.convert(r);
        if r.start > r.end {
            return None;
        }
        *actual = Some(self.to_utf16(r.start)..self.to_utf16(r.end));
        Some(self.content[r].into())
    }
    fn selected_text_range(
        &mut self,
        _: bool,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<UTF16Selection> {
        Some(UTF16Selection {
            range: self.to_utf16(self.selection.start)..self.to_utf16(self.selection.end),
            reversed: self.selection_reversed,
        })
    }
    fn marked_text_range(&self, _: &mut Window, _: &mut Context<Self>) -> Option<Range<usize>> {
        self.marked
            .clone()
            .map(|r| self.to_utf16(r.start)..self.to_utf16(r.end))
    }
    fn unmark_text(&mut self, _: &mut Window, cx: &mut Context<Self>) {
        self.marked = None;
        cx.notify();
    }
    fn replace_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.enabled {
            return;
        }
        let r = r
            .map(|r| self.convert(r))
            .or(self.marked.clone())
            .unwrap_or(self.selection.clone());
        if !self.accept_replacement(&r, text, cx) {
            return;
        }
        self.content = format!(
            "{}{}{}",
            &self.content[..r.start],
            text,
            &self.content[r.end..]
        )
        .into();
        let n = r.start + text.len();
        self.selection = n..n;
        self.selection_reversed = false;
        self.is_selecting = false;
        self.marked = None;
        self.correction = None;
        self.reveal_multiline_caret();
        cx.notify();
    }
    fn replace_and_mark_text_in_range(
        &mut self,
        r: Option<Range<usize>>,
        text: &str,
        selected: Option<Range<usize>>,
        w: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if !self.enabled {
            return;
        }
        let range = r
            .clone()
            .map(|r| self.convert(r))
            .or(self.marked.clone())
            .unwrap_or(self.selection.clone());
        if !self.accept_replacement(&range, text, cx) {
            return;
        }
        self.replace_text_in_range(r, text, w, cx);
        let start = self.selection.start - text.len();
        self.marked = Some(start..self.selection.end);
        if let Some(r) = selected {
            fn byte_offset(text: &str, n: usize) -> usize {
                let mut count = 0;
                for (i, c) in text.char_indices() {
                    if count >= n {
                        return i;
                    }
                    count += c.len_utf16();
                }
                text.len()
            }
            let anchor = start + byte_offset(text, r.start);
            let caret = start + byte_offset(text, r.end);
            self.selection = anchor.min(caret)..anchor.max(caret);
            self.selection_reversed = caret < anchor;
        }
        cx.notify();
    }
    fn bounds_for_range(
        &mut self,
        r: Range<usize>,
        bounds: Bounds<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<Bounds<Pixels>> {
        let r = self.convert(r);
        if let Some(mode) = &self.multiline {
            return mode.bounds_for_range(&self.content, r, bounds);
        }
        let l = self.line.as_ref()?;
        Some(Bounds::from_corners(
            point(
                bounds.left() + l.x_for_index(r.start) - self.scroll_x,
                bounds.top(),
            ),
            point(
                bounds.left() + l.x_for_index(r.end) - self.scroll_x,
                bounds.bottom(),
            ),
        ))
    }
    fn character_index_for_point(
        &mut self,
        p: Point<Pixels>,
        _: &mut Window,
        _: &mut Context<Self>,
    ) -> Option<usize> {
        if self.multiline.is_none() {
            self.line.as_ref()?;
        }
        self.bounds?;
        Some(self.to_utf16(self.index_for_position(p)))
    }
}
struct EntryElement(Entity<TextInput>);
impl IntoElement for EntryElement {
    type Element = Self;
    fn into_element(self) -> Self {
        self
    }
}
impl Element for EntryElement {
    type RequestLayoutState = ();
    type PrepaintState = ShapedLine;
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
        s.size.height = px(CONTROL_LINE_HEIGHT).into();
        (w.request_layout(s, [], cx), ())
    }
    fn prepaint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _: &mut (),
        w: &mut Window,
        cx: &mut App,
    ) -> ShapedLine {
        let input = self.0.read(cx);
        let text = input.content.clone();
        let style = w.text_style();
        let runs = input.text_runs(TextRun {
            len: text.len(),
            font: style.font(),
            color: style.color,
            background_color: None,
            underline: None,
            strikethrough: None,
        });
        let line = w
            .text_system()
            .shape_line(text.clone(), px(CONTROL_TEXT_SIZE), &runs, None);
        self.0.update(cx, |input, _| {
            let caret = line.x_for_index(input.caret());
            let visible = (bounds.size.width - px(4.)).max(px(0.));
            if caret - input.scroll_x > visible {
                input.scroll_x = caret - visible;
            }
            if caret < input.scroll_x {
                input.scroll_x = caret;
            }
        });
        line
    }
    fn paint(
        &mut self,
        _: Option<&GlobalElementId>,
        _: Option<&InspectorElementId>,
        b: Bounds<Pixels>,
        _: &mut (),
        line: &mut ShapedLine,
        w: &mut Window,
        cx: &mut App,
    ) {
        let input = self.0.read(cx);
        let focus = input.focus.clone();
        let r = input.selection.clone();
        let scroll_x = input.scroll_x;
        if input.enabled {
            w.handle_input(&focus, ElementInputHandler::new(b, self.0.clone()), cx);
            // Div::on_mouse_move only runs while hovered in GPUI 0.2.2.
            // A frame-scoped capture listener keeps an existing drag alive outside the field.
            let entity = self.0.clone();
            w.on_mouse_event(move |e: &MouseMoveEvent, phase, w, cx| {
                if phase == DispatchPhase::Capture && entity.read(cx).is_selecting {
                    entity.update(cx, |input, cx| input.on_mouse_move(e, w, cx));
                }
            });
        }
        let focused = input.enabled && focus.is_focused(w);
        w.with_content_mask(Some(ContentMask { bounds: b }), |w| {
            if focused {
                let x = b.left() + line.x_for_index(r.start) - scroll_x;
                let end = b.left() + line.x_for_index(r.end) - scroll_x;
                w.paint_quad(fill(
                    Bounds::new(
                        point(x, b.top()),
                        size(
                            if r.is_empty() { px(2.) } else { end - x },
                            px(CONTROL_LINE_HEIGHT),
                        ),
                    ),
                    if r.is_empty() {
                        rgb(ACCENT)
                    } else {
                        rgb(SELECTED)
                    },
                ));
            }
            let _ = line.paint(
                point(b.origin.x - scroll_x, b.origin.y),
                px(CONTROL_LINE_HEIGHT),
                w,
                cx,
            );
        });
        self.0.update(cx, |i, _| {
            i.bounds = Some(b);
            i.line = Some(line.clone());
        });
    }
}
impl Render for TextInput {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        if self.blur_observer.is_none() {
            self.blur_observer = Some(cx.on_blur(&self.focus, window, |this, _, cx| {
                this.touched = true;
                cx.notify();
            }));
        }
        let input = input_base()
            .id("text-input")
            .debug_selector({
                let name = self.debug_name.unwrap_or("text-input");
                move || name.into()
            })
            .key_context("TextInput")
            .when(self.enabled, |el| el.track_focus(&self.focus))
            .when(!self.enabled, |el| el.opacity(0.5))
            .focus(|s| focus_outline(s, ACCENT, ACCENT))
            .cursor(CursorStyle::IBeam)
            .on_action(cx.listener(Self::backspace))
            .on_action(cx.listener(Self::delete))
            .on_action(cx.listener(Self::left))
            .on_action(cx.listener(Self::right))
            .on_action(cx.listener(Self::home))
            .on_action(cx.listener(Self::end))
            .on_action(cx.listener(Self::select_left))
            .on_action(cx.listener(Self::select_right))
            .on_action(cx.listener(Self::select_home))
            .on_action(cx.listener(Self::select_end))
            .on_action(cx.listener(Self::all))
            .on_action(cx.listener(Self::paste))
            .on_action(cx.listener(Self::copy))
            .on_action(cx.listener(Self::cut))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::on_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .on_mouse_up_out(MouseButton::Left, cx.listener(Self::on_mouse_up))
            .when(self.multiline.is_some(), |el| {
                el.key_context("TextInput MultilineInput")
                    .h(px(multiline::EDITOR_HEIGHT))
                    .py(px(8.))
                    .on_action(cx.listener(Self::up))
                    .on_action(cx.listener(Self::down))
                    .on_action(cx.listener(Self::select_up))
                    .on_action(cx.listener(Self::select_down))
                    .on_action(cx.listener(Self::page_up))
                    .on_action(cx.listener(Self::page_down))
                    .on_action(cx.listener(Self::select_page_up))
                    .on_action(cx.listener(Self::select_page_down))
                    .on_action(cx.listener(Self::document_home))
                    .on_action(cx.listener(Self::document_end))
                    .on_action(cx.listener(Self::select_document_home))
                    .on_action(cx.listener(Self::select_document_end))
                    .on_action(cx.listener(Self::newline))
                    .on_scroll_wheel(cx.listener(Self::scroll_note))
            })
            .child(if self.multiline.is_some() {
                MultilineElement(cx.entity()).into_any_element()
            } else {
                EntryElement(cx.entity()).into_any_element()
            });
        let correction = if self.multiline.is_none() && self.touched && !self.composing() {
            self.validation_error()
                .map(str::to_owned)
                .or_else(|| self.correction.clone())
        } else {
            None
        };
        div()
            .w_full()
            .flex()
            .flex_col()
            .gap_1()
            .child(input)
            .when_some(correction, |el, message| {
                el.child(
                    div()
                        .id("field-correction")
                        .debug_selector(|| "field-correction".into())
                        .text_size(px(HELP_TEXT_SIZE))
                        .line_height(px(HELP_LINE_HEIGHT))
                        .text_color(rgb(ERROR))
                        .child(message),
                )
            })
    }
}

fn input_base() -> gpui::Div {
    div()
        .w_full()
        .h(px(CONTROL_HEIGHT))
        .flex_shrink_0()
        .flex()
        .items_center()
        .border_1()
        .border_color(rgb(CONTROL_LINE))
        .rounded(px(4.))
        .overflow_hidden()
        .px(px(12.))
        .bg(rgb(SURFACE))
        .font_family(FONT)
        .text_color(rgb(INK))
        .text_size(px(CONTROL_TEXT_SIZE))
        .line_height(px(CONTROL_LINE_HEIGHT))
        .font_weight(gpui::FontWeight::MEDIUM)
}
pub fn bind(cx: &mut App) {
    multiline::bind(cx);
    for key in ["ctrl", "cmd"] {
        cx.bind_keys([
            KeyBinding::new(&format!("{key}-a"), SelectAll, Some("TextInput")),
            KeyBinding::new(&format!("{key}-v"), Paste, Some("TextInput")),
            KeyBinding::new(&format!("{key}-c"), Copy, Some("TextInput")),
            KeyBinding::new(&format!("{key}-x"), Cut, Some("TextInput")),
        ]);
    }
    cx.bind_keys([
        KeyBinding::new("backspace", Backspace, Some("TextInput")),
        KeyBinding::new("delete", Delete, Some("TextInput")),
        KeyBinding::new("left", Left, Some("TextInput")),
        KeyBinding::new("right", Right, Some("TextInput")),
        KeyBinding::new("home", Home, Some("TextInput")),
        KeyBinding::new("end", End, Some("TextInput")),
        KeyBinding::new("shift-left", SelectLeft, Some("TextInput")),
        KeyBinding::new("shift-right", SelectRight, Some("TextInput")),
        KeyBinding::new("shift-home", SelectHome, Some("TextInput")),
        KeyBinding::new("shift-end", SelectEnd, Some("TextInput")),
        KeyBinding::new("cmd-left", Home, Some("TextInput")),
        KeyBinding::new("cmd-right", End, Some("TextInput")),
        KeyBinding::new("cmd-shift-left", SelectHome, Some("TextInput")),
        KeyBinding::new("cmd-shift-right", SelectEnd, Some("TextInput")),
    ]);
}

#[cfg(test)]
mod style_tests {
    use super::*;

    #[core::prelude::v1::test]
    fn input_focus_keeps_white_editable_surface_and_shared_metrics() {
        let mut input = input_base();
        let base = Style::default().refined(input.style().clone());
        let focused = base.clone().refined(focus_outline(
            gpui::StyleRefinement::default(),
            ACCENT,
            ACCENT,
        ));
        assert_eq!(
            base.background.as_ref().unwrap().color(),
            Some(rgb(SURFACE).into())
        );
        assert_eq!(base.border_color, Some(rgb(CONTROL_LINE).into()));
        assert_eq!(base.size.height, px(CONTROL_HEIGHT).into());
        assert_eq!(base.size, focused.size);
        assert_eq!(base.padding, focused.padding);
        assert_eq!(base.border_widths, focused.border_widths);
        assert_eq!(base.text, focused.text);
        assert_eq!(base.background, focused.background);
        assert_eq!(focused.border_color, Some(rgb(ACCENT).into()));
        assert!(contrast(INK, SURFACE) >= 4.5);
        assert!(contrast(ACCENT, SURFACE) >= 3.);
        assert!(contrast(INK, SELECTED) >= 4.5);
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    use gpui::{AvailableSpace, Modifiers, TestAppContext, VisualTestContext};

    struct InputGeometryView {
        input: Entity<TextInput>,
    }
    impl Render for InputGeometryView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div().w(px(160.)).child(self.input.clone())
        }
    }

    fn draw_input(input: &Entity<TextInput>, cx: &mut VisualTestContext, width: f32) {
        cx.draw(
            point(px(0.), px(0.)),
            size(
                AvailableSpace::Definite(px(width)),
                AvailableSpace::Definite(px(80.)),
            ),
            |_, _| div().w(px(width)).child(input.clone()).into_element(),
        );
    }
    fn position_for_index(
        input: &Entity<TextInput>,
        cx: &mut VisualTestContext,
        n: usize,
    ) -> Point<Pixels> {
        cx.update(|_, cx| {
            let input = input.read(cx);
            let bounds = input.bounds.unwrap();
            point(
                bounds.left() + input.line.as_ref().unwrap().x_for_index(n) - input.scroll_x,
                bounds.top() + px(12.),
            )
        })
    }

    #[gpui::test]
    fn touched_corrections_suppress_ime_and_reset_on_deliberate_value_load(
        cx: &mut TestAppContext,
    ) {
        let (input, cx) = cx.add_window_view(|window, cx| {
            let mut input = TextInput::with_value("", 0, cx);
            input.validator = Some(|s| s.is_empty().then_some("Enter a value."));
            window.focus(&input.focus);
            input
        });
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                assert!(!input.touched);
                assert_eq!(input.validation_error(), Some("Enter a value."));
                input.touched = true;
                input.replace_and_mark_text_in_range(None, "", None, window, cx);
                assert!(input.composing());
                assert!(input.validation_error().is_none());
                input.unmark_text(window, cx);
                assert_eq!(input.validation_error(), Some("Enter a value."));
                input.set_value("restored", cx);
                assert!(!input.touched);
                assert!(input.validation_error().is_none());
            })
        });
    }
    #[gpui::test]
    fn shift_selection_retains_anchor_across_direction_changes(cx: &mut TestAppContext) {
        cx.update(bind);
        let (input, cx) = cx.add_window_view(|window, cx| {
            let input = TextInput::with_value("a🦀bc", 0, cx);
            window.focus(&input.focus);
            input
        });
        cx.simulate_keystrokes("home right shift-right");
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                assert_eq!(input.selection, 1..5);
                let selection = input.selected_text_range(false, window, cx).unwrap();
                assert_eq!(selection.range, 1..3);
                assert!(!selection.reversed);
            })
        });
        cx.simulate_keystrokes("shift-left shift-left");
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                assert_eq!(input.selection, 0..1);
                assert!(
                    input
                        .selected_text_range(false, window, cx)
                        .unwrap()
                        .reversed
                );
            })
        });
        cx.simulate_keystrokes("shift-end");
        cx.update(|_, cx| {
            assert_eq!(input.read(cx).selection, 1..7);
            assert!(!input.read(cx).selection_reversed);
        });
        cx.simulate_keystrokes("shift-home");
        cx.update(|_, cx| {
            assert_eq!(input.read(cx).selection, 0..1);
            assert!(input.read(cx).selection_reversed);
        });
        cx.simulate_keystrokes("right shift-left");
        cx.update(|_, cx| assert_eq!(input.read(cx).selection, 0..1));
        cx.simulate_input("é");
        cx.update(|_, cx| {
            assert_eq!(input.read(cx).content.as_ref(), "é🦀bc");
            assert_eq!(input.read(cx).selection, 2..2);
            assert!(!input.read(cx).selection_reversed);
        });
    }

    #[gpui::test]
    fn mouse_selection_shift_click_word_and_outside_release(cx: &mut TestAppContext) {
        let (input, cx) = cx.add_window_view(|_, cx| TextInput::with_value("one 🦀 café", 0, cx));
        draw_input(&input, cx, 320.);
        let start = position_for_index(&input, cx, 4);
        let end = position_for_index(&input, cx, 9);
        cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        cx.update(|_, cx| assert_eq!(input.read(cx).selection, 4..9));
        draw_input(&input, cx, 320.);
        let outside = point(px(-20.), start.y);
        cx.simulate_mouse_move(outside, MouseButton::Left, Modifiers::none());
        cx.update(|_, cx| {
            assert_eq!(input.read(cx).selection, 0..4);
            assert!(input.read(cx).selection_reversed);
        });
        draw_input(&input, cx, 320.);
        cx.simulate_mouse_up(outside, MouseButton::Left, Modifiers::none());
        cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
        cx.update(|_, cx| {
            assert_eq!(input.read(cx).selection, 0..4);
            assert!(!input.read(cx).is_selecting);
        });
        cx.simulate_click(
            end,
            Modifiers {
                shift: true,
                ..Modifiers::none()
            },
        );
        cx.update(|_, cx| {
            assert_eq!(input.read(cx).selection, 4..9);
            assert!(!input.read(cx).selection_reversed);
        });
        draw_input(&input, cx, 320.);
        let word = position_for_index(&input, cx, 10);
        cx.simulate_event(MouseDownEvent {
            position: word,
            button: MouseButton::Left,
            click_count: 2,
            ..Default::default()
        });
        cx.simulate_mouse_up(word, MouseButton::Left, Modifiers::none());
        cx.update(|_, cx| {
            assert_eq!(
                &input.read(cx).content[input.read(cx).selection.clone()],
                "café"
            );
            assert!(!input.read(cx).is_selecting);
        });
        input.update(cx, |input, _| input.set_enabled(false));
        draw_input(&input, cx, 320.);
        cx.simulate_click(start, Modifiers::none());
        cx.update(|_, cx| assert_eq!(input.read(cx).selection, 9..14));
    }

    #[gpui::test]
    fn composition_runs_limits_and_utf16_conversion_are_preserved(cx: &mut TestAppContext) {
        let (input, cx) = cx.add_window_view(|window, cx| {
            let input = TextInput::with_value("a🦀b", 0, cx);
            window.focus(&input.focus);
            input
        });
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                assert_eq!(input.convert(2..99), 5..6);
                input.replace_and_mark_text_in_range(
                    Some(1..3),
                    "🦀é",
                    Some(Range { start: 3, end: 0 }),
                    window,
                    cx,
                );
                assert_eq!(input.content.as_ref(), "a🦀éb");
                assert_eq!(input.selection, 1..7);
                assert!(input.selection_reversed);
                assert_eq!(input.marked_text_range(window, cx), Some(1..4));
                let style = window.text_style();
                let run = TextRun {
                    len: input.content.len(),
                    font: style.font(),
                    color: style.color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                };
                let runs = input.text_runs(run.clone());
                assert_eq!(runs.iter().map(|r| r.len).collect::<Vec<_>>(), [1, 6, 1]);
                assert!(runs[0].underline.is_none());
                assert_eq!(runs[1].underline.as_ref().unwrap().thickness, px(1.));
                assert!(runs[2].underline.is_none());
                let state = (
                    input.content.clone(),
                    input.selection.clone(),
                    input.marked.clone(),
                );
                input.max_bytes = input.content.len();
                input.replace_and_mark_text_in_range(None, "x\n", None, window, cx);
                input.replace_and_mark_text_in_range(None, &"é".repeat(10), None, window, cx);
                assert_eq!(
                    (
                        input.content.clone(),
                        input.selection.clone(),
                        input.marked.clone()
                    ),
                    state
                );
                assert!(input.selection_reversed);
                input.unmark_text(window, cx);
                assert!(input.text_runs(run).iter().all(|r| r.underline.is_none()));
            })
        });
    }

    #[gpui::test]
    fn clipboard_keeps_exact_selected_bytes_and_existing_paste_rules(cx: &mut TestAppContext) {
        cx.update(bind);
        let (input, cx) = cx.add_window_view(|window, cx| {
            let input = TextInput::with_value("é🦀", 0, cx);
            window.focus(&input.focus);
            input
        });
        cx.simulate_keystrokes("shift-home ctrl-c");
        cx.update(|_, cx| {
            assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), "é🦀");
            cx.write_to_clipboard(ClipboardItem::new_string("a\r\nb".into()));
        });
        cx.simulate_keystrokes("ctrl-v ctrl-a ctrl-x");
        cx.update(|_, cx| {
            assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), "a  b");
            assert!(input.read(cx).content.is_empty());
        });
        cx.simulate_keystrokes("ctrl-c");
        cx.update(|_, cx| assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), ""));
    }

    #[gpui::test]
    fn active_selection_edge_scrolls_without_changing_field_geometry(cx: &mut TestAppContext) {
        cx.update(bind);
        // Keep the constrained field in the mounted view: key dispatch can repaint
        // the window root, unlike a one-off VisualTestContext::draw wrapper.
        let (view, cx) = cx.add_window_view(|window, cx| {
            let input = cx.new(|cx| TextInput::with_value(&"long address ".repeat(30), 0, cx));
            window.focus(&input.read(cx).focus);
            InputGeometryView { input }
        });
        let input = cx.update(|_, cx| view.read(cx).input.clone());
        cx.run_until_parked();
        let bounds = cx.debug_bounds("text-input").unwrap();
        assert_eq!(bounds.size, size(px(160.), px(CONTROL_HEIGHT)));
        cx.update(|_, cx| assert!(input.read(cx).scroll_x > px(0.)));
        cx.simulate_keystrokes("shift-home");
        cx.run_until_parked();
        cx.update(|_, cx| assert_eq!(input.read(cx).scroll_x, px(0.)));
        assert_eq!(cx.debug_bounds("text-input").unwrap(), bounds);
        cx.simulate_keystrokes("shift-end");
        cx.run_until_parked();
        cx.update(|_, cx| assert!(input.read(cx).scroll_x > px(0.)));
        assert_eq!(cx.debug_bounds("text-input").unwrap(), bounds);
        input.update(cx, |input, cx| {
            input.set_enabled(false);
            cx.notify();
        });
        cx.run_until_parked();
        assert_eq!(cx.debug_bounds("text-input").unwrap(), bounds);
    }
    #[gpui::test]
    fn native_input_actions_and_unicode_selection(cx: &mut TestAppContext) {
        cx.update(bind);
        let (input, cx) = cx.add_window_view(|window, cx| {
            let input = TextInput::new(cx);
            window.focus(&input.focus);
            input
        });
        cx.simulate_input("Site 🦀 café");
        cx.simulate_keystrokes("ctrl-a");
        cx.simulate_input("新しい website");
        cx.update(|_, cx| assert_eq!(input.read(cx).content.as_ref(), "新しい website"));
        cx.simulate_keystrokes("home delete end backspace");
        cx.update(|_, cx| assert_eq!(input.read(cx).content.as_ref(), "しい websit"));
    }
    #[gpui::test]
    fn ime_ranges_are_utf16_and_rejected_composition_keeps_selection(cx: &mut TestAppContext) {
        let (input, cx) = cx.add_window_view(|window, cx| {
            let input = TextInput::with_value("a🦀b", 0, cx);
            window.focus(&input.focus);
            input
        });
        cx.update(|window, cx| {
            input.update(cx, |input, cx| {
                input.replace_and_mark_text_in_range(Some(1..3), "日本", Some(1..2), window, cx);
                assert_eq!(input.content.as_ref(), "a日本b");
                assert_eq!(
                    input.selected_text_range(false, window, cx).unwrap().range,
                    2..3
                );
                let selection = input.selection.clone();
                input.replace_and_mark_text_in_range(None, &"x".repeat(5000), None, window, cx);
                assert_eq!(input.selection, selection);
                assert_eq!(input.content.as_ref(), "a日本b");
                input.unmark_text(window, cx);
                assert!(input.marked.is_none());
            })
        });
    }
}
