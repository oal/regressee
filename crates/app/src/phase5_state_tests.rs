// SPDX-License-Identifier: MPL-2.0
//! Final interactive-state regressions. Isolated data and GPUI simulation only.
use super::*;
use crate::phase1_setup_tests::inside;
use crate::phase4_hierarchy_tests::{activate, draw, tab_to};
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{TestAppContext, VisualTestContext};
use regressee_core::domain::{CaptureError, ProjectId, RunId};

/// Add a genuine stored check with gaps; do not relabel a successful screenshot
/// in a capture recipe. The production analyzer determines all classifications.
pub(super) fn gap_check(store: &mut regressee_storage::Store, project: ProjectId) -> RunId {
    let source = store
        .project_evidence(project)
        .unwrap()
        .latest
        .unwrap()
        .run
        .id;
    gap_check_from(store, source)
}

pub(super) fn gap_check_from(store: &mut regressee_storage::Store, source: RunId) -> RunId {
    let mut record = store.load_run(source).unwrap();
    let completed = record.clone();
    record.run.id = RunId::new();
    record.run.started_at = record.run.started_at.map(|t| t + 3600);
    record.run.finished_at = None;
    record.run.state = JobState::Running;
    record.captures.clear();
    record.capture = None;
    for t in &mut record.run.targets {
        t.outcome = TargetOutcome::Pending;
    }
    store.begin_run(&record).unwrap();
    record.captures = completed.captures;
    record.capture = completed.capture;
    record.run.targets = completed.run.targets;
    let gap = record.run.targets.len().saturating_sub(3);
    let key = record.run.targets[gap].key.clone();
    record.run.targets[gap].outcome = TargetOutcome::Failed {
        code: CaptureError::Timeout,
    };
    record.run.targets[gap].final_url = None;
    record.captures.retain(|c| c.metadata.key != key);
    if record
        .capture
        .as_ref()
        .is_some_and(|c| c.metadata.key == key)
    {
        record.capture = None;
    }
    record.run.state = JobState::CompletedWithErrors;
    record.run.finished_at = record.run.started_at.map(|t| t + 180);
    store.finish_run(&record).unwrap();
    regressee_app::review::analyze_run(
        store,
        record.run.id,
        regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
        &std::sync::atomic::AtomicBool::new(false),
    )
    .unwrap();
    record.run.id
}

/// Wheel the actual reading owner, with a fresh painted selector each frame.
/// No direct ScrollHandle offset or arbitrary whole-window visibility assertion.
pub(super) fn read_at(shell: &Entity<Shell>, cx: &mut VisualTestContext, selector: &'static str) {
    draw(cx);
    for _ in 0..12 {
        let b = cx
            .debug_bounds(selector)
            .unwrap_or_else(|| panic!("Absent reading copy {selector}"));
        let clip = shell.read_with(cx, |s, _| {
            if s.viewer_inspector_open() {
                s.review_details_scroll.bounds()
            } else {
                s.screen_scroll.bounds()
            }
        });
        if b.top() >= clip.top() && b.bottom() <= clip.bottom() {
            inside(b, clip);
            return;
        }
        cx.simulate_event(gpui::ScrollWheelEvent {
            position: gpui::point(clip.right() - px(4.), clip.top() + px(12.)),
            delta: gpui::ScrollDelta::Pixels(gpui::point(px(0.), clip.top() + px(8.) - b.top())),
            ..Default::default()
        });
        draw(cx);
    }
    panic!("Reading copy {selector} not revealed in its actual owner");
}

pub(super) fn edit(
    shell: &Entity<Shell>,
    cx: &mut VisualTestContext,
    control: Control,
    value: &str,
) {
    tab_to(shell, cx, control);
    cx.simulate_keystrokes("ctrl-a");
    cx.simulate_input(value);
    draw(cx);
}

pub(super) fn assert_form(shell: &Entity<Shell>, cx: &mut VisualTestContext, control: Control) {
    // Always deliver a fresh traversal into this field, even when Save's
    // correction already focused it before its taller error geometry painted.
    cx.update(|w, cx| shell.update(cx, |s, cx| s.move_focus(true, w, cx)));
    tab_to(shell, cx, control);
    // Let production next-frame reveal/blur callbacks run. Calling Window::draw
    // alone paints geometry but does not deliver the platform frame callback.
    cx.run_until_parked();
    draw(cx);
    cx.update(|w, cx| {
        shell.read_with(cx, |s, cx| {
            assert!(s.control_mounted(control) && s.control_enabled(control, cx));
            assert!(s.handle(control, cx).is_focused(w));
            inside(
                s.setup_control_bounds(control).unwrap().get().unwrap(),
                s.screen_scroll.bounds(),
            );
        })
    });
}

#[gpui::test]
fn final_access_focus_reveals_whole_padded_choice_rows(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Access;
            s.notice = None;
            s.error = None;
        });
        for (choice, selector) in [
            (Choice::PublicAccess, "choice-Choice(PublicAccess)"),
            (Choice::LoginRequired, "choice-Choice(LoginRequired)"),
        ] {
            let control = Control::Choice(choice);
            assert_form(&shell, cx, control);
            let actual = cx.debug_bounds(selector).unwrap();
            shell.read_with(cx, |s, _| {
                let measured = s.setup_control_bounds(control).unwrap().get().unwrap();
                assert_eq!(
                    actual, measured,
                    "focus geometry must include row padding, not just its content"
                );
                inside(actual, s.screen_scroll.bounds());
            });
        }
        owned.join_after_ui();
    }
}

#[gpui::test]
fn final_setup_alternates_custom_tail_mobile_and_invalid_reveal_are_actual(
    cx: &mut TestAppContext,
) {
    cx.update(text_input::bind);
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let baseline = store.load_project(project).unwrap().project.active_baseline;
    drop(store);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Setup;
            s.focus.setup_stage = SetupStage::Pages;
            s.error = None;
            s.notice = None;
        });
        assert_form(&shell, cx, Control::Guided(3));
        let mobile = shell.read_with(cx, |s, _| {
            s.draft_settings.profiles.iter().any(|p| p.name == "Mobile")
        });
        activate(&shell, cx, Control::Guided(3));
        shell.read_with(cx, |s, _| {
            assert_ne!(
                s.draft_settings.profiles.iter().any(|p| p.name == "Mobile"),
                mobile
            )
        });
        activate(&shell, cx, Control::Guided(6));
        edit(&shell, cx, Control::CrawlField(7), "1024");
        edit(&shell, cx, Control::CrawlField(8), "768");
        activate(&shell, cx, Control::Guided(12));
        assert_form(&shell, cx, Control::Guided(25));
        read_at(&shell, cx, "setup-image-limitations");
        for choice in [Choice::VisibleScreen, Choice::ReplaceImages, Choice::Fast] {
            activate(&shell, cx, Control::Choice(choice));
        }
        shell.read_with(cx, |s, _| {
            assert_eq!(s.draft_settings.discovery.speed, Speed::Fast);
            assert!(
                s.draft_settings
                    .profiles
                    .iter()
                    .all(|p| p.extent == CaptureExtent::VisibleScreen
                        && p.image_mode != regressee_core::domain::ImageMode::Normal)
            );
            assert!(
                s.draft_settings
                    .profiles
                    .iter()
                    .any(|p| p.name == "Custom" && p.width == 1024 && p.height == 768)
            );
        });
        edit(&shell, cx, Control::CrawlField(4), "9");
        let requests = shell.read_with(cx, |s, _| s.next_request);
        activate(&shell, cx, Control::Save);
        assert_form(&shell, cx, Control::CrawlField(4));
        shell.read_with(cx, |s, cx| {
            assert!(s.focus.advanced && s.crawl_fields[4].read(cx).touched);
            assert!(s.crawl_fields[4].read(cx).validation_error().is_some());
            assert_eq!(s.next_request, requests);
            assert!(s.pending.is_none() && s.task.is_none());
        });
        owned.join_after_ui();
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        baseline,
        store.load_project(project).unwrap().project.active_baseline
    );
}

#[gpui::test]
fn final_discovery_inclusion_and_all_classification_filters_keep_evidence(cx: &mut TestAppContext) {
    cx.update(text_input::bind);
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project, _, _, _) = approval_fixture::fixture(dir.path());
    gap_check(&mut store, project);
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.preview = None;
            s.result_filters_open = false;
            s.crawl_fields[9].update(cx, |f, cx| f.set_value("", cx));
            s.notice = None;
            s.error = None;
        });
        let selected = shell.read_with(cx, |s, _| s.selected_key.clone());
        activate(&shell, cx, Control::Guided(45));
        for (choice, index) in Choice::RESULTS.into_iter().zip(0..4) {
            activate(&shell, cx, Control::Choice(choice));
            shell.read_with(cx, |s, _| {
                assert_eq!(s.result_filter, index);
                assert_eq!(s.selected_key, selected);
                for i in s.result_indices() {
                    let t = &s.evidence.latest.as_ref().unwrap().run.targets[i];
                    let c = s
                        .evidence
                        .comparison
                        .as_ref()
                        .unwrap()
                        .revision
                        .results
                        .iter()
                        .find(|r| r.key == t.key)
                        .map(|r| r.classification);
                    assert!(match index {
                        0 => true,
                        1 => c == Some(Classification::Changed),
                        2 =>
                            !matches!(t.outcome, TargetOutcome::Captured { .. })
                                || matches!(
                                    c,
                                    Some(
                                        Classification::NotComparable
                                            | Classification::CouldNotCapture
                                    )
                                ),
                        _ => c == Some(Classification::NoSignificantChange),
                    });
                }
            });
        }
        activate(&shell, cx, Control::Guided(19));
        shell.read_with(cx, |s, _| assert!(s.group_profiles));
        activate(&shell, cx, Control::CloseOptions);
        shell.update(cx, |s, cx| {
            crate::options_ux::fixture_discovery(s);
            s.result_filter = 0;
            s.group_profiles = false;
            cx.notify();
        });
        activate(&shell, cx, Control::Guided(45));
        activate(&shell, cx, Control::Guided(22));
        shell.read_with(cx, |s, _| {
            assert_eq!(s.preview.as_ref().unwrap().included(), 74)
        });
        activate(&shell, cx, Control::Guided(22));
        shell.read_with(cx, |s, _| {
            assert_eq!(s.preview.as_ref().unwrap().included(), 75)
        });
        edit(&shell, cx, Control::CrawlField(9), "/pages/1/");
        activate(&shell, cx, Control::Guided(20));
        shell.read_with(cx, |s, _| {
            assert_eq!(s.preview.as_ref().unwrap().included(), 74)
        });
        activate(&shell, cx, Control::Guided(21));
        shell.read_with(cx, |s, _| {
            assert_eq!(s.preview.as_ref().unwrap().included(), 75)
        });
        edit(&shell, cx, Control::CrawlField(9), "/pages/75/");
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                assert!(!s.control_enabled(Control::Guided(21), cx));
                s.activate_control(Control::Guided(21), w, cx);
            })
        });
        shell.read_with(cx, |s, _| {
            assert_eq!(s.preview.as_ref().unwrap().included(), 75)
        });
    }
    owned.join_after_ui();
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
}

#[gpui::test]
fn final_import_and_network_actions_reveal_in_their_real_outer_clip_without_writes(
    cx: &mut TestAppContext,
) {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project, _, _, _) = approval_fixture::fixture(dir.path());
    for n in 0..3 {
        store
            .save_project(
                &Project::draft(
                    format!("Other synthetic project {n}"),
                    "https://example.test/",
                )
                .unwrap(),
                None,
            )
            .unwrap();
    }
    let baseline = store
        .load_project(project)
        .unwrap()
        .project
        .active_baseline
        .unwrap();
    let preview = store.export_preview(project, baseline).unwrap();
    let zip = dir.path().join("synthetic.zip");
    store
        .export_archive(&preview, &zip, &std::sync::atomic::AtomicBool::new(false))
        .unwrap();
    drop(store);
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            load_review(s, project, cx);
            s.focus.screen = Screen::Projects;
            s.notice = None;
            s.selected_import_path(zip.clone(), cx);
            finish_jobs(s, cx);
        });
        assert_form(&shell, cx, Control::Phase6(1));
        activate(&shell, cx, Control::Details(Detail::Portability));
        assert_form(&shell, cx, Control::Details(Detail::Portability));
        // Four visible project rows formerly aliased Network/Portability focus,
        // looping traversal before it could return to Cancel preview.
        shell.read_with(cx, |s, cx| {
            let controls = s
                .visible_controls()
                .into_iter()
                .filter(|c| s.control_enabled(*c, cx))
                .collect::<Vec<_>>();
            for (i, a) in controls.iter().enumerate() {
                for b in &controls[i + 1..] {
                    assert_ne!(s.handle(*a, cx), s.handle(*b, cx), "aliased {a:?}/{b:?}");
                }
            }
        });
        activate(&shell, cx, Control::Phase6(7));
        activate(&shell, cx, Control::Navigate(Screen::Settings));
        shell.update(cx, finish_jobs);
        assert_form(&shell, cx, Control::Details(Detail::ManagedNetwork));
        activate(&shell, cx, Control::Details(Detail::ManagedNetwork));
        assert_form(&shell, cx, Control::Details(Detail::Network));
        activate(&shell, cx, Control::Details(Detail::Network));
        read_at(&shell, cx, "network-limitations-copy");
        shell.read_with(cx, |s, _| assert!(s.details.contains(&Detail::Network)));
        owned.join_after_ui();
    }
    let store = regressee_storage::Store::open(AppPaths::isolated(dir.path())).unwrap();
    assert_eq!(store.list_projects(0).unwrap().items.len(), 4);
    assert_eq!(
        store.load_project(project).unwrap().project.active_baseline,
        Some(baseline)
    );
}
