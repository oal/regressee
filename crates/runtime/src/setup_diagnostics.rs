// SPDX-License-Identifier: MPL-2.0
//! Local setup failures only. Never include request URLs or browser/site stacks.
use crate::distribution::InstallProgress;
use std::{error::Error, fmt, time::Instant};

pub const FAILURE_PREFIX: &str = "Browser setup failed during ";

pub struct SetupAttempt {
    started: Instant,
    phase: Option<InstallProgress>,
    downloaded: Option<(u64, Option<u64>)>,
}

impl Default for SetupAttempt {
    fn default() -> Self {
        Self {
            started: Instant::now(),
            phase: None,
            downloaded: None,
        }
    }
}

impl SetupAttempt {
    pub fn observe(&mut self, phase: InstallProgress) {
        self.phase = Some(phase);
        if let InstallProgress::Downloading { bytes, total } = phase {
            self.downloaded = Some((bytes, total));
        }
    }

    pub fn failure(self, source: anyhow::Error) -> SetupFailure {
        SetupFailure {
            phase: self.phase,
            downloaded: self.downloaded,
            elapsed_seconds: self.started.elapsed().as_secs(),
            source,
        }
    }
}

#[derive(Debug)]
pub struct SetupFailure {
    phase: Option<InstallProgress>,
    downloaded: Option<(u64, Option<u64>)>,
    elapsed_seconds: u64,
    source: anyhow::Error,
}

impl fmt::Display for SetupFailure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let stage = match self.phase {
            None => "preparation",
            Some(InstallProgress::Downloading { .. }) => "download",
            Some(InstallProgress::Verifying) => "file verification",
            Some(InstallProgress::Extracting) => "file installation",
            Some(InstallProgress::TestingBrowser) => "browser startup test",
            Some(InstallProgress::Publishing) => "saving the verified installation",
            Some(InstallProgress::Ready) => "final readiness check",
        };
        let recovery = match self.phase {
            None => "Check Managed network and folder access in App settings before retrying.",
            Some(InstallProgress::Downloading { .. }) => {
                "Check your internet connection and Managed network settings, then retry browser setup."
            }
            Some(InstallProgress::Verifying) => {
                "Retry browser setup. If verification fails again, keep these details for support; do not bypass the file checks."
            }
            Some(InstallProgress::Extracting) => {
                "Check free disk space and cache-folder permissions in App settings before retrying."
            }
            Some(InstallProgress::TestingBrowser) => {
                "Check that your system meets the browser requirements. If the test fails again, keep these details for support; do not disable browser security."
            }
            Some(InstallProgress::Publishing | InstallProgress::Ready) => {
                "Close other Regressee windows and check cache-folder permissions and free space before retrying browser setup."
            }
        };
        writeln!(f, "{FAILURE_PREFIX}{stage}. {recovery}")?;
        writeln!(f, "Elapsed: {} seconds.", self.elapsed_seconds)?;
        if let Some((bytes, total)) = self.downloaded {
            match total {
                Some(total) => writeln!(f, "Download received: {bytes} of {total} bytes.")?,
                None => writeln!(f, "Download received: {bytes} bytes (total unknown).")?,
            }
        }
        writeln!(
            f,
            "Platform: {} {}.",
            std::env::consts::OS,
            std::env::consts::ARCH
        )?;
        // reqwest's Display includes the full request URL, which can contain
        // private override paths/queries. Keep its typed reason and underlying
        // transport/TLS/OS causes instead. Do not change other operations' errors.
        for (index, cause) in self.source.chain().take(16).enumerate() {
            let label = if index == 0 { "Error" } else { "Caused by" };
            if let Some(network) = cause.downcast_ref::<reqwest::Error>() {
                if let Some(status) = network.status() {
                    writeln!(f, "{label}: download server returned HTTP {status}.")?;
                } else {
                    let reason = if network.is_timeout() {
                        "network request timed out"
                    } else if network.is_connect() {
                        "could not connect to the download server"
                    } else if network.is_body() || network.is_decode() {
                        "could not read the download response"
                    } else if network.is_builder() {
                        "download network configuration is invalid"
                    } else {
                        "download request failed"
                    };
                    writeln!(f, "{label}: {reason}.")?;
                }
            } else {
                // Bound external TLS/OS text and strip control characters. Local
                // file paths can still appear; this is not an exported report.
                let text: String = cause
                    .to_string()
                    .chars()
                    .take(1024)
                    .map(|c| if c.is_control() { ' ' } else { c })
                    .collect();
                writeln!(f, "{label}: {text}")?;
            }
        }
        write!(
            f,
            "Existing projects and screenshots are unchanged. Keep these details before retrying; no diagnostic was uploaded."
        )
    }
}

impl Error for SetupFailure {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        Some(self.source.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[test]
    fn setup_failure_keeps_stage_download_bytes_and_nested_os_cause() {
        let mut attempt = SetupAttempt::default();
        attempt.observe(InstallProgress::Downloading {
            bytes: 123,
            total: Some(456),
        });
        attempt.observe(InstallProgress::Extracting);
        let source = Err::<(), _>(std::io::Error::from_raw_os_error(5))
            .context("Creating runtime file node/node.exe")
            .unwrap_err();
        let message = attempt.failure(source).to_string();
        assert!(message.starts_with("Browser setup failed during file installation."));
        assert!(message.contains("123 of 456 bytes"));
        assert!(message.contains("Creating runtime file node/node.exe"));
        assert!(message.contains("os error 5"));
        assert!(message.contains("Platform:"));
        assert!(message.contains("Check free disk space and cache-folder permissions"));
    }

    #[test]
    fn preparation_publication_and_unknown_download_totals_are_distinct() {
        let message = SetupAttempt::default()
            .failure(anyhow::anyhow!("Not enough space"))
            .to_string();
        assert!(message.starts_with("Browser setup failed during preparation."));
        assert!(!message.contains("Download received:"));
        let mut attempt = SetupAttempt::default();
        attempt.observe(InstallProgress::Downloading {
            bytes: 80,
            total: None,
        });
        attempt.observe(InstallProgress::Publishing);
        let message = attempt.failure(anyhow::anyhow!("File locked")).to_string();
        assert!(message.contains("saving the verified installation"));
        assert!(message.contains("80 bytes (total unknown)"));
        assert!(message.contains("Close other Regressee windows"));
    }

    #[test]
    fn network_status_survives_without_request_credentials_paths_or_queries() {
        use std::{
            io::{Read, Write},
            net::TcpListener,
            time::Duration,
        };
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!(
            "http://user:secret@{}/private?token=hidden",
            listener.local_addr().unwrap()
        );
        let server = std::thread::spawn(move || {
            let (mut socket, _) = listener.accept().unwrap();
            socket
                .set_read_timeout(Some(Duration::from_secs(5)))
                .unwrap();
            let mut request = [0; 4096];
            assert!(socket.read(&mut request).unwrap() > 0);
            socket.write_all(b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n").unwrap();
        });
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let error = runtime.block_on(async {
            reqwest::Client::builder()
                .no_proxy()
                .timeout(Duration::from_secs(5))
                .build()
                .unwrap()
                .get(url)
                .send()
                .await
                .unwrap()
                .error_for_status()
                .unwrap_err()
        });
        server.join().unwrap();
        let message = SetupAttempt::default().failure(error.into()).to_string();
        assert!(message.contains("HTTP 503 Service Unavailable"));
        for private in ["secret", "127.0.0.1", "/private", "token=hidden"] {
            assert!(!message.contains(private));
        }
    }
}
