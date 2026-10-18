// SPDX-License-Identifier: MPL-2.0
//! Public CLI/IPC against owned, retained DOM fixtures; no browser or network.
#![cfg(target_os = "linux")]
use anyhow::{Context, Result};
use regressee_app::{
    engine::{Client, Server},
    protocol::{Request, Response},
};
use regressee_core::{dom_snapshot::*, domain::*, evidence::*};
use regressee_storage::{AppPaths, Store};
use serde_json::{Value, json};
use std::{
    fs,
    process::Command,
    time::{Duration, Instant},
};

struct Fixture {
    root: tempfile::TempDir,
    client: Client,
    thread: Option<std::thread::JoinHandle<Result<()>>>,
    project: ProjectId,
    run: RunId,
    analysis: DomAnalysisId,
    key: TargetKey,
}

const ADVERSARIAL_DOM_TEXT: &str = "</pre></script><script>globalThis.executed=true</script><img src=\"https://attacker.invalid/x\" onerror=\"globalThis.executed=true\"><a href=\"javascript:globalThis.executed=true\">&'\"</a>";
const EXCLUDED_SENTINELS: &[&str] = &[
    "DOM_TEST_PASSWORD_VALUE_CANARY_6f12",
    "DOM_TEST_TEXTAREA_VALUE_CANARY_6f12",
    "DOM_TEST_SCRIPT_BODY_CANARY_6f12",
    "DOM_TEST_INLINE_EVENT_CANARY_6f12",
    "DOM_TEST_IFRAME_SRCDOC_CANARY_6f12",
    "DOM_TEST_OPTION_VALUE_CANARY_6f12",
    "DOM_TEST_CHECKED_CANARY_6f12",
];

#[path = "support/report_evidence.rs"]
mod report_evidence;

fn capture(store: &mut Store, record: &mut RunRecord, text: &str) -> Result<()> {
    let snapshot = DomSnapshot {
        nodes: vec![
            DomNode::Document,
            DomNode::Element {
                depth: 1,
                namespace: HTML_NAMESPACE.into(),
                name: "html".into(),
                prefix: None,
                attributes: vec![DomAttribute {
                    namespace: String::new(),
                    name: "data-fixture".into(),
                    value: text.into(),
                }],
            },
            DomNode::Text {
                depth: 2,
                text: text.into(),
            },
            DomNode::Element {
                depth: 2,
                namespace: HTML_NAMESPACE.into(),
                name: "script".into(),
                prefix: None,
                attributes: vec![],
            },
            DomNode::Element {
                depth: 2,
                namespace: HTML_NAMESPACE.into(),
                name: "input".into(),
                prefix: None,
                attributes: vec![
                    DomAttribute {
                        namespace: String::new(),
                        name: "name".into(),
                        value: "password".into(),
                    },
                    DomAttribute {
                        namespace: String::new(),
                        name: "type".into(),
                        value: "password".into(),
                    },
                ],
            },
            DomNode::Element {
                depth: 2,
                namespace: HTML_NAMESPACE.into(),
                name: "textarea".into(),
                prefix: None,
                attributes: vec![],
            },
            DomNode::Element {
                depth: 2,
                namespace: HTML_NAMESPACE.into(),
                name: "select".into(),
                prefix: None,
                attributes: vec![],
            },
            DomNode::Element {
                depth: 3,
                namespace: HTML_NAMESPACE.into(),
                name: "option".into(),
                prefix: None,
                attributes: vec![],
            },
            DomNode::Text {
                depth: 4,
                text: "Ordinary option label".into(),
            },
            DomNode::Element {
                depth: 2,
                namespace: HTML_NAMESPACE.into(),
                name: "button".into(),
                prefix: None,
                attributes: vec![],
            },
            DomNode::Text {
                depth: 3,
                text: "Continue".into(),
            },
        ],
    };
    // Negative inputs stay in memory: no raw excluded-data snapshot is written.
    // These qualify canonical validation, not browser collection/minimization.
    for (index, (element, attribute)) in [
        ("input", "value"),
        ("html", "onclick"),
        ("option", "value"),
        ("input", "checked"),
    ]
    .into_iter()
    .enumerate()
    {
        let mut invalid = snapshot.clone();
        let DomNode::Element { attributes, .. } = invalid
            .nodes
            .iter_mut()
            .find(|n| matches!(n, DomNode::Element { name, .. } if name == element))
            .unwrap()
        else {
            unreachable!()
        };
        attributes.push(DomAttribute {
            namespace: String::new(),
            name: attribute.into(),
            value: EXCLUDED_SENTINELS[[0, 3, 5, 6][index]].into(),
        });
        attributes.sort_by(|a, b| (&a.namespace, &a.name).cmp(&(&b.namespace, &b.name)));
        assert!(invalid.canonical_bytes().is_err());
    }
    for (element, sentinel) in [
        ("script", EXCLUDED_SENTINELS[2]),
        ("textarea", EXCLUDED_SENTINELS[1]),
    ] {
        let mut invalid = snapshot.clone();
        let index = invalid
            .nodes
            .iter()
            .position(|n| matches!(n, DomNode::Element { name, .. } if name == element))
            .unwrap();
        invalid.nodes.insert(
            index + 1,
            DomNode::Text {
                depth: 3,
                text: sentinel.into(),
            },
        );
        assert!(invalid.canonical_bytes().is_err());
    }
    let mut invalid = snapshot.clone();
    invalid.nodes.push(DomNode::Element {
        depth: 2,
        namespace: HTML_NAMESPACE.into(),
        name: "iframe".into(),
        prefix: None,
        attributes: vec![DomAttribute {
            namespace: String::new(),
            name: "srcdoc".into(),
            value: EXCLUDED_SENTINELS[4].into(),
        }],
    });
    assert!(invalid.canonical_bytes().is_err());
    let bytes = snapshot.canonical_bytes()?;
    let stats = snapshot.validate()?;
    let target = &mut record.run.targets[0];
    target.attempts = 1;
    target.final_url = Some(target.requested.clone());
    let metadata = DomCaptureMetadata {
        version: 1,
        run: record.run.id,
        key: target.key.clone(),
        requested: target.requested.to_string(),
        final_url: target.requested.to_string(),
        runtime: record.run.profiles[0].runtime.clone(),
        profile: record.run.profiles[0].clone(),
        policy: DomSnapshotPolicy::default(),
        sha256: dom_artifact_id(&bytes),
        bytes: bytes.len() as u64,
        node_count: stats.node_count,
        visited_nodes: stats.node_count,
        max_depth: stats.max_depth,
        coverage: stats.coverage,
        warnings: vec![],
        stable: true,
        duration_ms: 1,
        readiness: Readiness {
            font_failures: 0,
            image_failures: 0,
            resource_failures: 0,
        },
    };
    record.dom_captures = vec![store.ingest_dom_snapshot(bytes.as_slice(), &metadata)?];
    target.dom_outcome = DomTargetOutcome::Captured {
        artifact: metadata.sha256,
    };
    record.run.state = JobState::Completed;
    record.run.finished_at = Some(record.run.started_at.unwrap() + 1);
    store.finish_run(record)
}

impl Fixture {
    fn start() -> Result<Self> {
        let root = tempfile::tempdir()?;
        let paths = AppPaths::isolated(root.path());
        let mut store = Store::open(paths.clone())?;
        let mut project = Project::draft(
            "Owned DOM CLI fixture".into(),
            "http://127.0.0.1:43210/owned",
        )?;
        project.capture_plan = CapturePlan::for_mode(CaptureMode::DomSnapshotsOnly);
        project.profiles[0].runtime = RuntimeIdentity::Pinned {
            node: "fixture".into(),
            worker: "fixture".into(),
            playwright: "fixture".into(),
            browser: "fixture".into(),
        };
        let saved = store.save_project(&project, None)?;
        let page = InventoryPage {
            id: PageId::new(),
            canonical: project.sources[0].clone(),
            included: true,
            reason: None,
        };
        let key = TargetKey {
            page: page.id,
            profile: project.profiles[0].id,
        };
        let mut before = RunRecord {
            run: CaptureRun {
                id: RunId::new(),
                project: project.id,
                intent: RunIntent::NewBaseline,
                inventory: InventoryRevision {
                    id: InventoryId::new(),
                    project: project.id,
                    identity_version: 2,
                    pages: vec![page.clone()],
                    coverage: DiscoveryCoverage::Complete,
                },
                profiles: project.profiles.clone(),
                capture_plan: project.capture_plan.clone(),
                environment: project.environments[0].clone(),
                rendering: RenderingEnvironment {
                    os: "fixture".into(),
                    architecture: "fixture".into(),
                    browser: "fixture".into(),
                    worker_version: "fixture".into(),
                    profile_version: 1,
                    fonts_fingerprint: None,
                },
                state: JobState::Running,
                started_at: Some(100),
                finished_at: None,
                targets: vec![CaptureTarget {
                    key: key.clone(),
                    requested: page.canonical,
                    final_url: None,
                    attempts: 0,
                    outcome: TargetOutcome::NotRequested,
                    dom_outcome: DomTargetOutcome::Pending,
                    warnings: vec![],
                }],
            },
            capture: None,
            captures: vec![],
            dom_captures: vec![],
            queue: Some(QueueSettings {
                discovery: project.discovery.clone(),
                project_revision: saved.revision,
                resumed_at: vec![],
            }),
        };
        store.freeze_inventory(&before.run.inventory)?;
        store.begin_run(&before)?;
        capture(&mut store, &mut before, "before")?;
        let saved = store.activate_baseline(before.run.id, saved.revision)?;
        let baseline = saved.project.active_baseline.unwrap();
        let mut after = before.clone();
        after.run.id = RunId::new();
        after.run.intent = RunIntent::Check { baseline };
        after.run.state = JobState::Running;
        after.run.started_at = Some(200);
        after.run.finished_at = None;
        after.run.targets[0].attempts = 0;
        after.run.targets[0].final_url = None;
        after.run.targets[0].dom_outcome = DomTargetOutcome::Pending;
        after.dom_captures.clear();
        after.queue.as_mut().unwrap().project_revision = saved.revision;
        store.begin_run(&after)?;
        capture(&mut store, &mut after, ADVERSARIAL_DOM_TEXT)?;
        let b = dom_source(&before.run, before.dom_evidence(&key).unwrap())?;
        let a = dom_source(&after.run, after.dom_evidence(&key).unwrap())?;
        let options = DomDiffOptions::default();
        let result = compare_dom_snapshots(
            b.clone(),
            &store
                .read_verified_artifact(&b.artifact, MAX_DOM_BYTES as u64)?
                .1,
            a.clone(),
            &store
                .read_verified_artifact(&a.artifact, MAX_DOM_BYTES as u64)?
                .1,
            &options,
            || false,
        )?;
        let analysis = DomAnalysisId::new();
        store.save_dom_analysis(&DomComparisonEvidence {
            revision: DomAnalysisRevision {
                version: 1,
                id: analysis,
                baseline,
                run: after.run.id,
                algorithm: DOM_DIFF_ALGORITHM.into(),
                results: vec![result],
            },
            created_at: 202,
            options,
        })?;
        drop(store);
        let server = Server::without_runtime(paths.clone())?;
        let thread = std::thread::spawn(move || server.run());
        let client = Client::existing(paths)?;
        let deadline = Instant::now() + Duration::from_secs(5);
        while client.snapshot().is_err() {
            anyhow::ensure!(
                Instant::now() < deadline,
                "fixture engine startup timed out"
            );
            std::thread::sleep(Duration::from_millis(10));
        }
        Ok(Self {
            root,
            client,
            thread: Some(thread),
            project: project.id,
            run: after.run.id,
            analysis,
            key,
        })
    }
    fn call(&self, args: &[&str], exit: i32) -> Result<Value> {
        let output = Command::new(env!("CARGO_BIN_EXE_regressee"))
            .args(["--json", "--data-dir"])
            .arg(&self.client.paths.data)
            .arg("--cache-dir")
            .arg(&self.client.paths.cache)
            .args(args)
            .env_remove("DISPLAY")
            .env_remove("WAYLAND_DISPLAY")
            .output()?;
        let value: Value = serde_json::from_slice(&output.stdout).with_context(|| {
            format!(
                "stdout {:?}; stderr {}",
                output.stdout,
                String::from_utf8_lossy(&output.stderr)
            )
        })?;
        assert_eq!(
            output.status.code(),
            Some(exit),
            "{args:?}: {value}; stderr {}",
            String::from_utf8_lossy(&output.stderr)
        );
        Ok(value)
    }
    fn file(&self, value: &Value) -> Result<String> {
        let path = self
            .root
            .path()
            .join(format!("{}.json", uuid::Uuid::new_v4()));
        fs::write(&path, serde_json::to_vec(value)?)?;
        Ok(path.to_str().unwrap().into())
    }
    fn confirm(&self, args: &[&str]) -> Result<Value> {
        let mut args = args.to_vec();
        args.push("--prepare");
        let prepared = self.call(&args, 0)?;
        let ticket = &prepared["data"]["ticket"];
        self.call(
            &[
                "confirm",
                "--file",
                &self.file(ticket)?,
                "--confirmation",
                ticket["confirmation"]["id"].as_str().unwrap(),
            ],
            0,
        )
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Ok(Response::Control(control)) =
            self.client.control(regressee_app::control::Command::List)
            && let Some(id) = control.controller
            && let Ok(bytes) = fs::read(
                self.client
                    .paths
                    .data
                    .join("agent-sessions")
                    .join(format!("{id}.json")),
            )
            && let Ok(credential) =
                serde_json::from_slice::<regressee_app::control::Credential>(&bytes)
        {
            let _ = self
                .client
                .with_credential(credential)
                .control(regressee_app::control::Command::End);
        }
        let deadline = Instant::now() + Duration::from_secs(10);
        loop {
            if matches!(self.client.request(Request::Stop), Ok(Response::Ok)) {
                break;
            }
            if let Ok(snapshot) = self.client.snapshot() {
                for job in snapshot.jobs {
                    if !job.state.terminal() {
                        let _ = self.client.request(Request::Cancel { id: job.id });
                    }
                }
            }
            assert!(
                Instant::now() < deadline,
                "owned DOM fixture engine did not stop"
            );
            std::thread::sleep(Duration::from_millis(20));
        }
        if let Some(thread) = self.thread.take() {
            let _ = thread.join();
        }
    }
}

#[test]
fn retained_dom_only_public_reads_review_approval_and_inert_report() -> Result<()> {
    let f = Fixture::start()?;
    let p = f.project.to_string();
    let run = f.run.to_string();
    let analysis = f.analysis.to_string();
    let page = f.key.page.to_string();
    let profile = f.key.profile.to_string();
    f.call(
        &[
            "snapshot",
            "source",
            "--project",
            &p,
            "--run",
            &run,
            "--page",
            &page,
            "--profile",
            &profile,
            "--limit",
            "1",
        ],
        2,
    )?;
    f.call(&["crop", "preview", "--kind", "dom"], 2)?;
    let unsafe_source = f.root.path().join("source.html");
    f.call(
        &[
            "snapshot",
            "source",
            "--project",
            &p,
            "--run",
            &run,
            "--page",
            &page,
            "--profile",
            &profile,
            "--save",
            unsafe_source.to_str().unwrap(),
        ],
        2,
    )?;
    assert!(!unsafe_source.exists());
    let results = f.call(
        &[
            "results",
            "list",
            "--kind",
            "dom",
            "--project",
            &p,
            "--run",
            &run,
        ],
        0,
    )?;
    assert_eq!(
        results["data"]["value"]["rows"][0]["result"]["outcome"],
        "Changed"
    );
    assert!(results["data"]["value"]["rows"][0]["result"]["diff"].is_null());
    assert!(results["data"]["value"]["rows"][0]["diff_summary"]["added_lines"].is_number());
    f.call(
        &[
            "results",
            "artifacts",
            "--project",
            &p,
            "--run",
            &run,
            "--page",
            &page,
            "--profile",
            &profile,
        ],
        2,
    )?;
    let source = f.call(
        &[
            "snapshot",
            "source",
            "--project",
            &p,
            "--run",
            &run,
            "--analysis",
            &analysis,
            "--page",
            &page,
            "--profile",
            &profile,
            "--limit",
            "32",
        ],
        0,
    )?;
    assert!(
        source["data"]["value"]["page"]["text"]
            .as_str()
            .unwrap()
            .len()
            <= 32
    );
    assert!(source["data"]["value"]["page"]["next_offset"].is_number());
    let diff = f.call(
        &[
            "snapshot",
            "diff",
            "--project",
            &p,
            "--run",
            &run,
            "--page",
            &page,
            "--profile",
            &profile,
            "--limit",
            "1",
        ],
        0,
    )?;
    assert_eq!(diff["data"]["value"]["outcome"], "Changed");
    assert!(diff["data"]["value"]["page"]["summary"]["added_lines"].is_number());
    let started = f.call(&["agent", "start", "--name", "Owned DOM inspection"], 0)?;
    let session = started["data"]["session"].as_str().unwrap();
    let selected = f.call(
        &[
            "--session",
            session,
            "agent",
            "select",
            "--kind",
            "dom",
            "--project",
            &p,
            "--run",
            &run,
            "--analysis",
            &analysis,
            "--page",
            &page,
            "--profile",
            &profile,
            "--view",
            "before",
        ],
        0,
    )?;
    assert_eq!(
        selected["data"]["value"]["selection"]["kind"],
        "DomSnapshot"
    );
    f.call(
        &[
            "--session",
            session,
            "agent",
            "select",
            "--kind",
            "dom",
            "--project",
            &p,
            "--run",
            &run,
            "--view",
            "overlay",
        ],
        2,
    )?;
    f.call(&["--session", session, "agent", "end"], 0)?;
    f.call(
        &[
            "review",
            "set",
            "--kind",
            "dom",
            "--analysis",
            &analysis,
            "--page",
            &page,
            "--profile",
            &profile,
            "--state",
            "intentional",
            "--note",
            "</pre></script><img src=x onerror=evil()>",
        ],
        0,
    )?;
    let reviews = f.call(
        &[
            "review",
            "history",
            "--kind",
            "dom",
            "--analysis",
            &analysis,
            "--limit",
            "1",
        ],
        0,
    )?;
    assert_eq!(reviews["data"]["value"]["total"], 1);
    let selected = f.file(&json!([f.key]))?;
    let destination = f.root.path().join("report");
    let preview = f.call(
        &[
            "report",
            "preview",
            "--project",
            &p,
            "--run",
            &run,
            "--file",
            &selected,
            "--destination",
            destination.to_str().unwrap(),
        ],
        0,
    )?;
    assert!(
        preview["data"]["value"]["content"]["artifacts"]
            .as_array()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        preview["data"]["value"]["content"]["dom_artifacts"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
    let exported = f.confirm(&[
        "report",
        "export",
        "--file",
        &f.file(&preview["data"]["value"])?,
    ])?;
    let html = fs::read_to_string(destination.join("index.html"))?;
    for active in ["<script", "<img", "src=\"http", "href=\"javascript:"] {
        assert!(!html.contains(active));
    }
    assert!(html.contains("&lt;/pre&gt;&lt;/script&gt;"));
    assert!(!html.contains("original screenshot: unavailable"));
    assert!(destination.join("0-dom-after.canonical.txt").is_file());
    assert!(destination.join("0-dom-after.txt").is_file());
    for entry in fs::read_dir(&destination)? {
        let bytes = fs::read(entry?.path())?;
        for sentinel in EXCLUDED_SENTINELS {
            assert!(
                !bytes
                    .windows(sentinel.len())
                    .any(|window| window == sentinel.as_bytes())
            );
        }
    }
    report_evidence::retain(&f, &destination, &preview["data"]["value"], &exported)?;
    let approval = f.call(
        &[
            "baseline",
            "preview",
            "--kind",
            "dom",
            "--analysis",
            &analysis,
            "--file",
            &selected,
        ],
        0,
    )?;
    assert!(approval["data"]["value"]["screenshot"].is_null());
    for (index, pointer, replacement) in [
        (0, "/dom/inherited", json!(999999)),
        (1, "/dom/baseline/id", json!(uuid::Uuid::new_v4())),
        (2, "/dom/baseline/created_at", json!(1)),
        (3, "/dom/baseline/intended_count", json!(999)),
        (4, "/dom/baseline/dom_references/0/captured_at", json!(1)),
        (5, "/dom/context/display_revision", json!(999999)),
        (
            6,
            "/dom/context/binding/selections/0/before/policy/version",
            json!(2),
        ),
    ] {
        let mut tampered = approval["data"]["value"].clone();
        *tampered.pointer_mut(pointer).unwrap() = replacement;
        let file = f.file(&tampered)?;
        let refused = f.call(
            &[
                "baseline",
                "accept",
                "--kind",
                "dom",
                "--file",
                &file,
                "--prepare",
            ],
            5,
        )?;
        assert!(
            refused["data"].get("ticket").is_none(),
            "tamper {index} reached consent"
        );
    }
    let mut unknown = approval["data"]["value"].clone();
    unknown["approve_unselected_sibling"] = true.into();
    assert!(
        serde_json::from_value::<regressee_app::engine::dom::Command>(
            json!({"action":"Accept","arguments":unknown.clone()})
        )
        .is_err()
    );
    f.call(
        &[
            "baseline",
            "accept",
            "--kind",
            "dom",
            "--file",
            &f.file(&unknown)?,
        ],
        2,
    )?;
    let before_identity =
        preview["data"]["value"]["content"]["dom_artifacts"][0]["before"]["source"].clone();
    f.confirm(&[
        "baseline",
        "accept",
        "--kind",
        "dom",
        "--file",
        &f.file(&approval["data"]["value"])?,
    ])?;
    let retained = f.call(
        &[
            "results",
            "artifacts",
            "--kind",
            "dom",
            "--project",
            &p,
            "--run",
            &run,
            "--analysis",
            &analysis,
            "--page",
            &page,
            "--profile",
            &profile,
        ],
        0,
    )?;
    assert_eq!(
        retained["data"]["value"]["before"]["source"], before_identity,
        "approval must not rewrite the check's original reference"
    );
    Ok(())
}
