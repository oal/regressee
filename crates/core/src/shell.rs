// SPDX-License-Identifier: MPL-2.0
//! Testable navigation/focus rules shared by the native shell.
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
    Network,
    Portability,
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
        if matches!(control, ValidationNext) {
            return true;
        }
        match self {
            Self::Settings => true,
            Self::Website => matches!(control, Name | Address | Guided(11) | Save),
            Self::Access => matches!(
                control,
                Environment(_)
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
                matches!(control, Guided(1..=3 | 6..=10 | 12 | 13 | 25) | CrawlField(0..=8) | Choice(self::Choice::AddressesOnly | self::Choice::AddressesAndLinks | self::Choice::FullPage | self::Choice::VisibleScreen | self::Choice::Gentle | self::Choice::Balanced | self::Choice::Fast | self::Choice::OriginalImages | self::Choice::ReplaceImages) | Details(Detail::Capture) | Save)
            }
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
    Navigate(Screen),
    SetupStage(SetupStage),
    PagesBaseline,
    NewProject,
    Name,
    Address,
    Save,
    OpenProject(usize),
    SelectProject(usize),
    ChooseProject,
    CancelProjectChoice,
    PreviousPage,
    NextPage,
    ChooseFolder,
    About,
    CloseDialog,
    CaptureBaseline,
    SaveBaseline,
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
            return vec![Control::CloseDialog];
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
        let mut controls: Vec<_> = Screen::ALL.into_iter().map(Control::Navigate).collect();
        controls.push(Control::Shortcuts);
        controls.push(Control::PagesBaseline);
        if !self.busy && self.project_saved && self.screen != Screen::Projects {
            controls.push(if self.choosing_project {
                Control::CancelProjectChoice
            } else {
                Control::ChooseProject
            });
        }
        controls.push(Control::DismissFeedback);
        if self.busy && self.screen == Screen::Results && self.project_saved {
            controls.push(Control::Guided(53));
        }
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
                controls.extend(SetupStage::ALL.into_iter().map(Control::SetupStage));
                controls.extend([Control::Name, Control::Address]);
                controls.push(Control::Guided(11));
                controls.extend(
                    [Choice::AddressesOnly, Choice::AddressesAndLinks].map(Control::Choice),
                );
                controls.extend((0..3).map(Control::CrawlField));
                controls.extend([1, 2, 3].map(Control::Guided));
                controls.extend(
                    [
                        Choice::FullPage,
                        Choice::VisibleScreen,
                        Choice::OriginalImages,
                        Choice::ReplaceImages,
                        Choice::Gentle,
                        Choice::Balanced,
                        Choice::Fast,
                    ]
                    .map(Control::Choice),
                );
                controls.extend([7, 6].map(Control::Guided));
                if self.advanced {
                    controls.extend((3..9).map(Control::CrawlField));
                    controls.extend([8, 9, 10, 12].map(Control::Guided));
                    controls.push(Control::Guided(25));
                }
                if self.project_saved {
                    controls.extend((0..self.environments.min(16)).map(Control::Environment));
                    controls.push(Control::Guided(27));
                    controls
                        .extend([Choice::PublicAccess, Choice::LoginRequired].map(Control::Choice));
                    controls.extend((10..13).map(Control::CrawlField));
                    controls
                        .extend([Choice::SaveLogin, Choice::EphemeralLogin].map(Control::Choice));
                    controls.extend((13..15).map(Control::CrawlField));
                    controls.extend([37, 38].map(Control::Guided));
                    controls.extend([15, 16].map(Control::CrawlField));
                    controls.extend([Choice::LightTheme, Choice::DarkTheme].map(Control::Choice));
                    controls.extend([31, 32, 33, 34, 35, 36].map(Control::Guided));
                    controls.push(Control::CloseMappingPreview);
                }
                controls.push(Control::Details(Detail::Capture));
                controls.push(Control::ValidationNext);
                controls.push(Control::Save);
                if self.project_saved {
                    controls.push(Control::Guided(13));
                }
            }
            Screen::Settings => {
                controls.extend((0..3).map(Control::Phase7));
                controls.extend([1, 3, 6, 7, 13, 14, 15].map(Control::Phase6));
                controls.push(Control::Details(Detail::Portability));
                controls.extend([Control::SetupRuntime, Control::ChooseFolder, Control::About]);
                controls.extend([
                    Control::CrawlField(17),
                    Control::CrawlField(18),
                    Control::Guided(41),
                    Control::Guided(42),
                ]);
                controls.push(Control::Details(Detail::Network));
                controls.push(Control::ValidationNext);
            }
            Screen::Results => {
                if self.project_saved {
                    controls.push(Control::Phase5(30));
                    controls.extend([Control::CaptureBaseline, Control::RefreshEvidence]);
                    controls.extend([
                        Control::Guided(16),
                        Control::Guided(17),
                        Control::Guided(19),
                        Control::Guided(23),
                        Control::Guided(24),
                    ]);
                    controls.extend(Choice::RESULTS.into_iter().map(Control::Choice));
                    if self.preview {
                        controls.extend([
                            Control::Guided(20),
                            Control::Guided(21),
                            Control::Guided(22),
                        ]);
                    }
                    if self.resumable {
                        controls.push(Control::Guided(15));
                    }
                    controls.push(Control::CrawlField(9));
                    // Analysis controls can be mounted without image panes;
                    // the app filters these candidates by the actual sections.
                    controls.extend((0..29).chain(32..40).map(Control::Phase5));
                    controls.push(Control::Guided(44));
                    controls.push(Control::Guided(45));
                    controls.extend([Control::CloseOptions, Control::DiscoveryDetails]);
                    controls.extend([50, 51, 53, 54, 55, 56, 57].map(Control::Guided));
                    controls.push(Control::Details(Detail::Analysis));
                    controls.extend((0..14).map(Control::AnalysisField));
                }
                if self.candidate {
                    controls.push(Control::SaveBaseline);
                }
                if self.baseline {
                    controls.push(Control::CheckChanges);
                }
                if self.viewer {
                    controls.push(Control::Guided(46));
                    controls.push(Control::Viewer);
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
        if self.screen == Screen::Results {
            controls.sort_by_key(|control| match control {
                Control::Guided(53) => 0,
                Control::Navigate(_)
                | Control::ChooseProject
                | Control::CancelProjectChoice
                | Control::PagesBaseline
                | Control::DismissFeedback => 1,
                Control::Guided(50 | 51 | 45 | 16..=24)
                | Control::CrawlField(9)
                | Control::Choice(_)
                | Control::CloseOptions
                | Control::DiscoveryDetails => 2,
                Control::Guided(54) => 3,
                Control::Phase5(6..=9) | Control::Guided(55 | 56 | 57) => 4,
                Control::Guided(46)
                | Control::Phase5(36)
                | Control::Guided(44)
                | Control::Viewer => 5,
                Control::Phase5(20..=22) => 7,
                Control::Phase5(37 | 33) => 8,
                Control::Phase5(23..=25) => 9,
                _ => 6,
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
        assert_eq!(state.controls(), vec![Control::CloseDialog]);
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
                Control::CrawlField(13),
            ),
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
    fn focus_wraps_and_disabled_actions_are_not_activatable() {
        let mut s = FocusState {
            busy: false,
            screen: Screen::Setup,
            ..Default::default()
        };
        assert_eq!(
            s.next(Control::Save, false),
            Some(Control::Navigate(Screen::Projects))
        );
        assert_eq!(
            s.next(Control::Navigate(Screen::Projects), true),
            Some(Control::Save)
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
        assert_eq!(s.controls(), [Control::CloseDialog]);
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
        assert_eq!(s.controls(), [Control::CloseDialog]);
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
