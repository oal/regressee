// SPDX-License-Identifier: MPL-2.0
//! Project-scoped baseline, run history and deliberate storage cleanup.
use super::*;

impl Shell {
    pub(super) fn baseline_history(&self, cx: &mut Context<Self>) -> AnyElement {
        let mut body = div()
            .w_full()
            .max_w(px(1000.))
            .flex()
            .flex_col()
            .gap_6()
            .text_size(px(15.))
            .child(self.heading(
                "History",
                "Choose a saved reference or inspect a capture run. History stays on this computer until you explicitly delete it.",
            ));
        if self.project.is_none() {
            return body
                .child(controls::empty(
                    "Choose a project to see its history",
                    "Open a saved project from Projects. Its baseline versions, capture runs and storage will appear here.",
                ))
                .into_any_element();
        }

        let mut baselines = div()
            .debug_selector(|| "history-baselines-section".into())
            .w_full()
            .flex()
            .flex_col()
            .gap_3()
            .child(history_title("Baseline versions"))
            .child(history_help(
                "A baseline is the saved screenshot reference used for future checks. Capture runs below are the evidence it comes from.",
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(self.button(
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
                    )),
            );
        if let Some(b) = self.evidence.baselines.get(self.history_index) {
            let active = self
                .project
                .as_ref()
                .is_some_and(|p| p.project.active_baseline == Some(b.id));
            baselines = baselines
                .child(
                    div()
                        .w_full()
                        .border_t_1()
                        .border_b_1()
                        .border_color(rgb(LINE))
                        .py_3()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(
                            div()
                                .font_weight(FontWeight::SEMIBOLD)
                                .child(format!(
                                    "Selected baseline {} of {} — {}",
                                    self.history_index + 1,
                                    self.evidence.baselines.len(),
                                    if active { "Active reference" } else { "Retained version" },
                                )),
                        )
                        .child(format!(
                            "{} saved screenshots / {} intended screenshots",
                            b.references.len(),
                            b.intended_count,
                        ))
                        .child(history_help(format!(
                            "Saved {} ({})",
                            history_date(b.created_at),
                            relative_time(b.created_at),
                        )))
                        .child(history_help(if active {
                            "Future checks use this version's fixed page list and screen sizes."
                        } else {
                            "Use this version to make it the reference for future checks. This does not recapture the website or delete newer versions."
                        })),
                )
                .child(if b.parent.is_some() {
                    controls::status(
                        "Built from more than one capture time: approved screenshots replace selected references, while other references are inherited. This is not one simultaneous website snapshot.",
                        Tone::Warning,
                    )
                    .into_any_element()
                } else {
                    history_help("Saved without inheriting references from another baseline version.")
                        .into_any_element()
                });
        } else {
            baselines = baselines.child(controls::empty(
                "No saved baseline versions",
                "Open Pages & baseline to review pages, capture screenshots, then explicitly Save baseline. You can also inspect a baseline ZIP before importing it as a new project.",
            ));
        }
        baselines = baselines
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(self.button(
                        Control::Phase5(42),
                        "Use this baseline",
                        ActionStyle::Primary,
                        cx,
                    ))
                    .child(self.button(
                        Control::Phase6(2),
                        "Review baseline ZIP export…",
                        ActionStyle::Secondary,
                        cx,
                    ))
                    .child(self.button(
                        Control::Phase6(4),
                        "Review baseline deletion…",
                        ActionStyle::Danger,
                        cx,
                    ))
                    .child(self.button(
                        Control::Phase6(0),
                        "Inspect ZIP for import…",
                        ActionStyle::Secondary,
                        cx,
                    )),
            )
            .child(history_help(
                "Switching references is reversible while the version is retained. Permanent deletion is different: deleted evidence cannot be restored by choosing an older baseline.",
            ))
            .child(history_help(
                "ZIP sharing includes selected baseline evidence and project settings, never login sessions or cookies. Screenshots may contain private information; review the export preview before sharing.",
            ));
        body = body.child(baselines);

        let mut runs = div()
            .debug_selector(|| "history-runs-section".into())
            .w_full()
            .border_t_1()
            .border_color(rgb(LINE))
            .pt_5()
            .flex()
            .flex_col()
            .gap_3()
            .child(history_title("Capture & check runs"))
            .child(history_help(
                "Each run records a website capture, comparison check or preview. Keeping a run does not automatically make it a baseline.",
            ))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(self.button(
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
                    )),
            );
        if let Some(h) = &self.history
            && let Some(r) = h.runs.get(self.run_history_index)
        {
            runs = runs.child(
                div()
                    .w_full()
                    .border_t_1()
                    .border_b_1()
                    .border_color(rgb(LINE))
                    .py_3()
                    .flex()
                    .flex_col()
                    .gap_2()
                    .child(
                        div()
                            .font_weight(FontWeight::SEMIBOLD)
                            .child(format!(
                                "Selected run {} of {} on this history page — {}",
                                self.run_history_index + 1,
                                h.runs.len(),
                                match r.run.intent {
                                    RunIntent::NewBaseline => "Baseline capture",
                                    RunIntent::Check { .. } => "Comparison check",
                                    RunIntent::RepresentativePreview => "Representative preview",
                                },
                            )),
                    )
                    .child(format!(
                        "{} · {} captured screenshots",
                        job_label(r.run.state),
                        r.all_captures().count(),
                    ))
                    .child(history_help(format!(
                        "Started {} · Finished {}",
                        r.run.started_at.map(history_date).unwrap_or_else(|| "not recorded".into()),
                        r.run.finished_at.map(history_date).unwrap_or_else(|| "not recorded".into()),
                    )))
                    .child(history_help(if h.imported_runs.contains(&r.run.id) {
                        "Imported evidence — not independently verified on this computer. Imported ignored areas need revalidation."
                    } else {
                        "Recorded locally on this computer."
                    })),
            );
        } else {
            runs = runs.child(controls::empty(
                "No capture runs on this history page",
                "Open Pages & baseline to capture reviewed pages, or Review changes to run a check against a saved baseline. Refresh history to load the latest recorded evidence.",
            ));
        }
        runs = runs
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(self.button(
                        Control::Phase6(12),
                        "Previous history page",
                        ActionStyle::Secondary,
                        cx,
                    ))
                    .child(self.button(
                        Control::Phase6(11),
                        "Next history page",
                        ActionStyle::Secondary,
                        cx,
                    ))
                    .child(self.button(
                        Control::Phase6(5),
                        "Review run deletion…",
                        ActionStyle::Danger,
                        cx,
                    )),
            )
            .child(history_help(format!(
                "History page {}{}. Previous / next run moves within this page; previous / next history page loads another set of runs.",
                self.history.as_ref().map(|h| h.page + 1).unwrap_or(1),
                if self.history.as_ref().is_some_and(|h| h.more) { " · more runs available" } else { "" },
            )))
            .child(history_help(
                "Retained baselines protect their source runs. Review deletion impact before confirming; failed captures are not unchanged results.",
            ));
        body = body.child(runs);

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
            .border_color(rgb(LINE))
            .pt_5()
            .flex()
            .flex_col()
            .gap_3()
            .child(history_title("Storage & manual cleanup"))
            .child(history_help(
                "Nothing is deleted automatically. Choose a baseline or run above to review exactly what its deletion would remove.",
            ));
        if let Some(s) = &self.project_storage {
            for (label, bytes) in [
                ("Project evidence total", s.bytes),
                ("Original screenshots", s.original_bytes),
                ("Rebuildable derivatives", s.derived_bytes),
                ("Exclusive evidence", s.exclusive_bytes),
                ("Shared evidence", s.shared_bytes),
            ] {
                storage = storage.child(history_storage_row(label, history_bytes(bytes)));
            }
            storage = storage.child(history_storage_row(
                "Free disk space",
                s.available_bytes.map(history_bytes).unwrap_or_else(|| {
                    "Unavailable — free-space preflight cannot be claimed".into()
                }),
            ));
        } else {
            storage = storage.child(history_help(
                "Storage totals are not loaded. Refresh storage & history to request measured totals; unavailable is not zero.",
            ));
        }
        body = body.child(
            storage
                .child(history_help(
                    "Shared originals stay protected. Exclusive storage is not automatically deletable. Replacing a whole baseline requires a new capture and explicit save; browser/cache repair is separate in App settings.",
                ))
                .child(self.button(
                    Control::Phase6(8),
                    "Refresh storage & history",
                    ActionStyle::Secondary,
                    cx,
                )),
        );

        let mut provenance = div()
            .w_full()
            .border_t_1()
            .border_color(rgb(LINE))
            .pt_5()
            .flex()
            .flex_col()
            .gap_3()
            .child(self.button(
                Control::Details(Detail::History),
                if self.details.contains(&Detail::History) {
                    "Hide advanced provenance"
                } else {
                    "Show advanced provenance"
                },
                ActionStyle::Secondary,
                cx,
            ));
        if self.details.contains(&Detail::History) {
            provenance = provenance.child(history_help(
                "Technical source records for the selected baseline and run. Identifiers below are retained evidence links, not website names.",
            ));
            if let Some(b) = self.evidence.baselines.get(self.history_index) {
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
            }
            if let Some(h) = &self.history
                && let Some(r) = h.runs.get(self.run_history_index)
            {
                provenance = provenance
                    .child(history_help(format!("Run {}", r.run.id)))
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
                    "Showing {} of {} captured screenshots. Complete evidence and the fixed inventory remain retained.",
                    r.all_captures().count().min(32),
                    r.all_captures().count(),
                )));
            }
        }
        body.child(provenance).into_any_element()
    }
}

fn history_title(title: &'static str) -> Div {
    div()
        .text_size(px(21.))
        .font_weight(FontWeight::SEMIBOLD)
        .child(title)
}

fn history_help(text: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(14.))
        .text_color(rgb(MUTED))
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
        .border_color(rgb(LINE))
        .pb_2()
        .child(label)
        .child(value)
}

fn history_bytes(bytes: u64) -> String {
    format!("{} ({bytes} bytes)", human_bytes(bytes))
}

/// An exact UTC civil timestamp, without a new dependency or local-time claim.
/// Stored timestamps are Unix seconds, like `relative_time` in the shell.
fn history_date(timestamp: i64) -> String {
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
    fn storage_values_keep_exact_bytes() {
        assert!(history_bytes(0).ends_with("(0 bytes)"));
        assert!(history_bytes(1_048_577).ends_with("(1048577 bytes)"));
    }
}
