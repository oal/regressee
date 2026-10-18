// SPDX-License-Identifier: MPL-2.0
//! External saved-data refresh preserves the human's semantic location.
use super::*;
use regressee_app::control::{Selection, View};

impl Shell {
    pub(super) fn project_input_composing(&self, cx: &App) -> bool {
        self.name.read(cx).composing()
            || self.address.read(cx).composing()
            || self.timezone.read(cx).is_composing(cx)
            || self.crawl_fields.iter().any(|i| i.read(cx).composing())
    }
    pub(super) fn analysis_policy_draft(&self, cx: &App) -> bool {
        !self.analysis_signature.is_empty()
            && self
                .analysis_fields
                .iter()
                .take(9)
                .map(|f| f.read(cx).content.trim().to_owned())
                .collect::<Vec<_>>()
                != self.analysis_signature
    }
    fn passive_draft(&self, cx: &App) -> bool {
        self.mask_dirty
            || self.mask_drag.is_some()
            || self.mask_geometry_dirty(cx)
            || self.review_unsaved(cx)
            || self
                .pending_review_notes(cx)
                .map_or(true, |notes| !notes.is_empty())
            || self.analysis_pending.is_some()
            || self.analysis_due.is_some()
            || self.analysis_fields.iter().any(|f| f.read(cx).composing())
            || self.analysis_policy_draft(cx)
    }
    pub(super) fn refresh_selected_evidence(&mut self, cx: &mut Context<Self>) -> bool {
        if !self.jobs.is_remote() || !self.external_evidence_refresh {
            return false;
        }
        let Some(project) = self.project.as_ref().map(|p| p.project.id) else {
            return false;
        };
        let Some(run) = self.evidence.latest.as_ref().map(|r| r.run.id) else {
            return false;
        };
        if self.passive_evidence_pending.is_some() {
            return true;
        }
        if self.passive_draft(cx) {
            if self.notice.is_none() {
                self.notice=Some("Saved results changed in another client. Your selection and drafts are kept; save or deliberately discard your edits before refreshing them.".into());
                cx.notify();
            }
            return true;
        }
        let selection = Selection {
            project,
            run: Some(run),
            kind: if self.dom_active() {
                regressee_core::domain::CheckKind::DomSnapshot
            } else {
                regressee_core::domain::CheckKind::Screenshot
            },
            analysis: if self.dom_active() {
                self.evidence
                    .dom_comparison
                    .as_ref()
                    .map(|a| regressee_core::domain::AnalysisId(a.revision.id.0))
            } else {
                self.evidence.comparison.as_ref().map(|a| a.revision.id)
            },
            key: None,
            view: View::Compare,
        };
        self.next_request += 1;
        let id = self.next_request;
        if self.jobs.refresh_evidence(id, selection.clone()).is_ok() {
            self.passive_evidence_pending = Some((id, selection, self.data_refresh_generation));
        }
        true
    }
    pub(super) fn apply_passive_evidence(
        &mut self,
        update: &Update,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self
            .passive_evidence_pending
            .as_ref()
            .is_some_and(|(id, _, _)| *id == update.id)
        {
            return false;
        }
        let (_, selection, generation) = self.passive_evidence_pending.take().unwrap();
        let read_failures = self.jobs.take_evidence_failures(update.id);
        let current = self
            .project
            .as_ref()
            .is_some_and(|p| p.project.id == selection.project)
            && self.evidence.latest.as_ref().map(|r| r.run.id) == selection.run
            && (if selection.kind == regressee_core::domain::CheckKind::DomSnapshot {
                self.evidence
                    .dom_comparison
                    .as_ref()
                    .map(|a| regressee_core::domain::AnalysisId(a.revision.id.0))
            } else {
                self.evidence.comparison.as_ref().map(|a| a.revision.id)
            }) == selection.analysis;
        if !current
            || generation != self.data_refresh_generation
            || self.agent_locked()
            || self.pending.is_some()
            || self.task.is_some()
            || self.passive_draft(cx)
        {
            return true;
        }
        let Reply::Evidence(evidence) = &update.reply else {
            self.evidence_dirty = false;
            self.viewer_pending = None;
            self.focus.view_loading = false;
            self.viewer_read_failed = true;
            self.clear_viewer();
            self.error=Some("Could not refresh the selected saved result. Your selection and drafts are kept. Refresh or choose another saved result; no newer run was substituted.".into());
            cx.notify();
            return true;
        };
        if evidence.latest.as_ref().map(|r| r.run.id) != selection.run
            || (selection.analysis.is_some()
                && !read_failures.as_ref().is_some_and(|failures| {
                    failures.project == selection.project
                        && Some(failures.run) == selection.run
                        && failures.by_kind.contains_key(&selection.kind)
                })
                && (if selection.kind == regressee_core::domain::CheckKind::DomSnapshot {
                    evidence
                        .dom_comparison
                        .as_ref()
                        .map(|a| regressee_core::domain::AnalysisId(a.revision.id.0))
                } else {
                    evidence.comparison.as_ref().map(|a| a.revision.id)
                }) != selection.analysis)
        {
            return true;
        }
        let target = |e: &ProjectEvidence| {
            e.latest
                .as_ref()
                .and_then(|r| {
                    r.run
                        .targets
                        .iter()
                        .find(|t| Some(&t.key) == self.selected_key.as_ref())
                })
                .and_then(|t| serde_json::to_vec(t).ok())
        };
        let analysis_changed = self.evidence.comparison.as_ref().map(|a| a.revision.id)
            != evidence.comparison.as_ref().map(|a| a.revision.id);
        let pixels_changed = target(&self.evidence) != target(evidence) || analysis_changed;
        let before = self.evidence.before.clone();
        let baselines = self.history.as_ref().map(|h| h.baselines.clone());
        self.evidence = (**evidence).clone();
        self.dom.read_failures = read_failures;
        // History owns a separately paged list, not the exact run read's list.
        if let Some(baselines) = baselines {
            self.evidence.baselines = baselines;
        }
        // `before` is target-focused by the pixel reader, not the run's first row.
        self.evidence.before = before;
        if analysis_changed
            && let Some(policy) = self
                .evidence
                .comparison
                .as_ref()
                .and_then(|a| a.perceptual.as_ref())
                .map(|p| p.policy.clone())
        {
            self.analysis_policy = policy;
            self.set_analysis_fields(cx);
        }
        self.evidence_dirty = false;
        self.evidence_read_failed = false;
        *self.local_index.borrow_mut() = Default::default();
        self.invalidate_approval();
        self.focus.candidate = self.evidence.latest.as_ref().is_some_and(|r| {
            r.baseline_save_counts().is_some() && !self.capture_is_saved_baseline()
        });
        self.focus.baseline = self.evidence.baseline.is_some();
        self.focus.resumable = self
            .evidence
            .latest
            .as_ref()
            .is_some_and(|r| matches!(r.run.state, JobState::Paused | JobState::Interrupted));
        if pixels_changed {
            self.clear_viewer();
            self.reload_viewer(cx);
        }
        // The draft/IME guards above established that every cached note was
        // clean. Rehydrate saved note changes even when its identity is unchanged,
        // rather than turning the old saved text into a synthetic local draft.
        self.review_drafts = Default::default();
        self.dom.active_note = None;
        self.dom.notes.clear();
        self.dom.identity = None;
        self.sync_review_note(cx);
        cx.notify();
        true
    }

    pub(super) fn refresh_selected_history(&mut self) {
        if !self.jobs.is_remote() || !self.history_dirty || self.passive_history_pending.is_some() {
            return;
        }
        let (Some(project), Some(history)) = (&self.project, &self.history) else {
            return;
        };
        let project = project.project.id;
        let page = history.page;
        self.next_request += 1;
        let id = self.next_request;
        if self.jobs.refresh_history(id, project, page).is_ok() {
            self.passive_history_pending = Some((id, project, page, self.data_refresh_generation));
        }
    }

    pub(super) fn apply_passive_history(
        &mut self,
        update: &Update,
        cx: &mut Context<Self>,
    ) -> bool {
        if !self
            .passive_history_pending
            .is_some_and(|(id, _, _, _)| id == update.id)
        {
            return false;
        }
        let (_, project, page, generation) = self.passive_history_pending.take().unwrap();
        if self
            .project
            .as_ref()
            .is_none_or(|p| p.project.id != project)
            || self.history.as_ref().is_none_or(|h| h.page != page)
            || generation != self.data_refresh_generation
            || self.agent_locked()
            || self.pending.is_some()
            || self.task.is_some()
        {
            return true;
        }
        if let Reply::History { history } = &update.reply {
            if history.page != page {
                return true;
            }
            self.receive_history((**history).clone(), true);
        } else {
            self.error = Some("Could not refresh this History page. Your page and selection are kept; use Refresh to try again.".into());
        }
        self.history_dirty = false;
        cx.notify();
        true
    }

    pub(super) fn receive_history(
        &mut self,
        history: regressee_storage::HistoryPage,
        passive: bool,
    ) {
        let baseline = self
            .evidence
            .baselines
            .get(self.history_index)
            .map(|b| b.id);
        let run = self
            .history
            .as_ref()
            .and_then(|h| h.runs.get(self.run_history_index))
            .map(|r| r.run.id);
        // A removed/off-page row is no selection, never an implicit jump to row 0.
        // Deliberate page navigation retains the normal first-row default.
        let missing = if passive { usize::MAX } else { 0 };
        self.history_index = baseline
            .and_then(|id| history.baselines.iter().position(|b| b.id == id))
            .unwrap_or(missing);
        self.run_history_index = run
            .and_then(|id| history.runs.iter().position(|r| r.run.id == id))
            .unwrap_or(missing);
        let next_baseline = history.baselines.get(self.history_index).map(|b| b.id);
        let next_run = history.runs.get(self.run_history_index).map(|r| r.run.id);
        self.evidence.baselines = history.baselines.clone();
        self.focus.history_baselines = history.baselines.len();
        self.focus.history_runs = history.runs.len();
        self.history = Some(history);
        *self.history_filter_cache.borrow_mut() = Default::default();
        if baseline != next_baseline || run != next_run {
            self.clear_portability_previews();
        }
    }
}
