// SPDX-License-Identifier: MPL-2.0
use super::*;
use gpui::{ClipboardItem, Modifiers, TestAppContext};

struct NoteView {
    input: Entity<TextInput>,
    width: f32,
}
impl Render for NoteView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div().w(px(self.width)).child(self.input.clone())
    }
}
fn position(
    input: &Entity<TextInput>,
    cx: &mut gpui::VisualTestContext,
    n: usize,
) -> Point<Pixels> {
    cx.update(|_, cx| {
        let i = input.read(cx);
        let b = i.bounds.unwrap();
        let m = i.multiline.as_ref().unwrap();
        let p = m.current(&i.content).unwrap().position(n, m.upstream);
        point(b.left() + p.x, b.top() + p.y - m.scroll_y + px(10.))
    })
}

#[gpui::test]
fn note_byte_boundaries_replacement_and_corrective_shrinking(cx: &mut TestAppContext) {
    let (input, cx) = cx.add_window_view(|_, cx| TextInput::review_note("", 0, cx));
    cx.update(|w, cx| {
        input.update(cx, |i, cx| {
            assert_eq!(i.max_bytes, regressee_core::domain::REVIEW_NOTE_MAX_BYTES);
            for bytes in [4096, 4097, 5000, 8192] {
                i.set_value("", cx);
                let text = "x".repeat(bytes);
                i.replace_text_in_range(None, &text, w, cx);
                assert_eq!(i.content.as_ref(), text);
                i.backspace(&Backspace, w, cx);
                assert_eq!(i.content.len(), bytes - 1);
                i.replace_text_in_range(None, "x", w, cx);
                assert_eq!(i.content.as_ref(), text);
            }
            let state = (i.content.clone(), i.selection.clone(), i.marked.clone());
            i.replace_text_in_range(None, "x", w, cx);
            assert_eq!(
                (i.content.clone(), i.selection.clone(), i.marked.clone()),
                state
            );
            assert!(i.correction.as_ref().unwrap().contains("paste less"));
            i.set_value(&"x".repeat(8193), cx);
            i.backspace(&Backspace, w, cx);
            assert_eq!(i.content.len(), 8192);
            i.set_value(&"x".repeat(9000), cx);
            i.backspace(&Backspace, w, cx);
            assert_eq!(i.content.len(), 8999); // still oversized, but shrinking is permitted
            i.selection = 0..1;
            i.delete(&Delete, w, cx);
            assert_eq!(i.content.len(), 8998);
            i.selection = 0..1000;
            i.replace_text_in_range(None, "é\n🦀", w, cx);
            assert_eq!(i.content.len(), 8005);
            assert!(i.content.starts_with("é\n🦀"));
            assert!(i.correction.is_none());
        })
    });
}

#[gpui::test]
fn note_clipboard_newlines_unicode_limit_and_ime_are_exact(cx: &mut TestAppContext) {
    cx.update(super::super::bind);
    let (input, cx) = cx.add_window_view(|w, cx| {
        let i = TextInput::review_note("", 0, cx);
        w.focus(&i.focus);
        i
    });
    let text = "Headline is intentional.\nConfirm the product grid.\r\n日本語 🦀 café\n";
    cx.update(|_, cx| cx.write_to_clipboard(ClipboardItem::new_string(text.into())));
    cx.simulate_keystrokes("ctrl-v ctrl-a ctrl-c");
    cx.update(|w, cx| {
        assert_eq!(cx.read_from_clipboard().unwrap().text().unwrap(), text);
        input.update(cx, |i, cx| {
            assert_eq!(i.content.as_ref(), text);
            i.set_value(&"🦀".repeat(2048), cx);
            let state = (i.content.clone(), i.selection.clone());
            i.replace_text_in_range(None, "é", w, cx);
            assert_eq!((i.content.clone(), i.selection.clone()), state);
            i.selection = 0..4;
            i.replace_text_in_range(None, "é\n", w, cx);
            assert_eq!(i.content.len(), 8191);
            assert!(i.content.starts_with("é\n🦀"));
            i.set_value("a🦀\nb", cx);
            i.replace_and_mark_text_in_range(Some(1..3), "日本\n🦀", Some(3..5), w, cx);
            assert_eq!(i.content.as_ref(), "a日本\n🦀\nb");
            assert_eq!(i.selected_text_range(false, w, cx).unwrap().range, 4..6);
            assert_eq!(i.marked_text_range(w, cx), Some(1..6));
            let state = (
                i.content.clone(),
                i.selection.clone(),
                i.marked.clone(),
                i.selection_reversed,
            );
            i.replace_and_mark_text_in_range(None, &"🦀".repeat(2049), None, w, cx);
            assert_eq!(
                (
                    i.content.clone(),
                    i.selection.clone(),
                    i.marked.clone(),
                    i.selection_reversed
                ),
                state
            );
            assert!(i.correction.is_some());
            i.unmark_text(w, cx);
            let mut actual = None;
            assert_eq!(
                i.text_for_range(1..6, &mut actual, w, cx).unwrap(),
                "日本\n🦀"
            );
            assert_eq!(actual, Some(1..6));
        });
    });
    cx.simulate_keystrokes("ctrl-a ctrl-x");
    cx.update(|_, cx| assert!(input.read(cx).content.is_empty()));
    cx.simulate_keystrokes("enter");
    cx.update(|_, cx| assert_eq!(input.read(cx).content.as_ref(), "\n"));
}

#[gpui::test]
fn note_native_vertical_selection_caret_scroll_pointer_and_resize(cx: &mut TestAppContext) {
    cx.update(super::super::bind);
    let note = (1..=30)
        .map(|n| format!("Line {n}: the headline is intentional; confirm the product grid.\n"))
        .collect::<String>();
    let (view, cx) = cx.add_window_view(|w, cx| {
        let input = cx.new(|cx| TextInput::review_note(&note, 0, cx));
        w.focus(&input.read(cx).focus);
        NoteView { input, width: 320. }
    });
    let input = cx.update(|_, cx| view.read(cx).input.clone());
    cx.run_until_parked();
    let border = cx.debug_bounds("text-input").unwrap();
    assert_eq!(border.size.height, px(EDITOR_HEIGHT));
    cx.update(|_, cx| {
        let i = input.read(cx);
        assert!(i.multiline_overflows());
        assert!(i.multiline_scroll_offset().unwrap() > px(0.));
        assert!(
            i.multiline
                .as_ref()
                .unwrap()
                .layout
                .as_ref()
                .unwrap()
                .rows
                .len()
                > 30
        );
    });
    cx.simulate_keystrokes("ctrl-home down");
    cx.run_until_parked();
    cx.update(|_, cx| {
        let i = input.read(cx);
        let m = i.multiline.as_ref().unwrap();
        assert_eq!(
            m.current(&i.content)
                .unwrap()
                .position(i.caret(), m.upstream)
                .y,
            px(CONTROL_LINE_HEIGHT)
        );
    });
    cx.simulate_keystrokes("down");
    cx.run_until_parked();
    cx.update(|_, cx| {
        let i = input.read(cx);
        let m = i.multiline.as_ref().unwrap();
        assert_eq!(
            m.current(&i.content)
                .unwrap()
                .position(i.caret(), m.upstream)
                .y,
            px(2. * CONTROL_LINE_HEIGHT)
        );
    });
    cx.simulate_keystrokes("ctrl-home right right down");
    cx.run_until_parked();
    let down = cx.update(|_, cx| input.read(cx).caret());
    assert!(down > 2);
    cx.simulate_keystrokes("up");
    cx.run_until_parked();
    cx.update(|_, cx| assert_eq!(input.read(cx).caret(), 2));
    cx.simulate_keystrokes("shift-down shift-down shift-up");
    cx.run_until_parked();
    cx.update(|_, cx| {
        let i = input.read(cx);
        assert_eq!(i.selection.start, 2);
        assert_eq!(i.selection.end, down);
    });
    cx.simulate_keystrokes("ctrl-home pagedown");
    cx.run_until_parked();
    cx.update(|w, cx| {
        input.update(cx, |i, cx| {
            assert!(i.multiline_scroll_offset().unwrap() > px(0.));
            let b = i.bounds.unwrap();
            let n = i.to_utf16(i.caret());
            let caret = i.bounds_for_range(n..n, b, w, cx).unwrap();
            assert!(caret.top() >= b.top() && caret.bottom() <= b.bottom());
        })
    });
    cx.simulate_keystrokes("ctrl-home");
    cx.run_until_parked();
    let start = position(&input, cx, 2);
    let end = position(&input, cx, down);
    cx.simulate_mouse_down(start, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_move(end, MouseButton::Left, Modifiers::none());
    cx.simulate_mouse_up(end, MouseButton::Left, Modifiers::none());
    cx.update(|_, cx| assert_eq!(input.read(cx).selection, 2..down));
    let b = cx.update(|_, cx| input.read(cx).bounds.unwrap());
    cx.simulate_event(gpui::ScrollWheelEvent {
        position: b.center(),
        delta: gpui::ScrollDelta::Pixels(point(px(0.), px(-80.))),
        ..Default::default()
    });
    cx.run_until_parked();
    cx.update(|_, cx| assert!(input.read(cx).multiline_scroll_offset().unwrap() >= px(80.)));
    cx.simulate_keystrokes("ctrl-end");
    cx.run_until_parked();
    assert_eq!(cx.debug_bounds("text-input").unwrap(), border);
    let rows = cx.update(|_, cx| {
        input
            .read(cx)
            .multiline
            .as_ref()
            .unwrap()
            .layout
            .as_ref()
            .unwrap()
            .rows
            .len()
    });
    view.update(cx, |v, cx| {
        v.width = 240.;
        cx.notify();
    });
    cx.run_until_parked();
    cx.update(|_, cx| {
        let i = input.read(cx);
        assert_eq!(i.content.as_ref(), note);
        assert!(
            i.multiline
                .as_ref()
                .unwrap()
                .layout
                .as_ref()
                .unwrap()
                .rows
                .len()
                >= rows
        );
        assert_eq!(i.selection, note.len()..note.len());
    });
    cx.update(|w, cx| {
        input.update(cx, |i, cx| {
            let b = i.bounds.unwrap();
            let n = i.content.encode_utf16().count();
            let caret = i.bounds_for_range(n..n, b, w, cx).unwrap();
            assert!(caret.bottom() <= b.bottom() && caret.top() >= b.top());
            i.set_value("Short note.\nSecond line.", cx);
        })
    });
    cx.run_until_parked();
    cx.update(|_, cx| assert!(!input.read(cx).multiline_overflows()));
}
