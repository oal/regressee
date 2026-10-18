// SPDX-License-Identifier: MPL-2.0
//! Bounded, task-first disclosures. No capture, storage or network work here.
use super::*;

#[derive(Clone, PartialEq)]
pub(super) struct EnvironmentDraftSignature {
    pub project: Option<regressee_core::domain::ProjectId>,
    environment: Option<regressee_core::domain::EnvironmentId>,
    ephemeral: bool,
    fields: [String; 3],
    environments: Vec<regressee_core::domain::Environment>,
}

fn group(title: &'static str) -> Div {
    div()
        .w_full()
        .flex_shrink_0()
        .flex()
        .flex_col()
        .gap_2()
        .child(
            div()
                .text_size(px(14.))
                .line_height(px(20.))
                .font_weight(FontWeight::SEMIBOLD)
                .child(title),
        )
}

fn help(copy: impl Into<SharedString>) -> Div {
    div()
        .text_size(px(13.))
        .line_height(px(18.))
        .text_color(rgb(palette().muted))
        .child(copy.into())
}

impl Shell {
    pub(super) fn discovery_plan_current(&self) -> bool {
        self.preview
            .as_ref()
            .zip(self.project.as_ref())
            .is_some_and(|(preview, project)| {
                preview.project_revision == project.revision
                    && preview.settings == project.project
                    && preview.included() > 0
            })
    }
    pub(super) fn environment_draft_signature(&self, cx: &App) -> EnvironmentDraftSignature {
        EnvironmentDraftSignature {
            project: self.project.as_ref().map(|p| p.project.id),
            environment: self.environment_draft,
            ephemeral: self.login_ephemeral,
            fields: std::array::from_fn(|i| self.crawl_fields[10 + i].read(cx).content.to_string()),
            environments: self.draft_settings.environments.clone(),
        }
    }
    pub(super) fn options_panel_open(&self) -> bool {
        self.result_filters_open
            && (self.page_list_inventory().is_some()
                || (!self.page_list_workspace()
                    && !self.review_queue_collapsed
                    && self.evidence.latest.is_some()))
    }

    pub(super) fn options_panel_width(&self) -> f32 {
        (self.working_width - if self.narrow_workspace { 200. } else { 272. } - 12.)
            .clamp(1., if self.narrow_workspace { 320. } else { 360. })
    }

    fn options_control_index(control: Control) -> Option<usize> {
        Some(match control {
            Control::Guided(index @ 20..=22) => index - 20,
            Control::DiscoveryDetails => 3,
            Control::Choice(choice) => 4 + Choice::RESULTS.iter().position(|c| *c == choice)?,
            Control::Guided(19) => 8,
            Control::CloseMappingPreview => 9,
            Control::Guided(35) => 10,
            Control::Guided(36) => 11,
            _ => return None,
        })
    }

    pub(super) fn options_control_bounds(
        &self,
        control: Control,
    ) -> Option<std::rc::Rc<std::cell::Cell<Option<Bounds<Pixels>>>>> {
        self.options_bounds
            .get(Self::options_control_index(control)?)
            .cloned()
    }

    pub(super) fn options_anchor(&self, control: Control) -> Option<gpui::ScrollAnchor> {
        let index = Self::options_control_index(control)?;
        self.options_anchors.get(index).cloned()
    }

    pub(super) fn discovery_coverage_hint(&self) -> &'static str {
        use regressee_core::domain::DiscoveryCoverage;
        match self.preview.as_ref().map(|p| &p.inventory.coverage) {
            Some(DiscoveryCoverage::Interrupted) => "Page search interrupted · incomplete list",
            Some(DiscoveryCoverage::Limited { .. }) => {
                "Limited coverage · some pages may be missing"
            }
            _ => "Visible links explored · other pages may be missing",
        }
    }

    pub(super) fn options_panel(&self, cx: &mut Context<Self>) -> AnyElement {
        let discovery = self.preview.is_some();
        let recorded = !discovery && self.page_list_workspace();
        let body = if discovery {
            self.discovery_options(cx)
        } else if recorded {
            self.recorded_page_details()
        } else {
            div()
                .flex()
                .flex_col()
                .gap_4()
                .flex_shrink_0()
                .child(help(
                    "Filter the list without changing screenshots or review drafts.",
                ))
                .child(self.result_filter_options(cx))
                .into_any_element()
        };
        let panel = controls::disclosure_panel(
            "page-options-panel",
            if discovery {
                "Page options"
            } else if recorded {
                "Page details"
            } else {
                "Result filters"
            },
            self.button(Control::CloseOptions, "Close", ActionStyle::Secondary, cx),
            body,
            &self.options_scroll,
        );
        panel
            .absolute()
            .top_0()
            .bottom_0()
            .right_0()
            .w(px(self.options_panel_width()))
            .occlude()
            .when(discovery, |panel| {
                panel.child(
                    div()
                        .flex_shrink_0()
                        .p_3()
                        .border_t_1()
                        .border_color(rgb(palette().line))
                        .flex()
                        .flex_col()
                        .gap_2()
                        .child(help("Discard this unsaved list and its inclusion choices. Saved screenshots and the baseline are kept."))
                        .child(self.button(Control::DiscardPageList, "Discard discovered pages…", ActionStyle::Danger, cx)),
                )
            })
            .into_any_element()
    }

    fn recorded_page_details(&self) -> AnyElement {
        let inventory = self.page_list_inventory().unwrap();
        let selected = inventory
            .pages
            .iter()
            .find(|p| Some(p.id) == self.selected_page);
        let mut selected_group = group("Selected page");
        if let Some(page) = selected {
            selected_group = selected_group
                .child(
                    div()
                        .debug_selector(|| "recorded-selected-address".into())
                        .w_full()
                        .min_w_0()
                        .text_size(px(14.))
                        .line_height(px(20.))
                        .child(page.canonical.to_string()),
                )
                .child(help(if page.included {
                    "Included in this recorded capture list"
                } else {
                    "Excluded from this recorded capture list"
                }));
            if let Some(reason) = &page.reason {
                selected_group = selected_group.child(help(reason.clone()));
            }
        } else {
            selected_group =
                selected_group.child(help("Select a page to view its recorded details."));
        }
        div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_4()
            .child(selected_group)
            .child(help(
                "Recorded list · read-only. Saved screenshots and the baseline are unchanged.",
            ))
            .child(group("How complete is this page list?").child(help(inventory.coverage.label())))
            .into_any_element()
    }

    fn discovery_options(&self, cx: &mut Context<Self>) -> AnyElement {
        let p = self.preview.as_ref().unwrap();
        let selected = p
            .inventory
            .pages
            .iter()
            .find(|p| Some(p.id) == self.selected_page);
        let matches = self.preview_page_ids().len();
        let targets = p.included() * p.settings.profiles.len();
        let mut selected_group =
            group("Selected page").debug_selector(|| "discovery-selected-page".into());
        if let Some(page) = selected {
            selected_group = selected_group
                .child(
                    div()
                        .debug_selector(|| "discovery-selected-address".into())
                        .w_full()
                        .min_w_0()
                        .text_size(px(14.))
                        .line_height(px(20.))
                        .child(page.canonical.to_string()),
                )
                .child(help(if page.included {
                    "Included in capture"
                } else {
                    "Excluded from capture"
                }));
            if !page.included
                && let Some(reason) = &page.reason
            {
                selected_group = selected_group.child(help(reason.clone()));
            }
            if !page.included && page.reason.as_deref() != Some("Excluded in preview") {
                selected_group = selected_group.child(
                    help("This page can't be included here. Review project settings and find pages again.")
                        .debug_selector(|| "discovery-inclusion-blocked-help".into()),
                );
            }
        } else {
            selected_group =
                selected_group.child(help("Select a page in the list to change its inclusion."));
        }
        selected_group = selected_group.child(self.guided_button(
            22,
            if selected.is_some_and(|p| p.included) {
                "Exclude page"
            } else {
                "Include page"
            },
            cx,
        ));
        let matches_group = group("Search matches")
            .border_t_1()
            .border_color(rgb(palette().line))
            .pt_4()
            .child(help(if self.search.is_empty() {
                format!(
                    "{} · no search filter",
                    workflow::count_label(matches, "page", "pages")
                )
            } else {
                workflow::count_label(matches, "matching page", "matching pages")
            }))
            .child(
                div()
                    .flex()
                    .flex_wrap()
                    .gap_2()
                    .child(self.guided_button(20, "Exclude matches", cx))
                    .child(self.guided_button(21, "Restore matches", cx)),
            )
            .child(help(
                "Restore only pages you excluded here. Search alone doesn't change inclusion.",
            ));
        let estimate = group("Estimated capture")
            .border_t_1()
            .border_color(rgb(palette().line))
            .pt_4()
            .child(help(workflow::count_label(
                targets,
                "screenshot",
                "screenshots",
            )))
            .children(
                [
                    ("Disk space", format!("{}–{} MiB", targets / 2, targets * 8)),
                    (
                        "Time",
                        format!("{}–{} minutes", targets / 60 + 1, targets / 6 + 1),
                    ),
                ]
                .into_iter()
                .map(|(label, value)| {
                    div()
                        .flex()
                        .justify_between()
                        .gap_3()
                        .child(
                            div()
                                .text_size(px(14.))
                                .line_height(px(CONTROL_LINE_HEIGHT))
                                .child(label),
                        )
                        .child(
                            div()
                                .text_size(px(14.))
                                .line_height(px(CONTROL_LINE_HEIGHT))
                                .font_weight(FontWeight::MEDIUM)
                                .child(value),
                        )
                }),
            )
            .child(help(
                "Estimates only. Page height and website speed can increase them.",
            ));
        let body = div()
            .flex_shrink_0()
            .flex()
            .flex_col()
            .gap_4()
            .child(selected_group)
            .when(!self.discovery_plan_current(), |body| {
                body.child(help(if p.included() == 0 {
                    "Include at least one eligible page before capturing."
                } else {
                    "Settings changed since this list was found. In Project settings, choose Find pages again before capturing. Your current inclusion choices are kept until you confirm replacing this list."
                }))
            })
            .child(matches_group)
            .child(estimate);
        let details = if self.discovery_details_open {
            group("How complete is this page list?")
                .border_t_0()
                .pt_0()
                .debug_selector(|| "discovery-coverage-details".into())
                .child(help(format!(
                    "{} · {} listed addresses",
                    workflow::count_label(p.visited as usize, "page visit", "page visits"),
                    p.inventory.pages.len()
                )))
                .child(help(p.inventory.coverage.label()))
                .children(p.notes.iter().map(|note| help(note.clone())))
        } else {
            div()
        };
        body.child(self.inline_disclosure(
            Control::DiscoveryDetails,
            "Page search details",
            "",
            self.discovery_details_open,
            details,
            cx,
        ))
        .into_any_element()
    }

    pub(super) fn mapping_preview(&self, cx: &mut Context<Self>) -> AnyElement {
        let e = self.environment.as_ref().unwrap();
        let mut body = div().w_full().flex_shrink_0().flex().flex_col().gap_4();
        if e.mapped.is_empty() {
            body = body.child(help(
                "No page mappings available. Find pages or save a baseline, then preview again.",
            ).debug_selector(|| "mapping-preview-empty".into()));
        } else {
            body = body.children(e.mapped.iter().take(16).enumerate().map(
                |(i, (source, destination))| {
                    div()
                        .debug_selector(move || format!("mapping-preview-row-{i}"))
                        .flex_shrink_0()
                        .flex()
                        .flex_col()
                        .gap_2()
                        .border_t_1()
                        .border_color(rgb(palette().line))
                        .pt_3()
                        .child(
                            group("Source page")
                                .gap_1()
                                .child(help(source.clone()).text_color(rgb(palette().ink))),
                        )
                        .child(
                            group("Capture address")
                                .gap_1()
                                .child(help(destination.clone()).text_color(rgb(palette().ink))),
                        )
                },
            ));
        }
        body = body.child(group("Preview scope")
            .debug_selector(|| "mapping-preview-scope".into())
            .child(help(format!("{} · showing first {}", workflow::count_label(e.mapped.len(), "page mapping", "page mappings"), e.mapped.len().min(16))))
            .child(help("Uses saved environment settings. The access check visits at most 3 sample pages, not the whole website.")))
            .child(controls::status(if e.probed { "Access confirmed for sample pages" } else { "Access not yet confirmed for sample pages" }, Tone::Neutral));
        controls::disclosure_panel(
            "mapping-preview",
            "Mapped URL preview",
            self.button(
                Control::CloseMappingPreview,
                "Close",
                ActionStyle::Secondary,
                cx,
            ),
            body.into_any_element(),
            &self.mapping_preview_scroll,
        )
        .h(px(280.))
        .flex_shrink_0()
        .into_any_element()
    }
}

#[cfg(all(test, feature = "ui-tests"))]
pub(super) fn fixture_discovery(s: &mut Shell) {
    use regressee_core::domain::{DiscoveryCoverage, InventoryPage};
    let record = s.project.as_ref().unwrap();
    let mut inventory = s.evidence.latest.as_ref().unwrap().run.inventory.clone();
    inventory.pages = (0..128)
        .map(|i| InventoryPage {
            id: regressee_core::domain::PageId::new(),
            canonical: format!("https://example.test/pages/{i}/").parse().unwrap(),
            included: i < 75,
            reason: (i >= 75).then(|| "Excluded by fixture discovery boundary".into()),
        })
        .collect();
    inventory.coverage = DiscoveryCoverage::Limited { reason: "Some candidates were excluded, unavailable, cancelled or bounded. This is not proof of all website pages.".into() };
    s.selected_page = Some(inventory.pages[0].id);
    s.preview = Some(DiscoveryPreview {
        project_revision: record.revision,
        settings: record.project.clone(),
        inventory,
        aliases: vec![],
        notes: vec!["Only rendered links sampled after DOM readiness are explored. Delayed links, hidden application states and nested/virtualized content may contain additional pages. Queue completion is not an exhaustiveness guarantee.".into()],
        visited: 75,
    });
    s.focus.preview = true;
    s.focus.review_surface = ReviewSurface::Pages;
}
