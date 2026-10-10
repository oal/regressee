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
            Self::Setup => "Project setup",
            Self::Results => "Results",
            Self::History => "History",
            Self::Settings => "Settings",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Control {
    Navigate(Screen),
    NewProject,
    Name,
    Address,
    Save,
    OpenProject(usize),
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
}
#[derive(Debug, Clone)]
pub struct FocusState {
    pub screen: Screen,
    pub busy: bool,
    pub dialog: bool,
    pub projects: usize,
    pub page: usize,
    pub more: bool,
    pub project_saved: bool,
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
}
impl Default for FocusState {
    fn default() -> Self {
        Self {
            screen: Screen::Projects,
            busy: true,
            dialog: false,
            projects: 0,
            page: 0,
            more: false,
            project_saved: false,
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
        }
    }
}
impl FocusState {
    pub fn controls(&self) -> Vec<Control> {
        if self.dialog {
            return vec![Control::CloseDialog];
        }
        let mut controls: Vec<_> = Screen::ALL.into_iter().map(Control::Navigate).collect();
        controls.push(Control::DismissFeedback);
        if self.busy {
            if self.view_loading && self.screen == Screen::Results {
                controls.extend([
                    Control::ZoomOut,
                    Control::ZoomIn,
                    Control::PanLeft,
                    Control::PanRight,
                    Control::PanUp,
                    Control::PanDown,
                ]);
                controls.extend((0..18).map(Control::Phase5));
                controls.extend((0..9).map(Control::AnalysisField));
            }
            if self.screen == Screen::Results && self.project_saved {
                controls.extend([
                    Control::CrawlField(9),
                    Control::Guided(17),
                    Control::Guided(19),
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
        match self.screen {
            Screen::Projects => {
                controls.push(Control::NewProject);
                controls.extend([0, 1, 3, 6, 7, 14, 15].map(Control::Phase6));
                controls.extend((0..self.projects).map(Control::OpenProject));
                if self.page > 0 {
                    controls.push(Control::PreviousPage);
                }
                if self.more {
                    controls.push(Control::NextPage);
                }
            }
            Screen::Setup => {
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
                }
                controls.push(Control::Details(Detail::Capture));
                controls.push(Control::Save);
                if self.project_saved {
                    controls.push(Control::Guided(13));
                }
            }
            Screen::Settings => {
                controls.extend((0..3).map(Control::Phase7));
                controls.extend([1, 3, 6, 7, 13, 14, 15].map(Control::Phase6));
                controls.extend([Control::SetupRuntime, Control::ChooseFolder, Control::About]);
                controls.extend([
                    Control::CrawlField(17),
                    Control::CrawlField(18),
                    Control::Guided(41),
                    Control::Guided(42),
                ]);
                controls.push(Control::Details(Detail::Network));
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
                    controls.extend((0..29).map(Control::Phase5));
                    controls.push(Control::Details(Detail::Analysis));
                    controls.extend((0..10).map(Control::AnalysisField));
                }
                if self.candidate {
                    controls.push(Control::SaveBaseline);
                }
                if self.baseline {
                    controls.push(Control::CheckChanges);
                }
                if self.viewer {
                    controls.extend([
                        Control::ZoomOut,
                        Control::ZoomIn,
                        Control::PanLeft,
                        Control::PanRight,
                        Control::PanUp,
                        Control::PanDown,
                    ]);
                }
            }
            Screen::History if self.project_saved => {
                controls.extend((40..43).map(Control::Phase5));
                controls.extend((0..13).chain(14..16).map(Control::Phase6));
                controls.push(Control::Details(Detail::History));
            }
            _ => (),
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
