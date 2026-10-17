// SPDX-License-Identifier: MPL-2.0
//! Phase 5 independent consumer findings, through the real public CLI/engine.
//! Images here come from the signed synthetic worker, not live Chromium.
use super::*;

fn baseline(f: &Fixture) -> Result<String> {
    let (project, _) = f.create()?;
    let discovered = f.call(&["discovery", "start", "--project", &project], 0)?;
    let inventory = f.file("phase5-inventory.json", value(&discovered))?;
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
    f.prepare_confirm(&[
        "baseline",
        "save",
        "--run",
        run(&captured),
        "--expected-revision",
        "1",
    ])?;
    Ok(project)
}

#[test]
fn incomplete_save_show_wait_and_retry_preserve_exact_evidence_and_request() -> Result<()> {
    let f = Fixture::start()?;
    let project = baseline(&f)?;
    fs::write(&f.state, "failed")?;
    let request = uuid::Uuid::new_v4().to_string();
    let path = f.root.path().join("incomplete.json");
    let out = f.call(
        &[
            "capture",
            "check",
            "--project",
            &project,
            "--request-id",
            &request,
            "--save",
            path.to_str().unwrap(),
        ],
        3,
    )?;
    assert_eq!(out["code"], "incomplete_capture");
    let saved: Value = serde_json::from_slice(&fs::read(&path)?)?;
    assert_eq!(saved, *value(&out));
    assert_eq!(saved["latest"]["run"]["state"], "CompletedWithErrors");
    assert_eq!(out["data"]["request_id"], request);
    assert!(Path::new(out["data"]["retry_file"].as_str().unwrap()).is_file());
    let run_id = run(&out).to_string();
    for action in ["show", "wait", "retry"] {
        let path = f.root.path().join(format!("incomplete-{action}.json"));
        let id_flag = if action == "retry" {
            "--request-id"
        } else {
            "--job"
        };
        let recovered = f.call(
            &[
                "jobs",
                action,
                id_flag,
                &request,
                "--save",
                path.to_str().unwrap(),
            ],
            3,
        )?;
        assert_eq!(recovered["data"]["job"]["id"], request);
        assert_eq!(run(&recovered), run_id);
        assert_eq!(serde_json::from_slice::<Value>(&fs::read(&path)?)?, saved);
    }
    // An output collision must preserve the incomplete outcome/receipt, not
    // overwrite the file, silently claim success or submit capture again.
    let failed_save = f.call(
        &[
            "jobs",
            "retry",
            "--request-id",
            &request,
            "--save",
            path.to_str().unwrap(),
        ],
        2,
    )?;
    assert_eq!(failed_save["code"], "invalid_request");
    assert!(
        failed_save["data"]["message"]
            .as_str()
            .unwrap()
            .contains("no command was submitted")
    );
    let retained = f.call(&["jobs", "show", "--job", &request], 3)?;
    assert_eq!(retained["data"]["job"]["id"], request);
    assert_eq!(serde_json::from_slice::<Value>(&fs::read(&path)?)?, saved);
    Ok(())
}

#[test]
fn parser_product_conflict_and_action_required_failures_save_full_diagnostics() -> Result<()> {
    let f = Fixture::start()?;
    let (project, settings) = f.create()?;
    let settings = f.file("phase5-settings.json", &settings["project"])?;
    let missing = uuid::Uuid::new_v4().to_string();
    for (name, args, exit) in [
        (
            "parser",
            vec!["project", "show", "--project", "not-a-uuid"],
            2,
        ),
        ("missing", vec!["project", "show", "--project", &missing], 6),
        (
            "conflict",
            vec![
                "project",
                "update",
                "--project",
                &project,
                "--expected-revision",
                "99",
                "--file",
                &settings,
            ],
            5,
        ),
    ] {
        let path = f.root.path().join(format!("phase5-{name}.json"));
        let mut args = args;
        args.extend(["--save", path.to_str().unwrap()]);
        let out = f.call(&args, exit)?;
        assert_eq!(serde_json::from_slice::<Value>(&fs::read(&path)?)?, out);
        assert_eq!(out["ok"], false);
    }
    let start = f.call(
        &[
            "agent",
            "start",
            "--name",
            "phase5-save-failure",
            "--lease",
            "300",
        ],
        0,
    )?;
    let session = start["data"]["session"].as_str().unwrap();
    let path = f.root.path().join("phase5-action-required.json");
    let handoff = f.call(
        &[
            "--session",
            session,
            "agent",
            "handoff",
            "--message",
            "owned headless handoff",
            "--save",
            path.to_str().unwrap(),
        ],
        4,
    )?;
    assert_eq!(serde_json::from_slice::<Value>(&fs::read(path)?)?, handoff);
    assert_eq!(handoff["data"]["human_input"]["agent_paused"], true);
    f.call(&["--session", session, "agent", "end"], 0)?;
    Ok(())
}

#[test]
fn html_and_markdown_keep_visible_privacy_and_original_versus_derived_labels() -> Result<()> {
    let f = Fixture::start()?;
    let project = baseline(&f)?;
    fs::write(&f.state, "changed")?;
    let captured = f.call(&["capture", "check", "--project", &project], 1)?;
    let run_id = run(&captured);
    let analysis = value(&captured)["comparison"]["revision"]["id"]
        .as_str()
        .unwrap();
    let key = value(&captured)["latest"]["run"]["targets"][0]["key"].clone();
    let page = key["page"].as_str().unwrap();
    let profile = key["profile"].as_str().unwrap();
    let keys = f.file("phase5-report-keys.json", &json!([key]))?;
    f.call(
        &[
            "review",
            "set",
            "--analysis",
            analysis,
            "--page",
            page,
            "--profile",
            profile,
            "--state",
            "intentional",
            "--note",
            "</pre><script>untrusted()</script> ![](https://attacker.invalid/image)",
        ],
        0,
    )?;
    let destination = f.root.path().join("phase5-report");
    let plan = f.call(
        &[
            "report",
            "preview",
            "--project",
            &project,
            "--run",
            run_id,
            "--analysis",
            analysis,
            "--file",
            &keys,
            "--destination",
            destination.to_str().unwrap(),
        ],
        0,
    )?;
    let plan = f.file("phase5-report-preview.json", value(&plan))?;
    f.prepare_confirm(&["report", "export", "--file", &plan])?;
    let html = fs::read_to_string(destination.join("index.html"))?;
    let md = fs::read_to_string(destination.join("summary.md"))?;
    assert!(html.contains("<body>"));
    assert!(html.contains("</body></html>"));
    let warning = html
        .find("<h2 id=\"sharing-warning\">Before sharing</h2>")
        .unwrap();
    assert!(warning < html.find("<h2>Target").unwrap());
    for text in [&html, &md] {
        assert!(text.contains("LOCAL ONLY"));
        assert!(text.contains("query strings"));
        assert!(text.contains("Ignored areas are not redaction"));
        assert!(text.contains("Before — original screenshot"));
        assert!(text.contains("After — original screenshot"));
        assert!(text.contains("Difference — derived analysis"));
        assert!(text.contains("Crops are derived evidence"));
        assert!(!text.contains("<script>"));
        assert!(text.contains("&lt;script&gt;untrusted()"));
    }
    assert!(html.contains("href=\"0-before.png\">Original before screenshot PNG</a>"));
    assert!(html.contains("href=\"0-after.png\">Original after screenshot PNG</a>"));
    assert!(html.contains("href=\"0-difference.png\">Derived difference PNG</a>"));
    assert!(!html.contains("src=\"http"));
    assert!(html.contains("Content-Security-Policy"));
    let (_, details) = md.split_once("<pre>").unwrap();
    let (details, tail) = details.split_once("</pre>").unwrap();
    assert!(details.contains("&lt;/pre&gt;&lt;script&gt;untrusted()"));
    assert!(details.contains("![](https://attacker.invalid/image)"));
    assert!(
        !tail.contains("attacker.invalid"),
        "untrusted Markdown stays inert inside escaped preformatted details"
    );
    for (pane, label) in [
        ("before", "original before screenshot"),
        ("after", "original after screenshot"),
        ("difference", "derived difference analysis"),
    ] {
        let destination = f.root.path().join(format!("phase5-{pane}-crop.png"));
        let plan = f.call(
            &[
                "crop",
                "preview",
                "--project",
                &project,
                "--run",
                run_id,
                "--analysis",
                analysis,
                "--page",
                page,
                "--profile",
                profile,
                "--pane",
                pane,
                "--rect",
                "0,0,8,8",
                "--space",
                "screenshot-pixels",
                "--destination",
                destination.to_str().unwrap(),
            ],
            0,
        )?;
        let plan = f.file(&format!("phase5-{pane}-crop.json"), value(&plan))?;
        let out = f.prepare_confirm(&["crop", "export", "--file", &plan])?;
        assert_eq!(value(&out)["derived"], true);
        let reader = png::Decoder::new(fs::File::open(destination)?).read_info()?;
        assert!(
            reader
                .info()
                .uncompressed_latin1_text
                .iter()
                .any(|text| text.keyword == "Regressee derivative"
                    && text.text.contains(&format!("derived crop of {label}")))
        );
    }
    Ok(())
}
