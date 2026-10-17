// SPDX-License-Identifier: MPL-2.0
use regressee_app::recovery::Problem;
use regressee_runtime::protocol::FailureCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Destination {
    Workspace,
    AppSettings,
    ProjectSettings,
    History,
}

impl Destination {
    pub fn label(self) -> &'static str {
        match self {
            Self::Workspace => "Return to workspace",
            Self::AppSettings => "Open App settings",
            Self::ProjectSettings => "Open Project settings",
            Self::History => "Open History",
        }
    }
}

pub trait ProblemNavigation {
    fn destination(self) -> Destination;
}
impl ProblemNavigation for Problem {
    fn destination(self) -> Destination {
        match self {
            Self::Browser(_)
            | Self::Worker(FailureCode::UnsupportedVersion)
            | Self::PermissionDenied => Destination::AppSettings,
            Self::Worker(
                FailureCode::NeedsLogin
                | FailureCode::AccessDenied
                | FailureCode::NotFound
                | FailureCode::RateLimited
                | FailureCode::RedirectOutsideScope
                | FailureCode::ImageLimit,
            ) => Destination::ProjectSettings,
            Self::LowSpace => Destination::History,
            Self::Worker(_) => Destination::Workspace,
        }
    }
}
