// SPDX-License-Identifier: MPL-2.0
//! Bounded, synthetic native inspection of result hover and content-sized hints.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};

struct PageHintView {
    discovery: bool,
    address: SharedString,
}

impl Render for PageHintView {
    fn render(&mut self, _: &mut Window, _: &mut Context<Self>) -> impl IntoElement {
        div()
            .size_full()
            .p_6()
            .bg(rgb(GROUND))
            .child(controls::section(
                "Page-list hover hint",
                "Synthetic layout inspection",
            ))
            .child(div().w(px(320.)).child(if self.discovery {
                controls::discovery_row(
                    "capture-hint",
                    self.address.clone(),
                    "Included · Ready to capture",
                    false,
                    true,
                )
            } else {
                controls::page_row(
                    "capture-hint",
                    self.address.clone(),
                    "Desktop · Captured",
                    false,
                    true,
                )
            }))
    }
}

fn hover_row(cx: &mut VisualTestContext, selector: &'static str) {
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
    let row = cx.debug_bounds(selector).unwrap();
    cx.simulate_mouse_move(row.center(), None, gpui::Modifiers::none());
    cx.executor().advance_clock(Duration::from_millis(600));
    cx.run_until_parked();
    cx.update(|w, cx| {
        let _ = w.draw(cx);
    });
}

#[gpui::test]
#[ignore = "explicit two-size native offscreen row-hint inspection"]
fn renderer_capture_row_hints(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    let dir = tempfile::Builder::new()
        .prefix("row-hints-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            finish_jobs(s, cx);
            s.notice = None;
            s.error = None;
            s.navigation_preference = Some(false);
            s.review_queue_collapsed = false;
            s.configure_review_geometry(width);
            cx.notify();
        });
        hover_row(cx, "page-row-target-0");
        assert!(cx.debug_bounds("page-row-hint").is_none());
        capture(
            cx,
            &root,
            &format!("row-hints-{width:.0}x{height:.0}-review"),
        )
        .unwrap();
        owned.join_after_ui();
    }
    for (width, height) in [(1280., 800.), (820., 600.)] {
        for kind in ["page", "discovery", "long-page"] {
            let long = kind == "long-page";
            let (_, cx) = cx.add_window_view(|_, _| PageHintView {
                discovery: kind == "discovery",
                address: if long {
                    format!("https://example.test/{}", "long-page-name/".repeat(30)).into()
                } else {
                    "https://example.test/side/aviser/".into()
                },
            });
            cx.simulate_resize(size(px(width), px(height)));
            hover_row(cx, "page-row-capture-hint");
            let hint = cx.debug_bounds("page-row-hint").unwrap();
            if long {
                assert!(hint.size.width <= px(480.));
                assert!(hint.size.height > px(46.));
            } else {
                assert!(hint.size.width < px(480.));
                assert_eq!(hint.size.height, px(46.));
            }
            assert!(hint.right() <= px(width) && hint.bottom() <= px(height));
            eprintln!("{width}x{height} {kind} hint: {:?}", hint.size);
            capture(
                cx,
                &root,
                &format!("row-hints-{width:.0}x{height:.0}-{kind}"),
            )
            .unwrap();
        }
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}
