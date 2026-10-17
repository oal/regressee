// SPDX-License-Identifier: MPL-2.0
//! Normal builds use only the OS application directories.
use anyhow::Result;
use regressee_storage::AppPaths;

/// Public directory selection, shared by both installed entrypoints. Explicit
/// paths never consult developer overrides and need no HOME when both are set.
pub fn configured_paths(
    data: Option<std::path::PathBuf>,
    cache: Option<std::path::PathBuf>,
) -> Result<AppPaths> {
    if let Some(data) = data {
        Ok(AppPaths {
            cache: cache.unwrap_or_else(|| data.join("cache")),
            data,
        })
    } else {
        let mut paths = AppPaths::discover()?;
        if let Some(cache) = cache {
            paths.cache = cache;
        }
        Ok(paths)
    }
}

/// Read only the desktop's public directory arguments; native inspection flags
/// remain owned by its existing launch path. No storage is opened here.
pub fn desktop_paths(args: &[std::ffi::OsString]) -> Result<AppPaths> {
    let mut data = None;
    let mut cache = None;
    let mut i = 0;
    while i < args.len() {
        let slot = if args[i] == "--data-dir" {
            Some(&mut data)
        } else if args[i] == "--cache-dir" {
            Some(&mut cache)
        } else {
            None
        };
        if let Some(slot) = slot {
            anyhow::ensure!(slot.is_none(), "Duplicate public directory flag");
            i += 1;
            let value = args
                .get(i)
                .ok_or_else(|| anyhow::anyhow!("Public directory flag requires a path"))?;
            anyhow::ensure!(
                !value.to_string_lossy().starts_with("--"),
                "Public directory flag requires a path"
            );
            *slot = Some(std::path::PathBuf::from(value));
        }
        i += 1;
    }
    if data.is_some() || cache.is_some() {
        configured_paths(data, cache)
    } else {
        app_paths()
    }
}

pub fn app_paths() -> Result<AppPaths> {
    #[cfg(feature = "developer-tools")]
    if let Some(root) = std::env::var_os("REGRESSEE_DEV_DATA") {
        return Ok(AppPaths::isolated(std::path::Path::new(&root)));
    }
    AppPaths::discover()
}

/// Data-free executable metadata, available before graphics or storage startup.
pub fn build_info() -> serde_json::Value {
    serde_json::json!({
        "build_info_schema": 1,
        "version": env!("CARGO_PKG_VERSION"),
        "engine_protocol": crate::protocol::VERSION,
        "cli_schema": 1,
        "features": {
            "ui": false,
            "developer-tools": cfg!(feature = "developer-tools"),
            "test-support": cfg!(feature = "test-support"),
            "ui-inspection": false,
            "ui-renderer-capture": false,
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn explicit_public_directories_are_shared_and_data_free() -> Result<()> {
        let root = tempfile::tempdir()?;
        let data = root.path().join("not-created-data");
        let cache = root.path().join("not-created-cache");
        let args = vec![
            "--data-dir".into(),
            data.clone().into_os_string(),
            "--cache-dir".into(),
            cache.clone().into_os_string(),
        ];
        let cli = configured_paths(Some(data.clone()), Some(cache.clone()))?;
        let desktop = desktop_paths(&args)?;
        assert_eq!((cli.data, cli.cache), (desktop.data, desktop.cache));
        assert!(!data.exists() && !cache.exists());
        assert_eq!(
            configured_paths(Some(data.clone()), None)?.cache,
            data.join("cache")
        );
        assert!(desktop_paths(&["--data-dir".into()]).is_err());
        assert!(
            desktop_paths(&[
                "--data-dir".into(),
                data.clone().into_os_string(),
                "--data-dir".into(),
                data.into_os_string()
            ])
            .is_err()
        );
        Ok(())
    }
}
