// SPDX-License-Identifier: MPL-2.0
//! Synthetic stores and a private engine only. No browser or user data access.
use super::*;
use gpui::{TestAppContext, VisualTestContext};
use regressee_core::{dom_snapshot::*, domain::*, evidence::*};
use std::{io::Cursor, sync::atomic::AtomicBool, time::Instant};

pub(super) struct Fixture {
    pub root: tempfile::TempDir,
    pub project: ProjectRecord,
    pub evidence: ProjectEvidence,
    pub source: String,
}

pub(super) fn fixture(mode: CaptureMode, dom_ok: bool) -> Fixture {
    fixture_with_image_change(mode, dom_ok, false)
}
fn fixture_with_image_change(mode: CaptureMode, dom_ok: bool, image_changes: bool) -> Fixture {
    fixture_with_source(mode, dom_ok, image_changes, false)
}

#[cfg(feature = "ui-renderer-capture")]
pub(super) fn syntax_fixture(mode: CaptureMode) -> Fixture {
    fixture_with_source(mode, true, false, true)
}

fn fixture_with_source(
    mode: CaptureMode,
    dom_ok: bool,
    image_changes: bool,
    syntax: bool,
) -> Fixture {
    let root = tempfile::tempdir().unwrap();
    let paths = AppPaths::isolated(root.path());
    let mut store = regressee_storage::Store::open(paths).unwrap();
    let mut project =
        Project::draft("Synthetic DOM workspace".into(), "http://127.0.0.1:9/inert").unwrap();
    project.capture_plan = CapturePlan::for_mode(mode);
    project.profiles[0].width = 32;
    project.profiles[0].height = 32;
    project.profiles[0].runtime = RuntimeIdentity::Pinned {
        node: "inert".into(),
        worker: "inert".into(),
        playwright: "inert".into(),
        browser: "inert".into(),
    };
    let mut saved = store.save_project(&project, None).unwrap();
    let inventory = InventoryRevision {
        id: InventoryId::new(),
        project: project.id,
        identity_version: 2,
        pages: vec![InventoryPage {
            id: PageId::new(),
            canonical: project.sources[0].clone(),
            included: true,
            reason: None,
        }],
        coverage: DiscoveryCoverage::Complete,
    };
    store.freeze_inventory(&inventory).unwrap();
    let key = TargetKey {
        page: inventory.pages[0].id,
        profile: project.profiles[0].id,
    };
    let mut source = String::new();
    for after in [false, true] {
        let mut record = RunRecord {
            run: CaptureRun {
                id: RunId::new(),
                project: project.id,
                intent: if after {
                    RunIntent::Check {
                        baseline: saved.project.active_baseline.unwrap(),
                    }
                } else {
                    RunIntent::NewBaseline
                },
                inventory: inventory.clone(),
                profiles: project.profiles.clone(),
                capture_plan: project.capture_plan.clone(),
                environment: project.environments[0].clone(),
                rendering: RenderingEnvironment {
                    os: "synthetic".into(),
                    architecture: "synthetic".into(),
                    browser: "inert".into(),
                    worker_version: "inert".into(),
                    profile_version: 1,
                    fonts_fingerprint: None,
                },
                state: JobState::Running,
                started_at: Some(if after { 3 } else { 1 }),
                finished_at: None,
                targets: vec![CaptureTarget {
                    key: key.clone(),
                    requested: project.sources[0].clone(),
                    final_url: None,
                    attempts: 1,
                    outcome: if mode.screenshots() {
                        TargetOutcome::Pending
                    } else {
                        TargetOutcome::NotRequested
                    },
                    dom_outcome: DomTargetOutcome::Pending,
                    warnings: vec![],
                }],
            },
            capture: None,
            captures: vec![],
            dom_captures: vec![],
            queue: None,
        };
        store.begin_run(&record).unwrap();
        if mode.screenshots() {
            let png = root
                .path()
                .join(if after { "after.png" } else { "before.png" });
            let mut encoder = png::Encoder::new(std::fs::File::create(&png).unwrap(), 32, 32);
            encoder.set_color(png::ColorType::Rgba);
            encoder.set_depth(png::BitDepth::Eight);
            encoder
                .write_header()
                .unwrap()
                .write_image_data(
                    &if after && image_changes {
                        [220, 40, 80, 255]
                    } else {
                        [50, 80, 110, 255]
                    }
                    .repeat(32 * 32),
                )
                .unwrap();
            let image = regressee_app::images::ingest(
                &mut store,
                &png,
                ArtifactKind::Screenshot,
                &AtomicBool::new(false),
            )
            .unwrap();
            let metadata = CaptureMetadata {
                version: 1,
                key: key.clone(),
                requested: project.sources[0].to_string(),
                final_url: project.sources[0].to_string(),
                runtime: project.profiles[0].runtime.clone(),
                profile: project.profiles[0].clone(),
                width: 32,
                height: 32,
                sha256: image.original.clone(),
                bytes: store.verify_artifact(&image.original).unwrap().bytes,
                warnings: vec![],
                stable: true,
                duration_ms: 1,
                readiness: Readiness {
                    font_failures: 0,
                    image_failures: 0,
                    resource_failures: 0,
                },
                capture_mode: "chromium-new-headless".into(),
                replacement: None,
            };
            record.run.targets[0].outcome = TargetOutcome::Captured {
                artifact: image.original.clone(),
                width: 32,
                height: 32,
            };
            record.captures.push(CaptureEvidence { metadata, image });
        }
        if !after || dom_ok {
            let mut nodes = vec![
                DomNode::Document,
                DomNode::Element {
                    depth: 1,
                    namespace: HTML_NAMESPACE.into(),
                    name: "html".into(),
                    prefix: None,
                    attributes: vec![],
                },
            ];
            if syntax {
                nodes = DomSnapshot::from_canonical(include_bytes!(
                    "../../core/tests/fixtures/dom-v1.ndjson"
                ))
                .unwrap()
                .nodes;
                if after {
                    for node in &mut nodes {
                        match node {
                            DomNode::Element { attributes, .. } => {
                                for attribute in attributes {
                                    if attribute.name == "class" {
                                        attribute.value = "review ready".into();
                                    }
                                }
                            }
                            DomNode::Text { text, .. } if text == "Continue" => {
                                *text = "Continue to review 🦀".into();
                            }
                            _ => {}
                        }
                    }
                }
            } else {
                for i in 0..3200 {
                    nodes.push(DomNode::Text {
                        depth: 2,
                        text: if after && i == 1600 {
                            format!(
                                "Changed hydrated text 🦀 </pre><script> inert & {}",
                                "long source ".repeat(500)
                            )
                        } else {
                            format!("Recorded line {i}: {}", "ordinary text ".repeat(8))
                        },
                    });
                }
            }
            let snapshot = DomSnapshot { nodes };
            let bytes = snapshot.canonical_bytes().unwrap();
            let stats = snapshot.validate().unwrap();
            if after {
                source = snapshot.format_source().unwrap();
            }
            let metadata = DomCaptureMetadata {
                version: 1,
                run: record.run.id,
                key: key.clone(),
                requested: project.sources[0].to_string(),
                final_url: project.sources[0].to_string(),
                runtime: project.profiles[0].runtime.clone(),
                profile: project.profiles[0].clone(),
                policy: DomSnapshotPolicy::default(),
                sha256: dom_artifact_id(&bytes),
                bytes: bytes.len() as u64,
                node_count: stats.node_count,
                visited_nodes: stats.node_count,
                max_depth: stats.max_depth,
                coverage: stats.coverage,
                warnings: vec![],
                stable: true,
                duration_ms: 1,
                readiness: Readiness {
                    font_failures: 0,
                    image_failures: 0,
                    resource_failures: 0,
                },
            };
            record.dom_captures.push(
                store
                    .ingest_dom_snapshot(Cursor::new(bytes), &metadata)
                    .unwrap(),
            );
            record.run.targets[0].dom_outcome = DomTargetOutcome::Captured {
                artifact: metadata.sha256,
            };
        } else {
            record.run.targets[0].dom_outcome = DomTargetOutcome::Failed {
                code: DomCaptureError::LimitExceeded,
            };
        }
        record.run.targets[0].final_url = Some(project.sources[0].clone());
        record.run.state = if !after || dom_ok {
            JobState::Completed
        } else {
            JobState::CompletedWithErrors
        };
        record.run.finished_at = Some(if after { 4 } else { 2 });
        store.finish_run(&record).unwrap();
        if after {
            regressee_app::review::analyze_saved_run(
                &mut store,
                record.run.id,
                regressee_app::review::preset(regressee_app::analysis_engine::Preset::Balanced),
                &AtomicBool::new(false),
            )
            .unwrap();
        } else {
            saved = store
                .activate_baseline(record.run.id, saved.revision)
                .unwrap();
        }
    }
    Fixture {
        project: store.load_project(project.id).unwrap(),
        evidence: store.project_evidence(project.id).unwrap(),
        root,
        source,
    }
}

pub(super) struct Engine {
    client: regressee_app::engine::Client,
    thread: Option<std::thread::JoinHandle<anyhow::Result<()>>>,
}
impl Engine {
    pub fn start(fixture: &Fixture) -> Self {
        let paths = AppPaths::isolated(fixture.root.path());
        let server = regressee_app::engine::Server::without_runtime(paths.clone()).unwrap();
        let thread = std::thread::spawn(move || server.run());
        let client = regressee_app::engine::Client::existing(paths).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        while client.snapshot().is_err() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        Self {
            client,
            thread: Some(thread),
        }
    }
    pub fn jobs(&self) -> (Arc<Jobs>, std::sync::mpsc::Receiver<Update>) {
        Jobs::connect(self.client.clone())
    }
}
impl Drop for Engine {
    fn drop(&mut self) {
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut stopped = false;
        while Instant::now() < deadline {
            if matches!(
                self.client.request(regressee_app::protocol::Request::Stop),
                Ok(regressee_app::protocol::Response::Ok)
            ) {
                stopped = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(10));
        }
        if !stopped {
            // A panicked engine worker can leave an unfinishable job. Do not
            // hide the original failure behind an unbounded JoinHandle::join.
            if std::thread::panicking() {
                return;
            }
            panic!("Owned synthetic engine could not stop within five seconds");
        }
        if let Some(thread) = self.thread.take() {
            thread.join().unwrap().unwrap();
        }
    }
}

pub(super) fn load(s: &mut Shell, fixture: &Fixture, cx: &mut Context<Shell>) {
    crate::workflow_refinement_tests::finish_jobs(s, cx);
    // The same root metadata path as an ordinary project reopen, including all
    // real engine reads. No fixture record is injected into the native shell.
    s.focus.screen = Screen::Results;
    s.focus.choosing_project = true;
    s.queue(Work::Load(fixture.project.project.id), cx);
    crate::workflow_refinement_tests::finish_jobs(s, cx);
    s.queue(Work::Evidence(fixture.project.project.id), cx);
    crate::workflow_refinement_tests::finish_jobs(s, cx);
    assert_eq!(
        s.project.as_ref().unwrap().project.id,
        fixture.project.project.id
    );
    assert!(
        s.evidence.latest.is_some(),
        "Root evidence failed: {:?}",
        s.error
    );
    assert_eq!(
        s.evidence.latest.as_ref().unwrap().run.id,
        fixture.evidence.latest.as_ref().unwrap().run.id
    );
    assert!(!s.evidence_read_failed, "{:?}", s.error);
    s.focus.review_surface = ReviewSurface::Changes;
    s.notice = None;
    s.sync_dom_workspace(cx);
    s.choose_evidence_kind(CheckKind::DomSnapshot, cx);
    s.sync_review_note(cx);
}
pub(super) fn finish_source(s: &mut Shell, cx: &mut Context<Shell>) {
    let deadline = Instant::now() + Duration::from_secs(20);
    while s.dom.pending.is_some() {
        s.poll_dom_viewer(cx);
        assert!(Instant::now() < deadline, "DOM read did not finish");
        std::thread::sleep(Duration::from_millis(10));
    }
}

pub(super) fn unavailable_screenshot_fixture() -> Fixture {
    let fixture = fixture_with_image_change(CaptureMode::ScreenshotsAndDomSnapshots, true, true);
    let mut store =
        regressee_storage::Store::open(AppPaths::isolated(fixture.root.path())).unwrap();
    let run = fixture.evidence.latest.as_ref().unwrap();
    store
        .save_review(&ReviewDecision {
            analysis: fixture.evidence.comparison.as_ref().unwrap().revision.id,
            key: run.run.targets[0].key.clone(),
            state: ReviewState::NeedsFix,
            note: "Retain the independent screenshot note".into(),
            reviewed_at: 10,
        })
        .unwrap();
    let path = store
        .artifact_path(&run.captures[0].image.original)
        .unwrap();
    drop(store);
    std::fs::write(
        path,
        b"Synthetic unavailable screenshot; DOM source remains valid",
    )
    .unwrap();
    fixture
}

/// Exercise the real mounted editor and its automatic presentation focus, never
/// a direct TextInput::set_value or a fixture decision injected into the shell.
pub(super) fn open_dom_note(shell: &Entity<Shell>, cx: &mut VisualTestContext) {
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            assert!(s.dom_active());
            assert!(s.control_enabled(Control::Phase5(37), cx));
            s.activate_control(Control::Phase5(37), window, cx);
        })
    });
    cx.run_until_parked();
    cx.update(|window, cx| {
        let _ = window.draw(cx);
        shell.read_with(cx, |s, cx| {
            assert!(s.review_note_open);
            for control in [Control::AnalysisField(9), Control::Phase5(32)] {
                assert!(s.control_mounted(control));
                assert!(s.control_enabled(control, cx));
                assert!(
                    s.visible_controls().contains(&control),
                    "Missing focus candidate: {control:?}"
                );
            }
            assert!(s.analysis_fields[9].read(cx).focus.is_focused(window));
        });
    });
    let panel = cx.debug_bounds("viewer-inspector").unwrap();
    let save = cx.debug_bounds("action-Phase5(32)").unwrap();
    assert!(
        save.top() >= panel.top() && save.bottom() <= panel.bottom(),
        "Save note must remain pinned inside its inspector: {save:?}, {panel:?}"
    );
}

pub(super) fn type_dom_note(shell: &Entity<Shell>, cx: &mut VisualTestContext, text: &str) {
    open_dom_note(shell, cx);
    cx.simulate_input(text);
    cx.run_until_parked();
    shell.read_with(cx, |s, cx| {
        assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), text);
        assert!(s.dom_note_dirty(cx));
    });
}

#[gpui::test]
fn bounded_engine_source_and_native_kind_drafts_are_independent(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    let fixture = fixture(CaptureMode::ScreenshotsAndDomSnapshots, true);
    assert!(
        fixture.source.len() > 256 * 1024,
        "Exercise multiple real IPC source pages"
    );
    let engine = Engine::start(&fixture);
    let (jobs, events) = engine.jobs();
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load(s, &fixture, cx);
            finish_source(s, cx);
            assert!(s.dom.frame.is_some() && s.dom_review_comparable());
            s.dom.presentation = regressee_gui::jobs::DomPresentation::After;
            s.reload_dom_viewer(cx);
            finish_source(s, cx);
            let complete = s
                .dom
                .frame
                .as_ref()
                .unwrap()
                .document
                .rows()
                .iter()
                .map(|r| r.text.as_ref())
                .collect::<Vec<_>>()
                .join("\n")
                + "\n";
            assert_eq!(
                complete, fixture.source,
                "The GUI assembled all UTF-8 pages, not a searchable prefix"
            );
            s.dom.presentation = regressee_gui::jobs::DomPresentation::Before;
            s.reload_dom_viewer(cx);
            s.dom.presentation = regressee_gui::jobs::DomPresentation::After;
            s.reload_dom_viewer(cx);
            finish_source(s, cx);
            assert_eq!(
                s.dom.frame.as_ref().unwrap().identity.presentation,
                regressee_gui::jobs::DomPresentation::After
            );
            assert!(!s.control_mounted(Control::Guided(54)));
            assert!(!s.control_mounted(Control::ZoomIn));
            assert!(!s.control_mounted(Control::Phase5(14)));
            let before = serde_json::to_value(&s.evidence).unwrap();
            let key = s.selected_key.clone().unwrap();
            s.sync_review_note(cx);
            s.analysis_fields[9].update(cx, |i, cx| i.set_value("DOM-only draft 🦀", cx));
            s.choose_evidence_kind(CheckKind::Screenshot, cx);
            assert!(
                s.pending_dom_notes(cx)
                    .unwrap()
                    .iter()
                    .any(|d| d.note == "DOM-only draft 🦀")
            );
            assert!(
                !s.analysis_fields[9]
                    .read(cx)
                    .content
                    .contains("DOM-only draft")
            );
            s.choose_evidence_kind(CheckKind::DomSnapshot, cx);
            finish_source(s, cx);
            assert_eq!(
                s.analysis_fields[9].read(cx).content.as_ref(),
                "DOM-only draft 🦀"
            );
            s.kind_approval_action(23, cx);
            assert!(s.dom.selected.contains_key(&key));
            assert!(s.approval_selection.is_empty());
            assert_eq!(serde_json::to_value(&s.evidence).unwrap(), before);
            s.dom
                .source
                .update(cx, |source, cx| source.set_enabled(true, cx));
            s.find_local(&FindLocal, window, cx);
            s.focus.dialog = true;
            s.dom
                .source
                .update(cx, |source, cx| source.set_enabled(false, cx));
            assert!(!s.control_enabled(Control::DomViewer(0), cx));
            assert!(s.dom.source.read(cx).focus_handles(cx).is_empty());
        })
    });
    owner.join_after_ui();
}

#[gpui::test]
fn real_dom_review_and_explicit_consent_bind_sources_and_reject_stale_preview(
    cx: &mut TestAppContext,
) {
    crate::image_controls_tests::bind(cx);
    let fixture = fixture(CaptureMode::DomSnapshotsOnly, true);
    let engine = Engine::start(&fixture);
    let (jobs, events) = engine.jobs();
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|_, cx| {
        shell.update(cx, |s, cx| {
            load(s, &fixture, cx);
            finish_source(s, cx);
        })
    });
    type_dom_note(&shell, cx, "Reviewed DOM only");
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            s.activate_control(Control::Phase5(32), window, cx);
            crate::workflow_refinement_tests::finish_jobs(s, cx);
            s.activate_control(Control::Phase5(39), window, cx);
            s.activate_control(Control::Phase5(21), window, cx);
            crate::workflow_refinement_tests::finish_jobs(s, cx);
            assert!(s.evidence.reviews.is_empty());
            assert_eq!(s.evidence.dom_reviews[0].note, "Reviewed DOM only");
            s.kind_approval_action(23, cx);
            s.kind_approval_action(24, cx);
            crate::workflow_refinement_tests::finish_jobs(s, cx);
            assert!(s.focus.approval_dialog && s.approval_preview_current());
            let preview = s.dom.approval.as_ref().unwrap().clone();
            assert!(preview.screenshot.is_none() && preview.dom.is_some());
            let revision = s.project.as_ref().unwrap().revision;
            s.project.as_mut().unwrap().revision += 1;
            assert!(!s.kind_approval_matches(&preview));
            s.project.as_mut().unwrap().revision = revision;
            let key = s.selected_key.clone().unwrap();
            let selected = s.dom.selected[&key].clone();
            s.dom.selected.get_mut(&key).unwrap().after.run = RunId::new();
            assert!(!s.kind_approval_matches(&preview));
            s.dom.selected.insert(key, selected);
            assert!(s.kind_approval_matches(&preview));
            let baseline = s.project.as_ref().unwrap().project.active_baseline;
            s.kind_approval_action(25, cx);
            crate::workflow_refinement_tests::finish_jobs(s, cx);
            assert_ne!(
                s.project.as_ref().unwrap().project.active_baseline,
                baseline
            );
            assert!(s.dom.selected.is_empty());
            assert!(s.approval_selection.is_empty());
            assert!(!s.focus.approval_dialog);
        })
    });
    owner.join_after_ui();
}

#[gpui::test]
fn dom_note_real_typing_save_reopen_and_input_guards_do_not_depend_on_screenshots(
    cx: &mut TestAppContext,
) {
    crate::image_controls_tests::bind(cx);
    for unavailable_shot in [false, true] {
        let fixture = if unavailable_shot {
            unavailable_screenshot_fixture()
        } else {
            fixture(CaptureMode::DomSnapshotsOnly, true)
        };
        let engine = Engine::start(&fixture);
        let (jobs, events) = engine.jobs();
        let owner = jobs.clone();
        let (shell, view) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        view.simulate_resize(size(px(820.), px(600.)));
        view.update(|_, cx| {
            shell.update(cx, |s, cx| {
                load(s, &fixture, cx);
                finish_source(s, cx);
                assert!(s.viewer.is_none());
                assert_eq!(
                    s.kind_read_error(CheckKind::Screenshot).is_some(),
                    unavailable_shot
                );
            })
        });
        let text = "Typed DOM note 🦀 café\nA second line, not screenshot evidence.";
        type_dom_note(&shell, view, text);
        // Retain a genuinely typed dirty draft through the real Close/reopen path.
        view.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Phase5(39), window, cx);
                assert!(!s.review_note_open);
                assert!(s.dom_note_dirty(cx));
            })
        });
        open_dom_note(&shell, view);
        for guard in ["dialog", "prompt", "busy", "agent"] {
            view.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    match guard {
                        "dialog" => s.focus.dialog = true,
                        "prompt" => s.native_prompt = true,
                        "busy" => s.focus.busy = true,
                        "agent" => {
                            s.agent = crate::agent_control_tests::state(
                                regressee_app::control::Status::Active,
                                true,
                            )
                        }
                        _ => unreachable!(),
                    }
                    for control in [Control::AnalysisField(9), Control::Phase5(32)] {
                        assert!(!s.control_enabled(control, cx), "{guard}: {control:?}");
                    }
                    let pending = s.pending;
                    s.activate_control(Control::Phase5(32), window, cx);
                    assert_eq!(s.pending, pending, "{guard}: Save note must not dispatch");
                    cx.notify();
                })
            });
            view.update(|window, cx| {
                let _ = window.draw(cx);
            });
            view.simulate_input("MUST NOT ENTER A GUARDED NOTE");
            view.update(|_, cx| {
                shell.update(cx, |s, cx| {
                    assert_eq!(
                        s.analysis_fields[9].read(cx).content.as_ref(),
                        text,
                        "{guard}"
                    );
                    assert!(s.evidence.dom_reviews.is_empty());
                    s.focus.dialog = false;
                    s.focus.approval_dialog = false;
                    s.native_prompt = false;
                    s.focus.busy = false;
                    s.agent = Default::default();
                    cx.notify();
                })
            });
        }
        let expected = view.update(|window, cx| {
            shell.update(cx, |s, cx| {
                assert!(s.control_enabled(Control::Phase5(32), cx));
                s.activate_control(Control::Phase5(32), window, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert!(!s.dom_note_dirty(cx), "{:?}", s.error);
                let saved = s.evidence.dom_reviews.first().unwrap().clone();
                let result = s.dom_result().unwrap();
                assert_eq!(saved.note, text);
                assert_eq!(
                    saved.analysis,
                    s.evidence.dom_comparison.as_ref().unwrap().revision.id
                );
                assert_eq!(Some(&saved.before), result.before.as_ref());
                assert_eq!(Some(&saved.after), result.after.as_ref());
                assert_eq!(
                    saved.state,
                    ReviewState::Unreviewed,
                    "Save note is not a review classification"
                );
                saved
            })
        });
        view.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.activate_control(Control::Phase5(39), window, cx);
                s.activate_control(Control::Phase5(23), window, cx);
                s.activate_control(Control::Phase5(24), window, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert!(s.focus.approval_dialog && s.approval_preview_current());
                assert!(!s.control_enabled(Control::AnalysisField(9), cx));
                assert!(!s.control_enabled(Control::Phase5(32), cx));
            })
        });
        view.run_until_parked();
        view.simulate_input("MUST NOT EDIT WHILE REAL CONSENT IS OPEN");
        assert_eq!(
            shell.read_with(view, |s, cx| s.analysis_fields[9]
                .read(cx)
                .content
                .to_string()),
            text
        );
        view.simulate_keystrokes("escape");
        assert!(!shell.read_with(view, |s, _| s.focus.approval_dialog));
        owner.join_after_ui();

        // A fresh root client, not the first shell's reconciled reply or draft.
        let (jobs, events) = engine.jobs();
        let owner = jobs.clone();
        let (reopened, view) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        view.update(|_, cx| {
            reopened.update(cx, |s, cx| {
                load(s, &fixture, cx);
                finish_source(s, cx);
                assert_eq!(
                    s.evidence.dom_reviews.as_slice(),
                    std::slice::from_ref(&expected)
                );
            })
        });
        open_dom_note(&reopened, view);
        view.update(|window, cx| {
            reopened.update(cx, |s, cx| {
                assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), text);
                assert!(!s.dom_note_dirty(cx));
                s.activate_control(Control::Phase5(39), window, cx);
                s.navigate(Screen::History, window, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                s.run_history_index = s
                    .history
                    .as_ref()
                    .unwrap()
                    .runs
                    .iter()
                    .position(|record| {
                        record.run.id == fixture.evidence.latest.as_ref().unwrap().run.id
                    })
                    .unwrap();
                assert!(s.control_enabled(Control::Phase6(19), cx));
                s.activate_control(Control::Phase6(19), window, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                if !s.dom_active() {
                    assert!(s.control_enabled(Control::EvidenceKind(CheckKind::DomSnapshot), cx));
                    s.activate_control(Control::EvidenceKind(CheckKind::DomSnapshot), window, cx);
                }
                finish_source(s, cx);
                assert!(
                    s.history_inspection.is_some() && s.dom_active() && s.dom_review_comparable()
                );
                assert!(!s.control_enabled(Control::Phase5(37), cx));
                for control in [
                    Control::AnalysisField(9),
                    Control::Phase5(32),
                    Control::Phase5(21),
                ] {
                    assert!(!s.control_enabled(control, cx));
                    s.activate_control(control, window, cx);
                }
                assert!(s.pending.is_none());
                assert_eq!(
                    s.evidence.dom_reviews.as_slice(),
                    std::slice::from_ref(&expected)
                );
            })
        });
        view.run_until_parked();
        view.simulate_input("MUST NOT TYPE INTO RECORDED EVIDENCE");
        reopened.read_with(view, |s, cx| {
            assert!(!s.analysis_fields[9].read(cx).content.contains("MUST NOT"));
            assert_eq!(s.evidence.dom_reviews, [expected]);
        });
        owner.join_after_ui();
        drop(engine);
        let store =
            regressee_storage::Store::open(AppPaths::isolated(fixture.root.path())).unwrap();
        let shots = store
            .review_history(fixture.evidence.latest.as_ref().unwrap().run.id)
            .unwrap();
        if unavailable_shot {
            assert_eq!(shots.len(), 1);
            assert_eq!(shots[0].note, "Retain the independent screenshot note");
            assert_eq!(shots[0].state, ReviewState::NeedsFix);
        } else {
            assert!(shots.is_empty());
        }
    }
}

#[gpui::test]
fn short_dom_workspaces_keep_a_readable_source_viewport(cx: &mut TestAppContext) {
    crate::image_controls_tests::bind(cx);
    for mode in [
        CaptureMode::DomSnapshotsOnly,
        CaptureMode::ScreenshotsAndDomSnapshots,
    ] {
        let fixture = fixture(mode, true);
        let engine = Engine::start(&fixture);
        let (jobs, events) = engine.jobs();
        let owner = jobs.clone();
        let (shell, view) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        view.simulate_resize(size(px(820.), px(600.)));
        view.update(|_, cx| {
            shell.update(cx, |s, cx| {
                load(s, &fixture, cx);
                finish_source(s, cx);
                s.configure_review_geometry(820.);
            })
        });
        view.run_until_parked();
        view.update(|window, cx| {
            let _ = window.draw(cx);
        });
        let source = view.debug_bounds("dom-diff-source").unwrap();
        assert!(source.size.height >= px(48.), "{mode:?}: {source:?}");
        owner.join_after_ui();
    }
}

#[gpui::test]
fn snapshot_only_failure_and_stale_kind_consent_never_enable_image_actions(
    cx: &mut TestAppContext,
) {
    let fixture = fixture(CaptureMode::DomSnapshotsOnly, false);
    let engine = Engine::start(&fixture);
    let (jobs, events) = engine.jobs();
    let owner = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    cx.update(|window, cx| {
        shell.update(cx, |s, cx| {
            load(s, &fixture, cx);
            finish_source(s, cx);
            assert!(s.dom_active());
            assert!(!s.control_mounted(Control::EvidenceKind(CheckKind::Screenshot)));
            let zoom = s.zoom;
            for control in [
                Control::ZoomIn,
                Control::PanDown,
                Control::SplitView,
                Control::Guided(54),
                Control::ImageControls,
                Control::Phase5(14),
            ] {
                assert!(!s.control_mounted(control));
                assert!(!s.control_enabled(control, cx));
                s.activate_control(control, window, cx);
            }
            assert_eq!(s.zoom, zoom);
            assert!(!s.dom_review_comparable());
            assert!(
                s.dom_approval_candidate(s.selected_key.as_ref().unwrap())
                    .is_none()
            );
            assert!(!s.control_enabled(Control::Phase5(23), cx));
            assert!(!s.approval_preview_current());
        })
    });
    owner.join_after_ui();
}

#[test]
fn unstable_dom_receipts_remain_inspectable_but_are_excluded_from_save_consent() {
    let fixture = fixture(CaptureMode::DomSnapshotsOnly, true);
    let mut run = fixture.evidence.latest.unwrap();
    run.run.intent = RunIntent::NewBaseline;
    run.dom_captures[0].metadata.stable = false;
    run.dom_captures[0]
        .metadata
        .warnings
        .push("Unstable".into());
    assert_eq!(run.baseline_save_coverage().unwrap().dom_captures, 1);
    let eligible = regressee_gui::task::baseline_consent_coverage(&run).unwrap();
    assert_eq!(eligible.dom_captures, 0);
    assert_eq!(eligible.captured_targets, 0);
    assert_eq!(eligible.dom_intended, 1);
    assert_eq!(run.dom_captures.len(), 1);
}

#[gpui::test]
fn what_to_check_saves_each_mode_and_binds_the_pending_draft_signature(cx: &mut TestAppContext) {
    for mode in [
        CaptureMode::ScreenshotsOnly,
        CaptureMode::ScreenshotsAndDomSnapshots,
        CaptureMode::DomSnapshotsOnly,
    ] {
        let root = tempfile::tempdir().unwrap();
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(root.path()));
        let owner = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert_eq!(
                    s.draft_settings.capture_plan.mode,
                    CaptureMode::ScreenshotsOnly
                );
                s.focus.screen = Screen::Setup;
                s.focus.setup_stage = SetupStage::Pages;
                s.name
                    .update(cx, |i, cx| i.set_value("Capture mode fixture", cx));
                s.address
                    .update(cx, |i, cx| i.set_value("http://127.0.0.1:9/inert", cx));
                let signature = s.setup_signature(cx);
                s.activate_control(Control::CaptureMode(mode), window, cx);
                assert_eq!(s.draft_settings.capture_plan.mode, mode);
                assert_eq!(
                    signature == s.setup_signature(cx),
                    mode == CaptureMode::ScreenshotsOnly
                );
                if mode == CaptureMode::DomSnapshotsOnly {
                    assert!(!s.control_mounted(Control::Choice(Choice::FullPage)));
                    assert!(!s.control_mounted(Control::Choice(Choice::ReplaceImages)));
                    assert!(s.control_mounted(Control::Guided(1)));
                }
                s.save_settings(cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert_eq!(s.project.as_ref().unwrap().project.capture_plan.mode, mode);
            })
        });
        owner.join_after_ui();
    }
}

#[gpui::test]
fn fresh_real_ipc_root_reopen_and_refresh_keep_valid_sibling_reachable_after_corruption(
    cx: &mut TestAppContext,
) {
    crate::image_controls_tests::bind(cx);
    for bad in [CheckKind::DomSnapshot, CheckKind::Screenshot] {
        let fixture =
            fixture_with_image_change(CaptureMode::ScreenshotsAndDomSnapshots, true, true);
        let frozen = serde_json::to_value(&fixture.evidence.latest.as_ref().unwrap().run).unwrap();
        let path = {
            let store =
                regressee_storage::Store::open(AppPaths::isolated(fixture.root.path())).unwrap();
            let run = fixture.evidence.latest.as_ref().unwrap();
            let artifact = if bad == CheckKind::DomSnapshot {
                &run.dom_captures[0].metadata.sha256
            } else {
                &run.captures[0].image.original
            };
            store.artifact_path(artifact).unwrap()
        };
        std::fs::write(path, b"Synthetic corrupt unselected-kind artifact").unwrap();
        let engine = Engine::start(&fixture);
        let (jobs, events) = engine.jobs();
        let owner = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        let good = if bad == CheckKind::DomSnapshot {
            CheckKind::Screenshot
        } else {
            CheckKind::DomSnapshot
        };
        cx.update(|_, cx| {
            shell.update(cx, |s, cx| {
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                // Actual root flow: no fixture project/evidence injection. A fresh
                // client loads the project, then ordinary combined Evidence/refresh.
                s.focus.screen = Screen::Results;
                s.focus.choosing_project = true;
                s.queue(Work::Load(fixture.project.project.id), cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                s.queue(Work::Evidence(fixture.project.project.id), cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert!(!s.evidence_read_failed, "{bad:?}: {:?}", s.error);
                assert_eq!(
                    s.result_indices().len(),
                    1,
                    "The root loader must still expose the queue"
                );
                assert_eq!(
                    serde_json::to_value(&s.evidence.latest.as_ref().unwrap().run).unwrap(),
                    frozen,
                    "Do not rewrite a captured receipt into a fake failed/unchanged verdict"
                );
                let cause = s
                    .kind_read_error(bad)
                    .expect("Expose the actual kind-scoped engine failure");
                assert!(!cause.is_empty());
                assert!(s.kind_read_error(good).is_none());
                let key = s.selected_key.clone().unwrap();
                assert!(s.kind_status(bad, &key).contains("Unreadable evidence"));
                s.result_filter = 2;
                assert_eq!(
                    s.result_indices().len(),
                    1,
                    "Missing filter includes unreadable siblings"
                );
                s.result_filter = 3;
                assert!(
                    s.result_indices().is_empty(),
                    "Corruption is never unchanged"
                );
                s.result_filter = 0;
                s.choose_evidence_kind(good, cx);
                if good == CheckKind::DomSnapshot {
                    s.reload_dom_viewer(cx);
                    finish_source(s, cx);
                } else {
                    s.reload_viewer(cx);
                    crate::workflow_refinement_tests::finish_jobs(s, cx);
                }
                assert!(s.review_result_comparable(), "{good:?}: {:?}", s.error);
                s.sync_review_note(cx);
                s.analysis_fields[9].update(cx, |input, cx| {
                    input.set_value("Keep this valid sibling draft", cx)
                });
                s.queue(Work::Evidence(fixture.project.project.id), cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                s.sync_review_note(cx);
                assert_eq!(
                    s.analysis_fields[9].read(cx).content.as_ref(),
                    "Keep this valid sibling draft"
                );
                assert!(s.kind_read_error(bad).is_some());
                s.choose_evidence_kind(bad, cx);
                if bad == CheckKind::DomSnapshot {
                    s.reload_dom_viewer(cx);
                    finish_source(s, cx);
                    assert!(s.dom.failed);
                } else {
                    s.reload_viewer(cx);
                    assert!(s.viewer_read_failed);
                }
                assert!(!s.review_result_comparable());
                assert!(!s.control_enabled(Control::Phase5(23), cx));
                cx.notify();
            })
        });
        cx.run_until_parked();
        cx.update(|window, cx| {
            let _ = window.draw(cx);
        });
        assert!(
            cx.debug_bounds(if bad == CheckKind::DomSnapshot {
                "dom-diff-state"
            } else {
                "screenshot-kind-read-error"
            })
            .is_some()
        );
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                s.choose_evidence_kind(good, cx);
                if good == CheckKind::DomSnapshot {
                    s.reload_dom_viewer(cx);
                    finish_source(s, cx);
                } else {
                    s.reload_viewer(cx);
                    crate::workflow_refinement_tests::finish_jobs(s, cx);
                }
                s.sync_review_note(cx);
                assert_eq!(
                    s.analysis_fields[9].read(cx).content.as_ref(),
                    "Keep this valid sibling draft"
                );
                s.activate_control(Control::Phase5(21), window, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                if good == CheckKind::DomSnapshot {
                    assert_eq!(
                        s.evidence.dom_reviews[0].note,
                        "Keep this valid sibling draft"
                    );
                } else {
                    assert_eq!(s.evidence.reviews[0].note, "Keep this valid sibling draft");
                }
                s.kind_approval_action(23, cx);
                s.kind_approval_action(24, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert!(s.approval_preview_current(), "{good:?}: {:?}", s.error);
                let preview = s.dom.approval.as_ref().unwrap().clone();
                assert_eq!(preview.dom.is_some(), good == CheckKind::DomSnapshot);
                assert_eq!(preview.screenshot.is_some(), good == CheckKind::Screenshot);
                let baseline = fixture.evidence.baseline.as_ref().unwrap();
                if bad == CheckKind::DomSnapshot {
                    assert_eq!(preview.baseline.dom_references, baseline.dom_references);
                } else {
                    assert_eq!(preview.baseline.references, baseline.references);
                }
                s.kind_approval_action(25, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert_eq!(
                    s.project.as_ref().unwrap().project.active_baseline,
                    Some(preview.baseline.id)
                );
            })
        });
        owner.join_after_ui();
    }
}

#[gpui::test]
fn gui_same_check_independent_descendant_approvals_and_both_kind_atomic_consent(
    cx: &mut TestAppContext,
) {
    crate::image_controls_tests::bind(cx);
    for together in [false, true] {
        let fixture =
            fixture_with_image_change(CaptureMode::ScreenshotsAndDomSnapshots, true, true);
        let engine = Engine::start(&fixture);
        let (jobs, events) = engine.jobs();
        let owner = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.update(|window, cx| {
            shell.update(cx, |s, cx| {
                load(s, &fixture, cx);
                finish_source(s, cx);
                let original = s.evidence.baseline.clone().unwrap();
                let check = s.evidence.latest.as_ref().unwrap().run.id;
                let shot_analysis = s.evidence.comparison.as_ref().unwrap().revision.id;
                let dom_analysis = s.evidence.dom_comparison.as_ref().unwrap().revision.id;
                s.activate_control(Control::EvidenceKind(CheckKind::Screenshot), window, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert!(s.control_enabled(Control::Phase5(23), cx));
                s.activate_control(Control::Phase5(23), window, cx);
                if together {
                    s.activate_control(Control::EvidenceKind(CheckKind::DomSnapshot), window, cx);
                    finish_source(s, cx);
                    s.activate_control(Control::Phase5(23), window, cx);
                }
                s.activate_control(Control::Phase5(24), window, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert!(s.approval_preview_current(), "{:?}", s.error);
                let first = s.dom.approval.as_ref().unwrap().clone();
                assert!(first.screenshot.is_some());
                assert_eq!(first.dom.is_some(), together);
                assert_ne!(first.baseline.references, original.references);
                if together {
                    assert_ne!(first.baseline.dom_references, original.dom_references);
                } else {
                    assert_eq!(first.baseline.dom_references, original.dom_references);
                }
                s.activate_control(Control::Phase5(25), window, cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                assert_eq!(
                    s.project.as_ref().unwrap().project.active_baseline,
                    Some(first.baseline.id)
                );
                s.queue(Work::Evidence(fixture.project.project.id), cx);
                crate::workflow_refinement_tests::finish_jobs(s, cx);
                if !together {
                    s.activate_control(Control::EvidenceKind(CheckKind::DomSnapshot), window, cx);
                    finish_source(s, cx);
                    assert!(
                        s.control_mounted(Control::Phase5(23)),
                        "A→B screenshot approval must not hide DOM selection"
                    );
                    assert!(s.control_enabled(Control::Phase5(23), cx), "{:?}", s.error);
                    s.activate_control(Control::Phase5(23), window, cx);
                    s.activate_control(Control::Phase5(24), window, cx);
                    crate::workflow_refinement_tests::finish_jobs(s, cx);
                    assert!(s.approval_preview_current(), "{:?}", s.error);
                    let second = s.dom.approval.as_ref().unwrap().clone();
                    assert!(second.screenshot.is_none() && second.dom.is_some());
                    assert_eq!(second.baseline.parent, Some(first.baseline.id));
                    assert_eq!(
                        second.baseline.references, first.baseline.references,
                        "C inherits B's approved screenshot source"
                    );
                    assert_ne!(
                        second.baseline.dom_references,
                        first.baseline.dom_references
                    );
                    s.activate_control(Control::Phase5(25), window, cx);
                    crate::workflow_refinement_tests::finish_jobs(s, cx);
                    assert_eq!(
                        s.project.as_ref().unwrap().project.active_baseline,
                        Some(second.baseline.id)
                    );
                }
                assert_eq!(
                    s.evidence.latest.as_ref().unwrap().run.id,
                    check,
                    "No new check was run"
                );
                assert_eq!(
                    s.evidence.comparison.as_ref().unwrap().revision.id,
                    shot_analysis
                );
                assert_eq!(
                    s.evidence.dom_comparison.as_ref().unwrap().revision.id,
                    dom_analysis
                );
            })
        });
        owner.join_after_ui();
    }
}
