// SPDX-License-Identifier: MPL-2.0
//! Phase 3 exact storage/save/draft safety, not OS-input qualification.
use super::*;
use crate::workflow_refinement_tests::{finish_jobs, load_review};
use gpui::{EntityInputHandler, TestAppContext};
use regressee_core::domain::{REVIEW_NOTE_MAX_BYTES, ReviewDecision, ReviewState};

#[test]
fn storage_review_note_roundtrips_exact_bytes_and_rejects_without_rewriting() {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project, run, _, _) = approval_fixture::fixture(dir.path());
    let evidence = store.project_evidence(project).unwrap();
    let analysis = evidence.comparison.as_ref().unwrap().revision.id;
    let key = evidence.latest.as_ref().unwrap().run.targets[0].key.clone();
    let notes = [
        "x".repeat(4096),
        "x".repeat(4097),
        "x".repeat(5000),
        "x".repeat(8192),
        "🦀".repeat(2048),
        " First line.\n\n日本語 🦀 café\r\nLast line. \n".into(),
    ];
    for (n, note) in notes.into_iter().enumerate() {
        let decision = ReviewDecision {
            analysis,
            key: key.clone(),
            state: ReviewState::NeedsFix,
            note: note.clone(),
            reviewed_at: 100 + n as i64,
        };
        store.save_review(&decision).unwrap();
        assert_eq!(store.review_history(run).unwrap()[0], decision);
        assert_eq!(
            store.project_evidence(project).unwrap().reviews[0]
                .note
                .as_bytes(),
            note.as_bytes()
        );
    }
    let saved = store.review_history(run).unwrap();
    for note in [
        "x".repeat(8193),
        "é".repeat(4097),
        format!("{}x", "🦀".repeat(2048)),
    ] {
        let decision = ReviewDecision {
            analysis,
            key: key.clone(),
            state: ReviewState::Unreviewed,
            note,
            reviewed_at: 500,
        };
        assert!(
            store
                .save_review(&decision)
                .unwrap_err()
                .to_string()
                .contains(regressee_core::domain::REVIEW_NOTE_LIMIT_MESSAGE)
        );
        assert_eq!(store.review_history(run).unwrap(), saved);
    }
    assert_eq!(REVIEW_NOTE_MAX_BYTES, 8192);
}

#[gpui::test]
fn note_explicit_save_exit_boundaries_and_loaded_legacy_shrinking(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, project, _, _, _) = approval_fixture::fixture(dir.path());
    let evidence = store.project_evidence(project).unwrap();
    store
        .save_review(&ReviewDecision {
            analysis: evidence.comparison.as_ref().unwrap().revision.id,
            key: evidence.latest.as_ref().unwrap().run.targets[0].key.clone(),
            state: ReviewState::Intentional,
            note: "x".repeat(5000),
            reviewed_at: 100,
        })
        .unwrap();
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    cx.update(|w, cx| shell.update(cx, |s, cx| s.activate_control(Control::Phase5(37), w, cx)));
    cx.run_until_parked();
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            assert_eq!(s.analysis_fields[9].read(cx).content.len(), 5000);
            s.analysis_fields[9].update(cx, |i, cx| {
                i.replace_text_in_range(None, "é", w, cx);
                assert_eq!(i.content.len(), 5002);
                i.replace_text_in_range(Some(5000..5001), "", w, cx);
                assert_eq!(i.content.len(), 5000);
            });
        })
    });
    for note in [
        "x".repeat(4096),
        "x".repeat(4097),
        "x".repeat(5000),
        "x".repeat(8192),
        "🦀".repeat(2048),
        "Line one.\n日本語 🦀 café\r\nLast line.\n".into(),
    ] {
        shell.update(cx, |s, cx| {
            s.analysis_fields[9].update(cx, |i, cx| i.set_value(&note, cx));
            assert_eq!(s.note_for_save(cx).unwrap(), note);
            assert_eq!(s.pending_review_notes(cx).unwrap()[0].note, note);
            s.review_note_open = true;
            s.review_ux_action(32, cx);
            assert!(s.pending.is_some());
            finish_jobs(s, cx);
            assert!(!s.review_note_dirty(cx));
            assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), note);
            assert_eq!(s.note_status(cx), ("Saved".into(), false));
        });
    }
    for note in ["x".repeat(8193), "é".repeat(4097)] {
        shell.update(cx, |s, cx| {
            s.analysis_fields[9].update(cx, |i, cx| i.set_value(&note, cx));
            let request = s.next_request;
            let saved = s.evidence.reviews.clone();
            assert!(s.pending_review_notes(cx).is_err());
            assert!(s.note_for_save(cx).is_err());
            s.review_ux_action(32, cx);
            assert_eq!(s.next_request, request);
            assert_eq!(s.evidence.reviews, saved);
            assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), note);
            assert_eq!(
                s.note_status(cx),
                (
                    regressee_core::domain::REVIEW_NOTE_LIMIT_MESSAGE.into(),
                    true
                )
            );
        });
    }
    cx.update(|w, cx| {
        shell.update(cx, |s, cx| {
            s.analysis_fields[9].update(cx, |i, cx| {
                i.set_value(&"x".repeat(9000), cx);
                i.replace_text_in_range(Some(0..1), "", w, cx);
                assert_eq!(i.content.len(), 8999);
                i.replace_text_in_range(Some(0..1000), "é\n", w, cx);
                assert_eq!(i.content.len(), 8002);
            });
            assert_eq!(s.pending_review_notes(cx).unwrap()[0].note.len(), 8002);
        })
    });
    owned.join_after_ui();
}

#[gpui::test]
fn note_editor_close_escape_reopen_identity_reply_and_occlusion_guards(cx: &mut TestAppContext) {
    let dir = tempfile::tempdir().unwrap();
    let (store, project, _, _, _) = approval_fixture::fixture(dir.path());
    drop(store);
    let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
    let owned = jobs.clone();
    let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
    shell.update(cx, |s, cx| load_review(s, project, cx));
    let draft = "Intentional headline.\nConfirm the product grid with the release owner.\n";
    for (width, height) in [(1280., 800.), (820., 600.)] {
        cx.simulate_resize(size(px(width), px(height)));
        cx.update(|w, cx| {
            shell.update(cx, |s, cx| {
                s.analysis_fields[9].update(cx, |i, cx| i.set_value(draft, cx));
                s.activate_control(Control::Phase5(37), w, cx);
                assert!(s.review_note_open);
                for control in [
                    Control::Phase5(21),
                    Control::Phase5(23),
                    Control::Phase5(24),
                    Control::Phase5(33),
                ] {
                    assert!(!s.control_enabled(control, cx));
                }
                s.dismiss(&Dismiss, w, cx);
                assert!(!s.review_note_open);
                assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), draft);
                assert!(s.handle(Control::Phase5(37), cx).is_focused(w));
                assert_eq!(
                    s.review_blocking_reason(cx),
                    Some("Save your draft in Review note before Next unreviewed.")
                );
                assert!(!s.control_enabled(Control::Phase5(33), cx));
                s.activate_control(Control::Phase5(37), w, cx);
                assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), draft);
                s.activate_control(Control::Phase5(39), w, cx);
                assert!(s.handle(Control::Phase5(37), cx).is_focused(w));
                let analysis = s.evidence.comparison.as_ref().unwrap().revision.id;
                let old_key = s.selected_key.clone().unwrap();
                let other = TargetKey {
                    page: regressee_core::domain::PageId::new(),
                    profile: old_key.profile,
                };
                s.selected_key = Some(other.clone());
                assert!(s.note_for_save(cx).is_err());
                assert!(
                    s.pending_review_notes(cx)
                        .unwrap()
                        .iter()
                        .any(|note| note.key == old_key && note.note == draft)
                );
                s.sync_review_note(cx);
                s.analysis_fields[9].update(cx, |i, cx| i.set_value("other target draft", cx));
                s.selected_key = Some(old_key.clone());
                s.sync_review_note(cx);
                let reply = ReviewDecision {
                    analysis,
                    key: old_key.clone(),
                    state: ReviewState::Intentional,
                    note: "older saved note".into(),
                    reviewed_at: 100,
                };
                s.reconcile_exit_review_reply(&[reply]);
                assert!(
                    s.pending_review_notes(cx)
                        .unwrap()
                        .iter()
                        .any(|n| n.key == other && n.note == "other target draft")
                );
                assert_eq!(s.analysis_fields[9].read(cx).content.as_ref(), draft);
            })
        });
    }
    owned.join_after_ui();
}
