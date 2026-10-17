// SPDX-License-Identifier: MPL-2.0
//! Native screenshot policy on the application's single blocking storage owner.
#[cfg(test)]
use regressee_app::jobs::StorageFailure;
use regressee_app::jobs::{BackgroundJobs, Extension};
#[cfg(any(test, feature = "test-support"))]
use regressee_app::product::Product;
#[cfg(test)]
use regressee_core::domain::{ProjectId, RunId};
use regressee_storage::{AppPaths, Store};
use std::{
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver, SyncSender, TrySendError},
    },
    thread,
    time::Duration,
};

#[cfg(test)]
use crate::jobs::Reply;
use crate::jobs::{Request, Update, ViewerFrame, ViewerWork, Work};

#[derive(Default)]
pub struct ViewerMailbox {
    generation: u64,
    latest: Option<(u64, Request)>,
    prefetch: Option<Work>,
}
impl ViewerMailbox {
    fn submit(&mut self, request: Request) -> Result<(), &'static str> {
        if !matches!(&request.work, Work::Extension(_)) {
            return Err("Viewer navigation only accepts screenshot reads.");
        }
        self.generation = self
            .generation
            .checked_add(1)
            .ok_or("Viewer generation exhausted; restart Regressee.")?;
        self.latest = Some((self.generation, request));
        self.prefetch = None;
        Ok(())
    }
    fn take(&mut self) -> Option<(Request, bool, Option<u64>)> {
        if let Some((generation, request)) = self.latest.take() {
            Some((request, false, Some(generation)))
        } else {
            self.prefetch
                .take()
                .map(|work| (Request { id: 0, work }, true, None))
        }
    }
}

#[derive(Default)]
pub struct ViewerExtension(crate::viewer_cache::ViewerCache);
impl Extension for ViewerExtension {
    type Work = ViewerWork;
    type Reply = Box<ViewerFrame>;
    type Shared = Mutex<ViewerMailbox>;
    fn submit(
        shared: &Self::Shared,
        request: Request,
        stop: &AtomicBool,
    ) -> Result<(), &'static str> {
        let mut viewer = shared.lock().unwrap();
        if stop.load(Ordering::Acquire) {
            return Err("Background storage has stopped. Restart Regressee.");
        }
        viewer.submit(request)
    }
    fn take(shared: &Self::Shared) -> Option<(Request, bool, Option<u64>)> {
        shared.lock().unwrap().take()
    }
    fn before_work(&mut self, shared: &Self::Shared, work: &Work) {
        if invalidates_viewer(work) {
            self.0.invalidate();
            shared.lock().unwrap().prefetch = None;
        }
    }
    fn execute(
        &mut self,
        store: &mut Store,
        work: ViewerWork,
        speculative: bool,
    ) -> anyhow::Result<Box<ViewerFrame>> {
        let frame = match work {
            ViewerWork::ExactViewer(selection) => {
                let frame = crate::viewer_cache::ViewerCache::default().historical_frame(
                    store,
                    selection.project,
                    selection
                        .run
                        .ok_or_else(|| anyhow::anyhow!("Exact run required"))?,
                    selection.tile,
                    selection.key,
                )?;
                anyhow::ensure!(
                    frame.analysis == selection.analysis,
                    "Exact analysis unavailable in local test adapter"
                );
                frame
            }
            ViewerWork::Viewer {
                project,
                tile,
                key,
                analysis,
            } if speculative => self.0.prefetch(store, project, tile, key, analysis)?,
            ViewerWork::Viewer {
                project,
                tile,
                key,
                analysis,
            } => self.0.frame(store, project, tile, key, analysis)?,
            ViewerWork::HistoricalViewer {
                project,
                run,
                tile,
                key,
            } => crate::viewer_cache::ViewerCache::default()
                .historical_frame(store, project, run, tile, key)?,
        };
        Ok(Box::new(frame))
    }
    fn publish(
        sender: &SyncSender<Update>,
        update: Update,
        stop: &AtomicBool,
        shared: &Self::Shared,
        generation: u64,
    ) -> bool {
        publish_viewer(sender, update, stop, shared, generation)
    }
    fn stop(shared: &Self::Shared) {
        let mut viewer = shared.lock().unwrap();
        viewer.latest = None;
        viewer.prefetch = None;
    }
}

pub struct Jobs {
    pub appearance: regressee_app::appearance::AppearanceSettings,
    owner: Arc<BackgroundJobs<ViewerExtension>>,
}
impl std::ops::Deref for Jobs {
    type Target = BackgroundJobs<ViewerExtension>;
    fn deref(&self) -> &Self::Target {
        &self.owner
    }
}
impl Jobs {
    pub fn start(paths: AppPaths) -> (Arc<Self>, Receiver<Update>) {
        Self::wrap(paths, BackgroundJobs::start)
    }
    pub fn start_with_distribution(
        paths: AppPaths,
        distribution: regressee_runtime::distribution::Distribution,
    ) -> (Arc<Self>, Receiver<Update>) {
        Self::wrap(paths, |paths| {
            BackgroundJobs::start_with_distribution(paths, distribution)
        })
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn start_with_product(paths: AppPaths, product: Product) -> (Arc<Self>, Receiver<Update>) {
        Self::start_inner(paths, move |_| {
            if let Some(d) = &product.distribution {
                d.validate()?;
            }
            Ok(product)
        })
    }
    #[cfg(any(test, feature = "test-support"))]
    pub fn start_without_runtime(paths: AppPaths) -> (Arc<Self>, Receiver<Update>) {
        let product = Product {
            distribution: None,
            official_runtime: false,
            runtime_cache: paths.cache.join("product-runtime"),
            auth: Default::default(),
        };
        Self::start_with_product(paths, product)
    }
    #[cfg(any(test, feature = "test-support"))]
    fn start_inner(
        paths: AppPaths,
        configure: impl FnOnce(&AppPaths) -> anyhow::Result<Product> + Send + 'static,
    ) -> (Arc<Self>, Receiver<Update>) {
        Self::wrap(paths, |paths| {
            BackgroundJobs::start_with_factory(paths, configure)
        })
    }
    fn wrap(
        paths: AppPaths,
        start: impl FnOnce(AppPaths) -> (Arc<BackgroundJobs<ViewerExtension>>, Receiver<Update>),
    ) -> (Arc<Self>, Receiver<Update>) {
        let appearance = regressee_app::appearance::AppearanceSettings::start(paths.clone());
        let (owner, events) = start(paths);
        (Arc::new(Self { appearance, owner }), events)
    }
    pub fn submit_viewer(&self, request: Request) -> Result<(), &'static str> {
        if !matches!(&request.work, Work::Extension(_)) {
            return Err("Viewer navigation only accepts screenshot reads.");
        }
        self.owner.submit(request)
    }
    pub fn prefetch_viewer(&self, work: Work) -> Result<(), &'static str> {
        if !matches!(&work, Work::Extension(ViewerWork::Viewer { .. })) {
            return Err("Viewer prefetch only accepts screenshot reads.");
        }
        // Share the stop guard with ordinary submissions before accepting speculation.
        self.owner.with_extension(|shared, stop| {
            let mut viewer = shared.lock().unwrap();
            if stop.load(Ordering::Acquire) {
                return Err("Background storage has stopped. Restart Regressee.");
            }
            viewer.prefetch = Some(work);
            Ok(())
        })
    }
    pub fn stop(&self) {
        self.appearance.stop();
        self.owner.stop();
    }
    pub fn join_after_ui(&self) {
        self.stop();
        self.appearance.join_after_ui();
        self.owner.join_after_ui();
    }
}
impl Drop for Jobs {
    fn drop(&mut self) {
        self.stop();
    }
}

#[cfg(test)]
#[path = "viewer_jobs_tests.rs"]
mod tests;
#[cfg(feature = "ui")]
fn invalidates_viewer(work: &Work) -> bool {
    matches!(
        work,
        Work::Evidence(_)
            | Work::Save { .. }
            | Work::SaveEnvironment { .. }
            | Work::SaveReplacementEnvironment { .. }
            | Work::SaveReplacement { .. }
            | Work::Activate { .. }
            | Work::Reanalyze { .. }
            | Work::Mask { .. }
            | Work::Review(_)
            | Work::LoadAnalysis(_)
            | Work::Accept { .. }
            | Work::Restore { .. }
            | Work::Capture { .. }
            | Work::CaptureInventory { .. }
            | Work::PreviewImages { .. }
            | Work::Import { .. }
            | Work::Delete(_)
            | Work::CleanupTemporaries(_)
            | Work::DeleteUnreferenced(_)
    )
}
#[cfg(test)]
fn next_viewer_work(
    commands: &Receiver<Request>,
    mailbox: &Mutex<ViewerMailbox>,
) -> Option<(Request, bool, Option<u64>)> {
    // Queued commands (particularly mutations) always precede speculation.
    match commands.try_recv() {
        Ok(request) => Some((request, false, None)),
        Err(_) => mailbox.lock().unwrap().take(),
    }
}
#[cfg(feature = "ui")]
fn publish_viewer(
    sender: &SyncSender<Update>,
    mut update: Update,
    stop: &AtomicBool,
    mailbox: &Mutex<ViewerMailbox>,
    generation: u64,
) -> bool {
    while !stop.load(Ordering::Acquire) {
        // Hold the slot lock through try_send: a newer accepted request cannot
        // race a stale publication. Already-published replies still carry id.
        let viewer = mailbox.lock().unwrap();
        if viewer.generation != generation {
            return true;
        }
        match sender.try_send(update) {
            Ok(()) => return true,
            Err(TrySendError::Full(value)) => update = value,
            Err(TrySendError::Disconnected(_)) => return false,
        }
        drop(viewer);
        thread::sleep(Duration::from_millis(10));
    }
    false
}
