// SPDX-License-Identifier: MPL-2.0
//! Shared control roles. Arrow navigation moves focus only: it must never save a
//! review decision, change a draft or select a baseline reference implicitly.
use super::*;
use regressee_core::domain::CheckKind;

fn selection_group(control: Control) -> Vec<Control> {
    use Control::*;
    match control {
        EvidenceKind(_) => vec![
            EvidenceKind(CheckKind::Screenshot),
            EvidenceKind(CheckKind::DomSnapshot),
        ],
        DomSource(_) => (0..3).map(DomSource).collect(),
        Phase5(20..=22) => (20..=22).map(Phase5).collect(),
        Phase5(0..=2) => (0..=2).map(Phase5).collect(),
        Phase6(16..=18) => (16..=18).map(Phase6).collect(),
        HistoryKind(_) => (0..4).map(HistoryKind).collect(),
        Guided(56 | 57) => vec![Guided(57), Guided(56)],
        SettingsSection(_) => regressee_core::shell::SettingsSection::ALL
            .into_iter()
            .map(SettingsSection)
            .collect(),
        _ => Vec::new(),
    }
}

pub(super) fn selection_destination(
    control: Control,
    key: &str,
    enabled: impl Fn(Control) -> bool,
) -> Option<Control> {
    let group: Vec<_> = selection_group(control)
        .into_iter()
        .filter(|&item| enabled(item))
        .collect();
    let current = group.iter().position(|&item| item == control)?;
    let index = match key {
        "left" => (current + group.len() - 1) % group.len(),
        "right" => (current + 1) % group.len(),
        "home" => 0,
        "end" => group.len() - 1,
        _ => return None,
    };
    Some(group[index])
}

impl Shell {
    pub(super) fn selection_key(
        &mut self,
        control: Control,
        event: &gpui::KeyDownEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let modifiers = event.keystroke.modifiers;
        if modifiers.control
            || modifiers.platform
            || modifiers.alt
            || modifiers.shift
            || !self.control_enabled(control, cx)
            || !self.handle(control, cx).is_focused(window)
        {
            return;
        }
        if let Some(next) = selection_destination(control, &event.keystroke.key, |item| {
            self.control_enabled(item, cx)
        }) {
            self.reveal_form_control(next, false, window, cx);
            self.reveal_history_control(next);
            self.reveal_review_control(next, window, cx);
            window.focus(&self.handle(next, cx));
            cx.stop_propagation();
            cx.notify();
        }
    }

    pub(super) fn toggle_control(
        &self,
        control: Control,
        label: impl Into<SharedString>,
        checked: bool,
        cx: &mut Context<Self>,
    ) -> AnyElement {
        let element = controls::toggle(
            SharedString::from(format!("{control:?}")),
            label,
            &self.handle(control, cx),
            checked,
            self.control_enabled(control, cx),
        );
        self.wire_action_control(control, element, cx)
            .into_any_element()
    }

    /// Reserve room for the longest view/side selectors before adding tool text.
    /// DOM has no image/zoom tools, so its small toolbar can retain labels sooner.
    pub(super) fn inspection_tool_label<'a>(&self, label: &'a str) -> Option<&'a str> {
        (self.working_width >= if self.dom_active() { 640. } else { 900. }).then_some(label)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exclusive_groups_wrap_skip_disabled_and_do_not_cross_roles() {
        assert_eq!(
            selection_destination(Control::DomSource(0), "left", |_| true),
            Some(Control::DomSource(2))
        );
        assert_eq!(
            selection_destination(Control::Phase5(20), "right", |c| c != Control::Phase5(21)),
            Some(Control::Phase5(22))
        );
        assert_eq!(
            selection_destination(Control::HistoryKind(2), "home", |_| true),
            Some(Control::HistoryKind(0))
        );
        assert_eq!(
            selection_destination(Control::Guided(57), "end", |_| true),
            Some(Control::Guided(56))
        );
        assert_eq!(
            selection_destination(Control::Phase5(23), "right", |_| true),
            None
        );
        assert_eq!(
            selection_destination(Control::DomSource(0), "right", |_| false),
            None
        );
        assert_eq!(
            selection_destination(Control::DomSource(0), "enter", |_| true),
            None
        );
    }

    #[cfg(feature = "ui-tests")]
    #[gpui::test]
    fn group_keys_move_focus_without_recording_a_decision(cx: &mut gpui::TestAppContext) {
        use crate::keyboard_navigation_tests::{bind, press};
        use crate::workflow_refinement_tests::{finish_jobs, load_review};
        cx.update(bind);
        let dir = tempfile::tempdir().unwrap();
        let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
        let frozen = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
        for (width, height) in [(1280., 800.), (820., 600.)] {
            let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
            let owner = jobs.clone();
            let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
            cx.simulate_resize(size(px(width), px(height)));
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    load_review(s, project, cx);
                    s.navigation_preference = Some(false);
                    s.configure_review_geometry(width);
                    s.notice = None;
                    s.error = None;
                    assert!(s.control_enabled(Control::Phase5(20), cx));
                    window.focus(&s.handle(Control::Phase5(20), cx));
                    cx.notify();
                })
            });
            crate::phase4_hierarchy_tests::draw(cx);
            let requests = shell.read_with(cx, |s, _| s.next_request);
            for (key, expected) in [("right", 21), ("end", 22), ("home", 20), ("left", 22)] {
                press(cx, key);
                cx.update(|window, cx| {
                    shell.read_with(cx, |s, cx| {
                        assert_eq!(s.focused(window, cx), Some(Control::Phase5(expected)));
                        assert_eq!(s.next_request, requests, "focus is not a review write");
                        assert!(s.approval_selection.is_empty());
                    })
                });
            }
            press(cx, "tab");
            cx.update(|window, cx| {
                shell.read_with(cx, |s, cx| {
                    assert_eq!(s.focused(window, cx), Some(Control::Phase5(37)));
                })
            });
            let toolbar = cx.debug_bounds("viewer-toolbar").unwrap();
            for selector in ["action-Guided(50)", "action-Phase5(36)"] {
                crate::phase2_evidence_tests::inside(cx.debug_bounds(selector).unwrap(), toolbar);
            }
            // Fresh windows make absence checks meaningful for compact labels.
            assert_eq!(
                cx.debug_bounds("tool-label-Phase5(36)").is_some(),
                width >= 900.
            );
            shell.update(cx, finish_jobs);
            owner.join_after_ui();
            cx.update(|window, _| window.remove_window());
        }
        assert_eq!(
            frozen,
            serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
        );
    }
}
