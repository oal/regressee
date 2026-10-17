// SPDX-License-Identifier: MPL-2.0
//! Restrained native elevation. Apply only to opaque surfaces, never the
//! transparent viewer/captions or individual rows in a continuous list.
use super::theme::palette;
use gpui::{Background, BoxShadow, point, px, rgb, rgba};

#[derive(Clone, Copy, Debug)]
pub enum Elevation {
    Control,
    Surface,
    Floating,
    Dialog,
    Header,
    Footer,
    Rail,
}

impl Elevation {
    pub fn shadows(self) -> Vec<BoxShadow> {
        let shadow = |x, y, blur, alpha| BoxShadow {
            color: rgba((palette().shadow << 8) | alpha).into(),
            offset: point(px(x), px(y)),
            blur_radius: px(blur),
            spread_radius: px(0.),
        };
        match self {
            Self::Control => vec![shadow(0., 1., 2., 0x18)],
            Self::Surface => vec![shadow(0., 2., 6., 0x14), shadow(0., 1., 2., 0x0c)],
            Self::Floating => vec![shadow(0., 8., 24., 0x26), shadow(0., 2., 6., 0x18)],
            Self::Dialog => vec![shadow(0., 18., 48., 0x38), shadow(0., 4., 12., 0x24)],
            Self::Header => vec![shadow(0., 2., 6., 0x10)],
            Self::Footer => vec![shadow(0., -2., 6., 0x10)],
            Self::Rail => vec![shadow(2., 0., 6., 0x1c)],
        }
    }
}

/// One flat theme surface for project headers and inspector/disclosure chrome.
/// Hierarchy comes from surface lightness, rules and restrained edge shadows,
/// never a separately colored gradient band.
pub fn chrome() -> Background {
    rgb(palette().surface).into()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chrome_is_a_solid_semantic_surface_in_both_themes() {
        use crate::ui::theme::{is_dark, set_dark};
        let original = is_dark();
        for dark in [false, true] {
            set_dark(dark);
            assert_eq!(chrome(), Background::from(rgb(palette().surface)));
        }
        set_dark(original);
    }

    #[test]
    fn elevation_is_soft_directional_and_never_a_focus_halo() {
        for role in [
            Elevation::Control,
            Elevation::Surface,
            Elevation::Floating,
            Elevation::Dialog,
            Elevation::Header,
            Elevation::Footer,
            Elevation::Rail,
        ] {
            for shadow in role.shadows() {
                assert!(shadow.blur_radius > px(0.), "{role:?}");
                assert_ne!(shadow.offset, point(px(0.), px(0.)), "{role:?}");
                assert_eq!(shadow.spread_radius, px(0.));
                assert!(shadow.color.a > 0. && shadow.color.a < 0.25);
            }
        }
        assert!(
            Elevation::Control.shadows()[0].blur_radius
                < Elevation::Surface.shadows()[0].blur_radius
        );
        assert!(
            Elevation::Surface.shadows()[0].blur_radius
                < Elevation::Floating.shadows()[0].blur_radius
        );
        assert!(
            Elevation::Floating.shadows()[0].blur_radius
                < Elevation::Dialog.shadows()[0].blur_radius
        );
    }
}
