// SPDX-License-Identifier: MPL-2.0
//! One semantic map for recurring labelled actions. Never infer meaning from a
//! translated label or replace explicit save/delete/approval wording with a glyph.
use super::*;

impl Shell {
    pub(super) fn action_icon(&self, control: Control, cx: &App) -> Option<ToolIcon> {
        use Control::*;
        Some(match control {
            Navigate(Screen::Projects) => ToolIcon::Folder,
            Navigate(Screen::Results) | CheckChanges => ToolIcon::Compare,
            Navigate(Screen::History) => ToolIcon::Runs,
            Navigate(Screen::Setup | Screen::Settings) | OpenProject(_) => ToolIcon::Adjustments,
            PagesBaseline => ToolIcon::Baselines,
            BaselineUpdate => ToolIcon::Baselines,
            SettingsSection(self::SettingsSection::Website) => ToolIcon::Globe,
            SettingsSection(self::SettingsSection::Capture) => ToolIcon::Camera,
            SettingsSection(self::SettingsSection::Access) => ToolIcon::Lock,
            EvidenceKind(regressee_core::domain::CheckKind::Screenshot) => ToolIcon::Image,
            EvidenceKind(regressee_core::domain::CheckKind::DomSnapshot) => ToolIcon::Source,
            Phase6(16) => ToolIcon::Runs,
            Phase6(17) => ToolIcon::Baselines,
            Phase6(18) => ToolIcon::Storage,
            NewProject => ToolIcon::FolderPlus,
            SelectProject(_) => match self.focus.screen {
                Screen::History => ToolIcon::Runs,
                Screen::Setup => ToolIcon::Adjustments,
                _ => ToolIcon::Compare,
            },
            ChooseProject => ToolIcon::Swap,
            CancelProjectChoice | CloseDialog | CloseOptions | CloseMappingPreview => {
                ToolIcon::Close
            }
            Save if !self.unsaved(cx)
                && (self.focus.setup_stage == super::SetupStage::Website
                    || (self.focus.setup_stage == super::SetupStage::Access
                        && self.access_can_continue(cx))) =>
            {
                ToolIcon::Forward
            }
            Save | SaveBaseline | Guided(41) | Phase5(18 | 32) => ToolIcon::Save,
            SetupBack | PreviousPage | Guided(48) | Phase5(12) => ToolIcon::Previous,
            NextPage | Guided(49) => ToolIcon::Next,
            Guided(23) => ToolIcon::Up,
            Guided(24) => ToolIcon::Down,
            CaptureBaseline if self.preview.is_none() => ToolIcon::Zoom,
            CaptureBaseline => ToolIcon::Camera,
            Guided(13) => ToolIcon::Zoom,
            SetupRuntime => ToolIcon::Download,
            AgentInstall(_) => ToolIcon::Download,
            AgentRemove(_) => ToolIcon::Trash,
            AgentRefresh | AgentRetry(_) => ToolIcon::Refresh,
            About => ToolIcon::Help,
            Shortcuts => ToolIcon::Keyboard,
            ReviewDetails | DiscoveryDetails | Phase7(0) => ToolIcon::Details,
            ReviewCaptureFailures => ToolIcon::Flag,
            Phase5(20) => ToolIcon::Unreviewed,
            Phase5(21 | 35) | Guided(32) => ToolIcon::Check,
            Phase5(22) => ToolIcon::Flag,
            Phase5(37) => ToolIcon::Note,
            Phase5(13 | 33) | TaskContinue => ToolIcon::Forward,
            Phase5(23)
                if self
                    .selected_key
                    .as_ref()
                    .is_some_and(|key| self.approval_selection.contains(key)) =>
            {
                ToolIcon::Close
            }
            Phase5(23..=25) => ToolIcon::Baselines,
            Phase5(14) => ToolIcon::Draw,
            Phase5(39) | DismissFeedback | Phase6(7) => ToolIcon::Close,
            Phase5(30) => ToolIcon::Image,
            Phase5(34) => ToolIcon::Coordinates,
            Guided(31) => ToolIcon::Globe,
            Guided(42) | RefreshEvidence | HistoryRefresh | Phase5(4) => ToolIcon::Refresh,
            Phase6(0 | 1) => ToolIcon::Upload,
            Phase6(2 | 3) => ToolIcon::Download,
            Phase6(4..=6) => ToolIcon::Trash,
            DeleteProject(_) | DeleteCurrentProject | RemoveEnvironment | RemoveMapping(_) => {
                ToolIcon::Trash
            }
            Phase6(19) => {
                let check = self
                    .history
                    .as_ref()
                    .and_then(|h| h.runs.get(self.run_history_index))
                    .is_some_and(|r| matches!(r.run.intent, RunIntent::Check { .. }));
                if check {
                    ToolIcon::Compare
                } else {
                    ToolIcon::Image
                }
            }
            Phase6(13) => ToolIcon::Storage,
            CopyCaptureDiagnostics => ToolIcon::Copy,
            _ => return None,
        })
    }
}
