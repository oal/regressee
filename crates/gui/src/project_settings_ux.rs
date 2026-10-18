// SPDX-License-Identifier: MPL-2.0
//! Wayfinding within one settings draft and one existing scroll owner.
use super::*;

fn section_at(positions: &[Option<Pixels>; 3], offset: Pixels) -> SettingsSection {
    SettingsSection::ALL
        .into_iter()
        .rev()
        .find(|section| positions[section.index()].is_some_and(|top| top <= -offset + px(24.)))
        .unwrap_or(SettingsSection::Website)
}

impl Shell {
    pub(super) fn project_settings_open(&self) -> bool {
        self.focus.screen == Screen::Setup
            && self.focus.setup_stage == SetupStage::Settings
            && !self.focus.shows_project_picker()
    }
    pub(super) fn current_settings_section(&self) -> SettingsSection {
        section_at(
            &std::array::from_fn(|i| self.settings_section_positions[i].get()),
            self.screen_scroll.offset().y,
        )
    }
    pub(super) fn settings_navigation(&self, cx: &mut Context<Self>) -> AnyElement {
        let current = self.current_settings_section();
        div()
            .debug_selector(|| "settings-section-navigation".into())
            .mt_3()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_2()
            .child(
                div()
                    .flex()
                    .items_center()
                    .h(px(CONTROL_HEIGHT))
                    .flex_shrink_0()
                    .text_size(px(HELP_TEXT_SIZE))
                    .line_height(px(HELP_LINE_HEIGHT))
                    .text_color(rgb(palette().muted))
                    .child("Jump to"),
            )
            .children(
                SettingsSection::ALL
                    .into_iter()
                    .filter(|section| *section != SettingsSection::Access || self.project.is_some())
                    .map(|section| {
                        self.button(
                            Control::SettingsSection(section),
                            section.label(),
                            ActionStyle::Anchor {
                                selected: section == current,
                            },
                            cx,
                        )
                    }),
            )
            .into_any_element()
    }
    pub(super) fn measured_settings_section(&self, section: SettingsSection, body: Div) -> Div {
        let position = self.settings_section_positions[section.index()].clone();
        let scroll = self.screen_scroll.clone();
        body.relative()
            .debug_selector(move || format!("settings-section-{section:?}"))
            .child(
                canvas(
                    move |bounds, _, _| {
                        // Store content coordinates, not stale viewport coordinates:
                        // wheel, drag and keyboard reveal can use them immediately.
                        position.set(Some(
                            bounds.top() - scroll.bounds().top() - scroll.offset().y,
                        ));
                    },
                    |_, _, _, _| (),
                )
                .absolute()
                .top_0()
                .left_0()
                .size_full(),
            )
    }
    pub(super) fn settings_location_sync(&self, cx: &Context<Self>) -> impl IntoElement {
        let positions = self.settings_section_positions.clone();
        let scroll = self.screen_scroll.clone();
        let rendered = self.current_settings_section();
        let owner = cx.entity_id();
        canvas(
            |_, _, _| (),
            move |_, _, window, _| {
                if section_at(
                    &std::array::from_fn(|i| positions[i].get()),
                    scroll.offset().y,
                ) != rendered
                {
                    // The new layout may move section starts after a resize or
                    // disclosure. Update location once, without an animation loop.
                    window.on_next_frame(move |_, cx| cx.notify(owner));
                }
            },
        )
        .h(px(0.))
        .w_full()
    }
    pub(super) fn jump_to_settings_section(
        &mut self,
        section: SettingsSection,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let Some(top) = self.settings_section_positions[section.index()].get() else {
            return;
        };
        let destination = match section {
            SettingsSection::Website => Control::Name,
            SettingsSection::Capture => Control::CaptureMode(self.draft_settings.capture_plan.mode),
            SettingsSection::Access => Control::Choice(Choice::PublicAccess),
        };
        self.settings_scrollbar.cancel_drag();
        self.screen_scroll.set_offset(gpui::point(
            px(0.),
            (px(16.) - top).clamp(-self.screen_scroll.max_offset().height, px(0.)),
        ));
        // Explicit navigation places the next Tab at its destination; passive
        // scrolling never steals field focus or edits the unified draft.
        if self.control_enabled(destination, cx) {
            window.focus(&self.handle(destination, cx));
        }
        cx.notify();
    }
}
