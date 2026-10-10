// SPDX-License-Identifier: MPL-2.0
//! Proposal palette mirrored in DESIGN.md; shared metrics live here.
//! One family, task-oriented light mode.
pub const GROUND: u32 = 0xf6f8f9;
pub const SURFACE: u32 = 0xffffff;
pub const RAIL: u32 = 0xedf2f4;
pub const INK: u32 = 0x183138;
pub const MUTED: u32 = 0x526970;
pub const LINE: u32 = 0xc8d4d8;
pub const CONTROL_LINE: u32 = 0x87999f;
pub const ACTION_SURFACE: u32 = 0xe2e9ec;
pub const ACTION_HOVER: u32 = 0xd3dfe3;
pub const ACCENT: u32 = 0x08786f;
pub const ACCENT_HOVER: u32 = 0x05645c;
pub const SELECTED: u32 = 0xd9f0ec;
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

pub const FONT: &str = if cfg!(target_os = "windows") {
    "Segoe UI"
} else if cfg!(target_os = "macos") {
    ".AppleSystemUIFont"
} else {
    "DejaVu Sans"
};
