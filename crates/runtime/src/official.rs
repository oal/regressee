// SPDX-License-Identifier: MPL-2.0
//! App-owned official-source pins. This is NOT signed-pack verification or a
//! QualifiedPublic claim. Callers must supply the catalog embedded in the app,
//! never a manifest discovered in the cache or an environment configuration.
use crate::{
    distribution::{self, InstallProgress, Installed, Manifest, PACK_LIMIT, Scope},
    network::NetworkSettings,
};
use anyhow::{Context, Result, ensure};
use regressee_core::domain::RuntimeIdentity;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{Cursor, Read, Seek, Write},
    path::Path,
    sync::atomic::AtomicBool,
    time::Duration,
};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Catalog {
    pub manifest: Manifest,
    pub components: Vec<Component>,
    pub bundled_sha256: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Component {
    pub url: String,
    pub sha256: String,
    pub bytes: u64,
    pub format: ArchiveFormat,
    pub prefix: String,
    pub destination: String,
    pub only: Option<String>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArchiveFormat {
    Zip,
    TarGz,
}

fn digest_valid(s: &str) -> bool {
    s.len() == 64
        && s.bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn validate_catalog(c: &Catalog) -> Result<()> {
    let m = &c.manifest;
    ensure!(
        m.version == 3
            && m.scope == Scope::OfficialPinned
            && m.os == std::env::consts::OS
            && m.arch == std::env::consts::ARCH,
        "incompatible official runtime catalog"
    );
    ensure!(
        distribution::safe_name(&m.id)
            && !m.id.contains('/')
            && m.id.len() <= 128
            && distribution::safe_name(&m.node)
            && distribution::safe_name(&m.worker)
            && distribution::safe_name(&m.browser_root)
            && m.support_until
                .is_some_and(|until| until > distribution::now()),
        "unsafe or expired official runtime catalog"
    );
    match &m.identity {
        RuntimeIdentity::Pinned {
            node,
            worker,
            playwright,
            browser,
        } => {
            ensure!(
                [node, worker, playwright, browser].iter().all(|s| {
                    !s.is_empty() && s.len() <= 4096 && !s.chars().any(char::is_control)
                }),
                "incomplete official runtime identity"
            );
        }
        _ => anyhow::bail!("unconfigured official runtime identity"),
    }
    ensure!(
        !m.files.is_empty() && m.files.len() <= 20_000,
        "official file budget exceeded"
    );
    let mut unpacked = 0u64;
    for e in m.files.values() {
        ensure!(digest_valid(&e.sha256), "invalid official file digest");
        unpacked = unpacked
            .checked_add(e.bytes)
            .context("official size overflow")?;
    }
    ensure!(
        unpacked <= PACK_LIMIT
            && m.files.contains_key(&m.node)
            && m.files.contains_key(&m.worker)
            && m.files[&m.node].executable
            && m.files
                .keys()
                .any(|n| n.starts_with(&(m.browser_root.clone() + "/"))),
        "official runtime budget or entry point missing"
    );
    crate::bundle::validate(m)?;
    #[cfg(windows)]
    ensure!(m.links.is_empty(), "Windows bundle links unsupported");
    ensure!(
        digest_valid(&c.bundled_sha256),
        "invalid bundled library digest"
    );
    ensure!(
        !c.components.is_empty() && c.components.len() <= 8,
        "official component budget exceeded"
    );
    let mut downloaded = 0u64;
    let mut destinations: Vec<&str> = Vec::new();
    for a in &c.components {
        let u = url::Url::parse(&a.url).context("invalid official source URL")?;
        ensure!(
            u.scheme() == "https"
                && u.username().is_empty()
                && u.password().is_none()
                && u.query().is_none()
                && u.fragment().is_none()
                && u.port().is_none()
                && matches!(
                    u.host_str(),
                    Some(
                        "nodejs.org"
                            | "storage.googleapis.com"
                            | "cdn.playwright.dev"
                            | "playwright.download.prss.microsoft.com"
                    )
                ),
            "unapproved official source URL"
        );
        ensure!(
            digest_valid(&a.sha256) && a.bytes > 0 && a.bytes <= PACK_LIMIT,
            "invalid official archive pin"
        );
        downloaded = downloaded
            .checked_add(a.bytes)
            .context("official download size overflow")?;
        ensure!(
            distribution::safe_name(&a.prefix)
                && distribution::safe_name(&a.destination)
                && a.only.as_ref().is_none_or(|n| distribution::safe_name(n)),
            "unsafe official archive mapping"
        );
        ensure!(
            destinations.iter().all(|d| {
                let d = d.to_lowercase();
                let n = a.destination.to_lowercase();
                d != n && !d.starts_with(&(n.clone() + "/")) && !n.starts_with(&(d + "/"))
            }),
            "overlapping official archive destinations"
        );
        destinations.push(&a.destination);
        if let Some(only) = &a.only {
            ensure!(
                format!("{}/{only}", a.destination) == m.node,
                "selective official archive must provide Node only"
            );
        } else {
            ensure!(
                a.format == ArchiveFormat::Zip,
                "TAR extraction requires a selected Node file"
            );
        }
    }
    ensure!(
        downloaded <= PACK_LIMIT,
        "official download budget exceeded"
    );
    ensure!(
        m.files
            .keys()
            .chain(m.links.keys())
            .all(|name| !dependency_marker(c, name)),
        "host-validation markers must not be fabricated in the catalog"
    );
    Ok(())
}
pub fn validate(c: &Catalog, bundled: &[u8]) -> Result<()> {
    validate_catalog(c)?;
    ensure!(
        bundled.len() as u64 <= PACK_LIMIT
            && hex::encode(Sha256::digest(bundled)) == c.bundled_sha256,
        "bundled library checksum mismatch"
    );
    Ok(())
}

// Physical ancestors plus O_NOFOLLOW: neither pointers nor catalog files may
// redirect reads. The private cache is the same user-owned boundary as packs.
fn regular(path: &Path, cap: u64) -> Result<File> {
    distribution::safe_directory(path.parent().context("official file parent")?)?;
    let meta = fs::symlink_metadata(path)?;
    ensure!(
        meta.is_file() && regressee_core::platform::ordinary(&meta) && meta.len() <= cap,
        "unsafe or oversized official runtime file"
    );
    let mut options = File::options();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW);
    }
    let f = options.open(path)?;
    ensure!(
        f.metadata()?.is_file() && f.metadata()?.len() <= cap,
        "official file changed during read"
    );
    Ok(f)
}
fn directory_listed(m: &Manifest, name: &str) -> bool {
    m.files
        .keys()
        .chain(m.links.keys())
        .any(|n| n.starts_with(&(name.to_owned() + "/")))
}
fn inventory_name(root: &Path, path: &Path) -> Result<String> {
    let mut parts = Vec::new();
    for component in path.strip_prefix(root)?.components() {
        let std::path::Component::Normal(part) = component else {
            anyhow::bail!("non-normal official inventory path");
        };
        let part = part.to_str().context("non UTF-8 official path")?;
        // On Unix a literal backslash is a filename character, NOT a separator.
        // Validate each physical component before joining using catalog separators.
        ensure!(
            distribution::safe_name(part) && !part.contains('/'),
            "unsafe physical official inventory component"
        );
        parts.push(part);
    }
    let name = parts.join("/");
    ensure!(
        distribution::safe_name(&name),
        "unsafe official inventory path"
    );
    Ok(name)
}
// Playwright writes this empty, mutable host-validation cache marker itself.
// It is not executable/catalog evidence and we never manufacture it, change its
// timestamp or skip host validation. Permit only the exact pinned Chromium root.
fn dependency_marker(c: &Catalog, name: &str) -> bool {
    c.components.iter().any(|a| {
        a.only.is_none()
            && a.destination
                .starts_with(&(c.manifest.browser_root.clone() + "/chromium-"))
            && a.destination
                .rsplit_once('/')
                .is_some_and(|(parent, _)| name == format!("{parent}/DEPENDENCIES_VALIDATED"))
    })
}
fn inventory(root: &Path, dir: &Path, c: &Catalog, cancel: &AtomicBool) -> Result<()> {
    let m = &c.manifest;
    for entry in fs::read_dir(dir)? {
        distribution::check_cancel(cancel)?;
        let p = entry?.path();
        let meta = fs::symlink_metadata(&p)?;
        let name = inventory_name(root, &p)?;
        if meta.is_dir() {
            ensure!(
                regressee_core::platform::ordinary(&meta) && directory_listed(m, &name),
                "unlisted official runtime directory"
            );
            inventory(root, &p, c, cancel)?;
        } else if let Some(expected) = m.links.get(&name) {
            ensure!(
                meta.file_type().is_symlink() && fs::read_link(p)?.to_str() == Some(expected),
                "official bundle link changed"
            );
        } else {
            ensure!(
                meta.is_file()
                    && regressee_core::platform::ordinary(&meta)
                    && (m.files.contains_key(&name)
                        || name == "READY"
                        || dependency_marker(c, &name)),
                "unlisted official runtime file"
            );
            if dependency_marker(c, &name) {
                ensure!(
                    meta.len() == 0 && regular(&p, 0)?.metadata()?.len() == 0,
                    "unsafe Playwright dependency marker"
                );
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    ensure!(
                        meta.permissions().mode() & 0o7111 == 0,
                        "executable or privileged Playwright dependency marker"
                    );
                }
            }
            if name == "READY" {
                let mut value = String::new();
                regular(&p, 128)?.read_to_string(&mut value)?;
                ensure!(value == m.id, "official readiness marker changed");
            }
        }
    }
    Ok(())
}
fn verify(root: &Path, c: &Catalog, cancel: &AtomicBool) -> Result<Installed> {
    validate_catalog(c)?;
    distribution::check_cancel(cancel)?;
    distribution::safe_directory(root)?;
    inventory(root, root, c, cancel)?;
    for (name, e) in &c.manifest.files {
        let f = regular(&root.join(name), e.bytes)?;
        let meta = f.metadata()?;
        ensure!(
            meta.len() == e.bytes && distribution::hash(f, cancel)? == e.sha256,
            "official runtime damaged; repair required"
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            ensure!(
                meta.permissions().mode() & 0o7777 == c.manifest.modes[name],
                "official runtime file permissions changed"
            );
        }
    }
    for (name, target) in &c.manifest.links {
        let p = root.join(name);
        distribution::safe_directory(p.parent().context("bundle link parent")?)?;
        ensure!(
            fs::symlink_metadata(&p)?.file_type().is_symlink()
                && fs::read_link(p)?.to_str() == Some(target),
            "official bundle link missing or changed"
        );
    }
    Ok(Installed {
        root: root.to_owned(),
        manifest: c.manifest.clone(),
    })
}
pub fn resolve(cache: &Path, c: &Catalog, cancel: &AtomicBool) -> Result<Installed> {
    validate_catalog(c)?;
    distribution::check_cancel(cancel)?;
    let mut name = String::new();
    regular(&cache.join("current"), 128)?.read_to_string(&mut name)?;
    ensure!(
        distribution::safe_name(&name) && !name.contains('/') && name.starts_with("runtime-"),
        "invalid official runtime pointer"
    );
    let installed = verify(&cache.join(name), c, cancel)?;
    let mut ready = String::new();
    regular(&installed.root.join("READY"), 128)?.read_to_string(&mut ready)?;
    ensure!(ready == c.manifest.id, "official runtime smoke missing");
    Ok(installed)
}

fn write_file(
    mut input: impl Read,
    size: u64,
    name: &str,
    root: &Path,
    m: &Manifest,
    cancel: &AtomicBool,
) -> Result<()> {
    let expected = m
        .files
        .get(name)
        .context("unlisted official archive file")?;
    ensure!(
        size == expected.bytes,
        "official archive file size mismatch"
    );
    let path = root.join(name);
    distribution::private_directory(path.parent().context("official extraction parent")?)?;
    let mut output = File::options().write(true).create_new(true).open(&path)?;
    let mut digest = Sha256::new();
    let mut count = 0u64;
    let mut buf = [0; 65536];
    loop {
        distribution::check_cancel(cancel)?;
        let n = input.read(&mut buf)?;
        if n == 0 {
            break;
        }
        count = count
            .checked_add(n as u64)
            .context("official file size overflow")?;
        ensure!(count <= size, "official extraction exceeds file budget");
        output.write_all(&buf[..n])?;
        digest.update(&buf[..n]);
    }
    ensure!(
        count == size && hex::encode(digest.finalize()) == expected.sha256,
        "official extracted checksum mismatch"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        output.set_permissions(fs::Permissions::from_mode(m.modes[name]))?;
    }
    output.sync_all()?;
    Ok(())
}
fn mapped(name: &str, a: &Component) -> Result<Option<String>> {
    ensure!(
        distribution::safe_name(name),
        "unsafe official archive path"
    );
    if name == a.prefix {
        return Ok(None);
    }
    let rel = name
        .strip_prefix(&(a.prefix.clone() + "/"))
        .context("unexpected official archive prefix")?;
    if a.only.as_ref().is_some_and(|only| rel != only) {
        return Ok(None);
    }
    Ok(Some(format!("{}/{rel}", a.destination)))
}
fn count_entry(count: &mut usize) -> Result<()> {
    *count += 1;
    ensure!(*count <= 20480, "official archive entry budget exceeded");
    Ok(())
}
fn extract_zip(
    input: impl Read + Seek,
    a: Option<&Component>,
    root: &Path,
    m: &Manifest,
    cancel: &AtomicBool,
    count: &mut usize,
    links: &mut BTreeSet<String>,
) -> Result<()> {
    let mut input = input;
    regressee_core::zip_directory::validate(&mut input, 20480, |n| {
        distribution::safe_name(n.strip_suffix('/').unwrap_or(n))
    })?;
    input.rewind()?;
    let mut zip = zip::ZipArchive::new(input)?;
    for i in 0..zip.len() {
        distribution::check_cancel(cancel)?;
        count_entry(count)?;
        let mut file = zip.by_index(i)?;
        let raw = file
            .name()
            .strip_suffix('/')
            .unwrap_or(file.name())
            .to_owned();
        let name = match a {
            Some(a) => mapped(&raw, a)?,
            None => Some(raw),
        };
        let Some(name) = name else {
            continue;
        };
        if file.is_dir() {
            ensure!(
                file.size() == 0
                    && directory_listed(m, &name)
                    && file
                        .unix_mode()
                        .is_none_or(|mode| matches!(mode & 0o170000, 0 | 0o040000)),
                "unlisted or unsafe official archive directory"
            );
            continue; // Only create directories needed by catalog leaves.
        }
        if let Some(expected) = m.links.get(&name) {
            ensure!(
                file.is_symlink() && file.size() == expected.len() as u64 && links.insert(name),
                "invalid or duplicate official archive link"
            );
            let mut target = String::new();
            file.take(1025).read_to_string(&mut target)?;
            ensure!(&target == expected, "official archive link target mismatch");
        } else {
            ensure!(
                !file.is_symlink()
                    && file
                        .unix_mode()
                        .is_none_or(|mode| matches!(mode & 0o170000, 0 | 0o100000)),
                "unsafe official archive entry type"
            );
            let size = file.size();
            write_file(&mut file, size, &name, root, m, cancel)?;
        }
    }
    Ok(())
}
// Also bounds/cancels decompression while TAR skips unselected npm entries.
struct Bounded<'a, R> {
    inner: R,
    cancel: &'a AtomicBool,
    remaining: u64,
}
impl<R: Read> Read for Bounded<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        distribution::check_cancel(self.cancel).map_err(std::io::Error::other)?;
        let len = buf.len().min(65536);
        let n = self.inner.read(&mut buf[..len])?;
        self.remaining = self
            .remaining
            .checked_sub(n as u64)
            .ok_or_else(|| std::io::Error::other("official TAR decompression budget exceeded"))?;
        Ok(n)
    }
}
const TAR_METADATA_LIMIT: u64 = 16 * 1024;
const TAR_METADATA_TOTAL: u64 = 1024 * 1024;
fn discard_exact(reader: &mut impl Read, mut bytes: u64) -> Result<()> {
    let mut buf = [0; 65536];
    while bytes > 0 {
        let len = bytes.min(buf.len() as u64) as usize;
        reader.read_exact(&mut buf[..len])?;
        bytes -= len as u64;
    }
    Ok(())
}
// Walk physical 512-byte headers, not tar::Archive's interpreted iterator.
// Even raw iterators must not be trusted to avoid sparse/extension processing.
// Fixed-header accessors below cannot consume or allocate extension payloads.
fn extract_tar_stream(
    input: impl Read,
    a: &Component,
    root: &Path,
    m: &Manifest,
    cancel: &AtomicBool,
    count: &mut usize,
) -> Result<()> {
    let only = a.only.as_ref().context("TAR selection missing")?;
    let selected = format!("{}/{only}", a.prefix);
    let mut reader = Bounded {
        inner: input,
        cancel,
        remaining: PACK_LIMIT,
    };
    let mut metadata = 0u64;
    let mut extended_next = false;
    let mut found = false;
    loop {
        let mut block = [0u8; 512];
        reader
            .read_exact(&mut block)
            .context("truncated official TAR header")?;
        if block.iter().all(|b| *b == 0) {
            ensure!(!extended_next, "dangling official TAR extension");
            // Require two terminator blocks and zero-only padding; still drain
            // through the bounded decoder to check gzip CRC/truncation.
            reader.read_exact(&mut block)?;
            ensure!(
                block.iter().all(|b| *b == 0),
                "invalid official TAR terminator"
            );
            let mut buf = [0; 65536];
            loop {
                let n = reader.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                ensure!(
                    buf[..n].iter().all(|b| *b == 0),
                    "unlisted official TAR tail payload"
                );
            }
            break;
        }
        count_entry(count)?; // Includes every physical GNU/PAX metadata header.
        let header = tar::Header::from_byte_slice(&block);
        let checksum: u32 = block
            .iter()
            .enumerate()
            .map(|(i, b)| {
                if (148..156).contains(&i) {
                    b' ' as u32
                } else {
                    *b as u32
                }
            })
            .sum();
        ensure!(
            header.cksum()? == checksum,
            "official TAR header checksum mismatch"
        );
        ensure!(
            header.as_gnu().is_some()
                || header.as_ustar().is_some()
                || block[257..265].iter().all(|b| *b == 0),
            "unsupported official TAR header format"
        );
        let kind = block[156];
        // Reject sparse (including its physical extension chain) and global PAX
        // before reading any payload or asking an interpreted parser for an entry.
        ensure!(
            matches!(kind, 0 | b'0' | b'1' | b'2' | b'5' | b'L' | b'K' | b'x'),
            "unsupported official TAR entry type (sparse/global metadata forbidden)"
        );
        let size = header.entry_size()?;
        let padding = (512 - size % 512) % 512;
        ensure!(
            size.checked_add(padding)
                .is_some_and(|n| n <= reader.remaining),
            "official TAR entry exceeds remaining decompression budget"
        );
        if matches!(kind, b'L' | b'K' | b'x') {
            ensure!(
                size <= TAR_METADATA_LIMIT,
                "official TAR extension metadata too large"
            );
            metadata = metadata
                .checked_add(size)
                .context("official TAR metadata overflow")?;
            ensure!(
                metadata <= TAR_METADATA_TOTAL,
                "official TAR metadata budget exceeded"
            );
            // Real Node archives have unneeded npm longname/longlink entries.
            // Stream them without interpreting paths, sizes or sparse records.
            // Never select a following entry whose semantics they could alter.
            extended_next = true;
            discard_exact(&mut reader, size)?;
        } else {
            let name = header.path_bytes(); // At most the fixed name + ustar prefix.
            if name.as_ref() == selected.as_bytes() {
                ensure!(
                    !extended_next,
                    "selected Node must not use TAR extension metadata"
                );
                ensure!(
                    matches!(kind, 0 | b'0') && !found,
                    "selected Node is not a unique regular file"
                );
                write_file(
                    (&mut reader).take(size),
                    size,
                    &format!("{}/{only}", a.destination),
                    root,
                    m,
                    cancel,
                )?;
                found = true;
            } else {
                discard_exact(&mut reader, size)?;
            }
            extended_next = false;
        }
        discard_exact(&mut reader, padding)?;
    }
    ensure!(
        found,
        "selected Node missing from physical official TAR headers"
    );
    Ok(())
}
fn extract_tar(
    input: File,
    a: &Component,
    root: &Path,
    m: &Manifest,
    cancel: &AtomicBool,
    count: &mut usize,
) -> Result<()> {
    extract_tar_stream(
        flate2::read::GzDecoder::new(input),
        a,
        root,
        m,
        cancel,
        count,
    )
}
async fn await_cancel<T>(
    future: impl std::future::Future<Output = std::result::Result<T, reqwest::Error>>,
    cancel: &AtomicBool,
) -> Result<T> {
    tokio::pin!(future);
    loop {
        distribution::check_cancel(cancel)?;
        tokio::select! {
            result = &mut future => return Ok(result?),
            _ = tokio::time::sleep(Duration::from_millis(50)) => distribution::check_cancel(cancel)?,
        }
    }
}
fn pointer_safe(cache: &Path) -> Result<()> {
    distribution::safe_directory(cache)?;
    match fs::symlink_metadata(cache.join("current")) {
        Ok(m) => ensure!(
            m.is_file() && regressee_core::platform::ordinary(&m) && m.len() <= 128,
            "unsafe official runtime pointer"
        ),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
        Err(e) => return Err(e.into()),
    }
    Ok(())
}
pub fn setup_network_with_progress(
    cache: &Path,
    c: &Catalog,
    bundled: &[u8],
    cancel: &AtomicBool,
    progress: &mut dyn FnMut(InstallProgress),
    smoke: &mut dyn FnMut(&Installed) -> Result<()>,
    network: &NetworkSettings,
) -> Result<Installed> {
    validate(c, bundled)?;
    distribution::check_cancel(cancel)?;
    network.validate()?;
    distribution::private_directory(cache)?;
    pointer_safe(cache)?;
    let total: u64 = c.components.iter().map(|a| a.bytes).sum();
    let unpacked: u64 = c.manifest.files.values().map(|e| e.bytes).sum();
    let required = total
        .checked_add(unpacked)
        .and_then(|n| n.checked_add(128 * 1024 * 1024))
        .context("official disk budget overflow")?;
    ensure!(
        distribution::available_bytes(cache)? >= required,
        "not enough space for official browser setup"
    );
    let stage = tempfile::Builder::new()
        .prefix("official-setup-")
        .tempdir_in(cache)?;
    regressee_core::platform::protect(stage.path())?;
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;
    let mut bytes = 0u64;
    let mut count = 0usize;
    let mut links = BTreeSet::new();
    for a in &c.components {
        distribution::check_cancel(cancel)?;
        let mut archive = tempfile::NamedTempFile::new_in(cache)?;
        progress(InstallProgress::Downloading {
            bytes,
            total: Some(total),
        });
        runtime.block_on(async {
            let client = network.download_client()?;
            let mut response = await_cancel(client.get(&a.url).send(), cancel).await?;
            ensure!(
                response.status().is_success(),
                "official source refused (redirects are not accepted)"
            );
            ensure!(
                response.content_length().is_none_or(|n| n == a.bytes),
                "official archive length mismatch"
            );
            let mut received = 0u64;
            while let Some(chunk) = await_cancel(response.chunk(), cancel).await? {
                received = received
                    .checked_add(chunk.len() as u64)
                    .context("official download overflow")?;
                ensure!(
                    received <= a.bytes,
                    "official download exceeds pinned length"
                );
                archive.write_all(&chunk)?;
                bytes += chunk.len() as u64;
                progress(InstallProgress::Downloading {
                    bytes,
                    total: Some(total),
                });
            }
            ensure!(received == a.bytes, "official download truncated");
            Ok::<_, anyhow::Error>(())
        })?;
        archive.as_file().sync_all()?;
        progress(InstallProgress::Verifying);
        ensure!(
            distribution::hash(regular(archive.path(), a.bytes)?, cancel)? == a.sha256,
            "official archive checksum mismatch"
        );
        progress(InstallProgress::Extracting);
        match a.format {
            ArchiveFormat::Zip => extract_zip(
                regular(archive.path(), a.bytes)?,
                Some(a),
                stage.path(),
                &c.manifest,
                cancel,
                &mut count,
                &mut links,
            )?,
            ArchiveFormat::TarGz => extract_tar(
                regular(archive.path(), a.bytes)?,
                a,
                stage.path(),
                &c.manifest,
                cancel,
                &mut count,
            )?,
        }
    }
    runtime.shutdown_timeout(Duration::from_secs(1));
    progress(InstallProgress::Extracting);
    extract_zip(
        Cursor::new(bundled),
        None,
        stage.path(),
        &c.manifest,
        cancel,
        &mut count,
        &mut links,
    )?;
    ensure!(
        links.len() == c.manifest.links.len(),
        "official archive links missing"
    );
    // All link payloads were checked; create LAST so links cannot be extraction parents.
    #[cfg(unix)]
    for (name, target) in &c.manifest.links {
        distribution::check_cancel(cancel)?;
        let path = stage.path().join(name);
        distribution::private_directory(path.parent().context("official link parent")?)?;
        std::os::unix::fs::symlink(target, path)?;
    }
    progress(InstallProgress::Verifying);
    let installed = verify(stage.path(), c, cancel)?;
    progress(InstallProgress::TestingBrowser);
    distribution::check_cancel(cancel)?;
    smoke(&installed)?;
    distribution::check_cancel(cancel)?;
    let mut ready = File::options()
        .write(true)
        .create_new(true)
        .open(stage.path().join("READY"))?;
    ready.write_all(c.manifest.id.as_bytes())?;
    ready.sync_all()?;
    // Close the last descendant handle before publishing the directory on Windows.
    drop(ready);
    #[cfg(unix)]
    File::open(stage.path())?.sync_all()?;
    let root = cache.join(format!("runtime-{}", uuid::Uuid::new_v4()));
    let mut pointer = tempfile::NamedTempFile::new_in(cache)?;
    pointer.write_all(root.file_name().unwrap().as_encoded_bytes())?;
    pointer.as_file().sync_all()?;
    pointer_safe(cache)?;
    distribution::check_cancel(cancel)?;
    fs::rename(stage.path(), &root)
        .context("Cannot publish verified official runtime generation")?;
    // Once committed, do not report cancellation instead of the successful publication.
    pointer
        .persist(cache.join("current"))
        .map_err(|e| e.error)
        .context("Cannot publish official runtime pointer")?;
    #[cfg(unix)]
    File::open(cache)?.sync_all()?;
    progress(InstallProgress::Ready);
    Ok(Installed {
        root,
        manifest: c.manifest.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::distribution::Entry;
    use std::collections::BTreeMap;

    fn fixture() -> Result<(Catalog, Vec<u8>)> {
        let mut zip = zip::ZipWriter::new(Cursor::new(Vec::new()));
        let mut files = BTreeMap::new();
        let mut modes = BTreeMap::new();
        for (name, value, executable) in [
            ("node/bin/node", "inert node", true),
            ("worker/index.js", "inert worker", false),
            (
                "browsers/chromium-1248/chrome-linux64/chrome",
                "inert browser",
                true,
            ),
        ] {
            let mode = if executable { 0o700 } else { 0o600 };
            zip.start_file(
                name,
                zip::write::SimpleFileOptions::default().unix_permissions(mode),
            )?;
            zip.write_all(value.as_bytes())?;
            files.insert(
                name.into(),
                Entry {
                    sha256: hex::encode(Sha256::digest(value.as_bytes())),
                    bytes: value.len() as u64,
                    executable,
                },
            );
            modes.insert(name.into(), mode);
        }
        let bundled = zip.finish()?.into_inner();
        let catalog = Catalog {
            manifest: Manifest {
                version: 3,
                id: "inert-official-fixture".into(),
                os: std::env::consts::OS.into(),
                arch: std::env::consts::ARCH.into(),
                scope: Scope::OfficialPinned,
                support_until: Some(distribution::now() + 86400),
                identity: RuntimeIdentity::Pinned {
                    node: "inert".into(),
                    worker: "inert".into(),
                    playwright: "inert".into(),
                    browser: "inert".into(),
                },
                node: "node/bin/node".into(),
                worker: "worker/index.js".into(),
                browser_root: "browsers".into(),
                files,
                links: BTreeMap::new(),
                modes,
            },
            components: vec![
                Component {
                    url: "https://nodejs.org/dist/inert-fixture.tar.gz".into(),
                    sha256: "a".repeat(64),
                    bytes: 1,
                    format: ArchiveFormat::TarGz,
                    prefix: "node-fixture".into(),
                    destination: "node".into(),
                    only: Some("bin/node".into()),
                },
                Component {
                    url: "https://storage.googleapis.com/inert-fixture.zip".into(),
                    sha256: "b".repeat(64),
                    bytes: 1,
                    format: ArchiveFormat::Zip,
                    prefix: "chrome-linux64".into(),
                    destination: "browsers/chromium-1248/chrome-linux64".into(),
                    only: None,
                },
            ],
            bundled_sha256: hex::encode(Sha256::digest(&bundled)),
        };
        Ok((catalog, bundled))
    }
    #[test]
    fn catalog_refuses_untrusted_urls_paths_hashes_modes_and_expiry() -> Result<()> {
        let (c, bundled) = fixture()?;
        validate(&c, &bundled)?;
        for url in [
            "http://nodejs.org/a",
            "https://nodejs.org.evil.test/a",
            "https://user@nodejs.org/a",
            "https://nodejs.org/a?q=1",
            "https://nodejs.org/a#b",
        ] {
            let mut bad = c.clone();
            bad.components[0].url = url.into();
            assert!(validate(&bad, &bundled).is_err());
        }
        let mut bad = c.clone();
        bad.components[0].only = Some("../node".into());
        assert!(validate(&bad, &bundled).is_err());
        let mut bad = c.clone();
        bad.manifest.files.get_mut("node/bin/node").unwrap().sha256 = "A".repeat(64);
        assert!(validate(&bad, &bundled).is_err());
        let mut bad = c.clone();
        bad.manifest.modes.insert("node/bin/node".into(), 0o4700);
        assert!(validate(&bad, &bundled).is_err());
        let mut bad = c.clone();
        bad.manifest.support_until = Some(0);
        assert!(validate(&bad, &bundled).is_err());
        assert!(validate(&c, b"different library").is_err());
        assert_eq!(
            mapped("node-fixture/bin/node", &c.components[0])?,
            Some("node/bin/node".into())
        );
        assert!(mapped("node-fixture/../bin/node", &c.components[0]).is_err());
        Ok(())
    }
    #[test]
    fn inert_zip_is_verified_against_catalog_not_local_metadata() -> Result<()> {
        let (c, bundled) = fixture()?;
        let root = tempfile::tempdir()?;
        let cancel = AtomicBool::new(false);
        extract_zip(
            Cursor::new(&bundled),
            None,
            root.path(),
            &c.manifest,
            &cancel,
            &mut 0,
            &mut BTreeSet::new(),
        )?;
        verify(root.path(), &c, &cancel)?;
        assert!(
            extract_zip(
                Cursor::new(&bundled),
                None,
                root.path(),
                &c.manifest,
                &cancel,
                &mut 0,
                &mut BTreeSet::new()
            )
            .is_err()
        );
        fs::write(
            root.path().join("manifest.json"),
            b"untrusted local inventory",
        )?;
        assert!(verify(root.path(), &c, &cancel).is_err());
        Ok(())
    }
    #[test]
    fn only_empty_ordinary_pinned_host_validation_marker_is_permitted() -> Result<()> {
        let (c, bundled) = fixture()?;
        let root = tempfile::tempdir()?;
        let cancel = AtomicBool::new(false);
        extract_zip(
            Cursor::new(&bundled),
            None,
            root.path(),
            &c.manifest,
            &cancel,
            &mut 0,
            &mut BTreeSet::new(),
        )?;
        let marker = root
            .path()
            .join("browsers/chromium-1248/DEPENDENCIES_VALIDATED");
        fs::write(&marker, b"")?;
        verify(root.path(), &c, &cancel)?;
        assert!(!dependency_marker(&c, "worker/DEPENDENCIES_VALIDATED"));
        assert!(!dependency_marker(
            &c,
            "browsers/chromium-9999/DEPENDENCIES_VALIDATED"
        ));
        fs::write(&marker, b"not empty")?;
        assert!(verify(root.path(), &c, &cancel).is_err());
        fs::write(&marker, b"")?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&marker, fs::Permissions::from_mode(0o700))?;
            assert!(verify(root.path(), &c, &cancel).is_err());
            fs::remove_file(&marker)?;
            std::os::unix::fs::symlink("chrome-linux64/chrome", &marker)?;
            assert!(verify(root.path(), &c, &cancel).is_err());
        }
        Ok(())
    }
    #[cfg(unix)]
    #[test]
    fn resolver_rejects_literal_backslash_file_and_directory_aliases() -> Result<()> {
        let (c, bundled) = fixture()?;
        let cache = tempfile::tempdir()?;
        let root = cache.path().join("runtime-inert");
        distribution::private_directory(&root)?;
        let cancel = AtomicBool::new(false);
        extract_zip(
            Cursor::new(&bundled),
            None,
            &root,
            &c.manifest,
            &cancel,
            &mut 0,
            &mut BTreeSet::new(),
        )?;
        fs::write(root.join("READY"), c.manifest.id.as_bytes())?;
        fs::write(cache.path().join("current"), b"runtime-inert")?;
        resolve(cache.path(), &c, &cancel)?;
        // This is the original normalization collision, not a catalog mutation.
        let leaf = "node\\bin\\node";
        assert_eq!(leaf.replace('\\', "/"), c.manifest.node);
        fs::write(root.join(leaf), b"unverified extra bytes")?;
        assert!(resolve(cache.path(), &c, &cancel).is_err());
        assert_eq!(fs::read(root.join(leaf))?, b"unverified extra bytes");
        fs::remove_file(root.join(leaf))?;
        let dir = "node\\bin";
        assert!(directory_listed(&c.manifest, &dir.replace('\\', "/")));
        fs::create_dir(root.join(dir))?;
        assert!(resolve(cache.path(), &c, &cancel).is_err());
        assert_eq!(fs::read(cache.path().join("current"))?, b"runtime-inert");
        Ok(())
    }

    fn physical_header(name: &str, kind: u8, size: u64) -> Result<[u8; 512]> {
        let mut header = tar::Header::new_gnu();
        header.set_path(name)?;
        header.set_entry_type(tar::EntryType::new(kind));
        header.set_size(size);
        header.set_cksum();
        Ok(*header.as_bytes())
    }
    struct ReadCount<'a> {
        input: Cursor<Vec<u8>>,
        bytes: &'a std::cell::Cell<usize>,
    }
    impl Read for ReadCount<'_> {
        fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
            let n = self.input.read(buf)?;
            self.bytes.set(self.bytes.get() + n);
            Ok(n)
        }
    }
    #[test]
    fn tar_refuses_large_extensions_and_sparse_before_payload_reads() -> Result<()> {
        let (c, _) = fixture()?;
        let root = tempfile::tempdir()?;
        let cancel = AtomicBool::new(false);
        for kind in *b"LKxSg" {
            // Only a physical header is necessary. Previously L/K/x caused the
            // interpreted iterator to allocate/read before our entry guard.
            // S advertises an extension chain which must never be traversed.
            let mut header = physical_header("unselected-metadata", kind, TAR_METADATA_LIMIT + 1)?;
            if kind == b'S' {
                header[482] = 1; // GNU sparse isextended.
                let mut h = tar::Header::new_gnu();
                h.as_mut_bytes().copy_from_slice(&header);
                h.set_cksum();
                header = *h.as_bytes();
            }
            let bytes = std::cell::Cell::new(0);
            let input = ReadCount {
                input: Cursor::new(header.to_vec()),
                bytes: &bytes,
            };
            let mut count = 0;
            assert!(
                extract_tar_stream(
                    input,
                    &c.components[0],
                    root.path(),
                    &c.manifest,
                    &cancel,
                    &mut count
                )
                .is_err()
            );
            assert_eq!(bytes.get(), 512, "metadata payload must not be read");
            assert_eq!(count, 1, "physical metadata header must be counted");
            assert!(!root.path().join(&c.manifest.node).exists());
        }
        Ok(())
    }
    #[test]
    fn tar_streams_unselected_extensions_but_never_selects_extended_node() -> Result<()> {
        let (c, _) = fixture()?;
        let root = tempfile::tempdir()?;
        let cancel = AtomicBool::new(false);
        let selected = "node-fixture/bin/node";
        let mut input = Vec::new();
        input.extend_from_slice(&physical_header("././@LongLink", b'L', 4)?);
        input.extend_from_slice(b"npm\0");
        input.extend_from_slice(&[0; 508]);
        input.extend_from_slice(&physical_header("unselected-npm", b'0', 0)?);
        input.extend_from_slice(&physical_header(selected, b'0', 10)?);
        input.extend_from_slice(b"inert node");
        input.extend_from_slice(&[0; 502]);
        input.extend_from_slice(&[0; 1024]);
        let mut count = 0;
        extract_tar_stream(
            Cursor::new(&input),
            &c.components[0],
            root.path(),
            &c.manifest,
            &cancel,
            &mut count,
        )?;
        assert_eq!(count, 3);
        assert_eq!(fs::read(root.path().join(&c.manifest.node))?, b"inert node");
        // Removing the unselected physical leaf makes the extension apply to
        // Node: ignoring it and selecting the raw name would change semantics.
        input.drain(1024..1536);
        let other = tempfile::tempdir()?;
        assert!(
            extract_tar_stream(
                Cursor::new(&input),
                &c.components[0],
                other.path(),
                &c.manifest,
                &cancel,
                &mut 0
            )
            .is_err()
        );
        assert!(!other.path().join(&c.manifest.node).exists());
        let bytes = std::cell::Cell::new(0);
        let reader = ReadCount {
            input: Cursor::new(input),
            bytes: &bytes,
        };
        let mut count = 20480;
        assert!(
            extract_tar_stream(
                reader,
                &c.components[0],
                other.path(),
                &c.manifest,
                &cancel,
                &mut count
            )
            .is_err()
        );
        assert_eq!(bytes.get(), 512);
        Ok(())
    }
}
