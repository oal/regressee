// SPDX-License-Identifier: MPL-2.0
//! Native, selection-only timezone control. Search text is never a profile value.
use super::{controls::focus_outline, text_input::TextInput, theme::*};
use gpui::{
    AnyElement, App, AvailableSpace, Bounds, Context, Entity, FocusHandle, FontWeight, KeyBinding,
    MouseButton, MouseDownEvent, PathBuilder, Pixels, Point, ScrollStrategy, SharedString, Size,
    Subscription, UniformListScrollHandle, Window, actions, anchored, canvas, deferred, div, point,
    prelude::*, px, rgb, uniform_list,
};
use std::sync::OnceLock;

actions!(
    timezone_select,
    [Toggle, Next, Previous, First, Last, Choose, Close]
);

const FIELD_TEXT_SIZE: f32 = CONTROL_TEXT_SIZE;
const HEADER_HEIGHT: f32 = 80.;
const MAX_LIST_HEIGHT: f32 = 320.;
const POPUP_GAP: f32 = 4.;
const WINDOW_MARGIN: f32 = 8.;

fn timezones() -> &'static [&'static str] {
    static ZONES: OnceLock<Vec<&'static str>> = OnceLock::new();
    ZONES.get_or_init(|| {
        let mut zones: Vec<_> = include_str!("../../assets/timezones.txt")
            .lines()
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .collect();
        // UTC is the useful first choice; every other identifier is alphabetical.
        zones.sort_unstable_by_key(|name| (*name != "UTC", *name));
        zones.dedup();
        zones
    })
}

fn matches_query(zone: &str, query: &str) -> bool {
    let name = zone.to_ascii_lowercase().replace('_', " ");
    query
        .to_ascii_lowercase()
        .replace('_', " ")
        .split_whitespace()
        .all(|word| name.contains(word))
}

// Preserve the underlying value exactly; only its presentation is bounded and
// made single-line. Imported historical values must not become UI instructions.
fn display_value(value: &str) -> SharedString {
    if value.is_empty() {
        return "Choose a timezone".into();
    }
    let mut text: String = value
        .chars()
        .take(256)
        .map(|c| {
            if c.is_control() || matches!(c, '\u{061c}' | '\u{200e}' | '\u{200f}' | '\u{202a}'..='\u{202e}' | '\u{2066}'..='\u{2069}') {
                '\u{fffd}'
            } else {
                c
            }
        })
        .collect();
    if value.chars().count() > 256 {
        text.push('…');
    }
    text.into()
}

#[derive(Clone, Copy)]
struct PopupGeometry {
    origin: Point<Pixels>,
    width: Pixels,
    height: Pixels,
    list_height: Pixels,
}

fn popup_geometry(trigger: Bounds<Pixels>, viewport: Size<Pixels>, count: usize) -> PopupGeometry {
    let margin = px(WINDOW_MARGIN);
    let available_below = (viewport.height - margin - trigger.bottom() - px(POPUP_GAP)).max(px(0.));
    let available_above = (trigger.top() - margin - px(POPUP_GAP)).max(px(0.));
    let desired =
        px(HEADER_HEIGHT + 2.) + px((count.max(1) as f32 * CONTROL_HEIGHT).min(MAX_LIST_HEIGHT));
    let above = available_below < desired && available_above > available_below;
    let available = if above {
        available_above
    } else {
        available_below
    };
    let height = desired
        .min(available)
        .min((viewport.height - margin * 2.).max(px(0.)));
    let width = trigger
        .size
        .width
        .min((viewport.width - margin * 2.).max(px(0.)));
    let x = trigger
        .left()
        .min(viewport.width - margin - width)
        .max(margin);
    let y = if above {
        trigger.top() - px(POPUP_GAP) - height
    } else {
        trigger.bottom() + px(POPUP_GAP)
    };
    PopupGeometry {
        origin: point(x, y.max(margin)),
        width,
        height,
        list_height: (height - px(HEADER_HEIGHT + 2.)).max(px(0.)),
    }
}

/// Parent focus order should request `focus_handle` at the time of navigation:
/// while open, the search field replaces the closed select's focus stop.
pub struct TimezoneSelect {
    pub selected: SharedString,
    enabled: bool,
    open: bool,
    button_focus: FocusHandle,
    search_focus: FocusHandle,
    search: Entity<TextInput>,
    query: SharedString,
    filtered: Vec<usize>,
    highlighted: Option<usize>,
    scroll: UniformListScrollHandle,
    trigger_bounds: Option<Bounds<Pixels>>,
    clear_disabled_focus: bool,
    _search_observer: Subscription,
    focus_observers: Vec<Subscription>,
}

impl TimezoneSelect {
    pub fn new(value: &str, cx: &mut Context<Self>) -> Self {
        let search = cx.new(|cx| {
            let mut input = TextInput::search("Search region or city…", 0, cx);
            input.max_bytes = 256;
            input.set_enabled(false);
            input
        });
        let search_focus = search.read(cx).focus.clone();
        let observer = cx.observe(&search, |this, search, cx| {
            let query = search.read(cx).content.clone();
            if this.open && this.query != query {
                this.refilter(query, cx);
            }
        });
        Self {
            selected: value.to_owned().into(),
            enabled: true,
            open: false,
            button_focus: cx.focus_handle().tab_stop(true),
            search_focus,
            search,
            query: "".into(),
            filtered: (0..timezones().len()).collect(),
            highlighted: None,
            scroll: UniformListScrollHandle::new(),
            trigger_bounds: None,
            clear_disabled_focus: false,
            _search_observer: observer,
            focus_observers: Vec::new(),
        }
    }

    /// Import/switch profiles without coercing legacy aliases or unknown values.
    pub fn set_value(&mut self, value: &str, cx: &mut Context<Self>) {
        if self.selected.as_ref() != value {
            self.selected = value.to_owned().into();
            if self.open {
                self.refilter(self.query.clone(), cx);
            }
            cx.notify();
        }
    }

    /// As with shared controls, the caller notifies after changing enabled state.
    /// A window is not available here; render clears our focus, never another
    /// control's focus, on the next frame. Neither this nor dismissal edits value.
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
        self.button_focus = self.button_focus.clone().tab_stop(enabled && !self.open);
        if !enabled {
            self.open = false;
            self.highlighted = None;
            self.search_focus = self.search_focus.clone().tab_stop(false);
            self.clear_disabled_focus = true;
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Control handoff must suspend the nested IME handler immediately, before
    /// a later render removes the popup or clears its old native focus.
    pub fn suspend_input(&mut self, cx: &mut Context<Self>) {
        self.set_enabled(false);
        self.search.update(cx, |input, _| input.set_enabled(false));
        cx.notify();
    }

    pub fn is_composing(&self, cx: &App) -> bool {
        self.search.read(cx).composing()
    }
    #[cfg(feature = "ui-tests")]
    pub fn test_search_input(&self) -> Entity<TextInput> {
        self.search.clone()
    }

    pub fn focus_handle(&self, cx: &App) -> FocusHandle {
        if self.open && self.enabled {
            self.search.read(cx).focus.clone()
        } else {
            self.button_focus.clone()
        }
    }

    /// Call after `text_input::bind`, so Home/End in this search navigate options.
    /// Tab is deliberately not bound: the shell owns the complete focus order.
    pub fn bind(cx: &mut App) {
        for context in ["TimezoneSelect", "TimezoneSelectSearch > TextInput"] {
            cx.bind_keys([
                KeyBinding::new("down", Next, Some(context)),
                KeyBinding::new("up", Previous, Some(context)),
                KeyBinding::new("home", First, Some(context)),
                KeyBinding::new("end", Last, Some(context)),
                KeyBinding::new("enter", Choose, Some(context)),
                KeyBinding::new("escape", Close, Some(context)),
            ]);
        }
        cx.bind_keys([KeyBinding::new(
            "space",
            Toggle,
            Some("TimezoneSelectButton"),
        )]);
    }

    fn owns_focus(&self, window: &Window) -> bool {
        self.button_focus.is_focused(window) || self.search_focus.is_focused(window)
    }

    fn refilter(&mut self, query: SharedString, cx: &mut Context<Self>) {
        self.query = query;
        self.filtered = timezones()
            .iter()
            .enumerate()
            .filter_map(|(index, zone)| matches_query(zone, &self.query).then_some(index))
            .collect();
        self.highlighted = self
            .filtered
            .iter()
            .position(|index| timezones()[*index] == self.selected.as_ref())
            .or_else(|| (!self.filtered.is_empty()).then_some(0));
        self.scroll = UniformListScrollHandle::new();
        self.scroll_to_highlight();
        cx.notify();
    }

    fn scroll_to_highlight(&self) {
        if let Some(index) = self.highlighted {
            self.scroll.scroll_to_item(index, ScrollStrategy::Center);
        }
    }

    fn show(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled || self.open || !super::interaction::allowed(window, cx) {
            return;
        }
        self.open = true;
        self.button_focus = self.button_focus.clone().tab_stop(false);
        self.search.update(cx, |input, cx| {
            input.set_enabled(true);
            input.set_value("", cx);
        });
        self.search_focus = self.search.read(cx).focus.clone();
        self.refilter("".into(), cx);
        window.focus(&self.search_focus);
    }

    fn dismiss(&mut self, restore_focus: bool, window: &mut Window, cx: &mut Context<Self>) {
        if !self.open {
            return;
        }
        self.open = false;
        self.highlighted = None;
        self.button_focus = self.button_focus.clone().tab_stop(self.enabled);
        self.search.update(cx, |input, _| input.set_enabled(false));
        self.search_focus = self.search.read(cx).focus.clone();
        if restore_focus && self.enabled {
            window.focus(&self.button_focus);
        } else if self.search_focus.is_focused(window) {
            // Outside pointer/blur dismissal does not steal focus from its target.
            window.blur();
        }
        cx.notify();
    }

    fn select_index(&mut self, index: usize, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled || !self.open || !super::interaction::allowed(window, cx) {
            return;
        }
        if let Some(zone) = self.filtered.get(index).map(|index| timezones()[*index]) {
            self.selected = zone.into();
            self.dismiss(true, window, cx);
            cx.notify();
        }
    }

    fn toggle(&mut self, _: &Toggle, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled || !super::interaction::allowed(window, cx) {
            return;
        }
        if self.open {
            self.dismiss(true, window, cx);
        } else {
            self.show(window, cx);
        }
    }

    fn next(&mut self, _: &Next, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled || !super::interaction::allowed(window, cx) {
            return;
        }
        if !self.open {
            self.show(window, cx);
            return;
        }
        if let Some(index) = self.highlighted {
            self.highlighted = Some((index + 1).min(self.filtered.len().saturating_sub(1)));
            self.scroll_to_highlight();
            cx.notify();
        }
    }

    fn previous(&mut self, _: &Previous, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled || !super::interaction::allowed(window, cx) {
            return;
        }
        if !self.open {
            self.show(window, cx);
            return;
        }
        if let Some(index) = self.highlighted {
            self.highlighted = Some(index.saturating_sub(1));
            self.scroll_to_highlight();
            cx.notify();
        }
    }

    fn first(&mut self, _: &First, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled || !super::interaction::allowed(window, cx) {
            return;
        }
        self.show(window, cx);
        if self.enabled && !self.filtered.is_empty() {
            self.highlighted = Some(0);
            self.scroll_to_highlight();
            cx.notify();
        }
    }

    fn last(&mut self, _: &Last, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled || !super::interaction::allowed(window, cx) {
            return;
        }
        self.show(window, cx);
        if self.enabled {
            self.highlighted = self.filtered.len().checked_sub(1);
            self.scroll_to_highlight();
            cx.notify();
        }
    }

    fn choose(&mut self, _: &Choose, window: &mut Window, cx: &mut Context<Self>) {
        if !self.enabled || !super::interaction::allowed(window, cx) {
            return;
        }
        if !self.open {
            self.show(window, cx);
        } else if let Some(index) = self.highlighted {
            self.select_index(index, window, cx);
        }
    }

    fn close(&mut self, _: &Close, window: &mut Window, cx: &mut Context<Self>) {
        if self.open {
            self.dismiss(true, window, cx);
        } else {
            // Only an open dropdown owns Escape; leave the shell's ordinary
            // dismissal behavior available when the select is closed.
            cx.propagate();
        }
    }

    fn popup(
        &mut self,
        bounds: Bounds<Pixels>,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let geometry = popup_geometry(bounds, window.viewport_size(), self.filtered.len());
        let entity = cx.entity();
        let rows = self.filtered.clone();
        let highlighted = self.highlighted;
        let selected = self.selected.clone();
        let list = uniform_list("timezone-options", rows.len(), move |range, _, _| {
            range
                .map(|index| {
                    let zone = timezones()[rows[index]];
                    let current = zone == selected.as_ref();
                    let active = highlighted == Some(index);
                    let target = entity.clone();
                    div()
                        .id(("timezone-option", index))
                        .debug_selector(move || format!("timezone-option-{zone}"))
                        .w_full()
                        .h(px(CONTROL_HEIGHT))
                        .px(px(12.))
                        .flex()
                        .items_center()
                        .gap(px(8.))
                        .border_1()
                        .border_color(rgb(if active {
                            palette().ink
                        } else if current {
                            palette().selected
                        } else {
                            palette().surface
                        }))
                        .bg(rgb(if current {
                            palette().selected
                        } else {
                            palette().surface
                        }))
                        .text_size(px(FIELD_TEXT_SIZE))
                        .line_height(px(CONTROL_LINE_HEIGHT))
                        .font_weight(FontWeight::MEDIUM)
                        .text_color(rgb(if current {
                            palette().accent_hover
                        } else {
                            palette().ink
                        }))
                        .cursor_pointer()
                        .hover(move |s| {
                            s.bg(rgb(if current {
                                palette().selected
                            } else {
                                palette().ground
                            }))
                        })
                        .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
                        .on_click(move |_, window, cx| {
                            target.update(cx, |this, cx| this.select_index(index, window, cx));
                            cx.stop_propagation();
                        })
                        .child(
                            div()
                                .flex_1()
                                .min_w(px(0.))
                                .overflow_hidden()
                                .text_ellipsis()
                                .child(zone),
                        )
                        .when(current, |el| {
                            el.child(div().text_size(px(HELP_TEXT_SIZE)).line_height(px(HELP_LINE_HEIGHT)).child("Current"))
                        })
                })
                .collect::<Vec<_>>()
        })
        .h(geometry.list_height)
        .w_full()
        .track_scroll(self.scroll.clone());

        let popup = div()
            .id("timezone-popup")
            .shadow(super::depth::Elevation::Floating.shadows())
            .debug_selector(|| "timezone-popup".into())
            .key_context("TimezoneSelectSearch")
            .occlude()
            .w(geometry.width)
            .h(geometry.height)
            .flex()
            .flex_col()
            .overflow_hidden()
            .border_1()
            .border_color(rgb(palette().control_line))
            .rounded(px(4.))
            .bg(rgb(palette().surface))
            .text_color(rgb(palette().ink))
            .on_mouse_down(MouseButton::Left, |_, _, cx| cx.stop_propagation())
            .on_mouse_down_out(cx.listener(|this, event: &MouseDownEvent, window, cx| {
                if !this
                    .trigger_bounds
                    .is_some_and(|bounds| bounds.contains(&event.position))
                {
                    this.dismiss(false, window, cx);
                }
            }))
            .child(
                div()
                    .h(px(HEADER_HEIGHT))
                    .flex_shrink_0()
                    .p(px(8.))
                    .flex()
                    .flex_col()
                    .gap(px(4.))
                    .text_size(px(HELP_TEXT_SIZE))
                    .line_height(px(HELP_LINE_HEIGHT))
                    .font_weight(FontWeight::NORMAL)
                    .child("Find a timezone")
                    .child(
                        div()
                            .debug_selector(|| "timezone-search".into())
                            .h(px(CONTROL_HEIGHT))
                            .child(self.search.clone()),
                    ),
            )
            .child(
                div()
                    .debug_selector(|| "timezone-list".into())
                    .h(geometry.list_height)
                    .flex_shrink_0()
                    .overflow_hidden()
                    .when(self.filtered.is_empty(), |el| {
                        el.px(px(12.))
                            .flex()
                            .items_center()
                            .text_size(px(FIELD_TEXT_SIZE))
                            .line_height(px(CONTROL_LINE_HEIGHT))
                            .text_color(rgb(palette().muted))
                            .child("No matching timezones")
                    })
                    .when(!self.filtered.is_empty(), |el| {
                        el.child(super::scrollbar::vertical(
                            "timezone-scrollbar",
                            &self.scroll.0.borrow().base_handle,
                            list,
                        ))
                    }),
            );
        deferred(anchored().position(geometry.origin).child(popup))
            .with_priority(1)
            .into_any_element()
    }
}

/// Module-level convenience for callers that bind the other shared controls.
pub fn bind(cx: &mut App) {
    TimezoneSelect::bind(cx);
}

pub(super) fn chevron(open: bool) -> impl IntoElement {
    chevron_sized(open, 16.)
}

pub fn chevron_sized(open: bool, edge: f32) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            let at = |x: f32, y: f32| {
                bounds.origin + point(bounds.size.width * x, bounds.size.height * y)
            };
            let mut path = PathBuilder::stroke(px(2.));
            path.move_to(at(0.2, if open { 0.65 } else { 0.35 }));
            path.line_to(at(0.5, if open { 0.35 } else { 0.65 }));
            path.line_to(at(0.8, if open { 0.65 } else { 0.35 }));
            if let Ok(path) = path.build() {
                window.paint_path(path, rgb(palette().ink));
            }
        },
    )
    .size(px(edge))
    .flex_shrink_0()
}

impl Render for TimezoneSelect {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // GPUI suppresses focus-path notifications while a window is inactive.
        // Also reconcile actual focus on a frame, so Tab cannot strand an open
        // menu when no on_blur event is delivered (including the test platform).
        if self.open && !self.owns_focus(window) {
            self.dismiss(false, window, cx);
        }
        if self.focus_observers.is_empty() {
            for focus in [self.button_focus.clone(), self.search_focus.clone()] {
                self.focus_observers
                    .push(cx.on_blur(&focus, window, |this, window, cx| {
                        if !this.owns_focus(window) {
                            this.dismiss(false, window, cx);
                        }
                    }));
            }
        }
        if self.clear_disabled_focus {
            if !self.enabled {
                self.search.update(cx, |input, _| input.set_enabled(false));
                if self.owns_focus(window) {
                    window.blur();
                }
            }
            self.clear_disabled_focus = false;
        }
        let entity = cx.entity();
        let known = timezones().contains(&self.selected.as_ref());
        div()
            .id("timezone-select")
            .debug_selector(|| "timezone-select".into())
            .key_context("TimezoneSelect")
            .relative()
            .w_full()
            .h(px(CONTROL_HEIGHT))
            .flex()
            .flex_col()
            .flex_shrink_0()
            .font_family(FONT)
            .on_action(cx.listener(Self::toggle))
            .on_action(cx.listener(Self::next))
            .on_action(cx.listener(Self::previous))
            .on_action(cx.listener(Self::first))
            .on_action(cx.listener(Self::last))
            .on_action(cx.listener(Self::choose))
            .on_action(cx.listener(Self::close))
            .child(
                div()
                    .id("timezone-select-button")
                    .debug_selector(|| "timezone-select-button".into())
                    .key_context("TimezoneSelectButton")
                    .relative()
                    .w_full()
                    .h(px(CONTROL_HEIGHT))
                    .pl(px(12.))
                    .pr(px(40.))
                    .flex()
                    .items_center()
                    .gap(px(8.))
                    .border_1()
                    .border_color(rgb(if self.open {
                        palette().accent
                    } else {
                        palette().control_line
                    }))
                    .rounded(px(4.))
                    .bg(rgb(palette().action_surface))
                    .text_color(rgb(palette().ink))
                    .text_size(px(FIELD_TEXT_SIZE))
                    .line_height(px(CONTROL_LINE_HEIGHT))
                    .font_weight(FontWeight::MEDIUM)
                    .when(self.enabled, |el| {
                        el.track_focus(&self.button_focus)
                            .cursor_pointer()
                            .hover(|s| s.bg(rgb(palette().action_hover)))
                            .focus(|s| {
                                focus_outline(
                                    s.border_color(rgb(palette().accent)),
                                    palette().accent,
                                )
                            })
                    })
                    .when(!self.enabled, |el| el.opacity(0.5))
                    .on_mouse_down(
                        MouseButton::Left,
                        cx.listener(|this, _, window, cx| {
                            if this.enabled {
                                this.toggle(&Toggle, window, cx);
                                cx.stop_propagation();
                            }
                        }),
                    )
                    .child(
                        div()
                            .debug_selector(|| "timezone-value".into())
                            .flex()
                            .flex_1()
                            .min_w(px(0.))
                            .overflow_hidden()
                            .text_ellipsis()
                            .child(display_value(&self.selected)),
                    )
                    .when(!known && !self.selected.is_empty(), |el| {
                        el.child(
                            div()
                                .text_size(px(HELP_TEXT_SIZE))
                                .line_height(px(HELP_LINE_HEIGHT))
                                .text_color(rgb(palette().muted))
                                .child("Not in list"),
                        )
                    })
                    .child(
                        div()
                            .debug_selector(|| "timezone-chevron".into())
                            .absolute()
                            .right(px(12.))
                            .top(px(12.))
                            .size(px(16.))
                            .flex()
                            .flex_shrink_0()
                            .child(chevron(self.open)),
                    ),
            )
            // Construct the overlay from this frame's actual trigger bounds,
            // not cached render-time coordinates (which lag resize/scroll).
            // Deferred drawing escapes ancestor scroll/content masks, retains
            // the key-dispatch ancestry, and does not grow the form's layout.
            .child(
                canvas(
                    move |bounds, window, cx| {
                        let popup = entity.update(cx, |this, cx| {
                            this.trigger_bounds = Some(bounds);
                            (this.open && this.enabled).then(|| this.popup(bounds, window, cx))
                        });
                        popup.map(|mut popup| {
                            popup.layout_as_root(
                                window.viewport_size().map(AvailableSpace::Definite),
                                window,
                                cx,
                            );
                            popup.prepaint_at(point(px(0.), px(0.)), window, cx);
                            popup
                        })
                    },
                    |_, popup, window, cx| {
                        if let Some(mut popup) = popup {
                            popup.paint(window, cx);
                        }
                    },
                )
                .absolute()
                .top_0()
                .left_0()
                .w_full()
                .h_full(),
            )
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod tests {
    use super::*;
    use gpui::{Modifiers, TestAppContext, VisualTestContext, size};

    actions!(
        timezone_select_tests,
        [ShellActivate, ShellTab, ShellTabPrev]
    );

    struct SelectHost {
        select: Entity<TimezoneSelect>,
        after: Entity<TextInput>,
        shell_activations: usize,
    }

    impl Render for SelectHost {
        fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
            div()
                .key_context("Shell")
                .w(px(320.))
                .flex()
                .flex_col()
                .gap(px(12.))
                .on_action(cx.listener(|this, _: &ShellActivate, _, _| {
                    this.shell_activations += 1;
                }))
                .on_action(cx.listener(|this, _: &ShellTab, window, cx| {
                    window.focus(&this.after.read(cx).focus);
                }))
                .on_action(cx.listener(|this, _: &ShellTabPrev, window, cx| {
                    window.focus(&this.select.read(cx).focus_handle(cx));
                }))
                // The dropdown must escape a form/scroll ancestor's clip.
                .child(
                    div()
                        .h(px(CONTROL_HEIGHT))
                        .overflow_hidden()
                        .child(self.select.clone()),
                )
                .child(self.after.clone())
        }
    }

    fn setup<'a>(
        cx: &'a mut TestAppContext,
        value: &str,
    ) -> (Entity<SelectHost>, &'a mut VisualTestContext) {
        cx.update(|cx| {
            super::super::text_input::bind(cx);
            bind(cx);
            // Shell bindings are intentionally installed last. The select and
            // its search have nearer contexts, not a global binding-order hack.
            cx.bind_keys([
                KeyBinding::new("enter", ShellActivate, Some("Shell")),
                KeyBinding::new("tab", ShellTab, Some("Shell")),
                KeyBinding::new("shift-tab", ShellTabPrev, Some("Shell")),
            ]);
        });
        cx.add_window_view(|window, cx| {
            let select = cx.new(|cx| TimezoneSelect::new(value, cx));
            let after = cx.new(TextInput::new);
            window.focus(&select.read(cx).focus_handle(cx));
            SelectHost {
                select,
                after,
                shell_activations: 0,
            }
        })
    }

    #[gpui::test]
    fn closed_trigger_keeps_chevron_at_right_edge(cx: &mut TestAppContext) {
        let (_, cx) = setup(cx, "UTC");
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let trigger = cx.debug_bounds("timezone-select-button").unwrap();
        let value = cx.debug_bounds("timezone-value").unwrap();
        let arrow = cx.debug_bounds("timezone-chevron").unwrap();
        assert!(
            arrow.right() >= trigger.right() - px(16.),
            "trigger={trigger:?}, value={value:?}, arrow={arrow:?}"
        );
    }

    #[gpui::test]
    fn agent_suspension_blocks_nested_search_ime_before_next_render(cx: &mut TestAppContext) {
        use gpui::EntityInputHandler;
        let (host, cx) = setup(cx, "UTC");
        let select = host.read_with(cx, |host, _| host.select.clone());
        cx.update(|window, cx| {
            select.update(cx, |select, cx| {
                select.show(window, cx);
                select.search.update(cx, |input, cx| {
                    input.replace_and_mark_text_in_range(None, "日本", None, window, cx);
                });
                assert!(select.is_composing(cx));
                let retained = select.search.read(cx).content.clone();
                select.suspend_input(cx);
                select.search.update(cx, |input, cx| {
                    input.replace_text_in_range(None, "blocked", window, cx);
                    input.replace_and_mark_text_in_range(None, "blocked", None, window, cx);
                });
                assert_eq!(select.search.read(cx).content, retained);
                assert_eq!(select.selected.as_ref(), "UTC");
                assert!(!select.open);
                assert!(!select.is_enabled());
            });
        });
    }

    #[core::prelude::v1::test]
    fn bundled_canonical_list_is_complete_unique_and_cross_region() {
        let zones = timezones();
        assert_eq!(zones.len(), 437);
        assert_eq!(zones[0], "UTC");
        assert!(zones[1..].windows(2).all(|pair| pair[0] < pair[1]));
        for name in [
            "Europe/Oslo",
            "Europe/Kyiv",
            "Asia/Kolkata",
            "America/Coyhaique",
            "America/Argentina/Ushuaia",
            "Antarctica/Troll",
            "Africa/Juba",
            "Australia/Lord_Howe",
            "Indian/Chagos",
            "Pacific/Kanton",
            "Etc/UTC",
        ] {
            assert!(
                zones.contains(&name),
                "missing canonical Zone record: {name}"
            );
        }
        for alias in [
            "US/Eastern",
            "Europe/Kiev",
            "Asia/Calcutta",
            "GMT",
            "localtime",
            "posixrules",
        ] {
            assert!(!zones.contains(&alias));
        }
        assert!(
            zones
                .iter()
                .all(|zone| zone.is_ascii() && !zone.chars().any(char::is_control))
        );
        assert!(matches_query(
            "America/Argentina/Buenos_Aires",
            "ARGENTINA buenos aires"
        ));
        assert!(!matches_query("Europe/Oslo", "asia"));
    }

    #[core::prelude::v1::test]
    fn dropdown_geometry_is_bounded_and_prefers_the_available_side() {
        let viewport = size(px(820.), px(600.));
        let top = Bounds::new(point(px(240.), px(60.)), size(px(320.), px(40.)));
        let below = popup_geometry(top, viewport, timezones().len());
        assert_eq!(below.origin.y, top.bottom() + px(POPUP_GAP));
        assert_eq!(below.width, px(320.));
        assert_eq!(below.list_height, px(MAX_LIST_HEIGHT));
        let bottom = Bounds::new(point(px(700.), px(540.)), size(px(320.), px(40.)));
        let above = popup_geometry(bottom, viewport, timezones().len());
        assert_eq!(above.origin.y + above.height + px(POPUP_GAP), bottom.top());
        assert!(above.origin.x + above.width <= viewport.width - px(WINDOW_MARGIN));
        assert!(above.origin.y >= px(WINDOW_MARGIN));
        let small = popup_geometry(top, size(px(280.), px(220.)), timezones().len());
        assert!(small.width <= px(264.));
        assert!(small.origin.y + small.height <= px(212.));
        assert!(small.list_height < px(MAX_LIST_HEIGHT));
    }

    #[gpui::test]
    fn keyboard_selects_options_without_activating_the_shell(cx: &mut TestAppContext) {
        let (host, cx) = setup(cx, "Europe/Oslo");
        let select = cx.update(|_, cx| host.read(cx).select.clone());
        cx.simulate_keystrokes("down");
        cx.update(|window, cx| {
            let state = select.read(cx);
            assert!(state.open);
            assert_eq!(state.selected.as_ref(), "Europe/Oslo");
            assert!(state.focus_handle(cx).is_focused(window));
            assert_eq!(
                timezones()[state.filtered[state.highlighted.unwrap()]],
                "Europe/Oslo"
            );
        });
        cx.simulate_keystrokes("home down up");
        cx.update(|_, cx| assert_eq!(select.read(cx).highlighted, Some(0)));
        cx.simulate_keystrokes("end");
        cx.run_until_parked();
        assert!(cx.debug_bounds("timezone-option-Pacific/Wallis").is_some());
        cx.simulate_keystrokes("enter");
        cx.update(|window, cx| {
            let state = select.read(cx);
            assert_eq!(state.selected.as_ref(), "Pacific/Wallis");
            assert!(!state.open);
            assert!(state.button_focus.is_focused(window));
            assert_eq!(host.read(cx).shell_activations, 0);
        });
        cx.simulate_keystrokes("space home escape");
        cx.update(|window, cx| {
            let state = select.read(cx);
            assert_eq!(state.selected.as_ref(), "Pacific/Wallis");
            assert!(!state.open);
            assert!(state.button_focus.is_focused(window));
        });
    }

    #[gpui::test]
    fn search_is_a_filter_not_a_new_timezone_value(cx: &mut TestAppContext) {
        let (host, cx) = setup(cx, "UTC");
        let select = cx.update(|_, cx| host.read(cx).select.clone());
        cx.simulate_keystrokes("enter");
        cx.simulate_input("oSlO");
        cx.run_until_parked();
        cx.update(|_, cx| {
            let state = select.read(cx);
            assert_eq!(state.selected.as_ref(), "UTC");
            assert_eq!(state.filtered.len(), 1);
            assert_eq!(timezones()[state.filtered[0]], "Europe/Oslo");
        });
        cx.simulate_keystrokes("enter");
        cx.update(|_, cx| assert_eq!(select.read(cx).selected.as_ref(), "Europe/Oslo"));
        cx.simulate_keystrokes("enter");
        cx.simulate_input("no-such-timezone");
        cx.run_until_parked();
        cx.simulate_keystrokes("home end up down enter");
        cx.update(|_, cx| {
            let state = select.read(cx);
            assert!(state.open);
            assert!(state.filtered.is_empty());
            assert!(state.highlighted.is_none());
            assert_eq!(state.selected.as_ref(), "Europe/Oslo");
            assert_eq!(host.read(cx).shell_activations, 0);
        });
        cx.simulate_keystrokes("escape enter");
        cx.update(|_, cx| {
            assert!(select.read(cx).query.is_empty());
            assert_eq!(select.read(cx).filtered.len(), timezones().len());
        });
    }

    #[gpui::test]
    fn pointer_selection_notifies_and_popup_escapes_the_form_clip(cx: &mut TestAppContext) {
        use std::{cell::Cell, rc::Rc};
        let (host, cx) = setup(cx, "UTC");
        let select = cx.update(|_, cx| host.read(cx).select.clone());
        let notifications = Rc::new(Cell::new(0));
        cx.update(|_, cx| {
            let notifications = notifications.clone();
            cx.observe(&select, move |_, _| {
                notifications.set(notifications.get() + 1)
            })
            .detach();
        });
        let closed = cx.debug_bounds("timezone-select-button").unwrap();
        assert_eq!(closed.size, size(px(320.), px(CONTROL_HEIGHT)));
        cx.simulate_click(closed.center(), Modifiers::none());
        cx.run_until_parked();
        let popup = cx.debug_bounds("timezone-popup").unwrap();
        let list = cx.debug_bounds("timezone-list").unwrap();
        assert!(popup.top() >= closed.bottom());
        assert!(list.size.height <= px(MAX_LIST_HEIGHT));
        assert_eq!(cx.debug_bounds("timezone-select-button").unwrap(), closed);
        assert!(cx.debug_bounds("timezone-option-UTC").is_some());
        let search = cx.debug_bounds("timezone-search").unwrap();
        cx.simulate_click(
            point(search.left() + px(12.), search.top() + px(8.)),
            Modifiers::none(),
        );
        cx.update(|window, cx| {
            assert!(select.read(cx).search_focus.is_focused(window));
            assert!(!host.read(cx).after.read(cx).focus.is_focused(window));
        });
        cx.simulate_input("oslo");
        cx.run_until_parked();
        let row = cx.debug_bounds("timezone-option-Europe/Oslo").unwrap();
        let before = notifications.get();
        cx.simulate_click(row.center(), Modifiers::none());
        cx.run_until_parked();
        assert!(notifications.get() > before);
        cx.update(|_, cx| {
            assert_eq!(select.read(cx).selected.as_ref(), "Europe/Oslo");
            assert!(!select.read(cx).open);
        });
    }

    #[gpui::test]
    fn tab_blur_and_click_outside_close_without_trapping_focus(cx: &mut TestAppContext) {
        let (host, cx) = setup(cx, "Europe/Oslo");
        let select = cx.update(|_, cx| host.read(cx).select.clone());
        cx.simulate_keystrokes("enter tab");
        cx.run_until_parked();
        cx.update(|window, cx| {
            assert!(!select.read(cx).open);
            assert!(host.read(cx).after.read(cx).focus.is_focused(window));
        });
        cx.simulate_keystrokes("shift-tab enter");
        cx.run_until_parked();
        cx.update(|window, cx| assert!(select.read(cx).focus_handle(cx).is_focused(window)));
        cx.simulate_click(point(px(500.), px(10.)), Modifiers::none());
        cx.run_until_parked();
        cx.update(|window, cx| {
            assert!(!select.read(cx).open);
            assert!(!select.read(cx).owns_focus(window));
            assert_eq!(select.read(cx).selected.as_ref(), "Europe/Oslo");
        });
        // A later external focus assignment must not be undone by dismissal.
        cx.update(|window, cx| window.focus(&host.read(cx).after.read(cx).focus));
        cx.run_until_parked();
        cx.update(|window, cx| assert!(host.read(cx).after.read(cx).focus.is_focused(window)));
    }

    #[gpui::test]
    fn unknown_values_are_preserved_and_disabled_controls_cannot_select(cx: &mut TestAppContext) {
        let historical = "Unknown/Legacy\nZone";
        let (host, cx) = setup(cx, historical);
        let select = cx.update(|_, cx| host.read(cx).select.clone());
        assert!(!display_value(historical).contains('\n'));
        cx.simulate_keystrokes("enter escape");
        cx.update(|_, cx| assert_eq!(select.read(cx).selected.as_ref(), historical));
        cx.simulate_keystrokes("enter");
        select.update(cx, |this, cx| {
            this.set_enabled(false);
            cx.notify();
        });
        cx.run_until_parked();
        assert!(cx.debug_bounds("timezone-popup").is_none());
        let button = cx.debug_bounds("timezone-select-button").unwrap();
        cx.simulate_click(button.center(), Modifiers::none());
        cx.update(|window, cx| {
            select.update(cx, |this, cx| {
                this.choose(&Choose, window, cx);
                this.select_index(0, window, cx);
                assert!(!this.open);
                assert!(!this.owns_focus(window));
                assert_eq!(this.selected.as_ref(), historical);
                this.set_value("Europe/Kiev", cx);
                assert_eq!(this.selected.as_ref(), "Europe/Kiev");
                this.set_enabled(true);
                window.focus(&this.focus_handle(cx));
                cx.notify();
            });
        });
        cx.simulate_keystrokes("enter home enter");
        cx.update(|_, cx| assert_eq!(select.read(cx).selected.as_ref(), "UTC"));
    }
}
