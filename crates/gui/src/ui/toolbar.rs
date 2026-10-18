// SPDX-License-Identifier: MPL-2.0
//! Quiet native tools. Words identify modes; icon-only actions disclose their
//! name on both pointer hover and keyboard focus, without adding a focus stop.
//! Labelled controls never repeat their words in an unsolicited overlay.
use super::{
    controls::{self, ActionStyle},
    theme::*,
};
use gpui::{prelude::*, *};
use std::{cell::Cell, rc::Rc};

/// Window-local input mode for identifying hints, independent of actual focus.
/// Mouse clicks must retain focus without pinning a keyboard-only tooltip.
pub struct HintInput {
    keyboard: Rc<Cell<bool>>,
    view: EntityId,
    _keystrokes: Subscription,
}

impl HintInput {
    pub fn new<V: 'static>(window: &Window, cx: &mut Context<V>) -> Self {
        let keyboard = Rc::new(Cell::new(true));
        let keys = keyboard.clone();
        let view = cx.entity_id();
        let owner = window.window_handle();
        // Bound keys (notably Tab and Enter) may consume the event before a
        // capture_key_down listener runs. Observe them before action dispatch.
        let keystrokes = cx.intercept_keystrokes(move |_, window, cx| {
            if window.window_handle() == owner && !keys.replace(true) {
                cx.notify(view);
            }
        });
        Self {
            keyboard,
            view,
            _keystrokes: keystrokes,
        }
    }

    pub fn is_keyboard(&self) -> bool {
        self.keyboard.get()
    }

    /// Mount before the window's controls. Capture observes pointer input even
    /// when a child consumes it, without moving focus or intercepting activation.
    pub fn tracker(&self) -> impl IntoElement {
        let keyboard = self.keyboard.clone();
        let view = self.view;
        canvas(
            |_, _, _| (),
            move |_, (), window, _| {
                track_pointer::<MouseDownEvent>(window, keyboard.clone(), view);
                track_pointer::<MouseMoveEvent>(window, keyboard.clone(), view);
                track_pointer::<MouseExitEvent>(window, keyboard.clone(), view);
            },
        )
        .absolute()
        .size_full()
    }
}

fn track_pointer<E: MouseEvent>(window: &mut Window, keyboard: Rc<Cell<bool>>, view: EntityId) {
    window.on_mouse_event(move |_: &E, phase, _, cx| {
        if phase == DispatchPhase::Capture && keyboard.replace(false) {
            cx.notify(view);
        }
    });
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ToolIcon {
    Sidebar,
    Results,
    Compare,
    Difference,
    Overlay,
    Heatmap,
    Image,
    Split,
    Swap,
    Zoom,
    Search,
    Hand,
    Details,
    Adjustments,
    Filter,
    Previous,
    Next,
    Up,
    Down,
    Refresh,
    Close,
    Check,
    Runs,
    Baselines,
    Storage,
    Unreviewed,
    Flag,
    Note,
    Forward,
    Folder,
    FolderPlus,
    Save,
    Upload,
    Download,
    Trash,
    Camera,
    Help,
    Keyboard,
    Desktop,
    Phone,
    Globe,
    Draw,
    Coordinates,
    Copy,
    Source,
    Lock,
}

fn paths(icon: ToolIcon, bounds: Bounds<Pixels>) -> Vec<Path<Pixels>> {
    let at = |x: f32, y: f32| {
        bounds.origin + point(bounds.size.width * x / 20., bounds.size.height * y / 20.)
    };
    let mut paths = vec![];
    let mut line = |points: &[(f32, f32)]| {
        let mut p = PathBuilder::stroke(px(1.6));
        p.move_to(at(points[0].0, points[0].1));
        for &(x, y) in &points[1..] {
            p.line_to(at(x, y));
        }
        if let Ok(p) = p.build() {
            paths.push(p);
        }
    };
    match icon {
        ToolIcon::Source => {
            line(&[(7., 5.), (2., 10.), (7., 15.)]);
            line(&[(13., 5.), (18., 10.), (13., 15.)]);
            line(&[(11., 3.), (9., 17.)]);
        }
        ToolIcon::Lock => {
            line(&[(4., 9.), (16., 9.), (16., 18.), (4., 18.), (4., 9.)]);
            line(&[
                (6., 9.),
                (6., 5.),
                (8., 2.),
                (12., 2.),
                (14., 5.),
                (14., 9.),
            ]);
            line(&[(10., 12.), (10., 15.)]);
        }
        ToolIcon::Sidebar
        | ToolIcon::Results
        | ToolIcon::Compare
        | ToolIcon::Split
        | ToolIcon::Image => {
            line(&[(2., 3.), (18., 3.), (18., 17.), (2., 17.), (2., 3.)]);
            match icon {
                ToolIcon::Sidebar => line(&[(7., 3.), (7., 17.)]),
                ToolIcon::Results => {
                    line(&[(6., 3.), (6., 17.)]);
                    line(&[(9., 7.), (15., 7.)]);
                    line(&[(9., 10.), (15., 10.)]);
                    line(&[(9., 13.), (13., 13.)]);
                }
                ToolIcon::Compare => {
                    line(&[(10., 3.), (10., 17.)]);
                    line(&[(5., 7.), (5., 13.)]);
                    line(&[(15., 7.), (15., 13.)]);
                }
                ToolIcon::Split => {
                    line(&[(10., 1.), (10., 19.)]);
                    line(&[(5., 8.), (3., 10.), (5., 12.)]);
                    line(&[(15., 8.), (17., 10.), (15., 12.)]);
                }
                _ => line(&[(3., 14.), (7., 10.), (11., 14.), (14., 11.), (18., 15.)]),
            }
        }
        ToolIcon::Difference => {
            line(&[(3., 3.), (17., 3.), (17., 17.), (3., 17.), (3., 3.)]);
            line(&[(10., 3.), (10., 17.)]);
            line(&[(3., 17.), (10., 10.)]);
            line(&[(10., 10.), (17., 3.)]);
        }
        ToolIcon::Overlay | ToolIcon::Copy => {
            line(&[(2., 2.), (13., 2.), (13., 13.), (2., 13.), (2., 2.)]);
            line(&[(7., 7.), (18., 7.), (18., 18.), (7., 18.), (7., 7.)]);
        }
        ToolIcon::Heatmap => {
            line(&[(2., 17.), (2., 3.)]);
            line(&[(2., 17.), (18., 17.)]);
            line(&[(5., 13.), (8., 8.), (11., 11.), (15., 4.), (18., 6.)]);
        }
        ToolIcon::Swap => {
            line(&[(3., 6.), (17., 6.), (14., 3.)]);
            line(&[(17., 14.), (3., 14.), (6., 17.)]);
        }
        ToolIcon::Zoom | ToolIcon::Search => {
            line(&[(13., 13.), (18., 18.)]);
            let mut p = PathBuilder::stroke(px(1.6));
            p.move_to(at(8., 2.));
            p.cubic_bezier_to(at(14., 8.), at(11.3, 2.), at(14., 4.7));
            p.cubic_bezier_to(at(8., 14.), at(14., 11.3), at(11.3, 14.));
            p.cubic_bezier_to(at(2., 8.), at(4.7, 14.), at(2., 11.3));
            p.cubic_bezier_to(at(8., 2.), at(2., 4.7), at(4.7, 2.));
            if let Ok(p) = p.build() {
                paths.push(p);
            }
        }
        ToolIcon::Hand => {
            line(&[
                (5., 10.),
                (5., 5.),
                (7., 4.),
                (8., 5.),
                (8., 2.),
                (10., 2.),
                (11., 3.),
                (11., 7.),
                (11., 3.),
                (13., 3.),
                (14., 4.),
                (14., 8.),
                (14., 5.),
                (16., 5.),
                (17., 6.),
                (17., 12.),
                (14., 18.),
                (8., 18.),
                (3., 12.),
                (3., 10.),
                (5., 10.),
                (8., 13.),
            ]);
        }
        ToolIcon::Details => {
            line(&[
                (4., 2.),
                (12., 2.),
                (16., 6.),
                (16., 18.),
                (4., 18.),
                (4., 2.),
            ]);
            line(&[(12., 2.), (12., 6.), (16., 6.)]);
            line(&[(7., 10.), (13., 10.)]);
            line(&[(7., 14.), (13., 14.)]);
        }
        ToolIcon::Adjustments | ToolIcon::Filter => {
            if matches!(icon, ToolIcon::Filter) {
                line(&[
                    (2., 4.),
                    (18., 4.),
                    (12., 11.),
                    (12., 16.),
                    (8., 18.),
                    (8., 11.),
                    (2., 4.),
                ]);
            } else {
                for (y, x) in [(5., 7.), (10., 13.), (15., 7.)] {
                    line(&[(2., y), (x - 2., y)]);
                    line(&[(x + 2., y), (18., y)]);
                    line(&[
                        (x - 2., y - 2.),
                        (x + 2., y - 2.),
                        (x + 2., y + 2.),
                        (x - 2., y + 2.),
                        (x - 2., y - 2.),
                    ]);
                }
            }
        }
        ToolIcon::Previous => line(&[(12., 4.), (6., 10.), (12., 16.)]),
        ToolIcon::Next => line(&[(8., 4.), (14., 10.), (8., 16.)]),
        ToolIcon::Up => line(&[(4., 12.), (10., 6.), (16., 12.)]),
        ToolIcon::Down => line(&[(4., 8.), (10., 14.), (16., 8.)]),
        ToolIcon::Refresh => {
            line(&[(17., 2.), (17., 8.), (11., 8.)]);
            let mut p = PathBuilder::stroke(px(1.6));
            p.move_to(at(17., 8.));
            p.cubic_bezier_to(at(10., 2.), at(16., 4.), at(13., 2.));
            p.cubic_bezier_to(at(2., 10.), at(5.6, 2.), at(2., 5.6));
            p.cubic_bezier_to(at(10., 18.), at(2., 14.4), at(5.6, 18.));
            p.cubic_bezier_to(at(17., 14.), at(13., 18.), at(15.8, 16.5));
            if let Ok(p) = p.build() {
                paths.push(p);
            }
        }
        ToolIcon::Close => {
            line(&[(5., 5.), (15., 15.)]);
            line(&[(15., 5.), (5., 15.)]);
        }
        ToolIcon::Check => line(&[(3., 10.), (8., 15.), (17., 5.)]),
        ToolIcon::Runs => {
            line(&[
                (4., 4.),
                (10., 2.),
                (16., 5.),
                (18., 11.),
                (15., 16.),
                (9., 18.),
                (3., 15.),
            ]);
            line(&[(2., 4.), (2., 9.), (7., 9.)]);
            line(&[(10., 5.), (10., 10.), (13., 12.)]);
        }
        ToolIcon::Baselines => {
            line(&[(2., 6.), (10., 2.), (18., 6.), (10., 10.), (2., 6.)]);
            line(&[(2., 10.), (10., 14.), (18., 10.)]);
            line(&[(2., 14.), (10., 18.), (18., 14.)]);
        }
        ToolIcon::Storage => {
            line(&[
                (3., 4.),
                (10., 2.),
                (17., 4.),
                (17., 16.),
                (10., 18.),
                (3., 16.),
                (3., 4.),
                (10., 6.),
                (17., 4.),
            ]);
            line(&[(3., 10.), (10., 12.), (17., 10.)]);
        }
        ToolIcon::Unreviewed | ToolIcon::Help | ToolIcon::Globe => {
            if matches!(icon, ToolIcon::Help) {
                line(&[
                    (7., 7.),
                    (8., 5.),
                    (11., 5.),
                    (13., 7.),
                    (12., 9.),
                    (10., 10.),
                    (10., 12.),
                ]);
                line(&[(10., 14.), (10., 15.)]);
            } else if matches!(icon, ToolIcon::Globe) {
                line(&[(2., 10.), (18., 10.)]);
                line(&[
                    (10., 2.),
                    (6., 7.),
                    (6., 13.),
                    (10., 18.),
                    (14., 13.),
                    (14., 7.),
                    (10., 2.),
                ]);
            }
            let mut p = PathBuilder::stroke(px(1.6));
            p.move_to(at(10., 2.));
            p.cubic_bezier_to(at(18., 10.), at(14.4, 2.), at(18., 5.6));
            p.cubic_bezier_to(at(10., 18.), at(18., 14.4), at(14.4, 18.));
            p.cubic_bezier_to(at(2., 10.), at(5.6, 18.), at(2., 14.4));
            p.cubic_bezier_to(at(10., 2.), at(2., 5.6), at(5.6, 2.));
            if let Ok(p) = p.build() {
                paths.push(p);
            }
        }
        ToolIcon::Flag => {
            line(&[
                (4., 18.),
                (4., 3.),
                (10., 3.),
                (12., 5.),
                (17., 5.),
                (17., 13.),
                (11., 13.),
                (9., 11.),
                (4., 11.),
            ]);
        }
        ToolIcon::Note => {
            line(&[(9., 3.), (3., 3.), (3., 17.), (17., 17.), (17., 11.)]);
            line(&[
                (8., 12.),
                (9., 8.),
                (15., 2.),
                (18., 5.),
                (12., 11.),
                (8., 12.),
            ]);
            line(&[(13., 4.), (16., 7.)]);
        }
        ToolIcon::Forward => {
            line(&[(2., 10.), (18., 10.)]);
            line(&[(12., 4.), (18., 10.), (12., 16.)]);
        }
        ToolIcon::Folder | ToolIcon::FolderPlus => {
            line(&[
                (2., 5.),
                (8., 5.),
                (10., 8.),
                (18., 8.),
                (18., 17.),
                (2., 17.),
                (2., 5.),
            ]);
            if matches!(icon, ToolIcon::FolderPlus) {
                line(&[(10., 10.), (10., 15.)]);
                line(&[(7.5, 12.5), (12.5, 12.5)]);
            }
        }
        ToolIcon::Save => {
            line(&[
                (3., 2.),
                (14., 2.),
                (18., 6.),
                (18., 18.),
                (3., 18.),
                (3., 2.),
            ]);
            line(&[(7., 2.), (7., 7.), (13., 7.), (13., 2.)]);
            line(&[(7., 18.), (7., 12.), (14., 12.), (14., 18.)]);
        }
        ToolIcon::Upload | ToolIcon::Download => {
            line(&[(3., 13.), (3., 18.), (17., 18.), (17., 13.)]);
            if matches!(icon, ToolIcon::Download) {
                line(&[(10., 2.), (10., 13.)]);
                line(&[(6., 9.), (10., 13.), (14., 9.)]);
            } else {
                line(&[(10., 13.), (10., 2.)]);
                line(&[(6., 6.), (10., 2.), (14., 6.)]);
            }
        }
        ToolIcon::Trash => {
            line(&[(2., 5.), (18., 5.)]);
            line(&[(7., 5.), (7., 2.), (13., 2.), (13., 5.)]);
            line(&[(4., 5.), (5., 18.), (15., 18.), (16., 5.)]);
            line(&[(8., 8.), (8., 15.)]);
            line(&[(12., 8.), (12., 15.)]);
        }
        ToolIcon::Camera => {
            line(&[
                (2., 6.),
                (6., 6.),
                (8., 3.),
                (12., 3.),
                (14., 6.),
                (18., 6.),
                (18., 17.),
                (2., 17.),
                (2., 6.),
            ]);
            let mut p = PathBuilder::stroke(px(1.6));
            p.move_to(at(10., 7.));
            p.cubic_bezier_to(at(14., 11.), at(12.2, 7.), at(14., 8.8));
            p.cubic_bezier_to(at(10., 15.), at(14., 13.2), at(12.2, 15.));
            p.cubic_bezier_to(at(6., 11.), at(7.8, 15.), at(6., 13.2));
            p.cubic_bezier_to(at(10., 7.), at(6., 8.8), at(7.8, 7.));
            if let Ok(p) = p.build() {
                paths.push(p);
            }
        }
        ToolIcon::Keyboard => {
            line(&[(2., 4.), (18., 4.), (18., 16.), (2., 16.), (2., 4.)]);
            for y in [7., 10.] {
                for x in [5., 8., 11., 14.] {
                    line(&[(x, y), (x + 1., y)]);
                }
            }
            line(&[(6., 13.), (14., 13.)]);
        }
        ToolIcon::Desktop => {
            line(&[(2., 3.), (18., 3.), (18., 14.), (2., 14.), (2., 3.)]);
            line(&[(10., 14.), (10., 18.)]);
            line(&[(6., 18.), (14., 18.)]);
        }
        ToolIcon::Phone => {
            line(&[(5., 2.), (15., 2.), (15., 18.), (5., 18.), (5., 2.)]);
            line(&[(8., 5.), (12., 5.)]);
            line(&[(9., 15.), (11., 15.)]);
        }
        ToolIcon::Draw => {
            line(&[
                (3., 17.),
                (4., 12.),
                (14., 2.),
                (18., 6.),
                (8., 16.),
                (3., 17.),
            ]);
            line(&[(12., 4.), (16., 8.)]);
            line(&[(4., 12.), (8., 16.)]);
        }
        ToolIcon::Coordinates => {
            line(&[(6., 2.), (6., 16.), (18., 16.)]);
            line(&[(2., 6.), (16., 6.)]);
            line(&[(14., 2.), (14., 18.)]);
        }
    }
    paths
}

pub fn icon(icon: ToolIcon, color: u32, edge: f32) -> impl IntoElement {
    canvas(
        |_, _, _| (),
        move |bounds, (), window, _| {
            for path in paths(icon, bounds) {
                window.paint_path(path, rgb(color));
            }
        },
    )
    .size(px(edge))
    .flex_shrink_0()
}

pub fn separator() -> Div {
    div()
        .w(px(1.))
        .h(px(20.))
        .mx(px(4.))
        .flex_shrink_0()
        .bg(rgb(palette().line))
}

struct ToolHint(SharedString);
impl Render for ToolHint {
    fn render(&mut self, window: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        hint(self.0.clone(), window).debug_selector(|| "toolbar-hover-hint".into())
    }
}

fn hint(text: SharedString, window: &Window) -> Div {
    let text_style = TextStyle {
        font_family: FONT.into(),
        font_weight: FontWeight::NORMAL,
        ..Default::default()
    };
    let text_width = window
        .text_system()
        .shape_line(
            text.clone(),
            px(HELP_TEXT_SIZE),
            &[text_style.to_run(text.len())],
            None,
        )
        .width;
    let width = (text_width.ceil() + px(18.))
        .min(px(280.))
        .min((window.viewport_size().width - px(16.)).max(px(1.)));
    div()
        .w(width)
        .p_2()
        .border_1()
        .border_color(rgb(palette().control_line))
        .rounded(px(4.))
        .bg(rgb(palette().surface))
        .shadow(super::depth::Elevation::Floating.shadows())
        .text_color(rgb(palette().ink))
        .font_family(FONT)
        .font_weight(FontWeight::NORMAL)
        .text_size(px(HELP_TEXT_SIZE))
        .line_height(px(HELP_LINE_HEIGHT))
        .child(text)
}

/// Selected tools have a mint surface AND teal boundary; color is not the only
/// state cue. Preserve the ordinary action's guarded focus/disabled behavior.
#[allow(clippy::too_many_arguments)]
pub fn button(
    id: impl Into<ElementId>,
    glyph: ToolIcon,
    label: Option<SharedString>,
    name: SharedString,
    focus: &FocusHandle,
    selected: bool,
    enabled: bool,
    keyboard_hints: bool,
) -> Stateful<Div> {
    let icon_only = label.is_none();
    let focus = focus.clone();
    let focused = focus.clone();
    let focus_name = name.clone();
    let id = id.into();
    let name_id = id.clone();
    let mut frame = controls::action_frame_colors(
        id,
        &focus,
        ActionStyle::Choice { selected },
        enabled,
        if selected {
            rgb(palette().selected).into()
        } else {
            transparent_black()
        },
        rgb(palette().action_hover).into(),
    );
    frame.style().align_self = None;
    frame
        // Quiet tools and menu rows must not cast a shadow from a transparent
        // frame. Focus still replaces this with the shared functional ring.
        .shadow(Vec::new())
        .relative()
        .h(px(CONTROL_HEIGHT))
        .px(px(8.))
        .py(px(0.))
        .gap(px(8.))
        .border_color(if selected {
            rgb(palette().accent).into()
        } else {
            transparent_black()
        })
        .when(icon_only, |el| el.w(px(CONTROL_HEIGHT)))
        .child(icon(
            glyph,
            if selected {
                palette().accent_hover
            } else {
                palette().ink
            },
            18.,
        ))
        .when_some(label, |el, label| {
            el.child(
                div()
                    .debug_selector(move || format!("tool-label-{name_id}"))
                    .whitespace_nowrap()
                    .child(label),
            )
        })
        .when(icon_only && enabled, |el| {
            el.tooltip(move |_, cx| cx.new(|_| ToolHint(name.clone())).into())
                .child(
                    canvas(
                        move |bounds, window, cx| {
                            if !keyboard_hints || !focused.is_focused(window) {
                                return None;
                            }
                            let viewport = window.viewport_size();
                            let above = bounds.bottom() + px(80.) >= viewport.height;
                            let y = if above {
                                bounds.top() - px(6.)
                            } else {
                                bounds.bottom() + px(6.)
                            };
                            let mut popup = deferred(
                                anchored()
                                    .anchor(if above {
                                        Corner::BottomLeft
                                    } else {
                                        Corner::TopLeft
                                    })
                                    .snap_to_window_with_margin(px(8.))
                                    .position(point(bounds.left(), y))
                                    .child(
                                        hint(focus_name.clone(), window)
                                            .debug_selector(|| "toolbar-focus-hint".into()),
                                    ),
                            )
                            .with_priority(3)
                            .into_any_element();
                            popup.layout_as_root(
                                viewport.map(AvailableSpace::Definite),
                                window,
                                cx,
                            );
                            popup.prepaint_at(point(px(0.), px(0.)), window, cx);
                            Some(popup)
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
                    .size_full(),
                )
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ui-tests")]
    use core::prelude::v1::test;

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn hint_input_mode_is_window_local(cx: &mut TestAppContext) {
        struct Host {
            focus: FocusHandle,
            input: Option<HintInput>,
        }
        impl Render for Host {
            fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
                let input = self.input.get_or_insert_with(|| HintInput::new(window, cx));
                div()
                    .size_full()
                    .track_focus(&self.focus)
                    .child(input.tracker())
            }
        }
        let first_window = cx.add_window(|_, cx| Host {
            focus: cx.focus_handle(),
            input: None,
        });
        let second_window = cx.add_window(|_, cx| Host {
            focus: cx.focus_handle(),
            input: None,
        });
        let first = first_window.root(cx).unwrap();
        let second = second_window.root(cx).unwrap();
        let first_cx = &mut VisualTestContext::from_window(first_window.into(), cx);
        let second_cx = &mut VisualTestContext::from_window(second_window.into(), cx);
        for (host, cx) in [(&first, &mut *first_cx), (&second, &mut *second_cx)] {
            cx.update(|w, cx| {
                w.focus(&host.read(cx).focus);
                let _ = w.draw(cx);
            });
            cx.simulate_mouse_move(point(px(10.), px(10.)), None, Modifiers::none());
            assert!(!host.read_with(cx, |s, _| s.input.as_ref().unwrap().is_keyboard()));
        }
        second_cx.simulate_keystrokes("tab");
        assert!(second.read_with(second_cx, |s, _| s.input.as_ref().unwrap().is_keyboard()));
        assert!(!first.read_with(first_cx, |s, _| s.input.as_ref().unwrap().is_keyboard()));
    }

    #[core::prelude::v1::test]
    fn tool_family_has_native_geometry() {
        for glyph in [
            ToolIcon::Sidebar,
            ToolIcon::Results,
            ToolIcon::Compare,
            ToolIcon::Difference,
            ToolIcon::Overlay,
            ToolIcon::Heatmap,
            ToolIcon::Image,
            ToolIcon::Split,
            ToolIcon::Swap,
            ToolIcon::Zoom,
            ToolIcon::Hand,
            ToolIcon::Details,
            ToolIcon::Adjustments,
            ToolIcon::Filter,
            ToolIcon::Previous,
            ToolIcon::Next,
            ToolIcon::Up,
            ToolIcon::Down,
            ToolIcon::Refresh,
            ToolIcon::Close,
            ToolIcon::Check,
            ToolIcon::Runs,
            ToolIcon::Baselines,
            ToolIcon::Storage,
            ToolIcon::Unreviewed,
            ToolIcon::Flag,
            ToolIcon::Note,
            ToolIcon::Forward,
            ToolIcon::Folder,
            ToolIcon::FolderPlus,
            ToolIcon::Save,
            ToolIcon::Upload,
            ToolIcon::Download,
            ToolIcon::Trash,
            ToolIcon::Camera,
            ToolIcon::Help,
            ToolIcon::Keyboard,
            ToolIcon::Desktop,
            ToolIcon::Phone,
            ToolIcon::Globe,
            ToolIcon::Draw,
            ToolIcon::Coordinates,
            ToolIcon::Copy,
            ToolIcon::Source,
            ToolIcon::Lock,
        ] {
            assert!(
                !paths(
                    glyph,
                    Bounds::new(point(px(0.), px(0.)), size(px(18.), px(18.)))
                )
                .is_empty(),
                "{glyph:?}"
            );
        }
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn only_enabled_icon_only_tools_offer_identifying_hints(cx: &mut TestAppContext) {
        struct Host {
            focus: FocusHandle,
            labelled: bool,
            enabled: bool,
        }
        impl Render for Host {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div().size_full().p_4().child(button(
                    "hint-policy-tool",
                    ToolIcon::Details,
                    self.labelled.then(|| "Details".into()),
                    "Details".into(),
                    &self.focus,
                    false,
                    self.enabled,
                    true,
                ))
            }
        }
        for labelled in [true, false] {
            for enabled in [true, false] {
                // Fresh windows: removed GPUI debug selectors otherwise linger.
                let (host, cx) = cx.add_window_view(|_, cx| Host {
                    focus: cx.focus_handle(),
                    labelled,
                    enabled,
                });
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                let frame = cx.debug_bounds("action-hint-policy-tool").unwrap();
                cx.simulate_mouse_move(frame.center(), None, Modifiers::none());
                cx.executor()
                    .advance_clock(std::time::Duration::from_millis(200));
                cx.run_until_parked();
                assert!(cx.debug_bounds("toolbar-hover-hint").is_none());
                cx.executor()
                    .advance_clock(std::time::Duration::from_millis(400));
                cx.run_until_parked();
                cx.update(|w, cx| {
                    let _ = w.draw(cx);
                });
                assert_eq!(
                    cx.debug_bounds("toolbar-hover-hint").is_some(),
                    !labelled && enabled
                );
                cx.update(|w, cx| {
                    w.focus(&host.read(cx).focus);
                    let _ = w.draw(cx);
                });
                assert_eq!(
                    cx.debug_bounds("toolbar-focus-hint").is_some(),
                    !labelled && enabled
                );
                assert_eq!(frame, cx.debug_bounds("action-hint-policy-tool").unwrap());
            }
        }
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn icon_tools_keep_target_size_and_disclose_names_on_keyboard_focus(cx: &mut TestAppContext) {
        struct Host {
            focus: FocusHandle,
            enabled: bool,
        }
        impl Render for Host {
            fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
                div()
                    .size_full()
                    .flex()
                    .items_end()
                    .justify_end()
                    .p_2()
                    .child(button(
                        "test-tool",
                        ToolIcon::Details,
                        None,
                        "Details".into(),
                        &self.focus,
                        true,
                        self.enabled,
                        true,
                    ))
            }
        }
        let (host, cx) = cx.add_window_view(|_, cx| Host {
            focus: cx.focus_handle(),
            enabled: true,
        });
        cx.simulate_resize(size(px(820.), px(600.)));
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        let initial = cx.debug_bounds("action-test-tool").unwrap();
        assert_eq!(initial.size, size(px(40.), px(40.)));
        assert!(cx.debug_bounds("toolbar-focus-hint").is_none());
        cx.update(|w, cx| {
            w.focus(&host.read(cx).focus);
            let _ = w.draw(cx);
        });
        let hint = cx.debug_bounds("toolbar-focus-hint").unwrap();
        assert!(hint.bottom() <= initial.top());
        assert!(hint.left() >= px(8.) && hint.right() <= px(812.));
        assert_eq!(cx.debug_bounds("action-test-tool").unwrap(), initial);
        host.update(cx, |s, cx| {
            s.enabled = false;
            cx.notify();
        });
        cx.run_until_parked();
        cx.update(|w, cx| {
            let _ = w.draw(cx);
        });
        assert_eq!(cx.debug_bounds("action-test-tool").unwrap(), initial);
        // GPUI's diagnostic bounds map retains removed selectors across frames;
        // use a fresh window to establish absence rather than stale map entries.
        let (disabled, cx) = cx.add_window_view(|_, cx| Host {
            focus: cx.focus_handle(),
            enabled: false,
        });
        cx.update(|w, cx| {
            w.focus(&disabled.read(cx).focus);
            let _ = w.draw(cx);
        });
        assert!(cx.debug_bounds("toolbar-focus-hint").is_none());
    }
}
