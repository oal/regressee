// SPDX-License-Identifier: MPL-2.0
use regressee_app::{
    engine::{Client, ownership::Ownership},
    protocol::{Request, Response},
};
use std::{
    process::{Command, Stdio},
    thread,
    time::{Duration, Instant},
};
fn command(root: &std::path::Path, action: &str) -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_regressee"));
    c.args(["--json", "engine", action, "--data-dir"])
        .arg(root.join("data"))
        .arg("--cache-dir")
        .arg(root.join("cache"))
        .env_remove("DISPLAY")
        .env_remove("WAYLAND_DISPLAY")
        .env_remove("REGRESSEE_DISTRIBUTION")
        .env(
            "DBUS_SESSION_BUS_ADDRESS",
            "unix:path=/nonexistent-regressee-test-bus",
        )
        .stdin(Stdio::null());
    c
}
struct Stop(Client);
impl Drop for Stop {
    fn drop(&mut self) {
        let _ = self.0.request(Request::Stop);
    }
}

#[test]
fn stopped_protocol_eight_endpoint_upgrades_without_removing_data() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let paths = regressee_storage::AppPaths {
        data: root.path().join("data"),
        cache: root.path().join("cache"),
    };
    let owner = Ownership::acquire(&paths)?;
    let old_engine = uuid::Uuid::new_v4();
    regressee_app::engine::ownership::write_json(
        &owner.directory.join("endpoint.json"),
        &serde_json::json!({"version":8,"engine":old_engine,"cache":owner.paths.cache}),
    )?;
    let sentinel = paths.data.join("retained-project-data");
    std::fs::write(&sentinel, b"preserve existing data")?;
    let client = Client::existing(owner.paths.clone())?;
    let _stop = Stop(client.clone());
    drop(owner); // A successful explicit stop leaves this unowned descriptor.

    let started = command(root.path(), "start").output()?;
    assert!(
        started.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&started.stdout),
        String::from_utf8_lossy(&started.stderr)
    );
    assert!(client.snapshot().is_ok());
    let endpoint: serde_json::Value =
        serde_json::from_slice(&std::fs::read(paths.data.join("engine-v1/endpoint.json"))?)?;
    assert_eq!(endpoint["version"], regressee_app::protocol::VERSION);
    assert_ne!(endpoint["engine"], serde_json::json!(old_engine));
    assert_eq!(std::fs::read(sentinel)?, b"preserve existing data");
    assert!(Ownership::acquire(&paths).is_err());
    assert!(matches!(client.request(Request::Stop)?, Response::Ok));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(owner) = Ownership::acquire(&paths) {
            drop(owner);
            break;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    Ok(())
}

#[test]
fn simultaneous_autostarts_stale_endpoint_status_and_graceful_stop() -> anyhow::Result<()> {
    let root = tempfile::tempdir()?;
    let mut starts = vec![];
    for _ in 0..6 {
        let mut c = command(root.path(), "start");
        starts.push(thread::spawn(move || c.output().unwrap()));
    }
    let results: Vec<_> = starts.into_iter().map(|t| t.join().unwrap()).collect();
    for r in &results {
        assert!(r.status.success(), "{}", String::from_utf8_lossy(&r.stderr));
    }
    let snapshots: Vec<serde_json::Value> = results
        .iter()
        .map(|r| serde_json::from_slice(&r.stdout).unwrap())
        .collect();
    assert!(
        snapshots
            .iter()
            .all(|s| s["data"]["engine"] == snapshots[0]["data"]["engine"])
    );
    let client = Client::existing(serde_json::from_value(
        serde_json::json!({"data":root.path().join("data"),"cache":root.path().join("cache")}),
    )?)?;
    let _stop = Stop(client.clone());
    assert!(Ownership::acquire(&client.paths).is_err());
    assert!(!client.paths.data.join("private-sessions").exists());
    assert!(!client.paths.cache.join("product-runtime").exists());
    let other_cache = root.path().join("other-cache");
    std::fs::create_dir(&other_cache)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&other_cache, std::fs::Permissions::from_mode(0o700))?;
    }
    let mismatched = Client::existing(regressee_storage::AppPaths {
        data: client.paths.data.clone(),
        cache: other_cache,
    })?;
    assert!(
        mismatched
            .snapshot()
            .unwrap_err()
            .to_string()
            .contains("cache_directory_conflict")
    );
    let status = command(root.path(), "status").output()?;
    assert!(status.status.success());
    let reordered = Command::new(env!("CARGO_BIN_EXE_regressee"))
        .args(["--json", "engine", "status", "--cache-dir"])
        .arg(root.path().join("cache"))
        .arg("--data-dir")
        .arg(root.path().join("data"))
        .env_remove("HOME")
        .env_remove("USERPROFILE")
        .output()?;
    assert!(
        reordered.status.success(),
        "{}",
        String::from_utf8_lossy(&reordered.stderr)
    );
    assert_eq!(
        serde_json::from_slice::<serde_json::Value>(&status.stdout)?["data"]["engine"],
        snapshots[0]["data"]["engine"]
    );
    // A mismatched owner record is refused, not used to start another owner.
    let descriptor = client.paths.data.join("engine-v1/endpoint.json");
    let original = std::fs::read(&descriptor)?;
    let mut wrong: serde_json::Value = serde_json::from_slice(&original)?;
    for version in [1, 2, 3, 999] {
        wrong["version"] = serde_json::json!(version);
        std::fs::write(&descriptor, serde_json::to_vec(&wrong)?)?;
        let mismatch = command(root.path(), "start").output();
        std::fs::write(&descriptor, &original)?;
        let mismatch = mismatch?;
        assert!(!mismatch.status.success());
        assert!(String::from_utf8_lossy(&mismatch.stdout).contains("version_mismatch"));
        assert!(Ownership::acquire(&client.paths).is_err());
    }
    assert!(matches!(client.request(Request::Stop)?, Response::Ok));
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(owner) = Ownership::acquire(&client.paths) {
            drop(owner);
            break;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    let restart = command(root.path(), "start").output()?;
    assert!(restart.status.success());
    assert_ne!(
        serde_json::from_slice::<serde_json::Value>(&restart.stdout)?["data"]["engine"],
        snapshots[0]["data"]["engine"]
    );
    assert!(command(root.path(), "stop").output()?.status.success());
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(owner) = Ownership::acquire(&client.paths) {
            drop(owner);
            break;
        }
        assert!(Instant::now() < deadline);
        thread::sleep(Duration::from_millis(10));
    }
    // A stopped older owner's descriptor must not block an upgrade or pin cache.
    wrong["version"] = serde_json::json!(7);
    std::fs::write(&descriptor, serde_json::to_vec(&wrong)?)?;
    let changed = Client::autostart(
        mismatched.paths.clone(),
        regressee_app::engine::Launcher::exact(env!("CARGO_BIN_EXE_regressee").into()),
    )?;
    assert!(changed.snapshot().is_ok());
    assert!(matches!(changed.request(Request::Stop)?, Response::Ok));
    Ok(())
}
