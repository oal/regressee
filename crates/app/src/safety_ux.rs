// SPDX-License-Identifier: MPL-2.0
//! Explicit, snapshot-bound consent before discarding user work or deleting data.
use super::*;

struct Confirmation {
    title: String,
    message: String,
    keep: &'static str,
    confirm: &'static str,
    restore: Control,
}

impl Shell {
    fn confirm_destructive(
        &mut self,
        prompt: Confirmation,
        window: &mut Window,
        cx: &mut Context<Self>,
        commit: impl FnOnce(&mut Self, &mut Context<Self>) + 'static,
    ) {
        self.native_prompt = true;
        let answer = window.prompt(
            PromptLevel::Warning,
            &prompt.title,
            Some(&prompt.message),
            &[prompt.keep, prompt.confirm],
            cx,
        );
        cx.spawn_in(window, async move |this, cx| {
            let confirmed = answer.await == Ok(1);
            let _ = this.update_in(cx, |this, window, cx| {
                this.native_prompt = false;
                if confirmed {
                    commit(this, cx);
                }
                let restore = if this.control_enabled(prompt.restore, cx) {
                    prompt.restore
                } else if this.control_enabled(Control::CaptureBaseline, cx) {
                    Control::CaptureBaseline
                } else {
                    Control::Navigate(this.focus.screen)
                };
                window.focus(&this.handle(restore, cx));
                cx.notify();
            });
        })
        .detach();
        cx.notify();
    }

    /// Pointer and keyboard activation share this gate. A dismissed/failed prompt
    /// never dispatches work. Backend revision/fingerprint guards remain intact.
    pub(super) fn request_destructive_control(
        &mut self,
        control: Control,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) -> bool {
        let base = |title: &str, message: String, confirm| Confirmation {
            title: title.into(),
            message,
            keep: "Cancel",
            confirm,
            restore: control,
        };
        match control {
            Control::DiscardPageList => {
                let Some(preview) = self.preview.clone() else {
                    return true;
                };
                let mut prompt = base(
                    "Discard discovered pages?",
                    format!(
                        "This removes the unsaved list of {} and all its include/exclude choices. It does not delete recorded screenshots or change the saved baseline. To continue afterward, find pages again or use a recorded page list if available.",
                        workflow::count_label(
                            preview.inventory.pages.len(),
                            "page address",
                            "page addresses"
                        )
                    ),
                    "Discard pages",
                );
                prompt.keep = "Keep page list";
                self.confirm_destructive(prompt, window, cx, move |s, _| {
                    if s.preview.as_ref().is_some_and(|p| {
                        p.inventory == preview.inventory
                            && p.project_revision == preview.project_revision
                            && p.settings == preview.settings
                    }) {
                        s.preview = None;
                        s.focus.preview = false;
                        s.selected_page = None;
                        s.result_filters_open = false;
                        s.discovery_details_open = false;
                        s.focus.review_surface = ReviewSurface::Pages;
                        s.notice = Some(if s.evidence.latest.is_some() {
                            "Discovered pages discarded. Use recorded pages to prepare another capture, or Find pages to crawl again. Saved screenshots and the baseline are unchanged.".into()
                        } else {
                            "Discovered pages discarded. Choose Find pages to start again; the baseline is unchanged.".into()
                        });
                    }
                });
            }
            Control::Guided(34) => {
                let Some(project) = self.project.clone() else {
                    return true;
                };
                let signature = self.environment_draft_signature(cx);
                let name = self
                    .environment
                    .as_ref()
                    .map_or("selected environment", |e| e.name.as_str());
                let prompt = base(
                    "Forget this website login?",
                    format!(
                        "Remove Regressee's saved login for {name}. You will need to log in again before capturing private pages. This does not log you out of other browsers or delete screenshots, projects or baselines."
                    ),
                    "Forget login",
                );
                self.confirm_destructive(prompt, window, cx, move |s, cx| {
                    if s.project.as_ref().is_some_and(|p| {
                        p.project == project.project && p.revision == project.revision
                    }) && s.environment_draft_signature(cx) == signature
                    {
                        s.queue(Work::ForgetLogin(project.project.id), cx);
                    }
                });
            }
            Control::Phase5(42) => {
                let (Some(project), Some(baseline)) = (
                    &self.project,
                    self.evidence.baselines.get(self.history_index),
                ) else {
                    return true;
                };
                let (project, revision, baseline) =
                    (project.project.id, project.revision, baseline.clone());
                let prompt = base(
                    "Use this saved baseline?",
                    format!(
                        "Future checks will compare against the baseline saved {}. Your current baseline and other saved screenshots stay in History. No website is visited.",
                        workspace_history::history_date(baseline.created_at)
                    ),
                    "Use baseline",
                );
                self.confirm_destructive(prompt, window, cx, move |s, cx| {
                    if s.project
                        .as_ref()
                        .is_some_and(|p| p.project.id == project && p.revision == revision)
                        && s.evidence
                            .baselines
                            .get(s.history_index)
                            .is_some_and(|b| b.id == baseline.id)
                    {
                        s.queue(
                            Work::Restore {
                                baseline: baseline.id,
                                revision,
                            },
                            cx,
                        );
                    }
                });
            }
            Control::Phase6(6) => {
                let Some(preview) = self.deletion_preview.clone() else {
                    return true;
                };
                let prompt = base(
                    "Permanently delete the selected history record?",
                    format!(
                        "Delete the record shown in the preview, {} and {} used only by this record ({}). This cannot be undone. Files shared with other records or needed by saved baselines will be kept.",
                        workflow::count_label(preview.analyses, "analysis", "analyses"),
                        workflow::count_label(preview.objects, "file", "files"),
                        human_bytes(preview.bytes)
                    ),
                    "Delete permanently",
                );
                self.confirm_destructive(prompt, window, cx, move |s, cx| {
                    if s.deletion_preview.as_ref() == Some(&preview) {
                        s.queue(Work::Delete(Box::new(preview)), cx);
                    }
                });
            }
            Control::Phase6(index @ (14 | 15)) => {
                let Some(preview) = self.repair_preview.clone() else {
                    return true;
                };
                let prompt = if index == 14 {
                    base(
                        "Discard interrupted temporary files?",
                        format!(
                            "Permanently remove {} and {} ({}). Files still being saved, complete recovery files, saved screenshots and baselines will be kept. This cannot be undone.",
                            workflow::count_label(
                                preview.temporaries.len(),
                                "temporary file",
                                "temporary files"
                            ),
                            workflow::count_label(
                                preview.interrupted_imports.len(),
                                "interrupted import",
                                "interrupted imports"
                            ),
                            human_bytes(preview.temporary_bytes)
                        ),
                        "Discard temporary files",
                    )
                } else {
                    base(
                        "Permanently delete recovery originals?",
                        format!(
                            "Permanently remove {} not used by saved records. These may contain screenshots you could otherwise recover. This cannot be undone. Files used by saved records and baselines will be kept.",
                            workflow::count_label(
                                preview.valid_unreferenced.len(),
                                "complete file",
                                "complete files"
                            )
                        ),
                        "Delete recovery originals",
                    )
                };
                self.confirm_destructive(prompt, window, cx, move |s, cx| {
                    if s.repair_preview.as_ref() == Some(&preview) {
                        let work = if index == 14 {
                            Work::CleanupTemporaries(Box::new(preview))
                        } else {
                            Work::DeleteUnreferenced(Box::new(preview))
                        };
                        s.queue(work, cx);
                    }
                });
            }
            Control::SetupRuntime
                if self.runtime_readiness.as_ref().is_some_and(|s| {
                    matches!(
                        s.status,
                        regressee_app::product::RuntimeState::RepairRequired
                            | regressee_app::product::RuntimeState::UpdateRequired
                    )
                }) =>
            {
                let prompt = base(
                    "Replace the private browser components?",
                    "Download and install the browser components for this app version. Saved projects, screenshots and baselines will be kept. Checks need the browser version used for the baseline, so an older baseline may still need a different version.".into(),
                    "Replace browser components",
                );
                self.confirm_destructive(prompt, window, cx, |s, cx| {
                    s.queue(Work::SetupRuntime, cx);
                    s.focus.capturing = s.pending.is_some();
                });
            }
            Control::Guided(13) if self.preview.is_some() => {
                let preview = self.preview.as_ref().unwrap().inventory.clone();
                let prompt = base(
                    "Replace the discovered page list?",
                    "Finding pages again replaces the current unsaved list and its include/exclude choices. Saved screenshots and baselines stay unchanged. Cancel to keep reviewing this list.".into(),
                    "Find pages again",
                );
                self.confirm_destructive(prompt, window, cx, move |s, cx| {
                    if s.preview.as_ref().is_some_and(|p| p.inventory == preview) {
                        s.preview = None;
                        s.focus.preview = false;
                        s.guided(13, cx);
                    }
                });
            }
            _ => return false,
        }
        true
    }
}
