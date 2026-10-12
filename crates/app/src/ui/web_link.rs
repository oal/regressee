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

struct DestinationHint(SharedString);
impl Render for DestinationHint {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .w(px(440.))
            .max_w_full()
            .p_3()
            .flex()
            .flex_col()
            .gap_1()
            .bg(rgb(SURFACE))
            .border_1()
            .border_color(rgb(CONTROL_LINE))
            .rounded(px(3.))
            .text_size(px(13.))
            .line_height(px(18.))
            .text_color(rgb(INK))
            .child("Open in your default browser")
            .child(div().w_full().flex().flex_col().child(self.0.clone()))
    }
}

/// Keeps the URL legible and link-like at rest, not just on hover. Columns give
/// long destinations a definite wrapping measure in GPUI. A context caller can
/// opt into ellipsis; the destination hint still shows the complete address.
pub fn web_link(
    id: impl Into<ElementId>,
    address: impl Into<SharedString>,
    focus: &FocusHandle,
    enabled: bool,
) -> Stateful<Div> {
    let address = address.into();
    let hint = address.clone();
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
                // Interaction refinements cannot replace inherited GPUI text style.
                .focus(|s| focus_outline(s, INK, INK))
                .hover(|s| s.bg(rgb(ACTION_HOVER)))
                .active(|s| s.bg(rgb(SELECTED)))
                .tooltip(move |_, cx| cx.new(|_| DestinationHint(hint.clone())).into())
        })
        .when(!enabled, |el| el.opacity(0.5))
        .child(address)
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
