// SPDX-License-Identifier: MPL-2.0
//! User-selected Clear wayfinding palette; shared control metrics live here.
//! Unified neutral shell in light and dark, with teal interaction and untinted evidence.
use std::cell::Cell;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Palette {
    pub ground: u32,
    /// Bounded review workspace only; other screens retain their existing ground.
    pub review_ground: u32,
    pub surface: u32,
    pub rail: u32,
    pub rail_ink: u32,
    pub rail_muted: u32,
    pub rail_hover: u32,
    pub nav_selected_ink: u32,
    pub ink: u32,
    pub muted: u32,
    pub line: u32,
    pub control_line: u32,
    pub action_surface: u32,
    pub action_hover: u32,
    pub accent: u32,
    pub accent_hover: u32,
    pub on_accent: u32,
    pub selected: u32,
    /// Result selection is stronger than a toggled toolbar/navigation surface.
    pub result_selected: u32,
    pub error: u32,
    pub error_surface: u32,
    pub warning: u32,
    pub warning_surface: u32,
    pub shadow: u32,
}

pub const LIGHT: Palette = Palette {
    ground: 0xf4f4f4,
    review_ground: 0xededed,
    surface: 0xffffff,
    rail: 0xeaeaea,
    rail_ink: 0x242424,
    rail_muted: 0x595959,
    rail_hover: 0xdedede,
    nav_selected_ink: 0x00685f,
    ink: 0x242424,
    muted: 0x595959,
    line: 0xd6d6d6,
    control_line: 0x858585,
    action_surface: 0xf8f8f8,
    action_hover: 0xe8e8e8,
    // The artwork's luminous turquoise is too light for white button text.
    accent: 0x007970,
    accent_hover: 0x00685f,
    on_accent: 0xffffff,
    selected: 0xe2f5f0,
    result_selected: 0xccebe3,
    error: 0x9c3038,
    error_surface: 0xfff2f2,
    warning: 0x775311,
    warning_surface: 0xfff7df,
    shadow: 0x181818,
};

pub const DARK: Palette = Palette {
    ground: 0x1b1b1b,
    review_ground: 0x111111,
    surface: 0x232323,
    rail: 0x202020,
    rail_ink: 0xededed,
    rail_muted: 0xb8b8b8,
    rail_hover: 0x303030,
    nav_selected_ink: 0xbceee4,
    ink: 0xededed,
    muted: 0xb8b8b8,
    line: 0x3d3d3d,
    control_line: 0x808080,
    action_surface: 0x2b2b2b,
    action_hover: 0x333333,
    accent: 0x65d9bf,
    accent_hover: 0x85e7d1,
    on_accent: 0x072c29,
    selected: 0x203e38,
    result_selected: 0x244c43,
    error: 0xffacb2,
    error_surface: 0x40282f,
    warning: 0xf1ca7d,
    warning_surface: 0x382d1b,
    shadow: 0x080808,
};

/// Source-only roles. Syntax describes structure, never review/diff status.
/// Selection/search use neutral foregrounds rather than mixing token colors.
#[derive(Clone, Copy, Debug)]
pub struct DomPalette {
    pub structure: u32,
    pub attribute: u32,
    pub value: u32,
    pub added: u32,
    pub removed: u32,
    pub selection: u32,
    pub search: u32,
}

pub const DOM_LIGHT: DomPalette = DomPalette {
    structure: 0x235d96,
    attribute: 0x6e468b,
    value: 0x77501f,
    added: 0xeff8f5,
    removed: LIGHT.error_surface,
    selection: 0xc5e9e2,
    search: 0xfff0bd,
};

pub const DOM_DARK: DomPalette = DomPalette {
    structure: 0x9ac7f5,
    attribute: 0xcdb2ea,
    value: 0xe2c28d,
    added: 0x24332f,
    removed: DARK.error_surface,
    selection: 0x315d52,
    search: 0x4b3c1e,
};

pub fn dom_palette() -> DomPalette {
    if is_dark() { DOM_DARK } else { DOM_LIGHT }
}

thread_local! {
    // GPUI paints all windows on its one UI thread. Keep this thread-local, not
    // process-global: independent test apps/renderer threads cannot recolor each
    // other. Context-free shared builders resolve the same semantic palette as
    // entity Render and canvas paint callbacks. Changes require a full refresh.
    static DARK_MODE: Cell<bool> = const { Cell::new(false) };
}

pub fn palette() -> Palette {
    if is_dark() { DARK } else { LIGHT }
}

pub fn is_dark() -> bool {
    DARK_MODE.get()
}

/// UI-thread only. Caller refreshes every entity, not just the shell, on change.
pub fn set_dark(dark: bool) -> bool {
    DARK_MODE.replace(dark) != dark
}

pub fn desktop_is_dark(appearance: gpui::WindowAppearance) -> bool {
    matches!(
        appearance,
        gpui::WindowAppearance::Dark | gpui::WindowAppearance::VibrantDark
    )
}

// Shared typography: pair each size with its leading rather than inheriting
// GPUI's 1.618 default (or a parent's absolute heading/control line height).
pub const HEADLINE_TEXT_SIZE: f32 = 28.;
pub const HEADLINE_LINE_HEIGHT: f32 = 36.;
pub const REVIEW_HEADING_TEXT_SIZE: f32 = 24.;
pub const REVIEW_HEADING_LINE_HEIGHT: f32 = 32.;
pub const TITLE_TEXT_SIZE: f32 = 21.;
pub const TITLE_LINE_HEIGHT: f32 = 28.;
pub const BODY_TEXT_SIZE: f32 = 15.;
pub const BODY_LINE_HEIGHT: f32 = 22.;

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
    fn dom_syntax_and_interaction_layers_meet_body_contrast_in_both_themes() {
        for (p, dom) in [(LIGHT, DOM_LIGHT), (DARK, DOM_DARK)] {
            for background in [p.surface, dom.added, dom.removed] {
                for ink in [p.ink, p.muted, dom.structure, dom.attribute, dom.value] {
                    assert!(
                        contrast(ink, background) >= 4.5,
                        "{ink:06x} on {background:06x}"
                    );
                }
            }
            for background in [dom.selection, dom.search] {
                assert!(contrast(p.ink, background) >= 4.5);
            }
            assert!(contrast(p.on_accent, p.accent_hover) >= 4.5);
            assert!(contrast(p.accent_hover, dom.added) >= 4.5);
            assert!(contrast(p.error, dom.removed) >= 4.5);
            assert_ne!(dom.selection, dom.added);
            assert_ne!(dom.search, dom.selection);
            assert_ne!(dom.search, dom.removed);
        }
    }

    #[test]
    fn both_shells_share_neutral_surfaces_and_ink_not_separate_hue_families() {
        for p in [LIGHT, DARK] {
            for color in [
                p.ground,
                p.review_ground,
                p.surface,
                p.rail,
                p.rail_hover,
                p.rail_ink,
                p.rail_muted,
                p.ink,
                p.muted,
                p.line,
                p.control_line,
                p.action_surface,
                p.action_hover,
                p.shadow,
            ] {
                let r = (color >> 16) & 255;
                let g = (color >> 8) & 255;
                let b = color & 255;
                assert_eq!((r, g), (g, b), "Shell color must be neutral: {color:06x}");
            }
            assert_eq!(p.rail_ink, p.ink);
            assert_eq!(p.rail_muted, p.muted);
            assert_ne!(p.ground, p.surface);
            assert_ne!(p.rail, p.ground);
            assert_ne!(p.accent, p.ink);
        }
    }

    #[test]
    fn wayfinding_and_semantic_text_meet_body_contrast() {
        for p in [LIGHT, DARK] {
            for (ink, surface) in [
                (p.rail_ink, p.rail),
                (p.rail_ink, p.rail_hover),
                (p.rail_muted, p.rail),
                (p.rail_muted, p.rail_hover),
                (p.nav_selected_ink, p.selected),
                (p.ink, p.surface),
                (p.muted, p.surface),
                (p.muted, p.ground),
                (p.muted, p.action_surface),
                (p.ink, p.action_hover),
                (p.muted, p.selected),
                (p.accent, p.surface),
                (p.accent_hover, p.surface),
                (p.accent, p.ground),
                (p.accent_hover, p.ground),
                (p.accent, p.selected),
                (p.on_accent, p.accent),
                (p.on_accent, p.accent_hover),
                (p.accent_hover, p.selected),
                (p.error, p.error_surface),
                (p.warning, p.warning_surface),
                (p.error, p.ground),
                (p.warning, p.ground),
            ] {
                assert!(contrast(ink, surface) >= 4.5, "{ink:06x} on {surface:06x}");
            }
            assert_ne!(p.error, p.accent);
            assert_ne!(p.warning, p.accent);
            assert_ne!(p.error, p.warning);
            assert!(contrast(p.control_line, p.surface) >= 3.);
            assert!(contrast(p.control_line, p.action_surface) >= 3.);
            assert!(contrast(p.accent, p.action_surface) >= 3.);
            assert!(contrast(p.accent, p.selected) >= 3.);
            assert!(contrast(p.accent, p.action_hover) >= 3.);
        }
    }

    #[test]
    fn review_hierarchy_preserves_readability_and_strengthens_selection() {
        for p in [LIGHT, DARK] {
            assert!(contrast(p.surface, p.review_ground) > contrast(p.surface, p.ground));
            assert!(contrast(p.result_selected, p.surface) > contrast(p.selected, p.surface));
            for ink in [p.ink, p.muted, p.accent, p.accent_hover] {
                assert!(contrast(ink, p.review_ground) >= 4.5);
            }
            for ink in [p.ink, p.muted, p.accent_hover] {
                assert!(contrast(ink, p.result_selected) >= 4.5);
            }
            assert!(contrast(p.accent, p.result_selected) >= 3.);
            assert!(contrast(p.control_line, p.review_ground) >= 3.);
        }
    }
}
