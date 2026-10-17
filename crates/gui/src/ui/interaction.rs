// SPDX-License-Identifier: MPL-2.0
//! Live per-window input authority, shared by retained native callbacks.
//! Programmatic hydration/semantic projection is deliberately not input.
use gpui::{App, Global, Window, WindowId};
use std::{
    cell::Cell,
    collections::HashMap,
    rc::{Rc, Weak},
};

#[derive(Default)]
struct Gates(HashMap<WindowId, Weak<State>>);
impl Global for Gates {}
struct State {
    allowed: Cell<bool>,
    epoch: Cell<u64>,
}
#[derive(Clone)]
pub struct InputGate(Rc<State>);
impl Default for InputGate {
    fn default() -> Self {
        Self(Rc::new(State {
            allowed: Cell::new(true),
            epoch: Cell::new(0),
        }))
    }
}
impl InputGate {
    pub fn set_allowed(&self, allowed: bool) {
        if self.0.allowed.replace(allowed) && !allowed {
            self.0.epoch.set(self.0.epoch.get().wrapping_add(1));
        }
    }
    pub fn allows(&self) -> bool {
        self.0.allowed.get()
    }
    pub fn bind(&self, window: &Window, cx: &mut App) {
        if cx.try_global::<Gates>().is_none() {
            cx.set_global(Gates::default());
        }
        let gates = &mut cx.global_mut::<Gates>().0;
        gates.retain(|_, state| state.strong_count() != 0);
        gates.insert(window.window_handle().window_id(), Rc::downgrade(&self.0));
    }
}
pub fn gate(window: &Window, cx: &App) -> Option<InputGate> {
    cx.try_global::<Gates>()?
        .0
        .get(&window.window_handle().window_id())?
        .upgrade()
        .map(InputGate)
}
pub fn allowed(window: &Window, cx: &App) -> bool {
    gate(window, cx).is_none_or(|g| g.allows())
}
pub fn epoch(window: &Window, cx: &App) -> u64 {
    gate(window, cx).map_or(0, |g| g.0.epoch.get())
}
