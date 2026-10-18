// SPDX-License-Identifier: MPL-2.0
//! GUI-independent frozen capture progress, shared by application clients.
use regressee_core::{crawl::TargetCounts, domain::*};

/// No screenshots, session material, inferred progress or activity log.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CaptureStatus {
    pub run: RunId,
    pub project: ProjectId,
    pub pages: usize,
    pub excluded_pages: usize,
    pub profiles: Vec<String>,
    pub environment: String,
    pub total: u32,
    pub counts: TargetCounts,
    pub partially_captured: usize,
    /// The frozen run plan, never the project's current draft.
    pub capture_plan: CapturePlan,
    pub screenshots: KindCaptureCounts,
    pub dom_snapshots: KindCaptureCounts,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

/// Kind coverage is separate from page/profile totals: one partial target is not
/// two pages, and a successful sibling does not conceal an incomplete kind.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct KindCaptureCounts {
    pub requested: usize,
    pub baseline_omitted: usize,
    pub captured: usize,
    pub failed: usize,
    pub skipped: usize,
    pub cancelled: usize,
    pub pending: usize,
}
impl KindCaptureCounts {
    fn screenshots(run: &CaptureRun) -> Self {
        let mut counts = Self::default();
        for target in &run.targets {
            match target.outcome {
                TargetOutcome::NotRequested => {
                    counts.baseline_omitted += usize::from(run.capture_plan.mode.screenshots());
                    continue;
                }
                TargetOutcome::Pending => counts.pending += 1,
                TargetOutcome::Captured { .. } => counts.captured += 1,
                TargetOutcome::Failed { .. } => counts.failed += 1,
                TargetOutcome::Skipped { .. } => counts.skipped += 1,
                TargetOutcome::Cancelled => counts.cancelled += 1,
            }
            counts.requested += 1;
        }
        if matches!(run.intent, RunIntent::Check { .. }) && run.capture_plan.mode.screenshots() {
            counts.baseline_omitted = (run.inventory.pages.iter().filter(|p| p.included).count()
                * run.profiles.len())
            .saturating_sub(counts.requested);
        }
        counts
    }
    fn dom_snapshots(run: &CaptureRun) -> Self {
        let mut counts = Self::default();
        for target in &run.targets {
            match target.dom_outcome {
                DomTargetOutcome::NotRequested => {
                    counts.baseline_omitted += usize::from(run.capture_plan.mode.dom_snapshots());
                    continue;
                }
                DomTargetOutcome::Pending => counts.pending += 1,
                DomTargetOutcome::Captured { .. } => counts.captured += 1,
                DomTargetOutcome::Failed { .. } => counts.failed += 1,
                DomTargetOutcome::Skipped { .. } => counts.skipped += 1,
                DomTargetOutcome::Cancelled => counts.cancelled += 1,
            }
            counts.requested += 1;
        }
        if matches!(run.intent, RunIntent::Check { .. }) && run.capture_plan.mode.dom_snapshots() {
            counts.baseline_omitted = (run.inventory.pages.iter().filter(|p| p.included).count()
                * run.profiles.len())
            .saturating_sub(counts.requested);
        }
        counts
    }
}

impl CaptureStatus {
    pub fn from_run(run: &CaptureRun) -> Self {
        Self {
            run: run.id,
            project: run.project,
            pages: run.inventory.pages.iter().filter(|p| p.included).count(),
            excluded_pages: run.inventory.pages.iter().filter(|p| !p.included).count(),
            profiles: run
                .profiles
                .iter()
                .map(|p| {
                    if !run.capture_plan.mode.screenshots() {
                        return format!("{} · {} × {}", p.name, p.width, p.height);
                    }
                    format!(
                        "{} · {} × {} · {}",
                        p.name,
                        p.width,
                        p.height,
                        match p.extent {
                            CaptureExtent::FullPage => "Full page",
                            CaptureExtent::VisibleScreen => "Visible screen",
                        }
                    )
                })
                .collect(),
            environment: run.environment.name.clone(),
            total: run.targets.len() as u32,
            counts: target_counts(run),
            partially_captured: run
                .targets
                .iter()
                .filter(|t| t.any_kind_captured() && !t.requested_kinds_captured())
                .count(),
            capture_plan: run.capture_plan.clone(),
            screenshots: KindCaptureCounts::screenshots(run),
            dom_snapshots: KindCaptureCounts::dom_snapshots(run),
            started_at: run.started_at,
            finished_at: run.finished_at,
        }
    }

    pub fn processed(&self) -> u32 {
        self.total.saturating_sub(self.counts.pending as u32)
    }

    pub fn selection_summary(&self) -> String {
        format!(
            "{} of {} listed URLs included · {} excluded before capture · {} screen {}",
            self.pages,
            self.pages + self.excluded_pages,
            self.excluded_pages,
            self.profiles.len(),
            if self.profiles.len() == 1 {
                "size"
            } else {
                "sizes"
            },
        )
    }
}

/// Page/profile counts, not a screenshot-only count and not a count of kinds.
/// A captured sibling cannot advance a still-pending target or count as full
/// success when the other requested kind failed.
pub fn target_counts(run: &CaptureRun) -> TargetCounts {
    let mut counts = TargetCounts::default();
    for target in &run.targets {
        if matches!(target.outcome, TargetOutcome::Pending)
            || matches!(target.dom_outcome, DomTargetOutcome::Pending)
        {
            counts.pending += 1;
        } else if target.requested_kinds_captured() {
            counts.captured += 1;
        } else if matches!(target.outcome, TargetOutcome::Failed { .. })
            || matches!(target.dom_outcome, DomTargetOutcome::Failed { .. })
        {
            counts.failed += 1;
        } else if matches!(target.outcome, TargetOutcome::Cancelled)
            || matches!(target.dom_outcome, DomTargetOutcome::Cancelled)
        {
            counts.cancelled += 1;
        } else if matches!(target.outcome, TargetOutcome::Skipped { .. })
            || matches!(target.dom_outcome, DomTargetOutcome::Skipped { .. })
        {
            counts.skipped += 1;
        } else {
            counts.not_requested += 1;
        }
    }
    counts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_mode_keeps_kind_coverage_separate_from_page_profile_counts() -> anyhow::Result<()> {
        let project = Project::draft("Inert coverage fixture".into(), "http://127.0.0.1:9/")?;
        let page = InventoryPage {
            id: PageId::new(),
            canonical: project.sources[0].clone(),
            included: true,
            reason: None,
        };
        let mut profile = project.profiles[0].clone();
        profile.runtime = RuntimeIdentity::Pinned {
            node: "inert".into(),
            worker: "inert".into(),
            playwright: "inert".into(),
            browser: "inert".into(),
        };
        let screenshots = [
            TargetOutcome::Pending,
            TargetOutcome::Captured {
                artifact: "a".repeat(64).try_into()?,
                width: 1,
                height: 1,
            },
            TargetOutcome::Failed {
                code: CaptureError::Timeout,
            },
            TargetOutcome::Skipped {
                reason: "fixture".into(),
            },
            TargetOutcome::Cancelled,
        ];
        let snapshots = [
            DomTargetOutcome::Pending,
            DomTargetOutcome::Captured {
                artifact: "b".repeat(64).try_into()?,
            },
            DomTargetOutcome::Failed {
                code: DomCaptureError::LimitExceeded,
            },
            DomTargetOutcome::Skipped {
                reason: "fixture".into(),
            },
            DomTargetOutcome::Cancelled,
        ];
        for mode in [
            CaptureMode::ScreenshotsOnly,
            CaptureMode::ScreenshotsAndDomSnapshots,
            CaptureMode::DomSnapshotsOnly,
        ] {
            for (image_index, image) in screenshots.iter().enumerate() {
                for (dom_index, dom) in snapshots.iter().enumerate() {
                    let run = CaptureRun {
                        id: RunId::new(),
                        project: project.id,
                        intent: RunIntent::NewBaseline,
                        inventory: InventoryRevision {
                            id: InventoryId::new(),
                            project: project.id,
                            identity_version: 2,
                            pages: vec![page.clone()],
                            coverage: DiscoveryCoverage::Complete,
                        },
                        profiles: vec![profile.clone()],
                        capture_plan: CapturePlan::for_mode(mode),
                        environment: project.environments[0].clone(),
                        rendering: RenderingEnvironment {
                            os: "inert".into(),
                            architecture: "inert".into(),
                            browser: "inert".into(),
                            worker_version: "inert".into(),
                            profile_version: 1,
                            fonts_fingerprint: None,
                        },
                        state: JobState::Running,
                        started_at: Some(1),
                        finished_at: None,
                        targets: vec![CaptureTarget {
                            key: TargetKey {
                                page: page.id,
                                profile: profile.id,
                            },
                            requested: page.canonical.clone(),
                            final_url: Some(page.canonical.clone()),
                            attempts: 1,
                            outcome: if mode.screenshots() {
                                image.clone()
                            } else {
                                TargetOutcome::NotRequested
                            },
                            dom_outcome: if mode.dom_snapshots() {
                                dom.clone()
                            } else {
                                DomTargetOutcome::NotRequested
                            },
                            warnings: vec![],
                        }],
                    };
                    run.validate()?;
                    let status = CaptureStatus::from_run(&run);
                    assert_eq!(status.pages, 1);
                    assert_eq!(status.total, 1);
                    if !mode.screenshots() {
                        assert!(!status.profiles[0].contains("Full page"));
                        assert!(!status.profiles[0].contains("Visible screen"));
                    }
                    let pending = (mode.screenshots() && image_index == 0)
                        || (mode.dom_snapshots() && dom_index == 0);
                    let captured = (!mode.screenshots() || image_index == 1)
                        && (!mode.dom_snapshots() || dom_index == 1);
                    assert_eq!(status.counts.pending, usize::from(pending));
                    assert_eq!(status.counts.captured, usize::from(captured));
                    assert_eq!(status.processed(), u32::from(!pending));
                    assert_eq!(
                        status.screenshots.requested,
                        usize::from(mode.screenshots())
                    );
                    assert_eq!(
                        status.dom_snapshots.requested,
                        usize::from(mode.dom_snapshots())
                    );
                    let image_counts = [
                        status.screenshots.pending,
                        status.screenshots.captured,
                        status.screenshots.failed,
                        status.screenshots.skipped,
                        status.screenshots.cancelled,
                    ];
                    let dom_counts = [
                        status.dom_snapshots.pending,
                        status.dom_snapshots.captured,
                        status.dom_snapshots.failed,
                        status.dom_snapshots.skipped,
                        status.dom_snapshots.cancelled,
                    ];
                    for index in 0..5 {
                        assert_eq!(
                            image_counts[index],
                            usize::from(mode.screenshots() && image_index == index)
                        );
                        assert_eq!(
                            dom_counts[index],
                            usize::from(mode.dom_snapshots() && dom_index == index)
                        );
                    }
                    assert_eq!(
                        run.targets[0].requested_kinds_captured(),
                        (!mode.screenshots() || image_index == 1)
                            && (!mode.dom_snapshots() || dom_index == 1)
                    );
                }
            }
        }
        Ok(())
    }
}
