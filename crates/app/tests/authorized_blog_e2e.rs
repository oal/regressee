// SPDX-License-Identifier: MPL-2.0
//! Opt-in, bounded authorized public-blog pipeline; never an exhaustive-site claim.
//! Run only after the parent has approved the fresh evidence/scratch directories.
#![cfg(target_os = "linux")]

use anyhow::{Context, Result, ensure};
use regressee_app::product::{Product, RuntimeState};
use regressee_core::{
    crawl::{TargetCounts, exclusion, origin},
    domain::*,
    evidence::{RunRecord, identity_hash},
};
use regressee_runtime::distribution::{self, Scope};
use regressee_storage::{AppPaths, Store};
use serde_json::{Value, json};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
    fs::{self, File, OpenOptions},
    io::Write,
    os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt},
    path::{Path, PathBuf},
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::Instant,
};

const URL: &str = "https://example.test/";
const APPROVED: &str = "/path/to/private-tests/opencode";
const RETAINED: &str =
    "/path/to/private-tests/opencode/regressee-official-independent-_8cn2hk9/first-use-store";

fn physical_private(path: &Path) -> Result<()> {
    let meta = fs::symlink_metadata(path)?;
    ensure!(
        path.is_absolute() && fs::canonicalize(path)? == path && meta.is_dir(),
        "expected physical absolute directory: {}",
        path.display()
    );
    ensure!(
        meta.permissions().mode() & 0o7777 == 0o700 && meta.uid() == unsafe { libc::geteuid() },
        "directory must be owned and 0700"
    );
    Ok(())
}

fn new_file(path: &Path) -> Result<File> {
    Ok(OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)?)
}

struct Log {
    file: RefCell<File>,
    error: RefCell<Option<String>>,
    stages: RefCell<Vec<Value>>,
    start: Instant,
}
impl Log {
    fn emit(&self, value: Value) {
        let result = (|| -> Result<()> {
            let mut f = self.file.borrow_mut();
            writeln!(
                f,
                "{}",
                json!({"elapsed_seconds":self.start.elapsed().as_secs_f64(), "event":value})
            )?;
            f.sync_all()?;
            Ok(())
        })();
        if let Err(e) = result {
            *self.error.borrow_mut() = Some(format!("{e:#}"));
        }
    }
    fn stage<T>(&self, name: &str, run: impl FnOnce() -> Result<T>) -> Result<T> {
        self.emit(json!({"stage":name,"state":"started"}));
        eprintln!("Authorized blog stage: {name}");
        let start = Instant::now();
        let result = run();
        let settled = json!({"stage":name,"state":"settled","seconds":start.elapsed().as_secs_f64(),
            "success":result.is_ok(),"error":result.as_ref().err().map(|e| format!("{e:#}"))});
        self.stages.borrow_mut().push(settled.clone());
        self.emit(settled);
        result.with_context(|| format!("authorized blog stage {name}"))
    }
    fn progress(&self, stage: &str, message: &str, completed: u32, total: Option<u32>) {
        self.emit(json!({"stage":stage,"message":message,"completed":completed,"total":total}));
    }
}

// Direct public services own/drop their Worker pool; no Jobs actor is left alive.
// The parent also imposes an external 900s bound. Cancellation is set on unwind.
struct CancelOnDrop(Arc<AtomicBool>);
impl Drop for CancelOnDrop {
    fn drop(&mut self) {
        self.0.store(true, Ordering::Release);
    }
}

/// Read-only byte/mode/link inventory. Never follows runtime symlinks or writes
/// into the retained cache. Product's official resolver separately verifies pins.
fn cache_snapshot(root: &Path) -> Result<Value> {
    let mut entries = Vec::new();
    let mut pending = vec![root.to_path_buf()];
    let cancel = AtomicBool::new(false);
    while let Some(path) = pending.pop() {
        let m = fs::symlink_metadata(&path)?;
        let name = path.strip_prefix(root)?.to_string_lossy().into_owned();
        let entry = if m.file_type().is_symlink() {
            json!({"path":name,"kind":"link","mode":m.mode(),"target":fs::read_link(&path)?})
        } else if m.is_dir() {
            for child in fs::read_dir(&path)? {
                pending.push(child?.path());
            }
            json!({"path":name,"kind":"directory","mode":m.mode()})
        } else {
            ensure!(m.is_file(), "unexpected runtime-cache special file");
            json!({"path":name,"kind":"file","mode":m.mode(),"bytes":m.len(),
                "sha256":distribution::hash(File::open(&path)?, &cancel)?})
        };
        entries.push(entry);
    }
    entries.sort_by(|a, b| a["path"].as_str().cmp(&b["path"].as_str()));
    Ok(json!({"root":root,"tree_sha256":identity_hash(&entries)?,"entries":entries}))
}

fn capture_report(store: &Store, record: &RunRecord) -> Result<Value> {
    let mut originals = Vec::new();
    for c in record.all_captures() {
        for artifact in c.image.artifacts() {
            store.verify_artifact(&artifact)?;
        }
        let path = store.artifact_path(&c.image.original)?;
        let verified = store.verify_artifact(&c.image.original)?;
        let sha = distribution::hash(File::open(&path)?, &AtomicBool::new(false))?;
        ensure!(
            sha == c.image.original.as_str(),
            "stored original SHA mismatch"
        );
        originals.push(json!({"key":c.metadata.key,"path":path,"sha256":sha,
            "bytes":verified.bytes,"width":c.image.width,"height":c.image.height,
            "requested":c.metadata.requested,"final_url":c.metadata.final_url,
            "warnings":c.metadata.warnings,"stable":c.metadata.stable}));
    }
    Ok(
        json!({"record":record,"outcome_counts":TargetCounts::from_run(&record.run),
        "actual_captures":record.all_captures().count(),"originals":originals}),
    )
}

#[test]
#[ignore = "explicit authorized example.test HTTPS crawl; fresh private output, official sandboxed runtime only"]
fn authorized_blog_discovery_baseline_fixed_check_analysis_review() -> Result<()> {
    ensure!(
        std::env::var("REGRESSEE_AUTHORIZED_BLOG_URL").as_deref() == Ok(URL),
        "explicit REGRESSEE_AUTHORIZED_BLOG_URL=https://example.test/ required"
    );
    ensure!(
        std::env::var_os("REGRESSEE_DISTRIBUTION").is_none(),
        "distribution overrides are forbidden; use the default app-pinned catalog"
    );
    let root = PathBuf::from(
        std::env::var_os("REGRESSEE_AUTHORIZED_BLOG_ROOT")
            .context("explicit fresh approved private evidence root required")?,
    );
    physical_private(&root)?;
    ensure!(
        root.starts_with(APPROVED) && root != Path::new(APPROVED),
        "root outside approved parent"
    );
    ensure!(
        fs::read_dir(&root)?.next().is_none(),
        "evidence root must be fresh and empty"
    );
    // Linux's public supervisor requires a <=40-byte physical scratch parent.
    // The long approved evidence parent cannot satisfy that socket budget. The
    // caller supplies a separately approved NEW short directory, never a shared
    // product scratch namespace whose recovery could touch existing projects.
    let scratch = PathBuf::from(
        std::env::var_os("REGRESSEE_WORKER_SCRATCH_PARENT")
            .context("explicit fresh approved short worker scratch parent required")?,
    );
    physical_private(&scratch)?;
    ensure!(
        scratch.parent() == Some(Path::new("/tmp"))
            && scratch
                .file_name()
                .is_some_and(|name| name.to_string_lossy().starts_with("regressee-blog-"))
            && scratch.as_os_str().as_encoded_bytes().len() <= 40,
        "scratch must be a fresh short /tmp/regressee-blog-* directory (Linux socket limit)"
    );
    ensure!(
        fs::read_dir(&scratch)?.next().is_none(),
        "scratch must be fresh, not a previous namespace"
    );

    let mut report_file = new_file(&root.join("authorized-blog-report.json"))?;
    let log = Log {
        file: RefCell::new(new_file(&root.join("authorized-blog-progress.jsonl"))?),
        error: RefCell::new(None),
        stages: RefCell::new(Vec::new()),
        start: Instant::now(),
    };
    let cancel = Arc::new(AtomicBool::new(false));
    let _cancel_guard = CancelOnDrop(cancel.clone());
    let pause = Arc::new(AtomicBool::new(false));
    let paths = AppPaths::isolated(&root.join("blog-store"));
    let mut report = json!({"qualification":"Bounded authorized public-blog service pipeline, not native UI or exhaustive site coverage",
        "authorized_url":URL,"page_cap":12,"paths":paths,"scratch":scratch,
        "forms_or_clicks":false,"approval_applied":false,"exported":false,
        "limitations":["Rendered-link discovery only; origin-scoped navigation, not a claim that third-party page resources never load",
            "Live content may change; no required unchanged-result count", "Default full-page height is bounded by Product configuration (32768px)"]});
    let mut product = Product::configured(&paths.cache)?;
    let mut external_before = None;
    let mut failures = Vec::<String>::new();
    let mut project_id = None;
    let outcome = (|| -> Result<()> {
        if let Some(cache) = std::env::var_os("REGRESSEE_TEST_RUNTIME_CACHE") {
            let cache = PathBuf::from(cache);
            let retained_paths = AppPaths::isolated(Path::new(RETAINED));
            ensure!(
                cache == retained_paths.cache && fs::canonicalize(&cache)? == cache,
                "only the explicitly retained physical official AppPaths cache is permitted"
            );
            let retained = Product::configured(&cache)?;
            let snapshot = log.stage("readonly-runtime-before", || cache_snapshot(&cache))?;
            report["runtime_cache_before"] = snapshot.clone();
            external_before = Some((cache, snapshot));
            product.runtime_cache = retained.runtime_cache;
            ensure!(
                product.readiness(&cancel)?.status == RuntimeState::Ready,
                "retained cache is not ready; no automatic replacement/download is permitted"
            );
            product.status(&cancel)?;
        } else {
            ensure!(
                product.readiness(&cancel)?.status == RuntimeState::NotInstalled,
                "isolated default runtime must start absent"
            );
        }
        // Same first-use API as the app actor, not an alternate installer. When
        // retained current exists, ensure_runtime only revalidates, never setup.
        let mut setup_phases = Vec::new();
        let installed = log.stage("official-runtime-first-use-or-revalidate", || {
            product.ensure_runtime(cancel.clone(), &mut |phase| {
                let text = format!("{phase:?}");
                if setup_phases.last() != Some(&text) {
                    setup_phases.push(text.clone());
                }
                log.emit(json!({"stage":"runtime","phase":text}));
            })
        })?;
        report["runtime"] = json!({"root":installed.root,"manifest":installed.manifest,
            "status":product.status(&cancel)?,"setup_phases":setup_phases,
            "reused_readonly_cache":external_before.is_some()});
        ensure!(
            installed.manifest.scope == Scope::OfficialPinned
                && product.official_runtime
                && product.distribution.is_none(),
            "wrong runtime trust scope"
        );
        ensure!(
            external_before.is_none() || setup_phases.is_empty(),
            "retained runtime triggered setup"
        );
        let pointer_before = fs::read(product.runtime_cache.join("current"))?;
        let mut store = Store::open(paths.clone())?;
        let mut project = Project::draft("Authorized bounded example.test E2E".into(), URL)?;
        project.profiles = vec![CaptureProfile {
            width: 1280,
            height: 800,
            ..CaptureProfile::desktop()
        }];
        project.discovery.mode = DiscoveryMode::AddressesAndLinks;
        project.discovery.speed = Speed::Gentle;
        project.discovery.page_limit = 12;
        project.discovery.respect_robots = true;
        project.discovery.advanced.concurrency = Some(1);
        project.discovery.advanced.per_origin = 1;
        project.discovery.advanced.retries = 0;
        project.discovery.advanced.allow_action_links = false;
        project.discovery.advanced.robots_authorized = false;
        // Correct grammar: literal address substrings, NOT wildcard/regex rules.
        project.discovery.skipped_text = [
            "/logout",
            "/login",
            "/signin",
            "/signout",
            "/admin",
            "/wp-admin",
            "/wp-login",
            "/subscribe",
            "/unsubscribe",
            "/cart",
            "/checkout",
            "/delete",
            "/remove",
            "/feed",
            "?feed=",
            "&feed=",
        ]
        .into_iter()
        .map(str::to_owned)
        .collect();
        let saved = store.save_project(&project, None)?;
        project_id = Some(project.id);
        report["project_saved"] = serde_json::to_value(&saved)?;
        let saved_before = serde_json::to_value(&saved)?;
        let preview = log.stage("discovery", || {
            product.discover(&store, project.id, cancel.clone(), &mut |m, c, t| {
                log.progress("discovery", m, c, t)
            })
        })?;
        report["discovery"] = json!({"preview":preview,"included":preview.included(),
            "excluded":preview.inventory.pages.len()-preview.included(),"visited":preview.visited,
            "coverage_explanation":preview.inventory.coverage.label(),
            "exclusion_grammar":"literal case-sensitive address substrings plus built-in action-token guard"});
        ensure!(
            preview.project_revision == saved.revision
                && preview.settings == saved.project
                && serde_json::to_value(store.load_project(project.id)?)? == saved_before,
            "discovery settings/revision fence changed"
        );
        ensure!(
            (2..=12).contains(&preview.included()),
            "need >=2 included pages for meaningful evidence; inspect robots/exclusion reasons in report"
        );
        for page in preview.inventory.pages.iter().filter(|p| p.included) {
            ensure!(
                origin(&page.canonical) == "https://example.test"
                    && exclusion(&page.canonical, &project, 0).is_none(),
                "unapproved inventory target"
            );
        }
        let expected = preview.included() * project.profiles.len();
        report["expected_targets_per_run"] = json!(expected);
        let before = log.stage("baseline-capture-candidate", || {
            product.capture_inventory(
                &mut store,
                project.id,
                Some(preview),
                false,
                None,
                cancel.clone(),
                pause.clone(),
                &mut |m, c, t| log.progress("baseline-capture", m, c, t),
            )
        })?;
        report["baseline_candidate"] = capture_report(&store, &before)?;
        ensure!(
            before.run.intent == RunIntent::NewBaseline
                && store
                    .load_project(project.id)?
                    .project
                    .active_baseline
                    .is_none(),
            "candidate capture silently activated a baseline"
        );
        ensure!(
            before.run.state == JobState::Completed
                && before.run.targets.len() == expected
                && before.all_captures().count() == expected
                && expected >= 2,
            "incomplete baseline candidate; refusing activation, evidence retained"
        );
        ensure!(
            serde_json::to_value(store.load_project(project.id)?)? == saved_before,
            "candidate capture changed saved project"
        );
        let current_revision = store.load_project(project.id)?.revision;
        let activated = log.stage("explicit-save-baseline", || {
            product.activate(&mut store, before.run.id, current_revision)
        })?;
        let baseline = store.load_baseline(
            activated
                .project
                .active_baseline
                .context("baseline missing")?,
        )?;
        report["activation"] = serde_json::to_value(&activated)?;
        report["saved_baseline"] = serde_json::to_value(&baseline)?;
        // Saving the first baseline deliberately adopts the exact captured
        // profiles: pinned runtime/user agent and the renderer's height limit.
        // All other profile and project settings must remain untouched.
        ensure!(
            before.run.profiles.len() == project.profiles.len(),
            "capture profile count changed"
        );
        for (draft, captured) in project.profiles.iter().zip(&before.run.profiles) {
            let mut expected = draft.clone();
            expected.runtime = captured.runtime.clone();
            expected.user_agent = captured.user_agent.clone();
            expected.stabilization.max_height = draft.stabilization.max_height.min(32768);
            ensure!(
                expected == *captured && captured.runtime == installed.manifest.identity,
                "capture changed unrelated profile settings or runtime identity"
            );
        }
        let mut expected_activation = project.clone();
        expected_activation.active_baseline = Some(baseline.id);
        expected_activation.profiles = before.run.profiles.clone();
        ensure!(
            activated.project == expected_activation,
            "explicit baseline save changed unrelated settings"
        );
        let old_run = serde_json::to_value(store.load_run(before.run.id)?)?;
        let old_baseline = serde_json::to_value(&baseline)?;
        let activated_project = serde_json::to_value(&activated)?;
        let originals_before = capture_report(&store, &before)?;
        let check = log.stage("fixed-check-and-product-analysis", || {
            product.capture_inventory(
                &mut store,
                project.id,
                None,
                true,
                None,
                cancel.clone(),
                pause.clone(),
                &mut |m, c, t| log.progress("fixed-check-analysis", m, c, t),
            )
        })?;
        report["check"] = capture_report(&store, &check)?;
        let analysis = store
            .project_evidence(project.id)?
            .comparison
            .context("Product analysis missing")?;
        report["analysis"] = serde_json::to_value(&analysis)?;
        let mut classifications = BTreeMap::<String, usize>::from([
            ("Changed".into(), 0),
            ("NoSignificantChange".into(), 0),
            ("CouldNotCapture".into(), 0),
            ("NotComparable".into(), 0),
        ]);
        for r in &analysis.revision.results {
            *classifications
                .entry(format!("{:?}", r.classification))
                .or_default() += 1;
        }
        report["classification_counts"] = json!(classifications);
        // Review is triage only, never acceptance of live site changes. Retain
        // changed results as Unreviewed; failures explicitly need investigation.
        log.stage("review-triage-and-pure-preview", || {
            for result in &analysis.revision.results {
                let failed = matches!(result.classification, Classification::CouldNotCapture | Classification::NotComparable);
                store.save_review(&ReviewDecision { analysis:analysis.revision.id,key:result.key.clone(),
                    state:if failed { ReviewState::NeedsFix } else { ReviewState::Unreviewed },
                    note:if failed { "Automated E2E triage: capture/comparison not verified; investigate evidence, not approved." }
                        else { "Automated E2E evidence recorded; human review pending, no change approved." }.into(),
                    reviewed_at:distribution::now() })?;
            }
            let keys = analysis.revision.results.iter().filter(|r| r.classification == Classification::Changed)
                .map(|r| r.key.clone()).collect::<Vec<_>>();
            if !keys.is_empty() {
                report["optional_changed_approval_preview"] = match store.approval_preview(analysis.revision.id,&keys) {
                    Ok(p) => json!({"preview":p,"applied":false}),
                    Err(e) => json!({"refused":format!("{e:#}"),"applied":false}),
                };
            }
            report["reviews"] = serde_json::to_value(store.review_history(check.run.id)?)?;
            Ok(())
        })?;
        let original_preserved = serde_json::to_value(store.load_run(before.run.id)?)? == old_run
            && serde_json::to_value(store.load_baseline(baseline.id)?)? == old_baseline
            && capture_report(&store, &before)? == originals_before;
        let project_preserved =
            serde_json::to_value(store.load_project(project.id)?)? == activated_project;
        let history = store.baseline_history(project.id)?;
        report["preservation"] = json!({"baseline_and_original_screenshots":original_preserved,
            "saved_project_after_explicit_activation":project_preserved,"baseline_history":history});
        let before_keys = before
            .run
            .targets
            .iter()
            .map(|t| (&t.key, &t.requested))
            .collect::<BTreeSet<_>>();
        let check_keys = check
            .run
            .targets
            .iter()
            .map(|t| (&t.key, &t.requested))
            .collect::<BTreeSet<_>>();
        let analysis_keys = analysis
            .revision
            .results
            .iter()
            .map(|r| &r.key)
            .collect::<BTreeSet<_>>();
        let counts = TargetCounts::from_run(&check.run);
        for (ok, message) in [
            (
                original_preserved,
                "baseline record or original screenshot bytes changed",
            ),
            (
                project_preserved && history.len() == 1,
                "check/review changed project or activated another baseline",
            ),
            (
                check.run.intent
                    == (RunIntent::Check {
                        baseline: baseline.id,
                    }),
                "check did not pin explicit baseline",
            ),
            (
                check_keys == before_keys && check.run.profiles == before.run.profiles,
                "fixed targets/profiles changed",
            ),
            (
                check.run.targets.len() == expected
                    && counts.pending == 0
                    && counts.cancelled == 0
                    && counts.skipped == 0,
                "target outcomes incomplete or unexpected",
            ),
            (
                check.run.state == JobState::Completed
                    || check.run.state == JobState::CompletedWithErrors,
                "check did not honestly settle",
            ),
            (
                (check.run.state == JobState::Completed) == (counts.captured == expected)
                    && counts.captured
                        + counts.failed
                        + counts.skipped
                        + counts.cancelled
                        + counts.pending
                        == expected,
                "run state disagrees with recorded target outcomes",
            ),
            (
                counts.captured == check.all_captures().count() && counts.captured >= 2,
                "need >=2 actual check captures",
            ),
            (
                analysis.revision.results.len() == expected
                    && analysis_keys == check.run.targets.iter().map(|t| &t.key).collect(),
                "analysis classifications do not cover exact target map",
            ),
            (
                analysis.revision.baseline == baseline.id && analysis.revision.run == check.run.id,
                "analysis provenance mismatch",
            ),
        ] {
            if !ok {
                failures.push(message.into());
            }
        }
        for target in &check.run.targets {
            if !matches!(target.outcome, TargetOutcome::Captured { .. }) {
                let classification = analysis
                    .revision
                    .results
                    .iter()
                    .find(|r| r.key == target.key)
                    .map(|r| r.classification);
                if classification != Some(Classification::CouldNotCapture) {
                    failures.push("missing capture falsely classified".into());
                }
            }
        }
        let repeat = product.ensure_runtime(cancel.clone(), &mut |_| {
            failures.push("repeat runtime use triggered setup".into())
        })?;
        if repeat.manifest.identity != installed.manifest.identity
            || fs::read(product.runtime_cache.join("current"))? != pointer_before
        {
            failures.push("runtime generation/identity changed".into());
        }
        Ok(())
    })();
    // Even an early service failure retains the persisted queue, baseline and
    // analysis, and still verifies that the external cache was untouched.
    if let Some(id) = project_id {
        report["persisted_evidence_at_exit"] =
            match Store::open(paths.clone()).and_then(|s| s.project_evidence(id)) {
                Ok(e) => serde_json::to_value(e)?,
                Err(e) => json!({"read_error":format!("{e:#}")}),
            };
    }
    if let Some((cache, before)) = external_before {
        match log.stage("readonly-runtime-after", || {
            let after = cache_snapshot(&cache)?;
            let verifier = Product::configured(&cache)?;
            ensure!(
                verifier.readiness(&AtomicBool::new(false))?.status == RuntimeState::Ready,
                "retained runtime no longer verifies"
            );
            verifier.status(&AtomicBool::new(false))?;
            Ok(after)
        }) {
            Ok(after) => {
                let unchanged = before == after;
                report["runtime_cache_after"] = after;
                report["readonly_runtime_cache_unchanged"] = json!(unchanged);
                if !unchanged {
                    failures.push("retained cache bytes/modes/current/generation changed".into());
                }
            }
            Err(e) => failures.push(format!("retained cache post-verification: {e:#}")),
        }
    }
    if let Err(e) = &outcome {
        failures.push(format!("{e:#}"));
    }
    if let Some(e) = log.error.borrow().as_ref() {
        failures.push(format!("progress log: {e}"));
    }
    report["elapsed_seconds"] = json!(log.start.elapsed().as_secs_f64());
    report["stage_timings"] = json!(*log.stages.borrow());
    report["failures"] = json!(failures);
    report["passed"] = json!(failures.is_empty());
    serde_json::to_writer_pretty(&mut report_file, &report)?;
    report_file.write_all(b"\n")?;
    report_file.sync_all()?;
    ensure!(
        failures.is_empty(),
        "authorized blog evidence retained at {}: {}",
        root.display(),
        failures.join("; ")
    );
    Ok(())
}
