// SPDX-License-Identifier: MPL-2.0
//! Settings wayfinding and simulated GPUI input, not OS/AT qualification.
use super::*;
use crate::{
    phase1_setup_tests::inside, phase4_hierarchy_tests::draw,
    workflow_refinement_tests::load_review,
};
use gpui::{TestAppContext, VisualTestContext};

pub(super) fn prepare(
    s: &mut Shell,
    project: regressee_core::domain::ProjectId,
    cx: &mut Context<Shell>,
) {
    load_review(s, project, cx);
    s.focus.screen = Screen::Setup;
    s.focus.setup_stage = SetupStage::Settings;
    s.focus.advanced = false;
    s.error = None;
    s.notice = None;
    s.runtime_readiness = Some(crate::app_settings_tests::readiness(
        regressee_app::product::RuntimeState::Ready,
        true,
    ));
    s.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
    cx.notify();
}

pub(super) fn assert_destination(
    shell: &Entity<Shell>,
    cx: &mut VisualTestContext,
    section: SettingsSection,
) {
    draw(cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.current_settings_section(), section);
        assert_eq!(s.focus.setup_stage, SetupStage::Settings);
    });
    let bounds = cx
        .debug_bounds(match section {
            SettingsSection::Website => "settings-section-Website",
            SettingsSection::Capture => "settings-section-Capture",
            SettingsSection::Access => "settings-section-Access",
        })
        .unwrap();
    let clip = cx.debug_bounds("screen-scroll").unwrap();
    assert!(
        bounds.top() >= clip.top() && bounds.top() + px(64.) < clip.bottom(),
        "Section heading must be visible: {bounds:?}, {clip:?}"
    );
}

#[gpui::test]
fn project_settings_jumps_preserve_drafts_and_keep_wayfinding_pinned(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| {
        prepare(s, project, cx);
        s.name
            .update(cx, |i, cx| i.set_value("Retained settings draft", cx));
    });
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        draw(cx);
        let navigation = cx.debug_bounds("settings-section-navigation").unwrap();
        let footer = cx.debug_bounds("setup-footer-status").unwrap();
        let form = cx.debug_bounds("settings-section-Website").unwrap();
        let clip = cx.debug_bounds("screen-scroll").unwrap();
        assert!(
            form.right() <= clip.right() - px(24.),
            "The reserved scrollbar must not consume the form's right inset"
        );
        let requests = shell.read_with(cx, |s, _| s.next_request);
        for section in [
            SettingsSection::Access,
            SettingsSection::Capture,
            SettingsSection::Website,
        ] {
            let action = cx
                .debug_bounds(match section {
                    SettingsSection::Website => "action-SettingsSection(Website)",
                    SettingsSection::Capture => "action-SettingsSection(Capture)",
                    SettingsSection::Access => "action-SettingsSection(Access)",
                })
                .unwrap();
            inside(action, navigation);
            cx.simulate_click(action.center(), gpui::Modifiers::none());
            assert_destination(&shell, cx, section);
            let (destination, next) = match section {
                SettingsSection::Website => (Control::Name, Control::Address),
                // The sole selected Desktop profile cannot be unchecked. Jump
                // focus must start at the first enabled size, not stay behind.
                SettingsSection::Capture => (Control::Guided(2), Control::Guided(3)),
                SettingsSection::Access => (
                    Control::Choice(Choice::PublicAccess),
                    Control::Choice(Choice::LoginRequired),
                ),
            };
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    assert_eq!(s.focused(window, cx), Some(destination));
                    s.move_focus(false, window, cx);
                    assert_eq!(s.focused(window, cx), Some(next));
                })
            });
            draw(cx);
            assert_eq!(
                navigation,
                cx.debug_bounds("settings-section-navigation").unwrap()
            );
            assert_eq!(footer, cx.debug_bounds("setup-footer-status").unwrap());
            shell.read_with(cx, |s, cx| {
                assert_eq!(
                    s.next_request, requests,
                    "Jumps must not dispatch any worker operation"
                );
                assert_eq!(s.name.read(cx).content.as_ref(), "Retained settings draft");
                assert!(s.unsaved(cx));
                assert!(s.pending.is_none());
            });
        }
        // Guided setup and the project picker do not expose these destinations.
        shell.update(cx, |s, cx| {
            s.focus.setup_stage = SetupStage::Website;
            for section in SettingsSection::ALL {
                assert!(!s.control_enabled(Control::SettingsSection(section), cx));
            }
            s.focus.setup_stage = SetupStage::Settings;
        });
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    owned.join_after_ui();
}

#[gpui::test]
fn project_settings_scrollbar_pages_drags_outside_and_updates_location(cx: &mut TestAppContext) {
    use regressee_app::ui::scrollbar::ScrollbarGeometry;
    let dir = tempfile::tempdir().unwrap();
    let (_, project, _, _, _) = approval_fixture::fixture(dir.path());
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| prepare(s, project, cx));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(
                    Control::SettingsSection(SettingsSection::Website),
                    window,
                    cx,
                )
            })
        });
        draw(cx);
        let gutter = cx.debug_bounds("settings-scrollbar").unwrap();
        let clip = cx.debug_bounds("screen-scroll").unwrap();
        assert!(
            gutter.left() >= clip.right(),
            "Scrollbar must not cover the form"
        );
        let geometry = shell.read_with(cx, |s, _| {
            ScrollbarGeometry::new(
                gutter,
                f32::from(clip.size.height),
                f32::from(s.screen_scroll.max_offset().height),
                f32::from(s.screen_scroll.offset().y),
            )
            .unwrap()
        });
        let focus = cx.update(|window, cx| shell.read_with(cx, |s, cx| s.focused(window, cx)));
        cx.simulate_click(
            gpui::point(gutter.center().x, gutter.bottom() - px(12.)),
            gpui::Modifiers::none(),
        );
        draw(cx);
        assert!(shell.read_with(cx, |s, _| s.screen_scroll.offset().y < px(0.)));
        assert_eq!(
            focus,
            cx.update(|window, cx| shell.read_with(cx, |s, cx| s.focused(window, cx)))
        );
        // Grab the current thumb, drag beyond the entire reading clip, release
        // outside, then move again: no stuck drag and no stolen field focus.
        let current = shell.read_with(cx, |s, _| {
            ScrollbarGeometry::new(
                gutter,
                f32::from(clip.size.height),
                f32::from(s.screen_scroll.max_offset().height),
                f32::from(s.screen_scroll.offset().y),
            )
            .unwrap()
        });
        cx.simulate_mouse_down(
            current.thumb.center(),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        cx.simulate_mouse_move(
            gpui::point(gutter.center().x - px(50.), gutter.bottom() + px(100.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        draw(cx);
        shell.read_with(cx, |s, _| {
            assert_eq!(
                s.screen_scroll.offset().y,
                -s.screen_scroll.max_offset().height
            );
            assert_eq!(s.current_settings_section(), SettingsSection::Access);
        });
        cx.simulate_mouse_up(
            gpui::point(gutter.left() - px(50.), gutter.bottom() + px(100.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        let offset = shell.read_with(cx, |s, _| s.screen_scroll.offset());
        cx.simulate_mouse_move(geometry.thumb.center(), None, gpui::Modifiers::none());
        draw(cx);
        assert_eq!(offset, shell.read_with(cx, |s, _| s.screen_scroll.offset()));
        let bottom = shell.read_with(cx, |s, _| {
            ScrollbarGeometry::new(
                gutter,
                f32::from(clip.size.height),
                f32::from(s.screen_scroll.max_offset().height),
                f32::from(s.screen_scroll.offset().y),
            )
            .unwrap()
        });
        cx.simulate_mouse_down(
            bottom.thumb.center(),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        cx.simulate_mouse_move(
            gpui::point(gutter.center().x, gutter.top() - px(100.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        cx.simulate_mouse_up(
            gpui::point(gutter.center().x, gutter.top() - px(100.)),
            MouseButton::Left,
            gpui::Modifiers::none(),
        );
        draw(cx);
        assert_eq!(
            shell.read_with(cx, |s, _| s.screen_scroll.offset().y),
            px(0.)
        );
        cx.simulate_event(gpui::ScrollWheelEvent {
            position: gutter.center(),
            delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), px(-150.))),
            ..Default::default()
        });
        draw(cx);
        assert_eq!(
            shell.read_with(cx, |s, _| s.screen_scroll.offset().y),
            px(-150.)
        );
    }
    // Actual no-overflow layout: no painted/hit-test scrollbar to suggest scrolling.
    cx.simulate_resize(size(px(1280.), px(5000.)));
    draw(cx);
    shell.read_with(cx, |s, _| {
        assert_eq!(s.screen_scroll.max_offset().height, px(0.))
    });
    owned.join_after_ui();
}

#[gpui::test]
fn project_settings_tab_order_matches_painted_form_and_reveals_complete_controls(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (_, project, _, _, _) = approval_fixture::fixture(dir.path());
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| prepare(s, project, cx));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        for advanced in [false, true] {
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.focus.advanced = advanced;
                    window.focus(&s.handle(Control::SettingsSection(SettingsSection::Website), cx));
                    cx.notify();
                })
            });
            draw(cx);
            let order = shell.read_with(cx, |s, cx| {
                s.visible_controls()
                    .into_iter()
                    .filter(|c| s.control_enabled(*c, cx))
                    .collect::<Vec<_>>()
            });
            let start = order
                .iter()
                .position(|c| *c == Control::SettingsSection(SettingsSection::Website))
                .unwrap();
            let end = order.iter().position(|c| *c == Control::Save).unwrap();
            for reverse in [false, true] {
                let targets: Vec<_> = if reverse {
                    order[start..end].iter().rev().copied().collect()
                } else {
                    order[start + 1..=end].to_vec()
                };
                for target in targets {
                    cx.update(|window, cx| {
                        shell.update(cx, |s, cx| {
                            s.move_focus(reverse, window, cx);
                            assert_eq!(s.focused(window, cx), Some(target));
                        })
                    });
                    draw(cx);
                    if let Some(bounds) = shell.read_with(cx, |s, _| {
                        s.setup_control_bounds(target).and_then(|b| b.get())
                    }) {
                        inside(bounds, cx.debug_bounds("screen-scroll").unwrap());
                    }
                }
            }
        }
    }
    owned.join_after_ui();
}
