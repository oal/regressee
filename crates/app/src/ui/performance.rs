// SPDX-License-Identifier: MPL-2.0
//! Opt-in, data-free UI timing. Paint timing is CPU scene construction, not GPU
//! presentation or a substitute for inspecting the actual native frame.
use std::{
    sync::OnceLock,
    time::{Duration, Instant},
};

pub fn enabled() -> bool {
    static ENABLED: OnceLock<bool> = OnceLock::new();
    *ENABLED.get_or_init(|| {
        std::env::var_os("REGRESSEE_UI_DIAGNOSTICS").is_some_and(|value| value == "1")
    })
}

#[derive(Clone, Copy)]
pub struct FrameTiming {
    pub screen: &'static str,
    pub navigation: Instant,
    pub render: Instant,
    pub construction: Duration,
}

impl FrameTiming {
    pub fn report(&self, phase: &'static str) {
        eprintln!(
            "UI timing: screen={} phase={} navigation_ms={:.3} frame_ms={:.3} shell_construction_ms={:.3}",
            self.screen,
            phase,
            self.navigation.elapsed().as_secs_f64() * 1000.,
            self.render.elapsed().as_secs_f64() * 1000.,
            self.construction.as_secs_f64() * 1000.,
        );
    }
}
