// SPDX-License-Identifier: MPL-2.0
use anyhow::Result;
use ed25519_dalek::{Signer, SigningKey};
use regressee_core::domain::RuntimeIdentity;
use regressee_runtime::distribution::*;
use std::{
    collections::BTreeMap,
    fs::{self, File},
    io::Write,
    sync::atomic::AtomicBool,
};
use zip::{ZipWriter, write::SimpleFileOptions};
fn fixture(root: &std::path::Path, change: &str) -> Result<(std::path::PathBuf, Distribution)> {
    let key = SigningKey::from_bytes(&[39; 32]); // Test-only key, never linked as app trust.
    let d = Distribution {
        version: 1,
        url: "http://127.0.0.1/runtime.zip".parse()?,
        public_key: hex::encode(key.verifying_key().to_bytes()),
        scope: Scope::DevelopmentLoopback,
    };
    let payload = b"inert fixture, never an executable";
    let e = Entry {
        sha256: hash(&payload[..], &AtomicBool::new(false))?,
        bytes: payload.len() as u64,
        executable: false,
    };
    let mut m = Manifest {
        links: BTreeMap::new(),
        modes: BTreeMap::new(),
        version: 2,
        id: "inert-test".into(),
        os: std::env::consts::OS.into(),
        arch: std::env::consts::ARCH.into(),
        scope: d.scope,
        support_until: None,
        identity: RuntimeIdentity::Pinned {
            node: "test".into(),
            worker: "test".into(),
            playwright: "test".into(),
            browser: "test".into(),
        },
        node: "node/file".into(),
        worker: "worker/file".into(),
        browser_root: "browsers".into(),
        files: BTreeMap::from([
            ("node/file".into(), e.clone()),
            ("worker/file".into(), e.clone()),
            ("browsers/file".into(), e),
        ]),
    };
    match change {
        "path" => {
            let e = m.files.remove("worker/file").unwrap();
            m.files.insert("../outside".into(), e);
        }
        "budget" => m.files.get_mut("worker/file").unwrap().bytes = PACK_LIMIT + 1,
        "platform" => m.os = "not-this-os".into(),
        _ => (),
    }
    let bytes = serde_json::to_vec(&m)?;
    let pack = root.join(format!("{change}.zip"));
    let mut z = ZipWriter::new(File::create(&pack)?);
    let opts = SimpleFileOptions::default();
    z.start_file("manifest.json", opts)?;
    z.write_all(&bytes)?;
    z.start_file("manifest.sig", opts)?;
    z.write_all(&if change == "signature" {
        [0; 64]
    } else {
        key.sign(&bytes).to_bytes()
    })?;
    for name in m.files.keys() {
        if change == "symlink" && name == "worker/file" {
            z.add_symlink(name, "outside", opts)?;
        } else {
            z.start_file(name, opts)?;
            z.write_all(if change == "checksum" {
                b"bad"
            } else {
                payload
            })?;
        }
    }
    z.finish()?;
    Ok((pack, d))
}
#[test]
fn invalid_packs_never_smoke_or_replace_existing_pointer() -> Result<()> {
    let root = tempfile::tempdir()?;
    let cache = root.path().join("cache");
    fs::create_dir(&cache)?;
    fs::write(cache.join("current"), b"existing-sentinel")?;
    for name in [
        "path",
        "budget",
        "platform",
        "signature",
        "symlink",
        "checksum",
    ] {
        let (pack, d) = fixture(root.path(), name)?;
        let mut smoked = false;
        let e = install(&pack, &cache, &d, &AtomicBool::new(false), &mut |_| {
            smoked = true;
            Ok(())
        })
        .unwrap_err();
        assert!(!smoked, "{name} executed smoke");
        assert_eq!(fs::read(cache.join("current"))?, b"existing-sentinel");
        assert_eq!(fs::read_dir(&cache)?.count(), 1, "{name}: {e}");
        assert!(
            e.to_string().contains(match name {
                "path" => "unsafe",
                "budget" => "budget",
                "platform" => "incompatible",
                "signature" => "signature rejected",
                "symlink" => "archive type",
                "checksum" => "size mismatch",
                _ => unreachable!(),
            }),
            "{name}: {e}"
        );
    }
    Ok(())
}
#[test]
fn install_verify_repair_and_bad_smoke_preserve_old_generation() -> Result<()> {
    let root = tempfile::tempdir()?;
    let (pack, d) = fixture(root.path(), "valid")?;
    let cache = root.path().join("cache");
    let c = AtomicBool::new(false);
    let first = install(&pack, &cache, &d, &c, &mut |_| Ok(()))?;
    let pointer = fs::read(cache.join("current"))?;
    assert_eq!(resolve(&cache, &d, &c)?.root, first.root);
    assert!(
        install(&pack, &cache, &d, &c, &mut |_| anyhow::bail!(
            "smoke intentionally failed"
        ))
        .is_err()
    );
    assert_eq!(fs::read(cache.join("current"))?, pointer);
    fs::write(first.root.join("worker/file"), b"damaged")?;
    assert!(resolve(&cache, &d, &c).is_err());
    let second = install(&pack, &cache, &d, &c, &mut |_| Ok(()))?;
    assert_ne!(first.root, second.root);
    assert!(first.root.exists());
    assert_eq!(resolve(&cache, &d, &c)?.root, second.root);
    assert!(
        install(&pack, &cache, &d, &AtomicBool::new(true), &mut |_| panic!(
            "cancelled smoke"
        ))
        .is_err()
    );
    Ok(())
}
#[test]
fn trust_transport_and_literal_loopback_are_fail_closed() -> Result<()> {
    for name in [
        "CON",
        "worker/NUL.txt",
        "a./file",
        "a /file",
        "worker/COM1.exe",
        "worker/LPT9",
        "worker/a?b",
        "worker/line\nbreak",
    ] {
        assert!(!safe_name(name), "{name}");
    }
    let root = tempfile::tempdir()?;
    let (_, mut d) = fixture(root.path(), "valid")?;
    for u in [
        "http://example.com/pack",
        "http://localhost/pack",
        "http://127.0.0.1:pw@example.com/pack",
        "file:///tmp/pack",
    ] {
        d.url = u.parse()?;
        assert!(d.validate().is_err(), "{u}");
    }
    d.url = "https://example.com/pack".parse()?;
    d.validate()?;
    d.scope = Scope::QualifiedPublic;
    d.url = "http://127.0.0.1/pack".parse()?;
    assert!(d.validate().is_err());
    for u in ["http://127.0.0.1/", "http://[::1]/"] {
        assert!(loopback(&u.parse()?));
    }
    Ok(())
}
#[cfg(unix)]
#[test]
fn phase7_signed_bundle_links_modes_and_hostile_refusal() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let root = tempfile::tempdir()?;
    let (base, d) = fixture(root.path(), "bundle-base")?;
    let mut archive = zip::ZipArchive::new(File::open(base)?)?;
    let mut m: Manifest = serde_json::from_reader(archive.by_name("manifest.json")?)?;
    let browser = "browsers/Chrome.app/Contents/Frameworks/F.framework";
    let file = format!("{browser}/Versions/A/F");
    let e = m.files.remove("browsers/file").unwrap();
    m.files.insert(file.clone(), e);
    m.version = 3;
    m.modes = m.files.keys().map(|n| (n.clone(), 0o644)).collect();
    m.links
        .insert(format!("{browser}/Versions/Current"), "A".into());
    m.links
        .insert(format!("{browser}/F"), "Versions/Current/F".into());
    let key = SigningKey::from_bytes(&[39; 32]);
    let make = |m: &Manifest, name: &str| -> Result<std::path::PathBuf> {
        let path = root.path().join(format!("{name}.zip"));
        let mut z = ZipWriter::new(File::create(&path)?);
        let opts = SimpleFileOptions::default();
        let bytes = serde_json::to_vec(m)?;
        z.start_file("manifest.json", opts)?;
        z.write_all(&bytes)?;
        z.start_file("manifest.sig", opts)?;
        z.write_all(&key.sign(&bytes).to_bytes())?;
        for n in m.files.keys() {
            z.start_file(n, opts)?;
            z.write_all(b"inert fixture, never an executable")?;
        }
        for (n, v) in &m.links {
            z.add_symlink(n, v, opts)?;
        }
        z.finish()?;
        Ok(path)
    };
    let cache = root.path().join("bundle-cache");
    let cancel = AtomicBool::new(false);
    let good = install(&make(&m, "bundle-good")?, &cache, &d, &cancel, &mut |_| {
        Ok(())
    })?;
    assert_eq!(
        fs::read_link(good.root.join(format!("{browser}/Versions/Current")))?,
        std::path::PathBuf::from("A")
    );
    assert_eq!(
        fs::read(good.root.join(format!("{browser}/F")))?,
        b"inert fixture, never an executable"
    );
    assert_eq!(
        fs::metadata(good.root.join(&file))?.permissions().mode() & 0o777,
        0o644
    );
    let pointer = fs::read(cache.join("current"))?;
    for bad in [
        "escape",
        "absolute",
        "cycle",
        "dangling",
        "collision",
        "case-dir",
        "mode",
        "link-parent",
    ] {
        let mut hostile = m.clone();
        match bad {
            "escape" => {
                hostile
                    .links
                    .insert(format!("{browser}/F"), "../../../../../../outside".into());
            }
            "absolute" => {
                hostile
                    .links
                    .insert(format!("{browser}/F"), "/etc/passwd".into());
            }
            "cycle" => {
                hostile
                    .links
                    .insert(format!("{browser}/Versions/Current"), "Current".into());
            }
            "dangling" => {
                hostile
                    .links
                    .insert(format!("{browser}/F"), "missing".into());
            }
            "collision" => {
                hostile.links.insert(file.clone(), "F".into());
            }
            "case-dir" => {
                hostile
                    .files
                    .insert(file.replace("Contents", "contents"), m.files[&file].clone());
                hostile
                    .modes
                    .insert(file.replace("Contents", "contents"), 0o644);
            }
            "mode" => {
                hostile.modes.insert(file.clone(), 0o4755);
            }
            "link-parent" => {
                let n = format!("{browser}/Versions/Current/evil");
                hostile.files.insert(n.clone(), m.files[&file].clone());
                hostile.modes.insert(n, 0o644);
            }
            _ => unreachable!(),
        }
        // Validate collisions before ZIPWriter itself may deduplicate them.
        assert!(
            regressee_runtime::bundle::validate(&hostile).is_err(),
            "{bad}"
        );
        if bad != "collision" {
            assert!(
                install(&make(&hostile, bad)?, &cache, &d, &cancel, &mut |_| panic!(
                    "hostile smoke"
                ))
                .is_err(),
                "{bad}"
            );
        }
        assert_eq!(fs::read(cache.join("current"))?, pointer);
        verify(&good.root, &d, &cancel)?;
    }
    fs::remove_file(good.root.join(format!("{browser}/F")))?;
    std::os::unix::fs::symlink("Versions/A/F", good.root.join(format!("{browser}/F")))?;
    assert!(verify(&good.root, &d, &cancel).is_err());
    Ok(())
}
#[test]
fn http_download_failure_and_mid_body_cancel_preserve_cache() -> Result<()> {
    use std::{
        io::Read,
        net::TcpListener,
        sync::{Arc, atomic::Ordering},
        time::{Duration, Instant},
    };
    let root = tempfile::tempdir()?;
    let (_, mut d) = fixture(root.path(), "valid")?;
    let cache = root.path().join("cache");
    fs::create_dir(&cache)?;
    fs::write(cache.join("current"), b"unchanged")?;
    for mode in ["corrupt", "cancel", "oversized"] {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        listener.set_nonblocking(true)?;
        d.url = format!("http://{}/runtime.zip", listener.local_addr()?).parse()?;
        let done = Arc::new(AtomicBool::new(false));
        let stop = done.clone();
        let mode = mode.to_owned();
        let server = std::thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(3);
            let (mut s, _) = loop {
                match listener.accept() {
                    Ok(connection) => break connection,
                    Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                        if stop.load(Ordering::Acquire) || Instant::now() >= deadline {
                            return;
                        }
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    Err(_) => return,
                }
            };
            let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
            let _ = s.set_write_timeout(Some(Duration::from_secs(2)));
            let mut b = [0; 1024];
            let _ = s.read(&mut b);
            match mode.as_str() {
                "corrupt" => {
                    let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 3\r\n\r\nbad");
                }
                "oversized" => {
                    let _ = write!(
                        s,
                        "HTTP/1.1 200 OK\r\nContent-Length: {}\r\n\r\n",
                        PACK_LIMIT + 1
                    );
                }
                _ => {
                    let _ = s.write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 1000\r\n\r\nx");
                    while !stop.load(Ordering::Acquire) {
                        std::thread::sleep(Duration::from_millis(10));
                    }
                }
            }
        });
        let cancel = Arc::new(AtomicBool::new(false));
        let c = cancel.clone();
        let timer = std::thread::spawn(move || {
            std::thread::sleep(Duration::from_millis(150));
            c.store(true, Ordering::Release);
        });
        let start = Instant::now();
        let error = setup(&cache, &d, &cancel, &mut |_, _| {}, &mut |_| {
            panic!("bad download reached smoke")
        })
        .unwrap_err();
        assert!(
            start.elapsed() < Duration::from_secs(2),
            "unbounded download cancellation: {error}"
        );
        done.store(true, Ordering::Release);
        server.join().unwrap();
        timer.join().unwrap();
        assert_eq!(fs::read(cache.join("current"))?, b"unchanged");
        assert_eq!(fs::read_dir(&cache)?.count(), 1);
    }
    Ok(())
}
#[cfg(unix)]
#[test]
fn unsafe_pointer_and_runtime_symlinks_are_rejected_before_execution() -> Result<()> {
    use std::os::unix::fs::symlink;
    let root = tempfile::tempdir()?;
    let (pack, d) = fixture(root.path(), "valid")?;
    let cache = root.path().join("cache");
    fs::create_dir(&cache)?;
    let outside = root.path().join("outside");
    symlink(&outside, cache.join("current"))?;
    assert_eq!(
        install(&pack, &cache, &d, &AtomicBool::new(false), &mut |_| panic!(
            "unsafe pointer reached smoke"
        ))
        .unwrap_err()
        .to_string(),
        "unsafe runtime pointer"
    );
    assert!(!outside.exists());
    fs::remove_file(cache.join("current"))?;
    let i = install(&pack, &cache, &d, &AtomicBool::new(false), &mut |_| Ok(()))?;
    fs::remove_file(i.root.join("worker/file"))?;
    symlink(&outside, i.root.join("worker/file"))?;
    assert!(verify(&i.root, &d, &AtomicBool::new(false)).is_err());
    assert!(!outside.exists());
    Ok(())
}
