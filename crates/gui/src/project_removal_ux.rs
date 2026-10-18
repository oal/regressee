// SPDX-License-Identifier: MPL-2.0
//! Project lifecycle actions, distinct from editable mapping drafts and History.
use super::*;
use regressee_storage::{DeleteItem, DeletionPreview};

impl Shell {
    pub(super) fn scope_deletion_preview(&self) -> bool {
        self.deletion_preview
            .as_ref()
            .is_some_and(|p| p.scope.is_some())
    }
    pub(super) fn removal_enabled(&self, control: Control, cx: &App) -> bool {
        if self.pending.is_some()
            || self.analysis_pending.is_some()
            || self.analysis_due.is_some()
            || self.project_input_composing(cx)
        {
            return false;
        }
        match control {
            Control::RemoveEnvironment => self.environment_draft.is_some_and(|id| {
                self.draft_settings.environments.len() > 1
                    && self.draft_settings.environments.iter().any(|e| e.id == id)
                    && (self
                        .project
                        .as_ref()
                        .is_some_and(|p| !p.project.environments.iter().any(|e| e.id == id))
                        || !self.unsaved(cx))
            }),
            Control::RemoveMapping(index) => self
                .draft_settings
                .environments
                .iter()
                .find(|e| Some(e.id) == self.environment_draft)
                .is_some_and(|e| e.mappings.len() > 1 && index < e.mappings.len()),
            Control::DeleteProject(index) => index < self.projects.items.len(),
            Control::DeleteCurrentProject => self.project.is_some(),
            _ => false,
        }
    }
    pub(super) fn request_removal(
        &mut self,
        control: Control,
        _: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match control {
            Control::RemoveMapping(index) => {
                if let Some(env) = self
                    .draft_settings
                    .environments
                    .iter_mut()
                    .find(|e| Some(e.id) == self.environment_draft)
                    && env.mappings.len() > 1
                    && index < env.mappings.len()
                {
                    env.mappings.remove(index);
                    self.focus.mappings = env.mappings.len();
                    self.notice = Some("Address mapping removed from the draft. Save settings to apply it; recorded captures are unchanged.".into());
                    self.presentation_focus = Some(Control::Guided(37));
                }
            }
            Control::RemoveEnvironment => {
                let Some(project) = self.project.clone() else {
                    return;
                };
                let Some(environment) = self.environment_draft else {
                    return;
                };
                if !project
                    .project
                    .environments
                    .iter()
                    .any(|e| e.id == environment)
                {
                    self.draft_settings
                        .environments
                        .retain(|e| e.id != environment);
                    self.focus.environments = self.draft_settings.environments.len();
                    self.environment_draft = self
                        .environment
                        .as_ref()
                        .map(|e| e.environment)
                        .filter(|id| self.draft_settings.environments.iter().any(|e| e.id == *id))
                        .or_else(|| self.draft_settings.environments.first().map(|e| e.id));
                    let settings = self
                        .environment_draft
                        .and_then(|id| self.saved_login_forms.get(&(project.project.id, id)))
                        .cloned()
                        .unwrap_or_default();
                    self.login_ephemeral = settings.ephemeral;
                    for (index, value) in [
                        (
                            10,
                            settings
                                .login_address
                                .map(|u| u.to_string())
                                .unwrap_or_default(),
                        ),
                        (
                            11,
                            settings
                                .representative
                                .map(|u| u.to_string())
                                .unwrap_or_default(),
                        ),
                        (
                            12,
                            settings
                                .identity_origins
                                .iter()
                                .map(|u| u.as_str())
                                .collect::<Vec<_>>()
                                .join(" "),
                        ),
                    ] {
                        self.crawl_fields[index]
                            .update(cx, |input, cx| input.set_value(&value, cx));
                    }
                    self.focus.mappings = self
                        .draft_settings
                        .environments
                        .iter()
                        .find(|e| Some(e.id) == self.environment_draft)
                        .map_or(0, |e| e.mappings.len());
                    self.notice = Some("Unsaved environment removed. Recorded captures and saved logins are unchanged.".into());
                    self.presentation_focus = Some(Control::Guided(27));
                } else {
                    self.clear_portability_previews();
                    self.removal_restore = Some(control);
                    self.queue(
                        Work::DeletePreview {
                            project: project.project.id,
                            item: DeleteItem::Environment(environment),
                        },
                        cx,
                    );
                }
            }
            Control::DeleteProject(index) => {
                let Some(project) = self.projects.items.get(index).cloned() else {
                    return;
                };
                self.clear_portability_previews();
                self.removal_restore = Some(control);
                self.queue(
                    Work::DeletePreview {
                        project: project.id,
                        item: DeleteItem::Project,
                    },
                    cx,
                );
                if let Some(task) = &mut self.task {
                    task.project_id = Some(project.id);
                    task.project = project.name;
                    task.website = project.address;
                    task.environment.clear();
                }
            }
            Control::DeleteCurrentProject => {
                let Some(project) = self.project.as_ref().map(|p| p.project.id) else {
                    return;
                };
                self.clear_portability_previews();
                self.removal_restore = Some(control);
                self.queue(
                    Work::DeletePreview {
                        project,
                        item: DeleteItem::Project,
                    },
                    cx,
                );
            }
            _ => (),
        }
        cx.notify();
    }
    pub(super) fn project_deletion_section(&self, cx: &mut Context<Self>) -> Div {
        div().w_full().border_t_1().border_color(rgb(palette().line)).pt_5().flex().flex_col().gap_2()
            .when(self.project.is_some(), |section| section
                .debug_selector(||"project-deletion-section".into())
                .child(div().font_weight(FontWeight::SEMIBOLD).child("Delete project"))
                .child(div().text_size(px(HELP_TEXT_SIZE)).line_height(px(HELP_LINE_HEIGHT)).text_color(rgb(palette().muted))
                    .child("Permanently remove this project's settings, saved captures, baselines, review history and logins. Review what will be removed before confirming."))
                .child(self.button(Control::DeleteCurrentProject,"Delete project…",ActionStyle::Danger,cx)))
    }
    pub(super) fn scope_deletion_summary(&self, preview: &DeletionPreview) -> String {
        let scope = preview.scope.as_ref().expect("scope preview");
        let login = if scope.logins > 0 {
            format!(
                " {} will also be forgotten.",
                workflow::count_label(scope.logins, "saved login", "saved logins")
            )
        } else {
            String::new()
        };
        if matches!(preview.item, DeleteItem::Environment(_)) {
            return format!(
                "Remove this environment's saved settings.{login} Recorded captures, baseline versions and other environments stay unchanged. At least one environment will remain."
            );
        }
        let mut message = if scope.runs == 0 && scope.baselines == 0 {
            "Delete this project's settings. It has no saved captures or baselines.".into()
        } else {
            let mut records = Vec::new();
            if scope.runs > 0 {
                records.push(workflow::count_label(
                    scope.runs,
                    "recorded run",
                    "recorded runs",
                ));
            }
            if scope.baselines > 0 {
                records.push(workflow::count_label(
                    scope.baselines,
                    "baseline version",
                    "baseline versions",
                ));
            }
            format!(
                "Delete this project's settings and {}, including saved captures, review history and ignored areas.",
                records.join(" and ")
            )
        };
        message.push_str(&login);
        if preview.objects > 0 {
            message.push_str(&format!(
                "\n\n{} used only by this project ({}) will be deleted.",
                workflow::count_label(preview.objects, "file", "files"),
                human_bytes(preview.bytes)
            ));
        }
        if preview.shared_bytes > 0 {
            message.push_str(&format!(
                " {} of files shared with other projects will be kept.",
                human_bytes(preview.shared_bytes)
            ));
        }
        message.push_str("\n\nOther projects, the installed browser and exported ZIPs stay unchanged. This cannot be undone.");
        message
    }
    pub(super) fn receive_scope_deletion(
        &mut self,
        project: regressee_core::domain::ProjectId,
        item: DeleteItem,
        cleanup_pending: bool,
        cx: &mut Context<Self>,
    ) -> bool {
        let Some(preview) = self
            .deletion_preview
            .clone()
            .filter(|p| p.scope.is_some() && p.project == project && p.item == item)
        else {
            return false;
        };
        let scope = preview.scope.as_ref().unwrap();
        self.clear_portability_previews();
        self.projects_dirty = true;
        self.summary = None;
        self.saved_login_forms.retain(|(project, environment), _| {
            *project != preview.project
                || matches!(preview.item,DeleteItem::Environment(id) if id!=*environment)
        });
        if preview.item == DeleteItem::Project {
            if self
                .project
                .as_ref()
                .is_some_and(|p| p.project.id == preview.project)
            {
                if let Some((_, cancel)) = self.analysis_pending.take() {
                    cancel.store(true, std::sync::atomic::Ordering::Release);
                }
                self.analysis_due = None;
                self.passive_evidence_pending = None;
                self.passive_history_pending = None;
                self.viewer_pending = None;
                self.storage_pending = None;
                self.runtime_pending = None;
                self.invalidate_approval();
                self.project = None;
                self.focus.project_saved = false;
                self.focus.baseline = false;
                self.focus.candidate = false;
                self.focus.viewer = false;
                self.focus.preview = false;
                self.focus.resumable = false;
                self.focus.view_loading = false;
                self.evidence = ProjectEvidence::default();
                self.evidence_dirty = false;
                self.evidence_read_failed = false;
                self.history = None;
                self.history_inspection = None;
                self.project_storage = None;
                self.preview = None;
                self.selected_key = None;
                self.selected_page = None;
                self.mask_rectangles.clear();
                self.mask_dirty = false;
                self.mask_drag = None;
                self.mask_selected = None;
                self.mask_draw = false;
                for input in &self.analysis_fields[10..14] {
                    input.update(cx, |i, cx| i.set_value("", cx));
                }
                self.discard_review_drafts(cx);
                self.clear_viewer();
                self.dom.selected.clear();
                self.dom.approval = None;
                self.environment = None;
                self.environment_draft = None;
                self.environment_read_signature = None;
                self.login_hydration_origin = None;
                self.login_ephemeral = false;
                self.name.update(cx, |i, cx| i.set_value("", cx));
                self.address.update(cx, |i, cx| i.set_value("", cx));
                for input in &self.crawl_fields[10..13] {
                    input.update(cx, |i, cx| i.set_value("", cx));
                }
                self.draft_settings =
                    Project::draft("New website".into(), "https://example.test/").unwrap();
                self.set_crawl_fields(cx);
            }
            self.focus.screen = Screen::Projects;
            self.focus.choosing_project = false;
            self.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
            self.notice = Some(if cleanup_pending {
                format!(
                    "Project “{}” deleted, but file cleanup could not finish. Remaining cleanup will retry when Regressee starts. Other projects and shared files were kept.",
                    scope.name
                )
            } else {
                format!(
                    "Project “{}” deleted. Other projects and shared files were kept.",
                    scope.name
                )
            });
            self.presentation_focus = Some(Control::NewProject);
            self.queue(Work::List { page: 0 }, cx);
        } else {
            self.notice = Some(format!(
                "Environment “{}” removed. Recorded captures and baselines were kept.",
                scope.name
            ));
            self.environment = None;
            self.environment_draft = None;
            self.login_ephemeral = false;
            for input in &self.crawl_fields[10..13] {
                input.update(cx, |i, cx| i.set_value("", cx));
            }
            self.queue(Work::Load(preview.project), cx);
        }
        cx.notify();
        true
    }
}
