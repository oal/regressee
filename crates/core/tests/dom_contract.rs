// SPDX-License-Identifier: MPL-2.0
use regressee_core::archive::*;
use regressee_core::dom_snapshot::*;
use regressee_core::domain::*;
use regressee_core::evidence::*;
use std::cell::Cell;
use url::Url;

const GOLDEN: &[u8] = include_bytes!("fixtures/dom-v1.ndjson");

fn hash(c: char) -> ArtifactId {
    c.to_string().repeat(64).try_into().unwrap()
}
fn element(depth: u16, name: &str, attributes: Vec<DomAttribute>) -> DomNode {
    DomNode::Element {
        depth,
        namespace: HTML_NAMESPACE.into(),
        name: name.into(),
        attributes,
        prefix: None,
    }
}
fn attr(name: &str, value: &str) -> DomAttribute {
    DomAttribute {
        namespace: String::new(),
        name: name.into(),
        value: value.into(),
    }
}
fn namespaced_attr(name: &str, value: &str) -> DomAttribute {
    DomAttribute {
        namespace: "urn:fixture".into(),
        name: name.into(),
        value: value.into(),
    }
}
fn wire(nodes: &[DomNode]) -> Vec<u8> {
    let mut bytes = DOM_CANONICAL_HEADER.as_bytes().to_vec();
    for node in nodes {
        bytes.extend(serde_json::to_vec(node).unwrap());
        bytes.push(b'\n');
    }
    bytes
}
fn run(mode: CaptureMode, count: usize) -> CaptureRun {
    let project = Project::draft("Owned fixture".into(), "https://example.test/").unwrap();
    let mut profile = project.profiles[0].clone();
    profile.runtime = RuntimeIdentity::Pinned {
        node: "fixture-node".into(),
        worker: "fixture-worker-dom-v1".into(),
        playwright: "fixture-playwright".into(),
        browser: "fixture-browser".into(),
    };
    let pages: Vec<_> = (0..count)
        .map(|i| InventoryPage {
            id: PageId::new(),
            canonical: Url::parse(&format!("https://example.test/p{i}")).unwrap(),
            included: true,
            reason: None,
        })
        .collect();
    let targets = pages
        .iter()
        .map(|p| CaptureTarget {
            key: TargetKey {
                page: p.id,
                profile: profile.id,
            },
            requested: p.canonical.clone(),
            final_url: None,
            attempts: 0,
            outcome: if mode.screenshots() {
                TargetOutcome::Pending
            } else {
                TargetOutcome::NotRequested
            },
            dom_outcome: if mode.dom_snapshots() {
                DomTargetOutcome::Pending
            } else {
                DomTargetOutcome::NotRequested
            },
            warnings: vec![],
        })
        .collect();
    CaptureRun {
        id: RunId::new(),
        project: project.id,
        intent: RunIntent::NewBaseline,
        inventory: InventoryRevision {
            id: InventoryId::new(),
            project: project.id,
            identity_version: 2,
            pages,
            coverage: DiscoveryCoverage::Complete,
        },
        profiles: vec![profile],
        capture_plan: CapturePlan::for_mode(mode),
        environment: project.environments[0].clone(),
        rendering: RenderingEnvironment {
            os: "fixture".into(),
            architecture: "fixture".into(),
            browser: "fixture-browser".into(),
            worker_version: "fixture-worker-dom-v1".into(),
            profile_version: 1,
            fonts_fingerprint: None,
        },
        state: JobState::Queued,
        started_at: Some(1),
        finished_at: None,
        targets,
    }
}
fn readiness() -> Readiness {
    Readiness {
        font_failures: 0,
        image_failures: 0,
        resource_failures: 0,
    }
}
fn dom_capture(run: &mut CaptureRun, index: usize, stable: bool) -> DomCaptureEvidence {
    let stats = DomSnapshot::from_canonical(GOLDEN)
        .unwrap()
        .validate()
        .unwrap();
    let target = &mut run.targets[index];
    target.final_url = Some(target.requested.clone());
    target.attempts = 1;
    target.dom_outcome = DomTargetOutcome::Captured {
        artifact: dom_artifact_id(GOLDEN),
    };
    let profile = run.profiles[0].clone();
    DomCaptureEvidence {
        metadata: DomCaptureMetadata {
            version: 1,
            run: run.id,
            key: target.key.clone(),
            requested: target.requested.to_string(),
            final_url: target.requested.to_string(),
            runtime: profile.runtime.clone(),
            profile,
            policy: DomSnapshotPolicy::default(),
            sha256: dom_artifact_id(GOLDEN),
            bytes: GOLDEN.len() as u64,
            node_count: stats.node_count,
            visited_nodes: stats.node_count + 5,
            max_depth: stats.max_depth,
            coverage: stats.coverage,
            warnings: vec![
                "IframeDocumentsExcluded".into(),
                "ClosedShadowRootsExcluded".into(),
            ],
            stable,
            duration_ms: 100,
            readiness: readiness(),
        },
    }
}
fn screenshot(run: &mut CaptureRun, index: usize) -> CaptureEvidence {
    let target = &mut run.targets[index];
    target.final_url = Some(target.requested.clone());
    target.attempts = 1;
    target.outcome = TargetOutcome::Captured {
        artifact: hash('a'),
        width: 10,
        height: 10,
    };
    let profile = run.profiles[0].clone();
    CaptureEvidence {
        metadata: CaptureMetadata {
            version: 1,
            key: target.key.clone(),
            requested: target.requested.to_string(),
            final_url: target.requested.to_string(),
            runtime: profile.runtime.clone(),
            profile,
            width: 10,
            height: 10,
            sha256: hash('a'),
            bytes: 100,
            warnings: vec![],
            stable: true,
            duration_ms: 100,
            readiness: readiness(),
            capture_mode: "normal".into(),
            replacement: None,
        },
        image: ImageEvidence {
            original: hash('a'),
            thumbnail: hash('b'),
            width: 10,
            height: 10,
            tiles: vec![],
        },
    }
}
fn baseline(run: &CaptureRun) -> BaselineVersion {
    BaselineVersion {
        id: BaselineId::new(),
        project: run.project,
        parent: None,
        inventory: run.inventory.id,
        created_at: 3,
        references: run
            .targets
            .iter()
            .filter_map(|t| match &t.outcome {
                TargetOutcome::Captured { artifact, .. } => Some(BaselineReference {
                    key: t.key.clone(),
                    run: run.id,
                    artifact: artifact.clone(),
                    captured_at: 2,
                }),
                _ => None,
            })
            .collect(),
        dom_references: run
            .targets
            .iter()
            .filter_map(|t| match &t.dom_outcome {
                DomTargetOutcome::Captured { artifact } => Some(DomBaselineReference {
                    key: t.key.clone(),
                    run: run.id,
                    artifact: artifact.clone(),
                    captured_at: 2,
                    policy: DomSnapshotPolicy::default(),
                }),
                _ => None,
            })
            .collect(),
        capture_plan: run.capture_plan.clone(),
        source_run: Some(run.id),
        intended_targets: run.targets.iter().map(|t| t.key.clone()).collect(),
        intended_count: run.targets.len() as u32,
        explicitly_partial: false,
    }
}
fn source(key: &TargetKey, bytes: &[u8]) -> DomSourceIdentity {
    DomSourceIdentity {
        run: RunId::new(),
        key: key.clone(),
        artifact: dom_artifact_id(bytes),
        policy: DomSnapshotPolicy::default(),
        material_profile: hash('c'),
        rendering_environment: hash('d'),
        stable: true,
    }
}

#[test]
fn t02_t03_cross_language_golden_and_canonical_source() {
    let snapshot = DomSnapshot::from_canonical(GOLDEN).unwrap();
    assert_eq!(snapshot.canonical_bytes().unwrap(), GOLDEN);
    assert_eq!(
        snapshot.format_source().unwrap(),
        include_str!("fixtures/dom-v1-source.txt")
    );
    assert_eq!(
        dom_artifact_id(GOLDEN).as_str(),
        "d9876d0964cc64dc5279bd14f3fa6493134ac0dc7b7580ff9ba12484b58d807e"
    );
    let stats = snapshot.validate().unwrap();
    assert_eq!(stats.node_count, 28);
    assert_eq!(stats.max_depth, 10);
    assert_eq!(
        stats.coverage,
        DomCoverage {
            iframe_hosts: 1,
            open_shadow_roots: 1,
            template_contents: 2
        }
    );
    assert!(DOM_COVERAGE_NOTICE.contains("cannot reliably be counted"));
    for text in ["IDs", "classes", "  whitespace\r\n ", "\u{2028}\u{2029}"] {
        let nodes = vec![
            DomNode::Document,
            element(1, "div", vec![]),
            DomNode::Text {
                depth: 2,
                text: text.into(),
            },
        ];
        assert_eq!(
            DomSnapshot::from_canonical(&wire(&nodes)).unwrap().nodes,
            nodes
        );
    }
}

#[test]
fn t07_noncanonical_malformed_or_future_wire_is_rejected_without_echo() {
    let valid = String::from_utf8(GOLDEN.to_vec()).unwrap();
    for bad in [
        valid.replace("café", "caf\\u00e9"),
        valid.replace("[\"document\",0]", "[\"document\", 0]"),
        valid.replace("regressee-dom\",1", "regressee-dom\",2"),
        valid.trim_end().to_owned(),
        format!("\u{feff}{valid}"),
        valid.replace("[\"document\",0]", "[\"comment\",0,\"DO_NOT_ECHO\"]"),
        valid.replace("café", "\\ud800"),
        valid.replace("[\"document\",0]", "[\"document\",0,\"DO_NOT_ECHO\"]"),
    ] {
        let error = DomSnapshot::from_canonical(bad.as_bytes())
            .unwrap_err()
            .to_string();
        assert!(!error.contains("DO_NOT_ECHO"));
    }
    assert!(DomSnapshot::from_canonical(b"\xff").is_err());
    assert!(DomSnapshot::from_canonical(DOM_CANONICAL_HEADER.as_bytes()).is_err());
    let mut reversed = DomSnapshot::from_canonical(GOLDEN).unwrap();
    if let DomNode::Element { attributes, .. } = &mut reversed.nodes[4] {
        attributes.reverse();
    }
    assert!(DomSnapshot::from_canonical(&wire(&reversed.nodes)).is_err());
}

#[test]
fn t04_semantic_ingestion_enforces_exclusion_matrix_even_with_correct_hash() {
    let secret = "DO_NOT_RETAIN_OR_ECHO";
    for tag in ["input", "select", "option", "button", "textarea"] {
        for name in ["value", "x:VaLuE"] {
            let attribute = if name.contains(':') {
                namespaced_attr(name, secret)
            } else {
                attr(name, secret)
            };
            let bytes = wire(&[DomNode::Document, element(1, tag, vec![attribute])]);
            assert_eq!(dom_artifact_id(&bytes).as_str().len(), 64);
            let error = DomSnapshot::from_canonical(&bytes).unwrap_err().to_string();
            assert!(!error.contains(secret));
        }
    }
    for (tag, name, value) in [
        ("div", "onClick", secret),
        ("svg", "x:ONLOAD", secret),
        ("style", "NONCE", secret),
        ("input", "checked", ""),
        ("option", "selected", ""),
        ("iframe", "srcdoc", secret),
        ("a", "href", " \rJaVa\nScRiPt:alert(1)"),
        ("svg", "xlink:href", "javascript:alert(1)"),
    ] {
        let attribute = if name.contains(':') {
            namespaced_attr(name, value)
        } else {
            attr(name, value)
        };
        assert!(
            DomSnapshot::from_canonical(&wire(&[
                DomNode::Document,
                element(1, tag, vec![attribute])
            ]))
            .is_err(),
            "{tag}/{name}"
        );
    }
    for tag in ["script", "SCRIPT", "textarea", "iframe"] {
        assert!(
            DomSnapshot::from_canonical(&wire(&[
                DomNode::Document,
                element(1, tag, vec![]),
                DomNode::Text {
                    depth: 2,
                    text: secret.into()
                }
            ]))
            .is_err()
        );
    }
    let bytes = wire(&[
        DomNode::Document,
        element(1, "div", vec![attr("data-private", secret)]),
        DomNode::Text {
            depth: 2,
            text: "</script><button value=secret> & ordinary private text".into(),
        },
    ]);
    assert!(
        DomSnapshot::from_canonical(&bytes).is_ok(),
        "ordinary private content remains warned about, not guessed away"
    );
    assert!(DOM_PRIVACY_NOTICE.contains("not complete redaction"));
}

#[test]
fn t03_invalid_boundary_and_preorder_structures_fail() {
    let invalid = [
        vec![DomNode::Document, element(2, "div", vec![])],
        vec![
            DomNode::Document,
            element(1, "div", vec![]),
            element(1, "span", vec![]),
        ],
        vec![DomNode::Document, element(1, "template", vec![])],
        vec![
            DomNode::Document,
            element(1, "template", vec![]),
            element(2, "span", vec![]),
        ],
        vec![DomNode::Document, DomNode::OpenShadow { depth: 1 }],
        vec![
            DomNode::Document,
            element(1, "div", vec![]),
            DomNode::TemplateContent { depth: 2 },
        ],
        vec![
            DomNode::Document,
            element(1, "div", vec![]),
            DomNode::OpenShadow { depth: 2 },
            element(2, "span", vec![]),
        ],
        vec![
            DomNode::Document,
            element(1, "div", vec![]),
            DomNode::Text {
                depth: 2,
                text: "leaf".into(),
            },
            element(3, "span", vec![]),
        ],
        vec![
            DomNode::Document,
            element(1, "div", vec![]),
            DomNode::Doctype {
                depth: 1,
                name: "html".into(),
                public_id: String::new(),
                system_id: String::new(),
            },
        ],
    ];
    for nodes in invalid {
        assert!(DomSnapshot::from_canonical(&wire(&nodes)).is_err());
    }
}

#[test]
fn t07_hard_bounds_cover_depth_nodes_strings_attributes_and_final_bytes() {
    let mut deep = vec![DomNode::Document];
    deep.extend((1..=MAX_DOM_DEPTH).map(|depth| element(depth, "div", vec![])));
    assert!(DomSnapshot::from_canonical(&wire(&deep)).is_ok());
    deep.push(element(MAX_DOM_DEPTH + 1, "div", vec![]));
    assert!(DomSnapshot::from_canonical(&wire(&deep)).is_err());
    let mut many = vec![DomNode::Document, element(1, "div", vec![])];
    many.extend((2..MAX_DOM_NODES).map(|_| DomNode::Text {
        depth: 2,
        text: String::new(),
    }));
    assert!(DomSnapshot::from_canonical(&wire(&many)).is_ok());
    many.push(DomNode::Text {
        depth: 2,
        text: String::new(),
    });
    assert!(DomSnapshot::from_canonical(&wire(&many)).is_err());
    let excessive = (0..=MAX_DOM_ATTRIBUTES)
        .map(|i| attr(&format!("a{i:04}"), ""))
        .collect();
    assert!(
        DomSnapshot::from_canonical(&wire(&[DomNode::Document, element(1, "div", excessive)]))
            .is_err()
    );
    let large_attrs = (0..5)
        .map(|i| attr(&format!("a{i}"), &"x".repeat(MAX_DOM_STRING_BYTES)))
        .collect();
    assert!(
        DomSnapshot::from_canonical(&wire(&[DomNode::Document, element(1, "div", large_attrs)]))
            .is_err()
    );
    for text in [
        "x".repeat(MAX_DOM_STRING_BYTES + 1),
        "🦀".repeat(MAX_DOM_STRING_BYTES / 4 + 1),
    ] {
        assert!(
            DomSnapshot::from_canonical(&wire(&[
                DomNode::Document,
                element(1, "div", vec![]),
                DomNode::Text { depth: 2, text }
            ]))
            .is_err()
        );
    }
    let mut huge = vec![DomNode::Document, element(1, "div", vec![])];
    huge.extend((0..130).map(|_| DomNode::Text {
        depth: 2,
        text: "x".repeat(MAX_DOM_STRING_BYTES),
    }));
    assert!(DomSnapshot { nodes: huge }.canonical_bytes().is_err());
    assert!(DomSnapshot::from_canonical(&vec![b' '; MAX_DOM_BYTES + 1]).is_err());
}

#[test]
fn t08_line_diff_cases_are_deterministic_and_numbered() {
    for (old, new, added, removed) in [
        ("", "", 0, 0),
        ("", "a\n", 1, 0),
        ("a\n", "", 0, 1),
        ("a\nb\nc\n", "a\nx\nc\n", 1, 1),
        ("a\nb\na\n", "a\na\nb\n", 1, 1),
        ("café\n🦀\n", "café\n日本語\n", 1, 1),
        ("a\n", "a", 1, 1),
        ("a\r\n", "a\n", 1, 1),
    ] {
        let options = DomDiffOptions::default();
        let diff = diff_text(old, new, &options, || false).unwrap();
        diff.validate().unwrap();
        assert_eq!((diff.added_lines, diff.removed_lines), (added, removed));
        assert_eq!(diff.changed, old != new);
        assert_eq!(diff, diff_text(old, new, &options, || false).unwrap());
        assert_eq!(diff.fallback, None);
    }
    let zero_context = DomDiffOptions {
        context_lines: 0,
        ..Default::default()
    };
    let insert = diff_text("", "new\n", &zero_context, || false).unwrap();
    assert_eq!(
        (
            insert.hunks[0].old_start,
            insert.hunks[0].old_lines,
            insert.hunks[0].new_start
        ),
        (0, 0, 1)
    );
    let deleted = diff_text("old\n", "", &zero_context, || false).unwrap();
    assert_eq!(
        (
            deleted.hunks[0].old_start,
            deleted.hunks[0].new_start,
            deleted.hunks[0].new_lines
        ),
        (1, 0, 0)
    );
    let split = diff_text(
        "a\nb\nc\nd\ne\nf\ng\n",
        "x\nb\nc\nd\ne\nf\ny\n",
        &DomDiffOptions {
            context_lines: 1,
            ..Default::default()
        },
        || false,
    )
    .unwrap();
    assert_eq!(split.hunks.len(), 2);
    split.validate().unwrap();
}

#[test]
fn t08_budgets_cancellation_and_large_inputs_never_erase_exact_verdict() {
    for options in [
        DomDiffOptions {
            max_work: 0,
            ..Default::default()
        },
        DomDiffOptions {
            max_output_lines: 0,
            ..Default::default()
        },
        DomDiffOptions {
            max_output_bytes: 0,
            ..Default::default()
        },
    ] {
        let diff = diff_text("same\nold\nend\n", "same\nnew\nend\n", &options, || false).unwrap();
        assert!(diff.changed && diff.fallback.is_some());
        diff.validate().unwrap();
        assert!(
            !diff_text("same\n", "same\n", &options, || true)
                .unwrap()
                .changed
        );
    }
    let cancelled = diff_text("a\n", "b\n", &Default::default(), || true).unwrap();
    assert!(cancelled.changed && cancelled.hunks.is_empty());
    assert_eq!(cancelled.fallback, Some(DomDiffFallback::Cancelled));
    cancelled.validate().unwrap();
    let calls = Cell::new(0);
    let middle_cancel = diff_text(
        &"a\n".repeat(2000),
        &"b\n".repeat(2000),
        &Default::default(),
        || {
            calls.set(calls.get() + 1);
            calls.get() > 3
        },
    )
    .unwrap();
    assert!(middle_cancel.changed);
    middle_cancel.validate().unwrap();
    let many = "\n".repeat(MAX_DOM_DIFF_INPUT_LINES + 1);
    let limited = diff_text(&many, "different", &Default::default(), || false).unwrap();
    assert!(limited.changed && limited.display_limited);
    assert_eq!(limited.fallback, Some(DomDiffFallback::InputLineLimit));
    limited.validate().unwrap();
    assert!(
        diff_text(
            &"x".repeat(MAX_DOM_SOURCE_BYTES + 1),
            "",
            &Default::default(),
            || false
        )
        .is_err()
    );
}

#[test]
fn t08_exhaustive_small_edits_reconstruct_both_complete_originals() {
    let mut texts = vec![String::new()];
    for length in 1..=5 {
        for bits in 0..(1 << length) {
            let text: String = (0..length)
                .map(|i| {
                    if bits & (1 << i) == 0 {
                        "a\n"
                    } else {
                        "🦀\n"
                    }
                })
                .collect();
            texts.push(text);
        }
    }
    for before in &texts {
        for after in &texts {
            for max_work in [0, MAX_DOM_DIFF_WORK] {
                let options = DomDiffOptions {
                    context_lines: 100,
                    max_work,
                    ..Default::default()
                };
                let diff = diff_text(before, after, &options, || false).unwrap();
                diff.validate().unwrap();
                if before == after {
                    assert!(!diff.changed);
                    continue;
                }
                let old: String = diff
                    .hunks
                    .iter()
                    .flat_map(|h| &h.rows)
                    .filter(|row| row.old_line.is_some())
                    .map(|row| format!("{}{}", row.text, if row.has_newline { "\n" } else { "" }))
                    .collect();
                let new: String = diff
                    .hunks
                    .iter()
                    .flat_map(|h| &h.rows)
                    .filter(|row| row.new_line.is_some())
                    .map(|row| format!("{}{}", row.text, if row.has_newline { "\n" } else { "" }))
                    .collect();
                assert_eq!(&old, before);
                assert_eq!(&new, after);
            }
        }
    }
}

#[test]
fn t01_t14_default_serialization_preserves_legacy_profile_hash_and_review_identity() {
    let legacy = include_str!("fixtures/legacy-capture-profile.json").trim_end();
    let profile: CaptureProfile = serde_json::from_str(legacy).unwrap();
    assert_eq!(serde_json::to_string(&profile).unwrap(), legacy);
    assert_eq!(
        identity_hash(&profile).unwrap(),
        "1b25101a859048c7cf2d7f07f176f47cd03193f8470f89861ca08e29e7255ba5"
    );
    let project = Project::draft("Fixture".into(), "https://example.test/").unwrap();
    let json = serde_json::to_string(&project).unwrap();
    assert!(!json.contains("capture_plan"));
    assert_eq!(
        serde_json::from_str::<Project>(&json).unwrap().capture_plan,
        CapturePlan::default()
    );
    for mode in [
        CaptureMode::ScreenshotsOnly,
        CaptureMode::ScreenshotsAndDomSnapshots,
        CaptureMode::DomSnapshotsOnly,
    ] {
        let mut copy = project.clone();
        copy.capture_plan = CapturePlan::for_mode(mode);
        copy.validate().unwrap();
        assert_eq!(
            serde_json::from_slice::<Project>(&serde_json::to_vec(&copy).unwrap()).unwrap(),
            copy
        );
        assert_eq!(
            identity_hash(&copy.profiles[0]).unwrap(),
            identity_hash(&project.profiles[0]).unwrap()
        );
    }
    let legacy_run = run(CaptureMode::ScreenshotsOnly, 1);
    let record = RunRecord {
        run: legacy_run,
        capture: None,
        captures: vec![],
        dom_captures: vec![],
        queue: None,
    };
    let serialized = serde_json::to_string(&record).unwrap();
    assert!(!serialized.contains("dom_") && !serialized.contains("capture_plan"));
    let decoded: RunRecord = serde_json::from_str(&serialized).unwrap();
    assert!(decoded.dom_captures.is_empty());
    assert!(decoded.run.targets[0].dom_outcome.is_not_requested());
    let review = ReviewDecision {
        analysis: AnalysisId::new(),
        key: decoded.run.targets[0].key.clone(),
        state: ReviewState::Intentional,
        note: "legacy".into(),
        reviewed_at: 2,
    };
    let json = serde_json::to_value(&review).unwrap();
    assert_eq!(json.as_object().unwrap().len(), 5);
    assert!(json.get("kind").is_none());
    let bad = CapturePlan {
        dom_policy: Some(Default::default()),
        ..Default::default()
    };
    assert!(bad.validate().is_err());
    assert!(
        CapturePlan {
            version: 2,
            ..Default::default()
        }
        .validate()
        .is_err()
    );
}

#[test]
fn t05_t06_independent_outcomes_and_dom_only_have_no_image_requirement() {
    let mut dom_only = run(CaptureMode::DomSnapshotsOnly, 1);
    let dom = dom_capture(&mut dom_only, 0, true);
    dom_only.state = JobState::Completed;
    dom_only.finished_at = Some(2);
    dom_only.validate().unwrap();
    let json = serde_json::to_string(&dom).unwrap();
    for forbidden in [
        "\"image\"",
        "\"thumbnail\"",
        "\"eligible_pixels\"",
        "\"tiles\"",
    ] {
        assert!(!json.contains(forbidden));
    }
    dom.metadata.verify_snapshot(GOLDEN).unwrap();
    dom.metadata.validate_for_run(&dom_only).unwrap();
    assert!(dom.metadata.baseline_ready());
    let mut mixed = run(CaptureMode::ScreenshotsAndDomSnapshots, 1);
    screenshot(&mut mixed, 0);
    mixed.targets[0].dom_outcome = DomTargetOutcome::Failed {
        code: DomCaptureError::LimitExceeded,
    };
    mixed.state = JobState::CompletedWithErrors;
    mixed.finished_at = Some(2);
    mixed.validate().unwrap();
    mixed.state = JobState::Completed;
    assert!(mixed.validate().is_err());
    dom_capture(&mut mixed, 0, true);
    mixed.targets[0].outcome = TargetOutcome::Failed {
        code: CaptureError::ImageLimit,
    };
    mixed.state = JobState::CompletedWithErrors;
    mixed.validate().unwrap();
    assert_eq!(
        regressee_core::crawl::TargetCounts::from_run(&mixed).failed,
        1
    );
    assert_eq!(
        regressee_core::crawl::TargetCounts::dom_from_run(&mixed).captured,
        1
    );
    mixed.targets[0].dom_outcome = DomTargetOutcome::Failed {
        code: DomCaptureError::Capture {
            code: CaptureError::ImageLimit,
        },
    };
    assert!(mixed.validate().is_err());
}

#[test]
fn t07_metadata_rejects_corruption_misbindings_unknown_policy_and_instability() {
    let mut run = run(CaptureMode::DomSnapshotsOnly, 1);
    let mut capture = dom_capture(&mut run, 0, true);
    capture.metadata.verify_snapshot(GOLDEN).unwrap();
    let mut changed = GOLDEN.to_vec();
    changed[10] ^= 1;
    assert!(capture.metadata.verify_snapshot(&changed).is_err());
    capture.metadata.coverage.template_contents += 1;
    assert!(capture.metadata.verify_snapshot(GOLDEN).is_err());
    capture.metadata.coverage.template_contents -= 1;
    capture.metadata.run = RunId::new();
    assert!(capture.metadata.validate_for_run(&run).is_err());
    capture.metadata.run = run.id;
    capture.metadata.stable = false;
    assert!(!capture.metadata.baseline_ready());
    capture.metadata.verify_snapshot(GOLDEN).unwrap();
    capture.metadata.stable = true;
    capture.metadata.policy.version = 2;
    assert!(capture.metadata.verify_snapshot(GOLDEN).is_err());
    capture.metadata.policy.version = 1;
    capture.metadata.warnings.push("UnknownWarning".into());
    assert!(!capture.metadata.baseline_ready());
    let malicious = wire(&[
        DomNode::Document,
        element(1, "input", vec![attr("value", "SECRET")]),
    ]);
    capture.metadata.sha256 = dom_artifact_id(&malicious);
    capture.metadata.bytes = malicious.len() as u64;
    assert!(capture.metadata.verify_snapshot(&malicious).is_err());
}

#[test]
fn t01_t06_partial_target_union_frozen_intent_and_saved_slot_checking() {
    let mut original = run(CaptureMode::ScreenshotsAndDomSnapshots, 2);
    let image = screenshot(&mut original, 0);
    original.targets[0].dom_outcome = DomTargetOutcome::Failed {
        code: DomCaptureError::LimitExceeded,
    };
    let dom = dom_capture(&mut original, 1, true);
    original.targets[1].outcome = TargetOutcome::Failed {
        code: CaptureError::ImageLimit,
    };
    original.state = JobState::CompletedWithErrors;
    original.finished_at = Some(2);
    original.validate().unwrap();
    let record = RunRecord {
        run: original.clone(),
        capture: None,
        captures: vec![image],
        dom_captures: vec![dom],
        queue: None,
    };
    assert_eq!(record.baseline_save_counts(), Some((2, 2)));
    let coverage = record.baseline_save_coverage().unwrap();
    assert_eq!(
        (
            coverage.screenshot_captures,
            coverage.dom_captures,
            coverage.screenshot_intended,
            coverage.dom_intended
        ),
        (1, 1, 2, 2)
    );
    let mut baseline = baseline(&original);
    assert!(baseline.validate().is_err());
    baseline.explicitly_partial = true;
    baseline.validate().unwrap();
    assert_eq!(baseline.target_keys().len(), 2);
    assert_eq!(baseline.saved_units().len(), 2);
    assert_eq!(baseline.intended_units(&original).unwrap().len(), 4);
    assert_eq!(baseline.omitted_units(&original).unwrap().len(), 2);
    let mut check = original.clone();
    check.id = RunId::new();
    check.intent = RunIntent::Check {
        baseline: baseline.id,
    };
    check.state = JobState::Queued;
    check.finished_at = None;
    check.targets[0].outcome = TargetOutcome::Pending;
    check.targets[0].dom_outcome = DomTargetOutcome::NotRequested;
    check.targets[1].outcome = TargetOutcome::NotRequested;
    check.targets[1].dom_outcome = DomTargetOutcome::Pending;
    baseline.validate_check_slots(&check).unwrap();
    check.targets[0].dom_outcome = DomTargetOutcome::Pending;
    assert!(baseline.validate_check_slots(&check).is_err());
    let replacement = RunId::new();
    baseline.references[0].run = replacement;
    assert_eq!(baseline.frozen_source_run(), Some(original.id));
    assert!(
        baseline.source_runs().contains(&replacement)
            && baseline.source_runs().contains(&original.id)
    );
    let mut dom_only = run(CaptureMode::DomSnapshotsOnly, 1);
    dom_capture(&mut dom_only, 0, true);
    let snapshot_baseline = crate::baseline(&dom_only);
    snapshot_baseline.validate().unwrap();
    assert!(snapshot_baseline.references.is_empty());
    assert_eq!(snapshot_baseline.target_keys().len(), 1);
    assert_eq!(snapshot_baseline.frozen_source_run(), Some(dom_only.id));
}

#[test]
fn t06_t07_exact_comparison_gates_stability_runtime_and_material_identity() {
    let key = TargetKey {
        page: PageId::new(),
        profile: ProfileId::new(),
    };
    let before = source(&key, GOLDEN);
    let after = source(&key, GOLDEN);
    let unchanged = compare_dom_snapshots(
        before.clone(),
        GOLDEN,
        after.clone(),
        GOLDEN,
        &Default::default(),
        || false,
    )
    .unwrap();
    assert_eq!(unchanged.outcome, DomComparisonOutcome::Unchanged);
    for (mut candidate, reason) in [
        (after.clone(), DomNotComparableReason::Unstable),
        (
            after.clone(),
            DomNotComparableReason::MaterialProfileMismatch,
        ),
        (
            after.clone(),
            DomNotComparableReason::RenderingEnvironmentMismatch,
        ),
    ] {
        match reason {
            DomNotComparableReason::Unstable => candidate.stable = false,
            DomNotComparableReason::MaterialProfileMismatch => {
                candidate.material_profile = hash('e')
            }
            _ => candidate.rendering_environment = hash('e'),
        }
        let result = compare_dom_snapshots(
            before.clone(),
            GOLDEN,
            candidate,
            GOLDEN,
            &Default::default(),
            || false,
        )
        .unwrap();
        assert_eq!(
            result.outcome,
            DomComparisonOutcome::NotComparable { reason }
        );
        assert!(result.diff.is_none());
    }
    let changed_bytes = String::from_utf8(GOLDEN.to_vec())
        .unwrap()
        .replace("Ordinary label", "Different label")
        .into_bytes();
    let changed = compare_dom_snapshots(
        before,
        GOLDEN,
        source(&key, &changed_bytes),
        &changed_bytes,
        &DomDiffOptions {
            max_output_lines: 0,
            ..Default::default()
        },
        || false,
    )
    .unwrap();
    assert_eq!(changed.outcome, DomComparisonOutcome::Changed);
    assert!(changed.diff.unwrap().display_limited);
}

#[test]
fn t09_dom_reviews_and_approval_bind_kind_sources_and_both_baseline_parents() {
    let mut original = run(CaptureMode::ScreenshotsAndDomSnapshots, 1);
    screenshot(&mut original, 0);
    dom_capture(&mut original, 0, true);
    let checked = baseline(&original);
    let key = original.targets[0].key.clone();
    let before = DomSourceIdentity {
        run: original.id,
        ..source(&key, GOLDEN)
    };
    let changed_bytes = String::from_utf8(GOLDEN.to_vec())
        .unwrap()
        .replace("Continue", "Proceed")
        .into_bytes();
    let after = source(&key, &changed_bytes);
    let result = compare_dom_snapshots(
        before.clone(),
        GOLDEN,
        after.clone(),
        &changed_bytes,
        &Default::default(),
        || false,
    )
    .unwrap();
    let revision = DomAnalysisRevision {
        version: 1,
        id: DomAnalysisId::new(),
        baseline: checked.id,
        run: after.run,
        algorithm: DOM_DIFF_ALGORITHM.into(),
        results: vec![result],
    };
    let mut review = DomReviewDecision {
        analysis: revision.id,
        key: key.clone(),
        before: before.clone(),
        after: after.clone(),
        state: ReviewState::Intentional,
        note: "DOM only".into(),
        reviewed_at: 4,
    };
    review.validate_for(&revision).unwrap();
    review.after.run = RunId::new();
    assert!(review.validate_for(&revision).is_err());
    review.after = after.clone();
    review.note = "🦀".repeat(2049);
    assert!(review.validate_for(&revision).is_err());
    let mut current = checked.clone();
    current.id = BaselineId::new();
    current.parent = Some(checked.id);
    current.references[0].run = RunId::new();
    let binding = DomApprovalBinding {
        checked_baseline: checked.id,
        current_parent: current.id,
        project_revision: 7,
        selections: vec![DomApprovalSelection {
            analysis: revision.id,
            key: key.clone(),
            before,
            after,
        }],
    };
    binding.validate_sources(&revision, &current).unwrap();
    current.dom_references[0].run = RunId::new();
    assert!(
        binding.validate_sources(&revision, &current).is_err(),
        "equal hashes from a different source cannot rebase"
    );
    current.dom_references.clear();
    current.explicitly_partial = true;
    assert!(
        binding.validate_sources(&revision, &current).is_err(),
        "no After-only approval"
    );
}

#[test]
fn t10_archive_dom_version_is_explicit_and_legacy_settings_stay_unchanged() {
    let mut run = run(CaptureMode::DomSnapshotsOnly, 1);
    dom_capture(&mut run, 0, true);
    let baseline = baseline(&run);
    assert_eq!(
        ArchiveManifest::version_for_baseline(&baseline),
        DOM_ARCHIVE_VERSION
    );
    let mut project = Project::draft("Portable fixture".into(), "https://example.test/").unwrap();
    assert!(
        !serde_json::to_string(&PortableProject::from(&project))
            .unwrap()
            .contains("capture_plan")
    );
    project.capture_plan = run.capture_plan.clone();
    let mut manifest = ArchiveManifest {
        version: DOM_ARCHIVE_VERSION,
        project: PortableProject::from(&project),
        inventory: run.inventory.clone(),
        baseline,
        artifacts: vec![Artifact {
            id: dom_artifact_id(GOLDEN),
            bytes: GOLDEN.len() as u64,
            kind: ArtifactKind::DomSnapshot,
        }],
        provenance: vec![],
        masks: vec![],
    };
    manifest.validate_version().unwrap();
    manifest.version = ARCHIVE_VERSION;
    assert!(manifest.validate_version().is_err());
    manifest.version = 99;
    assert!(manifest.validate_version().is_err());
}

#[test]
fn t12_html_text_encoder_is_inert_and_preserves_untrusted_text() {
    let dangerous = "</script></pre><img src='https://example.test' onerror=\"x\"> &";
    let encoded = escape_dom_html_text(dangerous);
    assert!(
        !encoded.contains('<')
            && !encoded.contains('>')
            && !encoded.contains('"')
            && !encoded.contains('\'')
    );
    assert!(
        encoded.contains("&lt;/script&gt;")
            && encoded.contains("&amp;")
            && encoded.contains("&#39;")
            && encoded.contains("&quot;")
    );
}

#[test]
fn c1_mapped_capture_binds_only_the_exact_approved_destination() {
    let mut run = run(CaptureMode::DomSnapshotsOnly, 1);
    run.targets[0].requested = Url::parse("https://example.test/p0?b=2&a=1&a=0#/orders").unwrap();
    run.inventory.pages[0].canonical = run.targets[0].requested.clone();
    run.environment.mappings = vec![OriginMapping {
        source: Url::parse("https://example.test/").unwrap(),
        destination: Url::parse("https://stage.test/preview/").unwrap(),
    }];
    let mut capture = dom_capture(&mut run, 0, true);
    let mapped =
        regressee_core::environment::map_url(&run.targets[0].requested, &run.environment).unwrap();
    capture.metadata.requested = mapped.to_string();
    capture.metadata.final_url = mapped.to_string();
    run.targets[0].final_url = Some(mapped.clone());
    run.validate().unwrap();
    assert_eq!(
        mapped.as_str(),
        "https://stage.test/preview/p0?b=2&a=1&a=0#/orders"
    );
    capture.metadata.validate_for_run(&run).unwrap();
    capture.metadata.requested = run.targets[0].requested.to_string();
    assert!(
        capture.metadata.validate_for_run(&run).is_err(),
        "logical source cannot impersonate the mapped browser request"
    );
    capture.metadata.requested = "https://stage.test/preview/p0?a=0&a=1&b=2#/orders".into();
    assert!(
        capture.metadata.validate_for_run(&run).is_err(),
        "query order/multiplicity and fragment remain exact"
    );
    capture.metadata.requested = mapped.to_string();
    run.environment
        .mappings
        .push(run.environment.mappings[0].clone());
    assert!(
        capture.metadata.validate_for_run(&run).is_err(),
        "ambiguous mappings cannot authorize evidence"
    );
}

#[test]
fn c2_browser_html_names_and_namespace_identity_are_preserved_without_conflation() {
    // Names observed by the independent verifier in the pinned, offline Chromium
    // DOM, plus explicit DOM createElementNS prefix identity counterexamples.
    let bytes = include_bytes!("fixtures/dom-v1-html-names.ndjson");
    let snapshot = DomSnapshot::from_canonical(bytes).unwrap();
    assert_eq!(snapshot.canonical_bytes().unwrap(), bytes);
    for name in [
        "@click",
        "123",
        "[binding]",
        ":class",
        "x-on:click",
        "a:b:c",
    ] {
        let nodes = [
            DomNode::Document,
            element(1, "div", vec![attr(name, "ordinary")]),
        ];
        assert!(
            DomSnapshot::from_canonical(&wire(&nodes)).is_ok(),
            "HTML tokenizer name {name}"
        );
    }
    assert_eq!(dom_attribute_local_name("", "a:foo").unwrap(), "a:foo");
    assert_eq!(dom_attribute_local_name("", "b:foo").unwrap(), "b:foo");
    assert_eq!(
        dom_attribute_local_name("urn:fixture", "a:foo").unwrap(),
        "foo"
    );
    let distinct = vec![attr("a:foo", "A"), attr("b:foo", "B")];
    assert!(
        DomSnapshot::from_canonical(&wire(&[DomNode::Document, element(1, "div", distinct)]))
            .is_ok()
    );
    let duplicate = vec![namespaced_attr("a:foo", "A"), namespaced_attr("b:foo", "B")];
    assert!(
        DomSnapshot::from_canonical(&wire(&[DomNode::Document, element(1, "div", duplicate)]))
            .is_err(),
        "actual expanded names cannot duplicate"
    );
    for invalid in ["a:b:c", ":foo", "a:", "@click"] {
        assert!(dom_attribute_local_name("urn:fixture", invalid).is_err());
    }
    let mut first = element(1, "b", vec![]);
    if let DomNode::Element { prefix, .. } = &mut first {
        *prefix = Some("a".into());
    }
    let mut second = first.clone();
    if let DomNode::Element { prefix, .. } = &mut second {
        *prefix = Some("c".into());
    }
    let first_bytes = wire(&[DomNode::Document, first.clone()]);
    let second_bytes = wire(&[DomNode::Document, second]);
    assert_ne!(first_bytes, second_bytes);
    let first_source = DomSnapshot::from_canonical(&first_bytes)
        .unwrap()
        .format_source()
        .unwrap();
    let second_source = DomSnapshot::from_canonical(&second_bytes)
        .unwrap()
        .format_source()
        .unwrap();
    assert_ne!(first_source, second_source);
    assert!(first_source.contains("prefix=\"a\""));
    let key = TargetKey {
        page: PageId::new(),
        profile: ProfileId::new(),
    };
    let changed_prefix = compare_dom_snapshots(
        source(&key, &first_bytes),
        &first_bytes,
        source(&key, &second_bytes),
        &second_bytes,
        &Default::default(),
        || false,
    )
    .unwrap();
    assert_eq!(changed_prefix.outcome, DomComparisonOutcome::Changed);
    for prefix in [Some(String::new()), Some("a:b".into())] {
        if let DomNode::Element { prefix: value, .. } = &mut first {
            *value = prefix;
        }
        assert!(DomSnapshot::from_canonical(&wire(&[DomNode::Document, first.clone()])).is_err());
    }
    assert!(
        DomSnapshot::from_canonical(
            b"[\"regressee-dom\",1]\n[\"document\",0]\n[\"element\",1,\"urn:x\",\"b\",[],null]\n"
        )
        .is_err(),
        "null prefix must omit the optional field"
    );
    assert!(
        DomSnapshot::from_canonical(&wire(&[
            DomNode::Document,
            element(1, "div", vec![attr(&"x".repeat(1025), "")])
        ]))
        .is_err()
    );
}

#[test]
fn c2_local_name_privacy_gates_apply_inside_shadow_and_template_content() {
    for boundary in [false, true] {
        let host = if boundary { "template" } else { "div" };
        let marker = if boundary {
            DomNode::TemplateContent { depth: 2 }
        } else {
            DomNode::OpenShadow { depth: 2 }
        };
        for (tag, local) in [
            ("input", "value"),
            ("select", "value"),
            ("button", "value"),
            ("option", "selected"),
            ("input", "checked"),
            ("iframe", "srcdoc"),
            ("div", "onload"),
            ("style", "nonce"),
        ] {
            let malicious = wire(&[
                DomNode::Document,
                element(1, host, vec![]),
                marker.clone(),
                element(
                    3,
                    tag,
                    vec![namespaced_attr(&format!("x:{local}"), "DO_NOT_ECHO")],
                ),
            ]);
            let error = DomSnapshot::from_canonical(&malicious).unwrap_err();
            assert!(!error.to_string().contains("DO_NOT_ECHO"));
        }
        let ordinary = wire(&[
            DomNode::Document,
            element(1, host, vec![]),
            marker.clone(),
            element(
                3,
                "input",
                vec![attr("a:foo", "ordinary A"), attr("b:foo", "ordinary B")],
            ),
        ]);
        assert!(DomSnapshot::from_canonical(&ordinary).is_ok());
        let mut script = element(3, "script", vec![]);
        if let DomNode::Element { prefix, .. } = &mut script {
            *prefix = Some("s".into());
        }
        assert!(
            DomSnapshot::from_canonical(&wire(&[
                DomNode::Document,
                element(1, host, vec![]),
                marker.clone(),
                script,
                DomNode::Text {
                    depth: 4,
                    text: "DO_NOT_ECHO".into()
                }
            ]))
            .is_err()
        );
        let url_code = wire(&[
            DomNode::Document,
            element(1, host, vec![]),
            marker,
            element(
                3,
                "a",
                vec![namespaced_attr("x:href", " \0\tJa\nva\rScript:DO_NOT_ECHO")],
            ),
        ]);
        assert!(DomSnapshot::from_canonical(&url_code).is_err());
    }
}

#[test]
fn c3_historical_partial_approval_recovers_only_through_verified_ancestry() {
    let mut original = run(CaptureMode::ScreenshotsOnly, 2);
    screenshot(&mut original, 0);
    original.targets[1].outcome = TargetOutcome::Failed {
        code: CaptureError::Timeout,
    };
    original.state = JobState::CompletedWithErrors;
    original.finished_at = Some(2);
    original.validate().unwrap();
    let mut root = baseline(&original);
    root.source_run = None;
    root.intended_targets.clear();
    root.explicitly_partial = true;
    let mut check = original.clone();
    check.id = RunId::new();
    check.intent = RunIntent::Check { baseline: root.id };
    check.targets.truncate(1);
    check.state = JobState::Completed;
    check.finished_at = Some(4);
    check.targets[0].outcome = TargetOutcome::Captured {
        artifact: hash('e'),
        width: 10,
        height: 10,
    };
    root.validate_check_slots(&check).unwrap();
    let mut legacy = root.clone();
    legacy.parent = Some(root.id);
    legacy.id = BaselineId::new();
    legacy.references[0].run = check.id;
    legacy.references[0].artifact = hash('e');
    legacy.references[0].captured_at = 4;
    let historical_bytes = serde_json::to_vec(&legacy).unwrap();
    assert_eq!(
        legacy.frozen_source_run(),
        None,
        "current reference owner is not proven frozen configuration"
    );
    assert!(legacy.intended_target_keys(&check).is_err());
    assert!(
        legacy.freeze_source(&original).is_err(),
        "even a real original needs its linked ancestry proof after replacement"
    );
    assert!(legacy.freeze_source(&check).is_err());
    assert!(
        legacy
            .recover_legacy_source(&[], &original, |_| Ok(check.clone()))
            .is_err()
    );
    assert_eq!(
        serde_json::to_vec(&legacy).unwrap(),
        historical_bytes,
        "failed proof is atomic"
    );
    let mut unrelated = original.clone();
    unrelated.id = RunId::new();
    assert!(
        legacy
            .recover_legacy_source(std::slice::from_ref(&root), &unrelated, |_| Ok(
                check.clone()
            ))
            .is_err(),
        "same inventory is not source provenance"
    );
    let mut wrong_parent = root.clone();
    wrong_parent.id = BaselineId::new();
    assert!(
        legacy
            .recover_legacy_source(&[wrong_parent], &original, |_| Ok(check.clone()))
            .is_err()
    );
    let mut cyclic_parent = root.clone();
    cyclic_parent.parent = Some(legacy.id);
    assert!(
        legacy
            .recover_legacy_source(&[cyclic_parent, legacy.clone()], &original, |_| Ok(
                check.clone()
            ))
            .is_err()
    );
    let mut extra_coverage = root.clone();
    extra_coverage.references.push(BaselineReference {
        key: original.targets[1].key.clone(),
        run: original.id,
        artifact: hash('a'),
        captured_at: 2,
    });
    assert!(
        legacy
            .recover_legacy_source(&[extra_coverage], &original, |_| Ok(check.clone()))
            .is_err(),
        "recovery cannot manufacture an omitted slot"
    );
    let mut wrong_update = check.clone();
    wrong_update.id = RunId::new();
    assert!(
        legacy
            .recover_legacy_source(std::slice::from_ref(&root), &original, |_| Ok(
                wrong_update.clone()
            ))
            .is_err()
    );
    let mut new_capture = original.clone();
    new_capture.id = check.id;
    new_capture.finished_at = Some(4);
    new_capture.targets[0].outcome = check.targets[0].outcome.clone();
    assert!(
        legacy
            .recover_legacy_source(std::slice::from_ref(&root), &original, |_| Ok(
                new_capture.clone()
            ))
            .is_err(),
        "cannot cross a new baseline capture boundary"
    );
    let mut wrong_artifact = check.clone();
    wrong_artifact.targets[0].outcome = TargetOutcome::Captured {
        artifact: hash('f'),
        width: 10,
        height: 10,
    };
    assert!(
        legacy
            .recover_legacy_source(std::slice::from_ref(&root), &original, |_| Ok(
                wrong_artifact.clone()
            ))
            .is_err()
    );
    let mut wrong_checked_baseline = check.clone();
    wrong_checked_baseline.intent = RunIntent::Check {
        baseline: BaselineId::new(),
    };
    assert!(
        legacy
            .recover_legacy_source(std::slice::from_ref(&root), &original, |_| Ok(
                wrong_checked_baseline.clone()
            ))
            .is_err()
    );
    let ancestor = legacy.clone();
    legacy
        .recover_legacy_source(std::slice::from_ref(&root), &original, |id| {
            assert_eq!(id, check.id);
            Ok(check.clone())
        })
        .unwrap();
    assert_eq!(legacy.frozen_source_run(), Some(original.id));
    assert_eq!(legacy.intended_target_keys(&original).unwrap().len(), 2);
    assert_eq!(legacy.omitted_units(&original).unwrap().len(), 1);
    assert!(
        legacy.source_runs().contains(&original.id) && legacy.source_runs().contains(&check.id)
    );
    assert_eq!(
        serde_json::to_vec(&ancestor).unwrap(),
        historical_bytes,
        "original history was not rewritten"
    );
    let mut next_check = check.clone();
    next_check.id = RunId::new();
    next_check.intent = RunIntent::Check {
        baseline: ancestor.id,
    };
    next_check.finished_at = Some(5);
    next_check.targets[0].outcome = TargetOutcome::Captured {
        artifact: hash('f'),
        width: 10,
        height: 10,
    };
    let mut next = ancestor.clone();
    next.id = BaselineId::new();
    next.parent = Some(ancestor.id);
    next.references[0].run = next_check.id;
    next.references[0].artifact = hash('f');
    next.references[0].captured_at = 5;
    let chain = [ancestor, root.clone()];
    // The capture boundary may have an older parent, but that edge must not
    // point back into the verified current -> intermediate -> boundary chain.
    for parent in [next.id, chain[0].id] {
        let mut cyclic = chain.clone();
        cyclic.last_mut().unwrap().parent = Some(parent);
        let before = serde_json::to_vec(&next).unwrap();
        assert!(
            next.recover_legacy_source(&cyclic, &original, |id| Ok(if id == next_check.id {
                next_check.clone()
            } else {
                check.clone()
            }))
            .is_err()
        );
        assert_eq!(serde_json::to_vec(&next).unwrap(), before);
    }
    let mut older = chain.clone();
    older.last_mut().unwrap().parent = Some(BaselineId::new());
    let mut recovered = next.clone();
    recovered
        .recover_legacy_source(&older, &original, |id| {
            Ok(if id == next_check.id {
                next_check.clone()
            } else {
                check.clone()
            })
        })
        .unwrap();
    assert_eq!(recovered.intended_target_keys(&original).unwrap().len(), 2);
    let mut stale = next_check.clone();
    stale.intent = RunIntent::Check { baseline: root.id };
    assert!(
        next.recover_legacy_source(&chain, &original, |id| Ok(if id == stale.id {
            stale.clone()
        } else {
            check.clone()
        }))
        .is_err(),
        "selected Before must still exactly match the checked ancestor"
    );
    next.recover_legacy_source(&chain, &original, |id| {
        Ok(if id == next_check.id {
            next_check.clone()
        } else {
            check.clone()
        })
    })
    .unwrap();
    assert_eq!(next.intended_target_keys(&original).unwrap().len(), 2);
}
