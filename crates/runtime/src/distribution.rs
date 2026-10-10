// SPDX-License-Identifier: MPL-2.0
//! Configured trust, not a baked-in development/release signing key. Blocking API.
use anyhow::{Context, Result, ensure};
use ed25519_dalek::{Signature, VerifyingKey};
use regressee_core::domain::RuntimeIdentity;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, HashSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Component, Path, PathBuf},
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};

pub const PACK_LIMIT: u64 = 2 * 1024 * 1024 * 1024;
pub const MANIFEST_LIMIT: u64 = 4 * 1024 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Scope {
    DevelopmentLoopback,
    QualifiedPublic,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Distribution {
    pub version: u32,
    pub url: url::Url,
    pub public_key: String,
    pub scope: Scope,
}
impl Distribution {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.version == 1, "unsupported distribution configuration");
        ensure!(
            self.url.username().is_empty() && self.url.password().is_none(),
            "distribution URL contains credentials"
        );
        ensure!(
            self.url.scheme() == "https"
                || (self.scope == Scope::DevelopmentLoopback
                    && self.url.scheme() == "http"
                    && loopback(&self.url)),
            "runtime download requires HTTPS (development HTTP is loopback only)"
        );
        self.key()?;
        Ok(())
    }
    fn key(&self) -> Result<VerifyingKey> {
        let bytes: [u8; 32] = hex::decode(&self.public_key)?
            .try_into()
            .map_err(|_| anyhow::anyhow!("invalid trust key"))?;
        Ok(VerifyingKey::from_bytes(&bytes)?)
    }
}
pub fn loopback(url: &url::Url) -> bool {
    // Literal addresses only: never trust a DNS name to stay on loopback.
    matches!(url.host(), Some(url::Host::Ipv4(ip)) if ip.is_loopback())
        || matches!(url.host(), Some(url::Host::Ipv6(ip)) if ip.is_loopback())
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    pub sha256: String,
    pub bytes: u64,
    pub executable: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Manifest {
    pub version: u32,
    pub id: String,
    pub os: String,
    pub arch: String,
    pub scope: Scope,
    pub support_until: Option<i64>,
    pub identity: RuntimeIdentity,
    pub node: String,
    pub worker: String,
    pub browser_root: String,
    pub files: BTreeMap<String, Entry>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub links: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub modes: BTreeMap<String, u32>,
}
#[derive(Debug, Clone)]
pub struct Installed {
    pub root: PathBuf,
    pub manifest: Manifest,
}
/// Actual installer phases; only Downloading has a byte denominator.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstallProgress {
    Downloading { bytes: u64, total: Option<u64> },
    Verifying,
    Extracting,
    TestingBrowser,
    Ready,
}

/// Absence is not a resolver failure. Check physical ancestors BEFORE deciding
/// that first-use setup is allowed; dangling links and inaccessible paths refuse.
pub fn current_absent(cache: &Path) -> Result<bool> {
    safe_directory(cache)?;
    match fs::symlink_metadata(cache.join("current")) {
        Ok(_) => Ok(false),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(e) => Err(e.into()),
    }
}
pub fn check_cancel(cancel: &AtomicBool) -> Result<()> {
    ensure!(!cancel.load(Ordering::Acquire), "operation cancelled");
    Ok(())
}
pub fn safe_name(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 1024
        && !name
            .chars()
            .any(|c| c.is_control() || "\\:<>\"|?*".contains(c))
        && Path::new(name)
            .components()
            .all(|p| matches!(p, Component::Normal(_)))
        && name.split('/').all(|p| {
            let stem = p.split('.').next().unwrap_or("").to_ascii_uppercase();
            !p.is_empty()
                && p != "."
                && p != ".."
                && !p.ends_with(['.', ' '])
                && !matches!(stem.as_str(), "CON" | "PRN" | "AUX" | "NUL")
                && !(stem.len() == 4
                    && (stem.starts_with("COM") || stem.starts_with("LPT"))
                    && matches!(stem.as_bytes()[3], b'1'..=b'9'))
        })
}
pub fn safe_directory(path: &Path) -> Result<()> {
    let absolute = if path.is_absolute() {
        path.to_owned()
    } else {
        std::env::current_dir()?.join(path)
    };
    ensure!(
        !absolute
            .components()
            .any(|c| matches!(c, Component::ParentDir)),
        "unsafe runtime path"
    );
    for p in absolute.ancestors().collect::<Vec<_>>().into_iter().rev() {
        match fs::symlink_metadata(p) {
            Ok(m) => ensure!(
                m.is_dir() && regressee_core::platform::ordinary(&m),
                "runtime directory is unsafe: {}",
                p.display()
            ),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
pub fn private_directory(path: &Path) -> Result<()> {
    safe_directory(path)?;
    fs::create_dir_all(path)?;
    regressee_core::platform::protect(path)?;
    Ok(())
}
pub fn now() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as i64
}
fn validate(m: &Manifest, d: &Distribution) -> Result<()> {
    ensure!(
        matches!(m.version, 2 | 3)
            && m.os == std::env::consts::OS
            && m.arch == std::env::consts::ARCH
            && m.scope == d.scope,
        "incompatible runtime manifest"
    );
    ensure!(
        safe_name(&m.id)
            && !m.id.contains('/')
            && safe_name(&m.node)
            && safe_name(&m.worker)
            && safe_name(&m.browser_root),
        "unsafe runtime entry point"
    );
    ensure!(
        matches!(m.identity, RuntimeIdentity::Pinned { .. }),
        "unconfigured runtime identity"
    );
    if m.scope == Scope::QualifiedPublic {
        ensure!(
            m.support_until.is_some_and(|until| until > now()),
            "public runtime support expired or not qualified"
        );
    }
    ensure!(
        !m.files.is_empty() && m.files.len() <= 20_000,
        "runtime file count exceeds limits"
    );
    let mut total = 0u64;
    let mut folded = HashSet::new();
    for (name, e) in &m.files {
        ensure!(
            safe_name(name)
                && !matches!(name.as_str(), "manifest.json" | "manifest.sig" | "READY")
                && folded.insert(name.to_lowercase()),
            "unsafe or colliding runtime path"
        );
        ensure!(
            e.sha256.len() == 64
                && e.sha256
                    .bytes()
                    .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)),
            "invalid runtime hash"
        );
        total = total
            .checked_add(e.bytes)
            .context("runtime size overflow")?;
    }
    ensure!(
        total <= PACK_LIMIT
            && m.files.contains_key(&m.node)
            && m.files.contains_key(&m.worker)
            && m.files
                .keys()
                .any(|n| n.starts_with(&format!("{}/", m.browser_root))),
        "runtime budget or entry point missing"
    );
    crate::bundle::validate(m)?;
    #[cfg(windows)]
    ensure!(
        m.links.is_empty(),
        "Windows bundle links require unsupported symlink privileges"
    );
    Ok(())
}
fn signed(bytes: &[u8], signature: &[u8], d: &Distribution) -> Result<Manifest> {
    ensure!(
        bytes.len() as u64 <= MANIFEST_LIMIT,
        "runtime manifest exceeds limit"
    );
    d.key()?
        .verify_strict(bytes, &Signature::from_slice(signature)?)
        .context("runtime signature rejected")?;
    let m: Manifest = serde_json::from_slice(bytes)?;
    validate(&m, d)?;
    Ok(m)
}
pub fn hash(mut reader: impl Read, cancel: &AtomicBool) -> Result<String> {
    let mut h = Sha256::new();
    let mut buf = [0u8; 65536];
    loop {
        check_cancel(cancel)?;
        let n = reader.read(&mut buf)?;
        if n == 0 {
            break;
        }
        h.update(&buf[..n]);
    }
    Ok(hex::encode(h.finalize()))
}
fn regular(path: &Path, cap: u64) -> Result<File> {
    safe_directory(path.parent().context("runtime file parent")?)?;
    let m = fs::symlink_metadata(path)?;
    ensure!(
        m.is_file() && regressee_core::platform::ordinary(&m) && m.len() <= cap,
        "unsafe or oversized runtime file"
    );
    Ok(File::open(path)?)
}
pub fn verify(root: &Path, d: &Distribution, cancel: &AtomicBool) -> Result<Installed> {
    d.validate()?;
    safe_directory(root)?;
    let mut bytes = Vec::new();
    regular(&root.join("manifest.json"), MANIFEST_LIMIT)?.read_to_end(&mut bytes)?;
    let mut sig = Vec::new();
    regular(&root.join("manifest.sig"), 64)?.read_to_end(&mut sig)?;
    let m = signed(&bytes, &sig, d)?;
    fn inventory(root: &Path, dir: &Path, m: &Manifest, cancel: &AtomicBool) -> Result<()> {
        for e in fs::read_dir(dir)? {
            check_cancel(cancel)?;
            let p = e?.path();
            let meta = fs::symlink_metadata(&p)?;
            let name = p
                .strip_prefix(root)?
                .to_str()
                .context("non UTF-8 runtime path")?
                .replace('\\', "/");
            if meta.is_dir() {
                ensure!(
                    regressee_core::platform::ordinary(&meta)
                        && m.files
                            .keys()
                            .chain(m.links.keys())
                            .any(|n| n.starts_with(&format!("{name}/"))),
                    "unlisted runtime directory"
                );
                inventory(root, &p, m, cancel)?;
            } else if let Some(expected) = m.links.get(&name) {
                ensure!(
                    meta.file_type().is_symlink() && fs::read_link(&p)?.to_str() == Some(expected),
                    "Bundle link changed"
                );
            } else {
                ensure!(
                    meta.is_file()
                        && regressee_core::platform::ordinary(&meta)
                        && (m.files.contains_key(&name)
                            || matches!(name.as_str(), "manifest.json" | "manifest.sig" | "READY")),
                    "unlisted or unsafe runtime file"
                );
            }
        }
        Ok(())
    }
    inventory(root, root, &m, cancel)?;
    for (name, e) in &m.files {
        let f = regular(&root.join(name), e.bytes)?;
        ensure!(
            f.metadata()?.len() == e.bytes && hash(f, cancel)? == e.sha256,
            "runtime damaged; repair required"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            ensure!(
                if m.version == 3 {
                    fs::metadata(root.join(name))?.permissions().mode() & 0o7777 == m.modes[name]
                } else {
                    fs::metadata(root.join(name))?.permissions().mode() & 0o111
                        == if e.executable { 0o100 } else { 0 }
                },
                "runtime executable permissions changed"
            );
        }
    }
    for (name, expected) in &m.links {
        let path = root.join(name);
        ensure!(
            fs::symlink_metadata(&path)?.file_type().is_symlink()
                && fs::read_link(path)?.to_str() == Some(expected),
            "Bundle link missing or changed"
        );
    }
    Ok(Installed {
        root: root.to_owned(),
        manifest: m,
    })
}
/// ZIP library deduplicates names; inspect the classic central directory first.
fn raw_directory(pack: &Path) -> Result<()> {
    regressee_core::zip_directory::validate(regular(pack, PACK_LIMIT)?, 20002, safe_name)
}
/// A fresh generation is published only after signature, complete hashes and a
/// sandboxed worker screenshot smoke. Failed repair cannot modify the old pack.
pub fn setup(
    cache: &Path,
    d: &Distribution,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64, Option<u64>),
    smoke: &mut dyn FnMut(&Installed) -> Result<()>,
) -> Result<Installed> {
    setup_network(
        cache,
        d,
        cancel,
        progress,
        smoke,
        &crate::network::NetworkSettings::default(),
    )
}
pub fn setup_network(
    cache: &Path,
    d: &Distribution,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(u64, Option<u64>),
    smoke: &mut dyn FnMut(&Installed) -> Result<()>,
    network: &crate::network::NetworkSettings,
) -> Result<Installed> {
    setup_network_with_progress(
        cache,
        d,
        cancel,
        &mut |phase| {
            if let InstallProgress::Downloading { bytes, total } = phase {
                progress(bytes, total);
            }
        },
        smoke,
        network,
    )
}
pub fn setup_network_with_progress(
    cache: &Path,
    d: &Distribution,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(InstallProgress),
    smoke: &mut dyn FnMut(&Installed) -> Result<()>,
    network: &crate::network::NetworkSettings,
) -> Result<Installed> {
    d.validate()?;
    check_cancel(cancel)?;
    private_directory(cache)?;
    let mut downloaded = tempfile::NamedTempFile::new_in(cache)?;
    progress(InstallProgress::Downloading {
        bytes: 0,
        total: None,
    });
    check_cancel(cancel)?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let result = runtime.block_on(async {
        let client = network.client()?;
        let mut response = await_cancel(client.get(d.url.clone()).send(), cancel)
            .await?
            .error_for_status()?;
        ensure!(
            response.status().is_success(),
            "runtime HTTP redirects are not accepted; configure the exact trusted endpoint"
        );
        let total = response.content_length();
        ensure!(
            total.is_none_or(|n| n <= PACK_LIMIT),
            "runtime download exceeds limit"
        );
        let mut bytes = 0;
        progress(InstallProgress::Downloading { bytes, total });
        while let Some(chunk) = await_cancel(response.chunk(), cancel).await? {
            bytes += chunk.len() as u64;
            ensure!(bytes <= PACK_LIMIT, "runtime download exceeds limit");
            downloaded.write_all(&chunk)?;
            progress(InstallProgress::Downloading { bytes, total });
        }
        ensure!(
            total.is_none_or(|n| n == bytes),
            "incomplete runtime download"
        );
        Ok::<_, anyhow::Error>(())
    });
    runtime.shutdown_timeout(Duration::from_secs(1));
    result?;
    downloaded.as_file().sync_all()?;
    install_with_progress(downloaded.path(), cache, d, cancel, progress, smoke)
}
async fn await_cancel<T>(
    future: impl std::future::Future<Output = std::result::Result<T, reqwest::Error>>,
    cancel: &AtomicBool,
) -> Result<T> {
    tokio::pin!(future);
    loop {
        check_cancel(cancel)?;
        tokio::select! {result=&mut future=>return Ok(result?), _=tokio::time::sleep(Duration::from_millis(50))=>check_cancel(cancel)?}
    }
}
pub fn install(
    pack: &Path,
    cache: &Path,
    d: &Distribution,
    cancel: &AtomicBool,
    smoke: &mut dyn FnMut(&Installed) -> Result<()>,
) -> Result<Installed> {
    install_with_progress(pack, cache, d, cancel, &mut |_| {}, smoke)
}
pub fn install_with_progress(
    pack: &Path,
    cache: &Path,
    d: &Distribution,
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(InstallProgress),
    smoke: &mut dyn FnMut(&Installed) -> Result<()>,
) -> Result<Installed> {
    d.validate()?;
    check_cancel(cancel)?;
    private_directory(cache)?;
    progress(InstallProgress::Verifying);
    check_cancel(cancel)?;
    raw_directory(pack)?;
    match fs::symlink_metadata(cache.join("current")) {
        Ok(m) => ensure!(
            m.is_file() && regressee_core::platform::ordinary(&m),
            "unsafe runtime pointer"
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.into()),
    }
    let mut z = zip::ZipArchive::new(regular(pack, PACK_LIMIT)?)?;
    let mut bytes = Vec::new();
    z.by_name("manifest.json")?
        .take(MANIFEST_LIMIT + 1)
        .read_to_end(&mut bytes)?;
    let mut sig = Vec::new();
    z.by_name("manifest.sig")?.take(65).read_to_end(&mut sig)?;
    let m = signed(&bytes, &sig, d)?;
    ensure!(
        z.len() == m.files.len() + m.links.len() + 2,
        "unlisted runtime entries"
    );
    {
        let required = m.files.values().map(|e| e.bytes).sum::<u64>() + 128 * 1024 * 1024;
        ensure!(
            available_bytes(cache)? >= required,
            "not enough space for browser setup; free cache space and retry (evidence has not changed)"
        );
    }
    let stage = tempfile::Builder::new()
        .prefix("setup-")
        .tempdir_in(cache)?;
    progress(InstallProgress::Extracting);
    check_cancel(cancel)?;
    for i in 0..z.len() {
        check_cancel(cancel)?;
        let mut file = z.by_index(i)?;
        let name = file.name().to_owned();
        if let Some(expected) = m.links.get(&name) {
            ensure!(
                file.is_symlink() && file.size() == expected.len() as u64,
                "Invalid bundle link archive entry"
            );
            let mut value = String::new();
            file.take(1025).read_to_string(&mut value)?;
            ensure!(&value == expected, "Bundle link archive target differs");
            continue;
        }
        ensure!(
            !file.is_symlink() && !file.is_dir(),
            "unsafe runtime archive type"
        );
        ensure!(
            file.unix_mode()
                .is_none_or(|mode| matches!(mode & 0o170000, 0 | 0o100000)),
            "unsafe runtime archive type"
        );
        if matches!(name.as_str(), "manifest.json" | "manifest.sig") {
            continue;
        }
        let e = m.files.get(&name).context("unlisted runtime file")?;
        ensure!(file.size() == e.bytes, "runtime size mismatch");
        let path = stage.path().join(&name);
        private_directory(path.parent().unwrap())?;
        let mut out = File::options().create_new(true).write(true).open(&path)?;
        let mut count = 0;
        let mut digest = Sha256::new();
        let mut buf = [0; 65536];
        loop {
            check_cancel(cancel)?;
            let n = file.read(&mut buf)?;
            if n == 0 {
                break;
            }
            count += n as u64;
            ensure!(count <= e.bytes, "runtime size mismatch");
            out.write_all(&buf[..n])?;
            digest.update(&buf[..n]);
        }
        ensure!(
            count == e.bytes && hex::encode(digest.finalize()) == e.sha256,
            "runtime checksum mismatch"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            out.set_permissions(fs::Permissions::from_mode(if m.version == 3 {
                m.modes[&name]
            } else if e.executable {
                0o700
            } else {
                0o600
            }))?;
        }
        out.sync_all()?;
    }
    // Links are created LAST; no authenticated or unauthenticated link can ever
    // act as an extraction parent. Validation rejects link/leaf parent collisions.
    #[cfg(unix)]
    for (name, value) in &m.links {
        let path = stage.path().join(name);
        private_directory(path.parent().unwrap())?;
        std::os::unix::fs::symlink(value, path)?;
    }
    fs::write(stage.path().join("manifest.json"), bytes)?;
    fs::write(stage.path().join("manifest.sig"), sig)?;
    progress(InstallProgress::Verifying);
    check_cancel(cancel)?;
    let installed = verify(stage.path(), d, cancel)?;
    progress(InstallProgress::TestingBrowser);
    check_cancel(cancel)?;
    smoke(&installed)?;
    check_cancel(cancel)?;
    let ready = stage.path().join("READY");
    let mut f = File::options().create_new(true).write(true).open(ready)?;
    f.write_all(m.id.as_bytes())?;
    f.sync_all()?;
    let root = cache.join(format!("runtime-{}", uuid::Uuid::new_v4()));
    fs::rename(stage.path(), &root)?;
    let mut pointer = tempfile::NamedTempFile::new_in(cache)?;
    pointer.write_all(root.file_name().unwrap().as_encoded_bytes())?;
    pointer.as_file().sync_all()?;
    let current = cache.join("current");
    if let Ok(meta) = fs::symlink_metadata(&current) {
        ensure!(
            meta.is_file() && regressee_core::platform::ordinary(&meta),
            "unsafe runtime pointer"
        );
    }
    pointer.persist(&current).map_err(|e| e.error)?;
    #[cfg(unix)]
    File::open(cache)?.sync_all()?;
    progress(InstallProgress::Ready);
    check_cancel(cancel)?;
    Ok(Installed { root, manifest: m })
}
pub fn resolve(cache: &Path, d: &Distribution, cancel: &AtomicBool) -> Result<Installed> {
    let mut name = String::new();
    regular(&cache.join("current"), 128)?.read_to_string(&mut name)?;
    ensure!(
        safe_name(&name) && !name.contains('/') && name.starts_with("runtime-"),
        "invalid runtime pointer"
    );
    let i = verify(&cache.join(name), d, cancel)?;
    let mut ready = String::new();
    regular(&i.root.join("READY"), 128)?.read_to_string(&mut ready)?;
    ensure!(
        ready == i.manifest.id,
        "runtime smoke qualification missing"
    );
    Ok(i)
}
/// Local preflight only; unknown platform support fails closed.
pub fn available_bytes(path: &Path) -> Result<u64> {
    regressee_core::platform::available_bytes(path)
}
