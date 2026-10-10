// SPDX-License-Identifier: MPL-2.0
//! Cooperative cancellation at bounded I/O and row boundaries.
use std::{
    io::{self, Read},
    sync::atomic::{AtomicBool, Ordering},
};

pub fn check(cancel: &AtomicBool, operation: &str) -> anyhow::Result<()> {
    anyhow::ensure!(!cancel.load(Ordering::Relaxed), "{operation} cancelled");
    Ok(())
}

pub struct Cancellable<'a, R> {
    pub inner: R,
    pub cancel: &'a AtomicBool,
}
impl<R: Read> Read for Cancellable<'_, R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if self.cancel.load(Ordering::Relaxed) {
            return Err(io::Error::other("operation cancelled"));
        }
        let size = buf.len().min(65536);
        self.inner.read(&mut buf[..size])
    }
}
