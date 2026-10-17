// SPDX-License-Identifier: MPL-2.0
use super::*;
use regressee_app::control::{self, Command as Control};
use regressee_app::protocol::JobState;
struct Release(Client);
impl Drop for Release {
    fn drop(&mut self) {
        if let Ok(c) = self.0.register_frontend() {
            let _ = self.0.with_credential(c).control(Control::Takeover);
        }
    }
}
fn call(f: &Fixture, session: &str, args: &[&str], exit: i32) -> Result<Value> {
    let mut a = vec!["--session", session];
    a.extend(args);
    f.call(&a, exit)
}

#[test]
fn agent_cli_leases_headless_selection_handoff_and_takeover_preserve_actual_capture() -> Result<()>
{
    let f = Fixture::start()?;
    let _release = Release(f.client.clone());
    let (project, _) = f.create()?;
    let inventory = f.call(&["discovery", "start", "--project", &project], 0)?;
    let inventory = f.file("agent-inventory.json", value(&inventory))?;
    let started = f.call(
        &["agent", "start", "--name", "Release agent", "--lease", "90"],
        0,
    )?;
    let session = started["data"]["session"].as_str().unwrap();
    let credential_path = PathBuf::from(started["data"]["credential_file"].as_str().unwrap());
    regressee_app::engine::ownership::private_file(&credential_path, 4096)?;
    let credential: control::Credential = serde_json::from_slice(&fs::read(credential_path)?)?;
    assert!(!serde_json::to_string(&started)?.contains(&credential.secret.to_string()));
    let shown = f.call(&["agent", "show", "--id", session], 0)?;
    assert_eq!(shown["data"]["sessions"][0]["name"], "Release agent");
    f.call(&["agent", "start", "--name", "Other"], 5)?;
    f.call(
        &[
            "project",
            "create",
            "--name",
            "Intruder",
            "--url",
            "http://127.0.0.1:43210/site",
        ],
        5,
    )?;
    call(&f, session, &["agent", "renew", "--lease", "120"], 0)?;
    let selected = call(&f, session, &["agent", "select", "--project", &project], 0)?;
    assert_eq!(selected["data"]["value"]["selection"]["project"], project);
    let handoff = call(
        &f,
        session,
        &[
            "agent",
            "handoff",
            "--message",
            "A human can open the GUI to continue",
        ],
        4,
    )?;
    assert_eq!(handoff["code"], "action_required");
    assert_eq!(handoff["data"]["human_input"]["agent_paused"], true);
    let status = f.call(&["agent", "list"], 0)?;
    assert_eq!(status["data"]["sessions"][0]["status"], "Human");
    assert!(status["data"]["sessions"][0]["follow"].is_null());
    call(&f, session, &["agent", "select", "--project", &project], 5)?;
    call(&f, session, &["agent", "end"], 0)?;
    let started = f.call(&["agent", "start", "--name", "Capture agent"], 0)?;
    let session = started["data"]["session"].as_str().unwrap();
    fs::write(&f.state, "slow")?;
    let capture = call(
        &f,
        session,
        &[
            "capture",
            "baseline",
            "--project",
            &project,
            "--file",
            &inventory,
            "--detach",
        ],
        0,
    )?;
    let job = uuid::Uuid::parse_str(capture["data"]["job"]["id"].as_str().unwrap())?;
    while f.client.job(job)?.state == JobState::Queued {
        std::thread::sleep(Duration::from_millis(10));
    }
    let frozen = f.client.job(job)?.frozen;
    f.call(&["jobs", "cancel", "--job", &job.to_string()], 5)?;
    let frontend = f.client.register_frontend()?;
    f.client
        .with_credential(frontend)
        .control(Control::Takeover)?;
    call(
        &f,
        session,
        &["jobs", "cancel", "--job", &job.to_string()],
        5,
    )?;
    let done = f.call(&["jobs", "wait", "--job", &job.to_string()], 0)?;
    assert_eq!(done["data"]["job"]["state"], "Succeeded");
    assert_eq!(
        serde_json::to_value(f.client.job(job)?.frozen)?,
        serde_json::to_value(frozen)?
    );
    let saved = f.call(&["project", "show", "--project", &project], 0)?;
    assert!(
        value(&saved)["project"]["active_baseline"].is_null(),
        "takeover does not accept a baseline"
    );
    call(
        &f,
        session,
        &["jobs", "retry", "--request-id", &job.to_string()],
        0,
    )?;
    call(&f, session, &["agent", "renew"], 5)?;
    Ok(())
}

#[test]
fn actual_lease_expiry_refuses_old_agent_and_unlocks_anonymous_writes() -> Result<()> {
    let f = Fixture::start()?;
    let _release = Release(f.client.clone());
    let started = f.call(
        &["agent", "start", "--name", "Bounded agent", "--lease", "10"],
        0,
    )?;
    let session = started["data"]["session"].as_str().unwrap();
    std::thread::sleep(Duration::from_millis(10_100));
    call(
        &f,
        session,
        &[
            "project",
            "create",
            "--name",
            "Stale",
            "--url",
            "http://127.0.0.1:43210/site",
        ],
        5,
    )?;
    let status = f.call(&["agent", "list"], 0)?;
    assert!(status["data"]["controller"].is_null());
    assert_eq!(status["data"]["sessions"][0]["status"], "Expired");
    f.create()?;
    Ok(())
}
