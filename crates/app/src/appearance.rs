// SPDX-License-Identifier: MPL-2.0
//! Preferences off the UI thread. Production uses the engine client provider;
//! the local provider remains for explicit in-process tests/developer actors.
use regressee_core::appearance::AppAppearance;
use regressee_storage::{AppPaths, AppPreferences, PreferencesUpdate};
use std::{
    sync::{Arc, Condvar, Mutex},
    thread::{self, JoinHandle},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Snapshot {
    pub preference: AppAppearance,
    pub reduced_motion: bool,
    pub loaded: bool,
    pub saving: bool,
    pub error: Option<&'static str>,
}

struct State {
    snapshot: Snapshot,
    pending: bool,
    dirty: PreferencesUpdate,
    generation: u64,
    appearance_generation: u64,
    motion_generation: u64,
    stopped: bool,
}

pub struct AppearanceSettings {
    state: Arc<(Mutex<State>, Condvar)>,
    thread: Mutex<Option<JoinHandle<()>>>,
}

impl AppearanceSettings {
    pub fn start(paths: AppPaths) -> Self {
        let read = paths.clone();
        Self::with_provider(
            move || AppPreferences::load(&read),
            move |update| AppPreferences::update(&paths, update).map(|_| ()),
            false,
        )
    }
    pub fn start_client(client: crate::engine::Client) -> Self {
        let read = client.clone();
        Self::with_provider(
            move || match read.request(crate::protocol::Request::AppSettings)? {
                crate::protocol::Response::AppSettings(p) => Ok(p),
                _ => anyhow::bail!("Engine preferences unavailable"),
            },
            move |update| match client.request(crate::protocol::Request::SetPreferences(update))? {
                crate::protocol::Response::AppSettings(_) => Ok(()),
                _ => anyhow::bail!("Engine preference save unavailable"),
            },
            true,
        )
    }
    fn with_provider(
        load: impl Fn() -> anyhow::Result<AppPreferences> + Send + 'static,
        save: impl Fn(PreferencesUpdate) -> anyhow::Result<()> + Send + 'static,
        poll: bool,
    ) -> Self {
        let state = Arc::new((
            Mutex::new(State {
                snapshot: Snapshot {
                    preference: AppAppearance::System,
                    reduced_motion: false,
                    loaded: false,
                    saving: false,
                    error: None,
                },
                pending: false,
                dirty: PreferencesUpdate::default(),
                generation: 0,
                appearance_generation: 0,
                motion_generation: 0,
                stopped: false,
            }),
            Condvar::new(),
        ));
        let owner = state.clone();
        let thread = thread::spawn(move || {
            let loaded = load();
            let (lock, changed) = &*owner;
            let mut state = lock.lock().unwrap();
            match loaded {
                Ok(preferences) => {
                    state.snapshot.preference = preferences.appearance;
                    state.snapshot.reduced_motion = preferences.reduced_motion;
                }
                Err(_) => {
                    state.snapshot.error = Some(
                        "Could not read app preferences. Using System appearance and Normal motion for now (unless motion is forced by the environment). Choose either setting to save preferences again.",
                    )
                }
            }
            state.snapshot.loaded = true;
            changed.notify_all();
            loop {
                while !state.pending && !state.stopped {
                    if !poll {
                        state = changed.wait(state).unwrap();
                        continue;
                    }
                    let (next, timeout) = changed
                        .wait_timeout(state, std::time::Duration::from_millis(500))
                        .unwrap();
                    state = next;
                    if timeout.timed_out() && !state.stopped && !state.pending {
                        let generation = state.generation;
                        drop(state);
                        let latest = load();
                        state = lock.lock().unwrap();
                        // A failed local save stays visible until retried. Never
                        // replace a newer optimistic choice with an older poll.
                        if state.generation == generation
                            && !state.pending
                            && !state.snapshot.saving
                            && state.snapshot.error.is_none()
                            && let Ok(preferences) = latest
                        {
                            state.snapshot.preference = preferences.appearance;
                            state.snapshot.reduced_motion = preferences.reduced_motion;
                        }
                    }
                }
                if !state.pending {
                    break;
                }
                state.pending = false;
                let update = state.dirty;
                let generation = state.generation;
                drop(state);
                let result = save(update);
                state = lock.lock().unwrap();
                // A successful older save need not be replayed when only the
                // other setting changed meanwhile. Keep newer same-field intent
                // and failed fields dirty for the next save/retry.
                if result.is_ok() {
                    if state.appearance_generation <= generation {
                        state.dirty.appearance = None;
                    }
                    if state.motion_generation <= generation {
                        state.dirty.reduced_motion = None;
                    }
                }
                // A newer choice is already visible; an older completion must not
                // change it, announce it as saved, or hide its pending state.
                if state.generation == generation {
                    state.snapshot.saving = false;
                    state.snapshot.error = result.err().map(|_| "App preferences changed for this session, but could not be saved. Check available space and folder permissions, then select either setting again to retry.");
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
        self.choose_update(PreferencesUpdate {
            appearance: Some(preference),
            ..Default::default()
        });
    }

    pub fn choose_reduced_motion(&self, reduced_motion: bool) {
        self.choose_update(PreferencesUpdate {
            reduced_motion: Some(reduced_motion),
            ..Default::default()
        });
    }

    fn choose_update(&self, update: PreferencesUpdate) {
        let (lock, changed) = &*self.state;
        let mut state = lock.lock().unwrap();
        if !state.snapshot.loaded || state.stopped {
            return;
        }
        state.generation += 1;
        if let Some(preference) = update.appearance {
            state.snapshot.preference = preference;
            state.dirty.appearance = Some(preference);
            state.appearance_generation = state.generation;
        }
        if let Some(reduced_motion) = update.reduced_motion {
            state.snapshot.reduced_motion = reduced_motion;
            state.dirty.reduced_motion = Some(reduced_motion);
            state.motion_generation = state.generation;
        }
        state.snapshot.saving = true;
        state.snapshot.error = None;
        state.pending = true;
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
                owner.choose_reduced_motion(preference == AppAppearance::Dark);
            }
        }
        owner.choose(AppAppearance::Light);
        owner.choose_reduced_motion(true);
        owner.join_after_ui();
        assert_eq!(
            owner.snapshot(),
            Snapshot {
                preference: AppAppearance::Light,
                reduced_motion: true,
                loaded: true,
                saving: false,
                error: None
            }
        );
        let restarted = AppearanceSettings::start(paths);
        let snapshot = restarted.wait_loaded();
        assert_eq!(snapshot.preference, AppAppearance::Light);
        assert!(snapshot.reduced_motion);
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
        owner.choose_reduced_motion(true);
        owner.join_after_ui();
        let snapshot = owner.snapshot();
        assert_eq!(snapshot.preference, AppAppearance::Dark);
        assert!(snapshot.reduced_motion);
        assert!(!snapshot.saving);
        assert!(snapshot.error.unwrap().contains("could not be saved"));
    }

    #[test]
    fn stale_failed_save_cannot_replace_newer_choices_or_announce_them_saved() {
        use std::sync::mpsc;
        let (started, saves) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let gate = Mutex::new(gate);
        let owner = AppearanceSettings::with_provider(
            || Ok(AppPreferences::default()),
            move |update| {
                started.send(update).unwrap();
                if gate.lock().unwrap().recv().unwrap() {
                    Ok(())
                } else {
                    anyhow::bail!("synthetic failure")
                }
            },
            false,
        );
        owner.wait_loaded();
        owner.choose(AppAppearance::Dark);
        assert_eq!(saves.recv().unwrap().appearance, Some(AppAppearance::Dark));
        owner.choose_reduced_motion(true);
        owner.choose(AppAppearance::Light);
        release.send(false).unwrap();
        let latest = saves.recv().unwrap();
        assert_eq!(latest.appearance, Some(AppAppearance::Light));
        assert_eq!(latest.reduced_motion, Some(true));
        let snapshot = owner.snapshot();
        assert_eq!(snapshot.preference, AppAppearance::Light);
        assert!(snapshot.reduced_motion && snapshot.saving);
        assert!(snapshot.error.is_none());
        release.send(true).unwrap();
        owner.join_after_ui();
        assert!(!owner.snapshot().saving);
        assert!(owner.snapshot().error.is_none());
    }

    #[test]
    fn failed_motion_save_is_retried_with_next_appearance_choice() {
        use std::sync::atomic::{AtomicBool, Ordering};
        let fail = Arc::new(AtomicBool::new(true));
        let saved = Arc::new(Mutex::new(PreferencesUpdate::default()));
        let writer_fail = fail.clone();
        let writer_saved = saved.clone();
        let owner = AppearanceSettings::with_provider(
            || Ok(AppPreferences::default()),
            move |update| {
                anyhow::ensure!(!writer_fail.load(Ordering::SeqCst), "synthetic failure");
                *writer_saved.lock().unwrap() = update;
                Ok(())
            },
            false,
        );
        owner.wait_loaded();
        owner.choose_reduced_motion(true);
        let (lock, changed) = &*owner.state;
        let state = changed
            .wait_while(lock.lock().unwrap(), |s| s.snapshot.saving)
            .unwrap();
        assert!(state.snapshot.reduced_motion);
        assert!(state.snapshot.error.unwrap().contains("could not be saved"));
        drop(state);
        fail.store(false, Ordering::SeqCst);
        owner.choose(AppAppearance::Dark);
        owner.join_after_ui();
        assert_eq!(
            *saved.lock().unwrap(),
            PreferencesUpdate {
                appearance: Some(AppAppearance::Dark),
                reduced_motion: Some(true)
            }
        );
        assert!(owner.snapshot().error.is_none());
    }

    #[test]
    fn successful_older_save_does_not_replay_appearance_when_only_motion_changed() {
        use std::sync::mpsc;
        let (started, saves) = mpsc::channel();
        let (release, gate) = mpsc::channel();
        let gate = Mutex::new(gate);
        let owner = AppearanceSettings::with_provider(
            || Ok(AppPreferences::default()),
            move |update| {
                started.send(update).unwrap();
                gate.lock().unwrap().recv().unwrap();
                Ok(())
            },
            false,
        );
        owner.wait_loaded();
        owner.choose(AppAppearance::Dark);
        assert_eq!(saves.recv().unwrap().appearance, Some(AppAppearance::Dark));
        owner.choose_reduced_motion(true);
        release.send(()).unwrap();
        assert_eq!(
            saves.recv().unwrap(),
            PreferencesUpdate {
                appearance: None,
                reduced_motion: Some(true)
            }
        );
        assert!(owner.snapshot().saving);
        release.send(()).unwrap();
        owner.join_after_ui();
        assert!(owner.snapshot().reduced_motion);
        assert_eq!(owner.snapshot().preference, AppAppearance::Dark);
    }
}
