// SPDX-License-Identifier: MPL-2.0
//! Startup-only renderer logging, not the sanitized project diagnostic export.
use std::sync::atomic::{AtomicBool, Ordering};

struct RendererLogger {
    verbose: AtomicBool,
}

static LOGGER: RendererLogger = RendererLogger {
    verbose: AtomicBool::new(false),
};

fn renderer_target(target: &str) -> bool {
    target == "blade_graphics::hal::init" || target.starts_with("blade_graphics::hal::init::")
}

fn enabled(target: &str, level: log::Level, verbose: bool) -> bool {
    renderer_target(target)
        && level
            <= if verbose {
                log::Level::Info
            } else {
                log::Level::Warn
            }
}

impl log::Log for RendererLogger {
    fn enabled(&self, metadata: &log::Metadata<'_>) -> bool {
        enabled(
            metadata.target(),
            metadata.level(),
            self.verbose.load(Ordering::Relaxed),
        )
    }

    fn log(&self, record: &log::Record<'_>) {
        if self.enabled(record.metadata()) {
            eprintln!("Renderer {}: {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

/// Only the pinned renderer's initialization module is enabled. Project,
/// browser, credentials, network, and arbitrary GPUI logs are never selected.
pub fn initialize(verbose: bool) {
    LOGGER.verbose.store(verbose, Ordering::Relaxed);
    if log::set_logger(&LOGGER).is_ok() {
        log::set_max_level(if verbose {
            log::LevelFilter::Info
        } else {
            log::LevelFilter::Warn
        });
    } else if verbose {
        eprintln!("Renderer diagnostics could not install a logger; another logger is active.");
    }
    if verbose {
        eprintln!(
            "Renderer diagnostics: GPUI 0.2.2 on {}; initialization information only, not a sanitized project export.",
            std::env::consts::OS
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialization_rejections_are_visible_without_verbose_logging() {
        assert!(enabled(
            "blade_graphics::hal::init",
            log::Level::Warn,
            false
        ));
        assert!(enabled(
            "blade_graphics::hal::init",
            log::Level::Error,
            false
        ));
        assert!(!enabled(
            "blade_graphics::hal::init",
            log::Level::Info,
            false
        ));
        assert!(!enabled(
            "blade_graphics::hal::init",
            log::Level::Debug,
            true
        ));
        assert!(enabled("blade_graphics::hal::init", log::Level::Info, true));
    }

    #[test]
    fn unrelated_log_targets_are_not_enabled() {
        for target in [
            "regressee_app::product",
            "regressee_app::auth",
            "regressee_storage",
            "reqwest",
            "gpui::window",
            "blade_graphics::hal::command",
            "blade_graphics::hal::init_other",
            "untrusted::blade_graphics::hal::init",
        ] {
            assert!(!enabled(target, log::Level::Error, true), "{target}");
        }
    }
}
