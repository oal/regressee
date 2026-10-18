// SPDX-License-Identifier: MPL-2.0
//! Small machine-local preferences. Never included in baseline ZIPs or captures.
use super::*;
use regressee_core::appearance::AppAppearance;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct AppPreferences {
    pub appearance: AppAppearance,
    pub reduced_motion: bool,
}

/// Partial machine-local update: changing one choice never resets the other.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct PreferencesUpdate {
    pub appearance: Option<AppAppearance>,
    pub reduced_motion: Option<bool>,
}

impl AppPreferences {
    /// Called by the serial preference owner (or under the engine state lock).
    pub fn update(paths: &AppPaths, update: PreferencesUpdate) -> Result<Self> {
        // Explicit choices can repair an unreadable preference file, matching
        // startup's reported defaults; unsafe paths are still rejected by save.
        let mut preferences = Self::load(paths).unwrap_or_default();
        if let Some(appearance) = update.appearance {
            preferences.appearance = appearance;
        }
        if let Some(reduced_motion) = update.reduced_motion {
            preferences.reduced_motion = reduced_motion;
        }
        preferences.save(paths)?;
        Ok(preferences)
    }

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
            let preferences = AppPreferences {
                appearance,
                reduced_motion: true,
            };
            preferences.save(&paths)?;
            assert_eq!(AppPreferences::load(&paths)?, preferences);
        }
        assert!(!paths.database().exists());
        assert!(!paths.artifacts().exists());
        fs::write(AppPreferences::path(&paths), br#"{"appearance":"unknown"}"#)?;
        assert!(AppPreferences::load(&paths).is_err());
        Ok(())
    }

    #[test]
    fn old_preferences_default_motion_and_partial_updates_preserve_other_choices() -> Result<()> {
        let root = tempfile::tempdir()?;
        let paths = AppPaths::isolated(root.path());
        private_dir(&paths.data)?;
        fs::write(AppPreferences::path(&paths), br#"{"appearance":"dark"}"#)?;
        assert_eq!(
            AppPreferences::load(&paths)?,
            AppPreferences {
                appearance: AppAppearance::Dark,
                reduced_motion: false
            }
        );
        let motion = AppPreferences::update(
            &paths,
            PreferencesUpdate {
                reduced_motion: Some(true),
                ..Default::default()
            },
        )?;
        assert_eq!(motion.appearance, AppAppearance::Dark);
        let appearance = AppPreferences::update(
            &paths,
            PreferencesUpdate {
                appearance: Some(AppAppearance::Light),
                ..Default::default()
            },
        )?;
        assert!(appearance.reduced_motion);
        assert_eq!(AppPreferences::load(&paths)?, appearance);
        assert!(!paths.database().exists());
        assert!(!paths.artifacts().exists());
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
