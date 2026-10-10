// SPDX-License-Identifier: MPL-2.0
//! Session-secret boundary only; no plaintext persistence or archive integration.
use aes_gcm::{
    Aes256Gcm, KeyInit, Nonce,
    aead::{Aead, AeadCore, OsRng, Payload},
};
use regressee_core::domain::SessionReference;
use std::{
    fs,
    io::{Read, Write},
    path::{Path, PathBuf},
};
use zeroize::Zeroize;
use zeroize::Zeroizing;

pub struct SessionMaterial(Vec<u8>);
impl SessionMaterial {
    pub fn new(mut bytes: Vec<u8>) -> Result<Self, SecretError> {
        if bytes.len() > 16 * 1024 * 1024 {
            bytes.zeroize();
            return Err(SecretError::TooLarge);
        }
        Ok(Self(bytes))
    }
    pub fn expose(&self) -> &[u8] {
        &self.0
    }
}
impl std::fmt::Debug for SessionMaterial {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("SessionMaterial([REDACTED])")
    }
}
impl Drop for SessionMaterial {
    fn drop(&mut self) {
        self.0.zeroize();
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SecretError {
    SecureStoreUnavailable,
    ScopeMismatch,
    TooLarge,
    Missing,
    Corrupt,
}
impl std::fmt::Display for SecretError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SecureStoreUnavailable => "Secure storage is unavailable or locked. Unlock your OS credential store, or choose Ephemeral: login lasts until Regressee closes; nothing is saved in plaintext.",
            Self::ScopeMismatch => "Saved login belongs to another environment. Log in again.",
            Self::Missing => "Saved login is missing. Log in again.",
            Self::Corrupt => "Saved login could not be decrypted. Keep existing evidence and log in again.",
            Self::TooLarge => "Browser session exceeds the private 512 KiB transfer limit. This login flow is not supported.",
        })
    }
}
impl std::error::Error for SecretError {}

/// Injectable only at an explicit constructor boundary; production never selects
/// a mock provider from environment variables or portable settings.
pub trait KeyProvider: Send {
    fn key(&mut self) -> Result<Zeroizing<[u8; 32]>, SecretError>;
}
pub struct OsKeyProvider {
    sender: std::sync::mpsc::SyncSender<
        std::sync::mpsc::SyncSender<Result<Zeroizing<[u8; 32]>, SecretError>>,
    >,
}
impl OsKeyProvider {
    pub fn new(installation: uuid::Uuid) -> Self {
        let (sender, requests) = std::sync::mpsc::sync_channel::<
            std::sync::mpsc::SyncSender<Result<Zeroizing<[u8; 32]>, SecretError>>,
        >(1);
        std::thread::spawn(move || {
            // Only Regressee-owned entries are addressed, never enumeration.
            let account = format!("installation-{installation}");
            for reply in requests {
                let result = (|| {
                    if !cfg!(any(
                        target_os = "linux",
                        target_os = "macos",
                        target_os = "windows"
                    )) {
                        return Err(SecretError::SecureStoreUnavailable);
                    }
                    let entry = keyring::Entry::new("org.Regressee.session-key.v1", &account)
                        .map_err(|_| SecretError::SecureStoreUnavailable)?;
                    let bytes = match entry.get_secret() {
                        Ok(b) => Zeroizing::new(b),
                        Err(keyring::Error::NoEntry) => {
                            let key = Aes256Gcm::generate_key(&mut OsRng);
                            entry
                                .set_secret(&key)
                                .map_err(|_| SecretError::SecureStoreUnavailable)?;
                            let read = Zeroizing::new(
                                entry
                                    .get_secret()
                                    .map_err(|_| SecretError::SecureStoreUnavailable)?,
                            );
                            if read.as_slice() != key.as_slice() {
                                return Err(SecretError::SecureStoreUnavailable);
                            }
                            read
                        }
                        Err(_) => return Err(SecretError::SecureStoreUnavailable),
                    };
                    let mut key = Zeroizing::new([0; 32]);
                    if bytes.len() != 32 {
                        return Err(SecretError::Corrupt);
                    }
                    key.copy_from_slice(&bytes);
                    Ok(key)
                })();
                let _ = reply.send(result);
            }
        });
        Self { sender }
    }
}
impl KeyProvider for OsKeyProvider {
    fn key(&mut self) -> Result<Zeroizing<[u8; 32]>, SecretError> {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.sender
            .try_send(tx)
            .map_err(|_| SecretError::SecureStoreUnavailable)?;
        rx.recv_timeout(std::time::Duration::from_secs(5))
            .map_err(|_| SecretError::SecureStoreUnavailable)?
    }
}
pub struct EncryptedSessions {
    root: PathBuf,
    provider: Box<dyn KeyProvider>,
}
impl EncryptedSessions {
    pub fn open(root: &Path) -> Result<Self, SecretError> {
        prepare(root)?;
        let id_path = root.join("installation-id");
        let id = if id_path.exists() {
            let bytes = read_private(&id_path, 80)?;
            uuid::Uuid::parse_str(std::str::from_utf8(&bytes).map_err(|_| SecretError::Corrupt)?)
                .map_err(|_| SecretError::Corrupt)?
        } else {
            let id = uuid::Uuid::new_v4();
            // Never replace another cooperating instance's installation identity.
            let mut file = private_create(&id_path).map_err(|_| SecretError::Corrupt)?;
            file.write_all(id.to_string().as_bytes())
                .map_err(|_| SecretError::Corrupt)?;
            file.sync_all().map_err(|_| SecretError::Corrupt)?;
            id
        };
        Ok(Self {
            root: root.to_owned(),
            provider: Box::new(OsKeyProvider::new(id)),
        })
    }
    pub fn with_provider(root: &Path, provider: Box<dyn KeyProvider>) -> Result<Self, SecretError> {
        prepare(root)?;
        Ok(Self {
            root: root.to_owned(),
            provider,
        })
    }
    pub fn available(&mut self) -> Result<(), SecretError> {
        self.provider.key().map(|_| ())
    }
    fn path(&self, s: &SessionReference) -> Result<PathBuf, SecretError> {
        if s.id.0.is_nil() || s.project.0.is_nil() || s.environment.0.is_nil() {
            return Err(SecretError::ScopeMismatch);
        }
        Ok(self.root.join(format!("{}.session", s.id)))
    }
}
fn aad(s: &SessionReference) -> Vec<u8> {
    format!(
        "Regressee-session-v1:{}:{}:{}",
        s.project, s.environment, s.id
    )
    .into_bytes()
}
impl SessionSecrets for EncryptedSessions {
    fn save(&mut self, s: &SessionReference, m: &SessionMaterial) -> Result<(), SecretError> {
        if m.expose().len() > 512 * 1024 {
            return Err(SecretError::TooLarge);
        }
        let destination = self.path(s)?;
        let key = self.provider.key()?;
        let cipher = Aes256Gcm::new_from_slice(&*key).map_err(|_| SecretError::Corrupt)?;
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let encrypted = cipher
            .encrypt(
                &nonce,
                Payload {
                    msg: m.expose(),
                    aad: &aad(s),
                },
            )
            .map_err(|_| SecretError::Corrupt)?;
        let mut pending = tempfile::Builder::new()
            .prefix(".encrypted-pending-")
            .tempfile_in(&self.root)
            .map_err(|_| SecretError::Corrupt)?;
        pending
            .write_all(b"RGS1")
            .and_then(|_| pending.write_all(&nonce))
            .and_then(|_| pending.write_all(&encrypted))
            .and_then(|_| pending.as_file().sync_all())
            .map_err(|_| SecretError::Corrupt)?;
        // Session IDs are immutable; re-login receives a fresh ID.
        pending
            .persist_noclobber(destination)
            .map_err(|_| SecretError::Corrupt)?;
        #[cfg(unix)]
        fs::File::open(&self.root)
            .and_then(|f| f.sync_all())
            .map_err(|_| SecretError::Corrupt)?;
        Ok(())
    }
    fn load(&mut self, s: &SessionReference) -> Result<SessionMaterial, SecretError> {
        let bytes = read_private(&self.path(s)?, 512 * 1024 + 64)?;
        if bytes.len() < 32 || &bytes[..4] != b"RGS1" {
            return Err(SecretError::Corrupt);
        }
        let key = self.provider.key()?;
        let cipher = Aes256Gcm::new_from_slice(&*key).map_err(|_| SecretError::Corrupt)?;
        let plain = cipher
            .decrypt(
                Nonce::from_slice(&bytes[4..16]),
                Payload {
                    msg: &bytes[16..],
                    aad: &aad(s),
                },
            )
            .map_err(|_| SecretError::Corrupt)?;
        SessionMaterial::new(plain)
    }
    fn delete(&mut self, s: &SessionReference) -> Result<(), SecretError> {
        let path = self.path(s)?;
        // Identity AAD authenticates immutable ownership, not today's origins or
        // access policy. UUID/file existence alone never authorizes revocation.
        // Locked/missing keys and corrupt/foreign ciphertext fail without unlink.
        drop(self.load(s)?);
        fs::remove_file(path).map_err(|_| SecretError::Corrupt)
    }
}
fn prepare(root: &Path) -> Result<(), SecretError> {
    for p in root.ancestors().collect::<Vec<_>>().into_iter().rev() {
        match fs::symlink_metadata(p) {
            Ok(m) if m.is_dir() && regressee_core::platform::ordinary(&m) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            _ => return Err(SecretError::Corrupt),
        }
    }
    fs::create_dir_all(root).map_err(|_| SecretError::Corrupt)?;
    regressee_core::platform::protect(root).map_err(|_| SecretError::Corrupt)?;
    Ok(())
}
fn private_create(path: &Path) -> std::io::Result<fs::File> {
    let mut o = fs::OpenOptions::new();
    o.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        o.mode(0o600);
    }
    o.open(path)
}
fn read_private(path: &Path, limit: u64) -> Result<Vec<u8>, SecretError> {
    let m = fs::symlink_metadata(path).map_err(|e| {
        if e.kind() == std::io::ErrorKind::NotFound {
            SecretError::Missing
        } else {
            SecretError::Corrupt
        }
    })?;
    if !m.is_file() || !regressee_core::platform::ordinary(&m) || m.len() > limit {
        return Err(SecretError::Corrupt);
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if m.permissions().mode() & 0o077 != 0 {
            return Err(SecretError::Corrupt);
        }
    }
    let mut bytes = Vec::new();
    fs::File::open(path)
        .and_then(|f| f.take(limit + 1).read_to_end(&mut bytes))
        .map_err(|_| SecretError::Corrupt)?;
    if bytes.len() as u64 > limit {
        return Err(SecretError::TooLarge);
    }
    Ok(bytes)
}
/// Persistent implementations must encrypt with an OS-protected per-install key;
/// unavailable stores fail closed, offering ephemeral setup later, never JSON.
pub trait SessionSecrets: Send {
    fn load(&mut self, scope: &SessionReference) -> Result<SessionMaterial, SecretError>;
    fn save(
        &mut self,
        scope: &SessionReference,
        material: &SessionMaterial,
    ) -> Result<(), SecretError>;
    /// The caller must derive scope from trusted project/environment context;
    /// implementations authenticate ownership before deletion, never delete an
    /// installation-wide key, and fail closed when authentication is unavailable.
    fn delete(&mut self, scope: &SessionReference) -> Result<(), SecretError>;
}
pub struct UnavailableSecrets;
impl SessionSecrets for UnavailableSecrets {
    fn load(&mut self, _: &SessionReference) -> Result<SessionMaterial, SecretError> {
        Err(SecretError::SecureStoreUnavailable)
    }
    fn save(&mut self, _: &SessionReference, _: &SessionMaterial) -> Result<(), SecretError> {
        Err(SecretError::SecureStoreUnavailable)
    }
    fn delete(&mut self, _: &SessionReference) -> Result<(), SecretError> {
        Err(SecretError::SecureStoreUnavailable)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use regressee_core::domain::*;
    #[test]
    fn unavailable_store_fails_closed_and_debug_is_redacted() {
        let secret = SessionMaterial::new(b"never log this cookie".to_vec()).unwrap();
        assert!(!format!("{secret:?}").contains("cookie"));
        let scope = SessionReference {
            id: SessionId::new(),
            project: ProjectId::new(),
            environment: EnvironmentId::new(),
        };
        assert_eq!(
            UnavailableSecrets.save(&scope, &secret),
            Err(SecretError::SecureStoreUnavailable)
        );
    }
    struct TestKey([u8; 32]);
    impl KeyProvider for TestKey {
        fn key(&mut self) -> Result<Zeroizing<[u8; 32]>, SecretError> {
            Ok(Zeroizing::new(self.0))
        }
    }
    #[test]
    fn encrypted_restart_tamper_wrong_key_scope_and_unavailable_are_closed() {
        let root = tempfile::tempdir().unwrap();
        let scope = SessionReference {
            id: SessionId::new(),
            project: ProjectId::new(),
            environment: EnvironmentId::new(),
        };
        let plain = SessionMaterial::new(b"private-fixture-cookie-and-indexeddb".to_vec()).unwrap();
        let mut vault =
            EncryptedSessions::with_provider(root.path(), Box::new(TestKey([7; 32]))).unwrap();
        vault.save(&scope, &plain).unwrap();
        let file = root.path().join(format!("{}.session", scope.id));
        let bytes = fs::read(&file).unwrap();
        assert!(!bytes.windows(7).any(|b| b == b"private"));
        drop(vault);
        let mut restarted =
            EncryptedSessions::with_provider(root.path(), Box::new(TestKey([7; 32]))).unwrap();
        assert_eq!(restarted.load(&scope).unwrap().expose(), plain.expose());
        let wrong_scope = SessionReference {
            environment: EnvironmentId::new(),
            ..scope.clone()
        };
        assert!(matches!(
            restarted.load(&wrong_scope),
            Err(SecretError::Corrupt)
        ));
        let mut wrong =
            EncryptedSessions::with_provider(root.path(), Box::new(TestKey([8; 32]))).unwrap();
        assert!(matches!(wrong.load(&scope), Err(SecretError::Corrupt)));
        let mut bad = bytes;
        bad[20] ^= 1;
        fs::write(&file, bad).unwrap();
        assert!(matches!(restarted.load(&scope), Err(SecretError::Corrupt)));
        struct Missing;
        impl KeyProvider for Missing {
            fn key(&mut self) -> Result<Zeroizing<[u8; 32]>, SecretError> {
                Err(SecretError::SecureStoreUnavailable)
            }
        }
        let absent = SessionReference {
            id: SessionId::new(),
            ..scope
        };
        let mut unavailable =
            EncryptedSessions::with_provider(root.path(), Box::new(Missing)).unwrap();
        assert_eq!(
            unavailable.save(&absent, &plain),
            Err(SecretError::SecureStoreUnavailable)
        );
        assert!(!root.path().join(format!("{}.session", absent.id)).exists());
    }
    #[test]
    fn delete_authenticates_identity_and_preserves_foreign_locked_and_corrupt_state() {
        struct Locked;
        impl KeyProvider for Locked {
            fn key(&mut self) -> Result<Zeroizing<[u8; 32]>, SecretError> {
                Err(SecretError::SecureStoreUnavailable)
            }
        }
        let root = tempfile::tempdir().unwrap();
        let scope = SessionReference {
            id: SessionId::new(),
            project: ProjectId::new(),
            environment: EnvironmentId::new(),
        };
        let other = SessionReference {
            id: SessionId::new(),
            ..scope.clone()
        };
        let mut vault =
            EncryptedSessions::with_provider(root.path(), Box::new(TestKey([7; 32]))).unwrap();
        let material = SessionMaterial::new(b"private-delete-fixture".to_vec()).unwrap();
        vault.save(&scope, &material).unwrap();
        vault.save(&other, &material).unwrap();
        let file = root.path().join(format!("{}.session", scope.id));
        let other_file = root.path().join(format!("{}.session", other.id));
        let bytes = fs::read(&file).unwrap();
        let other_bytes = fs::read(&other_file).unwrap();
        for wrong in [
            SessionReference {
                project: ProjectId::new(),
                ..scope.clone()
            },
            SessionReference {
                environment: EnvironmentId::new(),
                ..scope.clone()
            },
        ] {
            assert_eq!(vault.delete(&wrong), Err(SecretError::Corrupt));
            assert_eq!(fs::read(&file).unwrap(), bytes);
        }
        let mut locked = EncryptedSessions::with_provider(root.path(), Box::new(Locked)).unwrap();
        assert_eq!(
            locked.delete(&scope),
            Err(SecretError::SecureStoreUnavailable)
        );
        assert_eq!(fs::read(&file).unwrap(), bytes);
        let mut bad = bytes.clone();
        bad[20] ^= 1;
        fs::write(&file, &bad).unwrap();
        assert_eq!(vault.delete(&scope), Err(SecretError::Corrupt));
        assert_eq!(fs::read(&file).unwrap(), bad);
        fs::write(&file, &bytes).unwrap();
        vault.delete(&scope).unwrap();
        assert!(!file.exists());
        assert_eq!(fs::read(&other_file).unwrap(), other_bytes);
        assert_eq!(vault.load(&other).unwrap().expose(), material.expose());
        assert_eq!(vault.delete(&scope), Err(SecretError::Missing));
    }
    #[test]
    #[ignore = "bounded real OS-store availability; touches only one fresh Regressee-owned entry"]
    fn actual_owned_os_store_availability() {
        let id = uuid::Uuid::new_v4();
        let mut provider = OsKeyProvider::new(id);
        match provider.key() {
            Ok(first) => {
                let second = provider.key().unwrap();
                assert_eq!(*first, *second);
                let entry = keyring::Entry::new(
                    "org.Regressee.session-key.v1",
                    &format!("installation-{id}"),
                )
                .unwrap();
                entry.delete_credential().unwrap();
                println!("Regressee-owned OS secure-store write/read/delete: PASS");
            }
            Err(SecretError::SecureStoreUnavailable) => println!(
                "OS secure store unavailable: EPHEMERAL required; persistence qualification NOT PASSED"
            ),
            Err(e) => panic!("owned secure-store check failed: {e}"),
        }
    }
}
