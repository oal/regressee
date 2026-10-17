// SPDX-License-Identifier: MPL-2.0
//! Two-size, offline native settings inspection; no OS-input qualification.
use super::*;
use crate::project_settings_tests::{assert_destination, prepare};

#[gpui::test]
#[ignore = "explicit native settings wayfinding inspection"]
fn renderer_capture_project_settings(cx: &mut TestAppContext) {
    let root = output_root().unwrap();
    cx.enable_offscreen_renderer().unwrap();
    cx.update(|cx| {
        text_input::bind(cx);
        timezone_select::bind(cx);
    });
    let dir = tempfile::Builder::new()
        .prefix("project-settings-synthetic-")
        .tempdir_in(&root)
        .unwrap();
    let (store, project, _, _, _) = crate::approval_fixture::fixture(dir.path());
    let original = serde_json::to_value(store.project_evidence(project).unwrap()).unwrap();
    let mut manifest = Vec::new();
    for (width, height) in [(1280., 800.), (820., 600.)] {
        let (jobs, events) = Jobs::start_without_runtime(AppPaths::isolated(dir.path()));
        let owned = jobs.clone();
        let (shell, cx) = cx.add_window_view(|_, cx| Shell::new(cx, jobs, events));
        cx.simulate_resize(size(px(width), px(height)));
        shell.update(cx, |s, cx| {
            prepare(s, project, cx);
            s.name.update(cx, |i, cx| i.set_value("example.test", cx));
            cx.notify();
        });
        crate::phase4_hierarchy_tests::draw(cx);
        for (scene, section, advanced) in [
            ("website", SettingsSection::Website, false),
            ("capture", SettingsSection::Capture, false),
            ("access", SettingsSection::Access, false),
            ("advanced", SettingsSection::Capture, true),
        ] {
            shell.update(cx, |s, cx| {
                s.focus.advanced = advanced;
                cx.notify();
            });
            crate::phase4_hierarchy_tests::draw(cx);
            cx.update(|window, cx| {
                shell.update(cx, |s, cx| {
                    s.activate_control(Control::SettingsSection(section), window, cx)
                })
            });
            assert_destination(&shell, cx, section);
            cx.run_until_parked();
            crate::phase4_hierarchy_tests::draw(cx);
            let label = format!("project-settings-{width:.0}x{height:.0}-{scene}");
            capture(cx, &root, &label).unwrap();
            manifest.push(label);
        }
        owned.join_after_ui();
    }
    assert_eq!(
        original,
        serde_json::to_value(store.project_evidence(project).unwrap()).unwrap()
    );
    fs::write(
        root.join("project-settings-manifest.json"),
        serde_json::to_vec_pretty(&manifest).unwrap(),
    )
    .unwrap();
}
