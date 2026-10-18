// SPDX-License-Identifier: MPL-2.0
//! Public installation contract against a disposable home, without an engine.
use serde_json::Value;
use std::{
    fs,
    path::Path,
    process::{Command, Output},
};

fn run(home: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_regressee"))
        .env("HOME", home)
        .env("USERPROFILE", home)
        .args(["--json", "--data-dir"])
        .arg(home.join("private data with spaces"))
        .arg("--cache-dir")
        .arg(home.join("cache 'æ'"))
        .args(args)
        .output()
        .unwrap()
}
fn json(output: &Output) -> Value {
    serde_json::from_slice(&output.stdout).unwrap()
}

#[test]
fn installs_updates_and_removes_for_each_host_without_an_engine() {
    let home = tempfile::tempdir().unwrap();
    let status = run(home.path(), &["integrations", "status"]);
    assert!(status.status.success(), "{status:?}");
    assert_eq!(
        json(&status)["data"]["integrations"]
            .as_array()
            .unwrap()
            .len(),
        3
    );
    assert_eq!(
        fs::read_dir(home.path()).unwrap().count(),
        0,
        "Status is read-only"
    );
    for (host, folder) in [
        ("claude-code", ".claude"),
        ("codex", ".agents"),
        ("github-copilot", ".copilot"),
    ] {
        let install = run(home.path(), &["integrations", "install", "--host", host]);
        assert!(install.status.success(), "{install:?}");
        let receipt = json(&install);
        assert_eq!(receipt["data"]["integrations"][0]["state"], "installed");
        assert_eq!(receipt["data"]["host_access_tested"], false);
        let path = home.path().join(folder).join("skills/regressee");
        let skill = fs::read_to_string(path.join("SKILL.md")).unwrap();
        let frontmatter = skill
            .strip_prefix("---\n")
            .unwrap()
            .split("\n---")
            .next()
            .unwrap();
        assert_eq!(
            serde_json::from_str::<Value>(frontmatter).unwrap()["name"],
            "regressee"
        );
        assert!(skill.contains(&format!(
            "engine protocol {}",
            regressee_app::protocol::VERSION
        )));
        let connection: Value =
            serde_json::from_slice(&fs::read(path.join("connection.json")).unwrap()).unwrap();
        assert_eq!(
            connection["data_dir"],
            home.path()
                .join("private data with spaces")
                .to_str()
                .unwrap()
        );
        assert_eq!(
            connection["cache_dir"],
            home.path().join("cache 'æ'").to_str().unwrap()
        );
        for reference in ["commands.md", "workflows.md", "safety-and-evidence.md"] {
            assert!(path.join("references").join(reference).is_file());
        }
        assert!(
            run(home.path(), &["integrations", "install", "--host", host])
                .status
                .success()
        );
        assert!(
            run(home.path(), &["integrations", "remove", "--host", host])
                .status
                .success()
        );
        assert!(!path.exists());
    }
    assert!(
        !home.path().join("private data with spaces").exists(),
        "No engine, database or product files"
    );
    assert!(!home.path().join("cache 'æ'").exists());
}

#[test]
fn edited_skills_and_foreign_config_are_kept() {
    let home = tempfile::tempdir().unwrap();
    let path = home.path().join(".claude");
    fs::create_dir(&path).unwrap();
    fs::write(
        path.join("settings.json"),
        "{\"permissions\":{\"deny\":[\"Bash\"]}}",
    )
    .unwrap();
    let settings = fs::read(path.join("settings.json")).unwrap();
    assert!(
        run(
            home.path(),
            &["integrations", "install", "--host", "claude-code"]
        )
        .status
        .success()
    );
    let skill = path.join("skills/regressee/SKILL.md");
    fs::write(&skill, "My custom instructions").unwrap();
    let status = run(
        home.path(),
        &["integrations", "status", "--host", "claude-code"],
    );
    assert_eq!(
        json(&status)["data"]["integrations"][0]["state"],
        "conflict"
    );
    for action in ["install", "remove"] {
        let output = run(
            home.path(),
            &["integrations", action, "--host", "claude-code"],
        );
        assert!(!output.status.success());
        assert_eq!(
            fs::read_to_string(&skill).unwrap(),
            "My custom instructions"
        );
    }
    assert_eq!(fs::read(path.join("settings.json")).unwrap(), settings);
}

#[test]
fn invalid_setup_arguments_never_create_files() {
    let home = tempfile::tempdir().unwrap();
    for args in [
        vec!["integrations", "install"],
        vec!["integrations", "install", "--host", "unknown"],
        vec!["integrations", "install", "--host", "codex", "--prepare"],
        vec!["integrations", "install", "--host", "codex", "--detach"],
        vec![
            "integrations",
            "install",
            "--host",
            "codex",
            "--session",
            "aa9edfc4-689f-42dd-ad38-3ebf1b9a9a20",
        ],
    ] {
        let output = run(home.path(), &args);
        assert_eq!(output.status.code(), Some(2), "{output:?}");
        assert_eq!(fs::read_dir(home.path()).unwrap().count(), 0);
    }
}
