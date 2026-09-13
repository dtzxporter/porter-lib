use std::path::PathBuf;

use iced::Color;

use porter_threads::IndexedParallelIterator;
use porter_threads::IntoParallelIterator;
use porter_threads::ParallelIterator;
use porter_threads::ParallelSlice;
use porter_threads::ParallelSliceMut;

use porter_utils::AtomicCancel;

use crate::Asset;
use crate::AssetContext;
use crate::AssetManager;
use crate::AssetPreview;
use crate::AssetState;
use crate::ColumnStatus;
use crate::Controller;
use crate::SearchTerm;
use crate::Settings;

/// Shared implementation tasks for asset managers.
pub trait AssetManagerTask: AssetManager {
    /// Implements the default logic for an asset managers `asset_info` method.
    fn assets_info_task<A, S>(&self, state: &S, index: usize) -> Vec<(String, Option<Color>)>
    where
        A: Asset,
        S: AssetState<A>;
    /// Implements the default logic for an asset managers `assets_visible` method.
    fn assets_visible_task<A, S>(&self, state: &S) -> usize
    where
        A: Asset,
        S: AssetState<A>;
    /// Implements the default logic for an asset managers `assets_total` method.
    fn assets_total_task<A, S>(&self, state: &S) -> usize
    where
        A: Asset,
        S: AssetState<A>;
    /// Implements the default logic for an asset managers `search` method.
    fn search_task<A, S>(&self, state: &S, term: Option<SearchTerm>)
    where
        A: Asset,
        S: AssetState<A>;
    /// Implements the default logic for an asset managers `sort` method.
    fn sort_task<A, S>(
        &self,
        state: &S,
        column: Option<usize>,
        statuses: Vec<ColumnStatus>,
    ) -> Vec<ColumnStatus>
    where
        A: Asset,
        S: AssetState<A>;
    /// Implements the default logic for an asset managers `export` method.
    fn export_task<A, S, C>(
        &self,
        state: &S,
        settings: Settings,
        controller: Controller,
        cancel: &AtomicCancel,
        assets: Vec<usize>,
        callback: C,
    ) where
        A: Asset,
        S: AssetState<A>,
        C: Fn(&A, &AssetContext<A, S>) -> bool,
        C: Send + Sync;
    /// Implements the default logic for an asset managers `preview` method.
    #[allow(clippy::too_many_arguments)]
    fn preview_task<A, S, C, E>(
        &self,
        state: &S,
        settings: Settings,
        controller: Controller,
        asset: usize,
        raw: bool,
        request_id: u64,
        callback: C,
    ) where
        A: Asset,
        S: AssetState<A>,
        C: FnOnce(&A, &AssetContext<A, S>, String, bool) -> Result<AssetPreview, E>,
        C: Send + Sync;
}

impl<T> AssetManagerTask for T
where
    T: AssetManager,
{
    fn assets_info_task<A, S>(&self, state: &S, index: usize) -> Vec<(String, Option<Color>)>
    where
        A: Asset,
        S: AssetState<A>,
    {
        let search = state.search_assets().read().unwrap();
        let render = state.render_assets().read().unwrap();
        let loaded = state.loaded_assets().read().unwrap();

        let asset = if let Some(search) = search.as_ref() {
            &render[search[index]]
        } else {
            &render[index]
        };

        let asset = &loaded[asset];

        vec![
            (asset.name(), None),
            (asset.type_name().to_owned(), Some(asset.color())),
            (asset.status().to_string(), Some(asset.status().color())),
            (asset.info(), None),
        ]
    }

    fn assets_visible_task<A, S>(&self, state: &S) -> usize
    where
        A: Asset,
        S: AssetState<A>,
    {
        let search_assets = state.search_assets();
        let render_assets = state.render_assets();

        if let Some(search) = search_assets.read().unwrap().as_ref() {
            search.len()
        } else {
            render_assets.read().unwrap().len()
        }
    }

    fn assets_total_task<A, S>(&self, state: &S) -> usize
    where
        A: Asset,
        S: AssetState<A>,
    {
        state
            .render_assets()
            .read()
            .unwrap()
            .len()
    }

    fn search_task<A, S>(&self, state: &S, term: Option<SearchTerm>)
    where
        A: Asset,
        S: AssetState<A>,
    {
        let search_assets = state.search_assets();
        let render_assets = state.render_assets();
        let loaded_assets = state.loaded_assets();

        let Some(term) = term else {
            *search_assets.write().unwrap() = None;
            return;
        };

        let assets = render_assets.read().unwrap();
        let loaded_assets = loaded_assets.read().unwrap();

        let results = assets
            .as_parallel_slice()
            .into_par_iter()
            .enumerate()
            .filter_map(|(index, asset)| {
                let asset = loaded_assets.get(asset).unwrap();

                if term.matches(asset.search()) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();

        *search_assets.write().unwrap() = Some(results);
    }

    fn sort_task<A, S>(
        &self,
        state: &S,
        _column: Option<usize>,
        statuses: Vec<ColumnStatus>,
    ) -> Vec<ColumnStatus>
    where
        A: Asset,
        S: AssetState<A>,
    {
        let mut render_assets = state
            .render_assets()
            .read()
            .unwrap()
            // We don't want to block the ui thread while sorting, so we have to clone the list.
            .clone();

        let loaded_assets = state.loaded_assets().read().unwrap();

        render_assets.par_sort_by_key(|key| {
            let asset = loaded_assets.get(key).unwrap();

            (asset.type_name(), asset.name())
        });

        *state.render_assets().write().unwrap() = render_assets;

        statuses
    }

    fn export_task<A, S, C>(
        &self,
        state: &S,
        settings: Settings,
        controller: Controller,
        cancel: &AtomicCancel,
        assets: Vec<usize>,
        callback: C,
    ) where
        A: Asset,
        S: AssetState<A>,
        C: Fn(&A, &AssetContext<A, S>) -> bool,
        C: Send + Sync,
    {
        AssetContext::new(settings, controller, state.clone())
            .into_exporter(callback)
            .run(assets, cancel)
    }

    fn preview_task<A, S, C, E>(
        &self,
        state: &S,
        settings: Settings,
        controller: Controller,
        asset: usize,
        raw: bool,
        request_id: u64,
        callback: C,
    ) where
        A: Asset,
        S: AssetState<A>,
        C: FnOnce(&A, &AssetContext<A, S>, String, bool) -> Result<AssetPreview, E>,
        C: Send + Sync,
    {
        let search = state.search_assets().read().unwrap();
        let render = state.render_assets().read().unwrap();
        let loaded = state.loaded_assets().read().unwrap();

        let asset = if let Some(search) = search.as_ref() {
            &render[search[asset]]
        } else {
            &render[asset]
        };

        let asset = &loaded[asset];

        if !asset.status().is_available() {
            controller.preview_update(request_id, AssetPreview::NotSupported);
            return;
        }

        let file_name = PathBuf::from(asset.name())
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .into_owned();

        let context = AssetContext::new(settings, controller, state.clone());

        let Ok(preview) = callback(asset, &context, file_name, raw) else {
            #[cfg(debug_assertions)]
            println!("Failed to preview asset: \"{}\"", asset.name());

            return context
                .controller
                .preview_update(request_id, AssetPreview::PreviewError);
        };

        context
            .controller
            .preview_update(request_id, preview);
    }
}
