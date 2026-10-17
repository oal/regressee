// SPDX-License-Identifier: MPL-2.0
//! Independent-review defects, exercised through the public subprocess contract.
use super::*;

fn changed(f: &Fixture) -> Result<(String, String, String, String, Value)> {
    let (p, _) = f.create()?;
    let d = f.call(&["discovery", "start", "--project", &p], 0)?;
    let inv = f.file("repair-inventory.json", value(&d))?;
    let b = f.call(&["capture", "baseline", "--project", &p, "--file", &inv], 0)?;
    let saved = f.prepare_confirm(&[
        "baseline",
        "save",
        "--run",
        run(&b),
        "--expected-revision",
        "1",
    ])?;
    let baseline = value(&saved)["project"]["active_baseline"]
        .as_str()
        .unwrap()
        .into();
    fs::write(&f.state, "changed")?;
    let c = f.call(&["capture", "check", "--project", &p], 1)?;
    Ok((
        p,
        run(&c).into(),
        value(&c)["comparison"]["revision"]["id"]
            .as_str()
            .unwrap()
            .into(),
        baseline,
        value(&c)["latest"]["run"]["targets"][0]["key"].clone(),
    ))
}

#[test]
fn default_evidence_and_exports_keep_original_reference_after_accept_and_restore() -> Result<()> {
    let f = Fixture::start()?;
    let (p, r, a, b0, key) = changed(&f)?;
    let page = key["page"].as_str().unwrap();
    let profile = key["profile"].as_str().unwrap();
    let explicit = f.call(
        &[
            "results",
            "artifacts",
            "--project",
            &p,
            "--run",
            &r,
            "--analysis",
            &a,
            "--page",
            page,
            "--profile",
            profile,
        ],
        0,
    )?;
    let keys = f.file("repair-keys.json", &json!([key]))?;
    let approval = f.call(
        &["baseline", "preview", "--analysis", &a, "--file", &keys],
        0,
    )?;
    let approval_file = f.file("repair-approval.json", value(&approval))?;
    let accepted = f.prepare_confirm(&["baseline", "accept", "--file", &approval_file])?;
    let b1 = value(&accepted)["project"]["active_baseline"]
        .as_str()
        .unwrap()
        .to_owned();
    for (n, restore) in [None, Some(b0.as_str()), Some(b1.as_str())]
        .into_iter()
        .enumerate()
    {
        if let Some(b) = restore {
            let revision =
                value(&f.call(&["project", "show", "--project", &p], 0)?)["revision"].to_string();
            f.prepare_confirm(&[
                "baseline",
                "restore",
                "--baseline",
                b,
                "--expected-revision",
                &revision,
            ])?;
        }
        let default = f.call(
            &[
                "results",
                "artifacts",
                "--project",
                &p,
                "--page",
                page,
                "--profile",
                profile,
            ],
            0,
        )?;
        assert_eq!(
            value(&default),
            value(&explicit),
            "default must be exactly the historical R/A/B0 evidence"
        );
        assert_eq!(value(&default)["baseline"], b0);
        assert!(value(&default)["before_run"].is_string());
        let rows = f.call(&["results", "list", "--project", &p], 0)?;
        assert_eq!(value(&rows)["baseline"], b0);
        let dest = f.root.path().join(format!("repair-default-report-{n}"));
        let preview = f.call(
            &[
                "report",
                "preview",
                "--project",
                &p,
                "--file",
                &keys,
                "--destination",
                dest.to_str().unwrap(),
            ],
            0,
        )?;
        assert_eq!(value(&preview)["content"]["baseline"]["id"], b0);
        assert_eq!(
            value(&preview)["content"]["analysis"]["revision"]["baseline"],
            b0
        );
        let file = f.file(&format!("repair-default-preview-{n}.json"), value(&preview))?;
        f.prepare_confirm(&["report", "export", "--file", &file])?;
        let crop = f.root.path().join(format!("repair-before-{n}.png"));
        let preview = f.call(
            &[
                "crop",
                "preview",
                "--project",
                &p,
                "--page",
                page,
                "--profile",
                profile,
                "--pane",
                "before",
                "--space",
                "screenshot-pixels",
                "--rect",
                "0,0,8,8",
                "--destination",
                crop.to_str().unwrap(),
            ],
            0,
        )?;
        assert_eq!(value(&preview)["source"], value(&explicit)["before"]);
        let file = f.file(&format!("repair-crop-preview-{n}.json"), value(&preview))?;
        f.prepare_confirm(&["crop", "export", "--file", &file])?;
    }
    // An explicit older analysis without --run also determines its own run.
    let old = f.call(
        &[
            "analysis",
            "reanalyze",
            "--run",
            &r,
            "--preset",
            "sensitive",
        ],
        0,
    )?;
    assert_ne!(value(&old)["revision"]["id"], a);
    let pinned = f.call(&["results", "list", "--project", &p, "--analysis", &a], 0)?;
    assert_eq!(value(&pinned)["selection"]["analysis"], a);
    Ok(())
}

#[test]
fn unrelated_review_activity_preserves_current_notes_and_complete_paginated_history() -> Result<()>
{
    let f = Fixture::start()?;
    let (p, r, a, _, key) = changed(&f)?;
    let page = key["page"].as_str().unwrap();
    let profile = key["profile"].as_str().unwrap();
    f.call(
        &[
            "review",
            "set",
            "--analysis",
            &a,
            "--page",
            page,
            "--profile",
            profile,
            "--state",
            "needs-fix",
            "--note",
            "KEEP_FIRST_TARGET_NOTE",
        ],
        0,
    )?;
    let rows = f.call(&["results", "list", "--project", &p, "--run", &r], 0)?;
    let other: TargetKey =
        serde_json::from_value(value(&rows)["rows"][1]["target"]["key"].clone())?;
    let analysis: AnalysisId = serde_json::from_value(json!(a))?;
    for n in 0..256 {
        let saved = f.client.execute(
            Operation::Review(ReviewDecision {
                analysis,
                key: other.clone(),
                state: ReviewState::Intentional,
                note: format!("unrelated {n}"),
                reviewed_at: 1000 + n,
            }),
            None,
        )?;
        assert_eq!(saved.state, regressee_app::protocol::JobState::Succeeded);
    }
    let rows = f.call(
        &[
            "results",
            "list",
            "--project",
            &p,
            "--run",
            &r,
            "--analysis",
            &a,
            "--page",
            page,
        ],
        0,
    )?;
    assert!(value(&rows).to_string().contains("KEEP_FIRST_TARGET_NOTE"));
    let keys = f.file("repair-review-keys.json", &json!([key]))?;
    let dest = f.root.path().join("repair-review-report");
    let report = f.call(
        &[
            "report",
            "preview",
            "--project",
            &p,
            "--run",
            &r,
            "--analysis",
            &a,
            "--file",
            &keys,
            "--destination",
            dest.to_str().unwrap(),
        ],
        0,
    )?;
    let file = f.file("repair-review-preview.json", value(&report))?;
    f.prepare_confirm(&["report", "export", "--file", &file])?;
    for name in ["index.html", "summary.md"] {
        assert!(fs::read_to_string(dest.join(name))?.contains("KEEP_FIRST_TARGET_NOTE"));
    }
    let history = f.call(&["history", "show", "--project", &p, "--run", &r], 0)?;
    assert_eq!(
        value(&history)["evidence"]["review_history_truncated"],
        true
    );
    let mut before = None;
    let mut seen = std::collections::BTreeSet::new();
    let mut found = false;
    loop {
        let mut args = vec!["review", "history", "--analysis", &a, "--limit", "100"];
        let cursor = before.map(|v: i64| v.to_string());
        if let Some(cursor) = &cursor {
            args.extend(["--before", cursor]);
        }
        let page = f.call(&args, 0)?;
        let page = value(&page);
        assert_eq!(page["total"], 257);
        for entry in page["items"].as_array().unwrap() {
            assert!(seen.insert(entry["id"].as_i64().unwrap()));
            found |= entry["decision"]["note"] == "KEEP_FIRST_TARGET_NOTE";
        }
        before = page["next_before"].as_i64();
        if before.is_none() {
            break;
        }
    }
    assert!(found);
    assert_eq!(seen.len(), 257);
    let filtered = f.call(
        &[
            "review",
            "history",
            "--analysis",
            &a,
            "--page",
            page,
            "--profile",
            profile,
        ],
        0,
    )?;
    assert_eq!(value(&filtered)["total"], 1);
    let new = f.call(
        &["analysis", "reanalyze", "--run", &r, "--preset", "relaxed"],
        0,
    )?;
    let new_a = value(&new)["revision"]["id"].as_str().unwrap();
    let empty = f.call(
        &[
            "results",
            "list",
            "--project",
            &p,
            "--run",
            &r,
            "--analysis",
            new_a,
            "--page",
            page,
        ],
        0,
    )?;
    assert!(
        value(&empty)["rows"][0]["reviews"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    let pinned = f.call(
        &[
            "results",
            "list",
            "--project",
            &p,
            "--run",
            &r,
            "--analysis",
            &a,
            "--page",
            page,
        ],
        0,
    )?;
    assert!(
        value(&pinned)
            .to_string()
            .contains("KEEP_FIRST_TARGET_NOTE")
    );
    Ok(())
}

#[test]
fn prepared_job_show_wait_and_retry_recover_identical_confirmable_ticket() -> Result<()> {
    let f = Fixture::start()?;
    let (p, _, _, b, _) = changed(&f)?;
    let revision =
        value(&f.call(&["project", "show", "--project", &p], 0)?)["revision"].to_string();
    let id = uuid::Uuid::new_v4().to_string();
    let original = f.call(
        &[
            "baseline",
            "restore",
            "--baseline",
            &b,
            "--expected-revision",
            &revision,
            "--prepare",
            "--request-id",
            &id,
        ],
        0,
    )?;
    let ticket = &original["data"]["ticket"];
    for action in ["show", "wait", "retry"] {
        let path = f
            .root
            .path()
            .join(format!("repair-recovered-{action}.json"));
        let flag = if action == "retry" {
            "--request-id"
        } else {
            "--job"
        };
        let recovered = f.call(
            &["jobs", action, flag, &id, "--save", path.to_str().unwrap()],
            0,
        )?;
        assert_eq!(&recovered["data"]["ticket"], ticket);
        assert_eq!(serde_json::from_slice::<Value>(&fs::read(path)?)?, *ticket);
    }
    let file = f.root.path().join("repair-recovered-retry.json");
    f.call(
        &[
            "confirm",
            "--file",
            file.to_str().unwrap(),
            "--confirmation",
            ticket["confirmation"]["id"].as_str().unwrap(),
        ],
        0,
    )?;
    let again = f.call(&["jobs", "retry", "--request-id", &id], 0)?;
    assert_eq!(&again["data"]["ticket"], ticket);
    f.call(
        &[
            "confirm",
            "--file",
            file.to_str().unwrap(),
            "--confirmation",
            ticket["confirmation"]["id"].as_str().unwrap(),
        ],
        5,
    )?;
    Ok(())
}

fn wait_for_operation(
    f: &Fixture,
    prior: &std::collections::BTreeSet<uuid::Uuid>,
    name: &str,
) -> Result<uuid::Uuid> {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        for info in f.client.snapshot()?.jobs {
            if !prior.contains(&info.id) {
                let j = match f.client.request(Request::Job { id: info.id })? {
                    Response::Job(j) => *j,
                    Response::Error { code, .. } if code == "not_found" => continue, // a completed prerequisite read may have been acknowledged
                    other => anyhow::bail!("Unexpected job response: {other:?}"),
                };
                if serde_json::to_value(j.submitted.operation)?["operation"] == name {
                    return Ok(info.id);
                }
            }
        }
        assert!(
            Instant::now() < deadline,
            "prerequisite {name} was not admitted"
        );
        std::thread::sleep(Duration::from_millis(10));
    }
}
fn interrupt_child(mut child: std::process::Child) -> Result<Value> {
    let start = Instant::now();
    unsafe {
        libc::kill(child.id() as i32, libc::SIGINT);
    }
    while child.try_wait()?.is_none() && start.elapsed() < Duration::from_millis(700) {
        std::thread::sleep(Duration::from_millis(10));
    }
    let prompt = child.try_wait()?.is_some();
    let out = child.wait_with_output()?;
    assert!(prompt, "Ctrl-C did not detach prerequisite wait promptly");
    assert_eq!(out.status.code(), Some(130));
    Ok(serde_json::from_slice(&out.stdout)?)
}
#[test]
fn every_configuration_prerequisite_detaches_without_submitting_continuation() -> Result<()> {
    let f = Fixture::start()?;
    let (p, saved) = f.create()?;
    let project = &saved["project"];
    let env = project["environments"][0]["id"].as_str().unwrap();
    let login = f.file(
        "repair-login-settings.json",
        &serde_json::to_value(regressee_storage::LocalLoginSettings::default())?,
    )?;
    for (area, field) in [
        ("profiles", "profiles"),
        ("environments", "environments"),
        ("scope", "discovery"),
        ("environment", "select"),
        ("environment", "select-explicit"),
    ] {
        let file = f.file(&format!("repair-{area}.json"), &project[field])?;
        let prior = f
            .client
            .snapshot()?
            .jobs
            .into_iter()
            .map(|j| j.id)
            .collect();
        let blocker = f.client.submit(Submit {
            request: uuid::Uuid::new_v4(),
            engine: f.client.snapshot()?.engine,
            epoch: 0,
            operation: Operation::TestWait { millis: 1800 },
            confirmation: None,
            expected_revision: None,
        })?;
        let mut args = if area == "environment" {
            vec![
                area,
                "select",
                "--project",
                &p,
                "--expected-revision",
                "1",
                "--environment",
                env,
            ]
        } else {
            vec![
                area,
                "set",
                "--project",
                &p,
                "--expected-revision",
                "1",
                "--file",
                &file,
            ]
        };
        if field == "select-explicit" {
            args.extend(["--login-settings", &login]);
        }
        let child = f
            .command(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let read = wait_for_operation(&f, &prior, "Load")?;
        std::thread::sleep(Duration::from_millis(30));
        let out = interrupt_child(child)?;
        assert_eq!(out["data"]["job"]["id"], read.to_string());
        assert_eq!(out["data"]["continuation_submitted"], false);
        assert!(out["data"]["retry_file"].is_string());
        f.client.wait(blocker.id)?;
        f.client.wait(read)?;
        assert_eq!(
            value(&f.call(&["project", "show", "--project", &p], 0)?)["revision"],
            1
        );
        for info in f.client.snapshot()?.jobs {
            if !prior.contains(&info.id) {
                let j = f.client.job(info.id)?;
                assert!(!matches!(
                    j.submitted.operation,
                    Operation::Save { .. }
                        | Operation::SaveEnvironment { .. }
                        | Operation::SaveReplacement { .. }
                ));
            }
        }
    }
    // The second environment prerequisite must also remain interruptible.
    let prior = f
        .client
        .snapshot()?
        .jobs
        .into_iter()
        .map(|j| j.id)
        .collect();
    let s = f.client.snapshot()?;
    let first = f.client.submit(Submit {
        request: uuid::Uuid::new_v4(),
        engine: s.engine,
        epoch: s.epoch,
        operation: Operation::TestWait { millis: 800 },
        confirmation: None,
        expected_revision: None,
    })?;
    let child = f
        .command(&[
            "environment",
            "select",
            "--project",
            &p,
            "--expected-revision",
            "1",
            "--environment",
            env,
        ])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    wait_for_operation(&f, &prior, "Load")?;
    let second = f.client.submit(Submit {
        request: uuid::Uuid::new_v4(),
        engine: s.engine,
        epoch: s.epoch,
        operation: Operation::TestWait { millis: 1800 },
        confirmation: None,
        expected_revision: None,
    })?;
    let read = wait_for_operation(&f, &prior, "EnvironmentStatus")?;
    let out = interrupt_child(child)?;
    assert_eq!(out["data"]["job"]["id"], read.to_string());
    assert_eq!(out["data"]["continuation_submitted"], false);
    f.client.wait(first.id)?;
    f.client.wait(second.id)?;
    f.client.wait(read)?;
    assert_eq!(
        value(&f.call(&["project", "show", "--project", &p], 0)?)["revision"],
        1
    );
    Ok(())
}

#[test]
fn domain_invalid_arguments_and_inapplicable_flags_never_start_storage() -> Result<()> {
    let root = tempfile::tempdir()?;
    let data = root.path().join("must-not-exist");
    let id = "00000000-0000-0000-0000-000000000001";
    let mut cases = vec![
        vec![
            "project",
            "create",
            "--name",
            "",
            "--url",
            "http://127.0.0.1/",
        ],
        vec!["project", "create", "--name", "x", "--url", "not-a-url"],
        vec![
            "project",
            "create",
            "--name",
            "x",
            "--url",
            "https://user:secret@example.invalid/",
        ],
        vec!["project", "list", "--prepare"],
        vec!["appearance", "set", "--value", "dark", "--prepare"],
        vec![
            "environment",
            "select",
            "--project",
            id,
            "--environment",
            id,
            "--expected-revision",
            "1",
            "--prepare",
        ],
        vec!["results", "list", "--project", id, "--limit", "201"],
        vec!["jobs", "list", "--limit", "0"],
        vec![
            "project",
            "list",
            "--request-id",
            "00000000-0000-0000-0000-000000000000",
        ],
    ];
    let bad = root.path().join("invalid-profiles.json");
    fs::write(&bad, r#"[{"width":0}]"#)?;
    cases.push(vec![
        "profiles",
        "set",
        "--project",
        id,
        "--expected-revision",
        "1",
        "--file",
        bad.to_str().unwrap(),
    ]);
    let mut project = Project::draft("Local validation".into(), "https://validation.invalid/")?;
    project.id = ProjectId(uuid::Uuid::parse_str(id)?);
    let valid_profiles = root.path().join("valid-profiles.json");
    fs::write(&valid_profiles, serde_json::to_vec(&project.profiles)?)?;
    cases.push(vec![
        "profiles",
        "set",
        "--project",
        id,
        "--expected-revision",
        "1",
        "--file",
        valid_profiles.to_str().unwrap(),
        "--prepare",
    ]);
    project.profiles[0].width = 0;
    let full_bad = root.path().join("domain-invalid-profiles.json");
    fs::write(&full_bad, serde_json::to_vec(&project.profiles)?)?;
    cases.push(vec![
        "profiles",
        "set",
        "--project",
        id,
        "--expected-revision",
        "1",
        "--file",
        full_bad.to_str().unwrap(),
    ]);
    let update = root.path().join("domain-invalid-project.json");
    fs::write(&update, serde_json::to_vec(&project)?)?;
    cases.push(vec![
        "project",
        "update",
        "--project",
        id,
        "--expected-revision",
        "1",
        "--file",
        update.to_str().unwrap(),
    ]);
    let scope = root.path().join("domain-invalid-scope.json");
    project.discovery.page_limit = 0;
    fs::write(&scope, serde_json::to_vec(&project.discovery)?)?;
    cases.push(vec![
        "scope",
        "set",
        "--project",
        id,
        "--expected-revision",
        "1",
        "--file",
        scope.to_str().unwrap(),
    ]);
    let envs = root.path().join("empty-environments.json");
    fs::write(&envs, "[]")?;
    cases.push(vec![
        "environments",
        "set",
        "--project",
        id,
        "--expected-revision",
        "1",
        "--file",
        envs.to_str().unwrap(),
    ]);
    for args in cases {
        let out = Command::new(env!("CARGO_BIN_EXE_regressee"))
            .args(["--json", "--data-dir"])
            .arg(&data)
            .args(&args)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .output()?;
        let e: Value = serde_json::from_slice(&out.stdout)?;
        assert_eq!(out.status.code(), Some(2), "{args:?}: {e}");
        assert_eq!(e["code"], "invalid_request");
        assert!(!data.exists(), "invalid command started storage: {args:?}");
    }
    Ok(())
}

#[test]
fn reports_export_frozen_queue_and_reference_settings_not_current_configuration() -> Result<()> {
    let f = Fixture::start()?;
    let (p, r, a, _, key) = changed(&f)?;
    let history = f.call(&["history", "show", "--project", &p, "--run", &r], 0)?;
    let recorded = &value(&history)["evidence"]["latest"];
    assert!(recorded["queue"].is_object());
    let mut project = value(&f.call(&["project", "show", "--project", &p], 0)?)["project"].clone();
    project["discovery"]["page_limit"] = json!(7);
    project["discovery"]["skipped_text"] = json!(["TODAYS_SETTINGS_NOT_RECORDED"]);
    let file = f.file("repair-new-settings.json", &project)?;
    f.call(
        &[
            "project",
            "update",
            "--project",
            &p,
            "--expected-revision",
            "2",
            "--file",
            &file,
        ],
        0,
    )?;
    let keys = f.file("repair-settings-keys.json", &json!([key]))?;
    let dest = f.root.path().join("repair-settings-report");
    let preview = f.call(
        &[
            "report",
            "preview",
            "--project",
            &p,
            "--run",
            &r,
            "--analysis",
            &a,
            "--file",
            &keys,
            "--destination",
            dest.to_str().unwrap(),
        ],
        0,
    )?;
    let content = &value(&preview)["content"];
    assert_eq!(content["queue"], recorded["queue"]);
    assert!(!content.to_string().contains("TODAYS_SETTINGS_NOT_RECORDED"));
    assert!(content["reference_settings"][0]["queue"].is_object());
    let file = f.file("repair-settings-preview.json", value(&preview))?;
    f.prepare_confirm(&["report", "export", "--file", &file])?;
    let html = fs::read_to_string(dest.join("index.html"))?;
    assert!(html.contains("project_revision"));
    assert!(html.contains("resumed_at"));
    assert!(!html.contains("TODAYS_SETTINGS_NOT_RECORDED"));
    Ok(())
}
