// SPDX-License-Identifier: MPL-2.0
//! Exact retained verifier specimens + independent encoded-stream/atomicity oracles.
use super::*;
use std::{collections::BTreeMap, io::Cursor, sync::atomic::AtomicBool};

const LONG: &[u8] = include_bytes!("../../../tests/fixtures/phase6-corrections/long-inflate.zip");
const UNKNOWN: &[u8] =
    include_bytes!("../../../tests/fixtures/phase6-corrections/unknown-coverage-field.zip");
const BAD_HASH: &str = "df6fa684825810ce6b558b8070085f7187895118d74f92a9deb75240044ca0d0";
fn entries(bytes: &[u8]) -> Result<Vec<(String, Vec<u8>)>> {
    let mut z = zip::ZipArchive::new(Cursor::new(bytes))?;
    (0..z.len())
        .map(|i| {
            let mut f = z.by_index(i)?;
            let name = f.name().to_owned();
            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes)?;
            Ok((name, bytes))
        })
        .collect()
}
fn zip_file(path: &Path, entries: &[(String, Vec<u8>)]) -> Result<()> {
    let mut z = zip::ZipWriter::new(fs::File::create(path)?);
    for (name, bytes) in entries {
        z.start_file(
            name,
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored)
                .unix_permissions(0o600),
        )?;
        z.write_all(bytes)?;
    }
    z.finish()?;
    Ok(())
}
fn valid_zip(path: &Path) -> Result<()> {
    // Keep the exact two genuinely captured Chromium originals. Remove ONLY the
    // independently reproduced unknown schema field to obtain a valid fixture.
    let mut e = entries(UNKNOWN)?;
    let mut m: serde_json::Value = serde_json::from_slice(&e[0].1)?;
    m["inventory"]["coverage"]["Limited"]
        .as_object_mut()
        .unwrap()
        .remove("script");
    e[0].1 = serde_json::to_vec(&m)?;
    zip_file(path, &e)
}
fn replace_original(
    mut entries: Vec<(String, Vec<u8>)>,
    bytes: &[u8],
) -> Result<Vec<(String, Vec<u8>)>> {
    let old = entries[1].0.strip_prefix("artifacts/").unwrap();
    let hash = hex::encode(Sha256::digest(bytes));
    let text = String::from_utf8(entries[0].1.clone())?.replace(old, &hash);
    let mut m: serde_json::Value = serde_json::from_str(&text)?;
    for a in m["artifacts"].as_array_mut().unwrap() {
        if a["id"] == hash {
            a["bytes"] = (bytes.len() as u64).into();
        }
    }
    for origin in m["provenance"].as_array_mut().unwrap() {
        for capture in origin["captures"].as_array_mut().unwrap() {
            if capture["sha256"] == hash {
                capture["bytes"] = (bytes.len() as u64).into();
            }
        }
    }
    entries[0].1 = serde_json::to_vec(&m)?;
    entries[1] = (format!("artifacts/{hash}"), bytes.to_vec());
    Ok(entries)
}
type Snapshot = (BTreeMap<String, Vec<String>>, BTreeMap<String, Vec<u8>>);
fn snapshot(s: &Store) -> Result<Snapshot> {
    let mut tables = s
        .connection
        .prepare("SELECT name FROM sqlite_master WHERE type='table' ORDER BY name")?;
    let names = tables
        .query_map([], |r| r.get::<_, String>(0))?
        .collect::<rusqlite::Result<Vec<_>>>()?;
    let mut rows = BTreeMap::new();
    for name in names {
        let mut q = s
            .connection
            .prepare(&format!("SELECT * FROM \"{}\"", name.replace('"', "\"\"")))?;
        let count = q.column_count();
        let mut values = q
            .query_map([], |r| {
                Ok((0..count)
                    .map(|i| format!("{:?}", r.get_ref(i).unwrap()))
                    .collect::<Vec<_>>()
                    .join("|"))
            })?
            .collect::<rusqlite::Result<Vec<_>>>()?;
        values.sort();
        rows.insert(name, values);
    }
    fn files(root: &Path, path: &Path, out: &mut BTreeMap<String, Vec<u8>>) -> Result<()> {
        for entry in fs::read_dir(path)? {
            let path = entry?.path();
            if path.is_dir() {
                out.insert(
                    format!("{}/", path.strip_prefix(root)?.display()),
                    Vec::new(),
                );
                files(root, &path, out)?;
            } else if !path
                .file_name()
                .unwrap()
                .to_string_lossy()
                .starts_with("regressee.sqlite3")
            {
                out.insert(
                    path.strip_prefix(root)?.display().to_string(),
                    fs::read(path)?,
                );
            }
        }
        Ok(())
    }
    let mut data = BTreeMap::new();
    for (prefix, path) in [("data", &s.paths.data), ("cache", &s.paths.cache)] {
        let mut local = BTreeMap::new();
        files(path, path, &mut local)?;
        data.extend(
            local
                .into_iter()
                .map(|(name, bytes)| (format!("{prefix}/{name}"), bytes)),
        );
    }
    Ok((rows, data))
}
fn specific<T>(r: Result<T>, cause: &str) {
    let error = match r {
        Ok(_) => panic!("accepted malformed specimen: {cause}"),
        Err(e) => e,
    };
    assert!(
        format!("{error:#}").contains(cause),
        "wanted {cause:?}, got {error:#}"
    );
}
fn crc(data: &[u8]) -> u32 {
    let mut c = !0u32;
    for &byte in data {
        c ^= u32::from(byte);
        for _ in 0..8 {
            c = (c >> 1) ^ if c & 1 != 0 { 0xedb88320 } else { 0 };
        }
    }
    !c
}
fn chunk(kind: &[u8; 4], payload: &[u8]) -> Vec<u8> {
    let mut v = (payload.len() as u32).to_be_bytes().to_vec();
    v.extend_from_slice(kind);
    v.extend_from_slice(payload);
    v.extend_from_slice(&crc(&v[4..]).to_be_bytes());
    v
}
fn stored(raw: &[u8]) -> Vec<u8> {
    let mut v = vec![0x78, 1];
    let blocks = raw.chunks(65535);
    let n = blocks.len();
    for (i, block) in blocks.enumerate() {
        v.push(u8::from(i + 1 == n));
        let count = block.len() as u16;
        v.extend_from_slice(&count.to_le_bytes());
        v.extend_from_slice(&(!count).to_le_bytes());
        v.extend_from_slice(block);
    }
    let (mut a, mut b) = (1u32, 0u32);
    for &byte in raw {
        a = (a + u32::from(byte)) % 65521;
        b = (b + a) % 65521;
    }
    v.extend_from_slice(&(b * 65536 + a).to_be_bytes());
    v
}
fn png(z: &[u8], terminator: &[u8]) -> Vec<u8> {
    let mut ihdr = 600u32.to_be_bytes().to_vec();
    ihdr.extend_from_slice(&300u32.to_be_bytes());
    ihdr.extend_from_slice(&[8, 6, 0, 0, 0]);
    [
        b"\x89PNG\r\n\x1a\n".as_slice(),
        &chunk(b"IHDR", &ihdr),
        &chunk(b"IDAT", z),
        &chunk(b"IEND", terminator),
    ]
    .concat()
}
#[test]
fn phase6_complete_png_stream_gate_agrees_with_core_and_preserves_all_evidence() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut s = Store::open(AppPaths::isolated(&root.path().join("store")))?;
    let good = root.path().join("good.zip");
    valid_zip(&good)?;
    let cancel = AtomicBool::new(false);
    let preview = s.inspect_archive(&good, &cancel)?.preview;
    let imported = s.import_archive(&good, &preview, &cancel)?;
    let b = s.load_baseline(imported.project.active_baseline.unwrap())?;
    let before = snapshot(&s)?;
    let exact = entries(LONG)?
        .into_iter()
        .find(|e| e.0 == format!("artifacts/{BAD_HASH}"))
        .unwrap()
        .1;
    let raw = vec![0; 2401 * 300];
    let z = stored(&raw);
    let mut truncated = z.clone();
    truncated.pop();
    let mut trailing = z.clone();
    trailing.push(0);
    let mut adler = z.clone();
    *adler.last_mut().unwrap() ^= 1;
    let mut crc_bad = png(&z, &[]);
    *crc_bad.last_mut().unwrap() ^= 1;
    for (name, bytes, cause) in [
        ("exact-long", exact, "zlib excess decoded bytes"),
        (
            "generated-long",
            png(&stored(&vec![0; 2401 * 301]), &[]),
            "zlib excess decoded bytes",
        ),
        (
            "short",
            png(&stored(&vec![0; 2401 * 299]), &[]),
            "zlib decoded byte count mismatch",
        ),
        (
            "trailing-zlib",
            png(&trailing, &[]),
            "zlib trailing compressed bytes",
        ),
        (
            "concatenated-zlib",
            png(&[z.as_slice(), z.as_slice()].concat(), &[]),
            "zlib trailing compressed bytes",
        ),
        ("adler", png(&adler, &[]), "zlib Adler32 mismatch"),
        (
            "truncated-zlib",
            png(&truncated, &[]),
            "zlib truncated stream",
        ),
        ("malformed-iend", png(&z, &[0]), "PNG invalid IEND"),
        ("crc", crc_bad, "PNG chunk CRC mismatch"),
    ] {
        let p = root.path().join(format!("{name}.png"));
        fs::write(&p, &bytes)?;
        specific(
            regressee_core::analysis_engine::validate_png(&mut Cursor::new(&bytes), &cancel),
            cause,
        );
        specific(images::validate_png(&p, &cancel), cause);
        specific(
            images::ingest(&mut s, &p, ArtifactKind::Screenshot, &cancel),
            cause,
        );
        let archive = root.path().join(format!("{name}.zip"));
        zip_file(
            &archive,
            &replace_original(entries(&fs::read(&good)?)?, &bytes)?,
        )?;
        specific(s.inspect_archive(&archive, &cancel), cause);
        specific(s.import_archive(&archive, &preview, &cancel), cause);
        assert_eq!(
            snapshot(&s)?,
            before,
            "{name} changed rows/files/active baseline"
        );
    }
    for r in &b.references {
        let p = s.artifact_path(&r.artifact)?;
        let bytes = fs::read(&p)?;
        assert_eq!(images::validate_png(&p, &cancel)?, (600, 300));
        assert_eq!(hex::encode(Sha256::digest(&bytes)), r.artifact.as_str());
        let image = images::ingest(&mut s, &p, ArtifactKind::Screenshot, &cancel)?;
        assert_eq!(image.original, r.artifact);
        assert_eq!(fs::read(&p)?, bytes);
    }
    let stopped = AtomicBool::new(true);
    let before = snapshot(&s)?;
    specific(
        images::ingest(
            &mut s,
            &root.path().join("exact-long.png"),
            ArtifactKind::Screenshot,
            &stopped,
        ),
        "cancelled",
    );
    assert_eq!(snapshot(&s)?, before);
    Ok(())
}
#[test]
fn phase6_rgb_rgba_originals_supported_and_other_color_depths_safely_refused() -> Result<()> {
    let root = tempfile::tempdir()?;
    let mut store = Store::open(AppPaths::isolated(&root.path().join("store")))?;
    let cancel = AtomicBool::new(false);
    for (color, depth) in [
        (png::ColorType::Rgb, png::BitDepth::Eight),
        (png::ColorType::Rgba, png::BitDepth::Eight),
        (png::ColorType::Grayscale, png::BitDepth::Eight),
        (png::ColorType::GrayscaleAlpha, png::BitDepth::Eight),
        (png::ColorType::Rgb, png::BitDepth::Sixteen),
    ] {
        let mut bytes = Vec::new();
        let mut e = png::Encoder::new(&mut bytes, 2, 2);
        e.set_color(color);
        e.set_depth(depth);
        e.add_text_chunk("Fixture".into(), "valid metadata".into())?;
        e.write_header()?.write_image_data(&vec![
            0;
            4 * color.samples()
                * if depth == png::BitDepth::Sixteen {
                    2
                } else {
                    1
                }
        ])?;
        let p = root.path().join(format!("{color:?}-{depth:?}.png"));
        fs::write(&p, &bytes)?;
        let before = snapshot(&store)?;
        if depth == png::BitDepth::Eight
            && matches!(color, png::ColorType::Rgb | png::ColorType::Rgba)
        {
            assert_eq!(images::validate_png(&p, &cancel)?, (2, 2));
            let image = images::ingest(&mut store, &p, ArtifactKind::Screenshot, &cancel)?;
            assert_eq!(image.original.as_str(), hex::encode(Sha256::digest(&bytes)));
            assert_eq!(fs::read(store.artifact_path(&image.original)?)?, bytes);
        } else {
            specific(
                images::ingest(&mut store, &p, ArtifactKind::Screenshot, &cancel),
                "PNG unsupported depth/color",
            );
            assert_eq!(snapshot(&store)?, before);
        }
    }
    Ok(())
}
#[test]
fn phase6_exact_hostile_archives_refuse_specific_causes_atomically_and_valid_roundtrip()
-> Result<()> {
    let root = tempfile::tempdir()?;
    let mut s = Store::open(AppPaths::isolated(&root.path().join("recipient")))?;
    let good = root.path().join("good.zip");
    valid_zip(&good)?;
    let cancel = AtomicBool::new(false);
    let p = s.inspect_archive(&good, &cancel)?.preview;
    let imported = s.import_archive(&good, &p, &cancel)?;
    let baseline = imported.project.active_baseline.unwrap();
    let before = snapshot(&s)?;
    for (name, bytes, cause) in [
        ("long", LONG, "zlib excess decoded bytes"),
        ("coverage", UNKNOWN, "unknown field `script`"),
    ] {
        let path = root.path().join(format!("{name}.zip"));
        fs::write(&path, bytes)?;
        specific(s.inspect_archive(&path, &cancel), cause);
        // Even an old/forged preview cannot bypass reinspection before publication.
        specific(s.import_archive(&path, &p, &cancel), cause);
        assert_eq!(snapshot(&s)?, before);
    }
    let exported = root.path().join("roundtrip.zip");
    s.export_archive(
        &s.export_preview(imported.project.id, baseline)?,
        &exported,
        &cancel,
    )?;
    assert_eq!(snapshot(&s)?, before, "export changed recipient store");
    let mut second = Store::open(AppPaths::isolated(&root.path().join("second")))?;
    let p = second.inspect_archive(&exported, &cancel)?.preview;
    let imported2 = second.import_archive(&exported, &p, &cancel)?;
    let b1 = s.load_baseline(baseline)?;
    let b2 = second.load_baseline(imported2.project.active_baseline.unwrap())?;
    assert!(
        matches!(second.load_inventory(b2.inventory)?.coverage, DiscoveryCoverage::Limited { reason } if reason == "fixture")
    );
    for (a, b) in b1.references.iter().zip(&b2.references) {
        assert_eq!(a.artifact, b.artifact);
        assert_eq!(a.captured_at, b.captured_at);
        assert_eq!(
            fs::read(s.artifact_path(&a.artifact)?)?,
            fs::read(second.artifact_path(&b.artifact)?)?
        );
    }
    Ok(())
}
#[test]
fn phase6_legacy_hash_consistent_malformed_original_cannot_activate_restore_or_export() -> Result<()>
{
    let root = tempfile::tempdir()?;
    let mut s = Store::open(AppPaths::isolated(&root.path().join("store")))?;
    let good = root.path().join("good.zip");
    valid_zip(&good)?;
    let cancel = AtomicBool::new(false);
    let p = s.inspect_archive(&good, &cancel)?.preview;
    let imported = s.import_archive(&good, &p, &cancel)?;
    let baseline = s.load_baseline(imported.project.active_baseline.unwrap())?;
    // Explicitly manufacture a pre-fix/low-level retained local run. This is NOT
    // ingestion acceptance: put_artifact is a generic hash-addressed byte store.
    let bad = entries(LONG)?
        .into_iter()
        .find(|e| e.0 == format!("artifacts/{BAD_HASH}"))
        .unwrap()
        .1;
    let mut wrong_dimensions = Vec::new();
    let mut encoder = png::Encoder::new(&mut wrong_dimensions, 600, 299);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder
        .write_header()?
        .write_image_data(&vec![0; 600 * 299 * 4])?;
    for (index, (bad, cause)) in [
        (bad, "zlib excess decoded bytes"),
        (
            wrong_dimensions,
            "Original screenshot PNG dimensions conflict",
        ),
    ]
    .into_iter()
    .enumerate()
    {
        let a = s.put_artifact(Cursor::new(&bad), ArtifactKind::Screenshot)?;
        let mut run = s.load_run(baseline.references[0].run)?;
        let other = s.load_run(baseline.references[1].run)?;
        run.run.id = RunId::new();
        run.run.intent = RunIntent::NewBaseline;
        for t in &mut run.run.targets {
            if let Some(captured) = other
                .run
                .targets
                .iter()
                .find(|v| v.key == t.key && matches!(v.outcome, TargetOutcome::Captured { .. }))
            {
                *t = captured.clone();
            }
        }
        run.captures.extend(other.captures.clone());
        // Imported source records may use the singular capture slot.
        if let Some(c) = run.capture.take() {
            run.captures.push(c);
        }
        if let Some(c) = other.capture {
            run.captures.push(c);
        }
        run.run.state = JobState::Running;
        run.run.started_at = Some(100);
        run.run.finished_at = None;
        let completed_targets = run.run.targets.clone();
        let completed_captures = run.captures.clone();
        for t in &mut run.run.targets {
            t.outcome = TargetOutcome::Pending;
            t.final_url = None;
        }
        run.captures.clear();
        s.begin_run(&run)?;
        run.run.targets = completed_targets;
        run.captures = completed_captures;
        let c = &mut run.captures[0];
        c.image.original = a.id.clone();
        c.metadata.sha256 = a.id.clone();
        c.metadata.bytes = a.bytes;
        let key = c.metadata.key.clone();
        run.run
            .targets
            .iter_mut()
            .find(|t| t.key == key)
            .unwrap()
            .outcome = TargetOutcome::Captured {
            artifact: a.id.clone(),
            width: 600,
            height: 300,
        };
        run.run.state = JobState::Completed;
        run.run.finished_at = Some(102);
        s.finish_run(&run)?;
        let retained = BaselineVersion {
            id: BaselineId::new(),
            project: imported.project.id,
            parent: None,
            inventory: baseline.inventory,
            created_at: 102,
            references: run
                .all_captures()
                .map(|c| BaselineReference {
                    key: c.metadata.key.clone(),
                    run: run.run.id,
                    artifact: c.image.original.clone(),
                    captured_at: 102,
                })
                .collect(),
            intended_count: 2,
            dom_references: vec![],
            capture_plan: CapturePlan::default(),
            source_run: None,
            intended_targets: vec![],
            explicitly_partial: false,
        };
        s.connection.execute(
            "INSERT INTO baselines VALUES(?1,?2,?3,?4)",
            params![
                retained.id.to_string(),
                retained.project.to_string(),
                run.run.id.to_string(),
                crate::capture::encode(&retained)?
            ],
        )?;
        let before = snapshot(&s)?;
        specific(s.activate_baseline(run.run.id, imported.revision), cause);
        specific(s.restore_baseline(retained.id, imported.revision), cause);
        let preview = s.export_preview(imported.project.id, retained.id)?;
        let destination = root.path().join(format!("refused-{index}.zip"));
        specific(s.export_archive(&preview, &destination, &cancel), cause);
        assert!(!destination.exists());
        assert_eq!(
            snapshot(&s)?,
            before,
            "refused publication changed existing rows/files"
        );
        assert_eq!(
            s.load_project(imported.project.id)?.project.active_baseline,
            Some(baseline.id)
        );
    }
    Ok(())
}
