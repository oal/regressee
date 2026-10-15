// SPDX-License-Identifier: MPL-2.0
//! Recognizable external web links; activation belongs to the shell's guarded
//! pointer/keyboard action path, not a second click-only URL handler.
use super::{controls::focus_outline, theme::*};
use gpui::{prelude::*, *};

/// Never hand arbitrary URL schemes or embedded credentials to the OS opener.
/// URLs shown as data (fields, diagnostics) are not automatically linkified.
pub fn web_url(value: &str) -> Option<url::Url> {
    if value.chars().any(char::is_whitespace) || value.chars().any(char::is_control) {
        return None;
    }
    let url = url::Url::parse(value).ok()?;
    (matches!(url.scheme(), "http" | "https")
        && url.host_str().is_some()
        && url.username().is_empty()
        && url.password().is_none())
    .then_some(url)
}

const LINK_HINT: &str = "Open in default browser";

struct LinkHint;
impl Render for LinkHint {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        let text_style = TextStyle {
            font_family: FONT.into(),
            font_weight: FontWeight::NORMAL,
            ..Default::default()
        };
        let text_width = window
            .text_system()
            .shape_line(
                LINK_HINT.into(),
                px(HELP_TEXT_SIZE),
                &[text_style.to_run(LINK_HINT.len())],
                None,
            )
            .width;
        let width =
            (text_width.ceil() + px(18.)).min((window.viewport_size().width - px(16.)).max(px(1.)));
        // GPUI anchors to the resting pointer and flips at window edges. A
        // transparent gutter keeps the visible bubble away from the cursor
        // in either direction, without changing other tooltips' placement.
        div().px(px(8.)).py(px(12.)).child(
            div()
                .debug_selector(|| "web-link-hint".into())
                .w(width)
                .px(px(8.))
                .py(px(6.))
                .flex()
                .flex_col()
                .bg(rgb(SURFACE))
                .border_1()
                .border_color(rgb(CONTROL_LINE))
                .rounded(px(3.))
                .font_family(FONT)
                .font_weight(FontWeight::NORMAL)
                .text_size(px(HELP_TEXT_SIZE))
                .line_height(px(HELP_LINE_HEIGHT))
                .text_color(rgb(INK))
                .text_decoration_none()
                .child(LINK_HINT),
        )
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod ui_tests {
    use super::*;
    use core::prelude::v1::test;
    use gpui::TestAppContext;
    use std::time::Duration;

    struct LinkView {
        focus: FocusHandle,
        enabled: bool,
        edge: bool,
    }

    impl Render for LinkView {
        fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
            div()
                .size_full()
                .bg(rgb(SURFACE))
                .font_family(FONT)
                .p(px(4.))
                .flex()
                .flex_col()
                .when(self.edge, |el| el.items_end().justify_end())
                .child(div().w(px(280.)).child(web_link(
                    "TooltipTest",
                    "https://example.test/",
                    &self.focus,
                    self.enabled,
                )))
        }
    }

    #[gpui::test]
    fn hints_are_delayed_compact_stationary_and_flip_away_from_the_link(cx: &mut TestAppContext) {
        for (width, height) in [(1280., 800.), (820., 600.)] {
            for edge in [false, true] {
                for enabled in [true, false] {
                    let (_, cx) = cx.add_window_view(|_, cx| LinkView {
                        focus: cx.focus_handle(),
                        enabled,
                        edge,
                    });
                    cx.simulate_resize(size(px(width), px(height)));
                    cx.update(|window, cx| {
                        let _ = window.draw(cx);
                    });
                    let link = cx.debug_bounds("link-TooltipTest").unwrap();
                    let pointer = point(link.right() - px(12.), link.center().y);
                    cx.simulate_mouse_move(pointer, None, Modifiers::none());
                    cx.executor().advance_clock(Duration::from_millis(200));
                    cx.run_until_parked();
                    cx.update(|window, cx| {
                        let _ = window.draw(cx);
                    });
                    assert!(cx.debug_bounds("web-link-hint").is_none());
                    cx.executor().advance_clock(Duration::from_millis(400));
                    cx.run_until_parked();
                    cx.update(|window, cx| {
                        let _ = window.draw(cx);
                    });
                    let hint = cx.debug_bounds("web-link-hint");
                    assert_eq!(hint.is_some(), enabled);
                    if let Some(hint) = hint {
                        assert!(hint.size.width < px(200.) && hint.size.height <= px(32.));
                        assert!(!hint.contains(&pointer));
                        assert!(hint.top() >= link.bottom() || hint.bottom() <= link.top());
                        assert!(hint.left() >= px(0.) && hint.right() <= px(width));
                        assert!(hint.top() >= px(0.) && hint.bottom() <= px(height));
                        cx.simulate_mouse_move(
                            point(pointer.x - px(40.), pointer.y),
                            None,
                            Modifiers::none(),
                        );
                        cx.run_until_parked();
                        cx.update(|window, cx| {
                            let _ = window.draw(cx);
                        });
                        assert_eq!(Some(hint), cx.debug_bounds("web-link-hint"));
                    }
                    assert_eq!(link, cx.debug_bounds("link-TooltipTest").unwrap());
                }
            }
        }
    }
}

#[derive(IntoElement)]
struct LinkLabel {
    address: SharedString,
    enabled: bool,
}

impl RenderOnce for LinkLabel {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let hovered = window.use_state(cx, |_, _| false);
        let hot = self.enabled && *hovered.read(cx);
        div()
            .id("web-link-label")
            .w_full()
            .min_w_0()
            .flex()
            .flex_col()
            // GPUI shapes text before applying interaction refinements. Set
            // ink before layout, inheriting typography/ellipsis from the owner.
            .text_color(rgb(ACCENT))
            .underline()
            .when(hot, |el| el.text_color(rgb(INK)).text_decoration_2())
            .when(self.enabled, |el| {
                el.on_hover(move |value, _, cx| {
                    hovered.update(cx, |hovered, cx| {
                        *hovered = *value;
                        cx.notify();
                    });
                })
            })
            .child(self.address)
    }
}

/// Keeps the URL legible and link-like at rest, not just on hover. Columns give
/// long destinations a definite wrapping measure in GPUI. A context caller can
/// opt into ellipsis. Hints explain activation, never repeat the address.
pub fn web_link(
    id: impl Into<ElementId>,
    address: impl Into<SharedString>,
    focus: &FocusHandle,
    enabled: bool,
) -> Stateful<Div> {
    let address = address.into();
    let id = id.into();
    let selector = id.clone();
    div()
        .id(id)
        .debug_selector(move || format!("link-{selector}"))
        .w_full()
        .min_w_0()
        .max_w_full()
        .min_h(px(24.))
        .py(px(2.))
        .flex()
        .flex_col()
        .justify_center()
        .rounded(px(2.))
        .border_1()
        .border_color(rgba(0x00000000))
        .text_size(px(14.))
        .line_height(px(20.))
        .font_weight(FontWeight::NORMAL)
        .text_color(rgb(ACCENT))
        .underline()
        .when(enabled, |el| {
            el.track_focus(focus)
                .cursor_pointer()
                .focus(|s| focus_outline(s, INK, INK))
                .tooltip(|_, cx| cx.new(|_| LinkHint).into())
        })
        .when(!enabled, |el| el.opacity(0.5))
        .child(LinkLabel { address, enabled })
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::prelude::v1::test;

    #[test]
    fn only_complete_credential_free_web_addresses_are_openable() {
        for value in [
            "https://example.test/a?b=2&a=1&a=0#/orders",
            "http://127.0.0.1:43210/",
            "https://example.test/a%20b",
        ] {
            assert_eq!(web_url(value).unwrap().as_str(), value);
        }
        for value in [
            "javascript:alert(1)",
            "file:///home/user/secret",
            "mailto:user@example.test",
            "data:text/html,test",
            "example.test",
            "/relative",
            "",
            "https://user:password@example.test/",
            "https://user@example.test/",
            "https://example.test/\n",
            "https://example.test/a b",
        ] {
            assert!(web_url(value).is_none(), "{value:?}");
        }
    }
}
