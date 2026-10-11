// SPDX-License-Identifier: MPL-2.0
//! User-selected Clear wayfinding palette; shared control metrics live here.
//! System UI type, light working surfaces and dark project navigation.
pub const GROUND: u32 = 0xf6fafd;
pub const SURFACE: u32 = 0xffffff;
pub const RAIL: u32 = 0x183f65;
pub const RAIL_INK: u32 = 0xffffff;
pub const RAIL_MUTED: u32 = 0xd7e8f5;
pub const RAIL_HOVER: u32 = 0x23517a;
pub const INK: u32 = 0x22313d;
pub const MUTED: u32 = 0x536778;
pub const LINE: u32 = 0xced9e2;
pub const CONTROL_LINE: u32 = 0x7e93a6;
pub const ACTION_SURFACE: u32 = 0xf7fafc;
pub const ACTION_HOVER: u32 = 0xe7f2fa;
pub const ACCENT: u32 = 0x1765ac;
pub const ACCENT_HOVER: u32 = 0x12558f;
pub const SELECTED: u32 = 0xe7f2fa;
pub const ERROR: u32 = 0x9c3038;
pub const ERROR_SURFACE: u32 = 0xfff2f2;
pub const WARNING: u32 = 0x775311;
pub const WARNING_SURFACE: u32 = 0xfff7df;

// Shared control metrics. State refinements must never change these values.
pub const CONTROL_HEIGHT: f32 = 40.;
pub const CONTROL_TEXT_SIZE: f32 = 14.;
pub const CONTROL_LINE_HEIGHT: f32 = 20.;
pub const HELP_TEXT_SIZE: f32 = 13.;
pub const HELP_LINE_HEIGHT: f32 = 18.;
pub const CHOICE_INSET: f32 = 12.;
pub const PAGE_ROW_HEIGHT: f32 = 64.;

pub const FONT: &str = if cfg!(target_os = "windows") {
    "Segoe UI"
} else if cfg!(target_os = "macos") {
    ".AppleSystemUIFont"
} else {
    "DejaVu Sans"
};

#[cfg(test)]
pub(super) fn contrast(foreground: u32, background: u32) -> f64 {
    fn luminance(color: u32) -> f64 {
        let channel = |shift: u32| {
            let value = ((color >> shift) & 255_u32) as f64 / 255.;
            if value <= 0.04045 {
                value / 12.92
            } else {
                ((value + 0.055) / 1.055).powf(2.4)
            }
        };
        0.2126 * channel(16) + 0.7152 * channel(8) + 0.0722 * channel(0)
    }
    let a = luminance(foreground);
    let b = luminance(background);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn wayfinding_and_semantic_text_meet_body_contrast() {
        for (ink, surface) in [
            (RAIL_INK, RAIL),
            (RAIL_INK, RAIL_HOVER),
            (RAIL_MUTED, RAIL),
            (RAIL, SELECTED),
            (INK, SURFACE),
            (MUTED, SURFACE),
            (MUTED, GROUND),
            (MUTED, ACTION_SURFACE),
            (INK, ACTION_HOVER),
            (SURFACE, ACCENT),
            (SURFACE, ACCENT_HOVER),
            (ACCENT_HOVER, SELECTED),
            (ERROR, ERROR_SURFACE),
            (WARNING, WARNING_SURFACE),
        ] {
            assert!(contrast(ink, surface) >= 4.5, "{ink:06x} on {surface:06x}");
        }
        assert_ne!(ERROR, ACCENT);
        assert_ne!(WARNING, ACCENT);
        assert_ne!(ERROR, WARNING);
        assert!(contrast(CONTROL_LINE, SURFACE) >= 3.);
        assert!(contrast(CONTROL_LINE, ACTION_SURFACE) >= 3.);
    }
}
