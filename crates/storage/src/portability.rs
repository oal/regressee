// SPDX-License-Identifier: MPL-2.0
use super::*;
use crate::{
    capture::{encode, now, pin},
    images::{check_cancel, validate_png},
};
use regressee_core::{archive::*, evidence::*};
use std::{
    collections::{BTreeMap, BTreeSet},
    sync::atomic::AtomicBool,
};
pub const ARCHIVE_BYTES: u64 = 1024 * 1024 * 1024;
pub const MANIFEST_BYTES: u64 = 8 * 1024 * 1024;
pub const ARCHIVE_ENTRIES: u16 = 10001;
pub const PRIVACY: &str = "Original screenshots may contain confidential information. Ignored regions DO NOT redact raw screenshots. Addresses and private URL queries remain in this ZIP. No login sessions, cookies, browser profiles, secret references, logs, machine paths or runtime binaries are included.";
pub const COMPATIBILITY: &str = "Imported portable provenance is not an independently verified local capture. Review environment mappings, establish your own login and install the exact matching runtime before explicitly checking the fixed targets. Other computers, fonts and operating systems can change rendering; caution mode is not a clean release check. Import never navigates or executes content.";
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ExportPreview {
    pub project: ProjectId,
    pub revision: u64,
    pub baseline: BaselineId,
    pub manifest_hash: String,
    pub targets: usize,
    pub originals: usize,
    pub bytes: u64,
    pub sources: usize,
    pub masks: usize,
    pub privacy: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ImportPreview {
    pub archive_hash: String,
    pub name: String,
    pub targets: usize,
    pub originals: usize,
    pub bytes: u64,
    pub masks: usize,
    pub warning: String,
}
pub struct ValidatedArchive {
    pub manifest: ArchiveManifest,
    pub preview: ImportPreview,
    stage: tempfile::TempDir,
}
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct ReplacementSummary {
    count: u32,
    geometry_measured_elements: u32,
    geometry_preserved: Option<bool>,
    cancelled_source_reloads: u32,
}
fn portable_metadata(mut m: CaptureMetadata) -> Result<CaptureMetadata> {
    if let Some(v) = &m.replacement {
        if v.get("count").is_some() {
            m.replacement = Some(serde_json::to_value(ReplacementSummary {
                count: v["count"]
                    .as_u64()
                    .context("replacement count missing")?
                    .try_into()?,
                geometry_measured_elements: v["geometryMeasuredElements"]
                    .as_u64()
                    .context("replacement geometry missing")?
                    .try_into()?,
                geometry_preserved: v["geometryPreserved"].as_bool(),
                cancelled_source_reloads: v["cancelledSourceReloads"]
                    .as_u64()
                    .unwrap_or(0)
                    .try_into()?,
            })?);
        } else {
            let _: ReplacementSummary = serde_json::from_value(v.clone())?;
        }
    }
    Ok(m)
}
fn portable_env(e: &Environment) -> PortableEnvironment {
    PortableEnvironment {
        name: e.name.clone(),
        mappings: e.mappings.clone(),
        access: e.access,
    }
}
impl Store {
    pub fn archive_manifest(
        &self,
        project: ProjectId,
        baseline: BaselineId,
    ) -> Result<ArchiveManifest> {
        let p = self.load_project(project)?;
        let mut b = self.load_baseline(baseline)?;
        ensure!(b.project == project, "Baseline belongs to another project");
        let inventory = self.load_inventory(b.inventory)?;
        // Ancestors are intentionally not included; the complete reference map stands alone.
        b.parent = None;
        let mut runs = BTreeMap::new();
        let mut artifacts = BTreeMap::new();
        let mut masks = Vec::new();
        let mut captures = BTreeMap::new();
        for r in &b.references {
            if let std::collections::btree_map::Entry::Vacant(e) = runs.entry(r.run) {
                // Load frozen metadata once per source. Only selected originals are
                // exported/verified; unrelated derivatives are not portable evidence.
                let source = self.load_run_snapshot(r.run)?;
                for c in source.all_captures() {
                    captures.insert((r.run, c.metadata.key.clone()), c.metadata.clone());
                }
                e.insert(CaptureProvenance {
                    run: r.run,
                    rendering: source.run.rendering.clone(),
                    profiles: source.run.profiles.clone(),
                    environment: portable_env(&source.run.environment),
                    started_at: source
                        .run
                        .started_at
                        .context("Source capture date missing")?,
                    finished_at: source
                        .run
                        .finished_at
                        .context("Source capture date missing")?,
                    captures: vec![],
                    discovery: source
                        .queue
                        .as_ref()
                        .map(|q| q.discovery.clone())
                        .unwrap_or_default(),
                });
            }
            let capture = captures
                .get(&(r.run, r.key.clone()))
                .context("Reference original missing")?;
            runs.get_mut(&r.run)
                .unwrap()
                .captures
                .push(portable_metadata(capture.clone())?);
            if let std::collections::btree_map::Entry::Vacant(e) =
                artifacts.entry(r.artifact.clone())
            {
                e.insert(self.verify_artifact(&r.artifact)?);
            }
            if let Some(mask) = self.latest_mask(project, &r.key)? {
                // Geometry is portable, pair authority is not. Recipient must revalidate.
                masks.push(PortableMask {
                    key: r.key.clone(),
                    width: mask.before_dimensions.width,
                    height: mask.before_dimensions.height,
                    rectangles: mask.rectangles,
                });
            }
        }
        let m = ArchiveManifest {
            version: ARCHIVE_VERSION,
            project: PortableProject::from(&p.project),
            inventory,
            baseline: b,
            artifacts: artifacts.into_values().collect(),
            provenance: runs.into_values().collect(),
            masks,
        };
        validate_manifest(&m)?;
        Ok(m)
    }
    pub fn export_preview(
        &self,
        project: ProjectId,
        baseline: BaselineId,
    ) -> Result<ExportPreview> {
        let m = self.archive_manifest(project, baseline)?;
        Ok(ExportPreview {
            project,
            revision: self.load_project(project)?.revision,
            baseline,
            manifest_hash: identity_hash(&m)?,
            targets: m.baseline.references.len(),
            originals: m.artifacts.len(),
            bytes: m.artifacts.iter().map(|a| a.bytes).sum(),
            sources: m.provenance.len(),
            masks: m.masks.len(),
            privacy: PRIVACY.into(),
        })
    }
    pub fn export_archive(
        &self,
        preview: &ExportPreview,
        destination: &Path,
        cancel: &AtomicBool,
    ) -> Result<()> {
        ensure!(
            &self.export_preview(preview.project, preview.baseline)? == preview,
            "Export contents/settings changed; preview again"
        );
        ensure!(
            !regular_file_or_absent(destination)?,
            "Export will not overwrite an existing file; choose a new ZIP name"
        );
        let m = self.archive_manifest(preview.project, preview.baseline)?;
        require_space(
            destination.parent().context("ZIP has no parent")?,
            preview.bytes + MANIFEST_BYTES,
        )?;
        let mut temp = tempfile::NamedTempFile::new_in(destination.parent().unwrap())?;
        {
            let mut zip = zip::ZipWriter::new(temp.as_file_mut());
            let options = zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .unix_permissions(0o600);
            zip.start_file("manifest.json", options)?;
            zip.write_all(&serde_json::to_vec(&m)?)?;
            for a in &m.artifacts {
                check_cancel(cancel)?;
                let path = self.artifact_path(&a.id)?;
                let dimensions = validate_png(&path, cancel)?;
                ensure!(
                    m.provenance
                        .iter()
                        .flat_map(|p| &p.captures)
                        .filter(|c| c.sha256 == a.id)
                        .all(|c| dimensions == (c.width, c.height)),
                    "Original screenshot PNG dimensions conflict"
                );
                zip.start_file(artifact_entry(&a.id), options)?;
                let mut f = fs::File::open(path)?;
                let mut buf = [0; 65536];
                loop {
                    check_cancel(cancel)?;
                    let n = f.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    zip.write_all(&buf[..n])?;
                }
            }
            zip.finish()?;
        }
        ensure!(
            temp.as_file().metadata()?.len() <= ARCHIVE_BYTES,
            "Large baseline exceeds the 1 GiB ZIP portability limit; select a smaller baseline"
        );
        temp.as_file().sync_all()?;
        check_cancel(cancel)?;
        temp.persist_noclobber(destination).map_err(|e| e.error)?;
        sync_directory(destination.parent().unwrap())?;
        Ok(())
    }
    pub fn inspect_archive(&self, path: &Path, cancel: &AtomicBool) -> Result<ValidatedArchive> {
        validate_archive(path, &self.paths.data, cancel)
    }
    pub fn import_archive(
        &mut self,
        path: &Path,
        expected: &ImportPreview,
        cancel: &AtomicBool,
    ) -> Result<ProjectRecord> {
        let validated = self.inspect_archive(path, cancel)?;
        ensure!(
            &validated.preview == expected,
            "Archive changed since preview; inspect it again"
        );
        self.publish_import(validated, cancel)
    }
    fn publish_import(
        &mut self,
        validated: ValidatedArchive,
        cancel: &AtomicBool,
    ) -> Result<ProjectRecord> {
        let mut m = validated.manifest;
        let project_id = ProjectId::new();
        let inventory_id = InventoryId::new();
        let baseline_id = BaselineId::new();
        let page_ids: BTreeMap<_, _> = m
            .inventory
            .pages
            .iter()
            .map(|p| (p.id, PageId::new()))
            .collect();
        let profile_ids: BTreeMap<_, _> = m.provenance[0]
            .profiles
            .iter()
            .map(|p| (p.id, ProfileId::new()))
            .collect();
        let run_ids: BTreeMap<_, _> = m.provenance.iter().map(|p| (p.run, RunId::new())).collect();
        let key = |k: &TargetKey| TargetKey {
            page: page_ids[&k.page],
            profile: profile_ids[&k.profile],
        };
        m.inventory.project = project_id;
        m.inventory.id = inventory_id;
        for p in &mut m.inventory.pages {
            p.id = page_ids[&p.id];
        }
        let mut future = m.project.profiles;
        for p in &mut future {
            p.id = ProfileId::new();
        }
        let mut project = Project {
            version: SCHEMA_VERSION,
            id: project_id,
            name: m.project.name,
            sources: m.project.sources,
            discovery: m.project.discovery,
            environments: m
                .project
                .environments
                .into_iter()
                .map(|e| Environment {
                    id: EnvironmentId::new(),
                    name: e.name,
                    mappings: e.mappings,
                    session: None,
                    access: e.access,
                })
                .collect(),
            profiles: future,
            active_baseline: Some(baseline_id),
        };
        // Build in an isolated private store. Failed decoding/derivative writes never touch recipient evidence.
        let staged_paths = AppPaths::isolated(&validated.stage.path().join("prepared"));
        let mut staged = Store::open(staged_paths.clone())?;
        let mut records = Vec::new();
        for origin in &mut m.provenance {
            check_cancel(cancel)?;
            for p in &mut origin.profiles {
                p.id = profile_ids[&p.id];
            }
            let env = if let Some(e) = project.environments.iter().find(|e| {
                e.name == origin.environment.name
                    && e.mappings == origin.environment.mappings
                    && e.access == origin.environment.access
            }) {
                e.clone()
            } else {
                ensure!(
                    project.environments.len() < 16,
                    "Selected historical provenance and current settings exceed 16 portable environments"
                );
                let e = Environment {
                    id: EnvironmentId::new(),
                    name: origin.environment.name.clone(),
                    mappings: origin.environment.mappings.clone(),
                    session: None,
                    access: origin.environment.access,
                };
                project.environments.push(e.clone());
                e
            };
            let mut captures = Vec::new();
            for meta in &mut origin.captures {
                meta.key = key(&meta.key);
                meta.profile.id = meta.key.profile;
                let image = images::ingest(
                    &mut staged,
                    &validated.stage.path().join(artifact_entry(&meta.sha256)),
                    ArtifactKind::Screenshot,
                    cancel,
                )?;
                captures.push(CaptureEvidence {
                    metadata: meta.clone(),
                    image,
                });
            }
            let mut targets = Vec::new();
            for page in m.inventory.pages.iter().filter(|p| p.included) {
                for profile in &origin.profiles {
                    let k = TargetKey {
                        page: page.id,
                        profile: profile.id,
                    };
                    let c = captures.iter().find(|c| c.metadata.key == k);
                    targets.push(CaptureTarget {
                        key: k,
                        requested: page.canonical.clone(),
                        final_url: c
                            .map(|c| url::Url::parse(&c.metadata.final_url))
                            .transpose()?,
                        attempts: if c.is_some() { 1 } else { 0 },
                        outcome: c
                            .map(|c| TargetOutcome::Captured {
                                artifact: c.image.original.clone(),
                                width: c.image.width,
                                height: c.image.height,
                            })
                            .unwrap_or(TargetOutcome::Skipped {
                                reason: "Not part of this selected portable snapshot".into(),
                            }),
                        warnings: vec![],
                    });
                }
            }
            let run = RunRecord {
                run: CaptureRun {
                    id: run_ids[&origin.run],
                    project: project_id,
                    intent: RunIntent::NewBaseline,
                    inventory: m.inventory.clone(),
                    profiles: origin.profiles.clone(),
                    environment: env,
                    rendering: origin.rendering.clone(),
                    state: if captures.len() == targets.len() {
                        JobState::Completed
                    } else {
                        JobState::CompletedWithErrors
                    },
                    started_at: Some(origin.started_at),
                    finished_at: Some(origin.finished_at),
                    targets,
                },
                capture: None,
                captures,
                queue: Some(QueueSettings {
                    discovery: origin.discovery.clone(),
                    project_revision: 1,
                    resumed_at: vec![],
                }),
            };
            run.run.validate()?;
            crate::capture::validate_capture(&run)?;
            records.push(run);
        }
        // Selected historical capture settings, never today's future replacement settings.
        // Future replacement settings remain portable and separately rebound. Checks read frozen source profiles.
        project.validate()?;
        m.baseline.id = baseline_id;
        m.baseline.project = project_id;
        m.baseline.inventory = inventory_id;
        m.baseline.parent = None;
        for r in &mut m.baseline.references {
            r.key = key(&r.key);
            r.run = run_ids[&r.run];
        }
        m.baseline.validate()?;
        let staged_artifacts: Vec<Artifact> = {
            let mut q = staged.connection.prepare("SELECT hash FROM artifacts")?;
            q.query_map([], |r| r.get::<_, String>(0))?
                .map(|id| staged.verify_artifact(&id?.try_into()?))
                .collect::<Result<_>>()?
        };
        require_space(
            &self.paths.data,
            staged_artifacts.iter().map(|a| a.bytes).sum(),
        )?;
        let tx = self
            .connection
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        project.name = unique_name(&tx, &project.name)?;
        // Atomic DB publication follows complete immutable object publication. A crash leaves only
        // complete unreferenced CAS files, retained and reported by explicit repair, never auto-GC.
        for (index, a) in staged_artifacts.iter().enumerate() {
            check_cancel(cancel)?;
            require_space(&self.paths.data, a.bytes)?;
            let destination = self.paths.artifacts().join(a.id.as_str());
            if regular_file_or_absent(&destination)? {
                verify_file(&destination, &a.id, a.bytes)?;
            } else {
                let mut f = tempfile::Builder::new()
                    .prefix(".pending-")
                    .tempfile_in(self.paths.artifacts())?;
                std::io::copy(
                    &mut fs::File::open(staged_paths.artifacts().join(a.id.as_str()))?,
                    &mut f,
                )?;
                f.as_file().sync_all()?;
                f.persist_noclobber(&destination).map_err(|e| e.error)?;
                sync_directory(&self.paths.artifacts())?;
            }
            import_fault(index + 1)?;
            tx.execute(
                "INSERT OR IGNORE INTO artifacts VALUES(?1,?2,?3)",
                params![a.id.as_str(), a.bytes, kind_name(a.kind)],
            )?;
            let actual: (u64, String) = tx.query_row(
                "SELECT bytes,kind FROM artifacts WHERE hash=?1",
                [a.id.as_str()],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )?;
            ensure!(
                actual == (a.bytes, kind_name(a.kind).into()),
                "Shared immutable artifact metadata conflicts"
            );
        }
        tx.execute(
            "INSERT INTO projects VALUES(?1,?2,?3,1,unixepoch())",
            params![project.id.to_string(), project.name, encode(&project)?],
        )?;
        tx.execute(
            "INSERT INTO inventories VALUES(?1,?2,?3)",
            params![
                inventory_id.to_string(),
                project_id.to_string(),
                encode(&m.inventory)?
            ],
        )?;
        for r in &records {
            tx.execute(
                "INSERT INTO runs VALUES(?1,?2,?3,?4,unixepoch(),0)",
                params![
                    r.run.id.to_string(),
                    project_id.to_string(),
                    serde_json::to_string(&r.run.state)?,
                    encode(r)?
                ],
            )?;
            tx.execute(
                "INSERT INTO imported_origins VALUES(?1,?2,?3)",
                params![r.run.id.to_string(), now(), validated.preview.archive_hash],
            )?;
            for c in r.all_captures() {
                for a in c.image.artifacts() {
                    pin(&tx, &self.paths, &ArtifactOwner::Run(r.run.id), &a)?;
                    pin(&tx, &self.paths, &ArtifactOwner::Baseline(baseline_id), &a)?;
                }
            }
        }
        tx.execute(
            "INSERT INTO baselines VALUES(?1,?2,?3,?4)",
            params![
                baseline_id.to_string(),
                project_id.to_string(),
                records[0].run.id.to_string(),
                encode(&m.baseline)?
            ],
        )?;
        for mask in m.masks {
            let k = key(&mask.key);
            let reference = m.baseline.references.iter().find(|r| r.key == k).unwrap();
            let c = records
                .iter()
                .find(|r| r.run.id == reference.run)
                .unwrap()
                .evidence(&k)
                .unwrap();
            let capture_id = format!("{}:{}", reference.run, c.image.original.as_str());
            let binding = regressee_core::analysis_engine::MaskBinding {
                revision: MaskId::new().to_string(),
                target: regressee_core::analysis_engine::Target {
                    page: k.page.to_string(),
                    profile: k.profile.to_string(),
                },
                before_capture_id: capture_id.clone(),
                after_capture_id: capture_id,
                before_dimensions: regressee_core::analysis_engine::Dimensions {
                    width: mask.width,
                    height: mask.height,
                },
                after_dimensions: regressee_core::analysis_engine::Dimensions {
                    width: mask.width,
                    height: mask.height,
                },
                rectangles: mask.rectangles,
            };
            tx.execute(
                "INSERT INTO ignore_masks VALUES(?1,?2,?3,?4,?5)",
                params![
                    binding.revision,
                    project_id.to_string(),
                    k.page.to_string(),
                    k.profile.to_string(),
                    encode(&binding)?
                ],
            )?;
        }
        check_cancel(cancel)?;
        tx.commit()?;
        Ok(ProjectRecord {
            project,
            revision: 1,
        })
    }
}
#[cfg(test)]
thread_local! {static IMPORT_FAULT:std::cell::Cell<Option<(usize,bool)>>=const{std::cell::Cell::new(None)};}
fn import_fault(_published: usize) -> Result<()> {
    #[cfg(test)]
    if let Some((n, disk_full)) = IMPORT_FAULT.get()
        && n == _published
    {
        IMPORT_FAULT.set(None);
        return Err(std::io::Error::new(
            if disk_full {
                std::io::ErrorKind::StorageFull
            } else {
                std::io::ErrorKind::Interrupted
            },
            "injected publication failure after a complete synced CAS object",
        )
        .into());
    }
    Ok(())
}

#[cfg(test)]
fn archive(root: &Path) -> Result<PathBuf> {
    let mut project = Project::draft(
        "Portable test".into(),
        "http://127.0.0.1:43210/secret?query=not-redacted",
    )?;
    project.profiles[0].width = 8;
    project.profiles[0].height = 8;
    project.profiles[0].runtime = RuntimeIdentity::Pinned {
        node: "fixture".into(),
        worker: "fixture".into(),
        playwright: "fixture".into(),
        browser: "fixture".into(),
    };
    let profile = project.profiles[0].clone();
    let page = InventoryPage {
        id: PageId::new(),
        canonical: project.sources[0].clone(),
        included: true,
        reason: None,
    };
    let key = TargetKey {
        page: page.id,
        profile: profile.id,
    };
    let inventory = InventoryRevision {
        id: InventoryId::new(),
        project: project.id,
        identity_version: 2,
        pages: vec![page],
        coverage: DiscoveryCoverage::Complete,
    };
    let run = RunId::new();
    let mut bytes = Vec::new();
    {
        let mut e = png::Encoder::new(&mut bytes, 8, 8);
        e.set_color(png::ColorType::Rgba);
        e.set_depth(png::BitDepth::Eight);
        e.write_header()?.write_image_data(&[127; 256])?;
    }
    let id: ArtifactId = hex::encode(Sha256::digest(&bytes)).try_into()?;
    let artifact = Artifact {
        id: id.clone(),
        bytes: bytes.len() as u64,
        kind: ArtifactKind::Screenshot,
    };
    let metadata = CaptureMetadata {
        version: 1,
        key: key.clone(),
        requested: project.sources[0].to_string(),
        final_url: project.sources[0].to_string(),
        runtime: profile.runtime.clone(),
        profile: profile.clone(),
        width: 8,
        height: 8,
        sha256: id.clone(),
        bytes: artifact.bytes,
        warnings: vec![],
        stable: true,
        duration_ms: 1,
        readiness: Readiness {
            font_failures: 0,
            image_failures: 0,
            resource_failures: 0,
        },
        capture_mode: "chromium-new-headless".into(),
        replacement: None,
    };
    let m = ArchiveManifest {
        version: 1,
        project: PortableProject::from(&project),
        inventory: inventory.clone(),
        baseline: BaselineVersion {
            id: BaselineId::new(),
            project: project.id,
            parent: None,
            inventory: inventory.id,
            created_at: 2,
            references: vec![BaselineReference {
                key,
                run,
                artifact: id.clone(),
                captured_at: 2,
            }],
            intended_count: 1,
            explicitly_partial: false,
        },
        artifacts: vec![artifact],
        provenance: vec![CaptureProvenance {
            run,
            rendering: RenderingEnvironment {
                os: "fixture".into(),
                architecture: "fixture".into(),
                browser: "fixture".into(),
                worker_version: "fixture".into(),
                profile_version: 1,
                fonts_fingerprint: None,
            },
            profiles: vec![profile],
            environment: portable_env(&project.environments[0]),
            started_at: 1,
            finished_at: 2,
            captures: vec![metadata],
            discovery: project.discovery,
        }],
        masks: vec![],
    };
    validate_manifest(&m)?;
    let path = root.join("baseline.zip");
    let mut z = zip::ZipWriter::new(fs::File::create(&path)?);
    let opts = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Stored)
        .unix_permissions(0o600);
    z.start_file("manifest.json", opts)?;
    z.write_all(&serde_json::to_vec(&m)?)?;
    z.start_file(artifact_entry(&id), opts)?;
    z.write_all(&bytes)?;
    z.finish()?;
    Ok(path)
}
#[test]
fn diskfull_interruption_rollback_and_explicit_orphan_recovery() -> Result<()> {
    for disk_full in [true, false] {
        let root = tempfile::tempdir()?;
        let path = archive(root.path())?;
        let mut store = Store::open(AppPaths::isolated(&root.path().join("recipient")))?;
        let existing = Project::draft("Keep me".into(), "http://127.0.0.1:43211/")?;
        store.save_project(&existing, None)?;
        let cancel = AtomicBool::new(false);
        let preview = store.inspect_archive(&path, &cancel)?.preview;
        IMPORT_FAULT.set(Some((1, disk_full)));
        let error = store.import_archive(&path, &preview, &cancel).unwrap_err();
        assert!(
            error.chain().any(
                |e| e.downcast_ref::<std::io::Error>().is_some_and(|e| e.kind()
                    == if disk_full {
                        std::io::ErrorKind::StorageFull
                    } else {
                        std::io::ErrorKind::Interrupted
                    })
            )
        );
        assert_eq!(store.summary()?.projects, 1);
        assert_eq!(store.summary()?.artifacts, 0);
        assert_eq!(store.load_project(existing.id)?.project, existing);
        drop(store);
        let mut reopened = Store::open(AppPaths::isolated(&root.path().join("recipient")))?;
        let repair = reopened.repair_preview()?;
        assert_eq!(repair.valid_unreferenced.len(), 1);
        assert!(repair.temporaries.is_empty());
        assert!(repair.missing_or_corrupt.is_empty());
        reopened.cleanup_temporaries(&repair)?;
        assert_eq!(
            reopened.repair_preview()?.valid_unreferenced.len(),
            1,
            "temporary repair must keep valid originals"
        );
        let old = reopened.repair_preview()?;
        let id: ArtifactId = old.valid_unreferenced[0].clone().try_into()?;
        // Adding any retained owner after impact invalidates GC authorization.
        let bytes = fs::read(reopened.paths.artifacts().join(id.as_str()))?;
        let a = reopened.put_artifact(bytes.as_slice(), ArtifactKind::Screenshot)?;
        reopened.add_reference(&ArtifactOwner::Run(RunId::new()), &a.id)?;
        assert!(reopened.delete_unreferenced_confirmed(&old).is_err());
        reopened.verify_artifact(&a.id)?;
    }
    Ok(())
}
#[test]
fn explicit_recovery_gc_is_confirmed_and_journal_rechecks_references() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut store = Store::open(AppPaths::isolated(root.path()))?;
    let a = store.put_artifact(&b"complete recovery evidence"[..], ArtifactKind::Screenshot)?;
    let b = store.put_artifact(&b"referenced evidence"[..], ArtifactKind::Screenshot)?;
    store.add_reference(&ArtifactOwner::Baseline(BaselineId::new()), &b.id)?;
    let preview = store.repair_preview()?;
    assert_eq!(preview.valid_unreferenced, vec![a.id.as_str()]);
    store.delete_unreferenced_confirmed(&preview)?;
    assert!(store.verify_artifact(&a.id).is_err());
    store.verify_artifact(&b.id)?;
    store
        .connection
        .execute("INSERT INTO pending_gc VALUES(?1)", [b.id.as_str()])?;
    store.complete_confirmed_gc()?;
    store.verify_artifact(&b.id)?;
    assert!(space_budget(0, 1).is_err());
    assert!(space_budget(129 * 1024 * 1024, 1024 * 1024).is_ok());
    Ok(())
}
#[cfg(target_os = "linux")]
#[test]
fn repair_classifies_dead_live_partial_and_valid_objects_and_guards_stale() -> Result<()> {
    let root = tempfile::tempdir()?;
    let paths = AppPaths::isolated(root.path());
    let mut store = Store::open(paths.clone())?;
    let valid = store.put_artifact(&b"valid retained evidence"[..], ArtifactKind::Screenshot)?;
    store.add_reference(&ArtifactOwner::Baseline(BaselineId::new()), &valid.id)?;
    fs::write(
        paths.artifacts().join(".pending-test"),
        b"partial interrupted write",
    )?;
    for (name, pid) in [
        (".import-staging-dead", u32::MAX),
        (".import-staging-live", std::process::id()),
    ] {
        private_dir(&paths.data.join(name))?;
        fs::write(paths.data.join(name).join(".owner-pid"), pid.to_string())?;
        private_dir(&paths.data.join(name).join("artifacts"))?;
        fs::write(
            paths.data.join(name).join("artifacts/partial"),
            b"private incomplete stage",
        )?;
    }
    let preview = store.repair_preview()?;
    assert_eq!(preview.temporaries, vec![".pending-test"]);
    assert_eq!(preview.interrupted_imports, vec![".import-staging-dead"]);
    fs::write(
        paths.artifacts().join(".pending-new"),
        b"another writer boundary",
    )?;
    assert!(store.cleanup_temporaries(&preview).is_err());
    let fresh = store.repair_preview()?;
    store.cleanup_temporaries(&fresh)?;
    assert!(!paths.data.join(".import-staging-dead").exists());
    assert!(
        paths
            .data
            .join(".import-staging-live/artifacts/partial")
            .exists()
    );
    store.verify_artifact(&valid.id)?;
    let corrupt = store.put_artifact(&b"will be corrupted"[..], ArtifactKind::Screenshot)?;
    fs::write(paths.artifacts().join(corrupt.id.as_str()), b"corrupt")?;
    assert_eq!(
        store.repair_preview()?.missing_or_corrupt,
        vec![corrupt.id.as_str()]
    );
    Ok(())
}
fn unique_name(tx: &rusqlite::Transaction<'_>, name: &str) -> Result<String> {
    let base: String = name.chars().take(96).collect();
    for i in 0..10000 {
        let candidate = if i == 0 {
            name.to_owned()
        } else {
            format!("{base} (imported {i})")
        };
        if tx.query_row(
            "SELECT count(*) FROM projects WHERE lower(name)=lower(?1)",
            [&candidate],
            |r| r.get::<_, u64>(0),
        )? == 0
        {
            return Ok(candidate);
        }
    }
    bail!("Too many imported projects with this name; rename an existing project first")
}
pub fn require_space(path: &Path, bytes: u64) -> Result<()> {
    space_budget(
        u128::from(regressee_core::platform::available_bytes(path)?),
        bytes,
    )
}
pub fn space_budget(available: u128, bytes: u64) -> Result<()> {
    if available < u128::from(bytes) + 128 * 1024 * 1024 {
        return Err(std::io::Error::new(
            std::io::ErrorKind::StorageFull,
            "Low disk space: free space and retry. Existing evidence is preserved.",
        )
        .into());
    }
    Ok(())
}
fn validate_manifest(m: &ArchiveManifest) -> Result<()> {
    ensure!(
        m.version == ARCHIVE_VERSION,
        "Unsupported/newer baseline archive version {}; this app supports {}",
        m.version,
        ARCHIVE_VERSION
    );
    ensure!(
        serde_json::to_vec(m)?.len() as u64 <= MANIFEST_BYTES,
        "Large baseline metadata exceeds 8 MiB portability limit"
    );
    m.inventory.validate()?;
    m.baseline.validate()?;
    ensure!(
        m.baseline.project == m.inventory.project
            && m.baseline.inventory == m.inventory.id
            && m.baseline.parent.is_none()
            && !m.baseline.id.0.is_nil()
            && m.baseline.created_at > 0,
        "Invalid portable baseline/inventory/parent"
    );
    let p = Project {
        version: SCHEMA_VERSION,
        id: m.inventory.project,
        name: m.project.name.clone(),
        sources: m.project.sources.clone(),
        discovery: m.project.discovery.clone(),
        environments: m
            .project
            .environments
            .iter()
            .map(|e| Environment {
                id: EnvironmentId::new(),
                name: e.name.clone(),
                mappings: e.mappings.clone(),
                session: None,
                access: e.access,
            })
            .collect(),
        profiles: m.project.profiles.clone(),
        active_baseline: None,
    };
    p.validate()?;
    ensure!(
        !m.provenance.is_empty() && m.provenance.len() <= m.baseline.references.len(),
        "Invalid provenance count"
    );
    let profiles = &m.provenance[0].profiles;
    let intended = m.inventory.pages.iter().filter(|p| p.included).count() * profiles.len();
    ensure!(
        intended == m.baseline.intended_count as usize,
        "Fixed inventory/profile coverage mismatch"
    );
    ensure!(
        intended
            .checked_mul(m.provenance.len())
            .is_some_and(|n| n <= MAX_TARGETS),
        "Large composite exceeds the 30,000 reconstructed source-target portability limit; this archive cannot be safely imported by this version"
    );
    let mut runs = BTreeSet::new();
    let mut captures = BTreeMap::new();
    for origin in &m.provenance {
        ensure!(
            !origin.run.0.is_nil()
                && runs.insert(origin.run)
                && origin.profiles == *profiles
                && origin.rendering == m.provenance[0].rendering
                && !origin.captures.is_empty()
                && origin.started_at > 0
                && origin.finished_at >= origin.started_at,
            "Duplicate/incompatible/empty provenance"
        );
        for p in &origin.profiles {
            p.validate()?;
            ensure!(p.ready_for_capture(), "Original profile runtime missing");
        }
        origin.discovery.validate()?;
        let e = Environment {
            id: EnvironmentId::new(),
            name: origin.environment.name.clone(),
            mappings: origin.environment.mappings.clone(),
            session: None,
            access: origin.environment.access,
        };
        let mut validation = p.clone();
        validation.environments = vec![e.clone()];
        validation.profiles = origin.profiles.clone();
        validation.validate()?;
        for label in [
            &origin.rendering.os,
            &origin.rendering.architecture,
            &origin.rendering.browser,
            &origin.rendering.worker_version,
        ] {
            validate_label(label, 128)?;
            ensure!(
                !label.contains(['/', '\\']),
                "Machine path is not portable rendering metadata"
            );
        }
        ensure!(
            origin.rendering.profile_version == 1
                && origin
                    .rendering
                    .fonts_fingerprint
                    .as_ref()
                    .is_none_or(|f| f.len() <= 128 && !f.contains(['/', '\\'])),
            "Invalid rendering fingerprint"
        );
        for c in &origin.captures {
            let page = m
                .inventory
                .pages
                .iter()
                .find(|p| p.id == c.key.page && p.included)
                .context("Foreign capture page")?;
            let profile = origin
                .profiles
                .iter()
                .find(|p| p.id == c.key.profile)
                .context("Foreign capture profile")?;
            let final_url = url::Url::parse(&c.final_url)?;
            validate_address(&final_url)?;
            ensure!(
                c.version == 1
                    && c.profile == *profile
                    && c.runtime == profile.runtime
                    && c.requested
                        == regressee_core::environment::map_url(&page.canonical, &e)?.as_str()
                    && c.capture_mode == "chromium-new-headless"
                    && c.duration_ms <= 300000
                    && c.warnings.len() <= 64
                    && c.warnings
                        .iter()
                        .all(|w| w.len() <= 128 && w.chars().all(|c| c.is_ascii_alphanumeric()))
                    && captures.insert((origin.run, c.key.clone()), c).is_none(),
                "Invalid/duplicate capture metadata binding"
            );
            images::budget(c.width, c.height)?;
            if let Some(v) = &c.replacement {
                let _: ReplacementSummary = serde_json::from_value(v.clone())
                    .context("Unexpected portable replacement metadata")?;
            }
        }
    }
    ensure!(
        captures.len() == m.baseline.references.len(),
        "Unselected history or missing capture included"
    );
    let mut needed = BTreeSet::new();
    for r in &m.baseline.references {
        let c = captures
            .get(&(r.run, r.key.clone()))
            .context("Baseline source missing")?;
        let origin = m.provenance.iter().find(|p| p.run == r.run).unwrap();
        ensure!(
            c.sha256 == r.artifact && r.captured_at == origin.finished_at,
            "Reference hash/date mismatch"
        );
        needed.insert(&r.artifact);
    }
    ensure!(
        m.artifacts.len() == needed.len() && m.artifacts.len() < ARCHIVE_ENTRIES as usize,
        "Large baseline exceeds 10,000 original portability limit"
    );
    let mut seen = BTreeSet::new();
    let mut total = 0u64;
    for a in &m.artifacts {
        ensure!(
            a.kind == ArtifactKind::Screenshot
                && (1..=MAX_ARTIFACT_BYTES).contains(&a.bytes)
                && needed.contains(&a.id)
                && seen.insert(&a.id),
            "Invalid/duplicate/unselected original"
        );
        total = total
            .checked_add(a.bytes)
            .context("Archive size overflow")?;
        for c in captures.values().filter(|c| c.sha256 == a.id) {
            ensure!(c.bytes == a.bytes, "Capture original size mismatch");
        }
    }
    ensure!(
        total + MANIFEST_BYTES + 2 * 1024 * 1024 <= ARCHIVE_BYTES,
        "Large baseline exceeds 1 GiB ZIP portability limit; select a smaller baseline"
    );
    let mut masks = BTreeSet::new();
    for mask in &m.masks {
        images::budget(mask.width, mask.height)?;
        ensure!(
            masks.insert(&mask.key) && m.baseline.references.iter().any(|r| r.key == mask.key),
            "Duplicate/foreign ignored-region metadata"
        );
        TargetMask {
            key: mask.key.clone(),
            width: mask.width,
            height: mask.height,
            rectangles: mask
                .rectangles
                .iter()
                .map(|r| Rectangle {
                    x: r.x,
                    y: r.y,
                    width: r.width,
                    height: r.height,
                })
                .collect(),
        }
        .validate()?;
    }
    Ok(())
}
fn validate_archive(path: &Path, data: &Path, cancel: &AtomicBool) -> Result<ValidatedArchive> {
    ensure!(regular_file_or_absent(path)?, "ZIP file missing");
    let size = fs::metadata(path)?.len();
    ensure!(
        size <= ARCHIVE_BYTES,
        "Large archive exceeds 1 GiB portability limit"
    );
    require_space(data, size + MANIFEST_BYTES)?;
    let stage = tempfile::Builder::new()
        .prefix(".import-staging-")
        .tempdir_in(data)?;
    private_dir(stage.path())?;
    let mut owner = create_private(&stage.path().join(".owner-pid"))?;
    owner.write_all(std::process::id().to_string().as_bytes())?;
    owner.sync_all()?;
    // Snapshot once: validation and import consume the exact same bytes even if the external file changes.
    let snapshot = stage.path().join("archive.zip");
    let mut out = create_private(&snapshot)?;
    let mut input = fs::File::open(path)?;
    let mut digest = Sha256::new();
    let mut count = 0u64;
    let mut buffer = [0; 65536];
    loop {
        check_cancel(cancel)?;
        let n = input.read(&mut buffer)?;
        if n == 0 {
            break;
        }
        count += n as u64;
        ensure!(count <= ARCHIVE_BYTES, "ZIP grew beyond limit");
        digest.update(&buffer[..n]);
        out.write_all(&buffer[..n])?;
    }
    out.sync_all()?;
    let archive_hash = hex::encode(digest.finalize());
    regressee_core::zip_directory::validate(fs::File::open(&snapshot)?, ARCHIVE_ENTRIES, |n| {
        n == "manifest.json" || validate_entry(n).is_ok()
    })?;
    let mut zip = zip::ZipArchive::new(fs::File::open(&snapshot)?)?;
    let mut total = 0u64;
    for i in 0..zip.len() {
        let entry = zip.by_index(i)?;
        ensure!(
            !entry.is_dir()
                && !entry.is_symlink()
                && entry
                    .unix_mode()
                    .is_none_or(|m| matches!(m & 0o170000, 0 | 0o100000) && m & 0o111 == 0)
                && matches!(
                    entry.compression(),
                    zip::CompressionMethod::Stored | zip::CompressionMethod::Deflated
                ),
            "Unsupported ZIP entry type/executable/compression"
        );
        let limit = if entry.name() == "manifest.json" {
            MANIFEST_BYTES
        } else {
            MAX_ARTIFACT_BYTES
        };
        ensure!(
            entry.size() <= limit
                && entry.size()
                    <= entry
                        .compressed_size()
                        .saturating_mul(200)
                        .saturating_add(1024 * 1024),
            "ZIP bomb or entry size exceeds limit"
        );
        total = total
            .checked_add(entry.size())
            .context("ZIP size overflow")?;
    }
    ensure!(
        total <= ARCHIVE_BYTES,
        "ZIP expanded size exceeds 1 GiB limit"
    );
    require_space(data, total.saturating_mul(3))?;
    let mut bytes = Vec::new();
    zip.by_name("manifest.json")
        .context("Baseline manifest missing")?
        .take(MANIFEST_BYTES + 1)
        .read_to_end(&mut bytes)?;
    ensure!(
        bytes.len() as u64 <= MANIFEST_BYTES,
        "Manifest exceeds 8 MiB limit"
    );
    let m: ArchiveManifest =
        serde_json::from_slice(&bytes).context("Malformed/unsupported baseline manifest")?;
    validate_manifest(&m)?;
    ensure!(
        zip.len() == m.artifacts.len() + 1,
        "Unlisted/duplicate ZIP entries"
    );
    private_dir(&stage.path().join("artifacts"))?;
    for a in &m.artifacts {
        check_cancel(cancel)?;
        let mut entry = zip
            .by_name(&artifact_entry(&a.id))
            .context("Original screenshot entry missing")?;
        ensure!(entry.size() == a.bytes, "ZIP original size mismatch");
        let target = stage.path().join(artifact_entry(&a.id));
        let mut f = create_private(&target)?;
        let n = std::io::copy(&mut (&mut entry).take(a.bytes + 1), &mut f)?;
        ensure!(n == a.bytes, "Truncated/oversized ZIP original");
        f.sync_all()?;
        verify_file(&target, &a.id, a.bytes)?;
        let dimensions = validate_png(&target, cancel)?;
        for c in m
            .provenance
            .iter()
            .flat_map(|p| &p.captures)
            .filter(|c| c.sha256 == a.id)
        {
            ensure!(
                dimensions == (c.width, c.height),
                "PNG dimensions disagree with manifest"
            );
        }
    }
    let preview = ImportPreview {
        archive_hash,
        name: m.project.name.clone(),
        targets: m.baseline.references.len(),
        originals: m.artifacts.len(),
        bytes: m.artifacts.iter().map(|a| a.bytes).sum(),
        masks: m.masks.len(),
        warning: format!("{PRIVACY}\n{COMPATIBILITY}"),
    };
    Ok(ValidatedArchive {
        manifest: m,
        preview,
        stage,
    })
}
