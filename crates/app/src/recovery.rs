// SPDX-License-Identifier: MPL-2.0
//! Shared failure causes and safe explanations, never native navigation or raw site stacks.
use crate::product::RuntimeState;
use regressee_runtime::protocol::FailureCode;

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum Problem {
    Browser(RuntimeState),
    Worker(FailureCode),
    PermissionDenied,
    LowSpace,
}

impl Problem {
    pub fn title(self) -> &'static str {
        match self {
            Self::Browser(RuntimeState::UpdateRequired) => "Browser update required",
            Self::Browser(RuntimeState::NotInstalled) => "Private browser is not installed",
            Self::Browser(RuntimeState::UnavailableDistribution) => {
                "Browser setup unavailable in this build"
            }
            Self::Browser(RuntimeState::UnsupportedPlatform) => {
                "Browser unavailable on this platform"
            }
            Self::Browser(_) => "Private browser needs repair",
            Self::Worker(FailureCode::NeedsLogin) => "Website login required",
            Self::Worker(FailureCode::AccessDenied) => "Website access denied",
            Self::Worker(FailureCode::Timeout) => "Website did not respond in time",
            Self::Worker(FailureCode::NotFound) => "Website page not found",
            Self::Worker(FailureCode::RateLimited) => "Website is limiting requests",
            Self::Worker(FailureCode::ServerError) => "Website server error",
            Self::Worker(FailureCode::RedirectOutsideScope) => {
                "Website redirected outside your boundaries"
            }
            Self::Worker(FailureCode::ImageLimit) => "Capture exceeds the supported size",
            Self::Worker(FailureCode::PayloadLimit) => "Browser data exceeds the supported limit",
            Self::Worker(FailureCode::UnsupportedVersion) => {
                "Browser components do not match this app"
            }
            Self::Worker(_) => "Browser operation could not finish",
            Self::PermissionDenied => "Folder access denied",
            Self::LowSpace => "Not enough disk space",
        }
    }

    pub fn message(self) -> &'static str {
        match self {
            Self::Browser(RuntimeState::UpdateRequired) => {
                "The installed private-browser components do not match this app build. Open App settings and choose Update browser, then return to your project and retry. Updating does not delete saved screenshots or change your baseline."
            }
            Self::Browser(RuntimeState::NotInstalled) => {
                "The private browser is missing. Open App settings and choose Install browser now, then return to your project and retry. Saved screenshots are kept."
            }
            Self::Browser(RuntimeState::UnavailableDistribution) => {
                "This version of Regressee cannot install its browser. Install a supported version before logging in, finding pages or capturing screenshots. Retrying will not fix this. You can still save projects and view saved screenshots."
            }
            Self::Browser(RuntimeState::UnsupportedPlatform) => {
                "This version of Regressee cannot run its browser on this computer. Use a version that supports your operating system and processor. You can still save projects and view saved screenshots."
            }
            Self::Browser(_) => {
                "The private browser could not be verified. Automatic replacement is blocked. Open App settings and choose Repair browser, then return to your project and retry. Repair does not delete saved screenshots or change your baseline."
            }
            Self::Worker(FailureCode::NeedsLogin) => {
                "The website requires a login, or the saved login has expired. In Project settings, open the login browser, finish logging in, then choose Confirm login before retrying."
            }
            Self::Worker(FailureCode::AccessDenied) => {
                "The website refused access. Check that your account can open the page and review website access in Project settings before retrying."
            }
            Self::Worker(FailureCode::Timeout) => {
                "The website did not finish responding within the time limit. Check that the page loads normally and that your connection works, then retry from the workspace."
            }
            Self::Worker(FailureCode::NotFound) => {
                "The website reported that this page does not exist. Check the starting address and environment mappings in Project settings before finding pages again."
            }
            Self::Worker(FailureCode::RateLimited) => {
                "The website asked Regressee to slow down. Wait before retrying and choose a gentler crawl speed in Project settings."
            }
            Self::Worker(FailureCode::ServerError) => {
                "The website reported a server error. Check that the website is available, then retry from the workspace."
            }
            Self::Worker(FailureCode::RedirectOutsideScope) => {
                "The page redirected to a website outside your approved boundaries. Review the starting addresses and environment mappings in Project settings; only approve websites you are authorized to visit."
            }
            Self::Worker(FailureCode::ImageLimit) => {
                "The page exceeds a capture size limit. Choose visible-screen-only capture in Project settings, then find pages and capture again."
            }
            Self::Worker(FailureCode::PayloadLimit) => {
                "The browser operation exceeded a data-transfer limit. Return to the workspace to review saved work. If it fails again, retain the app version and operation name for support; repeated retries may not fix this limit."
            }
            Self::Worker(FailureCode::UnsupportedVersion) => {
                "The installed browser components do not match this app. Open App settings and check the browser status before repairing or updating it."
            }
            Self::Worker(FailureCode::WorkerStopped) => {
                "The private browser stopped before the operation finished. Return to the workspace to review any saved work, then retry. If it happens again, check the private browser in App settings."
            }
            Self::Worker(_) => {
                "The browser could not complete this operation; the exact cause was not reported. Check that the page opens normally, then return to the workspace to retry. If it fails again, check the private browser in App settings."
            }
            Self::PermissionDenied => {
                "Regressee could not access a required local file or folder. Check its permissions in your operating system; App settings shows the data and cache locations. When exporting, choose a writable destination. Your drafts are kept; retry only after fixing access."
            }
            Self::LowSpace => {
                "There is not enough free disk space to finish. Free space before retrying. Use History to inspect storage before deleting anything; cleanup is never automatic."
            }
        }
    }
}
