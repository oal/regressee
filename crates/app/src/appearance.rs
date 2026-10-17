// SPDX-License-Identifier: MPL-2.0
//! Independent preference owner: small local writes never wait behind a capture.
//! The UI only exchanges snapshots; filesystem access and shutdown waits stay off it.
use regressee_core::appearance::AppAppearance;
use regressee_storage::{AppPaths, AppPreferences};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub preference: AppAppearance,
    pub loaded: bool,
    pub saving: bool,
    pub error: Option<&'static str>,
}

struct State {
    snapshot: Snapshot,
    pending: Option<AppAppearance>,
    stopped: bool,
}

pub struct AppearanceSettings {
    state: Arc<(Mutex<State>, Condvar)>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl AppearanceSettings {
    pub fn start(paths: AppPaths) -> Self {
        let state = Arc::new((
            Mutex::new(State {
                snapshot: Snapshot {
                    preference: AppAppearance::System,
                    loaded: false,
                    saving: false,
                    error: None,
                },
                pending: None,
                stopped: false,
            }),
            Condvar::new(),
        ));
        let owner = state.clone();
        let thread = thread::spawn(move || {
            let loaded = AppPreferences::load(&paths);
            let (lock, changed) = &*owner;
            let mut state = lock.lock().unwrap();
            match loaded {
                Ok(preferences) => state.snapshot.preference = preferences.appearance,
                Err(_) => {
                    state.snapshot.error = Some(
                        "Could not read your appearance preference. Following System for now; choose an appearance to save it again.",
                    )
                }
            }
            state.snapshot.loaded = true;
            changed.notify_all();
            loop {
                while state.pending.is_none() && !state.stopped {
                    state = changed.wait(state).unwrap();
                }
                let Some(preference) = state.pending.take() else {
                    break;
                };
                drop(state);
                let result = AppPreferences {
                    appearance: preference,
                }
                .save(&paths);
                state = lock.lock().unwrap();
                // A newer choice is already visible; an older completion must not
                // change it, announce it as saved, or hide its pending state.
                if state.pending.is_none() {
                    state.snapshot.saving = false;
                    state.snapshot.error = result.err().map(|_| "Appearance changed for this session, but could not be saved. Check available space and folder permissions, then select it again to retry.");
                }
                changed.notify_all();
            }
        });
        Self {
            state,
            thread: Mutex::new(Some(thread)),
        }
    }

    pub fn snapshot(&self) -> Snapshot {
        self.state.0.lock().unwrap().snapshot.clone()
    }

    /// Startup/headless only: before the event loop, never from a UI callback.
    pub fn wait_loaded(&self) -> Snapshot {
        let (lock, changed) = &*self.state;
        let state = changed
            .wait_while(lock.lock().unwrap(), |state| !state.snapshot.loaded)
            .unwrap();
        state.snapshot.clone()
    }

    pub fn choose(&self, preference: AppAppearance) {
        let (lock, changed) = &*self.state;
        let mut state = lock.lock().unwrap();
        if !state.snapshot.loaded || state.stopped {
            return;
        }
        state.snapshot.preference = preference;
        state.snapshot.saving = true;
        state.snapshot.error = None;
        state.pending = Some(preference);
        changed.notify_one();
    }

    pub fn stop(&self) {
        let (lock, changed) = &*self.state;
        lock.lock().unwrap().stopped = true;
        changed.notify_one();
    }

    /// Flush the final choice after the UI has exited, including an immediate quit.
    pub fn join_after_ui(&self) {
        self.stop();
        if let Some(thread) = self.thread.lock().unwrap().take() {
            let _ = thread.join();
        }
    }
}

impl Drop for AppearanceSettings {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rapid_changes_flush_the_last_choice_and_survive_restart() {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(root.path());
        let owner = AppearanceSettings::start(paths.clone());
        assert_eq!(owner.wait_loaded().preference, AppAppearance::System);
        for _ in 0..20 {
            for preference in AppAppearance::ALL {
                owner.choose(preference);
            }
        }
        owner.choose(AppAppearance::Light);
        owner.join_after_ui();
        assert_eq!(
            owner.snapshot(),
            Snapshot {
                preference: AppAppearance::Light,
                loaded: true,
                saving: false,
                error: None
            }
        );
        let restarted = AppearanceSettings::start(paths);
        assert_eq!(restarted.wait_loaded().preference, AppAppearance::Light);
        restarted.join_after_ui();
    }

    #[test]
    fn failed_write_keeps_the_visible_choice_and_reports_that_it_is_not_saved() {
        let root = tempfile::tempdir().unwrap();
        let paths = AppPaths::isolated(root.path());
        std::fs::write(&paths.data, b"not a directory").unwrap();
        let owner = AppearanceSettings::start(paths);
        assert!(owner.wait_loaded().error.is_some());
        owner.choose(AppAppearance::Dark);
        owner.join_after_ui();
        let snapshot = owner.snapshot();
        assert_eq!(snapshot.preference, AppAppearance::Dark);
        assert!(!snapshot.saving);
        assert!(snapshot.error.unwrap().contains("could not be saved"));
    }
}
