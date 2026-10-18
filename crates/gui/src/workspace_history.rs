// SPDX-License-Identifier: MPL-2.0
//! Project-scoped baseline, run history and deliberate storage cleanup.
use super::*;
use regressee_core::{domain::BaselineVersion, evidence::RunRecord};

impl Shell {
    fn history_pagination(&self, cx: &mut Context<Self>) -> Div {
        div()
            .debug_selector(|| "history-pagination".into())
            .flex()
            .items_center()
            .gap_2()
            .child(self.tool_control(
                Control::Phase6(12),
                ToolIcon::Previous,
                None,
                "Previous history page",
                false,
                cx,
            ))
            .child(
                div()
                    .text_size(px(HELP_TEXT_SIZE))
                    .line_height(px(HELP_LINE_HEIGHT))
                    .text_color(rgb(palette().muted))
                    .child(format!(
                        "Page {}",
                        self.history.as_ref().map_or(1, |h| h.page + 1)
                    )),
            )
            .child(self.tool_control(
                Control::Phase6(11),
                ToolIcon::Next,
                None,
                "Next history page",
                false,
                cx,
            ))
    }

    pub(super) fn history_action_bounds(
        &self,
        control: Control,
    ) -> Option<std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>> {
        if self.focus.screen != Screen::History {
            return None;
        }
        let index = match control {
            Control::Phase6(2) => 0,
            Control::Phase6(4) => 1,
            Control::Phase6(5) => 2,
            Control::Phase6(7) => 3,
            Control::Phase6(11) => 4,
            Control::Phase6(12) => 5,
            Control::Phase6(19) => 6,
            Control::Phase5(42) => 7,
            Control::Details(Detail::History) => 8,
            Control::Details(Detail::Portability) => 9,
            Control::Phase6(index @ 16..=18) => 10 + index - 16,
            Control::HistoryRefresh => 13,
            Control::HistoryClear => 14,
            Control::HistoryKind(index @ 0..=3) => 15 + index,
            Control::Phase6(3) => 19,
            Control::Phase6(6) => 20,
            Control::Phase6(0) => 21,
            Control::Phase5(40) => 22,
            Control::Phase5(41) => 23,
            Control::Phase6(9) => 24,
            Control::Phase6(10) => 25,
            Control::Phase6(8) => 26,
            Control::Phase6(1) => 27,
            _ => return None,
        };
        self.history_action_bounds.get(index).cloned()
    }
    pub(super) fn open_history_run(&mut self, cx: &mut Context<Self>) {
        if self.pending.is_some() {
            return;
        }
        // Refuse replacement rather than silently discarding either kind of draft.
        if self.unsaved(cx) || self.mask_dirty || self.mask_drag.is_some() {
            self.error = Some(
                "Save or discard your draft before opening saved screenshots. Nothing was changed."
                    .into(),
            );
            return;
        }
        if let (Some(project), Some(record)) = (
            &self.project,
            self.history
                .as_ref()
                .and_then(|h| h.runs.get(self.run_history_index)),
        ) {
            self.history_open_epoch = Some(self.history_selection_epoch);
            self.queue(
                Work::InspectRun {
                    project: project.project.id,
                    run: record.run.id,
                },
                cx,
            );
        }
    }

    pub(super) fn receive_history_inspection(
        &mut self,
        project: regressee_core::domain::ProjectId,
        run: regressee_core::domain::RunId,
        mut evidence: ProjectEvidence,
        cx: &mut Context<Self>,
    ) {
        if self.history_open_epoch.take() != Some(self.history_selection_epoch)
            || self.focus.screen != Screen::History
            || self.unsaved(cx)
            || self.mask_dirty
            || self.mask_drag.is_some()
            || !self.history_run_indices().contains(&self.run_history_index)
            || self
                .project
                .as_ref()
                .is_none_or(|p| p.project.id != project)
            || self
                .history
                .as_ref()
                .and_then(|h| h.runs.get(self.run_history_index))
                .is_none_or(|r| r.run.id != run)
            || evidence
                .latest
                .as_ref()
                .is_none_or(|r| r.run.id != run || r.run.project != project)
        {
            return;
        }
        self.invalidate_approval();
        if let Some((_, cancel)) = self.analysis_pending.take() {
            cancel.store(true, std::sync::atomic::Ordering::Release);
        }
        self.analysis_due = None;
        self.viewer_pending = None;
        self.clear_viewer();
        self.approval_selection.clear();
        self.mask_rectangles.clear();
        self.mask_selected = None;
        self.mask_draw = false;
        self.mask_drag = None;
        self.preview = None;
        self.focus.preview = false;
        self.discovery_preview_visible = false;
        self.review_tools = false;
        self.analysis_advanced = false;
        self.review_note_open = false;
        evidence.baselines = self.evidence.baselines.clone();
        self.result_filter = usize::from(
            local_ux::result_filter_scope(&evidence).is_some_and(|(_, compared)| compared),
        );
        self.selected_key = evidence
            .latest
            .as_ref()
            .and_then(|r| r.run.targets.first())
            .map(|t| t.key.clone());
        self.evidence = evidence;
        *self.local_index.borrow_mut() = Default::default();
        if let Some(index) = self.result_indices().first().copied() {
            self.selected_key = self
                .evidence
                .latest
                .as_ref()
                .map(|r| r.run.targets[index].key.clone());
        }
        if let Some(policy) = self
            .evidence
            .comparison
            .as_ref()
            .and_then(|a| a.perceptual.as_ref())
            .map(|p| p.policy.clone())
        {
            self.analysis_policy = policy;
            self.set_analysis_fields(cx);
        }
        self.discard_review_drafts(cx);
        self.load_mask_fields(cx);
        self.history_inspection = Some((project, run));
        *self.local_index.borrow_mut() = Default::default();
        self.focus.candidate = false;
        self.focus.resumable = false;
        self.focus.baseline = self.evidence.baseline.is_some();
        self.focus.viewer = self
            .evidence
            .latest
            .as_ref()
            .is_some_and(|r| r.all_captures().next().is_some() || !r.dom_captures.is_empty());
        self.focus.review_surface = if self.has_kind_comparison() {
            ReviewSurface::Changes
        } else {
            ReviewSurface::Inspect
        };
        self.focus.screen = Screen::Results;
        self.evidence_dirty = false;
        self.pan_x = 0.;
        self.pan_y = 0;
        // The persistent recorded-evidence heading/return action owns this
        // context; do not add a second banner that consumes inspection height.
        self.notice = None;
        self.sync_dom_workspace(cx);
        self.reload_viewer(cx);
    }
    pub(super) fn history_active_baseline(&self) -> Option<regressee_core::domain::BaselineId> {
        if let Some(history) = &self.history {
            return history.active_baseline;
        }
        if let Some(project) = &self.project {
            return project.project.active_baseline;
        }
        self.evidence.baseline.as_ref().map(|b| b.id)
    }

    /// Filter only evidence already in memory; no storage or website work.
    pub(super) fn history_baseline_indices(&self) -> Vec<usize> {
        let active = self.history_active_baseline();
        self.evidence
            .baselines
            .iter()
            .enumerate()
            .filter(|(_, baseline)| {
                self.history_query.is_empty()
                    || history_matches(
                        &self.history_query,
                        &history_baseline_search(baseline, active == Some(baseline.id)),
                    )
            })
            .map(|(index, _)| index)
            .collect()
    }

    pub(super) fn history_run_indices(&self) -> Vec<usize> {
        self.history
            .as_ref()
            .map(|history| {
                history
                    .runs
                    .iter()
                    .enumerate()
                    .filter(|(_, record)| {
                        history_kind_matches(self.history_kind, &record.run.intent)
                            && (self.history_query.is_empty()
                                || history_matches(
                                    &self.history_query,
                                    &history_run_search(
                                        record,
                                        history.imported_runs.contains(&record.run.id),
                                    ),
                                ))
                    })
                    .map(|(index, _)| index)
                    .collect()
            })
            .unwrap_or_default()
    }

    pub(super) fn baseline_history(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div()
            .w_full()
            .max_w(px(1000.))
            .flex()
            .flex_col()
            .gap_2()
            .text_size(px(15.))
            .line_height(px(BODY_LINE_HEIGHT))
            .child(self.heading(
                "History",
                "Saved captures, checks and baseline versions. Nothing is deleted automatically.",
            ));
        if self.project.is_none() {
            return body
                .child(controls::empty(
                    "Choose a project to see its history",
                    "Open a saved project from Projects. Its baseline versions, capture runs and storage will appear here.",
                ))
                .into_any_element();
        }

        let baseline_indices = self.history_baseline_indices();
        let run_indices = self.history_run_indices();
        let selected_baseline = self.evidence.baselines.get(self.history_index).filter(|_| {
            self.history_section == 1 && baseline_indices.contains(&self.history_index)
        });
        let selected_run = self
            .history
            .as_ref()
            .and_then(|h| h.runs.get(self.run_history_index))
            .filter(|_| self.history_section == 0 && run_indices.contains(&self.run_history_index));
        body = body.child(
            div()
                .debug_selector(|| "history-section-navigation".into())
                .w_full()
                .flex()
                .flex_wrap()
                .items_center()
                .gap_1()
                .border_b_1()
                .border_color(rgb(palette().line))
                .children(
                    ["Runs", "Baselines", "Storage"]
                        .into_iter()
                        .enumerate()
                        .map(|(section, label)| {
                            self.action_control(
                                Control::Phase6(16 + section),
                                label,
                                ActionStyle::Tab {
                                    selected: self.history_section == section,
                                },
                                cx,
                            )
                        }),
                )
                .when(self.history_section != 2, |el| {
                    el.child(div().flex_1()).child(self.tool_control(
                        Control::HistoryRefresh,
                        ToolIcon::Refresh,
                        None,
                        "Refresh saved history",
                        false,
                        cx,
                    ))
                }),
        );
        if self.history_section != 2 && self.history.is_none() {
            return body.child(if self.error.is_some() || self.pending.is_none() {
                controls::empty("History could not load", "Use Refresh to try reading saved history again. Saved history has not been changed.")
            } else {
                controls::loading("Loading saved history…", "Reading capture runs and baseline versions from disk.")
            }).into_any_element();
        }
        if self.history_section != 2 {
            body = body.child(
                div()
                    .debug_selector(|| "history-search-controls".into())
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(controls::search_field(
                        self.history_search.clone().into_any_element(),
                        (!self.history_query.is_empty()).then(|| {
                            self.tool_control(
                                Control::HistoryClear,
                                ToolIcon::Close,
                                None,
                                "Clear history search",
                                false,
                                cx,
                            )
                            .into_any_element()
                        }),
                    ))
                    .child(history_help(
                        "Loaded page only · date, type, state, environment or ID.",
                    )),
            );
        }

        let mut baselines = div()
            .debug_selector(|| "history-baselines-section".into())
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .child(history_help(format!(
                "{} of {} baseline versions · loaded page {}{}",
                baseline_indices.len(),
                self.evidence.baselines.len(),
                self.history.as_ref().map(|h| h.page + 1).unwrap_or(1),
                if self.history.as_ref().is_some_and(|h| h.more) {
                    " · more history available"
                } else {
                    ""
                },
            )));
        if !baseline_indices.is_empty() {
            baselines = baselines.child(
                controls::page_list(scrollbar::vertical(
                    "history-baseline-scrollbar",
                    &self.history_baseline_scroll,
                    div()
                        .id("history-baseline-list")
                        .track_scroll(&self.history_baseline_scroll)
                        .w_full()
                        .debug_selector(|| "history-baseline-list".into())
                        .h(px(history_list_height(baseline_indices.len())))
                        .flex_shrink_0()
                        .overflow_y_scroll()
                        .flex()
                        .flex_col()
                        .children(baseline_indices.iter().map(|&index| {
                            let b = &self.evidence.baselines[index];
                            let active = self.history_active_baseline() == Some(b.id);
                            self.history_choice_row(
                                Control::HistoryBaseline(index),
                                format!(
                                    "{} · {}",
                                    history_date(b.created_at),
                                    if active {
                                        "Current baseline"
                                    } else {
                                        "Saved version"
                                    }
                                ),
                                format!(
                                    "{}{}",
                                    baseline_kind_summary(b),
                                    if b.parent.is_some() {
                                        " · Composite"
                                    } else {
                                        ""
                                    }
                                ),
                                index == self.history_index,
                                cx,
                            )
                        })),
                ))
                .flex_shrink_0(),
            );
        } else if !self.evidence.baselines.is_empty() {
            baselines = baselines.child(history_help(
                "No baseline versions on this history page match. Change or clear the search. Your saved screenshots are unchanged.",
            ));
        }
        if baseline_indices.len() > 4 {
            baselines = baselines.child(history_help("Scroll or Tab through loaded versions."));
        }
        if let Some(b) = selected_baseline {
            let active = self.history_active_baseline() == Some(b.id);
            baselines = baselines
                .child(
                    div()
                        .w_full()
                        .debug_selector(|| "history-selected-baseline".into())
                        .border_t_1()
                        .border_b_1()
                        .border_color(rgb(palette().line))
                        .py_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!(
                                    "Baseline saved {} — {}",
                                    history_date(b.created_at),
                                     if active { "Current baseline" } else { "Saved version" },
                                )),
                        )
                        .child(baseline_kind_summary(b))
                        .when_some(history_capture_span(b), |el, (first, last)| el.child(history_help(
                            if first == last {
                                format!("References captured {}", history_date(first))
                            } else {
                                 format!("References captured between {} and {}. Show technical details for each source's capture time.", history_date(first), history_date(last))
                            },
                        )))
                        .child(div().flex().flex_wrap().gap_2()
                            .when(!active, |el| el.child(self.button(Control::Phase5(42), "Use this baseline…", ActionStyle::Primary, cx)))
                            .child(self.button(Control::Phase6(2), "Review ZIP export…", ActionStyle::Secondary, cx))
                            .child(self.button(Control::Phase6(4), "Delete baseline…", ActionStyle::Danger, cx)))
                        .child(history_help(if active {
                            "Future checks use this version's fixed page list and screen sizes."
                        } else {
                             "Use this version as the baseline for future checks. This does not recapture the website or delete newer versions."
                        })),
                )
                .when(b.parent.is_some(), |el| el.child(
                    controls::status(
                        "Selected check references were updated; other references were kept from the previous baseline. They were not all captured at the same time.",
                        Tone::Warning,
                    )
                    .into_any_element()
                ));
        } else if self.evidence.baselines.is_empty() {
            baselines = baselines.child(if self.history.as_ref().is_some_and(|h| h.page > 0 || h.active_baseline.is_some()) {
                controls::empty(
                    "No baseline versions on this history page",
                    "Load another history page to find saved versions. Your current baseline is unchanged.",
                )
            } else { controls::empty(
                "No saved baseline versions",
                "Open Pages & baseline to review pages, capture screenshots, then choose Save baseline. You can also import a baseline ZIP as a new project.",
            ) });
        } else if self.evidence.baselines.get(self.history_index).is_some() {
            baselines = baselines.child(history_help(
                "The selected baseline is outside the search. Clear the search or choose a matching version to see its details and actions.",
            ));
        } else {
            baselines = baselines.child(history_help(
                "Choose a baseline version above to inspect it.",
            ));
        }
        baselines = baselines
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(self.button(
                        Control::Phase6(0),
                        "Inspect ZIP for import…",
                        ActionStyle::Secondary,
                        cx,
                    )),
            )
            .child(history_help(
                "ZIPs contain baseline captures and project settings, but no login sessions or cookies. Screenshots and DOM text, including hidden content, attributes and URLs, may contain private information. Review before sharing.",
            ));
        if self.history_section == 1 {
            body = body.child(baselines);
        }

        let mut runs = div()
            .debug_selector(|| "history-runs-section".into())
            .w_full()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                controls::segmented_group("history-run-type-choices").children(
                    ["All runs", "Checks", "Baseline captures", "Previews"]
                        .into_iter()
                        .enumerate()
                        .map(|(kind, label)| {
                            self.action_control(
                                Control::HistoryKind(kind),
                                label,
                                ActionStyle::Segment {
                                    selected: self.history_kind == kind,
                                },
                                cx,
                            )
                        }),
                ),
            )
            .child(history_help(
                self.history
                    .as_ref()
                    .map(|h| {
                        format!(
                            "{} of {} runs shown on loaded history page {}{}",
                            run_indices.len(),
                            h.runs.len(),
                            h.page + 1,
                            if h.more {
                                " · more history available"
                            } else {
                                ""
                            },
                        )
                    })
                    .unwrap_or_else(|| "No capture/check history page loaded yet".into()),
            ));
        if let Some(h) = &self.history
            && !run_indices.is_empty()
        {
            runs = runs.child(
                controls::page_list(scrollbar::vertical(
                    "history-run-scrollbar",
                    &self.history_run_scroll,
                    div()
                        .id("history-run-list")
                        .track_scroll(&self.history_run_scroll)
                        .debug_selector(|| "history-run-list".into())
                        .w_full()
                        .h(px(history_list_height(run_indices.len())))
                        .flex_shrink_0()
                        .overflow_y_scroll()
                        .flex()
                        .flex_col()
                        .children(run_indices.iter().map(|&index| {
                            let r = &h.runs[index];
                            self.history_choice_row(
                                Control::HistoryRun(index),
                                format!(
                                    "{} · {}",
                                    r.run
                                        .started_at
                                        .map(history_date)
                                        .unwrap_or_else(|| "Start time not recorded".into()),
                                    history_run_type(&r.run.intent)
                                ),
                                format!("{} · {}", job_label(r.run.state), run_kind_summary(r)),
                                index == self.run_history_index,
                                cx,
                            )
                        })),
                ))
                .flex_shrink_0(),
            );
        } else if self.history.as_ref().is_some_and(|h| !h.runs.is_empty()) {
            runs = runs.child(history_help(
                "No runs on this loaded page match. Change the filters or load another history page; this is not a project-wide search.",
            ));
        }
        if run_indices.len() > 4 {
            runs = runs.child(history_help("Scroll or Tab through loaded runs."));
        }
        if let Some(h) = &self.history
            && let Some(r) = selected_run
        {
            runs = runs.child(
                div()
                    .w_full()
                    .debug_selector(|| "history-selected-run".into())
                    .border_t_1()
                    .border_b_1()
                    .border_color(rgb(palette().line))
                    .py_3()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!(
                                "{} — {}",
                                r.run.started_at.map(history_date).unwrap_or_else(|| "Start time not recorded".into()),
                                history_run_type(&r.run.intent),
                            )),
                    )
                    .child(format!("{} · {}", job_label(r.run.state), run_kind_summary(r)))
                    .child(div().flex().flex_wrap().gap_2()
                        .child(self.button(Control::Phase6(19), if matches!(r.run.intent, RunIntent::Check { .. }) { "Open check results" } else if r.run.capture_plan.mode.dom_snapshots() { "Open captures" } else { "Open screenshots" }, ActionStyle::Primary, cx))
                        .child(self.button(Control::Phase6(5), "Delete run…", ActionStyle::Danger, cx)))
                    .child(history_help(format!("Source environment: {}", r.run.environment.name)))
                     .child(history_help(format!(
                         "Finished {}",
                        r.run.finished_at.map(history_date).unwrap_or_else(|| "not recorded".into()),
                    )))
                    .child(history_help(if h.imported_runs.contains(&r.run.id) {
                        "Imported screenshots — not verified on this computer. Check that imported ignored areas fit the screenshots before using them."
                    } else {
                        "Recorded locally on this computer."
                    })),
            );
        } else if self.history.is_none() {
            runs = runs.child(history_help(
                "Capture history is not loaded yet. Browsing does not wait for storage totals.",
            ));
        } else if self.history.as_ref().is_some_and(|h| h.runs.is_empty()) {
            runs = runs.child(controls::empty(
                "No capture runs on this history page",
                "Open Pages & baseline to capture reviewed pages, or Review changes to run a check against a saved baseline. Other recorded runs may be on another history page.",
            ));
        } else if self
            .history
            .as_ref()
            .is_some_and(|h| h.runs.get(self.run_history_index).is_some())
        {
            runs = runs.child(history_help(
                "The selected run is outside the filters. Clear the search, choose All runs or select a matching row to see its details and actions.",
            ));
        } else {
            runs = runs.child(history_help("Choose a run above to inspect it."));
        }
        runs = runs.child(self.history_pagination(cx));
        if self.history_section == 0 {
            body = body.child(runs);
        }
        if self.history_section == 1 {
            body = body.child(self.history_pagination(cx));
        }

        // Keep the existing validated privacy/deletion previews and their exact
        // conditional action membership; no mutations or I/O happen here.
        if self.export_preview.is_some()
            || self.import_preview.is_some()
            || self.deletion_preview.is_some()
            || self.repair_preview.is_some()
        {
            body = body.child(self.portability_confirmation(cx));
        }

        let mut storage = div()
            .debug_selector(|| "history-storage-section".into())
            .w_full()
            .border_t_1()
            .border_color(rgb(palette().line))
            .pt_5()
            .flex()
            .flex_col()
            .gap_3()
            .child(history_title("Storage & manual cleanup"))
            .child(history_help(
                "Nothing is deleted automatically. Select a record in Baselines or Runs to preview deletion.",
            ));
        if let Some(s) = &self.project_storage {
            for (label, bytes) in [
                ("Saved project files", s.bytes),
                ("Original screenshots", s.original_bytes),
                ("Generated image files", s.derived_bytes),
                ("Files used only by this project", s.exclusive_bytes),
                ("Files shared with other projects", s.shared_bytes),
            ] {
                storage = storage.child(history_storage_row(label, history_bytes(bytes)));
            }
            storage = storage.child(history_storage_row(
                "Free disk space",
                s.available_bytes
                    .map(history_bytes)
                    .unwrap_or_else(|| "Could not measure available space".into()),
            ));
        } else if self.storage_pending.is_some() {
            storage = storage.child(controls::loading("Measuring local storage…", "Calculating the space used by this project’s saved files. History stays available while this runs."));
        } else {
            storage = storage.child(history_help(
                "Storage totals are not available yet. Choose Refresh storage totals to measure them. You can browse History while waiting.",
            ));
        }
        if self.history_section == 2 {
            body = body.child(
            storage
                .child(history_help(
                    "Files shared with other projects are kept. Files used only by this project may still be needed by its saved baselines. Preview deletion in Runs or Baselines to see what can be removed. Browser repair is separate in App settings.",
                ))
                .child(self.button(
                    Control::Phase6(8),
                    "Refresh storage totals",
                    ActionStyle::Secondary,
                    cx,
                )),
        );
        }

        let mut provenance = div().w_full().flex().flex_col().gap_3();
        if self.details.contains(&Detail::History) {
            if let Some(s) = &self.project_storage {
                provenance = provenance.child(history_help(format!("Exact storage: total {} bytes · originals {} bytes · derivatives {} bytes · exclusive {} bytes · shared {} bytes", s.bytes, s.original_bytes, s.derived_bytes, s.exclusive_bytes, s.shared_bytes)));
            }
            provenance = provenance.child(history_help(
                "Source records for the selected baseline and run. The IDs below identify saved records and files, not website names.",
            ));
            // Direct rows are the browsing surface; retain serial controls only
            // as an optional keyboard fallback alongside technical inspection.
            if selected_baseline.is_some() || selected_run.is_some() {
                provenance = provenance.child(
                    div()
                        .flex()
                        .flex_wrap()
                        .gap_2()
                        .when(selected_baseline.is_some(), |el| {
                            el.child(self.button(
                                Control::Phase5(40),
                                "Previous baseline",
                                ActionStyle::Secondary,
                                cx,
                            ))
                            .child(self.button(
                                Control::Phase5(41),
                                "Next baseline",
                                ActionStyle::Secondary,
                                cx,
                            ))
                        })
                        .when(selected_run.is_some(), |el| {
                            el.child(self.button(
                                Control::Phase6(10),
                                "Previous run",
                                ActionStyle::Secondary,
                                cx,
                            ))
                            .child(self.button(
                                Control::Phase6(9),
                                "Next run",
                                ActionStyle::Secondary,
                                cx,
                            ))
                        }),
                );
            }
            if let Some(b) = selected_baseline {
                provenance = provenance.child(history_help(format!(
                    "Baseline {} · created {} · parent {:?}",
                    b.id,
                    history_date(b.created_at),
                    b.parent,
                )));
                for r in b.references.iter().take(64) {
                    provenance = provenance.child(history_help(format!(
                        "Page {} / profile {} · source run {} · captured {} · image {}",
                        r.key.page,
                        r.key.profile,
                        r.run,
                        history_date(r.captured_at),
                        r.artifact.as_str(),
                    )));
                }
                if b.references.len() > 64 {
                    provenance = provenance.child(history_help(format!(
                        "Showing first 64 of {} references. The exact full reference map remains retained.",
                        b.references.len(),
                    )));
                }
                for r in b.dom_references.iter().take(64) {
                    provenance = provenance.child(history_help(format!("DOM snapshot · page {} / profile {} · source run {} · captured {} · artifact {} · policy {}", r.key.page, r.key.profile, r.run, history_date(r.captured_at), r.artifact.as_str(), r.policy.version)));
                }
                if b.dom_references.len() > 64 {
                    provenance = provenance.child(history_help(format!("Showing first 64 of {} DOM references. Complete sources remain available by opening their recorded run.", b.dom_references.len())));
                }
            }
            if let Some(r) = selected_run {
                provenance = provenance
                    .child(
                        history_help(format!("Run {}", r.run.id))
                            .debug_selector(|| "history-provenance-run".into()),
                    )
                    .child(history_help(format!(
                        "Rendering: {} / {} · browser {} · worker {} · fonts {:?}",
                        r.run.rendering.os,
                        r.run.rendering.architecture,
                        r.run.rendering.browser,
                        r.run.rendering.worker_version,
                        r.run.rendering.fonts_fingerprint,
                    )));
                for c in r.all_captures().take(32) {
                    provenance = provenance.child(history_help(format!(
                        "{} → {} · {} {}×{} DPR {} · {:?} · original {}",
                        c.metadata.requested,
                        c.metadata.final_url,
                        c.metadata.profile.name,
                        c.metadata.width,
                        c.metadata.height,
                        c.metadata.profile.dpr,
                        c.metadata.profile.image_mode,
                        c.image.original.as_str(),
                    )));
                }
                provenance = provenance.child(history_help(format!(
                    "Showing {} of {} captured screenshots. All screenshots and the saved page list are kept.",
                    r.all_captures().count().min(32),
                    r.all_captures().count(),
                )));
                for capture in r.dom_captures.iter().take(32) {
                    let c = &capture.metadata;
                    provenance = provenance.child(history_help(format!(
                        "DOM snapshot · {} → {} · {} · {} nodes · {} bytes · {} · artifact {}",
                        c.requested,
                        c.final_url,
                        c.profile.name,
                        c.node_count,
                        c.bytes,
                        if c.stable { "stable" } else { "unstable" },
                        c.sha256.as_str()
                    )));
                }
                if !r.dom_captures.is_empty() {
                    provenance = provenance.child(history_help(format!("Showing {} of {} DOM capture records. Open the run to inspect complete verified sources. {} {}", r.dom_captures.len().min(32), r.dom_captures.len(), regressee_core::dom_snapshot::DOM_COVERAGE_NOTICE, regressee_core::dom_snapshot::DOM_PRIVACY_NOTICE)));
                }
            }
        }
        body = body.child(
            div()
                .w_full()
                .min_w_0()
                .flex_shrink_0()
                .flex()
                .flex_col()
                .border_t_1()
                .border_color(rgb(palette().line))
                .pt_5()
                .child(self.inline_disclosure(
                    Control::Details(Detail::History),
                    "Technical details",
                    "",
                    self.details.contains(&Detail::History),
                    provenance,
                    cx,
                )),
        );
        body.into_any_element()
    }

    /// Loaded history is bounded (128 baseline versions and 64 runs). Keep rows mounted so
    /// the shared focus order never includes an unmounted virtual-list option.
    fn history_choice_row(
        &self,
        control: Control,
        title: String,
        details: String,
        selected: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let enabled = self.control_enabled(control, cx);
        let handle = self.handle(control, cx);
        controls::history_row(
            SharedString::from(format!("{control:?}")),
            title,
            details,
            &handle,
            selected,
            enabled,
        )
        .debug_selector(move || format!("history-row-{control:?}"))
        .when(enabled, |el| {
            el.on_click(cx.listener(move |this, event, window, cx| {
                this.click_control(control, event, window, cx)
            }))
        })
        .into_any_element()
    }
}

fn history_matches(query: &str, text: &str) -> bool {
    // The observer stores a normalized query; splitting permits a date + type
    // or environment + outcome search without requiring their display order.
    let text = text.to_lowercase();
    query.split_whitespace().all(|word| text.contains(word))
}

fn baseline_kind_summary(b: &BaselineVersion) -> String {
    match b.capture_plan.mode {
        CaptureMode::ScreenshotsOnly => format!(
            "{} saved screenshots / {} intended screenshots",
            b.references.len(),
            b.intended_count
        ),
        CaptureMode::DomSnapshotsOnly => format!(
            "{} saved DOM snapshots / {} intended DOM snapshots",
            b.dom_references.len(),
            b.intended_count
        ),
        CaptureMode::ScreenshotsAndDomSnapshots => format!(
            "{} screenshots · {} DOM snapshots / {} intended per kind",
            b.references.len(),
            b.dom_references.len(),
            b.intended_count
        ),
    }
}
fn run_kind_summary(r: &RunRecord) -> String {
    let counts = regressee_app::task::CaptureStatus::from_run(&r.run);
    let mut parts = Vec::new();
    for (enabled, label, c) in [
        (
            r.run.capture_plan.mode.screenshots(),
            "screenshots",
            &counts.screenshots,
        ),
        (
            r.run.capture_plan.mode.dom_snapshots(),
            "DOM snapshots",
            &counts.dom_snapshots,
        ),
    ] {
        if enabled {
            parts.push(format!(
                "{} / {} {label} captured{}",
                c.captured,
                c.requested,
                if c.baseline_omitted > 0 {
                    format!(" · {} baseline omissions", c.baseline_omitted)
                } else {
                    String::new()
                }
            ));
        }
    }
    parts.join(" · ")
}

#[cfg(test)]
fn history_short_id(id: String) -> String {
    id.chars().take(8).collect()
}

fn history_kind_matches(kind: usize, intent: &RunIntent) -> bool {
    match kind {
        0 => true,
        1 => matches!(intent, RunIntent::Check { .. }),
        2 => matches!(intent, RunIntent::NewBaseline),
        3 => matches!(intent, RunIntent::RepresentativePreview),
        _ => false,
    }
}

pub(super) fn history_run_type(intent: &RunIntent) -> &'static str {
    match intent {
        RunIntent::Check { .. } => "Comparison check",
        RunIntent::NewBaseline => "Baseline capture",
        RunIntent::RepresentativePreview => "Screenshot preview",
    }
}

fn history_baseline_search(baseline: &BaselineVersion, active: bool) -> String {
    format!(
        "Baseline version {} {} {} {} {} {} / {} saved {} {}",
        baseline.id,
        history_date(baseline.created_at),
        if active {
            "Current baseline Active reference"
        } else {
            "Saved version Retained version"
        },
        if baseline.parent.is_some() {
            "Composite inherited"
        } else {
            "Whole capture"
        },
        baseline.parent.map(|id| id.to_string()).unwrap_or_default(),
        baseline.references.len(),
        baseline.intended_count,
        if baseline.explicitly_partial {
            "Partial"
        } else {
            "Full coverage"
        },
        baseline_kind_summary(baseline),
    )
}

fn history_capture_span(baseline: &BaselineVersion) -> Option<(i64, i64)> {
    let mut times = baseline
        .references
        .iter()
        .map(|reference| reference.captured_at)
        .chain(baseline.dom_references.iter().map(|r| r.captured_at));
    let first = times.next()?;
    Some(times.fold((first, first), |(min, max), time| {
        (min.min(time), max.max(time))
    }))
}

fn history_run_search(record: &RunRecord, imported: bool) -> String {
    let run = &record.run;
    format!(
        "{} {} {:?} {} {:?} {} {} {} {} {} {} / {} captured {} {}",
        run.id,
        history_run_type(&run.intent),
        run.intent,
        job_label(run.state),
        run.state,
        run.started_at.map(history_date).unwrap_or_default(),
        run.finished_at.map(history_date).unwrap_or_default(),
        run.environment.name,
        run.environment.id,
        run.rendering.os,
        record.all_captures().count(),
        run.targets.len(),
        if imported { "Imported" } else { "Local" },
        run_kind_summary(record),
    )
}

fn history_title(title: &'static str) -> Div {
    div()
        .text_size(px(TITLE_TEXT_SIZE))
        .line_height(px(TITLE_LINE_HEIGHT))
        .font_weight(FontWeight::SEMIBOLD)
        .child(title)
}

/// Use actual mounted 64px rows, with a four-row bound for long loaded pages.
pub(super) fn history_list_height(records: usize) -> f32 {
    records.min(4) as f32 * 64.
}

fn history_help(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(14.))
        .line_height(px(CONTROL_LINE_HEIGHT))
        .text_color(rgb(palette().muted))
        .child(text.into())
}

fn history_storage_row(label: &'static str, value: String) -> Div {
    div()
        .w_full()
        .flex()
        .flex_wrap()
        .justify_between()
        .gap_2()
        .border_b_1()
        .border_color(rgb(palette().line))
        .pb_2()
        .child(label)
        .child(value)
}

fn history_bytes(bytes: u64) -> String {
    human_bytes(bytes)
}

/// An exact UTC civil timestamp, without a new dependency or local-time claim.
/// Stored timestamps are Unix seconds, like `relative_time` in the shell.
pub(super) fn history_date(timestamp: i64) -> String {
    let days = timestamp.div_euclid(86_400);
    let seconds = timestamp.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let day_of_era = z - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    if month <= 2 {
        year += 1;
    }
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn history_dates_are_exact_and_explicitly_utc() {
        assert_eq!(history_date(0), "1970-01-01 00:00:00 UTC");
        assert_eq!(history_date(-1), "1969-12-31 23:59:59 UTC");
        assert_eq!(history_date(951_782_400), "2000-02-29 00:00:00 UTC");
        assert_eq!(history_date(1_792_080_000), "2026-10-15 16:00:00 UTC");
    }

    #[test]
    fn storage_values_lead_with_readable_units() {
        assert_eq!(history_bytes(0), human_bytes(0));
        assert_eq!(history_bytes(1_048_577), human_bytes(1_048_577));
    }

    #[test]
    fn history_search_combines_terms_without_requiring_display_order() {
        let text = "2026-10-15 16:00:00 UTC Comparison check Complete Staging abc12345";
        assert!(history_matches("", text));
        assert!(history_matches(" staging  2026-10-15 check ", text));
        assert!(history_matches("abc12345 16:00 utc", text));
        assert!(!history_matches("staging cancelled", text));
        assert!(!history_matches("2026-10-16", text));
    }

    #[test]
    fn run_kind_controls_match_the_pinned_order() {
        use regressee_core::domain::BaselineId;
        let intents = [
            RunIntent::Check {
                baseline: BaselineId::new(),
            },
            RunIntent::NewBaseline,
            RunIntent::RepresentativePreview,
        ];
        for (index, intent) in intents.iter().enumerate() {
            assert!(history_kind_matches(0, intent));
            for kind in 1..=3 {
                assert_eq!(history_kind_matches(kind, intent), kind == index + 1);
            }
            assert!(!history_kind_matches(4, intent));
        }
    }

    fn baseline_fixture() -> BaselineVersion {
        use regressee_core::domain::*;
        BaselineVersion {
            capture_plan: Default::default(),
            dom_references: vec![],
            intended_targets: vec![],
            source_run: None,
            id: BaselineId::new(),
            project: ProjectId::new(),
            parent: Some(BaselineId::new()),
            inventory: InventoryId::new(),
            created_at: 1_792_080_000,
            references: vec![],
            intended_count: 3,
            explicitly_partial: true,
        }
    }

    #[test]
    fn baseline_search_retains_state_composite_date_and_full_identifier() {
        let baseline = baseline_fixture();
        let active = history_baseline_search(&baseline, true);
        let retained = history_baseline_search(&baseline, false);
        assert!(history_matches(
            "baseline active composite 2026-10-15 partial",
            &active
        ));
        assert!(!history_matches("retained", &active));
        assert!(history_matches("current baseline", &active));
        assert!(history_matches("retained inherited 16:00", &retained));
        assert!(history_matches("saved version", &retained));
        assert!(!history_matches("active", &retained));
        assert!(history_matches(&baseline.id.to_string(), &retained));
        assert!(history_matches(
            &history_short_id(baseline.id.to_string()),
            &retained
        ));
    }

    #[test]
    fn reference_capture_span_uses_actual_times_not_baseline_save_time() {
        use regressee_core::domain::*;
        let mut baseline = baseline_fixture();
        assert_eq!(history_capture_span(&baseline), None);
        for time in [200, -1, 100] {
            baseline.references.push(BaselineReference {
                key: TargetKey {
                    page: PageId::new(),
                    profile: ProfileId::new(),
                },
                run: RunId::new(),
                artifact: "a".repeat(64).try_into().unwrap(),
                captured_at: time,
            });
        }
        assert_eq!(history_capture_span(&baseline), Some((-1, 200)));
    }

    #[test]
    fn run_search_includes_source_environment_timestamps_outcome_and_import_origin() {
        use regressee_core::domain::*;
        let project = Project::draft("Search fixture".into(), "https://example.test/").unwrap();
        let record = RunRecord {
            dom_captures: vec![],
            run: CaptureRun {
                capture_plan: Default::default(),
                id: RunId::new(),
                project: project.id,
                intent: RunIntent::NewBaseline,
                inventory: InventoryRevision {
                    id: InventoryId::new(),
                    project: project.id,
                    identity_version: 1,
                    pages: vec![],
                    coverage: DiscoveryCoverage::Complete,
                },
                profiles: project.profiles.clone(),
                environment: project.environments[0].clone(),
                rendering: RenderingEnvironment {
                    os: "Linux".into(),
                    architecture: "x86_64".into(),
                    browser: "Chromium".into(),
                    worker_version: "fixture".into(),
                    profile_version: 1,
                    fonts_fingerprint: None,
                },
                state: JobState::CompletedWithErrors,
                started_at: Some(0),
                finished_at: Some(86_400),
                targets: vec![],
            },
            capture: None,
            captures: vec![],
            queue: None,
        };
        let imported = history_run_search(&record, true);
        assert!(history_matches(
            "newbaseline linux incomplete 1970-01-01 1970-01-02 imported",
            &imported
        ));
        assert!(history_matches(
            &record.run.environment.name.to_lowercase(),
            &imported
        ));
        assert!(history_matches(
            &record.run.environment.id.to_string(),
            &imported
        ));
        assert!(history_matches(&record.run.id.to_string(), &imported));
        assert!(!history_matches("local", &imported));
        assert!(history_matches(
            "local",
            &history_run_search(&record, false)
        ));
    }
}
