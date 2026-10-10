// SPDX-License-Identifier: MPL-2.0
//! Development-only, finite native-render inspection. Never OS-input acceptance.
//! Parent wires this module only under `ui-inspection` and calls `start` only for
//! `--inspect-ui`. All filesystem work runs on the background executor.
#![cfg(feature = "ui-inspection")]
use crate::{Choice, Control, Detail, Screen, Shell};
use anyhow::{Context as _, Result, bail, ensure};
use gpui::{App, Entity, Window, px};
use serde::Deserialize;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

const APPROVED: &str = "/path/to/private-tests/opencode";
const NAME: &str = "Native inspection saved project";
const LIMIT: Duration = Duration::from_secs(15 * 60);

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
struct Lease {
    version: u32,
    nonce: String,
    exe_sha256: String,
    fixture_project: String,
}

#[derive(Clone)]
struct Gate {
    dir: PathBuf,
    lease: Lease,
    started: Instant,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Ack {
    version: u32,
    nonce: String,
    stage: usize,
    label: String,
    png_sha256: String,
    personally_reviewed: bool,
}

#[derive(Clone, Copy)]
enum Scene {
    Projects,
    OpenFixture,
    Setup,
    Advanced,
    Timezone,
    Popup,
    Environment,
    Results,
    Analysis,
    History,
    HistoryLower,
    Settings,
    SettingsLower,
    New,
    InvalidSave,
    Fill,
    Save,
    Reopen,
}

// Fixed labels only; no user URLs, titles or arbitrary commands in the protocol.
const STAGES: &[(&str, Scene, [u32; 2])] = &[
    ("projects-initial", Scene::Projects, [1120, 800]),
    ("setup-fixture-cold", Scene::OpenFixture, [1120, 800]),
    ("projects-warm-one", Scene::Projects, [1120, 800]),
    ("setup-warm-one", Scene::Setup, [1120, 800]),
    ("projects-warm-two", Scene::Projects, [1120, 800]),
    ("setup-warm-two", Scene::Setup, [1120, 800]),
    ("setup-advanced-wide", Scene::Advanced, [1120, 800]),
    ("setup-timezone-wide", Scene::Timezone, [1120, 800]),
    ("timezone-popup-wide", Scene::Popup, [1120, 800]),
    ("environment-choices-wide", Scene::Environment, [1120, 800]),
    ("results-viewer-wide", Scene::Results, [1120, 800]),
    ("results-analysis-wide", Scene::Analysis, [1120, 800]),
    ("history-wide", Scene::History, [1120, 800]),
    ("history-lower-wide", Scene::HistoryLower, [1120, 800]),
    ("settings-wide", Scene::Settings, [1120, 800]),
    ("settings-lower-wide", Scene::SettingsLower, [1120, 800]),
    ("projects-small", Scene::Projects, [820, 600]),
    ("setup-small", Scene::Setup, [820, 600]),
    ("setup-advanced-small", Scene::Advanced, [820, 600]),
    ("setup-timezone-small", Scene::Timezone, [820, 600]),
    ("timezone-popup-small", Scene::Popup, [820, 600]),
    ("environment-choices-small", Scene::Environment, [820, 600]),
    ("results-viewer-small", Scene::Results, [820, 600]),
    ("results-analysis-small", Scene::Analysis, [820, 600]),
    ("history-small", Scene::History, [820, 600]),
    ("history-lower-small", Scene::HistoryLower, [820, 600]),
    ("settings-small", Scene::Settings, [820, 600]),
    ("settings-lower-small", Scene::SettingsLower, [820, 600]),
    ("new-project-empty", Scene::New, [820, 600]),
    ("new-project-validation", Scene::InvalidSave, [820, 600]),
    ("new-project-focused", Scene::Fill, [820, 600]),
    ("new-project-saved", Scene::Save, [820, 600]),
    ("new-project-reopened", Scene::Reopen, [820, 600]),
];

fn private_directory(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    ensure!(
        path.is_absolute() && path.starts_with(APPROVED),
        "outside approved scratch"
    );
    ensure!(fs::canonicalize(path)? == path, "nonphysical path");
    let meta = fs::symlink_metadata(path)?;
    ensure!(
        meta.is_dir() && meta.permissions().mode() & 0o777 == 0o700,
        "not private directory"
    );
    Ok(())
}

fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn validate() -> Result<Gate> {
    use sha2::{Digest as _, Sha256};
    ensure!(
        std::env::args_os().any(|a| a == "--inspect-ui"),
        "explicit CLI required"
    );
    let dir = PathBuf::from(
        std::env::var_os("REGRESSEE_NATIVE_INSPECTION_DIR").context("ack root required")?,
    );
    let store =
        PathBuf::from(std::env::var_os("REGRESSEE_DEV_DATA").context("synthetic store required")?);
    private_directory(&dir)?;
    private_directory(&store)?;
    let parent = dir.parent().context("unique round root required")?;
    private_directory(parent)?;
    ensure!(
        store.parent() == Some(parent) && store != dir,
        "roots must be separate siblings"
    );
    let lease_path = dir.join("lease.json");
    ensure!(
        fs::canonicalize(&lease_path)? == lease_path && fs::metadata(&lease_path)?.len() < 4096,
        "invalid lease file"
    );
    let lease: Lease = serde_json::from_slice(&fs::read(lease_path)?)?;
    ensure!(
        lease.version == 1
            && lease.nonce.len() == 32
            && lease.nonce.bytes().all(|b| b.is_ascii_hexdigit()),
        "invalid lease"
    );
    ensure!(digest(&lease.exe_sha256), "invalid executable hash");
    uuid::Uuid::parse_str(&lease.fixture_project)?;
    let exe = fs::canonicalize(std::env::current_exe()?)?;
    ensure!(
        exe == parent.join("bin/regressee"),
        "requires private copied binary"
    );
    ensure!(
        hex::encode(Sha256::digest(fs::read(exe)?)) == lease.exe_sha256,
        "copied binary hash changed"
    );
    Ok(Gate {
        dir,
        lease,
        started: Instant::now(),
    })
}

fn write_new(gate: &Gate, name: &str, value: &serde_json::Value) -> Result<()> {
    private_directory(&gate.dir)?;
    let temporary = gate.dir.join(format!(".{name}.tmp"));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&temporary)?;
    serde_json::to_writer(&mut file, value)?;
    file.write_all(b"\n")?;
    file.sync_all()?;
    // Atomic visibility without replacing an existing stage/result file.
    fs::hard_link(&temporary, gate.dir.join(name))?;
    fs::remove_file(temporary)?;
    Ok(())
}

fn wait_ack(gate: &Gate, index: usize, label: &str) -> Result<()> {
    let path = gate.dir.join(format!("advance-{index:03}.json"));
    while gate.started.elapsed() < LIMIT {
        private_directory(&gate.dir)?;
        match fs::symlink_metadata(&path) {
            Ok(meta) => {
                ensure!(
                    meta.is_file() && meta.len() < 4096 && fs::canonicalize(&path)? == path,
                    "invalid ack path"
                );
                let ack: Ack = serde_json::from_slice(&fs::read(&path)?)?;
                ensure!(
                    ack.version == 1
                        && ack.nonce == gate.lease.nonce
                        && ack.stage == index
                        && ack.label == label,
                    "ack mismatch"
                );
                ensure!(
                    ack.personally_reviewed && digest(&ack.png_sha256),
                    "actual image review acknowledgement required"
                );
                return Ok(());
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (),
            Err(error) => return Err(error.into()),
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    bail!("inspection deadline reached")
}

fn navigate(shell: &mut Shell, screen: Screen, window: &mut Window, cx: &mut gpui::Context<Shell>) {
    shell.navigate(screen, window, cx);
    shell.screen_scroll.set_offset(gpui::point(px(0.), px(0.)));
}

fn apply(
    scene: Scene,
    shell: &mut Shell,
    window: &mut Window,
    cx: &mut gpui::Context<Shell>,
    fixture: &str,
) -> Result<()> {
    match scene {
        Scene::Projects => navigate(shell, Screen::Projects, window, cx),
        Scene::OpenFixture => {
            let index = shell
                .projects
                .items
                .iter()
                .position(|p| p.id.to_string() == fixture)
                .context("fixture not listed")?;
            shell.activate_control(Control::OpenProject(index), window, cx);
        }
        Scene::Setup => navigate(shell, Screen::Setup, window, cx),
        Scene::Advanced => {
            navigate(shell, Screen::Setup, window, cx);
            if !shell.focus.advanced {
                shell.activate_control(Control::Guided(6), window, cx);
            }
        }
        Scene::Timezone => {
            navigate(shell, Screen::Setup, window, cx);
            window.focus(&shell.handle(Control::CrawlField(16), cx));
        }
        Scene::Popup => {
            window.focus(&shell.handle(Control::CrawlField(16), cx));
            window.dispatch_action(Box::new(crate::timezone_select::Toggle), cx);
        }
        Scene::Environment => {
            window.dispatch_action(Box::new(crate::timezone_select::Close), cx);
            shell.activate_control(Control::Choice(Choice::LoginRequired), window, cx);
            shell.activate_control(Control::Choice(Choice::EphemeralLogin), window, cx);
            window.focus(&shell.handle(Control::Choice(Choice::LoginRequired), cx));
        }
        Scene::Results => {
            // Restore only the synthetic draft choice edits; do not save fixture.
            shell.activate_control(Control::Choice(Choice::PublicAccess), window, cx);
            shell.login_ephemeral = shell
                .environment
                .as_ref()
                .is_some_and(|e| e.settings.ephemeral);
            navigate(shell, Screen::Results, window, cx);
        }
        Scene::Analysis => {
            if !shell.analysis_advanced {
                shell.activate_control(Control::Phase5(3), window, cx);
            }
            if !shell.details.contains(&Detail::Analysis) {
                shell.activate_control(Control::Details(Detail::Analysis), window, cx);
            }
        }
        Scene::History => navigate(shell, Screen::History, window, cx),
        Scene::HistoryLower => {
            if !shell.details.contains(&Detail::History) {
                shell.activate_control(Control::Details(Detail::History), window, cx);
            }
        }
        Scene::Settings => navigate(shell, Screen::Settings, window, cx),
        Scene::SettingsLower => (),
        Scene::New => {
            ensure!(!shell.unsaved(cx), "unexpected dirty fixture draft");
            window.focus(&shell.root_focus);
            window.dispatch_action(Box::new(crate::CreateProject), cx);
        }
        Scene::InvalidSave => {
            window.dispatch_action(Box::new(crate::SaveProject), cx);
        }
        Scene::Fill => {
            shell.name.update(cx, |input, cx| input.set_value(NAME, cx));
            shell.address.update(cx, |input, cx| {
                input.set_value("http://127.0.0.1:43210/", cx)
            });
            window.focus(&shell.handle(Control::Address, cx));
        }
        Scene::Save => {
            window.dispatch_action(Box::new(crate::SaveProject), cx);
        }
        Scene::Reopen => {
            let record = shell
                .project
                .as_ref()
                .context("new project was not saved")?;
            ensure!(
                record.project.name == NAME && record.project.id.to_string() != fixture,
                "unexpected saved project"
            );
            let id = record.project.id;
            shell.queue(crate::Work::Load(id), cx);
        }
    }
    cx.notify();
    Ok(())
}

fn position(scene: Scene, shell: &mut Shell, cx: &mut gpui::Context<Shell>) {
    // Real native scroll state, not a fake viewport or screenshot crop. Fractions
    // are predetermined; the resulting viewport still requires visual review.
    let fraction = match scene {
        Scene::Advanced => 0.23,
        Scene::Timezone => 0.94,
        Scene::Environment => 0.57,
        Scene::Analysis | Scene::HistoryLower | Scene::SettingsLower => 1.,
        Scene::Popup => return,
        _ => 0.,
    };
    shell.screen_scroll.set_offset(gpui::point(
        px(0.),
        -shell.screen_scroll.max_offset().height * fraction,
    ));
    cx.notify();
}

/// Call only from the CLI/feature-gated development harness, never normal startup.
pub(crate) fn start(shell: Entity<Shell>, window: &mut Window, cx: &mut App) {
    let background = cx.background_executor().clone();
    window.spawn(cx, async move |cx| {
        let result: Result<()> = async {
            let gate = background.spawn(async { validate() }).await?;
            // Wait for initial synthetic project list without reading files on UI.
            loop {
                ensure!(gate.started.elapsed() < LIMIT, "initial load deadline");
                let ready = cx.update(|_, cx| shell.read(cx).pending.is_none() && shell.read(cx).projects.items.iter().any(|p| p.id.to_string() == gate.lease.fixture_project))?;
                if ready { break; }
                background.timer(Duration::from_millis(100)).await;
            }
            for (index, &(label, scene, logical_size)) in STAGES.iter().enumerate() {
                ensure!(gate.started.elapsed() < LIMIT, "stage deadline");
                let started = Instant::now();
                cx.update(|window, cx| shell.update(cx, |shell, cx| apply(scene, shell, window, cx, &gate.lease.fixture_project)))??;
                // Deferred normal actions and background readers must finish first.
                background.timer(Duration::from_millis(250)).await;
                loop {
                    ensure!(gate.started.elapsed() < LIMIT, "idle deadline");
                    let idle = cx.update(|_, cx| { let s = shell.read(cx); s.pending.is_none() && s.analysis_pending.is_none() && s.analysis_due.is_none() && !s.focus.busy })?;
                    if idle { break; }
                    background.timer(Duration::from_millis(100)).await;
                }
                cx.update(|_, cx| shell.update(cx, |shell, cx| position(scene, shell, cx)))?;
                background.timer(Duration::from_millis(250)).await;
                let viewport = cx.update(|window, _| { let s = window.viewport_size(); [f32::from(s.width), f32::from(s.height)] })?;
                let event = serde_json::json!({"version":1,"nonce":gate.lease.nonce,"pid":std::process::id(),"stage":index,"label":label,"logical_size":logical_size,"observed_gpui_viewport":viewport,"handler_to_idle_ms":started.elapsed().as_secs_f64()*1000.,"internal_driver":true,"os_input_acceptance":false,"gpu_presentation_timing":false});
                let copy = gate.clone();
                background.spawn(async move { write_new(&copy, &format!("stage-{index:03}.json"), &event) }).await?;
                // Stage zero cannot advance until parent personally reads its PNG.
                let copy = gate.clone();
                background.spawn(async move { wait_ack(&copy, index, label) }).await?;
            }
            let value = serde_json::json!({"version":1,"stages":STAGES.len(),"native_visual_qualification":"requires separately reviewed PNGs","os_input_acceptance":false});
            background.spawn(async move { write_new(&gate, "complete.json", &value) }).await?;
            Ok(())
        }.await;
        if result.is_err() {
            // Never leak input/project/path content through error-chain logging.
            log::error!("ui-inspection stopped: gate, stage or deadline failure; no unsafe fallback");
        }
    }).detach();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_has_one_initial_gate_and_a_finite_size_transition() {
        assert_eq!(STAGES.len(), 33);
        assert_eq!(STAGES[0].0, "projects-initial");
        assert_eq!(STAGES[32].0, "new-project-reopened");
        for (index, &(label, _, size)) in STAGES.iter().enumerate() {
            assert!(label.bytes().all(|c| c.is_ascii_lowercase() || c == b'-'));
            assert_eq!(size, if index < 16 { [1120, 800] } else { [820, 600] });
            assert!(!STAGES[..index].iter().any(|stage| stage.0 == label));
        }
    }

    #[test]
    fn review_and_binary_digests_are_strict_lowercase_sha256() {
        assert!(digest(&"a".repeat(64)));
        assert!(!digest(&"A".repeat(64)));
        assert!(!digest(&"x".repeat(64)));
        assert!(!digest(&"a".repeat(63)));
    }
}
