// SPDX-License-Identifier: MPL-2.0
//! Machine-local application appearance, independent of website capture profiles.
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum AppAppearance {
    #[default]
    System,
    Light,
    Dark,
}

impl AppAppearance {
    pub const ALL: [Self; 3] = [Self::System, Self::Light, Self::Dark];

    pub fn label(self) -> &'static str {
        match self {
            Self::System => "System",
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn is_dark(self, desktop_is_dark: bool) -> bool {
        match self {
            Self::System => desktop_is_dark,
            Self::Light => false,
            Self::Dark => true,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn system_follows_desktop_and_overrides_do_not() {
        for desktop in [false, true] {
            assert_eq!(AppAppearance::System.is_dark(desktop), desktop);
            assert!(!AppAppearance::Light.is_dark(desktop));
            assert!(AppAppearance::Dark.is_dark(desktop));
        }
        assert_eq!(AppAppearance::default(), AppAppearance::System);
    }
}
