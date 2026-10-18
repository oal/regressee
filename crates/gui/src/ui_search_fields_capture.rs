// SPDX-License-Identifier: MPL-2.0
//! One bounded search-field inspection in both palettes at supported sizes.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

#[gpui::test]
#[ignore = "explicit three-size search and Filter focus-ring inspection"]
fn renderer_capture_queue_focus(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("queue-focus-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), false);
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        for appearance in [AppAppearance::Light, AppAppearance::Dark] {
            regressee_storage::AppPreferences {
                appearance,
                ..Default::default()
            }
            .save(&AppPaths::isolated(dir.path()))
            .unwrap();
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owned = jobs.clone();
            owned.appearance.wait_loaded();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            shell.update(cx, |s, cx| {
                load_review(s, project, cx);
                s.notice = None;
                s.error = None;
                s.navigation_preference = Some(false);
                s.configure_review_geometry(width);
                cx.notify();
            });
            cx.run_until_parked();
            for (scene, selector) in [("search", "text-input"), ("filter", "action-Guided(45)")] {
                let before = cx.debug_bounds(selector).unwrap();
                cx.update(|w, cx| {
                    shell.update(cx, |s, cx| {
                        if scene == "filter" {
                            s.choose(Choice::AllResults, cx);
                            w.focus(&s.handle(Control::Guided(45), cx));
                        } else {
                            w.focus(&s.crawl_fields[9].read(cx).focus);
                        }
                        cx.notify();
                    });
                });
                cx.run_until_parked();
                // Changing the queue filter can change its heading height, but
                // must never move controls horizontally or resize their targets.
                let bounds = cx.debug_bounds(selector).unwrap();
                assert_eq!(bounds.left(), before.left());
                assert_eq!(bounds.size, before.size);
                capture(
                    cx,
                    &root,
                    &format!(
                        "queue-focus-{}-{width:.0}x{height:.0}-{scene}",
                        appearance.label().to_lowercase()
                    ),
                )
                .unwrap();
                assert_complete_ring(cx, bounds);
            }
            shell.update(cx, finish_jobs);
            owned.join_after_ui();
            cx.update(|w, _| w.remove_window());
        }
    }
    set_dark(false);
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

fn assert_complete_ring(cx: &mut VisualTestContext, bounds: Bounds<Pixels>) {
    let frame = cx.update(|w, cx| {
        let _ = w.draw(cx);
        w.capture_offscreen_frame().unwrap()
    });
    let left = f32::from(bounds.left()) as usize;
    let right = f32::from(bounds.right()) as usize;
    let top = f32::from(bounds.top()) as usize;
    let bottom = f32::from(bounds.bottom()) as usize;
    let x = f32::from(bounds.center().x) as usize;
    let y = f32::from(bounds.center().y) as usize;
    for (offset, color) in [
        (1, palette().surface),
        (2, palette().accent),
        (3, palette().accent),
    ] {
        for (x, y) in [
            (left - offset, y),
            (right + offset - 1, y),
            (x, top - offset),
            (x, bottom + offset - 1),
        ] {
            let index = (y * frame.width as usize + x) * 4;
            assert_eq!(
                &frame.rgba[index..index + 4],
                &[(color >> 16) as u8, (color >> 8) as u8, color as u8, 255],
                "complete focus ring at ({x}, {y}) around {bounds:?}"
            );
        }
    }
}

struct TimezoneInspection(Entity<TimezoneSelect>);
impl Render for TimezoneInspection {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p(px(24.))
            .bg(rgb(palette().ground))
            .child(
                div()
                    .w(px(320.))
                    .text_color(rgb(palette().ink))
                    .font_family(FONT)
                    .child(controls::field(
                        "Capture timezone",
                        "Choose a timezone for future captures.",
                        self.0.clone().into_any_element(),
                    )),
            )
    }
}

#[gpui::test]
#[ignore = "explicit bounded native search-field inspection"]
fn renderer_capture_search_fields(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    crate::image_controls_tests::bind(cx);
    let dir = tempfile::Builder::new()
        .prefix("search-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _) = ui_review_matrix::fixture_named(&root, dir.path(), false);
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = vec![];
    for (width, height) in [(1440., 900.), (1280., 800.), (820., 600.)] {
        for appearance in [AppAppearance::Light, AppAppearance::Dark] {
            for scene in ["review", "pages", "history"] {
                regressee_storage::AppPreferences {
                    appearance,
                    ..Default::default()
                }
                .save(&AppPaths::isolated(dir.path()))
                .unwrap();
                let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
                let owned = jobs.clone();
                owned.appearance.wait_loaded();
                let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
                cx.simulate_resize(size(px(width), px(height)));
                shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.notice = None;
                    s.error = None;
                    s.navigation_preference = Some(false);
                    s.configure_review_geometry(width);
                    if scene == "history" {
                        s.focus.screen = Screen::History;
                        s.queue(Work::History { project, page: 0 }, cx);
                        finish_jobs(s, cx);
                    } else if scene == "pages" {
                        let inventory = s.evidence.latest.as_ref().unwrap().run.inventory.clone();
                        let record = s.project.as_ref().unwrap();
                        s.selected_page = Some(inventory.pages[0].id);
                        s.preview = Some(DiscoveryPreview {
                            project_revision: record.revision,
                            settings: record.project.clone(),
                            inventory,
                            aliases: vec![],
                            notes: vec![],
                            visited: 2,
                        });
                        s.focus.preview = true;
                        s.focus.candidate = false;
                        s.focus.review_surface = ReviewSurface::Pages;
                    }
                    cx.notify();
                });
                cx.run_until_parked();
                cx.update(|w, cx| {
                    w.blur();
                    let _ = w.draw(cx);
                });
                let label = format!(
                    "search-{}-{width:.0}x{height:.0}-{scene}",
                    appearance.label().to_lowercase()
                );
                capture(cx, &root, &label).unwrap();
                let row = cx.debug_bounds("search-field-row").unwrap();
                manifest.push(serde_json::json!({"label":label,"row":[f32::from(row.left()), f32::from(row.top()), f32::from(row.size.width), f32::from(row.size.height)]}));
                if width == 820. && scene != "pages" {
                    cx.update(|w, cx| {
                        shell.update(cx, |s, cx| {
                            let input = if scene == "history" {
                                &s.history_search
                            } else {
                                &s.crawl_fields[9]
                            };
                            input.update(cx, |f, cx| f.set_value("no-matching-page-or-run", cx));
                            w.focus(&input.read(cx).focus);
                            cx.notify();
                        })
                    });
                    cx.run_until_parked();
                    capture(cx, &root, &format!("{label}-focused-query")).unwrap();
                }
                shell.update(cx, finish_jobs);
                owned.join_after_ui();
                cx.update(|w, _| w.remove_window());
            }
            set_dark(appearance == AppAppearance::Dark);
            let (view, cx) = cx.add_window_view(|_, cx| {
                TimezoneInspection(cx.new(|cx| TimezoneSelect::new("UTC", cx)))
            });
            cx.simulate_resize(size(px(width), px(height)));
            cx.run_until_parked();
            let trigger = cx.debug_bounds("timezone-select-button").unwrap();
            cx.simulate_click(trigger.center(), gpui::Modifiers::none());
            cx.run_until_parked();
            capture(
                cx,
                &root,
                &format!(
                    "search-{}-{width:.0}x{height:.0}-timezone",
                    appearance.label().to_lowercase()
                ),
            )
            .unwrap();
            drop(view);
            cx.update(|w, _| w.remove_window());
        }
    }
    set_dark(false);
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    let sources: Vec<_> = [
        ("text_input", include_bytes!("ui/text_input.rs").as_slice()),
        ("controls", include_bytes!("ui/controls.rs").as_slice()),
        ("toolbar", include_bytes!("ui/toolbar.rs").as_slice()),
        ("timezone", include_bytes!("ui/timezone_select.rs").as_slice()),
        ("history", include_bytes!("workspace_history.rs").as_slice()),
        ("main", include_bytes!("main.rs").as_slice()),
        ("theme", include_bytes!("ui/theme.rs").as_slice()),
    ].into_iter().map(|(name, bytes)| serde_json::json!({"name":name,"sha256":hex::encode(Sha256::digest(bytes))})).collect();
    fs::write(
        root.join("search-manifest.json"),
        serde_json::to_vec_pretty(&serde_json::json!({"frames":manifest,"sources":sources}))
            .unwrap(),
    )
    .unwrap();
}
