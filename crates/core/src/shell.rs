// SPDX-License-Identifier: MPL-2.0
//! Testable navigation/focus rules shared by the native shell.
use crate::appearance::AppAppearance;
use crate::domain::{CaptureMode, CheckKind};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Choice {
    AddressesOnly,
    AddressesAndLinks,
    FullPage,
    VisibleScreen,
    Gentle,
    Balanced,
    Fast,
    OriginalImages,
    ReplaceImages,
    PublicAccess,
    LoginRequired,
    SaveLogin,
    EphemeralLogin,
    LightTheme,
    DarkTheme,
    AllResults,
    ChangedResults,
    MissingResults,
    UnchangedResults,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Detail {
    Capture,
    History,
    Analysis,
    IgnoredAreas,
    Network,
    Portability,
    Browser,
    LocalStorage,
    ManagedNetwork,
    SupportTools,
}
impl Choice {
    pub const SETUP: [Self; 9] = [
        Self::AddressesOnly,
        Self::AddressesAndLinks,
        Self::FullPage,
        Self::VisibleScreen,
        Self::Gentle,
        Self::Balanced,
        Self::Fast,
        Self::OriginalImages,
        Self::ReplaceImages,
    ];
    pub const ENVIRONMENT: [Self; 6] = [
        Self::PublicAccess,
        Self::LoginRequired,
        Self::SaveLogin,
        Self::EphemeralLogin,
        Self::LightTheme,
        Self::DarkTheme,
    ];
    pub const RESULTS: [Self; 4] = [
        Self::AllResults,
        Self::ChangedResults,
        Self::MissingResults,
        Self::UnchangedResults,
    ];
    pub fn index(self) -> usize {
        self as usize
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Projects,
    Setup,
    Results,
    History,
    Settings,
}
impl Screen {
    pub const ALL: [Self; 5] = [
        Self::Projects,
        Self::Setup,
        Self::Results,
        Self::History,
        Self::Settings,
    ];
    pub fn label(self) -> &'static str {
        match self {
            Self::Projects => "Projects",
            Self::Setup => "Project settings",
            Self::Results => "Review changes",
            Self::History => "History",
            Self::Settings => "App settings",
        }
    }
}
/// Presentation stages are independent of persistent project/capture state.
/// Saving a project never advances into a network operation.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum SetupStage {
    Website,
    Access,
    Pages,
    #[default]
    Settings,
}
impl SetupStage {
    pub const ALL: [Self; 4] = [Self::Website, Self::Access, Self::Pages, Self::Settings];
    pub fn index(self) -> usize {
        Self::ALL.iter().position(|s| *s == self).unwrap()
    }
    pub fn mounts(self, control: Control) -> bool {
        use Control::*;
        if matches!(control, ValidationNext | SetupBack) {
            return true;
        }
        match self {
            Self::Settings => true,
            Self::Website => matches!(control, Name | Address | Guided(11) | Save),
            Self::Access => matches!(
                control,
                Environment(_)
                    | RemoveEnvironment
                    | RemoveMapping(_)
                    | Phase6(6 | 7)
                    | Details(Detail::Portability)
                    | EnvironmentOptions
                    | Guided(6 | 27 | 31..=38)
                    | CloseMappingPreview
                    | CrawlField(10..=16)
                    | Choice(
                        self::Choice::PublicAccess
                            | self::Choice::LoginRequired
                            | self::Choice::SaveLogin
                            | self::Choice::EphemeralLogin
                            | self::Choice::LightTheme
                            | self::Choice::DarkTheme
                    )
                    | Save
            ),
            Self::Pages => {
                matches!(control, CaptureMode(_) | Guided(1..=3 | 6..=10 | 12 | 13 | 25) | CrawlField(0..=8) | Choice(self::Choice::AddressesOnly | self::Choice::AddressesAndLinks | self::Choice::FullPage | self::Choice::VisibleScreen | self::Choice::Gentle | self::Choice::Balanced | self::Choice::Fast | self::Choice::OriginalImages | self::Choice::ReplaceImages) | Details(Detail::Capture) | Save)
            }
        }
    }
}
/// Local destinations in the unified project-settings form, not setup stages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SettingsSection {
    Website,
    Capture,
    Access,
}
impl SettingsSection {
    pub const ALL: [Self; 3] = [Self::Website, Self::Capture, Self::Access];
    pub fn index(self) -> usize {
        self as usize
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Website => "Website",
            Self::Capture => "Capture plan",
            Self::Access => "Access & environments",
        }
    }
}
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum ReviewSurface {
    #[default]
    Changes,
    Pages,
    Capture,
    Inspect,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    CaptureMode(CaptureMode),
    EvidenceKind(CheckKind),
    DomSource(usize),
    DomViewer(usize),
    Appearance(AppAppearance),
    Navigate(Screen),
    SetupStage(SetupStage),
    SettingsSection(SettingsSection),
    SetupBack,
    EnvironmentOptions,
    PagesBaseline,
    NewProject,
    Name,
    Address,
    Save,
    OpenProject(usize),
    DeleteProject(usize),
    DeleteCurrentProject,
    RemoveEnvironment,
    RemoveMapping(usize),
    SelectProject(usize),
    ChooseProject,
    CancelProjectChoice,
    PreviousPage,
    NextPage,
    About,
    AboutRepository,
    CloseDialog,
    CaptureBaseline,
    SaveBaseline,
    ReviewCaptureFailures,
    ShowAllResults,
    CheckChanges,
    CancelCapture,
    SetupRuntime,
    RefreshEvidence,
    ZoomIn,
    ZoomOut,
    PanLeft,
    PanRight,
    PanUp,
    PanDown,
    Viewer,
    SplitView,
    SplitDivider,
    ReviewDetails,
    Guided(usize),
    CrawlField(usize),
    PauseCapture,
    Phase5(usize),
    AnalysisField(usize),
    Phase6(usize),
    Phase7(usize),
    Choice(Choice),
    Environment(usize),
    Details(Detail),
    DismissFeedback,
    HistorySearch,
    HistoryClear,
    HistoryRefresh,
    HistoryKind(usize),
    HistoryBaseline(usize),
    HistoryRun(usize),
    Shortcuts,
    ValidationNext,
    TaskDetails,
    TaskContinue,
    TaskLink(usize),
    CloseOptions,
    DiscoveryDetails,
    CloseMappingPreview,
    DiscardPageList,
    ImageControls,
    CopyCaptureDiagnostics,
}
#[derive(Debug, Clone)]
pub struct FocusState {
    pub screen: Screen,
    pub setup_stage: SetupStage,
    pub review_surface: ReviewSurface,
    pub busy: bool,
    pub dialog: bool,
    pub approval_dialog: bool,
    pub projects: usize,
    pub page: usize,
    pub more: bool,
    pub project_saved: bool,
    pub choosing_project: bool,
    pub candidate: bool,
    pub baseline: bool,
    pub viewer: bool,
    pub view_loading: bool,
    pub capturing: bool,
    pub advanced: bool,
    pub preview: bool,
    pub resumable: bool,
    pub pausable: bool,
    pub environments: usize,
    pub mappings: usize,
    pub history_baselines: usize,
    pub history_runs: usize,
}
impl Default for FocusState {
    fn default() -> Self {
        Self {
            screen: Screen::Projects,
            setup_stage: SetupStage::default(),
            review_surface: ReviewSurface::default(),
            busy: true,
            dialog: false,
            approval_dialog: false,
            projects: 0,
            page: 0,
            more: false,
            project_saved: false,
            choosing_project: false,
            candidate: false,
            baseline: false,
            viewer: false,
            view_loading: false,
            capturing: false,
            advanced: false,
            preview: false,
            resumable: false,
            pausable: false,
            environments: 1,
            mappings: 1,
            history_baselines: 0,
            history_runs: 0,
        }
    }
}
impl FocusState {
    pub fn shows_project_picker(&self) -> bool {
        self.screen == Screen::Projects
            || self.choosing_project
            || (matches!(self.screen, Screen::Results | Screen::History) && !self.project_saved)
    }
    pub fn controls(&self) -> Vec<Control> {
        if self.dialog {
            // The native shell filters the repository link out of keyboard help.
            return vec![Control::AboutRepository, Control::CloseDialog];
        }
        if self.approval_dialog {
            return vec![
                Control::Guided(47),
                Control::Guided(48),
                Control::Guided(49),
                Control::Guided(52),
                Control::Phase5(25),
            ];
        }
        // Screen::ALL is the shortcut order, not the painted project rail.
        let mut controls = vec![
            Control::Navigate(Screen::Projects),
            Control::Navigate(Screen::Results),
            Control::PagesBaseline,
            Control::Navigate(Screen::History),
            Control::Navigate(Screen::Setup),
            Control::Navigate(Screen::Settings),
        ];
        if self.busy && self.screen == Screen::Settings {
            // Read-only app information remains available during background reads,
            // alongside keyboard help and in the same order as the footer.
            controls.push(Control::About);
        }
        if self.screen != Screen::Settings || self.busy {
            controls.push(Control::Shortcuts);
        }
        if self.screen == Screen::Results && self.project_saved {
            controls.push(Control::Guided(53));
        }
        if !self.busy && self.project_saved && self.screen != Screen::Projects {
            controls.push(if self.choosing_project {
                Control::CancelProjectChoice
            } else {
                Control::ChooseProject
            });
        }
        controls.push(Control::DismissFeedback);
        if self.busy {
            if self.screen == Screen::History && self.project_saved {
                controls.extend([Control::HistorySearch, Control::HistoryClear]);
                controls.extend((0..4).map(Control::HistoryKind));
                controls.extend((16..=18).map(Control::Phase6));
                controls.extend((0..self.history_baselines.min(128)).map(Control::HistoryBaseline));
                controls.extend((0..self.history_runs.min(64)).map(Control::HistoryRun));
            }
            if self.view_loading && self.screen == Screen::Results {
                controls.push(Control::Viewer);
                controls.extend((0..18).map(Control::Phase5));
                controls.push(Control::Guided(46));
                controls.extend((0..9).map(Control::AnalysisField));
            }
            if self.screen == Screen::Results && self.project_saved {
                controls.extend([
                    Control::CrawlField(9),
                    Control::Guided(17),
                    Control::Guided(19),
                    Control::Guided(45),
                    Control::CloseOptions,
                    Control::DiscoveryDetails,
                ]);
                controls.extend(Choice::RESULTS.into_iter().map(Control::Choice));
            }
            if self.capturing {
                controls.push(Control::CancelCapture);
                if self.pausable {
                    controls.push(Control::PauseCapture);
                }
            }
            return controls;
        }
        if self.shows_project_picker() {
            controls.push(Control::NewProject);
            if self.screen == Screen::Projects {
                controls.extend([0, 1, 3, 6, 7, 14, 15].map(Control::Phase6));
                controls.push(Control::Details(Detail::Portability));
            }
            for index in 0..self.projects {
                controls.push(Control::SelectProject(index));
                if self.screen == Screen::Projects {
                    controls.push(Control::OpenProject(index));
                    controls.push(Control::DeleteProject(index));
                }
            }
            if self.page > 0 {
                controls.push(Control::PreviousPage);
            }
            if self.more {
                controls.push(Control::NextPage);
            }
            return controls;
        }
        match self.screen {
            Screen::Setup => {
                controls.extend([
                    Control::Phase6(6),
                    Control::Details(Detail::Portability),
                    Control::Phase6(7),
                ]);
                controls.extend(SetupStage::ALL.into_iter().map(Control::SetupStage));
                if self.setup_stage == SetupStage::Settings {
                    controls.extend(
                        SettingsSection::ALL
                            .into_iter()
                            .filter(|section| {
                                *section != SettingsSection::Access || self.project_saved
                            })
                            .map(Control::SettingsSection),
                    );
                }
                // Match the rendered form, with disclosed children immediately
                // after their trigger and pinned footer actions last.
                controls.extend([Control::Name, Control::Address]);
                controls.push(Control::Guided(11));
                controls.extend(
                    [
                        CaptureMode::ScreenshotsOnly,
                        CaptureMode::ScreenshotsAndDomSnapshots,
                        CaptureMode::DomSnapshotsOnly,
                    ]
                    .map(Control::CaptureMode),
                );
                controls.extend([1, 2, 3].map(Control::Guided));
                controls.extend([Choice::FullPage, Choice::VisibleScreen].map(Control::Choice));
                controls.extend(
                    [Choice::AddressesOnly, Choice::AddressesAndLinks].map(Control::Choice),
                );
                controls.push(Control::CrawlField(0));
                controls
                    .extend([Choice::OriginalImages, Choice::ReplaceImages].map(Control::Choice));
                controls.push(Control::Guided(6));
                if self.advanced {
                    controls.extend([1, 2].map(Control::CrawlField));
                    controls.extend(
                        [Choice::Gentle, Choice::Balanced, Choice::Fast].map(Control::Choice),
                    );
                    controls.push(Control::Guided(7));
                    controls.extend((3..7).map(Control::CrawlField));
                    controls.extend([8, 9, 10].map(Control::Guided));
                    controls.extend([7, 8].map(Control::CrawlField));
                    controls.extend([12, 25].map(Control::Guided));
                }
                controls.push(Control::Details(Detail::Capture));
                if self.project_saved {
                    controls
                        .extend([Choice::PublicAccess, Choice::LoginRequired].map(Control::Choice));
                    controls.extend((10..13).map(Control::CrawlField));
                    controls
                        .extend([Choice::SaveLogin, Choice::EphemeralLogin].map(Control::Choice));
                    controls.push(Control::EnvironmentOptions);
                    controls.extend((0..self.environments.min(16)).map(Control::Environment));
                    controls.push(Control::Guided(27));
                    if self.environments > 1 {
                        controls.push(Control::RemoveEnvironment);
                    }
                    controls.extend((13..15).map(Control::CrawlField));
                    controls.extend([37, 38].map(Control::Guided));
                    if self.mappings > 1 {
                        controls.extend((0..self.mappings.min(1000)).map(Control::RemoveMapping));
                    }
                    controls.extend([15, 16].map(Control::CrawlField));
                    controls.extend([Choice::LightTheme, Choice::DarkTheme].map(Control::Choice));
                    if self.setup_stage == SetupStage::Settings {
                        controls.extend([31, 32].map(Control::Guided));
                    }
                    controls.extend([33, 34, 35, 36].map(Control::Guided));
                    controls.push(Control::CloseMappingPreview);
                }
                if self.project_saved && self.setup_stage == SetupStage::Settings {
                    controls.push(Control::DeleteCurrentProject);
                }
                controls.push(Control::SetupBack);
                controls.push(Control::Save);
                if self.project_saved && self.setup_stage == SetupStage::Access {
                    controls.extend([31, 32].map(Control::Guided));
                }
                if self.project_saved {
                    controls.push(Control::Guided(13));
                }
                controls.push(Control::ValidationNext);
            }
            Screen::Settings => {
                // App settings follows its rendered groups. The app removes
                // collapsed/disabled children without moving their next sibling.
                controls.extend([
                    Control::SetupRuntime,
                    Control::Details(Detail::Browser),
                    Control::Appearance(AppAppearance::System),
                    Control::Appearance(AppAppearance::Light),
                    Control::Appearance(AppAppearance::Dark),
                    Control::Details(Detail::LocalStorage),
                    Control::Details(Detail::ManagedNetwork),
                    Control::CrawlField(17),
                    Control::CrawlField(18),
                    Control::Guided(42),
                    Control::Guided(41),
                    Control::Details(Detail::Network),
                    Control::ValidationNext,
                    Control::Details(Detail::SupportTools),
                    Control::Phase7(0),
                    Control::Phase7(1),
                    Control::Phase7(2),
                    Control::Phase6(13),
                    Control::Phase6(3),
                    Control::Phase6(1),
                    Control::Phase6(6),
                    Control::Phase6(14),
                    Control::Phase6(15),
                    Control::Details(Detail::Portability),
                    Control::Phase6(7),
                    Control::About,
                    Control::Shortcuts,
                ]);
            }
            Screen::Results => {
                if self.project_saved {
                    // Follow the workspace's reading order, not control-type
                    // buckets: heading, full-width toolbar, queue, evidence /
                    // inspector, review strip, then the baseline footer.
                    if self.candidate {
                        controls.push(Control::SaveBaseline);
                    }
                    controls.push(Control::ReviewCaptureFailures);
                    if self.baseline {
                        controls.push(Control::CheckChanges);
                    }
                    controls.extend([
                        Control::CaptureBaseline,
                        Control::Guided(16),
                        Control::Phase5(30),
                        Control::RefreshEvidence,
                        Control::Guided(51),
                    ]);
                    controls.extend(
                        [CheckKind::Screenshot, CheckKind::DomSnapshot].map(Control::EvidenceKind),
                    );
                    controls.extend([Control::Guided(50), Control::Guided(54)]);
                    // View's popup is arrow-key navigated; Tab continues from
                    // its trigger. Keep its candidates in painted menu order.
                    controls.extend((6..10).map(Control::Phase5));
                    controls.extend([
                        Control::Guided(55),
                        Control::SplitView,
                        Control::Guided(57), // Before, then After.
                        Control::Guided(56),
                    ]);
                    if self.viewer {
                        controls.push(Control::Guided(46));
                    }
                    controls.extend([
                        Control::ImageControls,
                        Control::Phase5(36),
                        Control::Guided(44),
                        Control::ShowAllResults,
                        Control::CrawlField(9),
                        Control::Guided(17),
                        Control::Guided(45),
                        Control::Guided(23),
                        Control::Guided(24),
                    ]);
                    if self.resumable {
                        controls.push(Control::Guided(15));
                    }
                    // Options overlay the evidence, with Close at the top.
                    controls.push(Control::CloseOptions);
                    controls.extend(Choice::RESULTS.into_iter().map(Control::Choice));
                    controls.push(Control::Guided(19));
                    if self.preview {
                        controls.extend([22, 20, 21].map(Control::Guided));
                        controls.extend([Control::DiscoveryDetails, Control::DiscardPageList]);
                    }
                    if self.viewer {
                        controls.extend([Control::Viewer, Control::SplitDivider]);
                    }
                    controls.extend((0..3).map(Control::DomSource));
                    controls.extend((0..10).map(Control::DomViewer));
                    controls.extend([10, 11].map(Control::Phase5));
                    controls.push(Control::Phase5(39));
                    // Mutually exclusive inspectors share Close, then their
                    // own top-to-bottom contents and pinned footer action.
                    controls.extend([Control::Phase5(38), Control::CopyCaptureDiagnostics]);
                    controls.extend([Control::AnalysisField(9), Control::Phase5(32)]);
                    controls.extend([0, 1, 2, 4, 5, 26].map(Control::Phase5));
                    controls.push(Control::Details(Detail::IgnoredAreas));
                    controls.extend([14, 34].map(Control::Phase5));
                    controls.extend((10..14).map(Control::AnalysisField));
                    controls.extend([35, 18, 15, 17, 16, 19, 3].map(Control::Phase5));
                    controls.extend((0..9).map(Control::AnalysisField));
                    controls.extend([27, 28].map(Control::Phase5));
                    controls.push(Control::Details(Detail::Analysis));
                    controls.extend([12, 13].map(Control::Phase5));
                    controls.extend((20..23).map(Control::Phase5));
                    controls.push(Control::ReviewDetails);
                    controls.extend([37, 33, 23, 24, 25].map(Control::Phase5));
                }
            }
            Screen::History if self.project_saved => {
                controls.extend([
                    Control::HistorySearch,
                    Control::HistoryClear,
                    Control::HistoryRefresh,
                ]);
                controls.extend((0..self.history_baselines.min(128)).map(Control::HistoryBaseline));
                controls.extend((0..4).map(Control::HistoryKind));
                controls.extend((0..self.history_runs.min(64)).map(Control::HistoryRun));
                controls.extend((40..43).map(Control::Phase5));
                controls.extend((0..13).chain(14..20).map(Control::Phase6));
                controls.push(Control::Details(Detail::History));
                controls.push(Control::Details(Detail::Portability));
            }
            _ => (),
        }
        if self.screen == Screen::Setup {
            controls.retain(|control| {
                matches!(
                    control,
                    Control::Navigate(_)
                        | Control::Shortcuts
                        | Control::SetupStage(_)
                        | Control::PagesBaseline
                        | Control::ChooseProject
                        | Control::DismissFeedback
                ) || self.setup_stage.mounts(*control)
            });
        }
        controls
    }
    pub fn next(&self, current: Control, reverse: bool) -> Option<Control> {
        let controls = self.controls();
        if controls.is_empty() {
            return None;
        }
        let i = controls.iter().position(|c| *c == current).unwrap_or(0);
        Some(
            controls[if reverse {
                (i + controls.len() - 1) % controls.len()
            } else {
                (i + 1) % controls.len()
            }],
        )
    }
    pub fn can_activate(&self, control: Control) -> bool {
        self.controls().contains(&control)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn removal_actions_follow_their_objects_and_protect_last_item_focus() {
        let mut state = FocusState {
            screen: Screen::Setup,
            busy: false,
            project_saved: true,
            environments: 2,
            mappings: 2,
            ..Default::default()
        };
        assert_eq!(
            state.next(Control::Guided(27), false),
            Some(Control::RemoveEnvironment)
        );
        assert_eq!(
            state.next(Control::RemoveEnvironment, false),
            Some(Control::CrawlField(13))
        );
        assert_eq!(
            state.next(Control::Guided(38), false),
            Some(Control::RemoveMapping(0))
        );
        assert_eq!(
            state.next(Control::RemoveMapping(1), false),
            Some(Control::CrawlField(15))
        );
        state.environments = 1;
        state.mappings = 1;
        assert!(!state.can_activate(Control::RemoveEnvironment));
        assert!(!state.can_activate(Control::RemoveMapping(0)));
        state.busy = true;
        assert!(!state.can_activate(Control::DeleteCurrentProject));
    }
    #[test]
    fn review_candidates_follow_rail_toolbar_queue_evidence_and_footer_order() {
        let state = FocusState {
            screen: Screen::Results,
            busy: false,
            project_saved: true,
            baseline: true,
            viewer: true,
            ..Default::default()
        };
        let expected = [
            Control::Navigate(Screen::Projects),
            Control::Navigate(Screen::Results),
            Control::PagesBaseline,
            Control::Navigate(Screen::History),
            Control::Navigate(Screen::Setup),
            Control::Navigate(Screen::Settings),
            Control::Guided(53),
            Control::ChooseProject,
            Control::CheckChanges,
            Control::RefreshEvidence,
            Control::Guided(50),
            Control::Guided(54),
            Control::Guided(57),
            Control::Guided(56),
            Control::Guided(46),
            Control::ImageControls,
            Control::Phase5(36),
            Control::Guided(44),
            Control::CrawlField(9),
            Control::Guided(17),
            Control::Guided(45),
            Control::Guided(23),
            Control::Guided(24),
            Control::Viewer,
            Control::SplitDivider,
            Control::Phase5(20),
            Control::Phase5(21),
            Control::Phase5(22),
            Control::Phase5(37),
            Control::Phase5(33),
            Control::Phase5(23),
            Control::Phase5(24),
        ];
        let actual: Vec<_> = state
            .controls()
            .into_iter()
            .filter(|c| expected.contains(c))
            .collect();
        assert_eq!(actual, expected);
    }

    #[test]
    fn busy_settings_keep_about_and_keyboard_help_without_unlocking_mutations() {
        let mut state = FocusState {
            screen: Screen::Settings,
            busy: true,
            ..Default::default()
        };
        for busy in [true, false] {
            state.busy = busy;
            assert!(state.can_activate(Control::About));
            assert!(state.can_activate(Control::Shortcuts));
            assert_eq!(state.next(Control::About, false), Some(Control::Shortcuts));
            assert_eq!(state.next(Control::Shortcuts, true), Some(Control::About));
        }
        state.busy = true;
        for control in [
            Control::SetupRuntime,
            Control::Save,
            Control::Guided(41),
            Control::CaptureBaseline,
            Control::Phase6(14),
        ] {
            assert!(!state.can_activate(control), "{control:?}");
        }
        state.dialog = true;
        assert!(!state.can_activate(Control::About));
        assert!(!state.can_activate(Control::Shortcuts));
    }

    #[test]
    fn history_rows_are_bounded_distinct_and_modal_safe() {
        let mut state = FocusState {
            screen: Screen::History,
            busy: false,
            project_saved: true,
            history_baselines: 128,
            history_runs: 64,
            ..Default::default()
        };
        assert!(state.can_activate(Control::HistoryBaseline(127)));
        assert!(state.can_activate(Control::HistoryRun(63)));
        assert!(!state.can_activate(Control::HistoryBaseline(128)));
        assert!(!state.can_activate(Control::HistoryRun(64)));
        assert!(state.can_activate(Control::HistorySearch));
        assert!(state.can_activate(Control::HistoryRefresh));
        state.busy = true;
        // Cached row selection and filtering are local; another storage read
        // is not. The native exclusive-task layer separately traps all focus.
        assert!(state.can_activate(Control::HistoryBaseline(0)));
        assert!(state.can_activate(Control::HistorySearch));
        assert!(state.can_activate(Control::Phase6(16)));
        assert!(!state.can_activate(Control::HistoryRefresh));
        state.busy = false;
        state.dialog = true;
        assert_eq!(
            state.controls(),
            [Control::AboutRepository, Control::CloseDialog]
        );
    }
    #[test]
    fn approval_decision_traps_navigation_and_mutations_until_dismissed() {
        let mut state = FocusState {
            screen: Screen::Results,
            busy: false,
            project_saved: true,
            approval_dialog: true,
            ..Default::default()
        };
        assert_eq!(
            state.controls(),
            [
                Control::Guided(47),
                Control::Guided(48),
                Control::Guided(49),
                Control::Guided(52),
                Control::Phase5(25)
            ]
        );
        assert!(!state.can_activate(Control::Navigate(Screen::Projects)));
        assert!(!state.can_activate(Control::Phase5(23)));
        assert!(!state.can_activate(Control::Phase5(26)));
        assert!(!state.can_activate(Control::Save));
        state.approval_dialog = false;
        assert!(state.can_activate(Control::Navigate(Screen::Projects)));
        assert!(state.can_activate(Control::Phase5(23)));
    }
    #[test]
    fn guided_stages_only_offer_their_actual_form_controls() {
        let mut state = FocusState {
            screen: Screen::Setup,
            busy: false,
            project_saved: true,
            setup_stage: SetupStage::Website,
            ..Default::default()
        };
        assert!(state.can_activate(Control::Name));
        assert!(state.can_activate(Control::Save));
        assert!(!state.can_activate(Control::CrawlField(0)));
        assert!(!state.can_activate(Control::Guided(31)));
        state.setup_stage = SetupStage::Access;
        assert!(!state.can_activate(Control::Name));
        assert!(state.can_activate(Control::Choice(Choice::PublicAccess)));
        assert!(state.can_activate(Control::Guided(31)));
        assert!(!state.can_activate(Control::Guided(13)));
        state.setup_stage = SetupStage::Pages;
        assert!(state.can_activate(Control::CrawlField(0)));
        assert!(state.can_activate(Control::Guided(13)));
        assert!(!state.can_activate(Control::Guided(31)));
        assert!(!state.can_activate(Control::Address));
        state.setup_stage = SetupStage::Settings;
        assert!(state.can_activate(Control::Address));
        assert!(state.can_activate(Control::Guided(31)));
    }
    #[test]
    fn results_and_history_offer_project_selection_without_opening_settings() {
        for screen in [Screen::Results, Screen::History] {
            let state = FocusState {
                screen,
                busy: false,
                projects: 64,
                page: 1,
                more: true,
                ..Default::default()
            };
            assert!(state.shows_project_picker());
            for control in [
                Control::NewProject,
                Control::SelectProject(63),
                Control::PreviousPage,
                Control::NextPage,
            ] {
                assert!(state.can_activate(control));
            }
            assert!(!state.can_activate(Control::OpenProject(0)));
            assert!(!state.can_activate(Control::CaptureBaseline));
            assert_eq!(
                state.next(Control::SelectProject(0), false),
                Some(Control::SelectProject(1))
            );
        }
    }
    #[test]
    fn project_picker_replaces_screen_controls_and_locks_selection_during_work() {
        let mut state = FocusState {
            screen: Screen::Results,
            busy: false,
            project_saved: true,
            projects: 2,
            ..Default::default()
        };
        assert!(state.can_activate(Control::ChooseProject));
        assert!(!state.can_activate(Control::SelectProject(0)));
        state.choosing_project = true;
        assert!(state.can_activate(Control::CancelProjectChoice));
        assert!(state.can_activate(Control::SelectProject(0)));
        assert!(!state.can_activate(Control::CaptureBaseline));
        state.busy = true;
        assert!(!state.can_activate(Control::SelectProject(0)));
        assert!(!state.can_activate(Control::CancelProjectChoice));
        assert!(state.can_activate(Control::Navigate(Screen::Projects)));
    }
    #[test]
    fn environment_fields_and_timezone_follow_the_visible_form_order() {
        let state = FocusState {
            screen: Screen::Setup,
            busy: false,
            project_saved: true,
            ..Default::default()
        };
        for (before, after) in [
            (
                Control::Choice(Choice::LoginRequired),
                Control::CrawlField(10),
            ),
            (Control::CrawlField(12), Control::Choice(Choice::SaveLogin)),
            (
                Control::Choice(Choice::EphemeralLogin),
                Control::EnvironmentOptions,
            ),
            (Control::Environment(0), Control::Guided(27)),
            (Control::Guided(27), Control::CrawlField(13)),
            (Control::Guided(38), Control::CrawlField(15)),
            (Control::CrawlField(15), Control::CrawlField(16)),
            (Control::CrawlField(16), Control::Choice(Choice::LightTheme)),
            (Control::Choice(Choice::DarkTheme), Control::Guided(31)),
        ] {
            assert_eq!(state.next(before, false), Some(after));
            assert_eq!(state.next(after, true), Some(before));
        }
    }
    #[test]
    fn settings_and_advanced_capture_candidates_follow_the_painted_sequence() {
        let mut state = FocusState {
            screen: Screen::Setup,
            busy: false,
            project_saved: true,
            advanced: true,
            ..Default::default()
        };
        for (before, after) in [
            (
                Control::SettingsSection(SettingsSection::Access),
                Control::Name,
            ),
            (
                Control::Guided(11),
                Control::CaptureMode(CaptureMode::ScreenshotsOnly),
            ),
            (
                Control::CaptureMode(CaptureMode::DomSnapshotsOnly),
                Control::Guided(1),
            ),
            (Control::Guided(3), Control::Choice(Choice::FullPage)),
            (
                Control::Choice(Choice::VisibleScreen),
                Control::Choice(Choice::AddressesOnly),
            ),
            (
                Control::CrawlField(0),
                Control::Choice(Choice::OriginalImages),
            ),
            (Control::Guided(6), Control::CrawlField(1)),
            (Control::CrawlField(2), Control::Choice(Choice::Gentle)),
            (Control::Choice(Choice::Fast), Control::Guided(7)),
            (Control::CrawlField(6), Control::Guided(8)),
            (Control::Guided(10), Control::CrawlField(7)),
            (Control::Guided(25), Control::Details(Detail::Capture)),
            (Control::Guided(13), Control::ValidationNext),
            (
                Control::Details(Detail::Capture),
                Control::Choice(Choice::PublicAccess),
            ),
        ] {
            assert_eq!(state.next(before, false), Some(after));
            assert_eq!(state.next(after, true), Some(before));
        }
        state.setup_stage = SetupStage::Pages;
        assert!(!state.can_activate(Control::SettingsSection(SettingsSection::Capture)));
        state.setup_stage = SetupStage::Settings;
        state.busy = true;
        assert!(!state.can_activate(Control::SettingsSection(SettingsSection::Website)));
    }
    #[test]
    fn focus_wraps_and_disabled_actions_are_not_activatable() {
        let mut s = FocusState {
            busy: false,
            screen: Screen::Setup,
            ..Default::default()
        };
        assert_eq!(s.next(Control::Save, false), Some(Control::ValidationNext));
        assert_eq!(
            s.next(Control::ValidationNext, false),
            Some(Control::Navigate(Screen::Projects))
        );
        assert_eq!(
            s.next(Control::Navigate(Screen::Projects), true),
            Some(Control::ValidationNext)
        );
        s.busy = true;
        assert!(!s.can_activate(Control::Save));
        assert!(s.can_activate(Control::Navigate(Screen::Settings)));
    }
    #[test]
    fn modal_traps_focus_and_restores_normal_order() {
        let mut s = FocusState {
            busy: false,
            dialog: true,
            ..Default::default()
        };
        assert_eq!(
            s.controls(),
            [Control::AboutRepository, Control::CloseDialog]
        );
        assert!(!s.can_activate(Control::NewProject));
        s.dialog = false;
        assert!(s.can_activate(Control::NewProject));
    }
    #[test]
    fn busy_results_keep_filters_focusable_without_enabling_mutations() {
        let mut s = FocusState {
            screen: Screen::Results,
            project_saved: true,
            capturing: true,
            pausable: true,
            ..Default::default()
        };
        for control in [
            Control::CrawlField(9),
            Control::Guided(17),
            Control::Choice(Choice::ChangedResults),
            Control::Guided(19),
            Control::CancelCapture,
            Control::PauseCapture,
        ] {
            assert!(s.can_activate(control));
        }
        assert_eq!(
            s.next(Control::CrawlField(9), false),
            Some(Control::Guided(17))
        );
        for control in [
            Control::Save,
            Control::SaveBaseline,
            Control::CheckChanges,
            Control::CaptureBaseline,
            Control::Guided(14),
        ] {
            assert!(!s.can_activate(control));
        }
        s.dialog = true;
        assert_eq!(
            s.controls(),
            [Control::AboutRepository, Control::CloseDialog]
        );
        s.dialog = false;
        s.screen = Screen::Setup;
        assert!(!s.can_activate(Control::CrawlField(9)));
    }
    #[test]
    fn hidden_screen_controls_are_not_focusable() {
        let s = FocusState {
            busy: false,
            screen: Screen::Results,
            ..Default::default()
        };
        assert!(!s.can_activate(Control::Name));
    }
    #[test]
    fn screen_focus_candidates_never_duplicate_controls() {
        for screen in Screen::ALL {
            for busy in [false, true] {
                let state = FocusState {
                    screen,
                    busy,
                    projects: 64,
                    project_saved: true,
                    baseline: true,
                    candidate: true,
                    viewer: true,
                    view_loading: true,
                    capturing: true,
                    advanced: true,
                    preview: true,
                    resumable: true,
                    pausable: true,
                    environments: 16,
                    ..Default::default()
                };
                let controls = state.controls();
                for (index, control) in controls.iter().enumerate() {
                    assert!(
                        !controls[index + 1..].contains(control),
                        "{screen:?}: duplicate {control:?}"
                    );
                }
            }
        }
    }
}
