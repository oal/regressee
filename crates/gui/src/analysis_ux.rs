// SPDX-License-Identifier: MPL-2.0
//! Progressive analysis presentation; drafts and mutations keep their incumbent
//! exact target/capture-pair and revision owners in Shell.
use super::*;

pub(super) fn analysis_help(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(HELP_TEXT_SIZE))
        .line_height(px(HELP_LINE_HEIGHT))
        .text_color(rgb(palette().muted))
        .child(text.into())
}

pub(super) fn policy_name(policy: &regressee_app::analysis_engine::Policy) -> &'static str {
    use regressee_app::analysis_engine::Preset;
    for (preset, name) in [
        (Preset::Sensitive, "Sensitive"),
        (Preset::Balanced, "Balanced"),
        (Preset::Relaxed, "Relaxed"),
    ] {
        if policy == &regressee_app::review::preset(preset) {
            return name;
        }
    }
    "Custom"
}

impl Shell {
    pub(super) fn record_review_field_bounds(&self, index: usize) -> impl IntoElement + use<> {
        let bounds = self.review_field_bounds[index].clone();
        canvas(move |paint, _, _| bounds.set(Some(paint)), |_, _, _, _| {})
            .absolute()
            .size_full()
    }
    pub(super) fn analysis_summary(&self) -> AnyElement {
        let mut summary = div()
            .debug_selector(|| "analysis-current-summary".into())
            .flex()
            .flex_col()
            .gap_1();
        let Some(a) = &self.evidence.comparison else {
            return summary.into_any_element();
        };
        let report = a.perceptual.as_ref().and_then(|p| {
            p.reports
                .iter()
                .find(|(key, _)| Some(key) == self.selected_key.as_ref())
                .map(|(_, r)| r)
        });
        let classification = self
            .selected_key
            .as_ref()
            .and_then(|key| a.review_classification(key));
        let issue = self.comparison_issue();
        let (label, help) = match classification {
            Some(Classification::Changed) => (
                "Changed",
                "The difference exceeds your sensitivity settings. Review the screenshots to decide whether it needs fixing.",
            ),
            Some(Classification::NoSignificantChange) => (
                "No significant change",
                "Differences are below your sensitivity settings. The screenshots may not be identical.",
            ),
            Some(Classification::CouldNotCapture) => (
                issue.as_ref().map_or("Could not capture", |i| i.heading),
                issue.as_ref().map_or(
                    "A missing screenshot does not mean the page is unchanged.",
                    |i| i.reason.as_str(),
                ),
            ),
            _ => (
                issue
                    .as_ref()
                    .map_or("Comparison unavailable", |i| i.heading),
                issue
                    .as_ref()
                    .map_or("No reason was recorded for this result.", |i| {
                        i.reason.as_str()
                    }),
            ),
        };
        summary = summary
            .child(
                div()
                    .font_weight(FontWeight::SEMIBOLD)
                    .child(format!("Displayed result · {label}")),
            )
            .child(analysis_help(help.to_owned()));
        if let Some(issue) = &issue {
            summary = summary.child(analysis_help(issue.recovery));
        } else if let Some(notice) = self.capture_notice() {
            summary = summary.child(analysis_help(notice.summary));
        }
        let displayed = a
            .perceptual
            .as_ref()
            .map(|p| policy_name(&p.policy))
            .unwrap_or("Earlier color-comparison method");
        let settings = a
            .perceptual
            .as_ref()
            .is_some_and(|p| p.policy == self.analysis_policy);
        summary = summary.child(analysis_help(if settings {
            format!("{displayed} sensitivity · saved result")
        } else {
            format!(
                "Displayed: {displayed} · current settings: {}",
                policy_name(&self.analysis_policy)
            )
        }));
        if let Some(r) = report {
            let limits = r
                .gates
                .iter()
                .map(analysis_gate_label)
                .chain(r.warnings.iter().map(analysis_warning_label))
                .collect::<Vec<_>>();
            if !limits.is_empty() && issue.is_none() {
                summary = summary.child(
                    div()
                        .text_size(px(HELP_TEXT_SIZE))
                        .line_height(px(HELP_LINE_HEIGHT))
                        .text_color(rgb(palette().warning))
                        .child(limits.join("; ")),
                );
            }
            if !r.triggers.is_empty() && issue.is_none() {
                summary = summary.child(analysis_help(format!(
                    "Reason: {}",
                    r.triggers
                        .iter()
                        .map(analysis_trigger_label)
                        .collect::<Vec<_>>()
                        .join(", ")
                )));
            }
        }
        if let Some((_, failure)) = a
            .perceptual
            .as_ref()
            .and_then(|p| {
                p.failures
                    .iter()
                    .find(|(key, _)| Some(key) == self.selected_key.as_ref())
            })
            .filter(|_| issue.is_none())
        {
            summary = summary.child(controls::error(failure.clone()));
        }
        summary.into_any_element()
    }

    pub(super) fn ignored_area_status(&self, cx: &App) -> String {
        let count = self.mask_rectangles.len();
        let state = if self.mask_geometry_dirty(cx) {
            "unapplied coordinates · Apply, then Save"
        } else if self.mask_dirty {
            "unsaved draft"
        } else if count == 0 {
            "no ignored areas saved"
        } else {
            "saved areas"
        };
        format!(
            "{count} area{} · {state}",
            if count == 1 { "" } else { "s" }
        )
    }

    pub(super) fn ignored_areas_disclosure(&self, cx: &mut Context<Self>) -> AnyElement {
        let open = self.details.contains(&Detail::IgnoredAreas);
        let title = format!(
            "Ignored areas · {}{}",
            self.mask_rectangles.len(),
            if self.mask_dirty || self.mask_geometry_dirty(cx) {
                " · draft"
            } else {
                ""
            }
        );
        let mut group = div()
            .debug_selector(|| "analysis-ignored-areas".into())
            .flex()
            .flex_col()
            .gap_2();
        if !open {
            return self
                .inline_disclosure(
                    Control::Details(Detail::IgnoredAreas),
                    title,
                    "",
                    open,
                    group,
                    cx,
                )
                .into_any_element();
        }
        group = group.child(self.keyboard_mask_editor(cx))
            .child(analysis_help("Areas stay at fixed pixel positions in these before-and-after screenshots; they do not follow moving content. Apply updates your draft. Save keeps the areas and recalculates the comparison. Ignored content remains visible in the original images."));
        let mut actions = div().flex().flex_wrap().gap_2();
        for (i, label) in [
            (15, "Select next rectangle"),
            (17, "Redraw selected rectangle"),
            (16, "Delete selected rectangle"),
            (19, "Check areas fit these screenshots"),
        ] {
            if self.control_mounted(Control::Phase5(i)) {
                actions = actions.child(self.button(
                    Control::Phase5(i),
                    label,
                    if i == 16 {
                        ActionStyle::Danger
                    } else {
                        ActionStyle::Secondary
                    },
                    cx,
                ));
            }
        }
        group = group.child(actions).child(
            analysis_help(self.ignored_area_status(cx))
                .debug_selector(|| "analysis-ignored-status".into()),
        );
        if self
            .viewer
            .as_ref()
            .and_then(|v| v.request.as_ref())
            .is_some_and(|r| r.validate().is_err())
        {
            group = group.child(controls::status("Saved areas came from different screenshots or image sizes. Review their positions, then choose Check areas fit these screenshots. Areas are never stretched automatically.", Tone::Warning));
        }
        self.inline_disclosure(
            Control::Details(Detail::IgnoredAreas),
            title,
            "",
            open,
            group,
            cx,
        )
        .into_any_element()
    }
}
