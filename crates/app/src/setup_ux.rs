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
