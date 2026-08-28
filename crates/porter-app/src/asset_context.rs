use std::collections::HashSet;
use std::marker::PhantomData;
use std::ops::Deref;
use std::sync::Arc;
use std::sync::Mutex;

use crate::Asset;
use crate::AssetExporter;
use crate::AssetState;
use crate::Controller;
use crate::Settings;

/// Context for all asset operations.
pub struct AssetContext<A: Asset, S: AssetState<A>> {
    pub(crate) settings: Settings,
    pub(crate) controller: Controller,
    pub(crate) queue: Option<Arc<Mutex<HashSet<A::Hash>>>>,
    pub(crate) state: S,
    _phantom: PhantomData<A>,
}

impl<A, S> AssetContext<A, S>
where
    A: Asset,
    S: AssetState<A>,
{
    /// Constructs a new asset context with the given settings, controller, and custom state.
    pub const fn new(settings: Settings, controller: Controller, state: S) -> Self {
        Self {
            settings,
            controller,
            queue: None,
            state,
            _phantom: PhantomData,
        }
    }

    /// Creates a new exporter with the given export callback.
    pub fn into_exporter<C>(mut self, callback: C) -> AssetExporter<A, S, C>
    where
        C: Fn(&A, &Self) -> bool,
        C: Send + Sync,
    {
        let queue: Arc<Mutex<HashSet<A::Hash>>> = Arc::new(Mutex::new(HashSet::new()));

        self.queue = Some(queue.clone());

        AssetExporter {
            context: self,
            queue,
            callback,
        }
    }

    /// Queues an asset to be exported on this current job.
    #[inline(never)]
    pub fn queue_asset_for_export<O>(&self, asset: &O)
    where
        O: Asset,
        O::Hash: Into<A::Hash>,
    {
        if let Some(queue) = &self.queue
            && let Ok(mut queue) = queue.lock()
        {
            queue.insert(asset.hash().into());
        }
    }

    /// Gets the controller.
    pub const fn controller(&self) -> &Controller {
        &self.controller
    }

    /// Gets the state.
    pub const fn state(&self) -> &S {
        &self.state
    }
}

impl<A, S> Deref for AssetContext<A, S>
where
    A: Asset,
    S: AssetState<A>,
{
    type Target = Settings;

    fn deref(&self) -> &Self::Target {
        &self.settings
    }
}
