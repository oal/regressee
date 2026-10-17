// SPDX-License-Identifier: MPL-2.0
//! Guided setup continuations are bound to a request and the exact submitted draft.
use super::*;

pub(super) struct SetupSave {
    pub request: u64,
    pub signature: String,
    pub next: Option<SetupStage>,
    pub discover: bool,
    pub project_before: Option<regressee_core::domain::ProjectId>,
    pub submitted_project: regressee_core::domain::ProjectId,
    pub environment: Option<regressee_core::domain::EnvironmentId>,
    pub login: Option<regressee_storage::LocalLoginSettings>,
}

impl Shell {
    pub(super) fn setup_login_fields_visible(&self) -> bool {
        self.focus.setup_stage == SetupStage::Settings
            || (self.setup_environment_open && self.setup_login_correction)
            || self
                .draft_settings
                .environments
                .iter()
                .find(|e| Some(e.id) == self.environment_draft)
                .is_some_and(|e| e.access == regressee_core::domain::AccessPolicy::InteractiveLogin)
    }
    pub(super) fn refresh_setup_origins(&mut self, cx: &App) {
        let origins = crawl::sources(&self.address.read(cx).content)
            .map(|sources| {
                sources
                    .iter()
                    .map(crawl::origin)
                    .collect::<std::collections::BTreeSet<_>>()
            })
            .unwrap_or_default();
        self.setup_origin_count = origins.len();
        if self.origins_confirmed && origins != self.approved_origins {
            self.origins_confirmed = false;
            self.approved_origins.clear();
        }
    }
    pub(super) fn setup_progress(&self, cx: &mut Context<Self>) -> AnyElement {
        use regressee_gui::ui::setup_progress::{self, StepState};
        let mut progress = div()
            .debug_selector(|| "setup-progress".into())
            .w(px(self.working_width.min(680.)))
            .flex()
            .items_center();
        for (index, (stage, label)) in [
            (SetupStage::Website, "Website"),
            (SetupStage::Access, "Access"),
            (SetupStage::Pages, "Pages & sizes"),
        ]
        .into_iter()
        .enumerate()
        {
            if index > 0 {
                progress = progress.child(setup_progress::connector());
            }
            let control = Control::SetupStage(stage);
            let enabled = self.control_enabled(control, cx);
            let step = setup_progress::step(
                format!("{control:?}").into(),
                index + 1,
                label,
                StepState::at(index, self.focus.setup_stage.index()),
                &self.handle(control, cx),
                enabled,
            )
            .when(enabled, |el| {
                el.on_click(
                    cx.listener(move |s, event, w, cx| s.click_control(control, event, w, cx)),
                )
            });
            progress = progress.child(step);
        }
        progress.into_any_element()
    }
    pub(super) fn setup_stage_available(&self, stage: SetupStage, cx: &App) -> bool {
        !self.focus.busy
            && (stage.index() <= self.focus.setup_stage.index()
                || (self.project.is_some()
                    && !self.unsaved(cx)
                    && !self.setup_composing(cx)
                    && (stage != SetupStage::Pages || self.access_can_continue(cx))))
    }
    pub(super) fn setup_back(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        self.focus.setup_stage = match self.focus.setup_stage {
            SetupStage::Access => SetupStage::Website,
            SetupStage::Pages => SetupStage::Access,
            _ => return,
        };
        self.show_mapping = false;
        self.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
        w.focus(&self.handle(Control::SetupStage(self.focus.setup_stage), cx));
        cx.notify();
    }
    pub(super) fn toggle_environment_options(&mut self, w: &mut Window, cx: &mut Context<Self>) {
        self.setup_environment_open = !self.setup_environment_open;
        if !self.setup_environment_open {
            self.show_mapping = false;
            self.setup_login_correction = false;
            self.timezone.update(cx, |s, _| s.set_enabled(false));
        }
        self.reveal_form_control(Control::EnvironmentOptions, false, w, cx);
        w.focus(&self.handle(Control::EnvironmentOptions, cx));
        self.disclosure_reveal
            .set(Some(Control::EnvironmentOptions));
        cx.notify();
    }
    pub(super) fn setup_composing(&self, cx: &App) -> bool {
        self.name.read(cx).composing()
            || self.address.read(cx).composing()
            || self
                .crawl_fields
                .iter()
                .take(17)
                .any(|f| f.read(cx).composing())
    }
    pub(super) fn setup_signature(&self, cx: &App) -> String {
        format!(
            "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}",
            (self.focus.screen, self.focus.setup_stage),
            self.project.as_ref().map(|p| (p.project.id, p.revision)),
            self.environment_draft,
            (&self.name.read(cx).content, self.name.read(cx).composing()),
            (
                &self.address.read(cx).content,
                self.address.read(cx).composing()
            ),
            self.draft_settings,
            self.crawl_fields
                .iter()
                .take(17)
                .map(|f| (f.read(cx).content.to_string(), f.read(cx).composing()))
                .collect::<Vec<_>>(),
            self.login_ephemeral,
            self.timezone.read(cx).selected
        )
    }

    pub(super) fn access_can_continue(&self, cx: &App) -> bool {
        !self.unsaved(cx)
            && !self.setup_composing(cx)
            && self.project.as_ref().is_some_and(|p| {
                p.project
                    .environments
                    .iter()
                    .find(|e| Some(e.id) == self.environment_draft)
                    .is_some_and(|e| {
                        e.access == regressee_core::domain::AccessPolicy::Public
                            || (e.session.is_some()
                                && self
                                    .environment
                                    .as_ref()
                                    .is_some_and(|s| s.environment == e.id))
                    })
            })
    }

    pub(super) fn guided_save(&mut self, discover: bool, cx: &mut Context<Self>) {
        if self.pending.is_some()
            || self.name.read(cx).composing()
            || self.address.read(cx).composing()
            || self
                .crawl_fields
                .iter()
                .take(17)
                .any(|f| f.read(cx).composing())
        {
            return;
        }
        let stage = self.focus.setup_stage;
        if !discover && !self.unsaved(cx) && self.project.is_some() {
            let next = match stage {
                SetupStage::Website => Some(SetupStage::Access),
                SetupStage::Access if self.access_can_continue(cx) => Some(SetupStage::Pages),
                _ => None,
            };
            if let Some(next) = next {
                self.focus.setup_stage = next;
                self.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
                cx.notify();
                return;
            }
        }
        self.setup_save = None;
        self.save_settings(cx);
        if let Some(save) = &mut self.setup_save {
            save.discover = discover;
            save.next = if discover {
                None
            } else {
                match stage {
                    SetupStage::Website => Some(SetupStage::Access),
                    SetupStage::Access => Some(SetupStage::Pages),
                    _ => None,
                }
            };
        }
    }
}
