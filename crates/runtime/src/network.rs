// SPDX-License-Identifier: MPL-2.0
//! Machine-local managed-network interfaces. Never disable TLS verification.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
};
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NetworkSettings {
    pub proxy: Option<String>,
    pub trusted_ca: Option<PathBuf>,
}
impl NetworkSettings {
    pub fn validate(&self) -> Result<()> {
        if let Some(s) = &self.proxy {
            let u = url::Url::parse(s)?;
            regressee_core::domain::validate_address(&u)?;
            ensure!(
                u.query().is_none() && u.fragment().is_none() && u.path() == "/",
                "Proxy must be an http(s) origin, without credentials, path or query"
            );
        }
        if let Some(p) = &self.trusted_ca {
            let m = fs::symlink_metadata(p)?;
            ensure!(
                m.is_file() && regressee_core::platform::ordinary(&m) && m.len() <= 1024 * 1024,
                "Trusted CA must be a regular PEM file up to 1 MiB"
            );
            reqwest::Certificate::from_pem(&fs::read(p)?)?;
        }
        Ok(())
    }
    pub fn load(path: &Path) -> Result<Self> {
        match fs::symlink_metadata(path) {
            Ok(m) => {
                ensure!(
                    m.is_file() && regressee_core::platform::ordinary(&m) && m.len() <= 16384,
                    "Unsafe network configuration"
                );
                let s: Self = serde_json::from_slice(&fs::read(path)?)?;
                s.validate()?;
                Ok(s)
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Self::default()),
            Err(e) => Err(e.into()),
        }
    }
    pub fn save(&self, path: &Path) -> Result<()> {
        self.validate()?;
        match fs::symlink_metadata(path) {
            Ok(m) => ensure!(
                m.is_file() && regressee_core::platform::ordinary(&m),
                "Unsafe network configuration"
            ),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
        let parent = path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("Network settings have no parent"))?;
        let mut f = tempfile::NamedTempFile::new_in(parent)?;
        use std::io::Write;
        f.write_all(&serde_json::to_vec(self)?)?;
        f.as_file().sync_all()?;
        f.persist(path)?;
        Ok(())
    }
    pub fn client(&self) -> Result<reqwest::Client> {
        self.validate()?;
        let mut b = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .connect_timeout(std::time::Duration::from_secs(20))
            .timeout(std::time::Duration::from_secs(120));
        if let Some(proxy) = &self.proxy {
            b = b.proxy(reqwest::Proxy::all(proxy)?);
        }
        if let Some(p) = &self.trusted_ca {
            b = b.add_root_certificate(reqwest::Certificate::from_pem(&fs::read(p)?)?);
        }
        Ok(b.build()?)
    }
    pub fn diagnostics(&self) -> String {
        format!(
            "{}; {}. TLS verification stays enabled. Browser custom CA must also be trusted by Chromium/your OS; a PEM here configures downloads and Node only. Proxy authentication and automatic VPN setup are not supported. VPN uses your existing OS network; enterprise qualification is pending.",
            if self.proxy.is_some() {
                "Configured proxy for runtime downloads and Chromium"
            } else {
                "Existing OS network; no configured proxy"
            },
            if self.trusted_ca.is_some() {
                "Custom download/Node CA configured"
            } else {
                "Default trusted CAs"
            }
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    #[test]
    fn linked_configuration_and_ca_are_refused_without_mutation() -> Result<()> {
        let root = tempfile::tempdir()?;
        let outside = root.path().join("outside.json");
        let bytes = br#"{"proxy":null,"trusted_ca":null}"#;
        fs::write(&outside, bytes)?;
        let link = root.path().join("network.json");
        std::os::unix::fs::symlink(&outside, &link)?;
        assert!(NetworkSettings::load(&link).is_err());
        assert!(NetworkSettings::default().save(&link).is_err());
        assert!(
            NetworkSettings {
                proxy: None,
                trusted_ca: Some(link.clone())
            }
            .validate()
            .is_err()
        );
        assert_eq!(fs::read(&outside)?, bytes);
        assert!(fs::symlink_metadata(link)?.file_type().is_symlink());
        Ok(())
    }
    #[test]
    fn configured_proxy_is_used_by_real_download_client_and_remains_local() -> Result<()> {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            sync::{Arc, Mutex},
            time::Duration,
        };
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let proxy = format!("http://{}/", listener.local_addr()?);
        let observed = Arc::new(Mutex::new(String::new()));
        let recorded = observed.clone();
        let thread = std::thread::spawn(move || -> std::io::Result<()> {
            let (mut socket, _) = listener.accept()?;
            socket.set_read_timeout(Some(Duration::from_secs(5)))?;
            let mut bytes = [0; 8192];
            let n = socket.read(&mut bytes)?;
            *recorded.lock().unwrap() = String::from_utf8_lossy(&bytes[..n]).into();
            socket.write_all(
                b"HTTP/1.1 200 OK\r\nContent-Length: 7\r\nConnection: close\r\n\r\nfixture",
            )?;
            Ok(())
        });
        let root = tempfile::tempdir()?;
        let config = NetworkSettings {
            proxy: Some(proxy),
            trusted_ca: None,
        };
        let path = root.path().join("network.json");
        config.save(&path)?;
        let loaded = NetworkSettings::load(&path)?;
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()?;
        let body = runtime.block_on(async {
            loaded
                .client()?
                .get("http://127.0.0.1:9/owned-runtime.zip")
                .send()
                .await?
                .text()
                .await
                .map_err(anyhow::Error::from)
        })?;
        assert_eq!(body, "fixture");
        thread.join().unwrap()?;
        assert!(
            observed
                .lock()
                .unwrap()
                .starts_with("GET http://127.0.0.1:9/owned-runtime.zip HTTP/1.1")
        );
        assert!(!loaded.diagnostics().contains(&loaded.proxy.unwrap()));
        Ok(())
    }
    #[test]
    fn network_credentials_and_tls_bypasses_are_not_interfaces() {
        for proxy in [
            "http://user:password@proxy.test/",
            "http://proxy.test/path",
            "http://proxy.test/?ignore_tls=true",
        ] {
            assert!(
                NetworkSettings {
                    proxy: Some(proxy.into()),
                    trusted_ca: None
                }
                .validate()
                .is_err()
            );
        }
        assert!(
            serde_json::from_str::<NetworkSettings>(
                r#"{"proxy":null,"trusted_ca":null,"ignore_https_errors":true}"#
            )
            .is_err()
        );
    }
}
