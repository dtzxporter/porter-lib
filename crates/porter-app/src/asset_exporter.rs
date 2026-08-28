use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;

use porter_utils::AtomicCancel;
use porter_utils::AtomicProgress;
use porter_utils::AtomicSemaphore;

use crate::Asset;
use crate::AssetContext;
use crate::AssetState;
use crate::AssetStatus;

/// An asset export job that processes all assets and dependencies.
pub struct AssetExporter<A: Asset, S: AssetState<A>, C> {
    pub(crate) context: AssetContext<A, S>,
    pub(crate) queue: Arc<Mutex<HashSet<A::Hash>>>,
    pub(crate) callback: C,
}

impl<A, S, C> AssetExporter<A, S, C>
where
    A: Asset,
    S: AssetState<A>,
    C: Fn(&A, &AssetContext<A, S>) -> bool,
    C: Send + Sync,
{
    /// Consumes the exporter and exports the given assets.
    pub fn run(mut self, assets: Vec<usize>, cancel: &AtomicCancel) {
        let this = &mut self;

        cancel.reset();

        let search = this.context.state.search_assets();
        let render = this.context.state.render_assets();
        let loaded = this.context.state.loaded_assets();

        let search = search.read().unwrap();
        let render = render.read().unwrap();

        let mut assets: Vec<A::Hash> = assets
            .into_iter()
            .rev()
            .map(|index| {
                if let Some(search) = search.as_ref() {
                    render[search[index]]
                } else {
                    render[index]
                }
            })
            .collect();

        drop(search);
        drop(render);

        let mut seen: HashSet<A::Hash> = assets
            .iter()
            .copied()
            // Insert the existing assets, so we don't export duplicates later.
            .collect();

        let progress = AtomicProgress::new();
        let semaphore = AtomicSemaphore::new();

        progress.reset(assets.len());

        let context = &this.context;
        let callback = &this.callback;

        let progress = &progress;
        let semaphore = &semaphore;

        let queue = &this.queue;

        porter_threads::scope(move |scope| {
            loop {
                while let Some(hash) = assets.pop() {
                    let lock = semaphore.wait();

                    scope.spawn(move |_| {
                        let loaded = loaded.read().unwrap();

                        let Some(asset) = loaded.get(&hash) else {
                            #[cfg(debug_assertions)]
                            println!("Failed to lookup asset for export: {hash:#02X?}");
                            context
                                .controller
                                .progress_update(progress.increment());
                            return drop(lock);
                        };

                        let status = asset.status();

                        if !status.is_available() {
                            context
                                .controller
                                .progress_update(progress.increment());
                            return drop(lock);
                        }

                        status.set(AssetStatus::EXPORTING);

                        context
                            .controller
                            .progress_update(progress.value());

                        if callback(asset, context) {
                            status.set(AssetStatus::EXPORTED);
                        } else {
                            #[cfg(debug_assertions)]
                            println!("Failed to export asset: \"{}\"", asset.name());
                            status.set(AssetStatus::ERROR);
                        }

                        context
                            .controller
                            .progress_update(progress.increment());
                    });

                    if cancel.is_cancelled() {
                        return;
                    }
                }

                let mut queue = queue.lock().unwrap();

                if queue.is_empty() {
                    if semaphore.is_idle() {
                        // No assets processing, no assets left to export.
                        return;
                    } else if assets.is_empty() {
                        // Assets are still processing, but we have nothing in the queue.
                        std::hint::spin_loop();
                    }
                } else {
                    // Queue the assets for export, and wait.
                    let new = std::mem::take(&mut *queue)
                        .difference(&seen)
                        .copied()
                        .collect::<Vec<_>>();

                    progress.add(new.len());

                    assets.extend(new.iter());

                    seen.extend(new);
                }

                drop(queue);

                if cancel.is_cancelled() {
                    return;
                }
            }
        });

        context.controller.progress_finish();
    }
}
