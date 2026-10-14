// SPDX-License-Identifier: MPL-2.0
//! Pre-destruction exit protection. `on_app_quit` is cleanup, not a veto hook.
use super::*;
use std::collections::VecDeque;

fn validate_exit_coverage(
    rectangles: &[regressee_app::analysis_engine::Rectangle],
    before: regressee_app::analysis_engine::Dimensions,
    after: regressee_app::analysis_engine::Dimensions,
) -> anyhow::Result<()> {
    anyhow::ensure!(
        !review_ux::all_masked(rectangles, before, after),
        "Ignored areas cover all screenshot pixels. Reduce or delete an area before saving; this cannot be a clean result."
    );
    Ok(())
}
fn validate_exit_pair(
    request: &regressee_app::analysis_engine::AnalysisRequest,
) -> anyhow::Result<()> {
    request.validate().map_err(|e| {
        anyhow::anyhow!(
            "Check that ignored areas fit these screenshots before quitting. Open Analysis, then choose Check areas fit these screenshots. Details: {e}"
        )
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Phase {
    Idle,
    Prompt,
    Saving,
    Confirmed,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SaveKind {
    Mask,
    Note,
    Project,
    Network,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Decision {
    Cancel,
    Discard,
    Save,
}

pub(super) struct ExitGuard {
    phase: Phase,
    remaining: VecDeque<SaveKind>,
    pending: Option<(u64, SaveKind)>,
    signature: String,
    login: Option<regressee_storage::LocalLoginSettings>,
    return_focus: Option<FocusHandle>,
    window: Option<gpui::AnyWindowHandle>,
    mask_saved: bool,
    notes: VecDeque<regressee_core::domain::ReviewDecision>,
    pending_note: Option<regressee_core::domain::ReviewDecision>,
}
impl Default for ExitGuard {
    fn default() -> Self {
        Self {
            phase: Phase::Idle,
            remaining: VecDeque::new(),
            pending: None,
            signature: String::new(),
            login: None,
            return_focus: None,
            window: None,
            mask_saved: false,
            notes: VecDeque::new(),
            pending_note: None,
        }
    }
}
impl ExitGuard {
    pub(super) fn saving(&self) -> bool {
        self.phase == Phase::Saving
    }
    fn request(&mut self, dirty: bool, pending: bool) -> bool {
        if self.phase != Phase::Idle || pending {
            return false;
        }
        self.phase = if dirty {
            Phase::Prompt
        } else {
            Phase::Confirmed
        };
        true
    }
    fn decide(&mut self, decision: Decision) {
        if self.phase != Phase::Prompt {
            return;
        }
        self.phase = match decision {
            Decision::Cancel => Phase::Idle,
            Decision::Discard => Phase::Confirmed,
            Decision::Save => Phase::Saving,
        };
    }
    fn fail(&mut self) {
        *self = Self::default();
    }
}

impl Shell {
    fn exit_network_text(&self, cx: &App) -> (String, String) {
        (
            self.crawl_fields[17].read(cx).content.to_string(),
            self.crawl_fields[18].read(cx).content.to_string(),
        )
    }
    fn exit_network_dirty(&self, cx: &App) -> bool {
        let text = self.exit_network_text(cx);
        self.exit_network_saved.as_ref().map_or_else(
            || !text.0.is_empty() || !text.1.is_empty(),
            |saved| &text != saved,
        )
    }
    fn exit_note_dirty(&self, cx: &App) -> bool {
        self.pending_review_notes(cx)
            .map_or(true, |notes| !notes.is_empty())
    }
    // In-progress "Add screen size" and origin-mapping forms are not yet project
    // settings. Do not silently apply them (or silently drop them) on exit.
    fn exit_builder_dirty(&self, cx: &App) -> bool {
        let width = self.crawl_fields[7].read(cx).content.trim();
        let height = self.crawl_fields[8].read(cx).content.trim();
        let custom_size = (width != "1440" || height != "900")
            && !self
                .draft_settings
                .profiles
                .iter()
                .any(|p| width == p.width.to_string() && height == p.height.to_string());
        let source = self.crawl_fields[13].read(cx).content.trim();
        let destination = self.crawl_fields[14].read(cx).content.trim();
        let source_url = url::Url::parse(source).ok();
        let destination_url = url::Url::parse(destination).ok();
        let mapping = (!source.is_empty() || !destination.is_empty())
            && !self
                .draft_settings
                .environments
                .iter()
                .flat_map(|e| &e.mappings)
                .any(|m| {
                    source_url.as_ref() == Some(&m.source)
                        && destination_url.as_ref() == Some(&m.destination)
                });
        custom_size || mapping
    }
    fn exit_project_dirty(&self, cx: &App) -> bool {
        self.environment_unsaved(cx)
            || self.project.as_ref().map_or_else(
                || {
                    let mut defaults = Project::draft("New website".into(), "https://example.test")
                        .expect("fixed draft defaults");
                    // New drafts get fresh IDs, but IDs alone are not user edits.
                    for (default, draft) in defaults
                        .profiles
                        .iter_mut()
                        .zip(&self.draft_settings.profiles)
                    {
                        default.id = draft.id;
                    }
                    for (default, draft) in defaults
                        .environments
                        .iter_mut()
                        .zip(&self.draft_settings.environments)
                    {
                        default.id = draft.id;
                    }
                    !self.name.read(cx).content.is_empty()
                        || !self.address.read(cx).content.is_empty()
                        || self.draft_settings.discovery != defaults.discovery
                        || self.draft_settings.profiles != defaults.profiles
                        || self.draft_settings.environments != defaults.environments
                        || self
                            .crawl_fields
                            .iter()
                            .take(7)
                            .zip(["500", "", "", "", "", "2", "2"])
                            .any(|(input, expected)| input.read(cx).content.as_ref() != expected)
                },
                |p| {
                    self.name.read(cx).content.as_ref() != p.project.name
                        || self.address.read(cx).content.as_ref()
                            != p.project
                                .sources
                                .iter()
                                .map(|u| u.as_str())
                                .collect::<Vec<_>>()
                                .join(" ")
                        || self.draft_settings.discovery != p.project.discovery
                        || self.draft_settings.profiles != p.project.profiles
                        || self.draft_settings.environments != p.project.environments
                        || p.project.profiles.first().is_some_and(|profile| {
                            self.crawl_fields[15].read(cx).content.trim() != profile.locale
                                || self.timezone.read(cx).selected.as_ref() != profile.timezone
                        })
                        || (self.environment.is_none()
                            && self.crawl_fields[10..13]
                                .iter()
                                .any(|v| !v.read(cx).content.is_empty()))
                        || self
                            .crawl_fields
                            .iter()
                            .take(7)
                            .enumerate()
                            .any(|(i, input)| {
                                let d = &p.project.discovery;
                                let expected = match i {
                                    0 => d.page_limit.to_string(),
                                    1 => d.section.clone().unwrap_or_default(),
                                    2 => d.skipped_text.join(", "),
                                    3 => d
                                        .advanced
                                        .max_depth
                                        .map(|v| v.to_string())
                                        .unwrap_or_default(),
                                    4 => d
                                        .advanced
                                        .concurrency
                                        .map(|v| v.to_string())
                                        .unwrap_or_default(),
                                    5 => d.advanced.per_origin.to_string(),
                                    _ => d.advanced.retries.to_string(),
                                };
                                input.read(cx).content.as_ref() != expected
                            })
                },
            )
    }
    fn exit_signature(&self, cx: &App) -> String {
        format!(
            "{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{:?}|{}",
            self.name.read(cx).content,
            self.address.read(cx).content,
            self.crawl_fields
                .iter()
                .map(|v| v.read(cx).content.to_string())
                .collect::<Vec<_>>(),
            self.draft_settings,
            self.timezone.read(cx).selected,
            self.login_ephemeral,
            self.mask_rectangles,
            self.analysis_fields[9].read(cx).content,
            self.selected_key,
            self.evidence.comparison.as_ref().map(|a| a.revision.id),
            self.review_exit_signature(cx)
        )
    }
    pub(super) fn request_exit(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.native_prompt || self.exit_guard.phase != Phase::Idle {
            return;
        }
        let draft_dirty = self.exit_project_dirty(cx)
            || self.mask_dirty
            || self.exit_network_dirty(cx)
            || self.review_unsaved(cx)
            || self.mask_geometry_dirty(cx)
            || self.exit_note_dirty(cx)
            || self.exit_builder_dirty(cx);
        let page_list_only = self.preview.is_some() && !draft_dirty;
        let dirty = draft_dirty || self.preview.is_some();
        // A submitted mutation cannot be rolled back with Cancel or Discard.
        // Do not destroy its reply receiver before we know its committed result.
        if self.pending.is_some() && !self.focus.capturing {
            self.notice = Some("Wait for the current operation to finish before quitting. Cancelling would not undo settings or screenshots already saved.".into());
            cx.notify();
            return;
        }
        if !self.exit_guard.request(dirty, false) {
            return;
        }
        // Runtime/discovery/capture work keeps its established stop-and-join
        // behavior after a clean or explicitly discarded exit. Saving edits
        // still requires waiting for that operation to finish first.
        if !dirty {
            self.finish_exit(cx);
            return;
        }
        let return_focus = window.focused(cx);
        self.exit_guard.return_focus = return_focus.clone();
        self.exit_guard.window = Some(window.window_handle());
        self.native_prompt = true;
        let answer = if page_list_only {
            window.prompt(
                PromptLevel::Warning,
                "Discard discovered pages and quit?",
                Some("The discovered page list and its include/exclude choices are unsaved and will be discarded when you quit. Saved settings, screenshots and baselines are kept. Keep the page list to continue reviewing."),
                &["Keep page list", "Discard pages and quit"],
                cx,
            )
        } else {
            window.prompt(
            PromptLevel::Warning,
            "Save your edits before quitting?",
            Some(if self.preview.is_some() {
                "Discovered pages and inclusion choices are unsaved and will be discarded when you quit, even with Save and quit. Project settings, ignored areas, review notes and network edits can be saved locally. No website is visited and no baseline is changed. Cancel to keep the page list."
            } else {
                "Project and website settings, ignored areas, review notes and this computer's network edits will be checked and saved locally. This does not visit websites, approve results or save a baseline. Discard only drops unsaved edits; previously saved work stays saved."
            }),
            &["Cancel", "Save and quit", "Discard and quit"],
            cx,
            )
        };
        cx.spawn_in(window, async move |this, cx| {
            let decision = match answer.await {
                Ok(1) if page_list_only => Decision::Discard,
                Ok(1) => Decision::Save,
                Ok(2) => Decision::Discard,
                _ => Decision::Cancel,
            };
            let _ = this.update_in(cx, |this, window, cx| {
                this.native_prompt = false;
                this.exit_guard.decide(decision);
                match decision {
                    Decision::Cancel => {
                        if let Some(handle) = return_focus {
                            window.focus(&handle);
                        }
                    }
                    Decision::Discard => {
                        this.discard_exit_review_drafts(cx);
                        this.finish_exit(cx);
                    }
                    Decision::Save => this.start_exit_save(cx),
                }
                cx.notify();
            });
        })
        .detach();
    }
    fn finish_exit(&mut self, cx: &mut Context<Self>) {
        if self.exit_guard.phase != Phase::Confirmed {
            return;
        }
        self.jobs.stop();
        cx.quit();
    }
    fn exit_save_failed(&mut self, message: impl Into<String>, cx: &mut Context<Self>) {
        let return_focus = self.exit_guard.return_focus.take();
        let window = self.exit_guard.window;
        let mask_saved = self.exit_guard.mask_saved;
        self.exit_guard.fail();
        self.native_prompt = false;
        if mask_saved && !self.mask_dirty {
            // Resume the usual mask-save analysis refresh if a later exit-save
            // step fails. Do not run it between note/settings save steps.
            self.schedule_analysis(cx);
        }
        self.error = Some(format!(
            "Could not save all edits: {} Your remaining edits are kept. Correct them and try Save and quit again, or discard them. Earlier successful saves are kept.",
            message.into()
        ));
        if self.presentation_focus.is_none()
            && let (Some(window), Some(focus)) = (window, return_focus)
        {
            cx.defer(move |cx| {
                let _ = window.update(cx, |_, window, _| window.focus(&focus));
            });
        }
        cx.notify();
    }
    fn start_exit_save(&mut self, cx: &mut Context<Self>) {
        if self.pending.is_some() {
            self.exit_save_failed("Another operation is still finishing.", cx);
            return;
        }
        if self.exit_builder_dirty(cx) {
            self.focus.screen = Screen::Setup;
            self.focus.setup_stage = SetupStage::Settings;
            self.exit_save_failed("Finish adding the screen size or website mapping in Settings first; these incomplete forms cannot be applied automatically.", cx);
            return;
        }
        if self.mask_geometry_dirty(cx) {
            self.presentation_focus = Some(Control::AnalysisField(10));
            self.exit_save_failed("Apply area edit first, then save ignored areas. Your numeric draft is preserved; it was not applied automatically.", cx);
            return;
        }
        if self
            .crawl_fields
            .iter()
            .chain(self.analysis_fields.iter())
            .chain([&self.name, &self.address])
            .any(|v| v.read(cx).composing())
        {
            self.exit_save_failed("Finish entering your text first.", cx);
            return;
        }
        let notes = match self.pending_review_notes(cx) {
            Ok(notes) => notes,
            Err(message) => {
                self.exit_save_failed(message, cx);
                return;
            }
        };
        if self.mask_dirty {
            self.exit_guard.remaining.push_back(SaveKind::Mask);
        }
        for note in notes {
            self.exit_guard.notes.push_back(note);
            self.exit_guard.remaining.push_back(SaveKind::Note);
        }
        if self.exit_project_dirty(cx) {
            self.exit_guard.remaining.push_back(SaveKind::Project);
        }
        if self.exit_network_dirty(cx) {
            self.exit_guard.remaining.push_back(SaveKind::Network);
        }
        self.next_exit_save(cx);
    }
    fn next_exit_save(&mut self, cx: &mut Context<Self>) {
        let Some(kind) = self.exit_guard.remaining.pop_front() else {
            if self.exit_project_dirty(cx)
                || self.mask_dirty
                || self.review_unsaved(cx)
                || self.mask_geometry_dirty(cx)
                || self.exit_note_dirty(cx)
                || self.exit_network_dirty(cx)
                || self.exit_builder_dirty(cx)
            {
                self.exit_save_failed(
                    "Some changes have not been saved. Save them before quitting.",
                    cx,
                );
                return;
            }
            self.exit_guard.phase = Phase::Confirmed;
            self.finish_exit(cx);
            return;
        };
        self.native_prompt = false;
        let result = (|| -> anyhow::Result<Option<Work>> {
            Ok(Some(match kind {
                SaveKind::Mask => {
                    let project = self
                        .project
                        .as_ref()
                        .ok_or_else(|| anyhow::anyhow!("Ignored areas have no saved project."))?;
                    let mut request = self
                        .viewer
                        .as_ref()
                        .and_then(|v| v.request.clone())
                        .ok_or_else(|| {
                            anyhow::anyhow!(
                                "Reopen the exact screenshot pair to save its ignored areas."
                            )
                        })?;
                    anyhow::ensure!(
                        !self.mask_geometry_dirty(cx),
                        "Apply area edit first. Your numeric draft was not changed."
                    );
                    anyhow::ensure!(
                        self.viewer_current(),
                        "Reopen the original screenshot pair before saving ignored areas."
                    );
                    validate_exit_pair(&request)?;
                    validate_exit_coverage(
                        &self.mask_rectangles,
                        request.before.dimensions,
                        request.after.dimensions,
                    )?;
                    request.policy = self.analysis_policy.clone();
                    request.mask = Some(regressee_app::analysis_engine::MaskBinding {
                        revision: regressee_core::domain::MaskId::new().to_string(),
                        target: request.before.target.clone(),
                        before_capture_id: request.before.capture_id.clone(),
                        after_capture_id: request.after.capture_id.clone(),
                        before_dimensions: request.before.dimensions,
                        after_dimensions: request.after.dimensions,
                        rectangles: self.mask_rectangles.clone(),
                    });
                    request.validate()?;
                    Work::Mask {
                        project: project.project.id,
                        request: Box::new(request),
                    }
                }
                SaveKind::Note => {
                    let mut note = self.exit_guard.notes.pop_front().ok_or_else(|| {
                        anyhow::anyhow!("The exact pending review-note snapshot is unavailable.")
                    })?;
                    let current = self
                        .pending_review_notes(cx)
                        .map_err(|e| anyhow::anyhow!(e))?;
                    anyhow::ensure!(
                        current.iter().any(|draft| draft.analysis == note.analysis
                            && draft.key == note.key
                            && draft.note == note.note
                            && draft.state == note.state),
                        "A review note or decision changed while saving. Review it before trying Save and quit again."
                    );
                    note.reviewed_at = regressee_runtime::distribution::now();
                    self.exit_guard.pending_note = Some(note.clone());
                    Work::Review(note)
                }
                SaveKind::Project => {
                    anyhow::ensure!(
                        self.environment.is_some()
                            || self.crawl_fields[10..13]
                                .iter()
                                .all(|v| v.read(cx).content.is_empty()),
                        "Load the website environment before saving its login edits."
                    );
                    anyhow::ensure!(
                        self.environment_draft.is_none() || self.environment.is_some(),
                        "Wait for the saved website environment to load before saving settings; unknown login settings must not be replaced."
                    );
                    self.exit_guard.login = if self.environment_draft.is_some() {
                        Some(self.local_login_settings(cx)?)
                    } else {
                        None
                    };
                    // Reuse the explicit settings-save path and its authoritative
                    // validation. It never activates a baseline or visits a site.
                    self.save_settings(cx);
                    return Ok(None);
                }
                SaveKind::Network => {
                    anyhow::ensure!(
                        self.network_status.is_some(),
                        "Wait for saved network settings to load before applying network edits."
                    );
                    self.crawl_fields[17].update(cx, |input, cx| {
                        input.touched = true;
                        cx.notify();
                    });
                    if let Some(error) = self.crawl_fields[17].read(cx).validation_error() {
                        anyhow::bail!("Check the proxy server: {error}");
                    }
                    let (proxy, ca) = self.exit_network_text(cx);
                    Work::SaveNetwork(regressee_runtime::network::NetworkSettings {
                        proxy: (!proxy.trim().is_empty()).then(|| proxy.trim().to_owned()),
                        trusted_ca: (!ca.trim().is_empty()).then(|| ca.trim().into()),
                    })
                }
            }))
        })();
        match result {
            Err(error) => {
                self.exit_save_failed(error.to_string(), cx);
                return;
            }
            Ok(Some(work)) => self.queue(work, cx),
            Ok(None) => (),
        }
        let Some(id) = self.pending else {
            let message = self.error.clone().unwrap_or_else(|| {
                "Some settings are invalid. Check the highlighted fields in Project settings."
                    .into()
            });
            if kind == SaveKind::Project {
                self.focus.screen = Screen::Setup;
                self.focus.setup_stage = SetupStage::Settings;
            }
            self.exit_save_failed(message, cx);
            return;
        };
        self.exit_guard.pending = Some((id, kind));
        self.exit_guard.signature = self.exit_signature(cx);
        self.native_prompt = true;
        self.notice = Some("Saving edits before quitting… The window stays open until every save is confirmed. Already committed saves cannot be undone by Cancel.".into());
        cx.notify();
    }
    pub(super) fn observe_exit_reply(&mut self, update: &Update) {
        if self.pending != Some(update.id) {
            return;
        }
        if let Reply::Network { settings, .. } = &update.reply {
            self.exit_network_saved = Some((
                settings.proxy.clone().unwrap_or_default(),
                settings
                    .trusted_ca
                    .as_ref()
                    .map(|p| p.to_string_lossy().into_owned())
                    .unwrap_or_default(),
            ));
        }
        if self.exit_guard.pending.is_none()
            && let Reply::SavedReview(history) = &update.reply
        {
            self.reconcile_exit_review_reply(history);
        }
    }
    pub(super) fn continue_exit_save(
        &mut self,
        update: Update,
        cx: &mut Context<Self>,
    ) -> Option<Update> {
        let Some((id, kind)) = self.exit_guard.pending else {
            return Some(update);
        };
        if update.id != id
            || matches!(
                &update.reply,
                Reply::Progress { .. } | Reply::CaptureStatus(_)
            )
        {
            return Some(update);
        }
        self.pending = None;
        self.task = None;
        self.exit_guard.pending = None;
        self.focus.busy = false;
        self.progress = None;
        let unchanged = self.exit_guard.signature == self.exit_signature(cx);
        let success = match (kind, update.reply) {
            (SaveKind::Mask, Reply::SavedMask) => {
                if unchanged {
                    self.mask_dirty = false;
                    self.exit_guard.mask_saved = true;
                }
                true
            }
            (SaveKind::Note, Reply::SavedReview(history)) => {
                let confirmed = self.exit_guard.pending_note.take().is_some_and(|note| {
                    history
                        .iter()
                        .find(|r| r.analysis == note.analysis && r.key == note.key)
                        .is_some_and(|r| r.note == note.note && r.state == note.state)
                });
                self.reconcile_exit_review_reply(&history);
                self.evidence.reviews = history;
                confirmed
            }
            (SaveKind::Project, Reply::Project(record)) => {
                // Record the committed revision even if a newer draft arrived.
                // Never reset that newer draft to the submitted values.
                if unchanged {
                    self.name
                        .update(cx, |v, cx| v.set_value(&record.project.name, cx));
                    self.address.update(cx, |v, cx| {
                        v.set_value(
                            &record
                                .project
                                .sources
                                .iter()
                                .map(|u| u.as_str())
                                .collect::<Vec<_>>()
                                .join(" "),
                            cx,
                        )
                    });
                    self.draft_settings = record.project.clone();
                    self.set_crawl_fields(cx);
                }
                if let Some(settings) = self.exit_guard.login.take()
                    && let Some(environment) = &mut self.environment
                {
                    // This is an authoritative committed snapshot, not an input
                    // reset; retain newer text against the new saved baseline.
                    environment.settings = settings;
                }
                self.project = Some(*record);
                self.focus.project_saved = true;
                self.projects_dirty = true;
                self.evidence_dirty = true;
                true
            }
            (SaveKind::Network, Reply::Network { settings, message }) => {
                if unchanged {
                    self.crawl_fields[17].update(cx, |v, cx| {
                        v.set_value(settings.proxy.as_deref().unwrap_or_default(), cx)
                    });
                    self.crawl_fields[18].update(cx, |v, cx| {
                        v.set_value(
                            &settings
                                .trusted_ca
                                .as_ref()
                                .map(|p| p.to_string_lossy().into_owned())
                                .unwrap_or_default(),
                            cx,
                        )
                    });
                }
                self.network_status = Some(message);
                self.network_read_signature = None;
                true
            }
            (_, Reply::Failure(error)) => {
                self.exit_save_failed(error.message(), cx);
                return None;
            }
            (_, Reply::ProductFailure(error)) => {
                self.exit_save_failed(error, cx);
                return None;
            }
            _ => false,
        };
        if !success || !unchanged {
            self.exit_save_failed(
                if success {
                    "New edits arrived while saving."
                } else {
                    "The save did not return the expected confirmation."
                },
                cx,
            );
        } else {
            self.next_exit_save(cx);
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn phase3_exit_rejects_multibyte_overflow_before_any_save_and_preserves_draft(
        cx: &mut gpui::TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            crate::workflow_refinement_tests::load_review(s, project, cx);
            for note in [
                "x".repeat(8193),
                "é".repeat(4097),
                format!("{}x", "🦀".repeat(2048)),
            ] {
                s.analysis_fields[9].update(cx, |i, cx| i.set_value(&note, cx));
                let requests = s.next_request;
                s.exit_guard.phase = Phase::Saving;
                s.start_exit_save(cx);
                assert_eq!(s.exit_guard.phase, Phase::Idle);
                assert_eq!(s.next_request, requests);
                assert!(s.pending.is_none());
                assert!(
                    s.error
                        .as_ref()
                        .unwrap()
                        .contains(regressee_core::domain::REVIEW_NOTE_LIMIT_MESSAGE)
                );
                assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), note);
            }
            let note = format!("{}\n", "é".repeat(4095)); // 8,191 bytes, exact exit snapshot
            s.analysis_fields[9].update(cx, |i, cx| i.set_value(&note, cx));
            s.exit_guard.phase = Phase::Saving;
            s.start_exit_save(cx);
            assert!(s.pending.is_some());
            assert_eq!(s.exit_guard.pending_note.as_ref().unwrap().note, note);
            // Existing reply/termination tests own Quit; this test settles the
            // real save as an ordinary reply, without terminating the test app.
            s.exit_guard = ExitGuard::default();
            crate::workflow_refinement_tests::finish_jobs(s, cx);
            assert!(!s.review_note_dirty(cx));
        });
        owned.join_after_ui();
    }
    #[test]
    fn clean_exit_and_pending_work() {
        let mut guard = ExitGuard::default();
        assert!(!guard.request(false, true));
        assert_eq!(guard.phase, Phase::Idle);
        assert!(guard.request(false, false));
        assert_eq!(guard.phase, Phase::Confirmed);
    }
    #[test]
    fn cancel_and_discard_never_start_saves() {
        for (decision, phase) in [
            (Decision::Cancel, Phase::Idle),
            (Decision::Discard, Phase::Confirmed),
        ] {
            let mut guard = ExitGuard::default();
            assert!(guard.request(true, false));
            assert!(!guard.request(true, false));
            guard.decide(decision);
            assert_eq!(guard.phase, phase);
            assert!(guard.pending.is_none());
        }
    }
    #[test]
    fn save_requires_confirmation_and_failure_preserves_open_state() {
        let mut guard = ExitGuard::default();
        guard.request(true, false);
        guard.decide(Decision::Save);
        assert!(guard.saving());
        guard.remaining.push_back(SaveKind::Mask);
        guard.pending = Some((41, SaveKind::Mask));
        assert!(!guard.request(false, false));
        guard.decide(Decision::Discard); // a second Quit cannot bypass saving
        assert!(guard.saving());
        guard.fail(); // invalid, failed or unconfirmed save: retry, not exit
        assert_eq!(guard.phase, Phase::Idle);
        assert!(guard.pending.is_none());
        assert!(guard.request(true, false));
    }
    #[test]
    fn exit_rejects_all_masked_union_without_changing_geometry() {
        use regressee_app::analysis_engine::{Dimensions, Rectangle};
        let d = Dimensions {
            width: 100,
            height: 80,
        };
        let areas = vec![
            Rectangle {
                x: 0,
                y: 0,
                width: 60,
                height: 80,
            },
            Rectangle {
                x: 40,
                y: 0,
                width: 60,
                height: 80,
            },
        ];
        let original = areas.clone();
        assert!(validate_exit_coverage(&areas, d, d).is_err());
        assert_eq!(areas, original);
        assert!(validate_exit_coverage(&areas[..1], d, d).is_ok());
    }
    #[test]
    fn exit_never_rebinds_a_stale_mask_implicitly() {
        use regressee_app::analysis_engine::{
            AnalysisRequest, CaptureDescriptor, Dimensions, MaskBinding, Target,
        };
        let before = CaptureDescriptor {
            capture_id: "before".into(),
            target: Target {
                page: "page".into(),
                profile: "profile".into(),
            },
            dimensions: Dimensions {
                width: 100,
                height: 80,
            },
            device_scale_factor: 1.,
            material_profile_id: "material".into(),
            rendering_environment_id: "rendering".into(),
            completed: true,
        };
        let request = AnalysisRequest {
            after: CaptureDescriptor {
                capture_id: "after".into(),
                ..before.clone()
            },
            policy: regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
            mask: Some(MaskBinding {
                revision: "saved-mask".into(),
                target: before.target.clone(),
                before_capture_id: "different-before".into(),
                after_capture_id: "after".into(),
                before_dimensions: before.dimensions,
                after_dimensions: before.dimensions,
                rectangles: vec![],
            }),
            before,
        };
        let original = request.clone();
        assert!(
            validate_exit_pair(&request)
                .unwrap_err()
                .to_string()
                .contains("Check that ignored areas fit these screenshots")
        );
        assert_eq!(request, original);
    }
}

#[cfg(all(test, feature = "ui-tests"))]
mod controller_tests {
    use super::*;
    use gpui::TestAppContext;

    // Controller-level, synthetic replies: these are not OS-close/input tests.
    #[gpui::test]
    fn pristine_and_invalid_project_drafts(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owner = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |shell, cx| {
            shell.pending = None;
            shell.focus.busy = false;
            assert!(
                !shell.exit_project_dirty(cx),
                "fresh UUIDs must not look like edits"
            );
            assert!(!shell.exit_network_dirty(cx));
            assert!(!shell.exit_builder_dirty(cx));
            shell
                .name
                .update(cx, |v, cx| v.set_value("Keep this draft", cx));
            shell
                .address
                .update(cx, |v, cx| v.set_value("not an address", cx));
            assert!(shell.exit_project_dirty(cx));
            shell.exit_guard.phase = Phase::Saving;
            shell.start_exit_save(cx);
            assert_eq!(shell.exit_guard.phase, Phase::Idle);
            assert!(shell.pending.is_none());
            assert_eq!(shell.address.read(cx).content.as_ref(), "not an address");
            assert!(
                shell
                    .error
                    .as_ref()
                    .unwrap()
                    .contains("Could not save all edits")
            );
        });
        owner.stop();
        owner.join_after_ui();
    }

    #[gpui::test]
    fn mask_waits_for_exact_success_and_retains_failed_draft(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owner = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |shell, cx| {
            shell.pending = Some(80);
            shell.exit_guard.phase = Phase::Saving;
            shell.exit_guard.pending = Some((80, SaveKind::Mask));
            shell.mask_dirty = true;
            shell
                .mask_rectangles
                .push(regressee_app::analysis_engine::Rectangle {
                    x: 1,
                    y: 1,
                    width: 8,
                    height: 8,
                });
            shell.exit_guard.signature = shell.exit_signature(cx);
            shell.apply_update(
                Update {
                    id: 79,
                    reply: Reply::SavedMask,
                },
                cx,
            );
            assert!(shell.mask_dirty);
            assert_eq!(shell.pending, Some(80));
            shell.apply_update(
                Update {
                    id: 80,
                    reply: Reply::Progress {
                        message: "Saving".into(),
                        completed: 0,
                        total: None,
                    },
                },
                cx,
            );
            assert!(shell.exit_guard.saving());
            assert!(shell.mask_dirty);
            shell.apply_update(
                Update {
                    id: 80,
                    reply: Reply::ProductFailure("Disk full".into()),
                },
                cx,
            );
            assert_eq!(shell.exit_guard.phase, Phase::Idle);
            assert!(shell.mask_dirty);
            assert_eq!(shell.mask_rectangles.len(), 1);
            assert!(shell.error.as_ref().unwrap().contains("Disk full"));

            // A confirmed mask save remains committed even if the next settings
            // step is invalid. Cancel is not described as a database rollback.
            shell.pending = Some(81);
            shell.exit_guard.phase = Phase::Saving;
            shell.exit_guard.pending = Some((81, SaveKind::Mask));
            shell.exit_guard.remaining.push_back(SaveKind::Project);
            shell.exit_guard.signature = shell.exit_signature(cx);
            shell.apply_update(
                Update {
                    id: 81,
                    reply: Reply::SavedMask,
                },
                cx,
            );
            assert!(!shell.mask_dirty);
            assert_eq!(shell.mask_rectangles.len(), 1);
            assert_eq!(shell.exit_guard.phase, Phase::Idle);
            assert!(
                shell
                    .error
                    .as_ref()
                    .unwrap()
                    .contains("Earlier successful saves are kept")
            );
        });
        owner.stop();
        owner.join_after_ui();
    }

    #[gpui::test]
    fn unconfirmed_save_and_newer_edits_cannot_exit(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owner = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |shell, cx| {
            shell.pending = Some(90);
            shell.exit_guard.phase = Phase::Saving;
            shell.exit_guard.pending = Some((90, SaveKind::Mask));
            shell.mask_dirty = true;
            shell.exit_guard.signature = shell.exit_signature(cx);
            shell.apply_update(
                Update {
                    id: 90,
                    reply: Reply::Runtime("Not a save reply".into()),
                },
                cx,
            );
            assert!(shell.mask_dirty);
            assert_eq!(shell.exit_guard.phase, Phase::Idle);
            shell.pending = Some(91);
            shell.exit_guard.phase = Phase::Saving;
            shell.exit_guard.pending = Some((91, SaveKind::Mask));
            shell.exit_guard.signature = shell.exit_signature(cx);
            shell
                .mask_rectangles
                .push(regressee_app::analysis_engine::Rectangle {
                    x: 2,
                    y: 2,
                    width: 4,
                    height: 4,
                });
            shell.apply_update(
                Update {
                    id: 91,
                    reply: Reply::SavedMask,
                },
                cx,
            );
            assert!(
                shell.mask_dirty,
                "a newer rectangle draft must not be cleared"
            );
            assert_eq!(shell.exit_guard.phase, Phase::Idle);
            assert!(shell.error.as_ref().unwrap().contains("New edits arrived"));
        });
        owner.stop();
        owner.join_after_ui();
    }

    #[gpui::test]
    fn two_target_notes_save_exact_bindings_and_preserve_disposition(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owner = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |shell, cx| {
            shell.pending = None;
            shell.focus.busy = false;
            let key = TargetKey {
                page: regressee_core::domain::PageId::new(),
                profile: regressee_core::domain::ProfileId::new(),
            };
            let analysis = regressee_core::domain::AnalysisId::new();
            let saved = regressee_core::domain::ReviewDecision {
                analysis,
                key: key.clone(),
                state: regressee_core::domain::ReviewState::NeedsFix,
                note: "Existing note".into(),
                reviewed_at: 0,
            };
            let other = TargetKey {
                page: regressee_core::domain::PageId::new(),
                profile: key.profile,
            };
            let second = regressee_core::domain::ReviewDecision {
                key: other.clone(),
                state: regressee_core::domain::ReviewState::Intentional,
                note: "Other existing note".into(),
                ..saved.clone()
            };
            shell.evidence.reviews = vec![saved.clone(), second.clone()];
            shell.review_drafts = review_ux::ReviewDrafts::exit_fixture(
                Some((analysis, other.clone())),
                vec![((analysis, key.clone()), "First draft".into())],
            );
            shell.analysis_fields[9].update(cx, |v, cx| v.set_value("Second draft", cx));
            // Selection has changed, but sync has not run: mounted text remains
            // attached to the model's active binding, never selected_key.
            shell.selected_key = Some(key.clone());
            let notes = shell.pending_review_notes(cx).unwrap();
            assert_eq!(notes.len(), 2);
            assert!(
                notes
                    .iter()
                    .any(|n| n.key == key && n.note == "First draft" && n.state == saved.state)
            );
            assert!(
                notes
                    .iter()
                    .any(|n| n.key == other && n.note == "Second draft" && n.state == second.state)
            );
            assert!(shell.exit_note_dirty(cx));
            shell.name.update(cx, |v, cx| {
                v.set_value("Invalid project keeps test window open", cx)
            });
            shell.exit_guard.phase = Phase::Saving;
            shell.start_exit_save(cx);
            let first = shell.exit_guard.pending_note.clone().unwrap();
            let first_id = shell.pending.unwrap();
            let mut history = vec![saved, second];
            history.retain(|n| n.key != first.key);
            history.insert(0, first.clone());
            shell.apply_update(
                Update {
                    id: first_id,
                    reply: Reply::SavedReview(history.clone()),
                },
                cx,
            );
            assert!(
                shell.exit_guard.saving(),
                "one note reply cannot finish two notes"
            );
            let second = shell.exit_guard.pending_note.clone().unwrap();
            assert_ne!(first.key, second.key);
            history.retain(|n| n.key != second.key);
            history.insert(0, second);
            shell.apply_update(
                Update {
                    id: shell.pending.unwrap(),
                    reply: Reply::SavedReview(history),
                },
                cx,
            );
            assert_eq!(shell.exit_guard.phase, Phase::Idle); // subsequent project validation failed
            assert!(!shell.exit_note_dirty(cx));
            shell.analysis_fields[9].update(cx, |v, cx| v.set_value("", cx));
            assert!(
                shell.exit_note_dirty(cx),
                "clearing a saved note is an edit"
            );
            let clearing = shell.pending_review_notes(cx).unwrap();
            assert_eq!(clearing.len(), 1);
            assert_eq!(clearing[0].key, other);
            assert_eq!(clearing[0].note, "");
            assert_eq!(
                clearing[0].state,
                regressee_core::domain::ReviewState::Intentional
            );
            shell.analysis_fields[9].update(cx, |v, cx| v.set_value(&"x".repeat(8193), cx));
            assert!(shell.pending_review_notes(cx).is_err());
            assert_eq!(
                shell.analysis_fields[9].read(cx).content.chars().count(),
                8193
            );

            shell.exit_network_saved = Some((String::new(), String::new()));
            shell.network_status = Some("Loaded".into());
            shell.crawl_fields[17].update(cx, |v, cx| v.set_value("ftp://invalid.test", cx));
            assert!(shell.exit_network_dirty(cx));
            shell.exit_guard.phase = Phase::Saving;
            shell.exit_guard.remaining.push_back(SaveKind::Network);
            shell.next_exit_save(cx);
            assert_eq!(shell.exit_guard.phase, Phase::Idle);
            assert!(shell.pending.is_none());
            assert_eq!(
                shell.crawl_fields[17].read(cx).content.as_ref(),
                "ftp://invalid.test"
            );
            assert!(shell.error.as_ref().unwrap().contains("proxy server"));
        });
        owner.stop();
        owner.join_after_ui();
    }

    #[gpui::test]
    fn unapplied_geometry_and_newer_retained_notes_stay_open(cx: &mut TestAppContext) {
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owner = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |shell, cx| {
            shell.pending = None;
            shell.focus.busy = false;
            shell
                .mask_rectangles
                .push(regressee_app::analysis_engine::Rectangle {
                    x: 1,
                    y: 2,
                    width: 8,
                    height: 9,
                });
            shell.mask_selected = Some(0);
            shell.load_mask_fields(cx);
            shell.analysis_fields[10].update(cx, |v, cx| v.set_value("-1", cx));
            let before = shell.mask_rectangles.clone();
            shell.exit_guard.phase = Phase::Saving;
            shell.start_exit_save(cx);
            assert_eq!(shell.exit_guard.phase, Phase::Idle);
            assert!(shell.pending.is_none());
            assert!(
                shell
                    .error
                    .as_ref()
                    .unwrap()
                    .contains("Apply area edit first")
            );
            assert_eq!(shell.mask_rectangles, before);
            assert_eq!(shell.analysis_fields[10].read(cx).content.as_ref(), "-1");
            shell.load_mask_fields(cx);

            let identity = (
                regressee_core::domain::AnalysisId::new(),
                TargetKey {
                    page: regressee_core::domain::PageId::new(),
                    profile: regressee_core::domain::ProfileId::new(),
                },
            );
            shell.review_drafts = review_ux::ReviewDrafts::exit_fixture(
                None,
                vec![(identity.clone(), "Submitted retained note".into())],
            );
            let submitted = shell.pending_review_notes(cx).unwrap().remove(0);
            shell.pending = Some(120);
            shell.exit_guard.phase = Phase::Saving;
            shell.exit_guard.pending = Some((120, SaveKind::Note));
            shell.exit_guard.pending_note = Some(submitted.clone());
            shell.exit_guard.signature = shell.exit_signature(cx);
            shell.review_drafts = review_ux::ReviewDrafts::exit_fixture(
                None,
                vec![(identity.clone(), "Newer retained note".into())],
            );
            shell.analysis_fields[12].update(cx, |v, cx| v.set_value("invalid width", cx));
            shell.apply_update(
                Update {
                    id: 120,
                    reply: Reply::SavedReview(vec![submitted]),
                },
                cx,
            );
            assert_eq!(shell.exit_guard.phase, Phase::Idle);
            assert!(shell.error.as_ref().unwrap().contains("New edits arrived"));
            let pending = shell.pending_review_notes(cx).unwrap();
            assert_eq!(pending.len(), 1);
            assert_eq!(pending[0].note, "Newer retained note");
            assert_eq!(pending[0].analysis, identity.0);
            assert_eq!(pending[0].key, identity.1);
            assert_eq!(
                shell.analysis_fields[12].read(cx).content.as_ref(),
                "invalid width"
            );
            let signature = shell.review_exit_signature(cx);
            let saved_reviews = shell.evidence.reviews.clone();
            shell.exit_guard.request(true, false);
            shell.exit_guard.decide(Decision::Cancel);
            assert_eq!(
                shell.review_exit_signature(cx),
                signature,
                "Cancel keeps retained notes and numeric drafts"
            );
            shell.exit_guard.request(true, false);
            shell.exit_guard.decide(Decision::Discard);
            shell.discard_exit_review_drafts(cx);
            assert!(shell.pending_review_notes(cx).unwrap().is_empty());
            assert!(!shell.mask_geometry_dirty(cx));
            assert_eq!(
                shell.evidence.reviews, saved_reviews,
                "Discard never erases committed dispositions"
            );
        });
        owner.stop();
        owner.join_after_ui();
    }
}
