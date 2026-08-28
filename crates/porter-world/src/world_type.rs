/// Type of world data.
#[derive(Debug, Clone, Copy)]
pub enum WorldType {
    /// World static data.
    World,
    /// World dynamic data.
    WorldEntities,
    /// World prefab data.
    WorldPrefab,
}
