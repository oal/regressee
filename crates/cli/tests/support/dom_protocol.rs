// SPDX-License-Identifier: MPL-2.0
//! Actual public CLI capture path over signed v3 synthetic channels, not Chromium.
use super::*;

struct Baseline {
    project: String,
    baseline: String,
    revision: String,
    keys: Vec<Value>,
}

fn baseline(f: &Fixture, mode: &str) -> Result<Baseline> {
    let (project, _) = f.create()?;
    let configured = f.call(
        &[
            "checks",
            "set",
            "--project",
            &project,
            "--expected-revision",
            "1",
            "--mode",
            mode,
        ],
        0,
    )?;
    let mut settings = value(&configured)["project"].clone();
    settings["discovery"]["speed"] = "Fast".into();
    settings["discovery"]["advanced"]["retries"] = 0.into();
    let file = f.file("dom-settings.json", &settings)?;
    f.call(
        &[
            "project",
            "update",
            "--project",
            &project,
            "--expected-revision",
            "2",
            "--file",
            &file,
        ],
        0,
    )?;
    let discovered = f.call(&["discovery", "start", "--project", &project], 0)?;
    let inventory = f.file("dom-inventory.json", value(&discovered))?;
    let captured = f.call(
        &[
            "capture",
            "baseline",
            "--project",
            &project,
            "--file",
            &inventory,
        ],
        0,
    )?;
    let keys = value(&captured)["latest"]["run"]["targets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|t| t["key"].clone())
        .collect();
    let saved = f.prepare_confirm(&[
        "baseline",
        "save",
        "--run",
        run(&captured),
        "--expected-revision",
        "3",
    ])?;
    Ok(Baseline {
        project,
        baseline: value(&saved)["project"]["active_baseline"]
            .as_str()
            .unwrap()
            .into(),
        revision: value(&saved)["revision"].as_u64().unwrap().to_string(),
        keys,
    })
}

fn mode(evidence: &Value) -> &str {
    evidence["latest"]["run"]["capture_plan"]["mode"]
        .as_str()
        .unwrap_or("ScreenshotsOnly")
}

#[test]
fn v3_corrupt_dom_sibling_does_not_block_screenshot_reads_crops_or_viewer() -> Result<()> {
    let f = Fixture::start()?;
    let b = baseline(&f, "both")?;
    fs::write(&f.state, "changed")?;
    let checked = f.call(&["capture", "check", "--project", &b.project], 1)?;
    let checked_run = run(&checked).to_owned();
    let image_analysis = value(&checked)["comparison"]["revision"]["id"]
        .as_str()
        .unwrap();
    let key = &b.keys[0];
    let page = key["page"].as_str().unwrap();
    let profile = key["profile"].as_str().unwrap();
    let dom = f.call(
        &[
            "results",
            "artifacts",
            "--kind",
            "dom",
            "--project",
            &b.project,
            "--run",
            &checked_run,
            "--page",
            page,
            "--profile",
            profile,
        ],
        0,
    )?;
    let id = value(&dom)["after"]["artifact"]["id"].as_str().unwrap();
    fs::write(
        f.client.paths.artifacts().join(id),
        b"deliberately damaged owned DOM original",
    )?;
    f.call(
        &[
            "results",
            "list",
            "--project",
            &b.project,
            "--run",
            &checked_run,
        ],
        0,
    )?;
    let images = f.call(
        &[
            "results",
            "artifacts",
            "--project",
            &b.project,
            "--run",
            &checked_run,
            "--page",
            page,
            "--profile",
            profile,
        ],
        0,
    )?;
    let crop = f.root.path().join("dom-corrupt-sibling-crop.png");
    let preview = f.call(
        &[
            "crop",
            "preview",
            "--project",
            &b.project,
            "--run",
            &checked_run,
            "--page",
            page,
            "--profile",
            profile,
            "--pane",
            "after",
            "--rect",
            "0,0,8,8",
            "--space",
            "screenshot-pixels",
            "--destination",
            crop.to_str().unwrap(),
        ],
        0,
    )?;
    let file = f.file("dom-corrupt-crop.json", value(&preview))?;
    f.prepare_confirm(&["crop", "export", "--file", &file])?;
    assert!(crop.is_file());
    let started = f.call(
        &["agent", "start", "--name", "Owned screenshot sibling check"],
        0,
    )?;
    let session = started["data"]["session"].as_str().unwrap();
    let selection = f.call(
        &[
            "--session",
            session,
            "agent",
            "select",
            "--project",
            &b.project,
            "--run",
            &checked_run,
            "--analysis",
            image_analysis,
            "--page",
            page,
            "--profile",
            profile,
        ],
        0,
    );
    f.call(&["--session", session, "agent", "end"], 0)?;
    assert_eq!(value(&selection?)["selection"]["kind"], "Screenshot");
    let snapshot = f.client.snapshot()?;
    let submitted = Submit {
        request: uuid::Uuid::new_v4(),
        engine: snapshot.engine,
        epoch: snapshot.epoch,
        operation: Operation::Viewer(ViewerSelection {
            project: serde_json::from_value(json!(&b.project))?,
            run: Some(serde_json::from_value(json!(&checked_run))?),
            tile: 0,
            key: Some(serde_json::from_value(key.clone())?),
            analysis: None,
        }),
        confirmation: None,
        expected_revision: None,
    };
    let mut job = f.client.submit(submitted)?;
    let deadline = Instant::now() + Duration::from_secs(5);
    while !job.state.terminal() {
        assert!(Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(10));
        job = match f.client.request(Request::Job { id: job.id })? {
            Response::Job(job) => *job,
            _ => anyhow::bail!("unexpected owned viewer receipt"),
        };
    }
    assert_eq!(job.state, regressee_app::protocol::JobState::Succeeded);
    let Some(Outcome::Viewer(frame)) = job.outcome else {
        anyhow::bail!("screenshot viewer failed after unrelated DOM corruption")
    };
    assert_eq!(
        frame.before_capture.unwrap().image.original.as_str(),
        value(&images)["before"]["artifact"]["id"].as_str().unwrap()
    );
    f.call(
        &[
            "results",
            "artifacts",
            "--kind",
            "dom",
            "--project",
            &b.project,
            "--run",
            &checked_run,
            "--page",
            page,
            "--profile",
            profile,
        ],
        6,
    )?;
    Ok(())
}

#[test]
fn v3_capture_modes_independent_changes_and_checks_keep_the_frozen_plan() -> Result<()> {
    for (flag, expected) in [
        ("screenshots", "ScreenshotsOnly"),
        ("both", "ScreenshotsAndDomSnapshots"),
        ("dom", "DomSnapshotsOnly"),
    ] {
        let f = Fixture::start()?;
        let b = baseline(&f, flag)?;
        let unchanged = f.call(&["capture", "check", "--project", &b.project], 0)?;
        assert_eq!(mode(value(&unchanged)), expected);
        if flag == "dom" {
            assert!(value(&unchanged)["comparison"].is_null());
            assert!(
                value(&unchanged)["latest"]["captures"]
                    .as_array()
                    .is_none_or(Vec::is_empty)
            );
            assert_eq!(
                value(&unchanged)["latest"]["run"]["targets"][0]["outcome"],
                "NotRequested"
            );
        }
        if flag != "screenshots" {
            let key = &b.keys[0];
            let source = f.call(
                &[
                    "snapshot",
                    "source",
                    "--project",
                    &b.project,
                    "--run",
                    run(&unchanged),
                    "--page",
                    key["page"].as_str().unwrap(),
                    "--profile",
                    key["profile"].as_str().unwrap(),
                    "--format",
                    "canonical",
                    "--limit",
                    "262144",
                ],
                0,
            )?;
            assert_eq!(
                value(&source)["page"]["text"].as_str().unwrap().as_bytes(),
                include_bytes!("../../../core/tests/fixtures/dom-v1.ndjson")
            );
            assert_eq!(
                value(&unchanged)["latest"]["dom_captures"][0]["metadata"]["run"],
                run(&unchanged)
            );
            assert!(
                value(&unchanged)["latest"]["dom_captures"][0]["metadata"]
                    .get("width")
                    .is_none()
            );
        }
        fs::write(&f.state, "dom-changed")?;
        let checked = f.call(
            &["capture", "check", "--project", &b.project],
            if flag == "screenshots" { 0 } else { 1 },
        )?;
        if flag == "both" {
            assert_eq!(
                value(&checked)["comparison"]["revision"]["results"][0]["classification"],
                "NoSignificantChange"
            );
            assert_eq!(
                value(&checked)["dom_comparison"]["revision"]["results"][0]["outcome"],
                "Changed"
            );
        }
        fs::write(&f.state, "image-changed")?;
        f.call(
            &["capture", "check", "--project", &b.project],
            if flag == "dom" { 0 } else { 1 },
        )?;
        fs::write(&f.state, "dom-failed")?;
        f.call(
            &["capture", "check", "--project", &b.project],
            if flag == "screenshots" { 0 } else { 3 },
        )?;
        fs::write(&f.state, "base")?;
        f.prepare_confirm(&[
            "checks",
            "set",
            "--project",
            &b.project,
            "--expected-revision",
            &b.revision,
            "--mode",
            if flag == "both" { "dom" } else { "both" },
            "--replacement",
        ])?;
        let frozen = f.call(&["capture", "check", "--project", &b.project], 0)?;
        assert_eq!(
            mode(value(&frozen)),
            expected,
            "the baseline plan, not the new project draft, owns checks"
        );
        assert_eq!(value(&frozen)["baseline"]["id"], b.baseline);
    }
    Ok(())
}

#[test]
fn v3_independent_reviews_sequential_sibling_approval_and_atomic_both_approval() -> Result<()> {
    let f = Fixture::start()?;
    let b = baseline(&f, "both")?;
    fs::write(&f.state, "changed")?;
    let checked = f.call(&["capture", "check", "--project", &b.project], 1)?;
    let checked_run = run(&checked).to_owned();
    let dom = value(&checked)["dom_comparison"]["revision"]["id"]
        .as_str()
        .unwrap()
        .to_owned();
    let key = &b.keys[0];
    let page = key["page"].as_str().unwrap();
    let profile = key["profile"].as_str().unwrap();
    let selected = f.file("dom-selected.json", &json!([key]))?;
    f.call(
        &[
            "review",
            "set",
            "--kind",
            "dom",
            "--analysis",
            &dom,
            "--page",
            page,
            "--profile",
            profile,
            "--state",
            "intentional",
            "--note",
            "Independent DOM note",
        ],
        0,
    )?;
    let pixels = f.call(
        &[
            "analysis",
            "reanalyze",
            "--run",
            &checked_run,
            "--preset",
            "sensitive",
        ],
        0,
    )?;
    let image = value(&pixels)["revision"]["id"].as_str().unwrap();
    let notes = f.call(
        &[
            "results",
            "list",
            "--kind",
            "dom",
            "--project",
            &b.project,
            "--run",
            &checked_run,
            "--analysis",
            &dom,
            "--page",
            page,
            "--profile",
            profile,
        ],
        0,
    )?;
    assert_eq!(
        value(&notes)["rows"][0]["reviews"][0]["note"],
        "Independent DOM note"
    );
    let image_preview = f.call(
        &[
            "baseline",
            "preview",
            "--analysis",
            image,
            "--file",
            &selected,
        ],
        0,
    )?;
    let image_file = f.file("dom-image-approval.json", value(&image_preview))?;
    let image_saved = f.prepare_confirm(&["baseline", "accept", "--file", &image_file])?;
    let parent = value(&image_saved)["project"]["active_baseline"]
        .as_str()
        .unwrap();
    let dom_preview = f.call(
        &[
            "baseline",
            "preview",
            "--kind",
            "dom",
            "--analysis",
            &dom,
            "--file",
            &selected,
        ],
        0,
    )?;
    assert!(value(&dom_preview)["screenshot"].is_null());
    assert_eq!(
        value(&dom_preview)["dom"]["context"]["binding"]["current_parent"],
        parent
    );
    assert_eq!(
        value(&dom_preview)["dom"]["context"]["binding"]["checked_baseline"],
        b.baseline
    );
    let dom_file = f.file("dom-dom-approval.json", value(&dom_preview))?;
    f.prepare_confirm(&["baseline", "accept", "--kind", "dom", "--file", &dom_file])?;

    fs::write(&f.state, "base")?;
    let second = f.call(&["capture", "check", "--project", &b.project], 1)?;
    let kinds = f.file("dom-both-selection.json", &json!({
        "screenshot":{"analysis":value(&second)["comparison"]["revision"]["id"],"selected":[key]},
        "dom":{"analysis":value(&second)["dom_comparison"]["revision"]["id"],"selected":[key]}
    }))?;
    let preview = f.call(&["baseline", "preview-kinds", "--file", &kinds], 0)?;
    assert!(value(&preview)["screenshot"].is_object() && value(&preview)["dom"].is_object());
    for (index, pointer, replacement) in [
        (0, "/screenshot/inherited", json!(999999)),
        (1, "/screenshot/provenance/0/1", json!("forged source date")),
        (
            2,
            "/screenshot/source_environments/0/1",
            json!("forged environment"),
        ),
        (3, "/screenshot/baseline/created_at", json!(1)),
        (4, "/dom/inherited", json!(999999)),
        (5, "/dom/baseline/dom_references/0/policy/version", json!(2)),
        (
            6,
            "/baseline/dom_references/0/artifact",
            json!("0".repeat(64)),
        ),
    ] {
        let mut tampered = value(&preview).clone();
        *tampered
            .pointer_mut(pointer)
            .context("expected canonical consent display field")? = replacement;
        let file = f.file(&format!("dom-tampered-{index}.json"), &tampered)?;
        let refused = f.call(
            &["baseline", "accept-kinds", "--file", &file, "--prepare"],
            5,
        )?;
        assert!(refused["data"].get("ticket").is_none());
    }
    let file = f.file("dom-both-approval.json", value(&preview))?;
    let saved = f.prepare_confirm(&["baseline", "accept-kinds", "--file", &file])?;
    let export = f.call(
        &[
            "archive",
            "export-preview",
            "--project",
            &b.project,
            "--baseline",
            value(&saved)["project"]["active_baseline"]
                .as_str()
                .unwrap(),
        ],
        0,
    )?;
    assert!(
        value(&export)["privacy"]
            .as_str()
            .unwrap()
            .contains("DOM snapshots")
    );
    let export_file = f.file("dom-export.json", value(&export))?;
    let archive = f.root.path().join("dom-baseline.zip");
    f.prepare_confirm(&[
        "archive",
        "export",
        "--file",
        &export_file,
        "--destination",
        archive.to_str().unwrap(),
    ])?;
    let imported = f.call(
        &[
            "archive",
            "import-preview",
            "--source",
            archive.to_str().unwrap(),
        ],
        0,
    )?;
    let import_file = f.file("dom-import.json", value(&imported))?;
    let imported = f.prepare_confirm(&[
        "archive",
        "import",
        "--file",
        &import_file,
        "--source",
        archive.to_str().unwrap(),
    ])?;
    assert_eq!(
        value(&imported)["project"]["capture_plan"]["mode"],
        "ScreenshotsAndDomSnapshots"
    );
    Ok(())
}

#[test]
fn v3_unstable_dom_is_not_a_pass_or_an_eligible_baseline_slot() -> Result<()> {
    let f = Fixture::start()?;
    let b = baseline(&f, "both")?;
    fs::write(&f.state, "dom-unstable")?;
    let checked = f.call(&["capture", "check", "--project", &b.project], 3)?;
    assert_eq!(
        value(&checked)["dom_comparison"]["revision"]["results"][0]["outcome"]["NotComparable"]["reason"],
        "Unstable"
    );
    let inventory = f.call(&["discovery", "existing", "--project", &b.project], 0)?;
    let inventory_file = f.file("dom-unstable-inventory.json", value(&inventory))?;
    let candidate = f.call(
        &[
            "capture",
            "baseline",
            "--project",
            &b.project,
            "--file",
            &inventory_file,
        ],
        0,
    )?;
    let prepared = f.call(
        &[
            "baseline",
            "save",
            "--run",
            run(&candidate),
            "--expected-revision",
            &b.revision,
            "--partial",
            "--prepare",
        ],
        0,
    )?;
    let consent = &prepared["data"]["ticket"]["confirmation"]["preview"];
    assert_eq!(consent["per_kind_coverage"]["dom_captures"], 0);
    assert_eq!(consent["captured_coverage"]["dom_captures"], b.keys.len());
    assert_eq!(
        consent["omitted_slots"].as_array().unwrap().len(),
        b.keys.len()
    );
    assert!(
        consent["omitted_slots"]
            .as_array()
            .unwrap()
            .iter()
            .all(|slot| slot["kind"] == "DomSnapshot")
    );
    let ticket = f.file("dom-unstable-ticket.json", &prepared["data"]["ticket"])?;
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
        0,
    )?;
    fs::write(&f.state, "base")?;
    let omitted = f.call(&["capture", "check", "--project", &b.project], 3)?;
    assert!(
        value(&omitted)["latest"]["dom_captures"]
            .as_array()
            .is_none_or(Vec::is_empty)
    );
    assert_eq!(
        value(&omitted)["dom_comparison"]["revision"]["results"][0]["outcome"]["NotComparable"]["reason"],
        "BaselineOmission"
    );
    Ok(())
}
