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
                .bg(rgb(palette().surface))
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
    fn links_keep_hover_and_focus_affordances_without_help_bubbles(cx: &mut TestAppContext) {
        for (width, height) in [(1280., 800.), (820., 600.)] {
            for edge in [false, true] {
                for enabled in [true, false] {
                    let (view, cx) = cx.add_window_view(|_, cx| LinkView {
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
                    assert!(cx.debug_bounds("web-link-hint").is_none());
                    cx.update(|window, cx| {
                        if enabled {
                            window.focus(&view.read(cx).focus);
                        }
                        let _ = window.draw(cx);
                    });
                    assert!(cx.debug_bounds("web-link-hint").is_none());
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
            .text_color(rgb(palette().accent))
            .underline()
            .when(hot, |el| {
                el.text_color(rgb(palette().ink)).text_decoration_2()
            })
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
/// opt into ellipsis. Underlines and pointer/focus states convey activation
/// without an unsolicited help bubble.
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
        .text_color(rgb(palette().accent))
        .underline()
        .when(enabled, |el| {
            el.track_focus(focus)
                .cursor_pointer()
                .focus(|s| focus_outline(s, palette().accent))
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
