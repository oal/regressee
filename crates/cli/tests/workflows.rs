// SPDX-License-Identifier: MPL-2.0
//! Public subprocess grammar over a real IPC engine and product services.
//! Signed synthetic worker produces owned PNGs; NOT a browser qualification.
#![cfg(target_os = "linux")]
use anyhow::{Context, Result};
use ed25519_dalek::{Signer, SigningKey};
use regressee_app::{
    engine::{Client, Server},
    product::Product,
    protocol::*,
};
use regressee_core::domain::*;
use regressee_runtime::distribution::{self, Distribution, Entry, Manifest, Scope};
use regressee_storage::AppPaths;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    fs,
    io::Write,
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::atomic::AtomicBool,
    time::{Duration, Instant},
};
use zip::{ZipWriter, write::SimpleFileOptions};

struct Fixture {
    root: tempfile::TempDir,
    client: Client,
    thread: Option<std::thread::JoinHandle<Result<()>>>,
    state: PathBuf,
}
impl Fixture {
    fn start() -> Result<Self> {
        let root = tempfile::tempdir()?;
        let paths = AppPaths::isolated(root.path());
        paths.prepare()?;
        let state = root.path().join("state");
        fs::write(&state, "base")?;
        let key = SigningKey::from_bytes(&[94; 32]);
        let d = Distribution {
            version: 1,
            url: "http://127.0.0.1:9/owned-synthetic.zip".parse()?,
            public_key: hex::encode(key.verifying_key().to_bytes()),
            scope: Scope::DevelopmentLoopback,
        };
        let python = fs::read(fs::canonicalize("/usr/bin/python3")?)?;
        let script = include_str!("synthetic_worker.py")
            .replace("__STATE__", &serde_json::to_string(&state)?)
            .into_bytes();
        let files = BTreeMap::from([
            ("node/python".to_string(), python),
            ("worker/main.py".into(), script),
            (
                "browsers/synthetic".into(),
                b"No browser in this synthetic fixture".to_vec(),
            ),
        ]);
        let manifest = Manifest {
            version: 2,
            id: "cli-synthetic-v1".into(),
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            scope: Scope::DevelopmentLoopback,
            support_until: None,
            identity: RuntimeIdentity::Pinned {
                node: "synthetic-python".into(),
                worker: "5.0.1".into(),
                playwright: "synthetic".into(),
                browser: "synthetic".into(),
            },
            node: "node/python".into(),
            worker: "worker/main.py".into(),
            browser_root: "browsers".into(),
            files: files
                .iter()
                .map(|(name, data)| {
                    Ok((
                        name.clone(),
                        Entry {
                            sha256: distribution::hash(&data[..], &AtomicBool::new(false))?,
                            bytes: data.len() as u64,
                            executable: name == "node/python",
                        },
                    ))
                })
                .collect::<Result<_>>()?,
            links: BTreeMap::new(),
            modes: BTreeMap::new(),
        };
        let pack = root.path().join("runtime.zip");
        let mut zip = ZipWriter::new(fs::File::create(&pack)?);
        let bytes = serde_json::to_vec(&manifest)?;
        for (name, data) in [
            ("manifest.json", bytes.as_slice()),
            ("manifest.sig", &key.sign(&bytes).to_bytes()),
        ] {
            zip.start_file(name, SimpleFileOptions::default())?;
            zip.write_all(data)?;
        }
        for (name, data) in files {
            zip.start_file(name, SimpleFileOptions::default())?;
            zip.write_all(&data)?;
        }
        zip.finish()?;
        let runtime_cache = paths.cache.join("product-runtime");
        distribution::install(
            &pack,
            &runtime_cache,
            &d,
            &AtomicBool::new(false),
            &mut |_| Ok(()),
        )?;
        let server = Server::with_product(
            paths.clone(),
            Product {
                distribution: Some(d),
                official_runtime: false,
                runtime_cache,
                auth: Default::default(),
            },
        )?;
        let thread = std::thread::spawn(move || server.run());
        let client = Client::existing(paths)?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while client.snapshot().is_err() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(Self {
            root,
            client,
            thread: Some(thread),
            state,
        })
    }
    fn command(&self, args: &[&str]) -> Command {
        let mut c = Command::new(env!("CARGO_BIN_EXE_regressee"));
        c.args(["--json", "--data-dir"])
            .arg(&self.client.paths.data)
            .arg("--cache-dir")
            .arg(&self.client.paths.cache)
            .args(args)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("REGRESSEE_DISTRIBUTION")
            .stdin(Stdio::null());
        c
    }
    fn call(&self, args: &[&str], exit: i32) -> Result<Value> {
        let out = self.command(args).output()?;
        let v: Value = serde_json::from_slice(&out.stdout).with_context(|| {
            format!(
                "stdout {:?}; stderr {}",
                out.stdout,
                String::from_utf8_lossy(&out.stderr)
            )
        })?;
        assert_eq!(
            out.status.code(),
            Some(exit),
            "{args:?}: {v}\nstderr {}",
            String::from_utf8_lossy(&out.stderr)
        );
        assert_eq!(v["schema_version"], 1);
        assert_eq!(v["exit_status"], exit);
        Ok(v)
    }
    fn file(&self, name: &str, value: &Value) -> Result<String> {
        let path = self.root.path().join(name);
        fs::write(&path, serde_json::to_vec(value)?)?;
        Ok(path.to_str().unwrap().into())
    }
    fn prepare_confirm(&self, args: &[&str]) -> Result<Value> {
        let mut args = args.to_vec();
        args.push("--prepare");
        let p = self.call(&args, 0)?;
        let ticket = self.file(
            &format!("ticket-{}.json", uuid::Uuid::new_v4()),
            &p["data"]["ticket"],
        )?;
        self.call(
            &[
                "confirm",
                "--file",
                &ticket,
                "--confirmation",
                p["data"]["ticket"]["confirmation"]["id"].as_str().unwrap(),
            ],
            0,
        )
    }
    fn create(&self) -> Result<(String, Value)> {
        let v = self.call(
            &[
                "project",
                "create",
                "--name",
                "Owned <script>alert(1)</script>",
                "--url",
                "http://127.0.0.1:43210/site",
            ],
            0,
        )?;
        Ok((
            v["data"]["value"]["project"]["id"].as_str().unwrap().into(),
            v["data"]["value"].clone(),
        ))
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        // These are exclusively owned synthetic fixtures. On a failed assertion,
        // clean up their admitted work explicitly before joining the owner.
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if matches!(self.client.request(Request::Stop), Ok(Response::Ok)) {
                break;
            }
            if let Ok(snapshot) = self.client.snapshot() {
                for j in snapshot.jobs {
                    if !j.state.terminal() {
                        let _ = self.client.request(Request::Cancel { id: j.id });
                    }
                }
            }
            assert!(
                Instant::now() < deadline,
                "owned fixture engine did not stop"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        if let Some(t) = self.thread.take() {
            let _ = t.join();
        }
    }
}
fn value(v: &Value) -> &Value {
    &v["data"]["value"]
}
fn run(v: &Value) -> &str {
    value(v)["latest"]["run"]["id"].as_str().unwrap()
}

#[path = "support/phase3_repairs.rs"]
mod phase3_repairs;
#[path = "support/phase4_control.rs"]
mod phase4_control;
#[path = "support/phase5_repairs.rs"]
mod phase5_repairs;

#[test]
fn new_project_discovery_baseline_check_review_accept_report_zip_and_retained_history() -> Result<()>
{
    let f = Fixture::start()?;
    let (p, created) = f.create()?;
    let mut settings = created["project"].clone();
    settings["discovery"]["speed"] = json!("Fast");
    settings["discovery"]["advanced"]["retries"] = json!(0);
    let config = f.file("config.json", &settings)?;
    f.call(
        &[
            "project",
            "update",
            "--project",
            &p,
            "--expected-revision",
            "1",
            "--file",
            &config,
        ],
        0,
    )?;
    let discovered = f.call(&["discovery", "start", "--project", &p], 0)?;
    let reopened = f.call(
        &[
            "discovery",
            "inventory",
            "--job",
            discovered["data"]["job"]["id"].as_str().unwrap(),
        ],
        0,
    )?;
    assert_eq!(
        value(&reopened)["inventory"]["id"],
        value(&discovered)["inventory"]["id"]
    );
    assert_eq!(
        value(&discovered)["inventory"]["pages"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    let inventory = f.file("inventory.json", value(&discovered))?;
    let baseline = f.call(
        &["capture", "baseline", "--project", &p, "--file", &inventory],
        0,
    )?;
    let baseline_run = run(&baseline).to_owned();
    let saved = f.prepare_confirm(&[
        "baseline",
        "save",
        "--run",
        &baseline_run,
        "--expected-revision",
        "2",
    ])?;
    let original = value(&saved)["project"]["active_baseline"]
        .as_str()
        .unwrap()
        .to_owned();
    fs::write(&f.state, "changed")?;
    let checked = f.call(&["capture", "check", "--project", &p], 1)?;
    assert_eq!(checked["code"], "visual_differences");
    let checked_run = run(&checked).to_owned();
    let analysis = value(&checked)["comparison"]["revision"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let listed = f.call(
        &[
            "results",
            "list",
            "--project",
            &p,
            "--run",
            &checked_run,
            "--outcome",
            "changed",
            "--limit",
            "1",
        ],
        0,
    )?;
    assert_eq!(value(&listed)["total"], 2);
    assert_eq!(value(&listed)["next_offset"], 1);
    let key = value(&listed)["rows"][0]["target"]["key"].clone();
    let page = key["page"].as_str().unwrap();
    let profile = key["profile"].as_str().unwrap();
    let selected = f.file("selected.json", &json!([key.clone()]))?;
    let artifacts = f.call(
        &[
            "results",
            "artifacts",
            "--project",
            &p,
            "--run",
            &checked_run,
            "--analysis",
            &analysis,
            "--page",
            page,
            "--profile",
            profile,
        ],
        0,
    )?;
    for name in ["before", "after", "difference"] {
        assert!(Path::new(value(&artifacts)[name]["original_path"].as_str().unwrap()).is_file());
        assert_eq!(value(&artifacts)[name]["width"], 64);
    }
    let original_path = value(&artifacts)["after"]["original_path"]
        .as_str()
        .unwrap();
    let original_bytes = fs::read(original_path)?;
    let review = f.call(
        &[
            "review",
            "set",
            "--analysis",
            &analysis,
            "--page",
            page,
            "--profile",
            profile,
            "--state",
            "intentional",
            "--note",
            "</pre><script>evil()</script> ![](https://attacker.invalid/x)",
        ],
        0,
    )?;
    assert!(!value(&review).as_array().unwrap().is_empty());
    let report_dest = f.root.path().join("report");
    let report = f.call(
        &[
            "report",
            "preview",
            "--project",
            &p,
            "--run",
            &checked_run,
            "--analysis",
            &analysis,
            "--file",
            &selected,
            "--destination",
            report_dest.to_str().unwrap(),
        ],
        0,
    )?;
    assert!(
        value(&report)["privacy"]
            .as_str()
            .unwrap()
            .contains("query strings")
    );
    let report_file = f.file("report-preview.json", value(&report))?;
    f.prepare_confirm(&["report", "export", "--file", &report_file])?;
    let html = fs::read_to_string(report_dest.join("index.html"))?;
    assert!(!html.contains("<script>"));
    assert!(html.contains("&lt;script&gt;evil()"));
    assert!(!html.contains("src=\"http"));
    assert!(html.contains("Content-Security-Policy"));
    assert_eq!(fs::read(report_dest.join("0-after.png"))?, original_bytes);
    let crop_dest = f.root.path().join("crop.png");
    let crop = f.call(
        &[
            "crop",
            "preview",
            "--project",
            &p,
            "--run",
            &checked_run,
            "--page",
            page,
            "--profile",
            profile,
            "--pane",
            "after",
            "--rect",
            "1,2,10,12",
            "--space",
            "screenshot-pixels",
            "--destination",
            crop_dest.to_str().unwrap(),
        ],
        0,
    )?;
    let crop_file = f.file("crop-preview.json", value(&crop))?;
    f.prepare_confirm(&["crop", "export", "--file", &crop_file])?;
    assert_eq!(
        regressee_storage::images::validate_png(&crop_dest, &AtomicBool::new(false))?,
        (10, 12)
    );
    assert_eq!(fs::read(original_path)?, original_bytes);
    let approval = f.call(
        &[
            "baseline",
            "preview",
            "--analysis",
            &analysis,
            "--file",
            &selected,
        ],
        0,
    )?;
    let approval_file = f.file("approval.json", value(&approval))?;
    let accepted = f.prepare_confirm(&["baseline", "accept", "--file", &approval_file])?;
    let active = value(&accepted)["project"]["active_baseline"]
        .as_str()
        .unwrap();
    assert_ne!(active, original);
    let history = f.call(&["history", "list", "--project", &p], 0)?;
    assert_eq!(value(&history)["baselines"].as_array().unwrap().len(), 2);
    f.call(
        &["history", "show", "--project", &p, "--run", &checked_run],
        0,
    )?;
    let export = f.call(
        &[
            "archive",
            "export-preview",
            "--project",
            &p,
            "--baseline",
            active,
        ],
        0,
    )?;
    let export_file = f.file("export.json", value(&export))?;
    let zip = f.root.path().join("baseline.zip");
    f.prepare_confirm(&[
        "archive",
        "export",
        "--file",
        &export_file,
        "--destination",
        zip.to_str().unwrap(),
    ])?;
    let imported_preview = f.call(
        &[
            "archive",
            "import-preview",
            "--source",
            zip.to_str().unwrap(),
        ],
        0,
    )?;
    let imported_file = f.file("import.json", value(&imported_preview))?;
    f.prepare_confirm(&[
        "archive",
        "import",
        "--file",
        &imported_file,
        "--source",
        zip.to_str().unwrap(),
    ])?;
    let revision = value(&accepted)["revision"].as_u64().unwrap().to_string();
    f.prepare_confirm(&[
        "baseline",
        "restore",
        "--baseline",
        &original,
        "--expected-revision",
        &revision,
    ])?;
    // The superseded accepted version can be removed; retained reference sources
    // and the original check history remain inspectable.
    let deletion = f.call(
        &[
            "storage",
            "delete-preview",
            "--project",
            &p,
            "--baseline",
            active,
        ],
        0,
    )?;
    let deletion_file = f.file("delete.json", value(&deletion))?;
    f.prepare_confirm(&["storage", "delete", "--file", &deletion_file])?;
    f.call(
        &["history", "show", "--project", &p, "--run", &checked_run],
        0,
    )?;
    f.call(&["storage", "usage", "--project", &p], 0)?;
    Ok(())
}

#[test]
fn stale_config_confirmation_failed_captures_and_private_path_limits() -> Result<()> {
    let f = Fixture::start()?;
    let (p, created) = f.create()?;
    let config = f.file("config.json", &created["project"])?;
    f.call(
        &[
            "project",
            "update",
            "--project",
            &p,
            "--expected-revision",
            "99",
            "--file",
            &config,
        ],
        5,
    )?;
    let d = f.call(&["discovery", "start", "--project", &p], 0)?;
    let inventory = f.file("inv.json", value(&d))?;
    let b = f.call(
        &["capture", "baseline", "--project", &p, "--file", &inventory],
        0,
    )?;
    let prepared = f.call(
        &[
            "baseline",
            "save",
            "--run",
            run(&b),
            "--expected-revision",
            "1",
            "--prepare",
        ],
        0,
    )?;
    let ticket = f.file("ticket.json", &prepared["data"]["ticket"])?;
    let confirmation = prepared["data"]["ticket"]["confirmation"]["id"]
        .as_str()
        .unwrap();
    f.call(
        &[
            "project",
            "update",
            "--project",
            &p,
            "--expected-revision",
            "1",
            "--file",
            &config,
        ],
        0,
    )?;
    f.call(
        &["confirm", "--file", &ticket, "--confirmation", confirmation],
        5,
    )?;
    f.call(
        &["capture", "baseline", "--project", &p, "--file", &inventory],
        5,
    )?;
    let existing = f.call(&["discovery", "existing", "--project", &p], 0)?;
    let existing_file = f.file("existing.json", value(&existing))?;
    let fresh = f.call(
        &[
            "capture",
            "baseline",
            "--project",
            &p,
            "--file",
            &existing_file,
        ],
        0,
    )?;
    f.prepare_confirm(&[
        "baseline",
        "save",
        "--run",
        run(&fresh),
        "--expected-revision",
        "2",
    ])?;
    fs::write(&f.state, "failed")?;
    let failed = f.call(&["capture", "check", "--project", &p], 3)?;
    assert_eq!(failed["code"], "incomplete_capture");
    let target = &value(&failed)["latest"]["run"]["targets"][0]["key"];
    let page = target["page"].as_str().unwrap();
    let profile = target["profile"].as_str().unwrap();
    let a = f.call(
        &[
            "results",
            "artifacts",
            "--project",
            &p,
            "--run",
            run(&failed),
            "--page",
            page,
            "--profile",
            profile,
        ],
        0,
    )?;
    assert!(value(&a)["after"].is_null());
    assert!(!value(&a)["warnings"].as_array().unwrap().is_empty());
    let crop_dest = f.root.path().join("not-created.png");
    f.call(
        &[
            "crop",
            "preview",
            "--project",
            &p,
            "--run",
            run(&failed),
            "--page",
            page,
            "--profile",
            profile,
            "--pane",
            "after",
            "--rect",
            "0,0,10,10",
            "--space",
            "screenshot-pixels",
            "--destination",
            crop_dest.to_str().unwrap(),
        ],
        3,
    )?;
    assert!(!crop_dest.exists());
    let selected = f.file("selected.json", &json!([target]))?;
    let unsafe_dest = f.client.paths.data.join("report");
    f.call(
        &[
            "report",
            "preview",
            "--project",
            &p,
            "--run",
            run(&failed),
            "--file",
            &selected,
            "--destination",
            unsafe_dest.to_str().unwrap(),
        ],
        2,
    )?;
    f.call(&["login", "open", "--project", &p], 4)?;
    assert!(!f.client.paths.data.join("private-sessions").exists());
    f.call(&["results", "list", "--project", &p, "--limit", "201"], 2)?;
    Ok(())
}

#[test]
fn detached_wait_ctrl_c_retry_pause_and_explicit_cancel_are_separate() -> Result<()> {
    let f = Fixture::start()?;
    let (p, _) = f.create()?;
    let d = f.call(&["discovery", "start", "--project", &p], 0)?;
    let inv = f.file("inv.json", value(&d))?;
    fs::write(&f.state, "slow")?;
    let request = uuid::Uuid::new_v4().to_string();
    let admitted = f.call(
        &[
            "capture",
            "baseline",
            "--project",
            &p,
            "--file",
            &inv,
            "--request-id",
            &request,
            "--detach",
        ],
        0,
    )?;
    assert_eq!(admitted["code"], "job_admitted");
    assert!(Path::new(admitted["data"]["retry_file"].as_str().unwrap()).exists());
    let wait = f
        .command(&[
            "jobs",
            "wait",
            "--job",
            &request,
            "--save",
            f.root.path().join("detached-wait.json").to_str().unwrap(),
        ])
        .stdout(Stdio::piped())
        .spawn()?;
    std::thread::sleep(Duration::from_millis(150));
    unsafe {
        libc::kill(wait.id() as i32, libc::SIGINT);
    }
    let out = wait.wait_with_output()?;
    assert_eq!(out.status.code(), Some(130));
    let detached: Value = serde_json::from_slice(&out.stdout)?;
    assert_eq!(detached["code"], "wait_detached");
    let saved: Value =
        serde_json::from_slice(&fs::read(f.root.path().join("detached-wait.json"))?)?;
    assert_eq!(
        saved, detached,
        "interrupted waits save a truthful diagnostic envelope, not null"
    );
    f.call(&["jobs", "pause", "--job", &request], 0)?;
    let paused = f.call(&["jobs", "wait", "--job", &request], 3)?;
    assert_eq!(paused["data"]["job"]["state"], "Paused");
    let run_id = run(&paused).to_string();
    let retried = f.call(&["jobs", "retry", "--request-id", &request], 3)?;
    assert_eq!(retried["data"]["job"]["id"], request);
    let resumed = f.call(
        &[
            "capture",
            "resume",
            "--project",
            &p,
            "--run",
            &run_id,
            "--detach",
        ],
        0,
    )?;
    let resumed_id = resumed["data"]["job"]["id"].as_str().unwrap();
    f.call(&["jobs", "cancel", "--job", resumed_id], 0)?;
    let cancelled = f.call(&["jobs", "wait", "--job", resumed_id], 3)?;
    assert_eq!(cancelled["data"]["job"]["state"], "Cancelled");
    f.call(&["jobs", "list", "--limit", "2"], 0)?;
    f.call(&["jobs", "ack", "--job", &request], 6)?;
    Ok(())
}

#[test]
fn metadata_help_and_parser_errors_are_data_free_clean_json() -> Result<()> {
    let root = tempfile::tempdir()?;
    let data = root.path().join("must-not-exist");
    for (args, exit) in [
        (vec!["capabilities"], 0),
        (vec!["--help"], 0),
        (vec!["--version"], 0),
        (vec!["--build-info"], 0),
        (vec!["project", "nonsense"], 2),
        (vec!["project", "list", "--limit", "1"], 2),
        (vec!["--unknown"], 2),
    ] {
        let out = Command::new(env!("CARGO_BIN_EXE_regressee"))
            .args(["--json", "--data-dir"])
            .arg(&data)
            .args(args)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .output()?;
        assert_eq!(
            out.status.code(),
            Some(exit),
            "{}",
            String::from_utf8_lossy(&out.stdout)
        );
        let envelope: Value = serde_json::from_slice(&out.stdout)?;
        assert_eq!(envelope["schema_version"], 1);
        assert!(!data.exists());
    }
    Ok(())
}

#[test]
fn typed_settings_stdin_diagnostics_appearance_and_input_bounds() -> Result<()> {
    let f = Fixture::start()?;
    let project = Project::draft("stdin project".into(), "http://127.0.0.1:43210/site")?;
    let mut child = f
        .command(&["project", "create", "--file", "-"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&project)?)?;
    let out = child.wait_with_output()?;
    assert!(out.status.success());
    let p = project.id.to_string();
    f.call(&["project", "list"], 0)?;
    f.call(&["profiles", "show", "--project", &p], 0)?;
    f.call(&["environments", "show", "--project", &p], 0)?;
    f.call(&["scope", "show", "--project", &p], 0)?;
    let profiles = f.file("profiles.json", &serde_json::to_value(&project.profiles)?)?;
    f.call(
        &[
            "profiles",
            "set",
            "--project",
            &p,
            "--expected-revision",
            "1",
            "--file",
            &profiles,
        ],
        0,
    )?;
    let environments = f.file(
        "environments.json",
        &serde_json::to_value(&project.environments)?,
    )?;
    f.call(
        &[
            "environments",
            "set",
            "--project",
            &p,
            "--expected-revision",
            "2",
            "--file",
            &environments,
        ],
        0,
    )?;
    let scope = f.file("scope.json", &serde_json::to_value(&project.discovery)?)?;
    f.call(
        &[
            "scope",
            "set",
            "--project",
            &p,
            "--expected-revision",
            "3",
            "--file",
            &scope,
        ],
        0,
    )?;
    f.call(
        &[
            "environment",
            "select",
            "--project",
            &p,
            "--environment",
            &project.environments[0].id.to_string(),
            "--expected-revision",
            "4",
        ],
        0,
    )?;
    f.call(&["environment", "status", "--project", &p], 0)?;
    f.call(&["appearance", "set", "--value", "dark"], 0)?;
    let appearance = f.call(&["appearance", "show"], 0)?;
    assert_eq!(appearance["data"], "dark");
    let network = f.call(&["network", "status"], 0)?;
    let network_file = f.file("network.json", &value(&network)["settings"])?;
    f.call(&["network", "set", "--file", &network_file], 0)?;
    f.call(&["browser", "status"], 0)?;
    let diagnostic = f.call(&["diagnostics", "preview"], 0)?;
    assert!(!value(&diagnostic).to_string().contains("127.0.0.1"));
    let diagnostic_file = f.file("diagnostic.json", value(&diagnostic))?;
    let destination = f.root.path().join("diagnostic-export.json");
    f.prepare_confirm(&[
        "diagnostics",
        "export",
        "--file",
        &diagnostic_file,
        "--destination",
        destination.to_str().unwrap(),
    ])?;
    let repair = f.call(&["repair", "preview"], 0)?;
    let repair_file = f.file("repair.json", value(&repair))?;
    f.prepare_confirm(&["repair", "clean", "--file", &repair_file])?;
    let oversized = f.root.path().join("oversized.json");
    let file = fs::File::create(&oversized)?;
    file.set_len(8 * 1024 * 1024 + 1)?;
    f.call(
        &["project", "create", "--file", oversized.to_str().unwrap()],
        2,
    )?;
    let save = f.root.path().join("existing-output.json");
    fs::write(&save, "keep")?;
    f.call(
        &[
            "project",
            "update",
            "--project",
            &p,
            "--expected-revision",
            "5",
            "--file",
            &f.file("current.json", &serde_json::to_value(&project)?)?,
            "--save",
            save.to_str().unwrap(),
        ],
        2,
    )?;
    assert_eq!(fs::read_to_string(save)?, "keep");
    Ok(())
}

#[test]
fn analysis_masks_stale_acceptance_reports_and_crop_path_integrity() -> Result<()> {
    let f = Fixture::start()?;
    let (p, _) = f.create()?;
    let d = f.call(&["discovery", "start", "--project", &p], 0)?;
    let inv = f.file("inv.json", value(&d))?;
    let second = value(&d)["inventory"]["pages"][1]["id"].as_str().unwrap();
    let excluded = f.call(
        &["discovery", "exclude", "--file", &inv, "--pages", second],
        0,
    )?;
    assert_eq!(value(&excluded)["inventory"]["pages"][1]["included"], false);
    let excluded_file = f.file("excluded.json", value(&excluded))?;
    let included = f.call(
        &[
            "discovery",
            "include",
            "--file",
            &excluded_file,
            "--pages",
            second,
        ],
        0,
    )?;
    assert_eq!(value(&included)["inventory"]["pages"][1]["included"], true);
    let b = f.call(&["capture", "baseline", "--project", &p, "--file", &inv], 0)?;
    f.prepare_confirm(&[
        "baseline",
        "save",
        "--run",
        run(&b),
        "--expected-revision",
        "1",
    ])?;
    fs::write(&f.state, "changed")?;
    let c = f.call(&["capture", "check", "--project", &p], 1)?;
    let r = run(&c);
    let analysis = value(&c)["comparison"]["revision"]["id"].as_str().unwrap();
    let key = value(&c)["latest"]["run"]["targets"][0]["key"].clone();
    let page = key["page"].as_str().unwrap();
    let profile = key["profile"].as_str().unwrap();
    let selected = f.file("keys.json", &json!([key]))?;
    let old = f.call(
        &[
            "baseline",
            "preview",
            "--analysis",
            analysis,
            "--file",
            &selected,
        ],
        0,
    )?;
    let old_file = f.file("old-approval.json", value(&old))?;
    let dest = f.root.path().join("stale-report");
    let preview = f.call(
        &[
            "report",
            "preview",
            "--project",
            &p,
            "--run",
            r,
            "--file",
            &selected,
            "--destination",
            dest.to_str().unwrap(),
        ],
        0,
    )?;
    let preview_file = f.file("old-report.json", value(&preview))?;
    let a = f.call(
        &[
            "results",
            "artifacts",
            "--project",
            &p,
            "--run",
            r,
            "--page",
            page,
            "--profile",
            profile,
        ],
        0,
    )?;
    let mut mask = value(&a)["analysis_report"]["request"].clone();
    mask["mask"] = json!({"revision":"owned-mask-v1","target":mask["before"]["target"],"before_capture_id":mask["before"]["capture_id"],"after_capture_id":mask["after"]["capture_id"],"before_dimensions":mask["before"]["dimensions"],"after_dimensions":mask["after"]["dimensions"],"rectangles":[{"x":0,"y":0,"width":8,"height":8}]});
    let mask_file = f.file("mask.json", &mask)?;
    f.call(&["masks", "save", "--project", &p, "--file", &mask_file], 0)?;
    let reanalyzed = f.call(
        &["analysis", "reanalyze", "--run", r, "--preset", "sensitive"],
        0,
    )?;
    let new_analysis = value(&reanalyzed)["revision"]["id"].as_str().unwrap();
    assert_ne!(analysis, new_analysis);
    f.call(&["analysis", "show", "--analysis", analysis], 0)?;
    let prepared = f.call(&["baseline", "accept", "--file", &old_file, "--prepare"], 0)?;
    let ticket = f.file("old-ticket.json", &prepared["data"]["ticket"])?;
    f.call(
        &[
            "confirm",
            "--file",
            &ticket,
            "--confirmation",
            prepared["data"]["ticket"]["confirmation"]["id"]
                .as_str()
                .unwrap(),
        ],
        5,
    )?;
    let prepared = f.call(
        &["report", "export", "--file", &preview_file, "--prepare"],
        0,
    )?;
    let ticket = f.file("report-ticket.json", &prepared["data"]["ticket"])?;
    f.call(
        &[
            "confirm",
            "--file",
            &ticket,
            "--confirmation",
            prepared["data"]["ticket"]["confirmation"]["id"]
                .as_str()
                .unwrap(),
        ],
        5,
    )?;
    assert!(!dest.exists());
    f.call(&["analysis", "select", "--analysis", new_analysis], 0)?;
    let dest = f.root.path().join("crop.png");
    for (rect, space) in [
        ("0,0,65,48", "screenshot-pixels"),
        ("0,0,10,10", "css-pixels"),
        ("4294967295,0,10,10", "screenshot-pixels"),
    ] {
        f.call(
            &[
                "crop",
                "preview",
                "--project",
                &p,
                "--run",
                r,
                "--page",
                page,
                "--profile",
                profile,
                "--pane",
                "after",
                "--rect",
                rect,
                "--space",
                space,
                "--destination",
                dest.to_str().unwrap(),
            ],
            2,
        )?;
    }
    let alias = f.root.path().join("alias");
    std::os::unix::fs::symlink(f.root.path(), &alias)?;
    let aliased = alias.join("report");
    f.call(
        &[
            "report",
            "preview",
            "--project",
            &p,
            "--run",
            r,
            "--file",
            &selected,
            "--destination",
            aliased.to_str().unwrap(),
        ],
        2,
    )?;
    let mut changed_project =
        value(&f.call(&["project", "show", "--project", &p], 0)?)["project"].clone();
    changed_project["profiles"][0]["width"] = json!(1000);
    let replacement = f.file("replacement.json", &changed_project)?;
    f.call(
        &[
            "project",
            "update",
            "--project",
            &p,
            "--expected-revision",
            "2",
            "--file",
            &replacement,
        ],
        4,
    )?;
    f.prepare_confirm(&[
        "project",
        "update",
        "--project",
        &p,
        "--expected-revision",
        "2",
        "--file",
        &replacement,
        "--replacement",
    ])?;
    Ok(())
}

#[test]
fn corrupted_evidence_and_forged_crop_sources_fail_without_output_or_overwrite() -> Result<()> {
    let f = Fixture::start()?;
    let (p, _) = f.create()?;
    let d = f.call(&["discovery", "start", "--project", &p], 0)?;
    let inv = f.file("inv.json", value(&d))?;
    let b = f.call(&["capture", "baseline", "--project", &p, "--file", &inv], 0)?;
    let target = &value(&b)["latest"]["run"]["targets"][0]["key"];
    let page = target["page"].as_str().unwrap();
    let profile = target["profile"].as_str().unwrap();
    let destination = f.root.path().join("crop.png");
    let preview = f.call(
        &[
            "crop",
            "preview",
            "--project",
            &p,
            "--run",
            run(&b),
            "--page",
            page,
            "--profile",
            profile,
            "--pane",
            "after",
            "--rect",
            "0,0,16,16",
            "--space",
            "screenshot-pixels",
            "--destination",
            destination.to_str().unwrap(),
        ],
        0,
    )?;
    let mut forged = value(&preview).clone();
    let original = forged["source"]["original_path"]
        .as_str()
        .unwrap()
        .to_string();
    let external = f.root.path().join("not-evidence.png");
    fs::write(&external, fs::read(&original)?)?;
    forged["source"]["original_path"] = json!(external);
    let file = f.file("forged.json", &forged)?;
    let prepared = f.call(&["crop", "export", "--file", &file, "--prepare"], 0)?;
    let ticket = f.file("forged-ticket.json", &prepared["data"]["ticket"])?;
    f.call(
        &[
            "confirm",
            "--file",
            &ticket,
            "--confirmation",
            prepared["data"]["ticket"]["confirmation"]["id"]
                .as_str()
                .unwrap(),
        ],
        5,
    )?;
    assert!(!destination.exists());
    let file = f.file("valid-crop.json", value(&preview))?;
    let prepared = f.call(&["crop", "export", "--file", &file, "--prepare"], 0)?;
    let ticket = f.file("valid-ticket.json", &prepared["data"]["ticket"])?;
    let mut wrong_action = prepared["data"]["ticket"].clone();
    wrong_action["operation"] = json!({"operation":"SetAppearance","arguments":"dark"});
    let wrong_file = f.file("wrong-action-ticket.json", &wrong_action)?;
    f.call(
        &[
            "confirm",
            "--file",
            &wrong_file,
            "--confirmation",
            prepared["data"]["ticket"]["confirmation"]["id"]
                .as_str()
                .unwrap(),
        ],
        5,
    )?;
    f.call(
        &[
            "confirm",
            "--file",
            &ticket,
            "--confirmation",
            &uuid::Uuid::new_v4().to_string(),
        ],
        5,
    )?;
    fs::write(&original, b"deliberately damaged owned synthetic PNG")?;
    f.call(
        &[
            "confirm",
            "--file",
            &ticket,
            "--confirmation",
            prepared["data"]["ticket"]["confirmation"]["id"]
                .as_str()
                .unwrap(),
        ],
        6,
    )?;
    assert!(!destination.exists());
    f.call(
        &[
            "results",
            "artifacts",
            "--project",
            &p,
            "--run",
            run(&b),
            "--page",
            page,
            "--profile",
            profile,
        ],
        6,
    )?;
    assert!(external.exists());
    Ok(())
}

#[test]
fn production_autostart_read_only_operations_never_install_runtime_or_credentials() -> Result<()> {
    let root = tempfile::tempdir()?;
    let data = root.path().join("data");
    let cache = root.path().join("cache");
    let paths = AppPaths {
        data: data.clone(),
        cache: cache.clone(),
    };
    for args in [
        vec!["browser", "status"],
        vec!["network", "status"],
        vec!["diagnostics", "preview"],
        vec!["project", "list"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_regressee"))
            .args(["--json", "--data-dir"])
            .arg(&data)
            .arg("--cache-dir")
            .arg(&cache)
            .args(args)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .env_remove("REGRESSEE_DISTRIBUTION")
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        let _: Value = serde_json::from_slice(&output.stdout)?;
        assert!(!cache.join("product-runtime").exists());
        assert!(!data.join("private-sessions").exists());
    }
    assert!(matches!(
        Client::existing(paths)?.request(Request::Stop)?,
        Response::Ok
    ));
    Ok(())
}
