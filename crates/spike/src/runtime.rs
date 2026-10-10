// SPDX-License-Identifier: MPL-2.0
//! Local signed-pack prototype. The development signing key is NOT release trust.
use anyhow::{Context, Result, bail, ensure};
use ed25519_dalek::{Signature, Signer, SigningKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::{Read, Seek, SeekFrom, Write},
    path::{Component, Path},
    sync::atomic::AtomicBool,
};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

const PACK_LIMIT: u64 = 2 * 1024 * 1024 * 1024;
const DEV_KEY: [u8; 32] = [17; 32];
pub const RUNTIME_ID: &str = "node22.20.0-pw1.56.1-protocol1";

#[derive(Serialize, Deserialize)]
struct Manifest {
    version: u32,
    runtime: String,
    os: String,
    arch: String,
    files: BTreeMap<String, Entry>,
}
#[derive(Serialize, Deserialize)]
struct Entry {
    sha256: String,
    bytes: u64,
    executable: bool,
}

fn hash_file(path: &Path) -> Result<String> {
    hash_file_cancellable(path, &AtomicBool::new(false))
}
fn hash_file_cancellable(path: &Path, cancel: &AtomicBool) -> Result<String> {
    hash_reader(File::open(path)?, cancel)
}
fn hash_reader(mut f: impl Read, cancel: &AtomicBool) -> Result<String> {
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        crate::cancel::check(cancel, "runtime verification")?;
        let n = f.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    crate::cancel::check(cancel, "runtime verification")?;
    Ok(hex::encode(h.finalize()))
}

pub fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('\\')
        && !name.contains(':')
        && Path::new(name)
            .components()
            .all(|c| matches!(c, Component::Normal(_)))
        && !name
            .split('/')
            .any(|c| c.is_empty() || c == "." || c == "..")
}

fn walk(root: &Path, path: &Path, manifest: &mut Manifest) -> Result<()> {
    for item in fs::read_dir(path)? {
        let path = item?.path();
        let meta = fs::symlink_metadata(&path)?;
        // Node's npm bin links aren't needed by installed end users.
        if meta.file_type().is_symlink() {
            continue;
        }
        if meta.is_dir() {
            walk(root, &path, manifest)?;
        } else if meta.is_file() {
            let name = path
                .strip_prefix(root)?
                .to_str()
                .context("non-UTF8 path")?
                .replace('\\', "/");
            ensure!(safe_name(&name), "unsafe pack path");
            #[cfg(unix)]
            let executable = {
                use std::os::unix::fs::PermissionsExt;
                meta.permissions().mode() & 0o111 != 0
            };
            #[cfg(not(unix))]
            let executable = false;
            manifest.files.insert(
                name,
                Entry {
                    sha256: hash_file(&path)?,
                    bytes: meta.len(),
                    executable,
                },
            );
        }
    }
    Ok(())
}

pub fn pack(source: &Path, out: &Path) -> Result<()> {
    let mut manifest = Manifest {
        version: 1,
        runtime: RUNTIME_ID.into(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        files: BTreeMap::new(),
    };
    walk(source, source, &mut manifest)?;
    ensure!(!manifest.files.is_empty(), "empty runtime source");
    let bytes = serde_json::to_vec(&manifest)?;
    let signature = SigningKey::from_bytes(&DEV_KEY).sign(&bytes);
    let partial = out.with_extension("partial");
    let mut z = ZipWriter::new(File::create(&partial)?);
    let opts = SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated)
        .compression_level(Some(1));
    z.start_file("manifest.json", opts)?;
    z.write_all(&bytes)?;
    z.start_file("manifest.sig", opts)?;
    z.write_all(&signature.to_bytes())?;
    for (name, entry) in &manifest.files {
        z.start_file(
            name,
            opts.unix_permissions(if entry.executable { 0o700 } else { 0o600 }),
        )?;
        std::io::copy(&mut File::open(source.join(name))?, &mut z)?;
    }
    z.finish()?.sync_all()?;
    fs::rename(partial, out)?;
    println!(
        "Created local DEVELOPMENT-signed pack: {} files",
        manifest.files.len()
    );
    Ok(())
}

fn read_manifest(z: &mut ZipArchive<File>) -> Result<(Manifest, Vec<u8>)> {
    let mut bytes = Vec::new();
    z.by_name("manifest.json")?
        .take(4 * 1024 * 1024 + 1)
        .read_to_end(&mut bytes)?;
    ensure!(bytes.len() <= 4 * 1024 * 1024, "manifest too large");
    let mut sig = Vec::new();
    z.by_name("manifest.sig")?.take(65).read_to_end(&mut sig)?;
    let sig = Signature::from_slice(&sig)?;
    SigningKey::from_bytes(&DEV_KEY)
        .verifying_key()
        .verify_strict(&bytes, &sig)
        .context("pack signature rejected")?;
    let m: Manifest = serde_json::from_slice(&bytes)?;
    validate_manifest(&m)?;
    Ok((m, bytes))
}

fn validate_manifest(m: &Manifest) -> Result<()> {
    ensure!(
        m.version == 1 && m.runtime == RUNTIME_ID,
        "incompatible pack version"
    );
    ensure!(
        m.os == std::env::consts::OS && m.arch == std::env::consts::ARCH,
        "pack is for another computer architecture/OS"
    );
    ensure!(m.files.len() <= 20_000, "too many files");
    ensure!(!m.files.is_empty(), "empty runtime manifest");
    for (name, entry) in &m.files {
        ensure!(
            safe_name(name) && !matches!(name.as_str(), "manifest.json" | "manifest.sig" | "READY"),
            "unsafe manifest path"
        );
        ensure!(
            entry.sha256.len() == 64
                && entry
                    .sha256
                    .bytes()
                    .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase()),
            "invalid manifest checksum"
        );
    }
    let total = m
        .files
        .values()
        .try_fold(0u64, |n, e| n.checked_add(e.bytes))
        .context("size overflow")?;
    ensure!(total <= PACK_LIMIT, "pack exceeds 2 GiB extraction budget");
    Ok(())
}

pub fn verify(root: &Path) -> Result<()> {
    verify_cancellable(root, &AtomicBool::new(false))
}

fn no_links(path: &Path) -> Result<()> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    let mut current = std::path::PathBuf::new();
    for part in absolute.components() {
        current.push(part);
        ensure!(
            !fs::symlink_metadata(&current)?.file_type().is_symlink(),
            "unexpected runtime link: {}",
            current.display()
        );
    }
    Ok(())
}

fn installed_inventory(
    root: &Path,
    path: &Path,
    manifest: &Manifest,
    cancel: &AtomicBool,
) -> Result<()> {
    for entry in fs::read_dir(path)? {
        crate::cancel::check(cancel, "runtime verification")?;
        let path = entry?.path();
        let meta = fs::symlink_metadata(&path)?;
        ensure!(!meta.file_type().is_symlink(), "unexpected runtime link");
        let name = path
            .strip_prefix(root)?
            .to_str()
            .context("non-UTF8 installed path")?
            .replace('\\', "/");
        if meta.is_dir() {
            ensure!(
                manifest
                    .files
                    .keys()
                    .any(|n| n.starts_with(&format!("{name}/"))),
                "unexpected installed directory"
            );
            installed_inventory(root, &path, manifest, cancel)?;
        } else {
            ensure!(meta.is_file(), "unexpected installed file type");
            ensure!(
                manifest.files.contains_key(&name)
                    || matches!(name.as_str(), "manifest.json" | "manifest.sig" | "READY"),
                "unexpected installed file: {name}"
            );
        }
    }
    Ok(())
}

pub fn verify_cancellable(root: &Path, cancel: &AtomicBool) -> Result<()> {
    crate::cancel::check(cancel, "runtime verification")?;
    no_links(&root.join("manifest.json"))?;
    no_links(&root.join("manifest.sig"))?;
    ensure!(
        fs::metadata(root.join("manifest.json"))?.is_file()
            && fs::metadata(root.join("manifest.json"))?.len() <= 4 * 1024 * 1024,
        "manifest too large or not a file"
    );
    ensure!(
        fs::metadata(root.join("manifest.sig"))?.is_file()
            && fs::metadata(root.join("manifest.sig"))?.len() == 64,
        "invalid manifest signature size"
    );
    let bytes = fs::read(root.join("manifest.json"))?;
    let sig = Signature::from_slice(&fs::read(root.join("manifest.sig"))?)?;
    SigningKey::from_bytes(&DEV_KEY)
        .verifying_key()
        .verify_strict(&bytes, &sig)
        .context("runtime signature rejected")?;
    let manifest: Manifest = serde_json::from_slice(&bytes)?;
    validate_manifest(&manifest)?;
    installed_inventory(root, root, &manifest, cancel)?;
    if root.join("READY").exists() {
        ensure!(
            fs::metadata(root.join("READY"))?.len() == RUNTIME_ID.len() as u64,
            "invalid runtime ready marker"
        );
        ensure!(
            fs::read(root.join("READY"))? == RUNTIME_ID.as_bytes(),
            "invalid runtime ready marker"
        );
    }
    for (name, entry) in manifest.files {
        crate::cancel::check(cancel, "runtime verification")?;
        let path = root.join(&name);
        no_links(&path)?;
        ensure!(
            !fs::symlink_metadata(&path)?.file_type().is_symlink(),
            "unexpected runtime link"
        );
        ensure!(
            fs::metadata(&path)?.is_file()
                && fs::metadata(&path)?.len() == entry.bytes
                && hash_file_cancellable(&path, cancel)? == entry.sha256,
            "runtime damaged; repair required"
        );
    }
    Ok(())
}

/// zip 2.4.2 deduplicates names before exposing by_index. Inspect the raw
/// classic central directory first. ZIP64/multi-disk packs are not supported.
fn validate_raw_directory(pack: &Path, cancel: &AtomicBool) -> Result<()> {
    let mut f = File::open(pack)?;
    let size = f.metadata()?.len();
    ensure!(size <= PACK_LIMIT, "compressed pack exceeds 2 GiB budget");
    let tail_len = size.min(65557) as usize;
    f.seek(SeekFrom::End(-(tail_len as i64)))?;
    let mut tail = vec![0; tail_len];
    f.read_exact(&mut tail)?;
    let end = (0..tail_len.saturating_sub(21))
        .rev()
        .find(|&i| {
            tail[i..i + 4] == *b"PK\x05\x06"
                && i + 22 + u16::from_le_bytes([tail[i + 20], tail[i + 21]]) as usize == tail_len
        })
        .context("invalid ZIP end directory")?;
    let u16at = |i| u16::from_le_bytes([tail[end + i], tail[end + i + 1]]);
    let u32at = |i| u32::from_le_bytes(tail[end + i..end + i + 4].try_into().unwrap());
    ensure!(
        u16at(4) == 0 && u16at(6) == 0 && u16at(8) == u16at(10) && u16at(10) != u16::MAX,
        "unsupported ZIP64 or multi-disk pack"
    );
    let count = u16at(10);
    ensure!(count <= 20002, "too many raw pack entries");
    let (length, offset) = (u32at(12) as u64, u32at(16) as u64);
    ensure!(
        offset + length == size - tail_len as u64 + end as u64,
        "invalid ZIP directory bounds"
    );
    f.seek(SeekFrom::Start(offset))?;
    let mut names = std::collections::HashSet::new();
    for _ in 0..count {
        crate::cancel::check(cancel, "runtime setup")?;
        let mut header = [0; 46];
        f.read_exact(&mut header)?;
        ensure!(
            &header[..4] == b"PK\x01\x02",
            "invalid raw ZIP directory entry"
        );
        let field = |i| u16::from_le_bytes([header[i], header[i + 1]]);
        let mut name = vec![0; field(28) as usize];
        f.read_exact(&mut name)?;
        let name = std::str::from_utf8(&name).context("non-UTF8 pack entry")?;
        ensure!(
            names.insert(name.to_owned()),
            "duplicate raw pack entry: {name}"
        );
        ensure!(safe_name(name), "unsafe archive entry");
        f.seek(SeekFrom::Current(
            i64::from(field(30)) + i64::from(field(32)),
        ))?;
        ensure!(
            f.stream_position()? <= offset + length,
            "invalid ZIP directory length"
        );
    }
    ensure!(
        f.stream_position()? == offset + length,
        "invalid ZIP directory length"
    );
    Ok(())
}

pub fn install(pack: &Path, root: &Path) -> Result<()> {
    install_cancellable(
        pack,
        root,
        std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
    )
}

pub fn install_cancellable(
    pack: &Path,
    root: &Path,
    cancel: std::sync::Arc<std::sync::atomic::AtomicBool>,
) -> Result<()> {
    crate::cancel::check(&cancel, "runtime setup")?;
    fs::create_dir_all(root.parent().context("runtime parent")?)?;
    no_links(root.parent().unwrap())?;
    if fs::symlink_metadata(root).is_ok() {
        no_links(root)?;
    }
    no_links(pack)?;
    let lock = root.with_extension("lock");
    let _lock_file = File::options()
        .write(true)
        .create_new(true)
        .open(&lock)
        .context("setup already running; stale lock requires explicit recovery")?;
    struct Lock(std::path::PathBuf);
    impl Drop for Lock {
        fn drop(&mut self) {
            let _ = fs::remove_file(&self.0);
        }
    }
    let _guard = Lock(lock);
    let stage = root.with_extension("stage");
    // Only our named staging cache is disposable; never evidence or unknown paths.
    if stage.exists() {
        bail!("interrupted stage exists; inspect/remove .runtime/installed.stage before retry");
    }
    let result = (|| -> Result<()> {
        crate::cancel::check(&cancel, "runtime setup")?;
        validate_raw_directory(pack, &cancel)?;
        let mut z = ZipArchive::new(File::open(pack)?)?;
        let (manifest, bytes) = read_manifest(&mut z)?;
        #[cfg(unix)]
        {
            let parent =
                std::ffi::CString::new(root.parent().unwrap().as_os_str().as_encoded_bytes())?;
            let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
            ensure!(
                unsafe { libc::statvfs(parent.as_ptr(), stat.as_mut_ptr()) } == 0,
                "cannot check available runtime disk space"
            );
            let stat = unsafe { stat.assume_init() };
            let required: u64 =
                manifest.files.values().map(|e| e.bytes).sum::<u64>() + 128 * 1024 * 1024;
            ensure!(
                u128::from(stat.f_bavail) * u128::from(stat.f_frsize) >= u128::from(required),
                "not enough space for setup; free runtime cache space and retry"
            );
        }
        ensure!(
            z.len() == manifest.files.len() + 2,
            "unexpected pack entries"
        );
        fs::create_dir(&stage)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&stage, fs::Permissions::from_mode(0o700))?;
        }
        let mut seen = std::collections::HashSet::new();
        for i in 0..z.len() {
            ensure!(
                !cancel.load(std::sync::atomic::Ordering::Relaxed),
                "runtime setup cancelled"
            );
            let mut file = z.by_index(i)?;
            let name = file.name().to_owned();
            ensure!(seen.insert(name.clone()), "duplicate pack entry");
            ensure!(
                safe_name(&name) && !file.is_symlink() && !file.is_dir(),
                "unsafe archive entry"
            );
            if name == "manifest.json" || name == "manifest.sig" {
                continue;
            }
            let e = manifest.files.get(&name).context("unlisted pack file")?;
            ensure!(file.size() == e.bytes, "pack size mismatch");
            let dest = stage.join(&name);
            fs::create_dir_all(dest.parent().unwrap())?;
            let mut out = File::options().write(true).create_new(true).open(&dest)?;
            let mut count = 0u64;
            let mut digest = Sha256::new();
            let mut buffer = [0u8; 65536];
            loop {
                ensure!(
                    !cancel.load(std::sync::atomic::Ordering::Relaxed),
                    "runtime setup cancelled"
                );
                let n = file.read(&mut buffer)?;
                if n == 0 {
                    break;
                }
                count += n as u64;
                ensure!(count <= e.bytes, "pack size mismatch");
                out.write_all(&buffer[..n])?;
                digest.update(&buffer[..n]);
            }
            ensure!(
                count == e.bytes && hex::encode(digest.finalize()) == e.sha256,
                "pack checksum mismatch"
            );
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                out.set_permissions(fs::Permissions::from_mode(if e.executable {
                    0o700
                } else {
                    0o600
                }))?;
            }
        }
        fs::write(stage.join("manifest.json"), bytes)?;
        let mut sig = Vec::new();
        z.by_name("manifest.sig")?.read_to_end(&mut sig)?;
        fs::write(stage.join("manifest.sig"), sig)?;
        verify_cancellable(&stage, &cancel)?;
        crate::cancel::check(&cancel, "runtime setup")?;
        let mut worker = crate::supervisor::Worker::spawn_unverified(&stage)?;
        worker.cancelled = cancel.clone();
        worker.request("handshake", serde_json::json!({}))?;
        worker.request("smoke", serde_json::json!({}))?;
        worker.shutdown()?;
        crate::cancel::check(&cancel, "runtime setup")?;
        fs::write(stage.join("READY"), RUNTIME_ID)?;
        let old = root.with_extension("previous");
        ensure!(
            !old.exists(),
            "previous runtime exists; inspect before repair"
        );
        if root.exists() {
            fs::rename(root, &old)?;
        }
        if let Err(e) = fs::rename(&stage, root) {
            if old.exists() {
                let _ = fs::rename(&old, root);
            }
            return Err(e.into());
        }
        // Only the runtime we just replaced is removed, never retained screenshots.
        if old.exists() {
            fs::remove_dir_all(old)?;
        }
        println!("Private runtime ready; sandboxed screenshot smoke passed");
        Ok(())
    })();
    if result.is_err() && stage.exists() {
        let _ = fs::remove_dir_all(stage);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    fn reject_install(pack: &Path, root: &Path, reason: &str) {
        let before = crate::supervisor::SPAWN_ATTEMPTS.load(std::sync::atomic::Ordering::Relaxed);
        assert_eq!(install(pack, root).unwrap_err().to_string(), reason);
        assert_eq!(
            crate::supervisor::SPAWN_ATTEMPTS.load(std::sync::atomic::Ordering::Relaxed),
            before,
            "rejected input reached worker spawn"
        );
    }
    fn reject_launch(root: &Path, reason: &str) {
        let before = crate::supervisor::SPAWN_ATTEMPTS.load(std::sync::atomic::Ordering::Relaxed);
        let error = crate::supervisor::Worker::spawn(root)
            .err()
            .expect("invalid installed runtime must not launch");
        assert_eq!(error.to_string(), reason);
        assert_eq!(
            crate::supervisor::SPAWN_ATTEMPTS.load(std::sync::atomic::Ordering::Relaxed),
            before,
            "installed rejection reached worker spawn"
        );
    }
    #[test]
    fn hashing_observes_mid_stream_cancellation() {
        struct Reader<'a>(&'a AtomicBool);
        impl Read for Reader<'_> {
            fn read(&mut self, b: &mut [u8]) -> std::io::Result<usize> {
                self.0.store(true, std::sync::atomic::Ordering::Relaxed);
                b.fill(42);
                Ok(b.len())
            }
        }
        let cancel = AtomicBool::new(false);
        assert_eq!(
            hash_reader(Reader(&cancel), &cancel)
                .unwrap_err()
                .to_string(),
            "runtime verification cancelled"
        );
    }
    fn fixture_manifest() -> Manifest {
        Manifest {
            version: 1,
            runtime: RUNTIME_ID.into(),
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            files: BTreeMap::from([(
                "worker/file".into(),
                Entry {
                    sha256: hex::encode(Sha256::digest(b"data")),
                    bytes: 4,
                    executable: false,
                },
            )]),
        }
    }
    fn signed_tree(root: &Path, m: &Manifest) -> Result<()> {
        fs::create_dir_all(root.join("worker"))?;
        fs::write(root.join("worker/file"), b"data")?;
        let bytes = serde_json::to_vec(m)?;
        fs::write(
            root.join("manifest.sig"),
            SigningKey::from_bytes(&DEV_KEY).sign(&bytes).to_bytes(),
        )?;
        fs::write(root.join("manifest.json"), bytes)?;
        Ok(())
    }
    #[test]
    fn installed_and_pack_invariants() -> Result<()> {
        let scratch = tempfile::tempdir()?;
        let parent = scratch.path();
        for (name, reason) in [
            ("schema", "incompatible pack version"),
            ("platform", "pack is for another computer architecture/OS"),
            ("budget", "pack exceeds 2 GiB extraction budget"),
            ("path", "unsafe manifest path"),
            ("count", "too many files"),
        ] {
            let mut m = fixture_manifest();
            match name {
                "schema" => m.version = 99,
                "platform" => m.os = "other".into(),
                "budget" => m.files.get_mut("worker/file").unwrap().bytes = PACK_LIMIT + 1,
                "path" => {
                    let e = m.files.remove("worker/file").unwrap();
                    m.files.insert("../outside".into(), e);
                }
                "count" => {
                    for i in 0..20001 {
                        m.files.insert(
                            format!("worker/{i}"),
                            Entry {
                                sha256: "00".repeat(32),
                                bytes: 0,
                                executable: false,
                            },
                        );
                    }
                }
                _ => unreachable!(),
            }
            let root = parent.join(name);
            signed_tree(&root, &m)?;
            assert_eq!(verify(&root).unwrap_err().to_string(), reason);
            reject_launch(&root, reason);
            let pack = parent.join(format!("{name}.zip"));
            let mut z = ZipWriter::new(File::create(&pack)?);
            let opts = SimpleFileOptions::default();
            let bytes = serde_json::to_vec(&m)?;
            for (name, data) in [
                ("manifest.json", bytes.clone()),
                (
                    "manifest.sig",
                    SigningKey::from_bytes(&DEV_KEY)
                        .sign(&bytes)
                        .to_bytes()
                        .to_vec(),
                ),
                ("worker/file", b"data".to_vec()),
            ] {
                z.start_file(name, opts)?;
                z.write_all(&data)?;
            }
            z.finish()?;
            let target = parent.join(format!("install-{name}"));
            fs::create_dir_all(&target)?;
            fs::write(target.join("sentinel"), b"keep")?;
            // Minimal pack cannot launch Node: the exact early cause rules out a
            // later spawn error masquerading as manifest rejection.
            reject_install(&pack, &target, reason);
            assert_eq!(fs::read(target.join("sentinel"))?, b"keep");
            assert!(!target.with_extension("stage").exists());
            assert!(!target.with_extension("lock").exists());
        }
        Ok(())
    }
    #[cfg(unix)]
    #[test]
    fn rejects_installed_links_and_unexpected_files() -> Result<()> {
        use std::os::unix::fs::symlink;
        let scratch = tempfile::tempdir()?;
        let parent = scratch.path();
        for name in ["worker", "manifest.json", "manifest.sig", "READY"] {
            let root = parent.join(name.replace('.', "-"));
            signed_tree(&root, &fixture_manifest())?;
            let path = root.join(name);
            let outside = parent.join(format!("outside-{}", name.replace('.', "-")));
            if path.exists() {
                fs::rename(&path, &outside)?;
            } else {
                fs::write(&outside, RUNTIME_ID)?;
            }
            symlink(&outside, &path)?;
            let error = verify(&root).unwrap_err().to_string();
            assert!(
                error.starts_with("unexpected runtime link"),
                "{name}: {error}"
            );
            reject_launch(&root, &error);
        }
        let root = parent.join("unexpected");
        signed_tree(&root, &fixture_manifest())?;
        fs::write(root.join("unlisted"), b"data")?;
        assert_eq!(
            verify(&root).unwrap_err().to_string(),
            "unexpected installed file: unlisted"
        );
        reject_launch(&root, "unexpected installed file: unlisted");
        let cancel = AtomicBool::new(true);
        assert_eq!(
            verify_cancellable(&root, &cancel).unwrap_err().to_string(),
            "runtime verification cancelled"
        );
        Ok(())
    }
    #[test]
    fn raw_duplicate_rejected_before_extraction_or_launch() -> Result<()> {
        let scratch = tempfile::tempdir()?;
        let parent = scratch.path();
        let pack = parent.join("duplicate.zip");
        let mut z = ZipWriter::new(File::create(&pack)?);
        let opts = SimpleFileOptions::default();
        let m = fixture_manifest();
        let bytes = serde_json::to_vec(&m)?;
        for (name, data) in [
            ("manifest.json", bytes.clone()),
            (
                "manifest.sig",
                SigningKey::from_bytes(&DEV_KEY)
                    .sign(&bytes)
                    .to_bytes()
                    .to_vec(),
            ),
            ("worker/file", b"data".to_vec()),
            ("worker/dupe", b"data".to_vec()),
        ] {
            z.start_file(name, opts)?;
            z.write_all(&data)?;
        }
        z.finish()?;
        let mut bytes = fs::read(&pack)?;
        for i in 0..bytes.len() - 10 {
            if &bytes[i..i + 11] == b"worker/dupe" {
                bytes[i..i + 11].copy_from_slice(b"worker/file");
            }
        }
        fs::write(&pack, bytes)?;
        let root = parent.join("installed");
        fs::create_dir(&root)?;
        fs::write(root.join("sentinel"), b"keep")?;
        reject_install(&pack, &root, "duplicate raw pack entry: worker/file");
        assert_eq!(fs::read(root.join("sentinel"))?, b"keep");
        assert!(!root.with_extension("stage").exists());
        assert!(!root.with_extension("lock").exists());
        Ok(())
    }
    #[test]
    fn signature_and_archive_link_rejections_are_not_spawn_failures() -> Result<()> {
        let scratch = tempfile::tempdir()?;
        let parent = scratch.path();
        for (name, reason) in [
            ("signature", "pack signature rejected"),
            ("archive-link", "unsafe archive entry"),
        ] {
            let pack = parent.join(format!("{name}.zip"));
            let mut z = ZipWriter::new(File::create(&pack)?);
            let opts = SimpleFileOptions::default();
            let bytes = serde_json::to_vec(&fixture_manifest())?;
            z.start_file("manifest.json", opts)?;
            z.write_all(&bytes)?;
            z.start_file("manifest.sig", opts)?;
            z.write_all(&if name == "signature" {
                [0; 64]
            } else {
                SigningKey::from_bytes(&DEV_KEY).sign(&bytes).to_bytes()
            })?;
            if name == "archive-link" {
                z.add_symlink("worker/file", "outside", opts)?;
            } else {
                z.start_file("worker/file", opts)?;
                z.write_all(b"data")?;
            }
            z.finish()?;
            let root = parent.join(name);
            fs::create_dir(&root)?;
            fs::write(root.join("sentinel"), b"keep")?;
            reject_install(&pack, &root, reason);
            assert_eq!(fs::read(root.join("sentinel"))?, b"keep");
            assert!(!root.with_extension("stage").exists());
            assert!(!root.with_extension("lock").exists());
        }
        Ok(())
    }
    #[test]
    fn rejects_traversal() {
        for n in ["../x", "/x", "C:x", "a\\b", "a/../b", "a//b", "a/./b", ""] {
            assert!(!safe_name(n), "{n}");
        }
        assert!(safe_name("worker/src/index.mjs"));
    }
    #[test]
    fn tamper_rejected() {
        let key = SigningKey::from_bytes(&DEV_KEY);
        let sig = key.sign(b"good");
        assert!(key.verifying_key().verify_strict(b"bad", &sig).is_err());
    }
    #[test]
    fn corrupt_archive_preserves_existing_runtime() -> Result<()> {
        let scratch = tempfile::tempdir()?;
        let parent = scratch.path();
        let root = parent.join("installed");
        fs::create_dir_all(&root)?;
        fs::write(root.join("evidence"), b"keep")?;
        let mut m = Manifest {
            version: 1,
            runtime: RUNTIME_ID.into(),
            os: std::env::consts::OS.into(),
            arch: std::env::consts::ARCH.into(),
            files: BTreeMap::new(),
        };
        m.files.insert(
            "worker/bad".into(),
            Entry {
                sha256: "00".repeat(32),
                bytes: 3,
                executable: false,
            },
        );
        let bytes = serde_json::to_vec(&m)?;
        let sig = SigningKey::from_bytes(&DEV_KEY).sign(&bytes);
        let pack = parent.join("corrupt.zip");
        let mut z = ZipWriter::new(File::create(&pack)?);
        let opts = SimpleFileOptions::default();
        z.start_file("manifest.json", opts)?;
        z.write_all(&bytes)?;
        z.start_file("manifest.sig", opts)?;
        z.write_all(&sig.to_bytes())?;
        z.start_file("worker/bad", opts)?;
        z.write_all(b"bad")?;
        z.finish()?;
        reject_install(&pack, &root, "pack checksum mismatch");
        assert_eq!(fs::read(root.join("evidence"))?, b"keep");
        assert!(!root.with_extension("stage").exists());
        assert!(!root.with_extension("lock").exists());
        Ok(())
    }
}
