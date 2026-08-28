use std::fmt::Debug;
use std::hash::Hash;

use iced::Color;

use crate::AssetStatus;
use crate::SearchAsset;
use crate::palette;

/// Implemented on assets to allow them to be processed by the exporter.
pub trait Asset: Send + Sync {
    /// Hash key type for assets.
    type Hash: Hash + Eq + Copy + Debug + Send + Sync + 'static;

    /// Gets the name of this asset.
    fn name(&self) -> String;
    /// Gets the type name of this asset.
    fn type_name(&self) -> &'static str;
    /// Gets the color of this asset.
    fn color(&self) -> Color {
        palette::TEXT_COLOR_DEFAULT
    }
    /// Gets the current status of this asset.
    fn status(&self) -> &AssetStatus;
    /// Gets the hash of this asset, every asset must have a unique hash.
    fn hash(&self) -> Self::Hash;
    /// Gets the formatted information of this asset defaulting to "N/A".
    fn info(&self) -> String {
        String::from("N/A")
    }
    /// Gets the asset search information.
    fn search(&self) -> SearchAsset {
        SearchAsset::new(self.name())
    }
    /// Gets whether or not the current asset is newer than the given other asset.
    fn is_newer_than(&self, other: &Self) -> bool {
        let _ = other;

        true
    }
}
