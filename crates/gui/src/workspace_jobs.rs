// SPDX-License-Identifier: MPL-2.0
//! Recover queue metadata through independent existing kind-scoped engine reads.
//! No failed artifact is trusted, rewritten, or substituted for a valid sibling.
use super::{Operation, Outcome, Remote};
use anyhow::{Context, Result, ensure};
use regressee_app::{control, engine::dom, protocol, terminal};
use regressee_core::{domain::*, evidence::ProjectEvidence};
use std::{collections::BTreeMap, sync::Arc};

/// GUI-local read status, separate from immutable stored capture/verdict fields.
#[derive(Clone, Debug)]
pub struct EvidenceReadFailures {
    pub project: ProjectId,
    pub run: RunId,
    pub by_kind: BTreeMap<CheckKind, Arc<str>>,
}

impl Remote {
    fn workspace_query(&self, operation: Operation) -> Result<Outcome> {
        let job = self.query(operation)?;
        let _ = self
            .client
            .request(protocol::Request::Acknowledge { id: job.id });
        match job
            .outcome
            .context("Saved evidence read has no terminal outcome")?
        {
            Outcome::ProductFailure(error) => anyhow::bail!("{error}"),
            Outcome::Failure(error) => anyhow::bail!("{}", error.message()),
            Outcome::ActionRequired(problem) => anyhow::bail!("{}", problem.message()),
            outcome if outcome.failed() => anyhow::bail!("Saved evidence could not be verified"),
            outcome => Ok(outcome),
        }
    }

    pub(super) fn recover_workspace(
        &self,
        id: u64,
        project: ProjectId,
        exact_run: Option<RunId>,
        selected: Option<control::Selection>,
    ) -> Result<Outcome> {
        let run = if let Some(run) = exact_run {
            run
        } else {
            // Resolve only an identity. This one-row prefix NEVER supplies queue
            // rows; complete frozen metadata comes from focused evidence below.
            let screenshot =
                self.workspace_query(Operation::Terminal(terminal::Command::Results {
                    selection: terminal::Selection {
                        project,
                        run: None,
                        analysis: None,
                    },
                    offset: 0,
                    limit: 1,
                    page: None,
                    profile: None,
                    classification: None,
                    capture_state: None,
                }));
            match screenshot {
                Ok(Outcome::Terminal(output)) => match *output {
                    terminal::Output::Results(page) => page
                        .selection
                        .run
                        .context("Screenshot result did not resolve its run")?,
                    _ => anyhow::bail!("Unexpected screenshot identity response"),
                },
                _ => match self.workspace_query(Operation::Dom(dom::Command::Results {
                    selection: dom::Selection {
                        project,
                        run: None,
                        analysis: None,
                    },
                    offset: 0,
                    limit: 1,
                    page: None,
                    profile: None,
                    outcome: None,
                    capture_state: None,
                }))? {
                    Outcome::Dom(output) => match *output {
                        dom::Output::Results(page) => page
                            .selection
                            .run
                            .context("DOM result did not resolve its run")?,
                        _ => anyhow::bail!("Unexpected DOM identity response"),
                    },
                    _ => anyhow::bail!("Unexpected DOM identity outcome"),
                },
            }
        };
        let mut healthy: BTreeMap<CheckKind, Box<ProjectEvidence>> = BTreeMap::new();
        let mut failures = BTreeMap::new();
        for kind in [CheckKind::Screenshot, CheckKind::DomSnapshot] {
            let result = self.workspace_query(Operation::SelectedEvidence(control::Selection {
                project,
                run: Some(run),
                kind,
                analysis: selected
                    .as_ref()
                    .filter(|selection| selection.kind == kind)
                    .and_then(|selection| selection.analysis),
                key: None,
                view: control::View::Compare,
            }));
            match result {
                Ok(Outcome::Evidence(evidence)) => {
                    ensure!(
                        evidence
                            .latest
                            .as_ref()
                            .is_some_and(|r| r.run.id == run && r.run.project == project),
                        "Focused evidence changed project/run identity"
                    );
                    healthy.insert(kind, evidence);
                }
                Ok(_) => anyhow::bail!("Unexpected focused saved-evidence response"),
                Err(error) => {
                    failures.insert(
                        kind,
                        Arc::<str>::from(error.to_string().chars().take(4096).collect::<String>()),
                    );
                }
            }
        }
        let mut evidence = healthy
            .values()
            .next()
            .context("Neither requested evidence kind could be read")?
            .as_ref()
            .clone();
        let record = evidence
            .latest
            .as_ref()
            .context("Saved run metadata is absent")?;
        let frozen = &record.run;
        let mode = record.run.capture_plan.mode;
        for sibling in healthy.values() {
            ensure!(
                sibling.latest.as_ref().is_some_and(|r| r.run == *frozen),
                "Frozen run metadata changed between kind reads; refresh again"
            );
            ensure!(
                sibling.baseline.as_ref().map(|b| b.id) == evidence.baseline.as_ref().map(|b| b.id),
                "Focused evidence disagrees about the checked baseline"
            );
        }
        if let Some(screenshot) = healthy.get(&CheckKind::Screenshot) {
            evidence.before = screenshot.before.clone();
            evidence.comparison = screenshot.comparison.clone();
            evidence.analyses = screenshot.analyses.clone();
            evidence.reviews = screenshot.reviews.clone();
            evidence.review_history_truncated = screenshot.review_history_truncated;
        } else {
            evidence.before = None;
            evidence.comparison = None;
            evidence.analyses.clear();
            evidence.reviews.clear();
        }
        if let Some(dom) = healthy.get(&CheckKind::DomSnapshot) {
            evidence.dom_before = dom.dom_before.clone();
            evidence.dom_comparison = dom.dom_comparison.clone();
            evidence.dom_analyses = dom.dom_analyses.clone();
            evidence.dom_reviews = dom.dom_reviews.clone();
        } else {
            evidence.dom_before = None;
            evidence.dom_comparison = None;
            evidence.dom_analyses.clear();
            evidence.dom_reviews.clear();
        }
        failures.retain(|kind, _| match kind {
            CheckKind::Screenshot => mode.screenshots(),
            CheckKind::DomSnapshot => mode.dom_snapshots(),
        });
        let mut pending = self.evidence_failures.lock().unwrap();
        // Bounded even when the UI abandons stale replies during navigation.
        while pending.len() >= 64 {
            pending.pop_first();
        }
        pending.insert(
            id,
            EvidenceReadFailures {
                project,
                run,
                by_kind: failures,
            },
        );
        Ok(Outcome::Evidence(Box::new(evidence)))
    }
}
