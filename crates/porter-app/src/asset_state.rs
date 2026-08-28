use std::collections::HashMap;
use std::sync::Arc;
use std::sync::RwLock;

use crate::Asset;

/// Implemented on the asset state inside the asset manager.
pub trait AssetState<A: Asset>: Send + Sync + Clone {
    /// Gets the search asset state.
    fn search_assets(&self) -> &Arc<RwLock<Option<Vec<usize>>>>;
    /// Gets the render asset state.
    fn render_assets(&self) -> &Arc<RwLock<Vec<A::Hash>>>;
    /// Gets the loaded asset state.
    fn loaded_assets(&self) -> &Arc<RwLock<HashMap<A::Hash, A>>>;
}
