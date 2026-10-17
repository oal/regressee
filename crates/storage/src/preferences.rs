// SPDX-License-Identifier: MPL-2.0
//! Small machine-local preferences. Never included in baseline ZIPs or captures.
use super::*;
use regressee_core::appearance::AppAppearance;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppPreferences {
    pub appearance: AppAppearance,
}

impl AppPreferences {
    fn path(paths: &AppPaths) -> PathBuf {
        paths.data.join("app-preferences.json")
    }

    pub fn load(paths: &AppPaths) -> Result<Self> {
        validate_directory_path(&paths.data)?;
        let path = Self::path(paths);
        let Some(metadata) = entry_metadata(&path)? else {
            return Ok(Self::default());
        };
        ensure!(
            metadata.is_file()
                && regressee_core::platform::ordinary(&metadata)
                && metadata.len() <= 4096,
            "Unsafe app preferences file"
        );
        Ok(serde_json::from_slice(&fs::read(path)?)?)
    }

    pub fn save(&self, paths: &AppPaths) -> Result<()> {
        private_dir(&paths.data)?;
        let path = Self::path(paths);
        if let Some(metadata) = entry_metadata(&path)? {
            ensure!(
                metadata.is_file() && regressee_core::platform::ordinary(&metadata),
                "Unsafe app preferences file"
            );
        }
        let mut file = tempfile::NamedTempFile::new_in(&paths.data)?;
        regressee_core::platform::protect(file.path())?;
        file.write_all(&serde_json::to_vec(self)?)?;
        file.as_file().sync_all()?;
        file.persist(path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn preferences_default_and_round_trip_without_creating_project_data() -> Result<()> {
        let root = tempfile::tempdir()?;
        let paths = AppPaths::isolated(root.path());
        assert_eq!(AppPreferences::load(&paths)?, AppPreferences::default());
        for appearance in AppAppearance::ALL {
            let preferences = AppPreferences { appearance };
            preferences.save(&paths)?;
            assert_eq!(AppPreferences::load(&paths)?, preferences);
        }
        assert!(!paths.database().exists());
        assert!(!paths.artifacts().exists());
        fs::write(AppPreferences::path(&paths), br#"{"appearance":"unknown"}"#)?;
        assert!(AppPreferences::load(&paths).is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn linked_preferences_and_linked_parent_are_not_followed() -> Result<()> {
        let root = tempfile::tempdir()?;
        let paths = AppPaths::isolated(root.path());
        private_dir(&paths.data)?;
        let outside = root.path().join("outside.json");
        fs::write(&outside, b"untouched")?;
        std::os::unix::fs::symlink(&outside, AppPreferences::path(&paths))?;
        assert!(AppPreferences::load(&paths).is_err());
        assert!(AppPreferences::default().save(&paths).is_err());
        assert_eq!(fs::read(&outside)?, b"untouched");
        let linked = AppPaths {
            data: root.path().join("linked"),
            cache: paths.cache,
        };
        std::os::unix::fs::symlink(&paths.data, &linked.data)?;
        assert!(AppPreferences::load(&linked).is_err());
        assert!(AppPreferences::default().save(&linked).is_err());
        Ok(())
    }
}
