use std::path::PathBuf;

use iced::Color;

use crate::PorterSettings;
use crate::PorterUI;

/// A unified asset trait used to normalize the information across games.
pub trait PorterAssetManager: Send + Sync + 'static {
    /// Returns the asset info in the form of the columns to render.
    fn asset_info(&self, row_index: usize, columns: usize) -> Vec<(String, Option<Color>)>;

    /// Returns the number of assets renderable, as in search for, or loaded.
    fn len(&self) -> usize;

    /// Whether or not the assets are empty.
    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    /// Returns the total number of assets loaded.
    fn loaded_len(&self) -> usize;

    /// Searches for assets, or resets the asset list when empty.
    fn search_assets(&self, search_term: String);

    /// Whether or not load files is supported.
    fn supports_load_files(&self) -> bool;

    /// Whether or not load game is supported.
    fn supports_load_game(&self) -> bool;

    /// Loads one or more given file in async.
    fn on_load_files(&self, settings: PorterSettings, files: Vec<PathBuf>) -> Result<(), String>;

    /// Loads a game's memory in async.
    fn on_load_game(&self, settings: PorterSettings) -> Result<(), String>;

    /// Exports a game's assets in async.
    fn on_export(&self, settings: PorterSettings, assets: Vec<usize>, ui: PorterUI);

    /// Loads a game's asset for previewing.
    fn on_preview(&self, settings: PorterSettings, asset: usize, request_id: u64, ui: PorterUI);

    /// Cancels an active export.
    fn cancel_export(&self);
}
