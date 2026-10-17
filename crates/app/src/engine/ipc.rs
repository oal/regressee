// SPDX-License-Identifier: MPL-2.0
use crate::protocol::MAX_FRAME;
use anyhow::{Context, Result, ensure};
use regressee_storage::AppPaths;
use sha2::{Digest, Sha256};
use std::time::Duration;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
pub trait Io: AsyncRead + AsyncWrite + Unpin + Send {}
impl<T: AsyncRead + AsyncWrite + Unpin + Send> Io for T {}
pub type Stream = Box<dyn Io>;
pub const IO_TIMEOUT: Duration = Duration::from_secs(5);
pub fn key(paths: &AppPaths) -> String {
    hex::encode(Sha256::digest(paths.data.as_os_str().as_encoded_bytes()))
}

pub async fn read<T: serde::de::DeserializeOwned>(io: &mut Stream) -> Result<T> {
    tokio::time::timeout(IO_TIMEOUT, async {
        let size = io.read_u32().await? as usize;
        ensure!(size > 0 && size <= MAX_FRAME, "IPC frame exceeds bounds");
        let mut bytes = vec![0; size];
        io.read_exact(&mut bytes).await?;
        Ok(serde_json::from_slice(&bytes)?)
    })
    .await
    .context("IPC read deadline exceeded")?
}
pub async fn write<T: serde::Serialize>(io: &mut Stream, value: &T) -> Result<()> {
    let bytes = serde_json::to_vec(value)?;
    ensure!(bytes.len() <= MAX_FRAME, "IPC response exceeds bounds");
    tokio::time::timeout(IO_TIMEOUT, async {
        io.write_u32(bytes.len() as u32).await?;
        io.write_all(&bytes).await?;
        io.flush().await?;
        Ok(())
    })
    .await
    .context("IPC write deadline exceeded")?
}

#[cfg(unix)]
pub struct Listener {
    listener: tokio::net::UnixListener,
    path: Option<std::path::PathBuf>,
}
#[cfg(unix)]
fn address(paths: &AppPaths) -> Result<std::os::unix::net::SocketAddr> {
    #[cfg(target_os = "linux")]
    {
        use std::os::linux::net::SocketAddrExt;
        // Abstract addresses have no pathname-length dependency or stale socket
        // files. Both endpoints independently verify the kernel peer UID.
        let name = format!(
            "regressee-engine-{}-{}",
            unsafe { libc::geteuid() },
            key(paths)
        );
        Ok(std::os::unix::net::SocketAddr::from_abstract_name(name)?)
    }
    #[cfg(not(target_os = "linux"))]
    {
        Ok(std::os::unix::net::SocketAddr::from_pathname(
            paths.data.join("engine-v1/socket"),
        )?)
    }
}
#[cfg(unix)]
fn peer(stream: &tokio::net::UnixStream) -> Result<()> {
    ensure!(
        stream.peer_cred()?.uid() == unsafe { libc::geteuid() },
        "IPC peer belongs to another user"
    );
    Ok(())
}
#[cfg(unix)]
impl Listener {
    pub fn bind(paths: &AppPaths) -> Result<Self> {
        #[cfg(not(target_os = "linux"))]
        let path = {
            let p = paths.data.join("engine-v1/socket");
            if let Ok(m) = std::fs::symlink_metadata(&p) {
                use std::os::unix::fs::{FileTypeExt, MetadataExt};
                ensure!(
                    m.file_type().is_socket() && m.uid() == unsafe { libc::geteuid() },
                    "Unsafe stale IPC endpoint"
                );
                std::fs::remove_file(&p)?; // Exclusive data lock is already held.
            }
            Some(p)
        };
        #[cfg(target_os = "linux")]
        let path = None;
        let listener = std::os::unix::net::UnixListener::bind_addr(&address(paths)?)?;
        listener.set_nonblocking(true)?;
        if let Some(p) = &path {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(p, std::fs::Permissions::from_mode(0o600))?;
        }
        Ok(Self {
            listener: tokio::net::UnixListener::from_std(listener)?,
            path,
        })
    }
    pub async fn accept(&mut self) -> Result<Stream> {
        let (stream, _) = self.listener.accept().await?;
        peer(&stream)?;
        Ok(Box::new(stream))
    }
}
#[cfg(unix)]
impl Drop for Listener {
    fn drop(&mut self) {
        if let Some(p) = &self.path {
            let _ = std::fs::remove_file(p);
        }
    }
}
#[cfg(unix)]
pub async fn connect(paths: &AppPaths) -> Result<Stream> {
    #[cfg(target_os = "linux")]
    let path = {
        use std::os::unix::ffi::OsStringExt;
        std::path::PathBuf::from(std::ffi::OsString::from_vec(
            format!(
                "\0regressee-engine-{}-{}",
                unsafe { libc::geteuid() },
                key(paths)
            )
            .into_bytes(),
        ))
    };
    #[cfg(not(target_os = "linux"))]
    let path = paths.data.join("engine-v1/socket");
    let stream = tokio::time::timeout(IO_TIMEOUT, tokio::net::UnixStream::connect(path))
        .await
        .context("IPC connect deadline exceeded")??;
    peer(&stream)?;
    Ok(Box::new(stream))
}

#[cfg(windows)]
#[path = "windows.rs"]
mod windows;
#[cfg(windows)]
pub use windows::{Listener, connect};
