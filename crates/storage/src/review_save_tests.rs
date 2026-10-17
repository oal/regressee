// SPDX-License-Identifier: MPL-2.0
//! Review persistence validates database metadata, not screenshot bytes.
use super::*;

fn fixture() -> Result<(tempfile::TempDir, Store, ComparisonEvidence, ReviewDecision)> {
    let root = tempfile::tempdir()?;
    let store = Store::open(AppPaths::isolated(root.path()))?;
    let project = ProjectId::new();
    let run = RunId::new();
    let baseline = BaselineId::new();
    // Minimal relational rows deliberately have no renderable capture evidence.
    // An annotation must not reopen runs, baselines or image files.
    store.connection.execute(
        "INSERT INTO projects VALUES(?1,'Review fixture','{}',1,100)",
        [project.to_string()],
    )?;
    store.connection.execute(
        "INSERT INTO runs VALUES(?1,?2,'\"Completed\"','{}',100,0)",
        params![run.to_string(), project.to_string()],
    )?;
    store.connection.execute(
        "INSERT INTO baselines VALUES(?1,?2,?3,'{}')",
        params![baseline.to_string(), project.to_string(), run.to_string()],
    )?;
    let analysis = ComparisonEvidence {
        revision: AnalysisRevision {
            id: AnalysisId::new(),
            run,
            baseline,
            algorithm: "rgba-max-channel-v1".into(),
            policy: AnalysisPolicy {
                color_tolerance: 0.,
                changed_percent: 0.,
                region_pixels: 0,
            },
            mask: None,
            results: vec![AnalysisResult {
                key: TargetKey {
                    page: PageId::new(),
                    profile: ProfileId::new(),
                },
                classification: Classification::Changed,
                eligible_pixels: 100,
                changed_pixels: 10,
                dimensions_changed: false,
            }],
        },
        heatmap: None,
        heatmaps: vec![],
        perceptual: None,
    };
    store.connection.execute(
        "INSERT INTO analyses VALUES(?1,?2,?3,?4)",
        params![
            analysis.revision.id.to_string(),
            run.to_string(),
            baseline.to_string(),
            encode(&analysis)?
        ],
    )?;
    let decision = ReviewDecision {
        analysis: analysis.revision.id,
        key: analysis.revision.results[0].key.clone(),
        state: ReviewState::Intentional,
        note: " Exact note.\n日本語 🦀 café\r\nLast line. \n".into(),
        reviewed_at: 101,
    };
    Ok((root, store, analysis, decision))
}

#[test]
fn review_save_returns_its_run_and_appends_exact_revision_bound_history() -> Result<()> {
    let (_root, mut store, analysis, mut decision) = fixture()?;
    let original = encode(&analysis)?;
    let mut expected = Vec::new();
    for state in [
        ReviewState::Intentional,
        ReviewState::NeedsFix,
        ReviewState::Unreviewed,
    ] {
        decision.state = state;
        decision.reviewed_at += 1;
        assert_eq!(store.save_review(&decision)?, analysis.revision.run);
        expected.insert(0, decision.clone());
        assert_eq!(store.review_history(analysis.revision.run)?, expected);
    }
    let stored: String = store.connection.query_row(
        "SELECT record FROM analyses WHERE id=?1",
        [decision.analysis.to_string()],
        |row| row.get(0),
    )?;
    assert_eq!(stored, original, "reviews must not rewrite evidence");
    assert!(store.review_history(RunId::new())?.is_empty());
    Ok(())
}

#[test]
fn review_save_rejects_invalid_decisions_without_appending_or_rewriting() -> Result<()> {
    let (_root, mut store, analysis, decision) = fixture()?;
    store.save_review(&decision)?;
    let original = store.review_history(analysis.revision.run)?;
    let mut invalid = Vec::new();
    let mut changed = decision.clone();
    changed.analysis = AnalysisId::new();
    invalid.push(changed);
    let mut changed = decision.clone();
    changed.key.page = PageId::new();
    invalid.push(changed);
    let mut changed = decision.clone();
    changed.key.profile = ProfileId::new();
    invalid.push(changed);
    for reviewed_at in [0, -1] {
        let mut changed = decision.clone();
        changed.reviewed_at = reviewed_at;
        invalid.push(changed);
    }
    for note in ["x".repeat(REVIEW_NOTE_MAX_BYTES + 1), "🦀".repeat(2049)] {
        let mut changed = decision.clone();
        changed.note = note;
        invalid.push(changed);
    }
    for decision in invalid {
        assert!(store.save_review(&decision).is_err());
        assert_eq!(store.review_history(analysis.revision.run)?, original);
    }
    Ok(())
}

#[test]
fn review_save_rejects_corrupt_metadata_and_relational_identity_mismatches() -> Result<()> {
    let (_root, mut store, analysis, decision) = fixture()?;
    store.save_review(&decision)?;
    let original = store.review_history(analysis.revision.run)?;
    let mut corrupt = vec!["not JSON".to_owned()];
    let mut changed = analysis.clone();
    changed.revision.id = AnalysisId::new();
    corrupt.push(encode(&changed)?);
    let mut changed = analysis.clone();
    changed.revision.run = RunId::new();
    corrupt.push(encode(&changed)?);
    let mut changed = analysis.clone();
    changed.revision.baseline = BaselineId::new();
    corrupt.push(encode(&changed)?);
    let mut changed = analysis.clone();
    changed.revision.policy.changed_percent = -1.;
    corrupt.push(encode(&changed)?);
    let mut changed = analysis.clone();
    changed.revision.results[0].changed_pixels = 101;
    corrupt.push(encode(&changed)?);
    let mut changed = analysis.clone();
    changed.revision.results[0].classification = Classification::NoSignificantChange;
    changed.revision.results[0].eligible_pixels = 0;
    changed.revision.results[0].changed_pixels = 0;
    corrupt.push(encode(&changed)?);
    for record in corrupt {
        store.connection.execute(
            "UPDATE analyses SET record=?1 WHERE id=?2",
            params![record, decision.analysis.to_string()],
        )?;
        assert!(store.save_review(&decision).is_err());
        assert_eq!(store.review_history(analysis.revision.run)?, original);
    }
    Ok(())
}
