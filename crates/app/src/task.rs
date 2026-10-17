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
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
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
            counts: TargetCounts::from_run(run),
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
