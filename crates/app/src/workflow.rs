// SPDX-License-Identifier: MPL-2.0
//! Native workflow presentation; no storage or network work is performed here.
use super::*;
use std::collections::{BTreeMap, BTreeSet};

/// Built once per presentation operation, never once per selected target.
/// No persisted/cache identity is trusted in place of current evidence checks.
#[derive(Default)]
pub(super) struct ReviewIndex {
    classifications: BTreeMap<TargetKey, Classification>,
    captured: BTreeSet<TargetKey>,
    eligible: BTreeSet<TargetKey>,
    target_labels: BTreeMap<TargetKey, String>,
    profile_labels: BTreeMap<regressee_core::domain::ProfileId, String>,
}
impl ReviewIndex {
    fn build(
        evidence: &ProjectEvidence,
        active: Option<regressee_core::domain::BaselineId>,
    ) -> Self {
        let Some(run) = &evidence.latest else {
            return Self::default();
        };
        let comparison = evidence
            .comparison
            .as_ref()
            .filter(|a| a.revision.run == run.run.id);
        let pages: BTreeMap<_, _> = run
            .run
            .inventory
            .pages
            .iter()
            .map(|p| (p.id, p.canonical.to_string()))
            .collect();
        let classifications = comparison
            .map(|a| {
                a.revision
                    .results
                    .iter()
                    .map(|r| (r.key.clone(), r.classification))
                    .collect()
            })
            .unwrap_or_default();
        let mut index = Self {
            classifications,
            captured: run.all_captures().map(|c| c.metadata.key.clone()).collect(),
            target_labels: run
                .run
                .targets
                .iter()
                .map(|t| {
                    (
                        t.key.clone(),
                        pages
                            .get(&t.key.page)
                            .cloned()
                            .unwrap_or_else(|| t.requested.to_string()),
                    )
                })
                .collect(),
            profile_labels: run
                .run
                .profiles
                .iter()
                .map(|p| (p.id, format!("{} · {} × {}", p.name, p.width, p.height)))
                .collect(),
            ..Self::default()
        };
        let Some(comparison) = comparison else {
            return index;
        };
        let Some(baseline) = evidence
            .baseline
            .as_ref()
            .filter(|b| Some(b.id) == active && b.id == comparison.revision.baseline)
        else {
            return index;
        };
        let references: BTreeSet<_> = baseline.references.iter().map(|r| &r.key).collect();
        let captures: BTreeSet<_> = run
            .all_captures()
            .filter(|c| c.metadata.stable && c.metadata.warnings.is_empty())
            .map(|c| &c.metadata.key)
            .collect();
        let results: BTreeSet<_> = comparison
            .revision
            .results
            .iter()
            .filter(|r| {
                r.eligible_pixels > 0
                    && matches!(
                        r.classification,
                        Classification::Changed | Classification::NoSignificantChange
                    )
            })
            .map(|r| &r.key)
            .collect();
        let reports: BTreeSet<_> = comparison.perceptual.as_ref().map(|p| p.reports.iter().filter(|(_, r)| r.gates.is_empty() && !r.warnings.contains(&regressee_app::analysis_engine::Warning::RenderingEnvironmentMismatch)).map(|(key, _)| key).collect()).unwrap_or_default();
        index.eligible = run
            .run
            .targets
            .iter()
            .filter(|t| {
                matches!(t.outcome, TargetOutcome::Captured { .. })
                    && references.contains(&t.key)
                    && captures.contains(&t.key)
                    && results.contains(&t.key)
                    && reports.contains(&t.key)
            })
            .map(|t| t.key.clone())
            .collect();
        index
    }
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Coverage {
    total: usize,
    changed: usize,
    unchanged: usize,
    missing: usize,
    unclassified: usize,
}
impl Coverage {
    fn from_run(run: &regressee_core::evidence::RunRecord, index: &ReviewIndex) -> Self {
        let mut coverage = Self::default();
        for target in &run.run.targets {
            coverage.count(
                matches!(target.outcome, TargetOutcome::Captured { .. })
                    && index.captured.contains(&target.key),
                index.classifications.get(&target.key).copied(),
            );
        }
        coverage
    }
    fn count(&mut self, captured: bool, classification: Option<Classification>) {
        self.total += 1;
        if !captured {
            self.missing += 1;
        } else {
            match classification {
                Some(Classification::Changed) => self.changed += 1,
                Some(Classification::NoSignificantChange) => self.unchanged += 1,
                Some(Classification::CouldNotCapture | Classification::NotComparable) => {
                    self.missing += 1
                }
                None => self.unclassified += 1,
            }
        }
    }
}

impl Shell {
    /// Offscreen fixtures must call this with their capture width too; the
    /// native render path calls it with the actual desktop viewport width.
    pub(super) fn configure_review_geometry(&mut self, width: f32) {
        self.narrow_workspace = width < 1100.;
        self.working_width = width - 240.;
        if self.viewer_fit && self.viewer.is_some() {
            let available = self.working_width - if self.narrow_workspace { 212. } else { 284. };
            let pane = if self.viewer_mode == 0 {
                (available - 12.) / 2.
            } else {
                available
            };
            self.zoom = (pane / self.viewer_dimensions().0.max(1) as f32).clamp(0.05, 4.);
        }
    }
    pub(super) fn results(&self, cx: &mut Context<Self>) -> AnyElement {
        if self.project.is_none() {
            return self.project_picker(cx);
        }
        let pages = self.focus.review_surface == ReviewSurface::Pages || self.preview.is_some();
        let (title, _next) = if self.focus.candidate {
            (
                "Inspect baseline",
                "Inspect the capture, then explicitly save your reference.",
            )
        } else if self.focus.review_surface == ReviewSurface::Capture {
            (
                "Capture coverage",
                "Completed evidence is kept; activation remains a separate action.",
            )
        } else if pages {
            (
                "Pages & baseline",
                "Review the intended inventory before capturing a replacement.",
            )
        } else {
            (
                "Review changes",
                "Inspect changes and gaps; preview intended approvals before saving.",
            )
        };
        let mut actions = div().flex().flex_wrap().items_center().gap_2();
        if self.focus.candidate {
            actions = actions.child(self.button(
                Control::SaveBaseline,
                "Save baseline…",
                ActionStyle::Primary,
                cx,
            ));
        }
        if self.focus.baseline && !pages {
            actions = actions.child(self.button(
                Control::CheckChanges,
                "Run check",
                ActionStyle::Primary,
                cx,
            ));
        }
        if pages || (!self.focus.baseline && !self.focus.candidate) {
            actions = actions.child(self.button(
                Control::CaptureBaseline,
                if self.preview.is_some() {
                    "Capture pages"
                } else {
                    "Find pages"
                },
                ActionStyle::Primary,
                cx,
            ));
        }
        if pages && self.evidence.baseline.is_some() && self.preview.is_none() {
            actions = actions.child(self.guided_button(16, "Page list", cx));
        }
        if self.preview.is_some() {
            actions = actions.child(self.button(
                Control::Phase5(30),
                "Preview page",
                ActionStyle::Secondary,
                cx,
            ));
        }
        actions = actions.child(self.button(
            Control::RefreshEvidence,
            "Refresh",
            ActionStyle::Secondary,
            cx,
        ));
        div()
            .debug_selector(|| "bounded-review".into())
            .size_full()
            .min_h_0()
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .flex_shrink_0()
                    .flex()
                    .flex_col()
                    .gap_1()
                    .child(
                        div()
                            .flex()
                            .flex_wrap()
                            .items_center()
                            .justify_between()
                            .gap_2()
                            .child(
                                div()
                                    .text_size(px(24.))
                                    .font_weight(FontWeight::SEMIBOLD)
                                    .child(title),
                            )
                            .child(actions),
                    )
                    .child(
                        div()
                            .debug_selector(|| "comparison-context".into())
                            .text_size(px(14.))
                            .line_height(px(20.))
                            .child(self.comparison_context()),
                    ),
            )
            .child(self.coverage_summary())
            .child(
                div()
                    .debug_selector(|| "queue-beside-viewer".into())
                    .flex_1()
                    .min_h_0()
                    .min_w_0()
                    .flex()
                    .gap_3()
                    .overflow_hidden()
                    .child(
                        div()
                            .debug_selector(|| "review-queue".into())
                            .w(px(if self.narrow_workspace { 200. } else { 272. }))
                            .h_full()
                            .min_h_0()
                            .flex_shrink_0()
                            .child(self.inventory_and_results(cx)),
                    )
                    .child(
                        div()
                            .debug_selector(|| "review-viewer".into())
                            .flex_1()
                            .min_w_0()
                            .h_full()
                            .min_h_0()
                            .flex()
                            .flex_col()
                            .gap_2()
                            .child(self.viewer_section(cx))
                            .when(self.review_tools, |viewer| {
                                viewer.child(
                                    div()
                                        .id("review-details-scroll")
                                        .debug_selector(|| "review-details-scroll".into())
                                        .h(px(100.))
                                        .flex_shrink_0()
                                        .overflow_y_scroll()
                                        .track_scroll(&self.review_details_scroll)
                                        .child(self.selected_capture_details(cx)),
                                )
                            }),
                    ),
            )
            .into_any_element()
    }

    fn displayed_run(&self) -> Option<&regressee_core::evidence::RunRecord> {
        let id = self.evidence.comparison.as_ref().map(|a| a.revision.run);
        self.evidence
            .latest
            .as_ref()
            .filter(|r| id.is_none_or(|id| id == r.run.id))
            .or_else(|| {
                self.history
                    .as_ref()
                    .and_then(|h| h.runs.iter().find(|r| Some(r.run.id) == id))
            })
    }

    fn compared_baseline(&self) -> Option<&regressee_core::domain::BaselineVersion> {
        let id = self.evidence.comparison.as_ref()?.revision.baseline;
        self.evidence
            .baseline
            .as_ref()
            .filter(|b| b.id == id)
            .or_else(|| self.evidence.baselines.iter().find(|b| b.id == id))
    }

    pub(super) fn comparison_context(&self) -> String {
        let reference = self
            .evidence
            .comparison
            .as_ref()
            .map(|a| {
                let historical = self
                    .project
                    .as_ref()
                    .is_some_and(|p| p.project.active_baseline != Some(a.revision.baseline));
                format!(
                    "{}baseline {}",
                    if historical {
                        "Historical "
                    } else {
                        "Compared "
                    },
                    short_id(a.revision.baseline.to_string())
                )
            })
            .unwrap_or_else(|| "No comparison reference — capture inspection only".into());
        let check = self
            .displayed_run()
            .map(|r| {
                format!(
                    "{} {} · {} · {}",
                    if matches!(r.run.intent, RunIntent::Check { .. }) {
                        "Check"
                    } else {
                        "Capture"
                    },
                    short_id(r.run.id.to_string()),
                    r.run
                        .finished_at
                        .map(capture_time)
                        .unwrap_or_else(|| "not finished".into()),
                    r.run.environment.name
                )
            })
            .unwrap_or_else(|| {
                self.evidence
                    .comparison
                    .as_ref()
                    .map(|a| {
                        format!(
                            "Check {} · record not loaded",
                            short_id(a.revision.run.to_string())
                        )
                    })
                    .unwrap_or_else(|| "No saved capture yet".into())
            });
        format!("{reference} · {check}")
    }

    pub(super) fn pane_caption(&self, pane: usize) -> (String, String, String) {
        let key = self.selected_key.as_ref();
        let run = self.displayed_run();
        let profile = run
            .and_then(|r| {
                r.run
                    .profiles
                    .iter()
                    .find(|p| Some(p.id) == key.map(|k| k.profile))
            })
            .map(|p| format!("{} · {} × {} CSS px", p.name, p.width, p.height))
            .unwrap_or_else(|| "Profile record not loaded".into());
        if pane == 0
            && let Some(comparison) = &self.evidence.comparison
        {
            let id = comparison.revision.baseline;
            let reference = self
                .compared_baseline()
                .and_then(|b| b.references.iter().find(|r| Some(&r.key) == key));
            let source = reference.and_then(|r| {
                self.history
                    .as_ref()
                    .and_then(|h| h.runs.iter().find(|run| run.run.id == r.run))
            });
            let capture = source
                .and_then(|r| key.and_then(|key| r.evidence(key)))
                .or_else(|| {
                    self.evidence
                        .before
                        .as_ref()
                        .filter(|c| Some(&c.metadata.key) == key)
                });
            let origin = capture
                .map(|c| capture_origin(&c.metadata.final_url))
                .unwrap_or_else(|| "Origin record not loaded".into());
            let environment = source
                .map(|r| r.run.environment.name.as_str())
                .unwrap_or("environment not loaded");
            let time = reference
                .map(|r| capture_time(r.captured_at))
                .unwrap_or_else(|| "capture time not loaded".into());
            let profile = capture
                .map(|c| {
                    format!(
                        "{} · {} × {} CSS px",
                        c.metadata.profile.name,
                        c.metadata.profile.width,
                        c.metadata.profile.height
                    )
                })
                .unwrap_or(profile);
            (
                format!("Baseline {}\n{time}", short_id(id.to_string())),
                format!("{environment}\n{origin}"),
                profile,
            )
        } else {
            let capture = run.and_then(|r| key.and_then(|key| r.evidence(key)));
            let label = if pane == 2 {
                if self.viewer_mode == 3 {
                    "Heatmap"
                } else {
                    "Difference"
                }
            } else {
                "Capture"
            };
            let title = run
                .map(|r| {
                    format!(
                        "{label} {}\n{}",
                        short_id(r.run.id.to_string()),
                        r.run
                            .finished_at
                            .map(capture_time)
                            .unwrap_or_else(|| "unfinished".into())
                    )
                })
                .unwrap_or_else(|| format!("{label} record not loaded"));
            let origin = capture
                .map(|c| capture_origin(&c.metadata.final_url))
                .unwrap_or_else(|| "no saved image".into());
            (
                title,
                format!(
                    "{}\n{origin}",
                    run.map(|r| r.run.environment.name.as_str())
                        .unwrap_or("environment not loaded")
                ),
                profile,
            )
        }
    }

    pub(super) fn approval_decision(&self, cx: &mut Context<Self>) -> AnyElement {
        let Some(preview) = self.approval.as_ref().filter(|p| self.approval_matches(p)) else {
            return div().into_any_element();
        };
        let index = self.review_index();
        let selected: BTreeSet<_> = preview.selected.iter().cloned().collect();
        let provenance: BTreeMap<_, _> = preview
            .provenance
            .iter()
            .map(|(key, requested, final_url)| (key.clone(), format!("{requested} → {final_url}")))
            .collect();
        let rows = preview.baseline.references.len();
        let mut order: Vec<_> = (0..rows).collect();
        order.sort_by_key(|i| !selected.contains(&preview.baseline.references[*i].key));
        let inspected = &preview.baseline.references
            [order[self.approval_reference_index.min(rows.saturating_sub(1))]];
        let exact = format!(
            "Page: {}\nProfile: {}\nSource run: {}\nCaptured: {}\nOriginal: {}\n{}",
            index
                .target_labels
                .get(&inspected.key)
                .cloned()
                .unwrap_or_else(|| inspected.key.page.to_string()),
            index
                .profile_labels
                .get(&inspected.key.profile)
                .cloned()
                .unwrap_or_else(|| inspected.key.profile.to_string()),
            inspected.run,
            capture_time(inspected.captured_at),
            inspected.artifact.as_str(),
            provenance
                .get(&inspected.key)
                .cloned()
                .unwrap_or_else(|| "Inherited provenance retained in History".into())
        );
        div().id("approval-modal-backdrop").occlude().absolute().inset_0().bg(rgba(0x183f6588)).flex().items_center().justify_center()
            .child(div().debug_selector(|| "approval-decision".into()).w(px(if self.narrow_workspace { 580. } else { 760. })).max_w_full().h(px(420.)).max_h_full().p_4().bg(rgb(SURFACE)).border_1().border_color(rgb(LINE)).rounded(px(4.)).flex().flex_col().gap_2()
                .child(div().text_size(px(22.)).font_weight(FontWeight::SEMIBOLD).child("Confirm selected reference changes"))
                .child(div().debug_selector(|| "approval-impact-summary".into()).text_size(px(14.)).child(format!("{} selected references replaced · {} inherited references kept", preview.selected.len(), preview.inherited)))
                .child(div().text_size(px(14.)).child("Composite capture times are not a simultaneous snapshot. Earlier versions remain in History."))
                .child(uniform_list("approval-reference-list", rows, cx.processor(move |this, range: std::ops::Range<usize>, _, cx| {
                    let Some(p) = &this.approval else { return Vec::new(); };
                    range.map(|i| {
                        let r = &p.baseline.references[order[i]];
                        let page = index.target_labels.get(&r.key).cloned().unwrap_or_else(|| format!("Page {}", r.key.page));
                        let profile = index.profile_labels.get(&r.key.profile).cloned().unwrap_or_else(|| format!("Profile {}", r.key.profile));
                        div().id(("approval-reference", i)).debug_selector(move || format!("approval-reference-{i}")).h(px(100.)).py_2().border_b_1().border_color(rgb(LINE)).flex().flex_col().gap_1()
                            .child(div().font_weight(FontWeight::SEMIBOLD).text_ellipsis().child(format!("{} · {page}", if selected.contains(&r.key) { "Replace" } else { "Keep" })))
                            .child(div().text_size(px(14.)).text_ellipsis().child(profile))
                            .child(div().text_size(px(14.)).text_ellipsis().child(format!("Run {} · {} · original {}", r.run, capture_time(r.captured_at), r.artifact.as_str())))
                            .child(div().text_size(px(14.)).text_ellipsis().child(provenance.get(&r.key).cloned().unwrap_or_else(|| "Inherited source provenance retained in History".into())))
                            .on_click(cx.listener(move |this, _, _, cx| { this.approval_reference_index = i; this.approval_provenance_scroll.set_offset(gpui::point(px(0.), px(0.))); cx.notify(); }))
                            .into_any_element()
                    }).collect::<Vec<_>>()
                })).debug_selector(|| "approval-reference-list".into()).flex_1().min_h_0().w_full())
                .child(div().id("approval-exact-provenance").debug_selector(|| "approval-exact-provenance".into()).h(px(100.)).flex_shrink_0().overflow_y_scroll().track_scroll(&self.approval_provenance_scroll).text_size(px(14.)).child(exact))
                .child(div().flex().flex_wrap().justify_between().gap_2()
                    .child(self.guided_button(48, "Previous", cx))
                    .child(self.guided_button(49, "Next", cx))
                    .child(self.guided_button(47, "Keep reviewing", cx))
                    .child(self.button(Control::Phase5(25), "Confirm baseline", ActionStyle::Primary, cx))))
            .into_any_element()
    }

    /// Conservative presentation eligibility. Storage repeats the complete
    /// provenance/profile checks when previewing and accepting a selection.
    pub(super) fn approval_eligible(&self, key: &TargetKey) -> bool {
        self.review_index().eligible.contains(key)
    }

    pub(super) fn review_index(&self) -> ReviewIndex {
        let active =
            if self.mask_dirty || self.analysis_due.is_some() || self.analysis_pending.is_some() {
                None
            } else {
                self.project
                    .as_ref()
                    .and_then(|p| p.project.active_baseline)
            };
        ReviewIndex::build(&self.evidence, active)
    }

    pub(super) fn approval_selection_eligible(&self) -> bool {
        let index = self.review_index();
        !self.approval_selection.is_empty()
            && self
                .approval_selection
                .iter()
                .all(|key| index.eligible.contains(key))
    }

    pub(super) fn approval_bar(&self, cx: &mut Context<Self>) -> AnyElement {
        let index = self.review_index();
        let eligible = self
            .approval_selection
            .iter()
            .filter(|key| index.eligible.contains(*key))
            .count();
        let selected = self
            .selected_key
            .as_ref()
            .is_some_and(|key| self.approval_selection.contains(key));
        div()
            .debug_selector(|| "approval-next-action".into())
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_1()
            .px_6()
            .py_2()
            .bg(rgb(SURFACE))
            .border_t_1()
            .border_color(rgb(LINE))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .items_center()
                    .gap_2()
                    .child(
                        div()
                            .flex_1()
                            .min_w(px(88.))
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!("{eligible} selected")),
                    )
                    .child(self.button(
                        Control::Phase5(23),
                        if selected {
                            "Deselect target"
                        } else {
                            "Select target"
                        },
                        ActionStyle::Choice { selected },
                        cx,
                    ))
                    .child(self.button(
                        Control::Phase5(24),
                        "Review approval…",
                        ActionStyle::Primary,
                        cx,
                    )),
            )
            .child(
                div().text_size(px(13.)).text_color(rgb(MUTED)).child(
                    "Preview before confirming. Unselected references and History are kept.",
                ),
            )
            .into_any_element()
    }

    pub(super) fn setup_heading(&self, cx: &mut Context<Self>) -> AnyElement {
        let (title, next) = match self.focus.setup_stage {
            SetupStage::Website => (
                "Set up your website",
                "Save your website locally. Next, choose whether login is needed.",
            ),
            SetupStage::Access => (
                "Website access",
                "No login? Continue to pages. Otherwise save access settings, log in, then confirm you are ready.",
            ),
            SetupStage::Pages => (
                "Pages & screen sizes",
                "Save your choices, then find pages and review the list before capturing.",
            ),
            SetupStage::Settings => (
                "Project settings",
                "Edit future capture settings. Saving never changes your active baseline.",
            ),
        };
        div()
            .flex()
            .flex_col()
            .gap_4()
            .child(self.heading(title, next))
            .when(self.focus.setup_stage != SetupStage::Settings, |body| {
                body.child(
                    div().flex().flex_wrap().gap_2().children(
                        [
                            (SetupStage::Website, "Website"),
                            (SetupStage::Access, "Access"),
                            (SetupStage::Pages, "Pages & sizes"),
                        ]
                        .into_iter()
                        .filter(|(stage, _)| {
                            !(self.focus.setup_stage == SetupStage::Access
                                && *stage == SetupStage::Pages)
                        })
                        .map(|(stage, label)| {
                            self.button(
                                Control::SetupStage(stage),
                                label,
                                ActionStyle::Choice {
                                    selected: self.focus.setup_stage == stage,
                                },
                                cx,
                            )
                        }),
                    ),
                )
            })
            .into_any_element()
    }

    pub(super) fn stage_control_mounted(&self, control: Control) -> bool {
        if self.focus.screen != Screen::Setup || self.focus.setup_stage != SetupStage::Access {
            return true;
        }
        let login = self
            .draft_settings
            .environments
            .iter()
            .find(|e| Some(e.id) == self.environment_draft)
            .is_some_and(|e| e.access == regressee_core::domain::AccessPolicy::InteractiveLogin);
        match control {
            Control::CrawlField(10..=12)
            | Control::Guided(31..=34)
            | Control::Choice(Choice::SaveLogin | Choice::EphemeralLogin) => login,
            Control::CrawlField(13..=16)
            | Control::Guided(37 | 38)
            | Control::Choice(Choice::LightTheme | Choice::DarkTheme) => self.focus.advanced,
            _ => true,
        }
    }

    pub(super) fn setup_footer(&self, cx: &mut Context<Self>) -> AnyElement {
        let stage = self.focus.setup_stage;
        div()
            .flex_shrink_0()
            .flex()
            .flex_wrap()
            .items_center()
            .gap_3()
            .px_6()
            .py_3()
            .bg(rgb(SURFACE))
            .border_t_1()
            .border_color(rgb(LINE))
            .child(self.button(
                Control::Save,
                if stage == SetupStage::Website {
                    "Save website & continue"
                } else {
                    "Save settings"
                },
                ActionStyle::Primary,
                cx,
            ))
            .when(stage == SetupStage::Access, |body| {
                body.child(self.button(
                    Control::SetupStage(SetupStage::Pages),
                    "Continue to pages & sizes",
                    ActionStyle::Secondary,
                    cx,
                ))
            })
            .when(
                self.project.is_some() && matches!(stage, SetupStage::Pages | SetupStage::Settings),
                |body| body.child(self.guided_button(13, "Find pages & preview", cx)),
            )
            .child(
                div()
                    .flex_1()
                    .min_w(px(180.))
                    .text_size(px(13.))
                    .text_color(rgb(MUTED))
                    .child(if self.unsaved(cx) {
                        "Save before visiting the website. Existing baseline unchanged."
                    } else {
                        "Settings saved locally. Capture and baseline activation remain explicit."
                    }),
            )
            .into_any_element()
    }

    pub(super) fn project_rail(&self, cx: &mut Context<Self>) -> AnyElement {
        let project = self.project.is_some();
        div()
            .w(px(192.))
            .flex_shrink_0()
            .h_full()
            .p_3()
            .bg(rgb(RAIL))
            .text_color(rgb(RAIL_INK))
            .flex()
            .flex_col()
            .gap_2()
            .child(
                div()
                    .px_3()
                    .pt_3()
                    .pb_5()
                    .text_size(px(22.))
                    .font_weight(FontWeight::SEMIBOLD)
                    .child("Regressee"),
            )
            .child(self.button(
                Control::Navigate(Screen::Projects),
                "Projects",
                ActionStyle::Navigation {
                    selected: self.focus.screen == Screen::Projects,
                },
                cx,
            ))
            .when(project, |rail| {
                rail.child(
                    div()
                        .px_3()
                        .py_4()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .text_ellipsis()
                                .child(self.project.as_ref().unwrap().project.name.clone()),
                        )
                        .child(div().text_size(px(13.)).text_color(rgb(RAIL_MUTED)).child(
                            if let Some(baseline) = &self.evidence.baseline {
                                format!("{} reference screenshots", baseline.references.len())
                            } else {
                                "No saved baseline yet".into()
                            },
                        )),
                )
            })
            .child(self.button(
                Control::Navigate(Screen::Results),
                "Review changes",
                ActionStyle::Navigation {
                    selected: self.focus.screen == Screen::Results
                        && self.focus.review_surface == ReviewSurface::Changes,
                },
                cx,
            ))
            .when(project, |rail| {
                rail.child(self.button(
                    Control::PagesBaseline,
                    "Pages & baseline",
                    ActionStyle::Navigation {
                        selected: self.focus.screen == Screen::Results
                            && self.focus.review_surface == ReviewSurface::Pages,
                    },
                    cx,
                ))
            })
            .child(self.button(
                Control::Navigate(Screen::History),
                "History",
                ActionStyle::Navigation {
                    selected: self.focus.screen == Screen::History,
                },
                cx,
            ))
            .child(self.button(
                Control::Navigate(Screen::Setup),
                if project {
                    "Project settings"
                } else {
                    "Website setup"
                },
                ActionStyle::Navigation {
                    selected: self.focus.screen == Screen::Setup,
                },
                cx,
            ))
            .child(div().flex_1())
            .child(
                div()
                    .border_t_1()
                    .border_color(rgb(RAIL_HOVER))
                    .pt_3()
                    .child(self.button(
                        Control::Navigate(Screen::Settings),
                        "App settings",
                        ActionStyle::Navigation {
                            selected: self.focus.screen == Screen::Settings,
                        },
                        cx,
                    )),
            )
            .child(
                div()
                    .px_3()
                    .py_2()
                    .text_size(px(13.))
                    .text_color(rgb(RAIL_MUTED))
                    .child("Private, local preview"),
            )
            .into_any_element()
    }

    pub(super) fn coverage_summary(&self) -> AnyElement {
        let Some(run) = &self.evidence.latest else {
            return div().into_any_element();
        };
        let coverage = Coverage::from_run(run, &self.review_index());
        let profiles = run
            .run
            .profiles
            .iter()
            .map(|p| p.name.clone())
            .collect::<Vec<_>>()
            .join(" + ");
        let scope = match run.run.inventory.coverage {
            regressee_core::domain::DiscoveryCoverage::Complete => "bounded discovery",
            regressee_core::domain::DiscoveryCoverage::Limited { .. } => "limited inventory",
            regressee_core::domain::DiscoveryCoverage::Interrupted => "interrupted discovery",
        };
        div()
            .debug_selector(|| "real-coverage-summary".into())
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_1()
            .py_2()
            .border_y_1()
            .border_color(rgb(LINE))
            .text_size(px(14.))
            .line_height(px(20.))
            .child(div().font_weight(FontWeight::SEMIBOLD).child(format!(
                "{} targets · {} changed · {} unchanged · {} gaps · {} awaiting",
                coverage.total,
                coverage.changed,
                coverage.unchanged,
                coverage.missing,
                coverage.unclassified
            )))
            .child(div().text_color(rgb(MUTED)).text_ellipsis().child(format!(
                    "{} fixed pages · {profiles} · {scope}; gaps are not unchanged",
                    run.run
                        .inventory
                        .pages
                        .iter()
                        .filter(|p| p.included)
                        .count()
                )))
            .into_any_element()
    }
}

fn short_id(id: String) -> String {
    id.chars().take(8).collect()
}
fn capture_origin(address: &str) -> String {
    url::Url::parse(address)
        .map(|u| u.origin().ascii_serialization())
        .unwrap_or_else(|_| "Origin record invalid".into())
}

/// Exact UTC civil time, never a claim about the user's local timezone.
fn capture_time(timestamp: i64) -> String {
    let days = timestamp.div_euclid(86_400);
    let seconds = timestamp.rem_euclid(86_400);
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let mut year = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = mp + if mp < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02} {:02}:{:02}:{:02} UTC",
        seconds / 3600,
        seconds / 60 % 60,
        seconds % 60
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(feature = "ui-tests")]
    fn large_evidence() -> (ProjectRecord, ProjectEvidence) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let record = store.load_project(project).unwrap();
        let mut e = store.project_evidence(project).unwrap();
        let target = e.latest.as_ref().unwrap().run.targets[0].clone();
        let capture = e
            .latest
            .as_ref()
            .unwrap()
            .all_captures()
            .next()
            .unwrap()
            .clone();
        let result = e.comparison.as_ref().unwrap().revision.results[0].clone();
        let report = e
            .comparison
            .as_ref()
            .unwrap()
            .perceptual
            .as_ref()
            .unwrap()
            .reports[0]
            .1
            .clone();
        let reference = e.baseline.as_ref().unwrap().references[0].clone();
        let run = e.latest.as_mut().unwrap();
        run.run.targets.clear();
        run.capture = None;
        run.captures.clear();
        let comparison = e.comparison.as_mut().unwrap();
        comparison.revision.results.clear();
        comparison.perceptual.as_mut().unwrap().reports.clear();
        e.baseline.as_mut().unwrap().references.clear();
        for i in 0..3000 {
            let mut t = target.clone();
            t.key.page = regressee_core::domain::PageId::new();
            t.requested = url::Url::parse(&format!("https://example.test/page-{i}")).unwrap();
            if i % 5 == 2 {
                t.outcome = TargetOutcome::Failed {
                    code: regressee_core::domain::CaptureError::NeedsLogin,
                };
            }
            let mut c = capture.clone();
            c.metadata.key = t.key.clone();
            run.captures.push(c);
            let mut r = result.clone();
            r.key = t.key.clone();
            r.classification = match i % 5 {
                0 => Classification::Changed,
                1 => Classification::NoSignificantChange,
                _ => Classification::NotComparable,
            };
            if i % 5 != 4 {
                comparison.revision.results.push(r);
            }
            let mut p = report.clone();
            p.request.before.target.page = t.key.page.to_string();
            p.request.after.target.page = t.key.page.to_string();
            comparison
                .perceptual
                .as_mut()
                .unwrap()
                .reports
                .push((t.key.clone(), p));
            let mut reference = reference.clone();
            reference.key = t.key.clone();
            e.baseline.as_mut().unwrap().references.push(reference);
            run.run.targets.push(t);
        }
        (record, e)
    }

    #[cfg(feature = "ui-tests")]
    #[test]
    fn indexed_3000_target_coverage_is_exact_and_rejects_stale_revisions() {
        let (record, mut e) = large_evidence();
        let index = ReviewIndex::build(&e, record.project.active_baseline);
        assert_eq!(index.classifications.len(), 2400);
        assert_eq!(index.eligible.len(), 1200);
        assert_eq!(
            Coverage::from_run(e.latest.as_ref().unwrap(), &index),
            Coverage {
                total: 3000,
                changed: 600,
                unchanged: 600,
                missing: 1200,
                unclassified: 600
            }
        );
        let mut missing = e.clone();
        let key = missing.latest.as_ref().unwrap().run.targets[1].key.clone();
        missing
            .latest
            .as_mut()
            .unwrap()
            .captures
            .retain(|c| c.metadata.key != key);
        let missing_index = ReviewIndex::build(&missing, record.project.active_baseline);
        assert_eq!(
            Coverage::from_run(missing.latest.as_ref().unwrap(), &missing_index),
            Coverage {
                total: 3000,
                changed: 600,
                unchanged: 599,
                missing: 1201,
                unclassified: 600
            }
        );
        assert!(!missing_index.eligible.contains(&key));
        e.comparison.as_mut().unwrap().revision.run = regressee_core::domain::RunId::new();
        let stale = ReviewIndex::build(&e, record.project.active_baseline);
        assert!(stale.classifications.is_empty() && stale.eligible.is_empty());
        assert_eq!(
            Coverage::from_run(e.latest.as_ref().unwrap(), &stale),
            Coverage {
                total: 3000,
                changed: 0,
                unchanged: 0,
                missing: 600,
                unclassified: 2400
            }
        );
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn review_3000_target_cpu_element_construction_benchmark(cx: &mut gpui::TestAppContext) {
        let (record, evidence) = large_evidence();
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        let timings = shell.update(cx, |s, cx| {
            let update = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
            s.apply_update(update, cx);
            s.project = Some(record);
            s.evidence = evidence;
            s.evidence_dirty = false;
            s.focus.screen = Screen::Results;
            s.focus.project_saved = true;
            s.focus.baseline = true;
            s.selected_key = s
                .evidence
                .latest
                .as_ref()
                .unwrap()
                .run
                .targets
                .first()
                .map(|t| t.key.clone());
            s.approval_selection = s.review_index().eligible;
            assert!(s.approval_selection_eligible());
            s.configure_review_geometry(1280.);
            let mut timings = Vec::new();
            for _ in 0..12 {
                let start = std::time::Instant::now();
                let _workspace = s.results(cx);
                let _decision_bar = s.approval_bar(cx);
                timings.push(start.elapsed().as_secs_f64() * 1000.);
            }
            timings.sort_by(f64::total_cmp);
            timings
        });
        let report = format!(
            "3000 synthetic targets; 1200 eligible selected; 12 samples\nCPU Rust element construction only (indexes, coverage, sorting, controls). No layout, paint or GPU readback included.\nmin_ms={:.3}\nmedian_ms={:.3}\nmax_ms={:.3}\n",
            timings[0], timings[6], timings[11]
        );
        std::fs::write(
            std::env::temp_dir().join("regressee-review-cpu-construction.txt"),
            report,
        )
        .unwrap();
        owned.join_after_ui();
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn review_first_viewport_contains_complete_rows_and_bounded_images(
        cx: &mut gpui::TestAppContext,
    ) {
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, valid, _) = crate::approval_fixture::fixture(dir.path());
        let key = store
            .project_evidence(project)
            .unwrap()
            .latest
            .unwrap()
            .run
            .targets[0]
            .key
            .clone();
        let preview = store
            .approval_preview(valid, std::slice::from_ref(&key))
            .unwrap();
        drop(store);
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        shell.update(cx, |s, cx| {
            let update = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
            s.apply_update(update, cx);
            s.focus.screen = Screen::Results;
            s.queue(Work::Load(project), cx);
            for _ in 0..12 {
                if s.pending.is_some() {
                    let update = s.events.recv_timeout(Duration::from_secs(5)).unwrap();
                    s.apply_update(update, cx);
                }
            }
            assert!(s.viewer.is_some());
            let run = s.evidence.latest.as_mut().unwrap();
            let target = run.run.targets[0].clone();
            for i in 1..8 {
                let mut t = target.clone();
                t.key.page = regressee_core::domain::PageId::new();
                t.requested = url::Url::parse(&format!("http://127.0.0.1:43210/page-{i}")).unwrap();
                run.run.targets.push(t);
            }
            s.notice = None;
        });
        for (width, height) in [(1280., 800.), (820., 600.)] {
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                let _ = window.draw(cx);
            });
            let queue = cx.debug_bounds("review-queue").unwrap();
            let viewer = cx.debug_bounds("review-viewer").unwrap();
            let row = cx.debug_bounds("page-row-target-0").unwrap();
            let image = cx.debug_bounds("phase5-after").unwrap();
            let bar = cx.debug_bounds("approval-next-action").unwrap();
            assert!(
                queue.right() < viewer.left(),
                "{width}: queue must stay beside evidence"
            );
            assert_eq!(row.size.height, px(PAGE_ROW_HEIGHT));
            assert!(
                row.top() >= queue.top() && row.bottom() <= queue.bottom(),
                "{width}: one complete result must open in view"
            );
            assert!(
                image.size.height >= px(64.),
                "{width}: image region must remain usable: {:?}",
                image.size
            );
            assert!(image.bottom() <= viewer.bottom() && viewer.bottom() <= bar.top());
            assert!(bar.size.height <= px(96.));
        }
        shell.update(cx, |s, cx| {
            s.approval_selection.insert(key);
            s.approval = Some(preview);
            s.focus.approval_dialog = true;
            s.presentation_focus = Some(Control::Phase5(25));
            cx.notify();
        });
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let decision = cx.debug_bounds("approval-decision").unwrap();
        let impact = cx.debug_bounds("approval-impact-summary").unwrap();
        let map = cx.debug_bounds("approval-reference-list").unwrap();
        let provenance = cx.debug_bounds("approval-exact-provenance").unwrap();
        assert!(decision.top() <= impact.top() && provenance.bottom() <= decision.bottom());
        assert!(map.size.height > px(0.) && map.bottom() <= provenance.top());
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert!(s.handle(Control::Phase5(25), cx).is_focused(window));
                let requests = s.next_request;
                s.activate_control(Control::CheckChanges, window, cx);
                s.activate_control(Control::Phase5(23), window, cx);
                assert_eq!(requests, s.next_request);
                let handles = [
                    s.handle(Control::Guided(47), cx),
                    s.handle(Control::Phase5(25), cx),
                ];
                for _ in 0..4 {
                    window.focus_next();
                    assert!(handles.contains(&window.focused(cx).unwrap()));
                }
                s.guided(47, cx);
                assert!(!s.focus.approval_dialog);
                assert!(s.approval.is_some());
                s.mask_dirty = true;
                assert!(!s.approval_matches(s.approval.as_ref().unwrap()));
                s.invalidate_approval();
                assert!(s.approval.is_none());
            })
        });
        owned.join_after_ui();
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn compared_reference_context_never_substitutes_the_current_active_baseline(
        cx: &mut gpui::TestAppContext,
    ) {
        let (mut record, evidence) = large_evidence();
        let compared = evidence.comparison.as_ref().unwrap().revision.baseline;
        let environment = evidence
            .latest
            .as_ref()
            .unwrap()
            .run
            .environment
            .name
            .clone();
        let active = regressee_core::domain::BaselineId::new();
        let dir = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        assert_ne!(compared, active);
        assert!(
            ReviewIndex::build(&evidence, Some(active))
                .eligible
                .is_empty()
        );
        assert_eq!(record.project.active_baseline, Some(compared));
        record.project.active_baseline = Some(active);
        shell.update(cx, |s, _| {
            s.project = Some(record);
            s.evidence = evidence;
            s.selected_key = Some(
                s.evidence.latest.as_ref().unwrap().run.targets[0]
                    .key
                    .clone(),
            );
            let context = s.comparison_context();
            assert!(context.contains("Historical baseline"));
            assert!(context.contains(&short_id(compared.to_string())));
            assert!(!context.contains(&short_id(active.to_string())));
            assert!(context.contains("UTC") && context.contains(&environment));
            assert!(
                s.pane_caption(0)
                    .0
                    .contains(&short_id(compared.to_string()))
            );
        });
        owned.join_after_ui();
    }
    #[test]
    fn coverage_never_calls_missing_or_unclassified_captures_clean() {
        let mut c = Coverage::default();
        c.count(false, Some(Classification::NoSignificantChange));
        c.count(true, None);
        c.count(true, Some(Classification::NotComparable));
        c.count(true, Some(Classification::CouldNotCapture));
        c.count(true, Some(Classification::Changed));
        c.count(true, Some(Classification::NoSignificantChange));
        assert_eq!(
            c,
            Coverage {
                total: 6,
                changed: 1,
                unchanged: 1,
                missing: 3,
                unclassified: 1
            }
        );
    }
}
